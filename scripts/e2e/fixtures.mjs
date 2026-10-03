// Loopback stand-ins for the gear the end-to-end tour talks to: an HTTP API,
// a UDP and a TCP sink, an OSC device that answers /ping with /pong, a small
// MQTT 3.1.1 broker and a WebSocket echo service. Everything listens on
// 127.0.0.1 and counts what it received, so the runner can check the far end
// of every send, not only what the interface says it did. No dependencies.

import { createHash, randomBytes } from "node:crypto";
import { createSocket } from "node:dgram";
import { createServer } from "node:http";
import { createServer as createTcpServer } from "node:net";

/** A free loopback port for the given protocol (the OS picks; it is released at once). */
export function freePort(kind = "tcp") {
  return new Promise((resolve, reject) => {
    if (kind === "udp") {
      const socket = createSocket("udp4");
      socket.on("error", reject);
      socket.bind(0, "127.0.0.1", () => { const { port } = socket.address(); socket.close(() => resolve(port)); });
      return;
    }
    const server = createTcpServer();
    server.on("error", reject);
    server.listen(0, "127.0.0.1", () => { const { port } = server.address(); server.close(() => resolve(port)); });
  });
}

// ---- OSC, just enough for /ping ,s → /pong ,s ------------------------------

const pad4 = (length) => (length + 4) & ~3;

function oscString(text) {
  const bytes = Buffer.from(text, "utf8");
  const out = Buffer.alloc(pad4(bytes.length));
  bytes.copy(out);
  return out;
}

function readString(buffer, at) {
  const end = buffer.indexOf(0, at);
  if (end < 0) return null;
  return { text: buffer.toString("utf8", at, end), next: pad4(end - at) + at };
}

/** `{address, args}` with string, int and float arguments; null for anything else. */
export function decodeOsc(buffer) {
  const address = readString(buffer, 0);
  if (!address || !address.text.startsWith("/")) return null;
  const tags = readString(buffer, address.next);
  if (!tags || !tags.text.startsWith(",")) return null;
  let at = tags.next;
  const args = [];
  for (const tag of tags.text.slice(1)) {
    if (tag === "s") { const value = readString(buffer, at); if (!value) return null; args.push(value.text); at = value.next; }
    else if (tag === "i") { args.push(buffer.readInt32BE(at)); at += 4; }
    else if (tag === "f") { args.push(buffer.readFloatBE(at)); at += 4; }
    else return null;
  }
  return { address: address.text, args };
}

export function encodeOsc(address, strings) {
  return Buffer.concat([oscString(address), oscString(`,${"s".repeat(strings.length)}`), ...strings.map(oscString)]);
}

// ---- MQTT 3.1.1 broker --------------------------------------------------------

/** Remaining length as MQTT writes it: 7 bits per byte, high bit = more. */
function varint(length) {
  const bytes = [];
  do {
    let byte = length % 128;
    length = Math.floor(length / 128);
    if (length > 0) byte |= 0x80;
    bytes.push(byte);
  } while (length > 0);
  return Buffer.from(bytes);
}

const packet = (header, body) => Buffer.concat([Buffer.from([header]), varint(body.length), body]);
const u16 = (value) => Buffer.from([value >> 8, value & 0xff]);
const mqttString = (text) => { const bytes = Buffer.from(text, "utf8"); return Buffer.concat([u16(bytes.length), bytes]); };

/** `+` is one level, `#` the rest; `$` topics are never matched by a leading wildcard. */
export function topicMatches(filter, topic) {
  const f = filter.split("/");
  const t = topic.split("/");
  if (topic.startsWith("$") && (f[0] === "+" || f[0] === "#")) return false;
  for (let i = 0; i < f.length; i++) {
    if (f[i] === "#") return true;
    if (i >= t.length) return false;
    if (f[i] !== "+" && f[i] !== t[i]) return false;
  }
  return f.length === t.length;
}

/**
 * Clean sessions only. Every subscription is granted QoS 0 (a broker may
 * grant less than asked), so delivery is always QoS 0; a client's QoS 1 and 2
 * publishes get their PUBACK, or PUBREC then PUBCOMP.
 */
function startBroker(counts) {
  const clients = new Set();
  const retained = new Map();
  const deliver = (client, topic, payload, retain) =>
    client.socket.write(packet(0x30 | (retain ? 1 : 0), Buffer.concat([mqttString(topic), payload])));

  const handle = (client, type, flags, body) => {
    switch (type) {
      case 1: // CONNECT
        client.socket.write(packet(0x20, Buffer.from([0, 0])));
        break;
      case 3: { // PUBLISH
        const qos = (flags >> 1) & 3;
        const retain = (flags & 1) === 1;
        const length = body.readUInt16BE(0);
        const topic = body.toString("utf8", 2, 2 + length);
        let at = 2 + length;
        const id = qos > 0 ? body.readUInt16BE(at) : 0;
        if (qos > 0) at += 2;
        const payload = body.subarray(at);
        counts.mqttPublishes += 1;
        counts.mqttTopics.add(topic);
        if (retain) {
          if (payload.length === 0) retained.delete(topic);
          else retained.set(topic, payload);
        }
        for (const other of clients) {
          if (other.filters.some((filter) => topicMatches(filter, topic))) deliver(other, topic, payload, false);
        }
        if (qos === 1) client.socket.write(packet(0x40, u16(id)));
        if (qos === 2) client.socket.write(packet(0x50, u16(id)));
        break;
      }
      case 6: // PUBREL
        client.socket.write(packet(0x70, body.subarray(0, 2)));
        break;
      case 8: { // SUBSCRIBE
        const id = body.readUInt16BE(0);
        const granted = [];
        let at = 2;
        while (at < body.length) {
          const length = body.readUInt16BE(at);
          const filter = body.toString("utf8", at + 2, at + 2 + length);
          at += 2 + length + 1;
          if (!client.filters.includes(filter)) client.filters.push(filter);
          granted.push(0);
          counts.mqttSubscriptions.push(filter);
        }
        client.socket.write(packet(0x90, Buffer.concat([u16(id), Buffer.from(granted)])));
        for (const [topic, payload] of retained) {
          if (granted.length && client.filters.some((filter) => topicMatches(filter, topic))) deliver(client, topic, payload, true);
        }
        break;
      }
      case 10: { // UNSUBSCRIBE
        const id = body.readUInt16BE(0);
        let at = 2;
        while (at < body.length) {
          const length = body.readUInt16BE(at);
          const filter = body.toString("utf8", at + 2, at + 2 + length);
          at += 2 + length;
          client.filters = client.filters.filter((other) => other !== filter);
        }
        client.socket.write(packet(0xb0, u16(id)));
        break;
      }
      case 12: // PINGREQ
        client.socket.write(packet(0xd0, Buffer.alloc(0)));
        break;
      case 14: // DISCONNECT
        client.socket.end();
        break;
      default:
        break;
    }
  };

  const server = createTcpServer((socket) => {
    const client = { socket, filters: [], buffer: Buffer.alloc(0) };
    clients.add(client);
    counts.mqttConnections += 1;
    socket.on("data", (chunk) => {
      client.buffer = Buffer.concat([client.buffer, chunk]);
      for (;;) {
        if (client.buffer.length < 2) return;
        let length = 0;
        let multiplier = 1;
        let at = 1;
        for (;;) {
          if (at >= client.buffer.length) return;
          const byte = client.buffer[at++];
          length += (byte & 0x7f) * multiplier;
          multiplier *= 128;
          if ((byte & 0x80) === 0) break;
        }
        if (client.buffer.length < at + length) return;
        const header = client.buffer[0];
        const body = client.buffer.subarray(at, at + length);
        client.buffer = client.buffer.subarray(at + length);
        handle(client, header >> 4, header & 0x0f, body);
      }
    });
    const gone = () => clients.delete(client);
    socket.on("close", gone);
    socket.on("error", gone);
  });
  return { server, clients };
}

// ---- HTTP Digest (RFC 7616), checked here with hashing of its own: SHA-256, qop=auth ----

export const DIGEST = { realm: "tour", username: "tour", password: "e2e-secret" };

function digestParams(header) {
  const params = {};
  for (const match of header.matchAll(/(\w+)=(?:"((?:[^"\\]|\\.)*)"|([^,\s]+))/g)) params[match[1]] = match[2] ?? match[3];
  return params;
}

/** Whether `header` answers one of `nonces` for `method` and this request's `target`, as RFC 7616 computes it. */
function digestAnswered(header, method, target, nonces) {
  if (!header?.startsWith("Digest ")) return false;
  const p = digestParams(header.slice(7));
  if (!nonces.has(p.nonce) || p.uri !== target || p.username !== DIGEST.username || p.realm !== DIGEST.realm || p.qop !== "auth" || p.algorithm !== "SHA-256") return false;
  const h = (text) => createHash("sha256").update(text).digest("hex");
  const ha1 = h(`${p.username}:${p.realm}:${DIGEST.password}`);
  const ha2 = h(`${method}:${p.uri}`);
  return p.response === h(`${ha1}:${p.nonce}:${p.nc}:${p.cnonce}:${p.qop}:${ha2}`);
}

// ---- WebSocket (RFC 6455), just enough for an echo service ---------------------

/** A frame from the server: unmasked, unfragmented. */
function wsFrame(opcode, payload) {
  const length = payload.length;
  const head = length < 126 ? Buffer.from([0x80 | opcode, length])
    : length < 65536 ? Buffer.from([0x80 | opcode, 126, length >> 8, length & 0xff])
    : (() => { const big = Buffer.alloc(10); big[0] = 0x80 | opcode; big[1] = 127; big.writeBigUInt64BE(BigInt(length), 2); return big; })();
  return Buffer.concat([head, payload]);
}

/** The next whole frame of `buffer` (a client's: masked), or null while it is still arriving. */
function wsParse(buffer) {
  if (buffer.length < 2) return null;
  const opcode = buffer[0] & 0x0f;
  const masked = (buffer[1] & 0x80) !== 0;
  let length = buffer[1] & 0x7f;
  let at = 2;
  if (length === 126) { if (buffer.length < 4) return null; length = buffer.readUInt16BE(2); at = 4; }
  else if (length === 127) { if (buffer.length < 10) return null; length = Number(buffer.readBigUInt64BE(2)); at = 10; }
  const mask = masked ? buffer.subarray(at, at + 4) : null;
  if (masked) at += 4;
  if (buffer.length < at + length) return null;
  const payload = Buffer.from(buffer.subarray(at, at + length));
  if (mask) for (let i = 0; i < payload.length; i++) payload[i] ^= mask[i % 4];
  return { opcode, payload, used: at + length };
}

/**
 * An echo service: `welcome` on connecting, then every text and binary
 * message back as it came; pings answered, a close answered. It takes the
 * first subprotocol offered and keeps the headers of each upgrade.
 */
function startWebSocket(counts) {
  const server = createServer((request, response) => response.writeHead(426, { connection: "close" }).end("WebSocket only"));
  const sockets = new Set();
  server.on("upgrade", (request, socket) => {
    const key = request.headers["sec-websocket-key"];
    if (!key) { socket.destroy(); return; }
    const accept = createHash("sha1").update(`${key}258EAFA5-E914-47DA-95CA-C5AB0DC85B11`).digest("base64");
    const offered = String(request.headers["sec-websocket-protocol"] ?? "").split(",").map((name) => name.trim()).filter(Boolean);
    counts.wsUpgrades.push({ path: request.url, protocols: offered, headers: request.headers });
    socket.write(["HTTP/1.1 101 Switching Protocols", "Upgrade: websocket", "Connection: Upgrade", `Sec-WebSocket-Accept: ${accept}`,
      ...(offered.length ? [`Sec-WebSocket-Protocol: ${offered[0]}`] : []), "", ""].join("\r\n"));
    sockets.add(socket);
    socket.write(wsFrame(1, Buffer.from("welcome")));
    let pending = Buffer.alloc(0);
    socket.on("data", (chunk) => {
      pending = Buffer.concat([pending, chunk]);
      for (let frame = wsParse(pending); frame; frame = wsParse(pending)) {
        pending = pending.subarray(frame.used);
        if (frame.opcode === 1 || frame.opcode === 2) {
          counts.wsMessages.push(frame.opcode === 1 ? frame.payload.toString("utf8") : `binary ${frame.payload.toString("hex")}`);
          socket.write(wsFrame(frame.opcode, frame.payload));
        } else if (frame.opcode === 9) {
          socket.write(wsFrame(10, frame.payload));
        } else if (frame.opcode === 8) {
          counts.wsCloses.push(frame.payload.length >= 2 ? frame.payload.readUInt16BE(0) : 1005);
          socket.end(wsFrame(8, frame.payload.subarray(0, 2)));
        }
      }
    });
    const gone = () => sockets.delete(socket);
    socket.on("close", gone);
    socket.on("error", gone);
  });
  return { server, sockets };
}

// ---- everything together ------------------------------------------------------

const listen = (server, port = 0) => new Promise((resolve, reject) => {
  server.once("error", reject);
  server.listen(port, "127.0.0.1", () => resolve(server.address().port));
});
const bindUdp = (socket) => new Promise((resolve, reject) => {
  socket.once("error", reject);
  socket.bind(0, "127.0.0.1", () => resolve(socket.address().port));
});

/**
 * Starts every fixture. `pongPort` is where the OSC device sends its /pong
 * (the port the experiment's wait listens on).
 */
export async function startFixtures({ pongPort }) {
  const counts = {
    http: 0, httpPaths: [], udp: 0, udpBytes: 0, tcpConnections: 0, tcpBytes: 0,
    pings: 0, statusPolls: 0, mqttConnections: 0, mqttPublishes: 0, mqttTopics: new Set(), mqttSubscriptions: [],
    wsUpgrades: [], wsMessages: [], wsCloses: [],
    digestChallenges: 0, digestAnswered: 0, me: [],
  };
  const nonces = new Set();

  const http = createServer((request, response) => {
    const chunks = [];
    request.on("data", (chunk) => chunks.push(chunk));
    request.on("end", () => {
      counts.http += 1;
      counts.httpPaths.push(request.url);
      // Digest: a challenge, then the answer checked; /login sets a cookie that /me wants.
      if (request.url.startsWith("/digest")) {
        if (digestAnswered(request.headers.authorization, request.method, request.url, nonces)) {
          counts.digestAnswered += 1;
          response.writeHead(200, { "content-type": "application/json" }).end(JSON.stringify({ digest: "ok" }));
        } else {
          counts.digestChallenges += 1;
          const nonce = randomBytes(12).toString("hex");
          nonces.add(nonce);
          response.writeHead(401, { "www-authenticate": `Digest realm="${DIGEST.realm}", nonce="${nonce}", qop="auth", algorithm=SHA-256, opaque="tour"` }).end();
        }
        return;
      }
      if (request.url === "/login") {
        response.writeHead(200, { "content-type": "application/json", "set-cookie": "tour=1; Path=/; HttpOnly" }).end(JSON.stringify({ ok: true }));
        return;
      }
      if (request.url === "/me") {
        const known = /(^|;\s*)tour=1(;|$)/.test(request.headers.cookie ?? "");
        counts.me.push(known ? 200 : 401);
        response.writeHead(known ? 200 : 401, { "content-type": "application/json" }).end(JSON.stringify({ me: known }));
        return;
      }
      const body = Buffer.concat(chunks).toString("utf8");
      const reply = JSON.stringify({ ok: true, method: request.method, path: request.url, body, n: counts.http });
      response.writeHead(200, { "content-type": "application/json", "x-fixture": "signal-lab-e2e" }).end(reply);
    });
  });
  const httpPort = await listen(http);

  const sink = createSocket("udp4");
  sink.on("message", (message) => { counts.udp += 1; counts.udpBytes += message.length; });
  const sinkPort = await bindUdp(sink);

  const tcp = createTcpServer((socket) => {
    counts.tcpConnections += 1;
    socket.on("data", (chunk) => { counts.tcpBytes += chunk.length; });
    socket.on("error", () => {});
  });
  const tcpPort = await listen(tcp);

  // /ping → /pong to the wait's port; /status → "busy" twice, then "ready", to the sender.
  const device = createSocket("udp4");
  device.on("message", (message, from) => {
    const osc = decodeOsc(message);
    if (osc?.address === "/ping") {
      counts.pings += 1;
      const strings = osc.args.filter((arg) => typeof arg === "string");
      device.send(encodeOsc("/pong", strings), pongPort, "127.0.0.1");
    } else if (osc?.address === "/status") {
      counts.statusPolls += 1;
      device.send(encodeOsc("/status", [counts.statusPolls <= 2 ? "busy" : "ready"]), from.port, from.address);
    }
  });
  const devicePort = await bindUdp(device);

  const broker = startBroker(counts);
  const mqttPort = await listen(broker.server);

  const websocket = startWebSocket(counts);
  const wsPort = await listen(websocket.server);

  // Datagrams into a port of the app (the impairment relay), from a socket of their own.
  const sender = createSocket("udp4");
  await bindUdp(sender);
  const sendUdp = async (port, n, text = "e2e") => {
    for (let i = 0; i < n; i++) {
      await new Promise((resolve, reject) => sender.send(Buffer.from(`${text} ${i}`), port, "127.0.0.1", (error) => error ? reject(error) : resolve()));
    }
  };

  const close = () => Promise.all([
    new Promise((resolve) => { http.closeAllConnections(); http.close(resolve); }),
    new Promise((resolve) => tcp.close(resolve)),
    new Promise((resolve) => { for (const client of broker.clients) client.socket.destroy(); broker.server.close(resolve); }),
    new Promise((resolve) => { for (const socket of websocket.sockets) socket.destroy(); websocket.server.close(resolve); }),
    ...[sink, device, sender].map((socket) => new Promise((resolve) => socket.close(resolve))),
  ]);

  return { ports: { http: httpPort, sink: sinkPort, tcp: tcpPort, device: devicePort, mqtt: mqttPort, ws: wsPort, pong: pongPort }, counts, sendUdp, close };
}

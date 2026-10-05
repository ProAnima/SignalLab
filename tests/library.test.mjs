import test from "node:test";
import assert from "node:assert/strict";
import {
  addFolder, allFolders, ancestors, freeFolderName, libraryTree, moveFolder, moveSignal, normalizeFolder, removeFolder, renameFolder,
  canonicalBody, signalPlace,
  frameSignalBlock, frameSignalBody, isHostPort, makeId, mqttConnectionFor, reachable, replayTarget, sameBroker, signalFromFrame, splitBroker,
} from "../src/lib/library.ts";

const signal = (id, group) => ({ id, name: id, group, note: "", body: { transport: "osc", target: "127.0.0.1:9000", address: "/x", args: [] } });

test("folders are paths; a group from before folders is a top-level folder", () => {
  assert.equal(normalizeFolder("  API / Auth/ "), "API/Auth");
  assert.equal(normalizeFolder("//"), "");
  const signals = [signal("a", "OSC"), signal("b", "API/Auth"), signal("c", ""), signal("d", "API/Auth")];
  assert.deepEqual(allFolders(signals, ["Empty/Inner"]), ["API", "API/Auth", "Empty", "Empty/Inner", "OSC"]);
  const tree = libraryTree(signals, ["Empty/Inner"]);
  assert.deepEqual(tree.folders.map((node) => node.name), ["API", "Empty", "OSC"], "folders by name");
  assert.deepEqual(tree.signals.map((s) => s.id), ["c"], "the root holds what has no folder");
  assert.equal(tree.total, 4);
  const api = tree.folders[0];
  assert.equal(api.total, 2, "a folder counts what is inside its folders too");
  assert.deepEqual(api.folders[0].signals.map((s) => s.id), ["b", "d"], "signals keep file order");
  assert.equal(tree.folders[1].folders[0].path, "Empty/Inner");
  const numbered = libraryTree([signal("x", "Scene 10"), signal("y", "Scene 2")], []);
  assert.deepEqual(numbered.folders.map((node) => node.name), ["Scene 2", "Scene 10"], "numbers in order, not as text");
  assert.deepEqual(ancestors("A/B/C"), ["A", "A/B", "A/B/C"]);
});

test("new, renamed, moved and removed folders keep every signal", () => {
  let state = { signals: [signal("a", "API"), signal("b", "API/Auth"), signal("c", "")], folders: [] };
  assert.equal(freeFolderName(state.signals, state.folders, "", "API"), "API 2");
  state = addFolder(state, "Venue/Stage");
  assert.deepEqual(state.folders, ["API", "API/Auth", "Venue", "Venue/Stage"], "every folder is listed, the empty ones too");

  const renamed = renameFolder(state, "API", "Service");
  assert.deepEqual(renamed.signals.map((s) => s.group), ["Service", "Service/Auth", ""], "everything inside follows");
  assert.equal(renameFolder(state, "API", "Venue"), null, "a rename never merges into a sibling");
  assert.equal(renameFolder(state, "API", "a/b"), null);
  assert.equal(renameFolder(state, "API", "  "), null);

  const moved = moveFolder(state, "API/Auth", "Venue");
  assert.deepEqual(moved.signals.map((s) => s.group), ["API", "Venue/Auth", ""]);
  assert.deepEqual(moved.folders, ["API", "Venue", "Venue/Auth", "Venue/Stage"]);
  assert.equal(moveFolder(state, "API", "API/Auth"), null, "not into itself");

  const removed = removeFolder(state, "API");
  assert.deepEqual(removed.signals.map((s) => s.group), ["", "Auth", ""], "removing a folder moves what is in it up a level");
  const emptyGone = removeFolder(state, "Venue/Stage");
  assert.deepEqual(emptyGone.folders, ["API", "API/Auth", "Venue"], "its parent stays, now empty itself");

  const filed = moveSignal(state, "c", "Venue/Stage");
  assert.equal(filed.signals.find((s) => s.id === "c").group, "Venue/Stage");
  const emptied = moveSignal(filed, "b", "");
  assert.ok(emptied.folders.includes("API/Auth"), "a folder its last signal left stays");
});

test("a sender knows it holds what it was saved as, whatever order the engine sends fields in", () => {
  const typed = { transport: "http", request: { method: "GET", url: "http://127.0.0.1:8080/", headers: [["Accept", "application/json"]], body: null, timeout_ms: 10000 } };
  const fromEngine = { request: { body: null, headers: [["Accept", "application/json"]], method: "GET", timeout_ms: 10000, url: "http://127.0.0.1:8080/" }, transport: "http" };
  assert.equal(canonicalBody(typed), canonicalBody(fromEngine));
  assert.notEqual(canonicalBody(typed), canonicalBody({ ...fromEngine, request: { ...fromEngine.request, url: "http://127.0.0.1:8081/" } }));
  // OSC floats are stored at 32 bits: 0.1 comes back as the same float.
  const osc = (value) => ({ transport: "osc", target: "127.0.0.1:9000", address: "/x", args: [{ type: "float", value }] });
  assert.equal(canonicalBody(osc(0.1)), canonicalBody(osc(Math.fround(0.1))));
  assert.notEqual(canonicalBody(osc(0.1)), canonicalBody(osc(0.2)));
  assert.equal(signalPlace(signal("Login", " API / Auth ")), "API / Auth / Login");
  assert.equal(signalPlace(signal("Ping", "")), "Ping");
});

// ---- Save as signal: what a captured frame becomes -------------------------

const hexOf = (text) => [...Buffer.from(text)].map((byte) => byte.toString(16).padStart(2, "0")).join(" ");
/** A frame as the engine sends it, kept whole unless said otherwise. */
const frame = (fields = {}) => ({
  seq: 7, ts: 0, proto: "udp", dir: "rx", source: "osc-monitor", job_id: 1, local: "127.0.0.1:9000", remote: "127.0.0.1:50000",
  bytes: 4, summary: "ping", detail: null, hex: null, verdict: null, kept: 4, ...fields,
});
/** An MQTT publish frame (a sent one by default), `publish` overriding what it says of itself. */
const publishFrame = (fields = {}, publish = {}) => frame({
  proto: "mqtt", dir: "tx", source: "mqtt", local: "127.0.0.1:50001", remote: "127.0.0.1:1883", bytes: 2, kept: 2, summary: "lab/lamp = ON",
  publish: { broker: "127.0.0.1:1883", topic: "lab/lamp", qos: 1, retain: true, text: true, ...publish },
  ...fields,
});
const t = (key) => key;

test("a datagram received on every address is replayed to loopback on that port", () => {
  assert.equal(reachable("0.0.0.0:9000"), "127.0.0.1:9000");
  assert.equal(reachable("[::]:9000"), "[::1]:9000");
  assert.equal(reachable("192.0.2.5:9000"), "192.0.2.5:9000", "any other address stays");
  assert.equal(reachable("[2001:db8::1]:9000"), "[2001:db8::1]:9000");
  assert.equal(reachable("10.0.0.0.5:9000"), "10.0.0.0.5:9000", "only the unspecified address itself");

  const bytes = hexOf("ping");
  const body = (fields) => frameSignalBody(frame(fields), bytes);
  assert.deepEqual(body({ local: "0.0.0.0:9000" }), { transport: "udp", target: "127.0.0.1:9000", payload: { kind: "hex", hex: bytes } });
  assert.equal(body({ local: "[::]:9000" }).target, "[::1]:9000");
  assert.equal(body({ local: "192.0.2.5:9000" }).target, "192.0.2.5:9000", "received on one address: that one");
  assert.equal(body({ dir: "tx", local: "0.0.0.0:51234", remote: "192.0.2.9:9001", source: "osc-send" }).target, "192.0.2.9:9001", "a sent frame goes where it was sent");
  assert.equal(body({ dir: "tx", remote: "0.0.0.0:9001" }).target, "127.0.0.1:9001");
  // The bytes are the engine's, not the summary's.
  assert.equal(body({}).payload.hex, bytes);
  assert.equal(frameSignalBody(frame({ local: "" }), bytes), null, "nowhere to send it");
  assert.equal(frameSignalBlock(frame({ local: "" })), "ins.noSignalForm");
});

test("a relayed frame goes to the address it was going to, whichever way it went", () => {
  // The relay's listen address is the frame's local side; its peer is where the datagram was heading.
  const toTarget = frame({ source: "netsim", dir: "tx", local: "0.0.0.0:9010", remote: "192.0.2.20:9000", verdict: "forwarded +40ms · client→target" });
  const toClient = frame({ source: "netsim", dir: "rx", local: "0.0.0.0:9010", remote: "192.0.2.30:50123", verdict: "forwarded +40ms · target→client" });
  assert.equal(replayTarget(toTarget), "192.0.2.20:9000");
  assert.equal(replayTarget(toClient), "192.0.2.30:50123", "not the relay's own address, which a received frame would be sent to");
  const bytes = hexOf("ping");
  assert.equal(frameSignalBody(toTarget, bytes).target, "192.0.2.20:9000");
  assert.equal(frameSignalBody(toClient, bytes).target, "192.0.2.30:50123");
  assert.equal(frameSignalBody({ ...toTarget, remote: "0.0.0.0:9000" }, bytes).target, "127.0.0.1:9000");
  // What older frames held there — a direction name — is no address.
  assert.equal(frameSignalBlock(frame({ source: "netsim", dir: "rx", local: "target→client", remote: "" })), "ins.noSignalForm");
});

test("only a datagram or an MQTT publish kept whole becomes a signal; the rest says why not", () => {
  assert.equal(frameSignalBlock(frame()), null);
  assert.equal(frameSignalBlock(frame({ proto: "osc" })), null);
  for (const proto of ["tcp", "http", "ws"]) {
    assert.equal(frameSignalBlock(frame({ proto })), "ins.noSignalForm", `${proto} has no signal form`);
    assert.equal(frameSignalBody(frame({ proto }), hexOf("ping")), null);
  }
  assert.equal(frameSignalBlock(frame({ proto: "tcp", source: "netsim", dir: "tx", remote: "192.0.2.20:9000" })), "ins.noSignalForm", "a relayed TCP chunk is a piece of a stream");
  assert.equal(frameSignalBlock(frame({ proto: "tcp", source: "emulator" })), "ins.noSignalForm");
  // Half a packet is another packet; so is none.
  assert.equal(frameSignalBlock(frame({ kept: 3 })), "ins.noExactCopy");
  assert.equal(frameSignalBlock(frame({ bytes: 70000, kept: 0 })), "ins.noExactCopy");
  assert.equal(frameSignalBlock(frame({ bytes: 0, kept: 0 })), "ins.noExactCopy", "a datagram of nothing kept nothing");
  assert.equal(frameSignalBody(frame({ kept: 3 }), "70 69 6e"), null);
  // The bytes asked for must be the frame's.
  assert.equal(frameSignalBody(frame(), "70 69 6e"), null, "3 bytes of a 4-byte frame");
  assert.equal(frameSignalBody(frame(), "zz"), null);
  assert.equal(frameSignalBody(frame(), ""), null);
  assert.ok(isHostPort("127.0.0.1:9000") && isHostPort("[::1]:9000") && isHostPort("device.local:9000"));
  assert.ok(!isHostPort("127.0.0.1") && !isHostPort("127.0.0.1:0") && !isHostPort("127.0.0.1:70000") && !isHostPort("target→client"));
});

test("an MQTT publish becomes an MQTT signal with its broker, topic, flags and exact payload", () => {
  const on = publishFrame();
  assert.equal(frameSignalBlock(on), null);
  assert.deepEqual(frameSignalBody(on, hexOf("ON")), { transport: "mqtt", broker: "127.0.0.1:1883", topic: "lab/lamp", payload: "ON", qos: 1, retain: true });
  // Byte for byte: spaces, a byte order mark and multi-byte characters survive.
  const text = "﻿ café ✓ ";
  const size = Buffer.byteLength(text);
  const exact = publishFrame({ bytes: size, kept: size }, { qos: 2, retain: false, topic: "a/b c" });
  assert.deepEqual(frameSignalBody(exact, hexOf(text)), { transport: "mqtt", broker: "127.0.0.1:1883", topic: "a/b c", payload: text, qos: 2, retain: false });
  // Clearing a retained value is a publish of nothing: it can be saved.
  const clear = publishFrame({ bytes: 0, kept: 0 });
  assert.equal(frameSignalBlock(clear), null);
  assert.deepEqual(frameSignalBody(clear, ""), { transport: "mqtt", broker: "127.0.0.1:1883", topic: "lab/lamp", payload: "", qos: 1, retain: true });
  // An emulator's broker listening on every address is reached on loopback.
  assert.equal(frameSignalBody(publishFrame({}, { broker: "0.0.0.0:1883" }), hexOf("ON")).broker, "127.0.0.1:1883");
  // Received or sent, the publish is the same message.
  assert.equal(frameSignalBody(publishFrame({ dir: "rx", source: "emulator" }), hexOf("ON")).topic, "lab/lamp");
  // Bytes that are not text cannot be a signal's payload; a control packet is not a publish at all.
  assert.equal(frameSignalBlock(publishFrame({ bytes: 3, kept: 3 }, { text: false })), "ins.notText");
  assert.equal(frameSignalBody(publishFrame({ bytes: 1, kept: 1 }, { text: true }), "ff"), null, "and a frame that lies about being text is still refused");
  const subscribe = { ...publishFrame({ bytes: 0, kept: 0, summary: "SUBSCRIBE lab/#" }), publish: undefined };
  assert.equal(frameSignalBlock(subscribe), "ins.noSignalForm");
  assert.equal(frameSignalBlock(publishFrame({ kept: 1 })), "ins.noExactCopy");
});

test("a saved frame is named, filed and noted, and never takes an id that is used", () => {
  const taken = [{ id: "ping", name: "ping", group: "", note: "", body: { transport: "osc", target: "127.0.0.1:9000", address: "/x", args: [] } }];
  const saved = signalFromFrame(frame(), hexOf("ping"), taken, "ping", t);
  assert.equal(saved.id, "ping-2");
  assert.equal(saved.group, "sig.capturedFolder");
  assert.equal(saved.note, "#7 udp ← 127.0.0.1:50000 · ping");
  assert.equal(saved.body.transport, "udp");
  const mqtt = signalFromFrame(publishFrame(), hexOf("ON"), [], "lab/lamp = ON", t);
  assert.deepEqual([mqtt.id, mqtt.body.transport, mqtt.note], ["lab-lamp-on", "mqtt", "#7 mqtt → 127.0.0.1:1883 · lab/lamp = ON"]);
  assert.equal(signalFromFrame(frame({ proto: "tcp" }), hexOf("ping"), [], "x", t), null);
  assert.equal(makeId("", []), "signal");
});

// ---- MQTT signals and the connection they ride --------------------------------

test("brokers are compared as written ports and hosts, never looked up", () => {
  assert.deepEqual(splitBroker("broker.example:8883"), { host: "broker.example", port: 8883 });
  assert.deepEqual(splitBroker("broker.example"), { host: "broker.example", port: 1883 });
  assert.deepEqual(splitBroker("[::1]:1884"), { host: "[::1]", port: 1884 });
  assert.deepEqual(splitBroker("[::1]"), { host: "[::1]", port: 1883 });
  assert.deepEqual(splitBroker("::1"), { host: "::1", port: 1883 }, "a bare IPv6 address has no port to split off");
  assert.ok(sameBroker("Broker.Example:1883", "broker.example"), "case does not matter, and 1883 is the default");
  assert.ok(sameBroker("broker.example:1883", "broker.example"));
  assert.ok(sameBroker("[::1]:1883", "[::1]"));
  assert.ok(!sameBroker("broker.example:1883", "broker.example:1884"));
  assert.ok(!sameBroker("localhost:1883", "127.0.0.1:1883"), "two names: nothing here looks them up");
  assert.ok(!sameBroker("192.0.2.1:1883", "192.0.2.2:1883"));
});

test("an MQTT signal rides a connection to its own broker, else makes one of its own", () => {
  const job = (id, kind, broker) => ({ id, kind, label: "", started_ms: 0, params: broker === undefined ? undefined : { broker } });
  const jobs = [job(1, "osc-monitor"), job(2, "mqtt", "192.0.2.10:1883"), job(3, "mqtt", "Broker.Example:1883"), job(4, "mqtt")];
  assert.equal(mqttConnectionFor("broker.example", jobs), 3, "the connection to that broker, whichever is first");
  assert.equal(mqttConnectionFor("192.0.2.10:1883", jobs), 2);
  assert.equal(mqttConnectionFor("192.0.2.11:1883", jobs), null, "another broker: not this connection");
  assert.equal(mqttConnectionFor("broker.example:8883", jobs), null, "another port is another broker");
  assert.equal(mqttConnectionFor("localhost:1883", [job(5, "mqtt", "127.0.0.1:1883")]), null, "localhost is not 127.0.0.1 here");
  assert.equal(mqttConnectionFor("192.0.2.10:1883", []), null);
  assert.equal(mqttConnectionFor("192.0.2.10:1883", [job(6, "osc-monitor", "192.0.2.10:1883")]), null, "only an MQTT job is a connection");
});

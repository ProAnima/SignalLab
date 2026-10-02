import test from "node:test";
import assert from "node:assert/strict";
import { createNode, createNodeIn, validPortsFor } from "../src/lib/experimentGraph.ts";
import { canRepeat, canRetry, replyFields, variablesBefore, writtenVariable } from "../src/lib/experimentData.ts";
import { isWait, previewLines } from "../src/lib/experimentText.ts";
import { asJson, protocolsOf, protocolsText } from "../src/lib/websocket.ts";

const doc = (nodes, edges = []) => ({ version: 8, name: "x", params: [], profiles: [], profile: null, seed: null, nodes, edges });

test("a WebSocket send, wait or close names the document's connect when added", () => {
  const connect = createNode("ws_connect", 0, 0);
  assert.ok(connect.url.startsWith("ws://127.0.0.1:"), "a new connect stays on loopback");
  for (const type of ["ws_send", "wait_ws", "ws_close"]) {
    assert.equal(createNodeIn(doc([connect]), type, 0, 0).connection, connect.id, type);
    assert.equal(createNodeIn(doc([]), type, 0, 0).connection, "", `${type} without a connect names none`);
  }
  assert.deepEqual(validPortsFor("wait_ws"), ["matched", "timeout"]);
  assert.deepEqual(validPortsFor("ws_connect"), ["next"]);
  assert.ok(isWait(createNode("wait_ws", 0, 0)) && !isWait(connect));
});

test("a connect retries but does not repeat; a send does both; the wait's reply has json", () => {
  assert.ok(canRetry(createNode("ws_connect", 0, 0)) && !canRepeat(createNode("ws_connect", 0, 0)));
  assert.ok(canRetry(createNode("ws_send", 0, 0)) && canRepeat(createNode("ws_send", 0, 0)));
  assert.ok(!canRetry(createNode("ws_close", 0, 0)) && !canRepeat(createNode("ws_close", 0, 0)));
  const wait = { ...createNode("wait_ws", 0, 0), mode: "contains", pattern: "pong" };
  assert.deepEqual(writtenVariable(wait), { name: "reply", port: "matched" });
  assert.deepEqual(replyFields(wait), ["json", "text", "match", "hex", "bytes", "from", "ms"]);
  const start = createNode("start", 0, 0);
  const after = createNode("log", 0, 0);
  const graph = doc([start, wait, after], [{ from: start.id, to: wait.id, port: "next" }, { from: wait.id, to: after.id, port: "matched" }]);
  assert.deepEqual(variablesBefore(graph, after.id).map((found) => found.name), ["reply"], "the reply is visible after Matched");
});

test("subprotocols are a comma list, previews show the upgrade, JSON reads formatted", () => {
  assert.deepEqual(protocolsOf(" graphql-transport-ws , v2.json,, "), ["graphql-transport-ws", "v2.json"]);
  assert.equal(protocolsText(["a", "b"]), "a, b");
  const connect = { ...createNode("ws_connect", 0, 0), headers: [["Authorization", "Bearer {{token}}"]], protocols: ["chat"] };
  assert.deepEqual(previewLines(connect), [connect.url, "Authorization: Bearer {{token}}", "Sec-WebSocket-Protocol: chat"]);
  assert.deepEqual(previewLines({ ...createNode("ws_send", 0, 0), connection: "socket", text: "ca fe", binary: true }), ["→ socket (hex)", "ca fe"]);
  assert.equal(asJson('{"a":1}'), JSON.stringify({ a: 1 }, null, 2));
  assert.equal(asJson("echo hi"), null);
  assert.equal(asJson("{broken"), null);
});

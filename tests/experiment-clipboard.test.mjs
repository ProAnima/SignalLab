import test from "node:test";
import assert from "node:assert/strict";
import { clipText, copyOf, dragNodes, freshCopy, moveNodes, nodesIn, placeCopy, readClip, removeNodes } from "../src/lib/experimentClipboard.ts";
import { NODE_HEIGHT, NODE_WIDTH } from "../src/lib/experimentGraph.ts";

// start → connect → send → close → end, and a Change impairment beside them naming the relay.
const fixture = () => ({
  version: 8, name: "Clipboard fixture",
  nodes: [
    { id: "start", type: "start", x: 40, y: 40 },
    { id: "connect", type: "ws_connect", x: 260, y: 40, url: "ws://127.0.0.1:9100/", headers: [], protocols: [], timeout_ms: 3000 },
    { id: "send", type: "ws_send", x: 480, y: 40, connection: "connect", text: "hello", binary: false },
    { id: "close", type: "ws_close", x: 700, y: 40, connection: "connect", code: 1000, reason: "" },
    { id: "end", type: "end", x: 920, y: 40 },
    { id: "relay", type: "impairment", x: 260, y: 300, listen: "127.0.0.1:9010", target: "127.0.0.1:9000", profile: {} },
    { id: "change", type: "impairment_change", x: 480, y: 300, relay: "relay", profile: {} },
  ],
  edges: [
    { from: "start", to: "connect" }, { from: "connect", to: "send" }, { from: "send", to: "close" }, { from: "close", to: "end" },
    { from: "relay", to: "change" },
  ],
});

test("a copy is the selected nodes and only the wires between them, never Start or End", () => {
  const clip = copyOf(fixture(), ["start", "connect", "send", "end"]);
  assert.deepEqual(clip.nodes.map((node) => node.id), ["connect", "send"]);
  assert.deepEqual(clip.edges, [{ from: "connect", to: "send" }]);
  assert.equal(copyOf(fixture(), ["start", "end"]), null, "nothing to copy");
  assert.equal(copyOf(fixture(), []), null);
});

test("clipboard text reads back as the same clip, and nothing else reads as one", () => {
  const clip = copyOf(fixture(), ["connect", "send"]);
  assert.deepEqual(readClip(clipText(clip)), clip);
  for (const text of ["", "hello", "{}", "[]", JSON.stringify({ signalLab: "nodes", version: 2, nodes: clip.nodes, edges: [] }),
    JSON.stringify({ ...clip, nodes: [{ ...clip.nodes[0], type: "teleport" }] }),
    JSON.stringify({ ...clip, nodes: [{ id: "s", type: "start", x: 1, y: 1 }] }),
    JSON.stringify({ ...clip, nodes: [clip.nodes[0], clip.nodes[0]] })]) {
    assert.equal(readClip(text), null, text.slice(0, 60));
  }
  // A wire to a node that was not copied is dropped, not trusted.
  const loose = readClip(JSON.stringify({ ...clip, edges: [...clip.edges, { from: "send", to: "elsewhere" }] }));
  assert.deepEqual(loose.edges, [{ from: "connect", to: "send" }]);
});

test("pasted nodes get ids of their own; a reference follows a copied node and keeps one left behind", () => {
  const doc = fixture();
  const copy = freshCopy(doc, copyOf(doc, ["connect", "send", "change"]));
  const ids = copy.nodes.map((node) => node.id);
  assert.equal(new Set(ids).size, 3);
  assert.ok(ids.every((id) => !doc.nodes.some((node) => node.id === id)), "new ids");
  const [connect, send, change] = copy.nodes;
  assert.equal(send.connection, connect.id, "the copied send uses the copied connection");
  assert.equal(change.relay, "relay", "the relay was not copied: the original stays named");
  assert.deepEqual(copy.edges, [{ from: connect.id, to: send.id }]);
  // Into an experiment without that relay: its first relay, or none.
  const elsewhere = { ...doc, nodes: doc.nodes.filter((node) => node.id !== "relay" && node.id !== "change"), edges: [] };
  assert.equal(freshCopy(elsewhere, copyOf(doc, ["change"])).nodes[0].relay, "");
  const another = { ...elsewhere, nodes: [...elsewhere.nodes, { ...doc.nodes[5], id: "other-relay" }] };
  assert.equal(freshCopy(another, copyOf(doc, ["change"])).nodes[0].relay, "other-relay");
});

test("a copied relay or emulator listens on the next free port; what sends to the original still does", () => {
  const doc = fixture();
  doc.nodes.push({ id: "api", type: "emulator", x: 40, y: 500, emulator: { bind: "127.0.0.1:18080" } }, { id: "wait", type: "wait_udp", x: 260, y: 500, bind: "127.0.0.1:9011", mode: "any", pattern: "", timeout_ms: 1000, variable: "" });
  const copy = freshCopy(doc, copyOf(doc, ["relay", "api", "connect"]));
  const byType = (type) => copy.nodes.find((node) => node.type === type);
  assert.equal(byType("impairment").listen, "127.0.0.1:9012", "9010 is the original's, 9011 the wait's");
  assert.equal(byType("impairment").target, "127.0.0.1:9000", "still forwards where the original does");
  assert.equal(byType("emulator").emulator.bind, "127.0.0.1:18081");
  assert.equal(byType("ws_connect").url, "ws://127.0.0.1:9100/", "a client's address is not a port of the run's");
  // Two copies at once do not take the same port either.
  const two = freshCopy(doc, { ...copyOf(doc, ["api"]), nodes: [doc.nodes.at(-2), { ...doc.nodes.at(-2), id: "api2" }] });
  assert.deepEqual(two.nodes.map((node) => node.emulator.bind), ["127.0.0.1:18081", "127.0.0.1:18082"]);
  // Into an experiment where the port is free, it keeps it.
  const empty = { ...doc, nodes: doc.nodes.filter((node) => node.type === "start" || node.type === "end"), edges: [] };
  assert.equal(freshCopy(empty, copyOf(doc, ["api"])).nodes[0].emulator.bind, "127.0.0.1:18080");
  // A port written as a parameter is the person's to choose.
  const templated = { ...doc, nodes: [...doc.nodes, { id: "p", type: "emulator", x: 0, y: 0, emulator: { bind: "127.0.0.1:{{port}}" } }] };
  assert.equal(freshCopy(templated, copyOf(templated, ["p"])).nodes[0].emulator.bind, "127.0.0.1:{{port}}");
});

test("a pasted group lands below its original, clear of every node, with its wires", () => {
  const doc = fixture();
  const copy = freshCopy(doc, copyOf(doc, ["connect", "send"]));
  const pasted = placeCopy(doc, copy);
  assert.equal(pasted.nodes.length, doc.nodes.length + 2);
  assert.equal(pasted.edges.length, doc.edges.length + 1);
  const added = pasted.nodes.slice(doc.nodes.length);
  const covered = (a, b) => Math.abs(a.x - b.x) < NODE_WIDTH + 12 && Math.abs(a.y - b.y) < NODE_HEIGHT + 12;
  for (const node of added) assert.ok(!doc.nodes.some((other) => covered(node, other)), `${node.id} covers nothing`);
  assert.equal(added[1].x - added[0].x, 220, "the group keeps its shape");
  // Pasted again: lower still.
  const twice = placeCopy(pasted, freshCopy(pasted, copyOf(doc, ["connect", "send"])));
  assert.ok(twice.nodes.at(-1).y > added[1].y);
});

test("removing a selection bridges the flow around it and keeps Start and End", () => {
  const doc = fixture();
  const removed = removeNodes(doc, ["start", "send", "close", "end"]);
  assert.deepEqual(removed.nodes.map((node) => node.id), ["start", "connect", "end", "relay", "change"]);
  assert.ok(removed.edges.some((edge) => edge.from === "connect" && edge.to === "end"), "connect → end");
});

test("a selection moves and drags as one, stopping at the margin as a whole", () => {
  const doc = fixture();
  const moved = moveNodes(doc, ["connect", "send"], 5, -20);
  assert.deepEqual(moved.nodes.slice(1, 3).map(({ x, y }) => [x, y]), [[265, 20], [485, 20]]);
  const stopped = moveNodes(doc, ["connect", "send"], 0, -100);
  assert.deepEqual(stopped.nodes.slice(1, 3).map(({ y }) => y), [12, 12], "not past the margin, still in a row");
  assert.equal(moveNodes(doc, [], 10, 10), doc);
  const start = new Map([["connect", { x: 260, y: 40 }], ["send", { x: 480, y: 40 }]]);
  const dragged = dragNodes(doc, start, 10.4, 30.6);
  assert.deepEqual(dragged.nodes.slice(1, 3).map(({ x, y }) => [x, y]), [[270, 71], [490, 71]]);
  assert.equal(dragNodes(dragged, start, 10.4, 30.6), dragged, "the same place again changes nothing");
});

test("a frame selects what it touches, drawn from any corner", () => {
  const nodes = fixture().nodes;
  assert.deepEqual(nodesIn(nodes, { x: 250, y: 30 }, { x: 500, y: 60 }), ["connect", "send"]);
  assert.deepEqual(nodesIn(nodes, { x: 500, y: 60 }, { x: 250, y: 30 }), ["connect", "send"]);
  assert.deepEqual(nodesIn(nodes, { x: 0, y: 200 }, { x: 10, y: 210 }), []);
});

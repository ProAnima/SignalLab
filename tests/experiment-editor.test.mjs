import test from "node:test";
import assert from "node:assert/strict";
import { historyReducer, newHistory, HISTORY_LIMIT } from "../src/lib/editHistory.ts";
import { addAfter, addBranch, anchorAfter, arrangeNodes, connect, createNode, disconnect, duplicateNode, insertOnEdge, isWired, missingOutputs, placeAfter, removeNode, requiredPortsFor, unreachableNodes, validPortsFor, waitForMqttMessage, waitForOscMessage, NODE_WIDTH, NODE_HEIGHT } from "../src/lib/experimentGraph.ts";
import { ADDABLE_NODES, NODE_CATALOG } from "../src/lib/experimentCatalog.ts";
import { jsonPath, suggestVariableName, templateAt, variablesBefore, isIdent, effectiveParams, renameParam, removeParam, setParamValue, addProfile, renameProfile, removeProfile, secretNames, replyFields, writtenVariable, canRetry, defaultReply, DEFAULT_RETRY } from "../src/lib/experimentData.ts";
import { describeError, failureNode, fieldLabel, isEngineError, messageParams, responseFailure } from "../src/lib/errors.ts";
import { en } from "../src/lib/locales/en.ts";
import { ru } from "../src/lib/locales/ru.ts";

test("every catalogue node can be inserted, duplicated and removed with localized metadata", () => {
  for (const type of ADDABLE_NODES) {
    const initial = fixture();
    const node = createNode(type, 100, 100);
    const inserted = insertOnEdge(initial, initial.edges[0], node);
    assert.equal(inserted.nodes.length, initial.nodes.length + 1);
    assert.ok(inserted.edges.some(edge => edge.to === node.id));
    const copy = duplicateNode(inserted, node.id);
    assert.equal(copy.type, type);
    assert.notEqual(copy.id, node.id);
    const removed = removeNode(inserted, node.id);
    assert.ok(removed.edges.some(edge => edge.from === "start" && edge.to === "request"));
    for (const locale of [en, ru]) {
      assert.ok(locale[NODE_CATALOG[type].title]);
      assert.ok(locale[NODE_CATALOG[type].description]);
    }
  }
});

const fixture = () => ({
  version: 1, name: "Editor fixture",
  nodes: [
    { id: "start", type: "start", x: 40, y: 40 },
    { id: "request", type: "http", x: 240, y: 40, request: { method: "GET", url: "http://127.0.0.1/", headers: [["X-Test", "original"]], body: null, timeout_ms: 4000 } },
    { id: "end", type: "end", x: 440, y: 40 },
  ],
  edges: [{ from: "start", to: "request" }, { from: "request", to: "end" }],
});

test("a long drag is one undo step, with exact graph restoration and redo", () => {
  const initial = fixture();
  let history = newHistory(initial);
  for (let y = 50; y <= 400; y += 10) {
    history = historyReducer(history, { type: "edit", group: "drag-1", update: (doc) => ({ ...doc, nodes: doc.nodes.map((node) => node.id === "request" ? { ...node, y } : node) }) });
  }
  assert.equal(history.past.length, 1);
  const moved = history.present;
  history = historyReducer(history, { type: "commit" });
  history = historyReducer(history, { type: "undo" });
  assert.deepEqual(history.present, initial);
  assert.equal(historyReducer(history, { type: "redo" }).present, moved);
});

test("field sessions are distinct; a new edit after undo discards redo", () => {
  let history = newHistory(fixture());
  const rename = (name) => (doc) => ({ ...doc, name });
  history = historyReducer(history, { type: "edit", group: "name", update: rename("a") });
  history = historyReducer(history, { type: "edit", group: "name", update: rename("abc") });
  history = historyReducer(history, { type: "commit" });
  history = historyReducer(history, { type: "edit", group: "name", update: rename("second edit") });
  assert.equal(history.past.length, 2);
  history = historyReducer(history, { type: "undo" });
  assert.equal(history.present.name, "abc");
  const noOp = historyReducer(history, { type: "edit", update: (doc) => doc });
  assert.equal(noOp, history);
  history = historyReducer(history, { type: "edit", update: rename("replacement") });
  assert.equal(history.future.length, 0);
  assert.equal(historyReducer(history, { type: "redo" }), history);
});

test("deletion and insertion restore original wires and positions on undo", () => {
  const initial = fixture();
  let history = newHistory(initial);
  history = historyReducer(history, { type: "edit", update: (doc) => removeNode(doc, "request") });
  assert.deepEqual(history.present.edges, [{ from: "start", to: "end", port: "next" }]);
  history = historyReducer(history, { type: "undo" });
  assert.deepEqual(history.present, initial);
  history = historyReducer(history, { type: "edit", update: (doc) => insertOnEdge(doc, doc.edges[1], { id: "delay", type: "delay", x: 400, y: 40, ms: 10 }) });
  const inserted = history.present;
  const delay = inserted.nodes.find((node) => node.id === "delay");
  const end = inserted.nodes.find((node) => node.id === "end");
  assert.ok(end.x >= delay.x + NODE_WIDTH);
  assert.deepEqual(historyReducer(history, { type: "undo" }).present, initial);
});

test("duplicate has independent nested parameters, free space and no copied wires", () => {
  const original = fixture();
  const copy = duplicateNode(original, "request");
  assert.notEqual(copy.id, "request");
  assert.equal(copy.type, "http");
  copy.request.headers[0][1] = "changed";
  assert.equal(original.nodes[1].request.headers[0][1], "original");
  const second = duplicateNode({ ...original, nodes: [...original.nodes, copy] }, "request");
  assert.ok(second.y >= copy.y + NODE_HEIGHT);
  assert.equal(duplicateNode(original, "start"), null);
  assert.equal(duplicateNode(original, "end"), null);
  assert.equal(original.edges.length, 2);
});

test("layout places joined branches after every predecessor and preserves the graph", () => {
  const graph = fixture();
  graph.nodes.splice(2, 0,
    { id: "branch", type: "branch_status", status: 200, x: 0, y: 0 },
    { id: "yes", type: "delay", ms: 10, x: 0, y: 0 },
    { id: "no", type: "delay", ms: 20, x: 0, y: 0 },
    { id: "draft", type: "udp", target: "127.0.0.1:9000", text: "test", x: 0, y: 0 });
  graph.edges = [graph.edges[0], { from: "request", to: "branch" },
    { from: "branch", to: "yes", port: "yes" }, { from: "branch", to: "no", port: "no" },
    { from: "yes", to: "end" }, { from: "no", to: "end" }];
  graph.nodes.reverse();
  const arranged = arrangeNodes(graph);
  assert.equal(arranged.edges, graph.edges);
  assert.equal(arranged.nodes.length, graph.nodes.length);
  for (const edge of graph.edges) {
    assert.ok(arranged.nodes.find((node) => node.id === edge.to).x > arranged.nodes.find((node) => node.id === edge.from).x);
  }
  for (let i = 0; i < arranged.nodes.length; i++) for (let j = i + 1; j < arranged.nodes.length; j++) {
    const a = arranged.nodes[i], b = arranged.nodes[j];
    assert.ok(Math.abs(a.x - b.x) >= NODE_WIDTH || Math.abs(a.y - b.y) >= NODE_HEIGHT);
  }
  assert.equal(arrangeNodes(arranged), arranged);
  const cyclic = { ...graph, edges: [...graph.edges, { from: "end", to: "request" }] };
  assert.equal(arrangeNodes(cyclic), cyclic);
});

test("rewiring cannot create a cycle and failed edits preserve undo history", () => {
  const graph = fixture();
  graph.nodes.splice(2, 0, { id: "delay", type: "delay", ms: 10, x: 400, y: 0 });
  graph.edges[1] = { from: "request", to: "delay" };
  graph.edges.push({ from: "delay", to: "end" });
  assert.equal(connect(graph, "delay", "next", "request"), graph);
  assert.equal(connect(graph, "request", "no", "end"), graph);
});

test("parallel fork and join wires connect, arrange and maintain DAG integrity", () => {
  const doc = fixture();
  const fork = createNode("fork", 200, 40);
  const join = createNode("join", 600, 40);
  const stream1 = createNode("delay", 400, 0);
  const stream2 = createNode("log", 400, 100);

  let flow = { ...doc, nodes: [doc.nodes[0], fork, stream1, stream2, join, doc.nodes[2]], edges: [] };
  flow = connect(flow, "start", "next", fork.id);
  flow = connect(flow, fork.id, "branch1", stream1.id);
  flow = connect(flow, fork.id, "branch2", stream2.id);
  flow = connect(flow, stream1.id, "next", join.id);
  flow = connect(flow, stream2.id, "next", join.id);
  flow = connect(flow, join.id, "next", "end");

  assert.equal(flow.edges.length, 6);
  assert.ok(flow.edges.some(e => e.from === fork.id && e.port === "branch1" && e.to === stream1.id));
  assert.ok(flow.edges.some(e => e.from === fork.id && e.port === "branch2" && e.to === stream2.id));
  assert.equal(flow.edges.filter(e => e.to === join.id).length, 2);

  // arrangeNodes arranges columns with fork before parallel branches and join after both
  const arranged = arrangeNodes(flow);
  const xFork = arranged.nodes.find(n => n.id === fork.id).x;
  const xStream1 = arranged.nodes.find(n => n.id === stream1.id).x;
  const xStream2 = arranged.nodes.find(n => n.id === stream2.id).x;
  const xJoin = arranged.nodes.find(n => n.id === join.id).x;
  assert.ok(xFork < xStream1);
  assert.ok(xFork < xStream2);
  assert.ok(xStream1 < xJoin);
  assert.ok(xStream2 < xJoin);
});

test("history stays bounded and loading another document resets it", () => {
  let history = newHistory(0);
  for (let i = 1; i <= HISTORY_LIMIT + 20; i++) history = historyReducer(history, { type: "edit", update: () => i });
  assert.equal(history.past.length, HISTORY_LIMIT);
  history = historyReducer(history, { type: "reset", document: 999 });
  assert.deepEqual(history, newHistory(999));
});

test("opening another experiment is reversible with parameters and connections intact", () => {
  const original = fixture();
  const next = { version: 1, name: "New experiment", nodes: [original.nodes[0], original.nodes[2]],
    edges: [{ from: "start", to: "end", port: "next" }] };
  let history = newHistory(original);
  history = historyReducer(history, { type: "edit", update: () => next });
  assert.equal(history.present, next);
  history = historyReducer(history, { type: "undo" });
  assert.deepEqual(history.present, original);
  history = historyReducer(history, { type: "redo" });
  assert.deepEqual(history.present, next);
});

test("adding after a node splices into its wire; End means before End", () => {
  const graph = fixture();
  const delay = createNode("delay", 0, 0);
  const anchor = anchorAfter(graph, "request");
  assert.deepEqual(anchor, { from: "request", port: "next" });
  const spot = placeAfter(graph, anchor);
  const next = addAfter(graph, anchor, { ...delay, ...spot });
  assert.ok(next.edges.some((edge) => edge.from === "request" && edge.to === delay.id));
  assert.ok(next.edges.some((edge) => edge.from === delay.id && edge.to === "end"));
  assert.equal(next.edges.filter((edge) => edge.from === "request").length, 1);
  const placed = next.nodes.find((node) => node.id === delay.id);
  const end = next.nodes.find((node) => node.id === "end");
  assert.equal(placed.y, graph.nodes[1].y);
  assert.ok(end.x >= placed.x + NODE_WIDTH, "downstream nodes make room");
  assert.deepEqual(anchorAfter(graph, "end"), { from: "request", port: "next" });
  // Two ways into End: "before End" is ambiguous, so there is no anchor.
  const joined = { ...graph, edges: [...graph.edges, { from: "start", to: "end", port: "next" }] };
  assert.equal(anchorAfter({ ...joined, edges: joined.edges.filter((edge) => !(edge.from === "start" && edge.to === "request")) }, "end"), null);
});

test("a branch fills its free output first and places the new node without overlap", () => {
  const graph = fixture();
  const branch = { id: "branch", type: "branch_status", status: 200, x: 440, y: 40 };
  graph.nodes.push(branch);
  graph.edges.push({ from: "branch", to: "end", port: "yes" });
  const anchor = anchorAfter(graph, "branch");
  assert.deepEqual(anchor, { from: "branch", port: "no" });
  const spot = placeAfter(graph, anchor);
  for (const node of graph.nodes) assert.ok(Math.abs(node.x - spot.x) >= NODE_WIDTH || Math.abs(node.y - spot.y) >= NODE_HEIGHT);
  const log = { ...createNode("log", 0, 0), ...spot };
  const next = addAfter(graph, anchor, log);
  assert.ok(next.edges.some((edge) => edge.from === "branch" && edge.port === "no" && edge.to === log.id));
  assert.ok(!missingOutputs(next).has("branch:no"));
  assert.ok(missingOutputs(next).has(`${log.id}:next`));
  // No usable anchor: the node is only added, never wired somewhere surprising.
  const loose = createNode("osc", 10, 10);
  const added = addAfter(graph, null, loose);
  assert.deepEqual(added.edges, graph.edges);
  assert.ok(added.nodes.some((node) => node.id === loose.id));
});

test("editor markers: missing outputs and unreachable nodes", () => {
  const graph = fixture();
  assert.equal(missingOutputs(graph).size, 0);
  assert.equal(unreachableNodes(graph).size, 0);
  const draft = createNode("udp", 600, 200);
  const withDraft = { ...graph, nodes: [...graph.nodes, draft] };
  assert.deepEqual([...missingOutputs(withDraft)], [`${draft.id}:next`]);
  assert.deepEqual([...unreachableNodes(withDraft)], [draft.id]);
});

test("waits have Matched and an optional Timeout, and add on loopback", () => {
  for (const type of ["wait_osc", "wait_udp"]) {
    const wait = createNode(type, 0, 0);
    assert.match(wait.bind, /^127\.0\.0\.1:\d+$/, "a new wait listens on loopback");
    assert.deepEqual(validPortsFor(type), ["matched", "timeout"]);
    assert.deepEqual(requiredPortsFor(type), ["matched"]);
    assert.equal(NODE_CATALOG[type].group, "observe");
  }
  const graph = fixture();
  const wait = { ...createNode("wait_osc", 440, 40), id: "wait" };
  graph.nodes.push(wait);
  graph.edges = [{ from: "start", to: "request" }, { from: "request", to: "wait" }, { from: "wait", to: "end", port: "matched" }];
  assert.equal(missingOutputs(graph).size, 0, "an unwired Timeout is not missing");
  assert.deepEqual(anchorAfter(graph, "wait"), { from: "wait", port: "timeout" }, "the free output is offered next");
  const spliced = insertOnEdge(graph, graph.edges[1], { ...createNode("wait_udp", 300, 40), id: "second" });
  assert.ok(spliced.edges.some((edge) => edge.from === "second" && edge.port === "matched" && edge.to === "wait"), "a spliced wait continues on Matched");
});

test("a reply is a variable on the Matched path only, with fields to suggest", () => {
  const graph = fixture();
  const wait = { ...createNode("wait_osc", 0, 0), id: "wait", variable: "reply" };
  const ok = { id: "ok", type: "log", message: "", x: 0, y: 0 };
  const late = { id: "late", type: "log", message: "", x: 0, y: 0 };
  graph.nodes.push(wait, ok, late);
  graph.edges = [{ from: "start", to: "request" }, { from: "request", to: "wait" }, { from: "wait", to: "ok", port: "matched" },
    { from: "wait", to: "late", port: "timeout" }, { from: "ok", to: "end" }, { from: "late", to: "end" }];
  assert.deepEqual(variablesBefore(graph, "ok").map((item) => item.name), ["reply"]);
  assert.deepEqual(variablesBefore(graph, "late"), [], "a timeout has no reply");
  assert.deepEqual(variablesBefore(graph, "end").map((item) => item.name), ["reply"], "set on some path into End");
  assert.deepEqual(writtenVariable(wait), { name: "reply", port: "matched" });
  assert.ok(replyFields(wait).includes("args[0]"));
  assert.ok(replyFields({ ...createNode("wait_udp", 0, 0) }).includes("match"));
  assert.ok(!replyFields({ ...createNode("wait_udp", 0, 0), mode: "any" }).includes("match"));
});

test("failures read the same everywhere: located, translated, with the detail kept", () => {
  const t = (key, params) => {
    const text = en[key] ?? key;
    return params ? text.replace(/\{(\w+)\}/g, (match, name) => name in params ? String(params[name]) : match) : text;
  };
  const error = { code: "transport.refused", params: { target: "127.0.0.1:8080" }, node: "request", field: { key: "header_value", index: 2 }, detail: "os error 10061" };
  assert.ok(isEngineError(error) && !isEngineError("text") && !isEngineError(null));
  const described = describeError(error, t, (id) => id === "request" ? "HTTP request" : null);
  assert.equal(described.where, "HTTP request · Header value 2");
  assert.equal(described.message, en["err.transport.refused"].replace("{target}", "127.0.0.1:8080"));
  assert.equal(described.text, `${described.where} — ${described.message}`);
  assert.equal(described.detail, "os error 10061");
  assert.equal(failureNode(error), "request");
  // A node that no longer exists is left out of the location, not shown as an id.
  assert.equal(describeError(error, t, () => null).where, "Header value 2");
  // Template errors say where in the field.
  assert.equal(describeError({ code: "template.unclosed", params: { position: "4" }, field: { key: "url" } }, t).where, "URL · position 4");
  // Operators are worded, not shown as codes.
  assert.equal(messageParams({ value: "1", op: "not_empty" }, t).op, en["exp.op.not_empty"]);
  assert.match(describeError({ code: "check.value_failed", params: { value: "401", op: "eq", expected: "200" } }, t).message, new RegExp(`401 ${en["exp.op.eq"]} 200`));
  // Unknown codes, legacy text, dictionary keys and thrown errors all render.
  assert.equal(describeError({ code: "from.the.future" }, t).message, en["err.unknown"].replace("{code}", "from.the.future"));
  assert.equal(describeError("bind 0.0.0.0:9000 failed", t).text, "bind 0.0.0.0:9000 failed");
  assert.equal(describeError("exp.invalidConnection", t).text, en["exp.invalidConnection"]);
  assert.equal(describeError(new Error("boom"), t).message, "boom");
  assert.equal(fieldLabel({ key: "no_such_field" }, t), "no_such_field");
  // A response without an answer becomes the same transport failure as in a run.
  assert.deepEqual(responseFailure({ error: "error sending request: tcp connect error", cause: "refused" }, "http://x/"),
    { code: "transport.refused", params: { target: "http://x/" }, detail: "error sending request: tcp connect error" });
  assert.equal(responseFailure({ error: "odd", cause: null }, "http://x/"), "odd");
  assert.equal(responseFailure({ error: null }, "http://x/"), null);
});

test("every code the UI can construct has a text in both languages", () => {
  for (const cause of ["refused", "timeout", "dns", "unreachable", "reset", "address_in_use", "address_unavailable", "denied", "tls", "target_invalid", "failed"]) {
    assert.ok(en[`err.transport.${cause}`] && ru[`err.transport.${cause}`], cause);
  }
  assert.ok(en["err.file.too_large"].includes("{max}"));
  for (const key of Object.keys(en).filter((key) => key.startsWith("err.") || key.startsWith("field."))) {
    const placeholders = (text) => [...text.matchAll(/\{(\w+)\}/g)].map((found) => found[1]).sort().join();
    assert.equal(placeholders(ru[key]), placeholders(en[key]), `${key}: both languages use the same values`);
  }
});

test("template helpers: suggestions only inside an open {{, visible variables, JSON paths and names", () => {
  assert.deepEqual(templateAt("url {{to", 8), { start: 4, query: "to" });
  assert.deepEqual(templateAt("{{", 2), { start: 0, query: "" });
  assert.equal(templateAt("{{token}} x", 11), null);
  assert.equal(templateAt(String.raw`\{{literal`, 11), null);
  assert.equal(templateAt("plain", 5), null);

  const graph = fixture();
  const take = { id: "take", type: "extract", variable: "token", from: "json", expr: "$.token", x: 0, y: 0 };
  const later = { id: "later", type: "extract", variable: "late", from: "status", expr: "", x: 0, y: 0 };
  graph.nodes.splice(2, 0, take, later);
  graph.edges = [{ from: "start", to: "request" }, { from: "request", to: "take" }, { from: "take", to: "later" }, { from: "later", to: "end" }];
  assert.deepEqual(variablesBefore(graph, "later").map((item) => item.name), ["token"]);
  assert.deepEqual(variablesBefore(graph, "end").map((item) => item.name), ["late", "token"]);
  assert.deepEqual(variablesBefore(graph, "request"), []);

  assert.equal(jsonPath(["items", 0, "id"]), "$.items[0].id");
  assert.equal(jsonPath(["first name", "x-y"]), '$["first name"].x-y');
  assert.equal(suggestVariableName(["data", "access-token"], []), "access_token");
  assert.equal(suggestVariableName(["items", 0], ["items"]), "items_2");
  assert.equal(suggestVariableName([0], []), "value");
  assert.equal(suggestVariableName(["2fa"], []), "v_2fa");
  assert.equal(suggestVariableName(["uuid"], []), "uuid_value");
  assert.ok(isIdent("api_2") && !isIdent("2x") && !isIdent("run") && !isIdent("a-b"));
});

test("profiles: precedence, and parameter edits carried into every profile", () => {
  let doc = { ...fixture(), params: [{ name: "api", value: "http://local" }, { name: "device", value: "127.0.0.1:9000" }],
    profiles: [{ name: "Stage", values: { api: "http://stage" } }], profile: "Stage", seed: null };
  assert.deepEqual(effectiveParams(doc, null), { api: "http://local", device: "127.0.0.1:9000" });
  assert.deepEqual(effectiveParams(doc, "Stage"), { api: "http://stage", device: "127.0.0.1:9000" });
  assert.deepEqual(effectiveParams(doc, "Stage", { device: "10.0.0.9:9000", ghost: "x" }), { api: "http://stage", device: "10.0.0.9:9000" });

  doc = renameParam(doc, 0, "base");
  assert.deepEqual(doc.profiles[0].values, { base: "http://stage" });
  // A name shared with another parameter leaves profile values alone: their owner is ambiguous.
  const clash = renameParam(doc, 1, "base");
  assert.deepEqual(clash.profiles[0].values, { base: "http://stage" });
  doc = setParamValue(doc, "Stage", "device", "10.0.0.5:9000");
  assert.equal(doc.profiles[0].values.device, "10.0.0.5:9000");
  doc = setParamValue(doc, "Stage", "device", null);
  assert.ok(!("device" in doc.profiles[0].values), "clearing inherits the default again");
  doc = setParamValue(doc, null, "device", "127.0.0.2:9000");
  assert.equal(doc.params[1].value, "127.0.0.2:9000");
  doc = removeParam(doc, 0);
  assert.deepEqual(doc.profiles[0].values, {});

  const added = addProfile(doc, "Stage");
  assert.equal(added.name, "Stage 2");
  doc = renameProfile(added.doc, "Stage", "Venue");
  assert.equal(doc.profile, "Venue", "the active reference follows a rename");
  doc = removeProfile(doc, "Venue");
  assert.equal(doc.profile, null);
  assert.deepEqual(doc.profiles.map((item) => item.name), ["Stage 2"]);
});

test("secret names are found in any field, once each", () => {
  const doc = fixture();
  doc.nodes[1].request.headers = [["Authorization", "Bearer {{ secret.API_TOKEN }}"], ["X-Key", "{{secret.KEY_2}}"]];
  doc.nodes[1].request.body = "{{secret.API_TOKEN}} and {{params.api}}";
  assert.deepEqual(secretNames(doc), ["API_TOKEN", "KEY_2"]);
  assert.deepEqual(secretNames(fixture()), []);
});

test("an output can feed several nodes: wires add up, and each one is cut, spliced and removed on its own", () => {
  const graph = fixture();
  const parallel = { ...createNode("delay", 240, 160), id: "parallel" };
  // A wire dragged out of a connected output adds a parallel node beside the flow.
  const branched = addBranch(graph, { from: "start", port: "next" }, parallel);
  assert.deepEqual(branched.edges.filter((edge) => edge.from === "start").map((edge) => edge.to), ["request", "parallel"]);
  // Dropped on a node: one more wire, the old one stays.
  const wired = connect(branched, "parallel", "next", "end");
  assert.ok(isWired(wired, "parallel", "next", "end") && isWired(wired, "request", "next", "end"));
  assert.equal(connect(wired, "start", "next", "parallel"), wired, "the same wire twice changes nothing");
  assert.equal(connect(wired, "parallel", "next", "start"), wired, "Start takes no input");
  // The ＋ on one wire splices into that wire only.
  const insert = { ...createNode("log", 300, 160), id: "log" };
  const spliced = addAfter(wired, { from: "start", port: "next", to: "parallel" }, insert);
  assert.deepEqual(spliced.edges.filter((edge) => edge.from === "start").map((edge) => edge.to).sort(), ["log", "request"]);
  assert.ok(isWired(spliced, "log", "next", "parallel"));
  const viaEdge = insertOnEdge(wired, wired.edges.find((edge) => edge.from === "start" && edge.to === "request"), { ...insert, id: "log2" });
  assert.ok(isWired(viaEdge, "start", "next", "parallel"), "the other wire of the output keeps its node");
  // × on one connection removes that wire, not the output's others.
  const cut = disconnect(wired, "start", "next", "parallel");
  assert.ok(isWired(cut, "start", "next", "request") && !isWired(cut, "start", "next", "parallel"));
  assert.equal(disconnect(wired, "start", "next").edges.some((edge) => edge.from === "start"), false, "without a target every wire of the output goes");
  // Removing a node bridges its wire, but never into a duplicate.
  const diamond = connect(connect(addBranch(graph, { from: "start", port: "next" }, { ...createNode("delay", 0, 0), id: "mid" }), "mid", "next", "end"), "start", "next", "end");
  const removed = removeNode(diamond, "mid");
  assert.equal(removed.edges.filter((edge) => edge.from === "start" && edge.to === "end").length, 1);
  // A loop through parallel wires is still refused.
  assert.equal(connect(wired, "end", "next", "start"), wired);
});

test("a send that waits for its reply writes the reply on Next, and only senders and waits retry", () => {
  const ping = { ...createNode("osc", 240, 40), id: "ping" };
  assert.equal(writtenVariable(ping), null, "without a reply nothing is written");
  assert.deepEqual(replyFields(ping), []);
  const asking = { ...ping, reply: { ...defaultReply("osc"), variable: "pong" } };
  assert.deepEqual(writtenVariable(asking), { name: "pong", port: "next" });
  assert.ok(replyFields(asking).includes("args[0]"));
  const udp = { ...createNode("udp", 240, 40), reply: { ...defaultReply("udp"), mode: "regex", pattern: "ACK (.+)" } };
  assert.ok(replyFields(udp).includes("match"), "a pattern's capture is offered");
  assert.ok(defaultReply("osc").bind.endsWith(":0"), "any free port until the user picks one");
  // The reply is known downstream of the send.
  const graph = fixture();
  const doc = { ...graph, nodes: [graph.nodes[0], asking, { ...createNode("log", 440, 40), id: "log" }, graph.nodes[2]],
    edges: [{ from: "start", to: "ping" }, { from: "ping", to: "log" }, { from: "log", to: "end" }] };
  assert.deepEqual(variablesBefore(doc, "log").map((item) => item.name), ["pong"]);
  for (const type of ["http", "tcp", "mqtt", "osc", "udp", "wait_osc", "wait_udp"]) assert.ok(canRetry(createNode(type, 0, 0)), type);
  for (const type of ["delay", "log", "assert_status", "extract", "fork", "start"]) assert.ok(!canRetry(createNode(type, 0, 0)), type);
  assert.ok(DEFAULT_RETRY.attempts >= 2 && DEFAULT_RETRY.attempts <= 10);
});

test("a node added after a lower branch stays in that branch's row instead of landing on another node", () => {
  const graph = fixture();
  const lower = { ...createNode("delay", 240, 200), id: "lower" };
  // The lower branch leads back up to End, in the main row.
  const doc = connect(addBranch(graph, { from: "start", port: "next" }, lower), "lower", "next", "end");
  const spot = placeAfter(doc, { from: "lower", port: "next" });
  assert.equal(spot.y, lower.y, "in the source's row, not End's");
  assert.ok(!doc.nodes.some((node) => Math.abs(node.x - spot.x) < NODE_WIDTH && Math.abs(node.y - spot.y) < NODE_HEIGHT), "on a free spot");
});

test("Wait for this: a received OSC message or MQTT topic becomes a wait that recognises it", () => {
  const wait = waitForOscMessage("0.0.0.0:9000", "/mixer/fader", [
    { type: "int", value: 3 }, { type: "float", value: 0.75 }, { type: "str", value: "main" }, { type: "bool", value: true },
  ]);
  assert.equal(wait.type, "wait_osc");
  assert.equal(wait.bind, "0.0.0.0:9000", "on the port the monitor heard it on");
  assert.equal(wait.address, "/mixer/fader");
  assert.deepEqual(wait.args, [
    { index: 0, op: "eq", value: "3" }, { index: 2, op: "eq", value: "main" }, { index: 3, op: "eq", value: "true" },
  ], "the float, a measurement, is left out");
  assert.deepEqual(validPortsFor(wait.type), ["matched", "timeout"]);
  const many = waitForOscMessage("0.0.0.0:9000", "/x", Array.from({ length: 40 }, (_, value) => ({ type: "int", value })));
  assert.equal(many.args.length, 16, "no more rules than the engine accepts");
  const topic = waitForMqttMessage("192.168.1.20", 1883, "lights/hall/state");
  assert.deepEqual([topic.type, topic.host, topic.port, topic.topic, topic.mode], ["wait_mqtt", "192.168.1.20", 1883, "lights/hall/state", "any"]);
});


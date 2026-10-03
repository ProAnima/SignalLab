import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { LOCALES } from "../src/lib/locales/index.ts";
import { en } from "../src/lib/locales/en.ts";
import { changedProfile, fullProfile, impairNotation, IMPAIR_PRESETS, presetOf, presetProfile, profileLabel } from "../src/lib/impairments.ts";
import { createNode, createNodeIn, routeThroughImpairment, validPortsFor } from "../src/lib/experimentGraph.ts";
import { messageParams } from "../src/lib/errors.ts";

const t = (key, params) => {
  const text = en[key] ?? key;
  return params ? text.replace(/\{(\w+)\}/g, (match, name) => name in params ? String(params[name]) : match) : text;
};

test("presets: each is itself until edited, and has a name and a tooltip in every language", () => {
  for (const preset of IMPAIR_PRESETS) {
    assert.equal(presetOf(presetProfile(preset)), preset);
    for (const { code, dict } of LOCALES) assert.ok(dict[`ns.preset.${preset}`] && dict[`ns.presetHint.${preset}`], `${code}: ${preset}`);
  }
  const edited = changedProfile(presetProfile("4g"), { latency_ms: 61 });
  assert.equal(edited.name, "", "a 4G with other values is not called 4G any more");
  assert.equal(presetOf(edited), null);
  assert.equal(changedProfile(presetProfile("4g"), { latency_ms: 60 }).name, "4g", "the same values keep the name");
  assert.equal(presetOf({ name: "lan", latency_ms: 1, jitter_ms: 1, loss: 0, duplicate: 0, corrupt: 0 }), "lan", "fields left out are the clean ones");
  assert.equal(profileLabel(presetProfile("satellite"), (preset) => `«${preset}»`), "«satellite»");
  assert.equal(profileLabel({ ...fullProfile(), name: "my link" }, () => "?"), "my link");
});

test("a profile without a name reads as the engine's timeline writes it", () => {
  // The same cases as engine/src/netsim.rs, profiles_are_checked_and_say_what_they_do.
  assert.equal(impairNotation(fullProfile()), "clean");
  assert.equal(impairNotation({ ...fullProfile(), latency_ms: 60, jitter_ms: 25, loss: 0.02, rate_kbps: 20000 }), "60 ms ±25 · loss 2% · 20000 kbps");
  assert.equal(impairNotation({ ...fullProfile(), offline: true, loss: 1 }), "offline");
  assert.equal(impairNotation({ ...fullProfile(), burst_start: 0.03, burst_length: 15 }), "bursts 3% × 15");
  const engine = readFileSync("engine/src/netsim.rs", "utf8");
  assert.ok(engine.includes('"60 ms ±25 · loss 2% · 20000 kbps"'), "the engine's test still says the same");
});

test("Route through impairment: a relay in front of the node, the node pointed at it, the wires kept", () => {
  const start = createNode("start", 0, 100);
  const udp = { ...createNode("udp", 300, 100), target: "127.0.0.1:9000" };
  const end = createNode("end", 600, 100);
  const doc = { version: 9, name: "x", params: [], profiles: [], profile: null, seed: null, nodes: [start, udp, end], edges: [{ from: start.id, to: udp.id, port: "next" }, { from: udp.id, to: end.id, port: "next" }] };
  const routed = routeThroughImpairment(doc, udp.id);
  assert.ok(routed);
  const { doc: next, relay } = routed;
  assert.equal(relay.type, "impairment");
  assert.equal(relay.target, "127.0.0.1:9000");
  assert.equal(next.nodes.find((node) => node.id === udp.id).target, relay.listen, "the node now sends to the relay");
  assert.deepEqual(next.edges.map((edge) => [edge.from, edge.to]).sort(), [[start.id, relay.id], [relay.id, udp.id], [udp.id, end.id]].sort());
  const again = routeThroughImpairment(next, udp.id).relay;
  assert.notEqual(again.listen, relay.listen, "a second relay takes another port");
  // Nor a port a wait or an emulator listens on, nor the node's own target.
  const crowded = { ...doc, nodes: [...doc.nodes.map((node) => node.id === udp.id ? { ...node, target: "127.0.0.1:9010" } : node),
    { ...createNode("wait_udp", 0, 0), bind: "127.0.0.1:9011" }, { ...createNode("emulator", 0, 0), emulator: { ...createNode("emulator", 0, 0).emulator, bind: "127.0.0.1:9012" } }] };
  assert.equal(routeThroughImpairment(crowded, udp.id).relay.listen, "127.0.0.1:9013");
  assert.equal(routeThroughImpairment(doc, start.id), null, "only OSC and UDP nodes");
  assert.deepEqual(validPortsFor("impairment"), ["next"]);

  const change = createNodeIn(next, "impairment_change", 0, 0);
  assert.equal(change.relay, relay.id, "a new Change impairment names the document's relay");
  assert.equal(createNodeIn(doc, "emulator_state", 0, 0).emulator, "", "and an Emulator down/up nothing, when there is none");
});

test("a preset in a step's message reads in the reader's language", () => {
  assert.deepEqual(messageParams({ profile: "4g", before: "lan" }, t), { profile: en["ns.preset.4g"], before: en["ns.preset.lan"] });
  assert.deepEqual(messageParams({ profile: "60 ms ±25" }, t), { profile: "60 ms ±25" }, "anything else as it is");
});

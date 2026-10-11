import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { STEPS_KEPT, STEPS_TAIL, stepsMade, timelineRows, writtenBy } from "../src/lib/experimentTimeline.ts";
import { hasRoom, MAX_NODES } from "../src/lib/experimentGraph.ts";
import { planned, ratePoints } from "../src/lib/load.ts";

const engine = (file) => readFileSync(new URL(`../engine/src/${file}`, import.meta.url), "utf8");
const constant = (text, name) => Number(new RegExp(`const ${name}(?:: \\w+)? = ([\\d_]+);`).exec(text)[1].replaceAll("_", ""));

const step = (node_id, extra = {}) => ({ job_id: 1, ts: 0, node_id, state: "passed", detail: "", ...extra });

test("a gap the engine left out is a row of its own, counted", () => {
  const events = [step("a"), step("b"), step("c", { omitted: 118_000 }), step("end")];
  const rows = timelineRows(events);
  assert.deepEqual(rows.map((row) => "omitted" in row ? `…${row.omitted}` : row.step.node_id), ["a", "b", "…118000", "c", "end"]);
  assert.equal(stepsMade(events), 118_004, "every step the run made, shown or not");
  assert.equal(stepsMade([]), 0);
});

test("the steps of a batch write their variables in order", () => {
  assert.equal(writtenBy([step("a"), step("b")]), null);
  assert.deepEqual(writtenBy([step("a", { vars: { x: 1, y: 1 } }), step("b"), step("c", { vars: { x: 2 } })]), { x: 2, y: 1 });
});

test("the editor's limits are the engine's", () => {
  assert.equal(STEPS_KEPT, constant(engine("experiment_flow.rs"), "STEPS_KEPT"));
  assert.equal(STEPS_TAIL, constant(engine("experiment_flow.rs"), "STEPS_TAIL"));
  assert.equal(MAX_NODES, constant(engine("experiment.rs"), "MAX_NODES"));
  const netsim = readFileSync(new URL("../src/views/NetsimView.tsx", import.meta.url), "utf8");
  assert.equal(constant(netsim, "MAX_RELAY_CONNECTIONS"), constant(engine("netsim_tcp.rs"), "MAX_CONNECTIONS"));
});

test("a document has room for nodes up to the engine's limit, not past it", () => {
  const doc = (n) => ({ nodes: Array.from({ length: n }, (_, i) => ({ id: `n${i}` })), edges: [] });
  assert.equal(hasRoom(doc(63)), true);
  assert.equal(hasRoom(doc(64)), false);
  assert.equal(hasRoom(doc(60), 4), true);
  assert.equal(hasRoom(doc(60), 5), false, "a paste of five onto sixty would make 65");
});

test("a Steps profile typed past its bounds draws at most the engine's 100 levels", () => {
  // Before: ten million steps were twenty million points, five seconds a draw.
  const started = performance.now();
  for (const steps of [1e7, 1e9, Infinity, Number.NaN, -3, 2.5]) {
    const points = ratePoints({ shape: "steps", from: 1, step: 1, every_ms: 1000, steps });
    assert.ok(points.length <= 200, `${steps}: ${points.length} points`);
    assert.ok(Number.isFinite(planned({ shape: "steps", from: 1, step: 1, every_ms: 1000, steps })));
  }
  assert.ok(performance.now() - started < 200, "drawn at once");
  assert.equal(ratePoints({ shape: "steps", from: 1, step: 1, every_ms: 1000, steps: 3 }).length, 6, "within the bounds, as before");
});

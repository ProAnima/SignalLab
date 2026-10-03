import test from "node:test";
import assert from "node:assert/strict";
import { LOCALES } from "../src/lib/locales/index.ts";
import { DEFAULT_LOAD, durationMs, expected, LOAD_METRICS, LOAD_SHAPES, loadBadge, loadNotation, metricOf, peakRate, planned, rateAt, ratePoints, withShape } from "../src/lib/load.ts";

test("each shape adds up to the requests the engine sends", () => {
  // The same profiles as engine/src/load.rs, each_shape_adds_up_to_its_requests_and_spaces_them_by_its_rate.
  assert.equal(planned({ shape: "constant", rate: 100, duration_ms: 1000 }), 100);
  assert.equal(planned({ shape: "ramp", from: 0, to: 100, duration_ms: 2000 }), 100);
  assert.equal(planned({ shape: "ramp", from: 100, to: 0, duration_ms: 2000 }), 100);
  assert.equal(planned({ shape: "steps", from: 10, step: 10, every_ms: 1000, steps: 3 }), 60);
  assert.equal(planned({ shape: "spike", base: 10, peak: 100, at_ms: 1000, spike_ms: 500, duration_ms: 2000 }), 65);
  assert.equal(planned({ shape: "steps", from: 10, step: -10, every_ms: 1000, steps: 2 }), 10, "a rest at nothing adds nothing");
  assert.equal(planned({ shape: "poisson", rate: 200, duration_ms: 10_000 }), 2000, "on average");
  assert.equal(planned({ shape: "constant", rate: 0.1, duration_ms: 1000 }), 1, "the first is due at once");
  assert.equal(planned({ shape: "ramp", from: 0, to: 0, duration_ms: 1000 }), 0);
  // 20 → 100/s over 1.5 s, the integration test's ramp.
  assert.equal(planned({ shape: "ramp", from: 20, to: 100, duration_ms: 1500 }), 90);
  assert.equal(expected({ shape: "constant", rate: 40, duration_ms: 2200 }), 88);
});

test("the chart's line is the rate over time", () => {
  const spike = { shape: "spike", base: 10, peak: 100, at_ms: 1000, spike_ms: 500, duration_ms: 2000 };
  assert.deepEqual(ratePoints(spike), [[0, 10], [1, 10], [1, 100], [1.5, 100], [1.5, 10], [2, 10]]);
  assert.deepEqual([0.5, 1.2, 1.7, 2.5].map((t) => rateAt(spike, t)), [10, 100, 10, 0]);
  assert.equal(rateAt({ shape: "ramp", from: 0, to: 100, duration_ms: 2000 }, 0.5), 25);
  assert.equal(peakRate({ shape: "steps", from: 10, step: 10, every_ms: 1000, steps: 3 }), 30);
  assert.equal(durationMs({ shape: "steps", from: 10, step: 10, every_ms: 1000, steps: 3 }), 3000);
});

test("another shape keeps how long the load runs and the rate it reaches", () => {
  const ramp = { shape: "ramp", from: 0, to: 200, duration_ms: 60_000 };
  for (const shape of LOAD_SHAPES) {
    const next = withShape(ramp, shape);
    assert.equal(next.shape, shape);
    assert.equal(durationMs(next), 60_000, shape);
    assert.equal(peakRate(next), 200, shape);
    assert.ok(planned(next) > 0, shape);
  }
  assert.equal(withShape(ramp, "ramp"), ramp, "the same shape, the same profile");
});

test("a profile as notation: the rates it goes through; the node's badge, the highest", () => {
  assert.equal(loadNotation({ shape: "ramp", from: 10, to: 200, duration_ms: 60_000 }), "10→200/s");
  assert.equal(loadNotation({ shape: "constant", rate: 40, duration_ms: 2200 }), "40/s");
  assert.equal(loadNotation({ shape: "poisson", rate: 12.34, duration_ms: 1000 }), "~12.3/s");
  assert.equal(loadNotation({ shape: "steps", from: 10, step: 10, every_ms: 1000, steps: 5 }), "10→50/s");
  assert.equal(loadNotation({ shape: "spike", base: 10, peak: 100, at_ms: 1000, spike_ms: 500, duration_ms: 2000 }), "10↑100/s");
  assert.equal(loadNotation(DEFAULT_LOAD.profile), "0→100/s");
  assert.equal(loadBadge({ shape: "ramp", from: 10, to: 200, duration_ms: 60_000 }), "200/s");
  assert.equal(loadBadge({ shape: "steps", from: 50, step: -10, every_ms: 1000, steps: 5 }), "50/s");
  assert.equal(loadBadge({ shape: "poisson", rate: 12.34, duration_ms: 1000 }), "~12.3/s");
});

test("every metric and shape has a name in every language", () => {
  for (const { code, dict } of LOCALES) {
    for (const metric of LOAD_METRICS) assert.ok(dict[`exp.metric.${metric}`], `${code}: ${metric}`);
    for (const shape of LOAD_SHAPES) assert.ok(dict[`exp.loadShape.${shape}`], `${code}: ${shape}`);
  }
  assert.equal(metricOf({ p95_ms: 12, missed: 3 }, "missed"), 3);
  assert.equal(metricOf({ p95_ms: 12, missed: 3 }, "p95_ms"), 12);
});

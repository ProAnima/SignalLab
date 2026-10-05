---
title: "Design: load, thresholds, comparison"
---

# Milestone 7 — load profiles, thresholds, run comparison

::: info A design note
Written while the feature was being designed and kept as the record of why it is
the way it is. What shipped is described in the user documentation, which is
the one to trust where the two differ.
:::

Status: delivered for HTTP, 2026-10-03. ROADMAP §7 says what and why; this says
how. What changed on the way: a load's progress is the step state `load` (the
interface's `exp.loading` was taken); the node's badge shows the highest rate
(`⚡120/s`) and its tooltip the profile, the requests and the time, so the node's
name keeps its room; a run is named by its report's file name, never a path; MCP
reads runs back with `list_runs` and `compare_runs`.

**Goal.** A run measures, not only passes: an HTTP node can send a load shaped
over time, the step passes or fails on thresholds (`p95 < 300 ms`,
`errors < 1 %`), and two runs of an experiment compare side by side.

**Done when** a ramp with thresholds fails for the right reason, and its
comparison with an earlier run shows the regression.

## The document (version 9)

A node setting next to Retry and Repeat — not a node kind, so every tool that
reads nodes (the catalogue, MCP, `signallab nodes`) sees one more field:

```json
{ "type": "http", "request": { … },
  "load": {
    "profile": { "shape": "ramp", "from": 10, "to": 200, "duration_ms": 60000 },
    "concurrency": 64,
    "thresholds": [
      { "metric": "p95_ms", "op": "lt", "value": 300 },
      { "metric": "error_rate", "op": "lt", "value": 1 }
    ] } }
```

| Shape | Fields | Rate over time |
| --- | --- | --- |
| `constant` | `rate`, `duration_ms` | `rate` throughout |
| `ramp` | `from`, `to`, `duration_ms` | linear `from` → `to` |
| `steps` | `from`, `step`, `every_ms`, `steps` | `from`, `from + step`, … each for `every_ms` |
| `spike` | `base`, `peak`, `at_ms`, `spike_ms`, `duration_ms` | `base`, `peak` from `at_ms` for `spike_ms`, `base` again |
| `poisson` | `rate`, `duration_ms` | arrivals at random, `rate` on average, drawn from the run's seed |

Rates are requests per second, 0.1 … 100 000 (the burst's bounds); a profile
lasts what a run may (`RUN_LIMIT`, 5 min) — a soak of hours needs that limit
raised first, which is a decision of its own; `concurrency` is 1 … 512. Load applies to HTTP nodes only
(OSC, UDP and MQTT later); a node has Load or Repeat, not both, and Load has no
Retry — a failed request is counted, not tried again. Files before version 9
have no `load` and open unchanged.

Thresholds: `metric` is one of `p50_ms`, `p90_ms`, `p95_ms`, `p99_ms`,
`mean_ms`, `max_ms`, `error_rate` (%), `rps` (achieved), `missed`; `op` is
`lt`, `le`, `gt`, `ge`; `value` a number.

## The engine

- **Schedule.** The burst's pacer generalised: the n-th request is due where the
  profile's cumulative count reaches n. Constant, ramp, steps and spike are
  piecewise linear, so the cumulative count is piecewise quadratic and inverts
  exactly per segment; Poisson draws exponential gaps from the run's seed
  (per node, as Repeat's jitter does). A request still waiting for a worker
  50 ms past its moment is skipped and counted as `missed`, as in the burst.
- **Requests** are the node's request rendered once (templates read at the
  start of the step), sent with the run's cookie jar and one Digest memory,
  `concurrency` at a time; the Inspector gets a sampled share through a gate.
- **Metrics** (`LoadMetrics` on the step's event, so in the report too): sent,
  ok, failed, missed; achieved rps; latency min/mean/max and p50/p90/p95/p99
  (`LatencyHistogram`); error rate; status counts (`200`, `503`, `timeout` …);
  bytes received per second; per second of the run: sent, failed, mean
  latency (for the charts); a coarse latency histogram.
- **Progress**: a `loading` step at most once a second — sent, rps, p95,
  failed — so the timeline moves while the load runs.
- **Thresholds** are read after the last answer: each has its actual value
  and a verdict on the event; the step fails with `load.threshold`
  (`metric`, `op`, `value`, `actual`) for the first that does not hold.
- **Stop** ends the load at once: requests in flight are dropped with the
  task, as the burst's are.

## Runs and comparison

- `experiment_runs { name?, limit }`: the run reports in `<data>/runs`, newest
  first — path, experiment, started, outcome, seed, and each load step's
  headline numbers — read from the files, so the command line's runs count.
- `experiment_compare { a, b }`: a pure function over two reports
  (`experiment_compare.rs`): per load step, matched by node id, each metric
  before and after with the change (absolute and %), and whether each
  threshold held in either run.

## The interface

- The HTTP node's **Load** section (beside Repeat): the shape, its numbers, a
  chart of the rate over time with the requests it adds up to, concurrency,
  and thresholds as rows. The node's badge says `⚡ 10→200/s 60 s`.
- While it runs the timeline row counts up; afterwards the node's properties
  show the result: the numbers, the latency histogram, rps and errors per
  second, status counts, and each threshold ✓ or ✗ with its actual value.
- **Compare** in the timeline: with the previous run of this experiment by
  default, or any earlier one; a table per load step, a regression in red.

## The command line and MCP

`signallab run` prints each load step's numbers and threshold verdicts; a
failed threshold is a failed test in the JUnit report, its message the metric
and the values. The catalogue (`cli/src/catalog.rs`) describes `load` on the
HTTP node; `cli/tests/nodes.rs` runs an HTTP node under a short load with
thresholds, here and on a server.

## Order of work

1. Engine: `Load` in the document (v9), validation, schedule, metrics,
   thresholds, progress — unit tests for the schedule and the thresholds, an
   integration test of a ramp against loopback that fails its p95 threshold
   for the right reason.
2. Interface: the Load section with its chart, the badge, the result in the
   properties, the timeline's progress; the tour runs a short load.
3. Runs and Compare: the two commands, the compare dialog, the tour compares
   two runs.
4. The command line, MCP, the catalogue, `nodes.rs`, docs.

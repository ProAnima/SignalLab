---
title: Load testing
description: Send an HTTP node's request on a load profile — constant, ramp, steps, spike or random arrivals — measure latencies, errors and the rate achieved, judge them with thresholds and compare two runs.
---

# Load testing an HTTP request

An [[ui:exp.node.http]] node can send its request many times over, on a
profile of requests per second, many at once — and measure what comes back:
latency percentiles, errors, the rate it reached. Thresholds decide whether the
step passes, and [[ui:exp.compare]] puts the numbers beside an earlier run's.

A load is a setting of the node, not a node of its own: the rest of the
experiment — emulators, impairment relays, other branches — runs around it as
usual.

## Putting a request under load {#turn-on}

1. Select an [[ui:exp.node.http]] node and fill in its request.
2. In its properties, turn on [[ui:exp.loadOn]].
3. Choose a [[ui:exp.loadShape]] and its numbers. The chart under them,
   [[ui:exp.loadChart]], draws the rate and says how many requests it adds up
   to, in how many seconds.
4. Set [[ui:exp.loadConcurrency]] — how many requests may be in flight at once.
5. Add or change [[ui:exp.thresholds]].
6. Run the experiment.

A load starts as a [[ui:exp.loadShape.ramp]] from 0 to 100 requests per
second over 30 000 ms, 32 at once, with two thresholds:
[[ui:exp.metric.p95_ms]] < 500 ms and [[ui:exp.metric.error_rate]] < 1 %.

**Load replaces Repeat and Retry.** Turning it on turns them off, and a node
with load and either of them is refused (`node.load_alone`): a failed request
is counted, not tried again. Only an HTTP request can run under load
(`node.load_unsupported`).

**The request is read once.** Its templates are resolved when the step starts,
so every request of the load is the same one: `{{counter}}` and `{{uuid}}` take
one value for all of them. See [templates](data.md#templates).

**One client for the whole load.** The requests share the run's cookie jar when
[[ui:exp.cookies]] is on, and one Digest memory, so one challenge answers them
all. Each request has the node's own timeout.

**The Inspector gets a sample:** at most one exchange every 100 ms, so a load
does not flood the [[ui:dock.inspector]].

## Profiles {#profiles}

| [[ui:exp.loadShape]] | Settings | The rate over time |
| --- | --- | --- |
| [[ui:exp.loadShape.constant]] | [[ui:exp.loadRate]], [[ui:exp.loadDuration]] | the rate throughout |
| [[ui:exp.loadShape.ramp]] | [[ui:exp.loadFrom]], [[ui:exp.loadTo]], [[ui:exp.loadDuration]] | in a straight line from one rate to the other |
| [[ui:exp.loadShape.steps]] | [[ui:exp.loadFrom]], [[ui:exp.loadStepBy]], [[ui:exp.loadEvery]], [[ui:exp.loadSteps]] | the first rate, then one step more at each level, every level for the same time |
| [[ui:exp.loadShape.spike]] | [[ui:exp.loadBase]], [[ui:exp.loadPeak]], [[ui:exp.loadAt]], [[ui:exp.loadSpikeFor]], [[ui:exp.loadDuration]] | the base rate, the peak for a while from a given moment, then the base rate again |
| [[ui:exp.loadShape.poisson]] | [[ui:exp.loadRate]], [[ui:exp.loadDuration]] | arrivals at random, the rate on average |

Switching the shape keeps what carries over: how long it runs and the highest
rate it reaches.

### Limits {#limits}

| Setting | Range |
| --- | --- |
| [[ui:exp.loadRate]] of [[ui:exp.loadShape.constant]] and [[ui:exp.loadShape.poisson]], [[ui:exp.loadPeak]] | 0.1–100 000 requests/s |
| [[ui:exp.loadFrom]], [[ui:exp.loadTo]], [[ui:exp.loadBase]] | 0–100 000 requests/s |
| Every level of [[ui:exp.loadShape.steps]], the last one included | 0–100 000 requests/s; the step may be negative |
| [[ui:exp.loadDuration]], [[ui:exp.loadEvery]] | 100–300 000 ms |
| [[ui:exp.loadSteps]] | 1–100, and all the levels together at most 300 000 ms |
| A spike | longer than 0 ms, and over by the end of the duration |
| [[ui:exp.loadConcurrency]] | 1–512 |
| [[ui:exp.thresholds]] | at most 16, each value a number, 0 or more |

A profile that adds up to no request at all is refused
(`load.nothing_planned`). The rates are those of the
[HTTP burst](../protocols/http.md).

A profile may last as long as a whole run, 300 s — but the run's
[time limit](flow.md#limit) counts every step, so leave room for the rest of
the experiment.

### How many requests {#planned}

A profile's requests are its rate added up over time:

| Profile | Requests |
| --- | --- |
| [[ui:exp.loadShape.constant]], 100/s for 1000 ms | 100 |
| [[ui:exp.loadShape.ramp]], 0 → 100/s over 2000 ms | 100 |
| [[ui:exp.loadShape.steps]], from 10/s up by 10/s, 3 levels of 1000 ms | 60 (10 + 20 + 30) |
| [[ui:exp.loadShape.spike]], 10/s with 100/s from 1000 ms for 500 ms, 2000 ms in all | 65 |
| [[ui:exp.loadShape.poisson]], 200/s for 10 000 ms | 2000 on average |

## The schedule {#schedule}

The n-th request is due at the moment the profile's count reaches n — the first
one at once. Every moment is computed from the start of the load, so a late
wake-up never shifts the requests after it, and the rate the profile describes
is the rate asked for.

[[ui:exp.loadShape.poisson]] draws the gaps between arrivals at random, from
the run's seed: the same seed gives the same moments, so a random load can be
repeated exactly. See [seeds](runs.md#seeds).

**Missed requests.** At most [[ui:exp.loadConcurrency]] requests are in flight.
When every one of them is still waiting for its answer, the next request waits
for a free slot. If it would go out more than 50 ms after its moment, it is not
sent late: it is skipped and counted as missed, with every other request that
fell due meanwhile, and the load goes on with the first one still on time. Many
missed requests mean the server, or [[ui:exp.loadConcurrency]], could not keep
up with the profile.

## While it runs {#progress}

Once a second the timeline shows the step as [[ui:exp.load]], with the seconds
gone, the requests sent, the rate over the last second, p95 so far and the
failed requests. [[ui:common.stop]] ends the load at once and drops the
requests in flight; a failure in another branch ends it within a second.

## What is measured {#metrics}

After the last answer the step has its measurements, kept on its last
timeline event and in the [run report](runs.md#report):

| Measurement | What |
| --- | --- |
| planned | the requests the profile adds up to ([[ui:exp.loadShape.poisson]]: on average) |
| sent | requests that were answered or failed |
| ok | answered with a 2xx status |
| failed | any other status, or no answer at all |
| missed | due while every slot was busy, and skipped |
| rps | requests sent per second: sent ÷ the profile's duration — or ÷ the time until the last request went out, when that was later |
| error_rate | failed, in % of sent |
| min, mean, max | the fastest, the average and the slowest request, ms |
| p50, p90, p95, p99 | the latency that 50, 90, 95 and 99 % of requests were at or under, ms |
| received_bytes | body bytes received in all |
| statuses | requests by status (`200`, `503`) and, without one, by cause (`timeout`, `refused`, `reset` …) |
| seconds | each second of the profile: requests sent, failed, their mean latency |
| histogram | requests by latency, up to 1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10 000 ms, and slower |

A request's latency runs from sending it to having read its whole answer, and
a failed request counts with the time it took to fail. The percentiles are read
from logarithmic buckets 1 % wide and are within 0.5 % of the true value,
however long the load runs.

## Thresholds {#thresholds}

A threshold is a row of [[ui:exp.thresholdMetric]], [[ui:exp.thresholdOp]] and
[[ui:exp.thresholdValue]]; [[ui:exp.thresholdAdd]] adds one.

| [[ui:exp.thresholdMetric]] | Read in |
| --- | --- |
| [[ui:exp.metric.p50_ms]], [[ui:exp.metric.p90_ms]], [[ui:exp.metric.p95_ms]], [[ui:exp.metric.p99_ms]] | ms |
| [[ui:exp.metric.mean_ms]], [[ui:exp.metric.max_ms]] | ms |
| [[ui:exp.metric.error_rate]] | % of the requests sent |
| [[ui:exp.metric.rps]] | requests per second achieved |
| [[ui:exp.metric.missed]] | requests |

[[ui:exp.thresholdOp]] is one of `<`, `≤`, `>`, `≥`. A few common ones:

| [[ui:exp.thresholdMetric]] | [[ui:exp.thresholdOp]] | [[ui:exp.thresholdValue]] | The step fails when |
| --- | --- | --- | --- |
| [[ui:exp.metric.p95_ms]] | `<` | 300 | one request in twenty or more took 300 ms or longer |
| [[ui:exp.metric.error_rate]] | `<` | 1 | 1 % or more of the requests failed |
| [[ui:exp.metric.rps]] | `≥` | 180 | the server could not take 180 requests a second |
| [[ui:exp.metric.missed]] | `≤` | 0 | a single request had to be skipped |

In a file a threshold is `{ "metric": "p95_ms", "op": "lt", "value": 300 }`;
the metrics are `p50_ms`, `p90_ms`, `p95_ms`, `p99_ms`, `mean_ms`, `max_ms`,
`error_rate`, `rps` and `missed`, the comparisons `lt`, `le`, `gt` and `ge`.

The thresholds are read after the last answer, in their order. The step fails
on the first that does not hold (`load.threshold`), its message giving the
threshold and the value measured, and the run fails with it. Without thresholds a load passes whatever it measured. When
another branch's failure ended the load early, that failure is the run's, not
a threshold.

## The result {#result}

When the step passes, the timeline sums it up: the requests, the rate, p95
and the share that failed. Select the node: its properties show
[[ui:exp.loadResult]] —

- each threshold, ✓ [[ui:exp.thresholdHeld]] or ✕ [[ui:exp.thresholdBroken]],
  with the value measured;
- [[ui:http.sent]], [[ui:exp.loadRps]], [[ui:exp.loadErrors]] with their share,
  [[ui:http.missed]];
- [[ui:http.p50]], [[ui:http.p90]], [[ui:http.p95]], [[ui:http.p99]],
  [[ui:http.avg]], [[ui:http.max]];
- [[ui:exp.loadPerSecond]]: the requests of each second, the failed ones in
  red, and their mean latency as a line;
- [[ui:exp.loadLatencies]]: how many requests took how long;
- the statuses and causes, each with its count.

The command line prints the same numbers and each threshold's verdict; see
[`signallab run`](../automation/cli.md#cli-run).

## Comparing two runs {#compare}

1. Run the experiment twice, or more.
2. In the timeline, press [[ui:exp.compare]]. It is there once a run has saved
   its report, and is disabled while a run goes.
3. The latest run is [[ui:exp.compareAfter]], the one before it
   [[ui:exp.compareBefore]]; either list picks another run.

The lists hold the runs of this experiment — by its name — from the reports in
the data folder, newest first, at most 50: each with its date and time, how it
ended and its seed. Runs from the command line are there too when it used the
same data folder. Renaming the experiment starts a new history.

For each load step, matched by node, a table shows every metric
[[ui:exp.compareBefore]], [[ui:exp.compareAfter]] and the
[[ui:exp.compareChange]], in the unit and in %. A change the wrong way by 5 %
or more — slower, more errors, more missed requests, a lower rate — is a
regression and shows in red; from nothing to something counts too. Under the
table, each threshold's verdict in both runs. A load step that only one of the
runs has is marked [[ui:exp.compareOnlyBefore]] or [[ui:exp.compareOnlyAfter]],
without changes. Runs without load steps show [[ui:exp.compareNoLoad]].

From a script, [`experiment_runs`](../api/commands.md#experiment_runs) lists the
runs and [`experiment_compare`](../api/commands.md#experiment_compare) compares
two, by their report's file name; `signallab mcp` offers the same to an
assistant ([MCP](../automation/mcp.md)).

## Checks after a load {#checks-after}

A load leaves no response of its own: it is measured, not checked. A check or
[[ui:exp.node.extract]] after it needs another request without load before it
on every path, or the experiment does not run (`graph.needs_http`). To check
one answer of the API under load, put a plain [[ui:exp.node.http]] after the
load, or in a parallel branch beside it.

[[ui:exp.sendNow]] on a node under load sends its request once.

---
title: How a run moves
description: Start and End, outputs and wires, parallel branches and Join, branching, Retry, Repeat and Loop, waits that listen from the start of the run, and what is checked before a run.
---

# How a run moves

A run starts at [[ui:exp.node.start]], follows wires from node to node and is
complete when every branch has finished and [[ui:exp.node.end]] has been
reached. This page explains the rules it follows; what each node does is in
[the nodes' reference](nodes.md), and the values that travel with it in
[data](data.md).

## Start and End {#start-end}

An experiment has exactly one [[ui:exp.node.start]] and one
[[ui:exp.node.end]].

- [[ui:exp.node.start]] has no input. It passes at once, and its row in the
  timeline gives the run's seed. Its output may have several wires: the
  experiment then begins with parallel branches.
- Every branch that reaches [[ui:exp.node.end]] stops there. End shows as
  running from the first arrival and passes once, after the last branch has
  finished — and not at all if any step failed. That pass is what makes the
  run [[ui:exp.passed]].
- A run in which every branch finished without error but none reached End
  fails with `run.no_end`.

## Outputs and wires {#outputs}

A node's step ends by choosing an output, and the run follows every wire of
that output. Most nodes have one output, [[ui:exp.outputPort]]; some choose
between several:

| Node | Outputs that must be wired | Outputs that may be wired |
| --- | --- | --- |
| [[ui:exp.node.end]] | — | — |
| [[ui:exp.node.fork]] | [[ui:exp.branch1]], [[ui:exp.branch2]] | — |
| [[ui:exp.node.branch_status]], [[ui:exp.node.branch_value]] | [[ui:exp.yes]], [[ui:exp.no]] | — |
| Every wait ([[ui:exp.node.wait_osc]], [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_http]], [[ui:exp.node.wait_ws]]) | [[ui:exp.portMatched]] | [[ui:exp.portTimeout]] |
| [[ui:exp.node.loop]] | [[ui:exp.portBody]], [[ui:exp.portDone]] | [[ui:exp.portLimit]] |
| Every other node | [[ui:exp.outputPort]] | — |

To connect, drag from an output onto a node; dropped on empty canvas, it adds
a new node there. Dragging from an output that already has a wire adds another
one. [[ui:exp.addNext]], the <kbd>A</kbd> key and the ＋ on a wire insert a node
into the existing wire instead.

An unfinished graph is a draft: it saves, but it does not run.
[[ui:exp.needsLinks]] in the toolbar says what is missing and shows the node.
See [what is checked before a run](#validation).

## Parallel branches {#parallel}

### Several wires from one output {#fan-out}

When an output has several wires — [[ui:exp.node.start]]'s included — every
node they lead to runs at the same time. The first wire continues the branch;
each further wire starts a parallel branch. Each branch carries its own copy of
the variables and of the latest HTTP response, so what one branch sets or
receives is not seen by the others.

### Parallel branch and Join {#fork-join}

[[ui:exp.node.fork]] passes at once and leaves through both
[[ui:exp.branch1]] and [[ui:exp.branch2]] — the same as two wires from one
output, drawn as a node.

[[ui:exp.node.join]] waits for **every** wire that leads into it, then
continues as one branch with the copies merged in the order of those wires:

- the variables of all of them — on a name two branches both set, the wire
  listed later in the experiment wins;
- the HTTP response of the last wire, in that order, that brings one;
- for the waits after it, the earliest of their latest actions.

The order of the wires decides, never which branch happened to finish first.

::: warning Join only what runs in parallel
A Join counts its wires, however they came to run in parallel: from a
[[ui:exp.node.fork]], from several wires of one output, from separate paths.
Behind [[ui:exp.yes]] and [[ui:exp.no]] of a branch only one path runs, so a
Join fed by both waits for a branch that never comes: the run fails with
`run.join_waiting`, naming how many wires were never followed. To bring
alternative paths together, wire them straight into the next node.
:::

A node that is not a Join, reached by two parallel branches, runs once for
each of them.

### When a step fails {#failure}

The first failure fails the run. The other branches start no new step: a
repeat or a load ends early, any other step they are in runs to its end. A
failure they meet meanwhile is reported in the timeline but is not the run's
error. A step that ran into a timeout while a
[[ui:exp.portTimeout]] wire was there did not fail — see [waits](#timeout).

## Branching {#branching}

| Node | Leaves through [[ui:exp.yes]] when |
| --- | --- |
| [[ui:exp.node.branch_status]] | the latest HTTP response on this path has the status given |
| [[ui:exp.node.branch_value]] | its comparison holds — see [comparing values](data.md#compare) |

Otherwise each leaves through [[ui:exp.no]]. A status branch needs an HTTP
request before it on every path; a branch on value needs the names it reads
known there. When the paths after [[ui:exp.yes]] and [[ui:exp.no]] meet again, the node where they meet runs once, and only the variables set on
both paths are known there ([where a variable is known](data.md#visibility)).

## Retry {#retry}

A step that sends or listens can try again when it fails: turn on
[[ui:exp.retryOn]] in its properties.

| Setting | What | Range | First given |
| --- | --- | --- | --- |
| [[ui:exp.attempts]] | attempts in all, the first included | 1–10 | 3 |
| [[ui:exp.retryDelay]] | the pause before the second attempt | 0–60 000 ms | 500 |
| [[ui:exp.backoff]] | [[ui:exp.backoff.fixed]]: every pause the same; [[ui:exp.backoff.exponential]]: each pause twice the one before | — | [[ui:exp.backoff.fixed]] |

- Retry applies to [[ui:exp.node.http]], [[ui:exp.node.tcp]],
  [[ui:exp.node.mqtt]], [[ui:exp.node.osc]], [[ui:exp.node.udp]],
  [[ui:exp.node.ws_connect]], [[ui:exp.node.ws_send]] and every wait. Other
  nodes refuse it (`node.retry_unsupported`), and an HTTP request under
  [load](load.md) takes no Retry.
- No single pause is longer than 60 s, whatever the doubling gives.
- Each failed attempt appears in the timeline as [[ui:exp.retry]], with its
  number and its reason. The step then passes, or fails with the last
  attempt's reason.
- Only the execution is repeated. A field whose template does not resolve
  fails at once.
- A send that waits for its reply sends again. A wait waits again, counting
  from the branch's latest action as before.
- A wait with a [[ui:exp.portTimeout]] wire does not fail on a timeout, so it
  is not retried: it follows [[ui:exp.portTimeout]].
- [Stop](#stop) ends a pause at once.

## Repeat {#repeat}

A step that sends can send again and again — a heartbeat, a poll, a steady
stream — without a loop in the graph: turn on [[ui:exp.repeatOn]].

| Setting | What | Range | First given |
| --- | --- | --- | --- |
| [[ui:exp.repeatBy]] | [[ui:exp.repeatBy.count]] or [[ui:exp.repeatBy.duration]] | — | [[ui:exp.repeatBy.count]] |
| [[ui:exp.repeatCount]] | sends in all, the first included | 2–10 000 | 10 |
| [[ui:exp.repeatDuration]] | how long to keep sending, from the first send | 1–300 000 ms | 10 000 |
| [[ui:exp.repeatInterval]] | the pause between two sends | 10–60 000 ms | 1000 |
| [[ui:exp.repeatJitter]] | each pause is up to this much longer, at random | 0–60 000 ms | 0 |

- Repeat applies to [[ui:exp.node.http]], [[ui:exp.node.tcp]],
  [[ui:exp.node.mqtt]], [[ui:exp.node.osc]], [[ui:exp.node.udp]] and
  [[ui:exp.node.ws_send]] (`node.repeat_unsupported` elsewhere). An HTTP
  request has either Repeat or [load](load.md), not both.
- Each send is made as a single one would be: its templates are read again —
  `{{counter}}` is the send's number, `{{now}}` its time — and Retry, when on,
  applies to each send. A send that waits for a reply waits for its own.
- For a time, a send is made only if it can start before the time is up.
- The jitter is drawn from the run's seed: the same seed gives the same
  pauses.
- The timeline reports progress as [[ui:exp.repeating]] at most once a second.
  The step passes after the last send, with that send's outcome; a send that
  fails for good fails the step.
- A failure on another branch ends the sends; [Stop](#stop) ends a pause at
  once.

It must fit in a run: the sends and their longest pauses
(`(count − 1) × (interval + jitter)`) at most 300 s (`node.repeat_too_long`),
and a timed repeat at most 10 000 sends (`node.repeat_too_many`).

## Loop {#loop}

[[ui:exp.node.loop]] runs the steps on its [[ui:exp.portBody]] output again and
again; the last of them is wired back to the Loop.

| Setting | What | Range |
| --- | --- | --- |
| [[ui:exp.loopMax]] | the most iterations | 1–1000 |
| [[ui:exp.loopUntilOn]] | an exit condition: [[ui:exp.value]], [[ui:exp.operator]], [[ui:exp.expected]], as in [[ui:exp.node.assert_value]] | optional |

1. Reached from outside, the Loop starts iteration 1 on [[ui:exp.portBody]].
2. Each time the body comes back, the exit condition is read — after the
   iteration, so the body always runs at least once and can set what it tests.
3. When the condition holds, the Loop leaves through [[ui:exp.portDone]].
4. Otherwise the next iteration starts, while any are left.
5. When the iterations run out first, the Loop leaves through
   [[ui:exp.portLimit]] if it is wired, and fails the run with `loop.limit`
   if it is not. Without a condition, the body runs every iteration and the
   Loop leaves through [[ui:exp.portDone]].

Inside the body `{{counter}}` is the iteration's number, since every node
counts its own executions. The condition and the steps after [[ui:exp.portDone]] or [[ui:exp.portLimit]] may
use what every iteration of the body sets — a status the body extracts, for
instance; the body itself sees only what was known when the Loop was reached.

The template [[ui:exp.templatePoll]] asks a device for its status every 0.3 s
until it answers `ready`, at most 10 times.

### What a body may hold {#loop-body}

A body runs as one branch, one iteration after another. The wire back to the
Loop is the only cycle an experiment may have; any other is `graph.cycle`.

| Rule | Error |
| --- | --- |
| Something on [[ui:exp.portBody]] leads back to the Loop | `loop.no_return` |
| Each output in the body has one wire | `loop.body_parallel` |
| Every output in the body leads on in the body or back to the Loop | `loop.body_leaves` |
| Only the Loop's [[ui:exp.portBody]] output leads into the body | `loop.body_entered` |
| No [[ui:exp.node.start]], [[ui:exp.node.end]], [[ui:exp.node.fork]], [[ui:exp.node.join]] or other [[ui:exp.node.loop]] in the body | `loop.body_unsupported` |

## Waits {#waits}

A wait passes when a message it is waiting for arrives:
[[ui:exp.node.wait_osc]], [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]],
[[ui:exp.node.wait_http]] and [[ui:exp.node.wait_ws]]. What each one matches is
in [the nodes' reference](nodes.md); this is how they listen.

### Listening from the start {#listening}

The run opens what its waits listen on **before its first step**, so a reply
that is quicker than the next step is not missed:

| Wait | Opened before the first step |
| --- | --- |
| OSC, UDP | one UDP socket per [[ui:exp.listenOn]] address, shared by every wait on it |
| HTTP request | one listener per address — the run's HTTP [emulator](faults.md#emulator) when one is there, otherwise one that answers `204` |
| MQTT | one connection per broker and topic filter, subscribed; retained messages the broker replays then are ignored |
| WebSocket | nothing: it reads the connection a [[ui:exp.node.ws_connect]] opened when it ran |

Because they open first, these addresses are fixed before the run: an OSC,
UDP or HTTP wait listens on a literal `IP:port` with a port other than 0, and
an MQTT wait's broker and topic take parameters only. A port that cannot be
opened — taken, or not an address of this computer — stops the run before any
traffic, at that wait's field. Everything is closed when the run ends, in any
way.

### Which messages count {#counting}

A wait considers the messages that arrived after the **latest action on its
branch** started — the latest request, message, publish, WebSocket connect or
send — or, before any action, after the run started. A message from before the
request does not count, and a Delay or a Log between the request and the wait
does not hide its reply. After a Join, the earliest latest action of the merged
branches counts.

A wait takes the first message that matches and consumes it: two waits never
match the same message.

Each socket, subscription or connection keeps at most 1024 messages and 64 MiB
for its waits; past that, the oldest are dropped and counted.

### Timeout {#timeout}

[[ui:exp.waitTimeout]] is 1–120 000 ms, 2000 at first. When nothing matches in
time:

- with a [[ui:exp.portTimeout]] wire, the wait follows it;
- without one, the step fails with `wait.timeout`, which says how many other
  messages arrived meanwhile — a wrong pattern looks different from a silent
  device — and, in its detail, how many older messages were dropped when the
  queue was full.

The wait's variable — `reply`, or `request` for HTTP — exists only after
[[ui:exp.portMatched]]. When the [[ui:dock.inspector]] is capturing, the step
also links the frame it matched: see [the timeline](runs.md#timeline).

## A reply on the same step {#reply}

An [[ui:exp.node.osc]] or a [[ui:exp.node.udp]] can wait for its own answer:
turn on [[ui:exp.expectReply]].

| Setting | What | First given |
| --- | --- | --- |
| [[ui:exp.replyOn]] | the address the answer is awaited on; port 0 is any free port | `0.0.0.0:0` |
| [[ui:exp.replyAddress]] (OSC), [[ui:exp.replyMode]] (UDP) | what the answer must be, as in the matching wait | any |
| [[ui:exp.waitTimeout]] | 1–120 000 ms | 2000 |
| [[ui:exp.replyVariable]] | the variable the answer is written to | `reply` |

The socket on [[ui:exp.replyOn]] is opened before the first step, like a
wait's, and the message **goes out from it**: a device that answers to the
sender's own port is heard, and one that answers to a fixed port is heard when
that port is the one given. The step passes with a matching answer, and the
variable exists after its output. There is no [[ui:exp.portTimeout]] output: no answer in
time fails the step, and Retry can send again. To branch on silence, use a
separate wait.

## What is checked before a run {#validation}

The editor checks the experiment as you edit it; the Run button checks it once
more. A problem names the node, and the field when there is one.

| Rule | Error |
| --- | --- |
| The experiment has a name | `doc.name_required` |
| 1–64 nodes, exactly one Start and one End | `doc.node_count`, `doc.start_end_count` |
| Start has no input | `graph.start_input` |
| A wire leads to another node that exists | `doc.connection_invalid` |
| The same wire is not there twice | `doc.connection_duplicate` |
| Every output that must be wired is | `graph.outputs_required` |
| A node has no wire on an output it does not have | `graph.port_unexpected` |
| Every node can be reached from Start | `graph.unreachable` |
| No cycle but a Loop's wire back | `graph.cycle`, and the [body rules](#loop-body) |
| A check or Extract has an HTTP request before it on every path; a request under load does not count | `graph.needs_http` |
| Every template parses, and every name it uses is known on every path | `template.*`, `name.*` — see [data](data.md#unknown-names) |
| Every field is present and in range | `node.*` |
| A node that names another — [[ui:exp.node.impairment_change]], [[ui:exp.node.emulator_state]], the WebSocket nodes — names one that is there, and a WebSocket node comes after its connect | `impair.relay_unknown`, `emulator.node_unknown`, `ws.connection_unknown`, `ws.connection_after` |
| Two of the run's sockets do not share a port | see [faults](faults.md#ports) |

The run then checks what it needs to start: every [secret](data.md#secret-check)
stored, every port open. Until everything holds, no step runs and nothing is
sent.

## Time limit {#limit}

A run lasts at most 300 s. One still going then is stopped and fails with
`run.timeout`. From the [command line](../automation/cli.md#cli-run) and
[the API](../api/run.md) the limit can be shorter, 1–300 s.

## Stop {#stop}

While a run goes, the Run button is [[ui:common.stop]]. Stop ends the run at
once: every branch, every pause of Retry or Repeat, every wait and every load —
requests in flight are dropped. Its sockets, subscriptions, emulators and
relays close, and its WebSocket connections send a close frame.
[[ui:app.stopAll]] in the header does the same to every job. A stopped run
saves no report; see [runs](runs.md#stop).

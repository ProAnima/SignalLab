---
title: "Design: reactive flows"
---

# Milestone 4 — reactive flows: detailed design

::: info A design note
Written while the feature was being designed and kept as the record of why it is
the way it is. What shipped is described in the user documentation, which is
the one to trust where the two differ.
:::

Status: PR 4.1 delivered, 2026-10-01; PR 4.2 (a reply in the same node, Retry, Wait for MQTT, Wait for this, frames from the timeline) delivered, 2026-10-02; PR 4.3 (Repeat, Loop) delivered, 2026-10-02 — milestone 4 is complete. Parent plan: [ROADMAP.md](./roadmap.md#4-reactive-flows-wait-for-replies-retry-repeat).

PR 4.1 lets an experiment **wait for a device's reply**. Waiting is where most things go wrong — a port is taken, a reply never comes, a pattern is mistyped — so the same PR replaces the engine's English error strings with **structured, localized errors** everywhere the experiment engine reports a problem.

## 1. Errors

### Shape

Every failure the engine reports to the interface is an `EngineError`:

```json
{
  "code": "transport.refused",
  "params": { "target": "127.0.0.1:8080" },
  "node": "login",
  "field": { "key": "url" },
  "detail": "tcp connect error: No connection could be made because the target machine actively refused it. (os error 10061)"
}
```

| Part | Meaning |
| --- | --- |
| `code` | Stable identifier and translation key: the interface shows `err.<code>` with `params` |
| `params` | Values for the message (`{target}`, `{name}`, `{ms}`) — never secret values |
| `node`, `field` | Where: the node id and the field (`field.<key>`, with `index` for "header 2") |
| `detail` | The underlying technical text (operating system, parser, library) — shown verbatim, collapsed, for diagnosis |

The text of a message lives only in the locale files (`en.ts` defines it, `ru.ts` is typed against it), so a new code is a compile-checked translation. The engine never builds sentences.

### Rules

- **One code per cause the user can act on.** "Connection refused", "timed out" and "name not found" are three codes because each has a different fix; the OS wording goes into `detail`.
- **Where, what, why.** The interface renders `Node · Field — message` and offers the detail underneath. Validation, run steps, the run result, *Send now*, the preview and the console all use the same renderer.
- **Masked like everything else.** `params` and `detail` pass through secret masking.
- **Language switching re-renders.** Console lines and step results store the error, not its text.
- **Every code has a translation.** A test scans the engine for `EngineError::new("…")` and `Field::…("…")` and fails if `en.ts` lacks the key, or has an `err.*` key no code uses; `ru.ts` is typed against `en.ts`, and `npm test` checks both languages use the same placeholders.
- **Values that are words are translated as words.** A comparison operator travels as `op` (`not_empty`) and is shown as `exp.op.<op>`; a template error's `position` is shown as part of the location.
- **Legacy text keeps working.** Commands outside the experiment engine still return plain strings; the renderer shows them as they are. Converting a module is local: its commands start returning `EngineError` and nothing else changes.
- **Reports keep the structure.** A run report stores `EngineError` values, so a report can be shown in either language later.

### Transport causes

`transport.rs` classifies failures (from `io::Error` kinds, Windows name-resolution codes and reqwest's error chain) before they become errors. The HTTP screen uses the same causes, so a refused request reads the same there as in a run.

| Code | When |
| --- | --- |
| `transport.refused` | Connection refused (nothing listening) |
| `transport.timeout` | No answer in time |
| `transport.dns` | Host name could not be resolved |
| `transport.unreachable` | Network or host unreachable |
| `transport.reset` | The other side closed or reset the connection |
| `transport.address_in_use` | A listening port is already taken |
| `transport.address_unavailable` | A listening address is not one of this computer's |
| `transport.denied` | The system or a firewall refused |
| `transport.tls` | HTTPS could not be established |
| `transport.target_invalid` | Target is not `IP:port` / `host:port` / not a valid URL |
| `transport.failed` | Anything else (detail has the reason) |

MQTT adds `mqtt.refused` (CONNACK return code), with `mqtt.refused_protocol`, `mqtt.refused_client_id`, `mqtt.refused_unavailable`, `mqtt.refused_credentials` and `mqtt.refused_not_authorized` for return codes 1–5, `mqtt.no_answer`, `mqtt.protocol` and `mqtt.topic_invalid`; the TCP part of a broker failure uses the transport codes. The tool screens (OSC, Broadcast, MQTT, Netsim, Storm, Scanner, HTTP, the library, the Inspector) report through the same codes since 2026-10-02. Note that Windows retries a connection to a closed local port for about two seconds before reporting it refused.

## 2. Responsibilities (SOLID)

`experiment.rs` had grown to hold the document model, validation, the runner and the network actions. It is split along reasons to change:

| Module | Single responsibility |
| --- | --- |
| `error.rs` | The error shape and its builders |
| `transport.rs` | Network failure causes and target resolution |
| `experiment.rs` | The document model: nodes, edges, outputs, versions |
| `experiment_validate.rs` | Structural and run-time validation |
| `experiment_data.rs` | Values: parameters, profiles, templated fields, extraction |
| `experiment_actions.rs` | Performing one network action and classifying its failure |
| `experiment_steps.rs` | What one step does with its branch's state: send, check, extract, branch, wait |
| `experiment_run.rs` | Executing a graph: branches, joins, listeners, events, reports, *Send now* |
| `matching.rs` | Pure matching: comparisons, OSC address patterns, argument and payload rules, reply values |
| `listen.rs` | Listening sockets, their bounded queues and waiting |

Dependencies point inward: the runner decides what runs next and depends on `experiment_steps`; a wait step depends on the `Matcher` trait and on `listen::Listener`; the listener depends on a `FrameSink` trait (the Inspector in the app, a no-op in tests); matching depends only on the error type and the OSC codec. New wait kinds (MQTT in 4.2) add a matcher and a source without touching the runner's loop.

## 3. PR 4.1 — Wait for OSC / UDP

### Nodes

| Node | Fields |
| --- | --- |
| **Wait for OSC** | `bind` (`IP:port` to listen on), `address` pattern, argument rules, `timeout_ms`, `variable` |
| **Wait for UDP** | `bind`, `mode` (`any`, `contains`, `regex`, `hex`), `pattern`, `timeout_ms`, `variable` |

Outputs **Matched** (required) and **Timeout** (optional). Without a Timeout wire, a timeout fails the step with `wait.timeout`.

- **OSC address patterns** follow OSC 1.0: `*` (any characters within a segment), `?`, `[a-z]`, `[!a-z]`, `{ping,pong}`. Without wildcards the address must match exactly.
- **Argument rules**: `argument n` — operator — value, using the comparison operators of *Check value*; the value is a template (`{{nonce}}`), resolved when the wait executes.
- **UDP modes**: `contains` and `regex` work on the payload as UTF-8 text; `hex` looks for a byte sequence (`de ad be ef`); `any` accepts any datagram.
- `pattern`, rule values and nothing else are templated. `bind` is literal: listeners are opened before templates can be resolved.

### Listeners are armed when the run starts

Every distinct `bind` gets one socket when the run starts, before the first step. Waits sharing a bind share the socket. Consequences:

- A reply that arrives **before** execution reaches the wait (the device answered faster than the next step began) is in the queue and matches. A wait considers messages that arrived after the **latest network action on its branch** started (the run's start when there was none; at a Join, the earliest of the merged branches), so a stale message from before the request does not count, and a Delay or Log between the request and the wait does not hide its reply.
- A port that cannot be bound stops the run **before any traffic** — `experiment_start` opens the sockets before the job exists — with `transport.address_in_use`, `transport.address_unavailable`, `transport.denied` or `wait.bind_failed` at the wait node's *Listen on* field. A port still held by the run that just ended is retried briefly instead of being reported as taken.
- A timeout reports how many other messages arrived meanwhile (`wait.timeout {ms, unmatched}`), which tells a wrong pattern from a silent device; a full queue says so in the detail.
- Each listener keeps at most 1024 messages; older ones are dropped and counted.
- Every received datagram is published to the Inspector as an `rx` frame.
- Listener tasks belong to the run: Stop, a failure or the end of the run closes the sockets.
- A matched message is consumed: two waits never match the same message.

### Reply variables

A match writes the variable (default `reply`) as JSON:

| Wait | Value |
| --- | --- |
| OSC | `{ "address": "/pong", "args": [42, "ok"], "from": "127.0.0.1:9000", "ms": 12 }` |
| UDP | `{ "text": "PONG 42", "hex": "50 4f 4e 47 20 34 32", "bytes": 7, "from": "…", "ms": 12 }` |

so `{{reply.args[0]}}`, `{{reply.from}}`, `{{reply.text}}` work in later fields. The variable is set on the **Matched** path only; validation knows that a Timeout path does not have it.

### Validation

`bind` must be `IP:port` with a non-zero port; timeouts 1–120 000 ms; the address pattern must start with `/` and be well-formed; a literal regex must compile; hex must be pairs of hex digits; argument indexes at most 63; the variable follows the variable naming rules.

### Interface

Group **Observe** in the add menu (mint marker). Forms with the fields above; the argument rule list reuses the operators of *Check value*. Summary on the canvas: `/pong* ⇠ :9001 · 2000 ms`. The timeline says which message satisfied the wait and how long it took, or that it timed out. Suggestions offer `reply.address`, `reply.args[0]`, `reply.from`, `reply.ms` (and `reply.text`, `reply.match`, `reply.hex` for UDP) after a wait, on the Matched path only. *Listen now* runs one wait on its own. A bundled template **OSC ping → reply** sends `/ping` with the run id and waits for `/pong` carrying it.

### Also in this PR

Screens are mounted on first visit and kept, hidden, afterwards, so switching tabs keeps typed values, responses, running monitors and each screen's scroll position.

### Moved to PR 4.2

*Expect reply* on action nodes, *Wait for this* from the OSC monitor, and Wait for MQTT.

## 4. PR 4.2, part 1 — a reply in the same node, and Retry

### Wait for a reply

An **OSC message** or a **UDP datagram** may wait for its answer in the same
step (`reply` in the node; document version 4):

| Reply of | Fields |
| --- | --- |
| OSC | `bind`, `address` pattern, argument rules, `timeout_ms`, `variable` — as *Wait for OSC* |
| UDP | `bind`, `mode`, `pattern`, `timeout_ms`, `variable` — as *Wait for UDP* |

- The socket on `bind` is opened when the run starts, like a wait's, and the
  message **goes out from it**: a device that answers the sender's own port is
  heard, and one that answers a fixed port is heard when that port is `bind`.
  Port 0 means any free port, for the first kind.
- The step passes with a matching reply (`exp.step.replied`), which becomes the
  variable on its Next output. No reply in time fails the step (`wait.timeout`
  in the field *Reply on*); there is no Timeout output — a separate Wait node is
  the way to branch on silence, and Retry is the way to send again.
- Validation: as for waits, plus `reply_bind` (port 0 allowed), `reply_address`
  and `reply_pattern` as fields of their own, so a problem points at the right
  input.

### Retry

Every action and wait may retry (`retry` in the node): `attempts` 1–10 in all,
`delay_ms` 0–60 000 before the second attempt, `backoff` `fixed` or
`exponential` (the pause doubles; never longer than a minute).

- A failed attempt is a step event of its own, state `retry`, with the reason
  (`error`) and `exp.step.retrying {attempt, attempts, ms}`; the timeline shows
  both, and the node's badge turns amber. The step then passes, or fails with
  the last attempt's reason.
- Only execution is retried: a template that does not resolve fails at once.
  A wait whose Timeout output is wired does not fail on a timeout, so it is not
  retried — it follows Timeout.
- An attempt of a send that expects a reply sends again; an attempt of a wait
  waits again, counting from the branch's latest action as before.
- Stop aborts the branch, so a pause ends with it; nothing is sent after Stop.
- Other node kinds refuse a retry (`node.retry_unsupported`).

### Interface

*wait for a reply* and *retry on failure* are checkboxes in the node's
properties that open their fields; the canvas shows `⇠ /pong` after a send that
waits and `↻3` for the attempts. Suggestions offer `reply.…` after such a send.

## 5. PR 4.2, part 2 — Wait for MQTT, Wait for this, the matched frame

### Wait for MQTT

| Field | |
| --- | --- |
| `host`, `port` | the broker; parameters only (`{{broker}}`) |
| `topic` | a subscription filter: `+` a whole level, `#` the whole last level; parameters only |
| `mode`, `pattern` | the payload rule of *Wait for UDP* (any, contains, regex, hex); the pattern is a template |
| `timeout_ms`, `variable` | as every wait; outputs Matched and optional Timeout |

- The run connects and subscribes **before its first step** (`subscribe.rs`, one
  connection per broker and filter, QoS 0, clean session), so a reply published
  right after the run's own action is not missed. That is why broker and topic
  take parameters only (`node.params_only`). A refused subscription
  (`mqtt.subscribe_refused` at *Topic*) or an absent broker (`transport.*` at
  *Broker*) stops the run before any traffic.
- **Retained messages** the broker replays on subscribing describe the past and
  are ignored; what counts is published after the run started.
- Messages fill the same `Inbox` as a UDP socket, so matching, consumption, the
  queue bound and the count of unmatched messages behave the same. The reply has
  `topic` besides `text`, `hex`, `bytes`, `match`, `from` and `ms`.
- The connection pings at half its 30 s keep-alive and closes with the run.

### Wait for this

The OSC monitor's rows and the MQTT screen's selected topic offer *Wait for
this*: a Wait for OSC on the monitor's port with the message's address and its
text, whole-number and true/false arguments as rules (floats are measurements
and are left out), or a Wait for MQTT on that topic at that broker. The node is
added before End like *Add to experiment*. The monitor must be stopped before a
run, since the run listens on its port itself.

### The frame a wait matched

Every received datagram and MQTT message the Inspector records keeps its frame
number; a matched wait (or reply) reports it as `frame` in its step. The
timeline lists those frames, and one opens the Inspector with that frame
selected and the filters cleared. Without capture armed there is no frame and
no link.

## 6. PR 4.3 — Repeat and Loop

Both are document version 5 (an older Signal Lab refuses the file instead of
dropping the settings).

### Repeat

Every action may send more than once (`repeat` in the node):

| Field | |
| --- | --- |
| `until` | `count` (a number of sends) or `duration` (as many as fit in a time) |
| `count` | 2–10 000 sends, the first included |
| `duration_ms` | up to the run limit (300 s), from the first send |
| `interval_ms` | 10–60 000 between two sends (100 per second at most) |
| `jitter_ms` | 0–60 000 more per pause, drawn from the run's seed |

- Each send is made like a single one: its templates are read afresh —
  `{{counter}}` is the send's number, `{{now}}` its time — and Retry applies to
  each. A send that fails for good fails the step; otherwise it passes once,
  after the last send, with that send's outcome and variables
  (`exp.step.repeated {n, ms}`). A send that expects a reply waits for its own.
- Progress is a step event, state `repeating`, at most once a second
  (`exp.step.repeatingCount` / `repeatingFor`), so a heartbeat does not flood
  the timeline.
- Bounded twice: by its own numbers, and by the run — the whole repetition must
  fit in 300 s (`node.repeat_too_long`) and a timed one in 10 000 sends
  (`node.repeat_too_many`). Waits and other kinds refuse it
  (`node.repeat_unsupported`). Stop ends a pause; another branch failing stops
  the sends too. *Send now* sends once.
- Jitter is reproducible: the same seed gives the same pauses
  (`Repeat::pause_before`, a stream of its own so templates draw the same
  numbers as without it).

### Loop

A **Loop** node (`max` 1–1000 iterations, optional `until` — the comparison of
*Check value*) has three outputs: **Body**, **Done** and an optional **Limit**.

- Reached from outside, it starts iteration 1 on Body. The steps on Body lead
  back to the Loop; each time they do, the exit condition is read — after the
  iteration, so the body always runs at least once and can set what it tests.
  Done follows when it holds, or after `max` iterations without a condition;
  Limit when the iterations run out before it held (unwired, `loop.limit` fails
  the run). Every arrival is a step event: `exp.step.loopIteration {n, max}`,
  `loopDone {n}`, `loopFinished {n}`, `loopLimit {max}`.
- `{{counter}}` in the body is the iteration's number (every node counts its
  executions). The branch carries which loops it is in and how far each has
  got (`BranchContext::loops`).
- **The one cycle a document may have.** `LoopShape` finds each body — the
  nodes reached from Body that lead back to the Loop — and its wires back.
  Validation and ordering see the graph without those wires, and with the body
  before whatever follows Done, so:
  - another cycle is still `graph.cycle`;
  - a body leads only on in itself or back (`loop.body_leaves`), is entered only
    through Body (`loop.body_entered`), runs as one branch — no second wire on
    an output (`loop.body_parallel`), no Start, End, Fork, Join or nested Loop
    (`loop.body_unsupported`) — and comes back (`loop.no_return`);
  - the exit condition, and the steps after Done or Limit, may use what every
    iteration of the body sets; the body itself only what is known on entry.
- The template *Poll until ready* asks a device for `/status` every 0.3 s until
  it answers `ready`, at most 10 times.

### Interface

*repeat sending* is a checkbox in an action's properties (times or a time,
every, jitter); the node shows `×10` or `5s`. The Loop's properties hold the
maximum and *stop early when* with the value, operator and expected value of
*Check value*, and the preview says what will be compared. On the canvas the
wire back runs dashed over the body (under it when there is no room above);
the editor allows exactly that wire (`connect`), and layout, insertion and
removal read the flow forward (`isBackEdge`, `loopBody`).

## 7. Tests

- Error: builders, masking of params and detail, serialization; a scan that every code and field key used by the engine has an English text and no text is stale.
- Matching: OSC patterns (wildcards, sets, alternatives, malformed patterns with positions), argument rules with numbers and text, UDP modes, hex parsing.
- Listener (loopback sockets): a reply sent before the wait is matched; one sent before `since` is not but is counted; a consumed message does not match twice; a waiting step wakes on arrival; the queue bound drops and counts; dropping the listener closes the port; binding a taken port reports `transport.address_in_use`.
- Wait step (loopback): the reply with the right argument matches and is written, Timeout is followed when wired and an error otherwise, a reply before the branch's latest request does not count.
- Validation: wait fields, the variable set only on Matched, optional Timeout, unexpected outputs.
- Transport classification of refused, timeout, name and invalid-URL errors (HTTP and TCP); MQTT causes to codes.
- UI (`npm test`): the renderer's location, operator wording, unknown codes, legacy text; waits' ports and reply variables; matching placeholders in both languages.
- Repeat (loopback, `engine/tests/repeat_loop.rs`): a count of sends each with its own `{{counter}}`, a timed repeat that never sends past its time and reports progress, Stop in a pause, a reply per send, and what validation refuses; the jitter's range and reproducibility.
- Loop: polling a device until it is ready (iterations, Done reading the last answer), Limit and its failure when unwired, a loop without a condition, Stop between iterations, and every body rule — plus `graph.cycle` elsewhere and names that Done may and may not read. Editor: only a body's last step wires back, layout and insertion read the flow forward, removing the only body step never wires the loop to itself.
- End to end (`npm run e2e`, CI): every screen in the desktop app and in the server, on Windows and Linux, including a repeated ping and the poll template (see [delivery.md](./delivery.md#e2e)).

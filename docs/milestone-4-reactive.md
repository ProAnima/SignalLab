# Milestone 4 — reactive flows: detailed design

Status: PR 4.1 delivered, 2026-10-01. Parent plan: [ROADMAP.md](../ROADMAP.md#4-reactive-flows-wait-for-replies-retry-repeat).

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
| `transport.target_invalid` | Target is not `IP:port` / not a valid URL |
| `transport.failed` | Anything else (detail has the reason) |

MQTT adds `mqtt.refused` (CONNACK return code), `mqtt.no_answer`, `mqtt.protocol` and `mqtt.topic_invalid`; the TCP part of a broker failure uses the transport codes. Note that Windows retries a connection to a closed local port for about two seconds before reporting it refused.

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

## 4. Tests

- Error: builders, masking of params and detail, serialization; a scan that every code and field key used by the engine has an English text and no text is stale.
- Matching: OSC patterns (wildcards, sets, alternatives, malformed patterns with positions), argument rules with numbers and text, UDP modes, hex parsing.
- Listener (loopback sockets): a reply sent before the wait is matched; one sent before `since` is not but is counted; a consumed message does not match twice; a waiting step wakes on arrival; the queue bound drops and counts; dropping the listener closes the port; binding a taken port reports `transport.address_in_use`.
- Wait step (loopback): the reply with the right argument matches and is written, Timeout is followed when wired and an error otherwise, a reply before the branch's latest request does not count.
- Validation: wait fields, the variable set only on Matched, optional Timeout, unexpected outputs.
- Transport classification of refused, timeout, name and invalid-URL errors (HTTP and TCP); MQTT causes to codes.
- UI (`npm test`): the renderer's location, operator wording, unknown codes, legacy text; waits' ports and reply variables; matching placeholders in both languages.

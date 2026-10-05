---
title: Emulators
description: Make Signal Lab the other side — an HTTP API, an OSC, UDP or TCP device, or an MQTT broker — that answers by your rules, fails on cue and counts what arrives.
---

# Emulators

An emulator is Signal Lab playing the API, device or service your system talks
to. It listens on one address and answers by rules: an HTTP API by routes, an
OSC, UDP or TCP device by "on this, reply that", an MQTT broker as any broker
does plus rules of its own. It can be slow, fail, or go down now and then, so
you can test what your system does when its dependency misbehaves. Every
exchange is counted, listed and sent to the [Inspector](inspector.md).

An emulator is one document. The [[ui:nav.emulators]] screen keeps a library of
them; the same document runs inside an experiment as an
[[ui:exp.node.emulator]] node, from the command line with `signallab emulate`,
and through the [API](../api/commands.md) and [MCP](../automation/mcp.md), and
answers the same way everywhere.

## The screen {#screen}

On the left is the library ([[ui:emu.library]]): every emulator with its
protocol and address, a pulsing dot and a request count on the ones running.
On the right are the selected emulator's settings and rules, and under them
what it has received ([[ui:emu.live]]).

## Making an emulator {#create}

1. Press one of the buttons at the top of the library:

   | Button | Makes | Listens on | With one rule that works as it stands |
   | --- | --- | --- | --- |
   | ＋ [[ui:emu.new.http]] | An HTTP API | `127.0.0.1:18080` | `GET /health` → 200 `{"status":"ok"}` |
   | ＋ [[ui:emu.new.osc]] | An OSC device | `127.0.0.1:9100` | `/ping` → `/pong` with the count as an int |
   | ＋ [[ui:emu.new.udp]] | A UDP device | `127.0.0.1:7100` | a datagram containing `PING` → `PONG 1`, `PONG 2`, … |
   | ＋ [[ui:emu.new.tcp]] | A TCP device | `127.0.0.1:7200` | a line containing `PING` → `PONG` |
   | ＋ [[ui:emu.new.mqtt]] | An MQTT broker | `127.0.0.1:1883` | a publish to `lab/<name>/set` → the same payload, retained, on `lab/<name>/state` |

   When another emulator of the library already uses that port, the next free
   one is taken.
2. Give it a [[ui:emu.name]] (at most 120 characters).
3. Set [[ui:emu.bind]]: `IP:port`. `127.0.0.1` answers this computer only;
   `0.0.0.0` answers the network as well.
4. Change the rules (below), and say in [[ui:emu.note]] what it stands in for.

Changes are saved on their own. [[ui:emu.duplicate]] makes a copy on the next
free port. [[ui:emu.delete]] asks once more ([[ui:emu.confirmDelete]]), stops
the emulator if it runs, and removes it from the library.

Rules are tried in order, first to last; the first that matches answers. Each
rule's header shows a one-line summary; click it to open or fold the rule. The
↑ and ↓ buttons move a rule, × removes it.

## Running it {#run}

1. Select the emulator and press [[ui:emu.start]]. Its port opens before the
   button comes back: a port already taken, or an emulator with a problem, is
   refused there with the reason.
2. Point your system at it. For an HTTP API, [[ui:emu.copyUrl]] copies its
   address (`http://127.0.0.1:18080`), and each route has a
   [[ui:emu.copyRouteUrl]] button for its own (not when its path holds a
   `{{…}}` template).
3. Watch [[ui:emu.received]] fill.
4. Press [[ui:emu.stop]], or stop its job from the console strip.

The state beside the buttons says [[ui:emu.notRunning]], where it answers, or
that it is down.

An emulator keeps answering with the rules it was started with. When you
change it while it runs, [[ui:emu.restart]] appears: press it to start again
with the rules as they are now. Until then the hit counts on the rules are
hidden, as they belong to the old rules.

[[ui:emu.takeDown]] makes a running emulator unavailable until you press
[[ui:emu.bringUp]]: an HTTP request gets 503, a TCP device and an MQTT broker
drop their connections and refuse new ones, an OSC or UDP device answers
nothing. See [Going down](#outage).

Two emulators of one transport cannot share a port: HTTP, TCP and MQTT
emulators listen on TCP ports, OSC and UDP emulators on UDP ports. An HTTP API
and an OSC device can both use port 8080; two HTTP APIs cannot. A second one on
a taken port is refused when it starts.

::: tip
In a browser connected to a [server](../server/index.md), the emulator runs on
the server. One listening on `0.0.0.0` is reached by the server's name, and
[[ui:emu.copyUrl]] copies that address; one on `127.0.0.1` answers only
programs on the server itself.
:::

## What arrived {#received}

While it runs, [[ui:emu.live]] counts:

| Count | What |
| --- | --- |
| [[ui:emu.total]] | Everything that arrived: requests, messages, lines. |
| [[ui:emu.unmatched]] | What no rule took. An HTTP request without a route still gets its answer (see [Requests no route takes](#fallback)); the others get none. |
| [[ui:emu.failed]] | Exchanges where a reply could not be made or sent. |
| [[ui:emu.down]] | What arrived while the emulator was down. Shown when it has an outage set or something met it down. Never counted as [[ui:emu.unmatched]]. |
| [[ui:emu.missed]] | MQTT only, when it happens: messages a client was too far behind to take. |

Each rule's header shows how many times it matched since the start.

[[ui:emu.received]] lists the newest 300 exchanges, newest first:

| Column | What |
| --- | --- |
| [[ui:emu.col.time]] | When it arrived. |
| [[ui:emu.col.from]] | The client's address. |
| [[ui:emu.col.request]] | What arrived, in protocol notation: `GET /users/7`, `/ping 1`, `POWER?`. |
| [[ui:emu.col.rule]] | The rule that took it (`#2`), or `—`. |
| [[ui:emu.col.reply]] | What went back: `200 OK · 37 B`, `/pong 3`, a payload; [[ui:emu.held]] or [[ui:emu.closed]] for a fault; the error when the reply failed; [[ui:emu.wasDown]] when it arrived while down. |
| [[ui:emu.col.ms]] | From arrival until the reply left, its delay included. |

The ⌕ button on a row ([[ui:emu.inspectFrame]]) opens that exchange in the
Inspector, when capture was on. When more than 200 exchanges arrive within a
fifth of a second, the list skips some and says how many. The engine keeps the
newest 500 exchanges of each running emulator, with what arrived, for the
command line, the API and MCP.

## HTTP API {#http}

An HTTP/1.1 server. Each request is answered by the first route that takes it.

### Routes {#routes}

A route takes a request when its method, its path and all its conditions match.

| Field | What |
| --- | --- |
| [[ui:emu.method]] | `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS`, or [[ui:emu.methodAny]]. A `GET` route answers `HEAD` too. |
| [[ui:emu.path]] | Starts with `/`. A segment `:name` takes any one segment, read as `{{request.params.name}}`; a last segment `*` takes everything below. A trailing `/` makes no difference; the query string is not part of the path. |
| [[ui:emu.conditions]] | Every one must hold. Add one with ＋ [[ui:emu.addCondition]]. |

Path examples:

| Path | Takes | Does not take |
| --- | --- | --- |
| `/health` | `/health`, `/health/` | `/health/db`, `/Health` |
| `/users/:id` | `/users/7` (`params.id` is `7`), `/users/a%20b` (`a b`) | `/users`, `/users/7/orders` |
| `/files/*` | `/files`, `/files/a`, `/files/a/b/c` | `/file`, `/other/files/a` |

A condition reads one part of the request ([[ui:emu.on]]) and compares it:

| [[ui:emu.on]] | Name | Reads |
| --- | --- | --- |
| [[ui:emu.on.header]] | A header name, any case | The header's value; a header sent several times, its values joined with `, `. |
| [[ui:emu.on.query]] | A query parameter | Its value, decoded; the first, when it is repeated. |
| [[ui:emu.on.body]] | — | The whole body as text. |
| [[ui:emu.on.json]] | A JSON path, such as `$.user.id` | That field of a JSON body. |

The comparisons are [[ui:exp.op.eq]], [[ui:exp.op.ne]], [[ui:exp.op.lt]],
[[ui:exp.op.le]], [[ui:exp.op.gt]], [[ui:exp.op.ge]], [[ui:exp.op.contains]],
[[ui:exp.op.matches]], [[ui:exp.op.empty]] and [[ui:exp.op.not_empty]]. Numbers
compare as numbers, text exactly. A header, parameter or field that is not
there is empty. A comparison that cannot be made — text against a number —
does not hold.

### Responses {#responses}

A route has one to 16 responses ([[ui:emu.responses]]).

| Field | What | Default |
| --- | --- | --- |
| [[ui:emu.status]] | 100–599. | 200 |
| [[ui:emu.fault]] | Something other than an answer; see [Faults](#faults). | [[ui:emu.fault.none]] |
| [[ui:emu.delay]] | How long to wait before answering, 0–60 000 ms. | 0 |
| [[ui:emu.jitter]] | Up to this much longer, at random, 0–60 000 ms. | 0 |
| [[ui:emu.weight]] | Its share when the route answers at random. Shown only then. | 1 |
| [[ui:emu.headers]] | Up to 32. Names may use parameters; values are [templates](#templates). | none |
| [[ui:emu.body]] | A [template](#templates), up to 256 KiB as written. | empty |

Without a `Content-Type` header, a body that is valid JSON goes as
`application/json` and any other body as `text/plain; charset=utf-8`.

With two responses or more, [[ui:emu.order]] says which one a request gets:

| [[ui:emu.order]] | Requests get | For |
| --- | --- | --- |
| [[ui:emu.order.sequence]] | The first, the second, …, then the last one from then on: 500, 500, 200, 200, 200… | Retries: fail twice, then work. |
| [[ui:emu.order.cycle]] | The first again after the last: 200, 500, 200, 500… | A dependency that fails now and then, regularly. |
| [[ui:emu.order.random]] | Each drawn by its weight. Weights 8 and 2 give the first about 80 % of the time. At least one weight must be above 0. | A realistic share of failures. |

[[ui:emu.preset]] adds a ready response to the route:

| Preset | Adds |
| --- | --- |
| [[ui:emu.preset.ok]] | 200, `{"ok":true}` |
| [[ui:emu.preset.created]] | 201, `{"id":"{{uuid}}"}`, header `Location: {{request.path}}/{{counter}}` |
| [[ui:emu.preset.notFound]] | 404, `{"error":"not found"}` |
| [[ui:emu.preset.error]] | 500, `{"error":"internal"}` |
| [[ui:emu.preset.unavailable]] | 503, `{"error":"unavailable"}`, header `Retry-After: 1` |
| [[ui:emu.preset.slow]] | 200, `{"ok":true}` after 2000 ms |
| [[ui:emu.preset.timeout]] | The fault [[ui:emu.fault.timeout]] |
| [[ui:emu.preset.reset]] | The fault [[ui:emu.fault.reset]] |
| [[ui:emu.preset.malformed]] | 200, `{"items":[{"id":1},{"id":2}]}` with the fault [[ui:emu.fault.malformed]] |

### Faults {#faults}

| [[ui:emu.fault]] | What the client meets |
| --- | --- |
| [[ui:emu.fault.none]] | The response. |
| [[ui:emu.fault.timeout]] | Nothing. The request is held for up to 2 minutes, then the connection is closed — so the client's own timeout is what is tested. The delay does not apply. |
| [[ui:emu.fault.reset]] | The connection closes without an answer, after the delay. |
| [[ui:emu.fault.malformed]] | A complete HTTP answer with the status and headers set, whose body stops halfway: JSON that does not parse. When the whole body was JSON, the content type still says `application/json`. |

### Requests no route takes {#fallback}

[[ui:emu.fallback]] decides what a request that matches no route gets:

- [[ui:emu.fallbackDefault]] — 404 with the body `{"error":"no_route"}`;
- [[ui:emu.fallbackCustom]] — a response you set, with everything a route's
  response has. Its `{{counter}}` counts the requests no route took.

Either way the request counts as [[ui:emu.unmatched]].

### What an HTTP reply can read {#http-request}

| Template | Is |
| --- | --- |
| `{{request.method}}` | `GET`, `POST`, … |
| `{{request.path}}` | The path, without the query. |
| `{{request.params.id}}` | The path segment named `:id`. |
| `{{request.query.page}}` | A query parameter, decoded. |
| `{{request.headers.x-key}}` | A header; names in lower case. |
| `{{request.body}}` | The body as text: its first 64 KiB. |
| `{{request.json.name}}` | A field of a JSON body, when the body is JSON and within 64 KiB. |
| `{{request.from}}` | The client's `IP:port`. |

A request body larger than 1 MiB gets 413 and is counted as [[ui:emu.failed]].
A reply that cannot be made — a template naming something the request does not
have — gets 500 with the error in its body, and is counted as
[[ui:emu.failed]].

## OSC device {#osc}

Each message that arrives — each message of a bundle on its own — is answered
by the first rule it matches. A datagram that is not OSC is counted as
[[ui:emu.unmatched]].

| Field | What |
| --- | --- |
| [[ui:emu.address]] | An OSC 1.0 address pattern: `*` any characters, `?` one, `[a-z]` a set, `{a,b}` either, each within one segment (see [OSC](../protocols/osc.md#patterns)). |
| [[ui:exp.argRules]] | Up to 16 conditions on the arguments, as in [[ui:exp.node.wait_osc]] (see [Nodes](../experiments/nodes.md#node-wait_osc)). |
| [[ui:emu.replyOn]] | Off: take the message and answer nothing. |
| [[ui:emu.replyAddress]] | The reply's address, a [template](#templates). |
| [[ui:emu.replyArgs]] | Up to 16 arguments, each a [[ui:emu.argType]] (`int`, `float`, `str`, `long`, `double`, `bool`, `blob`, `nil`) and a [[ui:emu.argValue]] template. |
| [[ui:emu.to]] | Empty: back to the sender's address and port. Otherwise `IP:port`. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 0–60 000 ms each. |

An argument's value is read as its type after the template is filled:
`{{request.args[0]}}` echoes the first argument as a number when the type is a
number. A `bool` takes `true`, `1`, `yes`, `on` or `false`, `0`, `no`, `off`;
a `blob` takes hex bytes; an empty value is the type's zero.

Replies leave from the emulator's own port, so a client that listens on the
port it sent from hears them.

An OSC reply can read `{{request.address}}`, `{{request.args[0]}}` and
`{{request.from}}`.

## UDP device {#udp}

Each datagram is answered by the first rule it matches.

| Field | What |
| --- | --- |
| [[ui:emu.match]] | [[ui:exp.mode.any]], [[ui:exp.mode.contains]], [[ui:exp.mode.regex]] or [[ui:exp.mode.hex]]. |
| [[ui:emu.pattern]] | The text, regular expression or bytes to look for. |
| [[ui:emu.reply]] | [[ui:emu.replyOff]], [[ui:emu.replyText]] or [[ui:emu.replyHex]], then the reply itself as a [template](#templates). |
| [[ui:emu.to]] | Empty: back to the sender. Otherwise `IP:port`. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 0–60 000 ms each. |

A UDP or TCP reply can read:

| Template | Is |
| --- | --- |
| `{{request.text}}` | The payload as text. |
| `{{request.match}}` | What matched: the text, a regular expression's first group (or the whole match), the bytes. |
| `{{request.hex}}` | The payload as hex bytes, its first 1024. |
| `{{request.bytes}}` | The payload's size. |
| `{{request.from}}` | The sender's `IP:port`. |

A text reply is at most 65 507 bytes.

## TCP device {#tcp}

A device that speaks lines on a TCP connection, as a projector or a matrix
switcher does. Each message a client sends is answered by the first rule it
matches; the reply goes back on the same connection.

| Field | What |
| --- | --- |
| [[ui:emu.delimiter]] | What ends a message, and is added after each reply and the greeting: [[ui:emu.delimiter.lf]] (a `\r` before it is dropped), [[ui:emu.delimiter.crlf]], [[ui:emu.delimiter.cr]], or [[ui:emu.delimiter.none]]. Empty lines are skipped. |
| [[ui:emu.greeting]] | Sent as a client connects; empty for none. It can read `{{request.from}}`. |
| [[ui:emu.match]], [[ui:emu.pattern]], [[ui:emu.reply]] | As for a [UDP device](#udp). |
| [[ui:emu.close]] | Close the connection after this rule's reply — `QUIT`, say. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 0–60 000 ms each. |

A message longer than 64 KiB without its delimiter is taken as it stands.

## MQTT broker {#mqtt}

A small MQTT 3.1.1 broker on plain TCP. It does what a broker does: clients
connect, subscribe with `+` and `#`, publish at QoS 0, 1 and 2, retained
messages and last wills work, and a second connection with a client's id takes
over from the first. Sessions are always clean: a client asking to keep its
session gets a fresh one, and nothing is queued for a client that is away.

On top of that, every message published to it is checked against the rules:
the first that matches also publishes a reply — a device reporting what it did.

| Field | What |
| --- | --- |
| [[ui:emu.username]], [[ui:emu.password]] | When a user name is set, a client must connect with it and the password; empty: anyone may connect. A password without a user name is refused, as MQTT 3.1.1 cannot carry one. |
| [[ui:emu.retained]] | Up to 64 messages ([[ui:emu.topic]], [[ui:emu.payload]], [[ui:emu.qos]]) held from the start, as if published with retain: a client that subscribes gets them first. |
| [[ui:emu.topicFilter]] | Which topics a rule takes: `+` one level, `#` the rest — `lab/+/set`. |
| [[ui:emu.match]], [[ui:emu.pattern]] | A condition on the payload, as for a [UDP device](#udp). |
| [[ui:emu.replyOn]] | Off: take the message and publish nothing more. |
| [[ui:emu.replyTopic]], [[ui:emu.replyPayload]] | [Templates](#templates). The topic cannot hold `+` or `#`. |
| [[ui:emu.qos]], [[ui:emu.retain]] | Of the reply. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 0–60 000 ms each. |

An MQTT reply can read `{{request.topic}}`, `{{request.levels[1]}}` (the
topic's levels, from 0), `{{request.payload}}`, `{{request.json.state}}`,
`{{request.match}}`, `{{request.qos}}`, `{{request.retain}}`,
`{{request.client}}` (the client id) and `{{request.from}}`.

## Templates in replies {#templates}

Replies are written in the same [template language](../experiments/data.md#templates)
as experiments, so a field means the same thing here and there. A reply can
read:

- `request` — what arrived, as listed for each protocol above;
- `{{counter}}` — how many messages this rule has taken since the emulator
  started, this one included;
- the [generators](../experiments/data.md#generators) — `{{uuid}}`,
  `{{now.iso}}`, random values and the rest; random ones are drawn from the
  emulator's seed;
- parameters, when the emulator runs in an experiment or is started with
  `signallab emulate --param`.

A reply never reads secrets, and an unknown name is an error, not an empty
text.

Some fields are fixed when the emulator starts, before anything arrives: a
path, a condition, an address pattern, a payload pattern, a topic filter,
[[ui:emu.to]], a header's name, retained messages and the broker's login. They
take text and parameters only, no `request` and no generators.

The seed drives the random order of responses, the jitter and the random
generators. On the [[ui:nav.emulators]] screen each start takes a new seed; an
experiment uses the run's seed, and `signallab emulate --seed` takes one you
give.

## Going down {#outage}

To test what your system does when a dependency flaps, tick
[[ui:emu.outage]]:

| Field | What | Default |
| --- | --- | --- |
| [[ui:emu.outageUp]] | How long it answers, 10–3 600 000 ms. | 10 000 |
| [[ui:emu.outageDown]] | How long it is down, 10–3 600 000 ms. | 3000 |
| [[ui:emu.outageFault]] | HTTP only: what a request meets while it is down. | [[ui:emu.outageFault.unavailable]] |

The schedule starts when the emulator starts and repeats: up, down, up,
down… While it is down:

| Emulator | Meets |
| --- | --- |
| HTTP | [[ui:emu.outageFault.unavailable]]: 503 with `Retry-After` set to the seconds until it is back (at least 1). [[ui:emu.outageFault.reset]]: the connection closes without an answer. [[ui:emu.outageFault.timeout]]: held for up to 2 minutes, then closed. |
| TCP device | Open connections are dropped within 0.1 s; new ones are closed as they arrive. |
| MQTT broker | Every connection is dropped; new ones are refused (CONNACK return code 3, server unavailable). |
| OSC, UDP device | Nothing is answered. |

What arrives while it is down counts as [[ui:emu.down]], not as
[[ui:emu.unmatched]], and its rules are not asked.

[[ui:emu.takeDown]] does the same on demand, whatever the schedule says, until
you press [[ui:emu.bringUp]]; HTTP then meets 503 without `Retry-After`. In an
experiment, the [[ui:exp.node.emulator_state]] node does it at a step of the
run (see [Nodes](../experiments/nodes.md#node-emulator_state) and
[Faults](../experiments/faults.md)).

## Problems {#problems}

While you edit, the emulator is checked a moment after each change, and a
problem shows under its buttons before you press [[ui:emu.start]]. A problem
names where it is — the rule, the response or the retained message, and the
field — and what is wrong: a path without its `/`, a regular expression that
does not compile, a reply template naming something other than `request`,
parameters and generators, a value out of range. [[ui:emu.start]] refuses an emulator with a problem.

## Limits {#limits}

| What | Limit | At the limit |
| --- | --- | --- |
| Routes or rules per emulator | 64 | Refused when checked. |
| Responses per route | 16 | Refused. |
| Conditions per route | 16 | Refused. |
| Headers per response | 32 | Refused. |
| Argument conditions, reply arguments (OSC) | 16 each | Refused. |
| Retained messages (MQTT) | 64 | Refused. |
| A body, reply or greeting as written | 256 KiB | Refused. |
| A delay or a jitter | 60 000 ms | Refused. |
| HTTP request body | 1 MiB | 413. |
| HTTP connections at once | 512 | More are closed as they arrive. |
| HTTP request head | 30 s | A client must send it within this. |
| TCP connections at once | 256 | More are closed as they arrive. |
| OSC and UDP replies waiting for their delay | 1024 | More are dropped and counted as [[ui:emu.failed]]. |
| MQTT clients at once | 256 | More are closed as they arrive. |
| MQTT packet | 256 KiB | The client's connection ends. |
| MQTT subscriptions per client | 100 | More are refused. |
| MQTT retained topics | 1000 topics, 16 MiB | A new retained message is routed, not retained. |
| MQTT messages waiting for one slow client | 1024 messages, 8 MiB | It misses them; counted as [[ui:emu.missed]]. |

## Mock this {#mock-this}

To make an emulator from a response that worked:

1. On the [[ui:nav.http]] screen, send a request and get a response — or use
   [[ui:exp.sendNow]] on an HTTP node of an experiment.
2. Press ⧉ [[ui:http.mockThis]] beside the response. The
   [[ui:http.mockTitle]] dialog shows the route it will make.
3. In [[ui:http.mockInto]], pick one of your HTTP emulators, or
   [[ui:http.mockNew]].
4. Press [[ui:http.mockAdd]]. The [[ui:nav.emulators]] screen opens on that
   emulator.

The route answers the request's method and path (without the query) with the
response's status, headers and body. Headers that belong to the one exchange
(`Content-Length`, `Date`, `Server`, `ETag` and the like) are left out, and the
body is sent as it was, even if it holds `{{`. A new emulator holds only this
route. Added to an existing emulator, the route goes first, so it answers
before a broader route; a running one picks it up when you press
[[ui:emu.restart]].

From an experiment, a URL written with templates becomes a pattern: its base
(`{{api}}`) is dropped, a segment that is one template (`/orders/{{order_id}}`)
becomes `:order_id`, and a segment only partly templated ends the path with
`*`.

## The starter set {#starter-set}

The first time Signal Lab finds no emulator library, it writes five, all on
this computer. Their names and notes are written in the language of the
interface at that moment.

| Emulator | Listens on | Does |
| --- | --- | --- |
| [[ui:seed.emu.demo-api.name]] | `127.0.0.1:8080` | `GET /health` → `{"status":"ok","time":…}`; `GET /users/:id` → a user with that id; `POST /users` → 201 with a `Location`; `GET /slow` → after 1500 ms; `/flaky` → 503, 503, then 200 from then on. |
| [[ui:seed.emu.osc-device.name]] | `127.0.0.1:9100` | `/ping` → `/pong` with the count; `/fader/*` → `/ack` with the address it got; `/cue/*` taken without a reply. |
| [[ui:seed.emu.udp-device.name]] | `127.0.0.1:7100` | `PING` → `PONG` and the count; anything else → `ACK` and its size in bytes. |
| [[ui:seed.emu.tcp-device.name]] | `127.0.0.1:7200` | Lines ending in CR LF. Greets with `READY`; `POWER?` → `POWER=ON`; `POWER ON` or `POWER OFF` → `OK ON` / `OK OFF`; `QUIT` → `BYE`, then hangs up. |
| [[ui:seed.emu.mqtt-broker.name]] | `127.0.0.1:1883` | Retains `online` on `lab/status`; `ON` or `OFF` published to `lab/<name>/set` → the same, retained, on `lab/<name>/state`. |

The starter [[ui:seed.http-reachable.name]] signal of the
[signal library](signals.md#starter-set) asks `http://127.0.0.1:8080/`, the
[[ui:seed.emu.demo-api.name]]'s address: it has no route for `/`, so it gets
404.

## The library file {#file}

The library is `emulators.json` in the data folder (see
[Files](../reference/files.md)); hover the count under the list to see its
path. It is written whole 0.7 s after the last change, through a temporary
file, so a failed write leaves the previous one. If the file cannot be read,
the list shows the error with the path, line and column, and the file is left
as it is: fix it and press [[ui:emu.reload]]. Press [[ui:emu.reload]] too after
editing it by hand. With no file, the starter set is written again.

```json
{
  "version": 1,
  "emulators": [
    {
      "id": "orders-api",
      "note": "Stands in for the orders service.",
      "emulator": {
        "name": "Orders API",
        "bind": "127.0.0.1:18080",
        "protocol": "http",
        "routes": [
          { "method": "GET", "path": "/orders/:id",
            "responses": [{ "body": "{\"id\":\"{{request.params.id}}\",\"state\":\"open\"}" }] },
          { "method": "POST", "path": "/orders", "order": "sequence",
            "responses": [{ "status": 503 }, { "status": 201, "body": "{\"id\":\"{{uuid}}\"}" }] }
        ],
        "outage": { "up_ms": 20000, "down_ms": 2000, "fault": "unavailable" }
      }
    }
  ]
}
```

The `emulator` object alone is a document `signallab emulate` reads too.

## In experiments and scripts {#elsewhere}

- In an experiment, an [[ui:exp.node.emulator]] node opens its emulator before
  the first step and answers until the run ends; what it received is counted
  in the report. An HTTP emulator there is also what
  [[ui:exp.node.wait_http]] ([Nodes](../experiments/nodes.md#node-wait_http))
  listens to, and an OSC or UDP emulator shares its
  port with the run's waits. Two emulators of one transport in one experiment
  cannot share a port. See [Nodes](../experiments/nodes.md#node-emulator) and
  [Faults](../experiments/faults.md).
- `signallab emulate` runs emulators from files or from this library until
  <kbd>Ctrl</kbd>+<kbd>C</kbd> or `--for`, printing what they answer; see
  [The command line](../automation/cli.md#cli-emulate).

## Related {#related}

- [Inspector](inspector.md) — every exchange, decoded.
- [Impairment](impairment.md) — a bad network between your system and an
  emulator.
- [Data and templates](../experiments/data.md#templates)

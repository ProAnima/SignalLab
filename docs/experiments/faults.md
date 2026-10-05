---
title: Faults
description: Impairment relays and emulators as nodes of a run — a bad network and a failing dependency switched on cue, counted phase by phase in the report and repeatable with the seed.
---

# Faults as nodes

To see how a system copes when the network degrades or a dependency goes
down, put the fault in the experiment. A relay or an emulator opens with the
run, a step switches it on cue, and the run report counts what happened in each
phase. The run's end — passed, failed or stopped — closes them, so nothing
stays impaired behind it.

| Node | What it does |
| --- | --- |
| [[ui:exp.node.impairment]] | a relay between the system under test and its target, impairing what goes through, for the whole run |
| [[ui:exp.node.impairment_change]] | switches a relay of the run to another profile, from this step on |
| [[ui:exp.node.emulator]] | an API, a device or a broker played by Signal Lab, for the whole run |
| [[ui:exp.node.emulator_state]] | takes an emulator of the run down, or brings it back |

All four are in the add menu under [[ui:exp.group.fault]] and
[[ui:exp.group.emulate]]. Their fields are in
[the nodes' reference](nodes.md); the relay itself is described on
[Impairment](../tools/impairment.md), the emulators on
[Emulators](../tools/emulators.md).

## Impairment {#impairment}

The system under test sends to the relay instead of its real target; the relay
forwards to the target, carries the answers back, and impairs both directions.

| Field | What |
| --- | --- |
| [[ui:exp.relayListen]] | `IP:port` the system under test sends or connects to, port not 0 |
| [[ui:exp.relayTarget]] | `IP:port` of the real destination, or `host:port`; a host name is looked up when the run starts |
| [[ui:ns.protocol]] | UDP — each datagram meets its own fate — or TCP — each connection is joined to one of its own to the target |
| the profile | a [[ui:ns.preset]] or values of your own |

What a relay reads of its profile depends on the protocol; the other values
are left out:

| Protocol | Impairments |
| --- | --- |
| UDP | latency, jitter, packet loss, burst loss, duplication, corruption, reordering, a bandwidth limit, offline |
| TCP | latency and jitter (a stream stays in order), a bandwidth limit (the sender is slowed down, nothing dropped), connections reset, connections left half-open, offline |

**Opened before the first step.** Every relay of the experiment opens when the
run starts, like the sockets of waits, so its [[ui:exp.relayListen]] and
[[ui:exp.relayTarget]] take text and parameters only
(`node.params_only`) — `{{relay}}` with a parameter `relay`, never a variable.
A port that cannot be opened, or a target name that cannot be found, stops the
run before any traffic, at the node.

**Passing in the flow.** When the run reaches the node, it passes at once and
the timeline says what it impairs, and with what. The relay works from the
start of the run to its end, wherever the node stands in the graph.

**Closed with the run.** However the run ends, the relay closes; a TCP relay's
connections close with it. A relay that stopped relaying on its own keeps the
reason: the steps that use it fail with it, and the report says so.

::: tip Route through impairment
On an [[ui:exp.node.osc]] or [[ui:exp.node.udp]] node,
[[ui:exp.routeThrough]] puts an Impairment in front of it: the relay listens
on a free port of `127.0.0.1`, forwards to the node's target with the
[[ui:ns.preset.lan]] preset, and the node now sends to the relay.
:::

## Change impairment {#change-impairment}

[[ui:exp.node.impairment_change]] names one of the experiment's relays in
[[ui:exp.relay]] and gives the profile it impairs with from that step on. The
relay keeps its port and its connections; the new values apply to the next
packet or chunk. The timeline shows the new profile.

Each change ends a **phase**. The run report keeps, for every relay:

- its listen and target addresses, and its protocol when it is TCP;
- its counts in all: received, forwarded, dropped, throttled, duplicated,
  corrupted, reordered, bytes — and for TCP the connections, those reset and
  those left half-open;
- each phase: the profile's name, when it began and ended in milliseconds from
  the moment the relay opened, and the same counts for that phase alone.

A packet is counted in the phase that decided its fate, even when its delayed
copy goes out after the switch. A relay keeps its last 1000 phases; older ones
are counted, not kept.

An [[ui:exp.node.impairment_change]] that names no relay of the experiment is
refused (`impair.relay_unknown`).

## Emulator {#emulator}

[[ui:exp.node.emulator]] plays a dependency for the whole run: an HTTP API, an
OSC, UDP or TCP device, or an MQTT broker. It is the same emulator the
[Emulators](../tools/emulators.md) screen runs on its own:
[[ui:emu.edit]] opens its rules, [[ui:emu.toLibrary]] keeps a copy in the
library, [[ui:emu.fromLibrary]] takes one from it.

- It opens before the first step and answers until the run ends; a port that
  cannot be opened stops the run before any traffic. In the flow it passes at
  once.
- Its address is a literal `IP:port`. Its matching patterns take parameters
  only; its replies are templates read with what arrived (`{{request.…}}`)
  and the run's parameters. It cannot read secrets.
- Its random choices — a weighted mix of responses, delay jitter, generators in
  replies — draw from the run's seed.
- An HTTP emulator is also what a [[ui:exp.node.wait_http]] on the same address
  listens to: it checks what the system under test sent. Without an emulator
  there, the run's own listener answers every request with `204`.
- An OSC or UDP emulator shares its port with the run's waits on it: both see
  every datagram.
- An MQTT emulator is a broker the run's [[ui:exp.node.mqtt]] and
  [[ui:exp.node.wait_mqtt]] nodes can use like any other.

The run report keeps, for each emulator node, its name, protocol and address,
and its counts: requests in all, those no rule took, those that failed, those
that met it down, messages a broker could not deliver to a slow client, and
the hits of each rule.

## Emulator down and up {#emulator-state}

[[ui:exp.node.emulator_state]] names one of the run's emulators in
[[ui:exp.emulatorNode]]; [[ui:exp.emulatorDownState]] is
[[ui:exp.emulatorGoesDown]] or [[ui:exp.emulatorComesUp]]. While it is down:

| Emulator | Meets |
| --- | --- |
| HTTP | what [[ui:exp.downFault]] says: [[ui:emu.outageFault.unavailable]] (`503`, body `{"error":"unavailable"}`), [[ui:emu.outageFault.reset]], or [[ui:emu.outageFault.timeout]] — the request is held until the client gives up, at most 120 s |
| TCP device, MQTT broker | connections are dropped and new ones refused |
| OSC, UDP device | nothing is answered |

What arrives while it is down is counted as `down`, never as a request no rule
took. A [[ui:exp.node.wait_http]] still sees the requests. The emulator stays
down until a step brings it up, whatever its own outage schedule says, and the
run's end closes it either way.

An [[ui:exp.node.emulator_state]] that names no emulator of the experiment is
refused (`emulator.node_unknown`).

## Outages on a schedule {#outage}

An emulator can also go down by itself: in its rules, [[ui:emu.outage]] sets
[[ui:emu.outageUp]] and [[ui:emu.outageDown]], each 10–3 600 000 ms, and
[[ui:emu.outageFault]] for HTTP. It answers for the first, is down for the
second, and so on, counting from when it opened — in a run, before the first
step. While down on its schedule, an HTTP emulator's `503` carries
`Retry-After` with the whole seconds until it is back, at least 1; a `503`
while an [[ui:exp.node.emulator_state]] step holds it down has none, since
nobody knows when that ends.

A schedule needs no step; a step needs no schedule. Use the schedule for a
dependency that flaps, the step for an outage at a chosen point of the flow.

## Example: an outage behind a slow link {#example}

A client asks an API for an order through a relay. While it asks, a second
branch slows the link to [[ui:ns.preset.4g]], takes the API down for two
seconds, brings it back and makes the link clean again. The client must keep
asking until it gets its answer.

```text
start → orders → link → split
split ─ branch1 → settle → until_ok ─ done → answered → joined
                           until_ok ─ body → get → status → pause → until_ok
split ─ branch2 → slow → down → outage → up → clean → joined
joined → end
```

The names are the nodes' ids in the file below.

1. Add a parameter `api` = `http://127.0.0.1:18091` — the relay, not the API.
2. Add an [[ui:exp.node.emulator]]: HTTP, `127.0.0.1:18090`, a route
   `GET /orders/:id` answering `200` with `{"order":"{{request.params.id}}"}`.
3. After it, an [[ui:exp.node.impairment]]: [[ui:exp.relayListen]]
   `127.0.0.1:18091`, [[ui:exp.relayTarget]] `127.0.0.1:18090`,
   [[ui:ns.protocol]] TCP, preset [[ui:ns.preset.lan]].
4. After it, a [[ui:exp.node.fork]].
5. On [[ui:exp.branch1]], the client: a 300 ms [[ui:exp.node.delay]], then a
   [[ui:exp.node.loop]] — [[ui:exp.loopMax]] 40, [[ui:exp.loopUntilOn]]
   `{{status}}` [[ui:exp.op.eq]] `200`. Its body: an [[ui:exp.node.http]]
   `GET {{api}}/orders/42`, an [[ui:exp.node.extract]] of the
   [[ui:exp.from.status]] into `status`, a 250 ms delay, wired back to the
   Loop. On [[ui:exp.portDone]], a
   [[ui:exp.node.log]] `Orders API answers again: HTTP {{status}}`.
6. On [[ui:exp.branch2]], the faults: a [[ui:exp.node.impairment_change]] of
   the relay to [[ui:ns.preset.4g]]; an [[ui:exp.node.emulator_state]] taking
   the Orders API [[ui:exp.emulatorGoesDown]] with
   [[ui:emu.outageFault.unavailable]]; a 2000 ms delay; another
   [[ui:exp.node.emulator_state]] bringing it [[ui:exp.emulatorComesUp]];
   another [[ui:exp.node.impairment_change]] back to [[ui:ns.preset.lan]].
7. Wire both branches into a [[ui:exp.node.join]], and the Join to
   [[ui:exp.node.end]].
8. Run it.

The timeline shows the client's requests answered `503` through the slow link,
the API coming back, then `200` and the Loop leaving through
[[ui:exp.portDone]]. The report counts five or so requests that met the API
down and one answered by its route, and the relay's three phases —
[[ui:ns.preset.lan]] for an instant, [[ui:ns.preset.4g]] for the outage,
[[ui:ns.preset.lan]] again — each with its own traffic.

::: details The experiment as a file
Save it as a `.json` file and open it with [[ui:exp.importJson]] in
[[ui:exp.documents]].

```json
{
  "version": 9,
  "name": "Outage behind a slow link",
  "params": [{ "name": "api", "value": "http://127.0.0.1:18091" }],
  "profiles": [],
  "profile": null,
  "seed": null,
  "nodes": [
    { "id": "start", "type": "start", "x": 40, "y": 270 },
    { "id": "orders", "type": "emulator", "x": 260, "y": 270,
      "emulator": { "name": "Orders API", "bind": "127.0.0.1:18090", "protocol": "http",
        "routes": [{ "method": "GET", "path": "/orders/:id", "when": [], "order": "sequence",
          "responses": [{ "status": 200, "headers": [], "body": "{\"order\":\"{{request.params.id}}\"}", "delay_ms": 0, "jitter_ms": 0, "fault": "none", "weight": 1 }] }],
        "fallback": null } },
    { "id": "link", "type": "impairment", "x": 490, "y": 270, "listen": "127.0.0.1:18091", "target": "127.0.0.1:18090", "protocol": "tcp",
      "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } },
    { "id": "split", "type": "fork", "x": 720, "y": 270 },
    { "id": "settle", "type": "delay", "x": 950, "y": 140, "ms": 300 },
    { "id": "until_ok", "type": "loop", "x": 1180, "y": 140, "max": 40,
      "until": { "value": "{{status}}", "op": "eq", "expected": "200" } },
    { "id": "get", "type": "http", "x": 1410, "y": 20,
      "request": { "method": "GET", "url": "{{api}}/orders/42", "headers": [], "body": null, "timeout_ms": 3000 } },
    { "id": "status", "type": "extract", "x": 1640, "y": 20, "variable": "status", "from": "status", "expr": "" },
    { "id": "pause", "type": "delay", "x": 1870, "y": 20, "ms": 250 },
    { "id": "answered", "type": "log", "x": 1410, "y": 140, "message": "Orders API answers again: HTTP {{status}}" },
    { "id": "slow", "type": "impairment_change", "x": 950, "y": 400, "relay": "link",
      "profile": { "name": "4g", "latency_ms": 60, "jitter_ms": 25, "rate_kbps": 20000 } },
    { "id": "down", "type": "emulator_state", "x": 1180, "y": 400, "emulator": "orders", "down": true, "fault": "unavailable" },
    { "id": "outage", "type": "delay", "x": 1410, "y": 400, "ms": 2000 },
    { "id": "up", "type": "emulator_state", "x": 1640, "y": 400, "emulator": "orders", "down": false, "fault": "unavailable" },
    { "id": "clean", "type": "impairment_change", "x": 1870, "y": 400, "relay": "link",
      "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } },
    { "id": "joined", "type": "join", "x": 2100, "y": 270 },
    { "id": "end", "type": "end", "x": 2330, "y": 270 }
  ],
  "edges": [
    { "from": "start", "to": "orders" },
    { "from": "orders", "to": "link" },
    { "from": "link", "to": "split" },
    { "from": "split", "to": "settle", "port": "branch1" },
    { "from": "split", "to": "slow", "port": "branch2" },
    { "from": "settle", "to": "until_ok" },
    { "from": "until_ok", "to": "get", "port": "body" },
    { "from": "get", "to": "status" },
    { "from": "status", "to": "pause" },
    { "from": "pause", "to": "until_ok" },
    { "from": "until_ok", "to": "answered", "port": "done" },
    { "from": "answered", "to": "joined" },
    { "from": "slow", "to": "down" },
    { "from": "down", "to": "outage" },
    { "from": "outage", "to": "up" },
    { "from": "up", "to": "clean" },
    { "from": "clean", "to": "joined" },
    { "from": "joined", "to": "end" }
  ]
}
```
:::

Two templates in [[ui:exp.documents]] do the same in other ways:
[[ui:exp.templateFaults]] sends datagrams to an emulated device through a UDP
relay switched clean, lossy, offline and clean again;
[[ui:exp.templateOutage]] takes an emulated API down for two seconds while a
client keeps asking.

## Ports {#ports}

The sockets of one run — waits, replies, emulators, relays — cannot share a
port of one protocol; a UDP and a TCP socket may use the same number. For a
relay's [[ui:exp.relayListen]], an address on `0.0.0.0` clashes with every
address on the same port.

| Socket | Cannot share its port with |
| --- | --- |
| an HTTP, TCP or MQTT emulator | another of them (`emulator.bind_taken`) |
| an OSC or UDP emulator | another of them (`emulator.bind_taken`) |
| a TCP or MQTT emulator | a [[ui:exp.node.wait_http]] (`emulator.bind_taken`) |
| a UDP relay's [[ui:exp.relayListen]] | another UDP relay, an OSC or UDP emulator, a wait or a reply's socket (`impair.bind_taken`) |
| a TCP relay's [[ui:exp.relayListen]] | another TCP relay, an HTTP, TCP or MQTT emulator, a [[ui:exp.node.wait_http]] (`impair.bind_taken`) |

Shared on purpose: an HTTP emulator and the [[ui:exp.node.wait_http]] steps on
its address; an OSC or UDP emulator and the waits on its port; waits on one
address among themselves.

A relay may not forward into itself, directly or through other relays: its
traffic would circle on loopback (`impair.loop`). Two relays in a row in front
of a device are fine.

## Repeating a faulty run {#seed}

Every decision a relay makes — whether a packet is lost, duplicated, corrupted
or held back, how much jitter it gets — is drawn from the run's seed, separately
for each direction and packet by packet. An emulator's random choices draw
from it too. Run again with the same seed and the same traffic, and the same
packets meet the same fate: a failure seen once can be seen again.

To keep the seed, press [[ui:exp.pinSeed]] next to it in the timeline, or run
with it from [[ui:exp.runWith]]; see [seeds](runs.md#seeds). What the seed
cannot hold still is timing: when the system under test sends, and so which
phase a packet falls into.

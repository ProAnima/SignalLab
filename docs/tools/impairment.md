---
title: Impairment
description: Put a relay between a client and its target that delays, drops, duplicates, reorders or throttles UDP datagrams, or delays, throttles, resets or stalls TCP streams.
---

# Impairment

The [[ui:nav.netsim]] screen runs a relay that sits between a client and the
target it talks to, and degrades what goes through it in both directions: a
bad Wi-Fi, a mobile link, a satellite hop, a link that drops out. Use it to see
how your client and your device behave on a network you do not have at hand.

You point the client at the relay instead of the real target; the relay passes
everything on to the target, and the answers back to the client, after doing
to each datagram or stream what its profile says. You can change the profile
while it runs, without dropping the port.

## Starting a relay {#start}

1. Pick the [[ui:ns.protocol]]: UDP for datagrams (OSC, most show-control and
   sensor traffic), TCP for streams (HTTP, MQTT, a TCP device).
2. Set [[ui:ns.listen]]: the `IP:port` the relay listens on. `0.0.0.0:9010`
   takes traffic from the network; `127.0.0.1:9010` only from this computer.
3. Set [[ui:ns.target]]: the `IP:port` of the real destination, or a host name
   and port such as `device.local:9000`.
4. Pick a [preset](#presets) or set the values of the [profile](#profile).
5. Press [[ui:ns.startRelay]].
6. Point your client at the relay's port instead of the target — for example
   `127.0.0.1:9010` instead of `127.0.0.1:9000`.

[[ui:ns.stopRelay]] closes the relay; nothing it still held back goes out
after that.

Defaults: UDP, listen `0.0.0.0:9010`, target `127.0.0.1:9000`, 40 ms latency,
15 ms jitter and 2 % loss.

[[ui:ns.listen]] is an address to bind, always `IP:port`. A host name in
[[ui:ns.target]] is looked up once, when you press [[ui:ns.startRelay]], the way
an [OSC](../protocols/osc.md) target is; a name that cannot be found is
refused and no relay starts. [[ui:ns.protocol]], [[ui:ns.listen]] and
[[ui:ns.target]] are fixed while the relay runs; stop it to change them.

```text
client ──► relay (0.0.0.0:9010) ──► target (127.0.0.1:9000)
client ◄── relay ◄───────────────── target
```

### UDP {#udp}

The relay sends each datagram on to the target from a port of its own, and
sends what the target answers back to the client. Each datagram meets its own
fate, decided when it arrives.

Answers go to the client that sent the most recent datagram: the relay serves
one client at a time.

### TCP {#tcp}

Each connection a client makes to the relay is joined to a new connection of
the relay's own to the target. Both streams of each connection — client to
target and target to client — are impaired chunk by chunk, as the relay reads
them (up to 16 KiB at a time). A stream always arrives in order, whatever the
jitter.

When the target refuses the connection, the client's connection is reset.

## The profile {#profile}

The values a relay applies, in both directions. A UDP relay reads the
datagram's values, a TCP relay the stream's; the others are hidden.

### Over UDP {#profile-udp}

| Value | What it does | Range |
| --- | --- | --- |
| [[ui:ns.offline]] | Every datagram is dropped, both ways, until you untick it. | on / off |
| [[ui:ns.latency]] | Added to every datagram. | 0–1000 ms |
| [[ui:ns.jitter]] | A random extra delay, from 0 to this, for each datagram — so datagrams can arrive out of order. | 0–500 ms |
| [[ui:ns.loss]] | The share of datagrams that never arrive, each on its own. | 0–100 % |
| [[ui:ns.burst]] | The chance a datagram starts a burst of losses: a link that drops out for a moment, unlike scattered loss. | 0–20 % |
| [[ui:ns.burstLength]] | How many datagrams a burst loses on average. Shown when [[ui:ns.burst]] is above 0; 5 when you first raise it. | 1–1000 |
| [[ui:ns.duplicate]] | The share of datagrams delivered twice. | 0–100 % |
| [[ui:ns.corrupt]] | The share of datagrams with one bit of one byte flipped. | 0–100 % |
| [[ui:ns.reorder]] | The share of datagrams held back — by the latency, and at least 20 ms — so the ones after them arrive first. | 0–100 % |
| [[ui:ns.rate]] | A bandwidth limit, in kilobits per second; 0 is none. Datagrams queue for the link at this rate; one that would wait more than a second is dropped as throttled. | 0, or 8–10 000 000 |

A burst works like this: a datagram that is not in a burst starts one with the
[[ui:ns.burst]] chance; every datagram in a burst is lost, and each ends it
with a chance of 1 in [[ui:ns.burstLength]], so a burst lasts that many
datagrams on average.

A datagram is decided in this order: offline, burst, loss, bandwidth, then
duplication, corruption, delay and reordering for each copy.

### Over TCP {#profile-tcp}

| Value | What it does | Range |
| --- | --- | --- |
| [[ui:ns.offline]] | Nothing flows, either way, and new connections wait for the target, until you untick it — then it all goes on. | on / off |
| [[ui:ns.latency]] | Added to every chunk of a stream, both ways. | 0–1000 ms |
| [[ui:ns.jitter]] | A random extra delay, from 0 to this, for each chunk — never ahead of the chunk before it. | 0–500 ms |
| [[ui:ns.reset]] | The share of chunks that reset their connection instead of going through: both the client and the target get a reset. | 0–100 % |
| [[ui:ns.stall]] | The share of chunks that leave their connection half-open: nothing more goes through, either way, and neither side is told. The relay keeps both sides open, untouched, until it stops. | 0–100 % |
| [[ui:ns.rate]] | A bandwidth limit for each stream of each connection, in kilobits per second; 0 is none. Past a second of queue the relay stops reading, so the sender slows down, as on a slow link. Nothing is dropped. | 0, or 8–10 000 000 |

Loss, bursts, duplication, corruption and reordering do not apply to TCP: a
real TCP stream retransmits what it loses and puts itself in order, so what a
client meets on a bad link is delay, a slow sender, resets and connections that
stop answering.

::: tip
The sliders stop at 1000 ms of latency and 500 ms of jitter. The relay itself
takes up to 60 000 ms of each, for example from an experiment file.
:::

## Presets {#presets}

A preset sets every value in one click. Its chip stays lit while the values are
still the preset's; change any value and it goes out.

| Preset | Over UDP | Over TCP |
| --- | --- | --- |
| [[ui:ns.preset.lan]] | 1 ms ±1 | 1 ms ±1 |
| [[ui:ns.preset.wifi]] | 20 ms ±30, 1 % loss, bursts 1 % × 3, 0.5 % duplicated, 2 % reordered | 20 ms ±30 |
| [[ui:ns.preset.4g]] | 60 ms ±25, 0.5 % loss, 0.5 % reordered, 20 000 kbit/s | 60 ms ±25, 20 000 kbit/s |
| [[ui:ns.preset.satellite]] | 300 ms ±30, 1 % loss, 2000 kbit/s | 300 ms ±30, 2000 kbit/s |
| [[ui:ns.preset.intermittent]] | 30 ms ±20, bursts 3 % × 15 | 30 ms ±20, 0.2 % of chunks left half-open |
| [[ui:ns.preset.offline]] | Nothing gets through | Nothing flows |

## Changing it while it runs {#live}

Change any value, or pick another preset, while the relay runs: it applies a
quarter of a second after your last change, without dropping the port or the
connections. The console notes each new profile, and the line under
[[ui:ns.live]] says what the relay does now — the preset's name, or the values
in short, such as `60 ms ±25 · loss 2% · 20000 kbps`.

A datagram or chunk is decided by the profile in force when it arrives; one
already on its way keeps its delay.

## The counters {#counters}

[[ui:ns.live]] shows what the relay has done since it started, both directions
together, updated four times a second.

Over UDP:

| Counter | What |
| --- | --- |
| [[ui:ns.received]] | Datagrams that reached the relay. |
| [[ui:ns.forwarded]] | Datagrams sent on; a duplicated one counts twice. |
| [[ui:ns.dropped]] | Lost on purpose: offline, a burst, or loss. |
| [[ui:ns.throttled]] | Dropped by the bandwidth limit, or because 10 000 were already on their way. |
| [[ui:ns.duplicated]] | Datagrams sent twice. |
| [[ui:ns.corrupted]] | Copies with a bit flipped. |
| [[ui:ns.reordered]] | Copies held back so later ones overtook them. |
| [[ui:common.volume]] | Bytes sent on. |

Over TCP:

| Counter | What |
| --- | --- |
| [[ui:ns.connections]] | Connections clients made to the relay. |
| [[ui:ns.received]] | Chunks the relay read, both ways. |
| [[ui:ns.forwarded]] | Chunks it wrote on. |
| [[ui:ns.resets]] | Connections reset. |
| [[ui:ns.stalled]] | Connections left half-open. |
| [[ui:ns.held]] | How often a stream waited for the bandwidth limit: the sender was slowed down, nothing was dropped. |
| [[ui:common.volume]] | Bytes written on. |

With capture on, the [Inspector](inspector.md) shows relayed datagrams and
chunks — at most one every 25 ms, both directions together — each with its
fate: `forwarded +43ms`, `· corrupted`, `· reordered`, `· copy 2/2`,
`dropped (loss)`, `dropped (burst)`, `dropped (offline)`, `throttled`, and over
TCP `reset` and `half-open`. → is client to target, ← target to client.

A relayed frame names real sockets: [[ui:ins.local]] is the address the relay
listens on and [[ui:bc.peer]] is where that datagram or chunk was going — the
target, or the client an answer went back to. The leg ends the verdict:
`forwarded +43ms · client→target`, `dropped (loss) · target→client`. So a
relayed datagram can be kept with [[ui:sig.fromFrame]] and is aimed at the
address it was going to; a TCP chunk is a piece of a stream and cannot (see
[Inspector](inspector.md#save-as-signal)).

## The same traffic, the same fate {#seed}

Every decision — which datagram is lost, delayed by how much, corrupted where —
is drawn from the relay's seed, separately for each direction and, over TCP,
for each connection. With the same seed and the same traffic, the relay drops
the same datagrams and delays them the same.

The [[ui:nav.netsim]] screen takes a new seed each time you start a relay. To
repeat a run exactly, use an [[ui:exp.node.impairment]] node in an
experiment: it draws from the run's seed, which you can pin (see
[Repeating a faulty run](../experiments/faults.md#seed)).

## In experiments {#experiments}

Two nodes put the same relay into an experiment:

- [[ui:exp.node.impairment]] opens a relay before the first step and closes it
  when the run ends, however it ends. Its listen and target addresses take
  parameters only; the target may be a host name, looked up when the run
  starts. See [Nodes](../experiments/nodes.md#node-impairment).
- [[ui:exp.node.impairment_change]] switches a relay of the run to another
  profile from that step on — clean, lossy, offline, clean again — and the run's
  report counts what each phase did. See
  [Nodes](../experiments/nodes.md#node-impairment_change).

On an OSC or UDP node, [[ui:exp.routeThrough]] puts an
[[ui:exp.node.impairment]] in front of it and points the node at the relay.
See [Faults](../experiments/faults.md).

## Related {#related}

- [Emulators](emulators.md) — the target to put behind the relay.
- [Inspector](inspector.md) — each datagram's fate.
- [UDP and TCP](../protocols/udp-tcp.md)

---
title: Broadcast
description: Send one payload to a list of hosts, a broadcast address, a multicast group or every host of a subnet, once or as a beacon, and listen for who answers with the discovery listener.
---

# Broadcast

The [[ui:nav.broadcast]] screen ([[ui:bc.title]]) sends one UDP payload to many
destinations at once, and listens on the other side to see who answers. Use it
to find devices on a network, to check that a multicast stream reaches a
receiver, or to play a device that answers discovery probes.

- The [[ui:bc.emitter]] sends to a list of hosts, a broadcast address, a
  multicast group, or every host of a subnet — once, or again and again as a
  beacon.
- The [[ui:bc.discovery]] listens on a port, joins multicast groups, lists every
  peer that talks to it, and can answer probes.

::: danger
Broadcast, multicast and a sweep reach every device on the network segment,
not only the one you have in mind, and a beacon keeps doing it. Check which
network you are on first, and only send to networks you own or are allowed to
test. The limits below are guard rails, not permission.
:::

## Sending {#send}

1. Choose the [[ui:bc.mode]] (below).
2. Enter the destination: the field's name changes with the mode. On a network
   with an IPv4 address, [[ui:bc.useSubnet]] fills it from this machine's
   address.
3. Choose the [[ui:bc.payload]] and write it.
4. Press [[ui:bc.sendOnce]]: one datagram goes to each destination.

### Modes {#modes}

| [[ui:bc.mode]] | Destination | What happens | Default |
| --- | --- | --- | --- |
| [[ui:bc.mode.list]] | [[ui:bc.targetList]]: `IP:port` or `host:port` entries separated by commas, semicolons or new lines — a space does not separate them. A name is looked up, its IPv4 address taken when it has one. | One datagram to each | `127.0.0.1:9000, 127.0.0.1:9001` |
| [[ui:bc.mode.broadcast]] | [[ui:bc.targetAddress]]: `255.255.255.255:port`, or an address ending in `.255` | One datagram that every host on the local network receives. Routers do not pass it on. | `255.255.255.255:9000` |
| [[ui:bc.mode.multicast]] | [[ui:bc.targetAddress]]: a group from `224.0.0.0` to `239.255.255.255`, with a port | One datagram to the group; only listeners that joined it receive it | `239.1.1.1:9000` |
| [[ui:bc.mode.sweep]] | [[ui:bc.targetCidr]], `a.b.c.d/nn`, and a [[ui:bc.port]] | One datagram to every usable host of the block, as unicast — for devices that ignore broadcast | `192.168.1.0/24`, port 9000 |

In a sweep the network and broadcast addresses are skipped (except in a `/31` or
`/32`), and a base that is not the network's own is rounded down to it:
`192.0.2.77/30` sweeps `192.0.2.77` and `192.0.2.78`. A sweep reaches at most
1024 hosts, so the widest block is a `/22` (1022 hosts); a wider one is refused
with the prefix to narrow it to.

Broadcast, multicast groups, sweeps and the discovery listener's joins are IPv4
only: IPv6 has no broadcast. A [[ui:bc.mode.list]] may also name IPv6 hosts
(see [Socket options](#socket-options)).

[[ui:bc.useSubnet]] fills `x.y.z.10:9000, x.y.z.11:9000` in [[ui:bc.mode.list]],
`x.y.z.255:9000` in [[ui:bc.mode.broadcast]] and `x.y.z.0/24` in
[[ui:bc.mode.sweep]], from this machine's address `x.y.z.w`. It assumes a `/24`
network.

### Payload {#payload}

| [[ui:bc.payload]] | What is sent |
| --- | --- |
| [[ui:bc.payload.osc]] | An OSC message: [[ui:common.address]] and typed [[ui:common.arguments]], as on the [OSC](osc.md) screen. Default `/hello/discover` with the text `who-is-there`. |
| [[ui:bc.payload.text]] | The [[ui:bc.text]] as UTF-8, exactly as typed, with no terminator. Default `HELLO-PROBE`. |
| [[ui:bc.payload.hex]] | The [[ui:bc.hex]] byte for byte — to replay a captured frame or speak a binary discovery protocol. Pairs of hex digits; anything else between them is ignored. Default `48 45 4c 4c 4f`. |

### Socket options {#socket-options}

[[ui:bc.socketOptions]] opens three more settings:

| Option | What | Default |
| --- | --- | --- |
| [[ui:bc.bindSource]] | The local `IP:port` the datagrams go out from. Pin it to choose the network card, or a source port a device answers to. `0.0.0.0:0`: any. | `0.0.0.0:0` |
| [[ui:bc.ttl]] | How many routers a datagram may cross, 1–255. For multicast this is the multicast hop limit: 1 keeps it on this network. | `1` |
| [[ui:bc.mcastLoop]] | Multicast only: deliver the group's datagrams to this machine too, so a listener here hears them | on |

With the default [[ui:bc.bindSource]] the datagrams go out from an IPv4 socket,
or from an IPv6 one when every destination is IPv6. A list that mixes the two
is sent from the IPv4 socket, and its IPv6 destinations fail; send them as a
list of their own, or pin [[ui:bc.bindSource]] to an IPv6 address.

### The result {#result}

[[ui:bc.lastEmit]] shows what went out: [[ui:bc.targets]], [[ui:common.packets]],
[[ui:common.volume]] and [[ui:common.errors]], and the first eight destinations
it reached ("… +n more" for the rest). An error to one destination does not stop
the others; the console line names how many failed.

## Repeating as a beacon {#beacon}

A beacon sends the same round — one datagram to each destination — on a
schedule, until you stop it. Devices that listen for a periodic announcement
need one.

1. Set up the mode, destination and payload as for a single send.
2. Under [[ui:bc.beacon]], set [[ui:bc.beaconRate]], and if it should stop by
   itself, [[ui:bc.beaconRounds]] or [[ui:bc.beaconSeconds]].
3. Press [[ui:bc.startBeacon]]. [[ui:bc.stopBeacon]] — or stopping its job in the
   console strip — ends it.

| Field | What | Default |
| --- | --- | --- |
| [[ui:bc.beaconRate]] | Rounds per second; must be above 0 | `2` |
| [[ui:bc.beaconRounds]] | Stop after this many rounds; 0 — no limit | `0` |
| [[ui:bc.beaconSeconds]] | Stop after this many seconds; 0 — no limit | `0` |

The rate times the number of destinations may be at most 50 000 datagrams a
second. A sweep of a `/24` (254 hosts) can therefore repeat about 196 times a
second at most. While the beacon runs, [[ui:bc.lastEmit]] shows
[[ui:bc.targets]] (the destinations in each round, from the first report on),
the totals, [[ui:bc.rounds]] and [[ui:bc.pps]] (datagrams per second), updated
four times a second, and [[ui:bc.sendOnce]] is unavailable. A beacon that has had more than
32 failed datagrams and not one sent — no route, broadcast not allowed — stops
by itself and says why.

## Listening for devices {#discovery}

The [[ui:bc.discovery]] binds a UDP port and records every peer that sends to
it: what answers a probe, or what a device announces on its own.

1. Set [[ui:common.bind]], the port the devices send to.
2. For multicast, list the groups in [[ui:bc.joinGroups]].
3. Press [[ui:bc.startListen]]. The settings lock until you press
   [[ui:bc.stopListen]].

| Field | What | Default |
| --- | --- | --- |
| [[ui:common.bind]] | Where to listen, `IP:port`. `0.0.0.0` listens on every network card. | `0.0.0.0:9000` |
| [[ui:bc.joinGroups]] | IPv4 multicast groups to join, comma separated; empty — unicast and broadcast only | `239.1.1.1` |
| [[ui:bc.interface]] | The IPv4 address of the network card to join the groups on; empty — the system chooses | empty |
| [[ui:bc.reuse]] | Listen on a port another program also uses (`SO_REUSEADDR`). It works only if that program allows sharing too. | on |
| [[ui:bc.respond]] | Answer probes, as a device would ([below](#auto-reply)) | off |

### Peers {#peers}

[[ui:bc.peers]] lists who has sent something, the most recent first, with the
number of peers seen, the packets heard and the replies sent above it:

| Column | What |
| --- | --- |
| [[ui:bc.peer]] | The sender's `IP:port`; its dot shows whether it was heard in the last 3 seconds |
| [[ui:bc.proto]] | `osc` when its last datagram decoded as OSC, else `udp` |
| [[ui:common.packets]] | How many it sent |
| [[ui:bc.age]] | Seconds since its last datagram |
| [[ui:bc.lastMessage]] | Its last datagram: the OSC address and arguments, or the start of the text |

The list is refreshed a few times a second and holds up to 512 peers; past
that, packets are still counted but new peers get no row.

### Answering probes {#auto-reply}

With [[ui:bc.respond]], the listener plays a device: it answers each datagram
it receives, from the listening port, back to the sender's address and port.

| Field | What | Default |
| --- | --- | --- |
| [[ui:bc.payload]] | The reply: OSC, text or hex, as for sending | OSC `/hello/here` with the text `signal-lab` |
| [[ui:bc.replyDelay]] | Wait this long before answering, as a slow device would | `0` |
| [[ui:bc.matchContains]] | Answer only datagrams whose decoded text contains this — the OSC address and arguments, or the start of the text; empty — every one | empty |

The listener never answers a datagram identical to its own reply, so two
listeners pointed at each other do not answer back and forth forever.

### Firewalls and shared ports {#firewall}

Broadcast and multicast traffic from other machines is blocked by default by
most Windows firewalls: allow Signal Lab on private networks when the app
offers it. Broadcast never crosses a router. To listen on a port the real
service already has, both sides must allow sharing ([[ui:bc.reuse]] here);
without it, a taken port is refused with a hint to turn it on. See
[Troubleshooting](../reference/troubleshooting.md).

## In the Inspector {#inspector}

With capture armed, the screen's traffic appears with the protocol `osc` or
`udp`, by its payload:

| Source | What | How many |
| --- | --- | --- |
| `broadcast` | [[ui:bc.sendOnce]]: each datagram, with the verdict `fan-out`, `broadcast`, `multicast` or `sweep`; a failed one with `error: …` | every one |
| `beacon` | A beacon's rounds | at most one round every 50 ms |
| `discovery` | Datagrams the listener receives | at most one every 40 ms |
| `discovery` | Its answers, with the verdict `auto-reply` | every one |

What the Inspector leaves out of a beacon or the listener is counted: the next
frame it draws carries the number in its verdict, `+n not shown` (for a beacon,
one frame for each destination of each round left out). See
[Inspector](../tools/inspector.md).

## On a server or in Docker {#server}

On a [server](../server/index.md), the screen sends and listens on the server's
network. In Docker, broadcast, multicast and discovery reach the local network
only when the container uses the host's network (`--network host`) on a Linux
host. With Docker's default bridge network, or Docker Desktop, only unicast to
hosts the container can reach works.

## Elsewhere {#elsewhere}

- [`signallab send udp`](../automation/cli.md#cli-send-udp) sends one datagram to
  one host; there is no command-line broadcast, multicast or sweep.
- A [[ui:exp.node.udp]] step sends a text datagram to one or more hosts, and a
  [[ui:exp.node.wait_udp]] step waits for one. See [UDP and TCP](udp-tcp.md).
- To play a device that answers by rules — several rules, replies built from
  what arrived — use a [[ui:emu.new.udp]] or [[ui:emu.new.osc]] emulator. See
  [Emulators](../tools/emulators.md).

## Problems {#troubleshooting}

| What you see | Usual cause |
| --- | --- |
| `… is not a broadcast address` | [[ui:bc.mode.broadcast]] takes `255.255.255.255:port` or an address ending in `.255`. For another subnet mask, use [[ui:bc.mode.sweep]]. |
| `… is not a multicast group` | The address is outside `224.0.0.0`–`239.255.255.255`. |
| `… spans … addresses, and a sweep reaches at most 1024 hosts` | The block is wider than a `/22`; narrow it. |
| `Set the port to sweep` | [[ui:bc.port]] is 0. |
| `… is over the … pps limit` | The rate times the number of destinations is over 50 000 a second: lower [[ui:bc.beaconRate]], or narrow the destination. |
| `… is already in use — turn on “share the port” to listen alongside it` | Another program has the port; tick [[ui:bc.reuse]]. |
| `Cannot join the multicast group …` | The group or [[ui:bc.interface]] is not usable on this machine — no network card with that address, or no multicast route. |
| Sent, but nobody answers | The devices listen on another port; the firewall here keeps their answers out; a router lies between you; or, in Docker, the container is not on the host network. |
| Probes go out, but the discovery listener hears no answers | Many devices answer to the address and port a probe came from — the emitter's own socket, which the screen does not read. Listen on the port the devices answer to, or send the probe from an experiment: a [[ui:exp.node.udp]] step with [[ui:exp.expectReply]] sends and listens on the same port. |

Every error message is listed in [Error messages](../reference/errors.md#broadcast).

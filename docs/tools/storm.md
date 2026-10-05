---
title: Storm
description: Send a controlled flood of UDP datagrams or TCP connections at a server you own, and watch the rate, throughput and errors live.
---

# Storm

The [[ui:nav.storm]] screen is a load source: it sends UDP datagrams, or opens
TCP connections, at one target as fast as you ask, for as long as you ask, and
meters what actually went out. Use it to see how your own server, device or
link holds up under a flood — whether it keeps answering, drops packets, or
falls over.

::: danger Responsible use
A storm sends real traffic at a real host. Point it only at hosts and networks
you own or are authorized to test. High rates can saturate links for everyone
on them and trip intrusion detection. The engine does not cap a storm's rate:
0 means as fast as this computer can send.
:::

## Starting a storm {#start}

1. Set [[ui:common.target]]: the `IP:port` or `host:port` to send to, such as
   `127.0.0.1:9000` or `test-rig.local:9000`. A host name is looked up when you
   launch, its IPv4 address taken when it has one.
2. Pick the [[ui:common.protocol]]: [[ui:st.udp]] or [[ui:st.tcp]].
3. Set [[ui:st.payloadSize]], [[ui:st.rate]] and [[ui:st.duration]].
4. Press [[ui:st.launch]].

The storm runs as a job: it shows in the console strip, and stops when its
duration is over, when you press [[ui:st.stop]], or with [[ui:app.stopAll]].
The fields are fixed while it runs.

| Field | What | Default |
| --- | --- | --- |
| [[ui:common.target]] | `IP:port` or `host:port` of the target. | `127.0.0.1:9000` |
| [[ui:common.protocol]] | [[ui:st.udp]]: separate datagrams. [[ui:st.tcp]]: a new TCP connection for each "packet". | [[ui:st.udp]] |
| [[ui:st.payloadSize]] | Bytes in each datagram or connection, 1–65 507. A larger or smaller value is brought into that range. | 512 |
| [[ui:st.rate]] | Packets (or connections) per second to aim for. 0: as fast as possible. | 1000 |
| [[ui:st.duration]] | Seconds to run. 0: until you stop it. | 10 |

### UDP flood {#udp}

Signal Lab sends datagrams of the payload size, every byte `0x55`, from one
port of its own to the target. A send that the system refuses counts as an
error — for example after the target answered that nothing listens on that
port.

### TCP connect flood {#tcp}

For each "packet", Signal Lab opens a TCP connection to the target, writes the
payload, and closes it. Connections are made one after another, not at the same
time, so the rate a TCP storm reaches is limited by how fast the target accepts
them. A connection that is refused, or not accepted within 500 ms, counts as an
error; a connection that took the payload counts as a packet.

### How the rate is kept {#pacing}

The rate is a schedule. The first packet goes out at once, and packet *n* is due
*n* ÷ [[ui:st.rate]] seconds after the start. Each time the storm wakes it sends
what is due, then sleeps until the next packet is due. So 50 per second is 50 per
second and 250 is 250 — a run of one second sends about that many — whatever the
system's timer granularity is. A [[ui:st.duration]] ends the storm on time even
when the next packet would be due later.

A storm never sends above its rate to make up for lost time. If this computer,
or a TCP target that accepts slowly, falls behind by more than 256 packets, the
older ones are dropped from the schedule rather than sent late in a burst.
[[ui:st.pps]] shows what was reached.

With [[ui:st.rate]] 0 there is no schedule: the storm sends 256 packets, lets
other work run, and sends the next 256, as fast as this computer can.

## Reading the throughput {#metrics}

[[ui:st.throughput]] updates four times a second:

| Metric | What |
| --- | --- |
| [[ui:common.packets]] | Datagrams sent, or connections that took the payload, since the start. |
| [[ui:st.pps]] | Packets per second over the last quarter of a second. |
| [[ui:st.rateLabel]] | Megabits per second of payload over the last quarter of a second. Headers (IP, UDP, TCP) are not counted. |
| [[ui:common.volume]] | Payload bytes sent since the start. |
| [[ui:common.errors]] | Sends or connections that failed. |

The chart under them plots [[ui:st.pps]] over the last minute.

When the storm ends, the counters show the final totals, and [[ui:st.pps]] and
[[ui:st.rateLabel]] go to 0.

What the numbers tell you:

- [[ui:st.pps]] well below [[ui:st.rate]] on UDP: this computer cannot send
  faster. Lower the rate or the payload size.
- [[ui:common.errors]] rising on UDP: the target's port is closed, or the
  network refuses the traffic.
- [[ui:common.errors]] rising on TCP: the target refuses connections or takes
  longer than 500 ms to accept them — it may have reached its limit.

A storm says how much went out, not how much arrived. To see what the target
received, watch it: its own logs, an [[ui:nav.osc]] monitor, or an
[emulator](emulators.md) in its place.

## In the Inspector {#inspector}

With capture on, a UDP storm puts one of its datagrams in the
[Inspector](inspector.md) each second, marked `sampled 1/s` — they are all the
same. The verdict also says how many were left out since the previous one, as
`sampled 1/s · +999 not shown`. A TCP storm puts none there.

## Related {#related}

- [Impairment](impairment.md) — a slow or lossy link instead of a flood.
- [HTTP](../protocols/http.md) — load bursts of HTTP requests, with latencies.
- [Load](../experiments/load.md) — HTTP load in an experiment, with thresholds.

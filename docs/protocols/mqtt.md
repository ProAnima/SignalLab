---
title: MQTT
description: Connect to an MQTT 3.1.1 broker, watch every topic it holds as a live tree, publish at QoS 0, 1 or 2, announce a last will and clear stuck retained values.
---

# MQTT

The [[ui:nav.mqtt]] screen is an MQTT client for looking at a broker and
changing what is in it. Connect, and by default it subscribes to `#`: every
topic the broker holds builds up as a tree with its latest value. From there you
publish, clear a retained value, save a topic as a signal or turn it into an
experiment step.

Signal Lab speaks **MQTT 3.1.1 over plain TCP**, with QoS 0, 1 and 2 for
subscribing, publishing and the last will. There is no MQTT 5 and no TLS: a
broker that only accepts `mqtts://` or MQTT 5 clients cannot be reached.

## Connecting {#connect}

1. Open [[ui:nav.mqtt]].
2. Enter [[ui:mq.host]] and [[ui:common.port]].
3. Leave [[ui:mq.clientId]] as it is unless the broker expects a particular
   one. Add [[ui:mq.username]] and [[ui:mq.password]] only if the broker asks.
4. Press [[ui:mq.connect]].

Connecting opens the TCP connection and completes the MQTT handshake before
anything else, so a wrong password or a closed port is reported right there. The
connection fields lock while connected; [[ui:mq.disconnect]] closes it. The
connection is a job in the console strip and can be stopped there too.

| Field | What | Default |
| --- | --- | --- |
| [[ui:mq.host]] | The broker's IP address or host name | `127.0.0.1` |
| [[ui:common.port]] | The broker's port | `1883` |
| [[ui:mq.clientId]] | Your client's name at the broker. It must not be empty, and must be unique there: a second client with the same id knocks the first one off. | `signal-lab-` and six random hex digits, new at every start of the app |
| [[ui:mq.username]], [[ui:mq.password]] | Sent only if the broker needs them — in plain text, since there is no TLS. A password without a user name is not sent at all: MQTT 3.1.1 cannot carry one. | empty |
| [[ui:mq.keepAlive]] | Seconds the connection may stay silent. Signal Lab pings the broker every half of it; a broker drops a client that is silent for 1.5 times this. 0 turns pinging off. | `60` |
| [[ui:mq.cleanSession]] | On: every connection starts with no stored subscriptions and no queued messages. Off asks the broker to keep them for this client id between connections. | on |
| [[ui:mq.scanFilter]] and its [[ui:mq.qos]] | A filter subscribed to as soon as the connection is up; `#` is every topic. Empty: none. | `#`, QoS 0 |
| [[ui:mq.willEnable]] | Give the broker a last will ([below](#will)) | off |

The broker has 6 seconds to accept the TCP connection and 6 more to answer the
handshake.

### The last will {#will}

A last will is a message the broker keeps for you and publishes itself if your
connection dies without a proper goodbye. Presence is usually built this way: a
device publishes `online` to its status topic, and its will sets the same topic
to `off`.

With [[ui:mq.willEnable]] ticked, set [[ui:mq.willTopic]] and
[[ui:mq.willPayload]] (`off` by default). The will is published at QoS 2 and
retained. Without a topic, no will is sent.

## Subscribing {#subscribe}

The scan filter is subscribed at connect. For more:

1. In [[ui:mq.addSubscription]], type a topic filter.
2. Choose its [[ui:mq.qos]].
3. Press [[ui:mq.subscribe]] or <kbd>Enter</kbd>.

A filter is a topic with wildcards:

| Wildcard | Stands for | Example |
| --- | --- | --- |
| `+` | exactly one level | `sensors/+/state` matches `sensors/door/state` |
| `#` | every level below, the last character only | `sensors/#` matches `sensors/door/state` and `sensors` |

[[ui:mq.subscriptions]] lists each filter with what the broker granted:
`qos0`, `qos1` or `qos2` — the broker may grant less than you asked — or
[[ui:mq.refused]]. [[ui:mq.unsubscribe]] unsubscribes from one.

| QoS | Delivery |
| --- | --- |
| 0 | At most once: sent and forgotten |
| 1 | At least once: acknowledged, may arrive twice |
| 2 | Exactly once: a two-step handshake; a redelivery is not shown twice |

## The topic tree {#topics}

Every message that arrives goes into [[ui:mq.topics]], a tree of the topic
levels. A topic shows its latest value, an **R** when that value is retained,
and how many messages it has had when more than one. Click a level to open or
close it.

- Type in the field above the tree to list only the topics whose path or latest
  value contains the text.
- Above the tree are the number of topics, how many hold a retained value and,
  while connected, the broker you are listening on.
- Payloads are shown as text; bytes that are not UTF-8 show as replacement
  characters.
- [[ui:common.clear]] empties the tree. Nothing else does: it stays as it is
  when you switch screens or disconnect, until the app closes.

Messages reach the screen in batches, ten times a second. When a broker sends
more than 4000 messages in a tenth of a second, the oldest of that batch are
left out of the tree and counted as "not shown" above it.

### A topic's panel {#topic}

Pick a topic with a value to see it below the tree: [[ui:mq.value]],
[[ui:mq.qos]], [[ui:mq.retain]], [[ui:common.bytes]], [[ui:mq.messages]] and
[[ui:mq.lastAt]]. Its buttons:

| Button | Does |
| --- | --- |
| [[ui:mq.editHere]] | Copies the topic, value, QoS and retain flag into [[ui:mq.publish]] |
| [[ui:mq.waitForThis]] | Adds a [Wait for MQTT](../experiments/nodes.md#node-wait_mqtt) step on this topic, at this broker, any payload, 2000 ms timeout, to the open experiment |
| [[ui:mq.clearRetained]] | Removes the retained value ([below](#clear-retained)) |
| [[ui:sig.fromFrame]] | Keeps the topic and its latest value as a signal in the [[ui:sig.capturedFolder]] folder |

## Publishing {#publish}

1. Connect.
2. Under [[ui:mq.publish]], enter the [[ui:mq.topic]] and the
   [[ui:sig.payload]].
3. Choose the [[ui:mq.qos]], and tick [[ui:mq.retain]] if the broker should keep
   the message as the topic's value for every client that subscribes later.
4. Press [[ui:mq.publishBtn]].

The console confirms each publish: at once for QoS 0, when the broker has
acknowledged it for QoS 1 and 2. A topic for publishing has no wildcards and
is not empty: a topic with `+` or `#` is refused before anything is sent, with
the same message a signal, a step and `signallab send mqtt` give, and the
connection stays as it was. Only a subscription takes filters with wildcards.

### Clearing a retained value {#clear-retained}

A retained value stays on the broker until it is replaced, and every client
that subscribes gets it first — a stale one is a classic reason a device boots
into the wrong state. The only way to remove it is to publish an empty payload
with retain set.

[[ui:mq.clearRetained]] in a topic's panel does that: press it, then
[[ui:mq.clearConfirm]]. It publishes the empty retained payload at QoS 1 on
your connection. It is available only while connected and when the topic's
latest value is retained. You can do the same by hand: an empty
[[ui:sig.payload]] with [[ui:mq.retain]] ticked.

::: warning
Clearing changes the broker for every client at once.
:::

## In the Inspector {#inspector}

With capture armed, MQTT traffic appears with the protocol `mqtt`:

| Source | What | How many |
| --- | --- | --- |
| `mqtt` | What the screen's connection publishes; an empty retained publish has the verdict `clears retained` | every one |
| `mqtt` | Messages the connection receives | at most one every 200 ms |
| `mqtt-send` | A publish that brought its own connection: a signal fired while the screen is not connected to the signal's broker, a step, `signallab send mqtt` (verdict `one-shot`) | every one |
| `experiment-wait` | Messages a [[ui:exp.node.wait_mqtt]] step's subscription receives, retained replays aside | every one |

The summary reads `topic = payload`, with the QoS and `retained` when they
apply. See [Inspector](../tools/inspector.md).

## Saving and reusing {#library}

- **Save as a signal.** [[ui:sig.saveNew]] under [[ui:mq.publish]] keeps the
  broker (the connection's [[ui:mq.host]] and [[ui:common.port]]), topic,
  payload, QoS and retain flag in the signal library;
  <kbd>Ctrl</kbd>+<kbd>S</kbd> in the publish panel does the same, and
  updates the signal once the panel is tied to it. See
  [Signals](../tools/signals.md).
- **Firing an MQTT signal.** While this screen is connected to the broker the
  signal names (same host, ignoring case, and same port; `1883` when the signal
  gives none), a signal fired from the library goes out over that connection,
  with its client id and credentials. Otherwise — not connected, or connected to
  another broker — it opens a connection of its own to its own broker — a fresh
  client id, no user name — publishes, waits for the acknowledgement its QoS
  calls for, and disconnects. Names are not looked up, so `localhost` and
  `127.0.0.1` count as different brokers. The library stores no password.
- **In an experiment.** A saved MQTT signal can be picked under
  [[ui:exp.group.signals]] in the experiment's [[ui:exp.addNode]] menu, which
  makes it an [[ui:exp.node.mqtt]] step.

## In experiments {#experiments}

| Step | What it does |
| --- | --- |
| [[ui:exp.node.mqtt]] | Connects, publishes one message and disconnects — no user name or password, a clean session, within 15 seconds. [Details](../experiments/nodes.md#node-mqtt) |
| [[ui:exp.node.wait_mqtt]] | Subscribes when the run starts and waits for a message on a topic filter whose payload matches; retained values replayed on subscribing are ignored. [Details](../experiments/nodes.md#node-wait_mqtt) |
| [[ui:exp.node.emulator]] | An MQTT broker of the run's own. [Details](../experiments/nodes.md#node-emulator) |

Neither step logs in, so they need a broker that accepts clients without a user
name.

### The broker emulator {#broker-emulator}

Signal Lab can also be the broker: an [[ui:emu.new.mqtt]] emulator routes what
clients publish to whoever subscribed — 3.1.1, plain TCP, QoS 0, 1 and 2,
retained messages, wills, an optional login — and answers by rules, like a
device. Point your gear and this screen at it to test without a real broker.
See [Emulators](../tools/emulators.md).

## From the command line {#cli}

`signallab send mqtt` publishes one message with a connection of its own:

```bash
signallab send mqtt 127.0.0.1:1883 lab/light/1/set on --qos 1
signallab send mqtt 127.0.0.1:1883 lab/light/1/state "" --retain
```

```text
✔ lab/light/1/set → 127.0.0.1:1883 · 2 B · qos1
```

The second line clears a retained value. Without a port, the broker is on
1883. It uses no credentials. It exits with 0 when the broker took the message,
1 when it could not be reached or refused it. See
[Command line](../automation/cli.md#cli-send-mqtt).

## Problems {#troubleshooting}

| What you see | Usual cause |
| --- | --- |
| `… refused the connection — nothing is listening on that port` | No broker on that address and port. |
| `… accepted the connection but did not answer in time — is it an MQTT broker?` | Something listens there, but does not speak MQTT, or speaks it over TLS. |
| `… answered with something other than MQTT 3.1.1` | Not an MQTT broker, or a broker that sent something Signal Lab cannot read. |
| `… does not accept MQTT 3.1.1 clients` | The broker only takes MQTT 5. |
| `… rejected the client ID — choose another one` | The id is too long or has characters the broker does not accept. |
| `… rejected the username or password` | Wrong credentials, or a password without a user name. |
| `… did not authorize this client — check its access rules` | The broker's access rules refuse this client. |
| `… is unavailable right now — try again later` | The broker is up but not taking clients. |
| `Enter a client ID — brokers refuse an empty one` | [[ui:mq.clientId]] is empty. |
| `A publish topic cannot contain the wildcards + or #` | The topic to publish to has a `+` or `#`. Those are for subscribing; publish to one topic at a time. |
| A filter shows [[ui:mq.refused]] | The broker's access rules forbid it, or the filter is malformed (`#` not last, `+` sharing a level with other characters). |
| The connection drops a moment after connecting | Another client connected with the same [[ui:mq.clientId]]. |
| Nothing appears in the tree | The scan filter is empty, or the broker lets this client see nothing. |

Every error message is listed in [Error messages](../reference/errors.md#mqtt).

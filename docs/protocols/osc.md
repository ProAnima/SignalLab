---
title: OSC
description: Send typed Open Sound Control messages, watch what arrives on a port and drive a continuous waveform into a device from the OSC screen.
---

# OSC

The [[ui:nav.osc]] screen is where you talk Open Sound Control (OSC 1.0) over
UDP by hand. It has three parts:

- [[ui:osc.sender]]: one message, typed arguments, sent when you press Enter.
- [[ui:osc.monitor]]: listens on a port and decodes every packet that arrives.
- [[ui:osc.generator]]: sends a value that follows a waveform, many times a
  second, and draws it.

Signal Lab encodes and decodes OSC itself. What it sends is what the
[Inspector](../tools/inspector.md) shows, byte for byte.

## Sending a message {#send}

1. Open [[ui:nav.osc]].
2. In [[ui:common.target]], enter the device's IP address or host name and its
   port, for example `127.0.0.1:9000` or `stage-mixer.local:9000`.
3. In [[ui:common.address]], enter the address the device listens for, for
   example `/mixer/fader/1`.
4. Under [[ui:common.arguments]], set each argument's type and value. Press
   [[ui:common.addArgument]] for another; ✕ removes one.
5. Press [[ui:common.send]], or <kbd>Enter</kbd> in any field of the sender.

The line under the buttons says what went out: the address, its size in bytes
and the target. Sending the same message again counts up (×2, ×3…), so you can
see that a repeated send did something. A failure is shown there instead, and
in the console.

The target, address and arguments are kept when you switch screens and when
you restart the app.

### Fields {#send-fields}

| Field | What | Default |
| --- | --- | --- |
| [[ui:common.target]] | `IP:port` or `host:port` of the receiver. An IPv6 address goes in brackets: `[::1]:9000`. A host name is looked up each time you send; when it has an IPv4 address, that one is used (so `localhost` reaches a receiver listening on `127.0.0.1`), otherwise its IPv6 one. A target without a port is refused. | `127.0.0.1:9000` |
| [[ui:common.address]] | The OSC address, starting with `/`, parts separated by `/`. One without the leading `/` is refused before anything is sent. | `/hello/avatar/1` |
| [[ui:common.arguments]] | Typed values after the address, in order. A message may have none. | one `float`, `1.0` |

### Argument types {#types}

The type of each argument is part of the message (its type tag), so a device
that expects a float may ignore an int with the same value.

| Type in the list | OSC tag | Value | How you enter it |
| --- | --- | --- | --- |
| `int` | `i` | 32-bit signed integer | a whole number |
| `float` | `f` | 32-bit floating point | a number, `0.75` |
| `str` | `s` | text | any text, sent as UTF-8 |
| `bool` | `T` / `F` | true or false | `true` or `false` from a list; carries no bytes, only the tag |
| `long` | `h` | 64-bit signed integer | a whole number |
| `double` | `d` | 64-bit floating point | a number |
| `nil` | `N` | nothing | no value |
| `blob` | `b` | bytes | not typed here: it appears, read-only and in hex, when you open a signal that has one |

A number field that does not hold a number sends `0`.

::: tip
An OSC `true` is the tag `T`, not the text `"true"`. A device waiting for a
bool silently ignores a string.
:::

## Bundles {#bundles}

The sender sends single messages, not bundles. When a bundle (`#bundle`)
arrives, the monitor, the experiment waits and the emulators unpack it: each
message inside it is handled on its own, and its time tag is ignored.

## Watching a port {#monitor}

To see what a device or a show controller sends:

1. In [[ui:common.bind]], enter the address and port to listen on. `0.0.0.0:9000`
   (the default) listens on every network card; `127.0.0.1:9000` only on this
   machine.
2. Press [[ui:osc.listen]]. The field locks while the monitor runs.
3. Point the sender at this machine's IP address and that port.

Every packet becomes a row, newest at the top:

| Column | What |
| --- | --- |
| [[ui:common.time]] | When it arrived, to the millisecond |
| [[ui:osc.from]] | The sender's `IP:port` |
| [[ui:osc.address]] | The OSC address, or [[ui:osc.decodeError]] when the packet is not valid OSC |
| [[ui:osc.args]] | The argument values; a blob shows as `blob[n]`, `nil` as `nil`. For a packet that could not be decoded, why. |

A bundle gives one row per message. The list keeps the latest 300 rows;
[[ui:common.clear]] empties it. Press [[ui:common.stop]] to close the port. The
monitor is also a job in the console strip, so it can be stopped from there.

The monitor reads packets of up to 64 KiB. It decodes the tags `i f s S b h d T F N I`
(`S` reads as text, `I` as nil); a packet with any other tag, or cut short, is
shown as a decode error rather than dropped.

### Turning a message into a wait {#wait-for-this}

Each row has a ⇠ button, [[ui:osc.waitForThis]]. It adds a
[Wait for OSC](../experiments/nodes.md#node-wait_osc) step to the open
experiment that listens on the monitor's [[ui:common.bind]] for this address,
with an "equals" rule for each text, whole-number and true/false argument —
floats, blobs and nil get none — up to 16 rules. Its timeout is 2000 ms. The
editor opens with the new step selected.

::: warning
Stop the monitor before you run that experiment. The run opens the same port
itself, and two listeners cannot share it.
:::

## Driving a waveform {#generator}

The [[ui:osc.generator]] sends one message after another to one address, with a
single argument whose value follows a waveform — a fader, a light level, a
position. Use it to see how a device follows a moving value, or to load a
receiver with a steady stream.

1. Set [[ui:common.target]] and [[ui:osc.address]]. Both are checked when you
   press [[ui:osc.startGen]], as the sender checks them.
2. Choose a [[ui:osc.waveform]], its [[ui:osc.freq]] and the [[ui:osc.rate]].
3. Set [[ui:osc.min]] and [[ui:osc.max]], the range of the value.
4. Press [[ui:osc.startGen]]. It runs until you press [[ui:osc.stopGen]] or stop
   its job in the console strip.

| Field | What | Default |
| --- | --- | --- |
| [[ui:common.target]] | `IP:port` or `host:port` of the receiver; a name is looked up once, when the generator starts | `127.0.0.1:9000` |
| [[ui:osc.address]] | The address every message is sent to; it starts with `/` | `/hello/lfo` |
| [[ui:osc.waveform]] | The shape of the value over time (below) | [[ui:wave.sine]] |
| [[ui:osc.freq]] | Cycles of the waveform per second | `1` |
| [[ui:osc.rate]] | Messages per second, from 0.1 to 5000; a value outside is held to that range | `60` |
| [[ui:osc.min]], [[ui:osc.max]] | The lowest and highest value. If Max is below Min, the value does not move. | `0`, `1` |
| [[ui:osc.asInt]] | Round to the nearest whole number and send an `int` instead of a `float` | off |

| Waveform | What the value does in each cycle |
| --- | --- |
| [[ui:wave.sine]] | Swings smoothly between Min and Max, starting at the middle and rising |
| [[ui:wave.triangle]] | Rises from Min to Max, then falls back to Min, starting at Min |
| [[ui:wave.saw]] | A falling sawtooth: starts at Max, falls to Min, then jumps back to Max |
| [[ui:wave.ramp]] | A rising sawtooth: starts at Min, rises to Max, then jumps back to Min |
| [[ui:wave.square]] | Max for the first half, Min for the second |
| [[ui:wave.random]] | A new random value between Min and Max with every message; the frequency is not used |
| [[ui:wave.constant]] | Max, every time; the frequency is not used |

### The scope {#scope}

Beside the fields, the scope draws the value as it is sent: the latest 300
points, scaled to fit. Below it are the waveform, frequency and rate, and the
last value sent. The scope is updated about 30 times a second however fast the
generator sends, so at high rates it shows a sample of the messages, not each
one.

If a send fails, the generator stops and the console says why.

## In the Inspector {#inspector}

With capture armed ([[ui:ins.arm]] in the [[ui:dock.inspector]]), OSC traffic
appears with the protocol `osc`:

| Source | What | Notes |
| --- | --- | --- |
| `osc-send` | Every message the sender, a library signal, an [[ui:exp.node.osc]] step or `signallab send osc` sends | A step that waits for a reply shows as `experiment` |
| `osc-monitor` | Every packet the monitor receives | A bundle is summarised by its first message and `+n more in bundle`; a malformed packet has the verdict `decode error: …` |
| `osc-gen` | The generator's messages | At most one every 100 ms is captured, with the verdict `sampled`; the next one after skipped messages adds how many were not drawn, as `sampled · +5 not shown` |

Each frame keeps the bytes it was built from. See [Inspector](../tools/inspector.md).

## Saving and reusing {#library}

- **Save as a signal.** [[ui:sig.saveNew]] under the buttons keeps the message —
  target, address and arguments — in the signal library, in a folder you choose.
  From then on the sender is tied to that signal: [[ui:sig.save]] (or
  <kbd>Ctrl</kbd>+<kbd>S</kbd> in the sender) updates it, [[ui:sig.saveAs]]
  makes a copy, and the chip beside them opens it in [[ui:nav.signals]]. Fire
  it later from [[ui:nav.signals]] or with <kbd>Ctrl</kbd>+<kbd>K</kbd> from any
  screen. See [Signals](../tools/signals.md).
- **Add to an experiment.** [[ui:common.toExperiment]] adds an
  [OSC message](../experiments/nodes.md#node-osc) step with the same target,
  address and arguments to the open experiment — right before End, or after the
  selected step — and opens it. While the experiment runs, nothing can be added;
  the console says so.

## In experiments {#experiments}

| Step | What it does |
| --- | --- |
| [[ui:exp.node.osc]] | Sends one message. Its target, address and text arguments take `{{templates}}`. With [[ui:exp.expectReply]] it sends from a port of its own and waits there for the answer in the same step. [Details](../experiments/nodes.md#node-osc) |
| [[ui:exp.node.wait_osc]] | Waits for a message whose address matches a pattern and whose arguments pass the rules. [Details](../experiments/nodes.md#node-wait_osc) |
| [[ui:exp.node.emulator]] | An OSC device that answers by rules for the whole run. See [Emulators](../tools/emulators.md). |

The OSC message step takes `IP:port` or `host:port` like the screen, and its
address must start with `/`. A host name is looked up each time the step sends.

### Address patterns {#patterns}

[[ui:exp.node.wait_osc]], the reply of [[ui:exp.node.osc]] and the rules of an
OSC emulator match addresses with OSC 1.0 patterns:

| Pattern | Matches |
| --- | --- |
| `*` | any run of characters, also none |
| `?` | exactly one character |
| `[0-9]`, `[a-c]` | one character from the set or range |
| `[!0-9]` | one character not in the set |
| `{ping,pong}` | one of the words |

Wildcards stay within one part between slashes: `/cue/*` matches `/cue/7` but
not `/cue/7/go`, and a pattern matches only an address with the same number of
parts. Matching is case-sensitive. A pattern starts with `/`, has no empty part
(`//`), no spaces, no `#` and no characters outside ASCII, and is at most 512
characters.

Argument rules compare argument number 0–63 with a value (equals, less than,
contains, matches a regular expression and so on); a wait has at most 16. When
a bundle arrives, the wait takes it if any message in it matches. See
[Data and templates](../experiments/data.md) for what a matched message gives
the steps after it.

## From the command line {#cli}

`signallab send osc` sends one message as the sender does:

```bash
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main
```

```text
✔ sent /cue/go (24 bytes) → 127.0.0.1:9000
```

Each argument is `tag:value`: `i:3`, `f:0.5`, `d:1.5`, `h:64`, `s:text`,
`b:de ad be ef` (hex bytes), or `T`, `F`, `N` on their own. Without a tag, a
whole number is `i`, a number with a decimal point is `f`, and anything else is
`s`; write `s:7` to send the text `7`. The target is `IP:port` or `host:port`. It
exits with 0 when the message went out, 1 when sending failed (a host name that
does not resolve included), and 2 when an argument, the address or the target
is invalid. [`signallab fire`](../automation/cli.md#cli-fire) sends a saved signal. See
[Command line](../automation/cli.md#cli-send-osc).

::: tip
In Git Bash on Windows, an argument that starts with `/` is turned into a file
path before `signallab` sees it. Run the command with `MSYS_NO_PATHCONV=1` in
front, or use PowerShell or `cmd`.
:::

## Problems {#troubleshooting}

| What you see | Usual cause |
| --- | --- |
| `… is not a valid address` on send | The target has no port, or is neither `IP:port` nor `host:port`. |
| `Cannot resolve …` on send | The host name does not resolve on this machine. Check it, or use the IP address. |
| `OSC addresses start with / (…)` | The address has no leading `/`. |
| The message is sent but the device does nothing | Wrong port or address; a different type than it expects (`int` instead of `float`, a text `"true"` instead of a bool). Watch it in the Inspector, or point the target at the monitor on this machine to see what goes out. |
| `… is already in use by another program` on [[ui:osc.listen]] | Another program — or a running experiment, emulator or a second monitor — has the port. |
| `… is not an address of this computer` | The [[ui:common.bind]] IP belongs to another machine. Use `0.0.0.0` or one of this machine's addresses. |
| Packets from other machines never arrive | On Windows, the firewall can keep them out: allow Signal Lab when the app offers it. Local traffic (`127.0.0.1`) is not affected. See [Troubleshooting](../reference/troubleshooting.md). |
| [[ui:osc.decodeError]] rows | The sender is not speaking OSC 1.0 on that port, or uses a type tag Signal Lab does not decode. |

On a server, the screen works on the server's network: `127.0.0.1` is the
server itself, and the monitor listens on the server's ports. See
[Server](../server/index.md).

Every error message is listed in [Error messages](../reference/errors.md#transport).

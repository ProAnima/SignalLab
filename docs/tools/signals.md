---
title: Signals
description: Keep OSC, UDP, HTTP and MQTT messages in a library of folders, and send them again from the Signals screen, from any screen with Ctrl+K, or from the command line.
---

# Signals

A signal is a message you have named and kept: an OSC message, a raw UDP
datagram, an HTTP request or an MQTT publish, with its target. You build it
once — on the [[ui:nav.signals]] screen, or by saving what you just sent from
another screen — and send it again whenever you need it, byte for byte the
same.

The library is a JSON file you can read, edit by hand, copy to another machine
or commit next to a project. The [[ui:nav.signals]] screen shows it as a tree of
folders on the left and the selected signal's fields on the right.

## What a signal can send {#transports}

Pick the kind in [[ui:sig.transport]]. Each kind has its own fields:

| [[ui:sig.transport]] | Fields | What goes out |
| --- | --- | --- |
| [[ui:sig.tr.osc]] | [[ui:common.target]], [[ui:common.address]], [[ui:common.arguments]] | One OSC message to one `IP:port` or `host:port`, from a fresh UDP port. See [OSC](../protocols/osc.md#types) for the argument types. |
| [[ui:sig.tr.udp]] | [[ui:common.target]], [[ui:sig.payloadKind]] ([[ui:sig.payloadText]] or [[ui:sig.payloadHex]]), [[ui:sig.payload]] | One datagram with exactly these bytes. Text goes as written, without a terminator; hex is pairs of digits such as `de ad be ef` (spaces and a `0x` prefix are allowed). |
| [[ui:sig.tr.http]] | [[ui:sig.method]], [[ui:sig.url]], [[ui:sig.timeout]], [[ui:sig.headers]], [[ui:field.auth]], [[ui:sig.body]] | One HTTP request. See [HTTP](../protocols/http.md). |
| [[ui:sig.tr.mqtt]] | [[ui:mq.broker]], [[ui:mq.topic]], [[ui:mq.qos]], [[ui:mq.retain]], [[ui:sig.payload]] | One publish. See [MQTT](../protocols/mqtt.md). |

Every signal also has a [[ui:sig.name]], a [[ui:sig.group]] and a
[[ui:sig.note]] — what it should make happen and what has to match on the far
end.

The targets of OSC and UDP signals are `IP:port` or `host:port` (for example
`127.0.0.1:9000`); a host name is looked up each time the signal is sent. The
broker of an MQTT signal is `host:port`; without a port it is `1883`.

When you change the kind of a signal, its message starts again from that kind's
defaults. Only the target is kept, and only between [[ui:sig.tr.osc]] and
[[ui:sig.tr.udp]], where the target means the same thing.

## Sending a signal {#send}

To send a signal from the [[ui:nav.signals]] screen, do any of these:

- Select it and press [[ui:sig.fire]].
- Press <kbd>Ctrl</kbd>+<kbd>Enter</kbd> while you work in its fields.
- Double-click it in the tree.

Each send writes a line to the console: what went where, the bytes sent, or
for HTTP the status and the time it took. A failure (a refused connection, a
host that cannot be reached) is a red line with the reason. The time of the
last send shows beside the buttons.

A signal goes out through the same commands as the screens of its protocol, so
the [Inspector](inspector.md) lists it under the tool that sent it, and the far
end cannot tell it from one you typed.

How each kind is sent:

| Kind | How it goes out |
| --- | --- |
| [[ui:sig.tr.osc]] | As the [[ui:nav.osc]] screen sends a message. |
| [[ui:sig.tr.udp]] | One datagram to the target. |
| [[ui:sig.tr.http]] | As the [[ui:nav.http]] screen sends a request, with its cookie jar while [[ui:http.keepCookies]] is on there. A refused connection or a timeout counts as a failure, not as a status. |
| [[ui:sig.tr.mqtt]] | While the [[ui:nav.mqtt]] screen is connected to the signal's broker, on that connection, with its client id and credentials. Otherwise — not connected, or connected to another broker — Signal Lab connects to the signal's broker for this one publish, with a client id of its own, no user name and a clean session, then disconnects. |

A connection is to the signal's broker when the host is the same, ignoring
case, and the port is the same, `1883` standing for a broker written without a
port. Names are not looked up: `localhost` and `127.0.0.1` are two different
brokers here, so a signal that names one is not sent over a connection made to
the other.

::: tip
An MQTT signal stores no password. To publish to a broker that asks for one,
connect to that broker on the [[ui:nav.mqtt]] screen first; the signal then
rides that connection.
:::

## Sending from any screen {#palette}

Press <kbd>Ctrl</kbd>+<kbd>K</kbd> on any screen to open the palette, type a few
letters of a signal's name, folder, target or message, and press
<kbd>Enter</kbd>. The palette closes and the signal is sent; you stay on the
screen you were watching.

| Key | What it does |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>K</kbd> | Opens the palette, or closes it. |
| <kbd>↑</kbd> <kbd>↓</kbd> | Moves the choice. |
| <kbd>Enter</kbd> | Sends the chosen signal. |
| <kbd>Esc</kbd> | Closes the palette without sending. |

The palette lists at most 12 signals: the first 12 of the library while you
have typed nothing, then the first 12 that match. A click on a row sends it;
a click outside closes the palette.

## Making signals {#create}

### On the Signals screen {#create-here}

1. Pick the folder the signal belongs in (see [current folder](#current-folder)).
2. Press [[ui:sig.new]]. A new OSC signal to `127.0.0.1:9000`, address
   `/hello`, appears in that folder, selected.
3. Change [[ui:sig.name]], [[ui:sig.transport]] and the fields of the message.

Every change is saved on its own; there is no save button on this screen. While
the library file cannot be read, nothing is saved and the fields are read only
(see [The library file](#file)).

[[ui:sig.duplicate]] puts a copy right after the selected signal, its name
followed by `·`. [[ui:sig.delete]] asks once more ([[ui:sig.confirmDelete]]):
the second click removes it from the file. Its folder stays, even when it is
now empty.

### From the HTTP, OSC and MQTT screens {#save-from-screens}

The sending part of three screens — [[ui:http.request]] on [[ui:nav.http]],
[[ui:osc.sender]] on [[ui:nav.osc]], [[ui:mq.publish]] on [[ui:nav.mqtt]] — can
keep what it would send as a signal.

1. Set up the message and send it until it does what you want.
2. Press [[ui:sig.saveNew]] (or <kbd>Ctrl</kbd>+<kbd>S</kbd> in that part of the
   screen). The [[ui:sig.saveTitle]] dialog opens.
3. Check [[ui:sig.name]]: it is suggested from what is being sent.
4. Pick or type a [[ui:sig.group]]. The folder you used last time is filled in;
   a path such as `Venue/Stage` that does not exist yet is made.
5. Press [[ui:sig.saveConfirm]].

From then on the screen is tied to that signal. A chip beside the buttons says
where it lives (`❖ Folder / Name`); click it to see the signal on the
[[ui:nav.signals]] screen.

| You see | It means | What you can do |
| --- | --- | --- |
| ✓ [[ui:sig.savedState]] (greyed) | The library holds exactly what the screen would send. | Nothing to save. |
| [[ui:sig.save]], and [[ui:sig.changed]] on the chip | The screen's message differs from the signal. | [[ui:sig.save]] or <kbd>Ctrl</kbd>+<kbd>S</kbd> writes the screen's message into that signal; its name, folder and note stay. |
| [[ui:sig.saveAs]] | — | Opens the dialog again, filled with the signal's name and folder, and saves a new signal. The screen is then tied to the new one. |

The comparison looks at the message, not at how it is written: the order of
JSON keys and the last digits of an OSC float beyond 32-bit precision do not
count as a change.

The [[ui:nav.http]] and [[ui:nav.osc]] screens keep the tie when Signal Lab
restarts; the [[ui:nav.mqtt]] screen keeps it until you close the app.

::: warning
An HTTP signal keeps its [[ui:field.auth]] — user name and password, or token —
in the library file as plain text. Anyone who can read the file can read them.
:::

### Opening a signal in its screen {#open-in-screen}

A selected HTTP, OSC or MQTT signal has a button that opens it in the screen of
its protocol ([[ui:nav.http]], [[ui:nav.osc]] or [[ui:nav.mqtt]]). The screen's
fields are filled from the signal and the screen is tied to it, as above: edit
there, send, then [[ui:sig.save]]. A raw UDP signal has no screen of its own.

### From a frame or a topic {#capture}

- In the [Inspector](inspector.md#save-as-signal), select a frame and press
  [[ui:sig.fromFrame]]. A datagram becomes a raw UDP signal with that frame's
  exact bytes; an MQTT publish becomes an MQTT signal with the same broker,
  topic, QoS, retain flag and payload. Other frames cannot be saved.
- On the [[ui:nav.mqtt]] screen, select a topic and press [[ui:sig.fromFrame]].
  You get an MQTT signal that publishes the topic's last value, with its QoS
  and retain flag, to the broker you are connected to.

Both go into the folder [[ui:sig.capturedFolder]] and are named after what was
captured.

### In an experiment {#in-experiments}

When you add a node to an experiment, the [[ui:exp.addNode]] menu also lists
your signals under [[ui:exp.group.signals]]. Picking one adds an OSC, HTTP,
MQTT or UDP node with the same message. A raw UDP signal with a hex payload is
not offered: the UDP node sends text. See [Nodes](../experiments/nodes.md).

## Folders {#folders}

A folder is a path of names joined by `/`: `API/Auth` is the folder `Auth`
inside `API`. A signal's [[ui:sig.group]] field holds the path of its folder;
empty means the top level ([[ui:sig.topLevel]]). Names are trimmed and empty
parts dropped when you leave the field, so ` API / Auth/ ` becomes `API/Auth`.

Folders are sorted by name, numbers in numeric order (`Cue 2` before
`Cue 10`); signals stay in the order of the file. Each folder shows how many
signals it holds, its sub-folders included. An empty folder is kept until you
remove it.

### The current folder {#current-folder}

The folder you clicked last, or the folder of the signal you selected, is the
current one: [[ui:sig.new]] and [[ui:sig.newFolder]] put things there. A chip
above the tree names it; click the chip to go back to the top level.

### Working with folders {#folder-tasks}

| To | Do this |
| --- | --- |
| Make a folder | Press ＋ [[ui:sig.newFolder]]. It is made inside the current folder, named [[ui:sig.newFolderName]] (with a number after it when that name is taken), and you rename it straight away. |
| Open or close a folder | Click it, or press <kbd>→</kbd> / <kbd>←</kbd> while it has focus. Signal Lab remembers which folders are closed. |
| Open or close them all | The ⊞ and ⊟ buttons above the tree ([[ui:sig.expandAll]], [[ui:sig.collapseAll]]). |
| Rename a folder | Press ✎ ([[ui:sig.renameFolder]]) or <kbd>F2</kbd> on it, type, then <kbd>Enter</kbd>; <kbd>Esc</kbd> cancels. |
| Move a signal or a folder | Drag it onto a folder, or onto empty space in the tree for the top level. |
| Move a signal by typing | Change its [[ui:sig.group]] field. |
| Remove a folder | Press × ([[ui:sig.removeFolder]]) and then [[ui:sig.confirmRemoveFolder]], or press <kbd>Delete</kbd> twice on it. |

A rename never merges two folders: a name with `/` in it, or one a sibling
folder already has, is refused, and the console says so. A folder cannot be
dragged into itself or into a folder inside it. Dragging a folder into a
folder that already holds one of the same name merges the two.

Removing a folder removes the folder only: its signals and sub-folders move up
one level. Nothing is deleted.

### Finding a signal {#filter}

Type in [[ui:sig.search]] above the tree. It matches the name, the folder, the
note, the target and the message. While you filter, every folder with a match
is open and the others are hidden.

## The starter set {#starter-set}

The first time Signal Lab finds no library file, it writes nine examples, each
about something that is easy to get wrong. Their names and notes are written in
the language of the interface at that moment; after that they are yours to
change. All of them point at this computer.

| Folder | Signal | Sends |
| --- | --- | --- |
| `OSC` | [[ui:seed.osc-fader.name]] | `/fader/1` with the float `0.75` to `127.0.0.1:9000` |
| `OSC` | [[ui:seed.osc-types.name]] | `/types` with int `-7`, float `1.5`, string `hi`, bool true, int64 `4294967296`, double `0.125` and nil |
| `OSC` | [[ui:seed.osc-id-and-value.name]] | `/tag` with the strings `reader-1` and `04a1b2c3` |
| `OSC` | [[ui:seed.osc-trigger.name]] | `/cue/go` with no arguments |
| `MQTT` | [[ui:seed.mqtt-publish.name]] | `1` to `lab/example/value` on `127.0.0.1:1883`, QoS 0 |
| `MQTT` | [[ui:seed.mqtt-retained.name]] | `night` to `lab/example/config`, QoS 1, retained |
| `MQTT` | [[ui:seed.mqtt-clear-retained.name]] | An empty retained payload to `lab/example/config`, QoS 1 |
| `HTTP` | [[ui:seed.http-reachable.name]] | `GET http://127.0.0.1:8080/`, timeout 4000 ms |
| [[ui:seed.folder.Raw]] | [[ui:seed.udp-raw.name]] | The bytes `de ad be ef` to `127.0.0.1:9000` |

To get the starter set back, move or rename `signals.json` and press
[[ui:sig.reload]]: with no file there, it is written again.

## The library file {#file}

The library is `signals.json` in the data folder: `Documents/SignalLab` in your
home folder on a desktop, or the server's data folder (see
[Files](../reference/files.md)). Hover the signal count under the tree to see
the full path.

- **Saved on its own.** Every change is written 0.7 s after the last one, the
  whole file at once, through a temporary file in the same folder that then
  takes the file's place — a write cut short leaves the previous file. While a
  write waits, the foot of the tree says [[ui:sig.saving]]; then
  [[ui:sig.saved]]. A write still waiting is made before an update restarts the
  app.
- **Edited by hand.** Signal Lab does not notice when the file changes under
  it. After editing it, or replacing it with one from another machine, press
  [[ui:sig.reload]]. Reloading reads the file again and drops a change that was
  still waiting to be written.
- **Never replaced while it is broken.** If the file is not valid JSON, or not
  a signal library, the tree shows the error with the file's path, line and
  column, and the console says the same. The file is left as it is, and
  nothing writes the library until it reads again: [[ui:sig.new]], renaming,
  moving and removing signals and folders, dragging, the fields of a signal,
  [[ui:sig.saveNew]], [[ui:sig.save]] and [[ui:sig.saveAs]] on the HTTP, OSC
  and MQTT screens, and [[ui:sig.fromFrame]] in the Inspector and on the MQTT
  screen are all off, and their tip says what is wrong. Fix the file, or
  remove it, and press [[ui:sig.reload]]: once it reads, everything works
  again.
- **A file that breaks while the app runs.** If you edit the file into
  something unreadable and the app then saves a change, the save is refused
  with the same error, the file is left as you made it, and the app stops
  writing until you fix it and press [[ui:sig.reload]]. A file you edited and
  left valid is replaced by the app's list at its next save, as above: reload
  first.

A short example of the file:

```json
{
  "version": 2,
  "signals": [
    {
      "id": "fader-value",
      "name": "Fader value",
      "group": "Venue/Stage",
      "note": "Main fader of desk A.",
      "body": {
        "transport": "osc",
        "target": "127.0.0.1:9000",
        "address": "/fader/1",
        "args": [{ "type": "float", "value": 0.75 }]
      }
    }
  ],
  "folders": ["Venue/Stage", "Venue/Empty for now"]
}
```

| Key | What |
| --- | --- |
| `version` | `2`. A version 1 file (before folders) reads the same, without empty folders. |
| `signals[].id` | Made from the name when the signal is made (`fader-value`, `fader-value-2`, …) and never changed by a rename. `signallab fire` finds a signal by it. |
| `signals[].group` | The folder path; `""` is the top level. |
| `signals[].body` | The message. `transport` is `osc`, `udp`, `http` or `mqtt`; the other keys are that kind's fields. |
| `folders` | Every folder, so an empty one is kept. Left out when there are none. A `group` no entry lists is a folder too. |

## From the command line {#cli}

`signallab fire` sends a signal of a library, through the same commands as the
app:

```bash
signallab fire "Fader value"
signallab fire fader-value --library ./show/signals.json
```

It finds the signal by its id first, then by its name, ignoring case. When
several signals have that name, it names their ids and sends nothing. Without
`--library` it reads the app's own `signals.json`; it never writes the file.
See [The command line](../automation/cli.md#cli-fire).

## Related {#related}

- [Inspector](inspector.md) — watch what a signal sends, and keep a captured
  frame as a signal.
- [Keyboard shortcuts](../reference/shortcuts.md)
- [Files](../reference/files.md) — where the data folder is.

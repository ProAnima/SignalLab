---
title: The editor
description: Build, edit and run experiments on the node canvas — adding and wiring nodes, selecting and copying, the properties pane, Send now, validation, running and the files behind it.
---

# The experiment editor

An experiment is a test written as a graph: nodes that send (an HTTP request,
an OSC message, an MQTT publish…), wait for an answer, check what came back,
play the other side or break the network on cue, joined by wires that say
what runs next. A run starts at [[ui:exp.node.start]], follows the wires and passes when it
reaches [[ui:exp.node.end]] with every step passed. You build it on the [[ui:nav.experiment]]
screen, run it there, and run the same file from the command line or a server.

The editor holds one experiment at a time and saves it as you work. To keep
several, export snapshots or open another from a file (see
[Saving and files](#files)).

Every kind of node, its fields and outputs are in the [node reference](nodes.md).
How values flow between nodes is [Data and templates](data.md); parallel
branches, loops, retries and repeats are [Flow](flow.md).

## The screen {#screen}

| Area | What it holds |
| --- | --- |
| Toolbar (top) | The experiment's name and save state, adding nodes, parameters, profiles, the panes, focus mode, fullscreen and Run |
| Canvas bar | Undo and redo, the node finder, [[ui:exp.arrange]] and zoom |
| Canvas | The graph: nodes, wires, and the run's progress drawn on them |
| [[ui:exp.properties]] (right) | The selected node's fields, its preview and [[ui:exp.sendNow]]; or what several selected nodes or a selected wire allow |
| [[ui:exp.timeline]] (bottom) | The steps of the current or last run, its outcome, report and seed |

The properties pane and the timeline have a handle on their inner edge: drag
it, or focus it with <kbd>Tab</kbd> and use the arrow keys (<kbd>Shift</kbd>
for bigger steps); a double click or <kbd>Enter</kbd> gives the pane its
default size back. Sizes are kept for the next time. The
timeline stays folded until the first run opens it.

## The toolbar {#toolbar}

| Control | What it does |
| --- | --- |
| ☰ [[ui:exp.documents]] | Opens the templates, opening a JSON file and exporting ([Saving and files](#files)) |
| [[ui:exp.name]] | The experiment's name; a run needs one. Run reports record it, and [[ui:exp.compare]] finds earlier runs by it |
| [[ui:exp.saved]] / [[ui:exp.saving]] / [[ui:exp.saveError]] | Whether the last change is on disk |
| ⚠ [[ui:exp.needsLinks]] | Shown while the experiment cannot run; its tooltip says why, a click selects the node it is about ([Validation](#validation)) |
| ＋ [[ui:exp.addNode]] | Opens the add menu, after the selected node when there is one (<kbd>A</kbd>) |
| [[ui:exp.profile]] | Which profile runs, the preview and Send now use; shown when the experiment has profiles. ⚠ marks a profile that would not run |
| `{ }` [[ui:exp.params]] | Parameters, profiles, the seed, cookies and secrets ([Data and templates](data.md)) |
| ☷ [[ui:exp.properties]] | Shows or hides the properties pane |
| ▢ [[ui:exp.focus]] | Hides the sidebar, the header and the bottom panel ([Focus mode and fullscreen](#focus)) |
| ⛶ [[ui:exp.fullscreen]] | The window in fullscreen, with focus mode |
| [[ui:exp.run]] | Checks, saves and runs the experiment; while it runs, the button is [[ui:common.stop]] |
| ▾ [[ui:exp.runWith]] | One run with another profile, other parameter values or a given seed, without changing the experiment |

## Moving around the canvas {#canvas}

- **Pan**: drag the empty canvas, or scroll.
- **Zoom**: <kbd>Ctrl</kbd> and the mouse wheel zoom around the pointer;
  − and ＋ in the canvas bar step by 10 %. Zoom goes from 15 % to 200 %; the
  percentage between the buttons shows where you are.
- **1:1** ([[ui:exp.resetZoom]], <kbd>Ctrl</kbd>+<kbd>1</kbd>) goes back to 100 %.
- ⊡ [[ui:exp.fit]] (<kbd>Ctrl</kbd>+<kbd>0</kbd>) shows the whole graph, at
  100 % at most.
- **Find a node**: [[ui:exp.nodes]] in the canvas bar (it shows how many nodes
  there are) or <kbd>Ctrl</kbd>+<kbd>F</kbd>. Type words from the node's name,
  its summary (a URL, an address, a topic) or its id; <kbd>↑</kbd>
  <kbd>↓</kbd> choose, <kbd>Enter</kbd> selects the node and brings it into
  view (zoomed to at least 80 %), <kbd>Esc</kbd> closes.

There is no minimap: [[ui:exp.fit]] and the finder do its work.

## Nodes and wires {#nodes-and-wires}

A node shows its kind, a one-line summary of what it does, and badges for its
settings: ↻ and a number for Retry, × and a count (or a time) for Repeat, ⚡
for Load. Its input is on the left — every node but [[ui:exp.node.start]] has one — and its
outputs on the right:

| Output | On | Followed when |
| --- | --- | --- |
| [[ui:exp.outputPort]] | Most nodes | The step passed |
| [[ui:exp.yes]] / [[ui:exp.no]] | [[ui:exp.node.branch_status]], [[ui:exp.node.branch_value]] | The comparison held / did not |
| [[ui:exp.branch1]] / [[ui:exp.branch2]] | [[ui:exp.node.fork]] | Always, both at once |
| [[ui:exp.portMatched]] / [[ui:exp.portTimeout]] | Waits | A matching message arrived / none did in time |
| [[ui:exp.portBody]] / [[ui:exp.portDone]] / [[ui:exp.portLimit]] | [[ui:exp.node.loop]] | Another iteration / the loop is over / it ran out of iterations |

**An output may have several wires.** The first wire continues the branch;
each further wire starts a parallel branch with a copy of the variables known
at that point. A [[[ui:exp.node.join]]](nodes.md#node-join) node waits for every wire that
leads into it. Details are in [Flow](flow.md).

[[ui:exp.portTimeout]] and [[ui:exp.portLimit]] are optional: left unwired, a
timeout or running out of iterations fails the step. Every other output must
have a wire before the experiment can run.

The wire from the body of a [[ui:exp.node.loop]] back to it is drawn as an arc
over the body; it is the only wire that may lead backwards.

## Adding nodes {#adding}

### The add menu {#add-menu}

The add menu lists every kind of node by group — [[ui:exp.group.action]],
[[ui:exp.group.observe]], [[ui:exp.group.emulate]], [[ui:exp.group.fault]],
[[ui:exp.group.data]], [[ui:exp.group.check]], [[ui:exp.group.flow]] — and
then [[ui:exp.group.signals]]: the signals of your [library](../tools/signals.md)
that can become a node (OSC, HTTP, UDP with a text payload, MQTT), with their
fields filled in.

Its search field has the focus as it opens. Type a few letters: every word
must appear in the node's name, its description or its type (`http`,
`wait_osc`); a signal is found by its name, folder, transport or target.
<kbd>↑</kbd> <kbd>↓</kbd> choose, <kbd>Enter</kbd> adds, <kbd>Esc</kbd>
closes. The line at the top says where the node goes: after a node, or
parallel to one.

A new node is selected, the properties pane opens, and its main field — the
URL, the address, the topic, the delay — has the focus with its text
selected, so you can type straight away. <kbd>Esc</kbd> in a field takes you
back to the node on the canvas, ready for the next <kbd>A</kbd>.

### After the selected node {#add-after}

<kbd>A</kbd>, ＋ [[ui:exp.addNode]] in the toolbar and ＋ [[ui:exp.addNext]]
in the properties pane add the next step after the selected node:

- It attaches to the node's first output that has no wire yet (the free
  [[ui:exp.no]] of a [[ui:exp.node.branch_status]], say), or else to its first
  output.
- If that output already has a wire, the new node is spliced into it (into the
  first, if it has several): the wire now runs through the new node, and
  everything after it moves right to make room.
- With [[ui:exp.node.end]] selected, the node goes before it, when one wire
  leads into it.
- With nothing selected, the node lands in the middle of the view, without
  wires.

A node with a single output that is put on the empty [[ui:exp.portBody]] of a
[[ui:exp.node.loop]] — this way or [on a new wire](#add-branch) — is also wired
back to it, so the body is complete at once.

### Into a wire {#insert}

Every wire has a ＋ in its middle ([[ui:exp.insertNode]]). It opens the add
menu, and the node you pick is spliced into that wire. A [[ui:exp.node.loop]] spliced in
this way continues the flow through its [[ui:exp.portDone]].

### On a new wire {#add-branch}

Drag a wire out of an output and let go on the empty canvas: the add menu
opens there, and the new node goes on a new wire of that output — beside the
wires it already has, so it runs in parallel with them. The same happens when
you click an output and then click the empty canvas, or double-click it.

### Anywhere {#add-anywhere}

Double-click the empty canvas to add a node at that spot, without wires.

### From other screens {#from-screens}

- [[ui:common.toExperiment]] on the [OSC](../protocols/osc.md) and
  [HTTP](../protocols/http.md) screens adds what you just tried as the next
  step — just before [[ui:exp.node.end]], when one wire leads into it — and
  switches to the editor.
- [[ui:osc.waitForThis]] next to a message in the OSC monitor adds a
  [[ui:exp.node.wait_osc]] that recognises it: its address and its text, whole-number and
  true/false arguments (floats are measurements that change, so they are left
  out). Stop the monitor before running: the run listens on that port itself.
- [[ui:mq.waitForThis]] on a topic of the [MQTT](../protocols/mqtt.md) tree
  adds a [[ui:exp.node.wait_mqtt]] on that topic, at that broker.

While a run is going, nodes cannot be added; the console says so.

## Connecting nodes {#connecting}

- **Drag** from an output onto a node. Letting go within 24 pixels of a node
  is enough.
- **Click** an output (or focus it and press <kbd>Enter</kbd> or
  <kbd>Space</kbd>): the canvas bar says [[ui:exp.chooseInput]]. Click a node
  or its input to connect; click the empty canvas to add a node there on a new
  wire; [[ui:exp.cancelLink]] or <kbd>Esc</kbd> gives up.

A wire is refused when it would create a cycle (other than the way back of a
[[ui:exp.node.loop]]), when it leads into [[ui:exp.node.start]] or out of
[[ui:exp.node.end]], or when it would connect a node to itself; the editor says
so. Drawing a wire that exists already changes nothing.

To **remove a wire**, click it — the properties pane shows what it connects —
and press <kbd>Delete</kbd>, or use [[ui:exp.removeWire]] there. Hovering a
wire also shows a × above its ＋. The properties of a node list its outgoing
wires, each with × ([[ui:exp.disconnect]]).

## Selecting several nodes {#selection}

| To | Do |
| --- | --- |
| Select a node | Click it, or move to it with <kbd>Tab</kbd> |
| Add a node to the selection, or take it out | <kbd>Shift</kbd> or <kbd>Ctrl</kbd> and a click |
| Select with a frame | <kbd>Shift</kbd> and drag on the empty canvas: every node the frame touches joins the selection |
| Select every node | <kbd>Ctrl</kbd>+<kbd>A</kbd> |
| Clear the selection | Click the empty canvas; <kbd>Esc</kbd> when several are selected |
| Move them | Drag one of them: they all move |
| Nudge them | With a node focused, the arrow keys move the selection 5 pixels, 20 with <kbd>Shift</kbd> |

The properties pane shows the last node picked; with several selected, it
shows how many, with [[ui:exp.copy]], [[ui:exp.duplicate]] and
[[ui:exp.deleteSelected]].

### Copy, cut and paste {#copy-paste}

<kbd>Ctrl</kbd>+<kbd>C</kbd> copies the selected nodes and the wires between
them as text; <kbd>Ctrl</kbd>+<kbd>X</kbd> also removes them;
<kbd>Ctrl</kbd>+<kbd>V</kbd> pastes them — into this experiment, into another
one opened later, or into another Signal Lab window. The text is JSON, so you
can also keep it in a file or a message.

- [[ui:exp.node.start]] and [[ui:exp.node.end]] are one of a kind: they are never
  copied.
- Wires between a copied node and the rest of the graph are not copied; wire
  the copy in yourself.
- Every pasted node gets a new id.
- A node that names another one — [[ui:exp.node.impairment_change]],
  [[ui:exp.node.emulator_state]], [[ui:exp.node.ws_send]],
  [[ui:exp.node.wait_ws]], [[ui:exp.node.ws_close]] — names the copy when that was copied too;
  otherwise it keeps naming the original if it is in this experiment, or the
  first node of that kind here.
- A pasted [[ui:exp.node.emulator]] or [[ui:exp.node.impairment]] whose port
  this experiment already listens on moves to the next free port. What sends to
  the original still does.
- The copy lands 32 pixels right of and one row below where it came from,
  further down until it covers no node, and is selected.

<kbd>Ctrl</kbd>+<kbd>D</kbd> ([[ui:exp.duplicate]]) does the same without the
clipboard.

### Deleting {#deleting}

<kbd>Delete</kbd> or <kbd>Backspace</kbd> ([[ui:exp.delete]],
[[ui:exp.deleteSelected]]) removes the selected nodes and their wires. A node
that had exactly one wire in and one wire out leaves a wire in its place, from
the node before it to the node after it, so a chain stays connected. [[ui:exp.node.start]]
and [[ui:exp.node.end]] cannot be deleted.

## Undo and redo {#undo}

<kbd>Ctrl</kbd>+<kbd>Z</kbd> undoes; <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd>
or <kbd>Ctrl</kbd>+<kbd>Y</kbd> redoes (↶ ↷ in the canvas bar). A drag, a
series of arrow-key nudges or the typing in one field is one step. The last
100 steps are kept while the app is open, across screen switches; opening
another experiment is one step too, so <kbd>Ctrl</kbd>+<kbd>Z</kbd> brings
the previous one back. Undo and redo wait while a run is going.

## Arrange {#arrange}

[[ui:exp.arrange]] lays the graph out from left to right: each node in the
column after the last node that leads to it, the body of a [[ui:exp.node.loop]]
in its row — and then fits the view. It is one step of the history. A draft
with a cycle that is not a loop's is left as it is.

## The properties pane {#properties}

With one node selected, the pane shows, from top to bottom:

1. The node's name (its tooltip says what it does) and, when the experiment
   cannot run because of this node, what is wrong.
2. Its fields. Fields that take [templates](data.md) suggest parameters,
   variables, secrets and generators as you type `{{`, or on
   <kbd>Ctrl</kbd>+<kbd>Space</kbd>.
3. Its settings: [[ui:exp.loadOn]] on an HTTP request, [[ui:exp.repeatOn]] on
   nodes that send, [[ui:exp.retryOn]] on nodes that send or listen, and
   [[ui:exp.expectReply]] on OSC and UDP messages (see [Node
   settings](nodes.md#settings)). Load replaces Repeat and Retry while it is on.
4. The last run's load result, on an HTTP node that ran under load.
5. The preview — [[ui:exp.preview]], [[ui:exp.previewWait]] or
   [[ui:exp.previewCheck]] — when the node has templates: what it would send,
   wait for or compare, resolved with the current parameters and the values
   known so far ([Send now and the preview](#send-now)).
6. [[ui:exp.sendNow]] or [[ui:exp.listenNow]], and what it did last.
7. Its outgoing wires, each with ×.
8. ⚡ [[ui:exp.routeThrough]] on an OSC or UDP message: an [[ui:exp.node.impairment]] is put
   in front of the node — listening on a free loopback port from 9010 up,
   forwarding to the node's target — and the node is pointed at it, so the
   next run degrades what it sends ([Faults](faults.md)).
9. ＋ [[ui:exp.addNext]], [[ui:exp.copy]], [[ui:exp.duplicate]] and
   [[ui:exp.delete]].

A double click on a node opens the pane with the cursor in its main field.
With several nodes selected the pane offers what can be done to all of them;
with a wire selected, what it connects and [[ui:exp.removeWire]]. While a run
is going, the fields are locked.

## Send now and the preview {#send-now}

[[ui:exp.sendNow]] (<kbd>Ctrl</kbd>+<kbd>Enter</kbd>, also from inside the
node's fields) sends the selected node on its own, without running the
experiment, through the same code a run uses. It is offered on
[[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]],
[[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_connect]] and
[[ui:exp.node.ws_send]].
On a wait it is [[ui:exp.listenNow]]: the wait listens from now until a message
matches or its timeout ends.

- The node uses the active profile, the stored secrets and the variable values
  known so far — from the last run and from earlier [[ui:exp.sendNow]] results.
  If a template names a value nobody has set yet, nothing is sent and the names
  are listed.
- It is sent once: Retry, Repeat and Load do not apply, no cookies are kept,
  and emulators and relays of the experiment are not started.
- A [[ui:exp.node.ws_send]] or [[ui:exp.node.wait_ws]] opens the connection its
  [[ui:exp.node.ws_connect]] describes, for that one test.
- On an HTTP request the response is shown — status, time, size, and the body,
  JSON formatted. The values the [[ui:exp.node.extract]] nodes after the
  request would take are filled in at once. Click a value in a JSON response to
  add an [[ui:exp.node.extract]] node for it right after the request, its
  variable named and its value known.
  [[ui:http.mockThis]] turns the response into a route of an
  [emulator](../tools/emulators.md).
- The result is also written to the console.

The preview is resolved by the engine as well, about a quarter of a second
after a change. Secret values are never shown, there or anywhere.

## Validation {#validation}

The editor checks the experiment as you edit, and once more when you press
[[ui:exp.run]]:

- An output that still needs a wire pulses amber.
- A node no wire from [[ui:exp.node.start]] reaches is drawn dashed; its tooltip says so.
- The node a problem is about is outlined, and the problem is written above its
  fields.
- ⚠ [[ui:exp.needsLinks]] in the toolbar names the problem; a click selects
  the node.

An experiment runs when, among other things:

- it has a name, exactly one [[ui:exp.node.start]] and one
  [[ui:exp.node.end]], and at most 64 nodes;
- every node is reachable from [[ui:exp.node.start]] and every required output has a wire;
- the only cycles are the bodies of [[ui:exp.node.loop]] nodes leading back to
  them;
- every check and [[ui:exp.node.extract]] has an HTTP request before it on every path (one
  under load does not count: it leaves no response);
- a [[ui:exp.node.ws_send]], [[ui:exp.node.wait_ws]] or [[ui:exp.node.ws_close]]
  comes after the [[ui:exp.node.ws_connect]] it uses;
- every field is filled in and in range, and every template names something
  known at that point.

Unfinished drafts are saved all the same. A run is refused, before any
traffic, when a secret it needs is not stored. Every message is listed in the
[error reference](../reference/errors.md).

## Running {#running}

[[ui:exp.run]] checks the experiment, saves it, and starts it. If it cannot
run, the problem is shown and its node selected. Otherwise the timeline opens
and nodes light up as the run reaches them: ● running, ✓ passed, ✕ failed,
↻ retrying, ⟳ repeating, ⚡ under load; wires that carried the flow are
coloured too.

- Waits, emulators and impairment relays open their ports before the first
  step, so nothing that arrives early is missed. A port that cannot be opened
  (another program has it, say) keeps the run from starting, and the node that
  needs it is shown.
- [[ui:common.stop]] ends the run at once: pauses, waits and loads included.
  The run is also a job in the console's job strip, which can stop it too.
- A run that takes longer than 300 seconds is stopped and fails.
- While a run is going the experiment cannot be edited; panning, zooming and
  selecting still work.

▾ [[ui:exp.runWith]] next to the button runs once with another profile, other
parameter values or a given seed; the experiment itself is not changed, and the
form keeps what you typed until the app closes ([Data and
templates](data.md)).

### The run timeline {#timeline}

The [[ui:exp.timeline]] lists one row per step: the time, the node and what
happened — passed, failed and why, a retry, a repeat's progress, a load's
numbers once a second. Click a row to select its node on the canvas.

Its title line holds:

- the outcome: [[ui:exp.passed]], [[ui:exp.failed]] with the reason, or
  [[ui:exp.stopped]];
- [[ui:exp.reportSaved]] — the run's report file (on a server, a download);
- [[ui:exp.compare]] — this run beside an earlier one of the same experiment;
- the profile and changed values the run used, when it had any;
- the run's seed, with [[ui:exp.pinSeed]] to keep it in the experiment so the
  next runs draw the same random values, or [[ui:exp.unpinSeed]] once it is
  pinned.

When the [Inspector](../tools/inspector.md) was capturing, a wait (or an
expected reply) that matched links to the frame it matched; a click opens it in
the Inspector. Reports, seeds and comparing runs are in [Runs and
reports](runs.md).

## Focus mode and fullscreen {#focus}

▢ [[ui:exp.focus]] hides the sidebar, the header and the bottom panel (console
and Inspector), leaving the screen to the editor. ⛶ [[ui:exp.fullscreen]] puts
the window in fullscreen and turns focus mode on; leaving fullscreen puts focus
mode back the way it was. Switching to another screen leaves focus mode.

<kbd>Esc</kbd> on the canvas, with nothing else to close, leaves fullscreen and
then focus mode.

## Saving and files {#files}

The experiment is saved by itself, a moment after each change, to
`experiment.json` in the data folder (`Documents/SignalLab` on a desktop; a
server has its own — see [Files and folders](../reference/files.md)). Drafts
that cannot run yet are saved too. A run saves first, so what ran is what is on
disk.

If that file cannot be read — edited by hand into broken JSON, say — the editor
says which file and why, and leaves it alone. Opening another experiment from
☰ [[ui:exp.documents]] then replaces it at the next save.

☰ [[ui:exp.documents]] opens the experiments dialog:

- [[ui:exp.templates]]: pick one and press [[ui:exp.openDocument]]. They all
  use loopback addresses.
- [[ui:exp.importJson]] reads an experiment file — written by this version of
  Signal Lab or an earlier one, up to 4 MiB — and shows its name and how many
  nodes and connections it has before you open it. Files from earlier versions
  are brought up to date as they open. A file that does not parse, or would not
  be a valid experiment, is refused with the reason, and the current experiment
  stays.
- [[ui:exp.exportJson]] writes a snapshot of the current experiment — nodes,
  wires, positions, parameters and profiles, never secret values — to a new
  file in `exports` in the data folder and shows its path; on a server, with a
  [[ui:common.download]] link.
- [[ui:exp.openDocument]] replaces the current experiment with the template or
  file. It does not run it, and <kbd>Ctrl</kbd>+<kbd>Z</kbd> brings the previous
  one back.

| Template | What it does |
| --- | --- |
| [[ui:exp.templateEmpty]] | [[ui:exp.node.start]] and [[ui:exp.node.end]], for your own flow |
| [[ui:exp.templateHttp]] | A GET to `http://127.0.0.1:8080/` and a check for status 200 — the experiment you start with |
| [[ui:exp.templateBranch]] | The same request; on 200 an OSC message to `127.0.0.1:9000`, otherwise a 500 ms delay |
| [[ui:exp.templateParallel]] | Two branches at once — a request and a log line — joined before [[ui:exp.node.end]] |
| [[ui:exp.templatePingReply]] | Sends `/ping` with the run's id to `127.0.0.1:9000` and waits on `127.0.0.1:9001` for `/pong` carrying it back |
| [[ui:exp.templatePoll]] | Asks a device for `/status` every 0.3 s until it answers `ready`, ten times at most |
| [[ui:exp.templateFlaky]] | An emulated API that fails twice before it answers, and a loop that asks until it does |
| [[ui:exp.templateFaults]] | Datagrams to an emulated device through an impairment relay while a parallel branch switches the network clean, lossy, offline and clean again |
| [[ui:exp.templateOutage]] | An emulated API taken down for two seconds by a parallel branch, and a client that keeps asking until it answers again |
| [[ui:exp.templateWsEcho]] | Connects to an echo service at `ws://127.0.0.1:9001/echo`, sends a JSON ping, expects it back unchanged, and closes |

The same files run without the editor: `signallab run experiment.json` — see
[The command line](../automation/cli.md).

## Keyboard shortcuts {#shortcuts}

Single keys act on the canvas and are left alone while you type in a field. On
a Mac, in a browser, <kbd>Cmd</kbd> works where <kbd>Ctrl</kbd> is written.

| Keys | Action |
| --- | --- |
| <kbd>A</kbd> | Add a node after the selected one, or mid-view when nothing is selected |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:exp.sendNow]], or [[ui:exp.listenNow]], on the selected node |
| <kbd>Ctrl</kbd>+<kbd>Z</kbd> | Undo |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd>, <kbd>Ctrl</kbd>+<kbd>Y</kbd> | Redo |
| <kbd>Ctrl</kbd>+<kbd>A</kbd> | Select every node |
| <kbd>Ctrl</kbd>+<kbd>C</kbd> / <kbd>Ctrl</kbd>+<kbd>X</kbd> / <kbd>Ctrl</kbd>+<kbd>V</kbd> | Copy / cut / paste the selected nodes and the wires between them |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | Duplicate the selected nodes |
| <kbd>Delete</kbd>, <kbd>Backspace</kbd> | Remove the selected wire, or the selected nodes |
| Arrow keys (a node focused) | Move the selected nodes by 5 pixels; with <kbd>Shift</kbd>, 20 |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | Find a node |
| <kbd>Ctrl</kbd>+<kbd>0</kbd> | Fit the graph |
| <kbd>Ctrl</kbd>+<kbd>1</kbd> | Zoom to 100 % |
| <kbd>Ctrl</kbd> + mouse wheel | Zoom around the pointer |
| <kbd>Shift</kbd> + click, <kbd>Ctrl</kbd> + click | Add a node to the selection, or take it out |
| <kbd>Shift</kbd> + drag on the empty canvas | Select with a frame |
| Double-click a node | Edit its fields |
| Double-click the empty canvas | Add a node there |
| <kbd>Enter</kbd>, <kbd>Space</kbd> on a focused output | Start a wire from it |
| `{{` or <kbd>Ctrl</kbd>+<kbd>Space</kbd> in a field | Suggest parameters, variables, secrets and generators |
| <kbd>Esc</kbd> in a field | Back to the node on the canvas |
| <kbd>Esc</kbd> on the canvas | Close the add menu; else cancel the wire being drawn; else let go of the selected wire; else clear a selection of several; else leave fullscreen; else leave focus mode |

In the add menu and the finder, <kbd>↑</kbd> <kbd>↓</kbd> choose,
<kbd>Enter</kbd> takes the choice and <kbd>Esc</kbd> closes. All of the app's
shortcuts are in [Keyboard shortcuts](../reference/shortcuts.md).

---
title: The window
description: Find your way around Signal Lab's window — the sidebar's screens, the header, the console and the Inspector, the signal palette, panes, tooltips and languages.
---

# The window

Signal Lab's window has four parts: the **sidebar** on the left lists the screens, the
**header** along the top holds what applies everywhere, the **screen** you chose fills
the middle, and the **bottom panel** shows the console, the running jobs and the
Inspector. The desktop app and a server's page in a browser look the same; the few
differences are in [In a browser](#browser).

## The sidebar {#sidebar}

Each screen is a tool of its own. Click one to open it:

| Screen | What it is for |
| --- | --- |
| [[ui:nav.experiment]] | Build test flows on a canvas, run them and read each run step by step. [Experiments](../experiments/index.md) |
| [[ui:nav.signals]] | The signal library: named messages in folders, to edit and send again. [Signals](../tools/signals.md) |
| [[ui:nav.emulators]] | Mock HTTP APIs, OSC, UDP and TCP devices and MQTT brokers that answer by rules. [Emulators](../tools/emulators.md) |
| [[ui:nav.osc]] | Send OSC messages, monitor a port, drive a waveform into an endpoint. [OSC](../protocols/osc.md) |
| [[ui:nav.mqtt]] | Connect to a broker, see every topic it holds, publish and clear retained values. [MQTT](../protocols/mqtt.md) |
| [[ui:nav.broadcast]] | Send to many hosts at once — a list, broadcast, multicast, a subnet sweep — and listen for who answers. [Broadcast and discovery](../protocols/broadcast.md) |
| [[ui:nav.http]] | One request and its whole response, then a load burst against the same endpoint. [HTTP](../protocols/http.md) |
| [[ui:nav.ws]] | Connect to a WebSocket service, send text or bytes, read every message. [WebSocket](../protocols/websocket.md) |
| [[ui:nav.netsim]] | A relay that degrades UDP or TCP traffic between a client and its target. [Impairment](../tools/impairment.md) |
| [[ui:nav.storm]] | Raw UDP or TCP load against your own servers and links. [Storm](../tools/storm.md) |
| [[ui:nav.scan]] | Which TCP ports of a host are open, with what the service says first. [Scanner](../tools/scanner.md) |

The sidebar starts as a narrow rail with each screen's symbol and a short name; hover one
to see its full name. The **☰** at the left of the header ([[ui:app.expandNav]] /
[[ui:app.collapseNav]]) switches between the rail and the full list, and Signal Lab
remembers which you chose.

A number on a screen's entry counts the jobs running from it, such as a monitor or an
emulator, so you can see what is still going without opening it. At the bottom of the
sidebar, the version opens About, and the line under it counts every running job.

Signal Lab opens on the screen you used last.

## The header {#header}

From left to right:

| Item | What it does |
| --- | --- |
| **☰** | Shows the sidebar as a rail or as a full list. |
| The host | [[ui:app.host]], then the name and network address of the computer the engine runs on: this one in the desktop app, the server in a browser. In a browser it also says [[ui:app.server]] — hover it to see where the server keeps its files — and the dot beside it changes when the page loses its connection. |
| The update button | Appears in the desktop app when a newer release has been found, and opens About to install it. See [Updates](install.md#updates). |
| The book | [[ui:app.docs]]: this documentation, at the page of the screen you are on. <kbd>F1</kbd> does the same from anywhere. |
| **✉** | [[ui:feedback.open]]: a message to the developers. See [Writing to the developers](#feedback). |
| **?** | [[ui:about.open]]: the version, who makes Signal Lab, how to reach them, and updates. |
| The flag | [[ui:app.language]]: the language of the interface. See [Languages](#languages). |
| [[ui:app.signOut]] | In a browser, when the server asks for an access token: ends this browser's session. |
| [[ui:app.stopAll]] | Stops every running job at once: monitors, generators, beacons, emulators, relays, storms, scans, runs. It is greyed out while nothing runs. |

### The documentation {#docs}

The documentation is built into the app and into the server, so it is there without an
internet connection, in the interface's language. The desktop app shows it in a window
of its own — pressing the button again from another screen turns that window to the
other screen's page — and sends links that leave the documentation to your browser. In
a browser it opens in a tab of its own. About has an [[ui:app.docs]] button too, which
opens [What is Signal Lab](index.md).

### About {#about}

About shows the version, the developer, the contact address (with a button that copies
it), the source code and the license. In the desktop app its [[ui:update.title]] section
looks for, shows and installs updates — see [Updates](install.md#updates). In a browser
it says [[ui:update.server]]. [[ui:about.writeUs]] opens the feedback form.

### Writing to the developers {#feedback}

The **✉** in the header opens a form that goes straight to the developers:

| Field | What to put there |
| --- | --- |
| [[ui:feedback.message]] | What happened, and what you expected instead. Required; at most 20 000 characters. |
| [[ui:feedback.email]] | Optional: where the developers can answer you. Only they see it. |
| [[ui:feedback.screenshots]] | Up to 6 images (PNG, JPEG, WebP or GIF), 8 MB each and 15 MB together. Paste one with <kbd>Ctrl</kbd>+<kbd>V</kbd>, drop files on the window, or press [[ui:feedback.addScreenshot]]. |
| [[ui:feedback.logs]] | The console's lines and [[ui:feedback.systemInfo]], each attached as a file of its own. [[ui:feedback.show]] shows exactly what is sent; untick either to leave it out. |

The attached logs leave out this computer's name, its network address and the names in
your folders' paths. <kbd>Ctrl</kbd>+<kbd>Enter</kbd> sends the form; when it has gone
you get a reference number, which the console keeps too. The message travels through
the studio's own service, which mails it on; the app holds no password for that.

## The bottom panel {#bottom-panel}

The panel under every screen has two tabs, [[ui:console.title]] and
[[ui:dock.inspector]], and between them and the panel's buttons, the strip of running
jobs.

- The **chevron** at its left ([[ui:console.collapse]] / [[ui:console.expand]]) folds the
  panel down to its bar or opens it again. Folded, the bar still shows the latest
  console line; click that line to open the panel.
- Drag the panel's top edge to make it taller or shorter (see [Resizing panes](#panes)).
- The button at its right makes it [[ui:dock.maximise]] and back
  ([[ui:dock.restore]]).

Whether the panel is open, which tab it shows and how tall it is are kept for the next
time.

### The console {#console}

The console says what every tool did and what went wrong, newest last: a message sent
and its size, a monitor started, a response's status and time, a job that ended and
why. Each line has the time (to the millisecond), a tag naming the tool, and the message,
coloured by what it is — done, information, a warning or an error.

- [[ui:console.autoscroll]] keeps the newest line in view as lines arrive; untick it to
  read back while more come in.
- [[ui:common.clear]] empties it.
- It keeps the last 500 lines.
- Switching the language rewrites the whole console in the new one.

### Jobs {#jobs-strip}

Each running job — a monitor, a generator, a beacon, a broker connection, an emulator,
a relay, a storm, a scan, an experiment run — has a pill in the strip with its number
and what it is, and its own button to stop it. With nothing running the strip says
[[ui:console.empty]]. See [Jobs](concepts.md#jobs).

### The Inspector tab {#inspector-tab}

The [[ui:dock.inspector]] tab shows every frame the tools send and receive while
capture is on, beside whichever screen you are working in. Its dot lights up while
capture is on, and a number counts the frames captured. The panel opens tall enough
for the Inspector's list and a frame's detail; a link to a frame elsewhere — in a run's
timeline or an emulator's list — opens this tab on that frame. Once opened, the
Inspector keeps its list and selection while the panel is closed. How to use it:
[Inspector](../tools/inspector.md).

## Sending a signal from anywhere {#palette}

Press <kbd>Ctrl</kbd>+<kbd>K</kbd> on any screen to open [[ui:sig.paletteTitle]]: type a
few letters of a signal's name, folder or target, choose with <kbd>↑</kbd> and
<kbd>↓</kbd>, and press <kbd>Enter</kbd> to send it. It lists up to 12 signals at a
time. <kbd>Esc</kbd>, a click outside it or <kbd>Ctrl</kbd>+<kbd>K</kbd> again closes
it. The console says what was sent and where. See [Signals](../tools/signals.md).

## Resizing panes {#panes}

A thin handle sits between panes that you can resize: the bottom panel's top edge
([[ui:layout.console]]) and, on the experiment screen, the edge of the properties
([[ui:layout.properties]]) and the top of the run timeline ([[ui:layout.timeline]]).

- Drag the handle.
- Or focus it with <kbd>Tab</kbd> and use the arrow keys: each press moves it 16 pixels,
  four times as far with <kbd>Shift</kbd>; <kbd>Home</kbd> and <kbd>End</kbd> go to the
  smallest and largest size.
- Double-click it, or press <kbd>Enter</kbd> on it, to give the pane its default size
  back.

The sizes are kept for the next time.

## Tooltips {#tooltips}

Screens show labels, values and states, and keep their explanations in tooltips: what
a field expects, what `0` means there, which key does the same. A tooltip appears when
you rest the pointer on something for about half a second, and at once when you reach
it with the keyboard; a field shows the tooltip of its label. <kbd>Esc</kbd>, typing,
a click or scrolling hides it. Screen readers read the same text.

Error messages say where and what went wrong, and why; the system's own wording is
folded under [[ui:err.details]].

## Screens keep their state {#state}

A screen opens the first time you visit it and then stays as it is while you work on
another: what you typed, the last response, a monitor's list of messages and the job
behind it, even the scroll position are all there when you come back. A running job
carries on whichever screen you look at.

Some values are also kept across restarts: the OSC message you were sending, the HTTP
request, the WebSocket address, the pane sizes, the screen you were on.

## The firewall notice {#firewall-notice}

Windows Defender Firewall decides, program by program, whether other machines may reach
it. The first time a program listens, Windows asks the person at the screen — and a
*Cancel* there, or a network Windows considers public, silently drops whatever other
machines send. A monitor that shows nothing is the usual sign.

So in the desktop app on Windows, the first time something starts listening — an OSC
monitor, the discovery listener, an impairment relay, an emulator or an experiment run —
Signal Lab looks at the firewall once. When the firewall is in the way, a notice under
the header says so:

- [[ui:fw.allow]] asks for administrator rights with Windows' own prompt, then lets
  other machines reach Signal Lab on private and domain networks.
- On a network Windows calls public — a venue's Wi-Fi, often — the button is
  [[ui:fw.allowPublic]] instead.
- [[ui:fw.dismiss]] hides the notice for this session.

Allowing replaces Signal Lab's inbound firewall rules with a single allow rule. Traffic
on this computer itself (`127.0.0.1`) is never affected, so you can ignore the notice
while you work on loopback. An install for everyone has the rule already; see
[What the setup adds](install.md#windows-setup-adds). `signallab doctor` reports the same
from a terminal and `signallab firewall allow` fixes it there — see
[The command line](../automation/cli.md). There is no notice on Linux or in a browser: a
server's firewall belongs to its administrator.

## In a browser {#browser}

A [server](../server/index.md)'s page is the same interface, with a few differences:

- The host in the header names the server, with [[ui:app.server]] beside it.
- If the server asks for an access token, you sign in once, and [[ui:app.signOut]] in
  the header ends the session.
- If the page loses its connection to the server, a bar says
  [[ui:app.connectionLost]] until it is back; the console notes both.
- Run reports, exports and Inspector captures are downloaded by the browser instead of
  shown as a path.
- About has no updates: the server is updated with its image.

Everything a server's page does happens on the server: traffic starts from it, monitors
listen on its ports, files land in its data folder. See
[Concepts](concepts.md#desktop-and-server).

## Languages {#languages}

The flag in the header shows the current language and its two letters. Click it to
open the list of every language, each by its flag and its own name, and choose one. In
the list, <kbd>↑</kbd>, <kbd>↓</kbd>, <kbd>Home</kbd> and <kbd>End</kbd> move, a letter
jumps to the next language starting with it — in its own name or in English, so
<kbd>g</kbd> finds Deutsch — <kbd>Enter</kbd> chooses and <kbd>Esc</kbd> closes it.

The interface switches at once, without a restart: every screen, tooltip and error, and
the console's earlier lines too. The first time Signal Lab starts it picks the first of
your system's languages it has, or English, and from then on it keeps your choice — in
a browser, for that browser.

Arabic turns the whole window right to left. What is data stays left to right as it is
written: addresses, hex dumps, code and the experiment canvas.

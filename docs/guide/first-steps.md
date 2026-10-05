---
title: First steps
description: A first session on one computer — send an OSC message and watch it arrive, save it as a signal, ask an emulated API, and build and run a small experiment.
---

# First steps

This session needs nothing but Signal Lab: everything goes to `127.0.0.1`, this
computer, so no device, network or firewall rule is involved. You will:

1. send an OSC message and watch it arrive;
2. see the same message in the Inspector;
3. save it to the library and send it again from anywhere;
4. start an emulated HTTP API and ask it something;
5. run an experiment against that API, read why it fails, fix it and add a check.

If you have not installed Signal Lab yet, see [Installing and updating](install.md).
Not sure where something is in the window? See [The window](interface.md).

## Send an OSC message and watch it arrive {#osc}

First, something to receive the message: the OSC screen's monitor.

1. Open [[ui:nav.osc]] in the sidebar.
2. Under [[ui:osc.monitor]], set [[ui:common.bind]] to `127.0.0.1:9000`, so the monitor
   listens on this computer only.
3. Press [[ui:osc.listen]]. The button becomes [[ui:common.stop]], the console says the
   monitor is listening, and the monitor appears as a job in the bottom panel's strip.

Now the message, from the sender beside it:

4. Under [[ui:osc.sender]], leave [[ui:common.target]] at `127.0.0.1:9000`, the port the
   monitor listens on.
5. Leave [[ui:common.address]] at `/hello/avatar/1` and the one float argument under
   [[ui:common.arguments]] at `1.0` — or type an address and values of your own.
6. Press [[ui:common.send]], or <kbd>Enter</kbd> in the target or address field.

A line appears in the monitor's table: the [[ui:common.time]] it arrived,
[[ui:osc.from]] (`127.0.0.1` and the port it was sent from), the [[ui:osc.address]] and
the [[ui:osc.args]]. Under the sender, a line confirms what was sent and its size in
bytes; send again and it counts the repeats.

::: tip A notice about the firewall?
On Windows, starting the monitor may bring up a notice under the header about Windows
Firewall. It is about messages from *other* machines; traffic on `127.0.0.1` is never
filtered. Press [[ui:fw.dismiss]] for now — [The firewall notice](interface.md#firewall-notice)
explains when to allow it.
:::

## See it in the Inspector {#inspector}

The Inspector records every frame every tool sends and receives — but only while
capture is on.

1. In the bottom panel, open the [[ui:dock.inspector]] tab.
2. Press [[ui:ins.arm]]. The tab's dot lights up.
3. Back in the sender, press [[ui:common.send]] once more.

Two rows appear, newest first: the message as sent (→) and as the monitor received it
(←), each with its protocol, the other end's address, its size and a summary. Click one:
[[ui:ins.detail]] shows which tool sent or received it and on which addresses, the
message [[ui:ins.decoded]], and the [[ui:ins.rawBytes]] it was made of.

Press [[ui:ins.disarm]] when you are done; while capture is off it costs nothing. More
in [Inspector](../tools/inspector.md).

## Save it as a signal and send it again {#signal}

A message you will want again belongs in the signal library.

1. On the OSC screen, press [[ui:sig.saveNew]] under the sender.
2. In [[ui:sig.saveTitle]], set [[ui:sig.name]] to `First message` and
   [[ui:sig.group]] to `Tutorial` — a new folder is made as you save into it.
3. Press [[ui:sig.saveConfirm]].

The sender is now tied to that signal: the button says [[ui:sig.savedState]], and a chip
beside it shows where the signal lives. Change the argument and the chip notes the
change; [[ui:sig.save]] (<kbd>Ctrl</kbd>+<kbd>S</kbd>) would update the signal.

Now send it again, three ways:

- **From the library.** Click the chip: [[ui:nav.signals]] opens with the signal
  selected in the `Tutorial` folder (or open [[ui:nav.signals]] and click it there).
  Press [[ui:sig.fire]], or <kbd>Ctrl</kbd>+<kbd>Enter</kbd>; a double-click on it in the
  list sends it too.
- **From anywhere.** On any screen, press <kbd>Ctrl</kbd>+<kbd>K</kbd>, type `first`,
  and press <kbd>Enter</kbd>.
- **From an experiment.** When you add a node, the menu lists your signals under
  [[ui:exp.group.signals]], ready to become a step that sends one.

Each time, the monitor shows the message arrive and the console names the signal. A
signal sends exactly what its screen would have sent. More in [Signals](../tools/signals.md).

When you are done with OSC, press [[ui:common.stop]] on the monitor.

## Ask an emulated API {#emulator}

Signal Lab ships with five emulators, all on `127.0.0.1`. One of them, the
[[ui:seed.emu.demo-api.name]], is an HTTP API on `127.0.0.1:8080` with these routes:

| Request | Answer |
| --- | --- |
| `GET /health` | `200` with `{"status":"ok","time":"…"}` — the current time |
| `GET /users/:id` | `200` with the user of that id, such as `{"id":"42","name":"User 42"}` |
| `POST /users` | `201` with a `Location` header and the new id |
| `GET /slow` | `200` after 1.5 seconds |
| any method, `/flaky` | `503`, `503`, then `200` from the third request on |
| anything else | `404` |

1. Open [[ui:nav.emulators]]. The [[ui:emu.library]] lists the five; select
   [[ui:seed.emu.demo-api.name]].
2. Press [[ui:emu.start]]. It now answers on `127.0.0.1:8080` and runs as a job.
3. Open [[ui:nav.http]]. The method is `GET`; set the URL to
   `http://127.0.0.1:8080/health`.
4. Press [[ui:common.send]], or <kbd>Enter</kbd> in the URL.

Under [[ui:http.response]] you see the [[ui:http.status]] `200`, the
[[ui:http.latency]], the [[ui:http.size]], the response headers and the JSON body. Send
`http://127.0.0.1:8080/flaky` three times: two `503` answers, then `200` — the way a
service that recovers looks to a client that retries.

Back on [[ui:nav.emulators]], the [[ui:emu.live]] panel counts every request, and
[[ui:emu.received]] lists each one with the [[ui:emu.col.rule]] that answered it and the
[[ui:emu.col.reply]]. Leave the [[ui:seed.emu.demo-api.name]] running for the next part.
More in [Emulators](../tools/emulators.md).

## Run an experiment {#experiment}

An experiment is a flow of steps you can run again and again. The one Signal Lab opens
with the first time — the [[ui:exp.templateHttp]] template — sends a request to
`http://127.0.0.1:8080/` and checks that the answer is `200`.

### Open the template {#open-template}

1. Open [[ui:nav.experiment]].
2. If the canvas does not show four nodes — [[ui:exp.node.start]],
   [[ui:exp.node.http]], [[ui:exp.node.assert_status]], [[ui:exp.node.end]] — press
   **☰** at the left of the toolbar ([[ui:exp.documents]]), pick
   [[ui:exp.templateHttp]] in the list of templates, and press
   [[ui:exp.openDocument]]. Opening replaces the experiment on the canvas;
   <kbd>Ctrl</kbd>+<kbd>Z</kbd> brings the previous one back.

Click a node to see its settings in [[ui:exp.properties]] on the right. Experiments save
themselves as you edit.

### Run it and read why it fails {#first-run}

3. Press [[ui:exp.run]].

The [[ui:exp.timeline]] opens under the canvas, one row per step as it starts
([[ui:exp.running]]) and again as it ends: the time, the node, and how it went. This run
fails:

- [[ui:exp.node.start]] passes and names the run's seed.
- [[ui:exp.node.http]] passes: the request went out and an answer came back,
  `HTTP 404`.
- [[ui:exp.node.assert_status]] fails: it expected `200` and received `404`.

The [[ui:seed.emu.demo-api.name]] has no route for `/`, so it answered `404` — and the
check caught it. The line at the top of the timeline says [[ui:exp.failed]] and why.
Click a row to select its node on the canvas.

::: tip The request itself failed?
If the [[ui:exp.node.http]] step fails with a refused connection, nothing is listening on
`127.0.0.1:8080`: start the [[ui:seed.emu.demo-api.name]] on [[ui:nav.emulators]] and run
again.
:::

### Fix the request {#fix}

4. Click the [[ui:exp.node.http]] node.
5. In [[ui:exp.properties]], change [[ui:sig.url]] to `http://127.0.0.1:8080/health`.
6. Press [[ui:exp.run]].

This time every step passes: [[ui:exp.node.assert_status]] says
[[ui:exp.step.checked]], [[ui:exp.node.end]] says [[ui:exp.step.complete]], and the
timeline's title says [[ui:exp.passed]].

### Add a check {#add-check}

A status of `200` says the service answered; it does not say what it answered. Check the
body too:

7. Click the [[ui:exp.node.assert_status]] node.
8. In [[ui:exp.properties]], press [[ui:exp.addNext]] — or press <kbd>A</kbd> with the
   canvas focused. A menu of nodes opens with a search field.
9. Type `assert_body` and press <kbd>Enter</kbd>. A [[ui:exp.node.assert_body]] node is
   added between [[ui:exp.node.assert_status]] and [[ui:exp.node.end]], already wired,
   with its [[ui:exp.contains]] field ready for typing.
10. Type `"status":"ok"`.
11. Press [[ui:exp.run]].

The new step passes. Change the text to something the body does not contain and run
again to see it fail with the reason.

### What a run leaves behind {#report}

- **A report.** When a run ends, [[ui:exp.reportSaved]] appears in the timeline's title;
  hover it to see the file. A run that ends, passed or failed, writes one into the
  `runs` folder of your data folder, with the values it used and every step. In a
  browser it is a download link.
- **A seed.** The title also shows the run's seed with [[ui:exp.pinSeed]]: random values
  in a run follow its seed, and pinning it repeats them exactly.

More in [Runs and reports](../experiments/runs.md).

## Clean up {#clean-up}

Press [[ui:app.stopAll]] in the header: it stops the [[ui:seed.emu.demo-api.name]] and
anything else still running. Your signal, the experiment and its reports stay in your
data folder.

## Where next {#next}

- [Concepts](concepts.md): the ideas behind screens, signals, jobs, emulators and
  experiments.
- [Experiments](../experiments/index.md): the editor in full, and every kind of node in
  [Nodes](../experiments/nodes.md).
- [OSC](../protocols/osc.md), [HTTP](../protocols/http.md) and the other protocols'
  pages, when you point Signal Lab at real gear.
- [The command line](../automation/cli.md): run the same experiment from a terminal or
  a pipeline.

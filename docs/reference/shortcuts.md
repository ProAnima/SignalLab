---
title: Keyboard shortcuts
description: Every keyboard shortcut of Signal Lab — anywhere in the app, in the experiment editor, the signal library, the tool screens, the panes and the language menu.
---

# Keyboard shortcuts

Every key Signal Lab answers to, grouped by where it works. Shortcuts with a
single letter or Delete never act while you type in a field.

::: tip
In a browser on a Mac, <kbd>⌘</kbd> works wherever <kbd>Ctrl</kbd> is written
below, except <kbd>Ctrl</kbd>+<kbd>Space</kbd> in a template field.
:::

## Anywhere {#anywhere}

| Keys | What they do |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>K</kbd> | Opens [[ui:sig.paletteTitle]], the signal palette, on any screen — even while typing in a field. Again closes it |
| <kbd>F1</kbd> | Opens this documentation at the page of the screen you are on, like [[ui:app.docs]] |
| <kbd>Tab</kbd>, <kbd>Shift</kbd>+<kbd>Tab</kbd> | Moves between controls. A control's tip shows as it takes the focus; any other key hides it |
| <kbd>Space</kbd>, <kbd>Enter</kbd> | Presses the button that has the focus — every clickable thing is a button |
| <kbd>Esc</kbd> | Closes the dialog that is open |

In the signal palette:

| Keys | What they do |
| --- | --- |
| Typing | Filters the signals |
| <kbd>↑</kbd> <kbd>↓</kbd> | Chooses a signal |
| <kbd>Enter</kbd> | Sends it and closes the palette |
| <kbd>Esc</kbd> | Closes the palette; so does a click beside it |

## Experiment editor {#editor}

On the canvas — when no text field has the focus:

| Keys | What they do |
| --- | --- |
| <kbd>A</kbd> | Opens the menu to add a node ([[ui:exp.addNode]]). With a node selected, the new one goes after it |
| <kbd>Delete</kbd> or <kbd>Backspace</kbd> | Removes the selected connection, or the selected nodes — never [[ui:exp.node.start]] or [[ui:exp.node.end]] |
| <kbd>Tab</kbd> | Moves through the nodes, their ports and the connections; a node or a connection that takes the focus is selected |
| <kbd>Enter</kbd> or <kbd>Space</kbd> on an output, then on a node or its input | Connects them |
| <kbd>←</kbd> <kbd>→</kbd> <kbd>↑</kbd> <kbd>↓</kbd> | Moves the selected nodes by 5 points (20 with <kbd>Shift</kbd>), while a node has the focus. One press-and-hold is one step of the history |
| <kbd>Esc</kbd> | Closes the add menu; else cancels a connection being drawn; else drops the selected connection; else clears a selection of several nodes; else leaves [[ui:exp.fullscreen]]; else leaves [[ui:exp.focus]] |
| <kbd>Ctrl</kbd>+<kbd>Z</kbd> | [[ui:exp.undo]] |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd>, <kbd>Ctrl</kbd>+<kbd>Y</kbd> | [[ui:exp.redo]] |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | [[ui:exp.duplicate]]: a copy of the selected nodes, with the connections between them |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | Finds a node by its type, what it does or its id |
| <kbd>Ctrl</kbd>+<kbd>0</kbd> | [[ui:exp.fit]]: the whole graph in view |
| <kbd>Ctrl</kbd>+<kbd>1</kbd> | [[ui:exp.resetZoom]]: 100 % |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:exp.sendNow]] — for a wait, [[ui:exp.listenNow]]: only the selected node, without running the experiment |
| <kbd>Ctrl</kbd>+<kbd>A</kbd> | Selects every node |
| <kbd>Ctrl</kbd>+<kbd>C</kbd> | [[ui:exp.copy]]: the selected nodes and the connections between them; [[ui:exp.node.start]] and [[ui:exp.node.end]] stay out, as with duplicating |
| <kbd>Ctrl</kbd>+<kbd>X</kbd> | Cuts them |
| <kbd>Ctrl</kbd>+<kbd>V</kbd> | Pastes nodes copied here or in another experiment, with new ids |

<kbd>Ctrl</kbd>+<kbd>A</kbd>, <kbd>C</kbd>, <kbd>X</kbd> and <kbd>V</kbd>
work on nodes while the editor has the focus, or while nothing has it and no
text is selected on the page. With text selected elsewhere — in the
[[ui:console.title]], in a report — they are the page's own:
<kbd>Ctrl</kbd>+<kbd>C</kbd> copies that text.

With the mouse:

| Gesture | What it does |
| --- | --- |
| Drag on the empty canvas | Moves the view |
| <kbd>Shift</kbd> + drag on the empty canvas | Adds the nodes a frame touches to the selection |
| <kbd>Shift</kbd> or <kbd>Ctrl</kbd> + click on a node | Adds it to the selection, or takes it out |
| Drag one of several selected nodes | Moves them all |
| Double-click on the empty canvas | Opens the add menu there |
| <kbd>Ctrl</kbd> + wheel | Zooms at the pointer |

In the add menu, the node finder and the properties:

| Where | Keys | What they do |
| --- | --- | --- |
| Add menu | Typing | Filters the nodes and saved signals |
| Add menu | <kbd>↑</kbd> <kbd>↓</kbd>, <kbd>Enter</kbd>, <kbd>Esc</kbd> | Choose, add, close |
| Node finder | <kbd>↑</kbd> <kbd>↓</kbd> | Choose a node |
| Node finder | <kbd>Enter</kbd> | Shows it on the canvas and selects it |
| Node finder | <kbd>Tab</kbd> | Back to the search field |
| Node finder | <kbd>Esc</kbd> | Closes it |
| [[ui:exp.properties]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:exp.sendNow]] for the node shown |
| [[ui:exp.properties]] | <kbd>Esc</kbd> in a field | Back to the canvas with the node selected, where <kbd>A</kbd> adds the next one |

In a field that takes templates (`{{name}}`):

| Keys | What they do |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>Space</kbd> | Types `{{` at the cursor and lists what can go there: parameters, variables, secrets, generators |
| Typing `{{` | Lists the same |
| <kbd>↑</kbd> <kbd>↓</kbd> | Chooses in the list |
| <kbd>Enter</kbd> or <kbd>Tab</kbd> | Inserts the choice and closes the `}}` |
| <kbd>Esc</kbd> | Closes the list; the field keeps the focus |

In the experiment's dialogs:

| Where | Keys | What they do |
| --- | --- | --- |
| [[ui:exp.params]] | <kbd>Enter</kbd> in the last parameter's value | Adds another parameter |
| [[ui:exp.params]], [[ui:exp.runWith]] | <kbd>Esc</kbd> | Closes the dialog |
| [[ui:exp.runWith]] | <kbd>Enter</kbd> in a field | [[ui:exp.run]] with these values |
| [[ui:exp.secrets]] | <kbd>Enter</kbd> in a value | Saves it |
| [[ui:exp.secrets]] | <kbd>Esc</kbd> in a value or a new name | Cancels; [[ui:exp.params]] stays open |

## Signal library {#signals}

| Where | Keys | What they do |
| --- | --- | --- |
| [[ui:nav.signals]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:sig.fire]]: sends the selected signal |
| [[ui:nav.signals]] | Double-click on a signal | Sends it |
| A folder | <kbd>F2</kbd> | [[ui:sig.renameFolder]] |
| Renaming a folder | <kbd>Enter</kbd>, <kbd>Esc</kbd> | Keeps the new name, or cancels |
| A folder | <kbd>Delete</kbd>, twice | [[ui:sig.removeFolder]]; what is in it moves up a level |
| A folder | <kbd>→</kbd> <kbd>←</kbd> | Opens it, closes it |
| [[ui:nav.http]], [[ui:nav.osc]], [[ui:nav.mqtt]] senders | <kbd>Ctrl</kbd>+<kbd>S</kbd> | [[ui:sig.save]] when the message is tied to a library signal (opened from it or saved there) and was changed; [[ui:sig.saveNew]] when it is not in the library yet |
| [[ui:sig.saveTitle]] | <kbd>Enter</kbd>, <kbd>Esc</kbd> | Saves, or cancels |

## Tool screens {#tools}

| Screen | Keys | What they do |
| --- | --- | --- |
| [[ui:nav.http]] | <kbd>Enter</kbd> in the URL, a header, the credentials or the timeout | Sends the request |
| [[ui:nav.http]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> anywhere in the request, the body included | Sends the request |
| [[ui:nav.osc]] | <kbd>Enter</kbd> in the target, the address or an argument | Sends the message |
| [[ui:nav.ws]] | <kbd>Enter</kbd> in the URL | Connects |
| [[ui:nav.ws]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> in [[ui:ws.message]] | Sends it |
| [[ui:nav.mqtt]] | <kbd>Enter</kbd> in [[ui:mq.addSubscription]] | Subscribes |
| [[ui:nav.broadcast]] | <kbd>Enter</kbd> in [[ui:bc.targetAddress]] or [[ui:bc.targetCidr]] — every mode but [[ui:bc.mode.list]] | [[ui:bc.sendOnce]] |

In [[ui:feedback.title]]: <kbd>Ctrl</kbd>+<kbd>Enter</kbd> in
[[ui:feedback.message]] sends it (so does <kbd>Enter</kbd> in
[[ui:feedback.email]]), <kbd>Ctrl</kbd>+<kbd>V</kbd> anywhere in the dialog
pastes a screenshot from the clipboard, and <kbd>Esc</kbd> closes it.

## Panes {#panes}

The handles between panes — [[ui:layout.console]], [[ui:layout.properties]],
[[ui:layout.timeline]] — take the focus with <kbd>Tab</kbd>:

| Keys | What they do |
| --- | --- |
| <kbd>↑</kbd> <kbd>↓</kbd> | The console and the timeline: taller, shorter, by 16 pixels |
| <kbd>←</kbd> <kbd>→</kbd> | The properties: wider, narrower, by 16 pixels (the other way round when the interface reads right to left) |
| <kbd>Shift</kbd> + an arrow | Four times as far |
| <kbd>Home</kbd>, <kbd>End</kbd> | The smallest size, the largest |
| <kbd>Enter</kbd> | The default size; so does a double-click on the handle |

## Language menu {#language}

The flag and letters in the header.

| Where | Keys | What they do |
| --- | --- | --- |
| On the button | <kbd>↓</kbd> or <kbd>↑</kbd> | Opens the list |
| In the list | <kbd>↓</kbd> <kbd>↑</kbd> | The next language, the previous one |
| In the list | <kbd>Home</kbd>, <kbd>End</kbd> | The first, the last |
| In the list | A letter | The next language whose name — in its own language, yours or English — or letters start with it |
| In the list | <kbd>Enter</kbd> or <kbd>Space</kbd> | Switches to it |
| In the list | <kbd>Esc</kbd> or <kbd>Tab</kbd> | Closes the list |

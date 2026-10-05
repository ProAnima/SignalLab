---
title: Tastenkürzel
description: Jedes Tastenkürzel von Signal Lab — überall in der App, im Experimenteditor, in der Signalbibliothek, auf den Werkzeugansichten, in den Bereichen und im Sprachmenü.
---

# Tastenkürzel

Jede Taste, auf die Signal Lab hört, gruppiert danach, wo sie wirkt. Kürzel mit
einem einzelnen Buchstaben oder Delete wirken nie, während Sie in ein Feld tippen.

::: tip
In einem Browser auf einem Mac wirkt <kbd>⌘</kbd> überall dort, wo unten
<kbd>Ctrl</kbd> steht, außer <kbd>Ctrl</kbd>+<kbd>Space</kbd> in einem
Template-Feld.
:::

## Überall {#anywhere}

| Tasten | Was sie tun |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>K</kbd> | Öffnet [[ui:sig.paletteTitle]], die Signalpalette, auf jeder Ansicht — sogar während Sie in ein Feld tippen. Erneut schließt sie |
| <kbd>F1</kbd> | Öffnet diese Dokumentation auf der Seite der Ansicht, in der Sie sind, wie [[ui:app.docs]] |
| <kbd>Tab</kbd>, <kbd>Shift</kbd>+<kbd>Tab</kbd> | Wechselt zwischen Steuerelementen. Der Hinweis eines Steuerelements erscheint, sobald es den Fokus erhält; jede andere Taste blendet ihn aus |
| <kbd>Space</kbd>, <kbd>Enter</kbd> | Drückt die Schaltfläche, die den Fokus hat — jedes anklickbare Element ist eine Schaltfläche |
| <kbd>Esc</kbd> | Schließt den offenen Dialog |

In der Signalpalette:

| Tasten | Was sie tun |
| --- | --- |
| Tippen | Filtert die Signale |
| <kbd>↑</kbd> <kbd>↓</kbd> | Wählt ein Signal |
| <kbd>Enter</kbd> | Sendet es und schließt die Palette |
| <kbd>Esc</kbd> | Schließt die Palette; ein Klick daneben ebenfalls |

## Experimenteditor {#editor}

Auf der Arbeitsfläche — wenn kein Textfeld den Fokus hat:

| Tasten | Was sie tun |
| --- | --- |
| <kbd>A</kbd> | Öffnet das Menü zum Hinzufügen eines Knotens ([[ui:exp.addNode]]). Ist ein Knoten ausgewählt, kommt der neue nach ihm |
| <kbd>Delete</kbd> oder <kbd>Backspace</kbd> | Entfernt die ausgewählte Verbindung oder die ausgewählten Knoten — nie [[ui:exp.node.start]] oder [[ui:exp.node.end]] |
| <kbd>Tab</kbd> | Wandert durch die Knoten, ihre Ports und die Verbindungen; ein Knoten oder eine Verbindung, der den Fokus erhält, ist ausgewählt |
| <kbd>Enter</kbd> oder <kbd>Space</kbd> auf einem Ausgang, dann auf einem Knoten oder seinem Eingang | Verbindet sie |
| <kbd>←</kbd> <kbd>→</kbd> <kbd>↑</kbd> <kbd>↓</kbd> | Verschiebt die ausgewählten Knoten um 5 Punkte (mit <kbd>Shift</kbd> 20), während ein Knoten den Fokus hat. Ein Gedrückthalten ist ein Schritt der Historie |
| <kbd>Esc</kbd> | Schließt das Hinzufügen-Menü; sonst bricht es eine gerade gezeichnete Verbindung ab; sonst lässt es die ausgewählte Verbindung fallen; sonst hebt es eine Auswahl mehrerer Knoten auf; sonst verlässt es [[ui:exp.fullscreen]]; sonst verlässt es [[ui:exp.focus]] |
| <kbd>Ctrl</kbd>+<kbd>Z</kbd> | [[ui:exp.undo]] |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd>, <kbd>Ctrl</kbd>+<kbd>Y</kbd> | [[ui:exp.redo]] |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | [[ui:exp.duplicate]]: eine Kopie der ausgewählten Knoten mit den Verbindungen zwischen ihnen |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | Findet einen Knoten anhand seines Typs, dessen, was er tut, oder seiner id |
| <kbd>Ctrl</kbd>+<kbd>0</kbd> | [[ui:exp.fit]]: der ganze Graph in Sicht |
| <kbd>Ctrl</kbd>+<kbd>1</kbd> | [[ui:exp.resetZoom]]: 100 % |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:exp.sendNow]] — bei einem Warten [[ui:exp.listenNow]]: nur der ausgewählte Knoten, ohne das Experiment auszuführen |
| <kbd>Ctrl</kbd>+<kbd>A</kbd> | Wählt jeden Knoten |
| <kbd>Ctrl</kbd>+<kbd>C</kbd> | [[ui:exp.copy]]: die ausgewählten Knoten und die Verbindungen zwischen ihnen; [[ui:exp.node.start]] und [[ui:exp.node.end]] bleiben außen vor, wie beim Duplizieren |
| <kbd>Ctrl</kbd>+<kbd>X</kbd> | Schneidet sie aus |
| <kbd>Ctrl</kbd>+<kbd>V</kbd> | Fügt Knoten ein, die hier oder in einem anderen Experiment kopiert wurden, mit neuen ids |

<kbd>Ctrl</kbd>+<kbd>A</kbd>, <kbd>C</kbd>, <kbd>X</kbd> und <kbd>V</kbd>
wirken auf Knoten, während der Editor den Fokus hat oder während nichts ihn hat
und kein Text auf der Seite ausgewählt ist. Ist anderswo Text ausgewählt — in der
[[ui:console.title]], in einem Bericht —, gehören sie der Seite selbst:
<kbd>Ctrl</kbd>+<kbd>C</kbd> kopiert diesen Text.

Mit der Maus:

| Geste | Was sie tut |
| --- | --- |
| Ziehen auf der leeren Arbeitsfläche | Verschiebt die Ansicht |
| <kbd>Shift</kbd> + Ziehen auf der leeren Arbeitsfläche | Fügt die Knoten, die ein Rahmen berührt, der Auswahl hinzu |
| <kbd>Shift</kbd> oder <kbd>Ctrl</kbd> + Klick auf einen Knoten | Fügt ihn der Auswahl hinzu oder nimmt ihn heraus |
| Einen von mehreren ausgewählten Knoten ziehen | Verschiebt sie alle |
| Doppelklick auf die leere Arbeitsfläche | Öffnet dort das Hinzufügen-Menü |
| <kbd>Ctrl</kbd> + Rad | Zoomt am Zeiger |

Im Hinzufügen-Menü, im Knotenfinder und in den Eigenschaften:

| Wo | Tasten | Was sie tun |
| --- | --- | --- |
| Hinzufügen-Menü | Tippen | Filtert die Knoten und gespeicherten Signale |
| Hinzufügen-Menü | <kbd>↑</kbd> <kbd>↓</kbd>, <kbd>Enter</kbd>, <kbd>Esc</kbd> | Wählen, hinzufügen, schließen |
| Knotenfinder | <kbd>↑</kbd> <kbd>↓</kbd> | Wählt einen Knoten |
| Knotenfinder | <kbd>Enter</kbd> | Zeigt ihn auf der Arbeitsfläche und wählt ihn aus |
| Knotenfinder | <kbd>Tab</kbd> | Zurück zum Suchfeld |
| Knotenfinder | <kbd>Esc</kbd> | Schließt ihn |
| [[ui:exp.properties]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:exp.sendNow]] für den gezeigten Knoten |
| [[ui:exp.properties]] | <kbd>Esc</kbd> in einem Feld | Zurück zur Arbeitsfläche mit dem ausgewählten Knoten, wo <kbd>A</kbd> den nächsten hinzufügt |

In einem Feld, das Templates aufnimmt (`{{name}}`):

| Tasten | Was sie tun |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>Space</kbd> | Tippt `{{` an der Einfügemarke und listet auf, was dort stehen kann: Parameter, Variablen, Geheimnisse, Generatoren |
| `{{` tippen | Listet dasselbe |
| <kbd>↑</kbd> <kbd>↓</kbd> | Wählt in der Liste |
| <kbd>Enter</kbd> oder <kbd>Tab</kbd> | Fügt die Wahl ein und schließt die `}}` |
| <kbd>Esc</kbd> | Schließt die Liste; das Feld behält den Fokus |

In den Dialogen des Experiments:

| Wo | Tasten | Was sie tun |
| --- | --- | --- |
| [[ui:exp.params]] | <kbd>Enter</kbd> im Wert des letzten Parameters | Fügt einen weiteren Parameter hinzu |
| [[ui:exp.params]], [[ui:exp.runWith]] | <kbd>Esc</kbd> | Schließt den Dialog |
| [[ui:exp.runWith]] | <kbd>Enter</kbd> in einem Feld | [[ui:exp.run]] mit diesen Werten |
| [[ui:exp.secrets]] | <kbd>Enter</kbd> in einem Wert | Speichert ihn |
| [[ui:exp.secrets]] | <kbd>Esc</kbd> in einem Wert oder einem neuen Namen | Bricht ab; [[ui:exp.params]] bleibt offen |

## Signalbibliothek {#signals}

| Wo | Tasten | Was sie tun |
| --- | --- | --- |
| [[ui:nav.signals]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:sig.fire]]: sendet das ausgewählte Signal |
| [[ui:nav.signals]] | Doppelklick auf ein Signal | Sendet es |
| Ein Ordner | <kbd>F2</kbd> | [[ui:sig.renameFolder]] |
| Einen Ordner umbenennen | <kbd>Enter</kbd>, <kbd>Esc</kbd> | Übernimmt den neuen Namen oder bricht ab |
| Ein Ordner | <kbd>Delete</kbd>, zweimal | [[ui:sig.removeFolder]]; was darin liegt, rückt eine Ebene hoch |
| Ein Ordner | <kbd>→</kbd> <kbd>←</kbd> | Öffnet ihn, schließt ihn |
| [[ui:nav.http]], [[ui:nav.osc]], [[ui:nav.mqtt]] Sender | <kbd>Ctrl</kbd>+<kbd>S</kbd> | [[ui:sig.save]], wenn die Nachricht an ein Bibliothekssignal gebunden (daraus geöffnet oder dort gespeichert) und geändert wurde; [[ui:sig.saveNew]], wenn sie noch nicht in der Bibliothek ist |
| [[ui:sig.saveTitle]] | <kbd>Enter</kbd>, <kbd>Esc</kbd> | Speichert oder bricht ab |

## Werkzeugansichten {#tools}

| Ansicht | Tasten | Was sie tun |
| --- | --- | --- |
| [[ui:nav.http]] | <kbd>Enter</kbd> in der URL, einem Header, den Zugangsdaten oder dem Timeout | Sendet die Anfrage |
| [[ui:nav.http]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> irgendwo in der Anfrage, den Body eingeschlossen | Sendet die Anfrage |
| [[ui:nav.osc]] | <kbd>Enter</kbd> im Ziel, der Adresse oder einem Argument | Sendet die Nachricht |
| [[ui:nav.ws]] | <kbd>Enter</kbd> in der URL | Verbindet |
| [[ui:nav.ws]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> in [[ui:ws.message]] | Sendet sie |
| [[ui:nav.mqtt]] | <kbd>Enter</kbd> in [[ui:mq.addSubscription]] | Abonniert |
| [[ui:nav.broadcast]] | <kbd>Enter</kbd> in [[ui:bc.targetAddress]] oder [[ui:bc.targetCidr]] — jeder Modus außer [[ui:bc.mode.list]] | [[ui:bc.sendOnce]] |

In [[ui:feedback.title]]: <kbd>Ctrl</kbd>+<kbd>Enter</kbd> in
[[ui:feedback.message]] sendet sie (das tut auch <kbd>Enter</kbd> in
[[ui:feedback.email]]), <kbd>Ctrl</kbd>+<kbd>V</kbd> irgendwo im Dialog fügt
einen Screenshot aus der Zwischenablage ein, und <kbd>Esc</kbd> schließt ihn.

## Bereiche {#panes}

Die Griffe zwischen Bereichen — [[ui:layout.console]],
[[ui:layout.properties]], [[ui:layout.timeline]] — erhalten den Fokus mit
<kbd>Tab</kbd>:

| Tasten | Was sie tun |
| --- | --- |
| <kbd>↑</kbd> <kbd>↓</kbd> | Konsole und Zeitleiste: höher, niedriger, um 16 Pixel |
| <kbd>←</kbd> <kbd>→</kbd> | Die Eigenschaften: breiter, schmaler, um 16 Pixel (andersherum, wenn die Oberfläche von rechts nach links liest) |
| <kbd>Shift</kbd> + eine Pfeiltaste | Viermal so weit |
| <kbd>Home</kbd>, <kbd>End</kbd> | Die kleinste Größe, die größte |
| <kbd>Enter</kbd> | Die Standardgröße; ein Doppelklick auf den Griff ebenso |

## Sprachmenü {#language}

Die Flagge und die Buchstaben in der Kopfzeile.

| Wo | Tasten | Was sie tun |
| --- | --- | --- |
| Auf der Schaltfläche | <kbd>↓</kbd> oder <kbd>↑</kbd> | Öffnet die Liste |
| In der Liste | <kbd>↓</kbd> <kbd>↑</kbd> | Die nächste Sprache, die vorherige |
| In der Liste | <kbd>Home</kbd>, <kbd>End</kbd> | Die erste, die letzte |
| In der Liste | Ein Buchstabe | Die nächste Sprache, deren Name — in ihrer eigenen Sprache, Ihrer oder Englisch — oder Buchstaben damit beginnt |
| In der Liste | <kbd>Enter</kbd> oder <kbd>Space</kbd> | Wechselt zu ihr |
| In der Liste | <kbd>Esc</kbd> oder <kbd>Tab</kbd> | Schließt die Liste |

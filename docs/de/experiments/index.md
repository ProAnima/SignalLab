---
title: Der Editor
description: Experimente auf der Knoten-Arbeitsfläche bauen, bearbeiten und ausführen — Knoten hinzufügen und verbinden, auswählen und kopieren, der Eigenschaften-Bereich, Jetzt senden, Prüfung, Ausführen und die Dateien dahinter.
---

# Der Experiment-Editor

Ein Experiment ist ein Test, der als Graph geschrieben ist: Knoten, die senden (eine
HTTP-Anfrage, eine OSC-Nachricht, eine MQTT-Veröffentlichung…), auf eine Antwort
warten, prüfen, was zurückkam, die andere Seite spielen oder das Netzwerk auf
Kommando unterbrechen, verbunden durch Drähte, die sagen, was als Nächstes läuft.
Ein Durchlauf beginnt bei [[ui:exp.node.start]], folgt den Drähten und besteht, wenn er
[[ui:exp.node.end]] erreicht und jeder Schritt bestanden ist. Sie bauen ihn in der
Ansicht [[ui:nav.experiment]], führen ihn dort aus und führen dieselbe Datei von der
Kommandozeile oder einem Server aus.

Der Editor hält jeweils ein Experiment und speichert es während der Arbeit. Um
mehrere zu behalten, exportieren Sie Momentaufnahmen oder öffnen Sie ein anderes aus
einer Datei (siehe [Speichern und Dateien](#files)).

Jede Art von Knoten, ihre Felder und Ausgänge stehen in der
[Knotenreferenz](nodes.md). Wie Werte zwischen Knoten fließen, steht in
[Daten und Vorlagen](data.md); parallele Zweige, Schleifen, Neuversuche und
Wiederholungen in [Ablauf](flow.md).

## Die Ansicht {#screen}

| Bereich | Was er enthält |
| --- | --- |
| Werkzeugleiste (oben) | Name und Speicherzustand des Experiments, Knoten hinzufügen, Parameter, Profile, die Bereiche, Fokusmodus, Vollbild und Ausführen |
| Arbeitsflächenleiste | Rückgängig und Wiederholen, die Knotensuche, [[ui:exp.arrange]] und Zoom |
| Arbeitsfläche | Der Graph: Knoten, Drähte und der auf ihnen gezeichnete Fortschritt des Durchlaufs |
| [[ui:exp.properties]] (rechts) | Die Felder des ausgewählten Knotens, seine Vorschau und [[ui:exp.sendNow]]; oder was mehrere ausgewählte Knoten oder ein ausgewählter Draht erlauben |
| [[ui:exp.timeline]] (unten) | Die Schritte des aktuellen oder letzten Durchlaufs, sein Ergebnis, Bericht und Startwert |

Der Eigenschaften-Bereich und die Zeitleiste haben einen Griff an ihrer Innenseite:
Ziehen Sie ihn, oder fokussieren Sie ihn mit <kbd>Tab</kbd> und verwenden Sie die
Pfeiltasten (<kbd>Shift</kbd> für größere Schritte); ein Doppelklick oder
<kbd>Enter</kbd> gibt dem Bereich seine Standardgröße zurück. Die Größen bleiben für
das nächste Mal erhalten. Die Zeitleiste bleibt eingeklappt, bis der erste Durchlauf
sie öffnet.

## Die Werkzeugleiste {#toolbar}

| Bedienelement | Was es tut |
| --- | --- |
| ☰ [[ui:exp.documents]] | Öffnet die Vorlagen, das Öffnen einer JSON-Datei und den Export ([Speichern und Dateien](#files)) |
| [[ui:exp.name]] | Der Name des Experiments; ein Durchlauf braucht einen. Berichte von Durchläufen halten ihn fest, und [[ui:exp.compare]] findet frühere Durchläufe darüber |
| [[ui:exp.saved]] / [[ui:exp.saving]] / [[ui:exp.saveError]] | Ob die letzte Änderung auf der Platte ist |
| ⚠ [[ui:exp.needsLinks]] | Wird angezeigt, solange das Experiment nicht laufen kann; sein Tooltip sagt, warum, ein Klick wählt den Knoten aus, um den es geht ([Prüfung](#validation)) |
| ＋ [[ui:exp.addNode]] | Öffnet das Hinzufügen-Menü, nach dem ausgewählten Knoten, wenn es einen gibt (<kbd>A</kbd>) |
| [[ui:exp.profile]] | Welches Profil der Durchlauf, die Vorschau und Jetzt senden verwenden; angezeigt, wenn das Experiment Profile hat. ⚠ markiert ein Profil, das nicht laufen würde |
| `{ }` [[ui:exp.params]] | Parameter, Profile, der Startwert, Cookies und Geheimnisse ([Daten und Vorlagen](data.md)) |
| ☷ [[ui:exp.properties]] | Zeigt oder verbirgt den Eigenschaften-Bereich |
| ▢ [[ui:exp.focus]] | Verbirgt die Seitenleiste, die Kopfzeile und den unteren Bereich ([Fokusmodus und Vollbild](#focus)) |
| ⛶ [[ui:exp.fullscreen]] | Das Fenster im Vollbild, mit Fokusmodus |
| [[ui:exp.run]] | Prüft, speichert und führt das Experiment aus; während es läuft, ist die Schaltfläche [[ui:common.stop]] |
| ▾ [[ui:exp.runWith]] | Ein Durchlauf mit einem anderen Profil, anderen Parameterwerten oder einem vorgegebenen Startwert, ohne das Experiment zu ändern |

## Auf der Arbeitsfläche bewegen {#canvas}

- **Schwenken**: Ziehen Sie die leere Arbeitsfläche oder scrollen Sie.
- **Zoom**: <kbd>Ctrl</kbd> und das Mausrad zoomen um den Zeiger; − und ＋ in der
  Arbeitsflächenleiste schreiten um 10 % weiter. Der Zoom geht von 15 % bis 200 %;
  der Prozentwert zwischen den Schaltflächen zeigt, wo Sie sind.
- **1:1** ([[ui:exp.resetZoom]], <kbd>Ctrl</kbd>+<kbd>1</kbd>) geht zurück auf 100 %.
- ⊡ [[ui:exp.fit]] (<kbd>Ctrl</kbd>+<kbd>0</kbd>) zeigt den ganzen Graphen, höchstens
  bei 100 %.
- **Einen Knoten finden**: [[ui:exp.nodes]] in der Arbeitsflächenleiste (es zeigt,
  wie viele Knoten es gibt) oder <kbd>Ctrl</kbd>+<kbd>F</kbd>. Tippen Sie Wörter aus
  dem Namen des Knotens, seiner Zusammenfassung (eine URL, eine Adresse, ein Topic)
  oder seiner id; <kbd>↑</kbd> <kbd>↓</kbd> wählen, <kbd>Enter</kbd> wählt den Knoten
  aus und holt ihn ins Sichtfeld (auf mindestens 80 % gezoomt), <kbd>Esc</kbd>
  schließt.

Es gibt keine Minimap: [[ui:exp.fit]] und die Suche erledigen ihre Arbeit.

## Knoten und Drähte {#nodes-and-wires}

Ein Knoten zeigt seine Art, eine einzeilige Zusammenfassung dessen, was er tut, und
Plaketten für seine Einstellungen: ↻ und eine Zahl für Neuversuch, × und eine Anzahl
(oder eine Zeit) für Wiederholung, ⚡ für Last. Sein Eingang ist links — jeder Knoten
außer [[ui:exp.node.start]] hat einen — und seine Ausgänge rechts:

| Ausgang | An | Verfolgt, wenn |
| --- | --- | --- |
| [[ui:exp.outputPort]] | Die meisten Knoten | Der Schritt bestanden ist |
| [[ui:exp.yes]] / [[ui:exp.no]] | [[ui:exp.node.branch_status]], [[ui:exp.node.branch_value]] | Der Vergleich zutraf / nicht zutraf |
| [[ui:exp.branch1]] / [[ui:exp.branch2]] | [[ui:exp.node.fork]] | Immer, beide zugleich |
| [[ui:exp.portMatched]] / [[ui:exp.portTimeout]] | Warteknoten | Eine passende Nachricht traf ein / rechtzeitig keine |
| [[ui:exp.portBody]] / [[ui:exp.portDone]] / [[ui:exp.portLimit]] | [[ui:exp.node.loop]] | Eine weitere Iteration / die Schleife ist vorbei / die Iterationen gingen aus |

**Ein Ausgang darf mehrere Drähte haben.** Der erste Draht führt den Zweig fort;
jeder weitere Draht startet einen parallelen Zweig mit einer Kopie der zu diesem
Zeitpunkt bekannten Variablen. Ein Knoten
[[[ui:exp.node.join]]](nodes.md#node-join) wartet auf jeden Draht, der in ihn führt.
Einzelheiten stehen in [Ablauf](flow.md).

[[ui:exp.portTimeout]] und [[ui:exp.portLimit]] sind optional: Ohne Draht lässt ein
Timeout oder das Ausgehen der Iterationen den Schritt fehlschlagen. Jeder andere
Ausgang muss einen Draht haben, bevor das Experiment laufen kann.

Der Draht vom Rumpf einer [[ui:exp.node.loop]] zurück zu ihr wird als Bogen über den
Rumpf gezeichnet; er ist der einzige Draht, der rückwärts führen darf.

## Knoten hinzufügen {#adding}

### Das Hinzufügen-Menü {#add-menu}

Das Hinzufügen-Menü listet jede Art von Knoten nach Gruppe auf —
[[ui:exp.group.action]], [[ui:exp.group.observe]], [[ui:exp.group.emulate]],
[[ui:exp.group.fault]], [[ui:exp.group.data]], [[ui:exp.group.check]],
[[ui:exp.group.flow]] — und dann [[ui:exp.group.signals]]: die Signale Ihrer
[Bibliothek](../tools/signals.md), die ein Knoten werden können (OSC, HTTP, UDP mit
Text-Nutzdaten, MQTT), mit ausgefüllten Feldern.

Ihr Suchfeld hat beim Öffnen den Fokus. Tippen Sie ein paar Buchstaben: Jedes Wort
muss im Namen des Knotens, seiner Beschreibung oder seinem Typ (`http`,
`wait_osc`) vorkommen; ein Signal wird über seinen Namen, seinen Ordner, seinen
Transport oder sein Ziel gefunden. <kbd>↑</kbd> <kbd>↓</kbd> wählen, <kbd>Enter</kbd>
fügt hinzu, <kbd>Esc</kbd> schließt. Die Zeile oben sagt, wohin der Knoten kommt:
nach einen Knoten oder parallel zu einem.

Ein neuer Knoten ist ausgewählt, der Eigenschaften-Bereich öffnet sich, und sein
Hauptfeld — die URL, die Adresse, das Topic, die Verzögerung — hat den Fokus, mit
ausgewähltem Text, sodass Sie sofort tippen können. <kbd>Esc</kbd> in einem Feld
bringt Sie zurück zum Knoten auf der Arbeitsfläche, bereit für das nächste
<kbd>A</kbd>.

### Nach dem ausgewählten Knoten {#add-after}

<kbd>A</kbd>, ＋ [[ui:exp.addNode]] in der Werkzeugleiste und ＋ [[ui:exp.addNext]]
im Eigenschaften-Bereich fügen den nächsten Schritt nach dem ausgewählten Knoten
hinzu:

- Er hängt sich an den ersten Ausgang des Knotens, der noch keinen Draht hat (etwa
  das freie [[ui:exp.no]] einer [[ui:exp.node.branch_status]]), oder sonst an seinen
  ersten Ausgang.
- Hat dieser Ausgang bereits einen Draht, wird der neue Knoten in ihn eingefügt (in
  den ersten, wenn es mehrere gibt): Der Draht läuft nun durch den neuen Knoten, und
  alles danach rückt nach rechts, um Platz zu machen.
- Ist [[ui:exp.node.end]] ausgewählt, kommt der Knoten davor, wenn ein Draht in ihn
  führt.
- Ist nichts ausgewählt, landet der Knoten in der Mitte der Ansicht, ohne Drähte.

Ein Knoten mit einem einzigen Ausgang, der auf den leeren [[ui:exp.portBody]] einer
[[ui:exp.node.loop]] gesetzt wird — so oder [auf einem neuen Draht](#add-branch) —,
wird auch zurück zu ihr verdrahtet, sodass der Rumpf sofort vollständig ist.

### In einen Draht {#insert}

Jeder Draht hat ein ＋ in der Mitte ([[ui:exp.insertNode]]). Es öffnet das
Hinzufügen-Menü, und der gewählte Knoten wird in diesen Draht eingefügt. Eine so
eingefügte [[ui:exp.node.loop]] setzt den Ablauf durch ihren [[ui:exp.portDone]]
fort.

### Auf einem neuen Draht {#add-branch}

Ziehen Sie einen Draht aus einem Ausgang und lassen Sie ihn auf der leeren
Arbeitsfläche los: Das Hinzufügen-Menü öffnet sich dort, und der neue Knoten kommt
auf einen neuen Draht dieses Ausgangs — neben die Drähte, die er bereits hat, sodass
er parallel zu ihnen läuft. Dasselbe geschieht, wenn Sie einen Ausgang anklicken und
dann die leere Arbeitsfläche anklicken oder doppelklicken.

### Überall {#add-anywhere}

Doppelklicken Sie auf die leere Arbeitsfläche, um an dieser Stelle einen Knoten ohne
Drähte hinzuzufügen.

### Aus anderen Ansichten {#from-screens}

- [[ui:common.toExperiment]] in den Ansichten [OSC](../protocols/osc.md) und
  [HTTP](../protocols/http.md) fügt das eben Ausprobierte als nächsten Schritt hinzu
  — direkt vor [[ui:exp.node.end]], wenn ein Draht in ihn führt — und wechselt zum
  Editor.
- [[ui:osc.waitForThis]] neben einer Nachricht im OSC-Monitor fügt einen
  [[ui:exp.node.wait_osc]] hinzu, der sie erkennt: ihre Adresse und ihren Text,
  Ganzzahl- und Wahr/Falsch-Argumente (Floats sind Messwerte, die sich ändern, daher
  bleiben sie weg). Stoppen Sie den Monitor vor dem Ausführen: Der Durchlauf lauscht
  selbst auf diesem Port.
- [[ui:mq.waitForThis]] an einem Topic des [MQTT](../protocols/mqtt.md)-Baums fügt
  einen [[ui:exp.node.wait_mqtt]] auf diesem Topic bei diesem Broker hinzu.

Während ein Durchlauf läuft, können keine Knoten hinzugefügt werden; die Konsole sagt
es.

## Knoten verbinden {#connecting}

- **Ziehen** Sie von einem Ausgang auf einen Knoten. Loslassen innerhalb von
  24 Pixeln um einen Knoten genügt.
- **Klicken** Sie einen Ausgang an (oder fokussieren Sie ihn und drücken Sie
  <kbd>Enter</kbd> oder <kbd>Space</kbd>): Die Arbeitsflächenleiste sagt
  [[ui:exp.chooseInput]]. Klicken Sie einen Knoten oder seinen Eingang an, um zu
  verbinden; klicken Sie die leere Arbeitsfläche an, um dort einen Knoten auf einem
  neuen Draht hinzuzufügen; [[ui:exp.cancelLink]] oder <kbd>Esc</kbd> gibt auf.

Ein Draht wird abgelehnt, wenn er einen Zyklus erzeugen würde (außer dem Rückweg
einer [[ui:exp.node.loop]]), wenn er in [[ui:exp.node.start]] hinein oder aus
[[ui:exp.node.end]] heraus führt oder wenn er einen Knoten mit sich selbst verbinden
würde; der Editor sagt es. Einen bereits vorhandenen Draht zu ziehen, ändert nichts.

Um einen **Draht zu entfernen**, klicken Sie ihn an — der Eigenschaften-Bereich zeigt,
was er verbindet — und drücken Sie <kbd>Delete</kbd>, oder verwenden Sie dort
[[ui:exp.removeWire]]. Beim Überfahren eines Drahts erscheint zudem ein × über seinem
＋. Die Eigenschaften eines Knotens listen seine ausgehenden Drähte auf, jeder mit ×
([[ui:exp.disconnect]]).

## Mehrere Knoten auswählen {#selection}

| Um | Tun Sie |
| --- | --- |
| Einen Knoten auswählen | Klicken Sie ihn an oder bewegen Sie sich mit <kbd>Tab</kbd> zu ihm |
| Einen Knoten zur Auswahl hinzufügen oder herausnehmen | <kbd>Shift</kbd> oder <kbd>Ctrl</kbd> und ein Klick |
| Mit einem Rahmen auswählen | <kbd>Shift</kbd> und auf der leeren Arbeitsfläche ziehen: Jeder Knoten, den der Rahmen berührt, tritt der Auswahl bei |
| Alle Knoten auswählen | <kbd>Ctrl</kbd>+<kbd>A</kbd> |
| Die Auswahl aufheben | Klicken Sie die leere Arbeitsfläche an; <kbd>Esc</kbd>, wenn mehrere ausgewählt sind |
| Sie verschieben | Ziehen Sie einen von ihnen: Alle bewegen sich |
| Sie leicht verschieben | Ist ein Knoten fokussiert, verschieben die Pfeiltasten die Auswahl um 5 Pixel, mit <kbd>Shift</kbd> um 20 |

Der Eigenschaften-Bereich zeigt den zuletzt angeklickten Knoten; sind mehrere
ausgewählt, zeigt er deren Anzahl, mit [[ui:exp.copy]], [[ui:exp.duplicate]] und
[[ui:exp.deleteSelected]].

### Kopieren, Ausschneiden und Einfügen {#copy-paste}

<kbd>Ctrl</kbd>+<kbd>C</kbd> kopiert die ausgewählten Knoten und die Drähte zwischen
ihnen als Text; <kbd>Ctrl</kbd>+<kbd>X</kbd> entfernt sie zusätzlich;
<kbd>Ctrl</kbd>+<kbd>V</kbd> fügt sie ein — in dieses Experiment, in ein später
geöffnetes anderes oder in ein anderes Signal-Lab-Fenster. Der Text ist JSON, sodass
Sie ihn auch in einer Datei oder einer Nachricht aufbewahren können.

- [[ui:exp.node.start]] und [[ui:exp.node.end]] sind einzigartig: Sie werden nie
  kopiert.
- Drähte zwischen einem kopierten Knoten und dem Rest des Graphen werden nicht
  kopiert; verdrahten Sie die Kopie selbst.
- Jeder eingefügte Knoten erhält eine neue id.
- Ein Knoten, der einen anderen benennt — [[ui:exp.node.impairment_change]],
  [[ui:exp.node.emulator_state]], [[ui:exp.node.ws_send]],
  [[ui:exp.node.wait_ws]], [[ui:exp.node.ws_close]] —, benennt die Kopie, wenn diese
  ebenfalls kopiert wurde; andernfalls benennt er weiter das Original, wenn es in
  diesem Experiment ist, oder den ersten Knoten dieser Art hier.
- Ein eingefügter [[ui:exp.node.emulator]] oder [[ui:exp.node.impairment]], auf
  dessen Port dieses Experiment bereits lauscht, weicht auf den nächsten freien Port
  aus. Was an das Original sendet, tut das weiterhin.
- Die Kopie landet 32 Pixel rechts von und eine Zeile unter ihrer Herkunft, weiter
  unten, bis sie keinen Knoten überdeckt, und ist ausgewählt.

<kbd>Ctrl</kbd>+<kbd>D</kbd> ([[ui:exp.duplicate]]) tut dasselbe ohne die
Zwischenablage.

### Löschen {#deleting}

<kbd>Delete</kbd> oder <kbd>Backspace</kbd> ([[ui:exp.delete]],
[[ui:exp.deleteSelected]]) entfernt die ausgewählten Knoten und ihre Drähte. Ein
Knoten, der genau einen Draht hinein und einen hinaus hatte, hinterlässt an seiner
Stelle einen Draht vom Knoten davor zum Knoten danach, sodass eine Kette verbunden
bleibt. [[ui:exp.node.start]] und [[ui:exp.node.end]] lassen sich nicht löschen.

## Rückgängig und Wiederholen {#undo}

<kbd>Ctrl</kbd>+<kbd>Z</kbd> macht rückgängig;
<kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd> oder <kbd>Ctrl</kbd>+<kbd>Y</kbd>
wiederholt (↶ ↷ in der Arbeitsflächenleiste). Ein Ziehen, eine Reihe von
Pfeiltasten-Verschiebungen oder das Tippen in einem Feld ist ein Schritt. Die letzten
100 Schritte bleiben erhalten, solange die App offen ist, über Ansichtswechsel hinweg;
das Öffnen eines anderen Experiments ist ebenfalls ein Schritt, sodass
<kbd>Ctrl</kbd>+<kbd>Z</kbd> das vorherige zurückbringt. Rückgängig und Wiederholen
warten, während ein Durchlauf läuft.

## Anordnen {#arrange}

[[ui:exp.arrange]] legt den Graphen von links nach rechts aus: jeden Knoten in der
Spalte nach dem letzten Knoten, der zu ihm führt, den Rumpf einer
[[ui:exp.node.loop]] in ihrer Zeile — und passt dann die Ansicht an. Es ist ein
Schritt der Historie. Ein Entwurf mit einem Zyklus, der nicht der einer Schleife ist,
bleibt, wie er ist.

## Der Eigenschaften-Bereich {#properties}

Ist ein Knoten ausgewählt, zeigt der Bereich von oben nach unten:

1. Den Namen des Knotens (sein Tooltip sagt, was er tut) und, wenn das Experiment
   wegen dieses Knotens nicht laufen kann, was falsch ist.
2. Seine Felder. Felder, die [Vorlagen](data.md) annehmen, schlagen Parameter,
   Variablen, Geheimnisse und Generatoren vor, während Sie `{{` tippen, oder bei
   <kbd>Ctrl</kbd>+<kbd>Space</kbd>.
3. Seine Einstellungen: [[ui:exp.loadOn]] bei einer HTTP-Anfrage, [[ui:exp.repeatOn]]
   bei Knoten, die senden, [[ui:exp.retryOn]] bei Knoten, die senden oder lauschen,
   und [[ui:exp.expectReply]] bei OSC- und UDP-Nachrichten (siehe
   [Knoteneinstellungen](nodes.md#settings)). Last ersetzt Wiederholung und
   Neuversuch, solange sie an ist.
4. Das Lastergebnis des letzten Durchlaufs, bei einem HTTP-Knoten, der unter Last
   lief.
5. Die Vorschau — [[ui:exp.preview]], [[ui:exp.previewWait]] oder
   [[ui:exp.previewCheck]] —, wenn der Knoten Vorlagen hat: was er senden, erwarten
   oder vergleichen würde, aufgelöst mit den aktuellen Parametern und den bisher
   bekannten Werten ([Jetzt senden und die Vorschau](#send-now)).
6. [[ui:exp.sendNow]] oder [[ui:exp.listenNow]] und was es zuletzt getan hat.
7. Seine ausgehenden Drähte, jeder mit ×.
8. ⚡ [[ui:exp.routeThrough]] bei einer OSC- oder UDP-Nachricht: Eine
   [[ui:exp.node.impairment]] wird vor den Knoten gesetzt — sie lauscht auf einem
   freien Loopback-Port ab 9010 aufwärts und leitet an das Ziel des Knotens weiter —,
   und der Knoten wird auf sie gerichtet, sodass der nächste Durchlauf
   verschlechtert, was er sendet ([Störungen](faults.md)).
9. ＋ [[ui:exp.addNext]], [[ui:exp.copy]], [[ui:exp.duplicate]] und
   [[ui:exp.delete]].

Ein Doppelklick auf einen Knoten öffnet den Bereich mit dem Cursor in seinem
Hauptfeld. Sind mehrere Knoten ausgewählt, bietet der Bereich, was mit allen getan
werden kann; ist ein Draht ausgewählt, was er verbindet und [[ui:exp.removeWire]].
Während ein Durchlauf läuft, sind die Felder gesperrt.

## Jetzt senden und die Vorschau {#send-now}

[[ui:exp.sendNow]] (<kbd>Ctrl</kbd>+<kbd>Enter</kbd>, auch aus dem Inneren der
Knotenfelder) sendet den ausgewählten Knoten für sich allein, ohne das Experiment
auszuführen, durch denselben Code, den ein Durchlauf verwendet. Es wird bei
[[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]],
[[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_connect]] und
[[ui:exp.node.ws_send]] angeboten.
Bei einem Warteknoten ist es [[ui:exp.listenNow]]: Der Warteknoten lauscht von jetzt
an, bis eine Nachricht passt oder sein Timeout endet.

- Der Knoten verwendet das aktive Profil, die gespeicherten Geheimnisse und die
  bisher bekannten Variablenwerte — aus dem letzten Durchlauf und aus früheren
  Ergebnissen von [[ui:exp.sendNow]]. Benennt eine Vorlage einen Wert, den noch
  niemand gesetzt hat, wird nichts gesendet und die Namen werden aufgelistet.
- Es wird einmal gesendet: Neuversuch, Wiederholung und Last gelten nicht, keine
  Cookies werden behalten, und Emulatoren und Relais des Experiments werden nicht
  gestartet.
- Ein [[ui:exp.node.ws_send]] oder [[ui:exp.node.wait_ws]] öffnet die Verbindung, die
  sein [[ui:exp.node.ws_connect]] beschreibt, für diesen einen Test.
- Bei einer HTTP-Anfrage wird die Antwort gezeigt — Status, Zeit, Größe und der
  Body, JSON-formatiert. Die Werte, die die [[ui:exp.node.extract]]-Knoten nach der
  Anfrage nehmen würden, werden sofort eingefüllt. Klicken Sie einen Wert in einer
  JSON-Antwort an, um direkt nach der Anfrage einen [[ui:exp.node.extract]]-Knoten
  dafür hinzuzufügen, mit benannter Variable und bekanntem Wert.
  [[ui:http.mockThis]] macht die Antwort zu einer Route eines
  [Emulators](../tools/emulators.md).
- Das Ergebnis wird außerdem in die Konsole geschrieben.

Die Vorschau wird ebenfalls von der Engine aufgelöst, etwa eine Viertelsekunde nach
einer Änderung. Geheimwerte werden dort und nirgends gezeigt.

## Prüfung {#validation}

Der Editor prüft das Experiment während der Bearbeitung und noch einmal, wenn Sie
[[ui:exp.run]] drücken:

- Ein Ausgang, der noch einen Draht braucht, pulsiert bernsteinfarben.
- Ein Knoten, den kein Draht von [[ui:exp.node.start]] erreicht, wird gestrichelt
  gezeichnet; sein Tooltip sagt es.
- Der Knoten, um den es bei einem Problem geht, wird umrandet, und das Problem steht
  über seinen Feldern.
- ⚠ [[ui:exp.needsLinks]] in der Werkzeugleiste benennt das Problem; ein Klick wählt
  den Knoten aus.

Ein Experiment läuft, wenn unter anderem:

- es einen Namen, genau eine [[ui:exp.node.start]] und eine [[ui:exp.node.end]] und
  höchstens 64 Knoten hat;
- jeder Knoten von [[ui:exp.node.start]] aus erreichbar ist und jeder erforderliche
  Ausgang einen Draht hat;
- die einzigen Zyklen die Rümpfe von [[ui:exp.node.loop]]-Knoten sind, die zu ihnen
  zurückführen;
- jede Prüfung und jeder [[ui:exp.node.extract]] auf jedem Pfad eine HTTP-Anfrage
  davor hat (eine unter Last zählt nicht: Sie hinterlässt keine Antwort);
- ein [[ui:exp.node.ws_send]], [[ui:exp.node.wait_ws]] oder [[ui:exp.node.ws_close]]
  nach dem [[ui:exp.node.ws_connect]] kommt, den er verwendet;
- jedes Feld ausgefüllt und im gültigen Bereich ist und jede Vorlage etwas benennt,
  das an dieser Stelle bekannt ist.

Unfertige Entwürfe werden trotzdem gespeichert. Ein Durchlauf wird vor jeglichem
Verkehr abgelehnt, wenn ein benötigtes Geheimnis nicht gespeichert ist. Jede Meldung
ist in der [Fehlerreferenz](../reference/errors.md) aufgeführt.

## Ausführen {#running}

[[ui:exp.run]] prüft das Experiment, speichert es und startet es. Kann es nicht
laufen, wird das Problem gezeigt und sein Knoten ausgewählt. Andernfalls öffnet sich
die Zeitleiste, und Knoten leuchten auf, während der Durchlauf sie erreicht: ● läuft,
✓ bestanden, ✕ fehlgeschlagen, ↻ Neuversuch, ⟳ Wiederholung, ⚡ unter Last; Drähte,
die den Ablauf trugen, sind ebenfalls gefärbt.

- Warteknoten, Emulatoren und Störungs-Relais öffnen ihre Ports vor dem ersten
  Schritt, damit nichts, was früh eintrifft, verloren geht. Ein Port, der sich nicht
  öffnen lässt (etwa weil ein anderes Programm ihn belegt), hindert den Durchlauf am
  Starten, und der Knoten, der ihn braucht, wird gezeigt.
- [[ui:common.stop]] beendet den Durchlauf sofort: Pausen, Warteknoten und Lasten
  eingeschlossen. Der Durchlauf ist außerdem ein Job in der Job-Leiste der Konsole,
  die ihn ebenfalls stoppen kann.
- Ein Durchlauf, der länger als 300 Sekunden dauert, wird gestoppt und schlägt fehl.
- Während ein Durchlauf läuft, lässt sich das Experiment nicht bearbeiten; Schwenken,
  Zoomen und Auswählen funktionieren weiterhin.

▾ [[ui:exp.runWith]] neben der Schaltfläche führt einmal mit einem anderen Profil,
anderen Parameterwerten oder einem vorgegebenen Startwert aus; das Experiment selbst
wird nicht geändert, und das Formular behält das Eingetippte, bis die App schließt
([Daten und Vorlagen](data.md)).

### Die Durchlauf-Zeitleiste {#timeline}

Die [[ui:exp.timeline]] listet eine Zeile pro Schritt: die Zeit, den Knoten und was
geschah — bestanden, fehlgeschlagen und warum, ein Neuversuch, der Fortschritt einer
Wiederholung, die Zahlen einer Last einmal pro Sekunde. Klicken Sie eine Zeile an, um
ihren Knoten auf der Arbeitsfläche auszuwählen.

Ihre Titelzeile enthält:

- das Ergebnis: [[ui:exp.passed]], [[ui:exp.failed]] mit dem Grund oder
  [[ui:exp.stopped]];
- [[ui:exp.reportSaved]] — die Berichtsdatei des Durchlaufs (auf einem Server ein
  Download);
- [[ui:exp.compare]] — dieser Durchlauf neben einem früheren desselben Experiments;
- das Profil und die geänderten Werte, die der Durchlauf verwendet hat, wenn es
  welche gab;
- den Startwert des Durchlaufs, mit [[ui:exp.pinSeed]], um ihn im Experiment zu
  behalten, damit die nächsten Durchläufe dieselben Zufallswerte ziehen, oder
  [[ui:exp.unpinSeed]], sobald er fixiert ist.

Wenn der [Inspektor](../tools/inspector.md) mitschnitt, ist ein Warteknoten (oder eine
erwartete Antwort), der passte, mit dem Frame verknüpft, auf den er passte; ein Klick
öffnet ihn im Inspektor. Berichte, Startwerte und das Vergleichen von Durchläufen
stehen in [Durchläufe und Berichte](runs.md).

## Fokusmodus und Vollbild {#focus}

▢ [[ui:exp.focus]] verbirgt die Seitenleiste, die Kopfzeile und den unteren Bereich
(Konsole und Inspektor) und überlässt den Bildschirm dem Editor.
⛶ [[ui:exp.fullscreen]] setzt das Fenster ins Vollbild und schaltet den Fokusmodus
ein; das Verlassen des Vollbilds setzt den Fokusmodus auf den vorherigen Zustand
zurück. Der Wechsel zu einer anderen Ansicht verlässt den Fokusmodus.

<kbd>Esc</kbd> auf der Arbeitsfläche verlässt, wenn es sonst nichts zu schließen
gibt, das Vollbild und dann den Fokusmodus.

## Speichern und Dateien {#files}

Das Experiment wird von selbst gespeichert, einen Moment nach jeder Änderung, in
`experiment.json` im Datenordner (`Documents/SignalLab` auf dem Desktop; ein Server
hat seinen eigenen — siehe [Dateien und Ordner](../reference/files.md)). Entwürfe,
die noch nicht laufen können, werden ebenfalls gespeichert. Ein Durchlauf speichert
zuerst, sodass das Gelaufene das auf der Platte ist.

Lässt sich diese Datei nicht lesen — etwa von Hand zu kaputtem JSON bearbeitet —,
sagt der Editor, welche Datei und warum, und lässt sie in Ruhe. Ein anderes Experiment
über ☰ [[ui:exp.documents]] zu öffnen, ersetzt sie dann beim nächsten Speichern.

☰ [[ui:exp.documents]] öffnet den Experimente-Dialog:

- [[ui:exp.templates]]: Wählen Sie eine und drücken Sie [[ui:exp.openDocument]]. Sie
  alle verwenden Loopback-Adressen.
- [[ui:exp.importJson]] liest eine Experimentdatei — von dieser Version von Signal Lab
  oder einer früheren geschrieben, bis zu 4 MiB — und zeigt ihren Namen und wie viele
  Knoten und Verbindungen sie hat, bevor Sie sie öffnen. Dateien aus früheren
  Versionen werden beim Öffnen auf den neuesten Stand gebracht. Eine Datei, die sich
  nicht parsen lässt oder kein gültiges Experiment wäre, wird mit dem Grund
  abgelehnt, und das aktuelle Experiment bleibt.
- [[ui:exp.exportJson]] schreibt eine Momentaufnahme des aktuellen Experiments —
  Knoten, Drähte, Positionen, Parameter und Profile, nie Geheimwerte — in eine neue
  Datei in `exports` im Datenordner und zeigt ihren Pfad; auf einem Server mit einem
  [[ui:common.download]]-Link.
- [[ui:exp.openDocument]] ersetzt das aktuelle Experiment durch die Vorlage oder
  Datei. Es führt sie nicht aus, und <kbd>Ctrl</kbd>+<kbd>Z</kbd> bringt das vorherige
  zurück.

| Vorlage | Was sie tut |
| --- | --- |
| [[ui:exp.templateEmpty]] | [[ui:exp.node.start]] und [[ui:exp.node.end]], für Ihren eigenen Ablauf |
| [[ui:exp.templateHttp]] | Ein GET an `http://127.0.0.1:8080/` und eine Prüfung auf Status 200 — das Experiment, mit dem Sie beginnen |
| [[ui:exp.templateBranch]] | Dieselbe Anfrage; bei 200 eine OSC-Nachricht an `127.0.0.1:9000`, sonst eine Verzögerung von 500 ms |
| [[ui:exp.templateParallel]] | Zwei Zweige zugleich — eine Anfrage und eine Logzeile —, vor [[ui:exp.node.end]] zusammengeführt |
| [[ui:exp.templatePingReply]] | Sendet `/ping` mit der id des Durchlaufs an `127.0.0.1:9000` und wartet auf `127.0.0.1:9001` auf `/pong`, das sie zurückbringt |
| [[ui:exp.templatePoll]] | Fragt ein Gerät alle 0,3 s nach `/status`, bis es `ready` antwortet, höchstens zehn Mal |
| [[ui:exp.templateFlaky]] | Eine emulierte API, die zweimal scheitert, bevor sie antwortet, und eine Schleife, die fragt, bis sie es tut |
| [[ui:exp.templateFaults]] | Datagramme an ein emuliertes Gerät durch ein Störungs-Relais, während ein paralleler Zweig das Netzwerk sauber, verlustbehaftet, offline und wieder sauber schaltet |
| [[ui:exp.templateOutage]] | Eine emulierte API, die von einem parallelen Zweig zwei Sekunden lang abgeschaltet wird, und ein Client, der weiter fragt, bis sie wieder antwortet |
| [[ui:exp.templateWsEcho]] | Verbindet sich mit einem Echo-Dienst unter `ws://127.0.0.1:9001/echo`, sendet ein JSON-Ping, erwartet es unverändert zurück und schließt |

Dieselben Dateien laufen ohne den Editor: `signallab run experiment.json` — siehe
[Die Kommandozeile](../automation/cli.md).

## Tastaturkürzel {#shortcuts}

Einzelne Tasten wirken auf der Arbeitsfläche und bleiben unberührt, während Sie in
einem Feld tippen. Auf einem Mac funktioniert im Browser <kbd>Cmd</kbd> dort, wo
<kbd>Ctrl</kbd> geschrieben steht.

| Tasten | Aktion |
| --- | --- |
| <kbd>A</kbd> | Fügt einen Knoten nach dem ausgewählten hinzu, oder mitten in der Ansicht, wenn nichts ausgewählt ist |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:exp.sendNow]] oder [[ui:exp.listenNow]] beim ausgewählten Knoten |
| <kbd>Ctrl</kbd>+<kbd>Z</kbd> | Rückgängig |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd>, <kbd>Ctrl</kbd>+<kbd>Y</kbd> | Wiederholen |
| <kbd>Ctrl</kbd>+<kbd>A</kbd> | Alle Knoten auswählen |
| <kbd>Ctrl</kbd>+<kbd>C</kbd> / <kbd>Ctrl</kbd>+<kbd>X</kbd> / <kbd>Ctrl</kbd>+<kbd>V</kbd> | Die ausgewählten Knoten und die Drähte zwischen ihnen kopieren / ausschneiden / einfügen |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | Die ausgewählten Knoten duplizieren |
| <kbd>Delete</kbd>, <kbd>Backspace</kbd> | Den ausgewählten Draht oder die ausgewählten Knoten entfernen |
| Pfeiltasten (ein Knoten fokussiert) | Verschieben die ausgewählten Knoten um 5 Pixel; mit <kbd>Shift</kbd> um 20 |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | Einen Knoten finden |
| <kbd>Ctrl</kbd>+<kbd>0</kbd> | Den Graphen einpassen |
| <kbd>Ctrl</kbd>+<kbd>1</kbd> | Auf 100 % zoomen |
| <kbd>Ctrl</kbd> + Mausrad | Um den Zeiger zoomen |
| <kbd>Shift</kbd> + Klick, <kbd>Ctrl</kbd> + Klick | Einen Knoten zur Auswahl hinzufügen oder herausnehmen |
| <kbd>Shift</kbd> + Ziehen auf der leeren Arbeitsfläche | Mit einem Rahmen auswählen |
| Doppelklick auf einen Knoten | Seine Felder bearbeiten |
| Doppelklick auf die leere Arbeitsfläche | Dort einen Knoten hinzufügen |
| <kbd>Enter</kbd>, <kbd>Space</kbd> bei einem fokussierten Ausgang | Von ihm aus einen Draht beginnen |
| `{{` oder <kbd>Ctrl</kbd>+<kbd>Space</kbd> in einem Feld | Parameter, Variablen, Geheimnisse und Generatoren vorschlagen |
| <kbd>Esc</kbd> in einem Feld | Zurück zum Knoten auf der Arbeitsfläche |
| <kbd>Esc</kbd> auf der Arbeitsfläche | Das Hinzufügen-Menü schließen; sonst den gerade gezogenen Draht abbrechen; sonst den ausgewählten Draht loslassen; sonst eine Auswahl mehrerer aufheben; sonst das Vollbild verlassen; sonst den Fokusmodus verlassen |

Im Hinzufügen-Menü und in der Suche wählen <kbd>↑</kbd> <kbd>↓</kbd>,
<kbd>Enter</kbd> übernimmt die Wahl und <kbd>Esc</kbd> schließt. Alle Tastaturkürzel
der App stehen in [Tastaturkürzel](../reference/shortcuts.md).

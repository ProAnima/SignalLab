---
title: Signale
description: OSC-, UDP-, HTTP- und MQTT-Nachrichten in einer Bibliothek aus Ordnern behalten und sie erneut senden — in der Ansicht Signale, in jeder Ansicht mit Ctrl+K oder von der Kommandozeile.
---

# Signale

Ein Signal ist eine Nachricht, die Sie benannt und aufbewahrt haben: eine OSC-Nachricht, ein
rohes UDP-Datagramm, eine HTTP-Anfrage oder eine MQTT-Veröffentlichung, mit ihrem Ziel. Sie
bauen es einmal — in der Ansicht [[ui:nav.signals]] oder indem Sie speichern, was Sie gerade
von einer anderen Ansicht gesendet haben — und senden es erneut, wann immer Sie es brauchen,
Byte für Byte gleich.

Die Bibliothek ist eine JSON-Datei, die Sie lesen, von Hand bearbeiten, auf einen anderen
Rechner kopieren oder neben einem Projekt einchecken können. Die Ansicht [[ui:nav.signals]]
zeigt sie als Baum von Ordnern links und die Felder des ausgewählten Signals rechts.

## Was ein Signal senden kann {#transports}

Wählen Sie die Art in [[ui:sig.transport]]. Jede Art hat ihre eigenen Felder:

| [[ui:sig.transport]] | Felder | Was hinausgeht |
| --- | --- | --- |
| [[ui:sig.tr.osc]] | [[ui:common.target]], [[ui:common.address]], [[ui:common.arguments]] | Eine OSC-Nachricht an ein `IP:port` oder `host:port`, von einem frischen UDP-Port. Siehe [OSC](../protocols/osc.md#types) für die Argumenttypen. |
| [[ui:sig.tr.udp]] | [[ui:common.target]], [[ui:sig.payloadKind]] ([[ui:sig.payloadText]] oder [[ui:sig.payloadHex]]), [[ui:sig.payload]] | Ein Datagramm mit genau diesen Bytes. Text geht so hinaus, wie er geschrieben steht, ohne Abschlusszeichen; Hex sind Ziffernpaare wie `de ad be ef` (Leerzeichen und ein Präfix `0x` sind erlaubt). |
| [[ui:sig.tr.http]] | [[ui:sig.method]], [[ui:sig.url]], [[ui:sig.timeout]], [[ui:sig.headers]], [[ui:field.auth]], [[ui:sig.body]] | Eine HTTP-Anfrage. Siehe [HTTP](../protocols/http.md). |
| [[ui:sig.tr.mqtt]] | [[ui:mq.broker]], [[ui:mq.topic]], [[ui:mq.qos]], [[ui:mq.retain]], [[ui:sig.payload]] | Eine Veröffentlichung. Siehe [MQTT](../protocols/mqtt.md). |

Jedes Signal hat außerdem einen [[ui:sig.name]], einen [[ui:sig.group]] und eine
[[ui:sig.note]] — was es bewirken soll und was auf der Gegenseite passen muss.

Die Ziele von OSC- und UDP-Signalen sind `IP:port` oder `host:port` (zum Beispiel
`127.0.0.1:9000`); ein Hostname wird bei jedem Senden des Signals nachgeschlagen. Der
Broker eines MQTT-Signals ist `host:port`; ohne Port ist es `1883`.

Wenn Sie die Art eines Signals ändern, beginnt seine Nachricht wieder bei den Standardwerten
dieser Art. Nur das Ziel bleibt erhalten, und nur zwischen [[ui:sig.tr.osc]] und
[[ui:sig.tr.udp]], wo das Ziel dasselbe bedeutet.

## Ein Signal senden {#send}

Um ein Signal aus der Ansicht [[ui:nav.signals]] zu senden, tun Sie eines davon:

- Wählen Sie es aus und drücken Sie [[ui:sig.fire]].
- Drücken Sie <kbd>Ctrl</kbd>+<kbd>Enter</kbd>, während Sie in seinen Feldern arbeiten.
- Doppelklicken Sie es im Baum.

Jedes Senden schreibt eine Zeile in die Konsole: was wohin ging, die gesendeten Bytes, bei
HTTP Status und Dauer. Ein Fehlschlag (eine abgelehnte Verbindung, ein nicht erreichbarer
Host) ist eine rote Zeile mit dem Grund. Die Zeit des letzten Sendens steht neben den
Schaltflächen.

Ein Signal geht über dieselben Kommandos hinaus wie die Ansichten seines Protokolls, daher
führt der [Inspektor](inspector.md) es unter dem Werkzeug auf, das es gesendet hat, und die
Gegenseite kann es nicht von einem selbst getippten unterscheiden.

Wie jede Art gesendet wird:

| Art | Wie sie hinausgeht |
| --- | --- |
| [[ui:sig.tr.osc]] | Wie die Ansicht [[ui:nav.osc]] eine Nachricht sendet. |
| [[ui:sig.tr.udp]] | Ein Datagramm an das Ziel. |
| [[ui:sig.tr.http]] | Wie die Ansicht [[ui:nav.http]] eine Anfrage sendet, mit ihrem Cookie-Speicher, solange dort [[ui:http.keepCookies]] an ist. Eine abgelehnte Verbindung oder ein Timeout zählt als Fehlschlag, nicht als Status. |
| [[ui:sig.tr.mqtt]] | Solange die Ansicht [[ui:nav.mqtt]] mit dem Broker des Signals verbunden ist, auf dieser Verbindung, mit ihrer Client-ID und ihren Anmeldedaten. Andernfalls — nicht verbunden oder mit einem anderen Broker verbunden — verbindet sich Signal Lab für diese eine Veröffentlichung mit dem Broker des Signals, mit eigener Client-ID, ohne Benutzernamen und mit einer sauberen Sitzung, und trennt danach. |

Eine Verbindung geht zum Broker des Signals, wenn der Host derselbe ist (Groß- und
Kleinschreibung egal) und der Port derselbe ist, wobei `1883` für einen ohne Port
geschriebenen Broker steht. Namen werden nicht nachgeschlagen: `localhost` und `127.0.0.1`
sind hier zwei verschiedene Broker, daher wird ein Signal, das einen nennt, nicht über eine
zum anderen aufgebaute Verbindung gesendet.

::: tip
Ein MQTT-Signal speichert kein Passwort. Um an einen Broker zu veröffentlichen, der eines
verlangt, verbinden Sie sich zuerst in der Ansicht [[ui:nav.mqtt]] mit diesem Broker; das
Signal reitet dann auf dieser Verbindung.
:::

## Von jeder Ansicht senden {#palette}

Drücken Sie in einer beliebigen Ansicht <kbd>Ctrl</kbd>+<kbd>K</kbd>, um die Palette zu
öffnen, tippen Sie einige Buchstaben des Namens, Ordners, Ziels oder der Nachricht eines
Signals und drücken Sie <kbd>Enter</kbd>. Die Palette schließt sich und das Signal wird
gesendet; Sie bleiben auf der Ansicht, die Sie gerade beobachtet haben.

| Taste | Was sie tut |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>K</kbd> | Öffnet die Palette oder schließt sie. |
| <kbd>↑</kbd> <kbd>↓</kbd> | Bewegt die Auswahl. |
| <kbd>Enter</kbd> | Sendet das gewählte Signal. |
| <kbd>Esc</kbd> | Schließt die Palette, ohne zu senden. |

Die Palette listet höchstens 12 Signale auf: die ersten 12 der Bibliothek, solange Sie nichts
getippt haben, danach die ersten 12, die passen. Ein Klick auf eine Zeile sendet es; ein
Klick daneben schließt die Palette.

## Signale anlegen {#create}

### In der Ansicht Signale {#create-here}

1. Wählen Sie den Ordner, in den das Signal gehört (siehe [aktueller Ordner](#current-folder)).
2. Drücken Sie [[ui:sig.new]]. Ein neues OSC-Signal an `127.0.0.1:9000`, Adresse
   `/hello`, erscheint in diesem Ordner, ausgewählt.
3. Ändern Sie [[ui:sig.name]], [[ui:sig.transport]] und die Felder der Nachricht.

Jede Änderung wird von selbst gespeichert; es gibt keine Schaltfläche zum Speichern in dieser
Ansicht. Solange die Bibliotheksdatei nicht gelesen werden kann, wird nichts gespeichert und
die Felder sind schreibgeschützt (siehe [Die Bibliotheksdatei](#file)).

[[ui:sig.duplicate]] legt eine Kopie direkt nach dem ausgewählten Signal an, sein Name
gefolgt von `·`. [[ui:sig.delete]] fragt noch einmal nach ([[ui:sig.confirmDelete]]): Der
zweite Klick entfernt es aus der Datei. Sein Ordner bleibt, auch wenn er nun leer ist.

### Von den Ansichten HTTP, OSC und MQTT {#save-from-screens}

Der sendende Teil dreier Ansichten — [[ui:http.request]] in [[ui:nav.http]],
[[ui:osc.sender]] in [[ui:nav.osc]], [[ui:mq.publish]] in [[ui:nav.mqtt]] — kann das, was er
senden würde, als Signal aufbewahren.

1. Richten Sie die Nachricht ein und senden Sie sie, bis sie tut, was Sie wollen.
2. Drücken Sie [[ui:sig.saveNew]] (oder <kbd>Ctrl</kbd>+<kbd>S</kbd> in diesem Teil der
   Ansicht). Der Dialog [[ui:sig.saveTitle]] öffnet sich.
3. Prüfen Sie [[ui:sig.name]]: Er wird aus dem vorgeschlagen, was gesendet wird.
4. Wählen oder tippen Sie einen [[ui:sig.group]]. Der zuletzt verwendete Ordner ist
   eingetragen; ein Pfad wie `Venue/Stage`, den es noch nicht gibt, wird angelegt.
5. Drücken Sie [[ui:sig.saveConfirm]].

Von da an ist die Ansicht an dieses Signal gebunden. Ein Chip neben den Schaltflächen sagt,
wo es liegt (`❖ Folder / Name`); klicken Sie darauf, um das Signal in der Ansicht
[[ui:nav.signals]] zu sehen.

| Sie sehen | Es bedeutet | Was Sie tun können |
| --- | --- | --- |
| ✓ [[ui:sig.savedState]] (ausgegraut) | Die Bibliothek enthält genau das, was die Ansicht senden würde. | Nichts zu speichern. |
| [[ui:sig.save]] und [[ui:sig.changed]] am Chip | Die Nachricht der Ansicht unterscheidet sich vom Signal. | [[ui:sig.save]] oder <kbd>Ctrl</kbd>+<kbd>S</kbd> schreibt die Nachricht der Ansicht in dieses Signal; Name, Ordner und Notiz bleiben. |
| [[ui:sig.saveAs]] | — | Öffnet den Dialog erneut, gefüllt mit Name und Ordner des Signals, und speichert ein neues Signal. Die Ansicht ist dann an das neue gebunden. |

Der Vergleich betrachtet die Nachricht, nicht, wie sie geschrieben ist: Die Reihenfolge der
JSON-Schlüssel und die letzten Stellen eines OSC-Floats jenseits der 32-Bit-Präzision zählen
nicht als Änderung.

Die Ansichten [[ui:nav.http]] und [[ui:nav.osc]] behalten die Bindung, wenn Signal Lab neu
startet; die Ansicht [[ui:nav.mqtt]] behält sie, bis Sie die App schließen.

::: warning
Ein HTTP-Signal bewahrt seine [[ui:field.auth]] — Benutzername und Passwort oder Token — als
Klartext in der Bibliotheksdatei auf. Wer die Datei lesen kann, kann sie lesen.
:::

### Ein Signal in seiner Ansicht öffnen {#open-in-screen}

Ein ausgewähltes HTTP-, OSC- oder MQTT-Signal hat eine Schaltfläche, die es in der Ansicht
seines Protokolls öffnet ([[ui:nav.http]], [[ui:nav.osc]] oder [[ui:nav.mqtt]]). Die Felder
der Ansicht werden aus dem Signal gefüllt und die Ansicht wird an es gebunden, wie oben: dort
bearbeiten, senden, dann [[ui:sig.save]]. Ein rohes UDP-Signal hat keine eigene Ansicht.

### Aus einem Frame oder einem Topic {#capture}

- Wählen Sie im [Inspektor](inspector.md#save-as-signal) einen Frame aus und drücken Sie
  [[ui:sig.fromFrame]]. Ein Datagramm wird zu einem rohen UDP-Signal mit den exakten Bytes
  dieses Frames; eine MQTT-Veröffentlichung wird zu einem MQTT-Signal mit demselben Broker,
  Topic, QoS, Retain-Flag und denselben Nutzdaten. Andere Frames lassen sich nicht speichern.
- Wählen Sie in der Ansicht [[ui:nav.mqtt]] ein Topic aus und drücken Sie [[ui:sig.fromFrame]].
  Sie erhalten ein MQTT-Signal, das den letzten Wert des Topics, mit seinem QoS und
  Retain-Flag, an den Broker veröffentlicht, mit dem Sie verbunden sind.

Beide landen im Ordner [[ui:sig.capturedFolder]] und werden nach dem benannt, was
mitgeschnitten wurde.

### In einem Experiment {#in-experiments}

Wenn Sie einem Experiment einen Knoten hinzufügen, listet das Menü [[ui:exp.addNode]] Ihre
Signale auch unter [[ui:exp.group.signals]] auf. Wählen Sie eines, wird ein OSC-, HTTP-,
MQTT- oder UDP-Knoten mit derselben Nachricht hinzugefügt. Ein rohes UDP-Signal mit
Hex-Nutzdaten wird nicht angeboten: Der UDP-Knoten sendet Text. Siehe
[Knoten](../experiments/nodes.md).

## Ordner {#folders}

Ein Ordner ist ein Pfad aus Namen, verbunden durch `/`: `API/Auth` ist der Ordner `Auth`
innerhalb von `API`. Das Feld [[ui:sig.group]] eines Signals enthält den Pfad seines Ordners;
leer bedeutet die oberste Ebene ([[ui:sig.topLevel]]). Namen werden getrimmt und leere Teile
verworfen, wenn Sie das Feld verlassen, sodass aus ` API / Auth/ ` `API/Auth` wird.

Ordner werden nach Namen sortiert, Zahlen in numerischer Reihenfolge (`Cue 2` vor `Cue 10`);
Signale bleiben in der Reihenfolge der Datei. Jeder Ordner zeigt, wie viele Signale er
enthält, seine Unterordner eingeschlossen. Ein leerer Ordner bleibt, bis Sie ihn entfernen.

### Der aktuelle Ordner {#current-folder}

Der zuletzt angeklickte Ordner oder der Ordner des ausgewählten Signals ist der aktuelle:
[[ui:sig.new]] und [[ui:sig.newFolder]] legen Dinge dort ab. Ein Chip über dem Baum nennt
ihn; klicken Sie auf den Chip, um zur obersten Ebene zurückzukehren.

### Mit Ordnern arbeiten {#folder-tasks}

| Um | Tun Sie dies |
| --- | --- |
| Einen Ordner anlegen | Drücken Sie ＋ [[ui:sig.newFolder]]. Er wird im aktuellen Ordner angelegt, heißt [[ui:sig.newFolderName]] (mit einer Zahl dahinter, wenn der Name vergeben ist), und Sie benennen ihn sofort um. |
| Einen Ordner öffnen oder schließen | Klicken Sie ihn an oder drücken Sie <kbd>→</kbd> / <kbd>←</kbd>, während er den Fokus hat. Signal Lab merkt sich, welche Ordner geschlossen sind. |
| Alle öffnen oder schließen | Die Schaltflächen ⊞ und ⊟ über dem Baum ([[ui:sig.expandAll]], [[ui:sig.collapseAll]]). |
| Einen Ordner umbenennen | Drücken Sie ✎ ([[ui:sig.renameFolder]]) oder <kbd>F2</kbd> darauf, tippen Sie, dann <kbd>Enter</kbd>; <kbd>Esc</kbd> bricht ab. |
| Ein Signal oder einen Ordner verschieben | Ziehen Sie es auf einen Ordner oder auf leeren Raum im Baum für die oberste Ebene. |
| Ein Signal durch Tippen verschieben | Ändern Sie sein Feld [[ui:sig.group]]. |
| Einen Ordner entfernen | Drücken Sie × ([[ui:sig.removeFolder]]) und dann [[ui:sig.confirmRemoveFolder]], oder drücken Sie zweimal <kbd>Delete</kbd> darauf. |

Ein Umbenennen führt niemals zwei Ordner zusammen: Ein Name mit `/` darin oder einer, den
ein benachbarter Ordner bereits hat, wird abgelehnt, und die Konsole sagt es. Ein Ordner kann
nicht in sich selbst oder in einen Ordner innerhalb seiner gezogen werden. Wird ein Ordner in
einen Ordner gezogen, der bereits einen mit demselben Namen enthält, werden die beiden
zusammengeführt.

Das Entfernen eines Ordners entfernt nur den Ordner: Seine Signale und Unterordner rücken
eine Ebene nach oben. Nichts wird gelöscht.

### Ein Signal finden {#filter}

Tippen Sie in [[ui:sig.search]] über dem Baum. Es passt auf Name, Ordner, Notiz, Ziel und
Nachricht. Während Sie filtern, ist jeder Ordner mit einem Treffer geöffnet und die anderen
sind ausgeblendet.

## Die Beispielsammlung {#starter-set}

Findet Signal Lab zum ersten Mal keine Bibliotheksdatei, schreibt es neun Beispiele, jedes
über etwas, das leicht falsch zu machen ist. Ihre Namen und Notizen werden in der Sprache
geschrieben, die die Oberfläche in diesem Moment hat; danach sind sie Ihre zum Ändern. Alle
zeigen auf diesen Computer.

| Ordner | Signal | Sendet |
| --- | --- | --- |
| `OSC` | [[ui:seed.osc-fader.name]] | `/fader/1` mit dem Float `0.75` an `127.0.0.1:9000` |
| `OSC` | [[ui:seed.osc-types.name]] | `/types` mit int `-7`, float `1.5`, string `hi`, bool true, int64 `4294967296`, double `0.125` und nil |
| `OSC` | [[ui:seed.osc-id-and-value.name]] | `/tag` mit den Strings `reader-1` und `04a1b2c3` |
| `OSC` | [[ui:seed.osc-trigger.name]] | `/cue/go` ohne Argumente |
| `MQTT` | [[ui:seed.mqtt-publish.name]] | `1` an `lab/example/value` auf `127.0.0.1:1883`, QoS 0 |
| `MQTT` | [[ui:seed.mqtt-retained.name]] | `night` an `lab/example/config`, QoS 1, retained |
| `MQTT` | [[ui:seed.mqtt-clear-retained.name]] | Leere Retained-Nutzdaten an `lab/example/config`, QoS 1 |
| `HTTP` | [[ui:seed.http-reachable.name]] | `GET http://127.0.0.1:8080/`, Timeout 4000 ms |
| [[ui:seed.folder.Raw]] | [[ui:seed.udp-raw.name]] | Die Bytes `de ad be ef` an `127.0.0.1:9000` |

Um die Beispielsammlung zurückzubekommen, verschieben oder benennen Sie `signals.json` um und
drücken Sie [[ui:sig.reload]]: Ohne Datei dort wird sie erneut geschrieben.

## Die Bibliotheksdatei {#file}

Die Bibliothek ist `signals.json` im Datenordner: `Documents/SignalLab` in Ihrem
Benutzerordner auf einem Desktop oder der Datenordner des Servers (siehe
[Dateien](../reference/files.md)). Fahren Sie mit der Maus über die Signalanzahl unter dem
Baum, um den ganzen Pfad zu sehen.

- **Von selbst gespeichert.** Jede Änderung wird 0,7 s nach der letzten geschrieben, die
  ganze Datei auf einmal, über eine temporäre Datei im selben Ordner, die dann den Platz der
  Datei einnimmt — ein abgebrochener Schreibvorgang hinterlässt die vorherige Datei. Während
  ein Schreibvorgang wartet, sagt der Fuß des Baums [[ui:sig.saving]]; dann
  [[ui:sig.saved]]. Ein noch wartender Schreibvorgang wird vor einem Neustart der App durch
  ein Update ausgeführt.
- **Von Hand bearbeitet.** Signal Lab bemerkt nicht, wenn sich die Datei unter ihm ändert.
  Nachdem Sie sie bearbeitet oder durch eine von einem anderen Rechner ersetzt haben, drücken
  Sie [[ui:sig.reload]]. Ein Neuladen liest die Datei erneut und verwirft eine Änderung, die
  noch auf das Schreiben wartete.
- **Niemals ersetzt, solange sie kaputt ist.** Ist die Datei kein gültiges JSON oder keine
  Signalbibliothek, zeigt der Baum den Fehler mit Pfad, Zeile und Spalte der Datei, und die
  Konsole sagt dasselbe. Die Datei bleibt, wie sie ist, und nichts schreibt die Bibliothek,
  bis sie wieder liest: [[ui:sig.new]], das Umbenennen, Verschieben und Entfernen von Signalen
  und Ordnern, das Ziehen, die Felder eines Signals, [[ui:sig.saveNew]], [[ui:sig.save]] und
  [[ui:sig.saveAs]] in den Ansichten HTTP, OSC und MQTT sowie [[ui:sig.fromFrame]] im
  Inspektor und in der Ansicht MQTT sind alle aus, und ihr Hinweis sagt, was falsch ist.
  Korrigieren Sie die Datei oder entfernen Sie sie und drücken Sie [[ui:sig.reload]]: Sobald
  sie liest, funktioniert alles wieder.
- **Eine Datei, die kaputtgeht, während die App läuft.** Wenn Sie die Datei in etwas
  Unlesbares bearbeiten und die App dann eine Änderung speichert, wird das Speichern mit
  demselben Fehler abgelehnt, die Datei bleibt, wie Sie sie gemacht haben, und die App hört
  auf zu schreiben, bis Sie sie korrigieren und [[ui:sig.reload]] drücken. Eine Datei, die
  Sie bearbeitet und gültig gelassen haben, wird bei ihrem nächsten Speichern durch die Liste
  der App ersetzt, wie oben: zuerst neu laden.

Ein kurzes Beispiel der Datei:

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

| Schlüssel | Was |
| --- | --- |
| `version` | `2`. Eine Datei der Version 1 (vor den Ordnern) liest sich genauso, ohne leere Ordner. |
| `signals[].id` | Wird beim Anlegen des Signals aus dem Namen gemacht (`fader-value`, `fader-value-2`, …) und von einem Umbenennen nie geändert. `signallab fire` findet ein Signal darüber. |
| `signals[].group` | Der Ordnerpfad; `""` ist die oberste Ebene. |
| `signals[].body` | Die Nachricht. `transport` ist `osc`, `udp`, `http` oder `mqtt`; die anderen Schlüssel sind die Felder dieser Art. |
| `folders` | Jeder Ordner, damit ein leerer erhalten bleibt. Entfällt, wenn es keine gibt. Eine `group`, die kein Eintrag listet, ist ebenfalls ein Ordner. |

## Von der Kommandozeile {#cli}

`signallab fire` sendet ein Signal einer Bibliothek, über dieselben Kommandos wie die App:

```bash
signallab fire "Fader value"
signallab fire fader-value --library ./show/signals.json
```

Es findet das Signal zuerst über seine id, dann über seinen Namen, Groß- und Kleinschreibung
egal. Wenn mehrere Signale diesen Namen haben, nennt es ihre ids und sendet nichts. Ohne
`--library` liest es die eigene `signals.json` der App; es schreibt die Datei nie. Siehe
[Die Kommandozeile](../automation/cli.md#cli-fire).

## Verwandte Seiten {#related}

- [Inspektor](inspector.md) — beobachten, was ein Signal sendet, und einen mitgeschnittenen
  Frame als Signal aufbewahren.
- [Tastaturkürzel](../reference/shortcuts.md)
- [Dateien](../reference/files.md) — wo der Datenordner liegt.

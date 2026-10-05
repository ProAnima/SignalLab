---
title: Erste Schritte
description: Eine erste Sitzung auf einem Computer — eine OSC-Nachricht senden und ihr Eintreffen beobachten, sie als Signal speichern, eine emulierte API befragen und ein kleines Experiment bauen und ausführen.
---

# Erste Schritte

Für diese Sitzung brauchen Sie nur Signal Lab: Alles geht an `127.0.0.1`, diesen
Computer, daher sind weder Gerät noch Netzwerk noch Firewall-Regel beteiligt. Sie werden:

1. eine OSC-Nachricht senden und ihr Eintreffen beobachten;
2. dieselbe Nachricht im Inspektor sehen;
3. sie in der Bibliothek speichern und von überall erneut senden;
4. eine emulierte HTTP-API starten und ihr eine Anfrage stellen;
5. ein Experiment gegen diese API ausführen, lesen, warum es fehlschlägt, es korrigieren und eine Prüfung hinzufügen.

Falls Sie Signal Lab noch nicht installiert haben, siehe [Installieren und aktualisieren](install.md).
Unsicher, wo im Fenster etwas zu finden ist? Siehe [Das Fenster](interface.md).

## Eine OSC-Nachricht senden und ihr Eintreffen beobachten {#osc}

Zuerst etwas, das die Nachricht empfängt: der Monitor der Ansicht OSC.

1. Öffnen Sie in der Seitenleiste [[ui:nav.osc]].
2. Setzen Sie unter [[ui:osc.monitor]] das Feld [[ui:common.bind]] auf `127.0.0.1:9000`, damit der Monitor
   nur auf diesem Computer empfängt.
3. Drücken Sie [[ui:osc.listen]]. Die Schaltfläche wird zu [[ui:common.stop]], die Konsole meldet, dass der
   Monitor empfängt, und der Monitor erscheint als Job in der Leiste des unteren Bereichs.

Jetzt die Nachricht, vom Sender daneben:

4. Lassen Sie unter [[ui:osc.sender]] das Feld [[ui:common.target]] auf `127.0.0.1:9000`, dem Port, auf dem der
   Monitor empfängt.
5. Lassen Sie [[ui:common.address]] auf `/hello/avatar/1` und das eine Float-Argument unter
   [[ui:common.arguments]] auf `1.0` — oder geben Sie eine eigene Adresse und eigene Werte ein.
6. Drücken Sie [[ui:common.send]] oder <kbd>Enter</kbd> im Feld für Ziel oder Adresse.

In der Tabelle des Monitors erscheint eine Zeile: [[ui:common.time]] des Eintreffens,
[[ui:osc.from]] (`127.0.0.1` und der Port, von dem gesendet wurde), [[ui:osc.address]] und
[[ui:osc.args]]. Unter dem Sender bestätigt eine Zeile, was gesendet wurde und wie groß es in
Bytes ist; senden Sie erneut, zählt sie die Wiederholungen.

::: tip Ein Hinweis zur Firewall?
Unter Windows kann beim Start des Monitors unter der Kopfzeile ein Hinweis zur Windows-Firewall
erscheinen. Er betrifft Nachrichten von *anderen* Rechnern; Verkehr auf `127.0.0.1` wird nie
gefiltert. Drücken Sie vorerst [[ui:fw.dismiss]] — [Der Firewall-Hinweis](interface.md#firewall-notice)
erklärt, wann Sie zulassen sollten.
:::

## Im Inspektor ansehen {#inspector}

Der Inspektor zeichnet jeden Frame auf, den ein Werkzeug sendet und empfängt — aber nur, solange
der Mitschnitt läuft.

1. Öffnen Sie im unteren Bereich die Registerkarte [[ui:dock.inspector]].
2. Drücken Sie [[ui:ins.arm]]. Der Punkt der Registerkarte leuchtet auf.
3. Drücken Sie im Sender noch einmal [[ui:common.send]].

Zwei Zeilen erscheinen, die neueste zuerst: die Nachricht, wie sie gesendet wurde (→) und wie der Monitor sie empfangen hat
(←), jeweils mit Protokoll, Adresse der Gegenseite, Größe und einer Zusammenfassung. Klicken Sie auf eine:
[[ui:ins.detail]] zeigt, welches Werkzeug sie gesendet oder empfangen hat und über welche Adressen, unter
[[ui:ins.decoded]] die dekodierte Nachricht und unter [[ui:ins.rawBytes]] die Bytes, aus denen sie bestand.

Drücken Sie [[ui:ins.disarm]], wenn Sie fertig sind; ausgeschaltet kostet der Mitschnitt nichts. Mehr
unter [Inspektor](../tools/inspector.md).

## Als Signal speichern und erneut senden {#signal}

Eine Nachricht, die Sie wieder brauchen werden, gehört in die Signalbibliothek.

1. Drücken Sie in der Ansicht OSC unter dem Sender [[ui:sig.saveNew]].
2. Setzen Sie im Dialog [[ui:sig.saveTitle]] das Feld [[ui:sig.name]] auf `First message` und
   [[ui:sig.group]] auf `Tutorial` — ein neuer Ordner entsteht, sobald Sie hinein speichern.
3. Drücken Sie [[ui:sig.saveConfirm]].

Der Sender ist jetzt an dieses Signal gebunden: Die Schaltfläche zeigt [[ui:sig.savedState]], und ein Chip
daneben zeigt, wo das Signal liegt. Ändern Sie das Argument, vermerkt der Chip die
Änderung; [[ui:sig.save]] (<kbd>Ctrl</kbd>+<kbd>S</kbd>) würde das Signal aktualisieren.

Senden Sie es jetzt erneut, auf drei Arten:

- **Aus der Bibliothek.** Klicken Sie auf den Chip: Die Ansicht [[ui:nav.signals]] öffnet sich mit dem Signal
  ausgewählt im Ordner `Tutorial` (oder öffnen Sie [[ui:nav.signals]] und klicken Sie es dort an).
  Drücken Sie [[ui:sig.fire]] oder <kbd>Ctrl</kbd>+<kbd>Enter</kbd>; auch ein Doppelklick darauf in der
  Liste sendet es.
- **Von überall.** Drücken Sie in einer beliebigen Ansicht <kbd>Ctrl</kbd>+<kbd>K</kbd>, tippen Sie `first`
  und drücken Sie <kbd>Enter</kbd>.
- **Aus einem Experiment.** Wenn Sie einen Knoten hinzufügen, listet das Menü Ihre Signale unter
  [[ui:exp.group.signals]] auf, bereit, zu einem Schritt zu werden, der eines davon sendet.

Jedes Mal zeigt der Monitor die eintreffende Nachricht, und die Konsole nennt das Signal. Ein
Signal sendet genau das, was seine Ansicht gesendet hätte. Mehr unter [Signale](../tools/signals.md).

Wenn Sie mit OSC fertig sind, drücken Sie beim Monitor [[ui:common.stop]].

## Eine emulierte API befragen {#emulator}

Signal Lab bringt fünf Emulatoren mit, alle auf `127.0.0.1`. Einer davon,
[[ui:seed.emu.demo-api.name]], ist eine HTTP-API auf `127.0.0.1:8080` mit diesen Routen:

| Anfrage | Antwort |
| --- | --- |
| `GET /health` | `200` mit `{"status":"ok","time":"…"}` — der aktuellen Uhrzeit |
| `GET /users/:id` | `200` mit dem Benutzer dieser ID, etwa `{"id":"42","name":"User 42"}` |
| `POST /users` | `201` mit einem `Location`-Header und der neuen ID |
| `GET /slow` | `200` nach 1,5 Sekunden |
| beliebige Methode, `/flaky` | `503`, `503`, dann ab der dritten Anfrage `200` |
| alles andere | `404` |

1. Öffnen Sie [[ui:nav.emulators]]. Die [[ui:emu.library]] listet die fünf auf; wählen Sie
   [[ui:seed.emu.demo-api.name]].
2. Drücken Sie [[ui:emu.start]]. Der Emulator antwortet jetzt auf `127.0.0.1:8080` und läuft als Job.
3. Öffnen Sie [[ui:nav.http]]. Die Methode ist `GET`; setzen Sie die URL auf
   `http://127.0.0.1:8080/health`.
4. Drücken Sie [[ui:common.send]] oder <kbd>Enter</kbd> in der URL.

Unter [[ui:http.response]] sehen Sie [[ui:http.status]] `200`,
[[ui:http.latency]], [[ui:http.size]], die Antwort-Header und den JSON-Body. Senden Sie
dreimal an `http://127.0.0.1:8080/flaky`: zwei Antworten `503`, dann `200` — so sieht ein
Dienst, der sich erholt, für einen Client aus, der es erneut versucht.

Zurück in [[ui:nav.emulators]] zählt der Bereich [[ui:emu.live]] jede Anfrage, und
[[ui:emu.received]] listet jede einzeln auf, mit der [[ui:emu.col.rule]], die sie beantwortet hat, und der
[[ui:emu.col.reply]]. Lassen Sie die [[ui:seed.emu.demo-api.name]] für den nächsten Teil laufen.
Mehr unter [Emulatoren](../tools/emulators.md).

## Ein Experiment ausführen {#experiment}

Ein Experiment ist ein Ablauf von Schritten, den Sie immer wieder ausführen können. Das Experiment, mit dem Signal Lab
beim ersten Mal öffnet — die Vorlage [[ui:exp.templateHttp]] —, sendet eine Anfrage an
`http://127.0.0.1:8080/` und prüft, ob die Antwort `200` ist.

### Die Vorlage öffnen {#open-template}

1. Öffnen Sie [[ui:nav.experiment]].
2. Zeigt die Arbeitsfläche nicht vier Knoten — [[ui:exp.node.start]],
   [[ui:exp.node.http]], [[ui:exp.node.assert_status]], [[ui:exp.node.end]] —, drücken Sie
   **☰** links in der Werkzeugleiste ([[ui:exp.documents]]), wählen Sie in der Liste der Vorlagen
   [[ui:exp.templateHttp]] und drücken Sie
   [[ui:exp.openDocument]]. Das Öffnen ersetzt das Experiment auf der Arbeitsfläche;
   <kbd>Ctrl</kbd>+<kbd>Z</kbd> holt das vorherige zurück.

Klicken Sie auf einen Knoten, um seine Einstellungen rechts unter [[ui:exp.properties]] zu sehen. Experimente speichern
sich beim Bearbeiten von selbst.

### Ausführen und lesen, warum es fehlschlägt {#first-run}

3. Drücken Sie [[ui:exp.run]].

Unter der Arbeitsfläche öffnet sich die [[ui:exp.timeline]], mit einer Zeile pro Schritt, wenn er beginnt
([[ui:exp.running]]), und einer weiteren, wenn er endet: die Zeit, der Knoten und wie es ausging. Dieser Durchlauf
schlägt fehl:

- [[ui:exp.node.start]] besteht und nennt den Startwert des Durchlaufs.
- [[ui:exp.node.http]] besteht: Die Anfrage ging hinaus, und eine Antwort kam zurück,
  `HTTP 404`.
- [[ui:exp.node.assert_status]] schlägt fehl: Erwartet war `200`, empfangen wurde `404`.

Die [[ui:seed.emu.demo-api.name]] hat keine Route für `/` und antwortete daher mit `404` — und die
Prüfung hat es bemerkt. Die Zeile oben in der Zeitleiste zeigt [[ui:exp.failed]] und den Grund.
Klicken Sie auf eine Zeile, um ihren Knoten auf der Arbeitsfläche auszuwählen.

::: tip Die Anfrage selbst ist fehlgeschlagen?
Schlägt der Schritt [[ui:exp.node.http]] mit einer abgelehnten Verbindung fehl, empfängt nichts auf
`127.0.0.1:8080`: Starten Sie die [[ui:seed.emu.demo-api.name]] in der Ansicht [[ui:nav.emulators]] und führen Sie das Experiment
erneut aus.
:::

### Die Anfrage korrigieren {#fix}

4. Klicken Sie auf den Knoten [[ui:exp.node.http]].
5. Ändern Sie unter [[ui:exp.properties]] das Feld [[ui:sig.url]] in `http://127.0.0.1:8080/health`.
6. Drücken Sie [[ui:exp.run]].

Diesmal besteht jeder Schritt: [[ui:exp.node.assert_status]] meldet
[[ui:exp.step.checked]], [[ui:exp.node.end]] meldet [[ui:exp.step.complete]], und der
Titel der Zeitleiste zeigt [[ui:exp.passed]].

### Eine Prüfung hinzufügen {#add-check}

Ein Status `200` sagt, dass der Dienst geantwortet hat, aber nicht, was er geantwortet hat. Prüfen Sie
auch den Body:

7. Klicken Sie auf den Knoten [[ui:exp.node.assert_status]].
8. Drücken Sie unter [[ui:exp.properties]] auf [[ui:exp.addNext]] — oder drücken Sie <kbd>A</kbd>, während die
   Arbeitsfläche den Fokus hat. Ein Menü der Knoten öffnet sich, mit einem Suchfeld.
9. Tippen Sie `assert_body` und drücken Sie <kbd>Enter</kbd>. Ein Knoten [[ui:exp.node.assert_body]] wird
   zwischen [[ui:exp.node.assert_status]] und [[ui:exp.node.end]] eingefügt, bereits verbunden,
   und sein Feld [[ui:exp.contains]] ist bereit für die Eingabe.
10. Tippen Sie `"status":"ok"`.
11. Drücken Sie [[ui:exp.run]].

Der neue Schritt besteht. Ändern Sie den Text in etwas, das der Body nicht enthält, und führen Sie
erneut aus, um ihn mit Begründung fehlschlagen zu sehen.

### Was ein Durchlauf hinterlässt {#report}

- **Einen Bericht.** Endet ein Durchlauf, erscheint im Titel der Zeitleiste [[ui:exp.reportSaved]];
  fahren Sie mit der Maus darüber, um die Datei zu sehen. Jeder Durchlauf, der endet, ob bestanden oder fehlgeschlagen, schreibt einen in den
  Ordner `runs` Ihres Datenordners, mit den verwendeten Werten und jedem Schritt. Im
  Browser ist es ein Download-Link.
- **Einen Startwert.** Der Titel zeigt außerdem den Startwert des Durchlaufs mit [[ui:exp.pinSeed]]: Zufallswerte
  eines Durchlaufs folgen seinem Startwert, und wenn Sie ihn fixieren, wiederholen sie sich exakt.

Mehr unter [Durchläufe und Berichte](../experiments/runs.md).

## Aufräumen {#clean-up}

Drücken Sie in der Kopfzeile [[ui:app.stopAll]]: Das stoppt die [[ui:seed.emu.demo-api.name]] und
alles andere, was noch läuft. Ihr Signal, das Experiment und seine Berichte bleiben in Ihrem
Datenordner.

## Wie es weitergeht {#next}

- [Konzepte](concepts.md): die Ideen hinter Ansichten, Signalen, Jobs, Emulatoren und
  Experimenten.
- [Experimente](../experiments/index.md): der Editor im Ganzen, und jede Art von Knoten unter
  [Knoten](../experiments/nodes.md).
- [OSC](../protocols/osc.md), [HTTP](../protocols/http.md) und die Seiten der anderen
  Protokolle, wenn Sie Signal Lab auf echte Technik richten.
- [Die Kommandozeile](../automation/cli.md): dasselbe Experiment aus einem Terminal oder
  einer Pipeline ausführen.

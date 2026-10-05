---
title: Durchläufe und Ergebnisse
description: Einen Durchlauf mit den Werten des Experiments oder anderen starten, die Zeitleiste, Stoppen, was bestanden und fehlgeschlagen bedeuten, der Bericht des Durchlaufs, Startwerte, einen einzelnen Knoten ausprobieren und Experimentdateien mit ihren Versionen.
---

# Durchläufe und Ergebnisse

## Einen Durchlauf starten {#start}

Drücken Sie [[ui:exp.run]] in der Werkzeugleiste des Editors. Bevor etwas gesendet
wird:

1. Das Experiment wird geprüft, wie der Editor es prüft — sein Graph, Felder, Namen
   und Werte ([was geprüft wird](flow.md#validation)) —, und jedes Geheimnis, das es
   verwendet, muss gespeichert sein ([Geheimnisse](data.md#secret-check)).
2. Es wird gespeichert.
3. Der Durchlauf öffnet, was er für seine ganze Länge braucht: seine Emulatoren,
   seine Störungs-Relais, die Sockets, auf denen seine Warteknoten lauschen, und
   seine MQTT-Abonnements.

Schlägt etwas davon fehl, läuft nichts: Das Problem wird gezeigt, und der Knoten, um
den es geht, wird ausgewählt. Andernfalls öffnet sich die Zeitleiste unter der
Arbeitsfläche, und die Schritte erscheinen darin, wie sie geschehen. Während der
Durchlauf läuft, wird [[ui:exp.run]] zu [[ui:common.stop]], und das Experiment lässt
sich nicht bearbeiten.

Der Durchlauf verwendet die Werte des aktiven Profils und den im Experiment fixierten
Startwert oder einen neuen. Um einmal mit anderen auszuführen, verwenden Sie
[[ui:exp.runWith]].

### Mit anderen Werten ausführen {#run-with}

Das ▾ neben [[ui:exp.run]] öffnet [[ui:exp.runWith]]: andere Werte für einen
Durchlauf, ohne das Experiment zu ändern.

| Feld | Was | Leer |
| --- | --- | --- |
| [[ui:exp.profile]] | das Profil für diesen Durchlauf; angezeigt, wenn das Experiment Profile hat | das aktive |
| jeder Parameter | ein Wert nur für diesen Durchlauf | der Wert des gewählten Profils, grau angezeigt |
| [[ui:exp.seed]] | der Startwert für diesen Durchlauf, 0–9 007 199 254 740 991; die Schaltfläche daneben füllt den Startwert des letzten Durchlaufs ein | der fixierte Startwert oder ein neuer |

[[ui:exp.run]] im Formular startet den Durchlauf; [[ui:exp.resetOverrides]] leert das
Formular. Was Sie getippt haben, bleibt für die Sitzung im Formular, sodass dieselbe
Änderung beim nächsten Mal ein Klick ist. Ein Profil, das nicht laufen würde, ist mit
⚠ markiert. Die Priorität der Werte steht in [Daten](data.md#precedence).

Hat das Experiment Profile oder wurden für einen Durchlauf Werte getippt, sagt die
Zeitleiste, welches Profil der Durchlauf verwendet hat — oder
[[ui:exp.runDefaults]] —, und, wenn Werte getippt wurden, [[ui:exp.overridden]].

## Die Zeitleiste {#timeline}

[[ui:exp.timeline]] liegt unter der Arbeitsfläche; ▸ und ▾ falten sie, ihr Rand ändert
ihre Größe. Sie enthält eine Zeile pro Schrittereignis, die älteste zuerst: die Zeit,
der Knoten, sein Zustand und was geschah — `HTTP 200 · 41 ms`, `token = abc123`,
`/pong 42 ← 127.0.0.1:9000 · 12 ms`. Ein Fehler sagt, warum, sein technisches Detail
im Tooltip. Ein Klick auf eine Zeile wählt ihren Knoten auf der Arbeitsfläche aus.

| Zustand | Der Schritt |
| --- | --- |
| [[ui:exp.running]] | hat begonnen |
| [[ui:exp.passed]] | endete gut und wählte seinen Ausgang |
| [[ui:exp.failed]] | schlug fehl; der erste Fehler ist der des Durchlaufs |
| [[ui:exp.retry]] | schlug einen Versuch lang fehl und versucht es erneut ([Neuversuch](flow.md#retry)) |
| [[ui:exp.repeating]] | sendet immer wieder, höchstens eine Zeile pro Sekunde ([Wiederholung](flow.md#repeat)) |
| [[ui:exp.load]] | ist unter Last, eine Zeile pro Sekunde ([Last](load.md#progress)) |

Auf der Arbeitsfläche trägt jeder Knoten eine Plakette mit seinem neuesten Zustand.

Der Kopf der Zeitleiste enthält:

- das Ergebnis: [[ui:exp.running]], [[ui:exp.passed]], [[ui:exp.failed]] mit dem
  Grund oder [[ui:exp.stopped]];
- [[ui:exp.reportSaved]], sobald der Bericht geschrieben ist — in einem Browser ein
  Link, der ihn herunterlädt, in der Desktop-App sein Pfad im Tooltip;
- [[ui:exp.compare]], um diesen Durchlauf neben einen früheren zu stellen
  ([Durchläufe vergleichen](load.md#compare));
- das Profil und geänderte Werte, wie oben;
- den Startwert des Durchlaufs mit [[ui:exp.pinSeed]] oder, wenn das Experiment einen
  fixiert hat, jenen Startwert mit [[ui:exp.unpinSeed]] ([Startwerte](#seeds)).

**Frames.** Während der [[ui:dock.inspector]] aufzeichnet, behält ein Warteknoten —
oder ein Senden, das auf seine Antwort wartet —, der eine Nachricht abgeglichen hat,
die Nummer des Frames jener Nachricht. Eine Schaltfläche unter den Zeilen nennt den
Knoten und den Frame; sie öffnet den Inspektor im unteren Bereich mit ausgewähltem
Frame. Siehe den [Inspektor](../tools/inspector.md).

Die Zeitleiste zeigt den letzten Durchlauf des Experiments in dieser Sitzung; sie wird
geleert, wenn ein anderes Experiment geöffnet wird.

## Stopp {#stop}

Drücken Sie [[ui:common.stop]] oder [[ui:app.stopAll]] in der Kopfzeile, um alle Jobs
auf einmal zu stoppen. Der Durchlauf endet sofort ([was stoppt](flow.md#stop)), die
Zeitleiste zeigt [[ui:exp.stopped]], und **kein Bericht wird gespeichert**. Ein
Durchlauf, der von der Kommandozeile oder der API auf einem Server gestartet wurde,
ist ein Job wie jeder andere: [[ui:app.stopAll]] auf jenem Server stoppt ihn
ebenfalls, und sein Aufrufer erfährt, dass er gestoppt wurde.

## Das Ergebnis {#result}

| Ergebnis | Bedeutet | Bericht |
| --- | --- | --- |
| [[ui:exp.passed]] | jeder Zweig endete, kein Schritt schlug fehl und das Ende wurde erreicht | gespeichert |
| [[ui:exp.failed]] | ein Schritt schlug fehl — eine Prüfung, ein Warteknoten ohne [[ui:exp.portTimeout]]-Verbindung, ein Netzwerkfehler, ein Schwellenwert —, oder dem Durchlauf ging die Zeit aus (`run.timeout`), ein Join wartete vergeblich (`run.join_waiting`), oder kein Zweig erreichte das Ende (`run.no_end`) | gespeichert, mit dem ersten Fehler |
| [[ui:exp.stopped]] | jemand stoppte ihn | keiner |
| startete nicht | das Experiment ist ungültig, ein Geheimnis fehlt oder ein Port ließ sich nicht öffnen | keiner |

Ein fehlgeschlagener Durchlauf nennt den Knoten und das Feld seines ersten Fehlers;
die [Fehlerreferenz](../reference/errors.md) listet jeden Code auf. Die Kommandozeile
sagt dasselbe mit ihrem Exit-Code: `0` bestanden, `1` fehlgeschlagen, `2` das
Experiment oder der Aufruf war ungültig (ein fehlendes Geheimnis zählt), `3` etwas
außerhalb des Experiments hielt es vom Laufen ab, etwa ein Port, der sich nicht
öffnen ließ. Siehe [`signallab run`](../automation/cli.md#cli-run).

## Der Bericht des Durchlaufs {#report}

Jeder Durchlauf, der von selbst endet — bestanden oder fehlgeschlagen —, schreibt
einen JSON-Bericht in den Ordner `runs` des Datenordners:
`Documents/SignalLab/runs` auf einem Desktop, der eigene Datenordner des Servers auf
einem Server ([Dateien](../reference/files.md)). Die Datei ist
`run-<start time in ms>-<job number>.json`; ein Bericht wird nie über einen anderen
geschrieben. Lässt sie sich nicht schreiben, sagt der Editor, warum.

| Schlüssel | Was |
| --- | --- |
| `version` | das Format des Berichts, jetzt 5 |
| `experiment` | der Name des Experiments |
| `document_version` | die Version des Experiments, jetzt 9 |
| `seed` | der Startwert, den der Durchlauf verwendete |
| `profile` | das Profil, mit dem er lief, oder `null` für die Standardwerte |
| `overrides` | die in [[ui:exp.runWith]] getippten Werte |
| `params` | jeder Parameterwert, den der Durchlauf verwendete |
| `started_ms`, `ended_ms` | Unix-Millisekunden |
| `outcome` | `passed` oder `failed` |
| `error` | der erste Fehler oder `null` |
| `steps` | jedes Schrittereignis, in Reihenfolge (unten) |
| `emulators` | die Zahlen jedes Emulator-Knotens — vorhanden, wenn es einen gibt ([Emulatoren](faults.md#emulator)) |
| `impairments` | die Zahlen und Phasen jedes Störungs-Knotens — vorhanden, wenn es einen gibt ([Phasen](faults.md#change-impairment)) |

Jedes Schrittereignis hat:

| Schlüssel | Was |
| --- | --- |
| `job_id`, `node_id` | der Durchlauf und der Knoten |
| `ts` | Unix-Millisekunden |
| `state` | `running`, `passed`, `failed`, `retry`, `repeating`, `load` |
| `detail` | was geschah, auf Englisch |
| `message_key`, `message_params` | dasselbe wie der Text der Oberfläche und seine Werte, sodass der Schritt in jeder Sprache gezeigt werden kann |
| `vars` | die Variablen, die der Schritt schrieb, falls welche |
| `error` | warum er fehlschlug: `code`, `params`, `node`, `field`, `detail` |
| `frame` | der Inspector-Frame, den ein Warteknoten abglich, wenn der Mitschnitt an war |
| `load` | was eine Last maß ([Messwerte](load.md#metrics)), bei ihrem letzten Ereignis |

Geheimniswerte erscheinen nie in einem Bericht: Sie werden als `••••` maskiert
([Maskierung](data.md#masking)).

Das Format des Berichts wuchs mit den Funktionen: Version 3 fügte die Zahlen der
Emulatoren hinzu, Version 4 die Phasen der Störungen, Version 5 die Messungen einer
Last.

Die Berichte sind die Durchlauf-Historie: [[ui:exp.compare]] liest sie, und ebenso
[`experiment_runs`](../api/commands.md#experiment_runs). Das `--report` der
Kommandozeile kopiert den Bericht eines Durchlaufs dorthin, wo Sie ihn wollen.

## Startwerte {#seeds}

Jeder Durchlauf hat einen Startwert, eine ganze Zahl von 0 bis 9 007 199 254 740 991.
Er ist, in dieser Reihenfolge:

1. der Startwert, der diesem Durchlauf in [[ui:exp.runWith]], auf der Kommandozeile
   (`--seed`) oder an die API gegeben wurde;
2. der im Experiment fixierte Startwert;
3. ein neuer zufälliger Startwert.

Die erste Zeile des Durchlaufs in der Zeitleiste gibt ihn, und der Bericht behält ihn.

Der Startwert entscheidet alles Zufällige, was ein Durchlauf tut: die
[Generatoren](data.md#generators) in Vorlagen, den Jitter der
[Wiederholung](flow.md#repeat), die Ankünfte einer [zufälligen
Last](load.md#schedule), das Schicksal jedes Pakets in einem
[Störungs-Relais](faults.md#seed) und die Zufallsentscheidungen eines Emulators.
Jeder zieht aus einem eigenen Strom, sodass parallele Zweige nie die Werte des
jeweils anderen verschieben.

Um einen Durchlauf zu wiederholen:

1. Drücken Sie [[ui:exp.pinSeed]] neben seinem Startwert in der Zeitleiste. Der
   Startwert wird im Experiment gespeichert, und jeder Durchlauf verwendet ihn, bis
   Sie [[ui:exp.unpinSeed]] drücken. Unter [[ui:exp.params]] zeigt und bearbeitet
   [[ui:exp.seed]] den fixierten Startwert; leer ist er
   [[ui:exp.seedRandom]].
2. Führen Sie mit demselben Profil und denselben Werten aus; der Bericht listet sie
   auf.

Was ein Startwert nicht wiederholen kann: die Zeit (`{{now}}`), `{{run.id}}` und wann
Geräte und das Netzwerk antworten.

## Einen einzelnen Knoten ausprobieren {#send-now}

Um einen Knoten auszuprobieren, ohne das Experiment auszuführen, wählen Sie ihn aus
und drücken Sie [[ui:exp.sendNow]] — oder <kbd>Ctrl</kbd>+<kbd>Enter</kbd> in seinen
Eigenschaften — an einem Knoten [[ui:exp.node.http]], [[ui:exp.node.tcp]],
[[ui:exp.node.osc]], [[ui:exp.node.udp]], [[ui:exp.node.mqtt]],
[[ui:exp.node.ws_connect]] oder [[ui:exp.node.ws_send]]. Bei einem Warteknoten ist es
[[ui:exp.listenNow]]: Er lauscht von jetzt an, bis eine Nachricht passt oder seine
Zeitüberschreitung endet.

Die Engine führt den Knoten mit dem Code aus, den ein Durchlauf verwendet, einmal:

- mit den Werten des aktiven Profils, den in dieser Sitzung bekannten Variablenwerten
  (aus dem letzten Durchlauf und früheren Versuchen) und den gespeicherten
  Geheimnissen;
- mit dem fixierten Startwert oder einem neuen; `{{run.id}}` ist `0` und `{{counter}}`
  ist `1`;
- ohne Neuversuch, Wiederholung oder Last — eine Sendung;
- **ohne Cookies**: eine Anfrage, nichts davor gesetzt, um zurückgesendet zu werden;
- ohne die Relais und Emulatoren des Durchlaufs. Ein [[ui:exp.node.wait_http]]
  lauscht auf einem eigenen Empfänger, und ein WebSocket-Senden oder -Warten öffnet
  die Verbindung, die sein [[ui:exp.node.ws_connect]] beschreibt, für diesen einen
  Versuch.

Hat ein Name, den der Knoten verwendet, noch keinen Wert, wird nichts gesendet, und
das Ergebnis sagt, welche Namen fehlen — führen Sie das Experiment aus oder verwenden
Sie zuerst [[ui:exp.sendNow]] an dem Knoten, der sie setzt.

Das Ergebnis zeigt ✓ oder ✕ und was geschah. Bei einer HTTP-Anfrage zeigt es außerdem
den Status, die Zeit, die Größe und die [[ui:http.response]]; in einer JSON-Antwort
lässt sich jeder Wert anklicken, um ihn zu [extrahieren](data.md#extract), und
[[ui:http.mockThis]] macht die Antwort zu einer Route eines Emulators. Die Werte, die
ein Warteknoten empfangen hat oder die die [[ui:exp.node.extract]]-Knoten direkt nach
einer Anfrage aus ihrer Antwort nehmen würden, werden der Vorschau und dem nächsten
[[ui:exp.sendNow]] bekannt. Während ein Durchlauf läuft, ist [[ui:exp.sendNow]] nicht
verfügbar.

Die Vorschau eines Knotens mit Vorlagen — was er [senden wird](data.md#preview) —
wird ebenfalls von der Engine aufgelöst, ohne etwas zu senden.

## Experimentdateien {#files}

### Das arbeitende Experiment {#working-file}

Der Editor hält ein Experiment, 0,7 s nach jeder Änderung von selbst in
`experiment.json` im Datenordner gespeichert; die Werkzeugleiste zeigt
[[ui:exp.saving]], [[ui:exp.saved]] oder [[ui:exp.saveError]]. Ein unfertiger Graph
speichert ebenfalls. Eine Datei, die sich nicht lesen lässt, wird mit ihrem Pfad
gemeldet, nie ersetzt. Eine Experimentdatei ist höchstens 4 MiB groß.

Auf einem Server liegt die Datei im Datenordner des Servers, sodass jeder Browser, der
den Editor dort öffnet, am selben Experiment arbeitet.

### Öffnen, Vorlagen und Export {#open-export}

Die Schaltfläche ☰ in der Werkzeugleiste öffnet [[ui:exp.documents]]:

- [[ui:exp.templates]]: [[ui:exp.templateEmpty]], [[ui:exp.templateHttp]],
  [[ui:exp.templateBranch]], [[ui:exp.templateParallel]],
  [[ui:exp.templatePingReply]], [[ui:exp.templatePoll]],
  [[ui:exp.templateFlaky]], [[ui:exp.templateFaults]],
  [[ui:exp.templateOutage]], [[ui:exp.templateWsEcho]]. Ihre Ziele liegen auf
  `127.0.0.1`.
- [[ui:exp.importJson]] liest eine Datei von bis zu 4 MiB — dieses Formats des
  Experiments oder eines älteren, das beim Öffnen auf den neuesten Stand gebracht wird
  — und prüft sie, bevor sie ihren Namen und die Zahl ihrer Knoten und Verbindungen
  zeigt. Die Datei muss auch in der Form, in der der Editor sie schreibt, eingerückt,
  in 4 MiB passen, sodass eine kompakte Datei nahe der Grenze abgelehnt werden kann.
  Eine beschädigte Datei wird mit Zeile und Spalte des Problems abgelehnt, eine Datei
  von einem neueren Signal Lab mit `doc.version_unsupported`, und das aktuelle
  Experiment bleibt.
- [[ui:exp.openDocument]] ersetzt das aktuelle Experiment durch das gewählte.
  <kbd>Ctrl</kbd>+<kbd>Z</kbd> holt das vorherige während dieser Sitzung zurück. Das
  Öffnen eines Experiments führt es nicht aus.
- [[ui:exp.exportJson]] schreibt eine Kopie in den Ordner `exports` des Datenordners,
  als `experiment-<time in ms>-<random>.json`, nie über eine andere Kopie; in einem
  Browser holt [[ui:common.download]] sie.

Die Kommandozeile und die API nehmen dieselben Dateien und die Vorlagen nach Namen:
`empty`, `http-check`, `status-branch`, `parallel-flows`, `osc-ping-reply`,
`poll-until-ready`, `flaky-api`, `fault-phases`, `dependency-outage`,
`websocket-echo`.

### Dokumentversionen {#versions}

Eine Experimentdatei hat eine `version`; dieses Signal Lab schreibt Version 9 und
öffnet jede frühere, wobei es ergänzt, was die ältere Datei nicht halten konnte. Eine
Datei einer neueren Version als 9 wird abgelehnt (`doc.version_unsupported`), statt
ohne das, was sie hält, geöffnet zu werden.

| Version | Hinzugefügt |
| --- | --- |
| 2 | Parameter und der Startwert |
| 3 | Profile |
| 4 | Neuversuch und eine von einem OSC- oder UDP-Senden erwartete Antwort |
| 5 | Wiederholung und [[ui:exp.node.loop]] |
| 6 | [[ui:exp.node.emulator]] und [[ui:exp.node.wait_http]] |
| 7 | [[ui:exp.node.impairment]], [[ui:exp.node.impairment_change]] und [[ui:exp.node.emulator_state]] |
| 8 | die WebSocket-Knoten, HTTP-Authentifizierung und der Cookie-Speicher |
| 9 | Last auf einer HTTP-Anfrage und Störung über TCP |

Eine Datei von vor Version 8 öffnet mit ausgeschaltetem [[ui:exp.cookies]], sodass sie
läuft wie zuvor; eine neuere Datei behält ihre eigene Einstellung. Erneut gespeichert,
wird jede Datei zu Version 9.

## Von der Kommandozeile oder einem Server {#automation}

Ein Durchlauf ist überall gleich: Die Kommandozeile und die API des Servers starten
denselben Durchlauf wie der Editor, mit denselben Schritten, demselben Ergebnis und
Bericht.

```bash
signallab run checkout.json --profile Stage -p api=http://192.0.2.10:8080 --seed 42 --report report.json
```

- [`signallab run`](../automation/cli.md#cli-run) führt Experimentdateien oder
  Vorlagen in diesem Prozess oder auf einem Server aus, gibt die Schritte wie die
  Zeitleiste aus und beendet sich mit dem Code des Ergebnisses.
- [`POST /api/run`](../api/run.md) führt einen auf einem Server aus und antwortet mit
  dem Ergebnis oder streamt seine Schritte, wie sie geschehen. Ein Client, der
  weggeht, stoppt den Durchlauf nicht; er läuft bis zu seinem Ende und behält seinen
  Bericht.
- In CI: [GitHub Actions und andere](../automation/ci.md).

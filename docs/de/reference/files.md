---
title: Dateien und Ordner
description: Wo Signal Lab seine Dateien aufbewahrt — den Datenordner, das Experiment, die Signal- und die Emulatorbibliothek, Berichte von Durchläufen und Exporte —, ihre Formate und was sicher zu bearbeiten, sichern und verschieben ist.
---

# Dateien und Ordner

Alles, was Signal Lab aufbewahrt, ist einfaches JSON (oder Text) in einem Ordner,
dem Datenordner. Geheimniswerte stehen nie darin.

## Der Datenordner {#data-folder}

| Wo Signal Lab läuft | Der Datenordner |
| --- | --- |
| Desktop-App, Windows | `Documents\SignalLab` in Ihrem Benutzerordner: `C:\Users\<you>\Documents\SignalLab` |
| Desktop-App, Linux | `~/Documents/SignalLab` |
| Server | `--data-dir` oder `SIGNALLAB_DATA_DIR`; ohne beides `Documents/SignalLab` im Benutzerordner des Benutzers, als der er läuft |
| Server, Docker-Image | `/data`, ein Volume (`signallab-data` in der Compose-Datei) |
| `signallab run` | Ein temporärer Ordner, beim Beenden entfernt — außer `--data-dir` nennt einen |

Die Desktop-App übernimmt außerdem `SIGNALLAB_DATA_DIR` aus ihrer Umgebung, wenn
es gesetzt ist. Der Ordner wird angelegt, sobald etwas hineingeschrieben wird.

::: tip
Unter Windows verwendet die App den Ordner `Documents` direkt in Ihrem
Benutzerordner, auch wenn Windows Ihre Dokumente woanders aufbewahrt (OneDrive).
:::

Auf einem Server werden Dateien auf dem Rechner des Servers geschrieben, nicht
auf Ihrem. Das [[ui:app.server]]-Abzeichen in der Kopfzeile sagt in seinem
Hinweis, wo; die API gibt es als `data_dir` von
[`app_info`](../api/commands.md#app_info) aus, und
[`/api/files`](../api/index.md#files) lädt herunter, was darin liegt.

Das `signallab emulate`, `signallab send` und `signallab mcp` der Kommandozeile
lesen die Bibliotheken der App aus demselben Ordner wie die Desktop-App.

## Was darin liegt {#contents}

| Datei | Was sie ist | Geschrieben |
| --- | --- | --- |
| `experiment.json` | Das im Editor geöffnete Experiment | Kurz nach jeder Änderung |
| `signals.json` | Die Signalbibliothek ([[ui:nav.signals]]) | Kurz nach jeder Änderung |
| `emulators.json` | Die Emulatorbibliothek ([[ui:nav.emulators]]) | Kurz nach jeder Änderung |
| `runs/run-<ms>-<job>.json` | Ein Bericht pro Durchlauf, der von selbst endete | Wenn der Durchlauf endet |
| `exports/experiment-<ms>-<16 hex digits>.json` | Ein Schnappschuss des Experiments | [[ui:exp.exportJson]] |
| `capture-<ms>.jsonl`, `capture-<ms>.txt` | Die Frames des Inspektors | [[ui:ins.exportJsonl]], [[ui:ins.exportTxt]] |
| `token` | Das Zugriffstoken eines Servers, nur für seinen Benutzer lesbar | `--generate-token`, beim ersten Start |
| `.experiment-<hex>.tmp`, `.signals-<hex>.tmp`, `.emulators-<hex>.tmp` | Ein Speichern auf dem Weg | Einen Moment, dann umbenannt |

`<ms>` ist eine Zeit in Millisekunden seit 1970; `<job>` ist die Job-Nummer des
Durchlaufs. Auf einem Server arbeiten alle Browser am selben `experiment.json`,
an denselben Bibliotheken und denselben Berichten.

## Formate {#formats}

Alle sind JSON in UTF-8, mit Einrückung geschrieben, damit sie sich gut lesen
und vergleichen lassen. Jede trägt eine `version`; eine Datei einer älteren
Version wird beim Öffnen gelesen und migriert und beim nächsten Speichern in der
aktuellen Version zurückgeschrieben — danach kann ein älteres Signal Lab sie
nicht mehr öffnen.

### experiment.json {#experiment-json}

Das Experimentdokument, Version 9 — dasselbe JSON, das [[ui:exp.exportJson]]
schreibt und [[ui:exp.importJson]] liest:

```json
{
  "version": 9,
  "name": "HTTP check",
  "params": [],
  "profiles": [],
  "profile": null,
  "seed": null,
  "cookies": true,
  "nodes": [ { "id": "start", "type": "start", "x": 40, "y": 80 }, … ],
  "edges": [ { "from": "start", "to": "request", "port": "next" }, … ]
}
```

- Höchstens 4 MiB und 1 bis 64 Knoten (`doc.node_count`).
- Versionen 1 bis 8 werden beim Öffnen migriert. Eine Datei von vor Version 8
  öffnet mit ausgeschaltetem `cookies`, damit sie läuft wie zuvor; die anderen
  Einstellungen, die jede Version hinzufügte (Parameter in 2, Profile in 3,
  Wiederholungen in 4, Repeat und Loops in 5, Emulatoren in 6, Impairments in 7,
  WebSocket- und HTTP-Authentifizierung in 8, Load in 9), starten leer.
- Eine neuere Version, als dieses Signal Lab kennt, wird abgelehnt
  (`doc.version_unsupported`), statt ohne das geöffnet zu werden, was es nicht
  lesen kann.
- Eine Datei, die nicht parst, wird mit ihrem Pfad, ihrer Zeile und Spalte
  gemeldet und nie ersetzt.
- Sie wird in eine temporäre Datei geschrieben und umbenannt, sodass ein
  fehlgeschlagenes Schreiben die vorherige hinterlässt.

Was die Knoten, Parameter und Profile sind:
[Experimente](../experiments/index.md), [Knoten](../experiments/nodes.md),
[Daten](../experiments/data.md).

### signals.json {#signals-json}

Die Signalbibliothek, Version 2:

```json
{
  "version": 2,
  "signals": [
    {
      "id": "…",
      "name": "Go cue",
      "group": "Stage/Cues",
      "note": "",
      "body": { "transport": "osc", "target": "127.0.0.1:9000", "address": "/cue/go", "args": [ { "type": "int", "value": 1 } ] }
    }
  ],
  "folders": [ "Stage", "Stage/Cues" ]
}
```

- `group` ist der Ordner des Signals als `/`-Pfad; leer ist die oberste Ebene.
  `folders` (in Version 2 hinzugefügt) listet jeden Ordner auf, leere
  eingeschlossen, und wird weggelassen, wenn es keine gibt. Eine Datei der
  Version 1 liest sich gleich, ohne leere Ordner.
- `body` ist eines von `osc`, `udp`, `http` oder `mqtt`; seine Felder stehen in
  [`signals_save`](../api/commands.md#signals_save).
- Wenn die Datei nicht existiert, wird der Startersatz geschrieben — jedes Ziel
  auf `127.0.0.1` — und in die Sprache der Oberfläche umbenannt.
- Eine Datei, die nicht parst, wird mit ihrem Pfad, ihrer Zeile und Spalte
  gemeldet (`signals.json_invalid`) und nie durch den Startersatz ersetzt:
  Reparieren oder löschen Sie sie. Nichts schreibt die Bibliothek, solange sie
  nicht liest — ein Speichern wird mit demselben Fehler abgelehnt und die Datei
  bleibt, wie sie ist —, bis [[ui:sig.reload]] sie erneut liest. Ein Speichern
  läuft über eine temporäre Datei im selben Ordner, sodass ein abgebrochenes
  Schreiben die vorherige Datei hinterlässt.

### emulators.json {#emulators-json}

Die Emulatorbibliothek, Version 1:

```json
{
  "version": 1,
  "emulators": [
    { "id": "demo-api", "note": "…", "emulator": { "name": "Demo API", "bind": "127.0.0.1:8080", "protocol": "http", "routes": [ … ] } }
  ]
}
```

Jeder Eintrag ist ein Emulatordokument mit einer `id` und einer `note`; das
Dokument ist unter [Emulatoren](../tools/emulators.md) beschrieben. Wie bei den
Signalen erhält eine fehlende Datei den Startersatz (jeder davon gebunden an
`127.0.0.1`), und eine kaputte wird gemeldet (`emulators.json_invalid`), nie
ersetzt. Sie wird über eine temporäre Datei geschrieben. `signallab emulate`
liest außerdem eine eigene Datei, die einen Emulator, eine Liste von ihnen oder
eine Bibliothek wie diese enthält.

### Berichte von Durchläufen {#run-reports}

`runs/run-<started ms>-<job>.json`, Berichtversion 5: eine Datei pro Durchlauf,
der bestanden oder fehlgeschlagen ist, nie überschrieben (ein zweiter Durchlauf
desselben Namens erhält `-2`, `-3`… angehängt). Ein gestoppter Durchlauf
speichert keinen.

| Feld | Was es ist |
| --- | --- |
| `version` | 5 |
| `experiment` | Der Name des Experiments |
| `document_version` | Die Version des Dokuments, das lief |
| `seed`, `profile` | Womit er lief |
| `overrides` | Werte, die nur für diesen Durchlauf angegeben wurden |
| `params` | Jeder verwendete Parameterwert |
| `started_ms`, `ended_ms` | Millisekunden seit 1970 |
| `outcome` | `passed` oder `failed` |
| `error` | Sein erster Fehlschlag oder null |
| `steps` | Jeder Schritt, wie [`experiment://step`](../api/events.md#event-experiment-step) |
| `emulators` | Was jeder [[ui:exp.node.emulator]]-Knoten empfangen und geantwortet hat (seit Version 3); weggelassen, wenn keiner |
| `impairments` | Was das Relais jedes [[ui:exp.node.impairment]]-Knotens tat, Phase für Phase (seit Version 4); weggelassen, wenn keiner |

Die Messungen eines Load-Schritts stehen auf seinem letzten Schritt (seit
Version 5). Der Durchlaufverlauf der Zeitleiste und [[ui:exp.compare]] lesen
diese Dateien; ein Bericht, der nicht gelesen werden kann, fällt aus der Liste.
Siehe [Durchläufe und Berichte](../experiments/runs.md).

### Exporte {#exports}

- `exports/experiment-…json`: das Experimentdokument, wie oben. Jeder Export ist
  eine neue Datei.
- `capture-….jsonl`: ein Inspektor-Frame pro Zeile, mit den Bytes, die er in
  `data` behält, base64.
- `capture-….txt`: die Frames zum Lesen, jeder mit einem Hex-Dump.

Auf einem Server lädt der Export des Inspektors auf Ihren Computer, sobald er
erstellt wird; der Export eines Experiments bietet [[ui:common.download]] an,
und [[ui:exp.reportSaved]] eines Durchlaufs in der Zeitleiste ist ein Link, der
seinen Bericht herunterlädt.

## Geheimnisse stehen nicht in diesen Dateien {#secrets}

Ein Experiment nennt ein Geheimnis — `{{secret.API_TOKEN}}` —, und es wird nur
der Name geschrieben. Der Wert wird aufbewahrt:

| Wo Signal Lab läuft | Wo Geheimniswerte liegen |
| --- | --- |
| Desktop-App, Windows | Windows-Anmeldeinformationsverwaltung, unter `SignalLab` ([[ui:exp.secrets]] im Editor) |
| Desktop-App, Linux | Nirgends: Geheimnisse können nicht gespeichert werden (`secret.unsupported`) |
| Server | Schreibgeschützt: die Umgebungsvariable `SIGNALLAB_SECRET_<NAME>` oder die Datei `<NAME>` in `--secrets-dir` (standardmäßig `/run/secrets/signallab`) |
| `signallab` | Dieselben Dateien und Variablen oder der Systemspeicher mit `--secrets system` |

::: warning
Was Sie direkt in ein Feld tippen, wird so aufbewahrt, wie Sie es getippt haben.
Ein Passwort in den Zugangsdaten eines HTTP-Signals, das Passwort eines
MQTT-Broker-Emulators, ein in einen Header eingefügtes Token — alles ist
Klartext in `signals.json`, `emulators.json` oder `experiment.json` und in ihren
Exporten. Verwenden Sie `{{secret.NAME}}` in einem Experiment für alles, was Sie
nicht in einen geteilten Ordner legen würden.
:::

## Einstellungen der Oberfläche {#settings}

Was die Oberfläche sich merkt — ihre Sprache, die zuletzt in jeder Ansicht
eingegebenen Werte, welcher Bereich offen und wie groß, [[ui:http.keepCookies]],
wann zuletzt nach Updates gesucht wurde und die Zufallszahl der Installation für
Updates —, wird von der Oberfläche selbst aufbewahrt, nicht im Datenordner: im
Desktop im eigenen Speicher der App, für die Seite eines Servers im Site-Speicher
des Browsers (pro Browser). Die Zugangsdaten der Ansicht [[ui:nav.http]] werden
dort nicht aufbewahrt.

Signal Lab schreibt keine Log-Dateien; siehe
[Fehlerbehebung](troubleshooting.md#logs).

## Sichern, Bearbeiten, Verschieben {#backup}

- **Sichern** Sie, indem Sie den ganzen Ordner kopieren. Alles darin ist in sich
  geschlossenes JSON; Geheimniswerte stehen nicht darin, setzen Sie sie auf einem
  neuen Rechner also erneut.
- **Bearbeiten** Sie `signals.json`, `emulators.json` und `experiment.json` von
  Hand, während Signal Lab geschlossen ist (oder, auf einem Server, während keine
  Seite offen ist): Die App schreibt die ganze Datei aus dem, was sie hält, daher
  wird eine Änderung während des Betriebs durch ihr nächstes Speichern
  überschrieben. Ein Fehler wird mit Zeile und Spalte gemeldet, wenn die Datei
  das nächste Mal gelesen wird, nie stillschweigend ersetzt — bei
  `signals.json` auch, wenn die App hineinspeichert: Dieses Speichern wird
  abgelehnt und die Datei bleibt, wie Sie sie hinterlassen haben.
- **Löschen** Sie `runs/`, `exports/` und `capture-*`-Dateien jederzeit. Das
  Löschen von `signals.json` oder `emulators.json` bringt den Startersatz zurück;
  das Löschen von `experiment.json` bringt das Starter-Experiment zurück.
- **Verschieben** Sie den Ordner, indem Sie ihn kopieren und Signal Lab auf die
  neue Stelle richten: `--data-dir` für einen Server, `SIGNALLAB_DATA_DIR` für
  die Desktop-App.
- **Teilen** Sie ein Experiment, indem Sie es exportieren oder sein JSON neben
  das Projekt committen, das es testet;
  [`signallab run`](../automation/cli.md#cli-run) führt es von dort aus.

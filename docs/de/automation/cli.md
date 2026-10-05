---
title: Kommandozeile
description: signallab führt Experimente ohne Fenster aus, sendet einzelne Nachrichten, spielt Emulatoren und prüft das Netzwerk — aus einem Terminal, einem Skript oder einem CI-Job.
---

# Die Kommandozeile: `signallab`

`signallab` ist Signal Lab ohne Fenster. Es führt Experimente bis zum Ende aus und
endet mit einem Code, den ein Skript versteht, sendet eine OSC-Nachricht, ein
Datagramm, eine HTTP-Anfrage, eine WebSocket-Nachricht oder eine MQTT-Veröffentlichung,
sendet ein Signal aus Ihrer Bibliothek, spielt einen Emulator, bis Sie ihn stoppen,
und sagt, was zwischen diesem Rechner und der Technik steht.

Es ist dieselbe Engine wie die App: Ein Durchlauf von der Kommandozeile macht
dieselben Schritte, schreibt denselben Bericht und sagt dieselben Dinge, in den
Sprachen der Oberfläche. Mit `--server` finden die Durchläufe stattdessen auf
einem [Signal-Lab-Server](../server/index.md) statt, mit dessen Netzwerk und
dessen Geheimnissen.

```bash
signallab run tests/smoke.json --param api=http://127.0.0.1:8080 --junit junit.xml
signallab run flaky-api                                # a bundled template, by name
signallab validate tests/*.json                        # the editor's check, nothing sent
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab fire "Fader value"
signallab emulate tests/orders-api.json --for 120      # play a dependency for two minutes
signallab doctor                                       # firewall, network, data folder
```

Für Pipelines siehe [Signal Lab in der CI](ci.md); für einen KI-Assistenten
[`signallab mcp`](mcp.md).

## Installieren {#install}

| Wo | Wie Sie `signallab` bekommen |
| --- | --- |
| Windows, der Installer (`.exe`) | Neben der App installiert, und dieser Ordner wird zu `PATH` hinzugefügt — Ihrem bei einer Installation „für mich“, dem der Maschine bei „für alle“. Öffnen Sie nach der Installation ein neues Terminal. Der Schalter `/NOPATH` des Installers lässt `PATH` unberührt. |
| Windows, die `.msi` | Neben der App installiert; der Installationsordner liegt im `PATH` der Maschine, solange die App installiert ist. |
| Linux, `.deb` und `.rpm` | `/usr/bin/signallab`. |
| Linux, AppImage | Nicht enthalten: Verwenden Sie das Archiv unten. |
| Jeder Rechner, ohne die App | Jede Veröffentlichung enthält `signallab-<version>-windows-x64.zip` und `signallab-<version>-linux-x64.tar.gz`, jeweils mit dem Programm und seiner Lizenz; `SHA256SUMS.txt` auf derselben Seite listet ihre Prüfsummen. |
| Das Server-Image | `/usr/local/bin/signallab` in `ghcr.io/proanima/signallab` (siehe [CI](ci.md#docker)). |

Prüfen Sie es mit:

```bash
signallab version
```

## Befehle {#commands}

| Befehl | Was er tut |
| --- | --- |
| [`run`](#cli-run) | Führt Experimente nacheinander aus; endet nur mit 0, wenn jedes bestanden wurde. |
| [`validate`](#cli-validate) | Prüft Experimente so, wie es der Editor vor einem Durchlauf tut; sendet nichts. |
| [`send`](#cli-send) | Sendet eine Nachricht: `osc`, `udp`, `http`, `ws` oder `mqtt`. |
| [`fire`](#cli-fire) | Sendet ein Signal einer Signalbibliothek, über seine ID oder seinen Namen. |
| [`emulate`](#cli-emulate) | Spielt eine HTTP-API, ein OSC-, UDP- oder TCP-Gerät oder einen MQTT-Broker, bis <kbd>Ctrl</kbd>+<kbd>C</kbd> oder `--for`. |
| [`emulators`](#cli-emulators) | Listet die Emulatoren der Bibliothek der App auf. |
| [`templates`](#cli-templates) | Listet die mitgelieferten Experimentvorlagen auf. |
| [`nodes`](#cli-nodes) | Beschreibt jede Art von Knoten, als JSON. |
| [`mcp`](#cli-mcp) | Stellt Signal Lab einem KI-Assistenten über das Model Context Protocol bereit. |
| [`doctor`](#cli-doctor) | Prüft die Firewall, das Netzwerk, den Datenordner und einen Server. |
| [`firewall`](#cli-firewall) | `firewall allow`: Lässt andere Rechner Signal Lab durch die Windows-Firewall erreichen. |
| [`version`](#cli-version) | Gibt die Version aus. |

`signallab help <command>` oder `signallab <command> --help` gibt die Optionen
eines Befehls aus.

## Optionen jedes Befehls {#global-options}

| Option | Was sie tut | Standard |
| --- | --- | --- |
| `--lang <code>` | Die Sprache der Nachrichten: `en`, `ru`, `es`, `fr`, `de`, `pt`, `zh`, `ja`, `ko`, `hi` oder `ar`. | `SIGNALLAB_LANG`, sonst die Locale, sonst `en` |
| `--json` | Ausgabe für Maschinen auf der Standardausgabe statt Text (siehe [Ausgabe](#output)). | aus |
| `-h`, `--help` | Die Hilfe des Befehls. | |
| `-V`, `--version` | Die Version; vor jedem Befehl, als `signallab --version`. | |

Sowohl `--lang` als auch `--json` dürfen überall in der Zeile stehen:
`signallab --json run smoke.json` und `signallab run smoke.json --json` sind dasselbe.

### Sprache {#language}

Nachrichten, Schrittexte, Fehler und der JUnit-Bericht verwenden die eigenen
Texte, Pluralformen und die Zahlenweise der Oberfläche. Die Sprache ist die erste von:

1. `--lang`;
2. `SIGNALLAB_LANG` (`ru`, `ru-RU` und `ru_RU.UTF-8` bedeuten alle Russisch);
3. die Locale: die erste von `LC_ALL`, `LC_MESSAGES` und `LANG`, die gesetzt ist;
4. Englisch.

::: tip
Windows-Terminals setzen meist keine der Locale-Variablen, `signallab`
spricht dort also Englisch, es sei denn, Sie setzen `SIGNALLAB_LANG` oder geben
`--lang` an.
:::

### Ausgabe für Menschen und für Maschinen {#output}

Ohne `--json` gehen Ergebnisse an die **Standardausgabe** und alles, was ein
Mensch unterwegs liest, an die **Standardfehlerausgabe**: die Schritte eines
Durchlaufs, während sie geschehen, warum etwas fehlschlug, wo der Bericht liegt.
`signallab run … 2>/dev/null` lässt eine Ergebniszeile pro Durchlauf übrig.

Mit `--json` trägt die Standardausgabe JSON und nichts anderes, und die
Standardfehlerausgabe bleibt ruhig:

| Befehl | Was `--json` auf der Standardausgabe ausgibt |
| --- | --- |
| `run` | Ein Objekt pro Zeile: `started`, ein `step` pro Schritt, `ended` (das Ergebnis des Durchlaufs), dann `summary`; `error` für einen Durchlauf, der nicht starten konnte. Siehe [Was `run` ausgibt](#run-output). |
| `validate` | Ein Objekt pro Zeile, pro Experiment: `valid` und `profile_issues` oder `error`. |
| `send`, `fire` | `{"type": "sent", "result": …}`; `send http` gibt `{"type": "response", "response": …}` aus, `send ws` `{"type": "exchange", "result": …}`. |
| `emulate` | Ein Objekt pro Zeile: `started`, ein `exchange` pro Anfrage, eine `summary` pro Emulator; `valid` mit `--check`. |
| `emulators` | `{"path": …, "emulators": [{id, name, protocol, bind, rules, note}, …]}`. |
| `templates` | `[{"name": …, "experiment": …}, …]`. |
| `doctor` | Ein Objekt: `version`, `network`, `data_dir`, `firewall`, `server`, `problems`. |
| `version` | `{"version": "…"}`. |
| `nodes` | Immer JSON, mit oder ohne `--json`. |

Ein Fehler ist `{"type": "error", "error": {…}, "exit_code": N}`. `error` ist
der Fehler der Engine: ein stabiler `code` (etwa `transport.refused` oder
`secret.missing`), seine `params` und der `node` und `field`, um die es geht. Ein
Skript kann in jeder Sprache auf `error.code` verzweigen; die Texte für jeden Code
sind in [Fehlermeldungen](../reference/errors.md) aufgeführt.

### Exit-Codes {#exit-codes}

| Code | Bedeutung |
| --- | --- |
| `0` | Jedes Experiment bestand; das Senden war erfolgreich; nichts steht im Weg. |
| `1` | Ein Experiment lief und schlug fehl, lief in ein Zeitlimit oder wurde gestoppt; ein Senden schlug fehl (abgelehnt, keine Antwort, ein unerwarteter Status); `doctor` fand etwas im Weg. |
| `2` | Der Befehl oder ein Dokument ist falsch: ein Argument, eine Datei, die sich nicht lesen lässt, ein Validierungsfehler, ein unbekannter Parameter, ein fehlendes Geheimnis. |
| `3` | Nichts konnte aus einem Grund außerhalb des Experiments laufen: Der Server ist nicht erreichbar oder verweigert das Token, ein Port lässt sich nicht öffnen, der Anmeldedatenspeicher schlägt fehl. |

Bei mehreren Experimenten entscheidet das schwerwiegendste Ergebnis, in dieser
Reihenfolge: `2`, dann `3`, dann `1`, dann `0`.

### Umgebungsvariablen {#environment}

| Variable | Was sie tut |
| --- | --- |
| `SIGNALLAB_LANG` | Die Sprache, wenn `--lang` nicht angegeben ist. |
| `LC_ALL`, `LC_MESSAGES`, `LANG` | Die Sprache, wenn nichts von beidem gesetzt ist. |
| `SIGNALLAB_SERVER` | Der Server für `run`, `validate`, `emulate`, `mcp` und `doctor`, wie `--server` ihn angibt. |
| `SIGNALLAB_TOKEN` | Das Zugriffstoken des Servers, wenn keine Tokendatei angegeben ist. |
| `SIGNALLAB_TOKEN_FILE` | Eine Datei mit dem Token des Servers, wie `--token-file` sie angibt. |
| `SIGNALLAB_SECRET_<NAME>` | Der Wert des Geheimnisses `NAME` für Durchläufe in diesem Prozess (siehe [Geheimnisse](#secrets)). |
| `SIGNALLAB_DATA_DIR` | Der Datenordner der App, wo `fire`, `emulators`, `emulate`, `mcp` und `doctor` standardmäßig nachsehen; sonst `Documents/SignalLab` in Ihrem Benutzerordner. |
| `GITHUB_ACTIONS` | Wenn er `true` ist, wird ein Durchlauf, der nicht besteht, zusätzlich als `::error`-Annotation ausgegeben, die GitHub auf der Seite des Durchlaufs zeigt. |

## `run` {#cli-run}

```text
signallab run [OPTIONS] <FILE>...
```

Führt Experimente nacheinander aus und endet nur mit `0`, wenn jedes bestanden wurde.

| Option | Was sie tut | Standard |
| --- | --- | --- |
| `<FILE>...` | Experimentdateien oder Namen [mitgelieferter Vorlagen](#cli-templates). | erforderlich |
| `-p`, `--param NAME=VALUE` | Ein Parameterwert für diesen Durchlauf; für mehr wiederholen. | die Werte des Dokuments |
| `--profile NAME` | Mit diesem Profil ausführen; jedes angegebene Experiment muss es haben. `""` führt mit den Standardwerten aus. | das Profil des Dokuments |
| `-m`, `--matrix NAME=V1,V2` | Einmal pro Wert ausführen; für mehr Namen wiederholen (siehe [Eine Matrix von Durchläufen](#matrix)). | |
| `--matrix-file PATH` | Kombinationen aus einer JSON-Datei. | |
| `--seed N` | Der Startwert der Zufallswerte, 0 bis 9007199254740991 (2⁵³ − 1). | der Startwert des Dokuments, sonst ein neuer pro Durchlauf |
| `--timeout SECONDS` | Lässt einen Durchlauf, der länger dauert, fehlschlagen, 1–300. | `300` |
| `--fail-fast` | Beim ersten Durchlauf anhalten, der nicht besteht; die übrigen werden nicht gestartet. | aus |
| `--junit PATH` | Dort einen JUnit-XML-Bericht schreiben (siehe [Berichte](#reports)). | |
| `--report PATH` | Den Bericht des Durchlaufs dorthin kopieren: eine Datei für einen Durchlauf, einen Ordner für mehrere. | |
| `--data-dir PATH` | Der Datenordner für die Durchläufe dieses Prozesses; ihre Berichte bleiben dort. | ein temporärer Ordner, der beim Beenden entfernt wird |
| `--server URL` | Auf diesem Server statt in diesem Prozess ausführen (siehe [Auf einem Server](#run-on-server)). | `SIGNALLAB_SERVER` |
| `--token-file PATH` | Eine Datei mit dem Token des Servers. | `SIGNALLAB_TOKEN_FILE`, sonst `SIGNALLAB_TOKEN` |
| `--secrets files\|system` | Woher die Werte der Geheimnisse in diesem Prozess kommen. | `files` |
| `--secrets-dir PATH` | Ein Ordner mit Geheimnisdateien, eine pro Name. | `/run/secrets/signallab`, wenn er existiert |

`--data-dir`, `--secrets` und `--secrets-dir` betreffen Durchläufe in diesem
Prozess; sie lassen sich nicht mit `--server` kombinieren.

### Dateien und Vorlagen {#run-inputs}

Eine `FILE` ist ein Experiment, wie die App es speichert und exportiert
([[ui:exp.exportJson]] in der Ansicht [[ui:nav.experiment]]). Jede Dokumentversion,
die die App öffnet, funktioniert; eine ältere wird beim Lesen auf den neuesten
Stand gebracht, so wie die App es beim Öffnen tut. Ein Name, der keine Datei ist,
wird unter den [mitgelieferten Vorlagen](#cli-templates) gesucht, mit oder ohne `.json`:

```bash
signallab run tests/stage-cues.json tests/api.json
signallab run osc-ping-reply --param device=192.0.2.20:9000
```

Jedes Experiment — und jede Kombination einer Matrix — wird so geprüft, wie es
der Editor vor dem ersten Durchlauf prüft. Eine defekte dritte Datei stoppt auch
die erste, bevor etwas gesendet wird, mit Exit-Code `2`.

### Parameter und Profile {#run-params}

`--param NAME=VALUE` setzt einen Parameter nur für diesen Durchlauf; die Datei
wird nicht geändert. Der Wert ist alles nach dem ersten `=`, `--param
url=http://127.0.0.1/?q=1` funktioniert also, und `--param note=` setzt einen
leeren Wert. Ein Wert gilt für jedes angegebene Experiment, das diesen Parameter
hat, und muss einen Parameter mindestens eines von ihnen benennen — ein falsch
geschriebener Name wird mit Exit-Code `2` abgelehnt.

`--profile NAME` führt mit einem der Profile des Experiments aus, so wie die
Wahl unter [[ui:exp.profile]] im Editor. Siehe
[Daten und Vorlagen](../experiments/data.md).

### Eine Matrix von Durchläufen {#matrix}

Eine Matrix führt dasselbe Experiment gegen jedes Ziel, jeden Benutzer, jede
Nutzdatengröße aus. Jede Kombination ist ein eigener Durchlauf, mit eigenem
Ergebnis, eigenem Bericht und eigener Suite im JUnit-Bericht.

```bash
signallab run smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest \
  --fail-fast --junit junit.xml
```

Das sind vier Durchläufe: `device` ändert sich am langsamsten, `user` am
schnellsten. Sie werden nach der Datei und ihren Werten benannt:
`smoke.json [device=192.0.2.20:9000, user=admin]`.

- `--matrix NAME=V1,V2` fügt eine Achse hinzu. Derselbe Name erneut fügt ihm
  Werte hinzu. Leerzeichen um Namen und Werte entfallen; ein zweimal angegebener
  Wert läuft einmal.
- `--matrix-file PATH` liest JSON in einer von zwei Formen:

  ```json
  { "device": ["192.0.2.20:9000", "192.0.2.21:9000"], "retries": [1, 3] }
  ```

  fügt Achsen hinzu — jede Kombination läuft, diese Namen nach denen von
  `--matrix`, in alphabetischer Reihenfolge;

  ```json
  [
    { "device": "192.0.2.20:9000", "user": "admin" },
    { "device": "192.0.2.21:9000", "user": "guest" }
  ]
  ```

  listet die Kombinationen selbst auf, jede mit den Achsen von `--matrix`
  gekreuzt. Werte sind Text, Zahlen oder `true`/`false`; ein Komma in einem Wert
  einer Datei bleibt Teil davon.
- Jeder Matrixname muss ein Parameter mindestens eines angegebenen Experiments
  sein. Ein Experiment ohne einen von ihnen läuft einmal, nicht einmal pro Wert,
  den es ignorieren würde.
- Ein Name, der sowohl von `--param` als auch von der Matrix oder sowohl von
  `--matrix` als auch von der Datei gesetzt wird, wird abgelehnt.
- Höchstens **256** Durchläufe aus einem Befehl; mehr wird abgelehnt, bevor
  etwas läuft.

### Auf einem Server ausführen {#run-on-server}

```bash
signallab run tests/stage.json --server http://192.0.2.10:1430 --token-file token.txt
```

Das Experiment kommt von diesem Rechner; der Server führt es mit seinem eigenen
Netzwerk, seinen eigenen [Geheimnissen](../server/index.md#secrets) und seinem
eigenen Datenordner aus und überträgt die Schritte zurück, während sie
geschehen. Der Bericht bleibt auf dem Server — sein Pfad wird ausgegeben —, und
`--report` lädt eine Kopie herunter. Ein Durchlauf auf einem Server ist dort ein
Job wie einer, der in seiner Oberfläche gestartet wurde: Jede angemeldete Seite
zeigt ihn. Wenn `signallab` mitten im Durchlauf verschwindet, wird der Durchlauf
auf dem Server trotzdem beendet und behält seinen Bericht.

Das Token wird aus `--token-file` (oder `SIGNALLAB_TOKEN_FILE`), sonst aus
`SIGNALLAB_TOKEN` gelesen und als `Authorization: Bearer` gesendet. Ein Server,
der nicht erreichbar ist, das Token verweigert oder mitten im Durchlauf aufhört
zu antworten, ist Exit-Code `3`. `signallab doctor --server URL` prüft die
Adresse und das Token für sich.

### Berichte {#reports}

Jeder Durchlauf schreibt denselben Bericht, den die App schreibt. Ohne
`--data-dir` verwenden die Durchläufe einen temporären Ordner, der beim Beenden
von `signallab` entfernt wird, sichern Sie also, was Sie brauchen:

- `--report PATH` kopiert den Bericht: bei einem Durchlauf nach `PATH` selbst,
  bei mehreren in den Ordner `PATH`, als `01-<file>.json`, `02-<file>.json`, …
  in der Reihenfolge, in der sie liefen.
- `--data-dir PATH` behält den Bericht jedes Durchlaufs in diesem Ordner (unter
  `runs/`) und gibt aus, wo jeder liegt.

`--junit PATH` schreibt einen JUnit-XML-Bericht, das Format, das jedes CI-System
liest:

- eine `<testsuite>` pro Durchlauf (pro Kombination einer Matrix), mit der Datei,
  dem Startwert, dem Ergebnis, dem Profil, dem Pfad des Berichts und jedem
  Matrixwert (`param.NAME`) als Eigenschaften;
- ein `<testcase>` pro Knoten, der lief, benannt nach dem Typ und der ID des
  Knotens, mit eigener Zeit;
- ein `<failure>` an dem Knoten, der fehlschlug: die Meldung in der gewählten
  Sprache, der Fehlercode als sein `type` und die Schritte des Knotens mit dem
  technischen Detail;
- ein `<skipped>`-Fall für jeden Knoten, den der Durchlauf nie erreichte (die
  andere Seite einer Verzweigung);
- eine Suite mit einem `<error>`-Fall für ein Experiment, das nicht starten
  konnte, und eine mit einem `<skipped>`-Fall für jeden Durchlauf, den
  `--fail-fast` nicht startete.

```xml
<testsuites name="Signal Lab" tests="6" failures="1" errors="0" time="2.006">
  <testsuite name="HTTP to OSC" tests="6" failures="1" errors="0" skipped="4" time="2.006" timestamp="2026-10-04T16:48:03">
    <properties>
      <property name="file" value="status-branch" />
      <property name="seed" value="42" />
      <property name="outcome" value="failed" />
      <property name="report" value="out/report.json" />
    </properties>
    <testcase name="Start (start)" classname="HTTP to OSC" time="0.001">
      <system-out>Started · seed 42</system-out>
    </testcase>
    <testcase name="HTTP request (request)" classname="HTTP to OSC" time="2.004">
      <failure message="http://127.0.0.1:8080/ refused the connection — nothing is listening on that port" type="transport.refused">…</failure>
    </testcase>
    <testcase name="Status branch (branch)" classname="HTTP to OSC" time="0.000">
      <skipped message="not reached in this run" />
    </testcase>
    …
  </testsuite>
</testsuites>
```

Ein HTTP-Knoten unter [Last](../experiments/load.md) fügt unter seinem letzten
Schritt eine Zeile pro Schwellenwert an, gehalten oder nicht
(`✕ p95 < 100 ms · 152.58 ms`); ein Schwellenwert, der nicht gehalten wird, lässt
den Durchlauf und seinen JUnit-Fall fehlschlagen (`type="load.threshold"`).

### Geheimnisse {#secrets}

Ein Experiment liest ein Geheimnis als `{{secret.NAME}}`. Für Durchläufe in
diesem Prozess kommt der Wert aus:

| `--secrets` | Woher der Wert von `NAME` kommt |
| --- | --- |
| `files` (der Standard) | Der Umgebungsvariablen `SIGNALLAB_SECRET_NAME`; sonst einer Datei namens `NAME` in `--secrets-dir` (standardmäßig `/run/secrets/signallab`, wenn dieser Ordner existiert — das Secrets-Layout von Docker). |
| `system` | Der Windows-Anmeldeinformationsverwaltung — wo die App die Werte ihrer [[ui:exp.secrets]] hält. Linux hat keinen Anmeldedatenspeicher, den `signallab` liest: Dort schlägt `--secrets system` mit Exit-Code `3` fehl. |

Ein abschließender Zeilenumbruch einer Datei ist nicht Teil des Werts, und ein
leerer Wert zählt als nicht gesetzt. Namen bestehen aus Buchstaben, Ziffern und
`_`, beginnen nicht mit einer Ziffer und sind höchstens 128 Zeichen lang; ein
Wert umfasst höchstens 16 KiB.

```bash
SIGNALLAB_SECRET_API_TOKEN="$API_TOKEN" signallab run tests/api.json
```

Ein Geheimnis, das nicht gesetzt ist, stoppt den Durchlauf vor jedem Verkehr, mit
Exit-Code `2` und dem fehlenden Namen. Ein Wert wird nie ausgegeben: Schritte,
Fehler, Berichte und der JUnit-Bericht zeigen an seiner Stelle `••••`. Mit
`--server` sind die Geheimnisse die des Servers.

### Was `run` ausgibt {#run-output}

Während ein Durchlauf läuft, ist jeder Schritt eine Zeile auf der
Standardfehlerausgabe — seine Zeit seit dem Start, der Knoten, sein Zustand und
was er tat —, wie die Zeitleiste der App. Wenn er endet, sagt eine Zeile auf der
Standardausgabe, wie es ausging:

```text
▶ HTTP check (http-check) · seed 1185927457137919
     0.000  Start         Running
     0.000  Start         Passed · Started · seed 1185927457137919
     0.000  HTTP request  Running
     2.004  HTTP request  Failed · URL — http://127.0.0.1:8080/ refused the connection — nothing is listening on that port
✖ HTTP check failed after 2 s: HTTP request · URL — http://127.0.0.1:8080/ refused the connection — nothing is listening on that port
  Technical details: error sending request for url (http://127.0.0.1:8080/): … (os error 10061)
  To run it again with the same random values: --seed 1185927457137919
```

Ein fehlgeschlagener Durchlauf nennt den verwendeten Startwert: `--seed` mit
dieser Zahl führt ihn erneut mit denselben Zufallswerten aus. Nach dem Ergebnis
folgen auf der Standardfehlerausgabe, wonach jeder Emulator des Durchlaufs
gefragt wurde — Anfragen, wie viele keine Regel annahm, wie viele fehlschlugen
und die Treffer pro Regel:

```text
✔ Retry a flaky API passed in 7 ms
  Flaky API: 3 requests, 0 without a rule, 0 failed · #1 3
```

und was jedes Störungs-Relais tat: was es empfing, verwarf und drosselte, und
jede Phase als weitergeleitet / empfangen:

```text
  127.0.0.1:19110 → 127.0.0.1:19100: 155 datagrams, 38 dropped, 0 throttled · lan 0.0–2.0 s 39/39, wifi 2.0–4.0 s 39/39, offline 4.0–6.0 s 0/38, lan 6.0–8.0 s 38/38
```

Ein Relais über TCP bewegt Blöcke von Datenströmen, keine Datagramme, und
verwirft nichts, seine Zeile zählt also Blöcke, Verbindungen, Zurücksetzungen,
halboffene Verbindungen und die Male, die ein Datenstrom durch die
Bandbreitengrenze zurückgehalten wurde:

```text
  127.0.0.1:19120 → 127.0.0.1:19101: 14 chunks, 2 connections, 1 reset, 0 half-open, 3 held back
```

Ein HTTP-Knoten unter [Last](../experiments/load.md) meldet seinen Fortschritt
höchstens einmal pro Sekunde als Schritt.

Bei mehreren Durchläufen beenden eine Zeile pro Durchlauf und eine Zählung die
Ausgabe: `3 runs: 2 passed, 1 failed`. Mit `--fail-fast` werden die Durchläufe,
die er nicht startete, auf der Standardfehlerausgabe gezählt.

Mit `--json` ist jede Zeile ein Objekt mit einem `type`:

```json
{"type":"started","experiment":"Empty experiment","file":"empty","job_id":1,"overridden":false,"profile":null,"seed":1,"started_ms":1791132204585}
{"type":"step","node_id":"start","state":"passed","detail":"Started","message_key":"exp.step.started","message_params":{"seed":1},"job_id":1,"ts":1791132204585}
{"type":"ended","experiment":"Empty experiment","file":"empty","outcome":"passed","seed":1,"params":{},"steps":[…],"report_path":"…","started_ms":1791132204585,"ended_ms":1791132204585,…}
{"type":"summary","total":1,"passed":1,"failed":0,"not_started":0,"exit_code":0}
```

Die Zeilen `started`, `ended` und `error` tragen `file` (das Argument, wie es
angegeben wurde) und, in einer Matrix, `matrix` (die Werte der Kombination).
`ended` ist das ganze Ergebnis des Durchlaufs: `outcome` (`passed`, `failed` oder
`stopped`), `seed`, `profile`, `params`, `error`, jeder Schritt, `emulators` und
`impairments`, wenn der Durchlauf sie hatte, und `report_path`. Der letzte Schritt
einer Last trägt `load` mit jeder Zahl, die sie maß: `planned`, `sent`, `ok`,
`failed`, `missed`, `rps`, `error_rate`, `min_ms`, `mean_ms`, `max_ms`, `p50_ms`
bis `p99_ms`, `statuses`, jede Sekunde (`seconds`), das `histogram` und das Urteil
jedes Schwellenwerts (`thresholds`).

## `validate` {#cli-validate}

```text
signallab validate [OPTIONS] <FILE>...
```

Prüft Experimente so, wie es der Editor vor einem Durchlauf tut — den Graphen,
jedes Feld, Vorlagen, Parameter und Geheimnisse — und sendet nichts. Endet mit
`0`, wenn jedes starten würde.

Es nimmt die Eingaben von [`run`](#cli-run): Dateien und Vorlagen, `--param`,
`--profile`, `--matrix`, `--matrix-file` sowie `--server`, `--token-file`,
`--secrets`, `--secrets-dir`. Mit `--server` prüft der Server sie, gegen seine
eigenen Geheimnisse.

```text
✔ tests/stage.json: Stage cues would run
  Rehearsal: Would not run: …
```

Ein Problem, das nur ein anderes Profil des Dokuments hat, wird unter ihm
aufgeführt, lässt die Prüfung aber nicht fehlschlagen. Mit `--json` eine Zeile
pro Experiment (pro Kombination): `{"experiment", "file", "valid": true,
"profile_issues": […]}`, oder `"valid": false` mit `error` und `exit_code`.

## `send` {#cli-send}

```text
signallab send <osc|udp|http|ws|mqtt> …
```

Sendet eine Nachricht über dieselben Befehle, die die Ansichten der App
verwenden, es sind also dieselben Bytes auf der Leitung. Ein Senden liest keine
Bibliothek und keine Geheimnisse.

Exit-Codes: `0` gesendet, `1` das Senden schlug fehl, `2` ein Argument ist falsch.

Ein `<host:port>` ist eine IP-Adresse oder ein Hostname und ein Port:
`127.0.0.1:9000`, `[::1]:9000` (eine IPv6-Adresse in Klammern) oder
`device.local:9000`. Ein Name wird aufgelöst, wenn der Befehl läuft, seine
IPv4-Adresse genommen, wenn er eine hat, `localhost:9000` erreicht also einen
Empfänger auf `127.0.0.1`. Ein Name, der sich nicht auflösen lässt, lässt das
Senden fehlschlagen (`1`); ein Ziel ohne Port ist ein ungültiges Argument (`2`).

### `send osc` {#cli-send-osc}

```text
signallab send osc <host:port> <address> [ARG]...
```

Eine OSC-Nachricht. Argumente werden mit einem Präfix typisiert oder abgeleitet:

| Argument | OSC-Typ |
| --- | --- |
| `i:3` | int32 |
| `f:0.5` | float32 |
| `d:1.5` | float64 (double) |
| `h:64` | int64 |
| `s:text` | Zeichenkette |
| `b:de ad be ef` | Blob, als Hex-Bytes |
| `T`, `F` | wahr, falsch |
| `N` | nil |
| `3`, `-3` | eine ganze Zahl ohne Präfix ist int32 |
| `2.5` | eine Dezimalzahl ohne Präfix ist float32 |
| alles andere | Zeichenkette |

```bash
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main 3
# ✔ sent /cue/go (32 bytes) → 127.0.0.1:9000
```

Setzen Sie ein Argument mit Leerzeichen in Anführungszeichen:
`"s:hello world"`. `s:7` sendet den Text `7`. Die Adresse muss mit `/` beginnen;
eine, die das nicht tut, ist ein ungültiges Argument (`2`).

::: warning Git Bash unter Windows
Git Bash wandelt Argumente, die mit `/` beginnen, in Windows-Pfade um,
`/cue/go` kommt also als `C:/Program Files/Git/cue/go` an. Schreiben Sie
`//cue/go` oder führen Sie mit `MSYS_NO_PATHCONV=1` aus. PowerShell und `cmd`
sind nicht betroffen.
:::

Siehe [OSC](../protocols/osc.md).

### `send udp` {#cli-send-udp}

```text
signallab send udp <host:port> (--text TEXT | --hex HEX)
```

Ein UDP-Datagramm. Geben Sie die Nutzdaten als `--text` oder als `--hex`-Bytes
an: `"de ad be ef"`, `deadbeef` oder `0xDE,0xAD`.

```bash
signallab send udp 127.0.0.1:7000 --text "PLAY 1"
signallab send udp 127.0.0.1:7000 --hex "de ad be ef"
```

### `send http` {#cli-send-http}

```text
signallab send http <METHOD> <URL> [OPTIONS]
```

Eine HTTP-Anfrage. Die Statuszeile geht an die Standardfehlerausgabe —
`HTTP 200 OK · 3 ms · 1,234 B` — und der Antwort-Body an die Standardausgabe,
sodass er weitergeleitet werden kann. Ein Body länger als 256 KiB wird dort
abgeschnitten, und die Standardfehlerausgabe sagt es.

| Option | Was sie tut | Standard |
| --- | --- | --- |
| `-H`, `--header "Name: value"` | Ein Anfrage-Header; für mehr wiederholen. | |
| `--body TEXT` | Der Anfrage-Body. `@FILE` sendet den Inhalt einer Textdatei. | keiner |
| `--expect-status STATUS` | Endet mit `1`, es sei denn, die Antwort hat diesen Status. | jeder Status ist `0` |
| `--timeout MS` | Millisekunden, die auf die Antwort gewartet wird. | `10000` |
| `-u`, `--user NAME:PASSWORD` | Anmeldedaten, als Basic gesendet. | |
| `--digest` | Mit `--user`: stattdessen die Digest-Challenge des Servers beantworten (MD5 oder SHA-256). | aus |
| `--bearer TOKEN` | `Authorization: Bearer TOKEN` senden. Nicht mit `--user`. | |

```bash
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab send http POST http://127.0.0.1:8080/cue -H "Content-Type: application/json" --body '{"cue": 1}'
signallab send http GET http://127.0.0.1:8080/admin -u admin:secret --digest
```

Ohne `--expect-status` ist jede Antwort ein Erfolg, eine `500` eingeschlossen.
Eine Anfrage, die keine Antwort erhält (abgelehnt, Zeitüberschreitung, ein Name,
der sich nicht auflösen lässt), ist Exit-Code `1`, und ebenso eine
Digest-Challenge, die nicht beantwortet werden konnte — der Grund wird nach der
Antwort ausgegeben.

::: tip
Argumente sind für andere Benutzer desselben Rechners sichtbar. Bewahren Sie
echte Passwörter für Experimente auf, wo sie [Geheimnisse](#secrets) sind.
:::

Siehe [HTTP](../protocols/http.md).

### `send ws` {#cli-send-ws}

```text
signallab send ws <URL> [OPTIONS]
```

Ein WebSocket-Austausch: mit `ws://…` oder `wss://…` verbinden, eine Nachricht
senden, optional auf die Antwort warten, schließen. Der Handshake und was
gesendet wurde, gehen an die Standardfehlerausgabe, die Antwort an die
Standardausgabe (Binärnachrichten als Hex).

| Option | Was sie tut | Standard |
| --- | --- | --- |
| `--text TEXT` | Diese Textnachricht senden. | nichts gesendet |
| `--hex HEX` | Diese Bytes als Binärnachricht senden. Nicht mit `--text`. | |
| `-H`, `--header "Name: value"` | Ein Header für die Upgrade-Anfrage; für mehr wiederholen. | |
| `--protocol NAME` | Ein anzubietendes Subprotokoll; für mehr wiederholen, in der Reihenfolge der Vorliebe. | |
| `--expect TEXT` | Auf eine Nachricht warten, die diesen Text enthält. | |
| `--expect-regex REGEX` | Auf eine Nachricht warten, die zu diesem regulären Ausdruck passt. | |
| `--wait` | Auf irgendeine Nachricht warten. | |
| `--timeout MS` | Millisekunden, die auf die Antwort gewartet wird. | `2000` |

```bash
signallab send ws ws://127.0.0.1:9001/echo --text '{"ping": 1}' --expect '"ping"'
signallab send ws ws://127.0.0.1:9001/feed --wait        # nothing sent: the server's first message
```

Ohne `--expect`, `--expect-regex` oder `--wait` verbindet es, sendet und
schließt, ohne zu warten. Kommt die erwartete Antwort nicht rechtzeitig, ist der
Exit-Code `1`. Siehe [WebSocket](../protocols/websocket.md).

### `send mqtt` {#cli-send-mqtt}

```text
signallab send mqtt <host:port> <topic> [payload] [--qos 0|1|2] [--retain]
```

Eine MQTT-3.1.1-Veröffentlichung über einfaches TCP, ohne Anmeldedaten, als
eigener Client mit einer frischen Client-ID, es verdrängt also nie eine
Verbindung, die schon da ist. Ohne Port liegt der Broker auf `1883`. Ein Topic
ist ein Topic: Enthält es ein `+` oder `#` oder ist es leer, wird der Befehl
abgelehnt, bevor er sich verbindet (`2`).

| Option | Was sie tut | Standard |
| --- | --- | --- |
| `[payload]` | Die Nutzdaten. | leer |
| `--qos 0\|1\|2` | Die Dienstgüte. | `0` |
| `--retain` | Behält sie als Retained-Wert des Topics. Leere Nutzdaten mit `--retain` löschen ihn. | aus |

```bash
signallab send mqtt 127.0.0.1:1883 lab/light/1/set on --qos 1
signallab send mqtt 127.0.0.1 lab/light/1/state "" --retain     # clear the retained value
```

Siehe [MQTT](../protocols/mqtt.md).

## `fire` {#cli-fire}

```text
signallab fire <signal> [--library PATH]
```

Sendet ein Signal einer Signalbibliothek genau so, wie die Ansicht
[[ui:nav.signals]] der App es auslöst: OSC-, UDP-, HTTP- und MQTT-Signale. Das
Signal wird über seine ID, sonst über seinen Namen gefunden, ohne Beachtung der
Groß- und Kleinschreibung; ein Name, den mehrere Signale teilen, wird mit ihren
IDs abgelehnt.

| Option | Was sie tut | Standard |
| --- | --- | --- |
| `<signal>` | Die ID oder der Name des Signals. | erforderlich |
| `--library PATH` | Die Bibliotheksdatei. | `signals.json` im Datenordner der App |

```bash
signallab fire "Fader value"
signallab fire go --library show/signals.json
```

Die Bibliothek wird nur gelesen, nie angelegt oder geändert. Siehe
[Signale](../tools/signals.md).

## `emulate` {#cli-emulate}

```text
signallab emulate [OPTIONS] <FILE|NAME>...
```

Spielt die Gegenseite — eine HTTP-API, ein OSC-, UDP- oder TCP-Gerät, einen
MQTT-Broker —, bis <kbd>Ctrl</kbd>+<kbd>C</kbd> oder `--for`, und gibt jede
Anfrage aus, sobald sie beantwortet wird. Eine `FILE` enthält einen Emulator,
eine Liste von ihnen oder eine ganze Bibliothek, wie die App sie schreibt; ein
`NAME` ist die ID oder der Name eines Eintrags in der Bibliothek der App (der
der Ansicht [[ui:nav.emulators]]).

| Option | Was sie tut | Standard |
| --- | --- | --- |
| `-p`, `--param NAME=VALUE` | Ein Wert, den seine Vorlagen als `{{NAME}}` lesen; für mehr wiederholen. | |
| `--bind IP:PORT` | Stattdessen dort empfangen. Nur mit einem Emulator. | der eigene des Emulators |
| `--for SECONDS` | Nach so langer Zeit stoppen. | bis <kbd>Ctrl</kbd>+<kbd>C</kbd> |
| `--seed N` | Der Startwert seiner Zufallsentscheidungen: eine zufällige Reihenfolge, Jitter, Generatoren. | |
| `--check` | Die Emulatoren prüfen und beenden, ohne einen Port zu öffnen. | aus |
| `--library PATH` | Die Bibliothek, in der Namen gesucht werden. | `emulators.json` im Datenordner der App |
| `--server URL`, `--token-file PATH` | Sie auf einem Server starten, ihnen über seine API folgen und sie am Ende stoppen. | |
| `--secrets`, `--secrets-dir` | Wie bei [`run`](#cli-run). | |

```text
$ signallab emulate tests/orders-api.json --for 60
Orders API (http) answering on 127.0.0.1:18099
answering for 60 s
+   1.209 s Orders API  #1  GET /orders/42 → 200 OK · 12 B  1 ms  ← 127.0.0.1:55744
+   1.209 s Orders API  —  GET /nothing → 404 Not Found · 20 B  2 ms  ← 127.0.0.1:55745
Orders API: 2 requests, 1 without a rule, 0 failed · #1 1
```

Jede Zeile ist die Zeit seit dem Start, der Emulator, die Regel, die geantwortet
hat (`#1`, oder `—` für keine), die Anfrage und was sie bekam, die Zeit, die sie
dauerte, und wer sie gesendet hat. Am Ende die Zählungen jedes Emulators:
Anfragen, wie viele keine Regel annahm, wie viele fehlschlugen, wie viele auf
einen Ausfall trafen oder unzugestellt blieben, wenn es welche gab, und die
Treffer pro Regel.

Exit-Codes: `0`, wenn er bei <kbd>Ctrl</kbd>+<kbd>C</kbd> oder `--for` stoppt;
`2`, wenn ein Emulator ungültig ist; `3`, wenn sein Port belegt ist oder sich
nicht öffnen lässt oder ein Socket fehlschlägt, während er antwortet.

Starten Sie ihn in einer Pipeline im Hintergrund, testen Sie das System dagegen
und lesen Sie die Zählungen am Ende:

```bash
signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
npm test          # the system under test, configured for the emulator's address
wait              # the last lines of mock.ndjson are the counts
```

Auf einem Server wird ein mit `--server` gestarteter Emulator gestoppt, wenn
`signallab` normal endet; einen, den ein abgebrochener Prozess zurückgelassen
hat, lässt sich über die Oberfläche des Servers stoppen. Siehe
[Emulatoren](../tools/emulators.md).

## `emulators` {#cli-emulators}

```text
signallab emulators [--library PATH]
```

Listet die Emulatoren der Bibliothek auf: ID, Name, Protokoll, Adresse und wie
viele Regeln. `--library PATH` liest eine andere Bibliotheksdatei statt
`emulators.json` im Datenordner der App. `signallab emulate <id>` startet einen.

Die Bibliothek wird nur gelesen. Gibt es keine solche Datei — die App legt die
in ihrem Datenordner beim ersten Start an —, sagt der Befehl es und endet mit `2`.

## `templates` {#cli-templates}

```text
signallab templates
```

Listet die mitgelieferten Vorlagen auf, die `run` und `validate` über ihren
Namen annehmen. Es sind die der App:

| Name | In der App | Parameter |
| --- | --- | --- |
| `empty` | [[ui:exp.templateEmpty]] | |
| `http-check` | [[ui:exp.templateHttp]] | |
| `status-branch` | [[ui:exp.templateBranch]] | |
| `parallel-flows` | [[ui:exp.templateParallel]] | |
| `osc-ping-reply` | [[ui:exp.templatePingReply]] | `device` = `127.0.0.1:9000` |
| `poll-until-ready` | [[ui:exp.templatePoll]] | `device` = `127.0.0.1:9000` |
| `flaky-api` | [[ui:exp.templateFlaky]] | `api` = `http://127.0.0.1:18080` |
| `fault-phases` | [[ui:exp.templateFaults]] | |
| `dependency-outage` | [[ui:exp.templateOutage]] | `api` = `http://127.0.0.1:18090` |
| `websocket-echo` | [[ui:exp.templateWsEcho]] | `service` = `ws://127.0.0.1:9001/echo` |

Jede Vorlage zeigt auf Loopback. `flaky-api`, `fault-phases` und
`dependency-outage` bringen ihre eigenen Emulatoren mit, laufen also, ohne dass
sonst etwas lauscht — ein schneller Weg zu sehen, wie `signallab` arbeitet.

## `nodes` {#cli-nodes}

```text
signallab nodes
```

Gibt als JSON aus, woraus ein Experiment besteht: die Form und die Regeln des
Dokuments, jede Art von Knoten mit ihrer Beschriftung, Beschreibung, Feldern,
Ausgängen und einem Beispiel, das die Engine annimmt, die Sprache der
`{{template}}`-Vorlagen, Lastprofile und das Emulator-Dokument. Es ist das, was
ein Assistent über [`signallab mcp`](mcp.md) liest, um ein Experiment zu
schreiben; für Menschen sagt [Knoten](../experiments/nodes.md) dasselbe mit mehr
Worten.

## `mcp` {#cli-mcp}

```text
signallab mcp [OPTIONS]
```

Stellt Signal Lab einem KI-Assistenten über das Model Context Protocol bereit,
auf stdin und stdout. Alles darüber — die Einrichtung in Claude Code, Claude
Desktop, Cursor oder VS Code, seine Optionen und seine Werkzeuge — steht unter
[Assistenten (MCP)](mcp.md).

## `doctor` {#cli-doctor}

```text
signallab doctor [--server URL] [--token-file PATH]
```

Sagt, was zwischen Signal Lab und der Technik stehen könnte, und endet mit `1`,
wenn etwas dazwischensteht. Seine Zeilen folgen [`--lang`](#language) wie der
Rest der Kommandozeile:

- das Netzwerk, in dem dieser Rechner ist: seinen Namen und seine Adresse;
- den Datenordner: ob er beschreibbar ist (ein noch nicht angelegter ist in
  Ordnung — die App legt ihn bei der ersten Verwendung an);
- die Firewall: unter Windows, für `signallab` und die Desktop-App jeweils, ob
  eine Regel andere Rechner in der Art von Netzwerk herein lässt, in dem der
  Rechner gerade ist (privat, Domäne oder öffentlich), oder eine Regel sie
  blockiert — was ein *Abbrechen* an der Systemabfrage zurücklässt; unter Linux,
  ob ufw oder firewalld an ist und der Befehl, der einen Port öffnet;
- mit `--server` (oder `SIGNALLAB_SERVER`): ob der Server antwortet und das
  Token annimmt.

```text
signallab [[version]]
Network: LAB-PC · 192.0.2.15
Data folder: C:\Users\lab\Documents\SignalLab — writable
Firewall · signallab (C:\…\Signal Lab\signallab.exe) · private network: not allowed yet — signallab firewall allow
Firewall · app (C:\…\Signal Lab\signal-lab.exe) · private network: allowed
✖ 1 thing in the way
```

`--json` gibt dasselbe als ein Objekt aus. Siehe
[Problemlösung](../reference/troubleshooting.md).

## `firewall` {#cli-firewall}

```text
signallab firewall allow [--public]
```

Lässt unter Windows andere Rechner Signal Lab erreichen — was ein Monitor oder
ein Warten braucht, um ein Gerät zu hören. Windows fragt zuerst nach
Administratorrechten; dann werden die eingehenden Regeln von `signallab` und der
Desktop-App (neben ihr gefunden oder dort, wo die Installer sie hinlegen) durch
je eine Erlaubnisregel ersetzt, blockierende Regeln eingeschlossen, in privaten
Netzwerken und Domänennetzen.

| Option | Was sie tut |
| --- | --- |
| `--public` | Auch in öffentlichen Netzwerken — das WLAN eines Veranstaltungsorts ist oft eines. |

Exit-Codes: `0` erledigt; `3`, wenn die Administratorabfrage abgelehnt wird oder
die Änderung fehlschlägt. Unter Linux wird nichts geändert: Es gibt den ufw-
oder firewalld-Befehl aus, der die Ports öffnet, auf denen Sie empfangen, und
endet mit `0`.

Die Firewall ändert sich nur, wenn Sie dies ausführen; nichts anderes in
`signallab` berührt sie.

## `version` {#cli-version}

```text
signallab version
```

Gibt `signallab [[version]]` aus — mit `--json`, `{"version": "[[version]]"}`.
`signallab --version` gibt dieselbe Version aus.

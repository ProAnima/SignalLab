---
title: Assistenten (MCP)
description: Mit signallab mcp kann ein KI-Assistent über das Model Context Protocol Experimente bauen, prüfen und ausführen, Nachrichten senden, empfangen und Emulatoren spielen.
---

# Signal Lab für einen Assistenten: `signallab mcp`

`signallab mcp` ist ein Server für das [Model Context Protocol](https://modelcontextprotocol.io).
Ein Assistent in Claude Code, Claude Desktop, Cursor, VS Code oder einem anderen
MCP-Client startet ihn und kann dann:

- erfahren, woraus ein Experiment besteht, eines schreiben, prüfen, ausführen und
  Schritt für Schritt nachlesen, warum es fehlgeschlagen ist;
- eine einzelne OSC-Nachricht, ein Datagramm, eine HTTP-Anfrage, eine WebSocket-Nachricht oder eine
  MQTT-Veröffentlichung senden und auf einem Port empfangen, was ein Gerät sendet;
- ein Signal aus Ihrer Bibliothek senden;
- eine Abhängigkeit spielen — eine HTTP-API, ein OSC-, UDP- oder TCP-Gerät, einen MQTT-Broker —
  und nachlesen, was Ihr System daran gesendet hat;
- frühere Durchläufe nachlesen und zwei davon vergleichen.

Jede Aktion läuft über dieselben Engine-Befehle, die auch die App verwendet. Ein
Experiment, das der Assistent ausführt, ist also derselbe Durchlauf, den die App machen würde, mit demselben
Bericht, und jeder Fehler ist so formuliert, wie die Oberfläche ihn formuliert.

::: warning
Senden, Durchläufe und Emulatoren bringen echten Verkehr ins Netzwerk. Sagen Sie dem Assistenten,
mit welchen Geräten er sprechen darf; die mitgelieferten Vorlagen zeigen auf Loopback
(`127.0.0.1`).
:::

## Einrichten {#setup}

`signallab` wird mit der Desktop-App geliefert und liegt nach der Installation in Ihrem `PATH`
(siehe [Installieren](cli.md#install)). Der Client startet `signallab mcp` selbst und
spricht mit ihm über stdin und stdout; Sie führen es nicht von Hand aus.

### Die Konfiguration ausgeben {#print-config}

`--print-config` gibt aus, was ein Client braucht, mit dem vollständigen Pfad dieses
`signallab`:

| Befehl | Was er ausgibt |
| --- | --- |
| `signallab mcp --print-config claude-code` | Die Kommandozeile `claude mcp add`. |
| `signallab mcp --print-config claude-desktop` | Den Eintrag `mcpServers` für die Konfigurationsdatei von Claude Desktop. |
| `signallab mcp --print-config cursor` | Denselben Eintrag `mcpServers`, für die `mcp.json` von Cursor. |
| `signallab mcp --print-config vscode` | Den Eintrag `servers` für die `.vscode/mcp.json` von VS Code. |

Für einen Assistenten, der mit einem [Laborserver](#on-a-server) arbeitet, fügen Sie
`--server URL` hinzu: Die ausgegebene Konfiguration enthält ihn dann, mit einem Platzhalter
für das Token.

### Claude Code {#claude-code}

Führen Sie die Zeile aus, die `--print-config claude-code` ausgibt, zum Beispiel:

```bash
claude mcp add signallab -- "C:\Program Files\Signal Lab\signallab.exe" mcp
```

### Claude Desktop und Cursor {#claude-desktop}

Tragen Sie den Eintrag in die Konfiguration des Clients ein — bei Claude Desktop
`claude_desktop_config.json`, bei Cursor `mcp.json` — und starten Sie den Client neu:

```json
{
  "mcpServers": {
    "signallab": {
      "command": "C:\\Program Files\\Signal Lab\\signallab.exe",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

### VS Code {#vscode}

```json
{
  "servers": {
    "signallab": {
      "type": "stdio",
      "command": "/usr/bin/signallab",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

### Andere Clients {#other-clients}

Jeder Client, der einen stdio-Server startet, funktioniert auf dieselbe Weise: Der Befehl ist
`signallab` (oder sein vollständiger Pfad), die Argumente sind `mcp` und beliebige der
[Optionen](#options). Unter Linux kann auch das Server-Image als Befehl dienen:

```bash
docker run -i --rm --network host --entrypoint signallab ghcr.io/proanima/signallab:[[version]] mcp
```

## Optionen {#options}

| Option | Was sie tut | Standard |
| --- | --- | --- |
| `--server URL` | Experimente, Sendevorgänge und Emulatoren auf diesem Signal-Lab-Server ausführen (siehe [Auf einem Laborserver](#on-a-server)). | `SIGNALLAB_SERVER` |
| `--token-file PATH` | Eine Datei mit dem Token des Servers. | `SIGNALLAB_TOKEN_FILE`, sonst `SIGNALLAB_TOKEN` |
| `--data-dir PATH` | Wo Durchläufe und ihre Berichte aufbewahrt werden. Nicht zusammen mit `--server`. | der Datenordner der App (`Documents/SignalLab`) |
| `--library PATH` | Die Signalbibliothek für `list_signals` und `fire_signal`. | die `signals.json` der App |
| `--emulators PATH` | Die Emulator-Bibliothek für `list_emulators` und `start_emulator`. | die `emulators.json` der App |
| `--secrets files\|system` | Woher die Werte der Geheimnisse für Durchläufe auf diesem Rechner kommen, wie bei [`run`](cli.md#secrets). Nicht zusammen mit `--server`. | `files` |
| `--secrets-dir PATH` | Ein Ordner mit Geheimnisdateien, eine pro Name. Nicht zusammen mit `--server`. | `/run/secrets/signallab`, wenn er existiert |
| `--lang <code>` | Die Sprache der Ergebnisse und Fehler. | `SIGNALLAB_LANG`, sonst die Locale, sonst `en` |
| `--print-config CLIENT` | Die Konfiguration eines Clients ausgeben und beenden: `claude-code`, `claude-desktop`, `cursor` oder `vscode`. | |

Durchläufe legen ihre Berichte im Datenordner der App ab, wo die App auch ihre eigenen aufbewahrt,
sodass sie über die Sitzung hinaus erhalten bleiben.

## Werkzeuge {#tools}

Werkzeuge, die nur lesen, sind als schreibgeschützt markiert, sodass ein Client sie ohne
Rückfrage ausführen lassen kann. Werkzeuge, die die Außenwelt erreichen — sie senden, empfangen oder starten
etwas —, sind entsprechend markiert, und ein Client kann Sie vor jedem Aufruf fragen. Keines ist als
zerstörerisch markiert.

| Werkzeug | Was es tut | Erreicht die Außenwelt |
| --- | --- | --- |
| `describe_nodes` | Das Experimentdokument, jede Art von Knoten mit ihren Feldern, Ausgängen und einem Beispiel, die Sprache der `{{template}}`-Vorlagen, Lastprofile und das Emulator-Dokument. | nein |
| `list_templates` | Die mitgelieferten Experimente, mit ihren Parametern. | nein |
| `get_template` | Ein mitgeliefertes Experiment als Dokument. | nein |
| `validate_experiment` | Prüft ein Experiment so, wie es der Editor vor einem Durchlauf tut; sendet nichts. | nein |
| `run_experiment` | Führt ein Experiment bis zum Ende aus und meldet jeden Schritt. | ja |
| `send_osc` | Eine OSC-Nachricht. | ja |
| `send_udp` | Ein UDP-Datagramm. | ja |
| `send_http` | Eine HTTP-Anfrage. | ja |
| `send_mqtt` | Eine MQTT-3.1.1-Veröffentlichung. | ja |
| `send_ws` | Ein WebSocket-Austausch. | ja |
| `listen` | Was eine Weile auf einem UDP-Port eintrifft. | ja |
| `list_signals` | Die Signale Ihrer Bibliothek. | nein |
| `fire_signal` | Sendet ein Signal der Bibliothek. | ja |
| `list_emulators` | Die Emulatoren Ihrer Bibliothek. | nein |
| `start_emulator` | Startet einen Emulator. | ja |
| `emulator_exchanges` | Was ein laufender Emulator empfangen und beantwortet hat. | nein |
| `set_emulator_down` | Schaltet einen laufenden Emulator aus oder wieder ein. | ja |
| `list_runs` | Die Berichte früherer Durchläufe. | nein |
| `compare_runs` | Zwei Durchläufe nebeneinander. | nein |
| `list_jobs` | Was gerade läuft. | nein |
| `stop_job` | Stoppt einen laufenden Job. | ja |

### Experimente {#tools-experiments}

`describe_nodes` liest der Assistent, bevor er ein Experiment schreibt; es ist dasselbe wie
[`signallab nodes`](cli.md#cli-nodes). `list_templates` und
`get_template` liefern funktionierende Beispiele zum Ausführen oder Anpassen.

`validate_experiment` und `run_experiment` nehmen das Experiment auf einem von drei
Wegen entgegen — genau eines der ersten drei Argumente — und die übrigen passen den Lauf an:

| Argument | Was es ist |
| --- | --- |
| `document` | Ein Experimentdokument, wie die App es speichert. |
| `file` | Der Pfad einer Experimentdatei auf dem Rechner, auf dem `signallab` läuft. |
| `template` | Der Name einer mitgelieferten Vorlage. |
| `params` | Parameterwerte für diesen Durchlauf: `{"name": "value"}`; Zahlen und Booleans werden als Text genommen. |
| `profile` | Mit diesem Profil des Dokuments ausführen; `""` für die Standardwerte. |
| `seed` | `run_experiment`: der Startwert der Zufallswerte. |
| `timeout` | `run_experiment`: Sekunden, die der Durchlauf dauern darf, 1 bis 300 (Standard 300). |

`run_experiment` antwortet, wenn der Durchlauf zu Ende ist: bestanden, fehlgeschlagen oder gestoppt, seine
Dauer und sein Startwert, jeder Schritt mit dem, was er getan hat oder warum er fehlschlug, was jeder
Emulator gefragt wurde, was jedes Störungs-Relais getan hat, und der Pfad des Berichts. Ein
Durchlauf, der fehlschlägt, ist eine normale Antwort — die Schritte sagen, warum —, kein fehlgeschlagener Aufruf.

### Einzelne Nachrichten {#tools-send}

| Werkzeug | Argumente |
| --- | --- |
| `send_osc` | `target` (`host:port`), `address`, `args`: Zahlen (ganze → int32, oder int64 jenseits seines Bereichs; sonst float32), Strings, Booleans, `null` oder `{"type": "int"\|"float"\|"str"\|"long"\|"double"\|"bool"\|"blob"\|"nil", "value": …}`. |
| `send_udp` | `target` und `text` oder `hex` (`"de ad be ef"`). |
| `send_http` | `method`, `url`, `headers` (`{"Name": "value"}`), `body`, `timeout_ms` (Standard 10000), `auth`: `{"scheme": "basic"\|"digest", "username", "password"}` oder `{"scheme": "bearer", "token"}`. Gibt den Status, die Zeit, die Header und den Body zurück — dessen ersten 16 KiB. |
| `send_mqtt` | `broker` (`host:port`, Port 1883, wenn keiner angegeben ist), `topic`, `payload`, `qos` (0, 1 oder 2), `retain`. Leere Nutzdaten mit `retain` löschen einen Retained-Wert. |
| `send_ws` | `url` (`ws://` oder `wss://`), `text` oder `hex`, `headers`, `protocols`, und um auf die Antwort zu warten `expect` (enthält), `expect_regex` oder `wait` (irgendeine Nachricht); `timeout_ms` 1 bis 120000 (Standard 2000). Gibt den Handshake zurück, was gesendet wurde und die Antwort, deren JSON geparst ist, wenn sie JSON ist. |

Das sind die Befehle, die die Ansichten der App verwenden; siehe [`signallab send`](cli.md#cli-send).

### Empfangen {#tools-listen}

`listen` öffnet für eine Weile einen UDP-Port **auf dem Rechner, auf dem `signallab mcp` läuft**,
und gibt zurück, was eingetroffen ist: OSC-Nachrichten dekodiert, andere Datagramme als Text
und Hex.

| Argument | Was es ist | Standard |
| --- | --- | --- |
| `bind` | `IP:port`, z. B. `0.0.0.0:9000`. | erforderlich |
| `protocol` | `osc` oder `udp`. | `osc` |
| `seconds` | Wie lange empfangen wird, 0,1 bis 60. | 5 |
| `max` | Nach so vielen Datagrammen aufhören, 1 bis 1000. | 100 |

Trifft auf `0.0.0.0` nichts ein, erinnert die Antwort den Assistenten daran, die Firewall zu prüfen
([`signallab doctor`](cli.md#cli-doctor)). Mit `--server` wird
`listen` abgelehnt: Auf einem Server empfängt ein Experiment mit einem Warteknoten dort.

### Signale und Emulatoren {#tools-library}

`list_signals` und `fire_signal` verwenden Ihre Signalbibliothek — die `signals.json` der App,
`--library` oder einen dem Aufruf mitgegebenen Pfad `library`. Ein Signal wird über seine ID oder seinen Namen
gesendet, genau so, wie die App es sendet.

`list_emulators` nennt die Emulatoren Ihrer Bibliothek. `start_emulator` startet
einen — ein Dokument in `emulator` oder die ID oder der Name eines Bibliothekseintrags in `name` — und
gibt seine Job-ID und Adresse zurück; er antwortet nach seinen Regeln bis `stop_job`.
`bind` verlegt ihn auf ein anderes `IP:port`, `params` gibt Werte an, die seine Vorlagen lesen,
`seed` legt seine Zufallsentscheidungen fest. `emulator_exchanges` (`job_id`, und `after` für
nur die neueren) listet auf, was eingetroffen ist und was jede Regel beantwortet hat.
`set_emulator_down` (`job_id`, `down` und `fault`: `unavailable`, `reset` oder
`timeout`) zieht einem laufenden Emulator den Stecker, bis er wieder eingeschaltet wird:
HTTP trifft auf das Fehlverhalten (`unavailable` antwortet 503), ein TCP-Gerät und ein MQTT-Broker
trennen Verbindungen, OSC und UDP antworten nicht. Siehe [Emulatoren](../tools/emulators.md).

### Durchläufe und Jobs {#tools-runs}

`list_runs` liest die Berichte früherer Durchläufe, die neuesten zuerst — die eines Experiments,
wenn `experiment` es nennt, höchstens `limit` (1 bis 500, Standard 50) — mit den Zahlen jedes
Lastschritts. `compare_runs` nimmt zwei ihrer Namen, `a` (vorher) und
`b` (nachher), und stellt Latenzen, Fehlerrate, erreichte Rate und verpasste Anfragen jedes Lastschritts
nebeneinander, wobei eine Änderung um 5 % oder mehr in die falsche Richtung als
Verschlechterung markiert wird. Siehe [Durchläufe und Berichte](../experiments/runs.md).

`list_jobs` listet auf, was läuft — Monitore, Generatoren, Emulatoren, Durchläufe —,
und `stop_job` stoppt einen anhand seiner ID.

## Ergebnisse und Fehler {#results}

Jede Antwort ist Text für das Modell und zugleich dasselbe als strukturierte Daten. Ein Fehler
ist als Fehler markiert und enthält den Fehler der Engine — einen stabilen `code`, seine
Werte, den Knoten und das Feld, um die es geht —, formuliert in der mit `--lang` gewählten Sprache.
Argumente, die der Assistent falsch angegeben hat, kommen in Worten zurück, mit denen er sie korrigieren kann.

## Fortschritt und Abbrechen {#progress}

Verlangt der Client bei `run_experiment` Fortschritt, wird jeder Schritt gemeldet,
sobald er geschieht (der Knoten und sein Zustand), sodass der Assistent — und Sie — den Durchlauf
vorankommen sehen. Das Abbrechen eines Aufrufs stoppt ihn; das Abbrechen von `run_experiment` stoppt den Durchlauf
selbst, so wie es [[ui:common.stop]] in der App tut.

Schließt der Client die Verbindung, werden laufende Aufrufe noch beendet, dann beendet sich
`signallab mcp`.

## Auf einem Laborserver {#on-a-server}

Mit `--server http://192.0.2.10:1430` finden die Experimente, Sendevorgänge, Signale und
Emulatoren **auf diesem Server** statt, über seine API — mit seinem Netzwerk, seinen
Geheimnissen und seinem Datenordner —, sodass der Assistent Technik erreicht, die nur das Labor
erreichen kann. Geben Sie das Token in der Umgebung des Clients an:

```json
{
  "mcpServers": {
    "signallab": {
      "command": "signallab",
      "args": ["mcp", "--server", "http://192.0.2.10:1430"],
      "env": { "SIGNALLAB_TOKEN": "<the server's token>" }
    }
  }
}
```

Was auf diesem Rechner bleibt: Die Signal- und Emulator-Bibliotheken (die der App oder
`--library` und `--emulators`) und die Dateien, die ein Aufruf nennt (`file`, `library`),
werden hier gelesen, und ihr Inhalt wird an den Server gesendet; `listen` wird abgelehnt.
Siehe [Signal Lab als Server betreiben](../server/index.md).

## Sicherheit {#safety}

- Der Assistent kann nur tun, was die Werkzeuge tun, und jedes Werkzeug ist ein eigener
  Befehl der App: Er kann nichts erreichen, was die App nicht erreichen könnte.
- Werkzeuge, die senden, empfangen oder etwas starten, sind als die Außenwelt erreichend
  markiert; Ihr Client entscheidet, ob er Sie vor jedem Aufruf fragt.
- Die Werte von Geheimnissen erreichen den Assistenten nie: Ein Experiment nennt sie
  `{{secret.NAME}}`, und jedes Ergebnis zeigt an ihrer Stelle `••••`.
- Ein Emulator oder ein Empfänger öffnet einen Port auf dem Rechner, auf dem er läuft;
  `list_jobs` und `stop_job` zeigen und beenden, was noch läuft.

## Protokoll {#protocol}

Für Autoren von Clients: JSON-RPC 2.0 über stdio, eine Nachricht pro Zeile; stdout
trägt nur Protokollnachrichten, und alles für Menschen geht nach stderr.
Protokollversionen `2025-06-18`, `2025-03-26` und `2024-11-05` (die neueste, wenn
der Client eine andere verlangt), Batches, `ping`, `tools/list` und `tools/call`;
Fortschritt als `notifications/progress` für einen Aufruf, der ein `progressToken` gesendet hat,
Abbruch durch `notifications/cancelled`. Die `instructions` des Servers sagen dem
Modell, wie die Werkzeuge zusammenspielen.

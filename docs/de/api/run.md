---
title: Durchläufe
description: POST /api/run führt ein Experiment auf einem Signal-Lab-Server aus und antwortet mit seinem Ergebnis oder strömt seine Schritte als NDJSON-Zeilen.
---

# Ein Experiment ausführen

Um ein Experiment auf einem Server aus einem Skript oder einer Pipeline
auszuführen und zu wissen, wie es lief, senden Sie es an `POST /api/run`. Der
Server führt es bis zum Ende aus und antwortet mit dem Ergebnis — oder, wenn Sie
es verlangen, mit jedem Schritt, sobald er geschieht. Das verwendet
[`signallab run --server`](../automation/cli.md#cli-run).

Es ist derselbe Durchlauf wie [[ui:exp.run]] des Editors und
[`experiment_start`](commands.md#experiment_start): ein Job, den jede offene
Seite sieht und stoppen kann, dieselben Ereignisse, derselbe Bericht im
Datenordner.

## Die Anfrage {#request}

```http
POST /api/run
Authorization: Bearer <token>
Content-Type: application/json

{ "template": "osc-ping-reply", "overrides": { "device": "192.0.2.20:9000" }, "seed": 42, "timeout": 30 }
```

Der Body nennt ein Experiment — ein `document` oder eine `template`, nicht
beides — und womit es ausgeführt wird:

| Feld | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `document` | object | — | Ein Experiment, wie der Editor es speichert und exportiert. Ältere Versionen werden migriert, wie beim Öffnen einer Datei |
| `template` | string | — | Eine mitgelieferte Vorlage nach Dateiname, mit oder ohne `.json` (unten) |
| `overrides` | object | `{}` | Parameterwerte nur für diesen Durchlauf. Werte dürfen Text, Zahlen oder boolesche Werte sein; jeder muss ein Parameter des Experiments sein |
| `profile` | string | das des Dokuments | Mit diesem Profil ausführen; `""` führt mit den Vorgaben aus |
| `seed` | number | das des Dokuments, sonst ein neues | 0 bis 9007199254740991; derselbe Startwert zieht dieselben Zufallswerte |
| `timeout` | number | `300` | Sekunden, bevor der Durchlauf mit `run.timeout` fehlschlägt; 1 bis 300 |

Ein Feld, das der Server nicht kennt, wird abgelehnt (`400`,
`api.run_invalid`). Das Dokument selbst wird gelesen wie eine geöffnete Datei,
ist also höchstens 4 MiB groß.

Die mitgelieferten Vorlagen — die Experimente, die der Editor unter
[[ui:exp.templates]] anbietet:

| `template` | Was es ist |
| --- | --- |
| `empty` | Start und Ende |
| `http-check` | Ein GET von `http://127.0.0.1:8080/`, dann eine Prüfung auf Status 200 |
| `status-branch` | Ein GET von `http://127.0.0.1:8080/`; bei 200 eine OSC-Nachricht, sonst eine Verzögerung von 500 ms |
| `parallel-flows` | Eine [[ui:exp.node.fork]] in ein GET von `http://127.0.0.1:8080/` und einen Log-Eintrag, Seite an Seite, dann [[ui:exp.node.join]] |
| `osc-ping-reply` | Ein OSC `/ping` an den Parameter `device` (`127.0.0.1:9000`), dann ein Warten von 2 s auf `/pong` auf `127.0.0.1:9001` |
| `poll-until-ready` | Eine [[ui:exp.node.loop]], die `device` über OSC nach `/status` fragt, bis es `ready` antwortet, höchstens 10-mal |
| `flaky-api` | Eine emulierte API (Parameter `api`), die scheitert, bevor sie funktioniert, in einer [[ui:exp.node.loop]] gefragt, bis sie 200 antwortet |
| `fault-phases` | Ein UDP-Gerät hinter einem Störungs-Relais, das 8 s lang angesprochen wird, während das Relais sauber, verlustbehaftet, offline und wieder sauber wird |
| `dependency-outage` | Eine emulierte API (Parameter `api`), die 2 s lang heruntergefahren wird, während eine [[ui:exp.node.loop]] sie fragt, bis sie wieder 200 antwortet |
| `websocket-echo` | Eine Verbindung zum Parameter `service` (`ws://127.0.0.1:9001/echo`), eine Nachricht, ein Warten auf ihr Echo, eine Prüfung, ein Schließen |

Öffnen Sie eine unter [[ui:exp.templates]], um ihre Knoten und Parameter zu
sehen; siehe [Experimente](../experiments/index.md).

## Das Ergebnis {#result}

Standardmäßig ist die Antwort `200` mit `Content-Type: application/json`,
gesendet, wenn der Durchlauf geendet hat: ein JSON-Objekt, das Ergebnis des
Durchlaufs.

```json
{
  "job_id": 12,
  "experiment": "OSC ping → reply",
  "outcome": "passed",
  "seed": 42,
  "profile": null,
  "overridden": true,
  "params": { "device": "192.0.2.20:9000" },
  "started_ms": 1759600000000,
  "ended_ms": 1759600000310,
  "steps": [ { "job_id": 12, "ts": 1759600000001, "node_id": "start", "state": "running", "detail": "", "message_key": null, "message_params": null }, … ],
  "report_path": "/data/runs/run-1759600000000-12.json"
}
```

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job des Durchlaufs |
| `experiment` | string | Der Name des Experiments |
| `outcome` | string | `passed`, `failed` oder `stopped` |
| `seed` | number | Der Startwert, mit dem er lief: geben Sie ihn als `seed` zurück, um dieselben Werte zu ziehen |
| `profile` | string or null | Das Profil, mit dem er lief |
| `overridden` | boolean | Einige Werte kamen aus `overrides` |
| `params` | object | Jeder Parameterwert, den der Durchlauf verwendete |
| `started_ms`, `ended_ms` | number | Millisekunden seit 1970 |
| `error` | `EngineError` | Warum er fehlschlug: sein erster Fehler. Weggelassen, wenn er bestand |
| `steps` | object[] | Jeder Schritt in der Reihenfolge, in der er geschah, wie [`experiment://step`](events.md#event-experiment-step) |
| `emulators` | object[] | Was jeder Knoten [[ui:exp.node.emulator]] empfangen und beantwortet hat: `node`, `name`, `protocol`, `local`, `counts`. Weggelassen, wenn es keine gibt |
| `impairments` | object[] | Was das Relais jedes Knotens [[ui:exp.node.impairment]] tat, Phase für Phase. Weggelassen, wenn es keine gibt |
| `report_path` | string | Der Bericht des Durchlaufs auf dem Server; laden Sie ihn mit [`/api/files`](index.md#files) herunter. Weggelassen, wenn keiner geschrieben wurde |
| `report_error` | `EngineError` | Warum der Bericht nicht geschrieben werden konnte. Sonst weggelassen |

Geheime Werte sind in allem maskiert. Die Berichtsdatei enthält dieselben
Schritte; siehe [Durchläufe und Berichte](../experiments/runs.md).

Während der Durchlauf läuft, sendet der Server alle 15 s ein Leerzeichen. JSON
ignoriert Leerraum vor einem Wert, daher lässt sich das Ergebnis weiterhin
parsen, und ein Proxy hält einen langen, stillen Durchlauf nicht für eine tote
Verbindung.

## Den Schritten folgen {#lines}

Um die Schritte zu sehen, sobald sie geschehen, verlangen Sie NDJSON:

```http
Accept: application/x-ndjson
```

Die Antwort ist `200` mit `Content-Type: application/x-ndjson`: ein JSON-Objekt
pro Zeile, jedes mit einem `type`.

| `type` | Wann | Der Rest der Zeile |
| --- | --- | --- |
| `started` | Zuerst, einmal | `job_id`, `experiment`, `seed`, `profile`, `overridden`, `started_ms` |
| `step` | Jeder Schritt | Der Schritt, wie [`experiment://step`](events.md#event-experiment-step) |
| `heartbeat` | Alle 15 s | Nichts |
| `ended` | Zuletzt, einmal | Das Ergebnis, wie oben |

```text
{"job_id":12,"experiment":"OSC ping → reply","seed":42,"profile":null,"overridden":true,"started_ms":1759600000000,"type":"started"}
{"job_id":12,"ts":1759600000001,"node_id":"start","state":"running","detail":"","message_key":null,"message_params":null,"type":"step"}
…
{"job_id":12,"experiment":"OSC ping → reply","outcome":"passed",…,"type":"ended"}
```

Lesen Sie die Zeilen bis `ended`; ignorieren Sie ein `type`, das Sie nicht
kennen. Der Server sendet `X-Accel-Buffering: no`, damit ein nginx-Proxy jede
Zeile sofort weitergibt.

## Status und Ausgang {#status}

Ein HTTP-Fehlerstatus bedeutet **kein Durchlauf gestartet**; der Body ist ein
[`EngineError`](index.md#errors):

| Status | Code | Warum |
| --- | --- | --- |
| `400` | `api.run_invalid` | Der Body ist keine Durchlauf-Anfrage: kein JSON, ein unbekanntes Feld, eine Überschreibung, die kein Text, keine Zahl und kein boolescher Wert ist |
| `400` | `api.run_source` | Weder `document` noch `template` oder beides |
| `415` | `command.json_required` | Nicht `Content-Type: application/json` |
| `422` | `api.template_unknown` | Keine mitgelieferte Vorlage dieses Namens |
| `422` | `file.json_invalid`, `file.too_large`, `doc.*` | Das Dokument kann nicht gelesen werden |
| `422` | jeder Validierungscode, `run.override_unknown`, `profile.active_missing`, `run.limit_range`, `seed.range`, `secret.missing`, `transport.address_in_use`… | Das Experiment kann nicht starten: Es validiert nicht, ein Wert liegt außerhalb des Bereichs, ein Geheimnis ist nicht gespeichert, ein Port, auf dem es lauscht, ist belegt |

Sobald der Durchlauf gestartet ist, ist der Status `200`, was auch geschieht:
Lesen Sie `outcome` im Ergebnis.

| `outcome` | Bedeutung |
| --- | --- |
| `passed` | Jeder Schritt bestand und [[ui:exp.node.end]] wurde erreicht |
| `failed` | Ein Schritt schlug fehl oder der Durchlauf dauerte länger als sein `timeout` (`run.timeout`); `error` sagt, welcher und warum |
| `stopped` | Er wurde gestoppt, bevor er endete: durch `job_stop`, [[ui:app.stopAll]] oder das Herunterfahren des Servers. `steps` enthält die Schritte, die er erreichte; es wird kein Bericht gespeichert |

## Ein Client, der weggeht {#disconnect}

Die Verbindung zu schließen stoppt den Durchlauf nicht. Er ist ein Job auf dem
Server: Er läuft bis zum Ende und speichert seinen Bericht, wie ein im Browser
gestarteter Durchlauf, wenn der Tab geschlossen wird. Finden Sie ihn mit
[`jobs_list`](commands.md#jobs_list), stoppen Sie ihn mit
[`job_stop`](commands.md#job_stop), und lesen Sie seinen Bericht danach mit
[`experiment_runs`](commands.md#experiment_runs). Wenn der Server
herunterfährt, wird der Durchlauf gestoppt und ein weiterhin verbundener Client
erhält `"outcome": "stopped"`.

## Beispiele {#examples}

Führen Sie eine mitgelieferte Vorlage aus und warten Sie auf den Ausgang:

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
curl -sS -X POST "$SERVER/api/run" \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"template":"osc-ping-reply","overrides":{"device":"192.0.2.20:9000"},"timeout":30}' \
  | jq -r .outcome
```

Senden Sie Ihr eigenes Experiment mit einem geänderten Parameter und geben Sie
jeden Schritt aus, sobald er geschieht:

```bash
jq '{document: ., overrides: {api: "http://192.0.2.10:8080"}, profile: ""}' smoke.json |
  curl -sSN -X POST "$SERVER/api/run" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    -H "Accept: application/x-ndjson" --data @- |
  jq -r 'select(.type == "step") | "\(.node_id)  \(.state)  \(.detail)"'
```

`-N` hält `curl` davon ab, die Zeilen zurückzuhalten. Um eine Pipeline bei einem
fehlgeschlagenen Durchlauf scheitern zu lassen, prüfen Sie `outcome`:

```bash
outcome=$(curl -sS -X POST "$SERVER/api/run" -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" -d '{"template":"http-check"}' | jq -r .outcome)
[ "$outcome" = "passed" ]
```

## Von der Kommandozeile {#cli}

[`signallab run`](../automation/cli.md#cli-run) mit `--server <url>` läuft auf
einem Server über diesen Endpunkt: Es sendet das gelesene Experiment mit
`overrides`, `seed` und `timeout`, verlangt NDJSON und gibt jeden Schritt aus,
sobald seine Zeile eintrifft. Das Token kommt aus `--token-file`, sonst aus
`SIGNALLAB_TOKEN`. Mit `--report` lädt es den Bericht über `/api/files`
herunter. Ein Server, der 60 s lang nichts sendet — nicht einmal einen
Heartbeat —, gilt als weg.

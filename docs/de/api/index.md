---
title: HTTP-API
description: Steuern Sie einen Signal-Lab-Server aus Skripten, CI und anderen Werkzeugen mit denselben Befehlen und Ereignissen, die seine eigene Oberfläche verwendet.
---

# Die HTTP-API

Um Signal Lab aus einem Skript, einer CI-Pipeline oder einem anderen Werkzeug
zu steuern, sprechen Sie mit einem Signal-Lab-Server (`signal-lab-server` oder
dem Docker-Image) über HTTP. Die Oberfläche, die der Server in einem Browser
zeigt, verwendet genau diese API: Jede Schaltfläche ist ein Aufruf von
`/api/invoke/<command>`, jede Live-Zahl trifft auf `/api/events` ein. Alles,
was eine Person auf der Seite des Servers tun kann, kann ein Skript also auch.

Die Desktop-App hat keine HTTP-API: Ihr Fenster erreicht ihre Engine innerhalb
der App. Um auf einem Desktop zu automatisieren, führen Sie einen Server auf
demselben Rechner aus (siehe [den Server](../server/index.md)) oder verwenden
Sie die Kommandozeile [`signallab`](../automation/cli.md).

## Endpunkte {#endpoints}

| Methode und Pfad | Was sie tut | Token |
| --- | --- | --- |
| `GET /api/health` | Ob der Server antwortet, seine Version, ob er ein Token verlangt | nicht nötig |
| `POST /api/invoke/<command>` | Führt einen Engine-Befehl aus: JSON-Argumente hinein, JSON-Ergebnis hinaus ([Befehle](commands.md)) | nötig |
| `POST /api/run` | Führt ein Experiment bis zum Ende aus: das Ergebnis oder seine Schritte als Zeilen ([Durchläufe](run.md)) | nötig |
| `GET /api/events` | WebSocket jedes Engine-Ereignisses ([Ereignisse](events.md)) | nötig |
| `GET /api/files?path=…` | Eine Datei, die die Engine im Datenordner geschrieben hat, als Download | nötig |
| `GET /api/openapi.json` | Diese API, beschrieben in OpenAPI 3.1 | nötig |
| `GET /login`, `POST /login` | Die Anmeldeseite und das Formular für Browser | nicht nötig |
| `POST /logout` | Beendet die Sitzung eines Browsers | eine Sitzung |

Jeder andere Pfad unter `/api/` antwortet mit `404` und dem Code
`api.not_found`; eine Methode, die ein Pfad nicht annimmt
(`GET /api/invoke/…`), ist `405` mit leerem Body. Alles andere ist die
Oberfläche; auf einem Server mit Token wird ein Browser ohne Sitzung zuerst zu
`/login` geschickt.

## Basis-URL {#base-url}

Ein Server empfängt auf `http://127.0.0.1:1430`, sofern nichts anderes mit
`--listen` (oder `SIGNALLAB_LISTEN`) angegeben wird. Das Docker-Image empfängt
auf jeder Netzwerkkarte, `0.0.0.0:1430`. Die Beispiele auf diesen Seiten
verwenden:

```bash
SERVER=http://127.0.0.1:1430
```

Der Server spricht einfaches HTTP. Für HTTPS stellen Sie einen Reverse-Proxy
vor ihn, der TLS beendet, und starten den Server mit `--secure-cookie`.

## Authentifizierung {#authentication}

Ein ohne Token gestarteter Server empfängt nur auf Loopback und braucht keine
Authentifizierung: Jeder auf diesem Rechner darf ihn verwenden. Ein Server, den
andere erreichen können, hat immer ein Token, und dann muss jede Anfrage außer
`/api/health` und `/login` es mitführen.

**Skripte** senden das Token im `Authorization`-Header:

```bash
TOKEN=$(cat token.txt)
curl -fsS "$SERVER/api/invoke/jobs_list" -X POST \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json"
```

Der Header muss genau `Bearer`, ein Leerzeichen und das Token sein. Ein
fehlendes oder falsches Token ist `401` mit dem Code `auth.required`.

**Browser** melden sich einmal unter `/login` mit dem Token an und erhalten ein
Sitzungs-Cookie, `signallab_session`: `HttpOnly`, `SameSite=Strict`, 7 Tage
lang gültig und `Secure`, wenn der Server mit `--secure-cookie` läuft.
`POST /logout` beendet sie. Ein falsches Token im Anmeldeformular kostet eine
Sekunde vor der Antwort, was das Raten langsam macht. Sitzungen liegen im
Arbeitsspeicher des Servers: Ein Neustart meldet jeden Browser ab, während
Skripte mit dem Token nicht betroffen sind. Der Server hält höchstens 1024
Sitzungen; darüber hinaus geht die älteste weg.

Woher das Token kommt, ist Sache der Einrichtung des Servers (`--token-file`,
`SIGNALLAB_TOKEN` oder die Datei `token`, die `--generate-token` im Datenordner
anlegt): siehe [Server-Sicherheit](../server/security.md). Ein Token hat
mindestens 24 Zeichen und keine Leerzeichen; `signal-lab-server token` gibt ein
neues aus.

::: warning
Wer das Token hat, kann den Server Verkehr senden lassen. Halten Sie die Datei,
in der es liegt, nur für sich lesbar, und setzen Sie es nie in eine URL — der
Server liest dort ohnehin kein Token.
:::

## Host und Origin {#host-origin}

Zwei Prüfungen laufen vor allem anderen, auf jedem Pfad, `/api/health`
eingeschlossen.

**`Host`.** Der `Host`-Header muss diesen Server benennen:

- Loopback-Namen bestehen immer: `localhost`, Namen mit der Endung
  `.localhost`, `127.x.x.x` und `[::1]`.
- Namen, die mit `--allowed-host` (oder `SIGNALLAB_ALLOWED_HOSTS`) angegeben
  wurden, bestehen.
- Ein Server mit Token und ohne `--allowed-host` antwortet auf jeden Namen.

Alles andere ist `403` mit `auth.host`. Sprechen Sie den Server unter einem
Namen an, den er akzeptiert: `curl` sendet den Host der URL, die Sie ihm geben.

**`Origin`.** Eine Anfrage, die etwas ändert (jede Methode außer `GET` und
`HEAD`), und das WebSocket-Upgrade müssen von der eigenen Seite des Servers
kommen, wenn sie einen `Origin`-Header mitführen: Ihr Host und Port müssen
gleich `Host` sein. Andernfalls lautet die Antwort `403` mit `auth.origin`;
`Origin: null` wird ebenfalls abgelehnt. Skripte und `curl` senden kein
`Origin` und bestehen diese Prüfung — sie brauchen trotzdem das Token.

**Nur JSON.** `POST /api/invoke/…` und `POST /api/run` nehmen
`Content-Type: application/json` an (Parameter wie `; charset=utf-8` sind in
Ordnung). Alles andere ist `415` mit `command.json_required`. Eine Webseite auf
einer anderen Seite kann das nicht senden, ohne den Server zuerst zu fragen,
und der Server stimmt nie zu.

## Einen Befehl aufrufen {#invoke}

```http
POST /api/invoke/<command>
Content-Type: application/json

{ "argument": "value", … }
```

- Der Body ist ein JSON-Objekt mit den Argumenten des Befehls. Ein Befehl ohne
  Argumente nimmt `{}` oder einen leeren Body (der `Content-Type`-Header ist
  trotzdem nötig).
- Argumentnamen sind camelCase, wie die Oberfläche sie sendet: `jobId`,
  `nodeId`. Als Argument übergebene Objekte (`config`, `request`, `document`,
  `library`…) behalten die Feldnamen, die die Engine schreibt, und die sind
  meist snake_case: `timeout_ms`, `port_start`.
- Ein Argument, das ein Befehl nicht kennt, ist ein Fehler, wird nie ignoriert:
  `422` mit `command.args_invalid`, das den Befehl nennt, und den Worten des
  Parsers in `detail`. So ist es auch bei einem fehlenden Pflichtargument. Ein
  Befehl ohne Argumente liest den Body gar nicht.
- Ein optionales Argument darf weggelassen oder als `null` gesendet werden.
- Die Antwort ist `200` mit dem Ergebnis des Befehls als JSON. Ein Befehl, der
  nichts zurückzugeben hat, antwortet mit `null`.

Jeder Befehl mit seinen Argumenten und seinem Ergebnis steht in
[Befehle](commands.md).

## Fehler {#errors}

| Status | Wann | Body |
| --- | --- | --- |
| `200` | Der Befehl lief; `/api/run` hat den Durchlauf gestartet | Das Ergebnis |
| `400` | Der Body ist kein JSON; eine Durchlauf-Anfrage kann nicht gelesen werden | `EngineError`: `command.args_invalid`, `api.run_invalid`, `api.run_source` |
| `400` | `/api/files` ohne `path` | Reiner Text des Webservers, kein `EngineError` |
| `401` | Kein Token oder ein falsches | `EngineError`: `auth.required` |
| `403` | Ein `Host` oder `Origin`, den der Server ablehnt | `EngineError`: `auth.host`, `auth.origin` |
| `404` | Kein solcher API-Pfad; eine Datei, die nicht im Datenordner liegt | `EngineError`: `api.not_found`, `file.not_found` |
| `405` | Eine Methode, die der Pfad nicht annimmt | Leer |
| `413` | Ein Anfrage-Body über 24 MiB | Reiner Text des Webservers, kein `EngineError` |
| `413` | Ein Download über 256 MiB | `EngineError`: `file.too_large` |
| `415` | Nicht `application/json` | `EngineError`: `command.json_required` |
| `422` | Der Befehl schlug fehl oder der Durchlauf konnte nicht starten | `EngineError`: jeder Code der Engine |
| `500` | Eine Datei konnte nicht gelesen werden | `EngineError`: `file.io` |

Ein unbekannter Befehlsname ist `422` mit `command.unknown`.

Jeder Fehler, den die Engine meldet, hat eine Gestalt, den `EngineError`:

```json
{
  "code": "transport.refused",
  "params": { "target": "http://127.0.0.1:8080/" },
  "node": "request",
  "field": { "key": "url" },
  "detail": "error sending request for url (http://127.0.0.1:8080/): tcp connect error: Connection refused (os error 111)"
}
```

| Feld | Was es ist |
| --- | --- |
| `code` | Was schiefging: eine stabile Kennung. Jeder Code und seine Meldung stehen in [Fehlermeldungen](../reference/errors.md), gruppiert nach dem Teil vor dem Punkt (zum Beispiel [`transport`](../reference/errors.md#transport)) |
| `params` | Die Werte, die die Meldung nennt, alle als Text. Weggelassen, wenn es keine gibt |
| `node` | Der Experimentknoten, um den es geht. Weggelassen, wenn es keinen gibt |
| `field` | Das Feld, um das es geht: `key` (benannt in [Felder](../reference/errors.md#fields)) und `index`, 1-basiert, für wiederholte Felder wie einen Header. Weggelassen, wenn es keines gibt |
| `detail` | Die eigenen Worte des Betriebssystems, eines Parsers oder einer Bibliothek, auf Englisch. Weggelassen, wenn es keine gibt |

Verzweigen Sie nach `code`, nie nach `detail`. Die geheimen Werte, die ein
Durchlauf oder ein einzelnes Senden verwendet, sind in jedem Fehler, den er
meldet, maskiert (`••••`).

Eine Anfrage, die ihren Server erreicht, aber einen Fehlerstatus bekommt oder
gar keinen, ist kein fehlgeschlagener Befehl: `http_request` antwortet mit
`200` und der Antwort, und `ok`, `error` und `cause` sagen, was geschah. Siehe
[`http_request`](commands.md#http_request).

## Grenzen {#limits}

| Was | Grenze | An der Grenze |
| --- | --- | --- |
| Ein Anfrage-Body | 24 MiB | `413` |
| Ein Experimentdokument | 4 MiB | `file.too_large` |
| Ein Download von `/api/files` | 256 MiB | `413`, `file.too_large` |
| Die Länge eines Durchlaufs | 300 s | Der Durchlauf schlägt mit `run.timeout` fehl |
| Das Feedback-Formular (`feedback_send`) | 15 MiB insgesamt | `feedback.too_large` |
| Ereignisse, die auf einen WebSocket warten | 4096 | Er erhält [`server://lagged`](events.md#event-server-lagged) mit der Zahl, die er verpasste |

## Ereignisse {#events}

`GET /api/events`, zu einem WebSocket hochgestuft, strömt jedes Ereignis, das
die Engine sendet — die Schritte eines Durchlaufs, das Ende eines Jobs, die
Nachrichten eines Monitors, die Frames des Inspektors —, als Textnachrichten
an jeden verbundenen Client:

```json
{ "event": "job://ended", "payload": { "job_id": 7, "kind": "storm", "error": null } }
```

Er nimmt dieselben Token- und `Origin`-Regeln an wie der Rest. Jeder Kanal und
seine Nutzdaten stehen in [Ereignisse](events.md).

## Dateien {#files}

`GET /api/files?path=<path>` lädt eine Datei herunter, die die Engine im
Datenordner des Servers geschrieben hat: einen Bericht eines Durchlaufs
(`report_path` des Ergebnisses eines Durchlaufs), einen Export des Experiments
oder des Inspektors, die Signal- oder Emulatorbibliothek. `path` ist der Pfad,
den die Engine Ihnen gegeben hat, auf dem Server (URL-kodiert):

```bash
curl -fsS -G "$SERVER/api/files" --data-urlencode "path=/data/runs/run-1759600000000-3.json" \
  -H "Authorization: Bearer $TOKEN" -o report.json
```

- Nur Dateien innerhalb des Datenordners werden ausgeliefert. Alles andere, ein
  Ordner oder eine Datei, die es nicht gibt, ist `404` mit `file.not_found`.
- Die Antwort ist `application/octet-stream` mit
  `Content-Disposition: attachment`. Zeichen des Dateinamens außer Buchstaben,
  Ziffern, `.`, `_` und `-` werden zu `_`.
- Eine Datei über 256 MiB ist `413` mit `file.too_large`.

Was der Datenordner enthält, steht in [Dateien und Ordner](../reference/files.md).

## Zustand {#health}

`GET /api/health` ist offen: Es braucht kein Token, nur einen `Host`, den der
Server akzeptiert.

```bash
curl -fsS "$SERVER/api/health"
```

```json
{ "status": "ok", "version": "[[version]]", "auth": true }
```

`auth` sagt, ob Anfragen ein Token brauchen. `signal-lab-server healthcheck`
fragt dieselbe Adresse, auf der der Server empfängt, und endet mit `0`, wenn er
antwortet; die Zustandsprüfung des Docker-Images führt ihn aus.

## OpenAPI-Beschreibung {#openapi}

`GET /api/openapi.json` beschreibt diese API in OpenAPI 3.1: die Endpunkte,
jeden Befehl mit seinen Argumenten, und die Durchlauf-Anfrage und das Ergebnis.
Es braucht das Token wie der Rest von `/api/`. Diese Seiten sind die
vollständige Referenz; wo die beiden voneinander abweichen, folgen diese Seiten
der Engine.

## Jobs {#jobs}

Lang laufende Arbeit — ein Monitor, ein Generator, ein Burst, ein Relais, eine
Verbindung, ein Emulator, ein Durchlauf — ist ein Job. Ein Befehl, der einen
startet, gibt seine `JobInfo` zurück, sobald er läuft:

```json
{ "id": 4, "kind": "osc-monitor", "label": "OSC monitor 0.0.0.0:9000", "params": { "bind": "0.0.0.0:9000" }, "started_ms": 1759600000000 }
```

| Feld | Was es ist |
| --- | --- |
| `id` | Die Nummer des Jobs, eindeutig, solange der Server läuft; andere Befehle nehmen sie als `jobId` oder `id` |
| `kind` | `experiment`, `osc-monitor`, `osc-gen`, `http-burst`, `netsim`, `storm`, `scan`, `beacon`, `discovery`, `mqtt`, `websocket` oder `emulator` |
| `label` | Eine englische Zeile, für Logs |
| `params` | Die Werte, aus denen das Label besteht (Ziel, Bind, Host…). Weggelassen, wenn es keine gibt |
| `started_ms` | Wann er startete, Millisekunden seit 1970 |

Diese Befehle starten einen Job: `experiment_start`, `osc_monitor_start`,
`osc_generator_start`, `http_burst_start`, `netsim_start`, `storm_start`,
`scan_start`, `broadcast_beacon_start`, `discovery_start`, `mqtt_connect`,
`ws_connect` und `emulator_start`. Der Server protokolliert jeden Start mit der
Adresse des Clients, der ihn angefordert hat; `POST /api/run` startet ebenfalls
einen Job.

- [`jobs_list`](commands.md#jobs_list) listet die laufenden Jobs,
  [`job_stop`](commands.md#job_stop) stoppt einen,
  [`jobs_stop_all`](commands.md#jobs_stop_all) stoppt jeden.
- Ein Job, der von selbst endet oder fehlschlägt, sendet
  [`job://ended`](events.md#event-job-ended). Ein Job, den Sie stoppen, sendet
  nichts mehr: `job_stop` mit der Antwort `true` ist die Bestätigung.
- Jobs gehören dem Server, nicht dem Client, der sie gestartet hat. Die Seite zu
  schließen oder das Skript zu beenden stoppt sie nicht, jeder Client sieht sie
  und kann sie stoppen, und ein Server, der herunterfährt, stoppt sie alle.

## Ein vollständiges Beispiel {#example}

Fragen Sie, ob der Server läuft, starten Sie mit einem Befehl einen kleinen
HTTP-Emulator, führen Sie das mitgelieferte Experiment `http-check` dagegen aus
und warten Sie auf das Ergebnis, stoppen Sie dann den Emulator. `jq` holt
Felder aus den Antworten.

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)            # leave out with a loopback server without a token
AUTH="Authorization: Bearer $TOKEN"
JSON="Content-Type: application/json"

# 1. Up? Which version? Does it want a token?
curl -fsS "$SERVER/api/health"
# {"status":"ok","version":"[[version]]","auth":true}

# 2. One command: an HTTP emulator on 127.0.0.1:8080 that answers GET / with 200
EMULATOR=$(curl -fsS -X POST "$SERVER/api/invoke/emulator_start" -H "$AUTH" -H "$JSON" -d '{
  "emulator": { "name": "Example", "bind": "127.0.0.1:8080", "protocol": "http",
                "routes": [ { "method": "GET", "path": "/", "responses": [ { "body": "ok" } ] } ] }
}' | jq .id)

# 3. Run the bundled experiment that expects 200 from http://127.0.0.1:8080/, and wait
curl -sS -X POST "$SERVER/api/run" -H "$AUTH" -H "$JSON" -d '{"template":"http-check"}' \
  | jq '{outcome, error, report_path}'
# {"outcome":"passed","error":null,"report_path":"/data/runs/run-1759600000000-2.json"}

# 4. Stop the emulator
curl -fsS -X POST "$SERVER/api/invoke/job_stop" -H "$AUTH" -H "$JSON" -d "{\"id\":$EMULATOR}"
# true
```

`curl -f` verwandelt einen Fehlerstatus in einen fehlgeschlagenen Befehl; lassen
Sie es weg (wie in Schritt 3), um den `EngineError` im Body zu sehen.

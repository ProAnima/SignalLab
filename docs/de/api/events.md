---
title: Ereignisse
description: Der WebSocket der Engine-Ereignisse auf einem Signal-Lab-Server und jeder Kanal mit seinen Nutzdaten und wann er gesendet wird.
---

# Ereignisse

Alles, was geschieht, während ein Job läuft — die Schritte eines Durchlaufs, die
Nachrichten eines Monitors, die Zahlen eines Bursts, die Frames des Inspektors,
das Ende eines Jobs —, wird als Ereignis gesendet. In einem Browser erhält die
Seite des Servers sie auf einem WebSocket, `/api/events`; ein Skript kann auf
demselben Socket lauschen. Die Desktop-App erhält dieselben Ereignisse, mit
denselben Namen und Nutzdaten, innerhalb der App.

## Abonnieren {#subscribe}

Öffnen Sie einen WebSocket zu `/api/events` auf dem Server:

```bash
websocat -H "Authorization: Bearer $TOKEN" ws://127.0.0.1:1430/api/events
```

- **Authentifizierung** ist die des übrigen API: das Token als
  `Authorization: Bearer` oder das Sitzungs-Cookie eines Browsers. Ohne sie
  wird das Upgrade mit `401` `auth.required` abgelehnt.
- **Origin**: Ein Client, der einen `Origin`-Header sendet, muss den eigenen des
  Servers senden (Host und Port gleich `Host`), sonst wird das Upgrade mit `403`
  `auth.origin` abgelehnt. Die meisten WebSocket-Bibliotheken außerhalb eines
  Browsers senden keinen.
- **Jedes Ereignis an jeden Client.** Es gibt nichts zu abonnieren: Jeder Socket
  erhält jedes Ereignis jedes Jobs, wer ihn auch gestartet hat. Wählen Sie aus,
  was Sie brauchen, nach `event` und nach `job_id` in den Nutzdaten.
- **Nur lauschen.** Der Server ignoriert, was ein Client sendet, außer einem
  Close; eine Nachricht über 64 KiB schließt den Socket.
- **Keep-alive.** Der Server pingt alle 20 s, damit ein stiller Socket durch
  Proxys offen bleibt. Wenn der Server stoppt, schließt er jeden Socket.
- **Nichts wird wiederholt.** Ereignisse, die gesendet wurden, während ein
  Client nicht verbunden war, sind für ihn verloren. Ein Client, der sich neu
  verbindet, sollte den aktuellen Zustand mit Befehlen lesen (`jobs_list`,
  `inspect_snapshot`, `emulator_exchanges`…).
- **Zurückfallen.** Bis zu 4096 Ereignisse warten auf einen Socket. Ein Client,
  der weiter zurückfällt, erhält
  [`server://lagged`](#event-server-lagged) mit der Zahl, die er verpasste.

## Nachrichtenformat {#format}

Jedes Ereignis ist eine Textnachricht mit einem JSON-Objekt:

```json
{ "event": "scan://open", "payload": { "job_id": 9, "ts": 1759600000123, "port": 8080, "banner": null } }
```

| Feld | Was es ist |
| --- | --- |
| `event` | Der Kanal, unten |
| `payload` | Die Werte des Ereignisses; ihre Gestalt hängt vom Kanal ab |

Zeiten (`ts`, `first_ms` und `last_ms` einer Gegenstelle) sind Millisekunden
seit 1970; Latenzen und andere Dauern (`*_latency_ms`, `p50_ms`…, `ms`) sind
Millisekunden. Fehler in Nutzdaten sind [`EngineError`](index.md#errors)-Objekte;
ihre Codes stehen in [Fehlermeldungen](../reference/errors.md).

## Kanäle {#channels}

| Kanal | Gesendet von | Wann |
| --- | --- | --- |
| [`experiment://step`](#event-experiment-step) | Ein Durchlauf | Ein Schritt startet, besteht, schlägt fehl, versucht erneut, wiederholt oder meldet Last |
| [`experiment://ended`](#event-experiment-ended) | Ein Durchlauf | Einmal, wenn der Durchlauf von selbst endet |
| [`job://ended`](#event-job-ended) | Jeder Job | Einmal, wenn der Job von selbst endet oder fehlschlägt |
| [`osc://message`](#event-osc-message) | OSC-Monitor | Jedes Paket |
| [`osc://gen-tick`](#event-osc-gen-tick) | OSC-Generator | Jede Nachricht oder 30- bis 45-mal pro Sekunde über 60 Nachrichten pro Sekunde |
| [`http://burst-progress`](#event-http-burst-progress) | HTTP-Burst | Alle 100 ms und am Ende |
| [`ws://state`](#event-ws-state) | WebSocket-Verbindung | Verbunden, geschlossen |
| [`ws://messages`](#event-ws-messages) | WebSocket-Verbindung | Alle 100 ms mit etwas Neuem |
| [`mqtt://state`](#event-mqtt-state) | MQTT-Verbindung | Verbunden, abonniert, geschlossen |
| [`mqtt://messages`](#event-mqtt-messages) | MQTT-Verbindung | Alle 100 ms mit etwas Neuem |
| [`mqtt://ack`](#event-mqtt-ack) | MQTT-Verbindung | Eine QoS-1/2-Veröffentlichung ist abgeschlossen; ein Unsubscribe wurde beantwortet |
| [`broadcast://emit-stat`](#event-broadcast-emit-stat) | Beacon | Alle 250 ms und am Ende |
| [`broadcast://peers`](#event-broadcast-peers) | Discovery-Listener | Alle 400 ms |
| [`netsim://stat`](#event-netsim-stat) | Störungs-Relais | Alle 250 ms |
| [`storm://stat`](#event-storm-stat) | Sturm | Alle 250 ms und am Ende |
| [`scan://open`](#event-scan-open) | Scanner | Jeder offene Port |
| [`scan://progress`](#event-scan-progress) | Scanner | Etwa alle 1 % des Bereichs und am Ende |
| [`emulator://activity`](#event-emulator-activity) | Emulator-Job | Alle 200 ms mit etwas Neuem |
| [`inspect://batch`](#event-inspect-batch) | Inspektor | Alle 120 ms mit neuen Frames, etwa einmal pro Sekunde bei Stille, solange der Mitschnitt läuft |
| [`server://lagged`](#event-server-lagged) | Der Server | Ein Client fiel zurück |

### `experiment://step` {#event-experiment-step}

Ein Schritt eines Durchlaufs: ein Knoten, der startet, besteht, fehlschlägt,
auf einen neuen Versuch wartet, sich wiederholt oder den Fortschritt einer Last
meldet. Ein mit `/api/run` gestarteter Durchlauf sendet dieselben Schritte in
seiner Antwort (siehe [Durchläufe](run.md)).

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job des Durchlaufs |
| `ts` | number | Wann |
| `node_id` | string | Der Knoten |
| `state` | string | `running`, `passed`, `failed`, `retry` (ein Versuch schlug fehl und der Schritt läuft nach einer Pause erneut), `repeating` (der Fortschritt einer wiederholten Aktion, höchstens einmal pro Sekunde) oder `load` (der Fortschritt einer Last, höchstens einmal pro Sekunde) |
| `detail` | string | Was geschah, auf Englisch; leer bei `running` und `failed` (siehe `error`) |
| `message_key` | string or null | Der Text der Oberfläche dafür, als Schlüssel ihres Wörterbuchs |
| `message_params` | object or null | Die Werte, die `message_key` nennt |
| `vars` | object | Variablen, die der Schritt schrieb; weggelassen, wenn keine |
| `error` | `EngineError` | Warum er fehlschlug oder warum der Versuch fehlschlug (`retry`); sonst weggelassen |
| `frame` | number | Der Inspektor-Frame der Nachricht, die ein Warten (oder die erwartete Antwort eines Sendens) traf, als der Mitschnitt lief; sonst weggelassen |
| `load` | object | Was eine Last gemessen hat, ihre Schwellenwerte gelesen — beim letzten Ereignis eines Lastschritts, bestanden oder fehlgeschlagen; sonst weggelassen. Siehe [Last](../experiments/load.md) |

Ein Knoten [[ui:exp.node.end]] eines Durchlaufs zeigt `running`, wenn der erste
Zweig ihn erreicht, und `passed`, sobald jeder Zweig ohne Fehler fertig ist.
Geheime Werte sind in jedem Feld maskiert.

### `experiment://ended` {#event-experiment-ended}

Ein Durchlauf endete von selbst: Er bestand, schlug fehl oder lief aus der
Zeit. Wird direkt nach denselben Nutzdaten auf
[`job://ended`](#event-job-ended) gesendet. Ein mit `job_stop` oder
[[ui:app.stopAll]] gestoppter Durchlauf sendet keines von beiden und speichert
keinen Bericht.

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job des Durchlaufs |
| `kind` | string | `experiment` |
| `seed` | number | Der Startwert, mit dem er lief |
| `profile` | string or null | Sein Profil |
| `overridden` | boolean | Einige Parameterwerte kamen von [[ui:exp.runWith]] oder `overrides` |
| `error` | `EngineError` or null | Der erste Fehler des Durchlaufs; null, wenn er bestand |
| `report_path` | string or null | Sein Bericht, in `runs/` des Datenordners |
| `report_error` | `EngineError` or null | Warum der Bericht nicht geschrieben werden konnte |

### `job://ended` {#event-job-ended}

Ein Job endete von selbst oder schlug fehl. Ein mit `job_stop` oder
`jobs_stop_all` gestoppter Job sendet es nicht.

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job |
| `kind` | string | `osc-monitor`, `osc-gen`, `http-burst`, `netsim`, `storm`, `scan`, `beacon`, `discovery`, `mqtt`, `websocket`, `emulator` oder `experiment` |
| `error` | `EngineError` or null | Warum er endete, wenn etwas schiefging |

Ein `job://ended` eines Durchlaufs trägt auch die Felder von
[`experiment://ended`](#event-experiment-ended). Was jede Art beendet:

| `kind` | Endet, wenn | `error` |
| --- | --- | --- |
| `osc-monitor` | Der Socket nicht mehr empfangen kann | `wait.receive_failed` |
| `osc-gen` | Seine Dauer vorbei ist oder ein Senden fehlschlägt | null oder `transport.*` |
| `http-burst` | Seine Gesamtzahl oder Dauer erreicht ist | null |
| `storm` | Seine Dauer vorbei ist | null |
| `scan` | Jeder Port des Bereichs versucht wurde | null |
| `beacon` | Seine Runden oder seine Dauer vorbei sind oder mehr als 32 Sendungen fehlschlugen, ohne dass eine gesendet wurde | null oder `transport.*` |
| `discovery` | Der Socket nicht mehr empfangen kann | `wait.receive_failed` |
| `mqtt` | Der Broker die Verbindung schloss oder sie verlorenging | `transport.*` (`transport.reset`, wenn der Broker sie schloss) oder `mqtt.protocol` |
| `websocket` | Die Verbindung sich schloss | null oder warum sie verlorenging |
| `netsim` | Das Relais nicht mehr arbeiten kann | warum |
| `emulator` | Sein Socket fehlschlägt | warum |
| `experiment` | Der Durchlauf endet | der Fehler des Durchlaufs oder null |

### `osc://message` {#event-osc-message}

Ein von einem OSC-Monitor empfangenes, dekodiertes UDP-Paket. Wird für jedes
Paket gesendet, ohne Bündelung.

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job des Monitors |
| `ts` | number | Wann es eintraf |
| `from` | string | Der Absender, `IP:port` |
| `bytes` | number | Die Größe des Pakets |
| `messages` | object[] | Jede Nachricht des Pakets (ein Bundle hat mehrere): `address` und `args` ([`OscArg`](commands.md#type-oscarg)`[]`) |
| `error` | `EngineError` or null | `osc.packet_malformed`, wenn das Paket nicht dekodierte (dann ist `messages` leer) |

### `osc://gen-tick` {#event-osc-gen-tick}

Der Fortschritt eines OSC-Generators: für jede Nachricht unter 60 Nachrichten
pro Sekunde; darüber für jede n-te, wobei n die Rate geteilt durch 30 und
abgerundet ist — 30 bis 45 Mal pro Sekunde.

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job des Generators |
| `ts` | number | Wann |
| `value` | number | Der gerade gesendete Wert, bevor er auf eine Ganzzahl oder einen 32-Bit-Float gerundet wird |
| `sent` | number | Bisher gesendete Nachrichten |

### `http://burst-progress` {#event-http-burst-progress}

Die Zahlen eines HTTP-Bursts, alle 100 ms, während er läuft, und einmal mehr mit
`done: true`, wenn er von selbst endet.

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job des Bursts |
| `ts` | number | Wann |
| `sent` | number | Bisher beantwortete oder fehlgeschlagene Anfragen |
| `ok` | number | Davon mit einem 2xx-Status beantwortet |
| `failed` | number | Davon jeder andere Status oder keine Antwort |
| `missed` | number | Anfragen eines getakteten Bursts, die zu lange auf einen freien Worker warteten und übersprungen wurden |
| `rps` | number | Anfragen pro Sekunde über die letzten 100 ms; im letzten Ereignis über den ganzen Burst |
| `last_latency_ms`, `min_latency_ms`, `max_latency_ms`, `avg_latency_ms` | number | Bisherige Latenzen |
| `p50_ms`, `p90_ms`, `p95_ms`, `p99_ms` | number | Perzentile jeder bisherigen Anfrage, Fehler eingeschlossen, innerhalb von 0,5 % |
| `done` | boolean | Das letzte Ereignis des Bursts |

### `ws://state` {#event-ws-state}

Eine von `ws_connect` geöffnete WebSocket-Verbindung hat sich verbunden oder
geschlossen. Eine Verbindung, deren Job gestoppt wurde, sendet kein `closed`.

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job der Verbindung |
| `ts` | number | Wann |
| `state` | string | `connected` oder `closed` |
| `handshake` | object | `url`, `peer`, `local`, `protocol` (das Subprotokoll, das der Server wählte, oder null) und `ms` (das Verbinden und das Upgrade) |
| `closed` | object or null | Bei `closed`: `code`, `reason`, `by` (`client`, `server` oder `lost`) und `error` |

### `ws://messages` {#event-ws-messages}

Was eine WebSocket-Verbindung seit dem letzten Ereignis gesendet und empfangen
hat, alle 100 ms, wenn es etwas gibt.

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job der Verbindung |
| `ts` | number | Wann |
| `messages` | object[] | In Reihenfolge: `ts`, `dir` (`rx` empfangen, `tx` gesendet), `kind` (`text` oder `binary`), `text` (die ersten 64 KiB als UTF-8, bei einer Binärnachricht ebenso; Bytes, die das nicht sind, werden `�`), `hex` (die ersten 4096 Bytes einer Binärnachricht als Hex, sonst null), `bytes` (die volle Größe) und `truncated` (mehr, als gezeigt wurde: über 64 KiB Text, über 4096 Bytes Binär) |
| `dropped` | number | Nachrichten, die aus diesem Ereignis weggelassen wurden, weil es mehr als 2000 waren; die ältesten zuerst |

### `mqtt://state` {#event-mqtt-state}

Der Zustand einer MQTT-Verbindung hat sich geändert.

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job der Verbindung |
| `ts` | number | Wann |
| `state` | string | `connected`; `subscribed` nach jeder Antwort auf ein Abonnement; `closed`, wenn die Verbindung endete (nicht, wenn ihr Job gestoppt wurde) |
| `broker` | string | `host:port` |
| `error` | `EngineError` or null | Warum eine `closed`-Verbindung endete (`transport.reset`, wenn der Broker sie schloss); sonst null |
| `grants` | object[] | Bei `subscribed`: jeder angeforderte Filter mit `filter`, `qos` (gewährt) und `accepted`; sonst leer |

### `mqtt://messages` {#event-mqtt-messages}

Was eine MQTT-Verbindung seit dem letzten Ereignis empfangen hat, alle 100 ms,
wenn es etwas gibt. Eine erneut zugestellte QoS-2-Nachricht wird einmal gezeigt.

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job der Verbindung |
| `ts` | number | Wann |
| `messages` | object[] | `ts`, `topic`, `payload` (als UTF-8; Bytes, die das nicht sind, werden `�`), `bytes`, `qos`, `retain`, `dup` |
| `dropped` | number | Nachrichten, die weggelassen wurden, weil mehr als 4000 in 100 ms eintrafen; die ältesten zuerst |

### `mqtt://ack` {#event-mqtt-ack}

Der Broker hat etwas abgeschlossen, das die Verbindung angefordert hatte.

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job der Verbindung |
| `ts` | number | Wann |
| `kind` | string | `published` (eine QoS-1- oder -2-Veröffentlichung ist vollständig) oder `unsubscribed` |
| `packet_id` | number | Die MQTT-Paket-ID |
| `topic` | string or null | Das veröffentlichte Thema; null bei `unsubscribed` |

### `broadcast://emit-stat` {#event-broadcast-emit-stat}

Die Zähler eines Beacons, alle 250 ms, und einmal mehr, wenn er von selbst mit
`pps` 0 endet.

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job des Beacons |
| `ts` | number | Wann |
| `targets` | number | Ziele in jeder Runde |
| `rounds` | number | Gesendete Runden |
| `packets`, `bytes` | number | Gesendete Datagramme und Bytes |
| `errors` | number | Fehlgeschlagene Sendungen |
| `pps` | number | Datagramme pro Sekunde über die letzten 250 ms |

### `broadcast://peers` {#event-broadcast-peers}

Was ein Discovery-Listener gehört hat, alle 400 ms.

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job des Listeners |
| `ts` | number | Wann |
| `peers` | object[] | Zuletzt Gehörtes zuerst: `addr`, `proto`, `packets`, `bytes`, `first_ms`, `last_ms`, `last_summary`, `responded` (seine Pakete, die beantwortet wurden, gezählt beim Eintreffen); höchstens 512 |
| `packets`, `bytes` | number | Alles Empfangene |
| `responses` | number | Gesendete Antworten |

### `netsim://stat` {#event-netsim-stat}

Die Zähler eines Störungs-Relais, alle 250 ms. Relais der Knoten
[[ui:exp.node.impairment]] eines Durchlaufs melden stattdessen im Bericht des
Durchlaufs.

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job des Relais |
| `ts` | number | Wann |
| `received`, `forwarded` | number | Datagramme oder Blöcke hinein und hinaus |
| `dropped` | number | Verloren durch `loss`, Bursts oder `offline` (UDP; ein TCP-Relais hält einen Datenstrom, während es offline ist, und verwirft nichts) |
| `throttled` | number | UDP: durch die Bandbreitengrenze verworfen oder weil zu viele schon unterwegs waren. TCP: Blöcke, die ihren Datenstrom wegen der Bandbreitengrenze zurückhielten |
| `duplicated`, `corrupted`, `reordered` | number | Was das Profil mit ihnen tat |
| `bytes` | number | Weitergegebene Bytes |
| `connections`, `reset`, `stalled` | number | TCP: angenommene, zurückgesetzte, halboffen gelassene Verbindungen; weggelassen, solange 0 |
| `profile` | string | Das Profil, mit dem es jetzt stört, wie die Zeitleiste es nennt: sein Name oder was es tut (`60 ms ±25 · loss 2%`) |

### `storm://stat` {#event-storm-stat}

Die Zähler eines Sturms, alle 250 ms, und einmal mehr, wenn er von selbst mit
`pps` und `mbps` 0 endet.

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job des Sturms |
| `ts` | number | Wann |
| `packets`, `bytes` | number | Gesendete Datagramme (oder TCP-Verbindungen) und Bytes |
| `errors` | number | Fehlgeschlagene Sendungen oder Verbindungen |
| `pps` | number | Pro Sekunde über die letzten 250 ms |
| `mbps` | number | Megabit pro Sekunde über die letzten 250 ms |

### `scan://open` {#event-scan-open}

Der Scanner hat einen offenen Port gefunden.

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job des Scans |
| `ts` | number | Wann |
| `port` | number | Der Port |
| `banner` | string or null | Was der Dienst zuerst sendete, wenn Banner angefordert wurden und er innerhalb von 400 ms etwas sagte |

### `scan://progress` {#event-scan-progress}

Wie weit ein Scan ist: etwa alle 1 % des Bereichs und wenn er von selbst mit
`done` gleich `total` endet (dieses kann zweimal eintreffen).

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job des Scans |
| `ts` | number | Wann |
| `done` | number | Versuchte Ports |
| `total` | number | Ports im Bereich |
| `open` | number | Gefundene offene Ports |

### `emulator://activity` {#event-emulator-activity}

Was ein mit `emulator_start` gestarteter Emulator seit dem letzten Ereignis
empfangen und beantwortet hat, alle 200 ms, wenn sich etwas geändert hat (ein
Austausch, das Herunter- oder Hochfahren oder eine Nachricht, die der
MQTT-Broker nicht zustellen konnte). Die Knoten [[ui:exp.node.emulator]] eines
Durchlaufs senden es nicht; ihre Zähler stehen im Bericht des Durchlaufs.

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job des Emulators |
| `ts` | number | Wann |
| `counts` | object | `total`, `unmatched`, `failed`, `down`, `hits` (pro Regel) und `missed` (MQTT; weggelassen, solange 0) — wie [`emulator_exchanges`](commands.md#emulator_exchanges) |
| `forced` | string | `unavailable`, `reset` oder `timeout`, während er heruntergefahren ist; sonst weggelassen |
| `exchanges` | object[] | Die neuen Austausche, wie `emulator_exchanges` sie auflistet, aber ohne `data`; höchstens 200 |
| `dropped` | number | Austausche nach den ersten 200 des Intervalls, die hier nicht gesendet werden; `emulator_exchanges` hat weiterhin die letzten 500 |

### `inspect://batch` {#event-inspect-batch}

Neue Inspektor-Frames. Wird nur gesendet, solange der Mitschnitt läuft: alle
120 ms, wenn es neue Frames gibt, und etwa einmal pro Sekunde, wenn es keine
gibt, damit die Zähler aktuell bleiben.

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `frames` | object[] | Die neuen [Frames](commands.md#type-frame), älteste zuerst, höchstens 250; ohne ihre Bytes (verwenden Sie `inspect_payload`) |
| `stats` | object | Die Zähler des Mitschnitts, [`CaptureStats`](commands.md#type-frame) |
| `skipped_now` | number | Frames, die seit dem letzten Batch aufgezeichnet, aber nicht in diesem sind — mehr als 250 trafen ein oder der Puffer ließ sie gehen. Sie sind weiterhin in einem Export, solange der Puffer sie hält |

### `server://lagged` {#event-server-lagged}

Nur Server. Dieser Client fiel mehr als 4096 Ereignisse zurück und verpasste
einige. Lesen Sie den Zustand mit Befehlen erneut.

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `skipped` | number | Wie viele Ereignisse er verpasste |

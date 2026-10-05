---
title: Knoten
description: Jede Art von Knoten eines Experiments — was er tut, seine Felder mit Standardwerten und Grenzen, seine Ausgänge, die Einstellungen, die er annimmt, und wie er in einer Experimentdatei aussieht.
---

# Referenz der Knoten

Jede Art von Knoten, die ein Experiment enthalten kann, in den Gruppen des
Hinzufügen-Menüs: [Aktionen](#actions), [Warteknoten](#waits),
[Emulation](#emulation), [Störungen](#faults), [Daten](#data),
[Prüfungen](#checks) und [Ablauf](#flow). Wie man sie hinzufügt und verdrahtet,
steht in [Der Editor](index.md); `signallab nodes` gibt denselben Katalog als
JSON aus, für Skripte und Assistenten ([Die Kommandozeile](../automation/cli.md)).

## Diese Seite lesen {#reading}

Jeder Knoten hat eine Tabelle seiner Felder:

- **Feld** ist der Name im Bereich der Eigenschaften; **In der Datei** ist der
  Schlüssel im JSON des Experiments.
- **Standard** ist, was ein Knoten erhält, wenn Sie ihn im Editor hinzufügen. Wo
  eine Datei einen Schlüssel weglassen darf, ist der Wert, den er dann annimmt,
  als *falls nicht vorhanden* angegeben; andere Schlüssel sind in einer Datei
  erforderlich.
- **Vorlagen**: *ja* — das Feld nimmt `{{templates}}`: Parameter, früher
  gesetzte Variablen, Geheimnisse und Generatoren, aufgelöst, während der Schritt
  läuft ([Daten und Vorlagen](data.md)). *Nur Parameter* — es wird vor dem ersten
  Schritt geöffnet, wenn nur Parameter bekannt sind. *Nein* — der Wert wird so
  genommen, wie er geschrieben ist.

Zeiten sind in Millisekunden. Grenzen werden vor dem Start eines Durchlaufs
geprüft; ein Feld außerhalb des Bereichs hält das Experiment vom Laufen ab und
wird am Knoten angezeigt.

## Ein Knoten in einer Datei {#file-shape}

In einer Experimentdatei ist ein Knoten ein Objekt mit einer `id` (eindeutig im
Experiment), seinem `type`, seinem Platz auf der Arbeitsfläche (`x`, `y`, null
oder mehr), seinen Feldern und den Einstellungen, die er verwendet (`retry`,
`repeat`, `load`, weggelassen, wenn aus). Eine Verbindung ist eine Kante von einem
Ausgang eines Knotens (`port`, `next` falls nicht vorhanden) zu einem anderen
Knoten:

```json
{
  "nodes": [
    { "id": "start", "type": "start", "x": 40, "y": 80 },
    { "id": "ping", "type": "udp", "x": 270, "y": 80, "target": "127.0.0.1:9000", "text": "PING",
      "retry": { "attempts": 3, "delay_ms": 500, "backoff": "fixed" } },
    { "id": "end", "type": "end", "x": 500, "y": 80 }
  ],
  "edges": [
    { "from": "start", "to": "ping", "port": "next" },
    { "from": "ping", "to": "end", "port": "next" }
  ]
}
```

Die Beispiele unten zeigen je einen Knoten, wie eine Datei ihn hält.

## Von vielen Knoten geteilte Einstellungen {#settings}

Diese werden im unteren Teil der Eigenschaften eines Knotens eingeschaltet.
Welcher Knoten welche annimmt, steht unter jedem Knoten.

| Einstellung | Nimmt sie | Was sie tut |
| --- | --- | --- |
| [Neuversuch](#retry) | Knoten, die senden oder lauschen: [[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]], [[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_connect]], [[ui:exp.node.ws_send]], und jeden Warteknoten | Versucht es erneut, wenn der Schritt fehlschlägt |
| [Wiederholung](#repeat) | Knoten, die senden: [[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]], [[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_send]] | Sendet immer wieder, eine Anzahl von Malen oder eine Zeit lang |
| [Last](#load) | [[ui:exp.node.http]] | Sendet die Anfrage nach einem Lastprofil, gemessen und an Schwellenwerten beurteilt |
| [Auf eine Antwort warten](#reply) | [[ui:exp.node.osc]], [[ui:exp.node.udp]] | Sendet und wartet im selben Schritt auf die Antwort |

### Neuversuch {#retry}

[[ui:exp.retryOn]]: Wenn der Schritt fehlschlägt — keine Verbindung, eine
Zeitüberschreitung, ein Warten ohne Treffer —, pausiert er und läuft erneut.
Jeder fehlgeschlagene Versuch ist eine Zeile in der Zeitleiste; der Schritt
schlägt fehl, wenn der letzte Versuch es tut. Eine Vorlage, die sich nicht
auflösen lässt, wird nicht erneut versucht. [[ui:common.stop]] beendet auch eine
Pause.

| Feld | In der Datei | Was | Standard und Grenzen |
| --- | --- | --- | --- |
| [[ui:exp.attempts]] | `retry.attempts` | Versuche insgesamt, der erste eingeschlossen | 3; 2–10 im Editor (eine Datei darf auch 1 sagen) |
| [[ui:exp.retryDelay]] | `retry.delay_ms` | Die Pause vor dem zweiten Versuch | 500; 0–60 000 |
| [[ui:exp.backoff]] | `retry.backoff` | [[ui:exp.backoff.fixed]] (`fixed`): jede Pause gleich; [[ui:exp.backoff.exponential]] (`exponential`): nach jedem Fehlschlag doppelt so lang | `fixed` (auch falls nicht vorhanden) |

Keine Pause ist länger als 60 Sekunden, wie sehr sie sich auch verdoppelt. Ein
Warten, dessen Ausgang [[ui:exp.portTimeout]] verdrahtet ist, schlägt bei einer
Zeitüberschreitung nicht fehl — es geht durch diesen Ausgang —, wird dann also
nicht erneut versucht.

### Wiederholung {#repeat}

[[ui:exp.repeatOn]]: Der Knoten sendet immer wieder — ein Herzschlag, eine
Abfrage, ein gleichmäßiger Strom —, ohne eine Schleife im Graphen. Jede Sendung
liest ihre Vorlagen neu (`{{counter}}` ist ihre Nummer, `{{now}}` ihre Zeit), und
Neuversuch, wenn an, gilt für jede Sendung. Der Schritt besteht, wenn jede Sendung
es tat; eine Sendung, die endgültig fehlschlägt, lässt den Schritt fehlschlagen.
Die Zeitleiste berichtet den Fortschritt höchstens einmal pro Sekunde.

| Feld | In der Datei | Was | Standard und Grenzen |
| --- | --- | --- | --- |
| [[ui:exp.repeatBy]] | `repeat.until` | [[ui:exp.repeatBy.count]] (`count`) oder [[ui:exp.repeatBy.duration]] (`duration`) | `count` (auch falls nicht vorhanden) |
| [[ui:exp.repeatCount]] | `repeat.count` | Sendungen insgesamt, die erste eingeschlossen | 10 (auch falls nicht vorhanden); 2–10 000 |
| [[ui:exp.repeatDuration]] | `repeat.duration_ms` | Wie lange weiter gesendet wird, ab der ersten Sendung | 10 000 (auch falls nicht vorhanden); 1–300 000 |
| [[ui:exp.repeatInterval]] | `repeat.interval_ms` | Die Pause zwischen zwei Sendungen | 1 000; 10–60 000; in einer Datei erforderlich |
| [[ui:exp.repeatJitter]] | `repeat.jitter_ms` | Jede Pause bis zu so viel länger, gezogen aus dem Startwert des Durchlaufs | 0 (auch falls nicht vorhanden); 0–60 000 |

Die Wiederholungen müssen in die 300 Sekunden eines Durchlaufs passen, und *eine
Zeit lang* muss weniger als 10 000 Sendungen brauchen (ihre Zeit geteilt durch das
Intervall).

### Last {#load}

[[ui:exp.loadOn]], nur an einer [[ui:exp.node.http]]: Die Anfrage wird nach einem
Profil gesendet — eine konstante Rate, eine Rampe, Stufen, eine Spitze oder
zufällige Ankünfte —, mit bis zu 512 gleichzeitig unterwegs (standardmäßig 32),
und gemessen: Latenzen, Fehler, die erreichte Rate. Schwellenwerte entscheiden,
ob der Schritt besteht. Last ersetzt Wiederholung und Neuversuch (eine
fehlgeschlagene Anfrage wird gezählt, nicht erneut versucht) und hinterlässt keine
Antwort für die Prüfungen danach. Ihre Felder und Ergebnisse stehen in
[Lasttests](load.md).

### Auf eine Antwort warten {#reply}

[[ui:exp.expectReply]], an einer [[ui:exp.node.osc]] oder einer
[[ui:exp.node.udp]]: Die Nachricht wird von dem Port gesendet, auf dem die Antwort
erwartet wird, sodass ein Gerät, das dem Absender antwortet, gehört wird, und der
Schritt besteht nur, wenn rechtzeitig eine passende Antwort eintrifft. Keine
Antwort lässt den Schritt fehlschlagen — Neuversuch sendet erneut. Die Antwort
wird in einer Variablen gespeichert, wie bei einem Warteknoten.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.replyOn]] | `reply.bind` | `IP:port`, von dem gesendet und auf dem gelauscht wird; Port 0 nimmt einen beliebigen freien Port | `0.0.0.0:0` | Nein |
| [[ui:exp.replyAddress]] (OSC) | `reply.address` | Das Adressmuster der Antwort, wie in [[[ui:exp.node.wait_osc]]](#node-wait_osc) | `/*` | Ja |
| [[ui:exp.argRules]] (OSC) | `reply.args` | Argumentregeln, wie in [[ui:exp.node.wait_osc]] | keine; höchstens 16 | Werte: ja |
| [[ui:exp.replyMode]] (UDP) | `reply.mode` | `any`, `contains`, `regex` oder `hex` — siehe [Nutzdaten abgleichen](#payload-matching) | `any` (auch falls nicht vorhanden) | Nein |
| [[ui:field.pattern]] (UDP) | `reply.pattern` | Was die Antwort enthalten oder worauf sie passen muss | leer; erforderlich außer bei `any` | Ja |
| [[ui:exp.waitTimeout]] | `reply.timeout_ms` | Wie lange gewartet wird | 2 000 (auch falls nicht vorhanden); 1–120 000 | Nein |
| [[ui:exp.replyVariable]] | `reply.variable` | Die Variable, in die die Antwort geschrieben wird | `reply` (auch falls nicht vorhanden) | Nein |

Der Port der Antwort wird vor dem ersten Schritt geöffnet, wie der eines
Warteknotens.

## Aktionen {#actions}

Knoten, die senden. Ein Warten nach einer Aktion zählt Nachrichten ab dem Moment,
in dem die Aktion begann.

### HTTP-Anfrage {#node-http}

Sendet eine HTTP-Anfrage und behält die Antwort für die Prüfungen, Zweige und
Knoten [[ui:exp.node.extract]] danach.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.method]] | `request.method` | GET, HEAD, POST, PUT, PATCH, DELETE oder OPTIONS (eine Datei darf jede Methode nennen) | `GET` | Nein |
| URL | `request.url` | Eine `http://`- oder `https://`-URL | `http://127.0.0.1:8080/` | Ja |
| [[ui:common.timeoutMs]] | `request.timeout_ms` | Für den ganzen Austausch | 4 000 (10 000 falls nicht vorhanden); 1–120 000 | Nein |
| [[ui:exp.headers]] | `request.headers` | `[[name, value], …]`; eine Zeile mit leerem Namen wird übersprungen | keine | Ja, Namen und Werte |
| [[ui:exp.body]] | `request.body` | Text, oder `null` für keinen | `null` | Ja |
| [[ui:field.auth]] | `request.auth` | [[ui:http.auth.none]], [[ui:http.auth.basic]], [[ui:http.auth.bearer]] oder [[ui:http.auth.digest]], mit [[ui:field.username]] und [[ui:field.password]], oder [[ui:field.token]] | keine | Ja |

- Jede Antwort lässt den Schritt bestehen, 404 und 500 eingeschlossen: Prüfen Sie
  den Status mit [[[ui:exp.node.assert_status]]](#node-assert_status) oder
  verzweigen Sie mit [[[ui:exp.node.branch_status]]](#node-branch_status) darauf.
  Eine Anfrage, die keine Antwort erhält — abgelehnt, eine Zeitüberschreitung,
  ein Name, der sich nicht auflöst, ein Zertifikat, dem nicht vertraut wird —,
  lässt den Schritt fehlschlagen.
- Weiterleitungen werden verfolgt, höchstens zehn. `https://`-Zertifikate werden
  geprüft.
- Der Antwort-Body wird bis zu 256 KiB für die Prüfungen behalten; ein größerer
  Body wird dort abgeschnitten (die Prüfungen sagen es, wenn das Gesuchte hinter
  dem Schnitt liegen könnte).
- Digest beantwortet die 401-Challenge des Servers und sendet die Anfrage erneut.
  Die Anmeldedaten gehen nur in die Anfrage: Schritte, Berichte und der Inspektor
  zeigen nie den `Authorization`-Header. Schreiben Sie ein Passwort als
  `{{secret.NAME}}`.
- Solange das Experiment Cookies behält (standardmäßig an, unter
  [[ui:exp.params]]), wird, was Server setzen, mit den späteren Anfragen des
  Durchlaufs an sie zurückgesendet.

Ausgänge: [[ui:exp.outputPort]]. Einstellungen: Neuversuch, Wiederholung, Last.

```json
{ "id": "cue", "type": "http", "x": 270, "y": 80,
  "request": { "method": "POST", "url": "{{api}}/cue", "headers": [["Content-Type", "application/json"]],
               "body": "{\"cue\": 1}", "timeout_ms": 5000,
               "auth": { "scheme": "bearer", "token": "{{secret.API_TOKEN}}" } } }
```

Siehe auch [HTTP](../protocols/http.md).

### TCP-Nachricht {#node-tcp}

Verbindet sich über TCP mit einem Host, schreibt die Nutzdaten, wartet bis zu
250 ms auf die ersten Bytes einer Antwort (es liest höchstens 1 024 Bytes, einmal)
und schließt die Verbindung. Die Größe der Antwort wird berichtet, nicht geprüft.

Im [Inspektor](../tools/inspector.md) ist der Schritt zwei `tcp`-Frames mit der
Quelle `experiment`: die geschriebenen Nutzdaten und, wenn eine kam, die gelesene
Antwort. Verwendete Geheimnisse werden in beiden maskiert, wie in jedem Frame.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.host]] | `host` | Ein Hostname oder eine IP-Adresse | `127.0.0.1` | Ja |
| [[ui:exp.port]] | `port` | | 9000; 1–65 535 | Nein |
| [[ui:common.timeoutMs]] | `timeout_ms` | Für Verbinden, Schreiben und die Antwort zusammen | 4 000 (auch falls nicht vorhanden); 1–120 000 | Nein |
| [[ui:exp.payload]] | `payload` | Der Text, der nach dem Verbinden geschrieben wird, als UTF-8 | `hello` | Ja |

Der Schritt schlägt fehl, wenn die Verbindung abgelehnt wird, der Name sich nicht
auflöst oder die Zeit abläuft. Ausgänge: [[ui:exp.outputPort]]. Einstellungen:
Neuversuch, Wiederholung. [[ui:exp.sendNow]] verbindet sich und schreibt die
Nutzdaten einmal, und das Ergebnis des Knotens sagt, wie viele Bytes gesendet
wurden und zurückkamen.

```json
{ "id": "go", "type": "tcp", "x": 270, "y": 80, "host": "127.0.0.1", "port": 5000, "payload": "GO\r\n", "timeout_ms": 2000 }
```

### OSC-Nachricht {#node-osc}

Sendet eine OSC-1.0-Nachricht über UDP.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:common.target]] | `target` | `IP:port` oder `host:port`; ein Hostname wird beim Senden des Schritts aufgelöst, seine IPv4-Adresse genommen, wenn er eine hat | `127.0.0.1:9000` | Ja |
| [[ui:common.address]] | `address` | Beginnt mit `/` | `/test` | Ja |
| [[ui:exp.arguments]] | `args` | `[{ "type", "value" }, …]` — `int`, `float`, `str`, `long`, `double`, `bool`, `blob` (Bytes), `nil` (kein Wert) | keine | Textwerte (`str`): ja |
| [[ui:exp.expectReply]] | `reply` | Optional: senden und auf die Antwort warten — siehe [Auf eine Antwort warten](#reply) | aus | |

Ausgänge: [[ui:exp.outputPort]]; mit erwarteter Antwort wird er nur gefolgt, wenn
die Antwort kam. Einstellungen: Neuversuch, Wiederholung, eine Antwort. ⚡
[[ui:exp.routeThrough]] in seinen Eigenschaften stellt ihm eine
[[[ui:exp.node.impairment]]](#node-impairment) voran.

```json
{ "id": "fader", "type": "osc", "x": 270, "y": 80, "target": "{{device}}", "address": "/fader/1",
  "args": [{ "type": "float", "value": 0.75 }] }
```

Siehe auch [OSC](../protocols/osc.md).

### UDP-Datagramm {#node-udp}

Sendet eine Textnutzlast als ein UDP-Datagramm an ein oder mehrere Ziele.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:common.target]] | `target` | `IP:port` oder `host:port`; mehrere durch Kommas, Semikolons oder Zeilenumbrüche getrennte erhalten je das Datagramm. Ein Hostname wird beim Senden des Schritts aufgelöst, seine IPv4-Adresse genommen, wenn er eine hat | `127.0.0.1:9000` | Ja |
| [[ui:exp.payload]] | `text` | Die Nutzdaten, als UTF-8 | `hello`; höchstens 65 507 Bytes | Ja |
| [[ui:exp.expectReply]] | `reply` | Optional: senden und auf die Antwort warten — siehe [Auf eine Antwort warten](#reply) | aus | |

Der Schritt schlägt fehl, wenn ein Ziel nicht erreichbar ist. Ausgänge:
[[ui:exp.outputPort]]. Einstellungen: Neuversuch, Wiederholung, eine Antwort.

```json
{ "id": "ping", "type": "udp", "x": 270, "y": 80, "target": "{{device}}", "text": "PING {{run.id}}",
  "reply": { "bind": "0.0.0.0:0", "mode": "contains", "pattern": "PONG", "timeout_ms": 1000, "variable": "pong" } }
```

### MQTT-Veröffentlichung {#node-mqtt}

Verbindet sich mit einem MQTT-Broker, veröffentlicht eine Nachricht und trennt
sich. Die Verbindung ist MQTT 3.1.1 über einfaches TCP, mit sauberer Sitzung und
ohne Benutzername oder Passwort. Verbinden, Veröffentlichen und die Bestätigung
des Brokers müssen alle innerhalb von 15 Sekunden geschehen.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.broker]] | `host` | Der Hostname oder die Adresse des Brokers | `127.0.0.1` | Ja |
| [[ui:exp.port]] | `port` | | 1883; 1–65 535 | Nein |
| [[ui:exp.topic]] | `topic` | Keine Wildcards (`+`, `#`) | `lab/test` | Ja |
| [[ui:exp.payload]] | `payload` | Die Nachricht, als Text | `hello` | Ja |
| QoS | `qos` | 0, 1 oder 2 | 0 | Nein |
| [[ui:exp.retain]] | `retain` | `true`: Der Broker behält sie als Wert des Topics | `false` | Nein |

Alle sechs Schlüssel sind in einer Datei erforderlich. Der Schritt schlägt fehl,
wenn der Broker nicht erreichbar ist oder die Verbindung oder die Nachricht
verweigert. Ausgänge: [[ui:exp.outputPort]]. Einstellungen: Neuversuch,
Wiederholung.

```json
{ "id": "light", "type": "mqtt", "x": 270, "y": 80, "host": "{{broker}}", "port": 1883,
  "topic": "lab/light/1/set", "payload": "on", "qos": 1, "retain": false }
```

Siehe auch [MQTT](../protocols/mqtt.md).

### WebSocket verbinden {#node-ws_connect}

Öffnet einen WebSocket für den Rest des Durchlaufs oder bis zu einem
[[[ui:exp.node.ws_close]]](#node-ws_close). Was von da an eintrifft, wird für die
[[[ui:exp.node.wait_ws]]](#node-wait_ws)-Schritte darauf behalten. URL und Header
werden aufgelöst, wenn der Schritt läuft, sodass ein früher extrahiertes Token
darin stehen kann. Erneut ausgeführt — in einer [[ui:exp.node.loop]] —, schließt
er zuerst seine vorherige Verbindung und öffnet eine neue. Wenn der Durchlauf
endet, wie auch immer, werden seine Verbindungen mit einem Close-Frame
geschlossen.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| URL | `url` | Eine `ws://`- oder `wss://`-URL | `ws://127.0.0.1:9001/` | Ja |
| [[ui:exp.headers]] | `headers` | `[[name, value], …]`, mit der Upgrade-Anfrage gesendet | keine | Ja, Namen und Werte |
| [[ui:exp.wsProtocols]] | `protocols` | Subprotokolle, die angeboten werden, in Reihenfolge der Präferenz; der Server wählt eines | keine | Nein |
| [[ui:common.timeoutMs]] | `timeout_ms` | Für Verbinden und das Upgrade | 5 000 (10 000 falls nicht vorhanden); 1–120 000 | Nein |

`wss://` vertraut denselben Zertifikaten wie `https://`. Der Schritt schlägt fehl,
wenn die Verbindung oder das Upgrade fehlschlägt; der Status des Servers steht im
Grund. Ausgänge: [[ui:exp.outputPort]]. Einstellungen: Neuversuch (keine
Wiederholung).

```json
{ "id": "socket", "type": "ws_connect", "x": 270, "y": 80, "url": "ws://127.0.0.1:9001/chat",
  "headers": [["Authorization", "Bearer {{token}}"]], "protocols": ["chat.v1"], "timeout_ms": 5000 }
```

Siehe auch [WebSocket](../protocols/websocket.md).

### WebSocket senden {#node-ws_send}

Sendet eine Nachricht auf der Verbindung, die ein [[ui:exp.node.ws_connect]]
geöffnet hat.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | Die id eines [[ui:exp.node.ws_connect]]-Knotens dieses Experiments | der erste | Nein |
| [[ui:exp.wsFormat]] | `binary` | [[ui:exp.wsText]] (`false`) oder [[ui:exp.wsBinary]] (`true`): Die Nutzdaten sind Bytes, als Hex geschrieben, `de ad be ef` | `false` (auch falls nicht vorhanden) | Nein |
| [[ui:exp.payload]] | `text` | Die Nachricht | `hello`; höchstens 16 MiB | Ja |

Der Connect muss auf seinem Pfad vor dem Senden kommen; ein Senden, dessen
Verbindung nicht offen ist, schlägt fehl. Antworten zählen ab dem Moment, in dem
die Nachricht geschrieben wird. Ausgänge: [[ui:exp.outputPort]]. Einstellungen:
Neuversuch, Wiederholung.

```json
{ "id": "hello", "type": "ws_send", "x": 500, "y": 80, "connection": "socket",
  "text": "{\"type\":\"ping\",\"id\":\"{{uuid}}\"}", "binary": false }
```

### WebSocket schließen {#node-ws_close}

Schließt eine Verbindung mit einem Close-Handshake. Die Zeitleiste sagt, wer sie
geschlossen hat: dieser Schritt, der Server früher (mit seinem Code) oder eine
Verbindung, die abgerissen war.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | Die id eines [[ui:exp.node.ws_connect]]-Knotens | der erste | Nein |
| [[ui:field.code]] | `code` | 1000 (normal) oder 3000–4999 für einen eigenen einer Anwendung | 1000 (auch falls nicht vorhanden) | Nein |
| [[ui:field.reason]] | `reason` | Wird mit dem Code gesendet | leer; höchstens 123 Bytes, nach den Vorlagen | Ja |

Ausgänge: [[ui:exp.outputPort]]. Keine Einstellungen.

```json
{ "id": "bye", "type": "ws_close", "x": 960, "y": 80, "connection": "socket", "code": 1000, "reason": "done" }
```

### Log-Marke {#node-log}

Schreibt eine Zeile in die Zeitleiste und den Bericht — einen Kontrollpunkt oder
die Werte, die ein Durchlauf erreichte.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.logMessage]] | `message` | Der Text | `Check point`; höchstens 10 000 Zeichen | Ja |

Ausgänge: [[ui:exp.outputPort]]. Keine Einstellungen.

```json
{ "id": "ready", "type": "log", "x": 500, "y": 80, "message": "device {{device}} ready" }
```

## Warteknoten {#waits}

Die Gruppe [[ui:exp.group.observe]]: Knoten, die darauf warten, dass etwas
eintrifft. Sie teilen diese Regeln:

- **Sie lauschen vom Start des Durchlaufs an.** Der Port oder das
  Broker-Abonnement eines Warteknotens wird vor dem ersten Schritt geöffnet,
  sodass ein Gerät, das schneller antwortet als der nächste Schritt beginnt,
  nicht verpasst wird. Zwei Warteknoten auf derselben Adresse teilen sich einen
  Socket.
- **Sie zählen ab der letzten Aktion auf ihrem Zweig.** Eine Nachricht, die vor
  der letzten Anfrage des Zweigs eintraf, ist keine Antwort darauf; vor jeder
  Aktion zählt alles seit dem Start des Durchlaufs.
- **Die erste passende Nachricht wird genommen.** Eine Nachricht, die ein
  Warteknoten genommen hat, wird von einem anderen nicht gesehen.
- **[[ui:exp.portMatched]] oder [[ui:exp.portTimeout]].** Bei einem Treffer wird
  die Nachricht in der Variablen des Warteknotens gespeichert und der Ablauf
  folgt [[ui:exp.portMatched]]. Wenn die Zeit abläuft, folgt er
  [[ui:exp.portTimeout]], wenn dieser Ausgang verdrahtet ist; andernfalls schlägt
  der Schritt fehl und sagt, wie viele andere Nachrichten eintrafen.
- Jeder Socket behält die neuesten 1 024 Nachrichten (und 64 MiB); ältere werden
  verworfen, und eine Zeitüberschreitung sagt, wie viele.
- [[ui:exp.listenNow]] lauscht mit diesem einen Schritt, von jetzt an.

Ausgänge: [[ui:exp.portMatched]] (erforderlich), [[ui:exp.portTimeout]]
(optional). Einstellungen: Neuversuch.

### Nutzdaten abgleichen {#payload-matching}

[[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_ws]] und
eine UDP-Antwort wählen, wie die Nutzdaten aussehen müssen:

| Option | In der Datei | Passt, wenn die Nutzdaten |
| --- | --- | --- |
| [[ui:exp.mode.any]] | `any` | irgendetwas sind |
| [[ui:exp.mode.contains]] | `contains` | als UTF-8-Text gelesen das Muster enthalten (Schreibweise beachtet) |
| [[ui:exp.mode.regex]] | `regex` | als UTF-8-Text gelesen auf den regulären Ausdruck passen |
| [[ui:exp.mode.hex]] | `hex` | die Bytes enthalten, als Hex-Paare geschrieben: `de ad be ef`, `deadbeef`, `0xde,0xad`, `DE:AD` |

Die passende Nachricht wird als Objekt gespeichert. Spätere Schritte lesen ihre
Felder als `{{reply.text}}` (mit dem Namen der Variablen anstelle von `reply`):

| Feld | Was |
| --- | --- |
| `text` | Die Nutzdaten als Text |
| `hex`, `bytes` | Die Nutzdaten als Hex (ihre ersten 1 024 Bytes) und ihre Größe in Bytes |
| `match` | Was gepasst hat: der Text, die erste Gruppe des regulären Ausdrucks (oder der ganze Treffer) oder die Bytes |
| `from` | `IP:port` des Absenders |
| `ms` | Millisekunden von der letzten Aktion des Zweigs (oder dem Start des Durchlaufs) bis zur Nachricht |
| `topic` | [[ui:exp.node.wait_mqtt]]: das Topic, an das sie veröffentlicht wurde |
| `json`, `kind` | [[ui:exp.node.wait_ws]]: die als JSON geparste Nachricht (`null`, wenn nicht), und `text` oder `binary` |

### Auf OSC warten {#node-wait_osc}

Wartet auf eine OSC-Nachricht, deren Adresse auf ein Muster passt und deren
Argumente jede Regel erfüllen. In einem Bundle ist die erste passende Nachricht
die genommene.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | `IP:port`, auf dem gelauscht wird; `0.0.0.0` für jede Netzwerkkarte | `127.0.0.1:9001` | Nein |
| [[ui:exp.addressPattern]] | `address` | `*` beliebige Zeichen, `?` eines, `[0-9]` eine Menge (`[!0-9]` außerhalb), `{ping,pong}` eines von beiden; Wildcards bleiben innerhalb eines `/`-Segments | `/pong`; höchstens 512 Zeichen | Ja |
| [[ui:exp.argRules]] | `args` | `[{ "index", "op", "value" }, …]`: Argument `index` wird mit `value` durch `op` verglichen ([Vergleiche](#comparisons)); alle müssen gelten | keine; höchstens 16, Index 0–63 | Werte: ja |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (auch falls nicht vorhanden); 1–120 000 | Nein |
| [[ui:exp.replyVariable]] | `variable` | Wo die Nachricht gespeichert wird | `reply` (auch falls nicht vorhanden) | Nein |

Ein Argument wird als Text verglichen: Zahlen wie geschrieben, Zeichenketten ohne
Anführungszeichen, `true`/`false`, ein Blob als Hex. Eine Regel zu einem Argument,
das die Nachricht nicht hat, gilt nicht. Die gespeicherte Nachricht hat `address`,
`args` (`{{reply.args[0]}}`), `from` und `ms`.

```json
{ "id": "status", "type": "wait_osc", "x": 500, "y": 80, "bind": "0.0.0.0:9001", "address": "/status",
  "args": [{ "index": 0, "op": "eq", "value": "ready" }], "timeout_ms": 5000, "variable": "reply" }
```

### Auf UDP warten {#node-wait_udp}

Wartet auf ein UDP-Datagramm, dessen Nutzdaten passen.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | `IP:port`, auf dem gelauscht wird | `127.0.0.1:9001` | Nein |
| [[ui:exp.waitMode]] | `mode` | Siehe [Nutzdaten abgleichen](#payload-matching) | `contains` (`any` falls nicht vorhanden) | Nein |
| [[ui:field.pattern]] | `pattern` | Was die Nutzdaten enthalten oder worauf sie passen müssen | `pong`; erforderlich außer bei `any` | Ja |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (auch falls nicht vorhanden); 1–120 000 | Nein |
| [[ui:exp.replyVariable]] | `variable` | | `reply` (auch falls nicht vorhanden) | Nein |

```json
{ "id": "ready", "type": "wait_udp", "x": 500, "y": 80, "bind": "0.0.0.0:9002", "mode": "contains",
  "pattern": "READY", "timeout_ms": 5000, "variable": "reply" }
```

### Auf MQTT warten {#node-wait_mqtt}

Wartet auf eine Nachricht, die an ein Topic bei einem Broker veröffentlicht wurde
und deren Nutzdaten passen. Der Durchlauf verbindet sich und abonniert vor seinem
ersten Schritt. Retained-Nachrichten, die der Broker beim Abonnieren erneut
zustellt, werden ignoriert: Nur was nach dem Start des Durchlaufs veröffentlicht
wird, zählt.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.broker]] | `host` | Der Broker | `127.0.0.1` | Nur Parameter |
| [[ui:exp.port]] | `port` | | 1883; 1–65 535 | Nein |
| [[ui:exp.topicFilter]] | `topic` | Ein Filter: `+` ist genau eine Ebene, `#` alles darunter (nur zuletzt) | `lab/#` | Nur Parameter |
| [[ui:exp.waitMode]] | `mode` | Siehe [Nutzdaten abgleichen](#payload-matching) | `any` (auch falls nicht vorhanden) | Nein |
| [[ui:field.pattern]] | `pattern` | | leer; erforderlich außer bei `any` | Ja |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (auch falls nicht vorhanden); 1–120 000 | Nein |
| [[ui:exp.replyVariable]] | `variable` | | `reply` (auch falls nicht vorhanden) | Nein |

```json
{ "id": "state", "type": "wait_mqtt", "x": 500, "y": 80, "host": "{{broker}}", "port": 1883,
  "topic": "lab/+/state", "mode": "contains", "pattern": "on", "timeout_ms": 5000, "variable": "reply" }
```

### Auf HTTP-Anfrage warten {#node-wait_http}

Wartet auf eine HTTP-Anfrage — einen Webhook, einen Callback — an den
[[[ui:exp.node.emulator]]](#node-emulator) des Durchlaufs auf dieser Adresse oder,
wenn der Durchlauf dort keinen HTTP-Emulator hat, an einen eigenen Empfänger des
Durchlaufs, der jede Anfrage mit 204 beantwortet. Die Anfrage muss zur Methode,
zum Pfad und zu jeder Bedingung passen.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | `IP:port` | `127.0.0.1:18080` — wo ein neuer [[ui:exp.node.emulator]] lauscht | Nein |
| [[ui:exp.method]] | `method` | Eine Methode oder [[ui:emu.methodAny]] (`ANY`); GET nimmt auch HEAD | `ANY` (auch falls nicht vorhanden) | Nein |
| [[ui:exp.path]] | `path` | `/hooks/:name` benennt ein Segment (`{{request.params.name}}`); ein abschließendes `/*` nimmt den Rest | `/*` (auch falls nicht vorhanden); höchstens 512 Zeichen | Ja |
| [[ui:emu.conditions]] | `when` | `[{ "on", "name", "op", "value" }, …]` zu einem `header`, einem `query`-Parameter, dem `body` oder einem `json`-Pfad; jede muss gelten | keine; höchstens 16 | Ja, Namen und Werte |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 5 000 (2 000 falls nicht vorhanden); 1–120 000 | Nein |
| [[ui:exp.replyVariable]] | `variable` | | `request` (auch falls nicht vorhanden) | Nein |

Die gespeicherte Anfrage hat `method`, `path`, `query`, `headers`, `body`, `json`,
`params`, `from` und `ms`: `{{request.json.event}}`,
`{{request.headers.x-key}}`.

```json
{ "id": "hook", "type": "wait_http", "x": 500, "y": 80, "bind": "127.0.0.1:18081", "method": "POST",
  "path": "/hooks/:name", "when": [{ "on": "json", "name": "$.event", "op": "eq", "value": "deploy" }],
  "timeout_ms": 5000, "variable": "request" }
```

### Auf WebSocket warten {#node-wait_ws}

Wartet auf eine Nachricht auf der Verbindung, die ein
[[ui:exp.node.ws_connect]] geöffnet hat, deren Nutzdaten passen. Nachrichten seit
der letzten Aktion des Zweigs zählen — der Connect selbst, ein Senden oder jede
andere Anfrage.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | Die id eines [[ui:exp.node.ws_connect]]-Knotens | der erste | Nein |
| [[ui:exp.waitMode]] | `mode` | Siehe [Nutzdaten abgleichen](#payload-matching) | `any` (auch falls nicht vorhanden) | Nein |
| [[ui:field.pattern]] | `pattern` | | leer; erforderlich außer bei `any` | Ja |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (auch falls nicht vorhanden); 1–120 000 | Nein |
| [[ui:exp.replyVariable]] | `variable` | | `reply` (auch falls nicht vorhanden) | Nein |

Eine JSON-Nachricht ist Feld für Feld lesbar: `{{reply.json.type}}`. Der Connect
muss auf seinem Pfad vor dem Warten kommen.

```json
{ "id": "pong", "type": "wait_ws", "x": 730, "y": 80, "connection": "socket", "mode": "contains",
  "pattern": "pong", "timeout_ms": 3000, "variable": "reply" }
```

## Emulation {#emulation}

### Emulator {#node-emulator}

Spielt eine Abhängigkeit — eine HTTP-API, ein OSC-, UDP- oder TCP-Gerät, einen
MQTT-Broker — für den ganzen Durchlauf. Er öffnet vor dem ersten Schritt und
antwortet, bis der Durchlauf endet; im Ablauf besteht der Schritt sofort. Was er
empfangen hat, wird Regel für Regel im Bericht des Durchlaufs gezählt.

| Feld | In der Datei | Was | Standard |
| --- | --- | --- | --- |
| [[ui:emu.edit]] | `emulator` | Der Emulator: `name`, `bind` (`IP:port`), `protocol` (`http`, `osc`, `udp`, `tcp`, `mqtt`), seine Routen oder Regeln und ein optionaler `outage` | Eine HTTP-API namens *API* auf `127.0.0.1:18080`, die `/health` beantwortet |

Die Eigenschaften zeigen in einer Zeile, was er spielt. [[ui:emu.edit]] öffnet
seine Regeln, denselben Editor wie in der Ansicht
[[[ui:nav.emulators]]](../tools/emulators.md); [[ui:emu.toLibrary]] behält eine
Kopie in der Emulator-Bibliothek, und [[ui:emu.fromLibrary]] ersetzt diesen durch
eine Kopie von dort. Die Regeln — Routen, Antworten, Fehlverhalten, Ausfälle —
sind dort beschrieben.

- Ein HTTP-Emulator ist auch das, worauf ein
  [[[ui:exp.node.wait_http]]](#node-wait_http) an seiner Adresse hört; ein OSC-
  oder UDP-Emulator teilt seinen Port mit den Warteknoten des Durchlaufs dort.
- Zwei Emulatoren eines Transports können sich in einem Durchlauf keinen Port
  teilen.
- [[[ui:exp.node.emulator_state]]](#node-emulator_state) schaltet ihn aus und
  wieder ein.

Ausgänge: [[ui:exp.outputPort]]. Keine Einstellungen.

```json
{ "id": "api", "type": "emulator", "x": 270, "y": 80,
  "emulator": { "name": "Orders API", "bind": "127.0.0.1:18080", "protocol": "http",
    "routes": [{ "method": "GET", "path": "/orders/:id", "order": "sequence",
                 "responses": [{ "status": 503 }, { "status": 200, "body": "{\"id\":\"{{request.params.id}}\"}" }] }] } }
```

## Störungen {#faults}

Knoten, die Dinge auf Kommando kaputtmachen. Ein Zweig aus
[[ui:exp.node.delay]]-Knoten und diesen neben dem Verkehr liest sich als Zeitplan;
[Störungen nach Zeitplan](faults.md) zeigt, wie.

### Störung {#node-impairment}

Ein Störungs-Relais für den ganzen Durchlauf: Das getestete System sendet an (oder
verbindet sich mit) [[ui:exp.relayListen]] statt an das echte Ziel; das Relais
leitet an [[ui:exp.relayTarget]] weiter, und die Antworten kommen denselben Weg
zurück, gestört durch das Profil. Es öffnet vor dem ersten Schritt und schließt,
wenn der Durchlauf endet, wie auch immer, sodass nichts gestört bleibt; im Ablauf
besteht der Schritt sofort. Jede Entscheidung zieht aus dem Startwert des
Durchlaufs: Derselbe Startwert und derselbe Verkehr erleiden dasselbe Schicksal.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.relayListen]] | `listen` | `IP:port`, an das das getestete System sendet | `127.0.0.1:9010` | Nur Parameter |
| [[ui:exp.relayTarget]] | `target` | `IP:port` des echten Ziels oder `host:port` — ein Hostname wird beim Start des Durchlaufs aufgelöst, und ein Name, der nicht gefunden wird, stoppt den Durchlauf an diesem Knoten | `127.0.0.1:9000` | Nur Parameter |
| [[ui:ns.protocol]] | `protocol` | UDP (`udp`): jedes Datagramm erleidet sein eigenes Schicksal; TCP (`tcp`): jede Verbindung wird mit einer eigenen zum Ziel gekoppelt, und beide Ströme werden gestört | UDP (`udp` falls nicht vorhanden) | Nein |
| [[ui:ns.preset]] und die Werte darunter | `profile` | Was das Relais dem Verkehr antut — siehe [Das Profil](#impair-profile) | [[ui:ns.preset.lan]] (keine Störung falls nicht vorhanden) | Nein |

Die Lauschadresse eines Relais kann kein anderer Socket des Durchlaufs sein, und
Relais dürfen nicht im Kreis zueinander weiterleiten. Ein per Name gegebenes Ziel
wird verfolgt, sobald es aufgelöst ist, sodass ein Kreis über einen Namen den
Durchlauf bei seinem Start stoppt. Der Bericht zählt jede Phase eines Relais
einzeln.

Ausgänge: [[ui:exp.outputPort]]. Keine Einstellungen.

```json
{ "id": "relay", "type": "impairment", "x": 270, "y": 80, "listen": "127.0.0.1:9010", "target": "{{device}}",
  "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } }
```

#### Das Profil {#impair-profile}

Ein Vorgabe-Chip — [[ui:ns.preset.lan]], [[ui:ns.preset.wifi]],
[[ui:ns.preset.4g]], [[ui:ns.preset.satellite]],
[[ui:ns.preset.intermittent]], [[ui:ns.preset.offline]] — füllt jeden Wert aus;
ändern Sie danach einen beliebigen davon. Ein Relais liest nur die Werte seines
Protokolls; in einer Datei darf jeder Schlüssel weggelassen werden (null, aus).

| Feld | In der Datei | Was | Grenzen | Protokoll |
| --- | --- | --- | --- | --- |
| — | `name` | Eine Bezeichnung für die Zeitleiste und den Bericht: der Schlüssel einer Vorgabe (`lan`, `wifi`, `4g`, `satellite`, `intermittent`, `offline`) oder eine eigene | höchstens 60 Zeichen | beide |
| [[ui:ns.offline]] | `offline` | Nichts kommt durch | `true` / `false` | beide |
| [[ui:ns.latency]] | `latency_ms` | Verzögerung, die jedem Paket oder Block eines Stroms hinzugefügt wird | 0–60 000 (der Schieberegler geht bis 1 000) | beide |
| [[ui:ns.jitter]] | `jitter_ms` | Eine zufällige zusätzliche Verzögerung bis zu so viel; ein TCP-Strom bleibt in der Reihenfolge | 0–60 000 (der Schieberegler geht bis 500) | beide |
| [[ui:ns.rate]] | `rate_kbps` | Eine Bandbreitengrenze, 0 für keine. UDP: ab einer Sekunde Warteschlange werden Datagramme als gedrosselt verworfen; TCP: der Sender wird gebremst, nichts wird verworfen | 0 oder 8–10 000 000 | beide |
| [[ui:ns.loss]] | `loss` | Die Wahrscheinlichkeit, dass ein Datagramm verworfen wird | 0–1 (der Schieberegler zeigt %) | UDP |
| [[ui:ns.burst]], [[ui:ns.burstLength]] | `burst_start`, `burst_length` | Die Wahrscheinlichkeit, dass ein Verlust-Burst beginnt, und wie viele Datagramme er im Mittel dauert | 0–1; 1–1 000, wenn Bursts an sind | UDP |
| [[ui:ns.duplicate]] | `duplicate` | Die Wahrscheinlichkeit, dass ein Datagramm zweimal gesendet wird | 0–1 | UDP |
| [[ui:ns.corrupt]] | `corrupt` | Die Wahrscheinlichkeit, dass ein Bit eines Datagramms gekippt wird | 0–1 | UDP |
| [[ui:ns.reorder]] | `reorder` | Die Wahrscheinlichkeit, dass ein Datagramm zurückgehalten wird, sodass spätere es überholen | 0–1 | UDP |
| [[ui:ns.reset]] | `reset` | Die Wahrscheinlichkeit, dass ein Block eines Stroms stattdessen seine Verbindung zurücksetzt — beide Seiten erhalten ein Reset | 0–1 | TCP |
| [[ui:ns.stall]] | `stall` | Die Wahrscheinlichkeit, dass ein Block seine Verbindung halboffen lässt: nichts geht mehr in beide Richtungen durch, und keiner Seite wird es gesagt | 0–1 | TCP |

Mehr über Relais, Vorgaben und was sie modellieren auf der Seite
[[[ui:nav.netsim]]](../tools/impairment.md).

### Störung ändern {#node-impairment_change}

Schaltet einen der [[ui:exp.node.impairment]]-Knoten des Durchlaufs ab diesem
Schritt auf ein anderes Profil um, ohne seinen Port zu verlieren. Die bisherige
Phase wird geschlossen und im Bericht gezählt.

| Feld | In der Datei | Was | Standard | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.relay]] | `relay` | Die id eines [[ui:exp.node.impairment]]-Knotens dieses Experiments | der erste | Nein |
| [[ui:ns.preset]] und die Werte darunter | `profile` | Womit es von jetzt an stört — siehe [Das Profil](#impair-profile); das Relais liest die Werte seines eigenen Protokolls | [[ui:ns.preset.offline]] (keine Störung falls nicht vorhanden) | Nein |

Der Schritt schlägt fehl, wenn das Relais nicht läuft — es konnte etwa nicht
weiterleiten. Ausgänge: [[ui:exp.outputPort]]. Keine Einstellungen.

```json
{ "id": "cut", "type": "impairment_change", "x": 730, "y": 200, "relay": "relay",
  "profile": { "name": "offline", "offline": true } }
```

### Emulator aus und an {#node-emulator_state}

Schaltet einen der Emulatoren des Durchlaufs aus oder wieder ein. Während er aus
ist, antwortet ein HTTP-Emulator, wie [[ui:exp.downFault]] sagt; ein TCP-Gerät und
ein MQTT-Broker trennen ihre Verbindungen und verweigern neue; OSC- und UDP-Geräte
antworten nichts. Wieder an, folgt der Emulator seinem eigenen Ausfallplan, wenn er
einen hat.

| Feld | In der Datei | Was | Standard |
| --- | --- | --- | --- |
| [[ui:exp.emulatorNode]] | `emulator` | Die id eines [[ui:exp.node.emulator]]-Knotens dieses Experiments | der erste |
| [[ui:exp.emulatorDownState]] | `down` | [[ui:exp.emulatorGoesDown]] (`true`) oder [[ui:exp.emulatorComesUp]] (`false`) | aus (`false` falls nicht vorhanden) |
| [[ui:exp.downFault]] | `fault` | Nur HTTP: [[ui:emu.outageFault.unavailable]] (`unavailable`), [[ui:emu.outageFault.reset]] (`reset`: die Verbindung wird ohne Antwort geschlossen) oder [[ui:emu.outageFault.timeout]] (`timeout`: die Anfrage wird festgehalten, bis der Client aufgibt, höchstens 120 s) | `unavailable` (auch falls nicht vorhanden) |

Ausgänge: [[ui:exp.outputPort]]. Keine Einstellungen. Nichts wird mit Vorlagen
gefüllt.

```json
{ "id": "down", "type": "emulator_state", "x": 500, "y": 200, "emulator": "api", "down": true, "fault": "unavailable" }
```

## Daten {#data}

### Wert extrahieren {#node-extract}

Speichert einen Teil der neuesten HTTP-Antwort auf seinem Pfad als Variable für
spätere Felder (`{{token}}`), Prüfungen und Zweige. Eine HTTP-Anfrage muss auf
jedem Pfad davor kommen. Ein Klick auf einen Wert in einer
[[ui:exp.sendNow]]-Antwort fügt einen für Sie hinzu.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.variable]] | `variable` | Der Name: Buchstaben, Ziffern und `_`, nicht mit einer Ziffer beginnend, kein reserviertes Wort, kein Name eines Parameters | `token` | Nein |
| [[ui:exp.extractFrom]] | `from` | [[ui:exp.from.json]] (`json`), [[ui:exp.from.header]] (`header`), [[ui:exp.from.status]] (`status`), [[ui:exp.from.body]] (`body`) oder [[ui:exp.from.regex]] (`regex`) | `json` | Nein |
| [[ui:exp.jsonPath]], [[ui:exp.headerName]] oder [[ui:exp.pattern]] | `expr` | Ein JSON-Pfad (`$.data.token`, `$.items[0]`, `$["first name"]`), ein Headername (beliebige Schreibweise) oder ein regulärer Ausdruck — seine erste Gruppe oder der ganze Treffer | `$.token`; nicht verwendet für Status und Body | Nein |

Der Schritt schlägt fehl, wenn es nichts zu nehmen gibt: Der Body ist kein JSON,
der Pfad oder Header fehlt, der Ausdruck passt nicht, oder — bei einem JSON-Feld
oder dem ganzen Body — der Body war länger als die behaltenen 256 KiB. Ein Status
wird als Zahl gespeichert; der Rest als Text oder als der gefundene JSON-Wert.
Ausgänge: [[ui:exp.outputPort]]. Keine Einstellungen.

```json
{ "id": "token", "type": "extract", "x": 500, "y": 80, "variable": "token", "from": "json", "expr": "$.data.token" }
```

Mehr über Variablen in [Daten und Vorlagen](data.md).

## Prüfungen {#checks}

Eine Prüfung besteht oder lässt den Durchlauf fehlschlagen. Die vier
Antwortprüfungen lesen die neueste HTTP-Antwort auf ihrem Pfad, daher muss eine
HTTP-Anfrage — keine unter Last — auf jedem Pfad vor ihnen kommen.

### HTTP-Status {#node-assert_status}

Besteht, wenn der Status der neuesten Antwort genau der angegebene ist.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.expectedStatus]] | `status` | | 200; 100–599 | Nein |

Ausgänge: [[ui:exp.outputPort]]. Keine Einstellungen.

```json
{ "id": "ok", "type": "assert_status", "x": 500, "y": 80, "status": 200 }
```

### Antworttext {#node-assert_body}

Besteht, wenn der Body der neuesten Antwort den Text exakt enthält (Schreibweise
eingeschlossen). Nur die ersten 256 KiB eines Bodys werden behalten: Text, der in
einem abgeschnittenen Body nicht gefunden wird, schlägt mit diesem Grund fehl.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.contains]] | `contains` | | `ok`; erforderlich | Ja |

Ausgänge: [[ui:exp.outputPort]]. Keine Einstellungen.

```json
{ "id": "ready", "type": "assert_body", "x": 500, "y": 80, "contains": "ready" }
```

### Antwort-Header {#node-assert_header}

Besteht, wenn die neueste Antwort den Header hat und sein Wert den Text enthält.
Der Name des Headers wird in beliebiger Schreibweise abgeglichen, der Wert exakt.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.headerName]] | `name` | | `content-type`; erforderlich | Ja |
| [[ui:exp.contains]] | `contains` | Was sein Wert enthalten muss; leer: Der Header muss nur da sein | `application/json` | Ja |

Ausgänge: [[ui:exp.outputPort]]. Keine Einstellungen.

```json
{ "id": "json", "type": "assert_header", "x": 500, "y": 80, "name": "Content-Type", "contains": "json" }
```

### Antwortzeit {#node-assert_latency}

Besteht, wenn die neueste Antwort höchstens so lange dauerte, vom Senden der
Anfrage bis zum Ende ihres Bodys.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.maxLatency]] | `max_ms` | | 1 000; 1–120 000 | Nein |

Ausgänge: [[ui:exp.outputPort]]. Keine Einstellungen.

```json
{ "id": "fast", "type": "assert_latency", "x": 500, "y": 80, "max_ms": 250 }
```

### Wert prüfen {#node-assert_value}

Vergleicht einen Wert — meist eine Variable, als Vorlage geschrieben — mit einem
erwarteten und besteht, wenn der Vergleich gilt.

| Feld | In der Datei | Was | Standard | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.value]] | `value` | Was verglichen wird: `{{token}}`, `{{reply.args[0]}}` | `{{token}}` | Ja |
| [[ui:exp.operator]] | `op` | Siehe [Vergleiche](#comparisons) | [[ui:exp.op.not_empty]] | Nein |
| [[ui:exp.expected]] | `expected` | Nicht von [[ui:exp.op.empty]] und [[ui:exp.op.not_empty]] verwendet | leer (auch falls nicht vorhanden) | Ja |

Ausgänge: [[ui:exp.outputPort]]. Keine Einstellungen.

```json
{ "id": "state", "type": "assert_value", "x": 730, "y": 80, "value": "{{state}}", "op": "eq", "expected": "ready" }
```

### Vergleiche {#comparisons}

[[ui:exp.node.assert_value]], [[ui:exp.node.branch_value]], die Abbruchbedingung
einer [[ui:exp.node.loop]], OSC-Argumentregeln und HTTP-Bedingungen vergleichen
gleich:

| Option | In der Datei | Gilt, wenn der Wert |
| --- | --- | --- |
| [[ui:exp.op.eq]] | `eq` | dem erwarteten entspricht — als Zahlen, wenn beide Zahlen sind (`200` = `200.0`), sonst als exakter Text |
| [[ui:exp.op.ne]] | `ne` | ihm nicht entspricht, nach derselben Regel |
| [[ui:exp.op.lt]], [[ui:exp.op.le]], [[ui:exp.op.gt]], [[ui:exp.op.ge]] | `lt`, `le`, `gt`, `ge` | kleiner, höchstens, größer, mindestens ist — beide müssen Zahlen sein: andernfalls lässt eine Prüfung, ein Zweig oder eine [[ui:exp.node.loop]] den Schritt fehlschlagen, und eine Argumentregel oder eine HTTP-Bedingung gilt nicht |
| [[ui:exp.op.contains]] | `contains` | den erwarteten Text enthält |
| [[ui:exp.op.matches]] | `matches` | auf den erwarteten regulären Ausdruck passt |
| [[ui:exp.op.empty]], [[ui:exp.op.not_empty]] | `empty`, `not_empty` | leer ist (Leerzeichen zählen als leer) / nicht |

## Ablauf {#flow}

Knoten, die entscheiden, wohin der Durchlauf geht. Mehr über Zweige, Joins und
Schleifen in [Ablauf](flow.md).

### Start {#node-start}

Wo der Durchlauf beginnt; jedes Experiment hat genau einen. Er hat keinen Eingang
und keine Felder. Die erste Zeile der Zeitleiste gibt den Startwert des Durchlaufs.

Ausgänge: [[ui:exp.outputPort]], erforderlich. Mehrere Verbindungen von ihm starten
auf einmal parallele Zweige.

```json
{ "id": "start", "type": "start", "x": 40, "y": 80 }
```

### Ende {#node-end}

Wo der Durchlauf vollständig wird; jedes Experiment hat genau ein Ende, und es hat
keine Ausgänge. Mehrere Zweige können dorthin führen: Der Durchlauf geht einmal
weiter, nachdem der letzte Zweig fertig ist, und nur, wenn keiner fehlschlug. Ein
Durchlauf, der [[ui:exp.node.end]] nie erreicht, schlägt fehl.

```json
{ "id": "end", "type": "end", "x": 960, "y": 80 }
```

### Verzögerung {#node-delay}

Wartet eine feste Zeit vor dem nächsten Schritt.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.delayMs]] | `ms` | | 300; 0–60 000 | Nein |

Ausgänge: [[ui:exp.outputPort]]. Keine Einstellungen. Für längere Wartezeiten
setzen Sie mehrere hintereinander oder in eine [[ui:exp.node.loop]].

```json
{ "id": "pause", "type": "delay", "x": 500, "y": 80, "ms": 500 }
```

### Statuszweig {#node-branch_status}

Wählt [[ui:exp.yes]], wenn die neueste HTTP-Antwort diesen Status hat, sonst
[[ui:exp.no]]. Eine HTTP-Anfrage muss auf jedem Pfad davor kommen.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.expectedStatus]] | `status` | | 200; 100–599 | Nein |

Ausgänge: [[ui:exp.yes]] und [[ui:exp.no]], beide erforderlich. Keine
Einstellungen.

```json
{ "id": "branch", "type": "branch_status", "x": 500, "y": 80, "status": 200 }
```

### Zweig auf einen Wert {#node-branch_value}

Wählt [[ui:exp.yes]], wenn ein Vergleich gilt, sonst [[ui:exp.no]]. Seine Felder
und [Vergleiche](#comparisons) sind die von
[[[ui:exp.node.assert_value]]](#node-assert_value); ein Vergleich, der sich nicht
anstellen lässt (`lt` auf Text), lässt den Schritt fehlschlagen.

| Feld | In der Datei | Was | Standard | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.value]] | `value` | Was verglichen wird | `{{token}}` | Ja |
| [[ui:exp.operator]] | `op` | | [[ui:exp.op.eq]] | Nein |
| [[ui:exp.expected]] | `expected` | | leer (auch falls nicht vorhanden) | Ja |

Ausgänge: [[ui:exp.yes]] und [[ui:exp.no]], beide erforderlich. Keine
Einstellungen.

```json
{ "id": "ok", "type": "branch_value", "x": 730, "y": 80, "value": "{{reply.args[0]}}", "op": "eq", "expected": "ok" }
```

### Paralleler Zweig {#node-fork}

Führt, was auf [[ui:exp.branch1]] und [[ui:exp.branch2]] folgt, zur selben Zeit
aus, jeder Zweig mit seiner eigenen Kopie der Variablen. Jeder Ausgang darf mehr
Verbindungen für mehr Zweige haben. Keine Felder.

Ausgänge: [[ui:exp.branch1]] und [[ui:exp.branch2]], beide erforderlich.

```json
{ "id": "split", "type": "fork", "x": 270, "y": 80 }
```

### Zweige zusammenführen {#node-join}

Wartet, bis jede in ihn führende Verbindung erreicht wurde, und fährt dann einmal
fort, mit den zusammengeführten Variablen der Zweige — wo zwei Zweige dieselbe
Variable setzen, gewinnt die, deren Verbindung später in der Datei steht — und der
neuesten HTTP-Antwort des letzten von ihnen, der eine hatte. Keine Felder.

Nur Zweige, die alle laufen, treffen sich hier: Ein [[ui:exp.node.join]] hinter
einem [[ui:exp.node.branch_status]], dessen [[ui:exp.yes]] und [[ui:exp.no]] nie
beide geschehen, fährt nie fort; wenn kein anderer Pfad [[ui:exp.node.end]]
erreicht, schlägt der Durchlauf an diesem Knoten fehl und sagt, auf wie viele
Zweige er noch wartete.

Ausgänge: [[ui:exp.outputPort]], erforderlich.

Jeder andere Knoten mit mehreren in ihn führenden Verbindungen läuft einmal für
jedes Eintreffen.

```json
{ "id": "joined", "type": "join", "x": 730, "y": 80 }
```

### Schleife {#node-loop}

Führt die Schritte an [[ui:exp.portBody]] — die zurück zu ihr führen — immer wieder
aus: höchstens eine Anzahl von Malen und, wenn sie eine Abbruchbedingung hat, bis
diese gilt.

| Feld | In der Datei | Was | Standard und Grenzen | Vorlagen |
| --- | --- | --- | --- | --- |
| [[ui:exp.loopMax]] | `max` | Höchstens so viele Iterationen | 5; 1–1 000 | Nein |
| [[ui:exp.loopUntilOn]] | `until` | Optionale Abbruchbedingung `{ "value", "op", "expected" }`, wie bei [[[ui:exp.node.assert_value]]](#node-assert_value) | aus | Wert und erwarteter Wert: ja |

- Der Rumpf läuft immer mindestens einmal. Die Abbruchbedingung wird nach jeder
  Iteration gelesen, sodass der Rumpf setzen kann, was er prüft.
- [[ui:exp.portDone]] folgt, wenn die Bedingung gilt — oder, ohne Bedingung, nach
  der letzten Iteration.
- [[ui:exp.portLimit]] folgt, wenn die Iterationen vor der Bedingung ausgingen.
  Ohne eine Verbindung daran lässt das den Schritt fehlschlagen.
- Im Rumpf ist `{{counter}}` die Nummer der Iteration.
- Ein Rumpf läuft als ein Zweig: Jeder Ausgang in ihm hat eine Verbindung; er
  enthält kein [[ui:exp.node.start]], [[ui:exp.node.end]], [[ui:exp.node.fork]],
  [[ui:exp.node.join]] oder anderes [[ui:exp.node.loop]]; er wird nur durch
  [[ui:exp.portBody]] betreten; und jede Verbindung in ihm führt im Rumpf weiter
  oder zurück zur Schleife.

Ausgänge: [[ui:exp.portBody]] und [[ui:exp.portDone]] (erforderlich),
[[ui:exp.portLimit]] (optional). Keine Einstellungen.

```json
{ "id": "poll", "type": "loop", "x": 270, "y": 80, "max": 10,
  "until": { "value": "{{status.args[0]}}", "op": "eq", "expected": "ready" } }
```


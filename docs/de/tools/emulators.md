---
title: Emulatoren
description: Machen Sie Signal Lab zur Gegenseite — einer HTTP-API, einem OSC-, UDP- oder TCP-Gerät oder einem MQTT-Broker —, die nach Ihren Regeln antwortet, auf Kommando scheitert und zählt, was eintrifft.
---

# Emulatoren

Ein Emulator ist Signal Lab in der Rolle der API, des Geräts oder des Dienstes, mit dem Ihr System
spricht. Er empfängt auf einer Adresse und antwortet nach Regeln: eine HTTP-API nach Routen, ein
OSC-, UDP- oder TCP-Gerät nach dem Muster „darauf antworte so“, ein MQTT-Broker wie jeder Broker,
dazu mit eigenen Regeln. Er kann langsam sein, scheitern oder ab und zu ausfallen, sodass
Sie testen können, was Ihr System tut, wenn seine Abhängigkeit sich danebenbenimmt. Jeder
Austausch wird gezählt, aufgelistet und an den [Inspektor](inspector.md) gesendet.

Ein Emulator ist ein einziges Dokument. Die Ansicht [[ui:nav.emulators]] führt eine Bibliothek
davon; dasselbe Dokument läuft in einem Experiment als
Knoten [[ui:exp.node.emulator]], auf der Kommandozeile mit `signallab emulate`
und über die [API](../api/commands.md) und [MCP](../automation/mcp.md), und
es antwortet überall gleich.

## Die Ansicht {#screen}

Links steht die Bibliothek ([[ui:emu.library]]): jeder Emulator mit Protokoll und
Adresse, bei den laufenden ein pulsierender Punkt und die Zahl der Anfragen.
Rechts stehen Einstellungen und Regeln des ausgewählten Emulators und darunter,
was er empfangen hat ([[ui:emu.live]]).

## Einen Emulator anlegen {#create}

1. Drücken Sie eine der Schaltflächen oben in der Bibliothek:

   | Schaltfläche | Legt an | Empfängt auf | Mit einer Regel, die sofort funktioniert |
   | --- | --- | --- | --- |
   | ＋ [[ui:emu.new.http]] | Eine HTTP-API | `127.0.0.1:18080` | `GET /health` → 200 `{"status":"ok"}` |
   | ＋ [[ui:emu.new.osc]] | Ein OSC-Gerät | `127.0.0.1:9100` | `/ping` → `/pong` mit dem Zähler als int |
   | ＋ [[ui:emu.new.udp]] | Ein UDP-Gerät | `127.0.0.1:7100` | ein Datagramm, das `PING` enthält → `PONG 1`, `PONG 2`, … |
   | ＋ [[ui:emu.new.tcp]] | Ein TCP-Gerät | `127.0.0.1:7200` | eine Zeile, die `PING` enthält → `PONG` |
   | ＋ [[ui:emu.new.mqtt]] | Einen MQTT-Broker | `127.0.0.1:1883` | eine Veröffentlichung an `lab/<name>/set` → dieselben Nutzdaten, retained, auf `lab/<name>/state` |

   Nutzt bereits ein anderer Emulator der Bibliothek diesen Port, wird der nächste freie
   genommen.
2. Geben Sie ihm einen [[ui:emu.name]] (höchstens 120 Zeichen).
3. Setzen Sie das Feld [[ui:emu.bind]] auf `IP:port`. `127.0.0.1` antwortet nur diesem Computer;
   `0.0.0.0` antwortet auch dem Netzwerk.
4. Ändern Sie die Regeln (siehe unten), und halten Sie unter [[ui:emu.note]] fest, wofür er einspringt.

Änderungen werden von selbst gespeichert. [[ui:emu.duplicate]] legt eine Kopie auf dem nächsten
freien Port an. [[ui:emu.delete]] fragt noch einmal nach ([[ui:emu.confirmDelete]]), stoppt
den Emulator, falls er läuft, und entfernt ihn aus der Bibliothek.

Regeln werden der Reihe nach geprüft, von der ersten bis zur letzten; die erste, die passt, antwortet. Der Kopf jeder
Regel zeigt eine einzeilige Zusammenfassung; klicken Sie darauf, um die Regel auf- oder zuzuklappen. Die
Schaltflächen ↑ und ↓ verschieben eine Regel, × entfernt sie.

## Starten {#run}

1. Wählen Sie den Emulator aus und drücken Sie [[ui:emu.start]]. Sein Port öffnet sich, bevor die
   Schaltfläche wieder bereit ist: Ein bereits belegter Port oder ein Emulator mit einem Problem wird
   dort mit dem Grund abgelehnt.
2. Richten Sie Ihr System darauf. Bei einer HTTP-API kopiert [[ui:emu.copyUrl]] ihre
   Adresse (`http://127.0.0.1:18080`), und jede Route hat eine Schaltfläche
   [[ui:emu.copyRouteUrl]] für ihre eigene (außer wenn ihr Pfad eine
   Vorlage `{{…}}` enthält).
3. Beobachten Sie, wie sich [[ui:emu.received]] füllt.
4. Drücken Sie [[ui:emu.stop]] oder stoppen Sie seinen Job in der Leiste der Konsole.

Der Zustand neben den Schaltflächen zeigt [[ui:emu.notRunning]], wo er antwortet oder
dass er ausgefallen ist.

Ein Emulator antwortet weiter mit den Regeln, mit denen er gestartet wurde. Ändern Sie
ihn, während er läuft, erscheint [[ui:emu.restart]]: Drücken Sie darauf, um ihn mit den Regeln neu zu starten,
wie sie jetzt sind. Bis dahin sind die Trefferzahlen an den Regeln ausgeblendet,
da sie zu den alten Regeln gehören.

[[ui:emu.takeDown]] macht einen laufenden Emulator unerreichbar, bis Sie
[[ui:emu.bringUp]] drücken: Eine HTTP-Anfrage erhält 503, ein TCP-Gerät und ein MQTT-Broker
trennen ihre Verbindungen und verweigern neue, ein OSC- oder UDP-Gerät antwortet
gar nicht. Siehe [Ausfälle](#outage).

Zwei Emulatoren desselben Transports können sich keinen Port teilen: HTTP-, TCP- und MQTT-Emulatoren
empfangen auf TCP-Ports, OSC- und UDP-Emulatoren auf UDP-Ports. Eine HTTP-API
und ein OSC-Gerät können beide Port 8080 verwenden; zwei HTTP-APIs nicht. Ein zweiter auf
einem belegten Port wird beim Start abgelehnt.

::: tip
In einem Browser, der mit einem [Server](../server/index.md) verbunden ist, läuft der Emulator auf
dem Server. Einer, der auf `0.0.0.0` empfängt, ist über den Namen des Servers erreichbar, und
[[ui:emu.copyUrl]] kopiert diese Adresse; einer auf `127.0.0.1` antwortet nur
Programmen auf dem Server selbst.
:::

## Was eingetroffen ist {#received}

Während er läuft, zählt [[ui:emu.live]]:

| Zähler | Was |
| --- | --- |
| [[ui:emu.total]] | Alles, was eingetroffen ist: Anfragen, Nachrichten, Zeilen. |
| [[ui:emu.unmatched]] | Was keine Regel angenommen hat. Eine HTTP-Anfrage ohne Route erhält trotzdem ihre Antwort (siehe [Anfragen, die keine Route annimmt](#fallback)); die anderen erhalten keine. |
| [[ui:emu.failed]] | Austausche, bei denen eine Antwort nicht erzeugt oder nicht gesendet werden konnte. |
| [[ui:emu.down]] | Was eintraf, während der Emulator ausgefallen war. Wird angezeigt, wenn ein Ausfall eingestellt ist oder etwas ihn im Ausfall angetroffen hat. Nie als [[ui:emu.unmatched]] gezählt. |
| [[ui:emu.missed]] | Nur MQTT, wenn es vorkommt: Nachrichten, die ein Client nicht annehmen konnte, weil er zu weit zurücklag. |

Der Kopf jeder Regel zeigt, wie oft sie seit dem Start gepasst hat.

[[ui:emu.received]] listet die neuesten 300 Austausche auf, die neuesten zuerst:

| Spalte | Was |
| --- | --- |
| [[ui:emu.col.time]] | Wann er eingetroffen ist. |
| [[ui:emu.col.from]] | Die Adresse des Clients. |
| [[ui:emu.col.request]] | Was eingetroffen ist, in Protokollnotation: `GET /users/7`, `/ping 1`, `POWER?`. |
| [[ui:emu.col.rule]] | Die Regel, die ihn angenommen hat (`#2`), oder `—`. |
| [[ui:emu.col.reply]] | Was zurückging: `200 OK · 37 B`, `/pong 3`, Nutzdaten; [[ui:emu.held]] oder [[ui:emu.closed]] bei einem Fehlverhalten; der Fehler, wenn die Antwort fehlschlug; [[ui:emu.wasDown]], wenn er während eines Ausfalls eintraf. |
| [[ui:emu.col.ms]] | Vom Eintreffen bis zum Abgang der Antwort, ihre Verzögerung eingeschlossen. |

Die Schaltfläche ⌕ in einer Zeile ([[ui:emu.inspectFrame]]) öffnet diesen Austausch im
Inspektor, sofern der Mitschnitt lief. Treffen mehr als 200 Austausche innerhalb einer
Fünftelsekunde ein, überspringt die Liste einige und sagt, wie viele. Die Engine behält die
neuesten 500 Austausche jedes laufenden Emulators samt dem Eingetroffenen für die
Kommandozeile, die API und MCP.

## HTTP-API {#http}

Ein HTTP/1.1-Server. Jede Anfrage beantwortet die erste Route, die sie annimmt.

### Routen {#routes}

Eine Route nimmt eine Anfrage an, wenn Methode, Pfad und alle Bedingungen passen.

| Feld | Was |
| --- | --- |
| [[ui:emu.method]] | `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS` oder [[ui:emu.methodAny]]. Eine `GET`-Route beantwortet auch `HEAD`. |
| [[ui:emu.path]] | Beginnt mit `/`. Ein Segment `:name` nimmt ein beliebiges einzelnes Segment an, lesbar als `{{request.params.name}}`; ein letztes Segment `*` nimmt alles darunter an. Ein abschließendes `/` macht keinen Unterschied; der Query-String gehört nicht zum Pfad. |
| [[ui:emu.conditions]] | Jede muss erfüllt sein. Fügen Sie eine mit ＋ [[ui:emu.addCondition]] hinzu. |

Pfadbeispiele:

| Pfad | Nimmt an | Nimmt nicht an |
| --- | --- | --- |
| `/health` | `/health`, `/health/` | `/health/db`, `/Health` |
| `/users/:id` | `/users/7` (`params.id` ist `7`), `/users/a%20b` (`a b`) | `/users`, `/users/7/orders` |
| `/files/*` | `/files`, `/files/a`, `/files/a/b/c` | `/file`, `/other/files/a` |

Eine Bedingung liest einen Teil der Anfrage ([[ui:emu.on]]) und vergleicht ihn:

| [[ui:emu.on]] | Name | Liest |
| --- | --- | --- |
| [[ui:emu.on.header]] | Ein Header-Name, Groß- und Kleinschreibung egal | Den Wert des Headers; bei einem mehrfach gesendeten Header seine Werte, verbunden mit `, `. |
| [[ui:emu.on.query]] | Ein Query-Parameter | Seinen Wert, dekodiert; den ersten, wenn er wiederholt vorkommt. |
| [[ui:emu.on.body]] | — | Den ganzen Body als Text. |
| [[ui:emu.on.json]] | Ein JSON-Pfad wie `$.user.id` | Dieses Feld eines JSON-Bodys. |

Die Vergleiche sind [[ui:exp.op.eq]], [[ui:exp.op.ne]], [[ui:exp.op.lt]],
[[ui:exp.op.le]], [[ui:exp.op.gt]], [[ui:exp.op.ge]], [[ui:exp.op.contains]],
[[ui:exp.op.matches]], [[ui:exp.op.empty]] und [[ui:exp.op.not_empty]]. Zahlen
werden als Zahlen verglichen, Text exakt. Ein Header, Parameter oder Feld, das fehlt,
ist leer. Ein Vergleich, der sich nicht anstellen lässt — Text gegen eine Zahl —,
ist nicht erfüllt.

### Antworten {#responses}

Eine Route hat eine bis 16 Antworten ([[ui:emu.responses]]).

| Feld | Was | Standard |
| --- | --- | --- |
| [[ui:emu.status]] | 100–599. | 200 |
| [[ui:emu.fault]] | Etwas anderes als eine Antwort; siehe [Fehlverhalten](#faults). | [[ui:emu.fault.none]] |
| [[ui:emu.delay]] | Wie lange vor dem Antworten gewartet wird, 0–60.000 ms. | 0 |
| [[ui:emu.jitter]] | Bis zu so viel länger, zufällig, 0–60.000 ms. | 0 |
| [[ui:emu.weight]] | Ihr Anteil, wenn die Route zufällig antwortet. Nur dann angezeigt. | 1 |
| [[ui:emu.headers]] | Bis zu 32. Namen können Parameter verwenden; Werte sind [Vorlagen](#templates). | keine |
| [[ui:emu.body]] | Eine [Vorlage](#templates), bis zu 256 KiB, wie geschrieben. | leer |

Ohne `Content-Type`-Header geht ein Body, der gültiges JSON ist, als
`application/json` hinaus, jeder andere als `text/plain; charset=utf-8`.

Bei zwei oder mehr Antworten legt [[ui:emu.order]] fest, welche eine Anfrage erhält:

| [[ui:emu.order]] | Anfragen erhalten | Wofür |
| --- | --- | --- |
| [[ui:emu.order.sequence]] | Die erste, die zweite, …, danach immer die letzte: 500, 500, 200, 200, 200… | Neuversuche: zweimal scheitern, dann funktionieren. |
| [[ui:emu.order.cycle]] | Nach der letzten wieder die erste: 200, 500, 200, 500… | Eine Abhängigkeit, die regelmäßig ab und zu ausfällt. |
| [[ui:emu.order.random]] | Jede nach ihrem Gewicht gezogen. Gewichte 8 und 2 geben der ersten etwa 80 % der Fälle. Mindestens ein Gewicht muss über 0 liegen. | Ein realistischer Anteil an Fehlern. |

[[ui:emu.preset]] fügt der Route eine fertige Antwort hinzu:

| Vorgabe | Fügt hinzu |
| --- | --- |
| [[ui:emu.preset.ok]] | 200, `{"ok":true}` |
| [[ui:emu.preset.created]] | 201, `{"id":"{{uuid}}"}`, Header `Location: {{request.path}}/{{counter}}` |
| [[ui:emu.preset.notFound]] | 404, `{"error":"not found"}` |
| [[ui:emu.preset.error]] | 500, `{"error":"internal"}` |
| [[ui:emu.preset.unavailable]] | 503, `{"error":"unavailable"}`, Header `Retry-After: 1` |
| [[ui:emu.preset.slow]] | 200, `{"ok":true}` nach 2000 ms |
| [[ui:emu.preset.timeout]] | Das Fehlverhalten [[ui:emu.fault.timeout]] |
| [[ui:emu.preset.reset]] | Das Fehlverhalten [[ui:emu.fault.reset]] |
| [[ui:emu.preset.malformed]] | 200, `{"items":[{"id":1},{"id":2}]}` mit dem Fehlverhalten [[ui:emu.fault.malformed]] |

### Fehlverhalten {#faults}

| [[ui:emu.fault]] | Worauf der Client trifft |
| --- | --- |
| [[ui:emu.fault.none]] | Die Antwort. |
| [[ui:emu.fault.timeout]] | Nichts. Die Anfrage wird bis zu 2 Minuten festgehalten, dann wird die Verbindung geschlossen — so wird das eigene Timeout des Clients getestet. Die Verzögerung gilt nicht. |
| [[ui:emu.fault.reset]] | Die Verbindung schließt sich ohne Antwort, nach der Verzögerung. |
| [[ui:emu.fault.malformed]] | Eine vollständige HTTP-Antwort mit dem eingestellten Status und den eingestellten Headern, deren Body auf halber Strecke abbricht: JSON, das sich nicht parsen lässt. War der ganze Body JSON, lautet der Content-Type trotzdem `application/json`. |

### Anfragen, die keine Route annimmt {#fallback}

[[ui:emu.fallback]] entscheidet, was eine Anfrage erhält, auf die keine Route passt:

- [[ui:emu.fallbackDefault]] — 404 mit dem Body `{"error":"no_route"}`;
- [[ui:emu.fallbackCustom]] — eine Antwort, die Sie festlegen, mit allem, was die Antwort einer
  Route hat. Ihr `{{counter}}` zählt die Anfragen, die keine Route angenommen hat.

So oder so zählt die Anfrage als [[ui:emu.unmatched]].

### Was eine HTTP-Antwort lesen kann {#http-request}

| Vorlage | Ist |
| --- | --- |
| `{{request.method}}` | `GET`, `POST`, … |
| `{{request.path}}` | Der Pfad, ohne Query. |
| `{{request.params.id}}` | Das Pfadsegment namens `:id`. |
| `{{request.query.page}}` | Ein Query-Parameter, dekodiert. |
| `{{request.headers.x-key}}` | Ein Header; Namen in Kleinbuchstaben. |
| `{{request.body}}` | Der Body als Text: seine ersten 64 KiB. |
| `{{request.json.name}}` | Ein Feld eines JSON-Bodys, wenn der Body JSON ist und höchstens 64 KiB groß. |
| `{{request.from}}` | `IP:port` des Clients. |

Ein Anfrage-Body über 1 MiB erhält 413 und wird als [[ui:emu.failed]] gezählt.
Lässt sich eine Antwort nicht erzeugen — eine Vorlage nennt etwas, das die Anfrage nicht
hat —, gibt es 500 mit dem Fehler im Body, und sie wird als
[[ui:emu.failed]] gezählt.

## OSC-Gerät {#osc}

Jede eintreffende Nachricht — jede Nachricht eines Bundles einzeln — beantwortet
die erste Regel, auf die sie passt. Ein Datagramm, das kein OSC ist, wird als
[[ui:emu.unmatched]] gezählt.

| Feld | Was |
| --- | --- |
| [[ui:emu.address]] | Ein Adressmuster nach OSC 1.0: `*` beliebige Zeichen, `?` ein Zeichen, `[a-z]` eine Menge, `{a,b}` eines von beiden, jeweils innerhalb eines Segments (siehe [OSC](../protocols/osc.md#patterns)). |
| [[ui:exp.argRules]] | Bis zu 16 Bedingungen an die Argumente, wie bei [[ui:exp.node.wait_osc]] (siehe [Knoten](../experiments/nodes.md#node-wait_osc)). |
| [[ui:emu.replyOn]] | Aus: die Nachricht annehmen und nichts antworten. |
| [[ui:emu.replyAddress]] | Die Adresse der Antwort, eine [Vorlage](#templates). |
| [[ui:emu.replyArgs]] | Bis zu 16 Argumente, jedes mit [[ui:emu.argType]] (`int`, `float`, `str`, `long`, `double`, `bool`, `blob`, `nil`) und einer Vorlage als [[ui:emu.argValue]]. |
| [[ui:emu.to]] | Leer: zurück an Adresse und Port des Absenders. Sonst `IP:port`. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | Jeweils 0–60.000 ms. |

Der Wert eines Arguments wird nach dem Füllen der Vorlage als sein Typ gelesen:
`{{request.args[0]}}` gibt das erste Argument als Zahl zurück, wenn der Typ eine
Zahl ist. Ein `bool` nimmt `true`, `1`, `yes`, `on` oder `false`, `0`, `no`, `off` an;
ein `blob` nimmt Hex-Bytes an; ein leerer Wert ist die Null des Typs.

Antworten gehen vom eigenen Port des Emulators aus, sodass ein Client, der auf dem
Port empfängt, von dem er gesendet hat, sie hört.

Eine OSC-Antwort kann `{{request.address}}`, `{{request.args[0]}}` und
`{{request.from}}` lesen.

## UDP-Gerät {#udp}

Jedes Datagramm beantwortet die erste Regel, auf die es passt.

| Feld | Was |
| --- | --- |
| [[ui:emu.match]] | [[ui:exp.mode.any]], [[ui:exp.mode.contains]], [[ui:exp.mode.regex]] oder [[ui:exp.mode.hex]]. |
| [[ui:emu.pattern]] | Der Text, der reguläre Ausdruck oder die Bytes, nach denen gesucht wird. |
| [[ui:emu.reply]] | [[ui:emu.replyOff]], [[ui:emu.replyText]] oder [[ui:emu.replyHex]], dann die Antwort selbst als [Vorlage](#templates). |
| [[ui:emu.to]] | Leer: zurück an den Absender. Sonst `IP:port`. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | Jeweils 0–60.000 ms. |

Eine UDP- oder TCP-Antwort kann lesen:

| Vorlage | Ist |
| --- | --- |
| `{{request.text}}` | Die Nutzdaten als Text. |
| `{{request.match}}` | Was gepasst hat: der Text, die erste Gruppe eines regulären Ausdrucks (oder der ganze Treffer), die Bytes. |
| `{{request.hex}}` | Die Nutzdaten als Hex-Bytes, die ersten 1024. |
| `{{request.bytes}}` | Die Größe der Nutzdaten. |
| `{{request.from}}` | `IP:port` des Absenders. |

Eine Textantwort umfasst höchstens 65.507 Bytes.

## TCP-Gerät {#tcp}

Ein Gerät, das zeilenweise über eine TCP-Verbindung spricht, wie ein Projektor oder eine
Kreuzschiene. Jede Nachricht, die ein Client sendet, beantwortet die erste Regel, auf die sie
passt; die Antwort geht über dieselbe Verbindung zurück.

| Feld | Was |
| --- | --- |
| [[ui:emu.delimiter]] | Was eine Nachricht beendet und nach jeder Antwort und der Begrüßung angehängt wird: [[ui:emu.delimiter.lf]] (ein `\r` davor wird verworfen), [[ui:emu.delimiter.crlf]], [[ui:emu.delimiter.cr]] oder [[ui:emu.delimiter.none]]. Leere Zeilen werden übersprungen. |
| [[ui:emu.greeting]] | Wird gesendet, sobald sich ein Client verbindet; leer für keine. Kann `{{request.from}}` lesen. |
| [[ui:emu.match]], [[ui:emu.pattern]], [[ui:emu.reply]] | Wie bei einem [UDP-Gerät](#udp). |
| [[ui:emu.close]] | Die Verbindung nach der Antwort dieser Regel schließen — etwa bei `QUIT`. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | Jeweils 0–60.000 ms. |

Eine Nachricht, die ohne ihr Endezeichen länger als 64 KiB wird, wird so angenommen, wie sie ist.

## MQTT-Broker {#mqtt}

Ein kleiner MQTT-3.1.1-Broker über einfaches TCP. Er tut, was ein Broker tut: Clients
verbinden sich, abonnieren mit `+` und `#`, veröffentlichen mit QoS 0, 1 und 2, Retained-Nachrichten
und Last Wills funktionieren, und eine zweite Verbindung mit der ID eines Clients übernimmt
von der ersten. Sitzungen sind immer sauber (Clean Session): Ein Client, der seine
Sitzung behalten möchte, erhält eine neue, und für einen abwesenden Client wird nichts zwischengespeichert.

Darüber hinaus wird jede an ihn veröffentlichte Nachricht gegen die Regeln geprüft:
Die erste, die passt, veröffentlicht zusätzlich eine Antwort — ein Gerät, das meldet, was es getan hat.

| Feld | Was |
| --- | --- |
| [[ui:emu.username]], [[ui:emu.password]] | Ist ein Benutzername gesetzt, muss sich ein Client damit und mit dem Passwort verbinden; leer: Jeder darf sich verbinden. Ein Passwort ohne Benutzernamen wird abgelehnt, da MQTT 3.1.1 keines übertragen kann. |
| [[ui:emu.retained]] | Bis zu 64 Nachrichten ([[ui:emu.topic]], [[ui:emu.payload]], [[ui:emu.qos]]), die von Beginn an gehalten werden, als wären sie mit Retain veröffentlicht: Ein Client, der abonniert, erhält sie zuerst. |
| [[ui:emu.topicFilter]] | Welche Topics eine Regel annimmt: `+` eine Ebene, `#` der Rest — `lab/+/set`. |
| [[ui:emu.match]], [[ui:emu.pattern]] | Eine Bedingung an die Nutzdaten, wie bei einem [UDP-Gerät](#udp). |
| [[ui:emu.replyOn]] | Aus: die Nachricht annehmen und nichts weiter veröffentlichen. |
| [[ui:emu.replyTopic]], [[ui:emu.replyPayload]] | [Vorlagen](#templates). Das Topic darf kein `+` oder `#` enthalten. |
| [[ui:emu.qos]], [[ui:emu.retain]] | Der Antwort. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | Jeweils 0–60.000 ms. |

Eine MQTT-Antwort kann `{{request.topic}}`, `{{request.levels[1]}}` (die
Ebenen des Topics, ab 0), `{{request.payload}}`, `{{request.json.state}}`,
`{{request.match}}`, `{{request.qos}}`, `{{request.retain}}`,
`{{request.client}}` (die Client-ID) und `{{request.from}}` lesen.

## Vorlagen in Antworten {#templates}

Antworten werden in derselben [Vorlagensprache](../experiments/data.md#templates)
geschrieben wie Experimente, sodass ein Feld hier und dort dasselbe bedeutet. Eine Antwort kann
lesen:

- `request` — was eingetroffen ist, wie oben für jedes Protokoll aufgeführt;
- `{{counter}}` — wie viele Nachrichten diese Regel seit dem Start des Emulators
  angenommen hat, diese eingeschlossen;
- die [Generatoren](../experiments/data.md#generators) — `{{uuid}}`,
  `{{now.iso}}`, Zufallswerte und der Rest; zufällige werden aus dem
  Startwert des Emulators gezogen;
- Parameter, wenn der Emulator in einem Experiment läuft oder mit
  `signallab emulate --param` gestartet wird.

Eine Antwort liest nie Geheimnisse, und ein unbekannter Name ist ein Fehler, kein leerer
Text.

Manche Felder werden beim Start des Emulators festgelegt, bevor etwas eintrifft: ein
Pfad, eine Bedingung, ein Adressmuster, ein Muster für Nutzdaten, ein Topic-Filter,
[[ui:emu.to]], der Name eines Headers, Retained-Nachrichten und die Anmeldung am Broker. Sie
nehmen nur Text und Parameter an, kein `request` und keine Generatoren.

Der Startwert bestimmt die zufällige Reihenfolge der Antworten, den Jitter und die Zufallsgeneratoren.
In der Ansicht [[ui:nav.emulators]] nimmt jeder Start einen neuen Startwert; ein
Experiment verwendet den Startwert des Durchlaufs, und `signallab emulate --seed` nimmt einen, den Sie
vorgeben.

## Ausfälle {#outage}

Um zu testen, was Ihr System tut, wenn eine Abhängigkeit immer wieder wegbricht, haken Sie
[[ui:emu.outage]] an:

| Feld | Was | Standard |
| --- | --- | --- |
| [[ui:emu.outageUp]] | Wie lange er antwortet, 10–3.600.000 ms. | 10.000 |
| [[ui:emu.outageDown]] | Wie lange er ausfällt, 10–3.600.000 ms. | 3000 |
| [[ui:emu.outageFault]] | Nur HTTP: worauf eine Anfrage während des Ausfalls trifft. | [[ui:emu.outageFault.unavailable]] |

Der Zeitplan beginnt mit dem Start des Emulators und wiederholt sich: an, aus, an,
aus… Während des Ausfalls:

| Emulator | Worauf der Client trifft |
| --- | --- |
| HTTP | [[ui:emu.outageFault.unavailable]]: 503 mit `Retry-After`, gesetzt auf die Sekunden bis zur Rückkehr (mindestens 1). [[ui:emu.outageFault.reset]]: Die Verbindung schließt sich ohne Antwort. [[ui:emu.outageFault.timeout]]: bis zu 2 Minuten festgehalten, dann geschlossen. |
| TCP-Gerät | Offene Verbindungen werden innerhalb von 0,1 s getrennt; neue werden geschlossen, sobald sie eintreffen. |
| MQTT-Broker | Jede Verbindung wird getrennt; neue werden abgelehnt (CONNACK-Rückgabecode 3, Server nicht verfügbar). |
| OSC-, UDP-Gerät | Nichts wird beantwortet. |

Was während des Ausfalls eintrifft, zählt als [[ui:emu.down]], nicht als
[[ui:emu.unmatched]], und seine Regeln werden nicht befragt.

[[ui:emu.takeDown]] tut dasselbe auf Abruf, unabhängig vom Zeitplan, bis
Sie [[ui:emu.bringUp]] drücken; HTTP erhält dann 503 ohne `Retry-After`. In einem
Experiment erledigt das der Knoten [[ui:exp.node.emulator_state]] an einem Schritt des
Durchlaufs (siehe [Knoten](../experiments/nodes.md#node-emulator_state) und
[Störungen](../experiments/faults.md)).

## Probleme {#problems}

Während Sie bearbeiten, wird der Emulator kurz nach jeder Änderung geprüft, und ein
Problem erscheint unter seinen Schaltflächen, bevor Sie [[ui:emu.start]] drücken. Ein Problem
nennt, wo es liegt — die Regel, die Antwort oder die Retained-Nachricht und das
Feld — und was falsch ist: ein Pfad ohne sein `/`, ein regulärer Ausdruck, der sich nicht
kompilieren lässt, eine Antwortvorlage, die etwas anderes nennt als `request`,
Parameter und Generatoren, ein Wert außerhalb des Bereichs. [[ui:emu.start]] lehnt einen Emulator mit einem Problem ab.

## Grenzen {#limits}

| Was | Grenze | An der Grenze |
| --- | --- | --- |
| Routen oder Regeln pro Emulator | 64 | Bei der Prüfung abgelehnt. |
| Antworten pro Route | 16 | Abgelehnt. |
| Bedingungen pro Route | 16 | Abgelehnt. |
| Header pro Antwort | 32 | Abgelehnt. |
| Argumentbedingungen, Antwortargumente (OSC) | je 16 | Abgelehnt. |
| Retained-Nachrichten (MQTT) | 64 | Abgelehnt. |
| Ein Body, eine Antwort oder eine Begrüßung, wie geschrieben | 256 KiB | Abgelehnt. |
| Eine Verzögerung oder ein Jitter | 60.000 ms | Abgelehnt. |
| HTTP-Anfrage-Body | 1 MiB | 413. |
| HTTP-Verbindungen gleichzeitig | 512 | Weitere werden geschlossen, sobald sie eintreffen. |
| HTTP-Anfragekopf | 30 s | Ein Client muss ihn innerhalb dieser Zeit senden. |
| TCP-Verbindungen gleichzeitig | 256 | Weitere werden geschlossen, sobald sie eintreffen. |
| OSC- und UDP-Antworten, die auf ihre Verzögerung warten | 1024 | Weitere werden verworfen und als [[ui:emu.failed]] gezählt. |
| MQTT-Clients gleichzeitig | 256 | Weitere werden geschlossen, sobald sie eintreffen. |
| MQTT-Paket | 256 KiB | Die Verbindung des Clients endet. |
| MQTT-Abonnements pro Client | 100 | Weitere werden abgelehnt. |
| MQTT-Retained-Topics | 1000 Topics, 16 MiB | Eine neue Retained-Nachricht wird weitergeleitet, aber nicht gehalten. |
| MQTT-Nachrichten, die auf einen langsamen Client warten | 1024 Nachrichten, 8 MiB | Er verpasst sie; gezählt als [[ui:emu.missed]]. |

## Eine Antwort nachbilden {#mock-this}

Um aus einer Antwort, die funktioniert hat, einen Emulator zu machen:

1. Senden Sie in der Ansicht [[ui:nav.http]] eine Anfrage und erhalten Sie eine Antwort — oder verwenden Sie
   [[ui:exp.sendNow]] an einem HTTP-Knoten eines Experiments.
2. Drücken Sie ⧉ [[ui:http.mockThis]] neben der Antwort. Der
   Dialog [[ui:http.mockTitle]] zeigt die Route, die angelegt wird.
3. Wählen Sie unter [[ui:http.mockInto]] einen Ihrer HTTP-Emulatoren oder
   [[ui:http.mockNew]].
4. Drücken Sie [[ui:http.mockAdd]]. Die Ansicht [[ui:nav.emulators]] öffnet sich bei diesem
   Emulator.

Die Route beantwortet Methode und Pfad der Anfrage (ohne Query) mit Status,
Headern und Body der Antwort. Header, die zu diesem einen Austausch gehören
(`Content-Length`, `Date`, `Server`, `ETag` und dergleichen), werden weggelassen, und der
Body wird gesendet, wie er war, selbst wenn er `{{` enthält. Ein neuer Emulator enthält nur diese
Route. Wird sie einem bestehenden Emulator hinzugefügt, kommt die Route an die erste Stelle, sodass sie
vor einer allgemeineren Route antwortet; ein laufender Emulator übernimmt sie, wenn Sie
[[ui:emu.restart]] drücken.

Aus einem Experiment wird eine mit Vorlagen geschriebene URL zu einem Muster: Ihre Basis
(`{{api}}`) entfällt, ein Segment, das aus genau einer Vorlage besteht (`/orders/{{order_id}}`),
wird zu `:order_id`, und ein nur teilweise aus Vorlagen bestehendes Segment beendet den Pfad mit
`*`.

## Die Beispiel-Emulatoren {#starter-set}

Findet Signal Lab zum ersten Mal keine Emulator-Bibliothek, legt es fünf an, alle auf
diesem Computer. Ihre Namen und Notizen werden in der Sprache geschrieben, die die
Oberfläche in diesem Moment hat.

| Emulator | Empfängt auf | Tut |
| --- | --- | --- |
| [[ui:seed.emu.demo-api.name]] | `127.0.0.1:8080` | `GET /health` → `{"status":"ok","time":…}`; `GET /users/:id` → ein Benutzer mit dieser ID; `POST /users` → 201 mit einem `Location`-Header; `GET /slow` → nach 1500 ms; `/flaky` → 503, 503, dann ab da 200. |
| [[ui:seed.emu.osc-device.name]] | `127.0.0.1:9100` | `/ping` → `/pong` mit dem Zähler; `/fader/*` → `/ack` mit der empfangenen Adresse; `/cue/*` wird ohne Antwort angenommen. |
| [[ui:seed.emu.udp-device.name]] | `127.0.0.1:7100` | `PING` → `PONG` und der Zähler; alles andere → `ACK` und seine Größe in Bytes. |
| [[ui:seed.emu.tcp-device.name]] | `127.0.0.1:7200` | Zeilen, die auf CR LF enden. Begrüßt mit `READY`; `POWER?` → `POWER=ON`; `POWER ON` oder `POWER OFF` → `OK ON` / `OK OFF`; `QUIT` → `BYE`, dann legt es auf. |
| [[ui:seed.emu.mqtt-broker.name]] | `127.0.0.1:1883` | Hält `online` auf `lab/status` als Retained-Nachricht; `ON` oder `OFF`, an `lab/<name>/set` veröffentlicht → dasselbe, retained, auf `lab/<name>/state`. |

Das Beispielsignal [[ui:seed.http-reachable.name]] der
[Signalbibliothek](signals.md#starter-set) fragt `http://127.0.0.1:8080/` an, die
Adresse von [[ui:seed.emu.demo-api.name]]: Dort gibt es keine Route für `/`, daher erhält es
404.

## Die Bibliotheksdatei {#file}

Die Bibliothek ist `emulators.json` im Datenordner (siehe
[Dateien](../reference/files.md)); fahren Sie mit der Maus über die Anzahl unter der Liste, um ihren
Pfad zu sehen. Sie wird 0,7 s nach der letzten Änderung vollständig geschrieben, über eine temporäre
Datei, sodass ein fehlgeschlagener Schreibvorgang die vorherige Fassung hinterlässt. Lässt sich die Datei nicht lesen,
zeigt die Liste den Fehler mit Pfad, Zeile und Spalte, und die Datei bleibt,
wie sie ist: Korrigieren Sie sie und drücken Sie [[ui:emu.reload]]. Drücken Sie [[ui:emu.reload]] auch, nachdem
Sie sie von Hand bearbeitet haben. Fehlt die Datei, werden die Beispiel-Emulatoren erneut angelegt.

```json
{
  "version": 1,
  "emulators": [
    {
      "id": "orders-api",
      "note": "Stands in for the orders service.",
      "emulator": {
        "name": "Orders API",
        "bind": "127.0.0.1:18080",
        "protocol": "http",
        "routes": [
          { "method": "GET", "path": "/orders/:id",
            "responses": [{ "body": "{\"id\":\"{{request.params.id}}\",\"state\":\"open\"}" }] },
          { "method": "POST", "path": "/orders", "order": "sequence",
            "responses": [{ "status": 503 }, { "status": 201, "body": "{\"id\":\"{{uuid}}\"}" }] }
        ],
        "outage": { "up_ms": 20000, "down_ms": 2000, "fault": "unavailable" }
      }
    }
  ]
}
```

Das Objekt `emulator` allein ist ein Dokument, das auch `signallab emulate` liest.

## In Experimenten und Skripten {#elsewhere}

- In einem Experiment öffnet ein Knoten [[ui:exp.node.emulator]] seinen Emulator vor
  dem ersten Schritt und antwortet, bis der Durchlauf endet; was er empfangen hat, wird im
  Bericht gezählt. Ein HTTP-Emulator dort ist auch das, worauf
  [[ui:exp.node.wait_http]] ([Knoten](../experiments/nodes.md#node-wait_http))
  hört, und ein OSC- oder UDP-Emulator teilt seinen
  Port mit den Warte-Knoten des Durchlaufs. Zwei Emulatoren desselben Transports in einem Experiment
  können sich keinen Port teilen. Siehe [Knoten](../experiments/nodes.md#node-emulator) und
  [Störungen](../experiments/faults.md).
- `signallab emulate` führt Emulatoren aus Dateien oder aus dieser Bibliothek aus, bis
  <kbd>Ctrl</kbd>+<kbd>C</kbd> oder `--for` sie beendet, und gibt aus, was sie antworten; siehe
  [Die Kommandozeile](../automation/cli.md#cli-emulate).

## Verwandte Seiten {#related}

- [Inspektor](inspector.md) — jeder Austausch, dekodiert.
- [Störung](impairment.md) — ein schlechtes Netzwerk zwischen Ihrem System und einem
  Emulator.
- [Daten und Vorlagen](../experiments/data.md#templates)

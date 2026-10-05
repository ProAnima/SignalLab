---
title: Inspektor
description: Jeden Frame aufzeichnen, den die Werkzeuge von Signal Lab senden und empfangen, filtern und ihn dekodiert und als Bytes lesen, exportieren und einen als Signal aufbewahren.
---

# Inspektor

Der Inspektor ist eine einzige Zeitleiste für jedes Werkzeug: Jede OSC-Nachricht, jedes
Datagramm, jeder HTTP-Austausch, jede MQTT-Veröffentlichung, jede WebSocket-Nachricht, jedes
weitergeleitete Paket und jeder Emulator-Austausch landet dort, dekodiert, mit den Bytes, aus
denen er bestand. Nutzen Sie ihn, um zu sehen, was tatsächlich über die Leitung ging, in
welcher Reihenfolge und was damit geschah.

Er lebt im unteren Bereich, als Registerkarte [[ui:dock.inspector]] neben der Konsole, sodass
er auf jeder Ansicht da ist. Klicken Sie die Registerkarte an, um ihn zu öffnen; der Bereich
öffnet sich hoch genug für ein paar Zeilen und die Details eines Frames. Die Schaltfläche ⤢
([[ui:dock.maximise]]) macht den Bereich so hoch wie das Fenster. Der Inspektor behält seine
Liste und Auswahl, während Sie den Bereich schließen oder die Ansicht wechseln.

## Mitschnitt {#capture}

Der Mitschnitt ist aus, wenn Signal Lab startet, und solange er aus ist, kostet er nichts:
Die Werkzeuge bauen nicht einmal Frames.

1. Öffnen Sie die Registerkarte [[ui:dock.inspector]].
2. Drücken Sie [[ui:ins.arm]]. Der Punkt auf der Registerkarte wird rot und pulsiert.
3. Nutzen Sie ein beliebiges Werkzeug. Frames erscheinen oben in der Liste, die neuesten
   zuerst.
4. Drücken Sie [[ui:ins.disarm]], wenn Sie haben, was Sie brauchen.

Die Registerkarte zeigt, wie viele Frames mitgeschnitten wurden, von jeder Ansicht aus.

Frames werden ab dem Moment mitgeschnitten, in dem Sie den Mitschnitt starten, nie vorher:
Starten Sie ihn zuerst, dann senden Sie.

Auf einem [Server](../server/index.md) gehört der Mitschnitt dem Server: Jede bei ihm
angemeldete Seite sieht dieselben Frames, und ihn auf einer Seite zu starten oder zu leeren
tut es für alle.

### Was mitgeschnitten wird {#sources}

| Werkzeug | Frames | Wie viele |
| --- | --- | --- |
| [OSC](../protocols/osc.md): Senden | Jede gesendete Nachricht | Jede |
| [OSC](../protocols/osc.md): Monitor | Jedes empfangene Paket; eines, das sich nicht dekodieren lässt, wird mit dem Dekodierfehler markiert | Jedes |
| [OSC](../protocols/osc.md): Signalgenerator | Gesendete Nachrichten | Höchstens eine pro 100 ms, markiert mit `sampled` |
| [Broadcast](../protocols/broadcast.md): einmal senden | Jedes Datagramm, eines pro Ziel; ein fehlgeschlagener Sendevorgang mit seinem Fehler | Jedes |
| [Broadcast](../protocols/broadcast.md): Beacon | Gesendete Datagramme | Höchstens eines pro 50 ms |
| [Broadcast](../protocols/broadcast.md): Erkennungs-Listener | Empfangene Sonden | Höchstens eine pro 40 ms |
| [Broadcast](../protocols/broadcast.md): Erkennungs-Listener | Seine Antworten (`auto-reply`) | Jede |
| [HTTP](../protocols/http.md): Senden, Signale, Experiment-Anfragen | Jeder Austausch: die Anfragezeile, der Status und die Zeit, die Antwort-Header und der Anfang des Bodys | Jeder |
| [HTTP](../protocols/http.md): Last-Burst und Anfragen unter Last | Austausche | Höchstens einer pro 100 ms |
| [MQTT](../protocols/mqtt.md): Verbindung | Gesendete Veröffentlichungen | Jede |
| [MQTT](../protocols/mqtt.md): Verbindung | Empfangene Nachrichten | Höchstens eine pro 200 ms |
| [MQTT](../protocols/mqtt.md): ein MQTT-Signal, gesendet, während die Ansicht nicht mit seinem Broker verbunden ist | Die Veröffentlichung | Jede |
| [WebSocket](../protocols/websocket.md) | Gesendete und empfangene Nachrichten | Jede, solange der Verkehr leicht ist; höchstens 200 pro Sekunde |
| [Emulatoren](emulators.md) | Was eintrifft und die Antwort, zusammen | Höchstens ein Austausch pro 10 ms |
| [Störung](impairment.md) | Jedes weitergeleitete Datagramm oder jeder Block, in beide Richtungen, mit seinem Schicksal | Höchstens eines pro 25 ms für beide Richtungen zusammen |
| [Sturm](storm.md) (UDP) | Flutpakete, alle gleich | Eines pro Sekunde, markiert mit `sampled 1/s`; ein TCP-Sturm schneidet keines mit |
| [Scanner](scanner.md) | Jeder offene Port, mit seinem Banner | Jeder |
| [Experimente](../experiments/index.md) | Was die Schritte eines Durchlaufs senden (bei einer TCP-Nachricht: die geschriebenen Nutzdaten und die gelesene Antwort) und was seine Warte-Knoten empfangen | Wie das Werkzeug, das es nutzt |

Ein Werkzeug, das Stichproben zieht, lässt den Rest absichtlich aus und zählt sie: Der
nächste Frame, den es doch darstellt, sagt in seinem Befund, wie viele es zurückhielt, als
`+n not shown` (`sampled · +5 not shown`). Die Zahl betrifft das, was das Werkzeug dargestellt
hätte, nicht das eigene Verwerfen des Mitschnitts (siehe [Zahlen und Lücken](#counts)).

## Die Frame-Liste {#list}

| Spalte | Was |
| --- | --- |
| [[ui:common.time]] | Wann er mitgeschnitten wurde, auf die Millisekunde. |
| [[ui:ins.dir]] | → gesendet (`tx`), ← empfangen (`rx`). Bei einem Relais ist → Client zum Ziel und ← Ziel zum Client. |
| [[ui:bc.proto]] | `osc`, `udp`, `tcp`, `http`, `mqtt` oder `ws`. |
| [[ui:bc.peer]] | Die Gegenseite: ein `IP:port`, eine URL, ein Broker. |
| [[ui:common.bytes]] | Die Größe des Frames. |
| [[ui:ins.summary]] | Eine Zeile in der eigenen Notation des Protokolls, etwa `/fader/1 0.75` oder `GET http://127.0.0.1:8080/ → 200 in 3ms`. |
| [[ui:ins.verdict]] | Was damit geschah, wenn es etwas zu sagen gibt. |

Der Befund ist grün, gelb oder rot. Rot ist ein Verlust oder ein Fehlschlag (`dropped (loss)`,
`failed`, `error: …`); gelb ist ein veränderter Frame oder nur eine Stichprobe von vielen
(`corrupted`, `copy 2/2`, `sampled`, `+n not shown`); grün ist der Rest. Manche Befunde, auf
die Sie treffen werden:

| Befund | Von | Bedeutet |
| --- | --- | --- |
| `forwarded +42ms` | Störung | Nach dieser Verzögerung weitergegeben; `· corrupted`, `· reordered` oder `· copy 1/2` können folgen. |
| `dropped (loss)`, `dropped (burst)`, `dropped (offline)` | Störung | Absichtlich verloren, und warum. |
| `throttled` | Störung | Von der Bandbreitengrenze verworfen. |
| `· client→target`, `· target→client` | Störung | Beendet den Befund jedes weitergeleiteten Frames, vor jedem `+n not shown`: in welche Richtung er unterwegs war. |
| `#2 → 200 OK · 37 B`, `— → 404 …` | Emulatoren | Welche Regel geantwortet hat (`—`: keine) und die Antwort. |
| `down`, `down → 503` | Emulatoren | Er traf ein, während der Emulator ausgefallen war. |
| `200 OK`, `failed` | HTTP | Der Antwortstatus oder gar keine Antwort. |
| `open` | Scanner | Ein offener Port. |
| `auto-reply` | Erkennung | Eine Antwort, die der Listener auf eine Sonde gesendet hat. |
| `clears retained` | MQTT | Eine leere Retained-Veröffentlichung. |
| `+n not shown` | Jedes Werkzeug, das Stichproben zieht | So viele Frames seit dem vorherigen wurden ausgelassen; es folgt dem anderen Befund des Frames nach einem `·`. |

Die Liste behält die neuesten 4000 Frames und stellt die neuesten 300 dar, die zu den Filtern
passen; unter den Filtern sagt sie, wie viele sie von wie vielen passenden anzeigt.

### Filtern {#filter}

- Tippen Sie in das Textfeld ([[ui:ins.filterPlaceholder]]), um Frames zu behalten, deren
  Inhalt, Gegenstelle, Quelle, Protokoll oder Befund den Text enthält.
- Klicken Sie auf Protokoll-Chips (`osc`, `udp`, `tcp`, `http`, `mqtt`, `ws`), um nur diese
  Protokolle anzuzeigen. Ist kein Chip an, wird jedes Protokoll angezeigt.
- Klicken Sie auf `tx` oder `rx`, um nur gesendete oder nur empfangene Frames anzuzeigen.
- [[ui:common.reset]] löscht alle drei.

Filter ändern nur, was die Liste zeigt. Mitschnitt, die Zahlen und ein Export umfassen immer
alles.

### Anhalten und leeren {#pause}

[[ui:ins.pause]] friert die Liste ein, damit Sie sie lesen können, während der Verkehr
weiterläuft; der Mitschnitt läuft weiter. [[ui:ins.resume]] lässt wieder neue Frames herein.
Frames, die eintrafen, während die Ansicht angehalten war, werden nicht in die Liste
aufgenommen, sind aber im Mitschnitt und in einem Export.

[[ui:common.clear]] leert die Liste und den Mitschnitt und setzt seine Zahlen zurück.

### Zahlen und Lücken {#counts}

Die Leiste oben zählt die mitgeschnittenen Frames und ihre Bytes sowie, wie voll der
Mitschnitt ist (gehaltene Frames von 8192).

Wenn Frames schneller eintreffen, als die Liste sie aufnehmen kann — mehr als 250 in etwa
einer Achtelsekunde —, überspringt die Liste die ältesten von ihnen. Ein gelber Chip zählt
dann die nicht angezeigten Frames, und eine Zeile in der Liste markiert, wo sie fehlen. Diese
Frames sind noch im Mitschnitt, sofern neuere sie nicht inzwischen hinausgedrängt haben:
Exportieren Sie ihn, um sie zu sehen.

## Die Details eines Frames {#detail}

Klicken Sie auf eine Zeile, um den Frame rechts zu sehen.

| Feld | Was |
| --- | --- |
| [[ui:ins.seq]] | Die Nummer des Frames. Die Nummern steigen in Mitschnittreihenfolge und werden nie wiederverwendet. |
| [[ui:common.time]] | Wann er mitgeschnitten wurde. |
| [[ui:ins.direction]] | Gesendet oder empfangen. |
| [[ui:common.protocol]] | Wie in der Liste. |
| [[ui:ins.source]] | Das Werkzeug, das ihn mitgeschnitten hat (`osc-send`, `osc-monitor`, `netsim`, `emulator`, `experiment-wait`, …), und seine Job-Nummer, wenn es zu einem gehört. |
| [[ui:ins.local]] | Die Adresse auf dieser Seite, wenn es eine gibt. Bei einem weitergeleiteten Frame die Adresse, auf der das Relais empfängt. |
| [[ui:bc.peer]] | Die Gegenseite. Bei einem weitergeleiteten Frame, wohin er ging: das Ziel oder der Client, an den die Antwort zurückging. |
| [[ui:ins.size]] | Seine Größe in Bytes. |
| [[ui:ins.verdict]] | Wie in der Liste. |

Unter [[ui:ins.decoded]] steht der Frame, in seinem Protokoll gelesen: jede Nachricht eines
OSC-Bundles mit ihren Argumenten, die Header einer HTTP-Antwort und der Anfang ihres Bodys,
die Anfrage eines Emulators und seine Antwort.

Unter [[ui:ins.rawBytes]] steht ein Hex-Dump: Offset, 16 Bytes in Hex und dieselben Bytes als
Text. Die Liste enthält das erste KiB jedes Frames; ist ein Frame länger, lädt eine
Schaltfläche unter dem Dump alles davon.

### Was ein Frame behält {#limits}

| Grenze | Wert | An der Grenze |
| --- | --- | --- |
| Bytes, die ein Frame behält | 256 KiB | Ein längerer Frame behält seine ersten 256 KiB und sagt, wie viel des Ganzen er behielt. |
| Frames im Mitschnitt | 8192 | Der älteste Frame macht Platz. |
| Bytes, die der Mitschnitt insgesamt behält | 64 MiB | Die ältesten Frames machen Platz. |

Manche Frames behalten keine Bytes: HTTP-Austausche (ihre Größe wird festgehalten, und die
Antwort-Header und der Anfang des Bodys stehen stattdessen im dekodierten Text) und offene
Ports des Scanners.

Ein MQTT-Frame behält die Nutzdaten der Nachricht, nicht das Protokollpaket darum herum; das
Topic, QoS und das Retain-Flag stehen in seinem Inhalt.

### Geheimnisse {#secrets}

Solange ein Durchlauf oder [[ui:exp.sendNow]] [Geheimnisse](../experiments/data.md#secrets)
verwendet, werden ihre Werte in jedem Frame maskiert, bevor er mitgeschnitten wird: `••••` im
Inhalt, im dekodierten Text, in den Adressen und im Befund, und `*` für jedes Byte in den
Nutzdaten, damit die Offsets im Dump stimmen. Auch die Anmeldedaten der Ansicht HTTP
erscheinen nie: Ein HTTP-Frame enthält die Antwort, nicht den gesendeten `Authorization`-Header.

## Einen Frame als Signal speichern {#save-as-signal}

Um ein Paket aufzubewahren, das Sie gefangen haben, und es später erneut zu senden — wenn das
Gerät, das es gesendet hat, nicht mehr da ist:

1. Wählen Sie den Frame aus.
2. Drücken Sie [[ui:sig.fromFrame]].

Das Signal landet im Ordner [[ui:sig.capturedFolder]] der [Signalbibliothek](signals.md) mit
jedem Byte des Frames, entnommen dem, was der Mitschnitt behielt, nicht dem dekodierten Text.
Es wird nach dem Inhalt des Frames benannt, und seine Notiz sagt, von welchem Frame es kam.

Was Sie erhalten, hängt vom Frame ab:

| Frame | Signal |
| --- | --- |
| Ein OSC- oder UDP-Datagramm | Ein rohes UDP-Signal mit den Bytes des Frames, hex. |
| Eine MQTT-Veröffentlichung — gesendet, empfangen oder von einem Emulator | Ein MQTT-Signal mit Broker, Topic, QoS und Retain-Flag des Frames und seinen Nutzdaten als Text, genau wie sie waren. Leere Nutzdaten bleiben erhalten, sodass das Löschen eines Retained-Werts gespeichert werden kann. |
| Alles andere: ein TCP-Datenstrom (auch einer, den ein Relais trug, oder der des TCP-Knotens), ein HTTP-Austausch, eine WebSocket-Nachricht, ein MQTT-Paket, das keine Veröffentlichung ist (das Subscribe eines Clients an einem Emulator) | Nichts: Die Schaltfläche ist aus, und ihr Hinweis sagt, warum. Ein Signal sendet ein Datagramm oder eine Veröffentlichung; diese lassen sich nicht so, wie sie waren, erneut senden. |

Ein Datagramm wird gesendet an:

- einen empfangenen Frame — die Adresse, die ihn empfangen hat (die Seite [[ui:ins.local]]),
  sodass das Signal für den Absender einspringt;
- einen gesendeten Frame — die Gegenstelle, an die er gesendet wurde;
- einen weitergeleiteten Frame, in beide Richtungen — die Adresse, an die er ging: das Ziel
  für einen vom Client kommenden Frame, der Client für eine Antwort.

Wenn diese Adresse jede Adresse dieses Computers ist — ein Monitor, der auf `0.0.0.0:9000`
oder `[::]:9000` empfängt —, wird das Signal stattdessen auf diesen Computer gerichtet:
`127.0.0.1:9000` oder `[::1]:9000`. Öffnen Sie das Signal und ändern Sie sein Ziel, wenn Sie
eine andere Adresse meinen.

Ein MQTT-Signal geht zu dem Broker, den der Frame nennt. Ein Broker, der auf jeder Adresse
empfängt, wird auf dieselbe Weise unter `127.0.0.1` erreicht.

Die Schaltfläche ist außerdem aus für:

- einen nicht vollständig behaltenen Frame: einen größeren als 256 KiB oder einen, der keine
  Bytes behielt;
- eine MQTT-Nachricht, deren Nutzdaten kein Text sind — die Nutzdaten eines Signals sind
  Text, daher ließen sich ihre Bytes nicht so, wie sie waren, erneut senden;
- einen empfangenen Frame, der keinen Socket auf dieser Seite nennt, sodass es keine Adresse
  zum Senden gibt.

Ein Frame, den der Mitschnitt bereits freigegeben hat, lässt sich ebenfalls nicht speichern;
die Konsole sagt es. Solange die Bibliotheksdatei nicht gelesen werden kann, ist
[[ui:sig.fromFrame]] ebenfalls aus, und der Hinweis zeigt den Fehler der Datei (siehe
[Signale](signals.md#file)).

## Exportieren {#export}

[[ui:ins.exportJsonl]] und [[ui:ins.exportTxt]] schreiben den ganzen Mitschnitt — bis zu 8192
Frames, jedes Byte, das jeder behielt, egal was die Filter zeigen — in eine Datei
`capture-<time>.jsonl` oder `capture-<time>.txt` im Datenordner (siehe
[Dateien](../reference/files.md)). Die Konsole sagt, wo. In einem mit einem Server verbundenen
Browser wird die Datei auf dem Server geschrieben, und Ihr Browser lädt sie herunter.

Ein leerer Mitschnitt wird nicht geschrieben; die Konsole sagt, dass es nichts zu speichern
gibt.

- **`.jsonl`** — ein JSON-Objekt pro Zeile, eine Zeile pro Frame: `seq`, `ts` (Millisekunden
  seit 1970), `proto`, `dir`, `source`, `job_id`, `local`, `remote`, `bytes`, `kept`,
  `summary`, `detail`, `hex` (der Dump des ersten KiB), `verdict` und `data`, die behaltenen
  Bytes als Base64.
- **`.txt`** — zum Lesen: eine Zeile pro Frame mit seiner Nummer, Zeit, Richtung, Protokoll,
  Gegenstelle, Größe und seinem Befund, dann sein Inhalt, sein dekodierter Text und ein
  Hex-Dump jedes Bytes, das er behielt.

```json
{"seq":12,"ts":1767225600123,"proto":"osc","dir":"rx","source":"osc-monitor","job_id":3,"local":"0.0.0.0:9000","remote":"127.0.0.1:53211","bytes":20,"summary":"/fader/1 0.75","detail":"/fader/1 0.75","hex":"0000  2f 66 61 64 65 72 2f 31  00 00 00 00 2c 66 00 00  |/fader/1....,f..|\n0010  3f 40 00 00                                       |?@..|\n","verdict":null,"kept":20,"data":"L2ZhZGVyLzEAAAAALGYAAD9AAAA="}
```

## Von anderswo kommend {#reveal}

Andere Ansichten zeigen auf Frames: Ein Warte-Knoten in der Zeitleiste eines Experiments
verlinkt den Frame, auf den er passte, und die Empfangsliste eines Emulators hat eine
Schaltfläche ⌕ ([[ui:emu.inspectFrame]]) an jedem Austausch. Folgt man einer, öffnet sich der
Inspektor mit diesem Frame ausgewählt, den Filtern geleert und der Ansicht fortgesetzt.

## Verwandte Seiten {#related}

- [Signale](signals.md) — was ein gespeicherter Frame wird.
- [Störung](impairment.md) — die Befunde des Relais.
- [Emulatoren](emulators.md) — was ein Emulator empfangen hat.

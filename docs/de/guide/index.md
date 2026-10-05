---
title: Was ist Signal Lab
description: Wofür Signal Lab gedacht ist, was Sie damit tun können, die zwei Arten, es zu betreiben, und wie es aufgebaut ist.
---

# Was ist Signal Lab

Signal Lab ist ein Testlabor für OSC und Netzwerkprotokolle. Es sendet die Nachrichten, die Ihre Geräte
und Dienste sprechen, zeigt Ihnen, was zurückkommt, vertritt das Gerät oder die API, die es noch
nicht gibt, verschlechtert absichtlich das Netzwerk dazwischen und macht aus all dem
Experimente, die Sie erneut ausführen können — aus der App, einem Skript oder einer CI-Pipeline.

Es wurde gebaut, um eine Installation in Betrieb zu nehmen, bevor es die Installation gibt: Showcontroller,
Medienserver, Sensoren und Cloud-API lassen sich jeweils gegen einen Stellvertreter testen,
auf einem einzigen Laptop, lange bevor sie sich vor Ort begegnen.

## Für wen es gedacht ist {#audience}

- **Wer Showsteuerung, Installationen und vernetzte Geräte in Betrieb nimmt**: Licht-
  und Medienserver, Controller, Sensoren, Projektoren, alles, was OSC, UDP,
  TCP oder MQTT spricht.
- **Wer APIs und Dienste testet**: HTTP- und WebSocket-Endpunkte, ihre Authentifizierung,
  ihr Verhalten unter Last und wenn eine Abhängigkeit ausfällt.
- **Wer beides automatisiert**: Dieselben Experimente laufen ohne Fenster in einer Pipeline, auf einem
  Laborserver neben der Technik oder gesteuert von einem KI-Assistenten.

## Was Sie damit tun können {#what-you-can-do}

### Verkehr senden und beobachten {#send-and-watch}

Jedes Protokoll hat eine eigene Ansicht:

| Protokoll | Was Sie tun können | Seite |
| --- | --- | --- |
| OSC | Nachrichten mit typisierten Argumenten senden, einen Port überwachen, eine Wellenform auf einen Endpunkt geben | [OSC](../protocols/osc.md) |
| Rohes UDP und TCP | Text oder Bytes in Experimenten und aus der Bibliothek senden, Geräte emulieren, die sie sprechen | [UDP und TCP](../protocols/udp-tcp.md) |
| HTTP | Eine Anfrage mit ihrer vollständigen Antwort, Basic-, Bearer- oder Digest-Authentifizierung, ein Cookie-Speicher, ein Last-Burst | [HTTP](../protocols/http.md) |
| WebSocket | Mit den Headern und Subprotokollen verbinden, die ein Dienst erwartet, Text oder Bytes senden, jede Nachricht lesen | [WebSocket](../protocols/websocket.md) |
| MQTT 3.1.1 | Mit einem Broker verbinden, jedes Topic beobachten, das er hält, mit QoS 0, 1 oder 2 veröffentlichen, einen Retained-Wert löschen | [MQTT](../protocols/mqtt.md) |
| Broadcast und Multicast | An eine Liste, eine Broadcast-Adresse, eine Multicast-Gruppe oder jeden Host eines Subnetzes senden; mithören, wer antwortet | [Broadcast und Erkennung](../protocols/broadcast.md) |

### Jeden Frame sehen {#inspect}

Der [Inspektor](../tools/inspector.md) zeichnet jeden Frame auf, den die Werkzeuge senden und empfangen, in
einer Zeitleiste: dekodiert, mit seinen Bytes, filterbar und exportierbar. Ein Frame, den er erfasst hat, kann
zu einem Signal werden, das ihn Byte für Byte erneut sendet.

### Behalten, was funktioniert {#library}

Eine Nachricht, die funktioniert hat, kommt in die [Signalbibliothek](../tools/signals.md): mit Namen, in Ordnern
abgelegt, mit einer Notiz, was sie bewirken soll. Sie senden sie erneut aus ihrer
Ansicht, aus der Bibliothek oder von überall mit <kbd>Ctrl</kbd>+<kbd>K</kbd>.

### Die Gegenseite übernehmen {#emulate}

[Emulatoren](../tools/emulators.md) antworten als die API, das Gerät oder der Dienst, mit dem Ihr System
spricht: eine HTTP-API mit Routen und Antworten, ein OSC-, UDP- oder TCP-Gerät mit Regeln, ein MQTT-Broker.
Sie können langsam antworten, in einer festgelegten Abfolge scheitern, fehlerhafte Bodys senden oder nach
Zeitplan ausfallen — so testen Sie, wie Ihr System damit zurechtkommt, bevor es das Original gibt.

### Das Netzwerk absichtlich stören {#impair}

Ein [Störungs-Relais](../tools/impairment.md) sitzt zwischen einem Client und seinem Ziel und
fügt UDP Latenz, Jitter, Verlust, Duplikate, Umordnung oder eine Bandbreitengrenze hinzu, oder es
verzögert und drosselt TCP-Verbindungen, setzt sie zurück oder lässt sie hängen. Jede Entscheidung folgt einem
Startwert, sodass derselbe Verkehr zweimal dasselbe Schicksal erleidet.

### Einen Test daraus machen {#experiments}

Ein [Experiment](../experiments/index.md) ist ein Ablauf von Schritten, den Sie auf einer Arbeitsfläche zeichnen: eine
Anfrage senden, auf die Antwort warten, sie prüfen, einen Wert extrahieren, verzweigen, in Schleifen wiederholen,
Zweige parallel ausführen, für den Durchlauf einen Emulator oder ein Störungs-Relais starten. Jeder Durchlauf
wird Schritt für Schritt protokolliert und als Bericht gespeichert, den Sie mit einem früheren vergleichen können.

### Einen Dienst unter Last setzen {#load}

Die HTTP-Anfrage eines Experiments kann [unter Last](../experiments/load.md) laufen — eine konstante
Rate, eine Rampe, Stufen, eine Spitze oder zufällige Ankünfte —, gemessen und an Schwellenwerten beurteilt.
Die Ansicht [HTTP](../protocols/http.md) bietet einen schnellen Last-Burst, und [Sturm](../tools/storm.md)
erzeugt rohe UDP- oder TCP-Last gegen Ihre eigenen Server und Leitungen.

### Finden, was da draußen ist {#discover}

[Broadcast und Erkennung](../protocols/broadcast.md) findet Geräte, die auf eine Suchanfrage antworten,
und der [Scanner](../tools/scanner.md) zeigt, welche TCP-Ports eines Hosts offen sind.

### Automatisieren {#automate}

Die [Kommandozeile](../automation/cli.md) `signallab` führt Experimente ohne Fenster aus und
endet mit einem Code, den eine Pipeline versteht; sie schreibt JUnit-Berichte für die [CI](../automation/ci.md).
Mit `signallab mcp` kann ein [KI-Assistent](../automation/mcp.md) Experimente lesen, schreiben und ausführen.
Ein [Server](../server/index.md) bietet dasselbe über seine [HTTP-API](../api/index.md).

## Zwei Arten, es zu betreiben {#two-ways}

| | Desktop-App | Server, im Browser |
| --- | --- | --- |
| Läuft unter | Windows 10 und 11 (x64), Linux (x86_64) | Linux als Docker-Image (amd64 und arm64) oder aus dem Quellcode gebaut |
| Sie nutzen es | in einem eigenen Fenster | in Chrome, Edge oder Firefox, von jedem Rechner, der den Server erreicht |
| Der Verkehr geht aus von | diesem Computer | dem Server |
| Mitgeliefert | die Kommandozeile `signallab` | die Kommandozeile `signallab`, im Image |
| Updates | von selbst, wenn Sie es sagen | mit seinem Image |

Beide sind dieselbe Oberfläche auf derselben Engine: Jede Ansicht, der Inspektor, Experimente
und Berichte funktionieren gleich. Ein Server ist die richtige Wahl, wenn die Technik in einem Netzwerk steht, das Ihr
eigener Computer nicht erreicht — ein Rack-PC oder ein kleiner Linux-Rechner daneben, bedient von einem
Laptop oder einer Pipeline. Siehe [Installieren und aktualisieren](install.md) und
[Der Server](../server/index.md); was sich zwischen beiden unterscheidet, steht unter
[Konzepte](concepts.md#desktop-and-server).

## Wie es aufgebaut ist {#organisation}

- **Ansichten** sind Werkzeuge für Arbeit, die Sie jetzt von Hand erledigen: dies senden, dort empfangen,
  jenen Emulator starten. Die Seitenleiste listet sie auf; jede behält, was Sie eingegeben und was sie empfangen hat,
  während Sie zu einer anderen wechseln. Siehe [Das Fenster](interface.md).
- **Experimente** sind Abläufe, die Sie einmal bauen und immer wieder ausführen, jedes Mal auf dieselbe Weise, mit
  einem Bericht für jeden Durchlauf.
- **Bibliotheken** bewahren auf, was Sie erstellt haben: Signale in der Signalbibliothek, Emulatoren in der
  Emulator-Bibliothek. Experimente nutzen beide.
- **Jobs** sind die lang laufende Arbeit — ein Monitor, ein Emulator, ein Relais, ein Durchlauf —, aufgelistet
  am unteren Rand des Fensters, wo Sie einen oder alle stoppen.

[Konzepte](concepts.md) erklärt jedes davon ausführlicher.

## Sicher von Haus aus {#defaults}

- Signal Lab sendet nichts, bevor Sie eine Schaltfläche drücken, und nur dorthin, wohin Sie es angeben. Die
  einzige Ausnahme ist die Update-Prüfung der Desktop-App, einmal täglich, die Sie abschalten können
  (siehe [Updates](install.md#updates)).
- Die Beispielsignale, die Beispiel-Emulatoren und die Experimentvorlagen zeigen alle auf
  `127.0.0.1`, diesen Computer. Das Netzwerk zu erreichen ist eine Entscheidung, die Sie treffen, indem Sie eine
  Adresse eingeben.
- Ein Server, der ohne Zugriffstoken gestartet wird, empfängt nur auf `127.0.0.1` und lehnt jede
  andere Adresse ab.
- Die Firewall ändert sich nur, wenn Sie es per Klick zulassen.

## Verantwortungsvoller Einsatz {#responsible-use}

::: danger Echter Verkehr an echte Hosts
[Sturm](../tools/storm.md), der [Scanner](../tools/scanner.md) und
[Broadcast](../protocols/broadcast.md) senden echten Verkehr an echte Hosts, und ein Broadcast
oder ein Subnetz-Sweep erreicht jedes Gerät im Segment, nicht nur das, an das Sie gedacht haben.
Richten Sie sie nur auf Systeme, die Ihnen gehören oder die Sie testen dürfen, und prüfen Sie zuerst, in welchem Netzwerk
Sie sich befinden. Hohe Raten können Leitungen auslasten und Angriffserkennungssysteme auslösen.
:::

Die Engine hat Leitplanken: Ein Sweep erreicht höchstens 1024 Hosts, und ein Beacon sendet
höchstens 50.000 Pakete pro Sekunde über alle seine Ziele hinweg. Es sind Leitplanken, keine
Erlaubnis.

## Sprachen {#languages}

Die Oberfläche, ihre Tooltips und Fehlermeldungen, die Kommandozeile `signallab`, die
Anmeldeseite des Servers und das Windows-Setup sprechen 11 Sprachen: English (Englisch), Русский
(Russisch), Español (Spanisch), Français (Französisch), Deutsch, Português (brasilianisches
Portugiesisch), 中文 (vereinfachtes Chinesisch), 日本語 (Japanisch), 한국어 (Koreanisch), हिन्दी (Hindi) und
العربية (Arabisch, von rechts nach links). Sie wechseln sie live in der Kopfzeile; siehe
[Das Fenster](interface.md#languages).

## Nächste Schritte {#next}

1. [Signal Lab installieren](install.md).
2. Sich im [Fenster](interface.md) zurechtfinden.
3. Die [ersten Schritte](first-steps.md) auf diesem Computer gehen.
4. Die [Konzepte](concepts.md) dahinter lesen.

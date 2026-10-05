---
title: Konzepte
description: Die Ideen hinter Signal Lab — Ansichten und Experimente, Signale, Jobs, Mitschnitt, Emulatoren, Störungs-Relais, Parameter, Vorlagen, Geheimnisse, Startwerte, Berichte und der Datenordner.
---

# Konzepte

Diese Seite erklärt die Ideen, auf denen Signal Lab aufbaut, damit sich der Rest
der Dokumentation leicht liest. Jeder Abschnitt verlinkt die Seite, die sein
Thema vollständig behandelt.

## Ansichten und Experimente {#screens-and-experiments}

Signal Lab kennt zwei Arbeitsweisen, und Sie werden beide nutzen.

- **Ansichten** sind Werkzeuge für Arbeit, die Sie jetzt von Hand erledigen:
  diese Nachricht senden, auf jenem Port hören, diesen Emulator starten, sehen,
  was der Broker hält. Sie versuchen, schauen, ändern etwas und versuchen es
  erneut. Jedes Protokoll und jedes Werkzeug hat eine Ansicht; siehe
  [Das Fenster](interface.md).
- **Experimente** sind Abläufe, die Sie einmal bauen und immer wieder gleich
  ausführen: eine Anfrage senden, auf die Antwort warten, sie prüfen, fortfahren
  oder verzweigen. Jeder Durchlauf wird Schritt für Schritt berichtet und
  gespeichert. Siehe [Experimente](../experiments/index.md).

Die beiden treffen sich an mehreren Stellen. In den Ansichten HTTP und OSC macht
[[ui:common.toExperiment]] aus dem gerade Gesendeten den nächsten Schritt des
Experiments. Im OSC-Monitor und in der Ansicht MQTT macht
[[ui:osc.waitForThis]] aus einer empfangenen Nachricht einen Schritt, der auf sie
wartet. Und ein Signal aus der Bibliothek kann ein Schritt werden — jedes außer
rohen UDP-Bytes, als Hex geschrieben.

## Signale und die Bibliothek {#signals}

Ein **Signal** ist eine Nachricht, die Sie behalten: ein Name, ein Ordner, eine
Notiz, was es bewirken soll, und was es sendet — eine OSC-Nachricht, rohe
UDP-Bytes, eine HTTP-Anfrage oder eine MQTT-Veröffentlichung. Die
**Signalbibliothek** hält sie in Ordnern, die Sie verschachteln, umbenennen und
verschieben können.

- Sie speichern ein Signal in den Ansichten OSC, HTTP und MQTT
  ([[ui:sig.saveNew]]), legen eines in der Ansicht [[ui:nav.signals]] an oder
  speichern einen Frame, den der Inspektor mitgeschnitten hat
  ([[ui:sig.fromFrame]]), der ihn dann Byte für Byte erneut sendet.
- Sie senden es aus der Ansicht [[ui:nav.signals]], von überall mit
  <kbd>Ctrl</kbd>+<kbd>K</kbd>, als Schritt eines Experiments oder mit
  `signallab fire` aus einem Terminal.
- Ein Signal sendet genau das, was seine Ansicht senden würde: dieselben Bytes,
  über denselben Weg, im Inspektor unter seinem echten Protokoll gezeigt.

Die Bibliothek ist eine einzige Datei, `signals.json`, im
[Datenordner](#data-folder): einfaches JSON, das Sie lesen, bearbeiten, auf einen
anderen Computer kopieren oder in einem Repository aufbewahren können. Sie
beginnt mit einer Reihe von Beispielsignalen, alle auf `127.0.0.1` gerichtet.
Siehe [Signale](../tools/signals.md).

## Jobs {#jobs}

Alles, was nach dem Drücken seiner Schaltfläche weiterläuft, ist ein **Job**: ein
OSC-Monitor oder -Generator, eine Broker- oder WebSocket-Verbindung, ein Beacon
oder Erkennungsempfänger, ein HTTP-Last-Burst, ein Emulator, ein Störungs-Relais,
ein Sturm, ein Scan, ein Durchlauf eines Experiments.

- Jeder Job hat ein Kärtchen in der Leiste des unteren Bereichs, mit seiner
  Nummer, seiner Art und einer Schaltfläche, um ihn zu stoppen. Die Seitenleiste
  zeigt, wie viele Jobs jede Ansicht laufen hat.
- [[ui:app.stopAll]] in der Kopfzeile stoppt jeden Job auf einmal.
- Ein Job, der von selbst endet — ein abgeschlossener Scan, ein bestandener
  Durchlauf, ein Monitor, dessen Port scheiterte —, lässt sein Kärtchen stehen,
  und die Konsole sagt, wie er endete.
- Ein Job läuft weiter, während Sie in anderen Ansichten arbeiten.
- Eine Aktualisierung zu installieren, stoppt zuerst jeden Job.

Auf einem Server gehören Jobs dem Server: Jede bei ihm angemeldete Seite sieht
dieselben Jobs und kann sie stoppen.

## Mitschnitt und der Inspektor {#capture}

Jedes Werkzeug — Sender, Monitore, Empfänger, Emulatoren, Relais, Durchläufe von
Experimenten — übergibt jeden Frame, den es sendet oder empfängt, einem
**Mitschnitt**, und der [Inspektor](../tools/inspector.md) zeigt ihn in einer
Zeitleiste.

- Der Mitschnitt ist **aus, bis Sie ihn** mit [[ui:ins.arm]] **starten**, und
  kostet nichts, solange er aus ist. Er bleibt an, in welcher Ansicht Sie auch
  sind, bis Sie ihn beenden.
- Er behält bis zu 8192 Frames und 64 MiB ihrer Bytes; die ältesten Frames machen
  neuen Platz. Jeder Frame behält bis zu 256 KiB seiner Bytes, und die Liste
  zeigt sein erstes KiB.
- [[ui:ins.pause]] hält die Liste an, damit Sie sie lesen können; der Mitschnitt
  läuft darunter weiter.
- Von einem Durchlauf verwendete Geheimniswerte werden in jedem Frame maskiert.
- Der ganze Mitschnitt lässt sich exportieren, jedes Byte darin, in eine
  `.jsonl`- oder `.txt`-Datei.

## Emulatoren {#emulators}

Ein **Emulator** spielt die Gegenseite: die API, das Gerät oder den Dienst, mit
dem Ihr System spricht. Jeder ist ein Dokument mit einem Protokoll, der Adresse,
auf der er empfängt, und Regeln, die sagen, was geantwortet wird:

| Protokoll | Was er nachbildet |
| --- | --- |
| HTTP | Eine API: Routen nach Methode und Pfad, Antworten der Reihe nach, abwechselnd oder zufällig, mit Verzögerungen und Fehlverhalten |
| OSC | Ein Gerät, das OSC-Nachrichten nach Adresse und Argumenten beantwortet |
| UDP | Ein Gerät, das Datagramme nach ihren Nutzdaten beantwortet |
| TCP | Ein Gerät, das Zeilen auf einer TCP-Verbindung beantwortet, mit einer Begrüßung |
| MQTT | Ein Broker, der weiterleitet, was Clients veröffentlichen, und nach Regeln wie ein Gerät antwortet |

Ein Emulator kann langsam antworten, scheitern, die Verbindung schließen, einen
fehlerhaften Body senden oder nach Zeitplan ausfallen. Jeder Austausch wird
gezählt, in seiner Ansicht aufgelistet und für den Inspektor mitgeschnitten.

Sie starten einen aus der Ansicht [[ui:nav.emulators]], wo er als Job läuft; aus
dem Knoten [[ui:exp.node.emulator]] eines Experiments, wo er für den ganzen
Durchlauf antwortet; oder mit `signallab emulate`. Die **Emulatorbibliothek** ist
`emulators.json` im Datenordner. Sie beginnt mit einem Emulator jeder Art, alle
auf `127.0.0.1`:

| Emulator | Empfängt auf | Was er tut |
| --- | --- | --- |
| [[ui:seed.emu.demo-api.name]] | `127.0.0.1:8080` (HTTP) | Eine Zustandsprüfung, einen Benutzer nach ID, ein Anlegen, eine langsame Antwort und eine Route, die zweimal scheitert, bevor sie funktioniert |
| [[ui:seed.emu.osc-device.name]] | `127.0.0.1:9100` (OSC) | Antwortet auf `/ping` mit `/pong` und einem Zähler, bestätigt `/fader/…` mit `/ack`, nimmt `/cue/…` ohne ein Wort an |
| [[ui:seed.emu.udp-device.name]] | `127.0.0.1:7100` (UDP) | Antwortet auf `PING` mit `PONG` und einem Zähler, auf alles andere mit der Anzahl der empfangenen Bytes |
| [[ui:seed.emu.tcp-device.name]] | `127.0.0.1:7200` (TCP) | Ein Zeilenprotokoll wie das eines Projektors: Begrüßt mit `READY`, meldet und schaltet den Strom, sagt `BYE` und legt bei `QUIT` auf |
| [[ui:seed.emu.mqtt-broker.name]] | `127.0.0.1:1883` (MQTT) | Eine retained Nachricht `lab/status` und eine Lampe: `ON` oder `OFF`, an `lab/<name>/set` veröffentlicht, wird auf `lab/<name>/state` beantwortet |

Siehe [Emulatoren](../tools/emulators.md).

## Störungs-Relais {#impairment}

Ein **Störungs-Relais** sitzt zwischen einem Client und seinem Ziel. Sie richten
den Client statt auf das echte Ziel auf die Empfangsadresse des Relais; das
Relais leitet in beide Richtungen weiter und verschlechtert, was hindurchgeht,
nach einem **Profil**:

- über **UDP** erleidet jedes Datagramm sein eigenes Schicksal: Latenz und
  Jitter, Verlust und Verlust-Bursts, Duplizierung, Verfälschung, Umordnung, eine
  Bandbreitengrenze oder gar nichts (offline);
- über **TCP** wird jede Verbindung mit einer eigenen zum Ziel gekoppelt, und
  beide Datenströme werden verzögert, an eine Bandbreitengrenze gehalten,
  zurückgesetzt oder halboffen gelassen.

Vorgaben setzen ein Profil mit einem Klick, von einem Kabel bis zu einer
Satellitenverbindung. Eine Änderung gilt, während das Relais läuft, ohne seinen
Port aufzugeben. Jede Entscheidung wird aus einem Startwert gezogen, sodass
derselbe Verkehr wieder dasselbe Schicksal erleidet.

In der Ansicht [[ui:nav.netsim]] läuft ein Relais als Job. In einem Experiment
öffnet ein Knoten [[ui:exp.node.impairment]] eines für den Durchlauf, und
[[ui:exp.node.impairment_change]] schaltet mitten im Durchlauf sein Profil um.
Siehe [Störung](../tools/impairment.md) und
[Störungen](../experiments/faults.md).

## Experimente {#experiments}

### Knoten und Verbindungen {#nodes-and-wires}

Ein Experiment ist ein Graph aus **Knoten**, die durch **Verbindungen** verbunden
sind. Jeder Knoten ist ein Schritt: Er sendet etwas, wartet auf etwas, prüft
einen Wert, extrahiert einen, ändert den Ablauf oder bereitet den Durchlauf vor —
ein Emulator, ein Störungs-Relais. Jedes Experiment hat genau einen
[[ui:exp.node.start]] und einen [[ui:exp.node.end]] und hält bis zu 64 Knoten.
Das im Editor offene Experiment wird beim Bearbeiten gespeichert. Siehe
[Knoten](../experiments/nodes.md).

### Ausgänge {#outputs}

Eine Verbindung führt vom **Ausgang** eines Knotens zum Eingang eines anderen.
Die meisten Knoten haben einen Ausgang; andere wählen zwischen mehreren:
[[ui:exp.yes]] und [[ui:exp.no]] für eine Verzweigung,
[[ui:exp.portMatched]] und [[ui:exp.portTimeout]] für ein Warten,
[[ui:exp.portBody]], [[ui:exp.portDone]] und [[ui:exp.portLimit]] für eine
Schleife, [[ui:exp.branch1]] und [[ui:exp.branch2]] für einen parallelen Zweig.

Ein Ausgang darf mehrere Verbindungen haben: Jede läuft als eigener Zweig
parallel, und ein [[ui:exp.node.join]] wartet auf alle Verbindungen, die in ihn
führen. Nur der Rumpf einer [[ui:exp.node.loop]] darf zurückführen; jeder andere
Zyklus ist ein Fehler. Siehe [Ablauf](../experiments/flow.md).

### Parameter und Profile {#parameters}

Ein **Parameter** ist ein benannter Wert — ein Host, ein Port, ein Benutzername —,
einmal unter [[ui:exp.params]] geschrieben und in jedem Feld als `{{name}}`
verwendet. Ein **Profil** ändert mehrere Parameter auf einmal: eines für den
Laptop, eines für die Bühne, eines für die Spielstätte. Sie wählen das Profil,
das Durchläufe verwenden, oder [[ui:exp.runWith]] ein Profil, andere Werte oder
einen Startwert für einen einzigen Durchlauf, ohne das Experiment zu ändern. Ein
Experiment hält bis zu 64 Parameter und 32 Profile. Siehe
[Daten](../experiments/data.md).

### Vorlagen {#templates}

Die meisten Textfelder von Knoten sind **Vorlagen**: einfacher Text mit
Ausdrücken in doppelten geschweiften Klammern, gefüllt, während der Schritt
läuft.

- `{{host}}` — ein Parameter oder eine früher im Durchlauf gesetzte Variable,
  etwa ein Wert, den ein Knoten [[ui:exp.node.extract]] aus einer Antwort nahm,
  oder eine Antwort, die ein Warten empfing (`{{reply.args[0]}}`).
- `{{secret.API_TOKEN}}` — ein Geheimnis.
- `{{run.id}}`, `{{run.seed}}`, `{{now}}`, `{{now.iso}}`, `{{counter}}` — der
  Durchlauf und der Moment.
- `{{uuid}}`, `{{random_int(1, 10)}}`, `{{random_float(0, 1, 2)}}`,
  `{{pick("a", "b")}}` — erzeugte Werte.

Nur die Engine füllt Vorlagen, ein Feld bedeutet also dasselbe in einem
Durchlauf, in der Vorschau des Editors und in [[ui:exp.sendNow]]. Ein unbekannter
Name ist ein Fehler, niemals eine leere Zeichenkette. Siehe
[Daten](../experiments/data.md).

### Geheimnisse {#secrets}

Ein **Geheimnis** ist ein Wert, den ein Experiment verwendet, aber nie
speichert — ein Token, ein Passwort. Das Experiment hält nur seinen Namen; Felder
verwenden es als `{{secret.NAME}}`; und jeder Text, den ein Durchlauf berichtet,
jeder Schritt, der Bericht und jeder Frame des Inspektors zeigen es maskiert.
Kein Befehl gibt jemals den Wert eines Geheimnisses zurück.

Wo die Werte liegen, hängt davon ab, wo Signal Lab läuft:

- **Die Desktop-App unter Windows** hält sie in der
  Windows-Anmeldeinformationsverwaltung. Sie legen sie unter [[ui:exp.params]] →
  [[ui:exp.secrets]] fest.
- **Die Desktop-App unter Linux** hat keinen Anmeldedatenspeicher, in dem sie sie
  halten könnte, Experimente mit Geheimnissen laufen dort also von der
  Kommandozeile oder auf einem Server.
- **Ein Server** liest sie nur lesend aus seiner Umgebung
  (`SIGNALLAB_SECRET_<NAME>`) oder aus einer Datei je Namen in seinem
  Geheimnisordner (standardmäßig `/run/secrets/signallab/<NAME>`); aus einem
  Browser lassen sie sich nicht festlegen.
- **Die Kommandozeile** liest sie wie ein Server oder, wenn verlangt, aus dem
  Anmeldedatenspeicher des Systems. Siehe
  [Die Kommandozeile](../automation/cli.md).

### Startwerte {#seeds}

Jeder Durchlauf hat einen **Startwert**, eine Zahl, die alles Zufällige in ihm
entscheidet: erzeugte Werte, den Jitter einer Wiederholung, die zufällige Wahl
einer Emulator-Antwort, jede Entscheidung eines Störungs-Relais. Derselbe
Startwert und derselbe Verkehr ergeben denselben Durchlauf. Für jeden Durchlauf
wird ein neuer Startwert gezogen, es sei denn, das Experiment fixiert einen —
[[ui:exp.pinSeed]] in der Zeitleiste des Durchlaufs fixiert den Startwert des
letzten Durchlaufs, und [[ui:exp.seed]] unter [[ui:exp.params]] legt einen fest.

### Durchläufe und Berichte {#reports}

Ein **Durchlauf** beginnt bei [[ui:exp.node.start]], folgt den Verbindungen und
besteht, wenn er [[ui:exp.node.end]] erreicht, ohne dass ein Schritt fehlschlug.
Er wird gestoppt, wenn er länger als 300 Sekunden dauert. Jeder Schritt erscheint
in der Zeitleiste des Durchlaufs, wenn er beginnt und endet.

Ein Durchlauf, der endet — bestanden oder fehlgeschlagen —, schreibt einen
**Bericht** in den Ordner `runs` des Datenordners: den Namen des Experiments, den
Startwert, das Profil und die verwendeten Werte, wann er begann und endete, das
Ergebnis und seinen Fehler, jeden Schritt und was seine Emulatoren und Relais
gezählt haben. Zwei Durchläufe desselben Experiments lassen sich vergleichen.
Siehe [Durchläufe und Berichte](../experiments/runs.md).

## Der Datenordner {#data-folder}

Alles, was Signal Lab behält, ist eine Datei in einem Ordner:
`Documents/SignalLab` in Ihrem Home-Ordner, unter Windows wie unter Linux. Ein
Server hält seinen eigenen, den Sie beim Start wählen (`/data` im Docker-Image).

| Datei oder Ordner | Was er hält |
| --- | --- |
| `experiment.json` | Das im Editor offene Experiment |
| `signals.json` | Die Signalbibliothek |
| `emulators.json` | Die Emulatorbibliothek |
| `runs/` | Ein Bericht für jeden Durchlauf |
| `exports/` | Aus dem Dialog der Experimente exportierte Experimente |
| `capture-….jsonl`, `capture-….txt` | Exporte des Inspektors |

Die Dateien sind JSON und werden vollständig geschrieben. Lässt sich eine nicht
lesen, sagt Signal Lab, welche Datei und wo der Fehler liegt, und lässt sie, wie
sie ist, statt von vorn zu beginnen. Siehe
[Dateien und Ordner](../reference/files.md).

## Desktop und Server {#desktop-and-server}

Die Desktop-App und ein Server führen dieselbe Engine hinter derselben
Oberfläche aus. Was sich unterscheidet:

| | Desktop-App | Server, im Browser |
| --- | --- | --- |
| Wo Verkehr ausgeht, wo Monitore hören | Dieser Computer | Der Server |
| Datenordner | `Documents/SignalLab` | Der des Servers; fahren Sie in der Kopfzeile mit der Maus über [[ui:app.server]], um ihn zu sehen |
| Geheimnisse | Windows-Anmeldeinformationsverwaltung, in der App festgelegt; unter Linux keine | Nur lesend, aus der Umgebung des Servers oder aus Geheimnisdateien |
| Berichte, Exporte, Mitschnitte | In den Datenordner geschrieben; der Pfad wird gezeigt | Vom Browser heruntergeladen |
| Anmelden | — | Mit dem Zugriffstoken des Servers, wenn er eines hat |
| Jobs, der Cookie-Speicher der Ansicht HTTP | Der dieser App | Der des Servers, von jeder bei ihm angemeldeten Seite geteilt |
| Firewall | Ein Hinweis bietet an, Signal Lab zuzulassen (Windows) | Wird von Signal Lab nie verändert |
| Updates | Installiert signierte Releases, wenn Sie klicken | Wird mit seinem Image aktualisiert |

Siehe [Der Server](../server/index.md) und
[Serversicherheit](../server/security.md).

## Was Signal Lab nicht von allein tut {#on-its-own}

- **Es sendet nur, wenn Sie handeln**, und nur an die Adressen, die Sie eingeben.
  Die App zu starten, sendet nichts — außer in der Desktop-App die tägliche
  Update-Prüfung, die Sie abschalten können. Feedback geht nur hinaus, wenn Sie
  das Formular absenden.
- **Seine Beispiele bleiben auf diesem Computer.** Die Beispielsignale, die
  Beispiel-Emulatoren, neue Emulatoren und die Vorlagen der Experimente verwenden
  alle `127.0.0.1`. Empfänger, die Sie starten — ein OSC-Monitor, der
  Erkennungsempfänger, ein Störungs-Relais —, stehen standardmäßig auf `0.0.0.0`,
  jeder Netzwerkkarte, damit andere Rechner sie erreichen; geben Sie `127.0.0.1`
  ein, um einen auf diesem Computer zu halten.
- **Es ändert die Firewall nur, wenn Sie** [[ui:fw.allow]] **klicken** und die
  Administratorabfrage von Windows bestätigen oder `signallab firewall allow`
  ausführen. Ein Server ändert nie die Firewall seines Hosts.
- **Ein Server ohne Zugriffstoken** hört nur auf `127.0.0.1` und weigert sich, auf
  einer anderen Adresse zu starten.
- **Es hält sich an Leitplanken**: Ein Broadcast-Sweep erreicht höchstens 1024
  Hosts, und ein Beacon sendet über alle seine Ziele hinweg höchstens 50 000
  Pakete pro Sekunde.

Die Leitplanken sind keine Erlaubnis: [Sturm](../tools/storm.md), der
[Scanner](../tools/scanner.md) und [Broadcast](../protocols/broadcast.md) senden
echten Verkehr. Verwenden Sie sie nur in Netzwerken und auf Hosts, die Ihnen
gehören oder die zu testen Sie berechtigt sind.

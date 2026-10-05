---
title: Das Fenster
description: Sich im Fenster von Signal Lab zurechtfinden — die Ansichten der Seitenleiste, die Kopfzeile, Konsole und Inspektor, die Signalpalette, Bereiche, Tooltips und Sprachen.
---

# Das Fenster

Das Fenster von Signal Lab besteht aus vier Teilen: Die **Seitenleiste** links listet die Ansichten auf, die
**Kopfzeile** oben enthält, was überall gilt, die gewählte **Ansicht** füllt
die Mitte, und der **untere Bereich** zeigt die Konsole, die laufenden Jobs und den
Inspektor. Die Desktop-App und die Seite eines Servers im Browser sehen gleich aus; die wenigen
Unterschiede stehen unter [Im Browser](#browser).

## Die Seitenleiste {#sidebar}

Jede Ansicht ist ein eigenes Werkzeug. Klicken Sie auf eine, um sie zu öffnen:

| Ansicht | Wofür sie da ist |
| --- | --- |
| [[ui:nav.experiment]] | Testabläufe auf einer Arbeitsfläche bauen, ausführen und jeden Durchlauf Schritt für Schritt lesen. [Experimente](../experiments/index.md) |
| [[ui:nav.signals]] | Die Signalbibliothek: benannte Nachrichten in Ordnern, zum Bearbeiten und erneuten Senden. [Signale](../tools/signals.md) |
| [[ui:nav.emulators]] | Nachgebildete HTTP-APIs, OSC-, UDP- und TCP-Geräte und MQTT-Broker, die nach Regeln antworten. [Emulatoren](../tools/emulators.md) |
| [[ui:nav.osc]] | OSC-Nachrichten senden, einen Port überwachen, eine Wellenform auf einen Endpunkt geben. [OSC](../protocols/osc.md) |
| [[ui:nav.mqtt]] | Mit einem Broker verbinden, jedes Topic sehen, das er hält, veröffentlichen und Retained-Werte löschen. [MQTT](../protocols/mqtt.md) |
| [[ui:nav.broadcast]] | An viele Hosts auf einmal senden — eine Liste, Broadcast, Multicast, ein Subnetz-Sweep — und mithören, wer antwortet. [Broadcast und Erkennung](../protocols/broadcast.md) |
| [[ui:nav.http]] | Eine Anfrage mit ihrer ganzen Antwort, dann ein Last-Burst gegen denselben Endpunkt. [HTTP](../protocols/http.md) |
| [[ui:nav.ws]] | Mit einem WebSocket-Dienst verbinden, Text oder Bytes senden, jede Nachricht lesen. [WebSocket](../protocols/websocket.md) |
| [[ui:nav.netsim]] | Ein Relais, das UDP- oder TCP-Verkehr zwischen einem Client und seinem Ziel verschlechtert. [Störung](../tools/impairment.md) |
| [[ui:nav.storm]] | Rohe UDP- oder TCP-Last gegen Ihre eigenen Server und Leitungen. [Sturm](../tools/storm.md) |
| [[ui:nav.scan]] | Welche TCP-Ports eines Hosts offen sind und was der Dienst als Erstes sagt. [Scanner](../tools/scanner.md) |

Die Seitenleiste erscheint zunächst schmal, mit dem Symbol und einem Kurznamen jeder Ansicht; fahren Sie mit der Maus über einen Eintrag,
um seinen vollen Namen zu sehen. Das **☰** links in der Kopfzeile ([[ui:app.expandNav]] /
[[ui:app.collapseNav]]) wechselt zwischen der schmalen Form und der vollständigen Liste, und Signal Lab
merkt sich Ihre Wahl.

Eine Zahl am Eintrag einer Ansicht zählt die Jobs, die von dort aus laufen, etwa einen Monitor oder einen
Emulator, sodass Sie sehen, was noch läuft, ohne die Ansicht zu öffnen. Am unteren Ende der
Seitenleiste öffnet die Version „Über Signal Lab“, und die Zeile darunter zählt alle laufenden Jobs.

Signal Lab öffnet sich in der Ansicht, die Sie zuletzt verwendet haben.

## Die Kopfzeile {#header}

Von links nach rechts:

| Element | Was es tut |
| --- | --- |
| **☰** | Zeigt die Seitenleiste schmal oder als vollständige Liste. |
| Der Host | [[ui:app.host]], dann Name und Netzwerkadresse des Computers, auf dem die Engine läuft: dieser Computer in der Desktop-App, der Server im Browser. Im Browser steht dort außerdem [[ui:app.server]] — fahren Sie mit der Maus darüber, um zu sehen, wo der Server seine Dateien ablegt —, und der Punkt daneben ändert sich, wenn die Seite ihre Verbindung verliert. |
| Die Update-Schaltfläche | Erscheint in der Desktop-App, wenn ein neueres Release gefunden wurde, und öffnet „Über Signal Lab“, um es zu installieren. Siehe [Updates](install.md#updates). |
| Das Buch | [[ui:app.docs]]: diese Dokumentation, auf der Seite der Ansicht, in der Sie gerade sind. <kbd>F1</kbd> tut dasselbe von überall. |
| **✉** | [[ui:feedback.open]]: eine Nachricht an die Entwickler. Siehe [An die Entwickler schreiben](#feedback). |
| **?** | [[ui:about.open]]: die Version, wer Signal Lab macht, wie Sie die Entwickler erreichen, und Updates. |
| Die Flagge | [[ui:app.language]]: die Sprache der Oberfläche. Siehe [Sprachen](#languages). |
| [[ui:app.signOut]] | Im Browser, wenn der Server ein Zugriffstoken verlangt: beendet die Sitzung dieses Browsers. |
| [[ui:app.stopAll]] | Stoppt alle laufenden Jobs auf einmal: Monitore, Generatoren, Beacons, Emulatoren, Relais, Stürme, Scans, Durchläufe. Ist ausgegraut, solange nichts läuft. |

### Die Dokumentation {#docs}

Die Dokumentation ist in die App und in den Server eingebaut, daher ist sie ohne
Internetverbindung verfügbar, in der Sprache der Oberfläche. Die Desktop-App zeigt sie in einem eigenen
Fenster — drücken Sie die Schaltfläche erneut in einer anderen Ansicht, springt dieses Fenster zur Seite
dieser Ansicht — und öffnet Links, die die Dokumentation verlassen, in Ihrem Browser. Im
Browser öffnet sie sich in einem eigenen Tab. Auch „Über Signal Lab“ hat eine Schaltfläche [[ui:app.docs]], die
[Was ist Signal Lab](index.md) öffnet.

### Über Signal Lab {#about}

„Über Signal Lab“ zeigt die Version, den Entwickler, die Kontaktadresse (mit einer Schaltfläche, die sie
kopiert), den Quellcode und die Lizenz. In der Desktop-App sucht der Abschnitt [[ui:update.title]]
nach Updates, zeigt sie an und installiert sie — siehe [Updates](install.md#updates). Im Browser
steht dort [[ui:update.server]]. [[ui:about.writeUs]] öffnet das Feedback-Formular.

### An die Entwickler schreiben {#feedback}

Das **✉** in der Kopfzeile öffnet ein Formular, das direkt an die Entwickler geht:

| Feld | Was hineingehört |
| --- | --- |
| [[ui:feedback.message]] | Was passiert ist und was Sie stattdessen erwartet haben. Pflichtfeld; höchstens 20.000 Zeichen. |
| [[ui:feedback.email]] | Optional: wohin die Entwickler Ihnen antworten können. Nur sie sehen die Adresse. |
| [[ui:feedback.screenshots]] | Bis zu 6 Bilder (PNG, JPEG, WebP oder GIF), je 8 MB und zusammen 15 MB. Fügen Sie eines mit <kbd>Ctrl</kbd>+<kbd>V</kbd> ein, ziehen Sie Dateien auf das Fenster, oder drücken Sie [[ui:feedback.addScreenshot]]. |
| [[ui:feedback.logs]] | Die Zeilen der Konsole und [[ui:feedback.systemInfo]], jeweils als eigene Datei angehängt. [[ui:feedback.show]] zeigt genau, was gesendet wird; entfernen Sie einen der Haken, um den Teil wegzulassen. |

Die angehängten Logs lassen den Namen dieses Computers, seine Netzwerkadresse und die Namen in
den Pfaden Ihrer Ordner weg. <kbd>Ctrl</kbd>+<kbd>Enter</kbd> sendet das Formular; ist es abgeschickt,
erhalten Sie eine Referenznummer, die auch die Konsole festhält. Die Nachricht läuft über
den eigenen Dienst des Studios, der sie per E-Mail weiterleitet; die App enthält dafür kein Passwort.

## Der untere Bereich {#bottom-panel}

Der Bereich unter jeder Ansicht hat zwei Registerkarten, [[ui:console.title]] und
[[ui:dock.inspector]], und zwischen ihnen und den Schaltflächen des Bereichs die Leiste der laufenden
Jobs.

- Der **Pfeil** links ([[ui:console.collapse]] / [[ui:console.expand]]) klappt den
  Bereich bis auf seine Leiste ein oder wieder auf. Eingeklappt zeigt die Leiste weiterhin die neueste
  Zeile der Konsole; klicken Sie auf diese Zeile, um den Bereich zu öffnen.
- Ziehen Sie die Oberkante des Bereichs, um ihn höher oder niedriger zu machen (siehe [Bereiche anpassen](#panes)).
- Die Schaltfläche rechts ([[ui:dock.maximise]]) bringt ihn auf die Höhe des Fensters und wieder zurück
  ([[ui:dock.restore]]).

Ob der Bereich offen ist, welche Registerkarte er zeigt und wie hoch er ist, bleibt für das nächste
Mal erhalten.

### Die Konsole {#console}

Die Konsole sagt, was jedes Werkzeug getan hat und was schiefging, das Neueste zuletzt: eine gesendete Nachricht
und ihre Größe, ein gestarteter Monitor, Status und Dauer einer Antwort, ein beendeter Job und
warum er endete. Jede Zeile hat die Uhrzeit (auf die Millisekunde), ein Kürzel für das Werkzeug und die Meldung,
gefärbt nach ihrer Art — erledigt, Information, Warnung oder Fehler.

- Mit [[ui:console.autoscroll]] bleibt die neueste Zeile sichtbar, während Zeilen eintreffen; entfernen Sie den Haken, um
  zurückzulesen, während weitere hinzukommen.
- [[ui:common.clear]] leert sie.
- Sie behält die letzten 500 Zeilen.
- Ein Wechsel der Sprache schreibt die ganze Konsole in der neuen Sprache neu.

### Jobs {#jobs-strip}

Jeder laufende Job — ein Monitor, ein Generator, ein Beacon, eine Broker-Verbindung, ein Emulator,
ein Relais, ein Sturm, ein Scan, ein Durchlauf eines Experiments — hat in der Leiste ein Kärtchen mit seiner Nummer
und seiner Art und eine eigene Schaltfläche, um ihn zu stoppen. Läuft nichts, steht in der Leiste
[[ui:console.empty]]. Siehe [Jobs](concepts.md#jobs).

### Die Registerkarte Inspektor {#inspector-tab}

Die Registerkarte [[ui:dock.inspector]] zeigt jeden Frame, den die Werkzeuge senden und empfangen, solange
der Mitschnitt läuft, neben der Ansicht, in der Sie gerade arbeiten. Ihr Punkt leuchtet, solange
der Mitschnitt läuft, und eine Zahl zählt die mitgeschnittenen Frames. Der Bereich öffnet sich hoch genug
für die Liste des Inspektors und die Details eines Frames; ein Link auf einen Frame an anderer Stelle — in der
Zeitleiste eines Durchlaufs oder in der Liste eines Emulators — öffnet diese Registerkarte bei diesem Frame. Einmal geöffnet,
behält der Inspektor seine Liste und Auswahl, auch während der Bereich geschlossen ist. Wie Sie ihn verwenden:
[Inspektor](../tools/inspector.md).

## Ein Signal von überall senden {#palette}

Drücken Sie in jeder Ansicht <kbd>Ctrl</kbd>+<kbd>K</kbd>, um die Palette [[ui:sig.paletteTitle]] zu öffnen: Tippen Sie
ein paar Buchstaben aus dem Namen, Ordner oder Ziel eines Signals, wählen Sie mit <kbd>↑</kbd> und
<kbd>↓</kbd> und drücken Sie <kbd>Enter</kbd>, um es zu senden. Sie listet bis zu 12 Signale auf
einmal auf. <kbd>Esc</kbd>, ein Klick außerhalb oder erneut <kbd>Ctrl</kbd>+<kbd>K</kbd> schließt
sie. Die Konsole sagt, was wohin gesendet wurde. Siehe [Signale](../tools/signals.md).

## Bereiche anpassen {#panes}

Zwischen Bereichen, deren Größe Sie ändern können, sitzt ein schmaler Griff: an der Oberkante des unteren Bereichs
([[ui:layout.console]]) und in der Ansicht der Experimente am Rand der Eigenschaften
([[ui:layout.properties]]) und an der Oberkante der Zeitleiste des Durchlaufs ([[ui:layout.timeline]]).

- Ziehen Sie den Griff.
- Oder fokussieren Sie ihn mit <kbd>Tab</kbd> und verwenden Sie die Pfeiltasten: Jeder Druck verschiebt ihn um 16 Pixel,
  mit <kbd>Shift</kbd> viermal so weit; <kbd>Home</kbd> und <kbd>End</kbd> springen zur
  kleinsten und größten Größe.
- Ein Doppelklick darauf oder <kbd>Enter</kbd> gibt dem Bereich seine Standardgröße
  zurück.

Die Größen bleiben für das nächste Mal erhalten.

## Tooltips {#tooltips}

Ansichten zeigen Beschriftungen, Werte und Zustände und verlagern ihre Erklärungen in Tooltips: was
ein Feld erwartet, was `0` dort bedeutet, welche Taste dasselbe tut. Ein Tooltip erscheint, wenn
Sie den Mauszeiger etwa eine halbe Sekunde auf einem Element ruhen lassen, und sofort, wenn Sie es
mit der Tastatur erreichen; ein Feld zeigt den Tooltip seiner Beschriftung. <kbd>Esc</kbd>, Tippen,
ein Klick oder Scrollen blendet ihn aus. Screenreader lesen denselben Text vor.

Fehlermeldungen sagen, wo und was schiefging und warum; der Wortlaut des Systems selbst ist
unter [[ui:err.details]] eingeklappt.

## Ansichten behalten ihren Zustand {#state}

Eine Ansicht öffnet sich, wenn Sie sie zum ersten Mal aufrufen, und bleibt dann, wie sie ist, während Sie in
einer anderen arbeiten: was Sie eingegeben haben, die letzte Antwort, die Nachrichtenliste eines Monitors und der Job
dahinter, sogar die Scrollposition sind alle noch da, wenn Sie zurückkehren. Ein laufender Job
läuft weiter, gleich welche Ansicht Sie betrachten.

Manche Werte bleiben auch über Neustarts hinweg erhalten: die OSC-Nachricht, die Sie gesendet haben, die HTTP-Anfrage,
die WebSocket-Adresse, die Größen der Bereiche, die Ansicht, in der Sie waren.

## Der Firewall-Hinweis {#firewall-notice}

Die Windows Defender Firewall entscheidet Programm für Programm, ob andere Rechner es erreichen
dürfen. Sobald ein Programm zum ersten Mal auf einem Port empfängt, fragt Windows die Person am Bildschirm — und ein
*Abbrechen* dort oder ein Netzwerk, das Windows als öffentlich einstuft, verwirft stillschweigend alles, was andere
Rechner senden. Ein Monitor, der nichts anzeigt, ist das übliche Anzeichen.

Deshalb prüft Signal Lab in der Desktop-App unter Windows die Firewall einmal, sobald zum ersten Mal etwas auf Empfang geht — ein OSC-Monitor,
der Erkennungsempfänger, ein Störungs-Relais, ein Emulator oder ein Durchlauf eines Experiments.
Steht die Firewall im Weg, meldet das ein Hinweis unter
der Kopfzeile:

- [[ui:fw.allow]] fordert mit der Abfrage von Windows selbst Administratorrechte an und lässt dann
  andere Rechner Signal Lab in privaten und Domänennetzwerken erreichen.
- In einem Netzwerk, das Windows als öffentlich einstuft — oft das WLAN einer Spielstätte —, heißt die Schaltfläche stattdessen
  [[ui:fw.allowPublic]].
- [[ui:fw.dismiss]] blendet den Hinweis für diese Sitzung aus.

Das Zulassen ersetzt die eingehenden Firewall-Regeln von Signal Lab durch eine einzige Zulassungsregel. Verkehr
auf diesem Computer selbst (`127.0.0.1`) ist nie betroffen, daher können Sie den Hinweis ignorieren,
solange Sie über Loopback arbeiten. Eine Installation für alle hat die Regel bereits; siehe
[Was das Setup hinzufügt](install.md#windows-setup-adds). `signallab doctor` meldet dasselbe
im Terminal, und `signallab firewall allow` behebt es dort — siehe
[Die Kommandozeile](../automation/cli.md). Unter Linux und im Browser gibt es keinen Hinweis: Die
Firewall eines Servers ist Sache seines Administrators.

## Im Browser {#browser}

Die Seite eines [Servers](../server/index.md) ist dieselbe Oberfläche, mit wenigen Unterschieden:

- Der Host in der Kopfzeile nennt den Server, mit [[ui:app.server]] daneben.
- Verlangt der Server ein Zugriffstoken, melden Sie sich einmal an, und [[ui:app.signOut]] in
  der Kopfzeile beendet die Sitzung.
- Verliert die Seite ihre Verbindung zum Server, zeigt eine Leiste
  [[ui:app.connectionLost]], bis sie wieder besteht; die Konsole vermerkt beides.
- Berichte der Durchläufe, Exporte und Mitschnitte des Inspektors lädt der Browser herunter, statt
  einen Pfad anzuzeigen.
- „Über Signal Lab“ hat keine Updates: Der Server wird mit seinem Image aktualisiert.

Alles, was die Seite eines Servers tut, geschieht auf dem Server: Der Verkehr geht von ihm aus, Monitore
empfangen auf seinen Ports, Dateien landen in seinem Datenordner. Siehe
[Konzepte](concepts.md#desktop-and-server).

## Sprachen {#languages}

Die Flagge in der Kopfzeile zeigt die aktuelle Sprache und ihre zwei Buchstaben. Klicken Sie darauf, um
die Liste aller Sprachen zu öffnen, jede mit ihrer Flagge und ihrem eigenen Namen, und wählen Sie eine. In
der Liste bewegen <kbd>↑</kbd>, <kbd>↓</kbd>, <kbd>Home</kbd> und <kbd>End</kbd> die Auswahl, ein Buchstabe
springt zur nächsten Sprache, die mit ihm beginnt — im eigenen Namen oder auf Englisch, sodass
<kbd>g</kbd> Deutsch findet —, <kbd>Enter</kbd> wählt aus und <kbd>Esc</kbd> schließt die Liste.

Die Oberfläche wechselt sofort, ohne Neustart: jede Ansicht, jeder Tooltip und jeder Fehler, und
auch die früheren Zeilen der Konsole. Beim ersten Start wählt Signal Lab die erste der
Sprachen Ihres Systems, die es kennt, oder Englisch, und behält von da an Ihre Wahl — im
Browser für diesen Browser.

Arabisch stellt das ganze Fenster auf rechts nach links um. Was Daten sind, bleibt so von links nach rechts, wie es
geschrieben ist: Adressen, Hex-Dumps, Code und die Arbeitsfläche der Experimente.

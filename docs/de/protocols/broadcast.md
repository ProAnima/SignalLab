---
title: Broadcast
description: Eine Nutzlast an eine Liste von Hosts, eine Broadcast-Adresse, eine Multicast-Gruppe oder jeden Host eines Subnetzes senden — einmal oder als Beacon — und mit dem Erkennungsempfänger mithören, wer antwortet.
---

# Broadcast

Die Ansicht [[ui:nav.broadcast]] ([[ui:bc.title]]) sendet eine UDP-Nutzlast an
viele Ziele auf einmal und hört auf der anderen Seite mit, wer antwortet. Nutzen
Sie sie, um Geräte in einem Netzwerk zu finden, zu prüfen, ob ein Multicast-Strom
einen Empfänger erreicht, oder ein Gerät zu spielen, das auf Erkennungsanfragen
antwortet.

- Der [[ui:bc.emitter]] sendet an eine Liste von Hosts, eine Broadcast-Adresse,
  eine Multicast-Gruppe oder jeden Host eines Subnetzes — einmal oder immer
  wieder als Beacon.
- Die [[ui:bc.discovery]] hört auf einem Port, tritt Multicast-Gruppen bei, listet
  jeden Peer auf, der mit ihr spricht, und kann auf Anfragen antworten.

::: danger
Broadcast, Multicast und ein Sweep erreichen jedes Gerät im Netzwerksegment,
nicht nur das, an das Sie denken, und ein Beacon tut das immer weiter. Prüfen Sie
zuerst, in welchem Netzwerk Sie sind, und senden Sie nur an Netzwerke, die Ihnen
gehören oder die Sie testen dürfen. Die Grenzen unten sind Leitplanken, keine
Erlaubnis.
:::

## Senden {#send}

1. Wählen Sie den [[ui:bc.mode]] (unten).
2. Geben Sie das Ziel ein: Der Name des Feldes ändert sich mit dem Modus. In einem
   Netzwerk mit einer IPv4-Adresse füllt [[ui:bc.useSubnet]] es aus der Adresse
   dieses Computers.
3. Wählen Sie die [[ui:bc.payload]] und schreiben Sie sie.
4. Drücken Sie [[ui:bc.sendOnce]]: Ein Datagramm geht an jedes Ziel.

### Modi {#modes}

| [[ui:bc.mode]] | Ziel | Was geschieht | Standard |
| --- | --- | --- | --- |
| [[ui:bc.mode.list]] | [[ui:bc.targetList]]: Einträge `IP:port` oder `host:port`, getrennt durch Kommas, Semikolons oder Zeilenumbrüche — ein Leerzeichen trennt sie nicht. Ein Name wird nachgeschlagen, seine IPv4-Adresse genommen, wenn er eine hat. | Ein Datagramm an jedes | `127.0.0.1:9000, 127.0.0.1:9001` |
| [[ui:bc.mode.broadcast]] | [[ui:bc.targetAddress]]: `255.255.255.255:port` oder eine Adresse, die auf `.255` endet | Ein Datagramm, das jeder Host im lokalen Netzwerk empfängt. Router reichen es nicht weiter. | `255.255.255.255:9000` |
| [[ui:bc.mode.multicast]] | [[ui:bc.targetAddress]]: eine Gruppe von `224.0.0.0` bis `239.255.255.255`, mit einem Port | Ein Datagramm an die Gruppe; nur Empfänger, die ihr beigetreten sind, erhalten es | `239.1.1.1:9000` |
| [[ui:bc.mode.sweep]] | [[ui:bc.targetCidr]], `a.b.c.d/nn`, und ein [[ui:bc.port]] | Ein Datagramm an jeden nutzbaren Host des Blocks, als Unicast — für Geräte, die Broadcast ignorieren | `192.168.1.0/24`, Port 9000 |

Bei einem Sweep werden die Netzwerk- und die Broadcast-Adresse übersprungen
(außer bei einem `/31` oder `/32`), und eine Basis, die nicht die des Netzwerks
selbst ist, wird darauf abgerundet: `192.0.2.77/30` überstreicht `192.0.2.77`
und `192.0.2.78`. Ein Sweep erreicht höchstens 1024 Hosts, der breiteste Block
ist also ein `/22` (1022 Hosts); ein breiterer wird mit dem Präfix abgelehnt, auf
das er zu verengen ist.

Broadcast, Multicast-Gruppen, Sweeps und die Beitritte des Erkennungsempfängers
gibt es nur über IPv4: IPv6 kennt keinen Broadcast. Eine [[ui:bc.mode.list]] darf
auch IPv6-Hosts nennen (siehe [Socket-Optionen](#socket-options)).

[[ui:bc.useSubnet]] füllt `x.y.z.10:9000, x.y.z.11:9000` in
[[ui:bc.mode.list]], `x.y.z.255:9000` in [[ui:bc.mode.broadcast]] und
`x.y.z.0/24` in [[ui:bc.mode.sweep]], aus der Adresse `x.y.z.w` dieses
Computers. Es setzt ein `/24`-Netzwerk voraus.

### Nutzdaten {#payload}

| [[ui:bc.payload]] | Was gesendet wird |
| --- | --- |
| [[ui:bc.payload.osc]] | Eine OSC-Nachricht: [[ui:common.address]] und typisierte [[ui:common.arguments]], wie in der Ansicht [OSC](osc.md). Standardmäßig `/hello/discover` mit dem Text `who-is-there`. |
| [[ui:bc.payload.text]] | Der [[ui:bc.text]] als UTF-8, genau wie eingegeben, ohne Abschlusszeichen. Standardmäßig `HELLO-PROBE`. |
| [[ui:bc.payload.hex]] | Das [[ui:bc.hex]] Byte für Byte — um einen mitgeschnittenen Frame erneut zu senden oder ein binäres Erkennungsprotokoll zu sprechen. Hex-Ziffernpaare; alles andere dazwischen wird ignoriert. Standardmäßig `48 45 4c 4c 4f`. |

### Socket-Optionen {#socket-options}

[[ui:bc.socketOptions]] öffnet drei weitere Einstellungen:

| Option | Was | Standard |
| --- | --- | --- |
| [[ui:bc.bindSource]] | Das lokale `IP:port`, von dem die Datagramme ausgehen. Legen Sie es fest, um die Netzwerkkarte zu wählen oder einen Quellport, auf den ein Gerät antwortet. `0.0.0.0:0`: beliebig. | `0.0.0.0:0` |
| [[ui:bc.ttl]] | Wie viele Router ein Datagramm überqueren darf, 1–255. Bei Multicast ist das das Multicast-Hop-Limit: 1 hält es in diesem Netzwerk. | `1` |
| [[ui:bc.mcastLoop]] | Nur Multicast: Die Datagramme der Gruppe auch an diesen Computer zustellen, damit ein Empfänger hier sie hört | an |

Mit dem Standard-[[ui:bc.bindSource]] gehen die Datagramme von einem
IPv4-Socket aus, oder von einem IPv6-Socket, wenn jedes Ziel IPv6 ist. Eine
Liste, die beides mischt, wird vom IPv4-Socket gesendet, und ihre IPv6-Ziele
scheitern; senden Sie sie als eigene Liste, oder legen Sie [[ui:bc.bindSource]]
auf eine IPv6-Adresse fest.

### Das Ergebnis {#result}

[[ui:bc.lastEmit]] zeigt, was hinausging: [[ui:bc.targets]],
[[ui:common.packets]], [[ui:common.volume]] und [[ui:common.errors]], und die
ersten acht Ziele, die es erreichte („… +n mehr“ für den Rest). Ein Fehler an
einem Ziel hält die anderen nicht auf; die Zeile der Konsole nennt, wie viele
scheiterten.

## Als Beacon wiederholen {#beacon}

Ein Beacon sendet dieselbe Runde — ein Datagramm an jedes Ziel — nach einem
Zeitplan, bis Sie ihn stoppen. Geräte, die auf eine regelmäßige Ankündigung
hören, brauchen einen.

1. Richten Sie Modus, Ziel und Nutzdaten ein wie für ein einzelnes Senden.
2. Stellen Sie unter [[ui:bc.beacon]] die [[ui:bc.beaconRate]] ein und, wenn er
   von selbst enden soll, [[ui:bc.beaconRounds]] oder [[ui:bc.beaconSeconds]].
3. Drücken Sie [[ui:bc.startBeacon]]. [[ui:bc.stopBeacon]] — oder das Stoppen
   seines Jobs in der Leiste der Konsole — beendet ihn.

| Feld | Was | Standard |
| --- | --- | --- |
| [[ui:bc.beaconRate]] | Runden pro Sekunde; muss über 0 liegen | `2` |
| [[ui:bc.beaconRounds]] | Nach so vielen Runden stoppen; 0 — keine Grenze | `0` |
| [[ui:bc.beaconSeconds]] | Nach so vielen Sekunden stoppen; 0 — keine Grenze | `0` |

Die Rate mal die Anzahl der Ziele darf höchstens 50 000 Datagramme pro Sekunde
betragen. Ein Sweep über ein `/24` (254 Hosts) kann sich also höchstens etwa
196-mal pro Sekunde wiederholen. Während der Beacon läuft, zeigt
[[ui:bc.lastEmit]] [[ui:bc.targets]] (die Ziele in jeder Runde, ab dem ersten
Bericht), die Summen, [[ui:bc.rounds]] und [[ui:bc.pps]] (Datagramme pro
Sekunde), viermal pro Sekunde aktualisiert, und [[ui:bc.sendOnce]] steht nicht
zur Verfügung. Ein Beacon, der mehr als 32 fehlgeschlagene Datagramme und kein
einziges gesendetes hatte — keine Route, Broadcast nicht erlaubt —, hält von
selbst an und sagt, warum.

## Auf Geräte hören {#discovery}

Die [[ui:bc.discovery]] belegt einen UDP-Port und zeichnet jeden Peer auf, der an
ihn sendet: was auf eine Anfrage antwortet, oder was ein Gerät von selbst
ankündigt.

1. Setzen Sie [[ui:common.bind]], den Port, an den die Geräte senden.
2. Für Multicast listen Sie die Gruppen in [[ui:bc.joinGroups]] auf.
3. Drücken Sie [[ui:bc.startListen]]. Die Einstellungen sind gesperrt, bis Sie
   [[ui:bc.stopListen]] drücken.

| Feld | Was | Standard |
| --- | --- | --- |
| [[ui:common.bind]] | Wo gehört wird, `IP:port`. `0.0.0.0` hört auf jeder Netzwerkkarte. | `0.0.0.0:9000` |
| [[ui:bc.joinGroups]] | IPv4-Multicast-Gruppen, denen beigetreten wird, kommagetrennt; leer — nur Unicast und Broadcast | `239.1.1.1` |
| [[ui:bc.interface]] | Die IPv4-Adresse der Netzwerkkarte, auf der den Gruppen beigetreten wird; leer — das System wählt | leer |
| [[ui:bc.reuse]] | Auf einem Port hören, den auch ein anderes Programm verwendet (`SO_REUSEADDR`). Es funktioniert nur, wenn dieses Programm das Teilen ebenfalls erlaubt. | an |
| [[ui:bc.respond]] | Auf Anfragen antworten, wie ein Gerät es täte ([unten](#auto-reply)) | aus |

### Peers {#peers}

[[ui:bc.peers]] listet auf, wer etwas gesendet hat, das Neueste zuerst, mit der
Anzahl der gesehenen Peers, der gehörten Pakete und der gesendeten Antworten
darüber:

| Spalte | Was |
| --- | --- |
| [[ui:bc.peer]] | Das `IP:port` des Absenders; sein Punkt zeigt, ob es in den letzten 3 Sekunden gehört wurde |
| [[ui:bc.proto]] | `osc`, wenn sein letztes Datagramm als OSC dekodiert wurde, sonst `udp` |
| [[ui:common.packets]] | Wie viele es gesendet hat |
| [[ui:bc.age]] | Sekunden seit seinem letzten Datagramm |
| [[ui:bc.lastMessage]] | Sein letztes Datagramm: die OSC-Adresse und die Argumente oder der Anfang des Textes |

Die Liste wird ein paar Mal pro Sekunde aktualisiert und hält bis zu 512 Peers;
darüber hinaus werden Pakete weiter gezählt, aber neue Peers erhalten keine
Zeile.

### Auf Anfragen antworten {#auto-reply}

Mit [[ui:bc.respond]] spielt der Empfänger ein Gerät: Er beantwortet jedes
Datagramm, das er empfängt, vom Empfangsport zurück an Adresse und Port des
Absenders.

| Feld | Was | Standard |
| --- | --- | --- |
| [[ui:bc.payload]] | Die Antwort: OSC, Text oder Hex, wie beim Senden | OSC `/hello/here` mit dem Text `signal-lab` |
| [[ui:bc.replyDelay]] | So lange vor dem Antworten warten, wie es ein langsames Gerät täte | `0` |
| [[ui:bc.matchContains]] | Nur Datagramme beantworten, deren dekodierter Text dies enthält — die OSC-Adresse und die Argumente oder der Anfang des Textes; leer — jedes | leer |

Der Empfänger beantwortet nie ein Datagramm, das mit seiner eigenen Antwort
identisch ist, sodass zwei aufeinander gerichtete Empfänger nicht endlos hin und
her antworten.

### Firewalls und geteilte Ports {#firewall}

Broadcast- und Multicast-Verkehr von anderen Rechnern wird von den meisten
Windows-Firewalls standardmäßig blockiert: Lassen Sie Signal Lab in privaten
Netzwerken zu, wenn die App es anbietet. Broadcast überquert nie einen Router. Um
auf einem Port zu hören, den der echte Dienst bereits hat, müssen beide Seiten
das Teilen erlauben ([[ui:bc.reuse]] hier); ohne das wird ein belegter Port mit
dem Hinweis abgelehnt, es einzuschalten. Siehe
[Fehlersuche](../reference/troubleshooting.md).

## Im Inspektor {#inspector}

Ist der Mitschnitt gestartet, erscheint der Verkehr der Ansicht je nach
Nutzdaten mit dem Protokoll `osc` oder `udp`:

| Quelle | Was | Wie viele |
| --- | --- | --- |
| `broadcast` | [[ui:bc.sendOnce]]: jedes Datagramm, mit dem Urteil `fan-out`, `broadcast`, `multicast` oder `sweep`; ein fehlgeschlagenes mit `error: …` | jedes einzelne |
| `beacon` | Die Runden eines Beacons | höchstens eine Runde alle 50 ms |
| `discovery` | Datagramme, die der Empfänger empfängt | höchstens eines alle 40 ms |
| `discovery` | Seine Antworten, mit dem Urteil `auto-reply` | jede einzelne |

Was der Inspektor von einem Beacon oder dem Empfänger auslässt, wird gezählt:
Der nächste Frame, den er zeichnet, trägt die Zahl in seinem Urteil,
`+n not shown` (bei einem Beacon ein Frame für jedes ausgelassene Ziel jeder
Runde). Siehe [Inspektor](../tools/inspector.md).

## Auf einem Server oder in Docker {#server}

Auf einem [Server](../server/index.md) sendet und hört die Ansicht im Netzwerk
des Servers. In Docker erreichen Broadcast, Multicast und die Erkennung das
lokale Netzwerk nur, wenn der Container auf einem Linux-Host das Netzwerk des
Hosts verwendet (`--network host`). Mit dem standardmäßigen Bridge-Netzwerk von
Docker oder mit Docker Desktop funktioniert nur Unicast an Hosts, die der
Container erreicht.

## Anderswo {#elsewhere}

- [`signallab send udp`](../automation/cli.md#cli-send-udp) sendet ein Datagramm
  an einen Host; es gibt keinen Broadcast, Multicast oder Sweep auf der
  Kommandozeile.
- Ein Schritt [[ui:exp.node.udp]] sendet ein Text-Datagramm an einen oder mehrere
  Hosts, und ein Schritt [[ui:exp.node.wait_udp]] wartet auf eines. Siehe
  [UDP und TCP](udp-tcp.md).
- Um ein Gerät zu spielen, das nach Regeln antwortet — mehrere Regeln, Antworten
  aus dem Eingetroffenen gebaut —, verwenden Sie einen Emulator
  [[ui:emu.new.udp]] oder [[ui:emu.new.osc]]. Siehe
  [Emulatoren](../tools/emulators.md).

## Probleme {#troubleshooting}

| Was Sie sehen | Übliche Ursache |
| --- | --- |
| `… is not a broadcast address` | [[ui:bc.mode.broadcast]] nimmt `255.255.255.255:port` oder eine Adresse, die auf `.255` endet. Für eine andere Subnetzmaske verwenden Sie [[ui:bc.mode.sweep]]. |
| `… is not a multicast group` | Die Adresse liegt außerhalb von `224.0.0.0`–`239.255.255.255`. |
| `… spans … addresses, and a sweep reaches at most 1024 hosts` | Der Block ist breiter als ein `/22`; verengen Sie ihn. |
| `Set the port to sweep` | [[ui:bc.port]] ist 0. |
| `… is over the … pps limit` | Die Rate mal die Anzahl der Ziele liegt über 50 000 pro Sekunde: Senken Sie die [[ui:bc.beaconRate]] oder verengen Sie das Ziel. |
| `… is already in use — turn on “share the port” to listen alongside it` | Ein anderes Programm hat den Port; haken Sie [[ui:bc.reuse]] an. |
| `Cannot join the multicast group …` | Die Gruppe oder [[ui:bc.interface]] ist auf diesem Computer nicht nutzbar — keine Netzwerkkarte mit dieser Adresse oder keine Multicast-Route. |
| Gesendet, aber niemand antwortet | Die Geräte hören auf einem anderen Port; die Firewall hier hält ihre Antworten fern; ein Router liegt dazwischen; oder, in Docker, der Container ist nicht im Netzwerk des Hosts. |
| Anfragen gehen hinaus, aber der Erkennungsempfänger hört keine Antworten | Viele Geräte antworten an die Adresse und den Port, von dem eine Anfrage kam — den eigenen Socket des Senders, den die Ansicht nicht liest. Hören Sie auf dem Port, an den die Geräte antworten, oder senden Sie die Anfrage aus einem Experiment: Ein Schritt [[ui:exp.node.udp]] mit [[ui:exp.expectReply]] sendet und hört auf demselben Port. |

Jede Fehlermeldung steht unter [Fehlermeldungen](../reference/errors.md#broadcast).

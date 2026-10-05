---
title: UDP und TCP
description: Wo Signal Lab rohe UDP-Datagramme und TCP-Daten sendet und empfängt — Schritte in Experimenten, Signale, die Kommandozeile, die Werkzeuge Last und Scan und emulierte Geräte — und wie Nutzdaten geschrieben werden.
---

# UDP und TCP

Rohes UDP und TCP haben keine eigene Ansicht. Sie sind das, was Sie für ein
Gerät mit eigenem Text- oder Binärprotokoll verwenden — einen Projektor, einen
Medienserver, einen Sensor — und kommen an mehreren Stellen vor:

| Um… | Verwenden Sie |
| --- | --- |
| ein Datagramm oder eine TCP-Nachricht als Schritt zu senden und auf die Antwort zu warten | die [Schritte im Experiment](#experiments) |
| ein Datagramm zu behalten, um es erneut zu senden, oder ein mitgeschnittenes wiederzugeben | ein [UDP-Signal](#signals) |
| ein Datagramm aus einem Skript zu senden | [`signallab send udp`](#cli) |
| auf einmal an viele Hosts zu senden, an eine Broadcast-Adresse oder eine Multicast-Gruppe | die Ansicht [Broadcast](broadcast.md) |
| einen Server oder eine Leitung mit Verkehr zu belasten | [Sturm](#storm) |
| herauszufinden, welche TCP-Ports ein Host offen hat | [Scanner](#scanner) |
| die Seite des Geräts zu spielen | ein Emulator für ein [UDP- oder TCP-Gerät](#emulators) |
| das Netzwerk zwischen zwei Enden zu verschlechtern | ein [Störungs-Relais](#impairment) |

OSC ist ein Format, das in UDP-Datagrammen übertragen wird; es hat seine eigene
Seite: [OSC](osc.md).

## Nutzdaten {#payloads}

Wo immer Sie rohe Nutzdaten schreiben, sind sie von einer von zwei Arten:

| Art | Was gesendet wird | Beispiel |
| --- | --- | --- |
| Text | Die Zeichen als UTF-8, genau wie eingegeben: kein Abschluss, kein Zeilenende angehängt. Ein Zeilenprotokoll braucht sein Zeilenende im Text. | `PING` |
| Hex-Bytes | Byte für Byte, geschrieben als Paare von Hex-Ziffern. Leerzeichen, `:`, `-` und `,` zwischen den Paaren und ein `0x` davor sind erlaubt. | `de ad be ef`, `DEADBEEF`, `0xde,0xad` |

Ein Datagramm trägt höchstens 65.507 Bytes. Eine ungerade Anzahl Hex-Ziffern
oder gar keine ist ein Fehler, bevor etwas gesendet wird.

::: info
UDP kennt keine Empfangsbestätigung. „Gesendet" heißt, dass das Datagramm diesen
Computer verlassen hat, nicht, dass es jemand empfangen hat. Um zu wissen, dass
ein Gerät Sie gehört hat, warten Sie auf seine Antwort.
:::

## Wohin es geht {#destinations}

Ein Ziel ist eine IP-Adresse und ein Port oder ein Hostname und ein Port:
`192.0.2.20:9000`, `[2001:db8::20]:9000` (eine IPv6-Adresse steht in eckigen
Klammern), `projector.local:9000`. Das gilt für das Ziel eines UDP-Schritts, ein
OSC-Ziel, ein UDP-Signal, `signallab send udp` und `signallab send osc`, die
Liste von [Broadcast](broadcast.md), das Ziel von
[Sturm](../tools/storm.md) und den Host des TCP-Schritts.

Ein Hostname wird bei jeder Verwendung nachgeschlagen. Hat er eine IPv4-Adresse,
wird diese verwendet — so erreicht `localhost` einen Dienst, der auf
`127.0.0.1` empfängt, wo die erste Adresse, die ein System für ihn auflistet,
`::1` sein mag —, und ein Name, der nur IPv6-Adressen hat, wird über einen
IPv6-Socket erreicht. Ein Name, der sich nicht auflösen lässt, scheitert mit
`Cannot resolve …`; ein Ziel ohne Port oder das keiner der beiden Formen
entspricht, mit `… is not a valid address`. Adressen, auf denen ein Dienst
*empfängt* (die eines Monitors, eines Warteknotens, eines Emulators), sind immer
`IP:port`.

## In Experimenten {#experiments}

| Schritt | Was er tut |
| --- | --- |
| [[ui:exp.node.udp]] | Sendet seine [[ui:exp.payload]] als Text an [[ui:common.target]]. Das Ziel ist `IP:port` oder `host:port` ([oben](#destinations)), und mehrere durch Kommas getrennte Ziele erhalten jeweils das Datagramm. Mit [[ui:exp.expectReply]] sendet er von [[ui:exp.replyOn]] und wartet dort im selben Schritt auf eine Antwort. [Details](../experiments/nodes.md#node-udp) |
| [[ui:exp.node.wait_udp]] | Empfängt auf [[ui:exp.listenOn]] (`IP:port`) und wartet auf ein Datagramm, dessen Nutzdaten passen. [Details](../experiments/nodes.md#node-wait_udp) |
| [[ui:exp.node.tcp]] | Verbindet sich mit [[ui:exp.host]] und [[ui:exp.port]], schreibt seine [[ui:exp.payload]] als Text, empfängt 250 ms lang auf eine Antwort und schließt. Der Schritt sagt, wie viele Bytes zurückkamen, nicht, was sie waren. [Details](../experiments/nodes.md#node-tcp) |

Die UDP-Nutzdaten und das UDP-Ziel, der TCP-Host und die TCP-Nutzdaten sowie das
Muster eines Warteknotens nehmen `{{templates}}`, sodass ein Datagramm die ID
des Durchlaufs oder einen Wert tragen kann, den ein früherer Schritt extrahiert
hat. Siehe [Daten und Vorlagen](../experiments/data.md).

### Ein Datagramm auswählen {#matching}

[[ui:exp.node.wait_udp]] und die Antwort von [[ui:exp.node.udp]] wählen ein
Datagramm anhand seiner Nutzdaten aus:

| [[ui:exp.waitMode]] | Nimmt ein Datagramm an, wenn |
| --- | --- |
| [[ui:exp.mode.any]] | immer: das erste, das eintrifft |
| [[ui:exp.mode.contains]] | seine als Text gelesenen Nutzdaten das Muster enthalten |
| [[ui:exp.mode.regex]] | seine als Text gelesenen Nutzdaten auf den regulären Ausdruck passen |
| [[ui:exp.mode.hex]] | seine Bytes die Bytes des in Hex geschriebenen Musters enthalten |

Was gepasst hat, bleibt in der Variablen des Schritts ([[ui:exp.replyVariable]],
`reply`, wenn Sie sie nicht umbenennen): `text`, `hex` (die ersten 1024 Bytes),
`bytes` (die Größe), `from` (`IP:port` des Absenders), `ms` (wie lange es
dauerte) und `match` (der gefundene Text oder die Bytes oder die erste Gruppe
eines regulären Ausdrucks).

Ein Warteknoten beginnt zu empfangen, wenn der Durchlauf startet, nicht wenn der
Schritt erreicht wird, sodass eine sehr schnelle Antwort nicht verpasst wird. Er
nimmt nur, was nach dem letzten Senden auf seinem Pfad eingetroffen ist.

### Grenzen und Voreinstellungen {#limits}

| Einstellung | Standard | Bereich |
| --- | --- | --- |
| TCP-Schritt: [[ui:common.timeoutMs]] (Verbinden, Schreiben und die Antwort zusammen) | 4000 ms | 1–120.000 ms |
| [[ui:exp.waitTimeout]] eines Warteknotens oder einer Antwort | 2000 ms | 1–120.000 ms |
| UDP-Nutzdaten | — | höchstens 65.507 Bytes |
| Empfangsadresse | — | `IP:port` mit einem Port; [[ui:exp.replyOn]] einer Antwort darf Port 0 verwenden (irgendein freier Port) |

Im Inspektor erscheint das Datagramm eines UDP-Schritts mit der Quelle
`broadcast` (oder `experiment`, wenn der Schritt auf eine Antwort wartet), und
jedes an einem Port eines Warteknotens eintreffende Datagramm mit der Quelle
`experiment-wait`. Ein TCP-Schritt erscheint als zwei `tcp`-Frames mit der
Quelle `experiment`: die Nutzdaten, die er geschrieben hat, und, wenn eine kam,
die Antwort, die er gelesen hat. Sein Eintrag in der Zeitleiste sagt, was er
gesendet hat und wie groß die Antwort war.

## Signale {#signals}

Ein Signal [[ui:sig.tr.udp]] in der Bibliothek ist ein Ziel und Nutzdaten, als
Text oder Hex-Bytes. Senden Sie es aus [[ui:nav.signals]] oder mit
<kbd>Ctrl</kbd>+<kbd>K</kbd> von jeder Ansicht aus. Sein Ziel darf ein Hostname
sein.

Jedes Datagramm, das der Inspektor vollständig behalten hat, kann eines werden:
[[ui:sig.fromFrame]] legt ein Hex-UDP-Signal im Ordner [[ui:sig.capturedFolder]]
an, das genau diese Bytes erneut sendet — an das Ziel des Frames, wenn der Frame
gesendet wurde, an die Adresse, die ihn empfangen hat, wenn er eingetroffen ist
(auf diesem Computer, wenn das jede Adresse war). Ein TCP-Block kann das nicht:
Er ist ein Stück eines Datenstroms. Siehe [Signale](../tools/signals.md) und
[Inspektor](../tools/inspector.md#save-as-signal).

Ein Text-UDP-Signal kann einem Experiment als Schritt [[ui:exp.node.udp]]
hinzugefügt werden; ein Hex-Signal nicht, weil der Schritt Text sendet.

## Von der Kommandozeile {#cli}

`signallab send udp` sendet ein Datagramm:

```bash
signallab send udp 127.0.0.1:9000 --text "PING"
signallab send udp 127.0.0.1:9000 --hex "de ad be ef"
```

```text
✔ sent 4 bytes → 127.0.0.1:9000
```

Geben Sie genau eines von `--text` und `--hex` an. Das Ziel ist `IP:port` oder
`host:port` ([oben](#destinations)). Es endet mit 0, wenn das Datagramm
hinausging, mit 1, wenn das Senden fehlschlug, und mit 2, wenn das Ziel oder die
Hex-Bytes ungültig sind. Ein `send tcp` gibt es nicht. Siehe
[Kommandozeile](../automation/cli.md#cli-send-udp).

## Sturm {#storm}

[[ui:nav.storm]] ist eine Lastquelle für Ihre eigenen Server und Leitungen:
[[ui:st.udp]] sendet Datagramme fester Größe mit einer festen Rate, [[ui:st.tcp]]
öffnet eine Verbindung, schreibt die Nutzdaten und schließt, immer wieder. Der
Durchsatz wird live gemessen. Siehe [Sturm](../tools/storm.md).

## Scanner {#scanner}

[[ui:nav.scan]] versucht eine TCP-Verbindung zu jedem Port in einem Bereich und
listet die auf, die annehmen, samt dem, was der Dienst zuerst sagt, wenn Sie
Banner anfordern. Siehe [Scanner](../tools/scanner.md).

::: danger
Sturm und Scanner senden echten Verkehr an echte Hosts. Richten Sie sie nur auf
Systeme, die Ihnen gehören oder deren Test Ihnen erlaubt ist: Ein Sturm kann
eine Leitung sättigen, und beide können eine Angriffserkennung auslösen.
:::

## Emulierte Geräte {#emulators}

In der Ansicht [[ui:nav.emulators]] kann Signal Lab das Gerät sein:

- ein [[ui:emu.new.udp]] beantwortet Datagramme nach Regeln zu ihren Nutzdaten —
  beliebig, einen Text enthaltend, auf einen regulären Ausdruck passend, Bytes
  enthaltend — mit einer Text- oder Hex-Antwort, aus dem Eingetroffenen gebaut,
  an den Absender oder an ein anderes `IP:port`, nach einer Verzögerung, wenn
  Sie eine festlegen;
- ein [[ui:emu.new.tcp]] nimmt Verbindungen an, teilt das Eintreffende an einem
  von Ihnen gewählten Zeilenende in Nachrichten (LF, CR LF, CR oder jeden Block,
  wie er kommt), beantwortet jede nach denselben Regeln, kann eine Begrüßung
  senden, wenn sich ein Client verbindet, und die Verbindung nach einer Antwort
  schließen.

Beide können auch als Schritt [[ui:exp.node.emulator]] für die Dauer eines
Durchlaufs laufen. Siehe [Emulatoren](../tools/emulators.md).

## Störung {#impairment}

Das Relais [[ui:nav.netsim]] sitzt zwischen einem Client und seinem Ziel und
verschlechtert, was passiert: pro Datagramm über UDP (Verzögerung, Verlust,
Duplikate, Umordnung, eine Bandbreitengrenze) oder pro Datenstrom über TCP
(Verzögerung, eine Bandbreitengrenze, Verbindungen zurückgesetzt oder halboffen
gelassen). Siehe [Störung](../tools/impairment.md) und, in einem Experiment,
[Störungen](../experiments/faults.md).

## „Port unreachable" unter Windows {#port-unreachable}

Wenn ein Datagramm einen Port erreicht, auf dem nichts empfängt, antwortet der
empfangende Rechner üblicherweise mit einer ICMP-Nachricht „Port unreachable".
Windows meldet diese Antwort beim nächsten Empfang des sendenden Sockets, als
wäre die Verbindung zurückgesetzt worden — obwohl UDP keine Verbindung kennt.

Signal Lab rechnet damit. Seine Empfänger — der OSC-Monitor, der
Erkennungsempfänger, die Warteknoten und Antworten von Experimenten, UDP- und
OSC-Emulatoren, das Störungs-Relais — bemerken es und empfangen weiter. Ein
Gerät, das verschwunden ist, hält sie nicht auf. Ein Empfänger, der wirklich
nichts mehr empfangen kann, beendet seinen Job, und die Konsole sagt, warum.

## Probleme {#troubleshooting}

| Was Sie sehen | Übliche Ursache |
| --- | --- |
| `… is not a valid address` | Das Ziel hat keinen Port oder ist weder `IP:port` noch `host:port`. |
| `Cannot resolve …` | Der Hostname lässt sich auf diesem Computer nicht auflösen. |
| `… refused the connection — nothing is listening on that port` (TCP) | Auf diesem Port empfängt nichts, oder eine Firewall weist ihn ab. |
| `No answer from … in time` (TCP) | Der Host antwortet überhaupt nicht — falsche Adresse oder eine Firewall, die verwirft statt abzulehnen. |
| Ein Warteknoten läuft in ein Timeout, obwohl das Gerät antwortet | Das Gerät antwortet an den Port, von dem das Datagramm kam, nicht an den Port des Warteknotens. Lassen Sie das Senden mit [[ui:exp.expectReply]] selbst auf die Antwort warten: Dann geht es von dem Port aus, an den die Antwort zurückkommt. |
| Datagramme von anderen Rechnern kommen nie an | Unter Windows kann die Firewall sie abhalten: Lassen Sie Signal Lab zu, wenn die App es anbietet. Siehe [Problemlösung](../reference/troubleshooting.md). |

Jede Fehlermeldung steht unter
[Fehlermeldungen](../reference/errors.md#transport).

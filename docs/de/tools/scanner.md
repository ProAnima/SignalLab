---
title: Scanner
description: Herausfinden, welche TCP-Ports eines Hosts Verbindungen annehmen, mit der Begrüßung, die jeder Dienst sendet, über einen parallelen TCP-Connect-Scan.
---

# Scanner

Die Ansicht [[ui:nav.scan]] versucht, zu jedem Port eines Bereichs auf einem Host eine
TCP-Verbindung zu öffnen, und listet die Ports auf, die annehmen. Nutzen Sie sie, um zu
bestätigen, welche Dienste tatsächlich auf einem Gerät empfangen — der Steuerport eines
Projektors, die Weboberfläche eines Switches, der Port, den Ihr eigener Dienst öffnen sollte —,
und bei Diensten, die zuerst grüßen, was sie sagen.

::: danger Verantwortungsvoller Gebrauch
Ein Scan verbindet sich mit jedem Port des Bereichs. Scannen Sie nur Hosts, die Ihnen gehören
oder für deren Test Sie autorisiert sind: In anderen Netzwerken kann ein Scan
Intrusion-Detection auslösen und ist oft gegen die Regeln.
:::

## Einen Host scannen {#scan}

1. Geben Sie den [[ui:sc.host]] ein: ein Host, eine IP-Adresse oder ein Name. Ein Name
   wird einmal nachgeschlagen, wenn der Scan startet; einer, der sich nicht auflösen
   lässt, hält ihn dort an, mit dem Grund.
2. Setzen Sie [[ui:sc.fromPort]] und [[ui:sc.toPort]] oder wählen Sie eine [[ui:sc.preset]].
3. Passen Sie [[ui:common.concurrency]] und [[ui:common.timeoutMs]] an, wenn nötig.
4. Lassen Sie [[ui:sc.grabBanner]] an, um zu lesen, was jeder Dienst als Erstes sagt.
5. Drücken Sie [[ui:sc.startScan]].

Eine Leiste unter der Schaltfläche zeigt, wie viele Ports von wie vielen versucht wurden und
wie viele offen sind. Offene Ports erscheinen rechts, sobald sie gefunden werden. Der Scan
endet, wenn jeder Port versucht wurde; [[ui:sc.stopScan]] beendet ihn früher, und noch nicht
versuchte Ports werden nicht versucht. Die Felder sind fest, solange er läuft.

| Feld | Was | Standard |
| --- | --- | --- |
| [[ui:sc.host]] | Der zu scannende Host. | `127.0.0.1` |
| [[ui:sc.fromPort]], [[ui:sc.toPort]] | Der Bereich, beide Enden eingeschlossen, 1–65 535: Eine Zahl, die über ein Ende hinaus getippt wird, hält dort an. Ist der erste größer, werden sie getauscht. | 1–1024 |
| [[ui:common.concurrency]] | Wie viele Verbindungsversuche gleichzeitig offen sind, 1–1024. | 400 |
| [[ui:common.timeoutMs]] | Wie lange auf jede Verbindung gewartet wird, 50–10 000 ms. | 500 |
| [[ui:sc.grabBanner]] | Nach dem Verbinden bis zu 400 ms darauf warten, dass der Dienst etwas sendet, und seine ersten 256 Bytes behalten. | an |

Ein [[ui:common.concurrency]] oder [[ui:common.timeoutMs]] außerhalb seines Bereichs wird
beim Start des Scans in ihn gebracht.

[[ui:sc.preset]] füllt den Bereich:

| Vorgabe | Ports |
| --- | --- |
| [[ui:sc.preset.wellKnown]] | 1–1024 |
| [[ui:sc.preset.common]] | 1–10 000 |
| [[ui:sc.preset.osc]] | 8000–9100 |
| [[ui:sc.preset.full]] | 1–65 535 |

## Wie lange ein Scan dauert {#duration}

Ein Port, der annimmt, antwortet sofort. Ein Port, der nicht annimmt, kann bis zum ganzen
Timeout kosten: Eine Firewall, die Verbindungsversuche verwirft, antwortet nie, und unter
Windows kann sogar eine Verweigerung länger dauern als der Standard-Timeout. Ein Scan eines
Hosts, der gar nichts beantwortet, dauert also etwa (unter Windows braucht ein
verweigerter Port etwa 2 s, weil Windows es erneut versucht, dort entscheidet also der
Timeout):

```text
ports ÷ concurrency × timeout
```

Der ganze Bereich mit den Standardwerten: 65 535 ÷ 400 × 0,5 s ≈ 82 s. Erhöhen Sie
[[ui:common.concurrency]] oder senken Sie [[ui:common.timeoutMs]], um schneller zu werden;
senken Sie den Timeout zu weit, werden offene Ports eines langsamen Hosts übersehen.

## Die Ergebnisse lesen {#results}

[[ui:sc.openPorts]] listet jeden Port auf, der eine Verbindung angenommen hat, in
Portreihenfolge:

| Spalte | Was |
| --- | --- |
| [[ui:sc.port]] | Der offene Port. |
| [[ui:common.time]] | Wann er gefunden wurde. |
| [[ui:sc.banner]] | Was der Dienst als Erstes gesendet hat, Zeilenumbrüche in Leerzeichen umgewandelt, oder `—`. |

Ein Port, der verweigert hat, und einer, der nicht innerhalb des Timeouts geantwortet hat,
bleiben beide aus: Der Scanner unterscheidet nicht zwischen geschlossen und gefiltert. Die
Liste behält jeden offenen Port, wie viele es auch sind.

Wenige offene Ports sind das übliche Ergebnis: Ein Windows-PC empfängt unter den ersten
tausend Ports typischerweise auf 135 und 445 (und 139 auf seiner Netzwerkadresse), die
meisten Geräte auf einer Handvoll. Zum Vergleich listen Sie auf, worauf dieser Computer
empfängt, mit `netstat -an` (oder `Get-NetTCPConnection -State Listen` in PowerShell).

### Nicht geprüfte Ports {#unchecked}

Jede Verbindung belegt einen der eigenen Ports dieses Computers. Gehen sie aus — oder die
Sockets —, scheitert ein Versuch hier, bevor er den Host erreicht; der Scanner wartet und
versucht es erneut, und ein Port, den er dennoch nicht versuchen kann, wird als
*nicht geprüft* gezählt, in Bernstein neben der Zahl der offenen angezeigt, nie als
geschlossen. Sehen Sie das, scannen Sie erneut mit einer niedrigeren
[[ui:common.concurrency]].

Nur Dienste, die zuerst sprechen, haben ein Banner — SSH, SMTP, FTP, viele
Gerätesteuerungsprotokolle. Ein Webserver wartet auf eine Anfrage, daher zeigt sein Port `—`.
Das Auslesen von Bannern lässt jeden offenen Port bis zu 400 ms länger dauern.

Eine Verbindung, die der Scanner öffnet, wird sofort mit einem Reset wieder geschlossen,
sie hält also danach nicht minutenlang den Port dieses Computers. Der Scanner sendet
nichts darauf.

Mit eingeschaltetem Mitschnitt steht jeder offene Port auch im [Inspektor](inspector.md), mit
seinem Banner und dem Befund `open`.

## Verwandte Seiten {#related}

- [Sturm](storm.md) — Last auf einem gefundenen Port.
- [UDP und TCP](../protocols/udp-tcp.md) — mit ihm sprechen.
- [Fehlersuche](../reference/troubleshooting.md)

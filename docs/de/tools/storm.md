---
title: Sturm
description: Eine kontrollierte Flut von UDP-Datagrammen oder TCP-Verbindungen an einen Server senden, der Ihnen gehört, und Rate, Durchsatz und Fehler live beobachten.
---

# Sturm

Die Ansicht [[ui:nav.storm]] ist eine Lastquelle: Sie sendet UDP-Datagramme oder öffnet
TCP-Verbindungen auf ein Ziel, so schnell Sie verlangen, so lange Sie verlangen, und misst,
was tatsächlich hinausging. Nutzen Sie sie, um zu sehen, wie Ihr eigener Server, Ihr Gerät
oder Ihre Leitung einer Flut standhält — ob er weiter antwortet, Pakete verwirft oder umfällt.

::: danger Verantwortungsvoller Gebrauch
Ein Sturm sendet echten Verkehr an einen echten Host. Richten Sie ihn nur auf Hosts und
Netzwerke, die Ihnen gehören oder für deren Test Sie autorisiert sind. Hohe Raten können
Leitungen für alle darauf auslasten und Intrusion-Detection auslösen. Die Engine begrenzt die
Rate eines Sturms nicht: 0 bedeutet so schnell, wie dieser Computer senden kann.
:::

## Einen Sturm starten {#start}

1. Setzen Sie [[ui:common.target]]: das `IP:port` oder `host:port`, an das gesendet wird,
   etwa `127.0.0.1:9000` oder `test-rig.local:9000`. Ein Hostname wird beim Start
   nachgeschlagen, seine IPv4-Adresse genommen, wenn er eine hat.
2. Wählen Sie das [[ui:common.protocol]]: [[ui:st.udp]] oder [[ui:st.tcp]].
3. Setzen Sie [[ui:st.payloadSize]], [[ui:st.rate]] und [[ui:st.duration]].
4. Drücken Sie [[ui:st.launch]].

Der Sturm läuft als Job: Er erscheint in der Leiste der Konsole und stoppt, wenn seine Dauer
vorbei ist, wenn Sie [[ui:st.stop]] drücken oder mit [[ui:app.stopAll]]. Die Felder sind fest,
solange er läuft.

| Feld | Was | Standard |
| --- | --- | --- |
| [[ui:common.target]] | `IP:port` oder `host:port` des Ziels. | `127.0.0.1:9000` |
| [[ui:common.protocol]] | [[ui:st.udp]]: einzelne Datagramme. [[ui:st.tcp]]: eine neue TCP-Verbindung für jedes „Paket“. | [[ui:st.udp]] |
| [[ui:st.payloadSize]] | Bytes in jedem Datagramm oder jeder Verbindung, 1–65 507. Ein größerer oder kleinerer Wert wird in diesen Bereich gebracht. | 512 |
| [[ui:st.rate]] | Angestrebte Pakete (oder Verbindungen) pro Sekunde. 0: so schnell wie möglich. | 1000 |
| [[ui:st.duration]] | Sekunden, die er läuft. 0: bis Sie ihn stoppen. | 10 |

### UDP-Flut {#udp}

Signal Lab sendet Datagramme in der Nutzdatengröße, jedes Byte `0x55`, von einem eigenen Port
an das Ziel. Ein Senden, das das System verweigert, zählt als Fehler — zum Beispiel, nachdem
das Ziel geantwortet hat, dass auf diesem Port nichts empfängt.

### TCP-Verbindungsflut {#tcp}

Für jedes „Paket“ öffnet Signal Lab eine TCP-Verbindung zum Ziel, schreibt die Nutzdaten und
schließt sie. Verbindungen werden eine nach der anderen aufgebaut, nicht gleichzeitig, daher
ist die Rate, die ein TCP-Sturm erreicht, dadurch begrenzt, wie schnell das Ziel sie annimmt.
Eine Verbindung, die verweigert oder nicht innerhalb von 500 ms angenommen wird, zählt als
Fehler; eine Verbindung, die die Nutzdaten aufnahm, zählt als Paket.

### Wie die Rate gehalten wird {#pacing}

Die Rate ist ein Zeitplan. Das erste Paket geht sofort hinaus, und Paket *n* ist *n* ÷
[[ui:st.rate]] Sekunden nach dem Start fällig. Jedes Mal, wenn der Sturm aufwacht, sendet er,
was fällig ist, und schläft dann bis zum nächsten fälligen Paket. So sind 50 pro Sekunde 50
pro Sekunde und 250 sind 250 — ein Durchlauf von einer Sekunde sendet etwa so viele —, was
auch immer die Timergranularität des Systems ist. Eine [[ui:st.duration]] beendet den Sturm
pünktlich, auch wenn das nächste Paket später fällig wäre.

Ein Sturm sendet nie über seine Rate hinaus, um verlorene Zeit aufzuholen. Wenn dieser
Computer oder ein langsam annehmendes TCP-Ziel um mehr als 256 Pakete zurückfällt, werden die
älteren aus dem Zeitplan verworfen, statt spät in einem Schwung gesendet. [[ui:st.pps]]
zeigt, was erreicht wurde.

Mit [[ui:st.rate]] 0 gibt es keinen Zeitplan: Der Sturm sendet 256 Pakete, lässt andere
Arbeit laufen und sendet die nächsten 256, so schnell dieser Computer kann.

## Den Durchsatz lesen {#metrics}

[[ui:st.throughput]] aktualisiert sich viermal pro Sekunde:

| Messwert | Was |
| --- | --- |
| [[ui:common.packets]] | Gesendete Datagramme oder Verbindungen, die die Nutzdaten aufnahmen, seit dem Start. |
| [[ui:st.pps]] | Pakete pro Sekunde über die letzte Viertelsekunde. |
| [[ui:st.rateLabel]] | Megabits pro Sekunde an Nutzdaten über die letzte Viertelsekunde. Header (IP, UDP, TCP) werden nicht gezählt. |
| [[ui:common.volume]] | Seit dem Start gesendete Nutzdatenbytes. |
| [[ui:common.errors]] | Fehlgeschlagene Sendevorgänge oder Verbindungen. |

Das Diagramm darunter zeichnet [[ui:st.pps]] über die letzte Minute.

Wenn der Sturm endet, zeigen die Zähler die Endsummen, und [[ui:st.pps]] und
[[ui:st.rateLabel]] gehen auf 0.

Was die Zahlen Ihnen sagen:

- [[ui:st.pps]] deutlich unter [[ui:st.rate]] bei UDP: Dieser Computer kann nicht schneller
  senden. Senken Sie die Rate oder die Nutzdatengröße.
- [[ui:common.errors]] steigend bei UDP: Der Port des Ziels ist geschlossen oder das Netzwerk
  verweigert den Verkehr.
- [[ui:common.errors]] steigend bei TCP: Das Ziel verweigert Verbindungen oder braucht länger
  als 500 ms, um sie anzunehmen — es hat vielleicht seine Grenze erreicht.

Ein Sturm sagt, wie viel hinausging, nicht, wie viel ankam. Um zu sehen, was das Ziel
empfangen hat, beobachten Sie es: seine eigenen Logs, einen [[ui:nav.osc]]-Monitor oder einen
[Emulator](emulators.md) an seiner Stelle.

## Im Inspektor {#inspector}

Mit eingeschaltetem Mitschnitt legt ein UDP-Sturm jede Sekunde eines seiner Datagramme in den
[Inspektor](inspector.md), markiert mit `sampled 1/s` — sie sind alle gleich. Der Befund sagt
auch, wie viele seit dem vorherigen ausgelassen wurden, als `sampled 1/s · +999 not shown`.
Ein TCP-Sturm legt keines dort ab.

## Verwandte Seiten {#related}

- [Störung](impairment.md) — eine langsame oder verlustbehaftete Leitung statt einer Flut.
- [HTTP](../protocols/http.md) — Last-Bursts aus HTTP-Anfragen, mit Latenzen.
- [Last](../experiments/load.md) — HTTP-Last in einem Experiment, mit Schwellenwerten.

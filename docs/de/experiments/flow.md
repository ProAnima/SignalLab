---
title: Wie sich ein Durchlauf bewegt
description: Start und Ende, Ausgänge und Verbindungen, parallele Zweige und Join, Verzweigung, Neuversuch, Wiederholung und Schleife, Warteknoten, die vom Start des Durchlaufs an lauschen, und was vor einem Durchlauf geprüft wird.
---

# Wie sich ein Durchlauf bewegt

Ein Durchlauf beginnt bei [[ui:exp.node.start]], folgt Verbindungen von Knoten zu
Knoten und ist vollständig, wenn jeder Zweig fertig ist und [[ui:exp.node.end]]
erreicht wurde. Diese Seite erklärt die Regeln, denen er folgt; was jeder Knoten
tut, steht in [der Referenz der Knoten](nodes.md), und die Werte, die mit ihm
reisen, in [Daten](data.md).

## Start und Ende {#start-end}

Ein Experiment hat genau einen [[ui:exp.node.start]] und ein [[ui:exp.node.end]].

- [[ui:exp.node.start]] hat keinen Eingang. Er geht sofort weiter, und seine Zeile
  in der Zeitleiste gibt den Startwert des Durchlaufs. Sein Ausgang darf mehrere
  Verbindungen haben: Das Experiment beginnt dann mit parallelen Zweigen.
- Jeder Zweig, der [[ui:exp.node.end]] erreicht, endet dort. Das Ende zeigt sich als
  laufend ab dem ersten Eintreffen und geht einmal weiter, nachdem der letzte Zweig
  fertig ist — und gar nicht, wenn ein Schritt fehlgeschlagen ist. Jener Durchgang
  ist es, der den Durchlauf [[ui:exp.passed]] macht.
- Ein Durchlauf, in dem jeder Zweig ohne Fehler endete, aber keiner das Ende
  erreichte, schlägt mit `run.no_end` fehl.

## Ausgänge und Verbindungen {#outputs}

Der Schritt eines Knotens endet mit der Wahl eines Ausgangs, und der Durchlauf folgt
jeder Verbindung dieses Ausgangs. Die meisten Knoten haben einen Ausgang,
[[ui:exp.outputPort]]; manche wählen zwischen mehreren:

| Knoten | Ausgänge, die verdrahtet sein müssen | Ausgänge, die verdrahtet sein dürfen |
| --- | --- | --- |
| [[ui:exp.node.end]] | — | — |
| [[ui:exp.node.fork]] | [[ui:exp.branch1]], [[ui:exp.branch2]] | — |
| [[ui:exp.node.branch_status]], [[ui:exp.node.branch_value]] | [[ui:exp.yes]], [[ui:exp.no]] | — |
| Jeder Warteknoten ([[ui:exp.node.wait_osc]], [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_http]], [[ui:exp.node.wait_ws]]) | [[ui:exp.portMatched]] | [[ui:exp.portTimeout]] |
| [[ui:exp.node.loop]] | [[ui:exp.portBody]], [[ui:exp.portDone]] | [[ui:exp.portLimit]] |
| Jeder andere Knoten | [[ui:exp.outputPort]] | — |

Zum Verbinden ziehen Sie von einem Ausgang auf einen Knoten; auf leerer Arbeitsfläche
fallen gelassen, fügt es dort einen neuen Knoten hinzu. Ziehen von einem Ausgang, der
bereits eine Verbindung hat, fügt eine weitere hinzu. [[ui:exp.addNext]], die Taste
<kbd>A</kbd> und das ＋ an einer Verbindung fügen stattdessen einen Knoten in die
bestehende Verbindung ein.

Ein unfertiger Graph ist ein Entwurf: Er speichert, aber er läuft nicht.
[[ui:exp.needsLinks]] in der Werkzeugleiste sagt, was fehlt, und zeigt den Knoten.
Siehe [was vor einem Durchlauf geprüft wird](#validation).

## Parallele Zweige {#parallel}

### Mehrere Verbindungen von einem Ausgang {#fan-out}

Wenn ein Ausgang mehrere Verbindungen hat — die von [[ui:exp.node.start]]
eingeschlossen —, läuft jeder Knoten, zu dem sie führen, zur selben Zeit. Die erste
Verbindung setzt den Zweig fort; jede weitere Verbindung startet einen parallelen
Zweig. Jeder Zweig trägt seine eigene Kopie der Variablen und der neuesten
HTTP-Antwort, sodass das, was ein Zweig setzt oder empfängt, von den anderen nicht
gesehen wird.

### Paralleler Zweig und Join {#fork-join}

[[ui:exp.node.fork]] geht sofort weiter und verlässt den Knoten durch
[[ui:exp.branch1]] und [[ui:exp.branch2]] — dasselbe wie zwei Verbindungen von einem
Ausgang, gezeichnet als Knoten.

[[ui:exp.node.join]] wartet auf **jede** Verbindung, die in ihn führt, und fährt dann
als ein Zweig fort, mit den in der Reihenfolge dieser Verbindungen zusammengeführten
Kopien:

- die Variablen aller — bei einem Namen, den zwei Zweige beide setzen, gewinnt die
  später im Experiment aufgelistete Verbindung;
- die HTTP-Antwort der letzten Verbindung in dieser Reihenfolge, die eine mitbringt;
- für die Warteknoten nach ihm die früheste ihrer letzten Aktionen.

Die Reihenfolge der Verbindungen entscheidet, nie welcher Zweig zufällig zuerst
fertig wurde.

::: warning Nur zusammenführen, was parallel läuft
Ein Join zählt seine Verbindungen, wie sie auch parallel zu laufen kamen: von einem
[[ui:exp.node.fork]], von mehreren Verbindungen eines Ausgangs, von getrennten
Pfaden. Hinter [[ui:exp.yes]] und [[ui:exp.no]] eines Zweigs läuft nur ein Pfad, ein
Join, der von beiden gespeist wird, wartet also auf einen Zweig, der nie kommt: Der
Durchlauf schlägt mit `run.join_waiting` fehl und nennt, wie viele Verbindungen nie
gefolgt wurde. Um alternative Pfade zusammenzubringen, verdrahten Sie sie gerade in
den nächsten Knoten.
:::

Ein Knoten, der kein Join ist und von zwei parallelen Zweigen erreicht wird, läuft
einmal für jeden von ihnen.

### Wenn ein Schritt fehlschlägt {#failure}

Der erste Fehler lässt den Durchlauf fehlschlagen. Die anderen Zweige starten keinen
neuen Schritt: Eine Wiederholung oder eine Last endet früh, jeder andere Schritt, in
dem sie sich befinden, läuft bis zu seinem Ende. Ein Fehler, auf den sie inzwischen
treffen, wird in der Zeitleiste berichtet, ist aber nicht der Fehler des Durchlaufs.
Ein Schritt, der in eine Zeitüberschreitung lief, während eine
[[ui:exp.portTimeout]]-Verbindung da war, ist nicht fehlgeschlagen — siehe
[Warteknoten](#timeout).

## Verzweigung {#branching}

| Knoten | Verlässt den Knoten durch [[ui:exp.yes]], wenn |
| --- | --- |
| [[ui:exp.node.branch_status]] | die neueste HTTP-Antwort auf diesem Pfad den angegebenen Status hat |
| [[ui:exp.node.branch_value]] | sein Vergleich gilt — siehe [Werte vergleichen](data.md#compare) |

Andernfalls verlässt jeder den Knoten durch [[ui:exp.no]]. Ein Statuszweig braucht auf
jedem Pfad davor eine HTTP-Anfrage; ein Zweig auf einen Wert braucht die Namen, die er
liest, dort bekannt. Wenn die Pfade nach [[ui:exp.yes]] und [[ui:exp.no]] wieder
zusammentreffen, läuft der Knoten, an dem sie zusammentreffen, einmal, und dort ist
nur bekannt, was auf beiden Pfaden gesetzt wurde ([wo eine Variable bekannt
ist](data.md#visibility)).

## Neuversuch {#retry}

Ein Schritt, der sendet oder lauscht, kann es erneut versuchen, wenn er fehlschlägt:
Schalten Sie [[ui:exp.retryOn]] in seinen Eigenschaften an.

| Einstellung | Was | Bereich | Zuerst gegeben |
| --- | --- | --- | --- |
| [[ui:exp.attempts]] | Versuche insgesamt, der erste eingeschlossen | 1–10 | 3 |
| [[ui:exp.retryDelay]] | die Pause vor dem zweiten Versuch | 0–60 000 ms | 500 |
| [[ui:exp.backoff]] | [[ui:exp.backoff.fixed]]: jede Pause gleich; [[ui:exp.backoff.exponential]]: jede Pause doppelt so lang wie die vorige | — | [[ui:exp.backoff.fixed]] |

- Neuversuch gilt für [[ui:exp.node.http]], [[ui:exp.node.tcp]],
  [[ui:exp.node.mqtt]], [[ui:exp.node.osc]], [[ui:exp.node.udp]],
  [[ui:exp.node.ws_connect]], [[ui:exp.node.ws_send]] und jeden Warteknoten. Andere
  Knoten lehnen es ab (`node.retry_unsupported`), und eine HTTP-Anfrage unter
  [Last](load.md) nimmt keinen Neuversuch an.
- Keine einzelne Pause ist länger als 60 s, was die Verdopplung auch ergibt.
- Jeder fehlgeschlagene Versuch erscheint in der Zeitleiste als [[ui:exp.retry]], mit
  seiner Nummer und seinem Grund. Der Schritt besteht dann oder schlägt mit dem Grund
  des letzten Versuchs fehl.
- Nur die Ausführung wird wiederholt. Ein Feld, dessen Vorlage sich nicht auflöst,
  schlägt sofort fehl.
- Ein Senden, das auf seine Antwort wartet, sendet erneut. Ein Warteknoten wartet
  erneut und zählt wie zuvor ab der letzten Aktion des Zweigs.
- Ein Warteknoten mit einer [[ui:exp.portTimeout]]-Verbindung schlägt bei einer
  Zeitüberschreitung nicht fehl und wird daher nicht erneut versucht: Er folgt
  [[ui:exp.portTimeout]].
- [Stopp](#stop) beendet eine Pause sofort.

## Wiederholung {#repeat}

Ein Schritt, der sendet, kann immer wieder senden — ein Herzschlag, ein Polling, ein
gleichmäßiger Strom —, ohne eine Schleife im Graphen: Schalten Sie
[[ui:exp.repeatOn]] an.

| Einstellung | Was | Bereich | Zuerst gegeben |
| --- | --- | --- | --- |
| [[ui:exp.repeatBy]] | [[ui:exp.repeatBy.count]] oder [[ui:exp.repeatBy.duration]] | — | [[ui:exp.repeatBy.count]] |
| [[ui:exp.repeatCount]] | Sendungen insgesamt, die erste eingeschlossen | 2–10 000 | 10 |
| [[ui:exp.repeatDuration]] | wie lange weiter gesendet wird, ab der ersten Sendung | 1–300 000 ms | 10 000 |
| [[ui:exp.repeatInterval]] | die Pause zwischen zwei Sendungen | 10–60 000 ms | 1000 |
| [[ui:exp.repeatJitter]] | jede Pause ist bis zu so viel länger, zufällig | 0–60 000 ms | 0 |

- Wiederholung gilt für [[ui:exp.node.http]], [[ui:exp.node.tcp]],
  [[ui:exp.node.mqtt]], [[ui:exp.node.osc]], [[ui:exp.node.udp]] und
  [[ui:exp.node.ws_send]] (`node.repeat_unsupported` sonst). Eine HTTP-Anfrage hat
  entweder Wiederholung oder [Last](load.md), nicht beides.
- Jede Sendung erfolgt wie eine einzelne: Ihre Vorlagen werden erneut gelesen —
  `{{counter}}` ist die Nummer der Sendung, `{{now}}` ihre Zeit — und Neuversuch
  gilt, wenn an, für jede Sendung. Ein Senden, das auf eine Antwort wartet, wartet
  auf seine eigene.
- Bei einer Dauer wird eine Sendung nur gemacht, wenn sie vor Ablauf der Dauer
  starten kann.
- Der Jitter wird aus dem Startwert des Durchlaufs gezogen: Derselbe Startwert ergibt
  dieselben Pausen.
- Die Zeitleiste berichtet den Fortschritt als [[ui:exp.repeating]] höchstens einmal
  pro Sekunde. Der Schritt besteht nach der letzten Sendung, mit deren Ergebnis; eine
  Sendung, die endgültig fehlschlägt, lässt den Schritt fehlschlagen.
- Ein Fehler auf einem anderen Zweig beendet die Sendungen; [Stopp](#stop) beendet
  eine Pause sofort.

Es muss in einen Durchlauf passen: Die Sendungen und ihre längsten Pausen
(`(count − 1) × (interval + jitter)`) höchstens 300 s (`node.repeat_too_long`), und
eine zeitgesteuerte Wiederholung höchstens 10 000 Sendungen
(`node.repeat_too_many`).

## Schleife {#loop}

[[ui:exp.node.loop]] führt die Schritte an seinem [[ui:exp.portBody]]-Ausgang immer
wieder aus; der letzte von ihnen ist zurück zur Schleife verdrahtet.

| Einstellung | Was | Bereich |
| --- | --- | --- |
| [[ui:exp.loopMax]] | die meisten Iterationen | 1–1000 |
| [[ui:exp.loopUntilOn]] | eine Abbruchbedingung: [[ui:exp.value]], [[ui:exp.operator]], [[ui:exp.expected]], wie bei [[ui:exp.node.assert_value]] | optional |

1. Von außen erreicht, startet die Schleife Iteration 1 an [[ui:exp.portBody]].
2. Jedes Mal, wenn der Rumpf zurückkommt, wird die Abbruchbedingung gelesen — nach
   der Iteration, sodass der Rumpf immer mindestens einmal läuft und setzen kann, was
   er prüft.
3. Wenn die Bedingung gilt, verlässt die Schleife den Knoten durch [[ui:exp.portDone]].
4. Andernfalls startet die nächste Iteration, solange welche übrig sind.
5. Wenn die Iterationen zuerst ausgehen, verlässt die Schleife den Knoten durch
   [[ui:exp.portLimit]], wenn dieser verdrahtet ist, und lässt den Durchlauf mit
   `loop.limit` fehlschlagen, wenn nicht. Ohne Bedingung läuft der Rumpf jede
   Iteration, und die Schleife verlässt den Knoten durch [[ui:exp.portDone]].

Im Rumpf ist `{{counter}}` die Nummer der Iteration, da jeder Knoten seine eigenen
Ausführungen zählt. Die Bedingung und die Schritte nach [[ui:exp.portDone]] oder
[[ui:exp.portLimit]] dürfen verwenden, was jede Iteration des Rumpfs setzt — etwa
einen Status, den der Rumpf extrahiert; der Rumpf selbst sieht nur, was bekannt war,
als die Schleife erreicht wurde.

Die Vorlage [[ui:exp.templatePoll]] fragt ein Gerät alle 0,3 s nach seinem Status,
bis es `ready` antwortet, höchstens 10-mal.

### Was ein Rumpf enthalten darf {#loop-body}

Ein Rumpf läuft als ein Zweig, eine Iteration nach der anderen. Die Verbindung zurück
zur Schleife ist der einzige Kreislauf, den ein Experiment haben darf; jeder andere
ist `graph.cycle`.

| Regel | Fehler |
| --- | --- |
| Etwas an [[ui:exp.portBody]] führt zurück zur Schleife | `loop.no_return` |
| Jeder Ausgang im Rumpf hat eine Verbindung | `loop.body_parallel` |
| Jeder Ausgang im Rumpf führt im Rumpf weiter oder zurück zur Schleife | `loop.body_leaves` |
| Nur der [[ui:exp.portBody]]-Ausgang der Schleife führt in den Rumpf | `loop.body_entered` |
| Kein [[ui:exp.node.start]], [[ui:exp.node.end]], [[ui:exp.node.fork]], [[ui:exp.node.join]] oder anderer [[ui:exp.node.loop]] im Rumpf | `loop.body_unsupported` |

## Warteknoten {#waits}

Ein Warteknoten besteht, wenn eine Nachricht eintrifft, auf die er wartet:
[[ui:exp.node.wait_osc]], [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]],
[[ui:exp.node.wait_http]] und [[ui:exp.node.wait_ws]]. Was jeder abgleicht, steht in
[der Referenz der Knoten](nodes.md); hier geht es darum, wie sie lauschen.

### Vom Start an lauschen {#listening}

Der Durchlauf öffnet, worauf seine Warteknoten lauschen, **vor seinem ersten
Schritt**, sodass eine Antwort, die schneller ist als der nächste Schritt, nicht
verpasst wird:

| Warten | Vor dem ersten Schritt geöffnet |
| --- | --- |
| OSC, UDP | ein UDP-Socket pro [[ui:exp.listenOn]]-Adresse, von jedem Warteknoten darauf geteilt |
| HTTP-Anfrage | ein Empfänger pro Adresse — der HTTP-[Emulator](faults.md#emulator) des Durchlaufs, wenn einer da ist, sonst einer, der `204` antwortet |
| MQTT | eine Verbindung pro Broker und Topic-Filter, abonniert; Retained-Nachrichten, die der Broker dann erneut zustellt, werden ignoriert |
| WebSocket | nichts: Er liest die Verbindung, die ein [[ui:exp.node.ws_connect]] öffnete, als er lief |

Weil sie zuerst öffnen, stehen diese Adressen vor dem Durchlauf fest: Ein OSC-, UDP-
oder HTTP-Warteknoten lauscht auf einem wörtlichen `IP:port` mit einem Port ungleich
0, und Broker und Topic eines MQTT-Warteknotens nehmen nur Parameter an. Ein Port,
der sich nicht öffnen lässt — belegt oder keine Adresse dieses Computers —, stoppt
den Durchlauf vor jedem Verkehr, am Feld jenes Warteknotens. Alles wird geschlossen,
wenn der Durchlauf endet, wie auch immer.

### Welche Nachrichten zählen {#counting}

Ein Warteknoten betrachtet die Nachrichten, die eingetroffen sind, nachdem die
**letzte Aktion auf seinem Zweig** begonnen hat — die letzte Anfrage, Nachricht,
Veröffentlichung, WebSocket-Verbindung oder Sendung —, oder, vor jeder Aktion,
nachdem der Durchlauf startete. Eine Nachricht von vor der Anfrage zählt nicht, und
eine Verzögerung oder ein Log zwischen der Anfrage und dem Warten verbergen seine
Antwort nicht. Nach einem Join zählt die früheste letzte Aktion der
zusammengeführten Zweige.

Ein Warteknoten nimmt die erste passende Nachricht und verbraucht sie: Zwei
Warteknoten gleichen nie dieselbe Nachricht ab.

Jeder Socket, jedes Abonnement oder jede Verbindung behält höchstens 1024 Nachrichten
und 64 MiB für seine Warteknoten; darüber hinaus werden die ältesten verworfen und
gezählt.

### Zeitüberschreitung {#timeout}

[[ui:exp.waitTimeout]] ist 1–120 000 ms, zuerst 2000. Wenn nichts rechtzeitig passt:

- mit einer [[ui:exp.portTimeout]]-Verbindung folgt der Warteknoten ihr;
- ohne eine schlägt der Schritt mit `wait.timeout` fehl, was sagt, wie viele andere
  Nachrichten inzwischen eintrafen — ein falsches Muster sieht anders aus als ein
  stilles Gerät —, und in seinem Detail, wie viele ältere Nachrichten verworfen
  wurden, als die Warteschlange voll war.

Die Variable des Warteknotens — `reply` oder `request` bei HTTP — existiert erst
nach [[ui:exp.portMatched]]. Wenn der [[ui:dock.inspector]] aufzeichnet, verknüpft
der Schritt außerdem den Frame, den er abgeglichen hat: siehe [die
Zeitleiste](runs.md#timeline).

## Eine Antwort im selben Schritt {#reply}

Ein [[ui:exp.node.osc]] oder ein [[ui:exp.node.udp]] kann auf seine eigene Antwort
warten: Schalten Sie [[ui:exp.expectReply]] an.

| Einstellung | Was | Zuerst gegeben |
| --- | --- | --- |
| [[ui:exp.replyOn]] | die Adresse, auf der die Antwort erwartet wird; Port 0 ist ein beliebiger freier Port | `0.0.0.0:0` |
| [[ui:exp.replyAddress]] (OSC), [[ui:exp.replyMode]] (UDP) | was die Antwort sein muss, wie bei einem abgleichenden Warteknoten | beliebig |
| [[ui:exp.waitTimeout]] | 1–120 000 ms | 2000 |
| [[ui:exp.replyVariable]] | die Variable, in die die Antwort geschrieben wird | `reply` |

Der Socket auf [[ui:exp.replyOn]] wird vor dem ersten Schritt geöffnet, wie der eines
Warteknotens, und die Nachricht **geht von ihm aus**: Ein Gerät, das an den eigenen
Port des Senders antwortet, wird gehört, und eines, das an einen festen Port
antwortet, wird gehört, wenn dieser Port der angegebene ist. Der Schritt besteht mit
einer passenden Antwort, und die Variable existiert nach seinem Ausgang. Es gibt
keinen [[ui:exp.portTimeout]]-Ausgang: Keine Antwort rechtzeitig lässt den Schritt
fehlschlagen, und Neuversuch kann erneut senden. Um auf Stille zu verzweigen,
verwenden Sie einen separaten Warteknoten.

## Was vor einem Durchlauf geprüft wird {#validation}

Der Editor prüft das Experiment, während Sie es bearbeiten; die Schaltfläche
Ausführen prüft es noch einmal. Ein Problem nennt den Knoten und, wenn es eines gibt,
das Feld.

| Regel | Fehler |
| --- | --- |
| Das Experiment hat einen Namen | `doc.name_required` |
| 1–64 Knoten, genau ein Start und ein Ende | `doc.node_count`, `doc.start_end_count` |
| Start hat keinen Eingang | `graph.start_input` |
| Eine Verbindung führt zu einem anderen Knoten, der existiert | `doc.connection_invalid` |
| Dieselbe Verbindung ist nicht zweimal da | `doc.connection_duplicate` |
| Jeder Ausgang, der verdrahtet sein muss, ist es | `graph.outputs_required` |
| Ein Knoten hat keine Verbindung an einem Ausgang, den er nicht hat | `graph.port_unexpected` |
| Jeder Knoten ist von Start aus erreichbar | `graph.unreachable` |
| Kein Kreislauf außer der Verbindung einer Schleife zurück | `graph.cycle` und die [Rumpfregeln](#loop-body) |
| Eine Prüfung oder ein Extract hat auf jedem Pfad davor eine HTTP-Anfrage; eine Anfrage unter Last zählt nicht | `graph.needs_http` |
| Jede Vorlage parst, und jeder Name, den sie verwendet, ist auf jedem Pfad bekannt | `template.*`, `name.*` — siehe [Daten](data.md#unknown-names) |
| Jedes Feld ist vorhanden und im Bereich | `node.*` |
| Ein Knoten, der einen anderen nennt — [[ui:exp.node.impairment_change]], [[ui:exp.node.emulator_state]], die WebSocket-Knoten —, nennt einen, der da ist, und ein WebSocket-Knoten kommt nach seinem Connect | `impair.relay_unknown`, `emulator.node_unknown`, `ws.connection_unknown`, `ws.connection_after` |
| Zwei der Sockets des Durchlaufs teilen sich keinen Port | siehe [Störungen](faults.md#ports) |

Der Durchlauf prüft dann, was er zum Starten braucht: jedes
[Geheimnis](data.md#secret-check) gespeichert, jeder Port offen. Bis alles gilt, läuft
kein Schritt und nichts wird gesendet.

## Zeitlimit {#limit}

Ein Durchlauf dauert höchstens 300 s. Einer, der dann noch läuft, wird gestoppt und
schlägt mit `run.timeout` fehl. Von der
[Kommandozeile](../automation/cli.md#cli-run) und [der API](../api/run.md) kann das
Limit kürzer sein, 1–300 s.

## Stopp {#stop}

Während ein Durchlauf läuft, ist die Schaltfläche Ausführen [[ui:common.stop]]. Stopp
beendet den Durchlauf sofort: jeden Zweig, jede Pause von Neuversuch oder
Wiederholung, jeden Warteknoten und jede Last — Anfragen unterwegs werden verworfen.
Seine Sockets, Abonnements, Emulatoren und Relais schließen sich, und seine
WebSocket-Verbindungen senden einen Close-Frame. [[ui:app.stopAll]] in der Kopfzeile
tut dasselbe mit jedem Job. Ein gestoppter Durchlauf speichert keinen Bericht; siehe
[Durchläufe](runs.md#stop).

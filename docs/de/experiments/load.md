---
title: Lasttests
description: Die Anfrage eines HTTP-Knotens nach einem Lastprofil senden — konstant, Rampe, Stufen, Spitze oder zufällige Ankünfte —, Latenzen, Fehler und die erreichte Rate messen, mit Schwellenwerten beurteilen und zwei Durchläufe vergleichen.
---

# Lasttest einer HTTP-Anfrage

Ein Knoten [[ui:exp.node.http]] kann seine Anfrage viele Male senden, nach einem
Profil von Anfragen pro Sekunde, viele gleichzeitig — und messen, was zurückkommt:
Latenz-Perzentile, Fehler, die erreichte Rate. Schwellenwerte entscheiden, ob der
Schritt besteht, und [[ui:exp.compare]] stellt die Zahlen neben die eines früheren Durchlaufs.

Last ist eine Einstellung des Knotens, kein eigener Knoten: Der Rest des
Experiments — Emulatoren, Störungs-Relais, andere Zweige — läuft wie gewohnt
um ihn herum.

## Eine Anfrage unter Last setzen {#turn-on}

1. Wählen Sie einen Knoten [[ui:exp.node.http]] aus und füllen Sie seine Anfrage aus.
2. Haken Sie in seinen Eigenschaften [[ui:exp.loadOn]] an.
3. Wählen Sie ein [[ui:exp.loadShape]] und seine Zahlen. Das Diagramm darunter,
   [[ui:exp.loadChart]], zeichnet die Rate und sagt, wie viele Anfragen sich daraus ergeben,
   in wie vielen Sekunden.
4. Setzen Sie [[ui:exp.loadConcurrency]] — wie viele Anfragen gleichzeitig unterwegs sein dürfen.
5. Fügen Sie [[ui:exp.thresholds]] hinzu oder ändern Sie sie.
6. Führen Sie das Experiment aus.

Eine Last beginnt als [[ui:exp.loadShape.ramp]] von 0 auf 100 Anfragen pro
Sekunde über 30.000 ms, 32 gleichzeitig, mit zwei Schwellenwerten:
[[ui:exp.metric.p95_ms]] < 500 ms und [[ui:exp.metric.error_rate]] < 1 %.

**Last ersetzt Wiederholung und Neuversuch.** Wird sie eingeschaltet, werden beide ausgeschaltet, und ein Knoten
mit Last und einem von beiden wird abgelehnt (`node.load_alone`): Eine fehlgeschlagene Anfrage
wird gezählt, nicht erneut versucht. Nur eine HTTP-Anfrage kann unter Last laufen
(`node.load_unsupported`).

**Die Anfrage wird einmal gelesen.** Ihre Vorlagen werden beim Start des Schritts aufgelöst,
sodass jede Anfrage der Last dieselbe ist: `{{counter}}` und `{{uuid}}` haben
für alle denselben Wert. Siehe [Vorlagen](data.md#templates).

**Ein Client für die ganze Last.** Die Anfragen teilen sich den Cookie-Speicher des Durchlaufs, wenn
[[ui:exp.cookies]] an ist, und ein Digest-Gedächtnis, sodass eine einzige Challenge sie
alle beantwortet. Jede Anfrage hat das eigene Timeout des Knotens.

**Der Inspektor erhält eine Stichprobe:** höchstens einen Austausch alle 100 ms, damit eine Last
den [[ui:dock.inspector]] nicht überflutet.

## Profile {#profiles}

| [[ui:exp.loadShape]] | Einstellungen | Die Rate über die Zeit |
| --- | --- | --- |
| [[ui:exp.loadShape.constant]] | [[ui:exp.loadRate]], [[ui:exp.loadDuration]] | durchgehend die Rate |
| [[ui:exp.loadShape.ramp]] | [[ui:exp.loadFrom]], [[ui:exp.loadTo]], [[ui:exp.loadDuration]] | in gerader Linie von einer Rate zur anderen |
| [[ui:exp.loadShape.steps]] | [[ui:exp.loadFrom]], [[ui:exp.loadStepBy]], [[ui:exp.loadEvery]], [[ui:exp.loadSteps]] | zuerst die Anfangsrate, dann auf jeder Stufe um einen Schritt mehr, jede Stufe gleich lang |
| [[ui:exp.loadShape.spike]] | [[ui:exp.loadBase]], [[ui:exp.loadPeak]], [[ui:exp.loadAt]], [[ui:exp.loadSpikeFor]], [[ui:exp.loadDuration]] | die Grundrate, ab einem bestimmten Moment eine Weile die Spitze, dann wieder die Grundrate |
| [[ui:exp.loadShape.poisson]] | [[ui:exp.loadRate]], [[ui:exp.loadDuration]] | zufällige Ankünfte, die Rate im Mittel |

Beim Wechsel der Form bleibt erhalten, was sich übertragen lässt: wie lange sie läuft und die höchste
Rate, die sie erreicht.

### Grenzen {#limits}

| Einstellung | Bereich |
| --- | --- |
| [[ui:exp.loadRate]] von [[ui:exp.loadShape.constant]] und [[ui:exp.loadShape.poisson]], [[ui:exp.loadPeak]] | 0,1–100.000 Anfragen/s |
| [[ui:exp.loadFrom]], [[ui:exp.loadTo]], [[ui:exp.loadBase]] | 0–100.000 Anfragen/s |
| Jede Stufe von [[ui:exp.loadShape.steps]], die letzte eingeschlossen | 0–100.000 Anfragen/s; der Schritt darf negativ sein |
| [[ui:exp.loadDuration]], [[ui:exp.loadEvery]] | 100–300.000 ms |
| [[ui:exp.loadSteps]] | 1–100, und alle Stufen zusammen höchstens 300.000 ms |
| Eine Spitze | länger als 0 ms und bis zum Ende der Dauer vorbei |
| [[ui:exp.loadConcurrency]] | 1–512 |
| [[ui:exp.thresholds]] | höchstens 16, jeder Wert eine Zahl, 0 oder größer |

Ein Profil, das insgesamt keine einzige Anfrage ergibt, wird abgelehnt
(`load.nothing_planned`). Die Raten sind die des
[HTTP-Bursts](../protocols/http.md).

Ein Profil darf so lange dauern wie ein ganzer Durchlauf, 300 s — doch das
[Zeitlimit](flow.md#limit) des Durchlaufs zählt jeden Schritt, lassen Sie also Raum für den Rest
des Experiments.

### Wie viele Anfragen {#planned}

Die Anfragen eines Profils sind seine Rate, über die Zeit aufsummiert:

| Profil | Anfragen |
| --- | --- |
| [[ui:exp.loadShape.constant]], 100/s für 1000 ms | 100 |
| [[ui:exp.loadShape.ramp]], 0 → 100/s über 2000 ms | 100 |
| [[ui:exp.loadShape.steps]], ab 10/s in Schritten von 10/s, 3 Stufen zu je 1000 ms | 60 (10 + 20 + 30) |
| [[ui:exp.loadShape.spike]], 10/s mit 100/s ab 1000 ms für 500 ms, insgesamt 2000 ms | 65 |
| [[ui:exp.loadShape.poisson]], 200/s für 10.000 ms | im Mittel 2000 |

## Der Zeitplan {#schedule}

Die n-te Anfrage ist in dem Moment fällig, in dem die Zählung des Profils n erreicht — die erste
sofort. Jeder Moment wird ab dem Start der Last berechnet, sodass ein verspätetes
Aufwachen nie die nachfolgenden Anfragen verschiebt und die Rate, die das Profil beschreibt,
auch die angeforderte ist.

Das Profil [[ui:exp.loadShape.poisson]] zieht die Abstände zwischen den Ankünften zufällig, aus
dem Startwert des Durchlaufs: Derselbe Startwert ergibt dieselben Momente, sodass sich eine zufällige Last
exakt wiederholen lässt. Siehe [Startwerte](runs.md#seeds).

**Verpasste Anfragen.** Höchstens so viele Anfragen, wie [[ui:exp.loadConcurrency]] vorgibt, sind unterwegs.
Wartet jede davon noch auf ihre Antwort, wartet die nächste Anfrage
auf einen freien Platz. Würde sie mehr als 50 ms nach ihrem Moment hinausgehen, wird sie nicht
verspätet gesendet: Sie wird übersprungen und als verpasst gezählt, mit jeder anderen Anfrage, die
inzwischen fällig wurde, und die Last geht mit der ersten weiter, die noch pünktlich ist. Viele
verpasste Anfragen bedeuten, dass der Server — oder der Wert [[ui:exp.loadConcurrency]] — mit dem Profil nicht
mithalten konnte.

## Während sie läuft {#progress}

Einmal pro Sekunde zeigt die Zeitleiste den Schritt als [[ui:exp.load]], mit den vergangenen
Sekunden, den gesendeten Anfragen, der Rate in der letzten Sekunde, dem bisherigen p95 und den
fehlgeschlagenen Anfragen. [[ui:common.stop]] beendet die Last sofort und verwirft die
Anfragen, die unterwegs sind; ein Fehler in einem anderen Zweig beendet sie innerhalb einer Sekunde.

## Was gemessen wird {#metrics}

Nach der letzten Antwort hat der Schritt seine Messwerte, festgehalten in seinem letzten
Ereignis der Zeitleiste und im [Bericht des Durchlaufs](runs.md#report):

| Messwert | Was |
| --- | --- |
| planned | die Anfragen, die das Profil ergibt ([[ui:exp.loadShape.poisson]]: im Mittel) |
| sent | Anfragen, die beantwortet wurden oder fehlschlugen |
| ok | mit einem 2xx-Status beantwortet |
| failed | jeder andere Status oder gar keine Antwort |
| missed | fällig, während jeder Platz belegt war, und übersprungen |
| rps | gesendete Anfragen pro Sekunde: sent ÷ Dauer des Profils — oder ÷ die Zeit bis zur letzten gesendeten Anfrage, wenn das später war |
| error_rate | failed, in % von sent |
| min, mean, max | die schnellste, die durchschnittliche und die langsamste Anfrage, ms |
| p50, p90, p95, p99 | die Latenz, bei oder unter der 50, 90, 95 und 99 % der Anfragen lagen, ms |
| received_bytes | insgesamt empfangene Body-Bytes |
| statuses | Anfragen nach Status (`200`, `503`) und, ohne Status, nach Ursache (`timeout`, `refused`, `reset` …) |
| seconds | jede Sekunde des Profils: gesendete Anfragen, fehlgeschlagene, ihre mittlere Latenz |
| histogram | Anfragen nach Latenz, bis 1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10.000 ms und langsamer |

Die Latenz einer Anfrage reicht vom Senden bis zum vollständigen Lesen ihrer Antwort, und
eine fehlgeschlagene Anfrage zählt mit der Zeit, die sie bis zum Fehlschlag brauchte. Die Perzentile werden
aus logarithmischen Klassen von 1 % Breite gelesen und liegen höchstens 0,5 % neben dem wahren Wert,
wie lange die Last auch läuft.

## Schwellenwerte {#thresholds}

Ein Schwellenwert ist eine Zeile aus [[ui:exp.thresholdMetric]], [[ui:exp.thresholdOp]] und
[[ui:exp.thresholdValue]]; die Schaltfläche [[ui:exp.thresholdAdd]] fügt einen hinzu.

| [[ui:exp.thresholdMetric]] | Gemessen in |
| --- | --- |
| [[ui:exp.metric.p50_ms]], [[ui:exp.metric.p90_ms]], [[ui:exp.metric.p95_ms]], [[ui:exp.metric.p99_ms]] | ms |
| [[ui:exp.metric.mean_ms]], [[ui:exp.metric.max_ms]] | ms |
| [[ui:exp.metric.error_rate]] | % der gesendeten Anfragen |
| [[ui:exp.metric.rps]] | erreichte Anfragen pro Sekunde |
| [[ui:exp.metric.missed]] | Anfragen |

[[ui:exp.thresholdOp]] ist einer von `<`, `≤`, `>`, `≥`. Ein paar gebräuchliche:

| [[ui:exp.thresholdMetric]] | [[ui:exp.thresholdOp]] | [[ui:exp.thresholdValue]] | Der Schritt schlägt fehl, wenn |
| --- | --- | --- | --- |
| [[ui:exp.metric.p95_ms]] | `<` | 300 | eine von zwanzig Anfragen oder mehr 300 ms oder länger dauerte |
| [[ui:exp.metric.error_rate]] | `<` | 1 | 1 % oder mehr der Anfragen fehlschlugen |
| [[ui:exp.metric.rps]] | `≥` | 180 | der Server keine 180 Anfragen pro Sekunde annehmen konnte |
| [[ui:exp.metric.missed]] | `≤` | 0 | auch nur eine einzige Anfrage übersprungen werden musste |

In einer Datei ist ein Schwellenwert `{ "metric": "p95_ms", "op": "lt", "value": 300 }`;
die Metriken sind `p50_ms`, `p90_ms`, `p95_ms`, `p99_ms`, `mean_ms`, `max_ms`,
`error_rate`, `rps` und `missed`, die Vergleiche `lt`, `le`, `gt` und `ge`.

Die Schwellenwerte werden nach der letzten Antwort gelesen, in ihrer Reihenfolge. Der Schritt schlägt
beim ersten fehl, der nicht eingehalten ist (`load.threshold`); seine Meldung nennt den
Schwellenwert und den gemessenen Wert, und der Durchlauf schlägt mit ihm fehl. Ohne Schwellenwerte besteht eine Last, was immer sie gemessen hat. Hat
der Fehler eines anderen Zweigs die Last vorzeitig beendet, ist dieser Fehler der des Durchlaufs, nicht
ein Schwellenwert.

## Das Ergebnis {#result}

Besteht der Schritt, fasst die Zeitleiste ihn zusammen: die Anfragen, die Rate, p95
und den Anteil der fehlgeschlagenen. Wählen Sie den Knoten aus: Seine Eigenschaften zeigen
[[ui:exp.loadResult]] —

- jeden Schwellenwert, ✓ [[ui:exp.thresholdHeld]] oder ✕ [[ui:exp.thresholdBroken]],
  mit dem gemessenen Wert;
- [[ui:http.sent]], [[ui:exp.loadRps]], [[ui:exp.loadErrors]] mit ihrem Anteil,
  [[ui:http.missed]];
- [[ui:http.p50]], [[ui:http.p90]], [[ui:http.p95]], [[ui:http.p99]],
  [[ui:http.avg]], [[ui:http.max]];
- [[ui:exp.loadPerSecond]]: die Anfragen jeder Sekunde, die fehlgeschlagenen in
  Rot, und ihre mittlere Latenz als Linie;
- [[ui:exp.loadLatencies]]: wie viele Anfragen wie lange gedauert haben;
- die Status und Ursachen, jeweils mit ihrer Anzahl.

Die Kommandozeile gibt dieselben Zahlen und das Urteil jedes Schwellenwerts aus; siehe
[`signallab run`](../automation/cli.md#cli-run).

## Zwei Durchläufe vergleichen {#compare}

1. Führen Sie das Experiment zweimal oder öfter aus.
2. Drücken Sie in der Zeitleiste [[ui:exp.compare]]. Die Schaltfläche ist da, sobald ein Durchlauf seinen
   Bericht gespeichert hat, und ist deaktiviert, solange ein Durchlauf läuft.
3. Der neueste Durchlauf ist [[ui:exp.compareAfter]], der davor
   [[ui:exp.compareBefore]]; in beiden Listen lässt sich ein anderer Durchlauf wählen.

Die Listen enthalten die Durchläufe dieses Experiments — nach seinem Namen — aus den Berichten im
Datenordner, die neuesten zuerst, höchstens 50: jeder mit Datum und Uhrzeit, wie er
endete, und seinem Startwert. Durchläufe von der Kommandozeile sind ebenfalls dabei, wenn sie denselben
Datenordner verwendet hat. Wird das Experiment umbenannt, beginnt eine neue Historie.

Für jeden Lastschritt, nach Knoten zugeordnet, zeigt eine Tabelle jede Metrik in den Spalten
[[ui:exp.compareBefore]], [[ui:exp.compareAfter]] und
[[ui:exp.compareChange]], in der Einheit und in %. Eine Änderung um 5 %
oder mehr in die falsche Richtung — langsamer, mehr Fehler, mehr verpasste Anfragen, eine niedrigere Rate — ist eine
Verschlechterung und erscheint in Rot; der Schritt von nichts zu etwas zählt ebenfalls. Unter der
Tabelle steht das Urteil jedes Schwellenwerts in beiden Durchläufen. Ein Lastschritt, den nur einer der
Durchläufe hat, ist als [[ui:exp.compareOnlyBefore]] oder [[ui:exp.compareOnlyAfter]] markiert,
ohne Änderungen. Durchläufe ohne Lastschritte zeigen [[ui:exp.compareNoLoad]].

Aus einem Skript listet [`experiment_runs`](../api/commands.md#experiment_runs) die
Durchläufe auf, und [`experiment_compare`](../api/commands.md#experiment_compare) vergleicht
zwei, anhand des Dateinamens ihres Berichts; `signallab mcp` bietet dasselbe einem
Assistenten an ([MCP](../automation/mcp.md)).

## Prüfungen nach einer Last {#checks-after}

Eine Last hinterlässt keine eigene Antwort: Sie wird gemessen, nicht geprüft. Eine Prüfung oder ein
Knoten [[ui:exp.node.extract]] danach braucht auf jedem Pfad davor eine weitere Anfrage ohne Last,
sonst läuft das Experiment nicht (`graph.needs_http`). Um eine Antwort
der API unter Last zu prüfen, setzen Sie einen einfachen Knoten [[ui:exp.node.http]] hinter die
Last oder in einen parallelen Zweig daneben.

[[ui:exp.sendNow]] an einem Knoten unter Last sendet seine Anfrage einmal.

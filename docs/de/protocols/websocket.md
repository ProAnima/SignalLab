---
title: WebSocket
description: Mit einem ws://- oder wss://-Dienst verbinden, mit den Headern und Subprotokollen, die er erwartet, Text- oder Binärnachrichten senden und jede Nachricht lesen, sobald sie eintrifft.
---

# WebSocket

Die Ansicht [[ui:nav.ws]] ist ein WebSocket-Client: Sie öffnet eine Verbindung zu
einem Dienst — mit den Headern und Subprotokollen, die der Dienst erwartet —,
sendet Text oder Bytes und listet jede Nachricht auf, die kommt und geht, die
neueste zuletzt. Nutzen Sie sie, um eine Live-API, eine Steueroberfläche oder ein
Gerät, das WebSocket spricht, auszuprobieren, bevor Sie es in einem Experiment
skripten.

## Verbinden {#connect}

1. Öffnen Sie [[ui:nav.ws]].
2. Geben Sie unter [[ui:field.url]] die Adresse ein: `ws://127.0.0.1:9001/` oder
   `wss://example.com/socket`.
3. Verlangt der Dienst sie, geben Sie [[ui:exp.wsProtocols]] ein und fügen
   [[ui:http.headers]] hinzu (einen `Authorization`-Header mit einem Token, ein
   Cookie).
4. Drücken Sie [[ui:ws.connect]]. Während das Upgrade läuft, zeigt die
   Schaltfläche [[ui:ws.connecting]]; ist es fertig, sind die Felder gesperrt
   und die Schaltfläche wird zu [[ui:ws.disconnect]].

| Feld | Was | Standard |
| --- | --- | --- |
| [[ui:field.url]] | `ws://` oder `wss://`, ein Host, ein optionaler Port (80 für `ws`, 443 für `wss`) und ein Pfad | `ws://127.0.0.1:9001/` |
| [[ui:exp.wsProtocols]] | Anzubietende Subprotokolle, durch Kommas getrennt, in der Reihenfolge der Vorliebe; der Server wählt eines. Ein Name hat keine Leerzeichen, Kommas oder Schrägstriche. | keine |
| [[ui:http.headers]] | Zusätzliche Header für die Upgrade-Anfrage; [[ui:http.addHeader]] fügt eine Zeile hinzu. Eine Zeile ohne Namen entfällt. | keine |

Das Verbinden — das Auflösen des Namens, die TCP-Verbindung, TLS für `wss://`
und das Upgrade — dauert höchstens 10 Sekunden. Die URL bleibt erhalten, wenn
Sie die Ansicht wechseln und wenn Sie die App neu starten; Header und
Subprotokolle nicht.

Verbunden zeigt der Bereich:

| Element | Was |
| --- | --- |
| [[ui:ws.state]] | [[ui:ws.open]], oder, sobald sie endet: von Ihnen oder vom Server geschlossen, mit dem Schließcode, oder [[ui:ws.lost]], wenn die Leitung ohne ein Schließen abbrach |
| [[ui:field.reason]] | Der Grund, den die schließende Seite nannte, falls es einen gab |
| [[ui:ws.subprotocol]] | Das Subprotokoll, das der Server wählte, oder — |
| [[ui:ws.peer]] | `IP:port` des Servers |
| [[ui:ws.upgradeTime]] | Wie lange das Verbinden und das Upgrade dauerten, in Millisekunden |

Die Verbindung ist ein Job: Sie erscheint in der Leiste der Konsole und kann
auch dort gestoppt werden.

### Sichere Verbindungen {#wss}

`wss://` vertraut denselben Zertifikaten wie HTTPS auf diesem System: Ein Server,
dessen Zertifikat dieses System nicht vertraut (selbst ausgestellt, abgelaufen,
ein anderer Name), wird mit "A secure connection to … could not be made"
abgelehnt. Es gibt keine Einstellung, die Prüfung zu überspringen.

## Eine Nachricht senden {#send}

1. Wählen Sie unter [[ui:ws.message]] [[ui:exp.wsText]] oder
   [[ui:exp.wsBinary]].
2. Schreiben Sie die Nachricht. Schreiben Sie für Binärdaten die Bytes als Paare
   von Hex-Ziffern: `de ad be ef`.
3. Drücken Sie [[ui:common.send]], oder <kbd>Ctrl</kbd>+<kbd>Enter</kbd> in der
   Nachricht.

Eine Nachricht ist höchstens 16 MiB groß (16.777.216 Bytes, bei Binärdaten als
Bytes gezählt, nicht als Hex-Ziffern), dieselbe Grenze wie für eine eintreffende
Nachricht und für den Schritt [[ui:exp.node.ws_send]]. Eine längere wird mit
`Too long: at most 16777216` abgelehnt, bevor etwas gesendet wird, und die
Verbindung bleibt offen.

Ist eine Textnachricht JSON, legt [[ui:http.formatJson]] sie mit Einrückungen
zurecht, bevor Sie sie senden. Die Nachricht bleibt erhalten, wenn Sie die
Ansicht wechseln und wenn Sie die App neu starten.

## Die Nachrichten lesen {#messages}

[[ui:ws.messages]] listet auf, was empfangen (↓) und gesendet (↑) wurde, die
neueste zuletzt, mit der Zeit, dem Anfang der Nachricht (300 Zeichen) und ihrer
Größe; eine Binärnachricht zeigt ihre Bytes in Hex und ist mit [[ui:ws.binary]]
markiert. Über der Liste steht, wie viele empfangen und gesendet wurden. Die
Liste folgt neuen Nachrichten, solange sie ans Ende gescrollt ist; scrollen Sie
hoch, bleibt sie, wo Sie sind.

Klicken Sie auf eine Nachricht, um sie ganz unter der Liste zu sehen: JSON
zurechtgelegt, Binärdaten als Hex. [[ui:ws.editHere]] kopiert sie in das
Nachrichtenfeld, um sie erneut zu senden oder zu ändern. Eine sehr lange
Nachricht wird teilweise gezeigt — Text bis 64 KiB, Binärdaten bis 4096 Bytes —
und kann dann nicht kopiert werden, da sie abgeschnitten gesendet würde.

Die Ansicht behält die neuesten 2000 Nachrichten; [[ui:common.clear]] leert die
Liste. Sendet ein Dienst schneller, als die Ansicht aufnehmen kann — mehr als
2000 in einer Zehntelsekunde —, werden die ältesten davon aus der Liste
weggelassen und als „nicht angezeigt" gezählt. Der Inspektor hat sie trotzdem,
solange der Mitschnitt läuft.

## Schließen {#close}

[[ui:ws.disconnect]] sendet ein Schließen-Frame mit Code 1000 (normal) und wartet
bis zu 2 Sekunden auf die Antwort des Servers, bevor sie auflegt. Der Zustand
liest sich dann als geschlossen mit 1000. Schließt der Server, zeigt der Zustand
seinen Code und Grund; bricht die Verbindung ohne ein Schließen-Frame ab, liest
sie sich als [[ui:ws.lost]], und die Konsole sagt, warum.

Signal Lab beantwortet die Pings des Servers selbst; Pings und Pongs werden
nicht aufgelistet. Eine Verbindung endet auch, wenn eine Nachricht über 16 MiB
eintrifft oder wenn das Senden einer solchen länger als 10 Sekunden dauert, weil
der Server aufgehört hat zu lesen. (Eine selbst gesendete über 16 MiB wird
abgelehnt und beendet nichts.)

## Im Inspektor {#inspector}

Läuft der Mitschnitt, erscheint der Verkehr der Verbindung mit dem Protokoll
`ws` und der Quelle `websocket`:

| Zusammenfassung | Was |
| --- | --- |
| `CONNECT ws://… (subprotocol)` | Die Verbindung wurde geöffnet |
| `TEXT …` | Eine Textnachricht und ihr Anfang |
| `BINARY n B …` | Eine Binärnachricht, ihre Größe und die ersten 16 Bytes |
| `CLOSE code reason` | Ein Schließen-Frame, gesendet oder empfangen |

Jeder Nachrichten-Frame behält seine Bytes. Ist der Verkehr leicht, wird jede
Nachricht mitgeschnitten; eine stark belastete Verbindung wird auf 200 Frames
pro Sekunde begrenzt, und der nächste mitgeschnittene Frame sagt, wie viele
weggelassen wurden (`+n not shown`). Siehe
[Inspektor](../tools/inspector.md).

## In Experimenten {#experiments}

Vier Schritte skripten ein WebSocket-Gespräch. Eine Verbindung wird von einem
Schritt geöffnet und von den anderen benannt:

| Schritt | Was er tut |
| --- | --- |
| [[ui:exp.node.ws_connect]] | Öffnet eine Verbindung für den Rest des Durchlaufs. Seine URL und Header nehmen `{{templates}}`, sodass ein früher extrahierter Token hineingehen kann. [Details](../experiments/nodes.md#node-ws_connect) |
| [[ui:exp.node.ws_send]] | Sendet eine Text- oder Binärnachricht über eine Verbindung. [Details](../experiments/nodes.md#node-ws_send) |
| [[ui:exp.node.wait_ws]] | Wartet auf eine Nachricht, deren Nutzdaten passen, wie es [[ui:exp.node.wait_udp]] tut; JSON-Nachrichten lassen sich danach Feld für Feld lesen. [Details](../experiments/nodes.md#node-wait_ws) |
| [[ui:exp.node.ws_close]] | Schließt eine Verbindung mit einem Schließen-Handshake: Code 1000 oder 3000–4999 für einen eigenen der Anwendung und ein Grund von bis zu 123 Bytes. [Details](../experiments/nodes.md#node-ws_close) |

Eine Verbindung, die der Durchlauf beim Ende — oder beim Stoppen — noch offen
hat, wird ordentlich geschlossen. Die Vorlage [[ui:exp.templateWsEcho]] ist ein
ausgearbeitetes Beispiel.

## Von der Kommandozeile {#cli}

`signallab send ws` führt einen Austausch aus: verbinden, eine Nachricht senden,
auf die Antwort warten, wenn Sie eine anfordern, schließen.

```bash
signallab send ws ws://127.0.0.1:9001/ --text '{"type":"ping"}' --expect pong
```

Der Handshake und das Gesendete gehen an die Standardfehlerausgabe, die Antwort
an die Standardausgabe:

```text
Connected to ws://127.0.0.1:9001/ in 4 ms
Sent 15 bytes
{"type":"pong"}
```

`--hex` sendet eine Binärnachricht; `-H` fügt einen Header hinzu und
`--protocol` bietet ein Subprotokoll an (beide wiederholbar). `--expect TEXT`,
`--expect-regex RE` oder `--wait` (irgendeine Nachricht) sagen, auf welche
Antwort gewartet wird, für `--timeout` Millisekunden (standardmäßig 2000). Es
endet mit 1, wenn die Antwort nicht kommt oder die Verbindung fehlschlägt. Siehe
[Kommandozeile](../automation/cli.md#cli-send-ws).

## Probleme {#troubleshooting}

| Was Sie sehen | Übliche Ursache |
| --- | --- |
| `… is not a WebSocket address` | Die URL beginnt nicht mit `ws://` oder `wss://` oder hat keinen Host. |
| `… answered HTTP n instead of switching to WebSocket` | Der Server hat das Upgrade abgelehnt: ein falscher Pfad (404), ein fehlender oder falscher Token (401, 403). Der Anfang seiner Antwort steht unter den technischen Details. |
| `… did not take any of the subprotocols offered` | Sie haben Subprotokolle angeboten, und der Server hat keines gewählt, oder er hat mit einem geantwortet, das Sie nicht angeboten haben. |
| `… is not a subprotocol name` | Ein Name mit einem Leerzeichen, einem Komma oder einem Schrägstrich. |
| `The header … cannot be sent with the upgrade` | Ein Header-Name oder -Wert mit Zeichen, die HTTP nicht erlaubt. |
| `… refused the connection` | Auf diesem Port empfängt nichts. |
| `A secure connection to … could not be made` | Dem Zertifikat wird hier nicht vertraut, oder TLS schlug fehl. Siehe [Sichere Verbindungen](#wss). |
| `The connection with … broke: the server did not keep to the WebSocket protocol` | Der Server hat etwas gesendet, das kein gültiges WebSocket ist. |
| `A WebSocket message is limited to … bytes` | Der Server hat eine Nachricht über 16 MiB gesendet, was die Verbindung beendet. |
| `Too long: at most 16777216` | Die Nachricht, die Sie senden wollten, ist über 16 MiB. Nichts wurde gesendet; die Verbindung ist offen. |

Jede Fehlermeldung steht unter
[Fehlermeldungen](../reference/errors.md#ws).

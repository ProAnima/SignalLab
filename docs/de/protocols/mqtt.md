---
title: MQTT
description: Mit einem MQTT-3.1.1-Broker verbinden, jedes Topic, das er hält, als lebendigen Baum beobachten, mit QoS 0, 1 oder 2 veröffentlichen, einen Last Will ankündigen und festhängende Retained-Werte löschen.
---

# MQTT

Die Ansicht [[ui:nav.mqtt]] ist ein MQTT-Client, um einen Broker anzusehen und zu
ändern, was in ihm steckt. Stellen Sie eine Verbindung her: Standardmäßig
abonniert sie `#`, sodass jedes Topic, das der Broker hält, als Baum mit seinem
letzten Wert entsteht. Von dort aus veröffentlichen Sie, löschen einen
Retained-Wert, speichern ein Topic als Signal oder machen es zu einem Schritt
eines Experiments.

Signal Lab spricht **MQTT 3.1.1 über einfaches TCP**, mit QoS 0, 1 und 2 zum
Abonnieren, Veröffentlichen und für den Last Will. Es gibt kein MQTT 5 und kein
TLS: Ein Broker, der nur `mqtts://` oder MQTT-5-Clients annimmt, ist nicht
erreichbar.

## Verbinden {#connect}

1. Öffnen Sie [[ui:nav.mqtt]].
2. Geben Sie [[ui:mq.host]] und [[ui:common.port]] ein.
3. Lassen Sie [[ui:mq.clientId]] so, wie sie ist, es sei denn, der Broker
   erwartet eine bestimmte. Fügen Sie [[ui:mq.username]] und [[ui:mq.password]]
   nur hinzu, wenn der Broker danach fragt.
4. Drücken Sie [[ui:mq.connect]].

Das Verbinden öffnet die TCP-Verbindung und schließt den MQTT-Handshake ab,
bevor etwas anderes geschieht; ein falsches Passwort oder ein geschlossener Port
wird also sofort dort gemeldet. Die Verbindungsfelder sind gesperrt, solange die
Verbindung steht; [[ui:mq.disconnect]] trennt sie. Die Verbindung ist ein Job in
der Leiste der Konsole und lässt sich auch dort stoppen.

| Feld | Was | Standard |
| --- | --- | --- |
| [[ui:mq.host]] | IP-Adresse oder Hostname des Brokers | `127.0.0.1` |
| [[ui:common.port]] | Der Port des Brokers | `1883` |
| [[ui:mq.clientId]] | Der Name Ihres Clients beim Broker. Er darf nicht leer sein und muss dort eindeutig sein: Ein zweiter Client mit derselben ID wirft den ersten hinaus. | `signal-lab-` und sechs zufällige Hex-Ziffern, bei jedem Start der App neu |
| [[ui:mq.username]], [[ui:mq.password]] | Werden nur gesendet, wenn der Broker sie braucht — im Klartext, da es kein TLS gibt. Ein Passwort ohne Benutzername wird gar nicht gesendet: MQTT 3.1.1 kann keines übertragen. | leer |
| [[ui:mq.keepAlive]] | Sekunden, die die Verbindung stumm bleiben darf. Signal Lab pingt den Broker nach der Hälfte davon; ein Broker trennt einen Client, der das 1,5-Fache dieser Zeit stumm bleibt. 0 schaltet das Pingen ab. | `60` |
| [[ui:mq.cleanSession]] | An: Jede Verbindung beginnt ohne gespeicherte Abonnements und ohne wartende Nachrichten. Aus bittet den Broker, sie für diese Client-ID zwischen den Verbindungen zu behalten. | an |
| [[ui:mq.scanFilter]] und sein [[ui:mq.qos]] | Ein Filter, der abonniert wird, sobald die Verbindung steht; `#` ist jedes Topic. Leer: keiner. | `#`, QoS 0 |
| [[ui:mq.willEnable]] | Dem Broker einen Last Will geben ([unten](#will)) | aus |

Der Broker hat 6 Sekunden, um die TCP-Verbindung anzunehmen, und weitere 6, um
den Handshake zu beantworten.

### Der Last Will {#will}

Ein Last Will ist eine Nachricht, die der Broker für Sie behält und selbst
veröffentlicht, wenn Ihre Verbindung ohne ordentlichen Abschied stirbt.
Anwesenheit wird üblicherweise so gebaut: Ein Gerät veröffentlicht `online` auf
sein Status-Topic, und sein Will setzt dasselbe Topic auf `off`.

Mit angehakten [[ui:mq.willEnable]] setzen Sie [[ui:mq.willTopic]] und
[[ui:mq.willPayload]] (standardmäßig `off`). Der Will wird mit QoS 2 und
retained veröffentlicht. Ohne Topic wird kein Will gesendet.

## Abonnieren {#subscribe}

Der Scan-Filter wird beim Verbinden abonniert. Für mehr:

1. Tippen Sie in [[ui:mq.addSubscription]] einen Topic-Filter.
2. Wählen Sie seinen [[ui:mq.qos]].
3. Drücken Sie [[ui:mq.subscribe]] oder <kbd>Enter</kbd>.

Ein Filter ist ein Topic mit Platzhaltern:

| Platzhalter | Steht für | Beispiel |
| --- | --- | --- |
| `+` | genau eine Ebene | `sensors/+/state` passt auf `sensors/door/state` |
| `#` | jede Ebene darunter, nur als letztes Zeichen | `sensors/#` passt auf `sensors/door/state` und `sensors` |

[[ui:mq.subscriptions]] listet jeden Filter mit dem, was der Broker gewährt hat:
`qos0`, `qos1` oder `qos2` — der Broker darf weniger gewähren als angefragt —
oder [[ui:mq.refused]]. [[ui:mq.unsubscribe]] bestellt einen ab.

| QoS | Zustellung |
| --- | --- |
| 0 | Höchstens einmal: gesendet und vergessen |
| 1 | Mindestens einmal: bestätigt, kann zweimal ankommen |
| 2 | Genau einmal: ein zweistufiger Handshake; eine erneute Zustellung wird nicht zweimal angezeigt |

## Der Topic-Baum {#topics}

Jede eintreffende Nachricht landet in [[ui:mq.topics]], einem Baum der
Topic-Ebenen. Ein Topic zeigt seinen letzten Wert, ein **R**, wenn dieser Wert
retained ist, und bei mehr als einer, wie viele Nachrichten es hatte. Klicken Sie
auf eine Ebene, um sie zu öffnen oder zu schließen.

- Tippen Sie in das Feld über dem Baum, um nur die Topics aufzulisten, deren
  Pfad oder letzter Wert den Text enthält.
- Über dem Baum stehen die Anzahl der Topics, wie viele einen Retained-Wert
  halten, und, solange die Verbindung steht, der Broker, auf den Sie hören.
- Nutzdaten werden als Text angezeigt; Bytes, die kein UTF-8 sind, erscheinen
  als Ersatzzeichen.
- [[ui:common.clear]] leert den Baum. Sonst tut das nichts: Er bleibt, wie er
  ist, wenn Sie die Ansicht wechseln oder die Verbindung trennen, bis die App
  geschlossen wird.

Nachrichten erreichen die Ansicht in Stapeln, zehnmal pro Sekunde. Sendet ein
Broker mehr als 4000 Nachrichten in einer Zehntelsekunde, werden die ältesten
dieses Stapels aus dem Baum gelassen und über ihm als „nicht angezeigt“ gezählt.

### Der Bereich eines Topics {#topic}

Wählen Sie ein Topic mit einem Wert, um ihn unter dem Baum zu sehen:
[[ui:mq.value]], [[ui:mq.qos]], [[ui:mq.retain]], [[ui:common.bytes]],
[[ui:mq.messages]] und [[ui:mq.lastAt]]. Seine Schaltflächen:

| Schaltfläche | Tut |
| --- | --- |
| [[ui:mq.editHere]] | Kopiert Topic, Wert, QoS und Retain-Flag in [[ui:mq.publish]] |
| [[ui:mq.waitForThis]] | Fügt dem offenen Experiment einen Schritt [Auf MQTT warten](../experiments/nodes.md#node-wait_mqtt) auf dieses Topic, an diesem Broker, beliebige Nutzdaten, 2000 ms Timeout, hinzu |
| [[ui:mq.clearRetained]] | Entfernt den Retained-Wert ([unten](#clear-retained)) |
| [[ui:sig.fromFrame]] | Behält das Topic und seinen letzten Wert als Signal im Ordner [[ui:sig.capturedFolder]] |

## Veröffentlichen {#publish}

1. Stellen Sie eine Verbindung her.
2. Geben Sie unter [[ui:mq.publish]] das [[ui:mq.topic]] und die
   [[ui:sig.payload]] ein.
3. Wählen Sie den [[ui:mq.qos]] und haken Sie [[ui:mq.retain]] an, wenn der
   Broker die Nachricht als Wert des Topics für jeden Client behalten soll, der
   später abonniert.
4. Drücken Sie [[ui:mq.publishBtn]].

Die Konsole bestätigt jede Veröffentlichung: bei QoS 0 sofort, bei QoS 1 und 2,
sobald der Broker sie bestätigt hat. Ein Topic zum Veröffentlichen hat keine
Platzhalter und ist nicht leer: Ein Topic mit `+` oder `#` wird abgelehnt, bevor
etwas gesendet wird, mit derselben Meldung, die ein Signal, ein Schritt und
`signallab send mqtt` geben, und die Verbindung bleibt, wie sie war. Nur ein
Abonnement nimmt Filter mit Platzhaltern an.

### Einen Retained-Wert löschen {#clear-retained}

Ein Retained-Wert bleibt auf dem Broker, bis er ersetzt wird, und jeder Client,
der abonniert, erhält ihn zuerst — ein veralteter ist ein klassischer Grund,
warum ein Gerät im falschen Zustand startet. Der einzige Weg, ihn zu entfernen,
ist, leere Nutzdaten mit gesetztem Retain zu veröffentlichen.

[[ui:mq.clearRetained]] im Bereich eines Topics tut das: Drücken Sie es, dann
[[ui:mq.clearConfirm]]. Es veröffentlicht die leeren Retained-Nutzdaten mit
QoS 1 über Ihre Verbindung. Es ist nur verfügbar, solange die Verbindung steht
und wenn der letzte Wert des Topics retained ist. Von Hand geht dasselbe: leere
[[ui:sig.payload]] mit angehakten [[ui:mq.retain]].

::: warning
Das Löschen verändert den Broker für alle Clients auf einmal.
:::

## Im Inspektor {#inspector}

Ist der Mitschnitt gestartet, erscheint MQTT-Verkehr mit dem Protokoll `mqtt`:

| Quelle | Was | Wie viele |
| --- | --- | --- |
| `mqtt` | Was die Verbindung der Ansicht veröffentlicht; eine leere Retained-Veröffentlichung hat das Urteil `clears retained` | jede einzelne |
| `mqtt` | Nachrichten, die die Verbindung empfängt | höchstens eine alle 200 ms |
| `mqtt-send` | Eine Veröffentlichung, die ihre eigene Verbindung mitbrachte: ein Signal, das gesendet wurde, während die Ansicht nicht mit dem Broker des Signals verbunden ist, ein Schritt, `signallab send mqtt` (Urteil `one-shot`) | jede einzelne |
| `experiment-wait` | Nachrichten, die das Abonnement eines Schritts [[ui:exp.node.wait_mqtt]] empfängt, abgesehen von erneut zugestellten Retained-Werten | jede einzelne |

Die Zusammenfassung liest sich `topic = payload`, mit dem QoS und `retained`,
wenn sie zutreffen. Siehe [Inspektor](../tools/inspector.md).

## Speichern und wiederverwenden {#library}

- **Als Signal speichern.** [[ui:sig.saveNew]] unter [[ui:mq.publish]] behält
  Broker (den [[ui:mq.host]] und [[ui:common.port]] der Verbindung), Topic,
  Nutzdaten, QoS und Retain-Flag in der Signalbibliothek;
  <kbd>Ctrl</kbd>+<kbd>S</kbd> im Bereich zum Veröffentlichen tut dasselbe und
  aktualisiert das Signal, sobald der Bereich daran gebunden ist. Siehe
  [Signale](../tools/signals.md).
- **Ein MQTT-Signal auslösen.** Während diese Ansicht mit dem Broker des Signals
  verbunden ist (derselbe Host, Groß- und Kleinschreibung egal, und derselbe
  Port; `1883`, wenn das Signal keinen angibt), geht ein aus der Bibliothek
  ausgelöstes Signal über diese Verbindung hinaus, mit ihrer Client-ID und ihren
  Anmeldedaten. Andernfalls — nicht verbunden oder mit einem anderen Broker
  verbunden — öffnet es eine eigene Verbindung zu seinem eigenen Broker — eine
  neue Client-ID, kein Benutzername —, veröffentlicht, wartet auf die
  Bestätigung, die sein QoS verlangt, und trennt. Namen werden nicht
  nachgeschlagen, `localhost` und `127.0.0.1` gelten also als verschiedene
  Broker. Die Bibliothek speichert kein Passwort.
- **In einem Experiment.** Ein gespeichertes MQTT-Signal lässt sich unter
  [[ui:exp.group.signals]] im Menü [[ui:exp.addNode]] des Experiments wählen,
  wodurch es ein Schritt [[ui:exp.node.mqtt]] wird.

## In Experimenten {#experiments}

| Schritt | Was er tut |
| --- | --- |
| [[ui:exp.node.mqtt]] | Verbindet, veröffentlicht eine Nachricht und trennt — ohne Benutzername oder Passwort, mit Clean Session, innerhalb von 15 Sekunden. [Details](../experiments/nodes.md#node-mqtt) |
| [[ui:exp.node.wait_mqtt]] | Abonniert beim Start des Durchlaufs und wartet auf eine Nachricht auf einem Topic-Filter, deren Nutzdaten passen; beim Abonnieren erneut zugestellte Retained-Werte werden ignoriert. [Details](../experiments/nodes.md#node-wait_mqtt) |
| [[ui:exp.node.emulator]] | Ein eigener MQTT-Broker des Durchlaufs. [Details](../experiments/nodes.md#node-emulator) |

Keiner der beiden Schritte meldet sich an, sie brauchen also einen Broker, der
Clients ohne Benutzername annimmt.

### Der Broker-Emulator {#broker-emulator}

Signal Lab kann auch der Broker sein: Ein Emulator [[ui:emu.new.mqtt]] leitet
weiter, was Clients veröffentlichen, an jeden, der abonniert hat — 3.1.1,
einfaches TCP, QoS 0, 1 und 2, Retained-Nachrichten, Wills, eine optionale
Anmeldung — und antwortet nach Regeln, wie ein Gerät. Richten Sie Ihre Technik
und diese Ansicht darauf, um ohne echten Broker zu testen.
Siehe [Emulatoren](../tools/emulators.md).

## Von der Kommandozeile {#cli}

`signallab send mqtt` veröffentlicht eine Nachricht mit einer eigenen Verbindung:

```bash
signallab send mqtt 127.0.0.1:1883 lab/light/1/set on --qos 1
signallab send mqtt 127.0.0.1:1883 lab/light/1/state "" --retain
```

```text
✔ lab/light/1/set → 127.0.0.1:1883 · 2 B · qos1
```

Die zweite Zeile löscht einen Retained-Wert. Ohne Port liegt der Broker auf
1883. Es werden keine Anmeldedaten verwendet. Der Befehl endet mit 0, wenn der
Broker die Nachricht annahm, mit 1, wenn er nicht erreichbar war oder sie
abgelehnt hat. Siehe
[Kommandozeile](../automation/cli.md#cli-send-mqtt).

## Probleme {#troubleshooting}

| Was Sie sehen | Übliche Ursache |
| --- | --- |
| `… refused the connection — nothing is listening on that port` | Kein Broker auf dieser Adresse und diesem Port. |
| `… accepted the connection but did not answer in time — is it an MQTT broker?` | Dort hört etwas, aber es spricht kein MQTT oder spricht es über TLS. |
| `… answered with something other than MQTT 3.1.1` | Kein MQTT-Broker, oder ein Broker, der etwas gesendet hat, das Signal Lab nicht lesen kann. |
| `… does not accept MQTT 3.1.1 clients` | Der Broker nimmt nur MQTT 5. |
| `… rejected the client ID — choose another one` | Die ID ist zu lang oder enthält Zeichen, die der Broker nicht annimmt. |
| `… rejected the username or password` | Falsche Anmeldedaten oder ein Passwort ohne Benutzername. |
| `… did not authorize this client — check its access rules` | Die Zugriffsregeln des Brokers lehnen diesen Client ab. |
| `… is unavailable right now — try again later` | Der Broker läuft, nimmt aber keine Clients an. |
| `Enter a client ID — brokers refuse an empty one` | [[ui:mq.clientId]] ist leer. |
| `A publish topic cannot contain the wildcards + or #` | Das Topic, an das veröffentlicht werden soll, enthält ein `+` oder `#`. Diese sind zum Abonnieren; veröffentlichen Sie jeweils an ein Topic. |
| Ein Filter zeigt [[ui:mq.refused]] | Die Zugriffsregeln des Brokers verbieten ihn, oder der Filter ist fehlerhaft (`#` nicht zuletzt, `+` teilt sich eine Ebene mit anderen Zeichen). |
| Die Verbindung bricht kurz nach dem Verbinden ab | Ein anderer Client hat sich mit derselben [[ui:mq.clientId]] verbunden. |
| Im Baum erscheint nichts | Der Scan-Filter ist leer, oder der Broker lässt diesen Client nichts sehen. |

Jede Fehlermeldung steht unter [Fehlermeldungen](../reference/errors.md#mqtt).

---
title: Störungen
description: Störungs-Relais und Emulatoren als Knoten eines Durchlaufs — ein schlechtes Netzwerk und eine ausfallende Abhängigkeit, auf Kommando geschaltet, im Bericht Phase für Phase gezählt und mit dem Startwert wiederholbar.
---

# Störungen als Knoten

Um zu sehen, wie ein System zurechtkommt, wenn das Netzwerk schlechter wird oder eine Abhängigkeit
ausfällt, nehmen Sie die Störung ins Experiment auf. Ein Relais oder ein Emulator öffnet mit dem
Durchlauf, ein Schritt schaltet ihn auf Kommando um, und der Bericht des Durchlaufs zählt, was in jeder
Phase geschah. Das Ende des Durchlaufs — bestanden, fehlgeschlagen oder gestoppt — schließt beide, sodass
danach nichts gestört bleibt.

| Knoten | Was er tut |
| --- | --- |
| [[ui:exp.node.impairment]] | ein Relais zwischen dem getesteten System und seinem Ziel, das stört, was hindurchgeht, für den ganzen Durchlauf |
| [[ui:exp.node.impairment_change]] | schaltet ein Relais des Durchlaufs ab diesem Schritt auf ein anderes Profil um |
| [[ui:exp.node.emulator]] | eine API, ein Gerät oder ein Broker, gespielt von Signal Lab, für den ganzen Durchlauf |
| [[ui:exp.node.emulator_state]] | schaltet einen Emulator des Durchlaufs aus oder wieder ein |

Alle vier stehen im Menü zum Hinzufügen unter [[ui:exp.group.fault]] und
[[ui:exp.group.emulate]]. Ihre Felder stehen in der
[Referenz der Knoten](nodes.md); das Relais selbst ist unter
[Störung](../tools/impairment.md) beschrieben, die Emulatoren unter
[Emulatoren](../tools/emulators.md).

## Störung {#impairment}

Das getestete System sendet an das Relais statt an sein echtes Ziel; das Relais
leitet an das Ziel weiter, trägt die Antworten zurück und stört beide Richtungen.

| Feld | Was |
| --- | --- |
| [[ui:exp.relayListen]] | `IP:port`, an das das getestete System sendet oder mit dem es sich verbindet, Port nicht 0 |
| [[ui:exp.relayTarget]] | `IP:port` des echten Ziels, oder `host:port`; ein Hostname wird beim Start des Durchlaufs aufgelöst |
| [[ui:ns.protocol]] | UDP — jedes Datagramm erleidet sein eigenes Schicksal — oder TCP — jede Verbindung wird mit einer eigenen zum Ziel gekoppelt |
| das Profil | eine [[ui:ns.preset]] oder eigene Werte |

Was ein Relais aus seinem Profil liest, hängt vom Protokoll ab; die übrigen Werte
bleiben unberücksichtigt:

| Protokoll | Störungen |
| --- | --- |
| UDP | Latenz, Jitter, Paketverlust, Burst-Verlust, Duplizierung, Verfälschung, Umordnung, eine Bandbreitengrenze, offline |
| TCP | Latenz und Jitter (ein Datenstrom bleibt in der richtigen Reihenfolge), eine Bandbreitengrenze (der Sender wird gebremst, nichts verworfen), zurückgesetzte Verbindungen, halboffen gelassene Verbindungen, offline |

**Vor dem ersten Schritt geöffnet.** Jedes Relais des Experiments öffnet beim Start des
Durchlaufs, wie die Sockets der Warteknoten, daher nehmen seine Felder [[ui:exp.relayListen]] und
[[ui:exp.relayTarget]] nur Text und Parameter an
(`node.params_only`) — `{{relay}}` mit einem Parameter `relay`, nie eine Variable.
Ein Port, der sich nicht öffnen lässt, oder ein Zielname, der sich nicht auflösen lässt, stoppt den Durchlauf vor jedem Verkehr, am Knoten.

**Im Ablauf sofort weiter.** Erreicht der Durchlauf den Knoten, geht es sofort weiter, und
die Zeitleiste sagt, was er stört und womit. Das Relais arbeitet vom
Start des Durchlaufs bis zu seinem Ende, wo immer der Knoten im Graphen steht.

**Mit dem Durchlauf geschlossen.** Wie auch immer der Durchlauf endet, das Relais schließt sich; die
Verbindungen eines TCP-Relais schließen sich mit ihm. Ein Relais, das von selbst aufgehört hat weiterzuleiten, behält den
Grund: Die Schritte, die es verwenden, schlagen damit fehl, und der Bericht sagt es.

::: tip Über eine Störung leiten
An einem Knoten [[ui:exp.node.osc]] oder [[ui:exp.node.udp]] setzt
[[ui:exp.routeThrough]] eine Störung davor: Das Relais empfängt
auf einem freien Port von `127.0.0.1`, leitet mit der Vorgabe
[[ui:ns.preset.lan]] an das Ziel des Knotens weiter, und der Knoten sendet nun an das Relais.
:::

## Störung ändern {#change-impairment}

[[ui:exp.node.impairment_change]] nennt im Feld [[ui:exp.relay]] eines der Relais des
Experiments und gibt das Profil an, mit dem es ab diesem Schritt stört. Das
Relais behält seinen Port und seine Verbindungen; die neuen Werte gelten ab dem nächsten
Paket oder Block. Die Zeitleiste zeigt das neue Profil.

Jede Änderung beendet eine **Phase**. Der Bericht des Durchlaufs hält für jedes Relais fest:

- seine Empfangs- und Zieladresse und sein Protokoll, wenn es TCP ist;
- seine Gesamtzahlen: empfangen, weitergeleitet, verworfen, gedrosselt, dupliziert,
  verfälscht, umgeordnet, Bytes — und bei TCP die Verbindungen, die zurückgesetzten und
  die halboffen gelassenen;
- jede Phase: den Namen des Profils, wann sie begann und endete, in Millisekunden ab
  dem Öffnen des Relais, und dieselben Zahlen für diese Phase allein.

Ein Paket wird in der Phase gezählt, die über sein Schicksal entschieden hat, auch wenn seine verzögerte
Kopie erst nach der Umschaltung hinausgeht. Ein Relais behält seine letzten 1000 Phasen; ältere werden
gezählt, nicht aufbewahrt.

Ein Knoten [[ui:exp.node.impairment_change]], der kein Relais des Experiments nennt, wird
abgelehnt (`impair.relay_unknown`).

## Emulator {#emulator}

Der Knoten [[ui:exp.node.emulator]] spielt für den ganzen Durchlauf eine Abhängigkeit: eine HTTP-API, ein
OSC-, UDP- oder TCP-Gerät oder einen MQTT-Broker. Es ist derselbe Emulator, den die Ansicht
[Emulatoren](../tools/emulators.md) eigenständig ausführt:
[[ui:emu.edit]] öffnet seine Regeln, [[ui:emu.toLibrary]] legt eine Kopie in der
Bibliothek ab, [[ui:emu.fromLibrary]] übernimmt einen von dort.

- Er öffnet vor dem ersten Schritt und antwortet, bis der Durchlauf endet; ein Port, der sich nicht
  öffnen lässt, stoppt den Durchlauf vor jedem Verkehr. Im Ablauf geht es sofort
  weiter.
- Seine Adresse ist ein festes `IP:port`. Seine Abgleichmuster nehmen nur Parameter
  an; seine Antworten sind Vorlagen, gelesen mit dem Eingetroffenen (`{{request.…}}`)
  und den Parametern des Durchlaufs. Geheimnisse kann er nicht lesen.
- Seine Zufallsentscheidungen — eine gewichtete Mischung von Antworten, Jitter der Verzögerung, Generatoren in
  Antworten — werden aus dem Startwert des Durchlaufs gezogen.
- Ein HTTP-Emulator ist auch das, worauf ein Knoten [[ui:exp.node.wait_http]] an derselben Adresse
  hört: Damit prüfen Sie, was das getestete System gesendet hat. Ohne Emulator
  dort beantwortet der eigene Empfänger des Durchlaufs jede Anfrage mit `204`.
- Ein OSC- oder UDP-Emulator teilt seinen Port mit den Warteknoten des Durchlaufs dort: Beide sehen
  jedes Datagramm.
- Ein MQTT-Emulator ist ein Broker, den die Knoten [[ui:exp.node.mqtt]] und
  [[ui:exp.node.wait_mqtt]] des Durchlaufs wie jeden anderen nutzen können.

Der Bericht des Durchlaufs hält für jeden Emulator-Knoten Name, Protokoll und Adresse fest
sowie seine Zahlen: Anfragen insgesamt, die ohne Regel, die fehlgeschlagenen, die im Ausfall
eingetroffenen, Nachrichten, die ein Broker einem langsamen Client nicht zustellen konnte, und
die Treffer jeder Regel.

## Emulator aus und an {#emulator-state}

[[ui:exp.node.emulator_state]] nennt im Feld [[ui:exp.emulatorNode]] einen der Emulatoren des Durchlaufs;
[[ui:exp.emulatorDownState]] ist [[ui:exp.emulatorGoesDown]] oder
[[ui:exp.emulatorComesUp]]. Während des Ausfalls:

| Emulator | Worauf der Client trifft |
| --- | --- |
| HTTP | was [[ui:exp.downFault]] vorgibt: [[ui:emu.outageFault.unavailable]] (`503`, Body `{"error":"unavailable"}`), [[ui:emu.outageFault.reset]] oder [[ui:emu.outageFault.timeout]] — die Anfrage wird festgehalten, bis der Client aufgibt, höchstens 120 s |
| TCP-Gerät, MQTT-Broker | Verbindungen werden getrennt und neue verweigert |
| OSC-, UDP-Gerät | nichts wird beantwortet |

Was während des Ausfalls eintrifft, wird als `down` gezählt, nie als Anfrage ohne Regel.
Ein Knoten [[ui:exp.node.wait_http]] sieht die Anfragen weiterhin. Der Emulator bleibt
aus, bis ein Schritt ihn wieder einschaltet, unabhängig von seinem eigenen Ausfallplan, und das
Ende des Durchlaufs schließt ihn in jedem Fall.

Ein Knoten [[ui:exp.node.emulator_state]], der keinen Emulator des Experiments nennt, wird
abgelehnt (`emulator.node_unknown`).

## Ausfälle nach Zeitplan {#outage}

Ein Emulator kann auch von selbst ausfallen: In seinen Regeln legt [[ui:emu.outage]] die Werte
[[ui:emu.outageUp]] und [[ui:emu.outageDown]] fest, jeweils 10–3.600.000 ms, und für HTTP
[[ui:emu.outageFault]]. Er antwortet für die erste Dauer, fällt für die zweite aus und so weiter,
gezählt ab seinem Öffnen — in einem Durchlauf also vor dem ersten
Schritt. Während eines geplanten Ausfalls trägt die `503` eines HTTP-Emulators
`Retry-After` mit den ganzen Sekunden bis zur Rückkehr, mindestens 1; eine `503`,
während ein Schritt [[ui:exp.node.emulator_state]] ihn ausgeschaltet hält, trägt keines, da
niemand weiß, wann das endet.

Ein Zeitplan braucht keinen Schritt; ein Schritt braucht keinen Zeitplan. Verwenden Sie den Zeitplan für eine
Abhängigkeit, die immer wieder wegbricht, und den Schritt für einen Ausfall an einer gewählten Stelle des Ablaufs.

## Beispiel: ein Ausfall hinter einer langsamen Leitung {#example}

Ein Client fragt über ein Relais eine API nach einem Auftrag. Während er fragt, verlangsamt ein zweiter
Zweig die Leitung auf [[ui:ns.preset.4g]], schaltet die API für zwei
Sekunden aus, schaltet sie wieder ein und macht die Leitung wieder sauber. Der Client muss so lange
fragen, bis er seine Antwort erhält.

```text
start → orders → link → split
split ─ branch1 → settle → until_ok ─ done → answered → joined
                           until_ok ─ body → get → status → pause → until_ok
split ─ branch2 → slow → down → outage → up → clean → joined
joined → end
```

Die Namen sind die IDs der Knoten in der Datei unten.

1. Fügen Sie einen Parameter `api` = `http://127.0.0.1:18091` hinzu — das Relais, nicht die API.
2. Fügen Sie einen Knoten [[ui:exp.node.emulator]] hinzu: HTTP, `127.0.0.1:18090`, eine Route
   `GET /orders/:id`, die mit `200` und `{"order":"{{request.params.id}}"}` antwortet.
3. Danach einen Knoten [[ui:exp.node.impairment]]: [[ui:exp.relayListen]]
   `127.0.0.1:18091`, [[ui:exp.relayTarget]] `127.0.0.1:18090`,
   [[ui:ns.protocol]] TCP, Vorgabe [[ui:ns.preset.lan]].
4. Danach einen Knoten [[ui:exp.node.fork]].
5. An [[ui:exp.branch1]] hängt der Client: eine [[ui:exp.node.delay]] von 300 ms, dann eine
   [[ui:exp.node.loop]] — [[ui:exp.loopMax]] 40, [[ui:exp.loopUntilOn]]
   `{{status}}` [[ui:exp.op.eq]] `200`. Ihr Rumpf: eine [[ui:exp.node.http]]
   `GET {{api}}/orders/42`, ein Knoten [[ui:exp.node.extract]], der den
   [[ui:exp.from.status]] in `status` ablegt, eine Verzögerung von 250 ms, zurück zur
   Schleife verbunden. Am Ausgang [[ui:exp.portDone]] eine
   [[ui:exp.node.log]] `Orders API answers again: HTTP {{status}}`.
6. An [[ui:exp.branch2]] hängen die Störungen: ein Knoten [[ui:exp.node.impairment_change]], der
   das Relais auf [[ui:ns.preset.4g]] umschaltet; ein Knoten [[ui:exp.node.emulator_state]], der
   die Orders API mit [[ui:emu.outageFault.unavailable]] auf [[ui:exp.emulatorGoesDown]] stellt;
   eine Verzögerung von 2000 ms; ein weiterer Knoten
   [[ui:exp.node.emulator_state]], der sie auf [[ui:exp.emulatorComesUp]] stellt;
   ein weiterer Knoten [[ui:exp.node.impairment_change]] zurück auf [[ui:ns.preset.lan]].
7. Verbinden Sie beide Zweige mit einem Knoten [[ui:exp.node.join]] und diesen mit
   [[ui:exp.node.end]].
8. Führen Sie es aus.

Die Zeitleiste zeigt die Anfragen des Clients, die über die langsame Leitung mit `503` beantwortet werden,
die Rückkehr der API, dann `200` und wie die Schleife über
[[ui:exp.portDone]] verlassen wird. Der Bericht zählt etwa fünf Anfragen, die die API
im Ausfall antrafen, und eine, die ihre Route beantwortete, sowie die drei Phasen des Relais —
[[ui:ns.preset.lan]] für einen Augenblick, [[ui:ns.preset.4g]] während des Ausfalls,
dann wieder [[ui:ns.preset.lan]] — jede mit ihrem eigenen Verkehr.

::: details Das Experiment als Datei
Speichern Sie es als `.json`-Datei und öffnen Sie es mit [[ui:exp.importJson]] unter
[[ui:exp.documents]].

```json
{
  "version": 9,
  "name": "Outage behind a slow link",
  "params": [{ "name": "api", "value": "http://127.0.0.1:18091" }],
  "profiles": [],
  "profile": null,
  "seed": null,
  "nodes": [
    { "id": "start", "type": "start", "x": 40, "y": 270 },
    { "id": "orders", "type": "emulator", "x": 260, "y": 270,
      "emulator": { "name": "Orders API", "bind": "127.0.0.1:18090", "protocol": "http",
        "routes": [{ "method": "GET", "path": "/orders/:id", "when": [], "order": "sequence",
          "responses": [{ "status": 200, "headers": [], "body": "{\"order\":\"{{request.params.id}}\"}", "delay_ms": 0, "jitter_ms": 0, "fault": "none", "weight": 1 }] }],
        "fallback": null } },
    { "id": "link", "type": "impairment", "x": 490, "y": 270, "listen": "127.0.0.1:18091", "target": "127.0.0.1:18090", "protocol": "tcp",
      "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } },
    { "id": "split", "type": "fork", "x": 720, "y": 270 },
    { "id": "settle", "type": "delay", "x": 950, "y": 140, "ms": 300 },
    { "id": "until_ok", "type": "loop", "x": 1180, "y": 140, "max": 40,
      "until": { "value": "{{status}}", "op": "eq", "expected": "200" } },
    { "id": "get", "type": "http", "x": 1410, "y": 20,
      "request": { "method": "GET", "url": "{{api}}/orders/42", "headers": [], "body": null, "timeout_ms": 3000 } },
    { "id": "status", "type": "extract", "x": 1640, "y": 20, "variable": "status", "from": "status", "expr": "" },
    { "id": "pause", "type": "delay", "x": 1870, "y": 20, "ms": 250 },
    { "id": "answered", "type": "log", "x": 1410, "y": 140, "message": "Orders API answers again: HTTP {{status}}" },
    { "id": "slow", "type": "impairment_change", "x": 950, "y": 400, "relay": "link",
      "profile": { "name": "4g", "latency_ms": 60, "jitter_ms": 25, "rate_kbps": 20000 } },
    { "id": "down", "type": "emulator_state", "x": 1180, "y": 400, "emulator": "orders", "down": true, "fault": "unavailable" },
    { "id": "outage", "type": "delay", "x": 1410, "y": 400, "ms": 2000 },
    { "id": "up", "type": "emulator_state", "x": 1640, "y": 400, "emulator": "orders", "down": false, "fault": "unavailable" },
    { "id": "clean", "type": "impairment_change", "x": 1870, "y": 400, "relay": "link",
      "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } },
    { "id": "joined", "type": "join", "x": 2100, "y": 270 },
    { "id": "end", "type": "end", "x": 2330, "y": 270 }
  ],
  "edges": [
    { "from": "start", "to": "orders" },
    { "from": "orders", "to": "link" },
    { "from": "link", "to": "split" },
    { "from": "split", "to": "settle", "port": "branch1" },
    { "from": "split", "to": "slow", "port": "branch2" },
    { "from": "settle", "to": "until_ok" },
    { "from": "until_ok", "to": "get", "port": "body" },
    { "from": "get", "to": "status" },
    { "from": "status", "to": "pause" },
    { "from": "pause", "to": "until_ok" },
    { "from": "until_ok", "to": "answered", "port": "done" },
    { "from": "answered", "to": "joined" },
    { "from": "slow", "to": "down" },
    { "from": "down", "to": "outage" },
    { "from": "outage", "to": "up" },
    { "from": "up", "to": "clean" },
    { "from": "clean", "to": "joined" },
    { "from": "joined", "to": "end" }
  ]
}
```
:::

Zwei Vorlagen unter [[ui:exp.documents]] tun dasselbe auf andere Weise:
[[ui:exp.templateFaults]] sendet Datagramme an ein emuliertes Gerät, über ein UDP-Relais,
das auf sauber, verlustbehaftet, offline und wieder sauber geschaltet wird;
[[ui:exp.templateOutage]] schaltet eine emulierte API für zwei Sekunden aus, während ein
Client weiter fragt.

## Ports {#ports}

Die Sockets eines Durchlaufs — Warteknoten, Antworten, Emulatoren, Relais — können sich keinen
Port desselben Protokolls teilen; ein UDP- und ein TCP-Socket dürfen dieselbe Nummer verwenden. Beim
Feld [[ui:exp.relayListen]] eines Relais kollidiert eine Adresse auf `0.0.0.0` mit jeder
Adresse auf demselben Port.

| Socket | Kann seinen Port nicht teilen mit |
| --- | --- |
| ein HTTP-, TCP- oder MQTT-Emulator | einem anderen davon (`emulator.bind_taken`) |
| ein OSC- oder UDP-Emulator | einem anderen davon (`emulator.bind_taken`) |
| ein TCP- oder MQTT-Emulator | einem Knoten [[ui:exp.node.wait_http]] (`emulator.bind_taken`) |
| [[ui:exp.relayListen]] eines UDP-Relais | einem anderen UDP-Relais, einem OSC- oder UDP-Emulator, einem Warteknoten oder dem Socket einer Antwort (`impair.bind_taken`) |
| [[ui:exp.relayListen]] eines TCP-Relais | einem anderen TCP-Relais, einem HTTP-, TCP- oder MQTT-Emulator, einem Knoten [[ui:exp.node.wait_http]] (`impair.bind_taken`) |

Absichtlich geteilt: ein HTTP-Emulator und die Schritte [[ui:exp.node.wait_http]] an
seiner Adresse; ein OSC- oder UDP-Emulator und die Warteknoten auf seinem Port; Warteknoten auf einer
Adresse untereinander.

Ein Relais darf nicht in sich selbst weiterleiten, weder direkt noch über andere Relais: Sein
Verkehr würde auf Loopback kreisen (`impair.loop`). Zwei Relais hintereinander vor
einem Gerät sind in Ordnung.

## Einen gestörten Durchlauf wiederholen {#seed}

Jede Entscheidung eines Relais — ob ein Paket verloren geht, dupliziert, verfälscht
oder zurückgehalten wird, wie viel Jitter es erhält — wird aus dem Startwert des Durchlaufs gezogen, getrennt
für jede Richtung und Paket für Paket. Auch die Zufallsentscheidungen eines Emulators stammen
daraus. Führen Sie mit demselben Startwert und demselben Verkehr erneut aus, erleiden dieselben
Pakete dasselbe Schicksal: Ein Fehler, den Sie einmal gesehen haben, lässt sich wieder sehen.

Um den Startwert zu behalten, drücken Sie daneben in der Zeitleiste [[ui:exp.pinSeed]] oder führen Sie
über [[ui:exp.runWith]] damit aus; siehe [Startwerte](runs.md#seeds). Was der Startwert
nicht festhalten kann, ist das Timing: wann das getestete System sendet und damit, in welche
Phase ein Paket fällt.

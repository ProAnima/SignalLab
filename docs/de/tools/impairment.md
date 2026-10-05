---
title: Störung
description: Ein Relais zwischen einen Client und sein Ziel setzen, das UDP-Datagramme verzögert, verwirft, dupliziert, umordnet oder drosselt oder TCP-Datenströme verzögert, drosselt, zurücksetzt oder halboffen lässt.
---

# Störung

Die Ansicht [[ui:nav.netsim]] betreibt ein Relais, das zwischen einem Client und dem Ziel
sitzt, mit dem er spricht, und verschlechtert, was hindurchgeht, in beide Richtungen: ein
schlechtes WLAN, eine Mobilfunkverbindung, ein Satellitensprung, eine Leitung, die ausfällt.
Nutzen Sie es, um zu sehen, wie sich Ihr Client und Ihr Gerät in einem Netzwerk verhalten,
das Sie nicht zur Hand haben.

Sie richten den Client auf das Relais statt auf das echte Ziel; das Relais gibt alles an das
Ziel weiter und die Antworten zurück an den Client, nachdem es mit jedem Datagramm oder
Datenstrom getan hat, was sein Profil sagt. Sie können das Profil im laufenden Betrieb ändern,
ohne den Port aufzugeben.

## Ein Relais starten {#start}

1. Wählen Sie das [[ui:ns.protocol]]: UDP für Datagramme (OSC, der meiste Show-Control- und
   Sensorverkehr), TCP für Datenströme (HTTP, MQTT, ein TCP-Gerät).
2. Setzen Sie [[ui:ns.listen]]: das `IP:port`, auf dem das Relais empfängt. `0.0.0.0:9010`
   nimmt Verkehr aus dem Netzwerk an; `127.0.0.1:9010` nur von diesem Computer.
3. Setzen Sie [[ui:ns.target]]: das `IP:port` des echten Ziels oder einen Hostnamen mit Port
   wie `device.local:9000`.
4. Wählen Sie eine [Vorgabe](#presets) oder setzen Sie die Werte des [Profils](#profile).
5. Drücken Sie [[ui:ns.startRelay]].
6. Richten Sie Ihren Client auf den Port des Relais statt auf das Ziel — zum Beispiel
   `127.0.0.1:9010` statt `127.0.0.1:9000`.

[[ui:ns.stopRelay]] schließt das Relais; nichts, was es noch zurückhielt, geht danach hinaus.

Standardwerte: UDP, Empfang auf `0.0.0.0:9010`, Ziel `127.0.0.1:9000`, 40 ms Latenz, 15 ms
Jitter und 2 % Verlust.

[[ui:ns.listen]] ist eine zu bindende Adresse, immer `IP:port`. Ein Hostname in
[[ui:ns.target]] wird einmal nachgeschlagen, wenn Sie [[ui:ns.startRelay]] drücken, so wie
ein [OSC](../protocols/osc.md)-Ziel; ein Name, der sich nicht finden lässt, wird abgelehnt und
kein Relais startet. [[ui:ns.protocol]], [[ui:ns.listen]] und [[ui:ns.target]] sind fest,
solange das Relais läuft; stoppen Sie es, um sie zu ändern.

```text
client ──► relay (0.0.0.0:9010) ──► target (127.0.0.1:9000)
client ◄── relay ◄───────────────── target
```

### UDP {#udp}

Das Relais sendet jedes Datagramm von einem eigenen Port an das Ziel und sendet zurück, was
das Ziel dem Client antwortet. Jedes Datagramm erleidet sein eigenes Schicksal, entschieden,
wenn es eintrifft.

Antworten gehen an den Client, der das jüngste Datagramm gesendet hat: Das Relais dient einem
Client nach dem anderen.

### TCP {#tcp}

Jede Verbindung, die ein Client zum Relais aufbaut, wird mit einer neuen eigenen Verbindung
des Relais zum Ziel gekoppelt. Beide Datenströme jeder Verbindung — Client zum Ziel und Ziel
zum Client — werden Block für Block gestört, wie das Relais sie liest (bis zu 16 KiB auf
einmal). Ein Datenstrom kommt immer in der richtigen Reihenfolge an, egal welcher Jitter.

Wenn das Ziel die Verbindung verweigert, wird die Verbindung des Clients zurückgesetzt.

## Das Profil {#profile}

Die Werte, die ein Relais in beide Richtungen anwendet. Ein UDP-Relais liest die Werte des
Datagramms, ein TCP-Relais die des Datenstroms; die übrigen sind ausgeblendet.

### Über UDP {#profile-udp}

| Wert | Was er tut | Bereich |
| --- | --- | --- |
| [[ui:ns.offline]] | Jedes Datagramm wird verworfen, in beide Richtungen, bis Sie ihn ausschalten. | an / aus |
| [[ui:ns.latency]] | Wird jedem Datagramm hinzugefügt. | 0–1000 ms |
| [[ui:ns.jitter]] | Eine zufällige Zusatzverzögerung von 0 bis zu diesem Wert für jedes Datagramm — sodass sie in anderer Reihenfolge ankommen können. | 0–500 ms |
| [[ui:ns.loss]] | Der Anteil der Datagramme, die nie ankommen, jedes für sich. | 0–100 % |
| [[ui:ns.burst]] | Die Wahrscheinlichkeit, dass ein Datagramm einen Verlust-Burst auslöst: eine Leitung, die kurz aussetzt, anders als verstreuter Verlust. | 0–20 % |
| [[ui:ns.burstLength]] | Wie viele Datagramme ein Burst im Mittel verliert. Wird angezeigt, wenn [[ui:ns.burst]] über 0 liegt; 5, wenn Sie ihn zum ersten Mal erhöhen. | 1–1000 |
| [[ui:ns.duplicate]] | Der Anteil der Datagramme, die doppelt zugestellt werden. | 0–100 % |
| [[ui:ns.corrupt]] | Der Anteil der Datagramme, in denen ein Bit eines Bytes gekippt wird. | 0–100 % |
| [[ui:ns.reorder]] | Der Anteil der Datagramme, die zurückgehalten werden — um die Latenz und mindestens 20 ms —, sodass die nachfolgenden zuerst ankommen. | 0–100 % |
| [[ui:ns.rate]] | Eine Bandbreitengrenze in Kilobits pro Sekunde; 0 ist keine. Datagramme stehen mit dieser Rate für die Leitung an; eines, das länger als eine Sekunde warten müsste, wird als gedrosselt verworfen. | 0 oder 8–10 000 000 |

Ein Burst funktioniert so: Ein Datagramm, das nicht in einem Burst ist, startet einen mit der
Wahrscheinlichkeit [[ui:ns.burst]]; jedes Datagramm in einem Burst geht verloren, und jedes
beendet ihn mit einer Chance von 1 zu [[ui:ns.burstLength]], sodass ein Burst im Mittel so
viele Datagramme dauert.

Ein Datagramm wird in dieser Reihenfolge entschieden: offline, Burst, Verlust, Bandbreite,
dann für jede Kopie Duplizierung, Verfälschung, Verzögerung und Umordnung.

### Über TCP {#profile-tcp}

| Wert | Was er tut | Bereich |
| --- | --- | --- |
| [[ui:ns.offline]] | Nichts fließt, in keine Richtung, und neue Verbindungen warten auf das Ziel, bis Sie ihn ausschalten — dann läuft alles weiter. | an / aus |
| [[ui:ns.latency]] | Wird jedem Block eines Datenstroms hinzugefügt, in beide Richtungen. | 0–1000 ms |
| [[ui:ns.jitter]] | Eine zufällige Zusatzverzögerung von 0 bis zu diesem Wert für jeden Block — nie vor dem Block davor. | 0–500 ms |
| [[ui:ns.reset]] | Der Anteil der Blöcke, die ihre Verbindung zurücksetzen, statt durchzugehen: Sowohl der Client als auch das Ziel erhalten einen Reset. | 0–100 % |
| [[ui:ns.stall]] | Der Anteil der Blöcke, die ihre Verbindung halboffen zurücklassen: Nichts geht mehr durch, in keine Richtung, und keine Seite erfährt davon. Das Relais hält beide Seiten offen und unberührt, bis es stoppt. | 0–100 % |
| [[ui:ns.rate]] | Eine Bandbreitengrenze für jeden Datenstrom jeder Verbindung in Kilobits pro Sekunde; 0 ist keine. Ab einer Sekunde Warteschlange hört das Relais auf zu lesen, sodass der Sender langsamer wird, wie auf einer langsamen Leitung. Nichts wird verworfen. | 0 oder 8–10 000 000 |

Verlust, Bursts, Duplizierung, Verfälschung und Umordnung gelten nicht für TCP: Ein echter
TCP-Datenstrom überträgt erneut, was er verliert, und ordnet sich selbst, sodass ein Client
auf einer schlechten Leitung Verzögerung, einen langsamen Sender, Resets und Verbindungen
antrifft, die aufhören zu antworten.

::: tip
Die Schieberegler enden bei 1000 ms Latenz und 500 ms Jitter. Das Relais selbst nimmt bis zu
60 000 ms von jedem an, zum Beispiel aus einer Experimentdatei.
:::

## Vorgaben {#presets}

Eine Vorgabe setzt jeden Wert mit einem Klick. Ihr Chip bleibt beleuchtet, solange die Werte
noch die der Vorgabe sind; ändern Sie einen Wert, geht er aus.

| Vorgabe | Über UDP | Über TCP |
| --- | --- | --- |
| [[ui:ns.preset.lan]] | 1 ms ±1 | 1 ms ±1 |
| [[ui:ns.preset.wifi]] | 20 ms ±30, 1 % Verlust, Bursts 1 % × 3, 0,5 % dupliziert, 2 % umgeordnet | 20 ms ±30 |
| [[ui:ns.preset.4g]] | 60 ms ±25, 0,5 % Verlust, 0,5 % umgeordnet, 20 000 kbit/s | 60 ms ±25, 20 000 kbit/s |
| [[ui:ns.preset.satellite]] | 300 ms ±30, 1 % Verlust, 2000 kbit/s | 300 ms ±30, 2000 kbit/s |
| [[ui:ns.preset.intermittent]] | 30 ms ±20, Bursts 3 % × 15 | 30 ms ±20, 0,2 % der Blöcke halboffen gelassen |
| [[ui:ns.preset.offline]] | Nichts kommt durch | Nichts fließt |

## Ändern im laufenden Betrieb {#live}

Ändern Sie einen beliebigen Wert oder wählen Sie eine andere Vorgabe, während das Relais
läuft: Sie greift eine Viertelsekunde nach Ihrer letzten Änderung, ohne den Port oder die
Verbindungen aufzugeben. Die Konsole vermerkt jedes neue Profil, und die Zeile unter
[[ui:ns.live]] sagt, was das Relais jetzt tut — den Namen der Vorgabe oder die Werte kurz,
etwa `60 ms ±25 · loss 2% · 20000 kbps`.

Ein Datagramm oder Block wird vom Profil entschieden, das gilt, wenn es eintrifft; eines, das
schon unterwegs ist, behält seine Verzögerung.

## Die Zähler {#counters}

[[ui:ns.live]] zeigt, was das Relais seit dem Start getan hat, beide Richtungen zusammen,
viermal pro Sekunde aktualisiert.

Über UDP:

| Zähler | Was |
| --- | --- |
| [[ui:ns.received]] | Datagramme, die das Relais erreicht haben. |
| [[ui:ns.forwarded]] | Weitergeleitete Datagramme; ein dupliziertes zählt zweimal. |
| [[ui:ns.dropped]] | Absichtlich verloren: offline, ein Burst oder Verlust. |
| [[ui:ns.throttled]] | Von der Bandbreitengrenze verworfen oder weil bereits 10 000 unterwegs waren. |
| [[ui:ns.duplicated]] | Datagramme, die zweimal gesendet wurden. |
| [[ui:ns.corrupted]] | Kopien mit einem gekippten Bit. |
| [[ui:ns.reordered]] | Kopien, die zurückgehalten wurden, sodass spätere sie überholten. |
| [[ui:common.volume]] | Weitergeleitete Bytes. |

Über TCP:

| Zähler | Was |
| --- | --- |
| [[ui:ns.connections]] | Verbindungen, die Clients zum Relais aufbauten. |
| [[ui:ns.received]] | Blöcke, die das Relais gelesen hat, in beide Richtungen. |
| [[ui:ns.forwarded]] | Blöcke, die es weiterschrieb. |
| [[ui:ns.resets]] | Zurückgesetzte Verbindungen. |
| [[ui:ns.stalled]] | Halboffen gelassene Verbindungen. |
| [[ui:ns.held]] | Wie oft ein Datenstrom auf die Bandbreitengrenze wartete: Der Sender wurde gebremst, nichts wurde verworfen. |
| [[ui:common.volume]] | Weitergeschriebene Bytes. |

Mit eingeschaltetem Mitschnitt zeigt der [Inspektor](inspector.md) weitergeleitete Datagramme
und Blöcke — höchstens eines alle 25 ms, beide Richtungen zusammen —, jedes mit seinem
Schicksal: `forwarded +43ms`, `· corrupted`, `· reordered`, `· copy 2/2`, `dropped (loss)`,
`dropped (burst)`, `dropped (offline)`, `throttled` und über TCP `reset` und `half-open`. →
ist Client zum Ziel, ← Ziel zum Client.

Ein weitergeleiteter Frame nennt echte Sockets: [[ui:ins.local]] ist die Adresse, auf der das
Relais empfängt, und [[ui:bc.peer]] ist, wohin dieses Datagramm oder dieser Block ging — das
Ziel oder der Client, an den eine Antwort zurückging. Die Richtung beendet den Befund:
`forwarded +43ms · client→target`, `dropped (loss) · target→client`. So lässt sich ein
weitergeleitetes Datagramm mit [[ui:sig.fromFrame]] aufbewahren und wird auf die Adresse
gerichtet, an die es ging; ein TCP-Block ist ein Stück eines Datenstroms und kann das nicht
(siehe [Inspektor](inspector.md#save-as-signal)).

## Derselbe Verkehr, dasselbe Schicksal {#seed}

Jede Entscheidung — welches Datagramm verloren geht, um wie viel verzögert, wo verfälscht
wird — wird aus dem Startwert des Relais gezogen, getrennt für jede Richtung und, über TCP,
für jede Verbindung. Mit demselben Startwert und demselben Verkehr verwirft das Relais
dieselben Datagramme und verzögert sie gleich.

Die Ansicht [[ui:nav.netsim]] nimmt jedes Mal einen neuen Startwert, wenn Sie ein Relais
starten. Um einen Durchlauf exakt zu wiederholen, verwenden Sie einen Knoten
[[ui:exp.node.impairment]] in einem Experiment: Er zieht aus dem Startwert des Durchlaufs,
den Sie fixieren können (siehe [Einen gestörten Durchlauf
wiederholen](../experiments/faults.md#seed)).

## In Experimenten {#experiments}

Zwei Knoten bringen dasselbe Relais in ein Experiment:

- [[ui:exp.node.impairment]] öffnet ein Relais vor dem ersten Schritt und schließt es, wenn
  der Durchlauf endet, wie auch immer er endet. Seine Empfangs- und Zieladressen nehmen nur
  Parameter an; das Ziel darf ein Hostname sein, der beim Start des Durchlaufs nachgeschlagen
  wird. Siehe [Knoten](../experiments/nodes.md#node-impairment).
- [[ui:exp.node.impairment_change]] schaltet ein Relais des Durchlaufs ab diesem Schritt auf
  ein anderes Profil um — sauber, verlustbehaftet, offline, wieder sauber —, und der Bericht
  des Durchlaufs zählt, was jede Phase tat. Siehe
  [Knoten](../experiments/nodes.md#node-impairment_change).

An einem OSC- oder UDP-Knoten setzt [[ui:exp.routeThrough]] ein
[[ui:exp.node.impairment]] davor und richtet den Knoten auf das Relais. Siehe
[Störungen](../experiments/faults.md).

## Verwandte Seiten {#related}

- [Emulatoren](emulators.md) — das Ziel, das hinter das Relais gehört.
- [Inspektor](inspector.md) — das Schicksal jedes Datagramms.
- [UDP und TCP](../protocols/udp-tcp.md)

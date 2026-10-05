---
title: OSC
description: Typisierte Open-Sound-Control-Nachrichten senden, beobachten, was auf einem Port eintrifft, und aus der OSC-Ansicht eine fortlaufende Wellenform in ein Gerät geben.
---

# OSC

Die Ansicht [[ui:nav.osc]] ist der Ort, an dem Sie Open Sound Control (OSC 1.0)
von Hand über UDP sprechen. Sie hat drei Teile:

- [[ui:osc.sender]]: eine Nachricht mit typisierten Argumenten, gesendet mit
  <kbd>Enter</kbd>.
- [[ui:osc.monitor]]: empfängt auf einem Port und dekodiert jedes Paket, das
  eintrifft.
- [[ui:osc.generator]]: sendet einen Wert, der einer Wellenform folgt, viele
  Male pro Sekunde, und zeichnet ihn.

Signal Lab kodiert und dekodiert OSC selbst. Was es sendet, ist genau das, was
der [Inspektor](../tools/inspector.md) zeigt, Byte für Byte.

## Eine Nachricht senden {#send}

1. Öffnen Sie [[ui:nav.osc]].
2. Geben Sie unter [[ui:common.target]] die IP-Adresse oder den Hostnamen des
   Geräts und seinen Port ein, zum Beispiel `127.0.0.1:9000` oder
   `stage-mixer.local:9000`.
3. Geben Sie unter [[ui:common.address]] die Adresse ein, auf die das Gerät
   hört, zum Beispiel `/mixer/fader/1`.
4. Setzen Sie unter [[ui:common.arguments]] Typ und Wert jedes Arguments.
   Drücken Sie [[ui:common.addArgument]] für ein weiteres; ✕ entfernt eines.
5. Drücken Sie [[ui:common.send]], oder <kbd>Enter</kbd> in einem beliebigen
   Feld des Senders.

Die Zeile unter den Schaltflächen sagt, was hinausging: die Adresse, ihre Größe
in Bytes und das Ziel. Wird dieselbe Nachricht erneut gesendet, zählt sie hoch
(×2, ×3…), sodass Sie sehen, dass ein wiederholtes Senden etwas bewirkt hat. Ein
Fehler wird stattdessen dort gezeigt, und in der Konsole.

Ziel, Adresse und Argumente bleiben erhalten, wenn Sie die Ansicht wechseln und
wenn Sie die App neu starten.

### Felder {#send-fields}

| Feld | Was | Standard |
| --- | --- | --- |
| [[ui:common.target]] | `IP:port` oder `host:port` des Empfängers. Eine IPv6-Adresse steht in eckigen Klammern: `[::1]:9000`. Ein Hostname wird bei jedem Senden nachgeschlagen; hat er eine IPv4-Adresse, wird diese verwendet (so erreicht `localhost` einen Empfänger, der auf `127.0.0.1` empfängt), sonst seine IPv6-Adresse. Ein Ziel ohne Port wird abgelehnt. | `127.0.0.1:9000` |
| [[ui:common.address]] | Die OSC-Adresse, beginnend mit `/`, Teile durch `/` getrennt. Eine ohne führendes `/` wird abgelehnt, bevor etwas gesendet wird. | `/hello/avatar/1` |
| [[ui:common.arguments]] | Typisierte Werte nach der Adresse, in ihrer Reihenfolge. Eine Nachricht darf keine haben. | ein `float`, `1.0` |

### Argumenttypen {#types}

Der Typ jedes Arguments ist Teil der Nachricht (sein Typ-Tag), daher kann ein
Gerät, das einen float erwartet, ein int mit demselben Wert ignorieren.

| Typ in der Liste | OSC-Tag | Wert | Wie Sie ihn eingeben |
| --- | --- | --- | --- |
| `int` | `i` | 32-Bit-Ganzzahl mit Vorzeichen | eine ganze Zahl |
| `float` | `f` | 32-Bit-Gleitkommazahl | eine Zahl, `0.75` |
| `str` | `s` | Text | beliebiger Text, als UTF-8 gesendet |
| `bool` | `T` / `F` | wahr oder falsch | `true` oder `false` aus einer Liste; trägt keine Bytes, nur das Tag |
| `long` | `h` | 64-Bit-Ganzzahl mit Vorzeichen | eine ganze Zahl |
| `double` | `d` | 64-Bit-Gleitkommazahl | eine Zahl |
| `nil` | `N` | nichts | kein Wert |
| `blob` | `b` | Bytes | hier nicht typisiert: Es erscheint, schreibgeschützt und in Hex, wenn Sie ein Signal öffnen, das eines enthält |

Ein Zahlenfeld, das keine Zahl enthält, sendet `0`.

::: tip
Ein OSC-`true` ist das Tag `T`, nicht der Text `"true"`. Ein Gerät, das auf
einen bool wartet, ignoriert einen String stillschweigend.
:::

## Bundles {#bundles}

Der Sender sendet einzelne Nachrichten, keine Bundles. Trifft ein Bundle
(`#bundle`) ein, packen es der Monitor, die Warteknoten des Experiments und die
Emulatoren aus: Jede Nachricht darin wird für sich behandelt, und ihr Zeit-Tag
wird ignoriert.

## Einen Port überwachen {#monitor}

Um zu sehen, was ein Gerät oder ein Show-Controller sendet:

1. Geben Sie unter [[ui:common.bind]] die Adresse und den Port zum Empfangen
   ein. `0.0.0.0:9000` (die Voreinstellung) empfängt auf jeder Netzwerkkarte;
   `127.0.0.1:9000` nur auf diesem Computer.
2. Drücken Sie [[ui:osc.listen]]. Das Feld ist gesperrt, solange der Monitor
   läuft.
3. Richten Sie den Sender auf die IP-Adresse dieses Computers und diesen Port.

Jedes Paket wird zu einer Zeile, das Neueste oben:

| Spalte | Was |
| --- | --- |
| [[ui:common.time]] | Wann es eingetroffen ist, auf die Millisekunde |
| [[ui:osc.from]] | `IP:port` des Absenders |
| [[ui:osc.address]] | Die OSC-Adresse, oder [[ui:osc.decodeError]], wenn das Paket kein gültiges OSC ist |
| [[ui:osc.args]] | Die Argumentwerte; ein Blob erscheint als `blob[n]`, `nil` als `nil`. Bei einem Paket, das nicht dekodiert werden konnte, der Grund. |

Ein Bundle ergibt eine Zeile pro Nachricht. Die Liste behält die neuesten 300
Zeilen; [[ui:common.clear]] leert sie. Drücken Sie [[ui:common.stop]], um den
Port zu schließen. Der Monitor ist außerdem ein Job in der Leiste der Konsole,
sodass er auch von dort gestoppt werden kann.

Der Monitor liest Pakete bis 64 KiB. Er dekodiert die Tags
`i f s S b h d T F N I` (`S` liest sich als Text, `I` als nil); ein Paket mit
einem anderen Tag oder ein abgeschnittenes wird als Dekodierfehler gezeigt statt
verworfen.

### Eine Nachricht in einen Warteknoten verwandeln {#wait-for-this}

Jede Zeile hat eine Schaltfläche ⇠, [[ui:osc.waitForThis]]. Sie fügt dem
geöffneten Experiment einen
[Auf OSC warten](../experiments/nodes.md#node-wait_osc)-Schritt hinzu, der auf
dem [[ui:common.bind]] des Monitors auf diese Adresse wartet, mit einer Regel
„ist gleich" für jedes Text-, Ganzzahl- und Wahr/Falsch-Argument — Floats, Blobs
und nil bekommen keine — bis zu 16 Regeln. Sein Timeout beträgt 2000 ms. Der
Editor öffnet sich mit dem neuen Schritt ausgewählt.

::: warning
Stoppen Sie den Monitor, bevor Sie dieses Experiment ausführen. Der Durchlauf
öffnet denselben Port selbst, und zwei Empfänger können ihn sich nicht teilen.
:::

## Eine Wellenform ausgeben {#generator}

Der [[ui:osc.generator]] sendet eine Nachricht nach der anderen an eine Adresse,
mit einem einzigen Argument, dessen Wert einer Wellenform folgt — ein Fader, ein
Lichtpegel, eine Position. Nutzen Sie ihn, um zu sehen, wie ein Gerät einem
bewegten Wert folgt, oder um einen Empfänger mit einem gleichmäßigen Strom zu
belasten.

1. Setzen Sie [[ui:common.target]] und [[ui:osc.address]]. Beide werden beim
   Drücken von [[ui:osc.startGen]] geprüft, wie der Sender sie prüft.
2. Wählen Sie eine [[ui:osc.waveform]], ihre [[ui:osc.freq]] und die
   [[ui:osc.rate]].
3. Setzen Sie [[ui:osc.min]] und [[ui:osc.max]], den Bereich des Werts.
4. Drücken Sie [[ui:osc.startGen]]. Er läuft, bis Sie [[ui:osc.stopGen]] drücken
   oder seinen Job in der Leiste der Konsole stoppen.

| Feld | Was | Standard |
| --- | --- | --- |
| [[ui:common.target]] | `IP:port` oder `host:port` des Empfängers; ein Name wird einmal nachgeschlagen, beim Start des Generators | `127.0.0.1:9000` |
| [[ui:osc.address]] | Die Adresse, an die jede Nachricht gesendet wird; sie beginnt mit `/` | `/hello/lfo` |
| [[ui:osc.waveform]] | Die Form des Werts über die Zeit (unten) | [[ui:wave.sine]] |
| [[ui:osc.freq]] | Zyklen der Wellenform pro Sekunde | `1` |
| [[ui:osc.rate]] | Nachrichten pro Sekunde, von 0,1 bis 5000; ein Wert außerhalb wird auf diesen Bereich begrenzt | `60` |
| [[ui:osc.min]], [[ui:osc.max]] | Der niedrigste und der höchste Wert. Liegt Max unter Min, bewegt sich der Wert nicht. | `0`, `1` |
| [[ui:osc.asInt]] | Auf die nächste ganze Zahl runden und ein `int` statt eines `float` senden | aus |

| Wellenform | Was der Wert in jedem Zyklus tut |
| --- | --- |
| [[ui:wave.sine]] | Schwingt weich zwischen Min und Max, beginnt in der Mitte und steigt |
| [[ui:wave.triangle]] | Steigt von Min auf Max, fällt dann zurück auf Min, beginnt bei Min |
| [[ui:wave.saw]] | Ein fallendes Sägezahn: beginnt bei Max, fällt auf Min, springt dann zurück auf Max |
| [[ui:wave.ramp]] | Ein steigendes Sägezahn: beginnt bei Min, steigt auf Max, springt dann zurück auf Min |
| [[ui:wave.square]] | Max in der ersten Hälfte, Min in der zweiten |
| [[ui:wave.random]] | Ein neuer Zufallswert zwischen Min und Max mit jeder Nachricht; die Frequenz wird nicht verwendet |
| [[ui:wave.constant]] | Jedes Mal Max; die Frequenz wird nicht verwendet |

### Der Scope {#scope}

Neben den Feldern zeichnet der Scope den Wert, wie er gesendet wird: die
neuesten 300 Punkte, so skaliert, dass sie hineinpassen. Darunter stehen die
Wellenform, die Frequenz und die Rate sowie der zuletzt gesendete Wert. Der
Scope wird etwa 30-mal pro Sekunde aktualisiert, so schnell der Generator auch
sendet; bei hohen Raten zeigt er daher eine Stichprobe der Nachrichten, nicht
jede einzelne.

Scheitert ein Senden, stoppt der Generator, und die Konsole sagt, warum.

## Im Inspektor {#inspector}

Läuft der Mitschnitt ([[ui:ins.arm]] in der [[ui:dock.inspector]]), erscheint
OSC-Verkehr mit dem Protokoll `osc`:

| Quelle | Was | Hinweise |
| --- | --- | --- |
| `osc-send` | Jede Nachricht, die der Sender, ein Bibliothekssignal, ein Schritt [[ui:exp.node.osc]] oder `signallab send osc` sendet | Ein Schritt, der auf eine Antwort wartet, erscheint als `experiment` |
| `osc-monitor` | Jedes Paket, das der Monitor empfängt | Ein Bundle wird durch seine erste Nachricht und `+n more in bundle` zusammengefasst; ein fehlerhaftes Paket hat das Urteil `decode error: …` |
| `osc-gen` | Die Nachrichten des Generators | Höchstens eine alle 100 ms wird mitgeschnitten, mit dem Urteil `sampled`; die nächste nach übersprungenen Nachrichten ergänzt, wie viele nicht gezeichnet wurden, als `sampled · +5 not shown` |

Jeder Frame behält die Bytes, aus denen er gebaut wurde. Siehe
[Inspektor](../tools/inspector.md).

## Speichern und wiederverwenden {#library}

- **Als Signal speichern.** [[ui:sig.saveNew]] unter den Schaltflächen behält die
  Nachricht — Ziel, Adresse und Argumente — in der Signalbibliothek, in einem
  Ordner Ihrer Wahl. Von da an ist der Sender an dieses Signal gebunden:
  [[ui:sig.save]] (oder <kbd>Ctrl</kbd>+<kbd>S</kbd> im Sender) aktualisiert es,
  [[ui:sig.saveAs]] legt eine Kopie an, und der Chip daneben öffnet es in
  [[ui:nav.signals]]. Senden Sie es später aus [[ui:nav.signals]] oder mit
  <kbd>Ctrl</kbd>+<kbd>K</kbd> von jeder Ansicht aus. Siehe
  [Signale](../tools/signals.md).
- **Zu einem Experiment hinzufügen.** [[ui:common.toExperiment]] fügt dem
  geöffneten Experiment einen Schritt
  [OSC-Nachricht](../experiments/nodes.md#node-osc) mit demselben Ziel, derselben
  Adresse und denselben Argumenten hinzu — direkt vor Ende oder nach dem
  ausgewählten Schritt — und öffnet es. Während das Experiment läuft, kann
  nichts hinzugefügt werden; die Konsole sagt es.

## In Experimenten {#experiments}

| Schritt | Was er tut |
| --- | --- |
| [[ui:exp.node.osc]] | Sendet eine Nachricht. Sein Ziel, seine Adresse und seine Textargumente nehmen `{{templates}}`. Mit [[ui:exp.expectReply]] sendet er von einem eigenen Port und wartet dort im selben Schritt auf die Antwort. [Details](../experiments/nodes.md#node-osc) |
| [[ui:exp.node.wait_osc]] | Wartet auf eine Nachricht, deren Adresse auf ein Muster passt und deren Argumente die Regeln erfüllen. [Details](../experiments/nodes.md#node-wait_osc) |
| [[ui:exp.node.emulator]] | Ein OSC-Gerät, das für den ganzen Durchlauf nach Regeln antwortet. Siehe [Emulatoren](../tools/emulators.md). |

Der OSC-Schritt nimmt `IP:port` oder `host:port` wie die Ansicht, und seine
Adresse muss mit `/` beginnen. Ein Hostname wird bei jedem Senden des Schritts
nachgeschlagen.

### Adressmuster {#patterns}

[[ui:exp.node.wait_osc]], die Antwort von [[ui:exp.node.osc]] und die Regeln
eines OSC-Emulators gleichen Adressen mit Mustern nach OSC 1.0 ab:

| Muster | Passt auf |
| --- | --- |
| `*` | eine beliebige Folge von Zeichen, auch keine |
| `?` | genau ein Zeichen |
| `[0-9]`, `[a-c]` | ein Zeichen aus der Menge oder dem Bereich |
| `[!0-9]` | ein Zeichen, das nicht in der Menge ist |
| `{ping,pong}` | eines der Wörter |

Platzhalter bleiben innerhalb eines Teils zwischen Schrägstrichen: `/cue/*`
passt auf `/cue/7`, aber nicht auf `/cue/7/go`, und ein Muster passt nur auf
eine Adresse mit derselben Anzahl von Teilen. Die Übereinstimmung ist groß- und
kleinschreibungssensitiv. Ein Muster beginnt mit `/`, hat keinen leeren Teil
(`//`), keine Leerzeichen, kein `#` und keine Zeichen außerhalb von ASCII, und
ist höchstens 512 Zeichen lang.

Argumentregeln vergleichen Argument Nummer 0–63 mit einem Wert (ist gleich,
kleiner als, enthält, passt auf einen regulären Ausdruck und so weiter); ein
Warteknoten hat höchstens 16. Trifft ein Bundle ein, nimmt der Warteknoten es
an, wenn irgendeine Nachricht darin passt. Was eine passende Nachricht den
Schritten danach gibt, steht unter
[Daten und Vorlagen](../experiments/data.md).

## Von der Kommandozeile {#cli}

`signallab send osc` sendet eine Nachricht, wie es der Sender tut:

```bash
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main
```

```text
✔ sent /cue/go (24 bytes) → 127.0.0.1:9000
```

Jedes Argument ist `tag:value`: `i:3`, `f:0.5`, `d:1.5`, `h:64`, `s:text`,
`b:de ad be ef` (Hex-Bytes) oder `T`, `F`, `N` für sich allein. Ohne Tag ist eine
ganze Zahl `i`, eine Zahl mit Dezimalpunkt `f` und alles andere `s`; schreiben
Sie `s:7`, um den Text `7` zu senden. Das Ziel ist `IP:port` oder `host:port`.
Es endet mit 0, wenn die Nachricht hinausging, mit 1, wenn das Senden fehlschlug
(ein Hostname, der sich nicht auflösen lässt, eingeschlossen), und mit 2, wenn
ein Argument, die Adresse oder das Ziel ungültig ist.
[`signallab fire`](../automation/cli.md#cli-fire) sendet ein gespeichertes
Signal. Siehe [Kommandozeile](../automation/cli.md#cli-send-osc).

::: tip
In Git Bash unter Windows wird ein Argument, das mit `/` beginnt, in einen
Dateipfad umgewandelt, bevor `signallab` es sieht. Führen Sie den Befehl mit
`MSYS_NO_PATHCONV=1` davor aus, oder verwenden Sie PowerShell oder `cmd`.
:::

## Probleme {#troubleshooting}

| Was Sie sehen | Übliche Ursache |
| --- | --- |
| `… is not a valid address` beim Senden | Das Ziel hat keinen Port oder ist weder `IP:port` noch `host:port`. |
| `Cannot resolve …` beim Senden | Der Hostname lässt sich auf diesem Computer nicht auflösen. Prüfen Sie ihn, oder verwenden Sie die IP-Adresse. |
| `OSC addresses start with / (…)` | Der Adresse fehlt das führende `/`. |
| Die Nachricht wird gesendet, aber das Gerät tut nichts | Falscher Port oder falsche Adresse; ein anderer Typ, als es erwartet (`int` statt `float`, ein Text `"true"` statt eines bool). Beobachten Sie es im Inspektor, oder richten Sie das Ziel auf den Monitor auf diesem Computer, um zu sehen, was hinausgeht. |
| `… is already in use by another program` bei [[ui:osc.listen]] | Ein anderes Programm — oder ein laufendes Experiment, ein Emulator oder ein zweiter Monitor — hat den Port. |
| `… is not an address of this computer` | Die IP unter [[ui:common.bind]] gehört zu einem anderen Rechner. Verwenden Sie `0.0.0.0` oder eine der Adressen dieses Computers. |
| Pakete von anderen Rechnern kommen nie an | Unter Windows kann die Firewall sie abhalten: Lassen Sie Signal Lab zu, wenn die App es anbietet. Lokaler Verkehr (`127.0.0.1`) ist nicht betroffen. Siehe [Problemlösung](../reference/troubleshooting.md). |
| Zeilen mit [[ui:osc.decodeError]] | Der Sender spricht auf diesem Port kein OSC 1.0 oder verwendet ein Typ-Tag, das Signal Lab nicht dekodiert. |

Auf einem Server arbeitet die Ansicht im Netzwerk des Servers: `127.0.0.1` ist
der Server selbst, und der Monitor empfängt auf den Ports des Servers. Siehe
[Server](../server/index.md).

Jede Fehlermeldung steht unter
[Fehlermeldungen](../reference/errors.md#transport).

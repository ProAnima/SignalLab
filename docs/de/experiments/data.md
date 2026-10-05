---
title: Daten und Vorlagen
description: Parameter und Profile, die Vorlagensprache mit ihren Generatoren, aus Antworten gewonnene Werte, Vergleiche und Geheimnisse, die die Engine nie verlassen.
---

# Daten in Experimenten

Werte wandern durch einen Durchlauf: Ein Parameter wählt das Ziel, ein Feld einer
Antwort wird zu einem Header der nächsten Anfrage, eine erzeugte ID geht in einem
Befehl hinaus und kommt in einer Prüfung zurück. Diese Seite behandelt, woher
diese Werte kommen und wie ein Feld sie verwendet.

| Quelle | Geschrieben als | Festgelegt wo |
| --- | --- | --- |
| Parameter | `{{api}}` oder `{{params.api}}` | Bereich [[ui:exp.params]], ein Profil, [[ui:exp.runWith]] |
| Variable | `{{token}}` oder `{{vars.token}}` | ein Knoten während des Durchlaufs: [[ui:exp.node.extract]], ein Warteknoten, ein Senden, das auf eine Antwort wartet |
| Geheimnis | `{{secret.API_TOKEN}}` | der Anmeldeinformationsspeicher des Computers oder Umgebung und Dateien des Servers |
| Eingebauter Wert | `{{run.seed}}`, `{{now.iso}}`, `{{counter}}` | der Durchlauf selbst |
| Generator | `{{uuid}}`, `{{random_int(1, 100)}}` | aus dem Startwert des Durchlaufs gezogen |

## Parameter {#parameters}

Ein Parameter ist ein benannter Textwert, den jedes Feld mit Vorlagen verwenden
kann. Halten Sie Ziele in Parametern, sodass eine Adressänderung eine einzige
Änderung ist statt einer pro Knoten.

### Einen Parameter hinzufügen {#add-parameter}

1. Drücken Sie [[ui:exp.params]] (`{ }`) in der Werkzeugleiste des Editors.
2. Drücken Sie auf der Registerkarte [[ui:exp.noProfile]] auf [[ui:exp.addParam]].
3. Tippen Sie den [[ui:exp.paramName]] und den [[ui:exp.paramValue]], zum Beispiel
   `api` und `http://127.0.0.1:8080`.
4. Schreiben Sie in ein Feld eines Knotens `{{api}}/login`.

Jede Änderung im Bereich ist eine Bearbeitung des Experiments: Sie wird mit ihm
gespeichert und mit <kbd>Ctrl</kbd>+<kbd>Z</kbd> rückgängig gemacht wie jede andere.

### Regeln {#parameter-rules}

| Regel | Grenze |
| --- | --- |
| Name | beginnt mit einem Buchstaben oder `_`, danach Buchstaben, Ziffern und `_` |
| Reservierte Namen | `vars`, `params`, `secret`, `run`, `node`, `now`, `uuid`, `counter`, `random_int`, `random_float`, `pick` |
| Parameter pro Experiment | 64 |
| Größe eines Werts | 64 KiB |
| Namen | eindeutig; eine Variable darf nicht den Namen eines Parameters haben |

Ein Wert ist reiner Text und wird so eingefügt, wie er geschrieben ist: `{{…}}`
in einem Wert wird nicht aufgelöst. Verlangt ein Feld einen Teil eines Parameters
(`{{config.ports[0]}}`), wird der Wert als JSON gelesen; ein Wert, der kein JSON
ist, hat keine Teile.

Ein Parameter, dessen Name ungültig, reserviert oder doppelt vergeben ist,
hindert das Experiment nicht am Speichern, sodass Sie weiter tippen können; das
Experiment läuft erst, wenn der Name korrigiert ist.

## Profile {#profiles}

Ein Profil ist eine benannte Menge von Parameterwerten — *Laptop*, *Bühne*,
*Veranstaltungsort* —, sodass das Wechseln des Ziels eine Wahl ist, keine
Bearbeitung jedes Knotens. Ein Profil ändert einige Parameter; die anderen
behalten ihren Standard.

### Ein Profil anlegen {#make-profile}

1. Öffnen Sie [[ui:exp.params]] und drücken Sie [[ui:exp.addProfile]]. Eine neue Registerkarte öffnet sich.
2. Benennen Sie es in [[ui:exp.profileName]] um.
3. Tippen Sie für jeden Parameter, den das Profil ändert, seinen Wert. Ein leeres
   Feld behält den Standard, im Feld grau angezeigt; [[ui:exp.resetToDefault]] (↺)
   löscht einen Wert.
4. Drücken Sie [[ui:exp.makeActive]], um damit auszuführen. Die Registerkarte des
   aktiven Profils trägt ● [[ui:exp.activeProfile]]. [[ui:exp.makeActive]] auf der
   Registerkarte [[ui:exp.noProfile]] kehrt zu den Standardwerten zurück.

Sobald ein Experiment Profile hat, wechselt eine Liste [[ui:exp.profile]] in der
Werkzeugleiste zwischen ihnen. Das aktive Profil wird von Durchläufen, der
Vorschau und [[ui:exp.sendNow]] verwendet, und es wird im Experiment gespeichert,
sodass eine exportierte Datei mit denselben Zielen öffnet.
[[ui:exp.removeProfile]] löscht das angezeigte Profil.

| Regel | Grenze |
| --- | --- |
| Profile pro Experiment | 32 |
| Name | 1–64 Zeichen, eindeutig (Leerzeichen an den Enden zählen nicht) |
| Werte | nur Parameter, die es gibt; höchstens 64 |

Das Umbenennen oder Entfernen eines Parameters ändert ihn in jedem Profil auf
einmal.

### Welchen Wert ein Durchlauf verwendet {#precedence}

Später gewinnt:

1. der Standard des Parameters, auf der Registerkarte [[ui:exp.noProfile]];
2. der Wert des aktiven Profils, wenn es einen setzt;
3. ein in [[ui:exp.runWith]] nur für diesen Durchlauf getippter Wert — siehe
   [mit anderen Werten ausführen](runs.md#run-with).

[[ui:exp.runWith]] darf nur Parameter setzen, die das Experiment hat. Der Bericht
des Durchlaufs hält das Profil, die für den Durchlauf getippten Werte und jeden
verwendeten Wert fest.

### Profile, die nicht laufen würden {#profile-issues}

Jedes Mal, wenn das Experiment geprüft wird, werden auch die anderen Profile und
die Standardwerte geprüft. Eines, das fehlschlagen würde — etwa eine URL, die
nicht `http://` oder `https://` ist —, trägt ⚠ in den Registerkarten und in der
Liste der Werkzeugleiste, und sein Tooltip nennt den Grund. Es hält Durchläufe
mit dem verwendeten Profil nicht auf.

## Vorlagen {#templates}

Text in `{{ }}` ist ein Ausdruck; alles andere in einem Feld bleibt genau so
erhalten, wie es geschrieben ist.

```text
{{api}}/users/{{user.id}}?trace={{uuid}}
Bearer {{secret.API_TOKEN}}
```

- Leerzeichen in den Klammern spielen keine Rolle: `{{ token }}` ist `{{token}}`.
- `\{{` schreibt ein wörtliches `{{`.
- Ein `}}` allein ist reiner Text.
- Ein Wert wird so eingefügt, wie er ist, ohne Anführungszeichen. Schreiben Sie in
  einem JSON-Body die Anführungszeichen selbst: `"id": "{{uuid}}"`.

### Namen {#names}

| Ausdruck | Wert |
| --- | --- |
| `{{name}}` | die Variable `name`, wenn auf diesem Pfad eine gesetzt ist, sonst der Parameter `name` |
| `{{vars.name}}` | nur die Variable |
| `{{params.name}}` | nur der Parameter |
| `{{secret.NAME}}` | das gespeicherte Geheimnis `NAME` — siehe [Geheimnisse](#secrets) |
| `{{name.field}}` | ein Feld eines JSON-Werts |
| `{{name[0]}}` | ein Element eines JSON-Arrays |
| `{{name["a b"]}}`, `{{name['a b']}}` | ein Feld, dessen Name andere Zeichen enthält |

Ein Feldname nach `.` darf Buchstaben, Ziffern, `_` und `-` enthalten. Schritte
verketten sich: `{{reply.args[0]}}`, `{{order.items[2].sku}}`.

### Wie Werte geschrieben werden {#value-text}

| Wert | Geschrieben als |
| --- | --- |
| Text | der Text |
| Zahl | ihre kürzeste Form: `42`, `0.5` |
| `true`, `false` | `true`, `false` |
| `null` | `null` |
| Objekt, Array | kompaktes JSON: `["x","y"]` |

### Eingebaute Werte {#built-ins}

| Ausdruck | Wert |
| --- | --- |
| `{{run.id}}` | die Jobnummer des Durchlaufs; `0` in der Vorschau und in [[ui:exp.sendNow]] |
| `{{run.seed}}` | der Startwert dieses Durchlaufs |
| `{{node.id}}` | die id des ausgeführten Knotens |
| `{{now}}` | die aktuelle Zeit, Unix-Millisekunden |
| `{{now.iso}}` | die aktuelle Zeit in UTC, ISO 8601 mit Millisekunden: `2026-09-30T12:34:56.789Z` |
| `{{counter}}` | wie oft dieser Knoten in diesem Durchlauf gelaufen ist, dieses Mal eingeschlossen, ab 1 |

`{{counter}}` zählt pro Knoten: im Rumpf einer [Schleife](flow.md#loop) ist es die
Nummer der Iteration, in einem Knoten, der sich [wiederholt](flow.md#repeat), die
Nummer des Sendens. `run`, `node` und `now` haben nur die aufgelisteten Felder;
alles andere ist ein Fehler.

### Generatoren {#generators}

| Ausdruck | Wert |
| --- | --- |
| `{{uuid}}` oder `{{uuid()}}` | eine UUID der Version 4 |
| `{{random_int(min, max)}}` | eine ganze Zahl von `min` bis `max`, beide eingeschlossen; ganzzahlige Argumente, `min` ≤ `max` |
| `{{random_float(min, max)}}` | eine Zahl von `min` bis ausschließlich `max`, mit 3 Dezimalstellen; `min` < `max` |
| `{{random_float(min, max, digits)}}` | dasselbe mit `digits` Dezimalstellen, 0–9 |
| `{{pick(a, b, c)}}` | eines der Argumente, mindestens eines |

Argumente werden durch Kommas getrennt. Eines in Anführungszeichen (`"dark blue"`
oder `'a, b'`) darf alles außer seinem eigenen Anführungszeichen enthalten; eines
ohne Anführungszeichen darf Buchstaben, Ziffern und `_ - . : / +` enthalten. Ein
leeres Argument ist ein Fehler.

Jeder Generator zieht aus dem Startwert des Durchlaufs. Die Werte einer Ausführung
eines Knotens hängen nur vom Startwert, der id des Knotens und davon ab, wie oft
der Knoten gelaufen ist, sodass parallele Zweige nie die Werte des jeweils anderen
ändern, und ein Durchlauf mit demselben Startwert erzeugt dieselben Werte erneut.
Ziehungen innerhalb eines Knotens folgen der Reihenfolge seiner Felder. `{{now}}`
und `{{run.id}}` sind nicht reproduzierbar. Siehe [Startwerte](runs.md#seeds).

### Vorschläge {#suggestions}

Das Tippen von `{{` in einem Feld mit Vorlagen oder <kbd>Ctrl</kbd>+<kbd>Space</kbd>
öffnet eine Liste in vier Gruppen: [[ui:exp.suggest.params]] mit ihren Werten,
[[ui:exp.suggest.vars]], die vor diesem Knoten gesetzt wurden, mit dem Knoten, der
sie setzt (auch Felder einer Antwort, etwa `reply.args[0]`),
[[ui:exp.suggest.secrets]] und [[ui:exp.suggest.generators]]. <kbd>↑</kbd> und
<kbd>↓</kbd> wählen, <kbd>Enter</kbd> oder <kbd>Tab</kbd> fügt ein, <kbd>Esc</kbd>
schließt die Liste und behält das Feld.

### Unbekannte Namen sind Fehler {#unknown-names}

Ein Name ohne Wert wird nie zu einer leeren Zeichenkette. Vor einem Durchlauf muss
jeder Name, den ein Feld verwendet, ein Parameter, ein gültiger Geheimnisname oder
eine Variable sein, die auf **jedem** Pfad gesetzt ist, der zu dem Knoten führt.
Der Editor zeigt auf den Knoten und das Feld:

| Problem | Vor dem Durchlauf | Während des Durchlaufs |
| --- | --- | --- |
| Ein Name, den niemand setzt | `name.unknown` | — |
| Eine Variable, die nur auf einigen Pfaden gesetzt ist | `name.not_on_every_path` | — |
| `{{params.x}}` ohne einen Parameter `x` | `param.unknown` | — |
| Ein Feld, das ein Wert nicht hat | — | `template.no_field` |
| Nicht geschlossenes `{{`, ein leeres `{{}}`, ein fehlerhaftes Argument | `template.*`, mit der Position | — |

Die Texte dieser Codes stehen unter [Fehler](../reference/errors.md).

## Welche Felder Vorlagen annehmen {#templated-fields}

| Knoten | Felder mit Vorlagen |
| --- | --- |
| [[ui:exp.node.http]] | URL, Header-Namen und -Werte, Body, der Benutzername und das Passwort von Basic und Digest, der Bearer-Token |
| [[ui:exp.node.osc]] | Ziel, Adresse, Textargumente; mit einer Antwort: ihr Adressmuster und die Regelwerte |
| [[ui:exp.node.udp]] | Ziel, Nutzdaten; mit einer Antwort: ihr Muster |
| [[ui:exp.node.tcp]] | Host, Nutzdaten |
| [[ui:exp.node.mqtt]] | Broker-Host, Topic, Nutzdaten |
| [[ui:exp.node.log]] | Nachricht |
| [[ui:exp.node.assert_body]] | erwarteter Text |
| [[ui:exp.node.assert_header]] | Header-Name, erwarteter Text |
| [[ui:exp.node.assert_value]], [[ui:exp.node.branch_value]], die Abbruchbedingung einer [[ui:exp.node.loop]] | Wert, erwarteter Wert |
| [[ui:exp.node.wait_osc]] | Adressmuster, Regelwerte |
| [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_ws]] | Muster |
| [[ui:exp.node.wait_mqtt]] | Broker und Topic (nur Parameter), Muster |
| [[ui:exp.node.wait_http]] | Pfadmuster, Bedingungen |
| [[ui:exp.node.impairment]] | Empfangen und Ziel (nur Parameter) |
| [[ui:exp.node.ws_connect]] | URL, Header-Namen und -Werte |
| [[ui:exp.node.ws_send]] | Nutzdaten |
| [[ui:exp.node.ws_close]] | Grund |

Zahlen — Ports, Timeouts, Verzögerungen, Status, typisierte OSC-Zahlen — und die
Empfangsadressen von Warteknoten sind wörtlich. Ein [[ui:exp.node.emulator]]
rendert seine eigenen Antworten mit dem Eingetroffenen (`{{request.…}}`) und den
Parametern; siehe [Störungen](faults.md#emulator).

**Nur Parameter.** Manche Felder werden vor dem ersten Schritt geöffnet, wenn es
noch keine Variable gibt: der Broker und das Topic eines [[ui:exp.node.wait_mqtt]],
das Empfangen und Ziel eines [[ui:exp.node.impairment]]. Sie nehmen Text und
Parameter an, nichts anderes (`node.params_only`).

**Wie Literale geprüft.** Ein Feld, das nur Parameter verwendet, wird vor dem
Durchlauf aufgelöst und als der Text geprüft, den der Durchlauf senden wird: Eine
URL muss `http://` oder `https://` sein, ein OSC-Ziel `IP:port` oder `host:port`,
ein Header-Name gültig. Ein Feld mit Variablen oder Generatoren wird geprüft, wenn
es läuft.

## Vorschau {#preview}

Hat der ausgewählte Knoten eine Vorlage, zeigen seine Eigenschaften, was er mit den
jetzt bekannten Werten tun wird: [[ui:exp.preview]] für ein Senden,
[[ui:exp.previewWait]] für ein Warten, [[ui:exp.previewCheck]] für einen Vergleich.
Die Engine löst es auf, mit demselben Code, den ein Durchlauf verwendet, sodass die
Vorschau dem Durchlauf nie widerspricht.

- Parameter kommen aus dem aktiven Profil.
- Variablen kommen aus dem, was der Editor in dieser Sitzung gesehen hat: die
  Schritte des letzten Durchlaufs und [[ui:exp.sendNow]].
- Ein gespeichertes Geheimnis erscheint als `••••`.
- Ein Name ohne Wert bleibt wie geschrieben, und die Vorschau listet ihn auf. Ein
  Geheimnis, das nicht gespeichert ist, wird getrennt aufgelistet.
- Generatoren verwenden den fixierten Startwert des Experiments oder `0`, wenn
  keiner fixiert ist, als erste Ausführung des Knotens. Mit einem fixierten
  Startwert zeigt die Vorschau die erzeugten Werte, die die erste Ausführung des
  Knotens in einem Durchlauf senden wird.

## Werte extrahieren {#extract}

[[ui:exp.node.extract]] liest einen Wert der neuesten HTTP-Antwort auf seinem Pfad
und schreibt ihn in eine Variable.

| Feld | Was |
| --- | --- |
| [[ui:exp.variable]] | die zu schreibende Variable; es gelten die Benennungsregeln von Parametern |
| [[ui:exp.extractFrom]] | woher der Wert kommt (unten) |
| [[ui:exp.jsonPath]], [[ui:exp.headerName]] oder [[ui:exp.pattern]] | was gelesen wird, je nach Quelle |

| [[ui:exp.extractFrom]] | Liest | Wert |
| --- | --- | --- |
| [[ui:exp.from.json]] | den Body als JSON, an einem Pfad | den JSON-Wert: Text, Zahl, Objekt, Array |
| [[ui:exp.from.header]] | den ersten Header mit diesem Namen, unabhängig von der Schreibweise | Text |
| [[ui:exp.from.status]] | den Statuscode | eine Zahl |
| [[ui:exp.from.body]] | den ganzen Body | Text |
| [[ui:exp.from.regex]] | den ersten Treffer im Body | die Erfassungsgruppe 1, wenn das Muster eine hat, sonst den ganzen Treffer |

**JSON-Pfade.** `$.token`, `$.items[0].id`, `$["a b"]`, `$['a b']['c-d']`; das
führende `$.` darf weggelassen werden (`token`, `items[0].id`), und `$` allein ist
der ganze Body.

**Reguläre Ausdrücke** verwenden die Syntax der Rust-`regex`-Engine, die kein
Look-around und keine Rückverweise hat. Der Treffer wird überall im Body gesucht;
verankern Sie ihn mit `^` und `$`, wenn das wichtig ist.

Der Schritt schlägt fehl und nennt, was fehlt, wenn:

- vor ihm auf diesem Pfad keine HTTP-Anfrage lief (`check.no_response`; der Editor
  lehnt bereits einen Graphen ab, in dem keine laufen kann, `graph.needs_http`);
- der Body kein JSON ist oder der Pfad nicht darin vorkommt;
- der Header nicht da ist oder das Muster nicht passt;
- der Body über den 256 KiB liegt, die eine Antwort behält, bei einem JSON-Pfad
  oder dem ganzen Body, und bei einem Muster, das im behaltenen Teil nichts
  getroffen hat (`extract.truncated`).

Die Zeitleiste zeigt den geschriebenen Wert: `token = abc123`.

::: tip Durch Klicken extrahieren
[[ui:exp.sendNow]] an einem [[ui:exp.node.http]] zeigt seine JSON-Antwort. Klicken
Sie einen Wert darin an: Nach der Anfrage wird ein Knoten [[ui:exp.node.extract]]
hinzugefügt, mit ausgefülltem Pfad und einem vom Schlüssel übernommenen Namen, und
der Wert ist der Vorschau sofort bekannt.
:::

## Variablen {#variables}

Eine Variable hält einen JSON-Wert. Diese Knoten schreiben eine:

| Knoten | Schreibt | Am Ausgang |
| --- | --- | --- |
| [[ui:exp.node.extract]] | den extrahierten Wert | seinen Ausgang |
| [[ui:exp.node.wait_osc]], [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_http]], [[ui:exp.node.wait_ws]] | was eingetroffen ist, Standardname `reply` (`request` bei HTTP) | nur [[ui:exp.portMatched]] |
| [[ui:exp.node.osc]], [[ui:exp.node.udp]] mit [[ui:exp.expectReply]] | die Antwort, Standardname `reply` | seinen Ausgang |

Was ein Warteknoten schreibt, ist ein Objekt; spätere Felder lesen seine Teile:

| Warten | Felder |
| --- | --- |
| OSC | `address`, `args`, `from`, `ms` |
| UDP | `text`, `hex`, `bytes`, `from`, `ms` und `match` bei einem Muster |
| MQTT | `topic` und die Felder von UDP |
| WebSocket | die Felder von UDP und `json`, wenn die Nachricht JSON ist |
| HTTP-Anfrage | `method`, `path`, `query`, `headers`, `body`, `json`, `params`, `from`, `ms` |

`ms` ist die Zeit von der letzten Aktion des Zweigs bis zum Eintreffen. Der genaue
Inhalt steht in [der Referenz der Knoten](nodes.md).

### Wo eine Variable bekannt ist {#visibility}

Eine Variable existiert ab dem Ausgang, der sie schreibt, auf den Pfaden, die durch
diesen Ausgang führen:

- Nach einer Zusammenführung alternativer Pfade — dem [[ui:exp.yes]] und
  [[ui:exp.no]] eines Zweigs, die wieder zusammentreffen — ist nur das bekannt,
  was **jeder** Pfad gesetzt hat.
- Nach [[ui:exp.node.join]] ist bekannt, was **irgendein** Zweig dorthin gesetzt
  hat: Sie alle liefen.
- Nach dem [[ui:exp.portDone]] oder [[ui:exp.portLimit]] einer
  [[ui:exp.node.loop]] und in ihrer Abbruchbedingung ist bekannt, was jede
  Iteration des Rumpfs setzt.
- Die Variable eines Warteknotens ist nach seinem [[ui:exp.portTimeout]]-Ausgang
  nicht bekannt.

Jeder parallele Zweig arbeitet an seiner eigenen Kopie der Variablen. Ein Join
führt die Kopien in der Reihenfolge seiner eingehenden Verbindungen zusammen,
wobei die spätere Verbindung einen Namen gewinnt, den beide gesetzt haben, sodass
das Ergebnis nie davon abhängt, welcher Zweig zuerst fertig wurde. Siehe
[wie ein Durchlauf sich bewegt](flow.md#parallel).

## Werte vergleichen {#compare}

[[ui:exp.node.assert_value]] lässt den Durchlauf fehlschlagen, wenn ein Vergleich
nicht gilt; [[ui:exp.node.branch_value]] verlässt den Knoten durch [[ui:exp.yes]]
oder [[ui:exp.no]]; eine [[ui:exp.node.loop]] verwendet denselben Vergleich als
Abbruchbedingung. Jeder hat einen [[ui:exp.value]], einen [[ui:exp.operator]] und
einen [[ui:exp.expected]]-Wert, und beide Texte sind Vorlagen:

| [[ui:exp.value]] | [[ui:exp.operator]] | [[ui:exp.expected]] |
| --- | --- | --- |
| `{{status}}` | [[ui:exp.op.lt]] | `300` |
| `{{reply.args[0]}}` | [[ui:exp.op.eq]] | `{{nonce}}` |

| [[ui:exp.operator]] | Gilt, wenn |
| --- | --- |
| [[ui:exp.op.eq]], [[ui:exp.op.ne]] | die beiden gleich (nicht gleich) sind — als Zahlen, wenn beide Zahlen sind (`200` gleich `200.0`), sonst als exakter Text, Schreibweise eingeschlossen |
| [[ui:exp.op.lt]], [[ui:exp.op.le]], [[ui:exp.op.gt]], [[ui:exp.op.ge]] | als Zahlen; eine Seite, die keine Zahl ist, lässt den Schritt fehlschlagen (`compare.not_numbers`) statt eines stillen *Nein* |
| [[ui:exp.op.contains]] | der Wert enthält den erwarteten Text, Schreibweise eingeschlossen |
| [[ui:exp.op.matches]] | der reguläre Ausdruck im erwarteten Wert passt irgendwo im Wert |
| [[ui:exp.op.empty]], [[ui:exp.op.not_empty]] | der Wert ist leer oder nicht, nach dem Trimmen von Leerzeichen; der erwartete Wert wird nicht verwendet |

Eine Zahl ist Text, der nach dem Trimmen von Leerzeichen als solche gelesen wird:
`42`, `-1.5`, `1e3`. Die Zeitleiste zeigt den Vergleich, wie er durchgeführt wurde,
`401 = 200`, jede Seite auf 120 Zeichen gekürzt.

## Geheimnisse {#secrets}

Ein Token oder Passwort geht als `{{secret.NAME}}` in ein Feld. Die Experimentdatei
behält nur den Namen; der Wert bleibt, wo er gespeichert ist, und erreicht nie die
Oberfläche.

### Wo Geheimnisse liegen {#secret-stores}

| Wo Signal Lab läuft | Speicher | Aus der Oberfläche |
| --- | --- | --- |
| Desktop-App unter Windows | die Windows-Anmeldeinformationsverwaltung, unter dem Dienst `SignalLab`, ein Eintrag pro Name | setzen, ersetzen, entfernen |
| Desktop-App unter Linux | keiner: Ein Durchlauf, der ein Geheimnis braucht, schlägt mit `secret.unsupported` fehl | — |
| Server | die Umgebungsvariable `SIGNALLAB_SECRET_<NAME>`, sonst die Datei `<NAME>` in seinem Geheimnisordner, `/run/secrets/signallab`, sofern nicht anders gesetzt | nur lesend |
| `signallab` auf der Kommandozeile | wie ein Server, oder die Windows-Anmeldeinformationsverwaltung mit `--secrets system` | — |

Die Überschrift des Bereichs [[ui:exp.secrets]] hat einen Tooltip, der sagt, welcher
davon gilt, wo Sie sind: der Windows-Speicher, Umgebung und Dateien des Servers
oder — in der Desktop-App unter Linux — dass es keinen Speicher gibt. Dort sagt
`secret.unsupported`, dass die Geheimnisse in der
Windows-Anmeldeinformationsverwaltung gehalten werden, die das System nicht hat.

Ein Geheimnis gehört dem Computer oder dem Server, nicht einem Experiment: Zwei
Experimente, die `{{secret.API_TOKEN}}` verwenden, verwenden denselben Wert.

Auf einem Server gewinnt die Umgebungsvariable über die Datei. Ein abschließender
Zeilenumbruch einer Datei gehört nicht zum Wert, und eine leere Datei zählt als kein
Geheimnis. Der Ordner des Servers wird mit `--secrets-dir` oder
`SIGNALLAB_SECRETS_DIR` gesetzt; siehe [den Server](../server/index.md). Für die
Kommandozeile siehe [`signallab run`](../automation/cli.md#cli-run).

| Regel | Grenze |
| --- | --- |
| Name | beginnt mit einem Buchstaben oder `_`, danach Buchstaben, Ziffern und `_`; höchstens 128 Zeichen |
| Wert | nicht leer, höchstens 16 KiB |

### Ein Geheimnis setzen {#set-secret}

Unter Windows:

1. Öffnen Sie [[ui:exp.params]]. Der Bereich [[ui:exp.secrets]] listet jedes
   Geheimnis auf, das die Felder des Experiments verwenden, jeweils
   [[ui:exp.secretStored]] oder [[ui:exp.secretMissing]].
2. Drücken Sie [[ui:exp.secretSet]] neben dem Namen oder [[ui:exp.addSecret]] für
   einen Namen, den noch kein Feld verwendet.
3. Tippen Sie den Wert — das Feld zeigt Punkte — und drücken Sie
   [[ui:exp.secretSave]] oder <kbd>Enter</kbd>. Das Feld wird geleert; nichts kann
   den Wert zurücklesen.

[[ui:exp.secretReplace]] speichert einen neuen Wert und [[ui:exp.secretRemove]]
löscht ihn aus dem Anmeldeinformationsspeicher. Ein in dieser Sitzung gespeicherter
Name wird auch in den Vorschlägen angeboten.

In einem Browser, der mit einem Server verbunden ist, sagt der Bereich nur
[[ui:exp.secretOnServer]] oder [[ui:exp.secretNotOnServer]]: Setzen Sie den Wert
dort, wo der Server läuft, auf eine von zwei Arten:

```bash
# in the server's environment
SIGNALLAB_SECRET_API_TOKEN='…'
# or as a file in its secrets folder
printf '%s' '…' > /run/secrets/signallab/API_TOKEN
```

Eine Datei wird jedes Mal gelesen, wenn ein Durchlauf startet, eine geänderte Datei
zählt also ab dem nächsten Durchlauf; eine geänderte Umgebungsvariable erfordert
einen Neustart des Servers.

### Vor einem Durchlauf {#secret-check}

Jedes Geheimnis, das die Felder des Durchlaufs verwenden, muss gespeichert sein.
Ein fehlendes stoppt den Durchlauf vor jedem Verkehr, am ersten Knoten und Feld, die
es verwenden (`secret.missing`). [[ui:exp.sendNow]] prüft dasselbe für seinen
Knoten.

### Maskierung {#masking}

Solange ein Durchlauf oder ein [[ui:exp.sendNow]] Geheimnisse verwendet, wird jedes
Vorkommen ihrer Werte in allem, was die Engine verlässt, durch `••••` ersetzt:

- Schritttexte, Fehler und die Variablen, die ein Schritt geschrieben hat;
- der Bericht des Durchlaufs;
- das Ergebnis von [[ui:exp.sendNow]], die gezeigte HTTP-Antwort eingeschlossen;
- die Frames des [[ui:dock.inspector]], die aufgezeichnet wurden, solange der
  Durchlauf dauert — in einem Hex-Dump wird jedes Byte eines Werts zu `*`, sodass
  die Offsets stimmen.

Basic-Authentifizierung sendet `name:password` in Base64; enthält einer der Teile
ein Geheimnis, wird auch dieser Base64-Text maskiert. Der Verkehr selbst trägt den
echten Wert. Die Vorschau zeigt ein gespeichertes Geheimnis als `••••`. Die
Antworten eines [[ui:exp.node.emulator]] können keine Geheimnisse verwenden.

## Befehle {#commands}

Die Vorschau ist [`experiment_resolve`](../api/commands.md#experiment_resolve);
Geheimnisse werden mit [`secret_status`](../api/commands.md#secret_status),
[`secret_set`](../api/commands.md#secret_set) und
[`secret_delete`](../api/commands.md#secret_delete) aufgelistet, gesetzt und
entfernt. Kein Befehl gibt den Wert eines Geheimnisses zurück.

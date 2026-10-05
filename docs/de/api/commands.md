---
title: Befehle
description: Jeder Befehl der Engine von Signal Lab, aufgerufen als POST /api/invoke/<command>, mit seinen Argumenten, Ergebnis und Fehlern.
---

# Befehle

Jeder Befehl, den die Engine hat, gruppiert nach dem, worauf er arbeitet. Jeder wird als
`POST /api/invoke/<command>` mit einem JSON-Objekt aus Argumenten aufgerufen und antwortet mit `200`
und seinem Ergebnis oder mit `422` und einem [`EngineError`](index.md#errors). Wie man sich
authentifiziert und was die Statuscodes bedeuten, steht in [der API-Übersicht](index.md).

## Konventionen {#conventions}

- **Argumentnamen** sind camelCase (`jobId`, `nodeId`). Ein Argument, das ein Befehl
  nicht kennt, oder ein fehlendes erforderliches wird mit
  `command.args_invalid` abgelehnt; jeder Befehl, der Argumente nimmt, kann damit fehlschlagen.
  Ein Befehl ohne Argumente liest den Body nicht.
- **Als Argument übergebene Objekte** — `config`, `request`, `document`,
  `library`, `emulator`, `profile` — verwenden die Feldnamen der Engine, meist
  snake_case (`timeout_ms`). Darin wird ein Feld, das die Engine nicht kennt,
  **ignoriert**, sodass ein falsch geschriebenes optionales Feld still seinen Standard behält. Nur
  das Formular von `feedback_send` lehnt unbekannte Felder ab.
- **Optionale** Argumente und Felder dürfen weggelassen oder als `null` gesendet werden; die
  Tabellen geben ihre Standardwerte an.
- **Ergebnisse** sind JSON. „null" heißt, der Befehl hat nichts zurückzugeben.
- **Adressen** als `IP:port` nehmen eine numerische Adresse und einen Port
  (`127.0.0.1:9000`, `[::1]:9000`); ein Hostname dort wird abgelehnt. Wo eine Tabelle
  `IP:port` oder `host:port` sagt, funktioniert auch ein Hostname: Er wird
  beim Ausführen des Befehls aufgelöst, und seine IPv4-Adresse wird verwendet, wenn er eine hat
  (so ist `localhost:9000` gleich `127.0.0.1:9000`).
- **Jobs**: Ein mit *Startet einen Job* markierter Befehl gibt eine
  [`JobInfo`](#type-jobinfo) zurück; die Arbeit läuft weiter, bis sie endet oder mit
  [`job_stop`](#job_stop) gestoppt wird. Siehe [Jobs](index.md#jobs).
- **Pfade** in Ergebnissen liegen auf der Maschine, auf der die Engine läuft — auf einem Server
  in dessen Datenordner; laden Sie sie mit [`/api/files`](index.md#files) herunter.

Die Beispiele verwenden diese Shell-Funktion:

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
invoke() {
  curl -sS -X POST "$SERVER/api/invoke/$1" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    --data "${2:-}"
}
```

## Anwendung {#application}

### app_info {#app_info}

Was die Engine ist und wo sie läuft. Keine Argumente.

**Ergebnis**

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `version` | string | Die Version von Signal Lab, `[[version]]` |
| `mode` | string | `desktop` oder `server` |
| `secrets_writable` | boolean | Ob [`secret_set`](#secret_set) und [`secret_delete`](#secret_delete) hier arbeiten können: `false` auf einem Server |
| `data_dir` | string | Der Datenordner, auf der Maschine, auf der die Engine läuft |
| `os` | string | `windows`, `linux`… |
| `arch` | string | `x86_64`, `aarch64`… |

### get_host_info {#get_host_info}

Der Name der Maschine und die Adresse, von der sie senden würde. Keine Argumente.

**Ergebnis**: `{ "local_ip": string, "hostname": string }`. `local_ip` ist die
IPv4-Adresse, die das System für Verkehr ins Internet wählt (gefunden, ohne
etwas zu senden), oder `127.0.0.1`, wenn es keine gibt. `hostname` ist der
Name des Computers, oder `localhost`, wenn das System ihn nicht nennt.

### firewall_status {#firewall_status}

Ob die Firewall des Systems andere Maschinen zu diesem Programm durchlässt. Nur
Windows hat eine Firewall pro Programm zum Lesen; anderswo ist `applies` `false`, und
von den übrigen ist nur `program` gefüllt. Keine Argumente.

**Ergebnis**

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `applies` | boolean | Es gibt hier eine Firewall pro Programm (Windows) |
| `program` | string | Das Programm, um das es bei den Regeln geht |
| `enabled` | boolean | Die Firewall ist für das Netzwerk an, in dem die Maschine gerade ist |
| `networks` | string[] | Die Arten von Netzwerk, in denen die Maschine ist: `domain`, `private`, `public` |
| `allowed` | boolean | Eine eingehende Regel lässt UDP für dieses Programm im aktuellen Netzwerk zu |
| `blocked` | boolean | Eine eingehende Regel blockiert dieses Programm im aktuellen Netzwerk; sie gewinnt gegen jede Erlaubnisregel |
| `rules` | number | Eingehende Regeln für dieses Programm, jeder Art |

**Fehler**: `firewall.failed`.

### firewall_allow {#firewall_allow}

Lässt andere Maschinen Signal Lab erreichen: Das System zeigt seine eigene
Administratorabfrage, dann werden die eingehenden Regeln des Programms (eine Blockregel eingeschlossen)
durch je eine Erlaubnisregel für Signal Lab und die Kommandozeile `signallab`
daneben ersetzt. Nur die Desktop-App unter Windows; ein Server lehnt ab, da niemand an seinem
Bildschirm ist, um die Abfrage zu beantworten.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `public` | boolean | ja | Auch in öffentlichen Netzwerken zulassen, nicht nur in privaten und Domänen-Netzwerken |

**Ergebnis**: die neue [`firewall_status`](#firewall_status).

**Fehler**: `firewall.server` (auf einem Server), `firewall.unsupported` (nicht
Windows), `firewall.declined` (die Abfrage wurde mit Nein beantwortet), `firewall.failed`.

### feedback_send {#feedback_send}

Sendet eine Nachricht an die Entwickler von Signal Lab, über den Hub des Studios, der
sie ihnen per E-Mail schickt.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `form` | object | ja | Die Nachricht, unten. Unbekannte Felder werden abgelehnt |

| Feld von `form` | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `message` | string | — | Was passiert ist; erforderlich, höchstens 20 000 Zeichen |
| `email` | string | keins | Wohin eine Antwort gehen darf |
| `meta` | object of strings | `{}` | Was die App über sich sagt (version, os, arch, mode, lang, screen) |
| `screenshots` | `{ name, data }[]` | `[]` | Bilder, `data` in base64; höchstens 6, je 8 MiB |
| `logs` | `{ name, text }[]` | `[]` | Textdateien; höchstens 4, je 2 MiB |

Alles zusammen höchstens 15 MiB.

**Ergebnis**: `{ "id": string }`, die Referenz, die die Entwickler erhalten.

**Fehler**: `feedback.message_required`, `feedback.message_too_long`,
`feedback.too_many_files`, `feedback.file_too_large`, `feedback.too_large`,
`feedback.invalid`, die Ablehnungen des Hubs (`feedback.email_invalid`,
`feedback.file_type`, `feedback.rate_limited`, `feedback.disabled`,
`feedback.send_failed`, `feedback.failed`) und die `transport.*` des Netzwerks.

```bash
invoke app_info
# {"version":"[[version]]","mode":"server","secrets_writable":false,"data_dir":"/data","os":"linux","arch":"x86_64"}
```

## Jobs {#jobs}

### jobs_list {#jobs_list}

Die laufenden Jobs, älteste zuerst. Keine Argumente.

**Ergebnis**: [`JobInfo`](#type-jobinfo)`[]`.

### job_stop {#job_stop}

Stoppt einen Job sofort: Seine Sockets schließen, sein Relay, Server oder seine Verbindung geht.
Ein gestoppter Job sendet kein [`job://ended`](events.md#event-job-ended); ein gestoppter
Durchlauf speichert keinen Bericht.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `id` | number | ja | Die `id` des Jobs |

**Ergebnis**: `true`, wenn ein Job mit dieser id lief, sonst `false`.

### jobs_stop_all {#jobs_stop_all}

Stoppt jeden laufenden Job, wer immer ihn gestartet hat. Keine Argumente.

**Ergebnis**: null.

```bash
invoke jobs_list
# [{"id":3,"kind":"osc-monitor","label":"OSC monitor 0.0.0.0:9000","params":{"bind":"0.0.0.0:9000"},"started_ms":1759600000000}]
invoke job_stop '{"id":3}'
# true
```

## Experimente und Durchläufe {#experiments}

Diese Befehle nehmen und geben ein Experimentdokument (`Experiment`): das JSON,
das der Editor speichert und exportiert, mit `version`, `name`, `params`, `profiles`,
`profile`, `seed`, `cookies`, `nodes` und `edges`. Seine Knoten stehen in
[Knoten](../experiments/nodes.md); seine Parameter, Profile und Vorlagen in
[Daten](../experiments/data.md). Ein Dokument ist höchstens 4 MiB
(`file.too_large`). Um ein Experiment auszuführen und auf sein Ergebnis zu warten, verwenden Sie
[`POST /api/run`](run.md) statt [`experiment_start`](#experiment_start).

### experiment_load {#experiment_load}

Das Arbeitsexperiment: `experiment.json` im Datenordner — auf einem Server das,
das seine Oberfläche zeigt. Gibt es keines, das Startexperiment. Ältere
Dokumentversionen werden migriert. Keine Argumente.

**Ergebnis**: `Experiment`.

**Fehler**: `file.io`, `file.json_invalid` (mit `path`, `line`
und `column` der Datei), `file.too_large`, `doc.version_unsupported` und die übrigen
`doc.*`-Prüfungen.

### experiment_save {#experiment_save}

Ersetzt das Arbeitsexperiment, `experiment.json` im Datenordner. Es wird
zuerst in eine temporäre Datei geschrieben, sodass ein fehlgeschlagener Schreibvorgang das vorherige belässt.

::: warning
Auf einem Server ist dies das Dokument, auf dem der Editor jedes Browsers arbeitet.
:::

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `document` | `Experiment` | ja | Das Dokument |

**Ergebnis**: string, der geschriebene Pfad.

**Fehler**: `doc.*`, die Größenprüfungen des Dokuments (`param.*`, `params.too_many`,
`profile.*`, `profiles.too_many`, `seed.range`), `file.too_large`, `file.io`.

### experiment_parse {#experiment_parse}

Liest ein Experiment aus JSON-Text, wie [[ui:exp.importJson]] es tut. Versionen 1
bis 8 werden auf Version 9 migriert, die aktuelle; eine Datei von vor Version 8
öffnet mit `cookies` aus, sodass sie so läuft wie zuvor. Eine Byte-Order-Mark wird übersprungen.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `text` | string | ja | Der Text der Datei |

**Ergebnis**: `Experiment`.

**Fehler**: `file.json_invalid` (`line`, `column`), `file.too_large`,
`doc.version_unsupported`, `doc.*`, die Größenprüfungen von
[`experiment_save`](#experiment_save).

### experiment_export {#experiment_export}

Schreibt eine Momentaufnahme eines Dokuments nach `exports/experiment-<ms>-<16 hex digits>.json`
im Datenordner. Jeder Export ist eine neue Datei.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `document` | `Experiment` | ja | Das Dokument |

**Ergebnis**: string, der geschriebene Pfad.

**Fehler**: die von [`experiment_save`](#experiment_save).

### experiment_validate {#experiment_validate}

Prüft, ob ein Dokument mit seinem aktiven Profil (oder seinen Standardwerten) laufen würde:
den Graphen, jedes Feld, Parameter und ob jedes Geheimnis, das es nennt, gespeichert ist.
Ein blockierendes Problem ist der Fehler. Bei Erfolg sagt es, welches der *anderen*
Profile fehlschlagen würde, sodass Sie es vor dem Wechseln wissen.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `document` | `Experiment` | ja | Das Dokument |
| `overrides` | object of strings | nein | Parameterwerte nur für diese Prüfung, wie [[ui:exp.runWith]] sie angibt |

**Ergebnis**: `{ "profile": string or null, "error": EngineError }[]` — jedes
andere Profil, das nicht validieren würde (`null`: die Standardwerte, ohne
Profil). Eine leere Liste heißt, jedes Profil ist in Ordnung.

**Fehler**: jeder Validierungscode (`doc.*`, `graph.*`, `node.*`, `param.*`,
`profile.*`, `template.*`, `loop.*`…), `run.override_unknown` (eine Überschreibung
für einen Parameter, den das Dokument nicht hat), `secret.missing`,
`secret.store`, `secret.unsupported`.

### experiment_resolve {#experiment_resolve}

Ein Knoten mit ausgefüllten Vorlagen, wie die Vorschau des Editors ihn zeigt: die
Werte des aktiven Profils und die Variablenwerte, die Sie angeben. Geheimnisse werden
als `••••` gezeigt, nie ihre Werte. Namen ohne Wert bleiben wie geschrieben und
werden aufgelistet.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `document` | `Experiment` | ja | Das Dokument |
| `nodeId` | string | ja | Der Knoten |
| `vars` | object | ja | Zu verwendende Variablenwerte, nach Namen; `{}` für keine |

**Ergebnis**: `{ "node": node, "missing": string[] }`.

**Fehler**: `node.not_found`, `template.*`, `secret.store`,
`secret.unsupported`.

### experiment_send_node {#experiment_send_node}

[[ui:exp.sendNow]]: führt einen Knoten für sich aus, durch denselben Code, den ein Durchlauf
verwendet. Eine Aktion wird gesendet; ein Warten hört von nun an zu, bis es passt oder
in eine Zeitüberschreitung läuft. Ein Knoten [[ui:exp.node.ws_send]] oder [[ui:exp.node.wait_ws]] öffnet die
Verbindung, die sein [[ui:exp.node.ws_connect]]-Knoten beschreibt. Nichts wird mit
Cookies gesendet, und die Knoten [[ui:exp.node.impairment]] und [[ui:exp.node.emulator]]
des Durchlaufs werden nicht geöffnet.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `document` | `Experiment` | ja | Das Dokument; sein aktives Profil gibt die Parameterwerte |
| `nodeId` | string | ja | Eine Aktion oder ein Warten |
| `vars` | object | ja | Variablenwerte, die die Vorlagen des Knotens lesen; `{}` für keine |

**Ergebnis**

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `detail` | string | Was passiert ist, auf Englisch |
| `response` | [`HttpResponse`](#type-httpresponse) oder null | Die Antwort eines HTTP-Knotens |
| `vars` | object | Was der Schritt gesetzt hat: die Antwort eines Wartens oder was die [[ui:exp.node.extract]]-Knoten nach einer Anfrage aus deren Antwort nehmen |

Geheimniswerte werden in all dem maskiert.

**Fehler**: `node.not_found`, `run.not_an_action` (keine Aktion und kein
Warten), `ws.connection_unknown`, `secret.missing`, `template.*` und alles, womit der
Schritt fehlschlägt: `transport.*`, `wait.timeout`, `check.*`…

### experiment_start {#experiment_start}

Startet einen Durchlauf, wie [[ui:exp.run]] es tut, und kehrt sofort zurück. Seine Schritte treffen als
[`experiment://step`](events.md#event-experiment-step)-Ereignisse ein, sein Ende als
[`experiment://ended`](events.md#event-experiment-ended), und sein Bericht wird
unter `runs/` im Datenordner gespeichert. Warten, Emulatoren, Impairment-Relays und
MQTT-Abonnements öffnen vor dem ersten Schritt, sodass ein belegter Port hier fehlschlägt. Ein Durchlauf
länger als 300 s schlägt mit `run.timeout` fehl. *Startet einen Job*
(`experiment`).

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `document` | `Experiment` | ja | Das Dokument |
| `overrides` | object of strings | nein | Parameterwerte nur für diesen Durchlauf |
| `seed` | number | nein | Der Startwert des Durchlaufs, 0 bis 9007199254740991; Standard: der des Dokuments, sonst ein neuer |

**Ergebnis**: [`JobInfo`](#type-jobinfo), `params.name` der Name des Experiments.

**Fehler**: alles, was [`experiment_validate`](#experiment_validate) meldet,
`seed.range`, `transport.address_in_use` und die anderen Bind-Fehler,
`emulator.*`, `impair.*`, `node.params_only` (eine Listen-Adresse oder Broker oder Topic
eines MQTT-Wartens, die beim Start des Durchlaufs nicht feststehen), und die
[`mqtt_connect`](#mqtt_connect)-Fehler eines Brokers, den ein MQTT-Warten nicht erreichen kann.

### experiment_runs {#experiment_runs}

Durchläufe, aus ihren Berichten in `runs/` zurückgelesen, neueste zuerst. Ein Bericht, der
nicht gelesen werden kann, bleibt weg.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `name` | string | nein | Nur Durchläufe des Experiments mit genau diesem Namen |
| `limit` | number | nein | Höchstens so viele; Standard 50, höchstens 500 |

**Ergebnis**: Durchlauf-Zusammenfassungen:

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `name` | string | Der Dateiname des Berichts, `run-<ms>-<job>.json`: was [`experiment_compare`](#experiment_compare) nimmt |
| `experiment` | string | Der Name des Experiments |
| `started_ms`, `ended_ms` | number | Millisekunden seit 1970 |
| `outcome` | string | `passed` oder `failed` |
| `seed` | number | Der Startwert des Durchlaufs |
| `profile` | string oder null | Sein Profil |
| `loads` | object[] | Jeder Lastschritt: `node`, `sent`, `rps`, `p95_ms`, `error_rate`, `held` (jede Schwelle gehalten) |

**Fehler**: `file.io`.

### experiment_compare {#experiment_compare}

Zwei Durchläufe nebeneinander, Lastschritt für Lastschritt, wie die
[[ui:exp.compare]] der Zeitleiste sie zeigt. Schritte werden über die Knoten-id zugeordnet.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `a` | string | ja | Der Berichtsdateiname des früheren Durchlaufs |
| `b` | string | ja | Der Berichtsdateiname des späteren Durchlaufs |

**Ergebnis**: `{ "a": summary, "b": summary, "steps": [...] }`, jeder Schritt mit
`node`, `missing_in` (`a` oder `b`, wenn nur ein Durchlauf ihn hat), `metrics`,
`sent` (`[a, b]`), `thresholds_a` und `thresholds_b` (jede Schwelle als
`{ metric, op, value, actual, held }`). `metrics` listet neun Metriken, jede als
`{ metric, a, b, change, percent, worse }`: `metric` ist `p50_ms`, `p90_ms`,
`p95_ms`, `p99_ms`, `mean_ms`, `max_ms`, `error_rate`, `rps` oder `missed`;
`change` ist `b − a`; `percent` die Änderung in % von `a` (null, wenn `a` 0 ist);
`worse`, dass sie sich in die falsche Richtung bewegte — höher, oder niedriger bei `rps` — um 5 % oder
mehr, oder von 0 auf irgendetwas. Ein Schritt, den nur ein Durchlauf hat, ist nie `worse`. Siehe
[Last](../experiments/load.md).

**Fehler**: `runs.name_invalid` (alles außer einem Berichtsdateinamen, keine Ordner),
`runs.not_found`, `file.json_invalid`, `file.io`.

```bash
invoke experiment_validate "$(jq '{document: .}' experiment.json)"
# []
```

## Geheimnisse {#secrets}

Geheimniswerte werden von Experimenten als `{{secret.NAME}}` verwendet und verlassen die
Engine nie: Kein Befehl gibt eines zurück. Wo sie aufbewahrt werden, hängt davon ab, wo die Engine
läuft:

| Wo | Speicher | Setzen und Entfernen |
| --- | --- | --- |
| Desktop-App, Windows | Windows-Anmeldeinformationsverwaltung | Ja |
| Desktop-App, Linux | Keiner | `secret.unsupported` |
| Server | `SIGNALLAB_SECRET_<NAME>`, oder die Datei `<NAME>` in `--secrets-dir` (Standard `/run/secrets/signallab`) | Nein: `secret.read_only` |

Ein Name beginnt mit einem lateinischen Buchstaben oder `_`, fährt mit lateinischen Buchstaben, Ziffern
und `_` fort und hat höchstens 128 Zeichen (`secret.name_invalid`).

### secret_status {#secret_status}

Welche der gegebenen Namen einen gespeicherten Wert haben.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `names` | string[] | ja | Die nachzuschlagenden Namen |

**Ergebnis**: ein Objekt, Name → `true` (gespeichert) oder `false`.

**Fehler**: `secret.name_invalid` (auf einem Server), `secret.store`,
`secret.too_large` (eine Serverdatei über 16 KiB), `secret.unsupported`.

### secret_set {#secret_set}

Speichert einen Wert unter einem Namen und ersetzt den dortigen. Nur Desktop-App.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `name` | string | ja | Der Name |
| `value` | string | ja | Nicht leer; höchstens 16 KiB |

**Ergebnis**: null.

**Fehler**: `secret.read_only` (auf einem Server), `secret.unsupported`,
`secret.name_invalid`, `secret.empty`, `secret.too_large`, `secret.store`.

### secret_delete {#secret_delete}

Entfernt einen gespeicherten Wert. Einen zu entfernen, der nicht gespeichert ist, ist kein Fehler.
Nur Desktop-App.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `name` | string | ja | Der Name |

**Ergebnis**: null.

**Fehler**: `secret.read_only` (auf einem Server), `secret.unsupported`,
`secret.name_invalid`, `secret.store`.

```bash
invoke secret_status '{"names":["API_TOKEN","MQTT_PASSWORD"]}'
# {"API_TOKEN":true,"MQTT_PASSWORD":false}
```

## OSC {#osc}

Siehe [OSC](../protocols/osc.md) für die Ansicht, der diese Befehle dienen.

### osc_send {#osc_send}

Sendet eine OSC-Nachricht in einem UDP-Datagramm, aus einem frischen Socket.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `target` | string | ja | `IP:port` oder `host:port`, wohin gesendet wird; ein Name wird aufgelöst, seine IPv4-Adresse genommen, wenn er eine hat |
| `address` | string | ja | Die OSC-Adresse, `/mixer/fader/1`; sie beginnt mit `/` |
| `args` | [`OscArg`](#type-oscarg)`[]` | ja | Die Argumente; `[]` für keine |

**Ergebnis**: Zahl, die gesendeten Bytes.

**Fehler**: `node.osc_address` (kein führendes `/`; Feld `address`),
`transport.target_invalid` (kein Port oder keine der beiden Formen), `transport.dns` (der
Name lässt sich nicht auflösen), `transport.*`.

### osc_monitor_start {#osc_monitor_start}

Hört auf einem UDP-Port auf OSC und dekodiert jedes Paket. Jedes trifft als
[`osc://message`](events.md#event-osc-message)-Ereignis ein. *Startet einen Job*
(`osc-monitor`, `params.bind`).

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `bind` | string | ja | `IP:port`, worauf gehört wird: `0.0.0.0:9000` jede Netzwerkkarte, `127.0.0.1:9000` nur diese Maschine |

**Ergebnis**: [`JobInfo`](#type-jobinfo).

**Fehler**: `node.bind_invalid`, `transport.address_in_use`,
`transport.address_unavailable`, `transport.denied`, `wait.bind_failed`. Der
Job endet mit `wait.receive_failed`, wenn der Socket nicht mehr empfangen kann.

### osc_generator_start {#osc_generator_start}

Sendet einen Strom von OSC-Nachrichten, deren einziges Argument einer Wellenform folgt.
Fortschritt trifft als [`osc://gen-tick`](events.md#event-osc-gen-tick) ein. *Startet
einen Job* (`osc-gen`, `params.target`, `params.address`).

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `config` | object | ja | Unten |

| Feld von `config` | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `target` | string | — | `IP:port` oder `host:port`, wohin gesendet wird; ein Name wird einmal beim Start des Jobs aufgelöst |
| `address` | string | — | Die OSC-Adresse; sie beginnt mit `/` |
| `rate` | number | — | Nachrichten pro Sekunde, zwischen 0,1 und 5000 gehalten |
| `waveform` | string | — | `sine`, `triangle`, `saw` (fallend: `max` bis `min`, dann sofort zurück), `ramp` (steigend: `min` bis `max`, dann sofort zurück), `square`, `random` oder `constant` (`max`) |
| `freq` | number | — | Zyklen der Wellenform pro Sekunde |
| `min`, `max` | number | — | Der Wertebereich |
| `as_int` | boolean | `false` | Runden und eine ganze Zahl statt eines Float senden |
| `duration_s` | number | `0` | Nach so vielen Sekunden stoppen; 0 läuft bis zum Stoppen |

**Ergebnis**: [`JobInfo`](#type-jobinfo).

**Fehler**: `node.osc_address`, `transport.target_invalid`, `transport.dns`,
`transport.*`. Der Job endet mit einem `transport.*`-Fehler, wenn ein Senden fehlschlägt.

```bash
invoke osc_send '{"target":"127.0.0.1:9000","address":"/cue/go","args":[{"type":"int","value":1}]}'
# 16
invoke osc_monitor_start '{"bind":"0.0.0.0:9000"}'
```

## HTTP und Cookies {#http}

Siehe [HTTP](../protocols/http.md).

### http_request {#http_request}

Sendet eine HTTP-Anfrage und gibt die Antwort zurück. Eine Anfrage, die keine
Antwort bekommt — abgelehnt, in eine Zeitüberschreitung gelaufen, ein Name, der sich nicht auflöst, ein Zertifikat,
dem nicht getraut wird —, ist **kein** Fehler des Befehls: Die Antwort sagt
es in `error` und `cause`.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `request` | [`HttpRequest`](#type-httprequest) | ja | Die Anfrage |
| `cookies` | boolean | nein | Den Cookie-Speicher der Ansicht [[ui:nav.http]] senden und behalten, was die Antwort setzt; Standard `false` |

**Ergebnis**: [`HttpResponse`](#type-httpresponse).

**Fehler**: `http.client_failed` (die Anfrage konnte nicht einmal vorbereitet werden).

### http_burst_start {#http_burst_start}

Sendet eine Anfrage viele Male, mehrere gleichzeitig, und misst sie. Ohne
`rate` sendet jeder Worker wieder, sobald er eine Antwort hat; mit einer
starten Anfragen nach einem festen Zeitplan, wie langsam die Antworten auch sind, und eine Anfrage,
die mehr als 50 ms über ihren Zeitpunkt hinaus auf einen freien Worker gewartet hat, wird übersprungen und
als verpasst gezählt. Fortschritt trifft als
[`http://burst-progress`](events.md#event-http-burst-progress) zehnmal pro
Sekunde ein. *Startet einen Job* (`http-burst`, `params.method`, `params.url` und
`params.rate`, wenn getaktet).

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `config` | object | ja | Die Felder von [`HttpRequest`](#type-httprequest) und die folgenden, in einem Objekt |

| Feld von `config` | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `concurrency` | number | — | Höchstens so viele in Flug, zwischen 1 und 512 gehalten |
| `total` | number | `0` | Nach so vielen Anfragen stoppen; 0: keine Anzahl |
| `duration_s` | number | `0` | Nach so vielen Sekunden stoppen; 0: keine Zeitbegrenzung |
| `rate` | number | `0` | Pro Sekunde gestartete Anfragen, 0,1 bis 100 000; 0: so schnell, wie Antworten kommen |
| `cookies` | boolean | `false` | Den Cookie-Speicher der Ansicht [[ui:nav.http]] verwenden |

Mit weder `total` noch `duration_s` läuft der Burst, bis er gestoppt wird.

**Ergebnis**: [`JobInfo`](#type-jobinfo).

**Fehler**: `http.rate_invalid`, `http.duration_invalid`, `http.client_failed`.

### http_cookies {#http_cookies}

Der Cookie-Speicher der Ansicht [[ui:nav.http]]: jeder Cookie, der nicht abgelaufen ist. Auf einem Server
gibt es einen Speicher für jede Seite und jedes Skript. Keine Argumente.

**Ergebnis**: Cookies, jeder mit `name`, `value`, `domain`, `host_only` (kein
Domain-Attribut: nur der Host, der ihn gesetzt hat, bekommt ihn zurück), `path`, `expires`
(Unix-Sekunden, null für einen Sitzungs-Cookie), `secure`, `http_only` und
`same_site` (string oder null).

### http_cookies_clear {#http_cookies_clear}

Leert den Cookie-Speicher der Ansicht [[ui:nav.http]]. Keine Argumente.

**Ergebnis**: null.

```bash
invoke http_request '{"request":{"method":"GET","url":"http://127.0.0.1:8080/health","headers":[["Accept","application/json"]],"body":null,"timeout_ms":5000}}' \
  | jq '{status, latency_ms, body}'
```

## WebSocket {#websocket}

Siehe [WebSocket](../protocols/websocket.md). Eine Verbindung, die `ws_connect`
öffnet, ist ein Job; die anderen nennen sie über `jobId`.

### ws_connect {#ws_connect}

Öffnet einen WebSocket und hält ihn offen. Was eintrifft und was gesendet wird, kommt als
[`ws://messages`](events.md#event-ws-messages) alle 100 ms; der Zustand der Verbindung
als [`ws://state`](events.md#event-ws-state). *Startet einen Job*
(`websocket`, `params.url`).

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | ja | Wo und wie verbunden wird |

**Ergebnis**: [`JobInfo`](#type-jobinfo).

**Fehler**: `ws.url_invalid`, `ws.header_invalid`, `ws.protocol_invalid`,
`ws.handshake_status` (der Server hat das Upgrade mit einem anderen Status beantwortet),
`ws.subprotocol_refused`, `ws.handshake_failed`, `transport.*`.

### ws_send {#ws_send}

Sendet eine Nachricht auf einer offenen Verbindung.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `jobId` | number | ja | Der Job der Verbindung |
| `message` | object | ja | `{ "text": "…" }` für eine Textnachricht oder `{ "hex": "de ad be ef" }` für eine binäre; genau eines davon |

**Ergebnis**: Zahl, die gesendeten Bytes.

**Fehler**: `ws.not_connected`, `ws.payload_required` (keines oder beide),
`hex.invalid` (auch ein leeres `hex`), `node.too_long` (über 16 MiB, Feld
`payload`; es wird nichts gesendet und die Verbindung bleibt offen), `ws.closed`, `transport.*` (`transport.timeout`, wenn der Server 10 s lang aufhörte zu
lesen).

### ws_close {#ws_close}

Schließt eine Verbindung mit einem Schließen-Handshake und wartet bis zu 2 s auf die Antwort des
Servers; dann endet der Job. Sobald eine Verbindung geendet hat, ist ihr Job fort und
sie zu schließen ist `ws.not_connected`.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `jobId` | number | ja | Der Job der Verbindung |
| `code` | number | nein | 1000 oder 3000 bis 4999 für einen eigenen der Anwendung; Standard 1000 |
| `reason` | string | nein | Höchstens 123 Bytes; Standard leer |

**Ergebnis**: `{ "code", "reason", "by", "error" }` — `by` ist `client`,
`server` oder `lost`; `code` ist 1005, wenn das Schließen keinen trug, und 1006, wenn
es kein Schließen-Frame gab.

**Fehler**: `ws.close_code`, `node.too_long`, `ws.not_connected`.

### ws_exchange {#ws_exchange}

Ein Austausch ohne Job: verbinden, eine Nachricht senden, wenn eine gegeben ist, auf eine
Antwort warten, wenn darum gebeten, schließen.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | ja | Wo und wie verbunden wird |
| `message` | object | nein | `{ "text" }` oder `{ "hex" }`, wie bei [`ws_send`](#ws_send) |
| `expect` | object | nein | Worauf gewartet wird: `mode` (`any`, `contains`, `regex`, `hex`; Standard `any`), `pattern` (Standard leer), `timeout_ms` (Standard 2000) |

Mit `expect` und ohne `message` zählt die erste passende Nachricht nach dem Verbinden —
eine Begrüßung.

**Ergebnis**: `{ "handshake", "sent", "reply", "closed" }` — `handshake` ist
`{ url, peer, local, protocol, ms }`; `sent` die gesendeten Bytes oder null; `reply`
`{ kind, text, hex, bytes, json, ms }` oder null (`json`: eine Textantwort geparst,
sonst null; `ms`: seit dem Senden oder seit dem Verbinden, wenn nichts gesendet wurde);
`closed` wie [`ws_close`](#ws_close) es zurückgibt.

**Fehler**: die von [`ws_connect`](#ws_connect) und [`ws_send`](#ws_send),
`wait.timeout` (mit `ms`, `unmatched` und `target`), `regex.invalid` und
`hex.invalid` (ein `pattern`, das sich nicht parsen lässt).

```bash
invoke ws_exchange '{"config":{"url":"ws://127.0.0.1:9001/"},"message":{"text":"{\"type\":\"ping\"}"},"expect":{"mode":"contains","pattern":"pong"}}' \
  | jq .reply.text
```

## MQTT {#mqtt}

MQTT 3.1.1 über einfaches TCP, QoS 0, 1 und 2. Siehe [MQTT](../protocols/mqtt.md).

### mqtt_connect {#mqtt_connect}

Verbindet mit einem Broker und hält die Verbindung. Der Befehl kehrt zurück, sobald der
Broker die Verbindung akzeptiert hat (CONNACK), sodass ein falsches Passwort oder ein geschlossener
Port sein Fehler ist. Nachrichten treffen als
[`mqtt://messages`](events.md#event-mqtt-messages) alle 100 ms ein; Zustands-
änderungen als [`mqtt://state`](events.md#event-mqtt-state); abgeschlossene QoS-1/2-
Veröffentlichungen und -Abmeldungen als [`mqtt://ack`](events.md#event-mqtt-ack).
*Startet einen Job* (`mqtt`, `params.broker`, `params.client`).

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | ja | Der Broker und wie verbunden wird |

**Ergebnis**: [`JobInfo`](#type-jobinfo).

**Fehler**: `mqtt.client_id_required`, `transport.*` (abgelehnt, unerreichbar,
`dns`, `timeout` nach 6 s), `mqtt.no_answer` (kein CONNACK innerhalb von 6 s),
`mqtt.protocol`, `mqtt.refused_protocol`, `mqtt.refused_client_id`,
`mqtt.refused_unavailable`, `mqtt.refused_credentials`,
`mqtt.refused_not_authorized`, `mqtt.refused`.

### mqtt_publish {#mqtt_publish}

Veröffentlicht auf einer offenen Verbindung.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `jobId` | number | ja | Der Job der Verbindung |
| `topic` | string | ja | Das Topic: nicht leer und kein `+` oder `#` |
| `payload` | string | ja | Der Payload, als UTF-8 gesendet |
| `qos` | number | ja | 0, 1 oder 2 (über 2 wird als 2 gesendet) |
| `retain` | boolean | ja | Den Broker bitten, ihn zu behalten; ein leerer Payload mit `retain` löscht einen retained-Wert |

**Ergebnis**: null. Eine QoS-1- oder -2-Veröffentlichung wird später von
[`mqtt://ack`](events.md#event-mqtt-ack) bestätigt.

**Fehler**: `node.topic_wildcard` (Feld `topic`) und `mqtt.topic_required`
(Feld `topic`), wie bei [`mqtt_publish_once`](#mqtt_publish_once) — der Befehl
weist sie ab, bevor er nach der Verbindung sucht; `mqtt.not_connected`.

### mqtt_subscribe {#mqtt_subscribe}

Abonniert mit einer offenen Verbindung Filter. Was der Broker gewährt, trifft als
[`mqtt://state`](events.md#event-mqtt-state) mit `state: "subscribed"` ein.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `jobId` | number | ja | Der Job der Verbindung |
| `filters` | `{ filter, qos }[]` | ja | Mindestens einer; `qos` ist standardmäßig 0. `+` und `#` sind Wildcards |

**Ergebnis**: null.

**Fehler**: `mqtt.filter_required`, `mqtt.not_connected`.

### mqtt_unsubscribe {#mqtt_unsubscribe}

Meldet eine offene Verbindung von Filtern ab.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `jobId` | number | ja | Der Job der Verbindung |
| `filters` | string[] | ja | Mindestens einer |

**Ergebnis**: null. Die Antwort des Brokers trifft als
[`mqtt://ack`](events.md#event-mqtt-ack) mit `kind: "unsubscribed"` ein.

**Fehler**: `mqtt.filter_required`, `mqtt.not_connected`.

### mqtt_publish_once {#mqtt_publish_once}

Verbindet, veröffentlicht eine Nachricht, wartet auf die Bestätigung, die ihre QoS verlangt
(bis zu 6 s), trennt. Sie bringt ihre eigene Verbindung mit, unter einer eigenen Client-id
— die ersten 12 Zeichen von `client_id`, `-o` und eine Zahl —, sodass sie nie eine
lebende Verbindung mit dieser id vom Broker verdrängt.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | ja | Der Broker; `subscribe` wird nicht verwendet |
| `topic` | string | ja | Nicht leer und ohne `+` oder `#` |
| `payload` | string | ja | Als UTF-8 gesendet |
| `qos` | number | ja | 0, 1 oder 2 (über 2 wird als 2 gesendet) |
| `retain` | boolean | ja | Den Broker bitten, ihn zu behalten |

**Ergebnis**: string, eine von Signal Lab geschriebene Zusammenfassung:
`<topic> → <broker> · <bytes> B · qos<n>`, mit ` retained`, wenn retained.

**Fehler**: `mqtt.topic_required`, `node.topic_wildcard` und die von
[`mqtt_connect`](#mqtt_connect) außer `mqtt.client_id_required`: eine leere
`client_id` wird hier akzeptiert.

```bash
invoke mqtt_publish_once '{"config":{"host":"127.0.0.1","port":1883,"client_id":"lab"},"topic":"lab/lamp/set","payload":"ON","qos":1,"retain":false}'
# "lab/lamp/set → 127.0.0.1:1883 · 2 B · qos1"
```

## Broadcast, Multicast und Discovery {#broadcast}

Siehe [Broadcast und Discovery](../protocols/broadcast.md).

::: danger
Broadcast und ein Sweep erreichen jeden Host eines Netzwerksegments. Senden Sie nur in
Netzwerken, für die Sie verantwortlich sind.
:::

### broadcast_send {#broadcast_send}

Sendet ein Datagramm an jedes Ziel, einmal.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `config` | object | ja | Unten |

| Feld von `config` | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `mode` | string | — | `list`, `broadcast`, `multicast` oder `sweep` |
| `target` | string | — | Je nach Modus, unten |
| `port` | number | `0` | Der Port, nur für `sweep` |
| `payload` | [`Payload`](#type-payload) | — | Was jedes Datagramm trägt |
| `bind` | string | beliebig | Das lokale `IP:port`, von dem gesendet wird; leer oder null: `0.0.0.0:0` (`[::]:0`, wenn jedes Ziel IPv6 ist) |
| `ttl` | number | `1` | IP-TTL oder das Multicast-Hop-Limit; 1 bis 255 |
| `multicast_loop` | boolean | `true` | Multicast kommt auch zu dieser Maschine zurück |
| `rate`, `count`, `duration_s` | number | `0` | Nur für [`broadcast_beacon_start`](#broadcast_beacon_start) |

| `mode` | `target` |
| --- | --- |
| `list` | `IP:port`- oder `host:port`-Einträge, getrennt durch Kommas, Semikolons oder Zeilenumbrüche (nicht durch Leerzeichen); ein Name wird aufgelöst, seine IPv4-Adresse genommen, wenn er eine hat |
| `broadcast` | `255.255.255.255:port` oder eine mit `.255` endende Adresse mit ihrem Port |
| `multicast` | Eine Gruppe von 224.0.0.0 bis 239.255.255.255 mit ihrem Port |
| `sweep` | Ein CIDR-Block, `192.0.2.0/24`: jeder nutzbare Host auf `port`; höchstens 1024 Hosts, also `/22` oder enger |

**Ergebnis**

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `targets` | number | Ziele |
| `packets`, `bytes` | number | Was hinausging |
| `errors` | number | Datagramme, die nicht gesendet werden konnten |
| `resolved` | string[] | Die ersten 8 Ziele |
| `summary` | string | Der Payload in einer Zeile |
| `error` | `EngineError` | Warum das erste gescheiterte Datagramm scheiterte; bleibt weg, wenn keines scheiterte |

**Fehler**: `broadcast.target_required`, `broadcast.not_broadcast`,
`broadcast.ipv6`, `broadcast.not_multicast`, `broadcast.sweep_port`,
`broadcast.cidr_invalid`, `broadcast.prefix_invalid`,
`broadcast.sweep_too_large`, `node.osc_address` (eine OSC-Adresse muss mit
`/` beginnen), `hex.empty`, `hex.invalid`, `node.bind_invalid`,
`socket.option_failed`, `transport.target_invalid`, `transport.dns`, Bind-Fehler.

### broadcast_beacon_start {#broadcast_beacon_start}

Sendet dieselbe Runde — ein Datagramm pro Ziel — immer wieder. Seine Zähler
treffen als [`broadcast://emit-stat`](events.md#event-broadcast-emit-stat) alle
250 ms ein. Nach mehr als 32 gescheiterten Sendevorgängen ohne einen einzigen gesendeten stoppt er mit
dem Grund. *Startet einen Job* (`beacon`, `params.mode`, `params.target`,
`params.targets`, `params.rate`).

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `config` | object | ja | Wie für [`broadcast_send`](#broadcast_send), mit den drei unten |

| Feld von `config` | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `rate` | number | — | Runden pro Sekunde; über 0, und Runden × Ziele höchstens 50 000 Datagramme pro Sekunde |
| `count` | number | `0` | Nach so vielen Runden stoppen; 0: keine Anzahl |
| `duration_s` | number | `0` | Nach so vielen Sekunden stoppen; 0: bis zum Stoppen |

**Ergebnis**: [`JobInfo`](#type-jobinfo).

**Fehler**: die von [`broadcast_send`](#broadcast_send),
`broadcast.rate_invalid`, `broadcast.rate_limit`.

### discovery_start {#discovery_start}

Hört auf einem UDP-Port, führt eine Liste jedes Peers, der etwas sendet, und
kann Sonden beantworten wie ein Gerät. Die Peers treffen als
[`broadcast://peers`](events.md#event-broadcast-peers) alle 400 ms ein. *Startet
einen Job* (`discovery`, `params.bind`, `params.groups`, `params.joined`).

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `config` | object | ja | Unten |

| Feld von `config` | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `bind` | string | — | `IP:port`, worauf gehört wird |
| `groups` | string[] | `[]` | Multicast-Gruppen zum Beitreten (IPv4) |
| `interface` | string | beliebig | Die lokale IPv4-Adresse, auf der den Gruppen beigetreten wird |
| `reuse` | boolean | `true` | Den Port mit einem bereits darauf hörenden Programm teilen (`SO_REUSEADDR`) |
| `respond` | boolean | `false` | Antworten, was eintrifft |
| `response` | [`Payload`](#type-payload) | keins | Die Antwort; nötig mit `respond` |
| `respond_delay_ms` | number | `0` | So lange vor dem Antworten warten |
| `match_contains` | string | keins | Nur Datagramme beantworten, deren Text dies enthält |

Höchstens 512 Peers werden aufgelistet; spätere werden nicht hinzugefügt.

**Ergebnis**: [`JobInfo`](#type-jobinfo).

**Fehler**: `node.bind_invalid`, `broadcast.port_shared` (der Port ist belegt
und `reuse` ist aus), `broadcast.interface_invalid`,
`broadcast.not_multicast`, `broadcast.join_failed`,
`broadcast.reply_missing`, `node.osc_address`, `hex.*`, Bind-Fehler. Der
Job endet mit `wait.receive_failed`, wenn der Socket nicht mehr empfangen kann.

```bash
invoke broadcast_send '{"config":{"mode":"list","target":"127.0.0.1:9000, 127.0.0.1:9001","payload":{"kind":"text","text":"PING"}}}' \
  | jq '{packets, errors}'
```

## Störung {#impairment}

Ein Relay zwischen einem Client und seinem Server, das verzögert, verwirft, dupliziert,
verfälscht, umordnet oder drosselt, was hindurchgeht, über UDP oder TCP. Siehe
[Störung](../tools/impairment.md).

### netsim_start {#netsim_start}

Startet ein Relay: Was auf `listen` eintrifft, geht weiter an `target`, und die Antworten
kommen denselben Weg zurück, beide vom Profil beeinträchtigt. Seine Zähler treffen als
[`netsim://stat`](events.md#event-netsim-stat) alle 250 ms ein. *Startet einen Job*
(`netsim`, `params.listen`, `params.target` und `params.protocol` für TCP).

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `config` | object | ja | Unten |

| Feld von `config` | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `listen` | string | — | `IP:port`, worauf das Relay hört; richten Sie den Client hierhin |
| `target` | string | — | `IP:port` des echten Servers oder `host:port` — ein Hostname wird einmal beim Start des Relays aufgelöst |
| `profile` | [`ImpairProfile`](#type-impairprofile) | — | Was dem Verkehr angetan wird |
| `seed` | number | neu | Der Startwert der Ziehungen: derselbe Startwert und derselbe Verkehr geben dieselben Verluste |
| `protocol` | string | `udp` | `udp` (Datagramme) oder `tcp` (Streams) |

**Ergebnis**: [`JobInfo`](#type-jobinfo).

**Fehler**: `node.range` (ein Wert des Profils außerhalb seines Bereichs, mit
`min`, `max` und dem Feld), `node.too_long`, `node.bind_invalid`,
`transport.target_invalid`, `transport.dns` (ein Zielname, der sich nicht
finden lässt), Bind-Fehler. Der Job endet mit
`wait.receive_failed`, wenn ein Socket nicht mehr empfangen kann.

### netsim_set_profile {#netsim_set_profile}

Ein laufendes Relay beeinträchtigt von nun an mit einem anderen Profil, ohne seine
Sockets zu schließen.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `jobId` | number | ja | Der Job des Relays |
| `profile` | [`ImpairProfile`](#type-impairprofile) | ja | Das neue Profil |

**Ergebnis**: null.

**Fehler**: `netsim.not_running`, `node.range`, `node.too_long`.

```bash
invoke netsim_start '{"config":{"listen":"127.0.0.1:9010","target":"127.0.0.1:9000","profile":{"latency_ms":80,"jitter_ms":20,"loss":0.02}}}'
```

## Storm und Scanner {#storm-scanner}

::: danger
Ein Storm belastet ein Ziel so stark, wie Sie verlangen, und ein Scan sondiert jeden Port eines
Bereichs. Richten Sie beide nur auf Hosts, für die Sie verantwortlich sind.
:::

### storm_start {#storm_start}

Sendet eine stetige Last aus UDP-Datagrammen oder TCP-Verbindungen an ein Ziel. Seine
Zähler treffen als [`storm://stat`](events.md#event-storm-stat) alle 250 ms ein.
*Startet einen Job* (`storm`, `params.protocol`, `params.target`, `params.rate`).

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `config` | object | ja | Unten |

| Feld von `config` | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `target` | string | — | `IP:port` oder `host:port`; ein Name wird einmal beim Start des Jobs aufgelöst |
| `protocol` | string | — | `udp`: Datagramme; `tcp`: eine Verbindung pro Einheit, die den Payload schreibt und schließt (jedes Verbinden kann 500 ms dauern) |
| `size` | number | — | Payload-Bytes, zwischen 1 und 65 507 gehalten |
| `rate` | number | — | Einheiten pro Sekunde, nach einem Zeitplan: Einheit *n* ist *n* / `rate` Sekunden nach dem Start fällig, und jedes Aufwachen sendet, was fällig ist (höchstens 256; ein weiter zurückliegender Zeitplan überspringt die älteren Einheiten); 0 sendet so schnell es kann |
| `duration_s` | number | `0` | Nach so vielen Sekunden stoppen; 0: bis zum Stoppen |

**Ergebnis**: [`JobInfo`](#type-jobinfo).

**Fehler**: `transport.target_invalid`, `transport.dns`. Gescheiterte Sendevorgänge werden
in den Ereignissen gezählt, nicht als Fehler gemeldet.

### scan_start {#scan_start}

Versucht eine TCP-Verbindung zu jedem Port eines Bereichs und meldet die offenen,
mit dem, was der Dienst zuerst sagt, wenn danach gefragt wird. Offene Ports treffen als
[`scan://open`](events.md#event-scan-open) ein, Fortschritt als
[`scan://progress`](events.md#event-scan-progress). *Startet einen Job* (`scan`,
`params.host`, `params.from`, `params.to`).

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `config` | object | ja | Unten |

| Feld von `config` | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `host` | string | — | Ein Hostname oder eine Adresse |
| `port_start`, `port_end` | number | — | Der Bereich, beide eingeschlossen; falsch herum angegeben werden sie vertauscht |
| `concurrency` | number | `256` | Versuche gleichzeitig, 1 bis 1024 |
| `timeout_ms` | number | `600` | Pro Port, 50 bis 10 000 |
| `grab_banner` | boolean | `false` | Bis zu 256 Bytes lesen, die der Dienst innerhalb von 400 ms nach dem Verbinden sendet |

**Ergebnis**: [`JobInfo`](#type-jobinfo).

**Fehler**: `scan.host_required`.

```bash
invoke scan_start '{"config":{"host":"127.0.0.1","port_start":8000,"port_end":9100,"grab_banner":true}}'
```

## Inspektor {#inspector}

Der Inspektor zeichnet auf, was die Werkzeuge senden und empfangen, als Frames, solange
der Mitschnitt läuft. Auf einem Server gibt es einen Inspektor für jede Seite und
jedes Skript. Siehe [den Inspektor](../tools/inspector.md).

### inspect_set_enabled {#inspect_set_enabled}

Schaltet den Mitschnitt scharf oder aus. Ausgeschaltet wird nichts aufgezeichnet.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `enabled` | boolean | ja | Scharfschalten (`true`) oder Ausschalten |

**Ergebnis**: [`CaptureStats`](#type-frame).

### inspect_stats {#inspect_stats}

Die Zähler des Mitschnitts. Keine Argumente.

**Ergebnis**: [`CaptureStats`](#type-frame).

### inspect_snapshot {#inspect_snapshot}

Die neuesten Frames, älteste zuerst.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `limit` | number | ja | Wie viele, 1 bis 8192 |

**Ergebnis**: [`Frame`](#type-frame)`[]`, ohne ihre Bytes (siehe
[`inspect_payload`](#inspect_payload)).

### inspect_clear {#inspect_clear}

Leert den Mitschnitt und seine Zähler. Keine Argumente.

**Ergebnis**: [`CaptureStats`](#type-frame).

### inspect_export {#inspect_export}

Schreibt jeden gehaltenen Frame nach `capture-<ms>.jsonl` oder `capture-<ms>.txt` im
Datenordner. In `jsonl` ist jede Zeile ein Frame mit den Bytes, die er in
`data` behält, base64; `txt` ist zum Lesen, mit einem Hex-Dump jedes Frames.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `format` | string | ja | `txt`; alles andere schreibt `jsonl` |

**Ergebnis**: string, der geschriebene Pfad.

**Fehler**: `inspect.empty`, `file.io`.

### inspect_payload {#inspect_payload}

Die Bytes, die ein Frame behält, jenseits der 1-KiB-Vorschau, die sein Batch trug.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `seq` | number | ja | Die Nummer des Frames |

**Ergebnis**: `{ "seq", "bytes", "kept", "dump", "hex" }` — `bytes` die Größe
des Frames, `kept` wie viele davon behalten werden (bis zu 256 KiB), `dump` jede Zeile als
`offset  hex  |ascii|`, `hex` das reine Hex, das ein Replay sendet.

**Fehler**: `inspect.frame_gone` (neuere Frames haben seinen Platz genommen),
`inspect.no_payload` (nur seine Größe wurde aufgezeichnet).

```bash
invoke inspect_set_enabled '{"enabled":true}'
invoke inspect_snapshot '{"limit":20}' | jq '.[] | {seq, proto, dir, summary}'
```

## Signalbibliothek {#signals}

Die Bibliothek ist `signals.json` im Datenordner. Sie ist nur Speicher: Ein Signal
wird mit dem Befehl seines Transports gesendet (`osc_send`, `broadcast_send`,
`http_request`, `mqtt_publish` oder `mqtt_publish_once`). Siehe
[Signale](../tools/signals.md) und [Dateien](../reference/files.md#signals-json).

### signals_load {#signals_load}

Liest die Bibliothek. Existiert die Datei nicht, wird zuerst das Starterset
geschrieben. Keine Argumente.

**Ergebnis**: `{ "path": string, "library": library, "seeded": boolean }` —
`seeded` ist true, wenn gerade das Starterset geschrieben wurde. Die Bibliothek ist
`{ "version", "signals": [...], "folders": [...] }`: `version` 2 (eine Version-1-
Datei kommt so zurück, wie sie ist), `folders` bleibt weg, wenn es keine gibt. Jedes Signal
hat `id`, `name`, `group` (seinen Ordner, `"A/B"`; leer für keinen), `note` und
`body`.

**Fehler**: `signals.json_invalid` (mit `path`, `line`, `column`; die Datei wird
nie ersetzt), `file.io`.

### signals_save {#signals_save}

Ersetzt die ganze Bibliotheksdatei, über eine temporäre Datei im selben Ordner.
Eine Datei, die existiert und sich nicht als Bibliothek lesen lässt, bleibt, wie sie ist.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `library` | object | ja | `{ version, signals, folders }`, wie `signals_load` es zurückgibt |

**Ergebnis**: string, der geschriebene Pfad.

**Fehler**: `signals.json_invalid` (die nun auf der Platte liegende Datei liest sich nicht, mit
`path`, `line`, `column`; es wird nichts geschrieben), `signals.encode`, `file.io`.

Der `body` eines Signals nach seinem `transport`:

| `transport` | Felder |
| --- | --- |
| `osc` | `target`, `address`, `args` ([`OscArg`](#type-oscarg)`[]`) |
| `udp` | `target`, `payload`: `{ "kind": "text", "text" }` oder `{ "kind": "hex", "hex" }` |
| `http` | `request` ([`HttpRequest`](#type-httprequest)) |
| `mqtt` | `broker` (`host:port`), `topic`, `payload`, `qos`, `retain` |

```bash
invoke signals_load | jq '.library.signals[] | {name, transport: .body.transport}'
```

## Emulatoren {#emulators}

Ein Emulator ist Signal Lab in der Rolle der Gegenseite: eine HTTP-API, ein OSC-, UDP- oder
TCP-Gerät, ein MQTT-Broker. Sein Dokument — `name`, `bind`, `protocol`, die
Regeln des Protokolls und ein optionaler `outage` — ist in
[Emulatoren](../tools/emulators.md) beschrieben. Die Bibliothek ist `emulators.json` im
Datenordner.

### emulators_load {#emulators_load}

Liest die Emulatorbibliothek. Existiert die Datei nicht, wird das Starterset
zuerst geschrieben. Keine Argumente.

**Ergebnis**: `{ "path", "library": { "version": 1, "emulators": [{ "id", "note", "emulator" }] }, "seeded" }`.

**Fehler**: `emulators.json_invalid` (mit `path`, `line`, `column`; nie
ersetzt), `file.io`.

### emulators_save {#emulators_save}

Ersetzt die ganze Emulatorbibliothek, über eine temporäre Datei.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `library` | object | ja | `{ version, emulators }`, wie `emulators_load` es zurückgibt |

**Ergebnis**: string, der geschriebene Pfad.

**Fehler**: `emulators.encode`, `file.io`.

### emulator_check {#emulator_check}

Ob ein Emulator starten würde: alles, was [`emulator_start`](#emulator_start)
prüft, bevor es bindet.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `emulator` | object | ja | Das Emulatordokument |
| `params` | object of strings | nein | Werte, die seine Vorlagen als Parameter lesen |

**Ergebnis**: null, wenn er starten würde.

**Fehler**: `emulator.*`; `node.*` für ein fehlendes, außerhalb des Bereichs, zu langes
oder fehlerhaftes Feld (`node.required`, `node.range`, `node.too_long`,
`node.bind_invalid`, `node.target_invalid`, `node.method_invalid`…);
`param.unknown`, `template.*`, `osc.pattern_*`, `regex.invalid`,
`hex.invalid`. Jeder hat `rule`, `retained` oder `response` in `params`, wenn das
Problem in einem davon liegt.

### emulator_start {#emulator_start}

Startet einen Emulator als eigenen Job. Sein Socket ist offen, wenn der Befehl
zurückkehrt. Was er empfängt und beantwortet, trifft als
[`emulator://activity`](events.md#event-emulator-activity) alle 200 ms ein, wenn
sich etwas geändert hat. *Startet einen Job* (`emulator`, `params.name`,
`params.protocol`, `params.local` und `params.source`, wenn `source`
gegeben ist).

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `emulator` | object | ja | Das Emulatordokument |
| `params` | object of strings | nein | Werte, die seine Vorlagen als Parameter lesen |
| `seed` | number | nein | Sein Startwert, 0 bis 9007199254740991; Standard: ein neuer |
| `source` | string | nein | Der Bibliothekseintrag, aus dem er stammt, als `params.source` am Job behalten |

**Ergebnis**: [`JobInfo`](#type-jobinfo).

**Fehler**: die von [`emulator_check`](#emulator_check), `seed.range`,
`transport.address_in_use` und die anderen Bind-Fehler.

### emulator_exchanges {#emulator_exchanges}

Was ein laufender Emulator empfangen und beantwortet hat. Er behält die letzten 500
Austausche.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `jobId` | number | ja | Der Job des Emulators |
| `after` | number | nein | Nur Austausche mit einer Nummer über dieser; Standard 0 |
| `limit` | number | nein | Höchstens so viele, 1 bis 500; Standard 500 |

**Ergebnis**

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `job_id` | number | Der Job |
| `name`, `protocol`, `local` | string | Der Emulator, sein Protokoll und die Adresse, auf der er hört |
| `counts` | object | `total`, `unmatched` (keine Regel nahm ihn), `failed`, `down` (traf ein, während er herunter war: sein Outage oder [`emulator_down`](#emulator_down)), `hits` (pro Regel) und `missed` (MQTT: Nachrichten, die ein zu weit zurückliegender Client nicht bekam; bleibt weg, solange 0) |
| `forced` | string | `unavailable`, `reset` oder `timeout`, solange er heruntergenommen ist; sonst bleibt es weg |
| `exchanges` | object[] | Jeder: `seq`, `ts`, `from`, `request`, `rule` (1-basiert; bleibt weg, wenn keine ihn nahm), `reply`, `status`, `fault`, `ms`, `error`, `frame`, `down` und `data` (die Anfrage, wie Vorlagen sie lesen) |

**Fehler**: `emulator.not_running`.

### emulator_down {#emulator_down}

Nimmt einen laufenden Emulator herunter, bis er wieder hochgebracht wird, was immer sein Outage-
Zeitplan sagt, oder bringt ihn zurück. Während er unten ist, begegnet ein HTTP-Emulator jeder
Anfrage mit `fault`, lassen ein TCP-Gerät und ein MQTT-Broker ihre Verbindungen fallen
und lehnen neue ab, und OSC- und UDP-Geräte antworten nichts.

| Argument | Typ | Erforderlich | Bedeutung |
| --- | --- | --- | --- |
| `jobId` | number | ja | Der Job des Emulators |
| `down` | boolean | ja | Herunter (`true`) oder hoch |
| `fault` | string | nein | Worauf HTTP-Anfragen treffen: `unavailable` (503, ohne `Retry-After`: wann er zurückkommt, ist nicht bekannt), `reset` (die Verbindung schließt), `timeout` (keine Antwort); Standard `unavailable` |

**Ergebnis**: null.

**Fehler**: `emulator.not_running`.

```bash
invoke emulator_exchanges '{"jobId":5,"after":0}' | jq '.counts, (.exchanges[] | {request, rule, status})'
```

## Gemeinsame Typen {#types}

### JobInfo {#type-jobinfo}

Was ein Befehl, der einen Job startet, zurückgibt, und was [`jobs_list`](#jobs_list)
auflistet: `id`, `kind`, `label` (Englisch, für Protokolle), `params` (die Werte, die
das Label nennt; bleibt weg, wenn es keine gibt) und `started_ms`. Siehe
[Jobs](index.md#jobs).

### OscArg {#type-oscarg}

Ein OSC-Argument, sein Typ und sein Wert:

| `type` | `value` | OSC-Tag |
| --- | --- | --- |
| `int` | 32-Bit-Ganzzahl | `i` |
| `float` | Zahl, als 32-Bit-Float gesendet | `f` |
| `str` | string | `s` |
| `long` | 64-Bit-Ganzzahl | `h` |
| `double` | Zahl, 64-Bit | `d` |
| `bool` | `true` oder `false` | `T` oder `F` |
| `blob` | Byte-Array, `[222, 173]` | `b` |
| `nil` | keins: `{ "type": "nil" }` | `N` |

### HttpRequest {#type-httprequest}

| Feld | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `method` | string | — | `GET`, `POST`… |
| `url` | string | — | `http://` oder `https://` |
| `headers` | `[name, value][]` | `[]` | Anfrage-Header |
| `body` | string oder null | null | Der Body |
| `timeout_ms` | number | `10000` | Für den ganzen Austausch |
| `auth` | object | keins | `{ "scheme": "basic", "username", "password" }`, `{ "scheme": "digest", "username", "password" }` oder `{ "scheme": "bearer", "token" }` |

Bis zu 10 Weiterleitungen werden verfolgt. Anmeldedaten und Cookies, die für einen Host
eingegeben wurden, gehen nie an einen anderen. Eine Digest-Anfrage beantwortet die 401-Challenge des Servers
und sendet erneut.

### HttpResponse {#type-httpresponse}

| Feld | Typ | Bedeutung |
| --- | --- | --- |
| `ok` | boolean | Ein 2xx-Status |
| `status`, `status_text` | number, string | Der Status; 0 und leer ohne Antwort |
| `latency_ms` | number | Bis der ganze Body da war |
| `headers` | `[name, value][]` | Antwort-Header |
| `body` | string | Der Body als Text, höchstens 256 KiB |
| `body_bytes` | number | Die volle Größe des Bodys |
| `truncated` | boolean | `body` wurde bei 256 KiB abgeschnitten |
| `error` | string oder null | Warum es keine Antwort gab, jede Ebene der Ursache |
| `cause` | string oder null | Welche Art von Fehler: `refused`, `timeout`, `dns`, `unreachable`, `reset`, `address_in_use`, `address_unavailable`, `denied`, `tls`, `target_invalid`, `failed` — dasselbe wie die `transport.*`-Codes |
| `digest` | object | Nur eine Digest-Anfrage, die auf eine 401 traf; sonst bleibt es weg. `challenged`: die Challenge wurde beantwortet und die Anfrage erneut gesendet. `error`: warum es nicht ging, ein `EngineError` (`http.digest_not_offered`, `http.digest_unsupported`, `http.digest_invalid`, `http.digest_other_origin`) oder null |

### Payload {#type-payload}

Was ein Broadcast- oder Discovery-Datagramm trägt:
`{ "kind": "osc", "address", "args" }`, `{ "kind": "text", "text" }` (so gesendet,
wie es ist, kein abschließendes Null) oder `{ "kind": "hex", "hex" }` (`de ad be ef`,
`deadbeef`, `0xDE,0xAD` — alles außer Hex-Ziffern wird ignoriert).

### MqttConfig {#type-mqttconfig}

| Feld | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `host` | string | — | Name oder Adresse des Brokers |
| `port` | number | — | Üblicherweise 1883 |
| `client_id` | string | — | Nicht leer; eine andere Verbindung mit derselben id wird vom Broker verdrängt |
| `username`, `password` | string | leer | `username` wird gesendet, wenn nicht leer; `password` nur zusammen mit einem `username` |
| `keep_alive_s` | number | `60` | Pings gehen bei der Hälfte davon; 0: keine |
| `clean_session` | boolean | `true` | Das CONNECT-Flag |
| `will` | object oder null | null | `{ topic, payload, qos, retain }`, vom Broker veröffentlicht, wenn die Verbindung verloren geht |
| `subscribe` | `{ filter, qos }[]` | `[]` | Abonniert, sobald die Verbindung steht |

### WsConfig {#type-wsconfig}

| Feld | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `url` | string | — | `ws://` oder `wss://` (`wss://` vertraut, was das System für HTTPS vertraut) |
| `headers` | `[name, value][]` | `[]` | Mit der Upgrade-Anfrage gesendet |
| `protocols` | string[] | `[]` | Subprotokolle zum Anbieten, in Reihenfolge der Präferenz |
| `timeout_ms` | number | `10000` | Für die Verbindung, TLS und das Upgrade zusammen |

Nachrichten sind in beide Richtungen höchstens 16 MiB.

### ImpairProfile {#type-impairprofile}

Jedes Feld ist optional; was weggelassen wird, tut nichts. Wahrscheinlichkeiten sind 0
bis 1.

| Feld | Bereich | Bedeutung | UDP | TCP |
| --- | --- | --- | --- | --- |
| `name` | höchstens 60 Zeichen | Ein Label für die Zeitleiste und den Bericht | ja | ja |
| `latency_ms` | 0 bis 60 000 | Zu allem hinzugefügte Verzögerung | ja | ja |
| `jitter_ms` | 0 bis 60 000 | Bis zu so viel mehr, jedes Mal neu gezogen | ja | ja |
| `loss` | 0 bis 1 | Ein Datagramm wird verworfen | ja | — |
| `duplicate` | 0 bis 1 | Ein Datagramm wird zweimal gesendet | ja | — |
| `corrupt` | 0 bis 1 | Ein Bit eines Datagramms wird gekippt | ja | — |
| `reorder` | 0 bis 1 | Ein Datagramm wird zurückgehalten, sodass spätere es überholen | ja | — |
| `rate_kbps` | 0 oder 8 bis 10 000 000 | Bandbreitenlimit, Kilobit pro Sekunde; 0: keins | ja | ja |
| `burst_start` | 0 bis 1 | Ein Datagramm beginnt einen Verlustburst | ja | — |
| `burst_length` | 1 bis 1000 | Datagramme, die ein Burst im Mittel dauert (nötig mit `burst_start`) | ja | — |
| `offline` | `true` oder `false` | Nichts kommt durch | ja | ja |
| `reset` | 0 bis 1 | Ein Chunk eines Streams setzt seine Verbindung zurück | — | ja |
| `stall` | 0 bis 1 | Ein Chunk eines Streams lässt seine Verbindung halb-offen | — | ja |

### Frame und CaptureStats {#type-frame}

Ein `Frame` ist ein aufgezeichnetes Paket, eine Anfrage oder Nachricht:

| Feld | Bedeutung |
| --- | --- |
| `seq` | Seine Nummer, steigend |
| `ts` | Wann, Millisekunden seit 1970 |
| `proto` | `osc`, `udp`, `tcp`, `http`, `mqtt`, `ws`… |
| `dir` | `tx` (gesendet) oder `rx` (empfangen) |
| `source` | Das Werkzeug, das ihn aufgezeichnet hat: `osc-monitor`, `broadcast`, `netsim`… |
| `job_id` | Sein Job oder null |
| `local`, `remote` | Die Adressen: `IP:port` dieser Seite und der anderen (eine URL oder ein Broker für HTTP, WebSocket und MQTT). Das `local` eines weitergeleiteten Frames ist die Adresse, auf der das Relay hört, und sein `remote`, wohin der Frame ging; das Leg beendet sein `verdict` (`· client→target`, `· target→client`) |
| `bytes` | Seine Größe |
| `summary` | Eine Zeile |
| `detail` | Eine Dekodierung über mehrere Zeilen oder null |
| `hex` | Ein Hex-Dump der ersten 1 KiB oder null |
| `verdict` | Was aus ihm wurde — `dropped`, `sampled`, ein Status — oder null |
| `kept` | Von `bytes`, wie viele behalten werden (bis zu 256 KiB); 0, wenn nur die Größe aufgezeichnet wurde |
| `publish` | Nur eine MQTT-Veröffentlichung: `{ broker, topic, qos, retain, text }` — der Broker als `host:port`, und ob die behaltenen Bytes, die der Payload der Nachricht sind, UTF-8-Text sind. Für jeden anderen Frame fehlt es |

`CaptureStats`: `enabled`, `total` (aufgezeichnete Frames), `bytes`, `skipped`
(aufgezeichnet, aber nie an die Oberfläche gesendet), `buffered` (gehaltene Frames),
`capacity` (8192), `held` (gehaltene Payload-Bytes) und `held_limit` (64 MiB). Die
ältesten Frames weichen jenseits einer der beiden Grenzen.

---
title: HTTP
description: Eine HTTP-Anfrage senden und die ganze Antwort lesen, sich mit Basic, Bearer oder Digest authentifizieren, Cookies behalten und einen Endpunkt unter einem gleichzeitigen Last-Burst messen.
---

# HTTP

Die Ansicht [[ui:nav.http]] ist ein Anfrageinspektor und ein Lastwerkzeug in
einem:

- eine einzelne Anfrage senden und Status, Dauer, Header und Body sehen;
- sich mit Basic, einem Bearer-Token oder Digest authentifizieren;
- die Cookies behalten, die ein Server setzt, wie es ein Browser tut;
- dieselbe Anfrage viele Male auf einmal senden — ein [[ui:http.burst]] — und
  den Durchsatz und die Latenz-Perzentile lesen.

## Eine Anfrage senden {#request}

1. Öffnen Sie [[ui:nav.http]].
2. Wählen Sie die Methode und geben Sie die URL ein, zum Beispiel
   `http://127.0.0.1:8080/health`.
3. Fügen Sie [[ui:http.headers]] hinzu, wenn der Server sie braucht;
   [[ui:http.addHeader]] fügt eine Zeile hinzu, ✕ entfernt eine. Eine Zeile ohne
   Namen wird nicht gesendet.
4. Schreiben Sie für eine andere Methode als GET und HEAD den [[ui:http.body]].
   Der Text bleibt im Feld, während Sie zu GET oder HEAD wechseln, und kommt mit
   der anderen Methode zurück, wird aber in der Zwischenzeit nicht gesendet.
5. Drücken Sie [[ui:common.send]].

Die Zeile unter den Schaltflächen gibt sofort das Urteil — Status, Dauer und
Größe oder warum es keine Antwort gab —, und der Bereich [[ui:http.response]]
zeigt den Rest. Die Anfrage (Methode, URL, Header, Body, Timeout und
[[ui:http.keepCookies]]) bleibt erhalten, wenn Sie die Ansicht wechseln und wenn
Sie die App neu starten; Anmeldedaten nicht.

| Taste | Wo | Wirkt |
| --- | --- | --- |
| <kbd>Enter</kbd> | die URL, ein Header, eine Anmeldung, der Timeout | sendet |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | jedes Feld der Anfrage, den Body eingeschlossen | sendet |
| <kbd>Ctrl</kbd>+<kbd>S</kbd> | jedes Feld der Anfrage | speichert sie als Signal ([unten](#library)) |

### Felder der Anfrage {#request-fields}

| Feld | Was | Standard |
| --- | --- | --- |
| [[ui:exp.method]] | GET, POST, PUT, PATCH, DELETE, HEAD oder OPTIONS | GET |
| [[ui:field.url]] | Eine `http://`- oder `https://`-URL | `http://127.0.0.1:8080/` |
| [[ui:http.headers]] | Name-Wert-Paare, so gesendet, wie geschrieben. Ohne einen eigenen `User-Agent` sendet Signal Lab `SignalLab/0.1`. | `Accept: application/json` |
| [[ui:field.auth]] | Wie sich die Anfrage authentifiziert ([unten](#auth)) | [[ui:http.auth.none]] |
| [[ui:http.keepCookies]] | Die Cookies zurücksenden, die Server setzen ([unten](#cookies)) | an |
| [[ui:http.body]] | Genau so gesendet, wie geschrieben; kein `Content-Type` wird hinzugefügt, fügen Sie also den passenden Header hinzu. Ein leerer Body wird nicht gesendet. Das Feld wird für GET und HEAD nicht angezeigt, und dann trägt die Anfrage überhaupt keinen Body — nicht im Senden, nicht in einem gespeicherten Signal, nicht in [[ui:common.toExperiment]] —, auch wenn Sie unter einer anderen Methode einen eingegeben haben. | leer |
| [[ui:common.timeoutMs]] | Wie lange der ganze Austausch dauern darf, Antwort und Body eingeschlossen | 10.000 |

## Authentifizierung {#auth}

| [[ui:field.auth]] | Felder | Was gesendet wird |
| --- | --- | --- |
| [[ui:http.auth.none]] | — | kein `Authorization`-Header |
| [[ui:http.auth.basic]] | [[ui:field.username]], [[ui:field.password]] | `Authorization: Basic …`, der Name und das Passwort in base64, mit der ersten Anfrage |
| [[ui:http.auth.bearer]] | [[ui:field.token]] | `Authorization: Bearer <token>` |
| [[ui:http.auth.digest]] | [[ui:field.username]], [[ui:field.password]] | anfangs nichts; die Antwort auf die Challenge des Servers ([unten](#digest)) |

Ein Wechsel zwischen Basic und Digest behält Name und Passwort.

### Digest {#digest}

Mit Digest sendet Signal Lab die Anfrage ohne Anmeldedaten. Antwortet der Server
mit `401` und einer Digest-Challenge, berechnet Signal Lab die Antwort aus der
Challenge und Ihrem Passwort und sendet die Anfrage erneut. Die Antwort, die Sie
sehen, ist die auf diese zweite Anfrage, markiert mit
[[ui:http.digestAnswered]], und die Latenz zählt beide Austausche — was ein
Client wartet.

- Algorithmen: MD5 und SHA-256 sowie ihre `-sess`-Varianten. Bietet ein Server
  beide an, wird SHA-256 verwendet.
- Quality of Protection: `auth` und `auth-int` sowie die ältere Antwort ohne
  `qop`.
- Sagt der Server, die Nonce sei abgelaufen (`stale`), oder fragt er mit einer
  neuen erneut, wird die Anfrage erneut beantwortet, bis zu 3 weitere Male.
  Weist er die Antwort auf die zuletzt gegebene Nonce ab, bleibt der `401`
  bestehen: Name oder Passwort ist falsch.
- Weiterleitungen folgt Signal Lab selbst, sodass die URL, die fragt, auch
  beantwortet wird. Eine Challenge von einem anderen Origin wird nicht
  beantwortet: Anmeldedaten, die für einen Host eingegeben wurden, gehen an
  keinen anderen. Der Wechsel von `http://` zu `https://` auf demselben Host und
  dessen Standardports gilt als derselbe Host.

Kann die Challenge nicht beantwortet werden, bleibt der `401` bestehen, und der
Bereich sagt, warum:

| Meldung | Bedeutung |
| --- | --- |
| The server answered 401 without asking for Digest | Der Server möchte ein anderes Verfahren; versuchen Sie Basic oder Bearer. |
| The server asked for Digest with … | Einen Algorithmus, den Signal Lab nicht beherrscht; es beherrscht MD5 und SHA-256. |
| The server asked for Digest without a realm or nonce | Die Challenge des Servers ist unvollständig. |
| The request was sent on to …, which asked for Digest | Eine Weiterleitung führte zu einem anderen Origin, dessen Challenge nicht beantwortet wird. |

### Wohin die Anmeldedaten gehen {#credentials}

Anmeldedaten gehen nur in den `Authorization`-Header der Anfrage, während sie
gesendet wird. Der Inspektor, die Konsole und die Berichte der Experimente
zeigen diesen Header nie. In dieser Ansicht werden sie nur im Speicher gehalten
und sind nach einem Neustart weg — es sei denn, die Anfrage ist an ein
gespeichertes Signal gebunden, das sie zurückbringt.

::: warning
Eine als Signal gespeicherte Anfrage behält ihre Anmeldedaten in der
Bibliotheksdatei `signals.json` als Klartext. Schreiben Sie in einem Experiment
stattdessen ein Passwort als `{{secret.NAME}}`; siehe
[Daten und Vorlagen](../experiments/data.md).
:::

## Cookies {#cookies}

Ist [[ui:http.keepCookies]] an, wird, was ein Server mit `Set-Cookie` setzt, im
Cookie-Speicher der Ansicht behalten und mit späteren Anfragen an diesen Server
zurückgesendet, nach den Regeln des Browsers (Domain, Pfad, `Secure`, Ablauf).
Den Speicher nutzen die Anfragen dieser Ansicht, ihr Burst und die HTTP-Signale,
die Sie aus der Bibliothek senden. Schalten Sie ihn aus, um Anfragen ohne
Cookies zu senden und keine zu behalten.

Der Cookie-Bereich unter der Anfrage und der Antwort listet auf, was der
Speicher hält: [[ui:http.cookieName]], [[ui:http.cookieValue]],
[[ui:http.cookieWhere]] (eine Domain, die mit `.` beginnt, umfasst auch ihre
Subdomains), [[ui:http.cookieExpires]] ([[ui:http.cookieSession]] für ein Cookie
ohne Ablauf) und [[ui:http.cookieFlags]] (`Secure`, `HttpOnly`, `SameSite`).
Abgelaufene Cookies werden nicht aufgelistet. [[ui:common.clear]] leert den
Speicher.

Der Speicher lebt so lange wie die App: Ein Neustart beginnt mit einem leeren.
Auf einem [Server](../server/index.md) gibt es einen Speicher für jede an ihm
angemeldete Seite. Ein Durchlauf eines Experiments hat einen eigenen Speicher
(siehe [Experimente](../experiments/index.md)), und `signallab send http`
verwendet keinen.

## Die Antwort {#response}

| Teil | Was |
| --- | --- |
| [[ui:http.status]] | Der Statuscode und sein Grund; `ERR`, wenn keine Antwort kam |
| [[ui:http.latency]] | Vom Senden bis zum letzten Byte des Bodys, in Millisekunden |
| [[ui:http.size]] | Die Größe des Bodys |
| Antwort-Header | Klicken Sie auf die Zeile mit ihrer Anzahl, um sie ein- oder auszublenden |
| Body | Formatiert, wenn er JSON ist; [[ui:http.rawBody]] und [[ui:http.formatJson]] schalten um. Bis 256 KiB werden gezeigt, danach `… (truncated)`. |

Gibt es keine Antwort, sagt der Bereich, warum, mit denselben Worten wie überall
in Signal Lab: abgelehnt, keine rechtzeitige Antwort, der Name lässt sich nicht
auflösen, ein Zertifikatsproblem und so weiter. Das technische Detail des
Systems ist darunter eingeklappt.

### Weiterleitungen {#redirects}

Weiterleitungen (301, 302, 303, 307, 308) werden verfolgt, bis zu 10; die
gezeigte Antwort ist die letzte. Nach 301, 302 und 303 geht die Anfrage als GET
ohne Body weiter (HEAD bleibt HEAD); nach 307 und 308 so, wie sie war.
`Authorization` und Cookies, die für einen Host eingegeben wurden, werden nicht
an einen anderen gesendet.

### Sichere Verbindungen {#tls}

Das Zertifikat eines `https://`-Servers wird gegen die Zertifikate geprüft, denen
dieses System vertraut. Ein selbst ausgestelltes oder abgelaufenes Zertifikat
wird mit "A secure connection to … could not be made" abgelehnt; es gibt keine
Einstellung, die Prüfung zu überspringen. Um einen Server mit Ihrem eigenen
Zertifikat zu testen, fügen Sie es den vertrauenswürdigen Zertifikaten des
Systems hinzu.

## Last-Burst {#burst}

Der [[ui:http.burst]] sendet die Anfrage auf dem Bildschirm — mit ihrer
Authentifizierung und, solange [[ui:http.keepCookies]] an ist, dem
Cookie-Speicher — viele Male und misst sie.

1. Setzen Sie [[ui:common.concurrency]], [[ui:http.total]],
   [[ui:http.duration]] und [[ui:http.rate]].
2. Drücken Sie [[ui:http.startBurst]]. Der Burst ist ein Job:
   [[ui:http.stopBurst]] oder ein Stoppen in der Leiste der Konsole beendet ihn.

| Feld | Was | Standard |
| --- | --- | --- |
| [[ui:common.concurrency]] | Anfragen gleichzeitig unterwegs, 1–512 | 20 |
| [[ui:http.total]] | Zu sendende Anfragen; 0 — weiter senden, bis die Dauer endet | 500 |
| [[ui:http.duration]] | Sekunden, die gelaufen wird; 0 — stoppen, wenn die Gesamtzahl gesendet ist | 0 |
| [[ui:http.rate]] | Pro Sekunde gestartete Anfragen, 0,1–100.000; 0 — so schnell, wie die Worker kommen | 0 |

Sind [[ui:http.total]] und [[ui:http.duration]] beide 0, läuft der Burst, bis Sie
ihn stoppen.

Es gibt zwei Arten zu senden:

- **[[ui:http.rate]] 0.** Jeder der Worker sendet erneut, sobald er eine Antwort
  hat. Das findet heraus, wie viel der Server annimmt, aber ein langsamer Server
  bremst auch den Burst.
- **Eine Rate.** Die Anfragen starten nach einem festen Zeitplan — bei 10 pro
  Sekunde eine alle 100 ms ab dem Start —, wie langsam die Antworten auch sind.
  Eine Anfrage, deren Moment kommt, während jeder Worker beschäftigt ist, wartet
  höchstens 50 ms auf einen; danach wird sie übersprungen und als
  [[ui:http.missed]] gezählt, nie verspätet gesendet. Verpasste Anfragen
  bedeuten, dass die Parallelität für diese Rate zu niedrig ist oder der Server
  langsamer ist, als die Rate benötigt.

| Zahl | Was |
| --- | --- |
| [[ui:http.sent]] | Anfragen, die eine Antwort hatten oder fehlschlugen |
| [[ui:http.ok]] | Mit einem 2xx-Status beantwortet |
| [[ui:http.failed]] | Keine Antwort oder jeder Status außerhalb von 200–299 |
| [[ui:http.missed]] | Übersprungen, wie oben (nur mit einer Rate) |
| [[ui:http.rps]] | Anfragen pro Sekunde über die letzten Zehntelsekunden; ist der Burst beendet, über den ganzen Burst. Mit einer Rate nennt die Beschriftung die geforderte Rate. |
| [[ui:http.p50]], [[ui:http.p90]], [[ui:http.p95]], [[ui:http.p99]] | Die Zeit, innerhalb derer dieser Anteil der Anfragen fertig wurde, Fehlschläge eingeschlossen; auf 0,5 % genau |
| [[ui:http.avg]], [[ui:http.min]], [[ui:http.max]] | Der Mittelwert, der schnellste und der langsamste |

Die Zahlen werden etwa 10-mal pro Sekunde aktualisiert. Das Diagramm daneben
zeichnet die Anfragen pro Sekunde über die letzten etwa 24 Sekunden.

Mit Digest wird die Challenge der ersten Anfrage einmal beantwortet, und diese
Antwort dient jeder Anfrage des Bursts.

::: warning
Ein Burst ist echte Last. Richten Sie ihn nur auf Server, die Ihnen gehören oder
deren Test Ihnen erlaubt ist.
:::

Für Rampen, Stufen, Spitzen und Schwellenwerte für Bestanden/Fehlgeschlagen
führen Sie die Anfrage
[unter Last in einem Experiment](../experiments/load.md) aus.

## Im Inspektor {#inspector}

Läuft der Mitschnitt, erscheint jeder Austausch als ein Frame mit dem Protokoll
`http` und der Quelle `http`: die Methode, URL, Status und Dauer in der
Zusammenfassung, die Antwort-Header und der Anfang des Bodys (2000 Zeichen) in
seinen Details, der Status als sein Urteil (`failed`, wenn keine Antwort kam,
`· digest after 401`, wenn eine Challenge beantwortet wurde). Der Frame zeichnet
die Größe des Bodys auf, nicht seine Bytes. Der `Authorization`-Header der
Anfrage ist nie darin. Ein Burst legt höchstens einen Austausch alle 100 ms in
den Mitschnitt. Siehe [Inspektor](../tools/inspector.md).

## Speichern und wiederverwenden {#library}

- **Als Signal speichern.** [[ui:sig.saveNew]] behält die Anfrage — Methode, URL,
  Header, Body, Timeout und Authentifizierung — in der Signalbibliothek. Die
  Ansicht bleibt an sie gebunden: [[ui:sig.save]] (<kbd>Ctrl</kbd>+<kbd>S</kbd>)
  aktualisiert sie, [[ui:sig.saveAs]] legt eine Kopie an, der Chip öffnet sie in
  [[ui:nav.signals]]. Wird ein HTTP-Signal aus der Bibliothek geöffnet, lädt es
  hierher zurück, Anmeldedaten eingeschlossen. Siehe
  [Signale](../tools/signals.md).
- **Zu einem Experiment hinzufügen.** [[ui:common.toExperiment]] fügt dem
  geöffneten Experiment einen Schritt
  [HTTP-Anfrage](../experiments/nodes.md#node-http) mit derselben Anfrage hinzu,
  direkt vor Ende oder nach dem ausgewählten Schritt, und öffnet es.
- **Dies nachbilden.** Unter einer Antwort legt [[ui:http.mockThis]] eine
  Emulator-Route an, die diese Methode und diesen Pfad mit diesem Status, diesen
  Headern und diesem Body beantwortet. Wählen Sie unter [[ui:http.mockInto]]
  einen HTTP-Emulator oder [[ui:http.mockNew]], und drücken Sie
  [[ui:http.mockAdd]]; die Route kommt in diesem Emulator an die erste Stelle,
  und die Ansicht [[ui:nav.emulators]] öffnet sich bei ihm. Siehe
  [Emulatoren](../tools/emulators.md).

## In Experimenten {#experiments}

| Schritt | Was er tut |
| --- | --- |
| [[ui:exp.node.http]] | Sendet eine Anfrage; seine URL, Header, Body und Anmeldedaten nehmen `{{templates}}`. Er kann [unter Last](../experiments/load.md) laufen. [Details](../experiments/nodes.md#node-http) |
| [[ui:exp.node.assert_status]], [[ui:exp.node.assert_body]], [[ui:exp.node.assert_header]], [[ui:exp.node.assert_latency]] | Prüfen die letzte Antwort. [Details](../experiments/nodes.md#node-assert_status) |
| [[ui:exp.node.extract]] | Behält ein JSON-Feld, einen Header, den Status, den Body oder den Treffer eines regulären Ausdrucks als Variable. [Details](../experiments/nodes.md#node-extract) |
| [[ui:exp.node.branch_status]] | Geht je nach Status durch Ja oder Nein weiter. [Details](../experiments/nodes.md#node-branch_status) |
| [[ui:exp.node.wait_http]] | Wartet auf eine eintreffende Anfrage — von dem System, das Sie testen — am eigenen Empfänger oder Emulator des Durchlaufs. [Details](../experiments/nodes.md#node-wait_http) |
| [[ui:exp.node.emulator]] | Eine HTTP-API, die für den ganzen Durchlauf nach Routen antwortet. [Details](../experiments/nodes.md#node-emulator) |

## Von der Kommandozeile {#cli}

`signallab send http` sendet eine Anfrage, wie es diese Ansicht tut:

```bash
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab send http POST http://127.0.0.1:8080/api/items \
  -H 'Content-Type: application/json' --body '{"name":"lamp"}'
signallab send http GET http://127.0.0.1:8080/private -u admin:secret --digest
```

Die Statuszeile geht an die Standardfehlerausgabe und der Body an die
Standardausgabe:

```text
HTTP 200 OK · 3 ms · 15 B
{"status":"ok"}
```

| Option | Was | Standard |
| --- | --- | --- |
| `-H`, `--header 'Name: value'` | Ein Header; für mehr wiederholen | — |
| `--body TEXT`, `--body @FILE` | Der Body oder der Inhalt einer Datei | — |
| `--expect-status N` | Endet mit 1, wenn der Status nicht N ist | — |
| `--timeout MS` | Wie lange auf die Antwort gewartet wird | 10.000 |
| `-u`, `--user NAME:PASSWORD` | Basic-Authentifizierung | — |
| `--digest` | Mit `--user`: stattdessen die Digest-Challenge des Servers beantworten | — |
| `--bearer TOKEN` | `Authorization: Bearer TOKEN` | — |
| `--json` | Die ganze Antwort als JSON auf der Standardausgabe ausgeben | — |

Es endet mit 0, wenn eine Antwort kam (und den erwarteten Status hatte), mit 1,
wenn keine kam, der Status nicht der erwartete war oder eine Digest-Challenge
nicht beantwortet werden konnte, und mit 2, wenn eine Option ungültig ist. Es
behält keine Cookies. Siehe
[Kommandozeile](../automation/cli.md#cli-send-http).

## Probleme {#troubleshooting}

| Was Sie sehen | Übliche Ursache |
| --- | --- |
| `… refused the connection — nothing is listening on that port` | Der Server läuft nicht oder empfängt auf einem anderen Port oder einer anderen Adresse. |
| `No answer from … in time` | Der Server ist langsam oder nicht erreichbar; prüfen Sie die Adresse, oder erhöhen Sie [[ui:common.timeoutMs]]. |
| `Cannot resolve …` | Der Hostname lässt sich auf diesem Computer nicht auflösen — ein Tippfehler oder ein Name, den nur ein anderes Netzwerk kennt. |
| `A secure connection to … could not be made` | Dem Zertifikat wird hier nicht vertraut (selbst ausgestellt, abgelaufen, ein anderer Name), oder TLS schlug fehl. Siehe [Sichere Verbindungen](#tls). |
| `… is not a valid address` | Die URL ist fehlerhaft oder beginnt nicht mit `http://` oder `https://`. |
| Der Server sagt, der Body fehle oder habe den falschen Typ | Kein `Content-Type`-Header, der zum Body passt, oder ein leerer Body. |
| `401` mit Digest | Lesen Sie die Meldung unter dem Status: siehe [Digest](#digest). |
| [[ui:http.missed]] über 0 | Erhöhen Sie [[ui:common.concurrency]], oder senken Sie die Rate: Der Server antwortet langsamer, als die Rate benötigt. |
| [[ui:http.failed]] hoch, obwohl der Server antwortet | Jeder Status außerhalb von 200–299 zählt als fehlgeschlagen, 404 und 500 eingeschlossen. |

Auf einem Server gehen die Anfragen vom Server aus: `127.0.0.1` ist der Server
selbst. Siehe [Server](../server/index.md).

Jede Fehlermeldung steht unter
[Fehlermeldungen](../reference/errors.md#transport).

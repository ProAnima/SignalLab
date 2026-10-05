---
title: Fehlerbehebung
description: Häufige Probleme mit Signal Lab und wie Sie sie beheben — nichts kommt an, ein belegter Port, die Firewall, Docker-Netzwerk, der Server, MQTT, Zertifikate, Updates und Logs.
---

# Fehlerbehebung

Jeder Fehlschlag, den Signal Lab meldet, hat einen Code; die Meldung zu jedem
steht in [Fehlermeldungen](errors.md). Netzwerkfehlschläge sind die
[`transport`](errors.md#transport)-Codes: `refused`, `timeout`, `dns`,
`unreachable`, `reset`, `address_in_use`, `address_unavailable`, `denied`,
`tls`, `target_invalid`, `failed`. Die Probleme unten sind die häufigen.

## Nichts kommt an {#nothing-arrives}

Finden Sie zuerst heraus, ob überhaupt etwas Signal Lab erreicht: Öffnen Sie im
unteren Bereich die Registerkarte [[ui:dock.inspector]] und drücken Sie
[[ui:ins.arm]]. Jedes Datagramm, jede Anfrage und jede Nachricht, die ein
Werkzeug sendet oder empfängt, wird dort aufgelistet, mit ihrer Herkunft.

### Die Empfangsadresse {#listen-address}

- Ein [[ui:common.bind]] von `0.0.0.0:<port>` empfängt auf jeder Netzwerkkarte;
  `127.0.0.1:<port>` hört nur diesen Rechner. Technik im Netzwerk braucht das
  erste.
- Das Gerät muss an die Adresse dieses Rechners und an den Port senden, auf dem
  Sie empfangen. Die Kopfzeile zeigt Name und Adresse dieses Rechners.
- Eine Adresse, die nicht die dieses Rechners ist, schlägt mit
  `address_unavailable` fehl.

### Die Firewall {#firewall}

Verkehr auf `127.0.0.1` wird nie gefiltert, weshalb ein Test auf einem Rechner
funktioniert, während derselbe Test von einem anderen Rechner nichts bekommt.

**Windows.** Die Windows-Firewall entscheidet pro Programm. Windows fragt meist
einmal, wenn ein Programm zum ersten Mal empfängt — und ein *Abbrechen* dort
hinterlässt eine Regel, die es blockiert, und die gewinnt gegen jede
Zulassungsregel. In einem Netzwerk, das Windows als öffentlich bezeichnet (das
WLAN eines Veranstaltungsorts oft), fragt es womöglich gar nicht.

- Die Desktop-App sieht die Firewall einmal an, wenn ein Monitor, ein
  Erkennungsempfänger, ein Relais, ein Durchlauf oder ein Emulator zum ersten
  Mal empfängt. Steht die Firewall im Weg, sagt sie es in einem Hinweis mit
  [[ui:fw.allow]] — oder [[ui:fw.allowPublic]] in einem öffentlichen Netzwerk.
  Windows fordert Administratorrechte an, dann werden die eingehenden Regeln des
  Programms, eine Blockregel eingeschlossen, durch eine Zulassungsregel ersetzt.
  [[ui:fw.dismiss]] blendet den Hinweis aus.
- Aus einem Terminal: `signallab doctor` zeigt, was im Weg steht, und
  `signallab firewall allow` behebt es (`--public` auch für öffentliche
  Netzwerke), mit derselben Administratorabfrage.
- Ein Setup (`.exe`), das *für alle* installiert wurde, legt die
  Zulassungsregeln selbst an (private Netzwerke und Domänennetzwerke), sofern es
  nicht mit `/NOFIREWALL` ausgeführt wurde. Ein Setup *für mich* kann das nicht,
  und das `.msi` überlässt die Firewall dem, der es verteilt.
- Ein Server ändert nie die Firewall seines Hosts: Sein Administrator öffnet die
  Ports (das Installationsskript bietet es an, mit ufw oder firewalld).

**Linux.** Eine Firewall wie ufw oder firewalld arbeitet nach Port, nicht nach
Programm. `signallab doctor` nennt die, die aktiv ist, und wie man einen Port
öffnet, zum Beispiel `sudo ufw allow 9000/udp`.

### Broadcast und Multicast {#broadcast-multicast}

- Router leiten Broadcast nicht weiter: `255.255.255.255` und `x.x.x.255`
  erreichen nur das Netzwerksegment, in dem die sendende Karte liegt. Bei
  mehreren Karten setzen Sie die Adresse der Karte in [[ui:bc.bindSource]] (unter
  [[ui:bc.socketOptions]]), zum Beispiel `10.0.0.5:0`.
- Ein Multicast-Datagramm erreicht nur Empfänger, die seiner Gruppe beigetreten
  sind — beim Erkennungsempfänger [[ui:bc.joinGroups]]. Mit [[ui:bc.ttl]] auf 1,
  der Voreinstellung, bleibt es in diesem Netzwerk.
- Um Ihr eigenes Multicast auf demselben Rechner zu hören, lassen Sie
  [[ui:bc.mcastLoop]] an.

### Ein Gerät, das nicht antwortet {#no-answer}

UDP hat keinen Zustellnachweis: Ein Datagramm, das an einen Port gesendet wird,
auf dem niemand empfängt, zählt trotzdem als gesendet. Windows meldet dann das
ICMP *Port unreachable*, das es zurückbekam, beim nächsten Empfang dieses Sockets
als Verbindungszurücksetzung; die Monitore, Empfänger, Relais und Warteknoten von
Signal Lab ignorieren es und hören weiter. Wenn also keine Antwort kommt, schlägt
ein Warten nach seiner Zeit mit `wait.timeout` fehl, nicht mit einem Fehler über
das Senden. Prüfen Sie im [[ui:dock.inspector]], dass die Nachricht an die
richtige Adresse ging, und prüfen Sie dann das Gerät.

## Ein Senden wird abgelehnt, bevor es hinausgeht {#refused-send}

Diese werden zuerst geprüft, und nichts wird gesendet, wenn eines fehlschlägt:

| Code | Warum | Behebung |
| --- | --- | --- |
| `transport.target_invalid` | Das Ziel hat keinen Port oder ist weder `IP:port` noch `host:port` | Schreiben Sie beides, etwa `192.0.2.20:9000` |
| `transport.dns` | Der Hostname lässt sich auf diesem Rechner nicht auflösen | Prüfen Sie den Namen oder verwenden Sie die Adresse. Ein Name mit einer IPv4-Adresse wird über IPv4 erreicht, daher findet `localhost:9000` einen Empfänger auf `127.0.0.1` |
| `node.osc_address` | Eine OSC-Adresse beginnt nicht mit `/` — im Sender, im Generator, in einem Signal oder einem Schritt | Beginnen Sie sie mit `/`, etwa `/cue/go` |
| `node.topic_wildcard` | Das Topic einer MQTT-Veröffentlichung enthält `+` oder `#`, bei der Verbindung der Ansicht wie überall sonst | Veröffentlichen Sie auf ein Topic; Wildcards sind zum Abonnieren |
| `node.too_long` bei einer WebSocket-Nachricht | Die Nachricht ist über 16 MiB | Senden Sie weniger; die Verbindung bleibt offen |

## Ein Port ist bereits belegt {#port-in-use}

`transport.address_in_use`: Ein anderes Programm — oder ein anderer Job von
Signal Lab — empfängt bereits auf diesem Port.

- **Ein Monitor und ein Durchlauf.** Ein Durchlauf öffnet die Ports seiner
  Warteknoten, Emulatoren und Relais vor seinem ersten Schritt, daher lässt ein
  Port, den ein [[ui:osc.monitor]], ein Erkennungsempfänger oder ein Emulator-Job
  hält, den Durchlauf schon vor dem Start fehlschlagen. Stoppen Sie zuerst diesen
  Job; [[ui:app.stopAll]] stoppt jeden.
- **Zwei Emulatoren** in einem Durchlauf können sich keinen Port eines Transports
  teilen: HTTP-, MQTT- und TCP-Emulatoren empfangen alle auf TCP, OSC- und
  UDP-Emulatoren auf UDP (`emulator.bind_taken`).
- **Neben dem echten Dienst empfangen.** Der Erkennungsempfänger kann sich einen
  Port mit einem Programm teilen, das ihn bereits hält: Lassen Sie
  [[ui:bc.reuse]] an. Unter Linux muss dieses Programm seinen Port ebenfalls
  teilen. Ohne das ist ein belegter Port `broadcast.port_shared`.
- **Gerade gestoppt.** Ein Port, den ein gestoppter Emulator oder Durchlauf
  hielt, wird einen Moment später freigegeben; ein sofort erneut gestarteter
  Emulator wartet kurz darauf.
- **Ports unter 1024** brauchen unter Linux Administratorrechte (`denied`). Das
  Server-Image läuft ohne, verwenden Sie also einen Port ab 1024.

## Der Server in Docker erreicht das Netzwerk nicht {#docker-network}

Broadcast, Multicast und Erkennung erreichen das physische Netzwerk nur mit
Host-Netzwerk — `network_mode: host` in der Compose-Datei oder
`docker run --network host` — und nur auf einem Linux-Host. Es lässt außerdem
Monitore, Warteknoten und Emulatoren auf den Ports des Hosts selbst empfangen.
Beim standardmäßigen Bridge-Netzwerk von Docker liegt der Container in einem
eigenen Netzwerk: Broadcast und Multicast verlassen es nie, und nur die Ports,
die Sie veröffentlichen, erreichen ihn.

Unter Windows und macOS erreicht das Host-Netzwerk von Docker das physische
Netzwerk nicht: Verwenden Sie unter Windows die Desktop-App oder betreiben Sie
den Server auf einem Linux-Host.

## SmartScreen warnt vor dem Installer {#smartscreen}

Die Installer sind noch nicht signiert, daher sagt Windows SmartScreen, dass es
den Herausgeber nicht kennt. Wählen Sie *Weitere Informationen*, dann *Trotzdem
ausführen*. Laden Sie Installer nur aus den Releases des Projekts auf GitHub
herunter.

## Der Server startet nicht {#server-start}

`signal-lab-server` prüft seine Einstellungen, bevor er empfängt, und endet mit
einer Meldung auf seiner Fehlerausgabe:

| Exit-Code | Meldung | Behebung |
| --- | --- | --- |
| 2 | refusing to listen on … without a token | Ein Server, den andere erreichen können, braucht ein Token: `--token-file` oder `SIGNALLAB_TOKEN` (erzeugen Sie eines mit `signal-lab-server token`) oder `--generate-token`. Oder empfangen Sie auf `127.0.0.1` |
| 2 | the token has N characters; it needs at least 24 | Verwenden Sie ein längeres Token |
| 2 | the token must not contain spaces or line breaks | Eine Token-Datei darf mit einem Zeilenumbruch enden; sonst nichts |
| 2 | give the token once | Verwenden Sie `--token`/`SIGNALLAB_TOKEN` oder `--token-file`/`SIGNALLAB_TOKEN_FILE`, nicht beides |
| 2 | --generate-token keeps the token in the data folder | Setzen Sie `--data-dir` oder `SIGNALLAB_DATA_DIR` |
| 2 | … it does not hold a valid token; remove it to have a new one made | Die `token`-Datei des Datenordners ist beschädigt |
| 2 | cannot read the token file … | Die von `--token-file` genannte Datei fehlt oder ist für diesen Benutzer nicht lesbar |
| 2 | cannot save the new token in … | `--generate-token` konnte `token` nicht im Datenordner schreiben: Machen Sie den Ordner für diesen Benutzer beschreibbar |
| 1 | cannot listen on … | Die Adresse ist nicht die dieses Rechners, oder der Port ist belegt |
| 1 | the data folder … must be writable by this user | In Docker muss ein bind-gemounteter Ordner für uid 10001 beschreibbar sein |

Ein Token, das `--generate-token` erzeugte, wird einmal ausgegeben, beim ersten
Start (`docker logs signallab` zeigt es), und in `token` im Datenordner
aufbewahrt: `docker exec signallab cat /data/token`. Siehe
[den Server](../server/index.md).

## Anmeldung am Server nicht möglich {#sign-in}

| Was Sie sehen | Warum | Behebung |
| --- | --- | --- |
| *That token is not right.* | Ein falsches Token | Kopieren Sie es erneut von dort, wo der Server es aufbewahrt; die Antwort dauert absichtlich eine Sekunde |
| `auth.host` | Der Server antwortet nicht auf den Namen in der Adressleiste | Öffnen Sie ihn über einen Namen, den er akzeptiert. Loopback-Namen (`localhost`, `127.x.x.x`, `[::1]`) bestehen immer. Ohne Token antwortet der Server nur auf diese und die Namen aus `--allowed-host`; mit Token auf jeden Namen, sofern `--allowed-host` ihn nicht einschränkt |
| `auth.origin` | Eine Anfrage kam von einer Seite eines anderen Origin | Reichen Sie hinter einem Reverse-Proxy den `Host` des Browsers an den Server weiter (nginx: `proxy_set_header Host $host;`), damit `Origin` und `Host` übereinstimmen |
| Zurück auf der Anmeldeseite nach der Anmeldung | Der Browser hat das Sitzungs-Cookie nicht behalten | Mit `--secure-cookie` muss der Server über HTTPS erreicht werden |
| Nach einer Weile abgemeldet | Sitzungen dauern 7 Tage und enden beim Neustart des Servers; über 1024 Sitzungen geht die älteste weg | Melden Sie sich erneut an |

## Die Verbindung zum Server bricht immer wieder ab {#connection-lost}

[[ui:app.connectionLost]] bedeutet, dass der Ereignis-Socket der Seite,
`/api/events`, geschlossen wurde. Die Seite verbindet sich von selbst neu, nach
einer halben Sekunde und dann seltener, bis zu alle 15 s. Was in der
Zwischenzeit geschah, wird nicht wiederholt: Laufende Jobs melden sich weiter,
während sie laufen. Stellen Sie hinter einem Reverse-Proxy sicher, dass er
WebSocket-Upgrades für `/api/events` weiterreicht und ruhige Verbindungen nicht
in unter 20 s schließt (der Server pingt alle 20 s). Eine Seite, die sagt, der
Server habe keine solche Adresse (`api.not_found`), ist älter als der Server:
Laden Sie sie neu.

## MQTT verbindet sich nicht {#mqtt}

Signal Lab spricht MQTT 3.1.1 über einfaches TCP. Es verbindet und wartet auf die
Antwort des Brokers (CONNACK), bevor es Erfolg meldet, daher steht der Grund auf
der Schaltfläche, die verbindet:

| Code | Warum | Behebung |
| --- | --- | --- |
| `transport.refused` | Auf diesem Port empfängt niemand | Prüfen Sie den Port: 1883 ist der übliche |
| `transport.timeout` | Keine TCP-Verbindung innerhalb von 6 s | Prüfen Sie die Adresse, das Netzwerk, die Firewall des Brokers |
| `transport.dns` | Der Hostname lässt sich nicht auflösen | Prüfen Sie den Namen oder verwenden Sie die Adresse |
| `transport.unreachable` | Keine Route zum Broker | Prüfen Sie das Netzwerk und die Adresse |
| `transport.reset` | Der Broker hat die Verbindung sofort geschlossen | Oft ein TLS-Port (8883) — Signal Lab spricht kein MQTT über TLS |
| `mqtt.no_answer` | Der Port ist offen, aber innerhalb von 6 s kam kein CONNACK | Oft ein WebSocket-Port — Signal Lab spricht kein MQTT über WebSocket |
| `mqtt.protocol` | Was geantwortet hat, ist kein MQTT-Broker | Prüfen Sie den Port |
| `mqtt.refused_protocol` | Der Broker akzeptiert MQTT 3.1.1 nicht | Aktivieren Sie 3.1.1 am Broker |
| `mqtt.refused_client_id` | Der Broker lehnt die Client-id ab | Verwenden Sie eine andere Client-id |
| `mqtt.refused_unavailable` | Der Broker ist nicht verfügbar | Versuchen Sie es später erneut |
| `mqtt.refused_credentials` | Benutzername oder Passwort ist falsch | Prüfen Sie sie |
| `mqtt.refused_not_authorized` | Der Benutzer darf sich nicht verbinden | Prüfen Sie die Zugriffsregeln des Brokers |
| `mqtt.client_id_required` | Die Client-id ist leer | Füllen Sie sie aus |

Ein Broker verwirft die ältere von zwei Verbindungen mit derselben Client-id:
Wenn sich eine Verbindung immer wieder schließt, suchen Sie nach einem anderen
Client mit derselben id.

## Ein Zertifikat wird nicht vertraut {#tls}

`transport.tls` bei `https://` oder `wss://`: Das Zertifikat des Servers wird
nicht vertraut, nennt nicht den Host, nach dem Sie gefragt haben, oder TLS
konnte nicht ausgehandelt werden. Signal Lab prüft Zertifikate wie das System und
hat keinen Schalter, um die Prüfung zu überspringen. HTTPS und WSS vertrauen
denselben Zertifikaten: denen des Betriebssystems, auf dem die Engine läuft — dem
Windows-Zertifikatsspeicher unter Windows, den CA-Zertifikaten des Systems unter
Linux und im Server-Image. Fügen Sie für ein selbstsigniertes Zertifikat oder
Ihre eigene CA dieses den vertrauten Zertifikaten jenes Rechners hinzu (für das
Server-Image ein darauf gebautes Image, das das Zertifikat hinzufügt), und
verbinden Sie sich über den Namen, den das Zertifikat trägt.

## Ein Geheimnis fehlt {#secrets}

`secret.missing`: Ein Experiment verwendet `{{secret.NAME}}`, und unter diesem
Namen ist dort, wo es läuft, kein Wert gespeichert.

- **Desktop-App unter Windows**: Setzen Sie es unter [[ui:exp.secrets]] in den
  [[ui:exp.params]] des Editors. Es wird in der
  Windows-Anmeldeinformationsverwaltung aufbewahrt, ein neuer Rechner braucht es
  also erneut gesetzt.
- **Server**: Legen Sie den Wert in die Datei `/run/secrets/signallab/NAME` (oder
  den Ordner, den `--secrets-dir` nennt) oder in die Umgebungsvariable
  `SIGNALLAB_SECRET_NAME`. Ein Server kann Geheimnisse nicht von der Seite setzen
  (`secret.read_only`).
- **Desktop-App unter Linux** hat keinen Speicher für Geheimnisse
  (`secret.unsupported`). Führen Sie ein solches Experiment mit `signallab run`
  aus, das Geheimnisse aus Dateien und Variablen liest, oder auf einem Server.

Siehe [Dateien](files.md#secrets).

## Ein Durchlauf stoppt nach fünf Minuten {#run-timeout}

Ein Durchlauf, der länger als 300 s dauert, schlägt mit `run.timeout` fehl; das
ist das längste, was ein Durchlauf dauern kann. Ein kürzeres Limit kann einem
Durchlauf über die API (`timeout` von [`/api/run`](../api/run.md#request)) oder
die Kommandozeile (`signallab run --timeout`) gegeben werden.

## Die App aktualisiert sich nicht {#updates}

- Die Desktop-App sucht einmal am Tag nach einer neuen Version, solange
  [[ui:update.auto]] an ist, und wenn Sie [[ui:update.check]] in
  [[ui:about.open]] drücken.
- Sie fragt zuerst beim Hub des Studios und bei GitHub, wenn der Hub nicht
  erreichbar ist. Blockiert ein Netzwerk beide, gibt [[ui:update.check]]
  [[ui:update.checkFailed]]; die tägliche Suche schlägt wortlos fehl.
- Es werden nur veröffentlichte Releases angeboten, nie Entwürfe oder
  Vorabversionen.
- Ein neues Release erreicht die App, wenn der Hub es anbietet, was einige Zeit
  nach seinem Erscheinen auf GitHub sein kann: Der Hub rollt ein Release jeweils
  für einen Teil der Installationen aus.
- Es wird nur installiert, wenn Sie [[ui:update.install]] drücken — laufende Jobs
  werden zuerst gestoppt —, und nur ein Release, dessen Signatur stimmt: sonst
  [[ui:update.installFailed]].
- Ein Server aktualisiert sich mit seinem Image: `docker compose pull &&
  docker compose up -d` im Ordner seiner Compose-Datei.

## Wo die Logs sind {#logs}

- **Desktop-App**: Sie schreibt keine Log-Dateien. Die Registerkarte
  [[ui:console.title]] im unteren Bereich listet auf, was jedes Werkzeug tat und
  was schiefging, und [[ui:feedback.open]] hängt sie an eine Nachricht an die
  Entwickler (ohne den Namen dieses Rechners, seine Adresse oder Ihre Ordner).
  Der Bericht jedes Durchlaufs liegt in `runs/` im Datenordner.
- **Server**: Er protokolliert auf seine Standardausgabe und -fehlerausgabe — in
  Docker `docker logs signallab`. Jeder Jobstart wird mit der Adresse des
  anfragenden Clients protokolliert. `--log` (oder `SIGNALLAB_LOG`) setzt die
  Stufe: `error`, `warn`, `info` (die Voreinstellung) oder `debug`;
  `--log-format json` (oder `SIGNALLAB_LOG_FORMAT`) schreibt ein JSON-Objekt pro
  Zeile.
- **Kommandozeile**: `signallab` schreibt seine Meldungen auf seine
  Fehlerausgabe; siehe [die Kommandozeile](../automation/cli.md).

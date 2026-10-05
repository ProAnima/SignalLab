---
title: Einen Server betreiben
description: Betreiben Sie Signal Lab auf einem Linux-Rechner neben der Technik und nutzen Sie es aus jedem Browser im Netzwerk, mit einer HTTP-API für Skripte und CI.
---

# Signal Lab als Server betreiben

`signal-lab-server` ist Signal Lab ohne Fenster: dieselbe Engine, die dieselbe
Oberfläche an einen Browser ausliefert. Stellen Sie ihn auf einen Rechner neben
der Technik — einen Rack-PC, eine Show-Control-VM, einen gemeinsamen
Labor-Rechner — und öffnen Sie ihn aus Chrome, Firefox oder Edge von überall im
Netzwerk: Jede Ansicht funktioniert wie in der Desktop-App, und Durchläufe,
Berichte und Exporte laden über den Browser herunter.

Skripte und Pipelines verwenden denselben Server über seine
[HTTP-API](../api/index.md), und
[`signallab --server`](../automation/cli.md#run-on-server) sendet Durchläufe an
ihn.

Einiges ist anders als in der Desktop-App:

- **Ein Server ist eine Engine.** Jede angemeldete Seite sieht dieselben
  laufenden Jobs, dieselben Signal- und Emulatorbibliotheken und denselben
  Cookie-Vorrat der Ansicht [[ui:nav.http]] ([[ui:http.keepCookies]]). Wer sich
  einen Server teilt, teilt diese.
- **Geheimnisse gehören dem Server**, gelesen aus seiner Umgebung oder aus
  Dateien; sie lassen sich nicht aus dem Browser setzen (siehe
  [Geheimnisse](#secrets)).
- **Die Firewall gehört dem Host.** Der Server ändert sie nie; der
  Firewall-Hinweis der Desktop-App erscheint nicht.
- **Er aktualisiert sich mit seinem Image**, nicht über den Updater der App
  (siehe [Aktualisieren](#update)).

## Auf einem Linux-Host, mit einem Befehl {#install-script}

Auf einem Linux-Rechner mit Internetzugang:

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh
```

Das Skript:

1. installiert Docker, falls es fehlt — es fragt vorher, über den Installer von
   Docker selbst (`get.docker.com`);
2. schreibt eine `compose.yaml` nach `/opt/signallab` (`~/signallab`, wenn Sie
   nicht root sind);
3. lädt das Image und startet den Server mit Host-Netzwerk, sodass OSC, UDP,
   Broadcast, Multicast und Erkennung das echte Netzwerk erreichen;
4. wartet, bis der Server seine Zustandsprüfung beantwortet (bis zu 90 Sekunden);
5. gibt die Adressen aus, die Sie öffnen können, das Zugriffstoken zum Anmelden
   und die Befehle zum Aktualisieren, zum Lesen der Logs und zum Entfernen;
6. bietet, wenn ufw oder firewalld aktiv ist, an, den Port des Servers zu öffnen
   (siehe [Die Firewall des Hosts](#firewall)).

Übergeben Sie Optionen nach `sh -s --`:

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh -s -- --version [[version]] --port 8430
```

| Option | Wirkung | Voreinstellung |
| --- | --- | --- |
| `--version X.Y.Z` | Die Version des Images (`latest` oder `X.Y.Z`; ein führendes `v` entfällt). | `latest` |
| `--port N` | Der Port, den Browser verwenden. | `1430` |
| `--listen IP:PORT` | Nur auf einer Adresse empfangen. | `0.0.0.0:<port>` |
| `--dir DIR` | Wohin die Compose-Datei kommt. | `/opt/<name>` als root, sonst `~/<name>` |
| `--name NAME` | Der Container und sein Daten-Volume: Kleinbuchstaben, Ziffern, `-` und `_`. Ein zweiter Server auf demselben Host braucht einen eigenen Namen. | `signallab` |
| `--image NAME` | Ein anderes Image oder eine andere Registry; ein vollständiges `NAME:TAG` wird unverändert verwendet. | `ghcr.io/proanima/signallab` |
| `--open-udp PORTS` | Bei aktiver Firewall auch UDP auf diesen Ports hereinlassen, für Monitore und Warteknoten: `9000,9100:9110`. | |
| `--no-firewall` | ufw oder firewalld nie ändern. | |
| `--yes`, `-y` | Ja antworten: Docker installieren, die Firewall öffnen, mit `--purge` Daten löschen. | |
| `--uninstall` | Den Server stoppen und entfernen; die Daten bleiben. | |
| `--purge` | Zusammen mit `--uninstall`: auch die Daten und das Token löschen. | |
| `--help`, `-h` | Die Optionen ausgeben. | |

Es braucht das Compose-Plugin von Docker (das Paket `docker-compose-plugin`), das
der Installer von Docker mitbringt. Das Image ist für x86_64 und arm64 gebaut.

**Führen Sie es erneut aus, um zu aktualisieren:** Derselbe Befehl lädt das
neueste Image (oder die von Ihnen angegebene `--version`) und startet den Server
neu; die Daten und das Token bleiben. Geben Sie bei `--name` denselben Namen
erneut an.

**Eigene Einstellungen** — Geheimnisse von Experimenten,
`SIGNALLAB_ALLOWED_HOSTS`, `SIGNALLAB_SECURE_COOKIE` hinter HTTPS — gehören in
`compose.override.yaml` neben die Compose-Datei. Docker Compose fügt sie ein,
und das Skript schreibt `compose.yaml` bei jedem Durchlauf neu, rührt die
Override-Datei aber nie an. Eine `compose.yaml`, die es nicht geschrieben hat,
wird als `compose.yaml.before-install` aufbewahrt.

```yaml
# compose.override.yaml
services:
  signallab:
    environment:
      SIGNALLAB_ALLOWED_HOSTS: lab-pc.example.com,192.0.2.10
      SIGNALLAB_SECRET_API_TOKEN: ${API_TOKEN}
```

**Zum Entfernen:**

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh -s -- --uninstall
```

Die Daten bleiben in ihrem Docker-Volume, und eine erneute Installation bringt
sie mit demselben Token zurück. `--uninstall --purge` löscht auch die Daten und
das Token, nachdem es gefragt hat.

### Die Firewall des Hosts {#firewall}

Mit Host-Netzwerk empfängt der Server auf den Ports des Hosts selbst, also
entscheidet die Firewall des Hosts, wer ihn erreicht. Ist ufw oder firewalld
aktiv, fragt das Skript, bevor es den TCP-Port des Servers und die UDP-Ports aus
`--open-udp` öffnet, schreibt auf, was es geöffnet hat (`.firewall` neben der
Compose-Datei), und `--uninstall` schließt genau das wieder. Wenn Sie ablehnen,
erreichen Browser auf anderen Rechnern den Server erst, sobald die Firewall sie
lässt, und Monitore und Warteknoten hören andere Rechner nur auf UDP-Ports, die
sie öffnet.

## Docker {#docker}

### Das Image {#image}

`ghcr.io/proanima/signallab`, für `linux/amd64` und `linux/arm64`, mit jedem
Release veröffentlicht:

| Tag | Was es ist |
| --- | --- |
| `X.Y.Z` | Dieses Release. |
| `X.Y` | Das neueste stabile Release dieser Linie. |
| `latest` | Das neueste stabile Release. |

Es enthält `signal-lab-server`, die gebaute Oberfläche und die Kommandozeile
`signallab` und startet den Server mit diesen Einstellungen:

| Variable | Wert im Image |
| --- | --- |
| `SIGNALLAB_LISTEN` | `0.0.0.0:1430` |
| `SIGNALLAB_DATA_DIR` | `/data` |
| `SIGNALLAB_UI_DIR` | `/usr/share/signal-lab/ui` |
| `SIGNALLAB_GENERATE_TOKEN` | `true` |

Es läuft als unprivilegierter Benutzer (uid und gid 10001), schreibt nur nach
`/data` (ein Volume), gibt Port `1430` frei und prüft alle 30 Sekunden seinen
eigenen Zustand.

### Starten {#docker-run}

```bash
docker run -d --name signallab --network host --restart unless-stopped \
  -v signallab-data:/data --read-only --cap-drop ALL --security-opt no-new-privileges \
  ghcr.io/proanima/signallab:[[version]]
docker logs signallab                    # on the first start: "Sign in with it:" and the token
```

Dann öffnen Sie `http://<host>:1430` und melden sich mit dem Token an. Um das
Token später erneut zu sehen:

```bash
docker exec signallab cat /data/token
```

`--read-only`, `--cap-drop ALL` und `no-new-privileges` sind optional und kosten
nichts: Der Server braucht keine Privilegien und schreibt nur nach `/data`.

### Docker Compose {#compose}

Die `deploy/compose.yaml` des Repositories entspricht einer Compose-Datei:

```yaml
name: signallab

services:
  signallab:
    image: ${SIGNALLAB_IMAGE:-ghcr.io/proanima/signallab:latest}
    container_name: signallab
    network_mode: host
    environment:
      SIGNALLAB_GENERATE_TOKEN: "true"
    volumes:
      - signallab-data:/data
    read_only: true
    cap_drop: [ALL]
    security_opt: ["no-new-privileges:true"]
    restart: unless-stopped
    stop_grace_period: 15s

volumes:
  signallab-data:
```

```bash
docker compose up -d
docker exec signallab cat /data/token
```

Setzen Sie `SIGNALLAB_IMAGE=ghcr.io/proanima/signallab:X.Y.Z`, um eine Version
festzulegen.

### Netzwerk {#networking}

| Docker-Netzwerk | Was funktioniert | Was nicht funktioniert |
| --- | --- | --- |
| `--network host` (`network_mode: host`), auf einem Linux-Host | Alles: OSC, UDP, TCP, HTTP, WebSocket und MQTT ins LAN, empfangende Ports, Broadcast, Multicast, Erkennung. | — |
| Bridge (die Voreinstellung), mit veröffentlichten Ports | Unicast zu den Hosts, die der Container erreicht; Empfänger auf veröffentlichten Ports (`-p 1430:1430 -p 9000:9000/udp`). | Broadcast und Multicast; Antworten an Ports, die nicht veröffentlicht sind. |
| Docker Desktop unter Windows oder macOS | Unicast und veröffentlichte Empfänger. | Host-Netzwerk zum physischen Netzwerk. Verwenden Sie unter Windows die Desktop-App. |

### Das Daten-Volume {#data-volume}

`/data` enthält alles, was der Server aufbewahrt: das Experiment, die Signal-
und die Emulatorbibliothek, Berichte von Durchläufen, Exporte und das Token. Ein
benanntes Volume, wie oben, gehört zunächst dem Benutzer des Images. Ein Ordner
des Hosts, der dorthin gemountet wird, muss für uid 10001 beschreibbar sein:

```bash
sudo mkdir -p /srv/signallab && sudo chown 10001:10001 /srv/signallab
docker run -d --name signallab --network host -v /srv/signallab:/data ghcr.io/proanima/signallab:[[version]]
```

## Ohne Docker {#binary}

Der Server wird als Image veröffentlicht. Um `signal-lab-server` direkt
auszuführen, bauen Sie ihn aus dem Quelltext (siehe
[Bauen](../../develop/building.md)):

```bash
npm install
npm run build                            # the interface, into dist/
cargo run --release -p signal-lab-server
```

Er liefert `dist/` auf `http://127.0.0.1:1430` aus, nur für diesen Rechner, ohne
Token. Fügen Sie `--listen` und ein Token hinzu, um ihn für das Netzwerk zu
öffnen.

## Optionen {#options}

Jede Option hat eine Umgebungsvariable, für Container. Optionen gewinnen gegen
Variablen.

| Option | Variable | Voreinstellung | Wirkung |
| --- | --- | --- | --- |
| `--listen IP:PORT` | `SIGNALLAB_LISTEN` | `127.0.0.1:1430` | Wo empfangen wird. Jede Adresse außer Loopback braucht ein Token. |
| `--token-file PATH` | `SIGNALLAB_TOKEN_FILE` | | Eine Datei mit dem Zugriffstoken (etwa ein Docker-Secret). |
| `--token TOKEN` | `SIGNALLAB_TOKEN` | | Das Zugriffstoken selbst. Bevorzugen Sie die Datei: Argumente sind für andere Benutzer des Rechners sichtbar. |
| `--generate-token` | `SIGNALLAB_GENERATE_TOKEN` | aus | Ohne angegebenes Token, auf einer Adresse jenseits von Loopback: das in `<data folder>/token` aufbewahrte Token verwenden und beim ersten Start erzeugen. |
| `--data-dir PATH` | `SIGNALLAB_DATA_DIR` | `Documents/SignalLab` im Benutzerordner | Der Datenordner. |
| `--secrets-dir PATH` | `SIGNALLAB_SECRETS_DIR` | `/run/secrets/signallab` | Ein Ordner mit schreibgeschützten Geheimnissen, eine Datei pro Name. |
| `--ui-dir PATH` | `SIGNALLAB_UI_DIR` | `ui` neben dem Programm, sonst `./dist` | Die gebaute Oberfläche. Ohne eine wird nur die API ausgeliefert. |
| `--allowed-host NAME` | `SIGNALLAB_ALLOWED_HOSTS` | | Hostnamen, über die der Server erreichbar sein darf, durch Kommas getrennt. Sie und die Loopback-Namen werden immer akzeptiert, mit Token oder ohne; ohne Angabe antwortet ein Server mit Token auf jeden Namen und einer ohne Token nur auf Loopback-Namen (siehe [Hostnamen](security.md#hosts)). |
| `--secure-cookie` | `SIGNALLAB_SECURE_COOKIE` | aus | Das Sitzungs-Cookie nur über HTTPS senden. Setzen Sie es hinter einem HTTPS-Proxy. |
| `--log FILTER` | `SIGNALLAB_LOG` | `info` | Was protokolliert wird: `error`, `warn`, `info`, `debug` oder pro Modul (`signal_lab_server=debug`). |
| `--log-format text\|json` | `SIGNALLAB_LOG_FORMAT` | `text` | Log-Zeilen als Text oder ein JSON-Objekt pro Zeile. |

Schalter nehmen `true` oder `false` aus ihrer Variable:
`SIGNALLAB_GENERATE_TOKEN=true`.

| Befehl | Wirkung |
| --- | --- |
| `signal-lab-server token` | Gibt ein neues zufälliges Token aus: 64 Hexadezimalzeichen. |
| `signal-lab-server healthcheck` | Endet mit `0`, wenn ein Server auf `--listen` antwortet (die Zustandsprüfung des Images). |
| `signal-lab-server --version` | Gibt die Version aus. |
| `signal-lab-server --help` | Gibt jede Option aus. |

**Exit-Codes:** `0` beim Stoppen durch <kbd>Ctrl</kbd>+<kbd>C</kbd> oder
`SIGTERM`; `2`, wenn die Einstellungen abgelehnt werden (kein Token auf einer
erreichbaren Adresse, ein zu kurzes Token, ein zweimal angegebenes Token, eine
nicht lesbare Token-Datei, `--generate-token` ohne Datenordner); `1`, wenn er
nicht auf der Adresse empfangen kann oder der Datenordner nicht beschreibbar
ist. Der Grund wird auf stderr ausgegeben.

## Das Zugriffstoken {#token}

Ohne Token empfängt der Server nur auf Loopback und bedient nur diesen Rechner.
Auf jeder anderen Adresse braucht er eines und verweigert ohne es den Start. Es
gibt drei Wege, es anzugeben:

| Wie | Wann man es verwendet |
| --- | --- |
| `--generate-token` (im Image an) | Nichts einzurichten: Beim ersten Start erzeugt der Server ein Token, legt es in `<data folder>/token` ab — nur für seinen eigenen Benutzer lesbar — und gibt es einmal im Log aus. Spätere Starts verwenden es wieder, sodass angemeldete Browser und Skripte über Neustarts und Updates hinweg weiterarbeiten. Es braucht einen Datenordner (`--data-dir`). |
| `--token-file PATH` | Ein eigenes Token in einer Datei, etwa ein Docker-Secret. Ein Zeilenumbruch an seinem Ende gehört nicht zum Token. |
| `SIGNALLAB_TOKEN` | Das Token in der Umgebung. |

Ein Token hat mindestens **24** Zeichen und keine Leerzeichen oder
Zeilenumbrüche; geben Sie es auf nur einem Weg an. `signal-lab-server token`
erzeugt ein gutes:

```bash
docker run --rm ghcr.io/proanima/signallab:[[version]] token > signallab_token.txt
```

und Compose reicht es als Secret hinein (die Datei muss für uid 10001 lesbar
sein):

```yaml
services:
  signallab:
    environment:
      SIGNALLAB_TOKEN_FILE: /run/secrets/signallab_token
    secrets:
      - signallab_token

secrets:
  signallab_token:
    file: ./signallab_token.txt
```

Ein ausdrücklich angegebenes Token gewinnt gegen `--generate-token`, und dann
wird keines erzeugt. Um ein erzeugtes Token zu ändern, stoppen Sie den Server,
löschen `<data folder>/token` und starten ihn erneut: Er erzeugt und gibt ein
neues aus. Eine beschädigte Token-Datei wird gemeldet, nie ersetzt.

## Anmelden {#sign-in}

Öffnen Sie `http://<host>:1430`. Ein Server mit Token schickt einen Browser
zuerst auf seine Anmeldeseite, in der Sprache des Browsers; fügen Sie das Token
einmal ein, und der Browser bleibt 7 Tage lang angemeldet. [[ui:app.signOut]] in
der Oberfläche beendet die Sitzung. Sitzungen liegen im Arbeitsspeicher des
Servers: Ein Neustart meldet alle ab.

Auf Loopback ohne Token gibt es keine Anmeldung.

Skripte senden das Token mit jeder Anfrage, als
`Authorization: Bearer <token>`:

```bash
curl -fsS http://192.0.2.10:1430/api/invoke/app_info \
  -H "Authorization: Bearer $(cat signallab_token.txt)" \
  -H "Content-Type: application/json" -d 'null'
```

Siehe [Die HTTP-API](../api/index.md) und [Server-Sicherheit](security.md) für
die Regeln hinter all dem.

## Der Datenordner {#data}

Der Server bewahrt seine Dateien im Datenordner auf: das Experiment, die Signal-
und die Emulatorbibliothek, Berichte von Durchläufen, Exporte und ein erzeugtes
Token. Er ist `--data-dir` (`SIGNALLAB_DATA_DIR`), im Image `/data` und sonst
`Documents/SignalLab` im Benutzerordner des Benutzers, als der er läuft. Ein mit
`--data-dir` angegebener Datenordner wird angelegt, wenn er fehlt, und beim
Start geprüft: Kann der Server dort nicht schreiben, stoppt er und nennt den
Ordner. Siehe [Dateien](../reference/files.md).

Downloads (Berichte, Exporte) kommen nur aus diesem Ordner.

## Geheimnisse {#secrets}

Experimente lesen Geheimnisse als `{{secret.NAME}}`. Auf einem Server sind sie
schreibgeschützt, aus:

1. der Umgebungsvariable `SIGNALLAB_SECRET_NAME`, sonst
2. der Datei `NAME` im Geheimnisordner (`--secrets-dir`, standardmäßig
   `/run/secrets/signallab` — das Docker-Secrets-Layout).

Ein Zeilenumbruch am Ende einer Datei gehört nicht zum Wert; ein leerer Wert
zählt als nicht gesetzt; ein Wert ist höchstens 16 KiB groß. Namen bestehen aus
Buchstaben, Ziffern und `_`, nicht mit einer Ziffer beginnend. Die Oberfläche
zeigt, welche Geheimnisse auf dem Server gesetzt sind, aber eines aus dem
Browser zu setzen wird abgelehnt — Werte gelangen nie an eine schwächere Stelle
und kommen nie wieder heraus (siehe [Geheimnisse](security.md#secrets)).

Mit Compose eine Geheimnisdatei pro Name:

```yaml
services:
  signallab:
    secrets:
      - source: api_token
        target: /run/secrets/signallab/API_TOKEN

secrets:
  api_token:
    file: ./api_token.txt
```

Die Datei muss für uid 10001 lesbar sein.

## Hinter einem HTTPS-Proxy {#https}

Der Server spricht einfaches HTTP. Für HTTPS stellen Sie einen Reverse-Proxy
davor (Caddy, nginx, Traefik), der:

- den `Host`-Header unverändert weiterreicht;
- WebSocket-Upgrades weiterreicht (die Oberfläche hält einen offen, zu
  `/api/events`);

und starten Sie den Server mit `--secure-cookie`, damit das Sitzungs-Cookie nur
über HTTPS reist, und mit `--allowed-host` auf den Namen, den die Leute
verwenden.

## Aktualisieren {#update}

[[ui:update.server]]: Der Updater der Desktop-App hat damit nichts zu tun.

- Mit dem Skript installiert: Führen Sie denselben Befehl erneut aus.
- Mit Compose: `docker compose pull && docker compose up -d`.
- Mit `docker run`: Laden Sie das neue Image, entfernen Sie dann den Container
  und starten Sie ihn mit demselben Volume erneut.

Die Daten und das Token liegen im Volume, bleiben also.

## Zustandsprüfung {#health}

`GET /api/health` antwortet jedem, ohne Token:

```bash
curl -s http://127.0.0.1:1430/api/health
# {"auth":true,"status":"ok","version":"[[version]]"}
```

`auth` sagt, ob der Server ein Token verlangt. Der Befehl
`signal-lab-server healthcheck` fragt dasselbe auf diesem Rechner und endet mit
`0`, wenn er antwortet; das Image führt ihn alle 30 Sekunden aus (5 Sekunden
Zeitlimit, 3 Versuche), sodass `docker ps` den Container als gesund anzeigt.

## Logs {#logs}

Der Server protokolliert nach stdout: `docker logs -f signallab`. Auf der
Voreinstellung (`info`) sagt er, wo er empfängt und ob er ein Token braucht,
seine Daten- und Oberflächenordner, jeden gestarteten Job — Monitore,
Generatoren, Stürme, Scans, Durchläufe, Emulatoren — mit der Adresse des
Clients, jede Anmeldung, jede Anmeldung mit einem falschen Token (als Warnung)
und wenn eine Seite bei den Ereignissen zurückfällt. Die Stufe `--log debug`
ergänzt jeden Befehl. Zeilen werden nur auf einem Terminal eingefärbt (nie mit
gesetztem `NO_COLOR`); `--log-format json` schreibt ein JSON-Objekt pro Zeile
für einen Log-Sammler.

## Stoppen {#stop}

<kbd>Ctrl</kbd>+<kbd>C</kbd>, `docker stop` oder `SIGTERM` schließt die
Verbindung jeder Seite, stoppt jeden Job — ein laufender Durchlauf endet als
gestoppt — und endet mit `0`. Die Compose-Dateien geben ihm dafür 15 Sekunden.

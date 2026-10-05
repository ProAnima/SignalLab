---
title: Server-Sicherheit
description: Wer einen Signal-Lab-Server nutzen kann und wie er andere fernhält — Tokens, Sitzungen, Host- und Origin-Prüfungen, Geheimnisse — und was Signal Lab nach außen sendet.
---

# Server-Sicherheit

Ein Signal-Lab-Server sendet echten Verkehr von dem Rechner aus, auf dem er läuft: OSC, UDP,
HTTP, MQTT, Stürme, Scans, Broadcasts. Wer ihn nutzen kann, kann all das
von diesem Rechner aus tun, daher ist der Server standardmäßig geschlossen und öffnet sich nur mit
einem Token.

::: warning
Behandeln Sie das Zugriffstoken wie ein Passwort für das Netzwerk des Rechners. Wer es
hat, kann vom Server aus Verkehr an alles senden, was der Server erreicht.
:::

## Auf einen Blick {#summary}

- **Ohne Token nur dieser Rechner.** Ohne Token empfängt der Server auf
  Loopback und antwortet nur auf Loopback-Hostnamen. Auf jeder anderen Adresse
  verweigert er den Start.
- **Ein Token für alle anderen.** Browser melden sich einmal an und erhalten ein
  Sitzungs-Cookie; Skripte senden das Token mit jeder Anfrage.
- **Nur die eigenen Seiten.** Anfragen, die etwas ändern, und der WebSocket für Ereignisse
  müssen vom eigenen Origin des Servers kommen; Befehle nehmen nur JSON an.
- **Nur die eigenen Namen.** Ein Hostname, auf den der Server nicht antwortet, wird abgelehnt,
  was DNS-Rebinding verhindert.
- **Geheimnisse bleiben drinnen.** Schreibgeschützt, aus der Umgebung oder aus Dateien, nie
  zurückgegeben, maskiert überall dort, wo sie sonst erschienen.
- **Nichts über seine Aufgabe hinaus.** Er ändert nie die Firewall des Hosts, liefert keine
  Datei außerhalb seines Datenordners aus und braucht keine Privilegien.

## Ohne Token: nur dieser Rechner {#loopback}

Ohne Token gestartet, empfängt der Server auf `127.0.0.1:1430` und braucht keine
Anmeldung: Er ist ein Werkzeug für die Person an diesem Rechner. Damit eine Webseite in
einem beliebigen Browser auf diesem Rechner ihn nicht über einen Namen erreicht, der zu
`127.0.0.1` aufgelöst wird (DNS-Rebinding), antwortet er nur auf Anfragen, deren `Host` ein
Loopback-Name ist — `localhost`, ein Name mit der Endung `.localhost`, `127.x.x.x` oder
`[::1]` — oder ein Name, den Sie mit `--allowed-host` zulassen.

Soll er ohne Token auf einer anderen Adresse empfangen, startet er nicht: Er
sagt, warum, und beendet sich mit dem Code `2`.

## Das Token {#token}

Ein Token hat mindestens 24 Zeichen, ohne Leerzeichen und Zeilenumbrüche.
`signal-lab-server token` gibt ein zufälliges mit 64 Hexadezimalzeichen aus, und
`--generate-token` (im Image eingeschaltet) erzeugt beim ersten Start eines, legt es im
Datenordner ab, lesbar nur für den eigenen Benutzer des Servers, und gibt es einmal aus. Alle Wege, eines
anzugeben, stehen unter [Das Zugriffstoken](index.md#token).

Der Vergleich eines Tokens dauert immer gleich lang, egal wo es abweicht, und ein falsches Token
kostet eine Sekunde Wartezeit und eine Warnung im Log — Raten ist langsam und hinterlässt
eine Spur.

## Browser: Sitzungen {#sessions}

Ein Browser, der nicht angemeldet ist, wird auf die Anmeldeseite geleitet. Sein Token wird einmal
gegen eine Sitzung getauscht, die in einem Cookie gehalten wird, das:

- `HttpOnly` ist — kein Skript in einer Seite kann es lesen;
- `SameSite=Strict` ist — keine Seite einer anderen Website kann den Browser dazu bringen, es zu senden;
- 7 Tage gültig ist;
- mit `--secure-cookie` zusätzlich `Secure` ist und so nur über HTTPS übertragen wird (setzen Sie
  es hinter einem HTTPS-Proxy).

Sitzungen liegen im Arbeitsspeicher des Servers: Ein Neustart meldet alle ab, und
[[ui:app.signOut]] beendet eine sofort. Höchstens 1024 Sitzungen werden gehalten; die älteste
fällt zuerst weg.

## Skripte: das Bearer-Token {#bearer}

Ein Skript, `signallab --server` und CI senden das Token mit jeder Anfrage:

```http
Authorization: Bearer <token>
```

Jeder Endpunkt braucht das Token oder eine Sitzung, außer `GET /api/health` (ob
der Server antwortet, seine Version, ob er ein Token verlangt) und der Anmeldeseite. Eine Anfrage an die API
ohne beides erhält `401` mit dem Fehler `auth.required`;
eine Seite erhält die Anmeldeseite.

## Hostnamen {#hosts}

| Server gestartet | Hostnamen, auf die er antwortet |
| --- | --- |
| ohne Token | Loopback-Namen und die Namen in `--allowed-host` |
| mit Token, ohne `--allowed-host` | jeder Name |
| mit Token und `--allowed-host` | Loopback-Namen und die Namen in `--allowed-host` |

`--allowed-host` (`SIGNALLAB_ALLOWED_HOSTS`) nimmt durch Kommas getrennte Namen an,
verglichen ohne den Port und ohne Beachtung der Groß- und Kleinschreibung:

```bash
signal-lab-server --listen 0.0.0.0:1430 --token-file token.txt --allowed-host lab-pc.example.com,192.0.2.10
```

Jeder andere `Host` erhält `403` mit dem Fehler `auth.host`. Setzen Sie es bei einem Server, der
unter bekannten Namen erreichbar ist, damit eine Seite einer anderen Website ihn nicht
über einen eigenen Namen erreichen kann.

## Origin und Content-Type {#origin}

- Jede Anfrage, die etwas ändert (alles außer `GET` und `HEAD`), und der
  WebSocket für Ereignisse müssen entweder keinen `Origin` tragen oder den des Servers — denselben
  Host und Port wie sein `Host`. Eine Seite einer anderen Website oder eine, die
  `Origin: null` sendet, erhält `403` mit `auth.origin`. Skripte und `curl` senden keinen
  `Origin` und sind davon nicht betroffen.
- Befehle nehmen nur `Content-Type: application/json` an (sonst `415` mit
  `command.json_required`), damit ein Formular auf einer anderen Website keinen senden kann.
- Der Server beantwortet keine Cross-Origin-Anfragen (CORS).

## Antwort-Header {#headers}

Jede Antwort trägt:

| Header | Wert |
| --- | --- |
| `Content-Security-Policy` | `default-src 'self'; connect-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; object-src 'none'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'` |
| `X-Content-Type-Options` | `nosniff` |
| `X-Frame-Options` | `DENY` |
| `Referrer-Policy` | `same-origin` |
| `Cache-Control` | `no-store` für die API und die Anmeldeseite |

Die Oberfläche lädt nichts außer dem, was der Server ausliefert, spricht mit nichts außer
dem Server und lässt sich nicht von einer anderen Seite einbetten.

## Dateien und Größen {#files}

- Ein Download (`GET /api/files?path=…`) wird nur aus dem Datenordner heraus
  ausgeliefert, höchstens 256 MiB; alles andere ergibt `404`.
- Ein Anfrage-Body ist höchstens 24 MiB groß.

## Geheimnisse {#secrets}

Der Wert eines Geheimnisses verlässt die Engine nie:

- Auf einem Server sind Werte **schreibgeschützt**: die Umgebungsvariable
  `SIGNALLAB_SECRET_<NAME>` oder die Datei `<NAME>` im Ordner der Geheimnisse
  (standardmäßig `/run/secrets/signallab`). Einen aus dem Browser zu setzen oder zu entfernen wird
  abgelehnt (`secret.read_only`), damit ein in eine Seite getippter Wert nie
  an einer schwächer geschützten Stelle landet. Siehe [Geheimnisse](index.md#secrets).
- Kein Befehl gibt einen Wert zurück; die Oberfläche erfährt nur, ob ein Name gesetzt ist.
- Experimente nennen Geheimnisse `{{secret.NAME}}`. Solange ein Durchlauf oder ein Senden sie
  verwendet, zeigt jeder Text, den er meldet — Schritte, Fehler, der Bericht des Durchlaufs — an ihrer Stelle `••••`,
  und Frames im [[ui:dock.inspector]] werden Byte für Byte maskiert.
- Die Zugangsdaten eines HTTP-Knotens werden erst beim Senden der Anfrage zu einem
  `Authorization`-Header; Schritte, Frames und Berichte tragen die Antwort, nie diesen
  Header.

## Was protokolliert wird {#audit}

Das Log hält mit der Adresse des Clients fest: jeden gestarteten Job — Stürme, Scans,
Broadcasts, Monitore, Generatoren, Durchläufe, Emulatoren —, jede Anmeldung und jede
Anmeldung mit einem falschen Token. Siehe [Logs](index.md#logs).

## Was der Server nie tut {#never}

- **Die Firewall des Hosts ändern.** Die Desktop-App kann eine Firewall-Regel anlegen, wenn
  Sie es verlangen; auf einem Server wird dieser Befehl abgelehnt (`firewall.server`). Die
  Firewall des Hosts gehört dem, der den Host betreibt. (Das Ein-Befehl-Installationsskript
  bietet an, den Port des Servers in ufw oder firewalld zu öffnen, und fragt vorher — siehe
  [Die Firewall des Hosts](index.md#firewall).)
- **Ohne Token erreichbar starten**, auf jeder Adresse außer Loopback.
- **Eine Datei von außerhalb seines Datenordners ausliefern.**
- **Ein Geheimnis speichern**, das in einen Browser getippt wurde.
- **Selbst TLS sprechen:** Setzen Sie einen HTTPS-Proxy davor (siehe
  [Hinter einem HTTPS-Proxy](index.md#https)).

Jede Grenze der Engine — höchstens 1024 Hosts in einem Sweep, höchstens 50.000
Pakete pro Sekunde von einem Beacon — gilt auf einem Server wie in der App. Es sind
Leitplanken, keine Erlaubnis: Senden Sie Verkehr nur an Systeme, die Ihnen gehören oder die Sie testen dürfen.

## Der Container {#container}

Das Image läuft als unprivilegierter Benutzer (uid und gid 10001) und schreibt nur nach
`/data`. Es läuft unverändert mit schreibgeschütztem Root-Dateisystem, ohne Capabilities
und mit `no-new-privileges` — so, wie das Installationsskript und `deploy/compose.yaml`
es starten. Jedes Image wird mit einer SBOM, einer Build-Provenienz und einer signierten
GitHub-Attestierung veröffentlicht:

```bash
gh attestation verify oci://ghcr.io/proanima/signallab:[[version]] -R ProAnima/SignalLab
```

## Was Signal Lab nach außen sendet {#outside}

Neben dem Verkehr, den Sie senden, spricht Signal Lab mit zwei Stellen, beide gehören dem Studio.

### Update-Prüfungen {#update-check}

Nur die **Desktop-App** sucht nach Updates; ein Server und ein Browser tun es nie.
Ist [[ui:update.auto]] an (unter [[ui:about.open]]), fragt die App einmal am Tag und jedes Mal,
wenn Sie [[ui:update.check]] drücken, beim Hub des Studios
(`hub.proanima.net`) nach — und beim neuesten Release auf GitHub nur dann, wenn der Hub nicht
erreichbar ist. Die Anfrage enthält:

- die Version der App;
- das Betriebssystem und die Prozessorarchitektur;
- eine Zufallszahl dieser Installation (`X-Install-Id`), einmal erzeugt und bei den
  Einstellungen der App aufbewahrt, damit ein neues Release zunächst einen Teil der Installationen erreichen kann. Sie sagt
  nichts über Sie oder den Computer.

Angeboten werden nur veröffentlichte Releases. Ein Download, dessen Signatur nicht zu dem
in die App eingebauten Schlüssel passt, wird nicht installiert, und nichts wird installiert, bis Sie
[[ui:update.install]] drücken.

### Feedback {#feedback}

[[ui:feedback.open]] (das ✉ in der Kopfzeile, auch unter [[ui:about.open]]) sendet eine Nachricht an die
Entwickler über den Hub des Studios, der sie per E-Mail weiterleitet; die App enthält dafür
kein Passwort. Sie sendet nur, was das Formular zeigt: Ihre Nachricht, Ihre E-Mail-Adresse, wenn
Sie eine angeben, die Screenshots, die Sie hinzufügen, und — unter [[ui:feedback.logs]] — das
Log der Konsole und [[ui:feedback.systemInfo]], die Sie beide vor dem Senden öffnen und
abwählen können. Der Name dieses Computers, seine Adresse und Ihre Ordner sind darin weggelassen.
Aus einem Browser sendet der Server das Formular.

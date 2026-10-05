---
title: Installieren und aktualisieren
description: Die Desktop-App von Signal Lab unter Windows oder Linux installieren, einen Download prüfen, die App aktuell halten und wieder entfernen.
---

# Installieren und aktualisieren

Signal Lab wird auf GitHub veröffentlicht: Jedes Release enthält die Desktop-App für Windows und
Linux, die Kommandozeile `signallab` einzeln und eine Liste von Prüfsummen. Um Signal Lab stattdessen auf einem
Server zu betreiben und im Browser zu nutzen, siehe [Der Server](../server/index.md).

## Was Sie herunterladen {#downloads}

Öffnen Sie das [neueste Release](https://github.com/ProAnima/SignalLab/releases/latest) und wählen Sie
die Datei für Ihr System. `<version>` ist die Version des Release, etwa `[[version]]`.

| Datei | Für |
| --- | --- |
| `Signal.Lab_<version>_x64-setup.exe` | Windows 10 und 11, x64 — das Setup, das die meisten brauchen |
| `Signal.Lab_<version>_x64_en-US.msi` | Windows, Verteilung durch die IT (Intune, Gruppenrichtlinie): installiert für alle Benutzer |
| `Signal.Lab_<version>_amd64.deb` | Linux x86_64: Debian, Ubuntu und verwandte Distributionen |
| `Signal.Lab-<version>-1.x86_64.rpm` | Linux x86_64: Fedora, RHEL, openSUSE und verwandte Distributionen |
| `Signal.Lab_<version>_amd64.AppImage` | Linux x86_64, jede Distribution, ohne Installation |
| `signallab-<version>-windows-x64.zip` | nur die Kommandozeile, für Windows |
| `signallab-<version>-linux-x64.tar.gz` | nur die Kommandozeile, für Linux |
| `SHA256SUMS.txt` | die SHA-256-Prüfsumme jeder obigen Datei |

Die `.sig`-Dateien und `latest.json` daneben sind für den eigenen Updater der App bestimmt; Sie
brauchen sie nicht.

## Windows {#windows}

Signal Lab läuft unter Windows 10 und 11, x64. Es verwendet die WebView2-Laufzeit, die zu
beiden gehört.

### Mit dem Setup installieren {#windows-setup}

1. Führen Sie `Signal.Lab_<version>_x64-setup.exe` aus.
2. Meldet Windows SmartScreen, es habe Ihren PC geschützt, wählen Sie **Weitere Informationen** und dann
   **Trotzdem ausführen** (siehe [SmartScreen](#smartscreen) unten).
3. Wählen Sie die Sprache des Setups. Es bietet dieselben 11 Sprachen wie die App.
4. Wählen Sie, für wen installiert wird:
   - Nur für Sie: Das Setup installiert nach `%LOCALAPPDATA%\Signal Lab` und braucht keine
     Administratorrechte.
   - Für alle Benutzer dieses Computers: Das Setup installiert nach `C:\Program Files\Signal Lab`
     und fordert Administratorrechte an.
5. Bestätigen Sie den Ordner und installieren Sie.
6. Die letzte Seite bietet an, Signal Lab sofort zu starten und eine Verknüpfung auf dem Desktop anzulegen.

Wenn Sie das Setup einer neueren Version über eine installierte ausführen, wird diese an Ort und Stelle aktualisiert.

### Was das Setup hinzufügt {#windows-setup-adds}

Neben der App richtet das Setup zwei Dinge ein, und das Deinstallationsprogramm entfernt beide:

- **Die Kommandozeile.** `signallab.exe` kommt neben die App und ihr Ordner in den `PATH` —
  Ihren eigenen `PATH` bei einer Installation nur für Sie, den des Computers bei einer Installation für
  alle —, sodass `signallab` in jedem Terminal funktioniert, das Sie danach öffnen. Siehe
  [Die Kommandozeile](../automation/cli.md).
- **Eine Firewall-Regel**, nur bei einer Installation für alle. Die Windows Defender Firewall lässt
  andere Rechner ein Programm nur erreichen, wenn eine Regel es erlaubt, und fragt die Person am
  Bildschirm, sobald das Programm zum ersten Mal auf einem Port empfängt; ein *Abbrechen* dort verwirft stillschweigend alles,
  was andere Rechner senden. Eine Installation für alle, die Administratorrechte hat, legt eine
  eingehende Zulassungsregel für `signal-lab.exe` (die App) und eine für `signallab.exe` (die
  Kommandozeile) an, für private Netzwerke und Domänennetzwerke. Eine Installation nur für Sie kann
  die Firewall nicht ändern: Die App bietet die Änderung mit der Administratorabfrage von Windows an, sobald
  etwas zum ersten Mal auf einem Port empfängt — siehe [den Firewall-Hinweis](interface.md#firewall-notice).

Verkehr auf diesem Computer selbst (`127.0.0.1`) wird nie gefiltert, daher funktioniert alles, was Sie über
Loopback tun, ohne jede Regel.

### Unbeaufsichtigte Installation {#windows-unattended}

Das Setup nimmt diese Schalter auf seiner Kommandozeile an:

| Schalter | Wirkung |
| --- | --- |
| `/S` | Installiert still, ohne Rückfragen. |
| `/P` | Installiert mit Fortschrittsanzeige und ohne Rückfragen. |
| `/ALLUSERS` | Installiert für alle (braucht eine Eingabeaufforderung mit erhöhten Rechten). |
| `/CURRENTUSER` | Installiert nur für den aktuellen Benutzer. |
| `/NS` | Legt keine Verknüpfungen an. |
| `/D=C:\Tools\Signal Lab` | Installiert in diesen Ordner. Muss als Letztes stehen und wird nicht in Anführungszeichen gesetzt. |
| `/NOPATH` | Lässt `PATH` unverändert: `signallab` wird installiert, liegt aber nicht im `PATH`. |
| `/NOFIREWALL` | Legt keine Firewall-Regeln an. |

Zum Beispiel eine stille Installation für alle ohne Firewall-Regeln, aus einer Eingabeaufforderung
mit erhöhten Rechten:

```powershell
.\Signal.Lab_[[version]]_x64-setup.exe /S /ALLUSERS /NOFIREWALL
```

### Das MSI {#windows-msi}

`Signal.Lab_<version>_x64_en-US.msi` ist für die Verteilung durch die IT gedacht. Es installiert für alle Benutzer
und legt `signallab.exe` neben die App und diesen Ordner in den `PATH` des Computers, legt aber
keine Firewall-Regel an:
Das bleibt dem überlassen, der es verteilt (etwa per Gruppenrichtlinie). Die App bietet die
Regel trotzdem an, sobald sie zum ersten Mal empfängt. Zur stillen Installation:

```powershell
msiexec /i "Signal.Lab_[[version]]_x64_en-US.msi" /qn
```

### SmartScreen {#smartscreen}

Die Installer sind noch nicht codesigniert, daher kennt Windows SmartScreen ihren
Herausgeber nicht und hält das Setup womöglich mit einer Warnung an. Wählen Sie **Weitere Informationen**, prüfen Sie, dass die
Datei die ist, die Sie von der Release-Seite heruntergeladen haben, und wählen Sie dann **Trotzdem ausführen**. Um
sicherzugehen, dass die Datei unverändert ist, [prüfen Sie sie vorher gegen `SHA256SUMS.txt`](#checksums).

Die eigenen Updates der App sind signiert und werden gesondert geprüft; siehe [Updates](#updates).

## Einen Download prüfen {#checksums}

`SHA256SUMS.txt` listet die SHA-256-Prüfsumme jeder Datei des Release auf, eine pro Zeile,
gefolgt vom Dateinamen. Laden Sie sie neben die Datei herunter und vergleichen Sie.

Unter Windows, in PowerShell:

```powershell
Get-FileHash .\Signal.Lab_[[version]]_x64-setup.exe -Algorithm SHA256
Select-String "Signal.Lab_[[version]]_x64-setup.exe" .\SHA256SUMS.txt
```

Die beiden Hashes müssen gleich sein (`Get-FileHash` gibt ihn in Großbuchstaben aus; die Groß- und Kleinschreibung
spielt keine Rolle).

Unter Linux, in dem Ordner, der beide Dateien enthält:

```bash
sha256sum --check --ignore-missing SHA256SUMS.txt
```

Es gibt nach jeder gefundenen Datei `OK` aus. Alles andere bedeutet, dass der Download nicht die
veröffentlichte Datei ist: Laden Sie sie erneut herunter.

## Linux {#linux}

Die Linux-App ist für x86_64 gebaut (unter Ubuntu 22.04) und braucht WebKitGTK 4.1, die Webansicht,
mit der sie ihr Fenster zeichnet. Für Arm-Prozessoren gibt es keinen Build; auf einem Arm-Rechner
betreiben Sie stattdessen den [Server](../server/index.md).

### Das .deb-Paket {#linux-deb}

```bash
sudo apt install ./Signal.Lab_[[version]]_amd64.deb
```

`apt` installiert mit, was das Paket braucht. Das Paket legt außerdem die Kommandozeile
unter `/usr/bin/signallab` ab.

### Das .rpm-Paket {#linux-rpm}

```bash
sudo dnf install ./Signal.Lab-[[version]]-1.x86_64.rpm
```

Unter openSUSE verwenden Sie `sudo zypper install` mit derselben Datei. Wie das `.deb` legt das Paket
die Kommandozeile unter `/usr/bin/signallab` ab.

### Das AppImage {#linux-appimage}

Das AppImage läuft ohne Installation:

```bash
chmod +x Signal.Lab_[[version]]_amd64.AppImage
./Signal.Lab_[[version]]_amd64.AppImage
```

Es enthält die Kommandozeile `signallab` nicht; nehmen Sie sie bei Bedarf aus dem
[Archiv der Kommandozeile](#cli-archives).

### Die Firewall unter Linux {#linux-firewall}

Unter Linux zeigt die App keinen Firewall-Hinweis: `ufw` und `firewalld` arbeiten nach Port, nicht nach
Programm. Ist eine davon aktiv und müssen andere Rechner einen Monitor, einen Empfänger, einen
Emulator oder ein Relais erreichen, öffnen Sie dort dessen Port, zum Beispiel:

```bash
sudo ufw allow 9000/udp
```

Loopback-Verkehr ist davon nicht betroffen.

## Die Kommandozeile einzeln {#cli-archives}

Die Desktop-Installer bringen `signallab` mit. Für einen Rechner ohne die App — einen
Build-Agenten, einen Server, einen Labor-PC — enthält jedes Release die Kommandozeile auch einzeln:

- `signallab-<version>-windows-x64.zip` enthält `signallab.exe` und die Lizenz.
- `signallab-<version>-linux-x64.tar.gz` enthält `signallab` und die Lizenz.

Entpacken Sie es in einen Ordner in Ihrem `PATH`. Unter Linux:

```bash
tar -xzf signallab-[[version]]-linux-x64.tar.gz signallab
sudo install signallab /usr/local/bin/
signallab version
```

Das Docker-Image des Servers enthält sie ebenfalls. Wie Sie sie verwenden: [Die Kommandozeile](../automation/cli.md).

## Updates {#updates}

### Die Desktop-App {#updates-desktop}

Die Desktop-App aktualisiert sich nur aus veröffentlichten Releases und nur, wenn Sie es veranlassen.

- **Sie sucht einmal täglich.** Ist [[ui:update.auto]] angehakt (das ist die Voreinstellung), sucht die App
  kurz nach dem Start nach einem neueren Release, wenn die letzte Suche einen Tag oder länger
  zurückliegt, und erneut jedes Mal, wenn im laufenden Betrieb ein weiterer Tag vergangen ist. Entfernen Sie den Haken unter „Über Signal Lab“,
  damit sie nicht mehr von selbst sucht.
- **Sie können sofort suchen.** Öffnen Sie „Über Signal Lab“ — das **?** in der Kopfzeile — und drücken Sie
  unter [[ui:update.title]] auf [[ui:update.check]].
- **Was die Prüfung sendet.** Sie fragt beim Update-Dienst des Studios (`hub.proanima.net`) an,
  welches Release diese Installation erhalten soll, und wendet sich nur dann an das neueste Release auf GitHub, wenn
  dieser Dienst nicht erreichbar ist. Die Anfrage übermittelt die Version der App, das System
  und den Prozessortyp sowie eine für diese Installation erzeugte Zufallszahl, mit der ein neues
  Release zunächst einen Teil der Installationen erreichen kann. Nichts, was Sie oder diesen Computer benennt, wird
  gesendet.
- **Wird ein Release gefunden**, meldet die Konsole es, und die Kopfzeile zeigt eine Update-Schaltfläche,
  die „Über Signal Lab“ öffnet. Dort zeigt [[ui:update.notes]] die Versionshinweise.
- **Nichts wird installiert, bevor Sie klicken.** Drücken Sie [[ui:update.install]] (laufen gerade
  Jobs, sagt die Schaltfläche, dass sie sie stoppt). Signal Lab speichert alles, was noch
  aussteht, stoppt jeden laufenden Job, lädt das Update herunter und prüft seine Signatur
  gegen den in die App eingebauten Schlüssel — ein Update, das nicht vom Release-Prozess signiert ist,
  wird abgelehnt —, installiert es dann und startet als neue Version neu.

Unter Windows führt das Update das Setup ohne Rückfragen aus und behält Ihren Ordner, den `PATH`
und die Firewall-Regeln bei. Unter Linux ersetzt es das AppImage oder installiert das neue `.deb`- oder
`.rpm`-Paket — bei einem Paket fragt das System zuerst nach Ihrem Passwort.

Vorabversionen und Entwürfe werden nie angeboten. Damit keine Installation eines Rechners
von selbst sucht — etwa wenn die IT Updates selbst verteilt —, setzen Sie die Umgebungsvariable
`SIGNALLAB_NO_UPDATE_CHECK` auf einen beliebigen Wert; die Schaltfläche [[ui:update.check]] funktioniert weiterhin.
Entwicklungs-Builds suchen nie von selbst.

### Ein Server {#updates-server}

Ein Server und die Seite, die ein Browser von ihm anzeigt, werden mit seinem Docker-Image aktualisiert, nie von
selbst: „Über Signal Lab“ weist darauf hin. Wie Sie einen Server aktualisieren: [Der Server](../server/index.md).

## Deinstallieren {#uninstall}

**Windows.** Öffnen Sie *Einstellungen → Apps → Installierte Apps*, suchen Sie Signal Lab und wählen Sie
*Deinstallieren*. Das Deinstallationsprogramm des Setups nimmt `signallab.exe` aus dem `PATH` und entfernt, wenn es
mit Administratorrechten läuft (wie bei einer Installation für alle), jede eingehende
Firewall-Regel für die App und die Kommandozeile, auch solche, die die Windows-Abfrage angelegt hat.
Es bietet ein Kästchen an, auch die Anwendungsdaten zu löschen: Das sind die eigenen Einstellungen der App
— die Sprache, die Größen der Bereiche, die Ansicht, in der Sie zuletzt waren, die zuletzt in die
Ansichten eingegebenen Werte —, nicht Ihre Dateien. Eine MSI-Installation wird ebenso entfernt; sie nimmt ihren `PATH`-Eintrag
mit und überlässt die Firewall dem, der sie verteilt hat.

**Linux.** Entfernen Sie das Paket mit dem Paketmanager, mit dem Sie es installiert haben, oder löschen Sie
die AppImage-Datei.

Keines von beiden rührt Ihren **Datenordner** an: Experimente, die Signal- und die Emulator-Bibliothek, Berichte
der Durchläufe und Exporte bleiben in `Documents/SignalLab` in Ihrem Benutzerordner. Löschen Sie diesen Ordner
selbst, wenn Sie sie loswerden möchten.

## Wo Ihre Daten liegen {#data}

Alles, was Sie erstellen, wird als einfache Dateien in einem Ordner aufbewahrt, `Documents/SignalLab` in Ihrem
Benutzerordner, unter Windows wie unter Linux: das aktuelle Experiment, die Signalbibliothek, die
Emulator-Bibliothek, Berichte der Durchläufe, Exporte und Mitschnitte des Inspektors. Jede Datei und ihr Format
sind unter [Dateien und Ordner](../reference/files.md) beschrieben.

## Als Server betreiben {#server}

Um Signal Lab aus einem Browser zu nutzen — auf einem Labor-PC neben der Technik, auf einem Linux-Rechner, in
Docker —, siehe [Der Server](../server/index.md). Auf einem Linux-Rechner mit Docker installiert und startet ein
einziger Befehl ihn.

---
title: CI-Pipelines
description: Führen Sie Signal-Lab-Experimente in GitHub Actions, GitLab CI oder jeder Pipeline aus, lassen Sie den Job bei einem Fehlschlag scheitern und behalten Sie einen JUnit-Bericht.
---

# Signal Lab in der CI

Die Experimente, die eine Installation hochbringen, sind auch ihre
Regressionstests. In einer Pipeline führt [`signallab run`](cli.md#cli-run) sie
ohne Fenster aus, gibt jeden Schritt aus, schreibt einen JUnit-Bericht, den jedes
CI-System zeigt, und endet mit einem Code, den der Job versteht:

| Exit-Code | Die Pipeline sollte |
| --- | --- |
| `0` | weitermachen: Jedes Experiment bestand |
| `1` | scheitern: Ein Experiment lief und schlug fehl |
| `2` | scheitern: Ein Experiment oder der Befehl ist falsch (ein Validierungsfehler, ein unbekannter Parameter, ein fehlendes Geheimnis) |
| `3` | scheitern oder erneut versuchen: Nichts konnte laufen (der Server ist nicht erreichbar oder verweigert das Token, ein Port lässt sich nicht öffnen) |

Es gibt vier Wege hinein:

| Weg | Wo es läuft |
| --- | --- |
| [Die GitHub Action](#github-actions) | Ein Linux-Runner, aus dem Server-Image. |
| [Das Image](#docker) | Jede CI, die Container ausführt: GitLab, Jenkins, eine Shell mit Docker. |
| [Die Binärdatei](#binary) | Jeder Runner, Windows eingeschlossen. |
| [Ein Laborserver](#lab-server) | Die Durchläufe finden auf einem Signal-Lab-Server neben der Technik statt; die Pipeline sendet sie nur. |

## GitHub Actions {#github-actions}

Das Repository ist auch eine GitHub Action. Sie führt `signallab` aus dem Image
`ghcr.io/proanima/signallab` aus, lässt den Job scheitern, wenn ein Experiment
fehlschlägt, und hinterlässt einen JUnit-Bericht:

```yaml
jobs:
  signallab:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: ProAnima/SignalLab@v[[version]]
        with:
          version: [[version]]
          experiments: tests/signallab/*.json
          params: |
            api=http://127.0.0.1:8080
        env:
          SIGNALLAB_SECRET_API_TOKEN: ${{ secrets.API_TOKEN }}

      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: signallab-junit
          path: signallab-junit.xml
```

Ein Durchlauf, der nicht besteht, ist außerdem eine Fehlerannotation auf der
Seite des Durchlaufs, neben dem fehlgeschlagenen Schritt, mit dem Experiment und
warum es fehlschlug.

### Eingaben {#action-inputs}

| Eingabe | Was sie tut | Standard |
| --- | --- | --- |
| `experiments` | Experimentdateien oder Namen mitgelieferter Vorlagen, durch Leerzeichen oder Zeilen getrennt. Muster wie `tests/*.json` werden erweitert; ein Pfad mit Leerzeichen darin wird nicht unterstützt. | erforderlich |
| `params` | Parameterwerte, `NAME=VALUE`, einer pro Zeile. | |
| `profile` | Mit diesem Profil der Experimente ausführen. | |
| `matrix` | Einmal pro Kombination ausführen: `NAME=V1,V2`, ein Name pro Zeile. | |
| `matrix-file` | Kombinationen aus einer JSON-Datei (siehe [Eine Matrix von Durchläufen](#matrix)). | |
| `fail-fast` | `"true"`: Beim ersten Durchlauf anhalten, der nicht besteht. | `"false"` |
| `server` | Auf diesem Signal-Lab-Server statt im Job ausführen, z. B. `http://192.0.2.10:1430`. | |
| `token` | Das Zugriffstoken des Servers. Übergeben Sie ein Geheimnis. | |
| `junit` | Wohin der JUnit-Bericht geht. | `signallab-junit.xml` |
| `timeout` | Sekunden, die ein Durchlauf dauern darf, 1 bis 300. | `300` |
| `version` | Der Tag des Images. | `latest` |
| `image` | Eine andere Registry oder ein lokal gebautes Image; `version` ist sein Tag. | `ghcr.io/proanima/signallab` |
| `lang` | Die Sprache der Nachrichten und des Berichts: `en`, `ru`, `es`, `fr`, `de`, `pt`, `zh`, `ja`, `ko`, `hi` oder `ar`. | `en` |
| `fail-on-error` | `"false"`: Den Schritt nicht scheitern lassen; stattdessen `exit-code` lesen. | `"true"` |

Pfade (`experiments`, `matrix-file`, `junit`) sind relativ zum Arbeitsverzeichnis
des Schritts.

::: tip
Fixieren Sie `version` auf die Veröffentlichung, mit der Sie getestet haben.
`latest` wandert mit jeder neuen stabilen Veröffentlichung.
:::

### Ausgaben {#action-outputs}

| Ausgabe | Was sie ist |
| --- | --- |
| `junit` | Der Pfad des JUnit-Berichts. |
| `exit-code` | Der Exit-Code von `signallab`: `0`, `1`, `2` oder `3`. |

### Nach einem Fehlschlag weitermachen {#fail-on-error}

Ein Schritt, der fehlschlägt, gibt keine Ausgaben an den Rest des Jobs weiter. Um
selbst zu entscheiden, setzen Sie `fail-on-error: "false"` und verzweigen Sie auf
`exit-code`:

```yaml
      - id: lab
        uses: ProAnima/SignalLab@v[[version]]
        with:
          experiments: tests/signallab/smoke.json
          fail-on-error: "false"

      - if: steps.lab.outputs.exit-code == '1'
        run: echo "an experiment failed; the report is ${{ steps.lab.outputs.junit }}"

      - if: steps.lab.outputs.exit-code != '0'
        run: exit 1
```

### Geheimnisse in der Action {#action-secrets}

Das `{{secret.NAME}}` eines Experiments liest `SIGNALLAB_SECRET_NAME`. Setzen Sie
diese Variablen am Schritt (`env:`) aus den Geheimnissen des Repositorys. Die
Action übergibt jede `SIGNALLAB_SECRET_…`-Variable ihres Schritts namentlich an
`signallab`; die Werte reisen in der Umgebung, nie auf einer Kommandozeile, und
jeder Bericht und jede Logzeile zeigt an ihrer Stelle `••••`. Ein Geheimnis, das
das Experiment braucht und das nicht gesetzt ist, lässt den Schritt mit Exit-Code
`2` scheitern, bevor etwas gesendet wird.

Die Eingabe `token` reist genauso, als `SIGNALLAB_TOKEN`.

### Wie die Action läuft {#action-runs}

- Sie braucht einen **Linux-Runner** mit Docker (`ubuntu-latest` hat es). Auf
  einem Windows- oder macOS-Runner stoppt sie mit Exit-Code `2` und einer
  Annotation; verwenden Sie dort [die Binärdatei](#binary).
- `signallab` läuft im Image mit **Host-Netzwerk**: Was der Runner erreicht,
  erreicht es. Ein Dienst, den der Job auf dem Runner gestartet hat — Ihr System
  unter Test oder ein `services:`-Container mit einem veröffentlichten Port —,
  ist unter `127.0.0.1`.
- Sie läuft als Benutzer des Runners, mit dem Arbeitsbereich unter demselben Pfad
  eingebunden, sodass der Bericht dem Job gehört.
- Sie übergibt `run` genau die obigen Eingaben und `--junit`. Für alles andere,
  was `signallab` kann — `--report`, `--seed`, `--json`, `emulate` —, verwenden
  Sie [das Image](#docker) direkt.

## Das Image, in jeder CI {#docker}

Das Server-Image enthält `signallab` als `/usr/local/bin/signallab`. Überschreiben
Sie den Entrypoint, um es zu verwenden.

**GitLab CI:**

```yaml
signallab:
  image:
    name: ghcr.io/proanima/signallab:[[version]]
    entrypoint: [""]
  script:
    - signallab run tests/signallab/*.json --junit signallab-junit.xml
  artifacts:
    when: always
    reports:
      junit: signallab-junit.xml
```

Geheimnisse sind CI/CD-Variablen namens `SIGNALLAB_SECRET_<NAME>` (markieren Sie
sie als maskiert); die Umgebung des Jobs übergibt sie `signallab`, wie sie sind.

**Docker, aus einer Shell oder einem beliebigen Scheduler** (cron, Jenkins, ein
Deploy-Skript):

```bash
docker run --rm --network host \
  --user "$(id -u):$(id -g)" \
  -v "$PWD:/work" -w /work \
  -e SIGNALLAB_SECRET_API_TOKEN \
  --entrypoint signallab \
  ghcr.io/proanima/signallab:[[version]] \
  run tests/smoke.json --junit junit.xml
echo "signallab exited with $?"
```

- `--network host` lässt die Durchläufe erreichen, was der Host erreicht,
  Broadcast und Multicast eingeschlossen — auf einem Linux-Host. Ohne es erreicht
  der Container andere Hosts nur per Unicast.
- Das Image läuft als unprivilegierter Benutzer (uid 10001). `--user` führt es
  stattdessen als Sie aus, sodass es den Bericht in Ihren Ordner schreiben kann;
  ohne es muss der Ordner für uid 10001 beschreibbar sein.
- `-e NAME` ohne Wert übergibt diese Variable aus Ihrer Umgebung.

## Die Binärdatei {#binary}

Jede Veröffentlichung enthält `signallab` für sich:
`signallab-<version>-linux-x64.tar.gz` und
`signallab-<version>-windows-x64.zip`, aufgeführt in `SHA256SUMS.txt` der
Veröffentlichung. Die Linux-Version ist auf Ubuntu 22.04 gebaut und verwendet das
systemeigene OpenSSL 3 (`libssl3`): Sie läuft auf dieser Veröffentlichung oder
einer neueren Distribution.

```yaml
      - name: Signal Lab
        run: |
          curl -fsSL -o signallab.tar.gz https://github.com/ProAnima/SignalLab/releases/download/v[[version]]/signallab-[[version]]-linux-x64.tar.gz
          tar -xzf signallab.tar.gz
          ./signallab run tests/signallab/smoke.json --junit signallab-junit.xml
```

Entpacken Sie auf einem Windows-Runner das Zip und führen Sie `signallab.exe`
genauso aus. Ein Rechner mit installierter Desktop-App hat `signallab` bereits in
seinem `PATH`.

## Durchläufe auf einem Laborserver {#lab-server}

Technik im Netzwerk einer Installation ist vom Labor aus erreichbar, nicht von
einem Cloud-Runner. Betreiben Sie dort einen [Signal-Lab-Server](../server/index.md),
bewahren Sie sein Token als CI-Geheimnis auf und senden Sie die Durchläufe an ihn:

```bash
signallab run tests/stage.json --server http://192.0.2.10:1430 --token-file token.txt --junit junit.xml --report reports/
```

```yaml
      - uses: ProAnima/SignalLab@v[[version]]
        with:
          experiments: tests/signallab/stage.json
          server: http://192.0.2.10:1430
          token: ${{ secrets.SIGNALLAB_TOKEN }}
```

Das Experiment kommt aus dem Checkout der Pipeline; der Server führt es mit
seinem eigenen Netzwerk, seinen Geheimnissen und seinem Datenordner aus,
überträgt die Schritte zurück und behält den Bericht (`--report` lädt eine Kopie
herunter). Das Token kommt aus `--token-file` oder `SIGNALLAB_TOKEN`. Die
Exit-Codes sind dieselben; ein Server, der nicht erreichbar ist oder das Token
verweigert, ist `3`. Der Runner muss den Server erreichen können — ein selbst
gehosteter Runner im Labor oder eine Serveradresse, die der Runner öffnen kann.

Um ohne `signallab` auf dem Server zu laufen, kann ein Skript seine HTTP-API
direkt aufrufen: siehe [Ein Experiment über HTTP ausführen](../api/run.md).

## Eine Matrix von Durchläufen {#matrix}

Ein Experiment, jedes Ziel: Jede Kombination ist ein eigener Durchlauf und eine
eigene Testsuite im JUnit-Bericht, benannt nach ihren Werten.

```bash
signallab run tests/smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest
```

In der Action ein Name pro Zeile:

```yaml
        with:
          experiments: tests/signallab/smoke.json
          matrix: |
            device=192.0.2.20:9000,192.0.2.21:9000
            user=admin,guest
          fail-fast: "true"
```

Oder aus einer Datei, mit `--matrix-file` (`matrix-file` in der Action):

```json
[
  { "device": "192.0.2.20:9000", "user": "admin" },
  { "device": "192.0.2.21:9000", "user": "guest" }
]
```

Jede Kombination wird geprüft, bevor die erste etwas sendet, höchstens 256
Durchläufe kommen aus einem Befehl, und `--fail-fast` lässt den Rest ungestartet
— sie erscheinen im JUnit-Bericht als übersprungen. Die Regeln stehen auf
[der Seite der Kommandozeile](cli.md#matrix).

Um jedes Profil eines Experiments zu versuchen, führen Sie einmal pro Profil aus
— je ein Schritt oder eine Job-Matrix Ihrer CI —, mit `--profile`.

## JUnit-Berichte {#junit}

`--junit PATH` (die Action schreibt immer einen) enthält eine Testsuite pro
Durchlauf und einen Testfall pro Knoten: den Fehlschlag dort, wo er geschah, in
der gewählten Sprache, mit seinem Fehlercode und technischem Detail; die Knoten,
die ein Durchlauf nie erreichte, als übersprungen; den Startwert, das Ergebnis,
die Datei und die Matrixwerte als Eigenschaften. GitHub (mit einer
Reporting-Action), GitLab (`artifacts:reports:junit`), Jenkins und Azure DevOps
zeigen ihn als Testergebnisse. Seine Struktur steht auf
[der Seite der Kommandozeile](cli.md#reports).

Der Startwert eines fehlgeschlagenen Durchlaufs steht in den Eigenschaften seiner
Suite und im Log: `--seed <that number>` führt ihn erneut mit denselben
Zufallswerten aus.

## Abhängigkeiten, die das System unter Test aufruft {#emulators}

Um Ihr eigenes System gegen eine API, ein Gerät oder einen Broker zu testen, die
in der CI nicht da sind, lassen Sie Signal Lab sie spielen:

- **Innerhalb eines Experiments** spielt ein Knoten [[ui:exp.node.emulator]] die
  Abhängigkeit für einen Durchlauf, und [[ui:exp.node.wait_http]] prüft, was Ihr
  System ihm gesendet hat. Die Ausgabe des Durchlaufs endet mit dem, wonach jeder
  Emulator gefragt wurde. Siehe [Emulatoren](../tools/emulators.md).
- **Um Ihre eigenen Tests herum** antwortet `signallab emulate` im Hintergrund,
  während sie laufen, und seine Zählungen sagen Ihnen, was aufgerufen wurde:

  ```bash
  signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
  npm test
  wait
  ```

## Ausgabe für ein Skript {#json}

`--json` gibt ein JSON-Objekt pro Zeile auf der Standardausgabe aus und nichts
anderes: `started`, jeden `step`, `ended` mit dem ganzen Ergebnis und eine
`summary` mit `total`, `passed`, `failed`, `not_started` und `exit_code`. Fehler
tragen den stabilen `code` der Engine, ein Skript kann also in jeder Sprache
darauf verzweigen. Siehe [Ausgabe](cli.md#output).

```bash
signallab run tests/smoke.json --json | jq -c 'select(.type == "ended") | {file, outcome, seed}'
```

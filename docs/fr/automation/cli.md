---
title: "Ligne de commande"
description: "signallab exécute des expériences sans fenêtre, envoie des messages uniques, joue des émulateurs et vérifie le réseau, depuis un terminal, un script ou une tâche de CI."
---

# La ligne de commande : `signallab`

`signallab` est Signal Lab sans fenêtre. Il exécute les expériences jusqu'au
bout et se termine avec un code qu'un script comprend, envoie un message OSC, un
datagramme, une requête HTTP, un message WebSocket ou une publication MQTT,
envoie un signal de votre bibliothèque, joue un émulateur jusqu'à ce que vous
l'arrêtiez, et dit ce qui se dresse entre cette machine et l'équipement.

C'est le même moteur que l'application : une exécution depuis la ligne de
commande suit les mêmes étapes, écrit le même rapport et dit les mêmes choses,
dans les langues de l'interface. Avec `--server`, les exécutions se font plutôt
sur un [serveur Signal Lab](../server/index.md), avec son réseau et ses secrets.

```bash
signallab run tests/smoke.json --param api=http://127.0.0.1:8080 --junit junit.xml
signallab run flaky-api                                # a bundled template, by name
signallab validate tests/*.json                        # the editor's check, nothing sent
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab fire "Fader value"
signallab emulate tests/orders-api.json --for 120      # play a dependency for two minutes
signallab doctor                                       # firewall, network, data folder
```

Pour les pipelines, voir [Signal Lab en CI](ci.md) ; pour un assistant d'IA,
[`signallab mcp`](mcp.md).

## Installation {#install}

| Où | Comment vous obtenez `signallab` |
| --- | --- |
| Windows, l'installateur (`.exe`) | Installé à côté de l'application, et ce dossier est ajouté au `PATH` — le vôtre pour une installation « pour moi », celui de la machine pour « pour tout le monde ». Ouvrez un nouveau terminal après l'installation. Le commutateur `/NOPATH` de l'installateur laisse `PATH` tel quel. |
| Windows, le `.msi` | Installé à côté de l'application ; le dossier d'installation est dans le `PATH` de la machine tant que l'application est installée. |
| Linux, `.deb` et `.rpm` | `/usr/bin/signallab`. |
| Linux, AppImage | Non inclus : utilisez l'archive ci-dessous. |
| N'importe quelle machine, sans l'application | Chaque version porte `signallab-<version>-windows-x64.zip` et `signallab-<version>-linux-x64.tar.gz`, chacun avec le programme et sa licence ; `SHA256SUMS.txt` sur la même page liste leurs sommes de contrôle. |
| L'image du serveur | `/usr/local/bin/signallab` dans `ghcr.io/proanima/signallab` (voir [CI](ci.md#docker)). |

Vérifiez-le avec :

```bash
signallab version
```

## Commandes {#commands}

| Commande | Ce qu'elle fait |
| --- | --- |
| [`run`](#cli-run) | Exécute les expériences l'une après l'autre ; ne se termine par 0 que si toutes ont réussi. |
| [`validate`](#cli-validate) | Vérifie les expériences comme l'éditeur avant une exécution ; n'envoie rien. |
| [`send`](#cli-send) | Envoie un message : `osc`, `udp`, `http`, `ws` ou `mqtt`. |
| [`fire`](#cli-fire) | Envoie un signal d'une bibliothèque de signaux, par son identifiant ou son nom. |
| [`emulate`](#cli-emulate) | Joue une API HTTP, un appareil OSC, UDP ou TCP, ou un broker MQTT jusqu'à <kbd>Ctrl</kbd>+<kbd>C</kbd> ou `--for`. |
| [`emulators`](#cli-emulators) | Liste les émulateurs de la bibliothèque de l'application. |
| [`templates`](#cli-templates) | Liste les modèles d'expérience intégrés. |
| [`nodes`](#cli-nodes) | Décrit chaque type de nœud, en JSON. |
| [`mcp`](#cli-mcp) | Sert Signal Lab à un assistant d'IA via le Model Context Protocol. |
| [`doctor`](#cli-doctor) | Vérifie le pare-feu, le réseau, le dossier de données et un serveur. |
| [`firewall`](#cli-firewall) | `firewall allow` : laisse d'autres machines atteindre Signal Lab à travers le Pare-feu Windows. |
| [`version`](#cli-version) | Affiche la version. |

`signallab help <command>` ou `signallab <command> --help` affiche les options
d'une commande.

## Options de chaque commande {#global-options}

| Option | Ce qu'elle fait | Par défaut |
| --- | --- | --- |
| `--lang <code>` | La langue des messages : `en`, `ru`, `es`, `fr`, `de`, `pt`, `zh`, `ja`, `ko`, `hi` ou `ar`. | `SIGNALLAB_LANG`, sinon les paramètres régionaux, sinon `en` |
| `--json` | Sortie pour les machines sur stdout plutôt que du texte (voir [Sortie](#output)). | désactivé |
| `-h`, `--help` | L'aide de la commande. | |
| `-V`, `--version` | La version ; avant toute commande, sous la forme `signallab --version`. | |

`--lang` et `--json` peuvent aller n'importe où sur la ligne :
`signallab --json run smoke.json` et `signallab run smoke.json --json` sont
identiques.

### Langue {#language}

Les messages, les textes d'étape, les échecs et le rapport JUnit utilisent les
propres textes de l'interface, ses formes plurielles et son style de nombres. La
langue est le premier de :

1. `--lang` ;
2. `SIGNALLAB_LANG` (`ru`, `ru-RU` et `ru_RU.UTF-8` signifient tous le russe) ;
3. les paramètres régionaux : le premier de `LC_ALL`, `LC_MESSAGES` et `LANG` qui est défini ;
4. l'anglais.

::: tip
Les terminaux Windows ne définissent généralement aucune des variables de
paramètres régionaux, donc `signallab` parle anglais sauf si vous définissez
`SIGNALLAB_LANG` ou passez `--lang`.
:::

### Sortie pour les personnes et pour les machines {#output}

Sans `--json`, les résultats vont sur **stdout** et tout ce qu'une personne lit
en chemin va sur **stderr** : les étapes d'une exécution au fur et à mesure,
pourquoi quelque chose a échoué, où se trouve le rapport.
`signallab run … 2>/dev/null` ne laisse qu'une ligne de verdict par exécution.

Avec `--json`, stdout ne porte que du JSON, et stderr reste silencieux :

| Commande | Ce qu'imprime `--json` sur stdout |
| --- | --- |
| `run` | Un objet par ligne : `started`, un `step` par étape, `ended` (le résultat de l'exécution), puis `summary` ; `error` pour une exécution qui n'a pas pu démarrer. Voir [Ce qu'imprime `run`](#run-output). |
| `validate` | Un objet par ligne, par expérience : `valid`, et `profile_issues` ou `error`. |
| `send`, `fire` | `{"type": "sent", "result": …}` ; `send http` imprime `{"type": "response", "response": …}`, `send ws` `{"type": "exchange", "result": …}`. |
| `emulate` | Un objet par ligne : `started`, un `exchange` par requête, un `summary` par émulateur ; `valid` avec `--check`. |
| `emulators` | `{"path": …, "emulators": [{id, name, protocol, bind, rules, note}, …]}`. |
| `templates` | `[{"name": …, "experiment": …}, …]`. |
| `doctor` | Un objet : `version`, `network`, `data_dir`, `firewall`, `server`, `problems`. |
| `version` | `{"version": "…"}`. |
| `nodes` | Toujours du JSON, avec ou sans `--json`. |

Un échec est `{"type": "error", "error": {…}, "exit_code": N}`. `error` est
l'erreur du moteur : un `code` stable (tel que `transport.refused` ou
`secret.missing`), ses `params`, et le `node` et le `field` qu'elle concerne. Un
script peut brancher sur `error.code` dans n'importe quelle langue ; les textes
de chaque code sont listés dans [Messages d'erreur](../reference/errors.md).

### Codes de sortie {#exit-codes}

| Code | Signification |
| --- | --- |
| `0` | Toutes les expériences ont réussi ; l'envoi a réussi ; rien ne se dresse en travers. |
| `1` | Une expérience a tourné et échoué, a dépassé le temps imparti ou a été arrêtée ; un envoi a échoué (refusé, pas de réponse, un statut inattendu) ; `doctor` a trouvé quelque chose en travers. |
| `2` | La commande ou un document est incorrect : un argument, un fichier illisible, une erreur de validation, un paramètre inconnu, un secret manquant. |
| `3` | Rien n'a pu s'exécuter pour une raison extérieure à l'expérience : le serveur est injoignable ou refuse le jeton, un port ne peut pas être ouvert, le magasin d'identifiants échoue. |

Avec plusieurs expériences, le résultat le plus grave décide, dans cet ordre :
`2`, puis `3`, puis `1`, puis `0`.

### Variables d'environnement {#environment}

| Variable | Ce qu'elle fait |
| --- | --- |
| `SIGNALLAB_LANG` | La langue, quand `--lang` n'est pas donné. |
| `LC_ALL`, `LC_MESSAGES`, `LANG` | La langue, quand aucune des deux précédentes n'est définie. |
| `SIGNALLAB_SERVER` | Le serveur pour `run`, `validate`, `emulate`, `mcp` et `doctor`, comme le donne `--server`. |
| `SIGNALLAB_TOKEN` | Le jeton d'accès du serveur, quand aucun fichier de jeton n'est donné. |
| `SIGNALLAB_TOKEN_FILE` | Un fichier contenant le jeton du serveur, comme le donne `--token-file`. |
| `SIGNALLAB_SECRET_<NAME>` | La valeur du secret `NAME` pour les exécutions de ce processus (voir [Secrets](#secrets)). |
| `SIGNALLAB_DATA_DIR` | Le dossier de données de l'application, où `fire`, `emulators`, `emulate`, `mcp` et `doctor` regardent par défaut ; sinon `Documents/SignalLab` dans votre dossier personnel. |
| `GITHUB_ACTIONS` | Quand il vaut `true`, une exécution qui ne passe pas est aussi imprimée comme annotation `::error`, que GitHub affiche sur la page de l'exécution. |

## `run` {#cli-run}

```text
signallab run [OPTIONS] <FILE>...
```

Exécute les expériences l'une après l'autre et ne se termine par `0` que si
toutes ont réussi.

| Option | Ce qu'elle fait | Par défaut |
| --- | --- | --- |
| `<FILE>...` | Fichiers d'expérience, ou noms de [modèles intégrés](#cli-templates). | obligatoire |
| `-p`, `--param NAME=VALUE` | Une valeur de paramètre pour cette exécution ; répétez pour en ajouter. | les valeurs du document |
| `--profile NAME` | Exécute avec ce profil ; chaque expérience donnée doit l'avoir. `""` exécute avec les valeurs par défaut. | le profil du document |
| `-m`, `--matrix NAME=V1,V2` | Exécute une fois par valeur ; répétez pour plus de noms (voir [Une matrice d'exécutions](#matrix)). | |
| `--matrix-file PATH` | Des combinaisons depuis un fichier JSON. | |
| `--seed N` | La graine des valeurs aléatoires, de 0 à 9007199254740991 (2⁵³ − 1). | la graine du document, sinon une nouvelle par exécution |
| `--timeout SECONDS` | Fait échouer une exécution qui prend plus de temps, 1–300. | `300` |
| `--fail-fast` | S'arrête à la première exécution qui ne passe pas ; les autres ne sont pas lancées. | désactivé |
| `--junit PATH` | Y écrit un rapport JUnit XML (voir [Rapports](#reports)). | |
| `--report PATH` | Y copie le rapport d'exécution : un fichier pour une exécution, un dossier pour plusieurs. | |
| `--data-dir PATH` | Le dossier de données des exécutions de ce processus ; leurs rapports y restent. | un dossier temporaire, supprimé à la sortie |
| `--server URL` | Exécute sur ce serveur plutôt que dans ce processus (voir [Sur un serveur](#run-on-server)). | `SIGNALLAB_SERVER` |
| `--token-file PATH` | Un fichier contenant le jeton du serveur. | `SIGNALLAB_TOKEN_FILE`, sinon `SIGNALLAB_TOKEN` |
| `--secrets files\|system` | D'où viennent les valeurs des secrets dans ce processus. | `files` |
| `--secrets-dir PATH` | Un dossier de fichiers de secrets, un par nom. | `/run/secrets/signallab` s'il existe |

`--data-dir`, `--secrets` et `--secrets-dir` concernent les exécutions dans ce
processus ; ils ne peuvent pas être combinés avec `--server`.

### Fichiers et modèles {#run-inputs}

Un `FILE` est une expérience telle que l'application l'enregistre et l'exporte
([[ui:exp.exportJson]] sur l'écran [[ui:nav.experiment]]). Toute version de
document que l'application ouvre fonctionne ; une plus ancienne est mise à jour
au fur et à mesure de sa lecture, comme le fait l'application quand elle
l'ouvre. Un nom qui n'est pas un fichier est cherché parmi les [modèles
intégrés](#cli-templates), avec ou sans `.json` :

```bash
signallab run tests/stage-cues.json tests/api.json
signallab run osc-ping-reply --param device=192.0.2.20:9000
```

Chaque expérience — et chaque combinaison d'une matrice — est vérifiée comme
l'éditeur le fait avant que la première s'exécute. Un troisième fichier cassé
arrête aussi le premier, avant que quoi que ce soit ne soit envoyé, avec le code
de sortie `2`.

### Paramètres et profils {#run-params}

`--param NAME=VALUE` définit un paramètre pour cette exécution uniquement ; le
fichier n'est pas modifié. La valeur est tout ce qui suit le premier `=`, donc
`--param url=http://127.0.0.1/?q=1` fonctionne, et `--param note=` définit une
valeur vide. Une valeur s'applique à chaque expérience donnée qui a ce
paramètre, et doit nommer un paramètre d'au moins l'une d'elles — un nom mal
orthographié est refusé avec le code de sortie `2`.

`--profile NAME` exécute avec l'un des profils de l'expérience, comme le fait de
le choisir sous [[ui:exp.profile]] dans l'éditeur. Voir
[Données et modèles](../experiments/data.md).

### Une matrice d'exécutions {#matrix}

Une matrice exécute la même expérience contre chaque cible, chaque utilisateur,
chaque taille de charge utile. Chaque combinaison est une exécution à part, avec
son propre résultat, son propre rapport et sa propre suite dans le rapport
JUnit.

```bash
signallab run smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest \
  --fail-fast --junit junit.xml
```

Cela fait quatre exécutions : `device` varie le plus lentement, `user` le plus
vite. Elles sont nommées d'après le fichier et leurs valeurs :
`smoke.json [device=192.0.2.20:9000, user=admin]`.

- `--matrix NAME=V1,V2` ajoute un axe. Le même nom ajouté de nouveau y ajoute
  des valeurs. Les espaces autour des noms et des valeurs sont supprimés ; une
  valeur donnée deux fois ne s'exécute qu'une fois.
- `--matrix-file PATH` lit du JSON sous l'une de deux formes :

  ```json
  { "device": ["192.0.2.20:9000", "192.0.2.21:9000"], "retries": [1, 3] }
  ```

  ajoute des axes — chaque combinaison s'exécute, ces noms après ceux de
  `--matrix`, par ordre alphabétique ;

  ```json
  [
    { "device": "192.0.2.20:9000", "user": "admin" },
    { "device": "192.0.2.21:9000", "user": "guest" }
  ]
  ```

  liste les combinaisons elles-mêmes, chacune croisée avec les axes
  `--matrix`. Les valeurs sont du texte, des nombres ou `true`/`false` ; une
  virgule à l'intérieur d'une valeur dans un fichier en fait partie.
- Chaque nom de matrice doit être un paramètre d'au moins une expérience
  donnée. Une expérience dépourvue de l'un d'eux s'exécute une fois, pas une
  fois par valeur qu'elle ignorerait.
- Un nom défini à la fois par `--param` et la matrice, ou à la fois par
  `--matrix` et le fichier, est refusé.
- Au plus **256** exécutions pour une commande ; au-delà, tout est refusé avant
  que quoi que ce soit ne s'exécute.

### Exécuter sur un serveur {#run-on-server}

```bash
signallab run tests/stage.json --server http://192.0.2.10:1430 --token-file token.txt
```

L'expérience vient de cette machine ; le serveur l'exécute avec son propre
réseau, ses propres [secrets](../server/index.md#secrets) et son propre dossier
de données, et renvoie les étapes au fur et à mesure. Le rapport reste sur le
serveur — son chemin est imprimé — et `--report` en télécharge une copie. Une
exécution sur un serveur est là-bas une tâche comme une démarrée dans son
interface : chaque page qui y est connectée la montre. Si `signallab` disparaît
en cours d'exécution, l'exécution se termine quand même sur le serveur et garde
son rapport.

Le jeton est lu depuis `--token-file` (ou `SIGNALLAB_TOKEN_FILE`), sinon depuis
`SIGNALLAB_TOKEN`, et envoyé comme `Authorization: Bearer`. Un serveur
injoignable, qui refuse le jeton ou cesse de répondre en cours d'exécution donne
le code de sortie `3`. `signallab doctor --server URL` vérifie l'adresse et le
jeton à eux seuls.

### Rapports {#reports}

Chaque exécution écrit le même rapport que l'application. Sans `--data-dir`, les
exécutions utilisent un dossier temporaire supprimé à la sortie de `signallab`,
alors conservez ce dont vous avez besoin :

- `--report PATH` copie le rapport : vers `PATH` lui-même pour une exécution, ou
  dans le dossier `PATH` pour plusieurs, sous la forme `01-<file>.json`,
  `02-<file>.json`, … dans l'ordre où elles se sont exécutées.
- `--data-dir PATH` conserve le rapport de chaque exécution dans ce dossier
  (sous `runs/`), et imprime où se trouve chacun.

`--junit PATH` écrit un rapport JUnit XML, le format que lit chaque système de
CI :

- un `<testsuite>` par exécution (par combinaison d'une matrice), avec le
  fichier, la graine, le résultat, le profil, le chemin du rapport et chaque
  valeur de matrice (`param.NAME`) comme propriétés ;
- un `<testcase>` par nœud exécuté, nommé d'après le type du nœud et son
  identifiant, avec son propre temps ;
- un `<failure>` sur le nœud qui a échoué : le message dans la langue choisie,
  le code d'erreur comme `type`, et les étapes du nœud avec le détail technique ;
- un cas `<skipped>` pour chaque nœud que l'exécution n'a jamais atteint (l'autre
  côté d'une branche) ;
- une suite avec un cas `<error>` pour une expérience qui n'a pas pu démarrer,
  et une avec un cas `<skipped>` pour chaque exécution que `--fail-fast` n'a pas
  lancée.

```xml
<testsuites name="Signal Lab" tests="6" failures="1" errors="0" time="2.006">
  <testsuite name="HTTP to OSC" tests="6" failures="1" errors="0" skipped="4" time="2.006" timestamp="2026-10-04T16:48:03">
    <properties>
      <property name="file" value="status-branch" />
      <property name="seed" value="42" />
      <property name="outcome" value="failed" />
      <property name="report" value="out/report.json" />
    </properties>
    <testcase name="Start (start)" classname="HTTP to OSC" time="0.001">
      <system-out>Started · seed 42</system-out>
    </testcase>
    <testcase name="HTTP request (request)" classname="HTTP to OSC" time="2.004">
      <failure message="http://127.0.0.1:8080/ refused the connection — nothing is listening on that port" type="transport.refused">…</failure>
    </testcase>
    <testcase name="Status branch (branch)" classname="HTTP to OSC" time="0.000">
      <skipped message="not reached in this run" />
    </testcase>
    …
  </testsuite>
</testsuites>
```

Un nœud HTTP sous [charge](../experiments/load.md) ajoute une ligne sous sa
dernière étape pour chaque seuil, tenu ou non (`✕ p95 < 100 ms · 152.58 ms`) ; un
seuil qui n'est pas tenu fait échouer l'exécution et son cas JUnit
(`type="load.threshold"`).

### Secrets {#secrets}

Une expérience lit un secret sous la forme `{{secret.NAME}}`. Pour les
exécutions dans ce processus, la valeur vient de :

| `--secrets` | D'où vient la valeur de `NAME` |
| --- | --- |
| `files` (la valeur par défaut) | La variable d'environnement `SIGNALLAB_SECRET_NAME` ; sinon un fichier nommé `NAME` dans `--secrets-dir` (par défaut `/run/secrets/signallab`, quand ce dossier existe — la disposition des secrets Docker). |
| `system` | Le Gestionnaire d'identifiants Windows — où l'application conserve les valeurs de ses [[ui:exp.secrets]]. Linux n'a pas de magasin d'identifiants que `signallab` lit : là, `--secrets system` échoue avec le code de sortie `3`. |

Le saut de ligne final d'un fichier ne fait pas partie de la valeur, et une
valeur vide compte comme non définie. Les noms sont des lettres, des chiffres et
`_`, ne commençant pas par un chiffre, jusqu'à 128 caractères ; une valeur fait
au plus 16 Kio.

```bash
SIGNALLAB_SECRET_API_TOKEN="$API_TOKEN" signallab run tests/api.json
```

Un secret non défini arrête l'exécution avant tout trafic, avec le code de
sortie `2` et le nom manquant. Une valeur n'est jamais imprimée : les étapes, les
erreurs, les rapports et le rapport JUnit affichent `••••` à sa place. Avec
`--server`, les secrets sont ceux du serveur.

### Ce qu'imprime `run` {#run-output}

Au fil d'une exécution, chaque étape est une ligne sur stderr — son temps depuis
le début, le nœud, son état et ce qu'il a fait — comme la chronologie de
l'application. Quand elle se termine, une ligne sur stdout dit comment cela
s'est passé :

```text
▶ HTTP check (http-check) · seed 1185927457137919
     0.000  Start         Running
     0.000  Start         Passed · Started · seed 1185927457137919
     0.000  HTTP request  Running
     2.004  HTTP request  Failed · URL — http://127.0.0.1:8080/ refused the connection — nothing is listening on that port
✖ HTTP check failed after 2 s: HTTP request · URL — http://127.0.0.1:8080/ refused the connection — nothing is listening on that port
  Technical details: error sending request for url (http://127.0.0.1:8080/): … (os error 10061)
  To run it again with the same random values: --seed 1185927457137919
```

Une exécution échouée nomme la graine qu'elle a utilisée : `--seed` avec ce
nombre la relance avec les mêmes valeurs aléatoires. Après le verdict viennent,
sur stderr, ce qui a été demandé à chaque émulateur de l'exécution — requêtes,
combien aucune règle n'a prises, combien ont échoué, et les correspondances par
règle :

```text
✔ Retry a flaky API passed in 7 ms
  Flaky API: 3 requests, 0 without a rule, 0 failed · #1 3
```

et ce qu'a fait chaque relais de dégradation : ce qu'il a reçu, abandonné et
limité, et chaque phase sous la forme transmis / reçu :

```text
  127.0.0.1:19110 → 127.0.0.1:19100: 155 datagrams, 38 dropped, 0 throttled · lan 0.0–2.0 s 39/39, wifi 2.0–4.0 s 39/39, offline 4.0–6.0 s 0/38, lan 6.0–8.0 s 38/38
```

Un relais sur TCP déplace des morceaux de flux, pas des datagrammes, et
n'abandonne rien, donc sa ligne compte les morceaux, les connexions, les
réinitialisations, les connexions à demi-ouvertes et les fois où un flux a été
retenu par la limite de bande passante :

```text
  127.0.0.1:19120 → 127.0.0.1:19101: 14 chunks, 2 connections, 1 reset, 0 half-open, 3 held back
```

Un nœud HTTP sous [charge](../experiments/load.md) rapporte sa progression comme
une étape au plus une fois par seconde.

Avec plusieurs exécutions, une ligne par exécution et un décompte terminent la
sortie : `3 runs: 2 passed, 1 failed`. Avec `--fail-fast`, les exécutions qu'il
n'a pas lancées sont comptées sur stderr.

Avec `--json`, chaque ligne est un objet avec un `type` :

```json
{"type":"started","experiment":"Empty experiment","file":"empty","job_id":1,"overridden":false,"profile":null,"seed":1,"started_ms":1791132204585}
{"type":"step","node_id":"start","state":"passed","detail":"Started","message_key":"exp.step.started","message_params":{"seed":1},"job_id":1,"ts":1791132204585}
{"type":"ended","experiment":"Empty experiment","file":"empty","outcome":"passed","seed":1,"params":{},"steps":[…],"report_path":"…","started_ms":1791132204585,"ended_ms":1791132204585,…}
{"type":"summary","total":1,"passed":1,"failed":0,"not_started":0,"exit_code":0}
```

Les lignes `started`, `ended` et `error` portent `file` (l'argument tel que
donné) et, dans une matrice, `matrix` (les valeurs de la combinaison). `ended`
est tout le résultat de l'exécution : `outcome` (`passed`, `failed` ou
`stopped`), `seed`, `profile`, `params`, `error`, chaque étape, `emulators` et
`impairments` quand l'exécution en avait, et `report_path`. La dernière étape
d'une charge porte `load` avec chaque nombre qu'elle a mesuré : `planned`,
`sent`, `ok`, `failed`, `missed`, `rps`, `error_rate`, `min_ms`, `mean_ms`,
`max_ms`, `p50_ms` à `p99_ms`, `statuses`, chaque seconde (`seconds`),
l'`histogram`, et le verdict de chaque seuil (`thresholds`).

## `validate` {#cli-validate}

```text
signallab validate [OPTIONS] <FILE>...
```

Vérifie les expériences comme le fait l'éditeur avant une exécution — le graphe,
chaque champ, les modèles, les paramètres et les secrets — et n'envoie rien. Se
termine par `0` quand toutes démarreraient.

Il prend les entrées de [`run`](#cli-run) : fichiers et modèles, `--param`,
`--profile`, `--matrix`, `--matrix-file`, et `--server`, `--token-file`,
`--secrets`, `--secrets-dir`. Avec `--server`, le serveur les vérifie, d'après
ses propres secrets.

```text
✔ tests/stage.json: Stage cues would run
  Rehearsal: Would not run: …
```

Un problème que seul un autre profil du document présente est listé sous
celui-ci, mais ne fait pas échouer la vérification. Avec `--json`, une ligne par
expérience (par combinaison) : `{"experiment", "file", "valid": true, "profile_issues": […]}` ou `"valid": false` avec `error` et `exit_code`.

## `send` {#cli-send}

```text
signallab send <osc|udp|http|ws|mqtt> …
```

Envoie un message via les mêmes commandes qu'utilisent les écrans de
l'application, donc ce sont les mêmes octets sur le fil. Un envoi ne lit aucune
bibliothèque ni aucun secret.

Codes de sortie : `0` envoyé, `1` l'envoi a échoué, `2` un argument est
incorrect.

Un `<host:port>` est une adresse IP ou un nom d'hôte, et un port :
`127.0.0.1:9000`, `[::1]:9000` (une adresse IPv6 entre crochets) ou
`device.local:9000`. Un nom est résolu quand la commande s'exécute, son adresse
IPv4 prise quand il en a une, donc `localhost:9000` atteint un récepteur sur
`127.0.0.1`. Un nom qui ne se résout pas fait échouer l'envoi (`1`) ; une cible
sans port est un argument invalide (`2`).

### `send osc` {#cli-send-osc}

```text
signallab send osc <host:port> <address> [ARG]...
```

Un message OSC. Les arguments sont typés avec un préfixe, ou déduits :

| Argument | Type OSC |
| --- | --- |
| `i:3` | int32 |
| `f:0.5` | float32 |
| `d:1.5` | float64 (double) |
| `h:64` | int64 |
| `s:text` | string |
| `b:de ad be ef` | blob, en octets hexadécimaux |
| `T`, `F` | vrai, faux |
| `N` | nil |
| `3`, `-3` | un entier simple est int32 |
| `2.5` | un décimal simple est float32 |
| tout le reste | string |

```bash
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main 3
# ✔ sent /cue/go (32 bytes) → 127.0.0.1:9000
```

Mettez entre guillemets un argument contenant des espaces : `"s:hello world"`.
`s:7` envoie le texte `7`. L'adresse doit commencer par `/` ; une qui ne le fait
pas est un argument invalide (`2`).

::: warning Git Bash on Windows
Git Bash réécrit les arguments qui commencent par `/` en chemins Windows, donc
`/cue/go` arrive comme `C:/Program Files/Git/cue/go`. Écrivez `//cue/go`, ou
exécutez avec `MSYS_NO_PATHCONV=1`. PowerShell et `cmd` ne sont pas affectés.
:::

Voir [OSC](../protocols/osc.md).

### `send udp` {#cli-send-udp}

```text
signallab send udp <host:port> (--text TEXT | --hex HEX)
```

Un datagramme UDP. Donnez la charge utile en `--text`, ou en octets `--hex` :
`"de ad be ef"`, `deadbeef` ou `0xDE,0xAD`.

```bash
signallab send udp 127.0.0.1:7000 --text "PLAY 1"
signallab send udp 127.0.0.1:7000 --hex "de ad be ef"
```

### `send http` {#cli-send-http}

```text
signallab send http <METHOD> <URL> [OPTIONS]
```

Une requête HTTP. La ligne de statut va sur stderr — `HTTP 200 OK · 3 ms · 1,234 B` — et le corps de la réponse sur stdout, pour pouvoir être redirigé. Un corps de plus de 256 Kio y est coupé, et stderr le dit.

| Option | Ce qu'elle fait | Par défaut |
| --- | --- | --- |
| `-H`, `--header "Name: value"` | Un en-tête de requête ; répétez pour en ajouter. | |
| `--body TEXT` | Le corps de la requête. `@FILE` envoie le contenu d'un fichier texte. | aucun |
| `--expect-status STATUS` | Se termine par `1` à moins que la réponse n'ait ce statut. | tout statut vaut `0` |
| `--timeout MS` | Millisecondes à attendre la réponse. | `10000` |
| `-u`, `--user NAME:PASSWORD` | Identifiants, envoyés en Basic. | |
| `--digest` | Avec `--user` : répond plutôt au défi Digest du serveur (MD5 ou SHA-256). | désactivé |
| `--bearer TOKEN` | Envoie `Authorization: Bearer TOKEN`. Pas avec `--user`. | |

```bash
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab send http POST http://127.0.0.1:8080/cue -H "Content-Type: application/json" --body '{"cue": 1}'
signallab send http GET http://127.0.0.1:8080/admin -u admin:secret --digest
```

Sans `--expect-status`, toute réponse est un succès, un `500` compris. Une
requête qui n'obtient aucune réponse (refusée, expirée, un nom qui ne se résout
pas) donne le code de sortie `1`, et un défi Digest auquel on n'a pas pu
répondre aussi — la raison est imprimée après la réponse.

::: tip
Les arguments sont visibles par les autres utilisateurs de la même machine.
Gardez les vrais mots de passe pour les expériences, où ils sont des
[secrets](#secrets).
:::

Voir [HTTP](../protocols/http.md).

### `send ws` {#cli-send-ws}

```text
signallab send ws <URL> [OPTIONS]
```

Un échange WebSocket : se connecter à `ws://…` ou `wss://…`, envoyer un message,
éventuellement attendre la réponse, fermer. La poignée de main et ce qui a été
envoyé vont sur stderr, la réponse sur stdout (les messages binaires en
hexadécimal).

| Option | Ce qu'elle fait | Par défaut |
| --- | --- | --- |
| `--text TEXT` | Envoie ce message texte. | rien envoyé |
| `--hex HEX` | Envoie ces octets comme message binaire. Pas avec `--text`. | |
| `-H`, `--header "Name: value"` | Un en-tête pour la requête de mise à niveau ; répétez pour en ajouter. | |
| `--protocol NAME` | Un sous-protocole à proposer ; répétez pour en ajouter, par ordre de préférence. | |
| `--expect TEXT` | Attend un message contenant ce texte. | |
| `--expect-regex REGEX` | Attend un message correspondant à cette expression régulière. | |
| `--wait` | Attend n'importe quel message. | |
| `--timeout MS` | Millisecondes à attendre la réponse. | `2000` |

```bash
signallab send ws ws://127.0.0.1:9001/echo --text '{"ping": 1}' --expect '"ping"'
signallab send ws ws://127.0.0.1:9001/feed --wait        # nothing sent: the server's first message
```

Sans `--expect`, `--expect-regex` ou `--wait`, il se connecte, envoie et ferme
sans attendre. Quand la réponse attendue n'arrive pas à temps, le code de sortie
est `1`. Voir [WebSocket](../protocols/websocket.md).

### `send mqtt` {#cli-send-mqtt}

```text
signallab send mqtt <host:port> <topic> [payload] [--qos 0|1|2] [--retain]
```

Une publication MQTT 3.1.1 sur TCP simple, sans identifiants, comme un client à
part avec un identifiant de client neuf, donc elle ne déloge jamais une connexion
déjà présente. Sans port, le broker est sur `1883`. Un sujet est un seul sujet :
avec un `+` ou `#` dedans, ou vide, la commande est refusée avant de se connecter
(`2`).

| Option | Ce qu'elle fait | Par défaut |
| --- | --- | --- |
| `[payload]` | La charge utile. | vide |
| `--qos 0\|1\|2` | La qualité de service. | `0` |
| `--retain` | La conserve comme valeur retenue du sujet. Une charge utile vide avec `--retain` l'efface. | désactivé |

```bash
signallab send mqtt 127.0.0.1:1883 lab/light/1/set on --qos 1
signallab send mqtt 127.0.0.1 lab/light/1/state "" --retain     # clear the retained value
```

Voir [MQTT](../protocols/mqtt.md).

## `fire` {#cli-fire}

```text
signallab fire <signal> [--library PATH]
```

Envoie un signal d'une bibliothèque de signaux exactement comme l'écran
[[ui:nav.signals]] de l'application l'envoie : signaux OSC, UDP, HTTP et MQTT.
Le signal est trouvé par son identifiant, sinon par son nom, sans tenir compte
de la casse ; un nom que plusieurs signaux partagent est refusé avec leurs
identifiants.

| Option | Ce qu'elle fait | Par défaut |
| --- | --- | --- |
| `<signal>` | L'identifiant ou le nom du signal. | obligatoire |
| `--library PATH` | Le fichier de bibliothèque. | `signals.json` dans le dossier de données de l'application |

```bash
signallab fire "Fader value"
signallab fire go --library show/signals.json
```

La bibliothèque est seulement lue, jamais créée ni modifiée. Voir
[Signaux](../tools/signals.md).

## `emulate` {#cli-emulate}

```text
signallab emulate [OPTIONS] <FILE|NAME>...
```

Joue l'autre côté — une API HTTP, un appareil OSC, UDP ou TCP, un broker MQTT —
jusqu'à <kbd>Ctrl</kbd>+<kbd>C</kbd> ou `--for`, et imprime chaque requête au fur
et à mesure qu'elle est traitée. Un `FILE` contient un émulateur, une liste
d'émulateurs, ou une bibliothèque entière telle que l'application l'écrit ; un
`NAME` est l'identifiant ou le nom de l'un d'eux dans la bibliothèque de
l'application (celle de l'écran [[ui:nav.emulators]]).

| Option | Ce qu'elle fait | Par défaut |
| --- | --- | --- |
| `-p`, `--param NAME=VALUE` | Une valeur que ses modèles lisent comme `{{NAME}}` ; répétez pour en ajouter. | |
| `--bind IP:PORT` | Écouter là plutôt. Seulement avec un émulateur. | celle de l'émulateur |
| `--for SECONDS` | S'arrête après ce temps. | jusqu'à <kbd>Ctrl</kbd>+<kbd>C</kbd> |
| `--seed N` | La graine de ses choix aléatoires : un ordre aléatoire, la gigue, les générateurs. | |
| `--check` | Vérifie les émulateurs et quitte, sans ouvrir de port. | désactivé |
| `--library PATH` | La bibliothèque dans laquelle les noms sont cherchés. | `emulators.json` dans le dossier de données de l'application |
| `--server URL`, `--token-file PATH` | Les démarre sur un serveur, les suit via son API, et les arrête à la fin. | |
| `--secrets`, `--secrets-dir` | Comme pour [`run`](#cli-run). | |

```text
$ signallab emulate tests/orders-api.json --for 60
Orders API (http) answering on 127.0.0.1:18099
answering for 60 s
+   1.209 s Orders API  #1  GET /orders/42 → 200 OK · 12 B  1 ms  ← 127.0.0.1:55744
+   1.209 s Orders API  —  GET /nothing → 404 Not Found · 20 B  2 ms  ← 127.0.0.1:55745
Orders API: 2 requests, 1 without a rule, 0 failed · #1 1
```

Chaque ligne est le temps depuis le début, l'émulateur, la règle qui a répondu
(`#1`, ou `—` pour aucune), la requête et ce qu'elle a obtenu, le temps qu'elle a
pris et qui l'a envoyée. À la fin, les décomptes de chaque émulateur : requêtes,
combien aucune règle n'a prises, combien ont échoué, combien ont rencontré une
panne ou n'ont pas été remises quand il y en avait, et les correspondances par
règle.

Codes de sortie : `0` quand il s'arrête sur <kbd>Ctrl</kbd>+<kbd>C</kbd> ou
`--for` ; `2` quand un émulateur n'est pas valide ; `3` quand son port est pris
ou ne peut pas être ouvert, ou qu'une socket échoue pendant qu'il répond.

Dans un pipeline, démarrez-le en arrière-plan, testez le système contre lui, et
lisez les décomptes à la fin :

```bash
signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
npm test          # the system under test, configured for the emulator's address
wait              # the last lines of mock.ndjson are the counts
```

Sur un serveur, un émulateur démarré avec `--server` est arrêté quand `signallab`
se termine normalement ; celui qu'un processus tué a laissé derrière peut être
arrêté depuis l'interface du serveur. Voir [Émulateurs](../tools/emulators.md).

## `emulators` {#cli-emulators}

```text
signallab emulators [--library PATH]
```

Liste les émulateurs de la bibliothèque : identifiant, nom, protocole, adresse et
nombre de règles. `--library PATH` lit un autre fichier de bibliothèque au lieu
de `emulators.json` dans le dossier de données de l'application.
`signallab emulate <id>` en démarre un.

La bibliothèque est seulement lue. S'il n'y a pas un tel fichier — l'application
crée celui de son dossier de données au premier démarrage —, la commande le dit
et se termine par `2`.

## `templates` {#cli-templates}

```text
signallab templates
```

Liste les modèles intégrés, que `run` et `validate` prennent par leur nom. Ce
sont ceux de l'application :

| Nom | Dans l'application | Paramètres |
| --- | --- | --- |
| `empty` | [[ui:exp.templateEmpty]] | |
| `http-check` | [[ui:exp.templateHttp]] | |
| `status-branch` | [[ui:exp.templateBranch]] | |
| `parallel-flows` | [[ui:exp.templateParallel]] | |
| `osc-ping-reply` | [[ui:exp.templatePingReply]] | `device` = `127.0.0.1:9000` |
| `poll-until-ready` | [[ui:exp.templatePoll]] | `device` = `127.0.0.1:9000` |
| `flaky-api` | [[ui:exp.templateFlaky]] | `api` = `http://127.0.0.1:18080` |
| `fault-phases` | [[ui:exp.templateFaults]] | |
| `dependency-outage` | [[ui:exp.templateOutage]] | `api` = `http://127.0.0.1:18090` |
| `websocket-echo` | [[ui:exp.templateWsEcho]] | `service` = `ws://127.0.0.1:9001/echo` |

Chaque modèle pointe vers la boucle locale. `flaky-api`, `fault-phases` et
`dependency-outage` apportent leurs propres émulateurs, donc ils s'exécutent sans
rien d'autre à l'écoute — un moyen rapide de voir `signallab` travailler.

## `nodes` {#cli-nodes}

```text
signallab nodes
```

Imprime, en JSON, de quoi est faite une expérience : la forme et les règles du
document, chaque type de nœud avec son libellé, sa description, ses champs, ses
sorties et un exemple que le moteur accepte, le langage `{{template}}`, les
profils de charge et le document d'émulateur. C'est ce qu'un assistant lit via
[`signallab mcp`](mcp.md) pour écrire une expérience ; pour les personnes,
[Nœuds](../experiments/nodes.md) dit la même chose avec plus de mots.

## `mcp` {#cli-mcp}

```text
signallab mcp [OPTIONS]
```

Sert Signal Lab à un assistant d'IA via le Model Context Protocol, sur stdin et
stdout. Tout — l'installer dans Claude Code, Claude Desktop, Cursor ou VS Code,
ses options et ses outils — se trouve sur [Assistants (MCP)](mcp.md).

## `doctor` {#cli-doctor}

```text
signallab doctor [--server URL] [--token-file PATH]
```

Dit ce qui pourrait se dresser entre Signal Lab et l'équipement, et se termine
par `1` quand c'est le cas. Ses lignes suivent [`--lang`](#language) comme le
reste de la ligne de commande :

- le réseau sur lequel se trouve cette machine : son nom et son adresse ;
- le dossier de données : s'il peut être écrit (un dossier pas encore créé
  convient — l'application le crée à la première utilisation) ;
- le pare-feu : sous Windows, pour `signallab` et l'application de bureau chacun,
  si une règle laisse entrer d'autres machines selon le type de réseau sur lequel
  se trouve la machine maintenant (privé, domaine ou public), ou si une règle
  les bloque — ce que laisse un *Annuler* à l'invite du système ; sous Linux, si
  ufw ou firewalld est actif, et la commande qui ouvre un port ;
- avec `--server` (ou `SIGNALLAB_SERVER`) : si le serveur répond et accepte le
  jeton.

```text
signallab [[version]]
Network: LAB-PC · 192.0.2.15
Data folder: C:\Users\lab\Documents\SignalLab — writable
Firewall · signallab (C:\…\Signal Lab\signallab.exe) · private network: not allowed yet — signallab firewall allow
Firewall · app (C:\…\Signal Lab\signal-lab.exe) · private network: allowed
✖ 1 thing in the way
```

`--json` imprime la même chose sous forme d'un objet. Voir
[Dépannage](../reference/troubleshooting.md).

## `firewall` {#cli-firewall}

```text
signallab firewall allow [--public]
```

Sous Windows, laisse d'autres machines atteindre Signal Lab — ce dont un moniteur
ou une attente a besoin pour entendre un appareil. Windows demande d'abord les
droits d'administrateur ; puis les règles entrantes de `signallab` et de
l'application de bureau (trouvée à côté, ou là où les installateurs la placent)
sont remplacées par une règle d'autorisation chacune, règles de blocage
comprises, sur les réseaux privés et de domaine.

| Option | Ce qu'elle fait |
| --- | --- |
| `--public` | Aussi sur les réseaux publics — le Wi-Fi d'un lieu l'est souvent. |

Codes de sortie : `0` fait ; `3` quand l'invite d'administrateur est refusée ou
que le changement échoue. Sous Linux, rien n'est modifié : il imprime la
commande ufw ou firewalld qui ouvre les ports sur lesquels vous écoutez, et se
termine par `0`.

Le pare-feu ne change que quand vous exécutez ceci ; rien d'autre dans
`signallab` n'y touche.

## `version` {#cli-version}

```text
signallab version
```

Imprime `signallab [[version]]` — avec `--json`,
`{"version": "[[version]]"}`. `signallab --version` imprime la même version.

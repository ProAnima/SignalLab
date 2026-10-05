---
title: "Pipelines de CI"
description: "Exécutez des expériences Signal Lab dans GitHub Actions, GitLab CI ou tout pipeline, faites échouer la tâche quand une échoue, et conservez un rapport JUnit."
---

# Signal Lab en CI

Les expériences qui mettent en service une installation sont aussi ses tests de
non-régression. Dans un pipeline, [`signallab run`](cli.md#cli-run) les exécute
sans fenêtre, imprime chaque étape, écrit un rapport JUnit que chaque système de
CI affiche, et se termine avec un code que la tâche comprend :

| Code de sortie | Le pipeline devrait |
| --- | --- |
| `0` | continuer : toutes les expériences ont réussi |
| `1` | échouer : une expérience a tourné et échoué |
| `2` | échouer : une expérience ou la commande est incorrecte (une erreur de validation, un paramètre inconnu, un secret manquant) |
| `3` | échouer ou réessayer : rien n'a pu s'exécuter (le serveur est injoignable ou refuse le jeton, un port ne peut pas être ouvert) |

Il y a quatre façons d'entrer :

| Voie | Où elle s'exécute |
| --- | --- |
| [L'action GitHub](#github-actions) | Un runner Linux, depuis l'image du serveur. |
| [L'image](#docker) | Tout CI qui exécute des conteneurs : GitLab, Jenkins, un shell avec Docker. |
| [Le binaire](#binary) | Tout runner, Windows compris. |
| [Un serveur de laboratoire](#lab-server) | Les exécutions se passent sur un serveur Signal Lab à côté de l'équipement ; le pipeline ne fait que les envoyer. |

## GitHub Actions {#github-actions}

Le dépôt est aussi une action GitHub. Elle exécute `signallab` depuis l'image
`ghcr.io/proanima/signallab`, fait échouer la tâche quand une expérience échoue,
et laisse un rapport JUnit :

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

Une exécution qui ne passe pas est aussi une annotation d'erreur sur la page de
l'exécution, à côté de l'étape qui échoue, avec l'expérience et la raison de son
échec.

### Entrées {#action-inputs}

| Entrée | Ce qu'elle fait | Par défaut |
| --- | --- | --- |
| `experiments` | Fichiers d'expérience ou noms de modèles intégrés, séparés par des espaces ou des lignes. Les motifs tels que `tests/*.json` sont développés ; un chemin contenant des espaces n'est pas pris en charge. | obligatoire |
| `params` | Valeurs de paramètres, `NAME=VALUE`, une par ligne. | |
| `profile` | Exécute avec ce profil des expériences. | |
| `matrix` | Exécute une fois par combinaison : `NAME=V1,V2`, un nom par ligne. | |
| `matrix-file` | Des combinaisons depuis un fichier JSON (voir [Une matrice d'exécutions](#matrix)). | |
| `fail-fast` | `"true"` : s'arrête à la première exécution qui ne passe pas. | `"false"` |
| `server` | Exécute sur ce serveur Signal Lab plutôt que dans la tâche, p. ex. `http://192.0.2.10:1430`. | |
| `token` | Le jeton d'accès du serveur. Passez un secret. | |
| `junit` | Où va le rapport JUnit. | `signallab-junit.xml` |
| `timeout` | Secondes qu'une exécution peut prendre, de 1 à 300. | `300` |
| `version` | L'étiquette de l'image. | `latest` |
| `image` | Un autre registre ou une image construite localement ; `version` est son étiquette. | `ghcr.io/proanima/signallab` |
| `lang` | La langue des messages et du rapport : `en`, `ru`, `es`, `fr`, `de`, `pt`, `zh`, `ja`, `ko`, `hi` ou `ar`. | `en` |
| `fail-on-error` | `"false"` : ne fait pas échouer l'étape ; lisez plutôt `exit-code`. | `"true"` |

Les chemins (`experiments`, `matrix-file`, `junit`) sont relatifs au répertoire
de travail de l'étape.

::: tip
Épinglez `version` sur la version avec laquelle vous avez testé. `latest` suit
chaque nouvelle version stable.
:::

### Sorties {#action-outputs}

| Sortie | Ce que c'est |
| --- | --- |
| `junit` | Le chemin du rapport JUnit. |
| `exit-code` | Le code de sortie de `signallab` : `0`, `1`, `2` ou `3`. |

### Continuer après un échec {#fail-on-error}

Une étape qui échoue ne transmet aucune sortie au reste de la tâche. Pour
décider vous-même, réglez `fail-on-error: "false"` et branchez sur `exit-code` :

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

### Secrets dans l'action {#action-secrets}

Le `{{secret.NAME}}` d'une expérience lit `SIGNALLAB_SECRET_NAME`. Définissez ces
variables sur l'étape (`env:`), à partir des secrets du dépôt. L'action remet
chaque variable `SIGNALLAB_SECRET_…` de son étape à `signallab` par son nom ; les
valeurs voyagent dans l'environnement, jamais sur une ligne de commande, et
chaque rapport et ligne de journal affiche `••••` à leur place. Un secret dont
l'expérience a besoin et qui n'est pas défini fait échouer l'étape avec le code
de sortie `2`, avant que quoi que ce soit ne soit envoyé.

L'entrée `token` voyage de la même façon, comme `SIGNALLAB_TOKEN`.

### Comment fonctionne l'action {#action-runs}

- Il a besoin d'un **runner Linux** avec Docker (`ubuntu-latest` en a un). Sur un
  runner Windows ou macOS, il s'arrête avec le code de sortie `2` et une
  annotation ; là, utilisez [le binaire](#binary).
- `signallab` s'exécute dans l'image avec le **réseau de l'hôte** : tout ce que le
  runner atteint, il l'atteint. Un service que la tâche a démarré sur le runner —
  votre système sous test, ou un conteneur `services:` avec un port publié — est
  à `127.0.0.1`.
- Il s'exécute en tant qu'utilisateur du runner, avec l'espace de travail monté
  au même chemin, donc le rapport appartient à la tâche.
- Il passe à `run` exactement les entrées ci-dessus, et `--junit`. Pour tout ce
  que `signallab` sait faire d'autre — `--report`, `--seed`, `--json`, `emulate`
  —, utilisez [l'image](#docker) directement.

## L'image, dans n'importe quel CI {#docker}

L'image du serveur porte `signallab` sous `/usr/local/bin/signallab`. Remplacez
le point d'entrée pour l'utiliser.

**GitLab CI :**

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

Les secrets sont des variables CI/CD nommées `SIGNALLAB_SECRET_<NAME>`
(marquez-les masquées) ; l'environnement de la tâche les remet à `signallab`
tels quels.

**Docker, depuis un shell ou n'importe quel ordonnanceur** (cron, Jenkins, un
script de déploiement) :

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

- `--network host` laisse les exécutions atteindre ce que l'hôte atteint,
  diffusion et multicast compris — sur un hôte Linux. Sans lui, le conteneur
  n'atteint les autres hôtes qu'en unicast.
- L'image s'exécute en tant qu'utilisateur non privilégié (uid 10001). `--user`
  l'exécute plutôt en tant que vous, pour qu'il puisse écrire le rapport dans
  votre dossier ; sans lui, le dossier doit être accessible en écriture pour
  l'uid 10001.
- `-e NAME` sans valeur transmet cette variable depuis votre environnement.

## Le binaire {#binary}

Chaque version porte `signallab` à part :
`signallab-<version>-linux-x64.tar.gz` et `signallab-<version>-windows-x64.zip`,
listés dans le `SHA256SUMS.txt` de la version. Celui pour Linux est construit sur
Ubuntu 22.04 et utilise l'OpenSSL 3 du système (`libssl3`) : il tourne sur cette
version ou une distribution plus récente.

```yaml
      - name: Signal Lab
        run: |
          curl -fsSL -o signallab.tar.gz https://github.com/ProAnima/SignalLab/releases/download/v[[version]]/signallab-[[version]]-linux-x64.tar.gz
          tar -xzf signallab.tar.gz
          ./signallab run tests/signallab/smoke.json --junit signallab-junit.xml
```

Sur un runner Windows, décompressez le zip et exécutez `signallab.exe` de la même
façon. Une machine avec l'application de bureau installée a déjà `signallab` dans
son `PATH`.

## Exécutions sur un serveur de laboratoire {#lab-server}

L'équipement sur le réseau d'une installation est atteignable depuis le
laboratoire, pas depuis un runner cloud. Lancez-y un [serveur Signal
Lab](../server/index.md), gardez son jeton comme secret de CI, et envoyez-lui les
exécutions :

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

L'expérience vient du checkout du pipeline ; le serveur l'exécute avec son propre
réseau, ses secrets et son dossier de données, renvoie les étapes et garde le
rapport (`--report` en télécharge une copie). Le jeton vient de `--token-file` ou
de `SIGNALLAB_TOKEN`. Les codes de sortie sont les mêmes ; un serveur injoignable
ou qui refuse le jeton donne `3`. Le runner doit pouvoir atteindre le serveur —
un runner auto-hébergé dans le laboratoire, ou une adresse de serveur que le
runner peut ouvrir.

Pour exécuter sur le serveur sans `signallab` du tout, un script peut appeler
directement son API HTTP : voir [Exécuter une expérience via HTTP](../api/run.md).

## Une matrice d'exécutions {#matrix}

Une expérience, chaque cible : chaque combinaison est une exécution à part et une
suite de tests à part dans le rapport JUnit, nommée d'après ses valeurs.

```bash
signallab run tests/smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest
```

Dans l'action, un nom par ligne :

```yaml
        with:
          experiments: tests/signallab/smoke.json
          matrix: |
            device=192.0.2.20:9000,192.0.2.21:9000
            user=admin,guest
          fail-fast: "true"
```

Ou depuis un fichier, avec `--matrix-file` (`matrix-file` dans l'action) :

```json
[
  { "device": "192.0.2.20:9000", "user": "admin" },
  { "device": "192.0.2.21:9000", "user": "guest" }
]
```

Chaque combinaison est vérifiée avant que la première n'envoie quoi que ce soit,
au plus 256 exécutions viennent d'une commande, et `--fail-fast` laisse les
autres non lancées — elles apparaissent dans le rapport JUnit comme ignorées. Les
règles sont sur [la page de la ligne de commande](cli.md#matrix).

Pour essayer chaque profil d'une expérience, exécutez une fois par profil — une
étape chacune, ou une matrice de tâches de votre CI — avec `--profile`.

## Rapports JUnit {#junit}

`--junit PATH` (l'action en écrit toujours un) contient une suite de tests par
exécution et un cas de test par nœud : l'échec là où il s'est produit, dans la
langue choisie, avec son code d'erreur et le détail technique ; les nœuds qu'une
exécution n'a jamais atteints comme ignorés ; la graine, le résultat, le fichier
et les valeurs de matrice comme propriétés. GitHub (avec une action de rapport),
GitLab (`artifacts:reports:junit`), Jenkins et Azure DevOps l'affichent comme
résultats de tests. Sa structure est sur [la page de la ligne de
commande](cli.md#reports).

La graine d'une exécution échouée est dans les propriétés de sa suite et dans le
journal : `--seed <that number>` la relance avec les mêmes valeurs aléatoires.

## Dépendances que le système sous test appelle {#emulators}

Pour tester votre propre système contre une API, un appareil ou un broker qui
n'est pas là en CI, laissez Signal Lab le jouer :

- **Dans une expérience**, un nœud [[ui:exp.node.emulator]] joue la dépendance
  pour une exécution, et [[ui:exp.node.wait_http]] vérifie ce que votre système
  lui a envoyé. La sortie de l'exécution se termine par ce qui a été demandé à
  chaque émulateur. Voir [Émulateurs](../tools/emulators.md).
- **Autour de vos propres tests**, `signallab emulate` répond en arrière-plan
  pendant qu'ils tournent, et ses décomptes vous disent ce qui a été appelé :

  ```bash
  signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
  npm test
  wait
  ```

## Sortie pour un script {#json}

`--json` imprime un objet JSON par ligne sur stdout et rien d'autre : `started`,
chaque `step`, `ended` avec tout le résultat, et un `summary` avec `total`,
`passed`, `failed`, `not_started` et `exit_code`. Les erreurs portent le `code`
stable du moteur, donc un script peut brancher dessus dans n'importe quelle
langue. Voir [Sortie](cli.md#output).

```bash
signallab run tests/smoke.json --json | jq -c 'select(.type == "ended") | {file, outcome, seed}'
```

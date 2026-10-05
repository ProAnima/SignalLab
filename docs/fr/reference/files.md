---
title: "Fichiers et dossiers"
description: "Où Signal Lab conserve ses fichiers — le dossier de données, l’expérience, les bibliothèques de signaux et d’émulateurs, les rapports d’exécution et les exports — leurs formats, et ce qu’il est sûr de modifier, de sauvegarder et de déplacer."
---

# Fichiers et dossiers

Tout ce que Signal Lab conserve est du JSON (ou du texte) en clair dans un seul
dossier, le dossier de données. Les valeurs secrètes ne s’y trouvent jamais.

## Le dossier de données {#data-folder}

| Où Signal Lab s’exécute | Le dossier de données |
| --- | --- |
| Application de bureau, Windows | `Documents\SignalLab` dans votre dossier utilisateur : `C:\Users\<you>\Documents\SignalLab` |
| Application de bureau, Linux | `~/Documents/SignalLab` |
| Serveur | `--data-dir`, ou `SIGNALLAB_DATA_DIR` ; sans l’un ou l’autre, `Documents/SignalLab` dans le dossier personnel de l’utilisateur sous lequel il tourne |
| Serveur, image Docker | `/data`, un volume (`signallab-data` dans le fichier compose) |
| `signallab run` | Un dossier temporaire, supprimé à sa fermeture — sauf si `--data-dir` en nomme un |

L’application de bureau prend aussi `SIGNALLAB_DATA_DIR` de son environnement
lorsqu’il est défini. Le dossier est créé lors de la première écriture.

::: tip
Sous Windows, l’application utilise le dossier `Documents` directement dans votre
dossier utilisateur, même lorsque Windows conserve vos documents ailleurs (OneDrive).
:::

Sur un serveur, les fichiers sont écrits sur la machine du serveur, pas sur la
vôtre. Le badge [[ui:app.server]] dans l’en-tête indique où, dans son info-bulle ;
l’API le donne comme `data_dir` de [`app_info`](../api/commands.md#app_info), et
[`/api/files`](../api/index.md#files) télécharge ce qu’il contient.

`signallab emulate`, `signallab send` et `signallab mcp` de la ligne de commande
lisent les bibliothèques de l’application depuis le même dossier que l’application
de bureau.

## Ce qu’il contient {#contents}

| Fichier | Ce que c’est | Écrit |
| --- | --- | --- |
| `experiment.json` | L’expérience ouverte dans l’éditeur | Peu après chaque modification |
| `signals.json` | La bibliothèque de signaux ([[ui:nav.signals]]) | Peu après chaque modification |
| `emulators.json` | La bibliothèque d’émulateurs ([[ui:nav.emulators]]) | Peu après chaque modification |
| `runs/run-<ms>-<job>.json` | Un rapport par exécution qui s’est terminée d’elle-même | Quand l’exécution se termine |
| `exports/experiment-<ms>-<16 hex digits>.json` | Un instantané de l’expérience | [[ui:exp.exportJson]] |
| `capture-<ms>.jsonl`, `capture-<ms>.txt` | Les trames de l’Inspecteur | [[ui:ins.exportJsonl]], [[ui:ins.exportTxt]] |
| `token` | Le jeton d’accès d’un serveur, lisible par son seul utilisateur | `--generate-token`, au premier démarrage |
| `.experiment-<hex>.tmp`, `.signals-<hex>.tmp`, `.emulators-<hex>.tmp` | Une sauvegarde en cours | Un instant, puis renommée |

`<ms>` est un temps en millisecondes depuis 1970 ; `<job>` est le numéro de tâche
de l’exécution. Sur un serveur, chaque navigateur travaille sur le même
`experiment.json`, les mêmes bibliothèques et les mêmes rapports.

## Formats {#formats}

Tous sont du JSON en UTF-8, écrit avec indentation pour bien se lire et se
comparer. Chacun porte une `version` ; un fichier d’une version plus ancienne est
lu et migré à son ouverture, puis réécrit dans la version courante à la prochaine
sauvegarde — après quoi un Signal Lab plus ancien ne peut plus l’ouvrir.

### experiment.json {#experiment-json}

Le document d’expérience, version 9 — le même JSON que [[ui:exp.exportJson]] écrit
et qu’[[ui:exp.importJson]] lit :

```json
{
  "version": 9,
  "name": "HTTP check",
  "params": [],
  "profiles": [],
  "profile": null,
  "seed": null,
  "cookies": true,
  "nodes": [ { "id": "start", "type": "start", "x": 40, "y": 80 }, … ],
  "edges": [ { "from": "start", "to": "request", "port": "next" }, … ]
}
```

- Au plus 4 Mio, et 1 à 64 nœuds (`doc.node_count`).
- Les versions 1 à 8 sont migrées à l’ouverture. Un fichier antérieur à la
  version 8 s’ouvre avec `cookies` désactivé, pour qu’il s’exécute comme avant ;
  les autres réglages que chaque version a ajoutés (les paramètres en 2, les
  profils en 3, les réessais en 4, les répétitions et les boucles en 5, les
  émulateurs en 6, les dégradations en 7, l’authentification WebSocket et HTTP
  en 8, la charge en 9) démarrent vides.
- Une version plus récente que celle que connaît ce Signal Lab est refusée
  (`doc.version_unsupported`) plutôt qu’ouverte sans ce qu’il ne peut pas lire.
- Un fichier qui ne s’analyse pas est signalé avec son chemin, sa ligne et sa
  colonne, et jamais remplacé.
- Il est écrit dans un fichier temporaire puis renommé, de sorte qu’une écriture
  échouée laisse le précédent.

Ce que sont les nœuds, les paramètres et les profils : [expériences](../experiments/index.md),
[nœuds](../experiments/nodes.md), [données](../experiments/data.md).

### signals.json {#signals-json}

La bibliothèque de signaux, version 2 :

```json
{
  "version": 2,
  "signals": [
    {
      "id": "…",
      "name": "Go cue",
      "group": "Stage/Cues",
      "note": "",
      "body": { "transport": "osc", "target": "127.0.0.1:9000", "address": "/cue/go", "args": [ { "type": "int", "value": 1 } ] }
    }
  ],
  "folders": [ "Stage", "Stage/Cues" ]
}
```

- `group` est le dossier du signal sous forme de chemin `/` ; vide, c’est le
  niveau supérieur. `folders` (ajouté en version 2) liste chaque dossier, y
  compris les vides, et est omis lorsqu’il n’y en a aucun. Un fichier version 1
  se lit de la même façon, sans les dossiers vides.
- `body` est l’un de `osc`, `udp`, `http` ou `mqtt` ; leurs champs sont dans
  [`signals_save`](../api/commands.md#signals_save).
- Lorsque le fichier n’existe pas, le jeu de départ est écrit — chaque cible sur
  `127.0.0.1` — et renommé dans la langue de l’interface.
- Un fichier qui ne s’analyse pas est signalé avec son chemin, sa ligne et sa
  colonne (`signals.json_invalid`) et n’est jamais remplacé par le jeu de départ :
  corrigez-le ou supprimez-le. Rien n’écrit la bibliothèque tant qu’elle ne se lit
  pas — une sauvegarde est refusée avec la même erreur et le fichier reste tel
  quel — jusqu’à ce qu’[[ui:sig.reload]] le relise. Une sauvegarde passe par un
  fichier temporaire dans le même dossier, donc une écriture interrompue laisse le
  fichier précédent.

### emulators.json {#emulators-json}

La bibliothèque d’émulateurs, version 1 :

```json
{
  "version": 1,
  "emulators": [
    { "id": "demo-api", "note": "…", "emulator": { "name": "Demo API", "bind": "127.0.0.1:8080", "protocol": "http", "routes": [ … ] } }
  ]
}
```

Chaque entrée est un document d’émulateur avec un `id` et une `note` ; le document
est décrit dans [émulateurs](../tools/emulators.md). Comme pour les signaux, un
fichier manquant reçoit le jeu de départ (chacun lié à `127.0.0.1`), et un fichier
cassé est signalé (`emulators.json_invalid`), jamais remplacé. Il est écrit via un
fichier temporaire. `signallab emulate` lit aussi un fichier qui lui est propre
contenant un émulateur, une liste de ceux-ci, ou une bibliothèque comme celle-ci.

### Rapports d’exécution {#run-reports}

`runs/run-<started ms>-<job>.json`, version de rapport 5 : un fichier par
exécution réussie ou échouée, jamais écrasé (une seconde exécution du même nom
reçoit `-2`, `-3`…). Une exécution arrêtée n’en enregistre aucun.

| Champ | Ce que c’est |
| --- | --- |
| `version` | 5 |
| `experiment` | Le nom de l’expérience |
| `document_version` | La version du document exécuté |
| `seed`, `profile` | Ce avec quoi elle a tourné |
| `overrides` | Les valeurs données pour cette exécution seulement |
| `params` | Chaque valeur de paramètre utilisée |
| `started_ms`, `ended_ms` | Millisecondes depuis 1970 |
| `outcome` | `passed` ou `failed` |
| `error` | Son premier échec, ou null |
| `steps` | Chaque étape, comme [`experiment://step`](../api/events.md#event-experiment-step) |
| `emulators` | Ce que chaque nœud [[ui:exp.node.emulator]] a reçu et répondu (depuis la version 3) ; omis lorsqu’il n’y en a aucun |
| `impairments` | Ce qu’a fait le relais de chaque nœud [[ui:exp.node.impairment]], phase par phase (depuis la version 4) ; omis lorsqu’il n’y en a aucun |

Les mesures d’une étape de charge sont sur sa dernière étape (depuis la version 5).
L’historique des exécutions de la chronologie et [[ui:exp.compare]] lisent ces
fichiers ; un rapport qui ne peut pas être lu est écarté de la liste. Voir
[exécutions et rapports](../experiments/runs.md).

### Exports {#exports}

- `exports/experiment-…json` : le document d’expérience, comme ci-dessus. Chaque
  export est un nouveau fichier.
- `capture-….jsonl` : une trame d’Inspecteur par ligne, avec les octets conservés
  dans `data`, en base64.
- `capture-….txt` : les trames à lire, chacune avec un vidage hexadécimal.

Sur un serveur, l’export de l’Inspecteur se télécharge sur votre ordinateur au fur
et à mesure ; l’export d’une expérience propose [[ui:common.download]], et le
[[ui:exp.reportSaved]] d’une exécution dans la chronologie est un lien qui
télécharge son rapport.

## Les secrets ne sont pas dans ces fichiers {#secrets}

Une expérience nomme un secret — `{{secret.API_TOKEN}}` — et seul le nom est
écrit. La valeur est conservée :

| Où Signal Lab s’exécute | Où sont les valeurs secrètes |
| --- | --- |
| Application de bureau, Windows | Gestionnaire d’informations d’identification Windows, sous `SignalLab` ([[ui:exp.secrets]] dans l’éditeur) |
| Application de bureau, Linux | Nulle part : les secrets ne peuvent pas être stockés (`secret.unsupported`) |
| Serveur | En lecture seule : la variable d’environnement `SIGNALLAB_SECRET_<NAME>`, ou le fichier `<NAME>` dans `--secrets-dir` (défaut `/run/secrets/signallab`) |
| `signallab` | Les mêmes fichiers et variables, ou le magasin du système avec `--secrets system` |

::: warning
Ce que vous saisissez directement dans un champ est conservé tel que vous l’avez
saisi. Un mot de passe dans les identifiants d’un signal HTTP, le mot de passe
d’un émulateur de broker MQTT, un jeton collé dans un en-tête — tout est en texte
clair dans `signals.json`, `emulators.json` ou `experiment.json`, et dans leurs
exports. Utilisez `{{secret.NAME}}` dans une expérience pour tout ce que vous ne
mettriez pas dans un dossier partagé.
:::

## Réglages de l’interface {#settings}

Ce que l’interface retient — sa langue, les valeurs saisies en dernier sur chaque
écran, le volet ouvert et sa taille, [[ui:http.keepCookies]], le moment du dernier
contrôle de mise à jour, et le nombre aléatoire de l’installation pour les mises à
jour — est conservé par l’interface elle-même, pas dans le dossier de données :
dans le stockage propre à l’application sur le bureau, dans le stockage de site du
navigateur pour la page d’un serveur (par navigateur). Les identifiants de l’écran
[[ui:nav.http]] n’y sont pas conservés.

Signal Lab n’écrit aucun fichier journal ; voir [dépannage](troubleshooting.md#logs).

## Sauvegarder, modifier, déplacer {#backup}

- **Sauvegarder** en copiant tout le dossier. Tout ce qu’il contient est du JSON
  autonome ; les valeurs secrètes n’y sont pas, redéfinissez-les donc sur une
  nouvelle machine.
- **Modifier** `signals.json`, `emulators.json` et `experiment.json` à la main
  pendant que Signal Lab est fermé (ou, sur un serveur, pendant qu’aucune page
  n’est ouverte) : l’application écrit tout le fichier à partir de ce qu’elle
  détient, donc une modification faite pendant qu’elle tourne est écrasée par sa
  prochaine sauvegarde. Une erreur est signalée avec la ligne et la colonne à la
  prochaine lecture du fichier, jamais remplacée en silence — pour `signals.json`,
  aussi lorsque l’application y sauvegarde : cette sauvegarde est refusée et le
  fichier reste tel que vous l’avez laissé.
- **Supprimer** les fichiers `runs/`, `exports/` et `capture-*` à tout moment.
  Supprimer `signals.json` ou `emulators.json` ramène le jeu de départ ; supprimer
  `experiment.json` ramène l’expérience de départ.
- **Déplacer** le dossier en le copiant et en indiquant le nouvel emplacement à
  Signal Lab : `--data-dir` pour un serveur, `SIGNALLAB_DATA_DIR` pour
  l’application de bureau.
- **Partager** une expérience en l’exportant, ou en validant son JSON à côté du
  projet qu’elle teste ; [`signallab run`](../automation/cli.md#cli-run) l’exécute
  depuis là.

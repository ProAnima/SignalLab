---
title: "Commandes"
description: "Chaque commande du moteur de Signal Lab, appelée en POST /api/invoke/<command>, avec ses arguments, son résultat et ses erreurs."
---

# Commandes

Chaque commande du moteur, groupée par ce sur quoi elle porte. Chacune s’appelle en
`POST /api/invoke/<command>` avec un objet JSON d’arguments, et répond `200`
avec son résultat ou `422` avec un [`EngineError`](index.md#errors). Comment
s’authentifier et ce que signifient les statuts est dans [la présentation de l’API](index.md).

## Conventions {#conventions}

- **Les noms d’arguments** sont en camelCase (`jobId`, `nodeId`). Un argument
  qu’une commande ne connaît pas, ou un argument requis omis, est refusé avec
  `command.args_invalid` ; toute commande qui prend des arguments peut échouer avec lui.
  Une commande sans argument ne lit pas le corps.
- **Les objets passés en argument** — `config`, `request`, `document`,
  `library`, `emulator`, `profile` — utilisent les noms de champs du moteur, le plus
  souvent en snake_case (`timeout_ms`). À l’intérieur, un champ que le moteur ne connaît pas est
  **ignoré**, si bien qu’un champ optionnel mal orthographié garde silencieusement sa
  valeur par défaut. Seul le formulaire de `feedback_send` refuse les champs inconnus.
- **Les arguments et champs optionnels** peuvent être omis ou envoyés à `null` ; les
  tableaux donnent leurs valeurs par défaut.
- **Les résultats** sont du JSON. « null » signifie que la commande n’a rien à retourner.
- **Les adresses** écrites `IP:port` prennent une adresse numérique et un port
  (`127.0.0.1:9000`, `[::1]:9000`) ; un nom d’hôte y est refusé. Là où un tableau
  dit `IP:port` ou `host:port`, un nom d’hôte fonctionne aussi : il est
  résolu quand la commande s’exécute, et son adresse IPv4 est utilisée quand il en a une
  (ainsi `localhost:9000` est `127.0.0.1:9000`).
- **Les tâches** : une commande marquée *Démarre une tâche* renvoie un
  [`JobInfo`](#type-jobinfo) ; le travail continue jusqu’à ce qu’il finisse ou soit arrêté avec
  [`job_stop`](#job_stop). Voir [les tâches](index.md#jobs).
- **Les chemins** dans les résultats sont sur la machine où tourne le moteur — sur un serveur,
  dans son dossier de données ; téléchargez-les avec [`/api/files`](index.md#files).

Les exemples utilisent cette fonction de shell :

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
invoke() {
  curl -sS -X POST "$SERVER/api/invoke/$1" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    --data "${2:-}"
}
```

## Application {#application}

### app_info {#app_info}

Ce qu’est le moteur et où il tourne. Aucun argument.

**Résultat**

| Champ | Type | Signification |
| --- | --- | --- |
| `version` | string | La version de Signal Lab, `[[version]]` |
| `mode` | string | `desktop` ou `server` |
| `secrets_writable` | boolean | Si [`secret_set`](#secret_set) et [`secret_delete`](#secret_delete) peuvent fonctionner ici : `false` sur un serveur |
| `data_dir` | string | Le dossier de données, sur la machine où tourne le moteur |
| `os` | string | `windows`, `linux`… |
| `arch` | string | `x86_64`, `aarch64`… |

### get_host_info {#get_host_info}

Le nom de la machine et l’adresse depuis laquelle elle enverrait. Aucun argument.

**Résultat** : `{ "local_ip": string, "hostname": string }`. `local_ip` est
l’adresse IPv4 que le système choisit pour le trafic vers Internet (trouvée sans rien
envoyer), ou `127.0.0.1` quand il n’y en a pas. `hostname` est le
nom de l’ordinateur, ou `localhost` quand le système ne le dit pas.

### firewall_status {#firewall_status}

Si le pare-feu du système laisse d’autres machines atteindre ce programme. Seul
Windows a un pare-feu par programme à lire ; ailleurs `applies` vaut `false`, et
parmi le reste seul `program` est rempli. Aucun argument.

**Résultat**

| Champ | Type | Signification |
| --- | --- | --- |
| `applies` | boolean | Il y a ici un pare-feu par programme (Windows) |
| `program` | string | Le programme dont les règles parlent |
| `enabled` | boolean | Le pare-feu est actif pour le réseau sur lequel la machine se trouve maintenant |
| `networks` | string[] | Les types de réseau sur lesquels la machine se trouve : `domain`, `private`, `public` |
| `allowed` | boolean | Une règle d’entrée laisse passer l’UDP pour ce programme sur le réseau actuel |
| `blocked` | boolean | Une règle d’entrée bloque ce programme sur le réseau actuel ; elle l’emporte sur toute règle d’autorisation |
| `rules` | number | Les règles d’entrée pour ce programme, de tout type |

**Erreurs** : `firewall.failed`.

### firewall_allow {#firewall_allow}

Laisse d’autres machines atteindre Signal Lab : le système affiche sa propre invite
d’administrateur, puis les règles d’entrée du programme (une règle de blocage comprise) sont
remplacées par une règle d’autorisation pour Signal Lab et une pour la ligne de commande
`signallab` voisine. Application de bureau sous Windows uniquement ; un serveur refuse,
car personne n’est devant son écran pour répondre à l’invite.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `public` | boolean | oui | Autoriser aussi sur les réseaux publics, pas seulement privés et de domaine |

**Résultat** : le nouveau [`firewall_status`](#firewall_status).

**Erreurs** : `firewall.server` (sur un serveur), `firewall.unsupported` (pas
Windows), `firewall.declined` (l’invite a reçu Non), `firewall.failed`.

### feedback_send {#feedback_send}

Envoie un message aux développeurs de Signal Lab, par le hub du studio, qui
le leur transmet par courriel.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `form` | object | oui | Le message, ci-dessous. Les champs inconnus sont refusés |

| Champ de `form` | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `message` | string | — | Ce qui s’est passé ; requis, 20 000 caractères au plus |
| `email` | string | aucun | Où une réponse peut aller |
| `meta` | object of strings | `{}` | Ce que l’application dit d’elle-même (version, os, arch, mode, lang, screen) |
| `screenshots` | `{ name, data }[]` | `[]` | Des images, `data` en base64 ; 6 au plus, 8 Mio chacune |
| `logs` | `{ name, text }[]` | `[]` | Des fichiers texte ; 4 au plus, 2 Mio chacun |

L’ensemble fait au plus 15 Mio.

**Résultat** : `{ "id": string }`, la référence que reçoivent les développeurs.

**Erreurs** : `feedback.message_required`, `feedback.message_too_long`,
`feedback.too_many_files`, `feedback.file_too_large`, `feedback.too_large`,
`feedback.invalid`, les refus du hub (`feedback.email_invalid`,
`feedback.file_type`, `feedback.rate_limited`, `feedback.disabled`,
`feedback.send_failed`, `feedback.failed`), et les `transport.*` du réseau.

```bash
invoke app_info
# {"version":"[[version]]","mode":"server","secrets_writable":false,"data_dir":"/data","os":"linux","arch":"x86_64"}
```

## Jobs {#jobs}

### jobs_list {#jobs_list}

Les tâches en cours, la plus ancienne d’abord. Aucun argument.

**Résultat** : [`JobInfo`](#type-jobinfo)`[]`.

### job_stop {#job_stop}

Arrête une tâche sur-le-champ : ses sockets se ferment, son relais, son serveur ou sa
connexion disparaît. Une tâche arrêtée n’envoie pas de
[`job://ended`](events.md#event-job-ended) ; une exécution arrêtée
n’enregistre pas de rapport.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `id` | number | oui | L’`id` de la tâche |

**Résultat** : `true` quand une tâche avec cet id était en cours, `false` sinon.

### jobs_stop_all {#jobs_stop_all}

Arrête toutes les tâches en cours, peu importe qui les a lancées. Aucun argument.

**Résultat** : null.

```bash
invoke jobs_list
# [{"id":3,"kind":"osc-monitor","label":"OSC monitor 0.0.0.0:9000","params":{"bind":"0.0.0.0:9000"},"started_ms":1759600000000}]
invoke job_stop '{"id":3}'
# true
```

## Experiments and runs {#experiments}

Ces commandes prennent et renvoient un document d’expérience (`Experiment`) : le JSON
que l’éditeur enregistre et exporte, avec `version`, `name`, `params`, `profiles`,
`profile`, `seed`, `cookies`, `nodes` et `edges`. Ses nœuds sont dans
[les nœuds](../experiments/nodes.md) ; ses paramètres, profils et modèles dans
[les données](../experiments/data.md). Un document fait au plus 4 Mio
(`file.too_large`). Pour exécuter une expérience et attendre son résultat, utilisez
[`POST /api/run`](run.md) plutôt que [`experiment_start`](#experiment_start).

### experiment_load {#experiment_load}

L’expérience de travail : `experiment.json` dans le dossier de données — sur un serveur,
celle que montre son interface. Quand il n’y en a pas, l’expérience de départ. Les versions
plus anciennes du document sont migrées. Aucun argument.

**Résultat** : `Experiment`.

**Erreurs** : `file.io`, `file.json_invalid` (avec le `path`, la `line`
et la `column` du fichier), `file.too_large`, `doc.version_unsupported` et les autres
contrôles `doc.*`.

### experiment_save {#experiment_save}

Remplace l’expérience de travail, `experiment.json` dans le dossier de données. Elle est
d’abord écrite dans un fichier temporaire, si bien qu’une écriture ratée laisse la précédente.

::: warning
Sur un serveur, c’est le document sur lequel travaille l’éditeur de chaque navigateur.
:::

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `document` | `Experiment` | oui | Le document |

**Résultat** : une chaîne, le chemin écrit.

**Erreurs** : `doc.*`, les contrôles de taille du document (`param.*`, `params.too_many`,
`profile.*`, `profiles.too_many`, `seed.range`), `file.too_large`, `file.io`.

### experiment_parse {#experiment_parse}

Lit une expérience à partir de texte JSON, comme le fait [[ui:exp.importJson]]. Les
versions 1 à 8 sont migrées vers la version 9, l’actuelle ; un fichier antérieur à la
version 8 s’ouvre avec `cookies` désactivé, si bien qu’il s’exécute comme avant. Un
indicateur d’ordre des octets est ignoré.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `text` | string | oui | Le texte du fichier |

**Résultat** : `Experiment`.

**Erreurs** : `file.json_invalid` (`line`, `column`), `file.too_large`,
`doc.version_unsupported`, `doc.*`, les contrôles de taille de
[`experiment_save`](#experiment_save).

### experiment_export {#experiment_export}

Écrit un instantané d’un document dans `exports/experiment-<ms>-<16 hex digits>.json`
dans le dossier de données. Chaque export est un nouveau fichier.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `document` | `Experiment` | oui | Le document |

**Résultat** : une chaîne, le chemin écrit.

**Erreurs** : celles de [`experiment_save`](#experiment_save).

### experiment_validate {#experiment_validate}

Vérifie qu’un document s’exécuterait avec son profil actif (ou ses valeurs par défaut) :
le graphe, chaque champ, les paramètres, et que chaque secret qu’il nomme est stocké.
Un problème bloquant constitue l’erreur. En cas de succès, elle dit lesquels des *autres*
profils échoueraient, si bien que vous le savez avant de changer.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `document` | `Experiment` | oui | Le document |
| `overrides` | object of strings | non | Des valeurs de paramètres pour ce contrôle uniquement, telles que les donne [[ui:exp.runWith]] |

**Résultat** : `{ "profile": string or null, "error": EngineError }[]` — chaque
autre profil qui ne validerait pas (`null` : les valeurs par défaut, sans
profil). Une liste vide signifie que tous les profils conviennent.

**Erreurs** : tout code de validation (`doc.*`, `graph.*`, `node.*`, `param.*`,
`profile.*`, `template.*`, `loop.*`…), `run.override_unknown` (un remplacement
pour un paramètre que le document n’a pas), `secret.missing`,
`secret.store`, `secret.unsupported`.

### experiment_resolve {#experiment_resolve}

Un nœud avec ses modèles remplis, comme le montre l’aperçu de l’éditeur : les
valeurs du profil actif et les valeurs de variables que vous donnez. Les secrets sont affichés
`••••`, jamais leur valeur. Les noms qui n’ont pas de valeur restent tels qu’écrits et
sont listés.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `document` | `Experiment` | oui | Le document |
| `nodeId` | string | oui | Le nœud |
| `vars` | object | oui | Les valeurs de variables à utiliser, par nom ; `{}` pour aucune |

**Résultat** : `{ "node": node, "missing": string[] }`.

**Erreurs** : `node.not_found`, `template.*`, `secret.store`,
`secret.unsupported`.

### experiment_send_node {#experiment_send_node}

[[ui:exp.sendNow]] : exécute un nœud tout seul, par le même code qu’utilise une
exécution. Une action est envoyée ; une attente écoute à partir de maintenant jusqu’à ce qu’elle
corresponde ou expire. Un nœud [[ui:exp.node.ws_send]] ou [[ui:exp.node.wait_ws]] ouvre la
connexion que décrit son nœud [[ui:exp.node.ws_connect]]. Rien n’est envoyé avec
des cookies, et les nœuds [[ui:exp.node.impairment]] et [[ui:exp.node.emulator]]
de l’exécution ne sont pas ouverts.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `document` | `Experiment` | oui | Le document ; son profil actif donne les valeurs des paramètres |
| `nodeId` | string | oui | Une action ou une attente |
| `vars` | object | oui | Les valeurs de variables que lisent les modèles du nœud ; `{}` pour aucune |

**Résultat**

| Champ | Type | Signification |
| --- | --- | --- |
| `detail` | string | Ce qui s’est passé, en anglais |
| `response` | [`HttpResponse`](#type-httpresponse) or null | La réponse d’un nœud HTTP |
| `vars` | object | Ce que l’étape a défini : la réponse d’une attente, ou ce que les nœuds [[ui:exp.node.extract]] après une requête prennent de sa réponse |

Les valeurs de secrets y sont masquées partout.

**Erreurs** : `node.not_found`, `run.not_an_action` (ni une action ni une
attente), `ws.connection_unknown`, `secret.missing`, `template.*`, et ce avec quoi
l’étape échoue : `transport.*`, `wait.timeout`, `check.*`…

### experiment_start {#experiment_start}

Démarre une exécution, comme le fait [[ui:exp.run]], et revient aussitôt. Ses étapes arrivent comme
événements [`experiment://step`](events.md#event-experiment-step), sa fin comme
[`experiment://ended`](events.md#event-experiment-ended), et son rapport est
enregistré sous `runs/` dans le dossier de données. Les attentes, les émulateurs, les relais de dégradation et
les abonnements MQTT s’ouvrent avant la première étape, si bien qu’un port déjà pris échoue
ici. Une exécution de plus de 300 s échoue avec `run.timeout`. *Démarre une tâche*
(`experiment`).

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `document` | `Experiment` | oui | Le document |
| `overrides` | object of strings | non | Des valeurs de paramètres pour cette exécution uniquement |
| `seed` | number | non | La graine de l’exécution, de 0 à 9007199254740991 ; par défaut : celle du document, sinon une nouvelle |

**Résultat** : [`JobInfo`](#type-jobinfo), `params.name` le nom de l’expérience.

**Erreurs** : tout ce que rapporte [`experiment_validate`](#experiment_validate),
`seed.range`, `transport.address_in_use` et les autres échecs de liaison,
`emulator.*`, `impair.*`, `node.params_only` (une adresse d’écoute ou le broker ou le topic d’une attente
MQTT qui n’est pas fixé au démarrage de l’exécution), et les
erreurs de [`mqtt_connect`](#mqtt_connect) d’un broker qu’une attente MQTT ne peut pas atteindre.

### experiment_runs {#experiment_runs}

Les exécutions relues depuis leurs rapports dans `runs/`, la plus récente d’abord. Un rapport qui
ne peut pas être lu est laissé de côté.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `name` | string | non | Seulement les exécutions de l’expérience portant exactement ce nom |
| `limit` | number | non | Au plus ce nombre ; par défaut 50, au plus 500 |

**Résultat** : des résumés d’exécution :

| Champ | Type | Signification |
| --- | --- | --- |
| `name` | string | Le nom de fichier du rapport, `run-<ms>-<job>.json` : ce que prend [`experiment_compare`](#experiment_compare) |
| `experiment` | string | Le nom de l’expérience |
| `started_ms`, `ended_ms` | number | Des millisecondes depuis 1970 |
| `outcome` | string | `passed` ou `failed` |
| `seed` | number | La graine de l’exécution |
| `profile` | string or null | Son profil |
| `loads` | object[] | Chaque étape de charge : `node`, `sent`, `rps`, `p95_ms`, `error_rate`, `held` (tous les seuils tenus) |

**Erreurs** : `file.io`.

### experiment_compare {#experiment_compare}

Deux exécutions côte à côte, étape de charge par étape de charge, comme les montre
[[ui:exp.compare]] dans la chronologie. Les étapes sont appariées par id de nœud.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `a` | string | oui | Le nom de fichier du rapport de l’exécution antérieure |
| `b` | string | oui | Le nom de fichier du rapport de l’exécution postérieure |

**Résultat** : `{ "a": summary, "b": summary, "steps": [...] }`, chaque étape avec
`node`, `missing_in` (`a` ou `b`, quand une seule exécution la possède), `metrics`,
`sent` (`[a, b]`), `thresholds_a` et `thresholds_b` (chaque seuil sous la forme
`{ metric, op, value, actual, held }`). `metrics` liste neuf mesures, chacune sous la forme
`{ metric, a, b, change, percent, worse }` : `metric` vaut `p50_ms`, `p90_ms`,
`p95_ms`, `p99_ms`, `mean_ms`, `max_ms`, `error_rate`, `rps` ou `missed` ;
`change` est `b − a` ; `percent` la variation en % de `a` (null quand `a` vaut 0) ;
`worse` le fait qu’elle a évolué dans le mauvais sens — plus haute, ou plus basse pour `rps` — de 5 %
ou plus, ou de 0 à n’importe quoi. Une étape que seule une exécution possède n’est jamais
`worse`. Voir [charge](../experiments/load.md).

**Erreurs** : `runs.name_invalid` (autre chose qu’un nom de fichier de rapport, sans dossier),
`runs.not_found`, `file.json_invalid`, `file.io`.

```bash
invoke experiment_validate "$(jq '{document: .}' experiment.json)"
# []
```

## Secrets {#secrets}

Les valeurs de secrets sont utilisées par les expériences sous la forme `{{secret.NAME}}` et ne quittent jamais le
moteur : aucune commande n’en renvoie. L’endroit où elles sont conservées dépend de l’endroit où tourne le moteur :

| Où | Stockage | Définir et supprimer |
| --- | --- | --- |
| Application de bureau, Windows | Gestionnaire d’informations d’identification Windows | Oui |
| Application de bureau, Linux | Aucun | `secret.unsupported` |
| Serveur | `SIGNALLAB_SECRET_<NAME>`, ou le fichier `<NAME>` dans `--secrets-dir` (par défaut `/run/secrets/signallab`) | Non : `secret.read_only` |

Un nom commence par une lettre latine ou `_`, se poursuit par des lettres latines, des chiffres
et `_`, et compte au plus 128 caractères (`secret.name_invalid`).

### secret_status {#secret_status}

Lesquels des noms donnés ont une valeur stockée.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `names` | string[] | oui | Les noms à rechercher |

**Résultat** : un objet, nom → `true` (stocké) ou `false`.

**Erreurs** : `secret.name_invalid` (sur un serveur), `secret.store`,
`secret.too_large` (le fichier d’un serveur de plus de 16 Kio), `secret.unsupported`.

### secret_set {#secret_set}

Stocke une valeur sous un nom, en remplaçant celle qui s’y trouvait. Application de bureau uniquement.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `name` | string | oui | Le nom |
| `value` | string | oui | Non vide ; 16 Kio au plus |

**Résultat** : null.

**Erreurs** : `secret.read_only` (sur un serveur), `secret.unsupported`,
`secret.name_invalid`, `secret.empty`, `secret.too_large`, `secret.store`.

### secret_delete {#secret_delete}

Supprime une valeur stockée. En supprimer une qui n’est pas stockée n’est pas une erreur.
Application de bureau uniquement.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `name` | string | oui | Le nom |

**Résultat** : null.

**Erreurs** : `secret.read_only` (sur un serveur), `secret.unsupported`,
`secret.name_invalid`, `secret.store`.

```bash
invoke secret_status '{"names":["API_TOKEN","MQTT_PASSWORD"]}'
# {"API_TOKEN":true,"MQTT_PASSWORD":false}
```

## OSC {#osc}

Voir [OSC](../protocols/osc.md) pour l’écran que servent ces commandes.

### osc_send {#osc_send}

Envoie un message OSC dans un datagramme UDP, depuis un socket neuf.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `target` | string | oui | `IP:port` ou `host:port` où envoyer ; un nom est résolu, son adresse IPv4 prise quand il en a une |
| `address` | string | oui | L’adresse OSC, `/mixer/fader/1` ; elle commence par `/` |
| `args` | [`OscArg`](#type-oscarg)`[]` | oui | Les arguments ; `[]` pour aucun |

**Résultat** : un nombre, les octets envoyés.

**Erreurs** : `node.osc_address` (pas de `/` initial ; champ `address`),
`transport.target_invalid` (pas de port, ou aucune des deux formes), `transport.dns` (le
nom ne se résout pas), `transport.*`.

### osc_monitor_start {#osc_monitor_start}

Écoute l’OSC sur un port UDP et décode chaque paquet. Chacun arrive comme événement
[`osc://message`](events.md#event-osc-message). *Démarre une tâche*
(`osc-monitor`, `params.bind`).

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `bind` | string | oui | `IP:port` sur lequel écouter : `0.0.0.0:9000` toutes les cartes réseau, `127.0.0.1:9000` cette machine seulement |

**Résultat** : [`JobInfo`](#type-jobinfo).

**Erreurs** : `node.bind_invalid`, `transport.address_in_use`,
`transport.address_unavailable`, `transport.denied`, `wait.bind_failed`. La
tâche se termine avec `wait.receive_failed` si le socket ne peut plus recevoir.

### osc_generator_start {#osc_generator_start}

Envoie un flux de messages OSC dont l’unique argument suit une forme d’onde.
La progression arrive comme [`osc://gen-tick`](events.md#event-osc-gen-tick). *Démarre
une tâche* (`osc-gen`, `params.target`, `params.address`).

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `config` | object | oui | Ci-dessous |

| Champ de `config` | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `target` | string | — | `IP:port` ou `host:port` où envoyer ; un nom est résolu une fois, au démarrage de la tâche |
| `address` | string | — | L’adresse OSC ; elle commence par `/` |
| `rate` | number | — | Des messages par seconde, maintenus entre 0,1 et 5000 |
| `waveform` | string | — | `sine`, `triangle`, `saw` (descendante : de `max` à `min`, puis retour d’un coup), `ramp` (montante : de `min` à `max`, puis retour d’un coup), `square`, `random` ou `constant` (`max`) |
| `freq` | number | — | Cycles de la forme d’onde par seconde |
| `min`, `max` | number | — | La plage de la valeur |
| `as_int` | boolean | `false` | Arrondir et envoyer un int au lieu d’un float |
| `duration_s` | number | `0` | S’arrêter après ce nombre de secondes ; 0 tourne jusqu’à l’arrêt |

**Résultat** : [`JobInfo`](#type-jobinfo).

**Erreurs** : `node.osc_address`, `transport.target_invalid`, `transport.dns`,
`transport.*`. La tâche se termine avec une erreur `transport.*` si un envoi échoue.

```bash
invoke osc_send '{"target":"127.0.0.1:9000","address":"/cue/go","args":[{"type":"int","value":1}]}'
# 16
invoke osc_monitor_start '{"bind":"0.0.0.0:9000"}'
```

## HTTP and cookies {#http}

Voir [HTTP](../protocols/http.md).

### http_request {#http_request}

Envoie une requête HTTP et renvoie la réponse. Une requête qui n’obtient pas de
réponse — refusée, expirée, un nom qui ne se résout pas, un certificat non approuvé — n’est **pas**
une erreur de la commande : la réponse le dit dans `error` et `cause`.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `request` | [`HttpRequest`](#type-httprequest) | oui | La requête |
| `cookies` | boolean | non | Envoyer la réserve de cookies de l’écran [[ui:nav.http]] et conserver ce que la réponse définit ; par défaut `false` |

**Résultat** : [`HttpResponse`](#type-httpresponse).

**Erreurs** : `http.client_failed` (la requête n’a même pas pu être préparée).

### http_burst_start {#http_burst_start}

Envoie une requête de nombreuses fois, plusieurs à la fois, et la mesure. Sans
`rate`, chaque ouvrier renvoie dès qu’il a une réponse ; avec une cadence, les requêtes
démarrent selon un calendrier fixe même si les réponses sont lentes, et une requête
qui a attendu plus de 50 ms après son instant pour un ouvrier libre est sautée et
comptée comme manquée. La progression arrive comme
[`http://burst-progress`](events.md#event-http-burst-progress) dix fois par
seconde. *Démarre une tâche* (`http-burst`, `params.method`, `params.url`, et
`params.rate` quand c’est cadencé).

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `config` | object | oui | Les champs de [`HttpRequest`](#type-httprequest) et ceux ci-dessous, dans un seul objet |

| Champ de `config` | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `concurrency` | number | — | Au plus ce nombre en vol, maintenu entre 1 et 512 |
| `total` | number | `0` | S’arrêter après ce nombre de requêtes ; 0 : sans limite |
| `duration_s` | number | `0` | S’arrêter après ce nombre de secondes ; 0 : sans limite de temps |
| `rate` | number | `0` | Requêtes démarrées par seconde, de 0,1 à 100 000 ; 0 : aussi vite que les réponses arrivent |
| `cookies` | boolean | `false` | Utiliser la réserve de cookies de l’écran [[ui:nav.http]] |

Sans `total` ni `duration_s`, la rafale tourne jusqu’à l’arrêt.

**Résultat** : [`JobInfo`](#type-jobinfo).

**Erreurs** : `http.rate_invalid`, `http.duration_invalid`, `http.client_failed`.

### http_cookies {#http_cookies}

La réserve de cookies de l’écran [[ui:nav.http]] : chaque cookie qui n’a pas expiré. Sur un serveur,
il y a une réserve pour chaque page et chaque script. Aucun argument.

**Résultat** : des cookies, chacun avec `name`, `value`, `domain`, `host_only` (pas d’attribut
Domain : seul l’hôte qui l’a défini le reçoit), `path`, `expires`
(en secondes Unix, null pour un cookie de session), `secure`, `http_only` et
`same_site` (une chaîne ou null).

### http_cookies_clear {#http_cookies_clear}

Vide la réserve de cookies de l’écran [[ui:nav.http]]. Aucun argument.

**Résultat** : null.

```bash
invoke http_request '{"request":{"method":"GET","url":"http://127.0.0.1:8080/health","headers":[["Accept","application/json"]],"body":null,"timeout_ms":5000}}' \
  | jq '{status, latency_ms, body}'
```

## WebSocket {#websocket}

Voir [WebSocket](../protocols/websocket.md). Une connexion ouverte par `ws_connect`
est une tâche ; les autres la nomment par `jobId`.

### ws_connect {#ws_connect}

Ouvre un WebSocket et le garde ouvert. Ce qui arrive et ce qui est envoyé vient comme
[`ws://messages`](events.md#event-ws-messages) toutes les 100 ms ; l’état de la connexion comme
[`ws://state`](events.md#event-ws-state). *Démarre une tâche*
(`websocket`, `params.url`).

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | oui | Où et comment se connecter |

**Résultat** : [`JobInfo`](#type-jobinfo).

**Erreurs** : `ws.url_invalid`, `ws.header_invalid`, `ws.protocol_invalid`,
`ws.handshake_status` (le serveur a répondu à la mise à niveau avec un autre statut),
`ws.subprotocol_refused`, `ws.handshake_failed`, `transport.*`.

### ws_send {#ws_send}

Envoie un message sur une connexion ouverte.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `jobId` | number | oui | La tâche de la connexion |
| `message` | object | oui | `{ "text": "…" }` pour un message texte, ou `{ "hex": "de ad be ef" }` pour un binaire ; exactement l’un des deux |

**Résultat** : un nombre, les octets envoyés.

**Erreurs** : `ws.not_connected`, `ws.payload_required` (aucun des deux ou les deux),
`hex.invalid` (un `hex` vide aussi), `node.too_long` (plus de 16 Mio, champ
`payload` ; rien n’est envoyé et la connexion reste ouverte), `ws.closed`, `transport.*` (`transport.timeout` quand le serveur a cessé de
lire pendant 10 s).

### ws_close {#ws_close}

Ferme une connexion par une poignée de main de fermeture et attend jusqu’à 2 s la réponse du
serveur ; la tâche se termine alors. Une fois qu’une connexion est terminée, sa tâche a disparu et
la fermer donne `ws.not_connected`.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `jobId` | number | oui | La tâche de la connexion |
| `code` | number | non | 1000, ou de 3000 à 4999 pour un code propre à une application ; par défaut 1000 |
| `reason` | string | non | 123 octets au plus ; par défaut vide |

**Résultat** : `{ "code", "reason", "by", "error" }` — `by` vaut `client`,
`server` ou `lost` ; `code` vaut 1005 quand la fermeture n’en portait aucun et 1006 quand
il n’y avait pas de trame de fermeture.

**Erreurs** : `ws.close_code`, `node.too_long`, `ws.not_connected`.

### ws_exchange {#ws_exchange}

Un échange sans tâche : se connecter, envoyer un message s’il est donné, attendre une
réponse si on le demande, fermer.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | oui | Où et comment se connecter |
| `message` | object | non | `{ "text" }` ou `{ "hex" }`, comme pour [`ws_send`](#ws_send) |
| `expect` | object | non | Ce qu’il faut attendre : `mode` (`any`, `contains`, `regex`, `hex` ; par défaut `any`), `pattern` (par défaut vide), `timeout_ms` (par défaut 2000) |

Avec `expect` et sans `message`, le premier message correspondant après la connexion
compte — un message de bienvenue.

**Résultat** : `{ "handshake", "sent", "reply", "closed" }` — `handshake` est
`{ url, peer, local, protocol, ms }` ; `sent` les octets envoyés ou null ; `reply`
`{ kind, text, hex, bytes, json, ms }` ou null (`json` : une réponse texte analysée,
sinon null ; `ms` : depuis l’envoi, ou depuis la connexion quand rien n’a été envoyé) ;
`closed` comme le renvoie [`ws_close`](#ws_close).

**Erreurs** : celles de [`ws_connect`](#ws_connect) et de [`ws_send`](#ws_send),
`wait.timeout` (avec `ms`, `unmatched` et `target`), `regex.invalid` et
`hex.invalid` (un `pattern` qui ne s’analyse pas).

```bash
invoke ws_exchange '{"config":{"url":"ws://127.0.0.1:9001/"},"message":{"text":"{\"type\":\"ping\"}"},"expect":{"mode":"contains","pattern":"pong"}}' \
  | jq .reply.text
```

## MQTT {#mqtt}

MQTT 3.1.1 sur TCP simple, QoS 0, 1 et 2. Voir [MQTT](../protocols/mqtt.md).

### mqtt_connect {#mqtt_connect}

Se connecte à un broker et garde la connexion. La commande revient une fois que le
broker a accepté la connexion (CONNACK), si bien qu’un mauvais mot de passe ou un port fermé
constitue son erreur. Les messages arrivent comme
[`mqtt://messages`](events.md#event-mqtt-messages) toutes les 100 ms ; les changements d’état
comme [`mqtt://state`](events.md#event-mqtt-state) ; les publications et désabonnements QoS 1/2
terminés comme [`mqtt://ack`](events.md#event-mqtt-ack).
*Démarre une tâche* (`mqtt`, `params.broker`, `params.client`).

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | oui | Le broker et comment s’y connecter |

**Résultat** : [`JobInfo`](#type-jobinfo).

**Erreurs** : `mqtt.client_id_required`, `transport.*` (refusé, injoignable,
`dns`, `timeout` après 6 s), `mqtt.no_answer` (aucun CONNACK dans les 6 s),
`mqtt.protocol`, `mqtt.refused_protocol`, `mqtt.refused_client_id`,
`mqtt.refused_unavailable`, `mqtt.refused_credentials`,
`mqtt.refused_not_authorized`, `mqtt.refused`.

### mqtt_publish {#mqtt_publish}

Publie sur une connexion ouverte.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `jobId` | number | oui | La tâche de la connexion |
| `topic` | string | oui | Le topic : non vide, et sans `+` ni `#` |
| `payload` | string | oui | La charge utile, envoyée en UTF-8 |
| `qos` | number | oui | 0, 1 ou 2 (au-dessus de 2 est envoyé comme 2) |
| `retain` | boolean | oui | Demander au broker de le conserver ; une charge utile vide avec `retain` efface une valeur retenue |

**Résultat** : null. Une publication QoS 1 ou 2 est confirmée plus tard par
[`mqtt://ack`](events.md#event-mqtt-ack).

**Erreurs** : `node.topic_wildcard` (champ `topic`) et `mqtt.topic_required`
(champ `topic`), comme pour [`mqtt_publish_once`](#mqtt_publish_once) — la commande
les refuse avant de chercher la connexion ; `mqtt.not_connected`.

### mqtt_subscribe {#mqtt_subscribe}

Abonne une connexion ouverte à des filtres. Ce que le broker accorde arrive comme
[`mqtt://state`](events.md#event-mqtt-state) avec `state: "subscribed"`.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `jobId` | number | oui | La tâche de la connexion |
| `filters` | `{ filter, qos }[]` | oui | Au moins un ; `qos` vaut 0 par défaut. `+` et `#` sont des jokers |

**Résultat** : null.

**Erreurs** : `mqtt.filter_required`, `mqtt.not_connected`.

### mqtt_unsubscribe {#mqtt_unsubscribe}

Désabonne une connexion ouverte de filtres.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `jobId` | number | oui | La tâche de la connexion |
| `filters` | string[] | oui | Au moins un |

**Résultat** : null. La réponse du broker arrive comme
[`mqtt://ack`](events.md#event-mqtt-ack) avec `kind: "unsubscribed"`.

**Erreurs** : `mqtt.filter_required`, `mqtt.not_connected`.

### mqtt_publish_once {#mqtt_publish_once}

Se connecte, publie un message, attend l’acquittement que son QoS demande
(jusqu’à 6 s), se déconnecte. Il apporte sa propre connexion, sous un identifiant de client
qui lui est propre — les 12 premiers caractères de `client_id`, `-o` et un numéro — si bien qu’il
ne déloge jamais du broker une connexion active avec cet id.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | oui | Le broker ; `subscribe` n’est pas utilisé |
| `topic` | string | oui | Non vide et sans `+` ni `#` |
| `payload` | string | oui | Envoyée en UTF-8 |
| `qos` | number | oui | 0, 1 ou 2 (au-dessus de 2 est envoyé comme 2) |
| `retain` | boolean | oui | Demander au broker de le conserver |

**Résultat** : une chaîne, un résumé écrit par Signal Lab :
`<topic> → <broker> · <bytes> B · qos<n>`, avec ` retained` quand c’est retenu.

**Erreurs** : `mqtt.topic_required`, `node.topic_wildcard`, et celles de
[`mqtt_connect`](#mqtt_connect) sauf `mqtt.client_id_required` : un
`client_id` vide est accepté ici.

```bash
invoke mqtt_publish_once '{"config":{"host":"127.0.0.1","port":1883,"client_id":"lab"},"topic":"lab/lamp/set","payload":"ON","qos":1,"retain":false}'
# "lab/lamp/set → 127.0.0.1:1883 · 2 B · qos1"
```

## Broadcast, multicast and discovery {#broadcast}

Voir [diffusion et découverte](../protocols/broadcast.md).

::: danger
La diffusion et un balayage atteignent chaque hôte d’un segment réseau. N’envoyez que sur
des réseaux dont vous êtes responsable.
:::

### broadcast_send {#broadcast_send}

Envoie un datagramme à chaque cible, une fois.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `config` | object | oui | Ci-dessous |

| Champ de `config` | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `mode` | string | — | `list`, `broadcast`, `multicast` ou `sweep` |
| `target` | string | — | Selon le mode, ci-dessous |
| `port` | number | `0` | Le port, pour `sweep` uniquement |
| `payload` | [`Payload`](#type-payload) | — | Ce que porte chaque datagramme |
| `bind` | string | any | L’`IP:port` local depuis lequel il est envoyé ; vide ou null : `0.0.0.0:0` (`[::]:0` quand toutes les cibles sont en IPv6) |
| `ttl` | number | `1` | TTL IP, ou la limite de sauts multicast ; de 1 à 255 |
| `multicast_loop` | boolean | `true` | Le multicast revient aussi à cette machine |
| `rate`, `count`, `duration_s` | number | `0` | Pour [`broadcast_beacon_start`](#broadcast_beacon_start) uniquement |

| `mode` | `target` |
| --- | --- |
| `list` | Des entrées `IP:port` ou `host:port` séparées par des virgules, des points-virgules ou des retours à la ligne (pas par des espaces) ; un nom est résolu, son adresse IPv4 prise quand il en a une |
| `broadcast` | `255.255.255.255:port`, ou une adresse se terminant par `.255` avec son port |
| `multicast` | Un groupe de 224.0.0.0 à 239.255.255.255 avec son port |
| `sweep` | Un bloc CIDR, `192.0.2.0/24` : chaque hôte utilisable sur `port` ; au plus 1024 hôtes, donc `/22` ou plus étroit |

**Résultat**

| Champ | Type | Signification |
| --- | --- | --- |
| `targets` | number | Destinations |
| `packets`, `bytes` | number | Ce qui est parti |
| `errors` | number | Les datagrammes qui n’ont pas pu être envoyés |
| `resolved` | string[] | Les 8 premières destinations |
| `summary` | string | La charge utile en une ligne |
| `error` | `EngineError` | Pourquoi le premier datagramme échoué a échoué ; omis quand aucun n’a échoué |

**Erreurs** : `broadcast.target_required`, `broadcast.not_broadcast`,
`broadcast.ipv6`, `broadcast.not_multicast`, `broadcast.sweep_port`,
`broadcast.cidr_invalid`, `broadcast.prefix_invalid`,
`broadcast.sweep_too_large`, `node.osc_address` (une adresse OSC doit commencer
par `/`), `hex.empty`, `hex.invalid`, `node.bind_invalid`,
`socket.option_failed`, `transport.target_invalid`, `transport.dns`, les échecs de
liaison.

### broadcast_beacon_start {#broadcast_beacon_start}

Envoie le même tour — un datagramme par cible — encore et encore. Ses compteurs
arrivent comme [`broadcast://emit-stat`](events.md#event-broadcast-emit-stat) toutes les
250 ms. Après plus de 32 envois échoués sans qu’aucun n’ait abouti, il s’arrête avec
la raison. *Démarre une tâche* (`beacon`, `params.mode`, `params.target`,
`params.targets`, `params.rate`).

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `config` | object | oui | Comme pour [`broadcast_send`](#broadcast_send), avec les trois ci-dessous |

| Champ de `config` | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `rate` | number | — | Tours par seconde ; supérieur à 0, et tours × cibles au plus 50 000 datagrammes par seconde |
| `count` | number | `0` | S’arrêter après ce nombre de tours ; 0 : sans limite |
| `duration_s` | number | `0` | S’arrêter après ce nombre de secondes ; 0 : jusqu’à l’arrêt |

**Résultat** : [`JobInfo`](#type-jobinfo).

**Erreurs** : celles de [`broadcast_send`](#broadcast_send),
`broadcast.rate_invalid`, `broadcast.rate_limit`.

### discovery_start {#discovery_start}

Écoute sur un port UDP, tient une liste de chaque pair qui envoie quelque chose, et
peut répondre aux sondes comme le ferait un appareil. Les pairs arrivent comme
[`broadcast://peers`](events.md#event-broadcast-peers) toutes les 400 ms. *Démarre
une tâche* (`discovery`, `params.bind`, `params.groups`, `params.joined`).

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `config` | object | oui | Ci-dessous |

| Champ de `config` | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `bind` | string | — | `IP:port` sur lequel écouter |
| `groups` | string[] | `[]` | Les groupes multicast à rejoindre (IPv4) |
| `interface` | string | any | L’adresse IPv4 locale sur laquelle rejoindre les groupes |
| `reuse` | boolean | `true` | Partager le port avec un programme qui l’écoute déjà (`SO_REUSEADDR`) |
| `respond` | boolean | `false` | Répondre à ce qui arrive |
| `response` | [`Payload`](#type-payload) | none | La réponse ; nécessaire avec `respond` |
| `respond_delay_ms` | number | `0` | Attendre ce temps avant de répondre |
| `match_contains` | string | none | Ne répondre qu’aux datagrammes dont le texte contient ceci |

Au plus 512 pairs sont listés ; les suivants ne sont pas ajoutés.

**Résultat** : [`JobInfo`](#type-jobinfo).

**Erreurs** : `node.bind_invalid`, `broadcast.port_shared` (le port est pris
et `reuse` est désactivé), `broadcast.interface_invalid`,
`broadcast.not_multicast`, `broadcast.join_failed`,
`broadcast.reply_missing`, `node.osc_address`, `hex.*`, les échecs de liaison. La
tâche se termine avec `wait.receive_failed` si le socket ne peut plus recevoir.

```bash
invoke broadcast_send '{"config":{"mode":"list","target":"127.0.0.1:9000, 127.0.0.1:9001","payload":{"kind":"text","text":"PING"}}}' \
  | jq '{packets, errors}'
```

## Impairment {#impairment}

Un relais entre un client et son serveur qui retarde, abandonne, duplique,
corrompt, réordonne ou bride ce qui passe, sur UDP ou TCP. Voir
[dégradation](../tools/impairment.md).

### netsim_start {#netsim_start}

Démarre un relais : ce qui arrive sur `listen` passe à `target`, et les réponses
reviennent de la même façon, les deux directions étant dégradées par le profil. Ses compteurs arrivent comme
[`netsim://stat`](events.md#event-netsim-stat) toutes les 250 ms. *Démarre une tâche*
(`netsim`, `params.listen`, `params.target`, et `params.protocol` pour TCP).

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `config` | object | oui | Ci-dessous |

| Champ de `config` | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `listen` | string | — | L’`IP:port` sur lequel le relais écoute ; pointez le client ici |
| `target` | string | — | L’`IP:port` du vrai serveur, ou `host:port` — un nom d’hôte est résolu une fois, au démarrage du relais |
| `profile` | [`ImpairProfile`](#type-impairprofile) | — | Ce qu’il faut faire au trafic |
| `seed` | number | new | La graine des tirages : la même graine et le même trafic donnent les mêmes abandons |
| `protocol` | string | `udp` | `udp` (datagrammes) ou `tcp` (flux) |

**Résultat** : [`JobInfo`](#type-jobinfo).

**Erreurs** : `node.range` (une valeur du profil hors de sa plage, avec
`min`, `max` et le champ), `node.too_long`, `node.bind_invalid`,
`transport.target_invalid`, `transport.dns` (un nom de cible introuvable), les
échecs de liaison. La tâche se termine avec
`wait.receive_failed` si un socket ne peut plus recevoir.

### netsim_set_profile {#netsim_set_profile}

Un relais en cours dégrade désormais avec un autre profil, sans fermer ses
sockets.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `jobId` | number | oui | La tâche du relais |
| `profile` | [`ImpairProfile`](#type-impairprofile) | oui | Le nouveau profil |

**Résultat** : null.

**Erreurs** : `netsim.not_running`, `node.range`, `node.too_long`.

```bash
invoke netsim_start '{"config":{"listen":"127.0.0.1:9010","target":"127.0.0.1:9000","profile":{"latency_ms":80,"jitter_ms":20,"loss":0.02}}}'
```

## Storm and scanner {#storm-scanner}

::: danger
Une tempête charge une cible aussi fort que vous le demandez, et un scan sonde chaque port d’une
plage. Ne les dirigez que vers des hôtes dont vous êtes responsable.
:::

### storm_start {#storm_start}

Envoie une charge régulière de datagrammes UDP ou de connexions TCP vers une cible. Ses
compteurs arrivent comme [`storm://stat`](events.md#event-storm-stat) toutes les 250 ms.
*Démarre une tâche* (`storm`, `params.protocol`, `params.target`, `params.rate`).

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `config` | object | oui | Ci-dessous |

| Champ de `config` | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `target` | string | — | `IP:port` ou `host:port` ; un nom est résolu une fois, au démarrage de la tâche |
| `protocol` | string | — | `udp` : des datagrammes ; `tcp` : une connexion par unité qui écrit la charge utile et se ferme (chaque connexion peut prendre 500 ms) |
| `size` | number | — | Des octets de charge utile, maintenus entre 1 et 65 507 |
| `rate` | number | — | Des unités par seconde, selon un calendrier : l’unité *n* est due *n* / `rate` secondes après le début, et chaque réveil envoie ce qui est dû (256 au plus ; un calendrier plus en retard saute les unités les plus anciennes) ; 0 envoie aussi vite qu’il peut |
| `duration_s` | number | `0` | S’arrêter après ce nombre de secondes ; 0 : jusqu’à l’arrêt |

**Résultat** : [`JobInfo`](#type-jobinfo).

**Erreurs** : `transport.target_invalid`, `transport.dns`. Les envois échoués sont
comptés dans les événements, pas rapportés comme des erreurs.

### scan_start {#scan_start}

Tente une connexion TCP vers chaque port d’une plage et rapporte ceux qui sont ouverts,
avec ce que dit le service en premier quand on lui demande. Les ports ouverts arrivent comme
[`scan://open`](events.md#event-scan-open), la progression comme
[`scan://progress`](events.md#event-scan-progress). *Démarre une tâche* (`scan`,
`params.host`, `params.from`, `params.to`).

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `config` | object | oui | Ci-dessous |

| Champ de `config` | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `host` | string | — | Un nom d’hôte ou une adresse |
| `port_start`, `port_end` | number | — | La plage, bornes comprises ; données dans le mauvais ordre, elles sont échangées |
| `concurrency` | number | `256` | Des tentatives à la fois, de 1 à 1024 |
| `timeout_ms` | number | `600` | Par port, de 50 à 10 000 |
| `grab_banner` | boolean | `false` | Lire jusqu’à 256 octets que le service envoie dans les 400 ms après la connexion |

**Résultat** : [`JobInfo`](#type-jobinfo).

**Erreurs** : `scan.host_required`.

```bash
invoke scan_start '{"config":{"host":"127.0.0.1","port_start":8000,"port_end":9100,"grab_banner":true}}'
```

## Inspector {#inspector}

L’Inspecteur enregistre ce que les outils envoient et reçoivent, sous forme de trames, tant que
la capture est armée. Sur un serveur, il y a un Inspecteur pour chaque page et
chaque script. Voir [l’Inspecteur](../tools/inspector.md).

### inspect_set_enabled {#inspect_set_enabled}

Arme ou désarme la capture. Tant qu’elle est désarmée, rien n’est enregistré.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `enabled` | boolean | oui | Armer (`true`) ou désarmer |

**Résultat** : [`CaptureStats`](#type-frame).

### inspect_stats {#inspect_stats}

Les compteurs de la capture. Aucun argument.

**Résultat** : [`CaptureStats`](#type-frame).

### inspect_snapshot {#inspect_snapshot}

Les trames les plus récentes, la plus ancienne d’abord.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `limit` | number | oui | Combien, de 1 à 8192 |

**Résultat** : [`Frame`](#type-frame)`[]`, sans leurs octets (voir
[`inspect_payload`](#inspect_payload)).

### inspect_clear {#inspect_clear}

Vide la capture et ses compteurs. Aucun argument.

**Résultat** : [`CaptureStats`](#type-frame).

### inspect_export {#inspect_export}

Écrit chaque trame conservée dans `capture-<ms>.jsonl` ou `capture-<ms>.txt` dans le
dossier de données. En `jsonl`, chaque ligne est une trame avec les octets qu’elle conserve dans
`data`, en base64 ; `txt` est fait pour la lecture, avec un vidage hexadécimal de chaque trame.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `format` | string | oui | `txt` ; toute autre valeur écrit `jsonl` |

**Résultat** : une chaîne, le chemin écrit.

**Erreurs** : `inspect.empty`, `file.io`.

### inspect_payload {#inspect_payload}

Les octets qu’une trame conserve, au-delà de l’aperçu de 1 Kio que portait son lot.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `seq` | number | oui | Le numéro de la trame |

**Résultat** : `{ "seq", "bytes", "kept", "dump", "hex" }` — `bytes` la taille de la
trame, `kept` combien d’entre eux sont conservés (jusqu’à 256 Kio), `dump` chaque ligne sous la forme
`offset  hex  |ascii|`, `hex` l’hexadécimal brut qu’envoie un rejeu.

**Erreurs** : `inspect.frame_gone` (des trames plus récentes ont pris sa place),
`inspect.no_payload` (seule sa taille a été enregistrée).

```bash
invoke inspect_set_enabled '{"enabled":true}'
invoke inspect_snapshot '{"limit":20}' | jq '.[] | {seq, proto, dir, summary}'
```

## Signal library {#signals}

La bibliothèque est `signals.json` dans le dossier de données. Ce n’est que du stockage : un signal
est envoyé avec la commande de son transport (`osc_send`, `broadcast_send`,
`http_request`, `mqtt_publish` ou `mqtt_publish_once`). Voir
[signaux](../tools/signals.md) et [fichiers](../reference/files.md#signals-json).

### signals_load {#signals_load}

Lit la bibliothèque. Quand le fichier n’existe pas, le jeu de départ est d’abord
écrit. Aucun argument.

**Résultat** : `{ "path": string, "library": library, "seeded": boolean }` —
`seeded` est vrai quand le jeu de départ vient d’être écrit. La bibliothèque est
`{ "version", "signals": [...], "folders": [...] }` : `version` 2 (un fichier de version 1
revient tel quel), `folders` omis quand il n’y en a aucun. Chaque signal
a `id`, `name`, `group` (son dossier, `"A/B"` ; vide pour aucun), `note` et
`body`.

**Erreurs** : `signals.json_invalid` (avec `path`, `line`, `column` ; le fichier
n’est jamais remplacé), `file.io`.

### signals_save {#signals_save}

Remplace tout le fichier de bibliothèque, via un fichier temporaire dans le même dossier.
Un fichier qui existe et ne se lit pas comme une bibliothèque est laissé tel quel.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `library` | object | oui | `{ version, signals, folders }` tel que le renvoie `signals_load` |

**Résultat** : une chaîne, le chemin écrit.

**Erreurs** : `signals.json_invalid` (le fichier maintenant sur le disque ne se lit pas, avec
`path`, `line`, `column` ; rien n’est écrit), `signals.encode`, `file.io`.

Le `body` d’un signal selon son `transport` :

| `transport` | Champs |
| --- | --- |
| `osc` | `target`, `address`, `args` ([`OscArg`](#type-oscarg)`[]`) |
| `udp` | `target`, `payload` : `{ "kind": "text", "text" }` ou `{ "kind": "hex", "hex" }` |
| `http` | `request` ([`HttpRequest`](#type-httprequest)) |
| `mqtt` | `broker` (`host:port`), `topic`, `payload`, `qos`, `retain` |

```bash
invoke signals_load | jq '.library.signals[] | {name, transport: .body.transport}'
```

## Emulators {#emulators}

Un émulateur, c’est Signal Lab qui joue l’autre côté : une API HTTP, un appareil
OSC, UDP ou TCP, un broker MQTT. Son document — `name`, `bind`, `protocol`, les
règles du protocole et une `outage` facultative — est décrit dans
[émulateurs](../tools/emulators.md). La bibliothèque est `emulators.json` dans le
dossier de données.

### emulators_load {#emulators_load}

Lit la bibliothèque d’émulateurs. Quand le fichier n’existe pas, le jeu de départ est
d’abord écrit. Aucun argument.

**Résultat** : `{ "path", "library": { "version": 1, "emulators": [{ "id", "note", "emulator" }] }, "seeded" }`.

**Erreurs** : `emulators.json_invalid` (avec `path`, `line`, `column` ; jamais
remplacé), `file.io`.

### emulators_save {#emulators_save}

Remplace toute la bibliothèque d’émulateurs, via un fichier temporaire.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `library` | object | oui | `{ version, emulators }` tel que le renvoie `emulators_load` |

**Résultat** : une chaîne, le chemin écrit.

**Erreurs** : `emulators.encode`, `file.io`.

### emulator_check {#emulator_check}

Si un émulateur démarrerait : tout ce que [`emulator_start`](#emulator_start)
vérifie avant de se lier.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `emulator` | object | oui | Le document de l’émulateur |
| `params` | object of strings | non | Les valeurs que ses modèles lisent comme paramètres |

**Résultat** : null quand il démarrerait.

**Erreurs** : `emulator.*` ; `node.*` pour un champ manquant, hors plage, trop long
ou malformé (`node.required`, `node.range`, `node.too_long`,
`node.bind_invalid`, `node.target_invalid`, `node.method_invalid`…) ;
`param.unknown`, `template.*`, `osc.pattern_*`, `regex.invalid`,
`hex.invalid`. Chacune a `rule`, `retained` ou `response` dans `params` quand le
problème se trouve dans l’un d’eux.

### emulator_start {#emulator_start}

Démarre un émulateur comme une tâche à part. Son socket est ouvert quand la commande
revient. Ce qu’il reçoit et ce à quoi il répond arrive comme
[`emulator://activity`](events.md#event-emulator-activity) toutes les 200 ms quand
quelque chose a changé. *Démarre une tâche* (`emulator`, `params.name`,
`params.protocol`, `params.local`, et `params.source` quand `source` est
donné).

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `emulator` | object | oui | Le document de l’émulateur |
| `params` | object of strings | non | Les valeurs que ses modèles lisent comme paramètres |
| `seed` | number | non | Sa graine, de 0 à 9007199254740991 ; par défaut : une nouvelle |
| `source` | string | non | L’entrée de bibliothèque dont il vient, conservée sur la tâche sous `params.source` |

**Résultat** : [`JobInfo`](#type-jobinfo).

**Erreurs** : celles de [`emulator_check`](#emulator_check), `seed.range`,
`transport.address_in_use` et les autres échecs de liaison.

### emulator_exchanges {#emulator_exchanges}

Ce qu’un émulateur en cours a reçu et ce à quoi il a répondu. Il conserve les 500
derniers échanges.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `jobId` | number | oui | La tâche de l’émulateur |
| `after` | number | non | Seulement les échanges numérotés au-dessus ; par défaut 0 |
| `limit` | number | non | Au plus ce nombre, de 1 à 500 ; par défaut 500 |

**Résultat**

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche |
| `name`, `protocol`, `local` | string | L’émulateur, son protocole et l’adresse sur laquelle il écoute |
| `counts` | object | `total`, `unmatched` (aucune règle ne l’a pris), `failed`, `down` (arrivé pendant qu’il était en panne : sa panne programmée ou [`emulator_down`](#emulator_down)), `hits` (par règle), et `missed` (MQTT : les messages qu’un client trop en retard n’a pas reçus ; omis tant qu’il vaut 0) |
| `forced` | string | `unavailable`, `reset` ou `timeout` pendant qu’il est mis en panne ; omis sinon |
| `exchanges` | object[] | Chacun : `seq`, `ts`, `from`, `request`, `rule` (à partir de 1 ; omis quand aucune ne l’a pris), `reply`, `status`, `fault`, `ms`, `error`, `frame`, `down`, et `data` (la requête telle que la lisent les modèles) |

**Erreurs** : `emulator.not_running`.

### emulator_down {#emulator_down}

Met un émulateur en cours en panne jusqu’à ce qu’il soit remis en service, quoi que dise son calendrier
de pannes, ou le remet en service. Pendant la panne, un émulateur HTTP accueille chaque
requête avec `fault`, un appareil TCP et un broker MQTT coupent leurs connexions
et refusent les nouvelles, et les appareils OSC et UDP ne répondent rien.

| Argument | Type | Requis | Signification |
| --- | --- | --- | --- |
| `jobId` | number | oui | La tâche de l’émulateur |
| `down` | boolean | oui | En panne (`true`) ou en service |
| `fault` | string | non | Ce que rencontrent les requêtes HTTP : `unavailable` (503, sans `Retry-After` : on ne sait pas quand il revient), `reset` (la connexion se ferme), `timeout` (aucune réponse) ; par défaut `unavailable` |

**Résultat** : null.

**Erreurs** : `emulator.not_running`.

```bash
invoke emulator_exchanges '{"jobId":5,"after":0}' | jq '.counts, (.exchanges[] | {request, rule, status})'
```

## Shared types {#types}

### JobInfo {#type-jobinfo}

Ce que renvoie une commande qui démarre une tâche, et ce que
[`jobs_list`](#jobs_list) liste : `id`, `kind`, `label` (en anglais, pour les journaux), `params` (les
valeurs que nomme le label ; omis quand il n’y en a pas) et `started_ms`. Voir
[les tâches](index.md#jobs).

### OscArg {#type-oscarg}

Un argument OSC, son type et sa valeur :

| `type` | `value` | Étiquette OSC |
| --- | --- | --- |
| `int` | entier 32 bits | `i` |
| `float` | nombre, envoyé en float 32 bits | `f` |
| `str` | chaîne | `s` |
| `long` | entier 64 bits | `h` |
| `double` | nombre, 64 bits | `d` |
| `bool` | `true` ou `false` | `T` ou `F` |
| `blob` | tableau d’octets, `[222, 173]` | `b` |
| `nil` | aucun : `{ "type": "nil" }` | `N` |

### HttpRequest {#type-httprequest}

| Champ | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `method` | string | — | `GET`, `POST`… |
| `url` | string | — | `http://` ou `https://` |
| `headers` | `[name, value][]` | `[]` | En-têtes de la requête |
| `body` | string or null | null | Le corps |
| `timeout_ms` | number | `10000` | Pour tout l’échange |
| `auth` | object | none | `{ "scheme": "basic", "username", "password" }`, `{ "scheme": "digest", "username", "password" }` ou `{ "scheme": "bearer", "token" }` |

Jusqu’à 10 redirections sont suivies. Les identifiants et les cookies saisis pour un hôte
ne passent jamais à un autre. Une requête Digest répond au challenge 401 du serveur
et renvoie.

### HttpResponse {#type-httpresponse}

| Champ | Type | Signification |
| --- | --- | --- |
| `ok` | boolean | Un statut 2xx |
| `status`, `status_text` | number, string | Le statut ; 0 et vide sans réponse |
| `latency_ms` | number | Jusqu’à l’arrivée du corps entier |
| `headers` | `[name, value][]` | En-têtes de la réponse |
| `body` | string | Le corps en texte, 256 Kio au plus |
| `body_bytes` | number | La taille complète du corps |
| `truncated` | boolean | `body` a été coupé à 256 Kio |
| `error` | string or null | Pourquoi il n’y a pas eu de réponse, chaque couche de la cause |
| `cause` | string or null | Le type d’échec : `refused`, `timeout`, `dns`, `unreachable`, `reset`, `address_in_use`, `address_unavailable`, `denied`, `tls`, `target_invalid`, `failed` — les mêmes que les codes `transport.*` |
| `digest` | object | Une requête Digest qui a rencontré un 401 uniquement ; omis sinon. `challenged` : le challenge a été répondu et la requête renvoyée. `error` : pourquoi elle n’a pas pu l’être, un `EngineError` (`http.digest_not_offered`, `http.digest_unsupported`, `http.digest_invalid`, `http.digest_other_origin`) ou null |

### Payload {#type-payload}

Ce que porte un datagramme de diffusion ou de découverte :
`{ "kind": "osc", "address", "args" }`, `{ "kind": "text", "text" }` (envoyé tel
quel, sans zéro terminal) ou `{ "kind": "hex", "hex" }` (`de ad be ef`,
`deadbeef`, `0xDE,0xAD` — tout ce qui n’est pas un chiffre hexadécimal est ignoré).

### MqttConfig {#type-mqttconfig}

| Champ | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `host` | string | — | Le nom ou l’adresse du broker |
| `port` | number | — | Habituellement 1883 |
| `client_id` | string | — | Non vide ; une autre connexion avec le même id est délogée par le broker |
| `username`, `password` | string | vide | `username` est envoyé quand il n’est pas vide ; `password` seulement avec un `username` |
| `keep_alive_s` | number | `60` | Les pings partent à la moitié ; 0 : aucun |
| `clean_session` | boolean | `true` | L’indicateur de CONNECT |
| `will` | object or null | null | `{ topic, payload, qos, retain }`, publié par le broker si la connexion est perdue |
| `subscribe` | `{ filter, qos }[]` | `[]` | Abonné dès que la connexion est en place |

### WsConfig {#type-wsconfig}

| Champ | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `url` | string | — | `ws://` ou `wss://` (`wss://` approuve ce que le système approuve pour HTTPS) |
| `headers` | `[name, value][]` | `[]` | Envoyés avec la requête de mise à niveau |
| `protocols` | string[] | `[]` | Les sous-protocoles à proposer, par ordre de préférence |
| `timeout_ms` | number | `10000` | Pour la connexion, TLS et la mise à niveau ensemble |

Les messages font au plus 16 Mio dans les deux sens.

### ImpairProfile {#type-impairprofile}

Chaque champ est facultatif ; ce qui est omis ne fait rien. Les probabilités vont de 0
à 1.

| Champ | Plage | Signification | UDP | TCP |
| --- | --- | --- | --- | --- |
| `name` | 60 caractères au plus | Un libellé pour la chronologie et le rapport | oui | oui |
| `latency_ms` | de 0 à 60 000 | Un retard ajouté à tout | oui | oui |
| `jitter_ms` | de 0 à 60 000 | Jusqu’à autant de plus, tiré à chaque fois | oui | oui |
| `loss` | de 0 à 1 | Un datagramme est abandonné | oui | — |
| `duplicate` | de 0 à 1 | Un datagramme est envoyé deux fois | oui | — |
| `corrupt` | de 0 à 1 | Un bit d’un datagramme est inversé | oui | — |
| `reorder` | de 0 à 1 | Un datagramme est retenu pour que les suivants le dépassent | oui | — |
| `rate_kbps` | 0, ou de 8 à 10 000 000 | Limite de bande passante, en kilobits par seconde ; 0 : aucune | oui | oui |
| `burst_start` | de 0 à 1 | Un datagramme démarre une rafale de pertes | oui | — |
| `burst_length` | de 1 à 1000 | Datagrammes que dure une rafale en moyenne (nécessaire avec `burst_start`) | oui | — |
| `offline` | `true` ou `false` | Rien ne passe | oui | oui |
| `reset` | de 0 à 1 | Un morceau d’un flux réinitialise sa connexion | — | oui |
| `stall` | de 0 à 1 | Un morceau d’un flux laisse sa connexion semi-ouverte | — | oui |

### Frame and CaptureStats {#type-frame}

Une `Frame` est un paquet, une requête ou un message capturé :

| Champ | Signification |
| --- | --- |
| `seq` | Son numéro, croissant |
| `ts` | Quand, en millisecondes depuis 1970 |
| `proto` | `osc`, `udp`, `tcp`, `http`, `mqtt`, `ws`… |
| `dir` | `tx` (envoyé) ou `rx` (reçu) |
| `source` | L’outil qui l’a capturé : `osc-monitor`, `broadcast`, `netsim`… |
| `job_id` | Sa tâche, ou null |
| `local`, `remote` | Les adresses : `IP:port` de ce côté-ci et de l’autre (une URL ou un broker pour HTTP, WebSocket et MQTT). Le `local` d’une trame relayée est l’adresse sur laquelle le relais écoute et son `remote` là où la trame allait ; le sens se lit dans son `verdict` (`· client→target`, `· target→client`) |
| `bytes` | Sa taille |
| `summary` | Une ligne |
| `detail` | Un décodage sur plusieurs lignes, ou null |
| `hex` | Un vidage hexadécimal du premier Kio, ou null |
| `verdict` | Ce qu’il est devenu — `dropped`, `sampled`, un statut — ou null |
| `kept` | Sur `bytes`, combien sont conservés (jusqu’à 256 Kio) ; 0 quand seule la taille a été enregistrée |
| `publish` | Une publication MQTT uniquement : `{ broker, topic, qos, retain, text }` — le broker en `host:port`, et si les octets conservés, qui sont la charge utile du message, sont du texte UTF-8. Absent pour toute autre trame |

`CaptureStats` : `enabled`, `total` (trames enregistrées), `bytes`, `skipped`
(enregistrées mais jamais envoyées à l’interface), `buffered` (trames retenues),
`capacity` (8192), `held` (octets de charge utile retenus) et `held_limit` (64 Mio). Les
trames les plus anciennes cèdent la place au-delà de l’une ou l’autre limite.

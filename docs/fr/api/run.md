---
title: "Exécutions"
description: "POST /api/run exécute une expérience sur un serveur Signal Lab et répond avec son résultat, ou diffuse ses étapes sous forme de lignes NDJSON."
---

# Exécuter une expérience

Pour exécuter une expérience sur un serveur depuis un script ou un pipeline et
savoir comment elle s’est passée, envoyez-la à `POST /api/run`. Le serveur
l’exécute jusqu’au bout et répond avec le résultat — ou, si vous le demandez,
avec chaque étape au fur et à mesure. C’est ce qu’utilise
[`signallab run --server`](../automation/cli.md#cli-run).

C’est la même exécution que le [[ui:exp.run]] de l’éditeur et
[`experiment_start`](commands.md#experiment_start) : une tâche que chaque page
ouverte voit et peut arrêter, les mêmes événements, le même rapport dans le
dossier de données.

## La demande {#request}

```http
POST /api/run
Authorization: Bearer <token>
Content-Type: application/json

{ "template": "osc-ping-reply", "overrides": { "device": "192.0.2.20:9000" }, "seed": 42, "timeout": 30 }
```

Le corps nomme une expérience — un `document` ou un `template`, pas les deux —
et avec quoi l’exécuter :

| Champ | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `document` | object | — | Une expérience telle que l’éditeur l’enregistre et l’exporte. Les versions plus anciennes sont migrées, comme à l’ouverture d’un fichier |
| `template` | string | — | Un modèle fourni, par nom de fichier, avec ou sans `.json` (plus bas) |
| `overrides` | object | `{}` | Les valeurs de paramètres pour cette exécution seulement. Les valeurs peuvent être du texte, des nombres ou des booléens ; chacune doit être un paramètre de l’expérience |
| `profile` | string | celui du document | Exécuter avec ce profil ; `""` exécute avec les valeurs par défaut |
| `seed` | number | celui du document, sinon une nouvelle | 0 à 9007199254740991 ; la même graine tire les mêmes valeurs aléatoires |
| `timeout` | number | `300` | Secondes avant que l’exécution échoue avec `run.timeout` ; 1 à 300 |

Un champ que le serveur ne connaît pas est refusé (`400`, `api.run_invalid`). Le
document lui-même est lu comme l’est un fichier ouvert, donc il fait au plus
4 Mio.

Les modèles fournis — les expériences que l’éditeur propose sous
[[ui:exp.templates]] :

| `template` | Ce que c’est |
| --- | --- |
| `empty` | Début et Fin |
| `http-check` | Un GET de `http://127.0.0.1:8080/`, puis une vérification du statut 200 |
| `status-branch` | Un GET de `http://127.0.0.1:8080/` ; sur 200 un message OSC, sinon un délai de 500 ms |
| `parallel-flows` | Un [[ui:exp.node.fork]] vers un GET de `http://127.0.0.1:8080/` et une entrée de journal, côte à côte, puis [[ui:exp.node.join]] |
| `osc-ping-reply` | Un `/ping` OSC vers le paramètre `device` (`127.0.0.1:9000`), puis une attente de 2 s de `/pong` sur `127.0.0.1:9001` |
| `poll-until-ready` | Une [[ui:exp.node.loop]] qui demande `/status` à `device` en OSC jusqu’à ce qu’il réponde `ready`, au plus 10 fois |
| `flaky-api` | Une API émulée (paramètre `api`) qui échoue avant de fonctionner, interrogée dans une [[ui:exp.node.loop]] jusqu’à ce qu’elle réponde 200 |
| `fault-phases` | Un appareil UDP derrière un relais de dégradation, auquel on envoie pendant 8 s pendant que le relais devient propre, avec pertes, hors ligne et de nouveau propre |
| `dependency-outage` | Une API émulée (paramètre `api`) mise en panne pendant 2 s pendant qu’une [[ui:exp.node.loop]] l’interroge jusqu’à ce qu’elle réponde de nouveau 200 |
| `websocket-echo` | Une connexion au paramètre `service` (`ws://127.0.0.1:9001/echo`), un message, une attente de son écho, une vérification, une fermeture |

Ouvrez-en un sous [[ui:exp.templates]] pour voir ses nœuds et paramètres ; voir
[expériences](../experiments/index.md).

## Le résultat {#result}

Par défaut la réponse est `200` avec `Content-Type: application/json`, envoyée
quand l’exécution est terminée : un objet JSON, le résultat de l’exécution.

```json
{
  "job_id": 12,
  "experiment": "OSC ping → reply",
  "outcome": "passed",
  "seed": 42,
  "profile": null,
  "overridden": true,
  "params": { "device": "192.0.2.20:9000" },
  "started_ms": 1759600000000,
  "ended_ms": 1759600000310,
  "steps": [ { "job_id": 12, "ts": 1759600000001, "node_id": "start", "state": "running", "detail": "", "message_key": null, "message_params": null }, … ],
  "report_path": "/data/runs/run-1759600000000-12.json"
}
```

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche de l’exécution |
| `experiment` | string | Le nom de l’expérience |
| `outcome` | string | `passed`, `failed` ou `stopped` |
| `seed` | number | La graine avec laquelle elle a tourné : rendez-la comme `seed` pour tirer les mêmes valeurs |
| `profile` | string ou null | Le profil avec lequel elle a tourné |
| `overridden` | boolean | Certaines valeurs venaient d’`overrides` |
| `params` | object | Chaque valeur de paramètre utilisée par l’exécution |
| `started_ms`, `ended_ms` | number | Millisecondes depuis 1970 |
| `error` | `EngineError` | Pourquoi elle a échoué : son premier échec. Omis quand elle a réussi |
| `steps` | object[] | Chaque étape dans l’ordre où elle s’est produite, comme [`experiment://step`](events.md#event-experiment-step) |
| `emulators` | object[] | Ce que chaque nœud [[ui:exp.node.emulator]] a reçu et répondu : `node`, `name`, `protocol`, `local`, `counts`. Omis quand il n’y en a aucun |
| `impairments` | object[] | Ce qu’a fait le relais de chaque nœud [[ui:exp.node.impairment]], phase par phase. Omis quand il n’y en a aucun |
| `report_path` | string | Le rapport de l’exécution sur le serveur ; téléchargez-le avec [`/api/files`](index.md#files). Omis quand aucun n’a été écrit |
| `report_error` | `EngineError` | Pourquoi le rapport n’a pas pu être écrit. Omis sinon |

Les valeurs de secrets sont masquées dans tout cela. Le fichier de rapport
contient les mêmes étapes ; voir [exécutions et rapports](../experiments/runs.md).

Pendant que l’exécution se déroule, le serveur envoie un espace toutes les 15 s.
JSON ignore les espaces avant une valeur, le résultat s’analyse donc toujours, et
un proxy ne prend pas une exécution longue et silencieuse pour une connexion
morte.

## Suivre les étapes {#lines}

Pour voir les étapes au fur et à mesure, demandez du NDJSON :

```http
Accept: application/x-ndjson
```

La réponse est `200` avec `Content-Type: application/x-ndjson` : un objet JSON
par ligne, chacun avec un `type`.

| `type` | Quand | Le reste de la ligne |
| --- | --- | --- |
| `started` | En premier, une fois | `job_id`, `experiment`, `seed`, `profile`, `overridden`, `started_ms` |
| `step` | Chaque étape | L’étape, comme [`experiment://step`](events.md#event-experiment-step) |
| `heartbeat` | Toutes les 15 s | Rien |
| `ended` | En dernier, une fois | Le résultat, comme plus haut |

```text
{"job_id":12,"experiment":"OSC ping → reply","seed":42,"profile":null,"overridden":true,"started_ms":1759600000000,"type":"started"}
{"job_id":12,"ts":1759600000001,"node_id":"start","state":"running","detail":"","message_key":null,"message_params":null,"type":"step"}
…
{"job_id":12,"experiment":"OSC ping → reply","outcome":"passed",…,"type":"ended"}
```

Lisez les lignes jusqu’à `ended` ; ignorez un `type` que vous ne connaissez pas.
Le serveur envoie `X-Accel-Buffering: no`, pour qu’un proxy nginx transmette
chaque ligne aussitôt.

## Statut et résultat {#status}

Un statut d’erreur HTTP signifie qu’**aucune exécution n’a démarré** ; le corps
est un [`EngineError`](index.md#errors) :

| Statut | Code | Pourquoi |
| --- | --- | --- |
| `400` | `api.run_invalid` | Le corps n’est pas une demande d’exécution : pas du JSON, un champ inconnu, un override qui n’est pas du texte, un nombre ou un booléen |
| `400` | `api.run_source` | Ni `document` ni `template`, ou les deux |
| `415` | `command.json_required` | Pas `Content-Type: application/json` |
| `422` | `api.template_unknown` | Aucun modèle fourni de ce nom |
| `422` | `file.json_invalid`, `file.too_large`, `doc.*` | Le document ne peut pas être lu |
| `422` | n’importe quel code de validation, `run.override_unknown`, `profile.active_missing`, `run.limit_range`, `seed.range`, `secret.missing`, `transport.address_in_use`… | L’expérience ne peut pas démarrer : elle ne valide pas, une valeur est hors limites, un secret n’est pas stocké, un port qu’elle écoute est pris |

Une fois l’exécution démarrée, le statut est `200`, quoi qu’il arrive : lisez
`outcome` dans le résultat.

| `outcome` | Signification |
| --- | --- |
| `passed` | Chaque étape a réussi et [[ui:exp.node.end]] a été atteint |
| `failed` | Une étape a échoué, ou l’exécution a duré plus que son `timeout` (`run.timeout`) ; `error` dit laquelle et pourquoi |
| `stopped` | Elle a été arrêtée avant de se terminer : par `job_stop`, [[ui:app.stopAll]], ou l’arrêt du serveur. `steps` contient les étapes qu’elle a atteintes ; aucun rapport n’est enregistré |

## Un client qui s’en va {#disconnect}

Fermer la connexion n’arrête pas l’exécution. C’est une tâche sur le serveur :
elle tourne jusqu’au bout et enregistre son rapport, comme le fait une exécution
démarrée dans un navigateur quand l’onglet est fermé. Retrouvez-la avec
[`jobs_list`](commands.md#jobs_list), arrêtez-la avec
[`job_stop`](commands.md#job_stop), et lisez son rapport ensuite avec
[`experiment_runs`](commands.md#experiment_runs). Quand le serveur s’arrête,
l’exécution est arrêtée et un client encore connecté reçoit
`"outcome": "stopped"`.

## Exemples {#examples}

Exécutez un modèle fourni et attendez le résultat :

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
curl -sS -X POST "$SERVER/api/run" \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"template":"osc-ping-reply","overrides":{"device":"192.0.2.20:9000"},"timeout":30}' \
  | jq -r .outcome
```

Envoyez votre propre expérience avec un paramètre modifié, et affichez chaque
étape au fur et à mesure :

```bash
jq '{document: ., overrides: {api: "http://192.0.2.10:8080"}, profile: ""}' smoke.json |
  curl -sSN -X POST "$SERVER/api/run" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    -H "Accept: application/x-ndjson" --data @- |
  jq -r 'select(.type == "step") | "\(.node_id)  \(.state)  \(.detail)"'
```

`-N` empêche `curl` de retenir les lignes. Pour faire échouer un pipeline sur une
exécution en échec, vérifiez `outcome` :

```bash
outcome=$(curl -sS -X POST "$SERVER/api/run" -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" -d '{"template":"http-check"}' | jq -r .outcome)
[ "$outcome" = "passed" ]
```

## Depuis la ligne de commande {#cli}

[`signallab run`](../automation/cli.md#cli-run) avec `--server <url>` exécute sur
un serveur via ce point de terminaison : il envoie l’expérience qu’il a lue, avec
`overrides`, `seed` et `timeout`, demande du NDJSON, et affiche chaque étape à
l’arrivée de sa ligne. Le jeton vient de `--token-file`, sinon
`SIGNALLAB_TOKEN`. Avec `--report` il télécharge le rapport via `/api/files`. Un
serveur qui n’envoie rien pendant 60 s — même pas un battement — est considéré
comme parti.

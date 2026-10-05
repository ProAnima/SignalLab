---
title: "API HTTP"
description: "Pilotez un serveur Signal Lab depuis des scripts, la CI et d’autres outils avec les mêmes commandes et événements que ceux qu’utilise sa propre interface."
---

# L’API HTTP

Pour piloter Signal Lab depuis un script, une chaîne de CI ou un autre outil,
parlez à un serveur Signal Lab (`signal-lab-server`, ou l’image Docker) en HTTP.
L’interface que le serveur affiche dans un navigateur utilise exactement cette
API : chaque bouton est un appel à `/api/invoke/<command>`, chaque valeur en
direct arrive sur `/api/events`. Tout ce qu’une personne peut faire sur la page
du serveur, un script peut donc le faire aussi.

L’application de bureau n’a pas d’API HTTP : sa fenêtre atteint son moteur à
l’intérieur de l’application. Pour automatiser sur un poste de bureau, exécutez
un serveur sur la même machine (voir [le serveur](../server/index.md)) ou
utilisez la ligne de commande [`signallab`](../automation/cli.md).

## Points de terminaison {#endpoints}

| Méthode et chemin | Ce que cela fait | Jeton |
| --- | --- | --- |
| `GET /api/health` | Si le serveur répond, sa version, s’il demande un jeton | pas nécessaire |
| `POST /api/invoke/<command>` | Exécute une commande du moteur : arguments JSON en entrée, résultat JSON en sortie ([commandes](commands.md)) | nécessaire |
| `POST /api/run` | Exécute une expérience jusqu’au bout : le résultat, ou ses étapes sous forme de lignes ([exécutions](run.md)) | nécessaire |
| `GET /api/events` | WebSocket de chaque événement du moteur ([événements](events.md)) | nécessaire |
| `GET /api/files?path=…` | Un fichier écrit par le moteur dans le dossier de données, en téléchargement | nécessaire |
| `GET /api/openapi.json` | Cette API décrite en OpenAPI 3.1 | nécessaire |
| `GET /login`, `POST /login` | La page et le formulaire de connexion pour les navigateurs | pas nécessaire |
| `POST /logout` | Met fin à la session d’un navigateur | une session |

Tout autre chemin sous `/api/` répond `404` avec le code `api.not_found` ; une
méthode qu’un chemin ne prend pas (`GET /api/invoke/…`) donne `405`, avec un
corps vide. Tout le reste est l’interface ; sur un serveur avec un jeton, un
navigateur sans session est d’abord renvoyé vers `/login`.

## URL de base {#base-url}

Un serveur écoute sur `http://127.0.0.1:1430` sauf indication contraire avec
`--listen` (ou `SIGNALLAB_LISTEN`). L’image Docker écoute sur toutes les cartes
réseau, `0.0.0.0:1430`. Les exemples de ces pages utilisent :

```bash
SERVER=http://127.0.0.1:1430
```

Le serveur parle HTTP en clair. Pour HTTPS, placez devant lui un proxy inverse
qui termine TLS et démarrez le serveur avec `--secure-cookie`.

## Authentification {#authentication}

Un serveur démarré sans jeton n’écoute qu’en boucle locale et ne demande aucune
authentification : quiconque sur cette machine peut l’utiliser. Un serveur que
d’autres peuvent joindre a toujours un jeton, et dès lors chaque requête sauf
`/api/health` et `/login` doit le porter.

**Les scripts** envoient le jeton dans l’en-tête `Authorization` :

```bash
TOKEN=$(cat token.txt)
curl -fsS "$SERVER/api/invoke/jobs_list" -X POST \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json"
```

L’en-tête doit être exactement `Bearer`, un espace, puis le jeton. Un jeton
manquant ou erroné donne `401` avec le code `auth.required`.

**Les navigateurs** se connectent une fois à `/login` avec le jeton et reçoivent
un cookie de session, `signallab_session` : `HttpOnly`, `SameSite=Strict`,
conservé 7 jours, et `Secure` quand le serveur tourne avec `--secure-cookie`.
`POST /logout` y met fin. Un jeton erroné sur le formulaire de connexion coûte
une seconde avant la réponse, ce qui ralentit les tentatives. Les sessions
vivent dans la mémoire du serveur : un redémarrage déconnecte tous les
navigateurs, tandis que les scripts munis du jeton ne sont pas affectés. Le
serveur conserve au plus 1024 sessions ; au-delà, la plus ancienne part.

D’où vient le jeton relève de la configuration du serveur (`--token-file`,
`SIGNALLAB_TOKEN`, ou le fichier `token` que `--generate-token` crée dans le
dossier de données) : voir [Sécurité du serveur](../server/security.md). Un
jeton compte au moins 24 caractères et aucun espace ; `signal-lab-server token`
en affiche un nouveau.

::: warning
Quiconque détient le jeton peut faire envoyer du trafic au serveur. Gardez le
fichier qui le contient lisible par vous seul, et ne le mettez jamais dans une
URL — le serveur n’y lit de toute façon pas de jeton.
:::

## Hôte et origine {#host-origin}

Deux contrôles s’exécutent avant tout le reste, sur chaque chemin, `/api/health`
compris.

**`Host`.** L’en-tête `Host` doit nommer ce serveur :

- Les noms de boucle locale passent toujours : `localhost`, les noms se
  terminant par `.localhost`, `127.x.x.x` et `[::1]`.
- Les noms donnés avec `--allowed-host` (ou `SIGNALLAB_ALLOWED_HOSTS`) passent.
- Un serveur avec un jeton et sans `--allowed-host` répond à n’importe quel nom.

Tout autre donne `403` avec `auth.host`. Adressez le serveur par un nom qu’il
accepte : `curl` envoie l’hôte de l’URL que vous lui donnez.

**`Origin`.** Une requête qui modifie quelque chose (toute méthode sauf `GET` et
`HEAD`) et la mise à niveau WebSocket doivent venir de la page propre au serveur
quand elles portent un en-tête `Origin` : son hôte et son port doivent être
égaux à `Host`. Sinon la réponse est `403` avec `auth.origin` ; `Origin: null`
est refusé aussi. Les scripts et `curl` n’envoient pas d’`Origin` et passent ce
contrôle — il leur faut tout de même le jeton.

**JSON uniquement.** `POST /api/invoke/…` et `POST /api/run` prennent
`Content-Type: application/json` (les paramètres comme `; charset=utf-8` sont
acceptés). Tout autre donne `415` avec `command.json_required`. Une page web
d’un autre site ne peut pas envoyer cela sans demander d’abord au serveur, et le
serveur ne donne jamais son accord.

## Appeler une commande {#invoke}

```http
POST /api/invoke/<command>
Content-Type: application/json

{ "argument": "value", … }
```

- Le corps est un objet JSON avec les arguments de la commande. Une commande
  sans argument prend `{}` ou un corps vide (l’en-tête `Content-Type` reste
  nécessaire).
- Les noms d’arguments sont en camelCase, comme l’interface les envoie :
  `jobId`, `nodeId`. Les objets passés en argument (`config`, `request`,
  `document`, `library`…) gardent les noms de champs écrits par le moteur, qui
  sont surtout en snake_case : `timeout_ms`, `port_start`.
- Un argument qu’une commande ne connaît pas est une erreur, jamais ignoré :
  `422` avec `command.args_invalid` nommant la commande, et les mots de
  l’analyseur dans `detail`. Un argument requis manquant aussi. Une commande
  sans argument ne lit pas du tout le corps.
- Un argument optionnel peut être omis ou envoyé à `null`.
- La réponse est `200` avec le résultat de la commande en JSON. Une commande
  qui n’a rien à renvoyer répond `null`.

Chaque commande, avec ses arguments et son résultat, se trouve dans
[commandes](commands.md).

## Erreurs {#errors}

| Statut | Quand | Corps |
| --- | --- | --- |
| `200` | La commande a tourné ; `/api/run` a démarré l’exécution | Le résultat |
| `400` | Le corps n’est pas du JSON ; une demande d’exécution ne peut pas être lue | `EngineError` : `command.args_invalid`, `api.run_invalid`, `api.run_source` |
| `400` | `/api/files` sans `path` | Du texte brut du serveur web, pas un `EngineError` |
| `401` | Aucun jeton, ou un mauvais | `EngineError` : `auth.required` |
| `403` | Un `Host` ou un `Origin` que le serveur refuse | `EngineError` : `auth.host`, `auth.origin` |
| `404` | Aucun chemin d’API de ce genre ; un fichier qui n’est pas dans le dossier de données | `EngineError` : `api.not_found`, `file.not_found` |
| `405` | Une méthode que le chemin ne prend pas | Vide |
| `413` | Un corps de requête de plus de 24 Mio | Du texte brut du serveur web, pas un `EngineError` |
| `413` | Un téléchargement de plus de 256 Mio | `EngineError` : `file.too_large` |
| `415` | Pas `application/json` | `EngineError` : `command.json_required` |
| `422` | La commande a échoué, ou l’exécution n’a pas pu démarrer | `EngineError` : n’importe quel code du moteur |
| `500` | Un fichier n’a pas pu être lu | `EngineError` : `file.io` |

Un nom de commande inconnu donne `422` avec `command.unknown`.

Chaque échec rapporté par le moteur a une seule forme, l’`EngineError` :

```json
{
  "code": "transport.refused",
  "params": { "target": "http://127.0.0.1:8080/" },
  "node": "request",
  "field": { "key": "url" },
  "detail": "error sending request for url (http://127.0.0.1:8080/): tcp connect error: Connection refused (os error 111)"
}
```

| Champ | Ce que c’est |
| --- | --- |
| `code` | Ce qui a mal tourné : un identifiant stable. Chaque code et son message sont listés dans [messages d’erreur](../reference/errors.md), groupés par la partie avant le point (par exemple [`transport`](../reference/errors.md#transport)) |
| `params` | Les valeurs que nomme le message, toutes en texte. Omis quand il n’y en a pas |
| `node` | Le nœud d’expérience concerné. Omis quand il n’y en a pas |
| `field` | Le champ concerné : `key` (nommé dans [champs](../reference/errors.md#fields)) et `index`, à partir de 1, pour les champs répétés comme un en-tête. Omis quand il n’y en a pas |
| `detail` | Les mots propres au système d’exploitation, à un analyseur ou à une bibliothèque, en anglais. Omis quand il n’y en a pas |

Branchez sur `code`, jamais sur `detail`. Les valeurs de secrets qu’utilise une
exécution ou un envoi unique sont masquées (`••••`) dans chaque erreur qu’il
rapporte.

Une requête qui atteint son serveur mais reçoit un statut d’erreur, ou aucune
réponse, n’est pas une commande en échec : `http_request` répond `200` avec la
réponse, et `ok`, `error` et `cause` disent ce qui s’est passé. Voir
[`http_request`](commands.md#http_request).

## Limites {#limits}

| Quoi | Limite | À la limite |
| --- | --- | --- |
| Un corps de requête | 24 Mio | `413` |
| Un document d’expérience | 4 Mio | `file.too_large` |
| Un téléchargement depuis `/api/files` | 256 Mio | `413`, `file.too_large` |
| La durée d’une exécution | 300 s | L’exécution échoue avec `run.timeout` |
| Le formulaire de retour (`feedback_send`) | 15 Mio au total | `feedback.too_large` |
| Les événements en attente pour un WebSocket | 4096 | Il reçoit [`server://lagged`](events.md#event-server-lagged) avec combien il en a manqués |

## Événements {#events}

`GET /api/events` mis à niveau en WebSocket diffuse chaque événement envoyé par le
moteur — les étapes d’une exécution, la fin d’une tâche, les messages d’un
moniteur, les trames de l’Inspecteur — à chaque client connecté, sous forme de
messages texte :

```json
{ "event": "job://ended", "payload": { "job_id": 7, "kind": "storm", "error": null } }
```

Il prend le même jeton et les mêmes règles d’`Origin` que le reste. Chaque canal
et sa charge utile se trouvent dans [événements](events.md).

## Fichiers {#files}

`GET /api/files?path=<path>` télécharge un fichier écrit par le moteur dans le
dossier de données du serveur : un rapport d’exécution (`report_path` du résultat
d’une exécution), un export de l’expérience ou de l’Inspecteur, la bibliothèque
de signaux ou d’émulateurs. `path` est le chemin que le moteur vous a donné, sur
le serveur (encodé pour URL) :

```bash
curl -fsS -G "$SERVER/api/files" --data-urlencode "path=/data/runs/run-1759600000000-3.json" \
  -H "Authorization: Bearer $TOKEN" -o report.json
```

- Seuls les fichiers situés à l’intérieur du dossier de données sont servis.
  Tout le reste, un dossier ou un fichier qui n’existe pas, donne `404` avec
  `file.not_found`.
- La réponse est `application/octet-stream` avec
  `Content-Disposition: attachment`. Les caractères du nom de fichier autres que
  les lettres, les chiffres, `.`, `_` et `-` deviennent `_`.
- Un fichier de plus de 256 Mio donne `413` avec `file.too_large`.

Ce que contient le dossier de données se trouve dans
[fichiers et dossiers](../reference/files.md).

## Santé {#health}

`GET /api/health` est ouvert : il ne demande aucun jeton, seulement un `Host` que
le serveur accepte.

```bash
curl -fsS "$SERVER/api/health"
```

```json
{ "status": "ok", "version": "[[version]]", "auth": true }
```

`auth` dit si les requêtes demandent un jeton. `signal-lab-server healthcheck`
interroge la même adresse sur laquelle le serveur écoute et se termine avec `0`
quand il répond ; le contrôle de santé de l’image Docker l’exécute.

## Description OpenAPI {#openapi}

`GET /api/openapi.json` décrit cette API en OpenAPI 3.1 : les points de
terminaison, chaque commande avec ses arguments, et la demande et le résultat
d’exécution. Il demande le jeton comme le reste de `/api/`. Ces pages sont la
référence complète ; là où les deux diffèrent, ces pages suivent le moteur.

## Tâches {#jobs}

Un travail de longue durée — un moniteur, un générateur, une rafale, un relais,
une connexion, un émulateur, une exécution — est une tâche. Une commande qui en
démarre une renvoie son `JobInfo` dès qu’elle tourne :

```json
{ "id": 4, "kind": "osc-monitor", "label": "OSC monitor 0.0.0.0:9000", "params": { "bind": "0.0.0.0:9000" }, "started_ms": 1759600000000 }
```

| Champ | Ce que c’est |
| --- | --- |
| `id` | Le numéro de la tâche, unique tant que le serveur tourne ; les autres commandes le prennent comme `jobId` ou `id` |
| `kind` | `experiment`, `osc-monitor`, `osc-gen`, `http-burst`, `netsim`, `storm`, `scan`, `beacon`, `discovery`, `mqtt`, `websocket` ou `emulator` |
| `label` | Une ligne en anglais, pour les journaux |
| `params` | Les valeurs dont le label est fait (target, bind, hôte…). Omis quand il n’y en a pas |
| `started_ms` | Quand elle a démarré, millisecondes depuis 1970 |

Ces commandes démarrent une tâche : `experiment_start`, `osc_monitor_start`,
`osc_generator_start`, `http_burst_start`, `netsim_start`, `storm_start`,
`scan_start`, `broadcast_beacon_start`, `discovery_start`, `mqtt_connect`,
`ws_connect` et `emulator_start`. Le serveur journalise chaque démarrage avec
l’adresse du client qui l’a demandé ; `POST /api/run` démarre lui aussi une tâche.

- [`jobs_list`](commands.md#jobs_list) liste les tâches en cours,
  [`job_stop`](commands.md#job_stop) en arrête une,
  [`jobs_stop_all`](commands.md#jobs_stop_all) les arrête toutes.
- Une tâche qui se termine d’elle-même, ou échoue, envoie
  [`job://ended`](events.md#event-job-ended). Une tâche que vous arrêtez n’envoie
  plus rien : `job_stop` répondant `true` en est la confirmation.
- Les tâches appartiennent au serveur, pas au client qui les a démarrées. Fermer
  la page ou terminer le script ne les arrête pas, chaque client les voit et peut
  les arrêter, et un serveur qui s’arrête les arrête toutes.

## Un exemple complet {#example}

Demandez si le serveur est en marche, démarrez un petit émulateur HTTP d’une
seule commande, exécutez l’expérience `http-check` fournie contre lui et
attendez le résultat, puis arrêtez l’émulateur. `jq` extrait des champs des
réponses.

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)            # leave out with a loopback server without a token
AUTH="Authorization: Bearer $TOKEN"
JSON="Content-Type: application/json"

# 1. Up? Which version? Does it want a token?
curl -fsS "$SERVER/api/health"
# {"status":"ok","version":"[[version]]","auth":true}

# 2. One command: an HTTP emulator on 127.0.0.1:8080 that answers GET / with 200
EMULATOR=$(curl -fsS -X POST "$SERVER/api/invoke/emulator_start" -H "$AUTH" -H "$JSON" -d '{
  "emulator": { "name": "Example", "bind": "127.0.0.1:8080", "protocol": "http",
                "routes": [ { "method": "GET", "path": "/", "responses": [ { "body": "ok" } ] } ] }
}' | jq .id)

# 3. Run the bundled experiment that expects 200 from http://127.0.0.1:8080/, and wait
curl -sS -X POST "$SERVER/api/run" -H "$AUTH" -H "$JSON" -d '{"template":"http-check"}' \
  | jq '{outcome, error, report_path}'
# {"outcome":"passed","error":null,"report_path":"/data/runs/run-1759600000000-2.json"}

# 4. Stop the emulator
curl -fsS -X POST "$SERVER/api/invoke/job_stop" -H "$AUTH" -H "$JSON" -d "{\"id\":$EMULATOR}"
# true
```

`curl -f` transforme un statut d’erreur en commande en échec ; omettez-le (comme
à l’étape 3) pour voir l’`EngineError` dans le corps.

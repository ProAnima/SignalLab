---
title: "Événements"
description: "Le WebSocket des événements du moteur sur un serveur Signal Lab, et chaque canal avec sa charge utile et le moment où il est envoyé."
---

# Événements

Tout ce qui se passe pendant qu’une tâche tourne — les étapes d’une exécution, les
messages d’un moniteur, les chiffres d’une rafale, les trames de l’Inspecteur, la
fin d’une tâche — est envoyé sous forme d’événement. Dans un navigateur, la page
du serveur les reçoit sur un seul WebSocket, `/api/events` ; un script peut
écouter sur le même socket. L’application de bureau reçoit les mêmes événements,
avec les mêmes noms et charges utiles, à l’intérieur de l’application.

## S’abonner {#subscribe}

Ouvrez un WebSocket vers `/api/events` sur le serveur :

```bash
websocat -H "Authorization: Bearer $TOKEN" ws://127.0.0.1:1430/api/events
```

- **L’authentification** est celle du reste de l’API : le jeton en
  `Authorization: Bearer`, ou le cookie de session d’un navigateur. Sans cela la
  mise à niveau est refusée avec `401` `auth.required`.
- **Origin** : un client qui envoie un en-tête `Origin` doit envoyer celui du
  serveur lui-même (hôte et port égaux à `Host`), sinon la mise à niveau est
  refusée avec `403` `auth.origin`. La plupart des bibliothèques WebSocket hors
  navigateur n’en envoient aucun.
- **Chaque événement à chaque client.** Il n’y a rien à quoi s’abonner : chaque
  socket reçoit chaque événement de chaque tâche, qui que ce soit qui l’ait
  démarrée. Choisissez ce dont vous avez besoin par `event`, et par `job_id` dans
  la charge utile.
- **Écoute seulement.** Le serveur ignore ce qu’envoie un client, sauf une
  fermeture ; un message de plus de 64 Kio ferme le socket.
- **Maintien en vie.** Le serveur envoie un ping toutes les 20 s, pour qu’un
  socket silencieux reste ouvert à travers les proxys. Quand le serveur
  s’arrête, il ferme chaque socket.
- **Rien n’est rejoué.** Les événements envoyés pendant qu’un client n’était pas
  connecté sont perdus pour lui. Un client qui se reconnecte doit relire l’état
  courant avec des commandes (`jobs_list`, `inspect_snapshot`,
  `emulator_exchanges`…).
- **Prendre du retard.** Jusqu’à 4096 événements attendent pour un socket. Un
  client qui prend plus de retard reçoit [`server://lagged`](#event-server-lagged)
  avec combien il en a manqués.

## Format des messages {#format}

Chaque événement est un message texte contenant un objet JSON :

```json
{ "event": "scan://open", "payload": { "job_id": 9, "ts": 1759600000123, "port": 8080, "banner": null } }
```

| Champ | Ce que c’est |
| --- | --- |
| `event` | Le canal, plus bas |
| `payload` | Les valeurs de l’événement ; sa forme dépend du canal |

Les heures (`ts`, les `first_ms` et `last_ms` d’un pair) sont des millisecondes
depuis 1970 ; les latences et les autres durées (`*_latency_ms`, `p50_ms`…, `ms`)
sont en millisecondes. Les erreurs des charges utiles sont des objets
[`EngineError`](index.md#errors) ; leurs codes sont listés dans
[messages d’erreur](../reference/errors.md).

## Canaux {#channels}

| Canal | Envoyé par | Quand |
| --- | --- | --- |
| [`experiment://step`](#event-experiment-step) | Une exécution | Une étape commence, réussit, échoue, réessaie, se répète ou rapporte une charge |
| [`experiment://ended`](#event-experiment-ended) | Une exécution | Une fois, quand l’exécution se termine d’elle-même |
| [`job://ended`](#event-job-ended) | Chaque tâche | Une fois, quand la tâche se termine d’elle-même ou échoue |
| [`osc://message`](#event-osc-message) | Moniteur OSC | Chaque paquet |
| [`osc://gen-tick`](#event-osc-gen-tick) | Générateur OSC | Chaque message, ou 30 à 45 fois par seconde au-delà de 60 messages par seconde |
| [`http://burst-progress`](#event-http-burst-progress) | Rafale HTTP | Toutes les 100 ms, et à la fin |
| [`ws://state`](#event-ws-state) | Connexion WebSocket | Connectée, fermée |
| [`ws://messages`](#event-ws-messages) | Connexion WebSocket | Toutes les 100 ms avec du nouveau |
| [`mqtt://state`](#event-mqtt-state) | Connexion MQTT | Connectée, abonnée, fermée |
| [`mqtt://messages`](#event-mqtt-messages) | Connexion MQTT | Toutes les 100 ms avec du nouveau |
| [`mqtt://ack`](#event-mqtt-ack) | Connexion MQTT | Une publication QoS 1/2 terminée ; un désabonnement ayant reçu une réponse |
| [`broadcast://emit-stat`](#event-broadcast-emit-stat) | Balise | Toutes les 250 ms, et à la fin |
| [`broadcast://peers`](#event-broadcast-peers) | Écoute de découverte | Toutes les 400 ms |
| [`netsim://stat`](#event-netsim-stat) | Relais de dégradation | Toutes les 250 ms |
| [`storm://stat`](#event-storm-stat) | Tempête | Toutes les 250 ms, et à la fin |
| [`scan://open`](#event-scan-open) | Scanner | Chaque port ouvert |
| [`scan://progress`](#event-scan-progress) | Scanner | Environ tous les 1 % de la plage, et à la fin |
| [`emulator://activity`](#event-emulator-activity) | Tâche d’émulateur | Toutes les 200 ms avec du nouveau |
| [`inspect://batch`](#event-inspect-batch) | Inspecteur | Toutes les 120 ms avec de nouvelles trames, environ une fois par seconde au calme, tant que la capture est active |
| [`server://lagged`](#event-server-lagged) | Le serveur | Un client a pris du retard |

### `experiment://step` {#event-experiment-step}

Une étape d’une exécution : un nœud qui commence, réussit, échoue, attend de
réessayer, se répète, ou rapporte la progression d’une charge. Une exécution
démarrée avec `/api/run` envoie les mêmes étapes sur sa réponse (voir
[exécutions](run.md)).

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche de l’exécution |
| `ts` | number | Quand |
| `node_id` | string | Le nœud |
| `state` | string | `running`, `passed`, `failed`, `retry` (une tentative a échoué et l’étape repart après une pause), `repeating` (la progression d’une action qui se répète, au plus une fois par seconde) ou `load` (la progression d’une charge, au plus une fois par seconde) |
| `detail` | string | Ce qui s’est passé, en anglais ; vide pour `running` et `failed` (voir `error`) |
| `message_key` | string ou null | Le texte de l’interface pour cela, sous forme de clé de son dictionnaire |
| `message_params` | object ou null | Les valeurs que nomme `message_key` |
| `vars` | object | Les variables écrites par l’étape ; omis quand il n’y en a aucune |
| `error` | `EngineError` | Pourquoi elle a échoué, ou pourquoi la tentative l’a fait (`retry`) ; omis sinon |
| `frame` | number | La trame de l’Inspecteur du message qu’une attente (ou la réponse attendue d’un envoi) a pris, quand la capture était active ; omis sinon |
| `load` | object | Ce qu’une charge a mesuré, ses seuils lus — sur le dernier événement d’une étape de charge, réussie ou échouée ; omis sinon. Voir [charge](../experiments/load.md) |

Le nœud [[ui:exp.node.end]] d’une exécution indique `running` quand la première
branche l’atteint et `passed` une fois que chaque branche s’est terminée sans
échec. Les valeurs de secrets sont masquées dans chaque champ.

### `experiment://ended` {#event-experiment-ended}

Une exécution s’est terminée d’elle-même : elle a réussi, échoué ou dépassé son
temps. Envoyé juste après la même charge utile sur
[`job://ended`](#event-job-ended). Une exécution arrêtée avec `job_stop` ou
[[ui:app.stopAll]] n’envoie ni l’un ni l’autre et n’enregistre aucun rapport.

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche de l’exécution |
| `kind` | string | `experiment` |
| `seed` | number | La graine avec laquelle elle a tourné |
| `profile` | string ou null | Son profil |
| `overridden` | boolean | Certaines valeurs de paramètres venaient de [[ui:exp.runWith]] ou d’`overrides` |
| `error` | `EngineError` ou null | Le premier échec de l’exécution ; null quand elle a réussi |
| `report_path` | string ou null | Son rapport, dans `runs/` du dossier de données |
| `report_error` | `EngineError` ou null | Pourquoi le rapport n’a pas pu être écrit |

### `job://ended` {#event-job-ended}

Une tâche s’est terminée d’elle-même ou a échoué. Une tâche arrêtée avec
`job_stop` ou `jobs_stop_all` ne l’envoie pas.

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche |
| `kind` | string | `osc-monitor`, `osc-gen`, `http-burst`, `netsim`, `storm`, `scan`, `beacon`, `discovery`, `mqtt`, `websocket`, `emulator` ou `experiment` |
| `error` | `EngineError` ou null | Pourquoi elle a pris fin, quand quelque chose a mal tourné |

Le `job://ended` d’une exécution porte aussi les champs
d’[`experiment://ended`](#event-experiment-ended). Ce qui termine chaque type :

| `kind` | Se termine quand | `error` |
| --- | --- | --- |
| `osc-monitor` | Le socket ne peut plus recevoir | `wait.receive_failed` |
| `osc-gen` | Sa durée est écoulée, ou un envoi échoue | null, ou `transport.*` |
| `http-burst` | Son total ou sa durée est atteint | null |
| `storm` | Sa durée est écoulée | null |
| `scan` | Chaque port de la plage a été essayé | null |
| `beacon` | Ses tours ou sa durée sont écoulés, ou plus de 32 envois ont échoué sans aucun réussi | null, ou `transport.*` |
| `discovery` | Le socket ne peut plus recevoir | `wait.receive_failed` |
| `mqtt` | Le broker a fermé la connexion ou elle a été perdue | `transport.*` (`transport.reset` quand le broker l’a fermée), ou `mqtt.protocol` |
| `websocket` | La connexion s’est fermée | null, ou pourquoi elle a été perdue |
| `netsim` | Le relais ne peut plus fonctionner | pourquoi |
| `emulator` | Son socket échoue | pourquoi |
| `experiment` | L’exécution se termine | l’échec de l’exécution, ou null |

### `osc://message` {#event-osc-message}

Un paquet UDP reçu par un moniteur OSC, décodé. Envoyé pour chaque paquet, sans
regroupement.

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche du moniteur |
| `ts` | number | Quand il est arrivé |
| `from` | string | L’expéditeur, `IP:port` |
| `bytes` | number | La taille du paquet |
| `messages` | object[] | Chaque message du paquet (un bundle en a plusieurs) : `address` et `args` ([`OscArg`](commands.md#type-oscarg)`[]`) |
| `error` | `EngineError` ou null | `osc.packet_malformed` quand le paquet n’a pas été décodé (alors `messages` est vide) |

### `osc://gen-tick` {#event-osc-gen-tick}

La progression d’un générateur OSC : pour chaque message en dessous de 60
messages par seconde ; au-delà, pour chaque n-ième, n étant le débit divisé par
30 et arrondi vers le bas — 30 à 45 fois par seconde.

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche du générateur |
| `ts` | number | Quand |
| `value` | number | La valeur qui vient d’être envoyée, avant d’être arrondie à un entier ou à un flottant 32 bits |
| `sent` | number | Messages envoyés jusqu’ici |

### `http://burst-progress` {#event-http-burst-progress}

Les chiffres d’une rafale HTTP, toutes les 100 ms pendant qu’elle tourne, et une
fois de plus avec `done: true` quand elle se termine d’elle-même.

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche de la rafale |
| `ts` | number | Quand |
| `sent` | number | Requêtes ayant reçu une réponse ou échoué jusqu’ici |
| `ok` | number | Parmi elles, celles répondant par un statut 2xx |
| `failed` | number | Parmi elles, tout autre statut ou aucune réponse |
| `missed` | number | Les requêtes d’une rafale cadencée qui ont attendu trop longtemps un travailleur libre et ont été sautées |
| `rps` | number | Requêtes par seconde sur les 100 dernières ms ; dans le dernier événement, sur toute la rafale |
| `last_latency_ms`, `min_latency_ms`, `max_latency_ms`, `avg_latency_ms` | number | Latences jusqu’ici |
| `p50_ms`, `p90_ms`, `p95_ms`, `p99_ms` | number | Centiles de chaque requête jusqu’ici, échecs compris, à 0,5 % près |
| `done` | boolean | Le dernier événement de la rafale |

### `ws://state` {#event-ws-state}

Une connexion WebSocket ouverte par `ws_connect` s’est connectée, ou fermée. Une
connexion dont la tâche a été arrêtée n’envoie pas de `closed`.

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche de la connexion |
| `ts` | number | Quand |
| `state` | string | `connected` ou `closed` |
| `handshake` | object | `url`, `peer`, `local`, `protocol` (le sous-protocole choisi par le serveur, ou null) et `ms` (la connexion et la mise à niveau) |
| `closed` | object ou null | Avec `closed` : `code`, `reason`, `by` (`client`, `server` ou `lost`) et `error` |

### `ws://messages` {#event-ws-messages}

Ce qu’une connexion WebSocket a envoyé et reçu depuis le dernier événement,
toutes les 100 ms quand il y a quelque chose.

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche de la connexion |
| `ts` | number | Quand |
| `messages` | object[] | Dans l’ordre : `ts`, `dir` (`rx` reçu, `tx` envoyé), `kind` (`text` ou `binary`), `text` (les 64 premiers Kio en UTF-8, ceux d’un message binaire aussi ; les octets qui n’en sont pas deviennent `�`), `hex` (les 4096 premiers octets d’un message binaire en hex, sinon null), `bytes` (la taille complète) et `truncated` (plus que ce qui a été montré : au-delà de 64 Kio de texte, au-delà de 4096 octets de binaire) |
| `dropped` | number | Messages écartés de cet événement parce qu’il y en avait plus de 2000 ; les plus anciens partent en premier |

### `mqtt://state` {#event-mqtt-state}

L’état d’une connexion MQTT a changé.

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche de la connexion |
| `ts` | number | Quand |
| `state` | string | `connected` ; `subscribed` après chaque réponse à un abonnement ; `closed` quand la connexion a pris fin (pas quand sa tâche a été arrêtée) |
| `broker` | string | `host:port` |
| `error` | `EngineError` ou null | Pourquoi une connexion `closed` a pris fin (`transport.reset` quand le broker l’a fermée) ; null sinon |
| `grants` | object[] | Avec `subscribed` : chaque filtre demandé, avec `filter`, `qos` (accordé) et `accepted` ; vide sinon |

### `mqtt://messages` {#event-mqtt-messages}

Ce qu’une connexion MQTT a reçu depuis le dernier événement, toutes les 100 ms
quand il y a quelque chose. Un message QoS 2 relivré est montré une fois.

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche de la connexion |
| `ts` | number | Quand |
| `messages` | object[] | `ts`, `topic`, `payload` (en UTF-8 ; les octets qui n’en sont pas deviennent `�`), `bytes`, `qos`, `retain`, `dup` |
| `dropped` | number | Messages écartés parce que plus de 4000 sont arrivés en 100 ms ; les plus anciens partent en premier |

### `mqtt://ack` {#event-mqtt-ack}

Le broker a terminé quelque chose que la connexion avait demandé.

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche de la connexion |
| `ts` | number | Quand |
| `kind` | string | `published` (une publication QoS 1 ou 2 est terminée) ou `unsubscribed` |
| `packet_id` | number | L’identifiant du paquet MQTT |
| `topic` | string ou null | Le topic publié ; null pour `unsubscribed` |

### `broadcast://emit-stat` {#event-broadcast-emit-stat}

Les compteurs d’une balise, toutes les 250 ms, et une fois de plus quand elle se
termine d’elle-même avec `pps` à 0.

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche de la balise |
| `ts` | number | Quand |
| `targets` | number | Destinations à chaque tour |
| `rounds` | number | Tours envoyés |
| `packets`, `bytes` | number | Datagrammes et octets envoyés |
| `errors` | number | Envois qui ont échoué |
| `pps` | number | Datagrammes par seconde sur les 250 dernières ms |

### `broadcast://peers` {#event-broadcast-peers}

Ce qu’une écoute de découverte a entendu, toutes les 400 ms.

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche de l’écoute |
| `ts` | number | Quand |
| `peers` | object[] | Les plus récemment entendus d’abord : `addr`, `proto`, `packets`, `bytes`, `first_ms`, `last_ms`, `last_summary`, `responded` (ses paquets qui ont reçu une réponse, comptés à leur arrivée) ; au plus 512 |
| `packets`, `bytes` | number | Tout ce qui a été reçu |
| `responses` | number | Réponses envoyées |

### `netsim://stat` {#event-netsim-stat}

Les compteurs d’un relais de dégradation, toutes les 250 ms. Les relais des nœuds
[[ui:exp.node.impairment]] d’une exécution rapportent dans le rapport de
l’exécution à la place.

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche du relais |
| `ts` | number | Quand |
| `received`, `forwarded` | number | Datagrammes ou blocs entrants et sortants |
| `dropped` | number | Perdus par `loss`, par des rafales ou par `offline` (UDP ; un relais TCP retient un flux pendant l’indisponibilité et ne perd rien) |
| `throttled` | number | UDP : abandonnés à cause de la limite de bande passante, ou parce que trop étaient déjà en route. TCP : les blocs qui ont retenu leur flux pour la limite de bande passante |
| `duplicated`, `corrupted`, `reordered` | number | Ce que le profil leur a fait |
| `bytes` | number | Octets transmis |
| `connections`, `reset`, `stalled` | number | TCP : connexions prises, réinitialisées, laissées à demi ouvertes ; omis tant que 0 |
| `profile` | string | Le profil avec lequel il dégrade maintenant, comme la chronologie le nomme : son nom, ou ce qu’il fait (`60 ms ±25 · loss 2%`) |

### `storm://stat` {#event-storm-stat}

Les compteurs d’une tempête, toutes les 250 ms, et une fois de plus quand elle se
termine d’elle-même avec `pps` et `mbps` à 0.

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche de la tempête |
| `ts` | number | Quand |
| `packets`, `bytes` | number | Datagrammes (ou connexions TCP) et octets envoyés |
| `errors` | number | Envois ou connexions qui ont échoué |
| `pps` | number | Par seconde sur les 250 dernières ms |
| `mbps` | number | Mégabits par seconde sur les 250 dernières ms |

### `scan://open` {#event-scan-open}

Le scanner a trouvé un port ouvert.

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche du scan |
| `ts` | number | Quand |
| `port` | number | Le port |
| `banner` | string ou null | Ce que le service a envoyé en premier, quand les bannières ont été demandées et qu’il a dit quelque chose dans les 400 ms |

### `scan://progress` {#event-scan-progress}

Où en est un scan : environ tous les 1 % de la plage, et quand il se termine
d’elle-même avec `done` égal à `total` (celui-là peut arriver deux fois).

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche du scan |
| `ts` | number | Quand |
| `done` | number | Ports essayés |
| `total` | number | Ports de la plage |
| `open` | number | Ports ouverts trouvés |

### `emulator://activity` {#event-emulator-activity}

Ce qu’un émulateur démarré avec `emulator_start` a reçu et répondu depuis le
dernier événement, toutes les 200 ms quand quelque chose a changé (un échange,
une mise en panne ou une remise en service, ou un message que le broker MQTT n’a
pas pu livrer). Les nœuds [[ui:exp.node.emulator]] d’une exécution ne l’envoient
pas ; leurs compteurs sont dans le rapport de l’exécution.

| Champ | Type | Signification |
| --- | --- | --- |
| `job_id` | number | La tâche de l’émulateur |
| `ts` | number | Quand |
| `counts` | object | `total`, `unmatched`, `failed`, `down`, `hits` (par règle) et `missed` (MQTT ; omis tant que 0) — comme [`emulator_exchanges`](commands.md#emulator_exchanges) |
| `forced` | string | `unavailable`, `reset` ou `timeout` tant qu’il est en panne ; omis sinon |
| `exchanges` | object[] | Les nouveaux échanges, comme `emulator_exchanges` les liste mais sans `data` ; au plus 200 |
| `dropped` | number | Les échanges au-delà des 200 premiers de l’intervalle, non envoyés ici ; `emulator_exchanges` a toujours les 500 derniers |

### `inspect://batch` {#event-inspect-batch}

De nouvelles trames de l’Inspecteur. Envoyé seulement tant que la capture est
active : toutes les 120 ms quand il y a de nouvelles trames, et environ une fois
par seconde quand il n’y en a pas, pour que les compteurs restent à jour.

| Champ | Type | Signification |
| --- | --- | --- |
| `frames` | object[] | Les nouvelles [trames](commands.md#type-frame), les plus anciennes d’abord, au plus 250 ; sans leurs octets (utiliser `inspect_payload`) |
| `stats` | object | Les compteurs de la capture, [`CaptureStats`](commands.md#type-frame) |
| `skipped_now` | number | Les trames capturées depuis le dernier lot mais absentes de celui-ci — plus de 250 sont arrivées, ou le tampon les a lâchées. Elles restent dans un export tant que le tampon les contient |

### `server://lagged` {#event-server-lagged}

Serveur uniquement. Ce client a pris plus de 4096 événements de retard et en a
manqué. Relisez l’état avec des commandes.

| Champ | Type | Signification |
| --- | --- | --- |
| `skipped` | number | Combien d’événements il a manqués |

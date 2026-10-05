---
title: "Nœuds"
description: "Chaque type de nœud d'une expérience — ce qu'il fait, ses champs avec leurs valeurs par défaut et leurs limites, ses sorties, les réglages qu'il accepte, et son aspect dans un fichier d'expérience."
---

# Référence des nœuds

Chaque type de nœud qu'une expérience peut contenir, dans les groupes du menu
d'ajout : [actions](#actions), [attentes](#waits), [émulation](#emulation),
[défaillances](#faults), [données](#data), [vérifications](#checks) et [flux](#flow).
Comment les ajouter et les relier est expliqué dans [L'éditeur](index.md) ;
`signallab nodes` imprime le même catalogue en JSON, pour les scripts et les
assistants ([La ligne de commande](../automation/cli.md)).

## Lire cette page {#reading}

Chaque nœud a un tableau de ses champs :

- **Champ** est le nom dans le volet des propriétés ; **Dans le fichier** est la
  clé dans le JSON de l'expérience.
- **Défaut** est ce qu'un nœud reçoit quand vous l'ajoutez dans l'éditeur. Là où un
  fichier peut omettre une clé, la valeur qu'il prend alors est donnée comme *si
  absente* ; les autres clés sont obligatoires dans un fichier.
- **Modèles** : *oui* — le champ accepte des `{{templates}}` : paramètres,
  variables définies plus tôt, secrets et générateurs, résolus pendant que l'étape
  s'exécute ([Données et modèles](data.md)). *Paramètres uniquement* — il est ouvert
  avant la première étape, quand seuls les paramètres sont connus. *Non* — la valeur
  est prise telle qu'écrite.

Les durées sont en millisecondes. Les limites sont vérifiées avant qu'une exécution
ne démarre ; un champ hors limites empêche l'expérience de s'exécuter et est
signalé sur le nœud.

## Un nœud dans un fichier {#file-shape}

Dans un fichier d'expérience, un nœud est un objet avec un `id` (unique dans
l'expérience), son `type`, sa place sur le canevas (`x`, `y`, zéro ou plus), ses
champs, et les réglages qu'il utilise (`retry`, `repeat`, `load`, omis quand ils
sont désactivés). Un fil est une arête allant de la sortie d'un nœud (`port`,
`next` si absent) vers un autre nœud :

```json
{
  "nodes": [
    { "id": "start", "type": "start", "x": 40, "y": 80 },
    { "id": "ping", "type": "udp", "x": 270, "y": 80, "target": "127.0.0.1:9000", "text": "PING",
      "retry": { "attempts": 3, "delay_ms": 500, "backoff": "fixed" } },
    { "id": "end", "type": "end", "x": 500, "y": 80 }
  ],
  "edges": [
    { "from": "start", "to": "ping", "port": "next" },
    { "from": "ping", "to": "end", "port": "next" }
  ]
}
```

Les exemples ci-dessous montrent un nœud chacun, tel qu'un fichier le contient.

## Réglages partagés par de nombreux nœuds {#settings}

Ils s'activent dans la partie basse des propriétés d'un nœud. Quel nœud accepte
lequel est indiqué sous chaque nœud.

| Réglage | Qui l'accepte | Ce qu'il fait |
| --- | --- | --- |
| [Réessai](#retry) | Les nœuds qui envoient ou écoutent : [[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]], [[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_connect]], [[ui:exp.node.ws_send]], et chaque attente | Réessaie quand l'étape échoue |
| [Répétition](#repeat) | Les nœuds qui envoient : [[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]], [[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_send]] | Envoie encore et encore, un certain nombre de fois ou pendant une durée |
| [Charge](#load) | [[ui:exp.node.http]] | Envoie la requête selon un profil de charge, mesurée et jugée par des seuils |
| [Attendre une réponse](#reply) | [[ui:exp.node.osc]], [[ui:exp.node.udp]] | Envoie et attend la réponse dans la même étape |

### Réessai {#retry}

[[ui:exp.retryOn]] : quand l'étape échoue — pas de connexion, un délai d'attente,
une attente sans rien qui corresponde — elle fait une pause et s'exécute de nouveau.
Chaque tentative échouée est une ligne dans la chronologie ; l'étape échoue quand la
dernière tentative échoue. Un modèle qui ne peut pas être résolu n'est pas réessayé.
[[ui:common.stop]] met aussi fin à une pause.

| Champ | Dans le fichier | Quoi | Défaut et limites |
| --- | --- | --- | --- |
| [[ui:exp.attempts]] | `retry.attempts` | Tentatives au total, la première comprise | 3 ; 2–10 dans l'éditeur (un fichier peut aussi dire 1) |
| [[ui:exp.retryDelay]] | `retry.delay_ms` | La pause avant la deuxième tentative | 500 ; 0–60 000 |
| [[ui:exp.backoff]] | `retry.backoff` | [[ui:exp.backoff.fixed]] (`fixed`) : la même pause à chaque fois ; [[ui:exp.backoff.exponential]] (`exponential`) : deux fois plus longue après chaque échec | `fixed` (aussi si absent) |

Aucune pause ne dépasse 60 secondes, quoi qu'il arrive au doublement. Une attente
dont la sortie [[ui:exp.portTimeout]] a un fil n'échoue pas sur un délai d'attente —
elle sort par cette sortie — elle n'est donc pas réessayée alors.

### Répétition {#repeat}

[[ui:exp.repeatOn]] : le nœud envoie encore et encore — un battement, une
interrogation, un flux régulier — sans boucle dans le graphe. Chaque envoi relit ses
modèles (`{{counter}}` est son numéro, `{{now}}` son instant), et le Réessai, quand
il est actif, s'applique à chaque envoi. L'étape réussit quand tous les envois ont
réussi ; un envoi qui échoue définitivement fait échouer l'étape. La chronologie
rapporte la progression au plus une fois par seconde.

| Champ | Dans le fichier | Quoi | Défaut et limites |
| --- | --- | --- | --- |
| [[ui:exp.repeatBy]] | `repeat.until` | [[ui:exp.repeatBy.count]] (`count`) ou [[ui:exp.repeatBy.duration]] (`duration`) | `count` (aussi si absent) |
| [[ui:exp.repeatCount]] | `repeat.count` | Envois au total, le premier compris | 10 (aussi si absent) ; 2–10 000 |
| [[ui:exp.repeatDuration]] | `repeat.duration_ms` | Combien de temps continuer d'envoyer, à partir du premier envoi | 10 000 (aussi si absent) ; 1–300 000 |
| [[ui:exp.repeatInterval]] | `repeat.interval_ms` | La pause entre deux envois | 1 000 ; 10–60 000 ; obligatoire dans un fichier |
| [[ui:exp.repeatJitter]] | `repeat.jitter_ms` | Chaque pause jusqu'à autant plus longue, tirée de la graine de l'exécution | 0 (aussi si absent) ; 0–60 000 |

Les répétitions doivent tenir dans les 300 secondes d'une exécution, et *pendant une
durée* doivent demander moins de 10 000 envois (sa durée divisée par l'intervalle).

### Charge {#load}

[[ui:exp.loadOn]], sur un [[ui:exp.node.http]] uniquement : la requête est envoyée
selon un profil — un débit constant, une rampe, des paliers, un pic ou des arrivées
aléatoires — avec jusqu'à 512 en vol en même temps (32 par défaut), et mesurée :
latences, erreurs, débit atteint. Des seuils décident si l'étape réussit. La charge
remplace la Répétition et le Réessai (une requête échouée est comptée, pas
réessayée), et ne laisse aucune réponse pour les vérifications qui la suivent. Ses
champs et résultats sont dans [Tests de charge](load.md).

### Attendre une réponse {#reply}

[[ui:exp.expectReply]], sur un [[ui:exp.node.osc]] ou un [[ui:exp.node.udp]] : le
message est envoyé depuis le port sur lequel la réponse est attendue, si bien qu'un
appareil qui répond à l'expéditeur est entendu, et l'étape ne réussit que quand une
réponse correspondante arrive à temps. Aucune réponse fait échouer l'étape — le
Réessai renvoie. La réponse est stockée dans une variable, comme celle d'une attente.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.replyOn]] | `reply.bind` | `IP:port` depuis lequel envoyer et écouter ; le port 0 prend n'importe quel port libre | `0.0.0.0:0` | Non |
| [[ui:exp.replyAddress]] (OSC) | `reply.address` | Le motif d'adresse de la réponse, comme dans [[[ui:exp.node.wait_osc]]](#node-wait_osc) | `/*` | Oui |
| [[ui:exp.argRules]] (OSC) | `reply.args` | Règles d'arguments, comme dans [[ui:exp.node.wait_osc]] | aucune ; 16 au plus | Valeurs : oui |
| [[ui:exp.replyMode]] (UDP) | `reply.mode` | `any`, `contains`, `regex` ou `hex` — voir [Correspondance de charge utile](#payload-matching) | `any` (aussi si absent) | Non |
| [[ui:field.pattern]] (UDP) | `reply.pattern` | Ce que la réponse doit contenir ou auquel elle doit correspondre | vide ; obligatoire sauf pour `any` | Oui |
| [[ui:exp.waitTimeout]] | `reply.timeout_ms` | Combien de temps attendre | 2 000 (aussi si absent) ; 1–120 000 | Non |
| [[ui:exp.replyVariable]] | `reply.variable` | La variable dans laquelle la réponse est stockée | `reply` (aussi si absent) | Non |

Le port de la réponse est ouvert avant la première étape, comme celui d'une attente.

## Actions {#actions}

Des nœuds qui envoient. Une attente après une action compte les messages à partir
du moment où l'action a démarré.

### Requête HTTP {#node-http}

Envoie une requête HTTP et conserve la réponse pour les vérifications, les
bifurcations et les nœuds [[ui:exp.node.extract]] qui la suivent.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.method]] | `request.method` | GET, HEAD, POST, PUT, PATCH, DELETE ou OPTIONS (un fichier peut nommer n'importe quelle méthode) | `GET` | Non |
| URL | `request.url` | Une URL `http://` ou `https://` | `http://127.0.0.1:8080/` | Oui |
| [[ui:common.timeoutMs]] | `request.timeout_ms` | Pour tout l'échange | 4 000 (10 000 si absent) ; 1–120 000 | Non |
| [[ui:exp.headers]] | `request.headers` | `[[name, value], …]` ; une ligne avec un nom vide est ignorée | aucune | Oui, noms et valeurs |
| [[ui:exp.body]] | `request.body` | Du texte, ou `null` pour aucun | `null` | Oui |
| [[ui:field.auth]] | `request.auth` | [[ui:http.auth.none]], [[ui:http.auth.basic]], [[ui:http.auth.bearer]] ou [[ui:http.auth.digest]], avec [[ui:field.username]] et [[ui:field.password]], ou [[ui:field.token]] | aucune | Oui |

- N'importe quelle réponse fait réussir l'étape, 404 et 500 compris : vérifiez le
  statut avec [[[ui:exp.node.assert_status]]](#node-assert_status) ou bifurquez
  dessus avec [[[ui:exp.node.branch_status]]](#node-branch_status). Une requête qui
  n'obtient aucune réponse — refusée, un délai d'attente, un nom qui ne se résout
  pas, un certificat qui n'est pas approuvé — fait échouer l'étape.
- Les redirections sont suivies, dix au plus. Les certificats `https://` sont
  vérifiés.
- Le corps de la réponse est conservé jusqu'à 256 Kio pour les vérifications ; un
  corps plus grand est coupé là (les vérifications le disent quand ce qu'elles
  cherchent peut se trouver après la coupure).
- Digest répond au challenge 401 du serveur et renvoie la requête. Les identifiants
  n'entrent que dans la requête : les étapes, les rapports et l'Inspecteur ne
  montrent jamais l'en-tête `Authorization`. Écrivez un mot de passe comme
  `{{secret.NAME}}`.
- Tant que l'expérience conserve les cookies (activé par défaut, sous
  [[ui:exp.params]]), ce que les serveurs définissent est renvoyé avec les requêtes
  ultérieures de l'exécution qui leur sont adressées.

Sorties : [[ui:exp.outputPort]]. Réglages : Réessai, Répétition, Charge.

```json
{ "id": "cue", "type": "http", "x": 270, "y": 80,
  "request": { "method": "POST", "url": "{{api}}/cue", "headers": [["Content-Type", "application/json"]],
               "body": "{\"cue\": 1}", "timeout_ms": 5000,
               "auth": { "scheme": "bearer", "token": "{{secret.API_TOKEN}}" } } }
```

Voir aussi [HTTP](../protocols/http.md).

### Message TCP {#node-tcp}

Se connecte à un hôte en TCP, écrit la charge utile, attend jusqu'à 250 ms les
premiers octets d'une réponse (il lit au plus 1 024 octets, une seule fois) et ferme
la connexion. La taille de la réponse est rapportée, pas vérifiée.

Dans l'[Inspecteur](../tools/inspector.md) l'étape est deux trames `tcp` avec la
source `experiment` : la charge utile écrite et, quand elle est venue, la réponse
lue. Les secrets utilisés sont masqués dans les deux, comme dans toute trame.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.host]] | `host` | Un nom d'hôte ou une adresse IP | `127.0.0.1` | Oui |
| [[ui:exp.port]] | `port` | | 9000 ; 1–65 535 | Non |
| [[ui:common.timeoutMs]] | `timeout_ms` | Pour la connexion, l'écriture et la réponse ensemble | 4 000 (aussi si absent) ; 1–120 000 | Non |
| [[ui:exp.payload]] | `payload` | Le texte écrit une fois connecté, en UTF-8 | `hello` | Oui |

L'étape échoue quand la connexion est refusée, que le nom ne se résout pas ou que le
temps s'écoule. Sorties : [[ui:exp.outputPort]]. Réglages : Réessai, Répétition.
[[ui:exp.sendNow]] se connecte et écrit la charge utile une fois, et le résultat du
nœud indique combien d'octets ont été envoyés et sont revenus.

```json
{ "id": "go", "type": "tcp", "x": 270, "y": 80, "host": "127.0.0.1", "port": 5000, "payload": "GO\r\n", "timeout_ms": 2000 }
```

### Message OSC {#node-osc}

Envoie un message OSC 1.0 en UDP.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:common.target]] | `target` | `IP:port` ou `host:port` ; un nom d'hôte est résolu quand l'étape envoie, son adresse IPv4 prise quand il en a une | `127.0.0.1:9000` | Oui |
| [[ui:common.address]] | `address` | Commence par `/` | `/test` | Oui |
| [[ui:exp.arguments]] | `args` | `[{ "type", "value" }, …]` — `int`, `float`, `str`, `long`, `double`, `bool`, `blob` (octets), `nil` (aucune valeur) | aucune | Valeurs texte (`str`) : oui |
| [[ui:exp.expectReply]] | `reply` | Facultatif : envoyer et attendre la réponse — voir [Attendre une réponse](#reply) | désactivé | |

Sorties : [[ui:exp.outputPort]] ; avec une réponse attendue, elle n'est suivie que
quand la réponse est venue. Réglages : Réessai, Répétition, une réponse.
⚡ [[ui:exp.routeThrough]] dans ses propriétés place un
[[[ui:exp.node.impairment]]](#node-impairment) devant lui.

```json
{ "id": "fader", "type": "osc", "x": 270, "y": 80, "target": "{{device}}", "address": "/fader/1",
  "args": [{ "type": "float", "value": 0.75 }] }
```

Voir aussi [OSC](../protocols/osc.md).

### Datagramme UDP {#node-udp}

Envoie une charge utile texte comme un datagramme UDP à une ou plusieurs cibles.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:common.target]] | `target` | `IP:port` ou `host:port` ; plusieurs séparées par des virgules, des points-virgules ou des retours à la ligne reçoivent chacune le datagramme. Un nom d'hôte est résolu quand l'étape envoie, son adresse IPv4 prise quand il en a une | `127.0.0.1:9000` | Oui |
| [[ui:exp.payload]] | `text` | La charge utile, en UTF-8 | `hello` ; 65 507 octets au plus | Oui |
| [[ui:exp.expectReply]] | `reply` | Facultatif : envoyer et attendre la réponse — voir [Attendre une réponse](#reply) | désactivé | |

L'étape échoue si une cible ne peut pas être atteinte. Sorties :
[[ui:exp.outputPort]]. Réglages : Réessai, Répétition, une réponse.

```json
{ "id": "ping", "type": "udp", "x": 270, "y": 80, "target": "{{device}}", "text": "PING {{run.id}}",
  "reply": { "bind": "0.0.0.0:0", "mode": "contains", "pattern": "PONG", "timeout_ms": 1000, "variable": "pong" } }
```

### Publication MQTT {#node-mqtt}

Se connecte à un broker MQTT, publie un message et se déconnecte. La connexion est
MQTT 3.1.1 sur TCP simple, avec une session propre et sans nom d'utilisateur ni mot
de passe. La connexion, la publication et l'accusé de réception du broker doivent
tous se produire en 15 secondes.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.broker]] | `host` | Le nom d'hôte ou l'adresse du broker | `127.0.0.1` | Oui |
| [[ui:exp.port]] | `port` | | 1883 ; 1–65 535 | Non |
| [[ui:exp.topic]] | `topic` | Aucun joker (`+`, `#`) | `lab/test` | Oui |
| [[ui:exp.payload]] | `payload` | Le message, en texte | `hello` | Oui |
| QoS | `qos` | 0, 1 ou 2 | 0 | Non |
| [[ui:exp.retain]] | `retain` | `true` : le broker le conserve comme valeur du topic | `false` | Non |

Les six clés sont obligatoires dans un fichier. L'étape échoue quand le broker ne
peut pas être atteint ou refuse la connexion ou le message. Sorties :
[[ui:exp.outputPort]]. Réglages : Réessai, Répétition.

```json
{ "id": "light", "type": "mqtt", "x": 270, "y": 80, "host": "{{broker}}", "port": 1883,
  "topic": "lab/light/1/set", "payload": "on", "qos": 1, "retain": false }
```

Voir aussi [MQTT](../protocols/mqtt.md).

### Connexion WebSocket {#node-ws_connect}

Ouvre un WebSocket pour le reste de l'exécution, ou jusqu'à un
[[[ui:exp.node.ws_close]]](#node-ws_close). Ce qui arrive à partir de là est conservé
pour les étapes [[[ui:exp.node.wait_ws]]](#node-wait_ws) qui s'y rapportent. L'URL et
les en-têtes sont résolus quand l'étape s'exécute, si bien qu'un jeton extrait plus
tôt peut s'y trouver. Exécutée de nouveau — dans une [[ui:exp.node.loop]] — elle
ferme d'abord sa connexion précédente et en ouvre une nouvelle. Quand l'exécution se
termine, de quelque façon que ce soit, ses connexions sont fermées par une trame de
fermeture.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| URL | `url` | Une URL `ws://` ou `wss://` | `ws://127.0.0.1:9001/` | Oui |
| [[ui:exp.headers]] | `headers` | `[[name, value], …]` envoyées avec la requête de mise à niveau | aucune | Oui, noms et valeurs |
| [[ui:exp.wsProtocols]] | `protocols` | Sous-protocoles à proposer, par ordre de préférence ; le serveur en choisit un | aucun | Non |
| [[ui:common.timeoutMs]] | `timeout_ms` | Pour la connexion et la mise à niveau | 5 000 (10 000 si absent) ; 1–120 000 | Non |

`wss://` approuve les mêmes certificats que `https://`. L'étape échoue quand la
connexion ou la mise à niveau échoue ; le statut du serveur est dans la raison.
Sorties : [[ui:exp.outputPort]]. Réglages : Réessai (pas Répétition).

```json
{ "id": "socket", "type": "ws_connect", "x": 270, "y": 80, "url": "ws://127.0.0.1:9001/chat",
  "headers": [["Authorization", "Bearer {{token}}"]], "protocols": ["chat.v1"], "timeout_ms": 5000 }
```

Voir aussi [WebSocket](../protocols/websocket.md).

### Envoi WebSocket {#node-ws_send}

Envoie un message sur la connexion qu'un [[ui:exp.node.ws_connect]] a ouverte.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | L'id d'un nœud [[ui:exp.node.ws_connect]] de cette expérience | le premier | Non |
| [[ui:exp.wsFormat]] | `binary` | [[ui:exp.wsText]] (`false`), ou [[ui:exp.wsBinary]] (`true`) : la charge utile est des octets écrits en hex, `de ad be ef` | `false` (aussi si absent) | Non |
| [[ui:exp.payload]] | `text` | Le message | `hello` ; 16 Mio au plus | Oui |

La connexion doit précéder l'envoi sur son chemin ; un envoi dont la connexion n'est
pas ouverte échoue. Les réponses comptent à partir du moment où le message est écrit.
Sorties : [[ui:exp.outputPort]]. Réglages : Réessai, Répétition.

```json
{ "id": "hello", "type": "ws_send", "x": 500, "y": 80, "connection": "socket",
  "text": "{\"type\":\"ping\",\"id\":\"{{uuid}}\"}", "binary": false }
```

### Fermeture WebSocket {#node-ws_close}

Ferme une connexion par une poignée de main de fermeture. La chronologie dit qui l'a
fermée : cette étape, le serveur plus tôt (avec son code), ou une connexion qui
s'était rompue.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | L'id d'un nœud [[ui:exp.node.ws_connect]] | le premier | Non |
| [[ui:field.code]] | `code` | 1000 (normal), ou 3000–4999 pour un code propre à une application | 1000 (aussi si absent) | Non |
| [[ui:field.reason]] | `reason` | Envoyée avec le code | vide ; 123 octets au plus, après les modèles | Oui |

Sorties : [[ui:exp.outputPort]]. Aucun réglage.

```json
{ "id": "bye", "type": "ws_close", "x": 960, "y": 80, "connection": "socket", "code": 1000, "reason": "done" }
```

### Marqueur de journal {#node-log}

Écrit une ligne dans la chronologie et le rapport — un point de contrôle, ou les
valeurs qu'une exécution a atteintes.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.logMessage]] | `message` | Le texte | `Check point` ; 10 000 caractères au plus | Oui |

Sorties : [[ui:exp.outputPort]]. Aucun réglage.

```json
{ "id": "ready", "type": "log", "x": 500, "y": 80, "message": "device {{device}} ready" }
```

## Attentes {#waits}

Le groupe [[ui:exp.group.observe]] : des nœuds qui attendent que quelque chose
arrive. Ils partagent ces règles :

- **Ils écoutent dès le début de l'exécution.** Le port d'une attente ou son
  abonnement au broker est ouvert avant la première étape, si bien qu'un appareil qui
  répond plus vite que le début de l'étape suivante n'est pas manqué. Deux attentes
  sur la même adresse partagent un socket.
- **Ils comptent à partir de la dernière action de leur branche.** Un message arrivé
  avant la dernière requête de la branche n'est pas une réponse à celle-ci ; avant
  toute action, tout ce qui est arrivé depuis le début de l'exécution compte.
- **Le premier message correspondant est pris.** Un message pris par une attente
  n'est pas vu par une autre.
- **[[ui:exp.portMatched]] ou [[ui:exp.portTimeout]].** Sur une correspondance, le
  message est stocké dans la variable de l'attente et le flux suit
  [[ui:exp.portMatched]]. Quand le temps s'écoule, il suit [[ui:exp.portTimeout]] si
  cette sortie a un fil ; sinon l'étape échoue, en indiquant combien d'autres
  messages sont arrivés.
- Chaque socket conserve les 1 024 derniers messages (et 64 Mio) ; les plus anciens
  sont abandonnés, et un délai d'attente indique combien l'ont été.
- [[ui:exp.listenNow]] écoute avec cette seule étape, à partir de maintenant.

Sorties : [[ui:exp.portMatched]] (obligatoire), [[ui:exp.portTimeout]] (facultatif).
Réglages : Réessai.

### Correspondance de charge utile {#payload-matching}

[[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_ws]] et une
réponse UDP choisissent l'aspect que la charge utile doit avoir :

| Option | Dans le fichier | Correspond quand la charge utile |
| --- | --- | --- |
| [[ui:exp.mode.any]] | `any` | est n'importe quoi |
| [[ui:exp.mode.contains]] | `contains` | lue comme du texte UTF-8, contient le motif (sensible à la casse) |
| [[ui:exp.mode.regex]] | `regex` | lue comme du texte UTF-8, correspond à l'expression régulière |
| [[ui:exp.mode.hex]] | `hex` | contient les octets, écrits en paires hex : `de ad be ef`, `deadbeef`, `0xde,0xad`, `DE:AD` |

Le message correspondant est stocké comme un objet. Les étapes ultérieures lisent ses
champs comme `{{reply.text}}` (avec le nom de la variable à la place de `reply`) :

| Champ | Quoi |
| --- | --- |
| `text` | La charge utile en texte |
| `hex`, `bytes` | La charge utile en hex (ses 1 024 premiers octets), et sa taille en octets |
| `match` | Ce qui a correspondu : le texte, le premier groupe de l'expression régulière (ou toute la correspondance), ou les octets |
| `from` | L'`IP:port` de l'expéditeur |
| `ms` | Millisecondes depuis la dernière action de la branche (ou le début de l'exécution) jusqu'au message |
| `topic` | [[ui:exp.node.wait_mqtt]] : le topic sur lequel il a été publié |
| `json`, `kind` | [[ui:exp.node.wait_ws]] : le message analysé comme JSON (`null` quand il ne l'est pas), et `text` ou `binary` |

### Attendre OSC {#node-wait_osc}

Attend un message OSC dont l'adresse correspond à un motif et dont les arguments
remplissent toutes les règles. Dans un bundle, le premier message qui correspond est
celui qui est pris.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | `IP:port` sur lequel écouter ; `0.0.0.0` pour toutes les cartes réseau | `127.0.0.1:9001` | Non |
| [[ui:exp.addressPattern]] | `address` | `*` n'importe quels caractères, `?` un seul, `[0-9]` un ensemble (`[!0-9]` en dehors), `{ping,pong}` l'un ou l'autre ; les jokers restent dans un même segment `/` | `/pong` ; 512 caractères au plus | Oui |
| [[ui:exp.argRules]] | `args` | `[{ "index", "op", "value" }, …]` : l'argument `index` comparé à `value` par `op` ([Comparaisons](#comparisons)) ; toutes doivent tenir | aucune ; 16 au plus, index 0–63 | Valeurs : oui |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (aussi si absent) ; 1–120 000 | Non |
| [[ui:exp.replyVariable]] | `variable` | Où le message est stocké | `reply` (aussi si absent) | Non |

Un argument se compare comme du texte : les nombres tels qu'écrits, les chaînes sans
guillemets, `true`/`false`, un blob en hex. Une règle sur un argument que le message
n'a pas ne tient pas. Le message stocké a `address`, `args` (`{{reply.args[0]}}`),
`from` et `ms`.

```json
{ "id": "status", "type": "wait_osc", "x": 500, "y": 80, "bind": "0.0.0.0:9001", "address": "/status",
  "args": [{ "index": 0, "op": "eq", "value": "ready" }], "timeout_ms": 5000, "variable": "reply" }
```

### Attendre UDP {#node-wait_udp}

Attend un datagramme UDP dont la charge utile correspond.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | `IP:port` sur lequel écouter | `127.0.0.1:9001` | Non |
| [[ui:exp.waitMode]] | `mode` | Voir [Correspondance de charge utile](#payload-matching) | `contains` (`any` si absent) | Non |
| [[ui:field.pattern]] | `pattern` | Ce que la charge utile doit contenir ou auquel elle doit correspondre | `pong` ; obligatoire sauf pour `any` | Oui |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (aussi si absent) ; 1–120 000 | Non |
| [[ui:exp.replyVariable]] | `variable` | | `reply` (aussi si absent) | Non |

```json
{ "id": "ready", "type": "wait_udp", "x": 500, "y": 80, "bind": "0.0.0.0:9002", "mode": "contains",
  "pattern": "READY", "timeout_ms": 5000, "variable": "reply" }
```

### Attendre MQTT {#node-wait_mqtt}

Attend un message publié sur un topic d'un broker, dont la charge utile correspond.
L'exécution se connecte et s'abonne avant sa première étape. Les messages retenus que
le broker rejoue à l'abonnement sont ignorés : seul ce qui est publié après le début
de l'exécution compte.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.broker]] | `host` | Le broker | `127.0.0.1` | Paramètres uniquement |
| [[ui:exp.port]] | `port` | | 1883 ; 1–65 535 | Non |
| [[ui:exp.topicFilter]] | `topic` | Un filtre : `+` est un niveau quelconque, `#` tout ce qui est en dessous (en dernier seulement) | `lab/#` | Paramètres uniquement |
| [[ui:exp.waitMode]] | `mode` | Voir [Correspondance de charge utile](#payload-matching) | `any` (aussi si absent) | Non |
| [[ui:field.pattern]] | `pattern` | | vide ; obligatoire sauf pour `any` | Oui |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (aussi si absent) ; 1–120 000 | Non |
| [[ui:exp.replyVariable]] | `variable` | | `reply` (aussi si absent) | Non |

```json
{ "id": "state", "type": "wait_mqtt", "x": 500, "y": 80, "host": "{{broker}}", "port": 1883,
  "topic": "lab/+/state", "mode": "contains", "pattern": "on", "timeout_ms": 5000, "variable": "reply" }
```

### Attendre une requête HTTP {#node-wait_http}

Attend une requête HTTP — un webhook, un rappel — adressée à
l'[[[ui:exp.node.emulator]]](#node-emulator) de l'exécution sur cette adresse, ou,
quand l'exécution n'a pas d'émulateur HTTP à cet endroit, à un écouteur propre à
l'exécution qui répond à chaque requête par 204. La requête doit correspondre à la
méthode, au chemin et à chaque condition.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | `IP:port` | `127.0.0.1:18080` — là où écoute un nouvel [[ui:exp.node.emulator]] | Non |
| [[ui:exp.method]] | `method` | Une méthode, ou [[ui:emu.methodAny]] (`ANY`) ; GET prend aussi HEAD | `ANY` (aussi si absent) | Non |
| [[ui:exp.path]] | `path` | `/hooks/:name` nomme un segment (`{{request.params.name}}`) ; un `/*` final prend le reste | `/*` (aussi si absent) ; 512 caractères au plus | Oui |
| [[ui:emu.conditions]] | `when` | `[{ "on", "name", "op", "value" }, …]` sur un `header`, un paramètre `query`, le `body` ou un chemin `json` ; chacune doit tenir | aucune ; 16 au plus | Oui, noms et valeurs |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 5 000 (2 000 si absent) ; 1–120 000 | Non |
| [[ui:exp.replyVariable]] | `variable` | | `request` (aussi si absent) | Non |

La requête stockée a `method`, `path`, `query`, `headers`, `body`, `json`, `params`,
`from` et `ms` : `{{request.json.event}}`, `{{request.headers.x-key}}`.

```json
{ "id": "hook", "type": "wait_http", "x": 500, "y": 80, "bind": "127.0.0.1:18081", "method": "POST",
  "path": "/hooks/:name", "when": [{ "on": "json", "name": "$.event", "op": "eq", "value": "deploy" }],
  "timeout_ms": 5000, "variable": "request" }
```

### Attendre WebSocket {#node-wait_ws}

Attend un message sur la connexion qu'un [[ui:exp.node.ws_connect]] a ouverte, dont
la charge utile correspond. Les messages depuis la dernière action de la branche
comptent — la connexion elle-même, un envoi, ou toute autre requête.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | L'id d'un nœud [[ui:exp.node.ws_connect]] | le premier | Non |
| [[ui:exp.waitMode]] | `mode` | Voir [Correspondance de charge utile](#payload-matching) | `any` (aussi si absent) | Non |
| [[ui:field.pattern]] | `pattern` | | vide ; obligatoire sauf pour `any` | Oui |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (aussi si absent) ; 1–120 000 | Non |
| [[ui:exp.replyVariable]] | `variable` | | `reply` (aussi si absent) | Non |

Un message JSON est lisible champ par champ : `{{reply.json.type}}`. La connexion
doit précéder l'attente sur son chemin.

```json
{ "id": "pong", "type": "wait_ws", "x": 730, "y": 80, "connection": "socket", "mode": "contains",
  "pattern": "pong", "timeout_ms": 3000, "variable": "reply" }
```

## Émulation {#emulation}

### Émulateur {#node-emulator}

Joue une dépendance — une API HTTP, un appareil OSC, UDP ou TCP, un broker MQTT —
pendant toute l'exécution. Il s'ouvre avant la première étape et répond jusqu'à la
fin de l'exécution ; dans le flux, l'étape est franchie aussitôt. Ce qu'il a reçu est
compté, règle par règle, dans le rapport de l'exécution.

| Champ | Dans le fichier | Quoi | Défaut |
| --- | --- | --- | --- |
| [[ui:emu.edit]] | `emulator` | L'émulateur : `name`, `bind` (`IP:port`), `protocol` (`http`, `osc`, `udp`, `tcp`, `mqtt`), ses routes ou règles, et un `outage` facultatif | Une API HTTP nommée *API* sur `127.0.0.1:18080` qui répond à `/health` |

Les propriétés montrent ce qu'il joue en une ligne. [[ui:emu.edit]] ouvre ses règles,
le même éditeur que l'écran [[[ui:nav.emulators]]](../tools/emulators.md) ;
[[ui:emu.toLibrary]] en garde une copie dans la bibliothèque d'émulateurs, et
[[ui:emu.fromLibrary]] remplace celui-ci par une copie qui en vient. Les règles —
routes, réponses, défaillances, pannes — y sont décrites.

- Un émulateur HTTP est aussi ce que lit un [[[ui:exp.node.wait_http]]](#node-wait_http)
  sur son adresse ; un émulateur OSC ou UDP partage son port avec les attentes de
  l'exécution à cet endroit.
- Deux émulateurs d'un même transport ne peuvent pas partager un port dans une
  exécution.
- [[[ui:exp.node.emulator_state]]](#node-emulator_state) le met en panne et le remet
  en service.

Sorties : [[ui:exp.outputPort]]. Aucun réglage.

```json
{ "id": "api", "type": "emulator", "x": 270, "y": 80,
  "emulator": { "name": "Orders API", "bind": "127.0.0.1:18080", "protocol": "http",
    "routes": [{ "method": "GET", "path": "/orders/:id", "order": "sequence",
                 "responses": [{ "status": 503 }, { "status": 200, "body": "{\"id\":\"{{request.params.id}}\"}" }] }] } }
```

## Défaillances {#faults}

Des nœuds qui cassent des choses à la demande. Une branche de nœuds
[[ui:exp.node.delay]] et ceux-ci à côté du trafic se lit comme un calendrier ;
[Défaillances sur un calendrier](faults.md) montre comment.

### Dégradation {#node-impairment}

Un relais de dégradation pour toute l'exécution : le système testé envoie à (ou se
connecte à) [[ui:exp.relayListen]] au lieu de sa vraie cible ; le relais transmet à
[[ui:exp.relayTarget]], et les réponses reviennent de la même façon, dégradées par le
profil. Il s'ouvre avant la première étape et se ferme quand l'exécution se termine,
de quelque façon que ce soit, si bien que rien ne reste dégradé ; dans le flux,
l'étape est franchie aussitôt. Chaque décision est tirée de la graine de l'exécution :
la même graine et le même trafic connaissent le même sort.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.relayListen]] | `listen` | `IP:port` auquel le système testé envoie | `127.0.0.1:9010` | Paramètres uniquement |
| [[ui:exp.relayTarget]] | `target` | `IP:port` de la vraie destination, ou `host:port` — un nom d'hôte est résolu au démarrage de l'exécution, et un nom introuvable arrête l'exécution sur ce nœud | `127.0.0.1:9000` | Paramètres uniquement |
| [[ui:ns.protocol]] | `protocol` | UDP (`udp`) : chaque datagramme connaît son propre sort ; TCP (`tcp`) : chaque connexion est jointe à une connexion à elle vers la cible, et les deux flux sont dégradés | UDP (`udp` si absent) | Non |
| [[ui:ns.preset]] et les valeurs en dessous | `profile` | Ce que le relais fait au trafic — voir [Le profil](#impair-profile) | [[ui:ns.preset.lan]] (aucune dégradation si absent) | Non |

L'adresse d'écoute d'un relais ne peut pas être un autre socket de l'exécution, et les
relais ne peuvent pas transmettre les uns vers les autres en cercle. Une cible donnée
par un nom est suivie une fois résolue, si bien qu'un cercle passant par un nom
arrête l'exécution à son démarrage. Le rapport compte chaque phase d'un relais
séparément.

Sorties : [[ui:exp.outputPort]]. Aucun réglage.

```json
{ "id": "relay", "type": "impairment", "x": 270, "y": 80, "listen": "127.0.0.1:9010", "target": "{{device}}",
  "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } }
```

#### Le profil {#impair-profile}

Un préréglage — [[ui:ns.preset.lan]], [[ui:ns.preset.wifi]], [[ui:ns.preset.4g]],
[[ui:ns.preset.satellite]], [[ui:ns.preset.intermittent]], [[ui:ns.preset.offline]] —
remplit toutes les valeurs ; modifiez ensuite celles que vous voulez. Un relais ne lit
que les valeurs de son protocole ; dans un fichier, toute clé peut être omise (zéro,
désactivé).

| Champ | Dans le fichier | Quoi | Limites | Protocole |
| --- | --- | --- | --- | --- |
| — | `name` | Un libellé pour la chronologie et le rapport : la clé d'un préréglage (`lan`, `wifi`, `4g`, `satellite`, `intermittent`, `offline`) ou le vôtre | 60 caractères au plus | les deux |
| [[ui:ns.offline]] | `offline` | Rien ne passe | `true` / `false` | les deux |
| [[ui:ns.latency]] | `latency_ms` | Délai ajouté à chaque paquet, ou morceau d'un flux | 0–60 000 (le curseur va jusqu'à 1 000) | les deux |
| [[ui:ns.jitter]] | `jitter_ms` | Un délai supplémentaire aléatoire jusqu'à autant ; un flux TCP reste dans l'ordre | 0–60 000 (le curseur va jusqu'à 500) | les deux |
| [[ui:ns.rate]] | `rate_kbps` | Une limite de bande passante, 0 pour aucune. UDP : au-delà d'une seconde de file, les datagrammes sont abandonnés comme bridés ; TCP : l'émetteur est ralenti, rien n'est abandonné | 0, ou 8–10 000 000 | les deux |
| [[ui:ns.loss]] | `loss` | La chance qu'un datagramme soit abandonné | 0–1 (le curseur montre des %) | UDP |
| [[ui:ns.burst]], [[ui:ns.burstLength]] | `burst_start`, `burst_length` | La chance qu'une rafale de pertes commence, et combien de datagrammes elle dure en moyenne | 0–1 ; 1–1 000 quand les rafales sont actives | UDP |
| [[ui:ns.duplicate]] | `duplicate` | La chance qu'un datagramme soit envoyé deux fois | 0–1 | UDP |
| [[ui:ns.corrupt]] | `corrupt` | La chance qu'un bit d'un datagramme soit inversé | 0–1 | UDP |
| [[ui:ns.reorder]] | `reorder` | La chance qu'un datagramme soit retenu, si bien que les suivants le dépassent | 0–1 | UDP |
| [[ui:ns.reset]] | `reset` | La chance qu'un morceau d'un flux réinitialise plutôt sa connexion — les deux côtés reçoivent une réinitialisation | 0–1 | TCP |
| [[ui:ns.stall]] | `stall` | La chance qu'un morceau laisse sa connexion semi-ouverte : plus rien ne passe dans un sens ni dans l'autre, et aucun côté n'est prévenu | 0–1 | TCP |

Pour en savoir plus sur les relais, les préréglages et ce qu'ils modélisent, voir la
page [[[ui:nav.netsim]]](../tools/impairment.md).

### Changer la dégradation {#node-impairment_change}

Fait passer l'un des nœuds [[ui:exp.node.impairment]] de l'exécution à un autre profil
à partir de cette étape, sans abandonner son port. La phase écoulée est close et
comptée dans le rapport.

| Champ | Dans le fichier | Quoi | Défaut | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.relay]] | `relay` | L'id d'un nœud [[ui:exp.node.impairment]] de cette expérience | le premier | Non |
| [[ui:ns.preset]] et les valeurs en dessous | `profile` | Avec quoi il dégrade à partir de maintenant — voir [Le profil](#impair-profile) ; le relais lit les valeurs de son propre protocole | [[ui:ns.preset.offline]] (aucune dégradation si absent) | Non |

L'étape échoue si le relais ne tourne pas — il a échoué à relayer, par exemple.
Sorties : [[ui:exp.outputPort]]. Aucun réglage.

```json
{ "id": "cut", "type": "impairment_change", "x": 730, "y": 200, "relay": "relay",
  "profile": { "name": "offline", "offline": true } }
```

### Émulateur en panne et en service {#node-emulator_state}

Met en panne l'un des émulateurs de l'exécution, ou le remet en service. Pendant la
panne, un émulateur HTTP répond comme l'indique [[ui:exp.downFault]] ; un appareil TCP
et un broker MQTT coupent leurs connexions et refusent les nouvelles ; les appareils
OSC et UDP ne répondent rien. De nouveau en service, l'émulateur suit son propre
calendrier de pannes, s'il en a un.

| Champ | Dans le fichier | Quoi | Défaut |
| --- | --- | --- | --- |
| [[ui:exp.emulatorNode]] | `emulator` | L'id d'un nœud [[ui:exp.node.emulator]] de cette expérience | le premier |
| [[ui:exp.emulatorDownState]] | `down` | [[ui:exp.emulatorGoesDown]] (`true`) ou [[ui:exp.emulatorComesUp]] (`false`) | en panne (`false` si absent) |
| [[ui:exp.downFault]] | `fault` | HTTP uniquement : [[ui:emu.outageFault.unavailable]] (`unavailable`), [[ui:emu.outageFault.reset]] (`reset` : la connexion est fermée sans réponse) ou [[ui:emu.outageFault.timeout]] (`timeout` : la requête est retenue jusqu'à ce que le client abandonne, 120 s au plus) | `unavailable` (aussi si absent) |

Sorties : [[ui:exp.outputPort]]. Aucun réglage. Aucun champ n'est un modèle.

```json
{ "id": "down", "type": "emulator_state", "x": 500, "y": 200, "emulator": "api", "down": true, "fault": "unavailable" }
```

## Données {#data}

### Extraire une valeur {#node-extract}

Enregistre une partie de la dernière réponse HTTP sur son chemin comme variable, pour
les champs ultérieurs (`{{token}}`), les vérifications et les bifurcations. Une
requête HTTP doit le précéder sur chaque chemin. Cliquer sur une valeur dans une
réponse de [[ui:exp.sendNow]] en ajoute un pour vous.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.variable]] | `variable` | Le nom : lettres, chiffres et `_`, ne commençant pas par un chiffre, ni un mot réservé, ni le nom d'un paramètre | `token` | Non |
| [[ui:exp.extractFrom]] | `from` | [[ui:exp.from.json]] (`json`), [[ui:exp.from.header]] (`header`), [[ui:exp.from.status]] (`status`), [[ui:exp.from.body]] (`body`) ou [[ui:exp.from.regex]] (`regex`) | `json` | Non |
| [[ui:exp.jsonPath]], [[ui:exp.headerName]] ou [[ui:exp.pattern]] | `expr` | Un chemin JSON (`$.data.token`, `$.items[0]`, `$["first name"]`), un nom d'en-tête (casse indifférente), ou une expression régulière — son premier groupe, ou toute la correspondance | `$.token` ; inutilisé pour le statut et le corps | Non |

L'étape échoue quand il n'y a rien à prendre : le corps n'est pas du JSON, le chemin
ou l'en-tête manque, l'expression ne correspond pas, ou — pour un champ JSON ou le
corps entier — le corps était plus long que les 256 Kio conservés. Un statut est
stocké comme un nombre ; le reste comme du texte, ou comme la valeur JSON trouvée.
Sorties : [[ui:exp.outputPort]]. Aucun réglage.

```json
{ "id": "token", "type": "extract", "x": 500, "y": 80, "variable": "token", "from": "json", "expr": "$.data.token" }
```

Pour en savoir plus sur les variables, voir [Données et modèles](data.md).

## Vérifications {#checks}

Une vérification réussit, ou fait échouer l'exécution. Les quatre vérifications de
réponse lisent la dernière réponse HTTP sur leur chemin, si bien qu'une requête HTTP —
pas une sous charge — doit les précéder sur chaque chemin.

### Statut HTTP {#node-assert_status}

Réussit quand le statut de la dernière réponse est exactement celui donné.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.expectedStatus]] | `status` | | 200 ; 100–599 | Non |

Sorties : [[ui:exp.outputPort]]. Aucun réglage.

```json
{ "id": "ok", "type": "assert_status", "x": 500, "y": 80, "status": 200 }
```

### Texte de la réponse {#node-assert_body}

Réussit quand le corps de la dernière réponse contient le texte, exactement (casse
comprise). Seuls les 256 premiers Kio d'un corps sont conservés : un texte
introuvable dans un corps qui a été coupé échoue avec cette raison.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.contains]] | `contains` | | `ok` ; obligatoire | Oui |

Sorties : [[ui:exp.outputPort]]. Aucun réglage.

```json
{ "id": "ready", "type": "assert_body", "x": 500, "y": 80, "contains": "ready" }
```

### En-tête de la réponse {#node-assert_header}

Réussit quand la dernière réponse a l'en-tête et que sa valeur contient le texte. Le
nom de l'en-tête est comparé sans tenir compte de la casse ; la valeur exactement.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.headerName]] | `name` | | `content-type` ; obligatoire | Oui |
| [[ui:exp.contains]] | `contains` | Ce que sa valeur doit contenir ; vide : l'en-tête doit seulement être là | `application/json` | Oui |

Sorties : [[ui:exp.outputPort]]. Aucun réglage.

```json
{ "id": "json", "type": "assert_header", "x": 500, "y": 80, "name": "Content-Type", "contains": "json" }
```

### Temps de réponse {#node-assert_latency}

Réussit quand la dernière réponse a pris au plus ce temps, de l'envoi de la requête à
la fin de son corps.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.maxLatency]] | `max_ms` | | 1 000 ; 1–120 000 | Non |

Sorties : [[ui:exp.outputPort]]. Aucun réglage.

```json
{ "id": "fast", "type": "assert_latency", "x": 500, "y": 80, "max_ms": 250 }
```

### Vérifier une valeur {#node-assert_value}

Compare une valeur — le plus souvent une variable, écrite comme un modèle — avec une
valeur attendue, et réussit quand la comparaison tient.

| Champ | Dans le fichier | Quoi | Défaut | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.value]] | `value` | Ce qui est comparé : `{{token}}`, `{{reply.args[0]}}` | `{{token}}` | Oui |
| [[ui:exp.operator]] | `op` | Voir [Comparaisons](#comparisons) | [[ui:exp.op.not_empty]] | Non |
| [[ui:exp.expected]] | `expected` | Inutilisé par [[ui:exp.op.empty]] et [[ui:exp.op.not_empty]] | vide (aussi si absent) | Oui |

Sorties : [[ui:exp.outputPort]]. Aucun réglage.

```json
{ "id": "state", "type": "assert_value", "x": 730, "y": 80, "value": "{{state}}", "op": "eq", "expected": "ready" }
```

### Comparaisons {#comparisons}

[[ui:exp.node.assert_value]], [[ui:exp.node.branch_value]], la condition de sortie
d'une [[ui:exp.node.loop]], les règles d'arguments OSC et les conditions HTTP
comparent de la même façon :

| Option | Dans le fichier | Tient quand la valeur |
| --- | --- | --- |
| [[ui:exp.op.eq]] | `eq` | est égale à la valeur attendue — comme nombres quand les deux sont des nombres (`200` = `200.0`), sinon comme texte exact |
| [[ui:exp.op.ne]] | `ne` | n'est pas égale, selon la même règle |
| [[ui:exp.op.lt]], [[ui:exp.op.le]], [[ui:exp.op.gt]], [[ui:exp.op.ge]] | `lt`, `le`, `gt`, `ge` | est inférieure, au plus, supérieure, au moins — les deux doivent être des nombres : sinon une vérification, une bifurcation ou une [[ui:exp.node.loop]] fait échouer l'étape, et une règle d'argument ou une condition HTTP ne tient pas |
| [[ui:exp.op.contains]] | `contains` | contient le texte attendu |
| [[ui:exp.op.matches]] | `matches` | correspond à l'expression régulière attendue |
| [[ui:exp.op.empty]], [[ui:exp.op.not_empty]] | `empty`, `not_empty` | est vide (les espaces comptent comme vide) / ne l'est pas |

## Flux {#flow}

Des nœuds qui décident où va l'exécution. Pour en savoir plus sur les branches, les
jointures et les boucles, voir [Flux](flow.md).

### Début {#node-start}

Là où l'exécution commence ; chaque expérience en a exactement un. Il n'a aucune
entrée ni aucun champ. La première ligne de la chronologie donne la graine de
l'exécution.

Sorties : [[ui:exp.outputPort]], obligatoire. Plusieurs fils partant de là démarrent
des branches parallèles en même temps.

```json
{ "id": "start", "type": "start", "x": 40, "y": 80 }
```

### Fin {#node-end}

Là où l'exécution se termine ; chaque expérience en a exactement une, et elle n'a
aucune sortie. Plusieurs branches peuvent y mener : l'exécution passe une fois, après
la fin de la dernière branche, et seulement si aucune n'a échoué. Une exécution qui
n'atteint jamais [[ui:exp.node.end]] échoue.

```json
{ "id": "end", "type": "end", "x": 960, "y": 80 }
```

### Délai {#node-delay}

Attend un temps fixe avant l'étape suivante.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.delayMs]] | `ms` | | 300 ; 0–60 000 | Non |

Sorties : [[ui:exp.outputPort]]. Aucun réglage. Pour des attentes plus longues,
mettez-en plusieurs à la suite ou dans une [[ui:exp.node.loop]].

```json
{ "id": "pause", "type": "delay", "x": 500, "y": 80, "ms": 500 }
```

### Bifurcation sur statut {#node-branch_status}

Choisit [[ui:exp.yes]] quand la dernière réponse HTTP a ce statut, sinon
[[ui:exp.no]]. Une requête HTTP doit la précéder sur chaque chemin.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.expectedStatus]] | `status` | | 200 ; 100–599 | Non |

Sorties : [[ui:exp.yes]] et [[ui:exp.no]], toutes deux obligatoires. Aucun réglage.

```json
{ "id": "branch", "type": "branch_status", "x": 500, "y": 80, "status": 200 }
```

### Bifurcation sur valeur {#node-branch_value}

Choisit [[ui:exp.yes]] quand une comparaison tient, sinon [[ui:exp.no]]. Ses champs et
ses [comparaisons](#comparisons) sont ceux de
[[[ui:exp.node.assert_value]]](#node-assert_value) ; une comparaison qui ne peut pas
être faite (`lt` sur du texte) fait échouer l'étape.

| Champ | Dans le fichier | Quoi | Défaut | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.value]] | `value` | Ce qui est comparé | `{{token}}` | Oui |
| [[ui:exp.operator]] | `op` | | [[ui:exp.op.eq]] | Non |
| [[ui:exp.expected]] | `expected` | | vide (aussi si absent) | Oui |

Sorties : [[ui:exp.yes]] et [[ui:exp.no]], toutes deux obligatoires. Aucun réglage.

```json
{ "id": "ok", "type": "branch_value", "x": 730, "y": 80, "value": "{{reply.args[0]}}", "op": "eq", "expected": "ok" }
```

### Branche parallèle {#node-fork}

Exécute ce qui suit [[ui:exp.branch1]] et [[ui:exp.branch2]] en même temps, chaque
branche avec sa propre copie des variables. Chaque sortie peut avoir plus de fils
pour plus de branches. Aucun champ.

Sorties : [[ui:exp.branch1]] et [[ui:exp.branch2]], toutes deux obligatoires.

```json
{ "id": "split", "type": "fork", "x": 270, "y": 80 }
```

### Joindre les branches {#node-join}

Attend que chaque fil qui y mène ait été atteint, puis continue une fois, avec les
variables des branches fusionnées — quand deux branches définissent la même variable,
celle dont le fil vient plus tard dans le fichier l'emporte — et la dernière réponse
HTTP de la dernière d'entre elles qui en avait une. Aucun champ.

Seules les branches qui s'exécutent toutes se rejoignent ici : un [[ui:exp.node.join]]
derrière une [[ui:exp.node.branch_status]], dont [[ui:exp.yes]] et [[ui:exp.no]] ne se
produisent jamais toutes les deux, ne continue jamais ; quand aucun autre chemin
n'atteint [[ui:exp.node.end]], l'exécution échoue sur ce nœud, en indiquant combien de
branches elle attendait encore.

Sorties : [[ui:exp.outputPort]], obligatoire.

Tout autre nœud avec plusieurs fils qui y mènent s'exécute une fois par arrivée.

```json
{ "id": "joined", "type": "join", "x": 730, "y": 80 }
```

### Boucle {#node-loop}

Exécute les étapes sur [[ui:exp.portBody]] — qui y ramènent — encore et encore : au
plus un certain nombre de fois, et, quand elle a une condition de sortie, jusqu'à ce
que celle-ci tienne.

| Champ | Dans le fichier | Quoi | Défaut et limites | Modèles |
| --- | --- | --- | --- | --- |
| [[ui:exp.loopMax]] | `max` | Itérations au plus | 5 ; 1–1 000 | Non |
| [[ui:exp.loopUntilOn]] | `until` | Condition de sortie facultative `{ "value", "op", "expected" }`, comme [[[ui:exp.node.assert_value]]](#node-assert_value) | désactivée | Valeur et attendue : oui |

- Le corps s'exécute toujours au moins une fois. La condition de sortie est lue après
  chaque itération, si bien que le corps peut définir ce qu'elle teste.
- [[ui:exp.portDone]] suit quand la condition tient — ou, sans condition, après la
  dernière itération.
- [[ui:exp.portLimit]] suit quand les itérations se sont épuisées avant que la
  condition ne tienne. Sans fil sur cette sortie, cela fait échouer l'étape.
- Dans le corps, `{{counter}}` est le numéro de l'itération.
- Un corps s'exécute comme une seule branche : chaque sortie à l'intérieur a un seul
  fil ; il ne contient ni [[ui:exp.node.start]], ni [[ui:exp.node.end]], ni
  [[ui:exp.node.fork]], ni [[ui:exp.node.join]], ni autre [[ui:exp.node.loop]] ; on n'y
  entre que par [[ui:exp.portBody]] ; et chaque fil y mène plus loin dans le corps ou
  revient à la boucle.

Sorties : [[ui:exp.portBody]] et [[ui:exp.portDone]] (obligatoires),
[[ui:exp.portLimit]] (facultative). Aucun réglage.

```json
{ "id": "poll", "type": "loop", "x": 270, "y": 80, "max": 10,
  "until": { "value": "{{status.args[0]}}", "op": "eq", "expected": "ready" } }
```

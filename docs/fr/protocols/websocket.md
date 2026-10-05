---
title: WebSocket
description: "Connectez-vous à un service ws:// ou wss:// avec les en-têtes et sous-protocoles qu’il attend, envoyez des messages texte ou binaires, et lisez chaque message dès qu’il arrive."
---

# WebSocket

L’écran [[ui:nav.ws]] est un client WebSocket : il ouvre une connexion vers un
service — avec les en-têtes et sous-protocoles que le service attend — envoie du texte
ou des octets, et liste chaque message qui va et vient, le plus récent en dernier. Utilisez-le pour
essayer une API en direct, une surface de contrôle ou un appareil qui parle WebSocket avant de le
scripter dans une expérience.

## Se connecter {#connect}

1. Ouvrez [[ui:nav.ws]].
2. Dans [[ui:field.url]], saisissez l’adresse : `ws://127.0.0.1:9001/` ou
   `wss://example.com/socket`.
3. Si le service les demande, saisissez [[ui:exp.wsProtocols]] et ajoutez
   des [[ui:http.headers]] (un en-tête `Authorization` avec un jeton, un cookie).
4. Appuyez sur [[ui:ws.connect]]. Pendant que la mise à niveau se fait, le bouton indique
   [[ui:ws.connecting]] ; une fois terminée, les champs se verrouillent et le bouton devient
   [[ui:ws.disconnect]].

| Champ | Quoi | Par défaut |
| --- | --- | --- |
| [[ui:field.url]] | `ws://` ou `wss://`, un hôte, un port facultatif (80 pour `ws`, 443 pour `wss`) et un chemin | `ws://127.0.0.1:9001/` |
| [[ui:exp.wsProtocols]] | Sous-protocoles à proposer, séparés par des virgules, par ordre de préférence ; le serveur en choisit un. Un nom n’a pas d’espaces, de virgules ni de barres obliques. | aucun |
| [[ui:http.headers]] | Des en-têtes supplémentaires pour la requête de mise à niveau ; [[ui:http.addHeader]] ajoute une ligne. Une ligne sans nom est omise. | aucun |

La connexion — la résolution du nom, la connexion TCP, TLS pour `wss://` et la
mise à niveau — a 10 secondes. L’URL est conservée quand vous changez d’écran et quand vous
redémarrez l’application ; les en-têtes et les sous-protocoles ne le sont pas.

Une fois connecté, le panneau montre :

| Élément | Quoi |
| --- | --- |
| [[ui:ws.state]] | [[ui:ws.open]], ou une fois terminé : fermé par vous ou par le serveur, avec le code de fermeture, ou [[ui:ws.lost]] quand la liaison s’est rompue sans fermeture |
| [[ui:field.reason]] | La raison donnée par le côté qui ferme, s’il y en a une |
| [[ui:ws.subprotocol]] | Le sous-protocole que le serveur a choisi, ou — |
| [[ui:ws.peer]] | L’`IP:port` du serveur |
| [[ui:ws.upgradeTime]] | Combien de temps la connexion et la mise à niveau ont pris, en millisecondes |

La connexion est une tâche : elle apparaît dans le bandeau de la console et peut y être arrêtée
aussi.

### Connexions sécurisées {#wss}

`wss://` fait confiance aux mêmes certificats que HTTPS sur ce système : un serveur dont
le certificat n’est pas de confiance pour ce système (auto-signé, expiré, un autre nom) est
refusé avec « A secure connection to … could not be made ». Il n’y a pas de réglage
pour ignorer la vérification.

## Envoyer un message {#send}

1. Sous [[ui:ws.message]], choisissez [[ui:exp.wsText]] ou [[ui:exp.wsBinary]].
2. Écrivez le message. Pour le binaire, écrivez les octets par paires de chiffres hex :
   `de ad be ef`.
3. Appuyez sur [[ui:common.send]], ou sur <kbd>Ctrl</kbd>+<kbd>Enter</kbd> dans le message.

Un message fait au plus 16 Mio (16 777 216 octets, comptés comme octets pour le binaire, pas
comme chiffres hex), la même limite que pour un message qui arrive et pour l’étape
[[ui:exp.node.ws_send]]. Un plus long est refusé avec `Too long: at most
16777216` avant que quoi que ce soit ne soit envoyé, et la connexion reste ouverte.

Quand un message texte est du JSON, [[ui:http.formatJson]] le met en forme avec des indentations
avant que vous ne l’envoyiez. Le message est conservé quand vous changez d’écran et quand vous
redémarrez l’application.

## Lire les messages {#messages}

[[ui:ws.messages]] liste ce qui a été reçu (↓) et envoyé (↑), le plus récent en dernier, avec
le temps, le début du message (300 caractères) et sa taille ; un message binaire
montre ses octets en hex et est marqué [[ui:ws.binary]]. Au-dessus de la liste
se trouvent combien ont été reçus et envoyés. La liste suit les nouveaux messages tant qu’elle est
défilée jusqu’au bout ; remontez et elle reste où vous êtes.

Cliquez sur un message pour le voir en entier sous la liste : JSON mis en forme, binaire en hex.
[[ui:ws.editHere]] le copie dans le champ du message, pour le renvoyer ou le modifier.
Un message très long est affiché en partie — texte jusqu’à 64 Kio, binaire jusqu’à
4096 octets — et ne peut alors pas être copié, puisqu’il serait envoyé coupé.

L’écran garde les 2000 derniers messages ; [[ui:common.clear]] vide la
liste. Si un service envoie plus vite que l’écran ne peut encaisser — plus de 2000 en un
dixième de seconde — les plus anciens de ceux-là sont laissés hors de la liste et comptés comme
« non affichés ». L’Inspecteur les a toujours tant que la capture est armée.

## Fermer {#close}

[[ui:ws.disconnect]] envoie une trame de fermeture avec le code 1000 (normal) et attend
jusqu’à 2 secondes la réponse du serveur avant de raccrocher. L’état indique alors
fermé avec 1000. Quand le serveur ferme, l’état montre son code et sa raison ;
quand la connexion se rompt sans trame de fermeture, il indique [[ui:ws.lost]] et
la console dit pourquoi.

Signal Lab répond lui-même aux pings du serveur ; les pings et les pongs ne sont pas
listés. Une connexion se termine aussi quand un message de plus de 16 Mio arrive, ou quand
en envoyer un prend plus de 10 secondes parce que le serveur a cessé de lire.
(En envoyer un de plus de 16 Mio vous-même est refusé et ne termine rien.)

## Dans l’Inspecteur {#inspector}

Quand la capture est armée, le trafic de la connexion apparaît avec le protocole `ws`
et la source `websocket` :

| Résumé | Quoi |
| --- | --- |
| `CONNECT ws://… (subprotocol)` | La connexion a été ouverte |
| `TEXT …` | Un message texte et son début |
| `BINARY n B …` | Un message binaire, sa taille et ses 16 premiers octets |
| `CLOSE code reason` | Une trame de fermeture, envoyée ou reçue |

Chaque trame de message garde ses octets. Quand le trafic est léger, chaque message est
capturé ; une connexion chargée est limitée à 200 trames par seconde, et la trame suivante
capturée indique combien ont été laissées de côté (`+n not shown`). Voir
[Inspecteur](../tools/inspector.md).

## Dans les expériences {#experiments}

Quatre étapes scriptent une conversation WebSocket. Une connexion est ouverte par une étape
et nommée par les autres :

| Étape | Ce qu’elle fait |
| --- | --- |
| [[ui:exp.node.ws_connect]] | Ouvre une connexion pour le reste de l’exécution. Son URL et ses en-têtes prennent des `{{templates}}`, si bien qu’un jeton extrait plus tôt peut y aller. [Détails](../experiments/nodes.md#node-ws_connect) |
| [[ui:exp.node.ws_send]] | Envoie un message texte ou binaire sur une connexion. [Détails](../experiments/nodes.md#node-ws_send) |
| [[ui:exp.node.wait_ws]] | Attend un message dont la charge utile correspond, comme le fait [[ui:exp.node.wait_udp]] ; les messages JSON peuvent être lus champ par champ ensuite. [Détails](../experiments/nodes.md#node-wait_ws) |
| [[ui:exp.node.ws_close]] | Ferme une connexion par une poignée de main de fermeture : code 1000, ou 3000–4999 pour une application, et une raison de jusqu’à 123 octets. [Détails](../experiments/nodes.md#node-ws_close) |

Une connexion que l’exécution a encore ouverte quand elle se termine — ou est arrêtée — est fermée
proprement. Le modèle [[ui:exp.templateWsEcho]] est un exemple travaillé.

## Depuis la ligne de commande {#cli}

`signallab send ws` fait un échange : se connecter, envoyer un message, attendre la
réponse si vous en demandez une, fermer.

```bash
signallab send ws ws://127.0.0.1:9001/ --text '{"type":"ping"}' --expect pong
```

La poignée de main et ce qui a été envoyé vont vers l’erreur standard, la réponse vers la sortie
standard :

```text
Connected to ws://127.0.0.1:9001/ in 4 ms
Sent 15 bytes
{"type":"pong"}
```

`--hex` envoie un message binaire ; `-H` ajoute un en-tête et `--protocol` propose un
sous-protocole (les deux répétables). `--expect TEXT`, `--expect-regex RE` ou `--wait`
(n’importe quel message) disent quelle réponse attendre, pendant `--timeout` millisecondes (2000
par défaut). Il sort avec 1 quand la réponse ne vient pas ou que la connexion
échoue. Voir [Ligne de commande](../automation/cli.md#cli-send-ws).

## Problèmes {#troubleshooting}

| Ce que vous voyez | Cause habituelle |
| --- | --- |
| `… is not a WebSocket address` | L’URL ne commence pas par `ws://` ou `wss://`, ou n’a pas d’hôte. |
| `… answered HTTP n instead of switching to WebSocket` | Le serveur a refusé la mise à niveau : un mauvais chemin (404), un jeton manquant ou erroné (401, 403). Le début de sa réponse se trouve sous les détails techniques. |
| `… did not take any of the subprotocols offered` | Vous avez proposé des sous-protocoles et le serveur n’en a choisi aucun, ou il a répondu avec un que vous n’avez pas proposé. |
| `… is not a subprotocol name` | Un nom avec un espace, une virgule ou une barre oblique. |
| `The header … cannot be sent with the upgrade` | Un nom ou une valeur d’en-tête avec des caractères que HTTP n’autorise pas. |
| `… refused the connection` | Rien n’écoute sur ce port. |
| `A secure connection to … could not be made` | Le certificat n’est pas de confiance ici, ou TLS a échoué. Voir [Connexions sécurisées](#wss). |
| `The connection with … broke: the server did not keep to the WebSocket protocol` | Le serveur a envoyé quelque chose qui n’est pas du WebSocket valide. |
| `A WebSocket message is limited to … bytes` | Le serveur a envoyé un message de plus de 16 Mio, ce qui termine la connexion. |
| `Too long: at most 16777216` | Le message que vous avez essayé d’envoyer fait plus de 16 Mio. Rien n’a été envoyé ; la connexion est ouverte. |

Chaque message d’erreur est listé dans [Messages d’erreur](../reference/errors.md#ws).

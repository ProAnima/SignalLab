---
title: "Qu’est-ce que Signal Lab"
description: "À quoi sert Signal Lab, ce que vous pouvez en faire, ses deux modes de fonctionnement et son organisation."
---

# Qu’est-ce que Signal Lab

Signal Lab est un banc d’essai pour OSC et les protocoles réseau. Il envoie les messages que parlent vos
équipements et vos services, vous montre ce qui revient, remplace l’appareil ou l’API qui n’est
pas encore là, dégrade exprès le réseau qui les relie, et fait de tout cela des
expériences que vous pouvez relancer — depuis l’application, un script ou un pipeline de CI.

Il a été conçu pour mettre en service une installation avant qu’elle n’existe : le contrôleur
de spectacle, le serveur média, les capteurs et l’API cloud peuvent chacun être testés face à un
remplaçant, sur un seul portable, bien avant de se rencontrer sur site.

## À qui il s’adresse {#audience}

- **Aux personnes qui mettent en service la régie de spectacle, les installations et les appareils en réseau** : serveurs
  d’éclairage et serveurs média, contrôleurs, capteurs, projecteurs, tout ce qui parle OSC, UDP,
  TCP ou MQTT.
- **Aux testeurs d’API et de services** : points de terminaison HTTP et WebSocket, leur authentification,
  leur comportement sous charge et quand une dépendance tombe en panne.
- **À ceux qui automatisent l’un ou l’autre** : les mêmes expériences s’exécutent sans fenêtre dans un pipeline, sur un
  serveur de laboratoire à côté des équipements, ou pilotées par un assistant d’IA.

## Ce que vous pouvez faire {#what-you-can-do}

### Envoyer et observer le trafic {#send-and-watch}

Chaque protocole a son propre écran :

| Protocole | Ce que vous pouvez faire | Page |
| --- | --- | --- |
| OSC | Envoyer des messages aux arguments typés, surveiller un port, envoyer une forme d’onde vers un point de terminaison | [OSC](../protocols/osc.md) |
| UDP et TCP bruts | Envoyer du texte ou des octets dans les expériences et depuis la bibliothèque, émuler des appareils qui les parlent | [UDP et TCP](../protocols/udp-tcp.md) |
| HTTP | Une requête avec sa réponse complète, l’authentification Basic, Bearer ou Digest, une réserve de cookies, une rafale de charge | [HTTP](../protocols/http.md) |
| WebSocket | Se connecter avec les en-têtes et sous-protocoles qu’attend un service, envoyer du texte ou des octets, lire chaque message | [WebSocket](../protocols/websocket.md) |
| MQTT 3.1.1 | Se connecter à un broker, observer chaque topic qu’il contient, publier en QoS 0, 1 ou 2, effacer une valeur retenue | [MQTT](../protocols/mqtt.md) |
| Diffusion et multidiffusion | Envoyer à une liste, à une adresse de diffusion, à un groupe de multidiffusion ou à chaque hôte d’un sous-réseau ; écouter qui répond | [Diffusion et découverte](../protocols/broadcast.md) |

### Voir chaque trame {#inspect}

L’[Inspecteur](../tools/inspector.md) enregistre chaque trame que les outils envoient et reçoivent, sur
une seule chronologie : décodée, avec ses octets, filtrable et exportable. Une trame qu’il a capturée peut
devenir un signal qui la rejoue octet par octet.

### Garder ce qui fonctionne {#library}

Un message qui a fonctionné va dans la [bibliothèque de signaux](../tools/signals.md) : nommé, rangé
dans des dossiers, avec une note sur ce qu’il doit déclencher. Vous le renvoyez depuis son
écran, depuis la bibliothèque ou de n’importe où avec <kbd>Ctrl</kbd>+<kbd>K</kbd>.

### Jouer l’autre côté {#emulate}

Les [émulateurs](../tools/emulators.md) répondent à la place de l’API, de l’appareil ou du service auquel votre système
s’adresse : une API HTTP avec ses routes et ses réponses, un appareil OSC, UDP ou TCP avec ses règles, un broker
MQTT. Ils peuvent répondre lentement, échouer selon une séquence, envoyer des corps mal formés ou tomber en panne selon un
calendrier — pour tester comment votre système s’en sort avant que le vrai ne soit là.

### Casser le réseau exprès {#impair}

Un [relais de dégradation](../tools/impairment.md) se place entre un client et sa cible et
ajoute au trafic UDP de la latence, de la gigue, des pertes, des doublons, du réordonnancement ou une limite de bande passante, ou bien retarde,
bride, réinitialise et bloque les connexions TCP. Chaque décision suit une graine : le même
trafic connaît deux fois le même sort.

### En faire un test {#experiments}

Une [expérience](../experiments/index.md) est un enchaînement d’étapes que vous dessinez sur un canevas : envoyer une
requête, attendre la réponse, la vérifier, extraire une valeur, bifurquer, boucler, exécuter des branches en
parallèle, démarrer un émulateur ou un relais de dégradation pour l’exécution. Chaque exécution est détaillée
étape par étape et enregistrée dans un rapport que vous pouvez comparer à un précédent.

### Mettre un service sous charge {#load}

La requête HTTP d’une expérience peut s’exécuter [sous charge](../experiments/load.md) — un débit
constant, une rampe, des paliers, un pic ou des arrivées aléatoires — mesurée et jugée par des seuils. L’écran
[HTTP](../protocols/http.md) offre une rafale de charge rapide, et [Tempête](../tools/storm.md)
génère une charge UDP ou TCP brute contre vos propres serveurs et liaisons.

### Trouver ce qu’il y a sur le réseau {#discover}

[Diffusion et découverte](../protocols/broadcast.md) trouve les appareils qui répondent à une sonde,
et le [Scanner](../tools/scanner.md) montre quels ports TCP d’un hôte sont ouverts.

### L’automatiser {#automate}

La [ligne de commande](../automation/cli.md) `signallab` exécute les expériences sans fenêtre et
se termine avec un code qu’un pipeline comprend ; elle écrit des rapports JUnit pour la [CI](../automation/ci.md).
`signallab mcp` permet à un [assistant d’IA](../automation/mcp.md) de lire, d’écrire et d’exécuter des expériences.
Un [serveur](../server/index.md) offre la même chose par son [API HTTP](../api/index.md).

## Deux modes de fonctionnement {#two-ways}

| | Application de bureau | Serveur, dans un navigateur |
| --- | --- | --- |
| Fonctionne sous | Windows 10 et 11 (x64), Linux (x86_64) | Linux, en image Docker (amd64 et arm64), ou compilé depuis les sources |
| Vous l’utilisez | dans sa propre fenêtre | dans Chrome, Edge ou Firefox, depuis toute machine qui peut joindre le serveur |
| Le trafic part de | cet ordinateur | le serveur |
| Fourni avec | la ligne de commande `signallab` | la ligne de commande `signallab`, dans l’image |
| Mises à jour | d’elle-même, quand vous le décidez | avec son image |

Les deux sont la même interface sur le même moteur : chaque écran, l’Inspecteur, les expériences
et les rapports fonctionnent de la même façon. Le serveur s’impose quand les équipements sont sur un réseau que votre
propre ordinateur ne peut pas joindre — un PC en rack ou un petit boîtier Linux à côté, utilisé depuis un
portable ou par un pipeline. Voir [Installation et mises à jour](install.md) et
[Le serveur](../server/index.md) ; ce qui diffère entre les deux est décrit dans
[Concepts](concepts.md#desktop-and-server).

## Organisation {#organisation}

- **Les écrans** sont des outils pour le travail que vous faites sur le moment, à la main : envoyer ceci, écouter là, démarrer
  cet émulateur. La barre latérale les liste ; chacun garde ce que vous avez saisi et ce qu’il a reçu
  pendant que vous passez à un autre. Voir [La fenêtre](interface.md).
- **Les expériences** sont des scénarios que vous construisez une fois et relancez, de la même façon à chaque fois, avec
  un rapport pour chaque exécution.
- **Les bibliothèques** conservent ce que vous avez créé : les signaux dans la bibliothèque de signaux, les émulateurs dans la
  bibliothèque d’émulateurs. Les expériences utilisent les deux.
- **Les tâches** sont le travail de longue durée — un moniteur, un émulateur, un relais, une exécution — listées
  en bas de la fenêtre, où vous en arrêtez une ou toutes.

[Concepts](concepts.md) explique chacun de ces éléments plus en détail.

## Sûr par défaut {#defaults}

- Signal Lab n’envoie rien tant que vous n’appuyez pas sur un bouton, et seulement là où vous le lui dites. La
  seule exception est la recherche de mises à jour de l’application de bureau, une fois par jour, que vous pouvez désactiver
  (voir [Mises à jour](install.md#updates)).
- Les signaux de départ, les émulateurs de départ et les modèles d’expérience visent tous
  `127.0.0.1`, cet ordinateur. Atteindre le réseau est un choix que vous faites en saisissant une
  adresse.
- Un serveur démarré sans jeton d’accès n’écoute que sur `127.0.0.1` et refuse toute
  autre adresse.
- Le pare-feu ne change que lorsque vous cliquez pour l’autoriser.

## Usage responsable {#responsible-use}

::: danger Du vrai trafic vers de vrais hôtes
[Tempête](../tools/storm.md), le [Scanner](../tools/scanner.md) et la
[Diffusion](../protocols/broadcast.md) envoient du vrai trafic à de vrais hôtes, et une diffusion
ou un balayage de sous-réseau atteint chaque appareil du segment, pas seulement celui que vous aviez en tête.
Ne les dirigez que vers des systèmes qui vous appartiennent ou que vous êtes autorisé à tester, et vérifiez d’abord sur quel réseau
vous êtes. Des débits élevés peuvent saturer les liaisons et déclencher la détection d’intrusion.
:::

Le moteur a des garde-fous : un balayage atteint au plus 1024 hôtes, et une balise envoie au
plus 50 000 paquets par seconde sur l’ensemble de ses cibles. Ce sont des garde-fous, pas une
autorisation.

## Langues {#languages}

L’interface, ses info-bulles et ses messages d’erreur, la ligne de commande `signallab`, la
page de connexion du serveur et le programme d’installation Windows parlent 11 langues : English (anglais), Русский
(russe), Español (espagnol), Français, Deutsch (allemand), Português (portugais
du Brésil), 中文 (chinois simplifié), 日本語 (japonais), 한국어 (coréen), हिन्दी (hindi) et
العربية (arabe, de droite à gauche). Vous changez de langue à la volée depuis l’en-tête ; voir
[La fenêtre](interface.md#languages).

## Pour continuer {#next}

1. [Installez Signal Lab](install.md).
2. Repérez-vous dans [la fenêtre](interface.md).
3. Faites les [premiers pas](first-steps.md) sur cet ordinateur.
4. Lisez les [concepts](concepts.md) sur lesquels il repose.

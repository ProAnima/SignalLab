---
title: "Diffusion"
description: "Envoyer une charge utile à une liste d'hôtes, une adresse de diffusion, un groupe de multidiffusion ou à tous les hôtes d'un sous-réseau, une fois ou en balise, et écouter qui répond avec l'écouteur de découverte."
---

# Diffusion

L'écran [[ui:nav.broadcast]] ([[ui:bc.title]]) envoie une charge utile UDP à de nombreuses
destinations à la fois, et écoute de l'autre côté pour voir qui répond. Servez-vous-en
pour trouver des appareils sur un réseau, vérifier qu'un flux de multidiffusion atteint un
récepteur, ou jouer un appareil qui répond aux sondes de découverte.

- L'[[ui:bc.emitter]] envoie à une liste d'hôtes, une adresse de diffusion, un
  groupe de multidiffusion, ou à tous les hôtes d'un sous-réseau — une fois, ou encore et encore en
  balise.
- L'[[ui:bc.discovery]] écoute sur un port, rejoint des groupes de multidiffusion, liste chaque
  pair qui lui parle, et peut répondre aux sondes.

::: danger
La diffusion, la multidiffusion et un balayage atteignent chaque appareil du segment réseau,
pas seulement celui que vous avez en tête, et une balise continue de le faire. Vérifiez sur quel
réseau vous êtes d'abord, et n'envoyez qu'à des réseaux qui vous appartiennent ou que vous êtes autorisé à
tester. Les limites ci-dessous sont des garde-fous, pas une autorisation.
:::

## Envoyer {#send}

1. Choisissez le [[ui:bc.mode]] (ci-dessous).
2. Saisissez la destination : le nom du champ change avec le mode. Sur un réseau
   avec une adresse IPv4, [[ui:bc.useSubnet]] le remplit à partir de l'adresse de cet ordinateur.
3. Choisissez la [[ui:bc.payload]] et écrivez-la.
4. Appuyez sur [[ui:bc.sendOnce]] : un datagramme part vers chaque destination.

### Modes {#modes}

| [[ui:bc.mode]] | Destination | Ce qui se passe | Par défaut |
| --- | --- | --- | --- |
| [[ui:bc.mode.list]] | [[ui:bc.targetList]] : des entrées `IP:port` ou `host:port` séparées par des virgules, des points-virgules ou des sauts de ligne — un espace ne les sépare pas. Un nom est résolu, et son adresse IPv4 prise quand il en a une. | Un datagramme vers chacune | `127.0.0.1:9000, 127.0.0.1:9001` |
| [[ui:bc.mode.broadcast]] | [[ui:bc.targetAddress]] : `255.255.255.255:port`, ou une adresse se terminant par `.255` | Un datagramme que chaque hôte du réseau local reçoit. Les routeurs ne le transmettent pas. | `255.255.255.255:9000` |
| [[ui:bc.mode.multicast]] | [[ui:bc.targetAddress]] : un groupe de `224.0.0.0` à `239.255.255.255`, avec un port | Un datagramme vers le groupe ; seuls les écouteurs qui l'ont rejoint le reçoivent | `239.1.1.1:9000` |
| [[ui:bc.mode.sweep]] | [[ui:bc.targetCidr]], `a.b.c.d/nn`, et un [[ui:bc.port]] | Un datagramme vers chaque hôte utilisable du bloc, en unicast — pour les appareils qui ignorent la diffusion | `192.168.1.0/24`, port 9000 |

Dans un balayage, les adresses de réseau et de diffusion sont sautées (sauf dans un `/31` ou
`/32`), et une base qui n'est pas celle du réseau est ramenée à elle :
`192.0.2.77/30` balaie `192.0.2.77` et `192.0.2.78`. Un balayage atteint au plus
1024 hôtes, le bloc le plus large est donc un `/22` (1022 hôtes) ; un bloc plus large est refusé
avec le préfixe vers lequel le réduire.

La diffusion, les groupes de multidiffusion, les balayages et les jointures de l'écouteur de découverte sont IPv4
uniquement : IPv6 n'a pas de diffusion. Une [[ui:bc.mode.list]] peut aussi nommer des hôtes IPv6
(voir [Options de socket](#socket-options)).

[[ui:bc.useSubnet]] remplit `x.y.z.10:9000, x.y.z.11:9000` dans [[ui:bc.mode.list]],
`x.y.z.255:9000` dans [[ui:bc.mode.broadcast]] et `x.y.z.0/24` dans
[[ui:bc.mode.sweep]], à partir de l'adresse de cet ordinateur `x.y.z.w`. Il suppose un réseau
en `/24`.

### Charge utile {#payload}

| [[ui:bc.payload]] | Ce qui est envoyé |
| --- | --- |
| [[ui:bc.payload.osc]] | Un message OSC : [[ui:common.address]] et des [[ui:common.arguments]] typés, comme sur l'écran [OSC](osc.md). Par défaut `/hello/discover` avec le texte `who-is-there`. |
| [[ui:bc.payload.text]] | Le [[ui:bc.text]] en UTF-8, exactement comme saisi, sans terminateur. Par défaut `HELLO-PROBE`. |
| [[ui:bc.payload.hex]] | Le [[ui:bc.hex]] octet par octet — pour rejouer une trame capturée ou parler un protocole de découverte binaire. Des paires de chiffres hexadécimaux ; tout ce qui les sépare est ignoré. Par défaut `48 45 4c 4c 4f`. |

### Options de socket {#socket-options}

[[ui:bc.socketOptions]] ouvre trois réglages de plus :

| Option | Quoi | Par défaut |
| --- | --- | --- |
| [[ui:bc.bindSource]] | L'`IP:port` local duquel partent les datagrammes. Épinglez-le pour choisir la carte réseau, ou un port source auquel un appareil répond. `0.0.0.0:0` : n'importe lequel. | `0.0.0.0:0` |
| [[ui:bc.ttl]] | Combien de routeurs un datagramme peut traverser, 1–255. Pour la multidiffusion, c'est la limite de sauts multicast : 1 le garde sur ce réseau. | `1` |
| [[ui:bc.mcastLoop]] | Multidiffusion uniquement : remettre aussi les datagrammes du groupe à cet ordinateur, pour qu'un écouteur ici les entende | activé |

Avec la [[ui:bc.bindSource]] par défaut, les datagrammes partent d'un socket IPv4,
ou d'un socket IPv6 quand toutes les destinations sont IPv6. Une liste qui mélange les deux
part du socket IPv4, et ses destinations IPv6 échouent ; envoyez-les comme une
liste à part, ou épinglez [[ui:bc.bindSource]] sur une adresse IPv6.

### Le résultat {#result}

[[ui:bc.lastEmit]] montre ce qui est parti : [[ui:bc.targets]], [[ui:common.packets]],
[[ui:common.volume]] et [[ui:common.errors]], et les huit premières destinations
atteintes (« … +n more » pour les autres). Une erreur vers une destination n'arrête pas
les autres ; la ligne de console nomme combien ont échoué.

## Répéter en balise {#beacon}

Une balise envoie la même tournée — un datagramme vers chaque destination — selon un
calendrier, jusqu'à ce que vous l'arrêtiez. Les appareils qui attendent une annonce périodique
en ont besoin.

1. Réglez le mode, la destination et la charge utile comme pour un envoi unique.
2. Sous [[ui:bc.beacon]], réglez [[ui:bc.beaconRate]], et s'il doit s'arrêter de lui-même,
   [[ui:bc.beaconRounds]] ou [[ui:bc.beaconSeconds]].
3. Appuyez sur [[ui:bc.startBeacon]]. [[ui:bc.stopBeacon]] — ou l'arrêt de sa tâche dans le
   bandeau de la console — y met fin.

| Champ | Quoi | Par défaut |
| --- | --- | --- |
| [[ui:bc.beaconRate]] | Tournées par seconde ; doit être supérieur à 0 | `2` |
| [[ui:bc.beaconRounds]] | S'arrêter après ce nombre de tournées ; 0 — sans limite | `0` |
| [[ui:bc.beaconSeconds]] | S'arrêter après ce nombre de secondes ; 0 — sans limite | `0` |

Le débit multiplié par le nombre de destinations peut atteindre au plus 50 000 datagrammes par
seconde. Un balayage d'un `/24` (254 hôtes) peut donc se répéter environ 196 fois par
seconde au plus. Pendant que la balise tourne, [[ui:bc.lastEmit]] affiche
[[ui:bc.targets]] (les destinations de chaque tournée, à partir du premier rapport),
les totaux, [[ui:bc.rounds]] et [[ui:bc.pps]] (datagrammes par seconde), mis à jour
quatre fois par seconde, et [[ui:bc.sendOnce]] est indisponible. Une balise qui a eu plus de
32 datagrammes en échec et pas un seul envoyé — aucune route, diffusion non autorisée — s'arrête
d'elle-même et dit pourquoi.

## Écouter les appareils {#discovery}

L'[[ui:bc.discovery]] lie un port UDP et enregistre chaque pair qui lui envoie quelque chose :
ce qui répond à une sonde, ou ce qu'un appareil annonce de lui-même.

1. Réglez [[ui:common.bind]], le port auquel les appareils envoient.
2. Pour la multidiffusion, listez les groupes dans [[ui:bc.joinGroups]].
3. Appuyez sur [[ui:bc.startListen]]. Les réglages se verrouillent jusqu'à ce que vous appuyiez sur
   [[ui:bc.stopListen]].

| Champ | Quoi | Par défaut |
| --- | --- | --- |
| [[ui:common.bind]] | Où écouter, `IP:port`. `0.0.0.0` écoute sur chaque carte réseau. | `0.0.0.0:9000` |
| [[ui:bc.joinGroups]] | Groupes de multidiffusion IPv4 à rejoindre, séparés par des virgules ; vide — unicast et diffusion seulement | `239.1.1.1` |
| [[ui:bc.interface]] | L'adresse IPv4 de la carte réseau sur laquelle rejoindre les groupes ; vide — le système choisit | vide |
| [[ui:bc.reuse]] | Écouter sur un port qu'un autre programme utilise aussi (`SO_REUSEADDR`). Ne fonctionne que si ce programme autorise aussi le partage. | activé |
| [[ui:bc.respond]] | Répondre aux sondes, comme le ferait un appareil ([ci-dessous](#auto-reply)) | désactivé |

### Pairs {#peers}

[[ui:bc.peers]] liste qui a envoyé quelque chose, le plus récent en premier, avec le
nombre de pairs vus, les paquets entendus et les réponses envoyées au-dessus :

| Colonne | Quoi |
| --- | --- |
| [[ui:bc.peer]] | L'`IP:port` de l'expéditeur ; son point indique s'il a été entendu dans les 3 dernières secondes |
| [[ui:bc.proto]] | `osc` quand son dernier datagramme s'est décodé en OSC, sinon `udp` |
| [[ui:common.packets]] | Combien il en a envoyé |
| [[ui:bc.age]] | Secondes depuis son dernier datagramme |
| [[ui:bc.lastMessage]] | Son dernier datagramme : l'adresse et les arguments OSC, ou le début du texte |

La liste est rafraîchie quelques fois par seconde et contient jusqu'à 512 pairs ; au-delà,
les paquets sont toujours comptés mais les nouveaux pairs n'obtiennent pas de ligne.

### Répondre aux sondes {#auto-reply}

Avec [[ui:bc.respond]], l'écouteur joue un appareil : il répond à chaque datagramme
qu'il reçoit, depuis le port d'écoute, à l'adresse et au port de l'expéditeur.

| Champ | Quoi | Par défaut |
| --- | --- | --- |
| [[ui:bc.payload]] | La réponse : OSC, texte ou hex, comme pour l'envoi | OSC `/hello/here` avec le texte `signal-lab` |
| [[ui:bc.replyDelay]] | Attendre ce délai avant de répondre, comme le ferait un appareil lent | `0` |
| [[ui:bc.matchContains]] | Répondre seulement aux datagrammes dont le texte décodé contient ceci — l'adresse et les arguments OSC, ou le début du texte ; vide — à tous | vide |

L'écouteur ne répond jamais à un datagramme identique à sa propre réponse, donc deux
écouteurs pointés l'un vers l'autre ne se répondent pas sans fin.

### Pare-feu et ports partagés {#firewall}

Le trafic de diffusion et de multidiffusion venant d'autres machines est bloqué par défaut par
la plupart des pare-feu Windows : autorisez Signal Lab sur les réseaux privés quand l'application
le propose. La diffusion ne traverse jamais un routeur. Pour écouter sur un port que le vrai
service possède déjà, les deux côtés doivent autoriser le partage ([[ui:bc.reuse]] ici) ;
sans cela, un port pris est refusé avec une indication pour l'activer. Voir
[Dépannage](../reference/troubleshooting.md).

## Dans l'Inspecteur {#inspector}

Avec la capture active, le trafic de l'écran apparaît avec le protocole `osc` ou
`udp`, selon sa charge utile :

| Source | Quoi | Combien |
| --- | --- | --- |
| `broadcast` | [[ui:bc.sendOnce]] : chaque datagramme, avec le verdict `fan-out`, `broadcast`, `multicast` ou `sweep` ; un échec avec `error: …` | chacun |
| `beacon` | Les tournées d'une balise | au plus une tournée toutes les 50 ms |
| `discovery` | Les datagrammes que reçoit l'écouteur | au plus un toutes les 40 ms |
| `discovery` | Ses réponses, avec le verdict `auto-reply` | chacune |

Ce que l'Inspecteur laisse de côté d'une balise ou de l'écouteur est compté : la prochaine
trame qu'il dessine porte le nombre dans son verdict, `+n not shown` (pour une balise,
une trame pour chaque destination de chaque tournée laissée de côté). Voir
[Inspecteur](../tools/inspector.md).

## Sur un serveur ou dans Docker {#server}

Sur un [serveur](../server/index.md), l'écran envoie et écoute sur le réseau du serveur.
Dans Docker, la diffusion, la multidiffusion et la découverte n'atteignent le réseau local
que lorsque le conteneur utilise le réseau de l'hôte (`--network host`) sur un hôte
Linux. Avec le réseau bridge par défaut de Docker, ou Docker Desktop, seul l'unicast vers des
hôtes que le conteneur peut joindre fonctionne.

## Ailleurs {#elsewhere}

- [`signallab send udp`](../automation/cli.md#cli-send-udp) envoie un datagramme à
  un hôte ; il n'y a pas de diffusion, de multidiffusion ni de balayage en ligne de commande.
- Une étape [[ui:exp.node.udp]] envoie un datagramme texte à un ou plusieurs hôtes, et une
  étape [[ui:exp.node.wait_udp]] en attend un. Voir [UDP et TCP](udp-tcp.md).
- Pour jouer un appareil qui répond selon des règles — plusieurs règles, des réponses construites à partir de
  ce qui est arrivé —, utilisez un émulateur [[ui:emu.new.udp]] ou [[ui:emu.new.osc]]. Voir
  [Émulateurs](../tools/emulators.md).

## Problèmes {#troubleshooting}

| Ce que vous voyez | Cause habituelle |
| --- | --- |
| `… is not a broadcast address` | [[ui:bc.mode.broadcast]] prend `255.255.255.255:port` ou une adresse se terminant par `.255`. Pour un autre masque de sous-réseau, utilisez [[ui:bc.mode.sweep]]. |
| `… is not a multicast group` | L'adresse est hors de `224.0.0.0`–`239.255.255.255`. |
| `… spans … addresses, and a sweep reaches at most 1024 hosts` | Le bloc est plus large qu'un `/22` ; réduisez-le. |
| `Set the port to sweep` | [[ui:bc.port]] vaut 0. |
| `… is over the … pps limit` | Le débit multiplié par le nombre de destinations dépasse 50 000 par seconde : baissez [[ui:bc.beaconRate]], ou réduisez la destination. |
| `… is already in use — turn on “share the port” to listen alongside it` | Un autre programme a le port ; cochez [[ui:bc.reuse]]. |
| `Cannot join the multicast group …` | Le groupe ou [[ui:bc.interface]] n'est pas utilisable sur cet ordinateur — aucune carte réseau avec cette adresse, ou aucune route multicast. |
| Envoyé, mais personne ne répond | Les appareils écoutent sur un autre port ; le pare-feu ici garde leurs réponses à l'extérieur ; un routeur se trouve entre vous ; ou, dans Docker, le conteneur n'est pas sur le réseau de l'hôte. |
| Les sondes partent, mais l'écouteur de découverte n'entend aucune réponse | Beaucoup d'appareils répondent à l'adresse et au port d'où vient une sonde — le socket propre à l'émetteur, que l'écran ne lit pas. Écoutez sur le port auquel les appareils répondent, ou envoyez la sonde depuis une expérience : une étape [[ui:exp.node.udp]] avec [[ui:exp.expectReply]] envoie et écoute sur le même port. |

Chaque message d'erreur est listé dans [Messages d'erreur](../reference/errors.md#broadcast).

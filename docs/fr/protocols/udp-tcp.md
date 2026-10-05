---
title: UDP et TCP
description: "Où Signal Lab envoie et reçoit des datagrammes UDP et des données TCP brutes — étapes d’expérience, signaux, ligne de commande, outils de charge et de scan et appareils émulés — et comment les charges utiles s’écrivent."
---

# UDP et TCP

L’UDP et le TCP bruts n’ont pas d’écran à eux. Ils servent pour un
appareil avec son propre protocole texte ou binaire — un projecteur, un serveur multimédia, un
capteur — et ils apparaissent à plusieurs endroits :

| Pour… | Utilisez |
| --- | --- |
| envoyer un datagramme ou un message TCP comme étape, et attendre la réponse | les [étapes d’expérience](#experiments) |
| garder un datagramme pour l’envoyer de nouveau, ou rejouer un que vous avez capturé | un [signal UDP](#signals) |
| envoyer un datagramme depuis un script | [`signallab send udp`](#cli) |
| envoyer à de nombreux hôtes à la fois, vers une adresse de diffusion ou un groupe de multidiffusion | l’écran [Diffusion](broadcast.md) |
| charger un serveur ou une liaison avec du trafic | [Tempête](#storm) |
| trouver quels ports TCP un hôte a ouverts | [Scanner](#scanner) |
| jouer le côté de l’appareil | un émulateur d’[appareil UDP ou TCP](#emulators) |
| dégrader le réseau entre deux extrémités | un [relais de dégradation](#impairment) |

OSC est un format porté dans des datagrammes UDP ; il a sa propre page :
[OSC](osc.md).

## Charges utiles {#payloads}

Partout où vous écrivez une charge utile brute, elle est de deux genres :

| Genre | Ce qui est envoyé | Exemple |
| --- | --- | --- |
| Texte | Les caractères en UTF-8, exactement comme saisis : pas de terminateur, pas de fin de ligne ajoutée. Un protocole ligne a besoin de sa fin de ligne dans le texte. | `PING` |
| Octets hex | Octet pour octet, écrits par paires de chiffres hex. Les espaces, `:`, `-` et `,` entre les paires et un `0x` devant elles sont autorisés. | `de ad be ef`, `DEADBEEF`, `0xde,0xad` |

Un datagramme porte au plus 65 507 octets. Un nombre impair de chiffres hex, ou aucun,
est une erreur avant que quoi que ce soit ne soit envoyé.

::: info
UDP n’a pas d’accusé de réception. « Envoyé » signifie que le datagramme a quitté cet ordinateur, pas
que quelque chose l’a reçu. Pour savoir qu’un appareil vous a entendu, attendez sa réponse.
:::

## Où cela va {#destinations}

Une destination est une adresse IP et un port, ou un nom d’hôte et un port :
`192.0.2.20:9000`, `[2001:db8::20]:9000` (une adresse IPv6 se met entre crochets),
`projector.local:9000`. Cela vaut pour la cible d’une étape UDP, une cible OSC,
un signal UDP, `signallab send udp` et `signallab send osc`, la liste de
[Diffusion](broadcast.md), la cible de [Tempête](../tools/storm.md) et l’hôte de l’étape
TCP.

Un nom d’hôte est résolu à chaque fois qu’il est utilisé. Quand il a une adresse IPv4,
c’est elle qui est utilisée — si bien que `localhost` atteint un service qui écoute sur `127.0.0.1`,
où la première adresse qu’un système liste pour lui peut être `::1` — et un nom qui
n’a que des adresses IPv6 est joint depuis un socket IPv6. Un nom qui ne se
résout pas échoue avec `Cannot resolve …` ; une destination sans port, ou qui n’est
ni l’une ni l’autre forme, avec `… is not a valid address`. Les adresses sur lesquelles un service *écoute*
(celles d’un moniteur, d’une attente, d’un émulateur) sont toujours `IP:port`.

## Dans les expériences {#experiments}

| Étape | Ce qu’elle fait |
| --- | --- |
| [[ui:exp.node.udp]] | Envoie sa [[ui:exp.payload]] en texte vers [[ui:common.target]]. La cible est `IP:port` ou `host:port` ([ci-dessus](#destinations)), et plusieurs cibles séparées par des virgules reçoivent chacune le datagramme. Avec [[ui:exp.expectReply]] il envoie depuis [[ui:exp.replyOn]] et attend là une réponse dans la même étape. [Détails](../experiments/nodes.md#node-udp) |
| [[ui:exp.node.wait_udp]] | Écoute sur [[ui:exp.listenOn]] (`IP:port`) et attend un datagramme dont la charge utile correspond. [Détails](../experiments/nodes.md#node-wait_udp) |
| [[ui:exp.node.tcp]] | Se connecte à [[ui:exp.host]] et [[ui:exp.port]], écrit sa [[ui:exp.payload]] en texte, écoute 250 ms une réponse, et ferme. L’étape dit combien d’octets sont revenus, pas ce qu’ils étaient. [Détails](../experiments/nodes.md#node-tcp) |

La charge utile et la cible UDP, l’hôte et la charge utile TCP, et le motif d’une attente prennent
des `{{templates}}`, si bien qu’un datagramme peut porter l’identifiant de l’exécution ou une valeur
extraite par une étape antérieure. Voir [Données et modèles](../experiments/data.md).

### Mettre un datagramme en correspondance {#matching}

[[ui:exp.node.wait_udp]] et la réponse de [[ui:exp.node.udp]] choisissent un datagramme
par sa charge utile :

| [[ui:exp.waitMode]] | Prend un datagramme quand |
| --- | --- |
| [[ui:exp.mode.any]] | toujours : le premier qui arrive |
| [[ui:exp.mode.contains]] | sa charge utile, lue comme du texte, contient le motif |
| [[ui:exp.mode.regex]] | sa charge utile, lue comme du texte, correspond à l’expression régulière |
| [[ui:exp.mode.hex]] | ses octets contiennent les octets du motif, écrits en hex |

Ce qui a correspondu est gardé dans la variable de l’étape ([[ui:exp.replyVariable]],
`reply` sauf si vous la renommez) : `text`, `hex` (les 1024 premiers octets), `bytes`
(la taille), `from` (l’`IP:port` de l’expéditeur), `ms` (combien de temps cela a pris) et `match`
(le texte ou les octets trouvés, ou le premier groupe d’une expression régulière).

Une attente commence à écouter au démarrage de l’exécution, pas quand l’étape est atteinte, si bien qu’une
réponse qui arrive très vite n’est pas manquée. Elle ne prend que ce qui est arrivé après
le dernier envoi sur son chemin.

### Limites et valeurs par défaut {#limits}

| Réglage | Par défaut | Plage |
| --- | --- | --- |
| Étape TCP : [[ui:common.timeoutMs]] (connexion, écriture et réponse ensemble) | 4000 ms | 1–120 000 ms |
| [[ui:exp.waitTimeout]] d’une attente ou d’une réponse | 2000 ms | 1–120 000 ms |
| Charge utile UDP | — | au plus 65 507 octets |
| Adresse d’écoute | — | `IP:port` avec un port ; le [[ui:exp.replyOn]] d’une réponse peut utiliser le port 0 (n’importe quel port libre) |

Dans l’Inspecteur, le datagramme d’une étape UDP apparaît avec la source `broadcast`
(ou `experiment` quand l’étape attend une réponse), et chaque datagramme qui arrive
au port d’une attente avec la source `experiment-wait`. Une étape TCP apparaît comme deux
trames `tcp` avec la source `experiment` : la charge utile qu’elle a écrite et, quand il y en a
une, la réponse qu’elle a lue. Son entrée de chronologie indique ce qu’elle a envoyé et la taille de la
réponse.

## Signaux {#signals}

Un signal [[ui:sig.tr.udp]] de la bibliothèque est une cible et une charge utile, en texte ou en
octets hex. Envoyez-le depuis [[ui:nav.signals]], ou avec <kbd>Ctrl</kbd>+<kbd>K</kbd>
depuis n’importe quel écran. Sa cible peut être un nom d’hôte.

Tout datagramme que l’Inspecteur a gardé entier peut en devenir un : [[ui:sig.fromFrame]] fabrique un signal
UDP hex, dans le dossier [[ui:sig.capturedFolder]], qui rejoue ces octets exacts — vers la
destination de la trame pour une trame qui a été envoyée, vers l’adresse qui l’a
reçue pour une qui est arrivée (sur cet ordinateur, quand c’était toutes les adresses).
Un morceau TCP ne le peut pas : c’est une pièce d’un flux. Voir [Signaux](../tools/signals.md) et
[Inspecteur](../tools/inspector.md#save-as-signal).

Un signal UDP texte peut être ajouté à une expérience comme étape [[ui:exp.node.udp]] ;
un signal hex ne le peut pas, parce que l’étape envoie du texte.

## Depuis la ligne de commande {#cli}

`signallab send udp` envoie un datagramme :

```bash
signallab send udp 127.0.0.1:9000 --text "PING"
signallab send udp 127.0.0.1:9000 --hex "de ad be ef"
```

```text
✔ sent 4 bytes → 127.0.0.1:9000
```

Donnez exactement l’un de `--text` et `--hex`. La cible est `IP:port` ou
`host:port` ([ci-dessus](#destinations)). Il sort avec 0 quand le datagramme est parti, 1 quand l’envoi a échoué,
et 2 quand la cible ou le hex est invalide. Il n’y a pas de `send tcp`. Voir [Ligne de commande](../automation/cli.md#cli-send-udp).

## Tempête {#storm}

[[ui:nav.storm]] est une source de charge pour vos propres serveurs et liaisons : un
[[ui:st.udp]] envoie des datagrammes d’une taille fixée à un débit fixé, un [[ui:st.tcp]]
ouvre une connexion, écrit la charge utile et ferme, encore et encore. Le débit
est mesuré en direct. Voir [Tempête](../tools/storm.md).

## Scanner {#scanner}

[[ui:nav.scan]] essaie une connexion TCP vers chaque port d’une plage et liste ceux
qui acceptent, avec ce que le service dit en premier si vous demandez des bannières. Voir
[Scanner](../tools/scanner.md).

::: danger
Tempête et Scanner émettent du vrai trafic vers de vrais hôtes. Ne les dirigez que vers des systèmes
que vous possédez ou êtes autorisé à tester : une tempête peut saturer une liaison, et les deux peuvent déclencher
une détection d’intrusion.
:::

## Appareils émulés {#emulators}

Sur l’écran [[ui:nav.emulators]], Signal Lab peut être l’appareil :

- un [[ui:emu.new.udp]] répond aux datagrammes par des règles sur leur charge utile — n’importe lequel,
  contenant un texte, correspondant à une expression régulière, contenant des octets — avec une
  réponse texte ou hex construite à partir de ce qui est arrivé, vers l’expéditeur ou vers un autre
  `IP:port`, après un délai si vous en définissez un ;
- un [[ui:emu.new.tcp]] accepte les connexions, découpe ce qui arrive en messages
  à une fin de ligne que vous choisissez (LF, CR LF, CR, ou chaque morceau dès qu’il arrive),
  répond à chacun par le même genre de règles, peut envoyer un accueil quand un client
  se connecte, et peut fermer la connexion après une réponse.

Les deux peuvent aussi tourner comme étape [[ui:exp.node.emulator]] pendant toute la durée d’une exécution.
Voir [Émulateurs](../tools/emulators.md).

## Dégradation {#impairment}

Le relais [[ui:nav.netsim]] se tient entre un client et sa cible et dégrade
ce qui passe : par datagramme en UDP (latence, perte, doublons, réordonnancement, une
limite de bande passante) ou par flux en TCP (latence, limite de bande passante, connexions
réinitialisées ou laissées semi-ouvertes). Voir [Dégradation](../tools/impairment.md) et, dans
une expérience, [Défaillances](../experiments/faults.md).

## « Port unreachable » sous Windows {#port-unreachable}

Quand un datagramme atteint un port où rien n’écoute, la machine réceptrice
répond habituellement par un message ICMP « port unreachable ». Windows rapporte cette
réponse à la prochaine réception du socket émetteur, comme si la connexion avait été
réinitialisée — alors qu’UDP n’a pas de connexion.

Signal Lab s’y attend. Ses écouteurs — le moniteur OSC, l’écouteur de découverte,
les attentes et réponses d’expérience, les émulateurs UDP et OSC, le relais de dégradation —
le notent et continuent d’écouter. Un appareil qui a disparu ne les arrête pas. Un écouteur qui
ne peut vraiment plus rien recevoir termine sa tâche, et la
console indique pourquoi.

## Problèmes {#troubleshooting}

| Ce que vous voyez | Cause habituelle |
| --- | --- |
| `… is not a valid address` | La cible n’a pas de port, ou n’est ni `IP:port` ni `host:port`. |
| `Cannot resolve …` | Le nom d’hôte ne se résout pas sur cet ordinateur. |
| `… refused the connection — nothing is listening on that port` (TCP) | Rien n’écoute sur ce port, ou un pare-feu le rejette. |
| `No answer from … in time` (TCP) | L’hôte ne répond pas du tout — mauvaise adresse, ou un pare-feu qui abandonne au lieu de rejeter. |
| Une attente expire alors que l’appareil répond | L’appareil répond au port d’où le datagramme est venu, pas au port de l’attente. Laissez l’envoi attendre lui-même la réponse avec [[ui:exp.expectReply]] : il part alors du port où la réponse revient. |
| Les datagrammes d’autres machines n’arrivent jamais | Sous Windows, le pare-feu peut les retenir : autorisez Signal Lab quand l’application le propose. Voir [Dépannage](../reference/troubleshooting.md). |

Chaque message d’erreur est listé dans [Messages d’erreur](../reference/errors.md#transport).

---
title: "MQTT"
description: "Se connecter à un broker MQTT 3.1.1, suivre chaque topic qu'il contient sous forme d'arbre en direct, publier en QoS 0, 1 ou 2, annoncer une dernière volonté et effacer les valeurs retenues bloquées."
---

# MQTT

L'écran [[ui:nav.mqtt]] est un client MQTT pour regarder un broker et modifier ce
qu'il contient. Connectez-vous, et par défaut il s'abonne à `#` : chaque
topic que le broker détient s'accumule en un arbre avec sa dernière valeur. De là, vous
publiez, effacez une valeur retenue, enregistrez un topic comme signal ou le transformez en
étape d'expérience.

Signal Lab parle **MQTT 3.1.1 sur TCP simple**, avec QoS 0, 1 et 2 pour
s'abonner, publier et la dernière volonté. Il n'y a pas de MQTT 5 ni de TLS : un
broker qui n'accepte que des clients `mqtts://` ou MQTT 5 ne peut pas être joint.

## Connexion {#connect}

1. Ouvrez [[ui:nav.mqtt]].
2. Saisissez [[ui:mq.host]] et [[ui:common.port]].
3. Laissez [[ui:mq.clientId]] tel quel, à moins que le broker n'en attende un précis.
   Ajoutez [[ui:mq.username]] et [[ui:mq.password]] seulement si le broker les demande.
4. Appuyez sur [[ui:mq.connect]].

Se connecter ouvre la connexion TCP et termine la poignée de main MQTT avant
tout le reste : un mot de passe erroné ou un port fermé est donc signalé tout de suite. Les
champs de connexion se verrouillent une fois connecté ; [[ui:mq.disconnect]] la ferme. La
connexion est une tâche dans le bandeau de la console et peut y être arrêtée aussi.

| Champ | Quoi | Par défaut |
| --- | --- | --- |
| [[ui:mq.host]] | L'adresse IP ou le nom d'hôte du broker | `127.0.0.1` |
| [[ui:common.port]] | Le port du broker | `1883` |
| [[ui:mq.clientId]] | Le nom de votre client auprès du broker. Il ne doit pas être vide et doit y être unique : un second client avec le même identifiant déloge le premier. | `signal-lab-` et six chiffres hexadécimaux aléatoires, nouveaux à chaque démarrage de l'application |
| [[ui:mq.username]], [[ui:mq.password]] | Envoyés seulement si le broker en a besoin — en clair, puisqu'il n'y a pas de TLS. Un mot de passe sans nom d'utilisateur n'est pas envoyé du tout : MQTT 3.1.1 ne peut pas le transporter. | vide |
| [[ui:mq.keepAlive]] | Secondes pendant lesquelles la connexion peut rester silencieuse. Signal Lab envoie un ping au broker toutes les moitiés de cette durée ; un broker déconnecte un client resté silencieux 1,5 fois cette durée. 0 désactive les pings. | `60` |
| [[ui:mq.cleanSession]] | Activé : chaque connexion démarre sans abonnement conservé ni message en file. Désactivé : le broker doit les conserver pour cet identifiant client entre les connexions. | activé |
| [[ui:mq.scanFilter]] et son [[ui:mq.qos]] | Un filtre auquel s'abonner dès que la connexion est établie ; `#` correspond à tous les topics. Vide : aucun. | `#`, QoS 0 |
| [[ui:mq.willEnable]] | Donner au broker une dernière volonté ([ci-dessous](#will)) | désactivé |

Le broker a 6 secondes pour accepter la connexion TCP et 6 de plus pour répondre à la
poignée de main.

### La dernière volonté {#will}

Une dernière volonté est un message que le broker garde pour vous et publie lui-même si votre
connexion meurt sans adieu convenable. La présence se construit souvent ainsi : un
appareil publie `online` sur son topic d'état, et sa volonté met le même topic
à `off`.

Avec [[ui:mq.willEnable]] coché, définissez [[ui:mq.willTopic]] et
[[ui:mq.willPayload]] (`off` par défaut). La volonté est publiée en QoS 2 et
retenue. Sans topic, aucune volonté n'est envoyée.

## S'abonner {#subscribe}

Le filtre de balayage est souscrit à la connexion. Pour en ajouter d'autres :

1. Dans [[ui:mq.addSubscription]], saisissez un filtre de topic.
2. Choisissez son [[ui:mq.qos]].
3. Appuyez sur [[ui:mq.subscribe]] ou sur <kbd>Enter</kbd>.

Un filtre est un topic avec des jokers :

| Joker | Représente | Exemple |
| --- | --- | --- |
| `+` | exactement un niveau | `sensors/+/state` correspond à `sensors/door/state` |
| `#` | tous les niveaux en dessous, en dernier caractère seulement | `sensors/#` correspond à `sensors/door/state` et `sensors` |

[[ui:mq.subscriptions]] liste chaque filtre avec ce que le broker a accordé :
`qos0`, `qos1` ou `qos2` — le broker peut accorder moins que ce que vous avez demandé — ou
[[ui:mq.refused]]. [[ui:mq.unsubscribe]] se désabonne d'un filtre.

| QoS | Remise |
| --- | --- |
| 0 | Au plus une fois : envoyé et oublié |
| 1 | Au moins une fois : acquitté, peut arriver deux fois |
| 2 | Exactement une fois : une poignée de main en deux temps ; une rediffusion n'est pas affichée deux fois |

## L'arbre des topics {#topics}

Chaque message qui arrive entre dans [[ui:mq.topics]], un arbre des niveaux de topic.
Un topic affiche sa dernière valeur, un **R** quand cette valeur est retenue,
et le nombre de messages qu'il a reçus quand il y en a plus d'un. Cliquez sur un niveau pour l'ouvrir ou
le fermer.

- Saisissez du texte dans le champ au-dessus de l'arbre pour ne lister que les topics dont le chemin ou la dernière
  valeur contient ce texte.
- Au-dessus de l'arbre figurent le nombre de topics, combien détiennent une valeur retenue et,
  une fois connecté, le broker sur lequel vous écoutez.
- Les charges utiles sont affichées en texte ; les octets qui ne sont pas de l'UTF-8 apparaissent en caractères
  de remplacement.
- [[ui:common.clear]] vide l'arbre. Rien d'autre ne le fait : il reste tel quel
  quand vous changez d'écran ou vous vous déconnectez, jusqu'à la fermeture de l'application.

Les messages atteignent l'écran par lots, dix fois par seconde. Quand un broker envoie
plus de 4000 messages en un dixième de seconde, les plus anciens de ce lot sont
laissés hors de l'arbre et comptés comme « non affichés » au-dessus de lui.

### Le volet d'un topic {#topic}

Choisissez un topic avec une valeur pour le voir sous l'arbre : [[ui:mq.value]],
[[ui:mq.qos]], [[ui:mq.retain]], [[ui:common.bytes]], [[ui:mq.messages]] et
[[ui:mq.lastAt]]. Ses boutons :

| Bouton | Fait |
| --- | --- |
| [[ui:mq.editHere]] | Copie le topic, la valeur, le QoS et l'indicateur retain dans [[ui:mq.publish]] |
| [[ui:mq.waitForThis]] | Ajoute une étape [Attendre MQTT](../experiments/nodes.md#node-wait_mqtt) sur ce topic, chez ce broker, n'importe quelle charge utile, délai de 2000 ms, à l'expérience ouverte |
| [[ui:mq.clearRetained]] | Retire la valeur retenue ([ci-dessous](#clear-retained)) |
| [[ui:sig.fromFrame]] | Garde le topic et sa dernière valeur comme signal dans le dossier [[ui:sig.capturedFolder]] |

## Publier {#publish}

1. Connectez-vous.
2. Sous [[ui:mq.publish]], saisissez le [[ui:mq.topic]] et la
   [[ui:sig.payload]].
3. Choisissez le [[ui:mq.qos]], et cochez [[ui:mq.retain]] si le broker doit garder
   le message comme valeur du topic pour chaque client qui s'abonnera plus tard.
4. Appuyez sur [[ui:mq.publishBtn]].

La console confirme chaque publication : aussitôt pour le QoS 0, quand le broker l'a
acquittée pour les QoS 1 et 2. Un topic de publication n'a pas de jokers et
n'est pas vide : un topic contenant `+` ou `#` est refusé avant que quoi que ce soit ne soit envoyé, avec
le même message qu'un signal, une étape et `signallab send mqtt`, et la
connexion reste telle quelle. Seul un abonnement prend des filtres à jokers.

### Effacer une valeur retenue {#clear-retained}

Une valeur retenue reste sur le broker jusqu'à ce qu'elle soit remplacée, et chaque client
qui s'abonne la reçoit en premier — une valeur périmée est une raison classique qu'un appareil démarre
dans le mauvais état. La seule façon de la retirer est de publier une charge utile vide
avec retain activé.

[[ui:mq.clearRetained]] dans le volet d'un topic fait cela : appuyez dessus, puis sur
[[ui:mq.clearConfirm]]. Il publie la charge utile retenue vide en QoS 1 sur
votre connexion. Il n'est disponible que connecté et quand la dernière valeur
du topic est retenue. Vous pouvez faire de même à la main : une
[[ui:sig.payload]] vide avec [[ui:mq.retain]] cochée.

::: warning
Effacer modifie le broker pour tous les clients à la fois.
:::

## Dans l'Inspecteur {#inspector}

Avec la capture active, le trafic MQTT apparaît avec le protocole `mqtt` :

| Source | Quoi | Combien |
| --- | --- | --- |
| `mqtt` | Ce que la connexion de l'écran publie ; une publication retenue vide a le verdict `clears retained` | chacune |
| `mqtt` | Les messages que la connexion reçoit | au plus un toutes les 200 ms |
| `mqtt-send` | Une publication qui a ouvert sa propre connexion : un signal envoyé alors que l'écran n'est pas connecté au broker du signal, une étape, `signallab send mqtt` (verdict `one-shot`) | chacune |
| `experiment-wait` | Les messages que reçoit l'abonnement d'une étape [[ui:exp.node.wait_mqtt]], rediffusions retenues mises à part | chacun |

Le résumé se lit `topic = payload`, avec le QoS et `retained` quand ils
s'appliquent. Voir [Inspecteur](../tools/inspector.md).

## Enregistrer et réutiliser {#library}

- **Enregistrer comme signal.** [[ui:sig.saveNew]] sous [[ui:mq.publish]] garde le
  broker (le [[ui:mq.host]] et le [[ui:common.port]] de la connexion), le topic,
  la charge utile, le QoS et l'indicateur retain dans la bibliothèque de signaux ;
  <kbd>Ctrl</kbd>+<kbd>S</kbd> dans le volet de publication fait de même, et
  met à jour le signal une fois le volet lié à lui. Voir
  [Signaux](../tools/signals.md).
- **Envoyer un signal MQTT.** Tant que cet écran est connecté au broker que
  nomme le signal (même hôte, casse ignorée, et même port ; `1883` quand le signal
  n'en donne aucun), un signal envoyé depuis la bibliothèque part par cette connexion,
  avec son identifiant client et ses identifiants. Sinon — non connecté, ou connecté à
  un autre broker — il ouvre une connexion à lui vers son propre broker — un identifiant
  client neuf, pas de nom d'utilisateur — publie, attend l'acquittement que réclame son QoS,
  et se déconnecte. Les noms ne sont pas résolus, donc `localhost` et
  `127.0.0.1` comptent comme des brokers différents. La bibliothèque ne conserve aucun mot de passe.
- **Dans une expérience.** Un signal MQTT enregistré peut être choisi sous
  [[ui:exp.group.signals]] dans le menu [[ui:exp.addNode]] de l'expérience, ce qui
  en fait une étape [[ui:exp.node.mqtt]].

## Dans les expériences {#experiments}

| Étape | Ce qu'elle fait |
| --- | --- |
| [[ui:exp.node.mqtt]] | Se connecte, publie un message et se déconnecte — pas de nom d'utilisateur ni de mot de passe, une session propre, en 15 secondes. [Détails](../experiments/nodes.md#node-mqtt) |
| [[ui:exp.node.wait_mqtt]] | S'abonne au démarrage de l'exécution et attend un message sur un filtre de topic dont la charge utile correspond ; les valeurs retenues rejouées à l'abonnement sont ignorées. [Détails](../experiments/nodes.md#node-wait_mqtt) |
| [[ui:exp.node.emulator]] | Un broker MQTT propre à l'exécution. [Détails](../experiments/nodes.md#node-emulator) |

Aucune des deux étapes ne s'authentifie, il leur faut donc un broker qui accepte les clients sans nom
d'utilisateur.

### Le broker émulé {#broker-emulator}

Signal Lab peut aussi être le broker : un émulateur [[ui:emu.new.mqtt]] achemine ce que les
clients publient vers qui s'y est abonné — 3.1.1, TCP simple, QoS 0, 1 et 2,
messages retenus, dernières volontés, une authentification facultative — et répond selon des règles, comme un
appareil. Dirigez vos équipements et cet écran vers lui pour tester sans vrai broker.
Voir [Émulateurs](../tools/emulators.md).

## Depuis la ligne de commande {#cli}

`signallab send mqtt` publie un message avec une connexion à lui :

```bash
signallab send mqtt 127.0.0.1:1883 lab/light/1/set on --qos 1
signallab send mqtt 127.0.0.1:1883 lab/light/1/state "" --retain
```

```text
✔ lab/light/1/set → 127.0.0.1:1883 · 2 B · qos1
```

La deuxième ligne efface une valeur retenue. Sans port, le broker est sur
1883. Il n'utilise aucun identifiant. Il se termine avec 0 quand le broker a pris le message,
1 quand il n'a pas pu être joint ou l'a refusé. Voir
[Ligne de commande](../automation/cli.md#cli-send-mqtt).

## Problèmes {#troubleshooting}

| Ce que vous voyez | Cause habituelle |
| --- | --- |
| `… refused the connection — nothing is listening on that port` | Aucun broker à cette adresse et ce port. |
| `… accepted the connection but did not answer in time — is it an MQTT broker?` | Quelque chose écoute là, mais ne parle pas MQTT, ou le parle en TLS. |
| `… answered with something other than MQTT 3.1.1` | Ce n'est pas un broker MQTT, ou un broker qui a envoyé quelque chose que Signal Lab ne peut pas lire. |
| `… does not accept MQTT 3.1.1 clients` | Le broker n'accepte que MQTT 5. |
| `… rejected the client ID — choose another one` | L'identifiant est trop long ou contient des caractères que le broker n'accepte pas. |
| `… rejected the username or password` | Identifiants erronés, ou un mot de passe sans nom d'utilisateur. |
| `… did not authorize this client — check its access rules` | Les règles d'accès du broker refusent ce client. |
| `… is unavailable right now — try again later` | Le broker est en marche mais n'accepte pas de clients. |
| `Enter a client ID — brokers refuse an empty one` | [[ui:mq.clientId]] est vide. |
| `A publish topic cannot contain the wildcards + or #` | Le topic de publication contient un `+` ou un `#`. Ils servent à s'abonner ; publiez sur un topic à la fois. |
| Un filtre affiche [[ui:mq.refused]] | Les règles d'accès du broker l'interdisent, ou le filtre est mal formé (`#` pas en dernier, `+` partageant un niveau avec d'autres caractères). |
| La connexion tombe un instant après la connexion | Un autre client s'est connecté avec le même [[ui:mq.clientId]]. |
| Rien n'apparaît dans l'arbre | Le filtre de balayage est vide, ou le broker ne laisse rien voir à ce client. |

Chaque message d'erreur est listé dans [Messages d'erreur](../reference/errors.md#mqtt).

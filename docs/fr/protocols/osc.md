---
title: OSC
description: "Envoyez des messages Open Sound Control typés, observez ce qui arrive sur un port et pilotez une forme d’onde continue vers un appareil depuis l’écran OSC."
---

# OSC

L’écran [[ui:nav.osc]] est l’endroit où vous parlez Open Sound Control (OSC 1.0) sur
UDP à la main. Il a trois parties :

- [[ui:osc.sender]] : un message, des arguments typés, envoyé quand vous appuyez sur Entrée.
- [[ui:osc.monitor]] : écoute sur un port et décode chaque paquet qui arrive.
- [[ui:osc.generator]] : envoie une valeur qui suit une forme d’onde, plusieurs fois par
  seconde, et la dessine.

Signal Lab encode et décode OSC lui-même. Ce qu’il envoie est ce que
l’[Inspecteur](../tools/inspector.md) montre, octet pour octet.

## Envoyer un message {#send}

1. Ouvrez [[ui:nav.osc]].
2. Dans [[ui:common.target]], saisissez l’adresse IP ou le nom d’hôte de l’appareil et son
   port, par exemple `127.0.0.1:9000` ou `stage-mixer.local:9000`.
3. Dans [[ui:common.address]], saisissez l’adresse que l’appareil écoute, par
   exemple `/mixer/fader/1`.
4. Sous [[ui:common.arguments]], réglez le type et la valeur de chaque argument. Appuyez sur
   [[ui:common.addArgument]] pour en ajouter un ; ✕ en supprime un.
5. Appuyez sur [[ui:common.send]], ou sur <kbd>Enter</kbd> dans n’importe quel champ de l’envoi.

La ligne sous les boutons indique ce qui est parti : l’adresse, sa taille en octets
et la cible. Renvoyer le même message compte les répétitions (×2, ×3…), pour que vous
voyiez qu’un envoi répété a fait quelque chose. Un échec s’affiche là à la place, et
dans la console.

La cible, l’adresse et les arguments sont conservés quand vous changez d’écran et quand
vous redémarrez l’application.

### Champs {#send-fields}

| Champ | Quoi | Par défaut |
| --- | --- | --- |
| [[ui:common.target]] | `IP:port` ou `host:port` du récepteur. Une adresse IPv6 se met entre crochets : `[::1]:9000`. Un nom d’hôte est résolu à chaque envoi ; quand il a une adresse IPv4, c’est elle qui est utilisée (si bien que `localhost` atteint un récepteur qui écoute sur `127.0.0.1`), sinon son adresse IPv6. Une cible sans port est refusée. | `127.0.0.1:9000` |
| [[ui:common.address]] | L’adresse OSC, commençant par `/`, les parties séparées par `/`. Une adresse sans le `/` initial est refusée avant que quoi que ce soit ne soit envoyé. | `/hello/avatar/1` |
| [[ui:common.arguments]] | Des valeurs typées après l’adresse, dans l’ordre. Un message peut n’en avoir aucune. | un `float`, `1.0` |

### Types d’arguments {#types}

Le type de chaque argument fait partie du message (sa balise de type), si bien qu’un appareil
qui attend un float peut ignorer un int de même valeur.

| Type dans la liste | Balise OSC | Valeur | Comment vous la saisissez |
| --- | --- | --- | --- |
| `int` | `i` | entier signé 32 bits | un nombre entier |
| `float` | `f` | virgule flottante 32 bits | un nombre, `0.75` |
| `str` | `s` | texte | n’importe quel texte, envoyé en UTF-8 |
| `bool` | `T` / `F` | vrai ou faux | `true` ou `false` dans une liste ; ne porte aucun octet, seulement la balise |
| `long` | `h` | entier signé 64 bits | un nombre entier |
| `double` | `d` | virgule flottante 64 bits | un nombre |
| `nil` | `N` | rien | aucune valeur |
| `blob` | `b` | octets | pas saisi ici : il apparaît, en lecture seule et en hex, quand vous ouvrez un signal qui en a un |

Un champ numérique qui ne contient pas un nombre envoie `0`.

::: tip
Un `true` OSC est la balise `T`, pas le texte `"true"`. Un appareil qui attend un
bool ignore silencieusement une chaîne.
:::

## Bundles {#bundles}

L’envoi envoie des messages seuls, pas des bundles. Quand un bundle (`#bundle`)
arrive, le moniteur, les attentes d’expérience et les émulateurs le décomposent : chaque
message qu’il contient est traité à part, et son horodatage est ignoré.

## Surveiller un port {#monitor}

Pour voir ce qu’envoie un appareil ou une console de spectacle :

1. Dans [[ui:common.bind]], saisissez l’adresse et le port sur lesquels écouter. `0.0.0.0:9000`
   (par défaut) écoute sur toutes les cartes réseau ; `127.0.0.1:9000` seulement sur cet
   ordinateur.
2. Appuyez sur [[ui:osc.listen]]. Le champ se verrouille pendant que le moniteur tourne.
3. Dirigez l’envoi vers l’adresse IP de cet ordinateur et ce port.

Chaque paquet devient une ligne, la plus récente en haut :

| Colonne | Quoi |
| --- | --- |
| [[ui:common.time]] | Quand il est arrivé, à la milliseconde |
| [[ui:osc.from]] | L’`IP:port` de l’expéditeur |
| [[ui:osc.address]] | L’adresse OSC, ou [[ui:osc.decodeError]] quand le paquet n’est pas de l’OSC valide |
| [[ui:osc.args]] | Les valeurs des arguments ; un blob s’affiche comme `blob[n]`, `nil` comme `nil`. Pour un paquet qui n’a pas pu être décodé, pourquoi. |

Un bundle donne une ligne par message. La liste garde les 300 dernières lignes ;
[[ui:common.clear]] la vide. Appuyez sur [[ui:common.stop]] pour fermer le port. Le
moniteur est aussi une tâche dans le bandeau de la console, et peut donc être arrêté de là.

Le moniteur lit des paquets jusqu’à 64 Kio. Il décode les balises `i f s S b h d T F N I`
(`S` se lit comme du texte, `I` comme nil) ; un paquet avec une autre balise, ou tronqué, est
affiché comme une erreur de décodage plutôt que supprimé.

### Transformer un message en attente {#wait-for-this}

Chaque ligne a un bouton ⇠, [[ui:osc.waitForThis]]. Il ajoute une étape
[Attente OSC](../experiments/nodes.md#node-wait_osc) à l’expérience ouverte qui écoute sur le [[ui:common.bind]] du moniteur pour cette adresse,
avec une règle « égal à » pour chaque argument texte, entier et vrai/faux —
les flottants, les blobs et nil n’en ont aucune — jusqu’à 16 règles. Son délai d’attente est de 2000 ms. L’éditeur
s’ouvre avec la nouvelle étape sélectionnée.

::: warning
Arrêtez le moniteur avant d’exécuter cette expérience. L’exécution ouvre elle-même le même port,
et deux écouteurs ne peuvent pas le partager.
:::

## Piloter une forme d’onde {#generator}

Le [[ui:osc.generator]] envoie un message après l’autre vers une adresse, avec un
seul argument dont la valeur suit une forme d’onde — un fader, un niveau de lumière, une
position. Utilisez-le pour voir comment un appareil suit une valeur qui bouge, ou pour charger un
récepteur avec un flux régulier.

1. Réglez [[ui:common.target]] et [[ui:osc.address]]. Les deux sont vérifiés quand vous
   appuyez sur [[ui:osc.startGen]], comme l’envoi les vérifie.
2. Choisissez une [[ui:osc.waveform]], sa [[ui:osc.freq]] et la [[ui:osc.rate]].
3. Réglez [[ui:osc.min]] et [[ui:osc.max]], l’étendue de la valeur.
4. Appuyez sur [[ui:osc.startGen]]. Elle tourne jusqu’à ce que vous appuyiez sur [[ui:osc.stopGen]] ou que vous arrêtiez
   sa tâche dans le bandeau de la console.

| Champ | Quoi | Par défaut |
| --- | --- | --- |
| [[ui:common.target]] | `IP:port` ou `host:port` du récepteur ; un nom est résolu une fois, au démarrage du générateur | `127.0.0.1:9000` |
| [[ui:osc.address]] | L’adresse à laquelle chaque message est envoyé ; elle commence par `/` | `/hello/lfo` |
| [[ui:osc.waveform]] | La forme de la valeur dans le temps (ci-dessous) | [[ui:wave.sine]] |
| [[ui:osc.freq]] | Cycles de la forme d’onde par seconde | `1` |
| [[ui:osc.rate]] | Messages par seconde, de 0,1 à 5000 ; une valeur hors de cet intervalle y est ramenée | `60` |
| [[ui:osc.min]], [[ui:osc.max]] | La valeur la plus basse et la plus haute. Si Max est inférieur à Min, la valeur ne bouge pas. | `0`, `1` |
| [[ui:osc.asInt]] | Arrondir au nombre entier le plus proche et envoyer un `int` au lieu d’un `float` | désactivé |

| Forme d’onde | Ce que fait la valeur à chaque cycle |
| --- | --- |
| [[ui:wave.sine]] | Oscille en douceur entre Min et Max, en partant du milieu et en montant |
| [[ui:wave.triangle]] | Monte de Min à Max, puis redescend à Min, en partant de Min |
| [[ui:wave.saw]] | Une dent de scie descendante : part de Max, descend à Min, puis saute de nouveau à Max |
| [[ui:wave.ramp]] | Une dent de scie montante : part de Min, monte à Max, puis saute de nouveau à Min |
| [[ui:wave.square]] | Max pendant la première moitié, Min pendant la seconde |
| [[ui:wave.random]] | Une nouvelle valeur aléatoire entre Min et Max à chaque message ; la fréquence n’est pas utilisée |
| [[ui:wave.constant]] | Max, à chaque fois ; la fréquence n’est pas utilisée |

### L’oscilloscope {#scope}

À côté des champs, l’oscilloscope dessine la valeur telle qu’elle est envoyée : les 300 derniers
points, mis à l’échelle. En dessous se trouvent la forme d’onde, la fréquence et le débit, et la
dernière valeur envoyée. L’oscilloscope est mis à jour environ 30 fois par seconde quelle que soit la vitesse du
générateur, donc à haut débit il montre un échantillon des messages, pas chacun d’eux.

Si un envoi échoue, le générateur s’arrête et la console indique pourquoi.

## Dans l’Inspecteur {#inspector}

Quand la capture est armée ([[ui:ins.arm]] dans [[ui:dock.inspector]]), le trafic OSC
apparaît avec le protocole `osc` :

| Source | Quoi | Remarques |
| --- | --- | --- |
| `osc-send` | Chaque message qu’envoient l’envoi, un signal de bibliothèque, une étape [[ui:exp.node.osc]] ou `signallab send osc` | Une étape qui attend une réponse s’affiche comme `experiment` |
| `osc-monitor` | Chaque paquet que le moniteur reçoit | Un bundle est résumé par son premier message et `+n more in bundle` ; un paquet malformé a le verdict `decode error: …` |
| `osc-gen` | Les messages du générateur | Au plus un toutes les 100 ms est capturé, avec le verdict `sampled` ; le suivant après des messages sautés ajoute combien n’ont pas été dessinés, comme `sampled · +5 not shown` |

Chaque trame garde les octets à partir desquels elle a été construite. Voir [Inspecteur](../tools/inspector.md).

## Enregistrer et réutiliser {#library}

- **Enregistrer comme signal.** [[ui:sig.saveNew]] sous les boutons conserve le message —
  cible, adresse et arguments — dans la bibliothèque de signaux, dans un dossier que vous choisissez.
  À partir de là, l’envoi est lié à ce signal : [[ui:sig.save]] (ou
  <kbd>Ctrl</kbd>+<kbd>S</kbd> dans l’envoi) le met à jour, [[ui:sig.saveAs]]
  en fait une copie, et la pastille à côté l’ouvre dans [[ui:nav.signals]]. Renvoyez-le
  plus tard depuis [[ui:nav.signals]] ou avec <kbd>Ctrl</kbd>+<kbd>K</kbd> depuis n’importe
  quel écran. Voir [Signaux](../tools/signals.md).
- **Ajouter à une expérience.** [[ui:common.toExperiment]] ajoute une étape
  [Message OSC](../experiments/nodes.md#node-osc) avec la même cible,
  adresse et arguments à l’expérience ouverte — juste avant Fin, ou après l’étape
  sélectionnée — et l’ouvre. Pendant que l’expérience tourne, rien ne peut être ajouté ;
  la console le dit.

## Dans les expériences {#experiments}

| Étape | Ce qu’elle fait |
| --- | --- |
| [[ui:exp.node.osc]] | Envoie un message. Sa cible, son adresse et ses arguments texte prennent des `{{templates}}`. Avec [[ui:exp.expectReply]] il envoie depuis un port propre et attend là la réponse dans la même étape. [Détails](../experiments/nodes.md#node-osc) |
| [[ui:exp.node.wait_osc]] | Attend un message dont l’adresse correspond à un motif et dont les arguments passent les règles. [Détails](../experiments/nodes.md#node-wait_osc) |
| [[ui:exp.node.emulator]] | Un appareil OSC qui répond selon des règles pendant toute l’exécution. Voir [Émulateurs](../tools/emulators.md). |

L’étape de message OSC prend un `IP:port` ou un `host:port` comme l’écran, et son
adresse doit commencer par `/`. Un nom d’hôte est résolu à chaque fois que l’étape envoie.

### Motifs d’adresse {#patterns}

[[ui:exp.node.wait_osc]], la réponse de [[ui:exp.node.osc]] et les règles d’un
émulateur OSC font correspondre les adresses avec les motifs OSC 1.0 :

| Motif | Correspond à |
| --- | --- |
| `*` | n’importe quelle suite de caractères, y compris aucune |
| `?` | exactement un caractère |
| `[0-9]`, `[a-c]` | un caractère de l’ensemble ou de l’intervalle |
| `[!0-9]` | un caractère hors de l’ensemble |
| `{ping,pong}` | l’un des mots |

Les jokers restent à l’intérieur d’une partie entre les barres obliques : `/cue/*` correspond à `/cue/7` mais
pas à `/cue/7/go`, et un motif ne correspond qu’à une adresse ayant le même nombre de
parties. La correspondance est sensible à la casse. Un motif commence par `/`, n’a pas de partie vide
(`//`), pas d’espaces, pas de `#` et pas de caractères hors ASCII, et fait au plus 512
caractères.

Les règles d’arguments comparent le numéro d’argument 0–63 avec une valeur (égal à, inférieur à,
contient, correspond à une expression régulière, etc.) ; une attente en a au plus 16. Quand
un bundle arrive, l’attente le prend si un message qu’il contient correspond. Voir
[Données et modèles](../experiments/data.md) pour ce qu’un message mis en correspondance donne
aux étapes qui le suivent.

## Depuis la ligne de commande {#cli}

`signallab send osc` envoie un message comme l’envoi le fait :

```bash
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main
```

```text
✔ sent /cue/go (24 bytes) → 127.0.0.1:9000
```

Chaque argument est `tag:value` : `i:3`, `f:0.5`, `d:1.5`, `h:64`, `s:text`,
`b:de ad be ef` (octets en hex), ou `T`, `F`, `N` seuls. Sans balise, un
nombre entier est `i`, un nombre avec un point décimal est `f`, et tout le reste est
`s` ; écrivez `s:7` pour envoyer le texte `7`. La cible est `IP:port` ou `host:port`. Il
sort avec 0 quand le message est parti, 1 quand l’envoi a échoué (un nom d’hôte qui
ne se résout pas compris), et 2 quand un argument, l’adresse ou la cible
est invalide. [`signallab fire`](../automation/cli.md#cli-fire) envoie un signal enregistré. Voir
[Ligne de commande](../automation/cli.md#cli-send-osc).

::: tip
Dans Git Bash sous Windows, un argument qui commence par `/` est transformé en chemin de fichier
avant que `signallab` ne le voie. Lancez la commande avec `MSYS_NO_PATHCONV=1` devant,
ou utilisez PowerShell ou `cmd`.
:::

## Problèmes {#troubleshooting}

| Ce que vous voyez | Cause habituelle |
| --- | --- |
| `… is not a valid address` à l’envoi | La cible n’a pas de port, ou n’est ni `IP:port` ni `host:port`. |
| `Cannot resolve …` à l’envoi | Le nom d’hôte ne se résout pas sur cet ordinateur. Vérifiez-le, ou utilisez l’adresse IP. |
| `OSC addresses start with / (…)` | L’adresse n’a pas de `/` initial. |
| Le message est envoyé mais l’appareil ne fait rien | Mauvais port ou adresse ; un type différent de celui qu’il attend (`int` au lieu de `float`, un texte `"true"` au lieu d’un bool). Regardez-le dans l’Inspecteur, ou dirigez la cible vers le moniteur de cet ordinateur pour voir ce qui part. |
| `… is already in use by another program` sur [[ui:osc.listen]] | Un autre programme — ou une expérience, un émulateur ou un second moniteur en cours — a le port. |
| `… is not an address of this computer` | L’IP de [[ui:common.bind]] appartient à une autre machine. Utilisez `0.0.0.0` ou l’une des adresses de cet ordinateur. |
| Les paquets d’autres machines n’arrivent jamais | Sous Windows, le pare-feu peut les retenir : autorisez Signal Lab quand l’application le propose. Le trafic local (`127.0.0.1`) n’est pas concerné. Voir [Dépannage](../reference/troubleshooting.md). |
| Des lignes [[ui:osc.decodeError]] | L’expéditeur ne parle pas OSC 1.0 sur ce port, ou utilise une balise de type que Signal Lab ne décode pas. |

Sur un serveur, l’écran travaille sur le réseau du serveur : `127.0.0.1` est le
serveur lui-même, et le moniteur écoute sur les ports du serveur. Voir
[Serveur](../server/index.md).

Chaque message d’erreur est listé dans [Messages d’erreur](../reference/errors.md#transport).

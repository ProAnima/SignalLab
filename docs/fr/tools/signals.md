---
title: "Signaux"
description: "Conservez les messages OSC, UDP, HTTP et MQTT dans une bibliothèque de dossiers, et renvoyez-les depuis l’écran Signaux, depuis n’importe quel écran avec Ctrl+K, ou depuis la ligne de commande."
---

# Signaux

Un signal est un message que vous avez nommé et conservé : un message OSC, un
datagramme UDP brut, une requête HTTP ou une publication MQTT, avec sa cible.
Vous le construisez une fois — sur l’écran [[ui:nav.signals]], ou en enregistrant
ce que vous venez d’envoyer depuis un autre écran — puis le renvoyez quand vous en
avez besoin, octet pour octet à l’identique.

La bibliothèque est un fichier JSON que vous pouvez lire, modifier à la main,
copier sur une autre machine ou versionner à côté d’un projet. L’écran
[[ui:nav.signals]] l’affiche sous forme d’un arbre de dossiers à gauche et des
champs du signal sélectionné à droite.

## Ce qu’un signal peut envoyer {#transports}

Choisissez le type dans [[ui:sig.transport]]. Chaque type a ses propres champs :

| [[ui:sig.transport]] | Champs | Ce qui part |
| --- | --- | --- |
| [[ui:sig.tr.osc]] | [[ui:common.target]], [[ui:common.address]], [[ui:common.arguments]] | Un message OSC vers un `IP:port` ou `host:port`, depuis un port UDP neuf. Voir [OSC](../protocols/osc.md#types) pour les types d’arguments. |
| [[ui:sig.tr.udp]] | [[ui:common.target]], [[ui:sig.payloadKind]] ([[ui:sig.payloadText]] ou [[ui:sig.payloadHex]]), [[ui:sig.payload]] | Un datagramme avec exactement ces octets. Le texte part tel qu’écrit, sans terminateur ; l’hexadécimal est par paires de chiffres comme `de ad be ef` (les espaces et un préfixe `0x` sont autorisés). |
| [[ui:sig.tr.http]] | [[ui:sig.method]], [[ui:sig.url]], [[ui:sig.timeout]], [[ui:sig.headers]], [[ui:field.auth]], [[ui:sig.body]] | Une requête HTTP. Voir [HTTP](../protocols/http.md). |
| [[ui:sig.tr.mqtt]] | [[ui:mq.broker]], [[ui:mq.topic]], [[ui:mq.qos]], [[ui:mq.retain]], [[ui:sig.payload]] | Une publication. Voir [MQTT](../protocols/mqtt.md). |

Chaque signal a aussi un [[ui:sig.name]], un [[ui:sig.group]] et une
[[ui:sig.note]] — ce qu’il doit provoquer et ce qui doit correspondre à l’autre
bout.

Les cibles des signaux OSC et UDP sont `IP:port` ou `host:port` (par exemple
`127.0.0.1:9000`) ; un nom d’hôte est résolu à chaque envoi du signal. Le broker
d’un signal MQTT est `host:port` ; sans port, c’est `1883`.

Quand vous changez le type d’un signal, son message repart des valeurs par défaut
de ce type. Seule la cible est conservée, et seulement entre [[ui:sig.tr.osc]] et
[[ui:sig.tr.udp]], où la cible signifie la même chose.

## Envoyer un signal {#send}

Pour envoyer un signal depuis l’écran [[ui:nav.signals]], faites l’une de ces
choses :

- Sélectionnez-le et appuyez sur [[ui:sig.fire]].
- Appuyez sur <kbd>Ctrl</kbd>+<kbd>Enter</kbd> pendant que vous travaillez dans ses champs.
- Double-cliquez dessus dans l’arbre.

Chaque envoi écrit une ligne dans la console : ce qui est parti et où, les octets
envoyés, ou pour HTTP le statut et le temps pris. Un échec (une connexion refusée,
un hôte injoignable) est une ligne rouge avec la raison. L’heure du dernier envoi
s’affiche à côté des boutons.

Un signal part par les mêmes commandes que les écrans de son protocole :
l’[Inspecteur](inspector.md) le liste donc sous l’outil qui l’a envoyé, et l’autre
bout ne peut pas le distinguer d’un message que vous avez tapé.

Comment chaque type est envoyé :

| Type | Comment il part |
| --- | --- |
| [[ui:sig.tr.osc]] | Comme l’écran [[ui:nav.osc]] envoie un message. |
| [[ui:sig.tr.udp]] | Un datagramme vers la cible. |
| [[ui:sig.tr.http]] | Comme l’écran [[ui:nav.http]] envoie une requête, avec son bocal à cookies tant que [[ui:http.keepCookies]] y est actif. Une connexion refusée ou un délai dépassé compte comme un échec, pas comme un statut. |
| [[ui:sig.tr.mqtt]] | Tant que l’écran [[ui:nav.mqtt]] est connecté au broker du signal, sur cette connexion, avec son identifiant client et ses identifiants. Sinon — non connecté, ou connecté à un autre broker — Signal Lab se connecte au broker du signal pour cette seule publication, avec un identifiant client qui lui est propre, sans nom d’utilisateur et en session propre, puis se déconnecte. |

Une connexion est celle du broker du signal quand l’hôte est le même, sans tenir
compte de la casse, et que le port est le même, `1883` représentant un broker
écrit sans port. Les noms ne sont pas résolus : `localhost` et `127.0.0.1` sont
ici deux brokers différents, si bien qu’un signal qui nomme l’un n’est pas envoyé
sur une connexion faite à l’autre.

::: tip
Un signal MQTT ne stocke aucun mot de passe. Pour publier vers un broker qui en
demande un, connectez-vous d’abord à ce broker sur l’écran [[ui:nav.mqtt]] ; le
signal emprunte alors cette connexion.
:::

## Envoyer depuis n’importe quel écran {#palette}

Appuyez sur <kbd>Ctrl</kbd>+<kbd>K</kbd> sur n’importe quel écran pour ouvrir la
palette, tapez quelques lettres du nom, du dossier, de la cible ou du message d’un
signal, puis appuyez sur <kbd>Enter</kbd>. La palette se ferme et le signal est
envoyé ; vous restez sur l’écran que vous regardiez.

| Touche | Ce qu’elle fait |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>K</kbd> | Ouvre la palette, ou la ferme. |
| <kbd>↑</kbd> <kbd>↓</kbd> | Déplace le choix. |
| <kbd>Enter</kbd> | Envoie le signal choisi. |
| <kbd>Esc</kbd> | Ferme la palette sans envoyer. |

La palette liste au plus 12 signaux : les 12 premiers de la bibliothèque tant que
vous n’avez rien tapé, puis les 12 premiers qui correspondent. Un clic sur une
ligne l’envoie ; un clic à l’extérieur ferme la palette.

## Créer des signaux {#create}

### Sur l’écran Signaux {#create-here}

1. Choisissez le dossier auquel le signal appartient (voir [dossier courant](#current-folder)).
2. Appuyez sur [[ui:sig.new]]. Un nouveau signal OSC vers `127.0.0.1:9000`, adresse
   `/hello`, apparaît dans ce dossier, sélectionné.
3. Modifiez [[ui:sig.name]], [[ui:sig.transport]] et les champs du message.

Chaque modification s’enregistre d’elle-même ; il n’y a pas de bouton
d’enregistrement sur cet écran. Tant que le fichier de bibliothèque ne peut pas
être lu, rien n’est enregistré et les champs sont en lecture seule (voir [Le
fichier de la bibliothèque](#file)).

[[ui:sig.duplicate]] place une copie juste après le signal sélectionné, son nom
suivi d’un `·`. [[ui:sig.delete]] demande une confirmation
([[ui:sig.confirmDelete]]) : le second clic le retire du fichier. Son dossier
reste, même s’il est désormais vide.

### Depuis les écrans HTTP, OSC et MQTT {#save-from-screens}

La partie d’envoi de trois écrans — [[ui:http.request]] sur [[ui:nav.http]],
[[ui:osc.sender]] sur [[ui:nav.osc]], [[ui:mq.publish]] sur [[ui:nav.mqtt]] — peut
conserver comme signal ce qu’elle enverrait.

1. Préparez le message et envoyez-le jusqu’à ce qu’il fasse ce que vous voulez.
2. Appuyez sur [[ui:sig.saveNew]] (ou <kbd>Ctrl</kbd>+<kbd>S</kbd> dans cette partie
   de l’écran). La boîte de dialogue [[ui:sig.saveTitle]] s’ouvre.
3. Vérifiez [[ui:sig.name]] : il est suggéré d’après ce qui est envoyé.
4. Choisissez ou tapez un [[ui:sig.group]]. Le dossier utilisé la dernière fois est
   prérempli ; un chemin tel que `Venue/Stage` qui n’existe pas encore est créé.
5. Appuyez sur [[ui:sig.saveConfirm]].

À partir de là, l’écran est lié à ce signal. Une pastille à côté des boutons
indique où il vit (`❖ Folder / Name`) ; cliquez dessus pour voir le signal sur
l’écran [[ui:nav.signals]].

| Vous voyez | Cela signifie | Ce que vous pouvez faire |
| --- | --- | --- |
| ✓ [[ui:sig.savedState]] (grisé) | La bibliothèque contient exactement ce que l’écran enverrait. | Rien à enregistrer. |
| [[ui:sig.save]], et [[ui:sig.changed]] sur la pastille | Le message de l’écran diffère du signal. | [[ui:sig.save]] ou <kbd>Ctrl</kbd>+<kbd>S</kbd> écrit le message de l’écran dans ce signal ; son nom, son dossier et sa note restent. |
| [[ui:sig.saveAs]] | — | Rouvre la boîte de dialogue, préremplie avec le nom et le dossier du signal, et enregistre un nouveau signal. L’écran est alors lié au nouveau. |

La comparaison porte sur le message, pas sur la façon dont il est écrit : l’ordre
des clés JSON et les derniers chiffres d’un flottant OSC au-delà de la précision
32 bits ne comptent pas comme un changement.

Les écrans [[ui:nav.http]] et [[ui:nav.osc]] conservent le lien quand Signal Lab
redémarre ; l’écran [[ui:nav.mqtt]] le conserve jusqu’à ce que vous fermiez
l’application.

::: warning
Un signal HTTP conserve son [[ui:field.auth]] — nom d’utilisateur et mot de passe,
ou jeton — dans le fichier de bibliothèque en texte clair. Quiconque peut lire le
fichier peut les lire.
:::

### Ouvrir un signal dans son écran {#open-in-screen}

Un signal HTTP, OSC ou MQTT sélectionné a un bouton qui l’ouvre dans l’écran de son
protocole ([[ui:nav.http]], [[ui:nav.osc]] ou [[ui:nav.mqtt]]). Les champs de
l’écran sont remplis depuis le signal et l’écran y est lié, comme ci-dessus :
modifiez sur place, envoyez, puis [[ui:sig.save]]. Un signal UDP brut n’a pas
d’écran propre.

### Depuis une trame ou un topic {#capture}

- Dans l’[Inspecteur](inspector.md#save-as-signal), sélectionnez une trame et
  appuyez sur [[ui:sig.fromFrame]]. Un datagramme devient un signal UDP brut avec
  les octets exacts de cette trame ; une publication MQTT devient un signal MQTT
  avec le même broker, topic, QoS, indicateur retain et charge utile. Les autres
  trames ne peuvent pas être enregistrées.
- Sur l’écran [[ui:nav.mqtt]], sélectionnez un topic et appuyez sur
  [[ui:sig.fromFrame]]. Vous obtenez un signal MQTT qui publie la dernière valeur
  du topic, avec son QoS et son indicateur retain, vers le broker auquel vous êtes
  connecté.

Les deux vont dans le dossier [[ui:sig.capturedFolder]] et sont nommés d’après ce
qui a été capturé.

### Dans une expérience {#in-experiments}

Quand vous ajoutez un nœud à une expérience, le menu [[ui:exp.addNode]] liste aussi
vos signaux sous [[ui:exp.group.signals]]. En choisir un ajoute un nœud OSC, HTTP,
MQTT ou UDP avec le même message. Un signal UDP brut avec une charge utile
hexadécimale n’est pas proposé : le nœud UDP envoie du texte. Voir
[Nœuds](../experiments/nodes.md).

## Dossiers {#folders}

Un dossier est un chemin de noms joints par `/` : `API/Auth` est le dossier `Auth`
à l’intérieur de `API`. Le champ [[ui:sig.group]] d’un signal contient le chemin de
son dossier ; vide signifie le niveau supérieur ([[ui:sig.topLevel]]). Les noms
sont détourés et les parties vides supprimées quand vous quittez le champ, si bien
que ` API / Auth/ ` devient `API/Auth`.

Les dossiers sont triés par nom, les nombres dans l’ordre numérique (`Cue 2` avant
`Cue 10`) ; les signaux restent dans l’ordre du fichier. Chaque dossier affiche
combien de signaux il contient, ses sous-dossiers compris. Un dossier vide est
conservé jusqu’à ce que vous le retiriez.

### Le dossier courant {#current-folder}

Le dossier sur lequel vous avez cliqué en dernier, ou le dossier du signal que vous
avez sélectionné, est le dossier courant : [[ui:sig.new]] et [[ui:sig.newFolder]] y
placent les choses. Une pastille au-dessus de l’arbre le nomme ; cliquez sur la
pastille pour revenir au niveau supérieur.

### Travailler avec les dossiers {#folder-tasks}

| Pour | Faites ceci |
| --- | --- |
| Créer un dossier | Appuyez sur ＋ [[ui:sig.newFolder]]. Il est créé dans le dossier courant, nommé [[ui:sig.newFolderName]] (avec un numéro après quand ce nom est pris), et vous le renommez aussitôt. |
| Ouvrir ou fermer un dossier | Cliquez dessus, ou appuyez sur <kbd>→</kbd> / <kbd>←</kbd> tant qu’il a le focus. Signal Lab se souvient des dossiers fermés. |
| Les ouvrir ou les fermer tous | Les boutons ⊞ et ⊟ au-dessus de l’arbre ([[ui:sig.expandAll]], [[ui:sig.collapseAll]]). |
| Renommer un dossier | Appuyez sur ✎ ([[ui:sig.renameFolder]]) ou <kbd>F2</kbd> dessus, tapez, puis <kbd>Enter</kbd> ; <kbd>Esc</kbd> annule. |
| Déplacer un signal ou un dossier | Faites-le glisser sur un dossier, ou sur un espace vide de l’arbre pour le niveau supérieur. |
| Déplacer un signal en tapant | Modifiez son champ [[ui:sig.group]]. |
| Retirer un dossier | Appuyez sur × ([[ui:sig.removeFolder]]) puis sur [[ui:sig.confirmRemoveFolder]], ou appuyez deux fois sur <kbd>Delete</kbd> dessus. |

Un renommage ne fusionne jamais deux dossiers : un nom contenant `/`, ou déjà porté
par un dossier voisin, est refusé, et la console le dit. Un dossier ne peut pas être
glissé dans lui-même ni dans un dossier qu’il contient. Glisser un dossier dans un
dossier qui en contient déjà un du même nom fusionne les deux.

Retirer un dossier ne retire que le dossier : ses signaux et sous-dossiers
remontent d’un niveau. Rien n’est supprimé.

### Trouver un signal {#filter}

Tapez dans [[ui:sig.search]] au-dessus de l’arbre. Cela correspond au nom, au
dossier, à la note, à la cible et au message. Pendant que vous filtrez, chaque
dossier contenant une correspondance est ouvert et les autres sont masqués.

## Le jeu de départ {#starter-set}

La première fois que Signal Lab ne trouve pas de fichier de bibliothèque, il écrit
neuf exemples, chacun sur quelque chose qu’il est facile de rater. Leurs noms et
leurs notes sont écrits dans la langue de l’interface à ce moment-là ; ensuite, ils
sont à vous. Tous pointent vers cet ordinateur.

| Dossier | Signal | Envoie |
| --- | --- | --- |
| `OSC` | [[ui:seed.osc-fader.name]] | `/fader/1` avec le flottant `0.75` vers `127.0.0.1:9000` |
| `OSC` | [[ui:seed.osc-types.name]] | `/types` avec int `-7`, flottant `1.5`, chaîne `hi`, booléen true, int64 `4294967296`, double `0.125` et nil |
| `OSC` | [[ui:seed.osc-id-and-value.name]] | `/tag` avec les chaînes `reader-1` et `04a1b2c3` |
| `OSC` | [[ui:seed.osc-trigger.name]] | `/cue/go` sans argument |
| `MQTT` | [[ui:seed.mqtt-publish.name]] | `1` vers `lab/example/value` sur `127.0.0.1:1883`, QoS 0 |
| `MQTT` | [[ui:seed.mqtt-retained.name]] | `night` vers `lab/example/config`, QoS 1, retenu |
| `MQTT` | [[ui:seed.mqtt-clear-retained.name]] | Une charge utile retenue vide vers `lab/example/config`, QoS 1 |
| `HTTP` | [[ui:seed.http-reachable.name]] | `GET http://127.0.0.1:8080/`, délai 4000 ms |
| [[ui:seed.folder.Raw]] | [[ui:seed.udp-raw.name]] | Les octets `de ad be ef` vers `127.0.0.1:9000` |

Pour récupérer le jeu de départ, déplacez ou renommez `signals.json` et appuyez sur
[[ui:sig.reload]] : sans fichier, il est réécrit.

## Le fichier de la bibliothèque {#file}

La bibliothèque est `signals.json` dans le dossier de données :
`Documents/SignalLab` dans votre dossier personnel sur un poste de bureau, ou le
dossier de données du serveur (voir [Fichiers](../reference/files.md)). Survolez le
compteur de signaux sous l’arbre pour voir le chemin complet.

- **Enregistré de lui-même.** Chaque modification est écrite 0,7 s après la
  dernière, tout le fichier d’un coup, via un fichier temporaire dans le même
  dossier qui prend ensuite la place du fichier — une écriture interrompue laisse le
  fichier précédent. Tant qu’une écriture attend, le pied de l’arbre indique
  [[ui:sig.saving]] ; puis [[ui:sig.saved]]. Une écriture encore en attente est
  effectuée avant qu’une mise à jour ne redémarre l’application.
- **Modifié à la main.** Signal Lab ne remarque pas quand le fichier change sous
  lui. Après l’avoir modifié, ou remplacé par celui d’une autre machine, appuyez sur
  [[ui:sig.reload]]. Le rechargement relit le fichier et abandonne une modification
  encore en attente d’écriture.
- **Jamais remplacé tant qu’il est cassé.** Si le fichier n’est pas du JSON valide,
  ou pas une bibliothèque de signaux, l’arbre affiche l’erreur avec le chemin, la
  ligne et la colonne du fichier, et la console dit la même chose. Le fichier est
  laissé tel quel, et rien n’écrit la bibliothèque tant qu’elle ne se relit pas :
  [[ui:sig.new]], renommer, déplacer et retirer des signaux et des dossiers,
  glisser, les champs d’un signal, [[ui:sig.saveNew]], [[ui:sig.save]] et
  [[ui:sig.saveAs]] sur les écrans HTTP, OSC et MQTT, et [[ui:sig.fromFrame]] dans
  l’Inspecteur et sur l’écran MQTT sont tous désactivés, et leur infobulle dit ce
  qui ne va pas. Corrigez le fichier, ou retirez-le, et appuyez sur
  [[ui:sig.reload]] : une fois qu’il se lit, tout fonctionne de nouveau.
- **Un fichier qui casse pendant que l’application tourne.** Si vous modifiez le
  fichier en quelque chose d’illisible et que l’application enregistre ensuite une
  modification, l’enregistrement est refusé avec la même erreur, le fichier est
  laissé tel que vous l’avez fait, et l’application cesse d’écrire jusqu’à ce que
  vous le corrigiez et appuyiez sur [[ui:sig.reload]]. Un fichier que vous avez
  modifié et laissé valide est remplacé par la liste de l’application à son prochain
  enregistrement, comme ci-dessus : rechargez d’abord.

Un court exemple du fichier :

```json
{
  "version": 2,
  "signals": [
    {
      "id": "fader-value",
      "name": "Fader value",
      "group": "Venue/Stage",
      "note": "Main fader of desk A.",
      "body": {
        "transport": "osc",
        "target": "127.0.0.1:9000",
        "address": "/fader/1",
        "args": [{ "type": "float", "value": 0.75 }]
      }
    }
  ],
  "folders": ["Venue/Stage", "Venue/Empty for now"]
}
```

| Clé | Quoi |
| --- | --- |
| `version` | `2`. Un fichier de version 1 (avant les dossiers) se lit de la même façon, sans dossiers vides. |
| `signals[].id` | Fabriqué à partir du nom quand le signal est créé (`fader-value`, `fader-value-2`, …) et jamais changé par un renommage. `signallab fire` trouve un signal par lui. |
| `signals[].group` | Le chemin du dossier ; `""` est le niveau supérieur. |
| `signals[].body` | Le message. `transport` est `osc`, `udp`, `http` ou `mqtt` ; les autres clés sont les champs de ce type. |
| `folders` | Chaque dossier, pour qu’un dossier vide soit conservé. Omis quand il n’y en a aucun. Un `group` qu’aucune entrée ne liste est un dossier aussi. |

## Depuis la ligne de commande {#cli}

`signallab fire` envoie un signal d’une bibliothèque, par les mêmes commandes que
l’application :

```bash
signallab fire "Fader value"
signallab fire fader-value --library ./show/signals.json
```

Il trouve le signal par son identifiant d’abord, puis par son nom, sans tenir
compte de la casse. Quand plusieurs signaux portent ce nom, il nomme leurs
identifiants et n’envoie rien. Sans `--library`, il lit le `signals.json` de
l’application ; il n’écrit jamais le fichier. Voir [La ligne de
commande](../automation/cli.md#cli-fire).

## Voir aussi {#related}

- [Inspecteur](inspector.md) — regardez ce qu’envoie un signal, et conservez une
  trame capturée comme signal.
- [Raccourcis clavier](../reference/shortcuts.md)
- [Fichiers](../reference/files.md) — où se trouve le dossier de données.

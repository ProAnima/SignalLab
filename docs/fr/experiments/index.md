---
title: "L’éditeur"
description: "Construire, modifier et exécuter des expériences sur le canevas de nœuds — ajouter et relier des nœuds, sélectionner et copier, le volet des propriétés, Envoyer maintenant, la validation, l’exécution et les fichiers derrière elle."
---

# L’éditeur d’expériences

Une expérience est un test écrit comme un graphe : des nœuds qui envoient (une requête
HTTP, un message OSC, une publication MQTT…), attendent une réponse, vérifient ce qui est
revenu, jouent l’autre côté ou coupent le réseau sur commande, reliés par des fils qui disent
ce qui s’exécute ensuite. Une exécution démarre à [[ui:exp.node.start]], suit les fils et réussit quand elle
atteint [[ui:exp.node.end]] avec chaque étape réussie. Vous la construisez sur l’écran [[ui:nav.experiment]],
l’exécutez là, et exécutez le même fichier depuis la ligne de commande ou un serveur.

L’éditeur contient une seule expérience à la fois et l’enregistre au fur et à mesure de
votre travail. Pour en garder plusieurs, exportez des instantanés ou ouvrez-en une autre depuis
un fichier (voir [Enregistrement et fichiers](#files)).

Chaque type de nœud, ses champs et ses sorties sont dans la [référence des nœuds](nodes.md).
La façon dont les valeurs circulent entre les nœuds est [Données et modèles](data.md) ; les
branches parallèles, les boucles, les réessais et les répétitions sont [Flux](flow.md).

## L’écran {#screen}

| Zone | Ce qu’elle contient |
| --- | --- |
| Barre d’outils (haut) | Le nom de l’expérience et son état d’enregistrement, l’ajout de nœuds, les paramètres, les profils, les volets, le mode focus, le plein écran et Exécuter |
| Barre du canevas | Annuler et rétablir, le chercheur de nœuds, [[ui:exp.arrange]] et le zoom |
| Canevas | Le graphe : les nœuds, les fils, et la progression de l’exécution dessinée dessus |
| [[ui:exp.properties]] (droite) | Les champs du nœud sélectionné, son aperçu et [[ui:exp.sendNow]] ; ou ce que plusieurs nœuds sélectionnés ou un fil sélectionné permettent |
| [[ui:exp.timeline]] (bas) | Les étapes de l’exécution en cours ou de la dernière, son résultat, son rapport et sa graine |

Le volet des propriétés et la chronologie ont une poignée sur leur bord intérieur : faites-la
glisser, ou placez-y le focus avec <kbd>Tab</kbd> et utilisez les flèches (<kbd>Shift</kbd>
pour de plus grands pas) ; un double-clic ou <kbd>Enter</kbd> rend au volet sa
taille par défaut. Les tailles sont conservées pour la prochaine fois. La
chronologie reste repliée jusqu’à ce que la première exécution l’ouvre.

## La barre d’outils {#toolbar}

| Contrôle | Ce qu’il fait |
| --- | --- |
| ☰ [[ui:exp.documents]] | Ouvre les modèles, l’ouverture d’un fichier JSON et l’export ([Enregistrement et fichiers](#files)) |
| [[ui:exp.name]] | Le nom de l’expérience ; une exécution en a besoin. Les rapports d’exécution l’enregistrent, et [[ui:exp.compare]] retrouve les exécutions précédentes par lui |
| [[ui:exp.saved]] / [[ui:exp.saving]] / [[ui:exp.saveError]] | Si la dernière modification est sur le disque |
| ⚠ [[ui:exp.needsLinks]] | Affiché tant que l’expérience ne peut pas s’exécuter ; son info-bulle dit pourquoi, un clic sélectionne le nœud concerné ([Validation](#validation)) |
| ＋ [[ui:exp.addNode]] | Ouvre le menu d’ajout, après le nœud sélectionné quand il y en a un (<kbd>A</kbd>) |
| [[ui:exp.profile]] | Le profil avec lequel s’exécutent l’aperçu et Envoyer maintenant ; affiché quand l’expérience a des profils. ⚠ marque un profil qui ne s’exécuterait pas |
| `{ }` [[ui:exp.params]] | Les paramètres, les profils, la graine, les cookies et les secrets ([Données et modèles](data.md)) |
| ☷ [[ui:exp.properties]] | Affiche ou masque le volet des propriétés |
| ▢ [[ui:exp.focus]] | Masque la barre latérale, l’en-tête et le panneau inférieur ([Mode focus et plein écran](#focus)) |
| ⛶ [[ui:exp.fullscreen]] | La fenêtre en plein écran, avec le mode focus |
| [[ui:exp.run]] | Vérifie, enregistre et exécute l’expérience ; pendant l’exécution, le bouton est [[ui:common.stop]] |
| ▾ [[ui:exp.runWith]] | Une exécution avec un autre profil, d’autres valeurs de paramètres ou une graine donnée, sans changer l’expérience |

## Se déplacer sur le canevas {#canvas}

- **Déplacer** : faites glisser le canevas vide, ou faites défiler.
- **Zoomer** : <kbd>Ctrl</kbd> et la molette de la souris zooment autour du pointeur ;
  − et ＋ dans la barre du canevas avancent de 10 %. Le zoom va de 15 % à 200 % ; le
  pourcentage entre les boutons indique où vous êtes.
- **1:1** ([[ui:exp.resetZoom]], <kbd>Ctrl</kbd>+<kbd>1</kbd>) revient à 100 %.
- ⊡ [[ui:exp.fit]] (<kbd>Ctrl</kbd>+<kbd>0</kbd>) montre tout le graphe, à
  100 % au plus.
- **Trouver un nœud** : [[ui:exp.nodes]] dans la barre du canevas (il indique combien de nœuds
  il y a) ou <kbd>Ctrl</kbd>+<kbd>F</kbd>. Tapez des mots du nom du nœud,
  de son résumé (une URL, une adresse, un topic) ou de son id ; <kbd>↑</kbd>
  <kbd>↓</kbd> choisissent, <kbd>Enter</kbd> sélectionne le nœud et l’amène dans
  la vue (zoomé à au moins 80 %), <kbd>Esc</kbd> ferme.

Il n’y a pas de mini-carte : [[ui:exp.fit]] et le chercheur font son travail.

## Nœuds et fils {#nodes-and-wires}

Un nœud montre son type, un résumé d’une ligne de ce qu’il fait, et des pastilles pour ses
réglages : ↻ et un nombre pour Réessai, × et un compte (ou une durée) pour Répétition, ⚡
pour Charge. Son entrée est à gauche — chaque nœud sauf [[ui:exp.node.start]] en a une — et ses
sorties à droite :

| Sortie | Sur | Suivie quand |
| --- | --- | --- |
| [[ui:exp.outputPort]] | La plupart des nœuds | L’étape a réussi |
| [[ui:exp.yes]] / [[ui:exp.no]] | [[ui:exp.node.branch_status]], [[ui:exp.node.branch_value]] | La comparaison est vérifiée / ne l’est pas |
| [[ui:exp.branch1]] / [[ui:exp.branch2]] | [[ui:exp.node.fork]] | Toujours, les deux à la fois |
| [[ui:exp.portMatched]] / [[ui:exp.portTimeout]] | Les attentes | Un message correspondant est arrivé / aucun à temps |
| [[ui:exp.portBody]] / [[ui:exp.portDone]] / [[ui:exp.portLimit]] | [[ui:exp.node.loop]] | Une autre itération / la boucle est finie / elle a épuisé ses itérations |

**Une sortie peut avoir plusieurs fils.** Le premier fil continue la branche ;
chaque fil suivant démarre une branche parallèle avec une copie des variables connues
à ce moment-là. Un nœud [[[ui:exp.node.join]]](nodes.md#node-join) attend chaque fil qui
y mène. Les détails sont dans [Flux](flow.md).

[[ui:exp.portTimeout]] et [[ui:exp.portLimit]] sont facultatifs : laissés non reliés, un
délai d’attente ou un épuisement des itérations fait échouer l’étape. Chaque autre sortie doit
avoir un fil avant que l’expérience puisse s’exécuter.

Le fil qui va du corps d’une [[ui:exp.node.loop]] jusqu’à elle est dessiné comme un arc
au-dessus du corps ; c’est le seul fil qui peut aller vers l’arrière.

## Ajouter des nœuds {#adding}

### Le menu d’ajout {#add-menu}

Le menu d’ajout liste chaque type de nœud par groupe — [[ui:exp.group.action]],
[[ui:exp.group.observe]], [[ui:exp.group.emulate]], [[ui:exp.group.fault]],
[[ui:exp.group.data]], [[ui:exp.group.check]], [[ui:exp.group.flow]] — et
ensuite [[ui:exp.group.signals]] : les signaux de votre [bibliothèque](../tools/signals.md)
qui peuvent devenir un nœud (OSC, HTTP, UDP avec une charge utile texte, MQTT), avec leurs
champs remplis.

Son champ de recherche a le focus à l’ouverture. Tapez quelques lettres : chaque mot
doit apparaître dans le nom du nœud, sa description ou son type (`http`,
`wait_osc`) ; un signal est trouvé par son nom, son dossier, son transport ou sa cible.
<kbd>↑</kbd> <kbd>↓</kbd> choisissent, <kbd>Enter</kbd> ajoute, <kbd>Esc</kbd>
ferme. La ligne en haut indique où va le nœud : après un nœud, ou
en parallèle d’un autre.

Un nouveau nœud est sélectionné, le volet des propriétés s’ouvre, et son champ principal — l’
URL, l’adresse, le topic, le délai — a le focus avec son texte
sélectionné, pour que vous puissiez taper tout de suite. <kbd>Esc</kbd> dans un champ vous ramène
au nœud sur le canevas, prêt pour le prochain <kbd>A</kbd>.

### Après le nœud sélectionné {#add-after}

<kbd>A</kbd>, ＋ [[ui:exp.addNode]] dans la barre d’outils et ＋ [[ui:exp.addNext]]
dans le volet des propriétés ajoutent l’étape suivante après le nœud sélectionné :

- Il se rattache à la première sortie du nœud qui n’a pas encore de fil (le
  [[ui:exp.no]] libre d’un [[ui:exp.node.branch_status]], par exemple), sinon à sa première
  sortie.
- Si cette sortie a déjà un fil, le nouveau nœud y est inséré (dans le
  premier, s’il y en a plusieurs) : le fil passe désormais par le nouveau nœud, et
  tout ce qui suit se décale à droite pour faire de la place.
- Avec [[ui:exp.node.end]] sélectionné, le nœud va avant lui, quand un fil
  y mène.
- Avec rien de sélectionné, le nœud atterrit au milieu de la vue, sans
  fils.

Un nœud à sortie unique qui est posé sur le [[ui:exp.portBody]] vide d’une
[[ui:exp.node.loop]] — de cette façon ou [sur un nouveau fil](#add-branch) — est aussi relié
en retour à elle, pour que le corps soit complet tout de suite.

### Dans un fil {#insert}

Chaque fil a un ＋ en son milieu ([[ui:exp.insertNode]]). Il ouvre le menu d’ajout,
et le nœud que vous choisissez est inséré dans ce fil. Une [[ui:exp.node.loop]] insérée de
cette façon continue le flux par son [[ui:exp.portDone]].

### Sur un nouveau fil {#add-branch}

Faites glisser un fil hors d’une sortie et relâchez sur le canevas vide : le menu d’ajout
s’ouvre là, et le nouveau nœud va sur un nouveau fil de cette sortie — à côté des
fils qu’elle a déjà, donc il s’exécute en parallèle avec eux. La même chose se produit quand
vous cliquez une sortie puis le canevas vide, ou double-cliquez dessus.

### N’importe où {#add-anywhere}

Double-cliquez le canevas vide pour ajouter un nœud à cet endroit, sans fils.

### Depuis d’autres écrans {#from-screens}

- [[ui:common.toExperiment]] sur les écrans [OSC](../protocols/osc.md) et
  [HTTP](../protocols/http.md) ajoute ce que vous venez d’essayer comme étape
  suivante — juste avant [[ui:exp.node.end]], quand un fil y mène — et
  bascule vers l’éditeur.
- [[ui:osc.waitForThis]] à côté d’un message dans le moniteur OSC ajoute une
  [[ui:exp.node.wait_osc]] qui le reconnaît : son adresse et son texte, les arguments
  entiers et vrai/faux (les flottants sont des mesures qui changent, donc ils sont laissés
  de côté). Arrêtez le moniteur avant d’exécuter : l’exécution écoute elle-même sur ce port.
- [[ui:mq.waitForThis]] sur un topic de l’arbre [MQTT](../protocols/mqtt.md)
  ajoute une [[ui:exp.node.wait_mqtt]] sur ce topic, chez ce broker.

Pendant qu’une exécution est en cours, on ne peut pas ajouter de nœuds ; la console le dit.

## Relier des nœuds {#connecting}

- **Faites glisser** d’une sortie vers un nœud. Relâcher à moins de 24 pixels d’un nœud
  suffit.
- **Cliquez** une sortie (ou placez-y le focus et appuyez sur <kbd>Enter</kbd> ou
  <kbd>Space</kbd>) : la barre du canevas indique [[ui:exp.chooseInput]]. Cliquez un nœud
  ou son entrée pour relier ; cliquez le canevas vide pour y ajouter un nœud sur un nouveau
  fil ; [[ui:exp.cancelLink]] ou <kbd>Esc</kbd> abandonne.

Un fil est refusé quand il créerait un cycle (autre que le chemin de retour d’une
[[ui:exp.node.loop]]), quand il mène vers [[ui:exp.node.start]] ou sort de
[[ui:exp.node.end]], ou quand il relierait un nœud à lui-même ; l’éditeur le
dit. Tracer un fil qui existe déjà ne change rien.

Pour **retirer un fil**, cliquez-le — le volet des propriétés montre ce qu’il relie —
et appuyez sur <kbd>Delete</kbd>, ou utilisez [[ui:exp.removeWire]] là. Survoler un
fil montre aussi un × au-dessus de son ＋. Les propriétés d’un nœud listent ses fils
sortants, chacun avec × ([[ui:exp.disconnect]]).

## Sélectionner plusieurs nœuds {#selection}

| Pour | Faites |
| --- | --- |
| Sélectionner un nœud | Cliquez-le, ou allez-y avec <kbd>Tab</kbd> |
| Ajouter un nœud à la sélection, ou l’en retirer | <kbd>Shift</kbd> ou <kbd>Ctrl</kbd> et un clic |
| Sélectionner au cadre | <kbd>Shift</kbd> et faites glisser sur le canevas vide : chaque nœud que le cadre touche rejoint la sélection |
| Sélectionner tous les nœuds | <kbd>Ctrl</kbd>+<kbd>A</kbd> |
| Vider la sélection | Cliquez le canevas vide ; <kbd>Esc</kbd> quand plusieurs sont sélectionnés |
| Les déplacer | Faites glisser l’un d’eux : ils bougent tous |
| Les décaler finement | Avec un nœud sous le focus, les flèches déplacent la sélection de 5 pixels, 20 avec <kbd>Shift</kbd> |

Le volet des propriétés montre le dernier nœud choisi ; avec plusieurs sélectionnés, il
montre combien, avec [[ui:exp.copy]], [[ui:exp.duplicate]] et
[[ui:exp.deleteSelected]].

### Copier, couper et coller {#copy-paste}

<kbd>Ctrl</kbd>+<kbd>C</kbd> copie les nœuds sélectionnés et les fils entre
eux sous forme de texte ; <kbd>Ctrl</kbd>+<kbd>X</kbd> les retire aussi ;
<kbd>Ctrl</kbd>+<kbd>V</kbd> les colle — dans cette expérience, dans une autre
ouverte plus tard, ou dans une autre fenêtre Signal Lab. Le texte est du JSON, donc vous
pouvez aussi le garder dans un fichier ou un message.

- [[ui:exp.node.start]] et [[ui:exp.node.end]] sont uniques : ils ne sont jamais
  copiés.
- Les fils entre un nœud copié et le reste du graphe ne sont pas copiés ; reliez
  la copie vous-même.
- Chaque nœud collé reçoit un nouvel id.
- Un nœud qui en nomme un autre — [[ui:exp.node.impairment_change]],
  [[ui:exp.node.emulator_state]], [[ui:exp.node.ws_send]],
  [[ui:exp.node.wait_ws]], [[ui:exp.node.ws_close]] — nomme la copie quand elle a été copiée aussi ;
  sinon il continue de nommer l’original s’il est dans cette expérience, ou le
  premier nœud de ce type ici.
- Un [[ui:exp.node.emulator]] ou [[ui:exp.node.impairment]] collé dont le port
  est déjà écouté par cette expérience passe au port libre suivant. Ce qui envoie à
  l’original continue de le faire.
- La copie atterrit 32 pixels à droite et une ligne sous l’endroit d’où elle vient,
  plus bas jusqu’à ne couvrir aucun nœud, et est sélectionnée.

<kbd>Ctrl</kbd>+<kbd>D</kbd> ([[ui:exp.duplicate]]) fait la même chose sans le
presse-papiers.

### Supprimer {#deleting}

<kbd>Delete</kbd> ou <kbd>Backspace</kbd> ([[ui:exp.delete]],
[[ui:exp.deleteSelected]]) retire les nœuds sélectionnés et leurs fils. Un nœud
qui avait exactement un fil entrant et un fil sortant laisse un fil à sa place, du
nœud avant lui au nœud après lui, pour qu’une chaîne reste connectée. [[ui:exp.node.start]]
et [[ui:exp.node.end]] ne peuvent pas être supprimés.

## Annuler et rétablir {#undo}

<kbd>Ctrl</kbd>+<kbd>Z</kbd> annule ; <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd>
ou <kbd>Ctrl</kbd>+<kbd>Y</kbd> rétablit (↶ ↷ dans la barre du canevas). Un glisser, une
série de décalages au clavier ou la saisie dans un champ est une étape. Les 100 dernières
étapes sont conservées tant que l’application est ouverte, à travers les changements d’écran ; ouvrir
une autre expérience est aussi une étape, donc <kbd>Ctrl</kbd>+<kbd>Z</kbd> ramène
la précédente. Annuler et rétablir attendent pendant qu’une exécution est en cours.

## Disposer {#arrange}

[[ui:exp.arrange]] dispose le graphe de gauche à droite : chaque nœud dans la
colonne après le dernier nœud qui y mène, le corps d’une [[ui:exp.node.loop]]
dans sa rangée — puis ajuste la vue. C’est une étape de l’historique. Un brouillon
avec un cycle qui n’est pas celui d’une boucle est laissé tel quel.

## Le volet des propriétés {#properties}

Avec un nœud sélectionné, le volet montre, de haut en bas :

1. Le nom du nœud (son info-bulle dit ce qu’il fait) et, quand l’expérience
   ne peut pas s’exécuter à cause de ce nœud, ce qui ne va pas.
2. Ses champs. Les champs qui prennent des [modèles](data.md) suggèrent des paramètres,
   des variables, des secrets et des générateurs au fur et à mesure que vous tapez `{{`, ou sur
   <kbd>Ctrl</kbd>+<kbd>Space</kbd>.
3. Ses réglages : [[ui:exp.loadOn]] sur une requête HTTP, [[ui:exp.repeatOn]] sur
   les nœuds qui envoient, [[ui:exp.retryOn]] sur les nœuds qui envoient ou écoutent, et
   [[ui:exp.expectReply]] sur les messages OSC et UDP (voir [Réglages des
   nœuds](nodes.md#settings)). La charge remplace la répétition et le réessai tant qu’elle est active.
4. Le résultat de charge de la dernière exécution, sur un nœud HTTP qui s’est exécuté sous charge.
5. L’aperçu — [[ui:exp.preview]], [[ui:exp.previewWait]] ou
   [[ui:exp.previewCheck]] — quand le nœud a des modèles : ce qu’il enverrait,
   attendrait ou comparerait, résolu avec les paramètres actuels et les valeurs
   connues jusqu’ici ([Envoyer maintenant et l’aperçu](#send-now)).
6. [[ui:exp.sendNow]] ou [[ui:exp.listenNow]], et ce qu’il a fait en dernier.
7. Ses fils sortants, chacun avec ×.
8. ⚡ [[ui:exp.routeThrough]] sur un message OSC ou UDP : une [[ui:exp.node.impairment]] est placée
   devant le nœud — écoutant sur un port de bouclage libre à partir de 9010,
   transmettant à la cible du nœud — et le nœud est pointé vers elle, pour que la
   prochaine exécution dégrade ce qu’il envoie ([Défaillances](faults.md)).
9. ＋ [[ui:exp.addNext]], [[ui:exp.copy]], [[ui:exp.duplicate]] et
   [[ui:exp.delete]].

Un double-clic sur un nœud ouvre le volet avec le curseur dans son champ principal.
Avec plusieurs nœuds sélectionnés, le volet offre ce qui peut être fait à tous ;
avec un fil sélectionné, ce qu’il relie et [[ui:exp.removeWire]]. Pendant qu’une exécution
est en cours, les champs sont verrouillés.

## Envoyer maintenant et l’aperçu {#send-now}

[[ui:exp.sendNow]] (<kbd>Ctrl</kbd>+<kbd>Enter</kbd>, aussi depuis l’intérieur des
champs du nœud) envoie le nœud sélectionné tout seul, sans exécuter l’
expérience, par le même code qu’une exécution. Il est proposé sur
[[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]],
[[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_connect]] et
[[ui:exp.node.ws_send]].
Sur une attente, c’est [[ui:exp.listenNow]] : l’attente écoute à partir de maintenant jusqu’à ce qu’un message
corresponde ou que son délai d’attente se termine.

- Le nœud utilise le profil actif, les secrets enregistrés et les valeurs de variables
  connues jusqu’ici — de la dernière exécution et des résultats [[ui:exp.sendNow]] précédents.
  Si un modèle nomme une valeur que personne n’a encore définie, rien n’est envoyé et les noms
  sont listés.
- Il est envoyé une fois : Réessai, Répétition et Charge ne s’appliquent pas, aucun cookie n’est conservé,
  et les émulateurs et relais de l’expérience ne sont pas démarrés.
- Un [[ui:exp.node.ws_send]] ou [[ui:exp.node.wait_ws]] ouvre la connexion que son
  [[ui:exp.node.ws_connect]] décrit, pour ce seul test.
- Sur une requête HTTP, la réponse est affichée — statut, durée, taille, et le corps,
  formaté en JSON. Les valeurs que les nœuds [[ui:exp.node.extract]] après la
  requête prendraient sont remplies aussitôt. Cliquez une valeur dans une réponse JSON pour
  ajouter un nœud [[ui:exp.node.extract]] pour elle juste après la requête, sa
  variable nommée et sa valeur connue.
  [[ui:http.mockThis]] transforme la réponse en route d’un
  [émulateur](../tools/emulators.md).
- Le résultat est aussi écrit dans la console.

L’aperçu est résolu par le moteur aussi, environ un quart de seconde
après une modification. Les valeurs de secrets ne sont jamais montrées, ni là ni ailleurs.

## Validation {#validation}

L’éditeur vérifie l’expérience au fur et à mesure, et une fois de plus quand vous appuyez sur
[[ui:exp.run]] :

- Une sortie qui a encore besoin d’un fil pulse en ambre.
- Un nœud qu’aucun fil depuis [[ui:exp.node.start]] n’atteint est dessiné en pointillés ; son info-bulle le dit.
- Le nœud concerné par un problème est entouré, et le problème est écrit au-dessus de ses
  champs.
- ⚠ [[ui:exp.needsLinks]] dans la barre d’outils nomme le problème ; un clic sélectionne
  le nœud.

Une expérience s’exécute quand, entre autres choses :

- elle a un nom, exactement un [[ui:exp.node.start]] et un
  [[ui:exp.node.end]], et au plus 64 nœuds ;
- chaque nœud est atteignable depuis [[ui:exp.node.start]] et chaque sortie requise a un fil ;
- les seuls cycles sont les corps des nœuds [[ui:exp.node.loop]] qui y ramènent ;
- chaque vérification et [[ui:exp.node.extract]] a une requête HTTP avant elle sur chaque chemin (une
  sous charge ne compte pas : elle ne laisse pas de réponse) ;
- un [[ui:exp.node.ws_send]], [[ui:exp.node.wait_ws]] ou [[ui:exp.node.ws_close]]
  vient après le [[ui:exp.node.ws_connect]] qu’il utilise ;
- chaque champ est rempli et dans les limites, et chaque modèle nomme quelque chose de
  connu à ce moment-là.

Les brouillons inachevés sont enregistrés quand même. Une exécution est refusée, avant tout
trafic, quand un secret dont elle a besoin n’est pas enregistré. Chaque message est listé dans la
[référence des erreurs](../reference/errors.md).

## Exécuter {#running}

[[ui:exp.run]] vérifie l’expérience, l’enregistre et la démarre. Si elle ne peut pas
s’exécuter, le problème est montré et son nœud sélectionné. Sinon la chronologie s’ouvre
et les nœuds s’allument au fur et à mesure que l’exécution les atteint : ● en cours, ✓ réussi, ✕ échoué,
↻ réessai, ⟳ répétition, ⚡ sous charge ; les fils qui ont porté le flux sont
colorés aussi.

- Les attentes, les émulateurs et les relais de dégradation ouvrent leurs ports avant la première
  étape, pour que rien de ce qui arrive tôt ne soit manqué. Un port qui ne peut pas être ouvert
  (un autre programme le détient, par exemple) empêche l’exécution de démarrer, et le nœud qui
  en a besoin est montré.
- [[ui:common.stop]] met fin à l’exécution tout de suite : pauses, attentes et charges comprises.
  L’exécution est aussi une tâche dans le bandeau des tâches de la console, qui peut aussi l’arrêter.
- Une exécution qui dure plus de 300 secondes est arrêtée et échoue.
- Pendant qu’une exécution est en cours, l’expérience ne peut pas être modifiée ; déplacer, zoomer et
  sélectionner fonctionnent toujours.

▾ [[ui:exp.runWith]] à côté du bouton exécute une fois avec un autre profil, d’autres valeurs
de paramètres ou une graine donnée ; l’expérience elle-même n’est pas modifiée, et le
formulaire garde ce que vous avez tapé jusqu’à la fermeture de l’application ([Données et
modèles](data.md)).

### La chronologie d’exécution {#timeline}

La [[ui:exp.timeline]] liste une ligne par étape : l’heure, le nœud et ce qui
s’est passé — réussi, échoué et pourquoi, un réessai, la progression d’une répétition, les chiffres d’une
charge une fois par seconde. Cliquez une ligne pour sélectionner son nœud sur le canevas.

Sa ligne de titre contient :

- le résultat : [[ui:exp.passed]], [[ui:exp.failed]] avec la raison, ou
  [[ui:exp.stopped]] ;
- [[ui:exp.reportSaved]] — le fichier de rapport de l’exécution (sur un serveur, un téléchargement) ;
- [[ui:exp.compare]] — cette exécution à côté d’une précédente de la même expérience ;
- le profil et les valeurs modifiées que l’exécution a utilisés, quand elle en avait ;
- la graine de l’exécution, avec [[ui:exp.pinSeed]] pour la garder dans l’expérience afin que les
  exécutions suivantes tirent les mêmes valeurs aléatoires, ou [[ui:exp.unpinSeed]] une fois qu’elle est
  épinglée.

Quand l’[Inspecteur](../tools/inspector.md) capturait, une attente (ou une
réponse attendue) qui a correspondu renvoie à la trame qu’elle a fait correspondre ; un clic l’ouvre dans
l’Inspecteur. Les rapports, les graines et la comparaison d’exécutions sont dans [Exécutions et
rapports](runs.md).

## Mode focus et plein écran {#focus}

▢ [[ui:exp.focus]] masque la barre latérale, l’en-tête et le panneau inférieur (console
et Inspecteur), laissant l’écran à l’éditeur. ⛶ [[ui:exp.fullscreen]] met
la fenêtre en plein écran et active le mode focus ; quitter le plein écran remet le mode
focus comme il était. Passer à un autre écran quitte le mode focus.

<kbd>Esc</kbd> sur le canevas, quand rien d’autre n’est à fermer, quitte le plein écran puis
le mode focus.

## Enregistrement et fichiers {#files}

L’expérience s’enregistre d’elle-même, un moment après chaque modification, dans
`experiment.json` dans le dossier de données (`Documents/SignalLab` sur un bureau ; un
serveur a le sien — voir [Fichiers et dossiers](../reference/files.md)). Les brouillons
qui ne peuvent pas encore s’exécuter sont enregistrés aussi. Une exécution enregistre d’abord, donc ce qui a été exécuté est ce qui
est sur le disque.

Si ce fichier ne peut pas être lu — modifié à la main en JSON cassé, par exemple — l’éditeur
dit quel fichier et pourquoi, et le laisse tranquille. Ouvrir une autre expérience depuis
☰ [[ui:exp.documents]] le remplace alors au prochain enregistrement.

☰ [[ui:exp.documents]] ouvre la boîte de dialogue des expériences :

- [[ui:exp.templates]] : choisissez-en un et appuyez sur [[ui:exp.openDocument]]. Ils utilisent
  tous des adresses de bouclage.
- [[ui:exp.importJson]] lit un fichier d’expérience — écrit par cette version de
  Signal Lab ou une précédente, jusqu’à 4 MiB — et montre son nom et combien de
  nœuds et de connexions il a avant que vous l’ouvriez. Les fichiers des versions précédentes
  sont mis à jour à l’ouverture. Un fichier qui ne s’analyse pas, ou qui ne serait pas
  une expérience valide, est refusé avec la raison, et l’expérience actuelle
  reste.
- [[ui:exp.exportJson]] écrit un instantané de l’expérience actuelle — nœuds,
  fils, positions, paramètres et profils, jamais les valeurs de secrets — dans un nouveau
  fichier dans `exports` dans le dossier de données et montre son chemin ; sur un serveur, avec un
  lien [[ui:common.download]].
- [[ui:exp.openDocument]] remplace l’expérience actuelle par le modèle ou
  le fichier. Elle ne l’exécute pas, et <kbd>Ctrl</kbd>+<kbd>Z</kbd> ramène la précédente.

| Modèle | Ce qu’il fait |
| --- | --- |
| [[ui:exp.templateEmpty]] | [[ui:exp.node.start]] et [[ui:exp.node.end]], pour votre propre flux |
| [[ui:exp.templateHttp]] | Un GET vers `http://127.0.0.1:8080/` et une vérification du statut 200 — l’expérience avec laquelle vous commencez |
| [[ui:exp.templateBranch]] | La même requête ; sur 200 un message OSC vers `127.0.0.1:9000`, sinon un délai de 500 ms |
| [[ui:exp.templateParallel]] | Deux branches à la fois — une requête et une ligne de journal — jointes avant [[ui:exp.node.end]] |
| [[ui:exp.templatePingReply]] | Envoie `/ping` avec l’id de l’exécution vers `127.0.0.1:9000` et attend sur `127.0.0.1:9001` un `/pong` qui le rapporte |
| [[ui:exp.templatePoll]] | Demande `/status` à un appareil toutes les 0,3 s jusqu’à ce qu’il réponde `ready`, dix fois au plus |
| [[ui:exp.templateFlaky]] | Une API émulée qui échoue deux fois avant de répondre, et une boucle qui demande jusqu’à ce qu’elle réponde |
| [[ui:exp.templateFaults]] | Des datagrammes vers un appareil émulé à travers un relais de dégradation pendant qu’une branche parallèle fait passer le réseau de propre à avec pertes, hors ligne, puis de nouveau propre |
| [[ui:exp.templateOutage]] | Une API émulée mise en panne deux secondes par une branche parallèle, et un client qui continue de demander jusqu’à ce qu’elle réponde de nouveau |
| [[ui:exp.templateWsEcho]] | Se connecte à un service d’écho à `ws://127.0.0.1:9001/echo`, envoie un ping JSON, s’attend à le recevoir inchangé, et ferme |

Les mêmes fichiers s’exécutent sans l’éditeur : `signallab run experiment.json` — voir
[La ligne de commande](../automation/cli.md).

## Raccourcis clavier {#shortcuts}

Les touches seules agissent sur le canevas et sont laissées tranquilles pendant que vous tapez dans un champ. Sur
un Mac, dans un navigateur, <kbd>Cmd</kbd> fonctionne là où <kbd>Ctrl</kbd> est écrit.

| Touches | Action |
| --- | --- |
| <kbd>A</kbd> | Ajouter un nœud après celui sélectionné, ou au milieu de la vue quand rien n’est sélectionné |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:exp.sendNow]], ou [[ui:exp.listenNow]], sur le nœud sélectionné |
| <kbd>Ctrl</kbd>+<kbd>Z</kbd> | Annuler |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd>, <kbd>Ctrl</kbd>+<kbd>Y</kbd> | Rétablir |
| <kbd>Ctrl</kbd>+<kbd>A</kbd> | Sélectionner tous les nœuds |
| <kbd>Ctrl</kbd>+<kbd>C</kbd> / <kbd>Ctrl</kbd>+<kbd>X</kbd> / <kbd>Ctrl</kbd>+<kbd>V</kbd> | Copier / couper / coller les nœuds sélectionnés et les fils entre eux |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | Dupliquer les nœuds sélectionnés |
| <kbd>Delete</kbd>, <kbd>Backspace</kbd> | Retirer le fil sélectionné, ou les nœuds sélectionnés |
| Flèches (un nœud sous le focus) | Déplacer les nœuds sélectionnés de 5 pixels ; avec <kbd>Shift</kbd>, 20 |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | Trouver un nœud |
| <kbd>Ctrl</kbd>+<kbd>0</kbd> | Ajuster le graphe |
| <kbd>Ctrl</kbd>+<kbd>1</kbd> | Zoomer à 100 % |
| <kbd>Ctrl</kbd> + molette de la souris | Zoomer autour du pointeur |
| <kbd>Shift</kbd> + clic, <kbd>Ctrl</kbd> + clic | Ajouter un nœud à la sélection, ou l’en retirer |
| <kbd>Shift</kbd> + glisser sur le canevas vide | Sélectionner au cadre |
| Double-clic sur un nœud | Modifier ses champs |
| Double-clic sur le canevas vide | Y ajouter un nœud |
| <kbd>Enter</kbd>, <kbd>Space</kbd> sur une sortie sous le focus | Démarrer un fil depuis elle |
| `{{` ou <kbd>Ctrl</kbd>+<kbd>Space</kbd> dans un champ | Suggérer des paramètres, des variables, des secrets et des générateurs |
| <kbd>Esc</kbd> dans un champ | Revenir au nœud sur le canevas |
| <kbd>Esc</kbd> sur le canevas | Fermer le menu d’ajout ; sinon annuler le fil en cours de tracé ; sinon relâcher le fil sélectionné ; sinon vider une sélection de plusieurs ; sinon quitter le plein écran ; sinon quitter le mode focus |

Dans le menu d’ajout et le chercheur, <kbd>↑</kbd> <kbd>↓</kbd> choisissent,
<kbd>Enter</kbd> prend le choix et <kbd>Esc</kbd> ferme. Tous les raccourcis de
l’application sont dans [Raccourcis clavier](../reference/shortcuts.md).

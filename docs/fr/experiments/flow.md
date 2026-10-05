---
title: "Comment avance une exécution"
description: "Début et Fin, sorties et fils, branches parallèles et Join, branchement, réessai, répétition et Boucle, les attentes qui écoutent dès le début de l’exécution, et ce qui est vérifié avant une exécution."
---

# Comment avance une exécution

Une exécution démarre à [[ui:exp.node.start]], suit les fils de nœud en nœud et est
terminée quand chaque branche a fini et que [[ui:exp.node.end]] a été
atteint. Cette page explique les règles qu’elle suit ; ce que fait chaque nœud est dans
[la référence des nœuds](nodes.md), et les valeurs qui voyagent avec elle dans
[données](data.md).

## Début et Fin {#start-end}

Une expérience a exactement un [[ui:exp.node.start]] et une
[[ui:exp.node.end]].

- [[ui:exp.node.start]] n’a pas d’entrée. Il passe aussitôt, et sa ligne dans la
  chronologie donne la graine de l’exécution. Sa sortie peut avoir plusieurs fils :
  l’expérience commence alors par des branches parallèles.
- Chaque branche qui atteint [[ui:exp.node.end]] s’y arrête. End s’affiche comme
  en cours dès la première arrivée et passe une fois, après que la dernière branche a
  fini — et pas du tout si une étape a échoué. C’est ce passage qui rend
  l’exécution [[ui:exp.passed]].
- Une exécution où chaque branche a fini sans erreur mais aucune n’a atteint End
  échoue avec `run.no_end`.

## Sorties et fils {#outputs}

L’étape d’un nœud se termine en choisissant une sortie, et l’exécution suit chaque fil de
cette sortie. La plupart des nœuds ont une seule sortie, [[ui:exp.outputPort]] ; certains choisissent
entre plusieurs :

| Nœud | Sorties qui doivent être reliées | Sorties qui peuvent être reliées |
| --- | --- | --- |
| [[ui:exp.node.end]] | — | — |
| [[ui:exp.node.fork]] | [[ui:exp.branch1]], [[ui:exp.branch2]] | — |
| [[ui:exp.node.branch_status]], [[ui:exp.node.branch_value]] | [[ui:exp.yes]], [[ui:exp.no]] | — |
| Chaque attente ([[ui:exp.node.wait_osc]], [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_http]], [[ui:exp.node.wait_ws]]) | [[ui:exp.portMatched]] | [[ui:exp.portTimeout]] |
| [[ui:exp.node.loop]] | [[ui:exp.portBody]], [[ui:exp.portDone]] | [[ui:exp.portLimit]] |
| Tout autre nœud | [[ui:exp.outputPort]] | — |

Pour relier, faites glisser d’une sortie vers un nœud ; déposée sur un canevas vide, elle ajoute
un nouveau nœud à cet endroit. Faire glisser depuis une sortie qui a déjà un fil en ajoute un autre.
[[ui:exp.addNext]], la touche <kbd>A</kbd> et le ＋ sur un fil insèrent un nœud
dans le fil existant à la place.

Un graphe inachevé est un brouillon : il s’enregistre, mais il ne s’exécute pas.
[[ui:exp.needsLinks]] dans la barre d’outils dit ce qui manque et montre le nœud.
Voir [ce qui est vérifié avant une exécution](#validation).

## Branches parallèles {#parallel}

### Plusieurs fils depuis une sortie {#fan-out}

Quand une sortie a plusieurs fils — celui de [[ui:exp.node.start]] compris — chaque
nœud vers lequel ils mènent s’exécute en même temps. Le premier fil poursuit la branche ;
chaque fil supplémentaire démarre une branche parallèle. Chaque branche porte sa propre copie des
variables et de la dernière réponse HTTP, si bien que ce qu’une branche définit ou
reçoit n’est pas vu par les autres.

### Branche parallèle et Join {#fork-join}

[[ui:exp.node.fork]] passe aussitôt et sort par les deux
[[ui:exp.branch1]] et [[ui:exp.branch2]] — la même chose que deux fils depuis une
sortie, dessiné comme un nœud.

[[ui:exp.node.join]] attend **chaque** fil qui y mène, puis
continue comme une seule branche avec les copies fusionnées dans l’ordre de ces fils :

- les variables de tous — sur un nom que deux branches définissent toutes deux, le fil
  listé plus tard dans l’expérience l’emporte ;
- la réponse HTTP du dernier fil, dans cet ordre, qui en apporte une ;
- pour les attentes après lui, la plus ancienne de leurs dernières actions.

L’ordre des fils décide, jamais la branche qui s’est trouvée finir en premier.

::: warning Ne joignez que ce qui s’exécute en parallèle
Un Join compte ses fils, quelle que soit la façon dont ils en sont venus à s’exécuter en parallèle : depuis un
[[ui:exp.node.fork]], depuis plusieurs fils d’une sortie, depuis des chemins séparés.
Derrière le [[ui:exp.yes]] et le [[ui:exp.no]] d’une branche, un seul chemin s’exécute, donc un
Join alimenté par les deux attend une branche qui n’arrive jamais : l’exécution échoue avec
`run.join_waiting`, en nommant combien de fils n’ont jamais été suivis. Pour réunir des
chemins alternatifs, reliez-les directement au nœud suivant.
:::

Un nœud qui n’est pas un Join, atteint par deux branches parallèles, s’exécute une fois pour
chacune d’elles.

### Quand une étape échoue {#failure}

Le premier échec fait échouer l’exécution. Les autres branches ne démarrent aucune nouvelle étape : une
répétition ou une charge se termine tôt, toute autre étape où elles se trouvent va jusqu’à sa fin. Un
échec qu’elles rencontrent entre-temps est signalé dans la chronologie mais n’est pas l’erreur de
l’exécution. Une étape qui a rencontré un délai d’attente alors qu’un
fil [[ui:exp.portTimeout]] était là n’a pas échoué — voir [attentes](#timeout).

## Branchement {#branching}

| Nœud | Sort par [[ui:exp.yes]] quand |
| --- | --- |
| [[ui:exp.node.branch_status]] | la dernière réponse HTTP sur ce chemin a le statut donné |
| [[ui:exp.node.branch_value]] | sa comparaison tient — voir [comparer des valeurs](data.md#compare) |

Sinon, chacun sort par [[ui:exp.no]]. Un branchement sur statut a besoin d’une requête
HTTP avant lui sur chaque chemin ; un branchement sur valeur a besoin que les noms qu’il lit soient
connus là. Quand les chemins après [[ui:exp.yes]] et [[ui:exp.no]] se rejoignent, le nœud où ils se rejoignent s’exécute une fois, et seules les variables définies sur
les deux chemins y sont connues ([où une variable est connue](data.md#visibility)).

## Réessai {#retry}

Une étape qui envoie ou écoute peut réessayer quand elle échoue : activez
[[ui:exp.retryOn]] dans ses propriétés.

| Réglage | Quoi | Plage | Valeur initiale |
| --- | --- | --- | --- |
| [[ui:exp.attempts]] | tentatives au total, la première comprise | 1–10 | 3 |
| [[ui:exp.retryDelay]] | la pause avant la deuxième tentative | 0–60 000 ms | 500 |
| [[ui:exp.backoff]] | [[ui:exp.backoff.fixed]] : chaque pause identique ; [[ui:exp.backoff.exponential]] : chaque pause deux fois la précédente | — | [[ui:exp.backoff.fixed]] |

- Le réessai s’applique à [[ui:exp.node.http]], [[ui:exp.node.tcp]],
  [[ui:exp.node.mqtt]], [[ui:exp.node.osc]], [[ui:exp.node.udp]],
  [[ui:exp.node.ws_connect]], [[ui:exp.node.ws_send]] et chaque attente. Les autres
  nœuds le refusent (`node.retry_unsupported`), et une requête HTTP sous
  [charge](load.md) ne prend pas de réessai.
- Aucune pause n’est plus longue que 60 s, quoi que donne le doublement.
- Chaque tentative échouée apparaît dans la chronologie comme [[ui:exp.retry]], avec son
  numéro et sa raison. L’étape passe ensuite, ou échoue avec la raison de la dernière
  tentative.
- Seule l’exécution est répétée. Un champ dont le modèle ne se résout pas
  échoue aussitôt.
- Un envoi qui attend sa réponse envoie de nouveau. Une attente attend de nouveau, en comptant
  depuis la dernière action de la branche comme avant.
- Une attente avec un fil [[ui:exp.portTimeout]] n’échoue pas sur un délai d’attente, donc elle
  n’est pas réessayée : elle suit [[ui:exp.portTimeout]].
- [Stop](#stop) met fin à une pause aussitôt.

## Répétition {#repeat}

Une étape qui envoie peut envoyer encore et encore — un battement, une interrogation, un flux
régulier — sans boucle dans le graphe : activez [[ui:exp.repeatOn]].

| Réglage | Quoi | Plage | Valeur initiale |
| --- | --- | --- | --- |
| [[ui:exp.repeatBy]] | [[ui:exp.repeatBy.count]] ou [[ui:exp.repeatBy.duration]] | — | [[ui:exp.repeatBy.count]] |
| [[ui:exp.repeatCount]] | envois au total, le premier compris | 2–10 000 | 10 |
| [[ui:exp.repeatDuration]] | combien de temps continuer d’envoyer, depuis le premier envoi | 1–300 000 ms | 10 000 |
| [[ui:exp.repeatInterval]] | la pause entre deux envois | 10–60 000 ms | 1000 |
| [[ui:exp.repeatJitter]] | chaque pause est plus longue de ce montant au plus, au hasard | 0–60 000 ms | 0 |

- La répétition s’applique à [[ui:exp.node.http]], [[ui:exp.node.tcp]],
  [[ui:exp.node.mqtt]], [[ui:exp.node.osc]], [[ui:exp.node.udp]] et
  [[ui:exp.node.ws_send]] (`node.repeat_unsupported` ailleurs). Une requête HTTP
  a soit la répétition, soit la [charge](load.md), pas les deux.
- Chaque envoi est fait comme un seul le serait : ses modèles sont relus —
  `{{counter}}` est le numéro de l’envoi, `{{now}}` son heure — et le réessai, quand il est actif,
  s’applique à chaque envoi. Un envoi qui attend une réponse attend la sienne.
- Pour une durée, un envoi n’est fait que s’il peut démarrer avant la fin du temps.
- La gigue est tirée de la graine de l’exécution : la même graine donne les mêmes
  pauses.
- La chronologie signale la progression comme [[ui:exp.repeating]] au plus une fois par seconde.
  L’étape passe après le dernier envoi, avec le résultat de cet envoi ; un envoi qui
  échoue définitivement fait échouer l’étape.
- Un échec sur une autre branche met fin aux envois ; [Stop](#stop) met fin à une pause
  aussitôt.

Elle doit tenir dans une exécution : les envois et leurs plus longues pauses
(`(count − 1) × (interval + jitter)`) 300 s au plus (`node.repeat_too_long`),
et une répétition minutée 10 000 envois au plus (`node.repeat_too_many`).

## Boucle {#loop}

[[ui:exp.node.loop]] exécute les étapes de sa sortie [[ui:exp.portBody]] encore et
encore ; la dernière d’entre elles est reliée en retour à la Boucle.

| Réglage | Quoi | Plage |
| --- | --- | --- |
| [[ui:exp.loopMax]] | le plus grand nombre d’itérations | 1–1000 |
| [[ui:exp.loopUntilOn]] | une condition de sortie : [[ui:exp.value]], [[ui:exp.operator]], [[ui:exp.expected]], comme dans [[ui:exp.node.assert_value]] | facultatif |

1. Atteinte depuis l’extérieur, la Boucle démarre l’itération 1 sur [[ui:exp.portBody]].
2. Chaque fois que le corps revient, la condition de sortie est lue — après
   l’itération, donc le corps s’exécute toujours au moins une fois et peut définir ce qu’il teste.
3. Quand la condition tient, la Boucle sort par [[ui:exp.portDone]].
4. Sinon, l’itération suivante démarre, tant qu’il en reste.
5. Quand les itérations s’épuisent d’abord, la Boucle sort par
   [[ui:exp.portLimit]] s’il est relié, et fait échouer l’exécution avec `loop.limit`
   s’il ne l’est pas. Sans condition, le corps s’exécute à chaque itération et la
   Boucle sort par [[ui:exp.portDone]].

À l’intérieur du corps, `{{counter}}` est le numéro de l’itération, puisque chaque nœud
compte ses propres exécutions. La condition et les étapes après [[ui:exp.portDone]] ou [[ui:exp.portLimit]] peuvent
utiliser ce que chaque itération du corps définit — un statut que le corps extrait, par
exemple ; le corps lui-même ne voit que ce qui était connu quand la Boucle a été atteinte.

Le modèle [[ui:exp.templatePoll]] demande son statut à un appareil toutes les 0,3 s
jusqu’à ce qu’il réponde `ready`, 10 fois au plus.

### Ce qu’un corps peut contenir {#loop-body}

Un corps s’exécute comme une seule branche, une itération après l’autre. Le fil qui revient à la
Boucle est le seul cycle qu’une expérience peut avoir ; tout autre est `graph.cycle`.

| Règle | Erreur |
| --- | --- |
| Quelque chose sur [[ui:exp.portBody]] revient à la Boucle | `loop.no_return` |
| Chaque sortie du corps a un seul fil | `loop.body_parallel` |
| Chaque sortie du corps continue dans le corps ou revient à la Boucle | `loop.body_leaves` |
| Seule la sortie [[ui:exp.portBody]] de la Boucle mène dans le corps | `loop.body_entered` |
| Aucun [[ui:exp.node.start]], [[ui:exp.node.end]], [[ui:exp.node.fork]], [[ui:exp.node.join]] ni autre [[ui:exp.node.loop]] dans le corps | `loop.body_unsupported` |

## Attentes {#waits}

Une attente passe quand un message qu’elle attend arrive :
[[ui:exp.node.wait_osc]], [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]],
[[ui:exp.node.wait_http]] et [[ui:exp.node.wait_ws]]. Ce que chacun prend est
dans [la référence des nœuds](nodes.md) ; voici comment ils écoutent.

### Écouter dès le début {#listening}

L’exécution ouvre ce sur quoi ses attentes écoutent **avant sa première étape**, pour qu’une réponse
plus rapide que l’étape suivante ne soit pas manquée :

| Attente | Ouvert avant la première étape |
| --- | --- |
| OSC, UDP | un socket UDP par adresse [[ui:exp.listenOn]], partagé par chaque attente dessus |
| Requête HTTP | un écouteur par adresse — l’[émulateur](faults.md#emulator) HTTP de l’exécution quand il y en a un, sinon un qui répond `204` |
| MQTT | une connexion par broker et filtre de topic, abonnée ; les messages retenus que le broker rejoue alors sont ignorés |
| WebSocket | rien : elle lit la connexion qu’un [[ui:exp.node.ws_connect]] a ouverte quand il s’est exécuté |

Parce qu’ils s’ouvrent en premier, ces adresses sont fixées avant l’exécution : une attente OSC,
UDP ou HTTP écoute sur un `IP:port` littéral avec un port différent de 0, et
le broker et le topic d’une attente MQTT n’acceptent que des paramètres. Un port qui ne peut pas être
ouvert — pris, ou qui n’est pas une adresse de cet ordinateur — arrête l’exécution avant tout
trafic, au champ de cette attente. Tout est fermé quand l’exécution se termine, de quelque
façon que ce soit.

### Quels messages comptent {#counting}

Une attente considère les messages arrivés après que la **dernière action sur sa
branche** a démarré — la dernière requête, le dernier message, la dernière publication, la dernière connexion WebSocket ou le dernier
envoi — ou, avant toute action, après le démarrage de l’exécution. Un message d’avant la
requête ne compte pas, et un délai ou un journal entre la requête et l’attente
ne cache pas sa réponse. Après un Join, la plus ancienne des dernières actions des branches
fusionnées compte.

Une attente prend le premier message qui correspond et le consomme : deux attentes ne
prennent jamais le même message.

Chaque socket, abonnement ou connexion conserve au plus 1024 messages et 64 Mio
pour ses attentes ; au-delà, les plus anciens sont abandonnés et comptés.

### Délai d’attente {#timeout}

[[ui:exp.waitTimeout]] vaut 1–120 000 ms, 2000 au départ. Quand rien ne correspond à
temps :

- avec un fil [[ui:exp.portTimeout]], l’attente le suit ;
- sans, l’étape échoue avec `wait.timeout`, qui dit combien d’autres
  messages sont arrivés entre-temps — un mauvais motif a l’air différent d’un appareil
  silencieux — et, dans son détail, combien de messages plus anciens ont été abandonnés quand la
  file était pleine.

La variable de l’attente — `reply`, ou `request` pour HTTP — n’existe qu’après
[[ui:exp.portMatched]]. Quand l’[[ui:dock.inspector]] capture, l’étape
relie aussi la trame qu’elle a prise : voir [la chronologie](runs.md#timeline).

## Une réponse à la même étape {#reply}

Un [[ui:exp.node.osc]] ou un [[ui:exp.node.udp]] peut attendre sa propre réponse :
activez [[ui:exp.expectReply]].

| Réglage | Quoi | Valeur initiale |
| --- | --- | --- |
| [[ui:exp.replyOn]] | l’adresse sur laquelle la réponse est attendue ; le port 0 signifie n’importe quel port libre | `0.0.0.0:0` |
| [[ui:exp.replyAddress]] (OSC), [[ui:exp.replyMode]] (UDP) | ce que la réponse doit être, comme dans l’attente correspondante | n’importe quoi |
| [[ui:exp.waitTimeout]] | 1–120 000 ms | 2000 |
| [[ui:exp.replyVariable]] | la variable dans laquelle la réponse est écrite | `reply` |

Le socket sur [[ui:exp.replyOn]] est ouvert avant la première étape, comme celui d’une
attente, et le message **part de lui** : un appareil qui répond au port même de
l’expéditeur est entendu, et un appareil qui répond à un port fixe est entendu quand
ce port est celui donné. L’étape passe avec une réponse qui correspond, et la
variable existe après sa sortie. Il n’y a pas de sortie [[ui:exp.portTimeout]] : aucune réponse à
temps fait échouer l’étape, et le réessai peut envoyer de nouveau. Pour brancher sur un silence, utilisez une
attente séparée.

## Ce qui est vérifié avant une exécution {#validation}

L’éditeur vérifie l’expérience au fur et à mesure que vous la modifiez ; le bouton Exécuter la vérifie une fois
de plus. Un problème nomme le nœud, et le champ quand il y en a un.

| Règle | Erreur |
| --- | --- |
| L’expérience a un nom | `doc.name_required` |
| 1–64 nœuds, exactement un Début et une Fin | `doc.node_count`, `doc.start_end_count` |
| Le Début n’a pas d’entrée | `graph.start_input` |
| Un fil mène à un autre nœud qui existe | `doc.connection_invalid` |
| Le même fil n’est pas là deux fois | `doc.connection_duplicate` |
| Chaque sortie qui doit être reliée l’est | `graph.outputs_required` |
| Un nœud n’a pas de fil sur une sortie qu’il n’a pas | `graph.port_unexpected` |
| Chaque nœud peut être atteint depuis le Début | `graph.unreachable` |
| Aucun cycle sauf le fil d’une Boucle qui revient | `graph.cycle`, et les [règles du corps](#loop-body) |
| Une vérification ou un Extract a une requête HTTP avant lui sur chaque chemin ; une requête sous charge ne compte pas | `graph.needs_http` |
| Chaque modèle s’analyse, et chaque nom qu’il utilise est connu sur chaque chemin | `template.*`, `name.*` — voir [données](data.md#unknown-names) |
| Chaque champ est présent et dans la plage | `node.*` |
| Un nœud qui en nomme un autre — [[ui:exp.node.impairment_change]], [[ui:exp.node.emulator_state]], les nœuds WebSocket — en nomme un qui existe, et un nœud WebSocket vient après son connect | `impair.relay_unknown`, `emulator.node_unknown`, `ws.connection_unknown`, `ws.connection_after` |
| Deux des sockets de l’exécution ne partagent pas un port | voir [défaillances](faults.md#ports) |

L’exécution vérifie ensuite ce dont elle a besoin pour démarrer : chaque [secret](data.md#secret-check)
stocké, chaque port ouvert. Tant que tout ne tient pas, aucune étape ne s’exécute et rien n’est
envoyé.

## Durée limite {#limit}

Une exécution dure au plus 300 s. Une exécution encore en cours est alors arrêtée et échoue avec
`run.timeout`. Depuis la [ligne de commande](../automation/cli.md#cli-run) et
[l’API](../api/run.md) la limite peut être plus courte, 1–300 s.

## Stop {#stop}

Pendant qu’une exécution se déroule, le bouton Exécuter est [[ui:common.stop]]. Stop met fin à l’exécution
aussitôt : chaque branche, chaque pause du réessai ou de la répétition, chaque attente et chaque charge —
les requêtes en vol sont abandonnées. Ses sockets, abonnements, émulateurs et
relais se ferment, et ses connexions WebSocket envoient une trame de fermeture.
[[ui:app.stopAll]] dans l’en-tête fait la même chose pour chaque tâche. Une exécution arrêtée
n’enregistre aucun rapport ; voir [exécutions](runs.md#stop).

---
title: "Exécutions et résultats"
description: "Démarrer une exécution avec les valeurs de l’expérience ou d’autres, la chronologie, l’arrêt, ce que signifient réussi et échoué, le rapport d’exécution, les graines, essayer un seul nœud, et les fichiers d’expérience avec leurs versions."
---

# Exécutions et résultats

## Démarrer une exécution {#start}

Appuyez sur [[ui:exp.run]] dans la barre d’outils de l’éditeur. Avant que quoi que ce soit ne soit envoyé :

1. L’expérience est vérifiée comme l’éditeur la vérifie — son graphe, ses champs,
   ses noms et ses valeurs ([ce qui est vérifié](flow.md#validation)) — et chaque secret
   qu’elle utilise doit être stocké ([secrets](data.md#secret-check)).
2. Elle est enregistrée.
3. L’exécution ouvre ce dont elle a besoin pour toute sa durée : ses émulateurs, ses
   relais de dégradation, les sockets sur lesquels ses attentes écoutent et ses abonnements
   MQTT.

Si l’un de ces points échoue, rien ne s’exécute : le problème est affiché, et le nœud
concerné est sélectionné. Sinon, la chronologie s’ouvre sous le canevas et les étapes
y apparaissent au fur et à mesure. Pendant que l’exécution se déroule, [[ui:exp.run]] devient
[[ui:common.stop]] et l’expérience ne peut pas être modifiée.

L’exécution utilise les valeurs du profil actif, et la graine épinglée dans
l’expérience ou une nouvelle. Pour exécuter une fois avec d’autres, utilisez [[ui:exp.runWith]].

### Exécuter avec d’autres valeurs {#run-with}

Le ▾ à côté de [[ui:exp.run]] ouvre [[ui:exp.runWith]] : d’autres valeurs pour une
exécution, sans modifier l’expérience.

| Champ | Quoi | Vide |
| --- | --- | --- |
| [[ui:exp.profile]] | le profil pour cette exécution ; affiché quand l’expérience a des profils | le profil actif |
| chaque paramètre | une valeur pour cette exécution seulement | la valeur du profil choisi, affichée en gris |
| [[ui:exp.seed]] | la graine pour cette exécution, 0–9 007 199 254 740 991 ; le bouton à côté remplit la graine de la dernière exécution | la graine épinglée, ou une nouvelle |

[[ui:exp.run]] dans le formulaire démarre l’exécution ; [[ui:exp.resetOverrides]] vide le
formulaire. Ce que vous avez saisi reste dans le formulaire pour la session, si bien que le même changement est
un clic la fois suivante. Un profil qui ne s’exécuterait pas est marqué ⚠. La
priorité des valeurs est dans [données](data.md#precedence).

Quand l’expérience a des profils, ou qu’une exécution a eu des valeurs saisies pour elle, la
chronologie dit quel profil l’exécution a utilisé — ou [[ui:exp.runDefaults]] — et,
quand des valeurs ont été saisies, [[ui:exp.overridden]].

## La chronologie {#timeline}

[[ui:exp.timeline]] se trouve sous le canevas ; ▸ et ▾ la replient, son bord la redimensionne.
Elle contient une ligne par événement d’étape, du plus ancien au plus récent : l’heure, le nœud, son
état et ce qui s’est passé — `HTTP 200 · 41 ms`, `token = abc123`,
`/pong 42 ← 127.0.0.1:9000 · 12 ms`. Un échec en dit la raison, son détail technique
dans l’info-bulle. Un clic sur une ligne sélectionne son nœud sur le canevas.

| État | L’étape |
| --- | --- |
| [[ui:exp.running]] | a démarré |
| [[ui:exp.passed]] | s’est terminée correctement, et a choisi sa sortie |
| [[ui:exp.failed]] | a échoué ; le premier échec est celui de l’exécution |
| [[ui:exp.retry]] | a échoué à une tentative et réessaiera ([réessai](flow.md#retry)) |
| [[ui:exp.repeating]] | envoie encore et encore, au plus une ligne par seconde ([répétition](flow.md#repeat)) |
| [[ui:exp.load]] | est sous charge, une ligne par seconde ([charge](load.md#progress)) |

Sur le canevas, chaque nœud porte une pastille avec son dernier état.

L’en-tête de la chronologie contient :

- le résultat : [[ui:exp.running]], [[ui:exp.passed]], [[ui:exp.failed]] avec la
  raison, ou [[ui:exp.stopped]] ;
- [[ui:exp.reportSaved]] une fois le rapport écrit — dans un navigateur un lien qui
  le télécharge, dans l’application de bureau son chemin dans l’info-bulle ;
- [[ui:exp.compare]], pour placer cette exécution à côté d’une précédente
  ([comparer des exécutions](load.md#compare)) ;
- le profil et les valeurs modifiées, comme ci-dessus ;
- la graine de l’exécution avec [[ui:exp.pinSeed]], ou, quand l’expérience en a une
  épinglée, cette graine avec [[ui:exp.unpinSeed]] ([graines](#seeds)).

**Trames.** Pendant que l’[[ui:dock.inspector]] capture, une attente — ou un envoi
qui attend sa réponse — qui a pris un message conserve le numéro de la trame de ce
message. Un bouton sous les lignes nomme le nœud et la trame ; il
ouvre l’Inspecteur dans le panneau inférieur avec cette trame sélectionnée. Voir
l’[Inspecteur](../tools/inspector.md).

La chronologie montre la dernière exécution de l’expérience dans cette session ; elle est
vidée quand une autre expérience est ouverte.

## Stop {#stop}

Appuyez sur [[ui:common.stop]], ou [[ui:app.stopAll]] dans l’en-tête pour chaque tâche à
la fois. L’exécution se termine aussitôt ([ce qui s’arrête](flow.md#stop)), la chronologie montre
[[ui:exp.stopped]], et **aucun rapport n’est enregistré**. Une exécution lancée depuis la ligne de commande
ou l’API sur un serveur est une tâche comme une autre : [[ui:app.stopAll]] sur ce
serveur l’arrête aussi, et son appelant apprend qu’elle a été arrêtée.

## Le résultat {#result}

| Résultat | Signifie | Rapport |
| --- | --- | --- |
| [[ui:exp.passed]] | chaque branche a fini, aucune étape n’a échoué, et la Fin a été atteinte | enregistré |
| [[ui:exp.failed]] | une étape a échoué — une vérification, une attente sans fil [[ui:exp.portTimeout]], une erreur réseau, un seuil — ou l’exécution a manqué de temps (`run.timeout`), un Join a attendu en vain (`run.join_waiting`), ou aucune branche n’a atteint la Fin (`run.no_end`) | enregistré, avec le premier échec |
| [[ui:exp.stopped]] | quelqu’un l’a arrêtée | aucun |
| n’a pas démarré | l’expérience est invalide, un secret manque, ou un port n’a pas pu être ouvert | aucun |

Une exécution échouée nomme le nœud et le champ de son premier échec ;
[la référence des erreurs](../reference/errors.md) liste chaque code. La ligne de commande
dit la même chose avec son code de sortie : `0` réussi, `1` échoué, `2` l’expérience ou
l’appel était invalide (un secret manquant compte), `3` quelque chose hors de
l’expérience l’a empêchée de s’exécuter, comme un port qui n’a pas pu être ouvert. Voir
[`signallab run`](../automation/cli.md#cli-run).

## Le rapport d’exécution {#report}

Chaque exécution qui se termine d’elle-même — réussie ou échouée — écrit un rapport JSON dans le
dossier `runs` du dossier de données : `Documents/SignalLab/runs` sur un poste de bureau, le
dossier de données propre au serveur sur un serveur ([fichiers](../reference/files.md)). Le fichier
est `run-<start time in ms>-<job number>.json` ; un rapport n’est jamais écrit par-dessus
un autre. S’il ne peut pas être écrit, l’éditeur en dit la raison.

| Clé | Quoi |
| --- | --- |
| `version` | le format du rapport, désormais 5 |
| `experiment` | le nom de l’expérience |
| `document_version` | la version de l’expérience, désormais 9 |
| `seed` | la graine que l’exécution a utilisée |
| `profile` | le profil avec lequel elle a été exécutée, ou `null` pour les valeurs par défaut |
| `overrides` | les valeurs saisies dans [[ui:exp.runWith]] |
| `params` | chaque valeur de paramètre que l’exécution a utilisée |
| `started_ms`, `ended_ms` | millisecondes Unix |
| `outcome` | `passed` ou `failed` |
| `error` | le premier échec, ou `null` |
| `steps` | chaque événement d’étape, dans l’ordre (ci-dessous) |
| `emulators` | les compteurs de chaque nœud Émulateur — présent quand il y en a un ([émulateurs](faults.md#emulator)) |
| `impairments` | les compteurs et phases de chaque nœud Dégradation — présent quand il y en a un ([phases](faults.md#change-impairment)) |

Chaque événement d’étape a :

| Clé | Quoi |
| --- | --- |
| `job_id`, `node_id` | l’exécution et le nœud |
| `ts` | millisecondes Unix |
| `state` | `running`, `passed`, `failed`, `retry`, `repeating`, `load` |
| `detail` | ce qui s’est passé, en anglais |
| `message_key`, `message_params` | la même chose que le texte de l’interface et ses valeurs, pour que l’étape puisse être affichée dans n’importe quelle langue |
| `vars` | les variables que l’étape a écrites, s’il y en a |
| `error` | pourquoi elle a échoué : `code`, `params`, `node`, `field`, `detail` |
| `frame` | la trame de l’Inspecteur qu’une attente a prise, si la capture était active |
| `load` | ce qu’une charge a mesuré ([mesures](load.md#metrics)), sur son dernier événement |

Les valeurs des secrets n’apparaissent jamais dans un rapport : elles sont masquées sous `••••`
([masquage](data.md#masking)).

Le format du rapport a grandi avec les fonctionnalités : la version 3 a ajouté les compteurs des émulateurs,
la version 4 les phases des dégradations, la version 5 les mesures d’une charge.

Les rapports sont l’historique des exécutions : [[ui:exp.compare]] les lit, et
[`experiment_runs`](../api/commands.md#experiment_runs) aussi. Le `--report` de la ligne de commande
copie le rapport d’une exécution où vous voulez.

## Graines {#seeds}

Chaque exécution a une graine, un nombre entier de 0 à 9 007 199 254 740 991. C’est,
dans l’ordre :

1. la graine donnée à cette exécution dans [[ui:exp.runWith]], en ligne de commande
   (`--seed`) ou à l’API ;
2. la graine épinglée dans l’expérience ;
3. une nouvelle graine aléatoire.

La première ligne de l’exécution dans la chronologie la donne, et le rapport la conserve.

La graine décide de tout ce qu’une exécution fait au hasard : les
[générateurs](data.md#generators) dans les modèles, la gigue de la
[répétition](flow.md#repeat), les arrivées d’une [charge aléatoire](load.md#schedule), le
sort de chaque paquet dans un [relais de dégradation](faults.md#seed) et les choix aléatoires d’un
émulateur. Chacun tire d’un flux qui lui est propre, si bien que les branches parallèles
ne décalent jamais les valeurs les unes des autres.

Pour répéter une exécution :

1. Appuyez sur [[ui:exp.pinSeed]] à côté de sa graine dans la chronologie. La graine est stockée
   dans l’expérience, et chaque exécution l’utilise jusqu’à ce que vous appuyiez sur
   [[ui:exp.unpinSeed]]. Dans [[ui:exp.params]], [[ui:exp.seed]] affiche et modifie
   la graine épinglée ; vide, elle vaut [[ui:exp.seedRandom]].
2. Exécutez avec le même profil et les mêmes valeurs ; le rapport les liste.

Ce qu’une graine ne peut pas répéter : le temps (`{{now}}`), `{{run.id}}`, et le moment où les appareils
et le réseau répondent.

## Essayer un seul nœud {#send-now}

Pour essayer un seul nœud sans exécuter l’expérience, sélectionnez-le et appuyez sur
[[ui:exp.sendNow]] — ou <kbd>Ctrl</kbd>+<kbd>Enter</kbd> dans ses propriétés —
sur un nœud [[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]],
[[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_connect]] ou
[[ui:exp.node.ws_send]]. Sur une attente, c’est [[ui:exp.listenNow]] : elle écoute à partir de maintenant jusqu’à ce qu’un message
corresponde ou que son délai d’attente se termine.

Le moteur exécute le nœud avec le code qu’une exécution utilise, une fois :

- avec les valeurs du profil actif, les valeurs de variables connues dans cette session
  (de la dernière exécution et des essais précédents) et les secrets stockés ;
- avec la graine épinglée, ou une nouvelle ; `{{run.id}}` vaut `0` et `{{counter}}`
  vaut `1` ;
- sans réessai, répétition ni charge — un seul envoi ;
- **sans cookies** : une requête, rien de défini avant elle à renvoyer ;
- sans les relais et émulateurs de l’exécution. Un [[ui:exp.node.wait_http]] écoute
  sur un écouteur qui lui est propre, et un envoi ou une attente WebSocket ouvre la connexion
  que son [[ui:exp.node.ws_connect]] décrit, pour ce seul essai.

Si un nom que le nœud utilise n’a pas encore de valeur, rien n’est envoyé et le résultat dit
quels noms manquent — exécutez l’expérience, ou utilisez [[ui:exp.sendNow]] sur le
nœud qui les définit, d’abord.

Le résultat montre ✓ ou ✕ et ce qui s’est passé. Pour une requête HTTP, il montre aussi
le statut, le temps, la taille et la [[ui:http.response]] ; dans une réponse JSON,
chaque valeur peut être cliquée pour [l’extraire](data.md#extract), et
[[ui:http.mockThis]] transforme la réponse en route d’un émulateur. Les
valeurs qu’une attente a reçues, ou que les nœuds [[ui:exp.node.extract]] juste après une
requête prendraient de sa réponse, deviennent connues de l’aperçu et du
[[ui:exp.sendNow]] suivant. Pendant qu’une exécution se déroule, [[ui:exp.sendNow]] n’est pas
disponible.

L’aperçu d’un nœud modélisable — ce qu’il [enverra](data.md#preview) — est
résolu par le moteur aussi, sans rien envoyer.

## Fichiers d’expérience {#files}

### L’expérience de travail {#working-file}

L’éditeur contient une expérience, enregistrée par lui-même 0,7 s après chaque changement dans
`experiment.json` du dossier de données ; la barre d’outils indique [[ui:exp.saving]],
[[ui:exp.saved]] ou [[ui:exp.saveError]]. Un graphe inachevé s’enregistre aussi. Un fichier
qui ne peut pas être lu est signalé avec son chemin, jamais remplacé. Un fichier d’expérience
fait au plus 4 Mio.

Sur un serveur, le fichier est dans le dossier de données du serveur, si bien que chaque navigateur qui
ouvre l’éditeur là travaille sur la même expérience.

### Ouvrir, modèles et export {#open-export}

Le bouton ☰ dans la barre d’outils ouvre [[ui:exp.documents]] :

- [[ui:exp.templates]] : [[ui:exp.templateEmpty]], [[ui:exp.templateHttp]],
  [[ui:exp.templateBranch]], [[ui:exp.templateParallel]],
  [[ui:exp.templatePingReply]], [[ui:exp.templatePoll]],
  [[ui:exp.templateFlaky]], [[ui:exp.templateFaults]],
  [[ui:exp.templateOutage]], [[ui:exp.templateWsEcho]]. Leurs cibles sont sur
  `127.0.0.1`.
- [[ui:exp.importJson]] lit un fichier de 4 Mio au plus — de cette version du
  format d’expérience ou d’une plus ancienne, qui est mise à jour à l’ouverture —
  et le vérifie avant d’afficher son nom et combien de nœuds et de connexions il
  a. Le fichier doit aussi tenir dans 4 Mio tel que l’éditeur l’écrit, indenté, si bien qu’un
  fichier compact proche de la limite peut être refusé. Un fichier cassé est refusé avec
  la ligne et la colonne du problème, un fichier d’un Signal Lab plus récent avec
  `doc.version_unsupported`, et l’expérience actuelle reste.
- [[ui:exp.openDocument]] remplace l’expérience actuelle par celle choisie.
  <kbd>Ctrl</kbd>+<kbd>Z</kbd> ramène la précédente pendant cette
  session. Ouvrir une expérience ne l’exécute pas.
- [[ui:exp.exportJson]] écrit une copie dans le dossier `exports` du dossier de
  données, sous `experiment-<time in ms>-<random>.json`, jamais par-dessus une autre copie ;
  dans un navigateur, [[ui:common.download]] la récupère.

La ligne de commande et l’API prennent les mêmes fichiers, et les modèles par leur nom :
`empty`, `http-check`, `status-branch`, `parallel-flows`, `osc-ping-reply`,
`poll-until-ready`, `flaky-api`, `fault-phases`, `dependency-outage`,
`websocket-echo`.

### Versions du document {#versions}

Un fichier d’expérience a une `version` ; ce Signal Lab écrit la version 9 et ouvre
toutes les précédentes, en comblant ce que le fichier plus ancien ne pouvait pas contenir. Un fichier d’une
version plus récente que 9 est refusé (`doc.version_unsupported`) plutôt qu’ouvert
sans ce qu’il contient.

| Version | Ajout |
| --- | --- |
| 2 | les paramètres et la graine |
| 3 | les profils |
| 4 | le réessai, et une réponse attendue par un envoi OSC ou UDP |
| 5 | la répétition, et [[ui:exp.node.loop]] |
| 6 | [[ui:exp.node.emulator]] et [[ui:exp.node.wait_http]] |
| 7 | [[ui:exp.node.impairment]], [[ui:exp.node.impairment_change]] et [[ui:exp.node.emulator_state]] |
| 8 | les nœuds WebSocket, l’authentification HTTP et la réserve de cookies |
| 9 | la charge sur une requête HTTP, et la dégradation sur TCP |

Un fichier d’avant la version 8 s’ouvre avec [[ui:exp.cookies]] désactivé, si bien qu’il s’exécute comme il
le faisait ; un fichier plus récent garde son propre réglage. Enregistré de nouveau, tout fichier devient la version
9.

## Depuis la ligne de commande ou un serveur {#automation}

Une exécution est la même partout : la ligne de commande et l’API du serveur démarrent la
même exécution que l’éditeur, avec les mêmes étapes, résultat et rapport.

```bash
signallab run checkout.json --profile Stage -p api=http://192.0.2.10:8080 --seed 42 --report report.json
```

- [`signallab run`](../automation/cli.md#cli-run) exécute des fichiers d’expérience ou des
  modèles dans ce processus ou sur un serveur, affiche les étapes comme la chronologie
  et sort avec le code du résultat.
- [`POST /api/run`](../api/run.md) en exécute une sur un serveur et répond avec le
  résultat, ou diffuse ses étapes au fur et à mesure. Un client qui s’en va n’arrête pas
  l’exécution ; elle va jusqu’à sa fin et garde son rapport.
- Dans CI : [GitHub Actions et autres](../automation/ci.md).

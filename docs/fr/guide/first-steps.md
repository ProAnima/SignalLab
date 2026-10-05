---
title: "Premiers pas"
description: "Une première séance sur un seul ordinateur : envoyer un message OSC et le voir arriver, l’enregistrer comme signal, interroger une API émulée, puis construire et exécuter une petite expérience."
---

# Premiers pas

Cette séance ne demande rien d’autre que Signal Lab : tout va vers `127.0.0.1`, cet
ordinateur, si bien qu’aucun appareil, aucun réseau ni aucune règle de pare-feu n’entre en jeu. Vous allez :

1. envoyer un message OSC et le voir arriver ;
2. voir le même message dans l’Inspecteur ;
3. l’enregistrer dans la bibliothèque et le renvoyer de n’importe où ;
4. démarrer une API HTTP émulée et l’interroger ;
5. exécuter une expérience contre cette API, lire pourquoi elle échoue, la corriger et ajouter une vérification.

Si vous n’avez pas encore installé Signal Lab, voir [Installation et mises à jour](install.md).
Vous ne savez pas où se trouve quelque chose dans la fenêtre ? Voir [La fenêtre](interface.md).

## Envoyer un message OSC et le voir arriver {#osc}

D’abord, de quoi recevoir le message : le moniteur de l’écran OSC.

1. Ouvrez [[ui:nav.osc]] dans la barre latérale.
2. Dans la partie [[ui:osc.monitor]], réglez le champ [[ui:common.bind]] sur `127.0.0.1:9000`, pour que le moniteur
   n’écoute que sur cet ordinateur.
3. Appuyez sur [[ui:osc.listen]]. Le bouton devient [[ui:common.stop]], la console indique que le
   moniteur écoute, et le moniteur apparaît comme tâche dans le bandeau du panneau inférieur.

Maintenant le message, depuis la partie d’envoi juste à côté :

4. Dans la partie [[ui:osc.sender]], laissez le champ [[ui:common.target]] à `127.0.0.1:9000`, le port sur lequel
   le moniteur écoute.
5. Laissez le champ [[ui:common.address]] à `/hello/avatar/1` et l’unique argument flottant sous
   [[ui:common.arguments]] à `1.0` — ou saisissez une adresse et des valeurs de votre choix.
6. Appuyez sur [[ui:common.send]], ou sur <kbd>Enter</kbd> dans le champ de la cible ou de l’adresse.

Une ligne apparaît dans le tableau du moniteur : [[ui:common.time]] d’arrivée,
[[ui:osc.from]] (`127.0.0.1` et le port d’envoi), [[ui:osc.address]] et
[[ui:osc.args]]. Sous la partie d’envoi, une ligne confirme ce qui a été envoyé et sa taille en
octets ; renvoyez, et elle compte les répétitions.

::: tip Un avertissement concernant le pare-feu ?
Sous Windows, démarrer le moniteur peut faire apparaître sous l’en-tête un avertissement concernant le Pare-feu
Windows. Il porte sur les messages venant d’*autres* machines ; le trafic sur `127.0.0.1` n’est jamais
filtré. Appuyez pour l’instant sur [[ui:fw.dismiss]] — [L’avertissement du pare-feu](interface.md#firewall-notice)
explique quand autoriser.
:::

## Le voir dans l’Inspecteur {#inspector}

L’Inspecteur enregistre chaque trame que chaque outil envoie et reçoit — mais seulement tant que
la capture est active.

1. Dans le panneau inférieur, ouvrez l’onglet [[ui:dock.inspector]].
2. Appuyez sur [[ui:ins.arm]]. Le point de l’onglet s’allume.
3. De retour dans la partie d’envoi, appuyez encore une fois sur [[ui:common.send]].

Deux lignes apparaissent, la plus récente en premier : le message tel qu’envoyé (→) et tel que le moniteur l’a reçu
(←), chacune avec son protocole, l’adresse de l’autre extrémité, sa taille et un résumé. Cliquez sur l’une d’elles :
le panneau [[ui:ins.detail]] montre quel outil l’a envoyée ou reçue et sur quelles adresses, le
message [[ui:ins.decoded]], et les [[ui:ins.rawBytes]] qui le composent.

Appuyez sur [[ui:ins.disarm]] quand vous avez terminé ; tant que la capture est désarmée, elle ne coûte rien. Plus de détails
dans [Inspecteur](../tools/inspector.md).

## L’enregistrer comme signal et le renvoyer {#signal}

Un message dont vous aurez de nouveau besoin a sa place dans la bibliothèque de signaux.

1. Sur l’écran OSC, appuyez sur [[ui:sig.saveNew]] sous la partie d’envoi.
2. Dans la fenêtre [[ui:sig.saveTitle]], réglez le champ [[ui:sig.name]] sur `First message` et le champ
   [[ui:sig.group]] sur `Tutorial` — un nouveau dossier est créé quand vous y enregistrez.
3. Appuyez sur [[ui:sig.saveConfirm]].

La partie d’envoi est maintenant liée à ce signal : le bouton indique [[ui:sig.savedState]], et une pastille
à côté montre où se trouve le signal. Modifiez l’argument et la pastille signale la
modification ; le bouton [[ui:sig.save]] (<kbd>Ctrl</kbd>+<kbd>S</kbd>) mettrait à jour le signal.

Renvoyez-le maintenant, de trois façons :

- **Depuis la bibliothèque.** Cliquez sur la pastille : l’écran [[ui:nav.signals]] s’ouvre avec le signal
  sélectionné dans le dossier `Tutorial` (ou ouvrez [[ui:nav.signals]] et cliquez sur le signal).
  Appuyez sur [[ui:sig.fire]], ou sur <kbd>Ctrl</kbd>+<kbd>Enter</kbd> ; un double-clic dessus dans la
  liste l’envoie aussi.
- **De n’importe où.** Sur n’importe quel écran, appuyez sur <kbd>Ctrl</kbd>+<kbd>K</kbd>, tapez `first`
  et appuyez sur <kbd>Enter</kbd>.
- **Depuis une expérience.** Quand vous ajoutez un nœud, le menu liste vos signaux sous
  [[ui:exp.group.signals]], prêts à devenir une étape qui en envoie un.

À chaque fois, le moniteur montre le message qui arrive et la console nomme le signal. Un
signal envoie exactement ce que son écran aurait envoyé. Plus de détails dans [Signaux](../tools/signals.md).

Quand vous en avez fini avec OSC, appuyez sur [[ui:common.stop]] sur le moniteur.

## Interroger une API émulée {#emulator}

Signal Lab est livré avec cinq émulateurs, tous sur `127.0.0.1`. L’un d’eux,
[[ui:seed.emu.demo-api.name]], est une API HTTP sur `127.0.0.1:8080` avec ces routes :

| Requête | Réponse |
| --- | --- |
| `GET /health` | `200` avec `{"status":"ok","time":"…"}` — l’heure actuelle |
| `GET /users/:id` | `200` avec l’utilisateur de cet identifiant, par exemple `{"id":"42","name":"User 42"}` |
| `POST /users` | `201` avec un en-tête `Location` et le nouvel identifiant |
| `GET /slow` | `200` au bout de 1,5 seconde |
| toute méthode, `/flaky` | `503`, `503`, puis `200` à partir de la troisième requête |
| tout le reste | `404` |

1. Ouvrez [[ui:nav.emulators]]. La partie [[ui:emu.library]] liste les cinq ; sélectionnez
   [[ui:seed.emu.demo-api.name]].
2. Appuyez sur [[ui:emu.start]]. L’émulateur répond maintenant sur `127.0.0.1:8080` et tourne comme tâche.
3. Ouvrez [[ui:nav.http]]. La méthode est `GET` ; réglez l’URL sur
   `http://127.0.0.1:8080/health`.
4. Appuyez sur [[ui:common.send]], ou sur <kbd>Enter</kbd> dans l’URL.

Sous [[ui:http.response]], vous voyez [[ui:http.status]] `200`,
[[ui:http.latency]], [[ui:http.size]], les en-têtes de la réponse et le corps JSON. Envoyez
`http://127.0.0.1:8080/flaky` trois fois : deux réponses `503`, puis `200` — ce à quoi ressemble un
service qui se rétablit pour un client qui réessaie.

De retour sur [[ui:nav.emulators]], le panneau [[ui:emu.live]] compte chaque requête, et la liste
[[ui:emu.received]] affiche chacune avec la [[ui:emu.col.rule]] qui y a répondu et la
[[ui:emu.col.reply]]. Laissez l’émulateur [[ui:seed.emu.demo-api.name]] tourner pour la suite.
Plus de détails dans [Émulateurs](../tools/emulators.md).

## Exécuter une expérience {#experiment}

Une expérience est un enchaînement d’étapes que vous pouvez exécuter encore et encore. Celle que Signal Lab ouvre
la première fois — le modèle [[ui:exp.templateHttp]] — envoie une requête à
`http://127.0.0.1:8080/` et vérifie que la réponse est `200`.

### Ouvrir le modèle {#open-template}

1. Ouvrez [[ui:nav.experiment]].
2. Si le canevas n’affiche pas quatre nœuds — [[ui:exp.node.start]],
   [[ui:exp.node.http]], [[ui:exp.node.assert_status]], [[ui:exp.node.end]] —, appuyez sur
   **☰** à gauche de la barre d’outils ([[ui:exp.documents]]), choisissez
   [[ui:exp.templateHttp]] dans la liste des modèles et appuyez sur
   [[ui:exp.openDocument]]. L’ouverture remplace l’expérience du canevas ;
   <kbd>Ctrl</kbd>+<kbd>Z</kbd> ramène la précédente.

Cliquez sur un nœud pour voir ses réglages dans le volet [[ui:exp.properties]], à droite. Les expériences s’enregistrent
d’elles-mêmes pendant que vous les modifiez.

### L’exécuter et lire pourquoi elle échoue {#first-run}

3. Appuyez sur [[ui:exp.run]].

La [[ui:exp.timeline]] s’ouvre sous le canevas, une ligne par étape quand elle démarre
([[ui:exp.running]]) et de nouveau quand elle se termine : l’heure, le nœud et le résultat. Cette exécution
échoue :

- L’étape [[ui:exp.node.start]] réussit et indique la graine de l’exécution.
- L’étape [[ui:exp.node.http]] réussit : la requête est partie et une réponse est revenue,
  `HTTP 404`.
- L’étape [[ui:exp.node.assert_status]] échoue : elle attendait `200` et a reçu `404`.

L’émulateur [[ui:seed.emu.demo-api.name]] n’a pas de route pour `/` : il a donc répondu `404` — et la
vérification l’a détecté. La ligne en haut de la chronologie indique [[ui:exp.failed]] et pourquoi.
Cliquez sur une ligne pour sélectionner son nœud sur le canevas.

::: tip La requête elle-même a échoué ?
Si l’étape [[ui:exp.node.http]] échoue sur une connexion refusée, rien n’écoute sur
`127.0.0.1:8080` : démarrez [[ui:seed.emu.demo-api.name]] sur l’écran [[ui:nav.emulators]] et relancez
l’exécution.
:::

### Corriger la requête {#fix}

4. Cliquez sur le nœud [[ui:exp.node.http]].
5. Dans le volet [[ui:exp.properties]], réglez le champ [[ui:sig.url]] sur `http://127.0.0.1:8080/health`.
6. Appuyez sur [[ui:exp.run]].

Cette fois, chaque étape réussit : [[ui:exp.node.assert_status]] indique
[[ui:exp.step.checked]], [[ui:exp.node.end]] indique [[ui:exp.step.complete]], et le
titre de la chronologie indique [[ui:exp.passed]].

### Ajouter une vérification {#add-check}

Un statut `200` dit que le service a répondu ; il ne dit pas ce qu’il a répondu. Vérifiez aussi
le corps :

7. Cliquez sur le nœud [[ui:exp.node.assert_status]].
8. Dans le volet [[ui:exp.properties]], appuyez sur [[ui:exp.addNext]] — ou appuyez sur <kbd>A</kbd> quand le
   canevas a le focus. Un menu de nœuds s’ouvre avec un champ de recherche.
9. Tapez `assert_body` et appuyez sur <kbd>Enter</kbd>. Un nœud [[ui:exp.node.assert_body]] est
   ajouté entre [[ui:exp.node.assert_status]] et [[ui:exp.node.end]], déjà relié,
   avec son champ [[ui:exp.contains]] prêt pour la saisie.
10. Tapez `"status":"ok"`.
11. Appuyez sur [[ui:exp.run]].

La nouvelle étape réussit. Remplacez le texte par quelque chose que le corps ne contient pas et relancez
pour la voir échouer en indiquant pourquoi.

### Ce que laisse une exécution {#report}

- **Un rapport.** Quand une exécution se termine, [[ui:exp.reportSaved]] apparaît dans le titre de la chronologie ;
  survolez-le pour voir le fichier. Une exécution qui se termine, réussie ou échouée, en écrit un dans le
  dossier `runs` de votre dossier de données, avec les valeurs qu’elle a utilisées et chaque étape. Dans un
  navigateur, c’est un lien de téléchargement.
- **Une graine.** Le titre affiche aussi la graine de l’exécution avec le bouton [[ui:exp.pinSeed]] : les valeurs aléatoires
  d’une exécution suivent sa graine, et l’épingler les reproduit exactement.

Plus de détails dans [Exécutions et rapports](../experiments/runs.md).

## Ranger {#clean-up}

Appuyez sur [[ui:app.stopAll]] dans l’en-tête : cela arrête l’émulateur [[ui:seed.emu.demo-api.name]] et
tout ce qui tourne encore. Votre signal, l’expérience et ses rapports restent dans votre
dossier de données.

## Et ensuite {#next}

- [Concepts](concepts.md) : les idées derrière les écrans, les signaux, les tâches, les émulateurs et
  les expériences.
- [Expériences](../experiments/index.md) : l’éditeur en détail, et chaque type de nœud dans
  [Nœuds](../experiments/nodes.md).
- [OSC](../protocols/osc.md), [HTTP](../protocols/http.md) et les pages des autres protocoles,
  quand vous dirigerez Signal Lab vers de vrais équipements.
- [La ligne de commande](../automation/cli.md) : exécuter la même expérience depuis un terminal ou
  un pipeline.

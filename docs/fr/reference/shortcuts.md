---
title: "Raccourcis clavier"
description: "Chaque raccourci clavier de Signal Lab — partout dans l’application, dans l’éditeur d’expériences, la bibliothèque de signaux, les écrans d’outils, les volets et le menu des langues."
---

# Raccourcis clavier

Chaque touche à laquelle Signal Lab répond, groupée par lieu d’utilisation. Les
raccourcis avec une seule lettre ou Delete n’agissent jamais pendant que vous
tapez dans un champ.

::: tip
Dans un navigateur sur un Mac, <kbd>⌘</kbd> fonctionne partout où <kbd>Ctrl</kbd>
est écrit ci-dessous, sauf <kbd>Ctrl</kbd>+<kbd>Space</kbd> dans un champ de modèle.
:::

## Partout {#anywhere}

| Touches | Ce qu’elles font |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>K</kbd> | Ouvre [[ui:sig.paletteTitle]], la palette de signaux, sur n’importe quel écran — même pendant la saisie dans un champ. Un nouvel appui la ferme |
| <kbd>F1</kbd> | Ouvre cette documentation à la page de l’écran où vous êtes, comme [[ui:app.docs]] |
| <kbd>Tab</kbd>, <kbd>Shift</kbd>+<kbd>Tab</kbd> | Se déplace entre les contrôles. L’info-bulle d’un contrôle apparaît lorsqu’il prend le focus ; toute autre touche la masque |
| <kbd>Space</kbd>, <kbd>Enter</kbd> | Active le bouton qui a le focus — toute chose cliquable est un bouton |
| <kbd>Esc</kbd> | Ferme la fenêtre de dialogue ouverte |

Dans la palette de signaux :

| Touches | Ce qu’elles font |
| --- | --- |
| La saisie | Filtre les signaux |
| <kbd>↑</kbd> <kbd>↓</kbd> | Choisit un signal |
| <kbd>Enter</kbd> | L’envoie et ferme la palette |
| <kbd>Esc</kbd> | Ferme la palette ; un clic à côté aussi |

## Éditeur d’expériences {#editor}

Sur le canevas — lorsqu’aucun champ de texte n’a le focus :

| Touches | Ce qu’elles font |
| --- | --- |
| <kbd>A</kbd> | Ouvre le menu pour ajouter un nœud ([[ui:exp.addNode]]). Avec un nœud sélectionné, le nouveau va après lui |
| <kbd>Delete</kbd> ou <kbd>Backspace</kbd> | Supprime la connexion sélectionnée, ou les nœuds sélectionnés — jamais [[ui:exp.node.start]] ni [[ui:exp.node.end]] |
| <kbd>Tab</kbd> | Parcourt les nœuds, leurs ports et les connexions ; un nœud ou une connexion qui prend le focus est sélectionné |
| <kbd>Enter</kbd> ou <kbd>Space</kbd> sur une sortie, puis sur un nœud ou son entrée | Les connecte |
| <kbd>←</kbd> <kbd>→</kbd> <kbd>↑</kbd> <kbd>↓</kbd> | Déplace les nœuds sélectionnés de 5 points (20 avec <kbd>Shift</kbd>), tant qu’un nœud a le focus. Un maintien compte pour une seule étape de l’historique |
| <kbd>Esc</kbd> | Ferme le menu d’ajout ; sinon annule une connexion en cours de tracé ; sinon abandonne la connexion sélectionnée ; sinon efface une sélection de plusieurs nœuds ; sinon quitte [[ui:exp.fullscreen]] ; sinon quitte [[ui:exp.focus]] |
| <kbd>Ctrl</kbd>+<kbd>Z</kbd> | [[ui:exp.undo]] |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd>, <kbd>Ctrl</kbd>+<kbd>Y</kbd> | [[ui:exp.redo]] |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | [[ui:exp.duplicate]] : une copie des nœuds sélectionnés, avec les connexions entre eux |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | Trouve un nœud par son type, ce qu’il fait ou son id |
| <kbd>Ctrl</kbd>+<kbd>0</kbd> | [[ui:exp.fit]] : tout le graphe dans la vue |
| <kbd>Ctrl</kbd>+<kbd>1</kbd> | [[ui:exp.resetZoom]] : 100 % |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:exp.sendNow]] — pour une attente, [[ui:exp.listenNow]] : le nœud sélectionné seulement, sans exécuter l’expérience |
| <kbd>Ctrl</kbd>+<kbd>A</kbd> | Sélectionne tous les nœuds |
| <kbd>Ctrl</kbd>+<kbd>C</kbd> | [[ui:exp.copy]] : les nœuds sélectionnés et les connexions entre eux ; [[ui:exp.node.start]] et [[ui:exp.node.end]] restent exclus, comme pour la duplication |
| <kbd>Ctrl</kbd>+<kbd>X</kbd> | Les coupe |
| <kbd>Ctrl</kbd>+<kbd>V</kbd> | Colle les nœuds copiés ici ou dans une autre expérience, avec de nouveaux id |

<kbd>Ctrl</kbd>+<kbd>A</kbd>, <kbd>C</kbd>, <kbd>X</kbd> et <kbd>V</kbd>
agissent sur les nœuds tant que l’éditeur a le focus, ou que rien ne l’a et qu’aucun
texte n’est sélectionné sur la page. Avec du texte sélectionné ailleurs — dans
[[ui:console.title]], dans un rapport — ce sont ceux de la page :
<kbd>Ctrl</kbd>+<kbd>C</kbd> copie ce texte.

Avec la souris :

| Geste | Ce qu’il fait |
| --- | --- |
| Glisser sur le canevas vide | Déplace la vue |
| <kbd>Shift</kbd> + glisser sur le canevas vide | Ajoute à la sélection les nœuds qu’un cadre touche |
| <kbd>Shift</kbd> ou <kbd>Ctrl</kbd> + clic sur un nœud | L’ajoute à la sélection, ou l’en retire |
| Glisser l’un de plusieurs nœuds sélectionnés | Les déplace tous |
| Double-clic sur le canevas vide | Ouvre le menu d’ajout à cet endroit |
| <kbd>Ctrl</kbd> + molette | Zoome sous le curseur |

Dans le menu d’ajout, le chercheur de nœuds et les propriétés :

| Où | Touches | Ce qu’elles font |
| --- | --- | --- |
| Menu d’ajout | La saisie | Filtre les nœuds et les signaux enregistrés |
| Menu d’ajout | <kbd>↑</kbd> <kbd>↓</kbd>, <kbd>Enter</kbd>, <kbd>Esc</kbd> | Choisir, ajouter, fermer |
| Chercheur de nœuds | <kbd>↑</kbd> <kbd>↓</kbd> | Choisit un nœud |
| Chercheur de nœuds | <kbd>Enter</kbd> | L’affiche sur le canevas et le sélectionne |
| Chercheur de nœuds | <kbd>Tab</kbd> | Retour au champ de recherche |
| Chercheur de nœuds | <kbd>Esc</kbd> | Le ferme |
| [[ui:exp.properties]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:exp.sendNow]] pour le nœud affiché |
| [[ui:exp.properties]] | <kbd>Esc</kbd> dans un champ | Retour au canevas avec le nœud sélectionné, où <kbd>A</kbd> ajoute le suivant |

Dans un champ qui prend des modèles (`{{name}}`) :

| Touches | Ce qu’elles font |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>Space</kbd> | Tape `{{` à la position du curseur et liste ce qui peut y aller : paramètres, variables, secrets, générateurs |
| Saisir `{{` | Liste la même chose |
| <kbd>↑</kbd> <kbd>↓</kbd> | Choisit dans la liste |
| <kbd>Enter</kbd> ou <kbd>Tab</kbd> | Insère le choix et ferme le `}}` |
| <kbd>Esc</kbd> | Ferme la liste ; le champ garde le focus |

Dans les boîtes de dialogue de l’expérience :

| Où | Touches | Ce qu’elles font |
| --- | --- | --- |
| [[ui:exp.params]] | <kbd>Enter</kbd> dans la valeur du dernier paramètre | Ajoute un autre paramètre |
| [[ui:exp.params]], [[ui:exp.runWith]] | <kbd>Esc</kbd> | Ferme la boîte de dialogue |
| [[ui:exp.runWith]] | <kbd>Enter</kbd> dans un champ | [[ui:exp.run]] avec ces valeurs |
| [[ui:exp.secrets]] | <kbd>Enter</kbd> dans une valeur | L’enregistre |
| [[ui:exp.secrets]] | <kbd>Esc</kbd> dans une valeur ou un nouveau nom | Annule ; [[ui:exp.params]] reste ouvert |

## Bibliothèque de signaux {#signals}

| Où | Touches | Ce qu’elles font |
| --- | --- | --- |
| [[ui:nav.signals]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:sig.fire]] : envoie le signal sélectionné |
| [[ui:nav.signals]] | Double-clic sur un signal | L’envoie |
| Un dossier | <kbd>F2</kbd> | [[ui:sig.renameFolder]] |
| Renommer un dossier | <kbd>Enter</kbd>, <kbd>Esc</kbd> | Garde le nouveau nom, ou annule |
| Un dossier | <kbd>Delete</kbd>, deux fois | [[ui:sig.removeFolder]] ; ce qu’il contient remonte d’un niveau |
| Un dossier | <kbd>→</kbd> <kbd>←</kbd> | L’ouvre, le ferme |
| Expéditeurs [[ui:nav.http]], [[ui:nav.osc]], [[ui:nav.mqtt]] | <kbd>Ctrl</kbd>+<kbd>S</kbd> | [[ui:sig.save]] lorsque le message est lié à un signal de la bibliothèque (ouvert depuis lui ou enregistré là) et a changé ; [[ui:sig.saveNew]] lorsqu’il n’est pas encore dans la bibliothèque |
| [[ui:sig.saveTitle]] | <kbd>Enter</kbd>, <kbd>Esc</kbd> | Enregistre, ou annule |

## Écrans d’outils {#tools}

| Écran | Touches | Ce qu’elles font |
| --- | --- | --- |
| [[ui:nav.http]] | <kbd>Enter</kbd> dans l’URL, un en-tête, les identifiants ou le délai d’attente | Envoie la requête |
| [[ui:nav.http]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> n’importe où dans la requête, le corps inclus | Envoie la requête |
| [[ui:nav.osc]] | <kbd>Enter</kbd> dans la cible, l’adresse ou un argument | Envoie le message |
| [[ui:nav.ws]] | <kbd>Enter</kbd> dans l’URL | Se connecte |
| [[ui:nav.ws]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> dans [[ui:ws.message]] | L’envoie |
| [[ui:nav.mqtt]] | <kbd>Enter</kbd> dans [[ui:mq.addSubscription]] | S’abonne |
| [[ui:nav.broadcast]] | <kbd>Enter</kbd> dans [[ui:bc.targetAddress]] ou [[ui:bc.targetCidr]] — tous les modes sauf [[ui:bc.mode.list]] | [[ui:bc.sendOnce]] |

Dans [[ui:feedback.title]] : <kbd>Ctrl</kbd>+<kbd>Enter</kbd> dans
[[ui:feedback.message]] l’envoie (tout comme <kbd>Enter</kbd> dans
[[ui:feedback.email]]), <kbd>Ctrl</kbd>+<kbd>V</kbd> n’importe où dans la boîte de
dialogue colle une capture d’écran du presse-papiers, et <kbd>Esc</kbd> la ferme.

## Volets {#panes}

Les poignées entre les volets — [[ui:layout.console]], [[ui:layout.properties]],
[[ui:layout.timeline]] — prennent le focus avec <kbd>Tab</kbd> :

| Touches | Ce qu’elles font |
| --- | --- |
| <kbd>↑</kbd> <kbd>↓</kbd> | La console et la chronologie : plus haut, plus bas, de 16 pixels |
| <kbd>←</kbd> <kbd>→</kbd> | Les propriétés : plus large, plus étroit, de 16 pixels (l’inverse quand l’interface se lit de droite à gauche) |
| <kbd>Shift</kbd> + une flèche | Quatre fois plus loin |
| <kbd>Home</kbd>, <kbd>End</kbd> | La plus petite taille, la plus grande |
| <kbd>Enter</kbd> | La taille par défaut ; un double-clic sur la poignée aussi |

## Menu des langues {#language}

Le drapeau et les lettres dans l’en-tête.

| Où | Touches | Ce qu’elles font |
| --- | --- | --- |
| Sur le bouton | <kbd>↓</kbd> ou <kbd>↑</kbd> | Ouvre la liste |
| Dans la liste | <kbd>↓</kbd> <kbd>↑</kbd> | La langue suivante, la précédente |
| Dans la liste | <kbd>Home</kbd>, <kbd>End</kbd> | La première, la dernière |
| Dans la liste | Une lettre | La langue suivante dont le nom — dans sa propre langue, la vôtre ou l’anglais — ou les lettres commencent par elle |
| Dans la liste | <kbd>Enter</kbd> ou <kbd>Space</kbd> | Bascule vers elle |
| Dans la liste | <kbd>Esc</kbd> ou <kbd>Tab</kbd> | Ferme la liste |

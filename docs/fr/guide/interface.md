---
title: "La fenêtre"
description: "Repérez-vous dans la fenêtre de Signal Lab : les écrans de la barre latérale, l’en-tête, la console et l’Inspecteur, la palette de signaux, les volets, les info-bulles et les langues."
---

# La fenêtre

La fenêtre de Signal Lab a quatre parties : la **barre latérale**, à gauche, liste les écrans ; l’**en-tête**,
en haut, regroupe ce qui s’applique partout ; l’**écran** que vous avez choisi occupe
le centre ; et le **panneau inférieur** affiche la console, les tâches en cours et
l’Inspecteur. L’application de bureau et la page d’un serveur dans un navigateur se ressemblent ; les rares
différences sont décrites dans [Dans un navigateur](#browser).

## La barre latérale {#sidebar}

Chaque écran est un outil à part entière. Cliquez sur l’un d’eux pour l’ouvrir :

| Écran | À quoi il sert |
| --- | --- |
| [[ui:nav.experiment]] | Construire des scénarios de test sur un canevas, les exécuter et lire chaque exécution étape par étape. [Expériences](../experiments/index.md) |
| [[ui:nav.signals]] | La bibliothèque de signaux : des messages nommés, rangés dans des dossiers, à modifier et à renvoyer. [Signaux](../tools/signals.md) |
| [[ui:nav.emulators]] | Des API HTTP fictives, des appareils OSC, UDP et TCP et des brokers MQTT qui répondent selon des règles. [Émulateurs](../tools/emulators.md) |
| [[ui:nav.osc]] | Envoyer des messages OSC, surveiller un port, envoyer une forme d’onde vers un point de terminaison. [OSC](../protocols/osc.md) |
| [[ui:nav.mqtt]] | Se connecter à un broker, voir chaque topic qu’il contient, publier et effacer des valeurs retenues. [MQTT](../protocols/mqtt.md) |
| [[ui:nav.broadcast]] | Envoyer à de nombreux hôtes à la fois — une liste, la diffusion, la multidiffusion, le balayage d’un sous-réseau — et écouter qui répond. [Diffusion et découverte](../protocols/broadcast.md) |
| [[ui:nav.http]] | Une requête et sa réponse complète, puis une rafale de charge contre le même point de terminaison. [HTTP](../protocols/http.md) |
| [[ui:nav.ws]] | Se connecter à un service WebSocket, envoyer du texte ou des octets, lire chaque message. [WebSocket](../protocols/websocket.md) |
| [[ui:nav.netsim]] | Un relais qui dégrade le trafic UDP ou TCP entre un client et sa cible. [Dégradation](../tools/impairment.md) |
| [[ui:nav.storm]] | Une charge UDP ou TCP brute contre vos propres serveurs et liaisons. [Tempête](../tools/storm.md) |
| [[ui:nav.scan]] | Quels ports TCP d’un hôte sont ouverts, avec ce que le service dit en premier. [Scanner](../tools/scanner.md) |

La barre latérale démarre en rail étroit avec le symbole et un nom court de chaque écran ; survolez-en un
pour voir son nom complet. Le **☰** à gauche de l’en-tête ([[ui:app.expandNav]] /
[[ui:app.collapseNav]]) bascule entre le rail et la liste complète, et Signal Lab
retient votre choix.

Un nombre sur l’entrée d’un écran compte les tâches lancées depuis celui-ci, comme un moniteur ou un
émulateur : vous voyez ce qui tourne encore sans l’ouvrir. En bas de la
barre latérale, la version ouvre À propos, et la ligne en dessous compte toutes les tâches en cours.

Signal Lab s’ouvre sur le dernier écran utilisé.

## L’en-tête {#header}

De gauche à droite :

| Élément | Rôle |
| --- | --- |
| **☰** | Affiche la barre latérale en rail ou en liste complète. |
| L’hôte | [[ui:app.host]], puis le nom et l’adresse réseau de l’ordinateur sur lequel tourne le moteur : celui-ci dans l’application de bureau, le serveur dans un navigateur. Dans un navigateur, il indique aussi [[ui:app.server]] — survolez-le pour voir où le serveur conserve ses fichiers — et le point à côté change quand la page perd sa connexion. |
| Le bouton de mise à jour | Apparaît dans l’application de bureau quand une version plus récente a été trouvée, et ouvre À propos pour l’installer. Voir [Mises à jour](install.md#updates). |
| Le livre | [[ui:app.docs]] : cette documentation, à la page de l’écran où vous êtes. <kbd>F1</kbd> fait de même depuis n’importe où. |
| **✉** | [[ui:feedback.open]] : un message aux développeurs. Voir [Écrire aux développeurs](#feedback). |
| **?** | [[ui:about.open]] : la version, qui fait Signal Lab, comment joindre ses auteurs, et les mises à jour. |
| Le drapeau | [[ui:app.language]] : la langue de l’interface. Voir [Langues](#languages). |
| [[ui:app.signOut]] | Dans un navigateur, quand le serveur demande un jeton d’accès : met fin à la session de ce navigateur. |
| [[ui:app.stopAll]] | Arrête d’un coup toutes les tâches en cours : moniteurs, générateurs, balises, émulateurs, relais, tempêtes, scans, exécutions. Il est grisé tant que rien ne tourne. |

### La documentation {#docs}

La documentation est intégrée à l’application et au serveur : elle est là sans
connexion Internet, dans la langue de l’interface. L’application de bureau l’affiche dans une fenêtre
à part — appuyer à nouveau sur le bouton depuis un autre écran amène cette fenêtre à la page
de cet écran — et envoie à votre navigateur les liens qui sortent de la documentation. Dans
un navigateur, elle s’ouvre dans un onglet à part. À propos a aussi un bouton [[ui:app.docs]], qui
ouvre [Qu’est-ce que Signal Lab](index.md).

### À propos {#about}

À propos affiche la version, le développeur, l’adresse de contact (avec un bouton qui la
copie), le code source et la licence. Dans l’application de bureau, sa section [[ui:update.title]]
recherche, affiche et installe les mises à jour — voir [Mises à jour](install.md#updates). Dans un navigateur,
elle indique [[ui:update.server]]. Le bouton [[ui:about.writeUs]] ouvre le formulaire de retour.

### Écrire aux développeurs {#feedback}

Le **✉** de l’en-tête ouvre un formulaire qui va directement aux développeurs :

| Champ | Quoi y mettre |
| --- | --- |
| [[ui:feedback.message]] | Ce qui s’est passé, et ce à quoi vous vous attendiez. Obligatoire ; 20 000 caractères au plus. |
| [[ui:feedback.email]] | Facultatif : où les développeurs peuvent vous répondre. Eux seuls le voient. |
| [[ui:feedback.screenshots]] | Jusqu’à 6 images (PNG, JPEG, WebP ou GIF), 8 Mo chacune et 15 Mo au total. Collez-en une avec <kbd>Ctrl</kbd>+<kbd>V</kbd>, déposez des fichiers sur la fenêtre ou appuyez sur [[ui:feedback.addScreenshot]]. |
| [[ui:feedback.logs]] | Les lignes de la console et [[ui:feedback.systemInfo]], chacun joint dans un fichier à part. Le bouton [[ui:feedback.show]] montre exactement ce qui est envoyé ; décochez l’un ou l’autre pour l’omettre. |

Les journaux joints omettent le nom de cet ordinateur, son adresse réseau et les noms figurant dans
les chemins de vos dossiers. <kbd>Ctrl</kbd>+<kbd>Enter</kbd> envoie le formulaire ; une fois parti,
vous recevez un numéro de référence, que la console conserve aussi. Le message transite par
le service du studio, qui le transmet par e-mail ; l’application ne détient aucun mot de passe pour cela.

## Le panneau inférieur {#bottom-panel}

Le panneau sous chaque écran a deux onglets, [[ui:console.title]] et
[[ui:dock.inspector]], et, entre eux et les boutons du panneau, le bandeau des
tâches en cours.

- Le **chevron** à sa gauche ([[ui:console.collapse]] / [[ui:console.expand]]) replie le
  panneau jusqu’à sa barre ou le rouvre. Une fois le panneau replié, sa barre affiche toujours la dernière
  ligne de la console ; cliquez sur cette ligne pour ouvrir le panneau.
- Faites glisser le bord supérieur du panneau pour l’agrandir ou le réduire (voir [Redimensionner les volets](#panes)).
- Le bouton à sa droite l’agrandit ([[ui:dock.maximise]]) et le ramène à sa taille
  ([[ui:dock.restore]]).

L’état ouvert ou replié du panneau, l’onglet qu’il affiche et sa hauteur sont conservés pour la fois
suivante.

### La console {#console}

La console indique ce que chaque outil a fait et ce qui s’est mal passé, la ligne la plus récente en dernier : un message envoyé
et sa taille, un moniteur démarré, le statut et la durée d’une réponse, une tâche terminée et
pourquoi. Chaque ligne porte l’heure (à la milliseconde), une étiquette qui nomme l’outil, et le message,
coloré selon sa nature — réussite, information, avertissement ou erreur.

- [[ui:console.autoscroll]] garde la ligne la plus récente visible à mesure que les lignes arrivent ; décochez-le pour
  relire en arrière pendant que d’autres arrivent.
- Le bouton [[ui:common.clear]] la vide.
- Elle conserve les 500 dernières lignes.
- Changer de langue réécrit toute la console dans la nouvelle.

### Les tâches {#jobs-strip}

Chaque tâche en cours — un moniteur, un générateur, une balise, une connexion à un broker, un émulateur,
un relais, une tempête, un scan, une exécution d’expérience — a une pastille dans le bandeau avec son numéro
et ce qu’elle est, et son propre bouton pour l’arrêter. Quand rien ne tourne, le bandeau indique
[[ui:console.empty]]. Voir [Tâches](concepts.md#jobs).

### L’onglet Inspecteur {#inspector-tab}

L’onglet [[ui:dock.inspector]] affiche chaque trame que les outils envoient et reçoivent pendant que
la capture est active, à côté de l’écran sur lequel vous travaillez. Son point s’allume tant que
la capture est active, et un nombre compte les trames capturées. Le panneau s’ouvre assez haut
pour la liste de l’Inspecteur et le détail d’une trame ; un lien vers une trame ailleurs — dans la
chronologie d’une exécution ou la liste d’un émulateur — ouvre cet onglet sur cette trame. Une fois ouvert,
l’Inspecteur conserve sa liste et sa sélection pendant que le panneau est fermé. Pour s’en servir :
[Inspecteur](../tools/inspector.md).

## Envoyer un signal de n’importe où {#palette}

Appuyez sur <kbd>Ctrl</kbd>+<kbd>K</kbd> sur n’importe quel écran pour ouvrir la palette [[ui:sig.paletteTitle]] : tapez
quelques lettres du nom, du dossier ou de la cible d’un signal, choisissez avec <kbd>↑</kbd> et
<kbd>↓</kbd>, et appuyez sur <kbd>Enter</kbd> pour l’envoyer. La palette liste jusqu’à 12 signaux à la
fois. <kbd>Esc</kbd>, un clic en dehors ou <kbd>Ctrl</kbd>+<kbd>K</kbd> à nouveau la
ferme. La console indique ce qui a été envoyé et où. Voir [Signaux](../tools/signals.md).

## Redimensionner les volets {#panes}

Une fine poignée sépare les volets redimensionnables : le bord supérieur du panneau inférieur
([[ui:layout.console]]) et, sur l’écran des expériences, le bord des propriétés
([[ui:layout.properties]]) et le haut de la chronologie de l’exécution ([[ui:layout.timeline]]).

- Faites glisser la poignée.
- Ou donnez-lui le focus avec <kbd>Tab</kbd> et utilisez les flèches : chaque appui la déplace de 16 pixels,
  quatre fois plus avec <kbd>Shift</kbd> ; <kbd>Home</kbd> et <kbd>End</kbd> mènent à la
  plus petite et à la plus grande taille.
- Double-cliquez dessus, ou appuyez sur <kbd>Enter</kbd> quand elle a le focus, pour rendre au volet sa taille
  par défaut.

Les tailles sont conservées pour la fois suivante.

## Info-bulles {#tooltips}

Les écrans affichent des libellés, des valeurs et des états, et gardent leurs explications dans des info-bulles : ce
qu’attend un champ, ce que `0` y signifie, quelle touche fait la même chose. Une info-bulle apparaît quand
vous laissez le pointeur sur un élément environ une demi-seconde, et immédiatement quand vous l’atteignez
au clavier ; un champ affiche l’info-bulle de son libellé. <kbd>Esc</kbd>, la saisie,
un clic ou le défilement la masquent. Les lecteurs d’écran lisent le même texte.

Les messages d’erreur disent où et ce qui s’est mal passé, et pourquoi ; le texte du système lui-même est
replié sous [[ui:err.details]].

## Les écrans gardent leur état {#state}

Un écran s’ouvre la première fois que vous le visitez, puis reste tel quel pendant que vous travaillez sur
un autre : ce que vous avez saisi, la dernière réponse, la liste des messages d’un moniteur et la tâche
qui la remplit, jusqu’à la position de défilement — tout est là quand vous revenez. Une tâche en cours
continue quel que soit l’écran que vous regardez.

Certaines valeurs sont aussi conservées d’un redémarrage à l’autre : le message OSC que vous envoyiez, la requête
HTTP, l’adresse WebSocket, la taille des volets, l’écran où vous étiez.

## L’avertissement du pare-feu {#firewall-notice}

Le Pare-feu Windows Defender décide, programme par programme, si d’autres machines peuvent
l’atteindre. La première fois qu’un programme écoute, Windows interroge la personne devant l’écran — et un
*Annuler* à ce moment-là, ou un réseau que Windows considère comme public, rejette sans bruit tout ce que les autres
machines envoient. Un moniteur qui n’affiche rien en est le signe habituel.

C’est pourquoi, dans l’application de bureau sous Windows, la première fois que quelque chose se met à écouter — un
moniteur OSC, l’écoute de découverte, un relais de dégradation, un émulateur ou une exécution d’expérience —,
Signal Lab consulte une fois le pare-feu. Quand le pare-feu fait obstacle, un avertissement sous
l’en-tête le signale :

- Le bouton [[ui:fw.allow]] demande les droits d’administrateur avec l’invite de Windows, puis permet
  aux autres machines d’atteindre Signal Lab sur les réseaux privés et de domaine.
- Sur un réseau que Windows qualifie de public — souvent le Wi-Fi d’une salle —, le bouton devient
  [[ui:fw.allowPublic]].
- Le bouton [[ui:fw.dismiss]] masque l’avertissement pour cette session.

Autoriser remplace les règles de pare-feu entrantes de Signal Lab par une seule règle d’autorisation. Le trafic
sur cet ordinateur lui-même (`127.0.0.1`) n’est jamais concerné : vous pouvez ignorer l’avertissement
tant que vous travaillez en boucle locale. Une installation pour tous a déjà la règle ; voir
[Ce que le programme d’installation ajoute](install.md#windows-setup-adds). `signallab doctor` indique la même chose
depuis un terminal et `signallab firewall allow` y règle le problème — voir
[La ligne de commande](../automation/cli.md). Il n’y a pas d’avertissement sous Linux ni dans un navigateur : le
pare-feu d’un serveur relève de son administrateur.

## Dans un navigateur {#browser}

La page d’un [serveur](../server/index.md) est la même interface, avec quelques différences :

- L’hôte dans l’en-tête nomme le serveur, avec [[ui:app.server]] à côté.
- Si le serveur demande un jeton d’accès, vous vous connectez une fois, et [[ui:app.signOut]] dans
  l’en-tête met fin à la session.
- Si la page perd sa connexion au serveur, une barre indique
  [[ui:app.connectionLost]] jusqu’à son retour ; la console note les deux.
- Les rapports d’exécution, les exports et les captures de l’Inspecteur sont téléchargés par le navigateur au lieu
  d’être indiqués par un chemin.
- À propos n’a pas de mises à jour : le serveur se met à jour avec son image.

Tout ce que fait la page d’un serveur se passe sur le serveur : le trafic part de lui, les moniteurs
écoutent sur ses ports, les fichiers arrivent dans son dossier de données. Voir
[Concepts](concepts.md#desktop-and-server).

## Langues {#languages}

Le drapeau de l’en-tête montre la langue actuelle et ses deux lettres. Cliquez dessus pour
ouvrir la liste de toutes les langues, chacune avec son drapeau et son propre nom, et choisissez-en une. Dans
la liste, <kbd>↑</kbd>, <kbd>↓</kbd>, <kbd>Home</kbd> et <kbd>End</kbd> déplacent la sélection, une lettre
saute à la langue suivante qui commence par elle — dans son propre nom ou en anglais, si bien que
<kbd>g</kbd> trouve Deutsch —, <kbd>Enter</kbd> choisit et <kbd>Esc</kbd> ferme la liste.

L’interface change aussitôt, sans redémarrage : chaque écran, chaque info-bulle et chaque erreur, et
aussi les lignes précédentes de la console. Au premier démarrage, Signal Lab choisit la première des
langues de votre système qu’il connaît, ou l’anglais, puis conserve votre choix — dans
un navigateur, pour ce navigateur.

L’arabe fait passer toute la fenêtre de droite à gauche. Ce qui relève des données reste de gauche à droite, tel qu’il est
écrit : les adresses, les vidages hexadécimaux, le code et le canevas des expériences.

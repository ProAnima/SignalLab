---
title: "Concepts"
description: "Les idées derrière Signal Lab — écrans et expériences, signaux, tâches, capture, émulateurs, relais de dégradation, paramètres, modèles, secrets, graines, rapports et dossier de données."
---

# Concepts

Cette page explique les idées sur lesquelles Signal Lab est construit, pour que le reste de la
documentation se lise facilement. Chaque section renvoie à la page qui traite son sujet en
entier.

## Écrans et expériences {#screens-and-experiments}

Signal Lab a deux façons de travailler, et vous utiliserez les deux.

- **Les écrans** sont des outils pour le travail que vous faites maintenant, à la main : envoyer ce message, écouter sur ce
  port, démarrer cet émulateur, voir ce que le broker détient. Vous essayez, regardez, changez quelque chose
  et réessayez. Chaque protocole et chaque outil a un écran ; voir [La fenêtre](interface.md).
- **Les expériences** sont des enchaînements que vous construisez une fois et exécutez encore et encore, de la même façon à chaque fois : envoyer
  une requête, attendre la réponse, la vérifier, continuer ou bifurquer. Chaque exécution est rapportée étape
  par étape et enregistrée. Voir [Expériences](../experiments/index.md).

Les deux se rejoignent à plusieurs endroits. Sur les écrans HTTP et OSC, [[ui:common.toExperiment]]
transforme ce que vous venez d'envoyer en l'étape suivante de l'expérience. Sur le moniteur OSC et
l'écran MQTT, [[ui:osc.waitForThis]] transforme un message que vous avez reçu en une étape qui
l'attend. Et un signal de la bibliothèque peut devenir une étape — n'importe lequel sauf des octets UDP bruts
écrits en hexadécimal.

## Signaux et bibliothèque {#signals}

Un **signal** est un message que vous gardez : un nom, un dossier, une note sur ce qu'il doit faire
advenir, et ce qu'il envoie — un message OSC, des octets UDP bruts, une requête HTTP ou une publication MQTT.
La **bibliothèque de signaux** les conserve dans des dossiers que vous pouvez imbriquer, renommer et déplacer
autour.

- Vous enregistrez un signal depuis les écrans OSC, HTTP et MQTT ([[ui:sig.saveNew]]), vous en créez un sur
  l'écran [[ui:nav.signals]], ou vous enregistrez une trame que l'Inspecteur a capturée
  ([[ui:sig.fromFrame]]), qui la rejoue alors octet par octet.
- Vous l'envoyez depuis l'écran [[ui:nav.signals]], depuis n'importe où avec
  <kbd>Ctrl</kbd>+<kbd>K</kbd>, comme étape d'une expérience, ou avec `signallab fire` depuis un
  terminal.
- Un signal envoie exactement ce que son écran enverrait : les mêmes octets, par le même chemin,
  affichés dans l'Inspecteur sous son vrai protocole.

La bibliothèque est un seul fichier, `signals.json`, dans le [dossier de données](#data-folder) : du JSON simple
que vous pouvez lire, modifier, copier vers une autre machine ou garder dans un dépôt. Il démarre avec un
ensemble de signaux d'exemple, tous dirigés vers `127.0.0.1`. Voir [Signaux](../tools/signals.md).

## Tâches {#jobs}

Tout ce qui continue de tourner après que vous avez appuyé sur son bouton est une **tâche** : un moniteur ou
générateur OSC, une connexion à un broker ou à un WebSocket, une balise ou un écouteur de découverte, une rafale de charge
HTTP, un émulateur, un relais de dégradation, une tempête, un scan, une exécution d'expérience.

- Chaque tâche a une pastille dans le bandeau du panneau inférieur, avec son numéro, ce qu'elle est et un
  bouton pour l'arrêter. La barre latérale indique combien de tâches chaque écran a en cours.
- [[ui:app.stopAll]] dans l'en-tête arrête toutes les tâches d'un coup.
- Une tâche qui se termine d'elle-même — un scan terminé, une exécution réussie, un moniteur dont le
  port a échoué — laisse sa pastille, et la console dit comment elle s'est terminée.
- Une tâche continue pendant que vous travaillez sur d'autres écrans.
- Installer une mise à jour arrête d'abord toutes les tâches.

Sur un serveur, les tâches appartiennent au serveur : chaque page qui y est connectée voit les mêmes tâches
et peut les arrêter.

## Capture et Inspecteur {#capture}

Chaque outil — émetteurs, moniteurs, écouteurs, émulateurs, relais, exécutions d'expérience — remet
chaque trame qu'il envoie ou reçoit à une **capture**, et l'[Inspecteur](../tools/inspector.md)
l'affiche dans une seule chronologie.

- La capture est **désactivée jusqu'à ce que vous l'activiez** avec [[ui:ins.arm]], et ne coûte rien tant qu'elle est
  désactivée. Elle reste active, quel que soit l'écran où vous êtes, jusqu'à ce que vous la désactiviez.
- Elle conserve jusqu'à 8192 trames et 64 Mio de leurs octets ; les trames les plus anciennes font de la place aux
  nouvelles. Chaque trame garde jusqu'à 256 Kio de ses octets, et la liste affiche son premier
  Kio.
- [[ui:ins.pause]] empêche la liste de bouger pour que vous puissiez la lire ; la capture continue
  en dessous.
- Les valeurs secrètes utilisées par une exécution sont masquées dans chaque trame.
- Toute la capture peut être exportée, chaque octet conservé, vers un fichier `.jsonl` ou `.txt`.

## Émulateurs {#emulators}

Un **émulateur** joue l'autre côté : l'API, l'appareil ou le service auquel votre système s'adresse.
Chacun est un document avec un protocole, l'adresse sur laquelle il écoute et des règles disant quoi
répondre :

| Protocole | Ce qu'il émule |
| --- | --- |
| HTTP | Une API : des routes par méthode et chemin, des réponses en séquence, à tour de rôle ou au hasard, avec des délais et des défaillances |
| OSC | Un appareil qui répond aux messages OSC par adresse et arguments |
| UDP | Un appareil qui répond aux datagrammes par leur charge utile |
| TCP | Un appareil qui répond par lignes sur une connexion TCP, avec un accueil |
| MQTT | Un broker qui achemine ce que les clients publient, et répond selon des règles comme un appareil |

Un émulateur peut répondre lentement, échouer, fermer la connexion, envoyer un corps mal formé, ou tomber
en panne selon un calendrier. Chaque échange est compté, listé sur son écran et capturé pour
l'Inspecteur.

Vous en démarrez un depuis l'écran [[ui:nav.emulators]], où il tourne comme une tâche ; depuis un
nœud [[ui:exp.node.emulator]] d'une expérience, où il répond pendant toute l'exécution ; ou avec
`signallab emulate`. La **bibliothèque d'émulateurs** est `emulators.json` dans le dossier de données. Elle
démarre avec un émulateur de chaque sorte, tous sur `127.0.0.1` :

| Émulateur | Écoute sur | Ce qu'il fait |
| --- | --- | --- |
| [[ui:seed.emu.demo-api.name]] | `127.0.0.1:8080` (HTTP) | Un contrôle de santé, un utilisateur par identifiant, une création, une réponse lente, et une route qui échoue deux fois avant de fonctionner |
| [[ui:seed.emu.osc-device.name]] | `127.0.0.1:9100` (OSC) | Répond à `/ping` par `/pong` et un compteur, acquitte `/fader/…` par `/ack`, prend `/cue/…` sans un mot |
| [[ui:seed.emu.udp-device.name]] | `127.0.0.1:7100` (UDP) | Répond à `PING` par `PONG` et un compteur, et à tout le reste par le nombre d'octets reçus |
| [[ui:seed.emu.tcp-device.name]] | `127.0.0.1:7200` (TCP) | Un protocole à lignes comme celui d'un projecteur : accueille par `READY`, rapporte et commute l'alimentation, dit `BYE` et raccroche sur `QUIT` |
| [[ui:seed.emu.mqtt-broker.name]] | `127.0.0.1:1883` (MQTT) | Un `lab/status` retenu, et une lampe : `ON` ou `OFF` publié sur `lab/<name>/set` reçoit sa réponse sur `lab/<name>/state` |

Voir [Émulateurs](../tools/emulators.md).

## Relais de dégradation {#impairment}

Un **relais de dégradation** se place entre un client et sa cible. Vous dirigez le client vers
l'adresse d'écoute du relais au lieu de la vraie cible ; le relais transmet dans les deux sens
et dégrade ce qui passe, selon un **profil** :

- en **UDP**, chaque datagramme connaît son propre sort : latence et gigue, perte et pertes en rafale de
  perte, duplication, corruption, réordonnancement, une limite de bande passante, ou rien du tout
  (hors ligne) ;
- en **TCP**, chaque connexion est jointe à une connexion à elle vers la cible, et les deux
  flux sont retardés, bridés à une limite de bande passante, réinitialisés, ou laissés semi-ouverts.

Les préréglages définissent un profil en un clic, d'un câble à une liaison satellite. Un changement s'applique
pendant que le relais tourne, sans abandonner son port. Chaque décision est tirée d'une graine, donc
le même trafic connaît le même sort de nouveau.

Sur l'écran [[ui:nav.netsim]], un relais tourne comme une tâche. Dans une expérience, un
nœud [[ui:exp.node.impairment]] en ouvre un pour l'exécution et [[ui:exp.node.impairment_change]]
bascule son profil en cours d'exécution. Voir [Dégradation](../tools/impairment.md) et
[Défaillances](../experiments/faults.md).

## Expériences {#experiments}

### Nœuds et fils {#nodes-and-wires}

Une expérience est un graphe de **nœuds** reliés par des **fils**. Chaque nœud est une étape : il
envoie quelque chose, attend quelque chose, vérifie une valeur, en extrait une, change le flux,
ou prépare l'exécution — un émulateur, un relais de dégradation. Chaque expérience a exactement un
[[ui:exp.node.start]] et un [[ui:exp.node.end]], et contient jusqu'à 64 nœuds. L'
expérience ouverte dans l'éditeur est enregistrée au fur et à mesure de vos modifications. Voir [Nœuds](../experiments/nodes.md).

### Sorties {#outputs}

Un fil va de la **sortie** d'un nœud à l'entrée d'un autre nœud. La plupart des nœuds ont une
sortie ; d'autres choisissent entre plusieurs : [[ui:exp.yes]] et [[ui:exp.no]] pour une bifurcation,
[[ui:exp.portMatched]] et [[ui:exp.portTimeout]] pour une attente, [[ui:exp.portBody]],
[[ui:exp.portDone]] et [[ui:exp.portLimit]] pour une boucle, [[ui:exp.branch1]] et
[[ui:exp.branch2]] pour une branche parallèle.

Une sortie peut avoir plusieurs fils : chacun s'exécute comme une branche à part, en parallèle, et une
[[ui:exp.node.join]] attend tous les fils qui y mènent. Seul le corps d'une
[[ui:exp.node.loop]] peut y ramener ; tout autre cycle est une erreur. Voir
[Flux](../experiments/flow.md).

### Paramètres et profils {#parameters}

Un **paramètre** est une valeur nommée — un hôte, un port, un nom d'utilisateur — écrite une fois sous
[[ui:exp.params]] et utilisée dans n'importe quel champ comme `{{name}}`. Un **profil** change certains
paramètres à la fois : un pour l'ordinateur portable, un pour la scène, un pour la salle. Vous choisissez
le profil que les exécutions utilisent, ou [[ui:exp.runWith]] un profil, d'autres valeurs ou une graine pour une
seule exécution, sans changer l'expérience. Une expérience contient jusqu'à 64 paramètres et
32 profils. Voir [Données](../experiments/data.md).

### Modèles {#templates}

La plupart des champs texte des nœuds sont des **modèles** : du texte simple avec des expressions entre doubles
accolades, remplies au fur et à mesure que l'étape s'exécute.

- `{{host}}` — un paramètre, ou une variable définie plus tôt dans l'exécution, comme une valeur qu'un
  nœud [[ui:exp.node.extract]] a prise d'une réponse, ou une réponse qu'une attente a reçue
  (`{{reply.args[0]}}`).
- `{{secret.API_TOKEN}}` — un secret.
- `{{run.id}}`, `{{run.seed}}`, `{{now}}`, `{{now.iso}}`, `{{counter}}` — l'exécution et
  le moment.
- `{{uuid}}`, `{{random_int(1, 10)}}`, `{{random_float(0, 1, 2)}}`, `{{pick("a", "b")}}`
  — des valeurs générées.

Seul le moteur remplit les modèles, donc un champ signifie la même chose dans une exécution, dans
l'aperçu de l'éditeur et dans [[ui:exp.sendNow]]. Un nom inconnu est une erreur, jamais une chaîne
vide. Voir [Données](../experiments/data.md).

### Secrets {#secrets}

Un **secret** est une valeur qu'une expérience utilise mais ne stocke jamais — un jeton, un mot de passe. L'
expérience ne détient que son nom ; les champs l'utilisent comme `{{secret.NAME}}` ; et chaque texte qu'une exécution
rapporte, chaque étape, le rapport et chaque trame de l'Inspecteur, l'affichent masqué. Aucune commande
ne rend jamais la valeur d'un secret.

L'endroit où vivent les valeurs dépend de l'endroit où Signal Lab tourne :

- **L'application de bureau sous Windows** les conserve dans le Gestionnaire d'identification Windows. Vous les définissez
  sous [[ui:exp.params]] → [[ui:exp.secrets]].
- **L'application de bureau sous Linux** n'a pas de magasin d'identifiants pour les conserver, donc les expériences
  qui utilisent des secrets s'exécutent depuis la ligne de commande ou un serveur là-bas.
- **Un serveur** les lit, en lecture seule, depuis son environnement (`SIGNALLAB_SECRET_<NAME>`) ou
  depuis un fichier par nom dans son dossier de secrets (`/run/secrets/signallab/<NAME>` par
  défaut) ; ils ne peuvent pas être définis depuis un navigateur.
- **La ligne de commande** les lit de la même façon qu'un serveur, ou depuis le magasin d'identifiants du système
  quand on le lui demande. Voir [La ligne de commande](../automation/cli.md).

### Graines {#seeds}

Chaque exécution a une **graine**, un nombre qui décide tout ce qui est aléatoire en elle : les valeurs
générées, la gigue d'une répétition, le choix aléatoire de réponse d'un émulateur, chaque décision
d'un relais de dégradation. La même graine et le même trafic donnent la même exécution. Une nouvelle
graine est tirée pour chaque exécution, à moins que l'expérience n'en épingle une — [[ui:exp.pinSeed]] dans la
chronologie d'exécution épingle la graine de la dernière exécution, et [[ui:exp.seed]] sous
[[ui:exp.params]] en définit une.

### Exécutions et rapports {#reports}

Une **exécution** démarre à [[ui:exp.node.start]], suit les fils et réussit quand elle atteint
[[ui:exp.node.end]] sans qu'aucune étape n'ait échoué. Elle est arrêtée si elle dure plus de 300
secondes. Chaque étape apparaît dans la chronologie d'exécution à son début et à sa fin.

Une exécution qui se termine, réussie ou échouée, écrit un **rapport** dans le dossier `runs` du dossier de
données : le nom de l'expérience, la graine, le profil et les valeurs utilisés, quand elle a commencé et fini, le résultat
et son erreur, chaque étape, et ce que ses émulateurs et relais ont compté. Deux exécutions de la
même expérience peuvent être comparées. Voir [Exécutions et rapports](../experiments/runs.md).

## Le dossier de données {#data-folder}

Tout ce que Signal Lab conserve est un fichier dans un seul dossier : `Documents/SignalLab` dans votre dossier
personnel, sous Windows comme sous Linux. Un serveur garde le sien, que vous choisissez à son
démarrage (`/data` dans l'image Docker).

| Fichier ou dossier | Ce qu'il contient |
| --- | --- |
| `experiment.json` | L'expérience ouverte dans l'éditeur |
| `signals.json` | La bibliothèque de signaux |
| `emulators.json` | La bibliothèque d'émulateurs |
| `runs/` | Un rapport pour chaque exécution |
| `exports/` | Les expériences exportées depuis la boîte de dialogue des expériences |
| `capture-….jsonl`, `capture-….txt` | Les exports de l'Inspecteur |

Les fichiers sont du JSON, écrits en entier. Si l'un ne peut pas être lu, Signal Lab dit quel fichier
et où est l'erreur, et le laisse tel quel plutôt que de repartir de zéro. Voir
[Fichiers et dossiers](../reference/files.md).

## Bureau et serveur {#desktop-and-server}

L'application de bureau et un serveur exécutent le même moteur derrière la même interface. Ce qui
diffère :

| | Application de bureau | Serveur, dans un navigateur |
| --- | --- | --- |
| Où le trafic part, où les moniteurs écoutent | Cet ordinateur | Le serveur |
| Dossier de données | `Documents/SignalLab` | Celui du serveur ; survolez [[ui:app.server]] dans l'en-tête pour le voir |
| Secrets | Gestionnaire d'identification Windows, définis dans l'application ; aucun sous Linux | En lecture seule, depuis l'environnement du serveur ou des fichiers de secrets |
| Rapports, exports, captures | Écrits dans le dossier de données ; le chemin est affiché | Téléchargés par le navigateur |
| Connexion | — | Avec le jeton d'accès du serveur, quand il en a un |
| Tâches, le bocal à cookies de l'écran HTTP | Celles de l'application | Celles du serveur, partagées par chaque page qui y est connectée |
| Pare-feu | Un avertissement propose d'autoriser Signal Lab (Windows) | Jamais modifié par Signal Lab |
| Mises à jour | Installe les versions signées quand vous cliquez | Mis à jour avec son image |

Voir [Le serveur](../server/index.md) et [Sécurité du serveur](../server/security.md).

## Ce que Signal Lab ne fait pas de lui-même {#on-its-own}

- **Il n'envoie que quand vous agissez**, et seulement vers les adresses que vous saisissez. Démarrer l'application
  n'envoie rien — sauf, dans l'application de bureau, la recherche quotidienne de mises à jour, que vous pouvez
  désactiver. Les retours ne partent que quand vous envoyez le formulaire.
- **Ses exemples restent sur cet ordinateur.** Les signaux de départ, les émulateurs de départ,
  les nouveaux émulateurs et les modèles d'expérience utilisent tous `127.0.0.1`. Les écouteurs que vous démarrez —
  un moniteur OSC, l'écouteur de découverte, un relais de dégradation — sont par défaut sur `0.0.0.0`, toutes les
  cartes réseau, pour que d'autres machines puissent les joindre ; saisissez `127.0.0.1` pour en garder un sur
  cet ordinateur.
- **Il ne change le pare-feu que quand vous cliquez** sur [[ui:fw.allow]] et confirmez l'invite d'administrateur de Windows,
  ou exécutez `signallab firewall allow`. Un serveur ne change jamais le pare-feu de son
  hôte.
- **Un serveur sans jeton d'accès** n'écoute que sur `127.0.0.1`, et refuse de démarrer
  sur toute autre adresse.
- **Il s'en tient à des garde-fous** : un balayage de diffusion atteint au plus 1024 hôtes, et une balise
  envoie au plus 50 000 paquets par seconde sur l'ensemble de ses cibles.

Les garde-fous ne sont pas une autorisation : [Tempête](../tools/storm.md), le
[Scanner](../tools/scanner.md) et [Diffusion](../protocols/broadcast.md) envoient du vrai
trafic. Ne les utilisez que sur des réseaux et des hôtes qui vous appartiennent ou que vous êtes autorisé à tester.

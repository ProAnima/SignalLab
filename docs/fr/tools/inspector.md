---
title: "Inspecteur"
description: "Capturez chaque trame que les outils de Signal Lab envoient et reçoivent, filtrez-les et lisez-les décodées et en octets, exportez-les, et conservez-en une comme signal."
---

# Inspecteur

L’Inspecteur est une seule chronologie pour chaque outil : chaque message OSC,
datagramme, échange HTTP, publication MQTT, message WebSocket, paquet relayé et
échange d’émulateur y atterrit, décodé, avec les octets dont il était fait.
Utilisez-le pour voir ce qui est réellement passé sur le fil, dans quel ordre, et
ce qui lui est arrivé.

Il vit dans le panneau inférieur, comme l’onglet [[ui:dock.inspector]] à côté de la
console, si bien qu’il est là sur chaque écran. Cliquez sur l’onglet pour l’ouvrir ;
le panneau s’ouvre assez haut pour quelques lignes et le détail d’une trame. Le
bouton ⤢ ([[ui:dock.maximise]]) rend le panneau aussi haut que la fenêtre.
L’Inspecteur conserve sa liste et sa sélection quand vous fermez le panneau ou
changez d’écran.

## Capturer {#capture}

La capture est désactivée au démarrage de Signal Lab, et tant qu’elle l’est, elle
ne coûte rien : les outils ne construisent même pas de trames.

1. Ouvrez l’onglet [[ui:dock.inspector]].
2. Appuyez sur [[ui:ins.arm]]. Le point de l’onglet devient rouge et pulse.
3. Utilisez n’importe quel outil. Les trames apparaissent en haut de la liste, la
   plus récente en premier.
4. Appuyez sur [[ui:ins.disarm]] quand vous avez ce qu’il vous faut.

L’onglet indique combien de trames ont été capturées, depuis n’importe quel écran.

Les trames sont capturées à partir du moment où vous armez la capture, jamais
avant : armez-la d’abord, puis envoyez.

Sur un [serveur](../server/index.md), la capture appartient au serveur : chaque page
qui y est connectée voit les mêmes trames, et l’armer ou la vider sur une page le
fait pour toutes.

### Ce qui est capturé {#sources}

| Outil | Trames | Combien |
| --- | --- | --- |
| [OSC](../protocols/osc.md) : envoi | Chaque message envoyé | Chacune |
| [OSC](../protocols/osc.md) : moniteur | Chaque paquet reçu ; un paquet qui ne se décode pas est marqué avec l’erreur de décodage | Chacune |
| [OSC](../protocols/osc.md) : générateur de signal | Messages envoyés | Au plus un par 100 ms, marqué `sampled` |
| [Diffusion](../protocols/broadcast.md) : envoi unique | Chaque datagramme, un par cible ; un envoi échoué avec son erreur | Chacune |
| [Diffusion](../protocols/broadcast.md) : balise | Datagrammes envoyés | Au plus un par 50 ms |
| [Diffusion](../protocols/broadcast.md) : écouteur de découverte | Sondes reçues | Au plus une par 40 ms |
| [Diffusion](../protocols/broadcast.md) : écouteur de découverte | Ses réponses (`auto-reply`) | Chacune |
| [HTTP](../protocols/http.md) : envoi, signaux, requêtes d’expérience | Chaque échange : la ligne de requête, le statut et le temps, les en-têtes de réponse et le début du corps | Chacun |
| [HTTP](../protocols/http.md) : rafale de charge, et requêtes sous charge | Échanges | Au plus un par 100 ms |
| [MQTT](../protocols/mqtt.md) : connexion | Publications envoyées | Chacune |
| [MQTT](../protocols/mqtt.md) : connexion | Messages reçus | Au plus un par 200 ms |
| [MQTT](../protocols/mqtt.md) : un signal MQTT envoyé alors que l’écran n’est pas connecté à son broker | La publication | Chacune |
| [WebSocket](../protocols/websocket.md) | Messages envoyés et reçus | Chacun tant que le trafic est léger ; au plus 200 par seconde |
| [Émulateurs](emulators.md) | Ce qui arrive et la réponse, ensemble | Au plus un échange par 10 ms |
| [Dégradation](impairment.md) | Chaque datagramme ou bloc relayé, dans les deux sens, avec son sort | Au plus un par 25 ms pour les deux sens réunis |
| [Storm](storm.md) (UDP) | Paquets du flot, tous identiques | Un par seconde, marqué `sampled 1/s` ; une tempête TCP n’en capture aucun |
| [Scanner](scanner.md) | Chaque port ouvert, avec sa bannière | Chacun |
| [Expériences](../experiments/index.md) | Ce que les étapes d’une exécution envoient (un message TCP : la charge utile écrite et la réponse lue), et ce que ses attentes reçoivent | Comme l’outil qu’elle utilise |

Un outil qui échantillonne laisse le reste de côté exprès, et les compte : la trame
suivante qu’il dessine indique combien il en a retenues, dans son verdict sous la
forme `+n not shown` (`sampled · +5 not shown`). Le compte porte sur ce que l’outil
aurait dessiné, pas sur ce que la capture elle-même a abandonné (voir [Comptes et
lacunes](#counts)).

## La liste des trames {#list}

| Colonne | Quoi |
| --- | --- |
| [[ui:common.time]] | Quand elle a été capturée, à la milliseconde près. |
| [[ui:ins.dir]] | → envoyé (`tx`), ← reçu (`rx`). Pour un relais, → va du client à la cible et ← de la cible au client. |
| [[ui:bc.proto]] | `osc`, `udp`, `tcp`, `http`, `mqtt` ou `ws`. |
| [[ui:bc.peer]] | L’autre côté : un `IP:port`, une URL, un broker. |
| [[ui:common.bytes]] | La taille de la trame. |
| [[ui:ins.summary]] | Une ligne dans la notation propre au protocole, comme `/fader/1 0.75` ou `GET http://127.0.0.1:8080/ → 200 in 3ms`. |
| [[ui:ins.verdict]] | Ce qui lui est arrivé, quand il y a quelque chose à dire. |

Le verdict est vert, ambre ou rouge. Rouge est une perte ou un échec
(`dropped (loss)`, `failed`, `error: …`) ; ambre est une trame altérée ou un simple
échantillon parmi beaucoup (`corrupted`, `copy 2/2`, `sampled`, `+n not shown`) ;
vert est le reste. Quelques verdicts que vous rencontrerez :

| Verdict | De | Signifie |
| --- | --- | --- |
| `forwarded +42ms` | Dégradation | Transmis après ce délai ; `· corrupted`, `· reordered` ou `· copy 1/2` peuvent suivre. |
| `dropped (loss)`, `dropped (burst)`, `dropped (offline)` | Dégradation | Perdu exprès, et pourquoi. |
| `throttled` | Dégradation | Abandonné par la limite de bande passante. |
| `· client→target`, `· target→client` | Dégradation | Termine le verdict de chaque trame relayée, avant tout `+n not shown` : dans quel sens elle allait. |
| `#2 → 200 OK · 37 B`, `— → 404 …` | Émulateurs | Quelle règle a répondu (`—` : aucune) et la réponse. |
| `down`, `down → 503` | Émulateurs | Elle est arrivée pendant que l’émulateur était en panne. |
| `200 OK`, `failed` | HTTP | Le statut de la réponse, ou aucune réponse du tout. |
| `open` | Scanner | Un port ouvert. |
| `auto-reply` | Découverte | Une réponse que l’écouteur a envoyée à une sonde. |
| `clears retained` | MQTT | Une publication retenue vide. |
| `+n not shown` | Tout outil qui échantillonne | Autant de trames depuis la précédente ont été laissées de côté ; il suit l’autre verdict de la trame après un `·`. |

La liste conserve les 4000 trames les plus récentes et dessine les 300 plus
récentes qui correspondent aux filtres ; sous les filtres, elle indique combien
elle en montre sur combien correspondent.

### Filtrer {#filter}

- Tapez dans le champ de texte ([[ui:ins.filterPlaceholder]]) pour ne garder que
  les trames dont le résumé, le pair, la source, le protocole ou le verdict contient
  le texte.
- Cliquez sur les pastilles de protocole (`osc`, `udp`, `tcp`, `http`, `mqtt`, `ws`)
  pour n’afficher que ces protocoles. Sans pastille active, tous les protocoles
  s’affichent.
- Cliquez sur `tx` ou `rx` pour n’afficher que les trames envoyées ou seulement
  reçues.
- [[ui:common.reset]] efface les trois.

Les filtres ne changent que ce que la liste affiche. La capture, les comptes et un
export couvrent toujours tout.

### Mettre en pause et vider {#pause}

[[ui:ins.pause]] fige la liste pour que vous puissiez la lire pendant que le trafic
continue ; la capture se poursuit. [[ui:ins.resume]] laisse de nouveau entrer les
nouvelles trames. Les trames arrivées pendant que la vue était en pause ne sont pas
ajoutées à la liste, mais elles sont dans la capture et dans un export.

[[ui:common.clear]] vide la liste et la capture, et réinitialise ses comptes.

### Comptes et lacunes {#counts}

La barre en haut compte les trames capturées et leurs octets, et à quel point la
capture est pleine (trames retenues sur 8192).

Quand les trames arrivent plus vite que la liste ne peut les prendre — plus de 250
en environ un huitième de seconde — la liste saute les plus anciennes. Une pastille
ambre compte alors les trames non affichées, et une ligne de la liste marque où
elles manquent. Ces trames sont toujours dans la capture, à moins que des plus
récentes ne les aient depuis poussées dehors : exportez-la pour les voir.

## Le détail d’une trame {#detail}

Cliquez sur une ligne pour voir la trame à droite.

| Champ | Quoi |
| --- | --- |
| [[ui:ins.seq]] | Le numéro de la trame. Les numéros croissent dans l’ordre de capture et ne sont jamais réutilisés. |
| [[ui:common.time]] | Quand elle a été capturée. |
| [[ui:ins.direction]] | Envoyée ou reçue. |
| [[ui:common.protocol]] | Comme dans la liste. |
| [[ui:ins.source]] | L’outil qui l’a capturée (`osc-send`, `osc-monitor`, `netsim`, `emulator`, `experiment-wait`, …), et son numéro de tâche quand elle en fait partie. |
| [[ui:ins.local]] | L’adresse de ce côté, quand il y en a une. Pour une trame relayée, l’adresse sur laquelle le relais écoute. |
| [[ui:bc.peer]] | L’autre côté. Pour une trame relayée, où elle allait : la cible, ou le client à qui la réponse est revenue. |
| [[ui:ins.size]] | Sa taille en octets. |
| [[ui:ins.verdict]] | Comme dans la liste. |

Sous [[ui:ins.decoded]] se trouve la trame lue dans son protocole : chaque message
d’un bundle OSC avec ses arguments, les en-têtes d’une réponse HTTP et le début de
son corps, la requête d’un émulateur et sa réponse.

Sous [[ui:ins.rawBytes]] se trouve un vidage hexadécimal : décalage, 16 octets en
hexadécimal, et les mêmes octets en texte. La liste porte le premier Kio de chaque
trame ; quand une trame est plus longue, un bouton sous le vidage charge tout.

### Ce qu’une trame conserve {#limits}

| Limite | Valeur | À la limite |
| --- | --- | --- |
| Octets qu’une trame conserve | 256 Kio | Une trame plus longue conserve ses 256 premiers Kio et indique quelle part du total elle a conservée. |
| Trames dans la capture | 8192 | La plus ancienne fait de la place. |
| Octets conservés par la capture au total | 64 Mio | Les plus anciennes font de la place. |

Certaines trames ne conservent aucun octet : les échanges HTTP (leur taille est
enregistrée, et les en-têtes de réponse et le début du corps sont dans le texte
décodé à la place) et les ports ouverts du Scanner.

Une trame MQTT conserve la charge utile du message, pas le paquet du protocole qui
l’entoure ; le topic, le QoS et l’indicateur retain sont dans son résumé.

### Secrets {#secrets}

Pendant qu’une exécution d’expérience ou [[ui:exp.sendNow]] utilise des
[secrets](../experiments/data.md#secrets), leurs valeurs sont masquées dans chaque
trame avant sa capture : `••••` dans le résumé, le texte décodé, les adresses et le
verdict, et `*` pour chaque octet de la charge utile, si bien que les décalages du
vidage restent exacts. Les identifiants de l’écran HTTP n’apparaissent jamais non
plus : une trame HTTP contient la réponse, pas l’en-tête `Authorization` qui a été
envoyé.

## Enregistrer une trame comme signal {#save-as-signal}

Pour conserver un paquet que vous avez attrapé et le renvoyer plus tard — quand
l’appareil qui l’a envoyé n’est plus là :

1. Sélectionnez la trame.
2. Appuyez sur [[ui:sig.fromFrame]].

Le signal va dans le dossier [[ui:sig.capturedFolder]] de la [bibliothèque de
signaux](signals.md) avec chaque octet de la trame, pris de ce que la capture a
conservé, pas du texte décodé. Il est nommé d’après le résumé de la trame, et sa
note dit de quelle trame il vient.

Ce que vous obtenez dépend de la trame :

| Trame | Signal |
| --- | --- |
| Un datagramme OSC ou UDP | Un signal UDP brut avec les octets de la trame, en hexadécimal. |
| Une publication MQTT — envoyée, reçue, ou celle d’un émulateur | Un signal MQTT avec le broker, le topic, le QoS et l’indicateur retain de la trame, et sa charge utile en texte, exactement telle qu’elle était. Une charge utile vide est conservée, si bien que l’effacement d’une valeur retenue peut être enregistré. |
| Tout le reste : un flux TCP (y compris celui qu’un relais a transporté, ou celui du nœud TCP), un échange HTTP, un message WebSocket, un paquet MQTT qui n’est pas une publication (l’abonnement d’un client à un émulateur) | Rien : le bouton est désactivé, et son infobulle dit pourquoi. Un signal envoie un datagramme ou une publication ; ceux-ci ne peuvent pas être renvoyés tels quels. |

Un datagramme est envoyé vers :

- une trame reçue — l’adresse qui l’a reçue (le côté [[ui:ins.local]]), pour que le
  signal remplace l’expéditeur ;
- une trame envoyée — le pair auquel elle a été envoyée ;
- une trame relayée, dans un sens ou l’autre — l’adresse vers laquelle elle allait :
  la cible pour une trame venant du client, le client pour une réponse.

Quand cette adresse est toutes les adresses de cet ordinateur — un moniteur
écoutant sur `0.0.0.0:9000` ou `[::]:9000` — le signal vise cet ordinateur à la
place : `127.0.0.1:9000` ou `[::1]:9000`. Ouvrez le signal et changez sa cible si
vous visez une autre adresse.

Un signal MQTT va au broker indiqué par la trame. Un broker écoutant sur toutes les
adresses est joint sur `127.0.0.1` de la même façon.

Le bouton est aussi désactivé pour :

- une trame non conservée en entier : une plus grande que 256 Kio, ou une qui n’a
  conservé aucun octet ;
- un message MQTT dont la charge utile n’est pas du texte — la charge utile d’un
  signal est du texte, donc ses octets ne pourraient pas être renvoyés tels quels ;
- une trame reçue qui ne nomme aucune socket de ce côté, donc aucune adresse
  d’envoi.

Une trame que la capture a déjà laissée partir ne peut pas être enregistrée non
plus ; la console le dit. Tant que le fichier de bibliothèque ne peut pas être lu,
[[ui:sig.fromFrame]] est aussi désactivé, et l’infobulle montre l’erreur du fichier
(voir [Signaux](signals.md#file)).

## Exporter {#export}

[[ui:ins.exportJsonl]] et [[ui:ins.exportTxt]] écrivent toute la capture — jusqu’à
8192 trames, chaque octet que chacune a conservé, quoi qu’affichent les filtres —
dans un fichier `capture-<time>.jsonl` ou `capture-<time>.txt` dans le dossier de
données (voir [Fichiers](../reference/files.md)). La console indique où. Dans un
navigateur connecté à un serveur, le fichier est écrit sur le serveur et votre
navigateur le télécharge.

Une capture vide n’est pas écrite ; la console dit qu’il n’y a rien à enregistrer.

- **`.jsonl`** — un objet JSON par ligne, une ligne par trame : `seq`, `ts`
  (millisecondes depuis 1970), `proto`, `dir`, `source`, `job_id`, `local`,
  `remote`, `bytes`, `kept`, `summary`, `detail`, `hex` (le vidage du premier Kio),
  `verdict`, et `data`, les octets conservés en base64.
- **`.txt`** — pour la lecture : une ligne par trame avec son numéro, son heure,
  son sens, son protocole, son pair, sa taille et son verdict, puis son résumé, son
  texte décodé et un vidage hexadécimal de chaque octet qu’elle a conservé.

```json
{"seq":12,"ts":1767225600123,"proto":"osc","dir":"rx","source":"osc-monitor","job_id":3,"local":"0.0.0.0:9000","remote":"127.0.0.1:53211","bytes":20,"summary":"/fader/1 0.75","detail":"/fader/1 0.75","hex":"0000  2f 66 61 64 65 72 2f 31  00 00 00 00 2c 66 00 00  |/fader/1....,f..|\n0010  3f 40 00 00                                       |?@..|\n","verdict":null,"kept":20,"data":"L2ZhZGVyLzEAAAAALGYAAD9AAAA="}
```

## Venir d’ailleurs {#reveal}

D’autres écrans pointent vers des trames : une attente dans la chronologie d’une
expérience lie la trame qu’elle a fait correspondre, et la liste des reçus d’un
émulateur a un bouton ⌕ ([[ui:emu.inspectFrame]]) sur chaque échange. En suivre un
ouvre l’Inspecteur avec cette trame sélectionnée, les filtres effacés et la vue
reprise.

## Voir aussi {#related}

- [Signaux](signals.md) — ce que devient une trame enregistrée.
- [Dégradation](impairment.md) — les verdicts du relais.
- [Émulateurs](emulators.md) — ce qu’un émulateur a reçu.

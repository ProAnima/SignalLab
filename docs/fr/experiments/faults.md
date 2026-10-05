---
title: "Défaillances"
description: "Relais de dégradation et émulateurs comme nœuds d’une exécution — un mauvais réseau et une dépendance défaillante déclenchés au bon moment, comptés phase par phase dans le rapport et reproductibles avec la graine."
---

# Les défaillances comme nœuds

Pour voir comment un système s’en sort quand le réseau se dégrade ou qu’une dépendance tombe
en panne, mettez la défaillance dans l’expérience. Un relais ou un émulateur s’ouvre avec
l’exécution, une étape le bascule au moment voulu, et le rapport d’exécution compte ce qui s’est passé dans chaque
phase. La fin de l’exécution — réussie, échouée ou arrêtée — les ferme : rien ne
reste dégradé derrière elle.

| Nœud | Rôle |
| --- | --- |
| [[ui:exp.node.impairment]] | un relais entre le système testé et sa cible, qui dégrade ce qui le traverse, pendant toute l’exécution |
| [[ui:exp.node.impairment_change]] | fait passer un relais de l’exécution à un autre profil, à partir de cette étape |
| [[ui:exp.node.emulator]] | une API, un appareil ou un broker joué par Signal Lab, pendant toute l’exécution |
| [[ui:exp.node.emulator_state]] | met en panne un émulateur de l’exécution, ou le remet en service |

Les quatre se trouvent dans le menu d’ajout, sous [[ui:exp.group.fault]] et
[[ui:exp.group.emulate]]. Leurs champs sont décrits dans
[la référence des nœuds](nodes.md) ; le relais lui-même est décrit dans
[Dégradation](../tools/impairment.md), les émulateurs dans
[Émulateurs](../tools/emulators.md).

## Dégradation {#impairment}

Le système testé envoie au relais au lieu de sa vraie cible ; le relais
transmet à la cible, rapporte les réponses et dégrade les deux sens.

| Champ | Quoi |
| --- | --- |
| [[ui:exp.relayListen]] | l’`IP:port` vers lequel le système testé envoie ou se connecte, port différent de 0 |
| [[ui:exp.relayTarget]] | l’`IP:port` de la vraie destination, ou `host:port` ; un nom d’hôte est résolu au démarrage de l’exécution |
| [[ui:ns.protocol]] | UDP — chaque datagramme connaît son propre sort — ou TCP — chaque connexion est jointe à une connexion qui lui est propre vers la cible |
| le profil | un [[ui:ns.preset]] ou des valeurs de votre choix |

Ce qu’un relais lit de son profil dépend du protocole ; les autres valeurs
sont ignorées :

| Protocole | Dégradations |
| --- | --- |
| UDP | latence, gigue, perte de paquets, pertes en rafale, duplication, corruption, réordonnancement, limite de bande passante, hors ligne |
| TCP | latence et gigue (un flux reste dans l’ordre), limite de bande passante (l’émetteur est ralenti, rien n’est abandonné), connexions réinitialisées, connexions laissées semi-ouvertes, hors ligne |

**Ouvert avant la première étape.** Chaque relais de l’expérience s’ouvre au démarrage de
l’exécution, comme les sockets des attentes : ses champs [[ui:exp.relayListen]] et
[[ui:exp.relayTarget]] n’acceptent donc que du texte et des paramètres
(`node.params_only`) — `{{relay}}` avec un paramètre `relay`, jamais une variable.
Un port qui ne peut pas être ouvert, ou un nom de cible qui ne peut pas être résolu, arrête l’exécution avant tout trafic, sur ce nœud.

**Franchi dans le flux.** Quand l’exécution atteint le nœud, elle le franchit aussitôt et
la chronologie indique ce qu’il dégrade, et comment. Le relais fonctionne du
début de l’exécution jusqu’à sa fin, où que se trouve le nœud dans le graphe.

**Fermé avec l’exécution.** Quelle que soit la fin de l’exécution, le relais se ferme ; les
connexions d’un relais TCP se ferment avec lui. Un relais qui a cessé de relayer de lui-même garde la
raison : les étapes qui l’utilisent échouent avec elle, et le rapport l’indique.

::: tip Faire passer par une dégradation
Sur un nœud [[ui:exp.node.osc]] ou [[ui:exp.node.udp]],
le bouton [[ui:exp.routeThrough]] place un nœud Dégradation devant lui : le relais écoute
sur un port libre de `127.0.0.1`, transmet à la cible du nœud avec le
préréglage [[ui:ns.preset.lan]], et le nœud envoie désormais au relais.
:::

## Changer la dégradation {#change-impairment}

Le nœud [[ui:exp.node.impairment_change]] désigne l’un des relais de l’expérience dans le champ
[[ui:exp.relay]] et donne le profil avec lequel il dégrade à partir de cette étape. Le
relais garde son port et ses connexions ; les nouvelles valeurs s’appliquent au paquet ou au segment
suivant. La chronologie affiche le nouveau profil.

Chaque changement termine une **phase**. Le rapport d’exécution conserve, pour chaque relais :

- ses adresses d’écoute et de cible, et son protocole quand c’est TCP ;
- ses compteurs au total : reçus, transmis, abandonnés, bridés, dupliqués,
  corrompus, réordonnés, octets — et pour TCP les connexions, celles réinitialisées et
  celles laissées semi-ouvertes ;
- chaque phase : le nom du profil, quand elle a commencé et fini, en millisecondes depuis
  l’ouverture du relais, et les mêmes compteurs pour cette seule phase.

Un paquet est compté dans la phase qui a décidé de son sort, même quand sa copie retardée
part après le changement. Un relais conserve ses 1000 dernières phases ; les plus anciennes
sont comptées, pas conservées.

Un nœud [[ui:exp.node.impairment_change]] qui ne désigne aucun relais de l’expérience est
refusé (`impair.relay_unknown`).

## Émulateur {#emulator}

Le nœud [[ui:exp.node.emulator]] joue une dépendance pendant toute l’exécution : une API HTTP, un
appareil OSC, UDP ou TCP, ou un broker MQTT. C’est le même émulateur que celui que l’écran
[Émulateurs](../tools/emulators.md) exécute de lui-même : le bouton
[[ui:emu.edit]] ouvre ses règles, [[ui:emu.toLibrary]] en garde une copie dans la
bibliothèque, [[ui:emu.fromLibrary]] en prend un dans celle-ci.

- Il s’ouvre avant la première étape et répond jusqu’à la fin de l’exécution ; un port qui
  ne peut pas être ouvert arrête l’exécution avant tout trafic. Dans le flux, il est franchi
  aussitôt.
- Son adresse est un `IP:port` littéral. Ses motifs de correspondance n’acceptent que des paramètres ;
  ses réponses sont des modèles lus avec ce qui est arrivé (`{{request.…}}`)
  et les paramètres de l’exécution. Il ne peut pas lire de secrets.
- Ses choix aléatoires — un mélange pondéré de réponses, la gigue du délai, les générateurs dans
  les réponses — sont tirés de la graine de l’exécution.
- Un émulateur HTTP est aussi ce qu’écoute un nœud [[ui:exp.node.wait_http]] sur la même adresse :
  celui-ci vérifie ce que le système testé a envoyé. Sans émulateur
  à cet endroit, l’écouteur propre à l’exécution répond `204` à chaque requête.
- Un émulateur OSC ou UDP partage son port avec les attentes de l’exécution sur ce port : les deux voient
  chaque datagramme.
- Un émulateur MQTT est un broker que les nœuds [[ui:exp.node.mqtt]] et
  [[ui:exp.node.wait_mqtt]] de l’exécution peuvent utiliser comme n’importe quel autre.

Le rapport d’exécution conserve, pour chaque nœud émulateur, son nom, son protocole et son adresse,
et ses compteurs : les requêtes au total, celles qu’aucune règle n’a prises, celles qui ont échoué, celles
qui l’ont trouvé en panne, les messages qu’un broker n’a pas pu remettre à un client lent, et
les correspondances de chaque règle.

## Émulateur en panne et en service {#emulator-state}

Le nœud [[ui:exp.node.emulator_state]] désigne l’un des émulateurs de l’exécution dans le champ
[[ui:exp.emulatorNode]] ; le champ [[ui:exp.emulatorDownState]] vaut
[[ui:exp.emulatorGoesDown]] ou [[ui:exp.emulatorComesUp]]. Pendant la panne :

| Émulateur | Ce qui se passe |
| --- | --- |
| HTTP | ce qu’indique le champ [[ui:exp.downFault]] : [[ui:emu.outageFault.unavailable]] (`503`, corps `{"error":"unavailable"}`), [[ui:emu.outageFault.reset]], ou [[ui:emu.outageFault.timeout]] — la requête est retenue jusqu’à ce que le client abandonne, 120 s au plus |
| Appareil TCP, broker MQTT | les connexions sont coupées et les nouvelles refusées |
| Appareil OSC, UDP | rien ne reçoit de réponse |

Ce qui arrive pendant la panne est compté comme `down`, jamais comme une requête qu’aucune règle
n’a prise. Un nœud [[ui:exp.node.wait_http]] voit toujours les requêtes. L’émulateur reste
en panne jusqu’à ce qu’une étape le remette en service, quoi que dise son propre calendrier de pannes, et la
fin de l’exécution le ferme de toute façon.

Un nœud [[ui:exp.node.emulator_state]] qui ne désigne aucun émulateur de l’expérience est
refusé (`emulator.node_unknown`).

## Pannes programmées {#outage}

Un émulateur peut aussi tomber en panne de lui-même : dans ses règles, [[ui:emu.outage]] définit
[[ui:emu.outageUp]] et [[ui:emu.outageDown]], chacun de 10–3 600 000 ms, et
[[ui:emu.outageFault]] pour HTTP. Il répond pendant la première durée, est en panne pendant la
seconde, et ainsi de suite, à compter de son ouverture — dans une exécution, avant la première
étape. Pendant une panne de son calendrier, le `503` d’un émulateur HTTP porte
`Retry-After` avec le nombre de secondes entières avant son retour, au moins 1 ; un `503`
alors qu’une étape [[ui:exp.node.emulator_state]] le maintient en panne n’en a pas, puisque
personne ne sait quand cela finira.

Un calendrier n’a pas besoin d’étape ; une étape n’a pas besoin de calendrier. Utilisez le calendrier pour une
dépendance qui flanche par intermittence, l’étape pour une panne à un point choisi du flux.

## Exemple : une panne derrière une liaison lente {#example}

Un client demande une commande à une API à travers un relais. Pendant qu’il demande, une deuxième
branche ralentit la liaison au niveau [[ui:ns.preset.4g]], met l’API en panne pendant deux
secondes, la remet en service et rend la liaison de nouveau propre. Le client doit continuer à
demander jusqu’à obtenir sa réponse.

```text
start → orders → link → split
split ─ branch1 → settle → until_ok ─ done → answered → joined
                           until_ok ─ body → get → status → pause → until_ok
split ─ branch2 → slow → down → outage → up → clean → joined
joined → end
```

Les noms sont les identifiants des nœuds dans le fichier ci-dessous.

1. Ajoutez un paramètre `api` = `http://127.0.0.1:18091` — le relais, pas l’API.
2. Ajoutez un nœud [[ui:exp.node.emulator]] : HTTP, `127.0.0.1:18090`, une route
   `GET /orders/:id` qui répond `200` avec `{"order":"{{request.params.id}}"}`.
3. Après lui, un nœud [[ui:exp.node.impairment]] : [[ui:exp.relayListen]]
   `127.0.0.1:18091`, [[ui:exp.relayTarget]] `127.0.0.1:18090`,
   [[ui:ns.protocol]] TCP, préréglage [[ui:ns.preset.lan]].
4. Après lui, un nœud [[ui:exp.node.fork]].
5. Sur [[ui:exp.branch1]], le client : un nœud [[ui:exp.node.delay]] de 300 ms, puis un nœud
   [[ui:exp.node.loop]] — [[ui:exp.loopMax]] 40, [[ui:exp.loopUntilOn]]
   `{{status}}` [[ui:exp.op.eq]] `200`. Son corps : un nœud [[ui:exp.node.http]]
   `GET {{api}}/orders/42`, un nœud [[ui:exp.node.extract]] qui place le
   [[ui:exp.from.status]] dans `status`, un délai de 250 ms, relié en retour à la
   Boucle. Sur [[ui:exp.portDone]], un nœud
   [[ui:exp.node.log]] `Orders API answers again: HTTP {{status}}`.
6. Sur [[ui:exp.branch2]], les défaillances : un nœud [[ui:exp.node.impairment_change]] qui fait passer
   le relais à [[ui:ns.preset.4g]] ; un nœud [[ui:exp.node.emulator_state]] qui met
   l’émulateur Orders API à l’état [[ui:exp.emulatorGoesDown]] avec
   [[ui:emu.outageFault.unavailable]] ; un délai de 2000 ms ; un autre nœud
   [[ui:exp.node.emulator_state]] qui le remet à l’état [[ui:exp.emulatorComesUp]] ;
   un autre nœud [[ui:exp.node.impairment_change]] qui revient à [[ui:ns.preset.lan]].
7. Reliez les deux branches à un nœud [[ui:exp.node.join]], et celui-ci au nœud
   [[ui:exp.node.end]].
8. Exécutez.

La chronologie montre les requêtes du client qui reçoivent `503` à travers la liaison lente,
l’API qui revient, puis `200` et la Boucle qui sort par
[[ui:exp.portDone]]. Le rapport compte environ cinq requêtes qui ont trouvé l’API
en panne et une qui a reçu la réponse de sa route, ainsi que les trois phases du relais —
[[ui:ns.preset.lan]] pendant un instant, [[ui:ns.preset.4g]] pendant la panne,
[[ui:ns.preset.lan]] de nouveau —, chacune avec son propre trafic.

::: details L’expérience sous forme de fichier
Enregistrez-la dans un fichier `.json` et ouvrez-la avec [[ui:exp.importJson]] dans
[[ui:exp.documents]].

```json
{
  "version": 9,
  "name": "Outage behind a slow link",
  "params": [{ "name": "api", "value": "http://127.0.0.1:18091" }],
  "profiles": [],
  "profile": null,
  "seed": null,
  "nodes": [
    { "id": "start", "type": "start", "x": 40, "y": 270 },
    { "id": "orders", "type": "emulator", "x": 260, "y": 270,
      "emulator": { "name": "Orders API", "bind": "127.0.0.1:18090", "protocol": "http",
        "routes": [{ "method": "GET", "path": "/orders/:id", "when": [], "order": "sequence",
          "responses": [{ "status": 200, "headers": [], "body": "{\"order\":\"{{request.params.id}}\"}", "delay_ms": 0, "jitter_ms": 0, "fault": "none", "weight": 1 }] }],
        "fallback": null } },
    { "id": "link", "type": "impairment", "x": 490, "y": 270, "listen": "127.0.0.1:18091", "target": "127.0.0.1:18090", "protocol": "tcp",
      "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } },
    { "id": "split", "type": "fork", "x": 720, "y": 270 },
    { "id": "settle", "type": "delay", "x": 950, "y": 140, "ms": 300 },
    { "id": "until_ok", "type": "loop", "x": 1180, "y": 140, "max": 40,
      "until": { "value": "{{status}}", "op": "eq", "expected": "200" } },
    { "id": "get", "type": "http", "x": 1410, "y": 20,
      "request": { "method": "GET", "url": "{{api}}/orders/42", "headers": [], "body": null, "timeout_ms": 3000 } },
    { "id": "status", "type": "extract", "x": 1640, "y": 20, "variable": "status", "from": "status", "expr": "" },
    { "id": "pause", "type": "delay", "x": 1870, "y": 20, "ms": 250 },
    { "id": "answered", "type": "log", "x": 1410, "y": 140, "message": "Orders API answers again: HTTP {{status}}" },
    { "id": "slow", "type": "impairment_change", "x": 950, "y": 400, "relay": "link",
      "profile": { "name": "4g", "latency_ms": 60, "jitter_ms": 25, "rate_kbps": 20000 } },
    { "id": "down", "type": "emulator_state", "x": 1180, "y": 400, "emulator": "orders", "down": true, "fault": "unavailable" },
    { "id": "outage", "type": "delay", "x": 1410, "y": 400, "ms": 2000 },
    { "id": "up", "type": "emulator_state", "x": 1640, "y": 400, "emulator": "orders", "down": false, "fault": "unavailable" },
    { "id": "clean", "type": "impairment_change", "x": 1870, "y": 400, "relay": "link",
      "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } },
    { "id": "joined", "type": "join", "x": 2100, "y": 270 },
    { "id": "end", "type": "end", "x": 2330, "y": 270 }
  ],
  "edges": [
    { "from": "start", "to": "orders" },
    { "from": "orders", "to": "link" },
    { "from": "link", "to": "split" },
    { "from": "split", "to": "settle", "port": "branch1" },
    { "from": "split", "to": "slow", "port": "branch2" },
    { "from": "settle", "to": "until_ok" },
    { "from": "until_ok", "to": "get", "port": "body" },
    { "from": "get", "to": "status" },
    { "from": "status", "to": "pause" },
    { "from": "pause", "to": "until_ok" },
    { "from": "until_ok", "to": "answered", "port": "done" },
    { "from": "answered", "to": "joined" },
    { "from": "slow", "to": "down" },
    { "from": "down", "to": "outage" },
    { "from": "outage", "to": "up" },
    { "from": "up", "to": "clean" },
    { "from": "clean", "to": "joined" },
    { "from": "joined", "to": "end" }
  ]
}
```
:::

Deux modèles de [[ui:exp.documents]] font la même chose autrement :
[[ui:exp.templateFaults]] envoie des datagrammes à un appareil émulé à travers un relais UDP
basculé tour à tour sur propre, avec pertes, hors ligne, puis de nouveau propre ;
[[ui:exp.templateOutage]] met une API émulée en panne pendant deux secondes pendant qu’un
client continue de demander.

## Ports {#ports}

Les sockets d’une même exécution — attentes, réponses, émulateurs, relais — ne peuvent pas partager un
port d’un même protocole ; un socket UDP et un socket TCP peuvent utiliser le même numéro. Pour le champ
[[ui:exp.relayListen]] d’un relais, une adresse sur `0.0.0.0` entre en conflit avec toutes les
adresses sur le même port.

| Socket | Ne peut pas partager son port avec |
| --- | --- |
| un émulateur HTTP, TCP ou MQTT | un autre de ces émulateurs (`emulator.bind_taken`) |
| un émulateur OSC ou UDP | un autre de ces émulateurs (`emulator.bind_taken`) |
| un émulateur TCP ou MQTT | un nœud [[ui:exp.node.wait_http]] (`emulator.bind_taken`) |
| le champ [[ui:exp.relayListen]] d’un relais UDP | un autre relais UDP, un émulateur OSC ou UDP, une attente ou le socket d’une réponse (`impair.bind_taken`) |
| le champ [[ui:exp.relayListen]] d’un relais TCP | un autre relais TCP, un émulateur HTTP, TCP ou MQTT, un nœud [[ui:exp.node.wait_http]] (`impair.bind_taken`) |

Partagés exprès : un émulateur HTTP et les étapes [[ui:exp.node.wait_http]] sur
son adresse ; un émulateur OSC ou UDP et les attentes sur son port ; les attentes sur une même
adresse entre elles.

Un relais ne peut pas transmettre vers lui-même, directement ou à travers d’autres relais : son
trafic tournerait en rond en boucle locale (`impair.loop`). Deux relais à la suite devant
un appareil ne posent pas de problème.

## Reproduire une exécution défaillante {#seed}

Chaque décision que prend un relais — si un paquet est perdu, dupliqué, corrompu
ou retenu, combien de gigue il subit — est tirée de la graine de l’exécution, séparément
pour chaque sens et paquet par paquet. Les choix aléatoires d’un émulateur en sont tirés
aussi. Relancez avec la même graine et le même trafic, et les mêmes
paquets connaissent le même sort : un échec vu une fois peut être revu.

Pour garder la graine, appuyez sur [[ui:exp.pinSeed]] à côté d’elle dans la chronologie, ou lancez l’exécution
avec elle depuis [[ui:exp.runWith]] ; voir [graines](runs.md#seeds). Ce que la graine
ne peut pas figer, c’est la temporisation : le moment où le système testé envoie, et donc la
phase dans laquelle tombe un paquet.

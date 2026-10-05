---
title: "Émulateurs"
description: "Faites de Signal Lab l’autre côté — une API HTTP, un appareil OSC, UDP ou TCP, ou un broker MQTT — qui répond selon vos règles, échoue à la demande et compte ce qui arrive."
---

# Émulateurs

Un émulateur, c’est Signal Lab qui joue l’API, l’appareil ou le service auquel votre système
s’adresse. Il écoute sur une adresse et répond selon des règles : une API HTTP par routes, un
appareil OSC, UDP ou TCP par « sur ceci, répondre cela », un broker MQTT comme le fait tout broker,
plus des règles qui lui sont propres. Il peut être lent, échouer ou tomber en panne de temps en temps, pour
que vous puissiez tester ce que fait votre système quand sa dépendance se comporte mal. Chaque
échange est compté, listé et envoyé à l’[Inspecteur](inspector.md).

Un émulateur est un seul document. L’écran [[ui:nav.emulators]] en conserve une bibliothèque ;
le même document s’exécute dans une expérience comme nœud
[[ui:exp.node.emulator]], depuis la ligne de commande avec `signallab emulate`,
et via l’[API](../api/commands.md) et [MCP](../automation/mcp.md), et
répond de la même façon partout.

## L’écran {#screen}

À gauche se trouve la bibliothèque ([[ui:emu.library]]) : chaque émulateur avec son
protocole et son adresse, un point qui pulse et un compteur de requêtes sur ceux qui tournent.
À droite se trouvent les réglages et les règles de l’émulateur sélectionné, et en dessous
ce qu’il a reçu ([[ui:emu.live]]).

## Créer un émulateur {#create}

1. Appuyez sur l’un des boutons en haut de la bibliothèque :

   | Bouton | Crée | Écoute sur | Avec une règle qui fonctionne telle quelle |
   | --- | --- | --- | --- |
   | ＋ [[ui:emu.new.http]] | Une API HTTP | `127.0.0.1:18080` | `GET /health` → 200 `{"status":"ok"}` |
   | ＋ [[ui:emu.new.osc]] | Un appareil OSC | `127.0.0.1:9100` | `/ping` → `/pong` avec le compteur en int |
   | ＋ [[ui:emu.new.udp]] | Un appareil UDP | `127.0.0.1:7100` | un datagramme contenant `PING` → `PONG 1`, `PONG 2`, … |
   | ＋ [[ui:emu.new.tcp]] | Un appareil TCP | `127.0.0.1:7200` | une ligne contenant `PING` → `PONG` |
   | ＋ [[ui:emu.new.mqtt]] | Un broker MQTT | `127.0.0.1:1883` | une publication sur `lab/<name>/set` → la même charge utile, retenue, sur `lab/<name>/state` |

   Quand un autre émulateur de la bibliothèque utilise déjà ce port, le port libre suivant
   est pris.
2. Donnez-lui un nom dans le champ [[ui:emu.name]] (120 caractères au plus).
3. Réglez le champ [[ui:emu.bind]] : `IP:port`. `127.0.0.1` ne répond qu’à cet ordinateur ;
   `0.0.0.0` répond aussi au réseau.
4. Modifiez les règles (voir plus bas), et indiquez dans le champ [[ui:emu.note]] ce qu’il remplace.

Les modifications s’enregistrent d’elles-mêmes. Le bouton [[ui:emu.duplicate]] crée une copie sur le port libre
suivant. Le bouton [[ui:emu.delete]] demande une confirmation ([[ui:emu.confirmDelete]]), arrête
l’émulateur s’il tourne et le retire de la bibliothèque.

Les règles sont essayées dans l’ordre, de la première à la dernière ; la première qui correspond répond. L’en-tête de chaque
règle affiche un résumé d’une ligne ; cliquez dessus pour déplier ou replier la règle. Les
boutons ↑ et ↓ déplacent une règle, × la supprime.

## L’exécuter {#run}

1. Sélectionnez l’émulateur et appuyez sur [[ui:emu.start]]. Son port s’ouvre avant que le
   bouton ne revienne : un port déjà pris, ou un émulateur qui a un problème, est
   refusé à ce moment-là avec la raison.
2. Dirigez votre système vers lui. Pour une API HTTP, le bouton [[ui:emu.copyUrl]] copie son
   adresse (`http://127.0.0.1:18080`), et chaque route a son propre bouton
   [[ui:emu.copyRouteUrl]] (sauf quand son chemin contient un
   modèle `{{…}}`).
3. Regardez la liste [[ui:emu.received]] se remplir.
4. Appuyez sur [[ui:emu.stop]], ou arrêtez sa tâche depuis le bandeau de la console.

L’état affiché à côté des boutons indique [[ui:emu.notRunning]], l’adresse où il répond, ou
qu’il est en panne.

Un émulateur continue de répondre avec les règles avec lesquelles il a démarré. Quand vous le
modifiez pendant qu’il tourne, le bouton [[ui:emu.restart]] apparaît : appuyez dessus pour redémarrer
avec les règles telles qu’elles sont maintenant. D’ici là, les compteurs de correspondances des règles sont
masqués, car ils appartiennent aux anciennes règles.

Le bouton [[ui:emu.takeDown]] rend un émulateur en cours indisponible jusqu’à ce que vous appuyiez sur
[[ui:emu.bringUp]] : une requête HTTP reçoit 503, un appareil TCP et un broker MQTT
coupent leurs connexions et refusent les nouvelles, un appareil OSC ou UDP ne répond
plus. Voir [Tomber en panne](#outage).

Deux émulateurs d’un même transport ne peuvent pas partager un port : les émulateurs HTTP, TCP et MQTT
écoutent sur des ports TCP, les émulateurs OSC et UDP sur des ports UDP. Une API HTTP
et un appareil OSC peuvent tous deux utiliser le port 8080 ; deux API HTTP ne le peuvent pas. Un second émulateur sur
un port pris est refusé au démarrage.

::: tip
Dans un navigateur connecté à un [serveur](../server/index.md), l’émulateur tourne sur
le serveur. Celui qui écoute sur `0.0.0.0` est joint par le nom du serveur, et
[[ui:emu.copyUrl]] copie cette adresse ; celui qui écoute sur `127.0.0.1` ne répond qu’aux
programmes du serveur lui-même.
:::

## Ce qui est arrivé {#received}

Pendant qu’il tourne, le panneau [[ui:emu.live]] compte :

| Compteur | Quoi |
| --- | --- |
| [[ui:emu.total]] | Tout ce qui est arrivé : requêtes, messages, lignes. |
| [[ui:emu.unmatched]] | Ce qu’aucune règle n’a pris. Une requête HTTP sans route reçoit tout de même sa réponse (voir [Requêtes qu’aucune route ne prend](#fallback)) ; les autres n’en reçoivent aucune. |
| [[ui:emu.failed]] | Les échanges pour lesquels une réponse n’a pas pu être construite ou envoyée. |
| [[ui:emu.down]] | Ce qui est arrivé pendant que l’émulateur était en panne. Affiché quand une panne est programmée ou que quelque chose l’a trouvé en panne. Jamais compté comme [[ui:emu.unmatched]]. |
| [[ui:emu.missed]] | MQTT uniquement, quand cela se produit : les messages qu’un client avait trop de retard pour recevoir. |

L’en-tête de chaque règle indique combien de fois elle a correspondu depuis le démarrage.

La liste [[ui:emu.received]] affiche les 300 échanges les plus récents, le plus récent en premier :

| Colonne | Quoi |
| --- | --- |
| [[ui:emu.col.time]] | Quand il est arrivé. |
| [[ui:emu.col.from]] | L’adresse du client. |
| [[ui:emu.col.request]] | Ce qui est arrivé, en notation du protocole : `GET /users/7`, `/ping 1`, `POWER?`. |
| [[ui:emu.col.rule]] | La règle qui l’a pris (`#2`), ou `—`. |
| [[ui:emu.col.reply]] | Ce qui est reparti : `200 OK · 37 B`, `/pong 3`, une charge utile ; [[ui:emu.held]] ou [[ui:emu.closed]] pour une défaillance ; l’erreur quand la réponse a échoué ; [[ui:emu.wasDown]] quand il est arrivé pendant une panne. |
| [[ui:emu.col.ms]] | De l’arrivée jusqu’au départ de la réponse, délai compris. |

Le bouton ⌕ d’une ligne ([[ui:emu.inspectFrame]]) ouvre cet échange dans
l’Inspecteur, si la capture était active. Quand plus de 200 échanges arrivent en un
cinquième de seconde, la liste en saute certains et indique combien. Le moteur conserve les
500 échanges les plus récents de chaque émulateur en cours, avec ce qui est arrivé, pour la
ligne de commande, l’API et MCP.

## API HTTP {#http}

Un serveur HTTP/1.1. Chaque requête reçoit la réponse de la première route qui la prend.

### Routes {#routes}

Une route prend une requête quand sa méthode, son chemin et toutes ses conditions correspondent.

| Champ | Quoi |
| --- | --- |
| [[ui:emu.method]] | `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS`, ou [[ui:emu.methodAny]]. Une route `GET` répond aussi à `HEAD`. |
| [[ui:emu.path]] | Commence par `/`. Un segment `:name` prend n’importe quel segment unique, lu comme `{{request.params.name}}` ; un dernier segment `*` prend tout ce qui se trouve en dessous. Un `/` final ne change rien ; la chaîne de requête ne fait pas partie du chemin. |
| [[ui:emu.conditions]] | Chacune doit être remplie. Ajoutez-en une avec ＋ [[ui:emu.addCondition]]. |

Exemples de chemins :

| Chemin | Prend | Ne prend pas |
| --- | --- | --- |
| `/health` | `/health`, `/health/` | `/health/db`, `/Health` |
| `/users/:id` | `/users/7` (`params.id` vaut `7`), `/users/a%20b` (`a b`) | `/users`, `/users/7/orders` |
| `/files/*` | `/files`, `/files/a`, `/files/a/b/c` | `/file`, `/other/files/a` |

Une condition lit une partie de la requête ([[ui:emu.on]]) et la compare :

| [[ui:emu.on]] | Nom | Lit |
| --- | --- | --- |
| [[ui:emu.on.header]] | Un nom d’en-tête, casse indifférente | La valeur de l’en-tête ; pour un en-tête envoyé plusieurs fois, ses valeurs jointes par `, `. |
| [[ui:emu.on.query]] | Un paramètre de la chaîne de requête | Sa valeur, décodée ; la première, s’il est répété. |
| [[ui:emu.on.body]] | — | Tout le corps, en texte. |
| [[ui:emu.on.json]] | Un chemin JSON, comme `$.user.id` | Ce champ d’un corps JSON. |

Les comparaisons sont [[ui:exp.op.eq]], [[ui:exp.op.ne]], [[ui:exp.op.lt]],
[[ui:exp.op.le]], [[ui:exp.op.gt]], [[ui:exp.op.ge]], [[ui:exp.op.contains]],
[[ui:exp.op.matches]], [[ui:exp.op.empty]] et [[ui:exp.op.not_empty]]. Les nombres
se comparent comme des nombres, le texte à l’identique. Un en-tête, un paramètre ou un champ
absent est vide. Une comparaison impossible — du texte face à un nombre —
n’est pas remplie.

### Réponses {#responses}

Une route a de une à 16 réponses ([[ui:emu.responses]]).

| Champ | Quoi | Par défaut |
| --- | --- | --- |
| [[ui:emu.status]] | 100–599. | 200 |
| [[ui:emu.fault]] | Autre chose qu’une réponse ; voir [Défaillances](#faults). | [[ui:emu.fault.none]] |
| [[ui:emu.delay]] | Combien de temps attendre avant de répondre, 0–60 000 ms. | 0 |
| [[ui:emu.jitter]] | Jusqu’à autant de plus, au hasard, 0–60 000 ms. | 0 |
| [[ui:emu.weight]] | Sa part quand la route répond au hasard. Affiché seulement dans ce cas. | 1 |
| [[ui:emu.headers]] | Jusqu’à 32. Les noms peuvent utiliser des paramètres ; les valeurs sont des [modèles](#templates). | aucun |
| [[ui:emu.body]] | Un [modèle](#templates), jusqu’à 256 Kio tel qu’écrit. | vide |

Sans en-tête `Content-Type`, un corps qui est du JSON valide part en
`application/json` et tout autre corps en `text/plain; charset=utf-8`.

Avec deux réponses ou plus, le champ [[ui:emu.order]] décide laquelle une requête reçoit :

| [[ui:emu.order]] | Les requêtes reçoivent | Pour |
| --- | --- | --- |
| [[ui:emu.order.sequence]] | La première, la deuxième, …, puis la dernière à partir de là : 500, 500, 200, 200, 200… | Les réessais : échouer deux fois, puis fonctionner. |
| [[ui:emu.order.cycle]] | De nouveau la première après la dernière : 200, 500, 200, 500… | Une dépendance qui échoue de temps en temps, régulièrement. |
| [[ui:emu.order.random]] | Chacune tirée selon son poids. Des poids de 8 et 2 donnent la première environ 80 % du temps. Au moins un poids doit être supérieur à 0. | Une part réaliste d’échecs. |

Le menu [[ui:emu.preset]] ajoute à la route une réponse toute prête :

| Préréglage | Ajoute |
| --- | --- |
| [[ui:emu.preset.ok]] | 200, `{"ok":true}` |
| [[ui:emu.preset.created]] | 201, `{"id":"{{uuid}}"}`, en-tête `Location: {{request.path}}/{{counter}}` |
| [[ui:emu.preset.notFound]] | 404, `{"error":"not found"}` |
| [[ui:emu.preset.error]] | 500, `{"error":"internal"}` |
| [[ui:emu.preset.unavailable]] | 503, `{"error":"unavailable"}`, en-tête `Retry-After: 1` |
| [[ui:emu.preset.slow]] | 200, `{"ok":true}` au bout de 2000 ms |
| [[ui:emu.preset.timeout]] | La défaillance [[ui:emu.fault.timeout]] |
| [[ui:emu.preset.reset]] | La défaillance [[ui:emu.fault.reset]] |
| [[ui:emu.preset.malformed]] | 200, `{"items":[{"id":1},{"id":2}]}` avec la défaillance [[ui:emu.fault.malformed]] |

### Défaillances {#faults}

| [[ui:emu.fault]] | Ce que rencontre le client |
| --- | --- |
| [[ui:emu.fault.none]] | La réponse. |
| [[ui:emu.fault.timeout]] | Rien. La requête est retenue jusqu’à 2 minutes, puis la connexion est fermée — c’est donc le délai d’attente du client lui-même qui est testé. Le délai ne s’applique pas. |
| [[ui:emu.fault.reset]] | La connexion se ferme sans réponse, après le délai. |
| [[ui:emu.fault.malformed]] | Une réponse HTTP complète, avec le statut et les en-têtes définis, dont le corps s’arrête à mi-chemin : du JSON qui ne s’analyse pas. Quand tout le corps était du JSON, le type de contenu indique toujours `application/json`. |

### Requêtes qu’aucune route ne prend {#fallback}

Le champ [[ui:emu.fallback]] décide de ce que reçoit une requête qui ne correspond à aucune route :

- [[ui:emu.fallbackDefault]] — 404 avec le corps `{"error":"no_route"}` ;
- [[ui:emu.fallbackCustom]] — une réponse que vous définissez, avec tout ce qu’a la réponse
  d’une route. Son `{{counter}}` compte les requêtes qu’aucune route n’a prises.

Dans les deux cas, la requête compte comme [[ui:emu.unmatched]].

### Ce qu’une réponse HTTP peut lire {#http-request}

| Modèle | Valeur |
| --- | --- |
| `{{request.method}}` | `GET`, `POST`, … |
| `{{request.path}}` | Le chemin, sans la chaîne de requête. |
| `{{request.params.id}}` | Le segment de chemin nommé `:id`. |
| `{{request.query.page}}` | Un paramètre de la chaîne de requête, décodé. |
| `{{request.headers.x-key}}` | Un en-tête ; noms en minuscules. |
| `{{request.body}}` | Le corps en texte : ses 64 premiers Kio. |
| `{{request.json.name}}` | Un champ d’un corps JSON, quand le corps est du JSON et tient dans 64 Kio. |
| `{{request.from}}` | L’`IP:port` du client. |

Un corps de requête de plus de 1 Mio reçoit 413 et est compté comme [[ui:emu.failed]].
Une réponse qui ne peut pas être construite — un modèle qui nomme quelque chose que la requête n’a
pas — reçoit 500 avec l’erreur dans son corps, et est comptée comme
[[ui:emu.failed]].

## Appareil OSC {#osc}

Chaque message qui arrive — chaque message d’un bundle séparément — reçoit la réponse
de la première règle à laquelle il correspond. Un datagramme qui n’est pas de l’OSC est compté comme
[[ui:emu.unmatched]].

| Champ | Quoi |
| --- | --- |
| [[ui:emu.address]] | Un motif d’adresse OSC 1.0 : `*` n’importe quels caractères, `?` un seul, `[a-z]` un ensemble, `{a,b}` l’un ou l’autre, chacun à l’intérieur d’un segment (voir [OSC](../protocols/osc.md#patterns)). |
| [[ui:exp.argRules]] | Jusqu’à 16 conditions sur les arguments, comme dans [[ui:exp.node.wait_osc]] (voir [Nœuds](../experiments/nodes.md#node-wait_osc)). |
| [[ui:emu.replyOn]] | Désactivé : prendre le message sans rien répondre. |
| [[ui:emu.replyAddress]] | L’adresse de la réponse, un [modèle](#templates). |
| [[ui:emu.replyArgs]] | Jusqu’à 16 arguments, chacun avec un [[ui:emu.argType]] (`int`, `float`, `str`, `long`, `double`, `bool`, `blob`, `nil`) et une [[ui:emu.argValue]] sous forme de modèle. |
| [[ui:emu.to]] | Vide : retour à l’adresse et au port de l’expéditeur. Sinon `IP:port`. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 0–60 000 ms chacun. |

La valeur d’un argument est lue selon son type une fois le modèle rempli :
`{{request.args[0]}}` renvoie le premier argument comme nombre quand le type est
numérique. Un `bool` accepte `true`, `1`, `yes`, `on` ou `false`, `0`, `no`, `off` ;
un `blob` accepte des octets en hex ; une valeur vide est le zéro du type.

Les réponses partent du port propre à l’émulateur : un client qui écoute sur le
port depuis lequel il a envoyé les entend.

Une réponse OSC peut lire `{{request.address}}`, `{{request.args[0]}}` et
`{{request.from}}`.

## Appareil UDP {#udp}

Chaque datagramme reçoit la réponse de la première règle à laquelle il correspond.

| Champ | Quoi |
| --- | --- |
| [[ui:emu.match]] | [[ui:exp.mode.any]], [[ui:exp.mode.contains]], [[ui:exp.mode.regex]] ou [[ui:exp.mode.hex]]. |
| [[ui:emu.pattern]] | Le texte, l’expression régulière ou les octets à rechercher. |
| [[ui:emu.reply]] | [[ui:emu.replyOff]], [[ui:emu.replyText]] ou [[ui:emu.replyHex]], puis la réponse elle-même sous forme de [modèle](#templates). |
| [[ui:emu.to]] | Vide : retour à l’expéditeur. Sinon `IP:port`. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 0–60 000 ms chacun. |

Une réponse UDP ou TCP peut lire :

| Modèle | Valeur |
| --- | --- |
| `{{request.text}}` | La charge utile en texte. |
| `{{request.match}}` | Ce qui a correspondu : le texte, le premier groupe d’une expression régulière (ou toute la correspondance), les octets. |
| `{{request.hex}}` | La charge utile en octets hex, ses 1024 premiers. |
| `{{request.bytes}}` | La taille de la charge utile. |
| `{{request.from}}` | L’`IP:port` de l’expéditeur. |

Une réponse texte fait au plus 65 507 octets.

## Appareil TCP {#tcp}

Un appareil qui parle par lignes sur une connexion TCP, comme un projecteur ou une matrice de
commutation. Chaque message qu’envoie un client reçoit la réponse de la première règle à laquelle il
correspond ; la réponse revient sur la même connexion.

| Champ | Quoi |
| --- | --- |
| [[ui:emu.delimiter]] | Ce qui termine un message, et qui est ajouté après chaque réponse et après l’accueil : [[ui:emu.delimiter.lf]] (un `\r` qui le précède est supprimé), [[ui:emu.delimiter.crlf]], [[ui:emu.delimiter.cr]], ou [[ui:emu.delimiter.none]]. Les lignes vides sont ignorées. |
| [[ui:emu.greeting]] | Envoyé quand un client se connecte ; vide pour aucun. Il peut lire `{{request.from}}`. |
| [[ui:emu.match]], [[ui:emu.pattern]], [[ui:emu.reply]] | Comme pour un [appareil UDP](#udp). |
| [[ui:emu.close]] | Fermer la connexion après la réponse de cette règle — à `QUIT`, par exemple. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 0–60 000 ms chacun. |

Un message de plus de 64 Kio sans son délimiteur est pris tel quel.

## Broker MQTT {#mqtt}

Un petit broker MQTT 3.1.1 sur TCP simple. Il fait ce que fait un broker : les clients
se connectent, s’abonnent avec `+` et `#`, publient en QoS 0, 1 et 2, les messages
retenus et les messages de dernière volonté fonctionnent, et une seconde connexion avec l’identifiant d’un client prend
le relais de la première. Les sessions sont toujours propres : un client qui demande à garder sa
session en reçoit une neuve, et rien n’est mis en file pour un client absent.

En plus, chaque message qui lui est publié est confronté aux règles :
la première qui correspond publie aussi une réponse — un appareil qui rend compte de ce qu’il a fait.

| Champ | Quoi |
| --- | --- |
| [[ui:emu.username]], [[ui:emu.password]] | Quand un nom d’utilisateur est défini, un client doit se connecter avec celui-ci et le mot de passe ; vide : tout le monde peut se connecter. Un mot de passe sans nom d’utilisateur est refusé, car MQTT 3.1.1 ne peut pas le transporter. |
| [[ui:emu.retained]] | Jusqu’à 64 messages ([[ui:emu.topic]], [[ui:emu.payload]], [[ui:emu.qos]]) présents dès le démarrage, comme s’ils avaient été publiés avec retain : un client qui s’abonne les reçoit en premier. |
| [[ui:emu.topicFilter]] | Les topics que prend une règle : `+` un niveau, `#` le reste — `lab/+/set`. |
| [[ui:emu.match]], [[ui:emu.pattern]] | Une condition sur la charge utile, comme pour un [appareil UDP](#udp). |
| [[ui:emu.replyOn]] | Désactivé : prendre le message sans rien publier de plus. |
| [[ui:emu.replyTopic]], [[ui:emu.replyPayload]] | Des [modèles](#templates). Le topic ne peut pas contenir `+` ni `#`. |
| [[ui:emu.qos]], [[ui:emu.retain]] | Ceux de la réponse. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 0–60 000 ms chacun. |

Une réponse MQTT peut lire `{{request.topic}}`, `{{request.levels[1]}}` (les
niveaux du topic, à partir de 0), `{{request.payload}}`, `{{request.json.state}}`,
`{{request.match}}`, `{{request.qos}}`, `{{request.retain}}`,
`{{request.client}}` (l’identifiant du client) et `{{request.from}}`.

## Modèles dans les réponses {#templates}

Les réponses s’écrivent dans le même [langage de modèles](../experiments/data.md#templates)
que les expériences : un champ signifie donc la même chose ici et là. Une réponse peut
lire :

- `request` — ce qui est arrivé, comme listé plus haut pour chaque protocole ;
- `{{counter}}` — combien de messages cette règle a pris depuis le démarrage de l’émulateur,
  celui-ci compris ;
- les [générateurs](../experiments/data.md#generators) — `{{uuid}}`,
  `{{now.iso}}`, les valeurs aléatoires et les autres ; les aléatoires sont tirés de la
  graine de l’émulateur ;
- les paramètres, quand l’émulateur tourne dans une expérience ou est démarré avec
  `signallab emulate --param`.

Une réponse ne lit jamais de secrets, et un nom inconnu est une erreur, pas un texte
vide.

Certains champs sont fixés au démarrage de l’émulateur, avant que quoi que ce soit n’arrive : un
chemin, une condition, un motif d’adresse, un motif de charge utile, un filtre de topic,
[[ui:emu.to]], le nom d’un en-tête, les messages retenus et les identifiants de connexion du broker. Ils
n’acceptent que du texte et des paramètres, ni `request` ni générateurs.

La graine pilote l’ordre aléatoire des réponses, la gigue et les générateurs
aléatoires. Sur l’écran [[ui:nav.emulators]], chaque démarrage prend une nouvelle graine ; une
expérience utilise la graine de l’exécution, et `signallab emulate --seed` prend celle que vous
donnez.

## Tomber en panne {#outage}

Pour tester ce que fait votre système quand une dépendance flanche par intermittence, cochez
[[ui:emu.outage]] :

| Champ | Quoi | Par défaut |
| --- | --- | --- |
| [[ui:emu.outageUp]] | Combien de temps il répond, 10–3 600 000 ms. | 10 000 |
| [[ui:emu.outageDown]] | Combien de temps il est en panne, 10–3 600 000 ms. | 3000 |
| [[ui:emu.outageFault]] | HTTP uniquement : ce que rencontre une requête pendant la panne. | [[ui:emu.outageFault.unavailable]] |

Le calendrier démarre avec l’émulateur et se répète : en service, en panne, en service,
en panne… Pendant la panne :

| Émulateur | Ce qui se passe |
| --- | --- |
| HTTP | [[ui:emu.outageFault.unavailable]] : 503 avec `Retry-After` réglé sur le nombre de secondes avant son retour (au moins 1). [[ui:emu.outageFault.reset]] : la connexion se ferme sans réponse. [[ui:emu.outageFault.timeout]] : retenue jusqu’à 2 minutes, puis fermée. |
| Appareil TCP | Les connexions ouvertes sont coupées en moins de 0,1 s ; les nouvelles sont fermées dès leur arrivée. |
| Broker MQTT | Toutes les connexions sont coupées ; les nouvelles sont refusées (code de retour CONNACK 3, serveur indisponible). |
| Appareil OSC, UDP | Rien ne reçoit de réponse. |

Ce qui arrive pendant la panne compte comme [[ui:emu.down]], pas comme
[[ui:emu.unmatched]], et ses règles ne sont pas consultées.

Le bouton [[ui:emu.takeDown]] fait la même chose à la demande, quoi que dise le calendrier, jusqu’à ce que
vous appuyiez sur [[ui:emu.bringUp]] ; HTTP reçoit alors 503 sans `Retry-After`. Dans une
expérience, le nœud [[ui:exp.node.emulator_state]] le fait à une étape de
l’exécution (voir [Nœuds](../experiments/nodes.md#node-emulator_state) et
[Défaillances](../experiments/faults.md)).

## Problèmes {#problems}

Pendant que vous le modifiez, l’émulateur est vérifié un instant après chaque changement, et un
problème s’affiche sous ses boutons avant que vous n’appuyiez sur [[ui:emu.start]]. Un problème
indique où il se trouve — la règle, la réponse ou le message retenu, et le
champ — et ce qui ne va pas : un chemin sans son `/`, une expression régulière qui
ne compile pas, un modèle de réponse qui nomme autre chose que `request`,
les paramètres et les générateurs, une valeur hors limites. Le bouton [[ui:emu.start]] refuse un émulateur qui a un problème.

## Limites {#limits}

| Quoi | Limite | À la limite |
| --- | --- | --- |
| Routes ou règles par émulateur | 64 | Refusé à la vérification. |
| Réponses par route | 16 | Refusé. |
| Conditions par route | 16 | Refusé. |
| En-têtes par réponse | 32 | Refusé. |
| Conditions sur les arguments, arguments de réponse (OSC) | 16 chacun | Refusé. |
| Messages retenus (MQTT) | 64 | Refusé. |
| Un corps, une réponse ou un accueil tel qu’écrit | 256 Kio | Refusé. |
| Un délai ou une gigue | 60 000 ms | Refusé. |
| Corps de requête HTTP | 1 Mio | 413. |
| Connexions HTTP simultanées | 512 | Les suivantes sont fermées dès leur arrivée. |
| En-tête de requête HTTP | 30 s | Un client doit l’envoyer dans ce délai. |
| Connexions TCP simultanées | 256 | Les suivantes sont fermées dès leur arrivée. |
| Réponses OSC et UDP en attente de leur délai | 1024 | Les suivantes sont abandonnées et comptées comme [[ui:emu.failed]]. |
| Clients MQTT simultanés | 256 | Les suivants sont fermés dès leur arrivée. |
| Paquet MQTT | 256 Kio | La connexion du client prend fin. |
| Abonnements MQTT par client | 100 | Les suivants sont refusés. |
| Topics MQTT retenus | 1000 topics, 16 Mio | Un nouveau message retenu est acheminé, mais pas retenu. |
| Messages MQTT en attente pour un client lent | 1024 messages, 8 Mio | Il les manque ; comptés comme [[ui:emu.missed]]. |

## Émuler ceci {#mock-this}

Pour créer un émulateur à partir d’une réponse qui a fonctionné :

1. Sur l’écran [[ui:nav.http]], envoyez une requête et obtenez une réponse — ou utilisez le bouton
   [[ui:exp.sendNow]] sur un nœud HTTP d’une expérience.
2. Appuyez sur ⧉ [[ui:http.mockThis]] à côté de la réponse. La boîte de dialogue
   [[ui:http.mockTitle]] montre la route qu’elle va créer.
3. Dans le champ [[ui:http.mockInto]], choisissez l’un de vos émulateurs HTTP, ou
   [[ui:http.mockNew]].
4. Appuyez sur [[ui:http.mockAdd]]. L’écran [[ui:nav.emulators]] s’ouvre sur cet
   émulateur.

La route répond à la méthode et au chemin de la requête (sans la chaîne de requête) avec le
statut, les en-têtes et le corps de la réponse. Les en-têtes propres à cet échange-là
(`Content-Length`, `Date`, `Server`, `ETag` et autres) sont omis, et le
corps est envoyé tel quel, même s’il contient `{{`. Un nouvel émulateur ne contient que cette
route. Ajoutée à un émulateur existant, la route passe en premier, pour répondre
avant une route plus large ; un émulateur en cours la prend en compte quand vous appuyez sur
[[ui:emu.restart]].

Depuis une expérience, une URL écrite avec des modèles devient un motif : sa base
(`{{api}}`) est supprimée, un segment qui est un seul modèle (`/orders/{{order_id}}`)
devient `:order_id`, et un segment modélisé seulement en partie termine le chemin par
`*`.

## Le jeu de départ {#starter-set}

La première fois que Signal Lab ne trouve pas de bibliothèque d’émulateurs, il en écrit cinq, tous sur
cet ordinateur. Leurs noms et leurs notes sont écrits dans la langue de
l’interface à ce moment-là.

| Émulateur | Écoute sur | Fait |
| --- | --- | --- |
| [[ui:seed.emu.demo-api.name]] | `127.0.0.1:8080` | `GET /health` → `{"status":"ok","time":…}` ; `GET /users/:id` → un utilisateur avec cet identifiant ; `POST /users` → 201 avec un `Location` ; `GET /slow` → au bout de 1500 ms ; `/flaky` → 503, 503, puis 200 à partir de là. |
| [[ui:seed.emu.osc-device.name]] | `127.0.0.1:9100` | `/ping` → `/pong` avec le compteur ; `/fader/*` → `/ack` avec l’adresse reçue ; `/cue/*` pris sans réponse. |
| [[ui:seed.emu.udp-device.name]] | `127.0.0.1:7100` | `PING` → `PONG` et le compteur ; tout le reste → `ACK` et sa taille en octets. |
| [[ui:seed.emu.tcp-device.name]] | `127.0.0.1:7200` | Lignes terminées par CR LF. Accueille avec `READY` ; `POWER?` → `POWER=ON` ; `POWER ON` ou `POWER OFF` → `OK ON` / `OK OFF` ; `QUIT` → `BYE`, puis raccroche. |
| [[ui:seed.emu.mqtt-broker.name]] | `127.0.0.1:1883` | Retient `online` sur `lab/status` ; `ON` ou `OFF` publié sur `lab/<name>/set` → la même chose, retenue, sur `lab/<name>/state`. |

Le signal de départ [[ui:seed.http-reachable.name]] de la
[bibliothèque de signaux](signals.md#starter-set) interroge `http://127.0.0.1:8080/`, l’adresse de
l’émulateur [[ui:seed.emu.demo-api.name]] : celui-ci n’a pas de route pour `/`, il reçoit donc
404.

## Le fichier de la bibliothèque {#file}

La bibliothèque est `emulators.json` dans le dossier de données (voir
[Fichiers](../reference/files.md)) ; survolez le compteur sous la liste pour voir son
chemin. Il est écrit en entier 0,7 s après la dernière modification, via un fichier
temporaire : une écriture ratée laisse donc le précédent intact. Si le fichier ne peut pas être lu,
la liste affiche l’erreur avec le chemin, la ligne et la colonne, et le fichier est laissé
tel quel : corrigez-le et appuyez sur [[ui:emu.reload]]. Appuyez aussi sur [[ui:emu.reload]] après
l’avoir modifié à la main. Sans fichier, le jeu de départ est réécrit.

```json
{
  "version": 1,
  "emulators": [
    {
      "id": "orders-api",
      "note": "Stands in for the orders service.",
      "emulator": {
        "name": "Orders API",
        "bind": "127.0.0.1:18080",
        "protocol": "http",
        "routes": [
          { "method": "GET", "path": "/orders/:id",
            "responses": [{ "body": "{\"id\":\"{{request.params.id}}\",\"state\":\"open\"}" }] },
          { "method": "POST", "path": "/orders", "order": "sequence",
            "responses": [{ "status": 503 }, { "status": 201, "body": "{\"id\":\"{{uuid}}\"}" }] }
        ],
        "outage": { "up_ms": 20000, "down_ms": 2000, "fault": "unavailable" }
      }
    }
  ]
}
```

L’objet `emulator` seul est un document que `signallab emulate` lit aussi.

## Dans les expériences et les scripts {#elsewhere}

- Dans une expérience, un nœud [[ui:exp.node.emulator]] ouvre son émulateur avant
  la première étape et répond jusqu’à la fin de l’exécution ; ce qu’il a reçu est compté
  dans le rapport. Un émulateur HTTP y est aussi ce qu’écoute
  [[ui:exp.node.wait_http]] ([Nœuds](../experiments/nodes.md#node-wait_http)),
  et un émulateur OSC ou UDP partage son
  port avec les attentes de l’exécution. Deux émulateurs d’un même transport dans une même expérience
  ne peuvent pas partager un port. Voir [Nœuds](../experiments/nodes.md#node-emulator) et
  [Défaillances](../experiments/faults.md).
- `signallab emulate` exécute des émulateurs depuis des fichiers ou depuis cette bibliothèque jusqu’à
  <kbd>Ctrl</kbd>+<kbd>C</kbd> ou `--for`, en affichant ce qu’ils répondent ; voir
  [La ligne de commande](../automation/cli.md#cli-emulate).

## Voir aussi {#related}

- [Inspecteur](inspector.md) — chaque échange, décodé.
- [Dégradation](impairment.md) — un mauvais réseau entre votre système et un
  émulateur.
- [Données et modèles](../experiments/data.md#templates)

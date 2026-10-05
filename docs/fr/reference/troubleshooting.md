---
title: "Dépannage"
description: "Problèmes courants avec Signal Lab et comment les résoudre — rien n’arrive, un port déjà utilisé, le pare-feu, la mise en réseau Docker, le serveur, MQTT, les certificats, les mises à jour et les journaux."
---

# Dépannage

Chaque échec signalé par Signal Lab a un code ; le message de chacun est dans
[messages d’erreur](errors.md). Les échecs réseau sont les codes
[`transport`](errors.md#transport) : `refused`, `timeout`, `dns`,
`unreachable`, `reset`, `address_in_use`, `address_unavailable`, `denied`,
`tls`, `target_invalid`, `failed`. Les problèmes ci-dessous sont les plus courants.

## Rien n’arrive {#nothing-arrives}

Déterminez d’abord si quelque chose atteint Signal Lab : ouvrez l’onglet
[[ui:dock.inspector]] du panneau inférieur et appuyez sur [[ui:ins.arm]]. Chaque
datagramme, requête et message qu’un outil envoie ou reçoit y est listé, avec son
origine.

### L’adresse d’écoute {#listen-address}

- Un [[ui:common.bind]] de `0.0.0.0:<port>` écoute sur chaque carte réseau ;
  `127.0.0.1:<port>` n’entend que cette machine. Le matériel sur le réseau a besoin
  du premier.
- L’appareil doit envoyer vers l’adresse de cette machine et vers le port sur
  lequel vous écoutez. L’en-tête indique le nom et l’adresse de cette machine.
- Une adresse qui n’est pas celle de cette machine échoue avec `address_unavailable`.

### Le pare-feu {#firewall}

Le trafic sur `127.0.0.1` n’est jamais filtré, c’est pourquoi un test sur une
machine fonctionne alors que le même test depuis une autre machine ne reçoit rien.

**Windows.** Le Pare-feu Windows décide par programme. Windows demande généralement
une fois, la première fois qu’un programme écoute — et un *Annuler* à ce
moment-là laisse une règle qui le bloque, laquelle l’emporte sur toute règle
d’autorisation. Sur un réseau que Windows qualifie de public (souvent le Wi-Fi
d’une salle), il peut ne pas demander du tout.

- L’application de bureau consulte le pare-feu une fois, lorsqu’un moniteur, un
  écouteur de découverte, un relais, une exécution ou un émulateur commence à
  écouter. Lorsque le pare-feu fait obstacle, elle le signale dans un avertissement
  avec [[ui:fw.allow]] — ou [[ui:fw.allowPublic]] sur un réseau public. Windows
  demande les droits d’administrateur, puis les règles entrantes du programme, une
  règle de blocage incluse, sont remplacées par une seule règle d’autorisation.
  [[ui:fw.dismiss]] masque l’avertissement.
- Depuis un terminal : `signallab doctor` montre ce qui fait obstacle, et
  `signallab firewall allow` le règle (`--public` aussi pour les réseaux publics),
  avec la même invite d’administrateur.
- Une installation (`.exe`) faite *pour tout le monde* ajoute elle-même les règles
  d’autorisation (réseaux privés et de domaine), sauf si elle a été lancée avec
  `/NOFIREWALL`. Une installation *pour moi* ne le peut pas, et le `.msi` laisse le
  pare-feu à qui le déploie.
- Un serveur ne modifie jamais le pare-feu de son hôte : son administrateur ouvre
  les ports (le script d’installation propose de le faire, avec ufw ou firewalld).

**Linux.** Un pare-feu tel que ufw ou firewalld fonctionne par port, pas par
programme. `signallab doctor` nomme celui qui est actif et comment ouvrir un port,
par exemple `sudo ufw allow 9000/udp`.

### Diffusion et multidiffusion {#broadcast-multicast}

- Les routeurs ne transmettent pas la diffusion : `255.255.255.255` et
  `x.x.x.255` n’atteignent que le segment réseau où se trouve la carte émettrice.
  Avec plusieurs cartes, mettez l’adresse de la carte dans [[ui:bc.bindSource]]
  (sous [[ui:bc.socketOptions]]), par exemple `10.0.0.5:0`.
- Un datagramme de multidiffusion n’atteint que les écouteurs qui ont rejoint son
  groupe — sur l’écouteur de découverte, [[ui:bc.joinGroups]]. Avec [[ui:bc.ttl]] à
  1, la valeur par défaut, il reste sur ce réseau.
- Pour entendre votre propre multidiffusion sur la même machine, laissez
  [[ui:bc.mcastLoop]] activé.

### Un appareil qui ne répond pas {#no-answer}

UDP n’a pas d’accusé de réception : un datagramme envoyé à un port où personne
n’écoute compte quand même comme envoyé. Windows rapporte alors l’ICMP *port
unreachable* reçu comme une réinitialisation de connexion à la prochaine réception
de ce socket ; les moniteurs, les écouteurs, les relais et les attentes de Signal
Lab l’ignorent et continuent d’écouter. Donc quand une réponse n’arrive pas, une
attente échoue avec `wait.timeout` après son délai, et non avec une erreur sur
l’envoi. Vérifiez dans [[ui:dock.inspector]] que le message est parti vers la bonne
adresse, puis vérifiez l’appareil.

## Un envoi est refusé avant de partir {#refused-send}

Ils sont vérifiés d’abord, et rien n’est envoyé quand l’un échoue :

| Code | Pourquoi | Correction |
| --- | --- | --- |
| `transport.target_invalid` | La destination n’a pas de port, ou n’est ni `IP:port` ni `host:port` | Écrivez les deux, par exemple `192.0.2.20:9000` |
| `transport.dns` | Le nom d’hôte ne se résout pas sur cette machine | Vérifiez le nom, ou utilisez l’adresse. Un nom avec une adresse IPv4 est atteint via IPv4, donc `localhost:9000` trouve un récepteur sur `127.0.0.1` |
| `node.osc_address` | Une adresse OSC ne commence pas par `/` — dans l’expéditeur, le générateur, un signal ou une étape | Faites-la commencer par `/`, par exemple `/cue/go` |
| `node.topic_wildcard` | Le topic d’une publication MQTT contient un `+` ou un `#`, sur la connexion de l’écran comme partout ailleurs | Publiez vers un seul topic ; les jokers servent à s’abonner |
| `node.too_long` sur un message WebSocket | Le message dépasse 16 Mio | Envoyez moins ; la connexion reste ouverte |

## Un port est déjà utilisé {#port-in-use}

`transport.address_in_use` : un autre programme — ou une autre tâche de Signal Lab —
écoute déjà sur ce port.

- **Un moniteur et une exécution.** Une exécution ouvre les ports de ses attentes,
  de ses émulateurs et de ses relais avant sa première étape, donc un port détenu
  par un [[ui:osc.monitor]], un écouteur de découverte ou une tâche d’émulateur fait
  échouer l’exécution avant qu’elle ne démarre. Arrêtez d’abord cette tâche ;
  [[ui:app.stopAll]] les arrête toutes.
- **Deux émulateurs** dans une exécution ne peuvent pas partager un port d’un même
  transport : les émulateurs HTTP, MQTT et TCP écoutent tous sur TCP, ceux OSC et
  UDP sur UDP (`emulator.bind_taken`).
- **Écouter à côté du vrai service.** L’écouteur de découverte peut partager un
  port avec un programme qui le détient déjà : laissez [[ui:bc.reuse]] activé. Sur
  Linux, ce programme doit aussi partager son port. Sans cela, un port pris donne
  `broadcast.port_shared`.
- **Vient de s’arrêter.** Un port détenu par un émulateur ou une exécution arrêtés
  est libéré un instant plus tard ; un émulateur redémarré aussitôt l’attend
  brièvement.
- **Les ports inférieurs à 1024** sur Linux demandent les droits d’administrateur
  (`denied`). L’image du serveur tourne sans aucun droit, utilisez donc un port de
  1024 ou plus.

## Le serveur dans Docker n’atteint pas le réseau {#docker-network}

La diffusion, la multidiffusion et la découverte n’atteignent le réseau physique
qu’avec la mise en réseau de l’hôte — `network_mode: host` dans le fichier compose,
ou `docker run --network host` — et seulement sur un hôte Linux. Elle permet aussi
aux moniteurs, aux attentes et aux émulateurs d’écouter sur les ports propres à
l’hôte. Avec le réseau bridge par défaut de Docker, le conteneur est sur un réseau
à lui : la diffusion et la multidiffusion n’en sortent jamais, et seuls les ports
que vous publiez l’atteignent.

Sous Windows et macOS, la mise en réseau de l’hôte de Docker n’atteint pas le
réseau physique : utilisez l’application de bureau sous Windows, ou exécutez le
serveur sur un hôte Linux.

## SmartScreen avertit à propos de l’installateur {#smartscreen}

Les installateurs ne sont pas encore signés, donc Windows SmartScreen dit qu’il ne
connaît pas l’éditeur. Choisissez *Plus d’informations*, puis *Exécuter quand même*.
Ne téléchargez les installateurs que depuis les versions du projet sur GitHub.

## Le serveur ne démarre pas {#server-start}

`signal-lab-server` vérifie ses réglages avant d’écouter et sort avec un message
sur sa sortie d’erreur :

| Code de sortie | Message | Correction |
| --- | --- | --- |
| 2 | refusing to listen on … without a token | Un serveur que d’autres peuvent atteindre a besoin d’un jeton : `--token-file` ou `SIGNALLAB_TOKEN` (fabriquez-en un avec `signal-lab-server token`), ou `--generate-token`. Ou écoutez sur `127.0.0.1` |
| 2 | the token has N characters; it needs at least 24 | Utilisez un jeton plus long |
| 2 | the token must not contain spaces or line breaks | Un fichier de jeton peut se terminer par un saut de ligne ; rien d’autre |
| 2 | give the token once | Utilisez `--token`/`SIGNALLAB_TOKEN` ou `--token-file`/`SIGNALLAB_TOKEN_FILE`, pas les deux |
| 2 | --generate-token keeps the token in the data folder | Définissez `--data-dir` ou `SIGNALLAB_DATA_DIR` |
| 2 | … it does not hold a valid token; remove it to have a new one made | Le fichier `token` du dossier de données est endommagé |
| 2 | cannot read the token file … | Le fichier nommé par `--token-file` manque ou n’est pas lisible par cet utilisateur |
| 2 | cannot save the new token in … | `--generate-token` n’a pas pu écrire `token` dans le dossier de données : rendez le dossier accessible en écriture à cet utilisateur |
| 1 | cannot listen on … | L’adresse n’est pas celle de cette machine, ou le port est pris |
| 1 | the data folder … must be writable by this user | Dans Docker, un dossier monté doit être accessible en écriture à l’uid 10001 |

Un jeton fabriqué par `--generate-token` est affiché une fois, au premier démarrage
(`docker logs signallab` le montre), et conservé dans `token` dans le dossier de
données : `docker exec signallab cat /data/token`. Voir [le serveur](../server/index.md).

## Impossible de se connecter au serveur {#sign-in}

| Ce que vous voyez | Pourquoi | Correction |
| --- | --- | --- |
| *That token is not right.* | Un mauvais jeton | Recopiez-le depuis l’endroit où le serveur le garde ; la réponse prend une seconde exprès |
| `auth.host` | Le serveur ne répond pas au nom de la barre d’adresse | Ouvrez-le par un nom qu’il accepte. Les noms de boucle locale (`localhost`, `127.x.x.x`, `[::1]`) passent toujours. Sans jeton, le serveur ne répond qu’à ceux-là et aux noms de `--allowed-host` ; avec un jeton, à tout nom sauf si `--allowed-host` le restreint |
| `auth.origin` | Une requête venait d’une page d’une autre origine | Derrière un proxy inverse, transmettez le `Host` du navigateur au serveur (nginx : `proxy_set_header Host $host;`), pour qu’`Origin` et `Host` s’accordent |
| De retour à la page de connexion après s’être connecté | Le navigateur n’a pas conservé le cookie de session | Avec `--secure-cookie`, le serveur doit être atteint via HTTPS |
| Déconnecté après un moment | Les sessions durent 7 jours et se terminent au redémarrage du serveur ; au-delà de 1024 sessions, la plus ancienne part | Reconnectez-vous |

## La connexion au serveur ne cesse de tomber {#connection-lost}

[[ui:app.connectionLost]] signifie que le socket d’événements de la page,
`/api/events`, s’est fermé. La page se reconnecte d’elle-même, après une
demi-seconde puis de moins en moins souvent, jusqu’à toutes les 15 s. Ce qui s’est
passé entre-temps n’est pas rejoué : les tâches en cours rapportent de nouveau au
fur et à mesure. Derrière un proxy inverse, assurez-vous qu’il transmet les mises
à niveau WebSocket pour `/api/events` et ne ferme pas les connexions silencieuses
en moins de 20 s (le serveur envoie un ping toutes les 20 s). Une page qui dit que
le serveur n’a pas cette adresse (`api.not_found`) est plus ancienne que le
serveur : rechargez-la.

## MQTT ne se connecte pas {#mqtt}

Signal Lab parle MQTT 3.1.1 sur TCP en clair. Il se connecte et attend la réponse
du broker (CONNACK) avant de signaler le succès, donc la raison est sur le bouton
qui connecte :

| Code | Pourquoi | Correction |
| --- | --- | --- |
| `transport.refused` | Rien n’écoute sur ce port | Vérifiez le port : 1883 est le port habituel |
| `transport.timeout` | Aucune connexion TCP en 6 s | Vérifiez l’adresse, le réseau, le pare-feu du broker |
| `transport.dns` | Le nom d’hôte ne se résout pas | Vérifiez le nom, ou utilisez l’adresse |
| `transport.unreachable` | Aucune route vers le broker | Vérifiez le réseau et l’adresse |
| `transport.reset` | Le broker a fermé la connexion aussitôt | Souvent un port TLS (8883) — Signal Lab ne parle pas MQTT sur TLS |
| `mqtt.no_answer` | Le port est ouvert, mais aucun CONNACK n’est venu en 6 s | Souvent un port WebSocket — Signal Lab ne parle pas MQTT sur WebSocket |
| `mqtt.protocol` | Ce qui a répondu n’est pas un broker MQTT | Vérifiez le port |
| `mqtt.refused_protocol` | Le broker n’accepte pas MQTT 3.1.1 | Activez 3.1.1 sur le broker |
| `mqtt.refused_client_id` | Le broker refuse l’id client | Utilisez un autre id client |
| `mqtt.refused_unavailable` | Le broker est indisponible | Réessayez plus tard |
| `mqtt.refused_credentials` | Le nom d’utilisateur ou le mot de passe est faux | Vérifiez-les |
| `mqtt.refused_not_authorized` | L’utilisateur n’est pas autorisé à se connecter | Vérifiez les règles d’accès du broker |
| `mqtt.client_id_required` | L’id client est vide | Remplissez-le |

Un broker abandonne la plus ancienne de deux connexions avec le même id client :
quand une connexion ne cesse de se fermer, cherchez un autre client utilisant le
même id.

## Un certificat n’est pas approuvé {#tls}

`transport.tls`, sur `https://` ou `wss://` : le certificat du serveur n’est pas
approuvé, ne nomme pas l’hôte que vous avez demandé, ou TLS n’a pas pu être
négocié. Signal Lab vérifie les certificats comme le fait le système et n’a aucun
interrupteur pour ignorer la vérification. HTTPS et WSS approuvent les mêmes
certificats : ceux du système d’exploitation où le moteur tourne — le magasin de
certificats Windows sous Windows, les certificats d’autorité du système sous Linux
et dans l’image du serveur. Pour un certificat auto-signé ou votre propre autorité,
ajoutez-le aux certificats approuvés de cette machine (pour l’image du serveur, une
image construite dessus qui ajoute le certificat), et connectez-vous par le nom que
porte le certificat.

## Un secret manque {#secrets}

`secret.missing` : une expérience utilise `{{secret.NAME}}` et aucune valeur n’est
stockée sous ce nom là où elle tourne.

- **Application de bureau sous Windows** : définissez-le sous [[ui:exp.secrets]]
  dans [[ui:exp.params]] de l’éditeur. Il est conservé dans le Gestionnaire
  d’informations d’identification Windows, donc une nouvelle machine a besoin qu’il
  soit redéfini.
- **Serveur** : mettez la valeur dans le fichier `/run/secrets/signallab/NAME` (ou
  le dossier nommé par `--secrets-dir`) ou dans la variable d’environnement
  `SIGNALLAB_SECRET_NAME`. Un serveur ne peut pas définir de secrets depuis la page
  (`secret.read_only`).
- **Application de bureau sous Linux** n’a pas de magasin pour les secrets
  (`secret.unsupported`). Exécutez une telle expérience avec `signallab run`, qui
  lit les secrets dans des fichiers et des variables, ou sur un serveur.

Voir [fichiers](files.md#secrets).

## Une exécution s’arrête après cinq minutes {#run-timeout}

Une exécution qui dure plus de 300 s échoue avec `run.timeout` ; c’est la durée
maximale d’une exécution. Une limite plus courte peut être donnée à une exécution
via l’API (`timeout` de [`/api/run`](../api/run.md#request)) ou la ligne de
commande (`signallab run --timeout`).

## L’application ne se met pas à jour {#updates}

- L’application de bureau cherche une nouvelle version une fois par jour tant
  qu’[[ui:update.auto]] est activé, et lorsque vous appuyez sur
  [[ui:update.check]] dans [[ui:about.open]].
- Elle interroge d’abord le hub du studio et GitHub lorsque le hub est injoignable.
  Quand un réseau bloque les deux, [[ui:update.check]] donne
  [[ui:update.checkFailed]] ; la recherche quotidienne échoue sans un mot.
- Seules les versions publiées lui sont proposées, jamais les brouillons ni les
  préversions.
- Une nouvelle version atteint l’application quand le hub la propose, ce qui peut
  arriver un moment après son apparition sur GitHub : le hub déploie une version
  auprès d’une partie des installations à la fois.
- Elle ne s’installe que lorsque vous appuyez sur [[ui:update.install]] — les
  tâches en cours sont arrêtées d’abord — et seulement si la signature de la
  version est valide : [[ui:update.installFailed]] sinon.
- Un serveur se met à jour avec son image : `docker compose pull && docker compose up -d`
  dans le dossier de son fichier compose.

## Où sont les journaux {#logs}

- **Application de bureau** : elle n’écrit aucun fichier journal. L’onglet
  [[ui:console.title]] du panneau inférieur liste ce que chaque outil a fait et ce
  qui a mal tourné, et [[ui:feedback.open]] le joint à un message aux développeurs
  (sans le nom de cet ordinateur, son adresse ni vos dossiers). Le rapport de chaque
  exécution est dans `runs/` dans le dossier de données.
- **Serveur** : il journalise sur sa sortie standard et sa sortie d’erreur —
  `docker logs signallab` dans Docker. Chaque démarrage de tâche est journalisé avec
  l’adresse du client qui l’a demandé. `--log` (ou `SIGNALLAB_LOG`) définit le
  niveau : `error`, `warn`, `info` (par défaut) ou `debug` ; `--log-format json` (ou
  `SIGNALLAB_LOG_FORMAT`) écrit un objet JSON par ligne.
- **Ligne de commande** : `signallab` écrit ses messages sur sa sortie d’erreur ;
  voir [la ligne de commande](../automation/cli.md).

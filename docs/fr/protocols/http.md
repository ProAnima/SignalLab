---
title: HTTP
description: "Envoyez une requête HTTP et lisez toute la réponse, authentifiez-vous avec Basic, Bearer ou Digest, conservez les cookies, et mesurez un point de terminaison sous une rafale de charge simultanée."
---

# HTTP

L’écran [[ui:nav.http]] est à la fois un inspecteur de requêtes et un outil de charge :

- envoyer une seule requête et voir le statut, le temps qu’elle a pris, les en-têtes et
  le corps ;
- s’authentifier avec Basic, un jeton Bearer ou Digest ;
- conserver les cookies qu’un serveur définit, comme le fait un navigateur ;
- envoyer la même requête de nombreuses fois en même temps — une [[ui:http.burst]] — et lire
  le débit et les percentiles de latence.

## Envoyer une requête {#request}

1. Ouvrez [[ui:nav.http]].
2. Choisissez la méthode et saisissez l’URL, par exemple `http://127.0.0.1:8080/health`.
3. Ajoutez des [[ui:http.headers]] si le serveur en a besoin ; [[ui:http.addHeader]] ajoute
   une ligne, ✕ en supprime une. Une ligne sans nom n’est pas envoyée.
4. Pour une méthode autre que GET et HEAD, écrivez le [[ui:http.body]]. Le texte reste dans le champ pendant que vous passez à GET ou HEAD et revient avec l’autre méthode, mais n’est pas envoyé entre-temps.
5. Appuyez sur [[ui:common.send]].

La ligne sous les boutons donne le verdict aussitôt — statut, temps et taille,
ou pourquoi il n’y a pas eu de réponse — et le panneau [[ui:http.response]] montre le
reste. La requête (méthode, URL, en-têtes, corps, délai d’attente et
[[ui:http.keepCookies]]) est conservée quand vous changez d’écran et quand vous redémarrez
l’application ; les identifiants ne le sont pas.

| Touche | Où | Fait |
| --- | --- | --- |
| <kbd>Enter</kbd> | l’URL, un en-tête, un identifiant, le délai d’attente | envoie |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | n’importe quel champ de la requête, le corps compris | envoie |
| <kbd>Ctrl</kbd>+<kbd>S</kbd> | n’importe quel champ de la requête | l’enregistre comme signal ([ci-dessous](#library)) |

### Champs de la requête {#request-fields}

| Champ | Quoi | Par défaut |
| --- | --- | --- |
| [[ui:exp.method]] | GET, POST, PUT, PATCH, DELETE, HEAD ou OPTIONS | GET |
| [[ui:field.url]] | Une URL `http://` ou `https://` | `http://127.0.0.1:8080/` |
| [[ui:http.headers]] | Des paires nom et valeur, envoyées telles quelles. Sans `User-Agent` à vous, Signal Lab envoie `SignalLab/0.1`. | `Accept: application/json` |
| [[ui:field.auth]] | Comment la requête s’authentifie ([ci-dessous](#auth)) | [[ui:http.auth.none]] |
| [[ui:http.keepCookies]] | Renvoyer les cookies que les serveurs définissent ([ci-dessous](#cookies)) | activé |
| [[ui:http.body]] | Envoyé exactement tel qu’écrit ; aucun `Content-Type` n’est ajouté, donc ajoutez l’en-tête qui correspond. Un corps vide n’est pas envoyé. Le champ n’est pas affiché pour GET et HEAD, et la requête ne porte alors aucun corps — ni dans l’envoi, ni dans un signal enregistré, ni dans [[ui:common.toExperiment]] — même si vous en avez saisi un sous une autre méthode. | vide |
| [[ui:common.timeoutMs]] | Combien de temps l’échange entier peut prendre, réponse et corps compris | 10 000 |

## Authentification {#auth}

| [[ui:field.auth]] | Champs | Ce qui est envoyé |
| --- | --- | --- |
| [[ui:http.auth.none]] | — | aucun en-tête `Authorization` |
| [[ui:http.auth.basic]] | [[ui:field.username]], [[ui:field.password]] | `Authorization: Basic …`, le nom et le mot de passe en base64, avec la première requête |
| [[ui:http.auth.bearer]] | [[ui:field.token]] | `Authorization: Bearer <token>` |
| [[ui:http.auth.digest]] | [[ui:field.username]], [[ui:field.password]] | rien au début ; la réponse au challenge du serveur ([ci-dessous](#digest)) |

Passer de Basic à Digest conserve le nom et le mot de passe.

### Digest {#digest}

Avec Digest, Signal Lab envoie la requête sans identifiants. Quand le serveur
répond `401` avec un challenge Digest, Signal Lab calcule la réponse à partir du
challenge et de votre mot de passe et renvoie la requête. La réponse que vous voyez
est celle de cette seconde requête, marquée [[ui:http.digestAnswered]], et la
latence compte les deux échanges — ce qu’un client attend.

- Algorithmes : MD5 et SHA-256, et leurs variantes `-sess`. Quand un serveur propose
  les deux, SHA-256 est utilisé.
- Qualité de protection : `auth` et `auth-int`, et l’ancienne réponse sans
  `qop`.
- Quand le serveur dit que le nonce est épuisé (`stale`), ou en demande un nouveau,
  la requête est de nouveau répondue, jusqu’à 3 fois de plus. Quand il refuse la
  réponse au nonce qu’il a donné en dernier, le `401` tient : le nom ou le mot de passe
  est faux.
- Les redirections sont suivies par Signal Lab lui-même, si bien que l’URL qui demande est celle à laquelle
  il est répondu. Un challenge d’une autre origine n’est pas répondu : des identifiants saisis
  pour un hôte ne vont à aucun autre. Passer de `http://` à `https://` sur le même
  hôte et les ports par défaut compte comme le même hôte.

Quand le challenge ne peut pas être répondu, le `401` tient et le panneau indique pourquoi :

| Message | Signification |
| --- | --- |
| Le serveur a répondu 401 sans demander Digest | Le serveur veut un autre schéma ; essayez Basic ou Bearer. |
| Le serveur a demandé Digest avec … | Un algorithme que Signal Lab ne parle pas ; il parle MD5 et SHA-256. |
| Le serveur a demandé Digest sans realm ni nonce | Le challenge du serveur est incomplet. |
| La requête a été transmise à …, qui a demandé Digest | Une redirection a mené à une autre origine, dont le challenge n’est pas répondu. |

### Où vont les identifiants {#credentials}

Les identifiants ne vont que dans l’en-tête `Authorization` de la requête, au moment où elle est
envoyée. L’Inspecteur, la console et les rapports d’expérience ne montrent jamais cet en-tête. Sur
cet écran, ils ne sont conservés qu’en mémoire et disparaissent après un redémarrage — sauf
si la requête est liée à un signal enregistré, qui les fait revenir.

::: warning
Une requête enregistrée comme signal conserve ses identifiants dans le fichier de bibliothèque,
`signals.json`, en texte clair. Dans une expérience, écrivez un mot de passe comme
`{{secret.NAME}}` ; voir [Données et modèles](../experiments/data.md).
:::

## Cookies {#cookies}

Avec [[ui:http.keepCookies]] activé, ce qu’un serveur définit avec `Set-Cookie` est conservé
dans la réserve de cookies de l’écran et renvoyé avec les requêtes ultérieures vers ce serveur,
selon les règles des navigateurs (domaine, chemin, `Secure`, expiration). La réserve est utilisée
par les requêtes de cet écran, sa rafale et les signaux HTTP que vous envoyez depuis la
bibliothèque. Désactivez-la pour envoyer des requêtes sans cookies et n’en conserver aucun.

Le panneau des cookies sous la requête et la réponse liste ce que la réserve contient :
[[ui:http.cookieName]], [[ui:http.cookieValue]], [[ui:http.cookieWhere]]
(un domaine commençant par `.` couvre aussi ses sous-domaines),
[[ui:http.cookieExpires]] ([[ui:http.cookieSession]] pour un cookie sans
expiration) et [[ui:http.cookieFlags]] (`Secure`, `HttpOnly`, `SameSite`).
Les cookies expirés ne sont pas listés. [[ui:common.clear]] vide la réserve.

La réserve vit aussi longtemps que l’application : un redémarrage repart avec une réserve vide. Sur un
[serveur](../server/index.md), il y a une réserve pour chaque page connectée.
Une exécution d’expérience a sa propre réserve (voir [Expériences](../experiments/index.md)),
et `signallab send http` n’en utilise aucune.

## La réponse {#response}

| Partie | Quoi |
| --- | --- |
| [[ui:http.status]] | Le code de statut et sa raison ; `ERR` quand aucune réponse n’est venue |
| [[ui:http.latency]] | De l’envoi au dernier octet du corps, en millisecondes |
| [[ui:http.size]] | La taille du corps |
| En-têtes de réponse | Cliquez sur la ligne avec leur nombre pour les afficher ou les masquer |
| Corps | Formaté quand c’est du JSON ; [[ui:http.rawBody]] et [[ui:http.formatJson]] basculent. Jusqu’à 256 Kio est affiché, puis `… (truncated)`. |

Quand il n’y a pas de réponse, le panneau indique pourquoi, dans les mêmes mots que partout
dans Signal Lab : refusée, pas de réponse à temps, le nom ne se résout pas, un
problème de certificat, etc. Le détail technique du système est replié
en dessous.

### Redirections {#redirects}

Les redirections (301, 302, 303, 307, 308) sont suivies, jusqu’à 10 ; la réponse affichée
est la dernière. Après 301, 302 et 303, la requête continue en GET sans
corps (HEAD reste HEAD) ; après 307 et 308, telle quelle. `Authorization` et les
cookies saisis pour un hôte ne sont pas envoyés à un autre.

### Connexions sécurisées {#tls}

Le certificat d’un serveur `https://` est vérifié par rapport aux certificats auxquels ce
système fait confiance. Un certificat auto-signé ou expiré est refusé avec
« A secure connection to … could not be made » ; il n’y a pas de réglage pour ignorer la
vérification. Pour tester un serveur avec votre propre certificat, ajoutez-le aux
certificats de confiance du système.

## Rafale de charge {#burst}

La [[ui:http.burst]] envoie la requête à l’écran — avec son authentification et,
tant que [[ui:http.keepCookies]] est activé, la réserve de cookies — de nombreuses fois, et la
mesure.

1. Réglez [[ui:common.concurrency]], [[ui:http.total]], [[ui:http.duration]] et
   [[ui:http.rate]].
2. Appuyez sur [[ui:http.startBurst]]. La rafale est une tâche : [[ui:http.stopBurst]], ou
   l’arrêter dans le bandeau de la console, y met fin.

| Champ | Quoi | Par défaut |
| --- | --- | --- |
| [[ui:common.concurrency]] | Requêtes en vol en même temps, 1–512 | 20 |
| [[ui:http.total]] | Requêtes à envoyer ; 0 — continuer d’envoyer jusqu’à la fin de la durée | 500 |
| [[ui:http.duration]] | Secondes d’exécution ; 0 — s’arrêter quand le total est envoyé | 0 |
| [[ui:http.rate]] | Requêtes lancées par seconde, 0,1–100 000 ; 0 — aussi vite que les travailleurs y arrivent | 0 |

Avec [[ui:http.total]] et [[ui:http.duration]] tous deux à 0, la rafale tourne jusqu’à
ce que vous l’arrêtiez.

Il y a deux façons d’envoyer :

- **[[ui:http.rate]] 0.** Chacun des travailleurs renvoie dès qu’il a une
  réponse. Cela trouve combien le serveur encaisse, mais un serveur lent ralentit aussi
  la rafale.
- **Un débit.** Les requêtes démarrent selon un calendrier fixe — à 10 par seconde, une
  toutes les 100 ms depuis le début — quelle que soit la lenteur des réponses. Une requête dont le
  moment arrive alors que tous les travailleurs sont occupés attend au plus 50 ms l’un
  d’eux ; après quoi elle est sautée et comptée comme [[ui:http.missed]], jamais envoyée
  en retard. Les requêtes manquées signifient que la simultanéité est trop faible pour ce
  débit, ou que le serveur est plus lent que le débit ne l’exige.

| Nombre | Quoi |
| --- | --- |
| [[ui:http.sent]] | Requêtes qui ont obtenu une réponse ou échoué |
| [[ui:http.ok]] | Répondues avec un statut 2xx |
| [[ui:http.failed]] | Aucune réponse, ou un statut hors de 200–299 |
| [[ui:http.missed]] | Sautées, comme ci-dessus (seulement avec un débit) |
| [[ui:http.rps]] | Requêtes par seconde sur le dernier dixième de seconde ; quand la rafale est terminée, sur toute la rafale. Avec un débit, l’étiquette nomme le débit demandé. |
| [[ui:http.p50]], [[ui:http.p90]], [[ui:http.p95]], [[ui:http.p99]] | Le temps dans lequel cette part des requêtes s’est terminée, échecs compris ; précis à 0,5 % près |
| [[ui:http.avg]], [[ui:http.min]], [[ui:http.max]] | La moyenne, la plus rapide et la plus lente |

Les nombres sont mis à jour environ 10 fois par seconde. Le graphique à côté trace
les requêtes par seconde sur les 24 dernières secondes environ.

Avec Digest, le challenge de la première requête est répondu une fois et cette réponse
sert à toutes les requêtes de la rafale.

::: warning
Une rafale est une vraie charge. Ne la dirigez que vers des serveurs que vous possédez ou êtes autorisé à tester.
:::

Pour les rampes, les paliers, les pics et les seuils de réussite/échec, exécutez la requête sous
[charge dans une expérience](../experiments/load.md).

## Dans l’Inspecteur {#inspector}

Quand la capture est armée, chaque échange apparaît comme une trame avec le protocole
`http` et la source `http` : la méthode, l’URL, le statut et le temps dans le résumé,
les en-têtes de réponse et le début du corps (2000 caractères) dans son détail,
le statut comme verdict (`failed` quand aucune réponse n’est venue, `· digest after 401`
quand un challenge a été répondu). La trame enregistre la taille du corps, pas ses
octets. L’en-tête `Authorization` de la requête n’y est jamais. Une rafale place au
plus un échange toutes les 100 ms dans la capture. Voir
[Inspecteur](../tools/inspector.md).

## Enregistrer et réutiliser {#library}

- **Enregistrer comme signal.** [[ui:sig.saveNew]] conserve la requête — méthode, URL,
  en-têtes, corps, délai d’attente et authentification — dans la bibliothèque de signaux. L’écran
  reste lié à elle : [[ui:sig.save]] (<kbd>Ctrl</kbd>+<kbd>S</kbd>) la met à jour,
  [[ui:sig.saveAs]] la copie, la pastille l’ouvre dans [[ui:nav.signals]]. Ouvrir
  un signal HTTP depuis la bibliothèque le recharge ici, identifiants compris.
  Voir [Signaux](../tools/signals.md).
- **Ajouter à une expérience.** [[ui:common.toExperiment]] ajoute une étape
  [Requête HTTP](../experiments/nodes.md#node-http) avec la même requête
  à l’expérience ouverte, juste avant Fin ou après l’étape sélectionnée, et l’ouvre.
- **Émuler ceci.** Sous une réponse, [[ui:http.mockThis]] fabrique une route d’émulateur
  qui répond à cette méthode et à ce chemin avec ce statut, ces en-têtes et ce corps. Choisissez
  un émulateur HTTP dans [[ui:http.mockInto]], ou [[ui:http.mockNew]], et appuyez sur
  [[ui:http.mockAdd]] ; la route passe en premier dans cet émulateur, et l’écran
  [[ui:nav.emulators]] s’ouvre dessus. Voir [Émulateurs](../tools/emulators.md).

## Dans les expériences {#experiments}

| Étape | Ce qu’elle fait |
| --- | --- |
| [[ui:exp.node.http]] | Envoie une requête ; son URL, ses en-têtes, son corps et ses identifiants prennent des `{{templates}}`. Elle peut s’exécuter [sous charge](../experiments/load.md). [Détails](../experiments/nodes.md#node-http) |
| [[ui:exp.node.assert_status]], [[ui:exp.node.assert_body]], [[ui:exp.node.assert_header]], [[ui:exp.node.assert_latency]] | Vérifient la dernière réponse. [Détails](../experiments/nodes.md#node-assert_status) |
| [[ui:exp.node.extract]] | Garde un champ JSON, un en-tête, le statut, le corps ou la correspondance d’une expression régulière comme variable. [Détails](../experiments/nodes.md#node-extract) |
| [[ui:exp.node.branch_status]] | Continue par Oui ou Non selon le statut. [Détails](../experiments/nodes.md#node-branch_status) |
| [[ui:exp.node.wait_http]] | Attend qu’une requête arrive — du système que vous testez — à l’écouteur ou à l’émulateur propre à l’exécution. [Détails](../experiments/nodes.md#node-wait_http) |
| [[ui:exp.node.emulator]] | Une API HTTP qui répond par routes pendant toute l’exécution. [Détails](../experiments/nodes.md#node-emulator) |

## Depuis la ligne de commande {#cli}

`signallab send http` envoie une requête, comme cet écran le fait :

```bash
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab send http POST http://127.0.0.1:8080/api/items \
  -H 'Content-Type: application/json' --body '{"name":"lamp"}'
signallab send http GET http://127.0.0.1:8080/private -u admin:secret --digest
```

La ligne de statut va vers l’erreur standard et le corps vers la sortie standard :

```text
HTTP 200 OK · 3 ms · 15 B
{"status":"ok"}
```

| Option | Quoi | Par défaut |
| --- | --- | --- |
| `-H`, `--header 'Name: value'` | Un en-tête ; répétez pour en ajouter | — |
| `--body TEXT`, `--body @FILE` | Le corps, ou le contenu d’un fichier | — |
| `--expect-status N` | Sortir avec 1 sauf si le statut est N | — |
| `--timeout MS` | Combien de temps attendre la réponse | 10 000 |
| `-u`, `--user NAME:PASSWORD` | Authentification Basic | — |
| `--digest` | Avec `--user` : répondre plutôt au challenge Digest du serveur | — |
| `--bearer TOKEN` | `Authorization: Bearer TOKEN` | — |
| `--json` | Imprimer toute la réponse en JSON sur la sortie standard | — |

Il sort avec 0 quand une réponse est venue (et avait le statut attendu), 1 quand
aucune n’est venue, que le statut n’était pas celui attendu ou qu’un challenge Digest n’a pas pu
être répondu, et 2 quand une option est invalide. Il ne conserve aucun cookie. Voir
[Ligne de commande](../automation/cli.md#cli-send-http).

## Problèmes {#troubleshooting}

| Ce que vous voyez | Cause habituelle |
| --- | --- |
| `… refused the connection — nothing is listening on that port` | Le serveur ne tourne pas, ou écoute sur un autre port ou une autre adresse. |
| `No answer from … in time` | Le serveur est lent ou injoignable ; vérifiez l’adresse, ou augmentez [[ui:common.timeoutMs]]. |
| `Cannot resolve …` | Le nom d’hôte ne se résout pas sur cet ordinateur — une faute de frappe, ou un nom que seul un autre réseau connaît. |
| `A secure connection to … could not be made` | Le certificat n’est pas de confiance ici (auto-signé, expiré, un autre nom), ou TLS a échoué. Voir [Connexions sécurisées](#tls). |
| `… is not a valid address` | L’URL est mal formée ou ne commence pas par `http://` ou `https://`. |
| Le serveur dit que le corps manque ou a un mauvais type | Aucun en-tête `Content-Type` qui correspond au corps, ou un corps vide. |
| `401` avec Digest | Lisez le message sous le statut : voir [Digest](#digest). |
| [[ui:http.missed]] au-dessus de 0 | Augmentez [[ui:common.concurrency]], ou baissez le débit : le serveur répond plus lentement que le débit ne l’exige. |
| [[ui:http.failed]] élevé alors que le serveur répond | Tout statut hors de 200–299 compte comme échoué, 404 et 500 compris. |

Sur un serveur, les requêtes partent du serveur : `127.0.0.1` est le serveur lui-même.
Voir [Serveur](../server/index.md).

Chaque message d’erreur est listé dans [Messages d’erreur](../reference/errors.md#transport).

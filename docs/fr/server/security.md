---
title: "Sécurité du serveur"
description: "Qui peut utiliser un serveur Signal Lab et comment il tient les autres à l’écart — jetons, sessions, contrôles d’hôte et d’origine, secrets — et ce que Signal Lab envoie vers l’extérieur."
---

# Sécurité du serveur

Un serveur Signal Lab envoie du vrai trafic depuis la machine sur laquelle il tourne : OSC, UDP,
HTTP, MQTT, tempêtes, scans, diffusions. Quiconque peut l’utiliser peut faire tout cela
depuis cette machine : le serveur est donc fermé par défaut et ne s’ouvre qu’avec un
jeton.

::: warning
Traitez le jeton d’accès comme un mot de passe donnant accès au réseau de la machine. Quiconque
le détient peut envoyer du trafic depuis le serveur vers tout ce que le serveur atteint.
:::

## En bref {#summary}

- **Sans jeton, cette machine seulement.** Sans jeton, le serveur écoute en
  boucle locale et ne répond qu’aux noms d’hôte de boucle locale. Sur toute autre adresse, il
  refuse de démarrer.
- **Un jeton pour tous les autres.** Les navigateurs se connectent une fois et reçoivent un cookie
  de session ; les scripts envoient le jeton avec chaque requête.
- **Seulement ses propres pages.** Les requêtes qui modifient quelque chose, et le WebSocket
  des événements, doivent venir de l’origine propre au serveur ; les commandes n’acceptent que du JSON.
- **Seulement ses propres noms.** Un nom d’hôte auquel le serveur ne répond pas est refusé,
  ce qui empêche le DNS rebinding.
- **Les secrets restent à l’intérieur.** En lecture seule, issus de l’environnement ou de fichiers, jamais
  renvoyés, masqués partout où ils apparaîtraient.
- **Rien au-delà de son rôle.** Il ne modifie jamais le pare-feu de l’hôte, ne sert aucun
  fichier hors de son dossier de données et n’a besoin d’aucun privilège.

## Sans jeton : cette machine seulement {#loopback}

Démarré sans jeton, le serveur écoute sur `127.0.0.1:1430` et ne demande aucune
connexion : c’est un outil pour la personne devant cette machine. Pour empêcher une page web ouverte dans
n’importe quel navigateur de cette machine de l’atteindre par un nom qui se résout en
`127.0.0.1` (DNS rebinding), il ne répond qu’aux requêtes dont le `Host` est un
nom de boucle locale — `localhost`, un nom se terminant par `.localhost`, `127.x.x.x` ou
`[::1]` — ou un nom que vous autorisez avec `--allowed-host`.

Si on lui demande d’écouter sur une autre adresse sans jeton, il ne démarre pas : il
indique pourquoi et se termine avec le code `2`.

## Le jeton {#token}

Un jeton compte au moins 24 caractères, sans espaces ni sauts de ligne.
`signal-lab-server token` en affiche un aléatoire de 64 caractères hexadécimaux, et
`--generate-token` (actif dans l’image) en crée un au premier démarrage, le conserve dans
le dossier de données, lisible par le seul utilisateur du serveur, et l’affiche une fois. Voir
[Le jeton d’accès](index.md#token) pour toutes les façons d’en fournir un.

La comparaison d’un jeton prend le même temps quel que soit l’endroit où il diffère, et un jeton erroné
coûte une attente d’une seconde et un avertissement dans le journal — deviner est lent et laisse
une trace.

## Navigateurs : les sessions {#sessions}

Un navigateur qui n’est pas connecté est renvoyé vers la page de connexion. Son jeton est
échangé une fois contre une session, conservée dans un cookie qui est :

- `HttpOnly` — aucun script d’une page ne peut le lire ;
- `SameSite=Strict` — aucune page d’un autre site ne peut faire en sorte que le navigateur l’envoie ;
- valable 7 jours ;
- `Secure` avec `--secure-cookie`, pour qu’il ne circule que sur HTTPS (activez-le derrière
  un proxy HTTPS).

Les sessions vivent dans la mémoire du serveur : un redémarrage déconnecte tout le monde, et
le bouton [[ui:app.signOut]] en termine une immédiatement. Au plus 1024 sessions sont conservées ; la plus ancienne
part la première.

## Scripts : le jeton Bearer {#bearer}

Un script, `signallab --server` et la CI envoient le jeton avec chaque requête :

```http
Authorization: Bearer <token>
```

Chaque point de terminaison exige le jeton ou une session, sauf `GET /api/health` (si
le serveur répond, sa version, s’il demande un jeton) et la page de
connexion. Une requête à l’API sans l’un ni l’autre reçoit `401` avec l’erreur `auth.required` ;
une page reçoit la page de connexion.

## Noms d’hôte {#hosts}

| Serveur démarré | Noms d’hôte auxquels il répond |
| --- | --- |
| sans jeton | les noms de boucle locale, et les noms de `--allowed-host` |
| avec un jeton, sans `--allowed-host` | n’importe quel nom |
| avec un jeton et `--allowed-host` | les noms de boucle locale, et les noms de `--allowed-host` |

`--allowed-host` (`SIGNALLAB_ALLOWED_HOSTS`) prend des noms séparés par des virgules,
comparés sans le port et sans tenir compte de la casse :

```bash
signal-lab-server --listen 0.0.0.0:1430 --token-file token.txt --allowed-host lab-pc.example.com,192.0.2.10
```

Tout autre `Host` reçoit `403` avec l’erreur `auth.host`. Définissez cette option sur un serveur
joignable sous des noms connus, pour qu’une page d’un autre site ne puisse pas l’atteindre
par un nom à elle.

## Origine et type de contenu {#origin}

- Chaque requête qui modifie quelque chose (tout sauf `GET` et `HEAD`), et le
  WebSocket des événements, ne doivent porter aucun `Origin`, ou celui du serveur lui-même — le même
  hôte et le même port que son `Host`. Une page d’un autre site, ou une page qui envoie
  `Origin: null`, reçoit `403` avec `auth.origin`. Les scripts et `curl` n’envoient pas
  d’`Origin` et ne sont pas concernés.
- Les commandes n’acceptent que `Content-Type: application/json` (sinon `415` avec
  `command.json_required`) : un formulaire d’un autre site ne peut donc pas en envoyer.
- Le serveur ne répond à aucune requête d’origine croisée (CORS).

## En-têtes de réponse {#headers}

Chaque réponse porte :

| En-tête | Valeur |
| --- | --- |
| `Content-Security-Policy` | `default-src 'self'; connect-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; object-src 'none'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'` |
| `X-Content-Type-Options` | `nosniff` |
| `X-Frame-Options` | `DENY` |
| `Referrer-Policy` | `same-origin` |
| `Cache-Control` | `no-store` pour l’API et la page de connexion |

L’interface ne charge que ce que sert le serveur, ne parle qu’au
serveur, et ne peut pas être intégrée dans le cadre d’une autre page.

## Fichiers et tailles {#files}

- Un téléchargement (`GET /api/files?path=…`) n’est servi que depuis l’intérieur du dossier de
  données, 256 Mio au plus ; tout le reste donne `404`.
- Un corps de requête fait au plus 24 Mio.

## Secrets {#secrets}

La valeur d’un secret ne quitte jamais le moteur :

- Sur un serveur, les valeurs sont **en lecture seule** : la variable d’environnement
  `SIGNALLAB_SECRET_<NAME>`, ou le fichier `<NAME>` dans le dossier des secrets
  (`/run/secrets/signallab` par défaut). En définir ou en supprimer un depuis le
  navigateur est refusé (`secret.read_only`) : une valeur saisie dans une page ne
  finit jamais stockée dans un endroit moins sûr. Voir [Secrets](index.md#secrets).
- Aucune commande ne renvoie de valeur ; l’interface apprend seulement si un nom est défini.
- Les expériences nomment les secrets sous la forme `{{secret.NAME}}`. Tant qu’une exécution ou un envoi les utilise,
  chaque texte qu’il rapporte — étapes, erreurs, rapport d’exécution — affiche `••••` à
  leur place, et les trames de l’onglet [[ui:dock.inspector]] sont masquées octet par octet.
- Les identifiants d’un nœud HTTP ne deviennent un en-tête `Authorization` qu’au moment où la
  requête est envoyée ; les étapes, les trames et les rapports portent la réponse, jamais cet
  en-tête.

## Ce qui est journalisé {#audit}

Le journal enregistre, avec l’adresse du client : chaque tâche démarrée — tempêtes, scans,
diffusions, moniteurs, générateurs, exécutions, émulateurs —, chaque connexion, et chaque
tentative de connexion avec un jeton erroné. Voir [Journaux](index.md#logs).

## Ce que le serveur ne fait jamais {#never}

- **Modifier le pare-feu de l’hôte.** L’application de bureau peut ajouter une règle de pare-feu quand
  vous le lui demandez ; sur un serveur, cette commande est refusée (`firewall.server`). Le
  pare-feu de l’hôte appartient à qui exploite l’hôte. (Le script d’installation en une commande
  propose d’ouvrir le port du serveur dans ufw ou firewalld, et demande d’abord —
  voir [Le pare-feu de l’hôte](index.md#firewall).)
- **Démarrer joignable sans jeton**, sur toute autre adresse que la boucle locale.
- **Servir un fichier situé hors de son dossier de données.**
- **Stocker un secret** saisi dans un navigateur.
- **Parler TLS** lui-même : placez un proxy HTTPS devant lui (voir
  [Derrière un proxy HTTPS](index.md#https)).

Chaque limite du moteur — 1024 hôtes au plus dans un balayage, 50 000
paquets par seconde au plus depuis une balise — s’applique sur un serveur comme dans l’application. Ce sont
des garde-fous, pas une autorisation : n’envoyez du trafic qu’à des systèmes qui vous appartiennent ou que vous pouvez tester.

## Le conteneur {#container}

L’image s’exécute sous un utilisateur non privilégié (uid et gid 10001) et n’écrit que dans
`/data`. Elle fonctionne telle quelle avec un système de fichiers racine en lecture seule, sans capabilities
et avec `no-new-privileges` — comme le script d’installation et `deploy/compose.yaml`
la démarrent. Chaque image est publiée avec un SBOM, une provenance de build et une attestation
GitHub signée :

```bash
gh attestation verify oci://ghcr.io/proanima/signallab:[[version]] -R ProAnima/SignalLab
```

## Ce que Signal Lab envoie vers l’extérieur {#outside}

En plus du trafic que vous envoyez, Signal Lab s’adresse à deux endroits, tous deux appartenant au studio.

### Recherche de mises à jour {#update-check}

Seule l’**application de bureau** recherche des mises à jour ; un serveur et un navigateur ne le font jamais.
Avec la case [[ui:update.auto]] cochée (dans [[ui:about.open]]), une fois par jour, et chaque fois que vous
appuyez sur [[ui:update.check]], l’application interroge le hub du studio
(`hub.proanima.net`) — et la dernière version publiée sur GitHub seulement quand le hub est
injoignable. La demande contient :

- la version de l’application ;
- le système d’exploitation et l’architecture du processeur ;
- un nombre aléatoire propre à cette installation (`X-Install-Id`), créé une fois et conservé avec les
  réglages de l’application, pour qu’une nouvelle version puisse atteindre d’abord une partie des installations. Il ne dit
  rien de vous ni de l’ordinateur.

Seules les versions publiées sont proposées. Un téléchargement dont la signature ne correspond pas à
la clé intégrée à l’application n’est pas installé, et rien n’est installé tant que vous
n’appuyez pas sur [[ui:update.install]].

### Retours {#feedback}

Le formulaire [[ui:feedback.open]] (le ✉ de l’en-tête, également dans [[ui:about.open]]) envoie un message aux
développeurs via le hub du studio, qui le transmet par e-mail ; l’application ne détient aucun
mot de passe pour cela. Il n’envoie que ce que montre le formulaire : votre message, votre e-mail si
vous en donnez un, les captures d’écran que vous ajoutez et — sous [[ui:feedback.logs]] — le
journal de la console et [[ui:feedback.systemInfo]], que vous pouvez chacun ouvrir avant
l’envoi et décocher. Le nom de cet ordinateur, son adresse et vos dossiers en sont
exclus. Depuis un navigateur, c’est le serveur qui envoie le formulaire.

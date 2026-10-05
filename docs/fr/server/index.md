---
title: "Exécuter un serveur"
description: "Exécutez Signal Lab sur une machine Linux à côté du matériel et utilisez-le depuis n’importe quel navigateur du réseau, avec une API HTTP pour les scripts et la CI."
---

# Exécuter Signal Lab comme serveur

`signal-lab-server` est Signal Lab sans fenêtre : le même moteur, servant la même
interface à un navigateur. Placez-le sur une machine à côté du matériel — un PC
en baie, une VM de contrôle de spectacle, une machine de laboratoire partagée — et
ouvrez-le depuis Chrome, Firefox ou Edge n’importe où sur le réseau : chaque écran
fonctionne comme dans l’application de bureau, et les exécutions, les rapports et
les exports se téléchargent via le navigateur.

Les scripts et les pipelines utilisent le même serveur via son [API HTTP](../api/index.md),
et [`signallab --server`](../automation/cli.md#run-on-server) lui envoie des exécutions.

Quelques points diffèrent de l’application de bureau :

- **Un serveur est un moteur.** Chaque page connectée à lui voit les mêmes tâches
  en cours, les mêmes bibliothèques de signaux et d’émulateurs et la même réserve
  de cookies de l’écran [[ui:nav.http]] ([[ui:http.keepCookies]]). Les personnes qui
  partagent un serveur partagent cela.
- **Les secrets sont ceux du serveur**, lus dans son environnement ou dans des
  fichiers ; ils ne peuvent pas être définis depuis le navigateur (voir [Secrets](#secrets)).
- **Le pare-feu est celui de l’hôte.** Le serveur ne le modifie jamais ;
  l’avertissement de pare-feu de l’application de bureau n’apparaît pas.
- **Il se met à jour avec son image**, non via le programme de mise à jour de
  l’application (voir [Mise à jour](#update)).

## Sur un hôte Linux, en une commande {#install-script}

Sur une machine Linux ayant accès à Internet :

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh
```

Le script :

1. installe Docker s’il manque — en demandant d’abord, avec l’installateur de
   Docker lui-même (`get.docker.com`) ;
2. écrit un `compose.yaml` dans `/opt/signallab` (`~/signallab` lorsque vous n’êtes
   pas root) ;
3. récupère l’image et démarre le serveur avec la mise en réseau de l’hôte, pour
   qu’OSC, UDP, la diffusion, la multidiffusion et la découverte atteignent le vrai réseau ;
4. attend que le serveur réponde à son contrôle de santé (jusqu’à 90 secondes) ;
5. affiche les adresses à ouvrir, le jeton d’accès avec lequel se connecter, et les
   commandes pour mettre à jour, lire les journaux et le supprimer ;
6. lorsque ufw ou firewalld est actif, propose d’ouvrir le port du serveur (voir
   [Le pare-feu de l’hôte](#firewall)).

Passez les options après `sh -s --` :

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh -s -- --version [[version]] --port 8430
```

| Option | Ce qu’elle fait | Défaut |
| --- | --- | --- |
| `--version X.Y.Z` | La version de l’image (`latest` ou `X.Y.Z` ; un `v` initial est retiré). | `latest` |
| `--port N` | Le port utilisé par les navigateurs. | `1430` |
| `--listen IP:PORT` | N’écouter que sur une adresse. | `0.0.0.0:<port>` |
| `--dir DIR` | Où va le fichier compose. | `/opt/<name>` en root, `~/<name>` sinon |
| `--name NAME` | Le conteneur et son volume de données : lettres minuscules, chiffres, `-` et `_`. Un second serveur sur le même hôte a besoin de son propre nom. | `signallab` |
| `--image NAME` | Une autre image ou un autre registre ; un `NAME:TAG` complet est utilisé tel quel. | `ghcr.io/proanima/signallab` |
| `--open-udp PORTS` | Avec un pare-feu actif, laisse aussi entrer l’UDP sur ces ports, pour les moniteurs et les attentes : `9000,9100:9110`. | |
| `--no-firewall` | Ne jamais modifier ufw ni firewalld. | |
| `--yes`, `-y` | Répondre oui : installer Docker, ouvrir le pare-feu, supprimer les données avec `--purge`. | |
| `--uninstall` | Arrêter et supprimer le serveur ; les données restent. | |
| `--purge` | Avec `--uninstall` : supprimer aussi les données et le jeton. | |
| `--help`, `-h` | Afficher les options. | |

Il a besoin du plugin Compose de Docker (le paquet `docker-compose-plugin`), que
l’installateur de Docker apporte. L’image est construite pour x86_64 et arm64.

**Relancez-le pour le mettre à jour :** la même commande récupère l’image la plus
récente (ou la `--version` que vous donnez) et redémarre le serveur ; les données
et le jeton restent. Avec `--name`, redonnez le même nom.

**Vos propres réglages** — secrets d’expérience, `SIGNALLAB_ALLOWED_HOSTS`,
`SIGNALLAB_SECURE_COOKIE` derrière HTTPS — vont dans `compose.override.yaml` à côté
du fichier compose. Docker Compose le fusionne, et le script réécrit `compose.yaml`
à chaque exécution mais ne touche jamais à l’override. Un `compose.yaml` qu’il n’a
pas écrit est conservé sous `compose.yaml.before-install`.

```yaml
# compose.override.yaml
services:
  signallab:
    environment:
      SIGNALLAB_ALLOWED_HOSTS: lab-pc.example.com,192.0.2.10
      SIGNALLAB_SECRET_API_TOKEN: ${API_TOKEN}
```

**Pour le supprimer :**

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh -s -- --uninstall
```

Les données restent dans leur volume Docker, et une réinstallation les ramène avec
le même jeton. `--uninstall --purge` supprime aussi les données et le jeton, après
avoir demandé.

### Le pare-feu de l’hôte {#firewall}

Avec la mise en réseau de l’hôte, le serveur écoute sur les ports de l’hôte
lui-même, si bien que le pare-feu de l’hôte décide qui l’atteint. Lorsque ufw ou
firewalld est actif, le script demande avant d’ouvrir le port TCP du serveur et les
ports UDP de `--open-udp`, note ce qu’il a ouvert (`.firewall` à côté du fichier
compose), et `--uninstall` referme exactement cela. Si vous refusez, les navigateurs
des autres machines n’atteignent le serveur que lorsque le pare-feu les laisse, et
les moniteurs et les attentes n’entendent les autres machines que sur les ports UDP
qu’il ouvre.

## Docker {#docker}

### L’image {#image}

`ghcr.io/proanima/signallab`, pour `linux/amd64` et `linux/arm64`, publiée à chaque
version :

| Étiquette | Ce que c’est |
| --- | --- |
| `X.Y.Z` | Cette version. |
| `X.Y` | La dernière version stable de cette série. |
| `latest` | La dernière version stable. |

Elle contient `signal-lab-server`, l’interface construite et la ligne de commande
`signallab`, et démarre le serveur avec ces réglages :

| Variable | Valeur dans l’image |
| --- | --- |
| `SIGNALLAB_LISTEN` | `0.0.0.0:1430` |
| `SIGNALLAB_DATA_DIR` | `/data` |
| `SIGNALLAB_UI_DIR` | `/usr/share/signal-lab/ui` |
| `SIGNALLAB_GENERATE_TOKEN` | `true` |

Elle tourne sous un utilisateur non privilégié (uid et gid 10001), n’écrit que dans
`/data` (un volume), expose le port `1430` et contrôle sa propre santé toutes les
30 secondes.

### Le démarrer {#docker-run}

```bash
docker run -d --name signallab --network host --restart unless-stopped \
  -v signallab-data:/data --read-only --cap-drop ALL --security-opt no-new-privileges \
  ghcr.io/proanima/signallab:[[version]]
docker logs signallab                    # on the first start: "Sign in with it:" and the token
```

Ouvrez ensuite `http://<host>:1430` et connectez-vous avec le jeton. Pour revoir le
jeton plus tard :

```bash
docker exec signallab cat /data/token
```

`--read-only`, `--cap-drop ALL` et `no-new-privileges` sont facultatifs et ne
coûtent rien : le serveur n’a besoin d’aucun privilège et n’écrit que dans `/data`.

### Docker Compose {#compose}

Le `deploy/compose.yaml` du dépôt est identique à un fichier Compose :

```yaml
name: signallab

services:
  signallab:
    image: ${SIGNALLAB_IMAGE:-ghcr.io/proanima/signallab:latest}
    container_name: signallab
    network_mode: host
    environment:
      SIGNALLAB_GENERATE_TOKEN: "true"
    volumes:
      - signallab-data:/data
    read_only: true
    cap_drop: [ALL]
    security_opt: ["no-new-privileges:true"]
    restart: unless-stopped
    stop_grace_period: 15s

volumes:
  signallab-data:
```

```bash
docker compose up -d
docker exec signallab cat /data/token
```

Définissez `SIGNALLAB_IMAGE=ghcr.io/proanima/signallab:X.Y.Z` pour épingler une version.

### Mise en réseau {#networking}

| Mise en réseau Docker | Ce qui fonctionne | Ce qui ne fonctionne pas |
| --- | --- | --- |
| `--network host` (`network_mode: host`), sur un hôte Linux | Tout : OSC, UDP, TCP, HTTP, WebSocket et MQTT vers le réseau local, les ports d’écoute, la diffusion, la multidiffusion, la découverte. | — |
| Bridge (par défaut), avec des ports publiés | L’unicast vers les hôtes que le conteneur atteint ; les écouteurs sur les ports publiés (`-p 1430:1430 -p 9000:9000/udp`). | La diffusion et la multidiffusion ; les réponses vers des ports non publiés. |
| Docker Desktop sur Windows ou macOS | L’unicast et les écouteurs publiés. | La mise en réseau de l’hôte vers le réseau physique. Sous Windows, utilisez l’application de bureau. |

### Le volume de données {#data-volume}

`/data` contient tout ce que le serveur conserve : l’expérience, les bibliothèques
de signaux et d’émulateurs, les rapports d’exécution, les exports et le jeton. Un
volume nommé, comme ci-dessus, appartient au départ à l’utilisateur de l’image. Un
dossier de l’hôte monté à cet endroit doit être accessible en écriture à l’uid 10001 :

```bash
sudo mkdir -p /srv/signallab && sudo chown 10001:10001 /srv/signallab
docker run -d --name signallab --network host -v /srv/signallab:/data ghcr.io/proanima/signallab:[[version]]
```

## Sans Docker {#binary}

Le serveur est publié sous forme d’image. Pour exécuter `signal-lab-server`
directement, construisez-le depuis les sources (voir [Construction](../../develop/building.md)) :

```bash
npm install
npm run build                            # the interface, into dist/
cargo run --release -p signal-lab-server
```

Il sert `dist/` sur `http://127.0.0.1:1430`, pour cette machine uniquement, sans
jeton nécessaire. Ajoutez `--listen` et un jeton pour l’ouvrir au réseau.

## Options {#options}

Chaque option a une variable d’environnement, pour les conteneurs. Les options
l’emportent sur les variables.

| Option | Variable | Défaut | Ce qu’elle fait |
| --- | --- | --- | --- |
| `--listen IP:PORT` | `SIGNALLAB_LISTEN` | `127.0.0.1:1430` | Où écouter. Toute adresse autre que la boucle locale a besoin d’un jeton. |
| `--token-file PATH` | `SIGNALLAB_TOKEN_FILE` | | Un fichier contenant le jeton d’accès (un secret Docker, par exemple). |
| `--token TOKEN` | `SIGNALLAB_TOKEN` | | Le jeton d’accès lui-même. Préférez le fichier : les arguments sont visibles par les autres utilisateurs de la machine. |
| `--generate-token` | `SIGNALLAB_GENERATE_TOKEN` | désactivé | Sans jeton fourni, sur une adresse au-delà de la boucle locale : utiliser le jeton conservé dans `<data folder>/token`, en le créant au premier démarrage. |
| `--data-dir PATH` | `SIGNALLAB_DATA_DIR` | `Documents/SignalLab` dans le dossier personnel de l’utilisateur | Le dossier de données. |
| `--secrets-dir PATH` | `SIGNALLAB_SECRETS_DIR` | `/run/secrets/signallab` | Un dossier de secrets en lecture seule, un fichier par nom. |
| `--ui-dir PATH` | `SIGNALLAB_UI_DIR` | `ui` à côté du programme, sinon `./dist` | L’interface construite. Sans elle, seule l’API est servie. |
| `--allowed-host NAME` | `SIGNALLAB_ALLOWED_HOSTS` | | Les noms d’hôte par lesquels le serveur peut être atteint, séparés par des virgules. Eux et les noms de boucle locale sont toujours acceptés, avec ou sans jeton ; sans aucun nom donné, un serveur avec un jeton répond à tout nom et un serveur sans jeton uniquement aux noms de boucle locale (voir [Noms d’hôte](security.md#hosts)). |
| `--secure-cookie` | `SIGNALLAB_SECURE_COOKIE` | désactivé | N’envoyer le cookie de session que via HTTPS. À activer derrière un proxy HTTPS. |
| `--log FILTER` | `SIGNALLAB_LOG` | `info` | Quoi journaliser : `error`, `warn`, `info`, `debug`, ou par module (`signal_lab_server=debug`). |
| `--log-format text\|json` | `SIGNALLAB_LOG_FORMAT` | `text` | Les lignes de journal en texte, ou un objet JSON par ligne. |

Les interrupteurs prennent `true` ou `false` depuis leur variable :
`SIGNALLAB_GENERATE_TOKEN=true`.

| Commande | Ce qu’elle fait |
| --- | --- |
| `signal-lab-server token` | Affiche un nouveau jeton aléatoire : 64 caractères hexadécimaux. |
| `signal-lab-server healthcheck` | Sort avec `0` lorsqu’un serveur répond sur `--listen` (le contrôle de santé de l’image). |
| `signal-lab-server --version` | Affiche la version. |
| `signal-lab-server --help` | Affiche toutes les options. |

**Codes de sortie :** `0` lorsqu’il est arrêté par <kbd>Ctrl</kbd>+<kbd>C</kbd> ou
`SIGTERM` ; `2` lorsque les réglages sont refusés (pas de jeton sur une adresse
joignable, un jeton trop court, un jeton fourni deux fois, un fichier de jeton
illisible, `--generate-token` sans dossier de données) ; `1` lorsqu’il ne peut pas
écouter sur l’adresse, ou que le dossier de données ne peut pas être écrit. La
raison est affichée sur la sortie d’erreur.

## Le jeton d’accès {#token}

Sans jeton, le serveur n’écoute que sur la boucle locale et ne sert que cette
machine. Sur toute autre adresse, il en a besoin et refuse de démarrer sans. Trois
façons de le fournir :

| Comment | Quand l’utiliser |
| --- | --- |
| `--generate-token` (actif dans l’image) | Rien à configurer : au premier démarrage, le serveur crée un jeton, l’enregistre dans `<data folder>/token` — lisible par son seul utilisateur — et l’affiche une fois dans le journal. Les démarrages suivants le réutilisent, si bien que les navigateurs connectés et les scripts continuent de fonctionner à travers les redémarrages et les mises à jour. Il a besoin d’un dossier de données (`--data-dir`). |
| `--token-file PATH` | Un jeton à vous dans un fichier, comme un secret Docker. Un saut de ligne à la fin n’en fait pas partie. |
| `SIGNALLAB_TOKEN` | Le jeton dans l’environnement. |

Un jeton a au moins **24** caractères et aucun espace ni saut de ligne ; donnez-le
d’une seule façon. `signal-lab-server token` en fabrique un bon :

```bash
docker run --rm ghcr.io/proanima/signallab:[[version]] token > signallab_token.txt
```

et Compose le transmet comme secret (le fichier doit être lisible par l’uid 10001) :

```yaml
services:
  signallab:
    environment:
      SIGNALLAB_TOKEN_FILE: /run/secrets/signallab_token
    secrets:
      - signallab_token

secrets:
  signallab_token:
    file: ./signallab_token.txt
```

Un jeton donné explicitement l’emporte sur `--generate-token`, et rien n’est créé
alors. Pour changer un jeton créé, arrêtez le serveur, supprimez
`<data folder>/token` et redémarrez-le : il en crée et en affiche un nouveau. Un
fichier de jeton endommagé est signalé, jamais remplacé.

## Se connecter {#sign-in}

Ouvrez `http://<host>:1430`. Un serveur avec un jeton envoie d’abord un navigateur
vers sa page de connexion, dans la langue du navigateur ; collez le jeton une fois,
et le navigateur reste connecté 7 jours. [[ui:app.signOut]] dans l’interface met fin
à la session. Les sessions sont conservées dans la mémoire du serveur : un
redémarrage déconnecte tout le monde.

Sur la boucle locale sans jeton, il n’y a pas de connexion.

Les scripts envoient le jeton avec chaque requête, sous la forme
`Authorization: Bearer <token>` :

```bash
curl -fsS http://192.0.2.10:1430/api/invoke/app_info \
  -H "Authorization: Bearer $(cat signallab_token.txt)" \
  -H "Content-Type: application/json" -d 'null'
```

Voir [L’API HTTP](../api/index.md), et [Sécurité du serveur](security.md) pour les
règles qui sous-tendent tout cela.

## Le dossier de données {#data}

Le serveur conserve ses fichiers dans le dossier de données : l’expérience, les
bibliothèques de signaux et d’émulateurs, les rapports d’exécution, les exports et
un jeton créé. C’est `--data-dir` (`SIGNALLAB_DATA_DIR`), `/data` dans l’image, et
sinon `Documents/SignalLab` dans le dossier personnel de l’utilisateur sous lequel
il tourne. Un dossier de données donné avec `--data-dir` est créé s’il manque et
vérifié au démarrage : si le serveur ne peut pas y écrire, il s’arrête et nomme le
dossier. Voir [Fichiers et dossiers](../reference/files.md).

Les téléchargements (rapports, exports) ne viennent que de l’intérieur de ce dossier.

## Secrets {#secrets}

Les expériences lisent les secrets sous la forme `{{secret.NAME}}`. Sur un serveur,
ils sont en lecture seule, depuis :

1. la variable d’environnement `SIGNALLAB_SECRET_NAME`, sinon
2. le fichier `NAME` dans le dossier des secrets (`--secrets-dir`, par défaut
   `/run/secrets/signallab` — la disposition des secrets Docker).

Un saut de ligne final dans un fichier ne fait pas partie de la valeur ; une valeur
vide compte comme non définie ; une valeur fait au plus 16 Kio. Les noms sont des
lettres, des chiffres et `_`, ne commençant pas par un chiffre. L’interface indique
quels secrets sont définis sur le serveur, mais en définir un depuis le navigateur
est refusé — les valeurs ne vont jamais dans un endroit plus faible, et n’en
ressortent jamais (voir [Secrets](security.md#secrets)).

Avec Compose, un fichier de secret par nom :

```yaml
services:
  signallab:
    secrets:
      - source: api_token
        target: /run/secrets/signallab/API_TOKEN

secrets:
  api_token:
    file: ./api_token.txt
```

Le fichier doit être lisible par l’uid 10001.

## Derrière un proxy HTTPS {#https}

Le serveur parle HTTP en clair. Pour HTTPS, placez un proxy inverse devant lui
(Caddy, nginx, Traefik) qui :

- transmet l’en-tête `Host` sans le modifier ;
- transmet les mises à niveau WebSocket (l’interface en garde une ouverte, vers `/api/events`) ;

et démarrez le serveur avec `--secure-cookie`, pour que le cookie de session ne
circule que via HTTPS, et avec `--allowed-host` réglé sur le nom que les gens utilisent.

## Mise à jour {#update}

[[ui:update.server]] : le programme de mise à jour de l’application de bureau n’a
rien à voir avec cela.

- Installé avec le script : relancez la même commande.
- Avec Compose : `docker compose pull && docker compose up -d`.
- Avec `docker run` : récupérez la nouvelle image, puis supprimez le conteneur et
  redémarrez-le avec le même volume.

Les données et le jeton sont dans le volume, donc ils restent.

## Contrôle de santé {#health}

`GET /api/health` répond à tout le monde, sans jeton :

```bash
curl -s http://127.0.0.1:1430/api/health
# {"auth":true,"status":"ok","version":"[[version]]"}
```

`auth` indique si le serveur demande un jeton. La commande
`signal-lab-server healthcheck` demande la même chose sur cette machine et sort avec
`0` lorsqu’il répond ; l’image l’exécute toutes les 30 secondes (délai de
5 secondes, 3 essais), si bien que `docker ps` montre le conteneur comme sain.

## Journaux {#logs}

Le serveur journalise sur stdout : `docker logs -f signallab`. Au niveau par défaut
(`info`), il indique où il écoute et s’il a besoin d’un jeton, ses dossiers de
données et d’interface, chaque tâche démarrée — moniteurs, générateurs, tempêtes,
scans, exécutions, émulateurs — avec l’adresse du client, chaque connexion, chaque
connexion avec un mauvais jeton (comme avertissement), et quand une page prend du
retard sur les événements. Le niveau `--log debug` ajoute chaque commande. Les
lignes sont colorées seulement sur un terminal (jamais quand `NO_COLOR` est défini) ;
`--log-format json` écrit un objet JSON par ligne pour un collecteur de journaux.

## Arrêt {#stop}

<kbd>Ctrl</kbd>+<kbd>C</kbd>, `docker stop` ou `SIGTERM` ferme la connexion de
chaque page, arrête chaque tâche — une exécution en cours se termine comme arrêtée —
et sort avec `0`. Les fichiers compose lui accordent 15 secondes pour cela.

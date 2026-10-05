---
title: "Assistants d’IA (MCP)"
description: "signallab mcp permet à un assistant d’IA de construire, vérifier et exécuter des expériences, d’envoyer des messages, d’écouter et de jouer des émulateurs, via le Model Context Protocol."
---

# Signal Lab pour un assistant : `signallab mcp`

`signallab mcp` est un serveur [Model Context Protocol](https://modelcontextprotocol.io).
Un assistant dans Claude Code, Claude Desktop, Cursor, VS Code ou tout
autre client MCP le démarre et peut alors :

- apprendre de quoi est faite une expérience, en écrire une, la vérifier, l’exécuter et lire,
  étape par étape, pourquoi elle a échoué ;
- envoyer un message OSC, un datagramme, une requête HTTP, un message WebSocket ou une publication
  MQTT, et écouter sur un port ce qu’envoie un appareil ;
- envoyer un signal de votre bibliothèque ;
- jouer une dépendance — une API HTTP, un appareil OSC, UDP ou TCP, un broker MQTT —
  et lire ce que votre système lui a envoyé ;
- relire des exécutions précédentes et en comparer deux.

Chaque action passe par les mêmes commandes du moteur que celles qu’utilise l’application : une
expérience que l’assistant exécute est l’exécution que ferait l’application, avec le même
rapport, et chaque échec est formulé comme l’interface le formule.

::: warning
Les envois, les exécutions et les émulateurs mettent du vrai trafic sur le réseau. Indiquez à l’assistant
les appareils auxquels il peut s’adresser ; les modèles intégrés visent la boucle locale
(`127.0.0.1`).
:::

## Mise en place {#setup}

`signallab` est fourni avec l’application de bureau et se trouve dans votre `PATH` après l’installation
(voir [Installation](cli.md#install)). Le client démarre lui-même `signallab mcp` et
lui parle par stdin et stdout ; vous ne le lancez pas à la main.

### Afficher la configuration {#print-config}

`--print-config` affiche ce dont un client a besoin, avec le chemin complet de ce
`signallab` :

| Commande | Ce qu’elle affiche |
| --- | --- |
| `signallab mcp --print-config claude-code` | La ligne de commande `claude mcp add`. |
| `signallab mcp --print-config claude-desktop` | L’entrée `mcpServers` pour le fichier de configuration de Claude Desktop. |
| `signallab mcp --print-config cursor` | La même entrée `mcpServers`, pour le `mcp.json` de Cursor. |
| `signallab mcp --print-config vscode` | L’entrée `servers` pour le `.vscode/mcp.json` de VS Code. |

Pour un assistant qui travaille sur un [serveur de laboratoire](#on-a-server), ajoutez
`--server URL` : la configuration affichée le reprend alors, avec un emplacement réservé
pour le jeton.

### Claude Code {#claude-code}

Exécutez la ligne qu’affiche `--print-config claude-code`, par exemple :

```bash
claude mcp add signallab -- "C:\Program Files\Signal Lab\signallab.exe" mcp
```

### Claude Desktop et Cursor {#claude-desktop}

Placez l’entrée dans la configuration du client — pour Claude Desktop,
`claude_desktop_config.json` ; pour Cursor, `mcp.json` — et redémarrez le client :

```json
{
  "mcpServers": {
    "signallab": {
      "command": "C:\\Program Files\\Signal Lab\\signallab.exe",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

### VS Code {#vscode}

```json
{
  "servers": {
    "signallab": {
      "type": "stdio",
      "command": "/usr/bin/signallab",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

### Autres clients {#other-clients}

Tout client qui démarre un serveur stdio fonctionne de la même façon : la commande est
`signallab` (ou son chemin complet), les arguments `mcp` et, au besoin, les
[options](#options). Sous Linux, l’image du serveur peut aussi servir de commande :

```bash
docker run -i --rm --network host --entrypoint signallab ghcr.io/proanima/signallab:[[version]] mcp
```

## Options {#options}

| Option | Effet | Par défaut |
| --- | --- | --- |
| `--server URL` | Exécute les expériences, les envois et les émulateurs sur ce serveur Signal Lab (voir [Sur un serveur de laboratoire](#on-a-server)). | `SIGNALLAB_SERVER` |
| `--token-file PATH` | Un fichier qui contient le jeton du serveur. | `SIGNALLAB_TOKEN_FILE`, sinon `SIGNALLAB_TOKEN` |
| `--data-dir PATH` | Où sont conservés les exécutions et leurs rapports. Pas avec `--server`. | le dossier de données de l’application (`Documents/SignalLab`) |
| `--library PATH` | La bibliothèque de signaux pour `list_signals` et `fire_signal`. | le `signals.json` de l’application |
| `--emulators PATH` | La bibliothèque d’émulateurs pour `list_emulators` et `start_emulator`. | le `emulators.json` de l’application |
| `--secrets files\|system` | D’où viennent les valeurs des secrets pour les exécutions sur cette machine, comme pour [`run`](cli.md#secrets). Pas avec `--server`. | `files` |
| `--secrets-dir PATH` | Un dossier de fichiers de secrets, un par nom. Pas avec `--server`. | `/run/secrets/signallab` s’il existe |
| `--lang <code>` | La langue des résultats et des échecs. | `SIGNALLAB_LANG`, sinon les paramètres régionaux, sinon `en` |
| `--print-config CLIENT` | Affiche la configuration d’un client et quitte : `claude-code`, `claude-desktop`, `cursor` ou `vscode`. | |

Les exécutions conservent leurs rapports dans le dossier de données de l’application, là où l’application conserve les siens :
ils restent donc après la session.

## Outils {#tools}

Les outils qui ne font que lire sont marqués en lecture seule : un client peut les laisser s’exécuter sans
demander. Les outils qui atteignent le monde extérieur — ils envoient, écoutent ou démarrent
quelque chose — sont marqués comme tels, et un client peut vous demander votre accord avant chaque appel. Aucun n’est
marqué comme destructif.

| Outil | Rôle | Atteint le monde extérieur |
| --- | --- | --- |
| `describe_nodes` | Le document d’expérience, chaque type de nœud avec ses champs, ses sorties et un exemple, le langage `{{template}}`, les profils de charge et le document d’émulateur. | non |
| `list_templates` | Les expériences intégrées, avec leurs paramètres. | non |
| `get_template` | Une expérience intégrée, sous forme de document. | non |
| `validate_experiment` | Vérifie une expérience comme l’éditeur le fait avant une exécution ; n’envoie rien. | non |
| `run_experiment` | Exécute une expérience jusqu’au bout et rend compte de chaque étape. | oui |
| `send_osc` | Un message OSC. | oui |
| `send_udp` | Un datagramme UDP. | oui |
| `send_http` | Une requête HTTP. | oui |
| `send_mqtt` | Une publication MQTT 3.1.1. | oui |
| `send_ws` | Un échange WebSocket. | oui |
| `listen` | Ce qui arrive sur un port UDP pendant un moment. | oui |
| `list_signals` | Les signaux de votre bibliothèque. | non |
| `fire_signal` | Envoie un signal de la bibliothèque. | oui |
| `list_emulators` | Les émulateurs de votre bibliothèque. | non |
| `start_emulator` | Démarre un émulateur. | oui |
| `emulator_exchanges` | Ce qu’un émulateur en cours a reçu et répondu. | non |
| `set_emulator_down` | Met en panne un émulateur en cours, ou le remet en service. | oui |
| `list_runs` | Les rapports des exécutions précédentes. | non |
| `compare_runs` | Deux exécutions côte à côte. | non |
| `list_jobs` | Ce qui tourne. | non |
| `stop_job` | Arrête une tâche en cours. | oui |

### Expériences {#tools-experiments}

`describe_nodes` est ce que l’assistant lit avant d’écrire une expérience ; c’est
la même chose que [`signallab nodes`](cli.md#cli-nodes). `list_templates` et
`get_template` donnent des exemples fonctionnels à exécuter ou à adapter.

`validate_experiment` et `run_experiment` reçoivent l’expérience de l’une de trois
façons — exactement une :

| Argument | Ce que c’est |
| --- | --- |
| `document` | Un document d’expérience, tel que l’application l’enregistre. |
| `file` | Le chemin d’un fichier d’expérience sur la machine où tourne `signallab`. |
| `template` | Le nom d’un modèle intégré. |
| `params` | Les valeurs des paramètres pour cette exécution : `{"name": "value"}` ; les nombres et les booléens sont pris comme du texte. |
| `profile` | Exécuter avec ce profil du document ; `""` pour les valeurs par défaut. |
| `seed` | `run_experiment` : la graine des valeurs aléatoires. |
| `timeout` | `run_experiment` : le nombre de secondes que peut durer l’exécution, de 1 à 300 (300 par défaut). |

`run_experiment` répond quand l’exécution est terminée : réussie, échouée ou arrêtée, sa
durée et sa graine, chaque étape avec ce qu’elle a fait ou la raison de son échec, ce qui a été demandé à chaque
émulateur, ce qu’a fait chaque relais de dégradation, et le chemin du rapport. Une
exécution qui échoue est une réponse normale — les étapes disent pourquoi —, pas un appel en échec.

### Messages uniques {#tools-send}

| Outil | Arguments |
| --- | --- |
| `send_osc` | `target` (`host:port`), `address`, `args` : des nombres (entiers → int32, ou int64 au-delà de sa plage ; sinon float32), des chaînes, des booléens, `null`, ou `{"type": "int"\|"float"\|"str"\|"long"\|"double"\|"bool"\|"blob"\|"nil", "value": …}`. |
| `send_udp` | `target`, et `text` ou `hex` (`"de ad be ef"`). |
| `send_http` | `method`, `url`, `headers` (`{"Name": "value"}`), `body`, `timeout_ms` (10 000 par défaut), `auth` : `{"scheme": "basic"\|"digest", "username", "password"}` ou `{"scheme": "bearer", "token"}`. Renvoie le statut, la durée, les en-têtes et le corps — ses 16 premiers Kio. |
| `send_mqtt` | `broker` (`host:port`, port 1883 s’il n’est pas indiqué), `topic`, `payload`, `qos` (0, 1 ou 2), `retain`. Une charge utile vide avec `retain` efface une valeur retenue. |
| `send_ws` | `url` (`ws://` ou `wss://`), `text` ou `hex`, `headers`, `protocols`, et pour attendre la réponse `expect` (contient), `expect_regex` ou `wait` (n’importe quel message) ; `timeout_ms` de 1 à 120 000 (2000 par défaut). Renvoie la poignée de main, ce qui a été envoyé et la réponse, avec son JSON analysé quand c’en est. |

Ce sont les commandes qu’utilisent les écrans de l’application ; voir [`signallab send`](cli.md#cli-send).

### Écoute {#tools-listen}

`listen` ouvre un port UDP **sur la machine où tourne `signallab mcp`**, pendant
un moment, et renvoie ce qui est arrivé : les messages OSC décodés, les autres datagrammes en texte
et en hex.

| Argument | Ce que c’est | Par défaut |
| --- | --- | --- |
| `bind` | `IP:port`, p. ex. `0.0.0.0:9000`. | obligatoire |
| `protocol` | `osc` ou `udp`. | `osc` |
| `seconds` | Combien de temps écouter, de 0,1 à 60. | 5 |
| `max` | S’arrêter après ce nombre de datagrammes, de 1 à 1000. | 100 |

Quand rien n’arrive sur `0.0.0.0`, la réponse rappelle à l’assistant de vérifier
le pare-feu ([`signallab doctor`](cli.md#cli-doctor)). Avec `--server`,
`listen` est refusé : sur un serveur, c’est une expérience avec un nœud d’attente qui y écoute.

### Signaux et émulateurs {#tools-library}

`list_signals` et `fire_signal` utilisent votre bibliothèque de signaux — le
`signals.json` de l’application, `--library`, ou un chemin `library` donné à l’appel. Un signal est
envoyé par son identifiant ou son nom, exactement comme l’application l’envoie.

`list_emulators` nomme les émulateurs de votre bibliothèque. `start_emulator` en démarre
un — un document dans `emulator`, ou l’identifiant ou le nom d’une entrée de la bibliothèque dans `name` — et
renvoie son identifiant de tâche et son adresse ; il répond selon ses règles jusqu’à `stop_job`.
`bind` le déplace vers un autre `IP:port`, `params` donne des valeurs que lisent ses modèles,
`seed` fixe ses choix aléatoires. `emulator_exchanges` (`job_id`, et `after` pour
n’avoir que les plus récents) liste ce qui est arrivé et ce que chaque règle a répondu.
`set_emulator_down` (`job_id`, `down`, et `fault` : `unavailable`, `reset` ou
`timeout`) débranche un émulateur en cours jusqu’à ce qu’il soit remis en service :
HTTP rencontre la défaillance (`unavailable` répond 503), un appareil TCP et un broker MQTT
coupent les connexions, OSC et UDP ne répondent rien. Voir [Émulateurs](../tools/emulators.md).

### Exécutions et tâches {#tools-runs}

`list_runs` lit les rapports des exécutions précédentes, la plus récente en premier — d’une seule expérience
quand `experiment` la nomme, au plus `limit` (de 1 à 500, 50 par défaut) — avec les chiffres de chaque
étape de charge. `compare_runs` prend deux de leurs noms, `a` (avant) et
`b` (après), et met côte à côte les latences, le taux d’erreurs, le débit atteint et les
requêtes manquées de chaque étape de charge, en marquant comme régression un écart de 5 % ou plus
dans le mauvais sens. Voir [Exécutions et rapports](../experiments/runs.md).

`list_jobs` liste ce qui tourne — moniteurs, générateurs, émulateurs, exécutions —
et `stop_job` en arrête un par son identifiant.

## Résultats et erreurs {#results}

Chaque réponse est du texte pour le modèle et la même chose sous forme de données structurées. Un échec
est marqué comme erreur et porte l’erreur du moteur — un `code` stable, ses
valeurs, le nœud et le champ qu’elle concerne —, formulée dans la langue choisie avec
`--lang`. Les arguments que l’assistant a mal donnés lui reviennent avec des mots qui lui permettent de les corriger.

## Progression et annulation {#progress}

Quand le client demande la progression de `run_experiment`, chaque étape est signalée
au moment où elle se produit (le nœud et son état) : l’assistant — et vous — voyez l’exécution
avancer. Annuler un appel l’arrête ; annuler `run_experiment` arrête l’exécution
elle-même, comme le fait le bouton [[ui:common.stop]] dans l’application.

Quand le client ferme la connexion, les appels encore en cours se terminent, puis
`signallab mcp` quitte.

## Sur un serveur de laboratoire {#on-a-server}

Avec `--server http://192.0.2.10:1430`, les expériences, les envois, les signaux et les
émulateurs se passent **sur ce serveur**, via son API — avec son réseau, ses
secrets et son dossier de données — : l’assistant atteint ainsi des équipements que seul le laboratoire peut
atteindre. Donnez le jeton dans l’environnement du client :

```json
{
  "mcpServers": {
    "signallab": {
      "command": "signallab",
      "args": ["mcp", "--server", "http://192.0.2.10:1430"],
      "env": { "SIGNALLAB_TOKEN": "<the server's token>" }
    }
  }
}
```

Ce qui reste sur cette machine : les bibliothèques de signaux et d’émulateurs (celles de l’application, ou
`--library` et `--emulators`) et les fichiers qu’un appel nomme (`file`, `library`)
sont lus ici, et leur contenu est envoyé au serveur ; `listen` est refusé.
Voir [Exécuter Signal Lab comme serveur](../server/index.md).

## Sécurité {#safety}

- L’assistant ne peut faire que ce que font les outils, et chaque outil est l’une des
  commandes propres à l’application : il ne peut rien atteindre que l’application ne pourrait atteindre.
- Les outils qui envoient, écoutent ou démarrent quelque chose sont marqués comme atteignant le
  monde extérieur ; votre client décide s’il vous demande votre accord avant chaque appel.
- Les valeurs des secrets n’atteignent jamais l’assistant : une expérience les nomme sous la forme
  `{{secret.NAME}}`, et chaque résultat affiche `••••` à leur place.
- Un émulateur ou une écoute ouvre un port sur la machine où il tourne ;
  `list_jobs` et `stop_job` montrent et arrêtent ce qui tourne encore.

## Protocole {#protocol}

Pour les auteurs de clients : JSON-RPC 2.0 sur stdio, un message par ligne ; stdout
ne transporte que les messages du protocole, et tout ce qui s’adresse à une personne va sur stderr.
Versions du protocole `2025-06-18`, `2025-03-26` et `2024-11-05` (la plus récente quand
le client en demande une autre), lots, `ping`, `tools/list` et `tools/call` ;
progression via `notifications/progress` pour un appel qui a envoyé un `progressToken`,
annulation via `notifications/cancelled`. Les `instructions` du serveur indiquent au
modèle comment les outils s’articulent.

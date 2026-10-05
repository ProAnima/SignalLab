---
title: "Scanner"
description: "Trouvez quels ports TCP d’un hôte acceptent des connexions, avec le message d’accueil que chaque service envoie, grâce à une analyse de connexion TCP concurrente."
---

# Scanner

L’écran [[ui:nav.scan]] essaie d’ouvrir une connexion TCP vers chaque port d’une
plage sur un seul hôte et liste les ports qui acceptent. Utilisez-le pour confirmer
quels services écoutent réellement sur un appareil — le port de commande d’un
projecteur, l’interface web d’un commutateur, le port que votre propre service était
censé ouvrir — et, pour les services qui saluent en premier, ce qu’ils disent.

::: danger Usage responsable
Une analyse se connecte à chaque port de la plage. N’analysez que des hôtes que vous
possédez ou êtes autorisé à tester : sur d’autres réseaux, une analyse peut
déclencher la détection d’intrusion et est souvent contraire aux règles.
:::

## Analyser un hôte {#scan}

1. Saisissez l’[[ui:sc.host]] : un seul hôte, une adresse IP ou un nom.
2. Réglez [[ui:sc.fromPort]] et [[ui:sc.toPort]], ou choisissez un [[ui:sc.preset]].
3. Ajustez [[ui:common.concurrency]] et [[ui:common.timeoutMs]] si besoin.
4. Laissez [[ui:sc.grabBanner]] actif pour lire ce que chaque service dit en premier.
5. Appuyez sur [[ui:sc.startScan]].

Une barre sous le bouton indique combien de ports ont été essayés, sur combien, et
combien sont ouverts. Les ports ouverts apparaissent à droite au fur et à mesure
qu’ils sont trouvés. L’analyse se termine quand chaque port a été essayé ;
[[ui:sc.stopScan]] la termine plus tôt, et les ports pas encore essayés ne sont pas
essayés. Les champs sont fixés tant qu’elle tourne.

| Champ | Quoi | Par défaut |
| --- | --- | --- |
| [[ui:sc.host]] | L’hôte à analyser. | `127.0.0.1` |
| [[ui:sc.fromPort]], [[ui:sc.toPort]] | La plage, les deux bornes comprises, 1–65 535. Quand la première est plus grande, elles sont échangées. | 1–1024 |
| [[ui:common.concurrency]] | Combien de tentatives de connexion sont ouvertes en même temps, 1–1024. | 400 |
| [[ui:common.timeoutMs]] | Combien de temps attendre chaque connexion, 50–10 000 ms. | 500 |
| [[ui:sc.grabBanner]] | Après la connexion, attendre jusqu’à 400 ms que le service envoie quelque chose, et conserver ses 256 premiers octets. | on |

Un [[ui:common.concurrency]] ou un [[ui:common.timeoutMs]] hors de sa plage est
ramené dedans au démarrage de l’analyse.

[[ui:sc.preset]] remplit la plage :

| Préréglage | Ports |
| --- | --- |
| [[ui:sc.preset.wellKnown]] | 1–1024 |
| [[ui:sc.preset.common]] | 1–10 000 |
| [[ui:sc.preset.osc]] | 8000–9100 |
| [[ui:sc.preset.full]] | 1–65 535 |

## Combien de temps prend une analyse {#duration}

Un port qui accepte répond aussitôt. Un port qui n’accepte pas peut coûter jusqu’au
délai d’attente entier : un pare-feu qui abandonne les tentatives de connexion ne
répond jamais, et sous Windows, même un refus peut prendre plus que le délai par
défaut. Ainsi une analyse d’un hôte qui ne répond rien prend environ :

```text
ports ÷ concurrency × timeout
```

La plage complète aux valeurs par défaut : 65 535 ÷ 400 × 0,5 s ≈ 82 s. Augmentez
[[ui:common.concurrency]] ou baissez [[ui:common.timeoutMs]] pour aller plus vite ;
baissez trop le délai et les ports ouverts d’un hôte lent sont manqués.

## Lire les résultats {#results}

[[ui:sc.openPorts]] liste chaque port qui a accepté une connexion, dans l’ordre des
ports :

| Colonne | Quoi |
| --- | --- |
| [[ui:sc.port]] | Le port ouvert. |
| [[ui:common.time]] | Quand il a été trouvé. |
| [[ui:sc.banner]] | Ce que le service a envoyé en premier, les sauts de ligne changés en espaces, ou `—`. |

Un port qui a refusé, et un port qui n’a pas répondu dans le délai, sont tous deux
laissés de côté : le scanner ne distingue pas fermé de filtré. La liste conserve
jusqu’à 2000 ports ouverts.

Seuls les services qui parlent en premier ont une bannière — SSH, SMTP, FTP, beaucoup
de protocoles de commande d’appareils. Un serveur web attend une requête, si bien
que son port affiche `—`. La capture des bannières rallonge chaque port ouvert de
400 ms au plus.

Une connexion que le scanner ouvre est refermée aussitôt. Le scanner n’envoie rien
dessus.

Avec la capture active, chaque port ouvert est aussi dans
l’[Inspecteur](inspector.md), avec sa bannière et le verdict `open`.

## Voir aussi {#related}

- [Tempête](storm.md) — de la charge sur un port que vous avez trouvé.
- [UDP et TCP](../protocols/udp-tcp.md) — parlez-lui.
- [Dépannage](../reference/troubleshooting.md)

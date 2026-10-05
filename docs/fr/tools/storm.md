---
title: "Tempête"
description: "Envoyez un flot contrôlé de datagrammes UDP ou de connexions TCP vers un serveur que vous possédez, et suivez le débit, le volume et les erreurs en direct."
---

# Tempête

L’écran [[ui:nav.storm]] est une source de charge : il envoie des datagrammes UDP,
ou ouvre des connexions TCP, vers une cible aussi vite que vous le demandez, aussi
longtemps que vous le demandez, et mesure ce qui est réellement parti. Utilisez-le
pour voir comment votre propre serveur, appareil ou liaison tient sous un flot —
s’il continue de répondre, abandonne des paquets, ou tombe.

::: danger Usage responsable
Une tempête envoie du vrai trafic vers un vrai hôte. Ne la pointez que vers des
hôtes et des réseaux que vous possédez ou êtes autorisé à tester. Des débits élevés
peuvent saturer les liaisons pour tous ceux qui les utilisent et déclencher la
détection d’intrusion. Le moteur ne plafonne pas le débit d’une tempête : 0 signifie
aussi vite que cet ordinateur peut envoyer.
:::

## Démarrer une tempête {#start}

1. Réglez [[ui:common.target]] : l’`IP:port` ou `host:port` vers lequel envoyer,
   comme `127.0.0.1:9000` ou `test-rig.local:9000`. Un nom d’hôte est résolu au
   lancement, son adresse IPv4 prise quand il en a une.
2. Choisissez le [[ui:common.protocol]] : [[ui:st.udp]] ou [[ui:st.tcp]].
3. Réglez [[ui:st.payloadSize]], [[ui:st.rate]] et [[ui:st.duration]].
4. Appuyez sur [[ui:st.launch]].

La tempête tourne comme une tâche : elle apparaît dans le bandeau de la console, et
s’arrête quand sa durée est écoulée, quand vous appuyez sur [[ui:st.stop]], ou avec
[[ui:app.stopAll]]. Les champs sont fixés tant qu’elle tourne.

| Champ | Quoi | Par défaut |
| --- | --- | --- |
| [[ui:common.target]] | `IP:port` ou `host:port` de la cible. | `127.0.0.1:9000` |
| [[ui:common.protocol]] | [[ui:st.udp]] : des datagrammes séparés. [[ui:st.tcp]] : une nouvelle connexion TCP pour chaque « paquet ». | [[ui:st.udp]] |
| [[ui:st.payloadSize]] | Octets dans chaque datagramme ou connexion, 1–65 507. Une valeur plus grande ou plus petite est ramenée dans cette plage. | 512 |
| [[ui:st.rate]] | Paquets (ou connexions) par seconde visés. 0 : aussi vite que possible. | 1000 |
| [[ui:st.duration]] | Secondes d’exécution. 0 : jusqu’à ce que vous l’arrêtiez. | 10 |

### Flot UDP {#udp}

Signal Lab envoie des datagrammes de la taille de la charge utile, chaque octet
`0x55`, depuis un port qui lui est propre vers la cible. Un envoi que le système
refuse compte comme une erreur — par exemple après que la cible a répondu que rien
n’écoute sur ce port.

### Flot de connexions TCP {#tcp}

Pour chaque « paquet », Signal Lab ouvre une connexion TCP vers la cible, écrit la
charge utile, et la ferme. Les connexions sont faites l’une après l’autre, pas en
même temps, si bien que le débit qu’atteint une tempête TCP est limité par la
vitesse à laquelle la cible les accepte. Une connexion refusée, ou non acceptée dans
les 500 ms, compte comme une erreur ; une connexion qui a pris la charge utile
compte comme un paquet.

### Comment le débit est tenu {#pacing}

Le débit est un calendrier. Le premier paquet part aussitôt, et le paquet *n* est
dû *n* ÷ [[ui:st.rate]] secondes après le début. Chaque fois que la tempête se
réveille, elle envoie ce qui est dû, puis dort jusqu’à ce que le paquet suivant soit
dû. Ainsi 50 par seconde font 50 par seconde et 250 font 250 — une exécution d’une
seconde en envoie à peu près autant — quelle que soit la granularité du minuteur du
système. Une [[ui:st.duration]] termine la tempête à l’heure même quand le paquet
suivant serait dû plus tard.

Une tempête n’envoie jamais au-dessus de son débit pour rattraper le temps perdu. Si
cet ordinateur, ou une cible TCP qui accepte lentement, prend plus de 256 paquets de
retard, les plus anciens sont écartés du calendrier plutôt qu’envoyés en retard dans
une rafale. [[ui:st.pps]] montre ce qui a été atteint.

Avec [[ui:st.rate]] 0, il n’y a pas de calendrier : la tempête envoie 256 paquets,
laisse tourner les autres travaux, et envoie les 256 suivants, aussi vite que cet
ordinateur peut.

## Lire le débit {#metrics}

[[ui:st.throughput]] s’actualise quatre fois par seconde :

| Mesure | Quoi |
| --- | --- |
| [[ui:common.packets]] | Datagrammes envoyés, ou connexions qui ont pris la charge utile, depuis le début. |
| [[ui:st.pps]] | Paquets par seconde sur le dernier quart de seconde. |
| [[ui:st.rateLabel]] | Mégabits par seconde de charge utile sur le dernier quart de seconde. Les en-têtes (IP, UDP, TCP) ne sont pas comptés. |
| [[ui:common.volume]] | Octets de charge utile envoyés depuis le début. |
| [[ui:common.errors]] | Envois ou connexions qui ont échoué. |

Le graphique en dessous trace [[ui:st.pps]] sur la dernière minute.

Quand la tempête se termine, les compteurs affichent les totaux finaux, et
[[ui:st.pps]] et [[ui:st.rateLabel]] passent à 0.

Ce que les chiffres vous disent :

- [[ui:st.pps]] bien en dessous de [[ui:st.rate]] sur UDP : cet ordinateur ne peut
  pas envoyer plus vite. Baissez le débit ou la taille de la charge utile.
- [[ui:common.errors]] qui monte sur UDP : le port de la cible est fermé, ou le
  réseau refuse le trafic.
- [[ui:common.errors]] qui monte sur TCP : la cible refuse les connexions ou met
  plus de 500 ms à les accepter — elle a peut-être atteint sa limite.

Une tempête dit combien est parti, pas combien est arrivé. Pour voir ce que la cible
a reçu, observez-la : ses propres journaux, un moniteur [[ui:nav.osc]], ou un
[émulateur](emulators.md) à sa place.

## Dans l’Inspecteur {#inspector}

Avec la capture active, une tempête UDP place un de ses datagrammes dans
l’[Inspecteur](inspector.md) chaque seconde, marqué `sampled 1/s` — ils sont tous
identiques. Le verdict dit aussi combien ont été laissés de côté depuis le
précédent, sous la forme `sampled 1/s · +999 not shown`. Une tempête TCP n’en place
aucun.

## Voir aussi {#related}

- [Dégradation](impairment.md) — une liaison lente ou avec pertes au lieu d’un flot.
- [HTTP](../protocols/http.md) — des rafales de charge de requêtes HTTP, avec les
  latences.
- [Charge](../experiments/load.md) — la charge HTTP dans une expérience, avec des
  seuils.

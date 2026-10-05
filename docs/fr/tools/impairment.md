---
title: "Dégradation"
description: "Placez un relais entre un client et sa cible qui retarde, perd, duplique, réordonne ou limite les datagrammes UDP, ou retarde, limite, réinitialise ou bloque les flux TCP."
---

# Dégradation

L’écran [[ui:nav.netsim]] exécute un relais qui se place entre un client et la
cible à laquelle il parle, et dégrade ce qui le traverse dans les deux sens : un
mauvais Wi-Fi, une liaison mobile, un saut satellite, une liaison qui décroche.
Utilisez-le pour voir comment votre client et votre appareil se comportent sur un
réseau que vous n’avez pas sous la main.

Vous pointez le client vers le relais au lieu de la vraie cible ; le relais
transmet tout à la cible, et les réponses au client, après avoir fait à chaque
datagramme ou flux ce que dit son profil. Vous pouvez modifier le profil pendant
qu’il tourne, sans abandonner le port.

## Démarrer un relais {#start}

1. Choisissez le [[ui:ns.protocol]] : UDP pour les datagrammes (OSC, la plupart du
   trafic de contrôle de spectacle et de capteurs), TCP pour les flux (HTTP, MQTT,
   un appareil TCP).
2. Réglez [[ui:ns.listen]] : l’`IP:port` sur lequel le relais écoute. `0.0.0.0:9010`
   reçoit le trafic du réseau ; `127.0.0.1:9010` seulement de cet ordinateur.
3. Réglez [[ui:ns.target]] : l’`IP:port` de la vraie destination, ou un nom d’hôte
   et un port comme `device.local:9000`.
4. Choisissez un [préréglage](#presets) ou réglez les valeurs du [profil](#profile).
5. Appuyez sur [[ui:ns.startRelay]].
6. Pointez votre client sur le port du relais au lieu de la cible — par exemple
   `127.0.0.1:9010` au lieu de `127.0.0.1:9000`.

[[ui:ns.stopRelay]] ferme le relais ; rien de ce qu’il retenait encore ne part
après cela.

Valeurs par défaut : UDP, écoute sur `0.0.0.0:9010`, cible `127.0.0.1:9000`, 40 ms
de latence, 15 ms de gigue et 2 % de perte.

[[ui:ns.listen]] est une adresse à lier, toujours `IP:port`. Un nom d’hôte dans
[[ui:ns.target]] est résolu une fois, quand vous appuyez sur [[ui:ns.startRelay]],
comme une cible [OSC](../protocols/osc.md) ; un nom introuvable est refusé et aucun
relais ne démarre. [[ui:ns.protocol]], [[ui:ns.listen]] et [[ui:ns.target]] sont
fixés tant que le relais tourne ; arrêtez-le pour les modifier.

```text
client ──► relay (0.0.0.0:9010) ──► target (127.0.0.1:9000)
client ◄── relay ◄───────────────── target
```

### UDP {#udp}

Le relais envoie chaque datagramme à la cible depuis un port qui lui est propre, et
renvoie au client ce que la cible répond. Chaque datagramme connaît son propre
sort, décidé à son arrivée.

Les réponses vont au client qui a envoyé le datagramme le plus récent : le relais
sert un client à la fois.

### TCP {#tcp}

Chaque connexion qu’un client ouvre vers le relais est jointe à une nouvelle
connexion propre au relais vers la cible. Les deux flux de chaque connexion — client
vers cible et cible vers client — sont dégradés bloc par bloc, au fur et à mesure
que le relais les lit (jusqu’à 16 Kio à la fois). Un flux arrive toujours dans
l’ordre, quelle que soit la gigue.

Quand la cible refuse la connexion, la connexion du client est réinitialisée.

## Le profil {#profile}

Les valeurs qu’un relais applique, dans les deux sens. Un relais UDP lit les
valeurs du datagramme, un relais TCP celles du flux ; les autres sont masquées.

### Sur UDP {#profile-udp}

| Valeur | Ce qu’elle fait | Plage |
| --- | --- | --- |
| [[ui:ns.offline]] | Chaque datagramme est abandonné, dans les deux sens, jusqu’à ce que vous le décocherez. | on / off |
| [[ui:ns.latency]] | Ajoutée à chaque datagramme. | 0–1000 ms |
| [[ui:ns.jitter]] | Un délai supplémentaire aléatoire, de 0 à cette valeur, pour chaque datagramme — si bien que les datagrammes peuvent arriver dans le désordre. | 0–500 ms |
| [[ui:ns.loss]] | La part de datagrammes qui n’arrivent jamais, chacun pour son propre compte. | 0–100 % |
| [[ui:ns.burst]] | La probabilité qu’un datagramme déclenche une rafale de pertes : une liaison qui décroche un moment, contrairement à une perte dispersée. | 0–20 % |
| [[ui:ns.burstLength]] | Combien de datagrammes une rafale perd en moyenne. Affiché quand [[ui:ns.burst]] est supérieur à 0 ; 5 quand vous le remontez pour la première fois. | 1–1000 |
| [[ui:ns.duplicate]] | La part de datagrammes livrés deux fois. | 0–100 % |
| [[ui:ns.corrupt]] | La part de datagrammes avec un bit d’un octet inversé. | 0–100 % |
| [[ui:ns.reorder]] | La part de datagrammes retenus — de la latence, et au moins 20 ms — pour que ceux qui les suivent arrivent en premier. | 0–100 % |
| [[ui:ns.rate]] | Une limite de bande passante, en kilobits par seconde ; 0 signifie aucune. Les datagrammes font la queue pour la liaison à ce débit ; un datagramme qui attendrait plus d’une seconde est abandonné comme throttled. | 0, ou 8–10 000 000 |

Une rafale fonctionne ainsi : un datagramme qui n’est pas dans une rafale en
déclenche une avec la probabilité [[ui:ns.burst]] ; chaque datagramme d’une rafale
est perdu, et chacun la termine avec une probabilité de 1 sur [[ui:ns.burstLength]],
si bien qu’une rafale dure ce nombre de datagrammes en moyenne.

Un datagramme est décidé dans cet ordre : hors ligne, rafale, perte, bande passante,
puis duplication, corruption, délai et réordonnancement pour chaque copie.

### Sur TCP {#profile-tcp}

| Valeur | Ce qu’elle fait | Plage |
| --- | --- | --- |
| [[ui:ns.offline]] | Rien ne circule, dans aucun sens, et les nouvelles connexions attendent la cible, jusqu’à ce que vous le décocherez — puis tout repart. | on / off |
| [[ui:ns.latency]] | Ajoutée à chaque bloc d’un flux, dans les deux sens. | 0–1000 ms |
| [[ui:ns.jitter]] | Un délai supplémentaire aléatoire, de 0 à cette valeur, pour chaque bloc — jamais en avance sur le bloc qui le précède. | 0–500 ms |
| [[ui:ns.reset]] | La part de blocs qui réinitialisent leur connexion au lieu de passer : le client et la cible reçoivent tous deux une réinitialisation. | 0–100 % |
| [[ui:ns.stall]] | La part de blocs qui laissent leur connexion à demi ouverte : plus rien ne passe, dans aucun sens, et aucun des deux côtés n’est prévenu. Le relais garde les deux côtés ouverts, intacts, jusqu’à son arrêt. | 0–100 % |
| [[ui:ns.rate]] | Une limite de bande passante pour chaque flux de chaque connexion, en kilobits par seconde ; 0 signifie aucune. Au-delà d’une seconde de file, le relais cesse de lire, si bien que l’émetteur ralentit, comme sur une liaison lente. Rien n’est abandonné. | 0, ou 8–10 000 000 |

La perte, les rafales, la duplication, la corruption et le réordonnancement ne
s’appliquent pas à TCP : un vrai flux TCP retransmet ce qu’il perd et se remet en
ordre, si bien que ce qu’un client rencontre sur une mauvaise liaison, c’est du
délai, un émetteur lent, des réinitialisations et des connexions qui cessent de
répondre.

::: tip
Les curseurs s’arrêtent à 1000 ms de latence et 500 ms de gigue. Le relais lui-même
accepte jusqu’à 60 000 ms de chacune, par exemple depuis un fichier d’expérience.
:::

## Préréglages {#presets}

Un préréglage règle chaque valeur en un clic. Sa pastille reste allumée tant que les
valeurs sont encore celles du préréglage ; modifiez une valeur et elle s’éteint.

| Préréglage | Sur UDP | Sur TCP |
| --- | --- | --- |
| [[ui:ns.preset.lan]] | 1 ms ±1 | 1 ms ±1 |
| [[ui:ns.preset.wifi]] | 20 ms ±30, 1 % de perte, rafales 1 % × 3, 0,5 % dupliqués, 2 % réordonnés | 20 ms ±30 |
| [[ui:ns.preset.4g]] | 60 ms ±25, 0,5 % de perte, 0,5 % réordonnés, 20 000 kbit/s | 60 ms ±25, 20 000 kbit/s |
| [[ui:ns.preset.satellite]] | 300 ms ±30, 1 % de perte, 2000 kbit/s | 300 ms ±30, 2000 kbit/s |
| [[ui:ns.preset.intermittent]] | 30 ms ±20, rafales 3 % × 15 | 30 ms ±20, 0,2 % de blocs laissés à demi ouverts |
| [[ui:ns.preset.offline]] | Rien ne passe | Rien ne circule |

## Le modifier pendant qu’il tourne {#live}

Modifiez n’importe quelle valeur, ou choisissez un autre préréglage, pendant que le
relais tourne : il s’applique un quart de seconde après votre dernière modification,
sans abandonner le port ni les connexions. La console note chaque nouveau profil, et
la ligne sous [[ui:ns.live]] dit ce que le relais fait maintenant — le nom du
préréglage, ou les valeurs en bref, comme `60 ms ±25 · loss 2% · 20000 kbps`.

Un datagramme ou un bloc est décidé par le profil en vigueur quand il arrive ; un
élément déjà en route garde son délai.

## Les compteurs {#counters}

[[ui:ns.live]] montre ce que le relais a fait depuis son démarrage, les deux sens
réunis, actualisé quatre fois par seconde.

Sur UDP :

| Compteur | Quoi |
| --- | --- |
| [[ui:ns.received]] | Datagrammes parvenus au relais. |
| [[ui:ns.forwarded]] | Datagrammes transmis ; un datagramme dupliqué compte deux fois. |
| [[ui:ns.dropped]] | Perdus exprès : hors ligne, une rafale, ou une perte. |
| [[ui:ns.throttled]] | Abandonnés par la limite de bande passante, ou parce que 10 000 étaient déjà en route. |
| [[ui:ns.duplicated]] | Datagrammes envoyés deux fois. |
| [[ui:ns.corrupted]] | Copies avec un bit inversé. |
| [[ui:ns.reordered]] | Copies retenues pour que de plus récentes les dépassent. |
| [[ui:common.volume]] | Octets transmis. |

Sur TCP :

| Compteur | Quoi |
| --- | --- |
| [[ui:ns.connections]] | Connexions ouvertes par des clients vers le relais. |
| [[ui:ns.received]] | Blocs lus par le relais, dans les deux sens. |
| [[ui:ns.forwarded]] | Blocs qu’il a écrits. |
| [[ui:ns.resets]] | Connexions réinitialisées. |
| [[ui:ns.stalled]] | Connexions laissées à demi ouvertes. |
| [[ui:ns.held]] | Combien de fois un flux a attendu la limite de bande passante : l’émetteur a été ralenti, rien n’a été abandonné. |
| [[ui:common.volume]] | Octets écrits. |

Avec la capture active, l’[Inspecteur](inspector.md) montre les datagrammes et
blocs relayés — au plus un toutes les 25 ms, les deux sens réunis — chacun avec son
sort : `forwarded +43ms`, `· corrupted`, `· reordered`, `· copy 2/2`,
`dropped (loss)`, `dropped (burst)`, `dropped (offline)`, `throttled`, et sur TCP
`reset` et `half-open`. → va du client à la cible, ← de la cible au client.

Une trame relayée nomme de vraies sockets : [[ui:ins.local]] est l’adresse sur
laquelle le relais écoute et [[ui:bc.peer]] est où allait ce datagramme ou ce bloc —
la cible, ou le client à qui une réponse est revenue. Le trajet termine le verdict :
`forwarded +43ms · client→target`, `dropped (loss) · target→client`. Ainsi un
datagramme relayé peut être conservé avec [[ui:sig.fromFrame]] et est visé vers
l’adresse vers laquelle il allait ; un bloc TCP est un morceau d’un flux et ne peut
pas l’être (voir [Inspecteur](inspector.md#save-as-signal)).

## Le même trafic, le même sort {#seed}

Chaque décision — quel datagramme est perdu, retardé de combien, corrompu où — est
tirée de la graine du relais, séparément pour chaque sens et, sur TCP, pour chaque
connexion. Avec la même graine et le même trafic, le relais abandonne les mêmes
datagrammes et les retarde de la même façon.

L’écran [[ui:nav.netsim]] prend une nouvelle graine à chaque démarrage d’un relais.
Pour répéter une exécution exactement, utilisez un nœud [[ui:exp.node.impairment]]
dans une expérience : il tire de la graine de l’exécution, que vous pouvez épingler
(voir [Répéter une exécution défaillante](../experiments/faults.md#seed)).

## Dans les expériences {#experiments}

Deux nœuds placent le même relais dans une expérience :

- [[ui:exp.node.impairment]] ouvre un relais avant la première étape et le ferme
  quand l’exécution se termine, quelle que soit la façon. Ses adresses d’écoute et
  de cible n’acceptent que des paramètres ; la cible peut être un nom d’hôte, résolu
  au démarrage de l’exécution. Voir
  [Nœuds](../experiments/nodes.md#node-impairment).
- [[ui:exp.node.impairment_change]] bascule un relais de l’exécution vers un autre
  profil à partir de cette étape — propre, avec pertes, hors ligne, de nouveau
  propre — et le rapport de l’exécution compte ce que chaque phase a fait. Voir
  [Nœuds](../experiments/nodes.md#node-impairment_change).

Sur un nœud OSC ou UDP, [[ui:exp.routeThrough]] place un
[[ui:exp.node.impairment]] devant lui et pointe le nœud vers le relais. Voir
[Défaillances](../experiments/faults.md).

## Voir aussi {#related}

- [Émulateurs](emulators.md) — la cible à placer derrière le relais.
- [Inspecteur](inspector.md) — le sort de chaque datagramme.
- [UDP et TCP](../protocols/udp-tcp.md)

---
title: "Tests de charge"
description: "Envoyer la requête d’un nœud HTTP selon un profil de charge — constant, rampe, paliers, pic ou arrivées aléatoires —, mesurer les latences, les erreurs et le débit atteint, les juger avec des seuils et comparer deux exécutions."
---

# Tester une requête HTTP sous charge

Un nœud [[ui:exp.node.http]] peut envoyer sa requête de nombreuses fois, selon un
profil de requêtes par seconde, beaucoup en même temps — et mesurer ce qui revient :
percentiles de latence, erreurs, débit atteint. Des seuils décident si
l’étape réussit, et le bouton [[ui:exp.compare]] place les chiffres à côté de ceux d’une exécution précédente.

Une charge est un réglage du nœud, pas un nœud à part : le reste de
l’expérience — émulateurs, relais de dégradation, autres branches — s’exécute autour d’elle comme
d’habitude.

## Mettre une requête sous charge {#turn-on}

1. Sélectionnez un nœud [[ui:exp.node.http]] et remplissez sa requête.
2. Dans ses propriétés, activez [[ui:exp.loadOn]].
3. Choisissez un [[ui:exp.loadShape]] et ses valeurs. Le graphique en dessous,
   [[ui:exp.loadChart]], trace le débit et indique à combien de requêtes il correspond,
   et en combien de secondes.
4. Réglez le champ [[ui:exp.loadConcurrency]] — combien de requêtes peuvent être en cours en même temps.
5. Ajoutez ou modifiez des [[ui:exp.thresholds]].
6. Exécutez l’expérience.

Une nouvelle charge est une [[ui:exp.loadShape.ramp]] de 0 à 100 requêtes par
seconde sur 30 000 ms, 32 en parallèle, avec deux seuils :
[[ui:exp.metric.p95_ms]] < 500 ms et [[ui:exp.metric.error_rate]] < 1 %.

**La charge remplace la répétition et le réessai.** L’activer les désactive, et un nœud
qui a une charge et l’un d’eux est refusé (`node.load_alone`) : une requête échouée
est comptée, pas retentée. Seule une requête HTTP peut s’exécuter sous charge
(`node.load_unsupported`).

**La requête est lue une seule fois.** Ses modèles sont résolus au démarrage de l’étape :
chaque requête de la charge est donc la même — `{{counter}}` et `{{uuid}}` prennent
une seule valeur pour toutes. Voir [modèles](data.md#templates).

**Un seul client pour toute la charge.** Les requêtes partagent la réserve de cookies de l’exécution quand
l’option [[ui:exp.cookies]] est cochée, et une seule mémoire Digest : un seul challenge leur répond
à toutes. Chaque requête a le délai d’attente propre au nœud.

**L’Inspecteur reçoit un échantillon :** au plus un échange toutes les 100 ms, pour qu’une charge
n’inonde pas l’onglet [[ui:dock.inspector]].

## Profils {#profiles}

| [[ui:exp.loadShape]] | Réglages | Le débit dans le temps |
| --- | --- | --- |
| [[ui:exp.loadShape.constant]] | [[ui:exp.loadRate]], [[ui:exp.loadDuration]] | le débit d’un bout à l’autre |
| [[ui:exp.loadShape.ramp]] | [[ui:exp.loadFrom]], [[ui:exp.loadTo]], [[ui:exp.loadDuration]] | en ligne droite d’un débit à l’autre |
| [[ui:exp.loadShape.steps]] | [[ui:exp.loadFrom]], [[ui:exp.loadStepBy]], [[ui:exp.loadEvery]], [[ui:exp.loadSteps]] | le premier débit, puis un palier de plus à chaque niveau, chaque niveau pendant la même durée |
| [[ui:exp.loadShape.spike]] | [[ui:exp.loadBase]], [[ui:exp.loadPeak]], [[ui:exp.loadAt]], [[ui:exp.loadSpikeFor]], [[ui:exp.loadDuration]] | le débit de base, la pointe pendant un moment à partir d’un instant donné, puis de nouveau le débit de base |
| [[ui:exp.loadShape.poisson]] | [[ui:exp.loadRate]], [[ui:exp.loadDuration]] | des arrivées au hasard, le débit en moyenne |

Changer de forme conserve ce qui peut l’être : la durée et le débit
le plus élevé atteint.

### Limites {#limits}

| Réglage | Plage |
| --- | --- |
| [[ui:exp.loadRate]] de [[ui:exp.loadShape.constant]] et [[ui:exp.loadShape.poisson]], [[ui:exp.loadPeak]] | 0,1–100 000 requêtes/s |
| [[ui:exp.loadFrom]], [[ui:exp.loadTo]], [[ui:exp.loadBase]] | 0–100 000 requêtes/s |
| Chaque niveau de [[ui:exp.loadShape.steps]], le dernier compris | 0–100 000 requêtes/s ; le palier peut être négatif |
| [[ui:exp.loadDuration]], [[ui:exp.loadEvery]] | 100–300 000 ms |
| [[ui:exp.loadSteps]] | 1–100, et tous les niveaux ensemble 300 000 ms au plus |
| Un pic | plus long que 0 ms, et terminé avant la fin de la durée |
| [[ui:exp.loadConcurrency]] | 1–512 |
| [[ui:exp.thresholds]] | 16 au plus, chaque valeur un nombre, 0 ou plus |

Un profil qui ne donne aucune requête est refusé
(`load.nothing_planned`). Les débits sont ceux de la
[rafale HTTP](../protocols/http.md).

Un profil peut durer aussi longtemps qu’une exécution entière, 300 s — mais la
[durée limite](flow.md#limit) de l’exécution compte chaque étape : laissez de la place pour le reste de
l’expérience.

### Combien de requêtes {#planned}

Les requêtes d’un profil sont son débit cumulé dans le temps :

| Profil | Requêtes |
| --- | --- |
| [[ui:exp.loadShape.constant]], 100/s pendant 1000 ms | 100 |
| [[ui:exp.loadShape.ramp]], 0 → 100/s sur 2000 ms | 100 |
| [[ui:exp.loadShape.steps]], à partir de 10/s, +10/s par palier, 3 niveaux de 1000 ms | 60 (10 + 20 + 30) |
| [[ui:exp.loadShape.spike]], 10/s avec 100/s à partir de 1000 ms pendant 500 ms, 2000 ms en tout | 65 |
| [[ui:exp.loadShape.poisson]], 200/s pendant 10 000 ms | 2000 en moyenne |

## Le calendrier {#schedule}

La n-ième requête est due à l’instant où le cumul du profil atteint n — la première
tout de suite. Chaque instant est calculé à partir du début de la charge : un réveil
tardif ne décale donc jamais les requêtes suivantes, et le débit que décrit le profil
est bien le débit demandé.

[[ui:exp.loadShape.poisson]] tire au hasard les intervalles entre les arrivées, à partir de
la graine de l’exécution : la même graine donne les mêmes instants, si bien qu’une charge aléatoire peut être
reproduite exactement. Voir [graines](runs.md#seeds).

**Requêtes manquées.** Au plus [[ui:exp.loadConcurrency]] requêtes sont en cours.
Quand toutes attendent encore leur réponse, la requête suivante attend
qu’une place se libère. Si elle devait partir plus de 50 ms après son instant, elle n’est pas
envoyée en retard : elle est sautée et comptée comme manquée, avec toutes les autres requêtes
devenues dues entre-temps, et la charge continue avec la première encore à l’heure. Beaucoup
de requêtes manquées signifient que le serveur, ou [[ui:exp.loadConcurrency]], n’a pas pu suivre
le profil.

## Pendant l’exécution {#progress}

Une fois par seconde, la chronologie affiche l’étape avec l’état [[ui:exp.load]], les secondes
écoulées, les requêtes envoyées, le débit de la dernière seconde, le p95 jusqu’ici et les
requêtes échouées. Le bouton [[ui:common.stop]] met fin à la charge immédiatement et abandonne les
requêtes en cours ; un échec dans une autre branche y met fin en moins d’une seconde.

## Ce qui est mesuré {#metrics}

Après la dernière réponse, l’étape dispose de ses mesures, conservées dans son dernier
événement de la chronologie et dans le [rapport d’exécution](runs.md#report) :

| Mesure | Quoi |
| --- | --- |
| planned | les requêtes auxquelles correspond le profil ([[ui:exp.loadShape.poisson]] : en moyenne) |
| sent | les requêtes qui ont reçu une réponse ou ont échoué |
| ok | celles qui ont reçu une réponse de statut 2xx |
| failed | tout autre statut, ou aucune réponse |
| missed | dues alors que toutes les places étaient occupées, et sautées |
| rps | requêtes envoyées par seconde : sent ÷ la durée du profil — ou ÷ le temps jusqu’au départ de la dernière requête, s’il est plus long |
| error_rate | failed, en % de sent |
| min, mean, max | la requête la plus rapide, la moyenne et la plus lente, ms |
| p50, p90, p95, p99 | la latence à laquelle ou sous laquelle se trouvaient 50, 90, 95 et 99 % des requêtes, ms |
| received_bytes | les octets de corps reçus au total |
| statuses | les requêtes par statut (`200`, `503`) et, faute de statut, par cause (`timeout`, `refused`, `reset` …) |
| seconds | chaque seconde du profil : requêtes envoyées, échouées, leur latence moyenne |
| histogram | les requêtes par latence, jusqu’à 1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10 000 ms, et au-delà |

La latence d’une requête va de son envoi à la lecture complète de sa réponse, et
une requête échouée compte avec le temps qu’elle a mis à échouer. Les percentiles sont lus
dans des classes logarithmiques larges de 1 % et sont exacts à 0,5 % près,
quelle que soit la durée de la charge.

## Seuils {#thresholds}

Un seuil est une ligne composée d’une [[ui:exp.thresholdMetric]], d’une [[ui:exp.thresholdOp]] et
d’une [[ui:exp.thresholdValue]] ; le bouton [[ui:exp.thresholdAdd]] en ajoute un.

| [[ui:exp.thresholdMetric]] | Unité |
| --- | --- |
| [[ui:exp.metric.p50_ms]], [[ui:exp.metric.p90_ms]], [[ui:exp.metric.p95_ms]], [[ui:exp.metric.p99_ms]] | ms |
| [[ui:exp.metric.mean_ms]], [[ui:exp.metric.max_ms]] | ms |
| [[ui:exp.metric.error_rate]] | % des requêtes envoyées |
| [[ui:exp.metric.rps]] | requêtes par seconde atteintes |
| [[ui:exp.metric.missed]] | requêtes |

La [[ui:exp.thresholdOp]] est l’un des opérateurs `<`, `≤`, `>`, `≥`. Quelques seuils courants :

| [[ui:exp.thresholdMetric]] | [[ui:exp.thresholdOp]] | [[ui:exp.thresholdValue]] | L’étape échoue quand |
| --- | --- | --- | --- |
| [[ui:exp.metric.p95_ms]] | `<` | 300 | une requête sur vingt ou plus a pris 300 ms ou davantage |
| [[ui:exp.metric.error_rate]] | `<` | 1 | 1 % des requêtes ou plus ont échoué |
| [[ui:exp.metric.rps]] | `≥` | 180 | le serveur n’a pas pu absorber 180 requêtes par seconde |
| [[ui:exp.metric.missed]] | `≤` | 0 | une seule requête a dû être sautée |

Dans un fichier, un seuil s’écrit `{ "metric": "p95_ms", "op": "lt", "value": 300 }` ;
les mesures sont `p50_ms`, `p90_ms`, `p95_ms`, `p99_ms`, `mean_ms`, `max_ms`,
`error_rate`, `rps` et `missed`, les comparaisons `lt`, `le`, `gt` et `ge`.

Les seuils sont lus après la dernière réponse, dans leur ordre. L’étape échoue
sur le premier qui n’est pas respecté (`load.threshold`), son message donnant le
seuil et la valeur mesurée, et l’exécution échoue avec elle. Sans seuils, une charge réussit quoi qu’elle ait mesuré. Quand
l’échec d’une autre branche a mis fin à la charge plus tôt, c’est cet échec qui fait échouer l’exécution, pas
un seuil.

## Le résultat {#result}

Quand l’étape réussit, la chronologie la résume : les requêtes, le débit, le p95
et la part d’échecs. Sélectionnez le nœud : ses propriétés affichent
[[ui:exp.loadResult]] —

- chaque seuil, ✓ [[ui:exp.thresholdHeld]] ou ✕ [[ui:exp.thresholdBroken]],
  avec la valeur mesurée ;
- [[ui:http.sent]], [[ui:exp.loadRps]], [[ui:exp.loadErrors]] avec leur part,
  [[ui:http.missed]] ;
- [[ui:http.p50]], [[ui:http.p90]], [[ui:http.p95]], [[ui:http.p99]],
  [[ui:http.avg]], [[ui:http.max]] ;
- [[ui:exp.loadPerSecond]] : les requêtes de chaque seconde, les échouées en
  rouge, et leur latence moyenne sous forme de courbe ;
- [[ui:exp.loadLatencies]] : combien de requêtes ont pris combien de temps ;
- les statuts et les causes, chacun avec son nombre.

La ligne de commande affiche les mêmes chiffres et le verdict de chaque seuil ; voir
[`signallab run`](../automation/cli.md#cli-run).

## Comparer deux exécutions {#compare}

1. Exécutez l’expérience deux fois, ou plus.
2. Dans la chronologie, appuyez sur [[ui:exp.compare]]. Le bouton est là dès qu’une exécution a enregistré
   son rapport, et il est désactivé pendant qu’une exécution est en cours.
3. La dernière exécution est dans [[ui:exp.compareAfter]], la précédente dans
   [[ui:exp.compareBefore]] ; chacune des deux listes permet d’en choisir une autre.

Les listes contiennent les exécutions de cette expérience — d’après son nom — tirées des rapports du
dossier de données, la plus récente en premier, 50 au plus : chacune avec sa date et son heure, sa façon de
se terminer et sa graine. Les exécutions lancées depuis la ligne de commande y figurent aussi si elle a utilisé le
même dossier de données. Renommer l’expérience démarre un nouvel historique.

Pour chaque étape de charge, appariée par nœud, un tableau donne chaque mesure dans les colonnes
[[ui:exp.compareBefore]], [[ui:exp.compareAfter]] et
[[ui:exp.compareChange]], dans l’unité et en %. Un écart de 5 % ou plus dans le mauvais sens
— plus lent, plus d’erreurs, plus de requêtes manquées, un débit plus faible — est une
régression et s’affiche en rouge ; passer de rien à quelque chose compte aussi. Sous le
tableau, le verdict de chaque seuil dans les deux exécutions. Une étape de charge que seule l’une des
exécutions possède est marquée [[ui:exp.compareOnlyBefore]] ou [[ui:exp.compareOnlyAfter]],
sans écarts. Pour des exécutions sans étape de charge, le tableau indique [[ui:exp.compareNoLoad]].

Depuis un script, [`experiment_runs`](../api/commands.md#experiment_runs) liste les
exécutions et [`experiment_compare`](../api/commands.md#experiment_compare) en compare
deux, d’après le nom de fichier de leur rapport ; `signallab mcp` offre la même chose à un
assistant ([MCP](../automation/mcp.md)).

## Vérifications après une charge {#checks-after}

Une charge ne laisse pas de réponse à elle : elle est mesurée, pas vérifiée. Une vérification ou un nœud
[[ui:exp.node.extract]] placé après elle a besoin d’une autre requête sans charge avant lui
sur chaque chemin, sinon l’expérience ne s’exécute pas (`graph.needs_http`). Pour vérifier
une réponse de l’API sous charge, placez un nœud [[ui:exp.node.http]] ordinaire après la
charge, ou dans une branche parallèle à côté d’elle.

Le bouton [[ui:exp.sendNow]] sur un nœud sous charge envoie sa requête une seule fois.

---
title: "Données et modèles"
description: "Paramètres et profils, le langage de modèles avec ses générateurs, les valeurs extraites des réponses, les comparaisons, et les secrets qui ne quittent jamais le moteur."
---

# Les données dans les expériences

Des valeurs circulent dans une exécution : un paramètre choisit la cible, un champ d’une
réponse devient un en-tête de la requête suivante, un identifiant généré part dans une
commande et revient dans une vérification. Cette page explique d’où viennent ces valeurs
et comment un champ les utilise.

| Source | Écrit sous la forme | Défini où |
| --- | --- | --- |
| Paramètre | `{{api}}` ou `{{params.api}}` | volet [[ui:exp.params]], un profil, [[ui:exp.runWith]] |
| Variable | `{{token}}` ou `{{vars.token}}` | un nœud pendant l’exécution : [[ui:exp.node.extract]], une attente, un envoi qui attend une réponse |
| Secret | `{{secret.API_TOKEN}}` | le magasin d’identifiants de l’ordinateur, ou l’environnement et les fichiers du serveur |
| Valeur intégrée | `{{run.seed}}`, `{{now.iso}}`, `{{counter}}` | l’exécution elle-même |
| Générateur | `{{uuid}}`, `{{random_int(1, 100)}}` | tiré de la graine de l’exécution |

## Paramètres {#parameters}

Un paramètre est une valeur texte nommée que n’importe quel champ modélisable peut utiliser. Gardez
les cibles dans des paramètres, pour qu’un changement d’adresse soit une seule modification au lieu d’une par
nœud.

### Ajouter un paramètre {#add-parameter}

1. Appuyez sur [[ui:exp.params]] (`{ }`) dans la barre d’outils de l’éditeur.
2. Dans l’onglet [[ui:exp.noProfile]], appuyez sur [[ui:exp.addParam]].
3. Saisissez le [[ui:exp.paramName]] et la [[ui:exp.paramValue]], par exemple
   `api` et `http://127.0.0.1:8080`.
4. Dans le champ d’un nœud, écrivez `{{api}}/login`.

Chaque changement dans le volet est une modification de l’expérience : il est enregistré avec elle
et annulé avec <kbd>Ctrl</kbd>+<kbd>Z</kbd> comme n’importe quel autre.

### Règles {#parameter-rules}

| Règle | Limite |
| --- | --- |
| Nom | commence par une lettre ou `_`, puis des lettres, des chiffres et `_` |
| Noms réservés | `vars`, `params`, `secret`, `run`, `node`, `now`, `uuid`, `counter`, `random_int`, `random_float`, `pick` |
| Paramètres par expérience | 64 |
| Taille d’une valeur | 64 Kio |
| Noms | uniques ; une variable ne peut pas avoir le nom d’un paramètre |

Une valeur est du texte brut et est insérée telle qu’écrite : `{{…}}` à l’intérieur d’une valeur n’est
pas résolu. Quand un champ demande une partie d’un paramètre
(`{{config.ports[0]}}`), la valeur est lue comme du JSON ; une valeur qui n’est pas du JSON
n’a pas de parties.

Un paramètre dont le nom est invalide, réservé ou pris deux fois n’empêche pas
l’expérience d’être enregistrée, pour que vous puissiez continuer à saisir ; l’expérience ne s’exécute pas
tant que le nom n’est pas corrigé.

## Profils {#profiles}

Un profil est un ensemble nommé de valeurs de paramètres — *Portable*, *Scène*, *Salle* —, pour que
changer de cible soit un choix, et non une modification de chaque nœud. Un profil
change certains paramètres ; les autres gardent leur valeur par défaut.

### Créer un profil {#make-profile}

1. Ouvrez [[ui:exp.params]] et appuyez sur [[ui:exp.addProfile]]. Un nouvel onglet s’ouvre.
2. Renommez-le dans [[ui:exp.profileName]].
3. Pour chaque paramètre que le profil change, saisissez sa valeur. Un champ vide garde
   la valeur par défaut, affichée en gris dans le champ ; [[ui:exp.resetToDefault]] (↺) efface
   une valeur.
4. Appuyez sur [[ui:exp.makeActive]] pour exécuter avec lui. L’onglet du profil actif porte
   ● [[ui:exp.activeProfile]]. [[ui:exp.makeActive]] sur l’onglet
   [[ui:exp.noProfile]] revient aux valeurs par défaut.

Une fois qu’une expérience a des profils, une liste [[ui:exp.profile]] dans la barre d’outils
permet de passer de l’un à l’autre. Le profil actif est utilisé par les exécutions, l’aperçu et
[[ui:exp.sendNow]], et il est enregistré dans l’expérience, si bien qu’un fichier exporté s’ouvre
avec les mêmes cibles. [[ui:exp.removeProfile]] supprime le profil affiché.

| Règle | Limite |
| --- | --- |
| Profils par expérience | 32 |
| Nom | 1–64 caractères, unique (les espaces aux extrémités ne comptent pas) |
| Valeurs | uniquement des paramètres qui existent ; 64 au plus |

Renommer ou supprimer un paramètre le change dans tous les profils à la fois.

### Quelle valeur une exécution utilise {#precedence}

La dernière gagne :

1. la valeur par défaut du paramètre, dans l’onglet [[ui:exp.noProfile]] ;
2. la valeur du profil actif, s’il en définit une ;
3. une valeur saisie dans [[ui:exp.runWith]] pour cette exécution seulement — voir
   [exécuter avec d’autres valeurs](runs.md#run-with).

[[ui:exp.runWith]] ne peut définir que des paramètres que l’expérience possède. Le rapport d’exécution
enregistre le profil, les valeurs saisies pour l’exécution et chaque valeur qu’il a utilisée.

### Profils qui ne s’exécuteraient pas {#profile-issues}

Chaque fois que l’expérience est vérifiée, les autres profils et les valeurs par défaut sont
vérifiés aussi. Un profil qui échouerait — disons, une URL qui n’est pas `http://` ou
`https://` — porte ⚠ dans les onglets et dans la liste de la barre d’outils, et son info-bulle
en dit la raison. Il n’empêche pas les exécutions avec le profil utilisé.

## Modèles {#templates}

Le texte à l’intérieur de `{{ }}` est une expression ; tout le reste dans un champ est conservé
exactement tel qu’écrit.

```text
{{api}}/users/{{user.id}}?trace={{uuid}}
Bearer {{secret.API_TOKEN}}
```

- Les espaces à l’intérieur des accolades n’ont pas d’importance : `{{ token }}` est `{{token}}`.
- `\{{` écrit un `{{` littéral.
- Un `}}` seul est du texte brut.
- Une valeur est insérée telle quelle, sans guillemets. Dans un corps JSON, écrivez les
  guillemets vous-même : `"id": "{{uuid}}"`.

### Noms {#names}

| Expression | Valeur |
| --- | --- |
| `{{name}}` | la variable `name` si elle est définie sur ce chemin, sinon le paramètre `name` |
| `{{vars.name}}` | la variable seulement |
| `{{params.name}}` | le paramètre seulement |
| `{{secret.NAME}}` | le secret stocké `NAME` — voir [secrets](#secrets) |
| `{{name.field}}` | un champ d’une valeur JSON |
| `{{name[0]}}` | un élément d’un tableau JSON |
| `{{name["a b"]}}`, `{{name['a b']}}` | un champ dont le nom a d’autres caractères |

Un nom de champ après `.` peut contenir des lettres, des chiffres, `_` et `-`. Les étapes s’enchaînent :
`{{reply.args[0]}}`, `{{order.items[2].sku}}`.

### Comment les valeurs sont écrites {#value-text}

| Valeur | Écrite sous la forme |
| --- | --- |
| texte | le texte |
| nombre | sa forme la plus courte : `42`, `0.5` |
| `true`, `false` | `true`, `false` |
| `null` | `null` |
| objet, tableau | JSON compact : `["x","y"]` |

### Valeurs intégrées {#built-ins}

| Expression | Valeur |
| --- | --- |
| `{{run.id}}` | le numéro de tâche de l’exécution ; `0` dans l’aperçu et dans [[ui:exp.sendNow]] |
| `{{run.seed}}` | la graine de cette exécution |
| `{{node.id}}` | l’identifiant du nœud en cours d’exécution |
| `{{now}}` | l’heure actuelle, en millisecondes Unix |
| `{{now.iso}}` | l’heure actuelle en UTC, ISO 8601 avec millisecondes : `2026-09-30T12:34:56.789Z` |
| `{{counter}}` | combien de fois ce nœud s’est exécuté dans cette exécution, cette fois comprise, à partir de 1 |

`{{counter}}` compte par nœud : dans le corps d’une [Boucle](flow.md#loop) c’est le
numéro de l’itération, dans un nœud qui [répète](flow.md#repeat) le numéro de l’envoi.
`run`, `node` et `now` n’ont que les champs listés ; tout autre est une erreur.

### Générateurs {#generators}

| Expression | Valeur |
| --- | --- |
| `{{uuid}}` ou `{{uuid()}}` | un UUID de version 4 |
| `{{random_int(min, max)}}` | un nombre entier de `min` à `max`, tous deux compris ; arguments entiers, `min` ≤ `max` |
| `{{random_float(min, max)}}` | un nombre de `min` jusqu’à `max` exclu, avec 3 décimales ; `min` < `max` |
| `{{random_float(min, max, digits)}}` | le même avec `digits` décimales, 0–9 |
| `{{pick(a, b, c)}}` | l’un des arguments, au moins un |

Les arguments sont séparés par des virgules. Un argument entre guillemets (`"dark blue"` ou `'a, b'`)
peut contenir n’importe quoi sauf son propre guillemet ; un argument sans guillemets peut contenir des lettres,
des chiffres et `_ - . : / +`. Un argument vide est une erreur.

Chaque générateur tire de la graine de l’exécution. Les valeurs d’une exécution d’un
nœud dépendent seulement de la graine, de l’identifiant du nœud et du nombre de fois où le nœud s’est
exécuté, si bien que les branches parallèles ne changent jamais les valeurs les unes des autres, et qu’une exécution avec
la même graine génère à nouveau les mêmes valeurs. Les tirages au sein d’un même nœud suivent
l’ordre de ses champs. `{{now}}` et `{{run.id}}` ne sont pas reproductibles. Voir
[graines](runs.md#seeds).

### Suggestions {#suggestions}

Saisir `{{` dans un champ modélisable, ou appuyer sur <kbd>Ctrl</kbd>+<kbd>Space</kbd>,
ouvre une liste en quatre groupes : [[ui:exp.suggest.params]] avec leurs valeurs,
[[ui:exp.suggest.vars]] définies en amont de ce nœud avec le nœud qui les définit
(les champs d’une réponse aussi, comme `reply.args[0]`), [[ui:exp.suggest.secrets]] et
[[ui:exp.suggest.generators]]. <kbd>↑</kbd> et <kbd>↓</kbd> choisissent,
<kbd>Enter</kbd> ou <kbd>Tab</kbd> insère, <kbd>Esc</kbd> ferme la liste et
garde le champ.

### Les noms inconnus sont des erreurs {#unknown-names}

Un nom sans valeur ne devient jamais une chaîne vide. Avant une exécution, chaque
nom qu’un champ utilise doit être un paramètre, un nom de secret valide, ou une variable définie
sur **chaque** chemin qui mène au nœud. L’éditeur désigne le nœud et
le champ :

| Problème | Avant l’exécution | Pendant l’exécution |
| --- | --- | --- |
| Un nom que personne ne définit | `name.unknown` | — |
| Une variable définie sur certains chemins seulement | `name.not_on_every_path` | — |
| `{{params.x}}` sans paramètre `x` | `param.unknown` | — |
| Un champ qu’une valeur n’a pas | — | `template.no_field` |
| Un `{{` non fermé, un `{{}}` vide, un argument mal formé | `template.*`, avec la position | — |

Les textes de ces codes sont dans [Erreurs](../reference/errors.md).

## Quels champs acceptent les modèles {#templated-fields}

| Nœud | Champs modélisables |
| --- | --- |
| [[ui:exp.node.http]] | URL, noms et valeurs d’en-têtes, corps, le nom d’utilisateur et le mot de passe de Basic et Digest, le jeton Bearer |
| [[ui:exp.node.osc]] | cible, adresse, arguments texte ; avec une réponse : son motif d’adresse et les valeurs de sa règle |
| [[ui:exp.node.udp]] | cible, charge utile ; avec une réponse : son motif |
| [[ui:exp.node.tcp]] | hôte, charge utile |
| [[ui:exp.node.mqtt]] | hôte du broker, topic, charge utile |
| [[ui:exp.node.log]] | message |
| [[ui:exp.node.assert_body]] | texte attendu |
| [[ui:exp.node.assert_header]] | nom d’en-tête, texte attendu |
| [[ui:exp.node.assert_value]], [[ui:exp.node.branch_value]], la condition de sortie d’une [[ui:exp.node.loop]] | valeur, valeur attendue |
| [[ui:exp.node.wait_osc]] | motif d’adresse, valeurs de la règle |
| [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_ws]] | motif |
| [[ui:exp.node.wait_mqtt]] | broker et topic (paramètres seulement), motif |
| [[ui:exp.node.wait_http]] | motif de chemin, conditions |
| [[ui:exp.node.impairment]] | écoute et cible (paramètres seulement) |
| [[ui:exp.node.ws_connect]] | URL, noms et valeurs d’en-têtes |
| [[ui:exp.node.ws_send]] | charge utile |
| [[ui:exp.node.ws_close]] | raison |

Les nombres — ports, délais d’attente, délais, statuts, nombres OSC typés — et les
adresses d’écoute des attentes sont littéraux. Un [[ui:exp.node.emulator]] rend
ses propres réponses avec ce qui est arrivé (`{{request.…}}`) et les paramètres ; voir
[défaillances](faults.md#emulator).

**Paramètres seulement.** Certains champs sont ouverts avant la première étape, quand aucune
variable n’existe encore : le broker et le topic d’une [[ui:exp.node.wait_mqtt]], l’écoute et la cible d’un
[[ui:exp.node.impairment]]. Ils n’acceptent que du texte et des paramètres,
rien d’autre (`node.params_only`).

**Vérifiés comme des littéraux.** Un champ qui n’utilise que des paramètres est résolu
avant l’exécution et vérifié comme le texte que l’exécution enverra : une URL doit être
`http://` ou `https://`, une cible OSC `IP:port` ou `host:port`, un nom d’en-tête valide. Un champ
avec des variables ou des générateurs est vérifié quand il s’exécute.

## Aperçu {#preview}

Quand le nœud sélectionné a un modèle, ses propriétés montrent ce qu’il fera
avec les valeurs connues maintenant : [[ui:exp.preview]] pour un envoi,
[[ui:exp.previewWait]] pour une attente, [[ui:exp.previewCheck]] pour une comparaison.
Le moteur le résout, avec le même code qu’une exécution utilise, si bien que l’aperçu ne
contredit jamais l’exécution.

- Les paramètres viennent du profil actif.
- Les variables viennent de ce que l’éditeur a vu dans cette session : les étapes de la dernière
  exécution, et [[ui:exp.sendNow]].
- Un secret stocké s’affiche sous la forme `••••`.
- Un nom sans valeur encore reste tel qu’écrit, et l’aperçu le liste. Un
  secret qui n’est pas stocké est listé à part.
- Les générateurs utilisent la graine épinglée de l’expérience, ou `0` quand aucune n’est épinglée, comme
  première exécution du nœud. Avec une graine épinglée, l’aperçu montre les
  valeurs générées que la première exécution du nœud dans une exécution enverra.

## Extraire des valeurs {#extract}

[[ui:exp.node.extract]] lit une valeur de la dernière réponse HTTP sur son
chemin et l’écrit dans une variable.

| Champ | Quoi |
| --- | --- |
| [[ui:exp.variable]] | la variable à écrire ; les règles de nommage des paramètres s’appliquent |
| [[ui:exp.extractFrom]] | d’où vient la valeur (ci-dessous) |
| [[ui:exp.jsonPath]], [[ui:exp.headerName]] ou [[ui:exp.pattern]] | quoi lire, selon la source |

| [[ui:exp.extractFrom]] | Lit | Valeur |
| --- | --- | --- |
| [[ui:exp.from.json]] | le corps en JSON, à un chemin | la valeur JSON : texte, nombre, objet, tableau |
| [[ui:exp.from.header]] | le premier en-tête portant ce nom, quelle que soit la casse | texte |
| [[ui:exp.from.status]] | le code de statut | un nombre |
| [[ui:exp.from.body]] | tout le corps | texte |
| [[ui:exp.from.regex]] | la première correspondance dans le corps | le groupe de capture 1 si le motif en a un, sinon toute la correspondance |

**Chemins JSON.** `$.token`, `$.items[0].id`, `$["a b"]`, `$['a b']['c-d']` ;
le `$.` initial peut être omis (`token`, `items[0].id`), et `$` seul est
tout le corps.

**Les expressions régulières** utilisent la syntaxe du moteur `regex` de Rust, qui n’a ni
look-around ni rétroréférences. La correspondance est cherchée n’importe où dans le
corps ; ancrez-la avec `^` et `$` quand cela compte.

L’étape échoue, en nommant ce qui manque, quand :

- aucune requête HTTP ne s’est exécutée avant elle sur ce chemin (`check.no_response` ; l’éditeur
  refuse déjà un graphe où aucune ne peut, `graph.needs_http`) ;
- le corps n’est pas du JSON, ou le chemin ne s’y trouve pas ;
- l’en-tête n’est pas là, ou le motif ne correspond pas ;
- le corps dépasse les 256 Kio qu’une réponse conserve, pour un chemin JSON ou tout le
  corps, et pour un motif qui n’a rien trouvé dans la partie conservée
  (`extract.truncated`).

La chronologie affiche la valeur écrite : `token = abc123`.

::: tip Extraire en cliquant
[[ui:exp.sendNow]] sur un [[ui:exp.node.http]] montre sa réponse JSON. Cliquez sur une
valeur : un nœud [[ui:exp.node.extract]] est ajouté après la requête, avec
le chemin rempli et un nom pris de la clé, et la valeur est connue de
l’aperçu aussitôt.
:::

## Variables {#variables}

Une variable contient une valeur JSON. Ces nœuds en écrivent une :

| Nœud | Écrit | En sortie |
| --- | --- | --- |
| [[ui:exp.node.extract]] | la valeur extraite | sa sortie |
| [[ui:exp.node.wait_osc]], [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_http]], [[ui:exp.node.wait_ws]] | ce qui est arrivé, nom par défaut `reply` (`request` pour HTTP) | [[ui:exp.portMatched]] seulement |
| [[ui:exp.node.osc]], [[ui:exp.node.udp]] avec [[ui:exp.expectReply]] | la réponse, nom par défaut `reply` | sa sortie |

Ce qu’une attente écrit est un objet ; les champs suivants en lisent les parties :

| Attente | Champs |
| --- | --- |
| OSC | `address`, `args`, `from`, `ms` |
| UDP | `text`, `hex`, `bytes`, `from`, `ms`, et `match` avec un motif |
| MQTT | `topic`, et les champs d’UDP |
| WebSocket | les champs d’UDP, et `json` quand le message est du JSON |
| Requête HTTP | `method`, `path`, `query`, `headers`, `body`, `json`, `params`, `from`, `ms` |

`ms` est le temps depuis la dernière action de la branche jusqu’à l’arrivée. Le contenu
exact est dans [la référence des nœuds](nodes.md).

### Où une variable est connue {#visibility}

Une variable existe à partir de la sortie qui l’écrit, sur les chemins qui
passent par cette sortie :

- Après une fusion de chemins alternatifs — le [[ui:exp.yes]] et le [[ui:exp.no]]
  d’une branche qui se rejoignent — seul ce que **chaque** chemin définit est connu.
- Après [[ui:exp.node.join]], ce que **n’importe quelle** branche y entrant définit est connu : toutes
  se sont exécutées.
- Après le [[ui:exp.portDone]] ou le [[ui:exp.portLimit]] d’une [[ui:exp.node.loop]],
  et dans sa condition de sortie, ce que chaque itération du corps définit est connu.
- La variable d’une attente n’est pas connue après sa sortie [[ui:exp.portTimeout]].

Chaque branche parallèle travaille sur sa propre copie des variables. Un Join fusionne les
copies dans l’ordre de ses fils entrants, le fil le plus tardif l’emportant sur un nom que les deux
définissent, si bien que le résultat ne dépend jamais de la branche qui a fini en premier. Voir
[comment une exécution avance](flow.md#parallel).

## Comparer des valeurs {#compare}

[[ui:exp.node.assert_value]] fait échouer l’exécution quand une comparaison ne tient pas ;
[[ui:exp.node.branch_value]] sort par [[ui:exp.yes]] ou [[ui:exp.no]] ;
une [[ui:exp.node.loop]] utilise la même comparaison comme condition de sortie. Chacun a
une [[ui:exp.value]], un [[ui:exp.operator]] et une valeur [[ui:exp.expected]], et
les deux textes sont des modèles :

| [[ui:exp.value]] | [[ui:exp.operator]] | [[ui:exp.expected]] |
| --- | --- | --- |
| `{{status}}` | [[ui:exp.op.lt]] | `300` |
| `{{reply.args[0]}}` | [[ui:exp.op.eq]] | `{{nonce}}` |

| [[ui:exp.operator]] | Tient quand |
| --- | --- |
| [[ui:exp.op.eq]], [[ui:exp.op.ne]] | les deux sont égaux (différents) — comme nombres quand les deux sont des nombres (`200` égale `200.0`), sinon comme texte exact, casse comprise |
| [[ui:exp.op.lt]], [[ui:exp.op.le]], [[ui:exp.op.gt]], [[ui:exp.op.ge]] | comme nombres ; un côté qui n’est pas un nombre fait échouer l’étape (`compare.not_numbers`) au lieu d’un *non* discret |
| [[ui:exp.op.contains]] | la valeur contient le texte attendu, casse comprise |
| [[ui:exp.op.matches]] | l’expression régulière dans la valeur attendue correspond n’importe où dans la valeur |
| [[ui:exp.op.empty]], [[ui:exp.op.not_empty]] | la valeur est vide, ou non, après suppression des espaces ; la valeur attendue n’est pas utilisée |

Un nombre est du texte qui se lit comme tel après suppression des espaces : `42`, `-1.5`,
`1e3`. La chronologie montre la comparaison telle que faite, `401 = 200`, chaque côté coupé
à 120 caractères.

## Secrets {#secrets}

Un jeton ou un mot de passe entre dans un champ sous la forme `{{secret.NAME}}`. Le fichier
d’expérience ne garde que le nom ; la valeur reste où elle est stockée et n’atteint jamais
l’interface.

### Où vivent les secrets {#secret-stores}

| Où Signal Lab s’exécute | Magasin | Depuis l’interface |
| --- | --- | --- |
| Application de bureau sous Windows | le Gestionnaire d’identification Windows, sous le service `SignalLab`, une entrée par nom | définir, remplacer, supprimer |
| Application de bureau sous Linux | aucun : une exécution qui a besoin d’un secret échoue avec `secret.unsupported` | — |
| Serveur | la variable d’environnement `SIGNALLAB_SECRET_<NAME>`, sinon le fichier `<NAME>` dans son dossier de secrets, `/run/secrets/signallab` sauf indication contraire | lecture seule |
| `signallab` en ligne de commande | comme un serveur, ou le Gestionnaire d’identification Windows avec `--secrets system` | — |

L’en-tête de la section [[ui:exp.secrets]] a une info-bulle qui dit laquelle de
ces situations s’applique là où vous êtes : le magasin Windows, l’environnement et les
fichiers du serveur, ou — dans l’application de bureau sous Linux — qu’il n’y a pas de magasin. Là,
`secret.unsupported` dit que les secrets sont conservés dans le Gestionnaire d’identification
Windows, que le système n’a pas.

Un secret appartient à l’ordinateur ou au serveur, pas à une expérience : deux
expériences qui utilisent `{{secret.API_TOKEN}}` utilisent la même valeur.

Sur un serveur, la variable d’environnement l’emporte sur le fichier. Le saut de ligne
final d’un fichier ne fait pas partie de la valeur, et un fichier vide compte comme aucun secret.
Le dossier du serveur est défini avec `--secrets-dir` ou `SIGNALLAB_SECRETS_DIR` ;
voir [le serveur](../server/index.md). Pour la ligne de commande, voir
[`signallab run`](../automation/cli.md#cli-run).

| Règle | Limite |
| --- | --- |
| Nom | commence par une lettre ou `_`, puis des lettres, des chiffres et `_` ; 128 caractères au plus |
| Valeur | non vide, 16 Kio au plus |

### Définir un secret {#set-secret}

Sous Windows :

1. Ouvrez [[ui:exp.params]]. La section [[ui:exp.secrets]] liste chaque secret
   que les champs de l’expérience utilisent, chacun [[ui:exp.secretStored]] ou
   [[ui:exp.secretMissing]].
2. Appuyez sur [[ui:exp.secretSet]] à côté du nom, ou sur [[ui:exp.addSecret]] pour un
   nom qu’aucun champ n’utilise encore.
3. Saisissez la valeur — le champ affiche des points — et appuyez sur [[ui:exp.secretSave]] ou
   <kbd>Enter</kbd>. Le champ est vidé ; rien ne peut relire la valeur.

[[ui:exp.secretReplace]] stocke une nouvelle valeur et [[ui:exp.secretRemove]]
la supprime du magasin d’identifiants. Un nom stocké dans cette session est aussi proposé
dans les suggestions.

Dans un navigateur connecté à un serveur, la section dit seulement
[[ui:exp.secretOnServer]] ou [[ui:exp.secretNotOnServer]] : définissez la valeur là où
le serveur tourne, de l’une des deux façons suivantes :

```bash
# in the server's environment
SIGNALLAB_SECRET_API_TOKEN='…'
# or as a file in its secrets folder
printf '%s' '…' > /run/secrets/signallab/API_TOKEN
```

Un fichier est lu chaque fois qu’une exécution démarre, donc un fichier modifié compte à partir de
l’exécution suivante ; une variable d’environnement modifiée nécessite un redémarrage du serveur.

### Avant une exécution {#secret-check}

Chaque secret que les champs de l’exécution utilisent doit être stocké. Un secret manquant arrête l’exécution
avant tout trafic, au premier nœud et champ qui l’utilisent
(`secret.missing`). [[ui:exp.sendNow]] vérifie la même chose pour son nœud.

### Masquage {#masking}

Pendant qu’une exécution ou un [[ui:exp.sendNow]] utilise des secrets, chaque occurrence de leurs
valeurs est remplacée par `••••` dans tout ce qui quitte le moteur :

- les textes des étapes, les erreurs et les variables qu’une étape a écrites ;
- le rapport d’exécution ;
- le résultat de [[ui:exp.sendNow]], la réponse HTTP qu’il affiche comprise ;
- les trames de l’[[ui:dock.inspector]], capturées pendant que l’exécution dure — dans un
  vidage hexadécimal, chaque octet d’une valeur devient `*`, pour que les décalages restent exacts.

L’authentification Basic envoie `name:password` en base64 ; quand l’une des deux parties contient un
secret, ce texte base64 est masqué aussi. Le trafic lui-même transporte la vraie
valeur. L’aperçu montre un secret stocké sous la forme `••••`. Les réponses d’un
[[ui:exp.node.emulator]] ne peuvent pas utiliser de secrets.

## Commandes {#commands}

L’aperçu est [`experiment_resolve`](../api/commands.md#experiment_resolve) ;
les secrets sont listés, définis et supprimés avec
[`secret_status`](../api/commands.md#secret_status),
[`secret_set`](../api/commands.md#secret_set) et
[`secret_delete`](../api/commands.md#secret_delete). Aucune commande ne renvoie la valeur d’un
secret.

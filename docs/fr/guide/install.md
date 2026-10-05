---
title: "Installation et mises à jour"
description: "Installer l’application de bureau Signal Lab sous Windows ou Linux, vérifier un téléchargement, la tenir à jour et la désinstaller."
---

# Installation et mises à jour

Signal Lab est publié sur GitHub : chaque version publiée contient l’application de bureau pour Windows et
Linux, la ligne de commande `signallab` seule et une liste de sommes de contrôle. Pour l’exécuter sur un
serveur et l’utiliser plutôt depuis un navigateur, voir [Le serveur](../server/index.md).

## Que télécharger {#downloads}

Ouvrez la [dernière version publiée](https://github.com/ProAnima/SignalLab/releases/latest) et choisissez
le fichier de votre système. `<version>` est le numéro de la version, par exemple `[[version]]`.

| Fichier | Pour |
| --- | --- |
| `Signal.Lab_<version>_x64-setup.exe` | Windows 10 et 11, x64 — le programme d’installation qu’il faut à la plupart des gens |
| `Signal.Lab_<version>_x64_en-US.msi` | Windows, déploiement par le service informatique (Intune, stratégie de groupe) : installe pour tous les utilisateurs |
| `Signal.Lab_<version>_amd64.deb` | Linux x86_64 : Debian, Ubuntu et leurs dérivées |
| `Signal.Lab-<version>-1.x86_64.rpm` | Linux x86_64 : Fedora, RHEL, openSUSE et leurs dérivées |
| `Signal.Lab_<version>_amd64.AppImage` | Linux x86_64, toute distribution, sans installation |
| `signallab-<version>-windows-x64.zip` | la ligne de commande seule, pour Windows |
| `signallab-<version>-linux-x64.tar.gz` | la ligne de commande seule, pour Linux |
| `SHA256SUMS.txt` | la somme de contrôle SHA-256 de chacun des fichiers ci-dessus |

Les fichiers `.sig` et `latest.json` qui les accompagnent servent au système de mise à jour de l’application ; vous
n’en avez pas besoin.

## Windows {#windows}

Signal Lab fonctionne sous Windows 10 et 11, x64. Il utilise le runtime WebView2, inclus dans
les deux.

### Installer avec le programme d’installation {#windows-setup}

1. Lancez `Signal.Lab_<version>_x64-setup.exe`.
2. Si Windows SmartScreen indique qu’il a protégé votre ordinateur, choisissez **Informations complémentaires**, puis
   **Exécuter quand même** (voir [SmartScreen](#smartscreen) plus bas).
3. Choisissez la langue du programme d’installation. Il propose les mêmes 11 langues que l’application.
4. Choisissez pour qui installer :
   - Pour vous seul : l’installation va dans `%LOCALAPPDATA%\Signal Lab` et ne demande pas
     de droits d’administrateur.
   - Pour tous les utilisateurs de cet ordinateur : l’installation va dans `C:\Program Files\Signal Lab`
     et demande des droits d’administrateur.
5. Confirmez le dossier et installez.
6. La dernière page propose de lancer Signal Lab tout de suite et de créer un raccourci sur le bureau.

Lancer le programme d’installation d’une version plus récente par-dessus une version installée la met à niveau sur place.

### Ce que le programme d’installation ajoute {#windows-setup-adds}

En plus de l’application, le programme d’installation met en place deux choses, et le programme de désinstallation retire les deux :

- **La ligne de commande.** `signallab.exe` est placé à côté de l’application et son dossier est ajouté au `PATH` —
  votre propre `PATH` pour une installation pour vous seul, celui de l’ordinateur pour une installation pour
  tous — pour que `signallab` fonctionne dans chaque terminal que vous ouvrez ensuite. Voir
  [La ligne de commande](../automation/cli.md).
- **Une règle de pare-feu**, uniquement pour une installation pour tous. Le Pare-feu Windows Defender ne laisse
  d’autres machines atteindre un programme que si une règle l’autorise, et interroge la personne devant
  l’écran la première fois que le programme écoute ; un *Annuler* à ce moment-là rejette sans bruit tout ce que
  les autres machines envoient. Une installation pour tous, qui dispose des droits d’administrateur, ajoute une
  règle d’autorisation entrante pour `signal-lab.exe` (l’application) et une pour `signallab.exe` (la
  ligne de commande), sur les réseaux privés et de domaine. Une installation pour vous seul ne peut pas modifier
  le pare-feu : l’application propose de le faire, avec l’invite d’administrateur de Windows, la première
  fois que quelque chose écoute — voir [l’avertissement du pare-feu](interface.md#firewall-notice).

Le trafic sur cet ordinateur lui-même (`127.0.0.1`) n’est jamais filtré : tout ce que vous faites en
boucle locale fonctionne sans aucune règle.

### Installations sans surveillance {#windows-unattended}

Le programme d’installation accepte ces options sur sa ligne de commande :

| Option | Effet |
| --- | --- |
| `/S` | Installe en silence, sans rien demander. |
| `/P` | Installe avec une barre de progression et sans questions. |
| `/ALLUSERS` | Installe pour tous les utilisateurs (demande une invite élevée). |
| `/CURRENTUSER` | Installe pour l’utilisateur actuel seulement. |
| `/NS` | Ne crée aucun raccourci. |
| `/D=C:\Tools\Signal Lab` | Installe dans ce dossier. Doit venir en dernier et ne se met pas entre guillemets. |
| `/NOPATH` | Ne touche pas au `PATH` : `signallab` est installé mais pas dans le `PATH`. |
| `/NOFIREWALL` | N’ajoute aucune règle de pare-feu. |

Par exemple, une installation silencieuse pour tous sans règles de pare-feu, depuis une invite
élevée :

```powershell
.\Signal.Lab_[[version]]_x64-setup.exe /S /ALLUSERS /NOFIREWALL
```

### Le MSI {#windows-msi}

`Signal.Lab_<version>_x64_en-US.msi` sert au déploiement par le service informatique. Il installe pour tous les utilisateurs,
place `signallab.exe` à côté de l’application et ajoute ce dossier au `PATH` de l’ordinateur, mais
n’ajoute aucune règle de pare-feu :
c’est à la charge de qui le déploie (par stratégie de groupe, par exemple). L’application propose quand même
la règle la première fois qu’elle écoute. Pour l’installer en silence :

```powershell
msiexec /i "Signal.Lab_[[version]]_x64_en-US.msi" /qn
```

### SmartScreen {#smartscreen}

Les programmes d’installation ne sont pas encore signés numériquement : Windows SmartScreen ne connaît donc pas leur
éditeur et peut arrêter l’installation avec un avertissement. Choisissez **Informations complémentaires**, vérifiez que le
fichier est bien celui que vous avez téléchargé depuis la page des versions publiées, puis choisissez **Exécuter quand même**. Pour être
sûr que le fichier n’a pas été modifié, [comparez-le d’abord à `SHA256SUMS.txt`](#checksums).

Les mises à jour de l’application elle-même sont signées et vérifiées séparément ; voir [Mises à jour](#updates).

## Vérifier un téléchargement {#checksums}

`SHA256SUMS.txt` liste la somme de contrôle SHA-256 de chaque fichier de la version, une par ligne,
suivie du nom du fichier. Téléchargez-le à côté du fichier et comparez.

Sous Windows, dans PowerShell :

```powershell
Get-FileHash .\Signal.Lab_[[version]]_x64-setup.exe -Algorithm SHA256
Select-String "Signal.Lab_[[version]]_x64-setup.exe" .\SHA256SUMS.txt
```

Les deux empreintes doivent être identiques (`Get-FileHash` l’affiche en majuscules ; la casse n’a pas
d’importance).

Sous Linux, dans le dossier qui contient les deux fichiers :

```bash
sha256sum --check --ignore-missing SHA256SUMS.txt
```

La commande affiche `OK` après chaque fichier trouvé. Tout autre résultat signifie que le téléchargement n’est pas le
fichier publié : téléchargez-le à nouveau.

## Linux {#linux}

L’application Linux est compilée pour x86_64 (sous Ubuntu 22.04) et nécessite WebKitGTK 4.1, le moteur
web avec lequel elle dessine sa fenêtre. Il n’existe pas de version pour processeurs Arm ; sur une machine Arm,
exécutez plutôt le [serveur](../server/index.md).

### Le paquet .deb {#linux-deb}

```bash
sudo apt install ./Signal.Lab_[[version]]_amd64.deb
```

`apt` installe en même temps ce dont le paquet a besoin. Le paquet place aussi la ligne de
commande dans `/usr/bin/signallab`.

### Le paquet .rpm {#linux-rpm}

```bash
sudo dnf install ./Signal.Lab-[[version]]-1.x86_64.rpm
```

Sous openSUSE, utilisez `sudo zypper install` avec le même fichier. Comme le `.deb`, le paquet
place la ligne de commande dans `/usr/bin/signallab`.

### L’AppImage {#linux-appimage}

L’AppImage s’exécute sans installation :

```bash
chmod +x Signal.Lab_[[version]]_amd64.AppImage
./Signal.Lab_[[version]]_amd64.AppImage
```

Elle n’inclut pas la ligne de commande `signallab` ; prenez-la dans
l’[archive de la ligne de commande](#cli-archives) si vous en avez besoin.

### Le pare-feu sous Linux {#linux-firewall}

Sous Linux, l’application n’affiche aucun avertissement de pare-feu : `ufw` et `firewalld` fonctionnent par port, pas par
programme. Si l’un d’eux est actif et que d’autres machines doivent atteindre un moniteur, un écouteur, un
émulateur ou un relais, ouvrez-y son port, par exemple :

```bash
sudo ufw allow 9000/udp
```

Le trafic en boucle locale n’est pas concerné.

## La ligne de commande seule {#cli-archives}

Les programmes d’installation de bureau apportent `signallab` avec eux. Pour une machine sans l’application — un
agent de build, un serveur, un PC de laboratoire —, chaque version publiée contient aussi la ligne de commande seule :

- `signallab-<version>-windows-x64.zip` contient `signallab.exe` et la licence.
- `signallab-<version>-linux-x64.tar.gz` contient `signallab` et la licence.

Décompressez-la dans un dossier de votre `PATH`. Sous Linux :

```bash
tar -xzf signallab-[[version]]-linux-x64.tar.gz signallab
sudo install signallab /usr/local/bin/
signallab version
```

L’image Docker du serveur la contient aussi. Pour l’utiliser : [La ligne de commande](../automation/cli.md).

## Mises à jour {#updates}

### L’application de bureau {#updates-desktop}

L’application de bureau se met à jour à partir des versions publiées uniquement, et seulement quand vous le décidez.

- **Elle vérifie une fois par jour.** Avec la case [[ui:update.auto]] cochée (elle l’est par défaut), l’application
  cherche une version plus récente peu après son démarrage si la dernière recherche date d’un jour ou
  plus, puis à nouveau chaque fois qu’un jour s’est écoulé pendant qu’elle tourne. Décochez-la dans À propos pour
  que l’application cesse de chercher d’elle-même.
- **Vous pouvez vérifier tout de suite.** Ouvrez À propos — le **?** de l’en-tête — et appuyez sur
  [[ui:update.check]] sous [[ui:update.title]].
- **Ce qu’envoie la vérification.** Elle demande au service de mise à jour du studio (`hub.proanima.net`)
  quelle version cette installation doit recevoir, et ne va voir la dernière version publiée sur GitHub que lorsque
  ce service est injoignable. La requête contient la version de l’application, le système
  et le type de processeur, ainsi qu’un nombre aléatoire créé pour cette installation, qui permet à une nouvelle
  version d’atteindre d’abord une partie des installations. Rien qui vous désigne, vous ou cet ordinateur, n’est
  envoyé.
- **Quand une version est trouvée**, la console le signale et l’en-tête affiche un bouton de mise à jour
  qui ouvre À propos. Là, [[ui:update.notes]] affiche les notes de version.
- **Rien ne s’installe tant que vous ne cliquez pas.** Appuyez sur [[ui:update.install]] (quand des tâches sont
  en cours, le bouton indique qu’il va les arrêter). Signal Lab enregistre tout ce qui est encore
  en attente, arrête toutes les tâches en cours, télécharge la mise à jour et vérifie sa signature
  avec la clé intégrée à l’application — une mise à jour qui n’est pas signée par le processus de publication
  est refusée —, puis l’installe et redémarre dans la nouvelle version.

Sous Windows, la mise à jour lance le programme d’installation sans poser de questions et conserve votre dossier, votre `PATH`
et vos règles de pare-feu. Sous Linux, elle remplace l’AppImage, ou installe le nouveau paquet `.deb` ou
`.rpm` — pour un paquet, le système vous demande d’abord votre mot de passe.

Les préversions et les brouillons ne sont jamais proposés. Pour qu’aucune installation d’une machine ne
cherche d’elle-même — quand le service informatique déploie lui-même les mises à jour, par exemple —, définissez la variable d’environnement
`SIGNALLAB_NO_UPDATE_CHECK` avec n’importe quelle valeur ; [[ui:update.check]] fonctionne toujours quand on appuie dessus.
Les versions de développement ne cherchent jamais d’elles-mêmes.

### Un serveur {#updates-server}

Un serveur, et la page qu’un navigateur affiche depuis celui-ci, se mettent à jour avec leur image Docker, jamais
d’eux-mêmes : À propos l’indique. Pour en mettre un à jour : [Le serveur](../server/index.md).

## Désinstallation {#uninstall}

**Windows.** Ouvrez *Paramètres → Applications → Applications installées*, trouvez Signal Lab et choisissez
*Désinstaller*. Le programme de désinstallation retire `signallab.exe` du `PATH` et, lorsqu’il s’exécute
avec les droits d’administrateur (comme le fait une installation pour tous), supprime toutes les règles de pare-feu
entrantes de l’application et de la ligne de commande, y compris celles créées par l’invite de Windows.
Il propose une case pour supprimer aussi les données de l’application : ce sont les réglages propres à l’application
— la langue, la taille des volets, l’écran où vous étiez, les dernières valeurs saisies dans les
écrans —, pas vos fichiers. Une installation MSI se désinstalle de la même façon ; elle emporte son entrée du `PATH`
et laisse le pare-feu à qui l’a déployée.

**Linux.** Supprimez le paquet avec le gestionnaire de paquets qui l’a installé, ou supprimez
le fichier AppImage.

Ni l’un ni l’autre ne touche à votre **dossier de données** : les expériences, les bibliothèques de signaux et d’émulateurs, les
rapports d’exécution et les exports restent dans `Documents/SignalLab`, dans votre dossier personnel. Supprimez ce dossier
vous-même si vous voulez qu’ils disparaissent.

## Où sont vos données {#data}

Tout ce que vous créez est conservé sous forme de fichiers ordinaires dans un seul dossier, `Documents/SignalLab`, dans votre
dossier personnel, sous Windows comme sous Linux : l’expérience en cours, la bibliothèque de signaux, la
bibliothèque d’émulateurs, les rapports d’exécution, les exports et les captures de l’Inspecteur. Chaque fichier et son format
sont décrits dans [Fichiers et dossiers](../reference/files.md).

## L’exécuter comme serveur {#server}

Pour utiliser Signal Lab depuis un navigateur — sur un PC de laboratoire à côté des équipements, sur un boîtier Linux, dans
Docker —, voir [Le serveur](../server/index.md). Sur une machine Linux avec Docker, une seule
commande l’installe et le démarre.

---
title: Installing and updating
description: Install the Signal Lab desktop app on Windows or Linux, check a download, keep it up to date and remove it again.
---

# Installing and updating

Signal Lab is released on GitHub: every release has the desktop app for Windows and
Linux, the `signallab` command line on its own, and a list of checksums. To run it on a
server and use it from a browser instead, see [The server](../server/index.md).

## What to download {#downloads}

Open the [latest release](https://github.com/ProAnima/SignalLab/releases/latest) and pick
the file for your system. `<version>` is the release's version, such as `[[version]]`.

| File | For |
| --- | --- |
| `Signal.Lab_<version>_x64-setup.exe` | Windows 10 and 11, x64 — the setup most people want |
| `Signal.Lab_<version>_x64_en-US.msi` | Windows, deployment by IT (Intune, Group Policy): installs for every user |
| `Signal.Lab_<version>_amd64.deb` | Linux x86_64: Debian, Ubuntu and their relatives |
| `Signal.Lab-<version>-1.x86_64.rpm` | Linux x86_64: Fedora, RHEL, openSUSE and their relatives |
| `Signal.Lab_<version>_amd64.AppImage` | Linux x86_64, any distribution, without installing |
| `signallab-<version>-windows-x64.zip` | the command line alone, for Windows |
| `signallab-<version>-linux-x64.tar.gz` | the command line alone, for Linux |
| `SHA256SUMS.txt` | the SHA-256 checksum of every file above |

The `.sig` files and `latest.json` next to them are for the app's own updater; you do
not need them.

## Windows {#windows}

Signal Lab runs on Windows 10 and 11, x64. It uses the WebView2 runtime, which is part of
both.

### Installing with the setup {#windows-setup}

1. Run `Signal.Lab_<version>_x64-setup.exe`.
2. If Windows SmartScreen says it protected your PC, choose **More info**, then
   **Run anyway** (see [SmartScreen](#smartscreen) below).
3. Pick the setup's language. It offers the same 11 languages as the app.
4. Choose who it is for:
   - Just for you: it installs into `%LOCALAPPDATA%\Signal Lab` and needs no
     administrator rights.
   - For everyone using this computer: it installs into `C:\Program Files\Signal Lab`
     and asks for administrator rights.
5. Confirm the folder and install.
6. The last page offers to start Signal Lab right away and to create a desktop shortcut.

Running the setup of a newer version over an installed one upgrades it in place.

### What the setup adds {#windows-setup-adds}

Besides the app, the setup puts two things in place, and the uninstaller removes both:

- **The command line.** `signallab.exe` goes next to the app and its folder onto `PATH` —
  your own `PATH` for an install just for you, the computer's for an install for
  everyone — so `signallab` works in every terminal you open afterwards. See
  [The command line](../automation/cli.md).
- **A firewall rule**, for an install for everyone only. Windows Defender Firewall lets
  other machines reach a program only when a rule allows it, and asks the person at the
  screen the first time the program listens; a *Cancel* there silently drops whatever
  other machines send. An install for everyone, which has administrator rights, adds an
  inbound allow rule for `signal-lab.exe` (the app) and one for `signallab.exe` (the
  command line), on private and domain networks. An install just for you cannot change
  the firewall: the app offers to, with Windows' own administrator prompt, the first
  time something listens — see [the firewall notice](interface.md#firewall-notice).

Traffic on this computer itself (`127.0.0.1`) is never filtered, so everything you do on
loopback works without any rule.

### Unattended installs {#windows-unattended}

The setup takes these switches on its command line:

| Switch | What it does |
| --- | --- |
| `/S` | Installs silently, asking nothing. |
| `/P` | Installs with a progress bar and no questions. |
| `/ALLUSERS` | Installs for everyone (needs an elevated prompt). |
| `/CURRENTUSER` | Installs just for the current user. |
| `/NS` | Creates no shortcuts. |
| `/D=C:\Tools\Signal Lab` | Installs into this folder. It must come last and is not quoted. |
| `/NOPATH` | Leaves `PATH` alone: `signallab` is installed but not on `PATH`. |
| `/NOFIREWALL` | Adds no firewall rules. |

For example, a silent install for everyone without firewall rules, from an elevated
prompt:

```powershell
.\Signal.Lab_[[version]]_x64-setup.exe /S /ALLUSERS /NOFIREWALL
```

### The MSI {#windows-msi}

`Signal.Lab_<version>_x64_en-US.msi` is for deployment by IT. It installs for every user
and puts `signallab.exe` next to the app and that folder on the computer's `PATH`, but
adds no firewall rule:
that is left to whoever deploys it (Group Policy, for example). The app still offers the
rule when it first listens. To install it silently:

```powershell
msiexec /i "Signal.Lab_[[version]]_x64_en-US.msi" /qn
```

### SmartScreen {#smartscreen}

The installers are not code-signed yet, so Windows SmartScreen does not know their
publisher and may stop the setup with a warning. Choose **More info**, check that the
file is the one you downloaded from the releases page, then choose **Run anyway**. To be
sure the file is unchanged, [check it against `SHA256SUMS.txt`](#checksums) first.

The app's own updates are signed and checked separately; see [Updates](#updates).

## Checking a download {#checksums}

`SHA256SUMS.txt` lists the SHA-256 checksum of every file of the release, one per line,
followed by the file's name. Download it next to the file and compare.

On Windows, in PowerShell:

```powershell
Get-FileHash .\Signal.Lab_[[version]]_x64-setup.exe -Algorithm SHA256
Select-String "Signal.Lab_[[version]]_x64-setup.exe" .\SHA256SUMS.txt
```

The two hashes must be the same (`Get-FileHash` prints it in capitals; the case does not
matter).

On Linux, in the folder that holds both files:

```bash
sha256sum --check --ignore-missing SHA256SUMS.txt
```

It prints `OK` after each file it found. Anything else means the download is not the
released file: download it again.

## Linux {#linux}

The Linux app is built for x86_64 (on Ubuntu 22.04) and needs WebKitGTK 4.1, the web
view it draws its window with. There is no build for Arm processors; on an Arm machine,
run the [server](../server/index.md) instead.

### The .deb package {#linux-deb}

```bash
sudo apt install ./Signal.Lab_[[version]]_amd64.deb
```

`apt` installs what the package needs along with it. The package also puts the command
line in `/usr/bin/signallab`.

### The .rpm package {#linux-rpm}

```bash
sudo dnf install ./Signal.Lab-[[version]]-1.x86_64.rpm
```

On openSUSE, use `sudo zypper install` with the same file. Like the `.deb`, the package
puts the command line in `/usr/bin/signallab`.

### The AppImage {#linux-appimage}

The AppImage runs without installing:

```bash
chmod +x Signal.Lab_[[version]]_amd64.AppImage
./Signal.Lab_[[version]]_amd64.AppImage
```

It does not include the `signallab` command line; take it from the
[command-line archive](#cli-archives) if you need it.

### The firewall on Linux {#linux-firewall}

On Linux the app shows no firewall notice: `ufw` and `firewalld` work by port, not by
program. If one of them is on and other machines must reach a monitor, a listener, an
emulator or a relay, open its port there, for example:

```bash
sudo ufw allow 9000/udp
```

Loopback traffic is not affected.

## The command line on its own {#cli-archives}

The desktop installers bring `signallab` with them. For a machine without the app — a
build agent, a server, a lab PC — each release also has the command line on its own:

- `signallab-<version>-windows-x64.zip` holds `signallab.exe` and the license.
- `signallab-<version>-linux-x64.tar.gz` holds `signallab` and the license.

Unpack it into a folder on your `PATH`. On Linux:

```bash
tar -xzf signallab-[[version]]-linux-x64.tar.gz signallab
sudo install signallab /usr/local/bin/
signallab version
```

The server's Docker image contains it too. How to use it: [The command line](../automation/cli.md).

## Updates {#updates}

### The desktop app {#updates-desktop}

The desktop app updates itself from published releases only, and only when you say so.

- **It looks once a day.** With [[ui:update.auto]] ticked (it is, by default) the app
  looks for a newer release soon after it starts when the last look was a day or more
  ago, and again whenever another day has passed while it runs. Untick it in About to
  stop looking on its own.
- **You can look now.** Open About — the **?** in the header — and press
  [[ui:update.check]] under [[ui:update.title]].
- **What the check sends.** It asks the studio's update service (`hub.proanima.net`)
  which release this install should get, and goes to GitHub's latest release only when
  that service cannot be reached. The request carries the app's version, the system
  and processor type, and a random number made for this install, which lets a new
  release reach a share of installs first. Nothing that names you or this computer is
  sent.
- **When a release is found**, the console says so and the header shows an update
  button that opens About. There, [[ui:update.notes]] shows the release notes.
- **Nothing installs until you click.** Press [[ui:update.install]] (when jobs are
  running, the button says it will stop them). Signal Lab saves everything still
  pending, stops every running job, downloads the update and checks its signature
  against the key built into the app — an update that is not signed by the release
  process is refused — then installs it and starts again as the new version.

On Windows the update runs the setup without questions and keeps your folder, `PATH`
and firewall rules. On Linux it replaces the AppImage, or installs the new `.deb` or
`.rpm` package — for a package, the system asks for your password first.

Pre-releases and drafts are never offered. To keep every install of a machine from
looking on its own — when IT deploys updates itself, say — set the environment variable
`SIGNALLAB_NO_UPDATE_CHECK` to any value; [[ui:update.check]] still works when pressed.
Development builds never look on their own.

### A server {#updates-server}

A server, and the page a browser shows from it, updates with its Docker image, never on
its own: About says so. How to update one: [The server](../server/index.md).

## Uninstalling {#uninstall}

**Windows.** Open *Settings → Apps → Installed apps*, find Signal Lab and choose
*Uninstall*. The setup's uninstaller takes `signallab.exe` off `PATH` and, when it runs
with administrator rights (as an install for everyone does), removes every inbound
firewall rule for the app and the command line, including any the Windows prompt made.
It offers a box to delete the application data as well: that is the app's own settings
— the language, pane sizes, the screen you were on, the values last typed into the
screens — not your files. An MSI install is removed the same way; it takes its `PATH`
entry with it and leaves the firewall to whoever deployed it.

**Linux.** Remove the package with the package manager you installed it with, or delete
the AppImage file.

Neither touches your **data folder**: experiments, the signal and emulator libraries, run
reports and exports stay in `Documents/SignalLab` in your home folder. Delete that folder
yourself if you want them gone.

## Where your data lives {#data}

Everything you make is kept as plain files in one folder, `Documents/SignalLab` in your
home folder, on Windows and Linux alike: the current experiment, the signal library, the
emulator library, run reports, exports and Inspector captures. Each file and its format
is described in [Files and folders](../reference/files.md).

## Running it as a server {#server}

To use Signal Lab from a browser — on a lab PC next to the gear, on a Linux box, in
Docker — see [The server](../server/index.md). On a Linux machine with Docker, one
command installs and starts it.

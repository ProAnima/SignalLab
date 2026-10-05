---
title: 安装与更新
description: 在 Windows 或 Linux 上安装 Signal Lab 桌面应用，校验下载的文件，保持更新，以及将其卸载。
---

# 安装与更新

Signal Lab 在 GitHub 上发布：每个发布版本都包含 Windows 和 Linux 版桌面应用、单独的 `signallab` 命令行，以及一份校验和列表。如果想在服务器上运行它、通过浏览器使用，请参见[服务器](../server/index.md)。

## 下载哪个文件 {#downloads}

打开[最新发布版本](https://github.com/ProAnima/SignalLab/releases/latest)，选择适合您系统的文件。`<version>` 是该发布版本的版本号，例如 `[[version]]`。

| 文件 | 适用于 |
| --- | --- |
| `Signal.Lab_<version>_x64-setup.exe` | Windows 10 和 11，x64——大多数人需要的安装程序 |
| `Signal.Lab_<version>_x64_en-US.msi` | Windows，由 IT 部署（Intune、组策略）：为所有用户安装 |
| `Signal.Lab_<version>_amd64.deb` | Linux x86_64：Debian、Ubuntu 及其衍生发行版 |
| `Signal.Lab-<version>-1.x86_64.rpm` | Linux x86_64：Fedora、RHEL、openSUSE 及其衍生发行版 |
| `Signal.Lab_<version>_amd64.AppImage` | Linux x86_64，任何发行版，无需安装 |
| `signallab-<version>-windows-x64.zip` | 单独的命令行，适用于 Windows |
| `signallab-<version>-linux-x64.tar.gz` | 单独的命令行，适用于 Linux |
| `SHA256SUMS.txt` | 以上每个文件的 SHA-256 校验和 |

旁边的 `.sig` 文件和 `latest.json` 供应用自身的更新程序使用，您不需要它们。

## Windows {#windows}

Signal Lab 运行于 Windows 10 和 11（x64）。它使用 WebView2 运行时，这两个系统都已自带。

### 使用安装程序安装 {#windows-setup}

1. 运行 `Signal.Lab_<version>_x64-setup.exe`。
2. 如果 Windows SmartScreen 提示已保护您的电脑，请选择**更多信息**，然后选择**仍要运行**（参见下文的 [SmartScreen](#smartscreen)）。
3. 选择安装程序的语言。它提供与应用相同的 11 种语言。
4. 选择为谁安装：
   - 仅为自己：安装到 `%LOCALAPPDATA%\Signal Lab`，不需要管理员权限。
   - 为使用这台电脑的所有人：安装到 `C:\Program Files\Signal Lab`，需要管理员权限。
5. 确认文件夹并安装。
6. 最后一页可以选择立即启动 Signal Lab，并创建桌面快捷方式。

在已安装的版本上运行新版本的安装程序，会就地升级。

### 安装程序额外添加的内容 {#windows-setup-adds}

除了应用本身，安装程序还会添加两样东西，卸载程序会将两者一并移除：

- **命令行**。`signallab.exe` 放在应用旁边，其文件夹会加入 `PATH`——仅为自己安装时是您自己的 `PATH`，为所有人安装时则是整台计算机的——因此此后打开的每个终端中都能使用 `signallab`。参见[命令行](../automation/cli.md)。
- **一条防火墙规则**，仅在为所有人安装时添加。Windows Defender 防火墙只有在有规则允许时，才让其他机器访问某个程序；程序第一次监听时，它会询问屏幕前的人，而在那里点击“取消”，其他机器发来的任何内容都会被悄无声息地丢弃。为所有人安装时具有管理员权限，会为 `signal-lab.exe`（应用）和 `signallab.exe`（命令行）各添加一条入站允许规则，适用于专用网络和域网络。仅为自己安装时无法更改防火墙：应用会在第一次有功能开始监听时，通过 Windows 自己的管理员提示提出添加——参见[防火墙提示](interface.md#firewall-notice)。

本机自身的流量（`127.0.0.1`）从不被过滤，因此在环回地址上进行的一切操作无需任何规则即可工作。

### 无人值守安装 {#windows-unattended}

安装程序在命令行上接受以下开关：

| 开关 | 作用 |
| --- | --- |
| `/S` | 静默安装，不询问任何问题。 |
| `/P` | 显示进度条安装，不询问任何问题。 |
| `/ALLUSERS` | 为所有人安装（需要以管理员身份运行的命令提示符）。 |
| `/CURRENTUSER` | 仅为当前用户安装。 |
| `/NS` | 不创建快捷方式。 |
| `/D=C:\Tools\Signal Lab` | 安装到此文件夹。它必须放在最后，且不加引号。 |
| `/NOPATH` | 不修改 `PATH`：会安装 `signallab`，但不加入 `PATH`。 |
| `/NOFIREWALL` | 不添加防火墙规则。 |

例如，在以管理员身份运行的命令提示符中，为所有人静默安装，且不添加防火墙规则：

```powershell
.\Signal.Lab_[[version]]_x64-setup.exe /S /ALLUSERS /NOFIREWALL
```

### MSI {#windows-msi}

`Signal.Lab_<version>_x64_en-US.msi` 用于由 IT 部署。它为所有用户安装，把 `signallab.exe` 放在应用旁边，并将该文件夹加入计算机的 `PATH`，但不添加防火墙规则：这交由部署者处理（例如通过组策略）。应用第一次监听时仍会提出添加规则。静默安装：

```powershell
msiexec /i "Signal.Lab_[[version]]_x64_en-US.msi" /qn
```

### SmartScreen {#smartscreen}

安装程序目前尚未进行代码签名，因此 Windows SmartScreen 不认识其发布者，可能会用警告阻止安装程序运行。请选择**更多信息**，确认该文件就是您从发布页面下载的文件，然后选择**仍要运行**。要确保文件未被改动，请先[用 `SHA256SUMS.txt` 校验它](#checksums)。

应用自身的更新经过签名，并会单独校验；参见[更新](#updates)。

## 校验下载的文件 {#checksums}

`SHA256SUMS.txt` 列出发布版本中每个文件的 SHA-256 校验和，每行一个，后面跟着文件名。把它下载到文件旁边，然后进行比较。

在 Windows 上，使用 PowerShell：

```powershell
Get-FileHash .\Signal.Lab_[[version]]_x64-setup.exe -Algorithm SHA256
Select-String "Signal.Lab_[[version]]_x64-setup.exe" .\SHA256SUMS.txt
```

两个哈希值必须相同（`Get-FileHash` 以大写字母输出；大小写无关紧要）。

在 Linux 上，在同时存放这两个文件的文件夹中：

```bash
sha256sum --check --ignore-missing SHA256SUMS.txt
```

它会在找到的每个文件后输出 `OK`。任何其他输出都表示下载的文件不是发布的文件：请重新下载。

## Linux {#linux}

Linux 版应用为 x86_64 构建（在 Ubuntu 22.04 上），需要 WebKitGTK 4.1，即用来绘制其窗口的 Web 视图。没有面向 Arm 处理器的版本；在 Arm 机器上，请改为运行[服务器](../server/index.md)。

### .deb 软件包 {#linux-deb}

```bash
sudo apt install ./Signal.Lab_[[version]]_amd64.deb
```

`apt` 会一并安装该软件包所需的依赖。该软件包还会把命令行放在 `/usr/bin/signallab`。

### .rpm 软件包 {#linux-rpm}

```bash
sudo dnf install ./Signal.Lab-[[version]]-1.x86_64.rpm
```

在 openSUSE 上，对同一个文件使用 `sudo zypper install`。与 `.deb` 一样，该软件包会把命令行放在 `/usr/bin/signallab`。

### AppImage {#linux-appimage}

AppImage 无需安装即可运行：

```bash
chmod +x Signal.Lab_[[version]]_amd64.AppImage
./Signal.Lab_[[version]]_amd64.AppImage
```

它不包含 `signallab` 命令行；如有需要，请从[命令行压缩包](#cli-archives)中获取。

### Linux 上的防火墙 {#linux-firewall}

在 Linux 上，应用不会显示防火墙提示：`ufw` 和 `firewalld` 按端口而不是按程序工作。如果其中之一已开启，而其他机器需要访问监视器、监听器、模拟器或中继，请在其中开放相应的端口，例如：

```bash
sudo ufw allow 9000/udp
```

环回流量不受影响。

## 单独的命令行 {#cli-archives}

桌面安装程序自带 `signallab`。对于没有安装应用的机器——构建代理、服务器、实验室电脑——每个发布版本还单独提供命令行：

- `signallab-<version>-windows-x64.zip` 包含 `signallab.exe` 和许可证。
- `signallab-<version>-linux-x64.tar.gz` 包含 `signallab` 和许可证。

将其解压到 `PATH` 中的某个文件夹。在 Linux 上：

```bash
tar -xzf signallab-[[version]]-linux-x64.tar.gz signallab
sudo install signallab /usr/local/bin/
signallab version
```

服务器的 Docker 镜像中也包含它。用法见[命令行](../automation/cli.md)。

## 更新 {#updates}

### 桌面应用 {#updates-desktop}

桌面应用只从已发布的版本自行更新，并且只在您同意时更新。

- **每天自动检查**。勾选 [[ui:update.auto]] 时（默认勾选），如果上次检查已是一天或更久以前，应用会在启动后不久查找新版本；运行期间每过一天也会再次检查。在“关于”中取消勾选，即可停止自动检查。
- **可以立即检查**。打开“关于”（顶栏中的 **?**），然后在 [[ui:update.title]] 下按 [[ui:update.check]]。
- **检查时发送什么**。它会询问工作室的更新服务（`hub.proanima.net`）此安装应获取哪个版本，只有在无法连接该服务时才转向 GitHub 上的最新发布版本。请求中包含应用的版本、系统和处理器类型，以及为此安装生成的一个随机数，借助它，新版本可以先推送给一部分安装。不会发送任何能识别您或这台电脑的信息。
- **找到新版本时**，控制台会给出提示，顶栏会显示一个更新按钮，点击可打开“关于”。在那里，[[ui:update.notes]] 显示发布说明。
- **在您点击之前，不会安装任何内容**。按 [[ui:update.install]]（有任务正在运行时，按钮会注明将停止它们）。Signal Lab 会保存所有尚未保存的内容，停止每个正在运行的任务，下载更新，并用应用内置的密钥校验其签名——未经发布流程签名的更新会被拒绝——然后安装它，并以新版本重新启动。

在 Windows 上，更新会以无提问的方式运行安装程序，并保留您的文件夹、`PATH` 和防火墙规则。在 Linux 上，它会替换 AppImage，或安装新的 `.deb` 或 `.rpm` 软件包；对于软件包，系统会先要求您输入密码。

预发布版本和草稿永远不会被推送。若要让一台机器上的每个安装都不自动检查——例如由 IT 自行部署更新时——请将环境变量 `SIGNALLAB_NO_UPDATE_CHECK` 设为任意值；按下 [[ui:update.check]] 时仍会检查。开发版本从不自动检查。

### 服务器 {#updates-server}

服务器以及浏览器从它打开的页面都随其 Docker 镜像更新，从不自行更新：“关于”中会注明这一点。如何更新服务器：参见[服务器](../server/index.md)。

## 卸载 {#uninstall}

**Windows**。打开“设置 → 应用 → 已安装的应用”，找到 Signal Lab，然后选择“卸载”。安装程序的卸载程序会把 `signallab.exe` 从 `PATH` 中移除；当它以管理员权限运行时（为所有人安装时即如此），还会删除应用和命令行的所有入站防火墙规则，包括 Windows 提示所创建的规则。它还提供一个复选框，可同时删除应用程序数据：即应用自身的设置——语言、窗格大小、您所在的界面、最后在各界面中输入的值——而不是您的文件。MSI 安装以同样的方式卸载；它会一并移除其 `PATH` 条目，防火墙则留给部署者处理。

**Linux**。用安装时所用的软件包管理器移除软件包，或删除 AppImage 文件。

两者都不会动您的**数据文件夹**：实验、信号库和模拟器库、运行报告和导出文件都保留在主文件夹下的 `Documents/SignalLab` 中。如果不再需要，请自行删除该文件夹。

## 数据存放位置 {#data}

您创建的一切都以普通文件的形式保存在一个文件夹中，即主文件夹下的 `Documents/SignalLab`，Windows 和 Linux 都一样：当前实验、信号库、模拟器库、运行报告、导出文件和检查器捕获。每个文件及其格式见[文件与文件夹](../reference/files.md)。

## 作为服务器运行 {#server}

要通过浏览器使用 Signal Lab——在设备旁的实验室电脑上、在 Linux 主机上、在 Docker 中——请参见[服务器](../server/index.md)。在装有 Docker 的 Linux 机器上，一条命令即可完成安装并启动。

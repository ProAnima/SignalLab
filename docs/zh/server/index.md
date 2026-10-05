---
title: 运行服务器
description: 在设备旁的 Linux 机器上运行 Signal Lab，并从网络上的任意浏览器使用它，附带供脚本和 CI 使用的 HTTP API。
---

# 以服务器方式运行 Signal Lab

`signal-lab-server` 是没有窗口的 Signal Lab：同一个引擎，向浏览器提供同一个界面。把它放在设备旁的机器上——一台机架 PC、一台演出控制虚拟机、一台共享的实验室机器——然后从网络上任意位置的 Chrome、Firefox 或 Edge 打开它：每个界面都和桌面应用中一样工作，运行、报告和导出都通过浏览器下载。

脚本和流水线通过其 [HTTP API](../api/index.md) 使用同一个服务器，[`signallab --server`](../automation/cli.md#run-on-server) 则把运行发送给它。

与桌面应用有几处不同：

- **一个服务器就是一个引擎。** 登录到它的每个页面看到的都是相同的正在运行的作业、相同的信号库和模拟器库，以及 [[ui:nav.http]] 界面（[[ui:http.keepCookies]]）的同一个 Cookie 罐。共享服务器的人共享这些东西。
- **机密属于服务器**，从其环境或文件读取；无法从浏览器设置（见[机密](#secrets)）。
- **防火墙属于主机。** 服务器从不更改它；桌面应用的防火墙提示不会出现。
- **它随其镜像更新**，而不是通过应用的更新器（见[更新](#update)）。

## 在 Linux 主机上，一条命令完成 {#install-script}

在一台可访问互联网的 Linux 机器上：

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh
```

脚本会：

1. 如果缺少 Docker 就安装它——它会先询问，并使用 Docker 自己的安装程序（`get.docker.com`）；
2. 把一个 `compose.yaml` 写入 `/opt/signallab`（当您不是 root 时是 `~/signallab`）；
3. 拉取镜像并使用主机网络启动服务器，让 OSC、UDP、广播、组播和发现到达真实网络；
4. 等待服务器通过其健康检查（最多 90 秒）；
5. 输出要打开的地址、用于登录的访问令牌，以及更新、读取日志和移除它的命令；
6. 当 ufw 或 firewalld 开启时，提出开放服务器的端口（见[主机的防火墙](#firewall)）。

把选项放在 `sh -s --` 之后：

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh -s -- --version [[version]] --port 8430
```

| 选项 | 作用 | 默认值 |
| --- | --- | --- |
| `--version X.Y.Z` | 镜像版本（`latest` 或 `X.Y.Z`；开头的 `v` 会被去掉）。 | `latest` |
| `--port N` | 浏览器使用的端口。 | `1430` |
| `--listen IP:PORT` | 只在一个地址上监听。 | `0.0.0.0:<port>` |
| `--dir DIR` | compose 文件放在哪里。 | 以 root 运行时为 `/opt/<name>`，否则为 `~/<name>` |
| `--name NAME` | 容器及其数据卷：小写字母、数字、`-` 和 `_`。同一主机上的第二个服务器需要有自己的名称。 | `signallab` |
| `--image NAME` | 另一个镜像或仓库；完整的 `NAME:TAG` 会按原样使用。 | `ghcr.io/proanima/signallab` |
| `--open-udp PORTS` | 开启防火墙时，也允许 UDP 在这些端口上进入，用于监视器和等待：`9000,9100:9110`。 | |
| `--no-firewall` | 从不更改 ufw 或 firewalld。 | |
| `--yes`、`-y` | 回答是：安装 Docker、开放防火墙、用 `--purge` 删除数据。 | |
| `--uninstall` | 停止并移除服务器；数据保留。 | |
| `--purge` | 与 `--uninstall` 一起：连数据和令牌一并删除。 | |
| `--help`、`-h` | 输出选项。 | |

它需要 Docker 的 Compose 插件（`docker-compose-plugin` 包），Docker 的安装程序会带来它。镜像为 x86_64 和 arm64 构建。

**再次运行即可更新：**同一条命令会拉取最新的镜像（或您指定的 `--version`）并重启服务器；数据和令牌保留。使用 `--name` 时，请再次给出相同的名称。

**您自己的设置**——实验机密、`SIGNALLAB_ALLOWED_HOSTS`、HTTPS 之后的 `SIGNALLAB_SECURE_COOKIE`——放在 compose 文件旁边的 `compose.override.yaml` 中。Docker Compose 会把它合并进来，脚本每次运行都会重写 `compose.yaml`，但从不改动 override。不是它写的 `compose.yaml` 会保留为 `compose.yaml.before-install`。

```yaml
# compose.override.yaml
services:
  signallab:
    environment:
      SIGNALLAB_ALLOWED_HOSTS: lab-pc.example.com,192.0.2.10
      SIGNALLAB_SECRET_API_TOKEN: ${API_TOKEN}
```

**要移除它：**

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh -s -- --uninstall
```

数据保留在其 Docker 卷中，再次安装会带着相同的令牌把它带回来。`--uninstall --purge` 在询问之后连数据和令牌一并删除。

### 主机的防火墙 {#firewall}

使用主机网络时，服务器监听主机自己的端口，因此由主机的防火墙决定谁能到达它。当 ufw 或 firewalld 开启时，脚本会在开放服务器的 TCP 端口和 `--open-udp` 的 UDP 端口之前先询问，并记录它开放了什么（compose 文件旁边的 `.firewall`），而 `--uninstall` 会恰好再次关闭这些端口。如果您拒绝，其他机器上的浏览器只有在防火墙放行后才能到达服务器，而监视器和等待只有在防火墙开放 UDP 端口时才能听到其他机器的消息。

## Docker {#docker}

### 镜像 {#image}

`ghcr.io/proanima/signallab`，用于 `linux/amd64` 和 `linux/arm64`，随每个版本发布：

| 标签 | 它是什么 |
| --- | --- |
| `X.Y.Z` | 那个版本。 |
| `X.Y` | 该系列最新的稳定版本。 |
| `latest` | 最新的稳定版本。 |

它包含 `signal-lab-server`、构建好的界面和命令行 `signallab`，并用这些设置启动服务器：

| 变量 | 镜像中的值 |
| --- | --- |
| `SIGNALLAB_LISTEN` | `0.0.0.0:1430` |
| `SIGNALLAB_DATA_DIR` | `/data` |
| `SIGNALLAB_UI_DIR` | `/usr/share/signal-lab/ui` |
| `SIGNALLAB_GENERATE_TOKEN` | `true` |

它以非特权用户（uid 和 gid 均为 10001）运行，只写入 `/data`（一个卷），暴露端口 `1430`，并每 30 秒检查一次自己的健康状态。

### 启动它 {#docker-run}

```bash
docker run -d --name signallab --network host --restart unless-stopped \
  -v signallab-data:/data --read-only --cap-drop ALL --security-opt no-new-privileges \
  ghcr.io/proanima/signallab:[[version]]
docker logs signallab                    # on the first start: "Sign in with it:" and the token
```

然后打开 `http://<host>:1430` 并用令牌登录。以后要再次查看令牌：

```bash
docker exec signallab cat /data/token
```

`--read-only`、`--cap-drop ALL` 和 `no-new-privileges` 是可选的，且不花任何代价：服务器不需要任何特权，只写入 `/data`。

### Docker Compose {#compose}

仓库中的 `deploy/compose.yaml` 与一个 Compose 文件相同：

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

设置 `SIGNALLAB_IMAGE=ghcr.io/proanima/signallab:X.Y.Z` 可以固定版本。

### 网络 {#networking}

| Docker 网络 | 什么有效 | 什么无效 |
| --- | --- | --- |
| `--network host`（`network_mode: host`），在 Linux 主机上 | 一切：到局域网的 OSC、UDP、TCP、HTTP、WebSocket 和 MQTT，监听端口，广播、组播、发现。 | — |
| 桥接（默认），带已发布端口 | 到容器可到达主机的单播；已发布端口上的监听器（`-p 1430:1430 -p 9000:9000/udp`）。 | 广播和组播；对未发布端口的应答。 |
| Windows 或 macOS 上的 Docker Desktop | 单播和已发布的监听器。 | 到物理网络的主机网络。在 Windows 上，请使用桌面应用。 |

### 数据卷 {#data-volume}

`/data` 保存服务器保留的一切：实验、信号库和模拟器库、运行报告、导出，以及令牌。像上面那样的命名卷，一开始就归镜像的用户所有。挂载到那里的一块主机文件夹必须可由 uid 10001 写入：

```bash
sudo mkdir -p /srv/signallab && sudo chown 10001:10001 /srv/signallab
docker run -d --name signallab --network host -v /srv/signallab:/data ghcr.io/proanima/signallab:[[version]]
```

## 不使用 Docker {#binary}

服务器以镜像形式发布。要直接运行 `signal-lab-server`，请从源代码构建它（见[构建](../../develop/building.md)）：

```bash
npm install
npm run build                            # the interface, into dist/
cargo run --release -p signal-lab-server
```

它在 `http://127.0.0.1:1430` 上提供 `dist/`，仅限这台机器，无需令牌。加上 `--listen` 和一个令牌即可向网络开放。

## 选项 {#options}

每个选项都有一个环境变量，供容器使用。选项优先于变量。

| 选项 | 变量 | 默认值 | 作用 |
| --- | --- | --- | --- |
| `--listen IP:PORT` | `SIGNALLAB_LISTEN` | `127.0.0.1:1430` | 在哪里监听。除环回外的任何地址都需要令牌。 |
| `--token-file PATH` | `SIGNALLAB_TOKEN_FILE` | | 保存访问令牌的文件（比如 Docker 机密）。 |
| `--token TOKEN` | `SIGNALLAB_TOKEN` | | 访问令牌本身。优先使用文件：参数对机器上的其他用户可见。 |
| `--generate-token` | `SIGNALLAB_GENERATE_TOKEN` | 关闭 | 没有给出令牌、且地址超出环回时：使用保存在 `<data folder>/token` 中的令牌，并在首次启动时生成它。 |
| `--data-dir PATH` | `SIGNALLAB_DATA_DIR` | 用户 home 文件夹中的 `Documents/SignalLab` | 数据文件夹。 |
| `--secrets-dir PATH` | `SIGNALLAB_SECRETS_DIR` | `/run/secrets/signallab` | 只读机密的文件夹，每个名称一个文件。 |
| `--ui-dir PATH` | `SIGNALLAB_UI_DIR` | 程序旁边的 `ui`，否则为 `./dist` | 构建好的界面。没有它时只提供 API。 |
| `--allowed-host NAME` | `SIGNALLAB_ALLOWED_HOSTS` | | 可以访问服务器的主机名，以逗号分隔。它们和环回名称始终被接受，无论有无令牌；没有给出时，带令牌的服务器应答任何名称，不带令牌的只应答环回名称（见[主机名](security.md#hosts)）。 |
| `--secure-cookie` | `SIGNALLAB_SECURE_COOKIE` | 关闭 | 只通过 HTTPS 发送会话 Cookie。在 HTTPS 代理之后设置它。 |
| `--log FILTER` | `SIGNALLAB_LOG` | `info` | 记录什么：`error`、`warn`、`info`、`debug`，或按模块（`signal_lab_server=debug`）。 |
| `--log-format text\|json` | `SIGNALLAB_LOG_FORMAT` | `text` | 日志行作为文本，或每行一个 JSON 对象。 |

开关从它们的变量接受 `true` 或 `false`：
`SIGNALLAB_GENERATE_TOKEN=true`。

| 命令 | 作用 |
| --- | --- |
| `signal-lab-server token` | 输出一个新的随机令牌：64 个十六进制字符。 |
| `signal-lab-server healthcheck` | 当有服务器在 `--listen` 上应答时以 `0` 退出（镜像的健康检查）。 |
| `signal-lab-server --version` | 输出版本。 |
| `signal-lab-server --help` | 输出每个选项。 |

**退出代码：**由 <kbd>Ctrl</kbd>+<kbd>C</kbd> 或 `SIGTERM` 停止时为 `0`；设置被拒绝时为 `2`（在可到达的地址上没有令牌、令牌太短、令牌给出了两次、无法读取令牌文件、`--generate-token` 没有数据文件夹）；无法在该地址上监听，或无法写入数据文件夹时为 `1`。原因会输出到 stderr。

## 访问令牌 {#token}

没有令牌时，服务器只在环回地址上监听，且只服务这台机器。在任何其他地址上它都需要令牌，没有令牌就拒绝启动。有三种提供方式：

| 方式 | 何时使用 |
| --- | --- |
| `--generate-token`（镜像中开启） | 无需设置：首次启动时服务器生成一个令牌，保存在 `<data folder>/token` 中——仅其自己的用户可读——并在日志中输出一次。以后的启动会复用它，因此在重启和更新之间，已登录的浏览器和脚本仍可工作。它需要一个数据文件夹（`--data-dir`）。 |
| `--token-file PATH` | 您自己的令牌放在文件中，例如 Docker 机密。它末尾的换行符不属于令牌。 |
| `SIGNALLAB_TOKEN` | 环境中的令牌。 |

令牌至少有 **24** 个字符，且不含空格或换行符；只能以一种方式提供。`signal-lab-server token` 能生成一个好的令牌：

```bash
docker run --rm ghcr.io/proanima/signallab:[[version]] token > signallab_token.txt
```

而 Compose 会把它作为机密传入（文件必须可由 uid 10001 读取）：

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

显式给出的令牌优先于 `--generate-token`，此时不会生成任何令牌。要更改已生成的令牌，请停止服务器，删除 `<data folder>/token`，然后再次启动：它会生成并输出一个新的。损坏的令牌文件会被报告，绝不会被替换。

## 登录 {#sign-in}

打开 `http://<host>:1430`。带令牌的服务器会先把浏览器带到其登录页面，语言与浏览器一致；粘贴一次令牌，浏览器就会保持登录 7 天。界面中的 [[ui:app.signOut]] 会结束会话。会话保存在服务器的内存中：重启会让所有人退出登录。

在环回地址上且没有令牌时，无需登录。

脚本在每个请求中都带上令牌，形式为 `Authorization: Bearer <token>`：

```bash
curl -fsS http://192.0.2.10:1430/api/invoke/app_info \
  -H "Authorization: Bearer $(cat signallab_token.txt)" \
  -H "Content-Type: application/json" -d 'null'
```

参见 [HTTP API](../api/index.md)，以及[服务器安全](security.md)了解这一切背后的规则。

## 数据文件夹 {#data}

服务器把文件保存在数据文件夹中：实验、信号库和模拟器库、运行报告、导出，以及生成的令牌。它是 `--data-dir`（`SIGNALLAB_DATA_DIR`），在镜像中是 `/data`，否则是它运行用户的 home 文件夹中的 `Documents/SignalLab`。用 `--data-dir` 指定的数据文件夹若缺失会被创建，并在启动时检查：如果服务器无法写入那里，它会停止并指出该文件夹。参见[文件](../reference/files.md)。

下载（报告、导出）只来自这个文件夹内部。

## 机密 {#secrets}

实验以 `{{secret.NAME}}` 读取机密。在服务器上它们只读，来自：

1. 环境变量 `SIGNALLAB_SECRET_NAME`，否则
2. 机密文件夹中的文件 `NAME`（`--secrets-dir`，默认为
   `/run/secrets/signallab`——Docker 机密的布局）。

文件末尾的换行符不属于值；空值等同于未设置；值最多 16 KiB。名称由字母、数字和 `_` 组成，且不以数字开头。界面会显示服务器上设置了哪些机密，但从浏览器设置会被拒绝——值永远不会去往更不安全的地方，也永远不会再出来（见[机密](security.md#secrets)）。

使用 Compose 时，每个名称一个机密文件：

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

该文件必须可由 uid 10001 读取。

## 在 HTTPS 代理之后 {#https}

服务器使用纯 HTTP。要使用 HTTPS，请在它前面放置一个反向代理（Caddy、nginx、Traefik），它：

- 原样传递 `Host` 标头；
- 传递 WebSocket 升级（界面保持一个到 `/api/events` 的连接）；

并用 `--secure-cookie` 启动服务器，让会话 Cookie 只通过 HTTPS 传输，同时把 `--allowed-host` 设置为人们使用的名称。

## 更新 {#update}

[[ui:update.server]]：桌面应用的更新器与此无关。

- 用脚本安装的：再次运行同一条命令。
- 使用 Compose 的：`docker compose pull && docker compose up -d`。
- 使用 `docker run` 的：拉取新镜像，然后移除容器并用同一个卷再次启动它。

数据和令牌在卷中，因此会保留。

## 健康检查 {#health}

`GET /api/health` 对所有人应答，无需令牌：

```bash
curl -s http://127.0.0.1:1430/api/health
# {"auth":true,"status":"ok","version":"[[version]]"}
```

`auth` 表示服务器是否要求令牌。命令 `signal-lab-server healthcheck` 在本机上做同样的询问，并在得到应答时以 `0` 退出；镜像每 30 秒运行一次（超时 5 秒，尝试 3 次），因此 `docker ps` 会把容器显示为健康。

## 日志 {#logs}

服务器记录到 stdout：`docker logs -f signallab`。在默认级别（`info`）下，它会说明在哪里监听、是否需要令牌、其数据和界面文件夹、每个启动的作业——监视器、生成器、风暴、扫描、运行、模拟器——连同客户端的地址、每次登录、每次使用错误令牌的登录（作为警告），以及页面何时落后于事件。`--log debug` 级别会增加每条命令。只有在终端上才会给行着色（设置了 `NO_COLOR` 时从不着色）；`--log-format json` 每行写一个 JSON 对象，供日志收集器使用。

## 停止 {#stop}

<kbd>Ctrl</kbd>+<kbd>C</kbd>、`docker stop` 或 `SIGTERM` 会关闭每个页面的连接、停止每个作业——进行中的运行会以已停止结束——并以 `0` 退出。compose 文件为此给它 15 秒。

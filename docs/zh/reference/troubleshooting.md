---
title: 故障排除
description: Signal Lab 的常见问题及其解决方法——什么都收不到、端口被占用、防火墙、Docker 网络、服务器、MQTT、证书、更新和日志。
---

# 故障排除

Signal Lab 报告的每个失败都有一个代码；每个代码的消息见[错误消息](errors.md)。网络失败是 [`transport`](errors.md#transport) 代码：`refused`、`timeout`、`dns`、`unreachable`、`reset`、`address_in_use`、`address_unavailable`、`denied`、`tls`、`target_invalid`、`failed`。下面是常见的问题。

## 什么都收不到 {#nothing-arrives}

首先弄清楚是否有任何内容到达 Signal Lab：打开底部面板中的 [[ui:dock.inspector]] 标签页，然后按 [[ui:ins.arm]]。工具发送或接收的每个数据报、请求和消息都会列在那里，并注明它来自哪里。

### 监听地址 {#listen-address}

- `0.0.0.0:<port>` 的 [[ui:common.bind]] 会监听每一块网卡；`127.0.0.1:<port>` 只听本机。网络上的设备需要前者。
- 设备必须发送到本机的地址以及您正在监听的端口。顶栏会显示本机的名称和地址。
- 不是本机的地址会以 `address_unavailable` 失败。

### 防火墙 {#firewall}

`127.0.0.1` 上的流量从不被过滤，这就是为什么在同一台机器上测试有效，而从另一台机器进行的同样测试却什么都收不到。

**Windows。** Windows 防火墙按程序决定。Windows 通常会在程序首次监听时询问一次——在那里选择*取消*会留下一条阻止它的规则，这条规则优先于任何允许规则。在 Windows 称为公用（通常是场所的 Wi-Fi）的网络上，它可能根本不询问。

- 当监视器、发现监听器、中继、运行或模拟器开始监听时，桌面应用会查看一次防火墙。当防火墙构成阻碍时，它会用一条带 [[ui:fw.allow]] 的提示说明——在公用网络上则是 [[ui:fw.allowPublic]]。Windows 会请求管理员权限，然后该程序的入站规则（包括阻止规则）会被替换为一条允许规则。[[ui:fw.dismiss]] 会隐藏提示。
- 从终端：`signallab doctor` 会显示有什么在阻碍，`signallab firewall allow` 会修复它（公用网络再加 `--public`），并出现同样的管理员提示。
- *为所有人*安装的安装程序（`.exe`）会自行添加允许规则（专用网络和域网络），除非它以 `/NOFIREWALL` 运行。*为我*安装的安装程序则不能，而 `.msi` 把防火墙留给部署它的人。
- 服务器从不更改其主机的防火墙：由它的管理员开放端口（安装脚本会提出用 ufw 或 firewalld 开放）。

**Linux。** ufw 或 firewalld 之类的防火墙按端口而不是按程序工作。`signallab doctor` 会指出哪一个在运行，以及如何开放端口，例如 `sudo ufw allow 9000/udp`。

### 广播和组播 {#broadcast-multicast}

- 路由器不转发广播：`255.255.255.255` 和 `x.x.x.255` 只到达发送网卡所在的网段。有多块网卡时，把网卡的地址填入 [[ui:bc.bindSource]]（在 [[ui:bc.socketOptions]] 下），例如 `10.0.0.5:0`。
- 组播数据报只到达加入了其组的监听器——在发现监听器上，即 [[ui:bc.joinGroups]]。[[ui:bc.ttl]] 为默认值 1 时，它停留在本网络内。
- 要在同一台机器上听到自己的组播，请让 [[ui:bc.mcastLoop]] 保持开启。

### 不应答的设备 {#no-answer}

UDP 没有送达回执：发往无人监听的端口的数据报仍算作已发送。随后 Windows 会把收到的 ICMP *port unreachable* 报告为该套接字下一次接收时的连接重置；Signal Lab 的监视器、监听器、中继和等待会忽略它并继续监听。因此当应答没有来时，等待会在其时间过后以 `wait.timeout` 失败，而不是以关于发送的错误失败。请在 [[ui:dock.inspector]] 中检查消息是否发往了正确的地址，然后再检查设备。

## 发送在发出之前被拒绝 {#refused-send}

这些会先被检查，其中一项失败时不会发送任何内容：

| 代码 | 原因 | 修复 |
| --- | --- | --- |
| `transport.target_invalid` | 目标没有端口，或者既不是 `IP:port` 也不是 `host:port` | 两者都写，例如 `192.0.2.20:9000` |
| `transport.dns` | 主机名在这台机器上无法解析 | 检查名称，或改用地址。带 IPv4 地址的名称会通过 IPv4 到达，因此 `localhost:9000` 会在 `127.0.0.1` 上找到接收者 |
| `node.osc_address` | OSC 地址不以 `/` 开头——在发送器、生成器、信号或某个步骤中 | 以 `/` 开头，例如 `/cue/go` |
| `node.topic_wildcard` | MQTT 发布的主题含有 `+` 或 `#`，界面上的连接以及其他地方都是如此 | 只发布到一个主题；通配符用于订阅 |
| WebSocket 消息上的 `node.too_long` | 消息超过 16 MiB | 少发送一些；连接保持打开 |

## 端口已被占用 {#port-in-use}

`transport.address_in_use`：另一个程序——或 Signal Lab 的另一个作业——已经在监听该端口。

- **监视器和运行。** 运行会在其第一个步骤之前打开其等待、模拟器和中继的端口，因此被 [[ui:osc.monitor]]、发现监听器或模拟器作业占用的端口会让运行在开始前就失败。请先停止那个作业；[[ui:app.stopAll]] 会停止每一个。
- **一个运行中的两个模拟器**不能共享同一传输的端口：HTTP、MQTT 和 TCP 模拟器都监听 TCP，OSC 和 UDP 模拟器则监听 UDP（`emulator.bind_taken`）。
- **与真实服务同时监听。** 发现监听器可以与已经占用端口的程序共享端口：让 [[ui:bc.reuse]] 保持开启。在 Linux 上，那个程序也必须共享其端口。否则，被占用的端口会报 `broadcast.port_shared`。
- **刚刚停止。** 已停止的模拟器或运行所占用的端口会在稍后释放；立即再次启动的模拟器会短暂等待它。
- Linux 上的 **1024 以下端口**需要管理员权限（`denied`）。服务器镜像没有任何权限运行，因此请使用 1024 或以上的端口。

## Docker 中的服务器连不上网络 {#docker-network}

广播、组播和发现只有在使用主机网络时才能到达物理网络——compose 文件中的 `network_mode: host`，或 `docker run --network host`——并且只在 Linux 主机上。它还会让监视器、等待和模拟器监听主机自己的端口。使用 Docker 默认的桥接网络时，容器处在一个自己的网络上：广播和组播永远不会离开它，只有您发布的端口才能到达它。

在 Windows 和 macOS 上，Docker 的主机网络无法到达物理网络：在 Windows 上请使用桌面应用，或在 Linux 主机上运行服务器。

## SmartScreen 对安装程序发出警告 {#smartscreen}

安装程序尚未签名，因此 Windows SmartScreen 会说它不认识发布者。选择*更多信息*，然后选择*仍要运行*。请只从该项目在 GitHub 上的发布版本下载安装程序。

## 服务器无法启动 {#server-start}

`signal-lab-server` 会在监听之前检查其设置，并在其错误输出上带消息退出：

| 退出代码 | 消息 | 修复 |
| --- | --- | --- |
| 2 | refusing to listen on … without a token | 其他人可访问的服务器需要令牌：`--token-file` 或 `SIGNALLAB_TOKEN`（用 `signal-lab-server token` 生成一个），或 `--generate-token`。或者监听 `127.0.0.1` |
| 2 | the token has N characters; it needs at least 24 | 使用更长的令牌 |
| 2 | the token must not contain spaces or line breaks | 令牌文件可以以换行符结尾；除此之外不行 |
| 2 | give the token once | 使用 `--token`/`SIGNALLAB_TOKEN` 或 `--token-file`/`SIGNALLAB_TOKEN_FILE`，不要同时使用两者 |
| 2 | --generate-token keeps the token in the data folder | 设置 `--data-dir` 或 `SIGNALLAB_DATA_DIR` |
| 2 | … it does not hold a valid token; remove it to have a new one made | 数据文件夹中的 `token` 文件已损坏 |
| 2 | cannot read the token file … | `--token-file` 指定的文件缺失，或此用户无法读取 |
| 2 | cannot save the new token in … | `--generate-token` 无法在数据文件夹中写入 `token`：请让此用户可写该文件夹 |
| 1 | cannot listen on … | 该地址不是本机的，或端口被占用 |
| 1 | the data folder … must be writable by this user | 在 Docker 中，绑定挂载的文件夹必须可由 uid 10001 写入 |

由 `--generate-token` 生成的令牌只在首次启动时输出一次（`docker logs signallab` 会显示它），并保存在数据文件夹的 `token` 中：`docker exec signallab cat /data/token`。参见[服务器](../server/index.md)。

## 无法登录服务器 {#sign-in}

| 您看到的内容 | 原因 | 修复 |
| --- | --- | --- |
| *That token is not right.* | 令牌错误 | 从服务器保存它的地方重新复制；这个应答故意要花一秒 |
| `auth.host` | 服务器不应答地址栏中的名称 | 用它接受的名称打开。环回名称（`localhost`、`127.x.x.x`、`[::1]`）总是通过。没有令牌时，服务器只应答这些名称以及 `--allowed-host` 中的名称；有令牌时，除非 `--allowed-host` 加以限定，否则应答任何名称 |
| `auth.origin` | 请求来自另一个来源的页面 | 在反向代理之后，把浏览器的 `Host` 传给服务器（nginx：`proxy_set_header Host $host;`），让 `Origin` 与 `Host` 一致 |
| 登录后又回到登录页面 | 浏览器没有保留会话 Cookie | 使用 `--secure-cookie` 时，必须通过 HTTPS 访问服务器 |
| 过一段时间后被登出 | 会话持续 7 天，并在服务器重启时结束；超过 1024 个会话时，最旧的会退出 | 重新登录 |

## 与服务器的连接不断断开 {#connection-lost}

[[ui:app.connectionLost]] 表示页面的事件套接字 `/api/events` 已关闭。页面会自行重连，先等半秒，然后间隔逐渐变长，最多每 15 秒一次。期间发生的事不会重放：正在运行的作业会在继续时再次报告。在反向代理之后，请确保它为 `/api/events` 传递 WebSocket 升级，并且不会在 20 秒内关闭空闲连接（服务器每 20 秒发送一次 ping）。如果页面说服务器没有这个地址（`api.not_found`），说明它比服务器旧：请重新加载它。

## MQTT 无法连接 {#mqtt}

Signal Lab 使用纯 TCP 上的 MQTT 3.1.1。它会连接并等待 broker 的应答（CONNACK）才报告成功，因此原因就在连接的按钮上：

| 代码 | 原因 | 修复 |
| --- | --- | --- |
| `transport.refused` | 该端口上没有程序监听 | 检查端口：通常是 1883 |
| `transport.timeout` | 6 秒内没有 TCP 连接 | 检查地址、网络、broker 的防火墙 |
| `transport.dns` | 主机名无法解析 | 检查名称，或改用地址 |
| `transport.unreachable` | 没有到 broker 的路由 | 检查网络和地址 |
| `transport.reset` | broker 立即关闭了连接 | 通常是 TLS 端口（8883）——Signal Lab 不使用 TLS 上的 MQTT |
| `mqtt.no_answer` | 端口是开的，但 6 秒内没有 CONNACK | 通常是 WebSocket 端口——Signal Lab 不使用 WebSocket 上的 MQTT |
| `mqtt.protocol` | 应答的不是 MQTT broker | 检查端口 |
| `mqtt.refused_protocol` | broker 不接受 MQTT 3.1.1 | 在 broker 上启用 3.1.1 |
| `mqtt.refused_client_id` | broker 拒绝了客户端 id | 换一个客户端 id |
| `mqtt.refused_unavailable` | broker 不可用 | 稍后再试 |
| `mqtt.refused_credentials` | 用户名或密码错误 | 检查它们 |
| `mqtt.refused_not_authorized` | 该用户不得连接 | 检查 broker 的访问规则 |
| `mqtt.client_id_required` | 客户端 id 为空 | 填写它 |

对于同一个客户端 id 的两个连接，broker 会断开较旧的那个：当某个连接不断关闭时，请查找使用同一 id 的另一个客户端。

## 证书不受信任 {#tls}

`transport.tls`，出现在 `https://` 或 `wss://` 上：服务器的证书不受信任、不包含您所请求的主机名，或者无法就 TLS 达成一致。Signal Lab 像系统一样检查证书，且没有跳过检查的开关。HTTPS 和 WSS 信任相同的证书：引擎所运行操作系统的证书——Windows 上的 Windows 证书存储，Linux 和服务器镜像上的系统 CA 证书。对于自签名证书或您自己的 CA，请把它加入那台机器的受信任证书（对于服务器镜像，则是基于它构建并加入该证书的镜像），并通过证书所带的名称连接。

## 缺少机密 {#secrets}

`secret.missing`：实验使用了 `{{secret.NAME}}`，但在它运行的地方没有以该名称存储的值。

- **Windows 上的桌面应用**：在编辑器的 [[ui:exp.params]] 下的 [[ui:exp.secrets]] 中设置它。它保存在 Windows 凭据管理器中，因此新机器需要重新设置。
- **服务器**：把值放在文件 `/run/secrets/signallab/NAME` 中（或 `--secrets-dir` 指定的文件夹），或放在环境变量 `SIGNALLAB_SECRET_NAME` 中。服务器无法从页面设置机密（`secret.read_only`）。
- **Linux 上的桌面应用**没有机密的存储（`secret.unsupported`）。请用 `signallab run` 运行这样的实验，它会从文件和变量读取机密，或者在服务器上运行。

参见[文件](files.md#secrets)。

## 运行在五分钟后停止 {#run-timeout}

耗时超过 300 秒的运行会以 `run.timeout` 失败；这是运行可以持续的最长时间。可以通过 API（[`/api/run`](../api/run.md#request) 的 `timeout`）或命令行（`signallab run --timeout`）为运行指定更短的时限。

## 应用不更新 {#updates}

- 桌面应用在 [[ui:update.auto]] 开启时每天查找一次新版本，在您于 [[ui:about.open]] 中按 [[ui:update.check]] 时也会查找。
- 它先询问工作室的 Hub，无法访问 Hub 时再询问 GitHub。当网络同时阻断两者时，[[ui:update.check]] 会给出 [[ui:update.checkFailed]]；每日查找则悄无声息地失败。
- 它只会收到已发布的版本，绝不会是草稿或预发布版本。
- 新版本在 Hub 提供它时才会到达应用，这可能是在它出现在 GitHub 上一段时间之后：Hub 会把一个版本一次推送给一部分安装。
- 只有在您按 [[ui:update.install]] 时它才会安装——正在运行的作业会先被停止——而且只会安装签名验证通过的版本：否则给出 [[ui:update.installFailed]]。
- 服务器随其镜像更新：在其 compose 文件所在文件夹中执行 `docker compose pull && docker compose up -d`。

## 日志在哪里 {#logs}

- **桌面应用**：它不写日志文件。底部面板中的 [[ui:console.title]] 标签页会列出每个工具做了什么、哪里出了错，[[ui:feedback.open]] 会把它附在发给开发者的消息中（不含这台计算机的名称、地址和您的文件夹）。每次运行的报告在数据文件夹的 `runs/` 中。
- **服务器**：它记录到自己的标准输出和错误——在 Docker 中是 `docker logs signallab`。每次作业启动都会连同发出请求的客户端地址一起记录。`--log`（或 `SIGNALLAB_LOG`）设置级别：`error`、`warn`、`info`（默认）或 `debug`；`--log-format json`（或 `SIGNALLAB_LOG_FORMAT`）每行写一个 JSON 对象。
- **命令行**：`signallab` 把消息写到自己的错误输出；参见[命令行](../automation/cli.md)。

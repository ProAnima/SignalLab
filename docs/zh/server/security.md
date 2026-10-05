---
title: 服务器安全
description: 谁可以使用 Signal Lab 服务器，以及它如何把其他人挡在门外——令牌、会话、主机与来源检查、机密——还有 Signal Lab 向外界发送了什么。
---

# 服务器安全

Signal Lab 服务器会从其所在的机器发送真实的流量：OSC、UDP、HTTP、MQTT、风暴、扫描、广播。谁能使用它，谁就能从那台机器做所有这些事，因此服务器默认是关闭的，只有凭令牌才会开放。

::: warning
请像对待这台机器网络的密码一样对待访问令牌。任何持有它的人，都可以从服务器向服务器能到达的任何地方发送流量。
:::

## 概览 {#summary}

- **没有令牌，只限本机**。没有令牌时，服务器只监听环回地址，并且只应答环回主机名。在任何其他地址上，它都拒绝启动。
- **其他所有人都需要令牌**。浏览器登录一次后会得到会话 Cookie；脚本在每个请求中都带上令牌。
- **只接受自己的页面**。会更改内容的请求和事件 WebSocket 必须来自服务器自己的来源；命令只接受 JSON。
- **只接受自己的名称**。服务器不应答的主机名会被拒绝，这可以阻止 DNS 重绑定。
- **机密留在内部**。只读，来自环境变量或文件，从不返回，在任何可能显示的地方都会被遮蔽。
- **不做分外之事**。它从不更改主机的防火墙，不提供数据文件夹之外的任何文件，也不需要任何特权。

## 没有令牌：只限本机 {#loopback}

不带令牌启动时，服务器监听 `127.0.0.1:1430`，无需登录：它是供这台机器前的人使用的工具。为了防止这台机器上任何浏览器中的网页通过解析到 `127.0.0.1` 的名称访问它（DNS 重绑定），它只应答 `Host` 为环回名称的请求——`localhost`、以 `.localhost` 结尾的名称、`127.x.x.x` 或 `[::1]`——或者您用 `--allowed-host` 允许的名称。

如果要求它在任何其他地址上监听而没有令牌，它不会启动：它会说明原因，并以代码 `2` 退出。

## 令牌 {#token}

令牌至少有 24 个字符，不含空格或换行符。`signal-lab-server token` 会输出一个由 64 个十六进制字符组成的随机令牌；`--generate-token`（镜像中默认开启）会在首次启动时生成一个，保存在数据文件夹中，仅服务器自己的用户可读，并只输出一次。提供令牌的各种方式见[访问令牌](index.md#token)。

比较令牌所用的时间，无论差异出现在哪里都相同；错误的令牌会让人等待一秒，并在日志中留下一条警告——猜测既慢又会留下痕迹。

## 浏览器：会话 {#sessions}

尚未登录的浏览器会被带到登录页面。它的令牌被换成一次会话，保存在一个 Cookie 中，该 Cookie：

- 是 `HttpOnly`——页面中的任何脚本都读不到它；
- 是 `SameSite=Strict`——其他站点的页面无法让浏览器发送它；
- 有效期为 7 天；
- 带 `--secure-cookie` 时为 `Secure`，因此只通过 HTTPS 传输（在 HTTPS 代理之后请设置它）。

会话保存在服务器的内存中：重启会让所有人退出登录，[[ui:app.signOut]] 则会立即结束一个会话。最多保留 1024 个会话；最旧的先被清除。

## 脚本：Bearer 令牌 {#bearer}

脚本、`signallab --server` 和 CI 在每个请求中都带上令牌：

```http
Authorization: Bearer <token>
```

除 `GET /api/health`（服务器是否应答、版本、是否要求令牌）和登录页面外，每个端点都需要令牌或会话。没有令牌或会话的 API 请求会得到 `401` 和错误 `auth.required`；页面请求则得到登录页面。

## 主机名 {#hosts}

| 服务器的启动方式 | 它应答的主机名 |
| --- | --- |
| 没有令牌 | 环回名称，以及 `--allowed-host` 中的名称 |
| 有令牌，没有 `--allowed-host` | 任何名称 |
| 有令牌，也有 `--allowed-host` | 环回名称，以及 `--allowed-host` 中的名称 |

`--allowed-host`（`SIGNALLAB_ALLOWED_HOSTS`）接受以逗号分隔的名称，比较时不含端口，也忽略大小写：

```bash
signal-lab-server --listen 0.0.0.0:1430 --token-file token.txt --allowed-host lab-pc.example.com,192.0.2.10
```

任何其他 `Host` 都会得到 `403` 和错误 `auth.host`。如果服务器可以通过已知的名称访问，请设置它，这样其他站点上的页面就无法通过自己的名称访问它。

## 来源与内容类型 {#origin}

- 每个会更改内容的请求（除 `GET` 和 `HEAD` 之外的任何请求）以及事件 WebSocket，都必须不带 `Origin`，或者带服务器自己的来源——与其 `Host` 相同的主机和端口。来自其他站点的页面，或发送 `Origin: null` 的页面，会得到 `403` 和 `auth.origin`。脚本和 `curl` 不发送 `Origin`，不受影响。
- 命令只接受 `Content-Type: application/json`（否则返回 `415` 和 `command.json_required`），因此其他站点上的表单无法发送命令。
- 服务器不应答任何跨源（CORS）请求。

## 响应标头 {#headers}

每个响应都带有：

| 标头 | 值 |
| --- | --- |
| `Content-Security-Policy` | `default-src 'self'; connect-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; object-src 'none'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'` |
| `X-Content-Type-Options` | `nosniff` |
| `X-Frame-Options` | `DENY` |
| `Referrer-Policy` | `same-origin` |
| `Cache-Control` | API 和登录页面为 `no-store` |

界面只加载服务器提供的内容，只与服务器通信，也无法被其他页面嵌入框架。

## 文件与大小 {#files}

- 下载（`GET /api/files?path=…`）只从数据文件夹内部提供，最大 256 MiB；其他一律返回 `404`。
- 请求正文最大 24 MiB。

## 机密 {#secrets}

机密的值从不离开引擎：

- 在服务器上，值是**只读**的：来自环境变量 `SIGNALLAB_SECRET_<NAME>`，或机密文件夹（默认为 `/run/secrets/signallab`）中的文件 `<NAME>`。从浏览器设置或移除机密会被拒绝（`secret.read_only`），因此在页面中输入的值绝不会最终存放在更不安全的地方。参见[机密](index.md#secrets)。
- 没有任何命令会返回值；界面只能得知某个名称是否已设置。
- 实验以 `{{secret.NAME}}` 引用机密。在运行或发送使用它们期间，它报告的每段文本（步骤、错误、运行报告）都在其位置显示 `••••`，[[ui:dock.inspector]] 中的帧则被逐字节遮蔽。
- HTTP 节点的凭据只在请求发出时才变成 `Authorization` 标头；步骤、帧和报告中带的是响应，从不带该标头。

## 记录了什么 {#audit}

日志会连同客户端的地址一起记录：启动的每个任务（风暴、扫描、广播、监视器、生成器、运行、模拟器）、每次登录，以及每次使用错误令牌的登录。参见[日志](index.md#logs)。

## 服务器从不做的事 {#never}

- **更改主机的防火墙**。桌面应用在您要求时可以添加防火墙规则；在服务器上，该命令会被拒绝（`firewall.server`）。主机的防火墙属于运行主机的人。（一键安装脚本会提出在 ufw 或 firewalld 中开放服务器的端口，并且会先询问——参见[主机的防火墙](index.md#firewall)。）
- **在没有令牌时，于环回以外的地址上启动。**
- **提供数据文件夹之外的文件**。
- **保存在浏览器中输入的机密。**
- **自己提供 TLS**：请在它前面放置 HTTPS 代理（参见[在 HTTPS 代理之后](index.md#https)）。

引擎的每个限制（一次遍历最多 1024 台主机，一个信标每秒最多 50,000 个数据包）在服务器上与在应用中一样有效。它们是防护上限，而不是许可：只向您拥有或获准测试的系统发送流量。

## 容器 {#container}

镜像以非特权用户（uid 和 gid 均为 10001）运行，只写入 `/data`。它可以原样在只读根文件系统、无任何能力和 `no-new-privileges` 的条件下运行——安装脚本和 `deploy/compose.yaml` 就是这样启动它的。每个镜像发布时都附带 SBOM、构建来源证明和经过签名的 GitHub 证明：

```bash
gh attestation verify oci://ghcr.io/proanima/signallab:[[version]] -R ProAnima/SignalLab
```

## Signal Lab 向外界发送什么 {#outside}

除了您发送的流量，Signal Lab 只与两个地方通信，都属于工作室。

### 更新检查 {#update-check}

只有**桌面应用**会查找更新；服务器和浏览器从不查找。在 [[ui:update.auto]] 开启时（位于 [[ui:about.open]] 中），每天一次，以及每当您按 [[ui:update.check]] 时，应用会询问工作室的 Hub（`hub.proanima.net`）——只有在无法连接 Hub 时才询问 GitHub 上的最新发布版本。询问中包含：

- 应用的版本；
- 操作系统和处理器架构；
- 此安装的一个随机数（`X-Install-Id`），生成一次，并与应用的设置一起保存，这样新版本可以先推送给一部分安装。它不包含任何关于您或这台电脑的信息。

只提供已发布的版本。签名与应用内置密钥不匹配的下载不会被安装，在您按下 [[ui:update.install]] 之前也不会安装任何内容。

### 反馈 {#feedback}

[[ui:feedback.open]]（顶栏中的 ✉，在 [[ui:about.open]] 中也有）会通过工作室的 Hub 向开发者发送一条消息，由 Hub 转发为邮件；应用中不保存任何相关密码。它只发送表单所显示的内容：您的留言，如果您提供的话还有您的电子邮箱，您添加的截图，以及在 [[ui:feedback.logs]] 下的控制台日志和 [[ui:feedback.systemInfo]]，这两项您在发送前都可以打开查看并取消勾选。这台电脑的名称、地址和您的文件夹都不会包含在其中。从浏览器发送时，由服务器发送该表单。

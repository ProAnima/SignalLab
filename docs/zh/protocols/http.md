---
title: HTTP
description: 发送单个 HTTP 请求并读取完整响应，使用 Basic、Bearer 或 Digest 认证，保留 Cookie，并在并发突发负载下测量一个端点。
---

# HTTP

[[ui:nav.http]] 界面集请求检查器与负载工具于一身：

- 发送单个请求并查看状态、耗时、标头和正文；
- 使用 Basic、Bearer 令牌或 Digest 认证；
- 像浏览器一样保留服务器设置的 Cookie；
- 同时多次发送同一个请求——一次 [[ui:http.burst]]——并读取吞吐量和延迟百分位数。

## 发送请求 {#request}

1. 打开 [[ui:nav.http]]。
2. 选择方法并输入 URL，例如 `http://127.0.0.1:8080/health`。
3. 如果服务器需要，添加 [[ui:http.headers]]；[[ui:http.addHeader]] 添加一行，✕ 移除一行。没有名称的行不会被发送。
4. 对于 GET 和 HEAD 以外的方法，填写 [[ui:http.body]]。切换到 GET 或 HEAD 时文本会留在字段中，切换回其他方法时又会出现，但在此期间不会被发送。
5. 按 [[ui:common.send]]。

按钮下方的一行会立即给出判定——状态、时间和大小，或为什么没有应答——其余内容由 [[ui:http.response]] 面板显示。切换界面和重启应用时，请求（方法、URL、标头、正文、超时和 [[ui:http.keepCookies]]）都会保留；凭据不会。

| 按键 | 位置 | 作用 |
| --- | --- | --- |
| <kbd>Enter</kbd> | URL、标头、凭据、超时 | 发送 |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | 请求的任意字段，包括正文 | 发送 |
| <kbd>Ctrl</kbd>+<kbd>S</kbd> | 请求的任意字段 | 将其保存为信号（[见下文](#library)） |

### 请求字段 {#request-fields}

| 字段 | 内容 | 默认值 |
| --- | --- | --- |
| [[ui:exp.method]] | GET、POST、PUT、PATCH、DELETE、HEAD 或 OPTIONS | GET |
| [[ui:field.url]] | 一个 `http://` 或 `https://` URL | `http://127.0.0.1:8080/` |
| [[ui:http.headers]] | 名称和值对，按书写的内容发送。没有您自己的 `User-Agent` 时，Signal Lab 发送 `SignalLab/0.1`。 | `Accept: application/json` |
| [[ui:field.auth]] | 请求如何进行认证（[见下文](#auth)） | [[ui:http.auth.none]] |
| [[ui:http.keepCookies]] | 回送服务器设置的 Cookie（[见下文](#cookies)） | 开 |
| [[ui:http.body]] | 按书写的内容原样发送；不添加 `Content-Type`，因此请添加与之匹配的标头。空正文不会被发送。GET 和 HEAD 不显示该字段，此时请求完全不携带正文——发送时不带、保存的信号中不带、[[ui:common.toExperiment]] 中也不带——即使您曾在另一种方法下输入过正文。 | 空 |
| [[ui:common.timeoutMs]] | 整个交换可以花多长时间，包括应答和正文 | 10 000 |

## 认证 {#auth}

| [[ui:field.auth]] | 字段 | 发送的内容 |
| --- | --- | --- |
| [[ui:http.auth.none]] | — | 没有 `Authorization` 标头 |
| [[ui:http.auth.basic]] | [[ui:field.username]]、[[ui:field.password]] | `Authorization: Basic …`，名称和密码以 base64 编码，随第一个请求发送 |
| [[ui:http.auth.bearer]] | [[ui:field.token]] | `Authorization: Bearer <token>` |
| [[ui:http.auth.digest]] | [[ui:field.username]]、[[ui:field.password]] | 起初什么都不发；对服务器质询的应答（[见下文](#digest)） |

在 Basic 和 Digest 之间切换会保留名称和密码。

### Digest {#digest}

使用 Digest 时，Signal Lab 先不带凭据发送请求。当服务器以 Digest 质询应答 `401` 时，Signal Lab 根据质询和您的密码算出应答，并再次发送请求。您看到的响应是针对第二个请求的那个，标记为 [[ui:http.digestAnswered]]，延迟把两次交换都计入——这正是客户端等待的时间。

- 算法：MD5 和 SHA-256，以及它们的 `-sess` 变体。当服务器同时提供两者时，使用 SHA-256。
- 保护质量：`auth` 和 `auth-int`，以及不带 `qop` 的旧式应答。
- 当服务器说 nonce 过期（`stale`），或用新 nonce 再次询问时，会再次作答并重发请求，最多再答 3 次。当它拒绝对其最后给出的 nonce 的应答时，`401` 就成立：名称或密码错了。
- 重定向由 Signal Lab 自己跟随，因此发出请求的 URL 就是被应答的那个。来自另一来源的质询不会被应答：为一个主机输入的凭据不会发给任何其他主机。在同一主机和默认端口上从 `http://` 移到 `https://` 算作同一主机。

当质询无法作答时，`401` 成立，面板会说明原因：

| 消息 | 含义 |
| --- | --- |
| 服务器应答了 401，但没有要求 Digest | 服务器想要另一种方案；请试试 Basic 或 Bearer。 |
| 服务器要求用……进行 Digest | 一种 Signal Lab 不会说的算法；它会 MD5 和 SHA-256。 |
| 服务器要求 Digest，但没有 realm 或 nonce | 服务器的质询不完整。 |
| 请求被转到了……，而它要求 Digest | 重定向指向了另一来源，其质询不会被应答。 |

### 凭据去往哪里 {#credentials}

凭据只在请求发出时进入其 `Authorization` 标头。检查器、控制台和实验报告从不显示该标头。在此界面上它们只保存在内存中，重启后即消失——除非请求与一个保存的信号关联，那样它们就会回来。

::: warning
保存为信号的请求会将其凭据以明文保存在库文件 `signals.json` 中。在实验中，请改为把密码写成 `{{secret.NAME}}`；见[数据与模板](../experiments/data.md)。
:::

## Cookie {#cookies}

开启 [[ui:http.keepCookies]] 后，服务器用 `Set-Cookie` 设置的内容会保存在该界面的 Cookie 存储中，并在之后对该服务器的请求中回送，遵循浏览器的规则（域、路径、`Secure`、过期）。该界面的请求、突发负载以及您从库中发送的 HTTP 信号都使用这个存储。关闭它则发送不带 Cookie 的请求，也不保留任何 Cookie。

请求和响应下方的 Cookie 面板列出存储中保存的内容：[[ui:http.cookieName]]、[[ui:http.cookieValue]]、[[ui:http.cookieWhere]]（以 `.` 开头的域也涵盖其子域）、[[ui:http.cookieExpires]]（没有过期时间的 Cookie 显示为 [[ui:http.cookieSession]]）和 [[ui:http.cookieFlags]]（`Secure`、`HttpOnly`、`SameSite`）。已过期的 Cookie 不会列出。[[ui:common.clear]] 清空该存储。

该存储与应用同生命周期：重启后从空开始。在[服务器](../server/index.md)上，每个登录的页面各有一个存储。一次实验运行有自己的存储（见[实验](../experiments/index.md)），而 `signallab send http` 不使用任何存储。

## 响应 {#response}

| 部分 | 内容 |
| --- | --- |
| [[ui:http.status]] | 状态码及其原因；没有应答时为 `ERR` |
| [[ui:http.latency]] | 从发送到正文最后一个字节的时间，以毫秒计 |
| [[ui:http.size]] | 正文的大小 |
| 响应标头 | 点击带有其数量的那一行可显示或隐藏它们 |
| 正文 | 是 JSON 时会格式化；[[ui:http.rawBody]] 和 [[ui:http.formatJson]] 可切换。最多显示 256 KiB，之后为 `… (truncated)`。 |

没有响应时，面板会用与 Signal Lab 各处相同的措辞说明原因：被拒绝、未及时应答、名称无法解析、证书问题等。来自系统的技术细节折叠在其下方。

### 重定向 {#redirects}

重定向（301、302、303、307、308）会被跟随，最多 10 次；显示的响应是最后一个。经过 301、302 和 303 之后，请求以不带正文的 GET 继续（HEAD 仍是 HEAD）；经过 307 和 308 之后则保持原样。为一个主机输入的 `Authorization` 和 Cookie 不会发往另一个主机。

### 安全连接 {#tls}

`https://` 服务器的证书会对照本系统信任的证书进行检查。自签名或已过期的证书会被拒绝，并提示“无法与……建立安全连接”；没有跳过检查的设置。要用自己的证书测试服务器，请把该证书添加到系统的受信任证书中。

## 突发负载 {#burst}

[[ui:http.burst]] 会把界面上的请求——连同其认证，以及在 [[ui:http.keepCookies]] 开启时的 Cookie 存储——发送许多次，并进行测量。

1. 设置 [[ui:common.concurrency]]、[[ui:http.total]]、[[ui:http.duration]] 和 [[ui:http.rate]]。
2. 按 [[ui:http.startBurst]]。突发负载是一个任务：[[ui:http.stopBurst]]，或在控制台任务条中停止它，都会结束它。

| 字段 | 内容 | 默认值 |
| --- | --- | --- |
| [[ui:common.concurrency]] | 同时在途的请求数，1–512 | 20 |
| [[ui:http.total]] | 要发送的请求数；0——一直发送到时长结束 | 500 |
| [[ui:http.duration]] | 运行的秒数；0——发送完总数后停止 | 0 |
| [[ui:http.rate]] | 每秒启动的请求数，0.1–100 000；0——能多快就多快 | 0 |

当 [[ui:http.total]] 和 [[ui:http.duration]] 都为 0 时，突发负载会一直运行，直到您停止它。

有两种发送方式：

- **[[ui:http.rate]] 为 0。**每个工作线程一得到应答就再次发送。这样能测出服务器能承受多少，但服务器慢也会拖慢突发负载。
- **设定速率。**请求按固定时间表启动——每秒 10 个时，从开始起每 100 ms 一个——无论应答多慢。某个请求到点时所有工作线程都忙，它会最多等待 50 ms 以求一个空闲；超过之后就被跳过并计为 [[ui:http.missed]]，绝不推迟发送。错过的请求意味着并发数对该速率太低，或者服务器比该速率所需的更慢。

| 数字 | 内容 |
| --- | --- |
| [[ui:http.sent]] | 已得到应答或失败的请求 |
| [[ui:http.ok]] | 以 2xx 状态应答的 |
| [[ui:http.failed]] | 没有应答，或任何不在 200–299 范围内的状态 |
| [[ui:http.missed]] | 被跳过，如上所述（仅在设定速率时） |
| [[ui:http.rps]] | 最近十分之一秒内的每秒请求数；突发负载结束后，则为整个突发期间的每秒请求数。设定速率时，标签会标出所要求的速率。 |
| [[ui:http.p50]]、[[ui:http.p90]]、[[ui:http.p95]]、[[ui:http.p99]] | 该比例的请求在此时间内完成，失败也计入；误差在 0.5% 以内 |
| [[ui:http.avg]]、[[ui:http.min]]、[[ui:http.max]] | 平均、最快和最慢 |

这些数字每秒大约更新 10 次。旁边的图表绘出最近约 24 秒内的每秒请求数。

使用 Digest 时，第一个请求的质询只作答一次，该应答即可用于突发负载的每个请求。

::: warning
突发负载是真实的负载。只把它指向您拥有或获准测试的服务器。
:::

对于斜坡、阶梯、尖峰以及通过/失败阈值，请在[实验中的负载](../experiments/load.md)下运行该请求。

## 在检查器中 {#inspector}

开启捕获后，每次交换都会以协议 `http`、来源 `http` 显示为一帧：摘要是方法、URL、状态和时间，详情是响应标头和正文的开头（2000 个字符），判定是状态（没有应答时为 `failed`，质询被作答时为 `· digest after 401`）。该帧记录正文的大小，而不是其字节。请求的 `Authorization` 标头从不在其中。突发负载最多每 100 ms 把一个交换放入捕获。见[检查器](../tools/inspector.md)。

## 保存与复用 {#library}

- **保存为信号。**[[ui:sig.saveNew]] 会把请求——方法、URL、标头、正文、超时和认证——保存到信号库中。界面会与它保持关联：[[ui:sig.save]]（<kbd>Ctrl</kbd>+<kbd>S</kbd>）更新它，[[ui:sig.saveAs]] 复制它，标签会在 [[ui:nav.signals]] 中打开它。从库中打开一个 HTTP 信号会把它连同凭据一起加载回这里。见[信号](../tools/signals.md)。
- **添加到实验。**[[ui:common.toExperiment]] 会把一个请求相同的 [HTTP 请求](../experiments/nodes.md#node-http)步骤添加到打开的实验，紧挨着 End 之前或所选步骤之后，并打开它。
- **模拟此请求。**在响应下方，[[ui:http.mockThis]] 会创建一个模拟器路由，按此方法、路径、状态、标头和正文应答。在 [[ui:http.mockInto]] 中选择一个 HTTP 模拟器，或选择 [[ui:http.mockNew]]，然后按 [[ui:http.mockAdd]]；该路由会放在该模拟器的最前面，[[ui:nav.emulators]] 界面会打开并定位到它。见[模拟器](../tools/emulators.md)。

## 在实验中 {#experiments}

| 步骤 | 作用 |
| --- | --- |
| [[ui:exp.node.http]] | 发送一个请求；其 URL、标头、正文和凭据接受 `{{templates}}`。它可以[在负载下](../experiments/load.md)运行。[详情](../experiments/nodes.md#node-http) |
| [[ui:exp.node.assert_status]]、[[ui:exp.node.assert_body]]、[[ui:exp.node.assert_header]]、[[ui:exp.node.assert_latency]] | 检查最新的响应。[详情](../experiments/nodes.md#node-assert_status) |
| [[ui:exp.node.extract]] | 把 JSON 字段、标头、状态、正文或正则表达式的匹配保存为变量。[详情](../experiments/nodes.md#node-extract) |
| [[ui:exp.node.branch_status]] | 按状态通过“是”或“否”继续。[详情](../experiments/nodes.md#node-branch_status) |
| [[ui:exp.node.wait_http]] | 等待一个请求到达——来自您测试的系统——在本次运行自己的监听器或模拟器上。[详情](../experiments/nodes.md#node-wait_http) |
| [[ui:exp.node.emulator]] | 一个在整个运行期间按路由应答的 HTTP API。[详情](../experiments/nodes.md#node-emulator) |

## 命令行 {#cli}

`signallab send http` 像该界面一样发送一个请求：

```bash
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab send http POST http://127.0.0.1:8080/api/items \
  -H 'Content-Type: application/json' --body '{"name":"lamp"}'
signallab send http GET http://127.0.0.1:8080/private -u admin:secret --digest
```

状态行写到标准错误，正文写到标准输出：

```text
HTTP 200 OK · 3 ms · 15 B
{"status":"ok"}
```

| 选项 | 内容 | 默认值 |
| --- | --- | --- |
| `-H`、`--header 'Name: value'` | 一个标头；可重复添加更多 | — |
| `--body TEXT`、`--body @FILE` | 正文，或文件的内容 | — |
| `--expect-status N` | 除非状态为 N，否则以 1 退出 | — |
| `--timeout MS` | 等待应答多长时间 | 10 000 |
| `-u`、`--user NAME:PASSWORD` | Basic 认证 | — |
| `--digest` | 与 `--user` 一起：改为应答服务器的 Digest 质询 | — |
| `--bearer TOKEN` | `Authorization: Bearer TOKEN` | — |
| `--json` | 把完整响应以 JSON 打印到标准输出 | — |

有响应到达时（且状态符合预期）以 0 退出，没有响应、状态不符合预期或 Digest 质询无法作答时以 1 退出，选项无效时以 2 退出。它不保留任何 Cookie。见[命令行](../automation/cli.md#cli-send-http)。

## 问题 {#troubleshooting}

| 您看到的情况 | 通常的原因 |
| --- | --- |
| `… refused the connection — nothing is listening on that port` | 服务器没有运行，或监听在另一个端口或地址上。 |
| `No answer from … in time` | 服务器很慢或不可达；请检查地址，或调高 [[ui:common.timeoutMs]]。 |
| `Cannot resolve …` | 该主机名在本机上无法解析——可能是拼写错误，或是只有其他网络才知道的名称。 |
| `A secure connection to … could not be made` | 此处的证书不受信任（自签名、已过期、名称不符），或 TLS 失败。见[安全连接](#tls)。 |
| `… is not a valid address` | URL 格式错误，或不是以 `http://` 或 `https://` 开头。 |
| 服务器说正文缺失或类型不对 | 没有与正文匹配的 `Content-Type` 标头，或正文为空。 |
| 使用 Digest 时出现 `401` | 请阅读状态下方的消息：见 [Digest](#digest)。 |
| [[ui:http.missed]] 大于 0 | 调高 [[ui:common.concurrency]]，或降低速率：服务器的应答比该速率所需的更慢。 |
| 服务器有应答，但 [[ui:http.failed]] 很高 | 任何不在 200–299 范围内的状态都算失败，404 和 500 也包括在内。 |

在服务器上，请求从服务器发出：`127.0.0.1` 就是服务器本身。见[服务器](../server/index.md)。

每条错误消息都列在[错误消息](../reference/errors.md#transport)中。

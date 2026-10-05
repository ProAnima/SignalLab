---
title: WebSocket
description: 带上服务所需的标头和子协议连接到 ws:// 或 wss:// 服务，发送文本或二进制消息，并读取到达的每条消息。
---

# WebSocket

[[ui:nav.ws]] 界面是一个 WebSocket 客户端：它向一个服务打开一个连接——带上服务所需的标头和子协议——发送文本或字节，并列出往来于其中的每条消息，最新的在最后。用它来在实验里编写脚本之前，试一下实时 API、控制面板或支持 WebSocket 的设备。

## 连接 {#connect}

1. 打开 [[ui:nav.ws]]。
2. 在 [[ui:field.url]] 中，输入地址：`ws://127.0.0.1:9001/` 或 `wss://example.com/socket`。
3. 如果服务需要，输入 [[ui:exp.wsProtocols]]，并添加 [[ui:http.headers]]（带令牌的 `Authorization` 标头、一个 Cookie）。
4. 按 [[ui:ws.connect]]。升级进行期间按钮显示 [[ui:ws.connecting]]；升级完成后字段会锁定，按钮变为 [[ui:ws.disconnect]]。

| 字段 | 内容 | 默认值 |
| --- | --- | --- |
| [[ui:field.url]] | `ws://` 或 `wss://`、主机、可选端口（`ws` 为 80，`wss` 为 443）以及路径 | `ws://127.0.0.1:9001/` |
| [[ui:exp.wsProtocols]] | 要提供的子协议，以逗号分隔，按优先顺序排列；服务器选择一个。名称中不能有空格、逗号或斜杠。 | 无 |
| [[ui:http.headers]] | 升级请求的额外标头；[[ui:http.addHeader]] 添加一行。没有名称的行会被略过。 | 无 |

连接——名称解析、TCP 连接、`wss://` 的 TLS 以及升级——有 10 秒时间。切换界面和重启应用时 URL 会保留；标头和子协议不会。

连接后，面板显示：

| 项目 | 内容 |
| --- | --- |
| [[ui:ws.state]] | [[ui:ws.open]]，或结束时：由您或服务器关闭，带关闭码；或在线路中断而没有关闭帧时为 [[ui:ws.lost]] |
| [[ui:field.reason]] | 关闭方给出的原因（如果有） |
| [[ui:ws.subprotocol]] | 服务器选择的子协议，或 — |
| [[ui:ws.peer]] | 服务器的 `IP:port` |
| [[ui:ws.upgradeTime]] | 连接和升级花的时间，以毫秒计 |

该连接是一个任务：它会出现在控制台任务条中，也可以在那里停止。

### 安全连接 {#wss}

`wss://` 信任与本系统上 HTTPS 相同的证书：本系统不信任其证书的服务器（自签名、已过期、名称不符）会被拒绝，并提示“无法与……建立安全连接”。没有跳过检查的设置。

## 发送一条消息 {#send}

1. 在 [[ui:ws.message]] 下，选择 [[ui:exp.wsText]] 或 [[ui:exp.wsBinary]]。
2. 写下消息。对于二进制，把字节写成成对的十六进制数字：`de ad be ef`。
3. 按 [[ui:common.send]]，或在消息中按 <kbd>Ctrl</kbd>+<kbd>Enter</kbd>。

一条消息最多 16 MiB（16 777 216 字节，二进制按字节计，而不是按十六进制数字计），与到达的消息以及 [[ui:exp.node.ws_send]] 步骤的限制相同。更长的一条会在发送任何内容之前被拒绝，并提示 `Too long: at most 16777216`，连接保持打开。

文本消息是 JSON 时，[[ui:http.formatJson]] 会在发送前用缩进把它排好。切换界面和重启应用时，消息会保留。

## 读取消息 {#messages}

[[ui:ws.messages]] 列出收到的（↓）和发送的（↑），最新的在最后，并带有时间、消息的开头（300 个字符）及其大小；二进制消息以十六进制显示其字节，并标记为 [[ui:ws.binary]]。列表上方是收到和发送的数量。列表在滚动到底部时会跟随新消息；向上滚动后它会停在您所在的位置。

点击一条消息可在列表下方查看它的完整内容：JSON 会排好，二进制以十六进制显示。[[ui:ws.editHere]] 会把它复制到消息字段，以便再次发送或修改。非常长的消息只显示一部分——文本最多 64 KiB，二进制最多 4096 字节——此时不能复制，因为复制出来的会被截断。

该界面保留最新的 2000 条消息；[[ui:common.clear]] 清空列表。如果服务发送的速度超过界面能承受的——十分之一秒内超过 2000 条——其中最早的会被排除在列表之外，并计为“未显示”。捕获开启期间，检查器仍然拥有它们。

## 关闭 {#close}

[[ui:ws.disconnect]] 发送一个关闭码为 1000（正常）的关闭帧，并最多等待 2 秒以获取服务器的应答，然后挂断。此后状态显示为以 1000 关闭。服务器关闭时，状态显示其代码和原因；连接在没有关闭帧的情况下中断时，状态显示 [[ui:ws.lost]]，控制台会说明原因。

Signal Lab 自行应答服务器的 ping；ping 和 pong 不会列出。当收到超过 16 MiB 的消息，或由于服务器停止读取而导致发送一条消息耗时超过 10 秒时，连接也会结束。（您自己发送超过 16 MiB 的消息会被拒绝，不会结束任何东西。）

## 在检查器中 {#inspector}

开启捕获后，该连接的流量会以协议 `ws`、来源 `websocket` 出现：

| 摘要 | 内容 |
| --- | --- |
| `CONNECT ws://… (subprotocol)` | 连接已打开 |
| `TEXT …` | 一条文本消息及其开头 |
| `BINARY n B …` | 一条二进制消息、其大小和前 16 个字节 |
| `CLOSE code reason` | 一个关闭帧，发出或收到 |

每条消息帧都保留其字节。流量较轻时每条消息都会被捕获；繁忙的连接被限制为每秒 200 帧，之后捕获的帧会说明有多少被略过（`+n not shown`）。见[检查器](../tools/inspector.md)。

## 在实验中 {#experiments}

四个步骤可为一段 WebSocket 对话编写脚本。连接由一个步骤打开，由其他步骤按名称引用：

| 步骤 | 作用 |
| --- | --- |
| [[ui:exp.node.ws_connect]] | 为本次运行的其余部分打开一个连接。其 URL 和标头接受 `{{templates}}`，因此可以放入更早提取出的令牌。[详情](../experiments/nodes.md#node-ws_connect) |
| [[ui:exp.node.ws_send]] | 在一个连接上发送文本或二进制消息。[详情](../experiments/nodes.md#node-ws_send) |
| [[ui:exp.node.wait_ws]] | 等待一条载荷匹配的消息，如同 [[ui:exp.node.wait_udp]] 那样；之后可以逐字段读取 JSON 消息。[详情](../experiments/nodes.md#node-wait_ws) |
| [[ui:exp.node.ws_close]] | 用关闭握手关闭一个连接：代码 1000，或应用自定义的 3000–4999，以及最多 123 字节的原因。[详情](../experiments/nodes.md#node-ws_close) |

运行结束时（或被停止时）仍然打开的连接会被妥善关闭。[[ui:exp.templateWsEcho]] 模板是一个完整的示例。

## 命令行 {#cli}

`signallab send ws` 进行一次交换：连接、发送一条消息、在您要求时等待应答、关闭。

```bash
signallab send ws ws://127.0.0.1:9001/ --text '{"type":"ping"}' --expect pong
```

握手和已发送的内容写到标准错误，应答写到标准输出：

```text
Connected to ws://127.0.0.1:9001/ in 4 ms
Sent 15 bytes
{"type":"pong"}
```

`--hex` 发送一条二进制消息；`-H` 添加一个标头，`--protocol` 提供一个子协议（两者都可重复）。`--expect TEXT`、`--expect-regex RE` 或 `--wait`（任意消息）说明要等待什么应答，最多等待 `--timeout` 毫秒（默认 2000）。应答没有到达或连接失败时以 1 退出。见[命令行](../automation/cli.md#cli-send-ws)。

## 问题 {#troubleshooting}

| 您看到的情况 | 通常的原因 |
| --- | --- |
| `… is not a WebSocket address` | URL 不是以 `ws://` 或 `wss://` 开头，或没有主机。 |
| `… answered HTTP n instead of switching to WebSocket` | 服务器拒绝了升级：路径错误（404）、令牌缺失或错误（401、403）。其应答的开头在技术细节下。 |
| `… did not take any of the subprotocols offered` | 您提供了子协议，而服务器没有选择其中任何一个，或它用了一个您没有提供的子协议来应答。 |
| `… is not a subprotocol name` | 名称中含有空格、逗号或斜杠。 |
| `The header … cannot be sent with the upgrade` | 标头名称或值中含有 HTTP 不允许的字符。 |
| `… refused the connection` | 该端口上没有程序监听。 |
| `A secure connection to … could not be made` | 此处的证书不受信任，或 TLS 失败。见[安全连接](#wss)。 |
| `The connection with … broke: the server did not keep to the WebSocket protocol` | 服务器发送了不是有效 WebSocket 的内容。 |
| `A WebSocket message is limited to … bytes` | 服务器发送了一条超过 16 MiB 的消息，连接随之结束。 |
| `Too long: at most 16777216` | 您试图发送的消息超过 16 MiB。没有发送任何内容；连接仍然打开。 |

每条错误消息都列在[错误消息](../reference/errors.md#ws)中。

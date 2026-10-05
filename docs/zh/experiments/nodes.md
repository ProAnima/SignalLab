---
title: 节点
description: 实验节点的每一种类型——它做什么、它的字段及默认值和限制、它的输出、它接受的设置，以及它在实验文件中的样子。
---

# 节点参考

实验可以包含的每一种节点，按添加菜单中的分组排列：
[操作](#actions)、[等待](#waits)、[模拟](#emulation)、
[故障](#faults)、[数据](#data)、[检查](#checks) 和[流程](#flow)。如何
添加和连接它们见[编辑器](index.md)；`signallab nodes` 会把同一份目录以 JSON
打印出来，供脚本和助手使用（[命令行](../automation/cli.md)）。

## 阅读本页 {#reading}

每个节点都有一张其字段的表格：

- **字段** 是属性窗格中的名称；**在文件中** 是实验 JSON 中的键。
- **默认值** 是在编辑器中添加节点时它得到的值。当文件可以省略某个键时，
  此时它取的值写作*缺省时*；其他键在文件中是必需的。
- **模板**：*是*——该字段接受 `{{templates}}`：参数、早先设置的变量、
  机密和生成器，在步骤运行时解析（[数据与模板](data.md)）。*仅参数*——它
  在第一个步骤之前打开，此时只知道参数。*否*——该值按书写的内容采用。

时间以毫秒计。限制在运行开始之前检查；超出范围的字段会让实验无法运行，
并在节点上显示。

## 文件中的节点 {#file-shape}

在实验文件中，节点是一个对象，包含 `id`（在实验中唯一）、它的 `type`、
它在画布上的位置（`x`、`y`，零或更大）、它的字段，以及它使用的设置
（`retry`、`repeat`、`load`，关闭时省略）。连线是一条边，从一个节点的输出
（`port`，缺省时为 `next`）指向另一个节点：

```json
{
  "nodes": [
    { "id": "start", "type": "start", "x": 40, "y": 80 },
    { "id": "ping", "type": "udp", "x": 270, "y": 80, "target": "127.0.0.1:9000", "text": "PING",
      "retry": { "attempts": 3, "delay_ms": 500, "backoff": "fixed" } },
    { "id": "end", "type": "end", "x": 500, "y": 80 }
  ],
  "edges": [
    { "from": "start", "to": "ping", "port": "next" },
    { "from": "ping", "to": "end", "port": "next" }
  ]
}
```

下面的示例各展示一个节点，如同文件所保存的样子。

## 许多节点共享的设置 {#settings}

这些在节点属性的下半部分开启。哪个节点接受哪一项，列在每个节点之下。

| 设置 | 接受它的节点 | 作用 |
| --- | --- | --- |
| [重试](#retry) | 发送或监听的节点：[[ui:exp.node.http]]、[[ui:exp.node.tcp]]、[[ui:exp.node.osc]]、[[ui:exp.node.udp]]、[[ui:exp.node.mqtt]]、[[ui:exp.node.ws_connect]]、[[ui:exp.node.ws_send]]，以及每个等待 | 步骤失败时再试一次 |
| [重复](#repeat) | 发送的节点：[[ui:exp.node.http]]、[[ui:exp.node.tcp]]、[[ui:exp.node.osc]]、[[ui:exp.node.udp]]、[[ui:exp.node.mqtt]]、[[ui:exp.node.ws_send]] | 一次又一次地发送，按次数或按时长 |
| [负载](#load) | [[ui:exp.node.http]] | 按负载模式发送请求，进行测量并由阈值判定 |
| [等待回复](#reply) | [[ui:exp.node.osc]]、[[ui:exp.node.udp]] | 在同一个步骤中发送并等待应答 |

### 重试 {#retry}

[[ui:exp.retryOn]]：当步骤失败时——无法连接、超时、等待没有任何匹配——
它会暂停并再次运行。每次失败的尝试都是时间线中的一行；最后一次尝试也失败
时，该步骤失败。无法解析的模板不会重试。[[ui:common.stop]] 也会结束暂停。

| 字段 | 在文件中 | 内容 | 默认值和限制 |
| --- | --- | --- | --- |
| [[ui:exp.attempts]] | `retry.attempts` | 总尝试次数，包含第一次 | 3；编辑器中为 2–10（文件也可以写 1） |
| [[ui:exp.retryDelay]] | `retry.delay_ms` | 第二次尝试前的暂停 | 500；0–60,000 |
| [[ui:exp.backoff]] | `retry.backoff` | [[ui:exp.backoff.fixed]]（`fixed`）：每次相同的暂停；[[ui:exp.backoff.exponential]]（`exponential`）：每次失败后翻倍 | `fixed`（缺省时也如此） |

无论怎样翻倍，暂停都不会超过 60 秒。当 [[ui:exp.portTimeout]] 输出有连线的
等待节点遇到超时时不会失败——它会从该输出离开——因此那时不会重试。

### 重复 {#repeat}

[[ui:exp.repeatOn]]：节点一次又一次地发送——心跳、轮询、稳定流——无需图中的
循环。每次发送都会重新读取其模板（`{{counter}}` 是它的序号，`{{now}}` 是它
的时间），开启重试时，重试适用于每次发送。每次发送都成功时该步骤通过；某次
发送彻底失败会使该步骤失败。时间线最多每秒报告一次进度。

| 字段 | 在文件中 | 内容 | 默认值和限制 |
| --- | --- | --- | --- |
| [[ui:exp.repeatBy]] | `repeat.until` | [[ui:exp.repeatBy.count]]（`count`）或 [[ui:exp.repeatBy.duration]]（`duration`） | `count`（缺省时也如此） |
| [[ui:exp.repeatCount]] | `repeat.count` | 总发送次数，包含第一次 | 10（缺省时也如此）；2–10,000 |
| [[ui:exp.repeatDuration]] | `repeat.duration_ms` | 从第一次发送起持续发送多久 | 10,000（缺省时也如此）；1–300,000 |
| [[ui:exp.repeatInterval]] | `repeat.interval_ms` | 两次发送之间的暂停 | 1,000；10–60,000；在文件中必需 |
| [[ui:exp.repeatJitter]] | `repeat.jitter_ms` | 每次暂停最多延长这么多，从本次运行的种子中抽取 | 0（缺省时也如此）；0–60,000 |

这些重复必须能容纳在运行的 300 秒内，并且*按时长*时所需的发送次数必须少于
10,000（时长除以间隔）。

### 负载 {#load}

[[ui:exp.loadOn]]，仅用于 [[ui:exp.node.http]]：请求按负载模式发送——恒定
速率、斜坡、阶梯、尖峰或随机到达——同时最多 512 个在途（默认 32 个），并
进行测量：延迟、错误、实际达到的速率。阈值决定步骤是否通过。负载取代重复和
重试（失败的请求只计数，不再重试），并且不会为之后的检查留下响应。它的字段
和结果见[负载测试](load.md)。

### 等待回复 {#reply}

[[ui:exp.expectReply]]，用于 [[ui:exp.node.osc]] 或 [[ui:exp.node.udp]]：消息
从等待回复的端口发出，因此会应答发送方的设备能被听到，只有当匹配的回复及时
到达时该步骤才通过。没有回复会使步骤失败——重试会再次发送。回复像等待节点
的一样存储在变量中。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.replyOn]] | `reply.bind` | 用于发送并监听的 `IP:port`；端口 0 取任意空闲端口 | `0.0.0.0:0` | 否 |
| [[ui:exp.replyAddress]] (OSC) | `reply.address` | 回复的地址模式，如同 [[[ui:exp.node.wait_osc]]](#node-wait_osc) | `/*` | 是 |
| [[ui:exp.argRules]] (OSC) | `reply.args` | 参数规则，如同 [[ui:exp.node.wait_osc]] | 无；最多 16 个 | 值：是 |
| [[ui:exp.replyMode]] (UDP) | `reply.mode` | `any`、`contains`、`regex` 或 `hex`——见[载荷匹配](#payload-matching) | `any`（缺省时也如此） | 否 |
| [[ui:field.pattern]] (UDP) | `reply.pattern` | 回复必须包含或匹配的内容 | 空；除非 `any` 否则必需 | 是 |
| [[ui:exp.waitTimeout]] | `reply.timeout_ms` | 等待多久 | 2,000（缺省时也如此）；1–120,000 | 否 |
| [[ui:exp.replyVariable]] | `reply.variable` | 存储回复的变量 | `reply`（缺省时也如此） | 否 |

回复的端口像等待节点的一样，在第一个步骤之前打开。

## 操作 {#actions}

发送的节点。动作之后的等待从该动作开始的那一刻起统计消息。

### HTTP 请求 {#node-http}

发送一个 HTTP 请求，并把响应保留给其后的检查、分支和 [[ui:exp.node.extract]]
节点。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.method]] | `request.method` | GET、HEAD、POST、PUT、PATCH、DELETE 或 OPTIONS（文件可以指定任意方法） | `GET` | 否 |
| URL | `request.url` | 一个 `http://` 或 `https://` URL | `http://127.0.0.1:8080/` | 是 |
| [[ui:common.timeoutMs]] | `request.timeout_ms` | 用于整个交换 | 4,000（缺省时为 10,000）；1–120,000 | 否 |
| [[ui:exp.headers]] | `request.headers` | `[[name, value], …]`；名称为空的行会被跳过 | 无 | 是，名称和值 |
| [[ui:exp.body]] | `request.body` | 文本，没有时为 `null` | `null` | 是 |
| [[ui:field.auth]] | `request.auth` | [[ui:http.auth.none]]、[[ui:http.auth.basic]]、[[ui:http.auth.bearer]] 或 [[ui:http.auth.digest]]，带 [[ui:field.username]] 和 [[ui:field.password]]，或 [[ui:field.token]] | 无 | 是 |

- 任何应答都会让步骤通过，包括 404 和 500：用
  [[[ui:exp.node.assert_status]]](#node-assert_status) 检查状态，或用
  [[[ui:exp.node.branch_status]]](#node-branch_status) 对其分支。得不到应答的请求——
  被拒绝、超时、无法解析的名称、不受信任的证书——会使步骤失败。
- 重定向会被跟随，最多十次。`https://` 证书会被验证。
- 响应正文最多保留 256 KiB 供检查使用；更大的正文会在此处截断（当检查所查找
  的内容可能位于截断之后时，检查会说明这一点）。
- Digest 会应答服务器的 401 质询并再次发送请求。凭据只进入请求：步骤、报告
  和检查器从不显示 `Authorization` 标头。把密码写成 `{{secret.NAME}}`。
- 当实验保留 Cookie 时（[[ui:exp.params]] 下默认开启），服务器设置的内容会
  在本次运行后续发往它们的请求中带回。

输出：[[ui:exp.outputPort]]。设置：重试、重复、负载。

```json
{ "id": "cue", "type": "http", "x": 270, "y": 80,
  "request": { "method": "POST", "url": "{{api}}/cue", "headers": [["Content-Type", "application/json"]],
               "body": "{\"cue\": 1}", "timeout_ms": 5000,
               "auth": { "scheme": "bearer", "token": "{{secret.API_TOKEN}}" } } }
```

另见 [HTTP](../protocols/http.md)。

### TCP 消息 {#node-tcp}

通过 TCP 连接到主机，写入载荷，最多等待 250 ms 以获取应答的首批字节（它最多
读取 1,024 字节，一次），然后关闭连接。应答的大小会被报告，但不检查。

在[检查器](../tools/inspector.md)中，该步骤是两帧 `tcp`，来源为 `experiment`：
写入的载荷，以及收到应答时读取的应答。两者中正在使用的机密都会被遮蔽，如同
任何帧一样。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.host]] | `host` | 主机名或 IP 地址 | `127.0.0.1` | 是 |
| [[ui:exp.port]] | `port` | | 9000；1–65,535 | 否 |
| [[ui:common.timeoutMs]] | `timeout_ms` | 用于连接、写入和应答的合计 | 4,000（缺省时也如此）；1–120,000 | 否 |
| [[ui:exp.payload]] | `payload` | 连接后写入的文本，以 UTF-8 编码 | `hello` | 是 |

当连接被拒绝、名称无法解析或时间耗尽时，该步骤失败。输出：
[[ui:exp.outputPort]]。设置：重试、重复。[[ui:exp.sendNow]] 连接并写入一次
载荷，节点的结果会说明发送了多少字节、返回了多少字节。

```json
{ "id": "go", "type": "tcp", "x": 270, "y": 80, "host": "127.0.0.1", "port": 5000, "payload": "GO\r\n", "timeout_ms": 2000 }
```

### OSC 消息 {#node-osc}

通过 UDP 发送一条 OSC 1.0 消息。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:common.target]] | `target` | `IP:port` 或 `host:port`；主机名在步骤发送时解析，它有一个 IPv4 地址时就取该地址 | `127.0.0.1:9000` | 是 |
| [[ui:common.address]] | `address` | 以 `/` 开头 | `/test` | 是 |
| [[ui:exp.arguments]] | `args` | `[{ "type", "value" }, …]`——`int`、`float`、`str`、`long`、`double`、`bool`、`blob`（字节）、`nil`（无值） | 无 | 文本（`str`）值：是 |
| [[ui:exp.expectReply]] | `reply` | 可选：发送并等待应答——见[等待回复](#reply) | 关闭 | |

输出：[[ui:exp.outputPort]]；期望回复时，只有回复到达才会跟随它。设置：重试、
重复、回复。⚡ 属性中的 [[ui:exp.routeThrough]] 会在它前面放置一个
[[[ui:exp.node.impairment]]](#node-impairment)。

```json
{ "id": "fader", "type": "osc", "x": 270, "y": 80, "target": "{{device}}", "address": "/fader/1",
  "args": [{ "type": "float", "value": 0.75 }] }
```

另见 [OSC](../protocols/osc.md)。

### UDP 数据报 {#node-udp}

把一个文本载荷作为一个 UDP 数据报发送到一个或多个目标。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:common.target]] | `target` | `IP:port` 或 `host:port`；用逗号、分号或换行分隔的多个地址各得到该数据报。主机名在步骤发送时解析，它有一个 IPv4 地址时就取该地址 | `127.0.0.1:9000` | 是 |
| [[ui:exp.payload]] | `text` | 载荷，以 UTF-8 编码 | `hello`；最多 65,507 字节 | 是 |
| [[ui:exp.expectReply]] | `reply` | 可选：发送并等待应答——见[等待回复](#reply) | 关闭 | |

如果任何目标无法到达，该步骤失败。输出：[[ui:exp.outputPort]]。
设置：重试、重复、回复。

```json
{ "id": "ping", "type": "udp", "x": 270, "y": 80, "target": "{{device}}", "text": "PING {{run.id}}",
  "reply": { "bind": "0.0.0.0:0", "mode": "contains", "pattern": "PONG", "timeout_ms": 1000, "variable": "pong" } }
```

### MQTT 发布 {#node-mqtt}

连接到 MQTT 代理，发布一条消息并断开。连接是明文 TCP 上的 MQTT 3.1.1，使用
干净会话，没有用户名或密码。连接、发布以及代理的确认都必须发生在 15 秒内。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.broker]] | `host` | 代理的主机名或地址 | `127.0.0.1` | 是 |
| [[ui:exp.port]] | `port` | | 1883；1–65,535 | 否 |
| [[ui:exp.topic]] | `topic` | 没有通配符（`+`、`#`） | `lab/test` | 是 |
| [[ui:exp.payload]] | `payload` | 消息，作为文本 | `hello` | 是 |
| QoS | `qos` | 0、1 或 2 | 0 | 否 |
| [[ui:exp.retain]] | `retain` | `true`：代理把它保留为该主题的值 | `false` | 否 |

文件中的全部六个键都是必需的。当代理无法到达或拒绝连接或消息时，该步骤失败。
输出：[[ui:exp.outputPort]]。设置：重试、重复。

```json
{ "id": "light", "type": "mqtt", "x": 270, "y": 80, "host": "{{broker}}", "port": 1883,
  "topic": "lab/light/1/set", "payload": "on", "qos": 1, "retain": false }
```

另见 [MQTT](../protocols/mqtt.md)。

### WebSocket 连接 {#node-ws_connect}

为本次运行的剩余时间打开一个 WebSocket，直到某个
[[[ui:exp.node.ws_close]]](#node-ws_close) 为止。从那时起到达的内容会保留给它上面的
[[[ui:exp.node.wait_ws]]](#node-wait_ws) 步骤。URL 和标头在步骤运行时解析，因此
早先提取的令牌可以出现在其中。再次运行——在 [[ui:exp.node.loop]] 中——它会先
关闭之前的连接并打开一个新的。运行结束时，无论以何种方式，它的连接都会以
关闭帧关闭。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| URL | `url` | 一个 `ws://` 或 `wss://` URL | `ws://127.0.0.1:9001/` | 是 |
| [[ui:exp.headers]] | `headers` | 随升级请求发送的 `[[name, value], …]` | 无 | 是，名称和值 |
| [[ui:exp.wsProtocols]] | `protocols` | 要提供的子协议，按偏好顺序；服务器选择一个 | 无 | 否 |
| [[ui:common.timeoutMs]] | `timeout_ms` | 用于连接和升级 | 5,000（缺省时为 10,000）；1–120,000 | 否 |

`wss://` 信任与 `https://` 相同的证书。当连接或升级失败时，该步骤失败；
服务器的状态在原因之中。输出：[[ui:exp.outputPort]]。设置：重试（不是重复）。

```json
{ "id": "socket", "type": "ws_connect", "x": 270, "y": 80, "url": "ws://127.0.0.1:9001/chat",
  "headers": [["Authorization", "Bearer {{token}}"]], "protocols": ["chat.v1"], "timeout_ms": 5000 }
```

另见 [WebSocket](../protocols/websocket.md)。

### WebSocket 发送 {#node-ws_send}

在某个 [[ui:exp.node.ws_connect]] 打开的连接上发送一条消息。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | 本实验某个 [[ui:exp.node.ws_connect]] 节点的 id | 第一个 | 否 |
| [[ui:exp.wsFormat]] | `binary` | [[ui:exp.wsText]]（`false`），或 [[ui:exp.wsBinary]]（`true`）：载荷是以十六进制写出的字节，`de ad be ef` | `false`（缺省时也如此） | 否 |
| [[ui:exp.payload]] | `text` | 消息 | `hello`；最多 16 MiB | 是 |

在其路径上，连接必须位于发送之前；连接未打开的发送会失败。应答从消息写出
那一刻起统计。输出：[[ui:exp.outputPort]]。设置：重试、重复。

```json
{ "id": "hello", "type": "ws_send", "x": 500, "y": 80, "connection": "socket",
  "text": "{\"type\":\"ping\",\"id\":\"{{uuid}}\"}", "binary": false }
```

### WebSocket 关闭 {#node-ws_close}

通过关闭握手关闭一个连接。时间线会说明是谁关闭的：此步骤、之前的服务器
（带其代码），或一个已经断开的连接。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | 本实验某个 [[ui:exp.node.ws_connect]] 节点的 id | 第一个 | 否 |
| [[ui:field.code]] | `code` | 1000（正常），或 3000–4999 供应用程序自定义 | 1000（缺省时也如此） | 否 |
| [[ui:field.reason]] | `reason` | 随代码一起发送 | 空；模板解析后最多 123 字节 | 是 |

输出：[[ui:exp.outputPort]]。无设置。

```json
{ "id": "bye", "type": "ws_close", "x": 960, "y": 80, "connection": "socket", "code": 1000, "reason": "done" }
```

### 日志标记 {#node-log}

往时间线和报告中写入一行——一个检查点，或运行达到的值。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.logMessage]] | `message` | 文本 | `Check point`；最多 10,000 个字符 | 是 |

输出：[[ui:exp.outputPort]]。无设置。

```json
{ "id": "ready", "type": "log", "x": 500, "y": 80, "message": "device {{device}} ready" }
```

## 等待 {#waits}

[[ui:exp.group.observe]] 组：等待某些内容到达的节点。它们共享这些规则：

- **它们从运行开始就监听。** 等待节点的端口或代理订阅在第一个步骤之前打开，
  因此比下一个步骤更早应答的设备不会被错过。同一地址上的两个等待节点共用
  同一个套接字。
- **它们从自己分支上的最新动作开始计数。** 在该分支最后一次请求之前到达的
  消息不算作它的应答；在任何动作之前，从运行开始以来的一切都计入。
- **第一条匹配的消息会被取用。** 一个等待节点取用的消息不会被另一个看到。
- **[[ui:exp.portMatched]] 或 [[ui:exp.portTimeout]]。** 匹配时，消息存储在
  等待节点的变量中，流程跟随 [[ui:exp.portMatched]]。时间耗尽时，如果该输出
  有连线，则跟随 [[ui:exp.portTimeout]]；否则步骤失败，并说明有多少其他消息
  到达。
- 每个套接字保留最新的 1,024 条消息（以及 64 MiB）；更早的会被丢弃，
  超时会说明丢弃了多少条。
- [[ui:exp.listenNow]] 只用那一个步骤监听，从现在开始。

输出：[[ui:exp.portMatched]]（必需）、[[ui:exp.portTimeout]]（可选）。
设置：重试。

### 载荷匹配 {#payload-matching}

[[ui:exp.node.wait_udp]]、[[ui:exp.node.wait_mqtt]]、[[ui:exp.node.wait_ws]]
和 UDP 回复选择载荷必须是什么样子：

| 选项 | 在文件中 | 当载荷 |
| --- | --- | --- |
| [[ui:exp.mode.any]] | `any` | 是任何内容时匹配 |
| [[ui:exp.mode.contains]] | `contains` | 按 UTF-8 文本读取，包含该模式（区分大小写）时匹配 |
| [[ui:exp.mode.regex]] | `regex` | 按 UTF-8 文本读取，匹配该正则表达式时匹配 |
| [[ui:exp.mode.hex]] | `hex` | 包含这些字节，写成十六进制对：`de ad be ef`、`deadbeef`、`0xde,0xad`、`DE:AD` 时匹配 |

匹配到的消息存储为一个对象。后续步骤把它的字段读作 `{{reply.text}}`
（用变量的名称代替 `reply`）：

| 字段 | 内容 |
| --- | --- |
| `text` | 作为文本的载荷 |
| `hex`, `bytes` | 十六进制表示的载荷（其前 1,024 字节），以及它的字节大小 |
| `match` | 匹配到的内容：文本、正则表达式的第一个分组（或整个匹配），或字节 |
| `from` | 发送方的 `IP:port` |
| `ms` | 从分支的最新动作（或运行开始）到该消息的毫秒数 |
| `topic` | [[ui:exp.node.wait_mqtt]]：它被发布到的主题 |
| `json`, `kind` | [[ui:exp.node.wait_ws]]：解析为 JSON 的消息（不是时则为 `null`），以及 `text` 或 `binary` |

### 等待 OSC {#node-wait_osc}

等待一条 OSC 消息，其地址匹配某个模式，且其参数满足每一条规则。在 bundle 中，
被取用的是第一条匹配的消息。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | 要监听的 `IP:port`；`0.0.0.0` 表示每块网卡 | `127.0.0.1:9001` | 否 |
| [[ui:exp.addressPattern]] | `address` | `*` 任意字符，`?` 一个字符，`[0-9]` 一个集合（`[!0-9]` 表示其外），`{ping,pong}` 二选一；通配符只在一个 `/` 段内 | `/pong`；最多 512 个字符 | 是 |
| [[ui:exp.argRules]] | `args` | `[{ "index", "op", "value" }, …]`：参数 `index` 用 `op` 与 `value` 比较（[比较](#comparisons)）；全部必须成立 | 无；最多 16 个，index 0–63 | 值：是 |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2,000（缺省时也如此）；1–120,000 | 否 |
| [[ui:exp.replyVariable]] | `variable` | 消息存储在哪里 | `reply`（缺省时也如此） | 否 |

参数按文本比较：数字按书写形式，字符串不带引号，`true`/`false`，blob 用
十六进制。针对消息没有的参数的规则不成立。存储的消息包含 `address`、`args`
（`{{reply.args[0]}}`）、`from` 和 `ms`。

```json
{ "id": "status", "type": "wait_osc", "x": 500, "y": 80, "bind": "0.0.0.0:9001", "address": "/status",
  "args": [{ "index": 0, "op": "eq", "value": "ready" }], "timeout_ms": 5000, "variable": "reply" }
```

### 等待 UDP {#node-wait_udp}

等待一个载荷匹配的 UDP 数据报。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | 要监听的 `IP:port` | `127.0.0.1:9001` | 否 |
| [[ui:exp.waitMode]] | `mode` | 见[载荷匹配](#payload-matching) | `contains`（缺省时为 `any`） | 否 |
| [[ui:field.pattern]] | `pattern` | 载荷必须包含或匹配的内容 | `pong`；除非 `any` 否则必需 | 是 |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2,000（缺省时也如此）；1–120,000 | 否 |
| [[ui:exp.replyVariable]] | `variable` | | `reply`（缺省时也如此） | 否 |

```json
{ "id": "ready", "type": "wait_udp", "x": 500, "y": 80, "bind": "0.0.0.0:9002", "mode": "contains",
  "pattern": "READY", "timeout_ms": 5000, "variable": "reply" }
```

### 等待 MQTT {#node-wait_mqtt}

等待一条发布到某个代理的某个主题、且载荷匹配的消息。本次运行在第一个步骤
之前连接并订阅。代理在订阅时重放的保留消息会被忽略：只有在运行开始之后发布
的内容才计入。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.broker]] | `host` | 代理 | `127.0.0.1` | 仅参数 |
| [[ui:exp.port]] | `port` | | 1883；1–65,535 | 否 |
| [[ui:exp.topicFilter]] | `topic` | 一个过滤器：`+` 是任意一级，`#` 是其下的所有内容（仅限最后） | `lab/#` | 仅参数 |
| [[ui:exp.waitMode]] | `mode` | 见[载荷匹配](#payload-matching) | `any`（缺省时也如此） | 否 |
| [[ui:field.pattern]] | `pattern` | | 空；除非 `any` 否则必需 | 是 |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2,000（缺省时也如此）；1–120,000 | 否 |
| [[ui:exp.replyVariable]] | `variable` | | `reply`（缺省时也如此） | 否 |

```json
{ "id": "state", "type": "wait_mqtt", "x": 500, "y": 80, "host": "{{broker}}", "port": 1883,
  "topic": "lab/+/state", "mode": "contains", "pattern": "on", "timeout_ms": 5000, "variable": "reply" }
```

### 等待 HTTP 请求 {#node-wait_http}

等待一个 HTTP 请求——webhook、回调——发往本次运行在该地址上的
[[[ui:exp.node.emulator]]](#node-emulator)，或者，当本次运行在那里没有 HTTP
模拟器时，发往运行自己的一个监听器，它以 204 应答每个请求。该请求必须匹配
方法、路径和每一项条件。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | `IP:port` | `127.0.0.1:18080`——新的 [[ui:exp.node.emulator]] 监听的地方 | 否 |
| [[ui:exp.method]] | `method` | 一个方法，或 [[ui:emu.methodAny]]（`ANY`）；GET 也接受 HEAD | `ANY`（缺省时也如此） | 否 |
| [[ui:exp.path]] | `path` | `/hooks/:name` 命名一个路径段（`{{request.params.name}}`）；末尾的 `/*` 取其余部分 | `/*`（缺省时也如此）；最多 512 个字符 | 是 |
| [[ui:emu.conditions]] | `when` | `[{ "on", "name", "op", "value" }, …]`，作用于 `header`、`query` 参数、`body` 或 `json` 路径；每一项都必须成立 | 无；最多 16 个 | 是，名称和值 |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 5,000（缺省时为 2,000）；1–120,000 | 否 |
| [[ui:exp.replyVariable]] | `variable` | | `request`（缺省时也如此） | 否 |

存储的请求包含 `method`、`path`、`query`、`headers`、`body`、`json`、
`params`、`from` 和 `ms`：`{{request.json.event}}`、`{{request.headers.x-key}}`。

```json
{ "id": "hook", "type": "wait_http", "x": 500, "y": 80, "bind": "127.0.0.1:18081", "method": "POST",
  "path": "/hooks/:name", "when": [{ "on": "json", "name": "$.event", "op": "eq", "value": "deploy" }],
  "timeout_ms": 5000, "variable": "request" }
```

### 等待 WebSocket {#node-wait_ws}

等待某个 [[ui:exp.node.ws_connect]] 打开的连接上的一条消息，其载荷匹配。在该
分支最新动作之后的消息才计入——连接本身、一次发送，或任何其他请求。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | 本实验某个 [[ui:exp.node.ws_connect]] 节点的 id | 第一个 | 否 |
| [[ui:exp.waitMode]] | `mode` | 见[载荷匹配](#payload-matching) | `any`（缺省时也如此） | 否 |
| [[ui:field.pattern]] | `pattern` | | 空；除非 `any` 否则必需 | 是 |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2,000（缺省时也如此）；1–120,000 | 否 |
| [[ui:exp.replyVariable]] | `variable` | | `reply`（缺省时也如此） | 否 |

JSON 消息可以逐字段读取：`{{reply.json.type}}`。在其路径上，连接必须位于
等待之前。

```json
{ "id": "pong", "type": "wait_ws", "x": 730, "y": 80, "connection": "socket", "mode": "contains",
  "pattern": "pong", "timeout_ms": 3000, "variable": "reply" }
```

## 模拟 {#emulation}

### 模拟器 {#node-emulator}

在整个运行期间扮演一个依赖——HTTP API、OSC、UDP 或 TCP 设备、MQTT 代理。
它在第一个步骤之前打开，并一直应答到运行结束；在流程中该步骤立即通过。
它收到的内容会按规则逐条统计在运行报告中。

| 字段 | 在文件中 | 内容 | 默认值 |
| --- | --- | --- | --- |
| [[ui:emu.edit]] | `emulator` | 模拟器：`name`、`bind`（`IP:port`）、`protocol`（`http`、`osc`、`udp`、`tcp`、`mqtt`）、它的路由或规则，以及可选的 `outage` | 一个名为 *API*、位于 `127.0.0.1:18080`、应答 `/health` 的 HTTP API |

属性用一行显示它扮演的内容。[[ui:emu.edit]] 打开它的规则，与
[[[ui:nav.emulators]]](../tools/emulators.md) 界面是同一个编辑器；
[[ui:emu.toLibrary]] 在模拟器库中保留一个副本，[[ui:emu.fromLibrary]] 用库中
的一个副本替换这个。规则——路由、响应、故障、停机——在那里介绍。

- HTTP 模拟器也是其地址上某个 [[[ui:exp.node.wait_http]]](#node-wait_http)
  所监听的对象；OSC 或 UDP 模拟器与本次运行在那里等待的节点共用端口。
- 同一种传输协议的两个模拟器在一次运行中不能共用一个端口。
- [[[ui:exp.node.emulator_state]]](#node-emulator_state) 让它停机并恢复。

输出：[[ui:exp.outputPort]]。无设置。

```json
{ "id": "api", "type": "emulator", "x": 270, "y": 80,
  "emulator": { "name": "Orders API", "bind": "127.0.0.1:18080", "protocol": "http",
    "routes": [{ "method": "GET", "path": "/orders/:id", "order": "sequence",
                 "responses": [{ "status": 503 }, { "status": 200, "body": "{\"id\":\"{{request.params.id}}\"}" }] }] } }
```

## 故障 {#faults}

按需破坏东西的节点。一分支的 [[ui:exp.node.delay]] 节点和这些位于流量旁的
节点读起来就像一份计划；[按计划注入故障](faults.md)会展示如何做到。

### 网络损伤 {#node-impairment}

一个用于整个运行的损伤中继：被测系统发往（或连接到）[[ui:exp.relayListen]]，
而不是真正的目标；中继转发到 [[ui:exp.relayTarget]]，应答以同样的方式返回，
并受到损伤配置的影响。它在第一个步骤之前打开，并在运行结束时关闭，无论以
何种方式，因此不会有任何东西一直处于损伤状态；在流程中该步骤立即通过。
每个决定都从本次运行的种子中抽取：相同的种子和相同的流量会得到相同的命运。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.relayListen]] | `listen` | 被测系统发往的 `IP:port` | `127.0.0.1:9010` | 仅参数 |
| [[ui:exp.relayTarget]] | `target` | 真正目的地的 `IP:port`，或 `host:port`——主机名在运行开始时解析，找不到的名称会在该节点处停止运行 | `127.0.0.1:9000` | 仅参数 |
| [[ui:ns.protocol]] | `protocol` | UDP（`udp`）：每个数据报有各自的命运；TCP（`tcp`）：每个连接都与一个自己的、通往目标的连接相接，两个流都会受到损伤 | UDP（缺省时为 `udp`） | 否 |
| [[ui:ns.preset]] 及其下的值 | `profile` | 中继对流量做什么——见[损伤配置](#impair-profile) | [[ui:ns.preset.lan]]（缺省时无损伤） | 否 |

中继的监听地址不能是本次运行中的另一个套接字，中继之间也不能循环转发。
以名称给出的目标会在解析后被跟随，因此经由名称的循环会在运行开始时停止运行。
报告把中继的每个阶段分开统计。

输出：[[ui:exp.outputPort]]。无设置。

```json
{ "id": "relay", "type": "impairment", "x": 270, "y": 80, "listen": "127.0.0.1:9010", "target": "{{device}}",
  "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } }
```

#### 损伤配置 {#impair-profile}

预设芯片——[[ui:ns.preset.lan]]、[[ui:ns.preset.wifi]]、
[[ui:ns.preset.4g]]、[[ui:ns.preset.satellite]]、
[[ui:ns.preset.intermittent]]、[[ui:ns.preset.offline]]——会填入每个值；
之后可以更改其中任何一个。中继只读取其协议的值；在文件中每个键都可以省略
（零、关闭）。

| 字段 | 在文件中 | 内容 | 限制 | 协议 |
| --- | --- | --- | --- | --- |
| — | `name` | 时间线和报告的标签：预设的键（`lan`、`wifi`、`4g`、`satellite`、`intermittent`、`offline`）或您自己的 | 最多 60 个字符 | 两者 |
| [[ui:ns.offline]] | `offline` | 什么都通不过 | `true` / `false` | 两者 |
| [[ui:ns.latency]] | `latency_ms` | 加到每个数据包或流的每个数据块上的延迟 | 0–60,000（滑块到 1,000） | 两者 |
| [[ui:ns.jitter]] | `jitter_ms` | 最多额外这么久的随机延迟；TCP 流保持有序 | 0–60,000（滑块到 500） | 两者 |
| [[ui:ns.rate]] | `rate_kbps` | 带宽限制，0 表示不限。UDP：排队超过一秒后，数据报会被作为限速丢弃；TCP：发送方被放慢，不丢弃任何内容 | 0，或 8–10,000,000 | 两者 |
| [[ui:ns.loss]] | `loss` | 数据报被丢弃的概率 | 0–1（滑块显示 %） | UDP |
| [[ui:ns.burst]], [[ui:ns.burstLength]] | `burst_start`, `burst_length` | 一次突发丢包开始的概率，以及它平均持续多少个数据报 | 0–1；开启突发时为 1–1,000 | UDP |
| [[ui:ns.duplicate]] | `duplicate` | 数据报被发送两次的概率 | 0–1 | UDP |
| [[ui:ns.corrupt]] | `corrupt` | 数据报有一位被翻转的概率 | 0–1 | UDP |
| [[ui:ns.reorder]] | `reorder` | 数据报被扣住、以致后来者超过它的概率 | 0–1 | UDP |
| [[ui:ns.reset]] | `reset` | 流的某个数据块改为重置其连接的概率——双方都收到重置 | 0–1 | TCP |
| [[ui:ns.stall]] | `stall` | 某个数据块使其连接保持半开的概率：两个方向都不再有内容通过，且双方都未被告知 | 0–1 | TCP |

关于中继、预设以及它们模拟的内容，详见
[[[ui:nav.netsim]]](../tools/impairment.md) 页面。

### 更改损伤 {#node-impairment_change}

从此步骤起，把本次运行中的某个 [[ui:exp.node.impairment]] 节点切换到另一个
损伤配置，而不断开其端口。至此为止的阶段会结束并计入报告。

| 字段 | 在文件中 | 内容 | 默认值 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.relay]] | `relay` | 本实验某个 [[ui:exp.node.impairment]] 节点的 id | 第一个 | 否 |
| [[ui:ns.preset]] 及其下的值 | `profile` | 它从今往后用什么施加损伤——见[损伤配置](#impair-profile)；中继读取它自己协议的值 | [[ui:ns.preset.offline]]（缺省时无损伤） | 否 |

如果中继没有运行，该步骤失败——比如说它转发失败了。输出：
[[ui:exp.outputPort]]。无设置。

```json
{ "id": "cut", "type": "impairment_change", "x": 730, "y": 200, "relay": "relay",
  "profile": { "name": "offline", "offline": true } }
```

### 模拟器停机与恢复 {#node-emulator_state}

让本次运行中的某个模拟器停机，或让它恢复。停机期间，HTTP 模拟器按
[[ui:exp.downFault]] 所说应答；TCP 设备和 MQTT 代理会断开连接并拒绝新连接；
OSC 和 UDP 设备不作任何应答。再次恢复后，模拟器会遵循它自己的停机计划
（如果有的话）。

| 字段 | 在文件中 | 内容 | 默认值 |
| --- | --- | --- | --- |
| [[ui:exp.emulatorNode]] | `emulator` | 本实验某个 [[ui:exp.node.emulator]] 节点的 id | 第一个 |
| [[ui:exp.emulatorDownState]] | `down` | [[ui:exp.emulatorGoesDown]]（`true`）或 [[ui:exp.emulatorComesUp]]（`false`） | 停机（缺省时为 `false`） |
| [[ui:exp.downFault]] | `fault` | 仅限 HTTP：[[ui:emu.outageFault.unavailable]]（`unavailable`）、[[ui:emu.outageFault.reset]]（`reset`：连接在没有应答的情况下关闭）或 [[ui:emu.outageFault.timeout]]（`timeout`：请求被挂起，直到客户端放弃，最多 120 s） | `unavailable`（缺省时也如此） |

输出：[[ui:exp.outputPort]]。无设置。没有任何内容使用模板。

```json
{ "id": "down", "type": "emulator_state", "x": 500, "y": 200, "emulator": "api", "down": true, "fault": "unavailable" }
```

## 数据 {#data}

### 提取值 {#node-extract}

把它路径上最新 HTTP 响应的一部分保存为变量，供之后的字段（`{{token}}`）、
检查和分支使用。在每条路径上，都必须有一个 HTTP 请求位于它之前。在
[[ui:exp.sendNow]] 响应中点击一个值会为您添加一个。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.variable]] | `variable` | 名称：字母、数字和 `_`，不以数字开头，不是保留字，不是参数的名称 | `token` | 否 |
| [[ui:exp.extractFrom]] | `from` | [[ui:exp.from.json]]（`json`）、[[ui:exp.from.header]]（`header`）、[[ui:exp.from.status]]（`status`）、[[ui:exp.from.body]]（`body`）或 [[ui:exp.from.regex]]（`regex`） | `json` | 否 |
| [[ui:exp.jsonPath]], [[ui:exp.headerName]] 或 [[ui:exp.pattern]] | `expr` | 一个 JSON 路径（`$.data.token`、`$.items[0]`、`$["first name"]`）、一个标头名称（任意大小写），或一个正则表达式——它的第一个分组，或整个匹配 | `$.token`；status 和 body 不使用 | 否 |

当没有可取的内容时该步骤失败：正文不是 JSON、路径或标头缺失、表达式不匹配，
或者——对于 JSON 字段或整个正文——正文比所保留的 256 KiB 更长。状态存储为
数字；其余存储为文本，或存储为找到的 JSON 值。输出：
[[ui:exp.outputPort]]。无设置。

```json
{ "id": "token", "type": "extract", "x": 500, "y": 80, "variable": "token", "from": "json", "expr": "$.data.token" }
```

关于变量的更多内容见[数据与模板](data.md)。

## 检查 {#checks}

检查要么通过，要么使运行失败。四项响应检查读取其路径上最新的 HTTP 响应，
因此在每条路径上，都必须有一个 HTTP 请求——不是处于负载下的请求——位于它们
之前。

### HTTP 状态 {#node-assert_status}

当最新响应的状态恰好是给定状态时通过。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.expectedStatus]] | `status` | | 200；100–599 | 否 |

输出：[[ui:exp.outputPort]]。无设置。

```json
{ "id": "ok", "type": "assert_status", "x": 500, "y": 80, "status": 200 }
```

### 响应文本 {#node-assert_body}

当最新响应的正文精确地（包括大小写）包含该文本时通过。正文只保留前
256 KiB：在已截断的正文中找不到的文本会以该原因失败。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.contains]] | `contains` | | `ok`；必需 | 是 |

输出：[[ui:exp.outputPort]]。无设置。

```json
{ "id": "ready", "type": "assert_body", "x": 500, "y": 80, "contains": "ready" }
```

### 响应标头 {#node-assert_header}

当最新响应具有该标头且其值包含该文本时通过。标头名称以任意大小写匹配；值
精确匹配。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.headerName]] | `name` | | `content-type`；必需 | 是 |
| [[ui:exp.contains]] | `contains` | 它的值必须包含的内容；为空：只要标头存在即可 | `application/json` | 是 |

输出：[[ui:exp.outputPort]]。无设置。

```json
{ "id": "json", "type": "assert_header", "x": 500, "y": 80, "name": "Content-Type", "contains": "json" }
```

### 响应时间 {#node-assert_latency}

当最新响应——从发送请求到其正文结束——最多用了这么长时间时通过。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.maxLatency]] | `max_ms` | | 1,000；1–120,000 | 否 |

输出：[[ui:exp.outputPort]]。无设置。

```json
{ "id": "fast", "type": "assert_latency", "x": 500, "y": 80, "max_ms": 250 }
```

### 检查值 {#node-assert_value}

把一个值——通常是写成模板的变量——与一个期望值比较，当比较成立时通过。

| 字段 | 在文件中 | 内容 | 默认值 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.value]] | `value` | 被比较的内容：`{{token}}`、`{{reply.args[0]}}` | `{{token}}` | 是 |
| [[ui:exp.operator]] | `op` | 见[比较](#comparisons) | [[ui:exp.op.not_empty]] | 否 |
| [[ui:exp.expected]] | `expected` | [[ui:exp.op.empty]] 和 [[ui:exp.op.not_empty]] 不使用 | 空（缺省时也如此） | 是 |

输出：[[ui:exp.outputPort]]。无设置。

```json
{ "id": "state", "type": "assert_value", "x": 730, "y": 80, "value": "{{state}}", "op": "eq", "expected": "ready" }
```

### 比较 {#comparisons}

[[ui:exp.node.assert_value]]、[[ui:exp.node.branch_value]]、[[ui:exp.node.loop]]
的退出条件、OSC 参数规则和 HTTP 条件以相同的方式比较：

| 选项 | 在文件中 | 当值 |
| --- | --- | --- |
| [[ui:exp.op.eq]] | `eq` | 等于期望值——两者都是数字时按数字比较（`200` = `200.0`），否则按精确文本 |
| [[ui:exp.op.ne]] | `ne` | 按相同规则不等于它时成立 |
| [[ui:exp.op.lt]], [[ui:exp.op.le]], [[ui:exp.op.gt]], [[ui:exp.op.ge]] | `lt`, `le`, `gt`, `ge` | 小于、至多、大于、至少——两者都必须是数字：否则检查、分支或 [[ui:exp.node.loop]] 会使步骤失败，而参数规则或 HTTP 条件不成立 |
| [[ui:exp.op.contains]] | `contains` | 包含期望文本时成立 |
| [[ui:exp.op.matches]] | `matches` | 匹配期望的正则表达式时成立 |
| [[ui:exp.op.empty]], [[ui:exp.op.not_empty]] | `empty`, `not_empty` | 为空（空格算作空）/ 不为空 |

## 流程 {#flow}

决定运行走向的节点。关于分支、汇合和循环的更多内容见[流程](flow.md)。

### 开始 {#node-start}

运行开始的地方；每个实验恰好有一个。它没有输入，也没有字段。时间线的第一行
给出本次运行的种子。

输出：[[ui:exp.outputPort]]，必需。从它引出的多条连线会同时启动并行分支。

```json
{ "id": "start", "type": "start", "x": 40, "y": 80 }
```

### 结束 {#node-end}

运行完成的地方；每个实验恰好有一个，它没有输出。多个分支可以通向它：运行只
通过一次，在最后一个分支结束后，并且只有在没有任何分支失败的情况下。从未
到达 [[ui:exp.node.end]] 的运行会失败。

```json
{ "id": "end", "type": "end", "x": 960, "y": 80 }
```

### 延时 {#node-delay}

在下一个步骤之前等待一段固定时间。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.delayMs]] | `ms` | | 300；0–60,000 | 否 |

输出：[[ui:exp.outputPort]]。无设置。更长的等待可以连续放几个，或放在
[[ui:exp.node.loop]] 中。

```json
{ "id": "pause", "type": "delay", "x": 500, "y": 80, "ms": 500 }
```

### 状态分支 {#node-branch_status}

当最新 HTTP 响应具有此状态时选择 [[ui:exp.yes]]，否则选择 [[ui:exp.no]]。
在每条路径上都必须有一个 HTTP 请求位于它之前。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.expectedStatus]] | `status` | | 200；100–599 | 否 |

输出：[[ui:exp.yes]] 和 [[ui:exp.no]]，两者都必需。无设置。

```json
{ "id": "branch", "type": "branch_status", "x": 500, "y": 80, "status": 200 }
```

### 按值分支 {#node-branch_value}

当比较成立时选择 [[ui:exp.yes]]，否则选择 [[ui:exp.no]]。它的字段和
[比较](#comparisons)与 [[[ui:exp.node.assert_value]]](#node-assert_value) 相同；
无法进行的比较（对文本使用 `lt`）会使步骤失败。

| 字段 | 在文件中 | 内容 | 默认值 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.value]] | `value` | 被比较的内容 | `{{token}}` | 是 |
| [[ui:exp.operator]] | `op` | | [[ui:exp.op.eq]] | 否 |
| [[ui:exp.expected]] | `expected` | | 空（缺省时也如此） | 是 |

输出：[[ui:exp.yes]] 和 [[ui:exp.no]]，两者都必需。无设置。

```json
{ "id": "ok", "type": "branch_value", "x": 730, "y": 80, "value": "{{reply.args[0]}}", "op": "eq", "expected": "ok" }
```

### 并行分支 {#node-fork}

同时运行跟随 [[ui:exp.branch1]] 和 [[ui:exp.branch2]] 的内容，每个分支都有
自己的变量副本。每个输出可以有多条连线以形成更多分支。无字段。

输出：[[ui:exp.branch1]] 和 [[ui:exp.branch2]]，两者都必需。

```json
{ "id": "split", "type": "fork", "x": 270, "y": 80 }
```

### 汇合分支 {#node-join}

等待汇入它的每条连线都被到达，然后继续一次，合并各分支的变量——当两个分支
设置了同一个变量时，其连线在文件中更靠后的那个获胜——并采用其中最后一个有
响应者的最新 HTTP 响应。无字段。

只有都会运行的分支才会在此汇合：[[ui:exp.node.join]] 位于
[[ui:exp.node.branch_status]] 之后时，其 [[ui:exp.yes]] 和 [[ui:exp.no]]
永远不会同时发生，因此永远不会继续；当没有其他路径到达
[[ui:exp.node.end]] 时，运行会在该节点处失败，并说明它还在等待多少个分支。

输出：[[ui:exp.outputPort]]，必需。

任何其他有多条连线汇入的节点都会为每次到达运行一次。

```json
{ "id": "joined", "type": "join", "x": 730, "y": 80 }
```

### 循环 {#node-loop}

反复运行 [[ui:exp.portBody]] 上的步骤——它们会回连到它：最多若干次，并且当
它有退出条件时，直到该条件成立。

| 字段 | 在文件中 | 内容 | 默认值和限制 | 模板 |
| --- | --- | --- | --- | --- |
| [[ui:exp.loopMax]] | `max` | 最多迭代次数 | 5；1–1,000 | 否 |
| [[ui:exp.loopUntilOn]] | `until` | 可选的退出条件 `{ "value", "op", "expected" }`，如同 [[[ui:exp.node.assert_value]]](#node-assert_value) | 关闭 | 值和期望值：是 |

- 主体至少运行一次。退出条件在每次迭代后读取，因此主体可以设置它所检验的
  内容。
- 当条件成立时——或者没有条件时，在最后一次迭代之后——跟随
  [[ui:exp.portDone]]。
- 当迭代在条件成立之前用完时，跟随 [[ui:exp.portLimit]]。它上面没有连线时，
  这会使步骤失败。
- 在主体内部，`{{counter}}` 是迭代的序号。
- 主体作为一个分支运行：其中每个输出只有一条连线；它不包含
  [[ui:exp.node.start]]、[[ui:exp.node.end]]、[[ui:exp.node.fork]]、
  [[ui:exp.node.join]] 或其他 [[ui:exp.node.loop]]；它只能通过
  [[ui:exp.portBody]] 进入；其中每条连线都在主体内继续，或回连到循环。

输出：[[ui:exp.portBody]] 和 [[ui:exp.portDone]]（必需）、
[[ui:exp.portLimit]]（可选）。无设置。

```json
{ "id": "poll", "type": "loop", "x": 270, "y": 80, "max": 10,
  "until": { "value": "{{status.args[0]}}", "op": "eq", "expected": "ready" } }
```

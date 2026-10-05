---
title: 模拟器
description: 让 Signal Lab 充当另一端——HTTP API、OSC、UDP 或 TCP 设备，或者 MQTT 代理——按您的规则应答，按需出故障，并统计收到的内容。
---

# 模拟器

模拟器就是由 Signal Lab 扮演的、您的系统所对接的 API、设备或服务。它监听一个地址，并按规则应答：HTTP API 按路由应答，OSC、UDP 或 TCP 设备按“收到这个，就回复那个”应答，MQTT 代理则像任何代理一样工作，另外还有自己的规则。它可以变慢、出错，或时不时停机，以便测试依赖出现异常时您的系统会怎样。每次交互都会被计数、列出，并发送到[检查器](inspector.md)。

一个模拟器就是一个文档。[[ui:nav.emulators]] 界面维护一个模拟器库；同一个文档可以作为 [[ui:exp.node.emulator]] 节点在实验中运行，可以通过命令行 `signallab emulate` 运行，也可以通过 [API](../api/commands.md) 和 [MCP](../automation/mcp.md) 运行，在哪里应答方式都一样。

## 界面 {#screen}

左侧是库（[[ui:emu.library]]）：每个模拟器及其协议和地址，正在运行的模拟器带有一个跳动的圆点和请求计数。右侧是所选模拟器的设置和规则，其下方是它收到的内容（[[ui:emu.live]]）。

## 创建模拟器 {#create}

1. 按库顶部的某个按钮：

   | 按钮 | 创建 | 监听于 | 自带一条可直接使用的规则 |
   | --- | --- | --- | --- |
   | ＋ [[ui:emu.new.http]] | HTTP API | `127.0.0.1:18080` | `GET /health` → 200 `{"status":"ok"}` |
   | ＋ [[ui:emu.new.osc]] | OSC 设备 | `127.0.0.1:9100` | `/ping` → `/pong`，附带 int 类型的计数 |
   | ＋ [[ui:emu.new.udp]] | UDP 设备 | `127.0.0.1:7100` | 包含 `PING` 的数据报 → `PONG 1`、`PONG 2`、… |
   | ＋ [[ui:emu.new.tcp]] | TCP 设备 | `127.0.0.1:7200` | 包含 `PING` 的一行 → `PONG` |
   | ＋ [[ui:emu.new.mqtt]] | MQTT 代理 | `127.0.0.1:1883` | 发布到 `lab/<name>/set` 的消息 → 在 `lab/<name>/state` 上以保留消息的形式发布相同的载荷 |

   如果库中已有其他模拟器使用该端口，则改用下一个空闲端口。
2. 填写 [[ui:emu.name]]（最多 120 个字符）。
3. 设置 [[ui:emu.bind]]：`IP:port`。`127.0.0.1` 只应答本机；`0.0.0.0` 也应答网络中的其他机器。
4. 修改规则（见下文），并在 [[ui:emu.note]] 中写明它所代替的对象。

更改会自动保存。[[ui:emu.duplicate]] 会在下一个空闲端口上创建一个副本。[[ui:emu.delete]] 会再确认一次（[[ui:emu.confirmDelete]]），如果模拟器正在运行就停止它，然后将其从库中移除。

规则按从前到后的顺序尝试；第一条匹配的规则进行应答。每条规则的标题栏显示一行摘要；点击它即可展开或折叠该规则。↑ 和 ↓ 按钮用于移动规则，× 用于移除规则。

## 运行 {#run}

1. 选择模拟器，按 [[ui:emu.start]]。在按钮恢复之前，其端口就已打开：如果端口已被占用，或模拟器存在问题，会在这一步被拒绝并给出原因。
2. 将您的系统指向它。对于 HTTP API，[[ui:emu.copyUrl]] 会复制其地址（`http://127.0.0.1:18080`），每个路由也都有一个 [[ui:emu.copyRouteUrl]] 按钮，用于复制该路由自己的地址（路径中含有 `{{…}}` 模板时除外）。
3. 观察 [[ui:emu.received]] 逐渐填满。
4. 按 [[ui:emu.stop]]，或在控制台的任务条中停止它的任务。

按钮旁边的状态显示 [[ui:emu.notRunning]]、它在哪里应答，或者它已停机。

模拟器会一直按启动时的规则应答。如果在它运行时修改了它，会出现 [[ui:emu.restart]]：按下它，即可按当前的规则重新启动。在此之前，规则上的命中次数会被隐藏，因为它们属于旧的规则。

[[ui:emu.takeDown]] 会让正在运行的模拟器不可用，直到您按下 [[ui:emu.bringUp]]：HTTP 请求会得到 503，TCP 设备和 MQTT 代理会断开现有连接并拒绝新连接，OSC 或 UDP 设备则不作任何应答。参见[停机](#outage)。

同一种传输协议的两个模拟器不能共用一个端口：HTTP、TCP 和 MQTT 模拟器监听 TCP 端口，OSC 和 UDP 模拟器监听 UDP 端口。一个 HTTP API 和一个 OSC 设备可以都使用端口 8080；两个 HTTP API 则不行。在已被占用的端口上启动第二个模拟器时会被拒绝。

::: tip
在连接到[服务器](../server/index.md)的浏览器中，模拟器运行在服务器上。监听 `0.0.0.0` 的模拟器通过服务器的名称访问，[[ui:emu.copyUrl]] 复制的就是这个地址；监听 `127.0.0.1` 的模拟器只应答服务器自身上的程序。
:::

## 收到的内容 {#received}

运行期间，[[ui:emu.live]] 会统计：

| 计数 | 内容 |
| --- | --- |
| [[ui:emu.total]] | 收到的一切：请求、消息、行。 |
| [[ui:emu.unmatched]] | 没有任何规则接收的内容。没有路由的 HTTP 请求仍会得到应答（参见[无路由接收的请求](#fallback)）；其他协议则不应答。 |
| [[ui:emu.failed]] | 无法生成或发送回复的交互。 |
| [[ui:emu.down]] | 模拟器停机期间收到的内容。设置了停机计划，或有内容在停机时到达时才显示。永远不计入 [[ui:emu.unmatched]]。 |
| [[ui:emu.missed]] | 仅限 MQTT，发生时才显示：客户端落后太多而无法接收的消息。 |

每条规则的标题栏显示自启动以来它匹配的次数。

[[ui:emu.received]] 列出最新的 300 次交互，最新的在最前：

| 列 | 内容 |
| --- | --- |
| [[ui:emu.col.time]] | 到达的时间。 |
| [[ui:emu.col.from]] | 客户端的地址。 |
| [[ui:emu.col.request]] | 收到的内容，以协议记法表示：`GET /users/7`、`/ping 1`、`POWER?`。 |
| [[ui:emu.col.rule]] | 接收它的规则（`#2`），或 `—`。 |
| [[ui:emu.col.reply]] | 返回的内容：`200 OK · 37 B`、`/pong 3`、一段载荷；故障时为 [[ui:emu.held]] 或 [[ui:emu.closed]]；回复失败时为错误；在停机期间到达时为 [[ui:emu.wasDown]]。 |
| [[ui:emu.col.ms]] | 从到达到回复发出的时间，包括其延时。 |

行上的 ⌕ 按钮（[[ui:emu.inspectFrame]]）会在检查器中打开该交互（前提是当时捕获已开启）。当五分之一秒内到达的交互超过 200 次时，列表会跳过一部分，并注明跳过了多少。引擎会为每个正在运行的模拟器保留最新的 500 次交互及其收到的内容，供命令行、API 和 MCP 使用。

## HTTP API {#http}

一个 HTTP/1.1 服务器。每个请求由第一个接收它的路由应答。

### 路由 {#routes}

当请求的方法、路径和所有条件都匹配时，路由就会接收该请求。

| 字段 | 内容 |
| --- | --- |
| [[ui:emu.method]] | `GET`、`POST`、`PUT`、`PATCH`、`DELETE`、`HEAD`、`OPTIONS`，或 [[ui:emu.methodAny]]。`GET` 路由也会应答 `HEAD`。 |
| [[ui:emu.path]] | 以 `/` 开头。路径段 `:name` 匹配任意一个路径段，以 `{{request.params.name}}` 读取；最后一段为 `*` 时匹配其下的所有内容。末尾的 `/` 没有影响；查询字符串不属于路径。 |
| [[ui:emu.conditions]] | 每一条都必须满足。用 ＋ [[ui:emu.addCondition]] 添加。 |

路径示例：

| 路径 | 匹配 | 不匹配 |
| --- | --- | --- |
| `/health` | `/health`、`/health/` | `/health/db`、`/Health` |
| `/users/:id` | `/users/7`（`params.id` 为 `7`）、`/users/a%20b`（`a b`） | `/users`、`/users/7/orders` |
| `/files/*` | `/files`、`/files/a`、`/files/a/b/c` | `/file`、`/other/files/a` |

条件读取请求的某一部分（[[ui:emu.on]]），并进行比较：

| [[ui:emu.on]] | 名称 | 读取 |
| --- | --- | --- |
| [[ui:emu.on.header]] | 标头名称，不区分大小写 | 该标头的值；标头发送了多次时，各值以 `, ` 连接。 |
| [[ui:emu.on.query]] | 查询参数 | 解码后的值；重复出现时取第一个。 |
| [[ui:emu.on.body]] | — | 整个正文，作为文本。 |
| [[ui:emu.on.json]] | JSON 路径，例如 `$.user.id` | JSON 正文中的该字段。 |

比较方式有 [[ui:exp.op.eq]]、[[ui:exp.op.ne]]、[[ui:exp.op.lt]]、[[ui:exp.op.le]]、[[ui:exp.op.gt]]、[[ui:exp.op.ge]]、[[ui:exp.op.contains]]、[[ui:exp.op.matches]]、[[ui:exp.op.empty]] 和 [[ui:exp.op.not_empty]]。数字按数值比较，文本精确比较。不存在的标头、参数或字段视为空。无法进行的比较（例如文本与数字比较）不成立。

### 响应 {#responses}

一个路由有 1 到 16 个响应（[[ui:emu.responses]]）。

| 字段 | 内容 | 默认值 |
| --- | --- | --- |
| [[ui:emu.status]] | 100–599。 | 200 |
| [[ui:emu.fault]] | 不返回正常应答，而是出现其他情况；参见[故障](#faults)。 | [[ui:emu.fault.none]] |
| [[ui:emu.delay]] | 应答前等待多久，0–60,000 ms。 | 0 |
| [[ui:emu.jitter]] | 随机延长至多此值，0–60,000 ms。 | 0 |
| [[ui:emu.weight]] | 路由随机应答时它所占的比重。仅在这种情况下显示。 | 1 |
| [[ui:emu.headers]] | 最多 32 个。名称可以使用参数；值是[模板](#templates)。 | 无 |
| [[ui:emu.body]] | 一个[模板](#templates)，按书写的内容计算最多 256 KiB。 | 空 |

没有 `Content-Type` 标头时，有效 JSON 的正文以 `application/json` 发送，其他正文以 `text/plain; charset=utf-8` 发送。

有两个或更多响应时，[[ui:emu.order]] 决定请求得到哪一个：

| [[ui:emu.order]] | 请求得到 | 用途 |
| --- | --- | --- |
| [[ui:emu.order.sequence]] | 第一个、第二个……此后一直是最后一个：500、500、200、200、200…… | 重试：失败两次，然后成功。 |
| [[ui:emu.order.cycle]] | 最后一个之后再回到第一个：200、500、200、500…… | 有规律地时而失败的依赖。 |
| [[ui:emu.order.random]] | 每次按权重抽取。权重为 8 和 2 时，第一个约占 80% 的次数。至少有一个权重必须大于 0。 | 符合实际的失败比例。 |

[[ui:emu.preset]] 会为路由添加一个现成的响应：

| 预设 | 添加 |
| --- | --- |
| [[ui:emu.preset.ok]] | 200，`{"ok":true}` |
| [[ui:emu.preset.created]] | 201，`{"id":"{{uuid}}"}`，标头 `Location: {{request.path}}/{{counter}}` |
| [[ui:emu.preset.notFound]] | 404，`{"error":"not found"}` |
| [[ui:emu.preset.error]] | 500，`{"error":"internal"}` |
| [[ui:emu.preset.unavailable]] | 503，`{"error":"unavailable"}`，标头 `Retry-After: 1` |
| [[ui:emu.preset.slow]] | 2000 ms 后返回 200，`{"ok":true}` |
| [[ui:emu.preset.timeout]] | 故障 [[ui:emu.fault.timeout]] |
| [[ui:emu.preset.reset]] | 故障 [[ui:emu.fault.reset]] |
| [[ui:emu.preset.malformed]] | 200，`{"items":[{"id":1},{"id":2}]}`，带故障 [[ui:emu.fault.malformed]] |

### 故障 {#faults}

| [[ui:emu.fault]] | 客户端遇到的情况 |
| --- | --- |
| [[ui:emu.fault.none]] | 正常的响应。 |
| [[ui:emu.fault.timeout]] | 什么都没有。请求最多被挂起 2 分钟，然后连接关闭——因此测试的是客户端自己的超时。延时不适用。 |
| [[ui:emu.fault.reset]] | 经过延时后，连接在没有应答的情况下关闭。 |
| [[ui:emu.fault.malformed]] | 一个完整的 HTTP 应答，带有设定的状态和标头，但正文在中途截断：无法解析的 JSON。如果整个正文原本是 JSON，内容类型仍为 `application/json`。 |

### 无路由接收的请求 {#fallback}

[[ui:emu.fallback]] 决定不匹配任何路由的请求会得到什么：

- [[ui:emu.fallbackDefault]]——404，正文为 `{"error":"no_route"}`；
- [[ui:emu.fallbackCustom]]——您设置的响应，具备路由响应的全部内容。其中的 `{{counter}}` 统计没有路由接收的请求数。

无论哪种方式，该请求都计为 [[ui:emu.unmatched]]。

### HTTP 回复可以读取的内容 {#http-request}

| 模板 | 含义 |
| --- | --- |
| `{{request.method}}` | `GET`、`POST`、… |
| `{{request.path}}` | 路径，不含查询字符串。 |
| `{{request.params.id}}` | 名为 `:id` 的路径段。 |
| `{{request.query.page}}` | 解码后的查询参数。 |
| `{{request.headers.x-key}}` | 一个标头；名称为小写。 |
| `{{request.body}}` | 正文文本：其前 64 KiB。 |
| `{{request.json.name}}` | JSON 正文中的一个字段，前提是正文为 JSON 且不超过 64 KiB。 |
| `{{request.from}}` | 客户端的 `IP:port`。 |

大于 1 MiB 的请求正文会得到 413，并计为 [[ui:emu.failed]]。无法生成的回复（模板引用了请求中没有的内容）会得到 500，错误信息在其正文中，并计为 [[ui:emu.failed]]。

## OSC 设备 {#osc}

到达的每条消息（消息包中的每条消息各自单独处理）由它匹配的第一条规则应答。不是 OSC 的数据报计为 [[ui:emu.unmatched]]。

| 字段 | 内容 |
| --- | --- |
| [[ui:emu.address]] | OSC 1.0 地址模式：`*` 任意字符，`?` 一个字符，`[a-z]` 字符集，`{a,b}` 二选一，均限于一个路径段内（参见 [OSC](../protocols/osc.md#patterns)）。 |
| [[ui:exp.argRules]] | 最多 16 个针对参数的条件，与 [[ui:exp.node.wait_osc]] 中相同（参见[节点](../experiments/nodes.md#node-wait_osc)）。 |
| [[ui:emu.replyOn]] | 关闭时：接收消息，不作任何应答。 |
| [[ui:emu.replyAddress]] | 回复的地址，一个[模板](#templates)。 |
| [[ui:emu.replyArgs]] | 最多 16 个参数，每个包括 [[ui:emu.argType]]（`int`、`float`、`str`、`long`、`double`、`bool`、`blob`、`nil`）和一个 [[ui:emu.argValue]] 模板。 |
| [[ui:emu.to]] | 留空：回复到发送方的地址和端口。否则填写 `IP:port`。 |
| [[ui:emu.delay]]、[[ui:emu.jitter]] | 各为 0–60,000 ms。 |

填好模板后，参数的值会按其类型读取：类型为数值时，`{{request.args[0]}}` 会以数字形式回显第一个参数。`bool` 接受 `true`、`1`、`yes`、`on` 或 `false`、`0`、`no`、`off`；`blob` 接受十六进制字节；空值即该类型的零值。

回复从模拟器自己的端口发出，因此在发送端口上监听的客户端能收到它们。

OSC 回复可以读取 `{{request.address}}`、`{{request.args[0]}}` 和 `{{request.from}}`。

## UDP 设备 {#udp}

每个数据报由它匹配的第一条规则应答。

| 字段 | 内容 |
| --- | --- |
| [[ui:emu.match]] | [[ui:exp.mode.any]]、[[ui:exp.mode.contains]]、[[ui:exp.mode.regex]] 或 [[ui:exp.mode.hex]]。 |
| [[ui:emu.pattern]] | 要查找的文本、正则表达式或字节。 |
| [[ui:emu.reply]] | [[ui:emu.replyOff]]、[[ui:emu.replyText]] 或 [[ui:emu.replyHex]]，然后是回复本身，一个[模板](#templates)。 |
| [[ui:emu.to]] | 留空：回复给发送方。否则填写 `IP:port`。 |
| [[ui:emu.delay]]、[[ui:emu.jitter]] | 各为 0–60,000 ms。 |

UDP 或 TCP 回复可以读取：

| 模板 | 含义 |
| --- | --- |
| `{{request.text}}` | 载荷文本。 |
| `{{request.match}}` | 匹配到的内容：文本、正则表达式的第一个分组（或整个匹配）、字节。 |
| `{{request.hex}}` | 以十六进制字节表示的载荷，取前 1024 个字节。 |
| `{{request.bytes}}` | 载荷的大小。 |
| `{{request.from}}` | 发送方的 `IP:port`。 |

文本回复最多 65,507 字节。

## TCP 设备 {#tcp}

一种在 TCP 连接上按行通信的设备，例如投影机或矩阵切换器。客户端发送的每条消息由它匹配的第一条规则应答；回复通过同一连接返回。

| 字段 | 内容 |
| --- | --- |
| [[ui:emu.delimiter]] | 标志一条消息结束的字符，也会添加在每条回复和问候语之后：[[ui:emu.delimiter.lf]]（其前的 `\r` 会被去掉）、[[ui:emu.delimiter.crlf]]、[[ui:emu.delimiter.cr]] 或 [[ui:emu.delimiter.none]]。空行会被跳过。 |
| [[ui:emu.greeting]] | 客户端连接时发送；留空则不发送。可以读取 `{{request.from}}`。 |
| [[ui:emu.match]]、[[ui:emu.pattern]]、[[ui:emu.reply]] | 与 [UDP 设备](#udp)相同。 |
| [[ui:emu.close]] | 在此规则的回复之后关闭连接，例如对 `QUIT`。 |
| [[ui:emu.delay]]、[[ui:emu.jitter]] | 各为 0–60,000 ms。 |

超过 64 KiB 仍没有结束符的消息，会按其现状接收。

## MQTT 代理 {#mqtt}

一个基于明文 TCP 的小型 MQTT 3.1.1 代理。它做代理该做的事：客户端连接，用 `+` 和 `#` 订阅，以 QoS 0、1 和 2 发布，保留消息和遗嘱消息都有效，使用同一客户端 ID 的第二个连接会取代第一个。会话总是全新的：要求保留会话的客户端会得到一个新会话，也不会为不在线的客户端排队任何内容。

除此之外，发布给它的每条消息都会与规则进行比对：第一条匹配的规则还会发布一条回复——就像设备在报告自己做了什么。

| 字段 | 内容 |
| --- | --- |
| [[ui:emu.username]]、[[ui:emu.password]] | 设置了用户名时，客户端必须用该用户名和密码连接；留空：任何人都可以连接。只有密码而没有用户名会被拒绝，因为 MQTT 3.1.1 无法单独携带密码。 |
| [[ui:emu.retained]] | 最多 64 条消息（[[ui:emu.topic]]、[[ui:emu.payload]]、[[ui:emu.qos]]），从启动起即持有，如同以 retain 发布：订阅的客户端会首先收到它们。 |
| [[ui:emu.topicFilter]] | 规则接收哪些主题：`+` 匹配一级，`#` 匹配其余部分，例如 `lab/+/set`。 |
| [[ui:emu.match]]、[[ui:emu.pattern]] | 针对载荷的条件，与 [UDP 设备](#udp)相同。 |
| [[ui:emu.replyOn]] | 关闭时：接收消息，不再发布任何内容。 |
| [[ui:emu.replyTopic]]、[[ui:emu.replyPayload]] | [模板](#templates)。主题中不能含有 `+` 或 `#`。 |
| [[ui:emu.qos]]、[[ui:emu.retain]] | 回复的设置。 |
| [[ui:emu.delay]]、[[ui:emu.jitter]] | 各为 0–60,000 ms。 |

MQTT 回复可以读取 `{{request.topic}}`、`{{request.levels[1]}}`（主题的各级，从 0 开始）、`{{request.payload}}`、`{{request.json.state}}`、`{{request.match}}`、`{{request.qos}}`、`{{request.retain}}`、`{{request.client}}`（客户端 ID）和 `{{request.from}}`。

## 回复中的模板 {#templates}

回复使用与实验相同的[模板语言](../experiments/data.md#templates)编写，因此一个字段在这里和在实验中含义相同。回复可以读取：

- `request`——收到的内容，见上文各协议的列表；
- `{{counter}}`——自模拟器启动以来此规则接收的消息数，包括本条；
- [生成器](../experiments/data.md#generators)——`{{uuid}}`、`{{now.iso}}`、随机值等；随机值从模拟器的种子中抽取；
- 参数，当模拟器在实验中运行，或用 `signallab emulate --param` 启动时。

回复从不读取机密；未知名称会报错，而不是得到空文本。

有些字段在模拟器启动时、任何内容到达之前就已确定：路径、条件、地址模式、载荷模式、主题过滤器、[[ui:emu.to]]、标头名称、保留消息以及代理的登录信息。它们只接受文本和参数，不接受 `request`，也不接受生成器。

种子决定响应的随机顺序、抖动和随机生成器。在 [[ui:nav.emulators]] 界面上，每次启动都会使用新的种子；实验使用本次运行的种子，`signallab emulate --seed` 则使用您给定的种子。

## 停机 {#outage}

要测试依赖时断时续时您的系统会怎样，请勾选 [[ui:emu.outage]]：

| 字段 | 内容 | 默认值 |
| --- | --- | --- |
| [[ui:emu.outageUp]] | 应答的时长，10–3,600,000 ms。 | 10,000 |
| [[ui:emu.outageDown]] | 停机的时长，10–3,600,000 ms。 | 3000 |
| [[ui:emu.outageFault]] | 仅限 HTTP：停机期间请求会遇到什么。 | [[ui:emu.outageFault.unavailable]] |

停机计划从模拟器启动时开始，并不断重复：在线、停机、在线、停机……停机期间：

| 模拟器 | 遇到的情况 |
| --- | --- |
| HTTP | [[ui:emu.outageFault.unavailable]]：503，`Retry-After` 设为距恢复还剩的秒数（至少为 1）。[[ui:emu.outageFault.reset]]：连接在没有应答的情况下关闭。[[ui:emu.outageFault.timeout]]：最多挂起 2 分钟，然后关闭。 |
| TCP 设备 | 已打开的连接在 0.1 s 内断开；新连接一到达就被关闭。 |
| MQTT 代理 | 所有连接都会断开；新连接会被拒绝（CONNACK 返回码 3，服务器不可用）。 |
| OSC、UDP 设备 | 不作任何应答。 |

停机期间到达的内容计为 [[ui:emu.down]]，而不是 [[ui:emu.unmatched]]，也不会交给规则处理。

[[ui:emu.takeDown]] 可以随时手动实现同样的效果，无论计划如何，直到您按下 [[ui:emu.bringUp]]；此时 HTTP 得到的 503 不带 `Retry-After`。在实验中，[[ui:exp.node.emulator_state]] 节点会在运行的某个步骤执行这一操作（参见[节点](../experiments/nodes.md#node-emulator_state)和[故障](../experiments/faults.md)）。

## 问题 {#problems}

编辑时，每次更改后片刻就会检查模拟器；在您按 [[ui:emu.start]] 之前，问题就会显示在按钮下方。问题会指出所在位置（规则、响应或保留消息，以及字段）和出错的内容：缺少 `/` 的路径、无法编译的正则表达式、引用了 `request`、参数和生成器以外内容的回复模板、超出范围的值。[[ui:emu.start]] 会拒绝启动存在问题的模拟器。

## 限制 {#limits}

| 项目 | 限制 | 达到限制时 |
| --- | --- | --- |
| 每个模拟器的路由或规则 | 64 | 检查时被拒绝。 |
| 每个路由的响应 | 16 | 被拒绝。 |
| 每个路由的条件 | 16 | 被拒绝。 |
| 每个响应的标头 | 32 | 被拒绝。 |
| 参数条件、回复参数（OSC） | 各 16 | 被拒绝。 |
| 保留消息（MQTT） | 64 | 被拒绝。 |
| 按书写的内容计算的正文、回复或问候语 | 256 KiB | 被拒绝。 |
| 延时或抖动 | 60,000 ms | 被拒绝。 |
| HTTP 请求正文 | 1 MiB | 413。 |
| 同时存在的 HTTP 连接 | 512 | 超出的连接一到达就被关闭。 |
| HTTP 请求头 | 30 s | 客户端必须在此时间内发送完。 |
| 同时存在的 TCP 连接 | 256 | 超出的连接一到达就被关闭。 |
| 等待延时的 OSC 和 UDP 回复 | 1024 | 超出的回复会被丢弃，并计为 [[ui:emu.failed]]。 |
| 同时连接的 MQTT 客户端 | 256 | 超出的客户端一到达就被关闭。 |
| MQTT 数据包 | 256 KiB | 该客户端的连接结束。 |
| 每个客户端的 MQTT 订阅 | 100 | 超出的订阅会被拒绝。 |
| MQTT 保留主题 | 1000 个主题，16 MiB | 新的保留消息会被路由，但不会被保留。 |
| 为一个慢速客户端排队等待的 MQTT 消息 | 1024 条消息，8 MiB | 该客户端会错过它们；计为 [[ui:emu.missed]]。 |

## 从响应创建模拟器 {#mock-this}

要根据一个成功的响应创建模拟器：

1. 在 [[ui:nav.http]] 界面上，发送请求并得到响应——或在实验的 HTTP 节点上使用 [[ui:exp.sendNow]]。
2. 按响应旁边的 ⧉ [[ui:http.mockThis]]。[[ui:http.mockTitle]] 对话框会显示它将创建的路由。
3. 在 [[ui:http.mockInto]] 中，选择您的某个 HTTP 模拟器，或选择 [[ui:http.mockNew]]。
4. 按 [[ui:http.mockAdd]]。[[ui:nav.emulators]] 界面会打开并定位到该模拟器。

该路由以响应的状态、标头和正文来应答该请求的方法和路径（不含查询字符串）。只属于那一次交互的标头（`Content-Length`、`Date`、`Server`、`ETag` 等）会被去掉，正文按原样发送，即使其中含有 `{{`。新模拟器只包含这一个路由。添加到现有模拟器时，该路由会放在最前面，以便在更宽泛的路由之前应答；正在运行的模拟器会在您按 [[ui:emu.restart]] 后采用它。

从实验创建时，用模板写的 URL 会变成模式：其基础部分（`{{api}}`）会被去掉，整段为一个模板的路径段（`/orders/{{order_id}}`）会变成 `:order_id`，只有部分使用模板的路径段则会以 `*` 结束路径。

## 初始模拟器 {#starter-set}

Signal Lab 第一次找不到模拟器库时，会写入五个模拟器，都在本机上。它们的名称和备注以当时的界面语言写成。

| 模拟器 | 监听于 | 作用 |
| --- | --- | --- |
| [[ui:seed.emu.demo-api.name]] | `127.0.0.1:8080` | `GET /health` → `{"status":"ok","time":…}`；`GET /users/:id` → 该 id 的用户；`POST /users` → 201，带 `Location`；`GET /slow` → 1500 ms 后应答；`/flaky` → 503、503，此后一直是 200。 |
| [[ui:seed.emu.osc-device.name]] | `127.0.0.1:9100` | `/ping` → `/pong`，附带计数；`/fader/*` → `/ack`，附带收到的地址；`/cue/*` 接收但不回复。 |
| [[ui:seed.emu.udp-device.name]] | `127.0.0.1:7100` | `PING` → `PONG` 和计数；其他任何内容 → `ACK` 及其字节大小。 |
| [[ui:seed.emu.tcp-device.name]] | `127.0.0.1:7200` | 以 CR LF 结尾的行。以 `READY` 问候；`POWER?` → `POWER=ON`；`POWER ON` 或 `POWER OFF` → `OK ON` / `OK OFF`；`QUIT` → `BYE`，然后挂断。 |
| [[ui:seed.emu.mqtt-broker.name]] | `127.0.0.1:1883` | 在 `lab/status` 上保留 `online`；发布到 `lab/<name>/set` 的 `ON` 或 `OFF` → 在 `lab/<name>/state` 上以保留消息的形式发布相同内容。 |

[信号库](signals.md#starter-set)中的初始信号 [[ui:seed.http-reachable.name]] 会请求 `http://127.0.0.1:8080/`，即 [[ui:seed.emu.demo-api.name]] 的地址：它没有 `/` 的路由，因此会得到 404。

## 库文件 {#file}

库是数据文件夹中的 `emulators.json`（参见[文件](../reference/files.md)）；将指针悬停在列表下方的计数上可以看到其路径。它会在最后一次更改 0.7 s 后通过临时文件整体写入，因此写入失败时会保留上一个版本。如果文件无法读取，列表会显示错误以及路径、行和列，文件保持原样：修复它，然后按 [[ui:emu.reload]]。手动编辑文件后，也请按 [[ui:emu.reload]]。没有文件时，会重新写入初始模拟器。

```json
{
  "version": 1,
  "emulators": [
    {
      "id": "orders-api",
      "note": "Stands in for the orders service.",
      "emulator": {
        "name": "Orders API",
        "bind": "127.0.0.1:18080",
        "protocol": "http",
        "routes": [
          { "method": "GET", "path": "/orders/:id",
            "responses": [{ "body": "{\"id\":\"{{request.params.id}}\",\"state\":\"open\"}" }] },
          { "method": "POST", "path": "/orders", "order": "sequence",
            "responses": [{ "status": 503 }, { "status": 201, "body": "{\"id\":\"{{uuid}}\"}" }] }
        ],
        "outage": { "up_ms": 20000, "down_ms": 2000, "fault": "unavailable" }
      }
    }
  ]
}
```

`emulator` 对象本身就是一个文档，`signallab emulate` 也能读取它。

## 在实验和脚本中 {#elsewhere}

- 在实验中，[[ui:exp.node.emulator]] 节点会在第一个步骤之前打开其模拟器，并一直应答到运行结束；它收到的内容会统计在报告中。其中的 HTTP 模拟器也是 [[ui:exp.node.wait_http]]（[节点](../experiments/nodes.md#node-wait_http)）所监听的对象，OSC 或 UDP 模拟器则与本次运行的等待节点共用端口。同一实验中同一传输协议的两个模拟器不能共用一个端口。参见[节点](../experiments/nodes.md#node-emulator)和[故障](../experiments/faults.md)。
- `signallab emulate` 会从文件或此库运行模拟器，直到按下 <kbd>Ctrl</kbd>+<kbd>C</kbd> 或达到 `--for` 的时长，并输出它们的应答；参见[命令行](../automation/cli.md#cli-emulate)。

## 相关内容 {#related}

- [检查器](inspector.md)——每次交互，均已解码。
- [网络损伤](impairment.md)——在您的系统与模拟器之间制造糟糕的网络。
- [数据与模板](../experiments/data.md#templates)

---
title: 事件
description: Signal Lab 服务器上引擎事件的 WebSocket，以及每个频道及其载荷和发送时机。
---

# 事件

任务运行期间发生的一切——运行的步骤、监视器的消息、突发的数字、检查器的帧、任务的结束——都会作为事件发送。在浏览器中，服务器页面通过一个 WebSocket（`/api/events`）接收它们；脚本可以监听同一个套接字。桌面应用在应用内部收到同样的事件，名称和载荷都相同。

## 订阅 {#subscribe}

在服务器上打开一个指向 `/api/events` 的 WebSocket：

```bash
websocat -H "Authorization: Bearer $TOKEN" ws://127.0.0.1:1430/api/events
```

- **认证**与 API 的其余部分相同：令牌为 `Authorization: Bearer`，或浏览器的会话 Cookie。没有它，升级会被拒绝，返回 `401` `auth.required`。
- **来源**：发送 `Origin` 标头的客户端必须发送服务器自己的来源（主机名和端口与 `Host` 相同），否则升级会被拒绝，返回 `403` `auth.origin`。浏览器之外的大多数 WebSocket 库都不发送它。
- **每个事件都发给每个客户端。** 没有什么需要订阅：每个套接字都会收到每个任务的每个事件，无论任务是谁启动的。请用 `event` 以及载荷中的 `job_id` 挑选您需要的内容。
- **只监听。** 服务器会忽略客户端发送的内容，关闭帧除外；大于 64 KiB 的消息会关闭套接字。
- **保活。** 服务器每 20 s ping 一次，因此安静的套接字也能在代理下保持打开。服务器停止时，会关闭每个套接字。
- **不会重放。** 客户端未连接期间发送的事件对它来说就已丢失。重新连接的客户端应通过命令（`jobs_list`、`inspect_snapshot`、`emulator_exchanges`……）读取当前状态。
- **落后。** 最多有 4096 个事件等待一个套接字。落后更多的客户端会收到 [`server://lagged`](#event-server-lagged)，并附上错过了多少个。

## 消息格式 {#format}

每个事件都是一条包含一个 JSON 对象的文本消息：

```json
{ "event": "scan://open", "payload": { "job_id": 9, "ts": 1759600000123, "port": 8080, "banner": null } }
```

| 字段 | 内容 |
| --- | --- |
| `event` | 频道，见下文 |
| `payload` | 事件的值；其形态取决于频道 |

时间（`ts`、对端的 `first_ms` 和 `last_ms`）是自 1970 年起的毫秒数；延迟和其他时长（`*_latency_ms`、`p50_ms`……、`ms`）单位为毫秒。载荷中的错误是 [`EngineError`](index.md#errors) 对象；它们的代码列在[错误消息](../reference/errors.md)中。

## 频道 {#channels}

| 频道 | 发送方 | 时机 |
| --- | --- | --- |
| [`experiment://step`](#event-experiment-step) | 一次运行 | 一个步骤开始、通过、失败、重试、重复或报告负载时 |
| [`experiment://ended`](#event-experiment-ended) | 一次运行 | 运行自行结束时发送一次 |
| [`job://ended`](#event-job-ended) | 每个任务 | 任务自行结束或失败时发送一次 |
| [`osc://message`](#event-osc-message) | OSC 监视器 | 每个数据包 |
| [`osc://gen-tick`](#event-osc-gen-tick) | OSC 生成器 | 每条消息；超过每秒 60 条消息时每秒 30 到 45 次 |
| [`http://burst-progress`](#event-http-burst-progress) | HTTP 突发 | 每 100 ms，以及结束时 |
| [`ws://state`](#event-ws-state) | WebSocket 连接 | 已连接、已关闭 |
| [`ws://messages`](#event-ws-messages) | WebSocket 连接 | 每 100 ms，有新的内容时 |
| [`mqtt://state`](#event-mqtt-state) | MQTT 连接 | 已连接、已订阅、已关闭 |
| [`mqtt://messages`](#event-mqtt-messages) | MQTT 连接 | 每 100 ms，有新的内容时 |
| [`mqtt://ack`](#event-mqtt-ack) | MQTT 连接 | 一次 QoS 1/2 发布完成；一次取消订阅得到应答 |
| [`broadcast://emit-stat`](#event-broadcast-emit-stat) | 信标 | 每 250 ms，以及结束时 |
| [`broadcast://peers`](#event-broadcast-peers) | 发现监听器 | 每 400 ms |
| [`netsim://stat`](#event-netsim-stat) | 损伤中继 | 每 250 ms |
| [`storm://stat`](#event-storm-stat) | 风暴 | 每 250 ms，以及结束时 |
| [`scan://open`](#event-scan-open) | 扫描器 | 每个打开的端口 |
| [`scan://progress`](#event-scan-progress) | 扫描器 | 大约每 1% 的范围，以及结束时 |
| [`emulator://activity`](#event-emulator-activity) | 模拟器任务 | 每 200 ms，有新的内容时 |
| [`inspect://batch`](#event-inspect-batch) | 检查器 | 捕获开启时，每 120 ms 有新帧时；安静时约每秒一次 |
| [`server://lagged`](#event-server-lagged) | 服务器 | 一个客户端落后了 |

### `experiment://step` {#event-experiment-step}

一次运行的一个步骤：节点开始、通过、失败、等待再次尝试、重复，或报告负载的进度。用 `/api/run` 启动的运行会在其响应中发送相同的步骤（参见[运行](run.md)）。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 该运行的任务 |
| `ts` | number | 时间 |
| `node_id` | string | 节点 |
| `state` | string | `running`、`passed`、`failed`、`retry`（一次尝试失败，步骤在暂停后再次运行）、`repeating`（重复动作的进度，每秒至多一次）或 `load`（负载的进度，每秒至多一次） |
| `detail` | string | 发生了什么，为英文；`running` 和 `failed` 时为空（参见 `error`） |
| `message_key` | string or null | 界面对应的文本，作为其字典中的一个键 |
| `message_params` | object or null | `message_key` 所提到的值 |
| `vars` | object | 步骤写入的变量；没有时省略 |
| `error` | `EngineError` | 它为何失败，或该次尝试为何失败（`retry`）；否则省略 |
| `frame` | number | 捕获开启时，某个等待（或发送的预期回复）所匹配消息的检查器帧号；否则省略 |
| `load` | object | 一次负载测得的内容及读取到的阈值——在负载步骤的最后一个事件上，无论通过还是失败；否则省略。参见[负载](../experiments/load.md) |

运行的 [[ui:exp.node.end]] 节点在第一个分支到达时显示 `running`，并在每个分支都无失败地完成后显示 `passed`。每个字段中的机密值都会被遮蔽。

### `experiment://ended` {#event-experiment-ended}

一次运行自行结束：它通过、失败或超时。紧跟在 [`job://ended`](#event-job-ended) 的相同载荷之后发送。用 `job_stop` 或 [[ui:app.stopAll]] 停止的运行二者都不发送，也不保存报告。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 该运行的任务 |
| `kind` | string | `experiment` |
| `seed` | number | 它运行所用的种子 |
| `profile` | string or null | 它的配置 |
| `overridden` | boolean | 部分参数值来自 [[ui:exp.runWith]] 或 `overrides` |
| `error` | `EngineError` or null | 运行的第一个失败；通过时为 null |
| `report_path` | string or null | 它的报告，位于数据文件夹的 `runs/` 中 |
| `report_error` | `EngineError` or null | 报告为何无法写入 |

### `job://ended` {#event-job-ended}

任务自行结束或失败。用 `job_stop` 或 `jobs_stop_all` 停止的任务不会发送它。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 任务 |
| `kind` | string | `osc-monitor`、`osc-gen`、`http-burst`、`netsim`、`storm`、`scan`、`beacon`、`discovery`、`mqtt`、`websocket`、`emulator` 或 `experiment` |
| `error` | `EngineError` or null | 出问题时它为何结束 |

运行的 `job://ended` 也携带 [`experiment://ended`](#event-experiment-ended) 的字段。每种任务的结束原因：

| `kind` | 结束于 | `error` |
| --- | --- | --- |
| `osc-monitor` | 套接字无法再接收 | `wait.receive_failed` |
| `osc-gen` | 其时长结束，或一次发送失败 | null，或 `transport.*` |
| `http-burst` | 达到其总数或时长 | null |
| `storm` | 其时长结束 | null |
| `scan` | 范围内的每个端口都已尝试 | null |
| `beacon` | 其轮数或时长结束，或超过 32 次发送失败且一次都没发出 | null，或 `transport.*` |
| `discovery` | 套接字无法再接收 | `wait.receive_failed` |
| `mqtt` | 代理关闭了连接，或连接丢失 | `transport.*`（代理关闭时为 `transport.reset`），或 `mqtt.protocol` |
| `websocket` | 连接已关闭 | null，或它为何丢失 |
| `netsim` | 中继无法再工作 | 原因 |
| `emulator` | 其套接字失败 | 原因 |
| `experiment` | 运行结束 | 运行的失败，或 null |

### `osc://message` {#event-osc-message}

OSC 监视器收到并解码的一个 UDP 数据包。每个数据包都发送，不批量。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 监视器的任务 |
| `ts` | number | 它到达的时间 |
| `from` | string | 发送方，`IP:port` |
| `bytes` | number | 数据包的大小 |
| `messages` | object[] | 数据包中的每条消息（一个包内消息组会有多条）：`address` 和 `args`（[`OscArg`](commands.md#type-oscarg)`[]`） |
| `error` | `EngineError` or null | 数据包未解码时为 `osc.packet_malformed`（此时 `messages` 为空） |

### `osc://gen-tick` {#event-osc-gen-tick}

OSC 生成器的进度：低于每秒 60 条消息时每条消息都发送；高于此值时每第 n 条发送一次，n 为速率除以 30 并向下取整——即每秒 30 到 45 次。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 生成器的任务 |
| `ts` | number | 时间 |
| `value` | number | 刚发送的值，在舍入为整数或 32 位浮点数之前 |
| `sent` | number | 到目前为止已发送的消息数 |

### `http://burst-progress` {#event-http-burst-progress}

HTTP 突发的数字，运行时每 100 ms 一次，并在它自行结束时再发送一次，带 `done: true`。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 突发的任务 |
| `ts` | number | 时间 |
| `sent` | number | 到目前为止已应答或失败的请求 |
| `ok` | number | 其中以 2xx 状态应答的 |
| `failed` | number | 其中其他状态或没有应答的 |
| `missed` | number | 有节奏的突发中等待空闲工作器过久而跳过的请求 |
| `rps` | number | 最近 100 ms 内的每秒请求数；在最后一个事件中，为整个突发的 |
| `last_latency_ms`, `min_latency_ms`, `max_latency_ms`, `avg_latency_ms` | number | 到目前为止的延迟 |
| `p50_ms`, `p90_ms`, `p95_ms`, `p99_ms` | number | 到目前为止每个请求的百分位数，含失败，误差在 0.5% 以内 |
| `done` | boolean | 突发的最后一个事件 |

### `ws://state` {#event-ws-state}

由 `ws_connect` 打开的 WebSocket 连接已连接或已关闭。任务被停止的连接不发送 `closed`。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 连接的任务 |
| `ts` | number | 时间 |
| `state` | string | `connected` 或 `closed` |
| `handshake` | object | `url`、`peer`、`local`、`protocol`（服务器选择的子协议，或 null）以及 `ms`（连接和升级） |
| `closed` | object or null | 带 `closed` 时：`code`、`reason`、`by`（`client`、`server` 或 `lost`）以及 `error` |

### `ws://messages` {#event-ws-messages}

自上一个事件以来一个 WebSocket 连接发送和接收的内容，每 100 ms 一次，有内容时。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 连接的任务 |
| `ts` | number | 时间 |
| `messages` | object[] | 按顺序：`ts`、`dir`（`rx` 接收，`tx` 发送）、`kind`（`text` 或 `binary`）、`text`（前 64 KiB 作为 UTF-8，二进制消息也一样；不是 UTF-8 的字节会变成 `�`）、`hex`（二进制消息前 4096 字节的十六进制，否则为 null）、`bytes`（完整大小）以及 `truncated`（超过显示的部分：文本超过 64 KiB，二进制超过 4096 字节） |
| `dropped` | number | 因为超过 2000 条而没有被包含进此事件的消息；最旧的先被丢弃 |

### `mqtt://state` {#event-mqtt-state}

MQTT 连接的状态发生变化。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 连接的任务 |
| `ts` | number | 时间 |
| `state` | string | `connected`；每次订阅得到应答后为 `subscribed`；连接结束时为 `closed`（其任务被停止时不是） |
| `broker` | string | `host:port` |
| `error` | `EngineError` or null | 一个 `closed` 连接为何结束（代理关闭它时为 `transport.reset`）；否则为 null |
| `grants` | object[] | 带 `subscribed` 时：每个请求的过滤器，含 `filter`、`qos`（授予的）和 `accepted`；否则为空 |

### `mqtt://messages` {#event-mqtt-messages}

自上一个事件以来一个 MQTT 连接接收到的内容，每 100 ms 一次，有内容时。重新投递的 QoS 2 消息只显示一次。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 连接的任务 |
| `ts` | number | 时间 |
| `messages` | object[] | `ts`、`topic`、`payload`（作为 UTF-8；不是 UTF-8 的字节会变成 `�`）、`bytes`、`qos`、`retain`、`dup` |
| `dropped` | number | 因为 100 ms 内到达超过 4000 条而略去的消息；最旧的先被丢弃 |

### `mqtt://ack` {#event-mqtt-ack}

代理完成了该连接请求的某件事。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 连接的任务 |
| `ts` | number | 时间 |
| `kind` | string | `published`（一次 QoS 1 或 2 发布已完成）或 `unsubscribed` |
| `packet_id` | number | MQTT 数据包 id |
| `topic` | string or null | 发布的主题；`unsubscribed` 时为 null |

### `broadcast://emit-stat` {#event-broadcast-emit-stat}

信标的计数器，每 250 ms 一次，并在它自行结束时再发送一次，`pps` 为 0。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 信标的任务 |
| `ts` | number | 时间 |
| `targets` | number | 每轮的目标数 |
| `rounds` | number | 已发送的轮数 |
| `packets`, `bytes` | number | 已发送的数据报和字节数 |
| `errors` | number | 失败的发送 |
| `pps` | number | 最近 250 ms 内的每秒数据报数 |

### `broadcast://peers` {#event-broadcast-peers}

发现监听器听到的内容，每 400 ms 一次。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 监听器的任务 |
| `ts` | number | 时间 |
| `peers` | object[] | 最近听到的在最前：`addr`、`proto`、`packets`、`bytes`、`first_ms`、`last_ms`、`last_summary`、`responded`（其数据包中得到应答的，按到达时计数）；最多 512 个 |
| `packets`, `bytes` | number | 收到的一切 |
| `responses` | number | 已发送的应答 |

### `netsim://stat` {#event-netsim-stat}

损伤中继的计数器，每 250 ms 一次。运行中 [[ui:exp.node.impairment]] 节点的中继改为在运行的报告中报告。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 中继的任务 |
| `ts` | number | 时间 |
| `received`, `forwarded` | number | 进出的数据报或数据块 |
| `dropped` | number | 因 `loss`、突发或 `offline` 丢失的（UDP；TCP 中继在离线期间保持流，不丢弃任何内容） |
| `throttled` | number | UDP：被带宽限制丢弃，或因为已有太多在途。TCP：为带宽限制而拖住其流的数据块 |
| `duplicated`, `corrupted`, `reordered` | number | 配置对它们做了什么 |
| `bytes` | number | 转发出去的字节数 |
| `connections`, `reset`, `stalled` | number | TCP：已接受的连接、已重置的、处于半开状态的；为 0 时省略 |
| `profile` | string | 它现在用来制造损伤的配置，按时间线的称法：其名称，或它所做的事（`60 ms ±25 · loss 2%`） |

### `storm://stat` {#event-storm-stat}

风暴的计数器，每 250 ms 一次，并在它自行结束时再发送一次，`pps` 和 `mbps` 为 0。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 风暴的任务 |
| `ts` | number | 时间 |
| `packets`, `bytes` | number | 已发送的数据报（或 TCP 连接）和字节数 |
| `errors` | number | 失败的发送或连接 |
| `pps` | number | 最近 250 ms 内的每秒数 |
| `mbps` | number | 最近 250 ms 内的每秒兆比特数 |

### `scan://open` {#event-scan-open}

扫描器发现了一个打开的端口。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 扫描的任务 |
| `ts` | number | 时间 |
| `port` | number | 端口 |
| `banner` | string or null | 服务最先发送的内容，前提是要求了 banner 且它在 400 ms 内说了些什么 |

### `scan://progress` {#event-scan-progress}

扫描的进度：大约每 1% 的范围，以及它自行结束、`done` 等于 `total` 时（最后这个可能到达两次）。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 扫描的任务 |
| `ts` | number | 时间 |
| `done` | number | 已尝试的端口 |
| `total` | number | 范围内的端口数 |
| `open` | number | 发现的打开端口数 |

### `emulator://activity` {#event-emulator-activity}

用 `emulator_start` 启动的模拟器自上一个事件以来接收和应答的内容，每 200 ms 一次，有变化时（一次交互、被停机或恢复，或 MQTT 代理无法投递的消息）。运行中的 [[ui:exp.node.emulator]] 节点不发送它；它们的计数器在运行的报告中。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 模拟器的任务 |
| `ts` | number | 时间 |
| `counts` | object | `total`、`unmatched`、`failed`、`down`、`hits`（每条规则）以及 `missed`（MQTT；为 0 时省略）——如 [`emulator_exchanges`](commands.md#emulator_exchanges) 所示 |
| `forced` | string | 被停机期间为 `unavailable`、`reset` 或 `timeout`；否则省略 |
| `exchanges` | object[] | 新的交互，如 `emulator_exchanges` 所列，但没有 `data`；最多 200 条 |
| `dropped` | number | 该时间段内超过前 200 条、未在此发送的交互；`emulator_exchanges` 仍保留最近 500 条 |

### `inspect://batch` {#event-inspect-batch}

新的检查器帧。仅在捕获开启时发送：有新帧时每 120 ms 一次，没有时约每秒一次，以便计数器保持最新。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `frames` | object[] | 新的[帧](commands.md#type-frame)，最旧的在前，最多 250 个；不含其字节（请用 `inspect_payload`） |
| `stats` | object | 捕获的计数器，[`CaptureStats`](commands.md#type-frame) |
| `skipped_now` | number | 自上一批以来捕获但不在此批中的帧——到达超过 250 个，或缓冲区放走了它们。只要缓冲区还保留它们，它们就仍在导出中 |

### `server://lagged` {#event-server-lagged}

仅服务器。此客户端落后了超过 4096 个事件，错过了一些。请用命令重新读取状态。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `skipped` | number | 它错过了多少个事件 |

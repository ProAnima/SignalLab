---
title: MQTT
description: 连接到 MQTT 3.1.1 代理，把它保存的每个主题显示为实时树，以 QoS 0、1 或 2 发布，公布遗嘱，并清除卡住的保留值。
---

# MQTT

[[ui:nav.mqtt]] 界面是一个 MQTT 客户端，用于查看代理并更改其中的内容。连接后，默认会订阅 `#`：代理保存的每个主题都会连同其最新值构成一棵树。在此之上，您可以发布、清除保留值、把主题保存为信号，或把它变成实验中的一个步骤。

Signal Lab 使用**明文 TCP 上的 MQTT 3.1.1**，订阅、发布和遗嘱都支持 QoS 0、1 和 2。不支持 MQTT 5，也不支持 TLS：只接受 `mqtts://` 或 MQTT 5 客户端的代理无法连接。

## 连接 {#connect}

1. 打开 [[ui:nav.mqtt]]。
2. 填写 [[ui:mq.host]] 和 [[ui:common.port]]。
3. 除非代理要求特定的客户端 ID，否则保持 [[ui:mq.clientId]] 不变。仅在代理要求时添加 [[ui:mq.username]] 和 [[ui:mq.password]]。
4. 按 [[ui:mq.connect]]。

连接会先打开 TCP 连接并完成 MQTT 握手，然后才做其他事，因此密码错误或端口关闭会当场报告。连接期间连接字段会锁定；[[ui:mq.disconnect]] 会关闭它。该连接是控制台任务条中的一个任务，也可以在那里停止。

| 字段 | 内容 | 默认值 |
| --- | --- | --- |
| [[ui:mq.host]] | 代理的 IP 地址或主机名 | `127.0.0.1` |
| [[ui:common.port]] | 代理的端口 | `1883` |
| [[ui:mq.clientId]] | 您的客户端在代理处的名称。不能为空，且在那里必须唯一：第二个使用相同 id 的客户端会把第一个挤下线。 | `signal-lab-` 加六个随机十六进制数字，每次启动应用都会更新 |
| [[ui:mq.username]]、[[ui:mq.password]] | 仅在代理需要时发送——由于没有 TLS，以明文发送。只有密码而没有用户名时根本不会发送：MQTT 3.1.1 无法单独携带密码。 | 空 |
| [[ui:mq.keepAlive]] | 连接可以保持静默的秒数。Signal Lab 每隔其一半时间向代理发送一次 ping；客户端静默达到该值的 1.5 倍时，代理会将其断开。0 表示关闭 ping。 | `60` |
| [[ui:mq.cleanSession]] | 开启：每次连接都不带已存储的订阅和已排队的消息。关闭则会请求代理为这个客户端 ID 在两次连接之间保留它们。 | 开启 |
| [[ui:mq.scanFilter]] 及其 [[ui:mq.qos]] | 连接一建立就订阅的过滤器；`#` 表示所有主题。留空：不订阅。 | `#`，QoS 0 |
| [[ui:mq.willEnable]] | 向代理提供一份遗嘱（[见下文](#will)） | 关闭 |

代理有 6 秒时间来接受 TCP 连接，另有 6 秒来应答握手。

### 遗嘱 {#will}

遗嘱是代理为您保存的消息，如果您的连接在没有正常告别的情况下断开，代理会自行发布它。在线状态通常就是这样实现的：设备向自己的状态主题发布 `online`，而它的遗嘱把同一主题设为 `off`。

勾选 [[ui:mq.willEnable]] 后，设置 [[ui:mq.willTopic]] 和 [[ui:mq.willPayload]]（默认为 `off`）。遗嘱以 QoS 2 发布，并保留。没有主题时，不发送遗嘱。

## 订阅 {#subscribe}

扫描过滤器会在连接时订阅。要订阅更多：

1. 在 [[ui:mq.addSubscription]] 中，输入一个主题过滤器。
2. 选择它的 [[ui:mq.qos]]。
3. 按 [[ui:mq.subscribe]] 或 <kbd>Enter</kbd>。

过滤器是带通配符的主题：

| 通配符 | 表示 | 示例 |
| --- | --- | --- |
| `+` | 恰好一级 | `sensors/+/state` 匹配 `sensors/door/state` |
| `#` | 其下所有层级，只能是最后一个字符 | `sensors/#` 匹配 `sensors/door/state` 和 `sensors` |

[[ui:mq.subscriptions]] 列出每个过滤器以及代理授予的结果：`qos0`、`qos1` 或 `qos2`——代理授予的等级可能低于您请求的——或 [[ui:mq.refused]]。[[ui:mq.unsubscribe]] 取消订阅其中一个。

| QoS | 投递 |
| --- | --- |
| 0 | 至多一次：发送后即遗忘 |
| 1 | 至少一次：会得到确认，可能到达两次 |
| 2 | 恰好一次：两步握手；重投不会显示两次 |

## 主题树 {#topics}

到达的每条消息都会进入 [[ui:mq.topics]]，即由主题各层级构成的树。主题会显示其最新值、值被保留时显示 **R**，以及收到多条消息时的消息条数。点击某一层即可展开或折叠它。

- 在上方的字段中输入内容，即可只列出路径或最新值包含该文本的主题。
- 树的上方显示主题数量、其中有多少保留了值，以及连接期间您正在监听的代理。
- 载荷以文本显示；不是 UTF-8 的字节显示为替换字符。
- [[ui:common.clear]] 会清空这棵树。没有其他操作会清空它：切换界面或断开连接时它都保持原样，直到应用关闭。

消息以每秒十批的方式到达界面。当代理在十分之一秒内发送超过 4000 条消息时，该批中最早的那些不会进入树，并在树的上方计为“未显示”。

### 主题面板 {#topic}

选择一个带值的主题，即可在树的下方看到它：[[ui:mq.value]]、[[ui:mq.qos]]、[[ui:mq.retain]]、[[ui:common.bytes]]、[[ui:mq.messages]] 和 [[ui:mq.lastAt]]。它的按钮：

| 按钮 | 作用 |
| --- | --- |
| [[ui:mq.editHere]] | 把主题、值、QoS 和保留标志复制到 [[ui:mq.publish]] |
| [[ui:mq.waitForThis]] | 向打开的实验添加一个[等待 MQTT](../experiments/nodes.md#node-wait_mqtt)步骤：主题为此主题，代理为此代理，任意载荷，2000 ms 超时 |
| [[ui:mq.clearRetained]] | 移除保留值（[见下文](#clear-retained)） |
| [[ui:sig.fromFrame]] | 把该主题及其最新值作为信号保存到 [[ui:sig.capturedFolder]] 文件夹 |

## 发布 {#publish}

1. 连接。
2. 在 [[ui:mq.publish]] 下，输入 [[ui:mq.topic]] 和 [[ui:sig.payload]]。
3. 选择 [[ui:mq.qos]]，如果代理应把这条消息作为该主题的值、提供给之后订阅的每个客户端，请勾选 [[ui:mq.retain]]。
4. 按 [[ui:mq.publishBtn]]。

控制台会确认每次发布：QoS 0 立即确认，QoS 1 和 2 则在代理确认后确认。用于发布的主题不含通配符，也不能为空：含 `+` 或 `#` 的主题会在发送任何内容之前被拒绝，并给出与信号、步骤和 `signallab send mqtt` 相同的消息，连接保持原样。只有订阅接受带通配符的过滤器。

### 清除保留值 {#clear-retained}

保留值会一直留在代理上，直到被替换；每个订阅的客户端都会先收到它——过期的保留值是设备启动后进入错误状态的典型原因。移除它的唯一方式是发布一个带保留标志的空载荷。

主题面板中的 [[ui:mq.clearRetained]] 会这样做：按下它，然后按 [[ui:mq.clearConfirm]]。它会在您的连接上以 QoS 1 发布空的保留载荷。仅在已连接且该主题的最新值被保留时可用。您也可以手动完成：[[ui:sig.payload]] 留空，并勾选 [[ui:mq.retain]]。

::: warning
清除会同时改变所有客户端看到的代理内容。
:::

## 在检查器中 {#inspector}

捕获开启时，MQTT 流量会以协议 `mqtt` 出现：

| 来源 | 内容 | 数量 |
| --- | --- | --- |
| `mqtt` | 界面连接发布的内容；空的保留发布判定为 `clears retained` | 每一条 |
| `mqtt` | 连接收到的消息 | 每 200 ms 至多一条 |
| `mqtt-send` | 自带连接的发布：界面未连接到信号所属代理时触发的信号、步骤、`signallab send mqtt`（判定 `one-shot`） | 每一条 |
| `experiment-wait` | [[ui:exp.node.wait_mqtt]] 步骤的订阅收到的消息，不含重放的保留值 | 每一条 |

摘要读作 `topic = payload`，适用时还带有 QoS 和 `retained`。参见[检查器](../tools/inspector.md)。

## 保存与复用 {#library}

- **保存为信号。** [[ui:mq.publish]] 下的 [[ui:sig.saveNew]] 会把代理（连接的 [[ui:mq.host]] 和 [[ui:common.port]]）、主题、载荷、QoS 和保留标志保存到信号库；在发布面板中按 <kbd>Ctrl</kbd>+<kbd>S</kbd> 效果相同，并在面板与该信号关联后更新它。参见[信号](../tools/signals.md)。
- **触发 MQTT 信号。** 当此界面连接到信号所指的代理时（主机相同，不区分大小写，端口也相同；信号未给出端口时为 `1883`），从库中触发的信号会经由该连接发出，并使用其客户端 ID 和凭据。否则——未连接，或连接到其他代理——它会向自己的代理打开一个属于自己的连接——全新的客户端 ID，没有用户名——发布，等待其 QoS 所要求的确认，然后断开。名称不做解析，因此 `localhost` 和 `127.0.0.1` 算作不同的代理。库中不保存密码。
- **在实验中。** 保存的 MQTT 信号可以在实验的 [[ui:exp.addNode]] 菜单中 [[ui:exp.group.signals]] 下列出，选中即可成为 [[ui:exp.node.mqtt]] 步骤。

## 在实验中 {#experiments}

| 步骤 | 作用 |
| --- | --- |
| [[ui:exp.node.mqtt]] | 连接、发布一条消息并断开——不使用用户名或密码，干净会话，15 秒内完成。[详情](../experiments/nodes.md#node-mqtt) |
| [[ui:exp.node.wait_mqtt]] | 运行开始时订阅，等待某个主题过滤器上载荷匹配的消息；订阅时重放的保留值会被忽略。[详情](../experiments/nodes.md#node-wait_mqtt) |
| [[ui:exp.node.emulator]] | 本次运行自己的 MQTT 代理。[详情](../experiments/nodes.md#node-emulator) |

这两个步骤都不登录，因此需要一个接受无用户名客户端的代理。

### 代理模拟器 {#broker-emulator}

Signal Lab 也可以充当代理：[[ui:emu.new.mqtt]] 模拟器把客户端发布的内容路由给已订阅者——3.1.1、明文 TCP、QoS 0、1 和 2、保留消息、遗嘱、可选登录——并像设备一样按规则应答。把您的设备和此界面指向它，即可在没有真实代理的情况下测试。参见[模拟器](../tools/emulators.md)。

## 从命令行 {#cli}

`signallab send mqtt` 用属于自己的连接发布一条消息：

```bash
signallab send mqtt 127.0.0.1:1883 lab/light/1/set on --qos 1
signallab send mqtt 127.0.0.1:1883 lab/light/1/state "" --retain
```

```text
✔ lab/light/1/set → 127.0.0.1:1883 · 2 B · qos1
```

第二行清除一个保留值。未给出端口时，代理位于 1883。它不使用凭据。代理接收了消息时退出码为 0，无法连接或拒绝时退出码为 1。参见[命令行](../automation/cli.md#cli-send-mqtt)。

## 问题 {#troubleshooting}

| 您看到的内容 | 常见原因 |
| --- | --- |
| `… refused the connection — nothing is listening on that port` | 该地址和端口上没有代理。 |
| `… accepted the connection but did not answer in time — is it an MQTT broker?` | 那里有程序在监听，但它不使用 MQTT，或者通过 TLS 使用。 |
| `… answered with something other than MQTT 3.1.1` | 不是 MQTT 代理，或代理发来了 Signal Lab 无法读取的内容。 |
| `… does not accept MQTT 3.1.1 clients` | 该代理只接受 MQTT 5。 |
| `… rejected the client ID — choose another one` | 该 ID 太长，或含有代理不接受的字符。 |
| `… rejected the username or password` | 凭据错误，或只有密码而没有用户名。 |
| `… did not authorize this client — check its access rules` | 代理的访问规则拒绝这个客户端。 |
| `… is unavailable right now — try again later` | 代理在线，但不接受客户端。 |
| `Enter a client ID — brokers refuse an empty one` | [[ui:mq.clientId]] 为空。 |
| `A publish topic cannot contain the wildcards + or #` | 要发布到的主题含有 `+` 或 `#`。它们用于订阅；发布时一次只能针对一个主题。 |
| 某个过滤器显示 [[ui:mq.refused]] | 代理的访问规则禁止它，或过滤器格式错误（`#` 不在最后，`+` 与其他字符共用一级）。 |
| 连接后不久就断开 | 另一个客户端使用了相同的 [[ui:mq.clientId]] 进行连接。 |
| 树中不出现任何内容 | 扫描过滤器为空，或代理不让这个客户端看到任何内容。 |

每条错误信息都列在[错误信息](../reference/errors.md#mqtt)中。

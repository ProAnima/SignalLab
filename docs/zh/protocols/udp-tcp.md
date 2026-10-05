---
title: UDP 与 TCP
description: Signal Lab 在哪里收发原始 UDP 数据报和 TCP 数据——实验步骤、信号、命令行、负载与扫描工具以及模拟设备——以及载荷如何书写。
---

# UDP 与 TCP

原始 UDP 和 TCP 没有自己的界面。当设备使用自己的文本或二进制协议时——投影机、媒体服务器、传感器——就会用到它们，它们出现在好几个地方：

| 要…… | 使用 |
| --- | --- |
| 把数据报或 TCP 消息作为步骤发送，并等待应答 | [实验步骤](#experiments) |
| 保留一个数据报以便再次发送，或重放您捕获到的一个 | [UDP 信号](#signals) |
| 从脚本发送一个数据报 | [`signallab send udp`](#cli) |
| 同时发送到多台主机、一个广播地址或一个组播组 | [广播](broadcast.md) 界面 |
| 用流量给服务器或链路加载 | [风暴](#storm) |
| 找出主机开放了哪些 TCP 端口 | [扫描器](#scanner) |
| 扮演设备那一端 | [UDP 或 TCP 设备](#emulators)模拟器 |
| 让两端之间的网络变差 | [损伤中继](#impairment) |

OSC 是一种承载在 UDP 数据报中的格式；它有自己的一页：[OSC](osc.md)。

## 载荷 {#payloads}

无论在哪里书写原始载荷，它都是两种之一：

| 种类 | 发送的内容 | 示例 |
| --- | --- | --- |
| 文本 | 这些字符以 UTF-8 发送，与输入完全一致：不添加终止符，也不添加行结束符。按行通信的协议需要在文本中写出其行结束符。 | `PING` |
| 十六进制字节 | 逐字节发送，写成成对的十六进制数字。数字对之间可以有空格、`:`、`-` 和 `,`，前面可以有 `0x`。 | `de ad be ef`、`DEADBEEF`、`0xde,0xad` |

一个数据报最多携带 65 507 字节。十六进制数字个数为奇数或没有数字，会在发送任何内容之前报错。

::: info
UDP 没有送达回执。“已发送”意味着数据报离开了本机，而不是有谁收到了它。要知道设备是否听到您，请等待它的应答。
:::

## 发往何处 {#destinations}

目标是 IP 地址加端口，或主机名加端口：`192.0.2.20:9000`、`[2001:db8::20]:9000`（IPv6 地址要放在方括号中）、`projector.local:9000`。这适用于 UDP 步骤的目标、OSC 目标、UDP 信号、`signallab send udp` 和 `signallab send osc`、[广播](broadcast.md)列表、[风暴](../tools/storm.md)的目标以及 TCP 步骤的主机。

每次使用主机名时都会解析它。当它有 IPv4 地址时，就使用该地址——因此 `localhost` 能到达监听在 `127.0.0.1` 上的服务，而系统为它列出的第一个地址可能是 `::1`——只有 IPv6 地址的主机名则通过 IPv6 套接字访问。无法解析的主机名会以 `Cannot resolve …` 失败；没有端口或两者都不是的目标会以 `… is not a valid address` 失败。服务*监听*的地址（监视器的、等待的、模拟器的）始终是 `IP:port`。

## 在实验中 {#experiments}

| 步骤 | 作用 |
| --- | --- |
| [[ui:exp.node.udp]] | 把它的 [[ui:exp.payload]] 作为文本发送到 [[ui:common.target]]。目标是 `IP:port` 或 `host:port`（[见上文](#destinations)），用逗号分隔的多个目标各自收到该数据报。开启 [[ui:exp.expectReply]] 时，它从 [[ui:exp.replyOn]] 发送，并在同一步骤中在那里等待应答。[详情](../experiments/nodes.md#node-udp) |
| [[ui:exp.node.wait_udp]] | 在 [[ui:exp.listenOn]]（`IP:port`）上监听，并等待一个载荷匹配的数据报。[详情](../experiments/nodes.md#node-wait_udp) |
| [[ui:exp.node.tcp]] | 连接到 [[ui:exp.host]] 和 [[ui:exp.port]]，把它的 [[ui:exp.payload]] 作为文本写入，监听 250 ms 等待应答，然后关闭。该步骤只说明返回了多少字节，而不说明它们是什么。[详情](../experiments/nodes.md#node-tcp) |

UDP 载荷和目标、TCP 主机和载荷，以及等待的模式都接受 `{{templates}}`，因此一个数据报可以携带本次运行的 id 或更早的步骤提取出的值。见[数据与模板](../experiments/data.md)。

### 匹配数据报 {#matching}

[[ui:exp.node.wait_udp]] 和 [[ui:exp.node.udp]] 的回复根据数据报的载荷来选择它：

| [[ui:exp.waitMode]] | 在以下情况接收数据报 |
| --- | --- |
| [[ui:exp.mode.any]] | 总是：第一个到达的 |
| [[ui:exp.mode.contains]] | 其载荷按文本读取后包含该模式 |
| [[ui:exp.mode.regex]] | 其载荷按文本读取后匹配该正则表达式 |
| [[ui:exp.mode.hex]] | 其字节包含该模式的字节（以十六进制书写） |

匹配到的内容保存在该步骤的变量中（[[ui:exp.replyVariable]]，除非您重命名，否则为 `reply`）：`text`、`hex`（前 1024 字节）、`bytes`（大小）、`from`（发送方的 `IP:port`）、`ms`（耗时）和 `match`（找到的文本或字节，或正则表达式的第一个分组）。

等待在运行开始时就开始监听，而不是在到达该步骤时，因此来得非常快的应答不会被错过。它只接收其路径上最近一次发送之后到达的内容。

### 限制与默认值 {#limits}

| 设置 | 默认值 | 范围 |
| --- | --- | --- |
| TCP 步骤：[[ui:common.timeoutMs]]（连接、写入和应答合计） | 4000 ms | 1–120 000 ms |
| 等待或回复的 [[ui:exp.waitTimeout]] | 2000 ms | 1–120 000 ms |
| UDP 载荷 | — | 最多 65 507 字节 |
| 监听地址 | — | 带端口的 `IP:port`；回复的 [[ui:exp.replyOn]] 可以使用端口 0（任意空闲端口） |

在检查器中，UDP 步骤的数据报以来源 `broadcast` 出现（当该步骤等待回复时为 `experiment`），到达等待端口的数据报则以来源 `experiment-wait` 出现。TCP 步骤以两帧 `tcp` 出现，来源为 `experiment`：它写入的载荷，以及收到应答时它读取的应答。其时间线条目说明它发送了什么以及应答有多大。

## 信号 {#signals}

库中的 [[ui:sig.tr.udp]] 信号是一个目标和一个载荷，为文本或十六进制字节。从 [[ui:nav.signals]] 发送它，或在任意界面按 <kbd>Ctrl</kbd>+<kbd>K</kbd>。它的目标可以是主机名。

检查器完整保留的任何数据报都可以变成一个信号：[[ui:sig.fromFrame]] 会在 [[ui:sig.capturedFolder]] 文件夹中创建一个十六进制 UDP 信号，重放那些确切的字节——对于发送的帧，发往该帧的目标；对于收到的帧，发往接收它的地址（当那是所有地址时，即本机）。TCP 数据块则不能：它是一段流的一部分。见[信号](../tools/signals.md)和[检查器](../tools/inspector.md#save-as-signal)。

文本 UDP 信号可以作为一个 [[ui:exp.node.udp]] 步骤添加到实验中；十六进制的不行，因为该步骤发送的是文本。

## 命令行 {#cli}

`signallab send udp` 发送一个数据报：

```bash
signallab send udp 127.0.0.1:9000 --text "PING"
signallab send udp 127.0.0.1:9000 --hex "de ad be ef"
```

```text
✔ sent 4 bytes → 127.0.0.1:9000
```

`--text` 和 `--hex` 必须恰好给出一个。目标是 `IP:port` 或 `host:port`（[见上文](#destinations)）。数据报发出时以 0 退出，发送失败时以 1 退出，目标或十六进制无效时以 2 退出。没有 `send tcp`。见[命令行](../automation/cli.md#cli-send-udp)。

## 风暴 {#storm}

[[ui:nav.storm]] 是给您的服务器和链路加载的负载源：[[ui:st.udp]] 以设定的速率发送设定大小的数据报，[[ui:st.tcp]] 打开一个连接、写入载荷然后关闭，如此反复。吞吐量实时测量。见[风暴](../tools/storm.md)。

## 扫描器 {#scanner}

[[ui:nav.scan]] 尝试与一个范围内的每个端口建立 TCP 连接，并列出接受连接的那些，如果您要求读取横幅，还会显示服务最先说的内容。见[扫描器](../tools/scanner.md)。

::: danger
风暴和扫描器会向真实主机发送真实流量。只把它们指向您拥有或获准测试的系统：风暴可能占满链路，而两者都可能触发入侵检测。
:::

## 模拟设备 {#emulators}

在 [[ui:nav.emulators]] 界面上，Signal Lab 可以扮演设备：

- [[ui:emu.new.udp]] 根据数据报载荷上的规则应答——任意、包含文本、匹配正则表达式、包含字节——用从到达内容构建的文本或十六进制回复，发给发送方或另一个 `IP:port`，如果您设置了延时则在延时之后；
- [[ui:emu.new.tcp]] 接受连接，按您选择的行结束符（LF、CR LF、CR，或每个数据块一到就作为一条）把到达的内容拆分为消息，用同类规则应答每一条，可以在客户端连接时发送问候语，并可以在一条回复之后关闭连接。

两者也可以作为 [[ui:exp.node.emulator]] 步骤在一次运行的时长内运行。见[模拟器](../tools/emulators.md)。

## 网络损伤 {#impairment}

[[ui:nav.netsim]] 中继位于客户端与其目标之间，劣化经过的内容：UDP 上按数据报（延迟、丢包、重复、乱序、带宽限制），或 TCP 上按流（延迟、带宽限制、连接被重置或保持半开）。见[网络损伤](../tools/impairment.md)，以及在实验中的[故障](../experiments/faults.md)。

## Windows 上的“端口不可达” {#port-unreachable}

当数据报到达一个无人监听的端口时，接收方机器通常会用一个 ICMP“端口不可达”消息来应答。Windows 会在发送方套接字的下一次接收时报告该应答，就好像连接被重置了一样——尽管 UDP 根本没有连接。

Signal Lab 预料到了这一点。它的监听器——OSC 监视器、发现监听器、实验等待与回复、UDP 和 OSC 模拟器、损伤中继——会注意到它并继续监听。已经离开的设备不会让它们停止。真正再也无法接收的监听器会结束其任务，控制台会说明原因。

## 问题 {#troubleshooting}

| 您看到的情况 | 通常的原因 |
| --- | --- |
| `… is not a valid address` | 目标没有端口，或者既不是 `IP:port` 也不是 `host:port`。 |
| `Cannot resolve …` | 该主机名在本机上无法解析。 |
| `… refused the connection — nothing is listening on that port`（TCP） | 该端口上没有程序监听，或防火墙拒绝了它。 |
| `No answer from … in time`（TCP） | 主机完全不响应——地址错误，或防火墙是丢弃而不是拒绝。 |
| 设备有应答，但等待却超时 | 设备应答的是数据报来源的端口，而不是等待的端口。让发送本身用 [[ui:exp.expectReply]] 等待回复：这样它就会从应答返回的端口发出。 |
| 来自其他机器的数据报始终不到达 | 在 Windows 上，防火墙可能挡住它们：应用提示时允许 Signal Lab。见[故障排除](../reference/troubleshooting.md)。 |

每条错误消息都列在[错误消息](../reference/errors.md#transport)中。

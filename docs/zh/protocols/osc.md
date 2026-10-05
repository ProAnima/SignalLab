---
title: OSC
description: 发送带类型的 Open Sound Control 消息，观察端口上到达的内容，并从 OSC 界面把一个连续波形送入设备。
---

# OSC

[[ui:nav.osc]] 界面是您手动通过 UDP 收发 Open Sound Control（OSC 1.0）的地方。它由三部分组成：

- [[ui:osc.sender]]：一条消息，带类型的参数，按 Enter 时发出。
- [[ui:osc.monitor]]：在一个端口上监听，并解码到达的每个数据包。
- [[ui:osc.generator]]：每秒多次发送一个跟随波形的值，并把它绘制出来。

Signal Lab 自己编码和解码 OSC。它发送的内容与[检查器](../tools/inspector.md)显示的逐字节一致。

## 发送一条消息 {#send}

1. 打开 [[ui:nav.osc]]。
2. 在 [[ui:common.target]] 中，输入设备的 IP 地址或主机名及其端口，例如 `127.0.0.1:9000` 或 `stage-mixer.local:9000`。
3. 在 [[ui:common.address]] 中，输入设备监听的地址，例如 `/mixer/fader/1`。
4. 在 [[ui:common.arguments]] 下，设置每个参数的类型和值。按 [[ui:common.addArgument]] 再添加一个；✕ 移除一个。
5. 按 [[ui:common.send]]，或在发送器的任一字段中按 <kbd>Enter</kbd>。

按钮下方的一行会说明发出了什么：地址、字节大小和目标。再次发送同一条消息会累加计数（×2、×3……），这样您就能看到重复发送确实起了作用。失败则改为显示在那里和控制台中。

切换界面和重启应用时，目标、地址和参数都会保留。

### 字段 {#send-fields}

| 字段 | 内容 | 默认值 |
| --- | --- | --- |
| [[ui:common.target]] | 接收方的 `IP:port` 或 `host:port`。IPv6 地址要放在方括号中：`[::1]:9000`。每次发送时都会解析主机名；当它有 IPv4 地址时，就使用该地址（因此 `localhost` 能到达监听在 `127.0.0.1` 上的接收方），否则使用其 IPv6 地址。没有端口的目标会被拒绝。 | `127.0.0.1:9000` |
| [[ui:common.address]] | OSC 地址，以 `/` 开头，各部分用 `/` 分隔。没有开头 `/` 的地址会在发送任何内容之前被拒绝。 | `/hello/avatar/1` |
| [[ui:common.arguments]] | 地址之后按顺序排列的带类型值。一条消息可以没有任何参数。 | 一个 `float`，`1.0` |

### 参数类型 {#types}

每个参数的类型是消息的一部分（其类型标签），因此期望 float 的设备可能会忽略值相同的 int。

| 列表中的类型 | OSC 标签 | 值 | 如何输入 |
| --- | --- | --- | --- |
| `int` | `i` | 32 位有符号整数 | 一个整数 |
| `float` | `f` | 32 位浮点数 | 一个数字，`0.75` |
| `str` | `s` | 文本 | 任意文本，以 UTF-8 发送 |
| `bool` | `T` / `F` | true 或 false | 从列表中选择 `true` 或 `false`；不携带任何字节，只有标签 |
| `long` | `h` | 64 位有符号整数 | 一个整数 |
| `double` | `d` | 64 位浮点数 | 一个数字 |
| `nil` | `N` | 无 | 没有值 |
| `blob` | `b` | 字节 | 不在这里输入：当您打开一个带 blob 的信号时，它会以只读和十六进制的形式出现 |

数字字段中如果放的不是数字，会发送 `0`。

::: tip
OSC 的 `true` 是标签 `T`，而不是文本 `"true"`。等待 bool 的设备会默默忽略字符串。
:::

## 消息包 {#bundles}

发送器发送的是单条消息，而不是消息包。当消息包（`#bundle`）到达时，监视器、实验等待和模拟器会把它拆开：其中的每条消息单独处理，其时间标签被忽略。

## 监视一个端口 {#monitor}

要查看设备或演出控制器发送了什么：

1. 在 [[ui:common.bind]] 中，输入要监听的地址和端口。`0.0.0.0:9000`（默认值）监听每一块网卡；`127.0.0.1:9000` 只监听本机。
2. 按 [[ui:osc.listen]]。监视器运行期间该字段会被锁定。
3. 把发送器的目标指向本机的 IP 地址和该端口。

每个数据包都会成为一行，最新的在最前：

| 列 | 内容 |
| --- | --- |
| [[ui:common.time]] | 到达的时刻，精确到毫秒 |
| [[ui:osc.from]] | 发送方的 `IP:port` |
| [[ui:osc.address]] | OSC 地址，或者当数据包不是有效 OSC 时为 [[ui:osc.decodeError]] |
| [[ui:osc.args]] | 参数值；blob 显示为 `blob[n]`，`nil` 显示为 `nil`。对于无法解码的数据包，显示原因。 |

一个消息包会为其中的每条消息各给出一行。列表保留最新的 300 行；[[ui:common.clear]] 清空它。按 [[ui:common.stop]] 关闭端口。监视器也是控制台任务条中的一个任务，因此也可以从那里停止。

监视器读取最大 64 KiB 的数据包。它解码标签 `i f s S b h d T F N I`（`S` 按文本读取，`I` 按 nil 读取）；带有任何其他标签或被截断的数据包会显示为解码错误，而不是被丢弃。

### 把一条消息变成等待 {#wait-for-this}

每一行都有一个 ⇠ 按钮，即 [[ui:osc.waitForThis]]。它会向打开的实验添加一个[等待 OSC](../experiments/nodes.md#node-wait_osc) 步骤，在监视器的 [[ui:common.bind]] 上监听该地址，并为每个文本、整数和 true/false 参数添加一条“等于”规则——float、blob 和 nil 则没有规则——最多 16 条规则。其超时为 2000 ms。编辑器打开时会选中这个新步骤。

::: warning
在运行该实验之前，请先停止监视器。运行会自己打开同一个端口，而两个监听器不能共用它。
:::

## 驱动一个波形 {#generator}

[[ui:osc.generator]] 向一个地址接连发送消息，只带一个参数，其值跟随一个波形——推子、灯光亮度、位置。用它来观察设备如何跟随一个变化的值，或用一个稳定的流给接收方加载。

1. 设置 [[ui:common.target]] 和 [[ui:osc.address]]。按下 [[ui:osc.startGen]] 时会像发送器那样对两者进行检查。
2. 选择 [[ui:osc.waveform]]、它的 [[ui:osc.freq]] 和 [[ui:osc.rate]]。
3. 设置 [[ui:osc.min]] 和 [[ui:osc.max]]，即值的范围。
4. 按 [[ui:osc.startGen]]。它会一直运行，直到您按 [[ui:osc.stopGen]] 或在控制台任务条中停止它的任务。

| 字段 | 内容 | 默认值 |
| --- | --- | --- |
| [[ui:common.target]] | 接收方的 `IP:port` 或 `host:port`；主机名在生成器启动时解析一次 | `127.0.0.1:9000` |
| [[ui:osc.address]] | 每条消息发送到的地址；以 `/` 开头 | `/hello/lfo` |
| [[ui:osc.waveform]] | 值随时间变化的形状（见下文） | [[ui:wave.sine]] |
| [[ui:osc.freq]] | 波形每秒的周期数 | `1` |
| [[ui:osc.rate]] | 每秒消息数，从 0.1 到 5000；超出范围的值会被限制到该范围 | `60` |
| [[ui:osc.min]]、[[ui:osc.max]] | 最低值和最高值。如果 Max 低于 Min，值不会变化。 | `0`、`1` |
| [[ui:osc.asInt]] | 舍入到最接近的整数并发送 `int` 而不是 `float` | 关 |

| 波形 | 值在每个周期中的变化 |
| --- | --- |
| [[ui:wave.sine]] | 在 Min 和 Max 之间平滑摆动，从中间开始并上升 |
| [[ui:wave.triangle]] | 从 Min 升到 Max，再回落到 Min，从 Min 开始 |
| [[ui:wave.saw]] | 下降锯齿波：从 Max 开始，降到 Min，再跳回 Max |
| [[ui:wave.ramp]] | 上升锯齿波：从 Min 开始，升到 Max，再跳回 Min |
| [[ui:wave.square]] | 前半段为 Max，后半段为 Min |
| [[ui:wave.random]] | 每条消息在 Min 和 Max 之间取一个新的随机值；不使用频率 |
| [[ui:wave.constant]] | 每次都取 Max；不使用频率 |

### 示波器 {#scope}

字段旁边，示波器会绘制发送时的值：最新的 300 个点，缩放以适配。其下方是波形、频率和速率，以及最后发送的值。无论生成器发送多快，示波器每秒大约更新 30 次，因此在高速率下它显示的是消息的抽样，而不是每一条。

如果发送失败，生成器会停止，控制台会说明原因。

## 在检查器中 {#inspector}

在开启捕获后（[[ui:dock.inspector]] 中的 [[ui:ins.arm]]），OSC 流量会以协议 `osc` 出现：

| 来源 | 内容 | 备注 |
| --- | --- | --- |
| `osc-send` | 发送器、库信号、[[ui:exp.node.osc]] 步骤或 `signallab send osc` 发送的每条消息 | 等待回复的步骤显示为 `experiment` |
| `osc-monitor` | 监视器收到的每个数据包 | 消息包以其第一条消息概括，并加上 `+n more in bundle`；格式错误的数据包的判定为 `decode error: …` |
| `osc-gen` | 生成器发送的消息 | 最多每 100 ms 捕获一条，判定为 `sampled`；跳过若干条后捕获的下一条会附上未绘制了多少条，如 `sampled · +5 not shown` |

每一帧都保留构建它所用的字节。见[检查器](../tools/inspector.md)。

## 保存与复用 {#library}

- **保存为信号。**按钮下方的 [[ui:sig.saveNew]] 会把这条消息——目标、地址和参数——保存到信号库中您选择的文件夹里。从那时起，发送器就与该信号关联：[[ui:sig.save]]（或发送器中的 <kbd>Ctrl</kbd>+<kbd>S</kbd>）会更新它，[[ui:sig.saveAs]] 会复制一份，旁边的标签会在 [[ui:nav.signals]] 中打开它。以后从 [[ui:nav.signals]] 或在任意界面按 <kbd>Ctrl</kbd>+<kbd>K</kbd> 发送它。见[信号](../tools/signals.md)。
- **添加到实验。**[[ui:common.toExperiment]] 会把一个目标、地址和参数都相同的 [OSC 消息](../experiments/nodes.md#node-osc)步骤添加到打开的实验——紧挨着 End 之前，或所选步骤之后——并打开它。实验运行期间不能添加任何内容；控制台会说明这一点。

## 在实验中 {#experiments}

| 步骤 | 作用 |
| --- | --- |
| [[ui:exp.node.osc]] | 发送一条消息。其目标、地址和文本参数接受 `{{templates}}`。开启 [[ui:exp.expectReply]] 时，它从自己的一个端口发送，并在同一步骤中在那里等待应答。[详情](../experiments/nodes.md#node-osc) |
| [[ui:exp.node.wait_osc]] | 等待一条地址匹配某个模式且参数通过规则的消息。[详情](../experiments/nodes.md#node-wait_osc) |
| [[ui:exp.node.emulator]] | 一个在整个运行期间按规则应答的 OSC 设备。见[模拟器](../tools/emulators.md)。 |

OSC 消息步骤像界面一样接受 `IP:port` 或 `host:port`，其地址必须以 `/` 开头。每次步骤发送时都会解析主机名。

### 地址模式 {#patterns}

[[ui:exp.node.wait_osc]]、[[ui:exp.node.osc]] 的回复以及 OSC 模拟器的规则都用 OSC 1.0 模式匹配地址：

| 模式 | 匹配 |
| --- | --- |
| `*` | 任意一串字符，也可以没有 |
| `?` | 恰好一个字符 |
| `[0-9]`、`[a-c]` | 集合或范围中的一个字符 |
| `[!0-9]` | 不在集合中的一个字符 |
| `{ping,pong}` | 其中一个词 |

通配符只限于斜杠之间的一个部分：`/cue/*` 匹配 `/cue/7`，但不匹配 `/cue/7/go`，而一个模式只匹配部分数相同的地址。匹配区分大小写。模式以 `/` 开头，没有空部分（`//`）、没有空格、没有 `#`，也没有 ASCII 之外的字符，且最多 512 个字符。

参数规则把参数编号 0–63 与一个值进行比较（等于、小于、包含、匹配正则表达式等）；一个等待最多有 16 条规则。当消息包到达时，只要其中有任何一条消息匹配，等待就会接收它。匹配到的消息为后续步骤提供了什么，见[数据与模板](../experiments/data.md)。

## 命令行 {#cli}

`signallab send osc` 像发送器一样发送一条消息：

```bash
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main
```

```text
✔ sent /cue/go (24 bytes) → 127.0.0.1:9000
```

每个参数都是 `tag:value` 的形式：`i:3`、`f:0.5`、`d:1.5`、`h:64`、`s:text`、`b:de ad be ef`（十六进制字节），或单独的 `T`、`F`、`N`。不带标签时，整数为 `i`，带小数点的数字为 `f`，其他一切为 `s`；写 `s:7` 可发送文本 `7`。目标是 `IP:port` 或 `host:port`。消息发出时以 0 退出，发送失败时（包括无法解析的主机名）以 1 退出，参数、地址或目标无效时以 2 退出。[`signallab fire`](../automation/cli.md#cli-fire) 发送一个保存的信号。见[命令行](../automation/cli.md#cli-send-osc)。

::: tip
在 Windows 上的 Git Bash 中，以 `/` 开头的参数会在 `signallab` 看到它之前被转换成文件路径。请在命令前加上 `MSYS_NO_PATHCONV=1` 运行，或改用 PowerShell 或 `cmd`。
:::

## 问题 {#troubleshooting}

| 您看到的情况 | 通常的原因 |
| --- | --- |
| 发送时出现 `… is not a valid address` | 目标没有端口，或者既不是 `IP:port` 也不是 `host:port`。 |
| 发送时出现 `Cannot resolve …` | 该主机名在本机上无法解析。请检查它，或改用 IP 地址。 |
| `OSC addresses start with / (…)` | 地址没有开头的 `/`。 |
| 消息已发出，但设备没有反应 | 端口或地址错误；类型与它期望的不同（用 `int` 而不是 `float`，用文本 `"true"` 而不是 bool）。在检查器中观察它，或把目标指向本机上的监视器，看看发出了什么。 |
| 按 [[ui:osc.listen]] 时出现 `… is already in use by another program` | 另一个程序——或正在运行的实验、模拟器或第二个监视器——占用了该端口。 |
| `… is not an address of this computer` | [[ui:common.bind]] 的 IP 属于另一台机器。请使用 `0.0.0.0` 或本机的一个地址。 |
| 来自其他机器的数据包始终不到达 | 在 Windows 上，防火墙可能挡住它们：应用提示时允许 Signal Lab。本机流量（`127.0.0.1`）不受影响。见[故障排除](../reference/troubleshooting.md)。 |
| [[ui:osc.decodeError]] 行 | 发送方在该端口上说的不是 OSC 1.0，或使用了 Signal Lab 不解码的类型标签。 |

在服务器上，该界面在服务器的网络上工作：`127.0.0.1` 就是服务器本身，监视器监听服务器的端口。见[服务器](../server/index.md)。

每条错误消息都列在[错误消息](../reference/errors.md#transport)中。

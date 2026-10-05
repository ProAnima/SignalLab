---
title: 广播
description: 把一个载荷发送到主机列表、广播地址、组播组或子网中的每台主机，发送一次或作为信标反复发送，并用发现监听器监听谁在应答。
---

# 广播

[[ui:nav.broadcast]] 界面（[[ui:bc.title]]）把一个 UDP 载荷一次性发送到多个目的地，并在另一端监听谁在应答。用它来查找网络中的设备、检查组播流能否到达接收方，或扮演一个应答发现探测的设备。

- [[ui:bc.emitter]] 发送到主机列表、广播地址、组播组或子网中的每台主机——一次，或作为信标反复发送。
- [[ui:bc.discovery]] 在一个端口上监听，加入组播组，列出与它通信的每个对端，并可以应答探测。

::: danger
广播、组播和遍历会到达网段上的每台设备，不只是您想到的那一台，而信标会持续这样做。请先确认您所在的网络，并且只发往您拥有或获准测试的网络。下面的限制只是防护上限，并不等于许可。
:::

## 发送 {#send}

1. 选择 [[ui:bc.mode]]（见下文）。
2. 输入目的地：字段的名称会随模式变化。在有 IPv4 地址的网络上，[[ui:bc.useSubnet]] 会根据本机地址填充它。
3. 选择 [[ui:bc.payload]] 并写入它。
4. 按 [[ui:bc.sendOnce]]：每个目的地收到一个数据报。

### 模式 {#modes}

| [[ui:bc.mode]] | 目的地 | 会发生什么 | 默认值 |
| --- | --- | --- | --- |
| [[ui:bc.mode.list]] | [[ui:bc.targetList]]：以逗号、分号或换行分隔的 `IP:port` 或 `host:port` 条目——空格不能分隔它们。名称会被解析，有 IPv4 地址时取该地址。 | 每个目的地一个数据报 | `127.0.0.1:9000, 127.0.0.1:9001` |
| [[ui:bc.mode.broadcast]] | [[ui:bc.targetAddress]]：`255.255.255.255:port`，或以 `.255` 结尾的地址 | 一个数据报，本地网络中的每台主机都会收到。路由器不会转发它。 | `255.255.255.255:9000` |
| [[ui:bc.mode.multicast]] | [[ui:bc.targetAddress]]：从 `224.0.0.0` 到 `239.255.255.255` 的组，带端口 | 一个发往该组的数据报；只有加入了该组的监听器会收到 | `239.1.1.1:9000` |
| [[ui:bc.mode.sweep]] | [[ui:bc.targetCidr]]、`a.b.c.d/nn` 和一个 [[ui:bc.port]] | 以单播向该地址块中每台可用主机发送一个数据报——用于忽略广播的设备 | `192.168.1.0/24`，端口 9000 |

遍历会跳过网络地址和广播地址（`/31` 或 `/32` 除外），不属于网络本身的基地址会向下取整到它：`192.0.2.77/30` 遍历 `192.0.2.77` 和 `192.0.2.78`。一次遍历最多覆盖 1024 台主机，因此最宽的地址块是 `/22`（1022 台主机）；更宽的会被拒绝，并给出应缩小到的前缀。

广播、组播组、遍历以及发现监听器加入的组都仅限 IPv4：IPv6 没有广播。[[ui:bc.mode.list]] 也可以指定 IPv6 主机（参见[套接字选项](#socket-options)）。

[[ui:bc.useSubnet]] 会根据本机地址 `x.y.z.w`，在 [[ui:bc.mode.list]] 中填入 `x.y.z.10:9000, x.y.z.11:9000`，在 [[ui:bc.mode.broadcast]] 中填入 `x.y.z.255:9000`，在 [[ui:bc.mode.sweep]] 中填入 `x.y.z.0/24`。它假定为 `/24` 网络。

### 载荷 {#payload}

| [[ui:bc.payload]] | 发送的内容 |
| --- | --- |
| [[ui:bc.payload.osc]] | 一条 OSC 消息：[[ui:common.address]] 和带类型的 [[ui:common.arguments]]，与 [OSC](osc.md) 界面相同。默认 `/hello/discover`，文本为 `who-is-there`。 |
| [[ui:bc.payload.text]] | [[ui:bc.text]] 按 UTF-8 编码，与输入完全一致，不带终止符。默认 `HELLO-PROBE`。 |
| [[ui:bc.payload.hex]] | [[ui:bc.hex]] 逐字节——用于重放捕获的一帧或使用二进制发现协议。十六进制数字成对出现；它们之间的其他内容会被忽略。默认 `48 45 4c 4c 4f`。 |

### 套接字选项 {#socket-options}

[[ui:bc.socketOptions]] 会展开另外三项设置：

| 选项 | 内容 | 默认值 |
| --- | --- | --- |
| [[ui:bc.bindSource]] | 数据报发出的本地 `IP:port`。固定它可选择网卡，或选择设备会应答的源端口。`0.0.0.0:0`：任意。 | `0.0.0.0:0` |
| [[ui:bc.ttl]] | 数据报可以经过的路由器数量，1–255。对组播来说这是组播跳数限制：1 会把它限制在本网络内。 | `1` |
| [[ui:bc.mcastLoop]] | 仅限组播：也把该组的数据报投递到本机，使这里的监听器能听到它们 | 开启 |

使用默认 [[ui:bc.bindSource]] 时，数据报从 IPv4 套接字发出；当所有目的地都是 IPv6 时，则从 IPv6 套接字发出。混用两者的列表会从 IPv4 套接字发送，其中的 IPv6 目的地会失败；请把它们作为单独的列表发送，或把 [[ui:bc.bindSource]] 固定到一个 IPv6 地址。

### 结果 {#result}

[[ui:bc.lastEmit]] 显示发出的内容：[[ui:bc.targets]]、[[ui:common.packets]]、[[ui:common.volume]] 和 [[ui:common.errors]]，以及它到达的前八个目的地（其余显示为“…… +n more”）。某个目的地出错不会阻止其他目的地；控制台行会说明失败了多少个。

## 作为信标反复发送 {#beacon}

信标按计划发送同一轮——每个目的地一个数据报——直到您停止它。监听周期性通告的设备就需要它。

1. 像单次发送那样设置模式、目的地和载荷。
2. 在 [[ui:bc.beacon]] 下，设置 [[ui:bc.beaconRate]]；如果它应自行停止，再设置 [[ui:bc.beaconRounds]] 或 [[ui:bc.beaconSeconds]]。
3. 按 [[ui:bc.startBeacon]]。[[ui:bc.stopBeacon]]——或在控制台任务条中停止它的任务——会结束它。

| 字段 | 内容 | 默认值 |
| --- | --- | --- |
| [[ui:bc.beaconRate]] | 每秒轮数；必须大于 0 | `2` |
| [[ui:bc.beaconRounds]] | 达到这么多轮后停止；0——不限制 | `0` |
| [[ui:bc.beaconSeconds]] | 这么多秒后停止；0——不限制 | `0` |

速率乘以目的地数量，最多为每秒 50,000 个数据报。因此对 `/24`（254 台主机）的遍历最多每秒重复约 196 次。信标运行期间，[[ui:bc.lastEmit]] 会显示 [[ui:bc.targets]]（每一轮的目的地，从第一份报告起）、总计、[[ui:bc.rounds]] 和 [[ui:bc.pps]]（每秒数据报数），每秒更新四次，且 [[ui:bc.sendOnce]] 不可用。当失败的数据报超过 32 个而没有一个成功发送时——没有路由、不允许广播——信标会自行停止并说明原因。

## 监听设备 {#discovery}

[[ui:bc.discovery]] 绑定一个 UDP 端口，并记录每个向它发送内容的对端：应答探测的内容，或设备自行通告的内容。

1. 设置 [[ui:common.bind]]，即设备发送到的端口。
2. 对于组播，在 [[ui:bc.joinGroups]] 中列出各分组。
3. 按 [[ui:bc.startListen]]。在您按 [[ui:bc.stopListen]] 之前，设置会锁定。

| 字段 | 内容 | 默认值 |
| --- | --- | --- |
| [[ui:common.bind]] | 监听的位置，`IP:port`。`0.0.0.0` 会在所有网卡上监听。 | `0.0.0.0:9000` |
| [[ui:bc.joinGroups]] | 要加入的 IPv4 组播组，以逗号分隔；留空——仅单播和广播 | `239.1.1.1` |
| [[ui:bc.interface]] | 在其上加入各分组的网卡的 IPv4 地址；留空——由系统选择 | 空 |
| [[ui:bc.reuse]] | 监听另一个程序也在使用的端口（`SO_REUSEADDR`）。只有当那个程序也允许共享时才有效。 | 开启 |
| [[ui:bc.respond]] | 像设备一样应答探测（[见下文](#auto-reply)） | 关闭 |

### 对端 {#peers}

[[ui:bc.peers]] 列出谁发送过内容，最新的在最前，上方还显示看到的对端数、听到的数据包和发出的回复：

| 列 | 内容 |
| --- | --- |
| [[ui:bc.peer]] | 发送方的 `IP:port`；其圆点表示最近 3 秒内是否听到过它 |
| [[ui:bc.proto]] | 其最后一个数据报解析为 OSC 时为 `osc`，否则为 `udp` |
| [[ui:common.packets]] | 它发送了多少 |
| [[ui:bc.age]] | 距其最后一个数据报的秒数 |
| [[ui:bc.lastMessage]] | 它的最后一个数据报：OSC 地址和参数，或文本的开头 |

列表每秒刷新几次，最多保留 512 个对端；超过之后，数据包仍会被计数，但新对端不会获得新行。

### 应答探测 {#auto-reply}

开启 [[ui:bc.respond]] 后，监听器会扮演设备：它从监听端口，对接收到每个数据报向发送方的地址和端口应答。

| 字段 | 内容 | 默认值 |
| --- | --- | --- |
| [[ui:bc.payload]] | 回复：OSC、文本或十六进制，与发送时相同 | OSC `/hello/here`，文本为 `signal-lab` |
| [[ui:bc.replyDelay]] | 像慢速设备那样，应答前等待这么久 | `0` |
| [[ui:bc.matchContains]] | 只应答解码后的文本包含此内容的数据报——OSC 地址和参数，或文本的开头；留空——全部应答 | 空 |

监听器不会应答与自己的回复完全相同的数据报，因此两个互指对方的监听器不会没完没了地互相应答。

### 防火墙与共享端口 {#firewall}

来自其他机器的广播和组播流量默认会被大多数 Windows 防火墙阻止：当应用给出提示时，允许 Signal Lab 在专用网络上通信。广播从不跨越路由器。要监听真实服务已占用的端口，双方都必须允许共享（此处为 [[ui:bc.reuse]]）；没有它，被占用的端口会被拒绝，并提示开启该项。参见[疑难解答](../reference/troubleshooting.md)。

## 在检查器中 {#inspector}

捕获开启时，此界面的流量会按其载荷以协议 `osc` 或 `udp` 出现：

| 来源 | 内容 | 数量 |
| --- | --- | --- |
| `broadcast` | [[ui:bc.sendOnce]]：每个数据报，判定为 `fan-out`、`broadcast`、`multicast` 或 `sweep`；失败的则为 `error: …` | 每一条 |
| `beacon` | 信标的各轮 | 每 50 ms 至多一轮 |
| `discovery` | 监听器收到的数据报 | 每 40 ms 至多一条 |
| `discovery` | 它的应答，判定为 `auto-reply` | 每一条 |

检查器从信标或监听器中略去的内容会被计数：它绘制的下一帧会在判定中带有该数字，即 `+n not shown`（对于信标，略去的每一轮中的每个目的地各计一帧）。参见[检查器](../tools/inspector.md)。

## 在服务器上或在 Docker 中 {#server}

在[服务器](../server/index.md)上，此界面在服务器的网络上发送和监听。在 Docker 中，只有当容器在 Linux 主机上使用宿主机的网络（`--network host`）时，广播、组播和发现才能到达本地网络。使用 Docker 的默认桥接网络或 Docker Desktop 时，只有向容器能够到达的主机发送单播才有效。

## 其他位置 {#elsewhere}

- [`signallab send udp`](../automation/cli.md#cli-send-udp) 向一台主机发送一个数据报；命令行没有广播、组播或遍历。
- [[ui:exp.node.udp]] 步骤向一台或多台主机发送文本数据报，[[ui:exp.node.wait_udp]] 步骤则等待一个。参见 [UDP 与 TCP](udp-tcp.md)。
- 要扮演一个按规则应答的设备——多条规则，回复根据收到的内容生成——请使用 [[ui:emu.new.udp]] 或 [[ui:emu.new.osc]] 模拟器。参见[模拟器](../tools/emulators.md)。

## 问题 {#troubleshooting}

| 您看到的内容 | 常见原因 |
| --- | --- |
| `… is not a broadcast address` | [[ui:bc.mode.broadcast]] 接受 `255.255.255.255:port` 或以 `.255` 结尾的地址。其他子网掩码请使用 [[ui:bc.mode.sweep]]。 |
| `… is not a multicast group` | 该地址超出 `224.0.0.0`–`239.255.255.255`。 |
| `… spans … addresses, and a sweep reaches at most 1024 hosts` | 地址块比 `/22` 更宽；请缩小它。 |
| `Set the port to sweep` | [[ui:bc.port]] 为 0。 |
| `… is over the … pps limit` | 速率乘以目的地数量超过每秒 50,000：降低 [[ui:bc.beaconRate]]，或缩小目的地。 |
| `… is already in use — turn on “share the port” to listen alongside it` | 另一个程序占用了该端口；勾选 [[ui:bc.reuse]]。 |
| `Cannot join the multicast group …` | 该组或 [[ui:bc.interface]] 在本机不可用——没有使用该地址的网卡，或没有组播路由。 |
| 已发送，但没人应答 | 设备监听的是另一个端口；这里的防火墙把它们的应答挡在外面；您与它们之间有路由器；或者在 Docker 中，容器不在宿主机网络上。 |
| 探测已发出，但发现监听器听不到任何应答 | 许多设备只向探测来自的地址和端口应答——即发送器自己的套接字，而此界面不读取它。请监听设备应答所用的端口，或从实验发送探测：带 [[ui:exp.expectReply]] 的 [[ui:exp.node.udp]] 步骤会在同一端口上发送并监听。 |

每条错误信息都列在[错误信息](../reference/errors.md#broadcast)中。

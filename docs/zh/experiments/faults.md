---
title: 故障
description: 把损伤中继和模拟器作为运行中的节点——按需切换糟糕的网络和失效的依赖，在报告中按阶段计数，并可借助种子重现。
---

# 以节点形式注入故障

要了解网络劣化或依赖停机时系统如何应对，就把故障放进实验中。中继或模拟器随运行一同打开，某个步骤按需切换它，运行报告则统计每个阶段发生了什么。运行结束时（无论通过、失败还是被停止）都会关闭它们，因此运行结束后不会有任何东西继续处于损伤状态。

| 节点 | 作用 |
| --- | --- |
| [[ui:exp.node.impairment]] | 位于被测系统与其目标之间的中继，在整个运行期间对经过的流量施加损伤 |
| [[ui:exp.node.impairment_change]] | 从此步骤起，把本次运行中的某个中继切换到另一个损伤配置 |
| [[ui:exp.node.emulator]] | 由 Signal Lab 扮演的 API、设备或代理，在整个运行期间有效 |
| [[ui:exp.node.emulator_state]] | 让本次运行中的某个模拟器停机，或让它恢复 |

这四种节点都位于添加菜单的 [[ui:exp.group.fault]] 和 [[ui:exp.group.emulate]] 下。它们的字段见[节点参考](nodes.md)；中继本身在[网络损伤](../tools/impairment.md)中介绍，模拟器在[模拟器](../tools/emulators.md)中介绍。

## 网络损伤 {#impairment}

被测系统把流量发往中继，而不是其真正的目标；中继转发到目标，把应答带回来，并对两个方向都施加损伤。

| 字段 | 内容 |
| --- | --- |
| [[ui:exp.relayListen]] | 被测系统发往或连接到的 `IP:port`，端口不能为 0 |
| [[ui:exp.relayTarget]] | 真正目的地的 `IP:port`，或 `host:port`；主机名会在运行开始时解析 |
| [[ui:ns.protocol]] | UDP（每个数据报的遭遇各自独立）或 TCP（每个连接都对应一个中继自己到目标的连接） |
| 损伤配置 | 一个 [[ui:ns.preset]]，或您自己的值 |

中继读取其损伤配置中的哪些值取决于协议，其他值会被忽略：

| 协议 | 损伤 |
| --- | --- |
| UDP | 延迟、抖动、丢包、突发丢包、重复、损坏、乱序、带宽限制、离线 |
| TCP | 延迟和抖动（流保持有序）、带宽限制（让发送方减速，不丢弃任何数据）、重置连接、使连接保持半开、离线 |

**在第一个步骤之前打开**。实验中的每个中继都在运行开始时打开，就像等待节点的套接字一样，因此其 [[ui:exp.relayListen]] 和 [[ui:exp.relayTarget]] 只接受文本和参数（`node.params_only`）：可以用带参数 `relay` 的 `{{relay}}`，但绝不能用变量。无法打开的端口，或无法解析的目标主机名，会在任何流量之前、在该节点处停止运行。

**在流程中直接通过**。运行到达该节点时，它会立即通过，时间线会显示它对什么施加了损伤、用的是什么配置。无论节点位于图中何处，中继都从运行开始一直工作到运行结束。

**随运行关闭**。无论运行以何种方式结束，中继都会关闭；TCP 中继的连接也随之关闭。自行停止转发的中继会保留原因：使用它的步骤会因此失败，报告中也会注明。

::: tip 经由损伤中继
在 [[ui:exp.node.osc]] 或 [[ui:exp.node.udp]] 节点上，[[ui:exp.routeThrough]] 会在它前面放置一个网络损伤节点：中继在 `127.0.0.1` 的某个空闲端口上监听，使用 [[ui:ns.preset.lan]] 预设转发到节点的目标，节点则改为发往中继。
:::

## 更改损伤 {#change-impairment}

[[ui:exp.node.impairment_change]] 在 [[ui:exp.relay]] 中指定实验的某个中继，并给出从该步骤起使用的损伤配置。中继保留其端口和连接；新值从下一个数据包或数据块起生效。时间线会显示新的损伤配置。

每次更改都会结束一个**阶段**。运行报告为每个中继保存：

- 其监听地址和目标地址，以及协议（为 TCP 时）；
- 其总计数：已接收、已转发、已丢弃、已限速、已重复、已损坏、已乱序、字节数；对于 TCP，还有连接数、被重置的连接数和保持半开的连接数；
- 每个阶段：损伤配置的名称，开始和结束的时间（以从中继打开时起的毫秒数表示），以及仅限该阶段的同样计数。

数据包计入决定其命运的那个阶段，即使它被延迟的副本在切换之后才发出。中继保留最近的 1000 个阶段；更早的阶段只计数，不保留。

未指定实验中任何中继的 [[ui:exp.node.impairment_change]] 会被拒绝（`impair.relay_unknown`）。

## 模拟器 {#emulator}

[[ui:exp.node.emulator]] 在整个运行期间扮演一个依赖：HTTP API、OSC、UDP 或 TCP 设备，或者 MQTT 代理。它与[模拟器](../tools/emulators.md)界面单独运行的模拟器相同：[[ui:emu.edit]] 打开其规则，[[ui:emu.toLibrary]] 在库中保存一个副本，[[ui:emu.fromLibrary]] 从库中取用一个。

- 它在第一个步骤之前打开，并一直应答到运行结束；无法打开的端口会在任何流量之前停止运行。在流程中它会立即通过。
- 其地址是字面的 `IP:port`。其匹配模式只接受参数；其回复是模板，可以读取收到的内容（`{{request.…}}`）和本次运行的参数。它不能读取机密。
- 其随机选择（按权重混合的响应、延时抖动、回复中的生成器）都从本次运行的种子中抽取。
- HTTP 模拟器也是同一地址上的 [[ui:exp.node.wait_http]] 所监听的对象：等待节点借此检查被测系统发送了什么。如果那里没有模拟器，本次运行自己的监听器会以 `204` 应答每个请求。
- OSC 或 UDP 模拟器与本次运行在该端口上的等待节点共用端口：两者都能看到每个数据报。
- MQTT 模拟器是一个代理，本次运行的 [[ui:exp.node.mqtt]] 和 [[ui:exp.node.wait_mqtt]] 节点可以像使用其他代理一样使用它。

运行报告为每个模拟器节点保存其名称、协议和地址，以及计数：请求总数、没有规则接收的请求数、失败的请求数、遇到停机的请求数、代理无法投递给慢速客户端的消息数，以及每条规则的命中次数。

## 模拟器停机与恢复 {#emulator-state}

[[ui:exp.node.emulator_state]] 在 [[ui:exp.emulatorNode]] 中指定本次运行的某个模拟器；[[ui:exp.emulatorDownState]] 为 [[ui:exp.emulatorGoesDown]] 或 [[ui:exp.emulatorComesUp]]。停机期间：

| 模拟器 | 遇到的情况 |
| --- | --- |
| HTTP | 由 [[ui:exp.downFault]] 决定：[[ui:emu.outageFault.unavailable]]（`503`，正文为 `{"error":"unavailable"}`）、[[ui:emu.outageFault.reset]]，或 [[ui:emu.outageFault.timeout]]（请求会被挂起，直到客户端放弃，最长 120 s） |
| TCP 设备、MQTT 代理 | 连接被断开，新连接被拒绝 |
| OSC、UDP 设备 | 不作任何应答 |

停机期间到达的内容计为 `down`，绝不计为没有规则接收的请求。[[ui:exp.node.wait_http]] 仍能看到这些请求。模拟器会一直停机，直到某个步骤让它恢复，无论其自身的停机计划如何；运行结束时无论如何都会关闭它。

未指定实验中任何模拟器的 [[ui:exp.node.emulator_state]] 会被拒绝（`emulator.node_unknown`）。

## 按计划停机 {#outage}

模拟器也可以自行停机：在其规则中，[[ui:emu.outage]] 设置 [[ui:emu.outageUp]] 和 [[ui:emu.outageDown]]（各为 10–3,600,000 ms），对于 HTTP 还有 [[ui:emu.outageFault]]。它先应答第一个时长，再停机第二个时长，如此循环，从它打开时开始计算；在运行中，即从第一个步骤之前开始。按计划停机期间，HTTP 模拟器的 `503` 带有 `Retry-After`，其值为距恢复还剩的整秒数，至少为 1；而在 [[ui:exp.node.emulator_state]] 步骤让它保持停机期间，`503` 不带该标头，因为没有人知道停机何时结束。

计划不需要步骤，步骤也不需要计划。时断时续的依赖用计划来模拟，在流程中选定的位置停机则用步骤。

## 示例：慢速链路后的停机 {#example}

一个客户端通过中继向 API 请求一个订单。在它请求期间，第二个分支把链路降速到 [[ui:ns.preset.4g]]，让 API 停机两秒，再让它恢复，并让链路重新恢复正常。客户端必须持续请求，直到得到应答。

```text
start → orders → link → split
split ─ branch1 → settle → until_ok ─ done → answered → joined
                           until_ok ─ body → get → status → pause → until_ok
split ─ branch2 → slow → down → outage → up → clean → joined
joined → end
```

这些名称是下面文件中各节点的 id。

1. 添加参数 `api` = `http://127.0.0.1:18091`，它指向中继，而不是 API。
2. 添加一个 [[ui:exp.node.emulator]]：HTTP，`127.0.0.1:18090`，路由 `GET /orders/:id` 以 `200` 和 `{"order":"{{request.params.id}}"}` 应答。
3. 在它之后添加一个 [[ui:exp.node.impairment]]：[[ui:exp.relayListen]] `127.0.0.1:18091`，[[ui:exp.relayTarget]] `127.0.0.1:18090`，[[ui:ns.protocol]] TCP，预设 [[ui:ns.preset.lan]]。
4. 在它之后添加一个 [[ui:exp.node.fork]]。
5. 在 [[ui:exp.branch1]] 上放置客户端：一个 300 ms 的 [[ui:exp.node.delay]]，然后是一个 [[ui:exp.node.loop]]，[[ui:exp.loopMax]] 为 40，[[ui:exp.loopUntilOn]] `{{status}}` [[ui:exp.op.eq]] `200`。其循环体：一个 [[ui:exp.node.http]] `GET {{api}}/orders/42`，一个把 [[ui:exp.from.status]] 提取到 `status` 的 [[ui:exp.node.extract]]，一个 250 ms 的延时，再连线回到循环节点。在 [[ui:exp.portDone]] 上放一个 [[ui:exp.node.log]] `Orders API answers again: HTTP {{status}}`。
6. 在 [[ui:exp.branch2]] 上放置故障：一个 [[ui:exp.node.impairment_change]]，把中继切换到 [[ui:ns.preset.4g]]；一个 [[ui:exp.node.emulator_state]]，将 Orders API 设为 [[ui:exp.emulatorGoesDown]]，并使用 [[ui:emu.outageFault.unavailable]]；一个 2000 ms 的延时；另一个 [[ui:exp.node.emulator_state]]，将其设为 [[ui:exp.emulatorComesUp]]；再一个 [[ui:exp.node.impairment_change]]，切换回 [[ui:ns.preset.lan]]。
7. 把两个分支都连到一个 [[ui:exp.node.join]]，再把该汇合节点连到 [[ui:exp.node.end]]。
8. 运行它。

时间线会显示客户端的请求经由慢速链路得到 `503` 应答，然后 API 恢复，接着是 `200`，循环通过 [[ui:exp.portDone]] 退出。报告会统计大约五个遇到 API 停机的请求和一个由其路由应答的请求，以及中继的三个阶段：短暂的 [[ui:ns.preset.lan]]、停机期间的 [[ui:ns.preset.4g]]、再次回到 [[ui:ns.preset.lan]]，每个阶段都有各自的流量。

::: details 实验文件
将其保存为 `.json` 文件，然后在 [[ui:exp.documents]] 中用 [[ui:exp.importJson]] 打开。

```json
{
  "version": 9,
  "name": "Outage behind a slow link",
  "params": [{ "name": "api", "value": "http://127.0.0.1:18091" }],
  "profiles": [],
  "profile": null,
  "seed": null,
  "nodes": [
    { "id": "start", "type": "start", "x": 40, "y": 270 },
    { "id": "orders", "type": "emulator", "x": 260, "y": 270,
      "emulator": { "name": "Orders API", "bind": "127.0.0.1:18090", "protocol": "http",
        "routes": [{ "method": "GET", "path": "/orders/:id", "when": [], "order": "sequence",
          "responses": [{ "status": 200, "headers": [], "body": "{\"order\":\"{{request.params.id}}\"}", "delay_ms": 0, "jitter_ms": 0, "fault": "none", "weight": 1 }] }],
        "fallback": null } },
    { "id": "link", "type": "impairment", "x": 490, "y": 270, "listen": "127.0.0.1:18091", "target": "127.0.0.1:18090", "protocol": "tcp",
      "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } },
    { "id": "split", "type": "fork", "x": 720, "y": 270 },
    { "id": "settle", "type": "delay", "x": 950, "y": 140, "ms": 300 },
    { "id": "until_ok", "type": "loop", "x": 1180, "y": 140, "max": 40,
      "until": { "value": "{{status}}", "op": "eq", "expected": "200" } },
    { "id": "get", "type": "http", "x": 1410, "y": 20,
      "request": { "method": "GET", "url": "{{api}}/orders/42", "headers": [], "body": null, "timeout_ms": 3000 } },
    { "id": "status", "type": "extract", "x": 1640, "y": 20, "variable": "status", "from": "status", "expr": "" },
    { "id": "pause", "type": "delay", "x": 1870, "y": 20, "ms": 250 },
    { "id": "answered", "type": "log", "x": 1410, "y": 140, "message": "Orders API answers again: HTTP {{status}}" },
    { "id": "slow", "type": "impairment_change", "x": 950, "y": 400, "relay": "link",
      "profile": { "name": "4g", "latency_ms": 60, "jitter_ms": 25, "rate_kbps": 20000 } },
    { "id": "down", "type": "emulator_state", "x": 1180, "y": 400, "emulator": "orders", "down": true, "fault": "unavailable" },
    { "id": "outage", "type": "delay", "x": 1410, "y": 400, "ms": 2000 },
    { "id": "up", "type": "emulator_state", "x": 1640, "y": 400, "emulator": "orders", "down": false, "fault": "unavailable" },
    { "id": "clean", "type": "impairment_change", "x": 1870, "y": 400, "relay": "link",
      "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } },
    { "id": "joined", "type": "join", "x": 2100, "y": 270 },
    { "id": "end", "type": "end", "x": 2330, "y": 270 }
  ],
  "edges": [
    { "from": "start", "to": "orders" },
    { "from": "orders", "to": "link" },
    { "from": "link", "to": "split" },
    { "from": "split", "to": "settle", "port": "branch1" },
    { "from": "split", "to": "slow", "port": "branch2" },
    { "from": "settle", "to": "until_ok" },
    { "from": "until_ok", "to": "get", "port": "body" },
    { "from": "get", "to": "status" },
    { "from": "status", "to": "pause" },
    { "from": "pause", "to": "until_ok" },
    { "from": "until_ok", "to": "answered", "port": "done" },
    { "from": "answered", "to": "joined" },
    { "from": "slow", "to": "down" },
    { "from": "down", "to": "outage" },
    { "from": "outage", "to": "up" },
    { "from": "up", "to": "clean" },
    { "from": "clean", "to": "joined" },
    { "from": "joined", "to": "end" }
  ]
}
```
:::

Signal Lab 还在 [[ui:exp.documents]] 中提供了两个模板，以其他方式实现同样的效果：[[ui:exp.templateFaults]] 通过一个 UDP 中继向模拟设备发送数据报，该中继依次切换为正常、丢包、离线、再恢复正常；[[ui:exp.templateOutage]] 在客户端持续请求期间，让一个模拟的 API 停机两秒。

## 端口 {#ports}

同一次运行的套接字（等待节点、回复、模拟器、中继）不能共用同一协议的端口；UDP 套接字和 TCP 套接字可以使用同一个端口号。对于中继的 [[ui:exp.relayListen]]，`0.0.0.0` 上的地址会与同一端口上的任何地址冲突。

| 套接字 | 不能与之共用端口的 |
| --- | --- |
| HTTP、TCP 或 MQTT 模拟器 | 另一个 HTTP、TCP 或 MQTT 模拟器（`emulator.bind_taken`） |
| OSC 或 UDP 模拟器 | 另一个 OSC 或 UDP 模拟器（`emulator.bind_taken`） |
| TCP 或 MQTT 模拟器 | [[ui:exp.node.wait_http]]（`emulator.bind_taken`） |
| UDP 中继的 [[ui:exp.relayListen]] | 另一个 UDP 中继、OSC 或 UDP 模拟器、等待节点或回复的套接字（`impair.bind_taken`） |
| TCP 中继的 [[ui:exp.relayListen]] | 另一个 TCP 中继，HTTP、TCP 或 MQTT 模拟器，[[ui:exp.node.wait_http]]（`impair.bind_taken`） |

有意共用的情况：HTTP 模拟器与其地址上的 [[ui:exp.node.wait_http]] 步骤；OSC 或 UDP 模拟器与其端口上的等待节点；同一地址上的多个等待节点之间。

中继不能直接或经由其他中继转发到自身，否则其流量会在环回地址上兜圈子（`impair.loop`）。在设备前串联两个中继则没有问题。

## 重现出故障的运行 {#seed}

中继做出的每个决定（数据包是否丢失、重复、损坏或被延缓，加多少抖动）都从本次运行的种子中抽取，两个方向分开，并逐包进行。模拟器的随机选择也从中抽取。用相同的种子和相同的流量再次运行，相同的数据包就会得到相同的结果：见过一次的失败可以再次看到。

要保留种子，请在时间线中按其旁边的 [[ui:exp.pinSeed]]，或通过 [[ui:exp.runWith]] 用它运行；参见[种子](runs.md#seeds)。种子无法固定的仍然是时序：被测系统何时发送，因而数据包落入哪个阶段。

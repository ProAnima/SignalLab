---
title: 运行如何流动
description: 开始与结束、输出与接线、并行分支与 Join、分支、重试、重复与循环、从运行开始就监听的等待，以及运行前检查的内容。
---

# 运行如何流动

一次运行从 [[ui:exp.node.start]] 开始，沿接线从一个节点到下一个节点，当每个分支都完成且到达 [[ui:exp.node.end]] 时结束。本页说明它遵循的规则；每个节点做什么见[节点参考](nodes.md)，随它流动的值见[数据](data.md)。

## 开始与结束 {#start-end}

一个实验恰好有一个 [[ui:exp.node.start]] 和一个 [[ui:exp.node.end]]。

- [[ui:exp.node.start]] 没有输入。它立即通过，其时间线中的
  行给出本次运行的种子。它的输出可以有多条接线：实验于是
  以并行分支开始。
- 到达 [[ui:exp.node.end]] 的每个分支都在那里停止。End 从第一次
  到达起就显示为运行中，并在最后一个分支完成后通过一次——如果有任何步骤失败则
  完全不通过。正是这次通过使运行 [[ui:exp.passed]]。
- 每个分支都无错误完成、但没有一个到达 End 的运行
  会以 `run.no_end` 失败。

## 输出与接线 {#outputs}

节点的步骤以选择一个输出结束，运行会沿该输出的每一条接线继续。大多数节点有一个输出，[[ui:exp.outputPort]]；有些在多个之间选择：

| 节点 | 必须接线的输出 | 可以接线的输出 |
| --- | --- | --- |
| [[ui:exp.node.end]] | — | — |
| [[ui:exp.node.fork]] | [[ui:exp.branch1]]、[[ui:exp.branch2]] | — |
| [[ui:exp.node.branch_status]]、[[ui:exp.node.branch_value]] | [[ui:exp.yes]]、[[ui:exp.no]] | — |
| 所有等待节点（[[ui:exp.node.wait_osc]]、[[ui:exp.node.wait_udp]]、[[ui:exp.node.wait_mqtt]]、[[ui:exp.node.wait_http]]、[[ui:exp.node.wait_ws]]） | [[ui:exp.portMatched]] | [[ui:exp.portTimeout]] |
| [[ui:exp.node.loop]] | [[ui:exp.portBody]]、[[ui:exp.portDone]] | [[ui:exp.portLimit]] |
| 其他所有节点 | [[ui:exp.outputPort]] | — |

要连接，从一个输出拖到一个节点上；拖放到空画布上，会在那里添加一个新节点。从一个已有接线的输出拖出会再添加一条接线。[[ui:exp.addNext]]、<kbd>A</kbd> 键以及接线上的 ＋ 则把节点插入到现有接线中。

未完成的图是草稿：它会保存，但不会运行。工具栏中的 [[ui:exp.needsLinks]] 说明缺少什么并显示该节点。参见[运行前检查的内容](#validation)。

## 并行分支 {#parallel}

### 一个输出引出多条接线 {#fan-out}

当一个输出有多条接线——包括 [[ui:exp.node.start]] 的——它们所通向的每个节点都同时运行。第一条接线延续该分支；每一条后续接线启动一个并行分支。每个分支都携带自己的变量副本和最近一次 HTTP 响应副本，因此一个分支设置或收到的内容不会被其他分支看到。

### 并行分支与 Join {#fork-join}

[[ui:exp.node.fork]] 立即通过，并同时从 [[ui:exp.branch1]] 和 [[ui:exp.branch2]] 离开——与一个输出引出两条接线相同，只是画成了一个节点。

[[ui:exp.node.join]] 等待通向它的**每一条**接线，然后作为一个分支继续，其副本按这些接线的顺序合并：

- 它们全部的变量——两条分支都设置的同名值，实验中后列出的接线胜出；
- 按该顺序，最后一条带来 HTTP 响应的接线的响应；
- 对于其后的等待节点，它们最近一次动作中最早的那个。

接线的顺序决定一切，绝不取决于哪个分支碰巧先完成。

::: warning 只 Join 并行运行的分支
Join 会统计它的接线，无论它们是如何变成并行的：来自一个
[[ui:exp.node.fork]]、来自一个输出的多条接线、来自不同的路径。分支的 [[ui:exp.yes]] 和 [[ui:exp.no]] 之后只有一条路径运行，因此由两者共同汇入的 Join 会等待一个永远不会到来的分支：运行会以 `run.join_waiting` 失败，并指出有多少条接线从未被跟随。要把可选路径汇合，请把它们直接接到下一个节点。
:::

被两条并行分支到达的非 Join 节点，会为它们各运行一次。

### 当步骤失败时 {#failure}

第一个失败使运行失败。其他分支不会开始新步骤：重复或负载提前结束，它们正在进行的任何其他步骤会运行到结束。它们期间遇到的一次失败会在时间线中报告，但不是本次运行的错误。在有 [[ui:exp.portTimeout]] 接线的情况下遇到超时的步骤没有失败——参见[等待](#timeout)。

## 分支 {#branching}

| 节点 | 何时从 [[ui:exp.yes]] 离开 |
| --- | --- |
| [[ui:exp.node.branch_status]] | 此路径上最近一次 HTTP 响应的状态为给定状态 |
| [[ui:exp.node.branch_value]] | 其比较成立——参见[比较值](data.md#compare) |

否则各自从 [[ui:exp.no]] 离开。状态分支在每条路径上都需要它之前有一个 HTTP 请求；按值分支需要它读取的名称在那里已知。当 [[ui:exp.yes]] 和 [[ui:exp.no]] 之后的路径再次汇合时，汇合处的节点只运行一次，且在那里只已知两条路径都设置的变量（[变量在哪里可见](data.md#visibility)）。

## 重试 {#retry}

发送或监听的步骤可以在失败时再试：在其属性中开启 [[ui:exp.retryOn]]。

| 设置 | 内容 | 范围 | 初始值 |
| --- | --- | --- | --- |
| [[ui:exp.attempts]] | 总共尝试次数，包括第一次 | 1–10 | 3 |
| [[ui:exp.retryDelay]] | 第二次尝试前的暂停 | 0–60 000 ms | 500 |
| [[ui:exp.backoff]] | [[ui:exp.backoff.fixed]]：每次暂停相同；[[ui:exp.backoff.exponential]]：每次暂停是上一次的两倍 | — | [[ui:exp.backoff.fixed]] |

- 重试适用于 [[ui:exp.node.http]]、[[ui:exp.node.tcp]]、
  [[ui:exp.node.mqtt]]、[[ui:exp.node.osc]]、[[ui:exp.node.udp]]、
  [[ui:exp.node.ws_connect]]、[[ui:exp.node.ws_send]] 以及所有等待节点。其他
  节点拒绝它（`node.retry_unsupported`），处于
  [负载](load.md)下的 HTTP 请求不接受重试。
- 无论翻倍后是多少，单次暂停都不超过 60 s。
- 每次失败的尝试都会在时间线中显示为 [[ui:exp.retry]]，带其
  次数和原因。步骤随后通过，或以最后一次尝试的原因失败。
- 只有执行被重复。模板无法解析的字段会立即
  失败。
- 等待回复的发送会再次发送。等待节点再次等待，像之前一样
  从分支上最近一次动作起计数。
- 带有 [[ui:exp.portTimeout]] 接线的等待节点不会因超时而失败，因此
  不会被重试：它沿 [[ui:exp.portTimeout]] 继续。
- [停止](#stop)会立即结束暂停。

## 重复 {#repeat}

发送的步骤可以反复发送——心跳、轮询、稳定流——而无需在图中有循环：开启 [[ui:exp.repeatOn]]。

| 设置 | 内容 | 范围 | 初始值 |
| --- | --- | --- | --- |
| [[ui:exp.repeatBy]] | [[ui:exp.repeatBy.count]] 或 [[ui:exp.repeatBy.duration]] | — | [[ui:exp.repeatBy.count]] |
| [[ui:exp.repeatCount]] | 总共发送次数，包括第一次 | 2–10 000 | 10 |
| [[ui:exp.repeatDuration]] | 从第一次发送起持续发送多久 | 1–300 000 ms | 10 000 |
| [[ui:exp.repeatInterval]] | 两次发送之间的暂停 | 10–60 000 ms | 1000 |
| [[ui:exp.repeatJitter]] | 每次暂停随机延长至多此值 | 0–60 000 ms | 0 |

- 重复适用于 [[ui:exp.node.http]]、[[ui:exp.node.tcp]]、
  [[ui:exp.node.mqtt]]、[[ui:exp.node.osc]]、[[ui:exp.node.udp]] 和
  [[ui:exp.node.ws_send]]（其他位置为 `node.repeat_unsupported`）。HTTP
  请求要么重复，要么[负载](load.md)，不能两者兼有。
- 每次发送都像单次发送那样进行：其模板会被重新读取——
  `{{counter}}` 是发送的编号，`{{now}}` 是其时间——重试开启时
  适用于每次发送。等待回复的发送等待自己那一次。
- 按时间重复时，只有当一次发送能在时间结束前开始时才会发出。
- 抖动从本次运行的种子中抽取：相同的种子给出相同的
  暂停。
- 时间线最多每秒一次将进度报告为 [[ui:exp.repeating]]。
  步骤在最后一次发送后以该次发送的结果通过；彻底失败的发送使步骤失败。
- 另一个分支的失败会结束发送；[停止](#stop)会立即结束暂停。

它必须能放进一次运行：发送次数及其最长暂停
（`(count − 1) × (interval + jitter)`）合计最多 300 s（`node.repeat_too_long`），
按时间重复最多发送 10 000 次（`node.repeat_too_many`）。

## 循环 {#loop}

[[ui:exp.node.loop]] 反复运行其 [[ui:exp.portBody]] 输出上的步骤；其中最后一个接回循环节点。

| 设置 | 内容 | 范围 |
| --- | --- | --- |
| [[ui:exp.loopMax]] | 最多迭代次数 | 1–1000 |
| [[ui:exp.loopUntilOn]] | 退出条件：[[ui:exp.value]]、[[ui:exp.operator]]、[[ui:exp.expected]]，与 [[ui:exp.node.assert_value]] 中相同 | 可选 |

1. 从外部到达时，循环节点在 [[ui:exp.portBody]] 上开始第 1 次迭代。
2. 每次循环体返回时读取退出条件——在迭代之后，因此循环体
   至少运行一次，并能设置它要测试的内容。
3. 条件成立时，循环节点从 [[ui:exp.portDone]] 离开。
4. 否则开始下一次迭代，只要还有剩余。
5. 迭代次数先用尽时，如果 [[ui:exp.portLimit]] 已接线，循环节点从它离开，
   否则以 `loop.limit` 使运行失败。没有条件时，循环体运行每一次迭代，
   循环节点从 [[ui:exp.portDone]] 离开。

在循环体内部，`{{counter}}` 是迭代的编号，因为每个节点
统计自己的执行次数。条件和 [[ui:exp.portDone]] 或 [[ui:exp.portLimit]] 之后的步骤
可以使用循环体每次迭代都设置的内容——例如循环体提取的状态；
循环体本身只能看到到达循环节点时已知的内容。

模板 [[ui:exp.templatePoll]] 每 0.3 s 向设备询问其状态，直到它应答 `ready`，最多 10 次。

### 循环体可以包含什么 {#loop-body}

循环体作为一个分支运行，一次迭代接一次迭代。接回循环节点的接线是实验唯一可以有的环；其他任何环都是 `graph.cycle`。

| 规则 | 错误 |
| --- | --- |
| [[ui:exp.portBody]] 上的内容接回循环节点 | `loop.no_return` |
| 循环体中的每个输出只有一条接线 | `loop.body_parallel` |
| 循环体中的每个输出都继续留在循环体中或接回循环节点 | `loop.body_leaves` |
| 只有循环节点的 [[ui:exp.portBody]] 输出通向循环体 | `loop.body_entered` |
| 循环体中不能有 [[ui:exp.node.start]]、[[ui:exp.node.end]]、[[ui:exp.node.fork]]、[[ui:exp.node.join]] 或其他 [[ui:exp.node.loop]] | `loop.body_unsupported` |

## 等待 {#waits}

等待节点在所等待的消息到达时通过：
[[ui:exp.node.wait_osc]]、[[ui:exp.node.wait_udp]]、[[ui:exp.node.wait_mqtt]]、
[[ui:exp.node.wait_http]] 和 [[ui:exp.node.wait_ws]]。每个匹配什么见
[节点参考](nodes.md)；这里说明它们如何监听。

### 从开始就监听 {#listening}

运行在**其第一个步骤之前**就打开其等待节点所监听的内容，因此比下一个步骤更快的回复不会被错过：

| 等待 | 在第一个步骤之前打开 |
| --- | --- |
| OSC、UDP | 每个 [[ui:exp.listenOn]] 地址一个 UDP 套接字，由使用该地址的每个等待节点共用 |
| HTTP 请求 | 每个地址一个监听器——有本次运行的 HTTP [模拟器](faults.md#emulator) 时就是它，否则是一个以 `204` 应答的监听器 |
| MQTT | 每个代理和主题过滤器一个连接，已订阅；代理随后重放的保留消息会被忽略 |
| WebSocket | 什么都没有：它读取 [[ui:exp.node.ws_connect]] 运行时打开的连接 |

因为它们最先打开，这些地址在运行前就已固定：OSC、
UDP 或 HTTP 等待节点监听字面的 `IP:port`，端口不能为 0，MQTT 等待节点的代理和主题只接受参数。无法打开的端口——被
占用，或不是本机的地址——会在任何流量之前、在该等待节点的字段处停止运行。运行以任何方式结束时，一切都会关闭。

### 哪些消息计数 {#counting}

等待节点考虑的是在**其分支上最近一次动作**开始之后到达的消息——最近一次请求、消息、发布、WebSocket 连接或
发送——或者在没有任何动作之前，运行开始之后到达的消息。请求之前的消息不计数，请求与等待之间的延时或日志不会隐藏其回复。Join 之后，合并分支中最早的那个最近动作计数。

等待节点取第一个匹配的消息并消费它：两个等待节点绝不会
匹配同一条消息。

每个套接字、订阅或连接为其等待节点最多保留 1024 条消息和 64 MiB；超过后，最旧的会被丢弃并计数。

### 超时 {#timeout}

[[ui:exp.waitTimeout]] 为 1–120 000 ms，初始为 2000。当时间内没有任何匹配时：

- 有 [[ui:exp.portTimeout]] 接线时，等待节点沿它继续；
- 没有时，步骤以 `wait.timeout` 失败，它说明期间还到达了多少条其他
  消息——错误的模式与设备沉默看起来不同——并在其详情中说明队列满时丢弃了多少条更旧的消息。

等待节点的变量——`reply`，HTTP 为 `request`——只在 [[ui:exp.portMatched]] 之后存在。当 [[ui:dock.inspector]] 正在捕获时，步骤
还会链接它匹配到的帧：参见[时间线](runs.md#timeline)。

## 同一步骤上的回复 {#reply}

[[ui:exp.node.osc]] 或 [[ui:exp.node.udp]] 可以等待自己的应答：开启 [[ui:exp.expectReply]]。

| 设置 | 内容 | 初始值 |
| --- | --- | --- |
| [[ui:exp.replyOn]] | 等待应答的地址；端口 0 表示任意空闲端口 | `0.0.0.0:0` |
| [[ui:exp.replyAddress]]（OSC）、[[ui:exp.replyMode]]（UDP） | 应答必须是什么，与匹配等待节点中相同 | 任意 |
| [[ui:exp.waitTimeout]] | 1–120 000 ms | 2000 |
| [[ui:exp.replyVariable]] | 应答写入的变量 | `reply` |

[[ui:exp.replyOn]] 上的套接字像等待节点的一样在第一个步骤之前打开，消息**从它发出**：向发送方自身端口应答的设备能被听到，向固定端口应答的设备在该端口正是给定端口时能被听到。步骤在收到匹配的应答后通过，变量在其输出之后存在。没有 [[ui:exp.portTimeout]] 输出：时间内没有应答会使步骤失败，重试可以再次发送。要按沉默分支，请使用单独的等待节点。

## 运行前检查的内容 {#validation}

编辑器在您编辑时检查实验；运行按钮会再检查一次。问题会指出节点，有字段时也指出字段。

| 规则 | 错误 |
| --- | --- |
| 实验有名称 | `doc.name_required` |
| 1–64 个节点，恰好一个 Start 和一个 End | `doc.node_count`、`doc.start_end_count` |
| Start 没有输入 | `graph.start_input` |
| 接线通向另一个存在的节点 | `doc.connection_invalid` |
| 同一条接线不会出现两次 | `doc.connection_duplicate` |
| 每个必须接线的输出都已接线 | `graph.outputs_required` |
| 节点在它没有的输出上没有接线 | `graph.port_unexpected` |
| 每个节点都能从 Start 到达 | `graph.unreachable` |
| 除了 Loop 接回的接线外没有环 | `graph.cycle`，以及[循环体规则](#loop-body) |
| 检查或 Extract 在每条路径上其之前都有 HTTP 请求；负载下的请求不计数 | `graph.needs_http` |
| 每个模板都能解析，且它使用的每个名称在每条路径上都已知 | `template.*`、`name.*`——参见[数据](data.md#unknown-names) |
| 每个字段都存在且在范围内 | `node.*` |
| 引用另一个节点的节点——[[ui:exp.node.impairment_change]]、[[ui:exp.node.emulator_state]]、WebSocket 节点——引用了一个存在的节点，且 WebSocket 节点在其 connect 之后 | `impair.relay_unknown`、`emulator.node_unknown`、`ws.connection_unknown`、`ws.connection_after` |
| 本次运行的两个套接字不共用端口 | 参见[故障](faults.md#ports) |

然后运行会检查它开始所需的条件：每个[机密](data.md#secret-check)已存储，每个端口已打开。在一切成立之前，不会运行任何步骤，也不会发送任何内容。

## 时间限制 {#limit}

一次运行最多持续 300 s。届时仍在进行的会被停止并以 `run.timeout` 失败。从[命令行](../automation/cli.md#cli-run)和
[API](../api/run.md) 可以设置更短的限制，1–300 s。

## 停止 {#stop}

运行进行期间，运行按钮是 [[ui:common.stop]]。停止会立即结束运行：每个分支、重试或重复的每次暂停、每个等待和每个负载——在途的请求会被放弃。它的套接字、订阅、模拟器和中继会关闭，其 WebSocket 连接会发送关闭帧。顶栏中的 [[ui:app.stopAll]] 对每个任务做同样的事。被停止的运行不保存报告；参见[运行](runs.md#stop)。

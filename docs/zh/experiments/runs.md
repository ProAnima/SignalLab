---
title: 运行与结果
description: 使用实验的值或其他值启动运行、时间线、停止、通过与失败的含义、运行报告、种子、试用单个节点，以及带版本号的实验文件。
---

# 运行与结果

## 启动一次运行 {#start}

在编辑器工具栏中按 [[ui:exp.run]]。在发送任何内容之前：

1. 实验按编辑器的方式被检查——其图、字段、
   名称和值（[检查的内容](flow.md#validation)）——并且它使用的每个机密
   都必须已存储（[机密](data.md#secret-check)）。
2. 它会被保存。
3. 运行会打开它整个时长所需的一切：其模拟器、其
   损伤中继、等待节点监听的套接字，以及其 MQTT
   订阅。

如果其中任何一项失败，什么都不会运行：问题会被显示，且它所
涉及的节点会被选中。否则时间线会在画布下方打开，步骤
会随其发生而出现。运行进行期间，[[ui:exp.run]] 变为
[[ui:common.stop]]，实验无法编辑。

运行使用使用中的配置的值，以及实验中固定的种子或一个新种子。要用其他值运行一次，请使用 [[ui:exp.runWith]]。

### 使用其他值运行 {#run-with}

[[ui:exp.run]] 旁的 ▾ 会打开 [[ui:exp.runWith]]：为一次
运行使用其他值，而不更改实验。

| 字段 | 内容 | 留空时 |
| --- | --- | --- |
| [[ui:exp.profile]] | 本次运行的配置；实验有配置时显示 | 使用中的那个 |
| 每个参数 | 仅用于本次运行的值 | 所选配置的值，以灰色显示 |
| [[ui:exp.seed]] | 本次运行的种子，0–9 007 199 254 740 991；其旁边的按钮填入上次运行的种子 | 固定的种子，或一个新种子 |

表单中的 [[ui:exp.run]] 启动运行；[[ui:exp.resetOverrides]] 清空
表单。您输入的内容会在本会话中保留在表单里，因此下次同样的更改只需
一次点击。无法运行的配置会标有 ⚠。值的
优先级见[数据](data.md#precedence)。

当实验有配置，或有运行为其输入过值时，时间线会
说明本次运行使用了哪个配置——或 [[ui:exp.runDefaults]]——以及，
输入过值时的 [[ui:exp.overridden]]。

## 时间线 {#timeline}

[[ui:exp.timeline]] 位于画布下方；▸ 和 ▾ 折叠它，其边缘可调整
大小。它每个步骤事件一行，最旧的在前：时间、节点、其
状态和发生的事——`HTTP 200 · 41 ms`、`token = abc123`、
`/pong 42 ← 127.0.0.1:9000 · 12 ms`。失败会说明原因，其技术详情
在工具提示中。点击一行会在画布上选中其节点。

| 状态 | 步骤 |
| --- | --- |
| [[ui:exp.running]] | 已开始 |
| [[ui:exp.passed]] | 顺利结束，并选择了其输出 |
| [[ui:exp.failed]] | 失败；第一个失败即本次运行的失败 |
| [[ui:exp.retry]] | 一次尝试失败，将再次尝试（[重试](flow.md#retry)） |
| [[ui:exp.repeating]] | 正在反复发送，最多每秒一行（[重复](flow.md#repeat)） |
| [[ui:exp.load]] | 处于负载下，每秒一行（[负载](load.md#progress)） |

在画布上，每个节点带有一个显示其最新状态的标记。

时间线的标题包含：

- 结果：[[ui:exp.running]]、带原因的 [[ui:exp.passed]]、[[ui:exp.failed]]，
  或 [[ui:exp.stopped]]；
- 报告写入后的 [[ui:exp.reportSaved]]——在浏览器中是一个下载它的链接，
  在桌面应用中是工具提示中的路径；
- [[ui:exp.compare]]，用于把本次运行与更早的一次并排
  （[对比运行](load.md#compare)）；
- 配置和更改的值，如上所述；
- 本次运行的种子及 [[ui:exp.pinSeed]]，或者，当实验固定了种子时，
  该种子及 [[ui:exp.unpinSeed]]（[种子](#seeds)）。

**帧。** 当 [[ui:dock.inspector]] 正在捕获时，匹配了消息的等待节点——或等待其
回复的发送——会保留该消息帧的编号。行下方的一个按钮会指出节点和帧；它
会在底部面板中打开检查器并选中该帧。参见
[检查器](../tools/inspector.md)。

时间线显示本次会话中该实验的上一次运行；打开另一个实验时它会被清空。

## 停止 {#stop}

按 [[ui:common.stop]]，或顶栏中用于一次性停止每个任务的 [[ui:app.stopAll]]。运行立即结束（[什么会停止](flow.md#stop)），时间线显示
[[ui:exp.stopped]]，且**不保存报告**。从命令行或在服务器上通过 API 启动的运行与任何其他任务一样：该服务器上的 [[ui:app.stopAll]] 也会停止它，其调用者会得知它被停止了。

## 结果 {#result}

| 结果 | 含义 | 报告 |
| --- | --- | --- |
| [[ui:exp.passed]] | 每个分支都完成，没有步骤失败，且到达了 End | 已保存 |
| [[ui:exp.failed]] | 某个步骤失败——一次检查、没有 [[ui:exp.portTimeout]] 接线的等待、网络错误、阈值——或者运行超出了时间（`run.timeout`）、Join 徒劳等待（`run.join_waiting`）、没有任何分支到达 End（`run.no_end`） | 已保存，带第一个失败 |
| [[ui:exp.stopped]] | 有人停止了它 | 无 |
| 未启动 | 实验无效、缺少机密，或某个端口无法打开 | 无 |

失败的运行会指出其第一个失败的节点和字段；
[错误参考](../reference/errors.md)列出了每个代码。命令行
用其退出码说明同样的事：`0` 通过，`1` 失败，`2` 实验或
调用无效（缺少机密也算），`3` 实验之外的因素阻止了它运行，例如某个端口无法打开。参见
[`signallab run`](../automation/cli.md#cli-run)。

## 运行报告 {#report}

每次自行结束的运行——通过或失败——都会在数据文件夹的
`runs` 文件夹中写入一份 JSON 报告：桌面上是 `Documents/SignalLab/runs`，
服务器上是服务器自己的数据文件夹（[文件](../reference/files.md)）。文件
名为 `run-<start time in ms>-<job number>.json`；报告绝不会覆盖
另一份。如果无法写入，编辑器会说明原因。

| 键 | 内容 |
| --- | --- |
| `version` | 报告的格式，现为 5 |
| `experiment` | 实验的名称 |
| `document_version` | 实验的版本，现为 9 |
| `seed` | 本次运行使用的种子 |
| `profile` | 它运行所用的配置，默认值时为 `null` |
| `overrides` | 在 [[ui:exp.runWith]] 中输入的值 |
| `params` | 本次运行使用的每个参数值 |
| `started_ms`、`ended_ms` | Unix 毫秒 |
| `outcome` | `passed` 或 `failed` |
| `error` | 第一个失败，或 `null` |
| `steps` | 每个步骤事件，按顺序（见下文） |
| `emulators` | 每个 Emulator 节点的计数——有时出现（[模拟器](faults.md#emulator)） |
| `impairments` | 每个 Impairment 节点的计数和阶段——有时出现（[阶段](faults.md#change-impairment)） |

每个步骤事件有：

| 键 | 内容 |
| --- | --- |
| `job_id`、`node_id` | 运行和节点 |
| `ts` | Unix 毫秒 |
| `state` | `running`、`passed`、`failed`、`retry`、`repeating`、`load` |
| `detail` | 发生了什么，英文 |
| `message_key`、`message_params` | 与界面文本及其值相同，因此步骤可以用任何语言显示 |
| `vars` | 步骤写入的变量（如有） |
| `error` | 失败原因：`code`、`params`、`node`、`field`、`detail` |
| `frame` | 等待节点匹配到的 Inspector 帧，如果捕获已开启 |
| `load` | 负载测得的内容（[测量项](load.md#metrics)），在其最后一个事件上 |

机密值绝不会出现在报告中：它们被遮蔽为 `••••`
（[遮蔽](data.md#masking)）。

报告的格式随功能增长：版本 3 添加了模拟器的
计数，版本 4 添加了损伤的阶段，版本 5 添加了负载的测量。

报告就是运行历史：[[ui:exp.compare]] 读取它们，
[`experiment_runs`](../api/commands.md#experiment_runs) 也是如此。命令行的
`--report` 会把一次运行的报告复制到您想要的位置。

## 种子 {#seeds}

每次运行都有一个种子，一个从 0 到 9 007 199 254 740 991 的整数。它依次
是：

1. 在 [[ui:exp.runWith]] 中、命令行
   （`--seed`）或 API 给本次运行的种子；
2. 实验中固定的种子；
3. 一个新的随机种子。

运行在时间线中的第一行给出它，报告保留它。

种子决定一次运行所做的每一件随机之事：模板中的
[生成器](data.md#generators)、[重复](flow.md#repeat)的抖动、[随机负载](load.md#schedule)的
到达、[损伤中继](faults.md#seed)中每个数据包的命运，以及模拟器的
随机选择。每个都从自己的流中抽取，因此并行分支
绝不会改变彼此的值。

要重现一次运行：

1. 在时间线中按其种子旁边的 [[ui:exp.pinSeed]]。种子会存入
   实验，每次运行都使用它，直到您按
   [[ui:exp.unpinSeed]]。在 [[ui:exp.params]] 中，[[ui:exp.seed]] 显示并编辑
   固定的种子；留空时为 [[ui:exp.seedRandom]]。
2. 使用相同的配置和值运行；报告会列出它们。

种子无法重现的：时间（`{{now}}`）、`{{run.id}}`，以及设备和网络何时应答。

## 试用单个节点 {#send-now}

要试用单个节点而不运行实验，请选中它并按
[[ui:exp.sendNow]]——或在其属性中按 <kbd>Ctrl</kbd>+<kbd>Enter</kbd>——
适用于 [[ui:exp.node.http]]、[[ui:exp.node.tcp]]、[[ui:exp.node.osc]]、
[[ui:exp.node.udp]]、[[ui:exp.node.mqtt]]、[[ui:exp.node.ws_connect]] 或
[[ui:exp.node.ws_send]] 节点。在等待节点上它是 [[ui:exp.listenNow]]：它从现在起监听到有消息
匹配或其超时结束。

引擎使用运行所用的代码执行该节点一次：

- 使用使用中的配置的值、此会话中已知的变量值
  （来自上次运行和更早的试用）以及已存储的机密；
- 使用固定的种子，或一个新种子；`{{run.id}}` 为 `0`，`{{counter}}`
  为 `1`；
- 不使用重试、重复或负载——一次发送；
- **不使用 Cookie**：一个请求，其之前没有设置任何要回发的内容；
- 不使用本次运行的中继和模拟器。[[ui:exp.node.wait_http]] 在自己的
  监听器上监听，WebSocket 发送或等待会为那一次试用打开其
  [[ui:exp.node.ws_connect]] 所描述的连接。

如果节点使用的某个名称尚无值，则不会发送任何内容，结果会说明
缺少哪些名称——请先运行实验，或对设置它们的节点使用 [[ui:exp.sendNow]]。

结果显示 ✓ 或 ✕ 以及发生的事。对于 HTTP 请求，它还显示
状态、时间、大小和 [[ui:http.response]]；在 JSON 响应中，
每个值都可以点击以[提取它](data.md#extract)，[[ui:http.mockThis]] 则把该响应变成一个模拟器的路由。等待节点收到的值，或请求之后紧接的
[[ui:exp.node.extract]] 节点将从其响应中取的值，会为预览和
下一次 [[ui:exp.sendNow]] 所知。运行进行期间，[[ui:exp.sendNow]] 不可用。

模板节点的预览——它[将发送](data.md#preview)什么——也由
引擎解析，不发送任何内容。

## 实验文件 {#files}

### 工作实验 {#working-file}

编辑器持有一个实验，每次更改 0.7 s 后自动保存到
数据文件夹中的 `experiment.json`；工具栏显示 [[ui:exp.saving]]、
[[ui:exp.saved]] 或 [[ui:exp.saveError]]。未完成的图也会保存。无法读取的文件会连同其路径一起报告，绝不会被替换。实验
文件最大 4 MiB。

在服务器上，该文件位于服务器的数据文件夹中，因此在其中打开
编辑器的每个浏览器都操作同一个实验。

### 打开、模板与导出 {#open-export}

工具栏中的 ☰ 按钮会打开 [[ui:exp.documents]]：

- [[ui:exp.templates]]：[[ui:exp.templateEmpty]]、[[ui:exp.templateHttp]]、
  [[ui:exp.templateBranch]]、[[ui:exp.templateParallel]]、
  [[ui:exp.templatePingReply]]、[[ui:exp.templatePoll]]、
  [[ui:exp.templateFlaky]]、[[ui:exp.templateFaults]]、
  [[ui:exp.templateOutage]]、[[ui:exp.templateWsEcho]]。它们的目标都在
  `127.0.0.1` 上。
- [[ui:exp.importJson]] 读取一个最大 4 MiB 的文件——属于实验格式的此版本
  或更旧的版本，打开时会更新到最新——
  并在显示其名称、多少节点和连接之前检查它。按编辑器书写时的缩进形式，该文件也必须不超过 4 MiB，因此
  接近限制的紧凑文件可能被拒绝。损坏的文件会被拒绝，并给出
  问题的行和列；来自更新的 Signal Lab 的文件会以
  `doc.version_unsupported` 被拒绝，当前实验保持不变。
- [[ui:exp.openDocument]] 用所选的实验替换当前实验。
  <kbd>Ctrl</kbd>+<kbd>Z</kbd> 在本会话期间可以恢复上一个。打开
  实验不会运行它。
- [[ui:exp.exportJson]] 把一份副本写入数据文件夹的 `exports`
  文件夹，文件名为 `experiment-<time in ms>-<random>.json`，绝不
  覆盖另一份副本；在浏览器中，[[ui:common.download]] 会获取它。

命令行和 API 接受相同的文件，以及按名称指定的模板：
`empty`、`http-check`、`status-branch`、`parallel-flows`、`osc-ping-reply`、
`poll-until-ready`、`flaky-api`、`fault-phases`、`dependency-outage`、
`websocket-echo`。

### 文档版本 {#versions}

实验文件有一个 `version`；此 Signal Lab 写入版本 9，并能
打开所有更早的版本，补齐旧文件无法保存的内容。版本比 9 新的
文件会被拒绝（`doc.version_unsupported`），而不是在缺少
其内容的情况下打开。

| 版本 | 新增 |
| --- | --- |
| 2 | 参数和种子 |
| 3 | 配置 |
| 4 | 重试，以及 OSC 或 UDP 发送等待的回复 |
| 5 | 重复，以及 [[ui:exp.node.loop]] |
| 6 | [[ui:exp.node.emulator]] 和 [[ui:exp.node.wait_http]] |
| 7 | [[ui:exp.node.impairment]]、[[ui:exp.node.impairment_change]] 和 [[ui:exp.node.emulator_state]] |
| 8 | WebSocket 节点、HTTP 认证和 Cookie 存储 |
| 9 | HTTP 请求的负载，以及 TCP 上的损伤 |

版本 8 之前的文件打开时 [[ui:exp.cookies]] 关闭，因此它会像以前那样
运行；更新的文件保留自己的设置。再次保存后，任何文件都会变成版本
9。

## 从命令行或服务器 {#automation}

运行在哪里都一样：命令行和服务器的 API 启动与编辑器相同的运行，具有相同的步骤、结果和报告。

```bash
signallab run checkout.json --profile Stage -p api=http://192.0.2.10:8080 --seed 42 --report report.json
```

- [`signallab run`](../automation/cli.md#cli-run) 在此进程或服务器上运行实验文件或
  模板，像时间线一样打印步骤
  并以结果的代码退出。
- [`POST /api/run`](../api/run.md) 在服务器上运行一个，并应答
  结果，或随其发生而流式传输其步骤。离开的客户端不会
  停止运行；它会运行到结束并保留其报告。
- 在 CI 中：[GitHub Actions 及其他](../automation/ci.md)。

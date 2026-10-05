---
title: 负载测试
description: 按负载模式发送 HTTP 节点的请求——恒定、斜坡、阶梯、尖峰或随机到达——测量延迟、错误和实际达到的速率，用阈值判定结果，并对比两次运行。
---

# HTTP 请求的负载测试

[[ui:exp.node.http]] 节点可以按每秒请求数的负载模式多次发送其请求，同时发出多个，并测量返回的结果：延迟百分位数、错误、达到的速率。阈值决定步骤是否通过，[[ui:exp.compare]] 则把这些数字与之前某次运行的数字并排列出。

负载是节点的一项设置，而不是单独的节点：实验的其余部分（模拟器、损伤中继、其他分支）照常在它周围运行。

## 让请求承受负载 {#turn-on}

1. 选择一个 [[ui:exp.node.http]] 节点，填写其请求。
2. 在其属性中，开启 [[ui:exp.loadOn]]。
3. 选择 [[ui:exp.loadShape]] 并填写相应的数值。下方的图表 [[ui:exp.loadChart]] 会绘出速率，并显示合计多少个请求、持续多少秒。
4. 设置 [[ui:exp.loadConcurrency]]，即同时可以有多少个请求在途。
5. 添加或修改 [[ui:exp.thresholds]]。
6. 运行实验。

负载的初始设置是 [[ui:exp.loadShape.ramp]]：在 30,000 ms 内从每秒 0 个请求增加到 100 个，并发 32 个，带两个阈值：[[ui:exp.metric.p95_ms]] < 500 ms 和 [[ui:exp.metric.error_rate]] < 1%。

**负载取代重复和重试**。开启负载会关闭这两项，同时带有负载和其中任一项的节点会被拒绝（`node.load_alone`）：失败的请求只计数，不会重试。只有 HTTP 请求可以在负载下运行（`node.load_unsupported`）。

**请求只读取一次**。其模板在步骤开始时解析，因此负载中的每个请求都完全相同：`{{counter}}` 和 `{{uuid}}` 对所有请求都取同一个值。参见[模板](data.md#templates)。

**整个负载只用一个客户端**。当 [[ui:exp.cookies]] 开启时，这些请求共用本次运行的 Cookie 存储，并共用同一份 Digest 认证状态，因此一次质询即可用于所有请求。每个请求都使用节点自己的超时。

**检查器只收到样本**：每 100 ms 最多一次交互，因此负载不会淹没 [[ui:dock.inspector]]。

## 负载模式 {#profiles}

| [[ui:exp.loadShape]] | 设置 | 速率随时间的变化 |
| --- | --- | --- |
| [[ui:exp.loadShape.constant]] | [[ui:exp.loadRate]]、[[ui:exp.loadDuration]] | 始终保持该速率 |
| [[ui:exp.loadShape.ramp]] | [[ui:exp.loadFrom]]、[[ui:exp.loadTo]]、[[ui:exp.loadDuration]] | 从一个速率沿直线变化到另一个速率 |
| [[ui:exp.loadShape.steps]] | [[ui:exp.loadFrom]]、[[ui:exp.loadStepBy]]、[[ui:exp.loadEvery]]、[[ui:exp.loadSteps]] | 先是起始速率，此后每一级增加一个步长，每一级持续相同的时间 |
| [[ui:exp.loadShape.spike]] | [[ui:exp.loadBase]]、[[ui:exp.loadPeak]]、[[ui:exp.loadAt]]、[[ui:exp.loadSpikeFor]]、[[ui:exp.loadDuration]] | 基础速率，从给定时刻起保持一段时间的峰值，然后回到基础速率 |
| [[ui:exp.loadShape.poisson]] | [[ui:exp.loadRate]]、[[ui:exp.loadDuration]] | 随机到达，平均为该速率 |

切换模式时会保留能够沿用的设置：持续多久，以及达到的最高速率。

### 限制 {#limits}

| 设置 | 范围 |
| --- | --- |
| [[ui:exp.loadShape.constant]] 和 [[ui:exp.loadShape.poisson]] 的 [[ui:exp.loadRate]]，[[ui:exp.loadPeak]] | 0.1–100,000 请求/s |
| [[ui:exp.loadFrom]]、[[ui:exp.loadTo]]、[[ui:exp.loadBase]] | 0–100,000 请求/s |
| [[ui:exp.loadShape.steps]] 的每一级，包括最后一级 | 0–100,000 请求/s；步长可以为负 |
| [[ui:exp.loadDuration]]、[[ui:exp.loadEvery]] | 100–300,000 ms |
| [[ui:exp.loadSteps]] | 1–100，且所有级合计最多 300,000 ms |
| 尖峰 | 长于 0 ms，并在时长结束前结束 |
| [[ui:exp.loadConcurrency]] | 1–512 |
| [[ui:exp.thresholds]] | 最多 16 个，每个值为大于或等于 0 的数字 |

合计没有任何请求的负载模式会被拒绝（`load.nothing_planned`）。速率范围与 [HTTP 突发负载](../protocols/http.md)相同。

负载模式最长可以持续一整次运行的时长，即 300 s——但运行的[时间限制](flow.md#limit)会计入每个步骤，因此请为实验的其余部分留出时间。

### 请求数 {#planned}

负载模式的请求数就是其速率随时间的累计：

| 负载模式 | 请求数 |
| --- | --- |
| [[ui:exp.loadShape.constant]]，100/s，持续 1000 ms | 100 |
| [[ui:exp.loadShape.ramp]]，0 → 100/s，持续 2000 ms | 100 |
| [[ui:exp.loadShape.steps]]，从 10/s 起每级增加 10/s，3 级，每级 1000 ms | 60（10 + 20 + 30） |
| [[ui:exp.loadShape.spike]]，10/s，从 1000 ms 起以 100/s 持续 500 ms，总计 2000 ms | 65 |
| [[ui:exp.loadShape.poisson]]，200/s，持续 10,000 ms | 平均 2000 |

## 调度 {#schedule}

第 n 个请求在负载模式的累计数达到 n 的时刻到期，第一个请求立即发出。每个时刻都从负载开始时算起，因此一次延迟的唤醒绝不会推迟其后的请求，负载模式所描述的速率就是所要求的速率。

[[ui:exp.loadShape.poisson]] 从本次运行的种子中随机抽取到达间隔：相同的种子给出相同的时刻，因此随机负载也可以精确重现。参见[种子](runs.md#seeds)。

**错过的请求**。同时在途的请求最多为 [[ui:exp.loadConcurrency]] 个。当它们全都还在等待应答时，下一个请求会等待空闲的并发槽。如果它的发出时间会比预定时刻晚 50 ms 以上，就不会延迟发送：它会被跳过并计为错过，期间到期的其他所有请求也是如此，负载随后从第一个仍然准时的请求继续。错过的请求很多，说明服务器或 [[ui:exp.loadConcurrency]] 跟不上负载模式。

## 运行期间 {#progress}

时间线每秒一次将该步骤显示为 [[ui:exp.load]]，并给出已过的秒数、已发送的请求数、最近一秒的速率、目前为止的 p95 以及失败的请求数。[[ui:common.stop]] 会立即结束负载，并放弃在途的请求；另一个分支中的失败会在一秒内结束负载。

## 测量内容 {#metrics}

最后一个应答之后，步骤就有了测量结果，保存在其最后一个时间线事件和[运行报告](runs.md#report)中：

| 测量项 | 内容 |
| --- | --- |
| planned | 负载模式合计的请求数（[[ui:exp.loadShape.poisson]]：平均值） |
| sent | 已得到应答或已失败的请求 |
| ok | 以 2xx 状态应答的请求 |
| failed | 任何其他状态，或根本没有应答 |
| missed | 在所有并发槽都忙时到期而被跳过的请求 |
| rps | 每秒发送的请求数：sent ÷ 负载模式的时长；如果最后一个请求发出得更晚，则 ÷ 到最后一个请求发出为止的时间 |
| error_rate | failed 占 sent 的百分比 |
| min, mean, max | 最快、平均和最慢的请求，ms |
| p50, p90, p95, p99 | 50%、90%、95% 和 99% 的请求不超过的延迟，ms |
| received_bytes | 收到的正文字节总数 |
| statuses | 按状态（`200`、`503`）统计的请求数，没有状态时按原因（`timeout`、`refused`、`reset` …）统计 |
| seconds | 负载模式的每一秒：发送的请求数、失败的请求数及其平均延迟 |
| histogram | 按延迟统计的请求数：不超过 1、2、5、10、20、50、100、200、500、1000、2000、5000、10,000 ms，以及更慢 |

请求的延迟从发送它开始，到读完整个应答为止；失败的请求按其失败所用的时间计算。百分位数从宽度为 1% 的对数分桶中读取，无论负载运行多久，与真实值的误差都在 0.5% 以内。

## 阈值 {#thresholds}

阈值是由 [[ui:exp.thresholdMetric]]、[[ui:exp.thresholdOp]] 和 [[ui:exp.thresholdValue]] 组成的一行；[[ui:exp.thresholdAdd]] 用于添加一个阈值。

| [[ui:exp.thresholdMetric]] | 单位 |
| --- | --- |
| [[ui:exp.metric.p50_ms]]、[[ui:exp.metric.p90_ms]]、[[ui:exp.metric.p95_ms]]、[[ui:exp.metric.p99_ms]] | ms |
| [[ui:exp.metric.mean_ms]]、[[ui:exp.metric.max_ms]] | ms |
| [[ui:exp.metric.error_rate]] | 占已发送请求的 % |
| [[ui:exp.metric.rps]] | 实际达到的每秒请求数 |
| [[ui:exp.metric.missed]] | 请求数 |

[[ui:exp.thresholdOp]] 为 `<`、`≤`、`>`、`≥` 之一。几个常见的例子：

| [[ui:exp.thresholdMetric]] | [[ui:exp.thresholdOp]] | [[ui:exp.thresholdValue]] | 步骤失败的条件 |
| --- | --- | --- | --- |
| [[ui:exp.metric.p95_ms]] | `<` | 300 | 每二十个请求中有一个或更多用时 300 ms 或更长 |
| [[ui:exp.metric.error_rate]] | `<` | 1 | 1% 或更多的请求失败 |
| [[ui:exp.metric.rps]] | `≥` | 180 | 服务器无法每秒处理 180 个请求 |
| [[ui:exp.metric.missed]] | `≤` | 0 | 哪怕只有一个请求不得不被跳过 |

在文件中，阈值写作 `{ "metric": "p95_ms", "op": "lt", "value": 300 }`；指标有 `p50_ms`、`p90_ms`、`p95_ms`、`p99_ms`、`mean_ms`、`max_ms`、`error_rate`、`rps` 和 `missed`，比较方式有 `lt`、`le`、`gt` 和 `ge`。

阈值在最后一个应答之后按顺序读取。步骤会在第一个不满足的阈值处失败（`load.threshold`），其消息给出该阈值和测得的值，运行也随之失败。没有阈值时，无论测得什么，负载都会通过。如果另一个分支的失败提前结束了负载，运行的失败就归于那个失败，而不是某个阈值。

## 结果 {#result}

步骤通过时，时间线会给出汇总：请求数、速率、p95 和失败比例。选中该节点，其属性会显示 [[ui:exp.loadResult]]：

- 每个阈值，✓ [[ui:exp.thresholdHeld]] 或 ✕ [[ui:exp.thresholdBroken]]，附带测得的值；
- [[ui:http.sent]]、[[ui:exp.loadRps]]、[[ui:exp.loadErrors]] 及其比例、[[ui:http.missed]]；
- [[ui:http.p50]]、[[ui:http.p90]]、[[ui:http.p95]]、[[ui:http.p99]]、[[ui:http.avg]]、[[ui:http.max]]；
- [[ui:exp.loadPerSecond]]：每一秒的请求，失败的以红色显示，平均延迟以折线显示；
- [[ui:exp.loadLatencies]]：多少请求用了多长时间；
- 各个状态和原因，以及各自的计数。

命令行会输出同样的数字和每个阈值的判定结果；参见 [`signallab run`](../automation/cli.md#cli-run)。

## 对比两次运行 {#compare}

1. 运行实验两次或更多次。
2. 在时间线中按 [[ui:exp.compare]]。有运行保存了报告后它就会出现，运行进行中时不可用。
3. 最近一次运行作为 [[ui:exp.compareAfter]]，前一次作为 [[ui:exp.compareBefore]]；两个列表都可以选择其他运行。

列表中包含此实验（按名称识别）的运行，取自数据文件夹中的报告，最新的在最前，最多 50 个：每个都带有日期和时间、结束方式及其种子。如果命令行使用了同一个数据文件夹，命令行的运行也会在其中。重命名实验会开始新的历史记录。

对于每个负载步骤（按节点匹配），一个表格会显示每个指标的 [[ui:exp.compareBefore]] 值、[[ui:exp.compareAfter]] 值和 [[ui:exp.compareChange]]，以单位和百分比表示。朝不利方向变化 5% 或更多（更慢、更多错误、更多错过的请求、更低的速率）属于性能退化，以红色显示；从无到有也算。表格下方是每个阈值在两次运行中的判定结果。只有其中一次运行包含的负载步骤会标记为 [[ui:exp.compareOnlyBefore]] 或 [[ui:exp.compareOnlyAfter]]，不显示变化。没有负载步骤的运行显示 [[ui:exp.compareNoLoad]]。

在脚本中，[`experiment_runs`](../api/commands.md#experiment_runs) 列出运行，[`experiment_compare`](../api/commands.md#experiment_compare) 按报告的文件名对比两次运行；`signallab mcp` 为 AI 助手提供同样的功能（[MCP](../automation/mcp.md)）。

## 负载之后的检查 {#checks-after}

负载本身不会留下响应：它只被测量，不被检查。负载之后的检查或 [[ui:exp.node.extract]]，需要在每条路径上都有一个不带负载的请求位于其前，否则实验无法运行（`graph.needs_http`）。要检查负载下 API 的某一次应答，请在负载之后放一个普通的 [[ui:exp.node.http]]，或将其放在旁边的并行分支中。

在负载下的节点上，[[ui:exp.sendNow]] 只发送一次其请求。

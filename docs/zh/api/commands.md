---
title: 命令
description: Signal Lab 引擎的每一条命令，以 POST /api/invoke/<command> 调用，并给出其参数、结果和错误。
---

# 命令

引擎拥有的每一条命令，按所作用的对象分组。每条命令以
`POST /api/invoke/<command>` 调用，带一个 JSON 参数对象，并以 `200`
返回其结果，或以 `422` 返回一个
[`EngineError`](index.md#errors)。如何认证以及各状态码的含义见
[API 概览](index.md)。

## 约定 {#conventions}

- **参数名**采用 camelCase（`jobId`、`nodeId`）。命令不认识的参数，或遗漏
  的必填参数，会以 `command.args_invalid` 被拒绝；每一条带参数的
  命令都可能因此失败。没有参数的命令不读取请求体。
- **作为参数传入的对象**——`config`、`request`、`document`、
  `library`、`emulator`、`profile`——使用引擎自己的字段名，大多是
  snake_case（`timeout_ms`）。在它们内部，引擎不认识的字段会
  **被忽略**，因此拼错的可选字段会悄悄保留其默认值。只有
  `feedback_send` 的表单会拒绝未知字段。
- **可选**参数和字段可以省略，或作为 `null` 发送；
  表格给出它们的默认值。
- **结果**是 JSON。“null”表示命令没有任何要返回的内容。
- **地址**写作 `IP:port` 时接受数字地址和端口
  （`127.0.0.1:9000`、`[::1]:9000`）；其中的主机名会被拒绝。当表格
  写作 `IP:port` 或 `host:port` 时，主机名也可以：它会在
  命令运行时被解析，并在拥有 IPv4 地址时使用该地址
  （因此 `localhost:9000` 就是 `127.0.0.1:9000`）。
- **任务**：标记为 *启动任务* 的命令返回一个
  [`JobInfo`](#type-jobinfo)；工作会持续到它结束，或被
  [`job_stop`](#job_stop) 停止。参见[任务](index.md#jobs)。
- **结果中的路径**位于引擎所运行的机器上——在服务器上，
  就在其数据文件夹内；用 [`/api/files`](index.md#files) 下载它们。

示例使用这个 shell 函数：

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
invoke() {
  curl -sS -X POST "$SERVER/api/invoke/$1" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    --data "${2:-}"
}
```

## 应用 {#application}

### app_info {#app_info}

引擎是什么，以及它在哪里运行。没有参数。

**结果**

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `version` | string | Signal Lab 的版本，`[[version]]` |
| `mode` | string | `desktop` 或 `server` |
| `secrets_writable` | boolean | [`secret_set`](#secret_set) 和 [`secret_delete`](#secret_delete) 能否在这里工作：服务器上为 `false` |
| `data_dir` | string | 数据文件夹，位于引擎所运行的机器上 |
| `os` | string | `windows`、`linux`…… |
| `arch` | string | `x86_64`、`aarch64`…… |

### get_host_info {#get_host_info}

这台机器的名称，以及它发送时将使用的地址。没有参数。

**结果**：`{ "local_ip": string, "hostname": string }`。`local_ip` 是
系统为发往互联网的流量所挑选的 IPv4 地址（无需发送任何内容即可找到），
没有时为 `127.0.0.1`。`hostname` 是计算机的名称，系统未提供时为
`localhost`。

### firewall_status {#firewall_status}

系统防火墙是否允许其他机器访问本程序。只有 Windows 有可按程序
读取的防火墙；其他地方 `applies` 为 `false`，其余字段中只有 `program`
被填写。没有参数。

**结果**

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `applies` | boolean | 这里有按程序设置的防火墙（Windows） |
| `program` | string | 这些规则所针对的程序 |
| `enabled` | boolean | 防火墙对机器当前所在的网络已开启 |
| `networks` | string[] | 机器所在的网络类型：`domain`、`private`、`public` |
| `allowed` | boolean | 在当前网络上，有一条入站规则允许本程序的 UDP 进入 |
| `blocked` | boolean | 在当前网络上，有一条入站规则阻止本程序；它优先于任何允许规则 |
| `rules` | number | 本程序的入站规则数量，含各种类型 |

**错误**：`firewall.failed`。

### firewall_allow {#firewall_allow}

让其他机器能够访问 Signal Lab：系统会显示自己的管理员提示，
随后本程序的入站规则（包括阻止规则）会被替换为分别针对 Signal Lab
和它旁边的 `signallab` 命令行的一条允许规则。仅限 Windows 上的桌面
应用；服务器会拒绝，因为它的屏幕前没有人来应答提示。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `public` | boolean | 是 | 在公共网络上也允许，而不只是专用和域网络 |

**结果**：新的 [`firewall_status`](#firewall_status)。

**错误**：`firewall.server`（在服务器上）、`firewall.unsupported`
（不是 Windows）、`firewall.declined`（提示被应答为“否”）、
`firewall.failed`。

### feedback_send {#feedback_send}

通过工作室的 hub 向 Signal Lab 的开发者发送一条消息，由 hub 邮寄给他们。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `form` | object | 是 | 消息，见下文。未知字段会被拒绝 |

| `form` 的字段 | 类型 | 默认值 | 含义 |
| --- | --- | --- | --- |
| `message` | string | — | 发生了什么；必填，最多 20 000 个字符 |
| `email` | string | 无 | 回复可以发往哪里 |
| `meta` | object of strings | `{}` | 应用对自身的说明（version、os、arch、mode、lang、screen） |
| `screenshots` | `{ name, data }[]` | `[]` | 图片，`data` 为 base64；最多 6 张，每张 8 MiB |
| `logs` | `{ name, text }[]` | `[]` | 文本文件；最多 4 个，每个 2 MiB |

全部内容合计最多 15 MiB。

**结果**：`{ "id": string }`，开发者收到的编号。

**错误**：`feedback.message_required`、`feedback.message_too_long`、
`feedback.too_many_files`、`feedback.file_too_large`、`feedback.too_large`、
`feedback.invalid`，hub 的拒绝（`feedback.email_invalid`、
`feedback.file_type`、`feedback.rate_limited`、`feedback.disabled`、
`feedback.send_failed`、`feedback.failed`），以及网络的 `transport.*`。

```bash
invoke app_info
# {"version":"[[version]]","mode":"server","secrets_writable":false,"data_dir":"/data","os":"linux","arch":"x86_64"}
```

## 任务 {#jobs}

### jobs_list {#jobs_list}

正在运行的任务，最旧的在前。没有参数。

**结果**：[`JobInfo`](#type-jobinfo)`[]`。

### job_stop {#job_stop}

立即停止一个任务：它的套接字关闭，它的中继、服务器或连接随之消失。
被停止的任务不发送 [`job://ended`](events.md#event-job-ended)；被停止的
运行不保存报告。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `id` | number | 是 | 任务的 `id` |

**结果**：该 id 的任务正在运行时为 `true`，否则为 `false`。

### jobs_stop_all {#jobs_stop_all}

停止每一个正在运行的任务，无论由谁启动。没有参数。

**结果**：null。

```bash
invoke jobs_list
# [{"id":3,"kind":"osc-monitor","label":"OSC monitor 0.0.0.0:9000","params":{"bind":"0.0.0.0:9000"},"started_ms":1759600000000}]
invoke job_stop '{"id":3}'
# true
```

## 实验与运行 {#experiments}

这些命令接收并返回一个实验文档（`Experiment`）：编辑器保存和导出的
JSON，含有 `version`、`name`、`params`、`profiles`、`profile`、`seed`、
`cookies`、`nodes` 和 `edges`。它的节点见[节点](../experiments/nodes.md)；
它的参数、配置和模板见[数据](../experiments/data.md)。文档最多 4 MiB
（`file.too_large`）。要运行一个实验并等待其结果，请使用
[`POST /api/run`](run.md)，而不是 [`experiment_start`](#experiment_start)。

### experiment_load {#experiment_load}

当前工作的实验：数据文件夹中的 `experiment.json`——在服务器上，就是
其界面所显示的那一个。没有时，为初始实验。较旧的文档版本会被迁移。
没有参数。

**结果**：`Experiment`。

**错误**：`file.io`、`file.json_invalid`（带有文件的 `path`、`line`
和 `column`）、`file.too_large`、`doc.version_unsupported`，以及其他
`doc.*` 检查。

### experiment_save {#experiment_save}

替换当前工作的实验，即数据文件夹中的 `experiment.json`。它会先写入
临时文件，因此写入失败会保留上一个文件。

::: warning
在服务器上，这就是每个浏览器的编辑器所操作的文档。
:::

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `document` | `Experiment` | 是 | 文档 |

**结果**：字符串，写入的路径。

**错误**：`doc.*`，文档的大小检查（`param.*`、`params.too_many`、
`profile.*`、`profiles.too_many`、`seed.range`）、`file.too_large`、`file.io`。

### experiment_parse {#experiment_parse}

从 JSON 文本读取实验，正如 [[ui:exp.importJson]] 所做的那样。版本 1
到 8 会迁移到版本 9，即当前版本；版本 8 之前的文件打开时 `cookies`
为关闭，因此会像从前一样运行。字节顺序标记会被跳过。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `text` | string | 是 | 文件的文本 |

**结果**：`Experiment`。

**错误**：`file.json_invalid`（`line`、`column`）、`file.too_large`、
`doc.version_unsupported`、`doc.*`，以及 [`experiment_save`](#experiment_save)
的大小检查。

### experiment_export {#experiment_export}

将文档的快照写入数据文件夹中的
`exports/experiment-<ms>-<16 hex digits>.json`。每次导出都是一个新文件。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `document` | `Experiment` | 是 | 文档 |

**结果**：字符串，写入的路径。

**错误**：与 [`experiment_save`](#experiment_save) 相同。

### experiment_validate {#experiment_validate}

检查文档能否用其当前配置（或其默认值）运行：图、每个字段、参数，以及
它命名的每个机密都已存储。阻塞性的问题就是错误。成功时，它会指出
*其他*配置中有哪些会失败，让您在切换前就能知道。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `document` | `Experiment` | 是 | 文档 |
| `overrides` | object of strings | 否 | 仅用于本次检查的参数值，按 [[ui:exp.runWith]] 给出的形式 |

**结果**：`{ "profile": string or null, "error": EngineError }[]`——每一个
无法通过验证的其他配置（`null`：默认值，不带配置）。空列表表示每个
配置都没问题。

**错误**：任何验证代码（`doc.*`、`graph.*`、`node.*`、`param.*`、
`profile.*`、`template.*`、`loop.*`……）、`run.override_unknown`（为文档
没有的参数提供了覆盖值）、`secret.missing`、`secret.store`、
`secret.unsupported`。

### experiment_resolve {#experiment_resolve}

一个节点的模板填充后的样子，正如编辑器的预览所显示：当前配置的值，
以及您给出的变量值。机密显示为 `••••`，绝不显示其值。没有值的名称
保持原样，并被列出。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `document` | `Experiment` | 是 | 文档 |
| `nodeId` | string | 是 | 节点 |
| `vars` | object | 是 | 要使用的变量值，按名称给出；没有则为 `{}` |

**结果**：`{ "node": node, "missing": string[] }`。

**错误**：`node.not_found`、`template.*`、`secret.store`、
`secret.unsupported`。

### experiment_send_node {#experiment_send_node}

[[ui:exp.sendNow]]：单独执行一个节点，走的是运行所用的同一段代码。
动作会被发送；等待从现在起监听，直到匹配或超时。[[ui:exp.node.ws_send]]
或 [[ui:exp.node.wait_ws]] 节点会打开其 [[ui:exp.node.ws_connect]] 节点所
描述的连接。发送时不带任何 Cookie，本次运行的 [[ui:exp.node.impairment]]
和 [[ui:exp.node.emulator]] 节点也不会打开。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `document` | `Experiment` | 是 | 文档；其当前配置给出参数值 |
| `nodeId` | string | 是 | 一个动作或一个等待 |
| `vars` | object | 是 | 节点模板读取的变量值；没有则为 `{}` |

**结果**

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `detail` | string | 发生了什么，用英文 |
| `response` | [`HttpResponse`](#type-httpresponse) or null | HTTP 节点的响应 |
| `vars` | object | 该步骤设置的内容：等待的回复，或请求之后 [[ui:exp.node.extract]] 节点从其响应中取走的内容 |

机密值在这一切中都被遮蔽。

**错误**：`node.not_found`、`run.not_an_action`（不是动作也不是
等待）、`ws.connection_unknown`、`secret.missing`、`template.*`，以及该
步骤失败时的任何错误：`transport.*`、`wait.timeout`、`check.*`……

### experiment_start {#experiment_start}

启动一次运行，正如 [[ui:exp.run]] 所做的那样，并立即返回。它的步骤以
[`experiment://step`](events.md#event-experiment-step) 事件到达，结束以
[`experiment://ended`](events.md#event-experiment-ended) 到达，其报告
保存在数据文件夹的 `runs/` 下。等待、模拟器、损伤中继和 MQTT 订阅都在
第一个步骤之前打开，因此已被占用的端口会在这里失败。超过 300 s 的运行
会以 `run.timeout` 失败。*启动任务*（`experiment`）。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `document` | `Experiment` | 是 | 文档 |
| `overrides` | object of strings | 否 | 仅用于本次运行的参数值 |
| `seed` | number | 否 | 本次运行的种子，0 到 9007199254740991；默认：文档的，否则为新的一个 |

**结果**：[`JobInfo`](#type-jobinfo)，`params.name` 为实验的名称。

**错误**：[`experiment_validate`](#experiment_validate) 报告的一切、
`seed.range`、`transport.address_in_use` 以及其他绑定失败、`emulator.*`、
`impair.*`、`node.params_only`（一个监听地址，或一个 MQTT 等待的代理或
主题，在运行开始时未固定），以及 MQTT 等待无法连接的代理的
[`mqtt_connect`](#mqtt_connect) 错误。

### experiment_runs {#experiment_runs}

从 `runs/` 中的报告读回的运行，最新的在最前。无法读取的报告会被略过。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `name` | string | 否 | 只包含名称与之完全相同的实验的运行 |
| `limit` | number | 否 | 最多这么多个；默认 50，最多 500 |

**结果**：运行摘要：

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `name` | string | 报告的文件名，`run-<ms>-<job>.json`：[`experiment_compare`](#experiment_compare) 所接收的内容 |
| `experiment` | string | 实验的名称 |
| `started_ms`, `ended_ms` | number | 自 1970 年以来的毫秒数 |
| `outcome` | string | `passed` 或 `failed` |
| `seed` | number | 本次运行的种子 |
| `profile` | string or null | 它的配置 |
| `loads` | object[] | 每个负载步骤：`node`、`sent`、`rps`、`p95_ms`、`error_rate`、`held`（每个阈值都保持住） |

**错误**：`file.io`。

### experiment_compare {#experiment_compare}

两次运行并排比较，逐负载步骤，正如时间线的 [[ui:exp.compare]] 所显示。
步骤按节点 id 匹配。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `a` | string | 是 | 较早一次运行的报告文件名 |
| `b` | string | 是 | 较晚一次运行的报告文件名 |

**结果**：`{ "a": summary, "b": summary, "steps": [...] }`，每个步骤带有
`node`、`missing_in`（`a` 或 `b`，当只有一次运行包含它时）、`metrics`、
`sent`（`[a, b]`）、`thresholds_a` 和 `thresholds_b`（每个阈值写作
`{ metric, op, value, actual, held }`）。`metrics` 列出九个指标，每个写作
`{ metric, a, b, change, percent, worse }`：`metric` 为 `p50_ms`、`p90_ms`、
`p95_ms`、`p99_ms`、`mean_ms`、`max_ms`、`error_rate`、`rps` 或 `missed`；
`change` 为 `b − a`；`percent` 为相对 `a` 的变化百分比（`a` 为 0 时为
null）；`worse` 表示它朝不利方向变化——更高，或对 `rps` 而言更低——幅度
达到 5% 或更多，或从 0 变为任何值。只有一次运行包含的步骤永远不会
`worse`。参见[负载](../experiments/load.md)。

**错误**：`runs.name_invalid`（除报告文件名外的任何内容，不含文件夹）、
`runs.not_found`、`file.json_invalid`、`file.io`。

```bash
invoke experiment_validate "$(jq '{document: .}' experiment.json)"
# []
```

## 机密 {#secrets}

机密值由实验以 `{{secret.NAME}}` 使用，绝不离开引擎：没有命令会返回
它们。它们存放的位置取决于引擎在哪里运行：

| 位置 | 存储 | 设置和删除 |
| --- | --- | --- |
| 桌面应用，Windows | Windows 凭据管理器 | 是 |
| 桌面应用，Linux | 无 | `secret.unsupported` |
| 服务器 | `SIGNALLAB_SECRET_<NAME>`，或 `--secrets-dir`（默认 `/run/secrets/signallab`）中的文件 `<NAME>` | 否：`secret.read_only` |

名称以拉丁字母或 `_` 开头，其后是拉丁字母、数字和 `_`，最多 128 个
字符（`secret.name_invalid`）。

### secret_status {#secret_status}

给定的名称中哪些已存储值。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `names` | string[] | 是 | 要查找的名称 |

**结果**：一个对象，名称 → `true`（已存储）或 `false`。

**错误**：`secret.name_invalid`（在服务器上）、`secret.store`、
`secret.too_large`（服务器上的文件超过 16 KiB）、`secret.unsupported`。

### secret_set {#secret_set}

在某个名称下存储一个值，替换原有的值。仅限桌面应用。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `name` | string | 是 | 名称 |
| `value` | string | 是 | 非空；最多 16 KiB |

**结果**：null。

**错误**：`secret.read_only`（在服务器上）、`secret.unsupported`、
`secret.name_invalid`、`secret.empty`、`secret.too_large`、`secret.store`。

### secret_delete {#secret_delete}

移除一个已存储的值。移除未存储的值不算错误。仅限桌面应用。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `name` | string | 是 | 名称 |

**结果**：null。

**错误**：`secret.read_only`（在服务器上）、`secret.unsupported`、
`secret.name_invalid`、`secret.store`。

```bash
invoke secret_status '{"names":["API_TOKEN","MQTT_PASSWORD"]}'
# {"API_TOKEN":true,"MQTT_PASSWORD":false}
```

## OSC {#osc}

这些命令所服务的界面见 [OSC](../protocols/osc.md)。

### osc_send {#osc_send}

从一个新建的套接字，用一条 UDP 数据报发送一条 OSC 消息。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `target` | string | 是 | 要发往的 `IP:port` 或 `host:port`；名称会被解析，有 IPv4 地址时使用该地址 |
| `address` | string | 是 | OSC 地址，`/mixer/fader/1`；它以 `/` 开头 |
| `args` | [`OscArg`](#type-oscarg)`[]` | 是 | 参数；没有则为 `[]` |

**结果**：数字，发送的字节数。

**错误**：`node.osc_address`（没有开头的 `/`；字段 `address`）、
`transport.target_invalid`（没有端口，或两种形式都不是）、`transport.dns`
（名称无法解析）、`transport.*`。

### osc_monitor_start {#osc_monitor_start}

在一个 UDP 端口上监听 OSC 并解码每个数据包。每一个都作为
[`osc://message`](events.md#event-osc-message) 事件到达。*启动任务*
（`osc-monitor`、`params.bind`）。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `bind` | string | 是 | 要监听的 `IP:port`：`0.0.0.0:9000` 为每块网卡，`127.0.0.1:9000` 仅本机 |

**结果**：[`JobInfo`](#type-jobinfo)。

**错误**：`node.bind_invalid`、`transport.address_in_use`、
`transport.address_unavailable`、`transport.denied`、`wait.bind_failed`。
如果套接字无法再接收，任务会以 `wait.receive_failed` 结束。

### osc_generator_start {#osc_generator_start}

发送一串 OSC 消息，其唯一的参数遵循某种波形。进度作为
[`osc://gen-tick`](events.md#event-osc-gen-tick) 到达。*启动任务*
（`osc-gen`、`params.target`、`params.address`）。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `config` | object | 是 | 见下文 |

| `config` 的字段 | 类型 | 默认值 | 含义 |
| --- | --- | --- | --- |
| `target` | string | — | 要发往的 `IP:port` 或 `host:port`；名称在任务启动时解析一次 |
| `address` | string | — | OSC 地址；它以 `/` 开头 |
| `rate` | number | — | 每秒消息数，保持在 0.1 到 5000 之间 |
| `waveform` | string | — | `sine`、`triangle`、`saw`（下降：从 `max` 到 `min`，然后立刻返回）、`ramp`（上升：从 `min` 到 `max`，然后立刻返回）、`square`、`random` 或 `constant`（`max`） |
| `freq` | number | — | 每秒的波形周期数 |
| `min`, `max` | number | — | 值的范围 |
| `as_int` | boolean | `false` | 取整并发送 int 而不是 float |
| `duration_s` | number | `0` | 这么多秒后停止；0 表示一直运行到被停止 |

**结果**：[`JobInfo`](#type-jobinfo)。

**错误**：`node.osc_address`、`transport.target_invalid`、`transport.dns`、
`transport.*`。如果某次发送失败，任务会以 `transport.*` 错误结束。

```bash
invoke osc_send '{"target":"127.0.0.1:9000","address":"/cue/go","args":[{"type":"int","value":1}]}'
# 16
invoke osc_monitor_start '{"bind":"0.0.0.0:9000"}'
```

## HTTP 与 Cookie {#http}

参见 [HTTP](../protocols/http.md)。

### http_request {#http_request}

发送一条 HTTP 请求并返回响应。没有收到响应的请求——被拒绝、超时、名称
无法解析、证书不受信任——**不是**命令的错误：响应会在 `error` 和
`cause` 中说明。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `request` | [`HttpRequest`](#type-httprequest) | 是 | 请求 |
| `cookies` | boolean | 否 | 发送 [[ui:nav.http]] 界面的 Cookie 存储，并保留应答设置的内容；默认 `false` |

**结果**：[`HttpResponse`](#type-httpresponse)。

**错误**：`http.client_failed`（请求甚至无法准备）。

### http_burst_start {#http_burst_start}

多次发送同一条请求，同时发出若干条，并对其进行测量。没有 `rate` 时，
每个工作者一拿到应答就再发一次；有 rate 时，无论应答多慢，请求都按
固定的时间表开始，而一条请求若为了等待空闲工作者而超过其时刻 50 ms
以上，就会被跳过并计为错过。进度作为
[`http://burst-progress`](events.md#event-http-burst-progress) 每秒到达
十次。*启动任务*（`http-burst`、`params.method`、`params.url`，限速时还有
`params.rate`）。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `config` | object | 是 | [`HttpRequest`](#type-httprequest) 的字段与下面这些字段，合在一个对象中 |

| `config` 的字段 | 类型 | 默认值 | 含义 |
| --- | --- | --- | --- |
| `concurrency` | number | — | 最多这么多条在途，保持在 1 到 512 之间 |
| `total` | number | `0` | 这么多条请求后停止；0：不限数量 |
| `duration_s` | number | `0` | 这么多秒后停止；0：不限时间 |
| `rate` | number | `0` | 每秒开始的请求数，0.1 到 100 000；0：应答来得多快就发多快 |
| `cookies` | boolean | `false` | 使用 [[ui:nav.http]] 界面的 Cookie 存储 |

既不设 `total` 也不设 `duration_s` 时，突发会一直运行到被停止。

**结果**：[`JobInfo`](#type-jobinfo)。

**错误**：`http.rate_invalid`、`http.duration_invalid`、`http.client_failed`。

### http_cookies {#http_cookies}

[[ui:nav.http]] 界面的 Cookie 存储：每一个尚未过期的 Cookie。在服务器
上，每个页面和脚本各有一个存储。没有参数。

**结果**：Cookie，每个带有 `name`、`value`、`domain`、`host_only`（没有
Domain 属性：只有设置它的主机才能拿回它）、`path`、`expires`（Unix 秒，
会话 Cookie 为 null）、`secure`、`http_only` 和 `same_site`（字符串或
null）。

### http_cookies_clear {#http_cookies_clear}

清空 [[ui:nav.http]] 界面的 Cookie 存储。没有参数。

**结果**：null。

```bash
invoke http_request '{"request":{"method":"GET","url":"http://127.0.0.1:8080/health","headers":[["Accept","application/json"]],"body":null,"timeout_ms":5000}}' \
  | jq '{status, latency_ms, body}'
```

## WebSocket {#websocket}

参见 [WebSocket](../protocols/websocket.md)。`ws_connect` 打开的连接是
一个任务；其他命令用 `jobId` 指定它。

### ws_connect {#ws_connect}

打开一个 WebSocket 并保持打开。到达的内容和发送的内容每 100 ms 作为
[`ws://messages`](events.md#event-ws-messages) 到达；连接的状态作为
[`ws://state`](events.md#event-ws-state)。*启动任务*（`websocket`、
`params.url`）。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | 是 | 在哪里、如何连接 |

**结果**：[`JobInfo`](#type-jobinfo)。

**错误**：`ws.url_invalid`、`ws.header_invalid`、`ws.protocol_invalid`、
`ws.handshake_status`（服务器用另一个状态应答了升级）、
`ws.subprotocol_refused`、`ws.handshake_failed`、`transport.*`。

### ws_send {#ws_send}

在一个已打开的连接上发送一条消息。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `jobId` | number | 是 | 连接的任务 |
| `message` | object | 是 | `{ "text": "…" }` 表示文本消息，或 `{ "hex": "de ad be ef" }` 表示二进制消息；二者恰好其一 |

**结果**：数字，发送的字节数。

**错误**：`ws.not_connected`、`ws.payload_required`（二者都没有或都
有）、`hex.invalid`（空的 `hex` 也算）、`node.too_long`（超过 16 MiB，
字段 `payload`；什么都不会发送，连接保持打开）、`ws.closed`、
`transport.*`（服务器停止读取 10 s 时为 `transport.timeout`）。

### ws_close {#ws_close}

通过关闭握手关闭一个连接，并最多等待 2 s 以获得服务器的应答；随后任务
结束。连接一旦结束，它的任务就消失了，再关闭它就是 `ws.not_connected`。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `jobId` | number | 是 | 连接的任务 |
| `code` | number | 否 | 1000，或应用自有的 3000 到 4999；默认 1000 |
| `reason` | string | 否 | 最多 123 字节；默认空 |

**结果**：`{ "code", "reason", "by", "error" }`——`by` 为 `client`、
`server` 或 `lost`；关闭没有携带代码时 `code` 为 1005，没有关闭帧时为
1006。

**错误**：`ws.close_code`、`node.too_long`、`ws.not_connected`。

### ws_exchange {#ws_exchange}

不涉及任务的一次交互：连接，若给出消息则发送，若要求则等待应答，关闭。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | 是 | 在哪里、如何连接 |
| `message` | object | 否 | `{ "text" }` 或 `{ "hex" }`，与 [`ws_send`](#ws_send) 相同 |
| `expect` | object | 否 | 要等待的内容：`mode`（`any`、`contains`、`regex`、`hex`；默认 `any`）、`pattern`（默认空）、`timeout_ms`（默认 2000） |

有 `expect` 但没有 `message` 时，连接后第一条匹配的消息即算数——一条
问候语。

**结果**：`{ "handshake", "sent", "reply", "closed" }`——`handshake` 为
`{ url, peer, local, protocol, ms }`；`sent` 为发送的字节数或 null；
`reply` 为 `{ kind, text, hex, bytes, json, ms }` 或 null（`json`：文本
应答解析后的结果，否则为 null；`ms`：自发送起，或未发送任何内容时自
连接起）；`closed` 与 [`ws_close`](#ws_close) 返回的相同。

**错误**：[`ws_connect`](#ws_connect) 和 [`ws_send`](#ws_send) 的那些，
`wait.timeout`（带有 `ms`、`unmatched` 和 `target`）、`regex.invalid` 和
`hex.invalid`（无法解析的 `pattern`）。

```bash
invoke ws_exchange '{"config":{"url":"ws://127.0.0.1:9001/"},"message":{"text":"{\"type\":\"ping\"}"},"expect":{"mode":"contains","pattern":"pong"}}' \
  | jq .reply.text
```

## MQTT {#mqtt}

基于明文 TCP 的 MQTT 3.1.1，QoS 0、1 和 2。参见 [MQTT](../protocols/mqtt.md)。

### mqtt_connect {#mqtt_connect}

连接到一个代理并保持连接。命令在代理接受连接（CONNACK）后返回，因此
错误的密码或已关闭的端口就是它的错误。消息每 100 ms 作为
[`mqtt://messages`](events.md#event-mqtt-messages) 到达；状态变化作为
[`mqtt://state`](events.md#event-mqtt-state)；已完成的 QoS 1/2 发布和
取消订阅作为 [`mqtt://ack`](events.md#event-mqtt-ack)。*启动任务*
（`mqtt`、`params.broker`、`params.client`）。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | 是 | 代理以及如何连接 |

**结果**：[`JobInfo`](#type-jobinfo)。

**错误**：`mqtt.client_id_required`、`transport.*`（被拒绝、不可达、
`dns`、6 s 后 `timeout`）、`mqtt.no_answer`（6 s 内没有 CONNACK）、
`mqtt.protocol`、`mqtt.refused_protocol`、`mqtt.refused_client_id`、
`mqtt.refused_unavailable`、`mqtt.refused_credentials`、
`mqtt.refused_not_authorized`、`mqtt.refused`。

### mqtt_publish {#mqtt_publish}

在一个已打开的连接上发布。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `jobId` | number | 是 | 连接的任务 |
| `topic` | string | 是 | 主题：非空，且不含 `+` 或 `#` |
| `payload` | string | 是 | 载荷，以 UTF-8 发送 |
| `qos` | number | 是 | 0、1 或 2（高于 2 按 2 发送） |
| `retain` | boolean | 是 | 请求代理保留它；空载荷加 `retain` 会清除一个保留值 |

**结果**：null。QoS 1 或 2 的发布稍后由
[`mqtt://ack`](events.md#event-mqtt-ack) 确认。

**错误**：`node.topic_wildcard`（字段 `topic`）和 `mqtt.topic_required`
（字段 `topic`），与 [`mqtt_publish_once`](#mqtt_publish_once) 相同——命令
在查找连接之前就会拒绝它们；`mqtt.not_connected`。

### mqtt_subscribe {#mqtt_subscribe}

让一个已打开的连接订阅若干过滤器。代理授予的内容作为
[`mqtt://state`](events.md#event-mqtt-state) 到达，带有
`state: "subscribed"`。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `jobId` | number | 是 | 连接的任务 |
| `filters` | `{ filter, qos }[]` | 是 | 至少一个；`qos` 默认为 0。`+` 和 `#` 是通配符 |

**结果**：null。

**错误**：`mqtt.filter_required`、`mqtt.not_connected`。

### mqtt_unsubscribe {#mqtt_unsubscribe}

让一个已打开的连接取消订阅若干过滤器。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `jobId` | number | 是 | 连接的任务 |
| `filters` | string[] | 是 | 至少一个 |

**结果**：null。代理的应答作为 [`mqtt://ack`](events.md#event-mqtt-ack)
到达，带有 `kind: "unsubscribed"`。

**错误**：`mqtt.filter_required`、`mqtt.not_connected`。

### mqtt_publish_once {#mqtt_publish_once}

连接，发布一条消息，等待其 QoS 所要求的确认（最多 6 s），断开。它带有
自己的连接，用自己的客户端 id——`client_id` 的前 12 个字符、`-o` 和一个
数字——因此绝不会把带有该 id 的活动连接从代理上顶掉。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | 是 | 代理；`subscribe` 不使用 |
| `topic` | string | 是 | 非空且不含 `+` 或 `#` |
| `payload` | string | 是 | 以 UTF-8 发送 |
| `qos` | number | 是 | 0、1 或 2（高于 2 按 2 发送） |
| `retain` | boolean | 是 | 请求代理保留它 |

**结果**：字符串，由 Signal Lab 写出的摘要：
`<topic> → <broker> · <bytes> B · qos<n>`，保留时为 ` retained`。

**错误**：`mqtt.topic_required`、`node.topic_wildcard`，以及
[`mqtt_connect`](#mqtt_connect) 的那些，但 `mqtt.client_id_required` 除外：
这里接受空的 `client_id`。

```bash
invoke mqtt_publish_once '{"config":{"host":"127.0.0.1","port":1883,"client_id":"lab"},"topic":"lab/lamp/set","payload":"ON","qos":1,"retain":false}'
# "lab/lamp/set → 127.0.0.1:1883 · 2 B · qos1"
```

## 广播、组播与发现 {#broadcast}

参见[广播与发现](../protocols/broadcast.md)。

::: danger
广播和子网遍历会触及一个网段的每一台主机。只在您负责的网络上发送。
:::

### broadcast_send {#broadcast_send}

向每个目标发送一条数据报，各一次。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `config` | object | 是 | 见下文 |

| `config` 的字段 | 类型 | 默认值 | 含义 |
| --- | --- | --- | --- |
| `mode` | string | — | `list`、`broadcast`、`multicast` 或 `sweep` |
| `target` | string | — | 按模式而定，见下文 |
| `port` | number | `0` | 端口，仅用于 `sweep` |
| `payload` | [`Payload`](#type-payload) | — | 每条数据报携带的内容 |
| `bind` | string | 任意 | 发送所用的本地 `IP:port`；空或 null：`0.0.0.0:0`（所有目标都是 IPv6 时为 `[::]:0`） |
| `ttl` | number | `1` | IP TTL，或组播跳数限制；1 到 255 |
| `multicast_loop` | boolean | `true` | 组播也会回到本机 |
| `rate`, `count`, `duration_s` | number | `0` | 仅用于 [`broadcast_beacon_start`](#broadcast_beacon_start) |

| `mode` | `target` |
| --- | --- |
| `list` | 以逗号、分号或换行分隔的 `IP:port` 或 `host:port` 条目（不以空格分隔）；名称会被解析，有 IPv4 地址时使用该地址 |
| `broadcast` | `255.255.255.255:port`，或以 `.255` 结尾的地址加其端口 |
| `multicast` | 224.0.0.0 到 239.255.255.255 之间的一个组加其端口 |
| `sweep` | 一个 CIDR 块，`192.0.2.0/24`：`port` 上每一个可用的主机；最多 1024 台主机，因此为 `/22` 或更窄 |

**结果**

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `targets` | number | 目标数 |
| `packets`, `bytes` | number | 发出的量 |
| `errors` | number | 无法发送的数据报 |
| `resolved` | string[] | 前 8 个目标 |
| `summary` | string | 一行的载荷 |
| `error` | `EngineError` | 第一条失败的数据报为何失败；没有失败时省略 |

**错误**：`broadcast.target_required`、`broadcast.not_broadcast`、
`broadcast.ipv6`、`broadcast.not_multicast`、`broadcast.sweep_port`、
`broadcast.cidr_invalid`、`broadcast.prefix_invalid`、
`broadcast.sweep_too_large`、`node.osc_address`（OSC 地址必须以 `/` 开头）、
`hex.empty`、`hex.invalid`、`node.bind_invalid`、
`socket.option_failed`、`transport.target_invalid`、`transport.dns`，以及
绑定失败。

### broadcast_beacon_start {#broadcast_beacon_start}

一遍又一遍地发送同一轮——每个目标一条数据报。它的计数每 250 ms 作为
[`broadcast://emit-stat`](events.md#event-broadcast-emit-stat) 到达。在超过
32 次发送失败且一次都没有成功发送之后，它会带着原因停止。*启动任务*
（`beacon`、`params.mode`、`params.target`、`params.targets`、`params.rate`）。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `config` | object | 是 | 与 [`broadcast_send`](#broadcast_send) 相同，外加下面三个 |

| `config` 的字段 | 类型 | 默认值 | 含义 |
| --- | --- | --- | --- |
| `rate` | number | — | 每秒的轮数；大于 0，且轮数 × 目标数最多每秒 50 000 条数据报 |
| `count` | number | `0` | 这么多轮后停止；0：不限数量 |
| `duration_s` | number | `0` | 这么多秒后停止；0：直到被停止 |

**结果**：[`JobInfo`](#type-jobinfo)。

**错误**：[`broadcast_send`](#broadcast_send) 的那些、
`broadcast.rate_invalid`、`broadcast.rate_limit`。

### discovery_start {#discovery_start}

在一个 UDP 端口上监听，维护一份发来任何内容的每个对端的列表，并能像
设备那样应答探测。对端每 400 ms 作为
[`broadcast://peers`](events.md#event-broadcast-peers) 到达。*启动任务*
（`discovery`、`params.bind`、`params.groups`、`params.joined`）。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `config` | object | 是 | 见下文 |

| `config` 的字段 | 类型 | 默认值 | 含义 |
| --- | --- | --- | --- |
| `bind` | string | — | 要监听的 `IP:port` |
| `groups` | string[] | `[]` | 要加入的组播组（IPv4） |
| `interface` | string | 任意 | 在这些组上加入所用的本地 IPv4 地址 |
| `reuse` | boolean | `true` | 与已在监听该端口的程序共享端口（`SO_REUSEADDR`） |
| `respond` | boolean | `false` | 应答到达的内容 |
| `response` | [`Payload`](#type-payload) | 无 | 应答；与 `respond` 一起需要 |
| `respond_delay_ms` | number | `0` | 应答前等待这么久 |
| `match_contains` | string | 无 | 只应答其文本包含此内容的数据报 |

最多列出 512 个对端；更晚的对端不会被加入。

**结果**：[`JobInfo`](#type-jobinfo)。

**错误**：`node.bind_invalid`、`broadcast.port_shared`（端口已被占用且
`reuse` 关闭）、`broadcast.interface_invalid`、`broadcast.not_multicast`、
`broadcast.join_failed`、`broadcast.reply_missing`、`node.osc_address`、
`hex.*`，以及绑定失败。如果套接字无法再接收，任务会以
`wait.receive_failed` 结束。

```bash
invoke broadcast_send '{"config":{"mode":"list","target":"127.0.0.1:9000, 127.0.0.1:9001","payload":{"kind":"text","text":"PING"}}}' \
  | jq '{packets, errors}'
```

## 网络损伤 {#impairment}

位于客户端与其服务器之间的中继，对经过的内容施加延迟、丢包、重复、
损坏、乱序或限速，基于 UDP 或 TCP。参见[网络损伤](../tools/impairment.md)。

### netsim_start {#netsim_start}

启动一个中继：到达 `listen` 的内容继续发往 `target`，应答以同样的
方式返回，两个方向都按配置施加损伤。它的计数每 250 ms 作为
[`netsim://stat`](events.md#event-netsim-stat) 到达。*启动任务*
（`netsim`、`params.listen`、`params.target`，TCP 时还有
`params.protocol`）。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `config` | object | 是 | 见下文 |

| `config` 的字段 | 类型 | 默认值 | 含义 |
| --- | --- | --- | --- |
| `listen` | string | — | 中继监听的 `IP:port`；把客户端指向这里 |
| `target` | string | — | 真实服务器的 `IP:port`，或 `host:port`——主机名在中继启动时解析一次 |
| `profile` | [`ImpairProfile`](#type-impairprofile) | — | 要对流量做什么 |
| `seed` | number | 新建 | 抽取所用的种子：相同的种子和相同的流量会产生相同的丢包 |
| `protocol` | string | `udp` | `udp`（数据报）或 `tcp`（流） |

**结果**：[`JobInfo`](#type-jobinfo)。

**错误**：`node.range`（配置中的某个值超出其范围，带有 `min`、`max`
和该字段）、`node.too_long`、`node.bind_invalid`、
`transport.target_invalid`、`transport.dns`（找不到的目标名称），以及
绑定失败。如果某个套接字无法再接收，任务会以 `wait.receive_failed`
结束。

### netsim_set_profile {#netsim_set_profile}

正在运行的中继从此改用另一个配置施加损伤，而不关闭它的套接字。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `jobId` | number | 是 | 中继的任务 |
| `profile` | [`ImpairProfile`](#type-impairprofile) | 是 | 新配置 |

**结果**：null。

**错误**：`netsim.not_running`、`node.range`、`node.too_long`。

```bash
invoke netsim_start '{"config":{"listen":"127.0.0.1:9010","target":"127.0.0.1:9000","profile":{"latency_ms":80,"jitter_ms":20,"loss":0.02}}}'
```

## 风暴与扫描 {#storm-scanner}

::: danger
风暴会按您要求的强度给目标加压，扫描会探测一个范围内的每个端口。只把
它们对准您负责的主机。
:::

### storm_start {#storm_start}

向一个目标持续发送 UDP 数据报或 TCP 连接的负载。它的计数每 250 ms 作为
[`storm://stat`](events.md#event-storm-stat) 到达。*启动任务*（`storm`、
`params.protocol`、`params.target`、`params.rate`）。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `config` | object | 是 | 见下文 |

| `config` 的字段 | 类型 | 默认值 | 含义 |
| --- | --- | --- | --- |
| `target` | string | — | `IP:port` 或 `host:port`；名称在任务启动时解析一次 |
| `protocol` | string | — | `udp`：数据报；`tcp`：每个单位一个连接，写入载荷后关闭（每次连接可能耗时 500 ms） |
| `size` | number | — | 载荷字节数，保持在 1 到 65 507 之间 |
| `rate` | number | — | 每秒的单位数，按时间表：第 *n* 个单位在启动后 *n* / `rate` 秒到期，每次唤醒发送到期的内容（最多 256 个；时间表落后更多时会跳过较旧的单位）；0 表示能发多快就发多快 |
| `duration_s` | number | `0` | 这么多秒后停止；0：直到被停止 |

**结果**：[`JobInfo`](#type-jobinfo)。

**错误**：`transport.target_invalid`、`transport.dns`。失败的发送会在
事件中计数，而不作为错误报告。

### scan_start {#scan_start}

尝试与一个范围内每个端口建立 TCP 连接，并报告开放的端口，以及被询问时
服务最先说的内容。开放的端口作为 [`scan://open`](events.md#event-scan-open)
到达，进度作为 [`scan://progress`](events.md#event-scan-progress)。*启动
任务*（`scan`、`params.host`、`params.from`、`params.to`）。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `config` | object | 是 | 见下文 |

| `config` 的字段 | 类型 | 默认值 | 含义 |
| --- | --- | --- | --- |
| `host` | string | — | 主机名或地址 |
| `port_start`, `port_end` | number | — | 范围，两端都包含；给反了会互换 |
| `concurrency` | number | `256` | 同时尝试的次数，1 到 1024 |
| `timeout_ms` | number | `600` | 每个端口，50 到 10 000 |
| `grab_banner` | boolean | `false` | 连接后 400 ms 内读取服务发送的最多 256 字节 |

**结果**：[`JobInfo`](#type-jobinfo)。

**错误**：`scan.host_required`。

```bash
invoke scan_start '{"config":{"host":"127.0.0.1","port_start":8000,"port_end":9100,"grab_banner":true}}'
```

## 检查器 {#inspector}

检查器在捕获开启期间，以帧的形式记录各工具发送和接收的内容。在服务器
上，每个页面和脚本各有一个检查器。参见[检查器](../tools/inspector.md)。

### inspect_set_enabled {#inspect_set_enabled}

开启或关闭捕获。关闭期间不记录任何内容。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `enabled` | boolean | 是 | 开启（`true`）或关闭 |

**结果**：[`CaptureStats`](#type-frame)。

### inspect_stats {#inspect_stats}

捕获的计数器。没有参数。

**结果**：[`CaptureStats`](#type-frame)。

### inspect_snapshot {#inspect_snapshot}

最新的帧，最旧的在前。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `limit` | number | 是 | 多少个，1 到 8192 |

**结果**：[`Frame`](#type-frame)`[]`，不含它们的字节（参见
[`inspect_payload`](#inspect_payload)）。

### inspect_clear {#inspect_clear}

清空捕获及其计数器。没有参数。

**结果**：[`CaptureStats`](#type-frame)。

### inspect_export {#inspect_export}

将持有的每一帧写入数据文件夹中的 `capture-<ms>.jsonl` 或
`capture-<ms>.txt`。在 `jsonl` 中，每行是一帧，其保留的字节以 base64 放在
`data` 中；`txt` 供阅读，每帧带一段十六进制转储。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `format` | string | 是 | `txt`；其他任何值写入 `jsonl` |

**结果**：字符串，写入的路径。

**错误**：`inspect.empty`、`file.io`。

### inspect_payload {#inspect_payload}

一帧所保留的字节，超出其批次携带的 1 KiB 预览部分。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `seq` | number | 是 | 帧的编号 |

**结果**：`{ "seq", "bytes", "kept", "dump", "hex" }`——`bytes` 为该帧的
大小，`kept` 为其中保留了多少（最多 256 KiB），`dump` 为每一行写作
`offset  hex  |ascii|`，`hex` 为重放时发送的纯十六进制。

**错误**：`inspect.frame_gone`（更新的帧取代了它）、
`inspect.no_payload`（只记录了它的大小）。

```bash
invoke inspect_set_enabled '{"enabled":true}'
invoke inspect_snapshot '{"limit":20}' | jq '.[] | {seq, proto, dir, summary}'
```

## 信号库 {#signals}

库是数据文件夹中的 `signals.json`。它只是存储：信号用其传输协议的命令
发送（`osc_send`、`broadcast_send`、`http_request`、`mqtt_publish` 或
`mqtt_publish_once`）。参见[信号](../tools/signals.md)和
[文件](../reference/files.md#signals-json)。

### signals_load {#signals_load}

读取库。文件不存在时，会先写入初始信号集。没有参数。

**结果**：`{ "path": string, "library": library, "seeded": boolean }`——
刚写入初始信号集时 `seeded` 为 true。库为
`{ "version", "signals": [...], "folders": [...] }`：`version` 2（版本 1 的
文件原样返回），没有文件夹时省略 `folders`。每个信号有 `id`、`name`、
`group`（其文件夹，`"A/B"`；没有则为空）、`note` 和 `body`。

**错误**：`signals.json_invalid`（带有 `path`、`line`、`column`；文件
绝不会被替换）、`file.io`。

### signals_save {#signals_save}

通过同一文件夹中的临时文件替换整个库文件。已存在但无法读作库的文件会
保持原样。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `library` | object | 是 | `{ version, signals, folders }`，与 `signals_load` 返回的相同 |

**结果**：字符串，写入的路径。

**错误**：`signals.json_invalid`（磁盘上当前的文件无法读取，带有
`path`、`line`、`column`；什么都不会写入）、`signals.encode`、`file.io`。

一个信号的 `body` 按其 `transport` 为：

| `transport` | 字段 |
| --- | --- |
| `osc` | `target`、`address`、`args`（[`OscArg`](#type-oscarg)`[]`） |
| `udp` | `target`、`payload`：`{ "kind": "text", "text" }` 或 `{ "kind": "hex", "hex" }` |
| `http` | `request`（[`HttpRequest`](#type-httprequest)） |
| `mqtt` | `broker`（`host:port`）、`topic`、`payload`、`qos`、`retain` |

```bash
invoke signals_load | jq '.library.signals[] | {name, transport: .body.transport}'
```

## 模拟器 {#emulators}

模拟器就是由 Signal Lab 扮演另一端：HTTP API、OSC、UDP 或 TCP 设备、
MQTT 代理。它的文档——`name`、`bind`、`protocol`、协议规则以及可选的
`outage`——在[模拟器](../tools/emulators.md)中描述。库是数据文件夹中的
`emulators.json`。

### emulators_load {#emulators_load}

读取模拟器库。文件不存在时，会先写入初始集合。没有参数。

**结果**：`{ "path", "library": { "version": 1, "emulators": [{ "id", "note", "emulator" }] }, "seeded" }`。

**错误**：`emulators.json_invalid`（带有 `path`、`line`、`column`；绝不
被替换）、`file.io`。

### emulators_save {#emulators_save}

通过临时文件替换整个模拟器库。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `library` | object | 是 | `{ version, emulators }`，与 `emulators_load` 返回的相同 |

**结果**：字符串，写入的路径。

**错误**：`emulators.encode`、`file.io`。

### emulator_check {#emulator_check}

一个模拟器能否启动：[`emulator_start`](#emulator_start) 在绑定之前检查
的一切。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `emulator` | object | 是 | 模拟器文档 |
| `params` | object of strings | 否 | 其模板作为参数读取的值 |

**结果**：能够启动时为 null。

**错误**：`emulator.*`；字段缺失、超出范围、过长或格式错误时为
`node.*`（`node.required`、`node.range`、`node.too_long`、
`node.bind_invalid`、`node.target_invalid`、`node.method_invalid`……）；
`param.unknown`、`template.*`、`osc.pattern_*`、`regex.invalid`、
`hex.invalid`。当问题出在其中之一时，每个错误在 `params` 中带有 `rule`、
`retained` 或 `response`。

### emulator_start {#emulator_start}

把一个模拟器作为独立任务启动。命令返回时它的套接字已经打开。它接收和
应答的内容在有变化时每 200 ms 作为
[`emulator://activity`](events.md#event-emulator-activity) 到达。*启动任务*
（`emulator`、`params.name`、`params.protocol`、`params.local`，给出
`source` 时还有 `params.source`）。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `emulator` | object | 是 | 模拟器文档 |
| `params` | object of strings | 否 | 其模板作为参数读取的值 |
| `seed` | number | 否 | 它的种子，0 到 9007199254740991；默认：新的一个 |
| `source` | string | 否 | 它来自的库条目，作为 `params.source` 保留在任务上 |

**结果**：[`JobInfo`](#type-jobinfo)。

**错误**：[`emulator_check`](#emulator_check) 的那些、`seed.range`、
`transport.address_in_use` 以及其他绑定失败。

### emulator_exchanges {#emulator_exchanges}

正在运行的模拟器接收和应答的内容。它保留最近的 500 次交互。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `jobId` | number | 是 | 模拟器的任务 |
| `after` | number | 否 | 只包含编号高于此值的交互；默认 0 |
| `limit` | number | 否 | 最多这么多个，1 到 500；默认 500 |

**结果**

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 任务 |
| `name`, `protocol`, `local` | string | 模拟器、其协议以及它监听的地址 |
| `counts` | object | `total`、`unmatched`（没有规则接收）、`failed`、`down`（它停机期间到达：其停机计划或 [`emulator_down`](#emulator_down)）、`hits`（按规则）以及 `missed`（MQTT：太落后的客户端没有收到的消息；为 0 时省略） |
| `forced` | string | 它被停机期间为 `unavailable`、`reset` 或 `timeout`；否则省略 |
| `exchanges` | object[] | 每一项：`seq`、`ts`、`from`、`request`、`rule`（从 1 开始；没有规则接收时省略）、`reply`、`status`、`fault`、`ms`、`error`、`frame`、`down`，以及 `data`（模板读取到的请求内容） |

**错误**：`emulator.not_running`。

### emulator_down {#emulator_down}

让一个正在运行的模拟器停机，直到被恢复，无论其停机计划怎么说，或把它
恢复回来。停机期间，HTTP 模拟器用 `fault` 应对每个请求，TCP 设备和 MQTT
代理断开连接并拒绝新连接，OSC 和 UDP 设备则不作任何应答。

| 参数 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `jobId` | number | 是 | 模拟器的任务 |
| `down` | boolean | 是 | 停机（`true`）或恢复 |
| `fault` | string | 否 | HTTP 请求遇到的情况：`unavailable`（503，不带 `Retry-After`：何时恢复不得而知）、`reset`（连接关闭）、`timeout`（不应答）；默认 `unavailable` |

**结果**：null。

**错误**：`emulator.not_running`。

```bash
invoke emulator_exchanges '{"jobId":5,"after":0}' | jq '.counts, (.exchanges[] | {request, rule, status})'
```

## 共享类型 {#types}

### JobInfo {#type-jobinfo}

启动任务的命令所返回的内容，也是 [`jobs_list`](#jobs_list) 所列出：
`id`、`kind`、`label`（英文，用于日志）、`params`（标签所指的值；没有时
省略）以及 `started_ms`。参见[任务](index.md#jobs)。

### OscArg {#type-oscarg}

一个 OSC 参数，其类型和值：

| `type` | `value` | OSC tag |
| --- | --- | --- |
| `int` | 32 位整数 | `i` |
| `float` | 数字，以 32 位浮点数发送 | `f` |
| `str` | 字符串 | `s` |
| `long` | 64 位整数 | `h` |
| `double` | 数字，64 位 | `d` |
| `bool` | `true` 或 `false` | `T` 或 `F` |
| `blob` | 字节数组，`[222, 173]` | `b` |
| `nil` | 无：`{ "type": "nil" }` | `N` |

### HttpRequest {#type-httprequest}

| 字段 | 类型 | 默认值 | 含义 |
| --- | --- | --- | --- |
| `method` | string | — | `GET`、`POST`…… |
| `url` | string | — | `http://` 或 `https://` |
| `headers` | `[name, value][]` | `[]` | 请求标头 |
| `body` | string or null | null | 正文 |
| `timeout_ms` | number | `10000` | 用于整个交互 |
| `auth` | object | 无 | `{ "scheme": "basic", "username", "password" }`、`{ "scheme": "digest", "username", "password" }` 或 `{ "scheme": "bearer", "token" }` |

最多跟随 10 次重定向。为一个主机输入的凭据和 Cookie 绝不会发往另一个
主机。Digest 请求会应答服务器的 401 质询并再次发送。

### HttpResponse {#type-httpresponse}

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `ok` | boolean | 状态为 2xx |
| `status`, `status_text` | number, string | 状态；没有响应时为 0 和空 |
| `latency_ms` | number | 直到整个正文到达 |
| `headers` | `[name, value][]` | 响应标头 |
| `body` | string | 正文的文本，最多 256 KiB |
| `body_bytes` | number | 正文的完整大小 |
| `truncated` | boolean | `body` 在 256 KiB 处被截断 |
| `error` | string or null | 为何没有响应，原因的所有层次 |
| `cause` | string or null | 失败的类别：`refused`、`timeout`、`dns`、`unreachable`、`reset`、`address_in_use`、`address_unavailable`、`denied`、`tls`、`target_invalid`、`failed`——与 `transport.*` 代码相同 |
| `digest` | object | 仅当 Digest 请求遇到了 401；否则省略。`challenged`：质询已被应答并再次发送请求。`error`：为何未能如此，一个 `EngineError`（`http.digest_not_offered`、`http.digest_unsupported`、`http.digest_invalid`、`http.digest_other_origin`）或 null |

### Payload {#type-payload}

广播或发现数据报所携带的内容：
`{ "kind": "osc", "address", "args" }`、`{ "kind": "text", "text" }`
（原样发送，没有终止零）或 `{ "kind": "hex", "hex" }`（`de ad be ef`、
`deadbeef`、`0xDE,0xAD`——十六进制数字以外的任何内容都被忽略）。

### MqttConfig {#type-mqttconfig}

| 字段 | 类型 | 默认值 | 含义 |
| --- | --- | --- | --- |
| `host` | string | — | 代理的名称或地址 |
| `port` | number | — | 通常为 1883 |
| `client_id` | string | — | 非空；同一 id 的另一个连接会被代理顶掉 |
| `username`, `password` | string | 空 | `username` 非空时发送；`password` 只在有 `username` 时一起发送 |
| `keep_alive_s` | number | `60` | 心跳按其一半发送；0：不发送 |
| `clean_session` | boolean | `true` | CONNECT 标志 |
| `will` | object or null | null | `{ topic, payload, qos, retain }`，连接丢失时由代理发布 |
| `subscribe` | `{ filter, qos }[]` | `[]` | 连接建立后立即订阅 |

### WsConfig {#type-wsconfig}

| 字段 | 类型 | 默认值 | 含义 |
| --- | --- | --- | --- |
| `url` | string | — | `ws://` 或 `wss://`（`wss://` 信任系统对 HTTPS 所信任的内容） |
| `headers` | `[name, value][]` | `[]` | 随升级请求一起发送 |
| `protocols` | string[] | `[]` | 要提供的子协议，按偏好排序 |
| `timeout_ms` | number | `10000` | 用于连接、TLS 和升级之和 |

消息两个方向都最多 16 MiB。

### ImpairProfile {#type-impairprofile}

每个字段都是可选的；省略的字段不产生任何效果。概率为 0 到 1。

| 字段 | 范围 | 含义 | UDP | TCP |
| --- | --- | --- | --- | --- |
| `name` | 最多 60 个字符 | 用于时间线和报告的标签 | 是 | 是 |
| `latency_ms` | 0 到 60 000 | 加在所有内容上的延迟 | 是 | 是 |
| `jitter_ms` | 0 到 60 000 | 每次抽取时最多再加这么多 | 是 | 是 |
| `loss` | 0 到 1 | 一条数据报被丢弃 | 是 | — |
| `duplicate` | 0 到 1 | 一条数据报被发送两次 | 是 | — |
| `corrupt` | 0 到 1 | 数据报的一位被翻转 | 是 | — |
| `reorder` | 0 到 1 | 一条数据报被暂扣，让后面的超过它 | 是 | — |
| `rate_kbps` | 0，或 8 到 10 000 000 | 带宽限制，千比特每秒；0：不限 | 是 | 是 |
| `burst_start` | 0 到 1 | 一条数据报开启一串丢失 | 是 | — |
| `burst_length` | 1 到 1000 | 一串平均持续的数据报数（与 `burst_start` 一起需要） | 是 | — |
| `offline` | `true` 或 `false` | 什么都通不过 | 是 | 是 |
| `reset` | 0 到 1 | 一个流的数据块重置其连接 | — | 是 |
| `stall` | 0 到 1 | 一个流的数据块让其连接保持半开 | — | 是 |

### Frame and CaptureStats {#type-frame}

`Frame` 是一个被捕获的数据包、请求或消息：

| 字段 | 含义 |
| --- | --- |
| `seq` | 它的编号，递增 |
| `ts` | 时间，自 1970 年以来的毫秒数 |
| `proto` | `osc`、`udp`、`tcp`、`http`、`mqtt`、`ws`…… |
| `dir` | `tx`（发送）或 `rx`（接收） |
| `source` | 捕获它的工具：`osc-monitor`、`broadcast`、`netsim`…… |
| `job_id` | 它的任务，或 null |
| `local`, `remote` | 地址：本侧和另一侧的 `IP:port`（HTTP、WebSocket 和 MQTT 为 URL 或代理）。被中继的帧，其 `local` 是中继监听的地址，`remote` 是帧原本要去的地址；该段以其 `verdict` 结尾（`· client→target`、`· target→client`） |
| `bytes` | 它的大小 |
| `summary` | 一行 |
| `detail` | 跨多行的解码，或 null |
| `hex` | 前 1 KiB 的十六进制转储，或 null |
| `verdict` | 它的结局——`dropped`、`sampled`、一个状态——或 null |
| `kept` | 在 `bytes` 中保留了多少（最多 256 KiB）；只记录了大小时为 0 |
| `publish` | 仅 MQTT 发布：`{ broker, topic, qos, retain, text }`——代理为 `host:port`，以及保留的字节（即消息的载荷）是否为 UTF-8 文本。其他所有帧都不含此项 |

`CaptureStats`：`enabled`、`total`（记录的帧数）、`bytes`、`skipped`
（已记录但从未发送到界面的帧）、`buffered`（持有的帧）、`capacity`（8192）、
`held`（持有的载荷字节数）和 `held_limit`（64 MiB）。超过任一限制时，最旧
的帧会让位。

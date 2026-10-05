---
title: 文件和文件夹
description: Signal Lab 把文件保存在哪里——数据文件夹、实验、信号库和模拟器库、运行报告和导出——它们的格式，以及哪些内容可以安全地编辑、备份和移动。
---

# 文件和文件夹

Signal Lab 保存的一切都是纯 JSON（或文本），放在一个文件夹中，即数据文件夹。机密值从不在其中。

## 数据文件夹 {#data-folder}

| Signal Lab 运行的位置 | 数据文件夹 |
| --- | --- |
| 桌面应用，Windows | 您用户文件夹中的 `Documents\SignalLab`：`C:\Users\<you>\Documents\SignalLab` |
| 桌面应用，Linux | `~/Documents/SignalLab` |
| 服务器 | `--data-dir`，或 `SIGNALLAB_DATA_DIR`；两者都没有时，为运行用户 home 文件夹中的 `Documents/SignalLab` |
| 服务器，Docker 镜像 | `/data`，一个卷（compose 文件中的 `signallab-data`） |
| `signallab run` | 一个临时文件夹，退出时删除——除非 `--data-dir` 指定了某个文件夹 |

桌面应用在设置时也会从其环境中读取 `SIGNALLAB_DATA_DIR`。该文件夹会在首次写入内容时创建。

::: tip
在 Windows 上，应用使用您用户文件夹中直接位于其下的 `Documents` 文件夹，即使 Windows 把您的文档存放在别处（OneDrive）也是如此。
:::

在服务器上，文件写在服务器的机器上，而不是您的机器上。顶栏中的 [[ui:app.server]] 徽章会在其提示中说明位置；API 会把它作为 [`app_info`](../api/commands.md#app_info) 的 `data_dir` 提供，[`/api/files`](../api/index.md#files) 则会下载其中的内容。

命令行的 `signallab emulate`、`signallab send` 和 `signallab mcp` 会像桌面应用一样从同一个文件夹读取应用的库。

## 其中有什么 {#contents}

| 文件 | 它是什么 | 写入时机 |
| --- | --- | --- |
| `experiment.json` | 编辑器中打开的实验 | 每次更改后不久 |
| `signals.json` | 信号库（[[ui:nav.signals]]） | 每次更改后不久 |
| `emulators.json` | 模拟器库（[[ui:nav.emulators]]） | 每次更改后不久 |
| `runs/run-<ms>-<job>.json` | 每个自行结束的运行的报告 | 运行结束时 |
| `exports/experiment-<ms>-<16 hex digits>.json` | 实验的快照 | [[ui:exp.exportJson]] |
| `capture-<ms>.jsonl`、`capture-<ms>.txt` | 检查器的帧 | [[ui:ins.exportJsonl]]、[[ui:ins.exportTxt]] |
| `token` | 服务器的访问令牌，仅其用户可读 | `--generate-token`，在首次启动时 |
| `.experiment-<hex>.tmp`、`.signals-<hex>.tmp`、`.emulators-<hex>.tmp` | 正在进行的一次保存 | 短暂存在，然后重命名 |

`<ms>` 是自 1970 年以来的毫秒时间；`<job>` 是运行的作业编号。在服务器上，每个浏览器都在同一个 `experiment.json`、同一批库和同一批报告上工作。

## 格式 {#formats}

它们都是 UTF-8 的 JSON，带缩进书写，便于阅读和对比差异。每个文件都带有一个 `version`；较旧版本的文件在打开时会被读取并迁移，并在下次保存时按当前版本写回——此后较旧的 Signal Lab 就无法打开它。

### experiment.json {#experiment-json}

实验文档，版本 9——与 [[ui:exp.exportJson]] 写出和 [[ui:exp.importJson]] 读取的 JSON 相同：

```json
{
  "version": 9,
  "name": "HTTP check",
  "params": [],
  "profiles": [],
  "profile": null,
  "seed": null,
  "cookies": true,
  "nodes": [ { "id": "start", "type": "start", "x": 40, "y": 80 }, … ],
  "edges": [ { "from": "start", "to": "request", "port": "next" }, … ]
}
```

- 最多 4 MiB，以及 1 到 64 个节点（`doc.node_count`）。
- 版本 1 到 8 在打开时迁移。版本 8 之前的文件打开时 `cookies` 为关闭，因此会按原来的方式运行；每个版本新增的其他设置（版本 2 的参数、版本 3 的配置、版本 4 的重试、版本 5 的重复和循环、版本 6 的模拟器、版本 7 的损伤、版本 8 的 WebSocket 和 HTTP 认证、版本 9 的负载）都从空开始。
- 比这个 Signal Lab 所知道的更新的版本会被拒绝（`doc.version_unsupported`），而不是在缺少它无法读取的内容时打开。
- 无法解析的文件会连同其路径、行和列一起报告，绝不会被替换。
- 它会写入一个临时文件然后重命名，因此写入失败会保留上一个文件。

节点、参数和配置是什么：[实验](../experiments/index.md)、[节点](../experiments/nodes.md)、[数据](../experiments/data.md)。

### signals.json {#signals-json}

信号库，版本 2：

```json
{
  "version": 2,
  "signals": [
    {
      "id": "…",
      "name": "Go cue",
      "group": "Stage/Cues",
      "note": "",
      "body": { "transport": "osc", "target": "127.0.0.1:9000", "address": "/cue/go", "args": [ { "type": "int", "value": 1 } ] }
    }
  ],
  "folders": [ "Stage", "Stage/Cues" ]
}
```

- `group` 是信号所在的文件夹，以 `/` 路径表示；为空则是顶层。`folders`（版本 2 新增）列出每个文件夹，包括空文件夹，没有时则省略。版本 1 的文件读取方式相同，只是没有空文件夹。
- `body` 是 `osc`、`udp`、`http` 或 `mqtt` 之一；它们的字段见 [`signals_save`](../api/commands.md#signals_save)。
- 当文件不存在时，会写入起始集合——每个目标都在 `127.0.0.1` 上——并重命名为界面所用的语言。
- 无法解析的文件会连同其路径、行和列一起报告（`signals.json_invalid`），绝不会被起始集合替换：请修复或删除它。在它无法读取期间，没有任何操作会写入库——保存会以同样的错误被拒绝，文件保持原样——直到 [[ui:sig.reload]] 再次读取它。保存会经过同一文件夹中的临时文件，因此被中断的写入会保留上一个文件。

### emulators.json {#emulators-json}

模拟器库，版本 1：

```json
{
  "version": 1,
  "emulators": [
    { "id": "demo-api", "note": "…", "emulator": { "name": "Demo API", "bind": "127.0.0.1:8080", "protocol": "http", "routes": [ … ] } }
  ]
}
```

每个条目都是一个带有 `id` 和 `note` 的模拟器文档；该文档在[模拟器](../tools/emulators.md)中描述。与信号一样，文件缺失时会得到起始集合（每个都绑定到 `127.0.0.1`），损坏的文件会被报告（`emulators.json_invalid`），绝不会被替换。它通过临时文件写入。`signallab emulate` 也会读取自己的文件，其中可以包含一个模拟器、一个模拟器列表，或像这样的一个库。

### 运行报告 {#run-reports}

`runs/run-<started ms>-<job>.json`，报告版本 5：每个通过或失败的运行一个文件，绝不覆盖（同名第二次运行会加 `-2`、`-3`……）。被停止的运行不会保存报告。

| 字段 | 它是什么 |
| --- | --- |
| `version` | 5 |
| `experiment` | 实验的名称 |
| `document_version` | 所运行文档的版本 |
| `seed`、`profile` | 运行所用的值 |
| `overrides` | 仅为此运行提供的值 |
| `params` | 它使用的每个参数值 |
| `started_ms`、`ended_ms` | 自 1970 年以来的毫秒数 |
| `outcome` | `passed` 或 `failed` |
| `error` | 它的第一个失败，或 null |
| `steps` | 每个步骤，格式为 [`experiment://step`](../api/events.md#event-experiment-step) |
| `emulators` | 每个 [[ui:exp.node.emulator]] 节点收到和应答的内容（自版本 3 起）；没有时省略 |
| `impairments` | 每个 [[ui:exp.node.impairment]] 节点的中继逐阶段做了什么（自版本 4 起）；没有时省略 |

负载步骤的测量结果在其最后一个步骤上（自版本 5 起）。时间线的运行历史和 [[ui:exp.compare]] 会读取这些文件；无法读取的报告会从列表中排除。参见[运行与报告](../experiments/runs.md)。

### 导出 {#exports}

- `exports/experiment-…json`：实验文档，如上所述。每次导出都是一个新文件。
- `capture-….jsonl`：每行一个检查器帧，带它在 `data` 中以 base64 保存的字节。
- `capture-….txt`：供阅读的帧，每个都带十六进制转储。

在服务器上，检查器的导出在生成时会下载到您的计算机；实验的导出会提供 [[ui:common.download]]，时间线中运行的 [[ui:exp.reportSaved]] 是一个可下载其报告的链接。

## 机密不在这些文件中 {#secrets}

实验会引用一个机密——`{{secret.API_TOKEN}}`——但只会写入名称。其值保存在：

| Signal Lab 运行的位置 | 机密值在哪里 |
| --- | --- |
| 桌面应用，Windows | Windows 凭据管理器，在 `SignalLab` 之下（编辑器中的 [[ui:exp.secrets]]） |
| 桌面应用，Linux | 没有：机密无法存储（`secret.unsupported`） |
| 服务器 | 只读：环境变量 `SIGNALLAB_SECRET_<NAME>`，或 `--secrets-dir` 中的文件 `<NAME>`（默认 `/run/secrets/signallab`） |
| `signallab` | 相同的文件和变量，或用 `--secrets system` 使用系统存储 |

::: warning
您直接输入字段的内容会按原样保存。HTTP 信号凭据中的密码、MQTT broker 模拟器中的密码、粘贴到标头中的令牌——在 `signals.json`、`emulators.json` 或 `experiment.json` 及其导出中都是明文。对于任何您不会放进共享文件夹的内容，请在实验中使用 `{{secret.NAME}}`。
:::

## 界面设置 {#settings}

界面记住的内容——它的语言、每个界面上最后输入的值、哪个窗格打开以及多大、[[ui:http.keepCookies]]、上次查找更新的时间，以及该安装用于更新的随机数——由界面自己保存，而不是保存在数据文件夹中：桌面上保存在应用自己的存储中，服务器页面上则保存在浏览器的站点存储中（按浏览器）。[[ui:nav.http]] 界面的凭据不会保存在那里。

Signal Lab 不写日志文件；参见[故障排除](troubleshooting.md#logs)。

## 备份、编辑、移动 {#backup}

- **备份**：复制整个文件夹。其中一切都是自包含的 JSON；机密值不在其中，因此在新机器上需要重新设置。
- **编辑** `signals.json`、`emulators.json` 和 `experiment.json` 时，请在 Signal Lab 关闭时手工进行（在服务器上则是没有页面打开时）：应用会根据自己的内容写入整个文件，因此它运行时所做的更改会在下次保存时被覆盖。错误会在下次读取文件时连同行和列一起报告，绝不会被静默替换——对于 `signals.json`，应用向其保存时也是如此：该保存会被拒绝，文件保持您离开时的样子。
- **删除** `runs/`、`exports/` 和 `capture-*` 文件随时都可以。删除 `signals.json` 或 `emulators.json` 会恢复起始集合；删除 `experiment.json` 会恢复起始实验。
- **移动**文件夹：复制它，并让 Signal Lab 指向新位置：服务器用 `--data-dir`，桌面应用用 `SIGNALLAB_DATA_DIR`。
- **共享**实验：将其导出，或者把它针对的项目的 JSON 一并提交；[`signallab run`](../automation/cli.md#cli-run) 会从那里运行它。

---
title: AI 助手（MCP）
description: signallab mcp 让 AI 助手通过 Model Context Protocol 构建、检查和运行实验，发送消息，监听端口，并扮演模拟器。
---

# 为 AI 助手准备的 Signal Lab：`signallab mcp`

`signallab mcp` 是一个 [Model Context Protocol](https://modelcontextprotocol.io) 服务器。Claude Code、Claude Desktop、Cursor、VS Code 或任何其他 MCP 客户端中的 AI 助手可以启动它，然后：

- 了解实验由什么构成，编写一个实验，检查它，运行它，并逐步读出它失败的原因；
- 发送一条 OSC 消息、一个数据报、一个 HTTP 请求、一条 WebSocket 消息或一次 MQTT 发布，并在某个端口上监听设备发来的内容；
- 触发您信号库中的一个信号；
- 扮演一个依赖（HTTP API、OSC、UDP 或 TCP 设备、MQTT 代理），并读取您的系统向它发送了什么；
- 读回之前的运行，并对比其中两次。

每个操作都通过应用所用的同一批引擎命令完成，因此助手运行的实验就是应用会进行的那次运行，有同样的报告，每个失败也都用界面的措辞来表述。

::: warning
发送、运行和模拟器都会把真实的流量放到网络上。请告诉助手哪些设备可以让它访问；内置模板指向环回地址（`127.0.0.1`）。
:::

## 设置 {#setup}

`signallab` 随桌面应用一起提供，安装后就在您的 `PATH` 中（参见[安装](cli.md#install)）。客户端会自行启动 `signallab mcp`，并通过标准输入和标准输出与它通信；您不需要手动运行它。

### 输出配置 {#print-config}

`--print-config` 会输出客户端所需的配置，其中包含这个 `signallab` 的完整路径：

| 命令 | 输出内容 |
| --- | --- |
| `signallab mcp --print-config claude-code` | `claude mcp add` 命令行。 |
| `signallab mcp --print-config claude-desktop` | 用于 Claude Desktop 配置文件的 `mcpServers` 条目。 |
| `signallab mcp --print-config cursor` | 同样的 `mcpServers` 条目，用于 Cursor 的 `mcp.json`。 |
| `signallab mcp --print-config vscode` | 用于 VS Code 的 `.vscode/mcp.json` 的 `servers` 条目。 |

对于在[实验室服务器](#on-a-server)上工作的助手，请加上 `--server URL`：输出的配置就会带上它，并为令牌留出占位符。

### Claude Code {#claude-code}

运行 `--print-config claude-code` 输出的那一行，例如：

```bash
claude mcp add signallab -- "C:\Program Files\Signal Lab\signallab.exe" mcp
```

### Claude Desktop 和 Cursor {#claude-desktop}

把该条目放进客户端的配置中（Claude Desktop 是 `claude_desktop_config.json`，Cursor 是 `mcp.json`），然后重启客户端：

```json
{
  "mcpServers": {
    "signallab": {
      "command": "C:\\Program Files\\Signal Lab\\signallab.exe",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

### VS Code {#vscode}

```json
{
  "servers": {
    "signallab": {
      "type": "stdio",
      "command": "/usr/bin/signallab",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

### 其他客户端 {#other-clients}

任何能启动 stdio 服务器的客户端，用法都相同：命令是 `signallab`（或其完整路径），参数是 `mcp` 以及任何[选项](#options)。在 Linux 上，服务器镜像也可以作为命令使用：

```bash
docker run -i --rm --network host --entrypoint signallab ghcr.io/proanima/signallab:[[version]] mcp
```

## 选项 {#options}

| 选项 | 作用 | 默认值 |
| --- | --- | --- |
| `--server URL` | 在这台 Signal Lab 服务器上运行实验、发送和模拟器（参见[在实验室服务器上](#on-a-server)）。 | `SIGNALLAB_SERVER` |
| `--token-file PATH` | 存放服务器令牌的文件。 | `SIGNALLAB_TOKEN_FILE`，否则 `SIGNALLAB_TOKEN` |
| `--data-dir PATH` | 运行及其报告的保存位置。不能与 `--server` 同用。 | 应用的数据文件夹（`Documents/SignalLab`） |
| `--library PATH` | 供 `list_signals` 和 `fire_signal` 使用的信号库。 | 应用的 `signals.json` |
| `--emulators PATH` | 供 `list_emulators` 和 `start_emulator` 使用的模拟器库。 | 应用的 `emulators.json` |
| `--secrets files\|system` | 在本机上运行时机密值的来源，与 [`run`](cli.md#secrets) 相同。不能与 `--server` 同用。 | `files` |
| `--secrets-dir PATH` | 存放机密文件的文件夹，每个名称一个文件。不能与 `--server` 同用。 | 存在时为 `/run/secrets/signallab` |
| `--lang <code>` | 结果和失败信息所用的语言。 | `SIGNALLAB_LANG`，否则为系统区域设置，否则为 `en` |
| `--print-config CLIENT` | 输出某个客户端的配置并退出：`claude-code`、`claude-desktop`、`cursor` 或 `vscode`。 | |

运行的报告保存在应用的数据文件夹中，与应用自己的报告放在一起，因此会话结束后仍然保留。

## 工具 {#tools}

只读取的工具标记为只读，因此客户端可以不经询问就让它们运行。会触及外部世界的工具（发送、监听或启动某些东西的工具）会如此标记，客户端可以在每次调用前询问您。没有任何工具被标记为具有破坏性。

| 工具 | 作用 | 是否触及外部世界 |
| --- | --- | --- |
| `describe_nodes` | 实验文档、每种节点及其字段、输出和示例，`{{template}}` 语言、负载模式以及模拟器文档。 | 否 |
| `list_templates` | 内置的实验及其参数。 | 否 |
| `get_template` | 以文档形式给出一个内置实验。 | 否 |
| `validate_experiment` | 像编辑器在运行前那样检查实验；不发送任何内容。 | 否 |
| `run_experiment` | 把实验运行到结束，并报告每个步骤。 | 是 |
| `send_osc` | 一条 OSC 消息。 | 是 |
| `send_udp` | 一个 UDP 数据报。 | 是 |
| `send_http` | 一个 HTTP 请求。 | 是 |
| `send_mqtt` | 一次 MQTT 3.1.1 发布。 | 是 |
| `send_ws` | 一次 WebSocket 交互。 | 是 |
| `listen` | 一段时间内到达某个 UDP 端口的内容。 | 是 |
| `list_signals` | 您信号库中的信号。 | 否 |
| `fire_signal` | 发送一个库中的信号。 | 是 |
| `list_emulators` | 您模拟器库中的模拟器。 | 否 |
| `start_emulator` | 启动一个模拟器。 | 是 |
| `emulator_exchanges` | 正在运行的模拟器收到了什么、应答了什么。 | 否 |
| `set_emulator_down` | 让正在运行的模拟器停机，或让它恢复。 | 是 |
| `list_runs` | 之前运行的报告。 | 否 |
| `compare_runs` | 并排对比两次运行。 | 否 |
| `list_jobs` | 正在运行的内容。 | 否 |
| `stop_job` | 停止一个正在运行的任务。 | 是 |

### 实验 {#tools-experiments}

`describe_nodes` 是助手在编写实验之前读取的内容；它与 [`signallab nodes`](cli.md#cli-nodes) 相同。`list_templates` 和 `get_template` 提供可运行或改编的可用示例。

`validate_experiment` 和 `run_experiment` 以三种方式之一接收实验，且只能用其中一种：

| 参数 | 含义 |
| --- | --- |
| `document` | 实验文档，与应用保存的一致。 |
| `file` | 运行 `signallab` 的机器上某个实验文件的路径。 |
| `template` | 某个内置模板的名称。 |
| `params` | 本次运行的参数值：`{"name": "value"}`；数字和布尔值按文本处理。 |
| `profile` | 使用文档中的这个配置运行；`""` 表示使用默认值。 |
| `seed` | `run_experiment`：随机值的种子。 |
| `timeout` | `run_experiment`：运行可以花费的秒数，1 到 300（默认 300）。 |

`run_experiment` 在运行结束时作答：通过、失败或被停止，其时长和种子，每个步骤做了什么或为何失败，每个模拟器被询问了什么，每个损伤中继做了什么，以及报告的路径。失败的运行是一个正常的答复（步骤会说明原因），而不是一次失败的调用。

### 单条消息 {#tools-send}

| 工具 | 参数 |
| --- | --- |
| `send_osc` | `target`（`host:port`）、`address`、`args`：数字（整数 → int32，超出其范围时为 int64；否则为 float32）、字符串、布尔值、`null`，或 `{"type": "int"\|"float"\|"str"\|"long"\|"double"\|"bool"\|"blob"\|"nil", "value": …}`。 |
| `send_udp` | `target`，以及 `text` 或 `hex`（`"de ad be ef"`）。 |
| `send_http` | `method`、`url`、`headers`（`{"Name": "value"}`）、`body`、`timeout_ms`（默认 10000）、`auth`：`{"scheme": "basic"\|"digest", "username", "password"}` 或 `{"scheme": "bearer", "token"}`。返回状态、用时、标头和正文（取其前 16 KiB）。 |
| `send_mqtt` | `broker`（`host:port`，未指定时端口为 1883）、`topic`、`payload`、`qos`（0、1 或 2）、`retain`。带 `retain` 的空载荷会清除保留值。 |
| `send_ws` | `url`（`ws://` 或 `wss://`）、`text` 或 `hex`、`headers`、`protocols`，以及用于等待应答的 `expect`（包含）、`expect_regex` 或 `wait`（任意消息）；`timeout_ms` 1 到 120000（默认 2000）。返回握手信息、发送的内容和应答；应答是 JSON 时会解析其 JSON。 |

这些是应用各界面所用的命令；参见 [`signallab send`](cli.md#cli-send)。

### 监听 {#tools-listen}

`listen` 会在**运行 `signallab mcp` 的机器上**打开一个 UDP 端口，持续一段时间，并返回到达的内容：OSC 消息已解码，其他数据报以文本和十六进制形式给出。

| 参数 | 含义 | 默认值 |
| --- | --- | --- |
| `bind` | `IP:port`，例如 `0.0.0.0:9000`。 | 必填 |
| `protocol` | `osc` 或 `udp`。 | `osc` |
| `seconds` | 监听多长时间，0.1 到 60。 | 5 |
| `max` | 收到这么多数据报后停止，1 到 1000。 | 100 |

当 `0.0.0.0` 上什么都没有到达时，答复会提醒助手检查防火墙（[`signallab doctor`](cli.md#cli-doctor)）。使用 `--server` 时，`listen` 会被拒绝：在服务器上，带等待节点的实验会在那里监听。

### 信号与模拟器 {#tools-library}

`list_signals` 和 `fire_signal` 使用您的信号库：应用的 `signals.json`、`--library`，或调用时给出的 `library` 路径。信号通过其 id 或名称触发，与应用触发它的方式完全一致。

`list_emulators` 列出您库中的模拟器。`start_emulator` 启动其中一个（`emulator` 中的文档，或 `name` 中库条目的 id 或名称），并返回它的任务 id 和地址；它按自己的规则应答，直到 `stop_job`。`bind` 把它移到另一个 `IP:port`，`params` 给出其模板读取的值，`seed` 固定其随机选择。`emulator_exchanges`（`job_id`，以及只要较新内容时用的 `after`）列出到达了什么、每条规则应答了什么。`set_emulator_down`（`job_id`、`down`，以及 `fault`：`unavailable`、`reset` 或 `timeout`）会让正在运行的模拟器“拔掉电源”，直到被重新恢复：HTTP 会遇到相应的故障（`unavailable` 应答 503），TCP 设备和 MQTT 代理会断开连接，OSC 和 UDP 不作任何应答。参见[模拟器](../tools/emulators.md)。

### 运行与任务 {#tools-runs}

`list_runs` 读取之前运行的报告，最新的在最前——用 `experiment` 指定名称时只读取该实验的——最多 `limit` 个（1 到 500，默认 50），并附带每个负载步骤的数字。`compare_runs` 接收其中两个的名称，`a`（之前）和 `b`（之后），把每个负载步骤的延迟、错误率、达到的速率和错过的请求并排列出，并把朝不利方向变化 5% 或更多的标记为性能退化。参见[运行与报告](../experiments/runs.md)。

`list_jobs` 列出正在运行的内容（监视器、生成器、模拟器、运行），`stop_job` 按 id 停止其中一个。

## 结果与错误 {#results}

每个答复都是给模型看的文本，同时也是同样内容的结构化数据。失败会被标记为错误，并带有引擎的错误：稳定的 `code`、它的值，以及所涉及的节点和字段，用 `--lang` 所选的语言表述。助手传错的参数会以它能据此纠正的措辞返回。

## 进度与取消 {#progress}

当客户端要求 `run_experiment` 报告进度时，每个步骤发生时都会被报告（节点及其状态），因此助手和您都能看到运行在推进。取消一次调用会停止它；取消 `run_experiment` 会停止运行本身，就像应用中的 [[ui:common.stop]] 一样。

当客户端关闭连接时，仍在进行的调用会完成，然后 `signallab mcp` 退出。

## 在实验室服务器上 {#on-a-server}

使用 `--server http://192.0.2.10:1430` 时，实验、发送、信号和模拟器都在**那台服务器上**通过其 API 进行——使用它的网络、它的机密和它的数据文件夹——因此助手能访问只有实验室才能访问的设备。请在客户端的环境中提供令牌：

```json
{
  "mcpServers": {
    "signallab": {
      "command": "signallab",
      "args": ["mcp", "--server", "http://192.0.2.10:1430"],
      "env": { "SIGNALLAB_TOKEN": "<the server's token>" }
    }
  }
}
```

留在本机的内容：信号库和模拟器库（应用的，或 `--library` 和 `--emulators` 指定的），以及调用中指定的文件（`file`、`library`）都在本机读取，其中的内容会发送到服务器；`listen` 会被拒绝。参见[将 Signal Lab 作为服务器运行](../server/index.md)。

## 安全 {#safety}

- 助手只能做工具能做的事，而每个工具都是应用自己的命令之一：它无法访问应用无法访问的任何东西。
- 会发送、监听或启动某些东西的工具都被标记为触及外部世界；是否在每次调用前询问您，由您的客户端决定。
- 机密值永远不会到达助手：实验以 `{{secret.NAME}}` 引用它们，每个结果中它们的位置都显示为 `••••`。
- 模拟器或监听器会在其所运行的机器上打开一个端口；`list_jobs` 和 `stop_job` 可以显示并结束仍在运行的内容。

## 协议 {#protocol}

面向客户端开发者：基于 stdio 的 JSON-RPC 2.0，每行一条消息；标准输出只承载协议消息，任何给人看的内容都输出到标准错误。支持的协议版本为 `2025-06-18`、`2025-03-26` 和 `2024-11-05`（客户端请求其他版本时，使用最新的），以及批处理、`ping`、`tools/list` 和 `tools/call`；对发送了 `progressToken` 的调用，进度以 `notifications/progress` 报告，取消通过 `notifications/cancelled` 进行。服务器的 `instructions` 告诉模型这些工具如何配合使用。

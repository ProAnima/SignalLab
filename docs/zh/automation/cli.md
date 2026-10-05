---
title: 命令行
description: signallab 无需窗口即可运行实验、发送单条消息、扮演模拟器并检查网络，可在终端、脚本或 CI 作业中使用。
---

# 命令行：`signallab`

`signallab` 是无需窗口的 Signal Lab。它把实验运行到结束，并以脚本能理解的退出码退出；发送一条 OSC 消息、数据报、HTTP 请求、WebSocket 消息或 MQTT 发布；触发信号库中的一个信号；扮演一个模拟器直到您停止它；并说明这台机器与设备之间隔着什么。

它与应用使用同一个引擎：从命令行进行的运行采取相同的步骤、写入相同的报告、说相同的话，并使用界面的语言。加上 `--server` 时，运行改为在 [Signal Lab 服务器](../server/index.md)上进行，使用它的网络和它的机密。

```bash
signallab run tests/smoke.json --param api=http://127.0.0.1:8080 --junit junit.xml
signallab run flaky-api                                # a bundled template, by name
signallab validate tests/*.json                        # the editor's check, nothing sent
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab fire "Fader value"
signallab emulate tests/orders-api.json --for 120      # play a dependency for two minutes
signallab doctor                                       # firewall, network, data folder
```

流水线请参见 [CI 中的 Signal Lab](ci.md)；AI 助手请参见 [`signallab mcp`](mcp.md)。

## 安装 {#install}

| 位置 | 如何获得 `signallab` |
| --- | --- |
| Windows，安装程序（`.exe`） | 安装在应用旁边，且该文件夹会加入 `PATH`——“仅为我”安装时是您的，“为所有人”安装时是整台计算机的。安装后请打开新终端。安装程序的 `/NOPATH` 开关会保持 `PATH` 不变。 |
| Windows，`.msi` | 安装在应用旁边；应用安装期间，安装文件夹会加入整台计算机的 `PATH`。 |
| Linux，`.deb` 和 `.rpm` | `/usr/bin/signallab`。 |
| Linux，AppImage | 未包含：请使用下面的压缩包。 |
| 任何机器，未安装应用 | 每个发布版本都提供 `signallab-<version>-windows-x64.zip` 和 `signallab-<version>-linux-x64.tar.gz`，各自包含程序及其许可证；同一页面上的 `SHA256SUMS.txt` 列出它们的校验和。 |
| 服务器镜像 | `ghcr.io/proanima/signallab` 中的 `/usr/local/bin/signallab`（参见 [CI](ci.md#docker)）。 |

用以下命令检查：

```bash
signallab version
```

## 命令 {#commands}

| 命令 | 作用 |
| --- | --- |
| [`run`](#cli-run) | 依次运行实验；只有当每个实验都通过时才以 0 退出。 |
| [`validate`](#cli-validate) | 像编辑器在运行前那样检查实验；不发送任何内容。 |
| [`send`](#cli-send) | 发送一条消息：`osc`、`udp`、`http`、`ws` 或 `mqtt`。 |
| [`fire`](#cli-fire) | 按 id 或名称发送信号库中的一个信号。 |
| [`emulate`](#cli-emulate) | 扮演 HTTP API、OSC、UDP 或 TCP 设备，或 MQTT 代理，直到 <kbd>Ctrl</kbd>+<kbd>C</kbd> 或 `--for`。 |
| [`emulators`](#cli-emulators) | 列出应用库中的模拟器。 |
| [`templates`](#cli-templates) | 列出内置的实验模板。 |
| [`nodes`](#cli-nodes) | 以 JSON 描述每一种节点。 |
| [`mcp`](#cli-mcp) | 通过 Model Context Protocol 把 Signal Lab 提供给 AI 助手。 |
| [`doctor`](#cli-doctor) | 检查防火墙、网络、数据文件夹和服务器。 |
| [`firewall`](#cli-firewall) | `firewall allow`：让其他机器可以穿过 Windows 防火墙访问 Signal Lab。 |
| [`version`](#cli-version) | 打印版本。 |

`signallab help <command>` 或 `signallab <command> --help` 会打印某个命令的选项。

## 每个命令的选项 {#global-options}

| 选项 | 作用 | 默认值 |
| --- | --- | --- |
| `--lang <code>` | 消息所用的语言：`en`、`ru`、`es`、`fr`、`de`、`pt`、`zh`、`ja`、`ko`、`hi` 或 `ar`。 | `SIGNALLAB_LANG`，否则为系统区域设置，否则为 `en` |
| `--json` | 在标准输出上输出给机器用的内容，而不是文本（参见[输出](#output)）。 | 关 |
| `-h`, `--help` | 该命令的帮助。 | |
| `-V`, `--version` | 版本；可放在任何命令之前，写作 `signallab --version`。 | |

`--lang` 和 `--json` 都可以放在命令行的任何位置：`signallab --json run smoke.json` 与 `signallab run smoke.json --json` 相同。

### 语言 {#language}

消息、步骤文本、失败信息和 JUnit 报告都使用界面自身的文本、复数形式和数字风格。语言的确定顺序如下：

1. `--lang`；
2. `SIGNALLAB_LANG`（`ru`、`ru-RU` 和 `ru_RU.UTF-8` 都表示俄语）；
3. 系统区域设置：`LC_ALL`、`LC_MESSAGES` 和 `LANG` 中第一个已设置的；
4. 英语。

::: tip
Windows 终端通常不设置任何区域设置变量，因此除非您设置 `SIGNALLAB_LANG` 或传入 `--lang`，否则 `signallab` 在那里说英语。
:::

### 给人看与给机器看的输出 {#output}

不加 `--json` 时，结果写到**标准输出**，而沿途给人读的一切都写到**标准错误**：运行进行中的每个步骤、某件事为何失败、报告在哪里。`signallab run … 2>/dev/null` 会为每次运行只留下一个结论行。

加上 `--json` 时，标准输出只承载 JSON，不承载其他内容，标准错误保持安静：

| 命令 | `--json` 在标准输出上打印的内容 |
| --- | --- |
| `run` | 每行一个对象：`started`、每个步骤一个 `step`、`ended`（运行的结果），然后是 `summary`；无法启动的运行为 `error`。参见 [`run` 打印什么](#run-output)。 |
| `validate` | 每个实验每行一个对象：`valid`，以及 `profile_issues` 或 `error`。 |
| `send`, `fire` | `{"type": "sent", "result": …}`；`send http` 打印 `{"type": "response", "response": …}`，`send ws` 打印 `{"type": "exchange", "result": …}`。 |
| `emulate` | 每行一个对象：`started`、每个请求一个 `exchange`、每个模拟器一个 `summary`；带 `--check` 时为 `valid`。 |
| `emulators` | `{"path": …, "emulators": [{id, name, protocol, bind, rules, note}, …]}`。 |
| `templates` | `[{"name": …, "experiment": …}, …]`。 |
| `doctor` | 一个对象：`version`、`network`、`data_dir`、`firewall`、`server`、`problems`。 |
| `version` | `{"version": "…"}`。 |
| `nodes` | 始终是 JSON，无论是否带 `--json`。 |

失败是 `{"type": "error", "error": {…}, "exit_code": N}`。`error` 是引擎的错误：稳定的 `code`（例如 `transport.refused` 或 `secret.missing`）、它的 `params`，以及所涉及的 `node` 和 `field`。脚本可以用任何语言根据 `error.code` 分支；每个代码的文本都列在[错误消息](../reference/errors.md)中。

### 退出码 {#exit-codes}

| 退出码 | 含义 |
| --- | --- |
| `0` | 每个实验都通过；发送成功；没有任何阻碍。 |
| `1` | 某个实验运行后失败、超时或被停止；发送失败（被拒绝、没有应答、意外的状态）；`doctor` 发现有阻碍。 |
| `2` | 命令或文档有误：某个参数、无法读取的文件、验证错误、未知参数、缺少机密。 |
| `3` | 出于实验之外的原因什么都无法运行：服务器无法访问或拒绝令牌、端口无法打开、凭据存储失败。 |

有多个实验时，最严重的结果决定退出码，顺序为：`2`，然后 `3`，然后 `1`，然后 `0`。

### 环境变量 {#environment}

| 变量 | 作用 |
| --- | --- |
| `SIGNALLAB_LANG` | 未给出 `--lang` 时所用的语言。 |
| `LC_ALL`, `LC_MESSAGES`, `LANG` | 上面两者都未设置时所用的语言。 |
| `SIGNALLAB_SERVER` | `run`、`validate`、`emulate`、`mcp` 和 `doctor` 所用的服务器，如同 `--server` 给出的那样。 |
| `SIGNALLAB_TOKEN` | 未给出令牌文件时，服务器的访问令牌。 |
| `SIGNALLAB_TOKEN_FILE` | 存放服务器令牌的文件，如同 `--token-file` 给出的那样。 |
| `SIGNALLAB_SECRET_<NAME>` | 本进程中运行的机密 `NAME` 的值（参见[机密](#secrets)）。 |
| `SIGNALLAB_DATA_DIR` | 应用的数据文件夹，`fire`、`emulators`、`emulate`、`mcp` 和 `doctor` 默认查找的位置；否则为您主文件夹中的 `Documents/SignalLab`。 |
| `GITHUB_ACTIONS` | 当它为 `true` 时，未通过的运行还会以 `::error` 标注打印，GitHub 会在运行的页面上显示它。 |

## `run` {#cli-run}

```text
signallab run [OPTIONS] <FILE>...
```

依次运行实验，只有当每个实验都通过时才以 `0` 退出。

| 选项 | 作用 | 默认值 |
| --- | --- | --- |
| `<FILE>...` | 实验文件，或[内置模板](#cli-templates)的名称。 | 必填 |
| `-p`, `--param NAME=VALUE` | 本次运行的一个参数值；可重复给出更多。 | 文档中的值 |
| `--profile NAME` | 使用此配置文件运行；给出的每个实验都必须有它。`""` 表示使用默认值运行。 | 文档中的配置文件 |
| `-m`, `--matrix NAME=V1,V2` | 每个值运行一次；可重复给出更多名称（参见[运行矩阵](#matrix)）。 | |
| `--matrix-file PATH` | 从 JSON 文件读取组合。 | |
| `--seed N` | 随机值的种子，0 到 9007199254740991（2⁵³ − 1）。 | 文档中的种子，否则每次运行一个新种子 |
| `--timeout SECONDS` | 运行耗时更长时视为失败，1–300。 | `300` |
| `--fail-fast` | 在第一个未通过的运行处停止；其余不启动。 | 关 |
| `--junit PATH` | 在那里写入一份 JUnit XML 报告（参见[报告](#reports)）。 | |
| `--report PATH` | 把运行报告复制到那里：一次运行是一个文件，多次运行是一个文件夹。 | |
| `--data-dir PATH` | 本进程中运行所用的数据文件夹；它们的报告留在那里。 | 临时文件夹，退出时删除 |
| `--server URL` | 在这台服务器上运行，而不是在本进程中（参见[在服务器上](#run-on-server)）。 | `SIGNALLAB_SERVER` |
| `--token-file PATH` | 存放服务器令牌的文件。 | `SIGNALLAB_TOKEN_FILE`，否则 `SIGNALLAB_TOKEN` |
| `--secrets files\|system` | 本进程中机密值的来源。 | `files` |
| `--secrets-dir PATH` | 存放机密文件的文件夹，每个名称一个文件。 | 存在时为 `/run/secrets/signallab` |

`--data-dir`、`--secrets` 和 `--secrets-dir` 针对的是本进程中的运行；它们不能与 `--server` 同用。

### 文件与模板 {#run-inputs}

`FILE` 是应用保存和导出时的实验（[[ui:nav.experiment]] 界面上的 [[ui:exp.exportJson]]）。应用能打开的任何文档版本都可以用；较旧的版本会在读取时被更新，就像应用打开它时那样。不是文件的名字会在[内置模板](#cli-templates)中查找，带或不带 `.json` 均可：

```bash
signallab run tests/stage-cues.json tests/api.json
signallab run osc-ping-reply --param device=192.0.2.20:9000
```

每个实验——以及矩阵的每种组合——都会像编辑器那样在第一个运行之前被检查。第三个文件有问题时，第一个也不会运行，且在发送任何内容之前就停止，退出码为 `2`。

### 参数与配置文件 {#run-params}

`--param NAME=VALUE` 仅为本运行设置一个参数；文件不会被更改。值是第一个 `=` 之后的所有内容，因此 `--param url=http://127.0.0.1/?q=1` 可用，而 `--param note=` 设置一个空值。一个值会应用到给出的、拥有该参数的每个实验，且必须至少是其中一个实验的参数——拼错的名称会被拒绝，退出码为 `2`。

`--profile NAME` 使用实验的某个配置文件运行，如同在编辑器中于 [[ui:exp.profile]] 下选择它那样。参见[数据与模板](../experiments/data.md)。

### 运行矩阵 {#matrix}

矩阵会让同一个实验针对每个目标、每个用户、每种载荷大小各运行一次。每种组合都是一次独立的运行，有自己的结果、自己的报告和自己在这份 JUnit 报告中的测试套件。

```bash
signallab run smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest \
  --fail-fast --junit junit.xml
```

这就是四次运行：`device` 变化最慢，`user` 最快。它们以文件名及其值命名：`smoke.json [device=192.0.2.20:9000, user=admin]`。

- `--matrix NAME=V1,V2` 添加一个轴。再次给出同一名称会为它添加值。名称和值周围的空格会被去掉；同一个值给两次只运行一次。
- `--matrix-file PATH` 读取两种形状之一的 JSON：

  ```json
  { "device": ["192.0.2.20:9000", "192.0.2.21:9000"], "retries": [1, 3] }
  ```

  添加轴——每种组合都会运行，这些名称排在 `--matrix` 的名称之后，按字母顺序；

  ```json
  [
    { "device": "192.0.2.20:9000", "user": "admin" },
    { "device": "192.0.2.21:9000", "user": "guest" }
  ]
  ```

  直接列出组合本身，每个都与 `--matrix` 的轴交叉。值可以是文本、数字或 `true`/`false`；文件中某个值内部的逗号仍是它的一部分。
- 每个矩阵名称都必须是给出的至少一个实验的参数。没有其中某个参数的实验只运行一次，而不是为它本会忽略的每个值各运行一次。
- 同时由 `--param` 和矩阵设置的名称，或同时由 `--matrix` 和文件设置的名称，会被拒绝。
- 一条命令最多产生 **256** 次运行；更多会在任何运行开始之前被拒绝。

### 在服务器上运行 {#run-on-server}

```bash
signallab run tests/stage.json --server http://192.0.2.10:1430 --token-file token.txt
```

实验来自本机；服务器使用它自己的网络、它自己的[机密](../server/index.md#secrets)和它自己的数据文件夹来运行它，并在每个步骤发生时把它流式送回。报告留在服务器上——会打印其路径——而 `--report` 会下载一份副本。在服务器上的运行就像在其中启动的运行一样，是那里的一个任务：登录它的每个页面都能看到它。如果 `signallab` 在运行中途退出，该运行仍会在服务器上完成并保留其报告。

令牌从 `--token-file`（或 `SIGNALLAB_TOKEN_FILE`）读取，否则从 `SIGNALLAB_TOKEN` 读取，并作为 `Authorization: Bearer` 发送。无法访问、拒绝令牌或在运行中途停止应答的服务器，退出码为 `3`。`signallab doctor --server URL` 会单独检查地址和令牌。

### 报告 {#reports}

每次运行都写入与应用相同的报告。不加 `--data-dir` 时，运行使用一个临时文件夹，`signallab` 退出时它会被删除，因此请保留您需要的内容：

- `--report PATH` 复制报告：一次运行时复制到 `PATH` 本身，多次运行时复制到文件夹 `PATH` 中，按运行顺序命名为 `01-<file>.json`、`02-<file>.json`……
- `--data-dir PATH` 把每次运行的报告都保存在该文件夹中（`runs/` 下），并打印每一份的位置。

`--junit PATH` 写入一份 JUnit XML 报告，即每个 CI 系统都能读取的格式：

- 每次运行（矩阵的每种组合）一个 `<testsuite>`，以属性形式包含文件、种子、结果、配置文件、报告路径和每个矩阵值（`param.NAME`）；
- 每个运行过的节点一个 `<testcase>`，以节点的类型及其 id 命名，并带有它自己的时间；
- 失败的节点上有一个 `<failure>`：用所选语言写成的消息、作为其 `type` 的错误代码，以及带有技术细节的该节点步骤；
- 运行从未到达的每个节点（分支的另一侧）有一个 `<skipped>` 用例；
- 无法启动的实验对应一个带 `<error>` 用例的套件，`--fail-fast` 未启动的每次运行则各有一个带 `<skipped>` 用例的套件。

```xml
<testsuites name="Signal Lab" tests="6" failures="1" errors="0" time="2.006">
  <testsuite name="HTTP to OSC" tests="6" failures="1" errors="0" skipped="4" time="2.006" timestamp="2026-10-04T16:48:03">
    <properties>
      <property name="file" value="status-branch" />
      <property name="seed" value="42" />
      <property name="outcome" value="failed" />
      <property name="report" value="out/report.json" />
    </properties>
    <testcase name="Start (start)" classname="HTTP to OSC" time="0.001">
      <system-out>Started · seed 42</system-out>
    </testcase>
    <testcase name="HTTP request (request)" classname="HTTP to OSC" time="2.004">
      <failure message="http://127.0.0.1:8080/ refused the connection — nothing is listening on that port" type="transport.refused">…</failure>
    </testcase>
    <testcase name="Status branch (branch)" classname="HTTP to OSC" time="0.000">
      <skipped message="not reached in this run" />
    </testcase>
    …
  </testsuite>
</testsuites>
```

[负载](../experiments/load.md)下的 HTTP 节点会在其最后一个步骤下为每个阈值添加一行，无论是否守住（`✕ p95 < 100 ms · 152.58 ms`）；没有守住的阈值会使该运行及其 JUnit 用例失败（`type="load.threshold"`）。

### 机密 {#secrets}

实验以 `{{secret.NAME}}` 读取机密。对于本进程中的运行，值来自：

| `--secrets` | `NAME` 的值的来源 |
| --- | --- |
| `files`（默认值） | 环境变量 `SIGNALLAB_SECRET_NAME`；否则是 `--secrets-dir` 中名为 `NAME` 的文件（默认为 `/run/secrets/signallab`，前提是该文件夹存在——即 Docker 机密的布局）。 |
| `system` | Windows 凭据管理器——应用保存其 [[ui:exp.secrets]] 值的地方。Linux 没有 `signallab` 能读取的凭据存储：在那里 `--secrets system` 会以退出码 `3` 失败。 |

文件末尾的换行不属于值的一部分，空值算作未设置。名称由字母、数字和 `_` 组成，不能以数字开头，最多 128 个字符；值最多 16 KiB。

```bash
SIGNALLAB_SECRET_API_TOKEN="$API_TOKEN" signallab run tests/api.json
```

未设置的机密会在任何流量发出之前停止运行，退出码为 `2`，并给出缺少的名称。值从不打印：步骤、错误、报告和 JUnit 报告都在其位置显示 `••••`。使用 `--server` 时，机密是服务器的。

### `run` 输出什么 {#run-output}

运行进行时，每个步骤是标准错误上的一行——自开始以来的时间、节点、其状态和它做了什么——就像应用的时间线。结束时，标准输出上的一行说明结果如何：

```text
▶ HTTP check (http-check) · seed 1185927457137919
     0.000  Start         Running
     0.000  Start         Passed · Started · seed 1185927457137919
     0.000  HTTP request  Running
     2.004  HTTP request  Failed · URL — http://127.0.0.1:8080/ refused the connection — nothing is listening on that port
✖ HTTP check failed after 2 s: HTTP request · URL — http://127.0.0.1:8080/ refused the connection — nothing is listening on that port
  Technical details: error sending request for url (http://127.0.0.1:8080/): … (os error 10061)
  To run it again with the same random values: --seed 1185927457137919
```

失败的运行会给出它所用的种子：把该数字传给 `--seed` 会用相同的随机值再次运行它。结论之后，标准错误上会给出运行的每个模拟器被请求了什么——请求数、有多少没有规则接收、有多少失败，以及每条规则的命中数：

```text
✔ Retry a flaky API passed in 7 ms
  Flaky API: 3 requests, 0 without a rule, 0 failed · #1 3
```

以及每个损伤中继做了什么：它接收、丢弃和限速了多少，以及每个阶段的转发 / 接收：

```text
  127.0.0.1:19110 → 127.0.0.1:19100: 155 datagrams, 38 dropped, 0 throttled · lan 0.0–2.0 s 39/39, wifi 2.0–4.0 s 39/39, offline 4.0–6.0 s 0/38, lan 6.0–8.0 s 38/38
```

TCP 上的中继搬运的是流的数据块，而不是数据报，并且不丢弃任何内容，因此它的行统计数据块、连接、重置、半开连接，以及流被带宽限制拖住的次数：

```text
  127.0.0.1:19120 → 127.0.0.1:19101: 14 chunks, 2 connections, 1 reset, 0 half-open, 3 held back
```

[负载](../experiments/load.md)下的 HTTP 节点以步骤形式报告进度，最多每秒一次。

有多次运行时，每次运行一行，最后以一行计数结束输出：`3 runs: 2 passed, 1 failed`。使用 `--fail-fast` 时，未启动的运行会在标准错误上计数。

使用 `--json` 时，每一行都是一个带 `type` 的对象：

```json
{"type":"started","experiment":"Empty experiment","file":"empty","job_id":1,"overridden":false,"profile":null,"seed":1,"started_ms":1791132204585}
{"type":"step","node_id":"start","state":"passed","detail":"Started","message_key":"exp.step.started","message_params":{"seed":1},"job_id":1,"ts":1791132204585}
{"type":"ended","experiment":"Empty experiment","file":"empty","outcome":"passed","seed":1,"params":{},"steps":[…],"report_path":"…","started_ms":1791132204585,"ended_ms":1791132204585,…}
{"type":"summary","total":1,"passed":1,"failed":0,"not_started":0,"exit_code":0}
```

`started`、`ended` 和 `error` 行带有 `file`（给出的参数原样），在矩阵中还带有 `matrix`（该组合的值）。`ended` 是运行的完整结果：`outcome`（`passed`、`failed` 或 `stopped`）、`seed`、`profile`、`params`、`error`、每个步骤，以及运行有的话还有 `emulators` 和 `impairments`，以及 `report_path`。负载的最后一个步骤带有 `load`，其中包含它测到的每个数字：`planned`、`sent`、`ok`、`failed`、`missed`、`rps`、`error_rate`、`min_ms`、`mean_ms`、`max_ms`、`p50_ms` 到 `p99_ms`、`statuses`、每一秒（`seconds`）、`histogram`，以及每个阈值的判定（`thresholds`）。

## `validate` {#cli-validate}

```text
signallab validate [OPTIONS] <FILE>...
```

像编辑器在运行前那样检查实验——图、每个字段、模板、参数和机密——不发送任何内容。当每个实验都能启动时以 `0` 退出。

它接受 [`run`](#cli-run) 的输入：文件和模板、`--param`、`--profile`、`--matrix`、`--matrix-file`，以及 `--server`、`--token-file`、`--secrets`、`--secrets-dir`。使用 `--server` 时，由服务器对照它自己的机密来检查它们。

```text
✔ tests/stage.json: Stage cues would run
  Rehearsal: Would not run: …
```

只有文档的另一个配置文件才有的问题会列在它之下，但不会使检查失败。使用 `--json` 时，每个实验（每种组合）一行：`{"experiment", "file", "valid": true, "profile_issues": […]}`，或带 `error` 和 `exit_code` 的 `"valid": false`。

## `send` {#cli-send}

```text
signallab send <osc|udp|http|ws|mqtt> …
```

通过应用各界面所用的同一批命令发送一条消息，因此线路上的字节完全相同。发送不读取任何库，也不读取任何机密。

退出码：`0` 已发送，`1` 发送失败，`2` 某个参数有误。

一个 `<host:port>` 是 IP 地址或主机名加端口：`127.0.0.1:9000`、`[::1]:9000`（方括号中的 IPv6 地址）或 `device.local:9000`。名称在命令运行时解析，当它有 IPv4 地址时就使用该地址，因此 `localhost:9000` 能到达监听在 `127.0.0.1` 上的接收方。无法解析的名称会使发送失败（`1`）；没有端口的目标是无效参数（`2`）。

### `send osc` {#cli-send-osc}

```text
signallab send osc <host:port> <address> [ARG]...
```

一条 OSC 消息。参数可带前缀给出类型，或自动推断：

| 参数 | OSC 类型 |
| --- | --- |
| `i:3` | int32 |
| `f:0.5` | float32 |
| `d:1.5` | float64（double） |
| `h:64` | int64 |
| `s:text` | 字符串 |
| `b:de ad be ef` | blob，以十六进制字节表示 |
| `T`、`F` | true、false |
| `N` | nil |
| `3`、`-3` | 普通整数是 int32 |
| `2.5` | 普通小数是 float32 |
| 其他任何内容 | 字符串 |

```bash
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main 3
# ✔ sent /cue/go (32 bytes) → 127.0.0.1:9000
```

带空格的参数要加引号：`"s:hello world"`。`s:7` 发送文本 `7`。地址必须以 `/` 开头；不这样的地址是无效参数（`2`）。

::: warning Git Bash on Windows
Git Bash 会把以 `/` 开头的参数改写成 Windows 路径，因此 `/cue/go` 会变成 `C:/Program Files/Git/cue/go`。请写成 `//cue/go`，或带 `MSYS_NO_PATHCONV=1` 运行。PowerShell 和 `cmd` 不受影响。
:::

参见 [OSC](../protocols/osc.md)。

### `send udp` {#cli-send-udp}

```text
signallab send udp <host:port> (--text TEXT | --hex HEX)
```

一个 UDP 数据报。载荷可用 `--text` 给出，或用 `--hex` 字节：`"de ad be ef"`、`deadbeef` 或 `0xDE,0xAD`。

```bash
signallab send udp 127.0.0.1:7000 --text "PLAY 1"
signallab send udp 127.0.0.1:7000 --hex "de ad be ef"
```

### `send http` {#cli-send-http}

```text
signallab send http <METHOD> <URL> [OPTIONS]
```

一个 HTTP 请求。状态行写到标准错误——`HTTP 200 OK · 3 ms · 1,234 B`——响应正文写到标准输出，因此可以管道传递下去。超过 256 KiB 的正文会在那里被截断，标准错误会说明这一点。

| 选项 | 作用 | 默认值 |
| --- | --- | --- |
| `-H`, `--header "Name: value"` | 一个请求标头；可重复添加更多。 | |
| `--body TEXT` | 请求正文。`@FILE` 发送某个文本文件的内容。 | 无 |
| `--expect-status STATUS` | 除非响应具有此状态，否则以 `1` 退出。 | 任何状态都是 `0` |
| `--timeout MS` | 等待响应的毫秒数。 | `10000` |
| `-u`, `--user NAME:PASSWORD` | 凭据，以 Basic 方式发送。 | |
| `--digest` | 与 `--user` 一起：改为应答服务器的 Digest 质询（MD5 或 SHA-256）。 | 关 |
| `--bearer TOKEN` | 发送 `Authorization: Bearer TOKEN`。不能与 `--user` 同用。 | |

```bash
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab send http POST http://127.0.0.1:8080/cue -H "Content-Type: application/json" --body '{"cue": 1}'
signallab send http GET http://127.0.0.1:8080/admin -u admin:secret --digest
```

不加 `--expect-status` 时，任何响应都算成功，`500` 也包括在内。没有得到响应的请求（被拒绝、超时、名称无法解析）退出码为 `1`，无法作答的 Digest 质询也是如此——原因会打印在响应之后。

::: tip
参数对同一台机器的其他用户可见。真实的密码请留给实验，在那里它们是[机密](#secrets)。
:::

参见 [HTTP](../protocols/http.md)。

### `send ws` {#cli-send-ws}

```text
signallab send ws <URL> [OPTIONS]
```

一次 WebSocket 交互：连接到 `ws://…` 或 `wss://…`，发送一条消息，可选地等待应答，然后关闭。握手和已发送的内容写到标准错误，应答写到标准输出（二进制消息以十六进制表示）。

| 选项 | 作用 | 默认值 |
| --- | --- | --- |
| `--text TEXT` | 发送这条文本消息。 | 不发送任何内容 |
| `--hex HEX` | 把这些字节作为二进制消息发送。不能与 `--text` 同用。 | |
| `-H`, `--header "Name: value"` | 升级请求的一个标头；可重复添加更多。 | |
| `--protocol NAME` | 要提供的子协议；可重复添加更多，按优先顺序排列。 | |
| `--expect TEXT` | 等待一条包含此文本的消息。 | |
| `--expect-regex REGEX` | 等待一条匹配此正则表达式的消息。 | |
| `--wait` | 等待任意消息。 | |
| `--timeout MS` | 等待应答的毫秒数。 | `2000` |

```bash
signallab send ws ws://127.0.0.1:9001/echo --text '{"ping": 1}' --expect '"ping"'
signallab send ws ws://127.0.0.1:9001/feed --wait        # nothing sent: the server's first message
```

不加 `--expect`、`--expect-regex` 或 `--wait` 时，它连接、发送并关闭，不等待。期望的应答没有及时到达时，退出码为 `1`。参见 [WebSocket](../protocols/websocket.md)。

### `send mqtt` {#cli-send-mqtt}

```text
signallab send mqtt <host:port> <topic> [payload] [--qos 0|1|2] [--retain]
```

一次 MQTT 3.1.1 发布，通过明文 TCP，不带凭据，作为拥有全新客户端 ID 的独立客户端，因此绝不会挤掉已有的连接。没有端口时，代理位于 `1883`。主题就是一个主题：含有 `+` 或 `#`，或为空时，命令会在连接之前被拒绝（`2`）。

| 选项 | 作用 | 默认值 |
| --- | --- | --- |
| `[payload]` | 载荷。 | 空 |
| `--qos 0\|1\|2` | 服务质量。 | `0` |
| `--retain` | 把它保留为该主题的保留值。带 `--retain` 的空载荷会清除它。 | 关 |

```bash
signallab send mqtt 127.0.0.1:1883 lab/light/1/set on --qos 1
signallab send mqtt 127.0.0.1 lab/light/1/state "" --retain     # clear the retained value
```

参见 [MQTT](../protocols/mqtt.md)。

## `fire` {#cli-fire}

```text
signallab fire <signal> [--library PATH]
```

完全像应用的 [[ui:nav.signals]] 界面那样发送信号库中的一个信号：OSC、UDP、HTTP 和 MQTT 信号。先按 id 查找该信号，然后按名称查找，不区分大小写；多个信号共用的名称会被拒绝，并给出它们的 id。

| 选项 | 作用 | 默认值 |
| --- | --- | --- |
| `<signal>` | 信号的 id 或名称。 | 必填 |
| `--library PATH` | 库文件。 | 应用数据文件夹中的 `signals.json` |

```bash
signallab fire "Fader value"
signallab fire go --library show/signals.json
```

库只被读取，从不创建或更改。参见[信号](../tools/signals.md)。

## `emulate` {#cli-emulate}

```text
signallab emulate [OPTIONS] <FILE|NAME>...
```

扮演另一端——HTTP API、OSC、UDP 或 TCP 设备、MQTT 代理——直到 <kbd>Ctrl</kbd>+<kbd>C</kbd> 或 `--for`，并在应答每个请求时打印它。`FILE` 包含一个模拟器、一个模拟器列表，或应用写出的整个库；`NAME` 是应用库（[[ui:nav.emulators]] 界面的）中某个模拟器的 id 或名称。

| 选项 | 作用 | 默认值 |
| --- | --- | --- |
| `-p`, `--param NAME=VALUE` | 其模板以 `{{NAME}}` 读取的值；可重复添加更多。 | |
| `--bind IP:PORT` | 改为在那里监听。仅适用于一个模拟器。 | 模拟器自己的 |
| `--for SECONDS` | 这么长时间后停止。 | 直到 <kbd>Ctrl</kbd>+<kbd>C</kbd> |
| `--seed N` | 其随机选择的种子：随机顺序、抖动、生成器。 | |
| `--check` | 检查模拟器并退出，不打开任何端口。 | 关 |
| `--library PATH` | 在其中查找名称的库。 | 应用数据文件夹中的 `emulators.json` |
| `--server URL`, `--token-file PATH` | 在服务器上启动它们，通过其 API 跟随它们，并在结束时停止它们。 | |
| `--secrets`, `--secrets-dir` | 与 [`run`](#cli-run) 相同。 | |

```text
$ signallab emulate tests/orders-api.json --for 60
Orders API (http) answering on 127.0.0.1:18099
answering for 60 s
+   1.209 s Orders API  #1  GET /orders/42 → 200 OK · 12 B  1 ms  ← 127.0.0.1:55744
+   1.209 s Orders API  —  GET /nothing → 404 Not Found · 20 B  2 ms  ← 127.0.0.1:55745
Orders API: 2 requests, 1 without a rule, 0 failed · #1 1
```

每一行是自开始以来的时间、模拟器、应答的规则（`#1`，没有则是 `—`）、请求及它得到的内容、所用时间以及发送者。结束时是每个模拟器的计数：请求数、有多少没有规则接收、有多少失败、有多少遇到停机或（在存在时）未能送达，以及每条规则的命中数。

退出码：在 <kbd>Ctrl</kbd>+<kbd>C</kbd> 或 `--for` 处停止时为 `0`；模拟器无效时为 `2`；其端口被占用或无法打开，或应答期间套接字出错时为 `3`。

在流水线中，把它放到后台启动，让被测系统针对它测试，最后读取计数：

```bash
signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
npm test          # the system under test, configured for the emulator's address
wait              # the last lines of mock.ndjson are the counts
```

在服务器上，用 `--server` 启动的模拟器会在 `signallab` 正常结束时被停止；被强制结束的进程遗留的模拟器可以从服务器的界面上停止。参见[模拟器](../tools/emulators.md)。

## `emulators` {#cli-emulators}

```text
signallab emulators [--library PATH]
```

列出库中的模拟器：id、名称、协议、地址以及有多少条规则。`--library PATH` 改为读取另一个库文件，而不是应用数据文件夹中的 `emulators.json`。`signallab emulate <id>` 启动其中一个。

库只被读取。如果没有该文件——应用第一次启动时会在其数据文件夹中创建它——命令会说明这一点并以 `2` 退出。

## `templates` {#cli-templates}

```text
signallab templates
```

列出内置模板，`run` 和 `validate` 按名称接受它们。它们就是应用自己的模板：

| 名称 | 在应用中 | 参数 |
| --- | --- | --- |
| `empty` | [[ui:exp.templateEmpty]] | |
| `http-check` | [[ui:exp.templateHttp]] | |
| `status-branch` | [[ui:exp.templateBranch]] | |
| `parallel-flows` | [[ui:exp.templateParallel]] | |
| `osc-ping-reply` | [[ui:exp.templatePingReply]] | `device` = `127.0.0.1:9000` |
| `poll-until-ready` | [[ui:exp.templatePoll]] | `device` = `127.0.0.1:9000` |
| `flaky-api` | [[ui:exp.templateFlaky]] | `api` = `http://127.0.0.1:18080` |
| `fault-phases` | [[ui:exp.templateFaults]] | |
| `dependency-outage` | [[ui:exp.templateOutage]] | `api` = `http://127.0.0.1:18090` |
| `websocket-echo` | [[ui:exp.templateWsEcho]] | `service` = `ws://127.0.0.1:9001/echo` |

每个模板都指向环回地址。`flaky-api`、`fault-phases` 和 `dependency-outage` 自带模拟器，因此不需要其他东西在监听就能运行——这是快速查看 `signallab` 工作方式的办法。

## `nodes` {#cli-nodes}

```text
signallab nodes
```

以 JSON 打印实验由什么构成：文档的形状和规则、每一种节点及其标签、描述、字段、输出和一个引擎会接受的示例、`{{template}}` 语言、负载配置文件和模拟器文档。它正是助手通过 [`signallab mcp`](mcp.md) 读取以编写实验的内容；对于人，[节点](../experiments/nodes.md)用更多文字说了同样的事。

## `mcp` {#cli-mcp}

```text
signallab mcp [OPTIONS]
```

通过 Model Context Protocol 在标准输入和标准输出上把 Signal Lab 提供给 AI 助手。全部内容——在 Claude Code、Claude Desktop、Cursor 或 VS Code 中进行设置、它的选项和工具——都在[助手（MCP）](mcp.md)上。

## `doctor` {#cli-doctor}

```text
signallab doctor [--server URL] [--token-file PATH]
```

说明 Signal Lab 与设备之间可能有什么阻碍，有阻碍时以 `1` 退出。它的各行像命令行的其余部分一样遵循 [`--lang`](#language)：

- 这台机器所在的网络：其名称和地址；
- 数据文件夹：是否可以写入（尚未创建的也可以——应用在第一次使用时创建它）；
- 防火墙：在 Windows 上，就 `signallab` 和桌面应用各自而言，是否有规则允许其他机器在当前所处类型的网络（专用、域或公用）上访问，或有规则阻挡它们——即在系统提示处点击*取消*会留下的结果；在 Linux 上，ufw 或 firewalld 是否开启，以及打开端口的命令；
- 加上 `--server`（或 `SIGNALLAB_SERVER`）时：服务器是否应答并接受令牌。

```text
signallab [[version]]
Network: LAB-PC · 192.0.2.15
Data folder: C:\Users\lab\Documents\SignalLab — writable
Firewall · signallab (C:\…\Signal Lab\signallab.exe) · private network: not allowed yet — signallab firewall allow
Firewall · app (C:\…\Signal Lab\signal-lab.exe) · private network: allowed
✖ 1 thing in the way
```

`--json` 把同样的内容打印为一个对象。参见[故障排除](../reference/troubleshooting.md)。

## `firewall` {#cli-firewall}

```text
signallab firewall allow [--public]
```

在 Windows 上，让其他机器可以访问 Signal Lab——这是监视器或等待节点听到设备所需要的。Windows 会先要求管理员权限；然后 `signallab` 和桌面应用（在它旁边找到，或安装程序放置的位置）的入站规则会被各替换为一条允许规则（阻挡规则也会被替换），适用于专用网络和域网络。

| 选项 | 作用 |
| --- | --- |
| `--public` | 公用网络上也如此——场馆的 Wi-Fi 往往就是其中之一。 |

退出码：`0` 已完成；管理员提示被拒绝或更改失败时为 `3`。在 Linux 上不会更改任何内容：它打印打开您所监听端口的 ufw 或 firewalld 命令，并以 `0` 退出。

只有当您运行此命令时防火墙才会更改；`signallab` 中没有其他任何东西会碰它。

## `version` {#cli-version}

```text
signallab version
```

打印 `signallab [[version]]`——加上 `--json` 时打印 `{"version": "[[version]]"}`。`signallab --version` 打印相同的版本。

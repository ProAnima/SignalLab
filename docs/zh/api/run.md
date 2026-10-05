---
title: 运行
description: POST /api/run 在 Signal Lab 服务器上运行一个实验，并用其结果作答，或将其步骤作为 NDJSON 行流式发送。
---

# 运行实验

若想从脚本或流水线在服务器上运行一个实验并了解其结果，请将它发送到 `POST /api/run`。服务器会把它运行到结束，并用结果作答——或者，如果您要求，也会在每一步发生时给出该步骤。这正是 [`signallab run --server`](../automation/cli.md#cli-run) 所用的。

它与编辑器的 [[ui:exp.run]] 和 [`experiment_start`](commands.md#experiment_start) 是同一次运行：一个每个打开的页面都能看到并停止的任务，同样的事件，数据文件夹中同样的报告。

## 请求 {#request}

```http
POST /api/run
Authorization: Bearer <token>
Content-Type: application/json

{ "template": "osc-ping-reply", "overrides": { "device": "192.0.2.20:9000" }, "seed": 42, "timeout": 30 }
```

正文指明一个实验——`document` 或 `template`，二者不能都有——以及用什么来运行它：

| 字段 | 类型 | 默认值 | 含义 |
| --- | --- | --- | --- |
| `document` | object | — | 实验，与编辑器保存和导出的一致。较旧的版本会被迁移，如同打开文件时那样 |
| `template` | string | — | 按文件名指定的内置模板，可带或不带 `.json`（见下文） |
| `overrides` | object | `{}` | 仅用于本次运行的参数值。值可以是字符串、数字或布尔值；每个都必须是该实验的参数 |
| `profile` | string | 文档中的 | 用此配置运行；`""` 表示使用默认值运行 |
| `seed` | number | 文档中的，否则为新值 | 0 到 9007199254740991；相同的种子抽取出相同的随机值 |
| `timeout` | number | `300` | 运行以 `run.timeout` 失败前的秒数；1 到 300 |

服务器不认识的字段会被拒绝（`400`、`api.run_invalid`）。文档本身会像打开的文件那样读取，因此最大为 4 MiB。

内置模板——编辑器在 [[ui:exp.templates]] 下提供的实验：

| `template` | 内容 |
| --- | --- |
| `empty` | 开始和结束 |
| `http-check` | 对 `http://127.0.0.1:8080/` 的 GET，然后检查状态是否为 200 |
| `status-branch` | 对 `http://127.0.0.1:8080/` 的 GET；200 时发送一条 OSC 消息，否则延迟 500 ms |
| `parallel-flows` | 一个 [[ui:exp.node.fork]] 分成对 `http://127.0.0.1:8080/` 的 GET 和一条日志记录，并排，然后 [[ui:exp.node.join]] |
| `osc-ping-reply` | 向参数 `device`（`127.0.0.1:9000`）发送 OSC `/ping`，然后在 `127.0.0.1:9001` 上等待 `/pong` 2 s |
| `poll-until-ready` | 一个 [[ui:exp.node.loop]]，通过 OSC 向 `device` 询问 `/status`，直到它应答 `ready`，至多 10 次 |
| `flaky-api` | 一个模拟的 API（参数 `api`），在正常工作前会失败，在 [[ui:exp.node.loop]] 中询问它直到应答 200 |
| `fault-phases` | 一个位于损伤中继之后的 UDP 设备，向它发送 8 s，期间中继依次为干净、丢包、离线、再次干净 |
| `dependency-outage` | 一个模拟的 API（参数 `api`），停机 2 s，期间一个 [[ui:exp.node.loop]] 询问它直到它再次应答 200 |
| `websocket-echo` | 连接到参数 `service`（`ws://127.0.0.1:9001/echo`），发送一条消息，等待它的回显，检查，关闭 |

在 [[ui:exp.templates]] 下打开一个即可查看其节点和参数；参见[实验](../experiments/index.md)。

## 结果 {#result}

默认情况下，应答为 `200`，带有 `Content-Type: application/json`，在运行结束时发送：一个 JSON 对象，即运行的结果。

```json
{
  "job_id": 12,
  "experiment": "OSC ping → reply",
  "outcome": "passed",
  "seed": 42,
  "profile": null,
  "overridden": true,
  "params": { "device": "192.0.2.20:9000" },
  "started_ms": 1759600000000,
  "ended_ms": 1759600000310,
  "steps": [ { "job_id": 12, "ts": 1759600000001, "node_id": "start", "state": "running", "detail": "", "message_key": null, "message_params": null }, … ],
  "report_path": "/data/runs/run-1759600000000-12.json"
}
```

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `job_id` | number | 该运行的任务 |
| `experiment` | string | 实验的名称 |
| `outcome` | string | `passed`、`failed` 或 `stopped` |
| `seed` | number | 它运行所用的种子：把它作为 `seed` 传回即可抽取相同的值 |
| `profile` | string or null | 它运行所用的配置 |
| `overridden` | boolean | 部分值来自 `overrides` |
| `params` | object | 本次运行用到的每个参数值 |
| `started_ms`, `ended_ms` | number | 自 1970 年起的毫秒数 |
| `error` | `EngineError` | 它为何失败：其第一个失败。通过时省略 |
| `steps` | object[] | 按发生顺序排列的每个步骤，如 [`experiment://step`](events.md#event-experiment-step) |
| `emulators` | object[] | 每个 [[ui:exp.node.emulator]] 节点接收和应答的内容：`node`、`name`、`protocol`、`local`、`counts`。没有时省略 |
| `impairments` | object[] | 每个 [[ui:exp.node.impairment]] 节点的中继逐阶段做了什么。没有时省略 |
| `report_path` | string | 该运行在服务器上的报告；用 [`/api/files`](index.md#files) 下载它。未写入时省略 |
| `report_error` | `EngineError` | 报告为何无法写入。否则省略 |

其中的机密值都会被遮蔽。报告文件包含相同的步骤；参见[运行与报告](../experiments/runs.md)。

运行进行期间，服务器每 15 s 发送一个空格。JSON 会忽略值前的空白，因此结果仍能解析，代理也不会把一次漫长而安静的运行当成已断开的连接。

## 跟踪步骤 {#lines}

要看到步骤发生的过程，请请求 NDJSON：

```http
Accept: application/x-ndjson
```

应答为 `200`，带有 `Content-Type: application/x-ndjson`：每行一个 JSON 对象，每个都带有一个 `type`。

| `type` | 何时 | 该行的其余部分 |
| --- | --- | --- |
| `started` | 首先，一次 | `job_id`、`experiment`、`seed`、`profile`、`overridden`、`started_ms` |
| `step` | 每个步骤 | 该步骤，如 [`experiment://step`](events.md#event-experiment-step) |
| `heartbeat` | 每 15 s | 无 |
| `ended` | 最后，一次 | 结果，如上所述 |

```text
{"job_id":12,"experiment":"OSC ping → reply","seed":42,"profile":null,"overridden":true,"started_ms":1759600000000,"type":"started"}
{"job_id":12,"ts":1759600000001,"node_id":"start","state":"running","detail":"","message_key":null,"message_params":null,"type":"step"}
…
{"job_id":12,"experiment":"OSC ping → reply","outcome":"passed",…,"type":"ended"}
```

读取各行直到 `ended`；忽略您不认识的 `type`。服务器会发送 `X-Accel-Buffering: no`，因此 nginx 代理会立即把每一行传递下去。

## 状态与结果 {#status}

HTTP 错误状态意味着**没有运行启动**；正文是一个 [`EngineError`](index.md#errors)：

| 状态 | 代码 | 原因 |
| --- | --- | --- |
| `400` | `api.run_invalid` | 正文不是运行请求：不是 JSON、未知字段、不是字符串/数字/布尔值的覆盖值 |
| `400` | `api.run_source` | `document` 和 `template` 都没有，或二者都有 |
| `415` | `command.json_required` | 不是 `Content-Type: application/json` |
| `422` | `api.template_unknown` | 没有该名称的内置模板 |
| `422` | `file.json_invalid`、`file.too_large`、`doc.*` | 文档无法读取 |
| `422` | 任意验证代码、`run.override_unknown`、`profile.active_missing`、`run.limit_range`、`seed.range`、`secret.missing`、`transport.address_in_use`…… | 实验无法启动：它未通过验证、某个值超出范围、某个机密未存储、它监听的某个端口已被占用 |

运行一旦启动，无论发生什么，状态都是 `200`：请读取结果中的 `outcome`。

| `outcome` | 含义 |
| --- | --- |
| `passed` | 每个步骤都通过，且到达了 [[ui:exp.node.end]] |
| `failed` | 某个步骤失败，或运行超出其 `timeout`（`run.timeout`）；`error` 说明是哪一个以及为什么 |
| `stopped` | 它在结束前被停止：被 `job_stop`、[[ui:app.stopAll]] 或服务器关闭。`steps` 包含它走到的步骤；不保存报告 |

## 客户端离开 {#disconnect}

关闭连接不会停止运行。它是服务器上的一个任务：会运行到结束并保存报告，就像在浏览器中启动的运行在标签页关闭时那样。用 [`jobs_list`](commands.md#jobs_list) 找到它，用 [`job_stop`](commands.md#job_stop) 停止它，之后用 [`experiment_runs`](commands.md#experiment_runs) 读取它的报告。服务器关闭时，运行会被停止，仍保持连接的客户端会收到 `"outcome": "stopped"`。

## 示例 {#examples}

运行一个内置模板并等待结果：

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
curl -sS -X POST "$SERVER/api/run" \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"template":"osc-ping-reply","overrides":{"device":"192.0.2.20:9000"},"timeout":30}' \
  | jq -r .outcome
```

发送您自己的实验并更改一个参数，在每一步发生时打印它：

```bash
jq '{document: ., overrides: {api: "http://192.0.2.10:8080"}, profile: ""}' smoke.json |
  curl -sSN -X POST "$SERVER/api/run" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    -H "Accept: application/x-ndjson" --data @- |
  jq -r 'select(.type == "step") | "\(.node_id)  \(.state)  \(.detail)"'
```

`-N` 会阻止 `curl` 把各行积压起来。要让失败的运行使流水线失败，请检查 `outcome`：

```bash
outcome=$(curl -sS -X POST "$SERVER/api/run" -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" -d '{"template":"http-check"}' | jq -r .outcome)
[ "$outcome" = "passed" ]
```

## 从命令行 {#cli}

带 `--server <url>` 的 [`signallab run`](../automation/cli.md#cli-run) 通过此端点在服务器上运行：它发送所读取的实验，带上 `overrides`、`seed` 和 `timeout`，请求 NDJSON，并在每一行到达时打印该步骤。令牌来自 `--token-file`，否则来自 `SIGNALLAB_TOKEN`。带 `--report` 时，它通过 `/api/files` 下载报告。服务器 60 s 内没有发送任何内容——连心跳都没有——就视为已离开。

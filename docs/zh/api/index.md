---
title: HTTP API
description: 用服务器自己界面所用的同一批命令和事件，从脚本、CI 和其他工具驱动 Signal Lab 服务器。
---

# HTTP API

若想从脚本、CI 流水线或其他工具驱动 Signal Lab，请通过 HTTP 与 Signal Lab 服务器（`signal-lab-server`，或 Docker 镜像）通信。服务器在浏览器中显示的界面用的正是这套 API：每个按钮都是一次 `/api/invoke/<command>` 调用，每个实时数字都通过 `/api/events` 到达。因此，人在服务器页面上能做的任何事，脚本也能做。

桌面应用没有 HTTP API：它的窗口在应用内部访问其引擎。要在桌面上实现自动化，请在同一台机器上运行服务器（参见[服务器](../server/index.md)），或使用命令行 [`signallab`](../automation/cli.md)。

## 端点 {#endpoints}

| 方法与路径 | 作用 | 令牌 |
| --- | --- | --- |
| `GET /api/health` | 服务器是否应答、其版本、是否需要令牌 | 不需要 |
| `POST /api/invoke/<command>` | 运行一条引擎命令：输入 JSON 参数，输出 JSON 结果（[命令](commands.md)） | 需要 |
| `POST /api/run` | 将实验运行到结束：返回结果，或将其步骤作为逐行输出（[运行](run.md)） | 需要 |
| `GET /api/events` | 每个引擎事件的 WebSocket（[事件](events.md)） | 需要 |
| `GET /api/files?path=…` | 引擎写入数据文件夹中的文件，作为下载 | 需要 |
| `GET /api/openapi.json` | 用 OpenAPI 3.1 描述的本 API | 需要 |
| `GET /login`、`POST /login` | 供浏览器使用的登录页面和表单 | 不需要 |
| `POST /logout` | 结束一个浏览器会话 | 会话 |

`/api/` 下的任何其他路径都会应答 `404`，代码为 `api.not_found`；路径不接受的方法（`GET /api/invoke/…`）为 `405`，正文为空。其余的一切都是界面；在带令牌的服务器上，没有会话的浏览器会先被送到 `/login`。

## 基础 URL {#base-url}

除非用 `--listen`（或 `SIGNALLAB_LISTEN`）另行指定，否则服务器监听 `http://127.0.0.1:1430`。Docker 镜像监听所有网卡，即 `0.0.0.0:1430`。这些页面上的示例使用：

```bash
SERVER=http://127.0.0.1:1430
```

服务器使用明文 HTTP。若要 HTTPS，请在它前面放置一个终止 TLS 的反向代理，并用 `--secure-cookie` 启动服务器。

## 认证 {#authentication}

不带令牌启动的服务器只监听环回地址，且无需认证：那台机器上的任何人都可以使用它。其他人可访问的服务器总是带令牌，此时除 `/api/health` 和 `/login` 外的每个请求都必须携带它。

**脚本**在 `Authorization` 标头中发送令牌：

```bash
TOKEN=$(cat token.txt)
curl -fsS "$SERVER/api/invoke/jobs_list" -X POST \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json"
```

该标头必须恰好是 `Bearer`、一个空格和令牌。缺失或错误的令牌会得到 `401`，代码为 `auth.required`。

**浏览器**在 `/login` 用令牌登录一次，并得到一个会话 Cookie `signallab_session`：`HttpOnly`、`SameSite=Strict`，保留 7 天；服务器以 `--secure-cookie` 运行时为 `Secure`。`POST /logout` 会结束它。登录表单上错误的令牌会在应答前等待一秒，这让猜测变得缓慢。会话保存在服务器的内存中：重启会让每个浏览器退出登录，而带令牌的脚本不受影响。服务器最多保留 1024 个会话；超过后，最旧的先被清除。

令牌从何而来取决于服务器的设置（`--token-file`、`SIGNALLAB_TOKEN`，或 `--generate-token` 在数据文件夹中生成的 `token` 文件）：参见[服务器安全](../server/security.md)。令牌至少有 24 个字符且不含空格；`signal-lab-server token` 会输出一个新的。

::: warning
任何持有令牌的人都能让服务器发送流量。请确保存放它的文件只有您可读，也绝不要把它放进 URL——反正服务器不会从那里读取令牌。
:::

## 主机名与来源 {#host-origin}

有两项检查会最先运行，作用于每个路径，包括 `/api/health`。

**`Host`。** `Host` 标头必须指明这台服务器：

- 环回名称总是通过：`localhost`、以 `.localhost` 结尾的名称、`127.x.x.x` 和 `[::1]`。
- 用 `--allowed-host`（或 `SIGNALLAB_ALLOWED_HOSTS`）给出的名称通过。
- 带令牌且没有 `--allowed-host` 的服务器应答任何名称。

其他任何情况都会得到 `403`，代码为 `auth.host`。请用服务器接受的名称来寻址它：`curl` 会发送您给它的 URL 中的主机名。

**`Origin`。** 会更改内容的请求（除 `GET` 和 `HEAD` 之外的任何方法）以及 WebSocket 升级，在携带 `Origin` 标头时必须来自服务器自己的页面：其主机名和端口必须与 `Host` 相同。否则应答为 `403`，代码为 `auth.origin`；`Origin: null` 也会被拒绝。脚本和 `curl` 不发送 `Origin`，会通过此项检查——它们仍需要令牌。

**仅限 JSON。** `POST /api/invoke/…` 和 `POST /api/run` 接受 `Content-Type: application/json`（诸如 `; charset=utf-8` 的参数没有问题）。其他任何情况都会得到 `415`，代码为 `command.json_required`。其他站点上的网页若不先询问服务器，就无法发送它，而服务器从不答应。

## 调用命令 {#invoke}

```http
POST /api/invoke/<command>
Content-Type: application/json

{ "argument": "value", … }
```

- 正文是一个包含该命令参数的 JSON 对象。不带参数的命令接受 `{}` 或空正文（仍需要 `Content-Type` 标头）。
- 参数名为 camelCase，与界面发送的一致：`jobId`、`nodeId`。作为参数传入的对象（`config`、`request`、`document`、`library`……）保留引擎写入的字段名，它们大多是 snake_case：`timeout_ms`、`port_start`。
- 命令不认识的参数会报错，绝不会被忽略：`422`，代码为 `command.args_invalid`，其中指出该命令，解析器的原话在 `detail` 中。缺少必需参数时同样如此。不带参数的命令完全不读取正文。
- 可选参数可以省略，也可以作为 `null` 发送。
- 应答为 `200`，内容是命令结果的 JSON。没有内容可返回的命令应答 `null`。

每条命令及其参数和结果都在[命令](commands.md)中。

## 错误 {#errors}

| 状态 | 何时 | 正文 |
| --- | --- | --- |
| `200` | 命令已运行；`/api/run` 已启动运行 | 结果 |
| `400` | 正文不是 JSON；无法读取运行请求 | `EngineError`：`command.args_invalid`、`api.run_invalid`、`api.run_source` |
| `400` | `/api/files` 缺少 `path` | Web 服务器的纯文本，不是 `EngineError` |
| `401` | 没有令牌，或令牌错误 | `EngineError`：`auth.required` |
| `403` | 服务器拒绝的 `Host` 或 `Origin` | `EngineError`：`auth.host`、`auth.origin` |
| `404` | 没有这样的 API 路径；文件不在数据文件夹中 | `EngineError`：`api.not_found`、`file.not_found` |
| `405` | 路径不接受的方法 | 空 |
| `413` | 请求正文超过 24 MiB | Web 服务器的纯文本，不是 `EngineError` |
| `413` | 下载超过 256 MiB | `EngineError`：`file.too_large` |
| `415` | 不是 `application/json` | `EngineError`：`command.json_required` |
| `422` | 命令失败，或运行无法启动 | `EngineError`：引擎的任意代码 |
| `500` | 文件无法读取 | `EngineError`：`file.io` |

未知的命令名会得到 `422`，代码为 `command.unknown`。

引擎报告的每个失败都只有一种形态，即 `EngineError`：

```json
{
  "code": "transport.refused",
  "params": { "target": "http://127.0.0.1:8080/" },
  "node": "request",
  "field": { "key": "url" },
  "detail": "error sending request for url (http://127.0.0.1:8080/): tcp connect error: Connection refused (os error 111)"
}
```

| 字段 | 内容 |
| --- | --- |
| `code` | 出了什么问题：一个稳定的标识符。每个代码及其消息都列在[错误消息](../reference/errors.md)中，按点号前的部分分组（例如 [`transport`](../reference/errors.md#transport)） |
| `params` | 消息中提到的值，全部为字符串。没有时省略 |
| `node` | 它所涉及的实验节点。没有时省略 |
| `field` | 它所涉及的字段：`key`（在[字段](../reference/errors.md#fields)中命名）以及从 1 开始的 `index`，用于标头这类重复字段。没有时省略 |
| `detail` | 操作系统、解析器或库的自有措辞，为英文。没有时省略 |

请根据 `code` 分支，绝不要根据 `detail`。一次运行或单次发送所用的机密值，在其报告的每个错误中都会被遮蔽（`••••`）。

到达其服务器但得到错误状态、或根本没有得到应答的请求，不算失败的命令：`http_request` 会带着响应应答 `200`，由 `ok`、`error` 和 `cause` 说明发生了什么。参见 [`http_request`](commands.md#http_request)。

## 限制 {#limits}

| 项目 | 限制 | 达到限制时 |
| --- | --- | --- |
| 请求正文 | 24 MiB | `413` |
| 实验文档 | 4 MiB | `file.too_large` |
| 从 `/api/files` 下载 | 256 MiB | `413`、`file.too_large` |
| 运行时长 | 300 s | 运行以 `run.timeout` 失败 |
| 反馈表单（`feedback_send`） | 合计 15 MiB | `feedback.too_large` |
| 等待单个 WebSocket 的事件 | 4096 | 它会收到 [`server://lagged`](events.md#event-server-lagged)，以及错过了多少个 |

## 事件 {#events}

升级为 WebSocket 的 `GET /api/events` 会以文本消息的形式，把引擎发送的每个事件——运行的步骤、任务的结束、监视器的消息、检查器的帧——流式发送给每个已连接的客户端：

```json
{ "event": "job://ended", "payload": { "job_id": 7, "kind": "storm", "error": null } }
```

它遵循与其余部分相同的令牌和 `Origin` 规则。每个频道及其载荷都在[事件](events.md)中。

## 文件 {#files}

`GET /api/files?path=<path>` 会下载引擎写入服务器数据文件夹中的文件：一份运行报告（运行结果中的 `report_path`）、实验或检查器的导出、信号库或模拟器库。`path` 是引擎在服务器上给您的路径（URL 编码）：

```bash
curl -fsS -G "$SERVER/api/files" --data-urlencode "path=/data/runs/run-1759600000000-3.json" \
  -H "Authorization: Bearer $TOKEN" -o report.json
```

- 只提供数据文件夹内部的文件。其他任何情况，无论是文件夹还是不存在的文件，都会得到 `404`，代码为 `file.not_found`。
- 应答为 `application/octet-stream`，带有 `Content-Disposition: attachment`。文件名中除字母、数字、`.`、`_` 和 `-` 之外的字符都会变成 `_`。
- 超过 256 MiB 的文件会得到 `413`，代码为 `file.too_large`。

数据文件夹中包含的内容见[文件与文件夹](../reference/files.md)。

## 健康检查 {#health}

`GET /api/health` 是开放的：它不需要令牌，只需要服务器接受的 `Host`。

```bash
curl -fsS "$SERVER/api/health"
```

```json
{ "status": "ok", "version": "[[version]]", "auth": true }
```

`auth` 表示请求是否需要令牌。`signal-lab-server healthcheck` 会询问服务器监听的同一个地址，并在它应答时以 `0` 退出；Docker 镜像的健康检查会运行它。

## OpenAPI 描述 {#openapi}

`GET /api/openapi.json` 用 OpenAPI 3.1 描述本 API：端点、每条命令及其参数，以及运行请求和结果。它会像 `/api/` 的其余部分一样需要令牌。这些页面是完整参考；两者不一致时，以这些页面对引擎的表述为准。

## 任务 {#jobs}

长时间运行的工作——监视器、生成器、突发、中继、连接、模拟器、运行——都是一个任务。启动任务的命令会在运行时立即返回其 `JobInfo`：

```json
{ "id": 4, "kind": "osc-monitor", "label": "OSC monitor 0.0.0.0:9000", "params": { "bind": "0.0.0.0:9000" }, "started_ms": 1759600000000 }
```

| 字段 | 内容 |
| --- | --- |
| `id` | 任务的编号，在服务器运行期间唯一；其他命令以 `jobId` 或 `id` 接收它 |
| `kind` | `experiment`、`osc-monitor`、`osc-gen`、`http-burst`、`netsim`、`storm`、`scan`、`beacon`、`discovery`、`mqtt`、`websocket` 或 `emulator` |
| `label` | 一行英文，用于日志 |
| `params` | 组成该标签的值（target、bind、host……）。没有时省略 |
| `started_ms` | 启动的时间，自 1970 年起的毫秒数 |

这些命令会启动任务：`experiment_start`、`osc_monitor_start`、`osc_generator_start`、`http_burst_start`、`netsim_start`、`storm_start`、`scan_start`、`broadcast_beacon_start`、`discovery_start`、`mqtt_connect`、`ws_connect` 和 `emulator_start`。服务器会连同发出请求的客户端地址一起记录每次启动；`POST /api/run` 也会启动一个任务。

- [`jobs_list`](commands.md#jobs_list) 列出正在运行的任务，[`job_stop`](commands.md#job_stop) 停止其中一个，[`jobs_stop_all`](commands.md#jobs_stop_all) 停止每一个。
- 自行结束或失败的任务会发送 [`job://ended`](events.md#event-job-ended)。您停止的任务不再发送任何内容：`job_stop` 应答 `true` 就是确认。
- 任务属于服务器，而不属于启动它们的客户端。关闭页面或结束脚本不会停止它们，每个客户端都能看到并停止它们，而服务器关闭时会停止所有任务。

## 一个完整示例 {#example}

询问服务器是否在运行，用一条命令启动一个小的 HTTP 模拟器，对它运行内置的 `http-check` 实验并等待结果，然后停止该模拟器。`jq` 从应答中取出字段。

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)            # leave out with a loopback server without a token
AUTH="Authorization: Bearer $TOKEN"
JSON="Content-Type: application/json"

# 1. Up? Which version? Does it want a token?
curl -fsS "$SERVER/api/health"
# {"status":"ok","version":"[[version]]","auth":true}

# 2. One command: an HTTP emulator on 127.0.0.1:8080 that answers GET / with 200
EMULATOR=$(curl -fsS -X POST "$SERVER/api/invoke/emulator_start" -H "$AUTH" -H "$JSON" -d '{
  "emulator": { "name": "Example", "bind": "127.0.0.1:8080", "protocol": "http",
                "routes": [ { "method": "GET", "path": "/", "responses": [ { "body": "ok" } ] } ] }
}' | jq .id)

# 3. Run the bundled experiment that expects 200 from http://127.0.0.1:8080/, and wait
curl -sS -X POST "$SERVER/api/run" -H "$AUTH" -H "$JSON" -d '{"template":"http-check"}' \
  | jq '{outcome, error, report_path}'
# {"outcome":"passed","error":null,"report_path":"/data/runs/run-1759600000000-2.json"}

# 4. Stop the emulator
curl -fsS -X POST "$SERVER/api/invoke/job_stop" -H "$AUTH" -H "$JSON" -d "{\"id\":$EMULATOR}"
# true
```

`curl -f` 会把错误状态变成失败的命令；省略它（如第 3 步）即可在正文中看到 `EngineError`。

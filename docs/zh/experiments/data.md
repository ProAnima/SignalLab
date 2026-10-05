---
title: 实验中的数据
description: 参数与配置、带生成器的模板语言、从响应中提取的值、比较，以及永不离开引擎的机密。
---

# 实验中的数据

值在一次运行中流动：一个参数选择目标，一个响应的字段成为下一个请求的标头，一个生成的 id 随命令发出又在检查中返回。本页介绍这些值来自哪里，以及字段如何使用它们。

| 来源 | 写法 | 在哪里设置 |
| --- | --- | --- |
| 参数 | `{{api}}` 或 `{{params.api}}` | [[ui:exp.params]] 面板、配置、[[ui:exp.runWith]] |
| 变量 | `{{token}}` 或 `{{vars.token}}` | 运行中的某个节点：[[ui:exp.node.extract]]、等待节点、发送并等待回复的节点 |
| 机密 | `{{secret.API_TOKEN}}` | 计算机的凭据存储，或服务器的环境变量和文件 |
| 内置值 | `{{run.seed}}`、`{{now.iso}}`、`{{counter}}` | 运行本身 |
| 生成器 | `{{uuid}}`、`{{random_int(1, 100)}}` | 从本次运行的种子中抽取 |

## 参数 {#parameters}

参数是一个具名文本值，任何模板字段都可以使用它。把目标放进参数里，这样更改地址只需改一处，而不是每个节点各改一次。

### 添加参数 {#add-parameter}

1. 在编辑器工具栏中按 [[ui:exp.params]]（`{ }`）。
2. 在 [[ui:exp.noProfile]] 标签页上，按 [[ui:exp.addParam]]。
3. 输入 [[ui:exp.paramName]] 和 [[ui:exp.paramValue]]，例如
   `api` 和 `http://127.0.0.1:8080`。
4. 在某个节点的字段中，写出 `{{api}}/login`。

面板中的每次更改都是对实验的一次编辑：它会随实验一起保存，也能像其他编辑一样用 <kbd>Ctrl</kbd>+<kbd>Z</kbd> 撤销。

### 规则 {#parameter-rules}

| 规则 | 限制 |
| --- | --- |
| 名称 | 以字母或 `_` 开头，之后是字母、数字和 `_` |
| 保留名称 | `vars`、`params`、`secret`、`run`、`node`、`now`、`uuid`、`counter`、`random_int`、`random_float`、`pick` |
| 每个实验的参数 | 64 |
| 单个值的大小 | 64 KiB |
| 名称 | 唯一；变量不能与参数同名 |

值就是纯文本，按原样插入：值内部的 `{{…}}` 不会被解析。当某个字段要求参数的一部分
（`{{config.ports[0]}}`）时，该值会按 JSON 读取；不是 JSON 的值没有部分。

名称无效、为保留名或重复的参数不会阻止实验保存，因此您可以继续输入；在名称修正之前，实验不会运行。

## 配置 {#profiles}

配置是一组具名的参数值——*笔记本*、*舞台*、*场馆*——因此切换目标是一种选择，而不是逐个编辑节点。配置更改部分参数；其余参数保持默认值。

### 创建配置 {#make-profile}

1. 打开 [[ui:exp.params]] 并按 [[ui:exp.addProfile]]。会打开一个新标签页。
2. 在 [[ui:exp.profileName]] 中重命名它。
3. 对配置更改的每个参数，输入其值。留空的字段保持
   默认值，字段中会以灰色显示；[[ui:exp.resetToDefault]]（↺）会清除
   一个值。
4. 按 [[ui:exp.makeActive]] 用它来运行。使用中的配置标签页带有
   ● [[ui:exp.activeProfile]]。在 [[ui:exp.noProfile]] 标签页上按
   [[ui:exp.makeActive]] 会回到默认值。

实验有了配置后，工具栏中的 [[ui:exp.profile]] 列表可在它们之间切换。运行、预览和
[[ui:exp.sendNow]] 都使用使用中的配置，它保存在实验中，因此导出的文件打开后具有相同的目标。[[ui:exp.removeProfile]] 会删除屏幕上显示的配置。

| 规则 | 限制 |
| --- | --- |
| 每个实验的配置 | 32 |
| 名称 | 1–64 个字符，唯一（首尾空格不计） |
| 值 | 只能是已存在的参数；最多 64 个 |

重命名或删除参数会同时更改每个配置中的它。

### 一次运行使用哪个值 {#precedence}

后者生效：

1. 参数的默认值，在 [[ui:exp.noProfile]] 标签页上；
2. 使用中的配置的值（如果它设置了该值）；
3. 在 [[ui:exp.runWith]] 中仅为本次运行输入的值——
   参见[使用其他值运行](runs.md#run-with)。

[[ui:exp.runWith]] 只能设置实验已有的参数。运行报告
会记录所用的配置、为本次运行输入的值以及它使用的每一个值。

### 无法运行的配置 {#profile-issues}

每次检查实验时，其他配置和默认值也会被检查。某个会失败的配置——比如一个不是 `http://` 或
`https://` 的 URL——会在标签页和工具栏列表中带有 ⚠，其工具提示会说明原因。它不会阻止使用其他配置的运行。

## 模板 {#templates}

`{{ }}` 内的文本是一个表达式；字段中的其他一切都会按原样保留。

```text
{{api}}/users/{{user.id}}?trace={{uuid}}
Bearer {{secret.API_TOKEN}}
```

- 花括号内的空格无关紧要：`{{ token }}` 就是 `{{token}}`。
- `\{{` 写出一个字面量 `{{`。
- 单独的 `}}` 是纯文本。
- 值按原样插入，不带引号。在 JSON 正文中，引号请自行书写：`"id": "{{uuid}}"`。

### 名称 {#names}

| 表达式 | 值 |
| --- | --- |
| `{{name}}` | 如果这条路径上设置了变量 `name`，则为该变量，否则为参数 `name` |
| `{{vars.name}}` | 仅变量 |
| `{{params.name}}` | 仅参数 |
| `{{secret.NAME}}` | 已存储的机密 `NAME`——参见[机密](#secrets) |
| `{{name.field}}` | JSON 值的一个字段 |
| `{{name[0]}}` | JSON 数组的一个元素 |
| `{{name["a b"]}}`、`{{name['a b']}}` | 名称含其他字符的字段 |

`.` 之后的字段名可以包含字母、数字、`_` 和 `-`。可以连续使用：
`{{reply.args[0]}}`、`{{order.items[2].sku}}`。

### 值的写法 {#value-text}

| 值 | 写法 |
| --- | --- |
| 文本 | 文本本身 |
| 数字 | 其最短形式：`42`、`0.5` |
| `true`、`false` | `true`、`false` |
| `null` | `null` |
| 对象、数组 | 紧凑 JSON：`["x","y"]` |

### 内置值 {#built-ins}

| 表达式 | 值 |
| --- | --- |
| `{{run.id}}` | 本次运行的任务编号；在预览和 [[ui:exp.sendNow]] 中为 `0` |
| `{{run.seed}}` | 本次运行的种子 |
| `{{node.id}}` | 正在执行的节点的 id |
| `{{now}}` | 当前时间，Unix 毫秒 |
| `{{now.iso}}` | 当前 UTC 时间，ISO 8601，带毫秒：`2026-09-30T12:34:56.789Z` |
| `{{counter}}` | 此节点在本次运行中运行了多少次，包括本次，从 1 开始 |

`{{counter}}` 按节点计数：在[循环](flow.md#loop)的循环体中它是
迭代的编号，在[重复](flow.md#repeat)的节点中它是发送的编号。
`run`、`node` 和 `now` 只有列出的字段；其他任何内容都是错误。

### 生成器 {#generators}

| 表达式 | 值 |
| --- | --- |
| `{{uuid}}` 或 `{{uuid()}}` | 一个版本 4 UUID |
| `{{random_int(min, max)}}` | 从 `min` 到 `max` 的整数，两端都包含；参数为整数，`min` ≤ `max` |
| `{{random_float(min, max)}}` | 从 `min` 到（但不包括）`max` 的数，带 3 位小数；`min` < `max` |
| `{{random_float(min, max, digits)}}` | 同上，带 `digits` 位小数，0–9 |
| `{{pick(a, b, c)}}` | 参数之一，至少一个 |

参数用逗号分隔。带引号的参数（`"dark blue"` 或 `'a, b'`）
可以包含除自身引号外的任何内容；不带引号的参数可以包含字母、
数字和 `_ - . : / +`。空参数是错误。

每个生成器都从本次运行的种子中抽取。一个节点的一次执行所得到的值
只取决于种子、节点的 id 以及该节点已运行了多少次，因此并行分支绝不会
改变彼此的值，相同的种子会再次生成相同的值。一个节点内的抽取
遵循其字段的顺序。`{{now}}` 和 `{{run.id}}` 不可重现。参见
[种子](runs.md#seeds)。

### 建议 {#suggestions}

在模板字段中输入 `{{`，或按 <kbd>Ctrl</kbd>+<kbd>Space</kbd>，
会打开一个分四组的列表：[[ui:exp.suggest.params]]（带其值）、
[[ui:exp.suggest.vars]]（在此节点上游设置的，带设置它们的节点
（回复的字段也是，例如 `reply.args[0]`））、[[ui:exp.suggest.secrets]] 和
[[ui:exp.suggest.generators]]。<kbd>↑</kbd> 和 <kbd>↓</kbd> 选择，
<kbd>Enter</kbd> 或 <kbd>Tab</kbd> 插入，<kbd>Esc</kbd> 关闭列表并
保留字段内容。

### 未知名称是错误 {#unknown-names}

没有值的名称绝不会变成空字符串。运行之前，字段使用的每个
名称都必须是参数、有效的机密名称，或者在通往该节点的**每一条**路径上都设置了变量。编辑器会指出节点和
字段：

| 问题 | 运行之前 | 运行期间 |
| --- | --- | --- |
| 没有人设置的名称 | `name.unknown` | — |
| 只在一部分路径上设置的变量 | `name.not_on_every_path` | — |
| `{{params.x}}` 却没有参数 `x` | `param.unknown` | — |
| 值不具有的字段 | — | `template.no_field` |
| 未闭合的 `{{`、空的 `{{}}`、格式错误的参数 | `template.*`，带位置 | — |

这些代码的文本见[错误](../reference/errors.md)。

## 哪些字段接受模板 {#templated-fields}

| 节点 | 模板字段 |
| --- | --- |
| [[ui:exp.node.http]] | URL、标头名称和值、正文、Basic 和 Digest 的用户名和密码、Bearer 令牌 |
| [[ui:exp.node.osc]] | 目标、地址、文本参数；带回复时：其地址模式和规则值 |
| [[ui:exp.node.udp]] | 目标、载荷；带回复时：其模式 |
| [[ui:exp.node.tcp]] | 主机、载荷 |
| [[ui:exp.node.mqtt]] | 代理主机、主题、载荷 |
| [[ui:exp.node.log]] | 消息 |
| [[ui:exp.node.assert_body]] | 预期文本 |
| [[ui:exp.node.assert_header]] | 标头名称、预期文本 |
| [[ui:exp.node.assert_value]]、[[ui:exp.node.branch_value]]、[[ui:exp.node.loop]] 的退出条件 | 值、预期值 |
| [[ui:exp.node.wait_osc]] | 地址模式、规则值 |
| [[ui:exp.node.wait_udp]]、[[ui:exp.node.wait_ws]] | 模式 |
| [[ui:exp.node.wait_mqtt]] | 代理和主题（仅参数）、模式 |
| [[ui:exp.node.wait_http]] | 路径模式、条件 |
| [[ui:exp.node.impairment]] | 监听和目标（仅参数） |
| [[ui:exp.node.ws_connect]] | URL、标头名称和值 |
| [[ui:exp.node.ws_send]] | 载荷 |
| [[ui:exp.node.ws_close]] | 原因 |

数字——端口、超时、延时、状态码、带类型的 OSC 数字——以及等待节点的
监听地址都是字面量。[[ui:exp.node.emulator]] 用收到的内容
（`{{request.…}}`）和参数来渲染自己的回复；参见
[故障](faults.md#emulator)。

**仅参数。** 有些字段在第一个步骤之前就已打开，此时还没有
变量存在：[[ui:exp.node.wait_mqtt]] 的代理和主题、一个
[[ui:exp.node.impairment]] 的监听和目标。它们只接受文本和参数，
没有别的（`node.params_only`）。

**像字面量一样检查。** 只使用参数的字段会在运行前解析，并按
本次运行将发送的文本进行检查：URL 必须是
`http://` 或 `https://`，OSC 目标为 `IP:port` 或 `host:port`，标头名称有效。带
变量或生成器的字段在运行时检查。

## 预览 {#preview}

当选中的节点带有模板时，其属性会显示它在已知当前值的情况下将会做什么：发送为 [[ui:exp.preview]]，
等待为 [[ui:exp.previewWait]]，比较为 [[ui:exp.previewCheck]]。
引擎会解析它，使用的代码与运行完全相同，因此预览绝不会与运行不一致。

- 参数来自使用中的配置。
- 变量来自编辑器在此会话中见过的内容：上次运行的步骤，
  以及 [[ui:exp.sendNow]]。
- 已存储的机密显示为 `••••`。
- 尚无值的名称按原样保留，预览会列出它。未存储的
  机密会单独列出。
- 生成器使用实验固定的种子，未固定时使用 `0`，作为
  节点的首次执行。固定了种子时，预览会显示节点在运行中首次执行
  将发送的生成值。

## 提取值 {#extract}

[[ui:exp.node.extract]] 读取其路径上最近一次 HTTP 响应中的一个值，
并把它写入变量。

| 字段 | 内容 |
| --- | --- |
| [[ui:exp.variable]] | 要写入的变量；适用参数的命名规则 |
| [[ui:exp.extractFrom]] | 值来自哪里（见下文） |
| [[ui:exp.jsonPath]]、[[ui:exp.headerName]] 或 [[ui:exp.pattern]] | 取决于来源，要读取的内容 |

| [[ui:exp.extractFrom]] | 读取 | 值 |
| --- | --- | --- |
| [[ui:exp.from.json]] | 按路径读取作为 JSON 的正文 | JSON 值：文本、数字、对象、数组 |
| [[ui:exp.from.header]] | 该名称的第一个标头，不区分大小写 | 文本 |
| [[ui:exp.from.status]] | 状态码 | 数字 |
| [[ui:exp.from.body]] | 整个正文 | 文本 |
| [[ui:exp.from.regex]] | 正文中的第一个匹配 | 如果模式有分组，则为第 1 个捕获组，否则为整个匹配 |

**JSON 路径。** `$.token`、`$.items[0].id`、`$["a b"]`、`$['a b']['c-d']`；
开头的 `$.` 可以省略（`token`、`items[0].id`），单独的 `$` 表示
整个正文。

**正则表达式**使用 Rust `regex` 引擎的语法，它没有
环视，也没有反向引用。匹配会在正文中任意位置搜索；需要时
用 `^` 和 `$` 锚定。

在下列情况下，步骤会失败并指出缺少什么：

- 在这条路径上它之前没有 HTTP 请求运行过（`check.no_response`；编辑器
  在无法成立的图中已经拒绝，`graph.needs_http`）；
- 正文不是 JSON，或者路径不在其中；
- 标头不存在，或者模式不匹配；
- 对于 JSON 路径或整个正文，正文超过响应保留的 256 KiB；
  或者对于模式，在保留的部分中没有任何匹配
  （`extract.truncated`）。

时间线会显示写入的值：`token = abc123`。

::: tip 点击即可提取
在 [[ui:exp.node.http]] 上使用 [[ui:exp.sendNow]] 会显示其 JSON 响应。点击其中的一个
值：请求之后会添加一个 [[ui:exp.node.extract]] 节点，路径已填好，名称取自该键，该值会立即
为预览所知。
:::

## 变量 {#variables}

变量保存一个 JSON 值。以下节点会写入变量：

| 节点 | 写入 | 在哪个输出上 |
| --- | --- | --- |
| [[ui:exp.node.extract]] | 提取的值 | 其输出 |
| [[ui:exp.node.wait_osc]]、[[ui:exp.node.wait_udp]]、[[ui:exp.node.wait_mqtt]]、[[ui:exp.node.wait_http]]、[[ui:exp.node.wait_ws]] | 到达的内容，默认名 `reply`（HTTP 为 `request`） | 仅 [[ui:exp.portMatched]] |
| [[ui:exp.node.osc]]、[[ui:exp.node.udp]] 带 [[ui:exp.expectReply]] | 回复，默认名 `reply` | 其输出 |

等待节点写入的是一个对象；之后的字段读取它的部分：

| 等待 | 字段 |
| --- | --- |
| OSC | `address`、`args`、`from`、`ms` |
| UDP | `text`、`hex`、`bytes`、`from`、`ms`，以及带模式时的 `match` |
| MQTT | `topic`，以及 UDP 的字段 |
| WebSocket | UDP 的字段，消息为 JSON 时还有 `json` |
| HTTP 请求 | `method`、`path`、`query`、`headers`、`body`、`json`、`params`、`from`、`ms` |

`ms` 是从分支上最近一次动作到到达的时间。确切内容见[节点参考](nodes.md)。

### 变量在哪里可见 {#visibility}

变量从写入它的那个输出起存在，在经过该输出的路径上存在：

- 在可选路径合并之后——分支的 [[ui:exp.yes]] 和 [[ui:exp.no]]
  再次汇合——只有**每一条**路径都设置的值才是已知的。
- 在 [[ui:exp.node.join]] 之后，任何汇入它的分支**任意一个**
  设置的值都是已知的：它们全都运行过。
- 在 [[ui:exp.node.loop]] 的 [[ui:exp.portDone]] 或 [[ui:exp.portLimit]]
  之后，以及在其退出条件中，循环体的每次迭代都设置的值是已知的。
- 等待节点的变量在其 [[ui:exp.portTimeout]] 输出之后不再已知。

每个并行分支都在自己的变量副本上工作。Join 按传入接线的顺序合并这些
副本，两条都设置了同名值时后面的接线胜出，因此结果绝不取决于
哪个分支先完成。参见
[运行如何流动](flow.md#parallel)。

## 比较值 {#compare}

[[ui:exp.node.assert_value]] 在比较不成立时使运行失败；
[[ui:exp.node.branch_value]] 从 [[ui:exp.yes]] 或 [[ui:exp.no]] 离开；
一个 [[ui:exp.node.loop]] 使用同样的比较作为其退出条件。每个都有
[[ui:exp.value]]、[[ui:exp.operator]] 和 [[ui:exp.expected]] 值，两段
文本都是模板：

| [[ui:exp.value]] | [[ui:exp.operator]] | [[ui:exp.expected]] |
| --- | --- | --- |
| `{{status}}` | [[ui:exp.op.lt]] | `300` |
| `{{reply.args[0]}}` | [[ui:exp.op.eq]] | `{{nonce}}` |

| [[ui:exp.operator]] | 成立条件 |
| --- | --- |
| [[ui:exp.op.eq]]、[[ui:exp.op.ne]] | 两者相等（不相等）——都为数字时按数字比较（`200` 等于 `200.0`），否则按精确文本比较，区分大小写 |
| [[ui:exp.op.lt]]、[[ui:exp.op.le]]、[[ui:exp.op.gt]]、[[ui:exp.op.ge]] | 按数字比较；某一侧不是数字时步骤失败（`compare.not_numbers`），而不是安静地给出*否* |
| [[ui:exp.op.contains]] | 值包含预期文本，区分大小写 |
| [[ui:exp.op.matches]] | 预期值中的正则表达式在值中任意位置匹配 |
| [[ui:exp.op.empty]]、[[ui:exp.op.not_empty]] | 去除空格后值为空，或不为空；不使用预期值 |

数字是去除空格后能读作数字的文本：`42`、`-1.5`、
`1e3`。时间线会显示实际进行的比较，`401 = 200`，每一侧截断
到 120 个字符。

## 机密 {#secrets}

令牌或密码以 `{{secret.NAME}}` 的形式放入字段。实验
文件只保留名称；值留在它被存储的地方，绝不
进入界面。

### 机密存放在哪里 {#secret-stores}

| Signal Lab 运行在哪里 | 存储 | 从界面 |
| --- | --- | --- |
| Windows 桌面应用 | Windows 凭据管理器，服务名 `SignalLab` 下，每个名称一条 | 设置、替换、移除 |
| Linux 桌面应用 | 无：需要机密的运行会以 `secret.unsupported` 失败 | — |
| 服务器 | 环境变量 `SIGNALLAB_SECRET_<NAME>`，否则是其机密文件夹中的文件 `<NAME>`，默认 `/run/secrets/signallab`，除非另有设置 | 只读 |
| 命令行 `signallab` | 像服务器一样，或者用 `--secrets system` 使用 Windows 凭据管理器 | — |

[[ui:exp.secrets]] 部分的标题带有一个工具提示，说明您所在的位置适用于其中哪一种：
Windows 存储、服务器的环境变量和
文件，或者——在 Linux 桌面应用中——没有存储。在那里，
`secret.unsupported` 说明机密保存在 Windows 凭据
管理器中，而该系统没有它。

机密属于计算机或服务器，而不属于某一个实验：使用
`{{secret.API_TOKEN}}` 的两个实验使用同一个值。

在服务器上，环境变量优先于文件。文件末尾的
换行不算值的一部分，空文件视为无机密。服务器的
文件夹用 `--secrets-dir` 或 `SIGNALLAB_SECRETS_DIR` 设置；
参见[服务器](../server/index.md)。命令行见
[`signallab run`](../automation/cli.md#cli-run)。

| 规则 | 限制 |
| --- | --- |
| 名称 | 以字母或 `_` 开头，之后是字母、数字和 `_`；最多 128 个字符 |
| 值 | 非空，最多 16 KiB |

### 设置机密 {#set-secret}

在 Windows 上：

1. 打开 [[ui:exp.params]]。[[ui:exp.secrets]] 部分列出实验的字段
   使用的每一个机密，每个都标有 [[ui:exp.secretStored]] 或
   [[ui:exp.secretMissing]]。
2. 按名称旁边的 [[ui:exp.secretSet]]，或对尚无字段使用的名称按
   [[ui:exp.addSecret]]。
3. 输入值——字段显示为圆点——然后按 [[ui:exp.secretSave]] 或
   <kbd>Enter</kbd>。字段会被清空；任何东西都无法再读回该值。

[[ui:exp.secretReplace]] 存储一个新值，[[ui:exp.secretRemove]]
从凭据存储中删除它。在此会话中存储过的名称也会出现在建议中。

在连接到服务器的浏览器中，该部分只会显示
[[ui:exp.secretOnServer]] 或 [[ui:exp.secretNotOnServer]]：在服务器运行的地方设置值，有两种方式：

```bash
# in the server's environment
SIGNALLAB_SECRET_API_TOKEN='…'
# or as a file in its secrets folder
printf '%s' '…' > /run/secrets/signallab/API_TOKEN
```

文件每次运行开始时都会读取，因此更改过的文件从下一次运行起生效；更改过的环境变量需要重启服务器。

### 运行之前 {#secret-check}

本次运行的字段使用的每一个机密都必须已存储。缺少的机密会在任何流量之前、
在第一个使用它的节点和字段处停止运行
（`secret.missing`）。[[ui:exp.sendNow]] 对其节点做同样的检查。

### 遮蔽 {#masking}

当一次运行或一次 [[ui:exp.sendNow]] 使用机密时，它们值的每次出现
都会在离开引擎的一切内容中被替换为 `••••`：

- 步骤文本、错误，以及步骤写入的变量；
- 运行报告；
- [[ui:exp.sendNow]] 的结果，包括它显示的 HTTP 响应；
- [[ui:dock.inspector]] 的帧，在运行持续期间捕获的——在十六进制
  转储中，值的每个字节变成 `*`，因此偏移量保持正确。

Basic 认证以 base64 发送 `name:password`；当其中任一部分包含
机密时，那段 base64 文本也会被遮蔽。流量本身携带真实
值。预览将已存储的机密显示为 `••••`。一个
[[ui:exp.node.emulator]] 的回复不能使用机密。

## 命令 {#commands}

预览是 [`experiment_resolve`](../api/commands.md#experiment_resolve)；
机密用
[`secret_status`](../api/commands.md#secret_status)、
[`secret_set`](../api/commands.md#secret_set) 和
[`secret_delete`](../api/commands.md#secret_delete) 列出、设置和移除。没有任何命令会返回
机密的值。

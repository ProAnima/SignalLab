---
title: 信号
description: 把 OSC、UDP、HTTP 和 MQTT 消息保存在带文件夹的库中，并从信号界面、任意界面（Ctrl+K）或命令行再次发送。
---

# 信号

信号是您命名并保存下来的一条消息：一条 OSC 消息、一个原始 UDP 数据报、一个 HTTP 请求或一次 MQTT 发布，连同它的目标。您只需构建一次——在 [[ui:nav.signals]] 界面上，或从另一个界面把刚发送的内容保存下来——之后随时可以再次发送，字节完全一致。

库是一个 JSON 文件，您可以阅读、手动编辑、复制到另一台机器，或提交到项目旁边。[[ui:nav.signals]] 界面将其显示为左侧的文件夹树，右侧是所选信号的字段。

## 信号可以发送什么 {#transports}

在 [[ui:sig.transport]] 中选择类型。每种类型都有自己的字段：

| [[ui:sig.transport]] | 字段 | 发出的内容 |
| --- | --- | --- |
| [[ui:sig.tr.osc]] | [[ui:common.target]]、[[ui:common.address]]、[[ui:common.arguments]] | 向一个 `IP:port` 或 `host:port` 发送一条 OSC 消息，从一个新的 UDP 端口发出。参数类型见 [OSC](../protocols/osc.md#types)。 |
| [[ui:sig.tr.udp]] | [[ui:common.target]]、[[ui:sig.payloadKind]]（[[ui:sig.payloadText]] 或 [[ui:sig.payloadHex]]）、[[ui:sig.payload]] | 一个数据报，内容正是这些字节。文本按所写发出，不带终止符；十六进制是成对的数字，例如 `de ad be ef`（允许空格和 `0x` 前缀）。 |
| [[ui:sig.tr.http]] | [[ui:sig.method]]、[[ui:sig.url]]、[[ui:sig.timeout]]、[[ui:sig.headers]]、[[ui:field.auth]]、[[ui:sig.body]] | 一个 HTTP 请求。见 [HTTP](../protocols/http.md)。 |
| [[ui:sig.tr.mqtt]] | [[ui:mq.broker]]、[[ui:mq.topic]]、[[ui:mq.qos]]、[[ui:mq.retain]]、[[ui:sig.payload]] | 一次发布。见 [MQTT](../protocols/mqtt.md)。 |

每个信号还有一个 [[ui:sig.name]]、一个 [[ui:sig.group]] 和一个 [[ui:sig.note]]——它应当促成什么，以及远端必须与之匹配什么。

OSC 和 UDP 信号的目标是 `IP:port` 或 `host:port`（例如 `127.0.0.1:9000`）；每次发送信号时都会解析主机名。MQTT 信号的代理是 `host:port`；不带端口时为 `1883`。

更改信号类型时，其消息会从该类型的默认值重新开始。只有目标会被保留，且仅在 [[ui:sig.tr.osc]] 与 [[ui:sig.tr.udp]] 之间保留，因为在这两者中目标的含义相同。

## 发送信号 {#send}

要从 [[ui:nav.signals]] 界面发送信号，可执行以下任一操作：

- 选中它并按 [[ui:sig.fire]]。
- 在其字段中操作时按 <kbd>Ctrl</kbd>+<kbd>Enter</kbd>。
- 在树中双击它。

每次发送都会向控制台写入一行：什么发往了哪里、发送的字节数，对于 HTTP 则是状态和所用的时间。失败（连接被拒绝、主机无法访问）会显示为带原因的红色行。上次发送的时间显示在按钮旁边。

信号通过与对应协议界面相同的命令发出，因此[检查器](inspector.md)会把它列在发送它的工具下，远端也无法把它与您手动输入的一条区分开来。

各种类型的发送方式：

| 类型 | 如何发出 |
| --- | --- |
| [[ui:sig.tr.osc]] | 与 [[ui:nav.osc]] 界面发送消息的方式相同。 |
| [[ui:sig.tr.udp]] | 向目标发送一个数据报。 |
| [[ui:sig.tr.http]] | 与 [[ui:nav.http]] 界面发送请求的方式相同，在 [[ui:http.keepCookies]] 开启时使用其 cookie 罐。连接被拒绝或超时算作失败，而不是状态。 |
| [[ui:sig.tr.mqtt]] | 当 [[ui:nav.mqtt]] 界面已连接到该信号的代理时，在该连接上发送，使用其客户端 ID 和凭据。否则——未连接，或连接到了另一个代理——Signal Lab 为这一次发布连接到该信号的代理，使用自己的客户端 ID、无用户名和干净会话，然后断开连接。 |

当主机相同（不区分大小写）且端口相同时，连接就属于该信号的代理，不带端口书写的代理以 `1883` 为准。此处不解析名称：`localhost` 和 `127.0.0.1` 是两个不同的代理，因此指向其中一个的信号不会通过连接到另一个的链路发送。

::: tip
MQTT 信号不存储密码。要发布到需要密码的代理，请先在 [[ui:nav.mqtt]] 界面上连接到该代理；信号随后会使用该连接。
:::

## 从任意界面发送 {#palette}

在任意界面按 <kbd>Ctrl</kbd>+<kbd>K</kbd> 打开快捷面板，输入信号名称、文件夹、目标或消息的几个字母，然后按 <kbd>Enter</kbd>。面板关闭，信号发出；您仍停留在原来观看的界面上。

| 按键 | 作用 |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>K</kbd> | 打开或关闭快捷面板。 |
| <kbd>↑</kbd> <kbd>↓</kbd> | 移动选择。 |
| <kbd>Enter</kbd> | 发送所选的信号。 |
| <kbd>Esc</kbd> | 关闭快捷面板而不发送。 |

快捷面板最多列出 12 个信号：未输入任何内容时是库中的前 12 个，然后是与输入匹配的前 12 个。点击某一行即发送它；点击面板外则关闭面板。

## 创建信号 {#create}

### 在信号界面上 {#create-here}

1. 选择信号所属的文件夹（见[当前文件夹](#current-folder)）。
2. 按 [[ui:sig.new]]。一个发往 `127.0.0.1:9000`、地址为 `/hello` 的新 OSC 信号会出现在该文件夹中并被选中。
3. 修改 [[ui:sig.name]]、[[ui:sig.transport]] 以及消息的字段。

每次更改都会自动保存；此界面上没有保存按钮。当库文件无法读取时，不会保存任何内容，字段为只读（见[库文件](#file)）。

[[ui:sig.duplicate]] 会在所选信号之后放置一个副本，其名称后跟 `·`。[[ui:sig.delete]] 会再确认一次（[[ui:sig.confirmDelete]]）：第二次点击会将其从文件中移除。它的文件夹会保留，即使现在已为空。

### 从 HTTP、OSC 和 MQTT 界面 {#save-from-screens}

三个界面的发送部分——[[ui:nav.http]] 上的 [[ui:http.request]]、[[ui:nav.osc]] 上的 [[ui:osc.sender]]、[[ui:nav.mqtt]] 上的 [[ui:mq.publish]]——可以把将要发送的内容保存为信号。

1. 设置好消息并发送，直到它达到您想要的效果。
2. 按 [[ui:sig.saveNew]]（或在界面的该部分按 <kbd>Ctrl</kbd>+<kbd>S</kbd>）。[[ui:sig.saveTitle]] 对话框会打开。
3. 检查 [[ui:sig.name]]：它会根据正在发送的内容给出建议。
4. 选择或输入一个 [[ui:sig.group]]。上次使用的文件夹会被填入；尚不存在的路径（如 `Venue/Stage`）会被创建。
5. 按 [[ui:sig.saveConfirm]]。

此后该界面就与该信号关联。按钮旁边的标签会显示它的位置（`❖ Folder / Name`）；点击它即可在 [[ui:nav.signals]] 界面上看到该信号。

| 您看到 | 含义 | 您可以做什么 |
| --- | --- | --- |
| ✓ [[ui:sig.savedState]]（变灰） | 库中保存的内容与界面将要发送的内容完全一致。 | 无需保存。 |
| [[ui:sig.save]]，以及标签上的 [[ui:sig.changed]] | 界面的消息与该信号不同。 | [[ui:sig.save]] 或 <kbd>Ctrl</kbd>+<kbd>S</kbd> 会把界面的消息写入该信号；其名称、文件夹和备注保持不变。 |
| [[ui:sig.saveAs]] | — | 再次打开对话框，填入该信号的名称和文件夹，并保存为一个新信号。界面随后与新信号关联。 |

比较针对的是消息本身，而不是它的书写方式：JSON 键的顺序，以及 OSC 浮点数超出 32 位精度的末尾数字，都不算作更改。

Signal Lab 重启后，[[ui:nav.http]] 和 [[ui:nav.osc]] 界面会保留这种关联；[[ui:nav.mqtt]] 界面的关联会保留到您关闭应用为止。

::: warning
HTTP 信号会将其 [[ui:field.auth]]——用户名和密码，或令牌——以明文保存在库文件中。任何能读取该文件的人都能读到它们。
:::

### 在其界面中打开信号 {#open-in-screen}

选中的 HTTP、OSC 或 MQTT 信号有一个按钮，可在其协议界面（[[ui:nav.http]]、[[ui:nav.osc]] 或 [[ui:nav.mqtt]]）中打开它。界面的字段会从该信号填入，界面也与它关联，如上所述：在那里编辑、发送，然后 [[ui:sig.save]]。原始 UDP 信号没有自己的界面。

### 从一帧或一个主题 {#capture}

- 在[检查器](inspector.md#save-as-signal)中，选择一帧并按 [[ui:sig.fromFrame]]。数据报会变成一个带有该帧确切字节的原始 UDP 信号；MQTT 发布会变成一个具有相同代理、主题、QoS、保留标志和载荷的 MQTT 信号。其他帧无法保存。
- 在 [[ui:nav.mqtt]] 界面上，选择一个主题并按 [[ui:sig.fromFrame]]。您会得到一个 MQTT 信号，它把该主题的最后一个值，连同其 QoS 和保留标志，发布到您所连接的代理。

两者都会放入文件夹 [[ui:sig.capturedFolder]]，并以所捕获的内容命名。

### 在实验中 {#in-experiments}

向实验添加节点时，[[ui:exp.addNode]] 菜单也会在 [[ui:exp.group.signals]] 下列出您的信号。选中一个会添加一个带有相同消息的 OSC、HTTP、MQTT 或 UDP 节点。带有十六进制载荷的原始 UDP 信号不会被提供：UDP 节点发送的是文本。参见[节点](../experiments/nodes.md)。

## 文件夹 {#folders}

文件夹是由 `/` 连接的名称路径：`API/Auth` 是 `API` 内的 `Auth` 文件夹。信号的 [[ui:sig.group]] 字段保存其文件夹的路径；为空表示顶层（[[ui:sig.topLevel]]）。当您离开该字段时，名称会被去除首尾空白，空的部分会被丢弃，因此 ` API / Auth/ ` 会变成 `API/Auth`。

文件夹按名称排序，数字按数值顺序（`Cue 2` 在 `Cue 10` 之前）；信号保持文件中的顺序。每个文件夹显示它包含多少个信号，包括其子文件夹。空文件夹会一直保留，直到您将其移除。

### 当前文件夹 {#current-folder}

您最后点击的文件夹，或您所选信号所在的文件夹，就是当前文件夹：[[ui:sig.new]] 和 [[ui:sig.newFolder]] 会把内容放在那里。树上方的一个标签会标出它的名称；点击该标签可返回顶层。

### 处理文件夹 {#folder-tasks}

| 目的 | 操作 |
| --- | --- |
| 新建文件夹 | 按 ＋ [[ui:sig.newFolder]]。它会在当前文件夹内创建，命名为 [[ui:sig.newFolderName]]（当该名称已被占用时会在后面加一个数字），随后您可以立即重命名它。 |
| 打开或关闭文件夹 | 点击它，或在它获得焦点时按 <kbd>→</kbd> / <kbd>←</kbd>。Signal Lab 会记住哪些文件夹是关闭的。 |
| 全部打开或关闭 | 树上方的 ⊞ 和 ⊟ 按钮（[[ui:sig.expandAll]]、[[ui:sig.collapseAll]]）。 |
| 重命名文件夹 | 按 ✎（[[ui:sig.renameFolder]]）或在其上按 <kbd>F2</kbd>，输入，然后按 <kbd>Enter</kbd>；<kbd>Esc</kbd> 取消。 |
| 移动信号或文件夹 | 将其拖到某个文件夹上，或拖到树中的空白处以移动到顶层。 |
| 通过输入移动信号 | 更改其 [[ui:sig.group]] 字段。 |
| 移除文件夹 | 按 ×（[[ui:sig.removeFolder]]），然后按 [[ui:sig.confirmRemoveFolder]]，或在其上按两次 <kbd>Delete</kbd>。 |

重命名绝不会合并两个文件夹：名称中含有 `/`，或同级文件夹已拥有该名称，都会被拒绝，控制台会说明这一点。文件夹不能拖入自身或它内部的文件夹。把一个文件夹拖入已包含同名文件夹的文件夹，会合并两者。

移除文件夹只移除该文件夹：其中的信号和子文件夹会上移一级。不会删除任何内容。

### 查找信号 {#filter}

在树上方的 [[ui:sig.search]] 中输入。它会匹配名称、文件夹、备注、目标和消息。筛选期间，每个包含匹配项的文件夹都会展开，其他文件夹则隐藏。

## 初始集合 {#starter-set}

Signal Lab 第一次找不到库文件时，会写入九个示例，每一个都涉及容易出错的地方。它们的名称和备注以当时的界面语言写成；之后它们就归您修改。它们全部指向本机。

| 文件夹 | 信号 | 发送内容 |
| --- | --- | --- |
| `OSC` | [[ui:seed.osc-fader.name]] | `/fader/1`，浮点数 `0.75`，发往 `127.0.0.1:9000` |
| `OSC` | [[ui:seed.osc-types.name]] | `/types`，int `-7`、float `1.5`、string `hi`、bool true、int64 `4294967296`、double `0.125` 和 nil |
| `OSC` | [[ui:seed.osc-id-and-value.name]] | `/tag`，字符串 `reader-1` 和 `04a1b2c3` |
| `OSC` | [[ui:seed.osc-trigger.name]] | `/cue/go`，无参数 |
| `MQTT` | [[ui:seed.mqtt-publish.name]] | `1` 发往 `127.0.0.1:1883` 上的 `lab/example/value`，QoS 0 |
| `MQTT` | [[ui:seed.mqtt-retained.name]] | `night` 发往 `lab/example/config`，QoS 1，保留 |
| `MQTT` | [[ui:seed.mqtt-clear-retained.name]] | 一个空的保留载荷发往 `lab/example/config`，QoS 1 |
| `HTTP` | [[ui:seed.http-reachable.name]] | `GET http://127.0.0.1:8080/`，超时 4000 ms |
| [[ui:seed.folder.Raw]] | [[ui:seed.udp-raw.name]] | 字节 `de ad be ef` 发往 `127.0.0.1:9000` |

要恢复初始集合，请移动或重命名 `signals.json`，然后按 [[ui:sig.reload]]：那里没有文件时，它会重新写入。

## 库文件 {#file}

库是数据文件夹中的 `signals.json`：桌面应用中是主文件夹下的 `Documents/SignalLab`，或服务器的数据文件夹（见[文件](../reference/files.md)）。将指针悬停在树下方的信号计数上可以看到完整路径。

- **自动保存。** 每次更改都会在最后一次更改 0.7 s 后写入，整个文件一次写入，通过同一文件夹中的临时文件写入，随后由该临时文件取代原文件——写入被中断时会保留上一个文件。写入等待期间，树底部显示 [[ui:sig.saving]]；随后显示 [[ui:sig.saved]]。更新重启应用之前，仍在等待的写入会先完成。
- **手动编辑。** Signal Lab 不会察觉文件在它背后发生变化。编辑它，或用来自另一台机器的文件替换它之后，请按 [[ui:sig.reload]]。重新加载会再次读取文件，并丢弃仍在等待写入的更改。
- **损坏时绝不替换。** 如果文件不是有效的 JSON，或不是信号库，树会显示错误及文件的路径、行和列，控制台也会说明相同的内容。文件保持原样，在它再次可读之前，不会有任何写入：[[ui:sig.new]]，信号和文件夹的重命名、移动和移除，拖动，信号的字段，HTTP、OSC 和 MQTT 界面上的 [[ui:sig.saveNew]]、[[ui:sig.save]] 和 [[ui:sig.saveAs]]，以及检查器和 MQTT 界面上的 [[ui:sig.fromFrame]]，全部不可用，它们的提示会说明问题所在。修复文件，或将其移除，然后按 [[ui:sig.reload]]：一旦可以读取，一切又会正常工作。
- **应用运行期间损坏的文件。** 如果您把文件编辑成无法读取的内容，而应用随后要保存更改，保存会以相同的错误被拒绝，文件保持您所编辑的样子，应用会停止写入，直到您修复它并按 [[ui:sig.reload]]。您编辑过且保持有效的文件，会在应用下一次保存时被应用的列表替换，如上所述：请先重新加载。

文件的一个简短示例：

```json
{
  "version": 2,
  "signals": [
    {
      "id": "fader-value",
      "name": "Fader value",
      "group": "Venue/Stage",
      "note": "Main fader of desk A.",
      "body": {
        "transport": "osc",
        "target": "127.0.0.1:9000",
        "address": "/fader/1",
        "args": [{ "type": "float", "value": 0.75 }]
      }
    }
  ],
  "folders": ["Venue/Stage", "Venue/Empty for now"]
}
```

| 键 | 内容 |
| --- | --- |
| `version` | `2`。版本 1 的文件（文件夹功能之前）读取方式相同，只是没有空文件夹。 |
| `signals[].id` | 创建信号时由名称生成（`fader-value`、`fader-value-2`、…），重命名时从不改变。`signallab fire` 通过它查找信号。 |
| `signals[].group` | 文件夹路径；`""` 表示顶层。 |
| `signals[].body` | 消息。`transport` 为 `osc`、`udp`、`http` 或 `mqtt`；其他键是该类型的字段。 |
| `folders` | 每个文件夹，以便保留空文件夹。没有文件夹时省略。没有条目列出的 `group` 也是一个文件夹。 |

## 从命令行 {#cli}

`signallab fire` 发送库中的一个信号，通过与应用相同的命令：

```bash
signallab fire "Fader value"
signallab fire fader-value --library ./show/signals.json
```

它先按 id 查找信号，然后按名称查找，不区分大小写。当多个信号具有该名称时，它会列出它们的 id，不发送任何内容。不带 `--library` 时，它读取应用自己的 `signals.json`；它从不写入该文件。参见[命令行](../automation/cli.md#cli-fire)。

## 相关内容 {#related}

- [检查器](inspector.md)——观察信号发送了什么，并把捕获的一帧保存为信号。
- [键盘快捷键](../reference/shortcuts.md)
- [文件](../reference/files.md)——数据文件夹在哪里。

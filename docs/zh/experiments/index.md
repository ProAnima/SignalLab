---
title: 编辑器
description: 在节点画布上构建、编辑并运行实验——添加与连接节点、选择与复制、属性面板、立即发送、校验、运行以及背后的文件。
---

# 实验编辑器

实验就是一个写成图的测试：发送数据的节点（HTTP 请求、OSC 消息、MQTT 发布……）、等待应答的节点、检查返回内容的节点、扮演另一端的节点，或按需破坏网络的节点，由说明下一步运行什么连线连接起来。一次运行从 [[ui:exp.node.start]] 开始，沿连线前进，当每个步骤都通过并到达 [[ui:exp.node.end]] 时即通过。您在 [[ui:nav.experiment]] 界面上构建它、在那里运行它，也可以用命令行或服务器运行同一个文件。

编辑器一次只保存一个实验，并在您操作时自动保存。要保留多个实验，请导出快照，或从文件中打开另一个（见[保存与文件](#files)）。

每种节点及其字段和输出见[节点参考](nodes.md)。值如何在节点之间流动见[数据与模板](data.md)；并行分支、循环、重试和重复见[流程](flow.md)。

## 界面 {#screen}

| 区域 | 内容 |
| --- | --- |
| 工具栏（顶部） | 实验的名称和保存状态、添加节点、参数、配置文件、各面板、专注模式、全屏和运行 |
| 画布栏 | 撤销与重做、节点查找器、[[ui:exp.arrange]] 和缩放 |
| 画布 | 图：节点、连线，以及在它们上面绘制的运行进度 |
| [[ui:exp.properties]]（右侧） | 所选节点的字段、预览和 [[ui:exp.sendNow]]；或选中多个节点或一条连线时允许的操作 |
| [[ui:exp.timeline]]（底部） | 当前或上一次运行的步骤、结果、报告和种子 |

属性面板和时间线在它们的靠内边缘各有一个手柄：拖动它，或用 <kbd>Tab</kbd> 聚焦后使用方向键（<kbd>Shift</kbd> 可加大步长）；双击或按 <kbd>Enter</kbd> 可让面板恢复默认大小。尺寸会保留到下次。时间线在第一次运行打开它之前一直折叠。

## 工具栏 {#toolbar}

| 控件 | 作用 |
| --- | --- |
| ☰ [[ui:exp.documents]] | 打开模板、打开 JSON 文件和导出（[保存与文件](#files)） |
| [[ui:exp.name]] | 实验的名称；运行需要它。运行报告会记录它，[[ui:exp.compare]] 依据它查找更早的运行 |
| [[ui:exp.saved]] / [[ui:exp.saving]] / [[ui:exp.saveError]] | 最后一次更改是否已写入磁盘 |
| ⚠ [[ui:exp.needsLinks]] | 实验无法运行时显示；其提示说明原因，点击可选中它所指的节点（[校验](#validation)） |
| ＋ [[ui:exp.addNode]] | 打开添加菜单，选中节点时添加在其之后（<kbd>A</kbd>） |
| [[ui:exp.profile]] | 运行、预览和立即发送使用哪个配置文件；实验有配置文件时显示。⚠ 标记无法运行的配置文件 |
| `{ }` [[ui:exp.params]] | 参数、配置文件、种子、Cookie 和机密（[数据与模板](data.md)） |
| ☷ [[ui:exp.properties]] | 显示或隐藏属性面板 |
| ▢ [[ui:exp.focus]] | 隐藏侧边栏、顶栏和底部面板（[专注模式与全屏](#focus)） |
| ⛶ [[ui:exp.fullscreen]] | 窗口进入全屏，并开启专注模式 |
| [[ui:exp.run]] | 检查、保存并运行实验；运行时按钮为 [[ui:common.stop]] |
| ▾ [[ui:exp.runWith]] | 用另一个配置文件、其他参数值或给定的种子运行一次，而不更改实验 |

## 在画布中移动 {#canvas}

- **平移**：拖动空白画布，或滚动。
- **缩放**：按住 <kbd>Ctrl</kbd> 并滚动鼠标滚轮，围绕指针缩放；画布栏中的 − 和 ＋ 每次缩放 10%。缩放范围从 15% 到 200%；两个按钮之间的百分比显示您当前的位置。
- **1:1**（[[ui:exp.resetZoom]]，<kbd>Ctrl</kbd>+<kbd>1</kbd>）回到 100%。
- ⊡ [[ui:exp.fit]]（<kbd>Ctrl</kbd>+<kbd>0</kbd>）显示整个图，最多 100%。
- **查找节点**：画布栏中的 [[ui:exp.nodes]]（显示有多少个节点）或 <kbd>Ctrl</kbd>+<kbd>F</kbd>。输入节点名称、摘要（URL、地址、主题）或 id 中的词语；<kbd>↑</kbd> <kbd>↓</kbd> 选择，<kbd>Enter</kbd> 选中该节点并把它带入视野（至少缩放到 80%），<kbd>Esc</kbd> 关闭。

没有小地图：[[ui:exp.fit]] 和查找器就能完成这项工作。

## 节点与连线 {#nodes-and-wires}

节点显示它的类型、一行它做什么的摘要，以及其设置的徽标：↻ 和数字表示重试，× 和计数（或时间）表示重复，⚡ 表示负载。它的输入在左侧——除 [[ui:exp.node.start]] 之外每个节点都有——输出在右侧：

| 输出 | 位于 | 何时被跟随 |
| --- | --- | --- |
| [[ui:exp.outputPort]] | 大多数节点 | 该步骤通过 |
| [[ui:exp.yes]] / [[ui:exp.no]] | [[ui:exp.node.branch_status]]、[[ui:exp.node.branch_value]] | 比较成立 / 不成立 |
| [[ui:exp.branch1]] / [[ui:exp.branch2]] | [[ui:exp.node.fork]] | 总是，两者同时 |
| [[ui:exp.portMatched]] / [[ui:exp.portTimeout]] | 等待节点 | 匹配的消息到达 / 没有消息及时到达 |
| [[ui:exp.portBody]] / [[ui:exp.portDone]] / [[ui:exp.portLimit]] | [[ui:exp.node.loop]] | 又一次迭代 / 循环结束 / 迭代次数用尽 |

**一个输出可以有多条连线。** 第一条连线延续当前分支；之后的每条连线都会启动一个并行分支，并带有该处已知变量的副本。一个 [[[ui:exp.node.join]]](nodes.md#node-join) 节点会等待通向它的每一条连线。详情见[流程](flow.md)。

[[ui:exp.portTimeout]] 和 [[ui:exp.portLimit]] 是可选的：不接线时，超时或迭代次数用尽都会使该步骤失败。其他每个输出都必须有连线，实验才能运行。

从 [[ui:exp.node.loop]] 的循环体返回它自身的连线，会画成横跨循环体的一条弧线；它是唯一可以向后指的连线。

## 添加节点 {#adding}

### 添加菜单 {#add-menu}

添加菜单按组列出每一种节点——[[ui:exp.group.action]]、[[ui:exp.group.observe]]、[[ui:exp.group.emulate]]、[[ui:exp.group.fault]]、[[ui:exp.group.data]]、[[ui:exp.group.check]]、[[ui:exp.group.flow]]——然后是 [[ui:exp.group.signals]]：您[库](../tools/signals.md)中可成为节点的信号（OSC、带文本载荷的 HTTP、UDP、MQTT），其字段已填好。

它打开时搜索框就获得焦点。输入几个字母：每个词都必须出现在节点的名称、描述或类型（`http`、`wait_osc`）中；信号则通过其名称、文件夹、传输方式或目标找到。<kbd>↑</kbd> <kbd>↓</kbd> 选择，<kbd>Enter</kbd> 添加，<kbd>Esc</kbd> 关闭。顶部的一行说明节点将去哪里：某个节点之后，或与某个节点并行。

新节点会被选中，属性面板打开，它的主字段——URL、地址、主题、延时——获得焦点且文本已选中，因此您可以立即输入。在字段中按 <kbd>Esc</kbd> 会回到画布上的该节点，准备按下一个 <kbd>A</kbd>。

### 在所选节点之后 {#add-after}

<kbd>A</kbd>、工具栏中的 ＋ [[ui:exp.addNode]] 以及属性面板中的 ＋ [[ui:exp.addNext]]，都会在所选节点之后添加下一步：

- 它连接到该节点第一个还没有连线的输出（比如 [[ui:exp.node.branch_status]] 空闲的 [[ui:exp.no]]），否则连接到它的第一个输出。
- 如果该输出已经有连线，新节点会被接入其中（若有多个，则接入第一个）：连线现在穿过新节点，其后的一切向右移动以腾出空间。
- 选中 [[ui:exp.node.end]] 时，节点会放在它之前，前提是有一条连线通向它。
- 未选中任何节点时，节点落在视图中央，不带连线。

一个只有单个输出的节点，如果放在 [[ui:exp.node.loop]] 空的 [[ui:exp.portBody]] 上——无论用这种方式还是[在新建连线上](#add-branch)——也会被连回它，因此循环体立即完整。

### 接入连线 {#insert}

每条连线中间都有一个 ＋（[[ui:exp.insertNode]]）。它打开添加菜单，您选中的节点会被接入该连线。以这种方式接入的 [[ui:exp.node.loop]] 会通过其 [[ui:exp.portDone]] 延续流程。

### 在新建连线上 {#add-branch}

从某个输出拖出连线并在空白画布上松开：添加菜单会在那里打开，新节点会加到该输出的一条新连线上——与它已有的连线并列，因此与它们并行运行。点击一个输出后再点击空白画布，或双击该输出，也会如此。

### 任意位置 {#add-anywhere}

双击空白画布即可在该位置添加一个节点，不带连线。

### 从其他界面 {#from-screens}

- [OSC](../protocols/osc.md) 和 [HTTP](../protocols/http.md) 界面上的 [[ui:common.toExperiment]] 会把您刚才尝试的内容作为下一步加入——当有一条连线通向 [[ui:exp.node.end]] 时，就加在它之前——并切换到编辑器。
- OSC 监视器中某条消息旁的 [[ui:osc.waitForThis]] 会添加一个识别它的 [[ui:exp.node.wait_osc]]：其地址和文本、整数参数以及 true/false 参数（浮点是会变化的测量值，因此不包含）。运行前请停止监视器：运行本身会监听该端口。
- [MQTT](../protocols/mqtt.md) 树中某个主题上的 [[ui:mq.waitForThis]] 会在该代理、该主题上添加一个 [[ui:exp.node.wait_mqtt]]。

运行进行期间无法添加节点；控制台会说明这一点。

## 连接节点 {#connecting}

- **拖动**：从输出拖到某个节点上。在距节点 24 像素以内松开即可。
- **点击**：点击一个输出（或聚焦它后按 <kbd>Enter</kbd> 或 <kbd>Space</kbd>）：画布栏会显示 [[ui:exp.chooseInput]]。点击某个节点或其输入即可连接；点击空白画布会在那里新建一条连线并添加节点；[[ui:exp.cancelLink]] 或 <kbd>Esc</kbd> 放弃。

如果连线会形成环（[[ui:exp.node.loop]] 的回路除外），或通向 [[ui:exp.node.start]]、从 [[ui:exp.node.end]] 引出，或把一个节点连到自身，都会被拒绝；编辑器会说明原因。绘制一条已存在的连线不会有任何改变。

要**移除一条连线**，点击它——属性面板会显示它连接了什么——然后按 <kbd>Delete</kbd>，或在那里使用 [[ui:exp.removeWire]]。将指针悬停在连线上，也会在其 ＋ 上方显示一个 ×。节点的属性会列出它引出的连线，每条都带 ×（[[ui:exp.disconnect]]）。

## 选择多个节点 {#selection}

| 操作 | 方法 |
| --- | --- |
| 选择一个节点 | 点击它，或用 <kbd>Tab</kbd> 移动过去 |
| 将节点加入选择，或移出 | 按住 <kbd>Shift</kbd> 或 <kbd>Ctrl</kbd> 并点击 |
| 用框选 | 按住 <kbd>Shift</kbd> 在空白画布上拖动：框触及的每个节点都会加入选择 |
| 选择所有节点 | <kbd>Ctrl</kbd>+<kbd>A</kbd> |
| 清除选择 | 点击空白画布；选中多个时按 <kbd>Esc</kbd> |
| 移动它们 | 拖动其中任意一个：它们会一起移动 |
| 微调它们 | 聚焦某个节点时，方向键将选择移动 5 像素，按住 <kbd>Shift</kbd> 则为 20 |

属性面板显示最后选中的节点；选中多个时，它会显示数量，以及 [[ui:exp.copy]]、[[ui:exp.duplicate]] 和 [[ui:exp.deleteSelected]]。

### 复制、剪切与粘贴 {#copy-paste}

<kbd>Ctrl</kbd>+<kbd>C</kbd> 将所选节点及其之间的连线复制为文本；<kbd>Ctrl</kbd>+<kbd>X</kbd> 还会移除它们；<kbd>Ctrl</kbd>+<kbd>V</kbd> 粘贴它们——粘贴到这个实验、之后打开的另一个实验，或另一个 Signal Lab 窗口。文本是 JSON，因此您也可以把它保存在文件或消息中。

- [[ui:exp.node.start]] 和 [[ui:exp.node.end]] 独一无二：它们从不被复制。
- 复制的节点与图中其余部分之间的连线不会被复制；请自行给副本接线。
- 每个粘贴的节点都会获得新的 id。
- 会引用另一个节点的节点——[[ui:exp.node.impairment_change]]、[[ui:exp.node.emulator_state]]、[[ui:exp.node.ws_send]]、[[ui:exp.node.wait_ws]]、[[ui:exp.node.ws_close]]——如果被引用的节点也一并复制了，就引用副本；否则，如果原节点在本实验中，就继续引用它，或者引用本实验中该类型的第一个节点。
- 粘贴的 [[ui:exp.node.emulator]] 或 [[ui:exp.node.impairment]]，如果其端口已被本实验监听，会移到下一个空闲端口。发送到原节点的一方仍然照旧。
- 副本落在原位置右侧 32 像素、下一行的位置，若与任何节点重叠则继续下移，并被选中。

<kbd>Ctrl</kbd>+<kbd>D</kbd>（[[ui:exp.duplicate]]）不经过剪贴板完成同样的操作。

### 删除 {#deleting}

<kbd>Delete</kbd> 或 <kbd>Backspace</kbd>（[[ui:exp.delete]]、[[ui:exp.deleteSelected]]）会移除所选节点及其连线。如果某节点恰好有一条输入连线和一条输出连线，会在它的位置留下一条连线，从它之前的节点连到之后的节点，从而保持链条相连。[[ui:exp.node.start]] 和 [[ui:exp.node.end]] 无法删除。

## 撤销与重做 {#undo}

<kbd>Ctrl</kbd>+<kbd>Z</kbd> 撤销；<kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd> 或 <kbd>Ctrl</kbd>+<kbd>Y</kbd> 重做（画布栏中的 ↶ ↷）。一次拖动、一连串方向键微调，或在一个字段中的输入，都算一步。应用打开期间会保留最近 100 步，跨界面切换也保留；打开另一个实验也算一步，因此 <kbd>Ctrl</kbd>+<kbd>Z</kbd> 可以恢复上一个实验。运行进行期间，撤销和重做会等待。

## 排列 {#arrange}

[[ui:exp.arrange]] 从左到右排列图：每个节点放在通向它的最后一个节点之后的那一列，[[ui:exp.node.loop]] 的循环体放在它所在的行——然后适应视图。这是历史记录中的一步。带有非 Loop 回路的环的草稿保持原样。

## 属性面板 {#properties}

选中一个节点时，面板从上到下显示：

1. 节点的名称（其提示说明它做什么），以及当实验因这个节点而无法运行时，出了什么问题。
2. 它的字段。接受[模板](data.md)的字段会在您输入 `{{` 时，或按 <kbd>Ctrl</kbd>+<kbd>Space</kbd> 时，建议参数、变量、机密和生成器。
3. 它的设置：HTTP 请求上的 [[ui:exp.loadOn]]、会发送的节点上的 [[ui:exp.repeatOn]]、会发送或监听的节点上的 [[ui:exp.retryOn]]，以及 OSC 和 UDP 消息上的 [[ui:exp.expectReply]]（见[节点设置](nodes.md#settings)）。负载开启时会取代重复和重试。
4. 上一次运行的负载结果，出现在按负载运行的 HTTP 节点上。
5. 预览——[[ui:exp.preview]]、[[ui:exp.previewWait]] 或 [[ui:exp.previewCheck]]——当节点带有模板时：用当前参数和目前已知的值解析出它将发送、等待或比较的内容（[立即发送与预览](#send-now)）。
6. [[ui:exp.sendNow]] 或 [[ui:exp.listenNow]]，以及它上次做了什么。
7. 它引出的连线，每条都带 ×。
8. OSC 或 UDP 消息上的 ⚡ [[ui:exp.routeThrough]]：会在该节点前面放一个 [[ui:exp.node.impairment]]——从 9010 起在一个空闲的环回端口上监听，转发到节点的目标——并把节点指向它，因此下一次运行会劣化它发送的内容（[故障](faults.md)）。
9. ＋ [[ui:exp.addNext]]、[[ui:exp.copy]]、[[ui:exp.duplicate]] 和 [[ui:exp.delete]]。

双击某个节点会打开面板并把光标放在它的主字段中。选中多个节点时，面板提供可对它们全体执行的操作；选中一条连线时，提供它连接了什么以及 [[ui:exp.removeWire]]。运行进行期间，字段会被锁定。

## 立即发送与预览 {#send-now}

[[ui:exp.sendNow]]（<kbd>Ctrl</kbd>+<kbd>Enter</kbd>，也可在节点的字段内使用）会单独发送所选节点，不运行整个实验，使用的代码与一次运行相同。它在 [[ui:exp.node.http]]、[[ui:exp.node.tcp]]、[[ui:exp.node.osc]]、[[ui:exp.node.udp]]、[[ui:exp.node.mqtt]]、[[ui:exp.node.ws_connect]] 和 [[ui:exp.node.ws_send]] 上提供。对于等待节点，它是 [[ui:exp.listenNow]]：等待从现在开始监听，直到有消息匹配或超时结束。

- 节点使用活动的配置文件、已存储的机密，以及目前已知的变量值——来自上一次运行和之前的 [[ui:exp.sendNow]] 结果。如果某个模板引用了尚无人设置的值，则什么都不发送，并列出这些名称。
- 它只发送一次：重试、重复和负载都不适用，不保留 Cookie，实验的模拟器和中继也不会启动。
- [[ui:exp.node.ws_send]] 或 [[ui:exp.node.wait_ws]] 会为那一次测试打开其 [[ui:exp.node.ws_connect]] 所描述的连接。
- 对于 HTTP 请求，会显示响应——状态、时间、大小，以及经过 JSON 格式化的正文。请求之后 [[ui:exp.node.extract]] 节点将提取的值会立即填入。点击 JSON 响应中的某个值，即可在请求之后立即为它添加一个 [[ui:exp.node.extract]] 节点，其变量已命名、值已知。[[ui:http.mockThis]] 会把该响应转换为一个[模拟器](../tools/emulators.md)的路由。
- 结果也会写入控制台。

预览同样由引擎解析，在更改后约四分之一秒进行。机密值在那里或任何地方都从不显示。

## 校验 {#validation}

编辑器在您编辑时检查实验，并在您按 [[ui:exp.run]] 时再检查一次：

- 仍需要连线的输出会呈琥珀色脉动。
- 从 [[ui:exp.node.start]] 出发的任何连线都无法到达的节点，会画成虚线；其提示会说明这一点。
- 问题所涉及的节点会加轮廓，问题写在它的字段上方。
- 工具栏中的 ⚠ [[ui:exp.needsLinks]] 会指出问题；点击可选中该节点。

实验在满足以下条件（以及其他条件）时才能运行：

- 它有名称、恰好一个 [[ui:exp.node.start]] 和一个 [[ui:exp.node.end]]，且节点最多 64 个；
- 每个节点都能从 [[ui:exp.node.start]] 到达，且每个必需的输出都有连线；
- 唯一的环是 [[ui:exp.node.loop]] 节点回到自身的循环体；
- 每个检查节点和 [[ui:exp.node.extract]] 在每条路径上其之前都有 HTTP 请求（按负载运行的那个不算：它不留下响应）；
- [[ui:exp.node.ws_send]]、[[ui:exp.node.wait_ws]] 或 [[ui:exp.node.ws_close]] 出现在它所用的 [[ui:exp.node.ws_connect]] 之后；
- 每个字段都已填写且在范围内，每个模板引用的都是该处已知的内容。

未完成的草稿同样会被保存。如果运行所需的某个机密未存储，运行会在发出任何流量之前被拒绝。每条消息都列在[错误参考](../reference/errors.md)中。

## 运行 {#running}

[[ui:exp.run]] 检查实验、保存它并启动它。如果它无法运行，会显示问题并选中相关节点。否则时间线会打开，节点在运行到达它们时亮起：● 运行中、✓ 通过、✕ 失败、↻ 重试中、⟳ 重复中、⚡ 负载中；承载过流程的连线也会着色。

- 等待节点、模拟器和损伤中继会在第一个步骤之前打开其端口，因此早到的内容不会被错过。无法打开的端口（比如被其他程序占用）会阻止运行启动，并显示需要它的那个节点。
- [[ui:common.stop]] 会立即结束运行：包括暂停、等待和负载。运行也是控制台任务条中的一个任务，任务条也能停止它。
- 耗时超过 300 秒的运行会被停止并判定为失败。
- 运行进行期间无法编辑实验；平移、缩放和选择仍然可用。

按钮旁的 ▾ [[ui:exp.runWith]] 会用另一个配置文件、其他参数值或给定的种子运行一次；实验本身不会被更改，表单会保留您输入的内容直到应用关闭（[数据与模板](data.md)）。

### 运行时间线 {#timeline}

[[ui:exp.timeline]] 为每个步骤列出一行：时间、节点以及发生了什么——通过、失败及原因、一次重试、重复的进度、负载每秒的数字。点击某一行可在画布上选中它的节点。

它的标题行包含：

- 结果：[[ui:exp.passed]]、带原因的 [[ui:exp.failed]]，或 [[ui:exp.stopped]]；
- [[ui:exp.reportSaved]]——运行的报告文件（在服务器上是一个下载）；
- [[ui:exp.compare]]——本次运行与同一实验更早的一次运行并列比较；
- 运行所用的配置文件和更改过的值（如果有）；
- 运行的种子，以及 [[ui:exp.pinSeed]] 可将它固定在实验中，好让接下来的运行抽取相同的随机值，或固定后出现的 [[ui:exp.unpinSeed]]。

当[检查器](../tools/inspector.md)正在捕获时，匹配的等待（或预期的回复）会链接到它匹配的那一帧；点击会在检查器中打开它。报告、种子和运行比较见[运行与报告](runs.md)。

## 专注模式与全屏 {#focus}

▢ [[ui:exp.focus]] 隐藏侧边栏、顶栏和底部面板（控制台和检查器），把屏幕留给编辑器。⛶ [[ui:exp.fullscreen]] 让窗口进入全屏并开启专注模式；离开全屏会把专注模式恢复到原来的状态。切换到另一个界面会离开专注模式。

在画布上按 <kbd>Esc</kbd>，在没有其他东西可以关闭时，会先离开全屏，然后离开专注模式。

## 保存与文件 {#files}

实验会在每次更改后片刻自动保存到数据文件夹中的 `experiment.json`（桌面上是 `Documents/SignalLab`；服务器有自己的——见[文件与文件夹](../reference/files.md)）。尚无法运行的草稿也会保存。运行会先保存，因此运行的就是磁盘上的内容。

如果该文件无法读取——比如被手工编辑成了损坏的 JSON——编辑器会说明是哪个文件以及原因，并不去动它。从 ☰ [[ui:exp.documents]] 打开另一个实验后，它会在下次保存时被替换。

☰ [[ui:exp.documents]] 打开实验对话框：

- [[ui:exp.templates]]：选一个并按 [[ui:exp.openDocument]]。它们都使用环回地址。
- [[ui:exp.importJson]] 读取一个实验文件——由这个版本或更早版本的 Signal Lab 写入，最多 4 MiB——并在打开前显示它的名称以及有多少节点和连接。来自更早版本的文件会在打开时更新到最新。无法解析或不会是有效实验的文件会被拒绝并给出原因，当前实验保持不变。
- [[ui:exp.exportJson]] 会把当前实验的快照——节点、连线、位置、参数和配置文件，绝不包含机密值——写入数据文件夹中 `exports` 里的一个新文件，并显示其路径；在服务器上则带有一个 [[ui:common.download]] 链接。
- [[ui:exp.openDocument]] 用模板或文件替换当前实验。它不会运行该实验，<kbd>Ctrl</kbd>+<kbd>Z</kbd> 可以恢复上一个。

| 模板 | 作用 |
| --- | --- |
| [[ui:exp.templateEmpty]] | [[ui:exp.node.start]] 和 [[ui:exp.node.end]]，用于您自己的流程 |
| [[ui:exp.templateHttp]] | 向 `http://127.0.0.1:8080/` 发送 GET 并检查状态是否为 200——您开始使用的实验 |
| [[ui:exp.templateBranch]] | 同样的请求；为 200 时向 `127.0.0.1:9000` 发送一条 OSC 消息，否则延时 500 ms |
| [[ui:exp.templateParallel]] | 同时两个分支——一个请求和一行日志——在 [[ui:exp.node.end]] 之前汇合 |
| [[ui:exp.templatePingReply]] | 把带本次运行 id 的 `/ping` 发送到 `127.0.0.1:9000`，并在 `127.0.0.1:9001` 上等待携带它返回的 `/pong` |
| [[ui:exp.templatePoll]] | 每 0.3 s 向设备询问一次 `/status`，直到它回答 `ready`，最多十次 |
| [[ui:exp.templateFlaky]] | 一个在应答前失败两次的模拟 API，以及一个不断询问直到它应答的循环 |
| [[ui:exp.templateFaults]] | 经过损伤中继向模拟设备发送数据报，同时一个并行分支把网络依次切换为干净、丢包、离线、再干净 |
| [[ui:exp.templateOutage]] | 一个被并行分支停机两秒的模拟 API，以及一个不断询问直到它再次应答的客户端 |
| [[ui:exp.templateWsEcho]] | 连接到 `ws://127.0.0.1:9001/echo` 的回显服务，发送一个 JSON ping，期望原样返回，然后关闭 |

同样的文件无需编辑器即可运行：`signallab run experiment.json`——见[命令行](../automation/cli.md)。

## 键盘快捷键 {#shortcuts}

单键作用于画布，在字段中输入时不受打扰。在 Mac 上、在浏览器中，<kbd>Cmd</kbd> 可替代所写的 <kbd>Ctrl</kbd>。

| 按键 | 操作 |
| --- | --- |
| <kbd>A</kbd> | 在所选节点之后添加一个节点，未选中时添加在视图中部 |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | 对所选节点执行 [[ui:exp.sendNow]] 或 [[ui:exp.listenNow]] |
| <kbd>Ctrl</kbd>+<kbd>Z</kbd> | 撤销 |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd>、<kbd>Ctrl</kbd>+<kbd>Y</kbd> | 重做 |
| <kbd>Ctrl</kbd>+<kbd>A</kbd> | 选择所有节点 |
| <kbd>Ctrl</kbd>+<kbd>C</kbd> / <kbd>Ctrl</kbd>+<kbd>X</kbd> / <kbd>Ctrl</kbd>+<kbd>V</kbd> | 复制 / 剪切 / 粘贴所选节点及其之间的连线 |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | 复制所选节点 |
| <kbd>Delete</kbd>、<kbd>Backspace</kbd> | 移除所选连线，或所选节点 |
| 方向键（聚焦某个节点时） | 将所选节点移动 5 像素；按住 <kbd>Shift</kbd> 则为 20 |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | 查找节点 |
| <kbd>Ctrl</kbd>+<kbd>0</kbd> | 适应图 |
| <kbd>Ctrl</kbd>+<kbd>1</kbd> | 缩放到 100% |
| <kbd>Ctrl</kbd> + 鼠标滚轮 | 围绕指针缩放 |
| <kbd>Shift</kbd> + 点击、<kbd>Ctrl</kbd> + 点击 | 将节点加入选择，或移出 |
| <kbd>Shift</kbd> + 在空白画布上拖动 | 框选 |
| 双击某个节点 | 编辑其字段 |
| 双击空白画布 | 在那里添加一个节点 |
| 在聚焦的输出上按 <kbd>Enter</kbd>、<kbd>Space</kbd> | 从它开始一条连线 |
| 在字段中输入 `{{` 或按 <kbd>Ctrl</kbd>+<kbd>Space</kbd> | 建议参数、变量、机密和生成器 |
| 在字段中按 <kbd>Esc</kbd> | 回到画布上的节点 |
| 在画布上按 <kbd>Esc</kbd> | 关闭添加菜单；否则取消正在绘制的连线；否则放开选中的连线；否则清除多选；否则离开全屏；否则离开专注模式 |

在添加菜单和查找器中，<kbd>↑</kbd> <kbd>↓</kbd> 选择，<kbd>Enter</kbd> 确认选择，<kbd>Esc</kbd> 关闭。应用的所有快捷键见[键盘快捷键](../reference/shortcuts.md)。

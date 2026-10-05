---
title: 键盘快捷键
description: Signal Lab 的每一个键盘快捷键——应用各处、实验编辑器、信号库、工具界面、窗格和语言菜单。
---

# 键盘快捷键

Signal Lab 应答的每一个按键，按生效的位置分组。单个字母或 Delete 的快捷键在字段中输入时从不生效。

::: tip
在 Mac 上的浏览器中，下文写作 <kbd>Ctrl</kbd> 的地方都可用 <kbd>⌘</kbd>，但模板字段中的 <kbd>Ctrl</kbd>+<kbd>Space</kbd> 除外。
:::

## 任意位置 {#anywhere}

| 按键 | 作用 |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>K</kbd> | 在任意界面打开 [[ui:sig.paletteTitle]]，即信号面板——即使在字段中输入时也可以。再次按下则关闭 |
| <kbd>F1</kbd> | 打开本文档中您当前界面所在的页面，与 [[ui:app.docs]] 相同 |
| <kbd>Tab</kbd>、<kbd>Shift</kbd>+<kbd>Tab</kbd> | 在控件之间移动。控件获得焦点时会显示其提示；按下其他按键则隐藏 |
| <kbd>Space</kbd>、<kbd>Enter</kbd> | 按下拥有焦点的按钮——每个可点击的东西都是按钮 |
| <kbd>Esc</kbd> | 关闭当前打开的对话框 |

在信号面板中：

| 按键 | 作用 |
| --- | --- |
| 输入 | 筛选信号 |
| <kbd>↑</kbd> <kbd>↓</kbd> | 选择一个信号 |
| <kbd>Enter</kbd> | 发送它并关闭面板 |
| <kbd>Esc</kbd> | 关闭面板；点击其旁边也可以 |

## 实验编辑器 {#editor}

在画布上——当没有文本字段拥有焦点时：

| 按键 | 作用 |
| --- | --- |
| <kbd>A</kbd> | 打开添加节点的菜单（[[ui:exp.addNode]]）。选中某个节点时，新节点会加在它后面 |
| <kbd>Delete</kbd> 或 <kbd>Backspace</kbd> | 删除选中的连线，或选中的节点——绝不会删除 [[ui:exp.node.start]] 或 [[ui:exp.node.end]] |
| <kbd>Tab</kbd> | 在节点、其端口和连线之间移动；获得焦点的节点或连线会被选中 |
| 在某个输出上按 <kbd>Enter</kbd> 或 <kbd>Space</kbd>，然后在某个节点或其输入上按 | 将它们连接起来 |
| <kbd>←</kbd> <kbd>→</kbd> <kbd>↑</kbd> <kbd>↓</kbd> | 当某个节点拥有焦点时，将选中的节点移动 5 点（按住 <kbd>Shift</kbd> 为 20）。按住不放算作历史记录中的一步 |
| <kbd>Esc</kbd> | 关闭添加菜单；否则取消正在绘制的连线；否则放下选中的连线；否则清除多节点选择；否则退出 [[ui:exp.fullscreen]]；否则退出 [[ui:exp.focus]] |
| <kbd>Ctrl</kbd>+<kbd>Z</kbd> | [[ui:exp.undo]] |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd>、<kbd>Ctrl</kbd>+<kbd>Y</kbd> | [[ui:exp.redo]] |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | [[ui:exp.duplicate]]：选中节点的副本，以及它们之间的连线 |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | 按类型、作用或 id 查找节点 |
| <kbd>Ctrl</kbd>+<kbd>0</kbd> | [[ui:exp.fit]]：让整个图可见 |
| <kbd>Ctrl</kbd>+<kbd>1</kbd> | [[ui:exp.resetZoom]]：100 % |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:exp.sendNow]]——对于等待类节点，则是 [[ui:exp.listenNow]]：只对选中的节点执行，而不运行实验 |
| <kbd>Ctrl</kbd>+<kbd>A</kbd> | 选中所有节点 |
| <kbd>Ctrl</kbd>+<kbd>C</kbd> | [[ui:exp.copy]]：选中的节点以及它们之间的连线；[[ui:exp.node.start]] 和 [[ui:exp.node.end]] 不会包含在内，与复制时相同 |
| <kbd>Ctrl</kbd>+<kbd>X</kbd> | 剪切它们 |
| <kbd>Ctrl</kbd>+<kbd>V</kbd> | 粘贴在此处或另一个实验中复制的节点，并使用新的 id |

<kbd>Ctrl</kbd>+<kbd>A</kbd>、<kbd>C</kbd>、<kbd>X</kbd> 和 <kbd>V</kbd>
在编辑器拥有焦点时作用于节点，或者在没有任何控件拥有焦点、且页面上没有选中文本时生效。当别处选中了文本——在
[[ui:console.title]] 中，或在某份报告里——它们则属于页面本身：
<kbd>Ctrl</kbd>+<kbd>C</kbd> 复制该文本。

使用鼠标：

| 手势 | 作用 |
| --- | --- |
| 在空白画布上拖动 | 移动视图 |
| 在空白画布上按住 <kbd>Shift</kbd> 拖动 | 将框选接触到的节点加入选择 |
| 在节点上按住 <kbd>Shift</kbd> 或 <kbd>Ctrl</kbd> 点击 | 将其加入选择，或移出选择 |
| 拖动多个选中节点中的一个 | 移动它们全部 |
| 在空白画布上双击 | 在那里打开添加菜单 |
| <kbd>Ctrl</kbd> + 滚轮 | 在指针处缩放 |

在添加菜单、节点查找器和属性中：

| 位置 | 按键 | 作用 |
| --- | --- | --- |
| 添加菜单 | 输入 | 筛选节点和已保存的信号 |
| 添加菜单 | <kbd>↑</kbd> <kbd>↓</kbd>、<kbd>Enter</kbd>、<kbd>Esc</kbd> | 选择、添加、关闭 |
| 节点查找器 | <kbd>↑</kbd> <kbd>↓</kbd> | 选择一个节点 |
| 节点查找器 | <kbd>Enter</kbd> | 在画布上显示并选中它 |
| 节点查找器 | <kbd>Tab</kbd> | 返回搜索字段 |
| 节点查找器 | <kbd>Esc</kbd> | 关闭它 |
| [[ui:exp.properties]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | 对所示节点执行 [[ui:exp.sendNow]] |
| [[ui:exp.properties]] | 在字段中按 <kbd>Esc</kbd> | 返回画布并选中该节点，此时 <kbd>A</kbd> 可添加下一个节点 |

在可接受模板（`{{name}}`）的字段中：

| 按键 | 作用 |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>Space</kbd> | 在光标处输入 `{{` 并列出可放入的内容：参数、变量、机密、生成器 |
| 输入 `{{` | 列出相同的内容 |
| <kbd>↑</kbd> <kbd>↓</kbd> | 在列表中选择 |
| <kbd>Enter</kbd> 或 <kbd>Tab</kbd> | 插入所选项并补全 `}}` |
| <kbd>Esc</kbd> | 关闭列表；字段保持焦点 |

在实验的对话框中：

| 位置 | 按键 | 作用 |
| --- | --- | --- |
| [[ui:exp.params]] | 在最后一个参数的值中按 <kbd>Enter</kbd> | 添加另一个参数 |
| [[ui:exp.params]]、[[ui:exp.runWith]] | <kbd>Esc</kbd> | 关闭对话框 |
| [[ui:exp.runWith]] | 在字段中按 <kbd>Enter</kbd> | 使用这些值执行 [[ui:exp.run]] |
| [[ui:exp.secrets]] | 在某个值中按 <kbd>Enter</kbd> | 保存它 |
| [[ui:exp.secrets]] | 在某个值或新名称中按 <kbd>Esc</kbd> | 取消；[[ui:exp.params]] 保持打开 |

## 信号库 {#signals}

| 位置 | 按键 | 作用 |
| --- | --- | --- |
| [[ui:nav.signals]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:sig.fire]]：发送选中的信号 |
| [[ui:nav.signals]] | 双击某个信号 | 发送它 |
| 某个文件夹 | <kbd>F2</kbd> | [[ui:sig.renameFolder]] |
| 重命名文件夹时 | <kbd>Enter</kbd>、<kbd>Esc</kbd> | 保存新名称，或取消 |
| 某个文件夹 | 按两次 <kbd>Delete</kbd> | [[ui:sig.removeFolder]]；其中的内容会上升一级 |
| 某个文件夹 | <kbd>→</kbd> <kbd>←</kbd> | 展开它、折叠它 |
| [[ui:nav.http]]、[[ui:nav.osc]]、[[ui:nav.mqtt]] 发送器 | <kbd>Ctrl</kbd>+<kbd>S</kbd> | 当消息与某个库信号关联（从它打开或保存到它）且已被修改时，执行 [[ui:sig.save]]；当它还不在库中时，执行 [[ui:sig.saveNew]] |
| [[ui:sig.saveTitle]] | <kbd>Enter</kbd>、<kbd>Esc</kbd> | 保存，或取消 |

## 工具界面 {#tools}

| 界面 | 按键 | 作用 |
| --- | --- | --- |
| [[ui:nav.http]] | 在 URL、某个标头、凭据或超时中按 <kbd>Enter</kbd> | 发送请求 |
| [[ui:nav.http]] | 在请求中的任意位置（包括正文）按 <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | 发送请求 |
| [[ui:nav.osc]] | 在目标、地址或某个参数中按 <kbd>Enter</kbd> | 发送消息 |
| [[ui:nav.ws]] | 在 URL 中按 <kbd>Enter</kbd> | 连接 |
| [[ui:nav.ws]] | 在 [[ui:ws.message]] 中按 <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | 发送它 |
| [[ui:nav.mqtt]] | 在 [[ui:mq.addSubscription]] 中按 <kbd>Enter</kbd> | 订阅 |
| [[ui:nav.broadcast]] | 在 [[ui:bc.targetAddress]] 或 [[ui:bc.targetCidr]] 中按 <kbd>Enter</kbd>——除 [[ui:bc.mode.list]] 外的所有模式 | [[ui:bc.sendOnce]] |

在 [[ui:feedback.title]] 中：在 [[ui:feedback.message]] 中按
<kbd>Ctrl</kbd>+<kbd>Enter</kbd> 会发送它（在
[[ui:feedback.email]] 中按 <kbd>Enter</kbd> 也可以），在对话框任意位置按
<kbd>Ctrl</kbd>+<kbd>V</kbd> 会从剪贴板粘贴截图，按 <kbd>Esc</kbd> 则关闭它。

## 窗格 {#panes}

窗格之间的手柄——[[ui:layout.console]]、[[ui:layout.properties]]、
[[ui:layout.timeline]]——可用 <kbd>Tab</kbd> 获得焦点：

| 按键 | 作用 |
| --- | --- |
| <kbd>↑</kbd> <kbd>↓</kbd> | 控制台和时间线：变高、变矮，每次 16 像素 |
| <kbd>←</kbd> <kbd>→</kbd> | 属性：变宽、变窄，每次 16 像素（界面从右向左阅读时方向相反） |
| <kbd>Shift</kbd> + 方向键 | 移动四倍距离 |
| <kbd>Home</kbd>、<kbd>End</kbd> | 最小尺寸、最大尺寸 |
| <kbd>Enter</kbd> | 默认尺寸；双击手柄也可以 |

## 语言菜单 {#language}

顶栏中的旗帜和字母。

| 位置 | 按键 | 作用 |
| --- | --- | --- |
| 在按钮上 | <kbd>↓</kbd> 或 <kbd>↑</kbd> | 打开列表 |
| 在列表中 | <kbd>↓</kbd> <kbd>↑</kbd> | 下一个语言、上一个语言 |
| 在列表中 | <kbd>Home</kbd>、<kbd>End</kbd> | 第一个、最后一个 |
| 在列表中 | 一个字母 | 下一个其名称——用其自身语言、您的语言或英语——或字母以该字母开头的语言 |
| 在列表中 | <kbd>Enter</kbd> 或 <kbd>Space</kbd> | 切换到它 |
| 在列表中 | <kbd>Esc</kbd> 或 <kbd>Tab</kbd> | 关闭列表 |

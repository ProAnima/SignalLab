---
title: 快速上手
description: 在一台计算机上的第一次使用——发送一条 OSC 消息并看着它到达，把它保存为信号，向模拟的 API 发出请求，再构建并运行一个小实验。
---

# 快速上手

这次操作只需要 Signal Lab：一切都发往 `127.0.0.1`，即本机，因此不涉及任何设备、网络或防火墙规则。您将：

1. 发送一条 OSC 消息，并看着它到达；
2. 在检查器中查看同一条消息；
3. 把它保存到库中，并在任何地方再次发送；
4. 启动一个模拟的 HTTP API，并向它发出请求；
5. 针对该 API 运行一个实验，查看它失败的原因，修复它，并添加一项检查。

如果还没有安装 Signal Lab，请参见[安装与更新](install.md)。不确定某样东西在窗口的哪里？请参见[窗口](interface.md)。

## 发送一条 OSC 消息并看着它到达 {#osc}

首先，需要有东西来接收消息：OSC 界面的监视器。

1. 在侧边栏中打开 [[ui:nav.osc]]。
2. 在 [[ui:osc.monitor]] 下，将 [[ui:common.bind]] 设为 `127.0.0.1:9000`，让监视器只监听本机。
3. 按 [[ui:osc.listen]]。按钮变为 [[ui:common.stop]]，控制台提示监视器正在监听，监视器也会作为一个任务出现在底部面板的任务条中。

然后，从旁边的发送器发送消息：

4. 在 [[ui:osc.sender]] 下，保持 [[ui:common.target]] 为 `127.0.0.1:9000`，即监视器监听的端口。
5. 保持 [[ui:common.address]] 为 `/hello/avatar/1`，[[ui:common.arguments]] 下唯一的浮点参数为 `1.0`——也可以输入您自己的地址和值。
6. 按 [[ui:common.send]]，或在目标或地址字段中按 <kbd>Enter</kbd>。

监视器的表格中会出现一行：[[ui:common.time]]（到达的时刻）、[[ui:osc.from]]（`127.0.0.1` 和发送端口）、[[ui:osc.address]] 和 [[ui:osc.args]]。发送器下方会有一行确认发送了什么及其字节大小；再次发送，它会统计重复的次数。

::: tip 出现了关于防火墙的提示？
在 Windows 上，启动监视器时，顶栏下方可能会出现一条关于 Windows 防火墙的提示。它针对的是来自**其他**机器的消息；`127.0.0.1` 上的流量从不被过滤。现在先按 [[ui:fw.dismiss]]——[防火墙提示](interface.md#firewall-notice)说明了何时应该允许。
:::

## 在检查器中查看 {#inspector}

检查器会记录每个工具发送和接收的每一帧，但只在捕获开启期间。

1. 在底部面板中，打开 [[ui:dock.inspector]] 标签页。
2. 按 [[ui:ins.arm]]。标签页的圆点会亮起。
3. 回到发送器，再按一次 [[ui:common.send]]。

会出现两行，最新的在最前：发出的消息（→）和监视器收到的消息（←），每行都有协议、对端地址、大小和摘要。点击其中一行：[[ui:ins.detail]] 会显示是哪个工具发送或接收了它、使用了哪些地址、[[ui:ins.decoded]] 后的消息，以及构成它的 [[ui:ins.rawBytes]]。

完成后按 [[ui:ins.disarm]]；捕获关闭时没有任何开销。更多内容见[检查器](../tools/inspector.md)。

## 保存为信号并再次发送 {#signal}

以后还会用到的消息，应该放进信号库。

1. 在 OSC 界面上，按发送器下方的 [[ui:sig.saveNew]]。
2. 在 [[ui:sig.saveTitle]] 对话框中，将 [[ui:sig.name]] 设为 `First message`，将 [[ui:sig.group]] 设为 `Tutorial`——保存到新文件夹时会自动创建它。
3. 按 [[ui:sig.saveConfirm]]。

现在发送器已与该信号关联：按钮显示 [[ui:sig.savedState]]，旁边的标签显示信号所在的位置。修改参数后，标签会标出这一改动；[[ui:sig.save]]（<kbd>Ctrl</kbd>+<kbd>S</kbd>）会更新该信号。

现在用三种方式再次发送它：

- **从库中发送**。点击标签：[[ui:nav.signals]] 界面会打开，并选中 `Tutorial` 文件夹中的该信号（也可以打开 [[ui:nav.signals]]，在那里点击它）。按 [[ui:sig.fire]] 或 <kbd>Ctrl</kbd>+<kbd>Enter</kbd>；在列表中双击它也会发送。
- **在任何地方发送**。在任意界面按 <kbd>Ctrl</kbd>+<kbd>K</kbd>，输入 `first`，然后按 <kbd>Enter</kbd>。
- **从实验中发送**。添加节点时，菜单会在 [[ui:exp.group.signals]] 下列出您的信号，选中即可成为发送该信号的步骤。

每次发送，监视器都会显示消息到达，控制台会写出信号的名称。信号发送的内容与其所属界面发送的完全一致。更多内容见[信号](../tools/signals.md)。

用完 OSC 后，在监视器上按 [[ui:common.stop]]。

## 向模拟的 API 发出请求 {#emulator}

Signal Lab 自带五个模拟器，都在 `127.0.0.1` 上。其中的 [[ui:seed.emu.demo-api.name]] 是位于 `127.0.0.1:8080` 的 HTTP API，包含以下路由：

| 请求 | 应答 |
| --- | --- |
| `GET /health` | `200`，带 `{"status":"ok","time":"…"}`，即当前时间 |
| `GET /users/:id` | `200`，带该 id 的用户，例如 `{"id":"42","name":"User 42"}` |
| `POST /users` | `201`，带 `Location` 标头和新的 id |
| `GET /slow` | 1.5 秒后返回 `200` |
| 任意方法，`/flaky` | `503`、`503`，从第三个请求起返回 `200` |
| 其他任何请求 | `404` |

1. 打开 [[ui:nav.emulators]]。[[ui:emu.library]] 中列出了这五个模拟器；选择 [[ui:seed.emu.demo-api.name]]。
2. 按 [[ui:emu.start]]。它现在会在 `127.0.0.1:8080` 上应答，并作为任务运行。
3. 打开 [[ui:nav.http]]。方法为 `GET`；将 URL 设为 `http://127.0.0.1:8080/health`。
4. 按 [[ui:common.send]]，或在 URL 中按 <kbd>Enter</kbd>。

在 [[ui:http.response]] 下可以看到 [[ui:http.status]] `200`、[[ui:http.latency]]、[[ui:http.size]]、响应标头和 JSON 正文。将 `http://127.0.0.1:8080/flaky` 发送三次：先是两次 `503` 应答，然后是 `200`——在一个会重试的客户端看来，能自行恢复的服务就是这样的。

回到 [[ui:nav.emulators]]，[[ui:emu.live]] 面板统计每个请求，[[ui:emu.received]] 列出每个请求，以及应答它的 [[ui:emu.col.rule]] 和 [[ui:emu.col.reply]]。让 [[ui:seed.emu.demo-api.name]] 继续运行，下一部分还会用到。更多内容见[模拟器](../tools/emulators.md)。

## 运行实验 {#experiment}

实验是可以反复运行的步骤流程。Signal Lab 第一次打开时显示的实验——[[ui:exp.templateHttp]] 模板——会向 `http://127.0.0.1:8080/` 发送请求，并检查应答是否为 `200`。

### 打开模板 {#open-template}

1. 打开 [[ui:nav.experiment]]。
2. 如果画布上没有显示四个节点（[[ui:exp.node.start]]、[[ui:exp.node.http]]、[[ui:exp.node.assert_status]]、[[ui:exp.node.end]]），请按工具栏左侧的 **☰**（[[ui:exp.documents]]），在模板列表中选择 [[ui:exp.templateHttp]]，然后按 [[ui:exp.openDocument]]。打开会替换画布上的实验；<kbd>Ctrl</kbd>+<kbd>Z</kbd> 可以恢复之前的实验。

点击一个节点，可在右侧的 [[ui:exp.properties]] 中查看其设置。实验会在编辑时自动保存。

### 运行并查看失败原因 {#first-run}

3. 按 [[ui:exp.run]]。

[[ui:exp.timeline]] 会在画布下方打开，每个步骤开始时（[[ui:exp.running]]）和结束时各显示一行：时间、节点以及结果。这次运行会失败：

- [[ui:exp.node.start]] 通过，并给出本次运行的种子。
- [[ui:exp.node.http]] 通过：请求已发出，并收到了应答 `HTTP 404`。
- [[ui:exp.node.assert_status]] 失败：它期望 `200`，收到的却是 `404`。

[[ui:seed.emu.demo-api.name]] 没有 `/` 的路由，所以应答了 `404`——检查发现了这一点。时间线顶部的一行显示 [[ui:exp.failed]] 及原因。点击某一行，可以在画布上选中对应的节点。

::: tip 请求本身失败了？
如果 [[ui:exp.node.http]] 步骤因连接被拒绝而失败，说明 `127.0.0.1:8080` 上没有任何程序在监听：请在 [[ui:nav.emulators]] 中启动 [[ui:seed.emu.demo-api.name]]，然后再次运行。
:::

### 修复请求 {#fix}

4. 点击 [[ui:exp.node.http]] 节点。
5. 在 [[ui:exp.properties]] 中，将 [[ui:sig.url]] 改为 `http://127.0.0.1:8080/health`。
6. 按 [[ui:exp.run]]。

这一次每个步骤都会通过：[[ui:exp.node.assert_status]] 显示 [[ui:exp.step.checked]]，[[ui:exp.node.end]] 显示 [[ui:exp.step.complete]]，时间线的标题显示 [[ui:exp.passed]]。

### 添加检查 {#add-check}

状态码 `200` 只说明服务做出了应答，并不说明它应答了什么。把正文也检查一下：

7. 点击 [[ui:exp.node.assert_status]] 节点。
8. 在 [[ui:exp.properties]] 中按 [[ui:exp.addNext]]，或在画布获得焦点时按 <kbd>A</kbd>。会打开一个带搜索框的节点菜单。
9. 输入 `assert_body`，然后按 <kbd>Enter</kbd>。[[ui:exp.node.assert_body]] 节点会被添加到 [[ui:exp.node.assert_status]] 与 [[ui:exp.node.end]] 之间，已经连好线，其 [[ui:exp.contains]] 字段可以直接输入。
10. 输入 `"status":"ok"`。
11. 按 [[ui:exp.run]]。

新步骤会通过。把文本改成正文中没有的内容，再运行一次，就能看到它失败并给出原因。

### 运行留下了什么 {#report}

- **一份报告**。运行结束时，时间线标题中会出现 [[ui:exp.reportSaved]]；将指针悬停在其上可以查看文件。结束的运行无论通过还是失败，都会在数据文件夹的 `runs` 文件夹中写入一份报告，包含它所用的值和每个步骤。在浏览器中，这是一个下载链接。
- **一个种子**。标题中还会显示本次运行的种子以及 [[ui:exp.pinSeed]]：运行中的随机值遵循其种子，固定种子即可精确重现这些值。

更多内容见[运行与报告](../experiments/runs.md)。

## 清理 {#clean-up}

按顶栏中的 [[ui:app.stopAll]]：它会停止 [[ui:seed.emu.demo-api.name]] 以及其他仍在运行的一切。您的信号、实验及其报告都保留在数据文件夹中。

## 接下来 {#next}

- [概念](concepts.md)：界面、信号、任务、模拟器和实验背后的理念。
- [实验](../experiments/index.md)：完整的编辑器介绍；每种节点见[节点](../experiments/nodes.md)。
- [OSC](../protocols/osc.md)、[HTTP](../protocols/http.md) 以及其他协议的页面：在把 Signal Lab 指向真实设备时阅读。
- [命令行](../automation/cli.md)：在终端或流水线中运行同一个实验。

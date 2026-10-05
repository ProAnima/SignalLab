---
title: 扫描器
description: 使用并发的 TCP 连接扫描，找出主机的哪些 TCP 端口接受连接，以及每个服务发送的问候语。
---

# 扫描器

[[ui:nav.scan]] 界面会尝试与一台主机上一个范围中的每个端口建立 TCP 连接，并列出接受连接的端口。用它来确认设备上实际有哪些服务在监听——投影机的控制端口、交换机的 Web 界面、您自己的服务本应打开的端口——以及对于先问候的服务，它们说了什么。

::: danger 负责任地使用
扫描会连接到范围中的每个端口。只扫描您拥有或获准测试的主机：在其他网络上，扫描可能触发入侵检测，而且常常违反规定。
:::

## 扫描一台主机 {#scan}

1. 输入 [[ui:sc.host]]：一台主机，一个 IP 地址或名称。
2. 设置 [[ui:sc.fromPort]] 和 [[ui:sc.toPort]]，或选择一个 [[ui:sc.preset]]。
3. 如果需要，调整 [[ui:common.concurrency]] 和 [[ui:common.timeoutMs]]。
4. 保持 [[ui:sc.grabBanner]] 开启，以读取每个服务最先说的内容。
5. 按 [[ui:sc.startScan]]。

按钮下方的一条会显示已尝试了多少个端口、共多少个，以及有多少个是开放的。开放端口在找到时出现在右侧。当每个端口都被尝试过后扫描结束；[[ui:sc.stopScan]] 会提前结束它，尚未尝试的端口不再尝试。运行期间字段是固定的。

| 字段 | 内容 | 默认值 |
| --- | --- | --- |
| [[ui:sc.host]] | 要扫描的主机。 | `127.0.0.1` |
| [[ui:sc.fromPort]]、[[ui:sc.toPort]] | 范围，两端包含在内，1–65 535。当前者更大时，两者会互换。 | 1–1024 |
| [[ui:common.concurrency]] | 同时打开的连接尝试数，1–1024。 | 400 |
| [[ui:common.timeoutMs]] | 每个连接等待多久，50–10 000 ms。 | 500 |
| [[ui:sc.grabBanner]] | 连接后，最多等待 400 ms 让服务发送一些内容，并保留其前 256 个字节。 | 开 |

超出范围的 [[ui:common.concurrency]] 或 [[ui:common.timeoutMs]] 会在扫描开始时被调整到范围内。

[[ui:sc.preset]] 会填入范围：

| 预设 | 端口 |
| --- | --- |
| [[ui:sc.preset.wellKnown]] | 1–1024 |
| [[ui:sc.preset.common]] | 1–10 000 |
| [[ui:sc.preset.osc]] | 8000–9100 |
| [[ui:sc.preset.full]] | 1–65 535 |

## 扫描需要多久 {#duration}

接受连接的端口会立即应答。不接受连接的端口可能耗费整个超时时间：丢弃连接尝试的防火墙从不应答，而在 Windows 上，即使是被拒绝也可能超过默认超时。因此，扫描一台什么都不应答的主机大约需要：

```text
ports ÷ concurrency × timeout
```

默认值下扫描完整范围：65 535 ÷ 400 × 0.5 s ≈ 82 s。提高 [[ui:common.concurrency]] 或降低 [[ui:common.timeoutMs]] 可以更快；把超时降得太低，慢速主机的开放端口会被漏掉。

## 读取结果 {#results}

[[ui:sc.openPorts]] 按端口顺序列出每个接受连接的端口：

| 列 | 内容 |
| --- | --- |
| [[ui:sc.port]] | 开放的端口。 |
| [[ui:common.time]] | 找到它的时间。 |
| [[ui:sc.banner]] | 服务最先发送的内容，换行会变成空格，或 `—`。 |

被拒绝的端口，以及未在超时内应答的端口，都会略去：扫描器不区分关闭和过滤。列表最多保留 2000 个开放端口。

只有先发言的服务才有横幅——SSH、SMTP、FTP、许多设备控制协议。Web 服务器等待请求，因此其端口显示 `—`。抓取横幅会让每个开放端口最多多花 400 ms。

扫描器打开的连接会立即再次关闭。扫描器不在其上发送任何内容。

捕获开启时，每个开放端口也会出现在[检查器](inspector.md)中，附带其横幅和判定 `open`。

## 相关内容 {#related}

- [风暴](storm.md)——对您找到的端口施加负载。
- [UDP 与 TCP](../protocols/udp-tcp.md)——与它通信。
- [疑难解答](../reference/troubleshooting.md)
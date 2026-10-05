---
layout: home
title: Signal Lab
description: Signal Lab 是面向 OSC 和网络协议的测试实验室，可以发送、捕获、模拟和劣化演出控制设备、各类设备与服务的流量，并把这些变成可重复的测试。
hero:
  name: Signal Lab
  text: OSC 与网络协议测试实验室
  tagline: 发送并观察设备和服务所用协议的流量，充当尚未到位的设备或 API，有意让网络出问题——然后在应用、脚本或 CI 中再次运行同样的检查。
  actions:
    - theme: brand
      text: 开始使用
      link: /zh/guide/
    - theme: alt
      text: 下载
      link: https://github.com/ProAnima/SignalLab/releases/latest
features:
  - title: 机柜上的每一种协议
    details: 带类型参数的 OSC、原始 UDP 与 TCP、支持 Basic、Bearer 和 Digest 认证的 HTTP、WebSocket 以及 MQTT 3.1.1——另有广播、组播、子网遍历和发现监听器。
    link: /zh/protocols/osc
  - title: 一个检查器，看清一切
    details: 工具发送和接收的每一帧都在同一条时间线上，已解码并附带字节。可以筛选、导出，还能逐字节重放捕获的帧。
    link: /zh/tools/inspector
  - title: 信号库
    details: 给有效的消息起个名字，放进文件夹，在任何地方用 Ctrl+K 再次发送——OSC、原始 UDP、HTTP 请求或 MQTT 发布都可以。
    link: /zh/tools/signals
  - title: 模拟器
    details: 扮演另一端——模拟 HTTP API、OSC/UDP/TCP 设备或 MQTT 代理——支持规则、序列、延时、故障以及按计划停机。
    link: /zh/tools/emulators
  - title: 网络损伤
    details: 在客户端与其目标之间放一个中继，加入延迟、抖动、丢包、重复、乱序或带宽限制，或者重置 TCP 连接——全部由种子决定，相同的流量会得到相同的结果。
    link: /zh/tools/impairment
  - title: 实验
    details: 可视化的测试流程——发送、等待回复、检查、分支、循环、并行运行分支——支持参数、配置和机密，每次运行都有报告。
    link: /zh/experiments/
  - title: 负载测试
    details: 让 HTTP 请求承受恒定速率、斜坡、阶梯、尖峰或随机到达的负载，测量 p50 到 p99、错误和实际达到的速率，并按阈值判定运行失败。
    link: /zh/experiments/load
  - title: 自动化
    details: 用 signallab 命令行在无界面的情况下运行实验——退出码、JUnit 报告、GitHub Action——或者通过 MCP 交给 AI 助手。
    link: /zh/automation/cli
  - title: 服务器与 API
    details: 在浏览器中使用同样的界面，在实验室电脑或 Docker 中运行同样的引擎，凭令牌登录——每个命令和每次运行都有对应的 HTTP API。
    link: /zh/server/
---

Signal Lab 可以作为桌面应用在 Windows 和 Linux 上运行，也可以作为服务器运行，在浏览器中打开。初次使用？先了解[什么是 Signal Lab](guide/index.md)，然后[安装](guide/install.md)，再跟着[快速上手](guide/first-steps.md)走一遍：收发一条消息、保存一个信号、让模拟的 API 应答，再运行一个小实验——全部在本机完成。

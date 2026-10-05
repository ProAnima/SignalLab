---
title: What is Signal Lab
description: What Signal Lab is for, what you can do with it, the two ways to run it and how it is organised.
---

# What is Signal Lab

Signal Lab is a test lab for OSC and network protocols. It sends the messages your gear
and services speak, shows you what comes back, stands in for the device or API that is
not there yet, degrades the network between them on purpose, and turns all of that into
experiments you can run again — from the app, a script or a CI pipeline.

It was built for bringing an installation up before the installation exists: the show
controller, the media server, the sensors and the cloud API can each be tested against a
stand-in, on one laptop, long before they meet on site.

## Who it is for {#audience}

- **People who bring up show control, installations and networked devices**: lighting
  and media servers, controllers, sensors, projectors, anything that speaks OSC, UDP,
  TCP or MQTT.
- **Testers of APIs and services**: HTTP and WebSocket endpoints, their authentication,
  their behaviour under load and when a dependency fails.
- **Whoever automates either**: the same experiments run headless in a pipeline, on a
  lab server next to the gear, or driven by an AI assistant.

## What you can do {#what-you-can-do}

### Send and watch traffic {#send-and-watch}

Each protocol has a screen of its own:

| Protocol | What you can do | Page |
| --- | --- | --- |
| OSC | Send messages with typed arguments, monitor a port, drive a waveform into an endpoint | [OSC](../protocols/osc.md) |
| Raw UDP and TCP | Send text or bytes in experiments and from the library, emulate devices that speak them | [UDP and TCP](../protocols/udp-tcp.md) |
| HTTP | One request with its full response, Basic, Bearer or Digest authentication, a cookie jar, a load burst | [HTTP](../protocols/http.md) |
| WebSocket | Connect with the headers and subprotocols a service expects, send text or bytes, read every message | [WebSocket](../protocols/websocket.md) |
| MQTT 3.1.1 | Connect to a broker, watch every topic it holds, publish at QoS 0, 1 or 2, clear a retained value | [MQTT](../protocols/mqtt.md) |
| Broadcast and multicast | Send to a list, a broadcast address, a multicast group or every host of a subnet; listen for who answers | [Broadcast and discovery](../protocols/broadcast.md) |

### See every frame {#inspect}

The [Inspector](../tools/inspector.md) records every frame the tools send and receive, in
one timeline: decoded, with its bytes, filterable and exportable. A frame it caught can
become a signal that replays it byte for byte.

### Keep what works {#library}

A message that worked goes into the [signal library](../tools/signals.md): named, filed
in folders, with a note on what it should make happen. You fire it again from its
screen, from the library or from anywhere with <kbd>Ctrl</kbd>+<kbd>K</kbd>.

### Play the other side {#emulate}

[Emulators](../tools/emulators.md) answer as the API, device or service your system talks
to: an HTTP API with routes and responses, an OSC, UDP or TCP device with rules, an MQTT
broker. They can answer slowly, fail in sequence, send malformed bodies or go down on a
schedule — so you can test how your system copes before the real thing is there.

### Break the network on purpose {#impair}

An [impairment relay](../tools/impairment.md) sits between a client and its target and
adds latency, jitter, loss, duplicates, reordering or a bandwidth limit to UDP, or delays,
throttles, resets and stalls TCP connections. Every decision follows a seed, so the same
traffic meets the same fate twice.

### Turn it into a test {#experiments}

An [experiment](../experiments/index.md) is a flow of steps you draw on a canvas: send a
request, wait for the reply, check it, extract a value, branch, loop, run branches in
parallel, start an emulator or an impairment relay for the run. Each run is reported
step by step and saved as a report you can compare with an earlier one.

### Put load on a service {#load}

An experiment's HTTP request can run [under load](../experiments/load.md) — a steady
rate, a ramp, steps, a spike or random arrivals — measured and judged by thresholds. The
[HTTP](../protocols/http.md) screen has a quick load burst, and [Storm](../tools/storm.md)
generates raw UDP or TCP load against your own servers and links.

### Find what is out there {#discover}

[Broadcast and discovery](../protocols/broadcast.md) finds devices that answer a probe,
and the [Scanner](../tools/scanner.md) shows which TCP ports of a host are open.

### Automate it {#automate}

The `signallab` [command line](../automation/cli.md) runs experiments without a window and
exits with a code a pipeline understands; it writes JUnit reports for [CI](../automation/ci.md).
`signallab mcp` lets an [AI assistant](../automation/mcp.md) read, write and run experiments.
A [server](../server/index.md) offers the same through its [HTTP API](../api/index.md).

## Two ways to run it {#two-ways}

| | Desktop app | Server, in a browser |
| --- | --- | --- |
| Runs on | Windows 10 and 11 (x64), Linux (x86_64) | Linux as a Docker image (amd64 and arm64), or built from the source |
| You use it | in its own window | in Chrome, Edge or Firefox, from any machine that can reach the server |
| Traffic starts from | this computer | the server |
| Comes with | the `signallab` command line | the `signallab` command line, in the image |
| Updates | itself, when you say so | with its image |

Both are the same interface on the same engine: every screen, the Inspector, experiments
and reports work the same way. A server is the choice when the gear is on a network your
own computer cannot reach — a rack PC or a small Linux box next to it, used from a
laptop or by a pipeline. See [Installing and updating](install.md) and
[The server](../server/index.md); what differs between the two is in
[Concepts](concepts.md#desktop-and-server).

## How it is organised {#organisation}

- **Screens** are tools for work you do now, by hand: send this, listen there, start
  that emulator. The sidebar lists them; each keeps what you typed and what it received
  while you switch to another. See [The window](interface.md).
- **Experiments** are flows you build once and run again, the same way every time, with
  a report for each run.
- **Libraries** keep what you made: signals in the signal library, emulators in the
  emulator library. Experiments use both.
- **Jobs** are the long-running work — a monitor, an emulator, a relay, a run — listed
  at the bottom of the window, where you stop one or all of them.

[Concepts](concepts.md) explains each of these in more depth.

## Safe by default {#defaults}

- Signal Lab sends nothing until you press a button, and only where you tell it to. The
  one exception is the desktop app's update check, once a day, which you can turn off
  (see [Updates](install.md#updates)).
- The starter signals, the starter emulators and the experiment templates all point at
  `127.0.0.1`, this computer. Reaching the network is a choice you make by typing an
  address.
- A server started without an access token listens only on `127.0.0.1` and refuses any
  other address.
- The firewall changes only when you click to allow it.

## Responsible use {#responsible-use}

::: danger Real traffic at real hosts
[Storm](../tools/storm.md), the [Scanner](../tools/scanner.md) and
[Broadcast](../protocols/broadcast.md) send real traffic to real hosts, and a broadcast
or a subnet sweep reaches every device on the segment, not just the one you had in mind.
Point them only at systems you own or are authorized to test, and check which network
you are on first. High rates can saturate links and trip intrusion detection.
:::

The engine has guard rails: a sweep reaches at most 1024 hosts, and a beacon sends at
most 50 000 packets per second across all its targets. They are guard rails, not
permission.

## Languages {#languages}

The interface, its tooltips and its error messages, the `signallab` command line, the
server's sign-in page and the Windows setup speak 11 languages: English, Русский
(Russian), Español (Spanish), Français (French), Deutsch (German), Português (Brazilian
Portuguese), 中文 (Simplified Chinese), 日本語 (Japanese), 한국어 (Korean), हिन्दी (Hindi) and
العربية (Arabic, right to left). You switch it live from the header; see
[The window](interface.md#languages).

## Next steps {#next}

1. [Install Signal Lab](install.md).
2. Find your way around [the window](interface.md).
3. Take the [first steps](first-steps.md) on this computer.
4. Read the [concepts](concepts.md) behind it.

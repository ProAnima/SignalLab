---
layout: home
title: Signal Lab
description: Signal Lab is a test lab for OSC and network protocols that sends, captures, emulates and impairs the traffic of show-control gear, devices and services, and turns it into repeatable tests.
hero:
  name: Signal Lab
  text: A test lab for OSC and network protocols
  tagline: Send and watch the traffic your gear and services speak, stand in for the device or API that is not there yet, break the network on purpose — then run the same checks again from the app, a script or CI.
  actions:
    - theme: brand
      text: Get started
      link: /guide/
    - theme: alt
      text: Download
      link: https://github.com/ProAnima/SignalLab/releases/latest
features:
  - title: Every protocol on the rack
    details: OSC with typed arguments, raw UDP and TCP, HTTP with Basic, Bearer and Digest authentication, WebSocket and MQTT 3.1.1 — plus broadcast, multicast, subnet sweeps and a discovery listener.
    link: /protocols/osc
  - title: One Inspector for everything
    details: Every frame the tools send and receive, in one timeline, decoded and with its bytes. Filter it, export it, and replay a captured frame byte for byte.
    link: /tools/inspector
  - title: A library of signals
    details: Name the message that worked, file it in a folder and fire it again from anywhere with Ctrl+K — OSC, raw UDP, an HTTP request or an MQTT publish.
    link: /tools/signals
  - title: Emulators
    details: Play the other side — a mock HTTP API, an OSC, UDP or TCP device, an MQTT broker — with rules, sequences, delays, faults and outages on a schedule.
    link: /tools/emulators
  - title: Network impairment
    details: A relay between a client and its target that adds latency, jitter, loss, duplicates, reordering or a bandwidth limit, or resets TCP connections — seeded, so the same traffic meets the same fate.
    link: /tools/impairment
  - title: Experiments
    details: Visual test flows — send, wait for the reply, check it, branch, loop, run branches in parallel — with parameters, profiles, secrets and a report for every run.
    link: /experiments/
  - title: Load testing
    details: Put an HTTP request under a constant rate, a ramp, steps, a spike or random arrivals, measure p50 to p99, errors and the rate achieved, and fail the run on thresholds.
    link: /experiments/load
  - title: Automation
    details: Run experiments headless with the signallab command line — exit codes, JUnit reports, a GitHub Action — or hand them to an AI assistant over MCP.
    link: /automation/cli
  - title: Server and API
    details: The same interface in a browser and the same engine on a lab PC or in Docker, signed in with a token — and an HTTP API for every command and run.
    link: /server/
---

Signal Lab runs as a desktop app on Windows and Linux, or as a server you open in a
browser. New to it? Read [what Signal Lab is](guide/index.md), [install it](guide/install.md),
then take the [first steps](guide/first-steps.md): a message sent and received, a signal
saved, an emulated API answering, and a small experiment run — all on this computer.

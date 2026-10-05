---
title: Scanner
description: Find which TCP ports of a host accept connections, with the greeting each service sends, using a concurrent TCP connect scan.
---

# Scanner

The [[ui:nav.scan]] screen tries to open a TCP connection to every port of a
range on one host and lists the ports that accept. Use it to confirm which
services are actually listening on a device — the control port of a projector,
the web interface of a switch, the port your own service was meant to open —
and, for services that greet first, what they say.

::: danger Responsible use
A scan connects to every port of the range. Scan only hosts you own or are
authorized to test: on other networks a scan can trip intrusion detection and
is often against the rules.
:::

## Scanning a host {#scan}

1. Enter the [[ui:sc.host]]: one host, an IP address or a name.
2. Set [[ui:sc.fromPort]] and [[ui:sc.toPort]], or pick a [[ui:sc.preset]].
3. Adjust [[ui:common.concurrency]] and [[ui:common.timeoutMs]] if you need to.
4. Leave [[ui:sc.grabBanner]] on to read what each service says first.
5. Press [[ui:sc.startScan]].

A bar under the button shows how many ports have been tried, of how many, and
how many are open. Open ports appear on the right as they are found. The scan
ends when every port has been tried; [[ui:sc.stopScan]] ends it earlier, and
ports not tried yet are not tried. The fields are fixed while it runs.

| Field | What | Default |
| --- | --- | --- |
| [[ui:sc.host]] | The host to scan. | `127.0.0.1` |
| [[ui:sc.fromPort]], [[ui:sc.toPort]] | The range, both ends included, 1–65 535. When the first is larger, they are swapped. | 1–1024 |
| [[ui:common.concurrency]] | How many connection attempts are open at the same time, 1–1024. | 400 |
| [[ui:common.timeoutMs]] | How long to wait for each connection, 50–10 000 ms. | 500 |
| [[ui:sc.grabBanner]] | After connecting, wait up to 400 ms for the service to send something, and keep its first 256 bytes. | on |

A [[ui:common.concurrency]] or [[ui:common.timeoutMs]] outside its range is
brought into it when the scan starts.

[[ui:sc.preset]] fills the range:

| Preset | Ports |
| --- | --- |
| [[ui:sc.preset.wellKnown]] | 1–1024 |
| [[ui:sc.preset.common]] | 1–10 000 |
| [[ui:sc.preset.osc]] | 8000–9100 |
| [[ui:sc.preset.full]] | 1–65 535 |

## How long a scan takes {#duration}

A port that accepts answers at once. A port that does not can cost up to the
whole timeout: a firewall that drops connection attempts never answers, and on
Windows even a refusal can take longer than the default timeout. So a scan of a
host that answers nothing takes about:

```text
ports ÷ concurrency × timeout
```

The full range at the defaults: 65 535 ÷ 400 × 0.5 s ≈ 82 s. Raise
[[ui:common.concurrency]] or lower [[ui:common.timeoutMs]] to go faster; lower
the timeout too far and a slow host's open ports are missed.

## Reading the results {#results}

[[ui:sc.openPorts]] lists every port that accepted a connection, in port order:

| Column | What |
| --- | --- |
| [[ui:sc.port]] | The open port. |
| [[ui:common.time]] | When it was found. |
| [[ui:sc.banner]] | What the service sent first, line breaks turned into spaces, or `—`. |

A port that refused, and one that did not answer within the timeout, are both
left out: the scanner does not tell closed from filtered. The list keeps up to
2000 open ports.

Only services that speak first have a banner — SSH, SMTP, FTP, many device
control protocols. A web server waits for a request, so its port shows `—`.
Grabbing banners makes each open port take up to 400 ms longer.

A connection the scanner opens is closed again at once. The scanner sends
nothing on it.

With capture on, every open port is also in the [Inspector](inspector.md), with
its banner and the verdict `open`.

## Related {#related}

- [Storm](storm.md) — load on a port you found.
- [UDP and TCP](../protocols/udp-tcp.md) — talk to it.
- [Troubleshooting](../reference/troubleshooting.md)

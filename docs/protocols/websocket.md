---
title: WebSocket
description: Connect to a ws:// or wss:// service with the headers and subprotocols it expects, send text or binary messages, and read every message as it arrives.
---

# WebSocket

The [[ui:nav.ws]] screen is a WebSocket client: it opens one connection to a
service — with the headers and subprotocols the service expects — sends text
or bytes, and lists every message that comes and goes, newest last. Use it to
try a live API, a control surface or a device that speaks WebSocket before you
script it in an experiment.

## Connecting {#connect}

1. Open [[ui:nav.ws]].
2. In [[ui:field.url]], type the address: `ws://127.0.0.1:9001/` or
   `wss://example.com/socket`.
3. If the service asks for them, enter [[ui:exp.wsProtocols]] and add
   [[ui:http.headers]] (an `Authorization` header with a token, a cookie).
4. Press [[ui:ws.connect]]. While the upgrade runs the button says
   [[ui:ws.connecting]]; once it is done, the fields lock and the button becomes
   [[ui:ws.disconnect]].

| Field | What | Default |
| --- | --- | --- |
| [[ui:field.url]] | `ws://` or `wss://`, a host, an optional port (80 for `ws`, 443 for `wss`) and a path | `ws://127.0.0.1:9001/` |
| [[ui:exp.wsProtocols]] | Subprotocols to offer, comma separated, in order of preference; the server picks one. A name has no spaces, commas or slashes. | none |
| [[ui:http.headers]] | Extra headers for the upgrade request; [[ui:http.addHeader]] adds a row. A row without a name is left out. | none |

Connecting — the name lookup, the TCP connection, TLS for `wss://` and the
upgrade — has 10 seconds. The URL is kept when you switch screens and when you
restart the app; headers and subprotocols are not.

When connected, the panel shows:

| Item | What |
| --- | --- |
| [[ui:ws.state]] | [[ui:ws.open]], or once it ends: closed by you or by the server, with the close code, or [[ui:ws.lost]] when the line broke without a close |
| [[ui:field.reason]] | The reason the closing side gave, if any |
| [[ui:ws.subprotocol]] | The subprotocol the server chose, or — |
| [[ui:ws.peer]] | The server's `IP:port` |
| [[ui:ws.upgradeTime]] | How long connecting and the upgrade took, in milliseconds |

The connection is a job: it appears in the console strip and can be stopped
there as well.

### Secure connections {#wss}

`wss://` trusts the same certificates as HTTPS on this system: a server whose
certificate this system does not trust (self-signed, expired, another name) is
refused with "A secure connection to … could not be made". There is no setting
to skip the check.

## Sending a message {#send}

1. Under [[ui:ws.message]], choose [[ui:exp.wsText]] or [[ui:exp.wsBinary]].
2. Write the message. For binary, write the bytes as pairs of hex digits:
   `de ad be ef`.
3. Press [[ui:common.send]], or <kbd>Ctrl</kbd>+<kbd>Enter</kbd> in the message.

A message is at most 16 MiB (16 777 216 bytes, counted as bytes for binary, not
as hex digits), the same limit as for a message that arrives and for the
[[ui:exp.node.ws_send]] step. A longer one is refused with `Too long: at most
16777216` before anything is sent, and the connection stays open.

When a text message is JSON, [[ui:http.formatJson]] lays it out with indents
before you send it. The message is kept when you switch screens and when you
restart the app.

## Reading the messages {#messages}

[[ui:ws.messages]] lists what was received (↓) and sent (↑), newest last, with
the time, the start of the message (300 characters) and its size; a binary
message shows its bytes in hex and is marked [[ui:ws.binary]]. Above the list
are how many were received and sent. The list follows new messages while it is
scrolled to the end; scroll up and it stays where you are.

Click a message to see it whole below the list: JSON laid out, binary as hex.
[[ui:ws.editHere]] copies it into the message field, to send again or change.
A very long message is shown in part — text up to 64 KiB, binary up to
4096 bytes — and then cannot be copied, since it would be sent cut.

The screen keeps the latest 2000 messages; [[ui:common.clear]] empties the
list. If a service sends faster than the screen can take — more than 2000 in a
tenth of a second — the oldest of those are left out of the list and counted as
"not shown". The Inspector still has them while capture is armed.

## Closing {#close}

[[ui:ws.disconnect]] sends a close frame with code 1000 (normal) and waits up
to 2 seconds for the server's answer before it hangs up. The state then reads
closed with 1000. When the server closes, the state shows its code and reason;
when the connection breaks without a close frame, it reads [[ui:ws.lost]] and
the console says why.

Signal Lab answers the server's pings by itself; pings and pongs are not
listed. A connection also ends when a message over 16 MiB arrives, or when
sending one takes longer than 10 seconds because the server stopped reading.
(Sending one over 16 MiB yourself is refused and ends nothing.)

## In the Inspector {#inspector}

With capture armed, the connection's traffic appears with the protocol `ws`
and the source `websocket`:

| Summary | What |
| --- | --- |
| `CONNECT ws://… (subprotocol)` | The connection was opened |
| `TEXT …` | A text message and its start |
| `BINARY n B …` | A binary message, its size and first 16 bytes |
| `CLOSE code reason` | A close frame, sent or received |

Each message frame keeps its bytes. While traffic is light every message is
captured; a busy connection is held to 200 frames a second, and the next frame
captured says how many were left out (`+n not shown`). See
[Inspector](../tools/inspector.md).

## In experiments {#experiments}

Four steps script a WebSocket conversation. A connection is opened by one step
and named by the others:

| Step | What it does |
| --- | --- |
| [[ui:exp.node.ws_connect]] | Opens a connection for the rest of the run. Its URL and headers take `{{templates}}`, so a token extracted earlier can go in them. [Details](../experiments/nodes.md#node-ws_connect) |
| [[ui:exp.node.ws_send]] | Sends a text or binary message on a connection. [Details](../experiments/nodes.md#node-ws_send) |
| [[ui:exp.node.wait_ws]] | Waits for a message whose payload matches, as [[ui:exp.node.wait_udp]] does; JSON messages can be read field by field afterwards. [Details](../experiments/nodes.md#node-wait_ws) |
| [[ui:exp.node.ws_close]] | Closes a connection with a close handshake: code 1000, or 3000–4999 for an application's own, and a reason of up to 123 bytes. [Details](../experiments/nodes.md#node-ws_close) |

A connection the run still has open when it ends — or is stopped — is closed
properly. The [[ui:exp.templateWsEcho]] template is a worked example.

## From the command line {#cli}

`signallab send ws` makes one exchange: connect, send a message, wait for the
answer if you ask for one, close.

```bash
signallab send ws ws://127.0.0.1:9001/ --text '{"type":"ping"}' --expect pong
```

The handshake and what was sent go to standard error, the answer to standard
output:

```text
Connected to ws://127.0.0.1:9001/ in 4 ms
Sent 15 bytes
{"type":"pong"}
```

`--hex` sends a binary message; `-H` adds a header and `--protocol` offers a
subprotocol (both repeatable). `--expect TEXT`, `--expect-regex RE` or `--wait`
(any message) say what answer to wait for, for `--timeout` milliseconds (2000
by default). It exits with 1 when the answer does not come or the connection
fails. See [Command line](../automation/cli.md#cli-send-ws).

## Problems {#troubleshooting}

| What you see | Usual cause |
| --- | --- |
| `… is not a WebSocket address` | The URL does not start with `ws://` or `wss://`, or has no host. |
| `… answered HTTP n instead of switching to WebSocket` | The server refused the upgrade: a wrong path (404), a missing or wrong token (401, 403). The start of its answer is under the technical details. |
| `… did not take any of the subprotocols offered` | You offered subprotocols and the server chose none of them, or it answered with one you did not offer. |
| `… is not a subprotocol name` | A name with a space, a comma or a slash. |
| `The header … cannot be sent with the upgrade` | A header name or value with characters HTTP does not allow. |
| `… refused the connection` | Nothing listens on that port. |
| `A secure connection to … could not be made` | The certificate is not trusted here, or TLS failed. See [Secure connections](#wss). |
| `The connection with … broke: the server did not keep to the WebSocket protocol` | The server sent something that is not valid WebSocket. |
| `A WebSocket message is limited to … bytes` | The server sent a message over 16 MiB, which ends the connection. |
| `Too long: at most 16777216` | The message you tried to send is over 16 MiB. Nothing was sent; the connection is open. |

Every error message is listed in [Error messages](../reference/errors.md#ws).

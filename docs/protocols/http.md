---
title: HTTP
description: Send one HTTP request and read the whole response, authenticate with Basic, Bearer or Digest, keep cookies, and measure an endpoint under a concurrent load burst.
---

# HTTP

The [[ui:nav.http]] screen is a request inspector and a load tool in one:

- send a single request and see the status, the time it took, the headers and
  the body;
- authenticate with Basic, a Bearer token or Digest;
- keep the cookies a server sets, as a browser does;
- send the same request many times at once — a [[ui:http.burst]] — and read
  the throughput and the latency percentiles.

## Sending a request {#request}

1. Open [[ui:nav.http]].
2. Choose the method and type the URL, for example `http://127.0.0.1:8080/health`.
3. Add [[ui:http.headers]] if the server needs them; [[ui:http.addHeader]] adds
   a row, ✕ removes one. A row without a name is not sent.
4. For a method other than GET and HEAD, write the [[ui:http.body]]. The text stays in the field while you switch to GET or HEAD and comes back with the other method, but is not sent meanwhile.
5. Press [[ui:common.send]].

The line under the buttons gives the verdict at once — status, time and size,
or why there was no response — and the [[ui:http.response]] panel shows the
rest. The request (method, URL, headers, body, timeout and
[[ui:http.keepCookies]]) is kept when you switch screens and when you restart
the app; credentials are not.

| Key | Where | Does |
| --- | --- | --- |
| <kbd>Enter</kbd> | the URL, a header, a credential, the timeout | sends |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | any field of the request, the body included | sends |
| <kbd>Ctrl</kbd>+<kbd>S</kbd> | any field of the request | saves it as a signal ([below](#library)) |

### Request fields {#request-fields}

| Field | What | Default |
| --- | --- | --- |
| [[ui:exp.method]] | GET, POST, PUT, PATCH, DELETE, HEAD or OPTIONS | GET |
| [[ui:field.url]] | An `http://` or `https://` URL | `http://127.0.0.1:8080/` |
| [[ui:http.headers]] | Name and value pairs, sent as written. Without a `User-Agent` of your own, Signal Lab sends `SignalLab/0.1`. | `Accept: application/json` |
| [[ui:field.auth]] | How the request authenticates ([below](#auth)) | [[ui:http.auth.none]] |
| [[ui:http.keepCookies]] | Send back the cookies servers set ([below](#cookies)) | on |
| [[ui:http.body]] | Sent exactly as written; no `Content-Type` is added, so add the header that matches. An empty body is not sent. The field is not shown for GET and HEAD, and then the request carries no body at all — not in the send, not in a saved signal, not in [[ui:common.toExperiment]] — even if you typed one under another method. | empty |
| [[ui:common.timeoutMs]] | How long the whole exchange may take, answer and body included | 10 000 |

## Authentication {#auth}

| [[ui:field.auth]] | Fields | What is sent |
| --- | --- | --- |
| [[ui:http.auth.none]] | — | no `Authorization` header |
| [[ui:http.auth.basic]] | [[ui:field.username]], [[ui:field.password]] | `Authorization: Basic …`, the name and password in base64, with the first request |
| [[ui:http.auth.bearer]] | [[ui:field.token]] | `Authorization: Bearer <token>` |
| [[ui:http.auth.digest]] | [[ui:field.username]], [[ui:field.password]] | nothing at first; the answer to the server's challenge ([below](#digest)) |

Switching between Basic and Digest keeps the name and password.

### Digest {#digest}

With Digest, Signal Lab sends the request without credentials. When the server
answers `401` with a Digest challenge, Signal Lab works out the answer from the
challenge and your password and sends the request again. The response you see
is the one to that second request, marked [[ui:http.digestAnswered]], and the
latency counts both exchanges — what a client waits.

- Algorithms: MD5 and SHA-256, and their `-sess` variants. When a server offers
  both, SHA-256 is used.
- Quality of protection: `auth` and `auth-int`, and the older answer without
  `qop`.
- When the server says the nonce ran out (`stale`), or asks again with a new
  one, the request is answered again, up to 3 more times. When it refuses the
  answer to the nonce it gave last, the `401` stands: the name or the password
  is wrong.
- Redirects are followed by Signal Lab itself, so the URL that asks is the one
  answered. A challenge from another origin is not answered: credentials typed
  for one host go to no other. Moving from `http://` to `https://` on the same
  host and default ports counts as the same host.

When the challenge cannot be answered, the `401` stands and the panel says why:

| Message | Meaning |
| --- | --- |
| The server answered 401 without asking for Digest | The server wants another scheme; try Basic or Bearer. |
| The server asked for Digest with … | An algorithm Signal Lab does not speak; it speaks MD5 and SHA-256. |
| The server asked for Digest without a realm or nonce | The server's challenge is incomplete. |
| The request was sent on to …, which asked for Digest | A redirect led to another origin, whose challenge is not answered. |

### Where the credentials go {#credentials}

Credentials go only into the request's `Authorization` header as it is sent.
The Inspector, the console and experiment reports never show that header. On
this screen they are kept in memory only and are gone after a restart — unless
the request is tied to a saved signal, which brings them back.

::: warning
A request saved as a signal keeps its credentials in the library file,
`signals.json`, as plain text. In an experiment, write a password as
`{{secret.NAME}}` instead; see [Data and templates](../experiments/data.md).
:::

## Cookies {#cookies}

With [[ui:http.keepCookies]] on, what a server sets with `Set-Cookie` is kept
in the screen's cookie jar and sent back with later requests to that server,
following the browser rules (domain, path, `Secure`, expiry). The jar is used
by this screen's requests, its burst and the HTTP signals you fire from the
library. Turn it off to send requests with no cookies and keep none.

The cookies panel under the request and the response lists what the jar holds:
[[ui:http.cookieName]], [[ui:http.cookieValue]], [[ui:http.cookieWhere]]
(a domain starting with `.` also covers its subdomains),
[[ui:http.cookieExpires]] ([[ui:http.cookieSession]] for a cookie with no
expiry) and [[ui:http.cookieFlags]] (`Secure`, `HttpOnly`, `SameSite`).
Expired cookies are not listed. [[ui:common.clear]] empties the jar.

The jar lives as long as the app: a restart starts with an empty one. On a
[server](../server/index.md), there is one jar for every page signed in to it.
An experiment run has a jar of its own (see [Experiments](../experiments/index.md)),
and `signallab send http` uses none.

## The response {#response}

| Part | What |
| --- | --- |
| [[ui:http.status]] | The status code and its reason; `ERR` when no response came |
| [[ui:http.latency]] | From sending to the last byte of the body, in milliseconds |
| [[ui:http.size]] | The size of the body |
| Response headers | Click the line with their count to show or hide them |
| Body | Formatted when it is JSON; [[ui:http.rawBody]] and [[ui:http.formatJson]] switch. Up to 256 KiB is shown, then `… (truncated)`. |

When there is no response, the panel says why, in the same words as everywhere
in Signal Lab: refused, no answer in time, the name does not resolve, a
certificate problem and so on. The technical detail from the system is folded
under it.

### Redirects {#redirects}

Redirects (301, 302, 303, 307, 308) are followed, up to 10; the response shown
is the last one. After 301, 302 and 303 the request goes on as GET without a
body (HEAD stays HEAD); after 307 and 308 as it was. `Authorization` and
cookies typed for one host are not sent on to another.

### Secure connections {#tls}

An `https://` server's certificate is checked against the certificates this
system trusts. A self-signed or expired certificate is refused with
"A secure connection to … could not be made"; there is no setting to skip the
check. To test a server with your own certificate, add it to the system's
trusted certificates.

## Load burst {#burst}

The [[ui:http.burst]] sends the request on screen — with its authentication and,
while [[ui:http.keepCookies]] is on, the cookie jar — many times, and measures
it.

1. Set [[ui:common.concurrency]], [[ui:http.total]], [[ui:http.duration]] and
   [[ui:http.rate]].
2. Press [[ui:http.startBurst]]. The burst is a job: [[ui:http.stopBurst]], or
   stopping it in the console strip, ends it.

| Field | What | Default |
| --- | --- | --- |
| [[ui:common.concurrency]] | Requests in flight at once, 1–512 | 20 |
| [[ui:http.total]] | Requests to send; 0 — keep sending until the duration ends | 500 |
| [[ui:http.duration]] | Seconds to run; 0 — stop when the total is sent | 0 |
| [[ui:http.rate]] | Requests started per second, 0.1–100 000; 0 — as fast as the workers go | 0 |

With both [[ui:http.total]] and [[ui:http.duration]] at 0, the burst runs until
you stop it.

There are two ways to send:

- **[[ui:http.rate]] 0.** Each of the workers sends again as soon as it has an
  answer. This finds how much the server takes, but a slow server also slows
  the burst down.
- **A rate.** The requests start on a fixed schedule — at 10 per second, one
  every 100 ms from the start — however slow the answers. A request whose
  moment comes while every worker is busy waits at most 50 ms for one; after
  that it is skipped and counted as [[ui:http.missed]], never sent late. Missed
  requests mean the concurrency is too low for this rate, or the server is
  slower than the rate needs.

| Number | What |
| --- | --- |
| [[ui:http.sent]] | Requests that have had an answer or failed |
| [[ui:http.ok]] | Answered with a 2xx status |
| [[ui:http.failed]] | No answer, or any status outside 200–299 |
| [[ui:http.missed]] | Skipped, as above (only with a rate) |
| [[ui:http.rps]] | Requests per second over the last tenth of a second; when the burst has ended, over the whole burst. With a rate, the label names the rate asked for. |
| [[ui:http.p50]], [[ui:http.p90]], [[ui:http.p95]], [[ui:http.p99]] | The time within which that share of the requests finished, failures included; accurate to within 0.5 % |
| [[ui:http.avg]], [[ui:http.min]], [[ui:http.max]] | The mean, fastest and slowest |

The numbers are updated about 10 times a second. The chart beside them draws
the requests per second over the last 24 seconds or so.

With Digest, the first request's challenge is answered once and that answer
serves every request of the burst.

::: warning
A burst is real load. Aim it only at servers you own or are allowed to test.
:::

For ramps, steps, spikes and pass/fail thresholds, run the request under
[load in an experiment](../experiments/load.md).

## In the Inspector {#inspector}

With capture armed, each exchange appears as one frame with the protocol
`http` and the source `http`: the method, URL, status and time in the summary,
the response headers and the start of the body (2000 characters) in its detail,
the status as its verdict (`failed` when no response came, `· digest after 401`
when a challenge was answered). The frame records the size of the body, not its
bytes. The request's `Authorization` header is never in it. A burst puts at
most one exchange every 100 ms into the capture. See
[Inspector](../tools/inspector.md).

## Saving and reusing {#library}

- **Save as a signal.** [[ui:sig.saveNew]] keeps the request — method, URL,
  headers, body, timeout and authentication — in the signal library. The screen
  stays tied to it: [[ui:sig.save]] (<kbd>Ctrl</kbd>+<kbd>S</kbd>) updates it,
  [[ui:sig.saveAs]] copies it, the chip opens it in [[ui:nav.signals]]. Opening
  an HTTP signal from the library loads it back here, credentials included.
  See [Signals](../tools/signals.md).
- **Add to an experiment.** [[ui:common.toExperiment]] adds an
  [HTTP request](../experiments/nodes.md#node-http) step with the same request
  to the open experiment, right before End or after the selected step, and opens
  it.
- **Mock this.** Below a response, [[ui:http.mockThis]] makes an emulator route
  that answers this method and path with this status, headers and body. Choose
  an HTTP emulator in [[ui:http.mockInto]], or [[ui:http.mockNew]], and press
  [[ui:http.mockAdd]]; the route goes first in that emulator, and the
  [[ui:nav.emulators]] screen opens on it. See [Emulators](../tools/emulators.md).

## In experiments {#experiments}

| Step | What it does |
| --- | --- |
| [[ui:exp.node.http]] | Sends a request; its URL, headers, body and credentials take `{{templates}}`. It can run [under load](../experiments/load.md). [Details](../experiments/nodes.md#node-http) |
| [[ui:exp.node.assert_status]], [[ui:exp.node.assert_body]], [[ui:exp.node.assert_header]], [[ui:exp.node.assert_latency]] | Check the latest response. [Details](../experiments/nodes.md#node-assert_status) |
| [[ui:exp.node.extract]] | Keeps a JSON field, a header, the status, the body or a regular expression's match as a variable. [Details](../experiments/nodes.md#node-extract) |
| [[ui:exp.node.branch_status]] | Goes on through Yes or No by the status. [Details](../experiments/nodes.md#node-branch_status) |
| [[ui:exp.node.wait_http]] | Waits for a request to arrive — from the system you test — at the run's own listener or emulator. [Details](../experiments/nodes.md#node-wait_http) |
| [[ui:exp.node.emulator]] | An HTTP API that answers by routes for the whole run. [Details](../experiments/nodes.md#node-emulator) |

## From the command line {#cli}

`signallab send http` sends one request, as this screen does:

```bash
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab send http POST http://127.0.0.1:8080/api/items \
  -H 'Content-Type: application/json' --body '{"name":"lamp"}'
signallab send http GET http://127.0.0.1:8080/private -u admin:secret --digest
```

The status line goes to standard error and the body to standard output:

```text
HTTP 200 OK · 3 ms · 15 B
{"status":"ok"}
```

| Option | What | Default |
| --- | --- | --- |
| `-H`, `--header 'Name: value'` | A header; repeat for more | — |
| `--body TEXT`, `--body @FILE` | The body, or a file's contents | — |
| `--expect-status N` | Exit 1 unless the status is N | — |
| `--timeout MS` | How long to wait for the response | 10 000 |
| `-u`, `--user NAME:PASSWORD` | Basic authentication | — |
| `--digest` | With `--user`: answer the server's Digest challenge instead | — |
| `--bearer TOKEN` | `Authorization: Bearer TOKEN` | — |
| `--json` | Print the whole response as JSON on standard output | — |

It exits with 0 when a response came (and had the expected status), 1 when
none came, the status was not the one expected or a Digest challenge could not
be answered, and 2 when an option is invalid. It keeps no cookies. See
[Command line](../automation/cli.md#cli-send-http).

## Problems {#troubleshooting}

| What you see | Usual cause |
| --- | --- |
| `… refused the connection — nothing is listening on that port` | The server is not running, or listens on another port or address. |
| `No answer from … in time` | The server is slow or unreachable; check the address, or raise [[ui:common.timeoutMs]]. |
| `Cannot resolve …` | The host name does not resolve on this machine — a typo, or a name only another network knows. |
| `A secure connection to … could not be made` | The certificate is not trusted here (self-signed, expired, another name), or TLS failed. See [Secure connections](#tls). |
| `… is not a valid address` | The URL is malformed or does not start with `http://` or `https://`. |
| The server says the body is missing or of the wrong type | No `Content-Type` header that matches the body, or an empty body. |
| `401` with Digest | Read the message under the status: see [Digest](#digest). |
| [[ui:http.missed]] above 0 | Raise [[ui:common.concurrency]], or lower the rate: the server answers slower than the rate needs. |
| [[ui:http.failed]] high though the server answers | Every status outside 200–299 counts as failed, 404 and 500 included. |

On a server, requests go out from the server: `127.0.0.1` is the server itself.
See [Server](../server/index.md).

Every error message is listed in [Error messages](../reference/errors.md#transport).

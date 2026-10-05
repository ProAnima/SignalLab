---
title: Server security
description: Who can use a Signal Lab server and how it keeps others out — tokens, sessions, host and origin checks, secrets — and what Signal Lab sends to the outside world.
---

# Server security

A Signal Lab server sends real traffic from the machine it runs on: OSC, UDP,
HTTP, MQTT, storms, scans, broadcasts. Whoever can use it can do all of that
from that machine, so the server is closed by default and opens only with a
token.

::: warning
Treat the access token like a password to the machine's network. Anyone who
has it can send traffic from the server to anything the server reaches.
:::

## At a glance {#summary}

- **No token, this machine only.** Without a token the server listens on
  loopback and answers only to loopback host names. On any other address it
  refuses to start.
- **A token for everyone else.** Browsers sign in once and get a session
  cookie; scripts send the token with every request.
- **Only its own pages.** Requests that change something, and the event
  WebSocket, must come from the server's own origin; commands take JSON only.
- **Only its own names.** A host name the server does not answer to is refused,
  which stops DNS rebinding.
- **Secrets stay inside.** Read-only, from the environment or files, never
  returned, masked wherever they would show.
- **Nothing beyond its job.** It never changes the host's firewall, serves no
  file outside its data folder, and needs no privileges.

## Without a token: this machine only {#loopback}

Started without a token, the server listens on `127.0.0.1:1430` and needs no
sign-in: it is a tool for the person at this machine. To keep a web page in
any browser on this machine from reaching it through a name that resolves to
`127.0.0.1` (DNS rebinding), it answers only requests whose `Host` is a
loopback name — `localhost`, a name ending in `.localhost`, `127.x.x.x` or
`[::1]` — or a name you allow with `--allowed-host`.

Asked to listen on any other address without a token, it does not start: it
says why and exits with code `2`.

## The token {#token}

A token has at least 24 characters, without spaces or line breaks.
`signal-lab-server token` prints a random one of 64 hexadecimal characters, and
`--generate-token` (on in the image) makes one on the first start, keeps it in
the data folder readable by the server's own user only, and prints it once. See
[The access token](index.md#token) for every way to give one.

Comparing a token takes the same time wherever it differs, and a wrong token
costs a one-second wait and a warning in the log — guessing is slow and leaves
a trace.

## Browsers: sessions {#sessions}

A browser that is not signed in is sent to the sign-in page. Its token is
exchanged once for a session, held in a cookie that is:

- `HttpOnly` — no script in a page can read it;
- `SameSite=Strict` — no other site's page can make the browser send it;
- valid for 7 days;
- `Secure` with `--secure-cookie`, so it travels over HTTPS only (set it behind
  an HTTPS proxy).

Sessions live in the server's memory: a restart signs everyone out, and
[[ui:app.signOut]] ends one at once. At most 1024 sessions are kept; the oldest
goes first.

## Scripts: the bearer token {#bearer}

A script, `signallab --server` and CI send the token with every request:

```http
Authorization: Bearer <token>
```

Every endpoint needs the token or a session, except `GET /api/health` (whether
the server answers, its version, whether it asks for a token) and the sign-in
page. A request to the API without one gets `401` with the error `auth.required`;
a page gets the sign-in page.

## Host names {#hosts}

| Server started | Host names it answers to |
| --- | --- |
| without a token | loopback names, and the names in `--allowed-host` |
| with a token, without `--allowed-host` | any name |
| with a token and `--allowed-host` | loopback names, and the names in `--allowed-host` |

`--allowed-host` (`SIGNALLAB_ALLOWED_HOSTS`) takes names separated by commas,
compared without the port and ignoring case:

```bash
signal-lab-server --listen 0.0.0.0:1430 --token-file token.txt --allowed-host lab-pc.example.com,192.0.2.10
```

Any other `Host` gets `403` with the error `auth.host`. Set it on a server that
is reachable under known names, so a page on another site cannot reach it
through a name of its own.

## Origin and content type {#origin}

- Every request that changes something (anything but `GET` and `HEAD`), and the
  WebSocket of events, must carry no `Origin`, or the server's own — the same
  host and port as its `Host`. A page on another site, or one that sends
  `Origin: null`, gets `403` with `auth.origin`. Scripts and `curl` send no
  `Origin` and are not affected.
- Commands take `Content-Type: application/json` only (`415` with
  `command.json_required` otherwise), so a form on another site cannot send one.
- The server answers no cross-origin (CORS) requests.

## Response headers {#headers}

Every response carries:

| Header | Value |
| --- | --- |
| `Content-Security-Policy` | `default-src 'self'; connect-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; object-src 'none'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'` |
| `X-Content-Type-Options` | `nosniff` |
| `X-Frame-Options` | `DENY` |
| `Referrer-Policy` | `same-origin` |
| `Cache-Control` | `no-store` for the API and the sign-in page |

The interface loads nothing but what the server serves, talks to nothing but
the server, and cannot be framed by another page.

## Files and sizes {#files}

- A download (`GET /api/files?path=…`) is served only from inside the data
  folder, at most 256 MiB; anything else is `404`.
- A request body is at most 24 MiB.

## Secrets {#secrets}

A secret's value never leaves the engine:

- On a server, values are **read-only**: the environment variable
  `SIGNALLAB_SECRET_<NAME>`, or the file `<NAME>` in the secrets folder
  (`/run/secrets/signallab` by default). Setting or removing one from the
  browser is refused (`secret.read_only`), so a value typed into a page never
  ends up stored somewhere weaker. See [Secrets](index.md#secrets).
- No command returns a value; the interface learns only whether a name is set.
- Experiments name secrets as `{{secret.NAME}}`. While a run or a send uses
  them, every text it reports — steps, errors, the run report — shows `••••` in
  their place, and frames in the [[ui:dock.inspector]] are masked byte for byte.
- An HTTP node's credentials become an `Authorization` header only as the
  request is sent; steps, frames and reports carry the response, never that
  header.

## What is logged {#audit}

The log records, with the client's address: every job started — storms, scans,
broadcasts, monitors, generators, runs, emulators — every sign-in, and every
sign-in with a wrong token. See [Logs](index.md#logs).

## What the server never does {#never}

- **Change the host's firewall.** The desktop app can add a firewall rule when
  you ask it to; on a server that command is refused (`firewall.server`). The
  host's firewall belongs to whoever runs the host. (The one-command install
  script offers to open the server's port in ufw or firewalld, and asks first —
  see [The host's firewall](index.md#firewall).)
- **Start reachable without a token**, on any address but loopback.
- **Serve a file from outside its data folder.**
- **Store a secret** typed into a browser.
- **Speak TLS** itself: put an HTTPS proxy in front of it (see
  [Behind an HTTPS proxy](index.md#https)).

Every limit of the engine — at most 1024 hosts in a sweep, at most 50 000
packets a second from a beacon — holds on a server as in the app. They are
guard rails, not permission: send traffic only to systems you own or may test.

## The container {#container}

The image runs as an unprivileged user (uid and gid 10001) and writes only to
`/data`. It runs unchanged with a read-only root filesystem, no capabilities
and `no-new-privileges` — as the install script and `deploy/compose.yaml`
start it. Each image is published with an SBOM, build provenance and a signed
GitHub attestation:

```bash
gh attestation verify oci://ghcr.io/proanima/signallab:[[version]] -R ProAnima/SignalLab
```

## What Signal Lab sends to the outside world {#outside}

Besides the traffic you send, Signal Lab talks to two places, both the studio's.

### Update checks {#update-check}

Only the **desktop app** looks for updates; a server and a browser never do.
With [[ui:update.auto]] on (in [[ui:about.open]]), once a day, and whenever you
press [[ui:update.check]], the app asks the studio's hub
(`hub.proanima.net`) — and GitHub's latest release only when the hub cannot be
reached. The question carries:

- the app's version;
- the operating system and the processor architecture;
- a random number of this install (`X-Install-Id`), made once and kept with the
  app's settings, so a new release can reach a share of installs first. It says
  nothing about you or the computer.

Only published releases are offered. A download whose signature does not match
the key built into the app is not installed, and nothing is installed until you
press [[ui:update.install]].

### Feedback {#feedback}

[[ui:feedback.open]] (the ✉ in the header, also in [[ui:about.open]]) sends a message to the
developers through the studio's hub, which mails it on; the app holds no
password for it. It sends only what the form shows: your message, your e-mail if
you give one, the screenshots you add, and — under [[ui:feedback.logs]] — the
console log and [[ui:feedback.systemInfo]], each of which you can open before
sending and untick. This computer's name, its address and your folders are left
out of them. From a browser, the server sends the form.

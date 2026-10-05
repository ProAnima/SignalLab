---
title: Troubleshooting
description: Common problems with Signal Lab and how to fix them — nothing arrives, a port in use, the firewall, Docker networking, the server, MQTT, certificates, updates and logs.
---

# Troubleshooting

Every failure Signal Lab reports has a code; the message for each is in
[error messages](errors.md). Network failures are the
[`transport`](errors.md#transport) codes: `refused`, `timeout`, `dns`,
`unreachable`, `reset`, `address_in_use`, `address_unavailable`, `denied`,
`tls`, `target_invalid`, `failed`. The problems below are the common ones.

## Nothing arrives {#nothing-arrives}

First find out whether anything reaches Signal Lab at all: open the
[[ui:dock.inspector]] tab in the bottom panel and press [[ui:ins.arm]]. Every
datagram, request and message a tool sends or receives is listed there, with
where it came from.

### The listening address {#listen-address}

- A [[ui:common.bind]] of `0.0.0.0:<port>` listens on every network card;
  `127.0.0.1:<port>` hears only this machine. Gear on the network needs the
  first.
- The device must send to this machine's address and to the port you listen
  on. The header shows this machine's name and address.
- An address that is not this machine's fails with `address_unavailable`.

### The firewall {#firewall}

Traffic on `127.0.0.1` is never filtered, which is why a test on one machine
works while the same test from another machine gets nothing.

**Windows.** Windows Firewall decides per program. Windows usually asks once,
the first time a program listens — and a *Cancel* there leaves a rule that
blocks it, which wins over any allow rule. On a network Windows calls public
(a venue's Wi-Fi, often), it may not ask at all.

- The desktop app looks at the firewall once, when a monitor, a discovery
  listener, a relay, a run or an emulator starts listening. When the firewall
  is in the way it says so in a notice with [[ui:fw.allow]] — or
  [[ui:fw.allowPublic]] on a public network. Windows asks for administrator
  rights, then the program's inbound rules, a block rule included, are
  replaced by one allow rule. [[ui:fw.dismiss]] hides the notice.
- From a terminal: `signallab doctor` shows what stands in the way, and
  `signallab firewall allow` fixes it (`--public` for public networks too),
  with the same administrator prompt.
- A setup (`.exe`) installed *for everyone* adds the allow rules itself
  (private and domain networks), unless it was run with `/NOFIREWALL`. A setup
  *for me* cannot, and the `.msi` leaves the firewall to whoever deploys it.
- A server never changes its host's firewall: its administrator opens the
  ports (the install script offers to, with ufw or firewalld).

**Linux.** A firewall such as ufw or firewalld works by port, not by program.
`signallab doctor` names the one that is on and how to open a port, for
example `sudo ufw allow 9000/udp`.

### Broadcast and multicast {#broadcast-multicast}

- Routers do not forward broadcast: `255.255.255.255` and `x.x.x.255` reach
  only the network segment the sending card is on. With several cards, put the
  card's address in [[ui:bc.bindSource]] (under [[ui:bc.socketOptions]]), for
  example `10.0.0.5:0`.
- A multicast datagram reaches only listeners that joined its group — on the
  discovery listener, [[ui:bc.joinGroups]]. With [[ui:bc.ttl]] at 1, the
  default, it stays on this network.
- To hear your own multicast on the same machine, leave
  [[ui:bc.mcastLoop]] on.

### A device that does not answer {#no-answer}

UDP has no delivery receipt: a datagram sent to a port nobody listens on still
counts as sent. Windows then reports the ICMP *port unreachable* it got back
as a connection reset on that socket's next receive; Signal Lab's monitors,
listeners, relays and waits ignore it and go on listening. So when a reply
does not come, a wait fails with `wait.timeout` after its time, not with an
error about the send. Check in the [[ui:dock.inspector]] that the message went
out to the right address, then check the device.

## A send is refused before it goes out {#refused-send}

These are checked first, and nothing is sent when one fails:

| Code | Why | Fix |
| --- | --- | --- |
| `transport.target_invalid` | The destination has no port, or is neither `IP:port` nor `host:port` | Write both, such as `192.0.2.20:9000` |
| `transport.dns` | The host name does not resolve on this machine | Check the name, or use the address. A name with an IPv4 address is reached over IPv4, so `localhost:9000` finds a receiver on `127.0.0.1` |
| `node.osc_address` | An OSC address does not start with `/` — in the sender, the generator, a signal or a step | Start it with `/`, such as `/cue/go` |
| `node.topic_wildcard` | The topic of an MQTT publish has a `+` or `#`, on the screen's connection as everywhere else | Publish to one topic; wildcards are for subscribing |
| `node.too_long` on a WebSocket message | The message is over 16 MiB | Send less; the connection stays open |

## A port is already in use {#port-in-use}

`transport.address_in_use`: another program — or another job of Signal Lab —
already listens on that port.

- **A monitor and a run.** A run opens the ports of its waits, emulators and
  relays before its first step, so a port held by a
  [[ui:osc.monitor]], a discovery listener or an emulator job makes the run
  fail before it starts. Stop that job first; [[ui:app.stopAll]] stops every
  one.
- **Two emulators** in a run cannot share a port of one transport: HTTP, MQTT
  and TCP emulators all listen on TCP, OSC and UDP ones on UDP
  (`emulator.bind_taken`).
- **Listening beside the real service.** The discovery listener can share a
  port with a program that already holds it: leave [[ui:bc.reuse]] on. On
  Linux, that program must share its port too. Without it, a taken port is
  `broadcast.port_shared`.
- **Just stopped.** A port a stopped emulator or run held is released a moment
  later; an emulator started again right away waits for it briefly.
- **Ports below 1024** on Linux need administrator rights (`denied`). The
  server image runs without any, so use a port of 1024 or above.

## The server in Docker does not reach the network {#docker-network}

Broadcast, multicast and discovery reach the physical network only with host
networking — `network_mode: host` in the compose file, or
`docker run --network host` — and only on a Linux host. It also lets
monitors, waits and emulators listen on the host's own ports. With Docker's
default bridge network, the container is on a network of its own: broadcast
and multicast never leave it, and only the ports you publish reach it.

On Windows and macOS, Docker's host networking does not reach the physical
network: use the desktop app on Windows, or run the server on a Linux host.

## SmartScreen warns about the installer {#smartscreen}

The installers are not signed yet, so Windows SmartScreen says it does not know
the publisher. Choose *More info*, then *Run anyway*. Download installers only
from the project's releases on GitHub.

## The server does not start {#server-start}

`signal-lab-server` checks its settings before it listens and exits with a
message on its error output:

| Exit code | Message | Fix |
| --- | --- | --- |
| 2 | refusing to listen on … without a token | A server others can reach needs a token: `--token-file` or `SIGNALLAB_TOKEN` (make one with `signal-lab-server token`), or `--generate-token`. Or listen on `127.0.0.1` |
| 2 | the token has N characters; it needs at least 24 | Use a longer token |
| 2 | the token must not contain spaces or line breaks | A token file may end in a line break; nothing else |
| 2 | give the token once | Use `--token`/`SIGNALLAB_TOKEN` or `--token-file`/`SIGNALLAB_TOKEN_FILE`, not both |
| 2 | --generate-token keeps the token in the data folder | Set `--data-dir` or `SIGNALLAB_DATA_DIR` |
| 2 | … it does not hold a valid token; remove it to have a new one made | The data folder's `token` file is damaged |
| 2 | cannot read the token file … | The file `--token-file` names is missing or not readable by this user |
| 2 | cannot save the new token in … | `--generate-token` could not write `token` in the data folder: make the folder writable by this user |
| 1 | cannot listen on … | The address is not this machine's, or the port is taken |
| 1 | the data folder … must be writable by this user | In Docker, a bind-mounted folder must be writable by uid 10001 |

A token that `--generate-token` made is printed once, on the first start
(`docker logs signallab` shows it), and kept in `token` in the data folder:
`docker exec signallab cat /data/token`. See [the server](../server/index.md).

## Cannot sign in to the server {#sign-in}

| What you see | Why | Fix |
| --- | --- | --- |
| *That token is not right.* | A wrong token | Copy it again from where the server keeps it; the answer takes one second on purpose |
| `auth.host` | The server does not answer to the name in the address bar | Open it by a name it accepts. Loopback names (`localhost`, `127.x.x.x`, `[::1]`) always pass. Without a token the server answers only to those and the names of `--allowed-host`; with a token, to any name unless `--allowed-host` narrows it |
| `auth.origin` | A request came from a page of another origin | Behind a reverse proxy, pass the browser's `Host` on to the server (nginx: `proxy_set_header Host $host;`), so that `Origin` and `Host` agree |
| Back at the sign-in page after signing in | The browser did not keep the session cookie | With `--secure-cookie`, the server must be reached over HTTPS |
| Signed out after a while | Sessions last 7 days and end when the server restarts; past 1024 sessions, the oldest goes | Sign in again |

## The connection to the server keeps dropping {#connection-lost}

[[ui:app.connectionLost]] means the page's event socket, `/api/events`, closed.
The page reconnects on its own, after half a second and then less often, up to
every 15 s. What happened meanwhile is not replayed: running jobs report again
as they go on. Behind a reverse proxy, make sure it passes WebSocket upgrades
for `/api/events` and does not close quiet connections in under 20 s (the
server pings every 20 s). A page that says the server has no such address
(`api.not_found`) is older than the server: reload it.

## MQTT does not connect {#mqtt}

Signal Lab speaks MQTT 3.1.1 over plain TCP. It connects and waits for the
broker's answer (CONNACK) before it reports success, so the reason is on the
button that connects:

| Code | Why | Fix |
| --- | --- | --- |
| `transport.refused` | Nothing listens on that port | Check the port: 1883 is the usual one |
| `transport.timeout` | No TCP connection within 6 s | Check the address, the network, the broker's firewall |
| `transport.dns` | The host name does not resolve | Check the name, or use the address |
| `transport.unreachable` | No route to the broker | Check the network and the address |
| `transport.reset` | The broker closed the connection at once | Often a TLS port (8883) — Signal Lab does not speak MQTT over TLS |
| `mqtt.no_answer` | The port is open, but no CONNACK came within 6 s | Often a WebSocket port — Signal Lab does not speak MQTT over WebSocket |
| `mqtt.protocol` | What answered is not an MQTT broker | Check the port |
| `mqtt.refused_protocol` | The broker does not accept MQTT 3.1.1 | Enable 3.1.1 on the broker |
| `mqtt.refused_client_id` | The broker refuses the client id | Use another client id |
| `mqtt.refused_unavailable` | The broker is unavailable | Try again later |
| `mqtt.refused_credentials` | The user name or password is wrong | Check them |
| `mqtt.refused_not_authorized` | The user may not connect | Check the broker's access rules |
| `mqtt.client_id_required` | The client id is empty | Fill it in |

A broker drops the older of two connections with the same client id: when a
connection keeps closing, look for another client using the same id.

## A certificate is not trusted {#tls}

`transport.tls`, on `https://` or `wss://`: the server's
certificate is not trusted, does not name the host you asked for, or TLS could
not be agreed. Signal Lab checks certificates the way the system does and has
no switch to skip the check. HTTPS and WSS trust the same certificates: those
of the operating system where the engine runs — the Windows certificate store
on Windows, the system's CA certificates on Linux and in the server image. For
a self-signed certificate or your own CA, add it to the trusted certificates of
that machine (for the server image, an image built on it that adds the
certificate), and connect by the name the certificate carries.

## A secret is missing {#secrets}

`secret.missing`: an experiment uses `{{secret.NAME}}` and no value is stored
under that name where it runs.

- **Desktop app on Windows**: set it under [[ui:exp.secrets]] in the editor's [[ui:exp.params]].
  It is kept in Windows Credential Manager, so a new machine needs it set
  again.
- **Server**: put the value in the file `/run/secrets/signallab/NAME` (or the
  folder `--secrets-dir` names) or in the environment variable
  `SIGNALLAB_SECRET_NAME`. A server cannot set secrets from the page
  (`secret.read_only`).
- **Desktop app on Linux** has no store for secrets (`secret.unsupported`).
  Run such an experiment with `signallab run`, which reads secrets from files
  and variables, or on a server.

See [files](files.md#secrets).

## A run stops after five minutes {#run-timeout}

A run that takes longer than 300 s fails with `run.timeout`; that is the
longest a run can take. A shorter limit can be given to a run through the API
(`timeout` of [`/api/run`](../api/run.md#request)) or the command line
(`signallab run --timeout`).

## The app does not update {#updates}

- The desktop app looks for a new version once a day while
  [[ui:update.auto]] is on, and when you press [[ui:update.check]] in
  [[ui:about.open]].
- It asks the studio's hub first and GitHub when the hub cannot be reached.
  When a network blocks both, [[ui:update.check]] gives
  [[ui:update.checkFailed]]; the daily look fails without a word.
- It is offered only published releases, never drafts or pre-releases.
- A new release reaches the app when the hub offers it, which can be some time
  after it appears on GitHub: the hub rolls a release out to a share of installs
  at a time.
- It installs only when you press [[ui:update.install]] — running jobs are
  stopped first — and only a release whose signature checks out:
  [[ui:update.installFailed]] otherwise.
- A server updates with its image: `docker compose pull && docker compose up -d`
  in the folder of its compose file.

## Where the logs are {#logs}

- **Desktop app**: it writes no log files. The [[ui:console.title]] tab in the
  bottom panel lists what each tool did and what went wrong, and
  [[ui:feedback.open]] attaches it to a message to the developers (without
  this computer's name, its address or your folders). Each run's report is in
  `runs/` in the data folder.
- **Server**: it logs to its standard output and error — `docker logs signallab`
  in Docker. Every job start is logged with the address of the client that
  asked. `--log` (or `SIGNALLAB_LOG`) sets the level: `error`, `warn`, `info`
  (the default) or `debug`; `--log-format json` (or `SIGNALLAB_LOG_FORMAT`)
  writes one JSON object per line.
- **Command line**: `signallab` writes its messages to its error output; see
  [the command line](../automation/cli.md).

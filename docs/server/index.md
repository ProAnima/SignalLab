---
title: Running a server
description: Run Signal Lab on a Linux machine next to the gear and use it from any browser on the network, with an HTTP API for scripts and CI.
---

# Running Signal Lab as a server

`signal-lab-server` is Signal Lab without a window: the same engine, serving the
same interface to a browser. Put it on a machine next to the gear — a rack PC, a
show-control VM, a shared lab box — and open it from Chrome, Firefox or Edge
anywhere on the network: every screen works as in the desktop app, and runs,
reports and exports download through the browser.

Scripts and pipelines use the same server through its [HTTP API](../api/index.md),
and [`signallab --server`](../automation/cli.md#run-on-server) sends runs to it.

A few things are different from the desktop app:

- **One server is one engine.** Every page signed in to it sees the same running
  jobs, the same signal and emulator libraries and the same cookie jar of the
  [[ui:nav.http]] screen ([[ui:http.keepCookies]]). People who share a server
  share those.
- **Secrets are the server's**, read from its environment or from files; they
  cannot be set from the browser (see [Secrets](#secrets)).
- **The firewall is the host's.** The server never changes it; the desktop
  app's firewall notice does not appear.
- **It updates with its image**, not through the app's updater (see
  [Updating](#update)).

## On a Linux host, in one command {#install-script}

On a Linux machine with internet access:

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh
```

The script:

1. installs Docker if it is missing — it asks first, using Docker's own
   installer (`get.docker.com`);
2. writes a `compose.yaml` into `/opt/signallab` (`~/signallab` when you are not
   root);
3. pulls the image and starts the server with host networking, so OSC, UDP,
   broadcast, multicast and discovery reach the real network;
4. waits until the server answers its health check (up to 90 seconds);
5. prints the addresses to open, the access token to sign in with, and the
   commands to update, read the logs and remove it;
6. when ufw or firewalld is on, offers to open the server's port (see
   [The host's firewall](#firewall)).

Pass options after `sh -s --`:

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh -s -- --version [[version]] --port 8430
```

| Option | What it does | Default |
| --- | --- | --- |
| `--version X.Y.Z` | The image version (`latest` or `X.Y.Z`; a leading `v` is dropped). | `latest` |
| `--port N` | The port browsers use. | `1430` |
| `--listen IP:PORT` | Listen on one address only. | `0.0.0.0:<port>` |
| `--dir DIR` | Where the compose file goes. | `/opt/<name>` as root, `~/<name>` otherwise |
| `--name NAME` | The container and its data volume: lower-case letters, digits, `-` and `_`. A second server on the same host needs a name of its own. | `signallab` |
| `--image NAME` | Another image or registry; a full `NAME:TAG` is used as it is. | `ghcr.io/proanima/signallab` |
| `--open-udp PORTS` | With a firewall on, also let UDP in on these ports, for monitors and waits: `9000,9100:9110`. | |
| `--no-firewall` | Never change ufw or firewalld. | |
| `--yes`, `-y` | Answer yes: install Docker, open the firewall, delete data with `--purge`. | |
| `--uninstall` | Stop and remove the server; the data stays. | |
| `--purge` | With `--uninstall`: delete the data and the token too. | |
| `--help`, `-h` | Print the options. | |

It needs Docker's Compose plugin (the `docker-compose-plugin` package), which
Docker's installer brings. The image is built for x86_64 and arm64.

**Run it again to update:** the same command pulls the newest image (or the
`--version` you give) and restarts the server; the data and the token stay.
With `--name`, give the same name again.

**Settings of your own** — experiment secrets, `SIGNALLAB_ALLOWED_HOSTS`,
`SIGNALLAB_SECURE_COOKIE` behind HTTPS — go in `compose.override.yaml` next to
the compose file. Docker Compose merges it in, and the script rewrites
`compose.yaml` on every run but never touches the override. A `compose.yaml` it
did not write is kept as `compose.yaml.before-install`.

```yaml
# compose.override.yaml
services:
  signallab:
    environment:
      SIGNALLAB_ALLOWED_HOSTS: lab-pc.example.com,192.0.2.10
      SIGNALLAB_SECRET_API_TOKEN: ${API_TOKEN}
```

**To remove it:**

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh -s -- --uninstall
```

The data stays in its Docker volume, and installing again brings it back with the
same token. `--uninstall --purge` deletes the data and the token too, after
asking.

### The host's firewall {#firewall}

With host networking the server listens on the host's own ports, so the host's
firewall decides who reaches it. When ufw or firewalld is on, the script asks
before it opens the server's TCP port and the UDP ports of `--open-udp`, writes
down what it opened (`.firewall` next to the compose file), and `--uninstall`
closes exactly that again. If you decline, browsers on other machines reach the
server only once the firewall lets them, and monitors and waits hear other
machines only on UDP ports it opens.

## Docker {#docker}

### The image {#image}

`ghcr.io/proanima/signallab`, for `linux/amd64` and `linux/arm64`, published
with every release:

| Tag | What it is |
| --- | --- |
| `X.Y.Z` | That release. |
| `X.Y` | The newest stable release of that line. |
| `latest` | The newest stable release. |

It holds `signal-lab-server`, the built interface and the command line
`signallab`, and starts the server with these settings:

| Variable | Value in the image |
| --- | --- |
| `SIGNALLAB_LISTEN` | `0.0.0.0:1430` |
| `SIGNALLAB_DATA_DIR` | `/data` |
| `SIGNALLAB_UI_DIR` | `/usr/share/signal-lab/ui` |
| `SIGNALLAB_GENERATE_TOKEN` | `true` |

It runs as an unprivileged user (uid and gid 10001), writes only to `/data` (a
volume), exposes port `1430`, and checks its own health every 30 seconds.

### Starting it {#docker-run}

```bash
docker run -d --name signallab --network host --restart unless-stopped \
  -v signallab-data:/data --read-only --cap-drop ALL --security-opt no-new-privileges \
  ghcr.io/proanima/signallab:[[version]]
docker logs signallab                    # on the first start: "Sign in with it:" and the token
```

Then open `http://<host>:1430` and sign in with the token. To see the token
again later:

```bash
docker exec signallab cat /data/token
```

`--read-only`, `--cap-drop ALL` and `no-new-privileges` are optional and cost
nothing: the server needs no privileges and writes only to `/data`.

### Docker Compose {#compose}

The repository's `deploy/compose.yaml` is the same as a Compose file:

```yaml
name: signallab

services:
  signallab:
    image: ${SIGNALLAB_IMAGE:-ghcr.io/proanima/signallab:latest}
    container_name: signallab
    network_mode: host
    environment:
      SIGNALLAB_GENERATE_TOKEN: "true"
    volumes:
      - signallab-data:/data
    read_only: true
    cap_drop: [ALL]
    security_opt: ["no-new-privileges:true"]
    restart: unless-stopped
    stop_grace_period: 15s

volumes:
  signallab-data:
```

```bash
docker compose up -d
docker exec signallab cat /data/token
```

Set `SIGNALLAB_IMAGE=ghcr.io/proanima/signallab:X.Y.Z` to pin a version.

### Networking {#networking}

| Docker networking | What works | What does not |
| --- | --- | --- |
| `--network host` (`network_mode: host`), on a Linux host | Everything: OSC, UDP, TCP, HTTP, WebSocket and MQTT to the LAN, listening ports, broadcast, multicast, discovery. | — |
| Bridge (the default), with published ports | Unicast to the hosts the container reaches; listeners on published ports (`-p 1430:1430 -p 9000:9000/udp`). | Broadcast and multicast; replies to ports that are not published. |
| Docker Desktop on Windows or macOS | Unicast and published listeners. | Host networking to the physical network. On Windows, use the desktop app. |

### The data volume {#data-volume}

`/data` holds everything the server keeps: the experiment, the signal and
emulator libraries, run reports, exports, and the token. A named volume, as
above, starts out owned by the image's user. A folder of the host mounted there
must be writable by uid 10001:

```bash
sudo mkdir -p /srv/signallab && sudo chown 10001:10001 /srv/signallab
docker run -d --name signallab --network host -v /srv/signallab:/data ghcr.io/proanima/signallab:[[version]]
```

## Without Docker {#binary}

The server is published as the image. To run `signal-lab-server` directly, build
it from the source (see [Building](../develop/building.md)):

```bash
npm install
npm run build                            # the interface, into dist/
cargo run --release -p signal-lab-server
```

It serves `dist/` on `http://127.0.0.1:1430`, for this machine only, with no
token needed. Add `--listen` and a token to open it to the network.

## Options {#options}

Every option has an environment variable, for containers. Options win over
variables.

| Option | Variable | Default | What it does |
| --- | --- | --- | --- |
| `--listen IP:PORT` | `SIGNALLAB_LISTEN` | `127.0.0.1:1430` | Where to listen. Any address but loopback needs a token. |
| `--token-file PATH` | `SIGNALLAB_TOKEN_FILE` | | A file holding the access token (a Docker secret, say). |
| `--token TOKEN` | `SIGNALLAB_TOKEN` | | The access token itself. Prefer the file: arguments are visible to other users of the machine. |
| `--generate-token` | `SIGNALLAB_GENERATE_TOKEN` | off | Without a token given, on an address beyond loopback: use the token kept in `<data folder>/token`, making it on the first start. |
| `--data-dir PATH` | `SIGNALLAB_DATA_DIR` | `Documents/SignalLab` in the user's home folder | The data folder. |
| `--secrets-dir PATH` | `SIGNALLAB_SECRETS_DIR` | `/run/secrets/signallab` | A folder of read-only secrets, one file per name. |
| `--ui-dir PATH` | `SIGNALLAB_UI_DIR` | `ui` next to the program, else `./dist` | The built interface. Without one, only the API is served. |
| `--allowed-host NAME` | `SIGNALLAB_ALLOWED_HOSTS` | | Host names the server may be reached by, comma-separated. They and the loopback names are always accepted, with a token or without; with none given, a server with a token answers to any name and one without only to loopback names (see [Host names](security.md#hosts)). |
| `--secure-cookie` | `SIGNALLAB_SECURE_COOKIE` | off | Send the session cookie over HTTPS only. Set it behind an HTTPS proxy. |
| `--log FILTER` | `SIGNALLAB_LOG` | `info` | What to log: `error`, `warn`, `info`, `debug`, or per module (`signal_lab_server=debug`). |
| `--log-format text\|json` | `SIGNALLAB_LOG_FORMAT` | `text` | Log lines as text, or one JSON object per line. |

Switches take `true` or `false` from their variable:
`SIGNALLAB_GENERATE_TOKEN=true`.

| Command | What it does |
| --- | --- |
| `signal-lab-server token` | Prints a new random token: 64 hexadecimal characters. |
| `signal-lab-server healthcheck` | Exits `0` when a server answers on `--listen` (the image's health check). |
| `signal-lab-server --version` | Prints the version. |
| `signal-lab-server --help` | Prints every option. |

**Exit codes:** `0` when stopped by <kbd>Ctrl</kbd>+<kbd>C</kbd> or `SIGTERM`;
`2` when the settings are refused (no token on a reachable address, a token too
short, a token given twice, a token file that cannot be read,
`--generate-token` without a data folder); `1` when it cannot listen on the
address, or the data folder cannot be written. The reason is printed on stderr.

## The access token {#token}

Without a token the server listens only on loopback and serves only this
machine. On any other address it needs one, and refuses to start without it.
There are three ways to give it:

| How | When to use it |
| --- | --- |
| `--generate-token` (on in the image) | Nothing to set up: on the first start the server makes a token, saves it in `<data folder>/token` — readable by its own user only — and prints it once in the log. Later starts reuse it, so signed-in browsers and scripts keep working across restarts and updates. It needs a data folder (`--data-dir`). |
| `--token-file PATH` | A token of your own in a file, such as a Docker secret. A line break at its end is not part of the token. |
| `SIGNALLAB_TOKEN` | The token in the environment. |

A token has at least **24** characters and no spaces or line breaks; give it in
one way only. `signal-lab-server token` makes a good one:

```bash
docker run --rm ghcr.io/proanima/signallab:[[version]] token > signallab_token.txt
```

and Compose hands it in as a secret (the file must be readable by uid 10001):

```yaml
services:
  signallab:
    environment:
      SIGNALLAB_TOKEN_FILE: /run/secrets/signallab_token
    secrets:
      - signallab_token

secrets:
  signallab_token:
    file: ./signallab_token.txt
```

A token given explicitly wins over `--generate-token`, and nothing is made then.
To change a made token, stop the server, delete `<data folder>/token` and start
it again: it makes and prints a new one. A token file that is damaged is
reported, never replaced.

## Signing in {#sign-in}

Open `http://<host>:1430`. A server with a token sends a browser to its sign-in
page first, in the browser's language; paste the token once, and the browser
stays signed in for 7 days. [[ui:app.signOut]] in the interface ends the
session. Sessions are kept in the server's memory: a restart signs everyone
out.

On loopback without a token there is no sign-in.

Scripts send the token with every request, as `Authorization: Bearer <token>`:

```bash
curl -fsS http://192.0.2.10:1430/api/invoke/app_info \
  -H "Authorization: Bearer $(cat signallab_token.txt)" \
  -H "Content-Type: application/json" -d 'null'
```

See [The HTTP API](../api/index.md), and [Server security](security.md) for
the rules behind all of this.

## The data folder {#data}

The server keeps its files in the data folder: the experiment, the signal and
emulator libraries, run reports, exports, and a made token. It is
`--data-dir` (`SIGNALLAB_DATA_DIR`), `/data` in the image, and otherwise
`Documents/SignalLab` in the home folder of the user it runs as. A data folder
given with `--data-dir` is made if it is missing and checked at start: if the
server cannot write there, it stops and names the folder. See
[Files](../reference/files.md).

Downloads (reports, exports) come only from inside this folder.

## Secrets {#secrets}

Experiments read secrets as `{{secret.NAME}}`. On a server they are read-only,
from:

1. the environment variable `SIGNALLAB_SECRET_NAME`, else
2. the file `NAME` in the secrets folder (`--secrets-dir`, by default
   `/run/secrets/signallab` — the Docker secrets layout).

A file's trailing line break is not part of the value; an empty value counts as
not set; a value is at most 16 KiB. Names are letters, digits and `_`, not
starting with a digit. The interface shows which secrets are set on the server,
but setting one from the browser is refused — values never go anywhere weaker,
and never come back out (see [Secrets](security.md#secrets)).

With Compose, a secret file per name:

```yaml
services:
  signallab:
    secrets:
      - source: api_token
        target: /run/secrets/signallab/API_TOKEN

secrets:
  api_token:
    file: ./api_token.txt
```

The file must be readable by uid 10001.

## Behind an HTTPS proxy {#https}

The server speaks plain HTTP. For HTTPS, put a reverse proxy in front of it
(Caddy, nginx, Traefik) that:

- passes the `Host` header on unchanged;
- passes WebSocket upgrades (the interface keeps one open, to `/api/events`);

and start the server with `--secure-cookie`, so the session cookie travels over
HTTPS only, and with `--allowed-host` set to the name people use.

## Updating {#update}

[[ui:update.server]]: the desktop app's updater has nothing to do with it.

- Installed with the script: run the same command again.
- With Compose: `docker compose pull && docker compose up -d`.
- With `docker run`: pull the new image, then remove the container and start it
  again with the same volume.

The data and the token are in the volume, so they stay.

## Health check {#health}

`GET /api/health` answers everyone, without a token:

```bash
curl -s http://127.0.0.1:1430/api/health
# {"auth":true,"status":"ok","version":"[[version]]"}
```

`auth` says whether the server asks for a token. The command
`signal-lab-server healthcheck` asks the same on this machine and exits `0` when
it answers; the image runs it every 30 seconds (5 seconds' timeout, 3 tries), so
`docker ps` shows the container as healthy.

## Logs {#logs}

The server logs to stdout: `docker logs -f signallab`. At the default level
(`info`) it says where it listens and whether it needs a token, its data and
interface folders, every job started — monitors, generators, storms, scans,
runs, emulators — with the client's address, every sign-in, every sign-in with a
wrong token (as a warning), and when a page falls behind the events. The level
`--log debug` adds every command. Lines are coloured only on a terminal (never with
`NO_COLOR` set); `--log-format json` writes one JSON object per line for a log
collector.

## Stopping {#stop}

<kbd>Ctrl</kbd>+<kbd>C</kbd>, `docker stop` or `SIGTERM` closes every page's
connection, stops every job — a run in progress ends as stopped — and exits
`0`. The compose files give it 15 seconds for that.

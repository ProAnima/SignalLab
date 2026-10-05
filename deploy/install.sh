#!/bin/sh
# Signal Lab server on this Linux machine, in one command:
#
#   curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh
#
# It installs Docker when it is missing (it asks first; --yes answers for you),
# writes a compose file, starts the server with host networking so OSC, UDP,
# broadcast and discovery reach the real network, waits until it answers, and
# prints the address and the access token. Run it again to update; the data and
# the token stay. More: https://proanima.github.io/SignalLab/server/, section 7.
#
#   sh install.sh [--version X.Y.Z] [--port N | --listen IP:PORT] [--dir DIR]
#                 [--name NAME] [--image NAME] [--open-udp PORTS] [--no-firewall]
#                 [--yes] [--uninstall [--purge]] [--help]
#
# With ufw or firewalld on, it offers to open the server's port (and the UDP
# ports given with --open-udp, for monitors and waits) and closes them again on
# --uninstall.
#
# Settings of your own (experiment secrets, allowed host names, HTTPS cookies)
# go in compose.override.yaml next to the compose file: this script rewrites
# compose.yaml on every run and never touches the override.
set -eu

IMAGE=ghcr.io/proanima/signallab
VERSION=latest
PORT=1430
LISTEN=
DIR=
YES=0
OPEN_UDP=
FIREWALL=1
ACTION=install
PURGE=0
NAME=signallab
MARK="# Written by Signal Lab's deploy/install.sh"
SCRIPT_URL=https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh

say() { printf '%s\n' "$*"; }
step() { printf '\n\033[1m==> %s\033[0m\n' "$*"; }
fail() { printf '\n\033[31msignal lab install: %s\033[0m\n' "$*" >&2; exit 1; }

# Also when the script came through a pipe, where there is no file to read the header from.
usage() {
  cat <<'EOF'
Signal Lab server on this Linux machine, with Docker and host networking.

  curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh
  curl -fsSL …/install.sh | sh -s -- --version 0.4.0 --port 8430

Options:
  --version X.Y.Z    the image version (default: latest)
  --port N           the port browsers use (default: 1430)
  --listen IP:PORT   listen on one address only (default: 0.0.0.0:PORT)
  --dir DIR          where the compose file goes (default: /opt/signallab as root, ~/signallab otherwise)
  --name NAME        the container and its data volume (default: signallab) — a second server needs its own
  --image NAME       another image or registry (a full NAME:TAG is used as it is)
  --open-udp PORTS   also let UDP in on these ports when a firewall is on: 9000,9100:9110
  --no-firewall      never change the firewall (ufw, firewalld)
  --yes              answer yes: install Docker, open the firewall
  --uninstall        stop and remove the server; the data stays
  --purge            with --uninstall: delete the data and the token too

Run it again to update: the data and the access token stay.
EOF
  exit 0
}

value() { [ $# -ge 2 ] && [ -n "$2" ] || fail "$1 needs a value"; }
while [ $# -gt 0 ]; do
  case "$1" in
    --version) value "$@"; VERSION=$2; shift 2 ;;
    --version=*) VERSION=${1#*=}; shift ;;
    --port) value "$@"; PORT=$2; shift 2 ;;
    --port=*) PORT=${1#*=}; shift ;;
    --listen) value "$@"; LISTEN=$2; shift 2 ;;
    --listen=*) LISTEN=${1#*=}; shift ;;
    --dir) value "$@"; DIR=$2; shift 2 ;;
    --dir=*) DIR=${1#*=}; shift ;;
    --name) value "$@"; NAME=$2; shift 2 ;;
    --name=*) NAME=${1#*=}; shift ;;
    --image) value "$@"; IMAGE=$2; shift 2 ;;
    --image=*) IMAGE=${1#*=}; shift ;;
    --open-udp) value "$@"; OPEN_UDP=$2; shift 2 ;;
    --open-udp=*) OPEN_UDP=${1#*=}; shift ;;
    --no-firewall) FIREWALL=0; shift ;;
    --yes|-y) YES=1; shift ;;
    --uninstall) ACTION=uninstall; shift ;;
    --purge) PURGE=1; shift ;;
    --help|-h) usage ;;
    *) fail "unknown option $1 (see --help)" ;;
  esac
done

case "$PORT" in ''|*[!0-9]*) fail "--port is a number, not $PORT" ;; esac
[ "$PORT" -ge 1 ] && [ "$PORT" -le 65535 ] || fail "--port is 1 to 65535, not $PORT"
[ -n "$LISTEN" ] || LISTEN="0.0.0.0:$PORT"
PORT=${LISTEN##*:}
case "$NAME" in ''|*[!a-z0-9_-]*) fail "--name is lower-case letters, digits, - and _" ;; esac
case "$OPEN_UDP" in *[!0-9,:]*) fail "--open-udp is ports and ranges: 9000,9100:9110" ;; esac
case "$VERSION" in latest|[0-9]*) ;; v[0-9]*) VERSION=${VERSION#v} ;; *) fail "--version is X.Y.Z or latest, not $VERSION" ;; esac
# A full reference (signallab:dev, registry/name:tag) is used as it is.
case "${IMAGE##*/}" in *:*) REF=$IMAGE ;; *) REF="$IMAGE:$VERSION" ;; esac

# ---- where, and with what rights ---------------------------------------------------

[ "$(uname -s)" = Linux ] || fail "this installs the server on Linux. On Windows and macOS, use the desktop app — Docker there cannot reach the physical network with host networking."
case "$(uname -m)" in x86_64|amd64|aarch64|arm64) ;; *) say "Note: the image is built for x86_64 and arm64; this machine is $(uname -m)." ;; esac

SUDO=
if [ "$(id -u)" -ne 0 ]; then
  command -v sudo >/dev/null 2>&1 && SUDO=sudo
fi
if [ -z "$DIR" ]; then
  if [ "$(id -u)" -eq 0 ]; then DIR="/opt/$NAME"; else DIR="$HOME/$NAME"; fi
fi

# A question works even when the script arrives through a pipe (curl … | sh).
ask() {
  [ "$YES" -eq 1 ] && return 0
  if [ -r /dev/tty ] && [ -w /dev/tty ]; then
    printf '%s [y/N] ' "$1" > /dev/tty
    read -r answer < /dev/tty || answer=
    case "$answer" in y|Y|yes|YES|д|Д|да|Да) return 0 ;; esac
    return 1
  fi
  return 1
}

docker_works() { $1 info >/dev/null 2>&1; }

find_docker() {
  DOCKER=
  if docker_works docker; then DOCKER=docker
  elif [ -n "$SUDO" ] && command -v docker >/dev/null 2>&1 && docker_works "$SUDO docker"; then DOCKER="$SUDO docker"
  fi
}

install_docker() {
  command -v curl >/dev/null 2>&1 || fail "curl is needed to install Docker"
  ask "Docker is not installed. Install it now with Docker's own script (get.docker.com)?" \
    || fail "Docker is needed. Install it (https://docs.docker.com/engine/install/) and run this again, or run it with --yes."
  step "Installing Docker"
  curl -fsSL https://get.docker.com | ${SUDO:+$SUDO }sh
  ${SUDO:+$SUDO }systemctl enable --now docker >/dev/null 2>&1 || true
}

if [ "$ACTION" = install ]; then
  find_docker
  if [ -z "$DOCKER" ]; then
    if command -v docker >/dev/null 2>&1; then
      fail "Docker is installed but does not answer. Start it (sudo systemctl start docker), or add yourself to the docker group, and run this again."
    fi
    install_docker
    find_docker
    [ -n "$DOCKER" ] || fail "Docker was installed but does not answer yet. Run this again in a moment."
  fi
else
  find_docker
  [ -n "$DOCKER" ] || fail "Docker does not answer, so there is nothing to remove from it."
fi

if $DOCKER compose version >/dev/null 2>&1; then COMPOSE="$DOCKER compose"
elif command -v docker-compose >/dev/null 2>&1; then COMPOSE="${SUDO:+$SUDO }docker-compose"
else fail "the Docker Compose plugin is missing. Install it (the docker-compose-plugin package) and run this again."
fi

# Files in DIR may need root when DIR is not ours.
AS_OWNER=
if ! mkdir -p "$DIR" 2>/dev/null || [ ! -w "$DIR" ]; then
  [ -n "$SUDO" ] || fail "cannot write to $DIR; choose another folder with --dir"
  AS_OWNER=$SUDO
  $AS_OWNER mkdir -p "$DIR"
fi
FILE="$DIR/compose.yaml"
in_dir() { (cd "$DIR" && $COMPOSE "$@"); }

# ---- the firewall: ufw or firewalld, by port ------------------------------------------

# What this script opened, one "tcp PORT" or "udp PORTS" per line, so --uninstall closes it.
OPENED="$DIR/.firewall"

firewall_tool() {
  if command -v ufw >/dev/null 2>&1 && ${SUDO:+$SUDO }ufw status 2>/dev/null | grep -q "Status: active"; then echo ufw
  elif command -v firewall-cmd >/dev/null 2>&1 && firewall-cmd --state 2>/dev/null | grep -q running; then echo firewalld
  fi
}

# open|close PROTOCOL PORTS — PORTS as ufw writes them (9000 or 9100:9110).
firewall_rule() {
  case "$TOOL" in
    ufw)
      if [ "$1" = open ]; then ${SUDO:+$SUDO }ufw allow "$3/$2" comment "Signal Lab" >/dev/null
      else ${SUDO:+$SUDO }ufw delete allow "$3/$2" >/dev/null 2>&1 || true; fi ;;
    firewalld)
      ports=$(printf '%s' "$3" | tr ':' '-')
      if [ "$1" = open ]; then ${SUDO:+$SUDO }firewall-cmd --quiet --permanent --add-port="$ports/$2"
      else ${SUDO:+$SUDO }firewall-cmd --quiet --permanent --remove-port="$ports/$2" || true; fi ;;
  esac
}

firewall_reload() {
  [ "$TOOL" = firewalld ] && ${SUDO:+$SUDO }firewall-cmd --quiet --reload
  return 0
}

# ---- uninstall ------------------------------------------------------------------------

if [ "$ACTION" = uninstall ]; then
  [ -f "$FILE" ] || fail "no Signal Lab installation in $DIR (use --dir)"
  step "Stopping and removing Signal Lab"
  if [ -f "$OPENED" ]; then
    TOOL=$(firewall_tool)
    if [ -n "$TOOL" ]; then
      while read -r protocol ports; do firewall_rule close "$protocol" "$ports"; done < "$OPENED"
      firewall_reload
      say "Closed in $TOOL what the install had opened."
    fi
    $AS_OWNER rm -f "$OPENED"
  fi
  if [ "$PURGE" -eq 1 ]; then
    ask "Also delete its data — experiments, signals, reports and the access token?" || fail "nothing was removed"
    in_dir down --volumes --remove-orphans
    $AS_OWNER rm -f "$FILE"
    say "Removed, with its data. $DIR is left for any files of your own."
  else
    in_dir down --remove-orphans
    say "Removed. The data stays in the Docker volume; run this script again to bring it back, or use --uninstall --purge to delete it."
  fi
  exit 0
fi

# ---- the compose file -------------------------------------------------------------------

if [ -f "$FILE" ] && ! grep -q "$MARK" "$FILE"; then
  $AS_OWNER cp "$FILE" "$FILE.before-install"
  say "Kept your compose.yaml as compose.yaml.before-install."
fi

step "Writing $FILE"
$AS_OWNER tee "$FILE" > /dev/null <<EOF
$MARK — it is rewritten on every run.
# Your own settings go in compose.override.yaml next to it (Docker Compose
# merges that file in): experiment secrets, SIGNALLAB_ALLOWED_HOSTS,
# SIGNALLAB_SECURE_COOKIE behind HTTPS. See https://proanima.github.io/SignalLab/server/

name: $NAME

services:
  signallab:
    image: $REF
    container_name: $NAME
    # OSC, UDP, broadcast, multicast and discovery reach the physical network.
    network_mode: host
    environment:
      SIGNALLAB_LISTEN: "$LISTEN"
      # The token is made in /data/token on the first start and kept.
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
EOF

# ---- start ------------------------------------------------------------------------------

step "Getting $REF"
if ! in_dir pull --quiet; then
  $DOCKER image inspect "$REF" >/dev/null 2>&1 || fail "cannot get the image $REF (is the version right? is this machine online?)"
  say "Using the copy of $REF this machine already has."
fi

step "Starting Signal Lab"
in_dir up --detach --remove-orphans

printf 'Waiting for it to answer'
state=
i=0
while [ $i -lt 90 ]; do
  state=$($DOCKER inspect --format '{{.State.Status}} {{if .State.Health}}{{.State.Health.Status}}{{end}}' "$NAME" 2>/dev/null || echo missing)
  case "$state" in
    "running healthy") break ;;
    exited*|dead*|missing)
      printf '\n'
      $DOCKER logs --tail 30 "$NAME" >&2 || true
      code=$($DOCKER inspect --format '{{.State.ExitCode}}' "$NAME" 2>/dev/null || echo "?")
      [ "$code" = 2 ] && fail "the server refused its settings (above). Images before 0.4.0 cannot make a token: use --version 0.4.0 or later."
      fail "the server stopped (exit code $code); its last lines are above."
      ;;
  esac
  printf '.'
  sleep 1
  i=$((i + 1))
done
printf '\n'
[ "$state" = "running healthy" ] || fail "the server did not answer within 90 seconds. See: $DOCKER logs $NAME"

TOKEN=$($DOCKER exec "$NAME" cat /data/token 2>/dev/null | tr -d '\r\n' || true)
RUNNING=$($DOCKER exec "$NAME" signal-lab-server --version 2>/dev/null | sed 's/^signal-lab-server/Signal Lab/' || true)
[ -n "$RUNNING" ] || RUNNING="Signal Lab"

# ---- tell them -------------------------------------------------------------------------

HOST=${LISTEN%:*}
ADDRESSES=
if [ "$HOST" = 0.0.0.0 ] || [ "$HOST" = "[::]" ]; then
  for ip in $(hostname -I 2>/dev/null || true); do
    case "$ip" in *:*|127.*) ;; *) ADDRESSES="$ADDRESSES http://$ip:$PORT" ;; esac
  done
  [ -n "$ADDRESSES" ] || ADDRESSES=" http://$(hostname):$PORT"
else
  ADDRESSES=" http://$HOST:$PORT"
fi

printf '\n\033[32m%s is running.\033[0m\n\n' "$RUNNING"
for address in $ADDRESSES; do say "  Open:    $address"; done
say "           (http://127.0.0.1:$PORT on this machine)"
if [ -n "$TOKEN" ]; then
  say "  Token:   $TOKEN"
  say "           Sign in with it once; a browser stays signed in for 7 days. Scripts send it as"
  say "           'Authorization: Bearer <token>'. Show it again: $DOCKER exec $NAME cat /data/token"
fi
say ""
AGAIN=
[ "$NAME" = signallab ] || AGAIN=" --name $NAME"
say "  Update:  curl -fsSL $SCRIPT_URL | sh${AGAIN:+ -s --$AGAIN}    (the data and the token stay)"
say "  Logs:    $DOCKER logs -f $NAME"
say "  Remove:  curl -fsSL $SCRIPT_URL | sh -s -- --uninstall$AGAIN    (add --purge to delete the data too)"
say "  Files:   $FILE  (your settings: compose.override.yaml)"

TOOL=
[ "$FIREWALL" -eq 1 ] && TOOL=$(firewall_tool)
if [ -n "$TOOL" ]; then
  wanted="$PORT/tcp"
  [ -n "$OPEN_UDP" ] && wanted="$wanted and UDP $OPEN_UDP"
  say ""
  if ask "The $TOOL firewall is on. Let other machines reach Signal Lab — $wanted?"; then
    : > "$OPENED.new"
    firewall_rule open tcp "$PORT" && echo "tcp $PORT" >> "$OPENED.new"
    for ports in $(printf '%s' "$OPEN_UDP" | tr ',' ' '); do
      firewall_rule open udp "$ports" && echo "udp $ports" >> "$OPENED.new"
    done
    firewall_reload
    $AS_OWNER mv "$OPENED.new" "$OPENED" 2>/dev/null || $AS_OWNER cp "$OPENED.new" "$OPENED"
    say "  Opened in $TOOL: $wanted (closed again by --uninstall)."
  else
    say "  The $TOOL firewall is on and was not changed: browsers elsewhere reach port $PORT only once it lets them,"
    say "  and monitors and waits hear other machines only on UDP ports it opens (--open-udp 9000,9100:9110)."
  fi
fi

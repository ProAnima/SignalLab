---
title: 서버로 실행하기
description: 장비 옆의 Linux 머신에서 Signal Lab을 실행하고 네트워크의 아무 브라우저에서나 사용하며, 스크립트와 CI를 위한 HTTP API를 제공합니다.
---

# Signal Lab을 서버로 실행하기

`signal-lab-server`는 창이 없는 Signal Lab입니다. 같은 엔진이 같은 인터페이스를 브라우저에
제공합니다. 장비 옆의 머신 — 랙 PC, 쇼 컨트롤 VM, 공용 랩 컴퓨터 — 에 두고 네트워크 어디서든
Chrome, Firefox, Edge에서 열면, 모든 화면이 데스크톱 앱에서처럼 동작하고, 실행, 보고서,
내보내기는 브라우저를 통해 다운로드됩니다.

스크립트와 파이프라인은 [HTTP API](../api/index.md)를 통해 같은 서버를 사용하며,
[`signallab --server`](../automation/cli.md#run-on-server)가 실행을 보냅니다.

데스크톱 앱과 다른 점 몇 가지:

- **서버 하나는 엔진 하나입니다.** 로그인한 모든 페이지가 같은 실행 중 작업, 같은 신호 및
  에뮬레이터 라이브러리, [[ui:nav.http]] 화면의 같은 쿠키 항아리([[ui:http.keepCookies]])를
  봅니다. 서버를 공유하는 사람은 이것들도 공유합니다.
- **시크릿은 서버의 것입니다.** 서버의 환경이나 파일에서 읽으며, 브라우저에서 설정할 수
  없습니다([시크릿](#secrets) 참고).
- **방화벽은 호스트의 것입니다.** 서버는 방화벽을 절대 바꾸지 않으며, 데스크톱 앱의 방화벽
  알림은 나타나지 않습니다.
- **이미지와 함께 업데이트됩니다.** 앱의 업데이트 기능이 아니라([업데이트](#update) 참고).

## Linux 호스트에서 명령 하나로 {#install-script}

인터넷에 연결된 Linux 머신에서:

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh
```

스크립트가 하는 일:

1. Docker가 없으면 설치합니다 — 먼저 물어보고, Docker 자체 설치 프로그램
   (`get.docker.com`)을 사용합니다.
2. `/opt/signallab`(root가 아니면 `~/signallab`)에 `compose.yaml`을 씁니다.
3. 이미지를 받아 호스트 네트워킹으로 서버를 시작해, OSC, UDP, 브로드캐스트, 멀티캐스트,
   디스커버리가 실제 네트워크에 닿게 합니다.
4. 서버가 상태 확인에 응답할 때까지 기다립니다(최대 90초).
5. 열어야 할 주소, 로그인할 액세스 토큰, 업데이트·로그 확인·제거 명령을 출력합니다.
6. ufw나 firewalld가 켜져 있으면 서버의 포트를 열 것인지 제안합니다
   ([호스트의 방화벽](#firewall) 참고).

`sh -s --` 뒤에 옵션을 전달합니다:

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh -s -- --version [[version]] --port 8430
```

| 옵션 | 하는 일 | 기본값 |
| --- | --- | --- |
| `--version X.Y.Z` | 이미지 버전(`latest` 또는 `X.Y.Z`, 앞의 `v`는 무시됨). | `latest` |
| `--port N` | 브라우저가 사용하는 포트. | `1430` |
| `--listen IP:PORT` | 한 주소에서만 수신. | `0.0.0.0:<port>` |
| `--dir DIR` | compose 파일이 들어갈 위치. | root면 `/opt/<name>`, 아니면 `~/<name>` |
| `--name NAME` | 컨테이너와 그 데이터 볼륨: 소문자, 숫자, `-`, `_`. 같은 호스트의 두 번째 서버는 고유한 이름이 필요합니다. | `signallab` |
| `--image NAME` | 다른 이미지나 레지스트리. 전체 `NAME:TAG`는 그대로 사용됩니다. | `ghcr.io/proanima/signallab` |
| `--open-udp PORTS` | 방화벽이 켜져 있을 때 모니터와 대기를 위해 이 포트의 UDP도 허용합니다: `9000,9100:9110`. | |
| `--no-firewall` | ufw나 firewalld를 절대 바꾸지 않음. | |
| `--yes`, `-y` | 예라고 답함: Docker 설치, 방화벽 열기, `--purge`로 데이터 삭제. | |
| `--uninstall` | 서버를 중지하고 제거. 데이터는 남습니다. | |
| `--purge` | `--uninstall`과 함께: 데이터와 토큰도 삭제. | |
| `--help`, `-h` | 옵션을 출력. | |

Docker의 Compose 플러그인(`docker-compose-plugin` 패키지)이 필요하며, Docker 설치 프로그램이
함께 제공합니다. 이미지는 x86_64와 arm64용으로 빌드됩니다.

**다시 실행하면 업데이트됩니다:** 같은 명령이 최신 이미지(또는 지정한 `--version`)를 받아
서버를 다시 시작하며, 데이터와 토큰은 남습니다. `--name`을 주었다면 같은 이름을 다시
주십시오.

**직접 설정** — 실험 시크릿, `SIGNALLAB_ALLOWED_HOSTS`, HTTPS 뒤의
`SIGNALLAB_SECURE_COOKIE` — 은 compose 파일 옆의 `compose.override.yaml`에 넣습니다. Docker
Compose가 이를 병합하고, 스크립트는 실행할 때마다 `compose.yaml`을 다시 쓰지만 오버라이드는
절대 건드리지 않습니다. 스크립트가 쓰지 않은 `compose.yaml`은
`compose.yaml.before-install`로 보존됩니다.

```yaml
# compose.override.yaml
services:
  signallab:
    environment:
      SIGNALLAB_ALLOWED_HOSTS: lab-pc.example.com,192.0.2.10
      SIGNALLAB_SECRET_API_TOKEN: ${API_TOKEN}
```

**제거하려면:**

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh -s -- --uninstall
```

데이터는 Docker 볼륨에 남고, 다시 설치하면 같은 토큰과 함께 돌아옵니다.
`--uninstall --purge`는 먼저 물어본 뒤 데이터와 토큰도 삭제합니다.

### 호스트의 방화벽 {#firewall}

호스트 네트워킹에서는 서버가 호스트 자체의 포트에서 수신하므로, 누가 서버에 닿는지는 호스트의
방화벽이 정합니다. ufw나 firewalld가 켜져 있으면 스크립트는 서버의 TCP 포트와
`--open-udp`의 UDP 포트를 열기 전에 먼저 물어보고, 연 것을 기록하며(`.firewall`, compose 파일
옆), `--uninstall`은 정확히 그 반대를 수행합니다. 거절하면 다른 머신의 브라우저는 방화벽이
허용한 뒤에야 서버에 닿고, 모니터와 대기는 방화벽이 연 UDP 포트에서만 다른 머신의 소리를
듣습니다.

## Docker {#docker}

### 이미지 {#image}

`ghcr.io/proanima/signallab`, `linux/amd64`과 `linux/arm64`용, 모든 릴리스와 함께 게시:

| 태그 | 무엇인지 |
| --- | --- |
| `X.Y.Z` | 해당 릴리스. |
| `X.Y` | 해당 계열의 최신 안정 릴리스. |
| `latest` | 최신 안정 릴리스. |

여기에는 `signal-lab-server`, 빌드된 인터페이스, 명령줄 `signallab`이 들어 있으며 다음 설정으로
서버를 시작합니다:

| 변수 | 이미지에서의 값 |
| --- | --- |
| `SIGNALLAB_LISTEN` | `0.0.0.0:1430` |
| `SIGNALLAB_DATA_DIR` | `/data` |
| `SIGNALLAB_UI_DIR` | `/usr/share/signal-lab/ui` |
| `SIGNALLAB_GENERATE_TOKEN` | `true` |

권한 없는 사용자(uid 및 gid 10001)로 실행되며, `/data`(볼륨)에만 쓰고, 포트 `1430`을
노출하며, 30초마다 자체 상태를 확인합니다.

### 시작하기 {#docker-run}

```bash
docker run -d --name signallab --network host --restart unless-stopped \
  -v signallab-data:/data --read-only --cap-drop ALL --security-opt no-new-privileges \
  ghcr.io/proanima/signallab:[[version]]
docker logs signallab                    # on the first start: "Sign in with it:" and the token
```

그런 다음 `http://<host>:1430`을 열고 토큰으로 로그인합니다. 나중에 토큰을 다시 보려면:

```bash
docker exec signallab cat /data/token
```

`--read-only`, `--cap-drop ALL`, `no-new-privileges`는 선택 사항이며 비용이 들지 않습니다.
서버는 권한이 필요 없고 `/data`에만 씁니다.

### Docker Compose {#compose}

저장소의 `deploy/compose.yaml`은 다음과 같은 Compose 파일입니다:

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

버전을 고정하려면 `SIGNALLAB_IMAGE=ghcr.io/proanima/signallab:X.Y.Z`로 설정합니다.

### 네트워킹 {#networking}

| Docker 네트워킹 | 되는 것 | 안 되는 것 |
| --- | --- | --- |
| Linux 호스트의 `--network host`(`network_mode: host`) | 모든 것: LAN으로의 OSC, UDP, TCP, HTTP, WebSocket, MQTT, 수신 포트, 브로드캐스트, 멀티캐스트, 디스커버리. | — |
| 게시된 포트가 있는 브리지(기본값) | 컨테이너가 닿는 호스트로의 유니캐스트, 게시된 포트의 리스너(`-p 1430:1430 -p 9000:9000/udp`). | 브로드캐스트와 멀티캐스트, 게시되지 않은 포트로의 응답. |
| Windows나 macOS의 Docker Desktop | 유니캐스트와 게시된 리스너. | 물리 네트워크로의 호스트 네트워킹. Windows에서는 데스크톱 앱을 사용하십시오. |

### 데이터 볼륨 {#data-volume}

`/data`에는 서버가 보관하는 모든 것이 들어 있습니다: 실험, 신호 및 에뮬레이터 라이브러리, 실행
보고서, 내보내기, 토큰. 위처럼 명명된 볼륨은 처음에는 이미지의 사용자가 소유합니다. 여기에
마운트한 호스트 폴더는 uid 10001이 쓸 수 있어야 합니다:

```bash
sudo mkdir -p /srv/signallab && sudo chown 10001:10001 /srv/signallab
docker run -d --name signallab --network host -v /srv/signallab:/data ghcr.io/proanima/signallab:[[version]]
```

## Docker 없이 {#binary}

서버는 이미지로 게시됩니다. `signal-lab-server`를 직접 실행하려면 소스에서
빌드하십시오([빌드](../../develop/building.md) 참고):

```bash
npm install
npm run build                            # the interface, into dist/
cargo run --release -p signal-lab-server
```

`dist/`를 `http://127.0.0.1:1430`에서 제공하며, 이 머신 전용이고 토큰이 필요 없습니다.
네트워크에 열려면 `--listen`과 토큰을 추가하십시오.

## 옵션 {#options}

모든 옵션에는 컨테이너를 위한 환경 변수가 있습니다. 옵션이 변수보다 우선합니다.

| 옵션 | 변수 | 기본값 | 하는 일 |
| --- | --- | --- | --- |
| `--listen IP:PORT` | `SIGNALLAB_LISTEN` | `127.0.0.1:1430` | 수신할 위치. 루프백이 아닌 주소에는 토큰이 필요합니다. |
| `--token-file PATH` | `SIGNALLAB_TOKEN_FILE` | | 액세스 토큰이 든 파일(예: Docker 시크릿). |
| `--token TOKEN` | `SIGNALLAB_TOKEN` | | 액세스 토큰 자체. 파일을 권장합니다: 인수는 머신의 다른 사용자에게 보입니다. |
| `--generate-token` | `SIGNALLAB_GENERATE_TOKEN` | 꺼짐 | 주어진 토큰 없이 루프백 너머 주소에서: `<data folder>/token`에 보관된 토큰을 사용하며, 처음 시작할 때 만듭니다. |
| `--data-dir PATH` | `SIGNALLAB_DATA_DIR` | 사용자 홈 폴더의 `Documents/SignalLab` | 데이터 폴더. |
| `--secrets-dir PATH` | `SIGNALLAB_SECRETS_DIR` | `/run/secrets/signallab` | 이름당 파일 하나인 읽기 전용 시크릿 폴더. |
| `--ui-dir PATH` | `SIGNALLAB_UI_DIR` | 프로그램 옆의 `ui`, 없으면 `./dist` | 빌드된 인터페이스. 없으면 API만 제공됩니다. |
| `--allowed-host NAME` | `SIGNALLAB_ALLOWED_HOSTS` | | 서버에 접근할 수 있는 호스트 이름(쉼표로 구분). 이것들과 루프백 이름은 토큰이 있든 없든 항상 허용됩니다. 없으면 토큰이 있는 서버는 모든 이름에, 없는 서버는 루프백 이름에만 응답합니다([호스트 이름](security.md#hosts) 참고). |
| `--secure-cookie` | `SIGNALLAB_SECURE_COOKIE` | 꺼짐 | 세션 쿠키를 HTTPS로만 보냅니다. HTTPS 프록시 뒤에서 설정하십시오. |
| `--log FILTER` | `SIGNALLAB_LOG` | `info` | 무엇을 기록할지: `error`, `warn`, `info`, `debug`, 또는 모듈별(`signal_lab_server=debug`). |
| `--log-format text\|json` | `SIGNALLAB_LOG_FORMAT` | `text` | 로그 줄을 텍스트로, 또는 한 줄에 JSON 객체 하나로. |

스위치는 변수에서 `true` 또는 `false`를 받습니다: `SIGNALLAB_GENERATE_TOKEN=true`.

| 명령 | 하는 일 |
| --- | --- |
| `signal-lab-server token` | 새 임의 토큰을 출력합니다: 16진수 64자. |
| `signal-lab-server healthcheck` | `--listen`에서 서버가 응답하면 `0`으로 종료합니다(이미지의 상태 확인). |
| `signal-lab-server --version` | 버전을 출력합니다. |
| `signal-lab-server --help` | 모든 옵션을 출력합니다. |

**종료 코드:** <kbd>Ctrl</kbd>+<kbd>C</kbd>나 `SIGTERM`으로 중지하면 `0`; 설정이 거부되면
(접근 가능한 주소에 토큰 없음, 토큰이 너무 짧음, 토큰을 두 번 지정, 읽을 수 없는 토큰 파일,
데이터 폴더 없는 `--generate-token`) `2`; 주소에서 수신할 수 없거나 데이터 폴더에 쓸 수 없으면
`1`. 이유는 stderr에 출력됩니다.

## 액세스 토큰 {#token}

토큰이 없으면 서버는 루프백에서만 수신하고 이 머신에만 서비스합니다. 다른 주소에서는 토큰이
필요하며, 없으면 시작을 거부합니다. 주는 방법은 세 가지입니다:

| 방법 | 언제 사용하는지 |
| --- | --- |
| `--generate-token`(이미지에서 켜짐) | 설정할 것이 없습니다: 처음 시작할 때 서버가 토큰을 만들어 `<data folder>/token`에 저장하고(자체 사용자만 읽을 수 있음) 로그에 한 번 출력합니다. 이후 시작은 재사용하므로 로그인한 브라우저와 스크립트가 재시작과 업데이트를 거쳐도 계속 작동합니다. 데이터 폴더(`--data-dir`)가 필요합니다. |
| `--token-file PATH` | Docker 시크릿처럼 파일에 담은 직접 만든 토큰. 끝의 줄바꿈은 토큰의 일부가 아닙니다. |
| `SIGNALLAB_TOKEN` | 환경에 있는 토큰. |

토큰은 최소 **24**자이며 공백이나 줄바꿈이 없어야 하고, 한 가지 방법으로만 주십시오.
`signal-lab-server token`이 좋은 토큰을 만듭니다:

```bash
docker run --rm ghcr.io/proanima/signallab:[[version]] token > signallab_token.txt
```

Compose는 이를 시크릿으로 전달합니다(파일은 uid 10001이 읽을 수 있어야 함):

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

명시적으로 준 토큰이 `--generate-token`보다 우선하며, 이때는 아무것도 만들어지지 않습니다.
만든 토큰을 바꾸려면 서버를 중지하고 `<data folder>/token`을 삭제한 뒤 다시 시작하십시오.
새 토큰을 만들어 출력합니다. 손상된 토큰 파일은 보고되며, 절대 교체되지 않습니다.

## 로그인 {#sign-in}

`http://<host>:1430`을 여십시오. 토큰이 있는 서버는 먼저 브라우저를 브라우저 언어의 로그인
페이지로 보냅니다. 토큰을 한 번 붙여넣으면 브라우저는 7일 동안 로그인 상태로 남습니다.
인터페이스의 [[ui:app.signOut]]이 세션을 끝냅니다. 세션은 서버 메모리에 보관됩니다. 다시
시작하면 모두 로그아웃됩니다.

토큰 없는 루프백에서는 로그인이 없습니다.

스크립트는 모든 요청과 함께 토큰을 `Authorization: Bearer <token>`으로 보냅니다:

```bash
curl -fsS http://192.0.2.10:1430/api/invoke/app_info \
  -H "Authorization: Bearer $(cat signallab_token.txt)" \
  -H "Content-Type: application/json" -d 'null'
```

[HTTP API](../api/index.md)와 이 모든 것의 배후 규칙은 [서버 보안](security.md)을
참고하십시오.

## 데이터 폴더 {#data}

서버는 데이터 폴더에 파일을 보관합니다: 실험, 신호 및 에뮬레이터 라이브러리, 실행 보고서,
내보내기, 만든 토큰. `--data-dir`(`SIGNALLAB_DATA_DIR`)이며, 이미지에서는 `/data`, 그 밖에는
실행 사용자의 홈 폴더에 있는 `Documents/SignalLab`입니다. `--data-dir`로 지정한 데이터 폴더는
없으면 만들고 시작할 때 확인합니다. 서버가 거기에 쓸 수 없으면 중지하고 폴더를 알립니다.
[파일](../reference/files.md)을 참고하십시오.

다운로드(보고서, 내보내기)는 이 폴더 안에서만 나옵니다.

## 시크릿 {#secrets}

실험은 시크릿을 `{{secret.NAME}}`으로 읽습니다. 서버에서는 읽기 전용이며 다음에서 읽습니다:

1. 환경 변수 `SIGNALLAB_SECRET_NAME`, 또는
2. 시크릿 폴더의 `NAME` 파일(`--secrets-dir`, 기본값 `/run/secrets/signallab` — Docker 시크릿
   레이아웃).

파일 끝의 줄바꿈은 값의 일부가 아니며, 빈 값은 설정되지 않은 것으로 간주하고, 값은 최대
16 KiB입니다. 이름은 문자, 숫자, `_`이며 숫자로 시작하지 않습니다. 인터페이스는 서버에 어떤
시크릿이 설정되어 있는지 보여주지만, 브라우저에서 설정하는 것은 거부됩니다. 값은 더 약한 곳에
저장되지 않고, 절대 되돌아 나오지 않습니다([시크릿](security.md#secrets) 참고).

Compose에서는 이름당 시크릿 파일 하나:

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

파일은 uid 10001이 읽을 수 있어야 합니다.

## HTTPS 프록시 뒤에서 {#https}

서버는 평문 HTTP를 사용합니다. HTTPS를 위해서는 앞에 리버스 프록시(Caddy, nginx, Traefik)를
두십시오. 프록시는:

- `Host` 헤더를 변경 없이 전달하고;
- WebSocket 업그레이드를 전달합니다(인터페이스가 `/api/events`로 하나를 열어 둡니다).

그리고 서버를 `--secure-cookie`와 함께 시작해 세션 쿠키가 HTTPS로만 오가게 하고,
`--allowed-host`를 사람들이 사용하는 이름으로 설정하십시오.

## 업데이트 {#update}

[[ui:update.server]]: 데스크톱 앱의 업데이트 기능과는 아무 관련이 없습니다.

- 스크립트로 설치: 같은 명령을 다시 실행.
- Compose: `docker compose pull && docker compose up -d`.
- `docker run`: 새 이미지를 받은 뒤 컨테이너를 제거하고 같은 볼륨으로 다시 시작.

데이터와 토큰은 볼륨에 있으므로 남습니다.

## 상태 확인 {#health}

`GET /api/health`는 토큰 없이 모든 사람에게 응답합니다:

```bash
curl -s http://127.0.0.1:1430/api/health
# {"auth":true,"status":"ok","version":"[[version]]"}
```

`auth`는 서버가 토큰을 요구하는지 알려줍니다. `signal-lab-server healthcheck` 명령은 이
머신에서 같은 것을 묻고 응답하면 `0`으로 종료합니다. 이미지는 이를 30초마다 실행하므로(5초
타임아웃, 3회 시도) `docker ps`가 컨테이너를 정상으로 표시합니다.

## 로그 {#logs}

서버는 표준 출력에 기록합니다: `docker logs -f signallab`. 기본 수준(`info`)에서는 수신
위치와 토큰 필요 여부, 데이터 및 인터페이스 폴더, 시작된 모든 작업(모니터, 발생기, 스톰,
스캔, 실행, 에뮬레이터)을 클라이언트 주소와 함께, 모든 로그인, 틀린 토큰으로 시도한 모든
로그인(경고로), 그리고 페이지가 이벤트를 따라가지 못할 때를 기록합니다. `--log debug` 수준은
모든 명령을 추가합니다. 색상은 터미널에서만 입혀지며(`NO_COLOR`가 설정되면 절대 아님),
`--log-format json`은 로그 수집기를 위해 한 줄에 JSON 객체 하나를 씁니다.

## 중지 {#stop}

<kbd>Ctrl</kbd>+<kbd>C</kbd>, `docker stop`, `SIGTERM`은 모든 페이지의 연결을 닫고, 모든
작업을 중지하며(진행 중인 실행은 중지됨으로 끝남), `0`으로 종료합니다. compose 파일은 이를
위해 15초를 줍니다.

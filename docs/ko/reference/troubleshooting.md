---
title: 문제 해결
description: Signal Lab의 흔한 문제와 해결 방법 — 아무것도 도착하지 않음, 사용 중인 포트, 방화벽, Docker 네트워킹, 서버, MQTT, 인증서, 업데이트, 로그.
---

# 문제 해결

Signal Lab이 보고하는 모든 실패에는 코드가 있습니다. 각 코드의 메시지는
[오류 메시지](errors.md)에 있습니다. 네트워크 실패는 [`transport`](errors.md#transport)
코드입니다: `refused`, `timeout`, `dns`, `unreachable`, `reset`, `address_in_use`,
`address_unavailable`, `denied`, `tls`, `target_invalid`, `failed`. 아래는 흔한 문제들입니다.

## 아무것도 도착하지 않음 {#nothing-arrives}

먼저 무언가 Signal Lab에 닿기는 하는지 알아내십시오. 하단 패널에서 [[ui:dock.inspector]] 탭을
열고 [[ui:ins.arm]]을 누릅니다. 도구가 보내거나 받는 모든 데이터그램, 요청, 메시지가 어디서
왔는지와 함께 거기에 나열됩니다.

### 수신 주소 {#listen-address}

- `0.0.0.0:<port>`의 [[ui:common.bind]]는 모든 네트워크 카드에서 수신하고,
  `127.0.0.1:<port>`는 이 머신의 소리만 듣습니다. 네트워크의 장비에는 첫 번째가 필요합니다.
- 장치는 이 머신의 주소와 사용자가 수신하는 포트로 보내야 합니다. 헤더가 이 머신의 이름과
  주소를 표시합니다.
- 이 머신의 것이 아닌 주소는 `address_unavailable`로 실패합니다.

### 방화벽 {#firewall}

`127.0.0.1`의 트래픽은 절대 필터링되지 않으므로, 한 머신에서의 테스트는 되지만 다른
머신에서의 같은 테스트는 아무것도 받지 못합니다.

**Windows.** Windows 방화벽은 프로그램별로 판단합니다. Windows는 보통 프로그램이 처음
수신할 때 한 번 묻습니다 — 그때 *취소*를 누르면 차단 규칙이 남아 허용 규칙을 이깁니다.
Windows가 공용이라고 부르는 네트워크(흔히 행사장 Wi-Fi)에서는 아예 묻지 않을 수 있습니다.

- 데스크톱 앱은 모니터, 디스커버리 리스너, 릴레이, 실행 또는 에뮬레이터가 수신을 시작할 때
  한 번 방화벽을 확인합니다. 방화벽이 가로막고 있으면 [[ui:fw.allow]] — 또는 공용
  네트워크에서는 [[ui:fw.allowPublic]] — 가 있는 알림으로 알립니다. Windows가 관리자 권한을
  요청한 뒤, 프로그램의 인바운드 규칙(차단 규칙 포함)이 허용 규칙 하나로 교체됩니다.
  [[ui:fw.dismiss]]가 알림을 숨깁니다.
- 터미널에서: `signallab doctor`가 무엇이 가로막고 있는지 보여주고, `signallab firewall allow`가
  고칩니다(공용 네트워크도 `--public`), 같은 관리자 프롬프트와 함께.
- *모두를 위해* 설치된 설정(`.exe`)은 허용 규칙을 스스로 추가합니다(개인 및 도메인 네트워크).
  `/NOFIREWALL`로 실행하지 않은 경우입니다. *나를 위해* 설치한 설정은 그럴 수 없고, `.msi`는
  방화벽을 배포하는 사람에게 맡깁니다.
- 서버는 호스트의 방화벽을 절대 바꾸지 않습니다. 관리자가 포트를 엽니다(설치 스크립트가 ufw나
  firewalld와 함께 열 것을 제안합니다).

**Linux.** ufw나 firewalld 같은 방화벽은 프로그램이 아니라 포트로 작동합니다.
`signallab doctor`가 켜져 있는 것과 포트를 여는 방법을 알려줍니다. 예를 들어
`sudo ufw allow 9000/udp`.

### 브로드캐스트와 멀티캐스트 {#broadcast-multicast}

- 라우터는 브로드캐스트를 전달하지 않습니다: `255.255.255.255`와 `x.x.x.255`는 보내는 카드가
  있는 네트워크 세그먼트에만 닿습니다. 카드가 여러 개면
  [[ui:bc.bindSource]] ([[ui:bc.socketOptions]] 아래)에 카드의 주소를 넣으십시오. 예:
  `10.0.0.5:0`.
- 멀티캐스트 데이터그램은 그 그룹에 가입한 리스너에게만 닿습니다 — 디스커버리 리스너에서는
  [[ui:bc.joinGroups]]. 기본값이 1인 [[ui:bc.ttl]]에서는 이 네트워크에 머뭅니다.
- 같은 머신에서 자신의 멀티캐스트를 들으려면 [[ui:bc.mcastLoop]]를 켜 두십시오.

### 응답하지 않는 장치 {#no-answer}

UDP에는 전달 확인이 없습니다. 아무도 수신하지 않는 포트로 보낸 데이터그램도 보낸 것으로
칩니다. 그러면 Windows가 돌아온 ICMP *포트 도달 불가*를 그 소켓의 다음 수신에서 연결
재설정으로 보고합니다. Signal Lab의 모니터, 리스너, 릴레이, 대기는 이를 무시하고 계속
수신합니다. 따라서 응답이 오지 않으면 대기는 시간이 지난 뒤 `wait.timeout`으로 실패하며,
전송에 관한 오류로 실패하지 않습니다. [[ui:dock.inspector]]에서 메시지가 올바른 주소로
나갔는지 확인한 다음, 장치를 확인하십시오.

## 보내기가 나가기 전에 거부됨 {#refused-send}

다음이 먼저 검사되며, 하나라도 실패하면 아무것도 보내지 않습니다:

| 코드 | 이유 | 해결 |
| --- | --- | --- |
| `transport.target_invalid` | 대상에 포트가 없거나 `IP:port`도 `host:port`도 아님 | 둘 다 쓰십시오. 예: `192.0.2.20:9000` |
| `transport.dns` | 호스트 이름이 이 머신에서 확인되지 않음 | 이름을 확인하거나 주소를 사용하십시오. IPv4 주소가 있는 이름은 IPv4로 접근하므로 `localhost:9000`은 `127.0.0.1`의 수신자를 찾습니다 |
| `node.osc_address` | OSC 주소가 `/`로 시작하지 않음 — 송신기, 발생기, 신호 또는 단계에서 | `/`로 시작하십시오. 예: `/cue/go` |
| `node.topic_wildcard` | MQTT 발행의 토픽에 `+`나 `#`이 있음 — 다른 곳과 마찬가지로 화면의 연결에서도 | 한 토픽에 발행하십시오. 와일드카드는 구독용입니다 |
| WebSocket 메시지의 `node.too_long` | 메시지가 16 MiB를 넘음 | 적게 보내십시오. 연결은 열린 채로 남습니다 |

## 포트가 이미 사용 중임 {#port-in-use}

`transport.address_in_use`: 다른 프로그램 — 또는 Signal Lab의 다른 작업 — 이 이미 그 포트에서
수신하고 있습니다.

- **모니터와 실행.** 실행은 첫 단계 전에 대기, 에뮬레이터, 릴레이의 포트를 엽니다. 따라서
  [[ui:osc.monitor]], 디스커버리 리스너 또는 에뮬레이터 작업이 잡고 있는 포트가 있으면 실행은
  시작 전에 실패합니다. 먼저 그 작업을 중지하십시오. [[ui:app.stopAll]]이 모든 것을
  중지합니다.
- **한 실행의 두 에뮬레이터**는 한 전송의 포트를 공유할 수 없습니다. HTTP, MQTT, TCP
  에뮬레이터는 모두 TCP에서, OSC와 UDP 에뮬레이터는 UDP에서 수신합니다
  (`emulator.bind_taken`).
- **실제 서비스 옆에서 수신.** 디스커버리 리스너는 이미 그 포트를 잡고 있는 프로그램과 포트를
  공유할 수 있습니다. [[ui:bc.reuse]]를 켜 두십시오. Linux에서는 그 프로그램도 포트를
  공유해야 합니다. 그러지 않으면 잡힌 포트는 `broadcast.port_shared`입니다.
- **막 중지함.** 중지된 에뮬레이터나 실행이 잡고 있던 포트는 잠시 뒤 풀립니다. 바로 다시
  시작한 에뮬레이터는 잠깐 기다립니다.
- Linux에서 **1024 미만의 포트**는 관리자 권한이 필요합니다(`denied`). 서버 이미지는 권한
  없이 실행되므로 1024 이상의 포트를 사용하십시오.

## Docker의 서버가 네트워크에 닿지 않음 {#docker-network}

브로드캐스트, 멀티캐스트, 디스커버리는 호스트 네트워킹에서만 물리 네트워크에 닿습니다 —
compose 파일의 `network_mode: host` 또는 `docker run --network host` — 그리고 Linux
호스트에서만입니다. 또한 모니터, 대기, 에뮬레이터가 호스트 자체의 포트에서 수신하게 합니다.
Docker의 기본 브리지 네트워크에서는 컨테이너가 자체 네트워크에 있습니다. 브로드캐스트와
멀티캐스트는 절대 빠져나가지 못하고, 게시한 포트만 컨테이너에 닿습니다.

Windows와 macOS에서는 Docker의 호스트 네트워킹이 물리 네트워크에 닿지 않습니다. Windows에서는
데스크톱 앱을 사용하거나, 서버를 Linux 호스트에서 실행하십시오.

## SmartScreen이 설치 프로그램을 경고함 {#smartscreen}

설치 프로그램은 아직 서명되지 않아, Windows SmartScreen이 게시자를 모른다고 말합니다.
*추가 정보*를 고른 뒤 *실행*을 고르십시오. 설치 프로그램은 GitHub의 프로젝트 릴리스에서만
다운로드하십시오.

## 서버가 시작하지 않음 {#server-start}

`signal-lab-server`는 수신하기 전에 설정을 확인하고 오류 출력에 메시지를 남기며 종료합니다:

| 종료 코드 | 메시지 | 해결 |
| --- | --- | --- |
| 2 | refusing to listen on … without a token | 다른 사람이 닿을 수 있는 서버에는 토큰이 필요합니다: `--token-file` 또는 `SIGNALLAB_TOKEN`(`signal-lab-server token`으로 만듦), 또는 `--generate-token`. 아니면 `127.0.0.1`에서 수신하십시오 |
| 2 | the token has N characters; it needs at least 24 | 더 긴 토큰을 사용하십시오 |
| 2 | the token must not contain spaces or line breaks | 토큰 파일은 줄바꿈으로 끝나도 됩니다. 그 밖에는 안 됩니다 |
| 2 | give the token once | `--token`/`SIGNALLAB_TOKEN` 또는 `--token-file`/`SIGNALLAB_TOKEN_FILE` 중 하나만 쓰십시오 |
| 2 | --generate-token keeps the token in the data folder | `--data-dir` 또는 `SIGNALLAB_DATA_DIR`을 설정하십시오 |
| 2 | … it does not hold a valid token; remove it to have a new one made | 데이터 폴더의 `token` 파일이 손상되었습니다 |
| 2 | cannot read the token file … | `--token-file`이 지정한 파일이 없거나 이 사용자가 읽을 수 없습니다 |
| 2 | cannot save the new token in … | `--generate-token`이 데이터 폴더에 `token`을 쓰지 못했습니다: 이 사용자가 폴더에 쓸 수 있게 하십시오 |
| 1 | cannot listen on … | 주소가 이 머신의 것이 아니거나 포트가 사용 중입니다 |
| 1 | the data folder … must be writable by this user | Docker에서 바인드 마운트한 폴더는 uid 10001이 쓸 수 있어야 합니다 |

`--generate-token`이 만든 토큰은 처음 시작할 때 한 번 출력되며(`docker logs signallab`가
보여줌), 데이터 폴더의 `token`에 보관됩니다:
`docker exec signallab cat /data/token`. [서버](../server/index.md)를 참고하십시오.

## 서버에 로그인할 수 없음 {#sign-in}

| 보이는 것 | 이유 | 해결 |
| --- | --- | --- |
| *토큰이 올바르지 않습니다.* | 틀린 토큰 | 서버가 보관하는 곳에서 다시 복사하십시오. 답은 일부러 1초가 걸립니다 |
| `auth.host` | 서버가 주소 표시줄의 이름에 응답하지 않음 | 서버가 받아들이는 이름으로 여십시오. 루프백 이름(`localhost`, `127.x.x.x`, `[::1]`)은 항상 통과합니다. 토큰이 없으면 서버는 그것들과 `--allowed-host`의 이름에만, 토큰이 있으면 `--allowed-host`가 좁히지 않는 한 모든 이름에 응답합니다 |
| `auth.origin` | 다른 오리진의 페이지에서 요청이 옴 | 리버스 프록시 뒤에서는 브라우저의 `Host`를 서버로 전달하십시오(nginx: `proxy_set_header Host $host;`), 그래야 `Origin`과 `Host`가 일치합니다 |
| 로그인 후 로그인 페이지로 돌아옴 | 브라우저가 세션 쿠키를 보관하지 않음 | `--secure-cookie`를 쓰면 서버에 HTTPS로 접근해야 합니다 |
| 잠시 후 로그아웃됨 | 세션은 7일 동안 지속되고 서버가 다시 시작하면 끝납니다. 1024개를 넘으면 가장 오래된 것이 사라집니다 | 다시 로그인하십시오 |

## 서버 연결이 계속 끊김 {#connection-lost}

[[ui:app.connectionLost]]는 페이지의 이벤트 소켓 `/api/events`가 닫혔다는 뜻입니다. 페이지는
반 초 뒤, 그다음 더 드물게 최대 15초마다 스스로 다시 연결합니다. 그동안 일어난 일은 재생되지
않습니다. 실행 중인 작업은 계속 진행되며 다시 보고합니다. 리버스 프록시 뒤에서는
`/api/events`의 WebSocket 업그레이드를 전달하고 20초 이내에 조용한 연결을 닫지 않게
하십시오(서버는 20초마다 핑을 보냅니다). 서버에 그런 주소가 없다고(`api.not_found`) 말하는
페이지는 서버보다 오래된 것입니다: 다시 불러오십시오.

## MQTT가 연결되지 않음 {#mqtt}

Signal Lab은 평문 TCP로 MQTT 3.1.1을 사용합니다. 성공을 보고하기 전에 연결하고 브로커의
응답(CONNACK)을 기다리므로, 이유는 연결하는 버튼에 있습니다:

| 코드 | 이유 | 해결 |
| --- | --- | --- |
| `transport.refused` | 그 포트에서 아무것도 수신하지 않음 | 포트를 확인하십시오: 보통 1883입니다 |
| `transport.timeout` | 6초 이내에 TCP 연결 없음 | 주소, 네트워크, 브로커의 방화벽을 확인하십시오 |
| `transport.dns` | 호스트 이름이 확인되지 않음 | 이름을 확인하거나 주소를 사용하십시오 |
| `transport.unreachable` | 브로커로 가는 경로 없음 | 네트워크와 주소를 확인하십시오 |
| `transport.reset` | 브로커가 즉시 연결을 닫음 | 흔히 TLS 포트(8883)입니다 — Signal Lab은 MQTT over TLS를 사용하지 않습니다 |
| `mqtt.no_answer` | 포트는 열려 있지만 6초 이내에 CONNACK이 없음 | 흔히 WebSocket 포트입니다 — Signal Lab은 MQTT over WebSocket을 사용하지 않습니다 |
| `mqtt.protocol` | 응답한 것이 MQTT 브로커가 아님 | 포트를 확인하십시오 |
| `mqtt.refused_protocol` | 브로커가 MQTT 3.1.1을 받아들이지 않음 | 브로커에서 3.1.1을 켜십시오 |
| `mqtt.refused_client_id` | 브로커가 클라이언트 id를 거부함 | 다른 클라이언트 id를 사용하십시오 |
| `mqtt.refused_unavailable` | 브로커를 사용할 수 없음 | 나중에 다시 시도하십시오 |
| `mqtt.refused_credentials` | 사용자 이름이나 비밀번호가 틀림 | 확인하십시오 |
| `mqtt.refused_not_authorized` | 사용자가 연결할 수 없음 | 브로커의 액세스 규칙을 확인하십시오 |
| `mqtt.client_id_required` | 클라이언트 id가 비어 있음 | 채우십시오 |

같은 클라이언트 id의 두 연결 중 브로커는 오래된 것을 끊습니다. 연결이 계속 닫히면 같은 id를
쓰는 다른 클라이언트를 찾아보십시오.

## 인증서를 신뢰하지 않음 {#tls}

`https://` 또는 `wss://`에서 `transport.tls`: 서버의 인증서를 신뢰할 수 없거나, 요청한
호스트를 지정하지 않거나, TLS를 합의할 수 없습니다. Signal Lab은 시스템과 같은 방식으로
인증서를 확인하며 검사를 건너뛰는 스위치가 없습니다. HTTPS와 WSS는 같은 인증서를 신뢰합니다:
엔진이 실행되는 운영 체제의 것 — Windows에서는 Windows 인증서 저장소, Linux와 서버
이미지에서는 시스템의 CA 인증서. 자체 서명 인증서나 직접 만든 CA는 그 머신의 신뢰할 수 있는
인증서에 추가하고(서버 이미지라면 인증서를 추가한 이미지를 만들어), 인증서가 지닌 이름으로
연결하십시오.

## 시크릿이 없음 {#secrets}

`secret.missing`: 실험이 `{{secret.NAME}}`을 사용하는데 실행되는 곳에 그 이름으로 저장된 값이
없습니다.

- **Windows의 데스크톱 앱**: 편집기의 [[ui:exp.params]]에 있는 [[ui:exp.secrets]]에서
  설정하십시오. Windows 자격 증명 관리자에 보관되므로 새 머신에서는 다시 설정해야 합니다.
- **서버**: `/run/secrets/signallab/NAME` 파일(또는 `--secrets-dir`가 지정한 폴더)이나 환경
  변수 `SIGNALLAB_SECRET_NAME`에 값을 넣으십시오. 서버는 페이지에서 시크릿을 설정할 수
  없습니다(`secret.read_only`).
- **Linux의 데스크톱 앱**에는 시크릿 저장소가 없습니다(`secret.unsupported`). 그런 실험은
  파일과 변수에서 시크릿을 읽는 `signallab run`으로, 또는 서버에서 실행하십시오.

[파일](files.md#secrets)을 참고하십시오.

## 실행이 5분 후 멈춤 {#run-timeout}

300초보다 오래 걸리는 실행은 `run.timeout`으로 실패합니다. 이것이 실행이 걸릴 수 있는 최장
시간입니다. API([`/api/run`](../api/run.md#request)의 `timeout`)나 명령줄
(`signallab run --timeout`)을 통해 실행에 더 짧은 제한을 줄 수 있습니다.

## 앱이 업데이트되지 않음 {#updates}

- 데스크톱 앱은 [[ui:update.auto]]가 켜져 있는 동안 하루에 한 번, 그리고 [[ui:about.open]]에서
  [[ui:update.check]]를 누를 때 새 버전을 찾습니다.
- 스튜디오의 허브에 먼저 묻고, 허브에 닿을 수 없으면 GitHub에 묻습니다. 네트워크가 둘 다
  막으면 [[ui:update.check]]가 [[ui:update.checkFailed]]를 냅니다. 매일의 확인은 아무 말 없이
  실패합니다.
- 게시된 릴리스만 제안되며, 초안이나 사전 릴리스는 절대 안 됩니다.
- 새 릴리스는 허브가 제안할 때 앱에 닿으며, GitHub에 나타난 뒤 한참 뒤일 수 있습니다. 허브는
  릴리스를 한 번에 설치 일부에 배포합니다.
- [[ui:update.install]]을 누를 때만 설치되며 — 실행 중인 작업은 먼저 중지됩니다 — 서명이
  확인된 릴리스만 설치됩니다. 아니면 [[ui:update.installFailed]]입니다.
- 서버는 이미지와 함께 업데이트됩니다: compose 파일이 있는 폴더에서
  `docker compose pull && docker compose up -d`.

## 로그가 있는 곳 {#logs}

- **데스크톱 앱**: 로그 파일을 쓰지 않습니다. 하단 패널의 [[ui:console.title]] 탭이 각 도구가
  한 일과 무엇이 잘못되었는지를 나열하며, [[ui:feedback.open]]이 이를 개발자에게 보내는
  메시지에 첨부합니다(이 컴퓨터의 이름, 주소, 사용자의 폴더 없이). 각 실행의 보고서는 데이터
  폴더의 `runs/`에 있습니다.
- **서버**: 표준 출력과 오류에 기록합니다 — Docker에서는 `docker logs signallab`. 모든 작업
  시작이 요청한 클라이언트의 주소와 함께 기록됩니다. `--log`(또는 `SIGNALLAB_LOG`)가 수준을
  설정합니다: `error`, `warn`, `info`(기본값), `debug`; `--log-format json`(또는
  `SIGNALLAB_LOG_FORMAT`)은 한 줄에 JSON 객체 하나를 씁니다.
- **명령줄**: `signallab`은 메시지를 오류 출력에 씁니다. [명령줄](../automation/cli.md)을
  참고하십시오.

---
title: 노드
description: 실험 노드의 모든 종류 — 하는 일, 기본값과 한도가 있는 필드, 출력, 받는 설정, 그리고 실험 파일에서 어떻게 보이는지.
---

# 노드 참조

실험이 담을 수 있는 모든 종류의 노드를 추가 메뉴의 그룹별로 정리했습니다:
[동작](#actions), [대기](#waits), [에뮬레이션](#emulation), [장애](#faults),
[데이터](#data), [검사](#checks), [흐름](#flow). 추가하고 연결하는 방법은
[편집기](index.md)에 있으며, `signallab nodes`는 같은 목록을 JSON으로 출력해
스크립트와 어시스턴트가 쓸 수 있습니다([명령줄](../automation/cli.md)).

## 이 페이지 읽기 {#reading}

각 노드에는 필드 표가 있습니다:

- **필드**는 속성 창에 표시되는 이름이고, **파일에서**는 실험 JSON의 키입니다.
- **기본값**은 편집기에서 노드를 추가할 때 가지는 값입니다. 파일에서 키를 생략할 수
  있는 경우, 그때 가지는 값을 *없으면*으로 표시하며, 나머지 키는 파일에서 필수입니다.
- **템플릿**: *예* — 필드에 `{{templates}}`를 쓸 수 있습니다. 매개변수, 앞서 설정한
  변수, 시크릿, 생성기이며, 단계가 실행될 때 해석됩니다([데이터와
  템플릿](data.md)). *매개변수만* — 첫 단계 전에 열리므로 매개변수만 알 수 있습니다.
  *아니요* — 쓴 그대로 값을 씁니다.

시간은 밀리초 단위입니다. 한도는 실행이 시작되기 전에 검사하며, 범위를 벗어난 필드는
실험이 실행되지 않게 하고 노드에 표시됩니다.

## 파일 안의 노드 {#file-shape}

실험 파일에서 노드는 `id`(실험 안에서 고유), `type`, 캔버스에서의 위치(`x`, `y`,
0 이상), 필드, 사용하는 설정(`retry`, `repeat`, `load`, 꺼져 있으면 생략)을 가진
객체입니다. 와이어는 한 노드의 출력(`port`, 없으면 `next`)에서 다른 노드로 가는
에지입니다:

```json
{
  "nodes": [
    { "id": "start", "type": "start", "x": 40, "y": 80 },
    { "id": "ping", "type": "udp", "x": 270, "y": 80, "target": "127.0.0.1:9000", "text": "PING",
      "retry": { "attempts": 3, "delay_ms": 500, "backoff": "fixed" } },
    { "id": "end", "type": "end", "x": 500, "y": 80 }
  ],
  "edges": [
    { "from": "start", "to": "ping", "port": "next" },
    { "from": "ping", "to": "end", "port": "next" }
  ]
}
```

아래 예제는 각각 노드 하나를 파일에 담긴 모습 그대로 보여 줍니다.

## 여러 노드가 공유하는 설정 {#settings}

이 설정은 노드 속성의 아래쪽에서 켭니다. 어떤 노드가 어떤 설정을 받는지는 각 노드
아래에 적혀 있습니다.

| 설정 | 받는 노드 | 하는 일 |
| --- | --- | --- |
| [재시도](#retry) | 보내거나 수신하는 노드: [[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]], [[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_connect]], [[ui:exp.node.ws_send]], 그리고 모든 대기 | 단계가 실패하면 다시 시도합니다 |
| [반복](#repeat) | 보내는 노드: [[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]], [[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_send]] | 여러 번 또는 일정 시간 동안 계속 보냅니다 |
| [부하](#load) | [[ui:exp.node.http]] | 부하 프로필에 따라 요청을 보내고, 측정하여 임계값으로 판정합니다 |
| [회신 대기](#reply) | [[ui:exp.node.osc]], [[ui:exp.node.udp]] | 같은 단계에서 보내고 답을 기다립니다 |

### 재시도 {#retry}

[[ui:exp.retryOn]]: 단계가 실패하면 — 연결되지 않거나, 시간 초과이거나, 대기에서
아무것도 일치하지 않으면 — 잠시 멈췄다가 다시 실행합니다. 실패한 시도마다 타임라인에
행이 하나씩 생기며, 마지막 시도까지 실패하면 단계가 실패합니다. 해석할 수 없는
템플릿은 다시 시도하지 않습니다. [[ui:common.stop]]도 멈춤을 끝냅니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 |
| --- | --- | --- | --- |
| [[ui:exp.attempts]] | `retry.attempts` | 첫 시도를 포함한 전체 시도 횟수 | 3; 편집기에서는 2–10(파일은 1도 가능) |
| [[ui:exp.retryDelay]] | `retry.delay_ms` | 두 번째 시도 전의 멈춤 | 500; 0–60 000 |
| [[ui:exp.backoff]] | `retry.backoff` | [[ui:exp.backoff.fixed]] (`fixed`): 매번 같은 시간만큼 멈춤; [[ui:exp.backoff.exponential]] (`exponential`): 실패할 때마다 두 배로 김 | `fixed` (없으면 이것) |

아무리 두 배가 되어도 멈춤이 60초를 넘지 않습니다. [[ui:exp.portTimeout]] 출력에
와이어가 있는 대기는 시간 초과로 실패하지 않고 그 출력으로 빠져나가므로, 그때는 다시
시도하지 않습니다.

### 반복 {#repeat}

[[ui:exp.repeatOn]]: 노드가 그래프의 루프 없이 계속 보냅니다 — 하트비트, 폴링,
지속적인 스트림처럼. 보낼 때마다 템플릿을 새로 읽으며(`{{counter}}`는 번호,
`{{now}}`는 시각), 재시도가 켜져 있으면 보낼 때마다 적용됩니다. 모든 전송이 통과하면
단계가 통과하고, 끝내 실패하는 전송이 있으면 단계가 실패합니다. 타임라인은 1초에
최대 한 번 진행을 보고합니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 |
| --- | --- | --- | --- |
| [[ui:exp.repeatBy]] | `repeat.until` | [[ui:exp.repeatBy.count]] (`count`) 또는 [[ui:exp.repeatBy.duration]] (`duration`) | `count` (없으면 이것) |
| [[ui:exp.repeatCount]] | `repeat.count` | 첫 전송을 포함한 전체 전송 횟수 | 10 (없으면 이것); 2–10 000 |
| [[ui:exp.repeatDuration]] | `repeat.duration_ms` | 첫 전송부터 계속 보낼 시간 | 10 000 (없으면 이것); 1–300 000 |
| [[ui:exp.repeatInterval]] | `repeat.interval_ms` | 두 전송 사이의 멈춤 | 1 000; 10–60 000; 파일에서 필수 |
| [[ui:exp.repeatJitter]] | `repeat.jitter_ms` | 멈춤마다 최대 이만큼 더 길어지며, 실행의 시드에서 뽑음 | 0 (없으면 이것); 0–60 000 |

반복은 실행의 300초 안에 들어가야 하며, *일정 시간 동안*은 전송이 10 000회
미만이어야 합니다(시간을 간격으로 나눈 값).

### 부하 {#load}

[[ui:exp.loadOn]], [[ui:exp.node.http]]에서만: 요청을 프로필에 따라 보냅니다 — 일정한
속도, 램프, 계단, 스파이크 또는 무작위 도착 — 동시에 최대 512개까지 진행하며(기본
32개), 지연 시간, 오류, 달성한 속도를 측정합니다. 임계값이 단계의 통과 여부를
정합니다. 부하는 반복과 재시도를 대신하며(실패한 요청은 세기만 하고 다시 시도하지
않음), 뒤따르는 검사에 남길 응답이 없습니다. 필드와 결과는 [부하
테스트](load.md)에 있습니다.

### 회신 대기 {#reply}

[[ui:exp.expectReply]], [[ui:exp.node.osc]]나 [[ui:exp.node.udp]]에서: 회신을
기다리는 포트에서 메시지를 보내므로, 보낸 쪽에 답하는 장치의 응답을 들을 수 있고,
일치하는 회신이 제때 도착해야만 단계가 통과합니다. 회신이 없으면 단계가 실패하며,
재시도가 다시 보냅니다. 회신은 대기 노드처럼 변수에 저장됩니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.replyOn]] | `reply.bind` | 보내고 수신할 `IP:port`; 포트 0은 아무 빈 포트나 사용 | `0.0.0.0:0` | 아니요 |
| [[ui:exp.replyAddress]] (OSC) | `reply.address` | 회신의 주소 패턴, [[[ui:exp.node.wait_osc]]](#node-wait_osc)에서처럼 | `/*` | 예 |
| [[ui:exp.argRules]] (OSC) | `reply.args` | 인수 규칙, [[ui:exp.node.wait_osc]]에서처럼 | 없음; 최대 16개 | 값: 예 |
| [[ui:exp.replyMode]] (UDP) | `reply.mode` | `any`, `contains`, `regex` 또는 `hex` — [페이로드 일치](#payload-matching) 참고 | `any` (없으면 이것) | 아니요 |
| [[ui:field.pattern]] (UDP) | `reply.pattern` | 회신이 포함하거나 일치해야 하는 것 | 비어 있음; `any`가 아니면 필수 | 예 |
| [[ui:exp.waitTimeout]] | `reply.timeout_ms` | 기다릴 시간 | 2 000 (없으면 이것); 1–120 000 | 아니요 |
| [[ui:exp.replyVariable]] | `reply.variable` | 회신을 저장할 변수 | `reply` (없으면 이것) | 아니요 |

회신의 포트는 대기 노드처럼 첫 단계 전에 열립니다.

## 동작 {#actions}

보내는 노드입니다. 동작 뒤의 대기는 동작이 시작된 순간부터 메시지를 셉니다.

### HTTP 요청 {#node-http}

HTTP 요청 하나를 보내고, 뒤따르는 검사, 분기, [[ui:exp.node.extract]] 노드를 위해
응답을 보관합니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.method]] | `request.method` | GET, HEAD, POST, PUT, PATCH, DELETE 또는 OPTIONS(파일은 아무 메서드나 쓸 수 있음) | `GET` | 아니요 |
| URL | `request.url` | `http://` 또는 `https://` URL | `http://127.0.0.1:8080/` | 예 |
| [[ui:common.timeoutMs]] | `request.timeout_ms` | 교환 전체에 대해 | 4 000 (없으면 10 000); 1–120 000 | 아니요 |
| [[ui:exp.headers]] | `request.headers` | `[[name, value], …]`; 이름이 빈 행은 건너뜀 | 없음 | 예, 이름과 값 |
| [[ui:exp.body]] | `request.body` | 텍스트, 또는 없으면 `null` | `null` | 예 |
| [[ui:field.auth]] | `request.auth` | [[ui:http.auth.none]], [[ui:http.auth.basic]], [[ui:http.auth.bearer]] 또는 [[ui:http.auth.digest]], [[ui:field.username]]과 [[ui:field.password]], 또는 [[ui:field.token]]과 함께 | 없음 | 예 |

- 404와 500을 포함해 어떤 응답이든 단계를 통과합니다: 상태는
  [[[ui:exp.node.assert_status]]](#node-assert_status)로 검사하거나
  [[[ui:exp.node.branch_status]]](#node-branch_status)로 분기하십시오. 답을 받지
  못한 요청 — 거부됨, 시간 초과, 이름이 확인되지 않음, 신뢰할 수 없는 인증서 —
  은 단계를 실패시킵니다.
- 리다이렉트는 최대 열 번까지 따라갑니다. `https://` 인증서는 검증합니다.
- 응답 본문은 검사를 위해 최대 256 KiB까지 보관하며, 더 큰 본문은 그 지점에서
  잘립니다(찾는 것이 잘린 뒤에 있을 수 있으면 검사가 그렇게 알려 줍니다).
- Digest는 서버의 401 챌린지에 답하고 요청을 다시 보냅니다. 자격 증명은 요청에만
  들어갑니다: 단계, 보고서, 인스펙터는 `Authorization` 헤더를 절대 보여 주지
  않습니다. 비밀번호는 `{{secret.NAME}}`으로 쓰십시오.
- 실험이 쿠키를 보관하는 동안([[ui:exp.params]] 아래에서 기본으로 켜짐), 서버가
  설정한 쿠키는 실행의 이후 요청에 함께 돌아갑니다.

출력: [[ui:exp.outputPort]]. 설정: 재시도, 반복, 부하.

```json
{ "id": "cue", "type": "http", "x": 270, "y": 80,
  "request": { "method": "POST", "url": "{{api}}/cue", "headers": [["Content-Type", "application/json"]],
               "body": "{\"cue\": 1}", "timeout_ms": 5000,
               "auth": { "scheme": "bearer", "token": "{{secret.API_TOKEN}}" } } }
```

[HTTP](../protocols/http.md)도 참고하십시오.

### TCP 메시지 {#node-tcp}

TCP로 호스트에 연결하고, 페이로드를 쓰고, 답의 첫 바이트를 최대 250 ms
기다린 뒤(한 번, 최대 1 024바이트를 읽음) 연결을 닫습니다. 답의 크기는 보고만 하고
검사하지는 않습니다.

[인스펙터](../tools/inspector.md)에서 이 단계는 출처가 `experiment`인 두 개의 `tcp`
프레임입니다: 쓴 페이로드와, 답이 왔으면 읽은 답입니다. 사용 중인 시크릿은 여느
프레임처럼 둘 다에서 마스킹됩니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.host]] | `host` | 호스트 이름 또는 IP 주소 | `127.0.0.1` | 예 |
| [[ui:exp.port]] | `port` | | 9000; 1–65 535 | 아니요 |
| [[ui:common.timeoutMs]] | `timeout_ms` | 연결, 쓰기, 답을 합쳐서 | 4 000 (없으면 이것); 1–120 000 | 아니요 |
| [[ui:exp.payload]] | `payload` | 연결되면 UTF-8로 한 번 쓰는 텍스트 | `hello` | 예 |

연결이 거부되거나, 이름이 확인되지 않거나, 시간이 다 되면 단계가 실패합니다. 출력:
[[ui:exp.outputPort]]. 설정: 재시도, 반복. [[ui:exp.sendNow]]는 한 번 연결해
페이로드를 쓰며, 노드의 결과가 몇 바이트를 보내고 돌려받았는지 알려 줍니다.

```json
{ "id": "go", "type": "tcp", "x": 270, "y": 80, "host": "127.0.0.1", "port": 5000, "payload": "GO\r\n", "timeout_ms": 2000 }
```

### OSC 메시지 {#node-osc}

OSC 1.0 메시지 하나를 UDP로 보냅니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:common.target]] | `target` | `IP:port` 또는 `host:port`; 호스트 이름은 단계가 보낼 때 조회하며, IPv4 주소가 있으면 그것을 씁니다 | `127.0.0.1:9000` | 예 |
| [[ui:common.address]] | `address` | `/`로 시작 | `/test` | 예 |
| [[ui:exp.arguments]] | `args` | `[{ "type", "value" }, …]` — `int`, `float`, `str`, `long`, `double`, `bool`, `blob`(바이트), `nil`(값 없음) | 없음 | 텍스트(`str`) 값: 예 |
| [[ui:exp.expectReply]] | `reply` | 선택 사항: 보내고 답을 기다림 — [회신 대기](#reply) 참고 | 꺼짐 | |

출력: [[ui:exp.outputPort]]; 회신을 기대하면, 회신이 왔을 때만 이어집니다. 설정:
재시도, 반복, 회신. ⚡ 속성의 [[ui:exp.routeThrough]]는 노드 앞에
[[[ui:exp.node.impairment]]](#node-impairment)를 둡니다.

```json
{ "id": "fader", "type": "osc", "x": 270, "y": 80, "target": "{{device}}", "address": "/fader/1",
  "args": [{ "type": "float", "value": 0.75 }] }
```

[OSC](../protocols/osc.md)도 참고하십시오.

### UDP 데이터그램 {#node-udp}

텍스트 페이로드를 UDP 데이터그램 하나로 하나 이상의 대상에 보냅니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:common.target]] | `target` | `IP:port` 또는 `host:port`; 쉼표, 세미콜론, 줄바꿈으로 구분한 여러 대상은 각각 데이터그램을 받습니다. 호스트 이름은 단계가 보낼 때 조회하며, IPv4 주소가 있으면 그것을 씁니다 | `127.0.0.1:9000` | 예 |
| [[ui:exp.payload]] | `text` | UTF-8로 된 페이로드 | `hello`; 최대 65 507바이트 | 예 |
| [[ui:exp.expectReply]] | `reply` | 선택 사항: 보내고 답을 기다림 — [회신 대기](#reply) 참고 | 꺼짐 | |

대상 하나라도 도달할 수 없으면 단계가 실패합니다. 출력: [[ui:exp.outputPort]].
설정: 재시도, 반복, 회신.

```json
{ "id": "ping", "type": "udp", "x": 270, "y": 80, "target": "{{device}}", "text": "PING {{run.id}}",
  "reply": { "bind": "0.0.0.0:0", "mode": "contains", "pattern": "PONG", "timeout_ms": 1000, "variable": "pong" } }
```

### MQTT 발행 {#node-mqtt}

MQTT 브로커에 연결하고, 메시지 하나를 발행한 뒤 연결을 끊습니다. 연결은 일반 TCP
위의 MQTT 3.1.1이며, clean session이고 사용자 이름도 비밀번호도 없습니다. 연결,
발행, 브로커의 승인이 모두 15초 안에 일어나야 합니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.broker]] | `host` | 브로커의 호스트 이름 또는 주소 | `127.0.0.1` | 예 |
| [[ui:exp.port]] | `port` | | 1883; 1–65 535 | 아니요 |
| [[ui:exp.topic]] | `topic` | 와일드카드(`+`, `#`) 없음 | `lab/test` | 예 |
| [[ui:exp.payload]] | `payload` | 텍스트로 된 메시지 | `hello` | 예 |
| QoS | `qos` | 0, 1 또는 2 | 0 | 아니요 |
| [[ui:exp.retain]] | `retain` | `true`: 브로커가 토픽의 값으로 보관 | `false` | 아니요 |

파일에서는 여섯 키가 모두 필수입니다. 브로커에 도달할 수 없거나 연결이나 메시지를
거부하면 단계가 실패합니다. 출력: [[ui:exp.outputPort]]. 설정: 재시도, 반복.

```json
{ "id": "light", "type": "mqtt", "x": 270, "y": 80, "host": "{{broker}}", "port": 1883,
  "topic": "lab/light/1/set", "payload": "on", "qos": 1, "retain": false }
```

[MQTT](../protocols/mqtt.md)도 참고하십시오.

### WebSocket 연결 {#node-ws_connect}

실행이 끝날 때까지, 또는 [[[ui:exp.node.ws_close]]](#node-ws_close)까지
WebSocket을 엽니다. 그때부터 도착하는 것은 그 위의
[[[ui:exp.node.wait_ws]]](#node-wait_ws) 단계를 위해 보관됩니다. URL과 헤더는 단계가
실행될 때 해석되므로, 앞서 추출한 토큰을 넣을 수 있습니다. [[ui:exp.node.loop]]
안에서 다시 실행하면 이전 연결을 먼저 닫고 새로 엽니다. 실행이 어떻게 끝나든 연결은
close 프레임과 함께 닫힙니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| URL | `url` | `ws://` 또는 `wss://` URL | `ws://127.0.0.1:9001/` | 예 |
| [[ui:exp.headers]] | `headers` | 업그레이드 요청과 함께 보내는 `[[name, value], …]` | 없음 | 예, 이름과 값 |
| [[ui:exp.wsProtocols]] | `protocols` | 선호 순서대로 제안할 하위 프로토콜; 서버가 하나를 고름 | 없음 | 아니요 |
| [[ui:common.timeoutMs]] | `timeout_ms` | 연결과 업그레이드에 대해 | 5 000 (없으면 10 000); 1–120 000 | 아니요 |

`wss://`는 `https://`와 같은 인증서를 신뢰합니다. 연결이나 업그레이드가 실패하면
단계가 실패하며, 서버의 상태가 이유에 들어 있습니다. 출력: [[ui:exp.outputPort]].
설정: 재시도(반복 아님).

```json
{ "id": "socket", "type": "ws_connect", "x": 270, "y": 80, "url": "ws://127.0.0.1:9001/chat",
  "headers": [["Authorization", "Bearer {{token}}"]], "protocols": ["chat.v1"], "timeout_ms": 5000 }
```

[WebSocket](../protocols/websocket.md)도 참고하십시오.

### WebSocket 보내기 {#node-ws_send}

[[ui:exp.node.ws_connect]]가 연 연결로 메시지 하나를 보냅니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | 이 실험의 [[ui:exp.node.ws_connect]] 노드 id | 첫 번째 것 | 아니요 |
| [[ui:exp.wsFormat]] | `binary` | [[ui:exp.wsText]] (`false`) 또는 [[ui:exp.wsBinary]] (`true`): 페이로드는 hex로 쓴 바이트, `de ad be ef` | `false` (없으면 이것) | 아니요 |
| [[ui:exp.payload]] | `text` | 메시지 | `hello`; 최대 16 MiB | 예 |

연결은 그 경로에서 보내기보다 앞서야 하며, 연결이 열려 있지 않은 보내기는
실패합니다. 답은 메시지를 쓴 순간부터 셉니다. 출력: [[ui:exp.outputPort]]. 설정:
재시도, 반복.

```json
{ "id": "hello", "type": "ws_send", "x": 500, "y": 80, "connection": "socket",
  "text": "{\"type\":\"ping\",\"id\":\"{{uuid}}\"}", "binary": false }
```

### WebSocket 닫기 {#node-ws_close}

close 핸드셰이크로 연결을 닫습니다. 타임라인은 누가 닫았는지 알려 줍니다: 이 단계,
앞서 서버(코드와 함께), 또는 이미 끊어진 연결입니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | [[ui:exp.node.ws_connect]] 노드 id | 첫 번째 것 | 아니요 |
| [[ui:field.code]] | `code` | 1000(정상), 또는 애플리케이션 고유의 3000–4999 | 1000 (없으면 이것) | 아니요 |
| [[ui:field.reason]] | `reason` | 코드와 함께 보냄 | 비어 있음; 템플릿을 적용한 뒤 최대 123바이트 | 예 |

출력: [[ui:exp.outputPort]]. 설정 없음.

```json
{ "id": "bye", "type": "ws_close", "x": 960, "y": 80, "connection": "socket", "code": 1000, "reason": "done" }
```

### 로그 표시 {#node-log}

타임라인과 보고서에 한 줄을 씁니다 — 체크포인트, 또는 실행이 도달한 값입니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.logMessage]] | `message` | 텍스트 | `Check point`; 최대 10 000자 | 예 |

출력: [[ui:exp.outputPort]]. 설정 없음.

```json
{ "id": "ready", "type": "log", "x": 500, "y": 80, "message": "device {{device}} ready" }
```

## 대기 {#waits}

[[ui:exp.group.observe]] 그룹: 무언가 도착하기를 기다리는 노드입니다. 다음과 같은
규칙을 공유합니다:

- **실행이 시작될 때부터 수신합니다.** 대기의 포트나 브로커 구독은 첫 단계 전에
  열리므로, 다음 단계가 시작되기보다 빠르게 답하는 장치를 놓치지 않습니다. 같은
  주소의 대기 두 개는 소켓 하나를 공유합니다.
- **자기 분기의 마지막 동작부터 셉니다.** 분기의 마지막 요청 전에 도착한 메시지는
  그 요청의 답이 아닙니다. 동작이 하나도 없으면 실행이 시작된 이후의 모든 것이
  해당합니다.
- **처음 일치하는 메시지를 가져갑니다.** 한 대기가 가져간 메시지는 다른 대기가 보지
  못합니다.
- **[[ui:exp.portMatched]] 또는 [[ui:exp.portTimeout]].** 일치하면 메시지가 대기의
  변수에 저장되고 흐름은 [[ui:exp.portMatched]]를 따릅니다. 시간이 다 되면, 그
  출력에 와이어가 있으면 [[ui:exp.portTimeout]]을 따르고, 없으면 단계가 실패하면서
  다른 메시지가 몇 개 도착했는지 알려 줍니다.
- 각 소켓은 최신 메시지 1 024개(그리고 64 MiB)를 보관하며, 더 오래된 것은 버려지고,
  시간 초과 시 몇 개였는지 알려 줍니다.
- [[ui:exp.listenNow]]는 그 한 단계만 지금부터 수신합니다.

출력: [[ui:exp.portMatched]] (필수), [[ui:exp.portTimeout]] (선택). 설정: 재시도.

### 페이로드 일치 {#payload-matching}

[[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_ws]]와 UDP
회신은 페이로드가 어떻게 생겨야 하는지 고릅니다:

| 옵션 | 파일에서 | 페이로드가 이럴 때 일치 |
| --- | --- | --- |
| [[ui:exp.mode.any]] | `any` | 무엇이든 |
| [[ui:exp.mode.contains]] | `contains` | UTF-8 텍스트로 읽어 패턴을 포함(대소문자 구분) |
| [[ui:exp.mode.regex]] | `regex` | UTF-8 텍스트로 읽어 정규식과 일치 |
| [[ui:exp.mode.hex]] | `hex` | hex 쌍으로 쓴 바이트를 포함: `de ad be ef`, `deadbeef`, `0xde,0xad`, `DE:AD` |

일치한 메시지는 객체로 저장됩니다. 이후 단계는 그 필드를 `{{reply.text}}`로
읽습니다(`reply` 자리에 변수 이름):

| 필드 | 내용 |
| --- | --- |
| `text` | 텍스트로 된 페이로드 |
| `hex`, `bytes` | hex로 된 페이로드(처음 1 024바이트)와 바이트 단위 크기 |
| `match` | 일치한 것: 텍스트, 정규식의 첫 번째 그룹(또는 일치한 전체), 또는 바이트 |
| `from` | 보낸 쪽의 `IP:port` |
| `ms` | 분기의 마지막 동작(또는 실행 시작)부터 메시지까지의 밀리초 |
| `topic` | [[ui:exp.node.wait_mqtt]]: 발행된 토픽 |
| `json`, `kind` | [[ui:exp.node.wait_ws]]: JSON으로 파싱한 메시지(아니면 `null`)와 `text` 또는 `binary` |

### OSC 대기 {#node-wait_osc}

주소가 패턴과 일치하고 인수가 모든 규칙을 만족하는 OSC 메시지를 기다립니다.
번들에서는 일치하는 첫 번째 메시지를 가져갑니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | 수신할 `IP:port`; 모든 네트워크 카드는 `0.0.0.0` | `127.0.0.1:9001` | 아니요 |
| [[ui:exp.addressPattern]] | `address` | `*`는 임의의 문자, `?`는 한 글자, `[0-9]`는 문자 집합(`[!0-9]`는 그 밖), `{ping,pong}`은 둘 중 하나; 와일드카드는 하나의 `/` 세그먼트 안에 머묾 | `/pong`; 최대 512자 | 예 |
| [[ui:exp.argRules]] | `args` | `[{ "index", "op", "value" }, …]`: `index` 인수를 `op`로 `value`와 비교([비교](#comparisons)); 모두 성립해야 함 | 없음; 최대 16개, index 0–63 | 값: 예 |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (없으면 이것); 1–120 000 | 아니요 |
| [[ui:exp.replyVariable]] | `variable` | 메시지를 저장할 곳 | `reply` (없으면 이것) | 아니요 |

인수는 텍스트로 비교합니다: 숫자는 쓴 그대로, 문자열은 따옴표 없이, `true`/`false`,
blob은 hex로. 메시지에 없는 인수에 대한 규칙은 성립하지 않습니다. 저장된 메시지에는
`address`, `args` (`{{reply.args[0]}}`), `from`, `ms`가 있습니다.

```json
{ "id": "status", "type": "wait_osc", "x": 500, "y": 80, "bind": "0.0.0.0:9001", "address": "/status",
  "args": [{ "index": 0, "op": "eq", "value": "ready" }], "timeout_ms": 5000, "variable": "reply" }
```

### UDP 대기 {#node-wait_udp}

페이로드가 일치하는 UDP 데이터그램을 기다립니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | 수신할 `IP:port` | `127.0.0.1:9001` | 아니요 |
| [[ui:exp.waitMode]] | `mode` | [페이로드 일치](#payload-matching) 참고 | `contains` (없으면 `any`) | 아니요 |
| [[ui:field.pattern]] | `pattern` | 페이로드가 포함하거나 일치해야 하는 것 | `pong`; `any`가 아니면 필수 | 예 |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (없으면 이것); 1–120 000 | 아니요 |
| [[ui:exp.replyVariable]] | `variable` | | `reply` (없으면 이것) | 아니요 |

```json
{ "id": "ready", "type": "wait_udp", "x": 500, "y": 80, "bind": "0.0.0.0:9002", "mode": "contains",
  "pattern": "READY", "timeout_ms": 5000, "variable": "reply" }
```

### MQTT 대기 {#node-wait_mqtt}

브로커의 토픽에 발행된 메시지 중 페이로드가 일치하는 것을 기다립니다. 실행은 첫
단계 전에 연결하고 구독합니다. 구독할 때 브로커가 재생하는 retained 메시지는
무시하며, 실행이 시작된 뒤 발행된 것만 해당합니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.broker]] | `host` | 브로커 | `127.0.0.1` | 매개변수만 |
| [[ui:exp.port]] | `port` | | 1883; 1–65 535 | 아니요 |
| [[ui:exp.topicFilter]] | `topic` | 필터: `+`는 한 단계, `#`는 그 아래 전부(마지막에만) | `lab/#` | 매개변수만 |
| [[ui:exp.waitMode]] | `mode` | [페이로드 일치](#payload-matching) 참고 | `any` (없으면 이것) | 아니요 |
| [[ui:field.pattern]] | `pattern` | | 비어 있음; `any`가 아니면 필수 | 예 |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (없으면 이것); 1–120 000 | 아니요 |
| [[ui:exp.replyVariable]] | `variable` | | `reply` (없으면 이것) | 아니요 |

```json
{ "id": "state", "type": "wait_mqtt", "x": 500, "y": 80, "host": "{{broker}}", "port": 1883,
  "topic": "lab/+/state", "mode": "contains", "pattern": "on", "timeout_ms": 5000, "variable": "reply" }
```

### HTTP 요청 대기 {#node-wait_http}

웹훅, 콜백 같은 HTTP 요청이 그 주소의 실행의
[[[ui:exp.node.emulator]]](#node-emulator)에 도착하기를 기다리거나, 그 주소에 실행의
HTTP 에뮬레이터가 없으면 모든 요청에 204로 답하는 실행 자체의 리스너에 도착하기를
기다립니다. 요청은 메서드, 경로, 모든 조건과 일치해야 합니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | `IP:port` | `127.0.0.1:18080` — 새 [[ui:exp.node.emulator]]가 수신하는 곳 | 아니요 |
| [[ui:exp.method]] | `method` | 메서드, 또는 [[ui:emu.methodAny]] (`ANY`); GET은 HEAD도 받음 | `ANY` (없으면 이것) | 아니요 |
| [[ui:exp.path]] | `path` | `/hooks/:name`은 한 세그먼트에 이름을 붙임(`{{request.params.name}}`); 마지막 `/*`는 나머지를 받음 | `/*` (없으면 이것); 최대 512자 | 예 |
| [[ui:emu.conditions]] | `when` | `header`, `query` 매개변수, `body` 또는 `json` 경로에 대한 `[{ "on", "name", "op", "value" }, …]`; 모두 성립해야 함 | 없음; 최대 16개 | 예, 이름과 값 |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 5 000 (없으면 2 000); 1–120 000 | 아니요 |
| [[ui:exp.replyVariable]] | `variable` | | `request` (없으면 이것) | 아니요 |

저장된 요청에는 `method`, `path`, `query`, `headers`, `body`, `json`, `params`,
`from`, `ms`가 있습니다: `{{request.json.event}}`, `{{request.headers.x-key}}`.

```json
{ "id": "hook", "type": "wait_http", "x": 500, "y": 80, "bind": "127.0.0.1:18081", "method": "POST",
  "path": "/hooks/:name", "when": [{ "on": "json", "name": "$.event", "op": "eq", "value": "deploy" }],
  "timeout_ms": 5000, "variable": "request" }
```

### WebSocket 대기 {#node-wait_ws}

[[ui:exp.node.ws_connect]]가 연 연결에서 페이로드가 일치하는 메시지를 기다립니다.
분기의 마지막 동작 이후의 메시지가 해당합니다 — 연결 자체, 보내기, 또는 다른
요청.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | [[ui:exp.node.ws_connect]] 노드 id | 첫 번째 것 | 아니요 |
| [[ui:exp.waitMode]] | `mode` | [페이로드 일치](#payload-matching) 참고 | `any` (없으면 이것) | 아니요 |
| [[ui:field.pattern]] | `pattern` | | 비어 있음; `any`가 아니면 필수 | 예 |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (없으면 이것); 1–120 000 | 아니요 |
| [[ui:exp.replyVariable]] | `variable` | | `reply` (없으면 이것) | 아니요 |

JSON 메시지는 필드별로 읽을 수 있습니다: `{{reply.json.type}}`. 연결은 그 경로에서
대기보다 앞서야 합니다.

```json
{ "id": "pong", "type": "wait_ws", "x": 730, "y": 80, "connection": "socket", "mode": "contains",
  "pattern": "pong", "timeout_ms": 3000, "variable": "reply" }
```

## 에뮬레이션 {#emulation}

### 에뮬레이터 {#node-emulator}

실행 내내 의존 대상 — HTTP API, OSC·UDP·TCP 장치, MQTT 브로커 — 의 역할을
맡습니다. 첫 단계 전에 열려 실행이 끝날 때까지 응답하며, 흐름에서는 바로
통과합니다. 받은 내용은 실행 보고서에 규칙별로 집계됩니다.

| 필드 | 파일에서 | 내용 | 기본값 |
| --- | --- | --- | --- |
| [[ui:emu.edit]] | `emulator` | 에뮬레이터: `name`, `bind` (`IP:port`), `protocol` (`http`, `osc`, `udp`, `tcp`, `mqtt`), 라우트 또는 규칙, 선택적 `outage` | `127.0.0.1:18080`에서 `/health`에 응답하는 *API*라는 이름의 HTTP API |

속성은 무엇을 맡는지 한 줄로 보여 줍니다. [[ui:emu.edit]]은 규칙을 열며,
[[[ui:nav.emulators]]](../tools/emulators.md) 화면과 같은 편집기입니다.
[[ui:emu.toLibrary]]는 에뮬레이터 라이브러리에 사본을 보관하고,
[[ui:emu.fromLibrary]]는 라이브러리의 사본으로 이것을 바꿉니다. 규칙 — 라우트,
응답, 장애, 다운 일정 — 은 그곳에 설명되어 있습니다.

- HTTP 에뮬레이터는 그 주소의 [[[ui:exp.node.wait_http]]](#node-wait_http)가 수신하는
  대상이기도 하며, OSC나 UDP 에뮬레이터는 그 포트의 실행 대기 노드와 포트를
  공유합니다.
- 한 실행에서 같은 전송 방식의 에뮬레이터 두 개는 포트를 공유할 수 없습니다.
- [[[ui:exp.node.emulator_state]]](#node-emulator_state)가 다운시키고 다시
  복구합니다.

출력: [[ui:exp.outputPort]]. 설정 없음.

```json
{ "id": "api", "type": "emulator", "x": 270, "y": 80,
  "emulator": { "name": "Orders API", "bind": "127.0.0.1:18080", "protocol": "http",
    "routes": [{ "method": "GET", "path": "/orders/:id", "order": "sequence",
                 "responses": [{ "status": 503 }, { "status": 200, "body": "{\"id\":\"{{request.params.id}}\"}" }] }] } }
```

## 장애 {#faults}

신호에 맞춰 망가뜨리는 노드입니다. [[ui:exp.node.delay]] 노드와 이것들을 트래픽
옆에 나란히 두면 일정처럼 읽히며, [일정에 따른 장애](faults.md)에서 방법을 보여
줍니다.

### 네트워크 장애 {#node-impairment}

실행 내내 동작하는 장애 릴레이입니다: 테스트 대상 시스템은 실제 대상 대신
[[ui:exp.relayListen]]로 보내거나 연결하고, 릴레이는 [[ui:exp.relayTarget]]으로
전달하며, 응답은 같은 길로 프로필에 따라 장애를 입고 돌아옵니다. 첫 단계 전에 열려
실행이 어떻게 끝나든 닫히므로, 장애가 남아 있는 일이 없습니다. 흐름에서는 바로
통과합니다. 모든 결정은 실행의 시드에서 뽑습니다: 같은 시드와 같은 트래픽은 같은
운명을 겪습니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.relayListen]] | `listen` | 테스트 대상 시스템이 보내는 `IP:port` | `127.0.0.1:9010` | 매개변수만 |
| [[ui:exp.relayTarget]] | `target` | 실제 목적지의 `IP:port`, 또는 `host:port` — 호스트 이름은 실행이 시작될 때 조회하며, 찾을 수 없는 이름은 이 노드에서 실행을 멈춤 | `127.0.0.1:9000` | 매개변수만 |
| [[ui:ns.protocol]] | `protocol` | UDP (`udp`): 데이터그램마다 따로 결과가 정해짐; TCP (`tcp`): 연결마다 대상으로 가는 연결을 하나씩 이어 주고, 양쪽 스트림에 장애를 적용 | UDP (없으면 `udp`) | 아니요 |
| [[ui:ns.preset]] 및 그 아래 값 | `profile` | 릴레이가 트래픽에 하는 일 — [프로필](#impair-profile) 참고 | [[ui:ns.preset.lan]] (없으면 장애 없음) | 아니요 |

릴레이의 수신 주소는 실행의 다른 소켓일 수 없으며, 릴레이는 서로에게 순환으로
전달할 수 없습니다. 이름으로 준 대상은 조회된 뒤 따라가므로, 이름을 거친 순환은
실행이 시작될 때 멈춥니다. 보고서는 릴레이의 각 구간을 따로 집계합니다.

출력: [[ui:exp.outputPort]]. 설정 없음.

```json
{ "id": "relay", "type": "impairment", "x": 270, "y": 80, "listen": "127.0.0.1:9010", "target": "{{device}}",
  "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } }
```

#### 프로필 {#impair-profile}

프리셋 칩 — [[ui:ns.preset.lan]], [[ui:ns.preset.wifi]], [[ui:ns.preset.4g]],
[[ui:ns.preset.satellite]], [[ui:ns.preset.intermittent]], [[ui:ns.preset.offline]]
— 이 모든 값을 채웁니다. 이후에 아무 값이나 바꿔도 됩니다. 릴레이는 자기
프로토콜의 값만 읽으며, 파일에서는 모든 키를 생략할 수 있습니다(0, 꺼짐).

| 필드 | 파일에서 | 내용 | 한도 | 프로토콜 |
| --- | --- | --- | --- | --- |
| — | `name` | 타임라인과 보고서에 쓸 이름표: 프리셋의 키 (`lan`, `wifi`, `4g`, `satellite`, `intermittent`, `offline`) 또는 직접 정한 이름 | 최대 60자 | 둘 다 |
| [[ui:ns.offline]] | `offline` | 아무것도 통과하지 못함 | `true` / `false` | 둘 다 |
| [[ui:ns.latency]] | `latency_ms` | 모든 패킷, 또는 스트림의 청크에 더하는 지연 | 0–60 000 (슬라이더는 1 000까지) | 둘 다 |
| [[ui:ns.jitter]] | `jitter_ms` | 최대 이만큼의 무작위 추가 지연; TCP 스트림은 순서를 유지 | 0–60 000 (슬라이더는 500까지) | 둘 다 |
| [[ui:ns.rate]] | `rate_kbps` | 대역폭 제한, 0이면 없음. UDP: 1초 분량 이상 대기열에 쌓이면 데이터그램을 대역 제한으로 버림; TCP: 송신 측이 느려질 뿐 아무것도 버리지 않음 | 0, 또는 8–10 000 000 | 둘 다 |
| [[ui:ns.loss]] | `loss` | 데이터그램이 버려질 확률 | 0–1 (슬라이더는 %로 표시) | UDP |
| [[ui:ns.burst]], [[ui:ns.burstLength]] | `burst_start`, `burst_length` | 손실 버스트가 시작될 확률과, 버스트가 평균 몇 데이터그램 동안 이어지는지 | 0–1; 버스트가 켜져 있으면 1–1 000 | UDP |
| [[ui:ns.duplicate]] | `duplicate` | 데이터그램이 두 번 전송될 확률 | 0–1 | UDP |
| [[ui:ns.corrupt]] | `corrupt` | 데이터그램의 한 비트가 뒤집힐 확률 | 0–1 | UDP |
| [[ui:ns.reorder]] | `reorder` | 데이터그램이 붙잡혀 뒤의 것들이 앞지르게 될 확률 | 0–1 | UDP |
| [[ui:ns.reset]] | `reset` | 스트림의 청크가 대신 연결을 리셋할 확률 — 양쪽 모두 리셋을 받음 | 0–1 | TCP |
| [[ui:ns.stall]] | `stall` | 청크가 연결을 반개방 상태로 남길 확률: 어느 방향으로도 더 이상 통과하지 못하고, 어느 쪽도 통보받지 않음 | 0–1 | TCP |

릴레이, 프리셋, 그것들이 모델링하는 것에 대한 더 자세한 내용은
[[[ui:nav.netsim]]](../tools/impairment.md) 페이지에 있습니다.

### 장애 변경 {#node-impairment_change}

실행의 [[ui:exp.node.impairment]] 노드 하나를 이 단계부터 다른 프로필로 전환하며,
포트는 그대로 유지합니다. 지금까지의 구간은 닫히고 보고서에 집계됩니다.

| 필드 | 파일에서 | 내용 | 기본값 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.relay]] | `relay` | 이 실험의 [[ui:exp.node.impairment]] 노드 id | 첫 번째 것 | 아니요 |
| [[ui:ns.preset]] 및 그 아래 값 | `profile` | 지금부터 무엇으로 장애를 입힐지 — [프로필](#impair-profile) 참고; 릴레이는 자기 프로토콜의 값을 읽음 | [[ui:ns.preset.offline]] (없으면 장애 없음) | 아니요 |

릴레이가 실행 중이 아니면 — 예컨대 중계에 실패했으면 — 단계가 실패합니다. 출력:
[[ui:exp.outputPort]]. 설정 없음.

```json
{ "id": "cut", "type": "impairment_change", "x": 730, "y": 200, "relay": "relay",
  "profile": { "name": "offline", "offline": true } }
```

### 에뮬레이터 다운과 복구 {#node-emulator_state}

실행의 에뮬레이터 하나를 다운시키거나 다시 복구합니다. 다운된 동안 HTTP
에뮬레이터는 [[ui:exp.downFault]]에 따라 응답하고, TCP 장치와 MQTT 브로커는 연결을
끊고 새 연결을 거부하며, OSC와 UDP 장치는 아무것도 응답하지 않습니다. 다시
복구되면 에뮬레이터는 자체 다운 일정이 있으면 그것을 따릅니다.

| 필드 | 파일에서 | 내용 | 기본값 |
| --- | --- | --- | --- |
| [[ui:exp.emulatorNode]] | `emulator` | 이 실험의 [[ui:exp.node.emulator]] 노드 id | 첫 번째 것 |
| [[ui:exp.emulatorDownState]] | `down` | [[ui:exp.emulatorGoesDown]] (`true`) 또는 [[ui:exp.emulatorComesUp]] (`false`) | 다운 (없으면 `false`) |
| [[ui:exp.downFault]] | `fault` | HTTP 전용: [[ui:emu.outageFault.unavailable]] (`unavailable`), [[ui:emu.outageFault.reset]] (`reset`: 응답 없이 연결이 닫힘) 또는 [[ui:emu.outageFault.timeout]] (`timeout`: 클라이언트가 포기할 때까지 요청을 붙잡아 둠, 최대 120초) | `unavailable` (없으면 이것) |

출력: [[ui:exp.outputPort]]. 설정 없음. 템플릿은 쓰지 않습니다.

```json
{ "id": "down", "type": "emulator_state", "x": 500, "y": 200, "emulator": "api", "down": true, "fault": "unavailable" }
```

## 데이터 {#data}

### 값 추출 {#node-extract}

그 경로의 최신 HTTP 응답 일부를 변수로 저장하여, 이후 필드(`{{token}}`), 검사,
분기에서 씁니다. 모든 경로에서 HTTP 요청이 그보다 앞서야 합니다.
[[ui:exp.sendNow]] 응답에서 값을 클릭하면 하나가 대신 추가됩니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.variable]] | `variable` | 이름: 글자, 숫자, `_`, 숫자로 시작하지 않음, 예약어 아님, 매개변수 이름 아님 | `token` | 아니요 |
| [[ui:exp.extractFrom]] | `from` | [[ui:exp.from.json]] (`json`), [[ui:exp.from.header]] (`header`), [[ui:exp.from.status]] (`status`), [[ui:exp.from.body]] (`body`) 또는 [[ui:exp.from.regex]] (`regex`) | `json` | 아니요 |
| [[ui:exp.jsonPath]], [[ui:exp.headerName]] 또는 [[ui:exp.pattern]] | `expr` | JSON 경로 (`$.data.token`, `$.items[0]`, `$["first name"]`), 헤더 이름(대소문자 무관), 또는 정규식 — 첫 번째 그룹, 또는 일치한 전체 | `$.token`; status와 body에는 쓰지 않음 | 아니요 |

가져올 것이 없으면 단계가 실패합니다: 본문이 JSON이 아니거나, 경로나 헤더가
없거나, 표현식이 일치하지 않거나, — JSON 필드나 본문 전체의 경우 — 본문이 보관한
256 KiB보다 길면. status는 숫자로 저장하고, 나머지는 텍스트로, 또는 찾은 JSON
값으로 저장합니다. 출력: [[ui:exp.outputPort]]. 설정 없음.

```json
{ "id": "token", "type": "extract", "x": 500, "y": 80, "variable": "token", "from": "json", "expr": "$.data.token" }
```

변수에 대한 더 자세한 내용은 [데이터와 템플릿](data.md)에 있습니다.

## 검사 {#checks}

검사는 통과하거나 실행을 실패시킵니다. 네 가지 응답 검사는 그 경로의 최신 HTTP
응답을 읽으므로, 모든 경로에서 부하가 아닌 HTTP 요청이 그보다 앞서야 합니다.

### HTTP status {#node-assert_status}

최신 응답의 상태가 주어진 값과 정확히 같으면 통과합니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.expectedStatus]] | `status` | | 200; 100–599 | 아니요 |

출력: [[ui:exp.outputPort]]. 설정 없음.

```json
{ "id": "ok", "type": "assert_status", "x": 500, "y": 80, "status": 200 }
```

### 응답 텍스트 {#node-assert_body}

최신 응답의 본문이 그 텍스트를 정확히(대소문자 포함) 포함하면 통과합니다. 본문은
처음 256 KiB만 보관하므로, 잘린 본문에서 찾지 못한 텍스트는 그 이유로
실패합니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.contains]] | `contains` | | `ok`; 필수 | 예 |

출력: [[ui:exp.outputPort]]. 설정 없음.

```json
{ "id": "ready", "type": "assert_body", "x": 500, "y": 80, "contains": "ready" }
```

### 응답 헤더 {#node-assert_header}

최신 응답에 그 헤더가 있고 그 값이 텍스트를 포함하면 통과합니다. 헤더 이름은
대소문자 무관하게, 값은 정확히 비교합니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.headerName]] | `name` | | `content-type`; 필수 | 예 |
| [[ui:exp.contains]] | `contains` | 그 값이 포함해야 하는 것; 비어 있으면 헤더가 있기만 하면 됨 | `application/json` | 예 |

출력: [[ui:exp.outputPort]]. 설정 없음.

```json
{ "id": "json", "type": "assert_header", "x": 500, "y": 80, "name": "Content-Type", "contains": "json" }
```

### 응답 시간 {#node-assert_latency}

요청을 보낸 순간부터 본문이 끝날 때까지, 최신 응답이 최대 이 시간만 걸렸으면
통과합니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.maxLatency]] | `max_ms` | | 1 000; 1–120 000 | 아니요 |

출력: [[ui:exp.outputPort]]. 설정 없음.

```json
{ "id": "fast", "type": "assert_latency", "x": 500, "y": 80, "max_ms": 250 }
```

### 값 검사 {#node-assert_value}

값 — 보통 템플릿으로 쓴 변수 — 을 기대값과 비교하여, 비교가 성립하면
통과합니다.

| 필드 | 파일에서 | 내용 | 기본값 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.value]] | `value` | 비교할 것: `{{token}}`, `{{reply.args[0]}}` | `{{token}}` | 예 |
| [[ui:exp.operator]] | `op` | [비교](#comparisons) 참고 | [[ui:exp.op.not_empty]] | 아니요 |
| [[ui:exp.expected]] | `expected` | [[ui:exp.op.empty]]와 [[ui:exp.op.not_empty]]는 쓰지 않음 | 비어 있음 (없으면 이것) | 예 |

출력: [[ui:exp.outputPort]]. 설정 없음.

```json
{ "id": "state", "type": "assert_value", "x": 730, "y": 80, "value": "{{state}}", "op": "eq", "expected": "ready" }
```

### 비교 {#comparisons}

[[ui:exp.node.assert_value]], [[ui:exp.node.branch_value]], [[ui:exp.node.loop]]의
종료 조건, OSC 인수 규칙, HTTP 조건은 같은 방식으로 비교합니다:

| 옵션 | 파일에서 | 값이 이럴 때 성립 |
| --- | --- | --- |
| [[ui:exp.op.eq]] | `eq` | 기대값과 같음 — 둘 다 숫자이면 숫자로(`200` = `200.0`), 아니면 정확한 텍스트로 |
| [[ui:exp.op.ne]] | `ne` | 같은 규칙으로, 같지 않음 |
| [[ui:exp.op.lt]], [[ui:exp.op.le]], [[ui:exp.op.gt]], [[ui:exp.op.ge]] | `lt`, `le`, `gt`, `ge` | 작음, 이하, 큼, 이상 — 둘 다 숫자여야 함: 아니면 검사, 분기, [[ui:exp.node.loop]]는 단계를 실패시키고, 인수 규칙이나 HTTP 조건은 성립하지 않음 |
| [[ui:exp.op.contains]] | `contains` | 기대한 텍스트를 포함 |
| [[ui:exp.op.matches]] | `matches` | 기대한 정규식과 일치 |
| [[ui:exp.op.empty]], [[ui:exp.op.not_empty]] | `empty`, `not_empty` | 비어 있음(공백도 비어 있는 것으로 봄) / 아님 |

## 흐름 {#flow}

실행이 어디로 가는지 정하는 노드입니다. 분기, 합류, 루프에 대한 더 자세한 내용은
[흐름](flow.md)에 있습니다.

### 시작 {#node-start}

실행이 시작하는 곳이며, 모든 실험에 정확히 하나가 있습니다. 입력도 필드도
없습니다. 타임라인의 첫 행이 실행의 시드를 보여 줍니다.

출력: [[ui:exp.outputPort]], 필수. 여기서 나가는 여러 와이어는 병렬 분기를 동시에
시작합니다.

```json
{ "id": "start", "type": "start", "x": 40, "y": 80 }
```

### 끝 {#node-end}

실행이 완료되는 곳이며, 모든 실험에 정확히 하나가 있고 출력이 없습니다. 여러
분기가 여기로 올 수 있습니다: 실행은 마지막 분기가 끝난 뒤 한 번 통과하며,
아무것도 실패하지 않았을 때만 그렇습니다. [[ui:exp.node.end]]에 도달하지 못한
실행은 실패합니다.

```json
{ "id": "end", "type": "end", "x": 960, "y": 80 }
```

### 지연 {#node-delay}

다음 단계 전에 정해진 시간만큼 기다립니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.delayMs]] | `ms` | | 300; 0–60 000 | 아니요 |

출력: [[ui:exp.outputPort]]. 설정 없음. 더 긴 대기는 여러 개를 이어 놓거나
[[ui:exp.node.loop]] 안에 두십시오.

```json
{ "id": "pause", "type": "delay", "x": 500, "y": 80, "ms": 500 }
```

### 상태 분기 {#node-branch_status}

최신 HTTP 응답이 이 상태이면 [[ui:exp.yes]], 아니면 [[ui:exp.no]]를 고릅니다.
모든 경로에서 HTTP 요청이 그보다 앞서야 합니다.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.expectedStatus]] | `status` | | 200; 100–599 | 아니요 |

출력: [[ui:exp.yes]]와 [[ui:exp.no]], 둘 다 필수. 설정 없음.

```json
{ "id": "branch", "type": "branch_status", "x": 500, "y": 80, "status": 200 }
```

### 값 분기 {#node-branch_value}

비교가 성립하면 [[ui:exp.yes]], 아니면 [[ui:exp.no]]를 고릅니다. 필드와
[비교](#comparisons)는 [[[ui:exp.node.assert_value]]](#node-assert_value)의 것과
같으며, 할 수 없는 비교(텍스트에 `lt`)는 단계를 실패시킵니다.

| 필드 | 파일에서 | 내용 | 기본값 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.value]] | `value` | 비교할 것 | `{{token}}` | 예 |
| [[ui:exp.operator]] | `op` | | [[ui:exp.op.eq]] | 아니요 |
| [[ui:exp.expected]] | `expected` | | 비어 있음 (없으면 이것) | 예 |

출력: [[ui:exp.yes]]와 [[ui:exp.no]], 둘 다 필수. 설정 없음.

```json
{ "id": "ok", "type": "branch_value", "x": 730, "y": 80, "value": "{{reply.args[0]}}", "op": "eq", "expected": "ok" }
```

### 병렬 분기 {#node-fork}

[[ui:exp.branch1]]과 [[ui:exp.branch2]] 뒤에 오는 것을 동시에 실행하며, 각 분기는
자기 변수 사본을 가집니다. 출력마다 더 많은 분기를 위해 와이어를 더 둘 수
있습니다. 필드 없음.

출력: [[ui:exp.branch1]]과 [[ui:exp.branch2]], 둘 다 필수.

```json
{ "id": "split", "type": "fork", "x": 270, "y": 80 }
```

### 분기 합류 {#node-join}

자기에게 들어오는 모든 와이어에 도달할 때까지 기다린 뒤, 분기들의 변수를 합쳐 한
번 이어갑니다 — 두 분기가 같은 변수를 설정하면 파일에서 와이어가 더 뒤에 오는
쪽이 이김 — 그리고 그것들 중 하나라도 가진 마지막 분기의 최신 HTTP 응답을
씁니다. 필드 없음.

모두 실행되는 분기만 여기서 만납니다: [[ui:exp.node.branch_status]] 뒤에 있는
[[ui:exp.node.join]]은 [[ui:exp.yes]]와 [[ui:exp.no]]가 둘 다 일어나는 일이 없어
이어지지 않으며, 다른 경로가 [[ui:exp.node.end]]에 도달하지 못하면 실행은 이
노드에서 실패하면서 아직 몇 개의 분기를 기다렸는지 알려 줍니다.

출력: [[ui:exp.outputPort]], 필수.

들어오는 와이어가 여러 개인 다른 노드는 도착마다 한 번씩 실행됩니다.

```json
{ "id": "joined", "type": "join", "x": 730, "y": 80 }
```

### 루프 {#node-loop}

[[ui:exp.portBody]]의 단계 — 루프로 되돌아오는 — 를 반복해서 실행합니다: 최대
정해진 횟수까지, 그리고 종료 조건이 있으면 그것이 성립할 때까지.

| 필드 | 파일에서 | 내용 | 기본값과 한도 | 템플릿 |
| --- | --- | --- | --- | --- |
| [[ui:exp.loopMax]] | `max` | 최대 반복 횟수 | 5; 1–1 000 | 아니요 |
| [[ui:exp.loopUntilOn]] | `until` | 선택적 종료 조건 `{ "value", "op", "expected" }`, [[[ui:exp.node.assert_value]]](#node-assert_value)에서처럼 | 꺼짐 | value와 expected: 예 |

- 본문은 항상 최소 한 번 실행됩니다. 종료 조건은 매 반복 뒤에 읽으므로, 본문이
  검사할 값을 설정할 수 있습니다.
- 조건이 성립하면 [[ui:exp.portDone]]으로 이어지며, 조건이 없으면 마지막 반복
  뒤에 이어집니다.
- 조건이 성립하기 전에 반복이 다하면 [[ui:exp.portLimit]]으로 이어집니다. 거기에
  와이어가 없으면 단계가 실패합니다.
- 본문 안에서 `{{counter}}`는 반복 번호입니다.
- 본문은 하나의 분기로 실행됩니다: 그 안의 각 출력에는 와이어가 하나이고,
  [[ui:exp.node.start]], [[ui:exp.node.end]], [[ui:exp.node.fork]],
  [[ui:exp.node.join]] 또는 다른 [[ui:exp.node.loop]]을 담지 않으며,
  [[ui:exp.portBody]]를 통해서만 들어가고, 그 안의 모든 와이어는 본문 안으로
  이어지거나 루프로 되돌아갑니다.

출력: [[ui:exp.portBody]]와 [[ui:exp.portDone]] (필수), [[ui:exp.portLimit]] (선택).
설정 없음.

```json
{ "id": "poll", "type": "loop", "x": 270, "y": 80, "max": 10,
  "until": { "value": "{{status.args[0]}}", "op": "eq", "expected": "ready" } }
```

---
title: WebSocket
description: 서비스가 기대하는 헤더와 하위 프로토콜로 ws:// 또는 wss:// 서비스에 연결하고, 텍스트나 이진 메시지를 보내며, 도착하는 모든 메시지를 읽습니다.
---

# WebSocket

[[ui:nav.ws]] 화면은 WebSocket 클라이언트입니다: 서비스가 기대하는 헤더와 하위 프로토콜로 서비스에
연결 하나를 열고, 텍스트나 바이트를 보내며, 오가는 모든 메시지를 최신이 마지막에 오도록 나열합니다.
실험으로 스크립트하기 전에 라이브 API, 컨트롤 서페이스, 또는 WebSocket을 말하는 장치를 시험할 때
사용하십시오.

## 연결하기 {#connect}

1. [[ui:nav.ws]]를 엽니다.
2. [[ui:field.url]]에 주소를 입력합니다: `ws://127.0.0.1:9001/` 또는
   `wss://example.com/socket`.
3. 서비스가 요구하면 [[ui:exp.wsProtocols]]를 입력하고 [[ui:http.headers]]를 추가합니다(토큰이 담긴
   `Authorization` 헤더, 쿠키).
4. [[ui:ws.connect]]를 누릅니다. 업그레이드가 진행되는 동안 버튼은 [[ui:ws.connecting]]을
   말합니다; 끝나면 필드가 잠기고 버튼이 [[ui:ws.disconnect]]가 됩니다.

| 필드 | 내용 | 기본값 |
| --- | --- | --- |
| [[ui:field.url]] | `ws://` 또는 `wss://`, 호스트, 선택적 포트(`ws`는 80, `wss`는 443), 경로 | `ws://127.0.0.1:9001/` |
| [[ui:exp.wsProtocols]] | 제안할 하위 프로토콜, 쉼표로 나누고 선호 순서대로; 서버가 하나를 고릅니다. 이름에는 공백, 쉼표, 슬래시가 없습니다. | 없음 |
| [[ui:http.headers]] | 업그레이드 요청의 추가 헤더; [[ui:http.addHeader]]가 행을 더합니다. 이름이 없는 행은 빠집니다. | 없음 |

연결 — 이름 조회, TCP 연결, `wss://`의 TLS, 업그레이드 — 에는 10초가 있습니다. URL은 화면을
전환해도, 앱을 다시 시작해도 유지됩니다; 헤더와 하위 프로토콜은 아닙니다.

연결되면 패널이 보여 줍니다:

| 항목 | 내용 |
| --- | --- |
| [[ui:ws.state]] | [[ui:ws.open]], 또는 끝났을 때: 닫은 쪽과 닫기 코드와 함께 당신이나 서버가 닫음, 또는 닫기 없이 선이 끊겼을 때 [[ui:ws.lost]] |
| [[ui:field.reason]] | 닫는 쪽이 준 이유, 있으면 |
| [[ui:ws.subprotocol]] | 서버가 고른 하위 프로토콜, 또는 — |
| [[ui:ws.peer]] | 서버의 `IP:port` |
| [[ui:ws.upgradeTime]] | 연결과 업그레이드가 걸린 시간, 밀리초 |

연결은 하나의 작업입니다: 콘솔 표시줄에 나타나고 거기서도 중지할 수 있습니다.

### 보안 연결 {#wss}

`wss://`는 이 시스템의 HTTPS와 같은 인증서를 신뢰합니다: 이 시스템이 신뢰하지 않는 인증서(자체
서명, 만료, 다른 이름)를 가진 서버는 "A secure connection to … could not be made"와 함께
거부됩니다. 검사를 건너뛰는 설정은 없습니다.

## 메시지 보내기 {#send}

1. [[ui:ws.message]] 아래에서 [[ui:exp.wsText]] 또는 [[ui:exp.wsBinary]]를 고릅니다.
2. 메시지를 씁니다. 이진이면 바이트를 hex 숫자 쌍으로 씁니다: `de ad be ef`.
3. [[ui:common.send]]를 누르거나, 메시지에서 <kbd>Ctrl</kbd>+<kbd>Enter</kbd>를 누릅니다.

메시지는 최대 16 MiB(16 777 216바이트, 이진은 hex 숫자가 아니라 바이트로 셉니다)이며, 도착하는
메시지와 [[ui:exp.node.ws_send]] 단계에도 같은 한도입니다. 더 긴 것은 아무것도 보내기 전에
`Too long: at most 16777216`과 함께 거부되고, 연결은 열린 채로 남습니다.

텍스트 메시지가 JSON이면 [[ui:http.formatJson]]이 보내기 전에 들여쓰기로 정리합니다. 메시지는 화면을
전환해도, 앱을 다시 시작해도 유지됩니다.

## 메시지 읽기 {#messages}

[[ui:ws.messages]]는 받은 것(↓)과 보낸 것(↑)을 최신이 마지막에 오도록 나열하며, 시각, 메시지의
시작(300자), 크기를 보여 줍니다; 이진 메시지는 바이트를 hex로 보여 주고 [[ui:ws.binary]]로
표시합니다. 목록 위에는 받은 수와 보낸 수가 있습니다. 목록은 끝까지 스크롤되어 있으면 새 메시지를
따라가고; 위로 스크롤하면 그 자리에 머뭅니다.

메시지를 클릭하면 목록 아래에서 온전히 볼 수 있습니다: JSON은 서식이 잡히고, 이진은 hex로.
[[ui:ws.editHere]]는 그것을 메시지 필드로 복사해 다시 보내거나 고칠 수 있게 합니다. 아주 긴
메시지는 일부만 표시되고 — 텍스트는 최대 64 KiB, 이진은 최대 4096바이트 — 그러면 잘려서 보내질
것이므로 복사할 수 없습니다.

화면은 최근 2000개 메시지를 유지하고, [[ui:common.clear]]가 목록을 비웁니다. 서비스가 화면이 감당할
수 있는 것보다 빠르게 보내면 — 0.1초에 2000개 넘게 — 그중 가장 오래된 것이 목록에서 빠지고 "not
shown"으로 세어집니다. 인스펙터는 캡처가 켜져 있는 동안 여전히 그것들을 가지고 있습니다.

## 닫기 {#close}

[[ui:ws.disconnect]]는 코드 1000(정상)의 닫기 프레임을 보내고, 끊기 전에 서버의 답을 최대 2초
기다립니다. 그러면 상태는 1000과 함께 닫힘으로 읽힙니다. 서버가 닫으면 상태는 그 코드와 이유를 보여
줍니다; 닫기 프레임 없이 연결이 끊기면 [[ui:ws.lost]]로 읽히고 콘솔이 이유를 말합니다.

Signal Lab은 서버의 ping에 스스로 답합니다; ping과 pong은 나열되지 않습니다. 연결은 16 MiB가 넘는
메시지가 도착할 때, 또는 서버가 읽기를 멈춰 메시지 하나를 보내는 데 10초 이상 걸릴 때도 끝납니다.
(16 MiB가 넘는 것을 직접 보내는 것은 거부되며 아무것도 끝내지 않습니다.)

## 인스펙터에서 {#inspector}

캡처가 켜져 있으면 연결의 트래픽이 `ws` 프로토콜과 `websocket` 출처로 나타납니다:

| 요약 | 내용 |
| --- | --- |
| `CONNECT ws://… (subprotocol)` | 연결이 열렸습니다 |
| `TEXT …` | 텍스트 메시지와 그 시작 |
| `BINARY n B …` | 이진 메시지, 크기와 처음 16바이트 |
| `CLOSE code reason` | 닫기 프레임, 보냈거나 받았습니다 |

각 메시지 프레임은 바이트를 보관합니다. 트래픽이 가벼우면 모든 메시지가 캡처됩니다; 바쁜 연결은
초당 200프레임으로 제한되고, 다음에 캡처된 프레임이 몇 개가 빠졌는지 말합니다(`+n not shown`).
[인스펙터](../tools/inspector.md)를 참고하십시오.

## 실험에서 {#experiments}

네 단계가 WebSocket 대화를 스크립트합니다. 한 단계가 연결을 열고 나머지가 그것을 이름으로
가리킵니다:

| 단계 | 하는 일 |
| --- | --- |
| [[ui:exp.node.ws_connect]] | 실행이 끝날 때까지 연결을 엽니다. URL과 헤더는 `{{templates}}`를 받으므로, 앞서 추출한 토큰을 넣을 수 있습니다. [자세히](../experiments/nodes.md#node-ws_connect) |
| [[ui:exp.node.ws_send]] | 연결로 텍스트나 이진 메시지를 보냅니다. [자세히](../experiments/nodes.md#node-ws_send) |
| [[ui:exp.node.wait_ws]] | [[ui:exp.node.wait_udp]]처럼 페이로드가 일치하는 메시지를 기다립니다; JSON 메시지는 나중에 필드별로 읽을 수 있습니다. [자세히](../experiments/nodes.md#node-wait_ws) |
| [[ui:exp.node.ws_close]] | 닫기 핸드셰이크로 연결을 닫습니다: 코드 1000, 또는 애플리케이션 자체의 것은 3000–4999, 그리고 최대 123바이트의 이유. [자세히](../experiments/nodes.md#node-ws_close) |

실행이 끝날 때 — 또는 중지될 때 — 아직 열려 있는 연결은 제대로 닫힙니다.
[[ui:exp.templateWsEcho]] 템플릿이 완성된 예입니다.

## 명령줄에서 {#cli}

`signallab send ws`는 교환 하나를 합니다: 연결하고, 메시지를 보내고, 요청하면 답을 기다리고,
닫습니다.

```bash
signallab send ws ws://127.0.0.1:9001/ --text '{"type":"ping"}' --expect pong
```

핸드셰이크와 보낸 것은 표준 오류로, 답은 표준 출력으로 나갑니다:

```text
Connected to ws://127.0.0.1:9001/ in 4 ms
Sent 15 bytes
{"type":"pong"}
```

`--hex`는 이진 메시지를 보냅니다; `-H`는 헤더를 더하고 `--protocol`은 하위 프로토콜을
제안합니다(둘 다 반복 가능). `--expect TEXT`, `--expect-regex RE` 또는 `--wait`(아무 메시지)는
어떤 답을 기다릴지 정하며, `--timeout` 밀리초(기본 2000) 동안입니다. 답이 오지 않거나 연결이
실패하면 1로 종료합니다. [명령줄](../automation/cli.md#cli-send-ws)을 참고하십시오.

## 문제 {#troubleshooting}

| 보이는 것 | 흔한 원인 |
| --- | --- |
| `… is not a WebSocket address` | URL이 `ws://` 또는 `wss://`로 시작하지 않거나, 호스트가 없습니다. |
| `… answered HTTP n instead of switching to WebSocket` | 서버가 업그레이드를 거부했습니다: 잘못된 경로(404), 없거나 틀린 토큰(401, 403). 답의 시작은 기술적 세부 아래에 있습니다. |
| `… did not take any of the subprotocols offered` | 하위 프로토콜을 제안했지만 서버가 아무것도 고르지 않았거나, 제안하지 않은 것을 답했습니다. |
| `… is not a subprotocol name` | 공백, 쉼표, 슬래시가 있는 이름입니다. |
| `The header … cannot be sent with the upgrade` | HTTP가 허용하지 않는 문자를 가진 헤더 이름이나 값. |
| `… refused the connection` | 그 포트에서 아무것도 수신하지 않습니다. |
| `A secure connection to … could not be made` | 여기서 인증서를 신뢰하지 않거나 TLS가 실패했습니다. [보안 연결](#wss)을 참고하십시오. |
| `The connection with … broke: the server did not keep to the WebSocket protocol` | 서버가 유효한 WebSocket이 아닌 것을 보냈습니다. |
| `A WebSocket message is limited to … bytes` | 서버가 16 MiB가 넘는 메시지를 보냈고, 연결이 끝납니다. |
| `Too long: at most 16777216` | 보내려 한 메시지가 16 MiB를 넘습니다. 아무것도 보내지 않았고 연결은 열려 있습니다. |

모든 오류 메시지는 [오류 메시지](../reference/errors.md#ws)에 정리되어 있습니다.

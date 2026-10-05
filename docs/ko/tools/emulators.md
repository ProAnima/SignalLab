---
title: 에뮬레이터
description: Signal Lab이 상대편 역할을 맡게 합니다. HTTP API, OSC·UDP·TCP 장치, MQTT 브로커가 직접 정한 규칙으로 응답하고, 원하는 때 실패하며, 도착한 것을 집계합니다.
---

# 에뮬레이터

에뮬레이터는 시스템이 통신하는 API, 장치, 서비스의 역할을 Signal Lab이 맡는 것입니다. 한 주소에서
수신하며 규칙에 따라 응답합니다. HTTP API는 라우트로, OSC·UDP·TCP 장치는 “이것을 받으면 저것으로
회신”하는 규칙으로, MQTT 브로커는 여느 브로커처럼 동작하면서 자체 규칙을 더해 응답합니다. 느리게
응답하거나, 실패하거나, 이따금 다운될 수 있으므로, 의존 대상이 오동작할 때 시스템이 어떻게
반응하는지 테스트할 수 있습니다. 모든 교환은 집계되고, 목록에 표시되며, [인스펙터](inspector.md)로
전달됩니다.

에뮬레이터는 하나의 문서입니다. [[ui:nav.emulators]] 화면은 에뮬레이터 라이브러리를 관리하며,
같은 문서가 실험 안에서는 [[ui:exp.node.emulator]] 노드로, 명령줄에서는 `signallab emulate`로,
그리고 [API](../api/commands.md)와 [MCP](../automation/mcp.md)를 통해 실행되고, 어디서든 똑같이
응답합니다.

## 화면 {#screen}

왼쪽에는 라이브러리([[ui:emu.library]])가 있습니다. 모든 에뮬레이터가 프로토콜, 주소와 함께
표시되며, 실행 중인 에뮬레이터에는 깜박이는 점과 요청 수가 붙습니다. 오른쪽에는 선택한 에뮬레이터의
설정과 규칙이 있고, 그 아래에 받은 내용([[ui:emu.live]])이 표시됩니다.

## 에뮬레이터 만들기 {#create}

1. 라이브러리 위쪽의 버튼 중 하나를 누릅니다.

   | 버튼 | 만드는 것 | 수신 주소 | 바로 동작하는 규칙 하나 |
   | --- | --- | --- | --- |
   | ＋ [[ui:emu.new.http]] | HTTP API | `127.0.0.1:18080` | `GET /health` → 200 `{"status":"ok"}` |
   | ＋ [[ui:emu.new.osc]] | OSC 장치 | `127.0.0.1:9100` | `/ping` → 횟수를 int로 담은 `/pong` |
   | ＋ [[ui:emu.new.udp]] | UDP 장치 | `127.0.0.1:7100` | `PING`이 들어 있는 데이터그램 → `PONG 1`, `PONG 2`, … |
   | ＋ [[ui:emu.new.tcp]] | TCP 장치 | `127.0.0.1:7200` | `PING`이 들어 있는 줄 → `PONG` |
   | ＋ [[ui:emu.new.mqtt]] | MQTT 브로커 | `127.0.0.1:1883` | `lab/<name>/set`으로의 발행 → 같은 페이로드를 retained로 `lab/<name>/state`에 |

   라이브러리의 다른 에뮬레이터가 이미 그 포트를 쓰고 있으면 다음 빈 포트를 사용합니다.
2. [[ui:emu.name]] 필드에 이름을 입력합니다(최대 120자).
3. [[ui:emu.bind]] 필드를 `IP:port` 형식으로 설정합니다. `127.0.0.1`은 이 컴퓨터에만 응답하고,
   `0.0.0.0`은 네트워크에도 응답합니다.
4. 규칙을 바꾸고(아래 참고), [[ui:emu.note]] 필드에 이 에뮬레이터가 무엇을 대신하는지 적습니다.

변경 사항은 자동으로 저장됩니다. [[ui:emu.duplicate]] 버튼은 다음 빈 포트에 사본을 만듭니다.
[[ui:emu.delete]] 버튼은 한 번 더 묻고([[ui:emu.confirmDelete]]), 에뮬레이터가 실행 중이면
중지한 뒤 라이브러리에서 제거합니다.

규칙은 처음부터 끝까지 순서대로 시도되며, 처음 일치하는 규칙이 응답합니다. 각 규칙의 머리글에는
한 줄 요약이 표시되며, 클릭하면 규칙을 열거나 접습니다. ↑와 ↓ 버튼은 규칙을 옮기고, ×는
제거합니다.

## 실행하기 {#run}

1. 에뮬레이터를 선택하고 [[ui:emu.start]] 버튼을 누릅니다. 버튼이 다시 활성화되기 전에 포트가
   열립니다. 이미 사용 중인 포트이거나 문제가 있는 에뮬레이터는 그 자리에서 이유와 함께
   거부됩니다.
2. 시스템이 에뮬레이터를 가리키도록 설정합니다. HTTP API의 경우 [[ui:emu.copyUrl]] 버튼이
   주소(`http://127.0.0.1:18080`)를 복사하며, 각 라우트에는 그 라우트의 주소를 복사하는
   [[ui:emu.copyRouteUrl]] 버튼이 있습니다(경로에 `{{…}}` 템플릿이 있으면 없습니다).
3. [[ui:emu.received]] 목록이 채워지는 것을 지켜봅니다.
4. [[ui:emu.stop]] 버튼을 누르거나 콘솔 표시줄에서 작업을 중지합니다.

버튼 옆의 상태 표시에는 [[ui:emu.notRunning]], 응답하는 주소, 또는 다운 상태가 표시됩니다.

에뮬레이터는 시작할 때의 규칙으로 계속 응답합니다. 실행 중에 변경하면 [[ui:emu.restart]] 버튼이
나타나며, 누르면 현재 규칙으로 다시 시작합니다. 그때까지 규칙의 일치 횟수는 이전 규칙의 것이므로
숨겨집니다.

[[ui:emu.takeDown]] 버튼은 실행 중인 에뮬레이터를 [[ui:emu.bringUp]] 버튼을 누를 때까지 사용할
수 없게 만듭니다. HTTP 요청은 503을 받고, TCP 장치와 MQTT 브로커는 연결을 끊고 새 연결을 거부하며,
OSC나 UDP 장치는 아무것도 응답하지 않습니다. [다운 일정](#outage)을 참고하십시오.

같은 전송 방식의 에뮬레이터 두 개는 포트를 공유할 수 없습니다. HTTP, TCP, MQTT 에뮬레이터는 TCP
포트에서, OSC와 UDP 에뮬레이터는 UDP 포트에서 수신합니다. HTTP API와 OSC 장치는 둘 다 포트 8080을
쓸 수 있지만, HTTP API 두 개는 그럴 수 없습니다. 이미 사용 중인 포트의 두 번째 에뮬레이터는 시작할
때 거부됩니다.

::: tip
[서버](../server/index.md)에 연결된 브라우저에서는 에뮬레이터가 서버에서 실행됩니다. `0.0.0.0`에서
수신하는 에뮬레이터는 서버 이름으로 접근하며, [[ui:emu.copyUrl]] 버튼은 그 주소를 복사합니다.
`127.0.0.1`에서 수신하는 에뮬레이터는 서버 자체의 프로그램에만 응답합니다.
:::

## 받은 내용 {#received}

실행 중에 [[ui:emu.live]] 영역은 다음을 셉니다.

| 집계 | 내용 |
| --- | --- |
| [[ui:emu.total]] | 요청, 메시지, 줄 등 도착한 모든 것. |
| [[ui:emu.unmatched]] | 어떤 규칙도 받지 않은 것. 라우트가 없는 HTTP 요청도 응답은 받으며([라우트가 받지 않는 요청](#fallback) 참고), 나머지는 아무것도 받지 않습니다. |
| [[ui:emu.failed]] | 회신을 만들거나 보낼 수 없었던 교환. |
| [[ui:emu.down]] | 에뮬레이터가 다운된 동안 도착한 것. 다운 일정이 설정되어 있거나 다운 상태에서 무언가 도착했을 때 표시됩니다. [[ui:emu.unmatched]]으로는 절대 집계되지 않습니다. |
| [[ui:emu.missed]] | MQTT 전용이며 발생했을 때만 표시됩니다. 클라이언트가 너무 뒤처져 받지 못한 메시지. |

각 규칙의 머리글에는 시작 이후 일치한 횟수가 표시됩니다.

[[ui:emu.received]] 목록에는 최신 교환 300개가 최신 항목부터 나열됩니다.

| 열 | 내용 |
| --- | --- |
| [[ui:emu.col.time]] | 도착한 시각. |
| [[ui:emu.col.from]] | 클라이언트의 주소. |
| [[ui:emu.col.request]] | 도착한 것을 프로토콜 표기로: `GET /users/7`, `/ping 1`, `POWER?`. |
| [[ui:emu.col.rule]] | 받은 규칙(`#2`) 또는 `—`. |
| [[ui:emu.col.reply]] | 돌아간 것: `200 OK · 37 B`, `/pong 3`, 페이로드. 장애이면 [[ui:emu.held]] 또는 [[ui:emu.closed]], 회신에 실패했으면 그 오류, 다운 중에 도착했으면 [[ui:emu.wasDown]]. |
| [[ui:emu.col.ms]] | 도착부터 회신이 나갈 때까지의 시간, 지연 포함. |

행의 ⌕ 버튼([[ui:emu.inspectFrame]])은 캡처가 켜져 있었을 때 그 교환을 인스펙터에서 엽니다.
0.2초 안에 200개가 넘는 교환이 도착하면 목록은 일부를 건너뛰고 몇 개를 건너뛰었는지 표시합니다.
엔진은 명령줄, API, MCP를 위해 실행 중인 각 에뮬레이터의 최신 교환 500개를 도착한 내용과 함께
보관합니다.

## HTTP API {#http}

HTTP/1.1 서버입니다. 각 요청에는 그 요청을 받는 첫 번째 라우트가 응답합니다.

### 라우트 {#routes}

메서드, 경로, 모든 조건이 일치하면 라우트가 요청을 받습니다.

| 필드 | 내용 |
| --- | --- |
| [[ui:emu.method]] | `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS` 또는 [[ui:emu.methodAny]]. `GET` 라우트는 `HEAD`에도 응답합니다. |
| [[ui:emu.path]] | `/`로 시작합니다. `:name` 세그먼트는 임의의 세그먼트 하나를 받으며 `{{request.params.name}}`으로 읽습니다. 마지막 세그먼트 `*`는 그 아래의 모든 것을 받습니다. 끝의 `/`는 차이가 없으며, 쿼리 문자열은 경로에 포함되지 않습니다. |
| [[ui:emu.conditions]] | 모두 충족되어야 합니다. ＋ [[ui:emu.addCondition]]으로 추가합니다. |

경로 예:

| 경로 | 받는 요청 | 받지 않는 요청 |
| --- | --- | --- |
| `/health` | `/health`, `/health/` | `/health/db`, `/Health` |
| `/users/:id` | `/users/7`(`params.id`는 `7`), `/users/a%20b`(`a b`) | `/users`, `/users/7/orders` |
| `/files/*` | `/files`, `/files/a`, `/files/a/b/c` | `/file`, `/other/files/a` |

조건은 요청의 한 부분([[ui:emu.on]])을 읽어 비교합니다.

| [[ui:emu.on]] | 이름 | 읽는 값 |
| --- | --- | --- |
| [[ui:emu.on.header]] | 헤더 이름, 대소문자 무관 | 헤더의 값. 여러 번 보낸 헤더는 값을 `, `로 이어 붙입니다. |
| [[ui:emu.on.query]] | 쿼리 매개변수 | 디코딩된 값. 반복되면 첫 번째 값. |
| [[ui:emu.on.body]] | — | 본문 전체를 텍스트로. |
| [[ui:emu.on.json]] | `$.user.id` 같은 JSON 경로 | JSON 본문의 해당 필드. |

비교 연산은 [[ui:exp.op.eq]], [[ui:exp.op.ne]], [[ui:exp.op.lt]], [[ui:exp.op.le]],
[[ui:exp.op.gt]], [[ui:exp.op.ge]], [[ui:exp.op.contains]], [[ui:exp.op.matches]],
[[ui:exp.op.empty]], [[ui:exp.op.not_empty]]입니다. 숫자는 숫자로, 텍스트는 정확히 일치하는지
비교합니다. 없는 헤더, 매개변수, 필드는 비어 있는 것으로 봅니다. 텍스트와 숫자처럼 비교할 수 없는
경우에는 조건이 충족되지 않습니다.

### 응답 {#responses}

라우트 하나에는 [[ui:emu.responses]]을 1~16개 둘 수 있습니다.

| 필드 | 내용 | 기본값 |
| --- | --- | --- |
| [[ui:emu.status]] | 100–599. | 200 |
| [[ui:emu.fault]] | 응답 대신 일어나는 일. [장애](#faults)를 참고하십시오. | [[ui:emu.fault.none]] |
| [[ui:emu.delay]] | 응답하기 전에 기다리는 시간, 0–60,000 ms. | 0 |
| [[ui:emu.jitter]] | 무작위로 최대 이만큼 더 기다립니다, 0–60,000 ms. | 0 |
| [[ui:emu.weight]] | 라우트가 무작위로 응답할 때 이 응답의 비중. 그때만 표시됩니다. | 1 |
| [[ui:emu.headers]] | 최대 32개. 이름에는 매개변수를 쓸 수 있고, 값은 [템플릿](#templates)입니다. | 없음 |
| [[ui:emu.body]] | [템플릿](#templates), 작성한 그대로 최대 256 KiB. | 비어 있음 |

`Content-Type` 헤더가 없으면 올바른 JSON인 본문은 `application/json`으로, 그 밖의 본문은
`text/plain; charset=utf-8`로 보냅니다.

응답이 두 개 이상이면 [[ui:emu.order]] 설정이 요청마다 어느 응답을 받을지 정합니다.

| [[ui:emu.order]] | 요청이 받는 응답 | 용도 |
| --- | --- | --- |
| [[ui:emu.order.sequence]] | 첫 번째, 두 번째, …, 그 뒤로는 계속 마지막 응답: 500, 500, 200, 200, 200… | 재시도: 두 번 실패한 뒤 동작. |
| [[ui:emu.order.cycle]] | 마지막 다음에 다시 첫 번째: 200, 500, 200, 500… | 규칙적으로 이따금 실패하는 의존 서비스. |
| [[ui:emu.order.random]] | 가중치에 따라 하나씩 추첨합니다. 가중치가 8과 2이면 첫 번째 응답이 약 80%의 확률로 나옵니다. 하나 이상의 가중치가 0보다 커야 합니다. | 현실적인 실패 비율. |

[[ui:emu.preset]] 메뉴는 라우트에 미리 준비된 응답을 추가합니다.

| 프리셋 | 추가되는 응답 |
| --- | --- |
| [[ui:emu.preset.ok]] | 200, `{"ok":true}` |
| [[ui:emu.preset.created]] | 201, `{"id":"{{uuid}}"}`, 헤더 `Location: {{request.path}}/{{counter}}` |
| [[ui:emu.preset.notFound]] | 404, `{"error":"not found"}` |
| [[ui:emu.preset.error]] | 500, `{"error":"internal"}` |
| [[ui:emu.preset.unavailable]] | 503, `{"error":"unavailable"}`, 헤더 `Retry-After: 1` |
| [[ui:emu.preset.slow]] | 2000 ms 뒤 200, `{"ok":true}` |
| [[ui:emu.preset.timeout]] | 장애 [[ui:emu.fault.timeout]] |
| [[ui:emu.preset.reset]] | 장애 [[ui:emu.fault.reset]] |
| [[ui:emu.preset.malformed]] | 200, `{"items":[{"id":1},{"id":2}]}`, 장애 [[ui:emu.fault.malformed]] 적용 |

### 장애 {#faults}

| [[ui:emu.fault]] | 클라이언트가 겪는 일 |
| --- | --- |
| [[ui:emu.fault.none]] | 응답을 받습니다. |
| [[ui:emu.fault.timeout]] | 아무것도 받지 못합니다. 요청을 최대 2분 동안 붙잡아 두었다가 연결을 닫으므로, 클라이언트 자체의 시간 제한이 테스트됩니다. 지연은 적용되지 않습니다. |
| [[ui:emu.fault.reset]] | 지연 후 응답 없이 연결이 닫힙니다. |
| [[ui:emu.fault.malformed]] | 상태와 헤더가 설정된 완전한 HTTP 응답이지만 본문이 중간에 끊겨, JSON이 파싱되지 않습니다. 본문 전체가 JSON이었다면 콘텐츠 유형은 여전히 `application/json`입니다. |

### 라우트가 받지 않는 요청 {#fallback}

[[ui:emu.fallback]] 설정은 어떤 라우트와도 일치하지 않는 요청이 무엇을 받을지 정합니다.

- [[ui:emu.fallbackDefault]] — 본문 `{"error":"no_route"}`와 함께 404.
- [[ui:emu.fallbackCustom]] — 직접 설정한 응답. 라우트의 응답에 있는 모든 항목을 쓸 수 있습니다.
  여기서 `{{counter}}`는 어떤 라우트도 받지 않은 요청의 수를 셉니다.

어느 쪽이든 요청은 [[ui:emu.unmatched]]으로 집계됩니다.

### HTTP 회신이 읽을 수 있는 값 {#http-request}

| 템플릿 | 값 |
| --- | --- |
| `{{request.method}}` | `GET`, `POST`, … |
| `{{request.path}}` | 쿼리를 뺀 경로. |
| `{{request.params.id}}` | `:id`라는 이름의 경로 세그먼트. |
| `{{request.query.page}}` | 디코딩된 쿼리 매개변수. |
| `{{request.headers.x-key}}` | 헤더. 이름은 소문자로 씁니다. |
| `{{request.body}}` | 본문을 텍스트로: 처음 64 KiB. |
| `{{request.json.name}}` | 본문이 JSON이고 64 KiB 이내일 때 JSON 본문의 필드. |
| `{{request.from}}` | 클라이언트의 `IP:port`. |

1 MiB보다 큰 요청 본문은 413을 받고 [[ui:emu.failed]]로 집계됩니다. 요청에 없는 것을 가리키는
템플릿처럼 회신을 만들 수 없으면 본문에 오류를 담은 500을 받고 [[ui:emu.failed]]로 집계됩니다.

## OSC 장치 {#osc}

도착하는 각 메시지(번들 안의 메시지는 하나씩 따로)에는 그 메시지와 일치하는 첫 번째 규칙이
응답합니다. OSC가 아닌 데이터그램은 [[ui:emu.unmatched]]으로 집계됩니다.

| 필드 | 내용 |
| --- | --- |
| [[ui:emu.address]] | OSC 1.0 주소 패턴: `*`는 임의의 문자열, `?`는 한 글자, `[a-z]`는 문자 집합, `{a,b}`는 둘 중 하나이며, 각각 한 세그먼트 안에서만 적용됩니다([OSC](../protocols/osc.md#patterns) 참고). |
| [[ui:exp.argRules]] | [[ui:exp.node.wait_osc]] 노드와 같은 방식의 인수 조건, 최대 16개([노드](../experiments/nodes.md#node-wait_osc) 참고). |
| [[ui:emu.replyOn]] | 끄면 메시지를 받기만 하고 아무것도 회신하지 않습니다. |
| [[ui:emu.replyAddress]] | 회신의 주소, [템플릿](#templates). |
| [[ui:emu.replyArgs]] | 최대 16개의 인수. 각 인수는 [[ui:emu.argType]] (`int`, `float`, `str`, `long`, `double`, `bool`, `blob`, `nil`)과 [[ui:emu.argValue]] 템플릿으로 이루어집니다. |
| [[ui:emu.to]] | 비어 있으면 보낸 쪽의 주소와 포트로 회신합니다. 아니면 `IP:port`. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 각각 0–60,000 ms. |

인수의 값은 템플릿을 채운 뒤 해당 타입으로 읽습니다. 타입이 숫자이면 `{{request.args[0]}}`은 첫
번째 인수를 숫자로 되돌려 보냅니다. `bool`은 `true`, `1`, `yes`, `on` 또는 `false`, `0`, `no`,
`off`를 받고, `blob`은 hex 바이트를 받으며, 빈 값은 그 타입의 0 값입니다.

회신은 에뮬레이터 자체의 포트에서 나가므로, 보낸 포트에서 수신하는 클라이언트는 회신을 받습니다.

OSC 회신은 `{{request.address}}`, `{{request.args[0]}}`, `{{request.from}}`을 읽을 수 있습니다.

## UDP 장치 {#udp}

각 데이터그램에는 그 데이터그램과 일치하는 첫 번째 규칙이 응답합니다.

| 필드 | 내용 |
| --- | --- |
| [[ui:emu.match]] | [[ui:exp.mode.any]], [[ui:exp.mode.contains]], [[ui:exp.mode.regex]], [[ui:exp.mode.hex]] 중 하나. |
| [[ui:emu.pattern]] | 찾을 텍스트, 정규식 또는 바이트. |
| [[ui:emu.reply]] | [[ui:emu.replyOff]], [[ui:emu.replyText]], [[ui:emu.replyHex]] 중 하나를 고른 뒤 회신 자체를 [템플릿](#templates)으로 씁니다. |
| [[ui:emu.to]] | 비어 있으면 보낸 쪽으로 회신합니다. 아니면 `IP:port`. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 각각 0–60,000 ms. |

UDP나 TCP 회신은 다음을 읽을 수 있습니다.

| 템플릿 | 값 |
| --- | --- |
| `{{request.text}}` | 페이로드를 텍스트로. |
| `{{request.match}}` | 일치한 것: 텍스트, 정규식의 첫 번째 그룹(또는 일치한 부분 전체), 바이트. |
| `{{request.hex}}` | 페이로드를 hex 바이트로, 처음 1024바이트. |
| `{{request.bytes}}` | 페이로드의 크기. |
| `{{request.from}}` | 보낸 쪽의 `IP:port`. |

텍스트 회신은 최대 65,507바이트입니다.

## TCP 장치 {#tcp}

프로젝터나 매트릭스 스위처처럼 TCP 연결에서 줄 단위로 통신하는 장치입니다. 클라이언트가 보내는 각
메시지에는 그 메시지와 일치하는 첫 번째 규칙이 응답하며, 회신은 같은 연결로 돌아갑니다.

| 필드 | 내용 |
| --- | --- |
| [[ui:emu.delimiter]] | 메시지의 끝을 나타내며, 각 회신과 인사말 뒤에도 붙습니다: [[ui:emu.delimiter.lf]] (앞의 `\r`은 버립니다), [[ui:emu.delimiter.crlf]], [[ui:emu.delimiter.cr]], [[ui:emu.delimiter.none]]. 빈 줄은 건너뜁니다. |
| [[ui:emu.greeting]] | 클라이언트가 연결할 때 보냅니다. 비워 두면 보내지 않습니다. `{{request.from}}`을 읽을 수 있습니다. |
| [[ui:emu.match]], [[ui:emu.pattern]], [[ui:emu.reply]] | [UDP 장치](#udp)와 같습니다. |
| [[ui:emu.close]] | 이 규칙의 회신 뒤에 연결을 닫습니다. 예: `QUIT`. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 각각 0–60,000 ms. |

구분자 없이 64 KiB보다 긴 메시지는 그 상태 그대로 받습니다.

## MQTT 브로커 {#mqtt}

일반 TCP에서 동작하는 작은 MQTT 3.1.1 브로커입니다. 브로커가 하는 일을 그대로 합니다. 클라이언트가
연결하고, `+`와 `#`로 구독하고, QoS 0, 1, 2로 발행하며, retained 메시지와 유언 메시지가 동작하고,
같은 클라이언트 id로 두 번째 연결이 들어오면 첫 번째 연결을 대신합니다. 세션은 항상 clean
session입니다. 세션 유지를 요청한 클라이언트도 새 세션을 받으며, 연결되어 있지 않은 클라이언트를
위해 대기열에 쌓아 두는 메시지는 없습니다.

여기에 더해, 브로커에 발행된 모든 메시지를 규칙과 대조합니다. 처음 일치하는 규칙은 회신도
발행하므로, 장치가 자기가 한 일을 보고하는 것처럼 동작합니다.

| 필드 | 내용 |
| --- | --- |
| [[ui:emu.username]], [[ui:emu.password]] | 사용자 이름을 설정하면 클라이언트는 그 이름과 비밀번호로 연결해야 합니다. 비워 두면 누구나 연결할 수 있습니다. 사용자 이름 없는 비밀번호는 MQTT 3.1.1이 전달할 수 없으므로 거부됩니다. |
| [[ui:emu.retained]] | retain으로 발행된 것처럼 시작부터 보관하는 메시지 최대 64개([[ui:emu.topic]], [[ui:emu.payload]], [[ui:emu.qos]]). 구독하는 클라이언트가 이 메시지를 먼저 받습니다. |
| [[ui:emu.topicFilter]] | 규칙이 받는 토픽: `+`는 한 단계, `#`는 나머지 전부 — `lab/+/set`. |
| [[ui:emu.match]], [[ui:emu.pattern]] | [UDP 장치](#udp)와 같은 방식의 페이로드 조건. |
| [[ui:emu.replyOn]] | 끄면 메시지를 받기만 하고 더 발행하지 않습니다. |
| [[ui:emu.replyTopic]], [[ui:emu.replyPayload]] | [템플릿](#templates). 토픽에는 `+`나 `#`를 쓸 수 없습니다. |
| [[ui:emu.qos]], [[ui:emu.retain]] | 회신에 적용됩니다. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 각각 0–60,000 ms. |

MQTT 회신은 `{{request.topic}}`, `{{request.levels[1]}}`(토픽의 단계, 0부터),
`{{request.payload}}`, `{{request.json.state}}`, `{{request.match}}`, `{{request.qos}}`,
`{{request.retain}}`, `{{request.client}}`(클라이언트 id), `{{request.from}}`을 읽을 수 있습니다.

## 회신의 템플릿 {#templates}

회신은 실험과 같은 [템플릿 언어](../experiments/data.md#templates)로 작성하므로, 필드는 여기서나
실험에서나 같은 뜻입니다. 회신은 다음을 읽을 수 있습니다.

- `request` — 위에서 프로토콜별로 정리한, 도착한 내용.
- `{{counter}}` — 에뮬레이터가 시작된 이후 이 규칙이 받은 메시지 수(이번 메시지 포함).
- [생성기](../experiments/data.md#generators) — `{{uuid}}`, `{{now.iso}}`, 임의 값 등. 임의
  값은 에뮬레이터의 시드에서 뽑습니다.
- 매개변수 — 에뮬레이터가 실험 안에서 실행되거나 `signallab emulate --param`으로 시작된 경우.

회신은 시크릿을 절대 읽지 않으며, 알 수 없는 이름은 빈 텍스트가 아니라 오류입니다.

일부 필드는 무언가 도착하기 전, 에뮬레이터가 시작될 때 고정됩니다. 경로, 조건, 주소 패턴, 페이로드
패턴, 토픽 필터, [[ui:emu.to]], 헤더 이름, retained 메시지, 브로커 로그인 정보입니다. 이 필드에는
텍스트와 매개변수만 쓸 수 있으며, `request`와 생성기는 쓸 수 없습니다.

시드는 응답의 무작위 순서, 지터, 무작위 생성기를 결정합니다. [[ui:nav.emulators]] 화면에서는
시작할 때마다 새 시드를 쓰고, 실험은 실행의 시드를 쓰며, `signallab emulate --seed`는 지정한 시드를
씁니다.

## 다운 일정 {#outage}

의존 서비스가 오락가락할 때 시스템이 어떻게 반응하는지 테스트하려면 [[ui:emu.outage]] 옵션을
선택하십시오.

| 필드 | 내용 | 기본값 |
| --- | --- | --- |
| [[ui:emu.outageUp]] | 응답하는 시간, 10–3,600,000 ms. | 10,000 |
| [[ui:emu.outageDown]] | 다운되어 있는 시간, 10–3,600,000 ms. | 3000 |
| [[ui:emu.outageFault]] | HTTP 전용: 다운된 동안 요청이 겪는 일. | [[ui:emu.outageFault.unavailable]] |

일정은 에뮬레이터가 시작될 때 시작되어 동작, 다운, 동작, 다운… 순으로 반복됩니다. 다운된 동안에는
다음과 같습니다.

| 에뮬레이터 | 겪는 일 |
| --- | --- |
| HTTP | [[ui:emu.outageFault.unavailable]]: 복구될 때까지 남은 초(최소 1)로 `Retry-After`를 설정한 503. [[ui:emu.outageFault.reset]]: 응답 없이 연결이 닫힙니다. [[ui:emu.outageFault.timeout]]: 최대 2분 동안 붙잡아 두었다가 닫습니다. |
| TCP 장치 | 열린 연결은 0.1초 안에 끊기고, 새 연결은 들어오는 즉시 닫힙니다. |
| MQTT 브로커 | 모든 연결이 끊기고, 새 연결은 거부됩니다(CONNACK 반환 코드 3, 서버 사용 불가). |
| OSC, UDP 장치 | 아무것도 응답하지 않습니다. |

다운된 동안 도착한 것은 [[ui:emu.unmatched]]이 아니라 [[ui:emu.down]]으로 집계되며, 규칙에
묻지 않습니다.

[[ui:emu.takeDown]] 버튼은 일정과 관계없이 [[ui:emu.bringUp]] 버튼을 누를 때까지 같은 일을 필요할
때 바로 합니다. 이때 HTTP는 `Retry-After` 없는 503을 받습니다. 실험에서는
[[ui:exp.node.emulator_state]] 노드가 실행의 한 단계에서 이 일을 합니다
([노드](../experiments/nodes.md#node-emulator_state)와 [장애 주입](../experiments/faults.md)
참고).

## 문제 {#problems}

편집하는 동안 에뮬레이터는 변경할 때마다 잠시 뒤 검사되며, [[ui:emu.start]] 버튼을 누르기 전에
버튼 아래에 문제가 표시됩니다. 문제는 그 위치(규칙, 응답 또는 retained 메시지와 필드)와 잘못된
내용을 알려 줍니다. `/` 없는 경로, 컴파일되지 않는 정규식, `request`, 매개변수, 생성기 외의 것을
가리키는 회신 템플릿, 범위를 벗어난 값 등입니다. [[ui:emu.start]] 버튼은 문제가 있는 에뮬레이터를 거부합니다.

## 한도 {#limits}

| 항목 | 한도 | 한도에 도달하면 |
| --- | --- | --- |
| 에뮬레이터당 라우트 또는 규칙 | 64 | 검사할 때 거부됩니다. |
| 라우트당 응답 | 16 | 거부됩니다. |
| 라우트당 조건 | 16 | 거부됩니다. |
| 응답당 헤더 | 32 | 거부됩니다. |
| 인수 조건, 회신 인수(OSC) | 각각 16 | 거부됩니다. |
| retained 메시지(MQTT) | 64 | 거부됩니다. |
| 작성한 그대로의 본문, 회신, 인사말 | 256 KiB | 거부됩니다. |
| 지연 또는 지터 | 60,000 ms | 거부됩니다. |
| HTTP 요청 본문 | 1 MiB | 413. |
| 동시 HTTP 연결 | 512 | 초과분은 들어오는 즉시 닫힙니다. |
| HTTP 요청 헤드 | 30초 | 클라이언트는 이 시간 안에 보내야 합니다. |
| 동시 TCP 연결 | 256 | 초과분은 들어오는 즉시 닫힙니다. |
| 지연을 기다리는 OSC 및 UDP 회신 | 1024 | 초과분은 버려지고 [[ui:emu.failed]]로 집계됩니다. |
| 동시 MQTT 클라이언트 | 256 | 초과분은 들어오는 즉시 닫힙니다. |
| MQTT 패킷 | 256 KiB | 클라이언트의 연결이 끝납니다. |
| 클라이언트당 MQTT 구독 | 100 | 초과분은 거부됩니다. |
| MQTT retained 토픽 | 토픽 1000개, 16 MiB | 새 retained 메시지는 전달되지만 보관되지 않습니다. |
| 느린 클라이언트 하나를 기다리는 MQTT 메시지 | 메시지 1024개, 8 MiB | 그 클라이언트는 메시지를 받지 못하며, [[ui:emu.missed]]으로 집계됩니다. |

## 모의 응답 만들기 {#mock-this}

잘 동작한 응답으로 에뮬레이터를 만들려면 다음과 같이 합니다.

1. [[ui:nav.http]] 화면에서 요청을 보내 응답을 받습니다. 또는 실험의 HTTP 노드에서
   [[ui:exp.sendNow]] 기능을 사용합니다.
2. 응답 옆의 ⧉ [[ui:http.mockThis]] 버튼을 누릅니다. [[ui:http.mockTitle]] 대화 상자에 만들어질
   라우트가 표시됩니다.
3. [[ui:http.mockInto]] 목록에서 HTTP 에뮬레이터 중 하나 또는 [[ui:http.mockNew]] 항목을
   고릅니다.
4. [[ui:http.mockAdd]] 버튼을 누릅니다. [[ui:nav.emulators]] 화면이 그 에뮬레이터를 연 상태로
   열립니다.

이 라우트는 요청의 메서드와 경로(쿼리 제외)에 대해 응답의 상태, 헤더, 본문으로 응답합니다. 그 한
번의 교환에만 해당하는 헤더(`Content-Length`, `Date`, `Server`, `ETag` 등)는 빠지며, 본문은 `{{`가
들어 있더라도 받은 그대로 보냅니다. 새 에뮬레이터에는 이 라우트만 들어 있습니다. 기존 에뮬레이터에
추가하면 라우트가 맨 앞에 들어가므로 더 넓은 범위의 라우트보다 먼저 응답합니다. 실행 중인
에뮬레이터는 [[ui:emu.restart]] 버튼을 누르면 이 라우트를 반영합니다.

실험에서 만들면 템플릿으로 쓴 URL이 패턴이 됩니다. 기본 부분(`{{api}}`)은 빠지고, 템플릿 하나로 된
세그먼트(`/orders/{{order_id}}`)는 `:order_id`가 되며, 일부만 템플릿인 세그먼트가 있으면 경로가
거기서 `*`로 끝납니다.

## 기본 제공 세트 {#starter-set}

Signal Lab이 에뮬레이터 라이브러리를 처음 찾지 못하면, 모두 이 컴퓨터에서 동작하는 에뮬레이터
다섯 개를 씁니다. 이름과 메모는 그 시점의 인터페이스 언어로 작성됩니다.

| 에뮬레이터 | 수신 주소 | 동작 |
| --- | --- | --- |
| [[ui:seed.emu.demo-api.name]] | `127.0.0.1:8080` | `GET /health` → `{"status":"ok","time":…}`; `GET /users/:id` → 해당 id의 사용자; `POST /users` → `Location`과 함께 201; `GET /slow` → 1500 ms 뒤 응답; `/flaky` → 503, 503, 그 뒤로는 200. |
| [[ui:seed.emu.osc-device.name]] | `127.0.0.1:9100` | `/ping` → 횟수와 함께 `/pong`; `/fader/*` → 받은 주소와 함께 `/ack`; `/cue/*`는 회신 없이 받습니다. |
| [[ui:seed.emu.udp-device.name]] | `127.0.0.1:7100` | `PING` → `PONG`과 횟수; 그 밖의 것 → `ACK`와 크기(바이트). |
| [[ui:seed.emu.tcp-device.name]] | `127.0.0.1:7200` | CR LF로 끝나는 줄. `READY`로 인사; `POWER?` → `POWER=ON`; `POWER ON` 또는 `POWER OFF` → `OK ON` / `OK OFF`; `QUIT` → `BYE` 후 연결 끊기. |
| [[ui:seed.emu.mqtt-broker.name]] | `127.0.0.1:1883` | `lab/status`에 `online`을 retained로 보관; `lab/<name>/set`에 발행된 `ON` 또는 `OFF` → 같은 값을 retained로 `lab/<name>/state`에. |

[신호 라이브러리](signals.md#starter-set)에 기본으로 들어 있는 [[ui:seed.http-reachable.name]]
신호는 [[ui:seed.emu.demo-api.name]] 에뮬레이터의 주소인 `http://127.0.0.1:8080/`에 요청합니다.
이 에뮬레이터에는 `/` 라우트가 없으므로 404를 받습니다.

## 라이브러리 파일 {#file}

라이브러리는 데이터 폴더의 `emulators.json`입니다([파일](../reference/files.md) 참고). 목록 아래의
개수에 포인터를 올리면 경로가 보입니다. 마지막 변경 0.7초 뒤에 임시 파일을 거쳐 통째로 쓰므로,
쓰기에 실패해도 이전 파일이 남습니다. 파일을 읽을 수 없으면 목록에 경로, 줄, 열과 함께 오류가
표시되고 파일은 그대로 둡니다. 파일을 고친 뒤 [[ui:emu.reload]] 버튼을 누르십시오. 파일을 직접
편집한 뒤에도 [[ui:emu.reload]] 버튼을 누르십시오. 파일이 없으면 기본 제공 세트를 다시 씁니다.

```json
{
  "version": 1,
  "emulators": [
    {
      "id": "orders-api",
      "note": "Stands in for the orders service.",
      "emulator": {
        "name": "Orders API",
        "bind": "127.0.0.1:18080",
        "protocol": "http",
        "routes": [
          { "method": "GET", "path": "/orders/:id",
            "responses": [{ "body": "{\"id\":\"{{request.params.id}}\",\"state\":\"open\"}" }] },
          { "method": "POST", "path": "/orders", "order": "sequence",
            "responses": [{ "status": 503 }, { "status": 201, "body": "{\"id\":\"{{uuid}}\"}" }] }
        ],
        "outage": { "up_ms": 20000, "down_ms": 2000, "fault": "unavailable" }
      }
    }
  ]
}
```

`emulator` 객체만으로도 `signallab emulate`가 읽는 문서가 됩니다.

## 실험과 스크립트에서 {#elsewhere}

- 실험에서는 [[ui:exp.node.emulator]] 노드가 첫 단계 전에 에뮬레이터를 열고 실행이 끝날 때까지
  응답하며, 받은 내용은 보고서에 집계됩니다. 여기서 HTTP 에뮬레이터는
  [[ui:exp.node.wait_http]] 노드([노드](../experiments/nodes.md#node-wait_http))가 수신하는
  대상이기도 하며, OSC나 UDP 에뮬레이터는 실행의 대기 노드와 포트를 공유합니다. 한 실험 안에서 같은
  전송 방식의 에뮬레이터 두 개는 포트를 공유할 수 없습니다. [노드](../experiments/nodes.md#node-emulator)와
  [장애 주입](../experiments/faults.md)을 참고하십시오.
- `signallab emulate`는 파일이나 이 라이브러리의 에뮬레이터를 <kbd>Ctrl</kbd>+<kbd>C</kbd>를
  누르거나 `--for` 시간이 지날 때까지 실행하며, 응답하는 내용을 출력합니다.
  [명령줄](../automation/cli.md#cli-emulate)을 참고하십시오.

## 관련 항목 {#related}

- [인스펙터](inspector.md) — 디코딩된 모든 교환.
- [네트워크 장애](impairment.md) — 시스템과 에뮬레이터 사이의 나쁜 네트워크.
- [데이터와 템플릿](../experiments/data.md#templates)

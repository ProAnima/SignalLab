---
title: 실행이 흐르는 방식
description: 시작과 종료, 출력과 연결, 병렬 분기와 합류, 분기, 재시도, 반복과 루프, 실행 시작부터 수신하는 대기, 그리고 실행 전에 검사하는 것.
---

# 실행이 흐르는 방식

실행은 [[ui:exp.node.start]]에서 시작해 연결을 따라 노드에서 노드로 이동하며, 모든 분기가 끝나고
[[ui:exp.node.end]]에 도달하면 완료됩니다. 이 페이지에서는 실행이 따르는 규칙을 설명합니다. 각
노드가 하는 일은 [노드 참조](nodes.md)에, 실행과 함께 이동하는 값은 [데이터](data.md)에 있습니다.

## 시작과 종료 {#start-end}

실험에는 [[ui:exp.node.start]]가 정확히 하나, [[ui:exp.node.end]]가 정확히 하나 있습니다.

- [[ui:exp.node.start]]에는 입력이 없습니다. 곧바로 통과하며, 타임라인의 그 행이 실행의 시드를
  알려 줍니다. 그 출력에는 여러 연결이 있을 수 있으며, 그러면 실험은 병렬 분기로 시작합니다.
- [[ui:exp.node.end]]에 도달하는 모든 분기는 거기서 멈춥니다. 종료는 첫 도착부터 실행 중으로
  표시되고, 마지막 분기가 끝난 뒤 한 번 통과합니다. 어느 단계든 실패하면 통과하지 않습니다. 이
  통과가 실행을 [[ui:exp.passed]]로 만듭니다.
- 모든 분기가 오류 없이 끝났지만 아무 분기도 종료에 도달하지 못한 실행은 `run.no_end`로
  실패합니다.

## 출력과 연결 {#outputs}

노드의 단계는 출력을 골라 끝나며, 실행은 그 출력의 모든 연결을 따릅니다. 대부분의 노드에는
[[ui:exp.outputPort]] 하나가 있고, 일부는 여러 출력 중에서 고릅니다:

| 노드 | 반드시 연결해야 하는 출력 | 연결해도 되는 출력 |
| --- | --- | --- |
| [[ui:exp.node.end]] | — | — |
| [[ui:exp.node.fork]] | [[ui:exp.branch1]], [[ui:exp.branch2]] | — |
| [[ui:exp.node.branch_status]], [[ui:exp.node.branch_value]] | [[ui:exp.yes]], [[ui:exp.no]] | — |
| 모든 대기([[ui:exp.node.wait_osc]], [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_http]], [[ui:exp.node.wait_ws]]) | [[ui:exp.portMatched]] | [[ui:exp.portTimeout]] |
| [[ui:exp.node.loop]] | [[ui:exp.portBody]], [[ui:exp.portDone]] | [[ui:exp.portLimit]] |
| 그 밖의 모든 노드 | [[ui:exp.outputPort]] | — |

연결하려면 출력에서 노드로 끌어다 놓습니다. 빈 캔버스에 놓으면 그 자리에 새 노드가 추가됩니다.
이미 연결이 있는 출력에서 끌면 연결이 하나 더 추가됩니다. [[ui:exp.addNext]], <kbd>A</kbd> 키,
연결 위의 ＋는 대신 기존 연결에 노드를 삽입합니다.

미완성 그래프는 초안입니다. 저장되지만 실행되지는 않습니다. 도구 모음의 [[ui:exp.needsLinks]]가
무엇이 빠졌는지 알려 주고 그 노드를 보여 줍니다. [실행 전에 검사하는 것](#validation)을
참고하십시오.

## 병렬 분기 {#parallel}

### 하나의 출력에서 나온 여러 연결 {#fan-out}

출력에 여러 연결이 있으면 — [[ui:exp.node.start]]의 것도 포함해 — 그 연결이 이끄는 모든 노드가
동시에 실행됩니다. 첫 연결은 분기를 이어 가고, 그 뒤의 연결마다 병렬 분기가 시작됩니다. 각 분기는
변수와 최신 HTTP 응답의 자기 사본을 나르므로, 한 분기가 설정하거나 받은 것을 다른 분기가 보지
못합니다.

### 병렬 분기와 합류 {#fork-join}

[[ui:exp.node.fork]]는 곧바로 통과하며 [[ui:exp.branch1]]과 [[ui:exp.branch2]] 양쪽으로 나갑니다.
하나의 출력에서 나온 두 연결을 노드로 그린 것과 같습니다.

[[ui:exp.node.join]]는 자기에게 들어오는 **모든** 연결을 기다린 뒤, 그 연결들의 순서대로 사본을
병합해 하나의 분기로 계속합니다:

- 모든 연결의 변수 — 두 분기가 모두 설정한 이름은 실험에서 뒤에 나열된 연결이 이깁니다.
- 그 순서에서 응답을 가져오는 마지막 연결의 HTTP 응답.
- 그 뒤의 대기들에는 그들의 마지막 동작 중 가장 이른 것.

연결의 순서가 결정하며, 어느 분기가 우연히 먼저 끝났는지는 상관없습니다.

::: warning 병렬로 실행되는 것만 합류시키십시오
합류는 어떻게 병렬로 실행되게 되었든 연결을 셉니다. [[ui:exp.node.fork]]에서 왔든, 하나의 출력에서
나온 여러 연결이든, 서로 다른 경로든 마찬가지입니다. 분기의 [[ui:exp.yes]]와 [[ui:exp.no]]
뒤에서는 경로 하나만 실행되므로, 둘 다로부터 들어오는 합류는 오지 않을 분기를 기다립니다: 실행은
`run.join_waiting`으로 실패하며, 몇 개의 연결이 끝내 따라오지 않았는지 알려 줍니다. 대안 경로를
다시 합치려면 다음 노드로 곧바로 연결하십시오.
:::

합류가 아닌 노드에 두 병렬 분기가 도달하면, 그 분기마다 한 번씩 실행됩니다.

### 단계가 실패할 때 {#failure}

첫 실패가 실행을 실패시킵니다. 다른 분기는 새 단계를 시작하지 않습니다: 반복이나 부하는 일찍
끝나고, 그 밖에 진행 중이던 단계는 끝까지 실행됩니다. 그동안 만나는 실패는 타임라인에 보고되지만
실행의 오류는 아닙니다. [[ui:exp.portTimeout]] 연결이 있는데도 시간 초과를 만난 단계는 실패한 것이
아닙니다 — [대기](#timeout)를 참고하십시오.

## 분기 {#branching}

| 노드 | [[ui:exp.yes]]로 나가는 경우 |
| --- | --- |
| [[ui:exp.node.branch_status]] | 이 경로의 최신 HTTP 응답이 주어진 상태일 때 |
| [[ui:exp.node.branch_value]] | 그 비교가 성립할 때 — [값 비교하기](data.md#compare)를 참고하십시오 |

그렇지 않으면 각각 [[ui:exp.no]]로 나갑니다. 상태 분기는 모든 경로에서 그 앞에 HTTP 요청이 있어야
하고, 값 분기는 읽는 이름들이 그곳에서 알려져 있어야 합니다. [[ui:exp.yes]]와 [[ui:exp.no]] 뒤의
경로가 다시 만나면, 만나는 노드는 한 번 실행되며, 두 경로 모두에서 설정한 변수만 그곳에서
알려집니다([변수가 알려지는 곳](data.md#visibility)).

## 재시도 {#retry}

보내거나 수신하는 단계는 실패했을 때 다시 시도할 수 있습니다: 속성에서 [[ui:exp.retryOn]]을
켜십시오.

| 설정 | 내용 | 범위 | 처음 값 |
| --- | --- | --- | --- |
| [[ui:exp.attempts]] | 첫 번째를 포함한 전체 시도 횟수 | 1–10 | 3 |
| [[ui:exp.retryDelay]] | 두 번째 시도 전의 일시 정지 | 0–60 000 ms | 500 |
| [[ui:exp.backoff]] | [[ui:exp.backoff.fixed]]: 모든 일시 정지가 같음; [[ui:exp.backoff.exponential]]: 일시 정지마다 앞의 두 배 | — | [[ui:exp.backoff.fixed]] |

- 재시도는 [[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.mqtt]],
  [[ui:exp.node.osc]], [[ui:exp.node.udp]], [[ui:exp.node.ws_connect]],
  [[ui:exp.node.ws_send]]와 모든 대기에 적용됩니다. 다른 노드는 거부하며
  (`node.retry_unsupported`), [부하](load.md)가 걸린 HTTP 요청은 재시도를 받지 않습니다.
- 두 배로 늘려도 일시 정지 하나가 60초를 넘지 않습니다.
- 실패한 시도마다 [[ui:exp.retry]]로 타임라인에 나타나며, 횟수와 이유가 표시됩니다. 그 뒤 단계가
  통과하거나, 마지막 시도의 이유로 실패합니다.
- 실행만 반복됩니다. 템플릿이 해석되지 않는 필드는 곧바로 실패합니다.
- 회신을 기다리는 전송은 다시 보냅니다. 대기는 앞에서처럼 분기의 마지막 동작부터 세어 다시
  기다립니다.
- [[ui:exp.portTimeout]] 연결이 있는 대기는 시간 초과로 실패하지 않으므로 재시도하지 않습니다:
  [[ui:exp.portTimeout]]을 따릅니다.
- [중지](#stop)는 일시 정지를 곧바로 끝냅니다.

## 반복 {#repeat}

보내는 단계는 그래프에 루프 없이도 계속 보낼 수 있습니다 — 하트비트, 폴링, 꾸준한 스트림.
[[ui:exp.repeatOn]]을 켜십시오.

| 설정 | 내용 | 범위 | 처음 값 |
| --- | --- | --- | --- |
| [[ui:exp.repeatBy]] | [[ui:exp.repeatBy.count]] 또는 [[ui:exp.repeatBy.duration]] | — | [[ui:exp.repeatBy.count]] |
| [[ui:exp.repeatCount]] | 첫 번째를 포함한 전체 전송 횟수 | 2–10 000 | 10 |
| [[ui:exp.repeatDuration]] | 첫 전송부터 계속 보낼 시간 | 1–300 000 ms | 10 000 |
| [[ui:exp.repeatInterval]] | 두 전송 사이의 일시 정지 | 10–60 000 ms | 1000 |
| [[ui:exp.repeatJitter]] | 일시 정지마다 무작위로 최대 이만큼 더 길어짐 | 0–60 000 ms | 0 |

- 반복은 [[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.mqtt]],
  [[ui:exp.node.osc]], [[ui:exp.node.udp]], [[ui:exp.node.ws_send]]에 적용됩니다(그 밖의 곳에서는
  `node.repeat_unsupported`). HTTP 요청은 반복과 [부하](load.md) 중 하나만 가지며, 둘 다는
  아닙니다.
- 각 전송은 한 번 보낼 때와 똑같이 이루어집니다: 템플릿을 다시 읽고 — `{{counter}}`는 전송 번호,
  `{{now}}`는 그 시각 — 재시도가 켜져 있으면 전송마다 적용됩니다. 회신을 기다리는 전송은 자기
  회신을 기다립니다.
- 시간 기준일 때는 시간이 다 되기 전에 시작할 수 있는 전송만 합니다.
- 지터는 실행의 시드에서 뽑습니다: 같은 시드는 같은 일시 정지를 줍니다.
- 타임라인은 진행 상황을 1초에 최대 한 번 [[ui:exp.repeating]]으로 보고합니다. 단계는 마지막 전송
  뒤에 그 전송의 결과와 함께 통과합니다. 끝내 실패하는 전송은 단계를 실패시킵니다.
- 다른 분기의 실패는 전송을 끝냅니다. [중지](#stop)는 일시 정지를 곧바로 끝냅니다.

실행 안에 들어가야 합니다: 전송과 그 최장 일시 정지(`(count − 1) × (interval + jitter)`)가 최대
300초이고(`node.repeat_too_long`), 시간 기준 반복은 최대 10 000회
전송입니다(`node.repeat_too_many`).

## 루프 {#loop}

[[ui:exp.node.loop]]는 [[ui:exp.portBody]] 출력의 단계를 계속 반복합니다. 그중 마지막 단계가
루프로 다시 연결됩니다.

| 설정 | 내용 | 범위 |
| --- | --- | --- |
| [[ui:exp.loopMax]] | 최대 반복 횟수 | 1–1000 |
| [[ui:exp.loopUntilOn]] | 종료 조건: [[ui:exp.node.assert_value]]에서처럼 [[ui:exp.value]], [[ui:exp.operator]], [[ui:exp.expected]] | 선택 사항 |

1. 바깥에서 도달하면 루프는 [[ui:exp.portBody]]에서 반복 1을 시작합니다.
2. 본문이 돌아올 때마다 종료 조건을 읽습니다 — 반복 뒤에 읽으므로 본문은 항상 최소 한 번
   실행되고 검사 대상을 설정할 수 있습니다.
3. 조건이 성립하면 루프는 [[ui:exp.portDone]]으로 나갑니다.
4. 그렇지 않으면 남은 반복이 있는 동안 다음 반복을 시작합니다.
5. 반복이 먼저 소진되면 루프는 [[ui:exp.portLimit]]이 연결되어 있으면 그리로 나가고, 아니면
   `loop.limit`으로 실행을 실패시킵니다. 조건이 없으면 본문은 매 반복 실행되고 루프는
   [[ui:exp.portDone]]으로 나갑니다.

본문 안에서 `{{counter}}`는 반복 번호입니다. 모든 노드가 자기 실행을 세기 때문입니다. 조건과
[[ui:exp.portDone]] 또는 [[ui:exp.portLimit]] 뒤의 단계는 본문의 모든 반복이 설정한 것 — 예를
들어 본문이 추출한 상태 — 을 쓸 수 있습니다. 본문 자체는 루프에 도달했을 때 알려져 있던 것만
봅니다.

[[ui:exp.templatePoll]] 템플릿은 장치가 `ready`라고 답할 때까지 0.3초마다 상태를 묻고, 최대
10번까지 합니다.

### 본문이 담을 수 있는 것 {#loop-body}

본문은 하나의 분기로, 반복을 차례로 실행합니다. 루프로 돌아가는 연결이 실험이 가질 수 있는 유일한
순환입니다. 그 밖의 것은 `graph.cycle`입니다.

| 규칙 | 오류 |
| --- | --- |
| [[ui:exp.portBody]]의 무언가가 루프로 돌아옴 | `loop.no_return` |
| 본문의 각 출력에 연결이 하나 | `loop.body_parallel` |
| 본문의 모든 출력이 본문 안으로 이어지거나 루프로 돌아감 | `loop.body_leaves` |
| 루프의 [[ui:exp.portBody]] 출력만이 본문으로 들어감 | `loop.body_entered` |
| 본문에 [[ui:exp.node.start]], [[ui:exp.node.end]], [[ui:exp.node.fork]], [[ui:exp.node.join]] 또는 다른 [[ui:exp.node.loop]]가 없음 | `loop.body_unsupported` |

## 대기 {#waits}

대기는 기다리는 메시지가 도착하면 통과합니다: [[ui:exp.node.wait_osc]],
[[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_http]],
[[ui:exp.node.wait_ws]]. 각각 무엇을 일치시키는지는 [노드 참조](nodes.md)에 있고, 여기서는 수신
방식만 다룹니다.

### 시작부터 수신하기 {#listening}

실행은 대기가 수신하는 것을 **첫 단계 전에** 엽니다. 다음 단계보다 빠른 회신을 놓치지 않기
위해서입니다:

| 대기 | 첫 단계 전에 여는 것 |
| --- | --- |
| OSC, UDP | [[ui:exp.listenOn]] 주소마다 UDP 소켓 하나, 그 주소의 모든 대기가 공유 |
| HTTP 요청 | 주소마다 리스너 하나 — 실행의 HTTP [에뮬레이터](faults.md#emulator)가 있으면 그것, 없으면 `204`로 응답하는 것 |
| MQTT | 브로커와 토픽 필터마다 연결 하나, 구독함. 그때 브로커가 재생하는 retained 메시지는 무시함 |
| WebSocket | 없음: [[ui:exp.node.ws_connect]]가 실행될 때 연 연결을 읽음 |

먼저 열리기 때문에 이 주소들은 실행 전에 고정됩니다. OSC, UDP, HTTP 대기는 포트가 0이 아닌
리터럴 `IP:port`에서 수신하고, MQTT 대기의 브로커와 토픽은 매개변수만 받습니다. 열 수 없는 포트 —
이미 사용 중이거나 이 컴퓨터의 주소가 아닌 — 는 트래픽이 생기기 전에 그 대기의 필드에서 실행을
멈춥니다. 실행이 어떻게 끝나든 모든 것이 닫힙니다.

### 어떤 메시지를 세는가 {#counting}

대기는 **자기 분기의 마지막 동작**이 시작된 뒤에 도착한 메시지를 대상으로 합니다 — 마지막 요청,
메시지, 발행, WebSocket 연결 또는 전송 — 또는 동작이 하나도 없으면 실행이 시작된 뒤의 것을
대상으로 합니다. 요청 전에 온 메시지는 세지 않으며, 요청과 대기 사이의 지연이나 로그가 그 회신을
가리지 않습니다. 합류 뒤에는 병합된 분기들의 마지막 동작 중 가장 이른 것이 기준입니다.

대기는 일치하는 첫 메시지를 가져가 소비합니다. 두 대기가 같은 메시지를 일치시키는 일은 없습니다.

각 소켓, 구독, 연결은 자기 대기를 위해 메시지 최대 1024개와 64 MiB를 보관합니다. 그보다 많아지면
가장 오래된 것을 버리고 세어 둡니다.

### 시간 초과 {#timeout}

[[ui:exp.waitTimeout]]은 1–120 000 ms이고 처음 값은 2000입니다. 제때 아무것도 일치하지 않으면:

- [[ui:exp.portTimeout]] 연결이 있으면 대기는 그것을 따릅니다.
- 없으면 단계는 `wait.timeout`으로 실패하며, 그동안 다른 메시지가 몇 개 도착했는지 알려 줍니다 —
  잘못된 패턴은 조용한 장치와 다르게 보입니다 — 그리고 세부 정보에 대기열이 가득 찼을 때 오래된
  메시지를 몇 개 버렸는지도 알려 줍니다.

대기의 변수 — `reply`, HTTP는 `request` — 는 [[ui:exp.portMatched]] 뒤에만 존재합니다.
[[ui:dock.inspector]]가 캡처 중이면 단계는 일치시킨 프레임도 연결합니다:
[타임라인](runs.md#timeline)을 참고하십시오.

## 같은 단계에서 회신 받기 {#reply}

[[ui:exp.node.osc]]나 [[ui:exp.node.udp]]는 자기 답을 기다릴 수 있습니다:
[[ui:exp.expectReply]]를 켜십시오.

| 설정 | 내용 | 처음 값 |
| --- | --- | --- |
| [[ui:exp.replyOn]] | 답을 기다리는 주소, 포트 0은 아무 빈 포트 | `0.0.0.0:0` |
| [[ui:exp.replyAddress]] (OSC), [[ui:exp.replyMode]] (UDP) | 일치하는 대기에서처럼 답이 무엇이어야 하는지 | 아무거나 |
| [[ui:exp.waitTimeout]] | 1–120 000 ms | 2000 |
| [[ui:exp.replyVariable]] | 답을 쓰는 변수 | `reply` |

[[ui:exp.replyOn]]의 소켓은 대기와 마찬가지로 첫 단계 전에 열리며, 메시지가 **그 소켓에서
나갑니다**. 보낸 쪽의 포트로 답하는 장치는 들리고, 고정 포트로 답하는 장치는 그 포트가 지정된
포트일 때 들립니다. 단계는 일치하는 답과 함께 통과하고, 변수는 그 출력 뒤에 존재합니다.
[[ui:exp.portTimeout]] 출력은 없습니다: 제때 답이 없으면 단계가 실패하고, 재시도가 다시 보낼 수
있습니다. 침묵으로 분기하려면 별도의 대기를 쓰십시오.

## 실행 전에 검사하는 것 {#validation}

편집기는 편집하는 동안 실험을 검사하고, 실행 버튼이 한 번 더 검사합니다. 문제는 노드를 알려 주고,
필드가 있으면 필드도 알려 줍니다.

| 규칙 | 오류 |
| --- | --- |
| 실험에 이름이 있음 | `doc.name_required` |
| 노드 1–64개, 시작과 종료가 각각 정확히 하나 | `doc.node_count`, `doc.start_end_count` |
| 시작에 입력이 없음 | `graph.start_input` |
| 연결이 존재하는 다른 노드로 이어짐 | `doc.connection_invalid` |
| 같은 연결이 두 번 있지 않음 | `doc.connection_duplicate` |
| 연결해야 하는 모든 출력이 연결됨 | `graph.outputs_required` |
| 노드가 가지지 않은 출력에 연결이 없음 | `graph.port_unexpected` |
| 모든 노드가 시작에서 도달 가능 | `graph.unreachable` |
| 루프로 돌아가는 연결 외의 순환이 없음 | `graph.cycle`, 그리고 [본문 규칙](#loop-body) |
| 검사나 추출이 모든 경로에서 그 앞에 HTTP 요청이 있음. 부하가 걸린 요청은 세지 않음 | `graph.needs_http` |
| 모든 템플릿이 해석되고, 사용하는 모든 이름이 모든 경로에서 알려짐 | `template.*`, `name.*` — [데이터](data.md#unknown-names)를 참고하십시오 |
| 모든 필드가 존재하고 범위 안에 있음 | `node.*` |
| 다른 것을 지정하는 노드 — [[ui:exp.node.impairment_change]], [[ui:exp.node.emulator_state]], WebSocket 노드 — 가 있는 것을 지정하고, WebSocket 노드가 그 연결 뒤에 옴 | `impair.relay_unknown`, `emulator.node_unknown`, `ws.connection_unknown`, `ws.connection_after` |
| 실행의 두 소켓이 포트를 공유하지 않음 | [장애 주입](faults.md#ports)을 참고하십시오 |

그런 다음 실행은 시작에 필요한 것을 검사합니다: 모든 [시크릿](data.md#secret-check)이 저장되어
있는지, 모든 포트가 열리는지입니다. 모든 것이 갖추어질 때까지 어떤 단계도 실행되지 않고 아무것도
보내지 않습니다.

## 시간 제한 {#limit}

실행은 최대 300초 지속됩니다. 그때까지 계속되는 실행은 중지되고 `run.timeout`으로 실패합니다.
[명령줄](../automation/cli.md#cli-run)과 [API](../api/run.md)에서는 한도를 더 짧게, 1–300초로 할
수 있습니다.

## 중지 {#stop}

실행이 진행되는 동안 실행 버튼은 [[ui:common.stop]]입니다. 중지는 실행을 곧바로 끝냅니다: 모든
분기, 재시도나 반복의 모든 일시 정지, 모든 대기와 모든 부하 — 진행 중인 요청은 버려집니다. 소켓,
구독, 에뮬레이터, 릴레이가 닫히고, WebSocket 연결은 닫기 프레임을 보냅니다. 헤더의
[[ui:app.stopAll]]은 모든 작업에 같은 일을 합니다. 중지된 실행은 보고서를 저장하지 않습니다.
[실행](runs.md#stop)을 참고하십시오.

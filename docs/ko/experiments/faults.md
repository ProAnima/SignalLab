---
title: 장애 주입
description: 네트워크 장애 릴레이와 에뮬레이터를 실행의 노드로 사용합니다. 나쁜 네트워크와 실패하는 의존 서비스를 원하는 때 켜고, 보고서에 구간별로 집계하며, 시드로 반복할 수 있습니다.
---

# 노드로 주입하는 장애

네트워크 품질이 떨어지거나 의존 서비스가 다운될 때 시스템이 어떻게 대처하는지 보려면 장애를 실험에
넣으십시오. 릴레이나 에뮬레이터가 실행과 함께 열리고, 단계가 원하는 때에 상태를 전환하며, 실행
보고서가 각 구간에서 일어난 일을 집계합니다. 실행이 통과하든, 실패하든, 중지되든 끝날 때 이들을
닫으므로, 실행이 끝난 뒤에 장애가 남아 있는 일은 없습니다.

| 노드 | 기능 |
| --- | --- |
| [[ui:exp.node.impairment]] | 테스트 대상 시스템과 그 대상 사이에서 실행 내내 통과하는 트래픽에 장애를 적용하는 릴레이 |
| [[ui:exp.node.impairment_change]] | 실행의 릴레이를 이 단계부터 다른 프로필로 전환 |
| [[ui:exp.node.emulator]] | 실행 내내 Signal Lab이 맡는 API, 장치, 브로커 |
| [[ui:exp.node.emulator_state]] | 실행의 에뮬레이터를 다운시키거나 다시 복구 |

네 노드 모두 추가 메뉴의 [[ui:exp.group.fault]] 및 [[ui:exp.group.emulate]] 아래에 있습니다. 필드는
[노드 참조](nodes.md)에 있으며, 릴레이 자체는 [네트워크 장애](../tools/impairment.md)에서,
에뮬레이터는 [에뮬레이터](../tools/emulators.md)에서 설명합니다.

## 네트워크 장애 {#impairment}

테스트 대상 시스템은 실제 대상 대신 릴레이로 보냅니다. 릴레이는 대상으로 전달하고, 응답을 다시
가져오며, 양방향 모두에 장애를 적용합니다.

| 필드 | 내용 |
| --- | --- |
| [[ui:exp.relayListen]] | 테스트 대상 시스템이 보내거나 연결하는 `IP:port`, 포트는 0이 아니어야 합니다 |
| [[ui:exp.relayTarget]] | 실제 목적지의 `IP:port`, 또는 `host:port`; 호스트 이름은 실행 시작 시 확인합니다 |
| [[ui:ns.protocol]] | UDP(데이터그램마다 따로 결과가 정해짐) 또는 TCP(연결마다 대상으로 가는 연결을 하나씩 이어 줌) |
| 프로필 | [[ui:ns.preset]] 또는 직접 정한 값 |

릴레이가 프로필에서 읽는 값은 프로토콜에 따라 다르며, 나머지 값은 무시합니다.

| 프로토콜 | 장애 |
| --- | --- |
| UDP | 지연 시간, 지터, 패킷 손실, 버스트 손실, 중복, 손상, 순서 바뀜, 대역폭 제한, 오프라인 |
| TCP | 지연 시간과 지터(스트림의 순서는 유지), 대역폭 제한(송신 측이 느려질 뿐 버리는 데이터 없음), 연결 리셋, 반개방 상태로 남은 연결, 오프라인 |

**첫 단계 전에 열립니다.** 실험의 모든 릴레이는 대기 노드의 소켓처럼 실행이 시작될 때 열리므로,
[[ui:exp.relayListen]]과 [[ui:exp.relayTarget]]에는 텍스트와 매개변수만 쓸 수 있습니다
(`node.params_only`). 매개변수 `relay`를 쓰는 `{{relay}}`는 되지만 변수는 안 됩니다. 열 수 없는
포트가 있거나 확인할 수 없는 대상 호스트 이름이 있으면 트래픽이 생기기 전에 그 노드에서 실행이 멈춥니다.

**흐름에서는 바로 통과합니다.** 실행이 노드에 도달하면 바로 통과하며, 타임라인에 무엇에 어떤 장애를
적용하는지 표시됩니다. 릴레이는 노드가 그래프의 어디에 있든 실행의 시작부터 끝까지 동작합니다.

**실행과 함께 닫힙니다.** 실행이 어떻게 끝나든 릴레이는 닫히며, TCP 릴레이의 연결도 함께 닫힙니다.
스스로 중계를 멈춘 릴레이는 그 이유를 보관합니다. 그 릴레이를 쓰는 단계는 그 이유로 실패하고,
보고서에도 그렇게 기록됩니다.

::: tip 네트워크 장애를 거쳐 보내기
[[ui:exp.node.osc]] 노드나 [[ui:exp.node.udp]] 노드에서 [[ui:exp.routeThrough]] 기능을 쓰면 노드
앞에 네트워크 장애 노드를 둡니다. 릴레이는 `127.0.0.1`의 빈 포트에서 수신하고 [[ui:ns.preset.lan]]
프리셋으로 노드의 대상에 전달하며, 노드는 이제 릴레이로 보냅니다.
:::

## 장애 변경 {#change-impairment}

[[ui:exp.node.impairment_change]] 노드는 [[ui:exp.relay]] 필드에서 실험의 릴레이 하나를 지정하고,
그 단계부터 적용할 프로필을 정합니다. 릴레이는 포트와 연결을 그대로 유지하며, 새 값은 다음 패킷이나
청크부터 적용됩니다. 타임라인에 새 프로필이 표시됩니다.

변경할 때마다 **구간**이 하나 끝납니다. 실행 보고서는 릴레이마다 다음을 보관합니다.

- 수신 주소와 대상 주소, 그리고 TCP이면 프로토콜.
- 전체 집계: 수신, 전달, 버림, 대역 제한, 중복, 손상, 순서 바뀜, 바이트. TCP이면 연결 수, 리셋된
  연결, 반개방 상태로 남은 연결도 포함됩니다.
- 각 구간: 프로필 이름, 릴레이가 열린 순간부터 밀리초 단위로 센 구간의 시작과 끝, 그리고 그 구간만의
  같은 집계.

패킷은 지연된 사본이 전환 뒤에 나가더라도 그 운명을 결정한 구간에 집계됩니다. 릴레이는 마지막 구간
1000개를 보관하며, 그보다 오래된 구간은 집계만 하고 보관하지 않습니다.

실험의 어떤 릴레이도 지정하지 않은 [[ui:exp.node.impairment_change]] 노드는 거부됩니다
(`impair.relay_unknown`).

## 에뮬레이터 {#emulator}

[[ui:exp.node.emulator]] 노드는 실행 내내 의존 대상의 역할을 맡습니다. HTTP API, OSC·UDP·TCP 장치,
MQTT 브로커입니다. [에뮬레이터](../tools/emulators.md) 화면이 단독으로 실행하는 것과 같은
에뮬레이터이며, [[ui:emu.edit]] 버튼은 규칙을 열고, [[ui:emu.toLibrary]] 버튼은 라이브러리에 사본을
보관하며, [[ui:emu.fromLibrary]] 버튼은 라이브러리에서 하나를 가져옵니다.

- 첫 단계 전에 열려 실행이 끝날 때까지 응답합니다. 열 수 없는 포트가 있으면 트래픽이 생기기 전에
  실행이 멈춥니다. 흐름에서는 바로 통과합니다.
- 주소는 리터럴 `IP:port`입니다. 일치 패턴에는 매개변수만 쓸 수 있으며, 회신은 도착한 내용
  (`{{request.…}}`)과 실행의 매개변수로 읽는 템플릿입니다. 시크릿은 읽을 수 없습니다.
- 응답의 가중치 혼합, 지연 지터, 회신의 생성기 같은 무작위 선택은 실행의 시드에서 뽑습니다.
- HTTP 에뮬레이터는 같은 주소의 [[ui:exp.node.wait_http]] 노드가 수신하는 대상이기도 하며, 테스트 대상
  시스템이 보낸 내용을 검사합니다. 그 주소에 에뮬레이터가 없으면 실행 자체의 리스너가 모든 요청에
  `204`로 응답합니다.
- OSC나 UDP 에뮬레이터는 그 포트의 실행 대기 노드와 포트를 공유하며, 양쪽 모두 모든 데이터그램을
  봅니다.
- MQTT 에뮬레이터는 실행의 [[ui:exp.node.mqtt]] 노드와 [[ui:exp.node.wait_mqtt]] 노드가 다른 브로커처럼
  사용할 수 있는 브로커입니다.

실행 보고서는 에뮬레이터 노드마다 이름, 프로토콜, 주소와 집계를 보관합니다. 전체 요청, 어떤 규칙도
받지 않은 요청, 실패한 요청, 다운 상태에서 받은 요청, 브로커가 느린 클라이언트에게 전달하지 못한
메시지, 각 규칙의 일치 횟수입니다.

## 에뮬레이터 다운과 복구 {#emulator-state}

[[ui:exp.node.emulator_state]] 노드는 [[ui:exp.emulatorNode]] 필드에서 실행의 에뮬레이터 하나를
지정하며, [[ui:exp.emulatorDownState]] 필드는 [[ui:exp.emulatorGoesDown]] 또는
[[ui:exp.emulatorComesUp]]입니다. 다운된 동안에는 다음과 같습니다.

| 에뮬레이터 | 겪는 일 |
| --- | --- |
| HTTP | [[ui:exp.downFault]] 설정에 따릅니다: [[ui:emu.outageFault.unavailable]] (`503`, 본문 `{"error":"unavailable"}`), [[ui:emu.outageFault.reset]], [[ui:emu.outageFault.timeout]] (클라이언트가 포기할 때까지, 최대 120초 동안 요청을 붙잡아 둠) |
| TCP 장치, MQTT 브로커 | 연결이 끊기고 새 연결은 거부됩니다 |
| OSC, UDP 장치 | 아무것도 응답하지 않습니다 |

다운된 동안 도착한 것은 `down`으로 집계되며, 어떤 규칙도 받지 않은 요청으로는 절대 집계되지 않습니다.
[[ui:exp.node.wait_http]] 노드는 여전히 요청을 봅니다. 에뮬레이터는 자체 다운 일정과 관계없이 어떤
단계가 복구할 때까지 다운 상태로 남으며, 어느 경우든 실행이 끝나면 닫힙니다.

실험의 어떤 에뮬레이터도 지정하지 않은 [[ui:exp.node.emulator_state]] 노드는 거부됩니다
(`emulator.node_unknown`).

## 일정에 따른 다운 {#outage}

에뮬레이터는 스스로 다운될 수도 있습니다. 규칙에서 [[ui:emu.outage]] 옵션을 켜면 [[ui:emu.outageUp]]과
[[ui:emu.outageDown]]을 각각 10–3,600,000 ms로, HTTP의 경우 [[ui:emu.outageFault]]도 설정합니다. 열린
시점부터 세어 첫 번째 시간 동안 응답하고, 두 번째 시간 동안 다운되기를 반복합니다. 실행에서는 첫 단계
전부터 셉니다. 일정에 따라 다운된 동안 HTTP 에뮬레이터의 `503`에는 복구될 때까지 남은 초(정수, 최소
1)를 담은 `Retry-After`가 붙습니다. [[ui:exp.node.emulator_state]] 단계가 다운 상태로 붙잡고 있는
동안의 `503`에는 언제 끝날지 아무도 모르므로 이 헤더가 없습니다.

일정에는 단계가 필요 없고, 단계에는 일정이 필요 없습니다. 오락가락하는 의존 서비스에는 일정을,
흐름의 정해진 지점에서 일어나는 다운에는 단계를 쓰십시오.

## 예: 느린 링크 뒤의 다운 {#example}

클라이언트가 릴레이를 거쳐 API에 주문을 요청합니다. 요청하는 동안 두 번째 분기가 링크를
[[ui:ns.preset.4g]]로 느리게 만들고, API를 2초 동안 다운시킨 뒤 복구하고, 링크를 다시 깨끗하게
되돌립니다. 클라이언트는 응답을 받을 때까지 계속 요청해야 합니다.

```text
start → orders → link → split
split ─ branch1 → settle → until_ok ─ done → answered → joined
                           until_ok ─ body → get → status → pause → until_ok
split ─ branch2 → slow → down → outage → up → clean → joined
joined → end
```

이름은 아래 파일에 있는 노드의 id입니다.

1. 매개변수 `api` = `http://127.0.0.1:18091`을 추가합니다. API가 아니라 릴레이의 주소입니다.
2. [[ui:exp.node.emulator]] 노드를 추가합니다. HTTP, `127.0.0.1:18090`, `GET /orders/:id`
   라우트가 `200`과 `{"order":"{{request.params.id}}"}`로 응답합니다.
3. 그 뒤에 [[ui:exp.node.impairment]] 노드를 둡니다. [[ui:exp.relayListen]]
   `127.0.0.1:18091`, [[ui:exp.relayTarget]] `127.0.0.1:18090`,
   [[ui:ns.protocol]] TCP, 프리셋 [[ui:ns.preset.lan]].
4. 그 뒤에 [[ui:exp.node.fork]] 노드를 둡니다.
5. [[ui:exp.branch1]]에는 클라이언트를 둡니다. 300 ms [[ui:exp.node.delay]] 노드, 그다음
   [[ui:exp.node.loop]] 노드([[ui:exp.loopMax]] 40, [[ui:exp.loopUntilOn]]
   `{{status}}` [[ui:exp.op.eq]] `200`)입니다. 루프의 본문은 `GET {{api}}/orders/42`를 보내는
   [[ui:exp.node.http]] 노드, [[ui:exp.from.status]] 값을 `status`에 넣는 [[ui:exp.node.extract]]
   노드, 250 ms 지연이며, 루프로 다시 연결합니다. [[ui:exp.portDone]] 출력에는
   [[ui:exp.node.log]] 노드 `Orders API answers again: HTTP {{status}}`를 둡니다.
6. [[ui:exp.branch2]]에는 장애를 둡니다. 릴레이를 [[ui:ns.preset.4g]]로 바꾸는
   [[ui:exp.node.impairment_change]] 노드, Orders API를 [[ui:emu.outageFault.unavailable]] 방식으로
   [[ui:exp.emulatorGoesDown]] 상태로 만드는 [[ui:exp.node.emulator_state]] 노드, 2000 ms 지연,
   API를 [[ui:exp.emulatorComesUp]] 상태로 되돌리는 또 하나의 [[ui:exp.node.emulator_state]] 노드,
   다시 [[ui:ns.preset.lan]]으로 되돌리는 또 하나의 [[ui:exp.node.impairment_change]] 노드입니다.
7. 두 분기를 [[ui:exp.node.join]] 노드에 연결하고, 그 합류 노드를
   [[ui:exp.node.end]] 노드에 연결합니다.
8. 실행합니다.

타임라인에는 느린 링크를 거친 클라이언트의 요청이 `503`을 받고, API가 복구된 뒤 `200`을 받고, 루프가
[[ui:exp.portDone]] 출력으로 나가는 과정이 표시됩니다. 보고서에는 다운된 API를 만난 요청 다섯 개
안팎과 라우트가 응답한 요청 하나, 그리고 릴레이의 세 구간([[ui:ns.preset.lan]] 잠깐,
다운 동안 [[ui:ns.preset.4g]], 다시 [[ui:ns.preset.lan]])이 각 구간의 트래픽과 함께 집계됩니다.

::: details 파일로 된 실험
`.json` 파일로 저장한 뒤 [[ui:exp.documents]] 메뉴의 [[ui:exp.importJson]]으로 여십시오.

```json
{
  "version": 9,
  "name": "Outage behind a slow link",
  "params": [{ "name": "api", "value": "http://127.0.0.1:18091" }],
  "profiles": [],
  "profile": null,
  "seed": null,
  "nodes": [
    { "id": "start", "type": "start", "x": 40, "y": 270 },
    { "id": "orders", "type": "emulator", "x": 260, "y": 270,
      "emulator": { "name": "Orders API", "bind": "127.0.0.1:18090", "protocol": "http",
        "routes": [{ "method": "GET", "path": "/orders/:id", "when": [], "order": "sequence",
          "responses": [{ "status": 200, "headers": [], "body": "{\"order\":\"{{request.params.id}}\"}", "delay_ms": 0, "jitter_ms": 0, "fault": "none", "weight": 1 }] }],
        "fallback": null } },
    { "id": "link", "type": "impairment", "x": 490, "y": 270, "listen": "127.0.0.1:18091", "target": "127.0.0.1:18090", "protocol": "tcp",
      "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } },
    { "id": "split", "type": "fork", "x": 720, "y": 270 },
    { "id": "settle", "type": "delay", "x": 950, "y": 140, "ms": 300 },
    { "id": "until_ok", "type": "loop", "x": 1180, "y": 140, "max": 40,
      "until": { "value": "{{status}}", "op": "eq", "expected": "200" } },
    { "id": "get", "type": "http", "x": 1410, "y": 20,
      "request": { "method": "GET", "url": "{{api}}/orders/42", "headers": [], "body": null, "timeout_ms": 3000 } },
    { "id": "status", "type": "extract", "x": 1640, "y": 20, "variable": "status", "from": "status", "expr": "" },
    { "id": "pause", "type": "delay", "x": 1870, "y": 20, "ms": 250 },
    { "id": "answered", "type": "log", "x": 1410, "y": 140, "message": "Orders API answers again: HTTP {{status}}" },
    { "id": "slow", "type": "impairment_change", "x": 950, "y": 400, "relay": "link",
      "profile": { "name": "4g", "latency_ms": 60, "jitter_ms": 25, "rate_kbps": 20000 } },
    { "id": "down", "type": "emulator_state", "x": 1180, "y": 400, "emulator": "orders", "down": true, "fault": "unavailable" },
    { "id": "outage", "type": "delay", "x": 1410, "y": 400, "ms": 2000 },
    { "id": "up", "type": "emulator_state", "x": 1640, "y": 400, "emulator": "orders", "down": false, "fault": "unavailable" },
    { "id": "clean", "type": "impairment_change", "x": 1870, "y": 400, "relay": "link",
      "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } },
    { "id": "joined", "type": "join", "x": 2100, "y": 270 },
    { "id": "end", "type": "end", "x": 2330, "y": 270 }
  ],
  "edges": [
    { "from": "start", "to": "orders" },
    { "from": "orders", "to": "link" },
    { "from": "link", "to": "split" },
    { "from": "split", "to": "settle", "port": "branch1" },
    { "from": "split", "to": "slow", "port": "branch2" },
    { "from": "settle", "to": "until_ok" },
    { "from": "until_ok", "to": "get", "port": "body" },
    { "from": "get", "to": "status" },
    { "from": "status", "to": "pause" },
    { "from": "pause", "to": "until_ok" },
    { "from": "until_ok", "to": "answered", "port": "done" },
    { "from": "answered", "to": "joined" },
    { "from": "slow", "to": "down" },
    { "from": "down", "to": "outage" },
    { "from": "outage", "to": "up" },
    { "from": "up", "to": "clean" },
    { "from": "clean", "to": "joined" },
    { "from": "joined", "to": "end" }
  ]
}
```
:::

Signal Lab의 [[ui:exp.documents]] 메뉴에는 같은 일을 다른 방식으로 하는 템플릿이 두 개 있습니다.
[[ui:exp.templateFaults]] 템플릿은 정상, 손실, 오프라인, 다시 정상으로 전환되는 UDP 릴레이를 거쳐
에뮬레이션된 장치로 데이터그램을 보내고, [[ui:exp.templateOutage]] 템플릿은 클라이언트가 계속 요청하는
동안 에뮬레이션된 API를 2초 동안 다운시킵니다.

## 포트 {#ports}

한 실행의 소켓(대기, 회신, 에뮬레이터, 릴레이)은 같은 프로토콜의 포트를 공유할 수 없습니다. UDP 소켓과
TCP 소켓은 같은 번호를 쓸 수 있습니다. 릴레이의 [[ui:exp.relayListen]]에서 `0.0.0.0` 주소는 같은
포트의 모든 주소와 충돌합니다.

| 소켓 | 포트를 공유할 수 없는 대상 |
| --- | --- |
| HTTP, TCP, MQTT 에뮬레이터 | 그중 다른 에뮬레이터(`emulator.bind_taken`) |
| OSC, UDP 에뮬레이터 | 그중 다른 에뮬레이터(`emulator.bind_taken`) |
| TCP, MQTT 에뮬레이터 | [[ui:exp.node.wait_http]] 노드(`emulator.bind_taken`) |
| UDP 릴레이의 [[ui:exp.relayListen]] | 다른 UDP 릴레이, OSC나 UDP 에뮬레이터, 대기 노드나 회신의 소켓(`impair.bind_taken`) |
| TCP 릴레이의 [[ui:exp.relayListen]] | 다른 TCP 릴레이, HTTP·TCP·MQTT 에뮬레이터, [[ui:exp.node.wait_http]] 노드(`impair.bind_taken`) |

의도적으로 공유하는 경우: HTTP 에뮬레이터와 그 주소의 [[ui:exp.node.wait_http]] 단계, OSC나 UDP
에뮬레이터와 그 포트의 대기 노드, 같은 주소의 대기 노드끼리.

릴레이는 직접이든 다른 릴레이를 거쳐서든 자기 자신에게 전달할 수 없습니다. 트래픽이 루프백에서 계속
돌게 되기 때문입니다(`impair.loop`). 장치 앞에 릴레이 두 개를 연달아 두는 것은 괜찮습니다.

## 장애가 있는 실행 반복하기 {#seed}

패킷을 잃을지, 중복할지, 손상시킬지, 붙잡아 둘지, 지터를 얼마나 줄지 등 릴레이가 내리는 모든 결정은
실행의 시드에서 방향별로, 패킷마다 따로 뽑습니다. 에뮬레이터의 무작위 선택도 시드에서 뽑습니다. 같은
시드와 같은 트래픽으로 다시 실행하면 같은 패킷이 같은 결과를 겪습니다. 한 번 본 실패를 다시 볼 수
있습니다.

시드를 유지하려면 타임라인에서 시드 옆의 [[ui:exp.pinSeed]] 버튼을 누르거나, [[ui:exp.runWith]]에서
그 시드로 실행하십시오. [시드](runs.md#seeds)를 참고하십시오. 시드로도 고정할 수 없는 것은
타이밍입니다. 테스트 대상 시스템이 언제 보내는지, 그래서 패킷이 어느 구간에 들어가는지입니다.

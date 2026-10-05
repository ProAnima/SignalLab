---
title: MQTT
description: MQTT 3.1.1 브로커에 연결하고, 브로커가 가진 모든 토픽을 라이브 트리로 보며, QoS 0, 1, 2로 발행하고, 유언 메시지를 알리고, 남아 있는 retained 값을 지웁니다.
---

# MQTT

[[ui:nav.mqtt]] 화면은 브로커를 살펴보고 그 안의 내용을 바꾸는 MQTT 클라이언트입니다.
연결하면 기본적으로 `#`를 구독합니다. 브로커가 가진 모든 토픽이 최신 값과 함께 트리로
쌓입니다. 거기서 발행하고, retained 값을 지우고, 토픽을 신호로 저장하거나 실험 단계로
바꿉니다.

Signal Lab은 **일반 TCP 위의 MQTT 3.1.1**을 사용하며, 구독, 발행, 유언 메시지에
QoS 0, 1, 2를 지원합니다. MQTT 5도 TLS도 없습니다. `mqtts://`나 MQTT 5 클라이언트만
받는 브로커에는 연결할 수 없습니다.

## 연결하기 {#connect}

1. [[ui:nav.mqtt]] 화면을 엽니다.
2. [[ui:mq.host]]와 [[ui:common.port]]를 입력합니다.
3. 브로커가 특정 id를 요구하지 않는 한 [[ui:mq.clientId]]는 그대로 둡니다. 브로커가
   요구할 때만 [[ui:mq.username]]과 [[ui:mq.password]]를 추가합니다.
4. [[ui:mq.connect]] 버튼을 누릅니다.

연결은 TCP 연결을 열고 무엇보다 먼저 MQTT 핸드셰이크를 완료하므로, 잘못된 비밀번호나 닫힌
포트가 바로 그 자리에서 보고됩니다. 연결된 동안에는 연결 필드가 잠깁니다.
[[ui:mq.disconnect]]가 연결을 닫습니다. 연결은 콘솔 표시줄의 작업이며 거기서 중지할 수도
있습니다.

| 필드 | 내용 | 기본값 |
| --- | --- | --- |
| [[ui:mq.host]] | 브로커의 IP 주소 또는 호스트 이름 | `127.0.0.1` |
| [[ui:common.port]] | 브로커의 포트 | `1883` |
| [[ui:mq.clientId]] | 브로커에서 이 클라이언트의 이름. 비워 둘 수 없고 그곳에서 유일해야 합니다. 같은 id의 두 번째 클라이언트가 들어오면 첫 번째 연결을 밀어냅니다. | `signal-lab-`과 임의의 hex 숫자 여섯 자리, 앱을 시작할 때마다 새로 생성 |
| [[ui:mq.username]], [[ui:mq.password]] | 브로커가 필요로 할 때만 보냅니다. TLS가 없으므로 평문입니다. 사용자 이름 없는 비밀번호는 아예 보내지 않습니다. MQTT 3.1.1이 실어 나를 수 없습니다. | 비어 있음 |
| [[ui:mq.keepAlive]] | 연결이 조용히 있어도 되는 시간(초). Signal Lab은 그 절반마다 브로커에 핑을 보냅니다. 이 값의 1.5배 동안 조용한 클라이언트는 브로커가 끊습니다. 0은 핑을 끕니다. | `60` |
| [[ui:mq.cleanSession]] | 켜면 모든 연결이 저장된 구독과 대기 중인 메시지 없이 시작합니다. 끄면 연결 사이에 이 클라이언트 id를 위해 보관해 달라고 브로커에 요청합니다. | 켬 |
| [[ui:mq.scanFilter]]와 그 [[ui:mq.qos]] | 연결이 되면 곧바로 구독하는 필터. `#`은 모든 토픽입니다. 비어 있으면 없음. | `#`, QoS 0 |
| [[ui:mq.willEnable]] | 브로커에 유언 메시지를 줍니다([아래](#will)) | 끔 |

브로커는 TCP 연결을 받아들이는 데 6초, 핸드셰이크에 응답하는 데 다시 6초가 주어집니다.

### 유언 메시지 {#will}

유언 메시지는 브로커가 대신 보관하다가, 연결이 제대로 작별하지 않고 끊기면 브로커가 스스로
발행하는 메시지입니다. 프레즌스는 보통 이렇게 만듭니다. 장치가 상태 토픽에 `online`을
발행하고, 그 유언이 같은 토픽을 `off`로 설정합니다.

[[ui:mq.willEnable]]을 선택한 상태에서 [[ui:mq.willTopic]]과
[[ui:mq.willPayload]] (기본값 `off`)를 설정합니다. 유언은 QoS 2로 retained로 발행됩니다.
토픽이 없으면 유언을 보내지 않습니다.

## 구독하기 {#subscribe}

스캔 필터는 연결할 때 구독합니다. 더 하려면:

1. [[ui:mq.addSubscription]]에 토픽 필터를 입력합니다.
2. 그 [[ui:mq.qos]]를 고릅니다.
3. [[ui:mq.subscribe]] 버튼을 누르거나 <kbd>Enter</kbd>를 누릅니다.

필터는 와일드카드가 있는 토픽입니다:

| 와일드카드 | 뜻 | 예 |
| --- | --- | --- |
| `+` | 정확히 한 단계 | `sensors/+/state`는 `sensors/door/state`와 일치 |
| `#` | 그 아래의 모든 단계, 마지막 글자일 때만 | `sensors/#`는 `sensors/door/state`와 `sensors`와 일치 |

[[ui:mq.subscriptions]]에는 각 필터와 브로커가 허락한 값이 나열됩니다. `qos0`, `qos1`,
`qos2`(브로커가 요청보다 낮게 허락할 수 있습니다) 또는 [[ui:mq.refused]]입니다.
[[ui:mq.unsubscribe]]는 그중 하나를 구독 취소합니다.

| QoS | 전달 |
| --- | --- |
| 0 | 최대 한 번: 보내고 잊습니다 |
| 1 | 최소 한 번: 확인 응답을 받으며, 두 번 도착할 수 있습니다 |
| 2 | 정확히 한 번: 두 단계 핸드셰이크이며, 재전달은 두 번 표시하지 않습니다 |

## 토픽 트리 {#topics}

도착하는 모든 메시지는 토픽 단계의 트리인 [[ui:mq.topics]]로 들어갑니다. 토픽은 최신 값을
보여 주고, 그 값이 retained이면 **R**을, 메시지가 둘 이상이면 받은 메시지 수를 보여 줍니다.
단계를 클릭하면 열거나 접습니다.

- 트리 위의 필드에 입력하면 경로나 최신 값에 그 텍스트가 들어 있는 토픽만 나열됩니다.
- 트리 위에는 토픽 수, retained 값을 가진 토픽 수, 연결 중일 때 수신 중인 브로커가
  표시됩니다.
- 페이로드는 텍스트로 표시되며, UTF-8이 아닌 바이트는 대체 문자로 나타납니다.
- [[ui:common.clear]]는 트리를 비웁니다. 그 밖에는 아무것도 비우지 않습니다. 화면을 바꾸거나
  연결을 끊어도 앱이 닫힐 때까지 그대로 남습니다.

메시지는 초당 열 번 묶음으로 화면에 도착합니다. 브로커가 0.1초 동안 4000개가 넘는 메시지를
보내면, 그 묶음에서 가장 오래된 것들이 트리에서 빠지고 그 위에 "표시되지 않음"으로
집계됩니다.

### 토픽의 패널 {#topic}

값이 있는 토픽을 고르면 트리 아래에 [[ui:mq.value]], [[ui:mq.qos]], [[ui:mq.retain]],
[[ui:common.bytes]], [[ui:mq.messages]], [[ui:mq.lastAt]]이 표시됩니다. 그 버튼은 다음과
같습니다:

| 버튼 | 하는 일 |
| --- | --- |
| [[ui:mq.editHere]] | 토픽, 값, QoS, retain 플래그를 [[ui:mq.publish]]로 복사합니다 |
| [[ui:mq.waitForThis]] | 열려 있는 실험에 이 토픽, 이 브로커, 아무 페이로드, 2000 ms 제한 시간의 [MQTT 기다리기](../experiments/nodes.md#node-wait_mqtt) 단계를 추가합니다 |
| [[ui:mq.clearRetained]] | retained 값을 제거합니다([아래](#clear-retained)) |
| [[ui:sig.fromFrame]] | 토픽과 최신 값을 [[ui:sig.capturedFolder]] 폴더의 신호로 보관합니다 |

## 발행하기 {#publish}

1. 연결합니다.
2. [[ui:mq.publish]] 아래에서 [[ui:mq.topic]]과 [[ui:sig.payload]]를 입력합니다.
3. [[ui:mq.qos]]를 고르고, 나중에 구독하는 모든 클라이언트를 위해 브로커가 그 메시지를
   토픽의 값으로 보관해야 하면 [[ui:mq.retain]]을 선택합니다.
4. [[ui:mq.publishBtn]] 버튼을 누릅니다.

콘솔은 발행할 때마다 확인해 줍니다. QoS 0이면 즉시, QoS 1과 2면 브로커가 확인 응답한
뒤입니다. 발행할 토픽에는 와일드카드가 없고 비어 있지 않아야 합니다. `+`나 `#`가 있는
토픽은 아무것도 보내기 전에 거부되며, 신호와 단계, `signallab send mqtt`가 주는 것과 같은
메시지가 나오고 연결은 그대로 유지됩니다. 와일드카드가 있는 필터를 받는 것은 구독뿐입니다.

### retained 값 지우기 {#clear-retained}

retained 값은 교체될 때까지 브로커에 남아 있고, 구독하는 모든 클라이언트가 그것을 먼저
받습니다. 오래된 값은 장치가 잘못된 상태로 부팅하는 전형적인 원인입니다. 그것을 제거하는
유일한 방법은 retain을 설정한 빈 페이로드를 발행하는 것입니다.

토픽 패널의 [[ui:mq.clearRetained]]가 그 일을 합니다. 누른 다음 [[ui:mq.clearConfirm]]을
누릅니다. 연결을 통해 QoS 1로 빈 retained 페이로드를 발행합니다. 연결되어 있으면서 토픽의
최신 값이 retained일 때만 쓸 수 있습니다. 직접 할 수도 있습니다. [[ui:mq.retain]]을 선택한
채 [[ui:sig.payload]]를 비우면 됩니다.

::: warning
지우기는 모든 클라이언트에 대해 브로커를 한 번에 바꿉니다.
:::

## 인스펙터에서 {#inspector}

캡처가 켜져 있으면 MQTT 트래픽이 `mqtt` 프로토콜로 나타납니다:

| 출처 | 내용 | 개수 |
| --- | --- | --- |
| `mqtt` | 화면의 연결이 발행하는 것. 빈 retained 발행은 판정이 `clears retained`입니다 | 모두 |
| `mqtt` | 연결이 받는 메시지 | 200 ms마다 최대 하나 |
| `mqtt-send` | 자체 연결을 가져온 발행: 화면이 신호의 브로커에 연결되어 있지 않을 때 발사한 신호, 단계, `signallab send mqtt`(판정 `one-shot`) | 모두 |
| `experiment-wait` | [[ui:exp.node.wait_mqtt]] 단계의 구독이 받는 메시지, retained 재생은 제외 | 모두 |

요약은 `topic = payload`로 읽히며, 해당되면 QoS와 `retained`가 붙습니다.
[인스펙터](../tools/inspector.md)를 참고하십시오.

## 저장하고 다시 쓰기 {#library}

- **신호로 저장.** [[ui:mq.publish]] 아래의 [[ui:sig.saveNew]]은 브로커(연결의
  [[ui:mq.host]]와 [[ui:common.port]]), 토픽, 페이로드, QoS, retain 플래그를 신호
  라이브러리에 보관합니다. 발행 패널에서 <kbd>Ctrl</kbd>+<kbd>S</kbd>를 눌러도 같은 일을
  하며, 패널이 그 신호에 연결되어 있으면 신호를 업데이트합니다.
  [신호](../tools/signals.md)를 참고하십시오.
- **MQTT 신호 발사.** 이 화면이 신호가 가리키는 브로커에 연결되어 있으면(호스트가 대소문자
  구분 없이 같고 포트도 같을 때, 신호에 포트가 없으면 `1883`), 라이브러리에서 발사한 신호가
  그 연결로 나가며 그 클라이언트 id와 자격 증명을 씁니다. 그렇지 않으면(연결되어 있지 않거나
  다른 브로커에 연결되어 있으면) 자기 브로커로 자체 연결을 엽니다(새 클라이언트 id, 사용자
  이름 없음). 발행하고, QoS가 요구하는 확인 응답을 기다린 뒤 연결을 끊습니다. 이름은 조회하지
  않으므로 `localhost`와 `127.0.0.1`은 다른 브로커로 칩니다. 라이브러리는 비밀번호를
  저장하지 않습니다.
- **실험에서.** 저장한 MQTT 신호는 실험의 [[ui:exp.addNode]] 메뉴에서
  [[ui:exp.group.signals]] 아래에 골라 넣을 수 있으며, 그러면 [[ui:exp.node.mqtt]] 단계가
  됩니다.

## 실험에서 {#experiments}

| 단계 | 하는 일 |
| --- | --- |
| [[ui:exp.node.mqtt]] | 연결하고 메시지 하나를 발행한 뒤 연결을 끊습니다. 사용자 이름도 비밀번호도 없고, clean session이며, 15초 안에 끝납니다. [자세히](../experiments/nodes.md#node-mqtt) |
| [[ui:exp.node.wait_mqtt]] | 실행이 시작될 때 구독하고, 페이로드가 일치하는 토픽 필터의 메시지를 기다립니다. 구독할 때 재생되는 retained 값은 무시합니다. [자세히](../experiments/nodes.md#node-wait_mqtt) |
| [[ui:exp.node.emulator]] | 실행 자체의 MQTT 브로커. [자세히](../experiments/nodes.md#node-emulator) |

두 단계 모두 로그인하지 않으므로, 사용자 이름 없이 클라이언트를 받는 브로커가 필요합니다.

### 브로커 에뮬레이터 {#broker-emulator}

Signal Lab이 브로커가 될 수도 있습니다. [[ui:emu.new.mqtt]] 에뮬레이터는 클라이언트가
발행한 것을 구독한 이들에게 라우팅하며(3.1.1, 일반 TCP, QoS 0, 1, 2, retained 메시지,
유언, 선택적 로그인), 장치처럼 규칙으로 응답합니다. 실제 브로커 없이 테스트하려면 장비와 이
화면을 그쪽으로 가리키십시오. [에뮬레이터](../tools/emulators.md)를 참고하십시오.

## 명령줄에서 {#cli}

`signallab send mqtt`는 자체 연결로 메시지 하나를 발행합니다:

```bash
signallab send mqtt 127.0.0.1:1883 lab/light/1/set on --qos 1
signallab send mqtt 127.0.0.1:1883 lab/light/1/state "" --retain
```

```text
✔ lab/light/1/set → 127.0.0.1:1883 · 2 B · qos1
```

두 번째 줄은 retained 값을 지웁니다. 포트가 없으면 브로커는 1883에 있습니다. 자격 증명은
쓰지 않습니다. 브로커가 메시지를 받으면 0, 연결할 수 없거나 거부하면 1로 끝납니다.
[명령줄](../automation/cli.md#cli-send-mqtt)을 참고하십시오.

## 문제 {#troubleshooting}

| 보이는 것 | 흔한 원인 |
| --- | --- |
| `… refused the connection — nothing is listening on that port` | 그 주소와 포트에 브로커가 없습니다. |
| `… accepted the connection but did not answer in time — is it an MQTT broker?` | 그곳에서 무언가가 수신하고 있지만 MQTT를 말하지 않거나, TLS 위에서 말합니다. |
| `… answered with something other than MQTT 3.1.1` | MQTT 브로커가 아니거나, Signal Lab이 읽을 수 없는 것을 보낸 브로커입니다. |
| `… does not accept MQTT 3.1.1 clients` | 브로커가 MQTT 5만 받습니다. |
| `… rejected the client ID — choose another one` | id가 너무 길거나 브로커가 받지 않는 문자가 있습니다. |
| `… rejected the username or password` | 잘못된 자격 증명이거나, 사용자 이름 없는 비밀번호입니다. |
| `… did not authorize this client — check its access rules` | 브로커의 접근 규칙이 이 클라이언트를 거부합니다. |
| `… is unavailable right now — try again later` | 브로커는 살아 있지만 클라이언트를 받지 않습니다. |
| `Enter a client ID — brokers refuse an empty one` | [[ui:mq.clientId]]가 비어 있습니다. |
| `A publish topic cannot contain the wildcards + or #` | 발행할 토픽에 `+`나 `#`가 있습니다. 그것은 구독용입니다. 한 번에 한 토픽에 발행하십시오. |
| 필터에 [[ui:mq.refused]]가 표시됨 | 브로커의 접근 규칙이 금지하거나, 필터가 잘못된 형식입니다(`#`가 마지막이 아니거나, `+`가 다른 문자와 한 단계를 공유). |
| 연결한 지 잠시 뒤 연결이 끊김 | 같은 [[ui:mq.clientId]]로 다른 클라이언트가 연결했습니다. |
| 트리에 아무것도 나타나지 않음 | 스캔 필터가 비어 있거나, 브로커가 이 클라이언트에게 아무것도 보여 주지 않습니다. |

모든 오류 메시지는 [오류 메시지](../reference/errors.md#mqtt)에 나열되어 있습니다.

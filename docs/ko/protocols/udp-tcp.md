---
title: UDP와 TCP
description: Signal Lab이 원시 UDP 데이터그램과 TCP 데이터를 보내고 받는 곳 — 실험 단계, 신호, 명령줄, 부하 및 스캔 도구, 에뮬레이션된 장치 — 과 페이로드를 쓰는 방법입니다.
---

# UDP와 TCP

원시 UDP와 TCP에는 전용 화면이 없습니다. 자체 텍스트나 이진 프로토콜을 쓰는 장치 — 프로젝터,
미디어 서버, 센서 — 에 사용하며, 여러 곳에 나타납니다:

| 하려면… | 사용 |
| --- | --- |
| 데이터그램이나 TCP 메시지를 단계로 보내고 답을 기다리려면 | [실험 단계](#experiments) |
| 데이터그램을 저장해 다시 보내거나, 캡처한 것을 재생하려면 | [UDP 신호](#signals) |
| 스크립트에서 데이터그램 하나를 보내려면 | [`signallab send udp`](#cli) |
| 여러 호스트에 한꺼번에, 브로드캐스트 주소나 멀티캐스트 그룹으로 보내려면 | [브로드캐스트](broadcast.md) 화면 |
| 서버나 링크에 트래픽 부하를 가하려면 | [Storm](#storm) |
| 호스트에서 열려 있는 TCP 포트를 찾으려면 | [스캐너](#scanner) |
| 장치의 역할을 대신하려면 | [UDP 또는 TCP 장치](#emulators) 에뮬레이터 |
| 두 끝 사이의 네트워크를 나쁘게 만들려면 | [장애 릴레이](#impairment) |

OSC는 UDP 데이터그램에 실려 가는 형식이며, 자체 페이지가 있습니다: [OSC](osc.md).

## 페이로드 {#payloads}

원시 페이로드를 쓰는 곳마다 그것은 두 가지 중 하나입니다:

| 종류 | 보내는 것 | 예 |
| --- | --- | --- |
| 텍스트 | 문자를 입력한 그대로 UTF-8로: 종결자도, 줄 끝도 덧붙이지 않습니다. 줄 프로토콜은 줄 끝을 텍스트에 넣어야 합니다. | `PING` |
| Hex 바이트 | 바이트 단위로, hex 숫자 쌍으로 씁니다. 쌍 사이의 공백, `:`, `-`, `,`와 앞의 `0x`가 허용됩니다. | `de ad be ef`, `DEADBEEF`, `0xde,0xad` |

데이터그램은 최대 65 507바이트를 담습니다. hex 숫자가 홀수 개이거나 하나도 없으면 아무것도 보내기
전에 오류입니다.

::: info
UDP에는 전달 확인이 없습니다. "보냄"은 데이터그램이 이 컴퓨터를 떠났다는 뜻일 뿐, 무언가가 받았다는
뜻이 아닙니다. 장치가 들었는지 알려면 그 답을 기다리십시오.
:::

## 목적지 {#destinations}

목적지는 IP 주소와 포트, 또는 호스트 이름과 포트입니다: `192.0.2.20:9000`,
`[2001:db8::20]:9000`(IPv6 주소는 대괄호에), `projector.local:9000`. 이는 UDP 단계의 대상, OSC
대상, UDP 신호, `signallab send udp`와 `signallab send osc`, [브로드캐스트](broadcast.md) 목록,
[Storm](../tools/storm.md)의 대상, TCP 단계의 호스트에 적용됩니다.

호스트 이름은 쓰일 때마다 조회됩니다. IPv4 주소가 있으면 그것을 사용하므로 — `localhost`가
`127.0.0.1`에서 수신하는 서비스에 도달합니다. 시스템이 먼저 나열하는 주소가 `::1`일 수 있습니다 —
IPv6 주소만 있는 이름은 IPv6 소켓에서 도달합니다. 해석되지 않는 이름은 `Cannot resolve …`로,
포트가 없거나 어느 형식도 아닌 목적지는 `… is not a valid address`로 실패합니다. 서비스가
*수신*하는 주소(모니터, 대기, 에뮬레이터의 주소)는 항상 `IP:port`입니다.

## 실험에서 {#experiments}

| 단계 | 하는 일 |
| --- | --- |
| [[ui:exp.node.udp]] | [[ui:exp.payload]]를 텍스트로 [[ui:common.target]]에 보냅니다. 대상은 `IP:port` 또는 `host:port`([위](#destinations))이고, 쉼표로 나눈 여러 대상은 각각 데이터그램을 받습니다. [[ui:exp.expectReply]]를 켜면 [[ui:exp.replyOn]]에서 보내고 같은 단계에서 그곳에서 답을 기다립니다. [자세히](../experiments/nodes.md#node-udp) |
| [[ui:exp.node.wait_udp]] | [[ui:exp.listenOn]] (`IP:port`)에서 수신하며 페이로드가 일치하는 데이터그램을 기다립니다. [자세히](../experiments/nodes.md#node-wait_udp) |
| [[ui:exp.node.tcp]] | [[ui:exp.host]]와 [[ui:exp.port]]에 연결해 [[ui:exp.payload]]를 텍스트로 쓰고, 250 ms 동안 답을 들은 뒤 닫습니다. 단계는 몇 바이트가 왔는지 말할 뿐 그 내용은 말하지 않습니다. [자세히](../experiments/nodes.md#node-tcp) |

UDP 페이로드와 대상, TCP 호스트와 페이로드, 대기의 패턴은 `{{templates}}`를 받으므로, 데이터그램이
실행의 id나 앞 단계가 추출한 값을 실을 수 있습니다. [데이터와 템플릿](../experiments/data.md)을
참고하십시오.

### 데이터그램 일치시키기 {#matching}

[[ui:exp.node.wait_udp]]와 [[ui:exp.node.udp]]의 회신은 페이로드로 데이터그램을 고릅니다:

| [[ui:exp.waitMode]] | 다음일 때 데이터그램을 받습니다 |
| --- | --- |
| [[ui:exp.mode.any]] | 항상: 가장 먼저 도착한 것 |
| [[ui:exp.mode.contains]] | 텍스트로 읽은 페이로드가 패턴을 포함 |
| [[ui:exp.mode.regex]] | 텍스트로 읽은 페이로드가 정규식과 일치 |
| [[ui:exp.mode.hex]] | 바이트가 hex로 쓴 패턴의 바이트를 포함 |

일치한 것은 단계의 변수([[ui:exp.replyVariable]], 이름을 바꾸지 않으면 `reply`)에 보관됩니다:
`text`, `hex`(처음 1024바이트), `bytes`(크기), `from`(보낸 쪽의 `IP:port`), `ms`(걸린 시간),
`match`(찾은 텍스트나 바이트, 또는 정규식의 첫 그룹).

대기는 단계에 도달했을 때가 아니라 실행이 시작될 때 수신을 시작하므로, 아주 빨리 오는 답도 놓치지
않습니다. 대기는 그 경로에서 마지막 전송 뒤에 도착한 것만 받습니다.

### 한도와 기본값 {#limits}

| 설정 | 기본값 | 범위 |
| --- | --- | --- |
| TCP 단계: [[ui:common.timeoutMs]] (연결, 쓰기, 답을 합쳐서) | 4000 ms | 1–120 000 ms |
| 대기 또는 회신의 [[ui:exp.waitTimeout]] | 2000 ms | 1–120 000 ms |
| UDP 페이로드 | — | 최대 65 507바이트 |
| 수신 주소 | — | 포트가 있는 `IP:port`; 회신의 [[ui:exp.replyOn]]은 포트 0(아무 빈 포트)을 쓸 수 있습니다 |

인스펙터에서 UDP 단계의 데이터그램은 `broadcast` 출처(단계가 회신을 기다리면 `experiment`)로
나타나고, 대기의 포트에 도착하는 모든 데이터그램은 `experiment-wait` 출처로 나타납니다. TCP 단계는
`experiment` 출처의 `tcp` 프레임 두 개로 나타납니다: 쓴 페이로드와, 왔을 때 읽은 답입니다.
타임라인 항목은 무엇을 보냈고 답이 얼마나 컸는지 말합니다.

## 신호 {#signals}

라이브러리의 [[ui:sig.tr.udp]] 신호는 대상과 페이로드이며, 텍스트나 hex 바이트입니다.
[[ui:nav.signals]]에서, 또는 아무 화면에서 <kbd>Ctrl</kbd>+<kbd>K</kbd>로 보냅니다. 그 대상은
호스트 이름일 수 있습니다.

인스펙터가 온전히 보관한 데이터그램이면 무엇이든 신호가 될 수 있습니다: [[ui:sig.fromFrame]]은
[[ui:sig.capturedFolder]] 폴더에 hex UDP 신호를 만들어 그 바이트를 정확히 재생합니다 — 보낸
프레임이면 그 프레임의 목적지로, 들어온 프레임이면 받은 주소로(이 컴퓨터에서 그 주소가 모든
주소였을 때). TCP 청크는 그럴 수 없습니다: 그것은 스트림의 한 조각입니다.
[신호](../tools/signals.md)와 [인스펙터](../tools/inspector.md#save-as-signal)를 참고하십시오.

텍스트 UDP 신호는 [[ui:exp.node.udp]] 단계로 실험에 추가할 수 있고, hex 신호는 그럴 수 없습니다.
단계가 텍스트를 보내기 때문입니다.

## 명령줄에서 {#cli}

`signallab send udp`는 데이터그램 하나를 보냅니다:

```bash
signallab send udp 127.0.0.1:9000 --text "PING"
signallab send udp 127.0.0.1:9000 --hex "de ad be ef"
```

```text
✔ sent 4 bytes → 127.0.0.1:9000
```

`--text`와 `--hex` 중 정확히 하나를 주십시오. 대상은 `IP:port` 또는 `host:port`([위](#destinations))입니다. 데이터그램이 나가면 0으로, 전송에 실패하면 1로, 대상이나 hex가
잘못되면 2로 종료합니다. `send tcp`는 없습니다.
[명령줄](../automation/cli.md#cli-send-udp)을 참고하십시오.

## Storm {#storm}

[[ui:nav.storm]]은 자신의 서버와 링크를 위한 부하 원천입니다: [[ui:st.udp]]는 정해진 크기의
데이터그램을 정해진 속도로 보내고, [[ui:st.tcp]]는 연결을 열고 페이로드를 쓰고 닫기를 거듭합니다.
처리량은 실시간으로 측정됩니다. [Storm](../tools/storm.md)을 참고하십시오.

## 스캐너 {#scanner}

[[ui:nav.scan]]은 범위의 각 포트에 TCP 연결을 시도하고, 받아 주는 포트를 나열하며, 배너를
요청하면 서비스가 처음 말하는 내용도 함께 보여 줍니다. [스캐너](../tools/scanner.md)를
참고하십시오.

::: danger
Storm과 스캐너는 실제 호스트로 실제 트래픽을 보냅니다. 소유했거나 테스트가 허용된 시스템에만
향하십시오: 폭주는 링크를 포화시킬 수 있고, 둘 다 침입 탐지를 건드릴 수 있습니다.
:::

## 에뮬레이션된 장치 {#emulators}

[[ui:nav.emulators]] 화면에서 Signal Lab이 장치가 될 수 있습니다:

- [[ui:emu.new.udp]]는 페이로드 규칙(임의, 텍스트 포함, 정규식 일치, 바이트 포함)으로
  데이터그램에 응답하며, 도착한 것으로 만든 텍스트나 hex 회신을 보낸 쪽이나 다른 `IP:port`로,
  지연을 설정했으면 지연 뒤에 보냅니다;
- [[ui:emu.new.tcp]]는 연결을 받고, 도착한 것을 고른 줄 끝(LF, CR LF, CR, 또는 매 청크)에서
  메시지로 나누어, 같은 종류의 규칙으로 각각 응답하고, 클라이언트가 연결할 때 인사말을 보낼 수
  있으며, 회신 뒤에 연결을 닫을 수 있습니다.

둘 다 실행이 지속되는 동안 [[ui:exp.node.emulator]] 단계로 실행할 수도 있습니다.
[에뮬레이터](../tools/emulators.md)를 참고하십시오.

## 장애 {#impairment}

[[ui:nav.netsim]] 릴레이는 클라이언트와 그 대상 사이에 앉아 지나가는 것을 열화합니다:
UDP에서는 데이터그램 단위로(지연, 손실, 중복, 재정렬, 대역폭 제한), TCP에서는 스트림 단위로(지연,
대역폭 제한, 연결 재설정 또는 반열림 유지). [장애](../tools/impairment.md)와, 실험 안에서는
[장애 주입](../experiments/faults.md)을 참고하십시오.

## Windows의 "포트 도달 불가" {#port-unreachable}

데이터그램이 아무것도 수신하지 않는 포트에 도달하면, 받는 컴퓨터는 보통 ICMP "포트 도달 불가"
메시지로 답합니다. Windows는 그 답을 보내는 소켓의 다음 수신에서, 마치 연결이 재설정된 것처럼
보고합니다 — UDP에 연결이 없는데도 말입니다.

Signal Lab은 이를 예상합니다. 수신자들 — OSC 모니터, 디스커버리 수신자, 실험의 대기와 회신, UDP와
OSC 에뮬레이터, 장애 릴레이 — 은 이를 기록하고 계속 수신합니다. 사라진 장치가 그것들을 멈추지
않습니다. 더는 정말로 받을 수 없는 수신자는 작업을 끝내고, 콘솔이 이유를 알려 줍니다.

## 문제 {#troubleshooting}

| 보이는 것 | 흔한 원인 |
| --- | --- |
| `… is not a valid address` | 대상에 포트가 없거나, `IP:port`도 `host:port`도 아닙니다. |
| `Cannot resolve …` | 이 컴퓨터에서 호스트 이름이 해석되지 않습니다. |
| `… refused the connection — nothing is listening on that port`(TCP) | 그 포트에서 아무것도 수신하지 않거나, 방화벽이 거부합니다. |
| `No answer from … in time`(TCP) | 호스트가 전혀 답하지 않습니다 — 주소가 틀렸거나, 거부 대신 버리는 방화벽입니다. |
| 장치는 답하는데 대기가 시간 초과됨 | 장치가 대기의 포트가 아니라 데이터그램이 온 포트로 답하기 때문입니다. [[ui:exp.expectReply]]로 전송 자체가 회신을 기다리게 하십시오: 그러면 답이 돌아오는 포트에서 나갑니다. |
| 다른 컴퓨터의 데이터그램이 도착하지 않음 | Windows에서는 방화벽이 막을 수 있습니다: 앱이 요청하면 Signal Lab을 허용하십시오. [문제 해결](../reference/troubleshooting.md)을 참고하십시오. |

모든 오류 메시지는 [오류 메시지](../reference/errors.md#transport)에 정리되어 있습니다.

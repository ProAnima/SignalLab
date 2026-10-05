---
title: 신호
description: OSC, UDP, HTTP, MQTT 메시지를 폴더 라이브러리에 보관하고, 신호 화면에서, 아무 화면에서 Ctrl+K로, 또는 명령줄에서 다시 보냅니다.
---

# 신호

신호는 이름을 붙여 보관해 둔 메시지입니다: OSC 메시지, 원시 UDP 데이터그램, HTTP 요청, MQTT
발행이며, 대상도 함께입니다. 한 번 만들어 두고 — [[ui:nav.signals]] 화면에서, 또는 다른 화면에서
방금 보낸 것을 저장해서 — 필요할 때마다 바이트 단위로 똑같이 다시 보냅니다.

라이브러리는 읽고, 손으로 고치고, 다른 컴퓨터로 복사하거나 프로젝트 옆에 커밋할 수 있는 JSON
파일입니다. [[ui:nav.signals]] 화면은 왼쪽에 폴더 트리로, 오른쪽에 선택한 신호의 필드로 보여
줍니다.

## 신호가 보낼 수 있는 것 {#transports}

[[ui:sig.transport]]에서 종류를 고릅니다. 종류마다 고유한 필드가 있습니다:

| [[ui:sig.transport]] | 필드 | 나가는 것 |
| --- | --- | --- |
| [[ui:sig.tr.osc]] | [[ui:common.target]], [[ui:common.address]], [[ui:common.arguments]] | `IP:port` 또는 `host:port` 하나로 가는 OSC 메시지 하나, 새 UDP 포트에서. 인수 타입은 [OSC](../protocols/osc.md#types)를 참고하십시오. |
| [[ui:sig.tr.udp]] | [[ui:common.target]], [[ui:sig.payloadKind]] ([[ui:sig.payloadText]] 또는 [[ui:sig.payloadHex]]), [[ui:sig.payload]] | 정확히 이 바이트들을 담은 데이터그램 하나. 텍스트는 종결자 없이 쓴 그대로 나가고, hex는 `de ad be ef`처럼 숫자 쌍입니다(공백과 `0x` 접두사가 허용됩니다). |
| [[ui:sig.tr.http]] | [[ui:sig.method]], [[ui:sig.url]], [[ui:sig.timeout]], [[ui:sig.headers]], [[ui:field.auth]], [[ui:sig.body]] | HTTP 요청 하나. [HTTP](../protocols/http.md)를 참고하십시오. |
| [[ui:sig.tr.mqtt]] | [[ui:mq.broker]], [[ui:mq.topic]], [[ui:mq.qos]], [[ui:mq.retain]], [[ui:sig.payload]] | 발행 하나. [MQTT](../protocols/mqtt.md)를 참고하십시오. |

모든 신호에는 [[ui:sig.name]], [[ui:sig.group]], [[ui:sig.note]]도 있습니다 — 무엇을 일으켜야
하는지, 그리고 상대편에서 무엇이 일치해야 하는지입니다.

OSC와 UDP 신호의 대상은 `IP:port` 또는 `host:port`입니다(예: `127.0.0.1:9000`). 호스트 이름은
신호를 보낼 때마다 조회됩니다. MQTT 신호의 브로커는 `host:port`이며, 포트가 없으면 `1883`입니다.

신호의 종류를 바꾸면 메시지는 그 종류의 기본값에서 다시 시작합니다. 유지되는 것은 대상뿐이며,
그것도 [[ui:sig.tr.osc]]와 [[ui:sig.tr.udp]] 사이에서만입니다 — 거기서는 대상이 같은 뜻이기
때문입니다.

## 신호 보내기 {#send}

[[ui:nav.signals]] 화면에서 신호를 보내려면 다음 중 하나를 하십시오:

- 신호를 선택하고 [[ui:sig.fire]]를 누릅니다.
- 그 신호의 필드에서 작업하는 중에 <kbd>Ctrl</kbd>+<kbd>Enter</kbd>를 누릅니다.
- 트리에서 신호를 두 번 클릭합니다.

보낼 때마다 콘솔에 한 줄이 기록됩니다: 무엇이 어디로 갔는지, 보낸 바이트, HTTP라면 상태와 걸린
시간입니다. 실패(연결 거부, 도달할 수 없는 호스트)는 이유와 함께 빨간 줄로 표시됩니다. 마지막으로
보낸 시각은 버튼 옆에 표시됩니다.

신호는 그 프로토콜의 화면과 같은 명령을 통해 나가므로, [인스펙터](inspector.md)는 그것을 보낸
도구 아래에 나열하며, 상대편은 직접 입력한 것과 구별할 수 없습니다.

종류마다 나가는 방식:

| 종류 | 나가는 방식 |
| --- | --- |
| [[ui:sig.tr.osc]] | [[ui:nav.osc]] 화면이 메시지를 보내는 방식과 같습니다. |
| [[ui:sig.tr.udp]] | 대상으로 데이터그램 하나를 보냅니다. |
| [[ui:sig.tr.http]] | [[ui:http.keepCookies]]가 켜져 있는 동안 [[ui:nav.http]] 화면이 요청을 보내는 방식과 같습니다. 연결 거부나 시간 초과는 상태가 아니라 실패로 셉니다. |
| [[ui:sig.tr.mqtt]] | [[ui:nav.mqtt]] 화면이 신호의 브로커에 연결되어 있으면 그 연결로, 그 클라이언트 id와 자격 증명으로 보냅니다. 그렇지 않으면 — 연결되어 있지 않거나 다른 브로커에 연결되어 있으면 — Signal Lab이 이번 발행 하나를 위해 신호의 브로커에 자체 클라이언트 id, 사용자 이름 없음, clean session으로 연결한 뒤 끊습니다. |

호스트가 대소문자 구분 없이 같고 포트도 같을 때 그 연결은 신호의 브로커에 대한 것입니다. 포트 없이
쓴 브로커는 `1883`으로 봅니다. 이름은 조회하지 않습니다: 여기서 `localhost`와 `127.0.0.1`은 서로
다른 두 브로커이므로, 하나를 가리키는 신호가 다른 하나로 만든 연결로 보내지지 않습니다.

::: tip
MQTT 신호는 비밀번호를 저장하지 않습니다. 비밀번호를 요구하는 브로커에 발행하려면 먼저
[[ui:nav.mqtt]] 화면에서 그 브로커에 연결하십시오. 그러면 신호가 그 연결을 타고 갑니다.
:::

## 아무 화면에서 보내기 {#palette}

아무 화면에서나 <kbd>Ctrl</kbd>+<kbd>K</kbd>를 눌러 팔레트를 열고, 신호의 이름, 폴더, 대상,
메시지의 몇 글자를 입력한 뒤 <kbd>Enter</kbd>를 누릅니다. 팔레트가 닫히고 신호가 전송되며, 보고
있던 화면에 그대로 남습니다.

| 키 | 하는 일 |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>K</kbd> | 팔레트를 열거나 닫습니다. |
| <kbd>↑</kbd> <kbd>↓</kbd> | 선택을 옮깁니다. |
| <kbd>Enter</kbd> | 선택한 신호를 보냅니다. |
| <kbd>Esc</kbd> | 보내지 않고 팔레트를 닫습니다. |

팔레트는 신호를 최대 12개 나열합니다: 아무것도 입력하지 않았을 때는 라이브러리의 처음 12개,
그다음에는 일치하는 처음 12개입니다. 행을 클릭하면 보내지고, 바깥을 클릭하면 팔레트가 닫힙니다.

## 신호 만들기 {#create}

### 신호 화면에서 {#create-here}

1. 신호가 속할 폴더를 고릅니다([현재 폴더](#current-folder) 참고).
2. [[ui:sig.new]]를 누릅니다. `127.0.0.1:9000`으로 가는 주소 `/hello`의 새 OSC 신호가 그 폴더에
   선택된 채로 나타납니다.
3. [[ui:sig.name]], [[ui:sig.transport]]와 메시지의 필드를 바꿉니다.

변경 사항은 저절로 저장됩니다. 이 화면에는 저장 버튼이 없습니다. 라이브러리 파일을 읽을 수 없는
동안에는 아무것도 저장되지 않고 필드는 읽기 전용입니다([라이브러리 파일](#file) 참고).

[[ui:sig.duplicate]]는 선택한 신호 바로 뒤에 사본을 만들고, 그 이름 뒤에 `·`를 붙입니다.
[[ui:sig.delete]]는 한 번 더 묻고([[ui:sig.confirmDelete]]): 두 번째 클릭이 파일에서 제거합니다.
빈 폴더가 되었더라도 그 폴더는 남습니다.

### HTTP, OSC, MQTT 화면에서 {#save-from-screens}

세 화면의 송신 영역 — [[ui:nav.http]]의 [[ui:http.request]], [[ui:nav.osc]]의 [[ui:osc.sender]],
[[ui:nav.mqtt]]의 [[ui:mq.publish]] — 은 보낼 내용을 신호로 보관할 수 있습니다.

1. 메시지를 설정하고 원하는 대로 동작할 때까지 보냅니다.
2. [[ui:sig.saveNew]]를 누릅니다(또는 화면의 그 영역에서 <kbd>Ctrl</kbd>+<kbd>S</kbd>).
   [[ui:sig.saveTitle]] 대화 상자가 열립니다.
3. [[ui:sig.name]]을 확인합니다: 보내는 내용에서 제안됩니다.
4. [[ui:sig.group]]을 고르거나 입력합니다. 지난번에 쓴 폴더가 채워지며, 아직 없는 `Venue/Stage` 같은
   경로는 만들어집니다.
5. [[ui:sig.saveConfirm]]를 누릅니다.

그때부터 화면은 그 신호와 연결됩니다. 버튼 옆의 칩이 신호가 사는 곳(`❖ Folder / Name`)을 알려
줍니다. 클릭하면 [[ui:nav.signals]] 화면에서 신호를 봅니다.

| 보이는 것 | 뜻 | 할 수 있는 것 |
| --- | --- | --- |
| ✓ [[ui:sig.savedState]] (회색) | 라이브러리에 화면이 보낼 내용이 정확히 들어 있습니다. | 저장할 것이 없습니다. |
| [[ui:sig.save]], 그리고 칩에 [[ui:sig.changed]] | 화면의 메시지가 신호와 다릅니다. | [[ui:sig.save]] 또는 <kbd>Ctrl</kbd>+<kbd>S</kbd>가 화면의 메시지를 그 신호에 씁니다. 이름, 폴더, 메모는 그대로입니다. |
| [[ui:sig.saveAs]] | — | 대화 상자를 신호의 이름과 폴더로 채워 다시 열고, 새 신호를 저장합니다. 그러면 화면이 새 신호와 연결됩니다. |

비교는 메시지를 보는 것이지, 쓰인 방식을 보는 것이 아닙니다: JSON 키의 순서와 32비트 정밀도를
넘는 OSC float의 끝자리는 변경으로 치지 않습니다.

[[ui:nav.http]]와 [[ui:nav.osc]] 화면은 Signal Lab이 다시 시작해도 연결을 유지합니다.
[[ui:nav.mqtt]] 화면은 앱을 닫을 때까지 유지합니다.

::: warning
HTTP 신호는 [[ui:field.auth]] — 사용자 이름과 비밀번호, 또는 토큰 — 를 라이브러리 파일에
평문으로 보관합니다. 파일을 읽을 수 있는 사람은 그것을 읽을 수 있습니다.
:::

### 신호를 해당 화면에서 열기 {#open-in-screen}

선택한 HTTP, OSC, MQTT 신호에는 그 프로토콜의 화면([[ui:nav.http]], [[ui:nav.osc]],
[[ui:nav.mqtt]])에서 여는 버튼이 있습니다. 화면의 필드가 신호로 채워지고 화면이 위에서처럼 그
신호와 연결됩니다: 거기서 편집하고, 보내고, [[ui:sig.save]]를 누르십시오. 원시 UDP 신호에는 자체
화면이 없습니다.

### 프레임이나 토픽에서 {#capture}

- [인스펙터](inspector.md#save-as-signal)에서 프레임을 선택하고 [[ui:sig.fromFrame]]를 누릅니다.
  데이터그램은 그 프레임의 정확한 바이트를 담은 원시 UDP 신호가 되고, MQTT 발행은 같은 브로커,
  토픽, QoS, retain 플래그, 페이로드를 담은 MQTT 신호가 됩니다. 다른 프레임은 저장할 수 없습니다.
- [[ui:nav.mqtt]] 화면에서 토픽을 선택하고 [[ui:sig.fromFrame]]를 누릅니다. 토픽의 최신 값을 그
  QoS와 retain 플래그와 함께 연결된 브로커로 발행하는 MQTT 신호를 얻습니다.

둘 다 [[ui:sig.capturedFolder]] 폴더에 들어가며, 잡힌 것의 이름을 따서 이름이 붙습니다.

### 실험에서 {#in-experiments}

실험에 노드를 추가할 때 [[ui:exp.addNode]] 메뉴는 [[ui:exp.group.signals]] 아래에 신호도
나열합니다. 하나를 고르면 같은 메시지를 담은 OSC, HTTP, MQTT 또는 UDP 노드가 추가됩니다. hex
페이로드를 담은 원시 UDP 신호는 나오지 않습니다: UDP 노드는 텍스트를 보내기 때문입니다.
[노드](../experiments/nodes.md)를 참고하십시오.

## 폴더 {#folders}

폴더는 `/`로 이어 붙인 이름들의 경로입니다: `API/Auth`는 `API` 안의 `Auth` 폴더입니다. 신호의
[[ui:sig.group]] 필드는 그 폴더의 경로를 담으며, 비어 있으면 최상위([[ui:sig.topLevel]])를
뜻합니다. 필드를 벗어날 때 이름은 다듬어지고 빈 부분은 버려지므로, ` API / Auth/ `는 `API/Auth`가
됩니다.

폴더는 이름순으로, 숫자는 숫자 순서로 정렬됩니다(`Cue 2`가 `Cue 10` 앞). 신호는 파일의 순서를
유지합니다. 각 폴더는 하위 폴더를 포함해 담고 있는 신호 수를 보여 줍니다. 빈 폴더는 제거할 때까지
유지됩니다.

### 현재 폴더 {#current-folder}

마지막으로 클릭한 폴더, 또는 선택한 신호의 폴더가 현재 폴더입니다: [[ui:sig.new]]와
[[ui:sig.newFolder]]가 거기에 만듭니다. 트리 위의 칩이 그 이름을 알려 주며, 칩을 클릭하면
최상위로 돌아갑니다.

### 폴더 다루기 {#folder-tasks}

| 하려면 | 이렇게 |
| --- | --- |
| 폴더 만들기 | ＋ [[ui:sig.newFolder]]를 누릅니다. 현재 폴더 안에 만들어지고, 이름은 [[ui:sig.newFolderName]]이며(그 이름이 이미 쓰이면 뒤에 숫자가 붙습니다), 곧바로 이름을 바꿉니다. |
| 폴더 열거나 닫기 | 클릭하거나, 포커스가 있을 때 <kbd>→</kbd> / <kbd>←</kbd>를 누릅니다. Signal Lab은 어떤 폴더가 접혀 있는지 기억합니다. |
| 모두 열거나 닫기 | 트리 위의 ⊞와 ⊟ 버튼([[ui:sig.expandAll]], [[ui:sig.collapseAll]]). |
| 폴더 이름 바꾸기 | ✎([[ui:sig.renameFolder]])를 누르거나 폴더에서 <kbd>F2</kbd>를 눌러 입력한 뒤 <kbd>Enter</kbd>를 누릅니다. <kbd>Esc</kbd>는 취소합니다. |
| 신호나 폴더 옮기기 | 폴더 위로, 또는 최상위로 보내려면 트리의 빈 공간으로 끌어다 놓습니다. |
| 입력해서 신호 옮기기 | [[ui:sig.group]] 필드를 바꿉니다. |
| 폴더 제거하기 | ×([[ui:sig.removeFolder]])를 누른 뒤 [[ui:sig.confirmRemoveFolder]]를 누르거나, 폴더에서 <kbd>Delete</kbd>를 두 번 누릅니다. |

이름 바꾸기는 두 폴더를 합치지 않습니다: `/`가 들어간 이름이나 형제 폴더가 이미 가진 이름은
거부되고, 콘솔이 그렇게 알려 줍니다. 폴더는 자기 자신이나 자기 안의 폴더로 끌 수 없습니다. 이미
같은 이름을 가진 폴더로 폴더를 끌면 둘이 합쳐집니다.

폴더를 제거하면 폴더만 제거됩니다: 그 신호와 하위 폴더는 한 단계 위로 올라갑니다. 아무것도
삭제되지 않습니다.

### 신호 찾기 {#filter}

트리 위의 [[ui:sig.search]]에 입력합니다. 이름, 폴더, 메모, 대상, 메시지를 일치시킵니다.
필터링하는 동안 일치하는 것이 있는 폴더는 모두 열리고 나머지는 숨겨집니다.

## 기본 제공 세트 {#starter-set}

Signal Lab이 라이브러리 파일을 처음 찾지 못하면, 각각 틀리기 쉬운 무언가에 관한 예시 아홉 개를
씁니다. 이름과 메모는 그 시점의 인터페이스 언어로 작성됩니다. 그 뒤로는 원하는 대로 바꿀 수
있습니다. 모두 이 컴퓨터를 가리킵니다.

| 폴더 | 신호 | 보내는 것 |
| --- | --- | --- |
| `OSC` | [[ui:seed.osc-fader.name]] | float `0.75`와 함께 `/fader/1`을 `127.0.0.1:9000`으로 |
| `OSC` | [[ui:seed.osc-types.name]] | int `-7`, float `1.5`, string `hi`, bool true, int64 `4294967296`, double `0.125`, nil과 함께 `/types`를 |
| `OSC` | [[ui:seed.osc-id-and-value.name]] | 문자열 `reader-1`과 `04a1b2c3`와 함께 `/tag`를 |
| `OSC` | [[ui:seed.osc-trigger.name]] | 인수 없이 `/cue/go` |
| `MQTT` | [[ui:seed.mqtt-publish.name]] | `1`을 `127.0.0.1:1883`의 `lab/example/value`로, QoS 0 |
| `MQTT` | [[ui:seed.mqtt-retained.name]] | `night`을 `lab/example/config`로, QoS 1, retained |
| `MQTT` | [[ui:seed.mqtt-clear-retained.name]] | 빈 retained 페이로드를 `lab/example/config`로, QoS 1 |
| `HTTP` | [[ui:seed.http-reachable.name]] | `GET http://127.0.0.1:8080/`, 타임아웃 4000 ms |
| [[ui:seed.folder.Raw]] | [[ui:seed.udp-raw.name]] | 바이트 `de ad be ef`를 `127.0.0.1:9000`으로 |

기본 제공 세트를 되돌리려면 `signals.json`을 옮기거나 이름을 바꾸고 [[ui:sig.reload]]를
누르십시오: 파일이 없으면 다시 작성됩니다.

## 라이브러리 파일 {#file}

라이브러리는 데이터 폴더의 `signals.json`입니다: 데스크톱에서는 홈 폴더의
`Documents/SignalLab`, 또는 서버의 데이터 폴더입니다([파일](../reference/files.md) 참고). 트리
아래의 신호 개수에 포인터를 올리면 전체 경로가 보입니다.

- **저절로 저장됩니다.** 변경 사항은 마지막 변경 0.7초 뒤에, 같은 폴더의 임시 파일을 거쳐 통째로
  한 번에 쓰이며, 임시 파일이 그 파일의 자리를 대신합니다 — 쓰기가 중간에 끊기면 이전 파일이
  남습니다. 쓰기가 기다리는 동안 트리 아래에 [[ui:sig.saving]]이, 그다음 [[ui:sig.saved]]가
  표시됩니다. 아직 기다리는 쓰기는 업데이트가 앱을 다시 시작하기 전에 이루어집니다.
- **손으로 편집할 때.** Signal Lab은 실행 중에 파일이 바뀌는 것을 알아차리지 못합니다. 파일을
  편집한 뒤, 또는 다른 컴퓨터의 파일로 바꾼 뒤 [[ui:sig.reload]]를 누르십시오. 다시 읽으면 파일을
  새로 읽고, 아직 쓰기를 기다리던 변경은 버립니다.
- **망가진 동안에는 교체되지 않습니다.** 파일이 유효한 JSON이 아니거나 신호 라이브러리가 아니면,
  트리에 파일의 경로, 줄, 열과 함께 오류가 표시되고 콘솔도 같은 말을 합니다. 파일은 그대로 두고,
  다시 읽을 때까지 라이브러리에 아무것도 쓰지 않습니다: [[ui:sig.new]], 신호와 폴더의 이름
  바꾸기·옮기기·제거, 끌어다 놓기, 신호의 필드, HTTP·OSC·MQTT 화면의 [[ui:sig.saveNew]],
  [[ui:sig.save]], [[ui:sig.saveAs]], 그리고 인스펙터와 MQTT 화면의 [[ui:sig.fromFrame]]이 모두
  꺼지고, 그 툴팁이 무엇이 잘못되었는지 말합니다. 파일을 고치거나 제거하고 [[ui:sig.reload]]를
  누르십시오: 한 번 읽히면 모든 것이 다시 동작합니다.
- **앱이 실행 중일 때 망가지는 파일.** 파일을 읽을 수 없게 편집한 뒤 앱이 변경을 저장하려 하면,
  저장은 같은 오류로 거부되고, 파일은 만든 그대로 두며, 앱은 고치고 [[ui:sig.reload]]를 누를
  때까지 쓰기를 멈춥니다. 편집했지만 유효하게 둔 파일은 위에서처럼 앱의 다음 저장 때 앱의 목록으로
  교체됩니다: 먼저 다시 읽으십시오.

파일의 짧은 예:

```json
{
  "version": 2,
  "signals": [
    {
      "id": "fader-value",
      "name": "Fader value",
      "group": "Venue/Stage",
      "note": "Main fader of desk A.",
      "body": {
        "transport": "osc",
        "target": "127.0.0.1:9000",
        "address": "/fader/1",
        "args": [{ "type": "float", "value": 0.75 }]
      }
    }
  ],
  "folders": ["Venue/Stage", "Venue/Empty for now"]
}
```

| 키 | 내용 |
| --- | --- |
| `version` | `2`. 버전 1 파일(폴더 이전)은 빈 폴더 없이 똑같이 읽힙니다. |
| `signals[].id` | 신호를 만들 때 이름에서 만들어지며(`fader-value`, `fader-value-2`, …), 이름을 바꿔도 바뀌지 않습니다. `signallab fire`는 이 id로 신호를 찾습니다. |
| `signals[].group` | 폴더 경로이며, `""`는 최상위입니다. |
| `signals[].body` | 메시지입니다. `transport`는 `osc`, `udp`, `http`, `mqtt` 중 하나이고, 나머지 키는 그 종류의 필드입니다. |
| `folders` | 모든 폴더이며, 그래서 빈 폴더도 유지됩니다. 없으면 빠집니다. 어떤 항목도 나열하지 않는 `group`도 폴더입니다. |

## 명령줄에서 {#cli}

`signallab fire`는 앱과 같은 명령을 통해 라이브러리의 신호를 보냅니다:

```bash
signallab fire "Fader value"
signallab fire fader-value --library ./show/signals.json
```

id로 먼저 찾고, 그다음 이름으로 대소문자 구분 없이 찾습니다. 그 이름을 가진 신호가 여러 개면 그
id들을 알려 주고 아무것도 보내지 않습니다. `--library`가 없으면 앱 자체의 `signals.json`을
읽습니다. 파일은 절대 쓰지 않습니다. [명령줄](../automation/cli.md#cli-fire)을 참고하십시오.

## 관련 항목 {#related}

- [인스펙터](inspector.md) — 신호가 보내는 것을 지켜보고, 잡은 프레임을 신호로 보관합니다.
- [키보드 단축키](../reference/shortcuts.md)
- [파일](../reference/files.md) — 데이터 폴더의 위치.

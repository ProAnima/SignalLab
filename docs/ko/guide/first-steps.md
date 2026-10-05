---
title: 첫 단계
description: 컴퓨터 한 대에서 진행하는 첫 세션입니다. OSC 메시지를 보내고 도착을 확인하고, 신호로 저장하고, 에뮬레이션된 API에 요청하고, 작은 실험을 만들어 실행합니다.
---

# 첫 단계

이 세션에는 Signal Lab만 있으면 됩니다. 모든 트래픽이 이 컴퓨터인 `127.0.0.1`로 가므로 장치,
네트워크, 방화벽 규칙이 관여하지 않습니다. 다음을 진행합니다.

1. OSC 메시지를 보내고 도착하는 것을 확인합니다.
2. 같은 메시지를 인스펙터에서 봅니다.
3. 메시지를 라이브러리에 저장하고 어디서든 다시 보냅니다.
4. 에뮬레이션된 HTTP API를 시작하고 요청을 보냅니다.
5. 그 API를 대상으로 실험을 실행하고, 실패한 이유를 읽고, 고치고, 검사를 추가합니다.

아직 Signal Lab을 설치하지 않았다면 [설치와 업데이트](install.md)를 참고하십시오.
창에서 무엇이 어디에 있는지 모르겠다면 [창](interface.md)을 참고하십시오.

## OSC 메시지를 보내고 도착 확인하기 {#osc}

먼저 메시지를 받을 것이 필요합니다. OSC 화면의 모니터를 사용합니다.

1. 사이드바에서 [[ui:nav.osc]] 화면을 엽니다.
2. [[ui:osc.monitor]] 아래에서 [[ui:common.bind]] 필드를 `127.0.0.1:9000`으로 설정해, 모니터가
   이 컴퓨터에서만 수신하도록 합니다.
3. [[ui:osc.listen]] 버튼을 누릅니다. 버튼이 [[ui:common.stop]] 버튼으로 바뀌고, 콘솔에 모니터가
   수신 중이라고 표시되며, 하단 패널의 표시줄에 모니터가 작업으로 나타납니다.

이제 옆에 있는 송신 영역에서 메시지를 보냅니다.

4. [[ui:osc.sender]] 아래의 [[ui:common.target]] 필드는 모니터가 수신하는 포트인
   `127.0.0.1:9000` 그대로 둡니다.
5. [[ui:common.address]] 필드는 `/hello/avatar/1`, [[ui:common.arguments]] 아래의 float 인수
   하나는 `1.0` 그대로 둡니다. 원하는 주소와 값을 직접 입력해도 됩니다.
6. [[ui:common.send]] 버튼을 누르거나, 대상 필드나 주소 필드에서 <kbd>Enter</kbd>를 누릅니다.

모니터 표에 한 줄이 나타납니다. 도착한 [[ui:common.time]], [[ui:osc.from]] (`127.0.0.1`과
메시지를 보낸 포트), [[ui:osc.address]], [[ui:osc.args]]입니다. 송신 영역 아래에는 무엇을 보냈는지와
그 크기(바이트)를 확인하는 줄이 표시되며, 다시 보내면 반복 횟수를 셉니다.

::: tip 방화벽 알림이 나타났습니까?
Windows에서는 모니터를 시작하면 헤더 아래에 Windows 방화벽에 관한 알림이 나타날 수 있습니다.
이 알림은 *다른* 컴퓨터에서 오는 메시지에 관한 것이며, `127.0.0.1`의 트래픽은 절대 필터링되지
않습니다. 지금은 [[ui:fw.dismiss]] 버튼을 누르십시오. 언제 허용해야 하는지는
[방화벽 알림](interface.md#firewall-notice)에서 설명합니다.
:::

## 인스펙터에서 보기 {#inspector}

인스펙터는 모든 도구가 보내고 받는 모든 프레임을 기록합니다. 단, 캡처가 켜져 있을 때만
기록합니다.

1. 하단 패널에서 [[ui:dock.inspector]] 탭을 엽니다.
2. [[ui:ins.arm]] 버튼을 누릅니다. 탭의 점에 불이 들어옵니다.
3. 송신 영역으로 돌아가 [[ui:common.send]] 버튼을 한 번 더 누릅니다.

최신 항목이 위에 오도록 두 행이 나타납니다. 보낸 메시지(→)와 모니터가 받은 메시지(←)이며, 각각
프로토콜, 상대편 주소, 크기, 요약이 표시됩니다. 하나를 클릭하면 [[ui:ins.detail]]에 어느 도구가
어느 주소에서 보내거나 받았는지, 메시지의 [[ui:ins.decoded]], 메시지를 이루는
[[ui:ins.rawBytes]]가 표시됩니다.

다 보았으면 [[ui:ins.disarm]] 버튼을 누르십시오. 캡처가 꺼져 있는 동안에는 아무 비용도 들지
않습니다. 자세한 내용은 [인스펙터](../tools/inspector.md)를 참고하십시오.

## 신호로 저장하고 다시 보내기 {#signal}

다시 쓸 메시지는 신호 라이브러리에 넣어 두십시오.

1. OSC 화면에서 송신 영역 아래의 [[ui:sig.saveNew]] 버튼을 누릅니다.
2. [[ui:sig.saveTitle]] 창에서 [[ui:sig.name]] 필드에 `First message`를, [[ui:sig.group]]
   필드에 `Tutorial`을 입력합니다. 새 폴더는 그 폴더에 저장할 때 만들어집니다.
3. [[ui:sig.saveConfirm]] 버튼을 누릅니다.

이제 송신 영역은 그 신호와 연결되어 있습니다. 버튼에 [[ui:sig.savedState]] 상태가 표시되고, 옆의
칩에 신호가 저장된 위치가 표시됩니다. 인수를 바꾸면 칩에 변경 사항이 표시되며,
[[ui:sig.save]] 버튼(<kbd>Ctrl</kbd>+<kbd>S</kbd>)을 누르면 신호가 업데이트됩니다.

이제 세 가지 방법으로 다시 보내 봅니다.

- **라이브러리에서.** 칩을 클릭하면 `Tutorial` 폴더에서 신호가 선택된 상태로 [[ui:nav.signals]]
  화면이 열립니다(또는 [[ui:nav.signals]] 화면을 열고 거기서 신호를 클릭합니다).
  [[ui:sig.fire]] 버튼이나 <kbd>Ctrl</kbd>+<kbd>Enter</kbd>를 누르십시오. 목록에서 신호를 두 번
  클릭해도 보내집니다.
- **어디서든.** 아무 화면에서나 <kbd>Ctrl</kbd>+<kbd>K</kbd>를 누르고 `first`를 입력한 다음
  <kbd>Enter</kbd>를 누릅니다.
- **실험에서.** 노드를 추가할 때 메뉴의 [[ui:exp.group.signals]] 아래에 신호 목록이 나오며,
  고르면 그 신호를 보내는 단계가 됩니다.

매번 모니터에 메시지가 도착하는 것이 보이고, 콘솔에 신호 이름이 표시됩니다. 신호는 원래 화면에서
보냈을 내용을 정확히 그대로 보냅니다. 자세한 내용은 [신호](../tools/signals.md)를 참고하십시오.

OSC 작업이 끝나면 모니터의 [[ui:common.stop]] 버튼을 누르십시오.

## 에뮬레이션된 API에 요청하기 {#emulator}

Signal Lab에는 모두 `127.0.0.1`에서 동작하는 에뮬레이터 다섯 개가 기본으로 들어 있습니다. 그중
[[ui:seed.emu.demo-api.name]] 에뮬레이터는 `127.0.0.1:8080`의 HTTP API이며 다음 라우트를
제공합니다.

| 요청 | 응답 |
| --- | --- |
| `GET /health` | `200`과 `{"status":"ok","time":"…"}` — 현재 시각 |
| `GET /users/:id` | `200`과 해당 id의 사용자, 예: `{"id":"42","name":"User 42"}` |
| `POST /users` | `201`과 `Location` 헤더, 새 id |
| `GET /slow` | 1.5초 뒤 `200` |
| 모든 메서드, `/flaky` | `503`, `503`, 세 번째 요청부터 `200` |
| 그 밖의 모든 요청 | `404` |

1. [[ui:nav.emulators]] 화면을 엽니다. [[ui:emu.library]]에 다섯 개가 나열되어 있으며, 그중
   [[ui:seed.emu.demo-api.name]] 에뮬레이터를 선택합니다.
2. [[ui:emu.start]] 버튼을 누릅니다. 이제 `127.0.0.1:8080`에서 응답하며 작업으로 실행됩니다.
3. [[ui:nav.http]] 화면을 엽니다. 메서드는 `GET`입니다. URL을
   `http://127.0.0.1:8080/health`로 설정합니다.
4. [[ui:common.send]] 버튼을 누르거나 URL 필드에서 <kbd>Enter</kbd>를 누릅니다.

[[ui:http.response]] 아래에 [[ui:http.status]] `200`, [[ui:http.latency]], [[ui:http.size]],
응답 헤더, JSON 본문이 표시됩니다. `http://127.0.0.1:8080/flaky`로 세 번 보내 보십시오. 두 번은
`503`, 그다음은 `200`이 돌아옵니다. 다시 시도하는 클라이언트에게 복구되는 서비스가 이렇게 보입니다.

[[ui:nav.emulators]] 화면으로 돌아가면 [[ui:emu.live]] 패널이 모든 요청을 세고 있으며,
[[ui:emu.received]] 목록에는 각 요청이 그 요청에 응답한 [[ui:emu.col.rule]], [[ui:emu.col.reply]]
열과 함께 나열됩니다. 다음 부분을 위해 [[ui:seed.emu.demo-api.name]] 에뮬레이터를 실행 중인 채로
두십시오. 자세한 내용은 [에뮬레이터](../tools/emulators.md)를 참고하십시오.

## 실험 실행하기 {#experiment}

실험은 몇 번이고 다시 실행할 수 있는 단계의 흐름입니다. Signal Lab을 처음 열 때 나오는 실험인
[[ui:exp.templateHttp]] 템플릿은 `http://127.0.0.1:8080/`으로 요청을 보내고 응답이 `200`인지
검사합니다.

### 템플릿 열기 {#open-template}

1. [[ui:nav.experiment]] 화면을 엽니다.
2. 캔버스에 [[ui:exp.node.start]], [[ui:exp.node.http]], [[ui:exp.node.assert_status]],
   [[ui:exp.node.end]] 네 노드가 보이지 않으면 도구 모음 왼쪽의 **☰**([[ui:exp.documents]])
   버튼을 누르고, 템플릿 목록에서 [[ui:exp.templateHttp]] 항목을 고른 다음
   [[ui:exp.openDocument]] 버튼을 누릅니다. 열면 캔버스의 실험이 바뀌며,
   <kbd>Ctrl</kbd>+<kbd>Z</kbd>를 누르면 이전 실험이 돌아옵니다.

노드를 클릭하면 오른쪽의 [[ui:exp.properties]] 영역에 설정이 표시됩니다. 실험은 편집하는 대로
자동으로 저장됩니다.

### 실행하고 실패한 이유 읽기 {#first-run}

3. [[ui:exp.run]] 버튼을 누릅니다.

캔버스 아래에 [[ui:exp.timeline]] 영역이 열리고, 단계마다 시작할 때([[ui:exp.running]])와 끝날 때
한 행씩 시각, 노드, 결과가 표시됩니다. 이 실행은 실패합니다.

- [[ui:exp.node.start]] 단계는 통과하고 실행의 시드를 표시합니다.
- [[ui:exp.node.http]] 단계는 통과합니다. 요청이 나갔고 응답 `HTTP 404`가 돌아왔습니다.
- [[ui:exp.node.assert_status]] 단계는 실패합니다. `200`을 기대했지만 `404`를 받았습니다.

[[ui:seed.emu.demo-api.name]] 에뮬레이터에는 `/` 라우트가 없어서 `404`로 응답했고, 검사가 이를
잡아냈습니다. 타임라인 맨 위 줄에 [[ui:exp.failed]] 상태와 그 이유가 표시됩니다. 행을 클릭하면
캔버스에서 해당 노드가 선택됩니다.

::: tip 요청 자체가 실패했습니까?
[[ui:exp.node.http]] 단계가 연결 거부로 실패하면 `127.0.0.1:8080`에서 아무것도 수신하고 있지 않은
것입니다. [[ui:nav.emulators]] 화면에서 [[ui:seed.emu.demo-api.name]] 에뮬레이터를 시작하고 다시
실행하십시오.
:::

### 요청 고치기 {#fix}

4. [[ui:exp.node.http]] 노드를 클릭합니다.
5. [[ui:exp.properties]] 영역에서 [[ui:sig.url]] 필드를 `http://127.0.0.1:8080/health`로 바꿉니다.
6. [[ui:exp.run]] 버튼을 누릅니다.

이번에는 모든 단계가 통과합니다. [[ui:exp.node.assert_status]] 단계에는 [[ui:exp.step.checked]],
[[ui:exp.node.end]] 단계에는 [[ui:exp.step.complete]] 표시가 나오고, 타임라인 제목에는
[[ui:exp.passed]] 상태가 표시됩니다.

### 검사 추가하기 {#add-check}

상태 `200`은 서비스가 응답했다는 뜻일 뿐, 무엇을 응답했는지는 알려 주지 않습니다. 본문도
검사하십시오.

7. [[ui:exp.node.assert_status]] 노드를 클릭합니다.
8. [[ui:exp.properties]] 영역에서 [[ui:exp.addNext]] 버튼을 누릅니다. 또는 캔버스에 포커스가 있을
   때 <kbd>A</kbd>를 누릅니다. 검색 필드가 있는 노드 메뉴가 열립니다.
9. `assert_body`를 입력하고 <kbd>Enter</kbd>를 누릅니다.
   [[ui:exp.node.assert_body]] 노드가 [[ui:exp.node.assert_status]]와 [[ui:exp.node.end]]
   사이에 이미 연결된 상태로 추가되며, [[ui:exp.contains]] 필드에 바로 입력할 수 있습니다.
10. `"status":"ok"`를 입력합니다.
11. [[ui:exp.run]] 버튼을 누릅니다.

새 단계가 통과합니다. 텍스트를 본문에 없는 내용으로 바꾸고 다시 실행하면 이유와 함께 실패하는
것을 볼 수 있습니다.

### 실행이 남기는 것 {#report}

- **보고서.** 실행이 끝나면 타임라인 제목에 [[ui:exp.reportSaved]] 표시가 나타나며, 포인터를
  올리면 파일이 보입니다. 통과했든 실패했든 끝난 실행은 데이터 폴더의 `runs` 폴더에 보고서를
  하나 쓰며, 여기에는 사용한 값과 모든 단계가 담깁니다. 브라우저에서는 다운로드 링크입니다.
- **시드.** 제목에는 [[ui:exp.pinSeed]] 버튼과 함께 실행의 시드도 표시됩니다. 실행 안의 임의
  값은 시드를 따르며, 시드를 고정하면 같은 값이 정확히 반복됩니다.

자세한 내용은 [실행과 보고서](../experiments/runs.md)를 참고하십시오.

## 정리하기 {#clean-up}

헤더의 [[ui:app.stopAll]] 버튼을 누르십시오. [[ui:seed.emu.demo-api.name]] 에뮬레이터와 아직
실행 중인 다른 모든 것이 중지됩니다. 신호, 실험, 실험의 보고서는 데이터 폴더에 남아 있습니다.

## 다음으로 {#next}

- [개념](concepts.md): 화면, 신호, 작업, 에뮬레이터, 실험의 바탕이 되는 개념.
- [실험](../experiments/index.md): 편집기의 모든 것, 그리고 [노드](../experiments/nodes.md)에
  정리된 모든 종류의 노드.
- [OSC](../protocols/osc.md), [HTTP](../protocols/http.md) 및 다른 프로토콜 페이지: Signal Lab을
  실제 장비에 연결할 때.
- [명령줄](../automation/cli.md): 같은 실험을 터미널이나 파이프라인에서 실행하기.

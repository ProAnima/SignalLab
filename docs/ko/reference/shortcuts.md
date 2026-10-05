---
title: 키보드 단축키
description: Signal Lab의 모든 키보드 단축키 — 앱 어디서나, 실험 편집기, 신호 라이브러리, 도구 화면, 패널, 언어 메뉴에서.
---

# 키보드 단축키

Signal Lab이 응답하는 모든 키를, 작동하는 위치로 묶었습니다. 한 글자나 Delete 단축키는 필드에
입력하는 동안에는 작동하지 않습니다.

::: tip
Mac의 브라우저에서는 아래에 <kbd>Ctrl</kbd>이라고 쓰인 곳마다 <kbd>⌘</kbd>가 작동합니다.
단, 템플릿 필드의 <kbd>Ctrl</kbd>+<kbd>Space</kbd>는 예외입니다.
:::

## 어디서나 {#anywhere}

| 키 | 하는 일 |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>K</kbd> | 아무 화면에서나 신호 팔레트인 [[ui:sig.paletteTitle]]을 엽니다 — 필드에 입력하는 중에도. 다시 누르면 닫힙니다 |
| <kbd>F1</kbd> | [[ui:app.docs]]처럼, 지금 있는 화면의 페이지에서 이 문서를 엽니다 |
| <kbd>Tab</kbd>, <kbd>Shift</kbd>+<kbd>Tab</kbd> | 컨트롤 사이를 이동합니다. 컨트롤의 팁은 포커스를 받을 때 표시되며, 다른 키를 누르면 숨겨집니다 |
| <kbd>Space</kbd>, <kbd>Enter</kbd> | 포커스가 있는 버튼을 누릅니다 — 클릭할 수 있는 모든 것은 버튼입니다 |
| <kbd>Esc</kbd> | 열려 있는 대화 상자를 닫습니다 |

신호 팔레트에서:

| 키 | 하는 일 |
| --- | --- |
| 입력 | 신호를 필터링합니다 |
| <kbd>↑</kbd> <kbd>↓</kbd> | 신호를 고릅니다 |
| <kbd>Enter</kbd> | 보내고 팔레트를 닫습니다 |
| <kbd>Esc</kbd> | 팔레트를 닫습니다. 옆을 클릭해도 마찬가지입니다 |

## 실험 편집기 {#editor}

캔버스에서 — 텍스트 필드에 포커스가 없을 때:

| 키 | 하는 일 |
| --- | --- |
| <kbd>A</kbd> | 노드를 추가하는 메뉴를 엽니다([[ui:exp.addNode]]). 노드가 선택되어 있으면 새 노드가 그 뒤에 놓입니다 |
| <kbd>Delete</kbd> 또는 <kbd>Backspace</kbd> | 선택된 연결 또는 선택된 노드를 제거합니다 — [[ui:exp.node.start]]나 [[ui:exp.node.end]]는 절대 안 됩니다 |
| <kbd>Tab</kbd> | 노드, 그 포트, 연결 사이를 이동합니다. 포커스를 받은 노드나 연결이 선택됩니다 |
| 출력에서 <kbd>Enter</kbd> 또는 <kbd>Space</kbd>, 그다음 노드나 그 입력에서 | 그것들을 연결합니다 |
| <kbd>←</kbd> <kbd>→</kbd> <kbd>↑</kbd> <kbd>↓</kbd> | 노드에 포커스가 있을 때 선택된 노드를 5포인트씩(<kbd>Shift</kbd>와 함께면 20) 이동합니다. 한 번 누르고 있으면 기록의 한 단계입니다 |
| <kbd>Esc</kbd> | 추가 메뉴를 닫고, 아니면 그리던 연결을 취소하고, 아니면 선택된 연결을 버리고, 아니면 여러 노드 선택을 지우고, 아니면 [[ui:exp.fullscreen]]을 나가고, 아니면 [[ui:exp.focus]]를 나갑니다 |
| <kbd>Ctrl</kbd>+<kbd>Z</kbd> | [[ui:exp.undo]] |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd>, <kbd>Ctrl</kbd>+<kbd>Y</kbd> | [[ui:exp.redo]] |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | [[ui:exp.duplicate]]: 선택된 노드와 그 사이 연결의 복사본 |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | 종류, 하는 일, id로 노드를 찾습니다 |
| <kbd>Ctrl</kbd>+<kbd>0</kbd> | [[ui:exp.fit]]: 그래프 전체를 화면에 |
| <kbd>Ctrl</kbd>+<kbd>1</kbd> | [[ui:exp.resetZoom]]: 100 % |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:exp.sendNow]] — 대기라면 [[ui:exp.listenNow]]: 실험을 실행하지 않고 선택된 노드만 |
| <kbd>Ctrl</kbd>+<kbd>A</kbd> | 모든 노드를 선택합니다 |
| <kbd>Ctrl</kbd>+<kbd>C</kbd> | [[ui:exp.copy]]: 선택된 노드와 그 사이 연결. 복제와 마찬가지로 [[ui:exp.node.start]]와 [[ui:exp.node.end]]는 빠집니다 |
| <kbd>Ctrl</kbd>+<kbd>X</kbd> | 그것들을 잘라냅니다 |
| <kbd>Ctrl</kbd>+<kbd>V</kbd> | 여기 또는 다른 실험에서 복사한 노드를 새 id로 붙여넣습니다 |

<kbd>Ctrl</kbd>+<kbd>A</kbd>, <kbd>C</kbd>, <kbd>X</kbd>, <kbd>V</kbd>는 편집기에
포커스가 있거나, 아무것에도 없고 페이지에 선택된 텍스트가 없을 때 노드에 작동합니다. 다른
곳 — [[ui:console.title]], 보고서 — 에 텍스트가 선택되어 있으면 페이지 자체의 것입니다:
<kbd>Ctrl</kbd>+<kbd>C</kbd>는 그 텍스트를 복사합니다.

마우스로:

| 제스처 | 하는 일 |
| --- | --- |
| 빈 캔버스에서 드래그 | 보기를 이동합니다 |
| 빈 캔버스에서 <kbd>Shift</kbd> + 드래그 | 프레임이 닿는 노드를 선택에 추가합니다 |
| 노드에서 <kbd>Shift</kbd> 또는 <kbd>Ctrl</kbd> + 클릭 | 선택에 추가하거나 제외합니다 |
| 선택된 여러 노드 중 하나를 드래그 | 모두 이동합니다 |
| 빈 캔버스에서 두 번 클릭 | 거기에 추가 메뉴를 엽니다 |
| <kbd>Ctrl</kbd> + 휠 | 포인터 위치에서 확대/축소합니다 |

추가 메뉴, 노드 찾기, 속성에서:

| 위치 | 키 | 하는 일 |
| --- | --- | --- |
| 추가 메뉴 | 입력 | 노드와 저장된 신호를 필터링합니다 |
| 추가 메뉴 | <kbd>↑</kbd> <kbd>↓</kbd>, <kbd>Enter</kbd>, <kbd>Esc</kbd> | 고르기, 추가, 닫기 |
| 노드 찾기 | <kbd>↑</kbd> <kbd>↓</kbd> | 노드를 고릅니다 |
| 노드 찾기 | <kbd>Enter</kbd> | 캔버스에 보여주고 선택합니다 |
| 노드 찾기 | <kbd>Tab</kbd> | 검색 필드로 돌아갑니다 |
| 노드 찾기 | <kbd>Esc</kbd> | 닫습니다 |
| [[ui:exp.properties]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | 표시된 노드에 [[ui:exp.sendNow]] |
| [[ui:exp.properties]] | 필드에서 <kbd>Esc</kbd> | 노드가 선택된 채 캔버스로 돌아가며, 거기서 <kbd>A</kbd>가 다음 노드를 추가합니다 |

템플릿(`{{name}}`)을 받는 필드에서:

| 키 | 하는 일 |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>Space</kbd> | 커서에 `{{`를 입력하고 거기에 올 수 있는 것을 나열합니다: 매개변수, 변수, 시크릿, 발생기 |
| `{{` 입력 | 같은 것을 나열합니다 |
| <kbd>↑</kbd> <kbd>↓</kbd> | 목록에서 고릅니다 |
| <kbd>Enter</kbd> 또는 <kbd>Tab</kbd> | 선택을 삽입하고 `}}`를 닫습니다 |
| <kbd>Esc</kbd> | 목록을 닫습니다. 필드는 포커스를 유지합니다 |

실험의 대화 상자에서:

| 위치 | 키 | 하는 일 |
| --- | --- | --- |
| [[ui:exp.params]] | 마지막 매개변수 값에서 <kbd>Enter</kbd> | 매개변수를 하나 더 추가합니다 |
| [[ui:exp.params]], [[ui:exp.runWith]] | <kbd>Esc</kbd> | 대화 상자를 닫습니다 |
| [[ui:exp.runWith]] | 필드에서 <kbd>Enter</kbd> | 이 값으로 [[ui:exp.run]] |
| [[ui:exp.secrets]] | 값에서 <kbd>Enter</kbd> | 저장합니다 |
| [[ui:exp.secrets]] | 값이나 새 이름에서 <kbd>Esc</kbd> | 취소합니다. [[ui:exp.params]]는 열린 채로 남습니다 |

## 신호 라이브러리 {#signals}

| 위치 | 키 | 하는 일 |
| --- | --- | --- |
| [[ui:nav.signals]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:sig.fire]]: 선택된 신호를 보냅니다 |
| [[ui:nav.signals]] | 신호를 두 번 클릭 | 보냅니다 |
| 폴더 | <kbd>F2</kbd> | [[ui:sig.renameFolder]] |
| 폴더 이름 바꾸기 | <kbd>Enter</kbd>, <kbd>Esc</kbd> | 새 이름을 유지하거나 취소합니다 |
| 폴더 | <kbd>Delete</kbd>, 두 번 | [[ui:sig.removeFolder]]; 안에 있는 것은 한 단계 위로 이동합니다 |
| 폴더 | <kbd>→</kbd> <kbd>←</kbd> | 엽니다, 닫습니다 |
| [[ui:nav.http]], [[ui:nav.osc]], [[ui:nav.mqtt]] 송신기 | <kbd>Ctrl</kbd>+<kbd>S</kbd> | 메시지가 라이브러리 신호에 연결되어 있고(거기서 열었거나 거기에 저장함) 변경되었을 때 [[ui:sig.save]]; 아직 라이브러리에 없으면 [[ui:sig.saveNew]] |
| [[ui:sig.saveTitle]] | <kbd>Enter</kbd>, <kbd>Esc</kbd> | 저장하거나 취소합니다 |

## 도구 화면 {#tools}

| 화면 | 키 | 하는 일 |
| --- | --- | --- |
| [[ui:nav.http]] | URL, 헤더, 자격 증명 또는 타임아웃에서 <kbd>Enter</kbd> | 요청을 보냅니다 |
| [[ui:nav.http]] | 본문을 포함해 요청 어디서나 <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | 요청을 보냅니다 |
| [[ui:nav.osc]] | 대상, 주소 또는 인수에서 <kbd>Enter</kbd> | 메시지를 보냅니다 |
| [[ui:nav.ws]] | URL에서 <kbd>Enter</kbd> | 연결합니다 |
| [[ui:nav.ws]] | [[ui:ws.message]]에서 <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | 보냅니다 |
| [[ui:nav.mqtt]] | [[ui:mq.addSubscription]]에서 <kbd>Enter</kbd> | 구독합니다 |
| [[ui:nav.broadcast]] | [[ui:bc.targetAddress]] 또는 [[ui:bc.targetCidr]]에서 <kbd>Enter</kbd> — [[ui:bc.mode.list]]를 제외한 모든 모드 | [[ui:bc.sendOnce]] |

[[ui:feedback.title]]에서: [[ui:feedback.message]]에서 <kbd>Ctrl</kbd>+<kbd>Enter</kbd>가
보내고([[ui:feedback.email]]에서 <kbd>Enter</kbd>도 마찬가지), 대화 상자 어디서나
<kbd>Ctrl</kbd>+<kbd>V</kbd>가 클립보드의 스크린샷을 붙여넣으며, <kbd>Esc</kbd>가 닫습니다.

## 패널 {#panes}

패널 사이의 핸들 — [[ui:layout.console]], [[ui:layout.properties]], [[ui:layout.timeline]] —
은 <kbd>Tab</kbd>으로 포커스를 받습니다:

| 키 | 하는 일 |
| --- | --- |
| <kbd>↑</kbd> <kbd>↓</kbd> | 콘솔과 타임라인: 16픽셀씩 높이기, 낮추기 |
| <kbd>←</kbd> <kbd>→</kbd> | 속성: 16픽셀씩 넓히기, 좁히기(인터페이스가 오른쪽에서 왼쪽으로 읽을 때는 반대) |
| <kbd>Shift</kbd> + 화살표 | 네 배로 |
| <kbd>Home</kbd>, <kbd>End</kbd> | 가장 작은 크기, 가장 큰 크기 |
| <kbd>Enter</kbd> | 기본 크기; 핸들을 두 번 클릭해도 마찬가지입니다 |

## 언어 메뉴 {#language}

헤더의 국기와 글자.

| 위치 | 키 | 하는 일 |
| --- | --- | --- |
| 버튼 위에서 | <kbd>↓</kbd> 또는 <kbd>↑</kbd> | 목록을 엽니다 |
| 목록에서 | <kbd>↓</kbd> <kbd>↑</kbd> | 다음 언어, 이전 언어 |
| 목록에서 | <kbd>Home</kbd>, <kbd>End</kbd> | 처음, 마지막 |
| 목록에서 | 글자 | 이름이 — 그 언어, 사용자의 언어 또는 영어로 — 또는 글자가 그것으로 시작하는 다음 언어 |
| 목록에서 | <kbd>Enter</kbd> 또는 <kbd>Space</kbd> | 그 언어로 전환합니다 |
| 목록에서 | <kbd>Esc</kbd> 또는 <kbd>Tab</kbd> | 목록을 닫습니다 |

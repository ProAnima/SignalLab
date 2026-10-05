---
title: 실행과 결과
description: 실험의 값이나 다른 값으로 실행 시작하기, 타임라인, 중지, 통과와 실패의 의미, 실행 보고서, 시드, 노드 하나 시험하기, 그리고 버전이 있는 실험 파일.
---

# 실행과 결과

## 실행 시작하기 {#start}

편집기 도구 모음에서 [[ui:exp.run]]을 누릅니다. 무엇이든 보내기 전에:

1. 편집기가 검사하는 대로 실험을 검사하고 — 그래프, 필드, 이름, 값([실행 전에 검사하는
   것](flow.md#validation)) — 사용하는 모든 시크릿이 저장되어 있어야 합니다([시크릿](data.md#secret-check)).
2. 저장합니다.
3. 실행은 전체 길이 동안 필요한 것을 엽니다: 에뮬레이터, 장애 릴레이, 대기가 수신하는 소켓,
   MQTT 구독입니다.

이 중 하나라도 실패하면 아무것도 실행되지 않습니다: 문제가 표시되고, 그 문제에 관한 노드가
선택됩니다. 그렇지 않으면 캔버스 아래에 타임라인이 열리고 단계가 일어나는 대로 나타납니다. 실행이
진행되는 동안 [[ui:exp.run]]은 [[ui:common.stop]]이 되고 실험은 편집할 수 없습니다.

실행은 활성 프로필의 값과, 실험에 고정된 시드 또는 새 시드를 사용합니다. 다른 값으로 한 번
실행하려면 [[ui:exp.runWith]]를 쓰십시오.

### 다른 값으로 실행하기 {#run-with}

[[ui:exp.run]] 옆의 ▾가 [[ui:exp.runWith]]를 엽니다: 실험을 바꾸지 않고 한 번의 실행에 다른 값을
씁니다.

| 필드 | 내용 | 비어 있으면 |
| --- | --- | --- |
| [[ui:exp.profile]] | 이 실행의 프로필. 실험에 프로필이 있으면 표시됨 | 활성 프로필 |
| 각 매개변수 | 이 실행에만 쓸 값 | 고른 프로필의 값, 회색으로 표시됨 |
| [[ui:exp.seed]] | 이 실행의 시드, 0–9 007 199 254 740 991. 옆의 버튼이 마지막 실행의 시드를 채움 | 고정된 시드, 또는 새 시드 |

양식의 [[ui:exp.run]]이 실행을 시작하고, [[ui:exp.resetOverrides]]가 양식을 비웁니다. 입력한
내용은 세션 동안 양식에 남으므로, 같은 변경을 다음에는 한 번의 클릭으로 할 수 있습니다. 실행되지
않을 프로필에는 ⚠가 붙습니다. 값의 우선순위는 [데이터](data.md#precedence)에 있습니다.

실험에 프로필이 있거나 실행에 값을 입력했으면, 타임라인은 실행이 어느 프로필을 썼는지 — 또는
[[ui:exp.runDefaults]] — 를 알려 주고, 값을 입력했으면 [[ui:exp.overridden]]도 알려 줍니다.

## 타임라인 {#timeline}

[[ui:exp.timeline]]은 캔버스 아래에 있습니다. ▸와 ▾로 접고, 가장자리를 끌어 크기를 바꿉니다.
단계 이벤트마다 한 행이 오래된 것부터 나열됩니다: 시각, 노드, 상태, 일어난 일 —
`HTTP 200 · 41 ms`, `token = abc123`, `/pong 42 ← 127.0.0.1:9000 · 12 ms`. 실패는 이유를 알려
주고, 기술적 세부는 툴팁에 있습니다. 행을 클릭하면 캔버스에서 그 노드가 선택됩니다.

| 상태 | 단계 |
| --- | --- |
| [[ui:exp.running]] | 시작됨 |
| [[ui:exp.passed]] | 잘 끝나고 출력을 골랐음 |
| [[ui:exp.failed]] | 실패했음. 첫 실패가 실행의 실패 |
| [[ui:exp.retry]] | 한 번 실패했고 다시 시도할 것임([재시도](flow.md#retry)) |
| [[ui:exp.repeating]] | 계속 보내는 중이며 1초에 최대 한 행([반복](flow.md#repeat)) |
| [[ui:exp.load]] | 부하 상태이며 1초에 한 행([부하](load.md#progress)) |

캔버스에서 각 노드는 최신 상태를 나타내는 배지를 답니다.

타임라인의 머리글에는 다음이 있습니다:

- 결과: [[ui:exp.running]], [[ui:exp.passed]], 이유가 붙은 [[ui:exp.failed]], 또는
  [[ui:exp.stopped]];
- 보고서가 작성되면 [[ui:exp.reportSaved]] — 브라우저에서는 내려받는 링크, 데스크톱 앱에서는
  툴팁에 경로;
- 이 실행을 이전 실행과 나란히 두는 [[ui:exp.compare]] ([실행 비교하기](load.md#compare));
- 위에서처럼 프로필과 바뀐 값;
- [[ui:exp.pinSeed]]가 붙은 실행의 시드, 또는 실험에 고정된 시드가 있으면 [[ui:exp.unpinSeed]]가
  붙은 그 시드([시드](#seeds)).

**프레임.** [[ui:dock.inspector]]가 캡처 중일 때, 메시지를 일치시킨 대기 — 또는 회신을 기다리는
전송 — 는 그 메시지 프레임의 번호를 보관합니다. 행 아래의 버튼이 노드와 프레임을 알려 주며, 누르면
하단 패널의 인스펙터가 그 프레임을 선택한 채로 열립니다. [인스펙터](../tools/inspector.md)를
참고하십시오.

타임라인은 이 세션에서 실험의 마지막 실행을 보여 줍니다. 다른 실험을 열면 비워집니다.

## 중지 {#stop}

[[ui:common.stop]]을 누르거나, 모든 작업을 한꺼번에 멈추려면 헤더의 [[ui:app.stopAll]]을
누릅니다. 실행은 곧바로 끝나고([중지](flow.md#stop)), 타임라인은 [[ui:exp.stopped]]를 표시하며,
**보고서는 저장되지 않습니다**. 명령줄이나 서버의 API에서 시작한 실행도 여느 작업과 같습니다: 그
서버의 [[ui:app.stopAll]]이 그것도 멈추며, 호출한 쪽은 중지되었음을 알게 됩니다.

## 결과 {#result}

| 결과 | 의미 | 보고서 |
| --- | --- | --- |
| [[ui:exp.passed]] | 모든 분기가 끝나고, 실패한 단계가 없고, 종료에 도달함 | 저장됨 |
| [[ui:exp.failed]] | 단계가 실패함 — 검사, [[ui:exp.portTimeout]] 연결이 없는 대기, 네트워크 오류, 임계값 — 또는 실행 시간이 다 됨(`run.timeout`), 합류가 헛되이 기다림(`run.join_waiting`), 아무 분기도 종료에 도달하지 못함(`run.no_end`) | 첫 실패와 함께 저장됨 |
| [[ui:exp.stopped]] | 누군가 중지함 | 없음 |
| 시작하지 않음 | 실험이 잘못되었거나, 시크릿이 없거나, 포트를 열 수 없음 | 없음 |

실패한 실행은 첫 실패의 노드와 필드를 알려 줍니다. [오류 참조](../reference/errors.md)에 모든
코드가 나열되어 있습니다. 명령줄은 종료 코드로 같은 것을 알려 줍니다: `0` 통과, `1` 실패,
`2` 실험이나 호출이 잘못됨(없는 시크릿도 포함), `3` 포트를 열 수 없는 것처럼 실험 밖의 무언가가
실행을 막음. [`signallab run`](../automation/cli.md#cli-run)을 참고하십시오.

## 실행 보고서 {#report}

스스로 끝난 모든 실행은 — 통과하든 실패하든 — 데이터 폴더의 `runs` 폴더에 JSON 보고서를 씁니다:
데스크톱에서는 `Documents/SignalLab/runs`, 서버에서는 서버 자체의 데이터 폴더입니다([파일](../reference/files.md)).
파일은 `run-<start time in ms>-<job number>.json`이며, 보고서가 다른 보고서를 덮어쓰는 일은
없습니다. 쓸 수 없으면 편집기가 이유를 알려 줍니다.

| 키 | 내용 |
| --- | --- |
| `version` | 보고서의 형식, 현재 5 |
| `experiment` | 실험의 이름 |
| `document_version` | 실험의 버전, 현재 9 |
| `seed` | 실행이 사용한 시드 |
| `profile` | 실행에 쓴 프로필, 기본값이면 `null` |
| `overrides` | [[ui:exp.runWith]]에 입력한 값 |
| `params` | 실행이 사용한 모든 매개변수 값 |
| `started_ms`, `ended_ms` | Unix 밀리초 |
| `outcome` | `passed` 또는 `failed` |
| `error` | 첫 실패, 또는 `null` |
| `steps` | 모든 단계 이벤트, 순서대로(아래) |
| `emulators` | 각 에뮬레이터 노드의 집계 — 있을 때 표시됨([에뮬레이터](faults.md#emulator)) |
| `impairments` | 각 장애 노드의 집계와 구간 — 있을 때 표시됨([구간](faults.md#change-impairment)) |

각 단계 이벤트에는 다음이 있습니다:

| 키 | 내용 |
| --- | --- |
| `job_id`, `node_id` | 실행과 노드 |
| `ts` | Unix 밀리초 |
| `state` | `running`, `passed`, `failed`, `retry`, `repeating`, `load` |
| `detail` | 무슨 일이 있었는지, 영어로 |
| `message_key`, `message_params` | 인터페이스의 텍스트와 그 값과 같아, 어느 언어로든 단계를 표시할 수 있음 |
| `vars` | 단계가 쓴 변수(있으면) |
| `error` | 실패한 이유: `code`, `params`, `node`, `field`, `detail` |
| `frame` | 캡처가 켜져 있었으면 대기가 일치시킨 인스펙터 프레임 |
| `load` | 부하가 측정한 것([측정 항목](load.md#metrics)), 마지막 이벤트에 |

시크릿 값은 보고서에 절대 나타나지 않습니다: `••••`로 마스킹됩니다([마스킹](data.md#masking)).

보고서의 형식은 기능과 함께 자랐습니다: 버전 3에서 에뮬레이터의 집계, 버전 4에서 장애의 구간,
버전 5에서 부하의 측정값이 추가되었습니다.

보고서는 실행 기록입니다: [[ui:exp.compare]]가 그것을 읽고,
[`experiment_runs`](../api/commands.md#experiment_runs)도 읽습니다. 명령줄의 `--report`는 실행의
보고서를 원하는 곳에 복사합니다.

## 시드 {#seeds}

모든 실행에는 시드가 있습니다. 0부터 9 007 199 254 740 991까지의 정수입니다. 순서는 다음과
같습니다:

1. [[ui:exp.runWith]]에서, 명령줄(`--seed`)에서, 또는 API에 이 실행을 위해 준 시드.
2. 실험에 고정된 시드.
3. 새 무작위 시드.

타임라인의 실행 첫 행이 그것을 알려 주고, 보고서가 그것을 보관합니다.

시드가 실행이 하는 모든 무작위를 결정합니다: 템플릿의 [생성기](data.md#generators),
[반복](flow.md#repeat)의 지터, [무작위 부하](load.md#schedule)의 도착, [장애 릴레이](faults.md#seed)에서
모든 패킷의 운명, 에뮬레이터의 무작위 선택입니다. 각각은 자기 스트림에서 뽑으므로, 병렬 분기가
서로의 값을 밀어내는 일은 없습니다.

실행을 반복하려면:

1. 타임라인에서 그 실행의 시드 옆의 [[ui:exp.pinSeed]]를 누릅니다. 시드는 실험에 저장되고,
   [[ui:exp.unpinSeed]]를 누를 때까지 모든 실행이 그것을 사용합니다. [[ui:exp.params]]에서
   [[ui:exp.seed]]가 고정된 시드를 보여 주고 편집하며, 비어 있으면 [[ui:exp.seedRandom]]입니다.
2. 같은 프로필과 값으로 실행합니다. 보고서가 그것들을 나열합니다.

시드로도 반복할 수 없는 것: 시간(`{{now}}`), `{{run.id}}`, 그리고 장치와 네트워크가 응답하는
시점입니다.

## 노드 하나 시험하기 {#send-now}

실험을 실행하지 않고 노드 하나를 시험하려면, 그 노드를 선택하고 [[ui:exp.node.http]],
[[ui:exp.node.tcp]], [[ui:exp.node.osc]], [[ui:exp.node.udp]], [[ui:exp.node.mqtt]],
[[ui:exp.node.ws_connect]], [[ui:exp.node.ws_send]] 노드에서 [[ui:exp.sendNow]]를 — 또는 속성에서
<kbd>Ctrl</kbd>+<kbd>Enter</kbd>를 — 누릅니다. 대기에서는 [[ui:exp.listenNow]]입니다: 지금부터
메시지가 일치하거나 시간 제한이 끝날 때까지 수신합니다.

엔진은 실행이 쓰는 코드로 그 노드를 한 번 수행합니다:

- 활성 프로필의 값, 이 세션에서 알려진 변수 값(마지막 실행과 이전 시도에서), 저장된 시크릿으로;
- 고정된 시드 또는 새 시드로. `{{run.id}}`는 `0`, `{{counter}}`는 `1`입니다;
- 재시도, 반복, 부하 없이 — 한 번 전송;
- **쿠키 없이**: 요청 하나, 그 앞에 돌려보낼 것으로 설정된 것이 없음;
- 실행의 릴레이와 에뮬레이터 없이. [[ui:exp.node.wait_http]]는 자체 리스너에서 수신하고,
  WebSocket 전송이나 대기는 [[ui:exp.node.ws_connect]]가 설명하는 연결을 그 한 번의 시도를 위해
  엽니다.

노드가 사용하는 이름에 아직 값이 없으면 아무것도 보내지 않고, 결과가 어떤 이름이 없는지 알려
줍니다. 먼저 실험을 실행하거나, 그 이름을 설정하는 노드에서 [[ui:exp.sendNow]]를 쓰십시오.

결과는 ✓ 또는 ✕와 일어난 일을 보여 줍니다. HTTP 요청이면 상태, 시간, 크기,
[[ui:http.response]]도 보여 줍니다. JSON 응답에서는 각 값을 클릭해 [추출](data.md#extract)할 수
있고, [[ui:http.mockThis]]는 그 응답을 에뮬레이터의 라우트로 바꿉니다. 대기가 받은 값, 또는 요청
바로 뒤의 [[ui:exp.node.extract]] 노드들이 그 응답에서 가져갈 값은 미리보기와 다음
[[ui:exp.sendNow]]에 알려집니다. 실행이 진행되는 동안에는 [[ui:exp.sendNow]]를 쓸 수 없습니다.

템플릿 노드의 미리보기 — [무엇을 보낼지](data.md#preview) — 도 엔진이 해석하며, 아무것도 보내지
않습니다.

## 실험 파일 {#files}

### 작업 중인 실험 {#working-file}

편집기는 실험을 하나 보관하며, 변경이 있을 때마다 0.7초 뒤 데이터 폴더의 `experiment.json`에
스스로 저장합니다. 도구 모음은 [[ui:exp.saving]], [[ui:exp.saved]], [[ui:exp.saveError]]를
표시합니다. 미완성 그래프도 저장됩니다. 읽을 수 없는 파일은 경로와 함께 보고되며, 절대 교체되지
않습니다. 실험 파일은 최대 4 MiB입니다.

서버에서는 파일이 서버의 데이터 폴더에 있으므로, 거기서 편집기를 여는 모든 브라우저가 같은 실험을
다룹니다.

### 열기, 템플릿, 내보내기 {#open-export}

도구 모음의 ☰ 버튼이 [[ui:exp.documents]]를 엽니다:

- [[ui:exp.templates]]: [[ui:exp.templateEmpty]], [[ui:exp.templateHttp]],
  [[ui:exp.templateBranch]], [[ui:exp.templateParallel]],
  [[ui:exp.templatePingReply]], [[ui:exp.templatePoll]],
  [[ui:exp.templateFlaky]], [[ui:exp.templateFaults]],
  [[ui:exp.templateOutage]], [[ui:exp.templateWsEcho]]. 그 대상은 `127.0.0.1`에 있습니다.
- [[ui:exp.importJson]]은 최대 4 MiB의 파일 — 이 버전의 실험 형식이거나, 열 때 최신으로 맞추는
  예전 버전 — 을 읽고, 이름과 노드·연결 수를 보여 주기 전에 검사합니다. 파일은 편집기가
  들여쓰기해 쓰는 형태로도 4 MiB 안에 들어야 하므로, 한도에 가까운 압축 파일은 거부될 수
  있습니다. 망가진 파일은 문제의 줄과 열과 함께 거부되고, 더 새로운 Signal Lab의 파일은
  `doc.version_unsupported`로 거부되며, 현재 실험은 그대로 남습니다.
- [[ui:exp.openDocument]]는 현재 실험을 고른 것으로 교체합니다. <kbd>Ctrl</kbd>+<kbd>Z</kbd>는 이
  세션 동안 이전 것을 되돌립니다. 실험을 여는 것이 그것을 실행하지는 않습니다.
- [[ui:exp.exportJson]]는 데이터 폴더의 `exports` 폴더에
  `experiment-<time in ms>-<random>.json` 형식의 사본을 씁니다. 다른 사본을 덮어쓰지 않습니다.
  브라우저에서는 [[ui:common.download]]가 그것을 내려받습니다.

명령줄과 API는 같은 파일을 받고, 템플릿은 이름으로 받습니다: `empty`, `http-check`,
`status-branch`, `parallel-flows`, `osc-ping-reply`, `poll-until-ready`, `flaky-api`,
`fault-phases`, `dependency-outage`, `websocket-echo`.

### 문서 버전 {#versions}

실험 파일에는 `version`이 있습니다. 이 Signal Lab은 버전 9를 쓰고 모든 이전 버전을 열며, 예전
파일이 담지 못한 것을 채웁니다. 버전 9보다 새로운 파일은 담긴 것을 빼고 여는 대신
거부됩니다(`doc.version_unsupported`).

| 버전 | 추가된 것 |
| --- | --- |
| 2 | 매개변수와 시드 |
| 3 | 프로필 |
| 4 | 재시도, 그리고 OSC나 UDP 전송이 기다리는 회신 |
| 5 | 반복, 그리고 [[ui:exp.node.loop]] |
| 6 | [[ui:exp.node.emulator]]와 [[ui:exp.node.wait_http]] |
| 7 | [[ui:exp.node.impairment]], [[ui:exp.node.impairment_change]], [[ui:exp.node.emulator_state]] |
| 8 | WebSocket 노드, HTTP 인증, 쿠키 저장소 |
| 9 | HTTP 요청의 부하, 그리고 TCP를 통한 장애 |

버전 8 이전의 파일은 [[ui:exp.cookies]]가 꺼진 채로 열려 예전처럼 실행됩니다. 더 새로운 파일은
자기 설정을 유지합니다. 다시 저장하면 어떤 파일이든 버전 9가 됩니다.

## 명령줄이나 서버에서 {#automation}

실행은 어디서나 같습니다: 명령줄과 서버의 API가 편집기와 같은 실행을 시작하며, 단계, 결과,
보고서도 같습니다.

```bash
signallab run checkout.json --profile Stage -p api=http://192.0.2.10:8080 --seed 42 --report report.json
```

- [`signallab run`](../automation/cli.md#cli-run)은 이 프로세스나 서버에서 실험 파일이나 템플릿을
  실행하고, 타임라인처럼 단계를 출력하며, 결과의 코드로 종료합니다.
- [`POST /api/run`](../api/run.md)은 서버에서 실행 하나를 돌리고 결과로 응답하거나, 단계가
  일어나는 대로 스트리밍합니다. 클라이언트가 사라져도 실행은 멈추지 않습니다: 끝까지 실행되고
  보고서를 보관합니다.
- CI에서는: [GitHub Actions와 그 밖의 것](../automation/ci.md).

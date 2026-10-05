---
title: 파일과 폴더
description: Signal Lab이 파일을 보관하는 위치 — 데이터 폴더, 실험, 신호 및 에뮬레이터 라이브러리, 실행 보고서와 내보내기 — 그 형식, 그리고 안전하게 편집·백업·이동할 수 있는 것.
---

# 파일과 폴더

Signal Lab이 보관하는 모든 것은 한 폴더, 즉 데이터 폴더 안의 평문 JSON(또는 텍스트)입니다.
시크릿 값은 여기에 없습니다.

## 데이터 폴더 {#data-folder}

| Signal Lab이 실행되는 곳 | 데이터 폴더 |
| --- | --- |
| 데스크톱 앱, Windows | 사용자 폴더의 `Documents\SignalLab`: `C:\Users\<you>\Documents\SignalLab` |
| 데스크톱 앱, Linux | `~/Documents/SignalLab` |
| 서버 | `--data-dir` 또는 `SIGNALLAB_DATA_DIR`; 둘 다 없으면 실행 사용자 홈 폴더의 `Documents/SignalLab` |
| 서버, Docker 이미지 | `/data`, 볼륨(compose 파일의 `signallab-data`) |
| `signallab run` | 종료할 때 제거되는 임시 폴더 — `--data-dir`가 하나를 지정하지 않는 한 |

데스크톱 앱은 환경에 설정되어 있으면 `SIGNALLAB_DATA_DIR`도 받습니다. 폴더는 처음 무언가를
쓸 때 만들어집니다.

::: tip
Windows에서 앱은 Windows가 문서를 다른 곳(OneDrive)에 보관하더라도 사용자 폴더 바로 안의
`Documents` 폴더를 사용합니다.
:::

서버에서는 파일이 사용자의 머신이 아니라 서버의 머신에 쓰입니다. 헤더의 [[ui:app.server]]
배지가 어디인지 팁에 표시하며, API는 이를 [`app_info`](../api/commands.md#app_info)의
`data_dir`로 주고, [`/api/files`](../api/index.md#files)가 그 안에 있는 것을 다운로드합니다.

명령줄의 `signallab emulate`, `signallab send`, `signallab mcp`는 데스크톱 앱과 같은 폴더에서
앱의 라이브러리를 읽습니다.

## 그 안의 내용 {#contents}

| 파일 | 무엇인지 | 쓰이는 시점 |
| --- | --- | --- |
| `experiment.json` | 편집기에서 열린 실험 | 변경 후 곧 |
| `signals.json` | 신호 라이브러리([[ui:nav.signals]]) | 변경 후 곧 |
| `emulators.json` | 에뮬레이터 라이브러리([[ui:nav.emulators]]) | 변경 후 곧 |
| `runs/run-<ms>-<job>.json` | 스스로 끝난 실행마다 보고서 하나 | 실행이 끝날 때 |
| `exports/experiment-<ms>-<16 hex digits>.json` | 실험의 스냅샷 | [[ui:exp.exportJson]] |
| `capture-<ms>.jsonl`, `capture-<ms>.txt` | 인스펙터의 프레임 | [[ui:ins.exportJsonl]], [[ui:ins.exportTxt]] |
| `token` | 서버의 액세스 토큰, 사용자만 읽을 수 있음 | `--generate-token`, 처음 시작할 때 |
| `.experiment-<hex>.tmp`, `.signals-<hex>.tmp`, `.emulators-<hex>.tmp` | 저장 중인 파일 | 잠시 후 이름이 바뀜 |

`<ms>`는 1970년 이후 밀리초 단위 시각이고, `<job>`은 실행의 작업 번호입니다. 서버에서는 모든
브라우저가 같은 `experiment.json`, 같은 라이브러리, 같은 보고서를 다룹니다.

## 형식 {#formats}

모두 UTF-8 JSON이며, 잘 읽히고 비교되도록 들여쓰기와 함께 쓰입니다. 각 파일에는 `version`이
있습니다. 이전 버전의 파일은 열 때 읽혀 마이그레이션되고, 다음 저장 때 현재 버전으로 다시
쓰입니다. 그 뒤에는 이전 Signal Lab이 열 수 없습니다.

### experiment.json {#experiment-json}

실험 문서, 버전 9 — [[ui:exp.exportJson]]이 쓰고 [[ui:exp.importJson]]이 읽는 것과 같은 JSON:

```json
{
  "version": 9,
  "name": "HTTP check",
  "params": [],
  "profiles": [],
  "profile": null,
  "seed": null,
  "cookies": true,
  "nodes": [ { "id": "start", "type": "start", "x": 40, "y": 80 }, … ],
  "edges": [ { "from": "start", "to": "request", "port": "next" }, … ]
}
```

- 최대 4 MiB, 노드 1~64개(`doc.node_count`).
- 버전 1~8은 열 때 마이그레이션됩니다. 버전 8 이전의 파일은 `cookies`가 꺼진 채로 열려
  원래대로 실행됩니다. 각 버전이 추가한 다른 설정(버전 2의 매개변수, 3의 프로파일, 4의
  재시도, 5의 반복과 루프, 6의 에뮬레이터, 7의 임페어먼트, 8의 WebSocket과 HTTP 인증, 9의
  부하)은 빈 상태로 시작합니다.
- 이 Signal Lab이 아는 것보다 새로운 버전은, 읽을 수 없는 부분 없이 여는 대신 거부됩니다
  (`doc.version_unsupported`).
- 파싱되지 않는 파일은 경로, 줄, 열과 함께 보고되며 절대 교체되지 않습니다.
- 임시 파일에 쓰고 이름을 바꾸므로, 쓰기가 실패해도 이전 파일이 남습니다.

노드, 매개변수, 프로파일이 무엇인지: [실험](../experiments/index.md),
[노드](../experiments/nodes.md), [데이터](../experiments/data.md).

### signals.json {#signals-json}

신호 라이브러리, 버전 2:

```json
{
  "version": 2,
  "signals": [
    {
      "id": "…",
      "name": "Go cue",
      "group": "Stage/Cues",
      "note": "",
      "body": { "transport": "osc", "target": "127.0.0.1:9000", "address": "/cue/go", "args": [ { "type": "int", "value": 1 } ] }
    }
  ],
  "folders": [ "Stage", "Stage/Cues" ]
}
```

- `group`은 `/` 경로로 표현한 신호의 폴더입니다. 비어 있으면 최상위입니다. `folders`(버전
  2에서 추가)는 빈 폴더를 포함해 모든 폴더를 나열하며, 없으면 생략됩니다. 버전 1 파일도
  같게 읽히지만 빈 폴더는 없습니다.
- `body`는 `osc`, `udp`, `http`, `mqtt` 중 하나입니다. 각 필드는
  [`signals_save`](../api/commands.md#signals_save)에 있습니다.
- 파일이 없으면 스타터 세트가 쓰이고(모든 대상이 `127.0.0.1`), 인터페이스 언어로 이름이
  바뀝니다.
- 파싱되지 않는 파일은 경로, 줄, 열과 함께 보고되며(`signals.json_invalid`) 스타터 세트로
  절대 교체되지 않습니다. 고치거나 삭제하십시오. 읽히지 않는 동안에는 아무것도 라이브러리를
  쓰지 않습니다. 저장은 같은 오류로 거부되고 파일은 [[ui:sig.reload]]가 다시 읽을 때까지
  그대로 남습니다. 저장은 같은 폴더의 임시 파일을 거치므로, 쓰기가 중단되어도 이전 파일이
  남습니다.

### emulators.json {#emulators-json}

에뮬레이터 라이브러리, 버전 1:

```json
{
  "version": 1,
  "emulators": [
    { "id": "demo-api", "note": "…", "emulator": { "name": "Demo API", "bind": "127.0.0.1:8080", "protocol": "http", "routes": [ … ] } }
  ]
}
```

각 항목은 `id`와 `note`가 있는 에뮬레이터 문서입니다. 문서는
[에뮬레이터](../tools/emulators.md)에 설명되어 있습니다. 신호와 마찬가지로, 없는 파일에는
스타터 세트(모두 `127.0.0.1`에 바인딩됨)가 들어가고, 손상된 파일은 보고되며
(`emulators.json_invalid`) 절대 교체되지 않습니다. 임시 파일을 거쳐 쓰입니다.
`signallab emulate`은 에뮬레이터 하나, 그 목록, 또는 이와 같은 라이브러리를 담은 자체 파일도
읽습니다.

### 실행 보고서 {#run-reports}

`runs/run-<started ms>-<job>.json`, 보고서 버전 5: 통과하거나 실패한 실행마다 파일 하나, 절대
덮어쓰지 않습니다(같은 이름의 두 번째 실행은 `-2`, `-3`…이 붙습니다). 중지된 실행은 아무것도
저장하지 않습니다.

| 필드 | 무엇인지 |
| --- | --- |
| `version` | 5 |
| `experiment` | 실험의 이름 |
| `document_version` | 실행된 문서의 버전 |
| `seed`, `profile` | 실행에 사용된 값 |
| `overrides` | 이 실행에만 주어진 값 |
| `params` | 사용한 모든 매개변수 값 |
| `started_ms`, `ended_ms` | 1970년 이후 밀리초 |
| `outcome` | `passed` 또는 `failed` |
| `error` | 첫 실패, 또는 null |
| `steps` | 모든 단계, [`experiment://step`](../api/events.md#event-experiment-step) 형식 |
| `emulators` | 각 [[ui:exp.node.emulator]] 노드가 받고 응답한 것(버전 3부터). 없으면 생략 |
| `impairments` | 각 [[ui:exp.node.impairment]] 노드의 릴레이가 단계별로 한 일(버전 4부터). 없으면 생략 |

부하 단계의 측정값은 마지막 단계에 있습니다(버전 5부터). 타임라인의 실행 기록과
[[ui:exp.compare]]가 이 파일들을 읽습니다. 읽을 수 없는 보고서는 목록에서 빠집니다.
[실행과 보고서](../experiments/runs.md)를 참고하십시오.

### 내보내기 {#exports}

- `exports/experiment-…json`: 위와 같은 실험 문서. 내보낼 때마다 새 파일입니다.
- `capture-….jsonl`: 한 줄에 인스펙터 프레임 하나, `data`에 보관된 바이트는 base64입니다.
- `capture-….txt`: 읽기 위한 프레임들, 각각 16진수 덤프와 함께.

서버에서는 인스펙터의 내보내기가 만들어지는 대로 사용자의 컴퓨터로 다운로드됩니다. 실험의
내보내기는 [[ui:common.download]]를 제공하며, 타임라인의 실행 [[ui:exp.reportSaved]]는
보고서를 다운로드하는 링크입니다.

## 시크릿은 이 파일들에 없습니다 {#secrets}

실험은 시크릿을 이름으로 지정하며 — `{{secret.API_TOKEN}}` — 이름만 기록됩니다. 값은 다음에
보관됩니다:

| Signal Lab이 실행되는 곳 | 시크릿 값이 있는 곳 |
| --- | --- |
| 데스크톱 앱, Windows | Windows 자격 증명 관리자, `SignalLab` 아래(편집기의 [[ui:exp.secrets]]) |
| 데스크톱 앱, Linux | 아무 데도 없음: 시크릿을 저장할 수 없습니다(`secret.unsupported`) |
| 서버 | 읽기 전용: 환경 변수 `SIGNALLAB_SECRET_<NAME>`, 또는 `--secrets-dir`(기본값 `/run/secrets/signallab`)의 `<NAME>` 파일 |
| `signallab` | 같은 파일과 변수, 또는 `--secrets system`을 쓰는 시스템 저장소 |

::: warning
필드에 바로 입력한 것은 입력한 그대로 보관됩니다. HTTP 신호의 자격 증명에 있는 비밀번호,
MQTT 브로커 에뮬레이터의 비밀번호, 헤더에 붙여넣은 토큰 — 모두 `signals.json`,
`emulators.json` 또는 `experiment.json`, 그리고 그 내보내기에서 평문입니다. 공유 폴더에 넣지
않을 것은 실험에서 `{{secret.NAME}}`을 사용하십시오.
:::

## 인터페이스 설정 {#settings}

인터페이스가 기억하는 것 — 언어, 각 화면에서 마지막으로 입력한 값, 열려 있는 패널과 그 크기,
[[ui:http.keepCookies]], 마지막으로 업데이트를 확인한 시각, 업데이트를 위한 설치의 임의 번호 —
은 데이터 폴더가 아니라 인터페이스 자체가 보관합니다. 데스크톱에서는 앱 자체의 저장소에,
서버의 페이지에서는 브라우저의 사이트 저장소에(브라우저별로). [[ui:nav.http]] 화면의 자격
증명은 거기에 보관되지 않습니다.

Signal Lab은 로그 파일을 쓰지 않습니다. [문제 해결](troubleshooting.md#logs)을 참고하십시오.

## 백업, 편집, 이동 {#backup}

- **백업**은 폴더 전체를 복사하면 됩니다. 안의 모든 것이 자기 완결적인 JSON이며, 시크릿 값이
  들어 있지 않으므로 새 머신에서 다시 설정하십시오.
- **편집**은 Signal Lab을 닫은 상태에서(또는 서버에서는 아무 페이지도 열려 있지 않을 때)
  `signals.json`, `emulators.json`, `experiment.json`을 손으로 하십시오. 앱은 자신이 가진
  것으로 파일 전체를 쓰므로, 실행 중에 한 변경은 다음 저장에 덮어써집니다. 실수는 파일을
  다음에 읽을 때 줄과 열과 함께 보고되며, 조용히 교체되지 않습니다 — `signals.json`은 앱이
  거기에 저장할 때도 마찬가지입니다. 그 저장은 거부되고 파일은 남겨 둔 그대로 남습니다.
- **삭제**는 `runs/`, `exports/`, `capture-*` 파일을 언제든 하십시오. `signals.json`이나
  `emulators.json`을 삭제하면 스타터 세트가, `experiment.json`을 삭제하면 스타터 실험이
  돌아옵니다.
- **이동**은 폴더를 복사하고 Signal Lab이 새 위치를 가리키게 하십시오. 서버는 `--data-dir`,
  데스크톱 앱은 `SIGNALLAB_DATA_DIR`.
- **공유**는 실험을 내보내거나, 실험이 테스트하는 프로젝트 옆에 그 JSON을 커밋하여
  하십시오. [`signallab run`](../automation/cli.md#cli-run)이 거기서 실행합니다.

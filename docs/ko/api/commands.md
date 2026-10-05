---
title: 명령
description: Signal Lab 엔진의 모든 명령으로, POST /api/invoke/<command>로 호출하며 인수, 결과, 오류를 가집니다.
---

# 명령

엔진이 가진 모든 명령을 다루는 대상별로 묶었습니다. 각 명령은 인수를 담은 JSON 객체와 함께
`POST /api/invoke/<command>`로 호출하며, 결과와 함께 `200`을, 또는
[`EngineError`](index.md#errors)와 함께 `422`를 응답합니다. 인증 방법과 상태 코드의 의미는
[API 개요](index.md)에 있습니다.

## 관례 {#conventions}

- **인수 이름**은 camelCase입니다(`jobId`, `nodeId`). 명령이 모르는 인수나 빠뜨린 필수 인수는
  `command.args_invalid`와 함께 거부되며, 인수를 받는 모든 명령이 이 오류로 실패할 수 있습니다.
  인수가 없는 명령은 본문을 읽지 않습니다.
- **인수로 전달되는 객체** — `config`, `request`, `document`, `library`, `emulator`, `profile` — 는
  엔진 자체의 필드 이름을 쓰며, 대부분 snake_case입니다(`timeout_ms`). 그 안에서 엔진이 모르는
  필드는 **무시**되므로, 철자가 틀린 선택적 필드는 조용히 기본값을 유지합니다. `feedback_send`의
  양식만이 모르는 필드를 거부합니다.
- **선택적** 인수와 필드는 빠뜨리거나 `null`로 보낼 수 있으며, 표에 기본값이 있습니다.
- **결과**는 JSON입니다. "null"은 명령이 돌려줄 것이 없다는 뜻입니다.
- **주소**를 `IP:port`로 적으면 숫자 주소와 포트를 받습니다(`127.0.0.1:9000`, `[::1]:9000`).
  그 자리에 호스트 이름은 거부됩니다. 표가 `IP:port` 또는 `host:port`라고 하면 호스트 이름도
  됩니다. 명령이 실행될 때 조회하며, IPv4 주소가 있으면 그것을 씁니다(따라서 `localhost:9000`은
  `127.0.0.1:9000`입니다).
- **작업**: *작업을 시작함*으로 표시된 명령은 [`JobInfo`](#type-jobinfo)를 돌려줍니다. 작업은
  끝나거나 [`job_stop`](#job_stop)으로 중지될 때까지 계속됩니다. [작업](index.md#jobs)을
  참고하십시오.
- **경로**는 결과에서 엔진이 실행되는 머신 기준입니다. 서버에서는 그 데이터 폴더 안이며,
  [`/api/files`](index.md#files)로 내려받습니다.

예제는 이 셸 함수를 사용합니다.

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
invoke() {
  curl -sS -X POST "$SERVER/api/invoke/$1" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    --data "${2:-}"
}
```

## 애플리케이션 {#application}

### app_info {#app_info}

엔진이 무엇이고 어디서 실행되는지입니다. 인수가 없습니다.

**결과**

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `version` | string | Signal Lab의 버전, `[[version]]` |
| `mode` | string | `desktop` 또는 `server` |
| `secrets_writable` | boolean | [`secret_set`](#secret_set)과 [`secret_delete`](#secret_delete)이 여기서 동작할 수 있는지: 서버에서는 `false` |
| `data_dir` | string | 데이터 폴더, 엔진이 실행되는 머신 기준 |
| `os` | string | `windows`, `linux`… |
| `arch` | string | `x86_64`, `aarch64`… |

### get_host_info {#get_host_info}

머신의 이름과 메시지를 보낼 때 쓸 주소입니다. 인수가 없습니다.

**결과**: `{ "local_ip": string, "hostname": string }`. `local_ip`는 시스템이 인터넷으로 가는
트래픽에 고르는 IPv4 주소이며(아무것도 보내지 않고 찾습니다), 없으면 `127.0.0.1`입니다.
`hostname`은 컴퓨터의 이름이며, 시스템이 말하지 않으면 `localhost`입니다.

### firewall_status {#firewall_status}

시스템 방화벽이 다른 머신에서 이 프로그램에 닿을 수 있게 하는지입니다. 프로그램별 방화벽을
읽을 수 있는 것은 Windows뿐이며, 다른 곳에서는 `applies`가 `false`이고 나머지 중 `program`만
채워집니다. 인수가 없습니다.

**결과**

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `applies` | boolean | 여기에 프로그램별 방화벽이 있음(Windows) |
| `program` | string | 규칙의 대상 프로그램 |
| `enabled` | boolean | 지금 머신이 있는 네트워크에서 방화벽이 켜져 있음 |
| `networks` | string[] | 머신이 있는 네트워크의 종류: `domain`, `private`, `public` |
| `allowed` | boolean | 인바운드 규칙이 현재 네트워크에서 이 프로그램의 UDP를 허용함 |
| `blocked` | boolean | 인바운드 규칙이 현재 네트워크에서 이 프로그램을 차단함. 허용 규칙을 이깁니다 |
| `rules` | number | 이 프로그램에 대한 모든 종류의 인바운드 규칙 |

**오류**: `firewall.failed`.

### firewall_allow {#firewall_allow}

다른 머신이 Signal Lab에 닿을 수 있게 합니다. 시스템이 자체 관리자 프롬프트를 표시한 뒤,
프로그램의 인바운드 규칙(차단 규칙 포함)이 Signal Lab과 그 옆의 `signallab` 명령줄에 대해 각각
하나의 허용 규칙으로 바뀝니다. Windows의 데스크톱 앱 전용이며, 서버는 화면에 프롬프트에 답할
사람이 없으므로 거부합니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `public` | boolean | 예 | 사설 및 도메인 네트워크뿐 아니라 공용 네트워크에서도 허용 |

**결과**: 새 [`firewall_status`](#firewall_status).

**오류**: `firewall.server`(서버에서), `firewall.unsupported`(Windows가 아님),
`firewall.declined`(프롬프트에 아니요라고 답함), `firewall.failed`.

### feedback_send {#feedback_send}

스튜디오의 허브를 통해 Signal Lab 개발자에게 메시지를 보냅니다. 허브가 개발자에게 메일로
전달합니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `form` | object | 예 | 아래의 메시지. 모르는 필드는 거부됩니다 |

| `form`의 필드 | 형식 | 기본값 | 의미 |
| --- | --- | --- | --- |
| `message` | string | — | 무슨 일이 있었는지. 필수이며 최대 20 000자 |
| `email` | string | 없음 | 답변을 받을 곳 |
| `meta` | object of strings | `{}` | 앱이 자신에 대해 말하는 것(version, os, arch, mode, lang, screen) |
| `screenshots` | `{ name, data }[]` | `[]` | 이미지, `data`는 base64. 최대 6개, 각 8 MiB |
| `logs` | `{ name, text }[]` | `[]` | 텍스트 파일. 최대 4개, 각 2 MiB |

모두 합쳐 최대 15 MiB입니다.

**결과**: `{ "id": string }`, 개발자가 받는 참조 번호입니다.

**오류**: `feedback.message_required`, `feedback.message_too_long`,
`feedback.too_many_files`, `feedback.file_too_large`, `feedback.too_large`,
`feedback.invalid`, 허브의 거부(`feedback.email_invalid`, `feedback.file_type`,
`feedback.rate_limited`, `feedback.disabled`, `feedback.send_failed`, `feedback.failed`),
그리고 네트워크의 `transport.*`.

```bash
invoke app_info
# {"version":"[[version]]","mode":"server","secrets_writable":false,"data_dir":"/data","os":"linux","arch":"x86_64"}
```

## 작업 {#jobs}

### jobs_list {#jobs_list}

실행 중인 작업을 오래된 것부터. 인수가 없습니다.

**결과**: [`JobInfo`](#type-jobinfo)`[]`.

### job_stop {#job_stop}

작업 하나를 즉시 중지합니다: 소켓이 닫히고, 릴레이, 서버 또는 연결이 사라집니다. 중지된 작업은
[`job://ended`](events.md#event-job-ended)를 보내지 않으며, 중지된 실행은 보고서를 저장하지
않습니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `id` | number | 예 | 작업의 `id` |

**결과**: 그 id의 작업이 실행 중이었으면 `true`, 아니면 `false`.

### jobs_stop_all {#jobs_stop_all}

누가 시작했든 실행 중인 모든 작업을 중지합니다. 인수가 없습니다.

**결과**: null.

```bash
invoke jobs_list
# [{"id":3,"kind":"osc-monitor","label":"OSC monitor 0.0.0.0:9000","params":{"bind":"0.0.0.0:9000"},"started_ms":1759600000000}]
invoke job_stop '{"id":3}'
# true
```

## 실험과 실행 {#experiments}

이 명령들은 실험 문서(`Experiment`)를 받고 돌려줍니다. 편집기가 저장하고 내보내는 JSON으로,
`version`, `name`, `params`, `profiles`, `profile`, `seed`, `cookies`, `nodes`, `edges`를
가집니다. 그 노드는 [노드](../experiments/nodes.md)에, 매개변수, 프로필, 템플릿은
[데이터](../experiments/data.md)에 있습니다. 문서는 최대 4 MiB입니다(`file.too_large`). 실험을
실행하고 결과를 기다리려면 [`experiment_start`](#experiment_start) 대신
[`POST /api/run`](run.md)을 사용하십시오.

### experiment_load {#experiment_load}

작업 중인 실험: 데이터 폴더의 `experiment.json`이며, 서버에서는 그 인터페이스가 보여 주는
것입니다. 없으면 시작 실험입니다. 오래된 문서 버전은 마이그레이션됩니다. 인수가 없습니다.

**결과**: `Experiment`.

**오류**: `file.io`, `file.json_invalid`(파일의 `path`, `line`, `column`과 함께),
`file.too_large`, `doc.version_unsupported` 그리고 다른 `doc.*` 검사.

### experiment_save {#experiment_save}

작업 중인 실험, 데이터 폴더의 `experiment.json`을 대체합니다. 먼저 임시 파일에 쓰므로, 쓰기에
실패해도 이전 것이 남습니다.

::: warning
서버에서는 모든 브라우저의 편집기가 작업하는 문서입니다.
:::

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `document` | `Experiment` | 예 | 문서 |

**결과**: string, 쓴 경로.

**오류**: `doc.*`, 문서의 크기 검사(`param.*`, `params.too_many`, `profile.*`,
`profiles.too_many`, `seed.range`), `file.too_large`, `file.io`.

### experiment_parse {#experiment_parse}

JSON 텍스트에서 실험을 읽습니다. [[ui:exp.importJson]]이 하는 것과 같습니다. 버전 1~8은 현재
버전인 9로 마이그레이션되며, 버전 8 이전의 파일은 `cookies`가 꺼진 채 열려 예전처럼
실행됩니다. 바이트 순서 표시는 건너뜁니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `text` | string | 예 | 파일의 텍스트 |

**결과**: `Experiment`.

**오류**: `file.json_invalid`(`line`, `column`), `file.too_large`,
`doc.version_unsupported`, `doc.*`, [`experiment_save`](#experiment_save)의 크기 검사.

### experiment_export {#experiment_export}

문서의 스냅숏을 데이터 폴더의 `exports/experiment-<ms>-<16 hex digits>.json`에 씁니다.
내보낼 때마다 새 파일입니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `document` | `Experiment` | 예 | 문서 |

**결과**: string, 쓴 경로.

**오류**: [`experiment_save`](#experiment_save)의 것들.

### experiment_validate {#experiment_validate}

문서가 활성 프로필(또는 기본값)로 실행될 수 있는지 검사합니다: 그래프, 모든 필드, 매개변수,
그리고 문서가 이름을 부르는 모든 시크릿이 저장되어 있는지. 막는 문제가 있으면 오류입니다.
성공하면 *다른* 프로필 중 어느 것이 실패할지 알려 주므로, 전환하기 전에 알 수 있습니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `document` | `Experiment` | 예 | 문서 |
| `overrides` | object of strings | 아니요 | 이번 검사에만 쓰는 매개변수 값. [[ui:exp.runWith]]가 주는 것과 같습니다 |

**결과**: `{ "profile": string or null, "error": EngineError }[]` — 검증되지 않을 다른
프로필마다 하나씩(프로필 없이 기본값은 `null`). 빈 목록이면 모든 프로필이 괜찮습니다.

**오류**: 모든 검증 코드(`doc.*`, `graph.*`, `node.*`, `param.*`, `profile.*`, `template.*`,
`loop.*`…), `run.override_unknown`(문서에 없는 매개변수에 대한 재정의), `secret.missing`,
`secret.store`, `secret.unsupported`.

### experiment_resolve {#experiment_resolve}

편집기의 미리보기가 보여 주는 대로 템플릿을 채운 노드 하나: 활성 프로필의 값과 직접 준 변수
값입니다. 시크릿은 `••••`로 표시되며 값은 결코 아닙니다. 값이 없는 이름은 적힌 그대로 남고
목록에 오릅니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `document` | `Experiment` | 예 | 문서 |
| `nodeId` | string | 예 | 노드 |
| `vars` | object | 예 | 이름별로 쓸 변수 값. 없으면 `{}` |

**결과**: `{ "node": node, "missing": string[] }`.

**오류**: `node.not_found`, `template.*`, `secret.store`, `secret.unsupported`.

### experiment_send_node {#experiment_send_node}

[[ui:exp.sendNow]]: 실행이 쓰는 같은 코드로 노드 하나를 단독으로 수행합니다. 동작이면 보내고,
대기이면 지금부터 일치하거나 시간 초과될 때까지 수신합니다. [[ui:exp.node.ws_send]] 또는
[[ui:exp.node.wait_ws]] 노드는 그 [[ui:exp.node.ws_connect]] 노드가 설명하는 연결을 엽니다.
쿠키 없이 아무것도 보내지 않으며, 실행의 [[ui:exp.node.impairment]]와
[[ui:exp.node.emulator]] 노드는 열리지 않습니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `document` | `Experiment` | 예 | 문서. 활성 프로필이 매개변수 값을 줍니다 |
| `nodeId` | string | 예 | 동작 또는 대기 |
| `vars` | object | 예 | 노드의 템플릿이 읽는 변수 값. 없으면 `{}` |

**결과**

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `detail` | string | 무슨 일이 있었는지, 영어 |
| `response` | [`HttpResponse`](#type-httpresponse) or null | HTTP 노드의 응답 |
| `vars` | object | 단계가 설정한 것: 대기의 회신, 또는 요청 뒤의 [[ui:exp.node.extract]] 노드가 그 응답에서 가져오는 것 |

시크릿 값은 이 모든 것에서 가려집니다.

**오류**: `node.not_found`, `run.not_an_action`(동작도 대기도 아님), `ws.connection_unknown`,
`secret.missing`, `template.*`, 그리고 단계가 실패하는 모든 것: `transport.*`,
`wait.timeout`, `check.*`…

### experiment_start {#experiment_start}

[[ui:exp.run]]이 하는 것처럼 실행을 시작하고 즉시 돌아옵니다. 그 단계는
[`experiment://step`](events.md#event-experiment-step) 이벤트로, 끝은
[`experiment://ended`](events.md#event-experiment-ended)로 도착하며, 보고서는 데이터 폴더의
`runs/` 아래에 저장됩니다. 대기, 에뮬레이터, 장애 릴레이, MQTT 구독은 첫 단계 전에 열리므로,
이미 사용 중인 포트는 여기서 실패합니다. 300초보다 긴 실행은 `run.timeout`으로 실패합니다.
*작업을 시작함*(`experiment`).

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `document` | `Experiment` | 예 | 문서 |
| `overrides` | object of strings | 아니요 | 이번 실행에만 쓰는 매개변수 값 |
| `seed` | number | 아니요 | 실행의 시드, 0~9007199254740991. 기본값: 문서의 것, 없으면 새 것 |

**결과**: [`JobInfo`](#type-jobinfo), `params.name`은 실험의 이름.

**오류**: [`experiment_validate`](#experiment_validate)가 보고하는 모든 것, `seed.range`,
`transport.address_in_use`와 다른 바인드 실패, `emulator.*`, `impair.*`,
`node.params_only`(실행이 시작될 때 고정되지 않은 수신 주소나 MQTT 대기의 브로커 또는 토픽),
그리고 MQTT 대기가 닿지 못하는 브로커의 [`mqtt_connect`](#mqtt_connect) 오류.

### experiment_runs {#experiment_runs}

`runs/`의 보고서에서 읽어 온 실행을 최신 것부터. 읽을 수 없는 보고서는 빠집니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `name` | string | 아니요 | 정확히 이 이름의 실험 실행만 |
| `limit` | number | 아니요 | 최대 이만큼. 기본값 50, 최대 500 |

**결과**: 실행 요약:

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `name` | string | 보고서의 파일 이름, `run-<ms>-<job>.json`: [`experiment_compare`](#experiment_compare)가 받는 것 |
| `experiment` | string | 실험의 이름 |
| `started_ms`, `ended_ms` | number | 1970년 이후 밀리초 |
| `outcome` | string | `passed` 또는 `failed` |
| `seed` | number | 실행의 시드 |
| `profile` | string or null | 프로필 |
| `loads` | object[] | 각 부하 단계: `node`, `sent`, `rps`, `p95_ms`, `error_rate`, `held`(모든 임계값이 지켜짐) |

**오류**: `file.io`.

### experiment_compare {#experiment_compare}

타임라인의 [[ui:exp.compare]]가 보여 주는 대로, 두 실행을 부하 단계별로 나란히. 단계는 노드
id로 짝지어집니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `a` | string | 예 | 이전 실행의 보고서 파일 이름 |
| `b` | string | 예 | 이후 실행의 보고서 파일 이름 |

**결과**: `{ "a": summary, "b": summary, "steps": [...] }`, 각 단계에는 `node`, `missing_in`
(한 실행에만 있을 때 `a` 또는 `b`), `metrics`, `sent`(`[a, b]`), `thresholds_a`,
`thresholds_b`(각 임계값은 `{ metric, op, value, actual, held }`)가 있습니다. `metrics`는 아홉
개의 지표를 나열하며 각각 `{ metric, a, b, change, percent, worse }`입니다: `metric`은
`p50_ms`, `p90_ms`, `p95_ms`, `p99_ms`, `mean_ms`, `max_ms`, `error_rate`, `rps` 또는
`missed`이고, `change`는 `b − a`, `percent`는 `a`에 대한 변화율(%)(`a`가 0일 때 null), `worse`는 잘못된
방향으로 5% 이상 움직였거나(`rps`는 낮아진 쪽) 0에서 다른 값으로 갔다는 뜻입니다. 한 실행에만
있는 단계는 결코 `worse`가 아닙니다. [부하](../experiments/load.md)를 참고하십시오.

**오류**: `runs.name_invalid`(보고서의 파일 이름이 아닌 것, 폴더 없음), `runs.not_found`,
`file.json_invalid`, `file.io`.

```bash
invoke experiment_validate "$(jq '{document: .}' experiment.json)"
# []
```

## 시크릿 {#secrets}

시크릿 값은 실험이 `{{secret.NAME}}`으로 사용하며 엔진을 떠나지 않습니다: 어떤 명령도 값을
돌려주지 않습니다. 어디에 보관되는지는 엔진이 어디서 실행되는지에 달렸습니다:

| 위치 | 저장소 | 설정과 제거 |
| --- | --- | --- |
| 데스크톱 앱, Windows | Windows 자격 증명 관리자 | 예 |
| 데스크톱 앱, Linux | 없음 | `secret.unsupported` |
| 서버 | `SIGNALLAB_SECRET_<NAME>`, 또는 `--secrets-dir`(기본값 `/run/secrets/signallab`)의 `<NAME>` 파일 | 아니요: `secret.read_only` |

이름은 라틴 문자 또는 `_`로 시작하고 라틴 문자, 숫자, `_`로 이어지며 최대 128자입니다
(`secret.name_invalid`).

### secret_status {#secret_status}

주어진 이름 중 저장된 값이 있는 것.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `names` | string[] | 예 | 조회할 이름 |

**결과**: 객체, 이름 → `true`(저장됨) 또는 `false`.

**오류**: `secret.name_invalid`(서버에서), `secret.store`, `secret.too_large`(서버의 파일이
16 KiB 초과), `secret.unsupported`.

### secret_set {#secret_set}

이름 아래에 값을 저장하고 거기 있던 것을 대체합니다. 데스크톱 앱 전용.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `name` | string | 예 | 이름 |
| `value` | string | 예 | 비어 있지 않고 최대 16 KiB |

**결과**: null.

**오류**: `secret.read_only`(서버에서), `secret.unsupported`, `secret.name_invalid`,
`secret.empty`, `secret.too_large`, `secret.store`.

### secret_delete {#secret_delete}

저장된 값을 제거합니다. 저장되지 않은 것을 제거하는 것은 오류가 아닙니다. 데스크톱 앱 전용.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `name` | string | 예 | 이름 |

**결과**: null.

**오류**: `secret.read_only`(서버에서), `secret.unsupported`, `secret.name_invalid`,
`secret.store`.

```bash
invoke secret_status '{"names":["API_TOKEN","MQTT_PASSWORD"]}'
# {"API_TOKEN":true,"MQTT_PASSWORD":false}
```

## OSC {#osc}

이 명령들이 제공하는 화면은 [OSC](../protocols/osc.md)를 참고하십시오.

### osc_send {#osc_send}

새 소켓에서 하나의 OSC 메시지를 하나의 UDP 데이터그램으로 보냅니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `target` | string | 예 | 보낼 `IP:port` 또는 `host:port`. 이름은 조회하며, IPv4 주소가 있으면 그것을 씁니다 |
| `address` | string | 예 | OSC 주소, `/mixer/fader/1`. `/`로 시작합니다 |
| `args` | [`OscArg`](#type-oscarg)`[]` | 예 | 인수. 없으면 `[]` |

**결과**: number, 보낸 바이트.

**오류**: `node.osc_address`(`/`로 시작하지 않음. 필드 `address`), `transport.target_invalid`
(포트 없음, 또는 어느 형식도 아님), `transport.dns`(이름이 확인되지 않음), `transport.*`.

### osc_monitor_start {#osc_monitor_start}

UDP 포트에서 OSC를 수신하고 모든 패킷을 디코딩합니다. 각 패킷은
[`osc://message`](events.md#event-osc-message) 이벤트로 도착합니다. *작업을 시작함*
(`osc-monitor`, `params.bind`).

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `bind` | string | 예 | 수신할 `IP:port`. `0.0.0.0:9000`은 모든 네트워크 카드, `127.0.0.1:9000`은 이 머신만 |

**결과**: [`JobInfo`](#type-jobinfo).

**오류**: `node.bind_invalid`, `transport.address_in_use`, `transport.address_unavailable`,
`transport.denied`, `wait.bind_failed`. 소켓이 더 이상 수신할 수 없으면 작업이
`wait.receive_failed`로 끝납니다.

### osc_generator_start {#osc_generator_start}

단일 인수가 파형을 따르는 OSC 메시지의 흐름을 보냅니다. 진행 상황은
[`osc://gen-tick`](events.md#event-osc-gen-tick)으로 도착합니다. *작업을 시작함*(`osc-gen`,
`params.target`, `params.address`).

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `config` | object | 예 | 아래 |

| `config`의 필드 | 형식 | 기본값 | 의미 |
| --- | --- | --- | --- |
| `target` | string | — | 보낼 `IP:port` 또는 `host:port`. 이름은 작업이 시작될 때 한 번 조회합니다 |
| `address` | string | — | OSC 주소. `/`로 시작합니다 |
| `rate` | number | — | 초당 메시지 수, 0.1~5000 사이로 유지됨 |
| `waveform` | string | — | `sine`, `triangle`, `saw`(하강: `max`에서 `min`으로, 곧바로 되돌아옴), `ramp`(상승: `min`에서 `max`로, 곧바로 되돌아옴), `square`, `random` 또는 `constant`(`max`) |
| `freq` | number | — | 초당 파형의 주기 |
| `min`, `max` | number | — | 값의 범위 |
| `as_int` | boolean | `false` | 반올림해 float 대신 int로 보냄 |
| `duration_s` | number | `0` | 이 초 수 뒤에 중지. 0이면 중지될 때까지 실행 |

**결과**: [`JobInfo`](#type-jobinfo).

**오류**: `node.osc_address`, `transport.target_invalid`, `transport.dns`, `transport.*`.
전송이 실패하면 작업이 `transport.*` 오류로 끝납니다.

```bash
invoke osc_send '{"target":"127.0.0.1:9000","address":"/cue/go","args":[{"type":"int","value":1}]}'
# 16
invoke osc_monitor_start '{"bind":"0.0.0.0:9000"}'
```

## HTTP와 쿠키 {#http}

[HTTP](../protocols/http.md)를 참고하십시오.

### http_request {#http_request}

HTTP 요청 하나를 보내고 응답을 돌려줍니다. 응답을 받지 못한 요청 — 거부, 시간 초과, 확인되지
않는 이름, 신뢰할 수 없는 인증서 — 은 명령의 오류가 **아닙니다**: 응답이 `error`와 `cause`에
말합니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `request` | [`HttpRequest`](#type-httprequest) | 예 | 요청 |
| `cookies` | boolean | 아니요 | [[ui:nav.http]] 화면의 쿠키 jar를 보내고 응답이 설정한 것을 유지. 기본값 `false` |

**결과**: [`HttpResponse`](#type-httpresponse).

**오류**: `http.client_failed`(요청을 준비조차 할 수 없었음).

### http_burst_start {#http_burst_start}

하나의 요청을 여러 번, 동시에 여러 개 보내고 측정합니다. `rate`가 없으면 각 워커가 응답이
오는 대로 다시 보내고, 있으면 응답이 아무리 느려도 요청이 정해진 일정에 따라 시작되며, 빈
워커를 기다리느라 자기 시각보다 50 ms 넘게 지연된 요청은 건너뛰고 놓친 것으로 셉니다. 진행
상황은 [`http://burst-progress`](events.md#event-http-burst-progress)로 초당 열 번 도착합니다.
*작업을 시작함*(`http-burst`, `params.method`, `params.url`, 그리고 속도를 맞출 때
`params.rate`).

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `config` | object | 예 | [`HttpRequest`](#type-httprequest)의 필드와 아래 것들을 하나의 객체로 |

| `config`의 필드 | 형식 | 기본값 | 의미 |
| --- | --- | --- | --- |
| `concurrency` | number | — | 동시에 최대 이만큼, 1~512 사이로 유지됨 |
| `total` | number | `0` | 이 요청 수 뒤에 중지. 0: 세지 않음 |
| `duration_s` | number | `0` | 이 초 수 뒤에 중지. 0: 시간 제한 없음 |
| `rate` | number | `0` | 초당 시작하는 요청, 0.1~100 000. 0: 응답이 오는 대로 |
| `cookies` | boolean | `false` | [[ui:nav.http]] 화면의 쿠키 jar 사용 |

`total`도 `duration_s`도 없으면 버스트는 중지될 때까지 실행됩니다.

**결과**: [`JobInfo`](#type-jobinfo).

**오류**: `http.rate_invalid`, `http.duration_invalid`, `http.client_failed`.

### http_cookies {#http_cookies}

[[ui:nav.http]] 화면의 쿠키 jar: 만료되지 않은 모든 쿠키. 서버에서는 모든 페이지와 스크립트에
jar 하나씩입니다. 인수가 없습니다.

**결과**: 쿠키, 각각 `name`, `value`, `domain`, `host_only`(Domain 속성이 없음: 설정한
호스트만 다시 받음), `path`, `expires`(Unix 초, 세션 쿠키면 null), `secure`, `http_only`,
`same_site`(string 또는 null).

### http_cookies_clear {#http_cookies_clear}

[[ui:nav.http]] 화면의 쿠키 jar를 비웁니다. 인수가 없습니다.

**결과**: null.

```bash
invoke http_request '{"request":{"method":"GET","url":"http://127.0.0.1:8080/health","headers":[["Accept","application/json"]],"body":null,"timeout_ms":5000}}' \
  | jq '{status, latency_ms, body}'
```

## WebSocket {#websocket}

[WebSocket](../protocols/websocket.md)을 참고하십시오. `ws_connect`가 여는 연결은 작업이며,
나머지는 `jobId`로 그것을 지목합니다.

### ws_connect {#ws_connect}

WebSocket을 열고 열린 채로 둡니다. 도착하는 것과 보내는 것은
[`ws://messages`](events.md#event-ws-messages)로 100 ms마다, 연결의 상태는
[`ws://state`](events.md#event-ws-state)로 도착합니다. *작업을 시작함*(`websocket`,
`params.url`).

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | 예 | 어디에 어떻게 연결할지 |

**결과**: [`JobInfo`](#type-jobinfo).

**오류**: `ws.url_invalid`, `ws.header_invalid`, `ws.protocol_invalid`,
`ws.handshake_status`(서버가 업그레이드에 다른 상태로 답함), `ws.subprotocol_refused`,
`ws.handshake_failed`, `transport.*`.

### ws_send {#ws_send}

열린 연결로 메시지 하나를 보냅니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `jobId` | number | 예 | 연결의 작업 |
| `message` | object | 예 | 텍스트 메시지면 `{ "text": "…" }`, 바이너리면 `{ "hex": "de ad be ef" }`. 둘 중 정확히 하나 |

**결과**: number, 보낸 바이트.

**오류**: `ws.not_connected`, `ws.payload_required`(둘 다 아니거나 둘 다), `hex.invalid`(빈
`hex`도), `node.too_long`(16 MiB 초과, 필드 `payload`. 아무것도 보내지 않고 연결은 열린 채로
남음), `ws.closed`, `transport.*`(서버가 10초 동안 읽기를 멈추면 `transport.timeout`).

### ws_close {#ws_close}

닫기 핸드셰이크로 연결을 닫고 서버의 답을 최대 2초 기다립니다. 그 뒤 작업이 끝납니다. 연결이
끝나면 그 작업은 사라지므로, 그것을 닫는 것은 `ws.not_connected`입니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `jobId` | number | 예 | 연결의 작업 |
| `code` | number | 아니요 | 1000, 또는 애플리케이션 고유의 3000~4999. 기본값 1000 |
| `reason` | string | 아니요 | 최대 123바이트. 기본값 비어 있음 |

**결과**: `{ "code", "reason", "by", "error" }` — `by`는 `client`, `server` 또는 `lost`,
`code`는 닫기가 코드를 싣지 않았으면 1005, 닫기 프레임이 없었으면 1006입니다.

**오류**: `ws.close_code`, `node.too_long`, `ws.not_connected`.

### ws_exchange {#ws_exchange}

작업 없는 교환 하나: 연결하고, 메시지가 주어지면 보내고, 요청받으면 답을 기다리고, 닫습니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | 예 | 어디에 어떻게 연결할지 |
| `message` | object | 아니요 | `{ "text" }` 또는 `{ "hex" }`, [`ws_send`](#ws_send)와 같음 |
| `expect` | object | 아니요 | 무엇을 기다릴지: `mode`(`any`, `contains`, `regex`, `hex`. 기본값 `any`), `pattern`(기본값 비어 있음), `timeout_ms`(기본값 2000) |

`expect`가 있고 `message`가 없으면, 연결한 뒤 처음 일치하는 메시지가 답이 됩니다(인사말).

**결과**: `{ "handshake", "sent", "reply", "closed" }` — `handshake`는
`{ url, peer, local, protocol, ms }`, `sent`는 보낸 바이트 또는 null, `reply`는
`{ kind, text, hex, bytes, json, ms }` 또는 null(`json`: 텍스트 답을 파싱한 것, 아니면 null.
`ms`: 보낸 뒤부터, 아무것도 보내지 않았으면 연결한 뒤부터), `closed`는
[`ws_close`](#ws_close)가 돌려주는 대로입니다.

**오류**: [`ws_connect`](#ws_connect)와 [`ws_send`](#ws_send)의 것들, `wait.timeout`(`ms`,
`unmatched`, `target`과 함께), `regex.invalid`, `hex.invalid`(파싱되지 않는 `pattern`).

```bash
invoke ws_exchange '{"config":{"url":"ws://127.0.0.1:9001/"},"message":{"text":"{\"type\":\"ping\"}"},"expect":{"mode":"contains","pattern":"pong"}}' \
  | jq .reply.text
```

## MQTT {#mqtt}

일반 TCP 위의 MQTT 3.1.1, QoS 0, 1, 2입니다. [MQTT](../protocols/mqtt.md)를 참고하십시오.

### mqtt_connect {#mqtt_connect}

브로커에 연결하고 연결을 유지합니다. 브로커가 연결을 받아들이면(CONNACK) 명령이 돌아오므로,
잘못된 비밀번호나 닫힌 포트가 그 오류입니다. 메시지는
[`mqtt://messages`](events.md#event-mqtt-messages)로 100 ms마다, 상태 변화는
[`mqtt://state`](events.md#event-mqtt-state)로, 완료된 QoS 1/2 발행과 구독 취소는
[`mqtt://ack`](events.md#event-mqtt-ack)로 도착합니다. *작업을 시작함*(`mqtt`,
`params.broker`, `params.client`).

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | 예 | 브로커와 연결 방법 |

**결과**: [`JobInfo`](#type-jobinfo).

**오류**: `mqtt.client_id_required`, `transport.*`(거부, 도달 불가, `dns`, 6초 뒤
`timeout`), `mqtt.no_answer`(6초 안에 CONNACK 없음), `mqtt.protocol`, `mqtt.refused_protocol`,
`mqtt.refused_client_id`, `mqtt.refused_unavailable`, `mqtt.refused_credentials`,
`mqtt.refused_not_authorized`, `mqtt.refused`.

### mqtt_publish {#mqtt_publish}

열린 연결로 발행합니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `jobId` | number | 예 | 연결의 작업 |
| `topic` | string | 예 | 토픽: 비어 있지 않고 `+`나 `#`가 없음 |
| `payload` | string | 예 | 페이로드, UTF-8로 전송 |
| `qos` | number | 예 | 0, 1 또는 2 (2 초과는 2로 전송) |
| `retain` | boolean | 예 | 브로커가 보관하도록 요청. `retain`과 함께 빈 페이로드를 보내면 보관된 값이 지워짐 |

**결과**: null. QoS 1 또는 2 발행은 나중에 [`mqtt://ack`](events.md#event-mqtt-ack)로
확인됩니다.

**오류**: `node.topic_wildcard`(필드 `topic`)와 `mqtt.topic_required`(필드 `topic`),
[`mqtt_publish_once`](#mqtt_publish_once)와 같으며, 이 명령은 연결을 찾기 전에 그것들을
거부합니다. `mqtt.not_connected`.

### mqtt_subscribe {#mqtt_subscribe}

열린 연결을 필터에 구독시킵니다. 브로커가 허용한 것은 `state: "subscribed"`와 함께
[`mqtt://state`](events.md#event-mqtt-state)로 도착합니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `jobId` | number | 예 | 연결의 작업 |
| `filters` | `{ filter, qos }[]` | 예 | 최소 하나. `qos` 기본값 0. `+`와 `#`는 와일드카드 |

**결과**: null.

**오류**: `mqtt.filter_required`, `mqtt.not_connected`.

### mqtt_unsubscribe {#mqtt_unsubscribe}

열린 연결을 필터에서 구독 취소합니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `jobId` | number | 예 | 연결의 작업 |
| `filters` | string[] | 예 | 최소 하나 |

**결과**: null. 브로커의 답은 `kind: "unsubscribed"`와 함께
[`mqtt://ack`](events.md#event-mqtt-ack)로 도착합니다.

**오류**: `mqtt.filter_required`, `mqtt.not_connected`.

### mqtt_publish_once {#mqtt_publish_once}

연결하고, 메시지 하나를 발행하고, QoS가 요구하는 확인을 기다리고(최대 6초), 연결을 끊습니다.
자체 연결을 자체 클라이언트 id로 엽니다(`client_id`의 처음 12자, `-o`, 숫자). 따라서 그 id의
살아 있는 연결을 브로커에서 밀어내지 않습니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | 예 | 브로커. `subscribe`는 쓰지 않음 |
| `topic` | string | 예 | 비어 있지 않고 `+`나 `#`가 없음 |
| `payload` | string | 예 | UTF-8로 전송 |
| `qos` | number | 예 | 0, 1 또는 2 (2 초과는 2로 전송) |
| `retain` | boolean | 예 | 브로커가 보관하도록 요청 |

**결과**: string, Signal Lab이 쓴 요약: `<topic> → <broker> · <bytes> B · qos<n>`, 보관되면
` retained`가 붙습니다.

**오류**: `mqtt.topic_required`, `node.topic_wildcard`, 그리고 `mqtt.client_id_required`를 뺀
[`mqtt_connect`](#mqtt_connect)의 것들: 여기서는 빈 `client_id`가 허용됩니다.

```bash
invoke mqtt_publish_once '{"config":{"host":"127.0.0.1","port":1883,"client_id":"lab"},"topic":"lab/lamp/set","payload":"ON","qos":1,"retain":false}'
# "lab/lamp/set → 127.0.0.1:1883 · 2 B · qos1"
```

## 브로드캐스트, 멀티캐스트, 디스커버리 {#broadcast}

[브로드캐스트와 디스커버리](../protocols/broadcast.md)를 참고하십시오.

::: danger
브로드캐스트와 스윕은 네트워크 세그먼트의 모든 호스트에 닿습니다. 자신이 책임지는 네트워크에서만
보내십시오.
:::

### broadcast_send {#broadcast_send}

각 대상에 데이터그램 하나를 한 번 보냅니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `config` | object | 예 | 아래 |

| `config`의 필드 | 형식 | 기본값 | 의미 |
| --- | --- | --- | --- |
| `mode` | string | — | `list`, `broadcast`, `multicast` 또는 `sweep` |
| `target` | string | — | 모드에 따라, 아래 참고 |
| `port` | number | `0` | 포트, `sweep` 전용 |
| `payload` | [`Payload`](#type-payload) | — | 각 데이터그램이 담는 것 |
| `bind` | string | 아무거나 | 보내는 로컬 `IP:port`. 비어 있거나 null이면 `0.0.0.0:0`(모든 대상이 IPv6이면 `[::]:0`) |
| `ttl` | number | `1` | IP TTL 또는 멀티캐스트 홉 제한. 1~255 |
| `multicast_loop` | boolean | `true` | 멀티캐스트가 이 머신으로도 돌아옴 |
| `rate`, `count`, `duration_s` | number | `0` | [`broadcast_beacon_start`](#broadcast_beacon_start) 전용 |

| `mode` | `target` |
| --- | --- |
| `list` | 쉼표, 세미콜론 또는 줄바꿈으로 구분한 `IP:port` 또는 `host:port` 항목(공백으로는 아님). 이름은 조회하며, IPv4 주소가 있으면 그것을 씁니다 |
| `broadcast` | `255.255.255.255:port`, 또는 `.255`로 끝나는 주소와 그 포트 |
| `multicast` | 224.0.0.0~239.255.255.255의 그룹과 그 포트 |
| `sweep` | CIDR 블록, `192.0.2.0/24`: `port`의 모든 사용 가능한 호스트. 최대 1024개 호스트이므로 `/22` 이하 |

**결과**

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `targets` | number | 목적지 |
| `packets`, `bytes` | number | 나간 것 |
| `errors` | number | 보내지 못한 데이터그램 |
| `resolved` | string[] | 처음 8개의 목적지 |
| `summary` | string | 페이로드를 한 줄로 |
| `error` | `EngineError` | 처음 실패한 데이터그램이 실패한 이유. 실패가 없으면 빠짐 |

**오류**: `broadcast.target_required`, `broadcast.not_broadcast`, `broadcast.ipv6`,
`broadcast.not_multicast`, `broadcast.sweep_port`, `broadcast.cidr_invalid`,
`broadcast.prefix_invalid`, `broadcast.sweep_too_large`, `node.osc_address`(OSC 주소는 `/`로
시작해야 함), `hex.empty`, `hex.invalid`, `node.bind_invalid`, `socket.option_failed`,
`transport.target_invalid`, `transport.dns`, 바인드 실패.

### broadcast_beacon_start {#broadcast_beacon_start}

같은 라운드(대상마다 데이터그램 하나)를 계속해서 보냅니다. 카운터는
[`broadcast://emit-stat`](events.md#event-broadcast-emit-stat)으로 250 ms마다 도착합니다. 전송이
하나도 나가지 않은 채 32번 넘게 실패하면 이유와 함께 멈춥니다. *작업을 시작함*(`beacon`,
`params.mode`, `params.target`, `params.targets`, `params.rate`).

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `config` | object | 예 | [`broadcast_send`](#broadcast_send)와 같고 아래 셋이 더해짐 |

| `config`의 필드 | 형식 | 기본값 | 의미 |
| --- | --- | --- | --- |
| `rate` | number | — | 초당 라운드. 0보다 커야 하고, 라운드 × 대상이 초당 최대 50 000 데이터그램 |
| `count` | number | `0` | 이 라운드 수 뒤에 중지. 0: 세지 않음 |
| `duration_s` | number | `0` | 이 초 수 뒤에 중지. 0: 중지될 때까지 |

**결과**: [`JobInfo`](#type-jobinfo).

**오류**: [`broadcast_send`](#broadcast_send)의 것들, `broadcast.rate_invalid`,
`broadcast.rate_limit`.

### discovery_start {#discovery_start}

UDP 포트에서 수신하고, 무언가를 보내는 모든 피어의 목록을 유지하며, 장치처럼 탐색에 답할 수
있습니다. 피어는 [`broadcast://peers`](events.md#event-broadcast-peers)로 400 ms마다
도착합니다. *작업을 시작함*(`discovery`, `params.bind`, `params.groups`, `params.joined`).

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `config` | object | 예 | 아래 |

| `config`의 필드 | 형식 | 기본값 | 의미 |
| --- | --- | --- | --- |
| `bind` | string | — | 수신할 `IP:port` |
| `groups` | string[] | `[]` | 가입할 멀티캐스트 그룹(IPv4) |
| `interface` | string | 아무거나 | 그룹에 가입할 로컬 IPv4 주소 |
| `reuse` | boolean | `true` | 이미 수신 중인 프로그램과 포트 공유(`SO_REUSEADDR`) |
| `respond` | boolean | `false` | 도착한 것에 답하기 |
| `response` | [`Payload`](#type-payload) | 없음 | 답. `respond`와 함께 필요 |
| `respond_delay_ms` | number | `0` | 답하기 전에 이만큼 기다림 |
| `match_contains` | string | 없음 | 텍스트에 이것이 들어 있는 데이터그램에만 답함 |

최대 512개의 피어가 나열되며, 그 뒤의 것은 추가되지 않습니다.

**결과**: [`JobInfo`](#type-jobinfo).

**오류**: `node.bind_invalid`, `broadcast.port_shared`(포트가 사용 중이고 `reuse`가 꺼져 있음),
`broadcast.interface_invalid`, `broadcast.not_multicast`, `broadcast.join_failed`,
`broadcast.reply_missing`, `node.osc_address`, `hex.*`, 바인드 실패. 소켓이 더 이상 수신할 수
없으면 작업이 `wait.receive_failed`로 끝납니다.

```bash
invoke broadcast_send '{"config":{"mode":"list","target":"127.0.0.1:9000, 127.0.0.1:9001","payload":{"kind":"text","text":"PING"}}}' \
  | jq '{packets, errors}'
```

## 장애 주입 {#impairment}

클라이언트와 서버 사이에서 통과하는 것을 지연, 손실, 복제, 손상, 재정렬 또는 제한하는
릴레이이며, UDP 또는 TCP 위에서 동작합니다. [장애 주입](../tools/impairment.md)을 참고하십시오.

### netsim_start {#netsim_start}

릴레이를 시작합니다: `listen`으로 도착한 것은 `target`으로 가고, 답은 같은 길로 돌아오며, 둘 다
프로필에 의해 장애를 입습니다. 카운터는 [`netsim://stat`](events.md#event-netsim-stat)으로
250 ms마다 도착합니다. *작업을 시작함*(`netsim`, `params.listen`, `params.target`, TCP면
`params.protocol`).

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `config` | object | 예 | 아래 |

| `config`의 필드 | 형식 | 기본값 | 의미 |
| --- | --- | --- | --- |
| `listen` | string | — | 릴레이가 수신할 `IP:port`. 클라이언트를 여기로 가리키십시오 |
| `target` | string | — | 실제 서버의 `IP:port`, 또는 `host:port` — 호스트 이름은 릴레이가 시작될 때 한 번 조회합니다 |
| `profile` | [`ImpairProfile`](#type-impairprofile) | — | 트래픽에 할 일 |
| `seed` | number | 새 것 | 추첨의 시드: 같은 시드와 같은 트래픽은 같은 손실을 냅니다 |
| `protocol` | string | `udp` | `udp`(데이터그램) 또는 `tcp`(스트림) |

**결과**: [`JobInfo`](#type-jobinfo).

**오류**: `node.range`(프로필의 값이 범위 밖, `min`, `max`와 필드와 함께), `node.too_long`,
`node.bind_invalid`, `transport.target_invalid`, `transport.dns`(찾을 수 없는 대상 이름),
바인드 실패. 소켓이 더 이상 수신할 수 없으면 작업이 `wait.receive_failed`로 끝납니다.

### netsim_set_profile {#netsim_set_profile}

실행 중인 릴레이가 이제부터 다른 프로필로 장애를 줍니다. 소켓은 닫지 않습니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `jobId` | number | 예 | 릴레이의 작업 |
| `profile` | [`ImpairProfile`](#type-impairprofile) | 예 | 새 프로필 |

**결과**: null.

**오류**: `netsim.not_running`, `node.range`, `node.too_long`.

```bash
invoke netsim_start '{"config":{"listen":"127.0.0.1:9010","target":"127.0.0.1:9000","profile":{"latency_ms":80,"jitter_ms":20,"loss":0.02}}}'
```

## 스톰과 스캐너 {#storm-scanner}

::: danger
스톰은 요청한 만큼 대상에 부하를 주고, 스캔은 범위의 모든 포트를 탐색합니다. 자신이 책임지는
호스트에만 겨누십시오.
:::

### storm_start {#storm_start}

하나의 대상에 UDP 데이터그램 또는 TCP 연결의 꾸준한 부하를 보냅니다. 카운터는
[`storm://stat`](events.md#event-storm-stat)으로 250 ms마다 도착합니다. *작업을 시작함*
(`storm`, `params.protocol`, `params.target`, `params.rate`).

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `config` | object | 예 | 아래 |

| `config`의 필드 | 형식 | 기본값 | 의미 |
| --- | --- | --- | --- |
| `target` | string | — | `IP:port` 또는 `host:port`. 이름은 작업이 시작될 때 한 번 조회합니다 |
| `protocol` | string | — | `udp`: 데이터그램. `tcp`: 단위마다 연결 하나가 페이로드를 쓰고 닫음(연결마다 최대 500 ms) |
| `size` | number | — | 페이로드 바이트, 1~65 507 사이로 유지됨 |
| `rate` | number | — | 초당 단위, 일정에 따라: 단위 *n*은 시작 후 *n* / `rate`초에 예정되고, 깨어날 때마다 예정된 것을 보냅니다(최대 256. 일정이 더 뒤처지면 오래된 단위를 건너뜀). 0이면 최대한 빠르게 보냄 |
| `duration_s` | number | `0` | 이 초 수 뒤에 중지. 0: 중지될 때까지 |

**결과**: [`JobInfo`](#type-jobinfo).

**오류**: `transport.target_invalid`, `transport.dns`. 실패한 전송은 이벤트에서 세어지며 오류로
보고되지 않습니다.

### scan_start {#scan_start}

범위의 모든 포트에 TCP 연결을 시도하고 열린 것을 보고하며, 요청하면 서비스가 처음 말하는 것도
함께 보고합니다. 열린 포트는 [`scan://open`](events.md#event-scan-open)으로, 진행 상황은
[`scan://progress`](events.md#event-scan-progress)로 도착합니다. *작업을 시작함*(`scan`,
`params.host`, `params.from`, `params.to`).

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `config` | object | 예 | 아래 |

| `config`의 필드 | 형식 | 기본값 | 의미 |
| --- | --- | --- | --- |
| `host` | string | — | 호스트 이름 또는 주소 |
| `port_start`, `port_end` | number | — | 범위, 양끝 포함. 거꾸로 주면 서로 바뀝니다 |
| `concurrency` | number | `256` | 동시 시도, 1~1024 |
| `timeout_ms` | number | `600` | 포트마다, 50~10 000 |
| `grab_banner` | boolean | `false` | 연결 후 400 ms 안에 서비스가 보내는 것을 최대 256바이트 읽음 |

**결과**: [`JobInfo`](#type-jobinfo).

**오류**: `scan.host_required`.

```bash
invoke scan_start '{"config":{"host":"127.0.0.1","port_start":8000,"port_end":9100,"grab_banner":true}}'
```

## 인스펙터 {#inspector}

인스펙터는 캡처가 켜져 있는 동안 도구가 보내고 받는 것을 프레임으로 기록합니다. 서버에서는 모든
페이지와 스크립트에 인스펙터 하나씩입니다. [인스펙터](../tools/inspector.md)를 참고하십시오.

### inspect_set_enabled {#inspect_set_enabled}

캡처를 켜거나 끕니다. 꺼져 있는 동안에는 아무것도 기록되지 않습니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `enabled` | boolean | 예 | 켜기(`true`) 또는 끄기 |

**결과**: [`CaptureStats`](#type-frame).

### inspect_stats {#inspect_stats}

캡처의 카운터. 인수가 없습니다.

**결과**: [`CaptureStats`](#type-frame).

### inspect_snapshot {#inspect_snapshot}

가장 최신 프레임, 오래된 것부터.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `limit` | number | 예 | 몇 개인지, 1~8192 |

**결과**: [`Frame`](#type-frame)`[]`, 바이트는 없음([`inspect_payload`](#inspect_payload) 참고).

### inspect_clear {#inspect_clear}

캡처와 그 카운터를 비웁니다. 인수가 없습니다.

**결과**: [`CaptureStats`](#type-frame).

### inspect_export {#inspect_export}

보관 중인 모든 프레임을 데이터 폴더의 `capture-<ms>.jsonl` 또는 `capture-<ms>.txt`에 씁니다.
`jsonl`에서는 각 줄이 프레임이고 보관한 바이트를 `data`에 base64로 담습니다. `txt`는 읽기용이며
각 프레임의 hex 덤프가 있습니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `format` | string | 예 | `txt`. 그 밖의 것은 `jsonl`을 씁니다 |

**결과**: string, 쓴 경로.

**오류**: `inspect.empty`, `file.io`.

### inspect_payload {#inspect_payload}

프레임이 보관한 바이트로, 배치가 실어 온 1 KiB 미리보기 이후의 것입니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `seq` | number | 예 | 프레임의 번호 |

**결과**: `{ "seq", "bytes", "kept", "dump", "hex" }` — `bytes`는 프레임의 크기, `kept`는 그중
보관된 수(최대 256 KiB), `dump`는 모든 행을 `offset  hex  |ascii|`로, `hex`는 재생이 보내는
순수 hex입니다.

**오류**: `inspect.frame_gone`(새 프레임이 그 자리를 차지함), `inspect.no_payload`(크기만
기록됨).

```bash
invoke inspect_set_enabled '{"enabled":true}'
invoke inspect_snapshot '{"limit":20}' | jq '.[] | {seq, proto, dir, summary}'
```

## 신호 라이브러리 {#signals}

라이브러리는 데이터 폴더의 `signals.json`입니다. 저장소일 뿐입니다: 신호는 그 전송 방식의
명령(`osc_send`, `broadcast_send`, `http_request`, `mqtt_publish` 또는 `mqtt_publish_once`)으로
보냅니다. [신호](../tools/signals.md)와 [파일](../reference/files.md#signals-json)을
참고하십시오.

### signals_load {#signals_load}

라이브러리를 읽습니다. 파일이 없으면 기본 제공 세트를 먼저 씁니다. 인수가 없습니다.

**결과**: `{ "path": string, "library": library, "seeded": boolean }` — `seeded`는 기본 제공
세트를 방금 썼을 때 true입니다. 라이브러리는
`{ "version", "signals": [...], "folders": [...] }`입니다: `version`은 2이고(버전 1 파일은
그대로 돌아옴), `folders`는 없으면 빠집니다. 각 신호에는 `id`, `name`, `group`(그 폴더,
`"A/B"`. 없으면 빈 값), `note`, `body`가 있습니다.

**오류**: `signals.json_invalid`(`path`, `line`, `column`과 함께. 파일은 결코 대체되지 않음),
`file.io`.

### signals_save {#signals_save}

같은 폴더의 임시 파일을 거쳐 라이브러리 파일 전체를 대체합니다. 존재하지만 라이브러리로 읽히지
않는 파일은 그대로 둡니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `library` | object | 예 | `signals_load`가 돌려주는 대로의 `{ version, signals, folders }` |

**결과**: string, 쓴 경로.

**오류**: `signals.json_invalid`(디스크의 파일이 읽히지 않음, `path`, `line`, `column`과 함께.
아무것도 쓰지 않음), `signals.encode`, `file.io`.

신호의 `body`는 `transport`에 따라:

| `transport` | 필드 |
| --- | --- |
| `osc` | `target`, `address`, `args` ([`OscArg`](#type-oscarg)`[]`) |
| `udp` | `target`, `payload`: `{ "kind": "text", "text" }` 또는 `{ "kind": "hex", "hex" }` |
| `http` | `request` ([`HttpRequest`](#type-httprequest)) |
| `mqtt` | `broker`(`host:port`), `topic`, `payload`, `qos`, `retain` |

```bash
invoke signals_load | jq '.library.signals[] | {name, transport: .body.transport}'
```

## 에뮬레이터 {#emulators}

에뮬레이터는 Signal Lab이 상대편 역할을 맡는 것입니다: HTTP API, OSC, UDP 또는 TCP 장치, MQTT
브로커입니다. 그 문서(`name`, `bind`, `protocol`, 프로토콜의 규칙과 선택적 `outage`)는
[에뮬레이터](../tools/emulators.md)에 설명되어 있습니다. 라이브러리는 데이터 폴더의
`emulators.json`입니다.

### emulators_load {#emulators_load}

에뮬레이터 라이브러리를 읽습니다. 파일이 없으면 기본 제공 세트를 먼저 씁니다. 인수가 없습니다.

**결과**: `{ "path", "library": { "version": 1, "emulators": [{ "id", "note", "emulator" }] }, "seeded" }`.

**오류**: `emulators.json_invalid`(`path`, `line`, `column`과 함께. 결코 대체되지 않음),
`file.io`.

### emulators_save {#emulators_save}

임시 파일을 거쳐 에뮬레이터 라이브러리 전체를 대체합니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `library` | object | 예 | `emulators_load`가 돌려주는 대로의 `{ version, emulators }` |

**결과**: string, 쓴 경로.

**오류**: `emulators.encode`, `file.io`.

### emulator_check {#emulator_check}

에뮬레이터가 시작될지 여부: [`emulator_start`](#emulator_start)가 바인드 전에 검사하는 모든
것.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `emulator` | object | 예 | 에뮬레이터 문서 |
| `params` | object of strings | 아니요 | 템플릿이 매개변수로 읽는 값 |

**결과**: 시작될 것이면 null.

**오류**: `emulator.*`, 그리고 필드가 없거나, 범위 밖이거나, 너무 길거나, 잘못된 형식일 때의
`node.*`(`node.required`, `node.range`, `node.too_long`, `node.bind_invalid`,
`node.target_invalid`, `node.method_invalid`…). `param.unknown`, `template.*`,
`osc.pattern_*`, `regex.invalid`, `hex.invalid`. 문제가 그중 하나에 있으면 각각 `rule`,
`retained` 또는 `response`를 `params`에 담습니다.

### emulator_start {#emulator_start}

에뮬레이터를 자체 작업으로 시작합니다. 명령이 돌아올 때 소켓이 열려 있습니다. 받고 응답한 것은
무언가 바뀌었을 때 [`emulator://activity`](events.md#event-emulator-activity)로 200 ms마다
도착합니다. *작업을 시작함*(`emulator`, `params.name`, `params.protocol`, `params.local`,
`source`가 주어졌으면 `params.source`).

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `emulator` | object | 예 | 에뮬레이터 문서 |
| `params` | object of strings | 아니요 | 템플릿이 매개변수로 읽는 값 |
| `seed` | number | 아니요 | 시드, 0~9007199254740991. 기본값: 새 것 |
| `source` | string | 아니요 | 그것이 나온 라이브러리 항목. 작업에 `params.source`로 남음 |

**결과**: [`JobInfo`](#type-jobinfo).

**오류**: [`emulator_check`](#emulator_check)의 것들, `seed.range`,
`transport.address_in_use`와 다른 바인드 실패.

### emulator_exchanges {#emulator_exchanges}

실행 중인 에뮬레이터가 받고 응답한 것. 마지막 500개 교환을 보관합니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `jobId` | number | 예 | 에뮬레이터의 작업 |
| `after` | number | 아니요 | 이보다 큰 번호의 교환만. 기본값 0 |
| `limit` | number | 아니요 | 최대 이만큼, 1~500. 기본값 500 |

**결과**

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 작업 |
| `name`, `protocol`, `local` | string | 에뮬레이터, 그 프로토콜, 수신 주소 |
| `counts` | object | `total`, `unmatched`(규칙이 받지 않음), `failed`, `down`(다운 중 도착: 다운 일정 또는 [`emulator_down`](#emulator_down)), `hits`(규칙별), `missed`(MQTT: 너무 뒤처진 클라이언트가 받지 못한 메시지. 0이면 빠짐) |
| `forced` | string | 다운된 동안 `unavailable`, `reset` 또는 `timeout`. 아니면 빠짐 |
| `exchanges` | object[] | 각각: `seq`, `ts`, `from`, `request`, `rule`(1부터. 받은 규칙이 없으면 빠짐), `reply`, `status`, `fault`, `ms`, `error`, `frame`, `down`, `data`(템플릿이 읽는 대로의 요청) |

**오류**: `emulator.not_running`.

### emulator_down {#emulator_down}

실행 중인 에뮬레이터를 다시 올릴 때까지 다운시키거나, 다운 일정이 무엇을 말하든 되돌립니다.
다운 중에는 HTTP 에뮬레이터가 각 요청을 `fault`로 맞이하고, TCP 장치와 MQTT 브로커는 연결을
끊고 새 연결을 거부하며, OSC와 UDP 장치는 아무것도 응답하지 않습니다.

| 인수 | 형식 | 필수 | 의미 |
| --- | --- | --- | --- |
| `jobId` | number | 예 | 에뮬레이터의 작업 |
| `down` | boolean | 예 | 다운(`true`) 또는 올리기 |
| `fault` | string | 아니요 | HTTP 요청이 맞는 것: `unavailable`(503, `Retry-After` 없이: 언제 돌아올지 알 수 없음), `reset`(연결이 닫힘), `timeout`(응답 없음). 기본값 `unavailable` |

**결과**: null.

**오류**: `emulator.not_running`.

```bash
invoke emulator_exchanges '{"jobId":5,"after":0}' | jq '.counts, (.exchanges[] | {request, rule, status})'
```

## 공용 타입 {#types}

### JobInfo {#type-jobinfo}

작업을 시작하는 명령이 돌려주고 [`jobs_list`](#jobs_list)가 나열하는 것: `id`, `kind`,
`label`(영어, 로그용), `params`(레이블이 부르는 값들. 없으면 빠짐), `started_ms`.
[작업](index.md#jobs)을 참고하십시오.

### OscArg {#type-oscarg}

OSC 인수 하나, 그 타입과 값:

| `type` | `value` | OSC 태그 |
| --- | --- | --- |
| `int` | 32비트 정수 | `i` |
| `float` | 숫자, 32비트 float로 전송 | `f` |
| `str` | string | `s` |
| `long` | 64비트 정수 | `h` |
| `double` | 숫자, 64비트 | `d` |
| `bool` | `true` 또는 `false` | `T` 또는 `F` |
| `blob` | 바이트 배열, `[222, 173]` | `b` |
| `nil` | 없음: `{ "type": "nil" }` | `N` |

### HttpRequest {#type-httprequest}

| 필드 | 형식 | 기본값 | 의미 |
| --- | --- | --- | --- |
| `method` | string | — | `GET`, `POST`… |
| `url` | string | — | `http://` 또는 `https://` |
| `headers` | `[name, value][]` | `[]` | 요청 헤더 |
| `body` | string or null | null | 본문 |
| `timeout_ms` | number | `10000` | 교환 전체에 |
| `auth` | object | 없음 | `{ "scheme": "basic", "username", "password" }`, `{ "scheme": "digest", "username", "password" }` 또는 `{ "scheme": "bearer", "token" }` |

최대 10번의 리다이렉트를 따릅니다. 한 호스트에 입력한 자격 증명과 쿠키는 다른 곳으로 결코
가지 않습니다. Digest 요청은 서버의 401 챌린지에 답하고 다시 보냅니다.

### HttpResponse {#type-httpresponse}

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `ok` | boolean | 2xx 상태 |
| `status`, `status_text` | number, string | 상태. 응답이 없으면 0과 빈 문자열 |
| `latency_ms` | number | 본문 전체가 도착할 때까지 |
| `headers` | `[name, value][]` | 응답 헤더 |
| `body` | string | 본문을 텍스트로, 최대 256 KiB |
| `body_bytes` | number | 본문의 전체 크기 |
| `truncated` | boolean | `body`가 256 KiB에서 잘림 |
| `error` | string or null | 응답이 없었던 이유, 원인의 모든 층 |
| `cause` | string or null | 실패의 종류: `refused`, `timeout`, `dns`, `unreachable`, `reset`, `address_in_use`, `address_unavailable`, `denied`, `tls`, `target_invalid`, `failed` — `transport.*` 코드와 같음 |
| `digest` | object | 401을 만난 Digest 요청에만. 아니면 빠짐. `challenged`: 챌린지에 답하고 요청을 다시 보냈음. `error`: 그럴 수 없었던 이유, `EngineError`(`http.digest_not_offered`, `http.digest_unsupported`, `http.digest_invalid`, `http.digest_other_origin`) 또는 null |

### Payload {#type-payload}

브로드캐스트 또는 디스커버리 데이터그램이 담는 것:
`{ "kind": "osc", "address", "args" }`, `{ "kind": "text", "text" }`(그대로 전송, 종료 0 없음)
또는 `{ "kind": "hex", "hex" }`(`de ad be ef`, `deadbeef`, `0xDE,0xAD` — hex 숫자가 아닌 것은
무시됩니다).

### MqttConfig {#type-mqttconfig}

| 필드 | 형식 | 기본값 | 의미 |
| --- | --- | --- | --- |
| `host` | string | — | 브로커의 이름 또는 주소 |
| `port` | number | — | 보통 1883 |
| `client_id` | string | — | 비어 있지 않음. 같은 id의 다른 연결은 브로커가 밀어냄 |
| `username`, `password` | string | 비어 있음 | `username`은 비어 있지 않을 때 전송됨. `password`는 `username`과 함께일 때만 |
| `keep_alive_s` | number | `60` | 핑은 그 절반마다. 0: 없음 |
| `clean_session` | boolean | `true` | CONNECT 플래그 |
| `will` | object or null | null | `{ topic, payload, qos, retain }`, 연결이 끊기면 브로커가 발행 |
| `subscribe` | `{ filter, qos }[]` | `[]` | 연결이 서면 곧바로 구독됨 |

### WsConfig {#type-wsconfig}

| 필드 | 형식 | 기본값 | 의미 |
| --- | --- | --- | --- |
| `url` | string | — | `ws://` 또는 `wss://`(`wss://`는 시스템이 HTTPS에 대해 신뢰하는 것을 신뢰) |
| `headers` | `[name, value][]` | `[]` | 업그레이드 요청과 함께 전송 |
| `protocols` | string[] | `[]` | 제안할 서브프로토콜, 선호 순 |
| `timeout_ms` | number | `10000` | 연결, TLS, 업그레이드를 합쳐 |

메시지는 양방향 모두 최대 16 MiB입니다.

### ImpairProfile {#type-impairprofile}

모든 필드는 선택적이며, 빠뜨린 것은 아무것도 하지 않습니다. 확률은 0~1입니다.

| 필드 | 범위 | 의미 | UDP | TCP |
| --- | --- | --- | --- | --- |
| `name` | 최대 60자 | 타임라인과 보고서용 레이블 | 예 | 예 |
| `latency_ms` | 0~60 000 | 모든 것에 더하는 지연 | 예 | 예 |
| `jitter_ms` | 0~60 000 | 매번 추첨하는 최대 이만큼의 추가 | 예 | 예 |
| `loss` | 0~1 | 데이터그램이 버려짐 | 예 | — |
| `duplicate` | 0~1 | 데이터그램이 두 번 전송됨 | 예 | — |
| `corrupt` | 0~1 | 데이터그램의 한 비트가 뒤집힘 | 예 | — |
| `reorder` | 0~1 | 데이터그램이 붙잡혀 뒤의 것이 앞지름 | 예 | — |
| `rate_kbps` | 0, 또는 8~10 000 000 | 대역폭 제한, 초당 킬로비트. 0: 없음 | 예 | 예 |
| `burst_start` | 0~1 | 데이터그램이 손실 버스트를 시작함 | 예 | — |
| `burst_length` | 1~1000 | 버스트가 평균적으로 지속되는 데이터그램 수(`burst_start`와 함께 필요) | 예 | — |
| `offline` | `true` 또는 `false` | 아무것도 통과하지 못함 | 예 | 예 |
| `reset` | 0~1 | 스트림의 청크가 연결을 리셋함 | — | 예 |
| `stall` | 0~1 | 스트림의 청크가 연결을 반쯤 열린 채로 둠 | — | 예 |

### Frame과 CaptureStats {#type-frame}

`Frame`은 캡처한 패킷, 요청 또는 메시지 하나입니다:

| 필드 | 의미 |
| --- | --- |
| `seq` | 번호, 증가함 |
| `ts` | 시각, 1970년 이후 밀리초 |
| `proto` | `osc`, `udp`, `tcp`, `http`, `mqtt`, `ws`… |
| `dir` | `tx`(보냄) 또는 `rx`(받음) |
| `source` | 캡처한 도구: `osc-monitor`, `broadcast`, `netsim`… |
| `job_id` | 그 작업, 또는 null |
| `local`, `remote` | 주소: 이쪽과 상대편의 `IP:port`(HTTP, WebSocket, MQTT에서는 URL 또는 브로커). 릴레이된 프레임의 `local`은 릴레이가 수신하는 주소이고 `remote`는 프레임이 가던 곳이며, 그 다리는 `verdict`를 끝맺습니다(`· client→target`, `· target→client`) |
| `bytes` | 크기 |
| `summary` | 한 줄 |
| `detail` | 여러 줄에 걸친 디코딩, 또는 null |
| `hex` | 처음 1 KiB의 hex 덤프, 또는 null |
| `verdict` | 그것이 어떻게 되었는지 — `dropped`, `sampled`, 상태 — 또는 null |
| `kept` | `bytes` 중 보관된 수(최대 256 KiB). 크기만 기록되면 0 |
| `publish` | MQTT 발행에만: `{ broker, topic, qos, retain, text }` — 브로커는 `host:port`, 그리고 보관된 바이트(메시지의 페이로드)가 UTF-8 텍스트인지. 다른 모든 프레임에는 없음 |

`CaptureStats`: `enabled`, `total`(기록된 프레임), `bytes`, `skipped`(기록되었지만 인터페이스로
결코 보내지지 않음), `buffered`(보관 중인 프레임), `capacity`(8192), `held`(보관 중인 페이로드
바이트), `held_limit`(64 MiB). 어느 한도를 넘으면 가장 오래된 프레임이 자리를 내줍니다.

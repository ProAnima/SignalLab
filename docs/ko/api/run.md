---
title: 실행
description: POST /api/run은 Signal Lab 서버에서 실험을 실행하고 그 결과로 응답하거나, 단계를 NDJSON 줄로 스트리밍합니다.
---

# 실험 실행하기

스크립트나 파이프라인에서 서버의 실험을 실행하고 어떻게 되었는지 알려면 `POST /api/run`으로
보내십시오. 서버는 실험을 끝까지 실행하고 결과로 응답합니다. 원하면 일어나는 대로 각 단계로
응답하기도 합니다. [`signallab run --server`](../automation/cli.md#cli-run)가 이것을
사용합니다.

편집기의 [[ui:exp.run]] 및 [`experiment_start`](commands.md#experiment_start)와 같은
실행입니다. 열려 있는 모든 페이지가 보고 중지할 수 있는 작업이며, 같은 이벤트, 데이터 폴더의
같은 보고서입니다.

## 요청 {#request}

```http
POST /api/run
Authorization: Bearer <token>
Content-Type: application/json

{ "template": "osc-ping-reply", "overrides": { "device": "192.0.2.20:9000" }, "seed": 42, "timeout": 30 }
```

본문은 실험 하나를 지정하고(`document` 또는 `template`, 둘 다는 아님) 무엇으로 실행할지
지정합니다.

| 필드 | 형식 | 기본값 | 의미 |
| --- | --- | --- | --- |
| `document` | object | — | 편집기가 저장하고 내보내는 형태의 실험. 파일을 열 때처럼 예전 버전은 마이그레이션됨 |
| `template` | string | — | 파일 이름으로 지정하는 기본 제공 템플릿, `.json`이 있어도 없어도 됨(아래) |
| `overrides` | object | `{}` | 이 실행에만 적용되는 매개변수 값. 값은 문자열, 숫자 또는 boolean일 수 있으며, 각각 실험의 매개변수여야 함 |
| `profile` | string | 문서의 값 | 이 프로필로 실행. `""`는 기본값으로 실행 |
| `seed` | number | 문서의 값, 없으면 새 값 | 0~9007199254740991. 같은 시드는 같은 임의 값을 뽑음 |
| `timeout` | number | `300` | 실행이 `run.timeout`으로 실패하기까지의 초. 1~300 |

서버가 모르는 필드는 거부됩니다(`400`, `api.run_invalid`). 문서 자체는 파일을 열 때와 같은
방식으로 읽히므로 최대 4 MiB입니다.

기본 제공 템플릿 — 편집기가 [[ui:exp.templates]] 아래에서 제공하는 실험입니다.

| `template` | 내용 |
| --- | --- |
| `empty` | 시작과 끝 |
| `http-check` | `http://127.0.0.1:8080/`에 대한 GET, 그다음 상태 200 검사 |
| `status-branch` | `http://127.0.0.1:8080/`에 대한 GET. 200이면 OSC 메시지, 아니면 500 ms 지연 |
| `parallel-flows` | `http://127.0.0.1:8080/`에 대한 GET과 로그 항목으로 나뉘는 [[ui:exp.node.fork]], 나란히, 그다음 [[ui:exp.node.join]] |
| `osc-ping-reply` | 매개변수 `device`(`127.0.0.1:9000`)로 보내는 OSC `/ping`, 그다음 `127.0.0.1:9001`에서 `/pong`을 2초 동안 대기 |
| `poll-until-ready` | `device`에게 OSC로 `/status`를 물어 `ready`라고 답할 때까지 반복하는 [[ui:exp.node.loop]], 최대 10번 |
| `flaky-api` | 제대로 동작하기 전에 실패하는 에뮬레이션된 API(매개변수 `api`), 200이라고 답할 때까지 [[ui:exp.node.loop]]에서 물어봄 |
| `fault-phases` | 장애 릴레이 뒤의 UDP 장치, 릴레이가 깨끗함, 손실 있음, 오프라인, 다시 깨끗함으로 변하는 동안 8초 동안 전송 |
| `dependency-outage` | 에뮬레이션된 API(매개변수 `api`)를 2초 동안 다운시키고, [[ui:exp.node.loop]]가 다시 200이라고 답할 때까지 물어봄 |
| `websocket-echo` | 매개변수 `service`(`ws://127.0.0.1:9001/echo`)에 대한 연결, 메시지, 메아리 대기, 검사, 닫기 |

[[ui:exp.templates]] 아래에서 하나를 열면 그 노드와 매개변수를 볼 수 있습니다.
[실험](../experiments/index.md)을 참고하십시오.

## 결과 {#result}

기본적으로 응답은 `Content-Type: application/json`과 함께 `200`이며, 실행이 끝났을 때
전송됩니다. 실행의 결과인 JSON 객체 하나입니다.

```json
{
  "job_id": 12,
  "experiment": "OSC ping → reply",
  "outcome": "passed",
  "seed": 42,
  "profile": null,
  "overridden": true,
  "params": { "device": "192.0.2.20:9000" },
  "started_ms": 1759600000000,
  "ended_ms": 1759600000310,
  "steps": [ { "job_id": 12, "ts": 1759600000001, "node_id": "start", "state": "running", "detail": "", "message_key": null, "message_params": null }, … ],
  "report_path": "/data/runs/run-1759600000000-12.json"
}
```

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 실행의 작업 |
| `experiment` | string | 실험의 이름 |
| `outcome` | string | `passed`, `failed` 또는 `stopped` |
| `seed` | number | 실행한 시드. 같은 값을 뽑으려면 `seed`로 다시 주십시오 |
| `profile` | string 또는 null | 실행한 프로필 |
| `overridden` | boolean | 일부 값이 `overrides`에서 왔음 |
| `params` | object | 실행이 사용한 모든 매개변수 값 |
| `started_ms`, `ended_ms` | number | 1970년 이후 밀리초 |
| `error` | `EngineError` | 실패한 이유: 첫 실패. 통과했으면 빠짐 |
| `steps` | object[] | 일어난 순서대로의 모든 단계, [`experiment://step`](events.md#event-experiment-step)처럼 |
| `emulators` | object[] | 각 [[ui:exp.node.emulator]] 노드가 받고 응답한 것: `node`, `name`, `protocol`, `local`, `counts`. 없으면 빠짐 |
| `impairments` | object[] | 각 [[ui:exp.node.impairment]] 노드의 릴레이가 국면별로 한 일. 없으면 빠짐 |
| `report_path` | string | 서버에서의 실행 보고서. [`/api/files`](index.md#files)로 다운로드. 쓰이지 않았으면 빠짐 |
| `report_error` | `EngineError` | 보고서를 쓰지 못한 이유. 아니면 빠짐 |

시크릿 값은 이 모두에서 가려집니다. 보고서 파일은 같은 단계를 담습니다.
[실행과 보고서](../experiments/runs.md)를 참고하십시오.

실행이 진행되는 동안 서버는 15초마다 공백을 보냅니다. JSON은 값 앞의 공백을 무시하므로 결과는
여전히 파싱되고, 프록시는 길고 조용한 실행을 죽은 연결로 보지 않습니다.

## 단계 따라가기 {#lines}

단계가 일어나는 대로 보려면 NDJSON을 요청하십시오.

```http
Accept: application/x-ndjson
```

응답은 `Content-Type: application/x-ndjson`과 함께 `200`입니다. 줄마다 JSON 객체 하나이며,
각각 `type`이 있습니다.

| `type` | 언제 | 줄의 나머지 |
| --- | --- | --- |
| `started` | 처음에 한 번 | `job_id`, `experiment`, `seed`, `profile`, `overridden`, `started_ms` |
| `step` | 각 단계 | 그 단계, [`experiment://step`](events.md#event-experiment-step)처럼 |
| `heartbeat` | 15초마다 | 없음 |
| `ended` | 마지막에 한 번 | 결과, 위와 같음 |

```text
{"job_id":12,"experiment":"OSC ping → reply","seed":42,"profile":null,"overridden":true,"started_ms":1759600000000,"type":"started"}
{"job_id":12,"ts":1759600000001,"node_id":"start","state":"running","detail":"","message_key":null,"message_params":null,"type":"step"}
…
{"job_id":12,"experiment":"OSC ping → reply","outcome":"passed",…,"type":"ended"}
```

`ended`가 나올 때까지 줄을 읽으십시오. 모르는 `type`은 무시하십시오. 서버는
`X-Accel-Buffering: no`를 보내므로 nginx 프록시가 각 줄을 곧바로 전달합니다.

## 상태와 결과 {#status}

HTTP 오류 상태는 **실행이 시작되지 않았음**을 뜻합니다. 본문은
[`EngineError`](index.md#errors)입니다.

| 상태 | 코드 | 이유 |
| --- | --- | --- |
| `400` | `api.run_invalid` | 본문이 실행 요청이 아님: JSON이 아니거나, 알 수 없는 필드이거나, 문자열·숫자·boolean이 아닌 override |
| `400` | `api.run_source` | `document`도 `template`도 없거나, 둘 다 있음 |
| `415` | `command.json_required` | `Content-Type: application/json`이 아님 |
| `422` | `api.template_unknown` | 그 이름의 기본 제공 템플릿이 없음 |
| `422` | `file.json_invalid`, `file.too_large`, `doc.*` | 문서를 읽을 수 없음 |
| `422` | 아무 검증 코드, `run.override_unknown`, `profile.active_missing`, `run.limit_range`, `seed.range`, `secret.missing`, `transport.address_in_use`… | 실험이 시작될 수 없음: 검증을 통과하지 못하거나, 값이 범위를 벗어나거나, 시크릿이 저장되어 있지 않거나, 수신할 포트가 이미 사용 중임 |

실행이 시작된 뒤에는 무슨 일이 있어도 상태가 `200`입니다. 결과의 `outcome`을 읽으십시오.

| `outcome` | 의미 |
| --- | --- |
| `passed` | 모든 단계가 통과하고 [[ui:exp.node.end]]에 도달함 |
| `failed` | 단계가 실패했거나, 실행이 `timeout`보다 오래 걸림(`run.timeout`). `error`가 어느 쪽인지와 이유를 말함 |
| `stopped` | 끝나기 전에 중지됨: `job_stop`, [[ui:app.stopAll]], 또는 서버 종료로. `steps`에는 도달한 단계들이 있음. 보고서는 저장되지 않음 |

## 사라지는 클라이언트 {#disconnect}

연결을 닫아도 실행은 중지되지 않습니다. 서버의 작업이기 때문입니다. 브라우저에서 시작한
실행이 탭을 닫아도 그런 것처럼, 끝까지 실행되고 보고서를 저장합니다.
[`jobs_list`](commands.md#jobs_list)로 찾고, [`job_stop`](commands.md#job_stop)으로 중지하고,
나중에 [`experiment_runs`](commands.md#experiment_runs)로 보고서를 읽으십시오. 서버가
종료되면 실행이 중지되고, 아직 연결되어 있는 클라이언트는 `"outcome": "stopped"`를 받습니다.

## 예제 {#examples}

기본 제공 템플릿을 실행하고 결과를 기다립니다.

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
curl -sS -X POST "$SERVER/api/run" \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"template":"osc-ping-reply","overrides":{"device":"192.0.2.20:9000"},"timeout":30}' \
  | jq -r .outcome
```

매개변수를 바꾼 자신의 실험을 보내고, 일어나는 대로 각 단계를 출력합니다.

```bash
jq '{document: ., overrides: {api: "http://192.0.2.10:8080"}, profile: ""}' smoke.json |
  curl -sSN -X POST "$SERVER/api/run" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    -H "Accept: application/x-ndjson" --data @- |
  jq -r 'select(.type == "step") | "\(.node_id)  \(.state)  \(.detail)"'
```

`-N`은 `curl`이 줄을 붙잡아 두지 않게 합니다. 실패한 실행으로 파이프라인을 실패시키려면
`outcome`을 검사하십시오.

```bash
outcome=$(curl -sS -X POST "$SERVER/api/run" -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" -d '{"template":"http-check"}' | jq -r .outcome)
[ "$outcome" = "passed" ]
```

## 명령줄에서 {#cli}

`--server <url>`과 함께 쓰는 [`signallab run`](../automation/cli.md#cli-run)은 이
엔드포인트를 통해 서버에서 실행합니다. 읽은 실험을 `overrides`, `seed`, `timeout`과 함께
보내고, NDJSON을 요청하며, 줄이 도착하는 대로 각 단계를 출력합니다. 토큰은 `--token-file`에서
오고, 없으면 `SIGNALLAB_TOKEN`입니다. `--report`를 쓰면 `/api/files`를 통해 보고서를
다운로드합니다. 60초 동안 아무것도 보내지 않는 서버(하트비트조차 없이)는 사라진 것으로
봅니다.

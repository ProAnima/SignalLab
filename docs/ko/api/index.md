---
title: HTTP API
description: 스크립트, CI 및 다른 도구에서 Signal Lab 서버 자체의 인터페이스가 쓰는 명령과 이벤트로 Signal Lab 서버를 제어합니다.
---

# HTTP API

스크립트, CI 파이프라인 또는 다른 도구에서 Signal Lab을 제어하려면 HTTP로 Signal Lab
서버(`signal-lab-server` 또는 도커 이미지)와 통신하십시오. 서버가 브라우저에 보여 주는
인터페이스는 정확히 이 API를 사용합니다. 모든 버튼은 `/api/invoke/<command>` 호출이고, 모든
실시간 숫자는 `/api/events`로 도착합니다. 따라서 서버 페이지에서 사람이 할 수 있는 일은
스크립트도 할 수 있습니다.

데스크톱 앱에는 HTTP API가 없습니다. 창은 앱 안에서 엔진에 도달합니다. 데스크톱에서
자동화하려면 같은 컴퓨터에서 서버를 실행하거나([서버](../server/index.md) 참고), 명령줄
[`signallab`](../automation/cli.md)을 사용하십시오.

## 엔드포인트 {#endpoints}

| 메서드와 경로 | 하는 일 | 토큰 |
| --- | --- | --- |
| `GET /api/health` | 서버가 응답하는지, 버전, 토큰을 요구하는지 | 필요 없음 |
| `POST /api/invoke/<command>` | 엔진 명령 하나를 실행합니다. JSON 인수를 받아 JSON 결과를 돌려줍니다([명령](commands.md)) | 필요 |
| `POST /api/run` | 실험을 끝까지 실행합니다. 결과, 또는 단계를 줄 단위로([실행](run.md)) | 필요 |
| `GET /api/events` | 모든 엔진 이벤트의 WebSocket([이벤트](events.md)) | 필요 |
| `GET /api/files?path=…` | 엔진이 데이터 폴더에 쓴 파일을 다운로드로 | 필요 |
| `GET /api/openapi.json` | 이 API를 OpenAPI 3.1로 설명 | 필요 |
| `GET /login`, `POST /login` | 브라우저용 로그인 페이지와 양식 | 필요 없음 |
| `POST /logout` | 브라우저의 세션을 끝냅니다 | 세션 |

`/api/` 아래의 다른 경로는 코드 `api.not_found`와 함께 `404`를 응답합니다. 경로가 받지 않는
메서드(`GET /api/invoke/…`)는 빈 본문과 함께 `405`입니다. 그 밖의 모든 것은
인터페이스입니다. 토큰이 있는 서버에서는 세션이 없는 브라우저가 먼저 `/login`으로
보내집니다.

## 기본 URL {#base-url}

서버는 `--listen`(또는 `SIGNALLAB_LISTEN`)으로 다르게 지정하지 않는 한
`http://127.0.0.1:1430`에서 수신합니다. 도커 이미지는 모든 네트워크 카드인 `0.0.0.0:1430`에서
수신합니다. 이 페이지들의 예제는 다음을 사용합니다.

```bash
SERVER=http://127.0.0.1:1430
```

서버는 일반 HTTP로 통신합니다. HTTPS를 쓰려면 TLS를 종료하는 리버스 프록시를 앞에 두고
서버를 `--secure-cookie`로 시작하십시오.

## 인증 {#authentication}

토큰 없이 시작한 서버는 루프백에서만 수신하며 인증이 필요 없습니다. 그 컴퓨터에 있는 사람은
누구나 사용할 수 있습니다. 다른 사람이 닿을 수 있는 서버에는 항상 토큰이 있으며, 그러면
`/api/health`와 `/login`을 제외한 모든 요청이 토큰을 함께 보내야 합니다.

**스크립트**는 `Authorization` 헤더로 토큰을 보냅니다.

```bash
TOKEN=$(cat token.txt)
curl -fsS "$SERVER/api/invoke/jobs_list" -X POST \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json"
```

헤더는 정확히 `Bearer`, 공백 하나, 토큰이어야 합니다. 토큰이 없거나 틀리면 코드
`auth.required`와 함께 `401`입니다.

**브라우저**는 `/login`에서 토큰으로 한 번 로그인하고 세션 쿠키 `signallab_session`을
받습니다. 이 쿠키는 `HttpOnly`, `SameSite=Strict`이며 7일 동안 유지되고, 서버가
`--secure-cookie`로 실행되면 `Secure`입니다. `POST /logout`이 세션을 끝냅니다. 로그인
양식에서 틀린 토큰은 답하기 전에 1초를 기다리게 하므로 추측이 느려집니다. 세션은 서버의
메모리에 있습니다. 다시 시작하면 모든 브라우저가 로그아웃되지만 토큰을 쓰는 스크립트는
영향을 받지 않습니다. 서버는 세션을 최대 1024개 보관하며, 그보다 많아지면 가장 오래된 것이
사라집니다.

토큰의 출처는 서버의 설정입니다(`--token-file`, `SIGNALLAB_TOKEN`, 또는 `--generate-token`이
데이터 폴더에 만드는 `token` 파일). [서버 보안](../server/security.md)을 참고하십시오. 토큰은
공백 없이 24자 이상입니다. `signal-lab-server token`이 새 토큰을 출력합니다.

::: warning
토큰을 가진 사람은 서버가 트래픽을 보내게 할 수 있습니다. 토큰이 들어 있는 파일은 자신만 읽을
수 있게 유지하고, URL에 절대 넣지 마십시오. 서버는 어차피 URL의 토큰을 읽지 않습니다.
:::

## 호스트와 오리진 {#host-origin}

`/api/health`를 포함한 모든 경로에서, 무엇보다 먼저 두 가지 검사가 실행됩니다.

**`Host`.** `Host` 헤더는 이 서버를 가리켜야 합니다.

- 루프백 이름은 항상 통과합니다. `localhost`, `.localhost`로 끝나는 이름, `127.x.x.x`,
  `[::1]`입니다.
- `--allowed-host`(또는 `SIGNALLAB_ALLOWED_HOSTS`)로 지정한 이름은 통과합니다.
- 토큰이 있고 `--allowed-host`가 없는 서버는 어떤 이름에도 응답합니다.

그 밖의 것은 `auth.host`와 함께 `403`입니다. 서버가 받아들이는 이름으로 서버에
접근하십시오. `curl`은 지정한 URL의 호스트를 보냅니다.

**`Origin`.** 무언가를 바꾸는 요청(`GET`과 `HEAD`가 아닌 모든 메서드)과 WebSocket
업그레이드는 `Origin` 헤더를 함께 보낼 때 서버 자체의 페이지에서 와야 합니다. 즉 호스트와
포트가 `Host`와 같아야 합니다. 그렇지 않으면 `auth.origin`과 함께 `403`입니다.
`Origin: null`도 거부됩니다. 스크립트와 `curl`은 `Origin`을 보내지 않으므로 이 검사를
통과합니다. 그래도 토큰은 필요합니다.

**JSON만.** `POST /api/invoke/…`와 `POST /api/run`은 `Content-Type: application/json`을
받습니다(`; charset=utf-8` 같은 매개변수는 괜찮습니다). 그 밖의 것은 `command.json_required`와
함께 `415`입니다. 다른 사이트의 웹 페이지는 서버에 먼저 묻지 않고는 이것을 보낼 수 없으며,
서버는 결코 동의하지 않습니다.

## 명령 호출하기 {#invoke}

```http
POST /api/invoke/<command>
Content-Type: application/json

{ "argument": "value", … }
```

- 본문은 명령의 인수를 담은 JSON 객체 하나입니다. 인수가 없는 명령은 `{}` 또는 빈 본문을
  받습니다(`Content-Type` 헤더는 여전히 필요합니다).
- 인수 이름은 인터페이스가 보내는 대로 camelCase입니다. `jobId`, `nodeId`입니다. 인수로
  전달되는 객체(`config`, `request`, `document`, `library`…)는 엔진이 쓰는 필드 이름을
  유지하며, 대부분 snake_case입니다. `timeout_ms`, `port_start`입니다.
- 명령이 모르는 인수는 오류입니다. 결코 무시되지 않습니다. 명령 이름을 담은
  `command.args_invalid`와 함께 `422`이고, 파서의 말이 `detail`에 들어갑니다. 필수 인수를
  빠뜨린 경우도 마찬가지입니다. 인수가 없는 명령은 본문을 전혀 읽지 않습니다.
- 선택적 인수는 빠뜨리거나 `null`로 보낼 수 있습니다.
- 응답은 명령의 결과를 JSON으로 담은 `200`입니다. 돌려줄 것이 없는 명령은 `null`을
  응답합니다.

모든 명령이 그 인수와 결과와 함께 [명령](commands.md)에 있습니다.

## 오류 {#errors}

| 상태 | 언제 | 본문 |
| --- | --- | --- |
| `200` | 명령이 실행됨. `/api/run`이 실행을 시작함 | 결과 |
| `400` | 본문이 JSON이 아님. 실행 요청을 읽을 수 없음 | `EngineError`: `command.args_invalid`, `api.run_invalid`, `api.run_source` |
| `400` | `/api/files`에 `path`가 없음 | 웹 서버의 일반 텍스트, `EngineError`가 아님 |
| `401` | 토큰이 없거나 틀림 | `EngineError`: `auth.required` |
| `403` | 서버가 거부하는 `Host` 또는 `Origin` | `EngineError`: `auth.host`, `auth.origin` |
| `404` | 그런 API 경로가 없음. 데이터 폴더에 없는 파일 | `EngineError`: `api.not_found`, `file.not_found` |
| `405` | 경로가 받지 않는 메서드 | 비어 있음 |
| `413` | 24 MiB를 넘는 요청 본문 | 웹 서버의 일반 텍스트, `EngineError`가 아님 |
| `413` | 256 MiB를 넘는 다운로드 | `EngineError`: `file.too_large` |
| `415` | `application/json`이 아님 | `EngineError`: `command.json_required` |
| `422` | 명령이 실패함, 또는 실행을 시작할 수 없음 | `EngineError`: 엔진의 아무 코드 |
| `500` | 파일을 읽을 수 없음 | `EngineError`: `file.io` |

알 수 없는 명령 이름은 `command.unknown`과 함께 `422`입니다.

엔진이 보고하는 모든 실패는 `EngineError`라는 하나의 형태를 가집니다.

```json
{
  "code": "transport.refused",
  "params": { "target": "http://127.0.0.1:8080/" },
  "node": "request",
  "field": { "key": "url" },
  "detail": "error sending request for url (http://127.0.0.1:8080/): tcp connect error: Connection refused (os error 111)"
}
```

| 필드 | 내용 |
| --- | --- |
| `code` | 무엇이 잘못되었는지: 안정적인 식별자. 모든 코드와 그 메시지는 [오류 메시지](../reference/errors.md)에 나열되며, 점 앞부분을 기준으로 묶입니다(예: [`transport`](../reference/errors.md#transport)) |
| `params` | 메시지가 가리키는 값들, 모두 문자열. 없으면 빠집니다 |
| `node` | 관련된 실험 노드. 없으면 빠집니다 |
| `field` | 관련된 필드: `key`([필드](../reference/errors.md#fields)에 이름이 있음)와, 헤더 같은 반복 필드의 경우 1부터 시작하는 `index`. 없으면 빠집니다 |
| `detail` | 운영 체제, 파서 또는 라이브러리 자체의 말, 영어. 없으면 빠집니다 |

`detail`이 아니라 `code`를 기준으로 분기하십시오. 실행이나 단일 전송이 사용하는 시크릿 값은
보고하는 모든 오류에서 가려집니다(`••••`).

서버에 도달했지만 오류 상태나 아무 응답도 받지 못한 요청은 실패한 명령이 아닙니다.
`http_request`는 응답과 함께 `200`을 응답하고, `ok`, `error`, `cause`가 무슨 일이 있었는지
말합니다. [`http_request`](commands.md#http_request)를 참고하십시오.

## 한도 {#limits}

| 항목 | 한도 | 한도에 도달하면 |
| --- | --- | --- |
| 요청 본문 | 24 MiB | `413` |
| 실험 문서 | 4 MiB | `file.too_large` |
| `/api/files`에서의 다운로드 | 256 MiB | `413`, `file.too_large` |
| 실행의 길이 | 300 s | 실행이 `run.timeout`으로 실패합니다 |
| 피드백 양식(`feedback_send`) | 모두 15 MiB | `feedback.too_large` |
| WebSocket 하나를 기다리는 이벤트 | 4096 | 얼마나 놓쳤는지와 함께 [`server://lagged`](events.md#event-server-lagged)를 받습니다 |

## 이벤트 {#events}

WebSocket으로 업그레이드된 `GET /api/events`는 엔진이 보내는 모든 이벤트(실행의 단계, 작업의
끝, 모니터의 메시지, 인스펙터의 프레임)를 연결된 모든 클라이언트에 텍스트 메시지로
스트리밍합니다.

```json
{ "event": "job://ended", "payload": { "job_id": 7, "kind": "storm", "error": null } }
```

나머지와 같은 토큰과 `Origin` 규칙을 따릅니다. 모든 채널과 그 페이로드는
[이벤트](events.md)에 있습니다.

## 파일 {#files}

`GET /api/files?path=<path>`는 엔진이 서버의 데이터 폴더에 쓴 파일, 즉 실행 보고서(실행
결과의 `report_path`), 실험 또는 인스펙터의 내보내기, 신호 또는 에뮬레이터 라이브러리를
다운로드합니다. `path`는 엔진이 알려 준 서버상의 경로입니다(URL 인코딩).

```bash
curl -fsS -G "$SERVER/api/files" --data-urlencode "path=/data/runs/run-1759600000000-3.json" \
  -H "Authorization: Bearer $TOKEN" -o report.json
```

- 데이터 폴더 안에 있는 파일만 제공됩니다. 그 밖의 것, 폴더, 또는 존재하지 않는 파일은
  `file.not_found`와 함께 `404`입니다.
- 응답은 `Content-Disposition: attachment`와 함께 `application/octet-stream`입니다. 파일
  이름에서 글자, 숫자, `.`, `_`, `-`가 아닌 문자는 `_`가 됩니다.
- 256 MiB를 넘는 파일은 `file.too_large`와 함께 `413`입니다.

데이터 폴더에 무엇이 들어 있는지는 [파일과 폴더](../reference/files.md)에 있습니다.

## 헬스체크 {#health}

`GET /api/health`는 열려 있습니다. 토큰이 필요 없고, 서버가 받아들이는 `Host`만 있으면
됩니다.

```bash
curl -fsS "$SERVER/api/health"
```

```json
{ "status": "ok", "version": "[[version]]", "auth": true }
```

`auth`는 요청에 토큰이 필요한지 말합니다. `signal-lab-server healthcheck`는 서버가 수신하는
같은 주소에 묻고 응답하면 `0`으로 종료합니다. 도커 이미지의 헬스체크가 이것을 실행합니다.

## OpenAPI 설명 {#openapi}

`GET /api/openapi.json`은 이 API를 OpenAPI 3.1로 설명합니다. 엔드포인트, 모든 명령과 그
인수, 실행 요청과 결과입니다. 나머지 `/api/`처럼 토큰이 필요합니다. 이 페이지들이 전체 참고
자료이며, 둘이 다를 경우 이 페이지들이 엔진을 따릅니다.

## 작업 {#jobs}

오래 실행되는 작업(모니터, 발생기, 버스트, 릴레이, 연결, 에뮬레이터, 실행)은 작업입니다.
작업을 시작하는 명령은 실행되자마자 그 `JobInfo`를 돌려줍니다.

```json
{ "id": 4, "kind": "osc-monitor", "label": "OSC monitor 0.0.0.0:9000", "params": { "bind": "0.0.0.0:9000" }, "started_ms": 1759600000000 }
```

| 필드 | 내용 |
| --- | --- |
| `id` | 작업의 번호로, 서버가 실행되는 동안 고유합니다. 다른 명령은 이를 `jobId` 또는 `id`로 받습니다 |
| `kind` | `experiment`, `osc-monitor`, `osc-gen`, `http-burst`, `netsim`, `storm`, `scan`, `beacon`, `discovery`, `mqtt`, `websocket` 또는 `emulator` |
| `label` | 로그용 영어 한 줄 |
| `params` | 레이블을 이루는 값들(대상, 바인드, 호스트…). 없으면 빠집니다 |
| `started_ms` | 시작한 시각, 1970년 이후 밀리초 |

다음 명령이 작업을 시작합니다. `experiment_start`, `osc_monitor_start`,
`osc_generator_start`, `http_burst_start`, `netsim_start`, `storm_start`, `scan_start`,
`broadcast_beacon_start`, `discovery_start`, `mqtt_connect`, `ws_connect`, `emulator_start`.
서버는 요청한 클라이언트의 주소와 함께 각 시작을 로그에 남깁니다. `POST /api/run`도 작업을
시작합니다.

- [`jobs_list`](commands.md#jobs_list)는 실행 중인 작업을 나열하고,
  [`job_stop`](commands.md#job_stop)은 하나를 중지하며,
  [`jobs_stop_all`](commands.md#jobs_stop_all)은 모두 중지합니다.
- 스스로 끝나거나 실패한 작업은 [`job://ended`](events.md#event-job-ended)를 보냅니다.
  중지한 작업은 더 이상 아무것도 보내지 않습니다. `job_stop`이 `true`로 응답하는 것이
  확인입니다.
- 작업은 시작한 클라이언트가 아니라 서버에 속합니다. 페이지를 닫거나 스크립트를 끝내도
  중지되지 않으며, 모든 클라이언트가 보고 중지할 수 있고, 서버가 종료되면 모두 중지됩니다.

## 전체 예제 {#example}

서버가 살아 있는지 묻고, 명령 하나로 작은 HTTP 에뮬레이터를 시작하고, 기본 제공
`http-check` 실험을 그 에뮬레이터에 대해 실행하고 결과를 기다린 다음, 에뮬레이터를
중지합니다. `jq`가 응답에서 필드를 뽑습니다.

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)            # leave out with a loopback server without a token
AUTH="Authorization: Bearer $TOKEN"
JSON="Content-Type: application/json"

# 1. Up? Which version? Does it want a token?
curl -fsS "$SERVER/api/health"
# {"status":"ok","version":"[[version]]","auth":true}

# 2. One command: an HTTP emulator on 127.0.0.1:8080 that answers GET / with 200
EMULATOR=$(curl -fsS -X POST "$SERVER/api/invoke/emulator_start" -H "$AUTH" -H "$JSON" -d '{
  "emulator": { "name": "Example", "bind": "127.0.0.1:8080", "protocol": "http",
                "routes": [ { "method": "GET", "path": "/", "responses": [ { "body": "ok" } ] } ] }
}' | jq .id)

# 3. Run the bundled experiment that expects 200 from http://127.0.0.1:8080/, and wait
curl -sS -X POST "$SERVER/api/run" -H "$AUTH" -H "$JSON" -d '{"template":"http-check"}' \
  | jq '{outcome, error, report_path}'
# {"outcome":"passed","error":null,"report_path":"/data/runs/run-1759600000000-2.json"}

# 4. Stop the emulator
curl -fsS -X POST "$SERVER/api/invoke/job_stop" -H "$AUTH" -H "$JSON" -d "{\"id\":$EMULATOR}"
# true
```

`curl -f`는 오류 상태를 실패한 명령으로 바꿉니다. 본문에서 `EngineError`를 보려면
(3단계처럼) 빼십시오.

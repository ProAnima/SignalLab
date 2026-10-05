---
title: HTTP
description: HTTP 요청 하나를 보내고 전체 응답을 읽으며, Basic, Bearer 또는 Digest로 인증하고, 쿠키를 유지하며, 동시 부하 폭주로 엔드포인트를 측정합니다.
---

# HTTP

[[ui:nav.http]] 화면은 요청 검사기와 부하 도구를 하나로 합친 것입니다:

- 요청 하나를 보내고 상태, 걸린 시간, 헤더, 본문을 봅니다;
- Basic, Bearer 토큰 또는 Digest로 인증합니다;
- 브라우저처럼 서버가 설정한 쿠키를 유지합니다;
- 같은 요청을 한꺼번에 여러 번 보내고 — [[ui:http.burst]] — 처리량과 지연 백분위수를 읽습니다.

## 요청 보내기 {#request}

1. [[ui:nav.http]]를 엽니다.
2. 메서드를 고르고 URL을 입력합니다. 예: `http://127.0.0.1:8080/health`.
3. 서버가 필요로 하면 [[ui:http.headers]]를 추가합니다; [[ui:http.addHeader]]는 행을 더하고, ✕는
   하나를 제거합니다. 이름이 없는 행은 보내지 않습니다.
4. GET과 HEAD가 아닌 메서드에서는 [[ui:http.body]]를 씁니다. 이 텍스트는 GET이나 HEAD로 전환하는
   동안 필드에 남아 있다가 다른 메서드로 돌아오면 다시 나오지만, 그동안 전송되지는 않습니다.
5. [[ui:common.send]]를 누릅니다.

버튼 아래 줄이 즉시 판정을 줍니다 — 상태, 시간, 크기, 또는 응답이 없었던 이유 — 그리고
[[ui:http.response]] 패널이 나머지를 보여 줍니다. 요청(메서드, URL, 헤더, 본문, 시간 제한,
[[ui:http.keepCookies]])은 화면을 전환해도, 앱을 다시 시작해도 유지됩니다; 자격 증명은 아닙니다.

| 키 | 위치 | 하는 일 |
| --- | --- | --- |
| <kbd>Enter</kbd> | URL, 헤더, 자격 증명, 시간 제한 | 전송 |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | 본문을 포함한 요청의 아무 필드 | 전송 |
| <kbd>Ctrl</kbd>+<kbd>S</kbd> | 요청의 아무 필드 | 신호로 저장([아래](#library)) |

### 요청 필드 {#request-fields}

| 필드 | 내용 | 기본값 |
| --- | --- | --- |
| [[ui:exp.method]] | GET, POST, PUT, PATCH, DELETE, HEAD 또는 OPTIONS | GET |
| [[ui:field.url]] | `http://` 또는 `https://` URL | `http://127.0.0.1:8080/` |
| [[ui:http.headers]] | 이름과 값 쌍, 쓴 그대로 전송. 자체 `User-Agent`가 없으면 Signal Lab은 `SignalLab/0.1`을 보냅니다. | `Accept: application/json` |
| [[ui:field.auth]] | 요청이 인증하는 방식([아래](#auth)) | [[ui:http.auth.none]] |
| [[ui:http.keepCookies]] | 서버가 설정한 쿠키를 돌려 보냅니다([아래](#cookies)) | 켜짐 |
| [[ui:http.body]] | 쓴 그대로 전송; `Content-Type`을 덧붙이지 않으므로 그에 맞는 헤더를 추가하십시오. 빈 본문은 보내지 않습니다. GET과 HEAD에서는 이 필드가 표시되지 않고, 그러면 요청이 본문을 전혀 싣지 않습니다 — 전송에도, 저장한 신호에도, [[ui:common.toExperiment]]에도 — 다른 메서드에서 입력했더라도 말입니다. | 비어 있음 |
| [[ui:common.timeoutMs]] | 답과 본문을 포함해 전체 교환이 걸릴 수 있는 시간 | 10 000 |

## 인증 {#auth}

| [[ui:field.auth]] | 필드 | 보내는 것 |
| --- | --- | --- |
| [[ui:http.auth.none]] | — | `Authorization` 헤더 없음 |
| [[ui:http.auth.basic]] | [[ui:field.username]], [[ui:field.password]] | `Authorization: Basic …`, 이름과 비밀번호를 base64로, 첫 요청과 함께 |
| [[ui:http.auth.bearer]] | [[ui:field.token]] | `Authorization: Bearer <token>` |
| [[ui:http.auth.digest]] | [[ui:field.username]], [[ui:field.password]] | 처음에는 아무것도; 서버의 challenge에 대한 답([아래](#digest)) |

Basic과 Digest 사이를 전환해도 이름과 비밀번호는 유지됩니다.

### Digest {#digest}

Digest에서는 Signal Lab이 자격 증명 없이 요청을 보냅니다. 서버가 Digest challenge와 함께 `401`로
답하면, Signal Lab은 challenge와 비밀번호로 답을 계산해 요청을 다시 보냅니다. 보이는 응답은 그 두
번째 요청의 것이며 [[ui:http.digestAnswered]]로 표시되고, 지연은 두 교환을 모두 셉니다 —
클라이언트가 기다리는 것입니다.

- 알고리즘: MD5와 SHA-256, 그리고 그 `-sess` 변형. 서버가 둘 다 제시하면 SHA-256을 씁니다.
- 보호 품질: `auth`와 `auth-int`, 그리고 `qop` 없는 예전 답.
- 서버가 nonce가 소진되었다고(`stale`) 하거나 새 nonce로 다시 요청하면, 요청에 최대 3번 더
  답합니다. 마지막으로 준 nonce에 대한 답을 거부하면 `401`이 유지됩니다: 이름이나 비밀번호가
  틀렸습니다.
- 리다이렉트는 Signal Lab이 직접 따라가므로, 요청한 URL이 곧 답하는 URL입니다. 다른 origin의
  challenge에는 답하지 않습니다: 한 호스트를 위해 입력한 자격 증명은 다른 곳으로 가지 않습니다.
  같은 호스트에서 기본 포트로 `http://`에서 `https://`로 옮기는 것은 같은 호스트로 봅니다.

challenge에 답할 수 없으면 `401`이 유지되고 패널이 이유를 말합니다:

| 메시지 | 뜻 |
| --- | --- |
| The server answered 401 without asking for Digest | 서버가 다른 방식을 원합니다; Basic이나 Bearer를 시도하십시오. |
| The server asked for Digest with … | Signal Lab이 말하지 않는 알고리즘입니다; MD5와 SHA-256을 말합니다. |
| The server asked for Digest without a realm or nonce | 서버의 challenge가 불완전합니다. |
| The request was sent on to …, which asked for Digest | 리다이렉트가 다른 origin으로 이어졌고, 그 challenge에는 답하지 않습니다. |

### 자격 증명이 가는 곳 {#credentials}

자격 증명은 보낼 때 요청의 `Authorization` 헤더에만 들어갑니다. 인스펙터, 콘솔, 실험 보고서는 그
헤더를 절대 보여 주지 않습니다. 이 화면에서는 메모리에만 보관되고 다시 시작하면 사라집니다 —
저장한 신호에 요청이 연결되어 있으면 그때는 다시 돌아옵니다.

::: warning
신호로 저장한 요청은 자격 증명을 라이브러리 파일 `signals.json`에 평문으로 보관합니다. 실험에서는
비밀번호를 `{{secret.NAME}}`으로 쓰십시오; [데이터와 템플릿](../experiments/data.md)을
참고하십시오.
:::

## 쿠키 {#cookies}

[[ui:http.keepCookies]]가 켜져 있으면, 서버가 `Set-Cookie`로 설정한 것이 이 화면의 쿠키 항아리에
보관되고 그 서버로의 이후 요청에 실려 돌아가며, 브라우저 규칙(도메인, 경로, `Secure`, 만료)을
따릅니다. 항아리는 이 화면의 요청, 그 폭주, 라이브러리에서 보내는 HTTP 신호가 사용합니다. 끄면
쿠키 없이 요청을 보내고 하나도 유지하지 않습니다.

요청과 응답 아래의 쿠키 패널은 항아리가 담은 것을 나열합니다: [[ui:http.cookieName]],
[[ui:http.cookieValue]], [[ui:http.cookieWhere]] (`.`로 시작하는 도메인은 하위 도메인도 포함합니다),
[[ui:http.cookieExpires]] (만료가 없는 쿠키는 [[ui:http.cookieSession]])와
[[ui:http.cookieFlags]] (`Secure`, `HttpOnly`, `SameSite`). 만료된 쿠키는 나열되지 않습니다.
[[ui:common.clear]]가 항아리를 비웁니다.

항아리는 앱이 사는 동안 삽니다: 다시 시작하면 빈 것으로 시작합니다. [서버](../server/index.md)에서는
로그인한 모든 페이지에 항아리가 하나씩 있습니다. 실험 실행은 자체 항아리를 가지며([실험](../experiments/index.md) 참고), `signallab send http`는 하나도 쓰지 않습니다.

## 응답 {#response}

| 부분 | 내용 |
| --- | --- |
| [[ui:http.status]] | 상태 코드와 그 이유; 응답이 오지 않으면 `ERR` |
| [[ui:http.latency]] | 전송부터 본문의 마지막 바이트까지, 밀리초 |
| [[ui:http.size]] | 본문의 크기 |
| 응답 헤더 | 개수가 적힌 줄을 클릭해 보이거나 숨깁니다 |
| 본문 | JSON이면 서식이 잡힙니다; [[ui:http.rawBody]]와 [[ui:http.formatJson]]이 전환합니다. 최대 256 KiB가 표시되고, 그 뒤는 `… (truncated)`. |

응답이 없으면 패널이 이유를 Signal Lab의 다른 곳과 같은 말로 알려 줍니다: 거부, 제때 답 없음,
이름이 해석되지 않음, 인증서 문제 등. 시스템의 기술적 세부는 그 아래에 접혀 있습니다.

### 리다이렉트 {#redirects}

리다이렉트(301, 302, 303, 307, 308)는 최대 10번까지 따라가며, 보이는 응답은 마지막 것입니다. 301,
302, 303 뒤에는 요청이 본문 없이 GET으로 이어가고(HEAD는 HEAD로 남습니다); 307, 308 뒤에는
원래대로 이어갑니다. 한 호스트를 위해 입력한 `Authorization`과 쿠키는 다른 곳으로 보내지 않습니다.

### 보안 연결 {#tls}

`https://` 서버의 인증서는 이 시스템이 신뢰하는 인증서와 대조해 검사됩니다. 자체 서명되었거나
만료된 인증서는 "A secure connection to … could not be made"와 함께 거부됩니다; 검사를 건너뛰는
설정은 없습니다. 자체 인증서가 있는 서버를 테스트하려면 시스템의 신뢰하는 인증서에 추가하십시오.

## 부하 폭주 {#burst}

[[ui:http.burst]]는 화면의 요청을 — 인증과, [[ui:http.keepCookies]]가 켜져 있는 동안 쿠키
항아리를 그대로 — 여러 번 보내고 측정합니다.

1. [[ui:common.concurrency]], [[ui:http.total]], [[ui:http.duration]], [[ui:http.rate]]를
   설정합니다.
2. [[ui:http.startBurst]]를 누릅니다. 폭주는 하나의 작업입니다: [[ui:http.stopBurst]], 또는 콘솔
   표시줄에서의 중지가 그것을 끝냅니다.

| 필드 | 내용 | 기본값 |
| --- | --- | --- |
| [[ui:common.concurrency]] | 동시에 진행 중인 요청 수, 1–512 | 20 |
| [[ui:http.total]] | 보낼 요청 수; 0 — 시간이 끝날 때까지 계속 보냄 | 500 |
| [[ui:http.duration]] | 실행할 초; 0 — 총계를 다 보내면 중지 | 0 |
| [[ui:http.rate]] | 초당 시작하는 요청 수, 0.1–100 000; 0 — 작업자가 가는 만큼 빠르게 | 0 |

[[ui:http.total]]과 [[ui:http.duration]]이 둘 다 0이면, 폭주는 중지할 때까지 실행됩니다.

보내는 방법은 두 가지입니다:

- **[[ui:http.rate]] 0.** 각 작업자는 답을 받는 즉시 다시 보냅니다. 서버가 얼마나 감당하는지
  알아내지만, 느린 서버는 폭주도 느리게 만듭니다.
- **속도.** 요청은 답이 얼마나 느리든 정해진 일정에 시작합니다 — 초당 10이면 시작부터 100 ms마다
  하나씩. 모든 작업자가 바쁠 때 자기 차례가 온 요청은 최대 50 ms까지 하나를 기다립니다; 그 뒤에는
  건너뛰고 [[ui:http.missed]]로 세며, 늦게라도 보내지 않습니다. 건너뛴 요청은 이 속도에 비해
  동시성이 너무 낮거나, 서버가 그 속도에 필요한 것보다 느리다는 뜻입니다.

| 숫자 | 내용 |
| --- | --- |
| [[ui:http.sent]] | 답을 받았거나 실패한 요청 |
| [[ui:http.ok]] | 2xx 상태로 답한 것 |
| [[ui:http.failed]] | 답이 없거나, 200–299 밖의 상태 |
| [[ui:http.missed]] | 위와 같이 건너뛴 것(속도가 있을 때만) |
| [[ui:http.rps]] | 지난 0.1초 동안의 초당 요청 수; 폭주가 끝났으면 전체 폭주 기준. 속도가 있으면 라벨에 요청한 속도를 밝힙니다. |
| [[ui:http.p50]], [[ui:http.p90]], [[ui:http.p95]], [[ui:http.p99]] | 그 비율의 요청이 실패 포함해 끝난 시간, 0.5 % 이내 정확도 |
| [[ui:http.avg]], [[ui:http.min]], [[ui:http.max]] | 평균, 가장 빠름, 가장 느림 |

숫자는 초당 약 10회 갱신됩니다. 옆의 차트는 지난 24초 정도의 초당 요청 수를 그립니다.

Digest에서는 첫 요청의 challenge를 한 번 답하고, 그 답이 폭주의 모든 요청에 쓰입니다.

::: warning
폭주는 실제 부하입니다. 소유했거나 테스트가 허용된 서버에만 향하십시오.
:::

램프, 단계, 스파이크, 통과/실패 임계값은 [실험에서 부하로](../experiments/load.md) 요청을
실행하십시오.

## 인스펙터에서 {#inspector}

캡처가 켜져 있으면 각 교환은 `http` 프로토콜과 `http` 출처의 한 프레임으로 나타납니다: 요약에는
메서드, URL, 상태, 시간이, 세부에는 응답 헤더와 본문의 시작(2000자)이, 판정에는 상태가
들어갑니다(응답이 없으면 `failed`, challenge에 답했으면 `· digest after 401`). 프레임은 본문의
크기를 기록할 뿐 바이트를 기록하지 않습니다. 요청의 `Authorization` 헤더는 절대 들어가지 않습니다.
폭주는 100 ms마다 최대 하나의 교환을 캡처에 넣습니다. [인스펙터](../tools/inspector.md)를
참고하십시오.

## 저장하고 재사용하기 {#library}

- **신호로 저장.** [[ui:sig.saveNew]]는 요청(메서드, URL, 헤더, 본문, 시간 제한, 인증)을 신호
  라이브러리에 보관합니다. 화면은 그 신호와 연결된 채로 남습니다: [[ui:sig.save]] (<kbd>Ctrl</kbd>+<kbd>S</kbd>)는 갱신하고, [[ui:sig.saveAs]]는 복사하며, 칩은 [[ui:nav.signals]]에서 엽니다. 라이브러리에서 HTTP 신호를 열면 자격 증명까지 이 화면으로 다시 로드됩니다. [신호](../tools/signals.md)를 참고하십시오.
- **실험에 추가.** [[ui:common.toExperiment]]는 같은 요청을 가진 [HTTP 요청](../experiments/nodes.md#node-http) 단계를 열려 있는 실험에 추가하고(End 바로 앞이나 선택한 단계 뒤) 엽니다.
- **모의 응답 만들기.** 응답 아래에서 [[ui:http.mockThis]]는 이 메서드와 경로를 이 상태, 헤더,
  본문으로 답하는 에뮬레이터 라우트를 만듭니다. [[ui:http.mockInto]]에서 HTTP 에뮬레이터를
  고르거나 [[ui:http.mockNew]]를 고르고 [[ui:http.mockAdd]]를 누르십시오; 라우트는 그
  에뮬레이터의 맨 앞에 들어가고, [[ui:nav.emulators]] 화면이 그것을 연 상태로 열립니다.
  [에뮬레이터](../tools/emulators.md)를 참고하십시오.

## 실험에서 {#experiments}

| 단계 | 하는 일 |
| --- | --- |
| [[ui:exp.node.http]] | 요청을 보냅니다; URL, 헤더, 본문, 자격 증명은 `{{templates}}`를 받습니다. [부하로](../experiments/load.md) 실행할 수 있습니다. [자세히](../experiments/nodes.md#node-http) |
| [[ui:exp.node.assert_status]], [[ui:exp.node.assert_body]], [[ui:exp.node.assert_header]], [[ui:exp.node.assert_latency]] | 최신 응답을 검사합니다. [자세히](../experiments/nodes.md#node-assert_status) |
| [[ui:exp.node.extract]] | JSON 필드, 헤더, 상태, 본문 또는 정규식의 일치를 변수로 보관합니다. [자세히](../experiments/nodes.md#node-extract) |
| [[ui:exp.node.branch_status]] | 상태에 따라 Yes 또는 No로 나아갑니다. [자세히](../experiments/nodes.md#node-branch_status) |
| [[ui:exp.node.wait_http]] | 테스트하는 시스템에서 실행의 자체 수신자나 에뮬레이터로 요청이 도착하기를 기다립니다. [자세히](../experiments/nodes.md#node-wait_http) |
| [[ui:exp.node.emulator]] | 실행 내내 라우트에 따라 응답하는 HTTP API. [자세히](../experiments/nodes.md#node-emulator) |

## 명령줄에서 {#cli}

`signallab send http`는 이 화면처럼 요청 하나를 보냅니다:

```bash
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab send http POST http://127.0.0.1:8080/api/items \
  -H 'Content-Type: application/json' --body '{"name":"lamp"}'
signallab send http GET http://127.0.0.1:8080/private -u admin:secret --digest
```

상태 줄은 표준 오류로, 본문은 표준 출력으로 나갑니다:

```text
HTTP 200 OK · 3 ms · 15 B
{"status":"ok"}
```

| 옵션 | 내용 | 기본값 |
| --- | --- | --- |
| `-H`, `--header 'Name: value'` | 헤더; 더 하려면 반복 | — |
| `--body TEXT`, `--body @FILE` | 본문, 또는 파일의 내용 | — |
| `--expect-status N` | 상태가 N이 아니면 1로 종료 | — |
| `--timeout MS` | 응답을 기다리는 시간 | 10 000 |
| `-u`, `--user NAME:PASSWORD` | Basic 인증 | — |
| `--digest` | `--user`와 함께: 대신 서버의 Digest challenge에 답합니다 | — |
| `--bearer TOKEN` | `Authorization: Bearer TOKEN` | — |
| `--json` | 전체 응답을 JSON으로 표준 출력에 인쇄 | — |

응답이 오면(그리고 기대한 상태였으면) 0으로, 오지 않거나 상태가 기대와 다르거나 Digest challenge에
답할 수 없었으면 1로, 옵션이 잘못되었으면 2로 종료합니다. 쿠키는 유지하지 않습니다.
[명령줄](../automation/cli.md#cli-send-http)을 참고하십시오.

## 문제 {#troubleshooting}

| 보이는 것 | 흔한 원인 |
| --- | --- |
| `… refused the connection — nothing is listening on that port` | 서버가 실행 중이 아니거나, 다른 포트나 주소에서 수신합니다. |
| `No answer from … in time` | 서버가 느리거나 도달할 수 없습니다; 주소를 확인하거나 [[ui:common.timeoutMs]]를 올리십시오. |
| `Cannot resolve …` | 이 컴퓨터에서 호스트 이름이 해석되지 않습니다 — 오타이거나, 다른 네트워크만 아는 이름입니다. |
| `A secure connection to … could not be made` | 여기서 인증서를 신뢰하지 않거나(자체 서명, 만료, 다른 이름), TLS가 실패했습니다. [보안 연결](#tls)을 참고하십시오. |
| `… is not a valid address` | URL이 잘못되었거나 `http://` 또는 `https://`로 시작하지 않습니다. |
| 서버가 본문이 없거나 타입이 틀렸다고 함 | 본문과 맞는 `Content-Type` 헤더가 없거나, 본문이 비어 있습니다. |
| Digest와 함께 `401` | 상태 아래의 메시지를 읽으십시오: [Digest](#digest)를 참고하십시오. |
| [[ui:http.missed]]가 0보다 큼 | [[ui:common.concurrency]]를 올리거나 속도를 낮추십시오: 서버가 그 속도에 필요한 것보다 느리게 답합니다. |
| 서버가 답하는데 [[ui:http.failed]]가 높음 | 200–299 밖의 모든 상태가 실패로 세어지며, 404와 500도 포함됩니다. |

서버에서는 요청이 서버에서 나갑니다: `127.0.0.1`은 서버 자신입니다.
[서버](../server/index.md)를 참고하십시오.

모든 오류 메시지는 [오류 메시지](../reference/errors.md#transport)에 정리되어 있습니다.

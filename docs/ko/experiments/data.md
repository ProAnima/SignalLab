---
title: 데이터와 템플릿
description: 매개변수와 프로필, 생성기를 갖춘 템플릿 언어, 응답에서 추출하는 값, 비교, 그리고 엔진을 절대 벗어나지 않는 시크릿.
---

# 실험의 데이터

값은 실행을 따라 이동합니다. 매개변수가 대상을 고르고, 한 응답의 필드가 다음 요청의 헤더가 되며,
생성된 id가 명령으로 나갔다가 검사로 돌아옵니다. 이 페이지에서는 그런 값이 어디에서 오는지, 필드가
그것을 어떻게 사용하는지 다룹니다.

| 출처 | 쓰는 법 | 설정하는 곳 |
| --- | --- | --- |
| 매개변수 | `{{api}}` 또는 `{{params.api}}` | [[ui:exp.params]] 패널, 프로필, [[ui:exp.runWith]] |
| 변수 | `{{token}}` 또는 `{{vars.token}}` | 실행 중의 노드: [[ui:exp.node.extract]], 대기, 회신을 기다리는 전송 |
| 시크릿 | `{{secret.API_TOKEN}}` | 컴퓨터의 자격 증명 저장소, 또는 서버의 환경과 파일 |
| 내장 값 | `{{run.seed}}`, `{{now.iso}}`, `{{counter}}` | 실행 자체 |
| 생성기 | `{{uuid}}`, `{{random_int(1, 100)}}` | 실행의 시드에서 뽑음 |

## 매개변수 {#parameters}

매개변수는 템플릿 필드라면 어디든 쓸 수 있는 이름 붙은 텍스트 값입니다. 대상을 매개변수에 두면
주소가 바뀔 때 노드마다 고치는 대신 한 번만 고치면 됩니다.

### 매개변수 추가하기 {#add-parameter}

1. 편집기 도구 모음에서 [[ui:exp.params]] (`{ }`)를 누릅니다.
2. [[ui:exp.noProfile]] 탭에서 [[ui:exp.addParam]]을 누릅니다.
3. [[ui:exp.paramName]]과 [[ui:exp.paramValue]]를 입력합니다. 예를 들어 `api`와
   `http://127.0.0.1:8080`입니다.
4. 노드의 필드에 `{{api}}/login`을 씁니다.

패널에서의 모든 변경은 실험의 편집입니다. 실험과 함께 저장되며, 다른 편집과 마찬가지로
<kbd>Ctrl</kbd>+<kbd>Z</kbd>로 되돌립니다.

### 규칙 {#parameter-rules}

| 규칙 | 한도 |
| --- | --- |
| 이름 | 글자나 `_`로 시작하고, 그다음은 글자, 숫자, `_` |
| 예약어 | `vars`, `params`, `secret`, `run`, `node`, `now`, `uuid`, `counter`, `random_int`, `random_float`, `pick` |
| 실험당 매개변수 | 64 |
| 값 하나의 크기 | 64 KiB |
| 이름 | 고유해야 하며, 변수는 매개변수의 이름을 가질 수 없습니다 |

값은 순수 텍스트이며 쓴 그대로 삽입됩니다. 값 안의 `{{…}}`는 해석되지 않습니다. 필드가 매개변수의
일부를 요청하면(`{{config.ports[0]}}`) 값을 JSON으로 읽습니다. JSON이 아닌 값에는 일부가 없습니다.

이름이 잘못되었거나, 예약어이거나, 중복된 매개변수는 실험 저장을 막지 않으므로 계속 입력할 수
있습니다. 이름을 고치기 전까지는 실험이 실행되지 않습니다.

## 프로필 {#profiles}

프로필은 이름 붙은 매개변수 값의 묶음입니다. *노트북*, *무대*, *공연장*처럼 대상을 전환하는 일이
모든 노드를 고치는 대신 하나를 고르는 일이 됩니다. 프로필은 일부 매개변수만 바꾸고, 나머지는
기본값을 유지합니다.

### 프로필 만들기 {#make-profile}

1. [[ui:exp.params]]를 열고 [[ui:exp.addProfile]]을 누릅니다. 새 탭이 열립니다.
2. [[ui:exp.profileName]]에서 이름을 바꿉니다.
3. 프로필이 바꾸는 각 매개변수의 값을 입력합니다. 빈 필드는 기본값을 유지하며, 필드에 회색으로
   표시됩니다. [[ui:exp.resetToDefault]] (↺)는 값을 지웁니다.
4. [[ui:exp.makeActive]]를 누르면 그 프로필로 실행합니다. 활성 프로필의 탭에는
   ● [[ui:exp.activeProfile]]가 붙습니다. [[ui:exp.noProfile]] 탭에서 [[ui:exp.makeActive]]를
   누르면 기본값으로 돌아갑니다.

실험에 프로필이 생기면 도구 모음의 [[ui:exp.profile]] 목록으로 프로필을 전환합니다. 활성 프로필은
실행, 미리보기, [[ui:exp.sendNow]]에 사용되며 실험에 저장되므로, 내보낸 파일을 열어도 같은 대상이
유지됩니다. [[ui:exp.removeProfile]]은 화면에 있는 프로필을 삭제합니다.

| 규칙 | 한도 |
| --- | --- |
| 실험당 프로필 | 32 |
| 이름 | 1–64자, 고유(양끝의 공백은 세지 않음) |
| 값 | 존재하는 매개변수만, 최대 64개 |

매개변수의 이름을 바꾸거나 제거하면 모든 프로필에서 한꺼번에 바뀝니다.

### 실행이 사용하는 값 {#precedence}

뒤에 오는 값이 이깁니다:

1. [[ui:exp.noProfile]] 탭에 있는 매개변수의 기본값.
2. 활성 프로필의 값(설정한 경우).
3. 이 실행에만 [[ui:exp.runWith]]에 입력한 값 — [다른 값으로 실행하기](runs.md#run-with)를
   참고하십시오.

[[ui:exp.runWith]]는 실험에 있는 매개변수만 설정할 수 있습니다. 실행 보고서에는 프로필, 실행을 위해
입력한 값, 그리고 사용한 모든 값이 기록됩니다.

### 실행되지 않을 프로필 {#profile-issues}

실험이 검사될 때마다 다른 프로필과 기본값도 함께 검사됩니다. 실행에 실패할 프로필, 이를테면
`http://`나 `https://`가 아닌 URL이 있는 프로필에는 탭과 도구 모음 목록에 ⚠가 붙고, 툴팁이 이유를
알려 줍니다. 사용 중인 프로필로 실행하는 것은 막지 않습니다.

## 템플릿 {#templates}

`{{ }}` 안의 텍스트는 표현식이고, 필드의 나머지는 쓴 그대로 유지됩니다.

```text
{{api}}/users/{{user.id}}?trace={{uuid}}
Bearer {{secret.API_TOKEN}}
```

- 중괄호 안의 공백은 상관없습니다: `{{ token }}`은 `{{token}}`입니다.
- `\{{`는 문자 그대로의 `{{`를 씁니다.
- 홀로 있는 `}}`는 순수 텍스트입니다.
- 값은 따옴표 없이 그대로 삽입됩니다. JSON 본문에서는 따옴표를 직접 씁니다:
  `"id": "{{uuid}}"`.

### 이름 {#names}

| 표현식 | 값 |
| --- | --- |
| `{{name}}` | 이 경로에 변수 `name`이 설정되어 있으면 그 변수, 아니면 매개변수 `name` |
| `{{vars.name}}` | 변수만 |
| `{{params.name}}` | 매개변수만 |
| `{{secret.NAME}}` | 저장된 시크릿 `NAME` — [시크릿](#secrets)을 참고하십시오 |
| `{{name.field}}` | JSON 값의 필드 |
| `{{name[0]}}` | JSON 배열의 원소 |
| `{{name["a b"]}}`, `{{name['a b']}}` | 다른 문자를 포함하는 이름의 필드 |

`.` 뒤의 필드 이름에는 글자, 숫자, `_`, `-`를 쓸 수 있습니다. 단계는 이어집니다:
`{{reply.args[0]}}`, `{{order.items[2].sku}}`.

### 값이 쓰이는 형태 {#value-text}

| 값 | 쓰이는 형태 |
| --- | --- |
| 텍스트 | 텍스트 그대로 |
| 숫자 | 가장 짧은 형태: `42`, `0.5` |
| `true`, `false` | `true`, `false` |
| `null` | `null` |
| 객체, 배열 | 압축된 JSON: `["x","y"]` |

### 내장 값 {#built-ins}

| 표현식 | 값 |
| --- | --- |
| `{{run.id}}` | 실행의 작업 번호. 미리보기와 [[ui:exp.sendNow]]에서는 `0` |
| `{{run.seed}}` | 이 실행의 시드 |
| `{{node.id}}` | 실행 중인 노드의 id |
| `{{now}}` | 현재 시각, Unix 밀리초 |
| `{{now.iso}}` | 현재 시각을 UTC로, 밀리초를 포함한 ISO 8601: `2026-09-30T12:34:56.789Z` |
| `{{counter}}` | 이 실행에서 이 노드가 실행된 횟수(이번 포함), 1부터 |

`{{counter}}`는 노드별로 셉니다. [루프](flow.md#loop) 본문에서는 반복 번호이고,
[반복](flow.md#repeat)하는 노드에서는 전송 번호입니다. `run`, `node`, `now`에는 나열된 필드만 있으며,
그 밖의 것은 오류입니다.

### 생성기 {#generators}

| 표현식 | 값 |
| --- | --- |
| `{{uuid}}` 또는 `{{uuid()}}` | 버전 4 UUID |
| `{{random_int(min, max)}}` | `min`부터 `max`까지(양끝 포함)의 정수. 인수는 정수이고 `min` ≤ `max` |
| `{{random_float(min, max)}}` | `min` 이상 `max` 미만의 수, 소수점 3자리. `min` < `max` |
| `{{random_float(min, max, digits)}}` | 소수점 `digits`자리(0–9)인 같은 값 |
| `{{pick(a, b, c)}}` | 인수 중 하나, 최소 하나 |

인수는 쉼표로 구분합니다. 따옴표로 묶은 인수(`"dark blue"` 또는 `'a, b'`)에는 자기 따옴표를 제외한
무엇이든 넣을 수 있고, 따옴표가 없는 인수에는 글자, 숫자, `_ - . : / +`를 쓸 수 있습니다. 빈 인수는
오류입니다.

모든 생성기는 실행의 시드에서 뽑습니다. 노드를 한 번 실행할 때의 값은 시드, 노드의 id, 노드가
실행된 횟수에만 달려 있으므로, 병렬 분기가 서로의 값을 바꾸지 않으며, 같은 시드로 실행하면 같은
값이 다시 생성됩니다. 한 노드 안에서의 추출은 필드 순서를 따릅니다. `{{now}}`와 `{{run.id}}`는
반복할 수 없습니다. [시드](runs.md#seeds)를 참고하십시오.

### 제안 {#suggestions}

템플릿 필드에서 `{{`를 입력하거나 <kbd>Ctrl</kbd>+<kbd>Space</kbd>를 누르면 네 그룹으로 이루어진
목록이 열립니다: 값과 함께 [[ui:exp.suggest.params]], 이 노드보다 앞에서 설정한 변수를 설정한
노드와 함께 보여 주는 [[ui:exp.suggest.vars]] (회신의 `reply.args[0]` 같은 필드도 포함),
[[ui:exp.suggest.secrets]], [[ui:exp.suggest.generators]]. <kbd>↑</kbd>와 <kbd>↓</kbd>로 고르고,
<kbd>Enter</kbd>나 <kbd>Tab</kbd>으로 삽입하며, <kbd>Esc</kbd>는 목록을 닫고 필드는 그대로 둡니다.

### 알 수 없는 이름은 오류 {#unknown-names}

값이 없는 이름이 빈 문자열이 되는 일은 없습니다. 실행 전에 필드가 사용하는 모든 이름은
매개변수이거나, 유효한 시크릿 이름이거나, 그 노드로 이어지는 **모든** 경로에서 설정된 변수여야
합니다. 편집기는 노드와 필드를 가리킵니다:

| 문제 | 실행 전 | 실행 중 |
| --- | --- | --- |
| 아무도 설정하지 않는 이름 | `name.unknown` | — |
| 일부 경로에서만 설정되는 변수 | `name.not_on_every_path` | — |
| 매개변수 `x` 없는 `{{params.x}}` | `param.unknown` | — |
| 값에 없는 필드 | — | `template.no_field` |
| 닫히지 않은 `{{`, 빈 `{{}}`, 잘못된 인수 | 위치와 함께 `template.*` | — |

이 코드의 문구는 [오류](../reference/errors.md)에 있습니다.

## 템플릿을 쓰는 필드 {#templated-fields}

| 노드 | 템플릿 필드 |
| --- | --- |
| [[ui:exp.node.http]] | URL, 헤더 이름과 값, 본문, Basic과 Digest의 사용자 이름과 비밀번호, Bearer 토큰 |
| [[ui:exp.node.osc]] | 대상, 주소, 텍스트 인수. 회신이 있으면 그 주소 패턴과 규칙 값 |
| [[ui:exp.node.udp]] | 대상, 페이로드. 회신이 있으면 그 패턴 |
| [[ui:exp.node.tcp]] | 호스트, 페이로드 |
| [[ui:exp.node.mqtt]] | 브로커 호스트, 토픽, 페이로드 |
| [[ui:exp.node.log]] | 메시지 |
| [[ui:exp.node.assert_body]] | 기대하는 텍스트 |
| [[ui:exp.node.assert_header]] | 헤더 이름, 기대하는 텍스트 |
| [[ui:exp.node.assert_value]], [[ui:exp.node.branch_value]], [[ui:exp.node.loop]]의 종료 조건 | 값, 기대하는 값 |
| [[ui:exp.node.wait_osc]] | 주소 패턴, 규칙 값 |
| [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_ws]] | 패턴 |
| [[ui:exp.node.wait_mqtt]] | 브로커와 토픽(매개변수만), 패턴 |
| [[ui:exp.node.wait_http]] | 경로 패턴, 조건 |
| [[ui:exp.node.impairment]] | 수신과 대상(매개변수만) |
| [[ui:exp.node.ws_connect]] | URL, 헤더 이름과 값 |
| [[ui:exp.node.ws_send]] | 페이로드 |
| [[ui:exp.node.ws_close]] | 이유 |

숫자(포트, 시간 제한, 지연, 상태, 타입이 지정된 OSC 숫자)와 대기의 수신 주소는 리터럴입니다.
[[ui:exp.node.emulator]]는 도착한 내용(`{{request.…}}`)과 매개변수로 자체 회신을 렌더링합니다.
[장애 주입](faults.md#emulator)을 참고하십시오.

**매개변수만.** 일부 필드는 아직 변수가 없는 첫 단계 전에 열립니다:
[[ui:exp.node.wait_mqtt]]의 브로커와 토픽, [[ui:exp.node.impairment]]의 수신과 대상입니다. 이들은
텍스트와 매개변수만 받고 그 밖의 것은 받지 않습니다(`node.params_only`).

**리터럴처럼 검사합니다.** 매개변수만 사용하는 필드는 실행 전에 해석되어 실행이 보낼 텍스트로
검사됩니다. URL은 `http://`나 `https://`여야 하고, OSC 대상은 `IP:port`나 `host:port`, 헤더 이름은
유효해야 합니다. 변수나 생성기가 있는 필드는 실행될 때 검사합니다.

## 미리보기 {#preview}

선택한 노드에 템플릿이 있으면 속성에 지금 알려진 값으로 무엇을 할지 표시됩니다. 전송에는
[[ui:exp.preview]], 대기에는 [[ui:exp.previewWait]], 비교에는 [[ui:exp.previewCheck]]입니다.
엔진이 실행과 같은 코드로 해석하므로, 미리보기가 실행과 어긋나는 일은 없습니다.

- 매개변수는 활성 프로필에서 옵니다.
- 변수는 편집기가 이 세션에서 본 것, 즉 마지막 실행의 단계와 [[ui:exp.sendNow]]에서 옵니다.
- 저장된 시크릿은 `••••`로 표시됩니다.
- 아직 값이 없는 이름은 쓴 그대로 남고, 미리보기가 그 이름을 나열합니다. 저장되지 않은 시크릿은
  따로 나열됩니다.
- 생성기는 실험에 고정된 시드를 사용하고, 고정된 시드가 없으면 `0`을 노드의 첫 실행으로
  사용합니다. 시드가 고정되어 있으면 미리보기는 실행에서 노드의 첫 실행이 보낼 생성 값을 보여
  줍니다.

## 값 추출하기 {#extract}

[[ui:exp.node.extract]]는 그 경로의 최신 HTTP 응답에서 값을 하나 읽어 변수에 씁니다.

| 필드 | 내용 |
| --- | --- |
| [[ui:exp.variable]] | 쓸 변수. 매개변수의 이름 규칙이 적용됩니다 |
| [[ui:exp.extractFrom]] | 값이 어디서 오는지(아래) |
| [[ui:exp.jsonPath]], [[ui:exp.headerName]] 또는 [[ui:exp.pattern]] | 무엇을 읽을지, 출처에 따라 다름 |

| [[ui:exp.extractFrom]] | 읽는 것 | 값 |
| --- | --- | --- |
| [[ui:exp.from.json]] | 본문을 JSON으로, 경로에 따라 | JSON 값: 텍스트, 숫자, 객체, 배열 |
| [[ui:exp.from.header]] | 그 이름의 첫 헤더, 대소문자 무관 | 텍스트 |
| [[ui:exp.from.status]] | 상태 코드 | 숫자 |
| [[ui:exp.from.body]] | 본문 전체 | 텍스트 |
| [[ui:exp.from.regex]] | 본문의 첫 일치 | 패턴에 그룹이 있으면 캡처 그룹 1, 없으면 일치한 부분 전체 |

**JSON 경로.** `$.token`, `$.items[0].id`, `$["a b"]`, `$['a b']['c-d']`이며, 맨 앞의 `$.`는
생략할 수 있고(`token`, `items[0].id`), `$` 하나는 본문 전체입니다.

**정규식**은 Rust `regex` 엔진의 문법을 사용하며, 둘러보기와 역참조가 없습니다. 일치는 본문
어디서든 찾으며, 그것이 중요할 때는 `^`와 `$`로 고정하십시오.

다음 경우에 단계가 실패하며, 무엇이 없는지 알려 줍니다:

- 이 경로에서 그 앞에 HTTP 요청이 실행되지 않았을 때(`check.no_response`. 편집기는 그럴 수 없는
  그래프를 이미 거부합니다, `graph.needs_http`);
- 본문이 JSON이 아니거나 그 안에 경로가 없을 때;
- 헤더가 없거나 패턴이 일치하지 않을 때;
- JSON 경로나 본문 전체의 경우 응답이 보관하는 256 KiB를 본문이 넘을 때, 그리고 보관된 부분에서
  아무것도 일치하지 않은 패턴일 때(`extract.truncated`).

타임라인에는 쓴 값이 표시됩니다: `token = abc123`.

::: tip 클릭으로 추출하기
[[ui:exp.node.http]]에서 [[ui:exp.sendNow]]를 누르면 JSON 응답이 표시됩니다. 그 안의 값을
클릭하십시오. 요청 뒤에 [[ui:exp.node.extract]] 노드가 추가되고 경로가 채워지며 키에서 이름을
따옵니다. 값은 곧바로 미리보기에 알려집니다.
:::

## 변수 {#variables}

변수는 JSON 값을 담습니다. 다음 노드가 변수를 씁니다:

| 노드 | 쓰는 것 | 출력에서 |
| --- | --- | --- |
| [[ui:exp.node.extract]] | 추출한 값 | 그 출력 |
| [[ui:exp.node.wait_osc]], [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_http]], [[ui:exp.node.wait_ws]] | 도착한 것, 기본 이름 `reply`(HTTP는 `request`) | [[ui:exp.portMatched]]에만 |
| [[ui:exp.node.osc]], [[ui:exp.node.udp]]에 [[ui:exp.expectReply]] | 회신, 기본 이름 `reply` | 그 출력 |

대기가 쓰는 것은 객체이며, 이후 필드가 그 일부를 읽습니다:

| 대기 | 필드 |
| --- | --- |
| OSC | `address`, `args`, `from`, `ms` |
| UDP | `text`, `hex`, `bytes`, `from`, `ms`, 그리고 패턴이 있을 때의 `match` |
| MQTT | `topic`, 그리고 UDP의 필드 |
| WebSocket | UDP의 필드, 메시지가 JSON이면 `json` |
| HTTP 요청 | `method`, `path`, `query`, `headers`, `body`, `json`, `params`, `from`, `ms` |

`ms`는 분기의 마지막 동작부터 도착까지의 시간입니다. 정확한 내용은 [노드 참조](nodes.md)에
있습니다.

### 변수가 알려지는 곳 {#visibility}

변수는 그 변수를 쓰는 출력 이후부터, 그 출력을 지나는 경로에서 존재합니다:

- 대안 경로가 다시 만난 뒤 — 분기의 [[ui:exp.yes]]와 [[ui:exp.no]]가 합쳐질 때 — **모든** 경로가
  설정한 것만 알려집니다.
- [[ui:exp.node.join]] 뒤에는 그 합류로 들어오는 **어느** 분기가 설정한 것이든 알려집니다: 모두
  실행되었기 때문입니다.
- [[ui:exp.node.loop]]의 [[ui:exp.portDone]]이나 [[ui:exp.portLimit]] 뒤와 그 종료 조건에서는
  본문의 모든 반복이 설정한 것이 알려집니다.
- 대기의 변수는 그 [[ui:exp.portTimeout]] 출력 뒤에는 알려지지 않습니다.

각 병렬 분기는 변수의 자기 사본으로 동작합니다. 합류는 들어오는 연결 순서대로 사본을 병합하며,
둘 다 설정한 이름은 뒤의 연결이 이기므로, 결과가 어느 분기가 먼저 끝났는지에 달리는 일은 없습니다.
[실행의 흐름](flow.md#parallel)을 참고하십시오.

## 값 비교하기 {#compare}

[[ui:exp.node.assert_value]]는 비교가 성립하지 않으면 실행을 실패시킵니다.
[[ui:exp.node.branch_value]]는 [[ui:exp.yes]]나 [[ui:exp.no]]로 나갑니다. [[ui:exp.node.loop]]는
같은 비교를 종료 조건으로 사용합니다. 각각 [[ui:exp.value]], [[ui:exp.operator]],
[[ui:exp.expected]] 값이 있으며, 두 텍스트 모두 템플릿입니다:

| [[ui:exp.value]] | [[ui:exp.operator]] | [[ui:exp.expected]] |
| --- | --- | --- |
| `{{status}}` | [[ui:exp.op.lt]] | `300` |
| `{{reply.args[0]}}` | [[ui:exp.op.eq]] | `{{nonce}}` |

| [[ui:exp.operator]] | 성립 조건 |
| --- | --- |
| [[ui:exp.op.eq]], [[ui:exp.op.ne]] | 둘이 같음(다름) — 둘 다 숫자이면 숫자로(`200`은 `200.0`과 같음), 아니면 대소문자를 포함한 정확한 텍스트로 |
| [[ui:exp.op.lt]], [[ui:exp.op.le]], [[ui:exp.op.gt]], [[ui:exp.op.ge]] | 숫자로. 숫자가 아닌 쪽이 있으면 조용히 *아니요*가 아니라 단계를 실패시킵니다(`compare.not_numbers`) |
| [[ui:exp.op.contains]] | 값에 기대하는 텍스트가 포함됨(대소문자 포함) |
| [[ui:exp.op.matches]] | 기대하는 값의 정규식이 값 어디서든 일치함 |
| [[ui:exp.op.empty]], [[ui:exp.op.not_empty]] | 공백을 제거한 뒤 값이 비었거나 비지 않음. 기대하는 값은 사용하지 않음 |

숫자는 공백을 제거한 뒤 숫자로 읽히는 텍스트입니다: `42`, `-1.5`, `1e3`. 타임라인에는 이루어진
그대로의 비교가 표시됩니다: `401 = 200`. 양쪽은 120자로 자릅니다.

## 시크릿 {#secrets}

토큰이나 비밀번호는 `{{secret.NAME}}` 형태로 필드에 넣습니다. 실험 파일에는 이름만 남고, 값은
저장된 곳에 머물며 인터페이스에 도달하지 않습니다.

### 시크릿이 사는 곳 {#secret-stores}

| Signal Lab이 실행되는 곳 | 저장소 | 인터페이스에서 |
| --- | --- | --- |
| Windows 데스크톱 앱 | Windows 자격 증명 관리자, 서비스 `SignalLab` 아래 이름마다 항목 하나 | 설정, 교체, 제거 |
| Linux 데스크톱 앱 | 없음: 시크릿이 필요한 실행은 `secret.unsupported`로 실패합니다 | — |
| 서버 | 환경 변수 `SIGNALLAB_SECRET_<NAME>`, 없으면 시크릿 폴더의 파일 `<NAME>`, 따로 설정하지 않으면 `/run/secrets/signallab` | 읽기 전용 |
| 명령줄의 `signallab` | 서버와 같거나, `--secrets system`을 쓰면 Windows 자격 증명 관리자 | — |

[[ui:exp.secrets]] 항목의 제목에는 지금 있는 곳에 어느 것이 해당하는지 알려 주는 툴팁이 있습니다.
Windows 저장소인지, 서버의 환경과 파일인지, 아니면 — Linux 데스크톱 앱에서는 — 저장소가 없다는
것입니다. 후자의 경우 `secret.unsupported`는 시크릿이 Windows 자격 증명 관리자에 보관되는데 그
시스템에는 그것이 없다고 말합니다.

시크릿은 실험 하나가 아니라 컴퓨터나 서버에 속합니다. `{{secret.API_TOKEN}}`을 쓰는 두 실험은 같은
값을 사용합니다.

서버에서는 환경 변수가 파일보다 우선합니다. 파일 끝의 줄 바꿈은 값의 일부가 아니며, 빈 파일은
시크릿이 없는 것으로 봅니다. 서버의 폴더는 `--secrets-dir`나 `SIGNALLAB_SECRETS_DIR`로 설정합니다.
[서버](../server/index.md)를 참고하십시오. 명령줄은
[`signallab run`](../automation/cli.md#cli-run)을 참고하십시오.

| 규칙 | 한도 |
| --- | --- |
| 이름 | 글자나 `_`로 시작하고, 그다음은 글자, 숫자, `_`이며 최대 128자 |
| 값 | 비어 있지 않고, 최대 16 KiB |

### 시크릿 설정하기 {#set-secret}

Windows에서는:

1. [[ui:exp.params]]를 엽니다. [[ui:exp.secrets]] 항목에는 실험의 필드가 사용하는 모든 시크릿이
   나열되며, 각각 [[ui:exp.secretStored]] 또는 [[ui:exp.secretMissing]]입니다.
2. 이름 옆의 [[ui:exp.secretSet]]을 누르거나, 아직 어떤 필드도 쓰지 않는 이름에는
   [[ui:exp.addSecret]]을 누릅니다.
3. 값을 입력하고(필드에는 점이 표시됨) [[ui:exp.secretSave]]나 <kbd>Enter</kbd>를 누릅니다. 필드가
   비워지며, 아무것도 값을 다시 읽을 수 없습니다.

[[ui:exp.secretReplace]]는 새 값을 저장하고, [[ui:exp.secretRemove]]는 자격 증명 저장소에서
삭제합니다. 이 세션에서 저장한 이름은 제안에도 나타납니다.

서버에 연결된 브라우저에서는 이 항목이 [[ui:exp.secretOnServer]] 또는
[[ui:exp.secretNotOnServer]]만 알려 줍니다. 서버가 실행되는 곳에서 두 가지 방법 중 하나로 값을
설정하십시오:

```bash
# in the server's environment
SIGNALLAB_SECRET_API_TOKEN='…'
# or as a file in its secrets folder
printf '%s' '…' > /run/secrets/signallab/API_TOKEN
```

파일은 실행이 시작될 때마다 읽으므로, 바뀐 파일은 다음 실행부터 반영됩니다. 바뀐 환경 변수는
서버를 다시 시작해야 합니다.

### 실행 전에 {#secret-check}

실행의 필드가 사용하는 모든 시크릿이 저장되어 있어야 합니다. 없는 시크릿은 트래픽이 생기기 전에,
그 시크릿을 사용하는 첫 노드와 필드에서 실행을 멈춥니다(`secret.missing`). [[ui:exp.sendNow]]도
자기 노드에 대해 같은 검사를 합니다.

### 마스킹 {#masking}

실행이나 [[ui:exp.sendNow]]가 시크릿을 사용하는 동안에는 엔진을 벗어나는 모든 것에서 그 값의 모든
출현이 `••••`로 바뀝니다:

- 단계 텍스트, 오류, 단계가 쓴 변수;
- 실행 보고서;
- [[ui:exp.sendNow]]의 결과(표시하는 HTTP 응답 포함);
- 실행이 지속되는 동안 캡처한 [[ui:dock.inspector]]의 프레임 — hex 덤프에서는 값의 각 바이트가
  `*`가 되어 오프셋이 그대로 유지됩니다.

Basic 인증은 `name:password`를 base64로 보냅니다. 어느 쪽이든 시크릿을 담고 있으면 그 base64
텍스트도 마스킹됩니다. 트래픽 자체는 실제 값을 나릅니다. 미리보기는 저장된 시크릿을 `••••`로
표시합니다. [[ui:exp.node.emulator]]의 회신은 시크릿을 사용할 수 없습니다.

## 명령 {#commands}

미리보기는 [`experiment_resolve`](../api/commands.md#experiment_resolve)입니다. 시크릿은
[`secret_status`](../api/commands.md#secret_status),
[`secret_set`](../api/commands.md#secret_set),
[`secret_delete`](../api/commands.md#secret_delete)로 나열, 설정, 제거합니다. 어떤 명령도 시크릿의
값을 반환하지 않습니다.

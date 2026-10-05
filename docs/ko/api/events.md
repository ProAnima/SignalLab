---
title: 이벤트
description: Signal Lab 서버에서 엔진 이벤트가 오는 WebSocket, 그리고 각 채널의 페이로드와 전송 시점입니다.
---

# 이벤트

작업이 실행되는 동안 일어나는 모든 것, 즉 실행의 단계, 모니터의 메시지, 버스트의 숫자,
인스펙터의 프레임, 작업의 끝은 이벤트로 전송됩니다. 브라우저에서는 서버의 페이지가 하나의
WebSocket `/api/events`로 이들을 받습니다. 스크립트도 같은 소켓을 들을 수 있습니다. 데스크톱
앱은 같은 이름과 페이로드를 가진 같은 이벤트를 앱 안에서 받습니다.

## 구독하기 {#subscribe}

서버의 `/api/events`에 WebSocket을 엽니다.

```bash
websocat -H "Authorization: Bearer $TOKEN" ws://127.0.0.1:1430/api/events
```

- **인증**은 나머지 API와 같습니다. `Authorization: Bearer`로 보내는 토큰, 또는 브라우저의
  세션 쿠키입니다. 없으면 업그레이드가 `401` `auth.required`로 거부됩니다.
- **Origin**: `Origin` 헤더를 보내는 클라이언트는 서버 자체의 것(호스트와 포트가 `Host`와
  같음)을 보내야 합니다. 그렇지 않으면 업그레이드가 `403` `auth.origin`으로 거부됩니다.
  브라우저 밖의 대부분의 WebSocket 라이브러리는 보내지 않습니다.
- **모든 클라이언트에 모든 이벤트.** 구독할 대상이 따로 없습니다. 각 소켓은 누가 시작했든
  모든 작업의 모든 이벤트를 받습니다. 필요한 것은 `event`와 페이로드의 `job_id`로
  고르십시오.
- **듣기만 함.** 서버는 닫기를 제외하고 클라이언트가 보내는 것을 무시합니다. 64 KiB보다 큰
  메시지는 소켓을 닫습니다.
- **연결 유지.** 서버는 20초마다 핑을 보내므로 조용한 소켓도 프록시를 통과해 열린 상태로
  유지됩니다. 서버가 멈추면 모든 소켓을 닫습니다.
- **재생되는 것은 없음.** 클라이언트가 연결되어 있지 않을 때 보낸 이벤트는 그 클라이언트에게
  사라집니다. 다시 연결한 클라이언트는 명령(`jobs_list`, `inspect_snapshot`,
  `emulator_exchanges`…)으로 현재 상태를 읽어야 합니다.
- **뒤처짐.** 소켓 하나에 최대 4096개의 이벤트가 대기합니다. 그보다 더 뒤처지는 클라이언트는
  얼마나 놓쳤는지와 함께 [`server://lagged`](#event-server-lagged)를 받습니다.

## 메시지 형식 {#format}

각 이벤트는 JSON 객체 하나를 담은 텍스트 메시지 하나입니다.

```json
{ "event": "scan://open", "payload": { "job_id": 9, "ts": 1759600000123, "port": 8080, "banner": null } }
```

| 필드 | 내용 |
| --- | --- |
| `event` | 아래의 채널 |
| `payload` | 이벤트의 값들. 형태는 채널에 따라 다릅니다 |

시각(`ts`, 피어의 `first_ms`, `last_ms`)은 1970년 이후 밀리초입니다. 지연 시간과 그 밖의
기간(`*_latency_ms`, `p50_ms`…, `ms`)은 밀리초입니다. 페이로드의 오류는
[`EngineError`](index.md#errors) 객체이며, 그 코드는
[오류 메시지](../reference/errors.md)에 나열됩니다.

## 채널 {#channels}

| 채널 | 보내는 곳 | 언제 |
| --- | --- | --- |
| [`experiment://step`](#event-experiment-step) | 실행 | 단계가 시작, 통과, 실패, 재시도, 반복되거나 부하를 보고할 때 |
| [`experiment://ended`](#event-experiment-ended) | 실행 | 실행이 스스로 끝났을 때 한 번 |
| [`job://ended`](#event-job-ended) | 모든 작업 | 작업이 스스로 끝나거나 실패했을 때 한 번 |
| [`osc://message`](#event-osc-message) | OSC 모니터 | 패킷마다 |
| [`osc://gen-tick`](#event-osc-gen-tick) | OSC 발생기 | 메시지마다, 초당 60개를 넘으면 초당 30~45번 |
| [`http://burst-progress`](#event-http-burst-progress) | HTTP 버스트 | 100 ms마다, 그리고 끝날 때 |
| [`ws://state`](#event-ws-state) | WebSocket 연결 | 연결됨, 닫힘 |
| [`ws://messages`](#event-ws-messages) | WebSocket 연결 | 새로운 것이 있을 때 100 ms마다 |
| [`mqtt://state`](#event-mqtt-state) | MQTT 연결 | 연결됨, 구독됨, 닫힘 |
| [`mqtt://messages`](#event-mqtt-messages) | MQTT 연결 | 새로운 것이 있을 때 100 ms마다 |
| [`mqtt://ack`](#event-mqtt-ack) | MQTT 연결 | QoS 1/2 발행이 완료됨. 구독 취소가 응답됨 |
| [`broadcast://emit-stat`](#event-broadcast-emit-stat) | 비컨 | 250 ms마다, 그리고 끝날 때 |
| [`broadcast://peers`](#event-broadcast-peers) | 디스커버리 리스너 | 400 ms마다 |
| [`netsim://stat`](#event-netsim-stat) | 장애 릴레이 | 250 ms마다 |
| [`storm://stat`](#event-storm-stat) | 스톰 | 250 ms마다, 그리고 끝날 때 |
| [`scan://open`](#event-scan-open) | 스캐너 | 열린 포트마다 |
| [`scan://progress`](#event-scan-progress) | 스캐너 | 범위의 약 1%마다, 그리고 끝날 때 |
| [`emulator://activity`](#event-emulator-activity) | 에뮬레이터 작업 | 새로운 것이 있을 때 200 ms마다 |
| [`inspect://batch`](#event-inspect-batch) | 인스펙터 | 캡처가 켜져 있는 동안 새 프레임이 있으면 120 ms마다, 조용하면 약 1초에 한 번 |
| [`server://lagged`](#event-server-lagged) | 서버 | 클라이언트가 뒤처졌을 때 |

### `experiment://step` {#event-experiment-step}

실행의 한 단계입니다. 노드가 시작, 통과, 실패, 다시 시도하기 위해 대기, 반복하거나 부하의
진행 상황을 보고하는 것입니다. `/api/run`으로 시작한 실행은 같은 단계를 응답으로 보냅니다
([실행](run.md) 참고).

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 실행의 작업 |
| `ts` | number | 시각 |
| `node_id` | string | 노드 |
| `state` | string | `running`, `passed`, `failed`, `retry`(시도가 실패해 잠시 뒤 단계가 다시 실행됨), `repeating`(반복 동작의 진행 상황, 초당 최대 한 번) 또는 `load`(부하의 진행 상황, 초당 최대 한 번) |
| `detail` | string | 무슨 일이 있었는지, 영어. `running`과 `failed`에서는 비어 있음(`error` 참고) |
| `message_key` | string 또는 null | 그것에 대한 인터페이스의 텍스트, 사전의 키로 |
| `message_params` | object 또는 null | `message_key`가 가리키는 값들 |
| `vars` | object | 단계가 쓴 변수. 없으면 빠짐 |
| `error` | `EngineError` | 실패한 이유, 또는 시도가 실패한 이유(`retry`). 아니면 빠짐 |
| `frame` | number | 캡처가 켜져 있었을 때 대기(또는 전송의 예상 회신)가 일치시킨 메시지의 인스펙터 프레임. 아니면 빠짐 |
| `load` | object | 부하가 측정한 것과 읽은 임계값 — 부하 단계의 마지막 이벤트에, 통과 또는 실패와 함께. 아니면 빠짐. [부하](../experiments/load.md) 참고 |

실행의 [[ui:exp.node.end]] 노드는 첫 분기가 도달하면 `running`을, 모든 분기가 실패 없이
끝나면 `passed`를 표시합니다. 시크릿 값은 모든 필드에서 가려집니다.

### `experiment://ended` {#event-experiment-ended}

실행이 스스로 끝났습니다. 통과, 실패 또는 시간 초과입니다.
[`job://ended`](#event-job-ended)의 같은 페이로드 바로 뒤에 전송됩니다. `job_stop` 또는
[[ui:app.stopAll]]로 중지한 실행은 둘 다 보내지 않고 보고서도 저장하지 않습니다.

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 실행의 작업 |
| `kind` | string | `experiment` |
| `seed` | number | 실행한 시드 |
| `profile` | string 또는 null | 프로필 |
| `overridden` | boolean | 일부 매개변수 값이 [[ui:exp.runWith]] 또는 `overrides`에서 왔음 |
| `error` | `EngineError` 또는 null | 실행의 첫 실패. 통과했으면 null |
| `report_path` | string 또는 null | 실행의 보고서, 데이터 폴더의 `runs/`에 |
| `report_error` | `EngineError` 또는 null | 보고서를 쓰지 못한 이유 |

### `job://ended` {#event-job-ended}

작업이 스스로 끝나거나 실패했습니다. `job_stop` 또는 `jobs_stop_all`로 중지한 작업은
이것을 보내지 않습니다.

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 작업 |
| `kind` | string | `osc-monitor`, `osc-gen`, `http-burst`, `netsim`, `storm`, `scan`, `beacon`, `discovery`, `mqtt`, `websocket`, `emulator` 또는 `experiment` |
| `error` | `EngineError` 또는 null | 무엇인가 잘못되었을 때 끝난 이유 |

실행의 `job://ended`는 [`experiment://ended`](#event-experiment-ended)의 필드도 함께
담습니다. 각 종류가 끝나는 때는 다음과 같습니다.

| `kind` | 끝나는 때 | `error` |
| --- | --- | --- |
| `osc-monitor` | 소켓이 더 이상 수신할 수 없을 때 | `wait.receive_failed` |
| `osc-gen` | 지속 시간이 끝나거나 전송이 실패할 때 | null, 또는 `transport.*` |
| `http-burst` | 총 횟수나 지속 시간에 도달했을 때 | null |
| `storm` | 지속 시간이 끝났을 때 | null |
| `scan` | 범위의 모든 포트를 시도했을 때 | null |
| `beacon` | 라운드나 지속 시간이 끝났거나, 32개 넘게 전송이 실패하고 하나도 나가지 않았을 때 | null, 또는 `transport.*` |
| `discovery` | 소켓이 더 이상 수신할 수 없을 때 | `wait.receive_failed` |
| `mqtt` | 브로커가 연결을 닫았거나 연결이 끊겼을 때 | `transport.*`(브로커가 닫았으면 `transport.reset`), 또는 `mqtt.protocol` |
| `websocket` | 연결이 닫혔을 때 | null, 또는 끊긴 이유 |
| `netsim` | 릴레이가 더 이상 동작할 수 없을 때 | 그 이유 |
| `emulator` | 소켓이 실패할 때 | 그 이유 |
| `experiment` | 실행이 끝날 때 | 실행의 실패, 또는 null |

### `osc://message` {#event-osc-message}

OSC 모니터가 받은 UDP 패킷 하나를 디코딩한 것입니다. 묶지 않고 패킷마다 전송됩니다.

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 모니터의 작업 |
| `ts` | number | 도착한 시각 |
| `from` | string | 보낸 쪽, `IP:port` |
| `bytes` | number | 패킷의 크기 |
| `messages` | object[] | 패킷의 각 메시지(번들은 여러 개): `address`와 `args` ([`OscArg`](commands.md#type-oscarg)`[]`) |
| `error` | `EngineError` 또는 null | 패킷이 디코딩되지 않았을 때 `osc.packet_malformed`(이때 `messages`는 비어 있음) |

### `osc://gen-tick` {#event-osc-gen-tick}

OSC 발생기의 진행 상황입니다. 초당 60개 미만이면 메시지마다, 그 이상이면 n번째마다이며,
n은 속도를 30으로 나눈 값의 내림입니다. 즉 초당 30~45번입니다.

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 발생기의 작업 |
| `ts` | number | 시각 |
| `value` | number | 방금 보낸 값, 정수나 32비트 float로 반올림되기 전 |
| `sent` | number | 지금까지 보낸 메시지 |

### `http://burst-progress` {#event-http-burst-progress}

HTTP 버스트의 숫자입니다. 실행되는 동안 100 ms마다, 스스로 끝나면 `done: true`와 함께 한 번
더 전송됩니다.

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 버스트의 작업 |
| `ts` | number | 시각 |
| `sent` | number | 지금까지 응답받거나 실패한 요청 |
| `ok` | number | 그중 2xx 상태로 응답받은 요청 |
| `failed` | number | 그중 그 밖의 상태이거나 응답이 없는 요청 |
| `missed` | number | 속도를 맞춘 버스트에서 빈 워커를 너무 오래 기다려 건너뛴 요청 |
| `rps` | number | 지난 100 ms 동안의 초당 요청. 마지막 이벤트에서는 버스트 전체 기준 |
| `last_latency_ms`, `min_latency_ms`, `max_latency_ms`, `avg_latency_ms` | number | 지금까지의 지연 시간 |
| `p50_ms`, `p90_ms`, `p95_ms`, `p99_ms` | number | 지금까지의 모든 요청(실패 포함)의 백분위수, 0.5% 이내 |
| `done` | boolean | 버스트의 마지막 이벤트 |

### `ws://state` {#event-ws-state}

`ws_connect`로 연 WebSocket 연결이 연결되었거나 닫혔습니다. 작업이 중지된 연결은 `closed`를
보내지 않습니다.

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 연결의 작업 |
| `ts` | number | 시각 |
| `state` | string | `connected` 또는 `closed` |
| `handshake` | object | `url`, `peer`, `local`, `protocol`(서버가 고른 서브프로토콜, 또는 null), `ms`(연결과 업그레이드) |
| `closed` | object 또는 null | `closed`일 때: `code`, `reason`, `by`(`client`, `server`, `lost`)와 `error` |

### `ws://messages` {#event-ws-messages}

WebSocket 연결이 마지막 이벤트 이후 보내고 받은 것입니다. 뭔가 있을 때 100 ms마다
전송됩니다.

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 연결의 작업 |
| `ts` | number | 시각 |
| `messages` | object[] | 순서대로: `ts`, `dir`(`rx` 받음, `tx` 보냄), `kind`(`text` 또는 `binary`), `text`(처음 64 KiB를 UTF-8로, 바이너리 메시지도 마찬가지. 아닌 바이트는 `�`가 됨), `hex`(바이너리 메시지의 처음 4096바이트를 hex로, 아니면 null), `bytes`(전체 크기), `truncated`(보여 준 것보다 많음: 텍스트 64 KiB 초과, 바이너리 4096바이트 초과) |
| `dropped` | number | 2000개를 넘어 이 이벤트에서 빠진 메시지. 가장 오래된 것부터 |

### `mqtt://state` {#event-mqtt-state}

MQTT 연결의 상태가 바뀌었습니다.

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 연결의 작업 |
| `ts` | number | 시각 |
| `state` | string | `connected`. 구독에 대한 응답마다 `subscribed`. 연결이 끝났을 때 `closed`(작업이 중지된 때는 아님) |
| `broker` | string | `host:port` |
| `error` | `EngineError` 또는 null | `closed` 연결이 끝난 이유(브로커가 닫았으면 `transport.reset`). 아니면 null |
| `grants` | object[] | `subscribed`일 때: 요청한 각 필터와 `filter`, `qos`(허용됨), `accepted`. 아니면 빈 배열 |

### `mqtt://messages` {#event-mqtt-messages}

MQTT 연결이 마지막 이벤트 이후 받은 것입니다. 뭔가 있을 때 100 ms마다 전송됩니다. 다시
전달된 QoS 2 메시지는 한 번만 표시됩니다.

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 연결의 작업 |
| `ts` | number | 시각 |
| `messages` | object[] | `ts`, `topic`, `payload`(UTF-8로. 아닌 바이트는 `�`가 됨), `bytes`, `qos`, `retain`, `dup` |
| `dropped` | number | 100 ms 동안 4000개가 넘게 도착해 빠진 메시지. 가장 오래된 것부터 |

### `mqtt://ack` {#event-mqtt-ack}

브로커가 연결이 요청한 무언가를 완료했습니다.

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 연결의 작업 |
| `ts` | number | 시각 |
| `kind` | string | `published`(QoS 1 또는 2 발행이 완료됨) 또는 `unsubscribed` |
| `packet_id` | number | MQTT 패킷 id |
| `topic` | string 또는 null | 발행한 토픽. `unsubscribed`이면 null |

### `broadcast://emit-stat` {#event-broadcast-emit-stat}

비컨의 카운터입니다. 250 ms마다, 스스로 끝나면 `pps` 0과 함께 한 번 더 전송됩니다.

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 비컨의 작업 |
| `ts` | number | 시각 |
| `targets` | number | 라운드마다의 목적지 |
| `rounds` | number | 보낸 라운드 |
| `packets`, `bytes` | number | 보낸 데이터그램과 바이트 |
| `errors` | number | 실패한 전송 |
| `pps` | number | 지난 250 ms 동안의 초당 데이터그램 |

### `broadcast://peers` {#event-broadcast-peers}

디스커버리 리스너가 들은 것입니다. 400 ms마다 전송됩니다.

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 리스너의 작업 |
| `ts` | number | 시각 |
| `peers` | object[] | 가장 최근에 들은 것부터: `addr`, `proto`, `packets`, `bytes`, `first_ms`, `last_ms`, `last_summary`, `responded`(응답한 패킷, 도착하는 대로 집계). 최대 512개 |
| `packets`, `bytes` | number | 받은 모든 것 |
| `responses` | number | 보낸 응답 |

### `netsim://stat` {#event-netsim-stat}

장애 릴레이의 카운터입니다. 250 ms마다 전송됩니다. 실행의
[[ui:exp.node.impairment]] 노드의 릴레이는 대신 실행의 보고서에 보고합니다.

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 릴레이의 작업 |
| `ts` | number | 시각 |
| `received`, `forwarded` | number | 들어오고 나간 데이터그램 또는 청크 |
| `dropped` | number | `loss`, 버스트 또는 `offline`으로 잃은 것(UDP. TCP 릴레이는 오프라인 동안 스트림을 붙잡아 두고 아무것도 버리지 않음) |
| `throttled` | number | UDP: 대역폭 제한으로, 또는 이미 너무 많이 가는 중이어서 버려진 것. TCP: 대역폭 제한 때문에 스트림을 붙잡아 둔 청크 |
| `duplicated`, `corrupted`, `reordered` | number | 프로필이 그것들에 한 일 |
| `bytes` | number | 전달한 바이트 |
| `connections`, `reset`, `stalled` | number | TCP: 받은 연결, 리셋한 연결, 반쯤 열어 둔 연결. 0이면 빠짐 |
| `profile` | string | 지금 장애를 주는 프로필, 타임라인이 부르는 대로: 이름, 또는 하는 일(`60 ms ±25 · loss 2%`) |

### `storm://stat` {#event-storm-stat}

스톰의 카운터입니다. 250 ms마다, 스스로 끝나면 `pps`와 `mbps` 0과 함께 한 번 더
전송됩니다.

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 스톰의 작업 |
| `ts` | number | 시각 |
| `packets`, `bytes` | number | 보낸 데이터그램(또는 TCP 연결)과 바이트 |
| `errors` | number | 실패한 전송 또는 연결 |
| `pps` | number | 지난 250 ms 동안의 초당 |
| `mbps` | number | 지난 250 ms 동안의 초당 메가비트 |

### `scan://open` {#event-scan-open}

스캐너가 열린 포트를 찾았습니다.

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 스캔의 작업 |
| `ts` | number | 시각 |
| `port` | number | 포트 |
| `banner` | string 또는 null | 배너를 요청했고 서비스가 400 ms 안에 무언가를 보냈을 때, 서비스가 처음 보낸 것 |

### `scan://progress` {#event-scan-progress}

스캔이 얼마나 진행되었는지입니다. 범위의 약 1%마다, 그리고 스스로 끝나면 `done`이 `total`과
같은 값으로 전송됩니다(이것은 두 번 도착할 수 있습니다).

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 스캔의 작업 |
| `ts` | number | 시각 |
| `done` | number | 시도한 포트 |
| `total` | number | 범위의 포트 |
| `open` | number | 찾은 열린 포트 |

### `emulator://activity` {#event-emulator-activity}

`emulator_start`로 시작한 에뮬레이터가 마지막 이벤트 이후 받고 응답한 것입니다. 무언가
바뀌었을 때(교환, 다운되거나 복구됨, 또는 MQTT 브로커가 전달하지 못한 메시지) 200 ms마다
전송됩니다. 실행의 [[ui:exp.node.emulator]] 노드는 이것을 보내지 않습니다. 그 카운터는 실행의
보고서에 있습니다.

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `job_id` | number | 에뮬레이터의 작업 |
| `ts` | number | 시각 |
| `counts` | object | `total`, `unmatched`, `failed`, `down`, `hits`(규칙별), `missed`(MQTT. 0이면 빠짐) — [`emulator_exchanges`](commands.md#emulator_exchanges)처럼 |
| `forced` | string | 다운된 동안 `unavailable`, `reset` 또는 `timeout`. 아니면 빠짐 |
| `exchanges` | object[] | 새 교환들, `emulator_exchanges`가 나열하는 대로지만 `data`는 없음. 최대 200개 |
| `dropped` | number | 그 간격의 처음 200개를 넘어 여기 보내지 않은 교환. `emulator_exchanges`에는 여전히 마지막 500개가 있음 |

### `inspect://batch` {#event-inspect-batch}

새 인스펙터 프레임입니다. 캡처가 켜져 있는 동안에만 전송됩니다. 새 프레임이 있으면 120 ms마다,
없으면 약 1초에 한 번 전송되어 카운터가 최신으로 유지됩니다.

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `frames` | object[] | 새 [프레임](commands.md#type-frame), 가장 오래된 것부터, 최대 250개. 바이트는 없음(`inspect_payload` 사용) |
| `stats` | object | 캡처의 카운터, [`CaptureStats`](commands.md#type-frame) |
| `skipped_now` | number | 마지막 배치 이후 캡처되었지만 이번 배치에 없는 프레임 — 250개가 넘게 도착했거나 버퍼가 놓아 준 것. 버퍼가 붙잡고 있는 동안에는 내보내기에 여전히 들어 있음 |

### `server://lagged` {#event-server-lagged}

서버 전용입니다. 이 클라이언트가 4096개가 넘는 이벤트만큼 뒤처져 일부를 놓쳤습니다. 명령으로
상태를 다시 읽으십시오.

| 필드 | 형식 | 의미 |
| --- | --- | --- |
| `skipped` | number | 놓친 이벤트의 수 |

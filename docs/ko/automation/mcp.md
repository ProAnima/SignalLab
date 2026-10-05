---
title: 어시스턴트(MCP)
description: signallab mcp를 사용하면 AI 어시스턴트가 Model Context Protocol을 통해 실험을 만들고, 검사하고, 실행하고, 메시지를 보내고, 수신하고, 에뮬레이터를 운영할 수 있습니다.
---

# 어시스턴트를 위한 Signal Lab: `signallab mcp`

`signallab mcp`는 [Model Context Protocol](https://modelcontextprotocol.io) 서버입니다. Claude Code,
Claude Desktop, Cursor, VS Code 또는 다른 MCP 클라이언트의 어시스턴트가 이 서버를 시작하면 다음을 할
수 있습니다.

- 실험이 무엇으로 구성되는지 파악하고, 실험을 작성하고, 검사하고, 실행하고, 실패했다면 그 이유를
  단계별로 읽습니다.
- OSC 메시지, 데이터그램, HTTP 요청, WebSocket 메시지, MQTT 발행을 하나씩 보내고, 장치가 보내는
  데이터를 포트에서 수신합니다.
- 라이브러리의 신호를 보냅니다.
- 의존 대상(HTTP API, OSC·UDP·TCP 장치, MQTT 브로커)의 역할을 맡고, 시스템이 보낸 내용을
  읽습니다.
- 이전 실행을 다시 읽고 두 실행을 비교합니다.

모든 동작은 앱이 사용하는 것과 같은 엔진 명령을 거칩니다. 따라서 어시스턴트가 실행한 실험은 앱이
실행했을 때와 같은 실행이며 보고서도 같고, 모든 실패는 인터페이스가 표현하는 방식 그대로 표현됩니다.

::: warning
전송, 실행, 에뮬레이터는 실제 트래픽을 네트워크에 보냅니다. 어시스턴트에게 어느 장치와 통신해도
되는지 알려 주십시오. 기본 제공 템플릿은 루프백(`127.0.0.1`)을 가리킵니다.
:::

## 설정하기 {#setup}

`signallab`은 데스크톱 앱과 함께 설치되며, 설치하면 `PATH`에 있습니다([설치](cli.md#install) 참고).
클라이언트가 `signallab mcp`를 직접 시작하고 표준 입력과 표준 출력으로 통신하므로, 직접 실행할
필요는 없습니다.

### 설정 출력하기 {#print-config}

`--print-config`는 클라이언트에 필요한 설정을 이 `signallab`의 전체 경로와 함께 출력합니다.

| 명령 | 출력 내용 |
| --- | --- |
| `signallab mcp --print-config claude-code` | `claude mcp add` 명령줄. |
| `signallab mcp --print-config claude-desktop` | Claude Desktop 설정 파일에 넣을 `mcpServers` 항목. |
| `signallab mcp --print-config cursor` | Cursor의 `mcp.json`에 넣을 같은 `mcpServers` 항목. |
| `signallab mcp --print-config vscode` | VS Code의 `.vscode/mcp.json`에 넣을 `servers` 항목. |

[랩 서버](#on-a-server)에서 작업하는 어시스턴트라면 `--server URL`을 추가하십시오. 출력되는 설정에
이 값이 들어가고, 토큰 자리에는 자리 표시자가 들어갑니다.

### Claude Code {#claude-code}

`--print-config claude-code`가 출력하는 줄을 실행하십시오. 예를 들면 다음과 같습니다.

```bash
claude mcp add signallab -- "C:\Program Files\Signal Lab\signallab.exe" mcp
```

### Claude Desktop과 Cursor {#claude-desktop}

항목을 클라이언트의 설정에 넣고(Claude Desktop은 `claude_desktop_config.json`, Cursor는
`mcp.json`) 클라이언트를 다시 시작하십시오.

```json
{
  "mcpServers": {
    "signallab": {
      "command": "C:\\Program Files\\Signal Lab\\signallab.exe",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

### VS Code {#vscode}

```json
{
  "servers": {
    "signallab": {
      "type": "stdio",
      "command": "/usr/bin/signallab",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

### 다른 클라이언트 {#other-clients}

stdio 서버를 시작하는 클라이언트라면 모두 같은 방식으로 동작합니다. 명령은 `signallab`(또는 그 전체
경로), 인수는 `mcp`와 [옵션](#options) 중 필요한 것입니다. Linux에서는 서버 이미지를 명령으로 쓸
수도 있습니다.

```bash
docker run -i --rm --network host --entrypoint signallab ghcr.io/proanima/signallab:[[version]] mcp
```

## 옵션 {#options}

| 옵션 | 기능 | 기본값 |
| --- | --- | --- |
| `--server URL` | 실험, 전송, 에뮬레이터를 이 Signal Lab 서버에서 실행합니다([랩 서버에서](#on-a-server) 참고). | `SIGNALLAB_SERVER` |
| `--token-file PATH` | 서버의 토큰이 들어 있는 파일. | `SIGNALLAB_TOKEN_FILE`, 없으면 `SIGNALLAB_TOKEN` |
| `--data-dir PATH` | 실행과 그 보고서를 보관할 위치. `--server`와 함께 쓸 수 없습니다. | 앱의 데이터 폴더(`Documents/SignalLab`) |
| `--library PATH` | `list_signals`와 `fire_signal`이 쓰는 신호 라이브러리. | 앱의 `signals.json` |
| `--emulators PATH` | `list_emulators`와 `start_emulator`가 쓰는 에뮬레이터 라이브러리. | 앱의 `emulators.json` |
| `--secrets files\|system` | 이 컴퓨터에서 하는 실행에서 시크릿 값을 가져오는 곳으로, [`run`](cli.md#secrets)과 같습니다. `--server`와 함께 쓸 수 없습니다. | `files` |
| `--secrets-dir PATH` | 이름마다 파일 하나씩 있는 시크릿 파일 폴더. `--server`와 함께 쓸 수 없습니다. | 있으면 `/run/secrets/signallab` |
| `--lang <code>` | 결과와 실패 메시지의 언어. | `SIGNALLAB_LANG`, 없으면 로캘, 그것도 없으면 `en` |
| `--print-config CLIENT` | 클라이언트의 설정을 출력하고 종료합니다: `claude-code`, `claude-desktop`, `cursor`, `vscode`. | |

실행은 앱 자체의 보고서가 있는 앱의 데이터 폴더에 보고서를 보관하므로, 세션이 끝난 뒤에도 남아
있습니다.

## 도구 {#tools}

읽기만 하는 도구는 읽기 전용으로 표시되므로 클라이언트가 묻지 않고 실행하게 할 수 있습니다. 보내거나,
수신하거나, 무언가를 시작하는 등 외부 세계에 닿는 도구는 그렇게 표시되며, 클라이언트가 호출할 때마다
사용자에게 물을 수 있습니다. 파괴적인 것으로 표시된 도구는 없습니다.

| 도구 | 기능 | 외부 세계에 닿음 |
| --- | --- | --- |
| `describe_nodes` | 실험 문서, 필드·출력·예제를 갖춘 모든 종류의 노드, `{{template}}` 언어, 부하 프로필, 에뮬레이터 문서. | 아니요 |
| `list_templates` | 기본 제공 실험과 그 매개변수. | 아니요 |
| `get_template` | 기본 제공 실험 하나를 문서로. | 아니요 |
| `validate_experiment` | 실행 전에 편집기가 하는 것처럼 실험을 검사합니다. 아무것도 보내지 않습니다. | 아니요 |
| `run_experiment` | 실험을 끝까지 실행하고 모든 단계를 보고합니다. | 예 |
| `send_osc` | OSC 메시지 하나. | 예 |
| `send_udp` | UDP 데이터그램 하나. | 예 |
| `send_http` | HTTP 요청 하나. | 예 |
| `send_mqtt` | MQTT 3.1.1 발행 하나. | 예 |
| `send_ws` | WebSocket 교환 하나. | 예 |
| `listen` | 일정 시간 동안 UDP 포트에 도착하는 것. | 예 |
| `list_signals` | 라이브러리의 신호. | 아니요 |
| `fire_signal` | 라이브러리의 신호를 보냅니다. | 예 |
| `list_emulators` | 라이브러리의 에뮬레이터. | 아니요 |
| `start_emulator` | 에뮬레이터를 시작합니다. | 예 |
| `emulator_exchanges` | 실행 중인 에뮬레이터가 받은 것과 응답한 것. | 아니요 |
| `set_emulator_down` | 실행 중인 에뮬레이터를 다운시키거나 복구합니다. | 예 |
| `list_runs` | 이전 실행의 보고서. | 아니요 |
| `compare_runs` | 두 실행을 나란히. | 아니요 |
| `list_jobs` | 실행 중인 것. | 아니요 |
| `stop_job` | 실행 중인 작업을 중지합니다. | 예 |

### 실험 {#tools-experiments}

`describe_nodes`는 어시스턴트가 실험을 작성하기 전에 읽는 도구이며,
[`signallab nodes`](cli.md#cli-nodes)와 같습니다. `list_templates`와 `get_template`은 실행하거나
고쳐 쓸 수 있는 동작하는 예제를 제공합니다.

`validate_experiment`와 `run_experiment`는 실험을 세 가지 방법 중 정확히 하나로 받습니다.

| 인수 | 내용 |
| --- | --- |
| `document` | 앱이 저장하는 형식의 실험 문서. |
| `file` | `signallab`이 실행되는 컴퓨터에 있는 실험 파일의 경로. |
| `template` | 기본 제공 템플릿의 이름. |
| `params` | 이번 실행의 매개변수 값: `{"name": "value"}`. 숫자와 불리언은 텍스트로 받습니다. |
| `profile` | 문서의 이 프로필로 실행합니다. 기본값을 쓰려면 `""`. |
| `seed` | `run_experiment`: 임의 값의 시드. |
| `timeout` | `run_experiment`: 실행에 허용되는 시간(초), 1~300(기본값 300). |

`run_experiment`는 실행이 끝났을 때 응답합니다. 통과, 실패, 중지 여부와 소요 시간, 시드, 각 단계가
한 일이나 실패한 이유, 각 에뮬레이터가 받은 요청, 각 네트워크 장애 릴레이가 한 일, 보고서의 경로입니다.
실패한 실행도 정상적인 응답입니다. 이유는 단계가 알려 주며, 호출이 실패한 것이 아닙니다.

### 단일 메시지 {#tools-send}

| 도구 | 인수 |
| --- | --- |
| `send_osc` | `target`(`host:port`), `address`, `args`: 숫자(정수는 int32이고 범위를 넘으면 int64, 그 밖에는 float32), 문자열, 불리언, `null`, 또는 `{"type": "int"\|"float"\|"str"\|"long"\|"double"\|"bool"\|"blob"\|"nil", "value": …}`. |
| `send_udp` | `target`, 그리고 `text` 또는 `hex`(`"de ad be ef"`). |
| `send_http` | `method`, `url`, `headers`(`{"Name": "value"}`), `body`, `timeout_ms`(기본값 10000), `auth`: `{"scheme": "basic"\|"digest", "username", "password"}` 또는 `{"scheme": "bearer", "token"}`. 상태, 시간, 헤더, 본문(처음 16 KiB)을 반환합니다. |
| `send_mqtt` | `broker`(`host:port`, 없으면 포트 1883), `topic`, `payload`, `qos`(0, 1, 2), `retain`. `retain`을 설정한 빈 페이로드는 retained 값을 지웁니다. |
| `send_ws` | `url`(`ws://` 또는 `wss://`), `text` 또는 `hex`, `headers`, `protocols`, 그리고 응답을 기다리려면 `expect`(포함), `expect_regex` 또는 `wait`(아무 메시지); `timeout_ms` 1~120000(기본값 2000). 핸드셰이크, 보낸 것, 응답을 반환하며, 응답이 JSON이면 파싱한 JSON도 함께 반환합니다. |

이 도구들은 앱의 화면이 쓰는 명령과 같습니다. [`signallab send`](cli.md#cli-send)를 참고하십시오.

### 수신 {#tools-listen}

`listen`은 `signallab mcp`가 실행되는 **컴퓨터에서** UDP 포트를 일정 시간 동안 열고 도착한 것을
반환합니다. OSC 메시지는 디코딩해서, 다른 데이터그램은 텍스트와 hex로 반환합니다.

| 인수 | 내용 | 기본값 |
| --- | --- | --- |
| `bind` | `IP:port`, 예: `0.0.0.0:9000`. | 필수 |
| `protocol` | `osc` 또는 `udp`. | `osc` |
| `seconds` | 수신하는 시간, 0.1~60. | 5 |
| `max` | 데이터그램이 이만큼 도착하면 중단, 1~1000. | 100 |

`0.0.0.0`에 아무것도 도착하지 않으면, 응답은 어시스턴트에게 방화벽을 확인하라고
알려 줍니다([`signallab doctor`](cli.md#cli-doctor)). `--server`를 쓰면 `listen`은 거부됩니다. 서버에서는
대기 노드가 있는 실험이 거기서 수신합니다.

### 신호와 에뮬레이터 {#tools-library}

`list_signals`와 `fire_signal`은 신호 라이브러리를 사용합니다. 앱의 `signals.json`, `--library`
또는 호출에 지정한 `library` 경로입니다. 신호는 앱이 보내는 것과 똑같이 id나 이름으로 보냅니다.

`list_emulators`는 라이브러리의 에뮬레이터 이름을 보여 줍니다. `start_emulator`는 에뮬레이터 하나를
시작합니다. `emulator`에 문서를 주거나 `name`에 라이브러리 항목의 id나 이름을 주며, 작업 id와 주소를
반환합니다. 에뮬레이터는 `stop_job`을 호출할 때까지 자신의 규칙에 따라 응답합니다. `bind`는 다른
`IP:port`로 옮기고, `params`는 템플릿이 읽을 값을 주며, `seed`는 무작위 선택을 고정합니다.
`emulator_exchanges`(`job_id`, 그리고 더 새로운 것만 받으려면 `after`)는 도착한 것과 각 규칙이 응답한
것을 나열합니다. `set_emulator_down`(`job_id`, `down`, 그리고 `fault`: `unavailable`, `reset`,
`timeout`)은 실행 중인 에뮬레이터를 다시 복구할 때까지 끊어 둡니다. HTTP는 장애를 겪고
(`unavailable`은 503으로 응답), TCP 장치와 MQTT 브로커는 연결을 끊고, OSC와 UDP는 아무것도 응답하지
않습니다. [에뮬레이터](../tools/emulators.md)를 참고하십시오.

### 실행과 작업 {#tools-runs}

`list_runs`는 이전 실행의 보고서를 최신순으로 읽습니다. `experiment`가 실험을 지정하면 그 실험의
것만, 최대 `limit`개(1~500, 기본값 50)를 각 부하 단계의 수치와 함께 반환합니다. `compare_runs`는 두
실행의 이름, `a`(이전)와 `b`(이후)를 받아 각 부하 단계의 지연 시간, 오류율, 달성한 속도, 누락된
요청을 나란히 놓고, 나쁜 방향으로 5% 이상 변한 것을 회귀로 표시합니다.
[실행과 보고서](../experiments/runs.md)를 참고하십시오.

`list_jobs`는 모니터, 발생기, 에뮬레이터, 실행 등 실행 중인 것을 나열하고, `stop_job`은 id로 하나를
중지합니다.

## 결과와 오류 {#results}

모든 응답은 모델이 읽는 텍스트이자 같은 내용의 구조화된 데이터입니다. 실패는 오류로 표시되며
엔진의 오류를 담습니다. 변하지 않는 `code`, 그 값, 해당 노드와 필드가 들어 있고, `--lang`으로 고른
언어로 표현됩니다. 어시스턴트가 잘못 지정한 인수는 어시스턴트가 고칠 수 있는 말로 돌려보냅니다.

## 진행 상황과 취소 {#progress}

클라이언트가 `run_experiment`의 진행 상황을 요청하면 모든 단계(노드와 그 상태)가 일어나는 대로
보고되므로, 어시스턴트와 사용자 모두 실행이 진행되는 것을 볼 수 있습니다. 호출을 취소하면 호출이
중단됩니다. `run_experiment`를 취소하면 앱에서 [[ui:common.stop]] 버튼을 누른 것처럼 실행 자체가
중지됩니다.

클라이언트가 연결을 닫으면 진행 중인 호출이 끝난 뒤 `signallab mcp`가 종료됩니다.

## 랩 서버에서 {#on-a-server}

`--server http://192.0.2.10:1430`을 지정하면 실험, 전송, 신호, 에뮬레이터가 API를 통해 **그 서버에서**
이루어집니다. 서버의 네트워크, 시크릿, 데이터 폴더를 사용하므로, 어시스턴트는 랩에서만 닿을 수 있는
장비에도 접근합니다. 토큰은 클라이언트의 환경에 지정하십시오.

```json
{
  "mcpServers": {
    "signallab": {
      "command": "signallab",
      "args": ["mcp", "--server", "http://192.0.2.10:1430"],
      "env": { "SIGNALLAB_TOKEN": "<the server's token>" }
    }
  }
}
```

이 컴퓨터에 남는 것은 다음과 같습니다. 신호 라이브러리와 에뮬레이터 라이브러리(앱의 것, 또는
`--library`와 `--emulators`로 지정한 것)와 호출이 지정한 파일(`file`, `library`)은 여기서 읽으며, 그
내용이 서버로 전송됩니다. `listen`은 거부됩니다.
[Signal Lab을 서버로 실행하기](../server/index.md)를 참고하십시오.

## 안전 {#safety}

- 어시스턴트는 도구가 하는 일만 할 수 있으며, 모든 도구는 앱 자체의 명령 중 하나입니다. 앱이 닿지
  못하는 곳에는 어시스턴트도 닿지 못합니다.
- 보내거나, 수신하거나, 무언가를 시작하는 도구는 외부 세계에 닿는 것으로 표시되며, 호출할 때마다
  사용자에게 물을지는 클라이언트가 결정합니다.
- 시크릿 값은 어시스턴트에게 전달되지 않습니다. 실험은 시크릿을 `{{secret.NAME}}`으로 지정하고,
  모든 결과에서는 그 자리에 `••••` 표시가 대신 나옵니다.
- 에뮬레이터나 수신 대기는 그것이 실행되는 컴퓨터의 포트를 엽니다. `list_jobs`와 `stop_job`으로
  아직 실행 중인 것을 확인하고 끝낼 수 있습니다.

## 프로토콜 {#protocol}

클라이언트 개발자를 위한 정보입니다. stdio 위의 JSON-RPC 2.0이며 한 줄에 메시지 하나입니다. 표준
출력에는 프로토콜 메시지만 나가고, 사람을 위한 내용은 모두 표준 오류로 갑니다. 프로토콜 버전
`2025-06-18`, `2025-03-26`, `2024-11-05`(클라이언트가 다른 버전을 요청하면 가장 최신 버전), 배치,
`ping`, `tools/list`, `tools/call`을 지원합니다. `progressToken`을 보낸 호출에는
`notifications/progress`로 진행 상황을 알리고, 취소는 `notifications/cancelled`로 받습니다. 서버의
`instructions`는 도구들이 어떻게 맞물리는지 모델에게 알려 줍니다.

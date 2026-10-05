---
title: 명령줄
description: signallab은 창 없이 실험을 실행하고, 단일 메시지를 보내고, 에뮬레이터를 재생하고, 네트워크를 점검합니다. 터미널, 스크립트 또는 CI 작업에서.
---

# 명령줄: `signallab`

`signallab`은 창 없는 Signal Lab입니다. 실험을 끝까지 실행하고 스크립트가 이해하는 코드로
종료하며, OSC 메시지, 데이터그램, HTTP 요청, WebSocket 메시지 또는 MQTT 발행을 하나 보내고,
라이브러리에서 신호를 보내며, 중지할 때까지 에뮬레이터를 재생하고, 이 컴퓨터와 장비 사이에
무엇이 있는지 알려 줍니다.

앱과 같은 엔진입니다. 명령줄의 실행은 같은 단계를 거치고, 같은 보고서를 쓰며, 인터페이스의
언어로 같은 말을 합니다. `--server`를 쓰면 실행이 [Signal Lab 서버](../server/index.md)에서 그
서버의 네트워크와 시크릿으로 대신 이루어집니다.

```bash
signallab run tests/smoke.json --param api=http://127.0.0.1:8080 --junit junit.xml
signallab run flaky-api                                # a bundled template, by name
signallab validate tests/*.json                        # the editor's check, nothing sent
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab fire "Fader value"
signallab emulate tests/orders-api.json --for 120      # play a dependency for two minutes
signallab doctor                                       # firewall, network, data folder
```

파이프라인은 [CI에서의 Signal Lab](ci.md)을, AI 어시스턴트는 [`signallab mcp`](mcp.md)를
참고하십시오.

## 설치 {#install}

| 위치 | `signallab`을 얻는 방법 |
| --- | --- |
| Windows, 설치 프로그램(`.exe`) | 앱 옆에 설치되며, 그 폴더가 `PATH`에 추가됩니다 — "나에게만" 설치하면 사용자 PATH, "모든 사용자" 설치하면 컴퓨터 PATH입니다. 설치 후 새 터미널을 여십시오. 설치 프로그램의 `/NOPATH` 스위치는 `PATH`를 그대로 둡니다. |
| Windows, `.msi` | 앱 옆에 설치됩니다. 앱이 설치되어 있는 동안 설치 폴더가 컴퓨터 `PATH`에 있습니다. |
| Linux, `.deb` 및 `.rpm` | `/usr/bin/signallab`. |
| Linux, AppImage | 포함되지 않습니다. 아래의 아카이브를 사용하십시오. |
| 앱 없이, 아무 컴퓨터에서나 | 모든 릴리스에는 `signallab-<version>-windows-x64.zip`과 `signallab-<version>-linux-x64.tar.gz`가 있으며, 각각 프로그램과 라이선스가 들어 있습니다. 같은 페이지의 `SHA256SUMS.txt`에 그 체크섬이 나열됩니다. |
| 서버 이미지 | `ghcr.io/proanima/signallab` 안의 `/usr/local/bin/signallab`([CI](ci.md#docker) 참고). |

다음으로 확인하십시오:

```bash
signallab version
```

## 명령 {#commands}

| 명령 | 하는 일 |
| --- | --- |
| [`run`](#cli-run) | 실험을 차례로 실행합니다. 모두 통과했을 때만 0으로 종료합니다. |
| [`validate`](#cli-validate) | 실행 전에 편집기처럼 실험을 검사합니다. 아무것도 보내지 않습니다. |
| [`send`](#cli-send) | 메시지 하나를 보냅니다: `osc`, `udp`, `http`, `ws` 또는 `mqtt`. |
| [`fire`](#cli-fire) | 신호 라이브러리의 신호를 id나 이름으로 보냅니다. |
| [`emulate`](#cli-emulate) | HTTP API, OSC, UDP 또는 TCP 장치, 또는 MQTT 브로커를 <kbd>Ctrl</kbd>+<kbd>C</kbd> 또는 `--for`까지 재생합니다. |
| [`emulators`](#cli-emulators) | 앱 라이브러리의 에뮬레이터를 나열합니다. |
| [`templates`](#cli-templates) | 기본 제공 실험 템플릿을 나열합니다. |
| [`nodes`](#cli-nodes) | 모든 종류의 노드를 JSON으로 설명합니다. |
| [`mcp`](#cli-mcp) | Model Context Protocol을 통해 Signal Lab을 AI 어시스턴트에게 제공합니다. |
| [`doctor`](#cli-doctor) | 방화벽, 네트워크, 데이터 폴더, 서버를 점검합니다. |
| [`firewall`](#cli-firewall) | `firewall allow`: 다른 컴퓨터가 Windows 방화벽을 통해 Signal Lab에 닿도록 허용합니다. |
| [`version`](#cli-version) | 버전을 출력합니다. |

`signallab help <command>` 또는 `signallab <command> --help`는 명령의 옵션을 출력합니다.

## 모든 명령의 옵션 {#global-options}

| 옵션 | 하는 일 | 기본값 |
| --- | --- | --- |
| `--lang <code>` | 메시지의 언어: `en`, `ru`, `es`, `fr`, `de`, `pt`, `zh`, `ja`, `ko`, `hi` 또는 `ar`. | `SIGNALLAB_LANG`, 없으면 로캘, 그것도 없으면 `en` |
| `--json` | 텍스트 대신 기계용 출력을 stdout에 씁니다([출력](#output) 참고). | 꺼짐 |
| `-h`, `--help` | 명령의 도움말. | |
| `-V`, `--version` | 버전. 명령 앞에, `signallab --version`처럼. | |

`--lang`과 `--json`은 줄의 어디에나 올 수 있습니다.
`signallab --json run smoke.json`과 `signallab run smoke.json --json`은 같습니다.

### 언어 {#language}

메시지, 단계 텍스트, 실패, JUnit 보고서는 인터페이스 자체의 텍스트, 복수형, 숫자 표기를
사용합니다. 언어는 다음 중 첫 번째입니다:

1. `--lang`;
2. `SIGNALLAB_LANG`(`ru`, `ru-RU`, `ru_RU.UTF-8`은 모두 러시아어를 뜻합니다);
3. 로캘: `LC_ALL`, `LC_MESSAGES`, `LANG` 중 설정된 첫 번째;
4. 영어.

::: tip
Windows 터미널은 보통 로캘 변수를 설정하지 않으므로, `SIGNALLAB_LANG`을 설정하거나 `--lang`을
넘기지 않으면 `signallab`은 거기서 영어로 말합니다.
:::

### 사람과 기계를 위한 출력 {#output}

`--json` 없이, 결과는 **stdout**으로 가고 사람이 읽는 것은 모두 **stderr**로 갑니다. 실행의
단계가 일어나는 대로, 무언가 실패한 이유, 보고서가 있는 위치입니다.
`signallab run … 2>/dev/null`은 실행마다 판정 한 줄만 남깁니다.

`--json`을 쓰면 stdout에는 JSON만 나가고, stderr는 조용합니다:

| 명령 | `--json`이 stdout에 출력하는 것 |
| --- | --- |
| `run` | 한 줄에 객체 하나: `started`, 단계마다 `step`, `ended`(실행의 결과), 그다음 `summary`; 시작하지 못한 실행에는 `error`. [`run`이 출력하는 것](#run-output)을 참고하십시오. |
| `validate` | 실험마다 한 줄에 객체 하나: `valid`, 그리고 `profile_issues` 또는 `error`. |
| `send`, `fire` | `{"type": "sent", "result": …}`; `send http`는 `{"type": "response", "response": …}`, `send ws`는 `{"type": "exchange", "result": …}`를 출력합니다. |
| `emulate` | 한 줄에 객체 하나: `started`, 요청마다 `exchange`, 에뮬레이터마다 `summary`; `--check`를 쓰면 `valid`. |
| `emulators` | `{"path": …, "emulators": [{id, name, protocol, bind, rules, note}, …]}`. |
| `templates` | `[{"name": …, "experiment": …}, …]`. |
| `doctor` | 객체 하나: `version`, `network`, `data_dir`, `firewall`, `server`, `problems`. |
| `version` | `{"version": "…"}`. |
| `nodes` | 항상 JSON, `--json`이 있든 없든. |

실패는 `{"type": "error", "error": {…}, "exit_code": N}`입니다. `error`는 엔진의
오류입니다: 변하지 않는 `code`(예: `transport.refused` 또는 `secret.missing`), 그 `params`,
그리고 그것이 관한 `node`와 `field`입니다. 스크립트는 어떤 언어로든 `error.code`로 분기할 수
있습니다. 모든 코드의 텍스트는 [오류 메시지](../reference/errors.md)에 나열되어 있습니다.

### 종료 코드 {#exit-codes}

| 코드 | 뜻 |
| --- | --- |
| `0` | 모든 실험이 통과했습니다. 전송이 성공했습니다. 가로막는 것이 없습니다. |
| `1` | 실험이 실행되어 실패했거나, 시간이 초과되었거나, 중지되었습니다. 전송이 실패했습니다(거부, 응답 없음, 예상치 못한 상태). `doctor`가 가로막는 것을 찾았습니다. |
| `2` | 명령이나 문서가 잘못되었습니다: 인수, 읽을 수 없는 파일, 검증 오류, 알 수 없는 매개변수, 없는 시크릿. |
| `3` | 실험 밖의 이유로 아무것도 실행할 수 없었습니다: 서버에 닿을 수 없거나 토큰을 거부함, 포트를 열 수 없음, 자격 증명 저장소 실패. |

실험이 여러 개면 가장 심각한 결과가 결정합니다. 순서는 `2`, 그다음 `3`, 그다음 `1`,
그다음 `0`입니다.

### 환경 변수 {#environment}

| 변수 | 하는 일 |
| --- | --- |
| `SIGNALLAB_LANG` | `--lang`이 없을 때의 언어. |
| `LC_ALL`, `LC_MESSAGES`, `LANG` | 위의 어느 것도 설정되지 않았을 때의 언어. |
| `SIGNALLAB_SERVER` | `run`, `validate`, `emulate`, `mcp`, `doctor`의 서버로, `--server`가 주는 것과 같습니다. |
| `SIGNALLAB_TOKEN` | 토큰 파일이 없을 때 서버의 접근 토큰. |
| `SIGNALLAB_TOKEN_FILE` | 서버의 토큰이 들어 있는 파일로, `--token-file`이 주는 것과 같습니다. |
| `SIGNALLAB_SECRET_<NAME>` | 이 프로세스에서 실행하는 데 쓰는 시크릿 `NAME`의 값([시크릿](#secrets) 참고). |
| `SIGNALLAB_DATA_DIR` | 앱의 데이터 폴더로, `fire`, `emulators`, `emulate`, `mcp`, `doctor`가 기본으로 찾는 곳입니다. 없으면 홈 폴더의 `Documents/SignalLab`입니다. |
| `GITHUB_ACTIONS` | 이 값이 `true`이면 통과하지 못한 실행이 `::error` 주석으로도 출력되며, GitHub가 실행 페이지에 표시합니다. |

## `run` {#cli-run}

```text
signallab run [OPTIONS] <FILE>...
```

실험을 차례로 실행하고 모두 통과했을 때만 `0`으로 종료합니다.

| 옵션 | 하는 일 | 기본값 |
| --- | --- | --- |
| `<FILE>...` | 실험 파일, 또는 [기본 제공 템플릿](#cli-templates)의 이름. | 필수 |
| `-p`, `--param NAME=VALUE` | 이번 실행을 위한 매개변수 값. 더 있으면 반복. | 문서의 값 |
| `--profile NAME` | 이 프로필로 실행합니다. 주어진 모든 실험에 그 프로필이 있어야 합니다. `""`는 기본값으로 실행합니다. | 문서의 프로필 |
| `-m`, `--matrix NAME=V1,V2` | 값마다 한 번씩 실행합니다. 이름이 더 있으면 반복([실행의 매트릭스](#matrix) 참고). | |
| `--matrix-file PATH` | JSON 파일에서 조합을 읽습니다. | |
| `--seed N` | 임의 값의 시드, 0~9007199254740991(2⁵³ − 1). | 문서의 시드, 없으면 실행마다 새 값 |
| `--timeout SECONDS` | 더 오래 걸리는 실행을 실패시킵니다, 1–300. | `300` |
| `--fail-fast` | 통과하지 못한 첫 실행에서 멈춥니다. 나머지는 시작하지 않습니다. | 꺼짐 |
| `--junit PATH` | JUnit XML 보고서를 그곳에 씁니다([보고서](#reports) 참고). | |
| `--report PATH` | 실행 보고서를 그곳에 복사합니다: 실행 하나면 파일, 여러 개면 폴더. | |
| `--data-dir PATH` | 이 프로세스의 실행을 위한 데이터 폴더. 그 보고서가 거기 남습니다. | 종료할 때 지워지는 임시 폴더 |
| `--server URL` | 이 프로세스 대신 이 서버에서 실행합니다([서버에서](#run-on-server) 참고). | `SIGNALLAB_SERVER` |
| `--token-file PATH` | 서버의 토큰이 들어 있는 파일. | `SIGNALLAB_TOKEN_FILE`, 없으면 `SIGNALLAB_TOKEN` |
| `--secrets files\|system` | 이 프로세스에서 시크릿 값이 오는 곳. | `files` |
| `--secrets-dir PATH` | 이름마다 파일 하나씩 있는 시크릿 파일 폴더. | 있으면 `/run/secrets/signallab` |

`--data-dir`, `--secrets`, `--secrets-dir`는 이 프로세스의 실행에 관한 것입니다. `--server`와
함께 쓸 수 없습니다.

### 파일과 템플릿 {#run-inputs}

`FILE`은 앱이 저장하고 내보내는 그대로의 실험입니다([[ui:nav.experiment]] 화면의
[[ui:exp.exportJson]]). 앱이 여는 모든 문서 버전이 동작합니다. 더 오래된 버전은 읽을 때
최신으로 맞춰지며, 앱이 열 때 하는 것과 같습니다. 파일이 아닌 이름은 `.json`이 있든 없든
[기본 제공 템플릿](#cli-templates)에서 찾습니다:

```bash
signallab run tests/stage-cues.json tests/api.json
signallab run osc-ping-reply --param device=192.0.2.20:9000
```

모든 실험과 매트릭스의 모든 조합은 첫 실행 전에 편집기가 검사하는 방식으로 검사됩니다. 세
번째 파일이 잘못되면 첫 번째도 멈추며, 아무것도 보내기 전에 종료 코드 `2`로 끝납니다.

### 매개변수와 프로필 {#run-params}

`--param NAME=VALUE`는 이번 실행에만 매개변수를 설정하며, 파일은 바뀌지 않습니다. 값은
첫 `=` 뒤의 전부이므로 `--param url=http://127.0.0.1/?q=1`이 동작하고 `--param note=`는 빈
값을 설정합니다. 값은 그 매개변수를 가진 주어진 모든 실험에 적용되며, 그중 적어도 하나의
매개변수 이름이어야 합니다. 이름을 잘못 쓰면 종료 코드 `2`로 거부됩니다.

`--profile NAME`은 편집기에서 [[ui:exp.profile]] 아래에서 고르는 것처럼 실험의 프로필 중
하나로 실행합니다. [데이터와 템플릿](../experiments/data.md)을 참고하십시오.

### 실행의 매트릭스 {#matrix}

매트릭스는 같은 실험을 모든 대상, 모든 사용자, 모든 페이로드 크기에 대해 실행합니다. 각
조합은 자체 결과와 자체 보고서, JUnit 보고서의 자체 스위트를 가진 별개의 실행입니다.

```bash
signallab run smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest \
  --fail-fast --junit junit.xml
```

네 번의 실행입니다. `device`가 가장 느리게, `user`가 가장 빠르게 변합니다. 이름은 파일과 그
값에서 따옵니다: `smoke.json [device=192.0.2.20:9000, user=admin]`.

- `--matrix NAME=V1,V2`는 축을 더합니다. 같은 이름을 다시 주면 그 축에 값을 더합니다. 이름과
  값 주변의 공백은 제거됩니다. 두 번 준 값은 한 번만 실행됩니다.
- `--matrix-file PATH`는 두 가지 형태 중 하나로 JSON을 읽습니다:

  ```json
  { "device": ["192.0.2.20:9000", "192.0.2.21:9000"], "retries": [1, 3] }
  ```

  는 축을 더합니다 — 모든 조합이 실행되며, 이 이름들은 `--matrix`의 이름 뒤에 알파벳 순으로
  옵니다;

  ```json
  [
    { "device": "192.0.2.20:9000", "user": "admin" },
    { "device": "192.0.2.21:9000", "user": "guest" }
  ]
  ```

  는 조합 자체를 나열하며, 각각은 `--matrix`의 축과 교차합니다. 값은 텍스트, 숫자 또는
  `true`/`false`입니다. 파일에서 값 안의 쉼표는 값의 일부로 남습니다.
- 모든 매트릭스 이름은 주어진 실험 중 적어도 하나의 매개변수여야 합니다. 그중 하나가 없는
  실험은 무시할 값마다가 아니라 한 번만 실행됩니다.
- `--param`과 매트릭스 양쪽에서, 또는 `--matrix`와 파일 양쪽에서 설정된 이름은 거부됩니다.
- 한 명령에서 최대 **256**번의 실행. 그보다 많으면 아무것도 실행되기 전에 거부됩니다.

### 서버에서 실행하기 {#run-on-server}

```bash
signallab run tests/stage.json --server http://192.0.2.10:1430 --token-file token.txt
```

실험은 이 컴퓨터에서 옵니다. 서버는 자체 네트워크, 자체
[시크릿](../server/index.md#secrets), 자체 데이터 폴더로 실행하고 단계를 일어나는 대로
되돌려 보냅니다. 보고서는 서버에 남으며 그 경로가 출력되고, `--report`는 사본을 내려받습니다.
서버에서의 실행은 인터페이스에서 시작한 것과 같은 그곳의 작업입니다: 서명한 모든 페이지가
그것을 보여 줍니다. 실행 중에 `signallab`이 사라져도 실행은 서버에서 끝나고 보고서를 남깁니다.

토큰은 `--token-file`(또는 `SIGNALLAB_TOKEN_FILE`)에서, 없으면 `SIGNALLAB_TOKEN`에서 읽어
`Authorization: Bearer`로 보냅니다. 닿을 수 없거나, 토큰을 거부하거나, 실행 도중 응답을 멈춘
서버는 종료 코드 `3`입니다. `signallab doctor --server URL`은 주소와 토큰을 따로 점검합니다.

### 보고서 {#reports}

각 실행은 앱이 쓰는 것과 같은 보고서를 씁니다. `--data-dir`이 없으면 실행은 `signallab`이
종료될 때 지워지는 임시 폴더를 사용하므로, 필요한 것은 남겨 두십시오:

- `--report PATH`는 보고서를 복사합니다: 실행 하나면 `PATH` 자체에, 여러 개면 `PATH` 폴더에
  `01-<file>.json`, `02-<file>.json`, …으로 실행 순서대로.
- `--data-dir PATH`는 모든 실행의 보고서를 그 폴더(`runs/` 아래)에 남기고, 각각 어디 있는지
  출력합니다.

`--junit PATH`는 모든 CI 시스템이 읽는 형식인 JUnit XML 보고서를 씁니다:

- 실행마다(매트릭스의 조합마다) `<testsuite>` 하나, 파일, 시드, 결과, 프로필, 보고서의 경로,
  각 매트릭스 값(`param.NAME`)을 속성으로 갖습니다;
- 실행된 노드마다 `<testcase>` 하나, 노드의 타입과 id에서 이름을 따오고 자체 시간을 갖습니다;
- 실패한 노드에 `<failure>`: 고른 언어의 메시지, `type`으로서의 오류 코드, 기술적 세부 사항이
  담긴 노드의 단계;
- 실행이 닿지 못한 노드마다(분기의 반대편) `<skipped>` 케이스;
- 시작할 수 없었던 실험을 위한 `<error>` 케이스가 하나 있는 스위트, 그리고 `--fail-fast`가
  시작하지 않은 실행마다 `<skipped>` 케이스가 하나 있는 스위트.

```xml
<testsuites name="Signal Lab" tests="6" failures="1" errors="0" time="2.006">
  <testsuite name="HTTP to OSC" tests="6" failures="1" errors="0" skipped="4" time="2.006" timestamp="2026-10-04T16:48:03">
    <properties>
      <property name="file" value="status-branch" />
      <property name="seed" value="42" />
      <property name="outcome" value="failed" />
      <property name="report" value="out/report.json" />
    </properties>
    <testcase name="Start (start)" classname="HTTP to OSC" time="0.001">
      <system-out>Started · seed 42</system-out>
    </testcase>
    <testcase name="HTTP request (request)" classname="HTTP to OSC" time="2.004">
      <failure message="http://127.0.0.1:8080/ refused the connection — nothing is listening on that port" type="transport.refused">…</failure>
    </testcase>
    <testcase name="Status branch (branch)" classname="HTTP to OSC" time="0.000">
      <skipped message="not reached in this run" />
    </testcase>
    …
  </testsuite>
</testsuites>
```

[부하](../experiments/load.md)를 받는 HTTP 노드는 마지막 단계 아래에 임계값마다 충족 여부를
한 줄 더합니다(`✕ p95 < 100 ms · 152.58 ms`). 충족하지 못한 임계값은 실행과 그 JUnit
케이스를 실패시킵니다(`type="load.threshold"`).

### 시크릿 {#secrets}

실험은 시크릿을 `{{secret.NAME}}`으로 읽습니다. 이 프로세스의 실행에서 값은 다음에서 옵니다:

| `--secrets` | `NAME`의 값이 오는 곳 |
| --- | --- |
| `files`(기본값) | 환경 변수 `SIGNALLAB_SECRET_NAME`; 없으면 `--secrets-dir`(기본값은 그 폴더가 있을 때 `/run/secrets/signallab` — Docker 시크릿 배치)에 있는 `NAME`이라는 파일. |
| `system` | Windows 자격 증명 관리자 — 앱이 [[ui:exp.secrets]]의 값을 보관하는 곳. Linux에는 `signallab`이 읽는 자격 증명 저장소가 없습니다. 그곳에서 `--secrets system`은 종료 코드 `3`으로 실패합니다. |

파일의 끝 줄바꿈은 값의 일부가 아니며, 빈 값은 설정되지 않은 것으로 칩니다. 이름은 문자,
숫자, `_`이며 숫자로 시작하지 않고 최대 128자입니다. 값은 최대 16 KiB입니다.

```bash
SIGNALLAB_SECRET_API_TOKEN="$API_TOKEN" signallab run tests/api.json
```

설정되지 않은 시크릿은 트래픽이 오가기 전에 실행을 멈추며, 종료 코드 `2`와 빠진 이름을 알려
줍니다. 값은 절대 출력되지 않습니다: 단계, 오류, 보고서, JUnit 보고서는 그 자리에 `••••`를
표시합니다. `--server`를 쓰면 시크릿은 서버의 것입니다.

### `run`이 출력하는 것 {#run-output}

실행이 진행되면서 각 단계는 stderr에 한 줄입니다 — 시작 이후의 시간, 노드, 그 상태와 한 일 —
앱의 타임라인처럼. 끝나면 stdout의 한 줄이 어떻게 되었는지 말합니다:

```text
▶ HTTP check (http-check) · seed 1185927457137919
     0.000  Start         Running
     0.000  Start         Passed · Started · seed 1185927457137919
     0.000  HTTP request  Running
     2.004  HTTP request  Failed · URL — http://127.0.0.1:8080/ refused the connection — nothing is listening on that port
✖ HTTP check failed after 2 s: HTTP request · URL — http://127.0.0.1:8080/ refused the connection — nothing is listening on that port
  Technical details: error sending request for url (http://127.0.0.1:8080/): … (os error 10061)
  To run it again with the same random values: --seed 1185927457137919
```

실패한 실행은 사용한 시드를 알려 줍니다: 그 숫자와 함께 `--seed`를 쓰면 같은 임의 값으로 다시
실행합니다. 판정 뒤에 stderr로 각 에뮬레이터가 무엇을 요청받았는지 옵니다 — 요청, 규칙이 받지
않은 수, 실패한 수, 규칙별 적중 수:

```text
✔ Retry a flaky API passed in 7 ms
  Flaky API: 3 requests, 0 without a rule, 0 failed · #1 3
```

그리고 각 네트워크 장애 릴레이가 한 일: 받은 것, 버린 것, 제한한 것, 그리고 각 구간을
전달/수신으로:

```text
  127.0.0.1:19110 → 127.0.0.1:19100: 155 datagrams, 38 dropped, 0 throttled · lan 0.0–2.0 s 39/39, wifi 2.0–4.0 s 39/39, offline 4.0–6.0 s 0/38, lan 6.0–8.0 s 38/38
```

TCP 기반 릴레이는 데이터그램이 아니라 스트림의 청크를 옮기고 아무것도 버리지 않으므로, 그
줄은 청크, 연결, 리셋, 절반 열린 연결, 대역폭 제한에 의해 스트림이 지연된 횟수를 셉니다:

```text
  127.0.0.1:19120 → 127.0.0.1:19101: 14 chunks, 2 connections, 1 reset, 0 half-open, 3 held back
```

[부하](../experiments/load.md)를 받는 HTTP 노드는 진행 상황을 최대 1초에 한 번 단계로
보고합니다.

실행이 여러 개면 실행마다 한 줄과 집계가 출력을 끝맺습니다:
`3 runs: 2 passed, 1 failed`. `--fail-fast`를 쓰면 시작하지 않은 실행이 stderr에
집계됩니다.

`--json`을 쓰면 모든 줄이 `type`을 가진 객체입니다:

```json
{"type":"started","experiment":"Empty experiment","file":"empty","job_id":1,"overridden":false,"profile":null,"seed":1,"started_ms":1791132204585}
{"type":"step","node_id":"start","state":"passed","detail":"Started","message_key":"exp.step.started","message_params":{"seed":1},"job_id":1,"ts":1791132204585}
{"type":"ended","experiment":"Empty experiment","file":"empty","outcome":"passed","seed":1,"params":{},"steps":[…],"report_path":"…","started_ms":1791132204585,"ended_ms":1791132204585,…}
{"type":"summary","total":1,"passed":1,"failed":0,"not_started":0,"exit_code":0}
```

`started`, `ended`, `error` 줄은 `file`(주어진 그대로의 인수)과, 매트릭스에서는
`matrix`(조합의 값)를 담습니다. `ended`는 실행의 전체 결과입니다: `outcome`(`passed`,
`failed` 또는 `stopped`), `seed`, `profile`, `params`, `error`, 모든 단계, 실행에 있었다면
`emulators`와 `impairments`, 그리고 `report_path`. 부하의 마지막 단계는 측정한 모든 숫자와
함께 `load`를 담습니다: `planned`, `sent`, `ok`, `failed`, `missed`, `rps`, `error_rate`,
`min_ms`, `mean_ms`, `max_ms`, `p50_ms`부터 `p99_ms`, `statuses`, 매초(`seconds`),
`histogram`, 각 임계값의 판정(`thresholds`).

## `validate` {#cli-validate}

```text
signallab validate [OPTIONS] <FILE>...
```

실행 전에 편집기가 하는 방식으로 실험을 검사합니다 — 그래프, 모든 필드, 템플릿, 매개변수,
시크릿 — 아무것도 보내지 않습니다. 모두 시작할 수 있으면 `0`으로 종료합니다.

[`run`](#cli-run)의 입력을 받습니다: 파일과 템플릿, `--param`, `--profile`, `--matrix`,
`--matrix-file`, 그리고 `--server`, `--token-file`, `--secrets`, `--secrets-dir`. `--server`를
쓰면 서버가 자체 시크릿으로 검사합니다.

```text
✔ tests/stage.json: Stage cues would run
  Rehearsal: Would not run: …
```

문서의 다른 프로필에만 있는 문제는 그 아래에 나열되지만 검사를 실패시키지는 않습니다.
`--json`을 쓰면 실험마다(조합마다) 한 줄: `{"experiment", "file", "valid": true, "profile_issues": […]}`, 또는 `error`와 `exit_code`와 함께 `"valid": false`.

## `send` {#cli-send}

```text
signallab send <osc|udp|http|ws|mqtt> …
```

앱의 화면이 쓰는 것과 같은 명령으로 메시지 하나를 보내므로, 회선 위의 바이트가 같습니다.
전송은 라이브러리도 시크릿도 읽지 않습니다.

종료 코드: `0` 전송됨, `1` 전송 실패, `2` 인수가 잘못됨.

`<host:port>`는 IP 주소 또는 호스트 이름과 포트입니다: `127.0.0.1:9000`, `[::1]:9000`(대괄호로
묶은 IPv6 주소) 또는 `device.local:9000`. 이름은 명령이 실행될 때 조회하며, IPv4 주소가 있으면
그것을 취하므로 `localhost:9000`은 `127.0.0.1`의 수신자에게 닿습니다. 확인되지 않는 이름은
전송을 실패시킵니다(`1`). 포트가 없는 대상은 잘못된 인수입니다(`2`).

### `send osc` {#cli-send-osc}

```text
signallab send osc <host:port> <address> [ARG]...
```

OSC 메시지 하나. 인수는 접두사로 타입을 지정하거나 추론합니다:

| 인수 | OSC 타입 |
| --- | --- |
| `i:3` | int32 |
| `f:0.5` | float32 |
| `d:1.5` | float64 (double) |
| `h:64` | int64 |
| `s:text` | 문자열 |
| `b:de ad be ef` | blob, hex 바이트로 |
| `T`, `F` | true, false |
| `N` | nil |
| `3`, `-3` | 그냥 정수는 int32 |
| `2.5` | 그냥 소수는 float32 |
| 그 밖의 것 | 문자열 |

```bash
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main 3
# ✔ sent /cue/go (32 bytes) → 127.0.0.1:9000
```

공백이 있는 인수는 따옴표로 묶으십시오: `"s:hello world"`. `s:7`은 텍스트 `7`을 보냅니다.
주소는 `/`로 시작해야 하며, 그렇지 않으면 잘못된 인수입니다(`2`).

::: warning Windows의 Git Bash
Git Bash는 `/`로 시작하는 인수를 Windows 경로로 바꾸므로 `/cue/go`가
`C:/Program Files/Git/cue/go`로 도착합니다. `//cue/go`라고 쓰거나
`MSYS_NO_PATHCONV=1`로 실행하십시오. PowerShell과 `cmd`는 영향을 받지 않습니다.
:::

[OSC](../protocols/osc.md)를 참고하십시오.

### `send udp` {#cli-send-udp}

```text
signallab send udp <host:port> (--text TEXT | --hex HEX)
```

UDP 데이터그램 하나. 페이로드는 `--text`로, 또는 `--hex` 바이트로 줍니다:
`"de ad be ef"`, `deadbeef` 또는 `0xDE,0xAD`.

```bash
signallab send udp 127.0.0.1:7000 --text "PLAY 1"
signallab send udp 127.0.0.1:7000 --hex "de ad be ef"
```

### `send http` {#cli-send-http}

```text
signallab send http <METHOD> <URL> [OPTIONS]
```

HTTP 요청 하나. 상태 줄은 stderr로 — `HTTP 200 OK · 3 ms · 1,234 B` — 응답 본문은 stdout으로
가므로 파이프로 넘길 수 있습니다. 256 KiB보다 긴 본문은 거기서 잘리며, stderr가 그렇게 알려
줍니다.

| 옵션 | 하는 일 | 기본값 |
| --- | --- | --- |
| `-H`, `--header "Name: value"` | 요청 헤더. 더 있으면 반복. | |
| `--body TEXT` | 요청 본문. `@FILE`은 텍스트 파일의 내용을 보냅니다. | 없음 |
| `--expect-status STATUS` | 응답이 이 상태가 아니면 `1`로 종료합니다. | 아무 상태나 `0` |
| `--timeout MS` | 응답을 기다리는 밀리초. | `10000` |
| `-u`, `--user NAME:PASSWORD` | 자격 증명, Basic으로 전송. | |
| `--digest` | `--user`와 함께: 대신 서버의 Digest challenge에 답합니다(MD5 또는 SHA-256). | 꺼짐 |
| `--bearer TOKEN` | `Authorization: Bearer TOKEN`을 보냅니다. `--user`와 함께 쓸 수 없습니다. | |

```bash
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab send http POST http://127.0.0.1:8080/cue -H "Content-Type: application/json" --body '{"cue": 1}'
signallab send http GET http://127.0.0.1:8080/admin -u admin:secret --digest
```

`--expect-status`가 없으면 `500`을 포함해 어떤 응답이든 성공입니다. 응답을 받지 못한
요청(거부, 시간 초과, 확인되지 않는 이름)은 종료 코드 `1`이며, 답할 수 없었던 Digest
challenge도 마찬가지입니다 — 이유는 응답 뒤에 출력됩니다.

::: tip
인수는 같은 컴퓨터의 다른 사용자에게 보입니다. 진짜 비밀번호는 [시크릿](#secrets)인 실험에
쓰십시오.
:::

[HTTP](../protocols/http.md)를 참고하십시오.

### `send ws` {#cli-send-ws}

```text
signallab send ws <URL> [OPTIONS]
```

WebSocket 교환 하나: `ws://…` 또는 `wss://…`에 연결해 메시지를 보내고, 선택적으로 답을 기다린
뒤 닫습니다. 핸드셰이크와 보낸 것은 stderr로, 답은 stdout으로 갑니다(이진 메시지는 hex로).

| 옵션 | 하는 일 | 기본값 |
| --- | --- | --- |
| `--text TEXT` | 이 텍스트 메시지를 보냅니다. | 아무것도 보내지 않음 |
| `--hex HEX` | 이 바이트를 이진 메시지로 보냅니다. `--text`와 함께 쓸 수 없습니다. | |
| `-H`, `--header "Name: value"` | 업그레이드 요청의 헤더. 더 있으면 반복. | |
| `--protocol NAME` | 제안할 하위 프로토콜. 더 있으면 선호 순서로 반복. | |
| `--expect TEXT` | 이 텍스트를 포함하는 메시지를 기다립니다. | |
| `--expect-regex REGEX` | 이 정규식과 일치하는 메시지를 기다립니다. | |
| `--wait` | 아무 메시지나 기다립니다. | |
| `--timeout MS` | 답을 기다리는 밀리초. | `2000` |

```bash
signallab send ws ws://127.0.0.1:9001/echo --text '{"ping": 1}' --expect '"ping"'
signallab send ws ws://127.0.0.1:9001/feed --wait        # nothing sent: the server's first message
```

`--expect`, `--expect-regex`, `--wait`가 없으면 연결하고 보낸 뒤 기다리지 않고 닫습니다.
기대한 답이 제때 오지 않으면 종료 코드는 `1`입니다.
[WebSocket](../protocols/websocket.md)를 참고하십시오.

### `send mqtt` {#cli-send-mqtt}

```text
signallab send mqtt <host:port> <topic> [payload] [--qos 0|1|2] [--retain]
```

평문 TCP 위의 MQTT 3.1.1 발행 하나로, 자격 증명 없이, 새 클라이언트 id를 가진 독자적
클라이언트로서, 이미 있는 연결을 끊지 않습니다. 포트가 없으면 브로커는 `1883`에 있습니다.
토픽은 하나의 토픽입니다: `+` 또는 `#`이 들어 있거나 비어 있으면 연결 전에 명령이
거부됩니다(`2`).

| 옵션 | 하는 일 | 기본값 |
| --- | --- | --- |
| `[payload]` | 페이로드. | 비어 있음 |
| `--qos 0\|1\|2` | 서비스 품질. | `0` |
| `--retain` | 토픽의 retained 값으로 남깁니다. `--retain`과 함께 빈 페이로드는 그것을 지웁니다. | 꺼짐 |

```bash
signallab send mqtt 127.0.0.1:1883 lab/light/1/set on --qos 1
signallab send mqtt 127.0.0.1 lab/light/1/state "" --retain     # clear the retained value
```

[MQTT](../protocols/mqtt.md)를 참고하십시오.

## `fire` {#cli-fire}

```text
signallab fire <signal> [--library PATH]
```

신호 라이브러리의 신호를 앱의 [[ui:nav.signals]] 화면이 보내는 것과 정확히 똑같이 보냅니다:
OSC, UDP, HTTP, MQTT 신호입니다. 신호는 id로, 없으면 대소문자를 무시한 이름으로 찾습니다.
여러 신호가 공유하는 이름은 그 id들과 함께 거부됩니다.

| 옵션 | 하는 일 | 기본값 |
| --- | --- | --- |
| `<signal>` | 신호의 id 또는 이름. | 필수 |
| `--library PATH` | 라이브러리 파일. | 앱의 데이터 폴더에 있는 `signals.json` |

```bash
signallab fire "Fader value"
signallab fire go --library show/signals.json
```

라이브러리는 읽기만 하며, 만들어지거나 바뀌지 않습니다.
[신호](../tools/signals.md)를 참고하십시오.

## `emulate` {#cli-emulate}

```text
signallab emulate [OPTIONS] <FILE|NAME>...
```

반대편 — HTTP API, OSC, UDP 또는 TCP 장치, MQTT 브로커 — 을 <kbd>Ctrl</kbd>+<kbd>C</kbd> 또는
`--for`까지 재생하고, 답하는 대로 모든 요청을 출력합니다. `FILE`은 에뮬레이터 하나, 그 목록,
또는 앱이 쓰는 그대로의 라이브러리 전체를 담습니다. `NAME`은 앱
라이브러리([[ui:nav.emulators]] 화면의)에 있는 것의 id 또는 이름입니다.

| 옵션 | 하는 일 | 기본값 |
| --- | --- | --- |
| `-p`, `--param NAME=VALUE` | 템플릿이 `{{NAME}}`으로 읽는 값. 더 있으면 반복. | |
| `--bind IP:PORT` | 대신 그곳에서 수신합니다. 에뮬레이터가 하나일 때만. | 에뮬레이터 자체 |
| `--for SECONDS` | 이 시간 뒤에 멈춥니다. | <kbd>Ctrl</kbd>+<kbd>C</kbd>까지 |
| `--seed N` | 무작위 선택의 시드: 무작위 순서, 지터, 생성기. | |
| `--check` | 포트를 열지 않고 에뮬레이터를 검사하고 종료합니다. | 꺼짐 |
| `--library PATH` | 이름을 찾을 라이브러리. | 앱의 데이터 폴더에 있는 `emulators.json` |
| `--server URL`, `--token-file PATH` | 서버에서 시작하고 그 API를 통해 따라가며 끝날 때 멈춥니다. | |
| `--secrets`, `--secrets-dir` | [`run`](#cli-run)과 같습니다. | |

```text
$ signallab emulate tests/orders-api.json --for 60
Orders API (http) answering on 127.0.0.1:18099
answering for 60 s
+   1.209 s Orders API  #1  GET /orders/42 → 200 OK · 12 B  1 ms  ← 127.0.0.1:55744
+   1.209 s Orders API  —  GET /nothing → 404 Not Found · 20 B  2 ms  ← 127.0.0.1:55745
Orders API: 2 requests, 1 without a rule, 0 failed · #1 1
```

각 줄은 시작 이후의 시간, 에뮬레이터, 답한 규칙(`#1`, 없으면 `—`), 요청과 받은 것, 걸린 시간,
보낸 사람입니다. 끝에는 각 에뮬레이터의 집계: 요청, 규칙이 받지 않은 수, 실패한 수, 중단을
겪거나 전달되지 않은 수(있는 경우), 규칙별 적중 수입니다.

종료 코드: <kbd>Ctrl</kbd>+<kbd>C</kbd> 또는 `--for`로 멈추면 `0`; 에뮬레이터가 유효하지 않으면
`2`; 포트가 사용 중이거나 열 수 없거나, 답하는 동안 소켓이 실패하면 `3`.

파이프라인에서는 백그라운드에서 시작하고, 그 대상으로 시스템을 테스트하고, 끝에 집계를
읽습니다:

```bash
signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
npm test          # the system under test, configured for the emulator's address
wait              # the last lines of mock.ndjson are the counts
```

서버에서는 `--server`로 시작한 에뮬레이터가 `signallab`이 정상 종료할 때 멈춥니다. 종료된
프로세스가 남긴 것은 서버의 인터페이스에서 멈출 수 있습니다.
[에뮬레이터](../tools/emulators.md)를 참고하십시오.

## `emulators` {#cli-emulators}

```text
signallab emulators [--library PATH]
```

라이브러리의 에뮬레이터를 나열합니다: id, 이름, 프로토콜, 주소, 규칙 수. `--library PATH`는
앱의 데이터 폴더에 있는 `emulators.json` 대신 다른 라이브러리 파일을 읽습니다.
`signallab emulate <id>`가 하나를 시작합니다.

라이브러리는 읽기만 합니다. 그런 파일이 없으면 — 앱은 처음 시작할 때 데이터 폴더의 것을
만듭니다 — 명령이 그렇게 알리고 `2`로 종료합니다.

## `templates` {#cli-templates}

```text
signallab templates
```

`run`과 `validate`가 이름으로 받는 기본 제공 템플릿을 나열합니다. 앱 자체의 것입니다:

| 이름 | 앱에서 | 매개변수 |
| --- | --- | --- |
| `empty` | [[ui:exp.templateEmpty]] | |
| `http-check` | [[ui:exp.templateHttp]] | |
| `status-branch` | [[ui:exp.templateBranch]] | |
| `parallel-flows` | [[ui:exp.templateParallel]] | |
| `osc-ping-reply` | [[ui:exp.templatePingReply]] | `device` = `127.0.0.1:9000` |
| `poll-until-ready` | [[ui:exp.templatePoll]] | `device` = `127.0.0.1:9000` |
| `flaky-api` | [[ui:exp.templateFlaky]] | `api` = `http://127.0.0.1:18080` |
| `fault-phases` | [[ui:exp.templateFaults]] | |
| `dependency-outage` | [[ui:exp.templateOutage]] | `api` = `http://127.0.0.1:18090` |
| `websocket-echo` | [[ui:exp.templateWsEcho]] | `service` = `ws://127.0.0.1:9001/echo` |

모든 템플릿은 루프백을 가리킵니다. `flaky-api`, `fault-phases`, `dependency-outage`는 자체
에뮬레이터를 가져오므로 다른 것이 수신하지 않아도 실행됩니다 — `signallab`이 동작하는 모습을
빠르게 보는 방법입니다.

## `nodes` {#cli-nodes}

```text
signallab nodes
```

실험이 무엇으로 구성되는지 JSON으로 출력합니다: 문서의 형태와 규칙, 레이블·설명·필드·출력·
엔진이 받아들이는 예제를 갖춘 모든 종류의 노드, `{{template}}` 언어, 부하 프로필, 에뮬레이터
문서. 어시스턴트가 [`signallab mcp`](mcp.md)를 통해 실험을 작성하려고 읽는 것이며, 사람에게는
[노드](../experiments/nodes.md)가 같은 내용을 더 많은 말로 전합니다.

## `mcp` {#cli-mcp}

```text
signallab mcp [OPTIONS]
```

Model Context Protocol을 통해 stdin과 stdout으로 Signal Lab을 AI 어시스턴트에게 제공합니다.
Claude Code, Claude Desktop, Cursor 또는 VS Code에서 설정하는 것, 그 옵션과 도구는 모두
[어시스턴트(MCP)](mcp.md)에 있습니다.

## `doctor` {#cli-doctor}

```text
signallab doctor [--server URL] [--token-file PATH]
```

Signal Lab과 장비 사이에 무엇이 있을 수 있는지 알리고, 무언가 있으면 `1`로 종료합니다. 그
줄은 명령줄의 나머지와 마찬가지로 [`--lang`](#language)을 따릅니다:

- 이 컴퓨터가 있는 네트워크: 이름과 주소;
- 데이터 폴더: 쓸 수 있는지 (아직 만들어지지 않았어도 괜찮습니다 — 앱이 처음 사용할 때
  만듭니다);
- 방화벽: Windows에서는 `signallab`과 데스크톱 앱 각각에 대해, 지금 이 컴퓨터가 있는 종류의
  네트워크(개인, 도메인 또는 공용)에서 규칙이 다른 컴퓨터를 들여보내는지, 또는 규칙이
  막는지 — 시스템 프롬프트에서 *취소*를 눌렀을 때 남는 상태입니다. Linux에서는 ufw 또는
  firewalld가 켜져 있는지, 포트를 여는 명령;
- `--server`(또는 `SIGNALLAB_SERVER`)와 함께: 서버가 응답하고 토큰을 받아들이는지.

```text
signallab [[version]]
Network: LAB-PC · 192.0.2.15
Data folder: C:\Users\lab\Documents\SignalLab — writable
Firewall · signallab (C:\…\Signal Lab\signallab.exe) · private network: not allowed yet — signallab firewall allow
Firewall · app (C:\…\Signal Lab\signal-lab.exe) · private network: allowed
✖ 1 thing in the way
```

`--json`은 같은 내용을 객체 하나로 출력합니다.
[문제 해결](../reference/troubleshooting.md)을 참고하십시오.

## `firewall` {#cli-firewall}

```text
signallab firewall allow [--public]
```

Windows에서 다른 컴퓨터가 Signal Lab에 닿도록 허용합니다 — 모니터나 대기가 장치를 들으려면
필요한 것입니다. Windows가 먼저 관리자 권한을 요청합니다. 그다음 `signallab`과 데스크톱
앱(옆에서 찾거나 설치 프로그램이 둔 곳에서 찾음)의 인바운드 규칙이 개인 및 도메인
네트워크에서 각각 허용 규칙 하나로 대체되며, 차단 규칙도 포함됩니다.

| 옵션 | 하는 일 |
| --- | --- |
| `--public` | 공용 네트워크에서도 — 행사장의 Wi-Fi는 흔히 그렇습니다. |

종료 코드: `0` 완료; 관리자 프롬프트를 거부하거나 변경이 실패하면 `3`. Linux에서는 아무것도
바뀌지 않습니다: 수신하는 포트를 여는 ufw 또는 firewalld 명령을 출력하고 `0`으로 종료합니다.

방화벽은 이것을 실행할 때만 바뀝니다. `signallab`의 다른 어떤 것도 그것을 건드리지 않습니다.

## `version` {#cli-version}

```text
signallab version
```

`signallab [[version]]`을 출력합니다 — `--json`을 쓰면 `{"version": "[[version]]"}`.
`signallab --version`도 같은 버전을 출력합니다.

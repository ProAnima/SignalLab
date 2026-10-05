---
title: CI 파이프라인
description: GitHub Actions, GitLab CI 또는 어떤 파이프라인에서든 Signal Lab 실험을 실행하고, 하나라도 실패하면 작업을 실패시키고, JUnit 보고서를 남깁니다.
---

# CI에서의 Signal Lab

설치를 가동시키는 실험은 그 회귀 테스트이기도 합니다. 파이프라인에서
[`signallab run`](cli.md#cli-run)은 창 없이 실행하고, 모든 단계를 출력하며, 모든 CI 시스템이
표시하는 JUnit 보고서를 쓰고, 작업이 이해하는 코드로 종료합니다:

| 종료 코드 | 파이프라인이 해야 할 일 |
| --- | --- |
| `0` | 계속 진행합니다: 모든 실험이 통과했습니다 |
| `1` | 실패시킵니다: 실험이 실행되어 실패했습니다 |
| `2` | 실패시킵니다: 실험이나 명령이 잘못되었습니다(검증 오류, 알 수 없는 매개변수, 없는 시크릿) |
| `3` | 실패 또는 재시도: 아무것도 실행할 수 없었습니다(서버에 닿을 수 없거나 토큰을 거부함, 포트를 열 수 없음) |

들어가는 방법은 네 가지입니다:

| 방법 | 실행되는 곳 |
| --- | --- |
| [GitHub Action](#github-actions) | Linux 러너, 서버 이미지에서. |
| [이미지](#docker) | 컨테이너를 실행하는 모든 CI: GitLab, Jenkins, Docker가 있는 셸. |
| [바이너리](#binary) | 모든 러너, Windows 포함. |
| [랩 서버](#lab-server) | 실행이 장비 옆의 Signal Lab 서버에서 이루어지고, 파이프라인은 보내기만 합니다. |

## GitHub Actions {#github-actions}

이 저장소는 GitHub Action이기도 합니다. `ghcr.io/proanima/signallab` 이미지에서 `signallab`을
실행하고, 실험이 실패하면 작업을 실패시키며, JUnit 보고서를 남깁니다:

```yaml
jobs:
  signallab:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: ProAnima/SignalLab@v[[version]]
        with:
          version: [[version]]
          experiments: tests/signallab/*.json
          params: |
            api=http://127.0.0.1:8080
        env:
          SIGNALLAB_SECRET_API_TOKEN: ${{ secrets.API_TOKEN }}

      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: signallab-junit
          path: signallab-junit.xml
```

통과하지 못한 실행은 실행 페이지에서 실패한 단계 옆에 오류 주석으로도 표시되며, 실험과 실패한
이유가 함께 나옵니다.

### 입력 {#action-inputs}

| 입력 | 하는 일 | 기본값 |
| --- | --- | --- |
| `experiments` | 실험 파일 또는 기본 제공 템플릿의 이름, 공백이나 줄로 구분합니다. `tests/*.json` 같은 패턴은 펼쳐집니다. 공백이 있는 경로는 지원하지 않습니다. | 필수 |
| `params` | 매개변수 값, `NAME=VALUE`, 한 줄에 하나. | |
| `profile` | 실험을 이 프로필로 실행합니다. | |
| `matrix` | 조합마다 한 번씩 실행합니다: `NAME=V1,V2`, 한 줄에 이름 하나. | |
| `matrix-file` | JSON 파일에서 조합을 읽습니다([실행의 매트릭스](#matrix) 참고). | |
| `fail-fast` | `"true"`: 통과하지 못한 첫 실행에서 멈춥니다. | `"false"` |
| `server` | 작업 대신 이 Signal Lab 서버에서 실행합니다, 예: `http://192.0.2.10:1430`. | |
| `token` | 서버의 접근 토큰. 시크릿을 넘기십시오. | |
| `junit` | JUnit 보고서가 가는 곳. | `signallab-junit.xml` |
| `timeout` | 실행에 허용되는 초, 1~300. | `300` |
| `version` | 이미지의 태그. | `latest` |
| `image` | 다른 레지스트리 또는 로컬에서 빌드한 이미지. `version`이 그 태그입니다. | `ghcr.io/proanima/signallab` |
| `lang` | 메시지와 보고서의 언어: `en`, `ru`, `es`, `fr`, `de`, `pt`, `zh`, `ja`, `ko`, `hi` 또는 `ar`. | `en` |
| `fail-on-error` | `"false"`: 단계를 실패시키지 않고 대신 `exit-code`를 읽습니다. | `"true"` |

경로(`experiments`, `matrix-file`, `junit`)는 단계의 작업 디렉터리를 기준으로 합니다.

::: tip
`version`을 테스트한 릴리스로 고정하십시오. `latest`는 새 안정 릴리스마다 옮겨 갑니다.
:::

### 출력 {#action-outputs}

| 출력 | 내용 |
| --- | --- |
| `junit` | JUnit 보고서의 경로. |
| `exit-code` | `signallab`의 종료 코드: `0`, `1`, `2` 또는 `3`. |

### 실패 후 계속하기 {#fail-on-error}

실패한 단계는 나머지 작업에 아무 출력도 넘기지 않습니다. 직접 결정하려면
`fail-on-error: "false"`로 설정하고 `exit-code`로 분기하십시오:

```yaml
      - id: lab
        uses: ProAnima/SignalLab@v[[version]]
        with:
          experiments: tests/signallab/smoke.json
          fail-on-error: "false"

      - if: steps.lab.outputs.exit-code == '1'
        run: echo "an experiment failed; the report is ${{ steps.lab.outputs.junit }}"

      - if: steps.lab.outputs.exit-code != '0'
        run: exit 1
```

### 액션의 시크릿 {#action-secrets}

실험의 `{{secret.NAME}}`은 `SIGNALLAB_SECRET_NAME`을 읽습니다. 그 변수들을 단계에(`env:`)
저장소의 시크릿에서 설정하십시오. 액션은 그 단계의 모든 `SIGNALLAB_SECRET_…` 변수를 이름으로
`signallab`에 넘깁니다. 값은 환경으로 오가며 명령줄에는 절대 오르지 않고, 모든 보고서와 로그
줄은 그 자리에 `••••`를 표시합니다. 실험이 필요로 하는데 설정되지 않은 시크릿은 아무것도
보내지기 전에 종료 코드 `2`로 단계를 실패시킵니다.

`token` 입력도 같은 방식으로 `SIGNALLAB_TOKEN`으로 오갑니다.

### 액션이 실행되는 방식 {#action-runs}

- Docker가 있는 **Linux 러너**가 필요합니다(`ubuntu-latest`에 있습니다). Windows 또는 macOS
  러너에서는 종료 코드 `2`와 주석과 함께 멈춥니다. 거기서는 [바이너리](#binary)를 쓰십시오.
- `signallab`은 이미지 안에서 **호스트 네트워킹**으로 실행됩니다: 러너가 닿는 것은 무엇이든
  닿습니다. 작업이 러너에서 시작한 서비스 — 테스트 대상 시스템, 또는 포트를 공개한
  `services:` 컨테이너 — 는 `127.0.0.1`에 있습니다.
- 러너의 사용자로 실행되며 작업 공간이 같은 경로에 마운트되므로, 보고서는 작업의 소유가
  됩니다.
- 위의 입력과 `--junit`을 그대로 `run`에 넘깁니다. `signallab`이 할 수 있는 다른 것 —
  `--report`, `--seed`, `--json`, `emulate` — 은 [이미지](#docker)를 직접 쓰십시오.

## 어떤 CI에서든 이미지 {#docker}

서버 이미지는 `signallab`을 `/usr/local/bin/signallab`으로 담고 있습니다. 그것을 쓰려면
엔트리포인트를 재정의하십시오.

**GitLab CI:**

```yaml
signallab:
  image:
    name: ghcr.io/proanima/signallab:[[version]]
    entrypoint: [""]
  script:
    - signallab run tests/signallab/*.json --junit signallab-junit.xml
  artifacts:
    when: always
    reports:
      junit: signallab-junit.xml
```

시크릿은 `SIGNALLAB_SECRET_<NAME>`이라는 CI/CD 변수입니다(가림으로 표시하십시오). 작업의
환경이 그것들을 그대로 `signallab`에 넘깁니다.

**셸 또는 어떤 스케줄러에서든 Docker**(cron, Jenkins, 배포 스크립트):

```bash
docker run --rm --network host \
  --user "$(id -u):$(id -g)" \
  -v "$PWD:/work" -w /work \
  -e SIGNALLAB_SECRET_API_TOKEN \
  --entrypoint signallab \
  ghcr.io/proanima/signallab:[[version]] \
  run tests/smoke.json --junit junit.xml
echo "signallab exited with $?"
```

- `--network host`는 실행이 호스트가 닿는 것에 닿게 합니다, 브로드캐스트와 멀티캐스트 포함 —
  Linux 호스트에서. 없으면 컨테이너는 유니캐스트로만 다른 호스트에 닿습니다.
- 이미지는 비특권 사용자(uid 10001)로 실행됩니다. `--user`를 쓰면 대신 당신으로 실행되어
  보고서를 당신 폴더에 쓸 수 있습니다. 없으면 폴더가 uid 10001에 대해 쓰기 가능해야 합니다.
- 값을 주지 않은 `-e NAME`은 그 변수를 당신의 환경에서 넘깁니다.

## 바이너리 {#binary}

모든 릴리스에는 `signallab`이 단독으로 들어 있습니다:
`signallab-<version>-linux-x64.tar.gz`와 `signallab-<version>-windows-x64.zip`이며, 릴리스의
`SHA256SUMS.txt`에 나열됩니다. Linux용은 Ubuntu 22.04에서 빌드되고 시스템의 OpenSSL
3(`libssl3`)를 사용합니다: 그 릴리스나 더 새로운 배포판에서 실행됩니다.

```yaml
      - name: Signal Lab
        run: |
          curl -fsSL -o signallab.tar.gz https://github.com/ProAnima/SignalLab/releases/download/v[[version]]/signallab-[[version]]-linux-x64.tar.gz
          tar -xzf signallab.tar.gz
          ./signallab run tests/signallab/smoke.json --junit signallab-junit.xml
```

Windows 러너에서는 zip을 풀고 `signallab.exe`를 같은 방식으로 실행하십시오. 데스크톱 앱이
설치된 컴퓨터에는 `signallab`이 이미 `PATH`에 있습니다.

## 랩 서버에서의 실행 {#lab-server}

설치의 네트워크에 있는 장비는 클라우드 러너가 아니라 랩에서 닿을 수 있습니다. 거기서
[Signal Lab 서버](../server/index.md)를 실행하고, 그 토큰을 CI 시크릿으로 보관하고, 실행을
그곳으로 보내십시오:

```bash
signallab run tests/stage.json --server http://192.0.2.10:1430 --token-file token.txt --junit junit.xml --report reports/
```

```yaml
      - uses: ProAnima/SignalLab@v[[version]]
        with:
          experiments: tests/signallab/stage.json
          server: http://192.0.2.10:1430
          token: ${{ secrets.SIGNALLAB_TOKEN }}
```

실험은 파이프라인의 체크아웃에서 옵니다. 서버는 자체 네트워크, 시크릿, 데이터 폴더로
실행하고, 단계를 되돌려 보내며, 보고서를 남깁니다(`--report`는 사본을 내려받습니다). 토큰은
`--token-file` 또는 `SIGNALLAB_TOKEN`에서 옵니다. 종료 코드는 같습니다. 닿을 수 없거나 토큰을
거부하는 서버는 `3`입니다. 러너는 서버에 닿을 수 있어야 합니다 — 랩의 자체 호스팅 러너, 또는
러너가 열 수 있는 서버 주소.

`signallab` 없이 서버에서 실행하려면 스크립트가 HTTP API를 직접 호출할 수 있습니다:
[HTTP로 실험 실행하기](../api/run.md)를 참고하십시오.

## 실행의 매트릭스 {#matrix}

실험 하나, 모든 대상: 각 조합은 별개의 실행이자 JUnit 보고서의 별개 테스트 스위트이며, 그
값에서 이름을 따옵니다.

```bash
signallab run tests/smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest
```

액션에서는 한 줄에 이름 하나:

```yaml
        with:
          experiments: tests/signallab/smoke.json
          matrix: |
            device=192.0.2.20:9000,192.0.2.21:9000
            user=admin,guest
          fail-fast: "true"
```

또는 파일에서, `--matrix-file`(액션에서는 `matrix-file`)로:

```json
[
  { "device": "192.0.2.20:9000", "user": "admin" },
  { "device": "192.0.2.21:9000", "user": "guest" }
]
```

모든 조합은 첫 조합이 무엇을 보내기 전에 검사되고, 한 명령에서 최대 256번의 실행이 나오며,
`--fail-fast`는 나머지를 시작하지 않고 둡니다 — 그것들은 JUnit 보고서에 건너뜀으로
나타납니다. 규칙은 [명령줄 페이지](cli.md#matrix)에 있습니다.

실험의 모든 프로필을 시도하려면 프로필마다 한 번씩 — 각각 한 단계, 또는 CI의 작업 매트릭스로 —
`--profile`과 함께 실행하십시오.

## JUnit 보고서 {#junit}

`--junit PATH`(액션은 항상 하나를 씁니다)는 실행마다 테스트 스위트를, 노드마다 테스트 케이스를
담습니다: 일어난 곳에서의 실패를 고른 언어로, 오류 코드와 기술적 세부 사항과 함께; 실행이 닿지
못한 노드를 건너뜀으로; 시드, 결과, 파일, 매트릭스 값을 속성으로. GitHub(보고 액션과 함께),
GitLab(`artifacts:reports:junit`), Jenkins, Azure DevOps가 그것을 테스트 결과로 표시합니다. 그
구조는 [명령줄 페이지](cli.md#reports)에 있습니다.

실패한 실행의 시드는 그 스위트의 속성과 로그에 있습니다: `--seed <that number>`는 같은 임의
값으로 다시 실행합니다.

## 테스트 대상 시스템이 호출하는 의존 대상 {#emulators}

CI에 없는 API, 장치 또는 브로커를 대상으로 자신의 시스템을 테스트하려면 Signal Lab이 그것을
연기하게 하십시오:

- **실험 안에서**, [[ui:exp.node.emulator]] 노드가 한 실행 동안 의존 대상 역할을 하고,
  [[ui:exp.node.wait_http]]가 당신의 시스템이 그것에 보낸 것을 검사합니다. 실행의 출력은 각
  에뮬레이터가 무엇을 요청받았는지로 끝납니다. [에뮬레이터](../tools/emulators.md)를
  참고하십시오.
- **자신의 테스트 주변에서**, `signallab emulate`가 그것들이 실행되는 동안 백그라운드에서
  응답하고, 그 집계가 무엇이 호출되었는지 알려 줍니다:

  ```bash
  signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
  npm test
  wait
  ```

## 스크립트를 위한 출력 {#json}

`--json`은 stdout에 한 줄에 JSON 객체 하나만 출력하고 다른 것은 없습니다: `started`, 모든
`step`, 전체 결과를 담은 `ended`, 그리고 `total`, `passed`, `failed`, `not_started`,
`exit_code`를 담은 `summary`. 오류는 엔진의 변하지 않는 `code`를 담으므로, 스크립트가 어떤
언어로든 그것으로 분기할 수 있습니다. [출력](cli.md#output)을 참고하십시오.

```bash
signallab run tests/smoke.json --json | jq -c 'select(.type == "ended") | {file, outcome, seed}'
```

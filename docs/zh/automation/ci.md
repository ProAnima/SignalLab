---
title: CI 流水线
description: 在 GitHub Actions、GitLab CI 或任何流水线中运行 Signal Lab 实验，实验失败时让作业失败，并保留一份 JUnit 报告。
---

# 流水线中的 Signal Lab

让一套装置完成安装的实验，同时也是它的回归测试。在流水线中，[`signallab run`](cli.md#cli-run) 无需窗口即可运行它们，打印每个步骤，写入一份每个 CI 系统都能显示的 JUnit 报告，并以作业能理解的退出码退出：

| 退出码 | 流水线应 |
| --- | --- |
| `0` | 继续：每个实验都通过 |
| `1` | 失败：某个实验运行后失败 |
| `2` | 失败：某个实验或命令有误（验证错误、未知参数、缺少机密） |
| `3` | 失败或重试：什么都无法运行（服务器无法访问或拒绝令牌、端口无法打开） |

有四种接入方式：

| 方式 | 运行位置 |
| --- | --- |
| [GitHub Action](#github-actions) | Linux 运行器，来自服务器镜像。 |
| [镜像](#docker) | 任何能运行容器的 CI：GitLab、Jenkins、带 Docker 的 shell。 |
| [可执行文件](#binary) | 任何运行器，Windows 也包括在内。 |
| [实验室服务器](#lab-server) | 运行发生在设备旁的 Signal Lab 服务器上；流水线只负责发送它们。 |

## GitHub Actions {#github-actions}

这个仓库本身也是一个 GitHub Action。它从镜像 `ghcr.io/proanima/signallab` 运行 `signallab`，实验失败时让作业失败，并留下一份 JUnit 报告：

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

未通过的运行还会在运行页面上、失败步骤旁边显示为一个错误标注，带有实验以及失败的原因。

### 输入 {#action-inputs}

| 输入 | 作用 | 默认值 |
| --- | --- | --- |
| `experiments` | 实验文件或内置模板的名称，以空格或换行分隔。诸如 `tests/*.json` 的模式会被展开；不支持包含空格的路径。 | 必填 |
| `params` | 参数值，`NAME=VALUE`，每行一个。 | |
| `profile` | 使用实验的这个配置文件运行。 | |
| `matrix` | 每种组合运行一次：`NAME=V1,V2`，每行一个名称。 | |
| `matrix-file` | 从 JSON 文件读取组合（参见[运行矩阵](#matrix)）。 | |
| `fail-fast` | `"true"`：在第一个未通过的运行处停止。 | `"false"` |
| `server` | 在这台 Signal Lab 服务器上运行，而不是在作业中，例如 `http://192.0.2.10:1430`。 | |
| `token` | 服务器的访问令牌。请传入一个机密。 | |
| `junit` | JUnit 报告的保存位置。 | `signallab-junit.xml` |
| `timeout` | 一次运行可以花费的秒数，1 到 300。 | `300` |
| `version` | 镜像的标签。 | `latest` |
| `image` | 另一个注册表或本地构建的镜像；`version` 是它的标签。 | `ghcr.io/proanima/signallab` |
| `lang` | 消息和报告所用的语言：`en`、`ru`、`es`、`fr`、`de`、`pt`、`zh`、`ja`、`ko`、`hi` 或 `ar`。 | `en` |
| `fail-on-error` | `"false"`：不让该步骤失败；改为读取 `exit-code`。 | `"true"` |

路径（`experiments`、`matrix-file`、`junit`）相对于该步骤的工作目录。

::: tip
请把 `version` 固定为您测试时所用的发布版本。`latest` 会随每个新的稳定版本移动。
:::

### 输出 {#action-outputs}

| 输出 | 内容 |
| --- | --- |
| `junit` | JUnit 报告的路径。 |
| `exit-code` | `signallab` 的退出码：`0`、`1`、`2` 或 `3`。 |

### 失败后继续 {#fail-on-error}

失败的步骤不会把任何输出交给作业的其余部分。要自行决定，请设置 `fail-on-error: "false"` 并根据 `exit-code` 分支：

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

### 动作中的机密 {#action-secrets}

实验的 `{{secret.NAME}}` 读取 `SIGNALLAB_SECRET_NAME`。请在步骤上设置这些变量（`env:`），取自仓库的机密。该动作会把它所在步骤的每个 `SIGNALLAB_SECRET_…` 变量按名称交给 `signallab`；值通过环境传递，绝不出现在命令行上，且每一份报告和每一行日志都在其位置显示 `••••`。实验需要而未被设置的机密会让该步骤以退出码 `2` 失败，且在发送任何内容之前。

`token` 输入以同样的方式传递，即 `SIGNALLAB_TOKEN`。

### 动作如何运行 {#action-runs}

- 它需要带 Docker 的 **Linux 运行器**（`ubuntu-latest` 就有）。在 Windows 或 macOS 运行器上，它会以退出码 `2` 停止并给出一个标注；在那里请使用[可执行文件](#binary)。
- `signallab` 在镜像中以**主机网络**运行：运行器能到达什么，它就能到达什么。作业在运行器上启动的服务——您的被测系统，或一个发布了端口的 `services:` 容器——位于 `127.0.0.1`。
- 它以运行器的用户身份运行，工作区挂载在同一路径，因此报告归属于该作业。
- 它把上述输入原样传给 `run`，外加 `--junit`。对于 `signallab` 能做的其他任何事——`--report`、`--seed`、`--json`、`emulate`——请直接使用[镜像](#docker)。

## 镜像，在任何 CI 中 {#docker}

服务器镜像把 `signallab` 放在 `/usr/local/bin/signallab`。要使用它，请覆盖入口点。

**GitLab CI：**

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

机密是名为 `SIGNALLAB_SECRET_<NAME>` 的 CI/CD 变量（请把它们标记为掩码）；作业的环境会原样把它们交给 `signallab`。

**Docker，从 shell 或任何调度器**（cron、Jenkins、部署脚本）：

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

- `--network host` 让运行能够到达主机所能到达的内容，包括广播和组播——在 Linux 主机上。不加它时，容器只能通过单播到达其他主机。
- 镜像以非特权用户（uid 10001）运行。`--user` 改为以您的身份运行它，这样它就能把报告写入您的文件夹；不加它时，该文件夹必须对 uid 10001 可写。
- 不带值的 `-e NAME` 会从您的环境传入该变量。

## 可执行文件 {#binary}

每个发布版本都单独提供 `signallab`：`signallab-<version>-linux-x64.tar.gz` 和 `signallab-<version>-windows-x64.zip`，列在发布版本的 `SHA256SUMS.txt` 中。Linux 版在 Ubuntu 22.04 上构建，使用系统的 OpenSSL 3（`libssl3`）：它可在该版本或更新的发行版上运行。

```yaml
      - name: Signal Lab
        run: |
          curl -fsSL -o signallab.tar.gz https://github.com/ProAnima/SignalLab/releases/download/v[[version]]/signallab-[[version]]-linux-x64.tar.gz
          tar -xzf signallab.tar.gz
          ./signallab run tests/signallab/smoke.json --junit signallab-junit.xml
```

在 Windows 运行器上，解压 zip 并以同样的方式运行 `signallab.exe`。安装了桌面应用的机器，其 `PATH` 中已经有 `signallab`。

## 在实验室服务器上运行 {#lab-server}

装置所在网络上的设备可以从实验室访问，却无法从云端运行器访问。在那里运行一个 [Signal Lab 服务器](../server/index.md)，把它的令牌保存为 CI 机密，并把运行发送给它：

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

实验来自流水线的检出；服务器使用它自己的网络、机密和数据文件夹运行它，把步骤流式送回并保留报告（`--report` 会下载一份副本）。令牌来自 `--token-file` 或 `SIGNALLAB_TOKEN`。退出码相同；无法访问或拒绝令牌的服务器为 `3`。运行器必须能够到达服务器——实验室中的自托管运行器，或运行器能够打开的服务器地址。

完全不使用 `signallab` 而在服务器上运行时，脚本可以直接调用其 HTTP API：参见[通过 HTTP 运行实验](../api/run.md)。

## 运行矩阵 {#matrix}

一个实验，每个目标：每种组合都是一次独立的运行，也是 JUnit 报告中一个独立的测试套件，以它们的值命名。

```bash
signallab run tests/smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest
```

在动作中，每行一个名称：

```yaml
        with:
          experiments: tests/signallab/smoke.json
          matrix: |
            device=192.0.2.20:9000,192.0.2.21:9000
            user=admin,guest
          fail-fast: "true"
```

或者来自文件，使用 `--matrix-file`（动作中的 `matrix-file`）：

```json
[
  { "device": "192.0.2.20:9000", "user": "admin" },
  { "device": "192.0.2.21:9000", "user": "guest" }
]
```

每种组合都会在第一个发送任何内容之前被检查，一条命令最多产生 256 次运行，而 `--fail-fast` 会让其余的不启动——它们会作为跳过出现在 JUnit 报告中。规则见[命令行页面](cli.md#matrix)。

要尝试实验的每个配置文件，请用 `--profile` 每个配置文件运行一次——各一个步骤，或使用 CI 的作业矩阵。

## JUnit 报告 {#junit}

`--junit PATH`（该动作总会写入一份）包含每次运行一个测试套件、每个节点一个测试用例：失败发生的位置、用所选语言写成的失败信息及其错误代码和技术细节；运行从未到达的节点作为跳过；以及作为属性的种子、结果、文件和矩阵值。GitHub（配合一个报告动作）、GitLab（`artifacts:reports:junit`）、Jenkins 和 Azure DevOps 都会把它显示为测试结果。其结构见[命令行页面](cli.md#reports)。

失败运行的种子在其套件的属性和日志中：`--seed <that number>` 会用相同的随机值再次运行它。

## 被测系统调用的依赖 {#emulators}

要让您自己的系统针对 CI 中并不存在的 API、设备或代理进行测试，请让 Signal Lab 扮演它：

- **在实验中**，[[ui:exp.node.emulator]] 节点为一次运行扮演该依赖，而 [[ui:exp.node.wait_http]] 检查您的系统向它发送了什么。运行的输出以每个模拟器被请求了什么结束。参见[模拟器](../tools/emulators.md)。
- **在您自己的测试周围**，`signallab emulate` 在它们运行时于后台应答，其计数会告诉您有什么被调用：

  ```bash
  signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
  npm test
  wait
  ```

## 给脚本的输出 {#json}

`--json` 在标准输出上每行打印一个 JSON 对象，不打印其他内容：`started`、每个 `step`、带完整结果的 `ended`，以及带 `total`、`passed`、`failed`、`not_started` 和 `exit_code` 的 `summary`。错误带有引擎稳定的 `code`，因此脚本可以用任何语言根据它分支。参见[输出](cli.md#output)。

```bash
signallab run tests/smoke.json --json | jq -c 'select(.type == "ended") | {file, outcome, seed}'
```

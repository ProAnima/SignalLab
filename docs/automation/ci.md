---
title: CI pipelines
description: Run Signal Lab experiments in GitHub Actions, GitLab CI or any pipeline, fail the job when one fails, and keep a JUnit report.
---

# Signal Lab in CI

The experiments that bring an installation up are also its regression tests.
In a pipeline, [`signallab run`](cli.md#cli-run) runs them without a window,
prints every step, writes a JUnit report every CI system shows, and exits with
a code the job understands:

| Exit code | The pipeline should |
| --- | --- |
| `0` | carry on: every experiment passed |
| `1` | fail: an experiment ran and failed |
| `2` | fail: an experiment or the command is wrong (a validation error, an unknown parameter, a missing secret) |
| `3` | fail or retry: nothing could run (the server cannot be reached or refuses the token, a port cannot be opened) |

There are four ways in:

| Way | Where it runs |
| --- | --- |
| [The GitHub Action](#github-actions) | A Linux runner, from the server image. |
| [The image](#docker) | Any CI that runs containers: GitLab, Jenkins, a shell with Docker. |
| [The binary](#binary) | Any runner, Windows included. |
| [A lab server](#lab-server) | The runs happen on a Signal Lab server next to the gear; the pipeline only sends them. |

## GitHub Actions {#github-actions}

The repository is also a GitHub Action. It runs `signallab` from the image
`ghcr.io/proanima/signallab`, fails the job when an experiment fails, and
leaves a JUnit report:

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

A run that does not pass is also an error annotation on the run's page, next to
the failing step, with the experiment and why it failed.

### Inputs {#action-inputs}

| Input | What it does | Default |
| --- | --- | --- |
| `experiments` | Experiment files or names of bundled templates, separated by spaces or lines. Patterns such as `tests/*.json` are expanded; a path with spaces in it is not supported. | required |
| `params` | Parameter values, `NAME=VALUE`, one per line. | |
| `profile` | Run with this profile of the experiments. | |
| `matrix` | Run once per combination: `NAME=V1,V2`, one name per line. | |
| `matrix-file` | Combinations from a JSON file (see [A matrix of runs](#matrix)). | |
| `fail-fast` | `"true"`: stop at the first run that does not pass. | `"false"` |
| `server` | Run on this Signal Lab server instead of in the job, e.g. `http://192.0.2.10:1430`. | |
| `token` | The server's access token. Pass a secret. | |
| `junit` | Where the JUnit report goes. | `signallab-junit.xml` |
| `timeout` | Seconds a run may take, 1 to 300. | `300` |
| `version` | The image's tag. | `latest` |
| `image` | Another registry or a locally built image; `version` is its tag. | `ghcr.io/proanima/signallab` |
| `lang` | The language of the messages and the report: `en`, `ru`, `es`, `fr`, `de`, `pt`, `zh`, `ja`, `ko`, `hi` or `ar`. | `en` |
| `fail-on-error` | `"false"`: do not fail the step; read `exit-code` instead. | `"true"` |

Paths (`experiments`, `matrix-file`, `junit`) are relative to the step's
working directory.

::: tip
Pin `version` to the release you tested with. `latest` moves to every new
stable release.
:::

### Outputs {#action-outputs}

| Output | What it is |
| --- | --- |
| `junit` | The path of the JUnit report. |
| `exit-code` | `signallab`'s exit code: `0`, `1`, `2` or `3`. |

### Going on after a failure {#fail-on-error}

A step that fails hands no outputs to the rest of the job. To decide yourself,
set `fail-on-error: "false"` and branch on `exit-code`:

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

### Secrets in the action {#action-secrets}

An experiment's `{{secret.NAME}}` reads `SIGNALLAB_SECRET_NAME`. Set those
variables on the step (`env:`), from the repository's secrets. The action hands
every `SIGNALLAB_SECRET_…` variable of its step to `signallab` by name; the
values travel in the environment, never on a command line, and every report and
log line shows `••••` in their place. A secret the experiment needs that is not
set fails the step with exit code `2`, before anything is sent.

The `token` input travels the same way, as `SIGNALLAB_TOKEN`.

### How the action runs {#action-runs}

- It needs a **Linux runner** with Docker (`ubuntu-latest` has it). On a
  Windows or macOS runner it stops with exit code `2` and an annotation; there,
  use [the binary](#binary).
- `signallab` runs in the image with **host networking**: whatever the runner
  reaches, it reaches. A service the job started on the runner — your system
  under test, or a `services:` container with a published port — is at
  `127.0.0.1`.
- It runs as the runner's user, with the workspace mounted at the same path,
  so the report belongs to the job.
- It passes `run` exactly the inputs above, and `--junit`. For anything else
  `signallab` can do — `--report`, `--seed`, `--json`, `emulate` — use
  [the image](#docker) directly.

## The image, in any CI {#docker}

The server image carries `signallab` as `/usr/local/bin/signallab`. Override the
entrypoint to use it.

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

Secrets are CI/CD variables named `SIGNALLAB_SECRET_<NAME>` (mark them masked);
the job's environment hands them to `signallab` as they are.

**Docker, from a shell or any scheduler** (cron, Jenkins, a deploy script):

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

- `--network host` lets the runs reach what the host reaches, broadcast and
  multicast included — on a Linux host. Without it the container reaches other
  hosts by unicast only.
- The image runs as an unprivileged user (uid 10001). `--user` runs it as you
  instead, so it can write the report into your folder; without it, the folder
  must be writable for uid 10001.
- `-e NAME` without a value passes that variable from your environment.

## The binary {#binary}

Every release carries `signallab` on its own:
`signallab-<version>-linux-x64.tar.gz` and `signallab-<version>-windows-x64.zip`,
listed in the release's `SHA256SUMS.txt`. The Linux one is built on Ubuntu
22.04 and uses the system's OpenSSL 3 (`libssl3`): it runs on that release or a
newer distribution.

```yaml
      - name: Signal Lab
        run: |
          curl -fsSL -o signallab.tar.gz https://github.com/ProAnima/SignalLab/releases/download/v[[version]]/signallab-[[version]]-linux-x64.tar.gz
          tar -xzf signallab.tar.gz
          ./signallab run tests/signallab/smoke.json --junit signallab-junit.xml
```

On a Windows runner, unpack the zip and run `signallab.exe` the same way. A
machine with the desktop app installed has `signallab` on its `PATH` already.

## Runs on a lab server {#lab-server}

Gear on an installation's network is reachable from the lab, not from a cloud
runner. Run a [Signal Lab server](../server/index.md) there, keep its token as a
CI secret, and send the runs to it:

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

The experiment comes from the pipeline's checkout; the server runs it with its
own network, secrets and data folder, streams the steps back and keeps the
report (`--report` downloads a copy). The token comes from `--token-file` or
`SIGNALLAB_TOKEN`. The exit codes are the same; a server that cannot be reached
or refuses the token is `3`. The runner must be able to reach the server — a
self-hosted runner in the lab, or a server address the runner can open.

To run on the server without `signallab` at all, a script can call its HTTP API
directly: see [Running an experiment over HTTP](../api/run.md).

## A matrix of runs {#matrix}

One experiment, every target: each combination is a run of its own and a test
suite of its own in the JUnit report, named after its values.

```bash
signallab run tests/smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest
```

In the action, one name per line:

```yaml
        with:
          experiments: tests/signallab/smoke.json
          matrix: |
            device=192.0.2.20:9000,192.0.2.21:9000
            user=admin,guest
          fail-fast: "true"
```

Or from a file, with `--matrix-file` (`matrix-file` in the action):

```json
[
  { "device": "192.0.2.20:9000", "user": "admin" },
  { "device": "192.0.2.21:9000", "user": "guest" }
]
```

Every combination is checked before the first one sends anything, at most 256
runs come from one command, and `--fail-fast` leaves the rest unstarted — they
appear in the JUnit report as skipped. The rules are on
[the command line's page](cli.md#matrix).

To try every profile of an experiment, run once per profile — one step each, or
a job matrix of your CI — with `--profile`.

## JUnit reports {#junit}

`--junit PATH` (the action always writes one) holds a test suite per run and a
test case per node: the failure where it happened, in the chosen language, with
its error code and technical detail; the nodes a run never reached as skipped;
the seed, the outcome, the file and the matrix values as properties. GitHub
(with a reporting action), GitLab (`artifacts:reports:junit`), Jenkins and
Azure DevOps show it as test results. Its structure is on
[the command line's page](cli.md#reports).

A failed run's seed is in its suite's properties and in the log:
`--seed <that number>` runs it again with the same random values.

## Dependencies the system under test calls {#emulators}

To test your own system against an API, a device or a broker that is not there
in CI, let Signal Lab play it:

- **Inside an experiment**, an [[ui:exp.node.emulator]] node plays the
  dependency for one run, and [[ui:exp.node.wait_http]] checks what your system
  sent it. The run's output ends with what each emulator was asked. See
  [Emulators](../tools/emulators.md).
- **Around your own tests**, `signallab emulate` answers in the background
  while they run, and its counts tell you what was called:

  ```bash
  signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
  npm test
  wait
  ```

## Output for a script {#json}

`--json` prints one JSON object per line on stdout and nothing else: `started`,
every `step`, `ended` with the whole result, and a `summary` with `total`,
`passed`, `failed`, `not_started` and `exit_code`. Errors carry the engine's
stable `code`, so a script can branch on it in any language. See
[Output](cli.md#output).

```bash
signallab run tests/smoke.json --json | jq -c 'select(.type == "ended") | {file, outcome, seed}'
```

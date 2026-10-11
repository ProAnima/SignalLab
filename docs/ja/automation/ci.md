---
title: CI パイプライン
description: GitHub Actions、GitLab CI、そのほかのパイプラインで Signal Lab の実験を実行し、失敗したらジョブを失敗させ、JUnit レポートを残します。
---

# CI での Signal Lab

設備を立ち上げる実験は、そのままその回帰テストにもなります。パイプラインでは、[`signallab run`](cli.md#cli-run) がウィンドウなしで実験を実行し、すべてのステップを出力し、どの CI システムでも表示できる JUnit レポートを書き、ジョブが理解できるコードで終了します。

| 終了コード | パイプラインの対応 |
| --- | --- |
| `0` | 続行する: すべての実験が成功した |
| `1` | 失敗させる: 実験が実行されて失敗した |
| `2` | 失敗させる: 実験またはコマンドが正しくない（検証エラー、不明なパラメーター、シークレットの不足） |
| `3` | 失敗させるか再試行する: 何も実行できなかった（サーバーに到達できない、またはトークンを拒否された、ポートを開けない） |

使い方は 4 通りあります。

| 方法 | 実行される場所 |
| --- | --- |
| [GitHub Action](#github-actions) | Linux ランナー。サーバーイメージから実行されます。 |
| [イメージ](#docker) | コンテナーを実行できるすべての CI: GitLab、Jenkins、Docker のあるシェル。 |
| [バイナリ](#binary) | Windows を含む、すべてのランナー。 |
| [ラボサーバー](#lab-server) | 機器の隣にある Signal Lab サーバーで実行されます。パイプラインは実行を送るだけです。 |

## GitHub Actions {#github-actions}

このリポジトリは GitHub Action でもあります。イメージ `ghcr.io/proanima/signallab` から `signallab` を実行し、実験が失敗するとジョブを失敗させ、JUnit レポートを残します。

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

成功しなかった実行は、その実行のページの、失敗したステップの隣に、実験名と失敗の理由を示すエラーアノテーションとしても表示されます。

### 入力 {#action-inputs}

| 入力 | 動作 | 既定値 |
| --- | --- | --- |
| `experiments` | 実験ファイル、または同梱テンプレートの名前。空白または改行で区切ります。`tests/*.json` のようなパターンは展開されます。空白を含むパスはサポートされません。 | 必須 |
| `params` | パラメーター値。`NAME=VALUE` を 1 行に 1 つ。 | |
| `profile` | 実験のこのプロファイルで実行します。 | |
| `matrix` | 組み合わせごとに 1 回実行します。`NAME=V1,V2` を 1 行に 1 つの名前で。 | |
| `matrix-file` | JSON ファイルからの組み合わせ（[実行のマトリクス](#matrix)を参照）。 | |
| `fail-fast` | `"true"`: 成功しなかった最初の実行で止めます。 | `"false"` |
| `server` | ジョブの中ではなく、この Signal Lab サーバーで実行します。例: `http://192.0.2.10:1430`。 | |
| `token` | サーバーのアクセストークン。シークレットを渡してください。 | |
| `junit` | JUnit レポートの出力先。 | `signallab-junit.xml` |
| `timeout` | 1 回の実行にかけてよい秒数。1〜300。 | `300` |
| `version` | イメージのタグ。 | `latest` |
| `image` | 別のレジストリ、またはローカルでビルドしたイメージ。`version` がそのタグになります。 | `ghcr.io/proanima/signallab` |
| `lang` | メッセージとレポートの言語: `en`、`ru`、`es`、`fr`、`de`、`pt`、`zh`、`ja`、`ko`、`hi`、`ar`。 | `en` |
| `fail-on-error` | `"false"`: ステップを失敗させません。代わりに `exit-code` を読んでください。 | `"true"` |

パス（`experiments`、`matrix-file`、`junit`）は、ステップの作業ディレクトリからの相対パスです。

::: tip
`version` は、テストに使ったリリースに固定してください。`latest` は、新しい安定版リリースのたびに移動します。
:::

### 出力 {#action-outputs}

| 出力 | 内容 |
| --- | --- |
| `junit` | JUnit レポートのパス。 |
| `exit-code` | `signallab` の終了コード: `0`、`1`、`2`、`3`。 |

### 失敗の後も続ける {#fail-on-error}

失敗したステップは、ジョブの残りの部分に出力を渡しません。自分で判断するには、`fail-on-error: "false"` を設定して、`exit-code` で分岐します。

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

### アクションでのシークレット {#action-secrets}

実験の `{{secret.NAME}}` は `SIGNALLAB_SECRET_NAME` を読みます。これらの変数は、リポジトリのシークレットから、ステップに（`env:` で）設定してください。アクションは、そのステップのすべての `SIGNALLAB_SECRET_…` 変数を、名前で `signallab` に渡します。値は環境変数として渡され、コマンドラインに現れることはなく、すべてのレポートとログの行では、その代わりに `••••` が表示されます。実験が必要とするシークレットが設定されていないと、何かが送信される前に、終了コード `2` でステップが失敗します。

`token` 入力も同じ方法で、`SIGNALLAB_TOKEN` として渡されます。

### アクションの動作 {#action-runs}

- Docker のある **Linux ランナー**が必要です（`ubuntu-latest` にはあります）。Windows や macOS のランナーでは、終了コード `2` とアノテーションを出して止まります。そこでは[バイナリ](#binary)を使ってください。
- `signallab` は、イメージの中で**ホストネットワーク**を使って実行されます。ランナーが到達できるものには、すべて到達できます。ジョブがランナー上で起動したサービス（テスト対象のシステムや、ポートを公開した `services:` コンテナー）は `127.0.0.1` にあります。
- ランナーのユーザーとして、ワークスペースを同じパスにマウントして実行されるので、レポートはジョブのものになります。
- `run` には、上記の入力と `--junit` だけが渡されます。`signallab` のほかの機能（`--report`、`--seed`、`--json`、`emulate`）を使うには、[イメージ](#docker)を直接使ってください。

## イメージを任意の CI で {#docker}

サーバーイメージには、`/usr/local/bin/signallab` として `signallab` が入っています。使うにはエントリポイントを上書きします。

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

シークレットは、`SIGNALLAB_SECRET_<NAME>` という名前の CI/CD 変数にします（マスク設定にしてください）。ジョブの環境が、それをそのまま `signallab` に渡します。

**シェルや任意のスケジューラー（cron、Jenkins、デプロイスクリプト）からの Docker:**

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

- `--network host` を付けると、実行はホストが到達できるものに到達できます。Linux ホストでは、ブロードキャストやマルチキャストも含みます。付けない場合、コンテナーはほかのホストにユニキャストでしか到達できません。
- イメージは、権限のないユーザー（uid 10001）として実行されます。`--user` を指定すると、代わりに自分のユーザーとして実行されるので、レポートを自分のフォルダーに書き込めます。指定しない場合、そのフォルダーは uid 10001 が書き込める必要があります。
- 値を付けない `-e NAME` は、自分の環境からその変数を渡します。

## バイナリ {#binary}

すべてのリリースには、`signallab` 単体のものも付いています。`signallab-<version>-linux-x64.tar.gz` と `signallab-<version>-windows-x64.zip` で、リリースの `SHA256SUMS.txt` に一覧されています。Linux 版は Ubuntu 22.04 でビルドされ、システムの OpenSSL 3（`libssl3`）を使うので、そのリリースか、それより新しいディストリビューションで動作します。

```yaml
      - name: Signal Lab
        run: |
          curl -fsSL -o signallab.tar.gz https://github.com/ProAnima/SignalLab/releases/download/v[[version]]/signallab-[[version]]-linux-x64.tar.gz
          tar -xzf signallab.tar.gz
          ./signallab run tests/signallab/smoke.json --junit signallab-junit.xml
```

Windows のランナーでは、zip を展開して、同じように `signallab.exe` を実行します。デスクトップアプリがインストールされているマシンでは、`signallab` はすでに `PATH` に入っています。

## ラボサーバーでの実行 {#lab-server}

設備のネットワーク上の機器には、ラボからは到達できますが、クラウドのランナーからは到達できません。そこで [Signal Lab サーバー](../server/index.md)を動かし、そのトークンを CI のシークレットとして保管して、実行をそこに送ります。

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

実験はパイプラインのチェックアウトから渡され、サーバーが自分のネットワーク、シークレット、データフォルダーで実行し、ステップを送り返して、レポートを保管します（`--report` でコピーをダウンロードします）。トークンは `--token-file` または `SIGNALLAB_TOKEN` から渡されます。終了コードは同じで、サーバーに到達できない、またはトークンを拒否された場合は `3` です。ランナーはサーバーに到達できなければなりません。ラボにあるセルフホストのランナーか、ランナーが開けるサーバーのアドレスを使ってください。

`signallab` をまったく使わずにサーバーで実行するには、スクリプトからその HTTP API を直接呼び出せます。[HTTP での実験の実行](../api/run.md)を参照してください。

## 実行のマトリクス {#matrix}

1 つの実験を、すべての宛先に対して実行します。各組み合わせは、独立した実行であり、JUnit レポートでは、その値から名前が付いた独自のテストスイートになります。

```bash
signallab run tests/smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest
```

アクションでは、1 行に 1 つの名前を書きます。

```yaml
        with:
          experiments: tests/signallab/smoke.json
          matrix: |
            device=192.0.2.20:9000,192.0.2.21:9000
            user=admin,guest
          fail-fast: "true"
```

あるいはファイルから、`--matrix-file`（アクションでは `matrix-file`）を使います。

```json
[
  { "device": "192.0.2.20:9000", "user": "admin" },
  { "device": "192.0.2.21:9000", "user": "guest" }
]
```

すべての組み合わせは、最初の 1 つが何かを送信する前にチェックされ、1 つのコマンドからの実行は最大 256 回で、`--fail-fast` は残りを未開始のままにします。それらは JUnit レポートにスキップとして表示されます。ルールは[コマンドラインのページ](cli.md#matrix)にあります。

実験のすべてのプロファイルを試すには、`--profile` を指定して、プロファイルごとに 1 回ずつ実行します。ステップを 1 つずつ用意するか、CI のジョブマトリクスを使います。

## JUnit レポート {#junit}

`--junit PATH`（アクションは常に書き出します）には、実行ごとのテストスイートと、ノードごとのテストケースが入ります。失敗は、起きた場所に、選択した言語で、エラーコードと技術的な詳細付きで記録され、実行が到達しなかったノードはスキップとして、シード、結果、ファイル、マトリクスの値はプロパティとして記録されます。GitHub（レポート用のアクションを使う場合）、GitLab（`artifacts:reports:junit`）、Jenkins、Azure DevOps は、これをテスト結果として表示します。構造は[コマンドラインのページ](cli.md#reports)にあります。

失敗した実行のシードは、そのスイートのプロパティとログに記録されています。`--seed <that number>` を指定すれば、同じ乱数値でもう一度実行できます。

## テスト対象のシステムが呼び出す依存先 {#emulators}

CI にない API、デバイス、ブローカーに対して自分のシステムをテストするには、Signal Lab にその役を演じさせます。

- **実験の中で**は、[[ui:exp.node.emulator]] ノードが 1 回の実行のあいだ依存先を演じ、[[ui:exp.node.wait_http]] が、自分のシステムがそこに何を送ったかをチェックします。実行の出力は、各エミュレーターが何を尋ねられたかで終わります。[エミュレーター](../tools/emulators.md)を参照してください。
- **自分のテストの周囲で**は、`signallab emulate` がテストの実行中にバックグラウンドで応答し、その件数から何が呼び出されたかが分かります。

  ```bash
  signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
  npm test
  wait
  ```

## スクリプト向けの出力 {#json}

`--json` は、stdout に 1 行に 1 つの JSON オブジェクトだけを出力します。`started`、すべての `step`、結果全体を持つ `ended`、そして `total`、`passed`、`failed`、`not_started`、`exit_code` を持つ `summary` です。エラーにはエンジンの安定した `code` が付いているので、スクリプトはどの言語でもそれで分岐できます。[出力](cli.md#output)を参照してください。

```bash
signallab run tests/smoke.json --json | jq -c 'select(.type == "ended") | {file, outcome, seed}'
```

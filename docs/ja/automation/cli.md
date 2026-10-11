---
title: コマンドライン
description: signallab は、ウィンドウなしで実験を実行し、単発のメッセージを送り、エミュレーターを動かし、ネットワークを調べます。ターミナル、スクリプト、CI ジョブから使えます。
---

# コマンドライン: `signallab`

`signallab` は、ウィンドウのない Signal Lab です。実験を最後まで実行してスクリプトが解釈できるコードで終了し、OSC メッセージ、データグラム、HTTP リクエスト、WebSocket メッセージ、MQTT パブリッシュを 1 つ送信し、ライブラリのシグナルを送り、止めるまでエミュレーターを動かし、このマシンと機器の間に何があるかを教えてくれます。

エンジンはアプリと同じです。コマンドラインからの実行は、同じステップをたどり、同じレポートを書き、インターフェイスの言語で同じことを伝えます。`--server` を付けると、実行は代わりに [Signal Lab サーバー](../server/index.md)で、そのサーバーのネットワークとシークレットを使って行われます。

```bash
signallab run tests/smoke.json --param api=http://127.0.0.1:8080 --junit junit.xml
signallab run flaky-api                                # 同梱のテンプレートを名前で
signallab validate tests/*.json                        # エディターのチェック。何も送信しない
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab fire "Fader value"
signallab emulate tests/orders-api.json --for 120      # 依存先を 2 分間動かす
signallab doctor                                       # ファイアウォール、ネットワーク、データフォルダー
```

パイプラインについては [CI での Signal Lab](ci.md) を、AI アシスタントについては [`signallab mcp`](mcp.md) を参照してください。

## インストール {#install}

| 場所 | `signallab` の入手方法 |
| --- | --- |
| Windows、セットアップ（`.exe`） | アプリと同じ場所にインストールされ、そのフォルダーが `PATH` に追加されます。追加先は、「自分用」のインストールでは自分の分、「全員用」ではマシン全体の分です。インストール後は新しいターミナルを開いてください。セットアップの `/NOPATH` スイッチを付けると、`PATH` はそのままになります。 |
| Windows、`.msi` | アプリと同じ場所にインストールされ、アプリがインストールされている間は、そのインストールフォルダーがマシンの `PATH` に入ります。 |
| Linux、`.deb` と `.rpm` | `/usr/bin/signallab`。 |
| Linux、AppImage | 含まれません。下のアーカイブを使ってください。 |
| アプリなしの任意のマシン | すべてのリリースに `signallab-<version>-windows-x64.zip` と `signallab-<version>-linux-x64.tar.gz` が付いており、それぞれにプログラムとライセンスが入っています。同じページの `SHA256SUMS.txt` にチェックサムが一覧されています。 |
| サーバーイメージ | `ghcr.io/proanima/signallab` の `/usr/local/bin/signallab`（[CI](ci.md#docker)を参照）。 |

次のコマンドで確認できます。

```bash
signallab version
```

## コマンド {#commands}

| コマンド | 動作 |
| --- | --- |
| [`run`](#cli-run) | 実験を順に実行します。すべてが成功した場合にだけ終了コード 0 で終わります。 |
| [`validate`](#cli-validate) | 実行前にエディターが行うのと同じ実験のチェックを行います。何も送信しません。 |
| [`send`](#cli-send) | メッセージを 1 つ送信します: `osc`、`udp`、`http`、`ws`、`mqtt`。 |
| [`fire`](#cli-fire) | シグナルライブラリのシグナルを、ID または名前で送信します。 |
| [`emulate`](#cli-emulate) | HTTP API、OSC・UDP・TCP のデバイス、MQTT ブローカーを、<kbd>Ctrl</kbd>+<kbd>C</kbd> または `--for` まで動かします。 |
| [`emulators`](#cli-emulators) | アプリのライブラリにあるエミュレーターを一覧表示します。 |
| [`templates`](#cli-templates) | 同梱の実験テンプレートを一覧表示します。 |
| [`nodes`](#cli-nodes) | すべての種類のノードを JSON で説明します。 |
| [`mcp`](#cli-mcp) | Model Context Protocol で、Signal Lab を AI アシスタントに提供します。 |
| [`doctor`](#cli-doctor) | ファイアウォール、ネットワーク、データフォルダー、サーバーをチェックします。 |
| [`firewall`](#cli-firewall) | `firewall allow`: Windows ファイアウォールを通して、ほかのマシンが Signal Lab に到達できるようにします。 |
| [`version`](#cli-version) | バージョンを表示します。 |

`signallab help <command>` または `signallab <command> --help` で、コマンドのオプションが表示されます。

## すべてのコマンドのオプション {#global-options}

| オプション | 動作 | 既定値 |
| --- | --- | --- |
| `--lang <code>` | メッセージの言語: `en`、`ru`、`es`、`fr`、`de`、`pt`、`zh`、`ja`、`ko`、`hi`、`ar`。 | `SIGNALLAB_LANG`、なければロケール、なければ `en` |
| `--json` | テキストの代わりに、機械向けの出力を stdout に出します（[出力](#output)を参照）。 | オフ |
| `-h`, `--help` | コマンドのヘルプ。 | |
| `-V`, `--version` | バージョン。コマンドの前に `signallab --version` として使います。 | |

`--lang` と `--json` は、行のどこに置いても構いません。`signallab --json run smoke.json` と `signallab run smoke.json --json` は同じです。

### 言語 {#language}

メッセージ、ステップのテキスト、失敗、JUnit レポートは、インターフェイス自身の文言、複数形、数値の書式を使います。言語は、次のうち最初に見つかったものです。

1. `--lang`。
2. `SIGNALLAB_LANG`（`ru`、`ru-RU`、`ru_RU.UTF-8` はいずれもロシア語を意味します）。
3. ロケール: `LC_ALL`、`LC_MESSAGES`、`LANG` のうち、最初に設定されているもの。
4. 英語。

::: tip
Windows のターミナルでは、通常、ロケール変数がどれも設定されていないので、`SIGNALLAB_LANG` を設定するか `--lang` を渡さない限り、`signallab` は英語で表示します。
:::

### 人向けの出力と機械向けの出力 {#output}

`--json` なしでは、結果は **stdout** に出力され、人が途中で読むものはすべて **stderr** に出力されます。実行中のステップ、失敗の理由、レポートの場所です。`signallab run … 2>/dev/null` とすれば、実行ごとに判定の 1 行だけが残ります。

`--json` を付けると、stdout には JSON だけが出力され、stderr は静かなままです。

| コマンド | `--json` が stdout に出力するもの |
| --- | --- |
| `run` | 1 行に 1 つのオブジェクト: `started`、ステップごとの `step`、`ended`（実行の結果）、そして `summary`。開始できなかった実行には `error`。[`run` の出力](#run-output)を参照。 |
| `validate` | 実験ごとに 1 行に 1 つのオブジェクト: `valid`、そして `profile_issues` または `error`。 |
| `send`、`fire` | `{"type": "sent", "result": …}`。`send http` は `{"type": "response", "response": …}`、`send ws` は `{"type": "exchange", "result": …}` を出力します。 |
| `emulate` | 1 行に 1 つのオブジェクト: `started`、リクエストごとの `exchange`、エミュレーターごとの `summary`。`--check` では `valid`。 |
| `emulators` | `{"path": …, "emulators": [{id, name, protocol, bind, rules, note}, …]}`。 |
| `templates` | `[{"name": …, "experiment": …}, …]`。 |
| `doctor` | 1 つのオブジェクト: `version`、`network`、`data_dir`、`firewall`、`server`、`problems`。 |
| `version` | `{"version": "…"}`。 |
| `nodes` | `--json` の有無にかかわらず、常に JSON。 |

失敗は `{"type": "error", "error": {…}, "exit_code": N}` です。`error` はエンジンのエラーで、安定した `code`（`transport.refused` や `secret.missing` など）、その `params`、対象の `node` と `field` を持ちます。スクリプトは、どの言語でも `error.code` で分岐できます。すべてのコードの文言は[エラーメッセージ](../reference/errors.md)に一覧されています。

### 終了コード {#exit-codes}

| コード | 意味 |
| --- | --- |
| `0` | すべての実験が成功した。送信が成功した。妨げになるものがない。 |
| `1` | 実験が実行されて失敗した、時間切れになった、または停止された。送信が失敗した（接続拒否、応答なし、予期しないステータス）。`doctor` が妨げになるものを見つけた。 |
| `2` | コマンドまたはドキュメントが正しくない: 引数、読み取れないファイル、検証エラー、不明なパラメーター、シークレットの不足。 |
| `3` | 実験の外の理由で何も実行できなかった: サーバーに到達できない、またはトークンを拒否された、ポートを開けない、資格情報ストアが失敗した。 |

複数の実験がある場合、最も深刻な結果が決め手になります。順に `2`、`3`、`1`、`0` です。

### 環境変数 {#environment}

| 変数 | 動作 |
| --- | --- |
| `SIGNALLAB_LANG` | `--lang` が指定されていないときの言語。 |
| `LC_ALL`、`LC_MESSAGES`、`LANG` | 上の 2 つのどちらも設定されていないときの言語。 |
| `SIGNALLAB_SERVER` | `run`、`validate`、`emulate`、`mcp`、`doctor` のためのサーバー。`--server` で指定するのと同じです。 |
| `SIGNALLAB_TOKEN` | トークンファイルが指定されていないときの、サーバーのアクセストークン。 |
| `SIGNALLAB_TOKEN_FILE` | サーバーのトークンを保持するファイル。`--token-file` で指定するのと同じです。 |
| `SIGNALLAB_SECRET_<NAME>` | このプロセスでの実行における、シークレット `NAME` の値（[シークレット](#secrets)を参照）。 |
| `SIGNALLAB_DATA_DIR` | アプリのデータフォルダー。`fire`、`emulators`、`emulate`、`mcp`、`doctor` が既定でここを見ます。未設定の場合は、ホームフォルダーの `Documents/SignalLab` です。 |
| `GITHUB_ACTIONS` | `true` の場合、成功しなかった実行は `::error` アノテーションとしても出力され、GitHub がその実行のページに表示します。 |

## `run` {#cli-run}

```text
signallab run [OPTIONS] <FILE>...
```

実験を順に実行し、すべてが成功した場合にだけ終了コード `0` で終わります。

| オプション | 動作 | 既定値 |
| --- | --- | --- |
| `<FILE>...` | 実験ファイル、または[同梱テンプレート](#cli-templates)の名前。 | 必須 |
| `-p`, `--param NAME=VALUE` | この実行のパラメーター値。複数指定するには繰り返します。 | ドキュメントの値 |
| `--profile NAME` | このプロファイルで実行します。指定したすべての実験にそのプロファイルが必要です。`""` は既定値で実行します。 | ドキュメントのプロファイル |
| `-m`, `--matrix NAME=V1,V2` | 値ごとに 1 回実行します。複数の名前には繰り返します（[実行のマトリクス](#matrix)を参照）。 | |
| `--matrix-file PATH` | JSON ファイルからの組み合わせ。 | |
| `--seed N` | 乱数値のシード。0 から 9007199254740991（2⁵³ − 1）。 | ドキュメントのシード、なければ実行ごとに新規 |
| `--timeout SECONDS` | これより長くかかる実行を失敗にします。1〜300。 | `300` |
| `--fail-fast` | 成功しなかった最初の実行で止めます。残りは開始されません。 | オフ |
| `--junit PATH` | そこに JUnit XML レポートを書き出します（[レポート](#reports)を参照）。 | |
| `--report PATH` | 実行レポートをそこにコピーします。1 回の実行ならファイル、複数ならフォルダーです。 | |
| `--data-dir PATH` | このプロセスの実行のデータフォルダー。レポートはそこに残ります。 | 一時フォルダー（終了時に削除） |
| `--server URL` | このプロセスではなく、このサーバーで実行します（[サーバー上で](#run-on-server)を参照）。 | `SIGNALLAB_SERVER` |
| `--token-file PATH` | サーバーのトークンを保持するファイル。 | `SIGNALLAB_TOKEN_FILE`、なければ `SIGNALLAB_TOKEN` |
| `--secrets files\|system` | このプロセスで、シークレットの値をどこから取得するか。 | `files` |
| `--secrets-dir PATH` | シークレットファイルのフォルダー。名前ごとに 1 ファイル。 | 存在する場合は `/run/secrets/signallab` |

`--data-dir`、`--secrets`、`--secrets-dir` は、このプロセスでの実行に関するものです。`--server` と組み合わせることはできません。

### ファイルとテンプレート {#run-inputs}

`FILE` は、アプリが保存、エクスポートする形の実験です（[[ui:nav.experiment]] 画面の [[ui:exp.exportJson]]）。アプリが開けるどのドキュメントバージョンでも使え、古いものはアプリが開くときと同じように、読み込み時に最新の形式に更新されます。ファイルでない名前は、`.json` の有無にかかわらず、[同梱テンプレート](#cli-templates)から探されます。

```bash
signallab run tests/stage-cues.json tests/api.json
signallab run osc-ping-reply --param device=192.0.2.20:9000
```

すべての実験（とマトリクスのすべての組み合わせ）は、最初の 1 つが実行される前に、エディターと同じ基準でチェックされます。3 つ目のファイルが壊れていれば、何かが送信される前に、最初の 1 つも終了コード `2` で止まります。

### パラメーターとプロファイル {#run-params}

`--param NAME=VALUE` は、この実行だけのパラメーターを設定します。ファイルは変更されません。値は最初の `=` より後のすべてなので、`--param url=http://127.0.0.1/?q=1` も使え、`--param note=` は空の値を設定します。値は、そのパラメーターを持つ、指定したすべての実験に適用され、少なくとも 1 つの実験のパラメーター名でなければなりません。綴りを間違えた名前は、終了コード `2` で拒否されます。

`--profile NAME` は、エディターで [[ui:exp.profile]] から選ぶのと同じように、実験のプロファイルの 1 つで実行します。[データとテンプレート](../experiments/data.md)を参照してください。

### 実行のマトリクス {#matrix}

マトリクスは、同じ実験を、すべての宛先、すべてのユーザー、すべてのペイロードサイズに対して実行します。各組み合わせは、それぞれ独自の結果、独自のレポート、JUnit レポート内の独自のスイートを持つ、独立した実行です。

```bash
signallab run smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest \
  --fail-fast --junit junit.xml
```

これは 4 回の実行です。`device` が最もゆっくり変わり、`user` が最も速く変わります。実行の名前は、ファイル名とその値から付きます: `smoke.json [device=192.0.2.20:9000, user=admin]`。

- `--matrix NAME=V1,V2` は軸を追加します。同じ名前をもう一度指定すると、その軸に値が加わります。名前と値の前後の空白は取り除かれ、2 回指定した値は 1 回だけ実行されます。
- `--matrix-file PATH` は、次の 2 つの形のどちらかの JSON を読みます。

  ```json
  { "device": ["192.0.2.20:9000", "192.0.2.21:9000"], "retries": [1, 3] }
  ```

  これは軸を追加します。すべての組み合わせが実行され、これらの名前は `--matrix` の名前の後に、アルファベット順で並びます。

  ```json
  [
    { "device": "192.0.2.20:9000", "user": "admin" },
    { "device": "192.0.2.21:9000", "user": "guest" }
  ]
  ```

  こちらは組み合わせそのものを列挙し、それぞれが `--matrix` の軸と掛け合わされます。値はテキスト、数値、`true`/`false` で、ファイル内の値の中のカンマは、値の一部のままです。
- マトリクスのすべての名前は、指定した少なくとも 1 つの実験のパラメーターでなければなりません。その名前のいずれかを持たない実験は、無視する値の数だけではなく、1 回だけ実行されます。
- `--param` とマトリクスの両方で設定された名前、または `--matrix` とファイルの両方で設定された名前は、拒否されます。
- 1 つのコマンドからの実行は最大 **256** 回です。それを超えると、何かが実行される前に拒否されます。

### サーバー上での実行 {#run-on-server}

```bash
signallab run tests/stage.json --server http://192.0.2.10:1430 --token-file token.txt
```

実験はこのマシンから渡され、サーバーが自分のネットワーク、自分の[シークレット](../server/index.md#secrets)、自分のデータフォルダーで実行し、ステップを発生するたびに送り返します。レポートはサーバーに残り（そのパスが表示されます）、`--report` はコピーをダウンロードします。サーバー上の実行は、そのサーバーのインターフェイスから開始したものと同じジョブで、サインインしているすべてのページに表示されます。実行の途中で `signallab` が去っても、実行はサーバーで最後まで続き、レポートが保存されます。

トークンは、`--token-file`（または `SIGNALLAB_TOKEN_FILE`）から読まれ、なければ `SIGNALLAB_TOKEN` から読まれて、`Authorization: Bearer` として送られます。サーバーに到達できない、トークンを拒否された、実行の途中で応答が止まった場合は、終了コード `3` になります。`signallab doctor --server URL` は、アドレスとトークンを単独でチェックします。

### レポート {#reports}

各実行は、アプリが書くのと同じレポートを書きます。`--data-dir` がない場合、実行は一時フォルダーを使い、`signallab` の終了時に削除されるので、必要なものは残してください。

- `--report PATH` はレポートをコピーします。1 回の実行なら `PATH` そのものに、複数ならフォルダー `PATH` に、実行された順に `01-<file>.json`、`02-<file>.json`、… として書きます。
- `--data-dir PATH` は、すべての実行のレポートをそのフォルダー（`runs/` の下）に残し、それぞれの場所を表示します。

`--junit PATH` は、すべての CI システムが読める形式である JUnit XML レポートを書き出します。

- 実行ごと（マトリクスでは組み合わせごと）に `<testsuite>` があり、プロパティとして、ファイル、シード、結果、プロファイル、レポートのパス、各マトリクスの値（`param.NAME`）を持ちます。
- 実行されたノードごとに `<testcase>` があり、ノードの種類と ID から名前が付き、それぞれ独自の時間を持ちます。
- 失敗したノードには `<failure>` があり、選択した言語のメッセージ、`type` としてのエラーコード、技術的な詳細を含むノードのステップが入ります。
- 実行が到達しなかったノード（分岐の反対側）ごとに `<skipped>` のケースがあります。
- 開始できなかった実験には `<error>` のケースが 1 つのスイートがあり、`--fail-fast` が開始しなかった実行ごとに `<skipped>` のケースが 1 つのスイートがあります。

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

[負荷](../experiments/load.md)をかけた HTTP ノードは、最後のステップの下に、成立したかどうかにかかわらず、しきい値ごとに 1 行を加えます（`✕ p95 < 100 ms · 152.58 ms`）。成立しなかったしきい値は、実行とその JUnit ケースを失敗にします（`type="load.threshold"`）。

### シークレット {#secrets}

実験はシークレットを `{{secret.NAME}}` として読みます。このプロセスでの実行では、値は次の場所から取られます。

| `--secrets` | `NAME` の値の取得元 |
| --- | --- |
| `files`（既定） | 環境変数 `SIGNALLAB_SECRET_NAME`。なければ、`--secrets-dir`（既定では `/run/secrets/signallab`。そのフォルダーが存在する場合。Docker のシークレットのレイアウトです）内の、`NAME` という名前のファイル。 |
| `system` | Windows の資格情報マネージャー。アプリが [[ui:exp.secrets]] の値を保存する場所です。Linux には `signallab` が読む資格情報ストアがないので、そこでは `--secrets system` は終了コード `3` で失敗します。 |

ファイルの末尾の改行は値に含まれず、空の値は未設定とみなされます。名前は英字、数字、`_` で、数字で始まらず、128 文字までです。値は最大 16 KiB です。

```bash
SIGNALLAB_SECRET_API_TOKEN="$API_TOKEN" signallab run tests/api.json
```

設定されていないシークレットがあると、トラフィックが発生する前に、終了コード `2` と、不足している名前を示して実行が止まります。値が出力されることはなく、ステップ、エラー、レポート、JUnit レポートでは、その代わりに `••••` が表示されます。`--server` を使う場合、シークレットはサーバーのものです。

### `run` の出力 {#run-output}

実行中は、各ステップが stderr に 1 行ずつ出力されます。開始からの時刻、ノード、状態、何をしたかです。アプリのタイムラインと同じです。実行が終わると、stdout の 1 行が結果を伝えます。

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

失敗した実行は、使ったシードを示します。その番号を `--seed` に指定すれば、同じ乱数値でもう一度実行できます。判定の後に、stderr へ、実行の各エミュレーターが何を尋ねられたかが出力されます。リクエスト数、どのルールにも該当しなかった数、失敗した数、ルールごとのヒット数です。

```text
✔ Retry a flaky API passed in 7 ms
  Flaky API: 3 requests, 0 without a rule, 0 failed · #1 3
```

さらに、各劣化リレーが何をしたかが、受信、破棄、スロットリングの数と、各フェーズの転送数／受信数として出力されます。

```text
  127.0.0.1:19110 → 127.0.0.1:19100: 155 datagrams, 38 dropped, 0 throttled · lan 0.0–2.0 s 39/39, wifi 2.0–4.0 s 39/39, offline 4.0–6.0 s 0/38, lan 6.0–8.0 s 38/38
```

TCP のリレーはデータグラムではなくストリームのチャンクを動かし、何も破棄しないので、その行にはチャンク、接続、リセット、ハーフオープンの接続、帯域制限でストリームが保留された回数が数えられます。

```text
  127.0.0.1:19120 → 127.0.0.1:19101: 14 chunks, 2 connections, 1 reset, 0 half-open, 3 held back
```

[負荷](../experiments/load.md)をかけた HTTP ノードは、進行状況を 1 秒に 1 回までのステップとして報告します。

複数の実行がある場合は、実行ごとの 1 行と件数が出力の最後に付きます。`3 runs: 2 passed, 1 failed` のようになります。`--fail-fast` では、開始されなかった実行が stderr に数えられます。

`--json` では、すべての行が `type` を持つオブジェクトです。

```json
{"type":"started","experiment":"Empty experiment","file":"empty","job_id":1,"overridden":false,"profile":null,"seed":1,"started_ms":1791132204585}
{"type":"step","node_id":"start","state":"passed","detail":"Started","message_key":"exp.step.started","message_params":{"seed":1},"job_id":1,"ts":1791132204585}
{"type":"ended","experiment":"Empty experiment","file":"empty","outcome":"passed","seed":1,"params":{},"steps":[…],"report_path":"…","started_ms":1791132204585,"ended_ms":1791132204585,…}
{"type":"summary","total":1,"passed":1,"failed":0,"not_started":0,"exit_code":0}
```

`started`、`ended`、`error` の行は、`file`（指定したとおりの引数）と、マトリクスでは `matrix`（組み合わせの値）を持ちます。`ended` は実行の結果のすべてです。`outcome`（`passed`、`failed`、`stopped`）、`seed`、`profile`、`params`、`error`、すべてのステップ、実行にあれば `emulators` と `impairments`、そして `report_path` です。負荷の最後のステップは、測定したすべての数値を持つ `load` を伴います。`planned`、`sent`、`ok`、`failed`、`missed`、`rps`、`error_rate`、`min_ms`、`mean_ms`、`max_ms`、`p50_ms` から `p99_ms`、`statuses`、秒ごとの値（`seconds`）、`histogram`、各しきい値の判定（`thresholds`）です。

## `validate` {#cli-validate}

```text
signallab validate [OPTIONS] <FILE>...
```

実行前にエディターが行うのと同じ方法で、実験をチェックします。グラフ、すべてのフィールド、テンプレート、パラメーター、シークレットが対象で、何も送信しません。すべてが開始できる場合は `0` で終了します。

[`run`](#cli-run) の入力を受け付けます。ファイルとテンプレート、`--param`、`--profile`、`--matrix`、`--matrix-file`、そして `--server`、`--token-file`、`--secrets`、`--secrets-dir` です。`--server` を使うと、サーバーが自分のシークレットに対してチェックします。

```text
✔ tests/stage.json: Stage cues would run
  Rehearsal: Would not run: …
```

ドキュメントの別のプロファイルだけにある問題は、その下に一覧されますが、チェックを失敗にはしません。`--json` では、実験（組み合わせ）ごとに 1 行になります: `{"experiment", "file", "valid": true, "profile_issues": […]}`、または `"valid": false` と `error`、`exit_code` です。

## `send` {#cli-send}

```text
signallab send <osc|udp|http|ws|mqtt> …
```

アプリの画面が使うのと同じコマンドでメッセージを 1 つ送信するので、ネットワーク上のバイト列は同じです。送信は、ライブラリもシークレットも読みません。

終了コード: `0` は送信成功、`1` は送信失敗、`2` は引数が正しくない場合です。

`<host:port>` は、IP アドレスまたはホスト名とポートです。`127.0.0.1:9000`、`[::1]:9000`（IPv6 アドレスは角括弧で囲みます）、`device.local:9000` のようにします。名前はコマンドの実行時に解決され、IPv4 アドレスがあればそれが使われるので、`localhost:9000` は `127.0.0.1` の受信側に届きます。解決できない名前は送信を失敗にし（`1`）、ポートのない宛先は無効な引数です（`2`）。

### `send osc` {#cli-send-osc}

```text
signallab send osc <host:port> <address> [ARG]...
```

OSC メッセージを 1 つ送信します。引数は、プレフィックスで型を指定するか、推測されます。

| 引数 | OSC の型 |
| --- | --- |
| `i:3` | int32 |
| `f:0.5` | float32 |
| `d:1.5` | float64 (double) |
| `h:64` | int64 |
| `s:text` | 文字列 |
| `b:de ad be ef` | blob（hex のバイト列） |
| `T`, `F` | true、false |
| `N` | nil |
| `3`, `-3` | 単なる整数は int32 |
| `3000000000` | 32 ビットを超える整数は int64 |
| `2.5`、`1e3` | 単なる小数、または指数付きの数値は float32 (float32 の範囲を超えると float64) |
| それ以外 | 文字列 |

```bash
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main 3
# ✔ sent /cue/go (32 bytes) → 127.0.0.1:9000
```

空白を含む引数は引用符で囲んでください: `"s:hello world"`。`s:7` はテキスト `7` を送信します。アドレスは `/` で始まらなければならず、そうでないものは無効な引数です（`2`）。

::: warning Windows の Git Bash
Git Bash は `/` で始まる引数を Windows のパスに書き換えるので、`/cue/go` は `C:/Program Files/Git/cue/go` として渡されます。`//cue/go` と書くか、`MSYS_NO_PATHCONV=1` を付けて実行してください。PowerShell と `cmd` は影響を受けません。
:::

[OSC](../protocols/osc.md) を参照してください。

### `send udp` {#cli-send-udp}

```text
signallab send udp <host:port> (--text TEXT | --hex HEX)
```

UDP データグラムを 1 つ送信します。ペイロードは `--text` か、`--hex` のバイト列で指定します: `"de ad be ef"`、`deadbeef`、`0xDE,0xAD`。

```bash
signallab send udp 127.0.0.1:7000 --text "PLAY 1"
signallab send udp 127.0.0.1:7000 --hex "de ad be ef"
```

### `send http` {#cli-send-http}

```text
signallab send http <METHOD> <URL> [OPTIONS]
```

HTTP リクエストを 1 つ送信します。ステータス行は stderr に出力され（`HTTP 200 OK · 3 ms · 1,234 B`）、レスポンスボディは stdout に出力されるので、そのままパイプで渡せます。256 KiB より長いボディはそこで切り捨てられ、stderr にその旨が表示されます。

| オプション | 動作 | 既定値 |
| --- | --- | --- |
| `-H`, `--header "Name: value"` | リクエストヘッダー。複数指定するには繰り返します。 | |
| `--body TEXT` | リクエストボディ。`@FILE` はテキストファイルの内容を送信します。 | なし |
| `--expect-status STATUS` | レスポンスがこのステータスでない場合、`1` で終了します。 | どのステータスでも `0` |
| `--timeout MS` | レスポンスを待つミリ秒数。 | `10000` |
| `-u`, `--user NAME:PASSWORD` | 資格情報。Basic として送られます。 | |
| `--digest` | `--user` と併用: 代わりに、サーバーの Digest チャレンジに答えます（MD5 または SHA-256）。 | オフ |
| `--bearer TOKEN` | `Authorization: Bearer TOKEN` を送信します。`--user` とは併用できません。 | |

```bash
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab send http POST http://127.0.0.1:8080/cue -H "Content-Type: application/json" --body '{"cue": 1}'
signallab send http GET http://127.0.0.1:8080/admin -u admin:secret --digest
```

`--expect-status` がなければ、`500` を含め、どんなレスポンスも成功です。レスポンスが得られないリクエスト（接続拒否、タイムアウト、解決できない名前）は終了コード `1` になり、答えられなかった Digest チャレンジも同様で、その理由はレスポンスの後に表示されます。

::: tip
引数は、同じマシンのほかのユーザーから見えます。本物のパスワードは、[シークレット](#secrets)として扱える実験のためにとっておいてください。
:::

[HTTP](../protocols/http.md) を参照してください。

### `send ws` {#cli-send-ws}

```text
signallab send ws <URL> [OPTIONS]
```

WebSocket のやり取りを 1 回行います。`ws://…` または `wss://…` に接続し、メッセージを送信し、必要なら応答を待って、閉じます。ハンドシェイクと送信内容は stderr に、応答は stdout に出力されます（バイナリメッセージは hex で）。

| オプション | 動作 | 既定値 |
| --- | --- | --- |
| `--text TEXT` | このテキストメッセージを送信します。 | 何も送信しない |
| `--hex HEX` | これらのバイト列をバイナリメッセージとして送信します。`--text` とは併用できません。 | |
| `-H`, `--header "Name: value"` | アップグレードリクエストのヘッダー。複数指定するには繰り返します。 | |
| `--protocol NAME` | 提示するサブプロトコル。複数指定するには、優先順に繰り返します。 | |
| `--expect TEXT` | このテキストを含むメッセージを待ちます。 | |
| `--expect-regex REGEX` | この正規表現に一致するメッセージを待ちます。 | |
| `--wait` | 任意のメッセージを待ちます。 | |
| `--timeout MS` | 応答を待つミリ秒数。 | `2000` |

```bash
signallab send ws ws://127.0.0.1:9001/echo --text '{"ping": 1}' --expect '"ping"'
signallab send ws ws://127.0.0.1:9001/feed --wait        # 何も送らない: サーバーの最初のメッセージを待つ
```

`--expect`、`--expect-regex`、`--wait` のどれもなければ、接続して送信し、待たずに閉じます。期待した応答が時間内に届かなかった場合、終了コードは `1` です。[WebSocket](../protocols/websocket.md) を参照してください。

### `send mqtt` {#cli-send-mqtt}

```text
signallab send mqtt <host:port> <topic> [payload] [--qos 0|1|2] [--retain]
```

プレーン TCP 上の MQTT 3.1.1 のパブリッシュを 1 回行います。資格情報なしで、新しいクライアント ID を持つ独立したクライアントとして行うので、すでにある接続を切ってしまうことはありません。ポートを省略すると、ブローカーは `1883` にあるものとされます。トピックは 1 つのトピックでなければならず、`+` や `#` を含むものや空のものは、接続する前にコマンドが拒否されます（`2`）。

| オプション | 動作 | 既定値 |
| --- | --- | --- |
| `[payload]` | ペイロード。 | 空 |
| `--qos 0\|1\|2` | サービス品質。 | `0` |
| `--retain` | トピックの保持された値として保持させます。`--retain` と空のペイロードの組み合わせは、保持された値を消去します。 | オフ |

```bash
signallab send mqtt 127.0.0.1:1883 lab/light/1/set on --qos 1
signallab send mqtt 127.0.0.1 lab/light/1/state "" --retain     # 保持された値を消去する
```

[MQTT](../protocols/mqtt.md) を参照してください。

## `fire` {#cli-fire}

```text
signallab fire <signal> [--library PATH]
```

シグナルライブラリのシグナルを、アプリの [[ui:nav.signals]] 画面が送信するのとまったく同じように送信します。OSC、UDP、HTTP、MQTT のシグナルが対象です。シグナルは ID で探され、なければ大文字と小文字を区別しない名前で探されます。複数のシグナルが同じ名前を持つ場合は、その ID を示して拒否されます。

| オプション | 動作 | 既定値 |
| --- | --- | --- |
| `<signal>` | シグナルの ID または名前。 | 必須 |
| `--library PATH` | ライブラリファイル。 | アプリのデータフォルダーの `signals.json` |

```bash
signallab fire "Fader value"
signallab fire go --library show/signals.json
```

ライブラリは読み取られるだけで、作成も変更もされません。[シグナル](../tools/signals.md)を参照してください。

## `emulate` {#cli-emulate}

```text
signallab emulate [OPTIONS] <FILE|NAME>...
```

相手側（HTTP API、OSC・UDP・TCP のデバイス、MQTT ブローカー）を、<kbd>Ctrl</kbd>+<kbd>C</kbd> または `--for` まで演じ、すべてのリクエストを応答するたびに出力します。`FILE` には、エミュレーター 1 つ、そのリスト、またはアプリが書き出す形のライブラリ全体が入ります。`NAME` は、アプリのライブラリ（[[ui:nav.emulators]] 画面のもの）にあるエミュレーターの ID または名前です。

| オプション | 動作 | 既定値 |
| --- | --- | --- |
| `-p`, `--param NAME=VALUE` | テンプレートが `{{NAME}}` として読む値。複数指定するには繰り返します。 | |
| `--bind IP:PORT` | 代わりにそこで待ち受けます。エミュレーターが 1 つのときだけ使えます。 | エミュレーター自身の設定 |
| `--for SECONDS` | これだけの時間の後に止めます。 | <kbd>Ctrl</kbd>+<kbd>C</kbd> まで |
| `--seed N` | ランダムな選択（ランダムな順序、ジッター、ジェネレーター）のシード。 | |
| `--check` | ポートを開かずに、エミュレーターをチェックして終了します。 | オフ |
| `--library PATH` | 名前を探すライブラリ。 | アプリのデータフォルダーの `emulators.json` |
| `--server URL`、`--token-file PATH` | サーバー上で開始し、その API を通して追跡し、最後に停止します。 | |
| `--secrets`、`--secrets-dir` | [`run`](#cli-run) と同じです。 | |

```text
$ signallab emulate tests/orders-api.json --for 60
Orders API (http) answering on 127.0.0.1:18099
answering for 60 s
+   1.209 s Orders API  #1  GET /orders/42 → 200 OK · 12 B  1 ms  ← 127.0.0.1:55744
+   1.209 s Orders API  —  GET /nothing → 404 Not Found · 20 B  2 ms  ← 127.0.0.1:55745
Orders API: 2 requests, 1 without a rule, 0 failed · #1 1
```

各行は、開始からの時刻、エミュレーター、応答したルール（`#1`、該当なしなら `—`）、リクエストとその応答、かかった時間、送信元です。最後に、各エミュレーターの件数が表示されます。リクエスト数、どのルールにも該当しなかった数、失敗した数、ダウンに当たった数や配信されなかった数（あった場合）、ルールごとのヒット数です。

終了コード: <kbd>Ctrl</kbd>+<kbd>C</kbd> または `--for` で止まったときは `0`、エミュレーターが無効なときは `2`、ポートが使用中か開けない、または応答中にソケットが失敗したときは `3` です。

パイプラインでは、バックグラウンドで起動し、それに対してシステムをテストし、最後に件数を読みます。

```bash
signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
npm test          # テスト対象のシステム。エミュレーターのアドレスに向けて設定済み
wait              # mock.ndjson の最後の行が件数
```

サーバーでは、`--server` で起動したエミュレーターは、`signallab` が正常に終了すると停止します。プロセスが強制終了されて残ったものは、サーバーのインターフェイスから停止できます。[エミュレーター](../tools/emulators.md)を参照してください。

## `emulators` {#cli-emulators}

```text
signallab emulators [--library PATH]
```

ライブラリのエミュレーターを、ID、名前、プロトコル、アドレス、ルールの数で一覧表示します。`--library PATH` は、アプリのデータフォルダーの `emulators.json` の代わりに、別のライブラリファイルを読みます。`signallab emulate <id>` で 1 つを開始できます。

ライブラリは読み取られるだけです。そのようなファイルがない場合（データフォルダーのものは、アプリが初めて起動したときに作られます）、コマンドはその旨を伝えて `2` で終了します。

## `templates` {#cli-templates}

```text
signallab templates
```

同梱のテンプレートを一覧表示します。`run` と `validate` は、これらを名前で受け付けます。アプリ自身のものです。

| 名前 | アプリでの名前 | パラメーター |
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

すべてのテンプレートは、ループバックを指しています。`flaky-api`、`fault-phases`、`dependency-outage` は独自のエミュレーターを持っているので、ほかに何も待ち受けていなくても実行でき、`signallab` が動くのを手早く確かめられます。

## `nodes` {#cli-nodes}

```text
signallab nodes
```

実験が何でできているかを JSON で出力します。ドキュメントの形とルール、ラベル・説明・フィールド・出力・エンジンが受け付ける例を持つ、すべての種類のノード、`{{template}}` 言語、負荷プロファイル、エミュレーターのドキュメントです。アシスタントが実験を書くために、[`signallab mcp`](mcp.md) を通して読むものです。人向けには、[ノード](../experiments/nodes.md)が同じことをより多くの言葉で説明しています。

## `mcp` {#cli-mcp}

```text
signallab mcp [OPTIONS]
```

Model Context Protocol で、stdin と stdout を通して、Signal Lab を AI アシスタントに提供します。Claude Code、Claude Desktop、Cursor、VS Code での設定、オプション、ツールを含むすべてが、[アシスタント (MCP)](mcp.md) にあります。

## `doctor` {#cli-doctor}

```text
signallab doctor [--server URL] [--token-file PATH]
```

Signal Lab と機器の間に何があるかを教え、何かあれば `1` で終了します。出力は、コマンドラインのほかの部分と同じように [`--lang`](#language) に従います。

- このマシンが接続しているネットワーク: その名前とアドレス。
- データフォルダー: 書き込めるかどうか（まだ作られていないものは問題ありません。アプリが初回の使用時に作ります）。
- ファイアウォール: Windows では、`signallab` とデスクトップアプリのそれぞれについて、いまマシンが接続しているネットワークの種類（プライベート、ドメイン、パブリック）で、ほかのマシンからの接続を許可するルールがあるか、それらをブロックするルールがあるか（システムのプロンプトで*キャンセル*を選んだときに残るもの）。Linux では、ufw または firewalld が有効かどうかと、ポートを開くコマンド。
- `--server`（または `SIGNALLAB_SERVER`）を指定した場合: サーバーが応答し、トークンを受け付けるかどうか。

```text
signallab [[version]]
Network: LAB-PC · 192.0.2.15
Data folder: C:\Users\lab\Documents\SignalLab — writable
Firewall · signallab (C:\…\Signal Lab\signallab.exe) · private network: not allowed yet — signallab firewall allow
Firewall · app (C:\…\Signal Lab\signal-lab.exe) · private network: allowed
✖ 1 thing in the way
```

`--json` は、同じ内容を 1 つのオブジェクトとして出力します。[トラブルシューティング](../reference/troubleshooting.md)を参照してください。

## `firewall` {#cli-firewall}

```text
signallab firewall allow [--public]
```

Windows で、ほかのマシンが Signal Lab に到達できるようにします。モニターや待機ノードがデバイスの通信を受け取るために必要なことです。Windows はまず管理者権限を求め、その後、`signallab` とデスクトップアプリ（その隣、またはインストーラーが置いた場所にあるもの）の受信規則が、ブロック規則も含めて、プライベートとドメインのネットワーク向けに、それぞれ 1 つの許可規則に置き換えられます。

| オプション | 動作 |
| --- | --- |
| `--public` | パブリックネットワークでも許可します。会場の Wi-Fi はたいていこれに当たります。 |

終了コード: 完了したら `0`、管理者権限のプロンプトが拒否された、または変更が失敗したら `3` です。Linux では何も変更されません。待ち受けるポートを開く ufw または firewalld のコマンドを表示して、`0` で終了します。

ファイアウォールが変更されるのは、このコマンドを実行したときだけです。`signallab` のほかの部分がファイアウォールに触れることはありません。

## `version` {#cli-version}

```text
signallab version
```

`signallab [[version]]` を表示します。`--json` では `{"version": "[[version]]"}` です。`signallab --version` も同じバージョンを表示します。

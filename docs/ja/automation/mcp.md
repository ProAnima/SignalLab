---
title: アシスタント (MCP)
description: signallab mcp を使うと、AI アシスタントが Model Context Protocol を通じて、実験の作成・チェック・実行、メッセージの送信、待ち受け、エミュレーターの動作を行えます。
---

# アシスタント向けの Signal Lab: `signallab mcp`

`signallab mcp` は [Model Context Protocol](https://modelcontextprotocol.io) のサーバーです。Claude Code、Claude Desktop、Cursor、VS Code、その他の MCP クライアントのアシスタントがこれを起動すると、次のことができるようになります。

- 実験が何でできているかを把握し、実験を書き、チェックし、実行して、失敗した理由をステップごとに読み取る。
- OSC メッセージ、データグラム、HTTP リクエスト、WebSocket メッセージ、MQTT パブリッシュを 1 件送信し、デバイスが送ってくるものをポートで待ち受ける。
- ライブラリのシグナルを送信する。
- 依存先 (HTTP API、OSC・UDP・TCP デバイス、MQTT ブローカー) を演じ、システムがそれに送ったものを読み取る。
- 以前の実行を読み返し、そのうちの 2 つを比較する。

すべての操作は、アプリが使うのと同じエンジンのコマンドを通じて行われます。そのため、アシスタントが実行する実験は、アプリが行う実行とまったく同じで、レポートも同じです。失敗は、インターフェイスと同じ言葉で表現されます。

::: warning
送信、実行、エミュレーターは、実際の通信をネットワークに流します。どの機器と通信してよいかをアシスタントに伝えてください。同梱のテンプレートの宛先はループバック (`127.0.0.1`) です。
:::

## セットアップ {#setup}

`signallab` はデスクトップアプリに付属しており、インストールすると `PATH` に追加されます ([インストール](cli.md#install)を参照)。クライアントが `signallab mcp` を自分で起動し、標準入出力でやり取りします。手動で実行する必要はありません。

### 設定を出力する {#print-config}

`--print-config` は、クライアントに必要な内容を、この `signallab` のフルパスとともに出力します。

| コマンド | 出力される内容 |
| --- | --- |
| `signallab mcp --print-config claude-code` | `claude mcp add` のコマンドライン。 |
| `signallab mcp --print-config claude-desktop` | Claude Desktop の設定ファイルに書く `mcpServers` のエントリ。 |
| `signallab mcp --print-config cursor` | Cursor の `mcp.json` に書く、同じ `mcpServers` のエントリ。 |
| `signallab mcp --print-config vscode` | VS Code の `.vscode/mcp.json` に書く `servers` のエントリ。 |

[ラボサーバー](#on-a-server)で動作するアシスタントには `--server URL` を追加します。出力される設定にそれが含まれ、トークンの部分はプレースホルダーになります。

### Claude Code {#claude-code}

`--print-config claude-code` が出力する行を実行します。たとえば次のようになります。

```bash
claude mcp add signallab -- "C:\Program Files\Signal Lab\signallab.exe" mcp
```

### Claude Desktop と Cursor {#claude-desktop}

エントリをクライアントの設定 (Claude Desktop なら `claude_desktop_config.json`、Cursor なら `mcp.json`) に書き込み、クライアントを再起動します。

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

### その他のクライアント {#other-clients}

stdio サーバーを起動できるクライアントなら、どれでも同じ方法で使えます。コマンドは `signallab` (またはそのフルパス)、引数は `mcp` と[オプション](#options)のいずれかです。Linux では、サーバーのイメージもコマンドとして使えます。

```bash
docker run -i --rm --network host --entrypoint signallab ghcr.io/proanima/signallab:[[version]] mcp
```

## オプション {#options}

| オプション | 機能 | 既定値 |
| --- | --- | --- |
| `--server URL` | 実験、送信、エミュレーターを、この Signal Lab サーバー上で実行します ([ラボサーバーで使う](#on-a-server)を参照)。 | `SIGNALLAB_SERVER` |
| `--token-file PATH` | サーバーのトークンが入っているファイル。 | `SIGNALLAB_TOKEN_FILE`、なければ `SIGNALLAB_TOKEN` |
| `--data-dir PATH` | 実行とそのレポートの保存先。`--server` とは併用できません。 | アプリのデータフォルダー (`Documents/SignalLab`) |
| `--library PATH` | `list_signals` と `fire_signal` が使うシグナルライブラリ。 | アプリの `signals.json` |
| `--emulators PATH` | `list_emulators` と `start_emulator` が使うエミュレーターライブラリ。 | アプリの `emulators.json` |
| `--secrets files\|system` | このマシン上の実行でシークレットの値をどこから取得するか。[`run`](cli.md#secrets) と同じです。`--server` とは併用できません。 | `files` |
| `--secrets-dir PATH` | シークレットのファイルを入れたフォルダー。名前ごとに 1 ファイル。`--server` とは併用できません。 | `/run/secrets/signallab` (存在する場合) |
| `--lang <code>` | 結果と失敗の言語。 | `SIGNALLAB_LANG`、なければロケール、それもなければ `en` |
| `--print-config CLIENT` | クライアントの設定を出力して終了します: `claude-code`、`claude-desktop`、`cursor`、`vscode`。 | |

実行のレポートは、アプリ自身のレポートと同じアプリのデータフォルダーに保存されるので、セッションが終わった後も残ります。

## ツール {#tools}

読み取りしか行わないツールは読み取り専用としてマークされているので、クライアントは確認なしで実行させることができます。外の世界に影響を与えるツール (送信する、待ち受ける、何かを起動する) はそのようにマークされ、クライアントは呼び出しのたびに確認を求めることができます。破壊的としてマークされたツールはありません。

| ツール | 機能 | 外の世界に影響するか |
| --- | --- | --- |
| `describe_nodes` | 実験のドキュメント、フィールド・出力・例を含むすべての種類のノード、`{{template}}` 言語、負荷プロファイル、エミュレーターのドキュメント。 | いいえ |
| `list_templates` | 同梱の実験とそのパラメーター。 | いいえ |
| `get_template` | 同梱の実験 1 件をドキュメントとして。 | いいえ |
| `validate_experiment` | 実行前にエディターが行うのと同じように実験をチェックします。何も送信しません。 | いいえ |
| `run_experiment` | 実験を最後まで実行し、すべてのステップを報告します。 | はい |
| `send_osc` | OSC メッセージ 1 件。 | はい |
| `send_udp` | UDP データグラム 1 件。 | はい |
| `send_http` | HTTP リクエスト 1 件。 | はい |
| `send_mqtt` | MQTT 3.1.1 のパブリッシュ 1 件。 | はい |
| `send_ws` | WebSocket のやり取り 1 回。 | はい |
| `listen` | しばらくの間 UDP ポートに届いたもの。 | はい |
| `list_signals` | ライブラリのシグナル。 | いいえ |
| `fire_signal` | ライブラリのシグナルを送信します。 | はい |
| `list_emulators` | ライブラリのエミュレーター。 | いいえ |
| `start_emulator` | エミュレーターを起動します。 | はい |
| `emulator_exchanges` | 実行中のエミュレーターが受信して応答した内容。 | いいえ |
| `set_emulator_down` | 実行中のエミュレーターをダウンさせるか、復帰させます。 | はい |
| `list_runs` | 以前の実行のレポート。 | いいえ |
| `compare_runs` | 2 つの実行を並べて表示します。 | いいえ |
| `list_jobs` | 実行中のもの。 | いいえ |
| `stop_job` | 実行中のジョブを停止します。 | はい |

### 実験 {#tools-experiments}

`describe_nodes` は、アシスタントが実験を書く前に読むものです。[`signallab nodes`](cli.md#cli-nodes) と同じ内容です。`list_templates` と `get_template` は、実行や調整に使える動作する例を提供します。

`validate_experiment` と `run_experiment` は、実験を次の 3 つのうち**ちょうど 1 つ**の方法で受け取ります。

| 引数 | 内容 |
| --- | --- |
| `document` | アプリが保存する形式の実験ドキュメント。 |
| `file` | `signallab` が動作しているマシン上の実験ファイルのパス。 |
| `template` | 同梱テンプレートの名前。 |
| `params` | この実行のパラメーターの値: `{"name": "value"}`。数値と真偽値はテキストとして扱われます。 |
| `profile` | ドキュメントのこのプロファイルで実行します。既定値を使うには `""`。 |
| `seed` | `run_experiment`: ランダムな値のシード。 |
| `timeout` | `run_experiment`: 実行にかけてよい秒数。1 から 300 (既定値は 300)。 |

`run_experiment` は、実行が終わったときに応答します。成功、失敗、停止のいずれか、所要時間とシード、すべてのステップについて何を行ったか・なぜ失敗したか、各エミュレーターに何が問い合わされたか、各劣化リレーが何をしたか、そしてレポートのパスです。実行が失敗しても、それは通常の応答であり (理由はステップが示します)、呼び出しの失敗ではありません。

### 単発のメッセージ {#tools-send}

| ツール | 引数 |
| --- | --- |
| `send_osc` | `target` (`host:port`)、`address`、`args`: 数値 (整数は int32、その範囲を超える場合は int64、それ以外は float32)、文字列、真偽値、`null`、または `{"type": "int"\|"float"\|"str"\|"long"\|"double"\|"bool"\|"blob"\|"nil", "value": …}`。 |
| `send_udp` | `target`、および `text` または `hex` (`"de ad be ef"`)。 |
| `send_http` | `method`、`url`、`headers` (`{"Name": "value"}`)、`body`、`timeout_ms` (既定値 10000)、`auth`: `{"scheme": "basic"\|"digest", "username", "password"}` または `{"scheme": "bearer", "token"}`。ステータス、時間、ヘッダー、ボディ (先頭の 16 KiB) を返します。 |
| `send_mqtt` | `broker` (`host:port`、指定がなければポート 1883)、`topic`、`payload`、`qos` (0、1、2)、`retain`。`retain` 付きの空のペイロードは、保持された値を消去します。 |
| `send_ws` | `url` (`ws://` または `wss://`)、`text` または `hex`、`headers`、`protocols`。応答を待つには `expect` (含む)、`expect_regex`、`wait` (任意のメッセージ) を使います。`timeout_ms` は 1 から 120000 (既定値 2000)。ハンドシェイク、送信した内容、応答を返し、応答が JSON の場合は解析した形で返します。 |

これらは、アプリの画面が使うのと同じコマンドです。[`signallab send`](cli.md#cli-send) を参照してください。

### 待ち受け {#tools-listen}

`listen` は、**`signallab mcp` が動作しているマシン上で** UDP ポートをしばらくの間開き、届いたものを返します。OSC メッセージはデコードされ、それ以外のデータグラムはテキストと hex で返されます。

| 引数 | 内容 | 既定値 |
| --- | --- | --- |
| `bind` | `IP:port`。例: `0.0.0.0:9000`。 | 必須 |
| `protocol` | `osc` または `udp`。 | `osc` |
| `seconds` | 待ち受ける秒数。0.1 から 60。 | 5 |
| `max` | このデータグラム数に達したら停止します。1 から 1000。 | 100 |

`0.0.0.0` で何も届かなかった場合、応答はアシスタントにファイアウォールの確認を促します ([`signallab doctor`](cli.md#cli-doctor))。`--server` を指定している場合、`listen` は拒否されます。サーバーでは、待機ノードを含む実験がそこで待ち受けます。

### シグナルとエミュレーター {#tools-library}

`list_signals` と `fire_signal` は、シグナルライブラリを使います。アプリの `signals.json`、`--library`、または呼び出しで渡された `library` のパスです。シグナルは、アプリが送信するのとまったく同じように、ID または名前で送信されます。

`list_emulators` は、ライブラリのエミュレーターの名前を返します。`start_emulator` はエミュレーターを 1 つ起動し (`emulator` にドキュメントを渡すか、`name` にライブラリのエントリの ID または名前を渡します)、そのジョブ ID とアドレスを返します。`stop_job` まで、ルールに従って応答します。`bind` で別の `IP:port` に移し、`params` でテンプレートが読み取る値を渡し、`seed` でランダムな選択を固定します。`emulator_exchanges` (`job_id`、新しいものだけを取得するには `after`) は、届いたものと各ルールが応答した内容を一覧表示します。`set_emulator_down` (`job_id`、`down`、`fault`: `unavailable`、`reset`、`timeout`) は、実行中のエミュレーターを、再び復帰させるまで停止状態にします。HTTP は障害を受けます (`unavailable` は 503 を返します)。TCP デバイスと MQTT ブローカーは接続を切断し、OSC と UDP は何も応答しません。[エミュレーター](../tools/emulators.md)を参照してください。

### 実行とジョブ {#tools-runs}

`list_runs` は以前の実行のレポートを新しいものから順に読み取ります。`experiment` で実験を指定するとその実験のものだけに絞られ、最大 `limit` 件 (1 から 500、既定値 50) で、各負荷ステップの数値が含まれます。`compare_runs` は、実行の名前を 2 つ、`a` (前) と `b` (後) で受け取り、各負荷ステップのレイテンシ、エラー率、達成したレート、取りこぼしを並べて表示します。悪い方向への 5% 以上の変化は悪化としてマークされます。[実行とレポート](../experiments/runs.md)を参照してください。

`list_jobs` は実行中のもの (モニター、ジェネレーター、エミュレーター、実行) を一覧表示し、`stop_job` は ID を指定して 1 つを停止します。

## 結果とエラー {#results}

すべての応答は、モデル向けのテキストと、それと同じ内容の構造化データです。失敗はエラーとしてマークされ、エンジンのエラー (安定した `code`、その値、対象のノードとフィールド) が、`--lang` で選んだ言語で表現されて付きます。アシスタントが誤って渡した引数は、修正できる言葉で返されます。

## 進捗とキャンセル {#progress}

クライアントが `run_experiment` の進捗を要求すると、各ステップがその都度 (ノードとその状態が) 報告されるので、アシスタントも、あなたも、実行が進む様子を見られます。呼び出しをキャンセルするとその呼び出しが停止します。`run_experiment` をキャンセルすると、アプリの [[ui:common.stop]] と同じように、実行そのものが停止します。

クライアントが接続を閉じると、進行中の呼び出しは完了し、その後 `signallab mcp` は終了します。

## ラボサーバーで使う {#on-a-server}

`--server http://192.0.2.10:1430` を指定すると、実験、送信、シグナル、エミュレーターは、**そのサーバー上で**、その API を通じて行われます。サーバーのネットワーク、シークレット、データフォルダーが使われるので、アシスタントはラボからしか到達できない機器にも届きます。トークンはクライアントの環境で渡します。

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

このマシンに残るもの: シグナルライブラリとエミュレーターライブラリ (アプリのもの、または `--library` と `--emulators`)、および呼び出しが指定するファイル (`file`、`library`) は、ここで読み取られ、その内容がサーバーに送られます。`listen` は拒否されます。[サーバーとして Signal Lab を実行する](../server/index.md)を参照してください。

## 安全性 {#safety}

- アシスタントができるのは、ツールが行うことだけです。そして、どのツールもアプリ自身のコマンドの 1 つなので、アプリが到達できないものにアシスタントが到達することはありません。
- 送信、待ち受け、何かの起動を行うツールは、外の世界に影響するものとしてマークされます。呼び出しのたびに確認するかどうかは、クライアントが決めます。
- シークレットの値がアシスタントに届くことはありません。実験ではそれを `{{secret.NAME}}` と名前で指定し、すべての結果ではその代わりに `••••` が表示されます。
- エミュレーターやリスナーは、動作しているマシンのポートを開きます。`list_jobs` と `stop_job` で、まだ実行中のものを確認して終了できます。

## プロトコル {#protocol}

クライアントの開発者向け: stdio 上の JSON-RPC 2.0 で、1 行に 1 メッセージです。stdout にはプロトコルのメッセージだけが出力され、人間向けのものはすべて stderr に出力されます。プロトコルのバージョンは `2025-06-18`、`2025-03-26`、`2024-11-05` (クライアントが別のバージョンを要求した場合は最新のもの)、バッチ、`ping`、`tools/list`、`tools/call` に対応します。進捗は `progressToken` を送った呼び出しに対する `notifications/progress` として、キャンセルは `notifications/cancelled` で行われます。サーバーの `instructions` は、ツール同士がどう組み合わさるかをモデルに伝えます。

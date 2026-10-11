---
title: HTTP API
description: インターフェイス自身が使うのと同じコマンドとイベントで、スクリプト、CI、ほかのツールから Signal Lab サーバーを操作します。
---

# HTTP API

スクリプト、CI パイプライン、ほかのツールから Signal Lab を操作するには、Signal Lab サーバー（`signal-lab-server`、または Docker イメージ）と HTTP で通信します。サーバーがブラウザーに表示するインターフェイスは、まさにこの API を使っています。すべてのボタンは `/api/invoke/<command>` の呼び出しで、すべてのライブの数値は `/api/events` に届きます。つまり、人がサーバーのページでできることは、スクリプトでもできます。

デスクトップアプリには HTTP API がありません。そのウィンドウは、アプリの中にあるエンジンに直接つながっています。デスクトップで自動化するには、同じマシンでサーバーを動かす（[サーバー](../server/index.md)を参照）か、コマンドライン [`signallab`](../automation/cli.md) を使ってください。

## エンドポイント {#endpoints}

| メソッドとパス | 動作 | トークン |
| --- | --- | --- |
| `GET /api/health` | サーバーが応答するか、そのバージョン、トークンを求めるか | 不要 |
| `POST /api/invoke/<command>` | エンジンのコマンドを 1 つ実行します。JSON の引数を受け取り、JSON の結果を返します（[コマンド](commands.md)） | 必要 |
| `POST /api/run` | 実験を最後まで実行します。結果を返すか、そのステップを行として返します（[実行](run.md)） | 必要 |
| `GET /api/events` | すべてのエンジンイベントの WebSocket（[イベント](events.md)） | 必要 |
| `GET /api/files?path=…` | エンジンがデータフォルダーに書き込んだファイルを、ダウンロードとして返します | 必要 |
| `GET /api/openapi.json` | この API を OpenAPI 3.1 で記述したもの | 必要 |
| `GET /login`、`POST /login` | ブラウザー向けのサインインのページとフォーム | 不要 |
| `POST /logout` | ブラウザーのセッションを終了します | セッション |

`/api/` の下のそれ以外のパスは、コード `api.not_found` で `404` を返します。パスが受け付けないメソッド（`GET /api/invoke/…`）は、本文が空の `405` です。それ以外はすべてインターフェイスで、トークンのあるサーバーでは、セッションのないブラウザーは先に `/login` へ送られます。

## ベース URL {#base-url}

サーバーは、`--listen`（または `SIGNALLAB_LISTEN`）で別の指定をしない限り、`http://127.0.0.1:1430` で待ち受けます。Docker イメージは、すべてのネットワークカードの `0.0.0.0:1430` で待ち受けます。これらのページの例では、次を使います。

```bash
SERVER=http://127.0.0.1:1430
```

サーバーはプレーンな HTTP を話します。HTTPS にするには、TLS を終端するリバースプロキシをその前に置き、サーバーを `--secure-cookie` 付きで起動してください。

## 認証 {#authentication}

トークンなしで起動したサーバーは、ループバックでしか待ち受けず、認証は不要です。そのマシン上の誰でも使えます。ほかの人が到達できるサーバーには、常にトークンがあり、その場合は `/api/health` と `/login` 以外のすべてのリクエストがトークンを持っていなければなりません。

**スクリプト**は、`Authorization` ヘッダーでトークンを送ります。

```bash
TOKEN=$(cat token.txt)
curl -fsS "$SERVER/api/invoke/jobs_list" -X POST \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json"
```

ヘッダーは、`Bearer`、空白 1 つ、トークンの形でなければなりません。トークンがない、または誤っている場合は、コード `auth.required` の `401` です。

**ブラウザー**は、`/login` でトークンを使って 1 回サインインし、セッション Cookie `signallab_session` を受け取ります。`HttpOnly`、`SameSite=Strict` で、7 日間保持され、サーバーが `--secure-cookie` で動いているときは `Secure` も付きます。`POST /logout` で終了します。サインインフォームで誤ったトークンを入力すると、応答までに 1 秒かかるので、推測は遅くなります。セッションはサーバーのメモリにあります。再起動するとすべてのブラウザーがサインアウトされますが、トークンを使うスクリプトは影響を受けません。サーバーが保持するセッションは最大 1024 個で、それを超えると最も古いものが破棄されます。

トークンの出どころは、サーバーのセットアップです（`--token-file`、`SIGNALLAB_TOKEN`、または `--generate-token` がデータフォルダーに作る `token` ファイル）。[サーバーのセキュリティ](../server/security.md)を参照してください。トークンは 24 文字以上で、空白を含みません。`signal-lab-server token` は新しいものを表示します。

::: warning
トークンを持っている人は、サーバーに通信を送信させることができます。トークンを入れたファイルは自分だけが読めるようにし、URL には決して入れないでください。サーバーは、そもそも URL にあるトークンを読みません。
:::

## Host と Origin {#host-origin}

ほかの何よりも先に、2 つのチェックが、すべてのパス（`/api/health` を含みます）で実行されます。

**`Host`。** `Host` ヘッダーは、このサーバーを指す名前でなければなりません。

- ループバックの名前は常に通ります: `localhost`、`.localhost` で終わる名前、`127.x.x.x`、`[::1]`。
- `--allowed-host`（または `SIGNALLAB_ALLOWED_HOSTS`）で指定した名前は通ります。
- トークンがあって `--allowed-host` がないサーバーは、どの名前にも応答します。

それ以外は `auth.host` の `403` です。サーバーが受け付ける名前でアクセスしてください。`curl` は、指定した URL のホストを送ります。

**`Origin`。** 何かを変更するリクエスト（`GET` と `HEAD` 以外のすべてのメソッド）と WebSocket のアップグレードは、`Origin` ヘッダーを持つ場合、サーバー自身のページからのものでなければなりません。そのホストとポートが `Host` と等しい必要があります。そうでなければ、`auth.origin` の `403` が返ります。`Origin: null` も拒否されます。スクリプトや `curl` は `Origin` を送らないので、このチェックは通ります。それでもトークンは必要です。

**JSON のみ。** `POST /api/invoke/…` と `POST /api/run` は、`Content-Type: application/json` を受け付けます（`; charset=utf-8` のようなパラメーターは問題ありません）。それ以外は、`command.json_required` の `415` です。別のサイトのウェブページは、サーバーに先に尋ねることなくこれを送れず、サーバーはそれに同意しません。

## コマンドの呼び出し {#invoke}

```http
POST /api/invoke/<command>
Content-Type: application/json

{ "argument": "value", … }
```

- 本文は、コマンドの引数を持つ JSON オブジェクト 1 つです。引数のないコマンドは、`{}` か空の本文を受け付けます（`Content-Type` ヘッダーは引き続き必要です）。
- 引数名は、インターフェイスが送るのと同じ camelCase です: `jobId`、`nodeId`。引数として渡すオブジェクト（`config`、`request`、`document`、`library` など）は、エンジンが書くフィールド名をそのまま保ち、そのほとんどは snake_case です: `timeout_ms`、`port_start`。
- コマンドが知らない引数は、無視されずにエラーになります。コマンド名を示す `command.args_invalid` の `422` で、パーサーの文言が `detail` に入ります。必須の引数が欠けている場合も同じです。引数のないコマンドは、本文をまったく読みません。
- 任意の引数は、省略するか `null` として送れます。
- 応答は、コマンドの結果を JSON にした `200` です。返すものがないコマンドは `null` を返します。

すべてのコマンドの引数と結果は、[コマンド](commands.md)にあります。

## エラー {#errors}

| ステータス | 状況 | 本文 |
| --- | --- | --- |
| `200` | コマンドが実行された。`/api/run` が実行を開始した | 結果 |
| `400` | 本文が JSON でない。実行リクエストを読み取れない | `EngineError`: `command.args_invalid`、`api.run_invalid`、`api.run_source` |
| `400` | `path` のない `/api/files` | ウェブサーバーからのプレーンテキスト。`EngineError` ではありません |
| `401` | トークンがない、または誤っている | `EngineError`: `auth.required` |
| `403` | サーバーが拒否する `Host` または `Origin` | `EngineError`: `auth.host`、`auth.origin` |
| `404` | そのような API パスがない。データフォルダーにないファイル | `EngineError`: `api.not_found`、`file.not_found` |
| `405` | パスが受け付けないメソッド | 空 |
| `413` | 24 MiB を超えるリクエスト本文 | ウェブサーバーからのプレーンテキスト。`EngineError` ではありません |
| `413` | 256 MiB を超えるダウンロード | `EngineError`: `file.too_large` |
| `415` | `application/json` でない | `EngineError`: `command.json_required` |
| `422` | コマンドが失敗した、または実行を開始できなかった | `EngineError`: エンジンの任意のコード |
| `500` | ファイルを読み取れなかった | `EngineError`: `file.io` |

不明なコマンド名は、`command.unknown` の `422` です。

エンジンが報告するすべての失敗は、`EngineError` という 1 つの形をしています。

```json
{
  "code": "transport.refused",
  "params": { "target": "http://127.0.0.1:8080/" },
  "node": "request",
  "field": { "key": "url" },
  "detail": "error sending request for url (http://127.0.0.1:8080/): tcp connect error: Connection refused (os error 111)"
}
```

| フィールド | 内容 |
| --- | --- |
| `code` | 何が起きたか。安定した識別子です。すべてのコードとそのメッセージは[エラーメッセージ](../reference/errors.md)に、ドットより前の部分でグループ化して一覧されています（たとえば [`transport`](../reference/errors.md#transport)） |
| `params` | メッセージが指す値。すべて文字列です。ない場合は省略されます |
| `node` | 対象となる実験のノード。ない場合は省略されます |
| `field` | 対象となるフィールド: `key`（[フィールド](../reference/errors.md#fields)にある名前）と、ヘッダーのような繰り返しのあるフィールドでは、1 から始まる `index`。ない場合は省略されます |
| `detail` | オペレーティングシステム、パーサー、ライブラリ自身の文言（英語）。ない場合は省略されます |

分岐には `code` を使い、`detail` は使わないでください。実行や単発の送信が使うシークレットの値は、それが報告するすべてのエラーの中で、マスクされます（`••••`）。

サーバーには届いたがエラーステータスが返った、あるいは何も返らなかったリクエストは、コマンドの失敗ではありません。`http_request` はレスポンスを添えて `200` を返し、`ok`、`error`、`cause` が何が起きたかを示します。[`http_request`](commands.md#http_request)を参照してください。

## 制限 {#limits}

| 項目 | 制限 | 制限に達したとき |
| --- | --- | --- |
| リクエスト本文 | 24 MiB | `413` |
| 実験のドキュメント | 4 MiB | `file.too_large` |
| `/api/files` からのダウンロード | 256 MiB | `413`、`file.too_large` |
| 実行の長さ | 300 秒 | 実行は `run.timeout` で失敗します |
| フィードバックフォーム（`feedback_send`） | 合計 15 MiB | `feedback.too_large` |
| 1 つの WebSocket が待つイベント | 4096 | 取りこぼした数とともに [`server://lagged`](events.md#event-server-lagged) を受け取ります |

## イベント {#events}

WebSocket にアップグレードした `GET /api/events` は、エンジンが送るすべてのイベント（実行のステップ、ジョブの終了、モニターのメッセージ、インスペクターのフレーム）を、接続しているすべてのクライアントへ、テキストメッセージとしてストリームします。

```json
{ "event": "job://ended", "payload": { "job_id": 7, "kind": "storm", "error": null } }
```

トークンと `Origin` のルールは、ほかと同じです。すべてのチャンネルとそのペイロードは[イベント](events.md)にあります。

## ファイル {#files}

`GET /api/files?path=<path>` は、エンジンがサーバーのデータフォルダーに書き込んだファイルをダウンロードします。実行レポート（実行の結果の `report_path`）、実験やインスペクターのエクスポート、シグナルやエミュレーターのライブラリです。`path` は、エンジンがあなたに渡した、サーバー上のパスです（URL エンコードします）。

```bash
curl -fsS -G "$SERVER/api/files" --data-urlencode "path=/data/runs/run-1759600000000-3.json" \
  -H "Authorization: Bearer $TOKEN" -o report.json
```

- 提供されるのは、データフォルダーの中のファイルだけです。それ以外、フォルダー、存在しないファイルは、`file.not_found` の `404` です。
- 応答は `application/octet-stream` で、`Content-Disposition: attachment` が付きます。ファイル名の、英字、数字、`.`、`_`、`-` 以外の文字は `_` になります。
- 256 MiB を超えるファイルは、`file.too_large` の `413` です。

データフォルダーに何が入っているかは、[ファイルとフォルダー](../reference/files.md)にあります。

## ヘルス {#health}

`GET /api/health` は公開されています。トークンは不要で、必要なのはサーバーが受け付ける `Host` だけです。

```bash
curl -fsS "$SERVER/api/health"
```

```json
{ "status": "ok", "version": "[[version]]", "auth": true }
```

`auth` は、リクエストにトークンが必要かどうかを示します。`signal-lab-server healthcheck` は、サーバーが待ち受けている同じアドレスに尋ね、応答があれば `0` で終了します。Docker イメージのヘルスチェックはこれを実行します。

## OpenAPI の記述 {#openapi}

`GET /api/openapi.json` は、この API を OpenAPI 3.1 で記述します。エンドポイント、引数付きのすべてのコマンド、実行のリクエストと結果です。`/api/` のほかの部分と同じく、トークンが必要です。これらのページが完全なリファレンスであり、両者が食い違う場合は、これらのページがエンジンに従っています。

## ジョブ {#jobs}

長く続く処理（モニター、ジェネレーター、バースト、リレー、接続、エミュレーター、実行）はジョブです。ジョブを開始するコマンドは、開始するとすぐに、その `JobInfo` を返します。

```json
{ "id": 4, "kind": "osc-monitor", "label": "OSC monitor 0.0.0.0:9000", "params": { "bind": "0.0.0.0:9000" }, "started_ms": 1759600000000 }
```

| フィールド | 内容 |
| --- | --- |
| `id` | ジョブの番号。サーバーが動いている間は一意で、ほかのコマンドは `jobId` または `id` としてこれを受け取ります |
| `kind` | `experiment`、`osc-monitor`、`osc-gen`、`http-burst`、`netsim`、`storm`、`scan`、`beacon`、`discovery`、`mqtt`、`websocket`、`emulator` |
| `label` | ログ向けの、英語の 1 行 |
| `params` | ラベルを構成する値（宛先、バインド、ホストなど）。ない場合は省略されます |
| `started_ms` | 開始した時刻。1970 年からのミリ秒 |

ジョブを開始するコマンドは、`experiment_start`、`osc_monitor_start`、`osc_generator_start`、`http_burst_start`、`netsim_start`、`storm_start`、`scan_start`、`broadcast_beacon_start`、`discovery_start`、`mqtt_connect`、`ws_connect`、`emulator_start` です。サーバーは、開始のたびに、要求したクライアントのアドレスとともにログに記録します。`POST /api/run` もジョブを開始します。

- [`jobs_list`](commands.md#jobs_list) は実行中のジョブを一覧し、[`job_stop`](commands.md#job_stop) は 1 つを停止し、[`jobs_stop_all`](commands.md#jobs_stop_all) はすべてを停止します。
- 自力で終了した、または失敗したジョブは、[`job://ended`](events.md#event-job-ended) を送ります。停止したジョブは、それ以上何も送りません。`job_stop` が `true` を返すことが、その確認です。
- ジョブはサーバーのものであり、開始したクライアントのものではありません。ページを閉じたりスクリプトを終了したりしても止まらず、すべてのクライアントがそれを見られて、停止することもでき、サーバーがシャットダウンするとすべて停止します。

## 完全な例 {#example}

サーバーが起動しているかを尋ね、1 つのコマンドで小さな HTTP エミュレーターを開始し、同梱の `http-check` 実験をそれに対して実行して結果を待ち、その後エミュレーターを停止します。`jq` が応答からフィールドを取り出します。

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)            # トークンのないループバックのサーバーでは省略する
AUTH="Authorization: Bearer $TOKEN"
JSON="Content-Type: application/json"

# 1. 起動している? バージョンは? トークンが必要か?
curl -fsS "$SERVER/api/health"
# {"status":"ok","version":"[[version]]","auth":true}

# 2. 1 つのコマンド: GET / に 200 で答える、127.0.0.1:8080 の HTTP エミュレーター
EMULATOR=$(curl -fsS -X POST "$SERVER/api/invoke/emulator_start" -H "$AUTH" -H "$JSON" -d '{
  "emulator": { "name": "Example", "bind": "127.0.0.1:8080", "protocol": "http",
                "routes": [ { "method": "GET", "path": "/", "responses": [ { "body": "ok" } ] } ] }
}' | jq .id)

# 3. http://127.0.0.1:8080/ から 200 を期待する同梱の実験を実行して、待つ
curl -sS -X POST "$SERVER/api/run" -H "$AUTH" -H "$JSON" -d '{"template":"http-check"}' \
  | jq '{outcome, error, report_path}'
# {"outcome":"passed","error":null,"report_path":"/data/runs/run-1759600000000-2.json"}

# 4. エミュレーターを停止する
curl -fsS -X POST "$SERVER/api/invoke/job_stop" -H "$AUTH" -H "$JSON" -d "{\"id\":$EMULATOR}"
# true
```

`curl -f` は、エラーステータスをコマンドの失敗にします。本文の `EngineError` を見たいときは、手順 3 のように省略してください。

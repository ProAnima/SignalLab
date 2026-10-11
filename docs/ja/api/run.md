---
title: 実行
description: POST /api/run は、Signal Lab サーバー上で実験を実行し、結果を返すか、ステップを NDJSON の行としてストリームします。
---

# 実験の実行

スクリプトやパイプラインからサーバー上で実験を実行し、その結果を知るには、実験を `POST /api/run` に送ります。サーバーは実験を最後まで実行し、結果を返します。求めれば、各ステップを発生するたびに返します。[`signallab run --server`](../automation/cli.md#cli-run) が使っているのはこれです。

エディターの [[ui:exp.run]] や [`experiment_start`](commands.md#experiment_start) と同じ実行です。開いているすべてのページから見えて停止できるジョブで、同じイベントが発生し、同じレポートがデータフォルダーに保存されます。

## リクエスト {#request}

```http
POST /api/run
Authorization: Bearer <token>
Content-Type: application/json

{ "template": "osc-ping-reply", "overrides": { "device": "192.0.2.20:9000" }, "seed": 42, "timeout": 30 }
```

本文には、実験を 1 つ（`document` または `template`。両方は不可）と、それを何で実行するかを指定します。

| フィールド | 型 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `document` | object | — | エディターが保存、エクスポートする形の実験。古いバージョンは、ファイルを開くときと同じように移行されます |
| `template` | string | — | ファイル名で指定する同梱テンプレート。`.json` の有無は問いません（下記） |
| `overrides` | object | `{}` | この実行だけのパラメーター値。値は文字列、数値、真偽値のいずれかで、それぞれが実験のパラメーターでなければなりません |
| `profile` | string | ドキュメントのもの | このプロファイルで実行します。`""` は既定値で実行します |
| `seed` | number | ドキュメントのもの、なければ新規 | 0 から 9007199254740991。同じシードは同じ乱数値を引きます |
| `timeout` | number | `300` | 実行が `run.timeout` で失敗するまでの秒数。1〜300 |

サーバーが知らないフィールドは拒否されます（`400`、`api.run_invalid`）。ドキュメントは、開いたファイルと同じように読まれるので、最大 4 MiB です。

同梱のテンプレートは、エディターが [[ui:exp.templates]] で提供する実験です。

| `template` | 内容 |
| --- | --- |
| `empty` | 開始と終了 |
| `http-check` | `http://127.0.0.1:8080/` への GET と、ステータス 200 のチェック |
| `status-branch` | `http://127.0.0.1:8080/` への GET。200 なら OSC メッセージ、それ以外は 500 ms の遅延 |
| `parallel-flows` | [[ui:exp.node.fork]] から、`http://127.0.0.1:8080/` への GET とログのエントリが並んで走り、[[ui:exp.node.join]] で合流します |
| `osc-ping-reply` | パラメーター `device`（`127.0.0.1:9000`）への OSC `/ping` と、`127.0.0.1:9001` での `/pong` を 2 秒待つ待機 |
| `poll-until-ready` | `device` に OSC で `/status` を尋ね、`ready` と答えるまで最大 10 回繰り返す [[ui:exp.node.loop]] |
| `flaky-api` | しばらく失敗してから動作するエミュレートされた API（パラメーター `api`）を、200 が返るまで [[ui:exp.node.loop]] で尋ねる |
| `fault-phases` | 劣化リレーの向こうにある UDP デバイスに 8 秒間送信し、その間にリレーが、正常、ロスあり、オフライン、正常と切り替わる |
| `dependency-outage` | 2 秒間ダウンさせるエミュレートされた API（パラメーター `api`）に、再び 200 が返るまで [[ui:exp.node.loop]] で尋ね続ける |
| `websocket-echo` | パラメーター `service`（`ws://127.0.0.1:9001/echo`）への接続、メッセージ、そのエコーの待機、チェック、クローズ |

ノードとパラメーターを見るには、[[ui:exp.templates]] で 1 つを開いてください。[実験](../experiments/index.md)を参照してください。

## 結果 {#result}

既定では、応答は `Content-Type: application/json` の `200` で、実行が終わったときに送られます。実行の結果である JSON オブジェクト 1 つです。

```json
{
  "job_id": 12,
  "experiment": "OSC ping → reply",
  "outcome": "passed",
  "seed": 42,
  "profile": null,
  "overridden": true,
  "params": { "device": "192.0.2.20:9000" },
  "started_ms": 1759600000000,
  "ended_ms": 1759600000310,
  "steps": [ { "job_id": 12, "ts": 1759600000001, "node_id": "start", "state": "running", "detail": "", "message_key": null, "message_params": null }, … ],
  "report_path": "/data/runs/run-1759600000000-12.json"
}
```

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | 実行のジョブ |
| `experiment` | string | 実験の名前 |
| `outcome` | string | `passed`、`failed`、`stopped` |
| `seed` | number | 実行に使われたシード。同じ値を引くには、これを `seed` として返します |
| `profile` | string or null | 実行に使われたプロファイル |
| `overridden` | boolean | 一部の値が `overrides` から来た |
| `params` | object | 実行が使ったすべてのパラメーター値 |
| `started_ms`、`ended_ms` | number | 1970 年からのミリ秒 |
| `error` | `EngineError` | 失敗の理由。最初の失敗です。成功した場合は省略されます |
| `steps` | object[] | 発生した順のすべてのステップ。[`experiment://step`](events.md#event-experiment-step) と同じ形です |
| `emulators` | object[] | 各 [[ui:exp.node.emulator]] ノードが受け取り、答えたもの: `node`、`name`、`protocol`、`local`、`counts`。ない場合は省略されます |
| `impairments` | object[] | 各 [[ui:exp.node.impairment]] ノードのリレーが、フェーズごとに何をしたか。ない場合は省略されます |
| `report_path` | string | サーバー上の実行のレポート。[`/api/files`](index.md#files) でダウンロードします。書き込まれなかった場合は省略されます |
| `report_error` | `EngineError` | レポートを書き込めなかった理由。そうでなければ省略されます |

シークレットの値は、そのすべてでマスクされます。レポートファイルには同じステップが入っています。[実行とレポート](../experiments/runs.md)を参照してください。

実行が続いている間、サーバーは 15 秒ごとに空白を 1 つ送ります。JSON は値の前の空白を無視するので、結果は問題なく解析でき、プロキシが、長く静かな実行を切れた接続と見なすこともありません。

## ステップを追う {#lines}

ステップを発生するたびに見るには、NDJSON を要求します。

```http
Accept: application/x-ndjson
```

応答は `Content-Type: application/x-ndjson` の `200` で、1 行に JSON オブジェクト 1 つが、それぞれ `type` を持って返ります。

| `type` | 発生するとき | 行の残りの内容 |
| --- | --- | --- |
| `started` | 最初に 1 回 | `job_id`、`experiment`、`seed`、`profile`、`overridden`、`started_ms` |
| `step` | ステップごと | ステップ。[`experiment://step`](events.md#event-experiment-step) と同じ形です |
| `heartbeat` | 15 秒ごと | なし |
| `ended` | 最後に 1 回 | 上記の結果 |

```text
{"job_id":12,"experiment":"OSC ping → reply","seed":42,"profile":null,"overridden":true,"started_ms":1759600000000,"type":"started"}
{"job_id":12,"ts":1759600000001,"node_id":"start","state":"running","detail":"","message_key":null,"message_params":null,"type":"step"}
…
{"job_id":12,"experiment":"OSC ping → reply","outcome":"passed",…,"type":"ended"}
```

`ended` まで行を読み、知らない `type` は無視してください。サーバーは `X-Accel-Buffering: no` を送るので、nginx のプロキシは各行をすぐに先へ渡します。

## ステータスと結果 {#status}

HTTP のエラーステータスは、**実行が開始されなかった**ことを意味します。本文は [`EngineError`](index.md#errors) です。

| ステータス | コード | 理由 |
| --- | --- | --- |
| `400` | `api.run_invalid` | 本文が実行リクエストではない: JSON でない、不明なフィールド、文字列・数値・真偽値でない上書き |
| `400` | `api.run_source` | `document` も `template` もない、または両方がある |
| `415` | `command.json_required` | `Content-Type: application/json` ではない |
| `422` | `api.template_unknown` | その名前の同梱テンプレートがない |
| `422` | `file.json_invalid`、`file.too_large`、`doc.*` | ドキュメントを読み取れない |
| `422` | 任意の検証コード、`run.override_unknown`、`profile.active_missing`、`run.limit_range`、`seed.range`、`secret.missing`、`transport.address_in_use`… | 実験を開始できない: 検証に通らない、値が範囲外、シークレットが保存されていない、待ち受けるポートが使用中 |

実行が開始された後は、何が起きてもステータスは `200` です。結果の `outcome` を読んでください。

| `outcome` | 意味 |
| --- | --- |
| `passed` | すべてのステップが成功し、[[ui:exp.node.end]] に到達した |
| `failed` | ステップが失敗した、または実行が `timeout` より長くかかった（`run.timeout`）。`error` がどちらで、なぜかを示す |
| `stopped` | 終了する前に停止された: `job_stop`、[[ui:app.stopAll]]、またはサーバーのシャットダウンによる。`steps` には到達したステップが入り、レポートは保存されない |

## クライアントが去った場合 {#disconnect}

接続を閉じても、実行は止まりません。これはサーバー上のジョブで、ブラウザーで開始した実行がタブを閉じても続くのと同じように、最後まで実行されてレポートが保存されます。[`jobs_list`](commands.md#jobs_list) で見つけ、[`job_stop`](commands.md#job_stop) で停止し、後で [`experiment_runs`](commands.md#experiment_runs) でそのレポートを読めます。サーバーがシャットダウンすると、実行は停止され、まだ接続しているクライアントは `"outcome": "stopped"` を受け取ります。

## 例 {#examples}

同梱のテンプレートを実行し、結果を待ちます。

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
curl -sS -X POST "$SERVER/api/run" \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"template":"osc-ping-reply","overrides":{"device":"192.0.2.20:9000"},"timeout":30}' \
  | jq -r .outcome
```

パラメーターを 1 つ変えた自分の実験を送り、各ステップを発生するたびに表示します。

```bash
jq '{document: ., overrides: {api: "http://192.0.2.10:8080"}, profile: ""}' smoke.json |
  curl -sSN -X POST "$SERVER/api/run" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    -H "Accept: application/x-ndjson" --data @- |
  jq -r 'select(.type == "step") | "\(.node_id)  \(.state)  \(.detail)"'
```

`-N` は、`curl` が行をためこむのを防ぎます。失敗した実行でパイプラインを失敗させるには、`outcome` をチェックします。

```bash
outcome=$(curl -sS -X POST "$SERVER/api/run" -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" -d '{"template":"http-check"}' | jq -r .outcome)
[ "$outcome" = "passed" ]
```

## コマンドラインから {#cli}

[`signallab run`](../automation/cli.md#cli-run) に `--server <url>` を付けると、このエンドポイントを通してサーバー上で実行されます。読み込んだ実験を `overrides`、`seed`、`timeout` とともに送り、NDJSON を要求し、各ステップを、その行が届くたびに表示します。トークンは `--token-file`、なければ `SIGNALLAB_TOKEN` から取られます。`--report` を付けると、`/api/files` を通してレポートをダウンロードします。60 秒間（ハートビートも含めて）何も送ってこないサーバーは、去ったものと見なされます。

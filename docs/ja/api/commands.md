---
title: コマンド
description: Signal Lab のエンジンのすべてのコマンドを、POST /api/invoke/<command> として呼び出す方法と、その引数、結果、エラーとともに説明します。
---

# コマンド

エンジンが持つすべてのコマンドを、対象ごとにまとめています。それぞれ、引数の JSON オブジェクトを付けて `POST /api/invoke/<command>` として呼び出し、結果とともに `200` を返すか、[`EngineError`](index.md#errors) とともに `422` を返します。認証の方法とステータスの意味は [API の概要](index.md)にあります。

## 規約 {#conventions}

- **引数名**は camelCase です（`jobId`、`nodeId`）。コマンドが知らない引数や、必須なのに省略された引数は `command.args_invalid` で拒否されます。引数を取るすべてのコマンドが、これで失敗する可能性があります。引数のないコマンドは本文を取らず、`null` か `{}` のみです。それ以外も `command.args_invalid` で拒否されます。
- **引数として渡すオブジェクト**（`config`、`request`、`document`、`library`、`emulator`、`profile`）は、エンジン自身のフィールド名を使い、そのほとんどは snake_case です（`timeout_ms`）。その中では、エンジンが知らないフィールドは**無視される**ので、綴りを間違えた任意のフィールドは、黙って既定値のままになります。不明なフィールドを拒否するのは、`feedback_send` のフォームだけです。
- **任意の**引数とフィールドは、省略するか `null` として送れます。既定値は表にあります。
- **結果**は JSON です。「null」は、コマンドが返すものがないことを意味します。
- `IP:port` と書かれた**アドレス**は、数値のアドレスとポートを受け付けます（`127.0.0.1:9000`、`[::1]:9000`）。そこにホスト名を書くと拒否されます。表が `IP:port` または `host:port` と書いている場所では、ホスト名も使えます。ホスト名はコマンドの実行時に解決され、IPv4 アドレスがあればそれが使われます（`localhost:9000` は `127.0.0.1:9000` になります）。
- **ジョブ**: *ジョブを開始する*と記したコマンドは、[`JobInfo`](#type-jobinfo) を返します。処理は、終わるか、[`job_stop`](#job_stop) で停止されるまで続きます。[ジョブ](index.md#jobs)を参照してください。
- 結果に含まれる**パス**は、エンジンが動いているマシン上のものです。サーバーでは、そのデータフォルダー内にあります。[`/api/files`](index.md#files) でダウンロードします。

例では、次のシェル関数を使います。

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
invoke() {
  curl -sS -X POST "$SERVER/api/invoke/$1" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    --data "${2:-}"
}
```

## アプリケーション {#application}

### app_info {#app_info}

エンジンが何であり、どこで動いているか。引数はありません。

**結果**

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `version` | string | Signal Lab のバージョン、`[[version]]` |
| `mode` | string | `desktop` または `server` |
| `secrets_writable` | boolean | [`secret_set`](#secret_set) と [`secret_delete`](#secret_delete) がここで動作するか。サーバーでは `false` |
| `data_dir` | string | エンジンが動いているマシン上のデータフォルダー |
| `os` | string | `windows`、`linux`… |
| `arch` | string | `x86_64`、`aarch64`… |

### get_host_info {#get_host_info}

マシンの名前と、送信元になるアドレス。引数はありません。

**結果**: `{ "local_ip": string, "hostname": string }`。`local_ip` は、インターネットへの通信のためにシステムが選ぶ IPv4 アドレス（何も送信せずに調べます）で、ない場合は `127.0.0.1` です。`hostname` はコンピューターの名前で、システムが答えないときは `localhost` です。

### firewall_status {#firewall_status}

システムのファイアウォールが、ほかのマシンからこのプログラムへの到達を許可しているか。プログラムごとのファイアウォールを読めるのは Windows だけで、それ以外では `applies` が `false` になり、残りのうち `program` だけが入ります。引数はありません。

**結果**

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `applies` | boolean | ここにはプログラムごとのファイアウォールがある（Windows） |
| `program` | string | ルールの対象のプログラム |
| `enabled` | boolean | マシンがいま接続しているネットワークで、ファイアウォールがオンである |
| `networks` | string[] | マシンが接続しているネットワークの種類: `domain`、`private`、`public` |
| `allowed` | boolean | 現在のネットワークで、このプログラムへの UDP を許可する受信規則がある |
| `blocked` | boolean | 現在のネットワークで、このプログラムをブロックする受信規則がある。どの許可規則よりも優先されます |
| `rules` | number | このプログラムの受信規則の数（種類を問わない） |

**エラー**: `firewall.failed`。

### firewall_allow {#firewall_allow}

ほかのマシンが Signal Lab に到達できるようにします。システム自身の管理者権限のプロンプトが表示され、その後、プログラムの受信規則（ブロック規則を含む）が、Signal Lab とその隣の `signallab` コマンドラインについて、それぞれ 1 つの許可規則に置き換えられます。Windows のデスクトップアプリのみです。サーバーは、プロンプトに答える人が画面の前にいないので拒否します。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `public` | boolean | はい | プライベートとドメインのネットワークだけでなく、パブリックネットワークでも許可する |

**結果**: 新しい [`firewall_status`](#firewall_status)。

**エラー**: `firewall.server`（サーバーの場合）、`firewall.unsupported`（Windows ではない）、`firewall.declined`（プロンプトで「いいえ」が選ばれた）、`firewall.failed`。

### feedback_send {#feedback_send}

Signal Lab の開発者にメッセージを、スタジオのハブ経由で送ります。ハブがそれをメールで開発者に送ります。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `form` | object | はい | メッセージ（下記）。不明なフィールドは拒否されます |

| `form` のフィールド | 型 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `message` | string | — | 何が起きたか。必須で、最大 20,000 文字 |
| `email` | string | なし | 返信を送れる宛先 |
| `meta` | object of strings | `{}` | アプリが自分について伝える内容（version、os、arch、mode、lang、screen） |
| `screenshots` | `{ name, data }[]` | `[]` | 画像。`data` は base64。最大 6 枚、各 8 MiB まで |
| `logs` | `{ name, text }[]` | `[]` | テキストファイル。最大 4 つ、各 2 MiB まで |

すべてを合わせて最大 15 MiB です。

**結果**: `{ "id": string }`。開発者が受け取る参照番号です。

**エラー**: `feedback.message_required`、`feedback.message_too_long`、`feedback.too_many_files`、`feedback.file_too_large`、`feedback.too_large`、`feedback.invalid`、ハブによる拒否（`feedback.email_invalid`、`feedback.file_type`、`feedback.rate_limited`、`feedback.disabled`、`feedback.send_failed`、`feedback.failed`）、ネットワークの `transport.*`。

```bash
invoke app_info
# {"version":"[[version]]","mode":"server","secrets_writable":false,"data_dir":"/data","os":"linux","arch":"x86_64"}
```

## ジョブ {#jobs}

### jobs_list {#jobs_list}

実行中のジョブを、古いものから順に返します。引数はありません。

**結果**: [`JobInfo`](#type-jobinfo)`[]`。

### job_stop {#job_stop}

1 つのジョブを直ちに停止します。ソケットが閉じ、リレー、サーバー、接続が終わります。応答は、それらが閉じてから返ります。直後に開始すれば、同じポートをバインドできます。停止したジョブは [`job://ended`](events.md#event-job-ended) を送らず、停止した実行はレポートを保存しません。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `id` | number | はい | ジョブの `id` |

**結果**: その ID のジョブが実行中だった場合は `true`、そうでなければ `false`。

### jobs_stop_all {#jobs_stop_all}

実行中のすべてのジョブを、誰が開始したかにかかわらず停止し、[`job_stop`](#job_stop) と同じく、それらがポートを手放してから応答します。まだ開始中のジョブ（ポートを開いている実行）は、開始と同時に停止されます。引数はありません。

**結果**: null。

```bash
invoke jobs_list
# [{"id":3,"kind":"osc-monitor","label":"OSC monitor 0.0.0.0:9000","params":{"bind":"0.0.0.0:9000"},"started_ms":1759600000000}]
invoke job_stop '{"id":3}'
# true
```

## 実験と実行 {#experiments}

これらのコマンドは、実験のドキュメント（`Experiment`）を受け取って返します。エディターが保存、エクスポートする JSON で、`version`、`name`、`params`、`profiles`、`profile`、`seed`、`cookies`、`nodes`、`edges` を持ちます。ノードは[ノード](../experiments/nodes.md)に、パラメーター、プロファイル、テンプレートは[データ](../experiments/data.md)にあります。ドキュメントは最大 4 MiB です（`file.too_large`）。実験を実行して結果を待つには、[`experiment_start`](#experiment_start) ではなく [`POST /api/run`](run.md) を使ってください。

### experiment_load {#experiment_load}

作業中の実験: データフォルダーの `experiment.json` です。サーバーでは、そのインターフェイスが表示しているものです。なければ、初期の実験です。古いドキュメントのバージョンは移行されます。引数はありません。

**結果**: `Experiment`。

**エラー**: `file.io`、`file.json_invalid`（ファイルの `path`、`line`、`column` 付き）、`file.too_large`、`doc.version_unsupported`、そのほかの `doc.*` のチェック。

### experiment_save {#experiment_save}

作業中の実験、つまりデータフォルダーの `experiment.json` を置き換えます。先に一時ファイルに書き込むので、書き込みに失敗しても前のものが残ります。

::: warning
サーバーでは、これはすべてのブラウザーのエディターが扱うドキュメントです。
:::

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `document` | `Experiment` | はい | ドキュメント |

**結果**: string。書き込んだパス。

**エラー**: `doc.*`、ドキュメントのサイズのチェック（`param.*`、`params.too_many`、`profile.*`、`profiles.too_many`、`seed.range`）、`file.too_large`、`file.io`。

### experiment_parse {#experiment_parse}

[[ui:exp.importJson]] と同じように、JSON テキストから実験を読み取ります。バージョン 1 から 8 は、現在のバージョン 9 に移行されます。バージョン 8 より前のファイルは `cookies` がオフの状態で開くので、以前と同じように実行されます。バイトオーダーマークは読み飛ばされます。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `text` | string | はい | ファイルのテキスト |

**結果**: `Experiment`。

**エラー**: `file.json_invalid`（`line`、`column`）、`file.too_large`、`doc.version_unsupported`、`doc.*`、[`experiment_save`](#experiment_save) のサイズのチェック。

### experiment_export {#experiment_export}

ドキュメントのスナップショットを、データフォルダーの `exports/experiment-<ms>-<16 hex digits>.json` に書き出します。エクスポートのたびに新しいファイルになります。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `document` | `Experiment` | はい | ドキュメント |

**結果**: string。書き込んだパス。

**エラー**: [`experiment_save`](#experiment_save) のもの。

### experiment_validate {#experiment_validate}

ドキュメントが、有効なプロファイル（なければ既定値）で実行できるかをチェックします。グラフ、すべてのフィールド、パラメーター、そして指定されているすべてのシークレットが保存されているかです。実行を妨げる問題がエラーになります。成功した場合は、*ほかの*プロファイルのうちどれが失敗するかが示されるので、切り替える前に分かります。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `document` | `Experiment` | はい | ドキュメント |
| `overrides` | object of strings | いいえ | このチェックだけのパラメーター値。[[ui:exp.runWith]] が与えるものと同じです |

**結果**: `{ "profile": string or null, "error": EngineError }[]`。検証に通らないほかのプロファイルごとの項目です（`null` は、プロファイルなしの既定値）。空のリストは、すべてのプロファイルに問題がないことを意味します。

**エラー**: 任意の検証コード（`doc.*`、`graph.*`、`node.*`、`param.*`、`profile.*`、`template.*`、`loop.*`…）、`run.override_unknown`（ドキュメントにないパラメーターへの上書き）、`secret.missing`、`secret.store`、`secret.unsupported`。

### experiment_resolve {#experiment_resolve}

エディターのプレビューが表示するように、テンプレートを埋めたノードを 1 つ返します。有効なプロファイルの値と、指定した変数の値が使われます。シークレットは `••••` と表示され、値は表示されません。値のない名前は書かれたままで、一覧されます。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `document` | `Experiment` | はい | ドキュメント |
| `nodeId` | string | はい | ノード |
| `vars` | object | はい | 使う変数の値を名前で指定。なければ `{}` |

**結果**: `{ "node": node, "missing": string[] }`。

**エラー**: `node.not_found`、`template.*`、`secret.store`、`secret.unsupported`。

### experiment_send_node {#experiment_send_node}

[[ui:exp.sendNow]]: 1 つのノードを、実行と同じコードで単独で実行します。アクションは送信され、待機ノードは、一致するかタイムアウトするまで、いまから待ち受けます。[[ui:exp.node.ws_send]] または [[ui:exp.node.wait_ws]] ノードは、対応する [[ui:exp.node.ws_connect]] ノードが記述する接続を開きます。Cookie は使われず、実行の [[ui:exp.node.impairment]] と [[ui:exp.node.emulator]] ノードは開かれません。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `document` | `Experiment` | はい | ドキュメント。有効なプロファイルがパラメーターの値を与えます |
| `nodeId` | string | はい | アクションまたは待機ノード |
| `vars` | object | はい | ノードのテンプレートが読む変数の値。なければ `{}` |

**結果**

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `detail` | string | 何が起きたか（英語） |
| `response` | [`HttpResponse`](#type-httpresponse) or null | HTTP ノードのレスポンス |
| `vars` | object | ステップが設定したもの: 待機ノードの応答、またはリクエストの後ろにある [[ui:exp.node.extract]] ノードがそのレスポンスから取るもの |

シークレットの値は、そのすべてでマスクされます。

**エラー**: `node.not_found`、`run.not_an_action`（アクションでも待機ノードでもない）、`ws.connection_unknown`、`secret.missing`、`template.*`、そしてステップが失敗する理由: `transport.*`、`wait.timeout`、`check.*`…

### experiment_start {#experiment_start}

[[ui:exp.run]] と同じように実行を開始し、すぐに戻ります。ステップは [`experiment://step`](events.md#event-experiment-step) イベントとして、終了は [`experiment://ended`](events.md#event-experiment-ended) として届き、レポートはデータフォルダーの `runs/` に保存されます。待機、エミュレーター、劣化リレー、MQTT のサブスクリプションは最初のステップの前に開かれるので、使用中のポートがあるとここで失敗します。300 秒より長い実行は `run.timeout` で失敗します。*ジョブを開始します*（`experiment`）。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `document` | `Experiment` | はい | ドキュメント |
| `overrides` | object of strings | いいえ | この実行だけのパラメーター値 |
| `seed` | number | いいえ | 実行のシード。0 から 9007199254740991。既定値: ドキュメントのもの、なければ新規 |

**結果**: [`JobInfo`](#type-jobinfo)。`params.name` は実験の名前です。

**エラー**: [`experiment_validate`](#experiment_validate) が報告するすべて、`seed.range`、`transport.address_in_use` とそのほかのバインドの失敗、`emulator.*`、`impair.*`、`node.params_only`（実行の開始時に固定されていない、待ち受けアドレスや MQTT 待機のブローカーまたはトピック）、MQTT 待機が到達できないブローカーに対する [`mqtt_connect`](#mqtt_connect) のエラー。

### experiment_runs {#experiment_runs}

`runs/` のレポートから読み戻した実行を、新しいものから順に返します。読み取れないレポートは除外されます。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `name` | string | いいえ | この名前と完全に一致する実験の実行のみ |
| `limit` | number | いいえ | 最大でこの件数。既定値は 50、上限は 500 |

**結果**: 実行の要約:

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `name` | string | レポートのファイル名 `run-<ms>-<job>.json`。[`experiment_compare`](#experiment_compare) が受け取るもの |
| `experiment` | string | 実験の名前 |
| `started_ms`、`ended_ms` | number | 1970 年からのミリ秒 |
| `outcome` | string | `passed` または `failed` |
| `seed` | number | 実行のシード |
| `profile` | string or null | そのプロファイル |
| `loads` | object[] | 各負荷ステップ: `node`、`sent`、`rps`、`p95_ms`、`error_rate`、`held`（すべてのしきい値が成立した） |

**エラー**: `file.io`。

### experiment_compare {#experiment_compare}

2 つの実行を、タイムラインの [[ui:exp.compare]] が表示するように、負荷ステップごとに並べます。ステップはノードの ID で対応付けられます。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `a` | string | はい | 前の実行のレポートのファイル名 |
| `b` | string | はい | 後の実行のレポートのファイル名 |

**結果**: `{ "a": summary, "b": summary, "steps": [...] }`。各ステップは、`node`、`missing_in`（片方の実行にしかない場合の `a` または `b`）、`metrics`、`sent`（`[a, b]`）、`thresholds_a` と `thresholds_b`（各しきい値を `{ metric, op, value, actual, held }` で）を持ちます。`metrics` には 9 つのメトリクスが一覧され、それぞれ `{ metric, a, b, change, percent, worse }` です。`metric` は `p50_ms`、`p90_ms`、`p95_ms`、`p99_ms`、`mean_ms`、`max_ms`、`error_rate`、`rps`、`missed` のいずれかで、`change` は `b − a`、`percent` は `a` に対する変化の割合（%。`a` が 0 のときは null）、`worse` は、悪い方向（高くなった、`rps` では低くなった）に 5% 以上、または 0 から何らかの値に動いたことを示します。片方の実行にしかないステップが `worse` になることはありません。[負荷](../experiments/load.md)を参照してください。

**エラー**: `runs.name_invalid`（レポートのファイル名以外。フォルダーは不可）、`runs.not_found`、`file.json_invalid`、`file.io`。

```bash
invoke experiment_validate "$(jq '{document: .}' experiment.json)"
# []
```

## シークレット {#secrets}

シークレットの値は、実験が `{{secret.NAME}}` として使い、エンジンの外に出ることはありません。値を返すコマンドはありません。どこに保存されるかは、エンジンが動いている場所で決まります。

| 場所 | ストア | 設定と削除 |
| --- | --- | --- |
| デスクトップアプリ、Windows | Windows 資格情報マネージャー | できる |
| デスクトップアプリ、Linux | なし | `secret.unsupported` |
| サーバー | `SIGNALLAB_SECRET_<NAME>`、または `--secrets-dir`（既定は `/run/secrets/signallab`）内のファイル `<NAME>` | できない: `secret.read_only` |

名前はラテン文字の英字または `_` で始まり、ラテン文字の英字、数字、`_` が続き、128 文字までです（`secret.name_invalid`）。

### secret_status {#secret_status}

指定した名前のうち、値が保存されているもの。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `names` | string[] | はい | 調べる名前 |

**結果**: オブジェクト。名前 → `true`（保存済み）または `false`。

**エラー**: `secret.name_invalid`（サーバーの場合）、`secret.store`、`secret.too_large`（16 KiB を超えるサーバーのファイル）、`secret.unsupported`。

### secret_set {#secret_set}

名前に値を保存し、すでにあるものを置き換えます。デスクトップアプリのみです。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `name` | string | はい | 名前 |
| `value` | string | はい | 空でないこと。最大 16 KiB |

**結果**: null。

**エラー**: `secret.read_only`（サーバーの場合）、`secret.unsupported`、`secret.name_invalid`、`secret.empty`、`secret.too_large`、`secret.store`。

### secret_delete {#secret_delete}

保存されている値を削除します。保存されていないものを削除してもエラーにはなりません。デスクトップアプリのみです。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `name` | string | はい | 名前 |

**結果**: null。

**エラー**: `secret.read_only`（サーバーの場合）、`secret.unsupported`、`secret.name_invalid`、`secret.store`。

```bash
invoke secret_status '{"names":["API_TOKEN","MQTT_PASSWORD"]}'
# {"API_TOKEN":true,"MQTT_PASSWORD":false}
```

## OSC {#osc}

これらのコマンドが対応する画面については、[OSC](../protocols/osc.md) を参照してください。

### osc_send {#osc_send}

OSC メッセージを、新しいソケットから 1 つの UDP データグラムで送信します。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `target` | string | はい | 送信先の `IP:port` または `host:port`。名前は解決され、IPv4 アドレスがあればそれが使われます |
| `address` | string | はい | OSC アドレス、`/mixer/fader/1`。`/` で始まります |
| `args` | [`OscArg`](#type-oscarg)`[]` | はい | 引数。なければ `[]` |

**結果**: number。送信したバイト数。

**エラー**: `node.osc_address`（先頭の `/` がない。フィールド `address`）、`transport.target_invalid`（ポートがない、またはどちらの形式でもない）、`transport.dns`（名前が解決できない）、`transport.*`。

### osc_monitor_start {#osc_monitor_start}

UDP ポートで OSC を待ち受け、すべてのパケットをデコードします。それぞれが [`osc://message`](events.md#event-osc-message) イベントとして届きます。*ジョブを開始します*（`osc-monitor`、`params.bind`）。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `bind` | string | はい | 待ち受ける `IP:port`: `0.0.0.0:9000` はすべてのネットワークカード、`127.0.0.1:9000` はこのマシンのみ |

**結果**: [`JobInfo`](#type-jobinfo)。

**エラー**: `node.bind_invalid`、`transport.address_in_use`、`transport.address_unavailable`、`transport.denied`、`wait.bind_failed`。ソケットがこれ以上受信できなくなると、ジョブは `wait.receive_failed` で終了します。

### osc_generator_start {#osc_generator_start}

引数が波形に従って変化する OSC メッセージのストリームを送信します。進行状況は [`osc://gen-tick`](events.md#event-osc-gen-tick) として届きます。*ジョブを開始します*（`osc-gen`、`params.target`、`params.address`）。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `config` | object | はい | 下記 |

| `config` のフィールド | 型 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `target` | string | — | 送信先の `IP:port` または `host:port`。名前はジョブの開始時に 1 回解決されます |
| `address` | string | — | OSC アドレス。`/` で始まります |
| `rate` | number | — | 毎秒のメッセージ数。0.1 から 5000 の範囲に制限されます |
| `waveform` | string | — | `sine`、`triangle`、`saw`（下降: `max` から `min` まで、その後すぐに戻る）、`ramp`（上昇: `min` から `max` まで、その後すぐに戻る）、`square`、`random`、`constant`（`max`） |
| `freq` | number | — | 1 秒あたりの波形のサイクル数 |
| `min`、`max` | number | — | 値の範囲 |
| `as_int` | boolean | `false` | 浮動小数点数の代わりに、丸めた整数を送ります |
| `duration_s` | number | `0` | この秒数の後に停止します。0 は停止するまで実行します |

**結果**: [`JobInfo`](#type-jobinfo)。

**エラー**: `node.osc_address`、`transport.target_invalid`、`transport.dns`、`transport.*`。送信が失敗すると、ジョブは `transport.*` エラーで終了します。

```bash
invoke osc_send '{"target":"127.0.0.1:9000","address":"/cue/go","args":[{"type":"int","value":1}]}'
# 16
invoke osc_monitor_start '{"bind":"0.0.0.0:9000"}'
```

## HTTP と Cookie {#http}

[HTTP](../protocols/http.md) を参照してください。

### http_request {#http_request}

HTTP リクエストを 1 つ送信し、レスポンスを返します。レスポンスが得られないリクエスト（接続拒否、タイムアウト、解決できない名前、信頼されない証明書）は、コマンドのエラーでは**ありません**。レスポンスの `error` と `cause` がそれを示します。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `request` | [`HttpRequest`](#type-httprequest) | はい | リクエスト |
| `cookies` | boolean | いいえ | [[ui:nav.http]] 画面の Cookie ジャーを送り、応答が設定したものを保持します。既定値は `false` |

**結果**: [`HttpResponse`](#type-httpresponse)。

**エラー**: `node.method_invalid`（フィールド `method`。`BAD METHOD` のような、HTTP のトークンではないもの）、`http.client_failed`（リクエストの準備すらできなかった）。

### http_burst_start {#http_burst_start}

1 つのリクエストを、複数同時に何度も送信して、測定します。`rate` がない場合は、各ワーカーが、応答を得るとすぐにもう一度送信します。ある場合は、応答がどれだけ遅くても決まったスケジュールでリクエストが開始され、空いているワーカーを自分の時刻より 50 ms を超えて待ったリクエストはスキップされ、取りこぼしとして数えられます。進行状況は [`http://burst-progress`](events.md#event-http-burst-progress) として 1 秒に 10 回届きます。*ジョブを開始します*（`http-burst`、`params.method`、`params.url`、レート指定の場合は `params.rate`）。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `config` | object | はい | [`HttpRequest`](#type-httprequest) のフィールドと下記のフィールドを、1 つのオブジェクトにしたもの |

| `config` のフィールド | 型 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `concurrency` | number | — | 同時に処理中になる最大数。1 から 512 の範囲に制限されます |
| `total` | number | `0` | この数のリクエストの後に停止します。0 は回数の指定なし |
| `duration_s` | number | `0` | この秒数の後に停止します。0 は時間制限なし |
| `rate` | number | `0` | 毎秒開始するリクエスト数。0.1 から 100,000。0 は、応答が来るのと同じ速さ |
| `cookies` | boolean | `false` | [[ui:nav.http]] 画面の Cookie ジャーを使います |

`total` も `duration_s` もなければ、バーストは停止するまで実行されます。

**結果**: [`JobInfo`](#type-jobinfo)。

**エラー**: `http.rate_invalid`、`http.duration_invalid`、`node.method_invalid`、`http.client_failed`。

### http_cookies {#http_cookies}

[[ui:nav.http]] 画面の Cookie ジャー: 期限が切れていないすべての Cookie です。サーバーでは、すべてのページとスクリプトに対して 1 つのジャーがあります。引数はありません。

**結果**: Cookie。それぞれが `name`、`value`、`domain`、`host_only`（Domain 属性なし: 設定したホストにだけ送り返される）、`path`、`expires`（Unix 秒。セッション Cookie では null）、`secure`、`http_only`、`same_site`（文字列または null）を持ちます。

### http_cookies_clear {#http_cookies_clear}

[[ui:nav.http]] 画面の Cookie ジャーを空にします。引数はありません。

**結果**: null。

```bash
invoke http_request '{"request":{"method":"GET","url":"http://127.0.0.1:8080/health","headers":[["Accept","application/json"]],"body":null,"timeout_ms":5000}}' \
  | jq '{status, latency_ms, body}'
```

## WebSocket {#websocket}

[WebSocket](../protocols/websocket.md) を参照してください。`ws_connect` が開く接続はジョブで、ほかのコマンドは `jobId` でそれを指定します。

### ws_connect {#ws_connect}

WebSocket を開いて、開いたままにします。届いたものと送ったものは、100 ms ごとに [`ws://messages`](events.md#event-ws-messages) として、接続の状態は [`ws://state`](events.md#event-ws-state) として届きます。*ジョブを開始します*（`websocket`、`params.url`）。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | はい | 接続先と接続方法 |

**結果**: [`JobInfo`](#type-jobinfo)。

**エラー**: `ws.url_invalid`、`ws.header_invalid`、`ws.protocol_invalid`、`ws.handshake_status`（サーバーがアップグレードに別のステータスで答えた）、`ws.subprotocol_refused`、`ws.handshake_failed`、`transport.*`。

### ws_send {#ws_send}

開いている接続で、メッセージを 1 つ送信します。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `jobId` | number | はい | 接続のジョブ |
| `message` | object | はい | テキストメッセージなら `{ "text": "…" }`、バイナリなら `{ "hex": "de ad be ef" }`。どちらか一方だけ |

**結果**: number。送信したバイト数。

**エラー**: `ws.not_connected`、`ws.payload_required`（どちらもない、または両方ある）、`hex.invalid`（空の `hex` も）、`node.too_long`（16 MiB を超える。フィールド `payload`。何も送信されず、接続は開いたまま）、`ws.closed`、`transport.*`（サーバーが 10 秒間読み取りを止めたときは `transport.timeout`）。

### ws_close {#ws_close}

クローズハンドシェイクで接続を閉じ、サーバーの応答を最大 2 秒待ちます。その後、ジョブは終了します。接続が終了した後は、そのジョブはなくなっており、閉じようとすると `ws.not_connected` になります。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `jobId` | number | はい | 接続のジョブ |
| `code` | number | いいえ | 1000、またはアプリケーション独自の 3000 から 4999。既定値は 1000 |
| `reason` | string | いいえ | 最大 123 バイト。既定値は空 |

**結果**: `{ "code", "reason", "by", "error" }`。`by` は `client`、`server`、`lost` のいずれかです。`code` は、クローズがコードを持っていなかったときは 1005、クローズフレームがなかったときは 1006 です。

**エラー**: `ws.close_code`、`node.too_long`、`ws.not_connected`。

### ws_exchange {#ws_exchange}

ジョブなしの 1 回のやり取りです。接続し、指定があればメッセージを送り、求められていれば応答を待ち、閉じます。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | はい | 接続先と接続方法 |
| `message` | object | いいえ | `{ "text" }` または `{ "hex" }`。[`ws_send`](#ws_send) と同じです |
| `expect` | object | いいえ | 待つ対象: `mode`（`any`、`contains`、`regex`、`hex`。既定値は `any`）、`pattern`（既定値は空）、`timeout_ms`（既定値は 2000） |

`expect` があって `message` がない場合は、接続後に最初に一致したメッセージが対象になります。あいさつなどです。

**結果**: `{ "handshake", "sent", "reply", "closed" }`。`handshake` は `{ url, peer, local, protocol, ms }`、`sent` は送信したバイト数または null、`reply` は `{ kind, text, hex, bytes, json, ms }` または null（`json`: テキストの応答を解析したもの。なければ null。`ms`: 送信からの時間、何も送らなかった場合は接続からの時間）、`closed` は [`ws_close`](#ws_close) が返すものと同じです。

**エラー**: [`ws_connect`](#ws_connect) と [`ws_send`](#ws_send) のもの、`wait.timeout`（`ms`、`unmatched`、`target` 付き）、`regex.invalid` と `hex.invalid`（解析できない `pattern`）。

```bash
invoke ws_exchange '{"config":{"url":"ws://127.0.0.1:9001/"},"message":{"text":"{\"type\":\"ping\"}"},"expect":{"mode":"contains","pattern":"pong"}}' \
  | jq .reply.text
```

## MQTT {#mqtt}

プレーン TCP 上の MQTT 3.1.1、QoS 0、1、2。[MQTT](../protocols/mqtt.md) を参照してください。

### mqtt_connect {#mqtt_connect}

ブローカーに接続し、接続を保持します。コマンドは、ブローカーが接続を受け入れた（CONNACK）時点で戻るので、誤ったパスワードや閉じたポートはそのエラーになります。メッセージは 100 ms ごとに [`mqtt://messages`](events.md#event-mqtt-messages) として、状態の変化は [`mqtt://state`](events.md#event-mqtt-state) として、完了した QoS 1/2 のパブリッシュとサブスクライブ解除は [`mqtt://ack`](events.md#event-mqtt-ack) として届きます。*ジョブを開始します*（`mqtt`、`params.broker`、`params.client`）。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | はい | ブローカーと接続方法 |

**結果**: [`JobInfo`](#type-jobinfo)。

**エラー**: `mqtt.client_id_required`、`transport.*`（接続拒否、到達不能、`dns`、6 秒後の `timeout`）、`mqtt.no_answer`（6 秒以内に CONNACK がない）、`mqtt.protocol`、`mqtt.refused_protocol`、`mqtt.refused_client_id`、`mqtt.refused_unavailable`、`mqtt.refused_credentials`、`mqtt.refused_not_authorized`、`mqtt.refused`。接続の前には、`node.topic_filter`（3.1.1 が禁じる `subscribe` のフィルター）、`node.too_long`（MQTT の上限 65,535 バイトを超えるユーザー名、パスワード、クライアント ID、Will）、`node.topic_wildcard` / `mqtt.topic_invalid`（パブリッシュできないトピックの Will）があります。ブローカーが、ping に対してさえ、`keep_alive_s` の 1.5 倍の間何も返さない接続は、`job://ended` に `transport.timeout` を載せて終了します。

### mqtt_publish {#mqtt_publish}

開いている接続でパブリッシュします。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `jobId` | number | はい | 接続のジョブ |
| `topic` | string | はい | トピック: 空でなく、`+` や `#` を含まない |
| `payload` | string | はい | ペイロード。UTF-8 として送信されます |
| `qos` | number | はい | 0、1、2（2 を超える値は 2 として送られます） |
| `retain` | boolean | はい | ブローカーに保持を求めます。`retain` を付けた空のペイロードは、保持された値を消去します |

**結果**: null。QoS 1 または 2 のパブリッシュは、後から [`mqtt://ack`](events.md#event-mqtt-ack) で確認されます。

**エラー**: `node.topic_wildcard`、`mqtt.topic_required`、`mqtt.topic_invalid`（NUL を含む）、`node.too_long`（65,535 バイトを超える）。すべてフィールド `topic` 付きで、[`mqtt_publish_once`](#mqtt_publish_once) と同じく、コマンドは接続を探す前にこれらを拒否します。`mqtt.not_connected`。

### mqtt_subscribe {#mqtt_subscribe}

開いている接続を、フィルターにサブスクライブします。ブローカーが許可したものは、`state: "subscribed"` の [`mqtt://state`](events.md#event-mqtt-state) として届きます。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `jobId` | number | はい | 接続のジョブ |
| `filters` | `{ filter, qos }[]` | はい | 1 つ以上。`qos` の既定値は 0。`+` と `#` はワイルドカードです |

**結果**: null。

**エラー**: `mqtt.filter_required`、`node.topic_filter`（フィールド `topic`。3.1.1 が禁じるフィルター。`a/#/b` や `a+` など。ブローカーはこれ 1 つで接続を閉じます）、`mqtt.not_connected`。

### mqtt_unsubscribe {#mqtt_unsubscribe}

開いている接続を、フィルターからサブスクライブ解除します。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `jobId` | number | はい | 接続のジョブ |
| `filters` | string[] | はい | 1 つ以上 |

**結果**: null。ブローカーの応答は、`kind: "unsubscribed"` の [`mqtt://ack`](events.md#event-mqtt-ack) として届きます。

**エラー**: `mqtt.filter_required`、`node.topic_filter`（フィールド `topic`）、`mqtt.not_connected`。

### mqtt_publish_once {#mqtt_publish_once}

接続し、メッセージを 1 つパブリッシュし、その QoS が求める確認応答を（最大 6 秒）待って、切断します。独自の接続を、独自のクライアント ID で持ちます。`client_id` の最初の 12 文字、`-o`、数字からなる ID なので、その ID の稼働中の接続をブローカーから追い出すことはありません。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | はい | ブローカー。`subscribe` は使われません |
| `topic` | string | はい | 空でなく、`+` や `#` を含まない |
| `payload` | string | はい | UTF-8 として送信されます |
| `qos` | number | はい | 0、1、2（2 を超える値は 2 として送られます） |
| `retain` | boolean | はい | ブローカーに保持を求めます |

**結果**: string。Signal Lab が書く要約: `<topic> → <broker> · <bytes> B · qos<n>`。保持した場合は ` retained` が付きます。

**エラー**: `mqtt.topic_required`、`node.topic_wildcard`、そして [`mqtt_connect`](#mqtt_connect) のうち `mqtt.client_id_required` を除くもの。ここでは、空の `client_id` が受け入れられます。

```bash
invoke mqtt_publish_once '{"config":{"host":"127.0.0.1","port":1883,"client_id":"lab"},"topic":"lab/lamp/set","payload":"ON","qos":1,"retain":false}'
# "lab/lamp/set → 127.0.0.1:1883 · 2 B · qos1"
```

## ブロードキャスト、マルチキャスト、ディスカバリー {#broadcast}

[ブロードキャストとディスカバリー](../protocols/broadcast.md)を参照してください。

::: danger
ブロードキャストとスイープは、ネットワークセグメントのすべてのホストに届きます。自分が責任を持つネットワークでだけ送信してください。
:::

### broadcast_send {#broadcast_send}

各宛先に、データグラムを 1 回ずつ送信します。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `config` | object | はい | 下記 |

| `config` のフィールド | 型 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `mode` | string | — | `list`、`broadcast`、`multicast`、`sweep` |
| `target` | string | — | モードによる（下記） |
| `port` | number | `0` | ポート。`sweep` のみ |
| `payload` | [`Payload`](#type-payload) | — | 各データグラムが運ぶもの |
| `bind` | string | any | 送信元になるローカルの `IP:port`。空または null: `0.0.0.0:0`（すべての宛先が IPv6 のときは `[::]:0`） |
| `ttl` | number | `1` | IP の TTL、またはマルチキャストのホップ制限。1 から 255 |
| `multicast_loop` | boolean | `true` | マルチキャストがこのマシンにも戻ってくる |
| `rate`、`count`、`duration_s` | number | `0` | [`broadcast_beacon_start`](#broadcast_beacon_start) のみ |

| `mode` | `target` |
| --- | --- |
| `list` | カンマ、セミコロン、改行（空白ではなく）で区切った `IP:port` または `host:port` の項目。名前は解決され、IPv4 アドレスがあればそれが使われます |
| `broadcast` | `255.255.255.255:port`、または `.255` で終わるアドレスとそのポート |
| `multicast` | 224.0.0.0 から 239.255.255.255 までのグループとそのポート |
| `sweep` | CIDR ブロック、`192.0.2.0/24`: `port` 上の、使えるすべてのホスト。最大 1024 ホストなので、`/22` かそれより狭い範囲 |

**結果**

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `targets` | number | 宛先の数 |
| `packets`、`bytes` | number | 送信されたもの |
| `errors` | number | 送信できなかったデータグラムの数 |
| `resolved` | string[] | 最初の 8 つの宛先 |
| `summary` | string | ペイロードを 1 行にしたもの |
| `error` | `EngineError` | 最初に失敗したデータグラムが失敗した理由。失敗がなければ省略されます |

**エラー**: `broadcast.target_required`、`broadcast.not_broadcast`、`broadcast.ipv6`、`broadcast.not_multicast`、`broadcast.sweep_port`、`broadcast.cidr_invalid`、`broadcast.prefix_invalid`、`broadcast.sweep_too_large`、`node.osc_address`（OSC アドレスは `/` で始まらなければならない）、`hex.empty`、`hex.invalid`、`node.bind_invalid`、`socket.option_failed`、`transport.target_invalid`、`transport.dns`、バインドの失敗。

### broadcast_beacon_start {#broadcast_beacon_start}

同じラウンド（宛先ごとに 1 つのデータグラム）を、繰り返し送信します。カウンターは 250 ms ごとに [`broadcast://emit-stat`](events.md#event-broadcast-emit-stat) として届きます。1 つも送れないまま 32 件を超える送信が失敗すると、理由を示して停止します。*ジョブを開始します*（`beacon`、`params.mode`、`params.target`、`params.targets`、`params.rate`）。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `config` | object | はい | [`broadcast_send`](#broadcast_send) と同じで、下記の 3 つを加えたもの |

| `config` のフィールド | 型 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `rate` | number | — | 毎秒のラウンド数。0 より大きく、ラウンド数 × 宛先数が毎秒 50,000 データグラム以下 |
| `count` | number | `0` | この数のラウンドの後に停止します。0 は回数の指定なし |
| `duration_s` | number | `0` | この秒数の後に停止します。0 は停止するまで |

**結果**: [`JobInfo`](#type-jobinfo)。

**エラー**: [`broadcast_send`](#broadcast_send) のもの、`broadcast.rate_invalid`、`broadcast.rate_limit`。

### discovery_start {#discovery_start}

UDP ポートで待ち受け、何かを送ってくるすべてのピアのリストを保持し、デバイスのようにプローブに応答することもできます。ピアは 400 ms ごとに [`broadcast://peers`](events.md#event-broadcast-peers) として届きます。*ジョブを開始します*（`discovery`、`params.bind`、`params.groups`、`params.joined`）。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `config` | object | はい | 下記 |

| `config` のフィールド | 型 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `bind` | string | — | 待ち受ける `IP:port` |
| `groups` | string[] | `[]` | 参加するマルチキャストグループ（IPv4） |
| `interface` | string | any | グループに参加するローカルの IPv4 アドレス |
| `reuse` | boolean | `true` | すでに待ち受けているプログラムとポートを共有します（`SO_REUSEADDR`） |
| `respond` | boolean | `false` | 届いたものに応答します |
| `response` | [`Payload`](#type-payload) | なし | 応答。`respond` には必要です |
| `respond_delay_ms` | number | `0` | 応答する前にこの時間待ちます |
| `match_contains` | string | なし | テキストにこれを含むデータグラムにだけ応答します |

一覧されるピアは最大 512 で、それ以降は追加されません。

**結果**: [`JobInfo`](#type-jobinfo)。

**エラー**: `node.bind_invalid`、`broadcast.port_shared`（ポートが使用中で、`reuse` がオフ）、`broadcast.interface_invalid`、`broadcast.not_multicast`、`broadcast.join_failed`、`broadcast.reply_missing`、`node.osc_address`、`hex.*`、バインドの失敗。ソケットがこれ以上受信できなくなると、ジョブは `wait.receive_failed` で終了します。

```bash
invoke broadcast_send '{"config":{"mode":"list","target":"127.0.0.1:9000, 127.0.0.1:9001","payload":{"kind":"text","text":"PING"}}}' \
  | jq '{packets, errors}'
```

## 劣化 {#impairment}

クライアントとそのサーバーの間にあって、通過するものを遅延、破棄、重複、破損、順序入れ替え、スロットリングするリレーで、UDP でも TCP でも動作します。[劣化](../tools/impairment.md)を参照してください。

### netsim_start {#netsim_start}

リレーを開始します。`listen` に届いたものは `target` へ進み、応答も同じ経路で戻り、どちらもプロファイルによって劣化します。カウンターは 250 ms ごとに [`netsim://stat`](events.md#event-netsim-stat) として届きます。*ジョブを開始します*（`netsim`、`params.listen`、`params.target`、TCP では `params.protocol`）。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `config` | object | はい | 下記 |

| `config` のフィールド | 型 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `listen` | string | — | リレーが待ち受ける `IP:port`。クライアントをここに向けます |
| `target` | string | — | 実際のサーバーの `IP:port`、または `host:port`。ホスト名はリレーの開始時に 1 回解決されます |
| `profile` | [`ImpairProfile`](#type-impairprofile) | — | トラフィックに対して何をするか |
| `seed` | number | 新規 | 乱数のシード: 同じシードと同じトラフィックなら、同じものが破棄されます |
| `protocol` | string | `udp` | `udp`（データグラム）または `tcp`（ストリーム） |

**結果**: [`JobInfo`](#type-jobinfo)。

**エラー**: `node.range`（プロファイルの値が範囲外。`min`、`max`、フィールド付き）、`node.too_long`、`node.bind_invalid`、`transport.target_invalid`、`transport.dns`（見つからない宛先の名前）、バインドの失敗。ソケットがこれ以上受信できなくなると、ジョブは `wait.receive_failed` で終了します。

### netsim_set_profile {#netsim_set_profile}

実行中のリレーが、いまから別のプロファイルで劣化させます。ソケットは閉じません。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `jobId` | number | はい | リレーのジョブ |
| `profile` | [`ImpairProfile`](#type-impairprofile) | はい | 新しいプロファイル |

**結果**: null。

**エラー**: `netsim.not_running`、`node.range`、`node.too_long`。

```bash
invoke netsim_start '{"config":{"listen":"127.0.0.1:9010","target":"127.0.0.1:9000","profile":{"latency_ms":80,"jitter_ms":20,"loss":0.02}}}'
```

## ストームとスキャナー {#storm-scanner}

::: danger
ストームは、指定したとおりの強さで宛先に負荷をかけ、スキャンは範囲のすべてのポートを調べます。自分が責任を持つホストにだけ向けてください。
:::

### storm_start {#storm_start}

1 つの宛先に、UDP データグラムまたは TCP 接続による一定の負荷を送ります。カウンターは 250 ms ごとに [`storm://stat`](events.md#event-storm-stat) として届きます。*ジョブを開始します*（`storm`、`params.protocol`、`params.target`、`params.rate`）。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `config` | object | はい | 下記 |

| `config` のフィールド | 型 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `target` | string | — | `IP:port` または `host:port`。名前はジョブの開始時に 1 回解決されます |
| `protocol` | string | — | `udp`: データグラム。`tcp`: 1 単位ごとに 1 つの接続で、ペイロードを書き込んで閉じます（接続ごとに最大 500 ms かかることがあります） |
| `size` | number | — | ペイロードのバイト数。1 から 65,507 の範囲に制限されます |
| `rate` | number | — | 毎秒の単位数。10,000,000 までに制限され、スケジュールに従います。単位 *n* は、開始から *n* / `rate` 秒後が期限で、各ウェイクアップでは期限に達したもの（最大 256）が送信されます。それ以上遅れたスケジュールでは古い単位がスキップされます。0 はできるだけ速く送信します |
| `duration_s` | number | `0` | この秒数の後に停止します。0（または負の数）は停止するまで |

**結果**: [`JobInfo`](#type-jobinfo)。

**エラー**: `transport.target_invalid`、`transport.dns`、そして UDP では、ソケットをまったく開けないときのバインドまたは接続の失敗（`transport.*`）。宛先が拒否した送信は、イベントでは `errors` として数えられます。このマシンが送れなかったもの（ポート、ソケット、バッファの不足）は `local` として数えられ、宛先のエラーとして数えられることはありません。

### scan_start {#scan_start}

範囲のすべてのポートに TCP 接続を試み、開いているものを報告します。求めれば、サービスが最初に送ってくる内容も取得します。開いているポートは [`scan://open`](events.md#event-scan-open) として、進行状況は [`scan://progress`](events.md#event-scan-progress) として届きます。*ジョブを開始します*（`scan`、`params.host`、`params.from`、`params.to`）。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `config` | object | はい | 下記 |

| `config` のフィールド | 型 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `host` | string | — | ホスト名またはアドレス |
| `port_start`、`port_end` | number | — | 範囲（両端を含む）。逆に指定した場合は入れ替えられます |
| `concurrency` | number | `256` | 同時の試行数。1 から 1024 |
| `timeout_ms` | number | `600` | ポートごとの時間。50 から 10,000 |
| `grab_banner` | boolean | `false` | 接続後 400 ms 以内にサービスが送ってくる、最大 256 バイトを読み取ります |

**結果**: [`JobInfo`](#type-jobinfo)。

**エラー**: `scan.host_required`。

```bash
invoke scan_start '{"config":{"host":"127.0.0.1","port_start":8000,"port_end":9100,"grab_banner":true}}'
```

## インスペクター {#inspector}

インスペクターは、キャプチャがオンの間、ツールが送受信するものをフレームとして記録します。サーバーでは、すべてのページとスクリプトに対して 1 つのインスペクターがあります。[インスペクター](../tools/inspector.md)を参照してください。

### inspect_set_enabled {#inspect_set_enabled}

キャプチャをオンまたはオフにします。オフの間は何も記録されません。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `enabled` | boolean | はい | オン（`true`）またはオフ |

**結果**: [`CaptureStats`](#type-frame)。

### inspect_stats {#inspect_stats}

キャプチャのカウンター。引数はありません。

**結果**: [`CaptureStats`](#type-frame)。

### inspect_snapshot {#inspect_snapshot}

最新のフレームを、古いものから順に返します。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `limit` | number | はい | 件数。1 から 8192 |

**結果**: [`Frame`](#type-frame)`[]`。バイト列は含みません（[`inspect_payload`](#inspect_payload) を参照）。

### inspect_clear {#inspect_clear}

キャプチャとそのカウンターを空にします。引数はありません。

**結果**: [`CaptureStats`](#type-frame)。

### inspect_export {#inspect_export}

保持しているすべてのフレームを、データフォルダーの `capture-<ms>.jsonl` または `capture-<ms>.txt` に書き出します。`jsonl` では、各行がフレームで、保持しているバイト列が `data` に base64 で入ります。`txt` は読むためのもので、各フレームの hex ダンプが付きます。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `format` | string | はい | `txt`。それ以外は `jsonl` で書き出します |

**結果**: string。書き込んだパス。

**エラー**: `inspect.empty`、`file.io`。

### inspect_payload {#inspect_payload}

フレームが保持しているバイト列。バッチが運んだ 256 バイトのプレビューより先の部分です。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `seq` | number | はい | フレームの番号 |

**結果**: `{ "seq", "bytes", "kept", "dump", "hex" }`。`bytes` はフレームのサイズ、`kept` はそのうち保持されている数（最大 256 KiB）、`dump` はすべての行を `offset  hex  |ascii|` で、`hex` はリプレイが送信するプレーンな hex です。

**エラー**: `inspect.frame_gone`（新しいフレームに置き換えられた）、`inspect.no_payload`（サイズしか記録されていない）。

```bash
invoke inspect_set_enabled '{"enabled":true}'
invoke inspect_snapshot '{"limit":20}' | jq '.[] | {seq, proto, dir, summary}'
```

## シグナルライブラリ {#signals}

ライブラリは、データフォルダーの `signals.json` です。保存するだけのもので、シグナルは、そのトランスポートのコマンド（`osc_send`、`broadcast_send`、`http_request`、`mqtt_publish`、`mqtt_publish_once`）で送信されます。[シグナル](../tools/signals.md)と[ファイル](../reference/files.md#signals-json)を参照してください。

### signals_load {#signals_load}

ライブラリを読み取ります。ファイルが存在しない場合は、先に初期セットが書き込まれます。引数はありません。

**結果**: `{ "path": string, "library": library, "seeded": boolean, "stamp": string }`。`seeded` は、初期セットがいま書き込まれたときに true です。`stamp` は、ファイルが保持していた内容を表し、次の保存の `base` に使います。ライブラリは `{ "version", "signals": [...], "folders": [...] }` で、`version` は 2（バージョン 1 のファイルはそのまま返ります）、`folders` はなければ省略されます。各シグナルは、`id`、`name`、`group`（そのフォルダー、`"A/B"`。なければ空）、`note`、`body` を持ちます。

**エラー**: `signals.json_invalid`（`path`、`line`、`column` 付き。ファイルは置き換えられません）、`file.io`。

### signals_save {#signals_save}

ライブラリファイル全体を、同じフォルダーの一時ファイルを経由して置き換えます。存在していて、ライブラリとして読めないファイルは、そのまま残されます。`base` を付けた場合は、読み込んだあとに変更されたファイル（サーバーの別のページ、またはエディターで編集した人によるもの）もそのまま残され、保存は拒否されます。そのため、その変更が、それを知らない一覧の下で失われることはありません。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `library` | object | はい | `signals_load` が返すとおりの `{ version, signals, folders }` |
| `base` | string | いいえ | ライブラリを読み込んだときの `stamp`（または、前回の保存が返したもの）。ファイルがなかった場合は `""` |

**結果**: string。書き込んだパス。`base` を付けた場合は `{ "path", "stamp" }`。ファイルがいま保持している内容の stamp で、次の保存の `base` に使います。

**エラー**: `signals.json_invalid`（ディスク上のファイルが読めない。`path`、`line`、`column` 付き。何も書き込まれません）、`file.changed`（`path` 付き。ファイルが `base` の内容ではなくなっている。何も書き込まれません。読み込み直してください）、`signals.encode`、`file.io`。

シグナルの `body` は、その `transport` によって決まります。

| `transport` | フィールド |
| --- | --- |
| `osc` | `target`、`address`、`args`（[`OscArg`](#type-oscarg)`[]`） |
| `udp` | `target`、`payload`: `{ "kind": "text", "text" }` または `{ "kind": "hex", "hex" }` |
| `http` | `request`（[`HttpRequest`](#type-httprequest)） |
| `mqtt` | `broker`（`host:port`）、`topic`、`payload`、`qos`、`retain` |

```bash
invoke signals_load | jq '.library.signals[] | {name, transport: .body.transport}'
```

## エミュレーター {#emulators}

エミュレーターは、Signal Lab が相手側を演じるものです。HTTP API、OSC・UDP・TCP のデバイス、MQTT ブローカーなどです。そのドキュメント（`name`、`bind`、`protocol`、プロトコルのルール、任意の `outage`）は、[エミュレーター](../tools/emulators.md)で説明されています。ライブラリは、データフォルダーの `emulators.json` です。

### emulators_load {#emulators_load}

エミュレーターライブラリを読み取ります。ファイルが存在しない場合は、先に初期セットが書き込まれます。引数はありません。

**結果**: `{ "path", "library": { "version": 1, "emulators": [{ "id", "note", "emulator" }] }, "seeded", "stamp" }`。`stamp` は、ファイルが保持していた内容を表し、次の保存の `base` に使います。

**エラー**: `emulators.json_invalid`（`path`、`line`、`column` 付き。置き換えられません）、`file.io`。

### emulators_save {#emulators_save}

エミュレーターライブラリ全体を、一時ファイルを経由して置き換えます。存在していて、ライブラリとして読めないファイルは、そのまま残されます。`base` を付けた場合は、読み込んだあとに変更されたファイルも、[`signals_save`](#signals_save) と同じようにそのまま残されます。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `library` | object | はい | `emulators_load` が返すとおりの `{ version, emulators }` |
| `base` | string | いいえ | ライブラリを読み込んだときの `stamp`（または、前回の保存が返したもの）。ファイルがなかった場合は `""` |

**結果**: string。書き込んだパス。`base` を付けた場合は `{ "path", "stamp" }`。

**エラー**: `emulators.json_invalid`（ディスク上のファイルが読めない。何も書き込まれません）、`file.changed`（ファイルが `base` の内容ではなくなっている。何も書き込まれません）、`emulators.encode`、`file.io`。

### emulator_check {#emulator_check}

エミュレーターが開始できるか。[`emulator_start`](#emulator_start) がバインドの前にチェックするすべてです。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `emulator` | object | はい | エミュレーターのドキュメント |
| `params` | object of strings | いいえ | テンプレートがパラメーターとして読む値 |

**結果**: 開始できる場合は null。

**エラー**: `emulator.*`。フィールドの欠落、範囲外、長すぎる、不正な形式に対する `node.*`（`node.required`、`node.range`、`node.too_long`、`node.bind_invalid`、`node.target_invalid`、`node.method_invalid`…）。`param.unknown`、`template.*`、`osc.pattern_*`、`regex.invalid`、`hex.invalid`。問題がルール、保持された値、レスポンスのいずれかにある場合、それぞれの `params` に `rule`、`retained`、`response` が入ります。

### emulator_start {#emulator_start}

エミュレーターを、独立したジョブとして開始します。コマンドが戻るときには、そのソケットは開いています。受け取って答えたものは、何かが変わったとき 200 ms ごとに [`emulator://activity`](events.md#event-emulator-activity) として届きます。*ジョブを開始します*（`emulator`、`params.name`、`params.protocol`、`params.local`、`source` が指定されている場合は `params.source`）。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `emulator` | object | はい | エミュレーターのドキュメント |
| `params` | object of strings | いいえ | テンプレートがパラメーターとして読む値 |
| `seed` | number | いいえ | そのシード。0 から 9007199254740991。既定値: 新規 |
| `source` | string | いいえ | 元になったライブラリのエントリ。ジョブに `params.source` として保持されます |

**結果**: [`JobInfo`](#type-jobinfo)。

**エラー**: [`emulator_check`](#emulator_check) のもの、`seed.range`、`transport.address_in_use` とそのほかのバインドの失敗。

### emulator_exchanges {#emulator_exchanges}

実行中のエミュレーターが受け取って答えたもの。最後の 500 件のやり取りが保持されます。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `jobId` | number | はい | エミュレーターのジョブ |
| `after` | number | いいえ | この番号より大きいやり取りだけ。既定値は 0 |
| `limit` | number | いいえ | 最大でこの件数。1 から 500。既定値は 500 |

**結果**

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | ジョブ |
| `name`、`protocol`、`local` | string | エミュレーター、そのプロトコル、待ち受けるアドレス |
| `counts` | object | `total`、`unmatched`（どのルールにも該当しなかった）、`failed`、`down`（ダウン中に届いた: そのダウンのスケジュール、または [`emulator_down`](#emulator_down)）、`hits`（ルールごと）、`missed`（MQTT: 遅れすぎたクライアントが受け取れなかったメッセージ。0 の間は省略）、`refused`（HTTP、TCP、MQTT: 上限を超えて、届いた時点で閉じられた接続。0 の間は省略） |
| `forced` | string | ダウンさせている間の `unavailable`、`reset`、`timeout`。そうでなければ省略されます |
| `exchanges` | object[] | それぞれ: `seq`、`ts`、`from`、`request`、`rule`（1 から始まる。どのルールにも該当しなければ省略）、`reply`、`status`、`fault`、`ms`、`error`、`frame`、`down`、`data`（テンプレートが読むとおりのリクエスト） |

**エラー**: `emulator.not_running`。

### emulator_down {#emulator_down}

実行中のエミュレーターを、ダウンのスケジュールがどうであれ、復帰させるまでダウンさせます。または、復帰させます。ダウン中、HTTP エミュレーターは各リクエストに `fault` で応答し、TCP デバイスと MQTT ブローカーは接続を切って新しい接続を拒否し、OSC と UDP のデバイスは何も応答しません。

| 引数 | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `jobId` | number | はい | エミュレーターのジョブ |
| `down` | boolean | はい | ダウン（`true`）または復帰 |
| `fault` | string | いいえ | HTTP リクエストへの応答: `unavailable`（503。`Retry-After` なし: いつ戻るかは分からない）、`reset`（接続が閉じる）、`timeout`（応答なし）。既定値は `unavailable` |

**結果**: null。

**エラー**: `emulator.not_running`。

```bash
invoke emulator_exchanges '{"jobId":5,"after":0}' | jq '.counts, (.exchanges[] | {request, rule, status})'
```

## 共通の型 {#types}

### JobInfo {#type-jobinfo}

ジョブを開始するコマンドが返すもので、[`jobs_list`](#jobs_list) が一覧するものです。`id`、`kind`、`label`（ログ向けの英語）、`params`（ラベルを構成する値。なければ省略）、`started_ms`。[ジョブ](index.md#jobs)を参照してください。

### OscArg {#type-oscarg}

1 つの OSC 引数で、その型と値です。

| `type` | `value` | OSC タグ |
| --- | --- | --- |
| `int` | 32 ビット整数 | `i` |
| `float` | 数値。32 ビット浮動小数点数として送られます | `f` |
| `str` | 文字列 | `s` |
| `long` | 64 ビット整数 | `h` |
| `double` | 数値。64 ビット | `d` |
| `bool` | `true` または `false` | `T` または `F` |
| `blob` | バイトの配列、`[222, 173]` | `b` |
| `nil` | なし: `{ "type": "nil" }` | `N` |
| `time` | タイムタグ。64 ビットを 10 進数のテキストで表したもの。`"1"` は「今すぐ」 | `t` |
| `char` | 1 文字、`"A"` | `c` |
| `color` | `[red, green, blue, alpha]` | `r` |
| `midi` | `[port, status, data1, data2]` | `m` |
| `impulse` | なし | `I` |
| `array` | OscArg の配列 | `[` … `]` |

最後の 6 つは、受信したメッセージが運ぶものです（OSC モニターの `osc://message`）。コマンドから送信することもできます。

### HttpRequest {#type-httprequest}

| フィールド | 型 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `method` | string | — | `GET`、`POST`… |
| `url` | string | — | `http://` または `https://` |
| `headers` | `[name, value][]` | `[]` | リクエストヘッダー |
| `body` | string or null | null | ボディ |
| `timeout_ms` | number | `10000` | ボディを含む通信全体に対する時間: 1 から 120,000。範囲外ではコマンドが `node.range`（フィールド `timeout_ms`）で失敗します |
| `auth` | object | なし | `{ "scheme": "basic", "username", "password" }`、`{ "scheme": "digest", "username", "password" }`、`{ "scheme": "bearer", "token" }` |

リダイレクトは最大 10 回まで追従します。あるホストのために入力した資格情報や Cookie が、別のホストに渡ることはありません。Digest のリクエストは、サーバーの 401 チャレンジに答えて、もう一度送信します。

### HttpResponse {#type-httpresponse}

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `ok` | boolean | 2xx のステータスで、ボディ全体が届いた |
| `status`、`status_text` | number, string | ステータス。レスポンスがないときは 0 と空 |
| `latency_ms` | number | ボディ全体が届くまでの時間 |
| `headers` | `[name, value][]` | レスポンスヘッダー |
| `body` | string | テキストとしてのボディ。最大 256 KiB |
| `body_bytes` | number | ボディの完全なサイズ |
| `truncated` | boolean | `body` が 256 KiB で切り捨てられた |
| `error` | string or null | レスポンスがなかった理由、またはボディが途中で止まった理由（その場合、`status`、`headers`、それまでのボディもあります）。原因のすべての層を含みます |
| `cause` | string or null | どの種類の失敗か: `refused`、`timeout`、`dns`、`unreachable`、`reset`、`address_in_use`、`address_unavailable`、`denied`、`tls`、`target_invalid`、`exhausted`（このマシンのソケットやローカルポートが尽きた）、`failed`。`transport.*` のコードと同じです |
| `digest` | object | 401 に出会った Digest のリクエストのみ。そうでなければ省略されます。`challenged`: チャレンジに答えてリクエストを再送した。`error`: できなかった理由。`EngineError`（`http.digest_not_offered`、`http.digest_unsupported`、`http.digest_invalid`、`http.digest_other_origin`）または null |

### Payload {#type-payload}

ブロードキャストやディスカバリーのデータグラムが運ぶもの: `{ "kind": "osc", "address", "args" }`、`{ "kind": "text", "text" }`（そのまま送られ、終端のゼロはなし）、または `{ "kind": "hex", "hex" }`（`de ad be ef`、`deadbeef`、`0xDE,0xAD`。16 進数以外の文字は無視されます）。

### MqttConfig {#type-mqttconfig}

| フィールド | 型 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `host` | string | — | ブローカーの名前またはアドレス |
| `port` | number | — | 通常は 1883 |
| `client_id` | string | — | 空でないこと。同じ ID を持つ別の接続は、ブローカーに追い出されます |
| `username`、`password` | string | 空 | `username` は空でないときに送られ、`password` は `username` と一緒にのみ送られます |
| `keep_alive_s` | number | `60` | ping はその半分の間隔で送られます。0: なし |
| `clean_session` | boolean | `true` | CONNECT のフラグ |
| `will` | object or null | null | `{ topic, payload, qos, retain }`。接続が失われたときにブローカーがパブリッシュします |
| `subscribe` | `{ filter, qos }[]` | `[]` | 接続が確立するとすぐにサブスクライブします |

### WsConfig {#type-wsconfig}

| フィールド | 型 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `url` | string | — | `ws://` または `wss://`（`wss://` は、システムが HTTPS で信頼するものを信頼します） |
| `headers` | `[name, value][]` | `[]` | アップグレードリクエストと一緒に送られます |
| `protocols` | string[] | `[]` | 提示するサブプロトコル（優先順） |
| `timeout_ms` | number | `10000` | 接続、TLS、アップグレードを合わせた時間 |

メッセージは、どちら向きも最大 16 MiB です。

### ImpairProfile {#type-impairprofile}

すべてのフィールドは任意で、省略したものは何もしません。確率は 0 から 1 です。

| フィールド | 範囲 | 意味 | UDP | TCP |
| --- | --- | --- | --- | --- |
| `name` | 最大 60 文字 | タイムラインとレポートのためのラベル | はい | はい |
| `latency_ms` | 0 から 60,000 | すべてに加える遅延 | はい | はい |
| `jitter_ms` | 0 から 60,000 | 毎回引かれる、この値までの追加の遅延 | はい | はい |
| `loss` | 0 から 1 | データグラムが破棄される | はい | — |
| `duplicate` | 0 から 1 | データグラムが 2 回送られる | はい | — |
| `corrupt` | 0 から 1 | データグラムの 1 ビットが反転する | はい | — |
| `reorder` | 0 から 1 | データグラムが保留され、後続のものに追い越される | はい | — |
| `rate_kbps` | 0、または 8 から 10,000,000 | 帯域制限（キロビット毎秒）。0: 制限なし | はい | はい |
| `burst_start` | 0 から 1 | データグラムがロスのバーストを開始する | はい | — |
| `burst_length` | 1 から 1000 | バーストが平均で続くデータグラム数（`burst_start` とともに必要） | はい | — |
| `offline` | `true` または `false` | 何も通さない | はい | はい |
| `reset` | 0 から 1 | ストリームのチャンクがその接続をリセットする | — | はい |
| `stall` | 0 から 1 | ストリームのチャンクがその接続をハーフオープンのままにする | — | はい |

### Frame と CaptureStats {#type-frame}

`Frame` は、キャプチャされたパケット、リクエスト、メッセージ 1 つです。

| フィールド | 意味 |
| --- | --- |
| `seq` | その番号。増加していきます |
| `ts` | 時刻。1970 年からのミリ秒 |
| `proto` | `osc`、`udp`、`tcp`、`http`、`mqtt`、`ws`… |
| `dir` | `tx`（送信）または `rx`（受信） |
| `source` | キャプチャしたツール: `osc-monitor`、`broadcast`、`netsim`… |
| `job_id` | そのジョブ。なければ null |
| `local`、`remote` | アドレス: こちら側と相手側の `IP:port`（HTTP、WebSocket、MQTT では URL またはブローカー）。リレーされたフレームの `local` はリレーが待ち受けるアドレス、`remote` はそのフレームが向かっていた先で、どちらの区間かは `verdict` の末尾に付きます（`· client→target`、`· target→client`） |
| `bytes` | そのサイズ |
| `summary` | 1 行の要約 |
| `detail` | 複数行にわたるデコード。なければ null |
| `hex` | 最初の 256 バイトの hex ダンプ。なければ null |
| `verdict` | どうなったか（`dropped`、`sampled`、ステータス）。なければ null |
| `kept` | `bytes` のうち、保持されている数（最大 256 KiB）。サイズだけが記録された場合は 0 |
| `publish` | MQTT のパブリッシュのみ: `{ broker, topic, qos, retain, text }`。ブローカーは `host:port` で、最後の値は、保持されたバイト列（メッセージのペイロード）が UTF-8 テキストかどうかです。ほかのすべてのフレームにはありません |

`CaptureStats`: `enabled`、`total`（記録されたフレーム）、`bytes`、`skipped`（記録されたがインターフェイスに送られなかったもの）、`buffered`（保持されているフレーム）、`capacity`（8192）、`held`（保持されているペイロードのバイト数）、`held_limit`（64 MiB）。どちらかの上限を超えると、最も古いフレームから譲られます。

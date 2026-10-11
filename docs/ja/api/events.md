---
title: イベント
description: Signal Lab サーバーのエンジンイベントの WebSocket と、すべてのチャンネルのペイロードおよび送信されるタイミングを説明します。
---

# イベント

ジョブの実行中に起こることはすべて（実行のステップ、モニターのメッセージ、バーストの数値、インスペクターのフレーム、ジョブの終了）、イベントとして送られます。ブラウザーでは、サーバーのページが 1 つの WebSocket、`/api/events` でそれらを受け取ります。スクリプトも同じソケットで待ち受けられます。デスクトップアプリは、同じ名前とペイロードのイベントを、アプリの内部で受け取ります。

## サブスクライブ {#subscribe}

サーバーの `/api/events` に WebSocket を開きます。

```bash
websocat -H "Authorization: Bearer $TOKEN" ws://127.0.0.1:1430/api/events
```

- **認証**は API のほかの部分と同じです。`Authorization: Bearer` のトークン、またはブラウザーのセッション Cookie です。それがないと、アップグレードは `401` の `auth.required` で拒否されます。
- **Origin**: `Origin` ヘッダーを送るクライアントは、サーバー自身のもの（ホストとポートが `Host` と等しい）を送らなければならず、そうでなければアップグレードは `403` の `auth.origin` で拒否されます。ブラウザー以外のほとんどの WebSocket ライブラリは、何も送りません。
- **すべてのイベントがすべてのクライアントに届きます。** サブスクライブするものはありません。各ソケットは、誰が開始したかにかかわらず、すべてのジョブのすべてのイベントを受け取ります。必要なものは、`event` と、ペイロードの `job_id` で選んでください。
- **受信専用です。** サーバーは、クローズ以外はクライアントが送るものを無視します。64 KiB を超えるメッセージは、ソケットを閉じます。
- **キープアライブ。** サーバーが 20 秒ごとに ping を送るので、静かなソケットもプロキシ越しに開いたままです。サーバーが停止するときは、すべてのソケットを閉じます。
- **何も再送されません。** クライアントが接続していない間に送られたイベントは、そのクライアントには失われます。再接続したクライアントは、コマンド（`jobs_list`、`inspect_snapshot`、`emulator_exchanges` など）で現在の状態を読んでください。
- **遅れた場合。** 1 つのソケットに対して、最大 4096 件のイベントが待機します。それ以上遅れたクライアントは、取りこぼした数とともに [`server://lagged`](#event-server-lagged) を受け取ります。

## メッセージの形式 {#format}

各イベントは、JSON オブジェクト 1 つを持つテキストメッセージ 1 つです。

```json
{ "event": "scan://open", "payload": { "job_id": 9, "ts": 1759600000123, "port": 8080, "banner": null } }
```

| フィールド | 内容 |
| --- | --- |
| `event` | チャンネル（下記） |
| `payload` | イベントの値。形はチャンネルによって異なります |

時刻（`ts`、ピアの `first_ms` と `last_ms`）は 1970 年からのミリ秒で、レイテンシやそのほかの時間（`*_latency_ms`、`p50_ms`…、`ms`）はミリ秒です。ペイロード内のエラーは [`EngineError`](index.md#errors) オブジェクトで、そのコードは[エラーメッセージ](../reference/errors.md)に一覧されています。

## チャンネル {#channels}

| チャンネル | 送信元 | 送信されるとき |
| --- | --- | --- |
| [`experiment://step`](#event-experiment-step) | 実行 | ステップが開始、成功、失敗、再試行、繰り返し、または負荷を報告したとき |
| [`experiment://ended`](#event-experiment-ended) | 実行 | 実行が自力で終了したとき、1 回 |
| [`job://ended`](#event-job-ended) | すべてのジョブ | ジョブが自力で終了または失敗したとき、1 回 |
| [`osc://message`](#event-osc-message) | OSC モニター | すべてのパケット。大量に届くときは 1 秒に 200 件にサンプリング |
| [`osc://monitor-stat`](#event-osc-monitor-stat) | OSC モニター | 250 ms ごと、および終了時 |
| [`osc://gen-tick`](#event-osc-gen-tick) | OSC ジェネレーター | すべてのメッセージ。毎秒 60 メッセージを超えると 1 秒に 30〜45 回 |
| [`http://burst-progress`](#event-http-burst-progress) | HTTP バースト | 100 ms ごと、および終了時 |
| [`ws://state`](#event-ws-state) | WebSocket 接続 | 接続時、クローズ時 |
| [`ws://messages`](#event-ws-messages) | WebSocket 接続 | 新しいものがあれば 100 ms ごと |
| [`mqtt://state`](#event-mqtt-state) | MQTT 接続 | 接続時、サブスクライブ時、クローズ時 |
| [`mqtt://messages`](#event-mqtt-messages) | MQTT 接続 | 新しいものがあれば 100 ms ごと |
| [`mqtt://ack`](#event-mqtt-ack) | MQTT 接続 | QoS 1/2 のパブリッシュが完了したとき、サブスクライブ解除に応答があったとき |
| [`broadcast://emit-stat`](#event-broadcast-emit-stat) | ビーコン | 250 ms ごと、および終了時 |
| [`broadcast://peers`](#event-broadcast-peers) | ディスカバリーリスナー | 400 ms ごと |
| [`netsim://stat`](#event-netsim-stat) | 劣化リレー | 250 ms ごと |
| [`storm://stat`](#event-storm-stat) | ストーム | 250 ms ごと、および終了時 |
| [`scan://open`](#event-scan-open) | スキャナー | 開いているポートごと |
| [`scan://progress`](#event-scan-progress) | スキャナー | 範囲の約 1% ごと、および終了時 |
| [`emulator://activity`](#event-emulator-activity) | エミュレーターのジョブ | 新しいものがあれば 200 ms ごと |
| [`inspect://batch`](#event-inspect-batch) | インスペクター | キャプチャ中、新しいフレームがあれば 120 ms ごと、静かなときは約 1 秒に 1 回 |
| [`server://lagged`](#event-server-lagged) | サーバー | クライアントが遅れたとき |

### `experiment://step` {#event-experiment-step}

実行の 1 ステップです。ノードが開始した、成功した、失敗した、再試行を待っている、繰り返している、または負荷の進行状況を報告している、のいずれかです。`/api/run` で開始した実行は、そのレスポンスで同じステップを送ります（[実行](run.md)を参照）。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | 実行のジョブ |
| `ts` | number | 時刻 |
| `node_id` | string | ノード |
| `state` | string | `running`、`passed`、`failed`、`retry`（試行が失敗し、一時停止の後にステップがもう一度実行される）、`repeating`（繰り返すアクションの進行状況。1 秒に 1 回まで）、`load`（負荷の進行状況。1 秒に 1 回まで） |
| `detail` | string | 何が起きたか（英語）。`running` と `failed` では空です（`error` を参照） |
| `message_key` | string or null | それに対するインターフェイスの文言を、その辞書のキーで示したもの |
| `message_params` | object or null | `message_key` が指す値 |
| `vars` | object | ステップが書き込んだ変数。なければ省略されます |
| `error` | `EngineError` | 失敗した、または試行が失敗した理由（`retry`）。そうでなければ省略されます |
| `frame` | number | 待機（または送信で期待した応答）が一致したメッセージのインスペクターのフレーム。キャプチャがオンだったときのみで、そうでなければ省略されます |
| `load` | object | 負荷が測定したもの（しきい値の判定を含む）。負荷のステップの最後のイベントで、成功でも失敗でも付きます。そうでなければ省略されます。[負荷](../experiments/load.md)を参照 |
| `omitted` | number | このステップの直前に省略されたステップの数。なければ省略されます。実行は、最初の 5,000 ステップを 1 つずつ、その後は 1 秒に 1 つ（とすべての失敗）、終了時に最後の 500 ステップを報告するので、1,000 回の反復を持つループも読みやすいままです |

実行の [[ui:exp.node.end]] ノードは、最初のブランチが到達したときに `running` と表示し、すべてのブランチが失敗なしで終了すると `passed` と表示します。シークレットの値は、すべてのフィールドでマスクされます。

### `experiment://ended` {#event-experiment-ended}

実行が自力で終了しました。成功、失敗、または時間切れです。[`job://ended`](#event-job-ended) の同じペイロードの直後に送られます。`job_stop` や [[ui:app.stopAll]] で停止した実行は、どちらも送らず、レポートも保存しません。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | 実行のジョブ |
| `kind` | string | `experiment` |
| `seed` | number | 実行に使われたシード |
| `profile` | string or null | そのプロファイル |
| `overridden` | boolean | 一部のパラメーター値が [[ui:exp.runWith]] または `overrides` から来た |
| `error` | `EngineError` or null | 実行の最初の失敗。成功した場合は null |
| `report_path` | string or null | データフォルダーの `runs/` にあるレポート |
| `report_error` | `EngineError` or null | レポートを書き込めなかった理由 |

### `job://ended` {#event-job-ended}

ジョブが自力で終了、または失敗しました。`job_stop` や `jobs_stop_all` で停止したジョブは、これを送りません。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | ジョブ |
| `kind` | string | `osc-monitor`、`osc-gen`、`http-burst`、`netsim`、`storm`、`scan`、`beacon`、`discovery`、`mqtt`、`websocket`、`emulator`、`experiment` |
| `error` | `EngineError` or null | 何かがうまくいかなかったときの、終了の理由 |

実行の `job://ended` には、[`experiment://ended`](#event-experiment-ended) のフィールドも含まれます。それぞれの種類が何で終了するかは、次のとおりです。

| `kind` | 終了するとき | `error` |
| --- | --- | --- |
| `osc-monitor` | ソケットがこれ以上受信できなくなったとき | `wait.receive_failed` |
| `osc-gen` | 継続時間が終わったとき、または送信が失敗したとき | null、または `transport.*` |
| `http-burst` | 合計または継続時間に達したとき | null |
| `storm` | 継続時間が終わったとき | null |
| `scan` | 範囲のすべてのポートを試したとき | null |
| `beacon` | ラウンド数または継続時間が終わったとき、または 1 つも送れないまま 32 件を超える送信が失敗したとき | null、または `transport.*` |
| `discovery` | ソケットがこれ以上受信できなくなったとき | `wait.receive_failed` |
| `mqtt` | ブローカーが接続を閉じた、または接続が失われたとき | `transport.*`（ブローカーが閉じたときは `transport.reset`）、または `mqtt.protocol` |
| `websocket` | 接続が閉じたとき | null、または失われた理由 |
| `netsim` | リレーがこれ以上動作できなくなったとき | その理由 |
| `emulator` | そのソケットが失敗したとき | その理由 |
| `experiment` | 実行が終了したとき | 実行の失敗、または null |

### `osc://message` {#event-osc-message}

OSC モニターが受信した、デコード済みの UDP パケット 1 つです。デコードされたパケットは、大量に届いてもインターフェイスへの経路があふれないように、最大で 1 秒に 200 件のイベントにサンプリングされます（[`osc://monitor-stat`](#event-osc-monitor-stat) が、いくつ受信し、いくつ転送しなかったかを示します）。不正な形式のパケットは、常に送られます。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | モニターのジョブ |
| `ts` | number | 到着した時刻 |
| `from` | string | 送信元の `IP:port` |
| `bytes` | number | パケットのサイズ |
| `messages` | object[] | パケット内の各メッセージ（バンドルには複数あります）: `address` と `args`（[`OscArg`](commands.md#type-oscarg)`[]`） |
| `error` | `EngineError` or null | パケットがデコードできなかったときの `osc.packet_malformed`（その場合 `messages` は空です） |

### `osc://monitor-stat` {#event-osc-monitor-stat}

OSC モニターの累計です。250 ms ごと、および終了時にもう一度送られるので、サンプリングされた一覧が、どれだけのトラフィックを表しているかが分かります。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | モニターのジョブ |
| `ts` | number | 時刻 |
| `packets` | number | 受信したパケット数（デコードできたかどうかを問いません） |
| `bytes` | number | 受信したバイト数 |
| `not_shown` | number | 受信したが [`osc://message`](#event-osc-message) として転送されなかったパケット数（レートのサンプリングが保留したもの） |
| `malformed` | number | OSC としてデコードできなかったパケット数 |

### `osc://gen-tick` {#event-osc-gen-tick}

OSC ジェネレーターの進行状況です。毎秒 60 メッセージ未満ではすべてのメッセージごとに、それを超えると n 件ごとに送られます。n はレートを 30 で割って切り捨てた値で、1 秒に 30〜45 回になります。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | ジェネレーターのジョブ |
| `ts` | number | 時刻 |
| `value` | number | いま送った値（整数または 32 ビット浮動小数点数に丸める前） |
| `sent` | number | これまでに送ったメッセージ数 |

### `http://burst-progress` {#event-http-burst-progress}

HTTP バーストの数値です。実行中は 100 ms ごとに、自力で終了したときは `done: true` を付けてもう一度送られます。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | バーストのジョブ |
| `ts` | number | 時刻 |
| `sent` | number | これまでに応答があった、または失敗したリクエスト数 |
| `ok` | number | そのうち、2xx のステータスとボディ全体が返ったもの |
| `failed` | number | そのうち、それ以外のステータス、途中で切れたボディ、応答なしのもの。このマシンが送れなかったものは含みません |
| `local` | number | そのうち、このマシンが送れなかったリクエスト: ソケットやローカルポートが尽きたもの（`transport.exhausted`）。`failed` にも、レイテンシにも含まれません |
| `missed` | number | レート指定のバーストで、空いているワーカーを待ちすぎてスキップされたリクエスト |
| `rps` | number | 直近の 100 ms での毎秒のリクエスト数。最後のイベントでは、バースト全体での値 |
| `last_latency_ms`、`min_latency_ms`、`max_latency_ms`、`avg_latency_ms` | number | これまでのレイテンシ |
| `p50_ms`、`p90_ms`、`p95_ms`、`p99_ms` | number | 失敗も含む、これまでのすべてのリクエストのパーセンタイル（誤差は 0.5% 以内） |
| `done` | boolean | バーストの最後のイベント |

### `ws://state` {#event-ws-state}

`ws_connect` で開いた WebSocket 接続が、接続した、または閉じました。ジョブが停止された接続は、`closed` を送りません。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | 接続のジョブ |
| `ts` | number | 時刻 |
| `state` | string | `connected` または `closed` |
| `handshake` | object | `url`、`peer`、`local`、`protocol`（サーバーが選んだサブプロトコル。なければ null）、`ms`（接続とアップグレードにかかった時間） |
| `closed` | object or null | `closed` のとき: `code`、`reason`、`by`（`client`、`server`、`lost`）、`error` |

### `ws://messages` {#event-ws-messages}

WebSocket 接続が、前回のイベント以降に送受信したものです。何かあれば 100 ms ごとに送られます。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | 接続のジョブ |
| `ts` | number | 時刻 |
| `messages` | object[] | 順番どおり: `ts`、`dir`（`rx` は受信、`tx` は送信）、`kind`（`text` または `binary`）、`text`（最初の 64 KiB を UTF-8 として。バイナリメッセージでは最初の 4096 バイト。UTF-8 でないバイトは `�` になります）、`hex`（バイナリメッセージの最初の 4096 バイトを hex で。そうでなければ null）、`bytes`（完全なサイズ）、`truncated`（表示された以上の内容がある: テキストでは 64 KiB を超える、バイナリでは 4096 バイトを超える） |
| `dropped` | number | 2000 件を超えた、または `text` と `hex` が 1 MiB を超えたために、このイベントから省かれたメッセージの数。古いものから省かれます |
| `received`、`sent` | number | これまでにその接続で受信、送信したメッセージ数（表示されたかどうかを問いません） |

### `mqtt://state` {#event-mqtt-state}

MQTT 接続の状態が変わりました。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | 接続のジョブ |
| `ts` | number | 時刻 |
| `state` | string | `connected`。サブスクライブへの各応答の後の `subscribed`。接続が終了したときの `closed`（ジョブが停止されたときは送られません） |
| `broker` | string | `host:port` |
| `error` | `EngineError` or null | `closed` の接続が終了した理由（ブローカーが閉じたときは `transport.reset`）。そうでなければ null |
| `grants` | object[] | `subscribed` のとき: 要求した各フィルターについて、`filter`、`qos`（付与されたもの）、`accepted`。それ以外は空です |

### `mqtt://messages` {#event-mqtt-messages}

MQTT 接続が、前回のイベント以降に受信したものです。何かあれば 100 ms ごとに送られます。再配信された QoS 2 のメッセージは、1 回だけ表示されます。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | 接続のジョブ |
| `ts` | number | 時刻 |
| `messages` | object[] | トピックごとに 1 つ、そのトピックに最後に届いたもの: `ts`、`topic`、`payload`（最初の 4096 バイトを UTF-8 として。そうでないバイトは `�` になります）、`bytes`（ペイロード全体のサイズ）、`truncated`（`payload` は冒頭だけ）、`qos`、`retain`、`dup`、`count`（前回のイベント以降にそのトピックに届いたメッセージ数。今回のものを含みます） |
| `dropped` | number | 常に 0: イベントが 4000 を超えるトピックまたは 2 MiB を超えるペイロードを運ぶことになるときは、それが送り出されるまで接続は読み取りを止め、ブローカーが待ちます |

### `mqtt://ack` {#event-mqtt-ack}

接続が要求したことを、ブローカーが完了しました。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | 接続のジョブ |
| `ts` | number | 時刻 |
| `kind` | string | `published`（QoS 1 または 2 のパブリッシュが完了した）または `unsubscribed` |
| `packet_id` | number | MQTT のパケット ID |
| `topic` | string or null | パブリッシュされたトピック。`unsubscribed` では null |

### `broadcast://emit-stat` {#event-broadcast-emit-stat}

ビーコンのカウンターです。250 ms ごとに、そして自力で終了したときに `pps` を 0 にしてもう一度送られます。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | ビーコンのジョブ |
| `ts` | number | 時刻 |
| `targets` | number | 各ラウンドの宛先の数 |
| `rounds` | number | 送信したラウンド数 |
| `packets`、`bytes` | number | 送信したデータグラム数とバイト数 |
| `errors` | number | 失敗した送信の数 |
| `local` | number | このマシンが送れなかった送信（バッファやソケットの不足）で、`errors` には含まれません。0 の間は省略されます |
| `pps` | number | 直近の 250 ms での毎秒のデータグラム数 |

### `broadcast://peers` {#event-broadcast-peers}

ディスカバリーリスナーがこれまでに聞き取ったものです。400 ms ごとに送られます。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | リスナーのジョブ |
| `ts` | number | 時刻 |
| `peers` | object[] | 最近聞き取ったものが先: `addr`、`proto`、`packets`、`bytes`、`first_ms`、`last_ms`、`last_summary`、`responded`（応答があったパケット数。届いた時点で数えます）。最大 512 |
| `packets`、`bytes` | number | 受信したすべて |
| `responses` | number | 送信した応答の数 |
| `unlisted` | number | テーブルが保持する 512 を超えた、異なるピアの数。ここで数えられ、`peers` には含まれません |
| `replies_skipped` | number | すでに自動応答が多すぎる（1024）ために応答しなかったプローブの数。これは、プローブの大量送信が際限なく増幅されるのを防ぐ上限です |

### `netsim://stat` {#event-netsim-stat}

劣化リレーのカウンターです。250 ms ごとに送られます。実行の [[ui:exp.node.impairment]] ノードのリレーは、代わりに実行のレポートに報告します。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | リレーのジョブ |
| `ts` | number | 時刻 |
| `received`、`forwarded` | number | 入ってきた、出ていったデータグラムまたはチャンクの数 |
| `dropped` | number | `loss`、バースト、`offline` で失われた数（UDP。TCP のリレーはオフラインの間ストリームを保留し、何も破棄しません） |
| `throttled` | number | UDP: 帯域制限によって、または送信待ちが多すぎたために破棄された数。TCP: 帯域制限のためにストリームを保留したチャンクの数 |
| `duplicated`、`corrupted`、`reordered` | number | プロファイルがそれらに対して行ったこと |
| `bytes` | number | 通過させたバイト数 |
| `connections`、`reset`、`stalled` | number | TCP: 受け付けた接続、リセットした接続、ハーフオープンのままにした接続の数。0 の間は省略されます |
| `refused` | number | TCP: 宛先に拒否された、または応答がなかったために、クライアントにはリセットとして伝えた接続の数。0 の間は省略されます |
| `failed` | number | このマシンが通過させられなかったもの: システムが送信しなかったデータグラム、ローカルポートやソケットがなかった接続（先に約 2 秒間、再試行されます）、またはリレーの同時 1,024 接続を超えた接続。0 の間は省略されます |
| `profile` | string | いま劣化に使っているプロファイル。タイムラインでの呼び方と同じで、その名前、または内容（`60 ms ±25 · loss 2%`） |

### `storm://stat` {#event-storm-stat}

ストームのカウンターです。250 ms ごとに、そして自力で終了したときに `pps` と `mbps` を 0 にしてもう一度送られます。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | ストームのジョブ |
| `ts` | number | 時刻 |
| `packets`、`bytes` | number | 送信したデータグラム（または TCP 接続）の数とバイト数 |
| `errors` | number | 宛先が拒否した、リセットした、または応答しなかった送信や接続の数 |
| `local` | number | このマシンが送れなかった単位の数。宛先が拒否したのではなく、ローカルポート、ソケット、バッファが尽きたもの。`errors` には数えられません |
| `pps` | number | 直近の 250 ms での毎秒の数 |
| `mbps` | number | 直近の 250 ms での毎秒のメガビット数 |

### `scan://open` {#event-scan-open}

スキャナーが開いているポートを見つけました。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | スキャンのジョブ |
| `ts` | number | 時刻 |
| `port` | number | ポート |
| `banner` | string or null | サービスが最初に送ってきたもの。バナーを要求していて、400 ms 以内にサービスが何か送ってきた場合 |

### `scan://progress` {#event-scan-progress}

スキャンの進み具合です。範囲の約 1% ごとと、自力で終了したときに `done` が `total` と等しい状態で送られます（後者は 2 回届くことがあります）。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | スキャンのジョブ |
| `ts` | number | 時刻 |
| `done` | number | 試したポートの数 |
| `total` | number | 範囲内のポートの数 |
| `open` | number | 見つかった開いているポートの数 |
| `unchecked` | number | このマシンが試せなかった（ローカルポートやソケットが尽きた）ポートの数。開いているとも閉じているとも言えません。通常は 0 |

### `emulator://activity` {#event-emulator-activity}

`emulator_start` で開始したエミュレーターが、前回のイベント以降に受け取り、答えたものです。何かが変わったとき（やり取り、ダウンまたは復帰、MQTT ブローカーが配信できなかったメッセージ）に 200 ms ごとに送られます。実行の [[ui:exp.node.emulator]] ノードはこれを送りません。そのカウンターは、実行のレポートにあります。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `job_id` | number | エミュレーターのジョブ |
| `ts` | number | 時刻 |
| `counts` | object | `total`、`unmatched`、`failed`、`down`、`hits`（ルールごと）、`missed`（MQTT。0 の間は省略）、`refused`（上限を超えた接続。0 の間は省略）。[`emulator_exchanges`](commands.md#emulator_exchanges) と同じです |
| `forced` | string | ダウンさせている間の `unavailable`、`reset`、`timeout`。そうでなければ省略されます |
| `exchanges` | object[] | 新しいやり取り。`emulator_exchanges` が一覧するのと同じ形ですが、`data` はありません。最大 200 |
| `dropped` | number | その間隔の最初の 200 件を超えて、ここでは送られなかったやり取りの数。`emulator_exchanges` には、最後の 500 件が残っています |

### `inspect://batch` {#event-inspect-batch}

インスペクターの新しいフレームです。キャプチャがオンのときだけ送られます。新しいフレームがあれば 120 ms ごとに、ないときは約 1 秒に 1 回送られるので、カウンターは最新のままです。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `frames` | object[] | 新しい[フレーム](commands.md#type-frame)。古いものが先で、最大 250。バイト列は含みません（`inspect_payload` を使ってください） |
| `stats` | object | キャプチャのカウンター、[`CaptureStats`](commands.md#type-frame) |
| `skipped_now` | number | 前回のバッチ以降にキャプチャされたが、このバッチに含まれなかったフレームの数。250 件を超えて届いたか、バッファが手放したものです。バッファが保持している間は、エクスポートに含まれます |

### `server://lagged` {#event-server-lagged}

サーバーのみです。このクライアントが 4096 件を超えるイベントだけ遅れ、一部を取りこぼしました。コマンドで状態を読み直してください。

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `skipped` | number | 取りこぼしたイベントの数 |

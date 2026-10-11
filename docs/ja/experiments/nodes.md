---
title: ノード
description: 実験のすべての種類のノードについて、動作、既定値と制限を含むフィールド、出力、使える設定、実験ファイルでの書き方を説明します。
---

# ノードリファレンス

実験が持てるすべての種類のノードを、追加メニューのグループ順に説明します。[アクション](#actions)、[待機](#waits)、[エミュレーション](#emulation)、[障害](#faults)、[データ](#data)、[チェック](#checks)、[フロー](#flow)です。追加や接続の方法は[エディター](index.md)にあります。`signallab nodes` は同じカタログを JSON で出力するので、スクリプトやアシスタントから使えます（[コマンドライン](../automation/cli.md)）。

## このページの読み方 {#reading}

各ノードには、フィールドの表があります。

- **フィールド**はプロパティペインでの名前、**ファイル内**は実験の JSON でのキーです。
- **既定値**は、エディターでノードを追加したときに設定される値です。ファイルでキーを省略できる場合、その際に使われる値を *if absent* として示しています。ほかのキーはファイルでは必須です。
- **テンプレート**: *はい* — そのフィールドは `{{templates}}`（パラメーター、先に設定された変数、シークレット、ジェネレーター）を受け付け、ステップの実行時に解決されます（[データとテンプレート](data.md)）。*パラメーターのみ* — 最初のステップの前に開かれるため、その時点で分かっているのはパラメーターだけです。*いいえ* — 値は書かれたとおりに使われます。

時間の単位はミリ秒です。制限は実行の開始前にチェックされ、範囲外のフィールドがあると実験は実行できず、そのノードに表示されます。

## ファイルの中のノード {#file-shape}

実験ファイルでは、ノードは次のものを持つオブジェクトです。`id`（実験内で一意）、`type`、キャンバス上の位置（`x`、`y`。0 個以上）、フィールド、そしてそのノードが使う設定（`retry`、`repeat`、`load`。オフのときは省略されます）。接続線は、あるノードの出力（`port`、省略時は `next`）から別のノードへのエッジです。

```json
{
  "nodes": [
    { "id": "start", "type": "start", "x": 40, "y": 80 },
    { "id": "ping", "type": "udp", "x": 270, "y": 80, "target": "127.0.0.1:9000", "text": "PING",
      "retry": { "attempts": 3, "delay_ms": 500, "backoff": "fixed" } },
    { "id": "end", "type": "end", "x": 500, "y": 80 }
  ],
  "edges": [
    { "from": "start", "to": "ping", "port": "next" },
    { "from": "ping", "to": "end", "port": "next" }
  ]
}
```

以下の例は、ファイルに保存されるとおりの形で、ノードを 1 つずつ示します。

## 多くのノードに共通の設定 {#settings}

これらは、ノードのプロパティの下部で有効にします。どのノードがどの設定を使えるかは、各ノードの項に記載しています。

| 設定 | 使えるノード | 動作 |
| --- | --- | --- |
| [再試行](#retry) | 送信または待ち受けするノード: [[ui:exp.node.http]]、[[ui:exp.node.tcp]]、[[ui:exp.node.osc]]、[[ui:exp.node.udp]]、[[ui:exp.node.mqtt]]、[[ui:exp.node.ws_connect]]、[[ui:exp.node.ws_send]]、およびすべての待機ノード | ステップが失敗したときにやり直します |
| [繰り返し](#repeat) | 送信するノード: [[ui:exp.node.http]]、[[ui:exp.node.tcp]]、[[ui:exp.node.osc]]、[[ui:exp.node.udp]]、[[ui:exp.node.mqtt]]、[[ui:exp.node.ws_send]] | 指定した回数、または指定した時間、送信を繰り返します |
| [負荷](#load) | [[ui:exp.node.http]] | 負荷プロファイルに従ってリクエストを送り、しきい値で測定、判定します |
| [応答を待つ](#reply) | [[ui:exp.node.osc]]、[[ui:exp.node.udp]] | 同じステップの中で送信し、応答を待ちます |

### 再試行 {#retry}

[[ui:exp.retryOn]]: ステップが失敗したとき（接続できない、タイムアウト、何も一致しない待機）に、一時停止してからもう一度実行します。失敗した試行はそれぞれタイムラインの 1 行になり、最後の試行が失敗するとステップも失敗します。解決できないテンプレートは再試行されません。[[ui:common.stop]] は一時停止も終了させます。

| フィールド | ファイル内 | 内容 | 既定値と制限 |
| --- | --- | --- | --- |
| [[ui:exp.attempts]] | `retry.attempts` | 最初の試行を含む、合計の試行回数 | 3。エディターでは 2〜10（ファイルでは 1 も指定できます） |
| [[ui:exp.retryDelay]] | `retry.delay_ms` | 2 回目の試行の前の一時停止 | 500。0〜60,000 |
| [[ui:exp.backoff]] | `retry.backoff` | [[ui:exp.backoff.fixed]]（`fixed`）: 毎回同じ間隔。[[ui:exp.backoff.exponential]]（`exponential`）: 失敗のたびに 2 倍の長さ | `fixed`（if absent も同じ） |

倍増しても、一時停止が 60 秒を超えることはありません。[[ui:exp.portTimeout]] 出力に接続線がある待機は、タイムアウトしても失敗せず、その出力から先へ進むので、その場合は再試行されません。

### 繰り返し {#repeat}

[[ui:exp.repeatOn]]: ノードは、グラフにループを置かなくても、送信を繰り返します。ハートビート、ポーリング、一定のストリームなどです。送信のたびにテンプレートを読み直し（`{{counter}}` はその送信の番号、`{{now}}` はその時刻）、再試行が有効な場合は、送信ごとに適用されます。すべての送信が成功するとステップが成功し、最終的に失敗した送信があるとステップは失敗します。タイムラインに進行状況が表示されるのは、1 秒に 1 回までです。

| フィールド | ファイル内 | 内容 | 既定値と制限 |
| --- | --- | --- | --- |
| [[ui:exp.repeatBy]] | `repeat.until` | [[ui:exp.repeatBy.count]]（`count`）または [[ui:exp.repeatBy.duration]]（`duration`） | `count`（if absent も同じ） |
| [[ui:exp.repeatCount]] | `repeat.count` | 最初の送信を含む、合計の送信回数 | 10（if absent も同じ）。2〜10,000 |
| [[ui:exp.repeatDuration]] | `repeat.duration_ms` | 最初の送信から、送信を続ける時間 | 10,000（if absent も同じ）。1〜300,000 |
| [[ui:exp.repeatInterval]] | `repeat.interval_ms` | 2 回の送信の間の一時停止 | 1,000。10〜60,000。ファイルでは必須 |
| [[ui:exp.repeatJitter]] | `repeat.jitter_ms` | 各一時停止に、実行のシードから決まる、この値までの時間が加わります | 0（if absent も同じ）。0〜60,000 |

繰り返しは 1 回の実行の 300 秒に収まる必要があり、*時間を指定*する場合は、必要な送信が 10,000 回未満（時間を間隔で割った値）でなければなりません。

### 負荷 {#load}

[[ui:exp.loadOn]]、[[ui:exp.node.http]] のみ: リクエストをプロファイル（一定のレート、ランプ、段階、スパイク、ランダムな到着）に従って、最大 512 件まで同時に（既定では 32 件）送信し、レイテンシ、エラー、達成したレートを測定します。しきい値でステップが成功するかどうかが決まります。負荷は繰り返しと再試行の代わりに使われ（失敗したリクエストは数えられ、再試行されません）、その後ろのチェックには、使えるレスポンスが残りません。フィールドと結果は[負荷テスト](load.md)にあります。

### 応答を待つ {#reply}

[[ui:exp.expectReply]]、[[ui:exp.node.osc]] または [[ui:exp.node.udp]]: メッセージは応答を待ち受けるポートから送信されるので、送信元に返答するデバイスの応答が受け取れ、ステップが成功するのは、一致する応答が時間内に届いたときだけです。応答がなければステップは失敗し、再試行が有効なら再送信されます。応答は、待機ノードの場合と同じように、変数に保存されます。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.replyOn]] | `reply.bind` | 送信元として使い、待ち受けも行う `IP:port`。ポート 0 は空いている任意のポートになります | `0.0.0.0:0` | いいえ |
| [[ui:exp.replyAddress]] (OSC) | `reply.address` | 応答のアドレスパターン。[[[ui:exp.node.wait_osc]]](#node-wait_osc)と同じです | `/*` | はい |
| [[ui:exp.argRules]] (OSC) | `reply.args` | 引数のルール。[[ui:exp.node.wait_osc]] と同じです | なし。最大 16 | 値: はい |
| [[ui:exp.replyMode]] (UDP) | `reply.mode` | `any`、`contains`、`regex`、`hex` のいずれか。[ペイロードの照合](#payload-matching)を参照 | `any`（if absent も同じ） | いいえ |
| [[ui:field.pattern]] (UDP) | `reply.pattern` | 応答に含まれていなければならない、または一致しなければならないもの | 空。`any` 以外では必須 | はい |
| [[ui:exp.waitTimeout]] | `reply.timeout_ms` | 待つ時間 | 2,000（if absent も同じ）。1〜120,000 | いいえ |
| [[ui:exp.replyVariable]] | `reply.variable` | 応答を保存する変数 | `reply`（if absent も同じ） | いいえ |

応答のポートは、待機ノードのものと同じように、最初のステップの前に開かれます。

## アクション {#actions}

送信するノードです。アクションの後ろにある待機ノードは、そのアクションが始まった時点からメッセージを数えます。

### HTTP リクエスト {#node-http}

HTTP リクエストを 1 つ送信し、その後ろにあるチェック、分岐、[[ui:exp.node.extract]] ノードのために、レスポンスを保持します。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.method]] | `request.method` | GET、HEAD、POST、PUT、PATCH、DELETE、OPTIONS（ファイルでは任意のメソッドを指定できます） | `GET` | いいえ |
| URL | `request.url` | `http://` または `https://` の URL | `http://127.0.0.1:8080/` | はい |
| [[ui:common.timeoutMs]] | `request.timeout_ms` | 通信全体に対する時間 | 4,000（if absent は 10,000）。1〜120,000 | いいえ |
| [[ui:exp.headers]] | `request.headers` | `[[name, value], …]`。名前が空の行は無視されます | なし | はい（名前と値） |
| [[ui:exp.body]] | `request.body` | テキスト。ボディなしは `null` | `null` | はい |
| [[ui:field.auth]] | `request.auth` | [[ui:http.auth.none]]、[[ui:http.auth.basic]]、[[ui:http.auth.bearer]]、[[ui:http.auth.digest]] のいずれかで、[[ui:field.username]] と [[ui:field.password]]、または [[ui:field.token]] を伴います | なし | はい |

- どんな応答でもステップは成功します。404 や 500 も同様です。ステータスは [[[ui:exp.node.assert_status]]](#node-assert_status)で確認するか、[[[ui:exp.node.branch_status]]](#node-branch_status)で分岐してください。応答が得られないリクエスト（接続拒否、タイムアウト、名前が解決できない、証明書が信頼されない）は、ステップを失敗にします。
- リダイレクトは最大 10 回まで追従します。`https://` の証明書は検証されます。
- レスポンスボディは、チェックのために 256 KiB まで保持されます。それより大きいボディはそこで切り捨てられます（探しているものが切り捨てられた先にあるかもしれないときは、チェックがそのことを表示します）。
- Digest は、サーバーの 401 チャレンジに答えて、リクエストを再送します。資格情報はリクエストにだけ入ります。ステップ、レポート、インスペクターに `Authorization` ヘッダーが表示されることはありません。パスワードは `{{secret.NAME}}` と書いてください。
- 実験が Cookie を保持している間（既定でオン。[[ui:exp.params]] で設定）、サーバーが設定したものは、その後の同じサーバーへのリクエストで送り返されます。

出力: [[ui:exp.outputPort]]。設定: 再試行、繰り返し、負荷。

```json
{ "id": "cue", "type": "http", "x": 270, "y": 80,
  "request": { "method": "POST", "url": "{{api}}/cue", "headers": [["Content-Type", "application/json"]],
               "body": "{\"cue\": 1}", "timeout_ms": 5000,
               "auth": { "scheme": "bearer", "token": "{{secret.API_TOKEN}}" } } }
```

関連: [HTTP](../protocols/http.md)。

### TCP メッセージ {#node-tcp}

ホストに TCP で接続し、ペイロードを書き込み、応答の最初のバイトを最大 250 ms 待ち（読み取るのは最大 1,024 バイトで、1 回だけです）、接続を閉じます。応答のサイズは報告されますが、チェックはされません。

[インスペクター](../tools/inspector.md)では、このステップは、送信元が `experiment` の 2 つの `tcp` フレームになります。書き込んだペイロードと、応答があればその読み取った内容です。使用中のシークレットは、どのフレームでもそうであるように、両方でマスクされます。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.host]] | `host` | ホスト名または IP アドレス | `127.0.0.1` | はい |
| [[ui:exp.port]] | `port` | | 9000。1〜65,535 | いいえ |
| [[ui:common.timeoutMs]] | `timeout_ms` | 接続、書き込み、応答の合計に対する時間 | 4,000（if absent も同じ）。1〜120,000 | いいえ |
| [[ui:exp.payload]] | `payload` | 接続後に書き込むテキスト（UTF-8） | `hello` | はい |

接続が拒否された場合、名前が解決できない場合、時間切れになった場合は、ステップが失敗します。出力: [[ui:exp.outputPort]]。設定: 再試行、繰り返し。[[ui:exp.sendNow]] は接続してペイロードを 1 回書き込み、ノードの結果には、送信したバイト数と返ってきたバイト数が表示されます。

```json
{ "id": "go", "type": "tcp", "x": 270, "y": 80, "host": "127.0.0.1", "port": 5000, "payload": "GO\r\n", "timeout_ms": 2000 }
```

### OSC メッセージ {#node-osc}

OSC 1.0 メッセージを UDP で 1 つ送信します。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:common.target]] | `target` | `IP:port` または `host:port`。ホスト名はステップの送信時に解決され、IPv4 アドレスがあればそれが使われます | `127.0.0.1:9000` | はい |
| [[ui:common.address]] | `address` | `/` で始まります | `/test` | はい |
| [[ui:exp.arguments]] | `args` | `[{ "type", "value" }, …]`。`int`、`float`、`str`、`long`、`double`、`bool`、`blob`（バイト列）、`nil`（値なし） | なし | テキスト（`str`）の値: はい |
| [[ui:exp.expectReply]] | `reply` | 任意: 送信して応答を待ちます。[応答を待つ](#reply)を参照 | オフ | |

出力: [[ui:exp.outputPort]]。応答を待つ設定にしている場合は、応答が届いたときだけたどられます。設定: 再試行、繰り返し、応答。プロパティの ⚡ [[ui:exp.routeThrough]] は、その前に[[[ui:exp.node.impairment]]](#node-impairment)を置きます。

```json
{ "id": "fader", "type": "osc", "x": 270, "y": 80, "target": "{{device}}", "address": "/fader/1",
  "args": [{ "type": "float", "value": 0.75 }] }
```

関連: [OSC](../protocols/osc.md)。

### UDP データグラム {#node-udp}

テキストのペイロードを、1 つまたは複数の宛先へ 1 つの UDP データグラムとして送信します。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:common.target]] | `target` | `IP:port` または `host:port`。カンマ、セミコロン、改行で区切った複数の宛先には、それぞれにデータグラムが送られます。ホスト名はステップの送信時に解決され、IPv4 アドレスがあればそれが使われます | `127.0.0.1:9000` | はい |
| [[ui:exp.payload]] | `text` | ペイロード（UTF-8） | `hello`。最大 65,507 バイト | はい |
| [[ui:exp.expectReply]] | `reply` | 任意: 送信して応答を待ちます。[応答を待つ](#reply)を参照 | オフ | |

届かない宛先が 1 つでもあれば、ステップは失敗します。出力: [[ui:exp.outputPort]]。設定: 再試行、繰り返し、応答。

```json
{ "id": "ping", "type": "udp", "x": 270, "y": 80, "target": "{{device}}", "text": "PING {{run.id}}",
  "reply": { "bind": "0.0.0.0:0", "mode": "contains", "pattern": "PONG", "timeout_ms": 1000, "variable": "pong" } }
```

### MQTT パブリッシュ {#node-mqtt}

MQTT ブローカーに接続し、メッセージを 1 つパブリッシュして、切断します。接続は、プレーン TCP 上の MQTT 3.1.1 で、クリーンセッション、ユーザー名とパスワードなしです。接続、パブリッシュ、ブローカーの確認応答は、すべて 15 秒以内に完了する必要があります。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.broker]] | `host` | ブローカーのホスト名またはアドレス | `127.0.0.1` | はい |
| [[ui:exp.port]] | `port` | | 1883。1〜65,535 | いいえ |
| [[ui:exp.topic]] | `topic` | ワイルドカード（`+`、`#`）は使えません | `lab/test` | はい |
| [[ui:exp.payload]] | `payload` | メッセージ（テキスト） | `hello` | はい |
| QoS | `qos` | 0、1、2 のいずれか | 0 | いいえ |
| [[ui:exp.retain]] | `retain` | `true`: ブローカーがそのトピックの値として保持します | `false` | いいえ |

ファイルでは、6 つのキーがすべて必須です。ブローカーに届かない場合や、ブローカーが接続またはメッセージを拒否した場合は、ステップが失敗します。出力: [[ui:exp.outputPort]]。設定: 再試行、繰り返し。

```json
{ "id": "light", "type": "mqtt", "x": 270, "y": 80, "host": "{{broker}}", "port": 1883,
  "topic": "lab/light/1/set", "payload": "on", "qos": 1, "retain": false }
```

関連: [MQTT](../protocols/mqtt.md)。

### WebSocket 接続 {#node-ws_connect}

WebSocket を、実行の残りの間、または [[[ui:exp.node.ws_close]]](#node-ws_close)が来るまで開きます。それ以降に届いたものは、その接続に対する[[[ui:exp.node.wait_ws]]](#node-wait_ws)ステップのために保持されます。URL とヘッダーはステップの実行時に解決されるので、先に抽出したトークンを含めることができます。[[ui:exp.node.loop]] の中などでもう一度実行されると、前の接続を先に閉じてから、新しい接続を開きます。実行がどのような形で終わっても、その接続はクローズフレームとともに閉じられます。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| URL | `url` | `ws://` または `wss://` の URL | `ws://127.0.0.1:9001/` | はい |
| [[ui:exp.headers]] | `headers` | アップグレードリクエストと一緒に送る `[[name, value], …]` | なし | はい（名前と値） |
| [[ui:exp.wsProtocols]] | `protocols` | 提示するサブプロトコル（優先順）。サーバーが 1 つを選びます | なし | いいえ |
| [[ui:common.timeoutMs]] | `timeout_ms` | 接続とアップグレードに対する時間 | 5,000（if absent は 10,000）。1〜120,000 | いいえ |

`wss://` は `https://` と同じ証明書を信頼します。接続またはアップグレードが失敗すると、ステップが失敗し、サーバーのステータスが理由に含まれます。出力: [[ui:exp.outputPort]]。設定: 再試行（繰り返しは不可）。

```json
{ "id": "socket", "type": "ws_connect", "x": 270, "y": 80, "url": "ws://127.0.0.1:9001/chat",
  "headers": [["Authorization", "Bearer {{token}}"]], "protocols": ["chat.v1"], "timeout_ms": 5000 }
```

関連: [WebSocket](../protocols/websocket.md)。

### WebSocket 送信 {#node-ws_send}

[[ui:exp.node.ws_connect]] が開いた接続で、メッセージを 1 つ送信します。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | この実験の [[ui:exp.node.ws_connect]] ノードの ID | 最初のもの | いいえ |
| [[ui:exp.wsFormat]] | `binary` | [[ui:exp.wsText]]（`false`）、または [[ui:exp.wsBinary]]（`true`）: ペイロードは `de ad be ef` のように hex で書いたバイト列です | `false`（if absent も同じ） | いいえ |
| [[ui:exp.payload]] | `text` | メッセージ | `hello`。最大 16 MiB | はい |

接続ノードは、その経路上で送信ノードより前になければなりません。接続が開いていない送信は失敗します。応答は、メッセージを書き込んだ時点から数えられます。出力: [[ui:exp.outputPort]]。設定: 再試行、繰り返し。

```json
{ "id": "hello", "type": "ws_send", "x": 500, "y": 80, "connection": "socket",
  "text": "{\"type\":\"ping\",\"id\":\"{{uuid}}\"}", "binary": false }
```

### WebSocket クローズ {#node-ws_close}

クローズハンドシェイクで接続を閉じます。タイムラインには、閉じたのが誰か（このステップか、先に閉じたサーバー（そのコード付き）か、切れていた接続か）が表示されます。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | [[ui:exp.node.ws_connect]] ノードの ID | 最初のもの | いいえ |
| [[ui:field.code]] | `code` | 1000（正常）、またはアプリケーション独自の 3000〜4999 | 1000（if absent も同じ） | いいえ |
| [[ui:field.reason]] | `reason` | コードと一緒に送られます | 空。テンプレートの解決後に最大 123 バイト | はい |

出力: [[ui:exp.outputPort]]。設定なし。

```json
{ "id": "bye", "type": "ws_close", "x": 960, "y": 80, "connection": "socket", "code": 1000, "reason": "done" }
```

### ログマーカー {#node-log}

タイムラインとレポートに 1 行を書き込みます。チェックポイントや、実行が到達した値の記録に使います。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.logMessage]] | `message` | テキスト | `Check point`。最大 10,000 文字 | はい |

出力: [[ui:exp.outputPort]]。設定なし。

```json
{ "id": "ready", "type": "log", "x": 500, "y": 80, "message": "device {{device}} ready" }
```

## 待機 {#waits}

[[ui:exp.group.observe]] グループ: 何かが届くのを待つノードです。次のルールは共通です。

- **実行の最初から待ち受けます。** 待機ノードのポートまたはブローカーのサブスクリプションは最初のステップの前に開かれるので、次のステップが始まるより速く応答するデバイスも取りこぼしません。同じアドレスに対する 2 つの待機ノードは、1 つのソケットを共有します。
- **ブランチ上の直近のアクションから数えます。** ブランチの最後のリクエストより前に届いたメッセージは、そのリクエストへの応答ではありません。アクションの前では、実行の開始以降のすべてが対象です。
- **最初に一致したメッセージが採用されます。** ある待機ノードが採用したメッセージは、ほかの待機ノードには見えません。
- **[[ui:exp.portMatched]] または [[ui:exp.portTimeout]]。** 一致した場合、メッセージは待機ノードの変数に保存され、フローは [[ui:exp.portMatched]] に従って進みます。時間切れになった場合、[[ui:exp.portTimeout]] 出力に接続線があればそちらに従い、なければステップが失敗して、ほかに何件のメッセージが届いたかが表示されます。
- 各ソケットは、直近の 1,024 件のメッセージ（と 64 MiB）を保持します。それより古いものは破棄され、タイムアウトの際にその件数が表示されます。
- [[ui:exp.listenNow]] は、そのステップ単独で、今から待ち受けます。

出力: [[ui:exp.portMatched]]（必須）、[[ui:exp.portTimeout]]（任意）。設定: 再試行。

### ペイロードの照合 {#payload-matching}

[[ui:exp.node.wait_udp]]、[[ui:exp.node.wait_mqtt]]、[[ui:exp.node.wait_ws]]、UDP の応答では、ペイロードがどのような形であるべきかを選びます。

| オプション | ファイル内 | ペイロードが次のとき一致します |
| --- | --- | --- |
| [[ui:exp.mode.any]] | `any` | 何でも一致します |
| [[ui:exp.mode.contains]] | `contains` | UTF-8 テキストとして読んだときに、パターンを含む（大文字と小文字を区別します） |
| [[ui:exp.mode.regex]] | `regex` | UTF-8 テキストとして読んだときに、正規表現に一致する |
| [[ui:exp.mode.hex]] | `hex` | hex のペアで書いたバイト列を含む: `de ad be ef`、`deadbeef`、`0xde,0xad`、`DE:AD` |

一致したメッセージは、オブジェクトとして保存されます。後のステップは、そのフィールドを `{{reply.text}}` のように読めます（`reply` の部分は変数名に置き換えます）。

| フィールド | 内容 |
| --- | --- |
| `text` | ペイロードのテキスト |
| `hex`、`bytes` | ペイロードの hex（最初の 1,024 バイト）と、そのバイト単位のサイズ |
| `match` | 一致した部分: テキスト、正規表現の最初のグループ（なければ一致全体）、またはバイト列 |
| `from` | 送信元の `IP:port` |
| `ms` | ブランチの直近のアクション（または実行の開始）からメッセージまでのミリ秒数 |
| `topic` | [[ui:exp.node.wait_mqtt]]: パブリッシュされたトピック |
| `json`、`kind` | [[ui:exp.node.wait_ws]]: JSON として解析したメッセージ（JSON でなければ `null`）と、`text` か `binary` か |

### OSC を待機 {#node-wait_osc}

アドレスがパターンに一致し、引数がすべてのルールを満たす OSC メッセージを待ちます。バンドルの中では、最初に一致したメッセージが採用されます。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | 待ち受ける `IP:port`。すべてのネットワークカードなら `0.0.0.0` | `127.0.0.1:9001` | いいえ |
| [[ui:exp.addressPattern]] | `address` | `*` は任意の文字列、`?` は 1 文字、`[0-9]` は文字の集合（`[!0-9]` はその外）、`{ping,pong}` はどちらか。ワイルドカードは `/` で区切られた 1 つのセグメントの中にとどまります | `/pong`。最大 512 文字 | はい |
| [[ui:exp.argRules]] | `args` | `[{ "index", "op", "value" }, …]`: `index` 番目の引数を `op` で `value` と比較します（[比較](#comparisons)）。すべて成り立つ必要があります | なし。最大 16、index は 0〜63 | 値: はい |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2,000（if absent も同じ）。1〜120,000 | いいえ |
| [[ui:exp.replyVariable]] | `variable` | メッセージの保存先 | `reply`（if absent も同じ） | いいえ |

引数はテキストとして比較されます。数値は書かれたとおり、文字列は引用符なし、`true`/`false`、blob は hex です。メッセージにない引数に対するルールは成り立ちません。保存されるメッセージには、`address`、`args`（`{{reply.args[0]}}`）、`from`、`ms` があります。

```json
{ "id": "status", "type": "wait_osc", "x": 500, "y": 80, "bind": "0.0.0.0:9001", "address": "/status",
  "args": [{ "index": 0, "op": "eq", "value": "ready" }], "timeout_ms": 5000, "variable": "reply" }
```

### UDP を待機 {#node-wait_udp}

ペイロードが一致する UDP データグラムを待ちます。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | 待ち受ける `IP:port` | `127.0.0.1:9001` | いいえ |
| [[ui:exp.waitMode]] | `mode` | [ペイロードの照合](#payload-matching)を参照 | `contains`（if absent は `any`） | いいえ |
| [[ui:field.pattern]] | `pattern` | ペイロードに含まれていなければならない、または一致しなければならないもの | `pong`。`any` 以外では必須 | はい |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2,000（if absent も同じ）。1〜120,000 | いいえ |
| [[ui:exp.replyVariable]] | `variable` | | `reply`（if absent も同じ） | いいえ |

```json
{ "id": "ready", "type": "wait_udp", "x": 500, "y": 80, "bind": "0.0.0.0:9002", "mode": "contains",
  "pattern": "READY", "timeout_ms": 5000, "variable": "reply" }
```

### MQTT を待機 {#node-wait_mqtt}

ブローカーのトピックにパブリッシュされた、ペイロードが一致するメッセージを待ちます。実行は、最初のステップの前に接続してサブスクライブします。サブスクライブ時にブローカーが再送する保持メッセージは無視されます。実行の開始後にパブリッシュされたものだけが対象です。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.broker]] | `host` | ブローカー | `127.0.0.1` | パラメーターのみ |
| [[ui:exp.port]] | `port` | | 1883。1〜65,535 | いいえ |
| [[ui:exp.topicFilter]] | `topic` | フィルター: `+` は任意の 1 階層、`#` はその下すべて（最後にだけ置けます） | `lab/#` | パラメーターのみ |
| [[ui:exp.waitMode]] | `mode` | [ペイロードの照合](#payload-matching)を参照 | `any`（if absent も同じ） | いいえ |
| [[ui:field.pattern]] | `pattern` | | 空。`any` 以外では必須 | はい |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2,000（if absent も同じ）。1〜120,000 | いいえ |
| [[ui:exp.replyVariable]] | `variable` | | `reply`（if absent も同じ） | いいえ |

```json
{ "id": "state", "type": "wait_mqtt", "x": 500, "y": 80, "host": "{{broker}}", "port": 1883,
  "topic": "lab/+/state", "mode": "contains", "pattern": "on", "timeout_ms": 5000, "variable": "reply" }
```

### HTTP リクエストを待機 {#node-wait_http}

HTTP リクエスト（Webhook やコールバック）を待ちます。宛先は、そのアドレスにある実行中の[[[ui:exp.node.emulator]]](#node-emulator)で、その場所に HTTP エミュレーターがない場合は、実行自身のリスナーが、すべてのリクエストに 204 で答えて受け取ります。リクエストは、メソッド、パス、すべての条件に一致する必要があります。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | `IP:port` | `127.0.0.1:18080`。新しい [[ui:exp.node.emulator]] が待ち受ける場所です | いいえ |
| [[ui:exp.method]] | `method` | メソッド、または [[ui:emu.methodAny]]（`ANY`）。GET は HEAD も受け付けます | `ANY`（if absent も同じ） | いいえ |
| [[ui:exp.path]] | `path` | `/hooks/:name` はセグメントに名前を付けます（`{{request.params.name}}`）。末尾の `/*` は残りすべてを受け付けます | `/*`（if absent も同じ）。最大 512 文字 | はい |
| [[ui:emu.conditions]] | `when` | `[{ "on", "name", "op", "value" }, …]`。`header`、`query` パラメーター、`body`、`json` パスに対する条件で、すべて成り立つ必要があります | なし。最大 16 | はい（名前と値） |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 5,000（if absent は 2,000）。1〜120,000 | いいえ |
| [[ui:exp.replyVariable]] | `variable` | | `request`（if absent も同じ） | いいえ |

保存されるリクエストには、`method`、`path`、`query`、`headers`、`body`、`json`、`params`、`from`、`ms` があります。たとえば `{{request.json.event}}`、`{{request.headers.x-key}}` のように読みます。

```json
{ "id": "hook", "type": "wait_http", "x": 500, "y": 80, "bind": "127.0.0.1:18081", "method": "POST",
  "path": "/hooks/:name", "when": [{ "on": "json", "name": "$.event", "op": "eq", "value": "deploy" }],
  "timeout_ms": 5000, "variable": "request" }
```

### WebSocket を待機 {#node-wait_ws}

[[ui:exp.node.ws_connect]] が開いた接続で、ペイロードが一致するメッセージを待ちます。ブランチ上の直近のアクション（接続自体、送信、そのほかのリクエスト）以降のメッセージが対象です。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | [[ui:exp.node.ws_connect]] ノードの ID | 最初のもの | いいえ |
| [[ui:exp.waitMode]] | `mode` | [ペイロードの照合](#payload-matching)を参照 | `any`（if absent も同じ） | いいえ |
| [[ui:field.pattern]] | `pattern` | | 空。`any` 以外では必須 | はい |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2,000（if absent も同じ）。1〜120,000 | いいえ |
| [[ui:exp.replyVariable]] | `variable` | | `reply`（if absent も同じ） | いいえ |

JSON メッセージは、`{{reply.json.type}}` のようにフィールドごとに読めます。接続ノードは、その経路上で待機ノードより前になければなりません。

```json
{ "id": "pong", "type": "wait_ws", "x": 730, "y": 80, "connection": "socket", "mode": "contains",
  "pattern": "pong", "timeout_ms": 3000, "variable": "reply" }
```

## エミュレーション {#emulation}

### エミュレーター {#node-emulator}

依存先（HTTP API、OSC、UDP、TCP のデバイス、MQTT ブローカー）を、実行の間ずっと演じます。最初のステップの前に開き、実行が終わるまで応答します。フローの中では、ステップはすぐに成功します。受け取ったものは、ルールごとに数えられ、実行のレポートに記録されます。

| フィールド | ファイル内 | 内容 | 既定値 |
| --- | --- | --- | --- |
| [[ui:emu.edit]] | `emulator` | エミュレーター: `name`、`bind`（`IP:port`）、`protocol`（`http`、`osc`、`udp`、`tcp`、`mqtt`）、そのルートまたはルール、任意の `outage` | `127.0.0.1:18080` 上で、`/health` に答える *API* という名前の HTTP API |

プロパティには、何を演じるかが 1 行で表示されます。[[ui:emu.edit]] はそのルールを開きます。[[[ui:nav.emulators]]](../tools/emulators.md)画面と同じエディターです。[[ui:emu.toLibrary]] はコピーをエミュレーターライブラリに保存し、[[ui:emu.fromLibrary]] はこのエミュレーターをそこからのコピーで置き換えます。ルール（ルート、レスポンス、障害、ダウン）はそちらで説明しています。

- HTTP エミュレーターは、そのアドレスに対する[[[ui:exp.node.wait_http]]](#node-wait_http)が読み取る相手でもあります。OSC または UDP エミュレーターは、そこにある実行の待機ノードとポートを共有します。
- 同じトランスポートの 2 つのエミュレーターは、1 回の実行の中でポートを共有できません。
- [[[ui:exp.node.emulator_state]]](#node-emulator_state)は、これをダウンさせ、また復帰させます。

出力: [[ui:exp.outputPort]]。設定なし。

```json
{ "id": "api", "type": "emulator", "x": 270, "y": 80,
  "emulator": { "name": "Orders API", "bind": "127.0.0.1:18080", "protocol": "http",
    "routes": [{ "method": "GET", "path": "/orders/:id", "order": "sequence",
                 "responses": [{ "status": 503 }, { "status": 200, "body": "{\"id\":\"{{request.params.id}}\"}" }] }] } }
```

## 障害 {#faults}

意図したタイミングで何かを壊すノードです。[[ui:exp.node.delay]] ノードとこれらのノードを、トラフィックの横の 1 つのブランチに並べると、スケジュールとして読めます。その方法は[スケジュールに沿った障害](faults.md)で説明しています。

### 劣化 {#node-impairment}

実行全体にわたる劣化リレーです。テスト対象のシステムは、実際の宛先の代わりに [[ui:exp.relayListen]] に送信（または接続）し、リレーは [[ui:exp.relayTarget]] へ転送します。応答も同じ経路で戻り、プロファイルによって劣化します。最初のステップの前に開き、実行がどのような形で終わっても終了時に閉じるので、劣化したまま残ることはありません。フローの中では、ステップはすぐに成功します。すべての判定は実行のシードから引かれるので、同じシードと同じトラフィックは、同じ運命をたどります。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.relayListen]] | `listen` | テスト対象のシステムが送信する `IP:port` | `127.0.0.1:9010` | パラメーターのみ |
| [[ui:exp.relayTarget]] | `target` | 実際の宛先の `IP:port`、または `host:port`。ホスト名は実行の開始時に解決され、見つからない名前があると、実行はこのノードで止まります | `127.0.0.1:9000` | パラメーターのみ |
| [[ui:ns.protocol]] | `protocol` | UDP（`udp`）: 各データグラムがそれぞれの運命をたどります。TCP（`tcp`）: 各接続は、宛先への専用の接続と結び付けられ、両方のストリームが劣化します | UDP（if absent は `udp`） | いいえ |
| [[ui:ns.preset]] とその下の値 | `profile` | リレーがトラフィックに何をするか。[プロファイル](#impair-profile)を参照 | [[ui:ns.preset.lan]]（if absent は劣化なし） | いいえ |

リレーの待ち受けアドレスは、実行内のほかのソケットと同じにはできず、リレー同士が輪になって転送し合うこともできません。名前で指定した宛先は、解決された後にたどられるので、名前を経由した輪は、実行の開始時に実行を止めます。レポートは、リレーの各フェーズを別々に数えます。

出力: [[ui:exp.outputPort]]。設定なし。

```json
{ "id": "relay", "type": "impairment", "x": 270, "y": 80, "listen": "127.0.0.1:9010", "target": "{{device}}",
  "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } }
```

#### プロファイル {#impair-profile}

プリセットのチップ（[[ui:ns.preset.lan]]、[[ui:ns.preset.wifi]]、[[ui:ns.preset.4g]]、[[ui:ns.preset.satellite]]、[[ui:ns.preset.intermittent]]、[[ui:ns.preset.offline]]）は、すべての値を入力します。その後で、どの値も変更できます。リレーは自分のプロトコルの値だけを読みます。ファイルでは、すべてのキーを省略できます（ゼロ、オフ）。

| フィールド | ファイル内 | 内容 | 制限 | プロトコル |
| --- | --- | --- | --- | --- |
| — | `name` | タイムラインとレポートのためのラベル: プリセットのキー（`lan`、`wifi`、`4g`、`satellite`、`intermittent`、`offline`）または独自の名前 | 最大 60 文字 | 両方 |
| [[ui:ns.offline]] | `offline` | 何も通しません | `true` / `false` | 両方 |
| [[ui:ns.latency]] | `latency_ms` | すべてのパケット、またはストリームのチャンクに加える遅延 | 0〜60,000（スライダーは 1,000 まで） | 両方 |
| [[ui:ns.jitter]] | `jitter_ms` | この値までのランダムな追加遅延。TCP のストリームは順序を保ちます | 0〜60,000（スライダーは 500 まで） | 両方 |
| [[ui:ns.rate]] | `rate_kbps` | 帯域制限。0 は制限なし。UDP: キューが 1 秒分を超えると、データグラムはスロットリングとして破棄されます。TCP: 送信側が遅くなり、何も破棄されません | 0、または 8〜10,000,000 | 両方 |
| [[ui:ns.loss]] | `loss` | データグラムが破棄される確率 | 0〜1（スライダーは % 表示） | UDP |
| [[ui:ns.burst]]、[[ui:ns.burstLength]] | `burst_start`、`burst_length` | ロスのバーストが始まる確率と、平均で何データグラム続くか | 0〜1、バーストがオンのとき 1〜1,000 | UDP |
| [[ui:ns.duplicate]] | `duplicate` | データグラムが 2 回送られる確率 | 0〜1 | UDP |
| [[ui:ns.corrupt]] | `corrupt` | データグラムの 1 ビットが反転する確率 | 0〜1 | UDP |
| [[ui:ns.reorder]] | `reorder` | データグラムが保留され、後続のものに追い越される確率 | 0〜1 | UDP |
| [[ui:ns.reset]] | `reset` | ストリームのチャンクが、代わりにその接続をリセットする確率。両側にリセットが届きます | 0〜1 | TCP |
| [[ui:ns.stall]] | `stall` | チャンクが接続をハーフオープンのままにする確率。どちら向きにも何も通らなくなり、どちらの側にも通知されません | 0〜1 | TCP |

リレー、プリセット、それらが何をモデルにしているかについては、[[[ui:nav.netsim]]](../tools/impairment.md)のページを参照してください。

### 劣化の変更 {#node-impairment_change}

実行中の [[ui:exp.node.impairment]] ノードの 1 つを、このステップ以降、別のプロファイルに切り替えます。ポートは閉じません。それまでのフェーズは閉じられ、レポートに数えられます。

| フィールド | ファイル内 | 内容 | 既定値 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.relay]] | `relay` | この実験の [[ui:exp.node.impairment]] ノードの ID | 最初のもの | いいえ |
| [[ui:ns.preset]] とその下の値 | `profile` | 今から何で劣化させるか。[プロファイル](#impair-profile)を参照。リレーは自分のプロトコルの値を読みます | [[ui:ns.preset.offline]]（if absent は劣化なし） | いいえ |

リレーが動いていない場合（たとえば中継に失敗した場合）、ステップは失敗します。出力: [[ui:exp.outputPort]]。設定なし。

```json
{ "id": "cut", "type": "impairment_change", "x": 730, "y": 200, "relay": "relay",
  "profile": { "name": "offline", "offline": true } }
```

### エミュレーターのダウン/復帰 {#node-emulator_state}

実行中のエミュレーターの 1 つをダウンさせる、または復帰させます。ダウンしている間、HTTP エミュレーターは [[ui:exp.downFault]] の指定どおりに応答します。TCP デバイスと MQTT ブローカーは接続を切って新しい接続を拒否し、OSC と UDP のデバイスは何も応答しません。復帰した後は、エミュレーターにダウンのスケジュールがあれば、それに従います。

| フィールド | ファイル内 | 内容 | 既定値 |
| --- | --- | --- | --- |
| [[ui:exp.emulatorNode]] | `emulator` | この実験の [[ui:exp.node.emulator]] ノードの ID | 最初のもの |
| [[ui:exp.emulatorDownState]] | `down` | [[ui:exp.emulatorGoesDown]]（`true`）または [[ui:exp.emulatorComesUp]]（`false`） | ダウン（if absent は `false`） |
| [[ui:exp.downFault]] | `fault` | HTTP のみ: [[ui:emu.outageFault.unavailable]]（`unavailable`）、[[ui:emu.outageFault.reset]]（`reset`: 応答なしで接続を閉じます）、[[ui:emu.outageFault.timeout]]（`timeout`: クライアントが諦めるまでリクエストを保留します。最長 120 秒） | `unavailable`（if absent も同じ） |

出力: [[ui:exp.outputPort]]。設定なし。テンプレートは使えません。

```json
{ "id": "down", "type": "emulator_state", "x": 500, "y": 200, "emulator": "api", "down": true, "fault": "unavailable" }
```

## データ {#data}

### 値の抽出 {#node-extract}

その経路上の直近の HTTP レスポンスの一部を、後のフィールド（`{{token}}`）、チェック、分岐のために変数として保存します。どの経路でも、その前に HTTP リクエストがなければなりません。[[ui:exp.sendNow]] のレスポンスで値をクリックすると、自動的に追加されます。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.variable]] | `variable` | 名前: 英字、数字、`_`。数字で始まらず、予約語やパラメーターの名前ではないこと | `token` | いいえ |
| [[ui:exp.extractFrom]] | `from` | [[ui:exp.from.json]]（`json`）、[[ui:exp.from.header]]（`header`）、[[ui:exp.from.status]]（`status`）、[[ui:exp.from.body]]（`body`）、[[ui:exp.from.regex]]（`regex`） | `json` | いいえ |
| [[ui:exp.jsonPath]]、[[ui:exp.headerName]]、[[ui:exp.pattern]] | `expr` | JSON パス（`$.data.token`、`$.items[0]`、`$["first name"]`）、ヘッダー名（大文字と小文字は区別しません）、または正規表現（最初のグループ、なければ一致全体） | `$.token`。ステータスとボディでは使われません | いいえ |

取り出すものがない場合、ステップは失敗します。ボディが JSON でない、パスまたはヘッダーがない、式が一致しない、あるいは JSON フィールドやボディ全体について、ボディが保持される 256 KiB より長かった場合です。ステータスは数値として保存され、そのほかはテキストとして、または見つかった JSON の値として保存されます。出力: [[ui:exp.outputPort]]。設定なし。

```json
{ "id": "token", "type": "extract", "x": 500, "y": 80, "variable": "token", "from": "json", "expr": "$.data.token" }
```

変数についての詳細は[データとテンプレート](data.md)にあります。

## チェック {#checks}

チェックは、成功するか、実行を失敗させます。4 つのレスポンスチェックは、その経路上の直近の HTTP レスポンスを読むので、どの経路でも、その前に HTTP リクエスト（負荷をかけたものではないもの）がなければなりません。

### HTTP ステータス {#node-assert_status}

直近のレスポンスのステータスが、指定したものとちょうど同じであれば成功します。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.expectedStatus]] | `status` | | 200。100〜599 | いいえ |

出力: [[ui:exp.outputPort]]。設定なし。

```json
{ "id": "ok", "type": "assert_status", "x": 500, "y": 80, "status": 200 }
```

### レスポンステキスト {#node-assert_body}

直近のレスポンスのボディが、そのテキストを（大文字と小文字も含めて）そのまま含んでいれば成功します。保持されるのはボディの最初の 256 KiB だけです。切り捨てられたボディでテキストが見つからなかった場合は、その理由で失敗します。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.contains]] | `contains` | | `ok`。必須 | はい |

出力: [[ui:exp.outputPort]]。設定なし。

```json
{ "id": "ready", "type": "assert_body", "x": 500, "y": 80, "contains": "ready" }
```

### レスポンスヘッダー {#node-assert_header}

直近のレスポンスにそのヘッダーがあり、その値がそのテキストを含んでいれば成功します。ヘッダー名は大文字と小文字を区別せずに照合され、値はそのまま照合されます。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.headerName]] | `name` | | `content-type`。必須 | はい |
| [[ui:exp.contains]] | `contains` | 値に含まれていなければならないもの。空の場合は、ヘッダーがあるだけで十分です | `application/json` | はい |

出力: [[ui:exp.outputPort]]。設定なし。

```json
{ "id": "json", "type": "assert_header", "x": 500, "y": 80, "name": "Content-Type", "contains": "json" }
```

### レスポンス時間 {#node-assert_latency}

直近のレスポンスに、リクエストの送信からボディの終わりまでで、この時間以内しかかからなかった場合に成功します。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.maxLatency]] | `max_ms` | | 1,000。1〜120,000 | いいえ |

出力: [[ui:exp.outputPort]]。設定なし。

```json
{ "id": "fast", "type": "assert_latency", "x": 500, "y": 80, "max_ms": 250 }
```

### 値のチェック {#node-assert_value}

値（たいていはテンプレートとして書いた変数）を期待値と比較し、比較が成り立てば成功します。

| フィールド | ファイル内 | 内容 | 既定値 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.value]] | `value` | 比較する対象: `{{token}}`、`{{reply.args[0]}}` | `{{token}}` | はい |
| [[ui:exp.operator]] | `op` | [比較](#comparisons)を参照 | [[ui:exp.op.not_empty]] | いいえ |
| [[ui:exp.expected]] | `expected` | [[ui:exp.op.empty]] と [[ui:exp.op.not_empty]] では使われません | 空（if absent も同じ） | はい |

出力: [[ui:exp.outputPort]]。設定なし。

```json
{ "id": "state", "type": "assert_value", "x": 730, "y": 80, "value": "{{state}}", "op": "eq", "expected": "ready" }
```

### 比較 {#comparisons}

[[ui:exp.node.assert_value]]、[[ui:exp.node.branch_value]]、[[ui:exp.node.loop]] の終了条件、OSC の引数ルール、HTTP の条件は、同じ方法で比較します。

| オプション | ファイル内 | 値が次のとき成り立ちます |
| --- | --- | --- |
| [[ui:exp.op.eq]] | `eq` | 期待値と等しい。両方が数値なら数値として（`200` = `200.0`）、そうでなければテキストとしてそのまま比較します |
| [[ui:exp.op.ne]] | `ne` | 同じ規則で、期待値と等しくない |
| [[ui:exp.op.lt]]、[[ui:exp.op.le]]、[[ui:exp.op.gt]]、[[ui:exp.op.ge]] | `lt`、`le`、`gt`、`ge` | より小さい、以下、より大きい、以上。両方が数値でなければなりません。そうでない場合、チェック、分岐、[[ui:exp.node.loop]] ではステップが失敗し、引数ルールや HTTP の条件では成り立ちません |
| [[ui:exp.op.contains]] | `contains` | 期待するテキストを含む |
| [[ui:exp.op.matches]] | `matches` | 期待する正規表現に一致する |
| [[ui:exp.op.empty]]、[[ui:exp.op.not_empty]] | `empty`、`not_empty` | 空である（空白だけも空とみなします）／空でない |

## フロー {#flow}

実行がどこへ進むかを決めるノードです。分岐、合流、ループの詳細は[フロー](flow.md)にあります。

### 開始 {#node-start}

実行が始まる場所です。すべての実験にちょうど 1 つあります。入力もフィールドもありません。タイムラインの最初の行に、実行のシードが表示されます。

出力: [[ui:exp.outputPort]]（必須）。ここから複数の接続線を引くと、並列ブランチがすぐに始まります。

```json
{ "id": "start", "type": "start", "x": 40, "y": 80 }
```

### 終了 {#node-end}

実行が完了する場所です。すべての実験にちょうど 1 つあり、出力はありません。複数のブランチがここに到達することがあり、実行が成功するのは、最後のブランチが終わった後に 1 回だけで、失敗したブランチがない場合に限ります。[[ui:exp.node.end]] に到達しない実行は失敗します。

```json
{ "id": "end", "type": "end", "x": 960, "y": 80 }
```

### 遅延 {#node-delay}

次のステップの前に、一定の時間待ちます。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.delayMs]] | `ms` | | 300。0〜60,000 | いいえ |

出力: [[ui:exp.outputPort]]。設定なし。もっと長く待つには、いくつか続けて置くか、[[ui:exp.node.loop]] の中に置きます。

```json
{ "id": "pause", "type": "delay", "x": 500, "y": 80, "ms": 500 }
```

### ステータス分岐 {#node-branch_status}

直近の HTTP レスポンスがこのステータスなら [[ui:exp.yes]]、そうでなければ [[ui:exp.no]] を選びます。どの経路でも、その前に HTTP リクエストがなければなりません。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.expectedStatus]] | `status` | | 200。100〜599 | いいえ |

出力: [[ui:exp.yes]] と [[ui:exp.no]]（どちらも必須）。設定なし。

```json
{ "id": "branch", "type": "branch_status", "x": 500, "y": 80, "status": 200 }
```

### 値で分岐 {#node-branch_value}

比較が成り立てば [[ui:exp.yes]]、そうでなければ [[ui:exp.no]] を選びます。フィールドと[比較](#comparisons)は[[[ui:exp.node.assert_value]]](#node-assert_value)のものと同じです。比較できない場合（テキストに対する `lt`）は、ステップが失敗します。

| フィールド | ファイル内 | 内容 | 既定値 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.value]] | `value` | 比較する対象 | `{{token}}` | はい |
| [[ui:exp.operator]] | `op` | | [[ui:exp.op.eq]] | いいえ |
| [[ui:exp.expected]] | `expected` | | 空（if absent も同じ） | はい |

出力: [[ui:exp.yes]] と [[ui:exp.no]]（どちらも必須）。設定なし。

```json
{ "id": "ok", "type": "branch_value", "x": 730, "y": 80, "value": "{{reply.args[0]}}", "op": "eq", "expected": "ok" }
```

### 並列ブランチ {#node-fork}

[[ui:exp.branch1]] と [[ui:exp.branch2]] の後ろにあるものを同時に実行し、各ブランチは変数の独自のコピーを持ちます。各出力にさらに接続線を引けば、ブランチを増やせます。フィールドはありません。

出力: [[ui:exp.branch1]] と [[ui:exp.branch2]]（どちらも必須）。

```json
{ "id": "split", "type": "fork", "x": 270, "y": 80 }
```

### ブランチの合流 {#node-join}

自分に入るすべての接続線に到達されるまで待ち、その後、ブランチの変数をマージして 1 回だけ続行します。2 つのブランチが同じ変数を設定した場合は、ファイル内で接続線が後ろにあるほうが優先されます。直近の HTTP レスポンスは、レスポンスを持っていた最後のブランチのものが引き継がれます。フィールドはありません。

ここで出会うのは、すべてが実行されるブランチだけです。[[ui:exp.node.branch_status]] の後ろに置いた [[ui:exp.node.join]] は、その [[ui:exp.yes]] と [[ui:exp.no]] が両方起こることはないので、続行されることがありません。ほかの経路が [[ui:exp.node.end]] に到達しない場合、実行はこのノードで失敗し、まだ何本のブランチを待っていたかが表示されます。

出力: [[ui:exp.outputPort]]（必須）。

ほかのノードに複数の接続線が入っている場合は、到達のたびに 1 回ずつ実行されます。

```json
{ "id": "joined", "type": "join", "x": 730, "y": 80 }
```

### ループ {#node-loop}

[[ui:exp.portBody]] の後ろにある（ここへ戻ってくる）ステップを繰り返し実行します。最大で決まった回数まで、終了条件があるときはそれが成り立つまでです。

| フィールド | ファイル内 | 内容 | 既定値と制限 | テンプレート |
| --- | --- | --- | --- | --- |
| [[ui:exp.loopMax]] | `max` | 最大の反復回数 | 5。1〜1,000 | いいえ |
| [[ui:exp.loopUntilOn]] | `until` | 任意の終了条件 `{ "value", "op", "expected" }`。[[[ui:exp.node.assert_value]]](#node-assert_value)と同じです | オフ | 値と期待値: はい |

- 本体は、少なくとも 1 回は必ず実行されます。終了条件は各反復の後に読まれるので、本体が、条件で調べる値を設定することもできます。
- [[ui:exp.portDone]] は、条件が成り立ったとき、条件がない場合は最後の反復の後に、たどられます。
- [[ui:exp.portLimit]] は、条件が成り立つ前に反復回数が尽きたときにたどられます。接続線がなければ、ステップが失敗します。
- 本体の中では、`{{counter}}` が反復の番号です。
- 本体は 1 つのブランチとして実行されます。その中の各出力には接続線が 1 本ずつあり、[[ui:exp.node.start]]、[[ui:exp.node.end]]、[[ui:exp.node.fork]]、[[ui:exp.node.join]]、ほかの [[ui:exp.node.loop]] は含められません。入口は [[ui:exp.portBody]] だけで、本体の中のすべての接続線は、本体の中でさらに進むか、ループへ戻ります。

出力: [[ui:exp.portBody]] と [[ui:exp.portDone]]（必須）、[[ui:exp.portLimit]]（任意）。設定なし。

```json
{ "id": "poll", "type": "loop", "x": 270, "y": 80, "max": 10,
  "until": { "value": "{{status.args[0]}}", "op": "eq", "expected": "ready" } }
```

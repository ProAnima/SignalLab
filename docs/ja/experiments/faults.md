---
title: 障害
description: 劣化リレーとエミュレーターを実行のノードとして使います。品質の悪いネットワークや失敗する依存先を狙ったタイミングで切り替え、レポートでフェーズごとに集計し、シードで再現できます。
---

# 障害をノードとして組み込む

ネットワークが劣化したときや依存先がダウンしたときにシステムがどう対処するかを見るには、障害を実験に組み込みます。リレーやエミュレーターは実行とともに開き、ステップが狙ったタイミングでそれを切り替え、実行レポートが各フェーズで起きたことを集計します。実行が終わると (成功、失敗、停止のいずれでも) それらは閉じられるので、実行の後に劣化が残ることはありません。

| ノード | 機能 |
| --- | --- |
| [[ui:exp.node.impairment]] | テスト対象のシステムとその宛先の間に入り、通過するものを実行全体にわたって劣化させるリレー |
| [[ui:exp.node.impairment_change]] | 実行のリレーを、このステップから別のプロファイルに切り替えます |
| [[ui:exp.node.emulator]] | Signal Lab が実行全体にわたって演じる API、デバイス、ブローカー |
| [[ui:exp.node.emulator_state]] | 実行のエミュレーターをダウンさせるか、復帰させます |

4 つとも、追加メニューの [[ui:exp.group.fault]] と [[ui:exp.group.emulate]] にあります。各フィールドは[ノードのリファレンス](nodes.md)に、リレー自体は[ネットワーク劣化](../tools/impairment.md)に、エミュレーターは[エミュレーター](../tools/emulators.md)に説明があります。

## 劣化 {#impairment}

テスト対象のシステムは、本来の宛先ではなくリレーに送信します。リレーは宛先に転送し、応答を戻し、両方向を劣化させます。

| フィールド | 内容 |
| --- | --- |
| [[ui:exp.relayListen]] | テスト対象のシステムが送信または接続する `IP:port` (ポートは 0 以外) |
| [[ui:exp.relayTarget]] | 本来の宛先の `IP:port` |
| [[ui:ns.protocol]] | UDP (データグラムごとに個別に結果が決まります) または TCP (各接続が、宛先への専用の接続 1 本とつながれます) |
| プロファイル | [[ui:ns.preset]] または独自の値 |

リレーがプロファイルのどの値を読むかはプロトコルによって決まり、それ以外の値は使われません。

| プロトコル | 劣化の種類 |
| --- | --- |
| UDP | レイテンシ、ジッター、パケットロス、バーストロス、重複、破損、順序の入れ替え、帯域制限、オフライン |
| TCP | レイテンシとジッター (ストリームの順序は保たれます)、帯域制限 (送信側が減速するだけで、何も破棄されません)、接続のリセット、ハーフオープンのままの接続、オフライン |

**最初のステップの前に開きます。** 実験のすべてのリレーは、待機のソケットと同様に実行の開始時に開くので、その [[ui:exp.relayListen]] と [[ui:exp.relayTarget]] にはテキストとパラメーターだけを使えます (`node.params_only`)。たとえばパラメーター `relay` を使った `{{relay}}` は使えますが、変数は使えません。開けないポートがあると、通信が始まる前に、そのノードで実行が停止します。

**フロー上ではすぐに通過します。** 実行がノードに到達すると、ノードはすぐに通過し、タイムラインには何をどのように劣化させるかが表示されます。リレーは、グラフ上のどこにノードがあっても、実行の開始から終了まで動作します。

**実行とともに閉じます。** 実行がどのように終わっても、リレーは閉じられます。TCP リレーの接続もそれとともに閉じられます。リレーが自ら中継を停止した場合はその理由が保持され、それを使うステップはその理由で失敗し、レポートにもそう記録されます。

::: tip 劣化を経由させる
[[ui:exp.node.osc]] または [[ui:exp.node.udp]] ノードで [[ui:exp.routeThrough]] を使うと、その前に劣化ノードが置かれます。リレーは `127.0.0.1` の空いているポートで待ち受け、[[ui:ns.preset.lan]] プリセットでノードの宛先に転送し、ノードはそれ以降リレーに送信します。
:::

## 劣化の変更 {#change-impairment}

[[ui:exp.node.impairment_change]] は、[[ui:exp.relay]] で実験のリレーの 1 つを指定し、そのステップ以降の劣化のプロファイルを与えます。リレーはポートと接続を保ったまま、次のパケットまたはチャンクから新しい値を適用します。タイムラインには新しいプロファイルが表示されます。

変更のたびに 1 つの**フェーズ**が終わります。実行レポートには、リレーごとに次のものが保持されます。

- 待ち受けアドレスと宛先アドレス、TCP の場合はそのプロトコル。
- 全体の集計: 受信、転送、破棄、帯域制限、重複、破損、順序入れ替え、バイト数。TCP の場合は、接続数、リセットされた接続、ハーフオープンのままの接続も。
- 各フェーズ: プロファイルの名前、リレーが開いた時点からのミリ秒で表した開始と終了の時刻、そのフェーズだけの同じ集計。

パケットは、その結果を決めたフェーズで数えられます。遅延させたコピーが切り替えの後に送り出された場合も同じです。リレーは最新 1000 個のフェーズを保持し、それより古いものは集計されますが保持されません。

実験のリレーを指定していない [[ui:exp.node.impairment_change]] は拒否されます (`impair.relay_unknown`)。

## エミュレーター {#emulator}

[[ui:exp.node.emulator]] は、実行全体にわたって依存先を演じます。HTTP API、OSC・UDP・TCP デバイス、MQTT ブローカーです。[エミュレーター](../tools/emulators.md)画面が単独で実行するものと同じエミュレーターで、[[ui:emu.edit]] でルールを開き、[[ui:emu.toLibrary]] でライブラリにコピーを保存し、[[ui:emu.fromLibrary]] でライブラリからコピーを取り込みます。

- 最初のステップの前に開き、実行が終わるまで応答します。開けないポートがあると、通信が始まる前に実行が停止します。フロー上ではすぐに通過します。
- アドレスはリテラルの `IP:port` です。一致条件のパターンにはパラメーターだけを使えます。応答は、届いたもの (`{{request.…}}`) と実行のパラメーターを読むテンプレートです。シークレットは読み取れません。
- ランダムな選択 (重み付きのレスポンスの組み合わせ、遅延のジッター、応答内のジェネレーター) は、実行のシードから抽選されます。
- HTTP エミュレーターは、同じアドレスの [[ui:exp.node.wait_http]] が待ち受ける対象でもあります。この待機で、テスト対象のシステムが送ったものを確認します。そこにエミュレーターがない場合は、実行自身のリスナーがすべてのリクエストに `204` で応答します。
- OSC や UDP のエミュレーターは、そのポートで待ち受ける実行の待機とポートを共有し、両方がすべてのデータグラムを受け取ります。
- MQTT エミュレーターは、実行の [[ui:exp.node.mqtt]] ノードと [[ui:exp.node.wait_mqtt]] ノードが、他のブローカーと同じように使えるブローカーです。

実行レポートには、エミュレーターノードごとに、名前、プロトコル、アドレスと、その集計が保持されます。リクエストの合計、どのルールにも一致しなかったもの、失敗したもの、ダウン中に届いたもの、ブローカーが遅いクライアントに配送できなかったメッセージ、各ルールのヒット数です。

## エミュレーターのダウンと復帰 {#emulator-state}

[[ui:exp.node.emulator_state]] は、[[ui:exp.emulatorNode]] で実行のエミュレーターの 1 つを指定します。[[ui:exp.emulatorDownState]] は [[ui:exp.emulatorGoesDown]] または [[ui:exp.emulatorComesUp]] です。ダウン中は次のようになります。

| エミュレーター | 受けるもの |
| --- | --- |
| HTTP | [[ui:exp.downFault]] の設定に従います: [[ui:emu.outageFault.unavailable]] (`503`、ボディ `{"error":"unavailable"}`)、[[ui:emu.outageFault.reset]]、または [[ui:emu.outageFault.timeout]] (クライアントがあきらめるまで、最大 120 秒間リクエストが保留されます) |
| TCP デバイス、MQTT ブローカー | 接続が切断され、新しい接続は拒否されます |
| OSC、UDP デバイス | 何も応答しません |

ダウン中に届いたものは `down` として数えられ、どのルールにも一致しなかったリクエストとして数えられることはありません。[[ui:exp.node.wait_http]] は引き続きリクエストを受け取ります。エミュレーターは、自身のダウンのスケジュールに関係なく、ステップが復帰させるまでダウンしたままです。どちらの場合も、実行の終了とともに閉じられます。

実験のエミュレーターを指定していない [[ui:exp.node.emulator_state]] は拒否されます (`emulator.node_unknown`)。

## スケジュールによるダウン {#outage}

エミュレーターは自らダウンすることもできます。ルールで [[ui:emu.outage]] にチェックを入れ、[[ui:emu.outageUp]] と [[ui:emu.outageDown]] (それぞれ 10–3,600,000 ms)、HTTP の場合は [[ui:emu.outageFault]] を設定します。開いた時点 (実行の場合は最初のステップの前) から数えて、前者の時間だけ応答し、後者の時間だけダウンし、それを繰り返します。スケジュールによるダウン中、HTTP エミュレーターの `503` には、復帰までの秒数 (整数、最低 1) を示す `Retry-After` が付きます。[[ui:exp.node.emulator_state]] ステップでダウンさせている間の `503` には付きません。それがいつ終わるかは誰にもわからないからです。

スケジュールにステップは不要で、ステップにスケジュールは不要です。不安定な依存先にはスケジュールを、フローの選んだ時点でのダウンにはステップを使います。

## 例: 遅い回線の先でのダウン {#example}

クライアントがリレーを経由して API に注文を問い合わせます。問い合わせている間に、2 つ目のブランチが回線を [[ui:ns.preset.4g]] に遅くし、API を 2 秒間ダウンさせ、復帰させてから、回線を再びクリーンにします。クライアントは応答を得るまで問い合わせ続ける必要があります。

```text
start → orders → link → split
split ─ branch1 → settle → until_ok ─ done → answered → joined
                           until_ok ─ body → get → status → pause → until_ok
split ─ branch2 → slow → down → outage → up → clean → joined
joined → end
```

名前は、下のファイルにあるノードの ID です。

1. パラメーター `api` = `http://127.0.0.1:18091` を追加します。API ではなく、リレーのアドレスです。
2. [[ui:exp.node.emulator]] を追加します: HTTP、`127.0.0.1:18090`、`GET /orders/:id` に `200` と `{"order":"{{request.params.id}}"}` で応答するルート。
3. その後に [[ui:exp.node.impairment]] を置きます: [[ui:exp.relayListen]] `127.0.0.1:18091`、[[ui:exp.relayTarget]] `127.0.0.1:18090`、[[ui:ns.protocol]] TCP、プリセット [[ui:ns.preset.lan]]。
4. その後に [[ui:exp.node.fork]] を置きます。
5. [[ui:exp.branch1]] にはクライアントを置きます: 300 ms の [[ui:exp.node.delay]]、続いて [[ui:exp.node.loop]] ([[ui:exp.loopMax]] 40、[[ui:exp.loopUntilOn]]: `{{status}}` が `200` [[ui:exp.op.eq]])。その本体は、[[ui:exp.node.http]] `GET {{api}}/orders/42`、[[ui:exp.from.status]] を `status` に保存する [[ui:exp.node.extract]]、250 ms の遅延で、最後はループに戻します。[[ui:exp.portDone]] には [[ui:exp.node.log]] `Orders API answers again: HTTP {{status}}` をつなぎます。
6. [[ui:exp.branch2]] には障害を置きます: リレーを [[ui:ns.preset.4g]] に切り替える [[ui:exp.node.impairment_change]]、Orders API を [[ui:emu.outageFault.unavailable]] で [[ui:exp.emulatorGoesDown]] の状態にする [[ui:exp.node.emulator_state]]、2000 ms の遅延、API を [[ui:exp.emulatorComesUp]] の状態に戻すもう 1 つの [[ui:exp.node.emulator_state]]、リレーを [[ui:ns.preset.lan]] に戻すもう 1 つの [[ui:exp.node.impairment_change]]。
7. 両方のブランチを [[ui:exp.node.join]] につなぎ、合流を [[ui:exp.node.end]] につなぎます。
8. 実行します。

タイムラインには、遅い回線を経由したクライアントのリクエストに `503` が返され、API が復帰し、その後 `200` が返ってループが [[ui:exp.portDone]] から抜ける様子が表示されます。レポートには、ダウン中の API に届いた 5 件ほどのリクエストと、ルートが応答した 1 件、そしてリレーの 3 つのフェーズ (一瞬の [[ui:ns.preset.lan]]、ダウン中の [[ui:ns.preset.4g]]、再び [[ui:ns.preset.lan]]) が、それぞれの通信とともに集計されます。

::: details ファイルとしての実験
`.json` ファイルとして保存し、[[ui:exp.documents]] の [[ui:exp.importJson]] で開きます。

```json
{
  "version": 9,
  "name": "Outage behind a slow link",
  "params": [{ "name": "api", "value": "http://127.0.0.1:18091" }],
  "profiles": [],
  "profile": null,
  "seed": null,
  "nodes": [
    { "id": "start", "type": "start", "x": 40, "y": 270 },
    { "id": "orders", "type": "emulator", "x": 260, "y": 270,
      "emulator": { "name": "Orders API", "bind": "127.0.0.1:18090", "protocol": "http",
        "routes": [{ "method": "GET", "path": "/orders/:id", "when": [], "order": "sequence",
          "responses": [{ "status": 200, "headers": [], "body": "{\"order\":\"{{request.params.id}}\"}", "delay_ms": 0, "jitter_ms": 0, "fault": "none", "weight": 1 }] }],
        "fallback": null } },
    { "id": "link", "type": "impairment", "x": 490, "y": 270, "listen": "127.0.0.1:18091", "target": "127.0.0.1:18090", "protocol": "tcp",
      "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } },
    { "id": "split", "type": "fork", "x": 720, "y": 270 },
    { "id": "settle", "type": "delay", "x": 950, "y": 140, "ms": 300 },
    { "id": "until_ok", "type": "loop", "x": 1180, "y": 140, "max": 40,
      "until": { "value": "{{status}}", "op": "eq", "expected": "200" } },
    { "id": "get", "type": "http", "x": 1410, "y": 20,
      "request": { "method": "GET", "url": "{{api}}/orders/42", "headers": [], "body": null, "timeout_ms": 3000 } },
    { "id": "status", "type": "extract", "x": 1640, "y": 20, "variable": "status", "from": "status", "expr": "" },
    { "id": "pause", "type": "delay", "x": 1870, "y": 20, "ms": 250 },
    { "id": "answered", "type": "log", "x": 1410, "y": 140, "message": "Orders API answers again: HTTP {{status}}" },
    { "id": "slow", "type": "impairment_change", "x": 950, "y": 400, "relay": "link",
      "profile": { "name": "4g", "latency_ms": 60, "jitter_ms": 25, "rate_kbps": 20000 } },
    { "id": "down", "type": "emulator_state", "x": 1180, "y": 400, "emulator": "orders", "down": true, "fault": "unavailable" },
    { "id": "outage", "type": "delay", "x": 1410, "y": 400, "ms": 2000 },
    { "id": "up", "type": "emulator_state", "x": 1640, "y": 400, "emulator": "orders", "down": false, "fault": "unavailable" },
    { "id": "clean", "type": "impairment_change", "x": 1870, "y": 400, "relay": "link",
      "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } },
    { "id": "joined", "type": "join", "x": 2100, "y": 270 },
    { "id": "end", "type": "end", "x": 2330, "y": 270 }
  ],
  "edges": [
    { "from": "start", "to": "orders" },
    { "from": "orders", "to": "link" },
    { "from": "link", "to": "split" },
    { "from": "split", "to": "settle", "port": "branch1" },
    { "from": "split", "to": "slow", "port": "branch2" },
    { "from": "settle", "to": "until_ok" },
    { "from": "until_ok", "to": "get", "port": "body" },
    { "from": "get", "to": "status" },
    { "from": "status", "to": "pause" },
    { "from": "pause", "to": "until_ok" },
    { "from": "until_ok", "to": "answered", "port": "done" },
    { "from": "answered", "to": "joined" },
    { "from": "slow", "to": "down" },
    { "from": "down", "to": "outage" },
    { "from": "outage", "to": "up" },
    { "from": "up", "to": "clean" },
    { "from": "clean", "to": "joined" },
    { "from": "joined", "to": "end" }
  ]
}
```
:::

2 つのテンプレートが [[ui:exp.documents]] にあり、別の方法で同じことを行います。[[ui:exp.templateFaults]] は、クリーン、ロスあり、オフライン、再びクリーンと切り替わる UDP リレーを経由して、エミュレートされたデバイスにデータグラムを送ります。[[ui:exp.templateOutage]] は、クライアントが問い合わせ続ける間に、エミュレートされた API を 2 秒間ダウンさせます。

## ポート {#ports}

1 つの実行のソケット (待機、応答、エミュレーター、リレー) は、同じプロトコルのポートを共有できません。UDP と TCP のソケットは同じ番号を使えます。リレーの [[ui:exp.relayListen]] では、`0.0.0.0` 上のアドレスは同じポートのすべてのアドレスと衝突します。

| ソケット | ポートを共有できない相手 |
| --- | --- |
| HTTP、TCP、MQTT のエミュレーター | これらのうちの別のエミュレーター (`emulator.bind_taken`) |
| OSC、UDP のエミュレーター | これらのうちの別のエミュレーター (`emulator.bind_taken`) |
| TCP、MQTT のエミュレーター | [[ui:exp.node.wait_http]] (`emulator.bind_taken`) |
| UDP リレーの [[ui:exp.relayListen]] | 別の UDP リレー、OSC または UDP のエミュレーター、待機または応答のソケット (`impair.bind_taken`) |
| TCP リレーの [[ui:exp.relayListen]] | 別の TCP リレー、HTTP・TCP・MQTT のエミュレーター、[[ui:exp.node.wait_http]] (`impair.bind_taken`) |

意図的に共有されるもの: HTTP エミュレーターとそのアドレスの [[ui:exp.node.wait_http]] ステップ、OSC または UDP のエミュレーターとそのポートの待機、同じアドレスの待機どうし。

リレーが、直接または他のリレーを経由して、自分自身に転送することはできません。その通信がループバック上で循環してしまうからです (`impair.loop`)。デバイスの前に 2 つのリレーを直列に置くのは問題ありません。

## 障害のある実行を再現する {#seed}

リレーが行うすべての判定 (パケットを失うか、重複させるか、破損させるか、保留するか、どれだけのジッターを加えるか) は、実行のシードから、方向ごとに別々に、パケットごとに抽選されます。エミュレーターのランダムな選択も、このシードから抽選されます。同じシードと同じ通信でもう一度実行すれば、同じパケットが同じ結果になります。一度見えた失敗は、もう一度見ることができます。

シードを保持するには、タイムラインでシードの横にある [[ui:exp.pinSeed]] を押すか、[[ui:exp.runWith]] からそのシードで実行します。[シード](runs.md#seeds)を参照してください。シードでも固定できないのはタイミングです。テスト対象のシステムがいつ送信するか、つまりパケットがどのフェーズに入るかは固定できません。

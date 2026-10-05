---
title: 負荷テスト
description: HTTP ノードのリクエストを負荷プロファイル (一定、ランプ、段階、スパイク、ランダムな到着) に従って送信し、レイテンシ、エラー、達成したレートを計測して、しきい値で判定し、2 つの実行を比較します。
---

# HTTP リクエストの負荷テスト

[[ui:exp.node.http]] ノードは、1 秒あたりのリクエスト数のプロファイルに従い、同時に多数のリクエストを何度も送信して、返ってきたものを計測できます。計測するのは、レイテンシのパーセンタイル、エラー、達成したレートです。ステップの成否はしきい値で決まり、[[ui:exp.compare]] でその数値を以前の実行の数値と並べて表示できます。

負荷はノードの設定であり、独立したノードではありません。実験の残りの部分 (エミュレーター、劣化リレー、他のブランチ) は、その周りでいつもどおり実行されます。

## リクエストに負荷をかける {#turn-on}

1. [[ui:exp.node.http]] ノードを選択し、リクエストを入力します。
2. そのプロパティで [[ui:exp.loadOn]] をオンにします。
3. [[ui:exp.loadShape]] とその数値を選びます。その下のグラフ [[ui:exp.loadChart]] にレートが描かれ、合計で何件のリクエストを何秒間で送るかが表示されます。
4. [[ui:exp.loadConcurrency]] を設定します。同時に処理中にできるリクエストの数です。
5. [[ui:exp.thresholds]] を追加または変更します。
6. 実験を実行します。

負荷の初期設定は、30,000 ms かけて毎秒 0 から 100 リクエストまで上げる [[ui:exp.loadShape.ramp]]、同時に 32 件、しきい値 2 つ ([[ui:exp.metric.p95_ms]] < 500 ms と [[ui:exp.metric.error_rate]] < 1%) です。

**負荷は繰り返しと再試行の代わりになります。** 負荷をオンにするとそれらはオフになり、負荷とそのどちらかを併用したノードは拒否されます (`node.load_alone`)。失敗したリクエストは数えられるだけで、再試行はされません。負荷をかけて実行できるのは HTTP リクエストだけです (`node.load_unsupported`)。

**リクエストは 1 回だけ読み取られます。** テンプレートはステップの開始時に解決されるので、負荷のすべてのリクエストは同じものになります。`{{counter}}` と `{{uuid}}` は、すべてのリクエストで同じ 1 つの値になります。[テンプレート](data.md#templates)を参照してください。

**負荷全体で 1 つのクライアントです。** [[ui:exp.cookies]] がオンのとき、リクエストは実行の Cookie 保管庫を共有し、Digest の認証状態も 1 つを共有するので、1 回のチャレンジへの応答がすべてのリクエストに使われます。各リクエストにはノード自身のタイムアウトが適用されます。

**インスペクターにはサンプルが送られます。** 送られるのは最大で 100 ms ごとに 1 件のやり取りなので、負荷が [[ui:dock.inspector]] をあふれさせることはありません。

## プロファイル {#profiles}

| [[ui:exp.loadShape]] | 設定 | 時間に対するレート |
| --- | --- | --- |
| [[ui:exp.loadShape.constant]] | [[ui:exp.loadRate]]、[[ui:exp.loadDuration]] | 全体を通して同じレート |
| [[ui:exp.loadShape.ramp]] | [[ui:exp.loadFrom]]、[[ui:exp.loadTo]]、[[ui:exp.loadDuration]] | 一方のレートからもう一方へ直線的に変化 |
| [[ui:exp.loadShape.steps]] | [[ui:exp.loadFrom]]、[[ui:exp.loadStepBy]]、[[ui:exp.loadEvery]]、[[ui:exp.loadSteps]] | 最初のレートから、段階ごとに増分 1 つずつ上がり、各段階は同じ時間 |
| [[ui:exp.loadShape.spike]] | [[ui:exp.loadBase]]、[[ui:exp.loadPeak]]、[[ui:exp.loadAt]]、[[ui:exp.loadSpikeFor]]、[[ui:exp.loadDuration]] | ベースレート、指定した時点からしばらくピーク、その後再びベースレート |
| [[ui:exp.loadShape.poisson]] | [[ui:exp.loadRate]]、[[ui:exp.loadDuration]] | ランダムな到着、レートは平均値 |

プロファイルの形を切り替えると、引き継げるものは保持されます。実行する時間と、到達する最高のレートです。

### 制限 {#limits}

| 設定 | 範囲 |
| --- | --- |
| [[ui:exp.loadShape.constant]] と [[ui:exp.loadShape.poisson]] の [[ui:exp.loadRate]]、[[ui:exp.loadPeak]] | 0.1–100,000 リクエスト/秒 |
| [[ui:exp.loadFrom]]、[[ui:exp.loadTo]]、[[ui:exp.loadBase]] | 0–100,000 リクエスト/秒 |
| [[ui:exp.loadShape.steps]] のすべての段階 (最後の段階を含む) | 0–100,000 リクエスト/秒。増分は負の値でもかまいません |
| [[ui:exp.loadDuration]]、[[ui:exp.loadEvery]] | 100–300,000 ms |
| [[ui:exp.loadSteps]] | 1–100。すべての段階の合計は最大 300,000 ms |
| スパイク | 0 ms より長く、継続時間の終わりまでに終わること |
| [[ui:exp.loadConcurrency]] | 1–512 |
| [[ui:exp.thresholds]] | 最大 16 個。各値は 0 以上の数値 |

リクエストが 1 件にもならないプロファイルは拒否されます (`load.nothing_planned`)。レートの範囲は [HTTP バースト](../protocols/http.md)と同じです。

プロファイルは実行全体と同じ 300 秒まで続けられます。ただし、実行の[制限時間](flow.md#limit)はすべてのステップを数えるので、実験の残りの部分のための余裕を残してください。

### リクエストの件数 {#planned}

プロファイルのリクエスト数は、レートを時間で合計したものです。

| プロファイル | リクエスト数 |
| --- | --- |
| [[ui:exp.loadShape.constant]]、100/秒で 1000 ms | 100 |
| [[ui:exp.loadShape.ramp]]、2000 ms で 0 → 100/秒 | 100 |
| [[ui:exp.loadShape.steps]]、10/秒から 10/秒ずつ増加、1000 ms の段階が 3 つ | 60 (10 + 20 + 30) |
| [[ui:exp.loadShape.spike]]、10/秒、1000 ms の時点から 500 ms 間 100/秒、全体で 2000 ms | 65 |
| [[ui:exp.loadShape.poisson]]、200/秒で 10,000 ms | 平均 2000 |

## スケジュール {#schedule}

n 番目のリクエストは、プロファイルのカウントが n に達した時点に予定されます (最初のリクエストはすぐに送られます)。各時点は負荷の開始から計算されるので、起床が遅れても後続のリクエストがずれることはなく、プロファイルが表すレートがそのまま要求されるレートになります。

[[ui:exp.loadShape.poisson]] は、到着の間隔を実行のシードからランダムに抽選します。同じシードなら同じ時点になるので、ランダムな負荷も正確に再現できます。[シード](runs.md#seeds)を参照してください。

**取りこぼしたリクエスト。** 同時に処理中にできるリクエストは、[[ui:exp.loadConcurrency]] で設定した数までです。そのすべてがまだ応答を待っているとき、次のリクエストは空きを待ちます。予定時刻から 50 ms 以上遅れて送ることになる場合、そのリクエストは遅れて送られることはなく、スキップされます。その間に予定時刻を迎えた他のリクエストとともに取りこぼしとして数えられ、負荷はまだ予定どおりに送れる最初のリクエストから続行されます。取りこぼしが多い場合は、サーバーまたは [[ui:exp.loadConcurrency]] がプロファイルについていけなかったことを意味します。

## 実行中 {#progress}

タイムラインには 1 秒に 1 回、ステップが [[ui:exp.load]] として表示され、経過秒数、送信したリクエスト数、直近 1 秒のレート、その時点までの p95、失敗したリクエスト数が示されます。[[ui:common.stop]] を押すと負荷はすぐに終了し、処理中のリクエストは破棄されます。他のブランチで失敗が起きた場合は、1 秒以内に負荷が終了します。

## 計測されるもの {#metrics}

最後の応答の後、ステップは計測結果を持ちます。計測結果は、タイムラインの最後のイベントと[実行レポート](runs.md#report)に保持されます。

| 計測値 | 内容 |
| --- | --- |
| planned | プロファイルの合計リクエスト数 ([[ui:exp.loadShape.poisson]] の場合は平均値) |
| sent | 応答を受け取ったか失敗したリクエスト |
| ok | 2xx のステータスで応答されたもの |
| failed | それ以外のステータス、または応答がまったくないもの |
| missed | すべての枠が使用中のときに予定時刻を迎え、スキップされたもの |
| rps | 1 秒あたりに送信したリクエスト数: sent ÷ プロファイルの継続時間。最後のリクエストが送られた時刻のほうが遅い場合は、sent ÷ そこまでの時間 |
| error_rate | sent に対する failed の割合 (%) |
| min, mean, max | 最も速いリクエスト、平均、最も遅いリクエスト (ms) |
| p50, p90, p95, p99 | リクエストの 50、90、95、99% がそれ以下に収まったレイテンシ (ms) |
| received_bytes | 受信したボディの合計バイト数 |
| statuses | ステータス別 (`200`、`503`) のリクエスト数。ステータスがない場合は原因別 (`timeout`、`refused`、`reset` …) |
| seconds | プロファイルの 1 秒ごとの、送信数、失敗数、その平均レイテンシ |
| histogram | レイテンシ別のリクエスト数: 1、2、5、10、20、50、100、200、500、1000、2000、5000、10,000 ms 以下、およびそれより遅いもの |

リクエストのレイテンシは、送信してから応答全体を読み終えるまでの時間です。失敗したリクエストは、失敗するまでにかかった時間で数えられます。パーセンタイルは幅 1% の対数バケットから読み取られ、負荷がどれだけ長く続いても、真の値との差は 0.5% 以内です。

## しきい値 {#thresholds}

しきい値は、[[ui:exp.thresholdMetric]]、[[ui:exp.thresholdOp]]、[[ui:exp.thresholdValue]] からなる 1 行です。[[ui:exp.thresholdAdd]] で追加します。

| [[ui:exp.thresholdMetric]] | 単位 |
| --- | --- |
| [[ui:exp.metric.p50_ms]]、[[ui:exp.metric.p90_ms]]、[[ui:exp.metric.p95_ms]]、[[ui:exp.metric.p99_ms]] | ms |
| [[ui:exp.metric.mean_ms]]、[[ui:exp.metric.max_ms]] | ms |
| [[ui:exp.metric.error_rate]] | 送信したリクエストに対する % |
| [[ui:exp.metric.rps]] | 達成した 1 秒あたりのリクエスト数 |
| [[ui:exp.metric.missed]] | リクエスト数 |

[[ui:exp.thresholdOp]] は `<`、`≤`、`>`、`≥` のいずれかです。よく使われるものをいくつか挙げます。

| [[ui:exp.thresholdMetric]] | [[ui:exp.thresholdOp]] | [[ui:exp.thresholdValue]] | ステップが失敗する条件 |
| --- | --- | --- | --- |
| [[ui:exp.metric.p95_ms]] | `<` | 300 | 20 件に 1 件以上のリクエストが 300 ms 以上かかった |
| [[ui:exp.metric.error_rate]] | `<` | 1 | リクエストの 1% 以上が失敗した |
| [[ui:exp.metric.rps]] | `≥` | 180 | サーバーが毎秒 180 リクエストを処理できなかった |
| [[ui:exp.metric.missed]] | `≤` | 0 | 1 件でもリクエストをスキップする必要があった |

ファイルでは、しきい値は `{ "metric": "p95_ms", "op": "lt", "value": 300 }` のように書きます。指標は `p50_ms`、`p90_ms`、`p95_ms`、`p99_ms`、`mean_ms`、`max_ms`、`error_rate`、`rps`、`missed`、比較は `lt`、`le`、`gt`、`ge` です。

しきい値は最後の応答の後に、並んでいる順に読み取られます。満たされない最初のしきい値でステップは失敗し (`load.threshold`)、そのメッセージにはしきい値と計測値が示され、実行もそれとともに失敗します。しきい値がなければ、負荷は何を計測しても成功します。他のブランチの失敗によって負荷が早く終わった場合は、しきい値ではなく、その失敗が実行の失敗になります。

## 結果 {#result}

ステップが成功すると、タイムラインにその概要が表示されます。リクエスト数、レート、p95、失敗の割合です。ノードを選択すると、そのプロパティに [[ui:exp.loadResult]] が表示されます。

- 各しきい値: ✓ [[ui:exp.thresholdHeld]] または ✕ [[ui:exp.thresholdBroken]] と、その計測値。
- [[ui:http.sent]]、[[ui:exp.loadRps]]、[[ui:exp.loadErrors]] (その割合とともに)、[[ui:http.missed]]。
- [[ui:http.p50]]、[[ui:http.p90]]、[[ui:http.p95]]、[[ui:http.p99]]、[[ui:http.avg]]、[[ui:http.max]]。
- [[ui:exp.loadPerSecond]]: 各秒のリクエスト数 (失敗したものは赤) と、その平均レイテンシの線。
- [[ui:exp.loadLatencies]]: どれだけの時間がかかったリクエストが何件あったか。
- ステータスと原因、それぞれの件数。

コマンドラインも同じ数値と各しきい値の判定を出力します。[`signallab run`](../automation/cli.md#cli-run) を参照してください。

## 2 つの実行を比較する {#compare}

1. 実験を 2 回以上実行します。
2. タイムラインで [[ui:exp.compare]] を押します。このボタンは実行がレポートを保存すると現れ、実行中は無効になります。
3. 最新の実行が [[ui:exp.compareAfter]]、その前の実行が [[ui:exp.compareBefore]] になります。どちらの一覧でも別の実行を選べます。

一覧には、データフォルダーのレポートから、この実験 (名前で識別されます) の実行が新しいものから順に最大 50 件表示されます。それぞれに日時、終わり方、シードが示されます。コマンドラインからの実行も、同じデータフォルダーを使っていれば含まれます。実験の名前を変更すると、新しい履歴が始まります。

各負荷ステップについて (ノードで対応付けられます)、表にすべての指標の [[ui:exp.compareBefore]]、[[ui:exp.compareAfter]]、[[ui:exp.compareChange]] が、単位と % で示されます。悪い方向への 5% 以上の変化 (遅くなった、エラーが増えた、取りこぼしが増えた、レートが下がった) は悪化として赤で示されます。ゼロから何かが出た場合も悪化に含まれます。表の下には、両方の実行における各しきい値の判定が表示されます。一方の実行にしかない負荷ステップには、変化なしで [[ui:exp.compareOnlyBefore]] または [[ui:exp.compareOnlyAfter]] と表示されます。負荷ステップのない実行では [[ui:exp.compareNoLoad]] と表示されます。

スクリプトからは、[`experiment_runs`](../api/commands.md#experiment_runs) で実行を一覧表示し、[`experiment_compare`](../api/commands.md#experiment_compare) でレポートのファイル名を指定して 2 つの実行を比較できます。`signallab mcp` は同じ機能をアシスタントに提供します ([MCP](../automation/mcp.md))。

## 負荷の後のチェック {#checks-after}

負荷は独自のレスポンスを残しません。計測されるだけで、チェックはされません。その後に置くチェックや [[ui:exp.node.extract]] には、すべてのパスで、その前に負荷なしの別のリクエストが必要です。そうでなければ実験は実行されません (`graph.needs_http`)。負荷がかかった状態の API の応答を 1 つチェックするには、負荷の後に通常の [[ui:exp.node.http]] を置くか、その横の並列ブランチに置きます。

負荷をかけたノードで [[ui:exp.sendNow]] を使うと、リクエストを 1 回だけ送信します。

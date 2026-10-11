---
title: ファイルとフォルダー
description: Signal Lab がファイルを保存する場所 (データフォルダー、実験、シグナルとエミュレーターのライブラリ、実行レポート、エクスポート)、その形式、そして編集、バックアップ、移動して安全なもの。
---

# ファイルとフォルダー

Signal Lab が保持するものは、すべて 1 つのフォルダー (データフォルダー) にある、プレーンな JSON (またはテキスト) です。シークレットの値は、そこには入りません。

## データフォルダー {#data-folder}

| Signal Lab の動作場所 | データフォルダー |
| --- | --- |
| デスクトップアプリ、Windows | ユーザーフォルダー内の `Documents\SignalLab`: `C:\Users\<you>\Documents\SignalLab` |
| デスクトップアプリ、Linux | `~/Documents/SignalLab` |
| サーバー | `--data-dir` または `SIGNALLAB_DATA_DIR`。どちらもなければ、実行ユーザーのホームフォルダーの `Documents/SignalLab` |
| サーバー、Docker イメージ | `/data`。ボリューム (compose ファイルでは `signallab-data`) |
| `signallab run` | 一時フォルダー。終了時に削除されます。ただし、`--data-dir` で指定した場合を除きます |

デスクトップアプリも、環境に `SIGNALLAB_DATA_DIR` が設定されていれば、それを使います。フォルダーは、最初に何かが書き込まれるときに作られます。

::: tip
Windows では、ドキュメントを別の場所 (OneDrive) に置いている場合でも、アプリはユーザーフォルダー直下の `Documents` フォルダーを使います。
:::

サーバーでは、ファイルはサーバーのマシンに書き込まれ、あなたのマシンには書き込まれません。ヘッダーの [[ui:app.server]] バッジのツールチップに、その場所が表示されます。API では [`app_info`](../api/commands.md#app_info) の `data_dir` として得られ、[`/api/files`](../api/index.md#files) からフォルダー内のものをダウンロードできます。

コマンドラインの `signallab emulate`、`signallab send`、`signallab mcp` は、デスクトップアプリと同じフォルダーからアプリのライブラリを読み込みます。

## 中身 {#contents}

| ファイル | 内容 | 書き込まれるタイミング |
| --- | --- | --- |
| `experiment.json` | エディターで開いている実験 | 変更のたびに、少し後 |
| `signals.json` | シグナルライブラリ ([[ui:nav.signals]]) | 変更のたびに、少し後 |
| `emulators.json` | エミュレーターライブラリ ([[ui:nav.emulators]]) | 変更のたびに、少し後 |
| `runs/run-<ms>-<job>.json` | 自然に終了した実行ごとのレポート | 実行の終了時 |
| `exports/experiment-<ms>-<16 hex digits>.json` | 実験のスナップショット | [[ui:exp.exportJson]] |
| `capture-<ms>.jsonl`、`capture-<ms>.txt` | インスペクターのフレーム | [[ui:ins.exportJsonl]]、[[ui:ins.exportTxt]] |
| `token` | サーバーのアクセストークン。そのユーザーだけが読めます | 最初の起動時の `--generate-token` |
| `.experiment-<hex>.tmp`、`.signals-<hex>.tmp`、`.emulators-<hex>.tmp` | 保存の途中のもの | 一瞬だけ存在し、その後名前が変わります |

`<ms>` は 1970 年からのミリ秒で表した時刻、`<job>` は実行のジョブ番号です。サーバーでは、どのブラウザーも、同じ `experiment.json`、同じライブラリ、同じレポートを使って作業します。

## 形式 {#formats}

すべて UTF-8 の JSON で、読みやすく差分も取りやすいようにインデント付きで書き込まれます。それぞれに `version` があります。古いバージョンのファイルは、開くときに読み込まれて移行され、次に保存されるときに現在のバージョンで書き戻されます。そのあとは、古い Signal Lab では開けません。この Signal Lab が知っているより新しいバージョンのファイルは拒否され (`doc.version_unsupported`)、そのまま残されます。無理に読み込むと、新しいバージョンが追加した内容が次の保存で失われてしまうためです。これは実験と 2 つのライブラリのすべてに当てはまります。

### experiment.json {#experiment-json}

バージョン 9 の実験ドキュメントです。[[ui:exp.exportJson]] が書き出し、[[ui:exp.importJson]] が読み込むものと同じ JSON です。

```json
{
  "version": 9,
  "name": "HTTP check",
  "params": [],
  "profiles": [],
  "profile": null,
  "seed": null,
  "cookies": true,
  "nodes": [ { "id": "start", "type": "start", "x": 40, "y": 80 }, … ],
  "edges": [ { "from": "start", "to": "request", "port": "next" }, … ]
}
```

- 最大 4 MiB で、ノードは 1〜64 個です (`doc.node_count`)。
- バージョン 1〜8 は、開くときに移行されます。バージョン 8 より前のファイルは `cookies` がオフの状態で開くので、以前と同じように実行されます。各バージョンで追加された設定 (2 でパラメーター、3 でプロファイル、4 で再試行、5 で繰り返しとループ、6 でエミュレーター、7 で劣化、8 で WebSocket と HTTP 認証、9 で負荷) は、空の状態から始まります。
- この Signal Lab が知っているより新しいバージョンは、読み取れないものを欠いたまま開くのではなく、拒否されます (`doc.version_unsupported`)。
- 解析できないファイルは、パス、行、列とともに報告され、置き換えられることはありません。
- 一時ファイルに書き込んでから名前を変更するため、書き込みに失敗しても、前のファイルが残ります。

ノード、パラメーター、プロファイルについては、[実験](../experiments/index.md)、[ノード](../experiments/nodes.md)、[データ](../experiments/data.md)を参照してください。

### signals.json {#signals-json}

バージョン 2 のシグナルライブラリです。

```json
{
  "version": 2,
  "signals": [
    {
      "id": "…",
      "name": "Go cue",
      "group": "Stage/Cues",
      "note": "",
      "body": { "transport": "osc", "target": "127.0.0.1:9000", "address": "/cue/go", "args": [ { "type": "int", "value": 1 } ] }
    }
  ],
  "folders": [ "Stage", "Stage/Cues" ]
}
```

- `group` は、シグナルのフォルダーを `/` で区切ったパスにしたものです。空の場合は最上位です。`folders` (バージョン 2 で追加) は、空のフォルダーを含むすべてのフォルダーを列挙し、フォルダーがなければ省略されます。バージョン 1 のファイルも同じように読み込まれますが、空のフォルダーはありません。
- `body` は `osc`、`udp`、`http`、`mqtt` のいずれかで、そのフィールドは [`signals_save`](../api/commands.md#signals_save) に記載されています。
- ファイルが存在しない場合は、初期セットが書き込まれます。宛先はすべて `127.0.0.1` で、インターフェイスの言語に合わせた名前になります。
- 解析できないファイルは、パス、行、列とともに報告され (`signals.json_invalid`)、初期セットで置き換えられることはありません。修正するか削除してください。読み込めない間は、何もライブラリに書き込みません。保存は同じエラーで拒否され、ファイルはそのままです。[[ui:sig.reload]] で再び読み込めるまで、この状態が続きます。保存は同じフォルダー内の一時ファイルを経由するため、書き込みが途中で切れても、前のファイルが残ります。

### emulators.json {#emulators-json}

バージョン 1 のエミュレーターライブラリです。

```json
{
  "version": 1,
  "emulators": [
    { "id": "demo-api", "note": "…", "emulator": { "name": "Demo API", "bind": "127.0.0.1:8080", "protocol": "http", "routes": [ … ] } }
  ]
}
```

各エントリは、`id` と `note` を持つエミュレーターのドキュメントです。ドキュメントについては[エミュレーター](../tools/emulators.md)を参照してください。シグナルと同様に、ファイルがなければ初期セット (すべて `127.0.0.1` にバインド) が書き込まれ、壊れているファイルは報告され (`emulators.json_invalid`)、置き換えられることはありません。書き込みは一時ファイルを経由します。`signallab emulate` は、1 つのエミュレーター、そのリスト、またはこのようなライブラリを含む独自のファイルも読み込めます。

### 実行レポート {#run-reports}

`runs/run-<started ms>-<job>.json` は、レポートのバージョン 6 です。成功または失敗した実行ごとに 1 つのファイルが作られ、上書きされることはありません (同じ名前の 2 回目の実行には、`-2`、`-3`… が付きます)。停止された実行は、何も保存しません。

| フィールド | 内容 |
| --- | --- |
| `version` | 6 |
| `experiment` | 実験の名前 |
| `document_version` | 実行されたドキュメントのバージョン |
| `seed`、`profile` | 実行に使われたもの |
| `overrides` | この実行にだけ指定された値 |
| `params` | 使われたすべてのパラメーターの値 |
| `started_ms`、`ended_ms` | 1970 年からのミリ秒 |
| `outcome` | `passed` または `failed` |
| `error` | 最初の失敗。なければ null |
| `steps` | ステップ。[`experiment://step`](../api/events.md#event-experiment-step) と同じ形式で、最初の 5,000 件のあとは 1 秒に 1 件と最後の 500 件に絞られ、欠落分は `omitted` に数えられます (バージョン 6 から) |
| `emulators` | 各 [[ui:exp.node.emulator]] ノードが受信して応答した内容 (バージョン 3 から)。ない場合は省略されます |
| `impairments` | 各 [[ui:exp.node.impairment]] ノードのリレーが、フェーズごとに行ったこと (バージョン 4 から)。ない場合は省略されます |

負荷のステップの測定値は、そのステップの最後のステップに入ります (バージョン 5 から)。タイムラインの実行履歴と [[ui:exp.compare]] は、これらのファイルを読み込みます。読み取れないレポートは、一覧から除かれます。[実行とレポート](../experiments/runs.md)を参照してください。

### エクスポート {#exports}

- `exports/experiment-…json`: 上記の実験ドキュメント。エクスポートのたびに新しいファイルになります。
- `capture-….jsonl`: 1 行に 1 つのインスペクターのフレーム。保持しているバイト列は `data` に base64 で入ります。
- `capture-….txt`: 読むためのフレームで、それぞれに hex ダンプが付きます。

サーバーでは、インスペクターのエクスポートは、作成されると同時にあなたのコンピューターにダウンロードされます。実験のエクスポートには [[ui:common.download]] が表示され、タイムラインの実行の [[ui:exp.reportSaved]] は、そのレポートをダウンロードするリンクです。

## シークレットはこれらのファイルに入りません {#secrets}

実験はシークレットを `{{secret.API_TOKEN}}` のように名前で指定し、書き込まれるのは名前だけです。値は、次の場所に保持されます。

| Signal Lab の動作場所 | シークレットの値の保存場所 |
| --- | --- |
| デスクトップアプリ、Windows | Windows 資格情報マネージャーの `SignalLab` の下 (エディターの [[ui:exp.secrets]]) |
| デスクトップアプリ、Linux | どこにもありません。シークレットは保存できません (`secret.unsupported`) |
| サーバー | 読み取り専用。環境変数 `SIGNALLAB_SECRET_<NAME>`、または `--secrets-dir` (既定は `/run/secrets/signallab`) 内のファイル `<NAME>` |
| `signallab` | 同じファイルと環境変数、または `--secrets system` を指定した場合はシステムのストア |

::: warning
フィールドに直接入力したものは、入力したとおりに保持されます。HTTP シグナルの認証情報にあるパスワード、MQTT ブローカーエミュレーターのパスワード、ヘッダーに貼り付けたトークンは、すべて `signals.json`、`emulators.json`、`experiment.json` に、またそのエクスポートにも、プレーンテキストで入ります。共有フォルダーに置きたくないものは、実験では `{{secret.NAME}}` を使ってください。
:::

## インターフェイスの設定 {#settings}

インターフェイスが記憶するもの、つまり言語、各画面で最後に入力した値、開いているペインとそのサイズ、[[ui:http.keepCookies]]、アップデートを最後に確認した時刻、アップデート用のインストール固有の乱数は、データフォルダーではなく、インターフェイス自身が保持します。デスクトップではアプリ自身のストレージ、サーバーのページではブラウザーのサイトストレージ (ブラウザーごと) です。[[ui:nav.http]] 画面の認証情報は、そこには保持されません。

Signal Lab はログファイルを書き込みません。[トラブルシューティング](troubleshooting.md#logs)を参照してください。

## バックアップ、編集、移動 {#backup}

- **バックアップ**は、フォルダー全体をコピーして行います。中身はすべて、それだけで完結する JSON です。シークレットの値は含まれないので、新しいマシンでは設定し直してください。
- `signals.json`、`emulators.json`、`experiment.json` を**手で編集**するのは、Signal Lab を閉じている間 (サーバーでは、どのページも開いていない間) にしてください。アプリは、自分が保持している内容からファイル全体を書き込みます。`signals.json` と `emulators.json` では、アプリが読み込んだあとに変更されたファイルへの保存は拒否され (`file.changed`)、あなたの変更は保たれます。再読み込みを押して読み込んでください。`experiment.json` は、エディターの次の保存で上書きされます。間違いは、次にファイルが読み込まれるときに、行と列とともに報告され、黙って置き換えられることはありません。`signals.json` と `emulators.json` については、アプリがそこに保存するときも同様です。その保存は拒否され、ファイルは編集したままの状態で残ります。
- `runs/`、`exports/`、`capture-*` ファイルは、いつでも**削除**できます。`signals.json` や `emulators.json` を削除すると初期セットが戻り、`experiment.json` を削除すると初期の実験が戻ります。
- フォルダーの**移動**は、コピーして、Signal Lab に新しい場所を指定します。サーバーでは `--data-dir`、デスクトップアプリでは `SIGNALLAB_DATA_DIR` です。
- 実験の**共有**は、エクスポートするか、テスト対象のプロジェクトと並べて JSON をコミットして行います。[`signallab run`](../automation/cli.md#cli-run) はそこから実行できます。

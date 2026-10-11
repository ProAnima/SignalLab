---
title: キーボードショートカット
description: Signal Lab のすべてのキーボードショートカット。アプリ全体、実験エディター、シグナルライブラリ、ツール画面、ペイン、言語メニューで使えるものです。
---

# キーボードショートカット

Signal Lab が反応するすべてのキーを、使える場所ごとにまとめています。1 文字のキーや Delete のショートカットは、フィールドに入力している間は動作しません。

::: tip
Mac のブラウザーでは、以下で <kbd>Ctrl</kbd> と書かれている場所では <kbd>⌘</kbd> が使えます。ただし、テンプレートフィールドの <kbd>Ctrl</kbd>+<kbd>Space</kbd> は除きます。
:::

## どこでも {#anywhere}

| キー | 動作 |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>K</kbd> | どの画面でも、フィールドに入力中でも、シグナルパレットの [[ui:sig.paletteTitle]] を開きます。もう一度押すと閉じます |
| <kbd>F1</kbd> | [[ui:app.docs]] と同じく、いま開いている画面のページで、このドキュメントを開きます |
| <kbd>Tab</kbd>、<kbd>Shift</kbd>+<kbd>Tab</kbd> | コントロール間を移動します。コントロールにフォーカスが移るとそのツールチップが表示され、ほかのキーを押すと隠れます |
| <kbd>Space</kbd>、<kbd>Enter</kbd> | フォーカスのあるボタンを押します。クリックできるものはすべてボタンです |
| <kbd>Esc</kbd> | 開いているダイアログを閉じます |

シグナルパレットでは、次のとおりです。

| キー | 動作 |
| --- | --- |
| 入力 | シグナルを絞り込みます |
| <kbd>↑</kbd> <kbd>↓</kbd> | シグナルを選びます |
| <kbd>Enter</kbd> | 送信して、パレットを閉じます |
| <kbd>Esc</kbd> | パレットを閉じます。パレットの外側をクリックしても閉じます |

## 実験エディター {#editor}

キャンバス上では、どのテキストフィールドにもフォーカスがないときに、次のキーが使えます。

| キー | 動作 |
| --- | --- |
| <kbd>A</kbd> | ノードを追加するメニュー ([[ui:exp.addNode]]) を開きます。ノードを選択している場合、新しいノードはその後ろに追加されます |
| <kbd>Delete</kbd> または <kbd>Backspace</kbd> | 選択した接続、または選択したノードを削除します。[[ui:exp.node.start]] と [[ui:exp.node.end]] は削除できません |
| <kbd>Tab</kbd> | ノード、そのポート、接続の間を移動します。フォーカスが移ったノードや接続は選択されます |
| 出力で <kbd>Enter</kbd> または <kbd>Space</kbd>、続けてノードまたはその入力で同じキー | それらを接続します |
| <kbd>←</kbd> <kbd>→</kbd> <kbd>↑</kbd> <kbd>↓</kbd> | ノードにフォーカスがある間、選択したノードを 5 ポイントずつ (<kbd>Shift</kbd> を押すと 20) 動かします。押し続ける操作は、履歴の 1 ステップになります |
| <kbd>Esc</kbd> | 追加メニューを閉じます。なければ、描いている接続をキャンセルします。なければ、選択した接続の選択を外します。なければ、複数ノードの選択を解除します。なければ [[ui:exp.fullscreen]] を終了します。なければ [[ui:exp.focus]] を終了します |
| <kbd>Ctrl</kbd>+<kbd>Z</kbd> | [[ui:exp.undo]] |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd>、<kbd>Ctrl</kbd>+<kbd>Y</kbd> | [[ui:exp.redo]] |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | [[ui:exp.duplicate]]: 選択したノードを、それらの間の接続ごとコピーします |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | ノードを、その種類、動作、ID で探します |
| <kbd>Ctrl</kbd>+<kbd>0</kbd> | [[ui:exp.fit]]: グラフ全体を表示します |
| <kbd>Ctrl</kbd>+<kbd>1</kbd> | [[ui:exp.resetZoom]]: 100% |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:exp.sendNow]]。待機の場合は [[ui:exp.listenNow]]: 実験を実行せずに、選択したノードだけを実行します |
| <kbd>Ctrl</kbd>+<kbd>A</kbd> | すべてのノードを選択します |
| <kbd>Ctrl</kbd>+<kbd>C</kbd> | [[ui:exp.copy]]: 選択したノードと、その間の接続をコピーします。複製のときと同じく、[[ui:exp.node.start]] と [[ui:exp.node.end]] は含まれません |
| <kbd>Ctrl</kbd>+<kbd>X</kbd> | それらを切り取ります |
| <kbd>Ctrl</kbd>+<kbd>V</kbd> | ここまたは別の実験でコピーしたノードを、新しい ID で貼り付けます |

<kbd>Ctrl</kbd>+<kbd>A</kbd>、<kbd>C</kbd>、<kbd>X</kbd>、<kbd>V</kbd> は、エディターにフォーカスがあるとき、またはどこにもフォーカスがなく、ページ上でテキストが選択されていないときに、ノードに対して動作します。[[ui:console.title]] やレポートなど、ほかの場所でテキストを選択しているときは、ページ本来の動作になります。<kbd>Ctrl</kbd>+<kbd>C</kbd> は、そのテキストをコピーします。

マウス操作は次のとおりです。

| 操作 | 動作 |
| --- | --- |
| 何もないキャンバスをドラッグ | 表示位置を移動します |
| 何もないキャンバスで <kbd>Shift</kbd> を押しながらドラッグ | 枠が触れたノードを選択に追加します |
| ノードを <kbd>Shift</kbd> または <kbd>Ctrl</kbd> を押しながらクリック | 選択に追加する、または選択から外します |
| 選択した複数のノードの 1 つをドラッグ | すべてを移動します |
| 何もないキャンバスでダブルクリック | その位置に追加メニューを開きます |
| <kbd>Ctrl</kbd> を押しながらホイール | ポインターの位置を中心にズームします |

追加メニュー、ノードの検索、プロパティでは、次のとおりです。

| 場所 | キー | 動作 |
| --- | --- | --- |
| 追加メニュー | 入力 | ノードと保存済みシグナルを絞り込みます |
| 追加メニュー | <kbd>↑</kbd> <kbd>↓</kbd>、<kbd>Enter</kbd>、<kbd>Esc</kbd> | 選ぶ、追加する、閉じる |
| ノードの検索 | <kbd>↑</kbd> <kbd>↓</kbd> | ノードを選びます |
| ノードの検索 | <kbd>Enter</kbd> | キャンバス上に表示して選択します |
| ノードの検索 | <kbd>Tab</kbd> | 検索フィールドに戻ります |
| ノードの検索 | <kbd>Esc</kbd> | 閉じます |
| [[ui:exp.properties]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | 表示中のノードを [[ui:exp.sendNow]] します |
| [[ui:exp.properties]] | フィールドで <kbd>Esc</kbd> | ノードを選択したままキャンバスに戻ります。そこで <kbd>A</kbd> を押すと次のノードを追加できます |

テンプレート (`{{name}}`) を受け付けるフィールドでは、次のとおりです。

| キー | 動作 |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>Space</kbd> | カーソル位置に `{{` を入力し、入れられるものを一覧表示します。パラメーター、変数、シークレット、ジェネレーターです |
| `{{` の入力 | 同じものを一覧表示します |
| <kbd>↑</kbd> <kbd>↓</kbd> | 一覧から選びます |
| <kbd>Enter</kbd> または <kbd>Tab</kbd> | 選んだものを挿入し、`}}` を閉じます |
| <kbd>Esc</kbd> | 一覧を閉じます。フィールドにはフォーカスが残ります |

実験のダイアログでは、次のとおりです。

| 場所 | キー | 動作 |
| --- | --- | --- |
| [[ui:exp.params]] | 最後のパラメーターの値で <kbd>Enter</kbd> | パラメーターをもう 1 つ追加します |
| [[ui:exp.params]]、[[ui:exp.runWith]] | <kbd>Esc</kbd> | ダイアログを閉じます |
| [[ui:exp.runWith]] | フィールドで <kbd>Enter</kbd> | これらの値で [[ui:exp.run]] します |
| [[ui:exp.secrets]] | 値で <kbd>Enter</kbd> | 保存します |
| [[ui:exp.secrets]] | 値または新しい名前で <kbd>Esc</kbd> | キャンセルします。[[ui:exp.params]] は開いたままです |

## シグナルライブラリ {#signals}

| 場所 | キー | 動作 |
| --- | --- | --- |
| [[ui:nav.signals]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:sig.fire]]: 選択したシグナルを送信します |
| [[ui:nav.signals]] | シグナルをダブルクリック | 送信します |
| フォルダー | <kbd>F2</kbd> | [[ui:sig.renameFolder]] |
| フォルダー名の変更中 | <kbd>Enter</kbd>、<kbd>Esc</kbd> | 新しい名前を確定する、またはキャンセルする |
| フォルダー | <kbd>Delete</kbd> を 2 回 | [[ui:sig.removeFolder]]。中身は 1 階層上に移ります |
| フォルダー | <kbd>→</kbd> <kbd>←</kbd> | 開く、閉じる |
| [[ui:nav.http]]、[[ui:nav.osc]]、[[ui:nav.mqtt]] の送信部分 | <kbd>Ctrl</kbd>+<kbd>S</kbd> | メッセージがライブラリのシグナルと結び付いていて (そこから開いたか、そこに保存した)、変更されているときは [[ui:sig.save]]。まだライブラリにないときは [[ui:sig.saveNew]] |
| [[ui:sig.saveTitle]] | <kbd>Enter</kbd>、<kbd>Esc</kbd> | 保存する、またはキャンセルする |

## ツール画面 {#tools}

| 画面 | キー | 動作 |
| --- | --- | --- |
| [[ui:nav.http]] | URL、ヘッダー、認証情報、タイムアウトで <kbd>Enter</kbd> | リクエストを送信します |
| [[ui:nav.http]] | ボディを含むリクエストのどこでも <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | リクエストを送信します |
| [[ui:nav.osc]] | 宛先、アドレス、引数で <kbd>Enter</kbd> | メッセージを送信します |
| [[ui:nav.ws]] | URL で <kbd>Enter</kbd> | 接続します |
| [[ui:nav.ws]] | [[ui:ws.message]] で <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | 送信します |
| [[ui:nav.mqtt]] | [[ui:mq.addSubscription]] で <kbd>Enter</kbd> | サブスクライブします |
| [[ui:nav.broadcast]] | [[ui:bc.targetAddress]] または [[ui:bc.targetCidr]] で <kbd>Enter</kbd>。[[ui:bc.mode.list]] 以外のすべてのモード | [[ui:bc.sendOnce]] |

[[ui:feedback.title]] では、[[ui:feedback.message]] で <kbd>Ctrl</kbd>+<kbd>Enter</kbd> を押すと送信されます ([[ui:feedback.email]] で <kbd>Enter</kbd> を押しても同様です)。ダイアログ内のどこでも <kbd>Ctrl</kbd>+<kbd>V</kbd> を押すと、クリップボードのスクリーンショットが貼り付けられ、<kbd>Esc</kbd> で閉じます。

## ペイン {#panes}

ペイン間のハンドル ([[ui:layout.console]]、[[ui:layout.properties]]、[[ui:layout.timeline]]) には、<kbd>Tab</kbd> でフォーカスを移せます。

| キー | 動作 |
| --- | --- |
| <kbd>↑</kbd> <kbd>↓</kbd> | コンソールとタイムライン: 16 ピクセルずつ高く、または低くします |
| <kbd>←</kbd> <kbd>→</kbd> | プロパティ: 16 ピクセルずつ広く、または狭くします (インターフェイスが右から左に読む言語では逆になります) |
| <kbd>Shift</kbd> + 矢印 | 4 倍の幅で動かします |
| <kbd>Home</kbd>、<kbd>End</kbd> | 最小のサイズ、最大のサイズ |
| <kbd>Enter</kbd> | 既定のサイズ。ハンドルをダブルクリックしても同じです |

## 言語メニュー {#language}

ヘッダーの国旗と文字です。

| 場所 | キー | 動作 |
| --- | --- | --- |
| ボタン上 | <kbd>↓</kbd> または <kbd>↑</kbd> | 一覧を開きます |
| 一覧内 | <kbd>↓</kbd> <kbd>↑</kbd> | 次の言語、前の言語 |
| 一覧内 | <kbd>Home</kbd>、<kbd>End</kbd> | 最初、最後 |
| 一覧内 | 文字 | その言語自身の言語、あなたの言語、または英語での名前か文字が、その文字で始まる次の言語 |
| 一覧内 | <kbd>Enter</kbd> または <kbd>Space</kbd> | その言語に切り替えます |
| 一覧内 | <kbd>Esc</kbd> または <kbd>Tab</kbd> | 一覧を閉じます |

---
layout: home
title: Signal Lab
description: Signal Lab は OSC とネットワークプロトコルのテストラボです。ショーコントロール機器、デバイス、サービスの通信を送信、キャプチャ、エミュレート、劣化させ、繰り返し実行できるテストにします。
hero:
  name: Signal Lab
  text: OSC とネットワークプロトコルのテストラボ
  tagline: 機器やサービスがやり取りする通信を送って観察し、まだ用意できていないデバイスや API の代わりを務め、ネットワークを意図的に壊します。そして同じチェックを、アプリからも、スクリプトや CI からも、何度でも実行できます。
  actions:
    - theme: brand
      text: はじめる
      link: /ja/guide/
    - theme: alt
      text: ダウンロード
      link: https://github.com/ProAnima/SignalLab/releases/latest
features:
  - title: ラックのすべてのプロトコル
    details: 型付き引数の OSC、生の UDP と TCP、Basic・Bearer・Digest 認証に対応した HTTP、WebSocket、MQTT 3.1.1。さらにブロードキャスト、マルチキャスト、サブネットスイープ、ディスカバリーリスナーも備えています。
    link: /ja/protocols/osc
  - title: すべてを 1 つのインスペクターで
    details: ツールが送受信するすべてのフレームを 1 つのタイムラインに、デコード結果とバイト列とともに表示します。絞り込みやエクスポートができ、キャプチャしたフレームをバイト単位でそのまま再送できます。
    link: /ja/tools/inspector
  - title: シグナルのライブラリ
    details: うまくいったメッセージに名前を付けてフォルダーに整理し、どこからでも Ctrl+K で再送信できます。OSC、生の UDP、HTTP リクエスト、MQTT パブリッシュに対応しています。
    link: /ja/tools/signals
  - title: エミュレーター
    details: 相手側を演じます。モックの HTTP API、OSC・UDP・TCP デバイス、MQTT ブローカーを、ルール、シーケンス、遅延、障害、スケジュールに沿ったダウンとともに用意できます。
    link: /ja/tools/emulators
  - title: ネットワーク劣化
    details: クライアントと宛先の間に入るリレーが、レイテンシ、ジッター、ロス、重複、順序の入れ替え、帯域制限を加えたり、TCP 接続をリセットしたりします。シードに基づくので、同じ通信は同じ結果になります。
    link: /ja/tools/impairment
  - title: 実験
    details: 送信、応答の待機、チェック、分岐、ループ、ブランチの並列実行を組み合わせたビジュアルなテストフローです。パラメーター、プロファイル、シークレットを使え、実行ごとにレポートが残ります。
    link: /ja/experiments/
  - title: 負荷テスト
    details: HTTP リクエストに一定のレート、ランプ、段階、スパイク、ランダムな到着で負荷をかけ、p50 から p99、エラー、達成したレートを計測し、しきい値を満たさなければ実行を失敗にします。
    link: /ja/experiments/load
  - title: 自動化
    details: signallab コマンドラインで実験をウィンドウなしで実行できます。終了コード、JUnit レポート、GitHub Action に対応し、MCP 経由で AI アシスタントに任せることもできます。
    link: /ja/automation/cli
  - title: サーバーと API
    details: 同じインターフェイスをブラウザーで、同じエンジンをラボの PC や Docker で、トークンでサインインして使えます。すべてのコマンドと実行に対応する HTTP API もあります。
    link: /ja/server/
---

Signal Lab は、Windows と Linux のデスクトップアプリとして、またはブラウザーで開くサーバーとして動作します。初めての方は、[Signal Lab とは](guide/index.md)を読んで[インストール](guide/install.md)し、[はじめの一歩](guide/first-steps.md)に進んでください。メッセージの送受信、シグナルの保存、エミュレートした API の応答、小さな実験の実行を、すべてこのコンピューター上で試します。

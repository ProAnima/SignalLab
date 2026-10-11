---
title: サーバーの実行
description: 機器の隣の Linux マシンで Signal Lab を動かし、ネットワーク上のどのブラウザーからでも使います。スクリプトや CI 向けの HTTP API もあります。
---

# Signal Lab をサーバーとして実行する

`signal-lab-server` は、ウィンドウのない Signal Lab です。エンジンは同じで、同じインターフェイスをブラウザーに提供します。機器の隣にあるマシン（ラックの PC、ショーコントロールの VM、共有のラボ用マシン）に置いておけば、ネットワーク上のどこからでも Chrome、Firefox、Edge で開けます。すべての画面がデスクトップアプリと同じように動作し、実行、レポート、エクスポートはブラウザー経由でダウンロードされます。

スクリプトやパイプラインは、同じサーバーを [HTTP API](../api/index.md) 経由で使い、[`signallab --server`](../automation/cli.md#run-on-server) は実行をそこに送ります。

デスクトップアプリとは、いくつか違う点があります。

- **サーバー 1 台はエンジン 1 つです。** サインインしているすべてのページには、実行中の同じジョブ、同じシグナルライブラリとエミュレーターライブラリ、[[ui:nav.http]] 画面の同じ Cookie ジャー（[[ui:http.keepCookies]]）が見えます。サーバーを共有する人たちは、それらを共有します。
- **シークレットはサーバーのものです。** 環境変数またはファイルから読み取られ、ブラウザーからは設定できません（[シークレット](#secrets)を参照）。
- **ファイアウォールはホストのものです。** サーバーがそれを変更することはなく、デスクトップアプリのファイアウォールの通知は表示されません。
- **更新はイメージで行います。** アプリのアップデーターは使いません（[更新](#update)を参照）。

## Linux ホストで、1 つのコマンドで {#install-script}

インターネットに接続できる Linux マシンでは、次のようにします。

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh
```

スクリプトは次のことを行います。

1. Docker がなければインストールします。先に確認を求め、Docker 自身のインストーラー（`get.docker.com`）を使います。
2. `/opt/signallab`（root でないときは `~/signallab`）に `compose.yaml` を書き込みます。
3. イメージを取得し、ホストネットワークでサーバーを起動するので、OSC、UDP、ブロードキャスト、マルチキャスト、ディスカバリーが実際のネットワークに届きます。
4. サーバーがヘルスチェックに応答するまで待ちます（最大 90 秒）。
5. 開くべきアドレス、サインインに使うアクセストークン、更新、ログの確認、削除のコマンドを表示します。
6. ufw または firewalld が有効なときは、サーバーのポートを開くかどうかを尋ねます（[ホストのファイアウォール](#firewall)を参照）。

オプションは `sh -s --` の後に渡します。

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh -s -- --version [[version]] --port 8430
```

| オプション | 動作 | 既定値 |
| --- | --- | --- |
| `--version X.Y.Z` | イメージのバージョン（`latest` または `X.Y.Z`。先頭の `v` は取り除かれます）。 | `latest` |
| `--port N` | ブラウザーが使うポート。 | `1430` |
| `--listen IP:PORT` | 1 つのアドレスだけで待ち受けます。 | `0.0.0.0:<port>` |
| `--dir DIR` | compose ファイルの置き場所。 | root なら `/opt/<name>`、それ以外は `~/<name>` |
| `--name NAME` | コンテナーとそのデータボリュームの名前: 小文字の英字、数字、`-`、`_`。同じホストで 2 台目のサーバーを動かすには、独自の名前が必要です。 | `signallab` |
| `--image NAME` | 別のイメージまたはレジストリ。完全な `NAME:TAG` はそのまま使われます。 | `ghcr.io/proanima/signallab` |
| `--open-udp PORTS` | ファイアウォールが有効なとき、モニターや待機のために、これらのポートへの UDP も許可します: `9000,9100:9110`。 | |
| `--no-firewall` | ufw や firewalld を一切変更しません。 | |
| `--yes`, `-y` | すべて「はい」と答えます: Docker のインストール、ファイアウォールの開放、`--purge` によるデータの削除。 | |
| `--uninstall` | サーバーを停止して削除します。データは残ります。 | |
| `--purge` | `--uninstall` と併用: データとトークンも削除します。 | |
| `--help`, `-h` | オプションを表示します。 | |

Docker の Compose プラグイン（`docker-compose-plugin` パッケージ）が必要で、Docker のインストーラーが一緒に入れてくれます。イメージは x86_64 と arm64 向けにビルドされています。

**更新するには、もう一度実行します。** 同じコマンドが最新のイメージ（または指定した `--version`）を取得してサーバーを再起動します。データとトークンはそのまま残ります。`--name` を使った場合は、同じ名前をもう一度指定してください。

**独自の設定**（実験のシークレット、`SIGNALLAB_ALLOWED_HOSTS`、HTTPS の背後での `SIGNALLAB_SECURE_COOKIE`）は、compose ファイルの隣の `compose.override.yaml` に書きます。Docker Compose がそれを統合し、スクリプトは実行のたびに `compose.yaml` を書き直しますが、オーバーライドには触れません。スクリプトが書いたものではない `compose.yaml` は、`compose.yaml.before-install` として残されます。

```yaml
# compose.override.yaml
services:
  signallab:
    environment:
      SIGNALLAB_ALLOWED_HOSTS: lab-pc.example.com,192.0.2.10
      SIGNALLAB_SECRET_API_TOKEN: ${API_TOKEN}
```

**削除するには:**

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh -s -- --uninstall
```

データは Docker のボリュームに残り、もう一度インストールすれば、同じトークンでそのまま戻ります。`--uninstall --purge` は、確認を求めた上で、データとトークンも削除します。

### ホストのファイアウォール {#firewall}

ホストネットワークでは、サーバーはホスト自身のポートで待ち受けるので、誰が到達できるかはホストのファイアウォールが決めます。ufw または firewalld が有効なとき、スクリプトは、サーバーの TCP ポートと `--open-udp` の UDP ポートを開く前に確認を求め、開いたものを書き留め（compose ファイルの隣の `.firewall`）、`--uninstall` はちょうどそれだけを閉じます。断った場合、ほかのマシンのブラウザーは、ファイアウォールが許可するまでサーバーに到達できず、モニターや待機が、ほかのマシンからの通信を聞き取れるのは、開いた UDP ポートだけです。

## Docker {#docker}

### イメージ {#image}

`ghcr.io/proanima/signallab` は、`linux/amd64` と `linux/arm64` 向けに、リリースごとに公開されます。

| タグ | 内容 |
| --- | --- |
| `X.Y.Z` | そのリリース。 |
| `X.Y` | そのラインの最新の安定版リリース。 |
| `latest` | 最新の安定版リリース。 |

イメージには、`signal-lab-server`、ビルド済みのインターフェイス、コマンドライン `signallab` が入っており、次の設定でサーバーを起動します。

| 変数 | イメージでの値 |
| --- | --- |
| `SIGNALLAB_LISTEN` | `0.0.0.0:1430` |
| `SIGNALLAB_DATA_DIR` | `/data` |
| `SIGNALLAB_UI_DIR` | `/usr/share/signal-lab/ui` |
| `SIGNALLAB_GENERATE_TOKEN` | `true` |

権限のないユーザー（uid と gid が 10001）として実行され、`/data`（ボリューム）にだけ書き込み、ポート `1430` を公開し、30 秒ごとに自分自身のヘルスをチェックします。

### 起動 {#docker-run}

```bash
docker run -d --name signallab --network host --restart unless-stopped \
  -v signallab-data:/data --read-only --cap-drop ALL --security-opt no-new-privileges \
  ghcr.io/proanima/signallab:[[version]]
docker logs signallab                    # 初回の起動時: 「Sign in with it:」とトークン
```

その後、`http://<host>:1430` を開き、トークンでサインインします。後でもう一度トークンを見るには、次のようにします。

```bash
docker exec signallab cat /data/token
```

`--read-only`、`--cap-drop ALL`、`no-new-privileges` は任意で、費用もかかりません。サーバーに特権は必要なく、書き込むのは `/data` だけです。

### Docker Compose {#compose}

リポジトリの `deploy/compose.yaml` は、同じものを Compose ファイルにしたものです。

```yaml
name: signallab

services:
  signallab:
    image: ${SIGNALLAB_IMAGE:-ghcr.io/proanima/signallab:latest}
    container_name: signallab
    network_mode: host
    environment:
      SIGNALLAB_GENERATE_TOKEN: "true"
    volumes:
      - signallab-data:/data
    read_only: true
    cap_drop: [ALL]
    security_opt: ["no-new-privileges:true"]
    restart: unless-stopped
    stop_grace_period: 15s

volumes:
  signallab-data:
```

```bash
docker compose up -d
docker exec signallab cat /data/token
```

バージョンを固定するには、`SIGNALLAB_IMAGE=ghcr.io/proanima/signallab:X.Y.Z` を設定します。

### ネットワーク {#networking}

| Docker のネットワーク | 動作するもの | 動作しないもの |
| --- | --- | --- |
| `--network host`（`network_mode: host`）、Linux ホスト | すべて: LAN への OSC、UDP、TCP、HTTP、WebSocket、MQTT、待ち受けポート、ブロードキャスト、マルチキャスト、ディスカバリー。 | — |
| ブリッジ（既定）、ポートを公開 | コンテナーが到達できるホストへのユニキャスト。公開したポートのリスナー（`-p 1430:1430 -p 9000:9000/udp`）。 | ブロードキャストとマルチキャスト。公開していないポートへの応答。 |
| Windows または macOS の Docker Desktop | ユニキャストと、公開したリスナー。 | 物理ネットワークへのホストネットワーク。Windows ではデスクトップアプリを使ってください。 |

### データボリューム {#data-volume}

`/data` には、サーバーが保持するものすべてが入ります。実験、シグナルとエミュレーターのライブラリ、実行レポート、エクスポート、トークンです。上のような名前付きボリュームは、最初からイメージのユーザーが所有しています。ホストのフォルダーをそこにマウントする場合は、uid 10001 が書き込める必要があります。

```bash
sudo mkdir -p /srv/signallab && sudo chown 10001:10001 /srv/signallab
docker run -d --name signallab --network host -v /srv/signallab:/data ghcr.io/proanima/signallab:[[version]]
```

## Docker を使わない場合 {#binary}

サーバーはイメージとして公開されています。`signal-lab-server` を直接実行するには、ソースからビルドしてください（[ビルド](../../develop/building.md)を参照）。

```bash
npm install
npm run build                            # インターフェイスを dist/ に
cargo run --release -p signal-lab-server
```

`dist/` を `http://127.0.0.1:1430` で提供します。このマシン専用で、トークンは不要です。ネットワークに公開するには、`--listen` とトークンを追加してください。

## オプション {#options}

すべてのオプションには、コンテナー向けの環境変数があります。オプションが変数より優先されます。

| オプション | 変数 | 既定値 | 動作 |
| --- | --- | --- | --- |
| `--listen IP:PORT` | `SIGNALLAB_LISTEN` | `127.0.0.1:1430` | 待ち受ける場所。ループバック以外のアドレスにはトークンが必要です。 |
| `--token-file PATH` | `SIGNALLAB_TOKEN_FILE` | | アクセストークンを保持するファイル（たとえば Docker のシークレット）。 |
| `--token TOKEN` | `SIGNALLAB_TOKEN` | | アクセストークンそのもの。ファイルのほうをお勧めします。引数は同じマシンのほかのユーザーから見えます。 |
| `--generate-token` | `SIGNALLAB_GENERATE_TOKEN` | オフ | トークンが指定されておらず、ループバックより外のアドレスの場合: `<data folder>/token` に保存されたトークンを使い、初回の起動時に作成します。 |
| `--data-dir PATH` | `SIGNALLAB_DATA_DIR` | ユーザーのホームフォルダーの `Documents/SignalLab` | データフォルダー。 |
| `--secrets-dir PATH` | `SIGNALLAB_SECRETS_DIR` | `/run/secrets/signallab` | 読み取り専用のシークレットのフォルダー。名前ごとに 1 ファイル。 |
| `--ui-dir PATH` | `SIGNALLAB_UI_DIR` | プログラムの隣の `ui`、なければ `./dist` | ビルド済みのインターフェイス。なければ、API だけが提供されます。 |
| `--allowed-host NAME` | `SIGNALLAB_ALLOWED_HOSTS` | | サーバーに到達するときに使える名前。カンマ区切り。これらとループバックの名前は、トークンの有無にかかわらず常に受け付けられます。何も指定しない場合、トークンのあるサーバーはどの名前にも応答し、ないサーバーはループバックの名前にだけ応答します（[ホスト名](security.md#hosts)を参照）。 |
| `--secure-cookie` | `SIGNALLAB_SECURE_COOKIE` | オフ | セッション Cookie を HTTPS でのみ送ります。HTTPS プロキシの背後で設定してください。 |
| `--log FILTER` | `SIGNALLAB_LOG` | `info` | 記録する内容: `error`、`warn`、`info`、`debug`、またはモジュールごと（`signal_lab_server=debug`）。 |
| `--log-format text\|json` | `SIGNALLAB_LOG_FORMAT` | `text` | ログの行をテキストにするか、1 行に 1 つの JSON オブジェクトにします。 |

スイッチは、変数では `true` または `false` を取ります: `SIGNALLAB_GENERATE_TOKEN=true`。

| コマンド | 動作 |
| --- | --- |
| `signal-lab-server token` | 新しいランダムなトークン（16 進数 64 文字）を表示します。 |
| `signal-lab-server healthcheck` | `--listen` でサーバーが応答すれば `0` で終了します（イメージのヘルスチェック）。 |
| `signal-lab-server --version` | バージョンを表示します。 |
| `signal-lab-server --help` | すべてのオプションを表示します。 |

**終了コード:** <kbd>Ctrl</kbd>+<kbd>C</kbd> または `SIGTERM` で停止したときは `0`。設定が拒否されたときは `2`（到達可能なアドレスでトークンがない、トークンが短すぎる、トークンが 2 回指定された、読み取れないトークンファイル、データフォルダーなしの `--generate-token`）。アドレスで待ち受けられない、またはデータフォルダーに書き込めないときは `1`。理由は stderr に出力されます。

## アクセストークン {#token}

トークンがない場合、サーバーはループバックでしか待ち受けず、このマシンにしかサービスを提供しません。それ以外のアドレスではトークンが必要で、なければ起動を拒否します。トークンの与え方は 3 つあります。

| 方法 | 使いどころ |
| --- | --- |
| `--generate-token`（イメージではオン） | 何も設定する必要がありません。初回の起動時にサーバーがトークンを作り、`<data folder>/token`（それを実行するユーザーだけが読み取れます）に保存し、ログに 1 回だけ表示します。以降の起動ではそれを再利用するので、サインイン済みのブラウザーやスクリプトは、再起動や更新をまたいで動き続けます。データフォルダー（`--data-dir`）が必要です。 |
| `--token-file PATH` | 自分で用意したトークンをファイルに入れます（Docker のシークレットなど）。末尾の改行はトークンの一部ではありません。 |
| `SIGNALLAB_TOKEN` | 環境変数としてのトークン。 |

トークンは **24** 文字以上で、空白や改行を含まず、与える方法は 1 つだけにします。`signal-lab-server token` で、よいものが作れます。

```bash
docker run --rm ghcr.io/proanima/signallab:[[version]] token > signallab_token.txt
```

Compose では、それをシークレットとして渡します（ファイルは uid 10001 が読み取れる必要があります）。

```yaml
services:
  signallab:
    environment:
      SIGNALLAB_TOKEN_FILE: /run/secrets/signallab_token
    secrets:
      - signallab_token

secrets:
  signallab_token:
    file: ./signallab_token.txt
```

明示的に与えたトークンは `--generate-token` より優先され、その場合は何も作られません。作成されたトークンを変えるには、サーバーを停止し、`<data folder>/token` を削除して、もう一度起動します。新しいトークンが作られて表示されます。壊れたトークンファイルは報告されるだけで、置き換えられることはありません。

## サインイン {#sign-in}

`http://<host>:1430` を開きます。トークンのあるサーバーは、まずブラウザーをサインインページに送ります（ブラウザーの言語で表示されます）。トークンを 1 回貼り付ければ、ブラウザーは 7 日間サインインしたままです。インターフェイスの [[ui:app.signOut]] でセッションが終わります。セッションはサーバーのメモリに保持されるので、再起動するとすべての人がサインアウトされます。

トークンのないループバックでは、サインインはありません。

スクリプトは、すべてのリクエストでトークンを `Authorization: Bearer <token>` として送ります。

```bash
curl -fsS http://192.0.2.10:1430/api/invoke/app_info \
  -H "Authorization: Bearer $(cat signallab_token.txt)" \
  -H "Content-Type: application/json" -d 'null'
```

[HTTP API](../api/index.md)を、そしてこれらすべての背後にあるルールについては[サーバーのセキュリティ](security.md)を参照してください。

## データフォルダー {#data}

サーバーは、ファイルをデータフォルダーに保持します。実験、シグナルとエミュレーターのライブラリ、実行レポート、エクスポート、作成されたトークンです。場所は `--data-dir`（`SIGNALLAB_DATA_DIR`）、イメージでは `/data`、それ以外では実行ユーザーのホームフォルダーの `Documents/SignalLab` です。`--data-dir` で指定したデータフォルダーは、なければ作成され、起動時にチェックされます。サーバーがそこに書き込めない場合は、フォルダー名を示して停止します。[ファイル](../reference/files.md)を参照してください。

ダウンロード（レポート、エクスポート）は、このフォルダーの中のものだけが対象です。

## シークレット {#secrets}

実験はシークレットを `{{secret.NAME}}` として読みます。サーバーでは読み取り専用で、次の場所から読まれます。

1. 環境変数 `SIGNALLAB_SECRET_NAME`。なければ、
2. シークレットフォルダー（`--secrets-dir`。既定では `/run/secrets/signallab`、Docker のシークレットのレイアウト）の `NAME` というファイル。

ファイルの末尾の改行は値の一部ではなく、空の値は未設定とみなされ、値は最大 16 KiB です。名前は英字、数字、`_` で、数字では始まりません。インターフェイスには、サーバーでどのシークレットが設定されているかが表示されますが、ブラウザーから設定することは拒否されます。値はより弱い場所へ渡ることも、外へ戻ってくることもありません（[シークレット](security.md#secrets)を参照）。

Compose では、名前ごとにシークレットファイルを用意します。

```yaml
services:
  signallab:
    secrets:
      - source: api_token
        target: /run/secrets/signallab/API_TOKEN

secrets:
  api_token:
    file: ./api_token.txt
```

ファイルは uid 10001 が読み取れる必要があります。

## HTTPS プロキシの背後で {#https}

サーバーはプレーンな HTTP を話します。HTTPS にするには、その前にリバースプロキシ（Caddy、nginx、Traefik）を置き、次のようにします。

- `Host` ヘッダーを変更せずに渡す。
- WebSocket のアップグレードを通す（インターフェイスは `/api/events` への接続を 1 つ開いたままにします）。

そしてサーバーを `--secure-cookie` 付きで起動し、セッション Cookie が HTTPS でのみ送られるようにして、`--allowed-host` には人々が使う名前を設定します。

## 更新 {#update}

[[ui:update.server]]。デスクトップアプリのアップデーターは関係ありません。

- スクリプトでインストールした場合: 同じコマンドをもう一度実行します。
- Compose の場合: `docker compose pull && docker compose up -d`。
- `docker run` の場合: 新しいイメージを取得し、コンテナーを削除して、同じボリュームでもう一度起動します。

データとトークンはボリュームの中にあるので、そのまま残ります。

## ヘルスチェック {#health}

`GET /api/health` は、トークンなしで、誰にでも応答します。

```bash
curl -s http://127.0.0.1:1430/api/health
# {"auth":true,"status":"ok","version":"[[version]]"}
```

`auth` は、サーバーがトークンを求めるかどうかを示します。コマンド `signal-lab-server healthcheck` は、このマシンで同じことを尋ね、応答があれば `0` で終了します。イメージはこれを 30 秒ごとに実行する（タイムアウト 5 秒、3 回まで）ので、`docker ps` ではコンテナーが healthy と表示されます。

## ログ {#logs}

サーバーは stdout にログを出力します: `docker logs -f signallab`。既定のレベル（`info`）では、待ち受けている場所とトークンが必要かどうか、データフォルダーとインターフェイスのフォルダー、開始されたすべてのジョブ（モニター、ジェネレーター、ストーム、スキャン、実行、エミュレーター）とそのクライアントのアドレス、すべてのサインイン、誤ったトークンでのすべてのサインイン（警告として）、ページがイベントに遅れたときが記録されます。`--log debug` のレベルでは、すべてのコマンドが加わります。行に色が付くのは端末のときだけです（`NO_COLOR` が設定されていれば付きません）。`--log-format json` は、ログコレクター向けに 1 行に 1 つの JSON オブジェクトを書き出します。

## 停止 {#stop}

<kbd>Ctrl</kbd>+<kbd>C</kbd>、`docker stop`、`SIGTERM` は、すべてのページの接続を閉じ、すべてのジョブを停止し（実行中の実行は停止として終了します）、`0` で終了します。compose ファイルには、そのための 15 秒が与えられています。

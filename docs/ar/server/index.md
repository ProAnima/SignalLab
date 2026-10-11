---
title: تشغيل خادم
description: تشغيل Signal Lab على جهاز Linux بجانب المعدات واستخدامه من أي متصفح على الشبكة، مع واجهة HTTP API للسكربتات وCI.
---

# تشغيل Signal Lab بوصفه خادمًا

`signal-lab-server` هو Signal Lab دون نافذة: المحرك نفسه، يقدّم الواجهة نفسها إلى متصفح. يوضع على جهاز بجانب
المعدات — حاسوب خزانة، أو آلة افتراضية للتحكم في العروض، أو صندوق مختبر مشترك — ويُفتح من Chrome أو Firefox أو
Edge في أي مكان على الشبكة: فتعمل كل شاشة كما في التطبيق المكتبي، وتُنزَّل التشغيلات والتقارير والصادرات عبر
المتصفح.

وتستخدم السكربتات والخطوط (CI) الخادم نفسه عبر [واجهة HTTP API](../api/index.md)، ويرسل
[`signallab --server`](../automation/cli.md#run-on-server) التشغيلات إليه.

وهناك أمور تختلف عن التطبيق المكتبي:

- **الخادم الواحد محرك واحد.** كل صفحة مسجَّل دخولها إليه ترى المهام الجارية نفسها ومكتبتي الإشارات والمحاكيات
  نفسيهما ومخزن ملفات تعريف الارتباط نفسه الخاص بشاشة [[ui:nav.http]] ([[ui:http.keepCookies]]). ومن
  يتشاركون خادمًا يتشاركون هذه.
- **الأسرار للخادم**، تُقرأ من بيئته أو من ملفات؛ ولا يمكن تعيينها من المتصفح (انظر [الأسرار](#secrets)).
- **جدار الحماية للمضيف.** لا يغيّره الخادم أبدًا؛ ولا يظهر إشعار جدار الحماية الخاص بالتطبيق المكتبي.
- **يُحدَّث بصورته**، لا عبر محدّث التطبيق (انظر [التحديث](#update)).

## على مضيف Linux، بأمر واحد {#install-script}

على جهاز Linux متصل بالإنترنت:

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh
```

ما يفعله السكربت:

1. يثبّت Docker إن كان مفقودًا — ويسأل أولًا، مستخدمًا مثبّت Docker نفسه (`get.docker.com`)؛
2. يكتب `compose.yaml` في `/opt/signallab` (و`~/signallab` حين لا يكون المستخدم root)؛
3. يسحب الصورة ويبدأ الخادم بشبكة المضيف، فتبلغ OSC وUDP والبث والبث المتعدد والاكتشاف الشبكة الحقيقية؛
4. ينتظر إلى أن يجيب الخادم فحص السلامة (حتى 90 ثانية)؛
5. يطبع العناوين التي تُفتح، والرمز المميز للدخول به، وأوامر التحديث وقراءة السجلات وإزالته؛
6. وحين يكون ufw أو firewalld مفعّلًا يعرض فتح منفذ الخادم (انظر [جدار حماية المضيف](#firewall)).

تُمرَّر الخيارات بعد `sh -s --`:

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh -s -- --version [[version]] --port 8430
```

| الخيار | ما يفعله | الافتراضي |
| --- | --- | --- |
| `--version X.Y.Z` | إصدار الصورة (`latest` أو `X.Y.Z`؛ ويُحذف `v` في البداية). | `latest` |
| `--port N` | المنفذ الذي تستخدمه المتصفحات. | `1430` |
| `--listen IP:PORT` | الاستماع على عنوان واحد فقط. | `0.0.0.0:<port>` |
| `--dir DIR` | حيث يوضع ملف compose. | `/opt/<name>` للمستخدم root، و`~/<name>` وإلا |
| `--name NAME` | الحاوية وحجم بياناتها: أحرف صغيرة وأرقام و`-` و`_`. والخادم الثاني على المضيف نفسه يحتاج إلى اسم خاص به. | `signallab` |
| `--image NAME` | صورة أو سجل آخر؛ و`NAME:TAG` كاملًا يُستخدم كما هو. | `ghcr.io/proanima/signallab` |
| `--open-udp PORTS` | مع جدار حماية مفعّل، يُسمح كذلك لـ UDP بالدخول على هذه المنافذ، للمراقبات وعقد الانتظار: `9000,9100:9110`. | |
| `--no-firewall` | عدم تغيير ufw أو firewalld أبدًا. | |
| `--yes`، `-y` | الإجابة بنعم: تثبيت Docker، وفتح جدار الحماية، وحذف البيانات مع `--purge`. | |
| `--uninstall` | إيقاف الخادم وإزالته؛ وتبقى البيانات. | |
| `--purge` | مع `--uninstall`: حذف البيانات والرمز المميز أيضًا. | |
| `--help`، `-h` | طباعة الخيارات. | |

يحتاج إلى إضافة Compose الخاصة بـ Docker (الحزمة `docker-compose-plugin`)، التي يأتي بها مثبّت Docker. والصورة
مبنية لـ x86_64 وarm64.

**يُشغَّل مرة أخرى للتحديث:** الأمر نفسه يسحب أحدث صورة (أو `--version` الذي تعطيه) ويعيد تشغيل الخادم؛
وتبقى البيانات والرمز المميز. ومع `--name` يُعطى الاسم نفسه من جديد.

**الإعدادات الخاصة بك** — أسرار التجارب، و`SIGNALLAB_ALLOWED_HOSTS`، و`SIGNALLAB_SECURE_COOKIE` خلف HTTPS —
تُوضع في `compose.override.yaml` بجانب ملف compose. يدمجه Docker Compose، ويعيد السكربت كتابة `compose.yaml` في
كل تشغيل لكنه لا يمس ملف override أبدًا. وملف `compose.yaml` لم يكتبه يُحفظ باسم `compose.yaml.before-install`.

```yaml
# compose.override.yaml
services:
  signallab:
    environment:
      SIGNALLAB_ALLOWED_HOSTS: lab-pc.example.com,192.0.2.10
      SIGNALLAB_SECRET_API_TOKEN: ${API_TOKEN}
```

**لإزالته:**

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh -s -- --uninstall
```

تبقى البيانات في حجم Docker الخاص بها، والتثبيت من جديد يعيدها بالرمز المميز نفسه. و`--uninstall --purge` يحذف
البيانات والرمز المميز أيضًا، بعد السؤال.

### جدار حماية المضيف {#firewall}

مع شبكة المضيف يستمع الخادم على منافذ المضيف نفسها، فيقرر جدار حماية المضيف من يبلغه. وحين يكون ufw أو
firewalld مفعّلًا يسأل السكربت قبل أن يفتح منفذ TCP الخاص بالخادم ومنافذ UDP في `--open-udp`، ويدوّن ما فتحه
(`.firewall` بجانب ملف compose)، و`--uninstall` يغلق ذلك بالضبط من جديد. وإذا رُفض، لا تبلغ المتصفحات على
الأجهزة الأخرى الخادم إلا حين يتيح لها جدار الحماية ذلك، ولا تسمع المراقبات وعقد الانتظار الأجهزة الأخرى إلا
على منافذ UDP التي يفتحها.

## Docker {#docker}

### الصورة {#image}

`ghcr.io/proanima/signallab`، لـ `linux/amd64` و`linux/arm64`، تُنشر مع كل إصدار:

| الوسم | ما هو |
| --- | --- |
| `X.Y.Z` | ذلك الإصدار. |
| `X.Y` | أحدث إصدار مستقر من ذلك الخط. |
| `latest` | أحدث إصدار مستقر. |

تحوي `signal-lab-server` والواجهة المبنية وسطر الأوامر `signallab`، وتبدأ الخادم بهذه الإعدادات:

| المتغير | القيمة في الصورة |
| --- | --- |
| `SIGNALLAB_LISTEN` | `0.0.0.0:1430` |
| `SIGNALLAB_DATA_DIR` | `/data` |
| `SIGNALLAB_UI_DIR` | `/usr/share/signal-lab/ui` |
| `SIGNALLAB_GENERATE_TOKEN` | `true` |

تعمل بمستخدم بلا امتيازات (uid وgid 10001)، وتكتب في `/data` فقط (حجم)، وتعرض المنفذ `1430`، وتفحص سلامتها بنفسها
كل 30 ثانية.

### بدؤها {#docker-run}

```bash
docker run -d --name signallab --network host --restart unless-stopped \
  -v signallab-data:/data --read-only --cap-drop ALL --security-opt no-new-privileges \
  ghcr.io/proanima/signallab:[[version]]
docker logs signallab                    # on the first start: "Sign in with it:" and the token
```

ثم يُفتح `http://<host>:1430` ويُسجَّل الدخول بالرمز المميز. ولرؤية الرمز مرة أخرى لاحقًا:

```bash
docker exec signallab cat /data/token
```

`--read-only` و`--cap-drop ALL` و`no-new-privileges` اختيارية ولا تكلّف شيئًا: فالخادم لا يحتاج إلى امتيازات ولا
يكتب إلا في `/data`.

### Docker Compose {#compose}

`deploy/compose.yaml` في المستودع هو الشيء نفسه ملف Compose:

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

يُعيَّن `SIGNALLAB_IMAGE=ghcr.io/proanima/signallab:X.Y.Z` لتثبيت إصدار.

### الشبكات {#networking}

| شبكة Docker | ما يعمل | ما لا يعمل |
| --- | --- | --- |
| `--network host` (`network_mode: host`)، على مضيف Linux | كل شيء: OSC وUDP وTCP وHTTP وWebSocket وMQTT إلى الشبكة المحلية، ومنافذ الاستماع، والبث، والبث المتعدد، والاكتشاف. | — |
| الجسر (الافتراضي)، مع منافذ منشورة | البث الأحادي إلى المضيفات التي تبلغها الحاوية؛ ومستمعات على منافذ منشورة (`-p 1430:1430 -p 9000:9000/udp`). | البث والبث المتعدد؛ والردود إلى منافذ غير منشورة. |
| Docker Desktop على Windows أو macOS | البث الأحادي والمستمعات المنشورة. | شبكة المضيف إلى الشبكة الفعلية. وعلى Windows يُستخدم التطبيق المكتبي. |

### حجم البيانات {#data-volume}

يحوي `/data` كل ما يحتفظ به الخادم: التجربة ومكتبتي الإشارات والمحاكيات وتقارير التشغيل والصادرات والرمز
المميز. والحجم المسمى، كما أعلاه، يبدأ مملوكًا لمستخدم الصورة. أما مجلد من المضيف يُركَّب هناك فيجب أن يكون
قابلًا للكتابة لـ uid 10001:

```bash
sudo mkdir -p /srv/signallab && sudo chown 10001:10001 /srv/signallab
docker run -d --name signallab --network host -v /srv/signallab:/data ghcr.io/proanima/signallab:[[version]]
```

## دون Docker {#binary}

يُنشر الخادم صورةً. ولتشغيل `signal-lab-server` مباشرة يُبنى من المصدر (انظر [البناء](../../develop/building.md)):

```bash
npm install
npm run build                            # the interface, into dist/
cargo run --release -p signal-lab-server
```

يقدّم `dist/` على `http://127.0.0.1:1430`، لهذا الجهاز وحده، دون حاجة إلى رمز مميز. ويُضاف `--listen` ورمز
لفتحه للشبكة.

## الخيارات {#options}

لكل خيار متغير بيئة، للحاويات. والخيارات تغلب المتغيرات.

| الخيار | المتغير | الافتراضي | ما يفعله |
| --- | --- | --- | --- |
| `--listen IP:PORT` | `SIGNALLAB_LISTEN` | `127.0.0.1:1430` | موضع الاستماع. أي عنوان غير عنوان الاسترجاع يحتاج إلى رمز مميز. |
| `--token-file PATH` | `SIGNALLAB_TOKEN_FILE` | | ملف يحوي رمز الوصول المميز (سر Docker مثلًا). |
| `--token TOKEN` | `SIGNALLAB_TOKEN` | | رمز الوصول المميز نفسه. يُفضَّل الملف: فالوسائط ظاهرة لمستخدمين آخرين على الجهاز. |
| `--generate-token` | `SIGNALLAB_GENERATE_TOKEN` | معطل | حين لا يُعطى رمز، وعلى عنوان وراء الاسترجاع: يُستخدم الرمز المحفوظ في `<data folder>/token`، ويُنشأ عند أول بدء. |
| `--data-dir PATH` | `SIGNALLAB_DATA_DIR` | `Documents/SignalLab` في المجلد الرئيسي للمستخدم | مجلد البيانات. |
| `--secrets-dir PATH` | `SIGNALLAB_SECRETS_DIR` | `/run/secrets/signallab` | مجلد أسرار للقراءة فقط، ملف لكل اسم. |
| `--ui-dir PATH` | `SIGNALLAB_UI_DIR` | `ui` بجانب البرنامج، وإلا `./dist` | الواجهة المبنية. ودونها لا تُقدَّم إلا واجهة API. |
| `--allowed-host NAME` | `SIGNALLAB_ALLOWED_HOSTS` | | أسماء المضيف التي يجوز بلوغ الخادم بها، مفصولة بفواصل. وهي وأسماء الاسترجاع مقبولة دائمًا، مع رمز مميز أو دونه؛ ومع عدم إعطاء أي منها، يجيب الخادم ذو الرمز عن أي اسم، وغير ذي الرمز عن أسماء الاسترجاع فقط (انظر [أسماء المضيف](security.md#hosts)). |
| `--secure-cookie` | `SIGNALLAB_SECURE_COOKIE` | معطل | إرسال ملف تعريف ارتباط الجلسة عبر HTTPS فقط. يُعيَّن خلف وكيل HTTPS. |
| `--log FILTER` | `SIGNALLAB_LOG` | `info` | ما يُسجَّل: `error` أو `warn` أو `info` أو `debug`، أو لكل وحدة (`signal_lab_server=debug`). |
| `--log-format text\|json` | `SIGNALLAB_LOG_FORMAT` | `text` | أسطر السجل نصًّا، أو كائن JSON واحدًا في كل سطر. |

المفاتيح تأخذ `true` أو `false` من متغيرها: `SIGNALLAB_GENERATE_TOKEN=true`.

| الأمر | ما يفعله |
| --- | --- |
| `signal-lab-server token` | يطبع رمزًا مميزًا عشوائيًّا جديدًا: 64 حرفًا ست عشريًّا. |
| `signal-lab-server healthcheck` | يخرج بـ `0` حين يجيب خادم على `--listen` (فحص سلامة الصورة). |
| `signal-lab-server --version` | يطبع الإصدار. |
| `signal-lab-server --help` | يطبع كل خيار. |

**رموز الخروج:** `0` عند الإيقاف بـ <kbd>Ctrl</kbd>+<kbd>C</kbd> أو `SIGTERM`؛ و`2` حين تُرفض الإعدادات (لا رمز
مميز على عنوان يمكن بلوغه، أو رمز أقصر مما يلزم، أو رمز أُعطي مرتين، أو ملف رمز تتعذر قراءته، أو
`--generate-token` دون مجلد بيانات)؛ و`1` حين يتعذر الاستماع على العنوان، أو تتعذر الكتابة في مجلد البيانات.
ويُطبع السبب على stderr.

## رمز الوصول المميز {#token}

دون رمز مميز لا يستمع الخادم إلا على عنوان الاسترجاع ولا يخدم إلا هذا الجهاز. وعلى أي عنوان آخر يحتاج إلى
واحد، ويرفض أن يبدأ بدونه. وهناك ثلاث طرق لإعطائه:

| الطريقة | متى تُستخدم |
| --- | --- |
| `--generate-token` (مفعّل في الصورة) | لا شيء يُعدّ: عند أول بدء ينشئ الخادم رمزًا ويحفظه في `<data folder>/token` — مقروءًا لمستخدمه وحده — ويطبعه مرة واحدة في السجل. وتعيد البدايات اللاحقة استخدامه، فتظل المتصفحات والسكربتات المسجَّل دخولها تعمل عبر إعادات التشغيل والتحديثات. ويحتاج إلى مجلد بيانات (`--data-dir`). |
| `--token-file PATH` | رمزك الخاص في ملف، كسر Docker. وفاصل السطر في آخره ليس من الرمز. |
| `SIGNALLAB_TOKEN` | الرمز في البيئة. |

للرمز المميز **24** حرفًا على الأقل ولا مسافات فيه ولا فواصل أسطر؛ ويُعطى بطريقة واحدة فقط. و`signal-lab-server token`
يصنع رمزًا جيدًا:

```bash
docker run --rm ghcr.io/proanima/signallab:[[version]] token > signallab_token.txt
```

ويسلّمه Compose سرًّا (ويجب أن يكون الملف مقروءًا لـ uid 10001):

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

الرمز المعطى صراحةً يغلب `--generate-token`، ولا يُنشأ شيء عندئذ. ولتغيير رمز مُنشأ يُوقَف الخادم ويُحذف
`<data folder>/token` ويُبدأ من جديد: فينشئ رمزًا جديدًا ويطبعه. وملف الرمز التالف يُبلَّغ عنه ولا يُستبدل أبدًا.

## تسجيل الدخول {#sign-in}

يُفتح `http://<host>:1430`. الخادم ذو الرمز المميز يرسل المتصفح أولًا إلى صفحة تسجيل الدخول، بلغة المتصفح؛
ويُلصق الرمز مرة واحدة، ويظل المتصفح مسجَّلًا دخوله 7 أيام. والزر [[ui:app.signOut]] في الواجهة ينهي الجلسة.
وتُحفظ الجلسات في ذاكرة الخادم: وإعادة التشغيل تُخرج الجميع.

وعلى الاسترجاع دون رمز مميز لا يوجد تسجيل دخول.

تُرسل السكربتات الرمز مع كل طلب، بالصيغة `Authorization: Bearer <token>`:

```bash
curl -fsS http://192.0.2.10:1430/api/invoke/app_info \
  -H "Authorization: Bearer $(cat signallab_token.txt)" \
  -H "Content-Type: application/json" -d 'null'
```

انظر [واجهة HTTP API](../api/index.md)، و[أمان الخادم](security.md) للقواعد التي يقوم عليها كل هذا.

## مجلد البيانات {#data}

يحتفظ الخادم بملفاته في مجلد البيانات: التجربة ومكتبتا الإشارات والمحاكيات وتقارير التشغيل والصادرات والرمز
المنشأ. وهو `--data-dir` (`SIGNALLAB_DATA_DIR`)، و`/data` في الصورة، وإلا `Documents/SignalLab` في المجلد
الرئيسي للمستخدم الذي يعمل به. ومجلد البيانات المعطى بـ `--data-dir` يُنشأ إن كان مفقودًا ويُفحص عند البدء:
فإذا تعذرت على الخادم الكتابة هناك توقف وسمّى المجلد. انظر [الملفات](../reference/files.md).

وتأتي التنزيلات (التقارير والصادرات) من داخل هذا المجلد فقط.

## الأسرار {#secrets}

تقرأ التجارب الأسرار بالصيغة `{{secret.NAME}}`. وعلى الخادم تكون للقراءة فقط، من:

1. متغير البيئة `SIGNALLAB_SECRET_NAME`، وإلا
2. الملف `NAME` في مجلد الأسرار (`--secrets-dir`، وافتراضيًّا `/run/secrets/signallab` — تخطيط أسرار Docker).

فاصل السطر في آخر الملف ليس من القيمة؛ والقيمة الفارغة تُعدّ غير معيَّنة؛ والقيمة 16 KiB على الأكثر. والأسماء
أحرف وأرقام و`_`، لا تبدأ برقم. وتعرض الواجهة أي الأسرار معيَّنة على الخادم، لكن تعيين سر من المتصفح مرفوض —
فالقيم لا تذهب أبدًا إلى مكان أضعف، ولا تخرج ثانية (انظر [الأسرار](security.md#secrets)).

ومع Compose، ملف سر لكل اسم:

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

ويجب أن يكون الملف مقروءًا لـ uid 10001.

## خلف وكيل HTTPS {#https}

يتحدث الخادم HTTP عاديًّا. ولاستخدام HTTPS يوضع أمامه وكيل عكسي (Caddy أو nginx أو Traefik):

- يمرر ترويسة `Host` دون تغيير؛
- ويمرر ترقيات WebSocket (تُبقي الواجهة واحدة مفتوحة، إلى `/api/events`)؛

ويبدأ الخادم بـ `--secure-cookie`، فيسافر ملف تعريف ارتباط الجلسة عبر HTTPS فقط، وبـ `--allowed-host` معيَّنًا
على الاسم الذي يستخدمه الناس.

## التحديث {#update}

[[ui:update.server]]: لا شأن لمحدّث التطبيق المكتبي به.

- المثبَّت بالسكربت: يُشغَّل الأمر نفسه مرة أخرى.
- مع Compose: `docker compose pull && docker compose up -d`.
- مع `docker run`: تُسحب الصورة الجديدة، ثم تُزال الحاوية وتُبدأ من جديد بالحجم نفسه.

البيانات والرمز المميز في الحجم، فتبقى.

## فحص السلامة {#health}

`GET /api/health` يجيب الجميع، دون رمز مميز:

```bash
curl -s http://127.0.0.1:1430/api/health
# {"auth":true,"status":"ok","version":"[[version]]"}
```

يبيّن `auth` هل يطلب الخادم رمزًا. والأمر `signal-lab-server healthcheck` يسأل السؤال نفسه على هذا الجهاز
ويخرج بـ `0` حين يجيب؛ وتشغّله الصورة كل 30 ثانية (مهلة 5 ثوان، و3 محاولات)، فيعرض `docker ps` الحاوية سليمة.

## السجلات {#logs}

يسجّل الخادم في stdout: `docker logs -f signallab`. وعند المستوى الافتراضي (`info`) يذكر أين يستمع وهل يحتاج إلى
رمز، ومجلدي بياناته وواجهته، وكل مهمة بدأت — المراقبات والمولدات والعواصف والمسوح والتشغيلات والمحاكيات — مع
عنوان العميل، وكل تسجيل دخول، وكل تسجيل دخول برمز خاطئ (تحذيرًا)، وحين تتخلف صفحة عن الأحداث. والمستوى
`--log debug` يضيف كل أمر. ولا تُلوَّن الأسطر إلا على طرفية (ولا تُلوَّن أبدًا مع تعيين `NO_COLOR`)؛ و
`--log-format json` يكتب كائن JSON واحدًا في كل سطر لجامع سجلات.

## الإيقاف {#stop}

<kbd>Ctrl</kbd>+<kbd>C</kbd> أو `docker stop` أو `SIGTERM` يغلق اتصال كل صفحة، ويوقف كل مهمة — والتشغيل الجاري
ينتهي موقوفًا — ويخرج بـ `0`. وتمنحه ملفات compose 15 ثانية لذلك.

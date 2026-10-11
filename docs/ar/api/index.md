---
title: واجهة HTTP API
description: التحكم في خادم Signal Lab من السكربتات وCI والأدوات الأخرى بالأوامر والأحداث نفسها التي تستخدمها واجهته.
---

# واجهة HTTP API

للتحكم في Signal Lab من سكربت أو خط CI أو أداة أخرى، يُخاطَب خادم Signal Lab (`signal-lab-server`، أو صورة Docker)
عبر HTTP. والواجهة التي يعرضها الخادم في المتصفح تستخدم هذه الواجهة نفسها بالضبط: كل زر استدعاء إلى
`/api/invoke/<command>`، وكل رقم حي يصل على `/api/events`. فكل ما يستطيع شخص فعله على صفحة الخادم يستطيع السكربت فعله
أيضًا.

ليس للتطبيق المكتبي واجهة HTTP API: فنافذته تبلغ محركه داخل التطبيق. وللأتمتة على سطح المكتب يُشغَّل خادم على
الجهاز نفسه (انظر [الخادم](../server/index.md)) أو يُستخدم سطر الأوامر [`signallab`](../automation/cli.md).

## نقاط النهاية {#endpoints}

| الطريقة والمسار | ما يفعله | الرمز المميز |
| --- | --- | --- |
| `GET /api/health` | هل يجيب الخادم، وإصداره، وهل يطلب رمزًا | غير مطلوب |
| `POST /api/invoke/<command>` | ينفذ أمرًا واحدًا من أوامر المحرك: وسائط JSON داخلة ونتيجة JSON خارجة ([الأوامر](commands.md)) | مطلوب |
| `POST /api/run` | يشغّل تجربة إلى نهايتها: النتيجة، أو خطواتها أسطرًا ([التشغيلات](run.md)) | مطلوب |
| `GET /api/events` | WebSocket بكل أحداث المحرك ([الأحداث](events.md)) | مطلوب |
| `GET /api/files?path=…` | ملف كتبه المحرك في مجلد البيانات، تنزيلًا | مطلوب |
| `GET /api/openapi.json` | وصف هذه الواجهة بصيغة OpenAPI 3.1 | مطلوب |
| `GET /login`، `POST /login` | صفحة تسجيل الدخول ونموذجه للمتصفحات | غير مطلوب |
| `POST /logout` | ينهي جلسة متصفح | جلسة |

أي مسار آخر تحت `/api/` يجيب `404` مع الرمز `api.not_found`؛ والطريقة التي لا يقبلها المسار
(`GET /api/invoke/…`) هي `405`، بمتن فارغ. وكل ما عدا ذلك هو الواجهة؛ وعلى خادم له رمز مميز يُرسل المتصفح الذي
بلا جلسة إلى `/login` أولًا.

## العنوان الأساسي {#base-url}

يستمع الخادم على `http://127.0.0.1:1430` ما لم يُقل غير ذلك بـ `--listen` (أو `SIGNALLAB_LISTEN`). وتستمع صورة
Docker على كل بطاقات الشبكة، `0.0.0.0:1430`. وتستخدم الأمثلة في هذه الصفحات:

```bash
SERVER=http://127.0.0.1:1430
```

يتحدث الخادم HTTP عاديًّا. ولاستخدام HTTPS يوضع أمامه وكيل عكسي ينهي TLS، ويبدأ الخادم بـ `--secure-cookie`.

## المصادقة {#authentication}

الخادم الذي بدأ دون رمز مميز يستمع على الاسترجاع فقط ولا يحتاج إلى مصادقة: يجوز لأي شخص على ذلك الجهاز
استخدامه. أما الخادم الذي يستطيع آخرون بلوغه فله رمز دائمًا، وحينئذ يجب أن يحمله كل طلب عدا `/api/health`
و`/login`.

**تُرسل السكربتات** الرمز في الترويسة `Authorization`:

```bash
TOKEN=$(cat token.txt)
curl -fsS "$SERVER/api/invoke/jobs_list" -X POST \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json"
```

يجب أن تكون الترويسة `Bearer` بالضبط، ثم مسافة واحدة، ثم الرمز. والرمز المفقود أو الخاطئ هو `401` مع الرمز
`auth.required`.

**تسجل المتصفحات** الدخول مرة واحدة في `/login` بالرمز وتحصل على ملف تعريف ارتباط للجلسة،
`signallab_session`: `HttpOnly` و`SameSite=Strict`، يُحتفظ به 7 أيام، و`Secure` حين يعمل الخادم بـ `--secure-cookie`.
و`POST /logout` ينهيه. والرمز الخاطئ في نموذج تسجيل الدخول يكلّف ثانية قبل الجواب، مما يبطئ التخمين. وتعيش
الجلسات في ذاكرة الخادم: فإعادة التشغيل تُخرج كل متصفح، ولا تتأثر السكربتات التي تحمل الرمز. ويحتفظ الخادم بما
لا يزيد على 1024 جلسة؛ وبعدها تذهب الأقدم.

ومن أين يأتي الرمز مسألة إعداد الخادم (`--token-file`، أو `SIGNALLAB_TOKEN`، أو ملف `token` الذي ينشئه
`--generate-token` في مجلد البيانات): انظر [أمان الخادم](../server/security.md). وللرمز 24 حرفًا على الأقل ولا
مسافات فيه؛ و`signal-lab-server token` يطبع رمزًا جديدًا.

::: warning
كل من يملك الرمز يستطيع جعل الخادم يرسل حركة مرور. يُترك الملف الذي هو فيه مقروءًا لك وحدك، ولا يوضع أبدًا في
عنوان URL — فالخادم لا يقرأ رمزًا هناك على أي حال.
:::

## Host وOrigin {#host-origin}

يجري فحصان قبل أي شيء آخر، على كل مسار، بما فيه `/api/health`.

**`Host`.** يجب أن تسمّي ترويسة `Host` هذا الخادم:

- أسماء الاسترجاع تمر دائمًا: `localhost`، والأسماء المنتهية بـ `.localhost`، و`127.x.x.x` و`[::1]`.
- الأسماء المعطاة بـ `--allowed-host` (أو `SIGNALLAB_ALLOWED_HOSTS`) تمر.
- الخادم ذو الرمز بلا `--allowed-host` يجيب عن أي اسم.

وأي شيء آخر هو `403` مع `auth.host`. يُخاطَب الخادم باسم يقبله: فـ `curl` يرسل مضيف عنوان URL الذي يُعطى له.

**`Origin`.** الطلب الذي يغيّر شيئًا (أي طريقة غير `GET` و`HEAD`) وترقية WebSocket يجب أن يأتيا من صفحة الخادم
نفسه حين يحملان ترويسة `Origin`: يجب أن يساوي مضيفها ومنفذها `Host`. وإلا فالجواب `403` مع `auth.origin`؛
و`Origin: null` مرفوض أيضًا. ولا ترسل السكربتات و`curl` `Origin` فتمر من هذا الفحص — وتظل تحتاج إلى الرمز.

**JSON فقط.** يقبل `POST /api/invoke/…` و`POST /api/run` `Content-Type: application/json` (ولا بأس بوسائط مثل
`; charset=utf-8`). وأي شيء آخر هو `415` مع `command.json_required`. وصفحة ويب في موقع آخر لا تستطيع إرسال ذلك
دون أن تسأل الخادم أولًا، والخادم لا يوافق أبدًا.

## استدعاء أمر {#invoke}

```http
POST /api/invoke/<command>
Content-Type: application/json

{ "argument": "value", … }
```

- المتن كائن JSON واحد بوسائط الأمر. والأمر بلا وسائط يقبل `{}` أو متنًا فارغًا (وترويسة `Content-Type` تظل
  مطلوبة).
- أسماء الوسائط بنمط camelCase، كما ترسلها الواجهة: `jobId` و`nodeId`. والكائنات الممررة وسيطًا (`config`
  و`request` و`document` و`library`…) تحتفظ بأسماء الحقول التي يكتبها المحرك، وهي في الغالب بنمط snake_case:
  `timeout_ms` و`port_start`.
- الوسيط الذي لا يعرفه الأمر خطأ ولا يُتجاهل أبدًا: `422` مع `command.args_invalid` يسمّي الأمر، وبكلمات المحلِّل
  في `detail`. وكذلك الوسيط المطلوب المتروك. والأمر بلا وسائط لا يقرأ المتن إطلاقًا.
- الوسيط الاختياري يجوز تركه أو إرساله `null`.
- الجواب `200` بنتيجة الأمر بصيغة JSON. والأمر الذي لا شيء عنده ليعيده يجيب `null`.

كل أمر، مع وسائطه ونتيجته، في [الأوامر](commands.md).

## الأخطاء {#errors}

| الحالة | متى | المتن |
| --- | --- | --- |
| `200` | نُفذ الأمر؛ أو بدأ `/api/run` التشغيل | النتيجة |
| `400` | المتن ليس JSON؛ أو تعذرت قراءة طلب تشغيل | `EngineError`: `command.args_invalid` و`api.run_invalid` و`api.run_source` |
| `400` | `/api/files` دون `path` | نص عادي من خادم الويب، لا `EngineError` |
| `401` | لا رمز، أو رمز خاطئ | `EngineError`: `auth.required` |
| `403` | `Host` أو `Origin` يرفضه الخادم | `EngineError`: `auth.host` و`auth.origin` |
| `404` | لا مسار API كهذا؛ أو ملف ليس في مجلد البيانات | `EngineError`: `api.not_found` و`file.not_found` |
| `405` | طريقة لا يقبلها المسار | فارغ |
| `413` | متن طلب يزيد على 24 MiB | نص عادي من خادم الويب، لا `EngineError` |
| `413` | تنزيل يزيد على 256 MiB | `EngineError`: `file.too_large` |
| `415` | ليس `application/json` | `EngineError`: `command.json_required` |
| `422` | فشل الأمر، أو تعذر بدء التشغيل | `EngineError`: أي رمز من رموز المحرك |
| `500` | تعذرت قراءة ملف | `EngineError`: `file.io` |

اسم الأمر المجهول هو `422` مع `command.unknown`.

لكل إخفاق يبلغ عنه المحرك شكل واحد، هو `EngineError`:

```json
{
  "code": "transport.refused",
  "params": { "target": "http://127.0.0.1:8080/" },
  "node": "request",
  "field": { "key": "url" },
  "detail": "error sending request for url (http://127.0.0.1:8080/): tcp connect error: Connection refused (os error 111)"
}
```

| الحقل | ما هو |
| --- | --- |
| `code` | ما الذي حدث خطأً: معرّف ثابت. وكل رمز ورسالته مسرودان في [رسائل الأخطاء](../reference/errors.md)، مجمّعين بحسب ما قبل النقطة (مثل [`transport`](../reference/errors.md#transport)) |
| `params` | القيم التي تسمّيها الرسالة، كلها سلاسل. ويُحذف حين لا توجد |
| `node` | عقدة التجربة التي يتعلق بها. ويُحذف حين لا توجد |
| `field` | الحقل الذي يتعلق به: `key` (مسمًّى في [الحقول](../reference/errors.md#fields)) و`index`، يبدأ من 1، للحقول المتكررة مثل ترويسة. ويُحذف حين لا يوجد |
| `detail` | كلمات نظام التشغيل أو المحلِّل أو المكتبة نفسها، بالإنجليزية. ويُحذف حين لا توجد |

يُتفرع على `code`، لا على `detail` أبدًا. وقيم الأسرار التي يستخدمها تشغيل أو إرسال منفرد محجوبة (`••••`) في كل
خطأ يبلغ عنه.

والطلب الذي يبلغ خادمه لكنه يلقى حالة خطأ، أو لا يلقى شيئًا، ليس أمرًا فاشلًا: فـ `http_request` يجيب `200`
بالاستجابة، وتقول `ok` و`error` و`cause` ما حدث. انظر [`http_request`](commands.md#http_request).

## الحدود {#limits}

| ما | الحد | عند الحد |
| --- | --- | --- |
| متن طلب | 24 MiB | `413` |
| مستند تجربة | 4 MiB | `file.too_large` |
| تنزيل من `/api/files` | 256 MiB | `413` و`file.too_large` |
| مدة تشغيل | 300 ثانية | يفشل التشغيل بـ `run.timeout` |
| نموذج الملاحظات (`feedback_send`) | 15 MiB إجمالًا | `feedback.too_large` |
| الأحداث المنتظرة لـ WebSocket واحد | 4096 | يتلقى [`server://lagged`](events.md#event-server-lagged) مع عدد ما فاته |

## الأحداث {#events}

`GET /api/events` المرقّى إلى WebSocket يبث كل حدث يرسله المحرك — خطوات تشغيل، ونهاية مهمة، ورسائل مراقب،
وإطارات الفاحص — إلى كل عميل متصل، رسائلَ نصية:

```json
{ "event": "job://ended", "payload": { "job_id": 7, "kind": "storm", "error": null } }
```

يقبل قواعد الرمز المميز و`Origin` نفسها كبقية الواجهة. وكل قناة وحمولتها في [الأحداث](events.md).

## الملفات {#files}

`GET /api/files?path=<path>` ينزّل ملفًّا كتبه المحرك في مجلد بيانات الخادم: تقرير تشغيل (`report_path` في نتيجة
تشغيل)، أو صادرات التجربة أو الفاحص، أو مكتبة الإشارات أو المحاكيات. و`path` هو المسار الذي أعطاك إياه المحرك، على
الخادم (مرمَّزًا في URL):

```bash
curl -fsS -G "$SERVER/api/files" --data-urlencode "path=/data/runs/run-1759600000000-3.json" \
  -H "Authorization: Bearer $TOKEN" -o report.json
```

- لا تُقدَّم إلا الملفات داخل مجلد البيانات. وأي شيء آخر، أو مجلد، أو ملف غير موجود، هو `404` مع
  `file.not_found`.
- الجواب `application/octet-stream` مع `Content-Disposition: attachment`. وتصير أحرف اسم الملف غير الأحرف
  والأرقام و`.` و`_` و`-` إلى `_`.
- الملف الذي يزيد على 256 MiB هو `413` مع `file.too_large`.

وما يحويه مجلد البيانات في [الملفات والمجلدات](../reference/files.md).

## السلامة {#health}

`GET /api/health` مفتوح: لا يحتاج إلى رمز مميز، بل إلى `Host` يقبله الخادم فقط.

```bash
curl -fsS "$SERVER/api/health"
```

```json
{ "status": "ok", "version": "[[version]]", "auth": true }
```

يبيّن `auth` هل تحتاج الطلبات إلى رمز. و`signal-lab-server healthcheck` يسأل العنوان نفسه الذي يستمع عليه
الخادم ويخرج بـ `0` حين يجيب؛ وفحص سلامة صورة Docker يشغّله.

## وصف OpenAPI {#openapi}

`GET /api/openapi.json` يصف هذه الواجهة بصيغة OpenAPI 3.1: نقاط النهاية، وكل أمر بوسائطه، وطلب التشغيل
ونتيجته. ويحتاج إلى الرمز المميز كبقية `/api/`. وهذه الصفحات هي المرجع الكامل؛ وحيث يختلف الاثنان تتبع
هذه الصفحات المحرك.

## المهام {#jobs}

العمل الطويل — مراقب، أو مولِّد، أو دفعة، أو مرحّل، أو اتصال، أو محاكٍ، أو تشغيل — مهمة. والأمر الذي يبدأ
واحدة يعيد `JobInfo` الخاص بها حال عملها:

```json
{ "id": 4, "kind": "osc-monitor", "label": "OSC monitor 0.0.0.0:9000", "params": { "bind": "0.0.0.0:9000" }, "started_ms": 1759600000000 }
```

| الحقل | ما هو |
| --- | --- |
| `id` | رقم المهمة، فريد ما دام الخادم يعمل؛ وتأخذه أوامر أخرى بوصفه `jobId` أو `id` |
| `kind` | `experiment` أو `osc-monitor` أو `osc-gen` أو `http-burst` أو `netsim` أو `storm` أو `scan` أو `beacon` أو `discovery` أو `mqtt` أو `websocket` أو `emulator` |
| `label` | سطر إنجليزي واحد، للسجلات |
| `params` | القيم التي تتألف منها التسمية (الهدف، العنوان، المضيف…). ويُحذف حين لا توجد |
| `started_ms` | وقت البدء، بالمللي ثانية منذ 1970 |

هذه الأوامر تبدأ مهمة: `experiment_start` و`osc_monitor_start` و`osc_generator_start` و`http_burst_start` و`netsim_start`
و`storm_start` و`scan_start` و`broadcast_beacon_start` و`discovery_start` و`mqtt_connect` و`ws_connect`
و`emulator_start`. ويسجّل الخادم كل بدء مع عنوان العميل الذي طلبه؛ و`POST /api/run` يبدأ مهمة أيضًا.

- [`jobs_list`](commands.md#jobs_list) يسرد المهام الجارية، و[`job_stop`](commands.md#job_stop) يوقف واحدة،
  و[`jobs_stop_all`](commands.md#jobs_stop_all) يوقفها كلها.
- المهمة التي تنتهي من تلقاء نفسها، أو تفشل، ترسل [`job://ended`](events.md#event-job-ended). والمهمة التي
  تُوقفها لا ترسل شيئًا بعد: فإجابة `job_stop` بـ `true` هي التأكيد.
- المهام ملك الخادم، لا العميل الذي بدأها. وإغلاق الصفحة أو إنهاء السكربت لا يوقفها، ويراها كل عميل ويستطيع
  إيقافها، والخادم الذي يُغلق يوقفها كلها.

## مثال كامل {#example}

يُسأل هل الخادم يعمل، ويُبدأ محاكي HTTP صغير بأمر واحد، وتُشغَّل ضده تجربة `http-check` المضمَّنة وتُنتظر
نتيجتها، ثم يُوقف المحاكي. ويستخرج `jq` الحقول من الأجوبة.

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)            # leave out with a loopback server without a token
AUTH="Authorization: Bearer $TOKEN"
JSON="Content-Type: application/json"

# 1. Up? Which version? Does it want a token?
curl -fsS "$SERVER/api/health"
# {"status":"ok","version":"[[version]]","auth":true}

# 2. One command: an HTTP emulator on 127.0.0.1:8080 that answers GET / with 200
EMULATOR=$(curl -fsS -X POST "$SERVER/api/invoke/emulator_start" -H "$AUTH" -H "$JSON" -d '{
  "emulator": { "name": "Example", "bind": "127.0.0.1:8080", "protocol": "http",
                "routes": [ { "method": "GET", "path": "/", "responses": [ { "body": "ok" } ] } ] }
}' | jq .id)

# 3. Run the bundled experiment that expects 200 from http://127.0.0.1:8080/, and wait
curl -sS -X POST "$SERVER/api/run" -H "$AUTH" -H "$JSON" -d '{"template":"http-check"}' \
  | jq '{outcome, error, report_path}'
# {"outcome":"passed","error":null,"report_path":"/data/runs/run-1759600000000-2.json"}

# 4. Stop the emulator
curl -fsS -X POST "$SERVER/api/invoke/job_stop" -H "$AUTH" -H "$JSON" -d "{\"id\":$EMULATOR}"
# true
```

`curl -f` يحوّل حالة الخطأ إلى أمر فاشل؛ ويُترك (كما في الخطوة 3) لرؤية `EngineError` في المتن.

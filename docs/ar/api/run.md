---
title: التشغيلات
description: يشغّل POST /api/run تجربة على خادم Signal Lab ويجيب بنتيجتها، أو يبث خطواتها أسطر NDJSON.
---

# تشغيل تجربة

لتشغيل تجربة على خادم من سكربت أو خط CI ومعرفة كيف سارت، تُرسل إلى `POST /api/run`. يشغّلها الخادم إلى نهايتها
ويجيب بالنتيجة — أو، إن طُلب ذلك، بكل خطوة كلما حدثت. وهذا ما يستخدمه
[`signallab run --server`](../automation/cli.md#cli-run).

وهو التشغيل نفسه الذي يجريه [[ui:exp.run]] في المحرر و[`experiment_start`](commands.md#experiment_start): مهمة
تراها كل صفحة مفتوحة وتستطيع إيقافها، والأحداث نفسها، والتقرير نفسه في مجلد البيانات.

## الطلب {#request}

```http
POST /api/run
Authorization: Bearer <token>
Content-Type: application/json

{ "template": "osc-ping-reply", "overrides": { "device": "192.0.2.20:9000" }, "seed": 42, "timeout": 30 }
```

يسمّي المتن تجربة واحدة — `document` أو `template`، لا كليهما — وما تُشغَّل به:

| الحقل | النوع | الافتراضي | المعنى |
| --- | --- | --- | --- |
| `document` | كائن | — | تجربة كما يحفظها المحرر ويصدّرها. وتُرحَّل الإصدارات الأقدم، كما عند فتح ملف |
| `template` | سلسلة | — | قالب مضمَّن باسم ملفه، مع `.json` أو دونها (أدناه) |
| `overrides` | كائن | `{}` | قيم معاملات لهذا التشغيل وحده. يجوز أن تكون القيم سلاسل أو أرقامًا أو قيمًا منطقية؛ ويجب أن تكون كل منها معاملًا في التجربة |
| `profile` | سلسلة | ملف المستند | التشغيل بهذا الملف؛ و`""` يشغّل بالقيم الافتراضية |
| `seed` | رقم | بذرة المستند، وإلا جديدة | من 0 إلى 9007199254740991؛ والبذرة نفسها تسحب القيم العشوائية نفسها |
| `timeout` | رقم | `300` | الثواني قبل أن يفشل التشغيل بـ `run.timeout`؛ من 1 إلى 300 |

الحقل الذي لا يعرفه الخادم يُرفض (`400`، `api.run_invalid`). ويُقرأ المستند نفسه كما يُقرأ ملف مفتوح، فهو 4 MiB على
الأكثر.

القوالب المضمَّنة — التجارب التي يعرضها المحرر تحت [[ui:exp.templates]]:

| `template` | ما هو |
| --- | --- |
| `empty` | Start وEnd |
| `http-check` | طلب GET إلى `http://127.0.0.1:8080/`، ثم فحص للحالة 200 |
| `status-branch` | طلب GET إلى `http://127.0.0.1:8080/`؛ وعند 200 رسالة OSC، وإلا تأخير 500 مللي ثانية |
| `parallel-flows` | [[ui:exp.node.fork]] إلى طلب GET إلى `http://127.0.0.1:8080/` وسطر سجل، جنبًا إلى جنب، ثم [[ui:exp.node.join]] |
| `osc-ping-reply` | رسالة OSC `/ping` إلى المعامل `device` (`127.0.0.1:9000`)، ثم انتظار ثانيتين لـ `/pong` على `127.0.0.1:9001` |
| `poll-until-ready` | [[ui:exp.node.loop]] تسأل `device` عن `/status` عبر OSC إلى أن يجيب `ready`، عشر مرات على الأكثر |
| `flaky-api` | واجهة API محاكاة (المعامل `api`) تفشل قبل أن تعمل، تُسأل في [[ui:exp.node.loop]] إلى أن تجيب 200 |
| `fault-phases` | جهاز UDP خلف مرحّل إضعاف، يُرسل إليه 8 ثوان بينما يصير المرحّل نظيفًا ثم فاقدًا ثم غير متصل ثم نظيفًا من جديد |
| `dependency-outage` | واجهة API محاكاة (المعامل `api`) تُعطَّل ثانيتين بينما تسألها [[ui:exp.node.loop]] إلى أن تجيب 200 من جديد |
| `websocket-echo` | اتصال بالمعامل `service` (`ws://127.0.0.1:9001/echo`)، ورسالة، وانتظار صداها، وفحص، وإغلاق |

يُفتح أحدها تحت [[ui:exp.templates]] لرؤية عقده ومعاملاته؛ وانظر [التجارب](../experiments/index.md).

## النتيجة {#result}

افتراضيًّا الجواب `200` مع `Content-Type: application/json`، يُرسل حين ينتهي التشغيل: كائن JSON واحد، هو نتيجة
التشغيل.

```json
{
  "job_id": 12,
  "experiment": "OSC ping → reply",
  "outcome": "passed",
  "seed": 42,
  "profile": null,
  "overridden": true,
  "params": { "device": "192.0.2.20:9000" },
  "started_ms": 1759600000000,
  "ended_ms": 1759600000310,
  "steps": [ { "job_id": 12, "ts": 1759600000001, "node_id": "start", "state": "running", "detail": "", "message_key": null, "message_params": null }, … ],
  "report_path": "/data/runs/run-1759600000000-12.json"
}
```

| الحقل | النوع | المعنى |
| --- | --- | --- |
| `job_id` | رقم | مهمة التشغيل |
| `experiment` | سلسلة | اسم التجربة |
| `outcome` | سلسلة | `passed` أو `failed` أو `stopped` |
| `seed` | رقم | البذرة التي عمل بها: تُعاد باسم `seed` لسحب القيم نفسها |
| `profile` | سلسلة أو null | ملف التعريف الذي عمل به |
| `overridden` | قيمة منطقية | جاءت بعض القيم من `overrides` |
| `params` | كائن | كل قيمة معامل استخدمها التشغيل |
| `started_ms`، `ended_ms` | رقم | مللي ثانية منذ 1970 |
| `error` | `EngineError` | سبب الفشل: أول فشل فيه. ويُحذف حين نجح |
| `steps` | كائنات[] | كل خطوة بترتيب حدوثها، مثل [`experiment://step`](events.md#event-experiment-step) |
| `emulators` | كائنات[] | ما تلقته كل عقدة [[ui:exp.node.emulator]] وأجابت به: `node` و`name` و`protocol` و`local` و`counts`. ويُحذف حين لا توجد |
| `impairments` | كائنات[] | ما فعله مرحّل كل عقدة [[ui:exp.node.impairment]]، مرحلةً مرحلة. ويُحذف حين لا توجد |
| `report_path` | سلسلة | تقرير التشغيل على الخادم؛ وينزَّل بـ [`/api/files`](index.md#files). ويُحذف حين لم يُكتب تقرير |
| `report_error` | `EngineError` | سبب تعذر كتابة التقرير. ويُحذف وإلا |

قيم الأسرار محجوبة في كل ذلك. ويحوي ملف التقرير الخطوات نفسها؛ انظر [التشغيلات والتقارير](../experiments/runs.md).

وأثناء استمرار التشغيل يرسل الخادم مسافة كل 15 ثانية. وتتجاهل JSON المسافات البيضاء قبل القيمة، فتظل النتيجة
تُحلَّل، ولا يحسب وكيلٌ تشغيلًا طويلًا صامتًا اتصالًا ميتًا.

## متابعة الخطوات {#lines}

لرؤية الخطوات كلما حدثت يُطلب NDJSON:

```http
Accept: application/x-ndjson
```

الجواب `200` مع `Content-Type: application/x-ndjson`: كائن JSON في كل سطر، ولكل منها `type`.

| `type` | متى | بقية السطر |
| --- | --- | --- |
| `started` | أولًا، مرة واحدة | `job_id` و`experiment` و`seed` و`profile` و`overridden` و`started_ms` |
| `step` | كل خطوة | الخطوة، مثل [`experiment://step`](events.md#event-experiment-step) |
| `heartbeat` | كل 15 ثانية | لا شيء |
| `ended` | أخيرًا، مرة واحدة | النتيجة، كما سبق |

```text
{"job_id":12,"experiment":"OSC ping → reply","seed":42,"profile":null,"overridden":true,"started_ms":1759600000000,"type":"started"}
{"job_id":12,"ts":1759600000001,"node_id":"start","state":"running","detail":"","message_key":null,"message_params":null,"type":"step"}
…
{"job_id":12,"experiment":"OSC ping → reply","outcome":"passed",…,"type":"ended"}
```

تُقرأ الأسطر إلى أن يأتي `ended`؛ ويُتجاهل `type` لا يُعرف. ويرسل الخادم `X-Accel-Buffering: no`، فيمرر وكيل
nginx كل سطر في الحال.

## الحالة والنتيجة {#status}

حالة خطأ HTTP تعني أنه **لم يبدأ أي تشغيل**؛ والمتن هو [`EngineError`](index.md#errors):

| الحالة | الرمز | السبب |
| --- | --- | --- |
| `400` | `api.run_invalid` | المتن ليس طلب تشغيل: ليس JSON، أو حقل مجهول، أو قيمة تجاوز ليست سلسلة أو رقمًا أو قيمة منطقية |
| `400` | `api.run_source` | لا `document` ولا `template`، أو كلاهما |
| `415` | `command.json_required` | ليس `Content-Type: application/json` |
| `422` | `api.template_unknown` | لا قالب مضمَّن بهذا الاسم |
| `422` | `file.json_invalid` و`file.too_large` و`doc.*` | تتعذر قراءة المستند |
| `422` | أي رمز تحقق، و`run.override_unknown` و`profile.active_missing` و`run.limit_range` و`seed.range` و`secret.missing` و`transport.address_in_use`… | تتعذر بداية التجربة: لا تجتاز التحقق، أو قيمة خارج النطاق، أو سر غير مخزن، أو منفذ تستمع عليه مأخوذ |

وما إن يبدأ التشغيل حتى تكون الحالة `200` مهما حدث: تُقرأ `outcome` في النتيجة.

| `outcome` | المعنى |
| --- | --- |
| `passed` | نجحت كل الخطوات وبُلغت [[ui:exp.node.end]] |
| `failed` | فشلت خطوة، أو استغرق التشغيل أكثر من `timeout` الخاصة به (`run.timeout`)؛ و`error` يقول أيهما ولماذا |
| `stopped` | أُوقف قبل أن ينتهي: بـ `job_stop`، أو [[ui:app.stopAll]]، أو بإغلاق الخادم. وفي `steps` الخطوات التي بلغها؛ ولا يُحفظ تقرير |

## عميل يغيب {#disconnect}

إغلاق الاتصال لا يوقف التشغيل. فهو مهمة على الخادم: يستمر إلى نهايته ويحفظ تقريره، كما يفعل تشغيل بدأ في متصفح
حين يُغلق التبويب. ويُعثر عليه بـ [`jobs_list`](commands.md#jobs_list)، ويُوقف بـ [`job_stop`](commands.md#job_stop)،
ويُقرأ تقريره بعد ذلك بـ [`experiment_runs`](commands.md#experiment_runs). وحين يُغلق الخادم يُوقف التشغيل
ويتلقى العميل الذي ما زال متصلًا `"outcome": "stopped"`.

## أمثلة {#examples}

تشغيل قالب مضمَّن وانتظار النتيجة:

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
curl -sS -X POST "$SERVER/api/run" \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"template":"osc-ping-reply","overrides":{"device":"192.0.2.20:9000"},"timeout":30}' \
  | jq -r .outcome
```

إرسال تجربتك أنت بمعامل مغيَّر، وطباعة كل خطوة كلما حدثت:

```bash
jq '{document: ., overrides: {api: "http://192.0.2.10:8080"}, profile: ""}' smoke.json |
  curl -sSN -X POST "$SERVER/api/run" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    -H "Accept: application/x-ndjson" --data @- |
  jq -r 'select(.type == "step") | "\(.node_id)  \(.state)  \(.detail)"'
```

`-N` يمنع `curl` من حبس الأسطر. ولإفشال خط عند تشغيل فاشل تُفحص `outcome`:

```bash
outcome=$(curl -sS -X POST "$SERVER/api/run" -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" -d '{"template":"http-check"}' | jq -r .outcome)
[ "$outcome" = "passed" ]
```

## من سطر الأوامر {#cli}

[`signallab run`](../automation/cli.md#cli-run) مع `--server <url>` يشغّل على خادم عبر نقطة النهاية هذه: يرسل
التجربة التي قرأها، مع `overrides` و`seed` و`timeout`، ويطلب NDJSON، ويطبع كل خطوة حين يصل سطرها. ويأتي الرمز
المميز من `--token-file`، وإلا `SIGNALLAB_TOKEN`. ومع `--report` ينزّل التقرير عبر `/api/files`. والخادم الذي لا
يرسل شيئًا 60 ثانية — ولا حتى نبضة — يُعدّ غائبًا.

---
title: الأوامر
description: كل أوامر محرك Signal Lab، تُستدعى بالصيغة POST /api/invoke/<command>، مع وسائطها ونتيجتها وأخطائها.
---

# الأوامر

كل أمر في المحرك، مجمّعًا بحسب ما يعمل عليه. يُستدعى كل منها بالصيغة `POST /api/invoke/<command>` مع كائن JSON
بالوسائط، ويجيب `200` بنتيجته أو `422` مع [`EngineError`](index.md#errors). وكيفية المصادقة ومعنى الحالات في
[نظرة عامة على الواجهة](index.md).

## الاصطلاحات {#conventions}

- **أسماء الوسائط** بنمط camelCase (`jobId` و`nodeId`). والوسيط الذي لا يعرفه الأمر، أو المطلوب المتروك، يُرفض
  بـ `command.args_invalid`؛ وكل أمر يقبل وسائط يمكن أن يفشل به. والأمر بلا وسائط لا يأخذ متنًا، أو `null`، أو `{}`؛ وأي شيء غير ذلك يُرفض بـ `command.args_invalid` أيضًا.
- **الكائنات الممررة وسيطًا** — `config` و`request` و`document` و`library` و`emulator` و`profile` — تستخدم
  أسماء حقول المحرك نفسها، وهي في الغالب بنمط snake_case (`timeout_ms`). وداخلها **يُتجاهل** الحقل الذي لا
  يعرفه المحرك، فالحقل الاختياري المكتوب خطأً يُبقي قيمته الافتراضية بصمت. ونموذج `feedback_send` وحده يرفض
  الحقول المجهولة.
- **الوسائط والحقول الاختيارية** يجوز تركها أو إرسالها `null`؛ وتذكر الجداول قيمها الافتراضية.
- **النتائج** بصيغة JSON. و«null» تعني أن الأمر لا شيء عنده ليعيده.
- **العناوين** المكتوبة `IP:port` تقبل عنوانًا رقميًّا ومنفذًا (`127.0.0.1:9000` و`[::1]:9000`)؛ واسم المضيف هناك
  مرفوض. وحيث يقول جدول `IP:port` أو `host:port` يصح اسم المضيف أيضًا: يُبحث عنه حين يعمل الأمر، ويُستخدم
  عنوان IPv4 الخاص به إن وُجد (فـ `localhost:9000` هو `127.0.0.1:9000`).
- **المهام**: الأمر المعلَّم *يبدأ مهمة* يعيد [`JobInfo`](#type-jobinfo)؛ ويستمر العمل إلى أن ينتهي أو يُوقَف بـ
  [`job_stop`](#job_stop). انظر [المهام](index.md#jobs).
- **المسارات** في النتائج على الجهاز الذي يعمل عليه المحرك — وعلى الخادم داخل مجلد بياناته؛ وتُنزَّل بـ
  [`/api/files`](index.md#files).

تستخدم الأمثلة دالة الصدفة هذه:

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
invoke() {
  curl -sS -X POST "$SERVER/api/invoke/$1" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    --data "${2:-}"
}
```

## التطبيق {#application}

### app_info {#app_info}

ما هو المحرك وأين يعمل. بلا وسائط.

**النتيجة**

| الحقل | النوع | المعنى |
| --- | --- | --- |
| `version` | سلسلة | إصدار Signal Lab، `[[version]]` |
| `mode` | سلسلة | `desktop` أو `server` |
| `secrets_writable` | قيمة منطقية | هل يمكن أن يعمل [`secret_set`](#secret_set) و[`secret_delete`](#secret_delete) هنا: `false` على الخادم |
| `data_dir` | سلسلة | مجلد البيانات، على الجهاز الذي يعمل عليه المحرك |
| `os` | سلسلة | `windows` أو `linux`… |
| `arch` | سلسلة | `x86_64` أو `aarch64`… |

### get_host_info {#get_host_info}

اسم الجهاز والعنوان الذي سيرسل منه. بلا وسائط.

**النتيجة**: `{ "local_ip": string, "hostname": string }`. `local_ip` هو عنوان IPv4 الذي يختاره النظام لحركة المرور
إلى الإنترنت (يُعثر عليه دون إرسال شيء)، أو `127.0.0.1` حين لا يوجد. و`hostname` هو اسم الكمبيوتر، أو `localhost` حين
لا يقول النظام.

### firewall_status {#firewall_status}

هل يتيح جدار حماية النظام للأجهزة الأخرى بلوغ هذا البرنامج. لا يوجد جدار حماية لكل برنامج يمكن قراءته إلا في
Windows؛ وفي غيره تكون `applies` هي `false`، ولا يُملأ من الباقي إلا `program`. بلا وسائط.

**النتيجة**

| الحقل | النوع | المعنى |
| --- | --- | --- |
| `applies` | قيمة منطقية | يوجد هنا جدار حماية لكل برنامج (Windows) |
| `program` | سلسلة | البرنامج الذي تخصه القواعد |
| `enabled` | قيمة منطقية | جدار الحماية مفعّل للشبكة التي عليها الجهاز الآن |
| `networks` | سلسلة[] | أنواع الشبكة التي عليها الجهاز: `domain` و`private` و`public` |
| `allowed` | قيمة منطقية | قاعدة واردة تتيح دخول UDP لهذا البرنامج على الشبكة الحالية |
| `blocked` | قيمة منطقية | قاعدة واردة تحجب هذا البرنامج على الشبكة الحالية؛ وتغلب أي قاعدة سماح |
| `rules` | رقم | القواعد الواردة لهذا البرنامج، من أي نوع |

**الأخطاء**: `firewall.failed`.

### firewall_allow {#firewall_allow}

يتيح للأجهزة الأخرى بلوغ Signal Lab: يعرض النظام مطالبة المسؤول الخاصة به، ثم تُستبدل القواعد الواردة للبرنامج
(بما فيها قاعدة الحجب) بقاعدة سماح واحدة لكل من Signal Lab وسطر الأوامر `signallab` المجاور له. للتطبيق
المكتبي على Windows فقط؛ والخادم يرفض، إذ لا أحد أمام شاشته ليجيب المطالبة.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `public` | قيمة منطقية | نعم | السماح على الشبكات العامة أيضًا، لا الخاصة وشبكات النطاق فقط |

**النتيجة**: [`firewall_status`](#firewall_status) الجديد.

**الأخطاء**: `firewall.server` (على خادم)، و`firewall.unsupported` (ليس Windows)، و`firewall.declined` (أُجيبت
المطالبة بـ لا)، و`firewall.failed`.

### feedback_send {#feedback_send}

يرسل رسالة إلى مطوري Signal Lab، عبر مركز الاستوديو الذي يرسلها إليهم بالبريد.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `form` | كائن | نعم | الرسالة، أدناه. والحقول المجهولة تُرفض |

| حقل `form` | النوع | الافتراضي | المعنى |
| --- | --- | --- | --- |
| `message` | سلسلة | — | ما حدث؛ مطلوب، 20 000 حرف على الأكثر |
| `email` | سلسلة | لا شيء | حيث يمكن أن يذهب جواب |
| `meta` | كائن من سلاسل | `{}` | ما يقوله التطبيق عن نفسه (version وos وarch وmode وlang وscreen) |
| `screenshots` | `{ name, data }[]` | `[]` | صور، و`data` بترميز base64؛ 6 على الأكثر، 8 MiB لكل منها |
| `logs` | `{ name, text }[]` | `[]` | ملفات نصية؛ 4 على الأكثر، 2 MiB لكل منها |

والمجموع كله 15 MiB على الأكثر.

**النتيجة**: `{ "id": string }`، المرجع الذي يصل إلى المطورين.

**الأخطاء**: `feedback.message_required` و`feedback.message_too_long` و`feedback.too_many_files`
و`feedback.file_too_large` و`feedback.too_large` و`feedback.invalid`، ورفوض المركز (`feedback.email_invalid`
و`feedback.file_type` و`feedback.rate_limited` و`feedback.disabled` و`feedback.send_failed` و`feedback.failed`)،
و`transport.*` الخاصة بالشبكة.

```bash
invoke app_info
# {"version":"[[version]]","mode":"server","secrets_writable":false,"data_dir":"/data","os":"linux","arch":"x86_64"}
```

## المهام {#jobs}

### jobs_list {#jobs_list}

المهام الجارية، الأقدم أولًا. بلا وسائط.

**النتيجة**: [`JobInfo`](#type-jobinfo)`[]`.

### job_stop {#job_stop}

يوقف مهمة واحدة في الحال: تُغلق مقابسها، ويذهب مرحّلها أو خادمها أو اتصالها. ولا يأتي الجواب إلا بعد أن تُغلق — فمن بدأ مهمة بعده مباشرة ربط المنفذ نفسه. والمهمة الموقوفة لا ترسل
[`job://ended`](events.md#event-job-ended)؛ والتشغيل الموقوف لا يحفظ تقريرًا.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `id` | رقم | نعم | `id` المهمة |

**النتيجة**: `true` حين كانت مهمة بهذا المعرّف تعمل، وإلا `false`.

### jobs_stop_all {#jobs_stop_all}

يوقف كل مهمة جارية، أيًّا كان من بدأها، ويجيب بعد أن تتخلى عن منافذها، كما يفعل [`job_stop`](#job_stop). والمهمة التي ما زالت تبدأ (تشغيل يفتح منافذه) تُوقَف عند بدئها. بلا وسائط.

**النتيجة**: null.

```bash
invoke jobs_list
# [{"id":3,"kind":"osc-monitor","label":"OSC monitor 0.0.0.0:9000","params":{"bind":"0.0.0.0:9000"},"started_ms":1759600000000}]
invoke job_stop '{"id":3}'
# true
```

## التجارب والتشغيلات {#experiments}

تأخذ هذه الأوامر مستند تجربة (`Experiment`) وتعيده: ملف JSON الذي يحفظه المحرر ويصدّره، مع `version` و`name` و`params`
و`profiles` و`profile` و`seed` و`cookies` و`nodes` و`edges`. وعقده في [العقد](../experiments/nodes.md)؛ ومعاملاته وملفات تعريفه
وقوالبه في [البيانات](../experiments/data.md). والمستند 4 MiB على الأكثر (`file.too_large`). ولتشغيل تجربة وانتظار نتيجتها
يُستخدم [`POST /api/run`](run.md) لا [`experiment_start`](#experiment_start).

### experiment_load {#experiment_load}

التجربة العاملة: `experiment.json` في مجلد البيانات — وعلى الخادم، التي تعرضها واجهته. وحين لا توجد تجربة، تجربة
البداية. وتُرحَّل إصدارات المستند الأقدم. بلا وسائط.

**النتيجة**: `Experiment`.

**الأخطاء**: `file.io` و`file.json_invalid` (مع `path` الملف و`line` و`column`) و`file.too_large`
و`doc.version_unsupported` وبقية فحوص `doc.*`.

### experiment_save {#experiment_save}

يستبدل التجربة العاملة، `experiment.json` في مجلد البيانات. ويُكتب أولًا في ملف مؤقت، فتترك الكتابة الفاشلة
السابق.

::: warning
على الخادم هذا هو المستند الذي يعمل عليه محرر كل متصفح.
:::

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `document` | `Experiment` | نعم | المستند |

**النتيجة**: سلسلة، المسار المكتوب.

**الأخطاء**: `doc.*`، وفحوص حجم المستند (`param.*` و`params.too_many` و`profile.*` و`profiles.too_many`
و`seed.range`)، و`file.too_large` و`file.io`.

### experiment_parse {#experiment_parse}

يقرأ تجربة من نص JSON، كما يفعل [[ui:exp.importJson]]. تُرحَّل الإصدارات من 1 إلى 8 إلى الإصدار 9، وهو الحالي؛
والملف من قبل الإصدار 8 يُفتح و`cookies` معطّلة، فيعمل كما كان. وتُتخطى علامة ترتيب البايتات.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `text` | سلسلة | نعم | نص الملف |

**النتيجة**: `Experiment`.

**الأخطاء**: `file.json_invalid` (`line` و`column`) و`file.too_large` و`doc.version_unsupported` و`doc.*`،
وفحوص الحجم في [`experiment_save`](#experiment_save).

### experiment_export {#experiment_export}

يكتب لقطة من مستند في `exports/experiment-<ms>-<16 hex digits>.json` ضمن مجلد البيانات. وكل تصدير ملف جديد.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `document` | `Experiment` | نعم | المستند |

**النتيجة**: سلسلة، المسار المكتوب.

**الأخطاء**: أخطاء [`experiment_save`](#experiment_save).

### experiment_validate {#experiment_validate}

يفحص أن المستند سيعمل بملف تعريفه النشط (أو بقيمه الافتراضية): المخطط وكل حقل والمعاملات، وأن كل سر يسمّيه
مخزن. والمشكلة المانعة هي الخطأ. وعند النجاح يقول أي ملفات التعريف *الأخرى* ستفشل، فيُعرف ذلك قبل التبديل.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `document` | `Experiment` | نعم | المستند |
| `overrides` | كائن من سلاسل | لا | قيم معاملات لهذا الفحص وحده، كما يعطيها [[ui:exp.runWith]] |

**النتيجة**: `{ "profile": string or null, "error": EngineError }[]` — كل ملف تعريف آخر لن يجتاز التحقق (`null`:
القيم الافتراضية، دون ملف تعريف). والقائمة الفارغة تعني أن كل ملف تعريف سليم.

**الأخطاء**: أي رمز تحقق (`doc.*` و`graph.*` و`node.*` و`param.*` و`profile.*` و`template.*` و`loop.*`…)،
و`run.override_unknown` (تجاوز لمعامل ليس في المستند)، و`secret.missing` و`secret.store` و`secret.unsupported`.

### experiment_resolve {#experiment_resolve}

عقدة واحدة بقوالبها معبأة، كما تعرضها معاينة المحرر: قيم ملف التعريف النشط وقيم المتغيرات التي تعطيها.
وتظهر الأسرار `••••`، لا قيمها أبدًا. والأسماء التي لا قيمة لها تبقى كما كُتبت وتُسرد.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `document` | `Experiment` | نعم | المستند |
| `nodeId` | سلسلة | نعم | العقدة |
| `vars` | كائن | نعم | قيم المتغيرات المستخدمة، بالاسم؛ و`{}` لعدمها |

**النتيجة**: `{ "node": node, "missing": string[] }`.

**الأخطاء**: `node.not_found` و`template.*` و`secret.store` و`secret.unsupported`.

### experiment_send_node {#experiment_send_node}

[[ui:exp.sendNow]]: ينفذ عقدة واحدة وحدها، عبر الشيفرة نفسها التي يستخدمها التشغيل. يُرسل الإجراء؛ وتستمع عقدة
الانتظار من الآن إلى أن تطابق أو تنتهي مهلتها. وعقدة [[ui:exp.node.ws_send]] أو [[ui:exp.node.wait_ws]] تفتح
الاتصال الذي تصفه عقدة [[ui:exp.node.ws_connect]] الخاصة بها. ولا يُرسل شيء بملفات تعريف الارتباط، ولا تُفتح عقد
[[ui:exp.node.impairment]] و[[ui:exp.node.emulator]] الخاصة بالتشغيل.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `document` | `Experiment` | نعم | المستند؛ ويعطي ملف تعريفه النشط قيم المعاملات |
| `nodeId` | سلسلة | نعم | إجراء أو عقدة انتظار |
| `vars` | كائن | نعم | قيم المتغيرات التي تقرؤها قوالب العقدة؛ و`{}` لعدمها |

**النتيجة**

| الحقل | النوع | المعنى |
| --- | --- | --- |
| `detail` | سلسلة | ما حدث، بالإنجليزية |
| `response` | [`HttpResponse`](#type-httpresponse) أو null | استجابة عقدة HTTP |
| `vars` | كائن | ما عيّنته الخطوة: رد عقدة انتظار، أو ما تأخذه عقد [[ui:exp.node.extract]] التالية لطلب من استجابته |

قيم الأسرار محجوبة في كل ذلك.

**الأخطاء**: `node.not_found` و`run.not_an_action` (ليست إجراءً ولا عقدة انتظار) و`ws.connection_unknown`
و`secret.missing` و`template.*`، وما تفشل به الخطوة: `transport.*` و`wait.timeout` و`check.*`…

### experiment_start {#experiment_start}

يبدأ تشغيلًا، كما يفعل [[ui:exp.run]]، ويعود في الحال. وتصل خطواته أحداث
[`experiment://step`](events.md#event-experiment-step)، ونهايته [`experiment://ended`](events.md#event-experiment-ended)،
ويُحفظ تقريره تحت `runs/` في مجلد البيانات. وتفتح عقد الانتظار والمحاكيات ومرحّلات الإضعاف واشتراكات MQTT قبل
الخطوة الأولى، فالمنفذ المأخوذ يفشل هنا. والتشغيل الأطول من 300 ثانية يفشل بـ `run.timeout`. *يبدأ مهمة*
(`experiment`).

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `document` | `Experiment` | نعم | المستند |
| `overrides` | كائن من سلاسل | لا | قيم معاملات لهذا التشغيل وحده |
| `seed` | رقم | لا | بذرة التشغيل، من 0 إلى 9007199254740991؛ والافتراضي: بذرة المستند، وإلا جديدة |

**النتيجة**: [`JobInfo`](#type-jobinfo)، و`params.name` اسم التجربة.

**الأخطاء**: كل ما يبلغ عنه [`experiment_validate`](#experiment_validate)، و`seed.range`
و`transport.address_in_use` وبقية إخفاقات الربط، و`emulator.*` و`impair.*`، و`node.params_only` (عنوان استماع أو
وسيط أو موضوع عقدة انتظار MQTT غير ثابت حين يبدأ التشغيل)، وأخطاء [`mqtt_connect`](#mqtt_connect) لوسيط
تتعذر على عقدة انتظار MQTT بلوغه.

### experiment_runs {#experiment_runs}

تشغيلات تُقرأ من تقاريرها في `runs/`، الأحدث أولًا. والتقرير الذي تتعذر قراءته يُترك.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `name` | سلسلة | لا | تشغيلات التجربة بهذا الاسم بالضبط فقط |
| `limit` | رقم | لا | هذا العدد على الأكثر؛ الافتراضي 50، والأقصى 500 |

**النتيجة**: ملخصات التشغيلات:

| الحقل | النوع | المعنى |
| --- | --- | --- |
| `name` | سلسلة | اسم ملف التقرير، `run-<ms>-<job>.json`: ما يأخذه [`experiment_compare`](#experiment_compare) |
| `experiment` | سلسلة | اسم التجربة |
| `started_ms`، `ended_ms` | رقم | مللي ثانية منذ 1970 |
| `outcome` | سلسلة | `passed` أو `failed` |
| `seed` | رقم | بذرة التشغيل |
| `profile` | سلسلة أو null | ملف تعريفه |
| `loads` | كائنات[] | كل خطوة حمل: `node` و`sent` و`rps` و`p95_ms` و`error_rate` و`held` (تحققت كل عتبة) |

**الأخطاء**: `file.io`.

### experiment_compare {#experiment_compare}

تشغيلان جنبًا إلى جنب، خطوة حمل بعد خطوة، كما يعرضهما [[ui:exp.compare]] في الخط الزمني. وتُطابَق الخطوات بمعرّف
العقدة.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `a` | سلسلة | نعم | اسم ملف تقرير التشغيل الأسبق |
| `b` | سلسلة | نعم | اسم ملف تقرير التشغيل الأحدث |

**النتيجة**: `{ "a": summary, "b": summary, "steps": [...] }`، ولكل خطوة `node` و`missing_in` (`a` أو `b`، حين لا
يملكها إلا تشغيل واحد) و`metrics` و`sent` (`[a, b]`) و`thresholds_a` و`thresholds_b` (كل عتبة بالشكل
`{ metric, op, value, actual, held }`). وتسرد `metrics` تسعة مقاييس، كل منها بالشكل
`{ metric, a, b, change, percent, worse }`: و`metric` هو `p50_ms` أو `p90_ms` أو `p95_ms` أو `p99_ms` أو `mean_ms` أو
`max_ms` أو `error_rate` أو `rps` أو `missed`؛ و`change` هو `b − a`؛ و`percent` التغيير بنسبة % من `a` (null حين تكون `a`
صفرًا)؛ و`worse` أنه تحرك في الاتجاه الخاطئ — أعلى، أو أدنى لـ `rps` — بنسبة 5 % أو أكثر، أو من 0 إلى أي شيء. والخطوة
التي لا يملكها إلا تشغيل واحد ليست `worse` أبدًا. انظر [الحمل](../experiments/load.md).

**الأخطاء**: `runs.name_invalid` (أي شيء غير اسم ملف تقرير، بلا مجلدات) و`runs.not_found` و`file.json_invalid`
و`file.io`.

```bash
invoke experiment_validate "$(jq '{document: .}' experiment.json)"
# []
```

## الأسرار {#secrets}

تستخدم التجارب قيم الأسرار بالصيغة `{{secret.NAME}}` ولا تغادر المحرك أبدًا: لا أمر يعيد واحدة. وموضع حفظها
يتوقف على مكان عمل المحرك:

| أين | المخزن | التعيين والإزالة |
| --- | --- | --- |
| التطبيق المكتبي، Windows | مدير بيانات الاعتماد في Windows | نعم |
| التطبيق المكتبي، Linux | لا شيء | `secret.unsupported` |
| الخادم | `SIGNALLAB_SECRET_<NAME>`، أو الملف `<NAME>` في `--secrets-dir` (الافتراضي `/run/secrets/signallab`) | لا: `secret.read_only` |

يبدأ الاسم بحرف لاتيني أو `_`، ويتبعه حروف لاتينية وأرقام و`_`، وطوله 128 حرفًا على الأكثر
(`secret.name_invalid`).

### secret_status {#secret_status}

أي الأسماء المعطاة لها قيمة مخزنة.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `names` | سلسلة[] | نعم | الأسماء المراد البحث عنها |

**النتيجة**: كائن، الاسم ← `true` (مخزن) أو `false`.

**الأخطاء**: `secret.name_invalid` (على خادم) و`secret.store` و`secret.too_large` (ملف خادم يزيد على 16 KiB)
و`secret.unsupported`.

### secret_set {#secret_set}

يخزّن قيمة تحت اسم، ويستبدل الموجودة هناك. للتطبيق المكتبي فقط.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `name` | سلسلة | نعم | الاسم |
| `value` | سلسلة | نعم | غير فارغة؛ 16 KiB على الأكثر |

**النتيجة**: null.

**الأخطاء**: `secret.read_only` (على خادم) و`secret.unsupported` و`secret.name_invalid` و`secret.empty`
و`secret.too_large` و`secret.store`.

### secret_delete {#secret_delete}

يزيل قيمة مخزنة. وإزالة ما ليس مخزنًا ليست خطأً. للتطبيق المكتبي فقط.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `name` | سلسلة | نعم | الاسم |

**النتيجة**: null.

**الأخطاء**: `secret.read_only` (على خادم) و`secret.unsupported` و`secret.name_invalid` و`secret.store`.

```bash
invoke secret_status '{"names":["API_TOKEN","MQTT_PASSWORD"]}'
# {"API_TOKEN":true,"MQTT_PASSWORD":false}
```

## OSC {#osc}

انظر [OSC](../protocols/osc.md) للشاشة التي تخدمها هذه الأوامر.

### osc_send {#osc_send}

يرسل رسالة OSC واحدة في مخطط بيانات UDP واحد، من مقبس جديد.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `target` | سلسلة | نعم | `IP:port` أو `host:port` للإرسال إليه؛ ويُبحث عن الاسم، ويؤخذ عنوان IPv4 إن وُجد |
| `address` | سلسلة | نعم | عنوان OSC، `/mixer/fader/1`؛ ويبدأ بـ `/` |
| `args` | [`OscArg`](#type-oscarg)`[]` | نعم | الوسائط؛ و`[]` لعدمها |

**النتيجة**: رقم، البايتات المرسلة.

**الأخطاء**: `node.osc_address` (لا `/` في البداية؛ الحقل `address`)، و`transport.target_invalid` (لا منفذ، أو لا
هذه الصيغة ولا تلك)، و`transport.dns` (الاسم لا يُحل)، و`transport.*`.

### osc_monitor_start {#osc_monitor_start}

يستمع إلى OSC على منفذ UDP ويفك ترميز كل حزمة. وتصل كل واحدة حدث [`osc://message`](events.md#event-osc-message).
*يبدأ مهمة* (`osc-monitor`، `params.bind`).

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `bind` | سلسلة | نعم | `IP:port` للاستماع عليه: `0.0.0.0:9000` كل بطاقات الشبكة، `127.0.0.1:9000` هذا الجهاز وحده |

**النتيجة**: [`JobInfo`](#type-jobinfo).

**الأخطاء**: `node.bind_invalid` و`transport.address_in_use` و`transport.address_unavailable` و`transport.denied`
و`wait.bind_failed`. وتنتهي المهمة بـ `wait.receive_failed` إذا لم يعد المقبس قادرًا على الاستقبال.

### osc_generator_start {#osc_generator_start}

يرسل تدفقًا من رسائل OSC يتبع وسيطها الوحيد شكلًا موجيًّا. ويصل التقدم [`osc://gen-tick`](events.md#event-osc-gen-tick).
*يبدأ مهمة* (`osc-gen`، `params.target`، `params.address`).

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `config` | كائن | نعم | أدناه |

| حقل `config` | النوع | الافتراضي | المعنى |
| --- | --- | --- | --- |
| `target` | سلسلة | — | `IP:port` أو `host:port` للإرسال إليه؛ ويُبحث عن الاسم مرة واحدة، حين تبدأ المهمة |
| `address` | سلسلة | — | عنوان OSC؛ ويبدأ بـ `/` |
| `rate` | رقم | — | رسائل في الثانية، تُقيَّد بين 0.1 و5000 |
| `waveform` | سلسلة | — | `sine` أو `triangle` أو `saw` (هابط: من `max` إلى `min`، ثم يعود في الحال) أو `ramp` (صاعد: من `min` إلى `max`، ثم يعود في الحال) أو `square` أو `random` أو `constant` (`max`) |
| `freq` | رقم | — | دورات الشكل الموجي في الثانية |
| `min`، `max` | رقم | — | نطاق القيمة |
| `as_int` | قيمة منطقية | `false` | التقريب وإرسال عدد صحيح بدل عشري |
| `duration_s` | رقم | `0` | التوقف بعد هذا العدد من الثواني؛ و0 يعمل إلى أن يُوقَف |

**النتيجة**: [`JobInfo`](#type-jobinfo).

**الأخطاء**: `node.osc_address` و`transport.target_invalid` و`transport.dns` و`transport.*`. وتنتهي المهمة
بخطأ `transport.*` إذا فشل إرسال.

```bash
invoke osc_send '{"target":"127.0.0.1:9000","address":"/cue/go","args":[{"type":"int","value":1}]}'
# 16
invoke osc_monitor_start '{"bind":"0.0.0.0:9000"}'
```

## HTTP وملفات تعريف الارتباط {#http}

انظر [HTTP](../protocols/http.md).

### http_request {#http_request}

يرسل طلب HTTP واحدًا ويعيد الاستجابة. والطلب الذي لا يلقى استجابة — مرفوض، أو انتهت مهلته، أو اسم لا يُحل،
أو شهادة غير موثوقة — **ليس** خطأً في الأمر: فالاستجابة تقول ذلك في `error` و`cause`.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `request` | [`HttpRequest`](#type-httprequest) | نعم | الطلب |
| `cookies` | قيمة منطقية | لا | إرسال مخزن ملفات تعريف الارتباط الخاص بشاشة [[ui:nav.http]] والاحتفاظ بما يعيّنه الجواب؛ الافتراضي `false` |

**النتيجة**: [`HttpResponse`](#type-httpresponse).

**الأخطاء**: `node.method_invalid` (الحقل `method`: ليس رمز HTTP، مثل `BAD METHOD`)، و`http.client_failed` (تعذر حتى تجهيز الطلب).

### http_burst_start {#http_burst_start}

يرسل طلبًا واحدًا مرات كثيرة، عدة منها معًا، ويقيسه. وبلا `rate` يرسل كل عامل من جديد ما إن يلقى جوابًا؛ ومع وجوده
تبدأ الطلبات وفق جدول ثابت مهما أبطأت الأجوبة، والطلب الذي انتظر أكثر من 50 مللي ثانية بعد لحظته عاملًا
حرًّا يُتخطى ويُحصى فائتًا. ويصل التقدم [`http://burst-progress`](events.md#event-http-burst-progress) عشر مرات في
الثانية. *يبدأ مهمة* (`http-burst`، `params.method`، `params.url`، و`params.rate` حين يكون على إيقاع).

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `config` | كائن | نعم | حقول [`HttpRequest`](#type-httprequest) وما يلي، في كائن واحد |

| حقل `config` | النوع | الافتراضي | المعنى |
| --- | --- | --- | --- |
| `concurrency` | رقم | — | هذا العدد على الأكثر قيد التنفيذ، يُقيَّد بين 1 و512 |
| `total` | رقم | `0` | التوقف بعد هذا العدد من الطلبات؛ 0: بلا عدد |
| `duration_s` | رقم | `0` | التوقف بعد هذا العدد من الثواني؛ 0: بلا حد زمني |
| `rate` | رقم | `0` | الطلبات المبدوءة في الثانية، من 0.1 إلى 100 000؛ 0: بأسرع ما تأتي الأجوبة |
| `cookies` | قيمة منطقية | `false` | استخدام مخزن ملفات تعريف الارتباط الخاص بشاشة [[ui:nav.http]] |

وبلا `total` ولا `duration_s` تعمل الدفعة إلى أن تُوقَف.

**النتيجة**: [`JobInfo`](#type-jobinfo).

**الأخطاء**: `http.rate_invalid` و`http.duration_invalid` و`node.method_invalid` و`http.client_failed`.

### http_cookies {#http_cookies}

مخزن ملفات تعريف الارتباط الخاص بشاشة [[ui:nav.http]]: كل ملف لم تنته صلاحيته. وعلى الخادم يوجد مخزن واحد لكل صفحة
وسكربت. بلا وسائط.

**النتيجة**: ملفات تعريف الارتباط، لكل منها `name` و`value` و`domain` و`host_only` (لا سمة Domain: فلا يعيده إلا
المضيف الذي عيّنه) و`path` و`expires` (ثوان Unix، وnull لملف جلسة) و`secure` و`http_only` و`same_site` (سلسلة أو
null).

### http_cookies_clear {#http_cookies_clear}

يفرّغ مخزن ملفات تعريف الارتباط الخاص بشاشة [[ui:nav.http]]. بلا وسائط.

**النتيجة**: null.

```bash
invoke http_request '{"request":{"method":"GET","url":"http://127.0.0.1:8080/health","headers":[["Accept","application/json"]],"body":null,"timeout_ms":5000}}' \
  | jq '{status, latency_ms, body}'
```

## WebSocket {#websocket}

انظر [WebSocket](../protocols/websocket.md). الاتصال الذي يفتحه `ws_connect` مهمة؛ وتسمّيه الأوامر الأخرى بـ `jobId`.

### ws_connect {#ws_connect}

يفتح WebSocket ويُبقيه مفتوحًا. وما يصل وما يُرسل يأتي [`ws://messages`](events.md#event-ws-messages) كل 100 مللي ثانية؛
وحالة الاتصال [`ws://state`](events.md#event-ws-state). *يبدأ مهمة* (`websocket`، `params.url`).

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | نعم | أين وكيف يُتصل |

**النتيجة**: [`JobInfo`](#type-jobinfo).

**الأخطاء**: `ws.url_invalid` و`ws.header_invalid` و`ws.protocol_invalid` و`ws.handshake_status` (أجاب الخادم
الترقية بحالة أخرى) و`ws.subprotocol_refused` و`ws.handshake_failed` و`transport.*`.

### ws_send {#ws_send}

يرسل رسالة واحدة على اتصال مفتوح.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `jobId` | رقم | نعم | مهمة الاتصال |
| `message` | كائن | نعم | `{ "text": "…" }` لرسالة نصية، أو `{ "hex": "de ad be ef" }` لرسالة ثنائية؛ واحد منهما بالضبط |

**النتيجة**: رقم، البايتات المرسلة.

**الأخطاء**: `ws.not_connected` و`ws.payload_required` (لا واحد منهما أو كلاهما) و`hex.invalid` (و`hex` فارغ أيضًا)
و`node.too_long` (أكثر من 16 MiB، الحقل `payload`؛ لا يُرسل شيء ويبقى الاتصال مفتوحًا) و`ws.closed` و`transport.*`
(`transport.timeout` حين توقف الخادم عن القراءة 10 ثوان).

### ws_close {#ws_close}

يغلق اتصالًا بمصافحة إغلاق وينتظر حتى ثانيتين جواب الخادم؛ ثم تنتهي المهمة. وبعد أن ينتهي اتصال تذهب مهمته،
وإغلاقه هو `ws.not_connected`.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `jobId` | رقم | نعم | مهمة الاتصال |
| `code` | رقم | لا | 1000، أو من 3000 إلى 4999 لما يخص التطبيق؛ الافتراضي 1000 |
| `reason` | سلسلة | لا | 123 بايتًا على الأكثر؛ الافتراضي فارغ |

**النتيجة**: `{ "code", "reason", "by", "error" }` — و`by` هو `client` أو `server` أو `lost`؛ و`code` هو 1005 حين لم يحمل
الإغلاق رمزًا و1006 حين لم يكن هناك إطار إغلاق.

**الأخطاء**: `ws.close_code` و`node.too_long` و`ws.not_connected`.

### ws_exchange {#ws_exchange}

تبادل واحد دون مهمة: يتصل، ويرسل رسالة إن أُعطيت، وينتظر جوابًا إن طُلب ذلك، ويغلق.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | نعم | أين وكيف يُتصل |
| `message` | كائن | لا | `{ "text" }` أو `{ "hex" }`، كما في [`ws_send`](#ws_send) |
| `expect` | كائن | لا | ما يُنتظر: `mode` (`any` أو `contains` أو `regex` أو `hex`؛ الافتراضي `any`) و`pattern` (الافتراضي فارغ) و`timeout_ms` (الافتراضي 2000) |

مع `expect` وبلا `message` تُحتسب أول رسالة مطابقة بعد الاتصال — تحية.

**النتيجة**: `{ "handshake", "sent", "reply", "closed" }` — و`handshake` هو `{ url, peer, local, protocol, ms }`؛ و`sent`
البايتات المرسلة أو null؛ و`reply` هو `{ kind, text, hex, bytes, json, ms }` أو null (`json`: الجواب النصي محلَّلًا، وإلا
null؛ و`ms`: منذ الإرسال، أو منذ الاتصال حين لم يُرسل شيء)؛ و`closed` كما يعيده [`ws_close`](#ws_close).

**الأخطاء**: أخطاء [`ws_connect`](#ws_connect) و[`ws_send`](#ws_send)، و`wait.timeout` (مع `ms` و`unmatched` و`target`)،
و`regex.invalid` و`hex.invalid` (`pattern` لا يُحلَّل).

```bash
invoke ws_exchange '{"config":{"url":"ws://127.0.0.1:9001/"},"message":{"text":"{\"type\":\"ping\"}"},"expect":{"mode":"contains","pattern":"pong"}}' \
  | jq .reply.text
```

## MQTT {#mqtt}

MQTT 3.1.1 عبر TCP عادي، وQoS 0 و1 و2. انظر [MQTT](../protocols/mqtt.md).

### mqtt_connect {#mqtt_connect}

يتصل بوسيط ويُبقي الاتصال. يعود الأمر حين يقبل الوسيط الاتصال (CONNACK)، فكلمة المرور الخاطئة أو المنفذ المغلق خطؤه.
وتصل الرسائل [`mqtt://messages`](events.md#event-mqtt-messages) كل 100 مللي ثانية؛ وتغيّرات الحالة
[`mqtt://state`](events.md#event-mqtt-state)؛ وعمليات النشر المكتملة بـ QoS 1/2 وإلغاءات الاشتراك
[`mqtt://ack`](events.md#event-mqtt-ack). *يبدأ مهمة* (`mqtt`، `params.broker`، `params.client`).

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | نعم | الوسيط وكيف يُتصل |

**النتيجة**: [`JobInfo`](#type-jobinfo).

**الأخطاء**: `mqtt.client_id_required` و`transport.*` (رُفض، أو يتعذر بلوغه، أو `dns`، أو `timeout` بعد 6 ثوان)
و`mqtt.no_answer` (لا CONNACK خلال 6 ثوان) و`mqtt.protocol` و`mqtt.refused_protocol` و`mqtt.refused_client_id`
و`mqtt.refused_unavailable` و`mqtt.refused_credentials` و`mqtt.refused_not_authorized` و`mqtt.refused`؛ وقبل الاتصال
`node.topic_filter` (مرشِّح `subscribe` تمنعه 3.1.1) و`node.too_long` (اسم مستخدم أو كلمة مرور أو معرّف عميل أو وصية
تتجاوز 65 535 بايتًا في MQTT) و`node.topic_wildcard` / `mqtt.topic_invalid` (وصية على موضوع لا يمكن النشر عليه). والاتصال
الذي لا يقول وسيطه شيئًا، ولا حتى ردًّا على الـ ping، طوال 1.5 × `keep_alive_s` ينتهي بـ `transport.timeout` في `job://ended`.

### mqtt_publish {#mqtt_publish}

ينشر على اتصال مفتوح.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `jobId` | رقم | نعم | مهمة الاتصال |
| `topic` | سلسلة | نعم | الموضوع: غير فارغ، ولا `+` ولا `#` |
| `payload` | سلسلة | نعم | الحمولة، تُرسل بترميز UTF-8 |
| `qos` | رقم | نعم | 0 أو 1 أو 2 (وما فوق 2 يُرسل 2) |
| `retain` | قيمة منطقية | نعم | يُطلب من الوسيط الاحتفاظ بها؛ والحمولة الفارغة مع `retain` تمسح قيمة محتفظًا بها |

**النتيجة**: null. ويُؤكَّد النشر بـ QoS 1 أو 2 لاحقًا بـ [`mqtt://ack`](events.md#event-mqtt-ack).

**الأخطاء**: `node.topic_wildcard` و`mqtt.topic_required` و`mqtt.topic_invalid` (فيه NUL) و`node.too_long` (يتجاوز
65 535 بايتًا)، كلها بالحقل `topic`، كما في [`mqtt_publish_once`](#mqtt_publish_once) — يرفضها الأمر قبل أن يبحث
عن الاتصال؛ و`mqtt.not_connected`.

### mqtt_subscribe {#mqtt_subscribe}

يشترك اتصال مفتوح في مرشِّحات. وما يمنحه الوسيط يصل [`mqtt://state`](events.md#event-mqtt-state) مع
`state: "subscribed"`.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `jobId` | رقم | نعم | مهمة الاتصال |
| `filters` | `{ filter, qos }[]` | نعم | واحد على الأقل؛ و`qos` افتراضيًّا 0. و`+` و`#` أحرف بدل |

**النتيجة**: null.

**الأخطاء**: `mqtt.filter_required` و`node.topic_filter` (الحقل `topic`: مرشِّح تمنعه 3.1.1، مثل `a/#/b` أو `a+` — فالوسطاء
يغلقون الاتصال بسببه) و`mqtt.not_connected`.

### mqtt_unsubscribe {#mqtt_unsubscribe}

يلغي اشتراك اتصال مفتوح من مرشِّحات.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `jobId` | رقم | نعم | مهمة الاتصال |
| `filters` | سلسلة[] | نعم | واحد على الأقل |

**النتيجة**: null. ويصل جواب الوسيط [`mqtt://ack`](events.md#event-mqtt-ack) مع `kind: "unsubscribed"`.

**الأخطاء**: `mqtt.filter_required` و`node.topic_filter` (الحقل `topic`) و`mqtt.not_connected`.

### mqtt_publish_once {#mqtt_publish_once}

يتصل، وينشر رسالة واحدة، وينتظر الإقرار الذي يطلبه QoS الخاص بها (حتى 6 ثوان)، ويقطع الاتصال. ويأتي باتصاله الخاص،
بمعرّف عميل خاص به — أول 12 حرفًا من `client_id` ثم `-o` ورقم — فلا يقطع أبدًا اتصالًا حيًّا بذلك المعرّف من الوسيط.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | نعم | الوسيط؛ و`subscribe` لا تُستخدم |
| `topic` | سلسلة | نعم | غير فارغ وبلا `+` أو `#` |
| `payload` | سلسلة | نعم | تُرسل بترميز UTF-8 |
| `qos` | رقم | نعم | 0 أو 1 أو 2 (وما فوق 2 يُرسل 2) |
| `retain` | قيمة منطقية | نعم | يُطلب من الوسيط الاحتفاظ بها |

**النتيجة**: سلسلة، ملخص يكتبه Signal Lab: `<topic> → <broker> · <bytes> B · qos<n>`، مع ` retained` حين تكون
محتفظًا بها.

**الأخطاء**: `mqtt.topic_required` و`node.topic_wildcard`، وأخطاء [`mqtt_connect`](#mqtt_connect) عدا
`mqtt.client_id_required`: فـ `client_id` الفارغ مقبول هنا.

```bash
invoke mqtt_publish_once '{"config":{"host":"127.0.0.1","port":1883,"client_id":"lab"},"topic":"lab/lamp/set","payload":"ON","qos":1,"retain":false}'
# "lab/lamp/set → 127.0.0.1:1883 · 2 B · qos1"
```

## البث والبث المتعدد والاكتشاف {#broadcast}

انظر [البث والاكتشاف](../protocols/broadcast.md).

::: danger
يبلغ البث والتمشيط كل مضيف في مقطع الشبكة. لا يُرسل إلا على شبكات أنت مسؤول عنها.
:::

### broadcast_send {#broadcast_send}

يرسل مخطط بيانات واحدًا إلى كل هدف، مرة واحدة.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `config` | كائن | نعم | أدناه |

| حقل `config` | النوع | الافتراضي | المعنى |
| --- | --- | --- | --- |
| `mode` | سلسلة | — | `list` أو `broadcast` أو `multicast` أو `sweep` |
| `target` | سلسلة | — | بحسب الوضع، أدناه |
| `port` | رقم | `0` | المنفذ، لـ `sweep` فقط |
| `payload` | [`Payload`](#type-payload) | — | ما يحمله كل مخطط بيانات |
| `bind` | سلسلة | أي | `IP:port` المحلي الذي يُرسل منه؛ فارغ أو null: `0.0.0.0:0` (و`[::]:0` حين تكون كل الأهداف IPv6) |
| `ttl` | رقم | `1` | IP TTL، أو حد قفزات البث المتعدد؛ من 1 إلى 255 |
| `multicast_loop` | قيمة منطقية | `true` | يعود البث المتعدد إلى هذا الجهاز أيضًا |
| `rate`، `count`، `duration_s` | رقم | `0` | لـ [`broadcast_beacon_start`](#broadcast_beacon_start) فقط |

| `mode` | `target` |
| --- | --- |
| `list` | مدخلات `IP:port` أو `host:port` تفصل بينها فواصل أو فواصل منقوطة أو أسطر جديدة (لا مسافات)؛ ويُبحث عن الاسم، ويؤخذ عنوان IPv4 إن وُجد |
| `broadcast` | `255.255.255.255:port`، أو عنوان ينتهي بـ `.255` مع منفذه |
| `multicast` | مجموعة من 224.0.0.0 إلى 239.255.255.255 مع منفذها |
| `sweep` | كتلة CIDR، `192.0.2.0/24`: كل مضيف صالح على `port`؛ و1024 مضيفًا على الأكثر، أي `/22` أو أضيق |

**النتيجة**

| الحقل | النوع | المعنى |
| --- | --- | --- |
| `targets` | رقم | الوجهات |
| `packets`، `bytes` | رقم | ما خرج |
| `errors` | رقم | مخططات البيانات التي تعذر إرسالها |
| `resolved` | سلسلة[] | أول 8 وجهات |
| `summary` | سلسلة | الحمولة في سطر واحد |
| `error` | `EngineError` | سبب فشل أول مخطط بيانات فاشل؛ ويُحذف حين لم يفشل شيء |

**الأخطاء**: `broadcast.target_required` و`broadcast.not_broadcast` و`broadcast.ipv6` و`broadcast.not_multicast`
و`broadcast.sweep_port` و`broadcast.cidr_invalid` و`broadcast.prefix_invalid` و`broadcast.sweep_too_large`
و`node.osc_address` (يجب أن يبدأ عنوان OSC بـ `/`) و`hex.empty` و`hex.invalid` و`node.bind_invalid`
و`socket.option_failed` و`transport.target_invalid` و`transport.dns`، وإخفاقات الربط.

### broadcast_beacon_start {#broadcast_beacon_start}

يرسل الجولة نفسها — مخطط بيانات لكل هدف — مرة بعد مرة. وتصل عدّاداته
[`broadcast://emit-stat`](events.md#event-broadcast-emit-stat) كل 250 مللي ثانية. وبعد أكثر من 32 إرسالًا فاشلًا
دون إرسال واحد ناجح يتوقف مع ذكر السبب. *يبدأ مهمة* (`beacon`، `params.mode`، `params.target`، `params.targets`،
`params.rate`).

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `config` | كائن | نعم | كما في [`broadcast_send`](#broadcast_send)، مع الثلاثة أدناه |

| حقل `config` | النوع | الافتراضي | المعنى |
| --- | --- | --- | --- |
| `rate` | رقم | — | جولات في الثانية؛ فوق 0، وحاصل ضرب الجولات في الأهداف 50 000 مخطط بيانات في الثانية على الأكثر |
| `count` | رقم | `0` | التوقف بعد هذا العدد من الجولات؛ 0: بلا عدد |
| `duration_s` | رقم | `0` | التوقف بعد هذا العدد من الثواني؛ 0: إلى أن يُوقَف |

**النتيجة**: [`JobInfo`](#type-jobinfo).

**الأخطاء**: أخطاء [`broadcast_send`](#broadcast_send)، و`broadcast.rate_invalid` و`broadcast.rate_limit`.

### discovery_start {#discovery_start}

يستمع على منفذ UDP، ويحتفظ بقائمة بكل نظير يرسل شيئًا، ويمكنه أن يجيب طلبات الاكتشاف كما يفعل جهاز. وتصل النظراء
[`broadcast://peers`](events.md#event-broadcast-peers) كل 400 مللي ثانية. *يبدأ مهمة* (`discovery`، `params.bind`،
`params.groups`، `params.joined`).

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `config` | كائن | نعم | أدناه |

| حقل `config` | النوع | الافتراضي | المعنى |
| --- | --- | --- | --- |
| `bind` | سلسلة | — | `IP:port` للاستماع عليه |
| `groups` | سلسلة[] | `[]` | مجموعات البث المتعدد المراد الانضمام إليها (IPv4) |
| `interface` | سلسلة | أي | عنوان IPv4 المحلي الذي يُنضَم عليه إلى المجموعات |
| `reuse` | قيمة منطقية | `true` | مشاركة المنفذ مع برنامج يستمع عليه أصلًا (`SO_REUSEADDR`) |
| `respond` | قيمة منطقية | `false` | إجابة ما يصل |
| `response` | [`Payload`](#type-payload) | لا شيء | الجواب؛ ويلزم مع `respond` |
| `respond_delay_ms` | رقم | `0` | الانتظار هذه المدة قبل الجواب |
| `match_contains` | سلسلة | لا شيء | إجابة مخططات البيانات التي يحوي نصها هذا فقط |

تُسرد 512 نظيرًا على الأكثر؛ ولا يُضاف اللاحقون.

**النتيجة**: [`JobInfo`](#type-jobinfo).

**الأخطاء**: `node.bind_invalid` و`broadcast.port_shared` (المنفذ مأخوذ و`reuse` معطل)
و`broadcast.interface_invalid` و`broadcast.not_multicast` و`broadcast.join_failed` و`broadcast.reply_missing`
و`node.osc_address` و`hex.*`، وإخفاقات الربط. وتنتهي المهمة بـ `wait.receive_failed` إذا لم يعد المقبس قادرًا على
الاستقبال.

```bash
invoke broadcast_send '{"config":{"mode":"list","target":"127.0.0.1:9000, 127.0.0.1:9001","payload":{"kind":"text","text":"PING"}}}' \
  | jq '{packets, errors}'
```

## الإضعاف {#impairment}

مرحّل بين عميل وخادمه يؤخر ما يمر عبره أو يُسقطه أو يزدوجه أو يُفسده أو يعيد ترتيبه أو يقيّده، عبر UDP أو TCP. انظر
[الإضعاف](../tools/impairment.md).

### netsim_start {#netsim_start}

يبدأ مرحّلًا: ما يصل على `listen` يمضي إلى `target`، وتعود الأجوبة بالطريق نفسه، وكلاهما مُضعَف بملف التعريف. وتصل
عدّاداته [`netsim://stat`](events.md#event-netsim-stat) كل 250 مللي ثانية. *يبدأ مهمة* (`netsim`، `params.listen`،
`params.target`، و`params.protocol` لـ TCP).

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `config` | كائن | نعم | أدناه |

| حقل `config` | النوع | الافتراضي | المعنى |
| --- | --- | --- | --- |
| `listen` | سلسلة | — | `IP:port` الذي يستمع عليه المرحّل؛ يُوجَّه العميل إليه |
| `target` | سلسلة | — | `IP:port` الخادم الحقيقي، أو `host:port` — ويُبحث عن اسم المضيف مرة واحدة، حين يبدأ المرحّل |
| `profile` | [`ImpairProfile`](#type-impairprofile) | — | ما يُفعل بحركة المرور |
| `seed` | رقم | جديدة | بذرة السحوبات: البذرة نفسها وحركة المرور نفسها تعطيان الإسقاطات نفسها |
| `protocol` | سلسلة | `udp` | `udp` (مخططات بيانات) أو `tcp` (تدفقات) |

**النتيجة**: [`JobInfo`](#type-jobinfo).

**الأخطاء**: `node.range` (قيمة في ملف التعريف خارج نطاقها، مع `min` و`max` والحقل) و`node.too_long`
و`node.bind_invalid` و`transport.target_invalid` و`transport.dns` (اسم هدف لا يُعثر عليه)، وإخفاقات الربط.
وتنتهي المهمة بـ `wait.receive_failed` إذا لم يعد مقبس قادرًا على الاستقبال.

### netsim_set_profile {#netsim_set_profile}

مرحّل جارٍ يُضعف بملف تعريف آخر من الآن، دون إغلاق مقابسه.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `jobId` | رقم | نعم | مهمة المرحّل |
| `profile` | [`ImpairProfile`](#type-impairprofile) | نعم | ملف التعريف الجديد |

**النتيجة**: null.

**الأخطاء**: `netsim.not_running` و`node.range` و`node.too_long`.

```bash
invoke netsim_start '{"config":{"listen":"127.0.0.1:9010","target":"127.0.0.1:9000","profile":{"latency_ms":80,"jitter_ms":20,"loss":0.02}}}'
```

## العاصفة والماسح {#storm-scanner}

::: danger
تحمّل العاصفة هدفًا بقدر ما تطلب، ويفحص التمشيط كل منفذ في نطاق. لا يُوجَّهان إلا إلى مضيفات أنت مسؤول عنها.
:::

### storm_start {#storm_start}

يرسل حملًا ثابتًا من مخططات بيانات UDP أو اتصالات TCP إلى هدف واحد. وتصل عدّاداته
[`storm://stat`](events.md#event-storm-stat) كل 250 مللي ثانية. *يبدأ مهمة* (`storm`، `params.protocol`،
`params.target`، `params.rate`).

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `config` | كائن | نعم | أدناه |

| حقل `config` | النوع | الافتراضي | المعنى |
| --- | --- | --- | --- |
| `target` | سلسلة | — | `IP:port` أو `host:port`؛ ويُبحث عن الاسم مرة واحدة، حين تبدأ المهمة |
| `protocol` | سلسلة | — | `udp`: مخططات بيانات؛ `tcp`: اتصال لكل وحدة يكتب الحمولة ويغلق (وقد يستغرق كل اتصال 500 مللي ثانية) |
| `size` | رقم | — | بايتات الحمولة، تُقيَّد بين 1 و65 507 |
| `rate` | رقم | — | وحدات في الثانية، تُقيَّد بحد 10 000 000، وفق جدول: الوحدة *n* مستحقة بعد *n* / `rate` ثانية من البدء، وكل استيقاظ يرسل ما استحق (256 على الأكثر؛ والجدول المتأخر أكثر يتخطى الوحدات الأقدم)؛ و0 يرسل بأسرع ما يمكن |
| `duration_s` | رقم | `0` | التوقف بعد هذا العدد من الثواني؛ و0 (أو عدد سالب) إلى أن يُوقَف |

**النتيجة**: [`JobInfo`](#type-jobinfo).

**الأخطاء**: `transport.target_invalid` و`transport.dns`، ولـ UDP إخفاق ربط أو اتصال (`transport.*`) حين يتعذر
فتح المقبس أصلًا. وعمليات الإرسال التي يرفضها الهدف تُحصى `errors` في الأحداث؛ وما عجز هذا الجهاز عن إرساله (نفاد
المنافذ أو المقابس أو المخازن) يُحصى `local`، ولا يُعدّ خطأ الهدف أبدًا.

### scan_start {#scan_start}

يجرب اتصال TCP بكل منفذ في نطاق ويبلغ عن المفتوحة، مع ما تقوله الخدمة أولًا حين يُطلب. وتصل المنافذ المفتوحة
[`scan://open`](events.md#event-scan-open)، والتقدم [`scan://progress`](events.md#event-scan-progress). *يبدأ مهمة*
(`scan`، `params.host`، `params.from`، `params.to`).

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `config` | كائن | نعم | أدناه |

| حقل `config` | النوع | الافتراضي | المعنى |
| --- | --- | --- | --- |
| `host` | سلسلة | — | اسم مضيف أو عنوان |
| `port_start`، `port_end` | رقم | — | النطاق، كلاهما مشمول؛ وإذا أُعطيا معكوسين تبادلا |
| `concurrency` | رقم | `256` | المحاولات في آن واحد، من 1 إلى 1024 |
| `timeout_ms` | رقم | `600` | لكل منفذ، من 50 إلى 10 000 |
| `grab_banner` | قيمة منطقية | `false` | قراءة ما يصل إلى 256 بايتًا ترسلها الخدمة خلال 400 مللي ثانية من الاتصال |

**النتيجة**: [`JobInfo`](#type-jobinfo).

**الأخطاء**: `scan.host_required`.

```bash
invoke scan_start '{"config":{"host":"127.0.0.1","port_start":8000,"port_end":9100,"grab_banner":true}}'
```

## الفاحص {#inspector}

يسجّل الفاحص ما ترسله الأدوات وتستقبله، إطارات، ما دام الالتقاط مفعّلًا. وعلى الخادم يوجد فاحص واحد لكل صفحة
وسكربت. انظر [الفاحص](../tools/inspector.md).

### inspect_set_enabled {#inspect_set_enabled}

يفعّل الالتقاط أو يعطّله. وما دام معطلًا لا يُسجَّل شيء.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `enabled` | قيمة منطقية | نعم | التفعيل (`true`) أو التعطيل |

**النتيجة**: [`CaptureStats`](#type-frame).

### inspect_stats {#inspect_stats}

عدّادات الالتقاط. بلا وسائط.

**النتيجة**: [`CaptureStats`](#type-frame).

### inspect_snapshot {#inspect_snapshot}

أحدث الإطارات، الأقدم أولًا.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `limit` | رقم | نعم | كم إطارًا، من 1 إلى 8192 |

**النتيجة**: [`Frame`](#type-frame)`[]`، دون بايتاتها (انظر [`inspect_payload`](#inspect_payload)).

### inspect_clear {#inspect_clear}

يفرّغ الالتقاط وعدّاداته. بلا وسائط.

**النتيجة**: [`CaptureStats`](#type-frame).

### inspect_export {#inspect_export}

يكتب كل إطار محتفظ به إلى `capture-<ms>.jsonl` أو `capture-<ms>.txt` في مجلد البيانات. وفي `jsonl` يكون كل سطر
إطارًا مع البايتات التي يحتفظ بها في `data`، بترميز base64؛ أما `txt` فللقراءة، مع مخرجات ست عشرية لكل إطار.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `format` | سلسلة | نعم | `txt`؛ وأي شيء آخر يكتب `jsonl` |

**النتيجة**: سلسلة، المسار المكتوب.

**الأخطاء**: `inspect.empty` و`file.io`.

### inspect_payload {#inspect_payload}

البايتات التي يحتفظ بها إطار، بعد المعاينة ذات الـ 256 بايتًا التي حملتها دفعته.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `seq` | رقم | نعم | رقم الإطار |

**النتيجة**: `{ "seq", "bytes", "kept", "dump", "hex" }` — و`bytes` حجم الإطار، و`kept` كم منها محتفظ به (حتى
256 KiB)، و`dump` كل صف بالشكل `offset  hex  |ascii|`، و`hex` الست عشري العادي الذي تُرسله إعادة الإرسال.

**الأخطاء**: `inspect.frame_gone` (أخذت إطارات أحدث مكانه) و`inspect.no_payload` (سُجّل حجمه فقط).

```bash
invoke inspect_set_enabled '{"enabled":true}'
invoke inspect_snapshot '{"limit":20}' | jq '.[] | {seq, proto, dir, summary}'
```

## مكتبة الإشارات {#signals}

المكتبة هي `signals.json` في مجلد البيانات. وهي تخزين فقط: فالإشارة تُرسل بأمر وسيلة نقلها (`osc_send` أو
`broadcast_send` أو `http_request` أو `mqtt_publish` أو `mqtt_publish_once`). انظر [الإشارات](../tools/signals.md)
و[الملفات](../reference/files.md#signals-json).

### signals_load {#signals_load}

يقرأ المكتبة. وحين لا يوجد الملف تُكتب مجموعة البداية أولًا. بلا وسائط.

**النتيجة**: `{ "path": string, "library": library, "seeded": boolean, "stamp": string }` — و`seeded` صحيحة حين كُتبت مجموعة البداية
للتو؛ و`stamp` يسمّي ما كان في الملف، ليكون `base` الحفظ التالي. والمكتبة هي `{ "version", "signals": [...], "folders": [...] }`: و`version` هي 2 (والملف من الإصدار 1 يعود كما هو)،
و`folders` تُحذف حين لا توجد. ولكل إشارة `id` و`name` و`group` (مجلدها، `"A/B"`؛ وفارغ لعدمه) و`note` و`body`.

**الأخطاء**: `signals.json_invalid` (مع `path` و`line` و`column`؛ ولا يُستبدل الملف أبدًا) و`file.io`.

### signals_save {#signals_save}

يستبدل ملف المكتبة كله، عبر ملف مؤقت في المجلد نفسه. والملف الموجود الذي لا يُقرأ مكتبةً يُترك كما هو. ومع `base`، فالملف الذي تغيّر منذ قراءته — من صفحة أخرى لخادم، أو من شخص في
محرر — يُترك كما هو أيضًا، ويُرفض الحفظ، فلا يضيع تغييره تحت قائمة لم ترَه.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `library` | كائن | نعم | `{ version, signals, folders }` كما يعيده `signals_load` |
| `base` | سلسلة | لا | الـ `stamp` الذي قُرئت به المكتبة (أو الذي أعاده آخر حفظ)؛ و`""` حين لم يكن ثمة ملف |

**النتيجة**: سلسلة، المسار المكتوب؛ ومع `base`، `{ "path", "stamp" }` — بصمة ما في الملف الآن، وهي `base` الحفظ التالي.

**الأخطاء**: `signals.json_invalid` (الملف الموجود الآن على القرص لا يُقرأ، مع `path` و`line` و`column`؛ ولا يُكتب
شيء) و`file.changed` (مع `path`: لم يعد الملف يحمل `base`؛ ولا يُكتب شيء — يُقرأ من جديد)
و`signals.encode` و`file.io`.

`body` الإشارة بحسب `transport`:

| `transport` | الحقول |
| --- | --- |
| `osc` | `target` و`address` و`args` ([`OscArg`](#type-oscarg)`[]`) |
| `udp` | `target` و`payload`: `{ "kind": "text", "text" }` أو `{ "kind": "hex", "hex" }` |
| `http` | `request` ([`HttpRequest`](#type-httprequest)) |
| `mqtt` | `broker` (`host:port`) و`topic` و`payload` و`qos` و`retain` |

```bash
invoke signals_load | jq '.library.signals[] | {name, transport: .body.transport}'
```

## المحاكيات {#emulators}

المحاكي هو Signal Lab يؤدي دور الطرف الآخر: واجهة HTTP API، أو جهاز OSC أو UDP أو TCP، أو وسيط MQTT. ومستنده —
`name` و`bind` و`protocol` وقواعد البروتوكول و`outage` اختياري — موصوف في [المحاكيات](../tools/emulators.md). والمكتبة هي
`emulators.json` في مجلد البيانات.

### emulators_load {#emulators_load}

يقرأ مكتبة المحاكيات. وحين لا يوجد الملف تُكتب مجموعة البداية أولًا. بلا وسائط.

**النتيجة**: `{ "path", "library": { "version": 1, "emulators": [{ "id", "note", "emulator" }] }, "seeded", "stamp" }` —
و`stamp` يسمّي ما كان في الملف، ليكون `base` الحفظ التالي.

**الأخطاء**: `emulators.json_invalid` (مع `path` و`line` و`column`؛ ولا يُستبدل أبدًا) و`file.io`.

### emulators_save {#emulators_save}

يستبدل مكتبة المحاكيات كلها، عبر ملف مؤقت. والملف الموجود الذي لا يُقرأ مكتبةً يُترك كما هو؛ ومع `base` كذلك الملف الذي
تغيّر منذ قراءته، كما في [`signals_save`](#signals_save).

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `library` | كائن | نعم | `{ version, emulators }` كما يعيده `emulators_load` |
| `base` | سلسلة | لا | الـ `stamp` الذي قُرئت به المكتبة (أو الذي أعاده آخر حفظ)؛ و`""` حين لم يكن ثمة ملف |

**النتيجة**: سلسلة، المسار المكتوب؛ ومع `base`، `{ "path", "stamp" }`.

**الأخطاء**: `emulators.json_invalid` (الملف الموجود الآن على القرص لا يُقرأ؛ ولا يُكتب شيء) و`file.changed` (لم يعد الملف يحمل
`base`؛ ولا يُكتب شيء) و`emulators.encode` و`file.io`.

### emulator_check {#emulator_check}

هل سيبدأ محاكٍ: كل ما يفحصه [`emulator_start`](#emulator_start) قبل أن يربط.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `emulator` | كائن | نعم | مستند المحاكي |
| `params` | كائن من سلاسل | لا | قيم تقرؤها قوالبه بوصفها معاملات |

**النتيجة**: null حين سيبدأ.

**الأخطاء**: `emulator.*`؛ و`node.*` لحقل مفقود أو خارج النطاق أو أطول من اللازم أو مشوَّه (`node.required`
و`node.range` و`node.too_long` و`node.bind_invalid` و`node.target_invalid` و`node.method_invalid`…)؛ و`param.unknown`
و`template.*` و`osc.pattern_*` و`regex.invalid` و`hex.invalid`. ولكل منها `rule` أو `retained` أو `response` في
`params` حين تكون المشكلة في واحدة منها.

### emulator_start {#emulator_start}

يبدأ محاكيًا مهمةً قائمة بذاتها. ويكون مقبسه مفتوحًا حين يعود الأمر. وما يتلقاه ويجيب به يصل
[`emulator://activity`](events.md#event-emulator-activity) كل 200 مللي ثانية حين يتغير شيء. *يبدأ مهمة*
(`emulator`، `params.name`، `params.protocol`، `params.local`، و`params.source` حين تُعطى `source`).

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `emulator` | كائن | نعم | مستند المحاكي |
| `params` | كائن من سلاسل | لا | قيم تقرؤها قوالبه بوصفها معاملات |
| `seed` | رقم | لا | بذرته، من 0 إلى 9007199254740991؛ والافتراضي: جديدة |
| `source` | سلسلة | لا | مدخل المكتبة الذي أتى منه، يُحتفظ به على المهمة بوصفه `params.source` |

**النتيجة**: [`JobInfo`](#type-jobinfo).

**الأخطاء**: أخطاء [`emulator_check`](#emulator_check)، و`seed.range` و`transport.address_in_use` وبقية إخفاقات
الربط.

### emulator_exchanges {#emulator_exchanges}

ما تلقاه محاكٍ جارٍ وأجاب به. يحتفظ بآخر 500 تبادل.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `jobId` | رقم | نعم | مهمة المحاكي |
| `after` | رقم | لا | التبادلات المرقمة فوق هذا فقط؛ الافتراضي 0 |
| `limit` | رقم | لا | هذا العدد على الأكثر، من 1 إلى 500؛ الافتراضي 500 |

**النتيجة**

| الحقل | النوع | المعنى |
| --- | --- | --- |
| `job_id` | رقم | المهمة |
| `name`، `protocol`، `local` | سلسلة | المحاكي وبروتوكوله والعنوان الذي يستمع عليه |
| `counts` | كائن | `total` و`unmatched` (لم تأخذه قاعدة) و`failed` و`down` (وصل وهو معطل: بانقطاعه أو [`emulator_down`](#emulator_down)) و`hits` (لكل قاعدة) و`missed` (MQTT: رسائل لم يتلقها عميل متأخر جدًّا؛ وتُحذف ما دامت 0) و`refused` (HTTP وTCP وMQTT: اتصالات أُغلقت عند وصولها، فوق الحد؛ وتُحذف ما دامت 0) |
| `forced` | سلسلة | `unavailable` أو `reset` أو `timeout` ما دام معطلًا؛ ويُحذف وإلا |
| `exchanges` | كائنات[] | لكل منها: `seq` و`ts` و`from` و`request` و`rule` (يبدأ من 1؛ ويُحذف حين لم تأخذه قاعدة) و`reply` و`status` و`fault` و`ms` و`error` و`frame` و`down` و`data` (الطلب كما تقرؤه القوالب) |

**الأخطاء**: `emulator.not_running`.

### emulator_down {#emulator_down}

يعطّل محاكيًا جاريًا إلى أن يُعاد، مهما قال جدول انقطاعه، أو يعيده إلى العمل. وما دام معطلًا يقابل محاكي HTTP كل
طلب بـ `fault`، ويقطع جهاز TCP ووسيط MQTT اتصالاتهما ويرفضان الجديدة، ولا تجيب أجهزة OSC وUDP بشيء.

| الوسيط | النوع | مطلوب | المعنى |
| --- | --- | --- | --- |
| `jobId` | رقم | نعم | مهمة المحاكي |
| `down` | قيمة منطقية | نعم | معطل (`true`) أو يعمل |
| `fault` | سلسلة | لا | ما تقابله طلبات HTTP: `unavailable` (503، دون `Retry-After`: فموعد عودته غير معروف) أو `reset` (يُغلق الاتصال) أو `timeout` (لا جواب)؛ الافتراضي `unavailable` |

**النتيجة**: null.

**الأخطاء**: `emulator.not_running`.

```bash
invoke emulator_exchanges '{"jobId":5,"after":0}' | jq '.counts, (.exchanges[] | {request, rule, status})'
```

## الأنواع المشتركة {#types}

### JobInfo {#type-jobinfo}

ما يعيده أمر يبدأ مهمة، وما يسرده [`jobs_list`](#jobs_list): `id` و`kind` و`label` (بالإنجليزية، للسجلات) و`params`
(القيم التي تسمّيها التسمية؛ وتُحذف حين لا توجد) و`started_ms`. انظر [المهام](index.md#jobs).

### OscArg {#type-oscarg}

وسيط OSC واحد، نوعه وقيمته:

| `type` | `value` | وسم OSC |
| --- | --- | --- |
| `int` | عدد صحيح 32 بت | `i` |
| `float` | رقم، يُرسل عشريًّا 32 بت | `f` |
| `str` | سلسلة | `s` |
| `long` | عدد صحيح 64 بت | `h` |
| `double` | رقم، 64 بت | `d` |
| `bool` | `true` أو `false` | `T` أو `F` |
| `blob` | مصفوفة بايتات، `[222, 173]` | `b` |
| `nil` | لا شيء: `{ "type": "nil" }` | `N` |
| `time` | وسم وقت، بتّاته الـ 64 نصًا عشريًّا: `"1"` تعني «فورًا» | `t` |
| `char` | محرف واحد، `"A"` | `c` |
| `color` | `[red, green, blue, alpha]` | `r` |
| `midi` | `[port, status, data1, data2]` | `m` |
| `impulse` | لا شيء | `I` |
| `array` | مصفوفة من OscArg | `[` … `]` |

الستة الأخيرة هي ما تحمله الرسائل المستلمة (`osc://message` الخاص بمراقب OSC)؛ ويمكن لأمر أن يرسلها أيضًا.

### HttpRequest {#type-httprequest}

| الحقل | النوع | الافتراضي | المعنى |
| --- | --- | --- | --- |
| `method` | سلسلة | — | `GET` و`POST`… |
| `url` | سلسلة | — | `http://` أو `https://` |
| `headers` | `[name, value][]` | `[]` | ترويسات الطلب |
| `body` | سلسلة أو null | null | المتن |
| `timeout_ms` | رقم | `10000` | للتبادل كله، بما فيه المتن: من 1 إلى 120 000؛ وخارجه يفشل الأمر بـ `node.range` (الحقل `timeout_ms`) |
| `auth` | كائن | لا شيء | `{ "scheme": "basic", "username", "password" }` أو `{ "scheme": "digest", "username", "password" }` أو `{ "scheme": "bearer", "token" }` |

تُتبع حتى 10 إعادات توجيه. وبيانات الاعتماد وملفات تعريف الارتباط المكتوبة لمضيف لا تنتقل أبدًا إلى آخر. وطلب
Digest يجيب تحدّي 401 الصادر من الخادم ويرسل من جديد.

### HttpResponse {#type-httpresponse}

| الحقل | النوع | المعنى |
| --- | --- | --- |
| `ok` | قيمة منطقية | حالة 2xx، ووصل المتن كله |
| `status`، `status_text` | رقم، سلسلة | الحالة؛ و0 وفارغة دون استجابة |
| `latency_ms` | رقم | إلى أن وصل المتن كله |
| `headers` | `[name, value][]` | ترويسات الاستجابة |
| `body` | سلسلة | المتن نصًّا، 256 KiB على الأكثر |
| `body_bytes` | رقم | حجم المتن الكامل |
| `truncated` | قيمة منطقية | قُطع `body` عند 256 KiB |
| `error` | سلسلة أو null | لماذا لم تكن هناك استجابة، أو لماذا توقف متنها قصيرًا (وحينئذ تكون `status` و`headers` والمتن حتى الآن موجودة أيضًا)، بكل طبقات السبب |
| `cause` | سلسلة أو null | نوع الإخفاق: `refused` أو `timeout` أو `dns` أو `unreachable` أو `reset` أو `address_in_use` أو `address_unavailable` أو `denied` أو `tls` أو `target_invalid` أو `exhausted` (نفدت على هذا الجهاز المقابس أو المنافذ المحلية) أو `failed` — وهي نفسها رموز `transport.*` |
| `digest` | كائن | لطلب Digest الذي لقي 401 فقط؛ ويُحذف وإلا. `challenged`: أُجيب التحدي وأُرسل الطلب من جديد. `error`: لماذا تعذر ذلك، وهو `EngineError` (`http.digest_not_offered` أو `http.digest_unsupported` أو `http.digest_invalid` أو `http.digest_other_origin`) أو null |

### Payload {#type-payload}

ما يحمله مخطط بيانات البث أو الاكتشاف: `{ "kind": "osc", "address", "args" }`، أو `{ "kind": "text", "text" }` (يُرسل
كما هو، بلا صفر ختامي)، أو `{ "kind": "hex", "hex" }` (`de ad be ef` أو `deadbeef` أو `0xDE,0xAD` — ويُتجاهل كل ما عدا
الأرقام الست عشرية).

### MqttConfig {#type-mqttconfig}

| الحقل | النوع | الافتراضي | المعنى |
| --- | --- | --- | --- |
| `host` | سلسلة | — | اسم الوسيط أو عنوانه |
| `port` | رقم | — | عادةً 1883 |
| `client_id` | سلسلة | — | غير فارغ؛ والاتصال الآخر بالمعرّف نفسه يقطعه الوسيط |
| `username`، `password` | سلسلة | فارغ | يُرسل `username` حين لا يكون فارغًا؛ و`password` مع `username` فقط |
| `keep_alive_s` | رقم | `60` | تذهب نبضات التحقق عند نصفها؛ 0: لا شيء |
| `clean_session` | قيمة منطقية | `true` | علم CONNECT |
| `will` | كائن أو null | null | `{ topic, payload, qos, retain }`، ينشره الوسيط إذا فُقد الاتصال |
| `subscribe` | `{ filter, qos }[]` | `[]` | يُشترك فيها ما إن يقوم الاتصال |

### WsConfig {#type-wsconfig}

| الحقل | النوع | الافتراضي | المعنى |
| --- | --- | --- | --- |
| `url` | سلسلة | — | `ws://` أو `wss://` (يثق `wss://` بما يثق به النظام لـ HTTPS) |
| `headers` | `[name, value][]` | `[]` | تُرسل مع طلب الترقية |
| `protocols` | سلسلة[] | `[]` | البروتوكولات الفرعية المعروضة، بترتيب التفضيل |
| `timeout_ms` | رقم | `10000` | للاتصال وTLS والترقية معًا |

الرسائل 16 MiB على الأكثر في الاتجاهين.

### ImpairProfile {#type-impairprofile}

كل حقل اختياري؛ وما يُحذف لا يفعل شيئًا. والاحتمالات من 0 إلى 1.

| الحقل | النطاق | المعنى | UDP | TCP |
| --- | --- | --- | --- | --- |
| `name` | 60 حرفًا على الأكثر | تسمية للخط الزمني والتقرير | نعم | نعم |
| `latency_ms` | من 0 إلى 60 000 | تأخير يُضاف إلى كل شيء | نعم | نعم |
| `jitter_ms` | من 0 إلى 60 000 | زيادة تصل إلى هذا المقدار، تُسحب في كل مرة | نعم | نعم |
| `loss` | من 0 إلى 1 | يُسقط مخطط بيانات | نعم | — |
| `duplicate` | من 0 إلى 1 | يُرسل مخطط بيانات مرتين | نعم | — |
| `corrupt` | من 0 إلى 1 | يُقلب بت واحد من مخطط بيانات | نعم | — |
| `reorder` | من 0 إلى 1 | يُحجز مخطط بيانات فتتجاوزه اللاحقة | نعم | — |
| `rate_kbps` | 0، أو من 8 إلى 10 000 000 | حد عرض النطاق، كيلوبت في الثانية؛ 0: لا حد | نعم | نعم |
| `burst_start` | من 0 إلى 1 | يبدأ مخطط بيانات دفعة فقد | نعم | — |
| `burst_length` | من 1 إلى 1000 | مخططات البيانات التي تدوم الدفعة في المتوسط (يلزم مع `burst_start`) | نعم | — |
| `offline` | `true` أو `false` | لا شيء يمر | نعم | نعم |
| `reset` | من 0 إلى 1 | جزء من تدفق يعيد تعيين اتصاله | — | نعم |
| `stall` | من 0 إلى 1 | جزء من تدفق يترك اتصاله نصف مفتوح | — | نعم |

### Frame وCaptureStats {#type-frame}

`Frame` حزمة أو طلب أو رسالة ملتقطة واحدة:

| الحقل | المعنى |
| --- | --- |
| `seq` | رقمه، تصاعدي |
| `ts` | متى، مللي ثانية منذ 1970 |
| `proto` | `osc` أو `udp` أو `tcp` أو `http` أو `mqtt` أو `ws`… |
| `dir` | `tx` (مُرسَل) أو `rx` (مستلَم) |
| `source` | الأداة التي التقطته: `osc-monitor` أو `broadcast` أو `netsim`… |
| `job_id` | مهمته، أو null |
| `local`، `remote` | العنوانان: `IP:port` لهذا الطرف وللطرف الآخر (عنوان URL أو وسيط لـ HTTP وWebSocket وMQTT). و`local` للإطار المرحَّل هو العنوان الذي يستمع عليه المرحّل، و`remote` وجهة الإطار؛ ويختم المقطع `verdict` الخاص به (`· client→target` و`· target→client`) |
| `bytes` | حجمه |
| `summary` | سطر واحد |
| `detail` | فك ترميز على عدة أسطر، أو null |
| `hex` | مخرجات ست عشرية لأول 256 بايتًا، أو null |
| `verdict` | ما آل إليه — `dropped` أو `sampled` أو حالة — أو null |
| `kept` | من `bytes`، كم منها محتفظ به (حتى 256 KiB)؛ و0 حين سُجّل الحجم فقط |
| `publish` | لنشر MQTT فقط: `{ broker, topic, qos, retain, text }` — الوسيط بالشكل `host:port`، وهل البايتات المحتفظ بها، وهي حمولة الرسالة، نص UTF-8. وتغيب عن كل إطار آخر |

`CaptureStats`: `enabled` و`total` (الإطارات المسجلة) و`bytes` و`skipped` (سُجّلت لكنها لم تُرسل إلى الواجهة) و`buffered`
(الإطارات المحتفظ بها) و`capacity` (8192) و`held` (بايتات الحمولة المحتفظ بها) و`held_limit` (64 MiB). وتفسح أقدم الإطارات
المكان بعد أي من الحدين.

---
title: سطر الأوامر
description: يشغّل signallab التجارب دون نافذة، ويرسل رسائل منفردة، ويؤدي دور المحاكيات، ويفحص الشبكة، من الطرفية أو سكربت أو مهمة CI.
---

# سطر الأوامر: `signallab`

`signallab` هو Signal Lab دون نافذة. يشغّل التجارب إلى نهايتها ويخرج برمز يفهمه السكربت، ويرسل رسالة OSC
واحدة أو مخطط بيانات أو طلب HTTP أو رسالة WebSocket أو نشر MQTT، ويُرسل إشارة من مكتبتك، ويؤدي دور محاكٍ
إلى أن يُوقَف، ويبيّن ما يقف بين هذا الجهاز والمعدات.

المحرك هو محرك التطبيق نفسه: فالتشغيل من سطر الأوامر ينفذ الخطوات نفسها ويكتب التقرير نفسه ويقول الأشياء
نفسها، بلغات الواجهة. ومع `--server` تجري التشغيلات على [خادم Signal Lab](../server/index.md) بدلًا من ذلك،
بشبكته وأسراره.

```bash
signallab run tests/smoke.json --param api=http://127.0.0.1:8080 --junit junit.xml
signallab run flaky-api                                # a bundled template, by name
signallab validate tests/*.json                        # the editor's check, nothing sent
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab fire "Fader value"
signallab emulate tests/orders-api.json --for 120      # play a dependency for two minutes
signallab doctor                                       # firewall, network, data folder
```

وللمسارات (خطوط CI) انظر [Signal Lab في CI](ci.md)؛ ولمساعد ذكاء اصطناعي،
[`signallab mcp`](mcp.md).

## التثبيت {#install}

| أين | كيف يُحصل على `signallab` |
| --- | --- |
| Windows، برنامج الإعداد (`.exe`) | يُثبَّت بجانب التطبيق، ويُضاف ذلك المجلد إلى `PATH` — الخاص بالمستخدم عند التثبيت «لي وحدي»، وللجهاز عند التثبيت «للجميع». تُفتح طرفية جديدة بعد التثبيت. ومفتاح `/NOPATH` في برنامج الإعداد يترك `PATH` كما هو. |
| Windows، ملف `.msi` | يُثبَّت بجانب التطبيق؛ ومجلد التثبيت على `PATH` الجهاز ما دام التطبيق مثبتًا. |
| Linux، `.deb` و`.rpm` | `/usr/bin/signallab`. |
| Linux، AppImage | غير مضمَّن: يُستخدم الأرشيف أدناه. |
| أي جهاز، دون التطبيق | يحمل كل إصدار `signallab-<version>-windows-x64.zip` و`signallab-<version>-linux-x64.tar.gz`، في كل منهما البرنامج ورخصته؛ ويسرد `SHA256SUMS.txt` في الصفحة نفسها مجاميعهما الاختبارية. |
| صورة الخادم | `/usr/local/bin/signallab` في `ghcr.io/proanima/signallab` (انظر [CI](ci.md#docker)). |

للتحقق منه:

```bash
signallab version
```

## الأوامر {#commands}

| الأمر | ما يفعله |
| --- | --- |
| [`run`](#cli-run) | يشغّل التجارب واحدة بعد أخرى؛ ويخرج بـ 0 فقط حين تنجح كلها. |
| [`validate`](#cli-validate) | يفحص التجارب كما يفعل المحرر قبل التشغيل؛ ولا يرسل شيئًا. |
| [`send`](#cli-send) | يرسل رسالة واحدة: `osc` أو `udp` أو `http` أو `ws` أو `mqtt`. |
| [`fire`](#cli-fire) | يرسل إشارة من مكتبة إشارات، بمعرّفها أو اسمها. |
| [`emulate`](#cli-emulate) | يؤدي دور واجهة HTTP API أو جهاز OSC أو UDP أو TCP أو وسيط MQTT إلى أن يُضغط <kbd>Ctrl</kbd>+<kbd>C</kbd> أو تنقضي `--for`. |
| [`emulators`](#cli-emulators) | يسرد محاكيات مكتبة التطبيق. |
| [`templates`](#cli-templates) | يسرد قوالب التجارب المضمَّنة. |
| [`nodes`](#cli-nodes) | يصف كل نوع من العقد، بصيغة JSON. |
| [`mcp`](#cli-mcp) | يخدم Signal Lab مساعد ذكاء اصطناعي عبر Model Context Protocol. |
| [`doctor`](#cli-doctor) | يفحص جدار الحماية والشبكة ومجلد البيانات وخادمًا. |
| [`firewall`](#cli-firewall) | `firewall allow`: يتيح للأجهزة الأخرى الوصول إلى Signal Lab عبر جدار حماية Windows. |
| [`version`](#cli-version) | يطبع الإصدار. |

`signallab help <command>` أو `signallab <command> --help` يطبع خيارات الأمر.

## خيارات كل أمر {#global-options}

| الخيار | ما يفعله | الافتراضي |
| --- | --- | --- |
| `--lang <code>` | لغة الرسائل: `en` أو `ru` أو `es` أو `fr` أو `de` أو `pt` أو `zh` أو `ja` أو `ko` أو `hi` أو `ar`. | `SIGNALLAB_LANG`، وإلا لغة النظام، وإلا `en` |
| `--json` | مخرجات للآلات على stdout بدل النص (انظر [المخرجات](#output)). | معطل |
| `-h`، `--help` | مساعدة الأمر. | |
| `-V`، `--version` | الإصدار؛ قبل أي أمر، مثل `signallab --version`. | |

يجوز أن يأتي `--lang` و`--json` في أي موضع من السطر:
`signallab --json run smoke.json` و`signallab run smoke.json --json` سواء.

### اللغة {#language}

تستخدم الرسائل ونصوص الخطوات والإخفاقات وتقرير JUnit نصوص الواجهة نفسها وصيغ الجمع وأسلوب الأرقام. واللغة
هي أول ما يلي:

1. `--lang`؛
2. `SIGNALLAB_LANG` (`ru` و`ru-RU` و`ru_RU.UTF-8` كلها تعني الروسية)؛
3. لغة النظام: أول ما هو معيَّن من `LC_ALL` و`LC_MESSAGES` و`LANG`؛
4. الإنجليزية.

::: tip
لا تعيّن طرفيات Windows في الغالب أيًّا من متغيرات اللغة، فيتحدث `signallab` الإنجليزية هناك ما لم يُعيَّن
`SIGNALLAB_LANG` أو يُمرَّر `--lang`.
:::

### مخرجات للبشر ومخرجات للآلات {#output}

بلا `--json` تذهب النتائج إلى **stdout** ويذهب كل ما يقرؤه شخص أثناء ذلك إلى **stderr**: خطوات التشغيل
كلما حدثت، وسبب الإخفاق، وموضع التقرير. و`signallab run … 2>/dev/null` يترك سطر حكم واحدًا لكل تشغيل.

ومع `--json` يحمل stdout صيغة JSON ولا شيء غيرها، ويبقى stderr صامتًا:

| الأمر | ما يطبعه `--json` على stdout |
| --- | --- |
| `run` | كائن في كل سطر: `started`، وكائن `step` لكل خطوة، و`ended` (نتيجة التشغيل)، ثم `summary`؛ و`error` لتشغيل تعذر بدؤه. انظر [ما يطبعه `run`](#run-output). |
| `validate` | كائن في كل سطر، لكل تجربة: `valid`، و`profile_issues` أو `error`. |
| `send`، `fire` | `{"type": "sent", "result": …}`؛ و`send http` يطبع `{"type": "response", "response": …}`، و`send ws` `{"type": "exchange", "result": …}`. |
| `emulate` | كائن في كل سطر: `started`، وكائن `exchange` لكل طلب، و`summary` لكل محاكٍ؛ و`valid` مع `--check`. |
| `emulators` | `{"path": …, "emulators": [{id, name, protocol, bind, rules, note}, …]}`. |
| `templates` | `[{"name": …, "experiment": …}, …]`. |
| `doctor` | كائن واحد: `version` و`network` و`data_dir` و`firewall` و`server` و`problems`. |
| `version` | `{"version": "…"}`. |
| `nodes` | JSON دائمًا، مع `--json` أو دونه. |

الإخفاق هو `{"type": "error", "error": {…}, "exit_code": N}`. و`error` هو خطأ المحرك: `code` ثابت (مثل
`transport.refused` أو `secret.missing`)، و`params` له، و`node` و`field` اللذان يتعلق بهما. ويمكن للسكربت أن
يتفرع على `error.code` بأي لغة؛ ونصوص كل رمز مسرودة في [رسائل الأخطاء](../reference/errors.md).

### رموز الخروج {#exit-codes}

| الرمز | المعنى |
| --- | --- |
| `0` | نجحت كل التجارب؛ ونجح الإرسال؛ ولا شيء يعترض الطريق. |
| `1` | عملت تجربة وفشلت، أو نفد وقتها أو أُوقفت؛ أو فشل إرسال (رُفض، أو لا جواب، أو حالة غير متوقعة)؛ أو وجد `doctor` ما يعترض الطريق. |
| `2` | الأمر أو مستند خاطئ: وسيط، أو ملف تتعذر قراءته، أو خطأ تحقق، أو معامل مجهول، أو سر مفقود. |
| `3` | تعذر تشغيل أي شيء لسبب خارج التجربة: تعذر بلوغ الخادم أو رفضه الرمز المميز، أو تعذر فتح منفذ، أو تعطل مخزن بيانات الاعتماد. |

ومع عدة تجارب تقرر أخطر نتيجة، بهذا الترتيب: `2` ثم `3` ثم `1` ثم `0`.

### متغيرات البيئة {#environment}

| المتغير | ما يفعله |
| --- | --- |
| `SIGNALLAB_LANG` | اللغة، حين لا يُعطى `--lang`. |
| `LC_ALL`، `LC_MESSAGES`، `LANG` | اللغة، حين لا يكون أي مما سبق معيَّنًا. |
| `SIGNALLAB_SERVER` | الخادم لـ `run` و`validate` و`emulate` و`mcp` و`doctor`، كما يعطيه `--server`. |
| `SIGNALLAB_TOKEN` | رمز وصول الخادم المميز، حين لا يُعطى ملف رمز. |
| `SIGNALLAB_TOKEN_FILE` | ملف يحوي رمز الخادم المميز، كما يعطيه `--token-file`. |
| `SIGNALLAB_SECRET_<NAME>` | قيمة السر `NAME` للتشغيلات في هذه العملية (انظر [الأسرار](#secrets)). |
| `SIGNALLAB_DATA_DIR` | مجلد بيانات التطبيق، حيث ينظر `fire` و`emulators` و`emulate` و`mcp` و`doctor` افتراضيًّا؛ وإلا `Documents/SignalLab` في مجلدك الرئيسي. |
| `GITHUB_ACTIONS` | حين تكون `true`، يُطبع التشغيل الذي لا ينجح أيضًا تعليقًا توضيحيًّا `::error`، يعرضه GitHub على صفحة التشغيل. |

## `run` {#cli-run}

```text
signallab run [OPTIONS] <FILE>...
```

يشغّل التجارب واحدة بعد أخرى ويخرج بـ `0` فقط حين تنجح كلها.

| الخيار | ما يفعله | الافتراضي |
| --- | --- | --- |
| `<FILE>...` | ملفات تجارب، أو أسماء [قوالب مضمَّنة](#cli-templates). | مطلوب |
| `-p`، `--param NAME=VALUE` | قيمة معامل لهذا التشغيل؛ تُكرر للمزيد. | قيم المستند |
| `--profile NAME` | التشغيل بهذا الملف؛ ويجب أن يكون في كل تجربة معطاة. و`""` يشغّل بالقيم الافتراضية. | ملف تعريف المستند |
| `-m`، `--matrix NAME=V1,V2` | تشغيل مرة لكل قيمة؛ وتُكرر لأسماء أخرى (انظر [مصفوفة من التشغيلات](#matrix)). | |
| `--matrix-file PATH` | تركيبات من ملف JSON. | |
| `--seed N` | بذرة القيم العشوائية، من 0 إلى 9007199254740991 (2⁵³ − 1). | بذرة المستند، وإلا بذرة جديدة لكل تشغيل |
| `--timeout SECONDS` | إفشال تشغيل يستغرق أكثر، من 1 إلى 300. | `300` |
| `--fail-fast` | التوقف عند أول تشغيل لا ينجح؛ ولا يُبدأ الباقي. | معطل |
| `--junit PATH` | كتابة تقرير JUnit XML هناك (انظر [التقارير](#reports)). | |
| `--report PATH` | نسخ تقرير التشغيل هناك: ملف لتشغيل واحد، ومجلد لعدة تشغيلات. | |
| `--data-dir PATH` | مجلد البيانات لتشغيلات هذه العملية؛ وتبقى تقاريرها فيه. | مجلد مؤقت، يُزال عند الخروج |
| `--server URL` | التشغيل على هذا الخادم بدل هذه العملية (انظر [على خادم](#run-on-server)). | `SIGNALLAB_SERVER` |
| `--token-file PATH` | ملف يحوي رمز الخادم المميز. | `SIGNALLAB_TOKEN_FILE`، وإلا `SIGNALLAB_TOKEN` |
| `--secrets files\|system` | من أين تأتي قيم الأسرار في هذه العملية. | `files` |
| `--secrets-dir PATH` | مجلد ملفات أسرار، ملف لكل اسم. | `/run/secrets/signallab` إن وُجد |

`--data-dir` و`--secrets` و`--secrets-dir` تخص التشغيلات في هذه العملية؛ ولا يمكن جمعها مع `--server`.

### الملفات والقوالب {#run-inputs}

`FILE` تجربة كما يحفظها التطبيق ويصدّرها ([[ui:exp.exportJson]] في شاشة [[ui:nav.experiment]]). ويصلح أي إصدار
مستند يفتحه التطبيق؛ والأقدم يُحدَّث أثناء قراءته، كما يفعل التطبيق عند فتحه. والاسم الذي ليس ملفًّا يُبحث عنه
بين [القوالب المضمَّنة](#cli-templates)، مع `.json` أو دونها:

```bash
signallab run tests/stage-cues.json tests/api.json
signallab run osc-ping-reply --param device=192.0.2.20:9000
```

تُفحص كل تجربة — وكل تركيبة من مصفوفة — كما يفحصها المحرر قبل أن يبدأ أولها. والملف الثالث المعطوب يوقف
الأول أيضًا، قبل إرسال أي شيء، برمز الخروج `2`.

### المعاملات وملفات التعريف {#run-params}

`--param NAME=VALUE` يعيّن معاملًا لهذا التشغيل وحده؛ ولا يتغير الملف. والقيمة هي كل ما بعد أول `=`، فيعمل
`--param url=http://127.0.0.1/?q=1`، ويعيّن `--param note=` قيمة فارغة. وتنطبق القيمة على كل تجربة معطاة فيها
ذلك المعامل، ويجب أن تسمّي معاملًا في واحدة منها على الأقل — والاسم المكتوب خطأ يُرفض برمز الخروج `2`.

`--profile NAME` يشغّل بأحد ملفات تعريف التجربة، كما يفعل اختياره تحت [[ui:exp.profile]] في المحرر. انظر
[البيانات والقوالب](../experiments/data.md).

### مصفوفة من التشغيلات {#matrix}

تشغّل المصفوفة التجربة نفسها على كل هدف وكل مستخدم وكل حجم حمولة. وكل تركيبة تشغيل قائم بذاته، بنتيجته
وتقريره ومجموعته في تقرير JUnit.

```bash
signallab run smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest \
  --fail-fast --junit junit.xml
```

هذه أربعة تشغيلات: `device` يتغير أبطأ، و`user` أسرع. وتُسمى باسم الملف وقيمها:
`smoke.json [device=192.0.2.20:9000, user=admin]`.

- `--matrix NAME=V1,V2` يضيف محورًا. والاسم نفسه مرة أخرى يضيف إليه قيمًا. وتُهمل المسافات حول الأسماء
  والقيم؛ والقيمة المعطاة مرتين تعمل مرة.
- `--matrix-file PATH` يقرأ JSON بإحدى صيغتين:

  ```json
  { "device": ["192.0.2.20:9000", "192.0.2.21:9000"], "retries": [1, 3] }
  ```

  يضيف محاور — تعمل كل التركيبات، وهذه الأسماء بعد أسماء `--matrix`، بترتيب أبجدي؛

  ```json
  [
    { "device": "192.0.2.20:9000", "user": "admin" },
    { "device": "192.0.2.21:9000", "user": "guest" }
  ]
  ```

  يسرد التركيبات نفسها، وتُضرب كل منها في محاور `--matrix`. والقيم نصوص أو أرقام أو `true`/`false`؛
  والفاصلة داخل قيمة في ملف تبقى جزءًا منها.
- يجب أن يكون كل اسم في المصفوفة معاملًا في تجربة واحدة معطاة على الأقل. والتجربة التي ليس فيها أحد
  الأسماء تعمل مرة واحدة، لا مرة لكل قيمة كانت ستتجاهلها.
- الاسم المعيَّن بـ `--param` وبالمصفوفة معًا، أو بـ `--matrix` وبالملف معًا، يُرفض.
- **256** تشغيلًا على الأكثر من أمر واحد؛ وما زاد يُرفض قبل أن يعمل شيء.

### التشغيل على خادم {#run-on-server}

```bash
signallab run tests/stage.json --server http://192.0.2.10:1430 --token-file token.txt
```

تأتي التجربة من هذا الجهاز؛ ويشغّلها الخادم بشبكته هو و[أسراره](../server/index.md#secrets) ومجلد بياناته،
ويبث الخطوات عائدًا كلما حدثت. ويبقى التقرير على الخادم — ويُطبع مساره — و`--report` ينزّل نسخة. والتشغيل
على خادم مهمة هناك كأي تشغيل بدأ من واجهته: وتعرضه كل صفحة مسجَّل دخولها إليه. وإذا غاب `signallab` في منتصف
التشغيل، ينتهي التشغيل على الخادم ويحتفظ بتقريره.

يُقرأ الرمز المميز من `--token-file` (أو `SIGNALLAB_TOKEN_FILE`)، وإلا من `SIGNALLAB_TOKEN`، ويُرسل بالصيغة
`Authorization: Bearer`. والخادم الذي يتعذر بلوغه، أو يرفض الرمز، أو يتوقف عن الإجابة في منتصف التشغيل
هو رمز الخروج `3`. ويفحص `signallab doctor --server URL` العنوان والرمز وحدهما.

### التقارير {#reports}

يكتب كل تشغيل التقرير نفسه الذي يكتبه التطبيق. وبلا `--data-dir` تستخدم التشغيلات مجلدًا مؤقتًا يُزال حين
يخرج `signallab`، فيُحتفظ بما يلزم:

- `--report PATH` ينسخ التقرير: إلى `PATH` نفسه لتشغيل واحد، أو إلى المجلد `PATH` لعدة تشغيلات، باسم
  `01-<file>.json` و`02-<file>.json` و… بترتيب تشغيلها.
- `--data-dir PATH` يحتفظ بتقرير كل تشغيل في ذلك المجلد (تحت `runs/`)، ويطبع موضع كل منها.

`--junit PATH` يكتب تقرير JUnit XML، الصيغة التي يقرؤها كل نظام CI:

- `<testsuite>` لكل تشغيل (لكل تركيبة في مصفوفة)، مع الملف والبذرة والنتيجة وملف التعريف ومسار التقرير
  وكل قيمة مصفوفة (`param.NAME`) خصائص؛
- `<testcase>` لكل عقدة عملت، باسم نوع العقدة ومعرّفها، مع زمنها الخاص؛
- `<failure>` على العقدة التي فشلت: الرسالة باللغة المختارة، ورمز الخطأ بوصفه `type` لها، وخطوات العقدة
  مع التفصيل التقني؛
- حالة `<skipped>` لكل عقدة لم يبلغها التشغيل (الجانب الآخر من فرع)؛
- ومجموعة بحالة `<error>` واحدة لتجربة تعذر بدؤها، ومجموعة بحالة `<skipped>` لكل تشغيل لم يبدأه
  `--fail-fast`.

```xml
<testsuites name="Signal Lab" tests="6" failures="1" errors="0" time="2.006">
  <testsuite name="HTTP to OSC" tests="6" failures="1" errors="0" skipped="4" time="2.006" timestamp="2026-10-04T16:48:03">
    <properties>
      <property name="file" value="status-branch" />
      <property name="seed" value="42" />
      <property name="outcome" value="failed" />
      <property name="report" value="out/report.json" />
    </properties>
    <testcase name="Start (start)" classname="HTTP to OSC" time="0.001">
      <system-out>Started · seed 42</system-out>
    </testcase>
    <testcase name="HTTP request (request)" classname="HTTP to OSC" time="2.004">
      <failure message="http://127.0.0.1:8080/ refused the connection — nothing is listening on that port" type="transport.refused">…</failure>
    </testcase>
    <testcase name="Status branch (branch)" classname="HTTP to OSC" time="0.000">
      <skipped message="not reached in this run" />
    </testcase>
    …
  </testsuite>
</testsuites>
```

وعقدة HTTP تحت [الحمل](../experiments/load.md) تضيف سطرًا تحت آخر خطواتها لكل عتبة، تحققت أم لا
(`✕ p95 < 100 ms · 152.58 ms`)؛ والعتبة التي لا تتحقق تُفشل التشغيل وحالته في JUnit
(`type="load.threshold"`).

### الأسرار {#secrets}

تقرأ التجربة السر بالصيغة `{{secret.NAME}}`. وللتشغيلات في هذه العملية تأتي القيمة من:

| `--secrets` | من أين تأتي قيمة `NAME` |
| --- | --- |
| `files` (الافتراضي) | متغير البيئة `SIGNALLAB_SECRET_NAME`؛ وإلا ملف باسم `NAME` في `--secrets-dir` (افتراضيًّا `/run/secrets/signallab`، حين يوجد ذلك المجلد — تخطيط أسرار Docker). |
| `system` | مدير بيانات الاعتماد في Windows — حيث يحفظ التطبيق قيم [[ui:exp.secrets]] فيه. وليس في Linux مخزن بيانات اعتماد يقرؤه `signallab`: فهناك يفشل `--secrets system` برمز الخروج `3`. |

فاصل السطر في آخر الملف ليس من القيمة، والقيمة الفارغة تُعدّ غير معيَّنة. والأسماء أحرف وأرقام و`_`، لا تبدأ
برقم، حتى 128 حرفًا؛ والقيمة 16 KiB على الأكثر.

```bash
SIGNALLAB_SECRET_API_TOKEN="$API_TOKEN" signallab run tests/api.json
```

السر غير المعيَّن يوقف التشغيل قبل أي حركة مرور، برمز الخروج `2` واسم المفقود. ولا تُطبع القيمة أبدًا: تعرض
الخطوات والأخطاء والتقارير وتقرير JUnit `••••` مكانها. ومع `--server` تكون الأسرار أسرار الخادم.

### ما يطبعه `run` {#run-output}

أثناء التشغيل تكون كل خطوة سطرًا على stderr — زمنها منذ البدء والعقدة وحالتها وما فعلته — كالخط الزمني في
التطبيق. وحين ينتهي، يقول سطر واحد على stdout كيف سار:

```text
▶ HTTP check (http-check) · seed 1185927457137919
     0.000  Start         Running
     0.000  Start         Passed · Started · seed 1185927457137919
     0.000  HTTP request  Running
     2.004  HTTP request  Failed · URL — http://127.0.0.1:8080/ refused the connection — nothing is listening on that port
✖ HTTP check failed after 2 s: HTTP request · URL — http://127.0.0.1:8080/ refused the connection — nothing is listening on that port
  Technical details: error sending request for url (http://127.0.0.1:8080/): … (os error 10061)
  To run it again with the same random values: --seed 1185927457137919
```

التشغيل الفاشل يسمّي البذرة التي استخدمها: و`--seed` بذلك الرقم يشغّله من جديد بالقيم العشوائية نفسها. وبعد
الحكم يأتي، على stderr، ما سُئل عنه كل محاكٍ في التشغيل — الطلبات، وكم لم تأخذه قاعدة، وكم فشل، والإصابات
لكل قاعدة:

```text
✔ Retry a flaky API passed in 7 ms
  Flaky API: 3 requests, 0 without a rule, 0 failed · #1 3
```

وما فعله كل مرحّل إضعاف: ما استلمه وأسقطه وقيّده، وكل مرحلة بالشكل مُمرَّر / مستلم:

```text
  127.0.0.1:19110 → 127.0.0.1:19100: 155 datagrams, 38 dropped, 0 throttled · lan 0.0–2.0 s 39/39, wifi 2.0–4.0 s 39/39, offline 4.0–6.0 s 0/38, lan 6.0–8.0 s 38/38
```

والمرحّل عبر TCP ينقل أجزاء من تدفقات لا مخططات بيانات، ولا يُسقط شيئًا، فيعدّ سطره الأجزاء والاتصالات
وإعادات التعيين والاتصالات نصف المفتوحة ومرات احتباس التدفق بسبب حد عرض النطاق:

```text
  127.0.0.1:19120 → 127.0.0.1:19101: 14 chunks, 2 connections, 1 reset, 0 half-open, 3 held back
```

وعقدة HTTP تحت [الحمل](../experiments/load.md) تبلغ عن تقدمها خطوةً مرة في الثانية على الأكثر.

ومع عدة تشغيلات ينتهي الخرج بسطر لكل تشغيل وعدّاد: `3 runs: 2 passed, 1 failed`. ومع `--fail-fast` تُحصى
التشغيلات التي لم يبدأها على stderr.

ومع `--json` يكون كل سطر كائنًا له `type`:

```json
{"type":"started","experiment":"Empty experiment","file":"empty","job_id":1,"overridden":false,"profile":null,"seed":1,"started_ms":1791132204585}
{"type":"step","node_id":"start","state":"passed","detail":"Started","message_key":"exp.step.started","message_params":{"seed":1},"job_id":1,"ts":1791132204585}
{"type":"ended","experiment":"Empty experiment","file":"empty","outcome":"passed","seed":1,"params":{},"steps":[…],"report_path":"…","started_ms":1791132204585,"ended_ms":1791132204585,…}
{"type":"summary","total":1,"passed":1,"failed":0,"not_started":0,"exit_code":0}
```

تحمل أسطر `started` و`ended` و`error` الحقل `file` (الوسيط كما أُعطي) و، في المصفوفة، `matrix` (قيم التركيبة).
و`ended` هو نتيجة التشغيل كاملة: `outcome` (`passed` أو `failed` أو `stopped`) و`seed` و`profile` و`params`
و`error` وكل خطوة، و`emulators` و`impairments` حين كانت للتشغيل، و`report_path`. وتحمل آخر خطوة في حمل
`load` بكل رقم قاسه: `planned` و`sent` و`ok` و`failed` و`missed` و`rps` و`error_rate` و`min_ms` و`mean_ms`
و`max_ms` و`p50_ms` إلى `p99_ms` و`statuses`، وكل ثانية (`seconds`)، و`histogram`، وحكم كل عتبة
(`thresholds`).

## `validate` {#cli-validate}

```text
signallab validate [OPTIONS] <FILE>...
```

يفحص التجارب كما يفعل المحرر قبل التشغيل — المخطط وكل حقل والقوالب والمعاملات والأسرار — ولا يرسل شيئًا.
ويخرج بـ `0` حين تبدأ كل تجربة.

يقبل مدخلات [`run`](#cli-run): ملفات وقوالب، و`--param` و`--profile` و`--matrix` و`--matrix-file`، و`--server`
و`--token-file` و`--secrets` و`--secrets-dir`. ومع `--server` يفحصها الخادم، على أسراره هو.

```text
✔ tests/stage.json: Stage cues would run
  Rehearsal: Would not run: …
```

والمشكلة التي لا تكون إلا في ملف تعريف آخر من المستند تُسرد تحته، لكنها لا تُفشل الفحص. ومع `--json`، سطر لكل تجربة
(لكل تركيبة): `{"experiment", "file", "valid": true, "profile_issues": […]}`، أو `"valid": false` مع `error`
و`exit_code`.

## `send` {#cli-send}

```text
signallab send <osc|udp|http|ws|mqtt> …
```

يرسل رسالة واحدة عبر الأوامر نفسها التي تستخدمها شاشات التطبيق، فتكون البايتات نفسها على السلك. والإرسال لا
يقرأ مكتبة ولا أسرارًا.

رموز الخروج: `0` أُرسل، `1` فشل الإرسال، `2` وسيط خاطئ.

`<host:port>` عنوان IP أو اسم مضيف، ومنفذ: `127.0.0.1:9000` أو `[::1]:9000` (عنوان IPv6 بين قوسين معقوفين) أو
`device.local:9000`. ويُبحث عن الاسم حين يعمل الأمر، ويؤخذ عنوان IPv4 إن وُجد، فيصل
`localhost:9000` إلى مستقبِل على `127.0.0.1`. والاسم الذي لا يُحل يُفشل الإرسال (`1`)؛ والهدف بلا منفذ وسيط
غير صالح (`2`).

### `send osc` {#cli-send-osc}

```text
signallab send osc <host:port> <address> [ARG]...
```

رسالة OSC واحدة. تُحدَّد أنواع الوسائط ببادئة، أو تُستنتج:

| الوسيط | نوع OSC |
| --- | --- |
| `i:3` | int32 |
| `f:0.5` | float32 |
| `d:1.5` | float64 (double) |
| `h:64` | int64 |
| `s:text` | سلسلة |
| `b:de ad be ef` | كتلة، بايتات ست عشرية |
| `T`، `F` | true، false |
| `N` | nil |
| `3`، `-3` | العدد الصحيح العادي int32 |
| `3000000000` | العدد الصحيح الذي يتجاوز 32 بتًّا int64 |
| `2.5`، `1e3` | العدد العشري العادي، أو العدد ذو الأس، float32 (وfloat64 فيما يتجاوز مدى float32) |
| أي شيء آخر | سلسلة |

```bash
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main 3
# ✔ sent /cue/go (32 bytes) → 127.0.0.1:9000
```

يوضع الوسيط الذي فيه مسافات بين علامتي اقتباس: `"s:hello world"`. و`s:7` يرسل النص `7`. ويجب أن يبدأ العنوان
بـ `/`؛ وما لا يبدأ بها وسيط غير صالح (`2`).

::: warning Git Bash على Windows
يحوّل Git Bash الوسائط التي تبدأ بـ `/` إلى مسارات Windows، فيصل `/cue/go` على هيئة
`C:/Program Files/Git/cue/go`. يُكتب `//cue/go`، أو يُشغَّل بـ `MSYS_NO_PATHCONV=1`. ولا يتأثر PowerShell و`cmd`.
:::

انظر [OSC](../protocols/osc.md).

### `send udp` {#cli-send-udp}

```text
signallab send udp <host:port> (--text TEXT | --hex HEX)
```

مخطط بيانات UDP واحد. تُعطى الحمولة بـ `--text`، أو بايتات `--hex`: `"de ad be ef"` أو `deadbeef` أو `0xDE,0xAD`.

```bash
signallab send udp 127.0.0.1:7000 --text "PLAY 1"
signallab send udp 127.0.0.1:7000 --hex "de ad be ef"
```

### `send http` {#cli-send-http}

```text
signallab send http <METHOD> <URL> [OPTIONS]
```

طلب HTTP واحد. يذهب سطر الحالة إلى stderr — `HTTP 200 OK · 3 ms · 1,234 B` — ومتن الاستجابة إلى stdout، فيمكن
تمريره إلى غيره. والمتن الأطول من 256 KiB يُقطع عندها، ويقول stderr ذلك.

| الخيار | ما يفعله | الافتراضي |
| --- | --- | --- |
| `-H`، `--header "Name: value"` | ترويسة طلب؛ وتُكرر للمزيد. | |
| `--body TEXT` | متن الطلب. و`@FILE` يرسل محتويات ملف نصي. | لا شيء |
| `--expect-status STATUS` | الخروج بـ `1` ما لم تكن للاستجابة هذه الحالة. | أي حالة تعني `0` |
| `--timeout MS` | المللي ثانية المنتظرة للاستجابة. | `10000` |
| `-u`، `--user NAME:PASSWORD` | بيانات اعتماد، تُرسل بصيغة Basic. | |
| `--digest` | مع `--user`: يجيب تحدّي Digest الصادر من الخادم بدلًا من ذلك (MD5 أو SHA-256). | معطل |
| `--bearer TOKEN` | يرسل `Authorization: Bearer TOKEN`. ولا يجتمع مع `--user`. | |

```bash
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab send http POST http://127.0.0.1:8080/cue -H "Content-Type: application/json" --body '{"cue": 1}'
signallab send http GET http://127.0.0.1:8080/admin -u admin:secret --digest
```

بلا `--expect-status` تكون أي استجابة نجاحًا، حتى `500`. والطلب الذي لا يلقى استجابة (رُفض، أو انتهت مهلته،
أو اسم لا يُحل) هو رمز الخروج `1`، وكذلك تحدّي Digest الذي تعذرت الإجابة عنه — ويُطبع السبب بعد الاستجابة.

::: tip
تظهر الوسائط لمستخدمين آخرين على الجهاز نفسه. تُترك كلمات المرور الحقيقية للتجارب، حيث تكون
[أسرارًا](#secrets).
:::

انظر [HTTP](../protocols/http.md).

### `send ws` {#cli-send-ws}

```text
signallab send ws <URL> [OPTIONS]
```

تبادل WebSocket واحد: الاتصال بـ `ws://…` أو `wss://…`، وإرسال رسالة، وانتظار الجواب اختياريًّا، ثم الإغلاق.
تذهب المصافحة وما أُرسل إلى stderr، والجواب إلى stdout (والرسائل الثنائية بالست عشري).

| الخيار | ما يفعله | الافتراضي |
| --- | --- | --- |
| `--text TEXT` | إرسال هذه الرسالة النصية. | لا شيء يُرسل |
| `--hex HEX` | إرسال هذه البايتات رسالةً ثنائية. ولا يجتمع مع `--text`. | |
| `-H`، `--header "Name: value"` | ترويسة لطلب الترقية؛ وتُكرر للمزيد. | |
| `--protocol NAME` | بروتوكول فرعي للعرض؛ ويُكرر للمزيد، بترتيب التفضيل. | |
| `--expect TEXT` | انتظار رسالة تحوي هذا النص. | |
| `--expect-regex REGEX` | انتظار رسالة تطابق هذا التعبير النمطي. | |
| `--wait` | انتظار أي رسالة. | |
| `--timeout MS` | المللي ثانية المنتظرة للجواب. | `2000` |

```bash
signallab send ws ws://127.0.0.1:9001/echo --text '{"ping": 1}' --expect '"ping"'
signallab send ws ws://127.0.0.1:9001/feed --wait        # nothing sent: the server's first message
```

بلا `--expect` أو `--expect-regex` أو `--wait` يتصل ويرسل ويغلق دون انتظار. وحين لا يأتي الجواب المتوقع في
الوقت المحدد يكون رمز الخروج `1`. انظر [WebSocket](../protocols/websocket.md).

### `send mqtt` {#cli-send-mqtt}

```text
signallab send mqtt <host:port> <topic> [payload] [--qos 0|1|2] [--retain]
```

نشر MQTT 3.1.1 واحد عبر TCP عادي، دون بيانات اعتماد، بوصفه عميلًا قائمًا بذاته بمعرّف عميل جديد، فلا يقطع
أبدًا اتصالًا موجودًا. وبلا منفذ يكون الوسيط على `1883`. والموضوع موضوع واحد: وإذا كان فيه `+` أو `#`، أو كان
فارغًا، رُفض الأمر قبل أن يتصل (`2`).

| الخيار | ما يفعله | الافتراضي |
| --- | --- | --- |
| `[payload]` | الحمولة. | فارغة |
| `--qos 0\|1\|2` | جودة الخدمة. | `0` |
| `--retain` | الاحتفاظ بها قيمةً محتفظًا بها للموضوع. والحمولة الفارغة مع `--retain` تمسحها. | معطل |

```bash
signallab send mqtt 127.0.0.1:1883 lab/light/1/set on --qos 1
signallab send mqtt 127.0.0.1 lab/light/1/state "" --retain     # clear the retained value
```

انظر [MQTT](../protocols/mqtt.md).

## `fire` {#cli-fire}

```text
signallab fire <signal> [--library PATH]
```

يرسل إشارة من مكتبة إشارات تمامًا كما تُرسلها شاشة [[ui:nav.signals]] في التطبيق: إشارات OSC وUDP وHTTP
وMQTT. وتُعرف الإشارة بمعرّفها، وإلا باسمها دون مراعاة حالة الأحرف؛ والاسم الذي تتشاركه عدة إشارات يُرفض
مع ذكر معرّفاتها.

| الخيار | ما يفعله | الافتراضي |
| --- | --- | --- |
| `<signal>` | معرّف الإشارة أو اسمها. | مطلوب |
| `--library PATH` | ملف المكتبة. | `signals.json` في مجلد بيانات التطبيق |

```bash
signallab fire "Fader value"
signallab fire go --library show/signals.json
```

تُقرأ المكتبة فقط، ولا تُنشأ ولا تتغير. انظر [الإشارات](../tools/signals.md).

## `emulate` {#cli-emulate}

```text
signallab emulate [OPTIONS] <FILE|NAME>...
```

يؤدي دور الطرف الآخر — واجهة HTTP API، أو جهاز OSC أو UDP أو TCP، أو وسيط MQTT — إلى أن يُضغط
<kbd>Ctrl</kbd>+<kbd>C</kbd> أو تنقضي `--for`، ويطبع كل طلب كلما أُجيب. و`FILE` يحوي محاكيًا واحدًا، أو قائمة
منها، أو مكتبة كاملة كما يكتبها التطبيق؛ و`NAME` معرّف أحدها أو اسمه في مكتبة التطبيق (مكتبة شاشة
[[ui:nav.emulators]]).

| الخيار | ما يفعله | الافتراضي |
| --- | --- | --- |
| `-p`، `--param NAME=VALUE` | قيمة تقرؤها قوالبه بالصيغة `{{NAME}}`؛ وتُكرر للمزيد. | |
| `--bind IP:PORT` | الاستماع هناك بدلًا من ذلك. مع محاكٍ واحد فقط. | عنوان المحاكي نفسه |
| `--for SECONDS` | التوقف بعد هذه المدة. | إلى أن يُضغط <kbd>Ctrl</kbd>+<kbd>C</kbd> |
| `--seed N` | بذرة اختياراته العشوائية: ترتيب عشوائي، وتذبذب، ومولدات. | |
| `--check` | فحص المحاكيات والخروج، دون فتح منفذ. | معطل |
| `--library PATH` | المكتبة التي يُبحث فيها عن الأسماء. | `emulators.json` في مجلد بيانات التطبيق |
| `--server URL`، `--token-file PATH` | تشغيلها على خادم، ومتابعتها عبر واجهة API الخاصة به، وإيقافها في النهاية. | |
| `--secrets`، `--secrets-dir` | كما في [`run`](#cli-run). | |

```text
$ signallab emulate tests/orders-api.json --for 60
Orders API (http) answering on 127.0.0.1:18099
answering for 60 s
+   1.209 s Orders API  #1  GET /orders/42 → 200 OK · 12 B  1 ms  ← 127.0.0.1:55744
+   1.209 s Orders API  —  GET /nothing → 404 Not Found · 20 B  2 ms  ← 127.0.0.1:55745
Orders API: 2 requests, 1 without a rule, 0 failed · #1 1
```

كل سطر هو الزمن منذ البدء، والمحاكي، والقاعدة التي أجابت (`#1`، أو `—` إن لم تكن هناك قاعدة)، والطلب وما
لقيه، والزمن الذي استغرقه، ومن أرسله. وفي النهاية عدّادات كل محاكٍ: الطلبات، وكم لم تأخذه قاعدة، وكم فشل،
وكم صادف انقطاعًا أو لم يُسلَّم حين كانت هناك حالات، والإصابات لكل قاعدة.

رموز الخروج: `0` حين يتوقف عند <kbd>Ctrl</kbd>+<kbd>C</kbd> أو `--for`؛ و`2` حين يكون محاكٍ غير صالح؛ و`3` حين
يكون منفذه مأخوذًا أو يتعذر فتحه، أو يفشل مقبس أثناء الإجابة.

وفي خط CI يُبدأ في الخلفية، ويُختبر النظام عليه، وتُقرأ العدّادات في النهاية:

```bash
signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
npm test          # the system under test, configured for the emulator's address
wait              # the last lines of mock.ndjson are the counts
```

وعلى الخادم، المحاكي الذي بدأ بـ `--server` يتوقف حين ينتهي `signallab` انتهاءً عاديًّا؛ والذي تركته عملية
قُتلت يمكن إيقافه من واجهة الخادم. انظر [المحاكيات](../tools/emulators.md).

## `emulators` {#cli-emulators}

```text
signallab emulators [--library PATH]
```

يسرد محاكيات المكتبة: المعرّف والاسم والبروتوكول والعنوان وعدد القواعد. و`--library PATH` يقرأ ملف مكتبة آخر
بدلًا من `emulators.json` في مجلد بيانات التطبيق. و`signallab emulate <id>` يبدأ أحدها.

المكتبة تُقرأ فقط. وإذا لم يوجد الملف — فالتطبيق ينشئ الملف الذي في مجلد بياناته عند أول تشغيل له — يقول الأمر ذلك
ويخرج بـ `2`.

## `templates` {#cli-templates}

```text
signallab templates
```

يسرد القوالب المضمَّنة، التي يأخذها `run` و`validate` بالاسم. وهي قوالب التطبيق نفسه:

| الاسم | في التطبيق | المعاملات |
| --- | --- | --- |
| `empty` | [[ui:exp.templateEmpty]] | |
| `http-check` | [[ui:exp.templateHttp]] | |
| `status-branch` | [[ui:exp.templateBranch]] | |
| `parallel-flows` | [[ui:exp.templateParallel]] | |
| `osc-ping-reply` | [[ui:exp.templatePingReply]] | `device` = `127.0.0.1:9000` |
| `poll-until-ready` | [[ui:exp.templatePoll]] | `device` = `127.0.0.1:9000` |
| `flaky-api` | [[ui:exp.templateFlaky]] | `api` = `http://127.0.0.1:18080` |
| `fault-phases` | [[ui:exp.templateFaults]] | |
| `dependency-outage` | [[ui:exp.templateOutage]] | `api` = `http://127.0.0.1:18090` |
| `websocket-echo` | [[ui:exp.templateWsEcho]] | `service` = `ws://127.0.0.1:9001/echo` |

كل قالب يشير إلى عنوان الاسترجاع. و`flaky-api` و`fault-phases` و`dependency-outage` تأتي بمحاكياتها الخاصة،
فتعمل دون أن يستمع شيء آخر — طريقة سريعة لرؤية `signallab` يعمل.

## `nodes` {#cli-nodes}

```text
signallab nodes
```

يطبع، بصيغة JSON، ما تتألف منه التجربة: شكل المستند وقواعده، وكل نوع من العقد مع تسميته ووصفه وحقوله
ومخارجه ومثال يقبله المحرك، ولغة `{{template}}`، وملفات تعريف الحمل، ومستند المحاكي. وهو ما يقرؤه المساعد عبر
[`signallab mcp`](mcp.md) ليكتب تجربة؛ وللبشر تقول صفحة [العقد](../experiments/nodes.md) الشيء نفسه بكلام أكثر.

## `mcp` {#cli-mcp}

```text
signallab mcp [OPTIONS]
```

يخدم Signal Lab مساعد ذكاء اصطناعي عبر Model Context Protocol، على stdin وstdout. وكل شيء عنه — إعداده في
Claude Code وClaude Desktop وCursor وVS Code، وخياراته وأدواته — في [المساعدات (MCP)](mcp.md).

## `doctor` {#cli-doctor}

```text
signallab doctor [--server URL] [--token-file PATH]
```

يقول ما قد يقف بين Signal Lab والمعدات، ويخرج بـ `1` حين يقف شيء. وتتبع أسطره [`--lang`](#language) كبقية سطر
الأوامر:

- الشبكة التي يوجد عليها هذا الجهاز: اسمه وعنوانه؛
- مجلد البيانات: هل يمكن الكتابة فيه (والذي لم يُنشأ بعد لا بأس به — ينشئه التطبيق عند أول استخدام)؛
- جدار الحماية: على Windows، لـ `signallab` وللتطبيق المكتبي كلًّا على حدة، هل تتيح قاعدة للأجهزة الأخرى
  الدخول على نوع الشبكة التي عليها الجهاز الآن (خاصة أو نطاق أو عامة)، أو تحجبها قاعدة — وهو ما يتركه *Cancel*
  عند مطالبة النظام؛ وعلى Linux، هل ufw أو firewalld يعمل، والأمر الذي يفتح منفذًا؛
- مع `--server` (أو `SIGNALLAB_SERVER`): هل يجيب الخادم ويقبل الرمز.

```text
signallab [[version]]
Network: LAB-PC · 192.0.2.15
Data folder: C:\Users\lab\Documents\SignalLab — writable
Firewall · signallab (C:\…\Signal Lab\signallab.exe) · private network: not allowed yet — signallab firewall allow
Firewall · app (C:\…\Signal Lab\signal-lab.exe) · private network: allowed
✖ 1 thing in the way
```

`--json` يطبع الشيء نفسه كائنًا واحدًا. انظر [استكشاف الأخطاء](../reference/troubleshooting.md).

## `firewall` {#cli-firewall}

```text
signallab firewall allow [--public]
```

على Windows، يتيح للأجهزة الأخرى الوصول إلى Signal Lab — وهو ما يحتاج إليه مراقب أو عقدة انتظار لتسمع جهازًا.
يطلب Windows أولًا صلاحيات المسؤول؛ ثم تُستبدل قواعد الوارد لـ `signallab` وللتطبيق المكتبي (الموجود بجانبه،
أو حيث تضعه برامج التثبيت) بقاعدة سماح واحدة لكل منهما، بما فيها قواعد الحجب، على الشبكات الخاصة وشبكات
النطاق.

| الخيار | ما يفعله |
| --- | --- |
| `--public` | وعلى الشبكات العامة أيضًا — وشبكة Wi-Fi في مكان العرض كثيرًا ما تكون كذلك. |

رموز الخروج: `0` تم؛ و`3` حين تُرفض مطالبة المسؤول أو يفشل التغيير. وعلى Linux لا يتغير شيء: بل يطبع أمر ufw
أو firewalld الذي يفتح المنافذ التي تستمع عليها، ويخرج بـ `0`.

لا يتغير جدار الحماية إلا حين يُشغَّل هذا الأمر؛ ولا شيء آخر في `signallab` يمسه.

## `version` {#cli-version}

```text
signallab version
```

يطبع `signallab [[version]]` — ومع `--json`، `{"version": "[[version]]"}`. و`signallab --version` يطبع الإصدار نفسه.

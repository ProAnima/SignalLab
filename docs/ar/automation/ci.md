---
title: خطوط CI
description: تشغيل تجارب Signal Lab في GitHub Actions أو GitLab CI أو أي خط معالجة، وإفشال المهمة حين تفشل تجربة، والاحتفاظ بتقرير JUnit.
---

# Signal Lab في CI

التجارب التي تُشغّل التركيب هي أيضًا اختبارات الانحدار الخاصة به. وفي خط CI يشغّلها
[`signallab run`](cli.md#cli-run) دون نافذة، ويطبع كل خطوة، ويكتب تقرير JUnit يعرضه كل نظام CI، ويخرج برمز
تفهمه المهمة:

| رمز الخروج | ما يجب أن يفعله الخط |
| --- | --- |
| `0` | المتابعة: نجحت كل التجارب |
| `1` | الفشل: عملت تجربة وفشلت |
| `2` | الفشل: تجربة أو الأمر خاطئ (خطأ تحقق، أو معامل مجهول، أو سر مفقود) |
| `3` | الفشل أو إعادة المحاولة: تعذر تشغيل أي شيء (تعذر بلوغ الخادم أو رفضه الرمز المميز، أو تعذر فتح منفذ) |

وهناك أربع طرق للدخول:

| الطريقة | أين تعمل |
| --- | --- |
| [GitHub Action](#github-actions) | مشغّل Linux، من صورة الخادم. |
| [الصورة](#docker) | أي CI يشغّل الحاويات: GitLab وJenkins وطرفية فيها Docker. |
| [الملف التنفيذي](#binary) | أي مشغّل، بما فيه Windows. |
| [خادم المختبر](#lab-server) | تجري التشغيلات على خادم Signal Lab بجانب المعدات؛ ولا يفعل الخط سوى إرسالها. |

## GitHub Actions {#github-actions}

المستودع هو أيضًا GitHub Action. تشغّل `signallab` من الصورة `ghcr.io/proanima/signallab`، وتُفشل المهمة حين
تفشل تجربة، وتترك تقرير JUnit:

```yaml
jobs:
  signallab:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: ProAnima/SignalLab@v[[version]]
        with:
          version: [[version]]
          experiments: tests/signallab/*.json
          params: |
            api=http://127.0.0.1:8080
        env:
          SIGNALLAB_SECRET_API_TOKEN: ${{ secrets.API_TOKEN }}

      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: signallab-junit
          path: signallab-junit.xml
```

والتشغيل الذي لا ينجح هو أيضًا تعليق خطأ توضيحي على صفحة التشغيل، بجانب الخطوة الفاشلة، مع التجربة وسبب
فشلها.

### المدخلات {#action-inputs}

| المدخل | ما يفعله | الافتراضي |
| --- | --- | --- |
| `experiments` | ملفات تجارب أو أسماء قوالب مضمَّنة، تفصل بينها مسافات أو أسطر. تُوسَّع الأنماط مثل `tests/*.json`؛ والمسار الذي فيه مسافات غير مدعوم. | مطلوب |
| `params` | قيم المعاملات، `NAME=VALUE`، واحدة في كل سطر. | |
| `profile` | التشغيل بهذا الملف من ملفات تعريف التجارب. | |
| `matrix` | تشغيل مرة لكل تركيبة: `NAME=V1,V2`، اسم واحد في كل سطر. | |
| `matrix-file` | تركيبات من ملف JSON (انظر [مصفوفة من التشغيلات](#matrix)). | |
| `fail-fast` | `"true"`: التوقف عند أول تشغيل لا ينجح. | `"false"` |
| `server` | التشغيل على خادم Signal Lab هذا بدل المهمة نفسها، مثل `http://192.0.2.10:1430`. | |
| `token` | رمز وصول الخادم المميز. يُمرَّر سر. | |
| `junit` | موضع تقرير JUnit. | `signallab-junit.xml` |
| `timeout` | الثواني التي يجوز أن يستغرقها تشغيل، من 1 إلى 300. | `300` |
| `version` | وسم الصورة. | `latest` |
| `image` | سجل آخر أو صورة مبنية محليًّا؛ و`version` هو وسمها. | `ghcr.io/proanima/signallab` |
| `lang` | لغة الرسائل والتقرير: `en` أو `ru` أو `es` أو `fr` أو `de` أو `pt` أو `zh` أو `ja` أو `ko` أو `hi` أو `ar`. | `en` |
| `fail-on-error` | `"false"`: عدم إفشال الخطوة؛ وتُقرأ `exit-code` بدلًا من ذلك. | `"true"` |

المسارات (`experiments` و`matrix-file` و`junit`) نسبية إلى مجلد عمل الخطوة.

::: tip
يُثبَّت `version` على الإصدار الذي اختُبر به. و`latest` ينتقل مع كل إصدار مستقر جديد.
:::

### المخرجات {#action-outputs}

| المخرج | ما هو |
| --- | --- |
| `junit` | مسار تقرير JUnit. |
| `exit-code` | رمز خروج `signallab`: `0` أو `1` أو `2` أو `3`. |

### المتابعة بعد الفشل {#fail-on-error}

الخطوة التي تفشل لا تسلّم أي مخرجات إلى بقية المهمة. وللقرار بنفسك يُعيَّن `fail-on-error: "false"` ويُتفرع على
`exit-code`:

```yaml
      - id: lab
        uses: ProAnima/SignalLab@v[[version]]
        with:
          experiments: tests/signallab/smoke.json
          fail-on-error: "false"

      - if: steps.lab.outputs.exit-code == '1'
        run: echo "an experiment failed; the report is ${{ steps.lab.outputs.junit }}"

      - if: steps.lab.outputs.exit-code != '0'
        run: exit 1
```

### الأسرار في الـ Action {#action-secrets}

`{{secret.NAME}}` في تجربة يقرأ `SIGNALLAB_SECRET_NAME`. وتُعيَّن هذه المتغيرات على الخطوة (`env:`)، من أسرار
المستودع. تسلّم الـ Action كل متغير `SIGNALLAB_SECRET_…` في خطوتها إلى `signallab` بالاسم؛ وتنتقل القيم في
البيئة، لا في سطر أوامر أبدًا، ويعرض كل تقرير وكل سطر سجل `••••` مكانها. والسر الذي تحتاج إليه التجربة ولم
يُعيَّن يُفشل الخطوة برمز الخروج `2`، قبل إرسال أي شيء.

ويسافر المدخل `token` بالطريقة نفسها، بوصفه `SIGNALLAB_TOKEN`.

### كيف تعمل الـ Action {#action-runs}

- تحتاج إلى **مشغّل Linux** فيه Docker (`ubuntu-latest` فيه ذلك). وعلى مشغّل Windows أو macOS تتوقف برمز
  الخروج `2` وتعليق توضيحي؛ وهناك يُستخدم [الملف التنفيذي](#binary).
- يعمل `signallab` في الصورة مع **شبكة المضيف**: فيبلغ ما يبلغه المشغّل. والخدمة التي بدأتها المهمة على
  المشغّل — النظام قيد الاختبار، أو حاوية في `services:` ذات منفذ منشور — تكون على `127.0.0.1`.
- يعمل بمستخدم المشغّل، مع تركيب مساحة العمل في المسار نفسه، فيكون التقرير ملكًا للمهمة.
- تمرر إلى `run` المدخلات أعلاه بالضبط، و`--junit`. ولأي شيء آخر يستطيعه `signallab` — `--report` و`--seed`
  و`--json` و`emulate` — تُستخدم [الصورة](#docker) مباشرة.

## الصورة، في أي CI {#docker}

تحمل صورة الخادم `signallab` بوصفه `/usr/local/bin/signallab`. وتُستبدل نقطة الدخول لاستخدامه.

**GitLab CI:**

```yaml
signallab:
  image:
    name: ghcr.io/proanima/signallab:[[version]]
    entrypoint: [""]
  script:
    - signallab run tests/signallab/*.json --junit signallab-junit.xml
  artifacts:
    when: always
    reports:
      junit: signallab-junit.xml
```

الأسرار متغيرات CI/CD تُسمى `SIGNALLAB_SECRET_<NAME>` (وتُعلَّم بأنها محجوبة)؛ وتسلّمها بيئة المهمة إلى
`signallab` كما هي.

**Docker، من طرفية أو أي مجدوِل** (cron أو Jenkins أو سكربت نشر):

```bash
docker run --rm --network host \
  --user "$(id -u):$(id -g)" \
  -v "$PWD:/work" -w /work \
  -e SIGNALLAB_SECRET_API_TOKEN \
  --entrypoint signallab \
  ghcr.io/proanima/signallab:[[version]] \
  run tests/smoke.json --junit junit.xml
echo "signallab exited with $?"
```

- `--network host` يتيح للتشغيلات بلوغ ما يبلغه المضيف، بما فيه البث والبث المتعدد — على مضيف Linux. ودونه
  لا تبلغ الحاوية المضيفات الأخرى إلا بالبث الأحادي.
- تعمل الصورة بمستخدم بلا امتيازات (uid 10001). و`--user` يشغّلها بك أنت بدلًا من ذلك، فتستطيع كتابة التقرير
  في مجلدك؛ ودونه يجب أن يكون المجلد قابلًا للكتابة لـ uid 10001.
- `-e NAME` بلا قيمة يمرر ذلك المتغير من بيئتك.

## الملف التنفيذي {#binary}

يحمل كل إصدار `signallab` وحده: `signallab-<version>-linux-x64.tar.gz` و`signallab-<version>-windows-x64.zip`،
مسرودين في `SHA256SUMS.txt` الخاص بالإصدار. وملف Linux مبني على Ubuntu 22.04 ويستخدم OpenSSL 3 الخاص بالنظام
(`libssl3`): فيعمل على ذلك الإصدار أو على توزيعة أحدث.

```yaml
      - name: Signal Lab
        run: |
          curl -fsSL -o signallab.tar.gz https://github.com/ProAnima/SignalLab/releases/download/v[[version]]/signallab-[[version]]-linux-x64.tar.gz
          tar -xzf signallab.tar.gz
          ./signallab run tests/signallab/smoke.json --junit signallab-junit.xml
```

وعلى مشغّل Windows يُفك الضغط عن ملف zip ويُشغَّل `signallab.exe` بالطريقة نفسها. والجهاز المثبَّت عليه
التطبيق المكتبي فيه `signallab` على `PATH` أصلًا.

## التشغيلات على خادم مختبر {#lab-server}

المعدات على شبكة تركيب ما تُبلَغ من المختبر، لا من مشغّل سحابي. يُشغَّل هناك [خادم Signal Lab](../server/index.md)،
ويُحفظ رمزه المميز سرًّا في CI، وتُرسل التشغيلات إليه:

```bash
signallab run tests/stage.json --server http://192.0.2.10:1430 --token-file token.txt --junit junit.xml --report reports/
```

```yaml
      - uses: ProAnima/SignalLab@v[[version]]
        with:
          experiments: tests/signallab/stage.json
          server: http://192.0.2.10:1430
          token: ${{ secrets.SIGNALLAB_TOKEN }}
```

تأتي التجربة من نسخة المستودع في الخط؛ ويشغّلها الخادم بشبكته وأسراره ومجلد بياناته، ويبث الخطوات عائدًا
ويحتفظ بالتقرير (و`--report` ينزّل نسخة). ويأتي الرمز المميز من `--token-file` أو `SIGNALLAB_TOKEN`. ورموز
الخروج نفسها؛ والخادم الذي يتعذر بلوغه أو يرفض الرمز هو `3`. ويجب أن يستطيع المشغّل بلوغ الخادم — مشغّل
مستضاف ذاتيًّا في المختبر، أو عنوان خادم يستطيع المشغّل فتحه.

وللتشغيل على الخادم دون `signallab` أصلًا يمكن لسكربت أن يستدعي واجهة HTTP API الخاصة به مباشرة: انظر
[تشغيل تجربة عبر HTTP](../api/run.md).

## مصفوفة من التشغيلات {#matrix}

تجربة واحدة، وكل هدف: كل تركيبة تشغيل قائم بذاته ومجموعة اختبار قائمة بذاتها في تقرير JUnit، تُسمى بقيمها.

```bash
signallab run tests/smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest
```

وفي الـ Action، اسم واحد في كل سطر:

```yaml
        with:
          experiments: tests/signallab/smoke.json
          matrix: |
            device=192.0.2.20:9000,192.0.2.21:9000
            user=admin,guest
          fail-fast: "true"
```

أو من ملف، بـ `--matrix-file` (`matrix-file` في الـ Action):

```json
[
  { "device": "192.0.2.20:9000", "user": "admin" },
  { "device": "192.0.2.21:9000", "user": "guest" }
]
```

تُفحص كل تركيبة قبل أن ترسل الأولى أي شيء، ويأتي 256 تشغيلًا على الأكثر من أمر واحد، و`--fail-fast` يترك
الباقي دون بدء — ويظهر في تقرير JUnit على أنه متخطًّى. والقواعد في [صفحة سطر الأوامر](cli.md#matrix).

ولتجربة كل ملف تعريف من ملفات تجربة يُشغَّل مرة لكل ملف — خطوة لكل منها، أو مصفوفة مهام في CI — بـ
`--profile`.

## تقارير JUnit {#junit}

`--junit PATH` (والـ Action تكتب واحدًا دائمًا) يحوي مجموعة اختبار لكل تشغيل وحالة اختبار لكل عقدة: الفشل حيث
حدث، باللغة المختارة، مع رمز خطئه وتفصيله التقني؛ والعقد التي لم يبلغها التشغيل متخطَّاة؛ والبذرة والنتيجة
والملف وقيم المصفوفة خصائص. ويعرضه GitHub (مع Action للتقارير) وGitLab (`artifacts:reports:junit`) وJenkins
وAzure DevOps نتائج اختبار. وبنيته في [صفحة سطر الأوامر](cli.md#reports).

وبذرة التشغيل الفاشل في خصائص مجموعته وفي السجل: و`--seed <that number>` يشغّله من جديد بالقيم العشوائية
نفسها.

## التبعيات التي يستدعيها النظام قيد الاختبار {#emulators}

لاختبار نظامك ضد واجهة API أو جهاز أو وسيط غير موجود في CI، يُترك Signal Lab يؤدي دوره:

- **داخل تجربة**، تؤدي عقدة [[ui:exp.node.emulator]] دور التبعية في تشغيل واحد، وتفحص
  [[ui:exp.node.wait_http]] ما أرسله إليها نظامك. وينتهي خرج التشغيل بما سُئل عنه كل محاكٍ. انظر
  [المحاكيات](../tools/emulators.md).
- **حول اختباراتك أنت**، يجيب `signallab emulate` في الخلفية أثناء عملها، وتخبرك عدّاداته بما استُدعي:

  ```bash
  signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
  npm test
  wait
  ```

## مخرجات لسكربت {#json}

`--json` يطبع كائن JSON واحدًا في كل سطر على stdout ولا شيء غيره: `started` وكل `step` و`ended` بالنتيجة كاملة،
و`summary` مع `total` و`passed` و`failed` و`not_started` و`exit_code`. وتحمل الأخطاء `code` المحرك الثابت، فيمكن
للسكربت أن يتفرع عليه بأي لغة. انظر [المخرجات](cli.md#output).

```bash
signallab run tests/smoke.json --json | jq -c 'select(.type == "ended") | {file, outcome, seed}'
```

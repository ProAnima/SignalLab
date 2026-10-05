---
title: CI पाइपलाइन
description: GitHub Actions, GitLab CI या किसी भी पाइपलाइन में Signal Lab प्रयोग चलाएँ, कोई विफल हो तो जॉब विफल करें, और एक JUnit रिपोर्ट रखें।
---

# CI में Signal Lab

जो प्रयोग किसी इंस्टॉलेशन को चालू करते हैं वे ही उसके रिग्रेशन टेस्ट भी हैं। किसी पाइपलाइन
में, [`signallab run`](cli.md#cli-run) उन्हें बिना विंडो के चलाता है, हर चरण छापता है, एक
JUnit रिपोर्ट लिखता है जिसे हर CI सिस्टम दिखाता है, और एक ऐसे कोड के साथ निकलता है जिसे जॉब
समझता है:

| निकास कोड | पाइपलाइन को क्या करना चाहिए |
| --- | --- |
| `0` | जारी रखें: हर प्रयोग पास हुआ |
| `1` | विफल करें: एक प्रयोग चला और विफल हुआ |
| `2` | विफल करें: कोई प्रयोग या कमांड ग़लत है (कोई सत्यापन त्रुटि, कोई अज्ञात पैरामीटर, कोई गुम सीक्रेट) |
| `3` | विफल करें या फिर प्रयास करें: कुछ नहीं चल सका (सर्वर तक पहुँचा नहीं जा सकता या टोकन अस्वीकार करता है, कोई पोर्ट खोला नहीं जा सकता) |

अंदर आने के चार तरीके हैं:

| तरीका | यह कहाँ चलता है |
| --- | --- |
| [GitHub Action](#github-actions) | एक Linux रनर, सर्वर इमेज से। |
| [इमेज](#docker) | कोई भी CI जो कंटेनर चलाता है: GitLab, Jenkins, Docker वाला कोई शेल। |
| [बाइनरी](#binary) | कोई भी रनर, Windows सहित। |
| [लैब सर्वर](#lab-server) | रन उपकरण के पास एक Signal Lab सर्वर पर होते हैं; पाइपलाइन केवल उन्हें भेजती है। |

## GitHub Actions {#github-actions}

यह रिपॉज़िटरी एक GitHub Action भी है। यह `ghcr.io/proanima/signallab` इमेज से `signallab`
चलाती है, जब कोई प्रयोग विफल होता है तो जॉब विफल करती है, और एक JUnit रिपोर्ट छोड़ती है:

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

जो रन पास नहीं होता वह रन के पेज पर विफल चरण के बगल में एक error एनोटेशन भी होता है, प्रयोग
और उसके विफल होने के कारण के साथ।

### इनपुट {#action-inputs}

| इनपुट | यह क्या करता है | डिफ़ॉल्ट |
| --- | --- | --- |
| `experiments` | प्रयोग फ़ाइलें या साथ आने वाले टेम्पलेट के नाम, जगहों या पंक्तियों से अलग किए हुए। `tests/*.json` जैसे पैटर्न फैलाए जाते हैं; जगहों वाला पथ समर्थित नहीं है। | ज़रूरी |
| `params` | पैरामीटर मान, `NAME=VALUE`, प्रति पंक्ति एक। | |
| `profile` | प्रयोगों के इस प्रोफ़ाइल के साथ चलाएँ। | |
| `matrix` | प्रति संयोजन एक बार चलाएँ: `NAME=V1,V2`, प्रति पंक्ति एक नाम। | |
| `matrix-file` | JSON फ़ाइल से संयोजन (देखें [रन का मैट्रिक्स](#matrix))। | |
| `fail-fast` | `"true"`: पहले ऐसे रन पर रुकें जो पास नहीं होता। | `"false"` |
| `server` | इस जॉब के बजाय इस Signal Lab सर्वर पर चलाएँ, जैसे `http://192.0.2.10:1430`। | |
| `token` | सर्वर का एक्सेस टोकन। एक सीक्रेट दें। | |
| `junit` | JUnit रिपोर्ट कहाँ जाती है। | `signallab-junit.xml` |
| `timeout` | एक रन में लग सकने वाले सेकंड, 1 से 300। | `300` |
| `version` | इमेज का टैग। | `latest` |
| `image` | कोई और रजिस्ट्री या स्थानीय रूप से बनी इमेज; `version` उसका टैग है। | `ghcr.io/proanima/signallab` |
| `lang` | संदेशों और रिपोर्ट की भाषा: `en`, `ru`, `es`, `fr`, `de`, `pt`, `zh`, `ja`, `ko`, `hi` या `ar`। | `en` |
| `fail-on-error` | `"false"`: चरण को विफल न करें; इसके बजाय `exit-code` पढ़ें। | `"true"` |

पथ (`experiments`, `matrix-file`, `junit`) चरण की कार्यशील निर्देशिका के सापेक्ष हैं।

::: tip
`version` को उस रिलीज़ पर पिन करें जिसके साथ आपने जाँचा। `latest` हर नई स्थिर रिलीज़ पर चला
जाता है।
:::

### आउटपुट {#action-outputs}

| आउटपुट | यह क्या है |
| --- | --- |
| `junit` | JUnit रिपोर्ट का पथ। |
| `exit-code` | `signallab` का निकास कोड: `0`, `1`, `2` या `3`। |

### विफलता के बाद आगे बढ़ना {#fail-on-error}

विफल होने वाला चरण बाकी जॉब को कोई आउटपुट नहीं देता। स्वयं तय करने के लिए,
`fail-on-error: "false"` सेट करें और `exit-code` पर शाखा बनाएँ:

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

### Action में सीक्रेट {#action-secrets}

किसी प्रयोग का `{{secret.NAME}}` `SIGNALLAB_SECRET_NAME` पढ़ता है। वे चर चरण पर (`env:`) सेट
करें, रिपॉज़िटरी के सीक्रेट से। यह Action अपने चरण के हर `SIGNALLAB_SECRET_…` चर को नाम से
`signallab` को सौंपती है; मान एनवायरनमेंट में चलते हैं, कभी कमांड लाइन पर नहीं, और हर रिपोर्ट
और लॉग पंक्ति उनकी जगह `••••` दिखाती है। प्रयोग को जो सीक्रेट चाहिए और सेट नहीं है, वह कुछ भी
भेजे जाने से पहले चरण को निकास कोड `2` के साथ विफल कर देता है।

`token` इनपुट उसी तरह चलता है, `SIGNALLAB_TOKEN` के रूप में।

### Action कैसे चलती है {#action-runs}

- इसे Docker के साथ एक **Linux रनर** चाहिए (`ubuntu-latest` में वह है)। Windows या macOS रनर
  पर यह निकास कोड `2` और एक एनोटेशन के साथ रुक जाती है; वहाँ [बाइनरी](#binary) उपयोग करें।
- `signallab` इमेज में **host networking** के साथ चलता है: रनर जहाँ पहुँचता है, यह भी पहुँचता
  है। जो सेवा जॉब ने रनर पर शुरू की — आपका सिस्टम अंडर टेस्ट, या प्रकाशित पोर्ट वाला कोई
  `services:` कंटेनर — `127.0.0.1` पर है।
- यह रनर के उपयोगकर्ता के रूप में चलती है, वर्कस्पेस उसी पथ पर माउंट होता है, इसलिए रिपोर्ट
  जॉब की होती है।
- यह `run` को ठीक ऊपर के इनपुट, और `--junit` देती है। `signallab` जो और कुछ कर सकता है —
  `--report`, `--seed`, `--json`, `emulate` — उसके लिए सीधे [इमेज](#docker) उपयोग करें।

## इमेज, किसी भी CI में {#docker}

सर्वर इमेज में `signallab` `/usr/local/bin/signallab` के रूप में होता है। इसका उपयोग करने के
लिए entrypoint बदलें।

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

सीक्रेट `SIGNALLAB_SECRET_<NAME>` नाम के CI/CD चर होते हैं (उन्हें masked चिह्नित करें); जॉब
का एनवायरनमेंट उन्हें जैसे हैं वैसे `signallab` को सौंपता है।

**Docker, किसी शेल या किसी शेड्यूलर से** (cron, Jenkins, कोई डिप्लॉय स्क्रिप्ट):

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

- `--network host` रन को वह पहुँचने देता है जहाँ होस्ट पहुँचता है, broadcast और multicast सहित
  — Linux होस्ट पर। इसके बिना कंटेनर दूसरे होस्ट तक केवल unicast से पहुँचता है।
- इमेज एक अपरिविलेज्ड उपयोगकर्ता (uid 10001) के रूप में चलती है। `--user` इसके बजाय इसे आपके
  रूप में चलाता है, ताकि यह रिपोर्ट आपके फ़ोल्डर में लिख सके; इसके बिना फ़ोल्डर uid 10001 के
  लिए लिखने योग्य होना चाहिए।
- मान के बिना `-e NAME` उस चर को आपके एनवायरनमेंट से पास करता है।

## बाइनरी {#binary}

हर रिलीज़ में `signallab` अकेले भी होता है: `signallab-<version>-linux-x64.tar.gz` और
`signallab-<version>-windows-x64.zip`, रिलीज़ की `SHA256SUMS.txt` में सूचीबद्ध। Linux वाला
Ubuntu 22.04 पर बनाया जाता है और सिस्टम के OpenSSL 3 (`libssl3`) का उपयोग करता है: यह उस रिलीज़
या नए वितरण पर चलता है।

```yaml
      - name: Signal Lab
        run: |
          curl -fsSL -o signallab.tar.gz https://github.com/ProAnima/SignalLab/releases/download/v[[version]]/signallab-[[version]]-linux-x64.tar.gz
          tar -xzf signallab.tar.gz
          ./signallab run tests/signallab/smoke.json --junit signallab-junit.xml
```

Windows रनर पर, zip खोलें और `signallab.exe` उसी तरह चलाएँ। जिस मशीन पर डेस्कटॉप ऐप इंस्टॉल है
उसके `PATH` पर `signallab` पहले से है।

## लैब सर्वर पर रन {#lab-server}

किसी इंस्टॉलेशन के नेटवर्क पर मौजूद उपकरण लैब से पहुँच में हैं, किसी क्लाउड रनर से नहीं। वहाँ
एक [Signal Lab सर्वर](../server/index.md) चलाएँ, उसका टोकन एक CI सीक्रेट के रूप में रखें, और
रन उसी को भेजें:

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

प्रयोग पाइपलाइन के checkout से आता है; सर्वर उसे अपने नेटवर्क, सीक्रेट और डेटा फ़ोल्डर के साथ
चलाता है, चरण वापस स्ट्रीम करता है और रिपोर्ट रखता है (`--report` एक प्रति डाउनलोड करता है)।
टोकन `--token-file` या `SIGNALLAB_TOKEN` से आता है। निकास कोड वही हैं; जिस सर्वर तक पहुँचा न
जाए या जो टोकन अस्वीकार करे वह `3` है। रनर को सर्वर तक पहुँच सकना चाहिए — लैब में एक
स्व-होस्टेड रनर, या कोई ऐसा सर्वर पता जिसे रनर खोल सके।

`signallab` के बिना ही सर्वर पर चलाने के लिए, कोई स्क्रिप्ट सीधे उसका HTTP API कॉल कर सकती है:
देखें [HTTP के ज़रिए प्रयोग चलाना](../api/run.md)।

## रन का मैट्रिक्स {#matrix}

एक प्रयोग, हर लक्ष्य: हर संयोजन अपना रन है और JUnit रिपोर्ट में अपना टेस्ट सूट, जिसका नाम
उसके मानों पर है।

```bash
signallab run tests/smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest
```

Action में, प्रति पंक्ति एक नाम:

```yaml
        with:
          experiments: tests/signallab/smoke.json
          matrix: |
            device=192.0.2.20:9000,192.0.2.21:9000
            user=admin,guest
          fail-fast: "true"
```

या किसी फ़ाइल से, `--matrix-file` के साथ (Action में `matrix-file`):

```json
[
  { "device": "192.0.2.20:9000", "user": "admin" },
  { "device": "192.0.2.21:9000", "user": "guest" }
]
```

हर संयोजन पहले के कुछ भी भेजने से पहले जाँचा जाता है, एक कमांड से अधिकतम 256 रन आते हैं, और
`--fail-fast` बाकी को बिना शुरू किए छोड़ देता है — वे JUnit रिपोर्ट में skipped के रूप में
दिखते हैं। नियम [कमांड लाइन के पेज](cli.md#matrix) पर हैं।

किसी प्रयोग का हर प्रोफ़ाइल आज़माने के लिए, `--profile` के साथ हर प्रोफ़ाइल के लिए एक बार
चलाएँ — हर एक के लिए एक चरण, या अपने CI का जॉब मैट्रिक्स।

## JUnit रिपोर्ट {#junit}

`--junit PATH` (Action हमेशा एक लिखती है) प्रति रन एक टेस्ट सूट और प्रति नोड एक टेस्ट केस
रखता है: विफलता वहीं जहाँ हुई, चुनी गई भाषा में, उसके त्रुटि कोड और तकनीकी विवरण के साथ; जिन
नोड तक रन कभी नहीं पहुँचा उन्हें skipped के रूप में; सीड, परिणाम, फ़ाइल और मैट्रिक्स मान
प्रॉपर्टी के रूप में। GitHub (किसी रिपोर्टिंग Action के साथ), GitLab
(`artifacts:reports:junit`), Jenkins और Azure DevOps इसे टेस्ट परिणामों के रूप में दिखाते
हैं। इसकी संरचना [कमांड लाइन के पेज](cli.md#reports) पर है।

विफल रन का सीड उसके सूट की प्रॉपर्टी में और लॉग में होता है: `--seed <that number>` उसे उन्हीं
रैंडम मानों के साथ फिर चलाता है।

## सिस्टम अंडर टेस्ट जो डिपेंडेंसी कॉल करता है {#emulators}

अपने सिस्टम को ऐसे API, डिवाइस या ब्रोकर के सामने जाँचने के लिए जो CI में मौजूद नहीं है,
Signal Lab को वह भूमिका निभाने दें:

- **किसी प्रयोग के अंदर**, एक [[ui:exp.node.emulator]] नोड एक रन के लिए वह डिपेंडेंसी निभाता
  है, और [[ui:exp.node.wait_http]] जाँचता है कि आपके सिस्टम ने उसे क्या भेजा। रन का आउटपुट इस
  बात पर समाप्त होता है कि हर एमुलेटर से क्या माँगा गया। देखें
  [एमुलेटर](../tools/emulators.md)।
- **अपने टेस्ट के आसपास**, `signallab emulate` उनके चलते समय बैकग्राउंड में जवाब देता है, और
  उसकी गिनतियाँ बताती हैं कि क्या कॉल किया गया:

  ```bash
  signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
  npm test
  wait
  ```

## स्क्रिप्ट के लिए आउटपुट {#json}

`--json` stdout पर प्रति पंक्ति एक JSON ऑब्जेक्ट और कुछ नहीं छापता: `started`, हर `step`,
पूरे परिणाम के साथ `ended`, और `total`, `passed`, `failed`, `not_started` और `exit_code` वाला
एक `summary`। त्रुटियाँ इंजन का स्थिर `code` रखती हैं, इसलिए कोई स्क्रिप्ट किसी भी भाषा में उस
पर शाखा बना सकती है। देखें [आउटपुट](cli.md#output)।

```bash
signallab run tests/smoke.json --json | jq -c 'select(.type == "ended") | {file, outcome, seed}'
```


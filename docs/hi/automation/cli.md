---
title: कमांड लाइन
description: signallab बिना विंडो के प्रयोग चलाता है, अकेले संदेश भेजता है, एमुलेटर चलाता है और नेटवर्क जाँचता है — टर्मिनल, स्क्रिप्ट या CI जॉब से।
---

# कमांड लाइन: `signallab`

`signallab` बिना विंडो के Signal Lab है। यह प्रयोगों को उनके अंत तक चलाता है और
एक ऐसे कोड के साथ बाहर निकलता है जिसे स्क्रिप्ट समझती है, एक OSC संदेश, डेटाग्राम, HTTP
रिक्वेस्ट, WebSocket संदेश या MQTT पब्लिश भेजता है, आपकी लाइब्रेरी से एक सिग्नल फायर करता है,
किसी एमुलेटर को तब तक चलाता है जब तक आप उसे रोकें, और बताता है कि इस मशीन और उपकरण के बीच
क्या है।

यह ऐप के समान ही इंजन है: कमांड लाइन से एक रन वही चरण लेता है, वही रिपोर्ट लिखता है और वही
बातें कहता है — इंटरफ़ेस की भाषाओं में। `--server` के साथ रन इसके बजाय एक
[Signal Lab सर्वर](../server/index.md) पर होते हैं, उसके नेटवर्क और उसके सीक्रेट के साथ।

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

पाइपलाइन के लिए, देखें [CI में Signal Lab](ci.md); किसी AI असिस्टेंट के लिए,
[`signallab mcp`](mcp.md)।

## इंस्टॉल करना {#install}

| कहाँ | `signallab` कैसे मिलता है |
| --- | --- |
| Windows, सेटअप (`.exe`) | ऐप के बगल में इंस्टॉल होता है, और वह फ़ोल्डर `PATH` में जोड़ दिया जाता है — "मेरे लिए" इंस्टॉल पर आपका, "सबके लिए" पर मशीन का। इंस्टॉल करने के बाद एक नया टर्मिनल खोलें। सेटअप का `/NOPATH` स्विच `PATH` को छेड़ता नहीं। |
| Windows, `.msi` | ऐप के बगल में इंस्टॉल होता है; जब तक ऐप इंस्टॉल है, इंस्टॉल फ़ोल्डर मशीन के `PATH` पर रहता है। |
| Linux, `.deb` और `.rpm` | `/usr/bin/signallab`। |
| Linux, AppImage | शामिल नहीं: नीचे दिया संग्रह उपयोग करें। |
| कोई भी मशीन, ऐप के बिना | हर रिलीज़ में `signallab-<version>-windows-x64.zip` और `signallab-<version>-linux-x64.tar.gz`, हर एक में प्रोग्राम और उसका लाइसेंस; उसी पेज पर `SHA256SUMS.txt` उनके चेकसम गिनाती है। |
| सर्वर इमेज | `ghcr.io/proanima/signallab` में `/usr/local/bin/signallab` (देखें [CI](ci.md#docker))। |

इसे इससे जाँचें:

```bash
signallab version
```

## कमांड {#commands}

| कमांड | यह क्या करता है |
| --- | --- |
| [`run`](#cli-run) | प्रयोगों को एक के बाद एक चलाता है; केवल तभी 0 के साथ निकलता है जब हर एक पास हुआ हो। |
| [`validate`](#cli-validate) | प्रयोगों को वैसे जाँचता है जैसे एडिटर रन से पहले जाँचता है; कुछ नहीं भेजता। |
| [`send`](#cli-send) | एक संदेश भेजता है: `osc`, `udp`, `http`, `ws` या `mqtt`। |
| [`fire`](#cli-fire) | सिग्नल लाइब्रेरी का कोई सिग्नल, उसकी id या नाम से भेजता है। |
| [`emulate`](#cli-emulate) | एक HTTP API, OSC, UDP या TCP डिवाइस, या MQTT ब्रोकर को <kbd>Ctrl</kbd>+<kbd>C</kbd> या `--for` तक चलाता है। |
| [`emulators`](#cli-emulators) | ऐप की लाइब्रेरी के एमुलेटर सूचीबद्ध करता है। |
| [`templates`](#cli-templates) | साथ आने वाले प्रयोग टेम्पलेट सूचीबद्ध करता है। |
| [`nodes`](#cli-nodes) | JSON के रूप में हर तरह का नोड बताता है। |
| [`mcp`](#cli-mcp) | Model Context Protocol के ज़रिए Signal Lab को किसी AI असिस्टेंट तक पहुँचाता है। |
| [`doctor`](#cli-doctor) | फ़ायरवॉल, नेटवर्क, डेटा फ़ोल्डर और एक सर्वर जाँचता है। |
| [`firewall`](#cli-firewall) | `firewall allow`: दूसरी मशीनों को Windows फ़ायरवॉल से Signal Lab तक पहुँचने देता है। |
| [`version`](#cli-version) | संस्करण छापता है। |

`signallab help <command>` या `signallab <command> --help` किसी कमांड के विकल्प छापता है।

## हर कमांड के विकल्प {#global-options}

| विकल्प | यह क्या करता है | डिफ़ॉल्ट |
| --- | --- | --- |
| `--lang <code>` | संदेशों की भाषा: `en`, `ru`, `es`, `fr`, `de`, `pt`, `zh`, `ja`, `ko`, `hi` या `ar`। | `SIGNALLAB_LANG`, वरना लोकेल, वरना `en` |
| `--json` | टेक्स्ट के बजाय stdout पर मशीनों के लिए आउटपुट (देखें [आउटपुट](#output))। | बंद |
| `-h`, `--help` | कमांड की मदद। | |
| `-V`, `--version` | संस्करण; किसी भी कमांड से पहले, `signallab --version` के रूप में। | |

`--lang` और `--json` दोनों पंक्ति में कहीं भी आ सकते हैं:
`signallab --json run smoke.json` और `signallab run smoke.json --json` एक ही हैं।

### भाषा {#language}

संदेश, चरण के टेक्स्ट, विफलताएँ और JUnit रिपोर्ट इंटरफ़ेस के अपने टेक्स्ट, बहुवचन रूप और
संख्या शैली उपयोग करते हैं। भाषा इनमें से पहली होती है:

1. `--lang`;
2. `SIGNALLAB_LANG` (`ru`, `ru-RU` और `ru_RU.UTF-8` सबका मतलब रूसी है);
3. लोकेल: `LC_ALL`, `LC_MESSAGES` और `LANG` में से जो पहला सेट हो;
4. अंग्रेज़ी।

::: tip
Windows टर्मिनल आम तौर पर लोकेल चरों में से कोई सेट नहीं करते, इसलिए `signallab` वहाँ
अंग्रेज़ी बोलता है जब तक आप `SIGNALLAB_LANG` सेट न करें या `--lang` न दें।
:::

### लोगों और मशीनों के लिए आउटपुट {#output}

`--json` के बिना परिणाम **stdout** पर जाते हैं और रास्ते में व्यक्ति जो कुछ पढ़ता है वह
**stderr** पर जाता है: रन के चरण जैसे होते हैं, कोई चीज़ क्यों विफल हुई, रिपोर्ट कहाँ है।
`signallab run … 2>/dev/null` प्रति रन एक निर्णय पंक्ति छोड़ता है।

`--json` के साथ stdout पर JSON और कुछ नहीं आता, और stderr चुप रहता है:

| कमांड | `--json` stdout पर क्या छापता है |
| --- | --- |
| `run` | प्रति पंक्ति एक ऑब्जेक्ट: `started`, प्रति चरण एक `step`, `ended` (रन का परिणाम), फिर `summary`; जो रन शुरू नहीं हो सका उसके लिए `error`। देखें [`run` क्या छापता है](#run-output)। |
| `validate` | प्रति प्रयोग, प्रति पंक्ति एक ऑब्जेक्ट: `valid`, और `profile_issues` या `error`। |
| `send`, `fire` | `{"type": "sent", "result": …}`; `send http` छापता है `{"type": "response", "response": …}`, `send ws` `{"type": "exchange", "result": …}`। |
| `emulate` | प्रति पंक्ति एक ऑब्जेक्ट: `started`, प्रति रिक्वेस्ट एक `exchange`, प्रति एमुलेटर एक `summary`; `--check` के साथ `valid`। |
| `emulators` | `{"path": …, "emulators": [{id, name, protocol, bind, rules, note}, …]}`। |
| `templates` | `[{"name": …, "experiment": …}, …]`। |
| `doctor` | एक ऑब्जेक्ट: `version`, `network`, `data_dir`, `firewall`, `server`, `problems`। |
| `version` | `{"version": "…"}`। |
| `nodes` | हमेशा JSON, `--json` के साथ या बिना। |

एक विफलता `{"type": "error", "error": {…}, "exit_code": N}` होती है। `error` इंजन की
त्रुटि है: एक स्थिर `code` (जैसे `transport.refused` या `secret.missing`), उसके `params`,
और वह `node` और `field` जिसके बारे में वह है। एक स्क्रिप्ट किसी भी भाषा में `error.code` पर
शाखा बना सकती है; हर कोड के टेक्स्ट [त्रुटि संदेश](../reference/errors.md) में सूचीबद्ध हैं।

### निकास कोड {#exit-codes}

| कोड | अर्थ |
| --- | --- |
| `0` | हर प्रयोग पास हुआ; भेजना सफल रहा; रास्ते में कुछ नहीं है। |
| `1` | एक प्रयोग चला और विफल हुआ, समय से बाहर हो गया या रोका गया; एक भेजना विफल हुआ (अस्वीकार, कोई जवाब नहीं, अप्रत्याशित स्टेटस); `doctor` ने रास्ते में कुछ पाया। |
| `2` | कमांड या दस्तावेज़ ग़लत है: कोई आर्ग्युमेंट, ऐसी फ़ाइल जो पढ़ी नहीं जा सकती, कोई सत्यापन त्रुटि, कोई अज्ञात पैरामीटर, कोई गुम सीक्रेट। |
| `3` | प्रयोग के बाहर की किसी वजह से कुछ नहीं चल सका: सर्वर तक पहुँचा नहीं जा सकता या टोकन अस्वीकार करता है, कोई पोर्ट खोला नहीं जा सकता, क्रेडेंशियल स्टोर विफल होता है। |

कई प्रयोगों के साथ सबसे गंभीर परिणाम तय करता है, इस क्रम में: `2`, फिर `3`, फिर `1`, फिर `0`।

### एनवायरनमेंट चर {#environment}

| चर | यह क्या करता है |
| --- | --- |
| `SIGNALLAB_LANG` | भाषा, जब `--lang` नहीं दिया गया हो। |
| `LC_ALL`, `LC_MESSAGES`, `LANG` | भाषा, जब ऊपर के दोनों में से कोई सेट न हो। |
| `SIGNALLAB_SERVER` | `run`, `validate`, `emulate`, `mcp` और `doctor` के लिए सर्वर, जैसे `--server` देता है। |
| `SIGNALLAB_TOKEN` | सर्वर का एक्सेस टोकन, जब कोई टोकन फ़ाइल न दी गई हो। |
| `SIGNALLAB_TOKEN_FILE` | सर्वर का टोकन रखने वाली फ़ाइल, जैसे `--token-file` देता है। |
| `SIGNALLAB_SECRET_<NAME>` | इस प्रोसेस में रन के लिए सीक्रेट `NAME` का मान (देखें [सीक्रेट](#secrets))। |
| `SIGNALLAB_DATA_DIR` | ऐप का डेटा फ़ोल्डर, जहाँ `fire`, `emulators`, `emulate`, `mcp` और `doctor` डिफ़ॉल्ट रूप से देखते हैं; वरना आपके होम फ़ोल्डर में `Documents/SignalLab`। |
| `GITHUB_ACTIONS` | जब यह `true` हो, तो जो रन पास नहीं होता वह `::error` एनोटेशन के रूप में भी छपता है, जिसे GitHub रन के पेज पर दिखाता है। |

## `run` {#cli-run}

```text
signallab run [OPTIONS] <FILE>...
```

प्रयोगों को एक के बाद एक चलाता है और केवल तभी `0` के साथ निकलता है जब हर एक पास हुआ हो।

| विकल्प | यह क्या करता है | डिफ़ॉल्ट |
| --- | --- | --- |
| `<FILE>...` | प्रयोग फ़ाइलें, या [साथ आने वाले टेम्पलेट](#cli-templates) के नाम। | ज़रूरी |
| `-p`, `--param NAME=VALUE` | इस रन के लिए एक पैरामीटर मान; और के लिए दोहराएँ। | दस्तावेज़ के मान |
| `--profile NAME` | इस प्रोफ़ाइल के साथ चलाएँ; दिया गया हर प्रयोग उसमें होना चाहिए। `""` डिफ़ॉल्ट के साथ चलाता है। | दस्तावेज़ का प्रोफ़ाइल |
| `-m`, `--matrix NAME=V1,V2` | प्रति मान एक बार चलाएँ; और नामों के लिए दोहराएँ (देखें [रन का मैट्रिक्स](#matrix))। | |
| `--matrix-file PATH` | JSON फ़ाइल से संयोजन। | |
| `--seed N` | रैंडम मानों का सीड, 0 से 9007199254740991 (2⁵³ − 1)। | दस्तावेज़ का सीड, वरना प्रति रन एक नया |
| `--timeout SECONDS` | इससे ज़्यादा समय लेने वाले रन को विफल करें, 1–300। | `300` |
| `--fail-fast` | पहले ऐसे रन पर रुकें जो पास नहीं होता; बाकी शुरू नहीं होते। | बंद |
| `--junit PATH` | वहाँ एक JUnit XML रिपोर्ट लिखें (देखें [रिपोर्ट](#reports))। | |
| `--report PATH` | रन रिपोर्ट वहाँ कॉपी करें: एक रन के लिए एक फ़ाइल, कई के लिए एक फ़ोल्डर। | |
| `--data-dir PATH` | इस प्रोसेस के रन के लिए डेटा फ़ोल्डर; उनकी रिपोर्ट वहीं रहती हैं। | एक अस्थायी फ़ोल्डर, बाहर निकलने पर हटा दिया जाता है |
| `--server URL` | इस प्रोसेस के बजाय इस सर्वर पर चलाएँ (देखें [सर्वर पर](#run-on-server))। | `SIGNALLAB_SERVER` |
| `--token-file PATH` | सर्वर का टोकन रखने वाली फ़ाइल। | `SIGNALLAB_TOKEN_FILE`, वरना `SIGNALLAB_TOKEN` |
| `--secrets files\|system` | इस प्रोसेस में सीक्रेट मान कहाँ से आते हैं। | `files` |
| `--secrets-dir PATH` | सीक्रेट फ़ाइलों का फ़ोल्डर, प्रति नाम एक। | `/run/secrets/signallab`, जब वह मौजूद हो |

`--data-dir`, `--secrets` और `--secrets-dir` इस प्रोसेस के रन के बारे में हैं; इन्हें
`--server` के साथ नहीं मिलाया जा सकता।

### फ़ाइलें और टेम्पलेट {#run-inputs}

एक `FILE` वैसा प्रयोग है जैसे ऐप उसे सहेजता और निर्यात करता है
([[ui:nav.experiment]] स्क्रीन पर [[ui:exp.exportJson]])। ऐप जो भी दस्तावेज़ संस्करण खोलता
है वह काम करता है; पुराना संस्करण पढ़ते समय अद्यतन कर दिया जाता है, जैसे ऐप उसे खोलते समय
करता है। जो नाम फ़ाइल नहीं है वह [साथ आने वाले टेम्पलेट](#cli-templates) में खोजा जाता है,
`.json` के साथ या बिना:

```bash
signallab run tests/stage-cues.json tests/api.json
signallab run osc-ping-reply --param device=192.0.2.20:9000
```

हर प्रयोग — और मैट्रिक्स का हर संयोजन — वैसे ही जाँचा जाता है जैसे एडिटर पहले के चलने से
पहले जाँचता है। टूटी हुई तीसरी फ़ाइल पहले को भी रोक देती है, कुछ भी भेजे जाने से पहले,
निकास कोड `2` के साथ।

### पैरामीटर और प्रोफ़ाइल {#run-params}

`--param NAME=VALUE` केवल इसी रन के लिए एक पैरामीटर सेट करता है; फ़ाइल नहीं बदलती। मान पहले
`=` के बाद का सब कुछ है, इसलिए `--param url=http://127.0.0.1/?q=1` काम करता है, और
`--param note=` एक ख़ाली मान सेट करता है। एक मान दिए गए हर उस प्रयोग पर लागू होता है जिसमें
वह पैरामीटर हो, और उसे उनमें से कम से कम एक का पैरामीटर होना चाहिए — ग़लत वर्तनी वाला नाम
निकास कोड `2` के साथ अस्वीकार किया जाता है।

`--profile NAME` प्रयोग के किसी एक प्रोफ़ाइल के साथ चलाता है, जैसे एडिटर में
[[ui:exp.profile]] के नीचे उसे चुनना करता है। देखें
[डेटा और टेम्पलेट](../experiments/data.md)।

### रन का मैट्रिक्स {#matrix}

एक मैट्रिक्स वही प्रयोग हर लक्ष्य, हर उपयोगकर्ता, हर पेलोड आकार के सामने चलाता है। हर
संयोजन अपना रन है, अपने परिणाम, अपनी रिपोर्ट और JUnit रिपोर्ट में अपने सूट के साथ।

```bash
signallab run smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest \
  --fail-fast --junit junit.xml
```

ये चार रन हैं: `device` सबसे धीमा बदलता है, `user` सबसे तेज़। उनके नाम फ़ाइल और उनके मानों
पर हैं: `smoke.json [device=192.0.2.20:9000, user=admin]`।

- `--matrix NAME=V1,V2` एक अक्ष जोड़ता है। वही नाम फिर देने पर उसमें मान जोड़ता है। नामों
  और मानों के आसपास की जगहें हटा दी जाती हैं; दो बार दिया मान एक बार चलता है।
- `--matrix-file PATH` दो आकृतियों में से एक में JSON पढ़ता है:

  ```json
  { "device": ["192.0.2.20:9000", "192.0.2.21:9000"], "retries": [1, 3] }
  ```

  अक्ष जोड़ता है — हर संयोजन चलता है, ये नाम `--matrix` वालों के बाद, वर्णानुक्रम में;

  ```json
  [
    { "device": "192.0.2.20:9000", "user": "admin" },
    { "device": "192.0.2.21:9000", "user": "guest" }
  ]
  ```

  संयोजनों को स्वयं सूचीबद्ध करता है, हर एक `--matrix` अक्षों के साथ मिलाकर। मान टेक्स्ट,
  संख्याएँ या `true`/`false` होते हैं; फ़ाइल में किसी मान के अंदर का अल्पविराम उसी का हिस्सा
  रहता है।
- हर मैट्रिक्स नाम दिए गए कम से कम एक प्रयोग का पैरामीटर होना चाहिए। जिस प्रयोग में उनमें से
  कोई नहीं है वह एक बार चलता है, न कि हर उस मान के लिए एक बार जिसे वह अनदेखा करेगा।
- `--param` और मैट्रिक्स दोनों से, या `--matrix` और फ़ाइल दोनों से सेट किया गया नाम अस्वीकार
  किया जाता है।
- एक कमांड से अधिकतम **256** रन; ज़्यादा होने पर कुछ भी चलने से पहले अस्वीकार कर दिया जाता
  है।

### सर्वर पर चलाना {#run-on-server}

```bash
signallab run tests/stage.json --server http://192.0.2.10:1430 --token-file token.txt
```

प्रयोग इस मशीन से आता है; सर्वर उसे अपने नेटवर्क, अपने [सीक्रेट](../server/index.md#secrets)
और अपने डेटा फ़ोल्डर के साथ चलाता है, और चरण जैसे होते हैं उन्हें वापस स्ट्रीम करता है।
रिपोर्ट सर्वर पर ही रहती है — उसका पथ छपता है — और `--report` एक प्रति डाउनलोड करता है।
सर्वर पर एक रन वहाँ उसी तरह का जॉब है जैसे उसके इंटरफ़ेस में शुरू किया गया: उसमें साइन इन
हर पेज उसे दिखाता है। अगर `signallab` रन के बीच चला जाए, तो रन सर्वर पर पूरा होता है और
अपनी रिपोर्ट रखता है।

टोकन `--token-file` (या `SIGNALLAB_TOKEN_FILE`) से, वरना `SIGNALLAB_TOKEN` से पढ़ा जाता है,
और `Authorization: Bearer` के रूप में भेजा जाता है। जिस सर्वर तक पहुँचा न जाए, जो टोकन
अस्वीकार करे या रन के बीच जवाब देना बंद कर दे, वह निकास कोड `3` है।
`signallab doctor --server URL` पते और टोकन की अपने आप जाँच करता है।

### रिपोर्ट {#reports}

हर रन वही रिपोर्ट लिखता है जो ऐप लिखता है। `--data-dir` के बिना रन एक अस्थायी फ़ोल्डर उपयोग
करते हैं जो `signallab` के बाहर निकलने पर हटा दिया जाता है, इसलिए जो चाहिए उसे रखें:

- `--report PATH` रिपोर्ट कॉपी करता है: एक रन के लिए `PATH` में ही, या कई के लिए फ़ोल्डर
  `PATH` में, `01-<file>.json`, `02-<file>.json`, … के रूप में, जिस क्रम में वे चले।
- `--data-dir PATH` हर रन की रिपोर्ट उस फ़ोल्डर में (`runs/` के नीचे) रखता है, और छापता है
  कि हर एक कहाँ है।

`--junit PATH` एक JUnit XML रिपोर्ट लिखता है, वह प्रारूप जिसे हर CI सिस्टम पढ़ता है:

- प्रति रन (मैट्रिक्स के प्रति संयोजन) एक `<testsuite>`, जिसमें फ़ाइल, सीड, परिणाम, प्रोफ़ाइल,
  रिपोर्ट का पथ और हर मैट्रिक्स मान (`param.NAME`) प्रॉपर्टी के रूप में होते हैं;
- जो नोड चला उसके लिए एक `<testcase>`, जिसका नाम नोड के प्रकार और उसकी id पर है, अपने समय
  के साथ;
- जो नोड विफल हुआ उस पर एक `<failure>`: चुनी गई भाषा में संदेश, उसके `type` के रूप में त्रुटि
  कोड, और तकनीकी विवरण के साथ नोड के चरण;
- रन जिस नोड तक कभी नहीं पहुँचा (किसी शाखा का दूसरा सिरा) उसके लिए एक `<skipped>` केस;
- जो प्रयोग शुरू नहीं हो सका उसके लिए एक `<error>` केस वाला सूट, और हर उस रन के लिए एक
  `<skipped>` केस वाला सूट जिसे `--fail-fast` ने शुरू नहीं किया।

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

[लोड](../experiments/load.md) के अधीन एक HTTP नोड अपने अंतिम चरण के नीचे हर थ्रेशोल्ड के
लिए एक पंक्ति जोड़ता है, टिका हो या नहीं (`✕ p95 < 100 ms · 152.58 ms`); जो थ्रेशोल्ड टिकता
नहीं वह रन और उसका JUnit केस विफल करता है (`type="load.threshold"`)।

### सीक्रेट {#secrets}

एक प्रयोग किसी सीक्रेट को `{{secret.NAME}}` के रूप में पढ़ता है। इस प्रोसेस में रन के लिए मान
यहाँ से आता है:

| `--secrets` | `NAME` का मान कहाँ से आता है |
| --- | --- |
| `files` (डिफ़ॉल्ट) | एनवायरनमेंट चर `SIGNALLAB_SECRET_NAME`; वरना `--secrets-dir` में `NAME` नाम की फ़ाइल (डिफ़ॉल्ट रूप से `/run/secrets/signallab`, जब वह फ़ोल्डर मौजूद हो — Docker सीक्रेट लेआउट)। |
| `system` | Windows क्रेडेंशियल मैनेजर — जहाँ ऐप अपने [[ui:exp.secrets]] के मान रखता है। Linux में कोई क्रेडेंशियल स्टोर नहीं है जिसे `signallab` पढ़ता है: वहाँ `--secrets system` निकास कोड `3` के साथ विफल होता है। |

फ़ाइल का अंतिम पंक्ति-विराम मान का हिस्सा नहीं है, और ख़ाली मान सेट न होना माना जाता है। नाम
अक्षर, अंक और `_` होते हैं, जो अंक से शुरू नहीं होते, 128 अक्षरों तक; एक मान अधिकतम 16 KiB का
होता है।

```bash
SIGNALLAB_SECRET_API_TOKEN="$API_TOKEN" signallab run tests/api.json
```

जो सीक्रेट सेट नहीं है वह किसी भी ट्रैफ़िक से पहले रन रोक देता है, निकास कोड `2` और गुम नाम के
साथ। मान कभी नहीं छपता: चरण, त्रुटियाँ, रिपोर्ट और JUnit रिपोर्ट उसकी जगह `••••` दिखाते हैं।
`--server` के साथ सीक्रेट सर्वर के होते हैं।

### `run` क्या छापता है {#run-output}

जैसे रन चलता है, हर चरण stderr पर एक पंक्ति होता है — शुरुआत से उसका समय, नोड, उसकी स्थिति
और उसने क्या किया — ऐप की टाइमलाइन की तरह। जब यह समाप्त होता है, stdout पर एक पंक्ति बताती
है कि वह कैसा रहा:

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

विफल रन अपने उपयोग किए सीड का नाम बताता है: उस संख्या के साथ `--seed` उसे उन्हीं रैंडम मानों
के साथ फिर चलाता है। निर्णय के बाद, stderr पर, यह आता है कि रन के हर एमुलेटर से क्या माँगा
गया — रिक्वेस्ट, कितनी बिना किसी नियम के, कितनी विफल, और प्रति नियम हिट:

```text
✔ Retry a flaky API passed in 7 ms
  Flaky API: 3 requests, 0 without a rule, 0 failed · #1 3
```

और हर बाधा रिले ने क्या किया: उसे क्या मिला, क्या छोड़ा और क्या थ्रॉटल किया, और हर चरण
अग्रेषित / प्राप्त के रूप में:

```text
  127.0.0.1:19110 → 127.0.0.1:19100: 155 datagrams, 38 dropped, 0 throttled · lan 0.0–2.0 s 39/39, wifi 2.0–4.0 s 39/39, offline 4.0–6.0 s 0/38, lan 6.0–8.0 s 38/38
```

TCP पर एक रिले स्ट्रीम के चंक ले जाता है, डेटाग्राम नहीं, और कुछ नहीं छोड़ता, इसलिए उसकी
पंक्ति चंक, कनेक्शन, रीसेट, हाफ-ओपन कनेक्शन और उन बारों को गिनती है जब कोई स्ट्रीम बैंडविड्थ
सीमा से रोका गया:

```text
  127.0.0.1:19120 → 127.0.0.1:19101: 14 chunks, 2 connections, 1 reset, 0 half-open, 3 held back
```

[लोड](../experiments/load.md) के अधीन एक HTTP नोड अपनी प्रगति एक चरण के रूप में अधिकतम एक
बार प्रति सेकंड बताता है।

कई रनों के साथ, प्रति रन एक पंक्ति और एक गिनती आउटपुट समाप्त करती है:
`3 runs: 2 passed, 1 failed`। `--fail-fast` के साथ, जो रन उसने शुरू नहीं किए वे stderr पर गिने
जाते हैं।

`--json` के साथ हर पंक्ति एक `type` वाला ऑब्जेक्ट है:

```json
{"type":"started","experiment":"Empty experiment","file":"empty","job_id":1,"overridden":false,"profile":null,"seed":1,"started_ms":1791132204585}
{"type":"step","node_id":"start","state":"passed","detail":"Started","message_key":"exp.step.started","message_params":{"seed":1},"job_id":1,"ts":1791132204585}
{"type":"ended","experiment":"Empty experiment","file":"empty","outcome":"passed","seed":1,"params":{},"steps":[…],"report_path":"…","started_ms":1791132204585,"ended_ms":1791132204585,…}
{"type":"summary","total":1,"passed":1,"failed":0,"not_started":0,"exit_code":0}
```

`started`, `ended` और `error` पंक्तियाँ `file` (जैसा आर्ग्युमेंट दिया गया) और, मैट्रिक्स
में, `matrix` (संयोजन के मान) रखती हैं। `ended` रन का पूरा परिणाम है: `outcome` (`passed`,
`failed` या `stopped`), `seed`, `profile`, `params`, `error`, हर चरण, `emulators` और
`impairments` जब रन में वे थे, और `report_path`। एक लोड का अंतिम चरण `load` रखता है जिसमें
वह हर संख्या मापता है: `planned`, `sent`, `ok`, `failed`, `missed`, `rps`, `error_rate`,
`min_ms`, `mean_ms`, `max_ms`, `p50_ms` से `p99_ms`, `statuses`, हर सेकंड (`seconds`),
`histogram`, और हर थ्रेशोल्ड का निर्णय (`thresholds`)।

## `validate` {#cli-validate}

```text
signallab validate [OPTIONS] <FILE>...
```

प्रयोगों को वैसे जाँचता है जैसे एडिटर रन से पहले जाँचता है — ग्राफ़, हर फ़ील्ड, टेम्पलेट,
पैरामीटर और सीक्रेट — और कुछ नहीं भेजता। `0` के साथ निकलता है जब हर एक शुरू हो सकता हो।

यह [`run`](#cli-run) के इनपुट लेता है: फ़ाइलें और टेम्पलेट, `--param`, `--profile`,
`--matrix`, `--matrix-file`, और `--server`, `--token-file`, `--secrets`, `--secrets-dir`।
`--server` के साथ सर्वर उन्हें अपने सीक्रेट के सामने जाँचता है।

```text
✔ tests/stage.json: Stage cues would run
  Rehearsal: Would not run: …
```

जो समस्या केवल दस्तावेज़ के किसी और प्रोफ़ाइल में है वह उसके नीचे सूचीबद्ध होती है, लेकिन
जाँच विफल नहीं करती। `--json` के साथ, प्रति प्रयोग (प्रति संयोजन) एक पंक्ति:
`{"experiment", "file", "valid": true, "profile_issues": […]}`,
या `error` और `exit_code` के साथ `"valid": false`।

## `send` {#cli-send}

```text
signallab send <osc|udp|http|ws|mqtt> …
```

ऐप की स्क्रीन जो कमांड उपयोग करती हैं उन्हीं से एक संदेश भेजता है, इसलिए तार पर वही बाइट
होते हैं। एक भेजना कोई लाइब्रेरी और कोई सीक्रेट नहीं पढ़ता।

निकास कोड: `0` भेजा गया, `1` भेजना विफल, `2` कोई आर्ग्युमेंट ग़लत।

एक `<host:port>` एक IP पता या होस्ट नाम, और एक पोर्ट है: `127.0.0.1:9000`, `[::1]:9000`
(कोष्ठकों में एक IPv6 पता) या `device.local:9000`। नाम कमांड चलने पर खोजा जाता है, उसका
IPv4 पता लिया जाता है जब वह हो, इसलिए `localhost:9000` `127.0.0.1` पर किसी रिसीवर तक
पहुँचता है। जो नाम हल न हो वह भेजना विफल करता है (`1`); जिस लक्ष्य में पोर्ट न हो वह अमान्य
आर्ग्युमेंट है (`2`)।

### `send osc` {#cli-send-osc}

```text
signallab send osc <host:port> <address> [ARG]...
```

एक OSC संदेश। आर्ग्युमेंट्स एक उपसर्ग के साथ टाइप किए जाते हैं, या अनुमानित:

| आर्ग्युमेंट | OSC प्रकार |
| --- | --- |
| `i:3` | int32 |
| `f:0.5` | float32 |
| `d:1.5` | float64 (double) |
| `h:64` | int64 |
| `s:text` | स्ट्रिंग |
| `b:de ad be ef` | blob, हेक्स बाइट के रूप में |
| `T`, `F` | सच, झूठ |
| `N` | nil |
| `3`, `-3` | सादा पूर्णांक int32 है |
| `2.5` | सादा दशमलव float32 है |
| बाकी कुछ भी | स्ट्रिंग |

```bash
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main 3
# ✔ sent /cue/go (32 bytes) → 127.0.0.1:9000
```

जगहों वाले आर्ग्युमेंट को उद्धरण में रखें: `"s:hello world"`। `s:7` टेक्स्ट `7` भेजता है।
पता `/` से शुरू होना चाहिए; जो नहीं होता वह अमान्य आर्ग्युमेंट है (`2`)।

::: warning Git Bash on Windows
Git Bash `/` से शुरू होने वाले आर्ग्युमेंट को Windows पथ में बदल देता है, इसलिए `/cue/go`
`C:/Program Files/Git/cue/go` के रूप में पहुँचता है। `//cue/go` लिखें, या `MSYS_NO_PATHCONV=1`
के साथ चलाएँ। PowerShell और `cmd` प्रभावित नहीं होते।
:::

देखें [OSC](../protocols/osc.md)।

### `send udp` {#cli-send-udp}

```text
signallab send udp <host:port> (--text TEXT | --hex HEX)
```

एक UDP डेटाग्राम। पेलोड `--text` के रूप में दें, या `--hex` बाइट के रूप में:
`"de ad be ef"`, `deadbeef` या `0xDE,0xAD`।

```bash
signallab send udp 127.0.0.1:7000 --text "PLAY 1"
signallab send udp 127.0.0.1:7000 --hex "de ad be ef"
```

### `send http` {#cli-send-http}

```text
signallab send http <METHOD> <URL> [OPTIONS]
```

एक HTTP रिक्वेस्ट। स्टेटस पंक्ति stderr पर जाती है — `HTTP 200 OK · 3 ms · 1,234 B` — और
रिस्पॉन्स बॉडी stdout पर, ताकि उसे आगे पाइप किया जा सके। 256 KiB से लंबी बॉडी वहाँ काट दी
जाती है, और stderr यह बताता है।

| विकल्प | यह क्या करता है | डिफ़ॉल्ट |
| --- | --- | --- |
| `-H`, `--header "Name: value"` | एक रिक्वेस्ट हेडर; और के लिए दोहराएँ। | |
| `--body TEXT` | रिक्वेस्ट बॉडी। `@FILE` किसी टेक्स्ट फ़ाइल की सामग्री भेजता है। | कोई नहीं |
| `--expect-status STATUS` | `1` के साथ निकलें जब तक रिस्पॉन्स में यह स्टेटस न हो। | कोई भी स्टेटस `0` है |
| `--timeout MS` | रिस्पॉन्स की प्रतीक्षा के लिए मिलीसेकंड। | `10000` |
| `-u`, `--user NAME:PASSWORD` | क्रेडेंशियल, Basic के रूप में भेजे जाते हैं। | |
| `--digest` | `--user` के साथ: इसके बजाय सर्वर की Digest चुनौती का जवाब दें (MD5 या SHA-256)। | बंद |
| `--bearer TOKEN` | `Authorization: Bearer TOKEN` भेजें। `--user` के साथ नहीं। | |

```bash
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab send http POST http://127.0.0.1:8080/cue -H "Content-Type: application/json" --body '{"cue": 1}'
signallab send http GET http://127.0.0.1:8080/admin -u admin:secret --digest
```

`--expect-status` के बिना कोई भी रिस्पॉन्स सफलता है, `500` भी शामिल। जिस रिक्वेस्ट को कोई
रिस्पॉन्स नहीं मिलता (अस्वीकार, समय समाप्त, ऐसा नाम जो हल न हो) वह निकास कोड `1` है, और
वैसा ही वह Digest चुनौती भी जिसका जवाब न दिया जा सके — वजह रिस्पॉन्स के बाद छपती है।

::: tip
आर्ग्युमेंट उसी मशीन के दूसरे उपयोगकर्ताओं को दिखते हैं। असली पासवर्ड प्रयोगों के लिए रखें,
जहाँ वे [सीक्रेट](#secrets) हैं।
:::

देखें [HTTP](../protocols/http.md)।

### `send ws` {#cli-send-ws}

```text
signallab send ws <URL> [OPTIONS]
```

एक WebSocket आदान-प्रदान: `ws://…` या `wss://…` से कनेक्ट करें, एक संदेश भेजें, वैकल्पिक
रूप से जवाब की प्रतीक्षा करें, बंद करें। हैंडशेक और जो भेजा गया वह stderr पर, जवाब stdout पर
(बाइनरी संदेश हेक्स के रूप में)।

| विकल्प | यह क्या करता है | डिफ़ॉल्ट |
| --- | --- | --- |
| `--text TEXT` | यह टेक्स्ट संदेश भेजें। | कुछ नहीं भेजा जाता |
| `--hex HEX` | ये बाइट बाइनरी संदेश के रूप में भेजें। `--text` के साथ नहीं। | |
| `-H`, `--header "Name: value"` | अपग्रेड रिक्वेस्ट के लिए एक हेडर; और के लिए दोहराएँ। | |
| `--protocol NAME` | देने के लिए एक सबप्रोटोकॉल; और के लिए दोहराएँ, प्राथमिकता के क्रम में। | |
| `--expect TEXT` | इस टेक्स्ट वाले संदेश की प्रतीक्षा करें। | |
| `--expect-regex REGEX` | इस रेगुलर एक्सप्रेशन से मेल खाते संदेश की प्रतीक्षा करें। | |
| `--wait` | किसी भी संदेश की प्रतीक्षा करें। | |
| `--timeout MS` | जवाब की प्रतीक्षा के लिए मिलीसेकंड। | `2000` |

```bash
signallab send ws ws://127.0.0.1:9001/echo --text '{"ping": 1}' --expect '"ping"'
signallab send ws ws://127.0.0.1:9001/feed --wait        # nothing sent: the server's first message
```

`--expect`, `--expect-regex` या `--wait` के बिना यह कनेक्ट करता है, भेजता है और बिना प्रतीक्षा
किए बंद करता है। जब अपेक्षित जवाब समय पर नहीं आता, तो निकास कोड `1` होता है। देखें
[WebSocket](../protocols/websocket.md)।

### `send mqtt` {#cli-send-mqtt}

```text
signallab send mqtt <host:port> <topic> [payload] [--qos 0|1|2] [--retain]
```

plain TCP पर एक MQTT 3.1.1 पब्लिश, बिना क्रेडेंशियल के, अपने ही क्लाइंट के रूप में एक नई
क्लाइंट id के साथ, इसलिए यह पहले से मौजूद कनेक्शन को कभी नहीं गिराता। पोर्ट के बिना ब्रोकर
`1883` पर है। एक टॉपिक एक टॉपिक है: उसमें `+` या `#` हो, या वह ख़ाली हो, तो कमांड कनेक्ट
करने से पहले अस्वीकार कर दी जाती है (`2`)।

| विकल्प | यह क्या करता है | डिफ़ॉल्ट |
| --- | --- | --- |
| `[payload]` | पेलोड। | ख़ाली |
| `--qos 0\|1\|2` | सेवा की गुणवत्ता। | `0` |
| `--retain` | इसे टॉपिक के retained मान के रूप में रखें। `--retain` के साथ ख़ाली पेलोड उसे साफ़ कर देता है। | बंद |

```bash
signallab send mqtt 127.0.0.1:1883 lab/light/1/set on --qos 1
signallab send mqtt 127.0.0.1 lab/light/1/state "" --retain     # clear the retained value
```

देखें [MQTT](../protocols/mqtt.md)।

## `fire` {#cli-fire}

```text
signallab fire <signal> [--library PATH]
```

सिग्नल लाइब्रेरी का कोई सिग्नल ठीक वैसे ही भेजता है जैसे ऐप की [[ui:nav.signals]] स्क्रीन
उसे फायर करती है: OSC, UDP, HTTP और MQTT सिग्नल। सिग्नल उसकी id से मिलता है, वरना उसके नाम
से, केस की परवाह किए बिना; जो नाम कई सिग्नल साझा करते हैं वह उनकी id के साथ अस्वीकार किया
जाता है।

| विकल्प | यह क्या करता है | डिफ़ॉल्ट |
| --- | --- | --- |
| `<signal>` | सिग्नल की id या नाम। | ज़रूरी |
| `--library PATH` | लाइब्रेरी फ़ाइल। | ऐप के डेटा फ़ोल्डर में `signals.json` |

```bash
signallab fire "Fader value"
signallab fire go --library show/signals.json
```

लाइब्रेरी केवल पढ़ी जाती है, कभी बनाई या बदली नहीं जाती। देखें
[सिग्नल](../tools/signals.md)।

## `emulate` {#cli-emulate}

```text
signallab emulate [OPTIONS] <FILE|NAME>...
```

दूसरी तरफ़ चलाता है — एक HTTP API, एक OSC, UDP या TCP डिवाइस, एक MQTT ब्रोकर —
<kbd>Ctrl</kbd>+<kbd>C</kbd> या `--for` तक, और हर रिक्वेस्ट को जवाब दिए जाते समय छापता है। एक
`FILE` में एक एमुलेटर, उनकी सूची, या पूरी लाइब्रेरी होती है जैसे ऐप उसे लिखता है; एक `NAME`
ऐप की लाइब्रेरी में किसी एक की id या नाम है ([[ui:nav.emulators]] स्क्रीन की)।

| विकल्प | यह क्या करता है | डिफ़ॉल्ट |
| --- | --- | --- |
| `-p`, `--param NAME=VALUE` | एक मान जिसे उसके टेम्पलेट `{{NAME}}` के रूप में पढ़ते हैं; और के लिए दोहराएँ। | |
| `--bind IP:PORT` | इसके बजाय वहाँ सुनें। केवल एक एमुलेटर के साथ। | एमुलेटर का अपना |
| `--for SECONDS` | इतनी देर बाद रुकें। | <kbd>Ctrl</kbd>+<kbd>C</kbd> तक |
| `--seed N` | उसके रैंडम चुनावों का सीड: एक रैंडम क्रम, जिटर, जनरेटर। | |
| `--check` | एमुलेटर जाँचें और बिना पोर्ट खोले बाहर निकलें। | बंद |
| `--library PATH` | जिस लाइब्रेरी में नाम खोजे जाते हैं। | ऐप के डेटा फ़ोल्डर में `emulators.json` |
| `--server URL`, `--token-file PATH` | उन्हें एक सर्वर पर शुरू करें, उसके API के ज़रिए उन्हें फ़ॉलो करें, और अंत में उन्हें रोकें। | |
| `--secrets`, `--secrets-dir` | [`run`](#cli-run) की तरह। | |

```text
$ signallab emulate tests/orders-api.json --for 60
Orders API (http) answering on 127.0.0.1:18099
answering for 60 s
+   1.209 s Orders API  #1  GET /orders/42 → 200 OK · 12 B  1 ms  ← 127.0.0.1:55744
+   1.209 s Orders API  —  GET /nothing → 404 Not Found · 20 B  2 ms  ← 127.0.0.1:55745
Orders API: 2 requests, 1 without a rule, 0 failed · #1 1
```

हर पंक्ति शुरुआत से समय, एमुलेटर, जवाब देने वाला नियम (`#1`, या कोई न हो तो `—`), रिक्वेस्ट
और उसे क्या मिला, उसमें लगा समय और इसे किसने भेजा, है। अंत में हर एमुलेटर की गिनतियाँ:
रिक्वेस्ट, कितनी बिना किसी नियम के, कितनी विफल, कितनी किसी आउटेज से मिलीं या जब कोई थे तो
बिना पहुँचे रह गए, और प्रति नियम हिट।

निकास कोड: `0` जब यह <kbd>Ctrl</kbd>+<kbd>C</kbd> या `--for` पर रुकता है; `2` जब कोई एमुलेटर
मान्य नहीं है; `3` जब उसका पोर्ट लिया हुआ है या खोला नहीं जा सकता, या जवाब देते समय कोई सॉकेट
विफल होता है।

किसी पाइपलाइन में, इसे बैकग्राउंड में शुरू करें, इसके सामने सिस्टम की जाँच करें, और अंत में
गिनतियाँ पढ़ें:

```bash
signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
npm test          # the system under test, configured for the emulator's address
wait              # the last lines of mock.ndjson are the counts
```

सर्वर पर, `--server` के साथ शुरू किया एमुलेटर तब रुकता है जब `signallab` सामान्य रूप से समाप्त
होता है; मारे गए प्रोसेस द्वारा छोड़ा गया एक एमुलेटर सर्वर के इंटरफ़ेस से रोका जा सकता है।
देखें [एमुलेटर](../tools/emulators.md)।

## `emulators` {#cli-emulators}

```text
signallab emulators [--library PATH]
```

लाइब्रेरी के एमुलेटर सूचीबद्ध करता है: id, नाम, प्रोटोकॉल, पता और कितने नियम। `--library PATH`
ऐप के डेटा फ़ोल्डर में `emulators.json` के बजाय कोई और लाइब्रेरी फ़ाइल पढ़ता है।
`signallab emulate <id>` एक शुरू करता है।

लाइब्रेरी केवल पढ़ी जाती है। अगर ऐसी कोई फ़ाइल न हो — ऐप पहली बार शुरू होने पर अपने डेटा
फ़ोल्डर में वाली बनाता है — तो कमांड यह बताती है और `2` के साथ निकलती है।

## `templates` {#cli-templates}

```text
signallab templates
```

साथ आने वाले टेम्पलेट सूचीबद्ध करता है, जिन्हें `run` और `validate` नाम से लेते हैं। वे ऐप के
अपने हैं:

| नाम | ऐप में | पैरामीटर |
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

हर टेम्पलेट लूपबैक की ओर इशारा करता है। `flaky-api`, `fault-phases` और `dependency-outage`
अपने एमुलेटर साथ लाते हैं, इसलिए वे बिना कुछ और सुने चलते हैं — `signallab` को काम करते देखने
का एक तेज़ तरीका।

## `nodes` {#cli-nodes}

```text
signallab nodes
```

JSON के रूप में छापता है कि एक प्रयोग किन चीज़ों से बनता है: दस्तावेज़ की आकृति और नियम, हर
तरह का नोड उसके लेबल, विवरण, फ़ील्ड, आउटपुट और इंजन द्वारा स्वीकार किए जाने वाले एक उदाहरण
के साथ, `{{template}}` भाषा, लोड प्रोफ़ाइल और एमुलेटर दस्तावेज़। यही वह है जो असिस्टेंट प्रयोग
लिखने के लिए [`signallab mcp`](mcp.md) के ज़रिए पढ़ता है; लोगों के लिए,
[नोड](../experiments/nodes.md) वही ज़्यादा शब्दों में बताता है।

## `mcp` {#cli-mcp}

```text
signallab mcp [OPTIONS]
```

stdin और stdout पर Model Context Protocol के ज़रिए Signal Lab को किसी AI असिस्टेंट तक पहुँचाता
है। इसका सब कुछ — Claude Code, Claude Desktop, Cursor या VS Code में इसे सेट करना, इसके
विकल्प और इसके टूल — [असिस्टेंट (MCP)](mcp.md) पर है।

## `doctor` {#cli-doctor}

```text
signallab doctor [--server URL] [--token-file PATH]
```

बताता है कि Signal Lab और उपकरण के बीच क्या हो सकता है, और जब कुछ हो तो `1` के साथ निकलता
है। इसकी पंक्तियाँ बाकी कमांड लाइन की तरह [`--lang`](#language) का अनुसरण करती हैं:

- जिस नेटवर्क पर यह मशीन है: उसका नाम और पता;
- डेटा फ़ोल्डर: क्या उसमें लिखा जा सकता है (अभी न बनाई गई ठीक है — ऐप उसे पहले उपयोग पर
  बनाता है);
- फ़ायरवॉल: Windows पर, `signallab` और डेस्कटॉप ऐप दोनों के लिए, क्या कोई नियम दूसरी मशीनों
  को अभी जिस तरह के नेटवर्क पर यह मशीन है (निजी, डोमेन या सार्वजनिक) उस पर अंदर आने देता है,
  या कोई नियम उन्हें रोकता है — सिस्टम के प्रॉम्प्ट पर *Cancel* करने से यही रह जाता है;
  Linux पर, क्या ufw या firewalld चालू है, और वह कमांड जो एक पोर्ट खोलती है;
- `--server` (या `SIGNALLAB_SERVER`) के साथ: क्या सर्वर जवाब देता है और टोकन स्वीकार करता है।

```text
signallab [[version]]
Network: LAB-PC · 192.0.2.15
Data folder: C:\Users\lab\Documents\SignalLab — writable
Firewall · signallab (C:\…\Signal Lab\signallab.exe) · private network: not allowed yet — signallab firewall allow
Firewall · app (C:\…\Signal Lab\signal-lab.exe) · private network: allowed
✖ 1 thing in the way
```

`--json` वही एक ऑब्जेक्ट के रूप में छापता है। देखें
[समस्या निवारण](../reference/troubleshooting.md)।

## `firewall` {#cli-firewall}

```text
signallab firewall allow [--public]
```

Windows पर, दूसरी मशीनों को Signal Lab तक पहुँचने देता है — जो एक मॉनिटर या प्रतीक्षा को किसी
डिवाइस को सुनने के लिए चाहिए। Windows पहले एडमिनिस्ट्रेटर अधिकार माँगता है; फिर `signallab`
और डेस्कटॉप ऐप (उसके बगल में मिलता है, या जहाँ इंस्टॉलर उसे रखते हैं) के inbound नियमों को
निजी और डोमेन नेटवर्कों पर, ब्लॉक करने वाले नियमों सहित, प्रत्येक के लिए एक allow नियम से बदल
दिया जाता है।

| विकल्प | यह क्या करता है |
| --- | --- |
| `--public` | सार्वजनिक नेटवर्कों पर भी — किसी स्थान का Wi-Fi अक्सर वैसा होता है। |

निकास कोड: `0` हो गया; `3` जब एडमिनिस्ट्रेटर प्रॉम्प्ट अस्वीकार किया जाए या बदलाव विफल हो।
Linux पर कुछ नहीं बदला जाता: यह वह ufw या firewalld कमांड छापता है जो आपके सुने जाने वाले
पोर्ट खोलती है, और `0` के साथ निकलता है।

फ़ायरवॉल केवल तब बदलता है जब आप इसे चलाते हैं; `signallab` में और कुछ भी इसे नहीं छूता।

## `version` {#cli-version}

```text
signallab version
```

`signallab [[version]]` छापता है — `--json` के साथ, `{"version": "[[version]]"}`।
`signallab --version` वही संस्करण छापता है।


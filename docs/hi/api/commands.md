---
title: कमांड
description: Signal Lab के इंजन की हर कमांड, जिसे POST /api/invoke/<command> के रूप में बुलाया जाता है, उसके आर्ग्युमेंट, नतीजे और त्रुटियों के साथ।
---

# कमांड

इंजन की हर कमांड, जिस पर वह काम करती है उसके अनुसार समूहित। हर एक को
`POST /api/invoke/<command>` के रूप में आर्ग्युमेंट के एक JSON ऑब्जेक्ट के साथ बुलाया जाता है, और
वह `200` के साथ अपना नतीजा या `422` के साथ एक [`EngineError`](index.md#errors) के रूप में जवाब देती
है। प्रमाणीकरण कैसे करें और स्टेटस का क्या अर्थ है, यह [API का अवलोकन](index.md) में है।

## परंपराएँ {#conventions}

- **आर्ग्युमेंट के नाम** camelCase हैं (`jobId`, `nodeId`)। जिस आर्ग्युमेंट को कोई कमांड
  नहीं जानती, या जो ज़रूरी आर्ग्युमेंट छूट गया हो, उसे
  `command.args_invalid` के साथ अस्वीकार कर दिया जाता है; आर्ग्युमेंट लेने वाली हर कमांड
  इसके साथ विफल हो सकती है।
  बिना आर्ग्युमेंट वाली कमांड बॉडी नहीं पढ़ती।
- **आर्ग्युमेंट के रूप में दिए गए ऑब्जेक्ट** — `config`, `request`, `document`,
  `library`, `emulator`, `profile` — इंजन के अपने फ़ील्ड नाम उपयोग करते हैं, ज़्यादातर
  snake_case (`timeout_ms`)। इनके भीतर जिस फ़ील्ड को इंजन नहीं जानता वह
  **नज़रअंदाज़** किया जाता है, इसलिए ग़लत लिखा वैकल्पिक फ़ील्ड चुपचाप अपना डिफ़ॉल्ट बनाए रखता
  है। केवल `feedback_send` का फ़ॉर्म अज्ञात फ़ील्ड अस्वीकार करता है।
- **वैकल्पिक** आर्ग्युमेंट और फ़ील्ड छोड़े जा सकते हैं या `null` के रूप में भेजे जा सकते हैं;
  तालिकाएँ उनके डिफ़ॉल्ट देती हैं।
- **नतीजे** JSON होते हैं। "null" का अर्थ है कि कमांड के पास लौटाने के लिए कुछ नहीं है।
- **पते** जो `IP:port` लिखे होते हैं, एक संख्यात्मक पता और पोर्ट लेते हैं
  (`127.0.0.1:9000`, `[::1]:9000`); वहाँ होस्ट नाम अस्वीकार कर दिया जाता है। जहाँ कोई तालिका
  `IP:port` या `host:port` कहती है, वहाँ होस्ट नाम भी चलता है: इसे
  कमांड चलने पर खोजा जाता है, और जब इसका IPv4 पता हो तो वही लिया जाता है
  (इसलिए `localhost:9000` यानी `127.0.0.1:9000`)।
- **जॉब**: *जॉब शुरू करती है* चिह्नित कमांड एक
  [`JobInfo`](#type-jobinfo) लौटाती है; काम तब तक चलता है जब तक वह समाप्त न हो या
  [`job_stop`](#job_stop) से रोक न दिया जाए। देखें [जॉब](index.md#jobs)।
- **पथ** नतीजों में उस मशीन पर होते हैं जहाँ इंजन चलता है — सर्वर पर
  उसके डेटा फ़ोल्डर के भीतर; उन्हें [`/api/files`](index.md#files) से डाउनलोड करें।

उदाहरण यह शेल फ़ंक्शन उपयोग करते हैं:

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
invoke() {
  curl -sS -X POST "$SERVER/api/invoke/$1" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    --data "${2:-}"
}
```

## एप्लिकेशन {#application}

### app_info {#app_info}

इंजन क्या है और कहाँ चलता है। कोई आर्ग्युमेंट नहीं।

**नतीजा**

| फ़ील्ड | प्रकार | अर्थ |
| --- | --- | --- |
| `version` | स्ट्रिंग | Signal Lab का वर्शन, `[[version]]` |
| `mode` | स्ट्रिंग | `desktop` या `server` |
| `secrets_writable` | बूलियन | क्या [`secret_set`](#secret_set) और [`secret_delete`](#secret_delete) यहाँ काम कर सकती हैं: सर्वर पर `false` |
| `data_dir` | स्ट्रिंग | डेटा फ़ोल्डर, उस मशीन पर जहाँ इंजन चलता है |
| `os` | स्ट्रिंग | `windows`, `linux`… |
| `arch` | स्ट्रिंग | `x86_64`, `aarch64`… |

### get_host_info {#get_host_info}

मशीन का नाम और वह पता जिससे वह भेजती। कोई आर्ग्युमेंट नहीं।

**नतीजा**: `{ "local_ip": string, "hostname": string }`. `local_ip` वह
IPv4 पता है जो सिस्टम इंटरनेट की ओर ट्रैफ़िक के लिए चुनता है (कुछ भेजे बिना
पता लगाया गया), या कोई न हो तो `127.0.0.1`। `hostname` कंप्यूटर का नाम है,
या सिस्टम न बताए तो `localhost`।

### firewall_status {#firewall_status}

क्या सिस्टम का फ़ायरवॉल दूसरी मशीनों को इस प्रोग्राम तक पहुँचने देता है। केवल
Windows में प्रति-प्रोग्राम फ़ायरवॉल पढ़ा जा सकता है; बाकी जगह `applies` `false` होता है, और
बाकी में से केवल `program` भरा होता है। कोई आर्ग्युमेंट नहीं।

**नतीजा**

| फ़ील्ड | प्रकार | अर्थ |
| --- | --- | --- |
| `applies` | बूलियन | यहाँ प्रति-प्रोग्राम फ़ायरवॉल है (Windows) |
| `program` | स्ट्रिंग | वह प्रोग्राम जिसके बारे में नियम हैं |
| `enabled` | बूलियन | मशीन अभी जिस नेटवर्क पर है उसके लिए फ़ायरवॉल चालू है |
| `networks` | स्ट्रिंग[] | मशीन जिन तरह के नेटवर्क पर है: `domain`, `private`, `public` |
| `allowed` | बूलियन | मौजूदा नेटवर्क पर इस प्रोग्राम के लिए एक इनबाउंड नियम UDP को अंदर आने देता है |
| `blocked` | बूलियन | मौजूदा नेटवर्क पर इस प्रोग्राम को एक इनबाउंड नियम ब्लॉक करता है; यह किसी भी allow नियम पर भारी पड़ता है |
| `rules` | संख्या | इस प्रोग्राम के इनबाउंड नियम, किसी भी तरह के |

**त्रुटियाँ**: `firewall.failed`।

### firewall_allow {#firewall_allow}

दूसरी मशीनों को Signal Lab तक पहुँचने देता है: सिस्टम अपना एडमिनिस्ट्रेटर
प्रॉम्प्ट दिखाता है, फिर प्रोग्राम के इनबाउंड नियम (एक ब्लॉक नियम सहित) बदलकर
Signal Lab और उसके बगल वाली `signallab` कमांड लाइन के लिए एक-एक allow नियम कर दिए जाते हैं।
केवल Windows पर डेस्कटॉप ऐप; सर्वर मना कर देता है, क्योंकि उसकी स्क्रीन पर कोई नहीं है जो
प्रॉम्प्ट का जवाब दे।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `public` | बूलियन | हाँ | सार्वजनिक नेटवर्क पर भी अनुमति दें, केवल private और domain पर ही नहीं |

**नतीजा**: नया [`firewall_status`](#firewall_status)।

**त्रुटियाँ**: `firewall.server` (सर्वर पर), `firewall.unsupported` (Windows नहीं),
`firewall.declined` (प्रॉम्प्ट का जवाब नहीं दिया गया), `firewall.failed`।

### feedback_send {#feedback_send}

Signal Lab के डेवलपर्स को एक संदेश भेजता है, स्टूडियो के हब के ज़रिए, जो उसे
उन्हें मेल कर देता है।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `form` | ऑब्जेक्ट | हाँ | संदेश, नीचे। अज्ञात फ़ील्ड अस्वीकार किए जाते हैं |

| `form` का फ़ील्ड | प्रकार | डिफ़ॉल्ट | अर्थ |
| --- | --- | --- | --- |
| `message` | स्ट्रिंग | — | क्या हुआ; आवश्यक, अधिकतम 20 000 अक्षर |
| `email` | स्ट्रिंग | कोई नहीं | जहाँ जवाब जा सकता है |
| `meta` | स्ट्रिंग का ऑब्जेक्ट | `{}` | ऐप अपने बारे में क्या कहता है (version, os, arch, mode, lang, screen) |
| `screenshots` | `{ name, data }[]` | `[]` | छवियाँ, `data` base64 में; अधिकतम 6, हर एक 8 MiB |
| `logs` | `{ name, text }[]` | `[]` | टेक्स्ट फ़ाइलें; अधिकतम 4, हर एक 2 MiB |

सब मिलाकर अधिकतम 15 MiB।

**नतीजा**: `{ "id": string }`, वह संदर्भ जो डेवलपर्स को मिलता है।

**त्रुटियाँ**: `feedback.message_required`, `feedback.message_too_long`,
`feedback.too_many_files`, `feedback.file_too_large`, `feedback.too_large`,
`feedback.invalid`, हब की अस्वीकृतियाँ (`feedback.email_invalid`,
`feedback.file_type`, `feedback.rate_limited`, `feedback.disabled`,
`feedback.send_failed`, `feedback.failed`), और नेटवर्क की `transport.*`।

```bash
invoke app_info
# {"version":"[[version]]","mode":"server","secrets_writable":false,"data_dir":"/data","os":"linux","arch":"x86_64"}
```

## जॉब {#jobs}

### jobs_list {#jobs_list}

चल रहे जॉब, सबसे पुराना पहले। कोई आर्ग्युमेंट नहीं।

**नतीजा**: [`JobInfo`](#type-jobinfo)`[]`।

### job_stop {#job_stop}

एक जॉब को तुरंत रोकता है: उसके सॉकेट बंद हो जाते हैं, उसका रिले, सर्वर या कनेक्शन चला जाता है।
रुका हुआ जॉब कोई [`job://ended`](events.md#event-job-ended) नहीं भेजता; रुका हुआ
रन कोई रिपोर्ट नहीं सहेजता।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `id` | संख्या | हाँ | जॉब की `id` |

**नतीजा**: `true` जब उस id वाला जॉब चल रहा था, वरना `false`।

### jobs_stop_all {#jobs_stop_all}

हर चल रहे जॉब को रोकता है, चाहे उसे किसी ने भी शुरू किया हो। कोई आर्ग्युमेंट नहीं।

**नतीजा**: null।

```bash
invoke jobs_list
# [{"id":3,"kind":"osc-monitor","label":"OSC monitor 0.0.0.0:9000","params":{"bind":"0.0.0.0:9000"},"started_ms":1759600000000}]
invoke job_stop '{"id":3}'
# true
```

## प्रयोग और रन {#experiments}

ये कमांड एक प्रयोग दस्तावेज़ (`Experiment`) लेती और लौटाती हैं: वही JSON
जिसे एडिटर सहेजता और एक्सपोर्ट करता है, जिसमें `version`, `name`, `params`, `profiles`,
`profile`, `seed`, `cookies`, `nodes` और `edges` होते हैं। इसके नोड
[नोड](../experiments/nodes.md) में हैं; इसके पैरामीटर, प्रोफ़ाइल और टेम्पलेट
[डेटा](../experiments/data.md) में। एक दस्तावेज़ अधिकतम 4 MiB
(`file.too_large`) होता है। किसी प्रयोग को चलाकर उसके नतीजे की प्रतीक्षा करने के लिए
[`experiment_start`](#experiment_start) की बजाय [`POST /api/run`](run.md) उपयोग करें।

### experiment_load {#experiment_load}

कार्यशील प्रयोग: डेटा फ़ोल्डर में `experiment.json` — सर्वर पर वही जो
उसका इंटरफ़ेस दिखाता है। कोई न हो तो शुरुआती प्रयोग। पुराने
दस्तावेज़ वर्शन माइग्रेट किए जाते हैं। कोई आर्ग्युमेंट नहीं।

**नतीजा**: `Experiment`।

**त्रुटियाँ**: `file.io`, `file.json_invalid` (फ़ाइल के `path`, `line`
और `column` के साथ), `file.too_large`, `doc.version_unsupported` और बाकी
`doc.*` जाँचें।

### experiment_save {#experiment_save}

कार्यशील प्रयोग को बदल देता है, डेटा फ़ोल्डर में `experiment.json`। इसे
पहले एक अस्थायी फ़ाइल में लिखा जाता है, इसलिए विफल लेखन पिछले वाले को बचाए रखता है।

::: warning
सर्वर पर यह वही दस्तावेज़ है जिस पर हर ब्राउज़र का एडिटर काम करता है।
:::

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `document` | `Experiment` | हाँ | दस्तावेज़ |

**नतीजा**: स्ट्रिंग, लिखा गया पथ।

**त्रुटियाँ**: `doc.*`, दस्तावेज़ की आकार जाँचें (`param.*`, `params.too_many`,
`profile.*`, `profiles.too_many`, `seed.range`), `file.too_large`, `file.io`।

### experiment_parse {#experiment_parse}

JSON टेक्स्ट से एक प्रयोग पढ़ता है, जैसे [[ui:exp.importJson]] करता है। वर्शन 1
से 8 को वर्शन 9 में माइग्रेट किया जाता है, जो मौजूदा है; वर्शन 8 से पहले की फ़ाइल
`cookies` बंद के साथ खुलती है, इसलिए वह पहले जैसे ही चलती है। एक byte-order mark छोड़
दिया जाता है।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `text` | स्ट्रिंग | हाँ | फ़ाइल का टेक्स्ट |

**नतीजा**: `Experiment`।

**त्रुटियाँ**: `file.json_invalid` (`line`, `column`), `file.too_large`,
`doc.version_unsupported`, `doc.*`, [`experiment_save`](#experiment_save) की
आकार जाँचें।

### experiment_export {#experiment_export}

एक दस्तावेज़ का स्नैपशॉट डेटा फ़ोल्डर में
`exports/experiment-<ms>-<16 hex digits>.json` पर लिखता है। हर एक्सपोर्ट एक नई फ़ाइल होती है।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `document` | `Experiment` | हाँ | दस्तावेज़ |

**नतीजा**: स्ट्रिंग, लिखा गया पथ।

**त्रुटियाँ**: [`experiment_save`](#experiment_save) वाली।

### experiment_validate {#experiment_validate}

जाँचता है कि कोई दस्तावेज़ अपने सक्रिय प्रोफ़ाइल (या उसके डिफ़ॉल्ट) के साथ चलेगा या नहीं:
ग्राफ़, हर फ़ील्ड, पैरामीटर, और यह कि वह जिन सीक्रेट का नाम लेता है वे सब संग्रहीत हैं।
कोई अवरोधक समस्या ही त्रुटि है। सफल होने पर यह बताता है कि *बाकी*
प्रोफ़ाइल में से कौन विफल होंगे, ताकि स्विच करने से पहले आपको पता हो।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `document` | `Experiment` | हाँ | दस्तावेज़ |
| `overrides` | स्ट्रिंग का ऑब्जेक्ट | नहीं | केवल इस जाँच के लिए पैरामीटर मान, जैसे [[ui:exp.runWith]] उन्हें देता है |

**नतीजा**: `{ "profile": string or null, "error": EngineError }[]` — हर
ऐसी दूसरी प्रोफ़ाइल जो मान्य नहीं होगी (`null`: डिफ़ॉल्ट, बिना प्रोफ़ाइल)। एक ख़ाली सूची का
अर्थ है कि हर प्रोफ़ाइल ठीक है।

**त्रुटियाँ**: कोई भी मान्यता कोड (`doc.*`, `graph.*`, `node.*`, `param.*`,
`profile.*`, `template.*`, `loop.*`…), `run.override_unknown` (ऐसे पैरामीटर के लिए override
जो दस्तावेज़ में नहीं है), `secret.missing`,
`secret.store`, `secret.unsupported`।

### experiment_resolve {#experiment_resolve}

एक नोड, जिसके टेम्पलेट भरे हुए हैं, जैसे एडिटर का प्रीव्यू दिखाता है: सक्रिय
प्रोफ़ाइल के मान और आपके दिए वेरिएबल मान। सीक्रेट `••••` के रूप में दिखाए जाते हैं,
कभी उनके मान नहीं। जिन नामों का कोई मान नहीं, वे जैसे लिखे थे वैसे रहते हैं और
सूचीबद्ध किए जाते हैं।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `document` | `Experiment` | हाँ | दस्तावेज़ |
| `nodeId` | स्ट्रिंग | हाँ | नोड |
| `vars` | ऑब्जेक्ट | हाँ | उपयोग करने के वेरिएबल मान, नाम से; कोई नहीं तो `{}` |

**नतीजा**: `{ "node": node, "missing": string[] }`।

**त्रुटियाँ**: `node.not_found`, `template.*`, `secret.store`,
`secret.unsupported`।

### experiment_send_node {#experiment_send_node}

[[ui:exp.sendNow]]: एक नोड को अकेले चलाता है, उसी कोड के ज़रिए जो एक रन
उपयोग करता है। एक action भेजा जाता है; एक wait अब से सुनता है जब तक वह मेल न खाए या
समय समाप्त न हो जाए। एक [[ui:exp.node.ws_send]] या [[ui:exp.node.wait_ws]] नोड वह
कनेक्शन खोलता है जिसका वर्णन उसका [[ui:exp.node.ws_connect]] नोड करता है। कुकीज़ के साथ
कुछ नहीं भेजा जाता, और रन के [[ui:exp.node.impairment]] और [[ui:exp.node.emulator]]
नोड नहीं खोले जाते।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `document` | `Experiment` | हाँ | दस्तावेज़; इसका सक्रिय प्रोफ़ाइल पैरामीटर मान देता है |
| `nodeId` | स्ट्रिंग | हाँ | एक action या एक wait |
| `vars` | ऑब्जेक्ट | हाँ | वेरिएबल मान जो नोड के टेम्पलेट पढ़ते हैं; कोई नहीं तो `{}` |

**नतीजा**

| फ़ील्ड | प्रकार | अर्थ |
| --- | --- | --- |
| `detail` | स्ट्रिंग | क्या हुआ, अंग्रेज़ी में |
| `response` | [`HttpResponse`](#type-httpresponse) या null | एक HTTP नोड का रिस्पॉन्स |
| `vars` | ऑब्जेक्ट | चरण ने क्या सेट किया: किसी wait का reply, या किसी रिक्वेस्ट के बाद के [[ui:exp.node.extract]] नोड उसके रिस्पॉन्स से जो लेते हैं |

सीक्रेट मान इन सब में मास्क किए जाते हैं।

**त्रुटियाँ**: `node.not_found`, `run.not_an_action` (action या
wait नहीं), `ws.connection_unknown`, `secret.missing`, `template.*`, और जो कुछ भी
चरण इनसे विफल हो: `transport.*`, `wait.timeout`, `check.*`…

### experiment_start {#experiment_start}

एक रन शुरू करता है, जैसे [[ui:exp.run]] करता है, और तुरंत लौट आता है। इसके चरण
[`experiment://step`](events.md#event-experiment-step) इवेंट के रूप में आते हैं, इसका अंत
[`experiment://ended`](events.md#event-experiment-ended) के रूप में, और इसकी रिपोर्ट
डेटा फ़ोल्डर में `runs/` के नीचे सहेजी जाती है। वेट, एमुलेटर, इम्पेयरमेंट रिले और
MQTT सब्सक्रिप्शन पहले चरण से पहले खुलते हैं, इसलिए लिया गया पोर्ट यहीं विफल होता है।
300 s से लंबा रन `run.timeout` के साथ विफल होता है। *जॉब शुरू करता है*
(`experiment`)।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `document` | `Experiment` | हाँ | दस्तावेज़ |
| `overrides` | स्ट्रिंग का ऑब्जेक्ट | नहीं | केवल इस रन के लिए पैरामीटर मान |
| `seed` | संख्या | नहीं | रन का सीड, 0 से 9007199254740991; डिफ़ॉल्ट: दस्तावेज़ का, वरना नया |

**नतीजा**: [`JobInfo`](#type-jobinfo), `params.name` प्रयोग का नाम।

**त्रुटियाँ**: जो कुछ भी [`experiment_validate`](#experiment_validate) बताता है,
`seed.range`, `transport.address_in_use` और बाकी bind विफलताएँ,
`emulator.*`, `impair.*`, `node.params_only` (एक listen पता या MQTT
wait का ब्रोकर या टॉपिक जो रन शुरू होने पर तय नहीं है), और
किसी ऐसे ब्रोकर की [`mqtt_connect`](#mqtt_connect) त्रुटियाँ जहाँ MQTT wait नहीं पहुँच सकता।

### experiment_runs {#experiment_runs}

`runs/` में रिपोर्ट से वापस पढ़े गए रन, सबसे नया पहले। जो रिपोर्ट पढ़ी नहीं जा सकती,
वह छोड़ दी जाती है।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `name` | स्ट्रिंग | नहीं | केवल इसी ठीक नाम वाले प्रयोग के रन |
| `limit` | संख्या | नहीं | अधिकतम इतने; डिफ़ॉल्ट 50, अधिकतम 500 |

**नतीजा**: रन सारांश:

| फ़ील्ड | प्रकार | अर्थ |
| --- | --- | --- |
| `name` | स्ट्रिंग | रिपोर्ट का फ़ाइल नाम, `run-<ms>-<job>.json`: जो [`experiment_compare`](#experiment_compare) लेता है |
| `experiment` | स्ट्रिंग | प्रयोग का नाम |
| `started_ms`, `ended_ms` | संख्या | 1970 से मिलीसेकंड |
| `outcome` | स्ट्रिंग | `passed` या `failed` |
| `seed` | संख्या | रन का सीड |
| `profile` | स्ट्रिंग या null | इसकी प्रोफ़ाइल |
| `loads` | ऑब्जेक्ट[] | हर load चरण: `node`, `sent`, `rps`, `p95_ms`, `error_rate`, `held` (हर थ्रेशोल्ड टिका रहा) |

**त्रुटियाँ**: `file.io`।

### experiment_compare {#experiment_compare}

दो रन साथ-साथ, load चरण दर load चरण, जैसे टाइमलाइन का
[[ui:exp.compare]] उन्हें दिखाता है। चरण नोड id से मिलाए जाते हैं।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `a` | स्ट्रिंग | हाँ | पहले वाले रन की रिपोर्ट फ़ाइल का नाम |
| `b` | स्ट्रिंग | हाँ | बाद वाले रन की रिपोर्ट फ़ाइल का नाम |

**नतीजा**: `{ "a": summary, "b": summary, "steps": [...] }`, हर चरण के साथ
`node`, `missing_in` (`a` या `b`, जब केवल एक रन में हो), `metrics`,
`sent` (`[a, b]`), `thresholds_a` और `thresholds_b` (हर थ्रेशोल्ड
`{ metric, op, value, actual, held }` के रूप में)। `metrics` नौ मीट्रिक सूचीबद्ध करता है,
हर एक `{ metric, a, b, change, percent, worse }` के रूप में: `metric` `p50_ms`, `p90_ms`,
`p95_ms`, `p99_ms`, `mean_ms`, `max_ms`, `error_rate`, `rps` या `missed` होता है;
`change` `b − a` है; `percent` `a` के % में बदलाव (null जब `a` 0 हो);
`worse` कि वह ग़लत दिशा में चला — ऊपर, या `rps` के लिए नीचे — 5 % या
ज़्यादा, या 0 से किसी भी चीज़ तक। जो चरण केवल एक रन में है, वह कभी `worse` नहीं होता। देखें
[लोड](../experiments/load.md)।

**त्रुटियाँ**: `runs.name_invalid` (रिपोर्ट के फ़ाइल नाम के अलावा कुछ भी, कोई फ़ोल्डर नहीं),
`runs.not_found`, `file.json_invalid`, `file.io`।

```bash
invoke experiment_validate "$(jq '{document: .}' experiment.json)"
# []
```

## सीक्रेट {#secrets}

सीक्रेट मान प्रयोगों द्वारा `{{secret.NAME}}` के रूप में उपयोग किए जाते हैं और कभी इंजन
से बाहर नहीं जाते: कोई कमांड एक भी नहीं लौटाती। वे कहाँ रखे जाते हैं यह इस पर निर्भर है कि
इंजन कहाँ चलता है:

| कहाँ | स्टोर | सेट करना और हटाना |
| --- | --- | --- |
| डेस्कटॉप ऐप, Windows | Windows Credential Manager | हाँ |
| डेस्कटॉप ऐप, Linux | कोई नहीं | `secret.unsupported` |
| सर्वर | `SIGNALLAB_SECRET_<NAME>`, या `--secrets-dir` में `<NAME>` फ़ाइल (डिफ़ॉल्ट `/run/secrets/signallab`) | नहीं: `secret.read_only` |

एक नाम एक लैटिन अक्षर या `_` से शुरू होता है, आगे लैटिन अक्षरों, अंकों
और `_` से चलता है, और अधिकतम 128 अक्षर का होता है (`secret.name_invalid`)।

### secret_status {#secret_status}

दिए गए नामों में से किन का मान संग्रहीत है।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `names` | स्ट्रिंग[] | हाँ | देखने के नाम |

**नतीजा**: एक ऑब्जेक्ट, नाम → `true` (संग्रहीत) या `false`।

**त्रुटियाँ**: `secret.name_invalid` (सर्वर पर), `secret.store`,
`secret.too_large` (सर्वर की फ़ाइल 16 KiB से बड़ी), `secret.unsupported`।

### secret_set {#secret_set}

एक नाम के नीचे मान संग्रहीत करता है, वहाँ का बदल देता है। केवल डेस्कटॉप ऐप।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `name` | स्ट्रिंग | हाँ | नाम |
| `value` | स्ट्रिंग | हाँ | ख़ाली नहीं; अधिकतम 16 KiB |

**नतीजा**: null।

**त्रुटियाँ**: `secret.read_only` (सर्वर पर), `secret.unsupported`,
`secret.name_invalid`, `secret.empty`, `secret.too_large`, `secret.store`।

### secret_delete {#secret_delete}

एक संग्रहीत मान हटाता है। जो संग्रहीत नहीं है उसे हटाना त्रुटि नहीं है।
केवल डेस्कटॉप ऐप।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `name` | स्ट्रिंग | हाँ | नाम |

**नतीजा**: null।

**त्रुटियाँ**: `secret.read_only` (सर्वर पर), `secret.unsupported`,
`secret.name_invalid`, `secret.store`।

```bash
invoke secret_status '{"names":["API_TOKEN","MQTT_PASSWORD"]}'
# {"API_TOKEN":true,"MQTT_PASSWORD":false}
```

## OSC {#osc}

इन कमांड की सेवा करने वाली स्क्रीन के लिए [OSC](../protocols/osc.md) देखें।

### osc_send {#osc_send}

एक ताज़ा सॉकेट से, एक UDP डेटाग्राम में एक OSC संदेश भेजता है।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `target` | स्ट्रिंग | हाँ | भेजने के लिए `IP:port` या `host:port`; नाम खोजा जाता है, जब इसका IPv4 पता हो तो वही लिया जाता है |
| `address` | स्ट्रिंग | हाँ | OSC पता, `/mixer/fader/1`; यह `/` से शुरू होता है |
| `args` | [`OscArg`](#type-oscarg)`[]` | हाँ | आर्ग्युमेंट; कोई नहीं तो `[]` |

**नतीजा**: संख्या, भेजे गए बाइट।

**त्रुटियाँ**: `node.osc_address` (शुरू में `/` नहीं; फ़ील्ड `address`),
`transport.target_invalid` (पोर्ट नहीं, या कोई भी रूप नहीं), `transport.dns` (नाम
resolve नहीं होता), `transport.*`।

### osc_monitor_start {#osc_monitor_start}

UDP पोर्ट पर OSC सुनता है और हर पैकेट को डिकोड करता है। हर एक
[`osc://message`](events.md#event-osc-message) इवेंट के रूप में आता है। *जॉब शुरू करता है*
(`osc-monitor`, `params.bind`)।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `bind` | स्ट्रिंग | हाँ | सुनने का `IP:port`: `0.0.0.0:9000` हर नेटवर्क कार्ड, `127.0.0.1:9000` केवल यह मशीन |

**नतीजा**: [`JobInfo`](#type-jobinfo)।

**त्रुटियाँ**: `node.bind_invalid`, `transport.address_in_use`,
`transport.address_unavailable`, `transport.denied`, `wait.bind_failed`। जॉब
`wait.receive_failed` के साथ समाप्त होता है अगर सॉकेट अब और प्राप्त न कर सके।

### osc_generator_start {#osc_generator_start}

ऐसे OSC संदेशों की धारा भेजता है जिनका अकेला आर्ग्युमेंट एक waveform का अनुसरण करता है।
प्रगति [`osc://gen-tick`](events.md#event-osc-gen-tick) के रूप में आती है। *जॉब शुरू
करता है* (`osc-gen`, `params.target`, `params.address`)।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `config` | ऑब्जेक्ट | हाँ | नीचे |

| `config` का फ़ील्ड | प्रकार | डिफ़ॉल्ट | अर्थ |
| --- | --- | --- | --- |
| `target` | स्ट्रिंग | — | भेजने के लिए `IP:port` या `host:port`; नाम एक बार खोजा जाता है, जब जॉब शुरू होता है |
| `address` | स्ट्रिंग | — | OSC पता; यह `/` से शुरू होता है |
| `rate` | संख्या | — | प्रति सेकंड संदेश, 0.1 और 5000 के बीच रखा गया |
| `waveform` | स्ट्रिंग | — | `sine`, `triangle`, `saw` (गिरता: `max` से `min`, फिर तुरंत वापस), `ramp` (चढ़ता: `min` से `max`, फिर तुरंत वापस), `square`, `random` या `constant` (`max`) |
| `freq` | संख्या | — | प्रति सेकंड waveform के चक्र |
| `min`, `max` | संख्या | — | मान की सीमा |
| `as_int` | बूलियन | `false` | गोल करके float की बजाय int भेजें |
| `duration_s` | संख्या | `0` | इतने सेकंड बाद रोकें; 0 तब तक चलता है जब तक रोका न जाए |

**नतीजा**: [`JobInfo`](#type-jobinfo)।

**त्रुटियाँ**: `node.osc_address`, `transport.target_invalid`, `transport.dns`,
`transport.*`। भेजना विफल होने पर जॉब `transport.*` त्रुटि के साथ समाप्त होता है।

```bash
invoke osc_send '{"target":"127.0.0.1:9000","address":"/cue/go","args":[{"type":"int","value":1}]}'
# 16
invoke osc_monitor_start '{"bind":"0.0.0.0:9000"}'
```

## HTTP और कुकीज़ {#http}

[HTTP](../protocols/http.md) देखें।

### http_request {#http_request}

एक HTTP रिक्वेस्ट भेजता है और रिस्पॉन्स लौटाता है। जिस रिक्वेस्ट का कोई
रिस्पॉन्स नहीं आता — refused, timed out, ऐसा नाम जो resolve नहीं होता, ऐसा प्रमाणपत्र
जिस पर भरोसा नहीं — वह कमांड की **त्रुटि नहीं** है: रिस्पॉन्स यह
`error` और `cause` में बताता है।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `request` | [`HttpRequest`](#type-httprequest) | हाँ | रिक्वेस्ट |
| `cookies` | बूलियन | नहीं | [[ui:nav.http]] स्क्रीन की कुकी जार भेजें और जवाब जो सेट करे उसे रखें; डिफ़ॉल्ट `false` |

**नतीजा**: [`HttpResponse`](#type-httpresponse)।

**त्रुटियाँ**: `http.client_failed` (रिक्वेस्ट तैयार भी नहीं हो सकी)।

### http_burst_start {#http_burst_start}

एक रिक्वेस्ट कई बार, कई एक साथ भेजता है, और उसे मापता है। `rate` के बिना,
हर वर्कर जवाब मिलते ही फिर भेजता है; उसके साथ, रिक्वेस्ट एक तय
शेड्यूल पर शुरू होती हैं चाहे जवाब कितने भी धीमे हों, और जो रिक्वेस्ट ख़ाली वर्कर के लिए अपने
क्षण से 50 ms से ज़्यादा प्रतीक्षा करती है, वह छोड़ दी जाती है और missed गिनी जाती है। प्रगति
[`http://burst-progress`](events.md#event-http-burst-progress) के रूप में सेकंड में दस बार आती
है। *जॉब शुरू करता है* (`http-burst`, `params.method`, `params.url`, और
paced होने पर `params.rate`)।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `config` | ऑब्जेक्ट | हाँ | [`HttpRequest`](#type-httprequest) के फ़ील्ड और नीचे वाले, एक ही ऑब्जेक्ट में |

| `config` का फ़ील्ड | प्रकार | डिफ़ॉल्ट | अर्थ |
| --- | --- | --- | --- |
| `concurrency` | संख्या | — | एक साथ अधिकतम इतने, 1 और 512 के बीच रखा गया |
| `total` | संख्या | `0` | इतनी रिक्वेस्ट के बाद रोकें; 0: कोई गिनती नहीं |
| `duration_s` | संख्या | `0` | इतने सेकंड बाद रोकें; 0: कोई समय सीमा नहीं |
| `rate` | संख्या | `0` | प्रति सेकंड शुरू होने वाली रिक्वेस्ट, 0.1 से 100 000; 0: जवाब जितनी तेज़ आएँ उतनी तेज़ |
| `cookies` | बूलियन | `false` | [[ui:nav.http]] स्क्रीन की कुकी जार उपयोग करें |

न `total` न `duration_s` होने पर, burst तब तक चलता है जब तक रोका न जाए।

**नतीजा**: [`JobInfo`](#type-jobinfo)।

**त्रुटियाँ**: `http.rate_invalid`, `http.duration_invalid`, `http.client_failed`।

### http_cookies {#http_cookies}

[[ui:nav.http]] स्क्रीन की कुकी जार: हर वह कुकी जिसकी अवधि समाप्त नहीं हुई। सर्वर पर
हर पेज और स्क्रिप्ट के लिए एक जार होती है। कोई आर्ग्युमेंट नहीं।

**नतीजा**: कुकीज़, हर एक में `name`, `value`, `domain`, `host_only` (कोई
Domain attribute नहीं: केवल वही होस्ट जिसने इसे सेट किया उसे यह वापस मिलती है), `path`,
`expires` (Unix सेकंड, session कुकी के लिए null), `secure`, `http_only` और
`same_site` (स्ट्रिंग या null)।

### http_cookies_clear {#http_cookies_clear}

[[ui:nav.http]] स्क्रीन की कुकी जार ख़ाली करता है। कोई आर्ग्युमेंट नहीं।

**नतीजा**: null।

```bash
invoke http_request '{"request":{"method":"GET","url":"http://127.0.0.1:8080/health","headers":[["Accept","application/json"]],"body":null,"timeout_ms":5000}}' \
  | jq '{status, latency_ms, body}'
```

## WebSocket {#websocket}

[WebSocket](../protocols/websocket.md) देखें। जो कनेक्शन `ws_connect`
खोलता है वह एक जॉब है; बाकी उसे `jobId` से नाम देते हैं।

### ws_connect {#ws_connect}

एक WebSocket खोलता है और उसे खुला रखता है। जो आता है और जो भेजा जाता है वह
[`ws://messages`](events.md#event-ws-messages) के रूप में हर 100 ms आता है; कनेक्शन की
स्थिति [`ws://state`](events.md#event-ws-state) के रूप में। *जॉब शुरू करता है*
(`websocket`, `params.url`)।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | हाँ | कहाँ और कैसे जुड़ना है |

**नतीजा**: [`JobInfo`](#type-jobinfo)।

**त्रुटियाँ**: `ws.url_invalid`, `ws.header_invalid`, `ws.protocol_invalid`,
`ws.handshake_status` (सर्वर ने upgrade का जवाब किसी दूसरे स्टेटस से दिया),
`ws.subprotocol_refused`, `ws.handshake_failed`, `transport.*`।

### ws_send {#ws_send}

एक खुले कनेक्शन पर एक संदेश भेजता है।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `jobId` | संख्या | हाँ | कनेक्शन का जॉब |
| `message` | ऑब्जेक्ट | हाँ | text संदेश के लिए `{ "text": "…" }`, या binary के लिए `{ "hex": "de ad be ef" }`; इनमें से ठीक एक |

**नतीजा**: संख्या, भेजे गए बाइट।

**त्रुटियाँ**: `ws.not_connected`, `ws.payload_required` (न कोई न दोनों),
`hex.invalid` (ख़ाली `hex` भी), `node.too_long` (16 MiB से ज़्यादा, फ़ील्ड
`payload`; कुछ नहीं भेजा जाता और कनेक्शन खुला रहता है), `ws.closed`, `transport.*`
(सर्वर 10 s तक पढ़ना बंद कर दे तो `transport.timeout`)।

### ws_close {#ws_close}

एक कनेक्शन को close handshake के साथ बंद करता है और सर्वर के जवाब के लिए अधिकतम 2 s
प्रतीक्षा करता है; फिर जॉब समाप्त हो जाता है। एक बार कनेक्शन समाप्त हो जाने पर उसका जॉब चला जाता
है और उसे बंद करना `ws.not_connected` होता है।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `jobId` | संख्या | हाँ | कनेक्शन का जॉब |
| `code` | संख्या | नहीं | 1000, या किसी ऐप्लिकेशन के अपने के लिए 3000 से 4999; डिफ़ॉल्ट 1000 |
| `reason` | स्ट्रिंग | नहीं | अधिकतम 123 बाइट; डिफ़ॉल्ट ख़ाली |

**नतीजा**: `{ "code", "reason", "by", "error" }` — `by` `client`,
`server` या `lost` होता है; `code` 1005 होता है जब बंद करने में कोई नहीं था और 1006 जब
कोई close frame नहीं था।

**त्रुटियाँ**: `ws.close_code`, `node.too_long`, `ws.not_connected`।

### ws_exchange {#ws_exchange}

बिना जॉब के एक आदान-प्रदान: जुड़ें, दिया हो तो एक संदेश भेजें, कहा हो तो
जवाब की प्रतीक्षा करें, बंद करें।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | हाँ | कहाँ और कैसे जुड़ना है |
| `message` | ऑब्जेक्ट | नहीं | `{ "text" }` या `{ "hex" }`, जैसे [`ws_send`](#ws_send) के लिए |
| `expect` | ऑब्जेक्ट | नहीं | किसकी प्रतीक्षा करनी है: `mode` (`any`, `contains`, `regex`, `hex`; डिफ़ॉल्ट `any`), `pattern` (डिफ़ॉल्ट ख़ाली), `timeout_ms` (डिफ़ॉल्ट 2000) |

`expect` के साथ और बिना `message`, जुड़ने के बाद का पहला मेल खाता संदेश
गिना जाता है — एक अभिवादन।

**नतीजा**: `{ "handshake", "sent", "reply", "closed" }` — `handshake`
`{ url, peer, local, protocol, ms }` है; `sent` भेजे गए बाइट या null; `reply`
`{ kind, text, hex, bytes, json, ms }` या null (`json`: पार्स किया गया text जवाब,
वरना null; `ms`: भेजने के बाद से, या जब कुछ भेजा न गया हो तो जुड़ने के बाद से);
`closed` जैसे [`ws_close`](#ws_close) लौटाता है।

**त्रुटियाँ**: [`ws_connect`](#ws_connect) और [`ws_send`](#ws_send) वाली,
`wait.timeout` (`ms`, `unmatched` और `target` के साथ), `regex.invalid` और
`hex.invalid` (ऐसा `pattern` जो parse नहीं होता)।

```bash
invoke ws_exchange '{"config":{"url":"ws://127.0.0.1:9001/"},"message":{"text":"{\"type\":\"ping\"}"},"expect":{"mode":"contains","pattern":"pong"}}' \
  | jq .reply.text
```

## MQTT {#mqtt}

सादे TCP पर MQTT 3.1.1, QoS 0, 1 और 2। देखें [MQTT](../protocols/mqtt.md)।

### mqtt_connect {#mqtt_connect}

एक ब्रोकर से जुड़ता है और कनेक्शन बनाए रखता है। कमांड तब लौटती है जब ब्रोकर
कनेक्शन स्वीकार कर ले (CONNACK), इसलिए ग़लत पासवर्ड या बंद पोर्ट इसकी त्रुटि है। संदेश
[`mqtt://messages`](events.md#event-mqtt-messages) के रूप में हर 100 ms आते हैं; स्थिति
का बदलाव [`mqtt://state`](events.md#event-mqtt-state) के रूप में; पूरे हुए QoS 1/2
publishes और unsubscribes [`mqtt://ack`](events.md#event-mqtt-ack) के रूप में।
*जॉब शुरू करता है* (`mqtt`, `params.broker`, `params.client`)।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | हाँ | ब्रोकर और कैसे जुड़ना है |

**नतीजा**: [`JobInfo`](#type-jobinfo)।

**त्रुटियाँ**: `mqtt.client_id_required`, `transport.*` (refused, unreachable,
`dns`, 6 s बाद `timeout`), `mqtt.no_answer` (6 s के भीतर कोई CONNACK नहीं),
`mqtt.protocol`, `mqtt.refused_protocol`, `mqtt.refused_client_id`,
`mqtt.refused_unavailable`, `mqtt.refused_credentials`,
`mqtt.refused_not_authorized`, `mqtt.refused`।

### mqtt_publish {#mqtt_publish}

एक खुले कनेक्शन पर पब्लिश करता है।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `jobId` | संख्या | हाँ | कनेक्शन का जॉब |
| `topic` | स्ट्रिंग | हाँ | टॉपिक: ख़ाली नहीं, और कोई `+` या `#` नहीं |
| `payload` | स्ट्रिंग | हाँ | पेलोड, UTF-8 के रूप में भेजा गया |
| `qos` | संख्या | हाँ | 0, 1 या 2 (2 से ऊपर 2 के रूप में भेजा जाता है) |
| `retain` | बूलियन | हाँ | ब्रोकर से इसे रखने को कहें; `retain` के साथ ख़ाली पेलोड एक retained मान साफ़ करता है |

**नतीजा**: null। QoS 1 या 2 publish की पुष्टि बाद में
[`mqtt://ack`](events.md#event-mqtt-ack) से होती है।

**त्रुटियाँ**: `node.topic_wildcard` (फ़ील्ड `topic`) और `mqtt.topic_required`
(फ़ील्ड `topic`), जैसे [`mqtt_publish_once`](#mqtt_publish_once) के लिए — कमांड
कनेक्शन खोजने से पहले इन्हें अस्वीकार कर देती है; `mqtt.not_connected`।

### mqtt_subscribe {#mqtt_subscribe}

एक खुले कनेक्शन को filters पर सब्सक्राइब करता है। ब्रोकर जो देता है वह
[`mqtt://state`](events.md#event-mqtt-state) के रूप में `state: "subscribed"` के साथ आता है।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `jobId` | संख्या | हाँ | कनेक्शन का जॉब |
| `filters` | `{ filter, qos }[]` | हाँ | कम से कम एक; `qos` डिफ़ॉल्ट 0। `+` और `#` wildcards हैं |

**नतीजा**: null।

**त्रुटियाँ**: `mqtt.filter_required`, `mqtt.not_connected`।

### mqtt_unsubscribe {#mqtt_unsubscribe}

एक खुले कनेक्शन को filters से अनसब्सक्राइब करता है।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `jobId` | संख्या | हाँ | कनेक्शन का जॉब |
| `filters` | स्ट्रिंग[] | हाँ | कम से कम एक |

**नतीजा**: null। ब्रोकर का जवाब
[`mqtt://ack`](events.md#event-mqtt-ack) के रूप में `kind: "unsubscribed"` के साथ आता है।

**त्रुटियाँ**: `mqtt.filter_required`, `mqtt.not_connected`।

### mqtt_publish_once {#mqtt_publish_once}

जुड़ता है, एक संदेश पब्लिश करता है, अपने QoS के माँगे acknowledgement की प्रतीक्षा करता है
(अधिकतम 6 s), डिसकनेक्ट करता है। यह अपना अलग कनेक्शन लाता है, अपनी ख़ुद की client id के नीचे
— `client_id` के पहले 12 अक्षर, `-o` और एक संख्या — इसलिए यह उस id वाले किसी जीवित कनेक्शन को
ब्रोकर से कभी नहीं गिराता।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | हाँ | ब्रोकर; `subscribe` उपयोग नहीं होता |
| `topic` | स्ट्रिंग | हाँ | ख़ाली नहीं और बिना `+` या `#` |
| `payload` | स्ट्रिंग | हाँ | UTF-8 के रूप में भेजा गया |
| `qos` | संख्या | हाँ | 0, 1 या 2 (2 से ऊपर 2 के रूप में भेजा जाता है) |
| `retain` | बूलियन | हाँ | ब्रोकर से इसे रखने को कहें |

**नतीजा**: स्ट्रिंग, Signal Lab द्वारा लिखा गया सारांश:
`<topic> → <broker> · <bytes> B · qos<n>`, retained होने पर ` retained` के साथ।

**त्रुटियाँ**: `mqtt.topic_required`, `node.topic_wildcard`, और
[`mqtt_connect`](#mqtt_connect) वाली पर `mqtt.client_id_required` नहीं: यहाँ ख़ाली
`client_id` स्वीकार किया जाता है।

```bash
invoke mqtt_publish_once '{"config":{"host":"127.0.0.1","port":1883,"client_id":"lab"},"topic":"lab/lamp/set","payload":"ON","qos":1,"retain":false}'
# "lab/lamp/set → 127.0.0.1:1883 · 2 B · qos1"
```

## ब्रॉडकास्ट, मल्टीकास्ट और डिस्कवरी {#broadcast}

[ब्रॉडकास्ट और डिस्कवरी](../protocols/broadcast.md) देखें।

::: danger
ब्रॉडकास्ट और एक स्वीप एक नेटवर्क सेगमेंट के हर होस्ट तक पहुँचते हैं। केवल उन्हीं
नेटवर्क पर भेजें जिनके लिए आप ज़िम्मेदार हैं।
:::

### broadcast_send {#broadcast_send}

हर लक्ष्य को एक डेटाग्राम, एक बार भेजता है।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `config` | ऑब्जेक्ट | हाँ | नीचे |

| `config` का फ़ील्ड | प्रकार | डिफ़ॉल्ट | अर्थ |
| --- | --- | --- | --- |
| `mode` | स्ट्रिंग | — | `list`, `broadcast`, `multicast` या `sweep` |
| `target` | स्ट्रिंग | — | mode के अनुसार, नीचे |
| `port` | संख्या | `0` | पोर्ट, केवल `sweep` के लिए |
| `payload` | [`Payload`](#type-payload) | — | हर डेटाग्राम क्या ले जाता है |
| `bind` | स्ट्रिंग | कोई भी | वह स्थानीय `IP:port` जिससे यह भेजा जाता है; ख़ाली या null: `0.0.0.0:0` (`[::]:0` जब हर लक्ष्य IPv6 हो) |
| `ttl` | संख्या | `1` | IP TTL, या multicast hop limit; 1 से 255 |
| `multicast_loop` | बूलियन | `true` | multicast इस मशीन तक भी वापस आता है |
| `rate`, `count`, `duration_s` | संख्या | `0` | केवल [`broadcast_beacon_start`](#broadcast_beacon_start) के लिए |

| `mode` | `target` |
| --- | --- |
| `list` | `IP:port` या `host:port` प्रविष्टियाँ, कॉमा, सेमीकोलन या नई पंक्तियों से अलग (स्पेस से नहीं); नाम खोजा जाता है, जब इसका IPv4 पता हो तो वही लिया जाता है |
| `broadcast` | `255.255.255.255:port`, या `.255` पर समाप्त होता पता अपने पोर्ट के साथ |
| `multicast` | 224.0.0.0 से 239.255.255.255 तक का समूह, अपने पोर्ट के साथ |
| `sweep` | एक CIDR ब्लॉक, `192.0.2.0/24`: `port` पर हर उपयोगी होस्ट; अधिकतम 1024 होस्ट, इसलिए `/22` या उससे संकरा |

**नतीजा**

| फ़ील्ड | प्रकार | अर्थ |
| --- | --- | --- |
| `targets` | संख्या | गंतव्य |
| `packets`, `bytes` | संख्या | जो बाहर गया |
| `errors` | संख्या | जो डेटाग्राम भेजे नहीं जा सके |
| `resolved` | स्ट्रिंग[] | पहले 8 गंतव्य |
| `summary` | स्ट्रिंग | पेलोड एक पंक्ति में |
| `error` | `EngineError` | पहला विफल डेटाग्राम क्यों विफल हुआ; कोई विफल न हो तो छोड़ दिया जाता है |

**त्रुटियाँ**: `broadcast.target_required`, `broadcast.not_broadcast`,
`broadcast.ipv6`, `broadcast.not_multicast`, `broadcast.sweep_port`,
`broadcast.cidr_invalid`, `broadcast.prefix_invalid`,
`broadcast.sweep_too_large`, `node.osc_address` (OSC पता `/` से शुरू होना चाहिए),
`hex.empty`, `hex.invalid`, `node.bind_invalid`,
`socket.option_failed`, `transport.target_invalid`, `transport.dns`, bind
विफलताएँ।

### broadcast_beacon_start {#broadcast_beacon_start}

वही दौर — हर लक्ष्य को एक डेटाग्राम — बार-बार भेजता है। इसके काउंटर
[`broadcast://emit-stat`](events.md#event-broadcast-emit-stat) के रूप में हर
250 ms आते हैं। 32 से ज़्यादा विफल भेजे जाने के बाद, बिना एक भी भेजे, यह
कारण के साथ रुक जाता है। *जॉब शुरू करता है* (`beacon`, `params.mode`, `params.target`,
`params.targets`, `params.rate`)।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `config` | ऑब्जेक्ट | हाँ | जैसे [`broadcast_send`](#broadcast_send) के लिए, नीचे वाले तीन के साथ |

| `config` का फ़ील्ड | प्रकार | डिफ़ॉल्ट | अर्थ |
| --- | --- | --- | --- |
| `rate` | संख्या | — | प्रति सेकंड दौर; 0 से ऊपर, और दौर × लक्ष्य अधिकतम 50 000 डेटाग्राम प्रति सेकंड |
| `count` | संख्या | `0` | इतने दौर के बाद रोकें; 0: कोई गिनती नहीं |
| `duration_s` | संख्या | `0` | इतने सेकंड बाद रोकें; 0: रोके जाने तक |

**नतीजा**: [`JobInfo`](#type-jobinfo)।

**त्रुटियाँ**: [`broadcast_send`](#broadcast_send) वाली,
`broadcast.rate_invalid`, `broadcast.rate_limit`।

### discovery_start {#discovery_start}

एक UDP पोर्ट पर सुनता है, हर उस पीयर की सूची रखता है जो कुछ भेजता है, और
किसी डिवाइस की तरह probes का जवाब दे सकता है। पीयर
[`broadcast://peers`](events.md#event-broadcast-peers) के रूप में हर 400 ms आते हैं। *जॉब
शुरू करता है* (`discovery`, `params.bind`, `params.groups`, `params.joined`)।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `config` | ऑब्जेक्ट | हाँ | नीचे |

| `config` का फ़ील्ड | प्रकार | डिफ़ॉल्ट | अर्थ |
| --- | --- | --- | --- |
| `bind` | स्ट्रिंग | — | सुनने का `IP:port` |
| `groups` | स्ट्रिंग[] | `[]` | जुड़ने के multicast समूह (IPv4) |
| `interface` | स्ट्रिंग | कोई भी | वह स्थानीय IPv4 पता जिस पर समूहों से जुड़ना है |
| `reuse` | बूलियन | `true` | पोर्ट उस प्रोग्राम के साथ साझा करें जो पहले से उस पर सुन रहा है (`SO_REUSEADDR`) |
| `respond` | बूलियन | `false` | जो आए उसका जवाब दें |
| `response` | [`Payload`](#type-payload) | कोई नहीं | जवाब; `respond` के साथ ज़रूरी |
| `respond_delay_ms` | संख्या | `0` | जवाब देने से पहले इतनी प्रतीक्षा करें |
| `match_contains` | स्ट्रिंग | कोई नहीं | केवल उन डेटाग्राम का जवाब दें जिनके टेक्स्ट में यह हो |

अधिकतम 512 पीयर सूचीबद्ध होते हैं; बाद वाले जोड़े नहीं जाते।

**नतीजा**: [`JobInfo`](#type-jobinfo)।

**त्रुटियाँ**: `node.bind_invalid`, `broadcast.port_shared` (पोर्ट लिया गया है
और `reuse` बंद है), `broadcast.interface_invalid`,
`broadcast.not_multicast`, `broadcast.join_failed`,
`broadcast.reply_missing`, `node.osc_address`, `hex.*`, bind विफलताएँ। जॉब
`wait.receive_failed` के साथ समाप्त होता है अगर सॉकेट अब और प्राप्त न कर सके।

```bash
invoke broadcast_send '{"config":{"mode":"list","target":"127.0.0.1:9000, 127.0.0.1:9001","payload":{"kind":"text","text":"PING"}}}' \
  | jq '{packets, errors}'
```

## इम्पेयरमेंट {#impairment}

एक क्लाइंट और उसके सर्वर के बीच एक रिले जो जो गुज़रता है उसे देर करता, गिराता, दोहराता,
ख़राब करता, पुनःक्रमित करता या गति सीमित करता है, UDP या TCP पर। देखें
[इम्पेयरमेंट](../tools/impairment.md)।

### netsim_start {#netsim_start}

एक रिले शुरू करता है: `listen` पर जो आता है वह `target` तक जाता है, और जवाब
उसी रास्ते वापस आते हैं, दोनों profile से impaired। इसके काउंटर
[`netsim://stat`](events.md#event-netsim-stat) के रूप में हर 250 ms आते हैं। *जॉब शुरू करता
है* (`netsim`, `params.listen`, `params.target`, और TCP के लिए `params.protocol`)।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `config` | ऑब्जेक्ट | हाँ | नीचे |

| `config` का फ़ील्ड | प्रकार | डिफ़ॉल्ट | अर्थ |
| --- | --- | --- | --- |
| `listen` | स्ट्रिंग | — | वह `IP:port` जिस पर रिले सुनता है; क्लाइंट को यहाँ इंगित करें |
| `target` | स्ट्रिंग | — | असली सर्वर का `IP:port`, या `host:port` — होस्ट नाम एक बार खोजा जाता है, जब रिले शुरू होता है |
| `profile` | [`ImpairProfile`](#type-impairprofile) | — | ट्रैफ़िक के साथ क्या करना है |
| `seed` | संख्या | नया | draws का सीड: वही सीड और वही ट्रैफ़िक वही drops देते हैं |
| `protocol` | स्ट्रिंग | `udp` | `udp` (डेटाग्राम) या `tcp` (streams) |

**नतीजा**: [`JobInfo`](#type-jobinfo)।

**त्रुटियाँ**: `node.range` (profile का मान उसकी सीमा से बाहर,
`min`, `max` और फ़ील्ड के साथ), `node.too_long`, `node.bind_invalid`,
`transport.target_invalid`, `transport.dns` (ऐसा target नाम जो मिल नहीं सकता),
bind विफलताएँ। जॉब `wait.receive_failed` के साथ समाप्त होता है
अगर कोई सॉकेट अब और प्राप्त न कर सके।

### netsim_set_profile {#netsim_set_profile}

एक चलता हुआ रिले अब से दूसरे profile से impair करता है, बिना अपने
sockets बंद किए।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `jobId` | संख्या | हाँ | रिले का जॉब |
| `profile` | [`ImpairProfile`](#type-impairprofile) | हाँ | नया profile |

**नतीजा**: null।

**त्रुटियाँ**: `netsim.not_running`, `node.range`, `node.too_long`।

```bash
invoke netsim_start '{"config":{"listen":"127.0.0.1:9010","target":"127.0.0.1:9000","profile":{"latency_ms":80,"jitter_ms":20,"loss":0.02}}}'
```

## स्टॉर्म और स्कैनर {#storm-scanner}

::: danger
एक स्टॉर्म किसी लक्ष्य पर उतना भार डालता है जितना आप कहें, और एक स्कैन किसी
रेंज के हर पोर्ट की जाँच करता है। इन्हें केवल उन्हीं होस्ट पर लक्षित करें जिनके लिए आप
ज़िम्मेदार हैं।
:::

### storm_start {#storm_start}

एक लक्ष्य को UDP डेटाग्राम या TCP कनेक्शन का स्थिर भार भेजता है। इसके
काउंटर [`storm://stat`](events.md#event-storm-stat) के रूप में हर 250 ms आते हैं।
*जॉब शुरू करता है* (`storm`, `params.protocol`, `params.target`, `params.rate`)।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `config` | ऑब्जेक्ट | हाँ | नीचे |

| `config` का फ़ील्ड | प्रकार | डिफ़ॉल्ट | अर्थ |
| --- | --- | --- | --- |
| `target` | स्ट्रिंग | — | `IP:port` या `host:port`; नाम एक बार खोजा जाता है, जब जॉब शुरू होता है |
| `protocol` | स्ट्रिंग | — | `udp`: डेटाग्राम; `tcp`: प्रति यूनिट एक कनेक्शन जो पेलोड लिखता है और बंद कर देता है (हर connect में 500 ms लग सकते हैं) |
| `size` | संख्या | — | पेलोड बाइट, 1 और 65 507 के बीच रखा गया |
| `rate` | संख्या | — | प्रति सेकंड यूनिट, एक शेड्यूल पर: यूनिट *n* शुरू से *n* / `rate` सेकंड पर देय है, और हर wake जो देय है वह भेजता है (अधिकतम 256; इससे पीछे का शेड्यूल पुरानी यूनिट छोड़ देता है); 0 जितनी तेज़ हो सके भेजता है |
| `duration_s` | संख्या | `0` | इतने सेकंड बाद रोकें; 0: रोके जाने तक |

**नतीजा**: [`JobInfo`](#type-jobinfo)।

**त्रुटियाँ**: `transport.target_invalid`, `transport.dns`। विफल भेजे गए इवेंट में
गिने जाते हैं, त्रुटियों के रूप में नहीं बताए जाते।

### scan_start {#scan_start}

एक रेंज के हर पोर्ट से TCP कनेक्शन आज़माता है और खुले वाले बताता है,
पूछे जाने पर सेवा पहले जो कहती है उसके साथ। खुले पोर्ट
[`scan://open`](events.md#event-scan-open) के रूप में आते हैं, प्रगति
[`scan://progress`](events.md#event-scan-progress) के रूप में। *जॉब शुरू करता है* (`scan`,
`params.host`, `params.from`, `params.to`)।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `config` | ऑब्जेक्ट | हाँ | नीचे |

| `config` का फ़ील्ड | प्रकार | डिफ़ॉल्ट | अर्थ |
| --- | --- | --- | --- |
| `host` | स्ट्रिंग | — | एक होस्ट नाम या एक पता |
| `port_start`, `port_end` | संख्या | — | रेंज, दोनों शामिल; उलटे दिए जाने पर वे बदल दिए जाते हैं |
| `concurrency` | संख्या | `256` | एक साथ प्रयास, 1 से 1024 |
| `timeout_ms` | संख्या | `600` | प्रति पोर्ट, 50 से 10 000 |
| `grab_banner` | बूलियन | `false` | जुड़ने के 400 ms के भीतर सेवा जो अधिकतम 256 बाइट भेजे उन्हें पढ़ें |

**नतीजा**: [`JobInfo`](#type-jobinfo)।

**त्रुटियाँ**: `scan.host_required`।

```bash
invoke scan_start '{"config":{"host":"127.0.0.1","port_start":8000,"port_end":9100,"grab_banner":true}}'
```

## इंस्पेक्टर {#inspector}

कैप्चर चालू रहते इंस्पेक्टर दर्ज करता है कि टूल क्या भेजते और प्राप्त करते हैं, फ़्रेम के रूप में।
सर्वर पर हर पेज और स्क्रिप्ट के लिए एक इंस्पेक्टर होता है। देखें
[इंस्पेक्टर](../tools/inspector.md)।

### inspect_set_enabled {#inspect_set_enabled}

कैप्चर arm या disarm करता है। disarmed रहते कुछ दर्ज नहीं होता।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `enabled` | बूलियन | हाँ | Arm (`true`) या disarm |

**नतीजा**: [`CaptureStats`](#type-frame)।

### inspect_stats {#inspect_stats}

कैप्चर के काउंटर। कोई आर्ग्युमेंट नहीं।

**नतीजा**: [`CaptureStats`](#type-frame)।

### inspect_snapshot {#inspect_snapshot}

सबसे नए फ़्रेम, सबसे पुराना पहले।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `limit` | संख्या | हाँ | कितने, 1 से 8192 |

**नतीजा**: [`Frame`](#type-frame)`[]`, उनके बाइट के बिना (देखें
[`inspect_payload`](#inspect_payload))।

### inspect_clear {#inspect_clear}

कैप्चर और उसके काउंटर ख़ाली करता है। कोई आर्ग्युमेंट नहीं।

**नतीजा**: [`CaptureStats`](#type-frame)।

### inspect_export {#inspect_export}

रखे गए हर फ़्रेम को डेटा फ़ोल्डर में `capture-<ms>.jsonl` या `capture-<ms>.txt` पर
लिखता है। `jsonl` में हर पंक्ति एक फ़्रेम होती है, जिसमें वह जो बाइट रखता है वे
`data` में, base64; `txt` पढ़ने के लिए है, हर फ़्रेम के hex dump के साथ।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `format` | स्ट्रिंग | हाँ | `txt`; कुछ और `jsonl` लिखता है |

**नतीजा**: स्ट्रिंग, लिखा गया पथ।

**त्रुटियाँ**: `inspect.empty`, `file.io`।

### inspect_payload {#inspect_payload}

एक फ़्रेम जो बाइट रखता है, उसके batch द्वारा लाए गए 1 KiB preview के आगे।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `seq` | संख्या | हाँ | फ़्रेम की संख्या |

**नतीजा**: `{ "seq", "bytes", "kept", "dump", "hex" }` — `bytes` फ़्रेम का
आकार, `kept` उनमें से कितने रखे गए (अधिकतम 256 KiB), `dump` हर पंक्ति
`offset  hex  |ascii|` के रूप में, `hex` वह सादा hex जो एक replay भेजता है।

**त्रुटियाँ**: `inspect.frame_gone` (नए फ़्रेम ने इसकी जगह ले ली),
`inspect.no_payload` (केवल इसका आकार दर्ज हुआ था)।

```bash
invoke inspect_set_enabled '{"enabled":true}'
invoke inspect_snapshot '{"limit":20}' | jq '.[] | {seq, proto, dir, summary}'
```

## सिग्नल लाइब्रेरी {#signals}

लाइब्रेरी डेटा फ़ोल्डर में `signals.json` है। यह केवल भंडारण है: एक सिग्नल
अपने transport की कमांड (`osc_send`, `broadcast_send`,
`http_request`, `mqtt_publish` या `mqtt_publish_once`) से भेजा जाता है। देखें
[सिग्नल](../tools/signals.md) और [फ़ाइलें](../reference/files.md#signals-json)।

### signals_load {#signals_load}

लाइब्रेरी पढ़ता है। जब फ़ाइल नहीं होती, तो पहले शुरुआती सेट लिखा जाता है। कोई आर्ग्युमेंट नहीं।

**नतीजा**: `{ "path": string, "library": library, "seeded": boolean }` —
`seeded` true होता है जब शुरुआती सेट अभी लिखा गया। लाइब्रेरी
`{ "version", "signals": [...], "folders": [...] }` है: `version` 2 (एक वर्शन 1
फ़ाइल जैसी है वैसी लौटती है), `folders` कोई न हो तो छोड़ दिया जाता है। हर सिग्नल
में `id`, `name`, `group` (इसका फ़ोल्डर, `"A/B"`; कोई नहीं तो ख़ाली), `note` और
`body` होते हैं।

**त्रुटियाँ**: `signals.json_invalid` (`path`, `line`, `column` के साथ; फ़ाइल कभी
बदली नहीं जाती), `file.io`।

### signals_save {#signals_save}

पूरी लाइब्रेरी फ़ाइल को बदल देता है, उसी फ़ोल्डर में एक अस्थायी फ़ाइल के ज़रिए।
जो फ़ाइल मौजूद है पर लाइब्रेरी के रूप में नहीं पढ़ी जाती, उसे जैसी है वैसी छोड़ दिया जाता है।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `library` | ऑब्जेक्ट | हाँ | `{ version, signals, folders }` जैसा `signals_load` लौटाता है |

**नतीजा**: स्ट्रिंग, लिखा गया पथ।

**त्रुटियाँ**: `signals.json_invalid` (अब डिस्क पर मौजूद फ़ाइल नहीं पढ़ी जाती,
`path`, `line`, `column` के साथ; कुछ लिखा नहीं जाता), `signals.encode`, `file.io`।

एक सिग्नल का `body`, उसके `transport` के अनुसार:

| `transport` | फ़ील्ड |
| --- | --- |
| `osc` | `target`, `address`, `args` ([`OscArg`](#type-oscarg)`[]`) |
| `udp` | `target`, `payload`: `{ "kind": "text", "text" }` या `{ "kind": "hex", "hex" }` |
| `http` | `request` ([`HttpRequest`](#type-httprequest)) |
| `mqtt` | `broker` (`host:port`), `topic`, `payload`, `qos`, `retain` |

```bash
invoke signals_load | jq '.library.signals[] | {name, transport: .body.transport}'
```

## एमुलेटर {#emulators}

एमुलेटर में Signal Lab दूसरा पक्ष बनकर खेलता है: एक HTTP API, एक OSC, UDP या
TCP डिवाइस, एक MQTT ब्रोकर। इसका दस्तावेज़ — `name`, `bind`, `protocol`,
प्रोटोकॉल के नियम और एक वैकल्पिक `outage` — का वर्णन
[एमुलेटर](../tools/emulators.md) में है। लाइब्रेरी डेटा फ़ोल्डर में `emulators.json` है।

### emulators_load {#emulators_load}

एमुलेटर लाइब्रेरी पढ़ता है। जब फ़ाइल नहीं होती, तो पहले शुरुआती सेट
लिखा जाता है। कोई आर्ग्युमेंट नहीं।

**नतीजा**: `{ "path", "library": { "version": 1, "emulators": [{ "id", "note", "emulator" }] }, "seeded" }`।

**त्रुटियाँ**: `emulators.json_invalid` (`path`, `line`, `column` के साथ; कभी
बदली नहीं जाती), `file.io`।

### emulators_save {#emulators_save}

पूरी एमुलेटर लाइब्रेरी को बदल देता है, एक अस्थायी फ़ाइल के ज़रिए।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `library` | ऑब्जेक्ट | हाँ | `{ version, emulators }` जैसा `emulators_load` लौटाता है |

**नतीजा**: स्ट्रिंग, लिखा गया पथ।

**त्रुटियाँ**: `emulators.encode`, `file.io`।

### emulator_check {#emulator_check}

क्या कोई एमुलेटर शुरू होगा: वह सब कुछ जो [`emulator_start`](#emulator_start)
bind करने से पहले जाँचता है।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `emulator` | ऑब्जेक्ट | हाँ | एमुलेटर दस्तावेज़ |
| `params` | स्ट्रिंग का ऑब्जेक्ट | नहीं | वे मान जो इसके टेम्पलेट पैरामीटर के रूप में पढ़ते हैं |

**नतीजा**: null जब यह शुरू होगा।

**त्रुटियाँ**: `emulator.*`; किसी फ़ील्ड के ग़ायब, सीमा से बाहर, बहुत लंबा
या ख़राब होने पर `node.*` (`node.required`, `node.range`, `node.too_long`,
`node.bind_invalid`, `node.target_invalid`, `node.method_invalid`…);
`param.unknown`, `template.*`, `osc.pattern_*`, `regex.invalid`,
`hex.invalid`। जब समस्या उनमें से किसी में हो तो हर एक में `params` में `rule`,
`retained` या `response` होता है।

### emulator_start {#emulator_start}

एक एमुलेटर को अपना अलग जॉब बनाकर शुरू करता है। कमांड लौटने पर इसका सॉकेट
खुला होता है। यह जो प्राप्त करता और जवाब देता है वह
[`emulator://activity`](events.md#event-emulator-activity) के रूप में हर 200 ms आता है जब
कुछ बदला हो। *जॉब शुरू करता है* (`emulator`, `params.name`,
`params.protocol`, `params.local`, और जब `source` दिया हो तो `params.source`)।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `emulator` | ऑब्जेक्ट | हाँ | एमुलेटर दस्तावेज़ |
| `params` | स्ट्रिंग का ऑब्जेक्ट | नहीं | वे मान जो इसके टेम्पलेट पैरामीटर के रूप में पढ़ते हैं |
| `seed` | संख्या | नहीं | इसका सीड, 0 से 9007199254740991; डिफ़ॉल्ट: एक नया |
| `source` | स्ट्रिंग | नहीं | जिस लाइब्रेरी प्रविष्टि से यह आता है, जॉब पर `params.source` के रूप में रखा जाता है |

**नतीजा**: [`JobInfo`](#type-jobinfo)।

**त्रुटियाँ**: [`emulator_check`](#emulator_check) वाली, `seed.range`,
`transport.address_in_use` और बाकी bind विफलताएँ।

### emulator_exchanges {#emulator_exchanges}

एक चलते एमुलेटर ने जो प्राप्त किया और जवाब दिया। यह अंतिम 500
आदान-प्रदान रखता है।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `jobId` | संख्या | हाँ | एमुलेटर का जॉब |
| `after` | संख्या | नहीं | केवल इससे ऊपर संख्या वाले आदान-प्रदान; डिफ़ॉल्ट 0 |
| `limit` | संख्या | नहीं | अधिकतम इतने, 1 से 500; डिफ़ॉल्ट 500 |

**नतीजा**

| फ़ील्ड | प्रकार | अर्थ |
| --- | --- | --- |
| `job_id` | संख्या | जॉब |
| `name`, `protocol`, `local` | स्ट्रिंग | एमुलेटर, इसका प्रोटोकॉल और वह पता जिस पर यह सुनता है |
| `counts` | ऑब्जेक्ट | `total`, `unmatched` (किसी नियम ने नहीं लिया), `failed`, `down` (जब यह बंद था तब आया: इसका outage या [`emulator_down`](#emulator_down)), `hits` (प्रति नियम), और `missed` (MQTT: ऐसे संदेश जो बहुत पीछे वाले क्लाइंट को नहीं मिले; 0 रहते छोड़ दिया जाता है) |
| `forced` | स्ट्रिंग | जब यह नीचे रखा हो तो `unavailable`, `reset` या `timeout`; वरना छोड़ दिया जाता है |
| `exchanges` | ऑब्जेक्ट[] | हर एक: `seq`, `ts`, `from`, `request`, `rule` (1-आधारित; किसी ने न लिया हो तो छोड़ा गया), `reply`, `status`, `fault`, `ms`, `error`, `frame`, `down`, और `data` (जिस रूप में टेम्पलेट इसे पढ़ते हैं वह request) |

**त्रुटियाँ**: `emulator.not_running`।

### emulator_down {#emulator_down}

एक चलते एमुलेटर को तब तक नीचे कर देता है जब तक उसे ऊपर न लाया जाए, चाहे उसका outage
शेड्यूल कुछ भी कहे, या उसे वापस ले आता है। नीचे रहते, एक HTTP एमुलेटर हर
रिक्वेस्ट का सामना `fault` से करता है, एक TCP डिवाइस और एक MQTT ब्रोकर अपने कनेक्शन तोड़ देते हैं
और नए अस्वीकार करते हैं, और OSC तथा UDP डिवाइस कुछ जवाब नहीं देते।

| आर्ग्युमेंट | प्रकार | आवश्यक | अर्थ |
| --- | --- | --- | --- |
| `jobId` | संख्या | हाँ | एमुलेटर का जॉब |
| `down` | बूलियन | हाँ | नीचे (`true`) या ऊपर |
| `fault` | स्ट्रिंग | नहीं | HTTP रिक्वेस्ट का सामना किससे: `unavailable` (503, `Retry-After` के बिना: कब लौटेगा पता नहीं), `reset` (कनेक्शन बंद हो जाता है), `timeout` (कोई जवाब नहीं); डिफ़ॉल्ट `unavailable` |

**नतीजा**: null।

**त्रुटियाँ**: `emulator.not_running`।

```bash
invoke emulator_exchanges '{"jobId":5,"after":0}' | jq '.counts, (.exchanges[] | {request, rule, status})'
```

## साझा प्रकार {#types}

### JobInfo {#type-jobinfo}

जो कमांड एक जॉब शुरू करती है वह क्या लौटाती है, और [`jobs_list`](#jobs_list)
क्या सूचीबद्ध करता है: `id`, `kind`, `label` (अंग्रेज़ी, लॉग के लिए), `params` (जिन मानों का
नाम label देता है; कोई न हो तो छोड़ दिया जाता है) और `started_ms`। देखें
[जॉब](index.md#jobs)।

### OscArg {#type-oscarg}

एक OSC आर्ग्युमेंट, इसका प्रकार और इसका मान:

| `type` | `value` | OSC tag |
| --- | --- | --- |
| `int` | 32-बिट पूर्णांक | `i` |
| `float` | संख्या, 32-बिट float के रूप में भेजी गई | `f` |
| `str` | स्ट्रिंग | `s` |
| `long` | 64-बिट पूर्णांक | `h` |
| `double` | संख्या, 64-बिट | `d` |
| `bool` | `true` या `false` | `T` या `F` |
| `blob` | बाइट की सरणी, `[222, 173]` | `b` |
| `nil` | कोई नहीं: `{ "type": "nil" }` | `N` |

### HttpRequest {#type-httprequest}

| फ़ील्ड | प्रकार | डिफ़ॉल्ट | अर्थ |
| --- | --- | --- | --- |
| `method` | स्ट्रिंग | — | `GET`, `POST`… |
| `url` | स्ट्रिंग | — | `http://` या `https://` |
| `headers` | `[name, value][]` | `[]` | रिक्वेस्ट हेडर |
| `body` | स्ट्रिंग या null | null | बॉडी |
| `timeout_ms` | संख्या | `10000` | पूरे आदान-प्रदान के लिए |
| `auth` | ऑब्जेक्ट | कोई नहीं | `{ "scheme": "basic", "username", "password" }`, `{ "scheme": "digest", "username", "password" }` या `{ "scheme": "bearer", "token" }` |

10 redirects तक अनुसरण किए जाते हैं। एक होस्ट के लिए टाइप किए गए credentials और
कुकीज़ कभी दूसरे पर नहीं जाते। एक Digest रिक्वेस्ट सर्वर की 401 challenge का जवाब
देती है और फिर भेजती है।

### HttpResponse {#type-httpresponse}

| फ़ील्ड | प्रकार | अर्थ |
| --- | --- | --- |
| `ok` | बूलियन | एक 2xx स्टेटस |
| `status`, `status_text` | संख्या, स्ट्रिंग | स्टेटस; बिना रिस्पॉन्स 0 और ख़ाली |
| `latency_ms` | संख्या | पूरी बॉडी आने तक |
| `headers` | `[name, value][]` | रिस्पॉन्स हेडर |
| `body` | स्ट्रिंग | बॉडी टेक्स्ट के रूप में, अधिकतम 256 KiB |
| `body_bytes` | संख्या | बॉडी का पूरा आकार |
| `truncated` | बूलियन | `body` 256 KiB पर काट दी गई |
| `error` | स्ट्रिंग या null | रिस्पॉन्स क्यों नहीं था, कारण की हर परत |
| `cause` | स्ट्रिंग या null | किस तरह की विफलता: `refused`, `timeout`, `dns`, `unreachable`, `reset`, `address_in_use`, `address_unavailable`, `denied`, `tls`, `target_invalid`, `failed` — `transport.*` कोड जैसे |
| `digest` | ऑब्जेक्ट | केवल 401 से मिली एक Digest रिक्वेस्ट; वरना छोड़ दिया जाता है। `challenged`: challenge का जवाब दिया गया और रिक्वेस्ट फिर भेजी गई। `error`: ऐसा क्यों नहीं हो सका, एक `EngineError` (`http.digest_not_offered`, `http.digest_unsupported`, `http.digest_invalid`, `http.digest_other_origin`) या null |

### Payload {#type-payload}

एक broadcast या discovery डेटाग्राम क्या ले जाता है:
`{ "kind": "osc", "address", "args" }`, `{ "kind": "text", "text" }` (जैसा है वैसा
भेजा जाता है, कोई terminating zero नहीं) या `{ "kind": "hex", "hex" }` (`de ad be ef`,
`deadbeef`, `0xDE,0xAD` — hex digits के अलावा कुछ भी नज़रअंदाज़ किया जाता है)।

### MqttConfig {#type-mqttconfig}

| फ़ील्ड | प्रकार | डिफ़ॉल्ट | अर्थ |
| --- | --- | --- | --- |
| `host` | स्ट्रिंग | — | ब्रोकर का नाम या पता |
| `port` | संख्या | — | आम तौर पर 1883 |
| `client_id` | स्ट्रिंग | — | ख़ाली नहीं; वही id वाला दूसरा कनेक्शन ब्रोकर द्वारा गिरा दिया जाता है |
| `username`, `password` | स्ट्रिंग | ख़ाली | `username` ख़ाली न हो तो भेजा जाता है; `password` केवल `username` के साथ |
| `keep_alive_s` | संख्या | `60` | पिंग इसके आधे पर जाते हैं; 0: कोई नहीं |
| `clean_session` | बूलियन | `true` | CONNECT फ़्लैग |
| `will` | ऑब्जेक्ट या null | null | `{ topic, payload, qos, retain }`, कनेक्शन टूटने पर ब्रोकर द्वारा पब्लिश किया जाता है |
| `subscribe` | `{ filter, qos }[]` | `[]` | कनेक्शन बनते ही सब्सक्राइब कर दिया जाता है |

### WsConfig {#type-wsconfig}

| फ़ील्ड | प्रकार | डिफ़ॉल्ट | अर्थ |
| --- | --- | --- | --- |
| `url` | स्ट्रिंग | — | `ws://` या `wss://` (`wss://` वही मानता है जो सिस्टम HTTPS के लिए मानता है) |
| `headers` | `[name, value][]` | `[]` | upgrade रिक्वेस्ट के साथ भेजे जाते हैं |
| `protocols` | स्ट्रिंग[] | `[]` | वरीयता क्रम में दिए जाने वाले subprotocols |
| `timeout_ms` | संख्या | `10000` | कनेक्शन, TLS और upgrade सबके लिए मिलाकर |

संदेश किसी भी दिशा में अधिकतम 16 MiB होते हैं।

### ImpairProfile {#type-impairprofile}

हर फ़ील्ड वैकल्पिक है; जो छोड़ा गया वह कुछ नहीं करता। प्रायिकताएँ 0
से 1 तक।

| फ़ील्ड | सीमा | अर्थ | UDP | TCP |
| --- | --- | --- | --- | --- |
| `name` | अधिकतम 60 अक्षर | टाइमलाइन और रिपोर्ट के लिए एक नाम | हाँ | हाँ |
| `latency_ms` | 0 से 60 000 | हर चीज़ में जोड़ी जाने वाली देर | हाँ | हाँ |
| `jitter_ms` | 0 से 60 000 | हर बार निकालकर इतना और तक | हाँ | हाँ |
| `loss` | 0 से 1 | एक डेटाग्राम गिरा दिया जाता है | हाँ | — |
| `duplicate` | 0 से 1 | एक डेटाग्राम दो बार भेजा जाता है | हाँ | — |
| `corrupt` | 0 से 1 | डेटाग्राम का एक बिट पलट दिया जाता है | हाँ | — |
| `reorder` | 0 से 1 | एक डेटाग्राम रोक रखा जाता है ताकि बाद वाले आगे निकल जाएँ | हाँ | — |
| `rate_kbps` | 0, या 8 से 10 000 000 | बैंडविड्थ सीमा, किलोबिट प्रति सेकंड; 0: कोई नहीं | हाँ | हाँ |
| `burst_start` | 0 से 1 | एक डेटाग्राम नुकसानों का burst शुरू करता है | हाँ | — |
| `burst_length` | 1 से 1000 | औसतन एक burst कितने डेटाग्राम चलता है (`burst_start` के साथ ज़रूरी) | हाँ | — |
| `offline` | `true` या `false` | कुछ भी नहीं निकलता | हाँ | हाँ |
| `reset` | 0 से 1 | stream का एक हिस्सा अपना कनेक्शन reset कर देता है | — | हाँ |
| `stall` | 0 से 1 | stream का एक हिस्सा अपना कनेक्शन half-open छोड़ देता है | — | हाँ |

### Frame और CaptureStats {#type-frame}

एक `Frame` एक कैप्चर किया गया पैकेट, रिक्वेस्ट या संदेश है:

| फ़ील्ड | अर्थ |
| --- | --- |
| `seq` | इसकी संख्या, बढ़ती हुई |
| `ts` | कब, 1970 से मिलीसेकंड |
| `proto` | `osc`, `udp`, `tcp`, `http`, `mqtt`, `ws`… |
| `dir` | `tx` (भेजा) या `rx` (प्राप्त) |
| `source` | वह टूल जिसने इसे कैप्चर किया: `osc-monitor`, `broadcast`, `netsim`… |
| `job_id` | इसका जॉब, या null |
| `local`, `remote` | पते: इस पक्ष और दूसरे पक्ष के `IP:port` (HTTP, WebSocket और MQTT के लिए एक URL या ब्रोकर)। एक relayed फ़्रेम का `local` वह पता है जिस पर रिले सुनता है और इसका `remote` वह जहाँ फ़्रेम जा रहा था; leg अपना `verdict` समाप्त करता है (`· client→target`, `· target→client`) |
| `bytes` | इसका आकार |
| `summary` | एक पंक्ति |
| `detail` | कई पंक्तियों में एक डिकोड, या null |
| `hex` | पहले 1 KiB का hex dump, या null |
| `verdict` | इसका क्या हुआ — `dropped`, `sampled`, एक स्टेटस — या null |
| `kept` | `bytes` में से कितने रखे गए (अधिकतम 256 KiB); केवल आकार दर्ज होने पर 0 |
| `publish` | केवल एक MQTT publish: `{ broker, topic, qos, retain, text }` — ब्रोकर `host:port` के रूप में, और क्या रखे गए बाइट, जो संदेश का पेलोड हैं, UTF-8 टेक्स्ट हैं। हर दूसरे फ़्रेम के लिए अनुपस्थित |

`CaptureStats`: `enabled`, `total` (दर्ज किए गए फ़्रेम), `bytes`, `skipped`
(दर्ज हुए पर इंटरफ़ेस को कभी नहीं भेजे गए), `buffered` (रखे गए फ़्रेम),
`capacity` (8192), `held` (रखे गए पेलोड बाइट) और `held_limit` (64 MiB)। किसी भी
सीमा के आगे सबसे पुराने फ़्रेम हट जाते हैं।

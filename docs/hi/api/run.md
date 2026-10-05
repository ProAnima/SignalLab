---
title: रन
description: POST /api/run किसी Signal Lab सर्वर पर एक प्रयोग चलाता है और उसके नतीजे के साथ जवाब देता है, या उसके चरण NDJSON पंक्तियों के रूप में स्ट्रीम करता है।
---

# प्रयोग चलाना

किसी स्क्रिप्ट या पाइपलाइन से सर्वर पर प्रयोग चलाकर यह जानने के लिए कि वह कैसा रहा, उसे
`POST /api/run` पर भेजें। सर्वर उसे अंत तक चलाता है और नतीजे के साथ जवाब देता है — या, यदि आप
माँगें, तो हर चरण के साथ जैसे-जैसे वह होता है। यही
[`signallab run --server`](../automation/cli.md#cli-run) उपयोग करता है।

यह एडिटर के [[ui:exp.run]] और
[`experiment_start`](commands.md#experiment_start) वाला ही रन है: ऐसा जॉब जिसे हर खुला पेज देखता है
और रोक सकता है, वही इवेंट, डेटा फ़ोल्डर में वही रिपोर्ट।

## रिक्वेस्ट {#request}

```http
POST /api/run
Authorization: Bearer <token>
Content-Type: application/json

{ "template": "osc-ping-reply", "overrides": { "device": "192.0.2.20:9000" }, "seed": 42, "timeout": 30 }
```

बॉडी एक प्रयोग नाम देती है — एक `document` या एक `template`, दोनों नहीं — और यह कि उसे किसके साथ
चलाना है:

| फ़ील्ड | प्रकार | डिफ़ॉल्ट | अर्थ |
| --- | --- | --- | --- |
| `document` | ऑब्जेक्ट | — | एक प्रयोग जैसे एडिटर उसे सहेजता और एक्सपोर्ट करता है। पुराने वर्शन माइग्रेट किए जाते हैं, जैसे फ़ाइल खोलने पर |
| `template` | स्ट्रिंग | — | फ़ाइल नाम से बंडल किया टेम्पलेट, `.json` के साथ या बिना (नीचे) |
| `overrides` | ऑब्जेक्ट | `{}` | केवल इस रन के लिए पैरामीटर मान। मान स्ट्रिंग, संख्याएँ या बूलियन हो सकते हैं; हर एक प्रयोग का पैरामीटर होना चाहिए |
| `profile` | स्ट्रिंग | दस्तावेज़ का | इस प्रोफ़ाइल के साथ चलाएँ; `""` डिफ़ॉल्ट के साथ चलाता है |
| `seed` | संख्या | दस्तावेज़ का, वरना नया | 0 से 9007199254740991; वही सीड वही रैंडम मान निकालता है |
| `timeout` | संख्या | `300` | रन के `run.timeout` के साथ विफल होने से पहले सेकंड; 1 से 300 |

सर्वर जिस फ़ील्ड को नहीं जानता उसे अस्वीकार कर देता है (`400`, `api.run_invalid`)। दस्तावेज़ ख़ुद खोली
गई फ़ाइल की तरह पढ़ा जाता है, इसलिए वह अधिकतम 4 MiB का होता है।

बंडल किए गए टेम्पलेट — वे प्रयोग जो एडिटर [[ui:exp.templates]] के नीचे देता है:

| `template` | यह क्या है |
| --- | --- |
| `empty` | Start और End |
| `http-check` | `http://127.0.0.1:8080/` का GET, फिर स्टेटस 200 की जाँच |
| `status-branch` | `http://127.0.0.1:8080/` का GET; 200 पर एक OSC संदेश, वरना 500 ms विलंब |
| `parallel-flows` | `http://127.0.0.1:8080/` के GET और एक लॉग प्रविष्टि में [[ui:exp.node.fork]], साथ-साथ, फिर [[ui:exp.node.join]] |
| `osc-ping-reply` | पैरामीटर `device` (`127.0.0.1:9000`) पर एक OSC `/ping`, फिर `127.0.0.1:9001` पर `/pong` के लिए 2 s की प्रतीक्षा |
| `poll-until-ready` | एक [[ui:exp.node.loop]] जो `device` से OSC पर `/status` पूछता है जब तक वह `ready` जवाब न दे, अधिकतम 10 बार |
| `flaky-api` | एक एमुलेट किया API (पैरामीटर `api`) जो काम करने से पहले विफल होता है, [[ui:exp.node.loop]] में पूछा जाता है जब तक 200 जवाब न दे |
| `fault-phases` | इम्पेयरमेंट रिले के पीछे एक UDP डिवाइस, 8 s तक भेजा जाता है जबकि रिले साफ़, लॉसी, ऑफ़लाइन और फिर साफ़ होता है |
| `dependency-outage` | एक एमुलेट किया API (पैरामीटर `api`) 2 s के लिए बंद किया जाता है जबकि एक [[ui:exp.node.loop]] उससे पूछता है जब तक वह फिर 200 जवाब न दे |
| `websocket-echo` | पैरामीटर `service` (`ws://127.0.0.1:9001/echo`) से एक कनेक्शन, एक संदेश, उसकी गूँज की प्रतीक्षा, एक जाँच, एक बंद |

किसी एक को [[ui:exp.templates]] के नीचे खोलकर उसके नोड और पैरामीटर देखें; देखें
[प्रयोग](../experiments/index.md)।

## नतीजा {#result}

डिफ़ॉल्ट रूप से जवाब `200` है, `Content-Type: application/json` के साथ, रन समाप्त होने पर भेजा
जाता है: एक JSON ऑब्जेक्ट, रन का नतीजा।

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

| फ़ील्ड | प्रकार | अर्थ |
| --- | --- | --- |
| `job_id` | संख्या | रन का जॉब |
| `experiment` | स्ट्रिंग | प्रयोग का नाम |
| `outcome` | स्ट्रिंग | `passed`, `failed` या `stopped` |
| `seed` | संख्या | जिस सीड के साथ यह चला: वही मान निकालने के लिए इसे `seed` के रूप में वापस दें |
| `profile` | स्ट्रिंग या null | जिस प्रोफ़ाइल के साथ यह चला |
| `overridden` | बूलियन | कुछ मान `overrides` से आए |
| `params` | ऑब्जेक्ट | रन द्वारा उपयोग किए हर पैरामीटर का मान |
| `started_ms`, `ended_ms` | संख्या | 1970 से मिलीसेकंड |
| `error` | `EngineError` | यह क्यों विफल हुआ: इसकी पहली विफलता। पास होने पर छोड़ दिया जाता है |
| `steps` | ऑब्जेक्ट[] | हर चरण उस क्रम में जिसमें वह हुआ, [`experiment://step`](events.md#event-experiment-step) की तरह |
| `emulators` | ऑब्जेक्ट[] | हर [[ui:exp.node.emulator]] नोड ने क्या पाया और जवाब दिया: `node`, `name`, `protocol`, `local`, `counts`। न हों तो छोड़ दिया जाता है |
| `impairments` | ऑब्जेक्ट[] | हर [[ui:exp.node.impairment]] नोड के रिले ने क्या किया, चरण-दर-चरण। न हों तो छोड़ दिया जाता है |
| `report_path` | स्ट्रिंग | सर्वर पर रन की रिपोर्ट; इसे [`/api/files`](index.md#files) से डाउनलोड करें। कोई न लिखी गई हो तो छोड़ दिया जाता है |
| `report_error` | `EngineError` | रिपोर्ट क्यों नहीं लिखी जा सकी। वरना छोड़ दिया जाता है |

इसमें हर जगह सीक्रेट मान मास्क होते हैं। रिपोर्ट फ़ाइल में वही चरण होते हैं; देखें
[रन और रिपोर्ट](../experiments/runs.md)।

जब तक रन चलता है, सर्वर हर 15 s में एक स्पेस भेजता है। JSON किसी मान से पहले का व्हाइटस्पेस
नज़रअंदाज़ करता है, इसलिए नतीजा फिर भी पार्स होता है, और कोई प्रॉक्सी लंबे, शांत रन को मृत कनेक्शन
नहीं समझता।

## चरणों का पीछा करना {#lines}

चरणों को होते हुए देखने के लिए, NDJSON माँगें:

```http
Accept: application/x-ndjson
```

जवाब `200` है, `Content-Type: application/x-ndjson` के साथ: हर पंक्ति में एक JSON ऑब्जेक्ट, हर एक में
एक `type`.

| `type` | कब | पंक्ति का बाकी हिस्सा |
| --- | --- | --- |
| `started` | पहले, एक बार | `job_id`, `experiment`, `seed`, `profile`, `overridden`, `started_ms` |
| `step` | हर चरण | चरण, [`experiment://step`](events.md#event-experiment-step) की तरह |
| `heartbeat` | हर 15 s | कुछ नहीं |
| `ended` | अंत में, एक बार | नतीजा, जैसा ऊपर |

```text
{"job_id":12,"experiment":"OSC ping → reply","seed":42,"profile":null,"overridden":true,"started_ms":1759600000000,"type":"started"}
{"job_id":12,"ts":1759600000001,"node_id":"start","state":"running","detail":"","message_key":null,"message_params":null,"type":"step"}
…
{"job_id":12,"experiment":"OSC ping → reply","outcome":"passed",…,"type":"ended"}
```

पंक्तियाँ तब तक पढ़ें जब तक `ended` न आए; जो `type` आप नहीं जानते उसे नज़रअंदाज़ करें। सर्वर
`X-Accel-Buffering: no` भेजता है, इसलिए nginx प्रॉक्सी हर पंक्ति तुरंत आगे भेज देता है।

## स्टेटस और नतीजा {#status}

HTTP त्रुटि स्टेटस का मतलब है **कोई रन शुरू नहीं हुआ**; बॉडी एक [`EngineError`](index.md#errors) है:

| स्टेटस | कोड | क्यों |
| --- | --- | --- |
| `400` | `api.run_invalid` | बॉडी रन रिक्वेस्ट नहीं है: JSON नहीं, अज्ञात फ़ील्ड, ऐसा override जो स्ट्रिंग, संख्या या बूलियन नहीं |
| `400` | `api.run_source` | न `document` न `template`, या दोनों |
| `415` | `command.json_required` | `Content-Type: application/json` नहीं |
| `422` | `api.template_unknown` | उस नाम का कोई बंडल किया टेम्पलेट नहीं |
| `422` | `file.json_invalid`, `file.too_large`, `doc.*` | दस्तावेज़ पढ़ा नहीं जा सकता |
| `422` | कोई भी वैलिडेशन कोड, `run.override_unknown`, `profile.active_missing`, `run.limit_range`, `seed.range`, `secret.missing`, `transport.address_in_use`… | प्रयोग शुरू नहीं हो सकता: वह वैलिड नहीं है, कोई मान रेंज से बाहर है, कोई सीक्रेट संग्रहित नहीं है, जिस पोर्ट पर वह सुनता है वह लिया हुआ है |

रन शुरू होने के बाद चाहे जो हो स्टेटस `200` रहता है: नतीजे में `outcome` पढ़ें।

| `outcome` | अर्थ |
| --- | --- |
| `passed` | हर चरण पास हुआ और [[ui:exp.node.end]] तक पहुँचा गया |
| `failed` | कोई चरण विफल हुआ, या रन अपने `timeout` (`run.timeout`) से ज़्यादा लंबा चला; `error` बताता है कि कौन-सा और क्यों |
| `stopped` | यह समाप्त होने से पहले रोका गया: `job_stop`, [[ui:app.stopAll]], या सर्वर के बंद होने से। `steps` में वे चरण हैं जहाँ तक यह पहुँचा; कोई रिपोर्ट सहेजी नहीं जाती |

## वह क्लाइंट जो चला जाता है {#disconnect}

कनेक्शन बंद करने से रन नहीं रुकता। यह सर्वर पर एक जॉब है: यह अंत तक चलता है और अपनी रिपोर्ट
सहेजता है, जैसे ब्राउज़र में शुरू किया रन टैब बंद होने पर करता है। इसे
[`jobs_list`](commands.md#jobs_list) से खोजें, [`job_stop`](commands.md#job_stop) से रोकें, और बाद
में इसकी रिपोर्ट [`experiment_runs`](commands.md#experiment_runs) से पढ़ें। सर्वर बंद होने पर रन रोका
जाता है और अब भी जुड़ा क्लाइंट `"outcome": "stopped"` पाता है।

## उदाहरण {#examples}

बंडल किया टेम्पलेट चलाएँ और नतीजे की प्रतीक्षा करें:

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
curl -sS -X POST "$SERVER/api/run" \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"template":"osc-ping-reply","overrides":{"device":"192.0.2.20:9000"},"timeout":30}' \
  | jq -r .outcome
```

अपना प्रयोग एक पैरामीटर बदलकर भेजें, और हर चरण को होते हुए छापें:

```bash
jq '{document: ., overrides: {api: "http://192.0.2.10:8080"}, profile: ""}' smoke.json |
  curl -sSN -X POST "$SERVER/api/run" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    -H "Accept: application/x-ndjson" --data @- |
  jq -r 'select(.type == "step") | "\(.node_id)  \(.state)  \(.detail)"'
```

`-N` `curl` को पंक्तियाँ रोके रखने से रोकता है। विफल रन पर पाइपलाइन विफल करने के लिए `outcome`
जाँचें:

```bash
outcome=$(curl -sS -X POST "$SERVER/api/run" -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" -d '{"template":"http-check"}' | jq -r .outcome)
[ "$outcome" = "passed" ]
```

## कमांड लाइन से {#cli}

[`signallab run`](../automation/cli.md#cli-run) `--server <url>` के साथ इस एंडपॉइंट के ज़रिए सर्वर पर
चलता है: यह जो प्रयोग पढ़ा उसे `overrides`, `seed` और `timeout` के साथ भेजता है, NDJSON माँगता है,
और हर चरण को उसकी पंक्ति आते ही छापता है। टोकन `--token-file` से आता है, वरना `SIGNALLAB_TOKEN`।
`--report` के साथ यह रिपोर्ट `/api/files` से डाउनलोड करता है। जो सर्वर 60 s तक कुछ नहीं भेजता —
हार्टबीट भी नहीं — वह गया हुआ माना जाता है।
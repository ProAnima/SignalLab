---
title: HTTP API
description: स्क्रिप्ट, CI और दूसरे टूल से Signal Lab सर्वर चलाएँ, उन्हीं कमांड और इवेंट के साथ जो उसका अपना इंटरफ़ेस उपयोग करता है।
---

# HTTP API

किसी स्क्रिप्ट, CI पाइपलाइन या दूसरे टूल से Signal Lab चलाने के लिए, किसी
Signal Lab सर्वर (`signal-lab-server`, या Docker इमेज) से HTTP पर बात करें। ब्राउज़र में सर्वर जो
इंटरफ़ेस दिखाता है, वह ठीक यही API उपयोग करता है: हर बटन `/api/invoke/<command>` पर एक
कॉल है, हर लाइव संख्या `/api/events` पर आती है। इसलिए सर्वर के पेज पर कोई व्यक्ति जो कुछ कर
सकता है, स्क्रिप्ट भी कर सकती है।

डेस्कटॉप ऐप का कोई HTTP API नहीं है: उसकी विंडो ऐप के भीतर ही अपने इंजन तक पहुँचती है। डेस्कटॉप
पर ऑटोमेशन के लिए, उसी मशीन पर एक सर्वर चलाएँ (देखें
[सर्वर](../server/index.md)) या कमांड लाइन
[`signallab`](../automation/cli.md) उपयोग करें।

## एंडपॉइंट {#endpoints}

| मेथड और पाथ | यह क्या करता है | टोकन |
| --- | --- | --- |
| `GET /api/health` | सर्वर जवाब देता है या नहीं, उसका वर्शन, क्या उसे टोकन चाहिए | ज़रूरी नहीं |
| `POST /api/invoke/<command>` | एक इंजन कमांड चलाता है: JSON आर्ग्युमेंट अंदर, JSON नतीजा बाहर ([कमांड](commands.md)) | ज़रूरी |
| `POST /api/run` | एक प्रयोग को उसके अंत तक चलाता है: नतीजा, या उसके चरण पंक्तियों के रूप में ([रन](run.md)) | ज़रूरी |
| `GET /api/events` | हर इंजन इवेंट का WebSocket ([इवेंट](events.md)) | ज़रूरी |
| `GET /api/files?path=…` | इंजन द्वारा डेटा फ़ोल्डर में लिखी एक फ़ाइल, डाउनलोड के रूप में | ज़रूरी |
| `GET /api/openapi.json` | यह API OpenAPI 3.1 में वर्णित | ज़रूरी |
| `GET /login`, `POST /login` | ब्राउज़र के लिए साइन-इन पेज और फ़ॉर्म | ज़रूरी नहीं |
| `POST /logout` | ब्राउज़र का सेशन समाप्त करता है | एक सेशन |

`/api/` के नीचे कोई भी दूसरा पाथ कोड `api.not_found` के साथ `404` जवाब देता है; जो मेथड कोई पाथ
नहीं लेता (`GET /api/invoke/…`) वह ख़ाली बॉडी के साथ `405` है। बाकी सब इंटरफ़ेस है; टोकन वाले
सर्वर पर बिना सेशन का ब्राउज़र पहले `/login` पर भेजा जाता है।

## बेस URL {#base-url}

जब तक `--listen` (या `SIGNALLAB_LISTEN`) से कुछ और न कहा जाए, सर्वर
`http://127.0.0.1:1430` पर सुनता है। Docker इमेज हर नेटवर्क कार्ड पर सुनती है,
`0.0.0.0:1430`। इन पेजों के उदाहरण उपयोग करते हैं:

```bash
SERVER=http://127.0.0.1:1430
```

सर्वर सादा HTTP बोलता है। HTTPS के लिए, उसके आगे TLS समाप्त करने वाला एक रिवर्स प्रॉक्सी रखें और
सर्वर को `--secure-cookie` के साथ शुरू करें।

## प्रमाणीकरण {#authentication}

बिना टोकन शुरू किया सर्वर केवल लूपबैक पर सुनता है और उसे प्रमाणीकरण की ज़रूरत नहीं: उस मशीन पर कोई
भी उसे उपयोग कर सकता है। जिस सर्वर तक दूसरे पहुँच सकते हैं, उसके पास हमेशा टोकन होता है, और तब
`/api/health` तथा `/login` के अलावा हर रिक्वेस्ट को उसे ले जाना होता है।

**स्क्रिप्ट** टोकन को `Authorization` हेडर में भेजती हैं:

```bash
TOKEN=$(cat token.txt)
curl -fsS "$SERVER/api/invoke/jobs_list" -X POST \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json"
```

हेडर ठीक `Bearer`, एक स्पेस, और टोकन होना चाहिए। टोकन न हो या ग़लत हो तो कोड `auth.required` के
साथ `401`।

**ब्राउज़र** टोकन के साथ एक बार `/login` पर साइन इन करते हैं और एक सेशन कुकी पाते हैं,
`signallab_session`: `HttpOnly`, `SameSite=Strict`, 7 दिन रखी जाती है, और जब सर्वर
`--secure-cookie` के साथ चलता है तो `Secure`। `POST /logout` इसे समाप्त करता है। साइन-इन फ़ॉर्म पर
ग़लत टोकन जवाब से पहले एक सेकंड लेता है, जिससे अनुमान लगाना धीमा हो जाता है। सेशन सर्वर की मेमोरी
में रहते हैं: रीस्टार्ट हर ब्राउज़र को साइन आउट कर देता है, जबकि टोकन वाली स्क्रिप्ट पर कोई असर
नहीं पड़ता। सर्वर अधिकतम 1024 सेशन रखता है; उससे आगे सबसे पुराना चला जाता है।

टोकन कहाँ से आता है, यह सर्वर की सेटअप है (`--token-file`, `SIGNALLAB_TOKEN`, या डेटा फ़ोल्डर में
`--generate-token` द्वारा बनाई गई `token` फ़ाइल): देखें
[सर्वर सुरक्षा](../server/security.md)। टोकन में कम से कम 24 अक्षर होते हैं और कोई स्पेस नहीं;
`signal-lab-server token` एक नया छापता है।

::: warning
टोकन वाला कोई भी सर्वर से ट्रैफ़िक भेजवा सकता है। जिस फ़ाइल में यह है, उसे केवल आप पढ़ सकें, और
इसे कभी URL में न रखें — सर्वर वहाँ से टोकन पढ़ता ही नहीं।
:::

## Host और Origin {#host-origin}

हर पाथ पर, `/api/health` समेत, सबसे पहले दो जाँचें चलती हैं।

**`Host`।** `Host` हेडर में इसी सर्वर का नाम होना चाहिए:

- लूपबैक नाम हमेशा पास होते हैं: `localhost`, `.localhost` पर समाप्त होने वाले नाम, `127.x.x.x`
  और `[::1]`।
- `--allowed-host` (या `SIGNALLAB_ALLOWED_HOSTS`) से दिए नाम पास होते हैं।
- टोकन वाला और बिना `--allowed-host` का सर्वर किसी भी नाम पर जवाब देता है।

बाकी कुछ भी `auth.host` के साथ `403` है। सर्वर को ऐसे नाम से संबोधित करें जो वह स्वीकार करता है:
`curl` उस URL का हॉस्ट भेजता है जो आप उसे देते हैं।

**`Origin`।** जो रिक्वेस्ट कुछ बदलती है (कोई भी मेथड `GET` और `HEAD` के अलावा) और WebSocket
अपग्रेड, जब वे `Origin` हेडर ले जाती हैं तो उन्हें सर्वर के अपने पेज से आना चाहिए: उसका हॉस्ट और
पोर्ट `Host` के बराबर होना चाहिए। वरना जवाब `auth.origin` के साथ `403` है; `Origin: null` भी
अस्वीकार किया जाता है। स्क्रिप्ट और `curl` कोई `Origin` नहीं भेजते और यह जाँच पास कर जाते हैं —
उन्हें फिर भी टोकन चाहिए।

**केवल JSON।** `POST /api/invoke/…` और `POST /api/run` `Content-Type: application/json` लेते हैं
(`; charset=utf-8` जैसे पैरामीटर ठीक हैं)। बाकी कुछ भी `command.json_required` के साथ `415` है।
किसी दूसरी साइट का वेब पेज सर्वर से पहले पूछे बिना यह नहीं भेज सकता, और सर्वर कभी सहमत नहीं होता।

## कमांड कॉल करना {#invoke}

```http
POST /api/invoke/<command>
Content-Type: application/json

{ "argument": "value", … }
```

- बॉडी कमांड के आर्ग्युमेंट वाला एक JSON ऑब्जेक्ट है। बिना आर्ग्युमेंट वाला कमांड `{}` या ख़ाली
  बॉडी लेता है (`Content-Type` हेडर फिर भी चाहिए)।
- आर्ग्युमेंट के नाम camelCase हैं, जैसे इंटरफ़ेस उन्हें भेजता है: `jobId`, `nodeId`. आर्ग्युमेंट के
  रूप में दिए ऑब्जेक्ट (`config`, `request`, `document`, `library`…) वही फ़ील्ड नाम रखते हैं जो
  इंजन लिखता है, जो ज़्यादातर snake_case हैं: `timeout_ms`, `port_start`.
- जो आर्ग्युमेंट कमांड नहीं जानता वह त्रुटि है, कभी नज़रअंदाज़ नहीं: `422` कोड `command.args_invalid`
  के साथ, जिसमें कमांड का नाम और `detail` में पार्सर के शब्द होते हैं। छोड़ा गया ज़रूरी आर्ग्युमेंट भी
  वही। बिना आर्ग्युमेंट वाला कमांड बॉडी पढ़ता ही नहीं।
- वैकल्पिक आर्ग्युमेंट छोड़ा जा सकता है या `null` के रूप में भेजा जा सकता है।
- जवाब `200` है, कमांड के नतीजे के साथ JSON में। जिस कमांड के पास लौटाने को कुछ नहीं, वह `null`
  जवाब देता है।

हर कमांड, उसके आर्ग्युमेंट और नतीजे के साथ, [कमांड](commands.md) में है।

## त्रुटियाँ {#errors}

| स्टेटस | कब | बॉडी |
| --- | --- | --- |
| `200` | कमांड चला; `/api/run` ने रन शुरू किया | नतीजा |
| `400` | बॉडी JSON नहीं है; रन रिक्वेस्ट पढ़ी नहीं जा सकती | `EngineError`: `command.args_invalid`, `api.run_invalid`, `api.run_source` |
| `400` | `path` के बिना `/api/files` | वेब सर्वर से सादा टेक्स्ट, `EngineError` नहीं |
| `401` | टोकन नहीं, या ग़लत | `EngineError`: `auth.required` |
| `403` | सर्वर द्वारा अस्वीकार किया गया `Host` या `Origin` | `EngineError`: `auth.host`, `auth.origin` |
| `404` | ऐसा कोई API पाथ नहीं; डेटा फ़ोल्डर में न रहने वाली फ़ाइल | `EngineError`: `api.not_found`, `file.not_found` |
| `405` | ऐसा मेथड जो पाथ नहीं लेता | ख़ाली |
| `413` | 24 MiB से बड़ी रिक्वेस्ट बॉडी | वेब सर्वर से सादा टेक्स्ट, `EngineError` नहीं |
| `413` | 256 MiB से बड़ा डाउनलोड | `EngineError`: `file.too_large` |
| `415` | `application/json` नहीं | `EngineError`: `command.json_required` |
| `422` | कमांड विफल हुआ, या रन शुरू नहीं हो सका | `EngineError`: इंजन का कोई भी कोड |
| `500` | फ़ाइल पढ़ी नहीं जा सकी | `EngineError`: `file.io` |

अज्ञात कमांड नाम `command.unknown` के साथ `422` है।

इंजन द्वारा बताई गई हर विफलता का एक ही रूप होता है, `EngineError`:

```json
{
  "code": "transport.refused",
  "params": { "target": "http://127.0.0.1:8080/" },
  "node": "request",
  "field": { "key": "url" },
  "detail": "error sending request for url (http://127.0.0.1:8080/): tcp connect error: Connection refused (os error 111)"
}
```

| फ़ील्ड | यह क्या है |
| --- | --- |
| `code` | क्या ग़लत हुआ: एक स्थिर पहचानकर्ता। हर कोड और उसका संदेश [त्रुटि संदेश](../reference/errors.md) में सूचीबद्ध है, डॉट से पहले वाले हिस्से से समूहित (जैसे [`transport`](../reference/errors.md#transport)) |
| `params` | संदेश में नामित मान, सब स्ट्रिंग में। न हों तो छोड़ दिए जाते हैं |
| `node` | जिस प्रयोग नोड के बारे में है। न हो तो छोड़ा जाता है |
| `field` | जिस फ़ील्ड के बारे में है: `key` ([fields](../reference/errors.md#fields) में नामित) और `index`, 1-आधारित, हेडर जैसे दोहराए जाने वाले फ़ील्ड के लिए। न हो तो छोड़ा जाता है |
| `detail` | ऑपरेटिंग सिस्टम, किसी पार्सर या लाइब्रेरी के अपने शब्द, अंग्रेज़ी में। न हों तो छोड़ दिए जाते हैं |

`code` पर शाखा बनाएँ, कभी `detail` पर नहीं। रन या एकल भेजाव द्वारा उपयोग किए गए सीक्रेट मान हर
रिपोर्ट की गई त्रुटि में मास्क (`••••`) होते हैं।

जो रिक्वेस्ट अपने सर्वर तक पहुँचती है पर त्रुटि स्टेटस, या कोई जवाब नहीं, पाती, वह विफल कमांड नहीं
है: `http_request` रिस्पॉन्स के साथ `200` जवाब देता है, और `ok`, `error` तथा `cause` बताते हैं कि
क्या हुआ। देखें [`http_request`](commands.md#http_request).

## सीमाएँ {#limits}

| क्या | सीमा | सीमा पर |
| --- | --- | --- |
| रिक्वेस्ट बॉडी | 24 MiB | `413` |
| प्रयोग दस्तावेज़ | 4 MiB | `file.too_large` |
| `/api/files` से डाउनलोड | 256 MiB | `413`, `file.too_large` |
| रन की अवधि | 300 s | रन `run.timeout` के साथ विफल होता है |
| फ़ीडबैक फ़ॉर्म (`feedback_send`) | कुल 15 MiB | `feedback.too_large` |
| एक WebSocket की प्रतीक्षा में इवेंट | 4096 | उसे [`server://lagged`](events.md#event-server-lagged) मिलता है, यह बताते हुए कि कितने छूटे |

## इवेंट {#events}

`GET /api/events` को WebSocket में अपग्रेड करने पर इंजन द्वारा भेजा हर इवेंट — रन के चरण, जॉब का अंत,
मॉनिटर के संदेश, इंस्पेक्टर के फ़्रेम — हर जुड़े क्लाइंट को टेक्स्ट संदेशों के रूप में स्ट्रीम होता है:

```json
{ "event": "job://ended", "payload": { "job_id": 7, "kind": "storm", "error": null } }
```

यह वही टोकन और `Origin` नियम लेता है जो बाकी सब। हर चैनल और उसका पेलोड [इवेंट](events.md) में है।

## फ़ाइलें {#files}

`GET /api/files?path=<path>` सर्वर के डेटा फ़ोल्डर में इंजन द्वारा लिखी फ़ाइल डाउनलोड करता है: रन
रिपोर्ट (रन के नतीजे का `report_path`), प्रयोग या इंस्पेक्टर का एक्सपोर्ट, सिग्नल या एमुलेटर लाइब्रेरी।
`path` वह पाथ है जो इंजन ने आपको दिया, सर्वर पर (URL-एन्कोड किया हुआ):

```bash
curl -fsS -G "$SERVER/api/files" --data-urlencode "path=/data/runs/run-1759600000000-3.json" \
  -H "Authorization: Bearer $TOKEN" -o report.json
```

- केवल डेटा फ़ोल्डर के भीतर की फ़ाइलें परोसी जाती हैं। बाकी कुछ भी, कोई फ़ोल्डर या न होने वाली फ़ाइल
  `file.not_found` के साथ `404` है।
- जवाब `Content-Disposition: attachment` के साथ `application/octet-stream` है। फ़ाइल नाम के अक्षर,
  अंक, `.`, `_` और `-` के अलावा बाकी वर्ण `_` बन जाते हैं।
- 256 MiB से बड़ी फ़ाइल `file.too_large` के साथ `413` है।

डेटा फ़ोल्डर में क्या होता है, यह [फ़ाइलें और फ़ोल्डर](../reference/files.md) में है।

## हेल्थ {#health}

`GET /api/health` खुला है: उसे टोकन नहीं चाहिए, केवल ऐसा `Host` जो सर्वर स्वीकार करे।

```bash
curl -fsS "$SERVER/api/health"
```

```json
{ "status": "ok", "version": "[[version]]", "auth": true }
```

`auth` बताता है कि रिक्वेस्ट को टोकन चाहिए या नहीं। `signal-lab-server healthcheck` उसी पते से
पूछता है जिस पर सर्वर सुनता है और जवाब मिलने पर `0` के साथ बाहर आता है; Docker इमेज का हेल्थ चेक इसे
चलाता है।

## OpenAPI विवरण {#openapi}

`GET /api/openapi.json` इस API को OpenAPI 3.1 में वर्णित करता है: एंडपॉइंट, हर कमांड उसके आर्ग्युमेंट
के साथ, और रन रिक्वेस्ट तथा नतीजा। इसे `/api/` के बाकी हिस्सों की तरह टोकन चाहिए। ये पेज पूरी संदर्भ
हैं; जहाँ दोनों अलग हों, वहाँ ये पेज इंजन का पालन करते हैं।

## जॉब {#jobs}

लंबे समय तक चलने वाला काम — मॉनिटर, जनरेटर, बर्स्ट, रिले, कनेक्शन, एमुलेटर, रन — एक जॉब है। जो
कमांड इसे शुरू करता है, वह चलते ही अपना `JobInfo` लौटाता है:

```json
{ "id": 4, "kind": "osc-monitor", "label": "OSC monitor 0.0.0.0:9000", "params": { "bind": "0.0.0.0:9000" }, "started_ms": 1759600000000 }
```

| फ़ील्ड | यह क्या है |
| --- | --- |
| `id` | जॉब का नंबर, सर्वर चलते समय अद्वितीय; दूसरे कमांड इसे `jobId` या `id` के रूप में लेते हैं |
| `kind` | `experiment`, `osc-monitor`, `osc-gen`, `http-burst`, `netsim`, `storm`, `scan`, `beacon`, `discovery`, `mqtt`, `websocket` या `emulator` |
| `label` | लॉग के लिए एक अंग्रेज़ी पंक्ति |
| `params` | वे मान जिनसे लेबल बनता है (target, bind, host…)। न हों तो छोड़ दिए जाते हैं |
| `started_ms` | यह कब शुरू हुआ, 1970 से मिलीसेकंड |

ये कमांड एक जॉब शुरू करते हैं: `experiment_start`, `osc_monitor_start`, `osc_generator_start`,
`http_burst_start`, `netsim_start`, `storm_start`, `scan_start`, `broadcast_beacon_start`,
`discovery_start`, `mqtt_connect`, `ws_connect` और `emulator_start`. सर्वर हर शुरुआत को उस क्लाइंट
के पते के साथ लॉग करता है जिसने माँगा; `POST /api/run` भी एक जॉब शुरू करता है।

- [`jobs_list`](commands.md#jobs_list) चल रहे जॉब की सूची देता है,
  [`job_stop`](commands.md#job_stop) एक को रोकता है,
  [`jobs_stop_all`](commands.md#jobs_stop_all) हर एक को रोकता है।
- जो जॉब अपने आप समाप्त होता है, या विफल होता है, वह
  [`job://ended`](events.md#event-job-ended) भेजता है। जिसे आप रोकते हैं वह और कुछ नहीं भेजता:
  `job_stop` का `true` जवाब ही पुष्टि है।
- जॉब सर्वर के हैं, उस क्लाइंट के नहीं जिसने उन्हें शुरू किया। पेज बंद करने या स्क्रिप्ट समाप्त होने
  से वे नहीं रुकते, हर क्लाइंट उन्हें देख सकता है और रोक सकता है, और जो सर्वर बंद होता है वह उन
  सबको रोक देता है।

## एक पूरा उदाहरण {#example}

पूछें कि सर्वर चालू है या नहीं, एक कमांड से छोटा HTTP एमुलेटर शुरू करें, उसके ख़िलाफ़ बंडल किया गया
`http-check` प्रयोग चलाएँ और नतीजे की प्रतीक्षा करें, फिर एमुलेटर रोकें। `jq` जवाबों से फ़ील्ड निकालता
है।

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

`curl -f` त्रुटि स्टेटस को विफल कमांड बना देता है; बॉडी में `EngineError` देखने के लिए इसे छोड़ दें
(जैसे चरण 3 में)।

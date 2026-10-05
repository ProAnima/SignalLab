---
title: Запуски
description: POST /api/run выполняет эксперимент на сервере Signal Lab и отвечает его результатом либо передаёт его шаги построчно в формате NDJSON.
---

# Запуск эксперимента

Чтобы выполнить эксперимент на сервере из скрипта или конвейера и узнать, чем всё
закончилось, отправьте его на `POST /api/run`. Сервер доводит запуск до конца и отвечает
результатом — а если вы попросите, то и каждым шагом по мере его выполнения. Именно этим
пользуется [`signallab run --server`](../automation/cli.md#cli-run).

Это тот же запуск, что и по кнопке [[ui:exp.run]] редактора и через
[`experiment_start`](commands.md#experiment_start): задача, которую видит и может
остановить каждая открытая страница, те же события, тот же отчёт в папке данных.

## Запрос {#request}

```http
POST /api/run
Authorization: Bearer <token>
Content-Type: application/json

{ "template": "osc-ping-reply", "overrides": { "device": "192.0.2.20:9000" }, "seed": 42, "timeout": 30 }
```

Тело называет один эксперимент — `document` или `template`, но не оба сразу — и то, с чем
его запускать:

| Поле | Тип | По умолчанию | Значение |
| --- | --- | --- | --- |
| `document` | объект | — | Эксперимент в том виде, как его сохраняет и экспортирует редактор. Старые версии переносятся на текущую, как при открытии файла |
| `template` | строка | — | Встроенный шаблон по имени файла, с `.json` или без (ниже) |
| `overrides` | объект | `{}` | Значения параметров только для этого запуска. Значения могут быть строками, числами или логическими; каждое должно быть параметром эксперимента |
| `profile` | строка | профиль документа | Запустить с этим профилем; `""` запускает со значениями по умолчанию |
| `seed` | число | seed документа, иначе новый | От 0 до 9007199254740991; один и тот же seed даёт одни и те же случайные значения |
| `timeout` | число | `300` | Секунд до того, как запуск завершится ошибкой `run.timeout`; от 1 до 300 |

Поле, которого сервер не знает, отклоняется (`400`, `api.run_invalid`). Сам документ
читается так же, как открываемый файл, поэтому он не больше 4 МиБ.

Встроенные шаблоны — эксперименты, которые редактор предлагает в разделе [[ui:exp.templates]]:

| `template` | Что это |
| --- | --- |
| `empty` | Старт и финиш |
| `http-check` | GET на `http://127.0.0.1:8080/`, затем проверка статуса 200 |
| `status-branch` | GET на `http://127.0.0.1:8080/`; при 200 — сообщение OSC, иначе пауза 500 мс |
| `parallel-flows` | Узел [[ui:exp.node.fork]] на GET `http://127.0.0.1:8080/` и запись в журнал, рядом друг с другом, затем узел [[ui:exp.node.join]] |
| `osc-ping-reply` | OSC `/ping` на параметр `device` (`127.0.0.1:9000`), затем ожидание 2 с `/pong` на `127.0.0.1:9001` |
| `poll-until-ready` | Узел [[ui:exp.node.loop]], который спрашивает `device` по OSC о `/status`, пока тот не ответит `ready`, не больше 10 раз |
| `flaky-api` | Эмулированный API (параметр `api`), который сначала даёт сбои, а потом работает; его опрашивает узел [[ui:exp.node.loop]], пока он не ответит 200 |
| `fault-phases` | Устройство UDP за реле помех, на которое отправляют 8 с, пока реле становится чистым, с потерями, отключённым и снова чистым |
| `dependency-outage` | Эмулированный API (параметр `api`), отключённый на 2 с, пока узел [[ui:exp.node.loop]] опрашивает его, пока он снова не ответит 200 |
| `websocket-echo` | Соединение с параметром `service` (`ws://127.0.0.1:9001/echo`), сообщение, ожидание его эха, проверка, закрытие |

Откройте любой из них в разделе [[ui:exp.templates]], чтобы увидеть его узлы и параметры;
см. [эксперименты](../experiments/index.md).

## Результат {#result}

По умолчанию ответ — `200` с `Content-Type: application/json`, отправленный, когда запуск
закончился: один объект JSON, результат запуска.

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

| Поле | Тип | Значение |
| --- | --- | --- |
| `job_id` | число | Задача запуска |
| `experiment` | строка | Имя эксперимента |
| `outcome` | строка | `passed`, `failed` или `stopped` |
| `seed` | число | seed, с которым он работал: передайте его обратно как `seed`, чтобы получить те же значения |
| `profile` | строка или null | Профиль, с которым он работал |
| `overridden` | логическое | Часть значений пришла из `overrides` |
| `params` | объект | Все значения параметров, которые использовал запуск |
| `started_ms`, `ended_ms` | число | Миллисекунды с 1970 года |
| `error` | `EngineError` | Почему он не прошёл: его первый сбой. Пропускается, если он прошёл |
| `steps` | object[] | Все шаги в порядке выполнения, как у [`experiment://step`](events.md#event-experiment-step) |
| `emulators` | object[] | Что принял и на что ответил каждый узел [[ui:exp.node.emulator]]: `node`, `name`, `protocol`, `local`, `counts`. Пропускается, если таких нет |
| `impairments` | object[] | Что сделало реле каждого узла [[ui:exp.node.impairment]] по фазам. Пропускается, если таких нет |
| `report_path` | строка | Отчёт запуска на сервере; скачайте его через [`/api/files`](index.md#files). Пропускается, если отчёт не записан |
| `report_error` | `EngineError` | Почему отчёт не удалось записать. Иначе пропускается |

Значения секретов маскируются во всём этом. В файле отчёта те же шаги; см. [запуски и
отчёты](../experiments/runs.md).

Пока запуск идёт, сервер каждые 15 с отправляет пробел. JSON игнорирует пробелы перед
значением, поэтому результат по-прежнему разбирается, а прокси не принимает долгий тихий
запуск за разорванное соединение.

## Следим за шагами {#lines}

Чтобы видеть шаги по мере их выполнения, запросите NDJSON:

```http
Accept: application/x-ndjson
```

Ответ — `200` с `Content-Type: application/x-ndjson`: по одному объекту JSON на строку,
у каждого есть `type`.

| `type` | Когда | Остальное в строке |
| --- | --- | --- |
| `started` | Первой, один раз | `job_id`, `experiment`, `seed`, `profile`, `overridden`, `started_ms` |
| `step` | На каждый шаг | Шаг, как у [`experiment://step`](events.md#event-experiment-step) |
| `heartbeat` | Каждые 15 с | Ничего |
| `ended` | Последней, один раз | Результат, как выше |

```text
{"job_id":12,"experiment":"OSC ping → reply","seed":42,"profile":null,"overridden":true,"started_ms":1759600000000,"type":"started"}
{"job_id":12,"ts":1759600000001,"node_id":"start","state":"running","detail":"","message_key":null,"message_params":null,"type":"step"}
…
{"job_id":12,"experiment":"OSC ping → reply","outcome":"passed",…,"type":"ended"}
```

Читайте строки до `ended`; незнакомый `type` игнорируйте. Сервер отправляет
`X-Accel-Buffering: no`, поэтому прокси nginx передаёт каждую строку сразу.

## Статус и исход {#status}

Статус ошибки HTTP означает, что **запуск не начался**; тело — это
[`EngineError`](index.md#errors):

| Статус | Код | Почему |
| --- | --- | --- |
| `400` | `api.run_invalid` | Тело — не запрос на запуск: не JSON, неизвестное поле, переопределение не строка, не число и не логическое |
| `400` | `api.run_source` | Нет ни `document`, ни `template`, или есть оба |
| `415` | `command.json_required` | Не `Content-Type: application/json` |
| `422` | `api.template_unknown` | Встроенного шаблона с таким именем нет |
| `422` | `file.json_invalid`, `file.too_large`, `doc.*` | Документ не удаётся прочитать |
| `422` | любой код проверки, `run.override_unknown`, `profile.active_missing`, `run.limit_range`, `seed.range`, `secret.missing`, `transport.address_in_use`… | Эксперимент не может начаться: он не проходит проверку, значение вне диапазона, секрет не сохранён, порт, который он слушает, занят |

Как только запуск начался, статус всегда `200`, что бы ни случилось: читайте `outcome` в
результате.

| `outcome` | Значение |
| --- | --- |
| `passed` | Все шаги прошли, и достигнут узел [[ui:exp.node.end]] |
| `failed` | Шаг не прошёл или запуск шёл дольше своего `timeout` (`run.timeout`); `error` говорит, какой и почему |
| `stopped` | Его остановили до конца: через `job_stop`, [[ui:app.stopAll]] или при остановке сервера. В `steps` — шаги, до которых он дошёл; отчёт не сохраняется |

## Клиент пропал {#disconnect}

Закрытое соединение запуск не останавливает. Это задача на сервере: она доходит до конца
и сохраняет отчёт, как запуск из браузера при закрытой вкладке. Найдите её через
[`jobs_list`](commands.md#jobs_list), остановите через [`job_stop`](commands.md#job_stop), а
отчёт потом прочитайте через [`experiment_runs`](commands.md#experiment_runs). Когда
сервер останавливается, запуск прерывается, а подключённый клиент получает
`"outcome": "stopped"`.

## Примеры {#examples}

Выполнить встроенный шаблон и дождаться исхода:

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
curl -sS -X POST "$SERVER/api/run" \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"template":"osc-ping-reply","overrides":{"device":"192.0.2.20:9000"},"timeout":30}' \
  | jq -r .outcome
```

Отправить собственный эксперимент с изменённым параметром и печатать каждый шаг по мере
выполнения:

```bash
jq '{document: ., overrides: {api: "http://192.0.2.10:8080"}, profile: ""}' smoke.json |
  curl -sSN -X POST "$SERVER/api/run" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    -H "Accept: application/x-ndjson" --data @- |
  jq -r 'select(.type == "step") | "\(.node_id)  \(.state)  \(.detail)"'
```

`-N` не даёт `curl` придерживать строки. Чтобы конвейер завершался ошибкой при неудавшемся
запуске, проверяйте `outcome`:

```bash
outcome=$(curl -sS -X POST "$SERVER/api/run" -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" -d '{"template":"http-check"}' | jq -r .outcome)
[ "$outcome" = "passed" ]
```

## Из командной строки {#cli}

[`signallab run`](../automation/cli.md#cli-run) с `--server <url>` выполняет запуск на
сервере через этот эндпоинт: он отправляет прочитанный эксперимент вместе с `overrides`,
`seed` и `timeout`, запрашивает NDJSON и печатает каждый шаг, как только приходит его
строка. Токен берётся из `--token-file`, иначе из `SIGNALLAB_TOKEN`. С `--report` он
скачивает отчёт через `/api/files`. Сервер, который молчит 60 с — даже без heartbeat, —
считается пропавшим.

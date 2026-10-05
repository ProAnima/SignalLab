---
title: Ejecuciones
description: POST /api/run ejecuta un experimento en un servidor de Signal Lab y responde con su resultado, o transmite sus pasos como líneas NDJSON.
---

# Ejecutar un experimento

Para ejecutar un experimento en un servidor desde un script o un pipeline y saber cómo fue, envíalo
a `POST /api/run`. El servidor lo ejecuta hasta el final y responde con el resultado — o, si lo
pides, con cada paso según ocurre. Esto es lo que usa
[`signallab run --server`](../automation/cli.md#cli-run).

Es la misma ejecución que la de [[ui:exp.run]] del editor y
[`experiment_start`](commands.md#experiment_start): una tarea que cada página abierta
ve y puede detener, los mismos eventos, el mismo informe en la carpeta de datos.

## La solicitud {#request}

```http
POST /api/run
Authorization: Bearer <token>
Content-Type: application/json

{ "template": "osc-ping-reply", "overrides": { "device": "192.0.2.20:9000" }, "seed": 42, "timeout": 30 }
```

El cuerpo nombra un experimento — un `document` o un `template`, no ambos — y con qué ejecutarlo:

| Campo | Tipo | Predeterminado | Significado |
| --- | --- | --- | --- |
| `document` | object | — | Un experimento tal como lo guarda y exporta el editor. Las versiones anteriores se migran, como al abrir un archivo |
| `template` | string | — | Una plantilla incluida por nombre de archivo, con o sin `.json` (más abajo) |
| `overrides` | object | `{}` | Valores de parámetros solo para esta ejecución. Los valores pueden ser cadenas, números o booleanos; cada uno debe ser un parámetro del experimento |
| `profile` | string | el del documento | Ejecuta con este perfil; `""` ejecuta con los predeterminados |
| `seed` | number | el del documento, si no uno nuevo | 0 a 9007199254740991; la misma semilla sortea los mismos valores aleatorios |
| `timeout` | number | `300` | Segundos antes de que la ejecución falle con `run.timeout`; 1 a 300 |

Un campo que el servidor no conoce se rechaza (`400`, `api.run_invalid`). El documento se lee como
se lee un archivo abierto, así que ocupa como máximo 4 MiB.

Las plantillas incluidas — los experimentos que el editor ofrece en [[ui:exp.templates]]:

| `template` | Qué es |
| --- | --- |
| `empty` | Inicio y Fin |
| `http-check` | Un GET de `http://127.0.0.1:8080/`, y luego una comprobación del estado 200 |
| `status-branch` | Un GET de `http://127.0.0.1:8080/`; con 200 un mensaje OSC, si no un retardo de 500 ms |
| `parallel-flows` | Un [[ui:exp.node.fork]] hacia un GET de `http://127.0.0.1:8080/` y una entrada de registro, en paralelo, y luego [[ui:exp.node.join]] |
| `osc-ping-reply` | Un `/ping` OSC al parámetro `device` (`127.0.0.1:9000`), y luego una espera de 2 s de `/pong` en `127.0.0.1:9001` |
| `poll-until-ready` | Un [[ui:exp.node.loop]] que pregunta a `device` por `/status` por OSC hasta que responde `ready`, como máximo 10 veces |
| `flaky-api` | Una API emulada (parámetro `api`) que falla antes de funcionar, preguntada en un [[ui:exp.node.loop]] hasta que responde 200 |
| `fault-phases` | Un dispositivo UDP tras un relé de degradación, al que se envía durante 8 s mientras el relé pasa por limpio, con pérdidas, offline y limpio otra vez |
| `dependency-outage` | Una API emulada (parámetro `api`) dejada caída 2 s mientras un [[ui:exp.node.loop]] le pregunta hasta que responde 200 otra vez |
| `websocket-echo` | Una conexión al parámetro `service` (`ws://127.0.0.1:9001/echo`), un mensaje, una espera de su eco, una comprobación, un cierre |

Abre una en [[ui:exp.templates]] para ver sus nodos y parámetros; consulta
[experimentos](../experiments/index.md).

## El resultado {#result}

Por defecto, la respuesta es `200` con `Content-Type: application/json`, enviada cuando la
ejecución ha terminado: un objeto JSON, el resultado de la ejecución.

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

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea de la ejecución |
| `experiment` | string | El nombre del experimento |
| `outcome` | string | `passed`, `failed` o `stopped` |
| `seed` | number | La semilla con la que se ejecutó: devuélvela como `seed` para sortear los mismos valores |
| `profile` | string or null | El perfil con el que se ejecutó |
| `overridden` | boolean | Algunos valores vinieron de `overrides` |
| `params` | object | Cada valor de parámetro que usó la ejecución |
| `started_ms`, `ended_ms` | number | Milisegundos desde 1970 |
| `error` | `EngineError` | Por qué falló: su primer fallo. Se omite cuando se superó |
| `steps` | object[] | Cada paso en el orden en que ocurrió, como [`experiment://step`](events.md#event-experiment-step) |
| `emulators` | object[] | Lo que recibió y respondió cada nodo [[ui:exp.node.emulator]]: `node`, `name`, `protocol`, `local`, `counts`. Se omite cuando no hay ninguno |
| `impairments` | object[] | Lo que hizo el relé de cada nodo [[ui:exp.node.impairment]], fase por fase. Se omite cuando no hay ninguno |
| `report_path` | string | El informe de la ejecución en el servidor; descárgalo con [`/api/files`](index.md#files). Se omite cuando no se escribió ninguno |
| `report_error` | `EngineError` | Por qué no se pudo escribir el informe. Se omite en caso contrario |

Los valores secretos se enmascaran en todo ello. El archivo del informe contiene los mismos pasos;
consulta [ejecuciones e informes](../experiments/runs.md).

Mientras la ejecución sigue, el servidor envía un espacio cada 15 s. JSON ignora el espacio en
blanco antes de un valor, así que el resultado se sigue analizando, y un proxy no toma una
ejecución larga y silenciosa por una conexión muerta.

## Seguir los pasos {#lines}

Para ver los pasos según ocurren, pide NDJSON:

```http
Accept: application/x-ndjson
```

La respuesta es `200` con `Content-Type: application/x-ndjson`: un objeto JSON por línea, cada uno
con un `type`.

| `type` | Cuándo | El resto de la línea |
| --- | --- | --- |
| `started` | Primero, una vez | `job_id`, `experiment`, `seed`, `profile`, `overridden`, `started_ms` |
| `step` | Cada paso | El paso, como [`experiment://step`](events.md#event-experiment-step) |
| `heartbeat` | Cada 15 s | Nada |
| `ended` | Último, una vez | El resultado, como arriba |

```text
{"job_id":12,"experiment":"OSC ping → reply","seed":42,"profile":null,"overridden":true,"started_ms":1759600000000,"type":"started"}
{"job_id":12,"ts":1759600000001,"node_id":"start","state":"running","detail":"","message_key":null,"message_params":null,"type":"step"}
…
{"job_id":12,"experiment":"OSC ping → reply","outcome":"passed",…,"type":"ended"}
```

Lee las líneas hasta `ended`; ignora un `type` que no conozcas. El servidor
envía `X-Accel-Buffering: no`, así que un proxy nginx pasa cada línea de inmediato.

## Estado y desenlace {#status}

Un estado de error HTTP significa que **no se inició ninguna ejecución**; el cuerpo es un
[`EngineError`](index.md#errors):

| Estado | Código | Por qué |
| --- | --- | --- |
| `400` | `api.run_invalid` | El cuerpo no es una solicitud de ejecución: no es JSON, un campo desconocido, un override que no es cadena, número o booleano |
| `400` | `api.run_source` | Ni `document` ni `template`, o ambos |
| `415` | `command.json_required` | No es `Content-Type: application/json` |
| `422` | `api.template_unknown` | No hay ninguna plantilla incluida con ese nombre |
| `422` | `file.json_invalid`, `file.too_large`, `doc.*` | El documento no se puede leer |
| `422` | cualquier código de validación, `run.override_unknown`, `profile.active_missing`, `run.limit_range`, `seed.range`, `secret.missing`, `transport.address_in_use`… | El experimento no puede iniciarse: no valida, un valor está fuera de rango, un secreto no está guardado, un puerto en el que escucha está ocupado |

Una vez que la ejecución ha empezado, el estado es `200`, pase lo que pase: lee
`outcome` en el resultado.

| `outcome` | Significado |
| --- | --- |
| `passed` | Cada paso se superó y se alcanzó [[ui:exp.node.end]] |
| `failed` | Un paso falló, o la ejecución tardó más que su `timeout` (`run.timeout`); `error` dice cuál y por qué |
| `stopped` | Se detuvo antes de terminar: por `job_stop`, [[ui:app.stopAll]], o el apagado del servidor. `steps` tiene los pasos a los que llegó; no se guarda ningún informe |

## Un cliente que se va {#disconnect}

Cerrar la conexión no detiene la ejecución. Es una tarea del servidor: se ejecuta hasta el final y
guarda su informe, como hace una ejecución iniciada en un navegador cuando se cierra la pestaña.
Encuéntrala con [`jobs_list`](commands.md#jobs_list), deténla con
[`job_stop`](commands.md#job_stop), y lee su informe después con
[`experiment_runs`](commands.md#experiment_runs). Cuando el servidor se apaga,
la ejecución se detiene y un cliente aún conectado recibe `"outcome": "stopped"`.

## Ejemplos {#examples}

Ejecuta una plantilla incluida y espera el desenlace:

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
curl -sS -X POST "$SERVER/api/run" \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"template":"osc-ping-reply","overrides":{"device":"192.0.2.20:9000"},"timeout":30}' \
  | jq -r .outcome
```

Envía tu propio experimento con un parámetro cambiado, e imprime cada paso según ocurre:

```bash
jq '{document: ., overrides: {api: "http://192.0.2.10:8080"}, profile: ""}' smoke.json |
  curl -sSN -X POST "$SERVER/api/run" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    -H "Accept: application/x-ndjson" --data @- |
  jq -r 'select(.type == "step") | "\(.node_id)  \(.state)  \(.detail)"'
```

`-N` evita que `curl` retenga las líneas. Para hacer fallar un pipeline ante una ejecución fallida,
comprueba `outcome`:

```bash
outcome=$(curl -sS -X POST "$SERVER/api/run" -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" -d '{"template":"http-check"}' | jq -r .outcome)
[ "$outcome" = "passed" ]
```

## Desde la línea de comandos {#cli}

[`signallab run`](../automation/cli.md#cli-run) con `--server <url>` se ejecuta en un
servidor a través de este endpoint: envía el experimento que leyó, con
`overrides`, `seed` y `timeout`, pide NDJSON e imprime cada paso según llega su
línea. El token viene de `--token-file`, si no de `SIGNALLAB_TOKEN`.
Con `--report` descarga el informe a través de `/api/files`. Un servidor que
no envía nada durante 60 s — ni siquiera un latido — cuenta como desaparecido.

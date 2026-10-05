---
title: Comandos
description: Todos los comandos del motor de Signal Lab, llamados como POST /api/invoke/<command>, con sus argumentos, resultado y errores.
---

# Comandos

Cada comando del motor, agrupado por aquello con lo que trabaja. Cada uno se
llama como `POST /api/invoke/<command>` con un objeto JSON de argumentos, y
responde `200` con su resultado o `422` con un
[`EngineError`](index.md#errors). Cómo autenticarse y qué significan los
estados está en [la visión general de la API](index.md).

## Convenciones {#conventions}

- **Los nombres de los argumentos** están en camelCase (`jobId`, `nodeId`). Un
  argumento que un comando no conoce, o uno obligatorio que falta, se rechaza
  con `command.args_invalid`; todo comando que toma argumentos puede fallar con
  este. Un comando sin argumentos no lee el cuerpo.
- **Los objetos pasados como argumento** — `config`, `request`, `document`,
  `library`, `emulator`, `profile` — usan los nombres de campo del propio
  motor, casi siempre en snake_case (`timeout_ms`). Dentro de ellos, un campo
  que el motor no conoce se **ignora**, así que un campo opcional mal escrito
  conserva su valor predeterminado en silencio. Solo el formulario de
  `feedback_send` rechaza campos desconocidos.
- Los argumentos y campos **opcionales** pueden omitirse o enviarse como
  `null`; las tablas dan sus valores predeterminados.
- Los **resultados** son JSON. "null" significa que el comando no tiene nada
  que devolver.
- Las **direcciones** escritas `IP:port` toman una dirección numérica y un
  puerto (`127.0.0.1:9000`, `[::1]:9000`); un nombre de host ahí se rechaza.
  Donde una tabla dice `IP:port` o `host:port`, un nombre de host también
  sirve: se resuelve cuando se ejecuta el comando, y se usa su dirección IPv4
  cuando tiene una (así, `localhost:9000` es `127.0.0.1:9000`).
- **Tareas**: un comando marcado con *Inicia una tarea* devuelve un
  [`JobInfo`](#type-jobinfo); el trabajo continúa hasta que termina o se
  detiene con [`job_stop`](#job_stop). Consulta [tareas](index.md#jobs).
- Las **rutas** de los resultados están en el equipo donde funciona el motor —
  en un servidor, dentro de su carpeta de datos; descárgalas con
  [`/api/files`](index.md#files).

Los ejemplos usan esta función de shell:

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
invoke() {
  curl -sS -X POST "$SERVER/api/invoke/$1" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    --data "${2:-}"
}
```

## Aplicación {#application}

### app_info {#app_info}

Qué es el motor y dónde funciona. Sin argumentos.

**Resultado**

| Campo | Tipo | Significado |
| --- | --- | --- |
| `version` | string | La versión de Signal Lab, `[[version]]` |
| `mode` | string | `desktop` o `server` |
| `secrets_writable` | boolean | Si [`secret_set`](#secret_set) y [`secret_delete`](#secret_delete) pueden funcionar aquí: `false` en un servidor |
| `data_dir` | string | La carpeta de datos, en el equipo donde funciona el motor |
| `os` | string | `windows`, `linux`… |
| `arch` | string | `x86_64`, `aarch64`… |

### get_host_info {#get_host_info}

El nombre del equipo y la dirección desde la que enviaría. Sin argumentos.

**Resultado**: `{ "local_ip": string, "hostname": string }`. `local_ip` es la
dirección IPv4 que el sistema elige para el tráfico hacia internet (se
averigua sin enviar nada), o `127.0.0.1` cuando no hay ninguna. `hostname` es
el nombre del equipo, o `localhost` cuando el sistema no lo dice.

### firewall_status {#firewall_status}

Si el firewall del sistema deja que otros equipos lleguen a este programa.
Solo Windows tiene un firewall por programa que leer; en los demás, `applies`
es `false` y del resto solo se rellena `program`. Sin argumentos.

**Resultado**

| Campo | Tipo | Significado |
| --- | --- | --- |
| `applies` | boolean | Aquí hay un firewall por programa (Windows) |
| `program` | string | El programa al que se refieren las reglas |
| `enabled` | boolean | El firewall está activado para la red en la que está el equipo ahora |
| `networks` | string[] | Los tipos de red en los que está el equipo: `domain`, `private`, `public` |
| `allowed` | boolean | Una regla de entrada deja entrar UDP para este programa en la red actual |
| `blocked` | boolean | Una regla de entrada bloquea este programa en la red actual; prevalece sobre cualquier regla de permitir |
| `rules` | number | Reglas de entrada para este programa, de cualquier tipo |

**Errores**: `firewall.failed`.

### firewall_allow {#firewall_allow}

Deja que otros equipos lleguen a Signal Lab: el sistema muestra su propio aviso
de administrador y luego las reglas de entrada del programa (una regla de
bloqueo incluida) se sustituyen por una regla de permitir para Signal Lab y
otra para la línea de comandos `signallab` que está a su lado. Solo la
aplicación de escritorio en Windows; un servidor lo rechaza, porque no hay
nadie ante su pantalla para responder al aviso.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `public` | boolean | sí | Permitir también en redes públicas, no solo en las privadas y de dominio |

**Resultado**: el nuevo [`firewall_status`](#firewall_status).

**Errores**: `firewall.server` (en un servidor), `firewall.unsupported` (no es
Windows), `firewall.declined` (el aviso se respondió No), `firewall.failed`.

### feedback_send {#feedback_send}

Envía un mensaje a los desarrolladores de Signal Lab, a través del hub del
estudio, que se lo reenvía por correo.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `form` | object | sí | El mensaje, más abajo. Los campos desconocidos se rechazan |

| Campo de `form` | Tipo | Predeterminado | Significado |
| --- | --- | --- | --- |
| `message` | string | — | Qué pasó; obligatorio, como máximo 20 000 caracteres |
| `email` | string | ninguno | Adónde puede ir una respuesta |
| `meta` | objeto de cadenas | `{}` | Lo que la aplicación dice de sí misma (version, os, arch, mode, lang, screen) |
| `screenshots` | `{ name, data }[]` | `[]` | Imágenes, `data` en base64; como máximo 6, 8 MiB cada una |
| `logs` | `{ name, text }[]` | `[]` | Archivos de texto; como máximo 4, 2 MiB cada uno |

Todo junto es como máximo 15 MiB.

**Resultado**: `{ "id": string }`, la referencia que reciben los desarrolladores.

**Errores**: `feedback.message_required`, `feedback.message_too_long`,
`feedback.too_many_files`, `feedback.file_too_large`, `feedback.too_large`,
`feedback.invalid`, los rechazos del hub (`feedback.email_invalid`,
`feedback.file_type`, `feedback.rate_limited`, `feedback.disabled`,
`feedback.send_failed`, `feedback.failed`), y los `transport.*` de la red.

```bash
invoke app_info
# {"version":"[[version]]","mode":"server","secrets_writable":false,"data_dir":"/data","os":"linux","arch":"x86_64"}
```

## Tareas {#jobs}

### jobs_list {#jobs_list}

Las tareas en marcha, la más antigua primero. Sin argumentos.

**Resultado**: [`JobInfo`](#type-jobinfo)`[]`.

### job_stop {#job_stop}

Detiene una tarea al instante: sus sockets se cierran, su relé, servidor o
conexión desaparece. Una tarea detenida no envía
[`job://ended`](events.md#event-job-ended); una ejecución detenida no guarda
ningún informe.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `id` | number | sí | El `id` de la tarea |

**Resultado**: `true` cuando había una tarea con ese id en marcha, `false` en
caso contrario.

### jobs_stop_all {#jobs_stop_all}

Detiene todas las tareas en marcha, quienquiera que las iniciara. Sin
argumentos.

**Resultado**: null.

```bash
invoke jobs_list
# [{"id":3,"kind":"osc-monitor","label":"OSC monitor 0.0.0.0:9000","params":{"bind":"0.0.0.0:9000"},"started_ms":1759600000000}]
invoke job_stop '{"id":3}'
# true
```

## Experimentos y ejecuciones {#experiments}

Estos comandos toman y devuelven un documento de experimento (`Experiment`):
el JSON que el editor guarda y exporta, con `version`, `name`, `params`,
`profiles`, `profile`, `seed`, `cookies`, `nodes` y `edges`. Sus nodos están en
[nodos](../experiments/nodes.md); sus parámetros, perfiles y plantillas, en
[datos](../experiments/data.md). Un documento es como máximo 4 MiB
(`file.too_large`). Para ejecutar un experimento y esperar su resultado, usa
[`POST /api/run`](run.md) en lugar de [`experiment_start`](#experiment_start).

### experiment_load {#experiment_load}

El experimento de trabajo: `experiment.json` en la carpeta de datos — en un
servidor, el que muestra su interfaz. Cuando no hay ninguno, el experimento
inicial. Las versiones antiguas del documento se migran. Sin argumentos.

**Resultado**: `Experiment`.

**Errores**: `file.io`, `file.json_invalid` (con el `path`, la `line` y la
`column` del archivo), `file.too_large`, `doc.version_unsupported` y las demás
comprobaciones `doc.*`.

### experiment_save {#experiment_save}

Sustituye el experimento de trabajo, `experiment.json` en la carpeta de datos.
Se escribe primero en un archivo temporal, así que una escritura fallida deja
el anterior.

::: warning
En un servidor, este es el documento sobre el que trabaja el editor de todos
los navegadores.
:::

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `document` | `Experiment` | sí | El documento |

**Resultado**: string, la ruta escrita.

**Errores**: `doc.*`, las comprobaciones de tamaño del documento (`param.*`,
`params.too_many`, `profile.*`, `profiles.too_many`, `seed.range`),
`file.too_large`, `file.io`.

### experiment_parse {#experiment_parse}

Lee un experimento desde texto JSON, como hace [[ui:exp.importJson]]. Las
versiones 1 a 8 se migran a la versión 9, la actual; un archivo anterior a la
versión 8 se abre con `cookies` desactivado, así que se ejecuta como antes. Se
omite una marca de orden de bytes.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `text` | string | sí | El texto del archivo |

**Resultado**: `Experiment`.

**Errores**: `file.json_invalid` (`line`, `column`), `file.too_large`,
`doc.version_unsupported`, `doc.*`, las comprobaciones de tamaño de
[`experiment_save`](#experiment_save).

### experiment_export {#experiment_export}

Escribe una instantánea de un documento en
`exports/experiment-<ms>-<16 hex digits>.json` en la carpeta de datos. Cada
exportación es un archivo nuevo.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `document` | `Experiment` | sí | El documento |

**Resultado**: string, la ruta escrita.

**Errores**: los de [`experiment_save`](#experiment_save).

### experiment_validate {#experiment_validate}

Comprueba que un documento se ejecutaría con su perfil activo (o sus valores
predeterminados): el grafo, cada campo, los parámetros, y que todo secreto que
nombra esté guardado. Un problema que lo impide es el error. Si todo va bien,
indica cuál de los *otros* perfiles fallaría, para que lo sepas antes de
cambiar.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `document` | `Experiment` | sí | El documento |
| `overrides` | objeto de cadenas | no | Valores de parámetros solo para esta comprobación, tal como los da [[ui:exp.runWith]] |

**Resultado**: `{ "profile": string or null, "error": EngineError }[]` — cada
otro perfil que no se validaría (`null`: los valores predeterminados, sin
perfil). Una lista vacía significa que todos los perfiles están bien.

**Errores**: cualquier código de validación (`doc.*`, `graph.*`, `node.*`,
`param.*`, `profile.*`, `template.*`, `loop.*`…), `run.override_unknown` (un
valor de sustitución para un parámetro que el documento no tiene),
`secret.missing`, `secret.store`, `secret.unsupported`.

### experiment_resolve {#experiment_resolve}

Un nodo con sus plantillas rellenas, tal como lo muestra la vista previa del
editor: los valores del perfil activo y los valores de variables que indiques.
Los secretos se muestran como `••••`, nunca sus valores. Los nombres que no
tienen valor se quedan tal como se escribieron y se enumeran.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `document` | `Experiment` | sí | El documento |
| `nodeId` | string | sí | El nodo |
| `vars` | object | sí | Valores de variables que usar, por nombre; `{}` para ninguno |

**Resultado**: `{ "node": node, "missing": string[] }`.

**Errores**: `node.not_found`, `template.*`, `secret.store`,
`secret.unsupported`.

### experiment_send_node {#experiment_send_node}

[[ui:exp.sendNow]]: realiza un nodo por sí solo, a través del mismo código que
usa una ejecución. Una acción se envía; una espera escucha desde ese momento
hasta que coincide o se agota el tiempo. Un nodo [[ui:exp.node.ws_send]] o
[[ui:exp.node.wait_ws]] abre la conexión que describe su nodo
[[ui:exp.node.ws_connect]]. No se envía nada con cookies, y los nodos
[[ui:exp.node.impairment]] y [[ui:exp.node.emulator]] de la ejecución no se
abren.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `document` | `Experiment` | sí | El documento; su perfil activo da los valores de los parámetros |
| `nodeId` | string | sí | Una acción o una espera |
| `vars` | object | sí | Valores de variables que leen las plantillas del nodo; `{}` para ninguno |

**Resultado**

| Campo | Tipo | Significado |
| --- | --- | --- |
| `detail` | string | Qué pasó, en inglés |
| `response` | [`HttpResponse`](#type-httpresponse) o null | La respuesta de un nodo HTTP |
| `vars` | object | Lo que fijó el paso: la respuesta de una espera, o lo que los nodos [[ui:exp.node.extract]] posteriores a una solicitud toman de su respuesta |

Los valores secretos se enmascaran en todo ello.

**Errores**: `node.not_found`, `run.not_an_action` (no es una acción ni una
espera), `ws.connection_unknown`, `secret.missing`, `template.*`, y aquello con
lo que falle el paso: `transport.*`, `wait.timeout`, `check.*`…

### experiment_start {#experiment_start}

Inicia una ejecución, como hace [[ui:exp.run]], y vuelve de inmediato. Sus
pasos llegan como eventos
[`experiment://step`](events.md#event-experiment-step), su final como
[`experiment://ended`](events.md#event-experiment-ended), y su informe se
guarda bajo `runs/` en la carpeta de datos. Las esperas, los emuladores, los
relés de degradación y las suscripciones MQTT se abren antes del primer paso,
así que un puerto ocupado falla aquí. Una ejecución de más de 300 s falla con
`run.timeout`. *Inicia una tarea* (`experiment`).

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `document` | `Experiment` | sí | El documento |
| `overrides` | objeto de cadenas | no | Valores de parámetros solo para esta ejecución |
| `seed` | number | no | La semilla de la ejecución, de 0 a 9007199254740991; predeterminado: la del documento, si no, una nueva |

**Resultado**: [`JobInfo`](#type-jobinfo), con `params.name`, el nombre del
experimento.

**Errores**: todo lo que informa
[`experiment_validate`](#experiment_validate), `seed.range`,
`transport.address_in_use` y los demás fallos de enlace, `emulator.*`,
`impair.*`, `node.params_only` (una dirección de escucha, o el bróker o el tema
de una espera MQTT que no está fijado cuando empieza la ejecución), y los
errores de [`mqtt_connect`](#mqtt_connect) de un bróker al que una espera MQTT
no puede llegar.

### experiment_runs {#experiment_runs}

Ejecuciones leídas de sus informes en `runs/`, la más reciente primero. Un
informe que no se puede leer se omite.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `name` | string | no | Solo las ejecuciones del experimento con este nombre exacto |
| `limit` | number | no | Como máximo esta cantidad; predeterminado 50, como máximo 500 |

**Resultado**: resúmenes de ejecuciones:

| Campo | Tipo | Significado |
| --- | --- | --- |
| `name` | string | El nombre del archivo del informe, `run-<ms>-<job>.json`: lo que toma [`experiment_compare`](#experiment_compare) |
| `experiment` | string | El nombre del experimento |
| `started_ms`, `ended_ms` | number | Milisegundos desde 1970 |
| `outcome` | string | `passed` o `failed` |
| `seed` | number | La semilla de la ejecución |
| `profile` | string o null | Su perfil |
| `loads` | object[] | Cada paso de carga: `node`, `sent`, `rps`, `p95_ms`, `error_rate`, `held` (se cumplieron todos los umbrales) |

**Errores**: `file.io`.

### experiment_compare {#experiment_compare}

Dos ejecuciones una junto a otra, paso de carga a paso de carga, tal como los
muestra [[ui:exp.compare]] de la línea de tiempo. Los pasos se emparejan por el
id del nodo.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `a` | string | sí | El nombre del archivo del informe de la ejecución anterior |
| `b` | string | sí | El nombre del archivo del informe de la ejecución posterior |

**Resultado**: `{ "a": summary, "b": summary, "steps": [...] }`, cada paso con
`node`, `missing_in` (`a` o `b`, cuando solo una ejecución lo tiene), `metrics`,
`sent` (`[a, b]`), `thresholds_a` y `thresholds_b` (cada umbral como
`{ metric, op, value, actual, held }`). `metrics` enumera nueve métricas, cada
una como `{ metric, a, b, change, percent, worse }`: `metric` es `p50_ms`,
`p90_ms`, `p95_ms`, `p99_ms`, `mean_ms`, `max_ms`, `error_rate`, `rps` o
`missed`; `change` es `b − a`; `percent`, el cambio en % de `a` (null cuando `a`
es 0); `worse`, que se movió en el sentido malo — más alto, o más bajo para
`rps` — en un 5 % o más, o de 0 a cualquier cosa. Un paso que solo tiene una
ejecución nunca es `worse`. Consulta [carga](../experiments/load.md).

**Errores**: `runs.name_invalid` (cualquier cosa que no sea el nombre del
archivo de un informe, sin carpetas), `runs.not_found`, `file.json_invalid`,
`file.io`.

```bash
invoke experiment_validate "$(jq '{document: .}' experiment.json)"
# []
```

## Secretos {#secrets}

Los experimentos usan los valores secretos como `{{secret.NAME}}` y nunca salen
del motor: ningún comando devuelve uno. Dónde se guardan depende de dónde
funcione el motor:

| Dónde | Almacén | Definir y quitar |
| --- | --- | --- |
| Aplicación de escritorio, Windows | Administrador de credenciales de Windows | Sí |
| Aplicación de escritorio, Linux | Ninguno | `secret.unsupported` |
| Servidor | `SIGNALLAB_SECRET_<NAME>`, o el archivo `<NAME>` en `--secrets-dir` (predeterminado `/run/secrets/signallab`) | No: `secret.read_only` |

Un nombre empieza por una letra latina o `_`, continúa con letras latinas,
dígitos y `_`, y tiene como máximo 128 caracteres (`secret.name_invalid`).

### secret_status {#secret_status}

Cuáles de los nombres dados tienen un valor guardado.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `names` | string[] | sí | Los nombres que buscar |

**Resultado**: un objeto, nombre → `true` (guardado) o `false`.

**Errores**: `secret.name_invalid` (en un servidor), `secret.store`,
`secret.too_large` (el archivo de un servidor de más de 16 KiB),
`secret.unsupported`.

### secret_set {#secret_set}

Guarda un valor bajo un nombre, sustituyendo el que hubiera. Solo la aplicación
de escritorio.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `name` | string | sí | El nombre |
| `value` | string | sí | No vacío; como máximo 16 KiB |

**Resultado**: null.

**Errores**: `secret.read_only` (en un servidor), `secret.unsupported`,
`secret.name_invalid`, `secret.empty`, `secret.too_large`, `secret.store`.

### secret_delete {#secret_delete}

Quita un valor guardado. Quitar uno que no está guardado no es un error. Solo
la aplicación de escritorio.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `name` | string | sí | El nombre |

**Resultado**: null.

**Errores**: `secret.read_only` (en un servidor), `secret.unsupported`,
`secret.name_invalid`, `secret.store`.

```bash
invoke secret_status '{"names":["API_TOKEN","MQTT_PASSWORD"]}'
# {"API_TOKEN":true,"MQTT_PASSWORD":false}
```

## OSC {#osc}

Consulta [OSC](../protocols/osc.md) para ver la pantalla a la que sirven estos
comandos.

### osc_send {#osc_send}

Envía un mensaje OSC en un datagrama UDP, desde un socket nuevo.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `target` | string | sí | `IP:port` o `host:port` al que enviar; un nombre se resuelve y se toma su dirección IPv4 cuando tiene una |
| `address` | string | sí | La dirección OSC, `/mixer/fader/1`; empieza por `/` |
| `args` | [`OscArg`](#type-oscarg)`[]` | sí | Los argumentos; `[]` para ninguno |

**Resultado**: number, los bytes enviados.

**Errores**: `node.osc_address` (sin `/` inicial; campo `address`),
`transport.target_invalid` (sin puerto, o ninguna de las dos formas),
`transport.dns` (el nombre no se resuelve), `transport.*`.

### osc_monitor_start {#osc_monitor_start}

Escucha OSC en un puerto UDP y decodifica cada paquete. Cada uno llega como un
evento [`osc://message`](events.md#event-osc-message). *Inicia una tarea*
(`osc-monitor`, `params.bind`).

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `bind` | string | sí | `IP:port` en el que escuchar: `0.0.0.0:9000` todas las tarjetas de red, `127.0.0.1:9000` solo este equipo |

**Resultado**: [`JobInfo`](#type-jobinfo).

**Errores**: `node.bind_invalid`, `transport.address_in_use`,
`transport.address_unavailable`, `transport.denied`, `wait.bind_failed`. La
tarea termina con `wait.receive_failed` si el socket ya no puede recibir.

### osc_generator_start {#osc_generator_start}

Envía un flujo de mensajes OSC cuyo único argumento sigue una forma de onda. El
progreso llega como [`osc://gen-tick`](events.md#event-osc-gen-tick). *Inicia
una tarea* (`osc-gen`, `params.target`, `params.address`).

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `config` | object | sí | Más abajo |

| Campo de `config` | Tipo | Predeterminado | Significado |
| --- | --- | --- | --- |
| `target` | string | — | `IP:port` o `host:port` al que enviar; un nombre se resuelve una vez, cuando empieza la tarea |
| `address` | string | — | La dirección OSC; empieza por `/` |
| `rate` | number | — | Mensajes por segundo, mantenidos entre 0,1 y 5000 |
| `waveform` | string | — | `sine`, `triangle`, `saw` (descendente: de `max` a `min`, y de vuelta al instante), `ramp` (ascendente: de `min` a `max`, y de vuelta al instante), `square`, `random` o `constant` (`max`) |
| `freq` | number | — | Ciclos de la forma de onda por segundo |
| `min`, `max` | number | — | El rango del valor |
| `as_int` | boolean | `false` | Redondear y enviar un int en lugar de un float |
| `duration_s` | number | `0` | Parar tras estos segundos; 0 se ejecuta hasta que se detenga |

**Resultado**: [`JobInfo`](#type-jobinfo).

**Errores**: `node.osc_address`, `transport.target_invalid`, `transport.dns`,
`transport.*`. La tarea termina con un error `transport.*` si falla un envío.

```bash
invoke osc_send '{"target":"127.0.0.1:9000","address":"/cue/go","args":[{"type":"int","value":1}]}'
# 16
invoke osc_monitor_start '{"bind":"0.0.0.0:9000"}'
```

## HTTP y cookies {#http}

Consulta [HTTP](../protocols/http.md).

### http_request {#http_request}

Envía una solicitud HTTP y devuelve la respuesta. Una solicitud que no recibe
respuesta — rechazada, agotada por tiempo, un nombre que no se resuelve, un
certificado en el que no se confía — **no** es un error del comando: la
respuesta lo dice en `error` y `cause`.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `request` | [`HttpRequest`](#type-httprequest) | sí | La solicitud |
| `cookies` | boolean | no | Enviar el almacén de cookies de la pantalla [[ui:nav.http]] y guardar lo que fije la respuesta; predeterminado `false` |

**Resultado**: [`HttpResponse`](#type-httpresponse).

**Errores**: `http.client_failed` (la solicitud ni siquiera se pudo preparar).

### http_burst_start {#http_burst_start}

Envía una solicitud muchas veces, varias a la vez, y la mide. Sin una `rate`,
cada trabajador vuelve a enviar en cuanto tiene una respuesta; con una, las
solicitudes empiezan según una planificación fija por lentas que sean las
respuestas, y una solicitud que esperó más de 50 ms pasada su hora a que un
trabajador quedara libre se omite y se cuenta como omitida. El progreso llega
como [`http://burst-progress`](events.md#event-http-burst-progress) diez veces
por segundo. *Inicia una tarea* (`http-burst`, `params.method`, `params.url`, y
`params.rate` cuando va a ritmo).

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `config` | object | sí | Los campos de [`HttpRequest`](#type-httprequest) y los de más abajo, en un solo objeto |

| Campo de `config` | Tipo | Predeterminado | Significado |
| --- | --- | --- | --- |
| `concurrency` | number | — | Como máximo esta cantidad en vuelo, mantenida entre 1 y 512 |
| `total` | number | `0` | Parar tras esta cantidad de solicitudes; 0: sin recuento |
| `duration_s` | number | `0` | Parar tras estos segundos; 0: sin límite de tiempo |
| `rate` | number | `0` | Solicitudes iniciadas por segundo, de 0,1 a 100 000; 0: tan rápido como llegan las respuestas |
| `cookies` | boolean | `false` | Usar el almacén de cookies de la pantalla [[ui:nav.http]] |

Sin `total` ni `duration_s`, la ráfaga se ejecuta hasta que se detenga.

**Resultado**: [`JobInfo`](#type-jobinfo).

**Errores**: `http.rate_invalid`, `http.duration_invalid`, `http.client_failed`.

### http_cookies {#http_cookies}

El almacén de cookies de la pantalla [[ui:nav.http]]: cada cookie que no ha
caducado. En un servidor hay un almacén para cada página y script. Sin
argumentos.

**Resultado**: cookies, cada una con `name`, `value`, `domain`, `host_only`
(sin atributo Domain: solo el host que la fijó la recibe), `path`, `expires`
(segundos Unix, null para una cookie de sesión), `secure`, `http_only` y
`same_site` (string o null).

### http_cookies_clear {#http_cookies_clear}

Vacía el almacén de cookies de la pantalla [[ui:nav.http]]. Sin argumentos.

**Resultado**: null.

```bash
invoke http_request '{"request":{"method":"GET","url":"http://127.0.0.1:8080/health","headers":[["Accept","application/json"]],"body":null,"timeout_ms":5000}}' \
  | jq '{status, latency_ms, body}'
```

## WebSocket {#websocket}

Consulta [WebSocket](../protocols/websocket.md). Una conexión que abre
`ws_connect` es una tarea; los demás la nombran por `jobId`.

### ws_connect {#ws_connect}

Abre un WebSocket y lo mantiene abierto. Lo que llega y lo que se envía llega
como [`ws://messages`](events.md#event-ws-messages) cada 100 ms; el estado de la
conexión, como [`ws://state`](events.md#event-ws-state). *Inicia una tarea*
(`websocket`, `params.url`).

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | sí | Dónde y cómo conectarse |

**Resultado**: [`JobInfo`](#type-jobinfo).

**Errores**: `ws.url_invalid`, `ws.header_invalid`, `ws.protocol_invalid`,
`ws.handshake_status` (el servidor respondió a la actualización con otro
estado), `ws.subprotocol_refused`, `ws.handshake_failed`, `transport.*`.

### ws_send {#ws_send}

Envía un mensaje en una conexión abierta.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `jobId` | number | sí | La tarea de la conexión |
| `message` | object | sí | `{ "text": "…" }` para un mensaje de texto, o `{ "hex": "de ad be ef" }` para uno binario; exactamente uno de los dos |

**Resultado**: number, los bytes enviados.

**Errores**: `ws.not_connected`, `ws.payload_required` (ninguno de los dos o
ambos), `hex.invalid` (un `hex` vacío también), `node.too_long` (más de 16 MiB,
campo `payload`; no se envía nada y la conexión sigue abierta), `ws.closed`,
`transport.*` (`transport.timeout` cuando el servidor dejó de leer durante 10
s).

### ws_close {#ws_close}

Cierra una conexión con un handshake de cierre y espera hasta 2 s la respuesta
del servidor; la tarea termina entonces. Una vez que una conexión ha terminado,
su tarea desaparece y cerrarla es `ws.not_connected`.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `jobId` | number | sí | La tarea de la conexión |
| `code` | number | no | 1000, o 3000 a 4999 para los propios de una aplicación; predeterminado 1000 |
| `reason` | string | no | Como máximo 123 bytes; predeterminado vacío |

**Resultado**: `{ "code", "reason", "by", "error" }` — `by` es `client`,
`server` o `lost`; `code` es 1005 cuando el cierre no llevaba ninguno y 1006
cuando no hubo trama de cierre.

**Errores**: `ws.close_code`, `node.too_long`, `ws.not_connected`.

### ws_exchange {#ws_exchange}

Un intercambio sin tarea: conectar, enviar un mensaje si se da, esperar una
respuesta si se pide, cerrar.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | sí | Dónde y cómo conectarse |
| `message` | object | no | `{ "text" }` o `{ "hex" }`, como para [`ws_send`](#ws_send) |
| `expect` | object | no | Qué esperar: `mode` (`any`, `contains`, `regex`, `hex`; predeterminado `any`), `pattern` (predeterminado vacío), `timeout_ms` (predeterminado 2000) |

Con `expect` y sin `message`, cuenta el primer mensaje que coincide tras
conectar — un saludo.

**Resultado**: `{ "handshake", "sent", "reply", "closed" }` — `handshake` es
`{ url, peer, local, protocol, ms }`; `sent`, los bytes enviados o null; `reply`
`{ kind, text, hex, bytes, json, ms }` o null (`json`: una respuesta de texto
analizada, si no null; `ms`: desde el envío, o desde la conexión cuando no se
envió nada); `closed` tal como lo devuelve [`ws_close`](#ws_close).

**Errores**: los de [`ws_connect`](#ws_connect) y [`ws_send`](#ws_send),
`wait.timeout` (con `ms`, `unmatched` y `target`), `regex.invalid` y
`hex.invalid` (un `pattern` que no se puede analizar).

```bash
invoke ws_exchange '{"config":{"url":"ws://127.0.0.1:9001/"},"message":{"text":"{\"type\":\"ping\"}"},"expect":{"mode":"contains","pattern":"pong"}}' \
  | jq .reply.text
```

## MQTT {#mqtt}

MQTT 3.1.1 sobre TCP sin cifrar, QoS 0, 1 y 2. Consulta
[MQTT](../protocols/mqtt.md).

### mqtt_connect {#mqtt_connect}

Se conecta a un bróker y mantiene la conexión. El comando devuelve cuando el
bróker ha aceptado la conexión (CONNACK), así que una contraseña incorrecta o
un puerto cerrado es su error. Los mensajes llegan como
[`mqtt://messages`](events.md#event-mqtt-messages) cada 100 ms; los cambios de
estado, como [`mqtt://state`](events.md#event-mqtt-state); las publicaciones y
desuscripciones QoS 1/2 completadas, como
[`mqtt://ack`](events.md#event-mqtt-ack). *Inicia una tarea* (`mqtt`,
`params.broker`, `params.client`).

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | sí | El bróker y cómo conectarse |

**Resultado**: [`JobInfo`](#type-jobinfo).

**Errores**: `mqtt.client_id_required`, `transport.*` (rechazado, inalcanzable,
`dns`, `timeout` tras 6 s), `mqtt.no_answer` (sin CONNACK en 6 s),
`mqtt.protocol`, `mqtt.refused_protocol`, `mqtt.refused_client_id`,
`mqtt.refused_unavailable`, `mqtt.refused_credentials`,
`mqtt.refused_not_authorized`, `mqtt.refused`.

### mqtt_publish {#mqtt_publish}

Publica en una conexión abierta.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `jobId` | number | sí | La tarea de la conexión |
| `topic` | string | sí | El tema: no vacío, y sin `+` ni `#` |
| `payload` | string | sí | La carga útil, enviada como UTF-8 |
| `qos` | number | sí | 0, 1 o 2 (más de 2 se envía como 2) |
| `retain` | boolean | sí | Pedir al bróker que la conserve; una carga útil vacía con `retain` borra un valor retenido |

**Resultado**: null. Una publicación QoS 1 o 2 se confirma más tarde con
[`mqtt://ack`](events.md#event-mqtt-ack).

**Errores**: `node.topic_wildcard` (campo `topic`) y `mqtt.topic_required`
(campo `topic`), igual que para [`mqtt_publish_once`](#mqtt_publish_once) — el
comando los rechaza antes de buscar la conexión; `mqtt.not_connected`.

### mqtt_subscribe {#mqtt_subscribe}

Suscribe una conexión abierta a filtros. Lo que concede el bróker llega como
[`mqtt://state`](events.md#event-mqtt-state) con `state: "subscribed"`.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `jobId` | number | sí | La tarea de la conexión |
| `filters` | `{ filter, qos }[]` | sí | Al menos uno; `qos` es 0 de forma predeterminada. `+` y `#` son comodines |

**Resultado**: null.

**Errores**: `mqtt.filter_required`, `mqtt.not_connected`.

### mqtt_unsubscribe {#mqtt_unsubscribe}

Desuscribe una conexión abierta de unos filtros.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `jobId` | number | sí | La tarea de la conexión |
| `filters` | string[] | sí | Al menos uno |

**Resultado**: null. La respuesta del bróker llega como
[`mqtt://ack`](events.md#event-mqtt-ack) con `kind: "unsubscribed"`.

**Errores**: `mqtt.filter_required`, `mqtt.not_connected`.

### mqtt_publish_once {#mqtt_publish_once}

Se conecta, publica un mensaje, espera el acuse que pide su QoS (hasta 6 s) y se
desconecta. Trae su propia conexión, bajo un id de cliente propio — los
primeros 12 caracteres de `client_id`, `-o` y un número — así que nunca expulsa
del bróker una conexión viva con ese id.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | sí | El bróker; no se usa `subscribe` |
| `topic` | string | sí | No vacío y sin `+` ni `#` |
| `payload` | string | sí | Enviada como UTF-8 |
| `qos` | number | sí | 0, 1 o 2 (más de 2 se envía como 2) |
| `retain` | boolean | sí | Pedir al bróker que la conserve |

**Resultado**: string, un resumen escrito por Signal Lab:
`<topic> → <broker> · <bytes> B · qos<n>`, con ` retained` cuando es retenida.

**Errores**: `mqtt.topic_required`, `node.topic_wildcard`, y los de
[`mqtt_connect`](#mqtt_connect) pero `mqtt.client_id_required`: un `client_id`
vacío se acepta aquí.

```bash
invoke mqtt_publish_once '{"config":{"host":"127.0.0.1","port":1883,"client_id":"lab"},"topic":"lab/lamp/set","payload":"ON","qos":1,"retain":false}'
# "lab/lamp/set → 127.0.0.1:1883 · 2 B · qos1"
```

## Difusión, multidifusión y descubrimiento {#broadcast}

Consulta [difusión y descubrimiento](../protocols/broadcast.md).

::: danger
La difusión y un barrido llegan a todos los hosts de un segmento de red. Envía
solo en redes de las que seas responsable.
:::

### broadcast_send {#broadcast_send}

Envía un datagrama a cada destino, una vez.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `config` | object | sí | Más abajo |

| Campo de `config` | Tipo | Predeterminado | Significado |
| --- | --- | --- | --- |
| `mode` | string | — | `list`, `broadcast`, `multicast` o `sweep` |
| `target` | string | — | Según el modo, más abajo |
| `port` | number | `0` | El puerto, solo para `sweep` |
| `payload` | [`Payload`](#type-payload) | — | Lo que lleva cada datagrama |
| `bind` | string | cualquiera | El `IP:port` local desde el que se envía; vacío o null: `0.0.0.0:0` (`[::]:0` cuando todos los destinos son IPv6) |
| `ttl` | number | `1` | TTL de IP, o el límite de saltos de multidifusión; 1 a 255 |
| `multicast_loop` | boolean | `true` | La multidifusión vuelve también a este equipo |
| `rate`, `count`, `duration_s` | number | `0` | Solo para [`broadcast_beacon_start`](#broadcast_beacon_start) |

| `mode` | `target` |
| --- | --- |
| `list` | entradas `IP:port` o `host:port` separadas por comas, puntos y comas o saltos de línea (no por espacios); un nombre se resuelve y se toma su dirección IPv4 cuando tiene una |
| `broadcast` | `255.255.255.255:port`, o una dirección que termina en `.255` con su puerto |
| `multicast` | Un grupo de 224.0.0.0 a 239.255.255.255 con su puerto |
| `sweep` | Un bloque CIDR, `192.0.2.0/24`: todos los hosts utilizables en `port`; como máximo 1024 hosts, así que `/22` o más estrecho |

**Resultado**

| Campo | Tipo | Significado |
| --- | --- | --- |
| `targets` | number | Destinos |
| `packets`, `bytes` | number | Lo que salió |
| `errors` | number | Datagramas que no se pudieron enviar |
| `resolved` | string[] | Los primeros 8 destinos |
| `summary` | string | La carga útil en una línea |
| `error` | `EngineError` | Por qué falló el primer datagrama fallido; se omite cuando no falló ninguno |

**Errores**: `broadcast.target_required`, `broadcast.not_broadcast`,
`broadcast.ipv6`, `broadcast.not_multicast`, `broadcast.sweep_port`,
`broadcast.cidr_invalid`, `broadcast.prefix_invalid`,
`broadcast.sweep_too_large`, `node.osc_address` (una dirección OSC debe empezar
por `/`), `hex.empty`, `hex.invalid`, `node.bind_invalid`,
`socket.option_failed`, `transport.target_invalid`, `transport.dns`, fallos de
enlace.

### broadcast_beacon_start {#broadcast_beacon_start}

Envía la misma ronda — un datagrama por destino — una y otra vez. Sus recuentos
llegan como [`broadcast://emit-stat`](events.md#event-broadcast-emit-stat) cada
250 ms. Tras más de 32 envíos fallidos sin haber enviado ninguno, se detiene con
el motivo. *Inicia una tarea* (`beacon`, `params.mode`, `params.target`,
`params.targets`, `params.rate`).

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `config` | object | sí | Como para [`broadcast_send`](#broadcast_send), con los tres de más abajo |

| Campo de `config` | Tipo | Predeterminado | Significado |
| --- | --- | --- | --- |
| `rate` | number | — | Rondas por segundo; mayor que 0, y rondas × destinos como máximo 50 000 datagramas por segundo |
| `count` | number | `0` | Parar tras esta cantidad de rondas; 0: sin recuento |
| `duration_s` | number | `0` | Parar tras estos segundos; 0: hasta que se detenga |

**Resultado**: [`JobInfo`](#type-jobinfo).

**Errores**: los de [`broadcast_send`](#broadcast_send),
`broadcast.rate_invalid`, `broadcast.rate_limit`.

### discovery_start {#discovery_start}

Escucha en un puerto UDP, mantiene una lista de cada par que envía algo, y
puede responder a sondeos como lo haría un dispositivo. Los pares llegan como
[`broadcast://peers`](events.md#event-broadcast-peers) cada 400 ms. *Inicia una
tarea* (`discovery`, `params.bind`, `params.groups`, `params.joined`).

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `config` | object | sí | Más abajo |

| Campo de `config` | Tipo | Predeterminado | Significado |
| --- | --- | --- | --- |
| `bind` | string | — | `IP:port` en el que escuchar |
| `groups` | string[] | `[]` | Grupos de multidifusión a los que unirse (IPv4) |
| `interface` | string | cualquiera | La dirección IPv4 local en la que unirse a los grupos |
| `reuse` | boolean | `true` | Compartir el puerto con un programa que ya escucha en él (`SO_REUSEADDR`) |
| `respond` | boolean | `false` | Responder a lo que llegue |
| `response` | [`Payload`](#type-payload) | ninguno | La respuesta; necesaria con `respond` |
| `respond_delay_ms` | number | `0` | Esperar este tiempo antes de responder |
| `match_contains` | string | ninguno | Responder solo a los datagramas cuyo texto contenga esto |

Se enumeran como máximo 512 pares; los posteriores no se añaden.

**Resultado**: [`JobInfo`](#type-jobinfo).

**Errores**: `node.bind_invalid`, `broadcast.port_shared` (el puerto está
ocupado y `reuse` está desactivado), `broadcast.interface_invalid`,
`broadcast.not_multicast`, `broadcast.join_failed`, `broadcast.reply_missing`,
`node.osc_address`, `hex.*`, fallos de enlace. La tarea termina con
`wait.receive_failed` si el socket ya no puede recibir.

```bash
invoke broadcast_send '{"config":{"mode":"list","target":"127.0.0.1:9000, 127.0.0.1:9001","payload":{"kind":"text","text":"PING"}}}' \
  | jq '{packets, errors}'
```

## Degradación {#impairment}

Un relé entre un cliente y su servidor que retrasa, descarta, duplica, corrompe,
reordena o limita lo que pasa, por UDP o TCP. Consulta
[degradación](../tools/impairment.md).

### netsim_start {#netsim_start}

Inicia un relé: lo que llega a `listen` sigue hacia `target`, y las respuestas
vuelven por el mismo camino, ambos degradados por el perfil. Sus recuentos
llegan como [`netsim://stat`](events.md#event-netsim-stat) cada 250 ms. *Inicia
una tarea* (`netsim`, `params.listen`, `params.target`, y `params.protocol`
para TCP).

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `config` | object | sí | Más abajo |

| Campo de `config` | Tipo | Predeterminado | Significado |
| --- | --- | --- | --- |
| `listen` | string | — | `IP:port` en el que escucha el relé; apunta el cliente aquí |
| `target` | string | — | `IP:port` del servidor real, o `host:port` — un nombre de host se resuelve una vez, cuando empieza el relé |
| `profile` | [`ImpairProfile`](#type-impairprofile) | — | Qué hacerle al tráfico |
| `seed` | number | nueva | La semilla de los sorteos: la misma semilla y el mismo tráfico dan los mismos descartes |
| `protocol` | string | `udp` | `udp` (datagramas) o `tcp` (flujos) |

**Resultado**: [`JobInfo`](#type-jobinfo).

**Errores**: `node.range` (un valor del perfil fuera de su rango, con `min`,
`max` y el campo), `node.too_long`, `node.bind_invalid`,
`transport.target_invalid`, `transport.dns` (un nombre de destino que no se
encuentra), fallos de enlace. La tarea termina con `wait.receive_failed` si un
socket ya no puede recibir.

### netsim_set_profile {#netsim_set_profile}

Un relé en marcha degrada con otro perfil a partir de ahora, sin cerrar sus
sockets.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `jobId` | number | sí | La tarea del relé |
| `profile` | [`ImpairProfile`](#type-impairprofile) | sí | El nuevo perfil |

**Resultado**: null.

**Errores**: `netsim.not_running`, `node.range`, `node.too_long`.

```bash
invoke netsim_start '{"config":{"listen":"127.0.0.1:9010","target":"127.0.0.1:9000","profile":{"latency_ms":80,"jitter_ms":20,"loss":0.02}}}'
```

## Tormenta y escáner {#storm-scanner}

::: danger
Una tormenta carga un destino tan fuerte como se lo pidas, y un escaneo sondea
cada puerto de un rango. Apúntalos solo a hosts de los que seas responsable.
:::

### storm_start {#storm_start}

Envía una carga constante de datagramas UDP o conexiones TCP a un destino. Sus
recuentos llegan como [`storm://stat`](events.md#event-storm-stat) cada 250 ms.
*Inicia una tarea* (`storm`, `params.protocol`, `params.target`,
`params.rate`).

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `config` | object | sí | Más abajo |

| Campo de `config` | Tipo | Predeterminado | Significado |
| --- | --- | --- | --- |
| `target` | string | — | `IP:port` o `host:port`; un nombre se resuelve una vez, cuando empieza la tarea |
| `protocol` | string | — | `udp`: datagramas; `tcp`: una conexión por unidad que escribe la carga útil y se cierra (cada conexión puede tardar 500 ms) |
| `size` | number | — | Bytes de carga útil, mantenidos entre 1 y 65 507 |
| `rate` | number | — | Unidades por segundo, según una planificación: la unidad *n* toca *n* / `rate` segundos después del inicio, y cada despertar envía lo que toca (como máximo 256; una planificación más atrasada omite las unidades más antiguas); 0 envía tan rápido como puede |
| `duration_s` | number | `0` | Parar tras estos segundos; 0: hasta que se detenga |

**Resultado**: [`JobInfo`](#type-jobinfo).

**Errores**: `transport.target_invalid`, `transport.dns`. Los envíos fallidos
se cuentan en los eventos, no se informan como errores.

### scan_start {#scan_start}

Prueba una conexión TCP a cada puerto de un rango e informa de los abiertos,
con lo que dice el servicio primero cuando se le pregunta. Los puertos abiertos
llegan como [`scan://open`](events.md#event-scan-open), el progreso como
[`scan://progress`](events.md#event-scan-progress). *Inicia una tarea* (`scan`,
`params.host`, `params.from`, `params.to`).

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `config` | object | sí | Más abajo |

| Campo de `config` | Tipo | Predeterminado | Significado |
| --- | --- | --- | --- |
| `host` | string | — | Un nombre de host o una dirección |
| `port_start`, `port_end` | number | — | El rango, ambos incluidos; dados al revés, se intercambian |
| `concurrency` | number | `256` | Intentos a la vez, de 1 a 1024 |
| `timeout_ms` | number | `600` | Por puerto, de 50 a 10 000 |
| `grab_banner` | boolean | `false` | Leer hasta 256 bytes que envíe el servicio dentro de los 400 ms siguientes a conectar |

**Resultado**: [`JobInfo`](#type-jobinfo).

**Errores**: `scan.host_required`.

```bash
invoke scan_start '{"config":{"host":"127.0.0.1","port_start":8000,"port_end":9100,"grab_banner":true}}'
```

## Inspector {#inspector}

El Inspector registra lo que envían y reciben las herramientas, como tramas,
mientras la captura está activada. En un servidor hay un Inspector para cada
página y script. Consulta [el Inspector](../tools/inspector.md).

### inspect_set_enabled {#inspect_set_enabled}

Activa o desactiva la captura. Mientras está desactivada no se registra nada.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `enabled` | boolean | sí | Activar (`true`) o desactivar |

**Resultado**: [`CaptureStats`](#type-frame).

### inspect_stats {#inspect_stats}

Los recuentos de la captura. Sin argumentos.

**Resultado**: [`CaptureStats`](#type-frame).

### inspect_snapshot {#inspect_snapshot}

Las tramas más recientes, la más antigua primero.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `limit` | number | sí | Cuántas, de 1 a 8192 |

**Resultado**: [`Frame`](#type-frame)`[]`, sin sus bytes (consulta
[`inspect_payload`](#inspect_payload)).

### inspect_clear {#inspect_clear}

Vacía la captura y sus recuentos. Sin argumentos.

**Resultado**: [`CaptureStats`](#type-frame).

### inspect_export {#inspect_export}

Escribe cada trama guardada en `capture-<ms>.jsonl` o `capture-<ms>.txt` en la
carpeta de datos. En `jsonl`, cada línea es una trama con los bytes que guarda
en `data`, en base64; `txt` es para leer, con un volcado hex de cada trama.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `format` | string | sí | `txt`; cualquier otra cosa escribe `jsonl` |

**Resultado**: string, la ruta escrita.

**Errores**: `inspect.empty`, `file.io`.

### inspect_payload {#inspect_payload}

Los bytes que guarda una trama, más allá de la vista previa de 1 KiB que
llevaba su lote.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `seq` | number | sí | El número de la trama |

**Resultado**: `{ "seq", "bytes", "kept", "dump", "hex" }` — `bytes`, el tamaño
de la trama; `kept`, cuántos de ellos se guardan (hasta 256 KiB); `dump`, cada
fila como `offset  hex  |ascii|`; `hex`, el hex plano que envía una repetición.

**Errores**: `inspect.frame_gone` (tramas más recientes ocuparon su lugar),
`inspect.no_payload` (solo se registró su tamaño).

```bash
invoke inspect_set_enabled '{"enabled":true}'
invoke inspect_snapshot '{"limit":20}' | jq '.[] | {seq, proto, dir, summary}'
```

## Biblioteca de señales {#signals}

La biblioteca es `signals.json` en la carpeta de datos. Solo es almacenamiento:
una señal se envía con el comando de su transporte (`osc_send`,
`broadcast_send`, `http_request`, `mqtt_publish` o `mqtt_publish_once`).
Consulta [señales](../tools/signals.md) y
[archivos](../reference/files.md#signals-json).

### signals_load {#signals_load}

Lee la biblioteca. Cuando el archivo no existe, se escribe primero el conjunto
inicial. Sin argumentos.

**Resultado**: `{ "path": string, "library": library, "seeded": boolean }` —
`seeded` es true cuando se acaba de escribir el conjunto inicial. La biblioteca
es `{ "version", "signals": [...], "folders": [...] }`: `version` 2 (un archivo
de versión 1 se devuelve tal cual), `folders` se omite cuando no hay ninguna.
Cada señal tiene `id`, `name`, `group` (su carpeta, `"A/B"`; vacío para
ninguna), `note` y `body`.

**Errores**: `signals.json_invalid` (con `path`, `line`, `column`; el archivo
nunca se sustituye), `file.io`.

### signals_save {#signals_save}

Sustituye todo el archivo de la biblioteca, a través de un archivo temporal en
la misma carpeta. Un archivo que existe y no se lee como biblioteca se deja
como está.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `library` | object | sí | `{ version, signals, folders }` tal como lo devuelve `signals_load` |

**Resultado**: string, la ruta escrita.

**Errores**: `signals.json_invalid` (el archivo que hay ahora en el disco no se
lee, con `path`, `line`, `column`; no se escribe nada), `signals.encode`,
`file.io`.

El `body` de una señal según su `transport`:

| `transport` | Campos |
| --- | --- |
| `osc` | `target`, `address`, `args` ([`OscArg`](#type-oscarg)`[]`) |
| `udp` | `target`, `payload`: `{ "kind": "text", "text" }` o `{ "kind": "hex", "hex" }` |
| `http` | `request` ([`HttpRequest`](#type-httprequest)) |
| `mqtt` | `broker` (`host:port`), `topic`, `payload`, `qos`, `retain` |

```bash
invoke signals_load | jq '.library.signals[] | {name, transport: .body.transport}'
```

## Emuladores {#emulators}

Un emulador es Signal Lab haciendo el otro lado: una API HTTP, un dispositivo
OSC, UDP o TCP, un bróker MQTT. Su documento — `name`, `bind`, `protocol`, las
reglas del protocolo y un `outage` opcional — se describe en
[emuladores](../tools/emulators.md). La biblioteca es `emulators.json` en la
carpeta de datos.

### emulators_load {#emulators_load}

Lee la biblioteca de emuladores. Cuando el archivo no existe, se escribe primero
el conjunto inicial. Sin argumentos.

**Resultado**: `{ "path", "library": { "version": 1, "emulators": [{ "id", "note", "emulator" }] }, "seeded" }`.

**Errores**: `emulators.json_invalid` (con `path`, `line`, `column`; nunca se
sustituye), `file.io`.

### emulators_save {#emulators_save}

Sustituye toda la biblioteca de emuladores, a través de un archivo temporal.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `library` | object | sí | `{ version, emulators }` tal como lo devuelve `emulators_load` |

**Resultado**: string, la ruta escrita.

**Errores**: `emulators.encode`, `file.io`.

### emulator_check {#emulator_check}

Si un emulador se iniciaría: todo lo que comprueba
[`emulator_start`](#emulator_start) antes de enlazar.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `emulator` | object | sí | El documento del emulador |
| `params` | objeto de cadenas | no | Valores que sus plantillas leen como parámetros |

**Resultado**: null cuando se iniciaría.

**Errores**: `emulator.*`; `node.*` por un campo que falta, está fuera de rango,
es demasiado largo o está mal formado (`node.required`, `node.range`,
`node.too_long`, `node.bind_invalid`, `node.target_invalid`,
`node.method_invalid`…); `param.unknown`, `template.*`, `osc.pattern_*`,
`regex.invalid`, `hex.invalid`. Cada uno lleva `rule`, `retained` o `response`
en `params` cuando el problema está en uno de ellos.

### emulator_start {#emulator_start}

Inicia un emulador como tarea propia. Su socket está abierto cuando vuelve el
comando. Lo que recibe y responde llega como
[`emulator://activity`](events.md#event-emulator-activity) cada 200 ms cuando
algo ha cambiado. *Inicia una tarea* (`emulator`, `params.name`,
`params.protocol`, `params.local`, y `params.source` cuando se da `source`).

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `emulator` | object | sí | El documento del emulador |
| `params` | objeto de cadenas | no | Valores que sus plantillas leen como parámetros |
| `seed` | number | no | Su semilla, de 0 a 9007199254740991; predeterminado: una nueva |
| `source` | string | no | La entrada de la biblioteca de la que viene, guardada en la tarea como `params.source` |

**Resultado**: [`JobInfo`](#type-jobinfo).

**Errores**: los de [`emulator_check`](#emulator_check), `seed.range`,
`transport.address_in_use` y los demás fallos de enlace.

### emulator_exchanges {#emulator_exchanges}

Lo que un emulador en marcha recibió y respondió. Guarda los últimos 500
intercambios.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `jobId` | number | sí | La tarea del emulador |
| `after` | number | no | Solo los intercambios numerados por encima de este; predeterminado 0 |
| `limit` | number | no | Como máximo esta cantidad, de 1 a 500; predeterminado 500 |

**Resultado**

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea |
| `name`, `protocol`, `local` | string | El emulador, su protocolo y la dirección en la que escucha |
| `counts` | object | `total`, `unmatched` (ninguna regla lo tomó), `failed`, `down` (llegó mientras estaba caído: su outage o [`emulator_down`](#emulator_down)), `hits` (por regla), y `missed` (MQTT: mensajes que un cliente demasiado atrasado no recibió; se omite mientras es 0) |
| `forced` | string | `unavailable`, `reset` o `timeout` mientras está caído a propósito; se omite en caso contrario |
| `exchanges` | object[] | Cada uno: `seq`, `ts`, `from`, `request`, `rule` (basado en 1; se omite cuando ninguna lo tomó), `reply`, `status`, `fault`, `ms`, `error`, `frame`, `down`, y `data` (la solicitud como la leen las plantillas) |

**Errores**: `emulator.not_running`.

### emulator_down {#emulator_down}

Deja caído un emulador en marcha hasta que se lo levante, diga lo que diga su
programa de caídas, o lo vuelve a levantar. Mientras está caído, un emulador
HTTP recibe cada solicitud con `fault`, un dispositivo TCP y un bróker MQTT
cortan sus conexiones y rechazan las nuevas, y los dispositivos OSC y UDP no
responden nada.

| Argumento | Tipo | Obligatorio | Significado |
| --- | --- | --- | --- |
| `jobId` | number | sí | La tarea del emulador |
| `down` | boolean | sí | Caído (`true`) o activo |
| `fault` | string | no | Lo que encuentran las solicitudes HTTP: `unavailable` (503, sin `Retry-After`: no se sabe cuándo vuelve), `reset` (la conexión se cierra), `timeout` (sin respuesta); predeterminado `unavailable` |

**Resultado**: null.

**Errores**: `emulator.not_running`.

```bash
invoke emulator_exchanges '{"jobId":5,"after":0}' | jq '.counts, (.exchanges[] | {request, rule, status})'
```

## Tipos compartidos {#types}

### JobInfo {#type-jobinfo}

Lo que devuelve un comando que inicia una tarea, y lo que enumera
[`jobs_list`](#jobs_list): `id`, `kind`, `label` (en inglés, para los
registros), `params` (los valores que nombra la etiqueta; se omite cuando no
hay ninguno) y `started_ms`. Consulta [tareas](index.md#jobs).

### OscArg {#type-oscarg}

Un argumento OSC, su tipo y su valor:

| `type` | `value` | Etiqueta OSC |
| --- | --- | --- |
| `int` | entero de 32 bits | `i` |
| `float` | número, enviado como float de 32 bits | `f` |
| `str` | string | `s` |
| `long` | entero de 64 bits | `h` |
| `double` | número, de 64 bits | `d` |
| `bool` | `true` o `false` | `T` o `F` |
| `blob` | matriz de bytes, `[222, 173]` | `b` |
| `nil` | ninguno: `{ "type": "nil" }` | `N` |

### HttpRequest {#type-httprequest}

| Campo | Tipo | Predeterminado | Significado |
| --- | --- | --- | --- |
| `method` | string | — | `GET`, `POST`… |
| `url` | string | — | `http://` o `https://` |
| `headers` | `[name, value][]` | `[]` | Encabezados de la solicitud |
| `body` | string o null | null | El cuerpo |
| `timeout_ms` | number | `10000` | Para todo el intercambio |
| `auth` | object | ninguno | `{ "scheme": "basic", "username", "password" }`, `{ "scheme": "digest", "username", "password" }` o `{ "scheme": "bearer", "token" }` |

Se siguen hasta 10 redirecciones. Las credenciales y cookies escritas para un
host nunca pasan a otro. Una solicitud Digest responde al desafío 401 del
servidor y envía de nuevo.

### HttpResponse {#type-httpresponse}

| Campo | Tipo | Significado |
| --- | --- | --- |
| `ok` | boolean | Un estado 2xx |
| `status`, `status_text` | number, string | El estado; 0 y vacío sin respuesta |
| `latency_ms` | number | Hasta que llegó todo el cuerpo |
| `headers` | `[name, value][]` | Encabezados de la respuesta |
| `body` | string | El cuerpo como texto, como máximo 256 KiB |
| `body_bytes` | number | El tamaño completo del cuerpo |
| `truncated` | boolean | `body` se cortó a 256 KiB |
| `error` | string o null | Por qué no hubo respuesta, cada capa de la causa |
| `cause` | string o null | Qué tipo de fallo: `refused`, `timeout`, `dns`, `unreachable`, `reset`, `address_in_use`, `address_unavailable`, `denied`, `tls`, `target_invalid`, `failed` — los mismos que los códigos `transport.*` |
| `digest` | object | Una solicitud Digest que solo encontró un 401; se omite en caso contrario. `challenged`: el desafío se respondió y la solicitud se envió de nuevo. `error`: por qué no pudo ser, un `EngineError` (`http.digest_not_offered`, `http.digest_unsupported`, `http.digest_invalid`, `http.digest_other_origin`) o null |

### Payload {#type-payload}

Lo que lleva un datagrama de difusión o descubrimiento:
`{ "kind": "osc", "address", "args" }`, `{ "kind": "text", "text" }` (enviado
tal cual, sin un cero final) o `{ "kind": "hex", "hex" }` (`de ad be ef`,
`deadbeef`, `0xDE,0xAD` — se ignora cualquier cosa que no sean dígitos hex).

### MqttConfig {#type-mqttconfig}

| Campo | Tipo | Predeterminado | Significado |
| --- | --- | --- | --- |
| `host` | string | — | El nombre o la dirección del bróker |
| `port` | number | — | Normalmente 1883 |
| `client_id` | string | — | No vacío; el bróker expulsa otra conexión con el mismo id |
| `username`, `password` | string | vacío | `username` se envía cuando no está vacío; `password` solo junto con un `username` |
| `keep_alive_s` | number | `60` | Los pings van a la mitad de él; 0: ninguno |
| `clean_session` | boolean | `true` | El flag de CONNECT |
| `will` | object o null | null | `{ topic, payload, qos, retain }`, publicado por el bróker si se pierde la conexión |
| `subscribe` | `{ filter, qos }[]` | `[]` | Suscrito en cuanto la conexión está activa |

### WsConfig {#type-wsconfig}

| Campo | Tipo | Predeterminado | Significado |
| --- | --- | --- | --- |
| `url` | string | — | `ws://` o `wss://` (`wss://` confía en lo que el sistema confía para HTTPS) |
| `headers` | `[name, value][]` | `[]` | Enviados con la solicitud de actualización |
| `protocols` | string[] | `[]` | Subprotocolos que ofrecer, por orden de preferencia |
| `timeout_ms` | number | `10000` | Para la conexión, el TLS y la actualización juntos |

Los mensajes son como máximo 16 MiB en ambos sentidos.

### ImpairProfile {#type-impairprofile}

Todos los campos son opcionales; lo que se omite no hace nada. Las
probabilidades van de 0 a 1.

| Campo | Rango | Significado | UDP | TCP |
| --- | --- | --- | --- | --- |
| `name` | como máximo 60 caracteres | Una etiqueta para la línea de tiempo y el informe | sí | sí |
| `latency_ms` | 0 a 60 000 | Retardo añadido a todo | sí | sí |
| `jitter_ms` | 0 a 60 000 | Hasta este tiempo más, sorteado cada vez | sí | sí |
| `loss` | 0 a 1 | Se descarta un datagrama | sí | — |
| `duplicate` | 0 a 1 | Un datagrama se envía dos veces | sí | — |
| `corrupt` | 0 a 1 | Se invierte un bit de un datagrama | sí | — |
| `reorder` | 0 a 1 | Un datagrama se retiene para que los posteriores lo adelanten | sí | — |
| `rate_kbps` | 0, o 8 a 10 000 000 | Límite de ancho de banda, kilobits por segundo; 0: ninguno | sí | sí |
| `burst_start` | 0 a 1 | Un datagrama inicia una ráfaga de pérdidas | sí | — |
| `burst_length` | 1 a 1000 | Datagramas que dura una ráfaga de media (necesario con `burst_start`) | sí | — |
| `offline` | `true` o `false` | No pasa nada | sí | sí |
| `reset` | 0 a 1 | Un fragmento de un flujo restablece su conexión | — | sí |
| `stall` | 0 a 1 | Un fragmento de un flujo deja su conexión semiabierta | — | sí |

### Frame y CaptureStats {#type-frame}

Una `Frame` es un paquete, una solicitud o un mensaje capturado:

| Campo | Significado |
| --- | --- |
| `seq` | Su número, creciente |
| `ts` | Cuándo, milisegundos desde 1970 |
| `proto` | `osc`, `udp`, `tcp`, `http`, `mqtt`, `ws`… |
| `dir` | `tx` (enviado) o `rx` (recibido) |
| `source` | La herramienta que la capturó: `osc-monitor`, `broadcast`, `netsim`… |
| `job_id` | Su tarea, o null |
| `local`, `remote` | Las direcciones: `IP:port` de este lado y del otro (una URL o un bróker para HTTP, WebSocket y MQTT). El `local` de una trama retransmitida por un relé es la dirección en la que escucha el relé y su `remote` adónde iba la trama; el sentido termina su `verdict` (`· client→target`, `· target→client`) |
| `bytes` | Su tamaño |
| `summary` | Una línea |
| `detail` | Una decodificación de varias líneas, o null |
| `hex` | Un volcado hex de los primeros 1 KiB, o null |
| `verdict` | Qué fue de ella — `dropped`, `sampled`, un estado — o null |
| `kept` | De `bytes`, cuántos se guardan (hasta 256 KiB); 0 cuando solo se registró el tamaño |
| `publish` | Solo una publicación MQTT: `{ broker, topic, qos, retain, text }` — el bróker como `host:port`, y si los bytes guardados, que son la carga útil del mensaje, son texto UTF-8. Ausente en cualquier otra trama |

`CaptureStats`: `enabled`, `total` (tramas registradas), `bytes`, `skipped`
(registradas pero nunca enviadas a la interfaz), `buffered` (tramas guardadas),
`capacity` (8192), `held` (bytes de carga útil guardados) y `held_limit`
(64 MiB). Las tramas más antiguas ceden al superar cualquiera de los dos
límites.

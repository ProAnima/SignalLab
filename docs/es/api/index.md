---
title: La API HTTP
description: Controla un servidor de Signal Lab desde scripts, CI y otras herramientas con los mismos comandos y eventos que usa su propia interfaz.
---

# La API HTTP

Para controlar Signal Lab desde un script, un pipeline de CI u otra herramienta, habla con un
servidor de Signal Lab (`signal-lab-server`, o la imagen de Docker) por HTTP. La interfaz que el
servidor muestra en un navegador usa exactamente esta API: cada botón es una llamada a
`/api/invoke/<command>`, cada número en vivo llega por `/api/events`. Así que todo lo que una
persona puede hacer en la página del servidor, un script también puede hacerlo.

La aplicación de escritorio no tiene API HTTP: su ventana llega a su motor dentro de la aplicación.
Para automatizar en un escritorio, ejecuta un servidor en el mismo equipo (consulta
[el servidor](../server/index.md)) o usa la línea de comandos
[`signallab`](../automation/cli.md).

## Endpoints {#endpoints}

| Método y ruta | Qué hace | Token |
| --- | --- | --- |
| `GET /api/health` | Si el servidor responde, su versión, si pide un token | no hace falta |
| `POST /api/invoke/<command>` | Ejecuta un comando del motor: argumentos JSON de entrada, resultado JSON de salida ([comandos](commands.md)) | hace falta |
| `POST /api/run` | Ejecuta un experimento hasta el final: el resultado, o sus pasos como líneas ([ejecuciones](run.md)) | hace falta |
| `GET /api/events` | WebSocket de cada evento del motor ([eventos](events.md)) | hace falta |
| `GET /api/files?path=…` | Un archivo que el motor escribió en la carpeta de datos, como descarga | hace falta |
| `GET /api/openapi.json` | Esta API descrita en OpenAPI 3.1 | hace falta |
| `GET /login`, `POST /login` | La página y el formulario de inicio de sesión para navegadores | no hace falta |
| `POST /logout` | Termina la sesión de un navegador | una sesión |

Cualquier otra ruta bajo `/api/` responde `404` con el código `api.not_found`; un método que una
ruta no acepta (`GET /api/invoke/…`) es `405`, con el cuerpo vacío. Todo lo demás es la interfaz;
en un servidor con token, un navegador sin sesión se envía primero a `/login`.

## Base URL {#base-url}

Un servidor escucha en `http://127.0.0.1:1430` a menos que se indique otra cosa con `--listen` (o
`SIGNALLAB_LISTEN`). La imagen de Docker escucha en todas las tarjetas de red, `0.0.0.0:1430`. Los
ejemplos de estas páginas usan:

```bash
SERVER=http://127.0.0.1:1430
```

El servidor habla HTTP sin cifrar. Para HTTPS, pon delante un proxy inverso que termine TLS e inicia
el servidor con `--secure-cookie`.

## Autenticación {#authentication}

Un servidor iniciado sin token escucha solo en loopback y no necesita autenticación: cualquiera en
ese equipo puede usarlo. Un servidor al que otros pueden llegar siempre tiene un token, y entonces
cada solicitud salvo `/api/health` y `/login` debe llevarlo.

**Los scripts** envían el token en el encabezado `Authorization`:

```bash
TOKEN=$(cat token.txt)
curl -fsS "$SERVER/api/invoke/jobs_list" -X POST \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json"
```

El encabezado debe ser exactamente `Bearer`, un espacio y el token. Un token que falta o es
incorrecto es `401` con el código `auth.required`.

**Los navegadores** inician sesión una vez en `/login` con el token y reciben una cookie de sesión,
`signallab_session`: `HttpOnly`, `SameSite=Strict`, válida durante 7 días y `Secure` cuando el
servidor funciona con `--secure-cookie`. `POST /logout` la termina. Un token incorrecto en el
formulario de inicio de sesión cuesta un segundo antes de la respuesta, lo que hace lento
adivinarlo. Las sesiones viven en la memoria del servidor: un reinicio cierra la sesión de todos los
navegadores, mientras que los scripts con el token no se ven afectados. El servidor conserva como
máximo 1024 sesiones; pasadas esas, se elimina la más antigua.

De dónde sale el token es cosa de la configuración del servidor (`--token-file`, `SIGNALLAB_TOKEN`
o el archivo `token` que `--generate-token` crea en la carpeta de datos): consulta
[seguridad del servidor](../server/security.md). Un token tiene al menos 24 caracteres y ningún
espacio; `signal-lab-server token` muestra uno nuevo.

::: warning
Cualquiera que tenga el token puede hacer que el servidor envíe tráfico. Mantén el archivo que lo
contiene legible solo por ti, y nunca lo pongas en una URL — el servidor no lee un token de ahí de
todas formas.
:::

## Host y origen {#host-origin}

Se ejecutan dos comprobaciones antes que nada, en cada ruta, `/api/health` incluida.

**`Host`.** El encabezado `Host` debe nombrar a este servidor:

- Los nombres de loopback siempre pasan: `localhost`, nombres que terminan en `.localhost`,
  `127.x.x.x` y `[::1]`.
- Los nombres indicados con `--allowed-host` (o `SIGNALLAB_ALLOWED_HOSTS`) pasan.
- Un servidor con token y sin `--allowed-host` responde a cualquier nombre.

Cualquier otra cosa es `403` con `auth.host`. Dirígete al servidor por un nombre que acepte:
`curl` envía el host de la URL que le das.

**`Origin`.** Una solicitud que cambia algo (cualquier método salvo `GET` y `HEAD`) y el upgrade
de WebSocket deben venir de la propia página del servidor cuando llevan un encabezado `Origin`: su
host y puerto deben ser iguales a `Host`. De lo contrario la respuesta es `403` con `auth.origin`;
`Origin: null` también se rechaza. Los scripts y `curl` no envían `Origin` y pasan esta
comprobación — siguen necesitando el token.

**Solo JSON.** `POST /api/invoke/…` y `POST /api/run` aceptan
`Content-Type: application/json` (parámetros como `; charset=utf-8` están bien). Cualquier otra
cosa es `415` con `command.json_required`. Una página web de otro sitio no puede enviar eso sin
preguntar antes al servidor, y el servidor nunca dice que sí.

## Llamar a un comando {#invoke}

```http
POST /api/invoke/<command>
Content-Type: application/json

{ "argument": "value", … }
```

- El cuerpo es un objeto JSON con los argumentos del comando. Un comando sin argumentos acepta `{}`
  o un cuerpo vacío (el encabezado `Content-Type` sigue haciendo falta).
- Los nombres de los argumentos son camelCase, como los envía la interfaz: `jobId`, `nodeId`. Los
  objetos pasados como argumento (`config`, `request`, `document`, `library`…) conservan los
  nombres de campo que escribe el motor, que son en su mayoría snake_case: `timeout_ms`,
  `port_start`.
- Un argumento que un comando no conoce es un error, nunca se ignora: `422` con
  `command.args_invalid` nombrando el comando, y las palabras del analizador en `detail`. También
  lo es un argumento obligatorio que falta. Un comando sin argumentos no lee el cuerpo en absoluto.
- Un argumento opcional puede dejarse fuera o enviarse como `null`.
- La respuesta es `200` con el resultado del comando como JSON. Un comando que no tiene nada que
  devolver responde `null`.

Cada comando, con sus argumentos y su resultado, está en [comandos](commands.md).

## Errores {#errors}

| Estado | Cuándo | Cuerpo |
| --- | --- | --- |
| `200` | El comando se ejecutó; `/api/run` inició la ejecución | El resultado |
| `400` | El cuerpo no es JSON; no se puede leer una solicitud de ejecución | `EngineError`: `command.args_invalid`, `api.run_invalid`, `api.run_source` |
| `400` | `/api/files` sin `path` | Texto sin formato del servidor web, no un `EngineError` |
| `401` | No hay token, o es incorrecto | `EngineError`: `auth.required` |
| `403` | Un `Host` o `Origin` que el servidor rechaza | `EngineError`: `auth.host`, `auth.origin` |
| `404` | No existe esa ruta de la API; un archivo que no está en la carpeta de datos | `EngineError`: `api.not_found`, `file.not_found` |
| `405` | Un método que la ruta no acepta | Vacío |
| `413` | Un cuerpo de solicitud de más de 24 MiB | Texto sin formato del servidor web, no un `EngineError` |
| `413` | Una descarga de más de 256 MiB | `EngineError`: `file.too_large` |
| `415` | No es `application/json` | `EngineError`: `command.json_required` |
| `422` | El comando falló, o la ejecución no pudo iniciarse | `EngineError`: cualquier código del motor |
| `500` | No se pudo leer un archivo | `EngineError`: `file.io` |

Un nombre de comando desconocido es `422` con `command.unknown`.

Cada fallo que informa el motor tiene una sola forma, el `EngineError`:

```json
{
  "code": "transport.refused",
  "params": { "target": "http://127.0.0.1:8080/" },
  "node": "request",
  "field": { "key": "url" },
  "detail": "error sending request for url (http://127.0.0.1:8080/): tcp connect error: Connection refused (os error 111)"
}
```

| Campo | Qué es |
| --- | --- |
| `code` | Qué fue mal: un identificador estable. Cada código y su mensaje están listados en [mensajes de error](../reference/errors.md), agrupados por la parte anterior al punto (por ejemplo [`transport`](../reference/errors.md#transport)) |
| `params` | Los valores que nombra el mensaje, todos como cadenas. Se omite cuando no hay ninguno |
| `node` | El nodo del experimento al que se refiere. Se omite cuando no hay ninguno |
| `field` | El campo al que se refiere: `key` (indicado en [campos](../reference/errors.md#fields)) e `index`, empezando en 1, para campos repetidos como un encabezado. Se omite cuando no hay ninguno |
| `detail` | Las palabras propias del sistema operativo, de un analizador o de una biblioteca, en inglés. Se omiten cuando no hay ninguna |

Ramifica por `code`, nunca por `detail`. Los valores secretos que usa una ejecución o un único
envío se enmascaran (`••••`) en cada error que informa.

Una solicitud que llega a su servidor pero recibe un estado de error, o ninguno, no es un comando
fallido: `http_request` responde `200` con la respuesta, y `ok`, `error` y `cause` dicen qué pasó.
Consulta [`http_request`](commands.md#http_request).

## Límites {#limits}

| Qué | Límite | Al llegar al límite |
| --- | --- | --- |
| Un cuerpo de solicitud | 24 MiB | `413` |
| Un documento de experimento | 4 MiB | `file.too_large` |
| Una descarga desde `/api/files` | 256 MiB | `413`, `file.too_large` |
| La duración de una ejecución | 300 s | La ejecución falla con `run.timeout` |
| El formulario de comentarios (`feedback_send`) | 15 MiB en total | `feedback.too_large` |
| Eventos en espera para un WebSocket | 4096 | Recibe [`server://lagged`](events.md#event-server-lagged) con cuántos se perdió |

## Eventos {#events}

`GET /api/events` convertido en un WebSocket transmite cada evento que envía el motor — los pasos
de una ejecución, el final de una tarea, los mensajes de un monitor, las tramas del Inspector — a
cada cliente conectado, como mensajes de texto:

```json
{ "event": "job://ended", "payload": { "job_id": 7, "kind": "storm", "error": null } }
```

Necesita el mismo token y las mismas reglas de `Origin` que el resto. Cada canal y su carga útil
están en [eventos](events.md).

## Archivos {#files}

`GET /api/files?path=<path>` descarga un archivo que el motor escribió en la carpeta de datos del
servidor: el informe de una ejecución (`report_path` del resultado de una ejecución), una
exportación del experimento o del Inspector, la biblioteca de señales o de emuladores. `path` es la
ruta que te dio el motor, en el servidor (codificada para URL):

```bash
curl -fsS -G "$SERVER/api/files" --data-urlencode "path=/data/runs/run-1759600000000-3.json" \
  -H "Authorization: Bearer $TOKEN" -o report.json
```

- Solo se sirven archivos dentro de la carpeta de datos. Cualquier otra cosa, una carpeta o un
  archivo que no existe, es `404` con `file.not_found`.
- La respuesta es `application/octet-stream` con `Content-Disposition: attachment`. Los caracteres
  del nombre del archivo que no sean letras, dígitos, `.`, `_` y `-` se convierten en `_`.
- Un archivo de más de 256 MiB es `413` con `file.too_large`.

Qué contiene la carpeta de datos está en [archivos y carpetas](../reference/files.md).

## Salud {#health}

`GET /api/health` está abierta: no necesita token, solo un `Host` que el servidor acepte.

```bash
curl -fsS "$SERVER/api/health"
```

```json
{ "status": "ok", "version": "[[version]]", "auth": true }
```

`auth` dice si las solicitudes necesitan un token. `signal-lab-server healthcheck` pregunta a la
misma dirección en la que escucha el servidor y termina con `0` cuando responde; la comprobación de
estado de la imagen de Docker lo ejecuta.

## Descripción OpenAPI {#openapi}

`GET /api/openapi.json` describe esta API en OpenAPI 3.1: los endpoints, cada comando con sus
argumentos, y la solicitud y el resultado de una ejecución. Necesita el token como el resto de
`/api/`. Estas páginas son la referencia completa; donde las dos difieran, estas páginas siguen al
motor.

## Tareas {#jobs}

El trabajo de larga duración — un monitor, un generador, una ráfaga, un relé, una conexión, un
emulador, una ejecución — es una tarea. Un comando que inicia una devuelve su `JobInfo` en cuanto
se ejecuta:

```json
{ "id": 4, "kind": "osc-monitor", "label": "OSC monitor 0.0.0.0:9000", "params": { "bind": "0.0.0.0:9000" }, "started_ms": 1759600000000 }
```

| Campo | Qué es |
| --- | --- |
| `id` | El número de la tarea, único mientras el servidor funciona; otros comandos lo toman como `jobId` o `id` |
| `kind` | `experiment`, `osc-monitor`, `osc-gen`, `http-burst`, `netsim`, `storm`, `scan`, `beacon`, `discovery`, `mqtt`, `websocket` o `emulator` |
| `label` | Una línea en inglés, para los registros |
| `params` | Los valores de los que se compone la etiqueta (destino, bind, host…). Se omiten cuando no hay ninguno |
| `started_ms` | Cuándo empezó, milisegundos desde 1970 |

Estos comandos inician una tarea: `experiment_start`, `osc_monitor_start`, `osc_generator_start`,
`http_burst_start`, `netsim_start`, `storm_start`, `scan_start`, `broadcast_beacon_start`,
`discovery_start`, `mqtt_connect`, `ws_connect` y `emulator_start`. El servidor registra cada inicio
con la dirección del cliente que lo pidió; `POST /api/run` también inicia una tarea.

- [`jobs_list`](commands.md#jobs_list) enumera las tareas en curso,
  [`job_stop`](commands.md#job_stop) detiene una,
  [`jobs_stop_all`](commands.md#jobs_stop_all) las detiene todas.
- Una tarea que termina sola, o falla, envía
  [`job://ended`](events.md#event-job-ended). Una tarea que detienes no envía nada más:
  `job_stop` respondiendo `true` es la confirmación.
- Las tareas pertenecen al servidor, no al cliente que las inició. Cerrar la página o terminar el
  script no las detiene, cada cliente las ve y puede detenerlas, y un servidor que se apaga las
  detiene todas.

## Un ejemplo completo {#example}

Pregunta si el servidor está activo, inicia un pequeño emulador HTTP con un comando, ejecuta el
experimento `http-check` incluido contra él y espera el resultado, y luego detén el emulador. `jq`
extrae campos de las respuestas.

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

`curl -f` convierte un estado de error en un comando fallido; omítelo (como en el paso 3) para ver
el `EngineError` en el cuerpo.

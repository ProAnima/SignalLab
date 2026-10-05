---
title: Eventos
description: El WebSocket de eventos del motor en un servidor de Signal Lab, y cada canal con su carga útil y cuándo se envía.
---

# Eventos

Todo lo que ocurre mientras se ejecuta una tarea — los pasos de una ejecución, los mensajes de un
monitor, las cifras de una ráfaga, las tramas del Inspector, el final de una tarea — se envía como
un evento. En un navegador, la página del servidor los recibe por un solo WebSocket,
`/api/events`; un script puede escuchar en el mismo socket. La aplicación de escritorio recibe los
mismos eventos, con los mismos nombres y cargas útiles, dentro de la aplicación.

## Suscribirse {#subscribe}

Abre un WebSocket a `/api/events` en el servidor:

```bash
websocat -H "Authorization: Bearer $TOKEN" ws://127.0.0.1:1430/api/events
```

- **Autenticación** es la del resto de la API: el token como
  `Authorization: Bearer`, o la cookie de sesión de un navegador. Sin eso, el
  upgrade se rechaza con `401` `auth.required`.
- **Origin**: un cliente que envía un encabezado `Origin` debe enviar el del propio servidor
  (host y puerto iguales a `Host`), o el upgrade se rechaza con `403`
  `auth.origin`. La mayoría de las bibliotecas de WebSocket fuera de un navegador no envían
  ninguno.
- **Todos los eventos a todos los clientes.** No hay nada a lo que suscribirse: cada
  socket recibe cada evento de cada tarea, la haya iniciado quien la haya iniciado. Elige lo
  que necesites por `event`, y por `job_id` en la carga útil.
- **Solo escuchar.** El servidor ignora lo que envía un cliente, salvo un cierre; un
  mensaje de más de 64 KiB cierra el socket.
- **Keep-alive.** El servidor hace ping cada 20 s, así que un socket silencioso sigue abierto
  a través de los proxys. Cuando el servidor se detiene, cierra cada socket.
- **Nada se reproduce.** Los eventos enviados mientras un cliente no estaba conectado se
  pierden para él. Un cliente que se reconecta debe leer el estado actual con
  comandos (`jobs_list`, `inspect_snapshot`, `emulator_exchanges`…).
- **Quedarse atrás.** Hasta 4096 eventos esperan a un socket. Un cliente que se queda aún más
  atrás recibe [`server://lagged`](#event-server-lagged) con cuántos se perdió.

## Formato del mensaje {#format}

Cada evento es un mensaje de texto que contiene un objeto JSON:

```json
{ "event": "scan://open", "payload": { "job_id": 9, "ts": 1759600000123, "port": 8080, "banner": null } }
```

| Campo | Qué es |
| --- | --- |
| `event` | El canal, más abajo |
| `payload` | Los valores del evento; su forma depende del canal |

Las horas (`ts`, el `first_ms` y el `last_ms` de un interlocutor) son milisegundos desde 1970;
las latencias y otras duraciones (`*_latency_ms`, `p50_ms`…, `ms`) son
milisegundos. Los errores de las cargas útiles son
objetos [`EngineError`](index.md#errors); sus códigos están listados en
[mensajes de error](../reference/errors.md).

## Canales {#channels}

| Canal | Lo envía | Cuándo |
| --- | --- | --- |
| [`experiment://step`](#event-experiment-step) | Una ejecución | Un paso empieza, se supera, falla, reintenta, repite o informa de una carga |
| [`experiment://ended`](#event-experiment-ended) | Una ejecución | Una vez, cuando la ejecución termina sola |
| [`job://ended`](#event-job-ended) | Cada tarea | Una vez, cuando la tarea termina sola o falla |
| [`osc://message`](#event-osc-message) | Monitor OSC | Cada paquete |
| [`osc://gen-tick`](#event-osc-gen-tick) | Generador OSC | Cada mensaje, o de 30 a 45 veces por segundo por encima de 60 mensajes por segundo |
| [`http://burst-progress`](#event-http-burst-progress) | Ráfaga HTTP | Cada 100 ms, y al final |
| [`ws://state`](#event-ws-state) | Conexión WebSocket | Conectada, cerrada |
| [`ws://messages`](#event-ws-messages) | Conexión WebSocket | Cada 100 ms con algo nuevo |
| [`mqtt://state`](#event-mqtt-state) | Conexión MQTT | Conectada, suscrita, cerrada |
| [`mqtt://messages`](#event-mqtt-messages) | Conexión MQTT | Cada 100 ms con algo nuevo |
| [`mqtt://ack`](#event-mqtt-ack) | Conexión MQTT | Se completó una publicación QoS 1/2; se respondió a una cancelación de suscripción |
| [`broadcast://emit-stat`](#event-broadcast-emit-stat) | Baliza | Cada 250 ms, y al final |
| [`broadcast://peers`](#event-broadcast-peers) | Escucha de descubrimiento | Cada 400 ms |
| [`netsim://stat`](#event-netsim-stat) | Relé de degradación | Cada 250 ms |
| [`storm://stat`](#event-storm-stat) | Tormenta | Cada 250 ms, y al final |
| [`scan://open`](#event-scan-open) | Escáner | Cada puerto abierto |
| [`scan://progress`](#event-scan-progress) | Escáner | Aproximadamente cada 1 % del rango, y al final |
| [`emulator://activity`](#event-emulator-activity) | Tarea de emulador | Cada 200 ms con algo nuevo |
| [`inspect://batch`](#event-inspect-batch) | Inspector | Cada 120 ms con tramas nuevas, aproximadamente una vez por segundo cuando no hay nada, mientras la captura está activada |
| [`server://lagged`](#event-server-lagged) | El servidor | Un cliente se quedó atrás |

### `experiment://step` {#event-experiment-step}

Un paso de una ejecución: un nodo que empieza, se supera, falla, espera para volver a intentarlo,
repite o informa del progreso de una carga. Una ejecución iniciada con `/api/run` envía
los mismos pasos en su respuesta (consulta [ejecuciones](run.md)).

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea de la ejecución |
| `ts` | number | Cuándo |
| `node_id` | string | El nodo |
| `state` | string | `running`, `passed`, `failed`, `retry` (un intento falló y el paso se ejecuta otra vez tras una pausa), `repeating` (el progreso de una acción que se repite, como máximo una vez por segundo) o `load` (el progreso de una carga, como máximo una vez por segundo) |
| `detail` | string | Qué ocurrió, en inglés; vacío para `running` y `failed` (consulta `error`) |
| `message_key` | string or null | El texto de la interfaz para ello, como clave de su diccionario |
| `message_params` | object or null | Los valores que nombra `message_key` |
| `vars` | object | Las variables que escribió el paso; se omite cuando no hay ninguna |
| `error` | `EngineError` | Por qué falló, o por qué falló el intento (`retry`); se omite en caso contrario |
| `frame` | number | La trama del Inspector del mensaje con el que coincidió una espera (o la respuesta esperada de un envío), cuando la captura estaba activada; se omite en caso contrario |
| `load` | object | Lo que midió una carga, leídos sus umbrales — en el último evento de un paso de carga, superado o fallido; se omite en caso contrario. Consulta [carga](../experiments/load.md) |

El nodo [[ui:exp.node.end]] de una ejecución muestra `running` cuando la primera rama lo alcanza y
`passed` cuando cada rama ha terminado sin un fallo. Los valores secretos se enmascaran en cada
campo.

### `experiment://ended` {#event-experiment-ended}

Una ejecución terminó sola: se superó, falló o se quedó sin tiempo. Se envía justo después de
la misma carga útil en [`job://ended`](#event-job-ended). Una ejecución detenida con
`job_stop` o [[ui:app.stopAll]] no envía ninguno de los dos y no guarda ningún informe.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea de la ejecución |
| `kind` | string | `experiment` |
| `seed` | number | La semilla con la que se ejecutó |
| `profile` | string or null | Su perfil |
| `overridden` | boolean | Algunos valores de parámetros vinieron de [[ui:exp.runWith]] o `overrides` |
| `error` | `EngineError` or null | El primer fallo de la ejecución; null cuando se superó |
| `report_path` | string or null | Su informe, en `runs/` de la carpeta de datos |
| `report_error` | `EngineError` or null | Por qué no se pudo escribir el informe |

### `job://ended` {#event-job-ended}

Una tarea terminó sola o falló. Una tarea detenida con `job_stop` o
`jobs_stop_all` no lo envía.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea |
| `kind` | string | `osc-monitor`, `osc-gen`, `http-burst`, `netsim`, `storm`, `scan`, `beacon`, `discovery`, `mqtt`, `websocket`, `emulator` o `experiment` |
| `error` | `EngineError` or null | Por qué terminó, cuando algo fue mal |

El `job://ended` de una ejecución lleva también los campos de
[`experiment://ended`](#event-experiment-ended). Qué termina cada tipo:

| `kind` | Termina cuando | `error` |
| --- | --- | --- |
| `osc-monitor` | El socket ya no puede recibir | `wait.receive_failed` |
| `osc-gen` | Su duración terminó, o falla un envío | null, o `transport.*` |
| `http-burst` | Se alcanza su total o su duración | null |
| `storm` | Su duración terminó | null |
| `scan` | Se probó cada puerto del rango | null |
| `beacon` | Sus rondas o su duración terminaron, o fallaron más de 32 envíos sin ninguno enviado | null, o `transport.*` |
| `discovery` | El socket ya no puede recibir | `wait.receive_failed` |
| `mqtt` | El bróker cerró la conexión o se perdió | `transport.*` (`transport.reset` cuando el bróker la cerró), o `mqtt.protocol` |
| `websocket` | La conexión se cerró | null, o por qué se perdió |
| `netsim` | El relé ya no puede funcionar | por qué |
| `emulator` | Su socket falla | por qué |
| `experiment` | La ejecución termina | el fallo de la ejecución, o null |

### `osc://message` {#event-osc-message}

Un paquete UDP que recibió un monitor OSC, decodificado. Se envía por cada paquete, sin
agrupar.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea del monitor |
| `ts` | number | Cuándo llegó |
| `from` | string | El emisor, `IP:port` |
| `bytes` | number | El tamaño del paquete |
| `messages` | object[] | Cada mensaje del paquete (un bundle tiene varios): `address` y `args` ([`OscArg`](commands.md#type-oscarg)`[]`) |
| `error` | `EngineError` or null | `osc.packet_malformed` cuando el paquete no se decodificó (entonces `messages` está vacío) |

### `osc://gen-tick` {#event-osc-gen-tick}

El progreso de un generador OSC: por cada mensaje por debajo de 60 mensajes por segundo;
por encima, por cada n-ésimo, siendo n la tasa dividida entre 30 y redondeada hacia abajo —
de 30 a 45 veces por segundo.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea del generador |
| `ts` | number | Cuándo |
| `value` | number | El valor recién enviado, antes de redondearse a un entero o a un float de 32 bits |
| `sent` | number | Mensajes enviados hasta ahora |

### `http://burst-progress` {#event-http-burst-progress}

Las cifras de una ráfaga HTTP, cada 100 ms mientras se ejecuta, y una vez más con
`done: true` cuando termina sola.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea de la ráfaga |
| `ts` | number | Cuándo |
| `sent` | number | Solicitudes respondidas o fallidas hasta ahora |
| `ok` | number | De esas, respondidas con un estado 2xx |
| `failed` | number | De esas, con cualquier otro estado o sin respuesta |
| `missed` | number | Las solicitudes de una ráfaga regulada que esperaron demasiado a un trabajador libre y se omitieron |
| `rps` | number | Solicitudes por segundo durante los últimos 100 ms; en el último evento, durante toda la ráfaga |
| `last_latency_ms`, `min_latency_ms`, `max_latency_ms`, `avg_latency_ms` | number | Latencias hasta ahora |
| `p50_ms`, `p90_ms`, `p95_ms`, `p99_ms` | number | Percentiles de cada solicitud hasta ahora, fallos incluidos, con un margen del 0,5 % |
| `done` | boolean | El último evento de la ráfaga |

### `ws://state` {#event-ws-state}

Una conexión WebSocket abierta por `ws_connect` se conectó o se cerró. Una
conexión cuya tarea se detuvo no envía `closed`.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea de la conexión |
| `ts` | number | Cuándo |
| `state` | string | `connected` o `closed` |
| `handshake` | object | `url`, `peer`, `local`, `protocol` (el subprotocolo que eligió el servidor, o null) y `ms` (la conexión y el upgrade) |
| `closed` | object or null | Con `closed`: `code`, `reason`, `by` (`client`, `server` o `lost`) y `error` |

### `ws://messages` {#event-ws-messages}

Lo que una conexión WebSocket envió y recibió desde el último evento, cada
100 ms cuando hay algo.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea de la conexión |
| `ts` | number | Cuándo |
| `messages` | object[] | En orden: `ts`, `dir` (`rx` recibido, `tx` enviado), `kind` (`text` o `binary`), `text` (los primeros 64 KiB como UTF-8, también en un mensaje binario; los bytes que no lo son se convierten en `�`), `hex` (los primeros 4096 bytes de un mensaje binario como hex, si no null), `bytes` (el tamaño completo) y `truncated` (más de lo que se mostró: más de 64 KiB de texto, más de 4096 bytes de binario) |
| `dropped` | number | Mensajes excluidos de este evento porque había más de 2000; los más antiguos van primero |

### `mqtt://state` {#event-mqtt-state}

El estado de una conexión MQTT cambió.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea de la conexión |
| `ts` | number | Cuándo |
| `state` | string | `connected`; `subscribed` tras cada respuesta a una suscripción; `closed` cuando la conexión terminó (no cuando se detuvo su tarea) |
| `broker` | string | `host:port` |
| `error` | `EngineError` or null | Por qué terminó una conexión `closed` (`transport.reset` cuando el bróker la cerró); null en caso contrario |
| `grants` | object[] | Con `subscribed`: cada filtro pedido, con `filter`, `qos` (concedido) y `accepted`; vacío en caso contrario |

### `mqtt://messages` {#event-mqtt-messages}

Lo que una conexión MQTT recibió desde el último evento, cada 100 ms cuando hay
algo. Un mensaje QoS 2 reentregado se muestra una vez.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea de la conexión |
| `ts` | number | Cuándo |
| `messages` | object[] | `ts`, `topic`, `payload` (como UTF-8; los bytes que no lo son se convierten en `�`), `bytes`, `qos`, `retain`, `dup` |
| `dropped` | number | Mensajes excluidos porque llegaron más de 4000 en 100 ms; los más antiguos van primero |

### `mqtt://ack` {#event-mqtt-ack}

El bróker completó algo que pidió la conexión.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea de la conexión |
| `ts` | number | Cuándo |
| `kind` | string | `published` (una publicación QoS 1 o 2 está completa) o `unsubscribed` |
| `packet_id` | number | El id del paquete MQTT |
| `topic` | string or null | El tema publicado; null para `unsubscribed` |

### `broadcast://emit-stat` {#event-broadcast-emit-stat}

Los contadores de una baliza, cada 250 ms, y una vez más cuando termina sola con
`pps` 0.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea de la baliza |
| `ts` | number | Cuándo |
| `targets` | number | Destinos en cada ronda |
| `rounds` | number | Rondas enviadas |
| `packets`, `bytes` | number | Datagramas y bytes enviados |
| `errors` | number | Envíos que fallaron |
| `pps` | number | Datagramas por segundo durante los últimos 250 ms |

### `broadcast://peers` {#event-broadcast-peers}

Lo que ha oído una escucha de descubrimiento, cada 400 ms.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea de la escucha |
| `ts` | number | Cuándo |
| `peers` | object[] | El oído más recientemente primero: `addr`, `proto`, `packets`, `bytes`, `first_ms`, `last_ms`, `last_summary`, `responded` (sus paquetes que se respondieron, contados según llegaron); como máximo 512 |
| `packets`, `bytes` | number | Todo lo recibido |
| `responses` | number | Respuestas enviadas |

### `netsim://stat` {#event-netsim-stat}

Los contadores de un relé de degradación, cada 250 ms. Los relés de los
nodos [[ui:exp.node.impairment]] de una ejecución informan en el informe de la ejecución.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea del relé |
| `ts` | number | Cuándo |
| `received`, `forwarded` | number | Datagramas o fragmentos de entrada y salida |
| `dropped` | number | Perdidos por `loss`, ráfagas u `offline` (UDP; un relé TCP retiene un flujo mientras está offline y no descarta nada) |
| `throttled` | number | UDP: descartados por el límite de ancho de banda, o porque ya había demasiados en camino. TCP: fragmentos que retuvieron su flujo por el límite de ancho de banda |
| `duplicated`, `corrupted`, `reordered` | number | Lo que el perfil les hizo |
| `bytes` | number | Bytes reenviados |
| `connections`, `reset`, `stalled` | number | TCP: conexiones tomadas, restablecidas, dejadas a medias; se omite mientras sea 0 |
| `profile` | string | El perfil con el que degrada ahora, como lo nombra la línea de tiempo: su nombre, o lo que hace (`60 ms ±25 · loss 2%`) |

### `storm://stat` {#event-storm-stat}

Los contadores de una tormenta, cada 250 ms, y una vez más cuando termina sola con
`pps` y `mbps` 0.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea de la tormenta |
| `ts` | number | Cuándo |
| `packets`, `bytes` | number | Datagramas (o conexiones TCP) y bytes enviados |
| `errors` | number | Envíos o conexiones que fallaron |
| `pps` | number | Por segundo durante los últimos 250 ms |
| `mbps` | number | Megabits por segundo durante los últimos 250 ms |

### `scan://open` {#event-scan-open}

El escáner encontró un puerto abierto.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea del escaneo |
| `ts` | number | Cuándo |
| `port` | number | El puerto |
| `banner` | string or null | Lo que envió primero el servicio, cuando se pidieron banners y dijo algo en menos de 400 ms |

### `scan://progress` {#event-scan-progress}

Cómo de lejos va un escaneo: aproximadamente cada 1 % del rango, y cuando termina solo
con `done` igual a `total` (ese puede llegar dos veces).

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea del escaneo |
| `ts` | number | Cuándo |
| `done` | number | Puertos probados |
| `total` | number | Puertos del rango |
| `open` | number | Puertos abiertos encontrados |

### `emulator://activity` {#event-emulator-activity}

Lo que un emulador iniciado con `emulator_start` recibió y respondió desde el
último evento, cada 200 ms cuando algo cambió (un intercambio, que se lo dejara caído o se lo
levantara, o un mensaje que el bróker MQTT no pudo entregar). Los nodos [[ui:exp.node.emulator]] de una ejecución no lo envían; sus contadores
están en el informe de la ejecución.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | number | La tarea del emulador |
| `ts` | number | Cuándo |
| `counts` | object | `total`, `unmatched`, `failed`, `down`, `hits` (por regla), y `missed` (MQTT; se omite mientras sea 0) — como [`emulator_exchanges`](commands.md#emulator_exchanges) |
| `forced` | string | `unavailable`, `reset` o `timeout` mientras está caído; se omite en caso contrario |
| `exchanges` | object[] | Los intercambios nuevos, como los enumera `emulator_exchanges` pero sin `data`; como máximo 200 |
| `dropped` | number | Intercambios más allá de los primeros 200 del intervalo, no enviados aquí; `emulator_exchanges` sigue teniendo los últimos 500 |

### `inspect://batch` {#event-inspect-batch}

Tramas nuevas del Inspector. Solo se envía mientras la captura está activada: cada 120 ms cuando
hay tramas nuevas, y aproximadamente una vez por segundo cuando no hay ninguna, para que los
contadores sigan al día.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `frames` | object[] | Las [tramas](commands.md#type-frame) nuevas, de la más antigua a la más reciente, como máximo 250; sin sus bytes (usa `inspect_payload`) |
| `stats` | object | Los contadores de la captura, [`CaptureStats`](commands.md#type-frame) |
| `skipped_now` | number | Tramas capturadas desde el último lote pero que no están en este — llegaron más de 250, o el búfer las dejó ir. Siguen en una exportación mientras el búfer las retenga |

### `server://lagged` {#event-server-lagged}

Solo servidor. Este cliente se quedó más de 4096 eventos atrás y se perdió algunos.
Lee el estado otra vez con comandos.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `skipped` | number | Cuántos eventos se perdió |

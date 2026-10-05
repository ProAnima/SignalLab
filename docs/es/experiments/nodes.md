---
title: Nodos
description: Cada tipo de nodo de un experimento — qué hace, sus campos con valores predeterminados y límites, sus salidas, los ajustes que admite y cómo se ve en un archivo de experimento.
---

# Referencia de nodos

Cada tipo de nodo que puede contener un experimento, en los grupos del menú de
añadir: [acciones](#actions), [esperas](#waits), [emulación](#emulation),
[fallos](#faults), [datos](#data), [comprobaciones](#checks) y [flujo](#flow). Cómo
añadirlos y conectarlos está en [El editor](index.md); `signallab nodes` imprime el
mismo catálogo como JSON, para scripts y asistentes ([La línea de
comandos](../automation/cli.md)).

## Cómo leer esta página {#reading}

Cada nodo tiene una tabla de sus campos:

- **Campo** es el nombre en el panel de propiedades; **En el archivo** es la clave
  en el JSON del experimento.
- **Predeterminado** es lo que recibe un nodo cuando lo añades en el editor. Donde un
  archivo puede omitir una clave, el valor que toma entonces se indica como *si
  falta*; las demás claves son obligatorias en un archivo.
- **Plantillas**: *sí* — el campo admite `{{templates}}`: parámetros, variables
  definidas antes, secretos y generadores, resueltos al ejecutarse el paso
  ([Datos y plantillas](data.md)). *Solo parámetros* — se abre antes del primer
  paso, cuando solo se conocen los parámetros. *No* — el valor se toma tal como se
  escribe.

Los tiempos están en milisegundos. Los límites se comprueban antes de que empiece una
ejecución; un campo fuera de rango impide que el experimento se ejecute y se muestra
en el nodo.

## Un nodo en un archivo {#file-shape}

En un archivo de experimento, un nodo es un objeto con un `id` (único en el
experimento), su `type`, su lugar en el lienzo (`x`, `y`, cero o más), sus campos y
los ajustes que usa (`retry`, `repeat`, `load`, omitidos cuando están desactivados).
Un cable es una arista desde la salida de un nodo (`port`, `next` si falta) hasta
otro nodo:

```json
{
  "nodes": [
    { "id": "start", "type": "start", "x": 40, "y": 80 },
    { "id": "ping", "type": "udp", "x": 270, "y": 80, "target": "127.0.0.1:9000", "text": "PING",
      "retry": { "attempts": 3, "delay_ms": 500, "backoff": "fixed" } },
    { "id": "end", "type": "end", "x": 500, "y": 80 }
  ],
  "edges": [
    { "from": "start", "to": "ping", "port": "next" },
    { "from": "ping", "to": "end", "port": "next" }
  ]
}
```

Los ejemplos de abajo muestran un nodo cada uno, tal como lo contiene un archivo.

## Ajustes compartidos por muchos nodos {#settings}

Se activan en la parte inferior de las propiedades de un nodo. Qué nodo admite cuál
se indica bajo cada nodo.

| Ajuste | Lo admite | Qué hace |
| --- | --- | --- |
| [Reintentar](#retry) | Nodos que envían o escuchan: [[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]], [[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_connect]], [[ui:exp.node.ws_send]], y todas las esperas | Vuelve a intentarlo cuando falla el paso |
| [Repetir](#repeat) | Nodos que envían: [[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]], [[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_send]] | Envía una y otra vez, un número de veces o durante un tiempo |
| [Carga](#load) | [[ui:exp.node.http]] | Envía la solicitud según un perfil de carga, medido y juzgado por umbrales |
| [Esperar una respuesta](#reply) | [[ui:exp.node.osc]], [[ui:exp.node.udp]] | Envía y espera la respuesta en el mismo paso |

### Reintentar {#retry}

[[ui:exp.retryOn]]: cuando falla el paso — sin conexión, un tiempo de espera
agotado, una espera sin nada que coincida — hace una pausa y se ejecuta de nuevo.
Cada intento fallido es una fila en la línea de tiempo; el paso falla cuando falla el
último intento. Una plantilla que no se puede resolver no se reintenta.
[[ui:common.stop]] también termina una pausa.

| Campo | En el archivo | Qué | Predeterminado y límites |
| --- | --- | --- | --- |
| [[ui:exp.attempts]] | `retry.attempts` | Intentos en total, el primero incluido | 3; 2–10 en el editor (un archivo también puede decir 1) |
| [[ui:exp.retryDelay]] | `retry.delay_ms` | La pausa antes del segundo intento | 500; 0–60 000 |
| [[ui:exp.backoff]] | `retry.backoff` | [[ui:exp.backoff.fixed]] (`fixed`): la misma pausa cada vez; [[ui:exp.backoff.exponential]] (`exponential`): el doble de larga tras cada fallo | `fixed` (también si falta) |

Ninguna pausa dura más de 60 segundos, por mucho que se duplique. Una espera cuya
salida [[ui:exp.portTimeout]] tiene un cable no falla por un tiempo de espera
agotado — sale por esa salida — así que entonces no se reintenta.

### Repetir {#repeat}

[[ui:exp.repeatOn]]: el nodo envía una y otra vez — un latido, un sondeo, un flujo
constante — sin un bucle en el grafo. Cada envío lee sus plantillas de nuevo
(`{{counter}}` es su número, `{{now}}` su hora), y Reintentar, cuando está activado,
se aplica a cada envío. El paso se supera cuando lo hizo cada envío; un envío que
falla definitivamente falla el paso. La línea de tiempo informa del progreso como
máximo una vez por segundo.

| Campo | En el archivo | Qué | Predeterminado y límites |
| --- | --- | --- | --- |
| [[ui:exp.repeatBy]] | `repeat.until` | [[ui:exp.repeatBy.count]] (`count`) o [[ui:exp.repeatBy.duration]] (`duration`) | `count` (también si falta) |
| [[ui:exp.repeatCount]] | `repeat.count` | Envíos en total, el primero incluido | 10 (también si falta); 2–10 000 |
| [[ui:exp.repeatDuration]] | `repeat.duration_ms` | Cuánto tiempo seguir enviando, desde el primer envío | 10 000 (también si falta); 1–300 000 |
| [[ui:exp.repeatInterval]] | `repeat.interval_ms` | La pausa entre dos envíos | 1 000; 10–60 000; obligatorio en un archivo |
| [[ui:exp.repeatJitter]] | `repeat.jitter_ms` | Cada pausa hasta este tiempo más larga, extraída de la semilla de la ejecución | 0 (también si falta); 0–60 000 |

Las repeticiones deben caber en los 300 segundos de una ejecución, y *durante un
tiempo* debe necesitar menos de 10 000 envíos (su tiempo dividido entre el
intervalo).

### Carga {#load}

[[ui:exp.loadOn]], solo en un [[ui:exp.node.http]]: la solicitud se envía según un
perfil — una tasa constante, una rampa, escalones, un pico o llegadas aleatorias —
con hasta 512 en vuelo a la vez (32 por defecto), y se mide: latencias, errores, la
tasa alcanzada. Los umbrales deciden si el paso se supera. La carga sustituye a
Repetir y Reintentar (una solicitud fallida se cuenta, no se reintenta) y no deja
respuesta para las comprobaciones posteriores. Sus campos y resultados están en
[Pruebas de carga](load.md).

### Esperar una respuesta {#reply}

[[ui:exp.expectReply]], en un [[ui:exp.node.osc]] o un [[ui:exp.node.udp]]: el
mensaje se envía desde el puerto en el que se espera la respuesta, así que se oye a
un dispositivo que responde al remitente, y el paso solo se supera cuando llega una
respuesta que coincide a tiempo. Sin respuesta, el paso falla — Reintentar envía de
nuevo. La respuesta se guarda en una variable, como la de una espera.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.replyOn]] | `reply.bind` | `IP:port` desde el que enviar y en el que escuchar; el puerto 0 toma cualquier puerto libre | `0.0.0.0:0` | No |
| [[ui:exp.replyAddress]] (OSC) | `reply.address` | El patrón de dirección de la respuesta, como en [[[ui:exp.node.wait_osc]]](#node-wait_osc) | `/*` | Sí |
| [[ui:exp.argRules]] (OSC) | `reply.args` | Reglas de argumentos, como en [[ui:exp.node.wait_osc]] | ninguna; como máximo 16 | Valores: sí |
| [[ui:exp.replyMode]] (UDP) | `reply.mode` | `any`, `contains`, `regex` o `hex` — consulta [Coincidencia de cargas útiles](#payload-matching) | `any` (también si falta) | No |
| [[ui:field.pattern]] (UDP) | `reply.pattern` | Lo que la respuesta debe contener o con lo que debe coincidir | vacío; obligatorio salvo `any` | Sí |
| [[ui:exp.waitTimeout]] | `reply.timeout_ms` | Cuánto esperar | 2 000 (también si falta); 1–120 000 | No |
| [[ui:exp.replyVariable]] | `reply.variable` | La variable en la que se guarda la respuesta | `reply` (también si falta) | No |

El puerto de la respuesta se abre antes del primer paso, como el de una espera.
## Acciones {#actions}

Nodos que envían. Una espera tras una acción cuenta los mensajes desde el momento en
que empezó la acción.

### Solicitud HTTP {#node-http}

Envía una solicitud HTTP y guarda la respuesta para las comprobaciones, las ramas y
los nodos [[ui:exp.node.extract]] posteriores.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.method]] | `request.method` | GET, HEAD, POST, PUT, PATCH, DELETE u OPTIONS (un archivo puede nombrar cualquier método) | `GET` | No |
| URL | `request.url` | Una URL `http://` o `https://` | `http://127.0.0.1:8080/` | Sí |
| [[ui:common.timeoutMs]] | `request.timeout_ms` | Para todo el intercambio | 4 000 (10 000 si falta); 1–120 000 | No |
| [[ui:exp.headers]] | `request.headers` | `[[name, value], …]`; se omite una fila con un nombre vacío | ninguna | Sí, nombres y valores |
| [[ui:exp.body]] | `request.body` | Texto, o `null` para ninguno | `null` | Sí |
| [[ui:field.auth]] | `request.auth` | [[ui:http.auth.none]], [[ui:http.auth.basic]], [[ui:http.auth.bearer]] o [[ui:http.auth.digest]], con [[ui:field.username]] y [[ui:field.password]], o [[ui:field.token]] | ninguno | Sí |

- Cualquier respuesta supera el paso, 404 y 500 incluidos: comprueba el estado con
  [[[ui:exp.node.assert_status]]](#node-assert_status) o ramifícalo con
  [[[ui:exp.node.branch_status]]](#node-branch_status). Una solicitud que no obtiene
  respuesta — rechazada, un tiempo de espera agotado, un nombre que no se resuelve,
  un certificado que no es de confianza — falla el paso.
- Se siguen las redirecciones, como máximo diez. Los certificados `https://` se
  verifican.
- El cuerpo de la respuesta se conserva hasta 256 KiB para las comprobaciones; un
  cuerpo mayor se corta ahí (las comprobaciones lo indican cuando lo que buscan puede
  estar más allá del corte).
- Digest responde al desafío 401 del servidor y envía la solicitud de nuevo. Las
  credenciales solo entran en la solicitud: los pasos, los informes y el Inspector
  nunca muestran el encabezado `Authorization`. Escribe una contraseña como
  `{{secret.NAME}}`.
- Mientras el experimento conserva las cookies (activado por defecto, en
  [[ui:exp.params]]), lo que los servidores establecen se devuelve con las solicitudes
  posteriores de la ejecución a ellos.

Salidas: [[ui:exp.outputPort]]. Ajustes: Reintentar, Repetir, Carga.

```json
{ "id": "cue", "type": "http", "x": 270, "y": 80,
  "request": { "method": "POST", "url": "{{api}}/cue", "headers": [["Content-Type", "application/json"]],
               "body": "{\"cue\": 1}", "timeout_ms": 5000,
               "auth": { "scheme": "bearer", "token": "{{secret.API_TOKEN}}" } } }
```

Véase también [HTTP](../protocols/http.md).

### Mensaje TCP {#node-tcp}

Se conecta a un host por TCP, escribe la carga útil, espera hasta 250 ms los primeros
bytes de una respuesta (lee como máximo 1 024 bytes, una vez) y cierra la conexión.
El tamaño de la respuesta se informa, no se comprueba.

En el [Inspector](../tools/inspector.md) el paso son dos tramas `tcp` con el origen
`experiment`: la carga útil escrita y, cuando llegó una, la respuesta leída. Los
secretos en uso se enmascaran en ambas, como en cualquier trama.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.host]] | `host` | Un nombre de host o una dirección IP | `127.0.0.1` | Sí |
| [[ui:exp.port]] | `port` | | 9000; 1–65 535 | No |
| [[ui:common.timeoutMs]] | `timeout_ms` | Para conectarse, escribir y la respuesta en conjunto | 4 000 (también si falta); 1–120 000 | No |
| [[ui:exp.payload]] | `payload` | El texto escrito una vez conectado, como UTF-8 | `hello` | Sí |

El paso falla cuando se rechaza la conexión, no se resuelve el nombre o se agota el
tiempo. Salidas: [[ui:exp.outputPort]]. Ajustes: Reintentar, Repetir.
[[ui:exp.sendNow]] se conecta y escribe la carga útil una vez, y el resultado del nodo
indica cuántos bytes se enviaron y volvieron.

```json
{ "id": "go", "type": "tcp", "x": 270, "y": 80, "host": "127.0.0.1", "port": 5000, "payload": "GO\r\n", "timeout_ms": 2000 }
```

### Mensaje OSC {#node-osc}

Envía un mensaje OSC 1.0 por UDP.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:common.target]] | `target` | `IP:port` o `host:port`; un nombre de host se resuelve cuando el paso envía, y se toma su dirección IPv4 cuando la tiene | `127.0.0.1:9000` | Sí |
| [[ui:common.address]] | `address` | Empieza por `/` | `/test` | Sí |
| [[ui:exp.arguments]] | `args` | `[{ "type", "value" }, …]` — `int`, `float`, `str`, `long`, `double`, `bool`, `blob` (bytes), `nil` (sin valor) | ninguna | Valores de texto (`str`): sí |
| [[ui:exp.expectReply]] | `reply` | Opcional: enviar y esperar la respuesta — consulta [Esperar una respuesta](#reply) | Desactivado | |

Salidas: [[ui:exp.outputPort]]; cuando se espera una respuesta, solo se sigue cuando
llegó la respuesta. Ajustes: Reintentar, Repetir, una respuesta. ⚡
[[ui:exp.routeThrough]] en sus propiedades pone un
[[[ui:exp.node.impairment]]](#node-impairment) delante.

```json
{ "id": "fader", "type": "osc", "x": 270, "y": 80, "target": "{{device}}", "address": "/fader/1",
  "args": [{ "type": "float", "value": 0.75 }] }
```

Véase también [OSC](../protocols/osc.md).
### Datagrama UDP {#node-udp}

Envía una carga útil de texto como un datagrama UDP a uno o varios destinos.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:common.target]] | `target` | `IP:port` o `host:port`; varios separados por comas, puntos y comas o saltos de línea reciben cada uno el datagrama. Un nombre de host se resuelve cuando el paso envía, y se toma su dirección IPv4 cuando la tiene | `127.0.0.1:9000` | Sí |
| [[ui:exp.payload]] | `text` | La carga útil, como UTF-8 | `hello`; como máximo 65 507 bytes | Sí |
| [[ui:exp.expectReply]] | `reply` | Opcional: enviar y esperar la respuesta — consulta [Esperar una respuesta](#reply) | Desactivado | |

El paso falla si no se puede alcanzar algún destino. Salidas:
[[ui:exp.outputPort]]. Ajustes: Reintentar, Repetir, una respuesta.

```json
{ "id": "ping", "type": "udp", "x": 270, "y": 80, "target": "{{device}}", "text": "PING {{run.id}}",
  "reply": { "bind": "0.0.0.0:0", "mode": "contains", "pattern": "PONG", "timeout_ms": 1000, "variable": "pong" } }
```

### Publicación MQTT {#node-mqtt}

Se conecta a un bróker MQTT, publica un mensaje y se desconecta. La conexión es MQTT
3.1.1 sobre TCP sin cifrar, con una sesión limpia y sin nombre de usuario ni
contraseña. La conexión, la publicación y el acuse del bróker deben ocurrir en 15
segundos.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.broker]] | `host` | El nombre o la dirección del bróker | `127.0.0.1` | Sí |
| [[ui:exp.port]] | `port` | | 1883; 1–65 535 | No |
| [[ui:exp.topic]] | `topic` | Sin comodines (`+`, `#`) | `lab/test` | Sí |
| [[ui:exp.payload]] | `payload` | El mensaje, como texto | `hello` | Sí |
| QoS | `qos` | 0, 1 o 2 | 0 | No |
| [[ui:exp.retain]] | `retain` | `true`: el bróker lo conserva como el valor del tema | `false` | No |

Las seis claves son obligatorias en un archivo. El paso falla cuando no se puede
alcanzar el bróker o rechaza la conexión o el mensaje. Salidas:
[[ui:exp.outputPort]]. Ajustes: Reintentar, Repetir.

```json
{ "id": "light", "type": "mqtt", "x": 270, "y": 80, "host": "{{broker}}", "port": 1883,
  "topic": "lab/light/1/set", "payload": "on", "qos": 1, "retain": false }
```

Véase también [MQTT](../protocols/mqtt.md).

### Conexión WebSocket {#node-ws_connect}

Abre un WebSocket para el resto de la ejecución, o hasta un
[[[ui:exp.node.ws_close]]](#node-ws_close). Lo que llega a partir de entonces se
guarda para los pasos [[[ui:exp.node.wait_ws]]](#node-wait_ws) sobre él. La URL y los
encabezados se resuelven cuando se ejecuta el paso, así que un token extraído antes
puede estar en ellos. Si se ejecuta de nuevo — en un [[ui:exp.node.loop]] — cierra
primero su conexión anterior y abre una nueva. Cuando termina la ejecución, de
cualquier forma, sus conexiones se cierran con una trama de cierre.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| URL | `url` | Una URL `ws://` o `wss://` | `ws://127.0.0.1:9001/` | Sí |
| [[ui:exp.headers]] | `headers` | `[[name, value], …]` enviados con la solicitud de mejora | ninguno | Sí, nombres y valores |
| [[ui:exp.wsProtocols]] | `protocols` | Subprotocolos que ofrecer, por orden de preferencia; el servidor elige uno | ninguno | No |
| [[ui:common.timeoutMs]] | `timeout_ms` | Para conectarse y la mejora | 5 000 (10 000 si falta); 1–120 000 | No |

`wss://` confía en los mismos certificados que `https://`. El paso falla cuando falla
la conexión o la mejora; el estado del servidor está en el motivo. Salidas:
[[ui:exp.outputPort]]. Ajustes: Reintentar (no Repetir).

```json
{ "id": "socket", "type": "ws_connect", "x": 270, "y": 80, "url": "ws://127.0.0.1:9001/chat",
  "headers": [["Authorization", "Bearer {{token}}"]], "protocols": ["chat.v1"], "timeout_ms": 5000 }
```

Véase también [WebSocket](../protocols/websocket.md).

### Envío WebSocket {#node-ws_send}

Envía un mensaje por la conexión que abrió un [[ui:exp.node.ws_connect]].

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | El id de un nodo [[ui:exp.node.ws_connect]] de este experimento | el primero | No |
| [[ui:exp.wsFormat]] | `binary` | [[ui:exp.wsText]] (`false`), o [[ui:exp.wsBinary]] (`true`): la carga útil son bytes escritos en hex, `de ad be ef` | `false` (también si falta) | No |
| [[ui:exp.payload]] | `text` | El mensaje | `hello`; como máximo 16 MiB | Sí |

La conexión debe preceder al envío en su ruta; un envío cuya conexión no está
abierta falla. Las respuestas cuentan desde el momento en que se escribe el mensaje.
Salidas: [[ui:exp.outputPort]]. Ajustes: Reintentar, Repetir.

```json
{ "id": "hello", "type": "ws_send", "x": 500, "y": 80, "connection": "socket",
  "text": "{\"type\":\"ping\",\"id\":\"{{uuid}}\"}", "binary": false }
```

### Cierre WebSocket {#node-ws_close}

Cierra una conexión con un saludo de cierre. La línea de tiempo dice quién la cerró:
este paso, el servidor antes (con su código), o una conexión que se había roto.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | El id de un nodo [[ui:exp.node.ws_connect]] | el primero | No |
| [[ui:field.code]] | `code` | 1000 (normal), o 3000–4999 para uno propio de la aplicación | 1000 (también si falta) | No |
| [[ui:field.reason]] | `reason` | Se envía con el código | vacío; como máximo 123 bytes, tras las plantillas | Sí |

Salidas: [[ui:exp.outputPort]]. Sin ajustes.

```json
{ "id": "bye", "type": "ws_close", "x": 960, "y": 80, "connection": "socket", "code": 1000, "reason": "done" }
```

### Marca de registro {#node-log}

Escribe una línea en la línea de tiempo y el informe — un punto de control, o los
valores a los que llegó una ejecución.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.logMessage]] | `message` | El texto | `Check point`; como máximo 10 000 caracteres | Sí |

Salidas: [[ui:exp.outputPort]]. Sin ajustes.

```json
{ "id": "ready", "type": "log", "x": 500, "y": 80, "message": "device {{device}} ready" }
```

## Esperas {#waits}

El grupo [[ui:exp.group.observe]]: nodos que esperan que llegue algo. Comparten estas
reglas:

- **Escuchan desde el principio de la ejecución.** El puerto o la suscripción al
  bróker de una espera se abre antes del primer paso, así que no se pierde un
  dispositivo que responde más rápido de lo que empieza el siguiente paso. Dos esperas
  en la misma dirección comparten un socket.
- **Cuentan desde la última acción de su rama.** Un mensaje que llegó antes de la
  última solicitud de la rama no es una respuesta a ella; antes de cualquier acción,
  cuenta todo desde que empezó la ejecución.
- **Se toma el primer mensaje que coincide.** Un mensaje que tomó una espera no lo ve
  otra.
- **[[ui:exp.portMatched]] o [[ui:exp.portTimeout]].** Cuando coincide, el mensaje se
  guarda en la variable de la espera y el flujo sigue por [[ui:exp.portMatched]].
  Cuando se agota el tiempo, sigue por [[ui:exp.portTimeout]] si esa salida tiene un
  cable; si no, el paso falla, indicando cuántos otros mensajes llegaron.
- Cada socket conserva los 1 024 mensajes más recientes (y 64 MiB); los más antiguos
  se descartan, y un tiempo de espera agotado dice cuántos eran.
- [[ui:exp.listenNow]] escucha con ese único paso, a partir de ahora.

Salidas: [[ui:exp.portMatched]] (obligatoria), [[ui:exp.portTimeout]] (opcional).
Ajustes: Reintentar.

### Coincidencia de cargas útiles {#payload-matching}

[[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_ws]] y una
respuesta UDP eligen cómo debe ser la carga útil:

| Opción | En el archivo | Coincide cuando la carga útil |
| --- | --- | --- |
| [[ui:exp.mode.any]] | `any` | es cualquier cosa |
| [[ui:exp.mode.contains]] | `contains` | leída como texto UTF-8, contiene el patrón (distingue mayúsculas y minúsculas) |
| [[ui:exp.mode.regex]] | `regex` | leída como texto UTF-8, coincide con la expresión regular |
| [[ui:exp.mode.hex]] | `hex` | contiene los bytes, escritos como pares hex: `de ad be ef`, `deadbeef`, `0xde,0xad`, `DE:AD` |

El mensaje que coincidió se guarda como un objeto. Los pasos posteriores leen sus
campos como `{{reply.text}}` (con el nombre de la variable en lugar de `reply`):

| Campo | Qué |
| --- | --- |
| `text` | La carga útil como texto |
| `hex`, `bytes` | La carga útil en hex (sus primeros 1 024 bytes), y su tamaño en bytes |
| `match` | Lo que coincidió: el texto, el primer grupo de la expresión regular (o toda la coincidencia), o los bytes |
| `from` | El `IP:port` del remitente |
| `ms` | Milisegundos desde la última acción de la rama (o el inicio de la ejecución) hasta el mensaje |
| `topic` | [[ui:exp.node.wait_mqtt]]: el tema en el que se publicó |
| `json`, `kind` | [[ui:exp.node.wait_ws]]: el mensaje analizado como JSON (`null` cuando no lo es), y `text` o `binary` |
### Esperar OSC {#node-wait_osc}

Espera un mensaje OSC cuya dirección coincide con un patrón y cuyos argumentos
cumplen todas las reglas. En un bundle, se toma el primer mensaje que coincide.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | `IP:port` en el que escuchar; `0.0.0.0` para todas las tarjetas de red | `127.0.0.1:9001` | No |
| [[ui:exp.addressPattern]] | `address` | `*` cualquier carácter, `?` uno, `[0-9]` un conjunto (`[!0-9]` fuera de él), `{ping,pong}` cualquiera de los dos; los comodines se quedan dentro de un segmento `/` | `/pong`; como máximo 512 caracteres | Sí |
| [[ui:exp.argRules]] | `args` | `[{ "index", "op", "value" }, …]`: el argumento `index` comparado con `value` por `op` ([Comparaciones](#comparisons)); deben cumplirse todas | ninguna; como máximo 16, índice 0–63 | Valores: sí |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (también si falta); 1–120 000 | No |
| [[ui:exp.replyVariable]] | `variable` | Dónde se guarda el mensaje | `reply` (también si falta) | No |

Un argumento se compara como texto: los números tal como se escriben, las cadenas sin
comillas, `true`/`false`, un blob en hex. Una regla sobre un argumento que el mensaje
no tiene no se cumple. El mensaje guardado tiene `address`, `args`
(`{{reply.args[0]}}`), `from` y `ms`.

```json
{ "id": "status", "type": "wait_osc", "x": 500, "y": 80, "bind": "0.0.0.0:9001", "address": "/status",
  "args": [{ "index": 0, "op": "eq", "value": "ready" }], "timeout_ms": 5000, "variable": "reply" }
```

### Esperar UDP {#node-wait_udp}

Espera un datagrama UDP cuya carga útil coincide.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | `IP:port` en el que escuchar | `127.0.0.1:9001` | No |
| [[ui:exp.waitMode]] | `mode` | Consulta [Coincidencia de cargas útiles](#payload-matching) | `contains` (`any` si falta) | No |
| [[ui:field.pattern]] | `pattern` | Lo que la carga útil debe contener o con lo que debe coincidir | `pong`; obligatorio salvo `any` | Sí |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (también si falta); 1–120 000 | No |
| [[ui:exp.replyVariable]] | `variable` | | `reply` (también si falta) | No |

```json
{ "id": "ready", "type": "wait_udp", "x": 500, "y": 80, "bind": "0.0.0.0:9002", "mode": "contains",
  "pattern": "READY", "timeout_ms": 5000, "variable": "reply" }
```

### Esperar MQTT {#node-wait_mqtt}

Espera un mensaje publicado en un tema de un bróker, cuya carga útil coincide. La
ejecución se conecta y se suscribe antes de su primer paso. Los mensajes retenidos
que el bróker reproduce al suscribirse se ignoran: solo cuenta lo que se publica
después de que empezó la ejecución.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.broker]] | `host` | El bróker | `127.0.0.1` | Solo parámetros |
| [[ui:exp.port]] | `port` | | 1883; 1–65 535 | No |
| [[ui:exp.topicFilter]] | `topic` | Un filtro: `+` es un nivel cualquiera, `#` todo lo que hay debajo (solo al final) | `lab/#` | Solo parámetros |
| [[ui:exp.waitMode]] | `mode` | Consulta [Coincidencia de cargas útiles](#payload-matching) | `any` (también si falta) | No |
| [[ui:field.pattern]] | `pattern` | | vacío; obligatorio salvo `any` | Sí |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (también si falta); 1–120 000 | No |
| [[ui:exp.replyVariable]] | `variable` | | `reply` (también si falta) | No |

```json
{ "id": "state", "type": "wait_mqtt", "x": 500, "y": 80, "host": "{{broker}}", "port": 1883,
  "topic": "lab/+/state", "mode": "contains", "pattern": "on", "timeout_ms": 5000, "variable": "reply" }
```

### Esperar una solicitud HTTP {#node-wait_http}

Espera una solicitud HTTP — un webhook, una retrollamada — al
[[[ui:exp.node.emulator]]](#node-emulator) de la ejecución en esa dirección o, cuando
la ejecución no tiene ahí ningún emulador HTTP, a un escucha propio de la ejecución
que responde a cada solicitud con 204. La solicitud debe coincidir con el método, la
ruta de acceso y todas las condiciones.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | `IP:port` | `127.0.0.1:18080` — donde escucha un [[ui:exp.node.emulator]] nuevo | No |
| [[ui:exp.method]] | `method` | Un método, o [[ui:emu.methodAny]] (`ANY`); GET también acepta HEAD | `ANY` (también si falta) | No |
| [[ui:exp.path]] | `path` | `/hooks/:name` nombra un segmento (`{{request.params.name}}`); un `/*` final toma el resto | `/*` (también si falta); como máximo 512 caracteres | Sí |
| [[ui:emu.conditions]] | `when` | `[{ "on", "name", "op", "value" }, …]` sobre un `header`, un parámetro `query`, el `body` o una ruta `json`; debe cumplirse cada uno | ninguna; como máximo 16 | Sí, nombres y valores |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 5 000 (2 000 si falta); 1–120 000 | No |
| [[ui:exp.replyVariable]] | `variable` | | `request` (también si falta) | No |

La solicitud guardada tiene `method`, `path`, `query`, `headers`, `body`, `json`,
`params`, `from` y `ms`: `{{request.json.event}}`, `{{request.headers.x-key}}`.

```json
{ "id": "hook", "type": "wait_http", "x": 500, "y": 80, "bind": "127.0.0.1:18081", "method": "POST",
  "path": "/hooks/:name", "when": [{ "on": "json", "name": "$.event", "op": "eq", "value": "deploy" }],
  "timeout_ms": 5000, "variable": "request" }
```
### Esperar WebSocket {#node-wait_ws}

Espera un mensaje en la conexión que abrió un [[ui:exp.node.ws_connect]], cuya carga
útil coincide. Cuentan los mensajes desde la última acción de la rama — la propia
conexión, un envío o cualquier otra solicitud.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | El id de un nodo [[ui:exp.node.ws_connect]] | el primero | No |
| [[ui:exp.waitMode]] | `mode` | Consulta [Coincidencia de cargas útiles](#payload-matching) | `any` (también si falta) | No |
| [[ui:field.pattern]] | `pattern` | | vacío; obligatorio salvo `any` | Sí |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (también si falta); 1–120 000 | No |
| [[ui:exp.replyVariable]] | `variable` | | `reply` (también si falta) | No |

Un mensaje JSON se puede leer campo por campo: `{{reply.json.type}}`. La conexión debe
preceder a la espera en su ruta.

```json
{ "id": "pong", "type": "wait_ws", "x": 730, "y": 80, "connection": "socket", "mode": "contains",
  "pattern": "pong", "timeout_ms": 3000, "variable": "reply" }
```

## Emulación {#emulation}

### Emulador {#node-emulator}

Hace el papel de una dependencia — una API HTTP, un dispositivo OSC, UDP o TCP, un
bróker MQTT — durante toda la ejecución. Se abre antes del primer paso y responde
hasta que termina la ejecución; en el flujo, el paso se supera de inmediato. Lo que
recibió se cuenta, regla por regla, en el informe de la ejecución.

| Campo | En el archivo | Qué | Predeterminado |
| --- | --- | --- | --- |
| [[ui:emu.edit]] | `emulator` | El emulador: `name`, `bind` (`IP:port`), `protocol` (`http`, `osc`, `udp`, `tcp`, `mqtt`), sus rutas o reglas, y una `outage` opcional | Una API HTTP llamada *API* en `127.0.0.1:18080` que responde a `/health` |

Las propiedades muestran lo que hace en una línea. [[ui:emu.edit]] abre sus reglas, el
mismo editor que la pantalla [[[ui:nav.emulators]]](../tools/emulators.md);
[[ui:emu.toLibrary]] guarda una copia en la biblioteca de emuladores, y
[[ui:emu.fromLibrary]] sustituye este por una copia de ella. Las reglas — rutas,
respuestas, fallos, caídas — se describen allí.

- Un emulador HTTP es también lo que lee una [[[ui:exp.node.wait_http]]](#node-wait_http)
  en su dirección; un emulador OSC o UDP comparte su puerto con las esperas de la
  ejecución allí.
- Dos emuladores de un mismo transporte no pueden compartir un puerto en una ejecución.
- [[[ui:exp.node.emulator_state]]](#node-emulator_state) lo baja y lo vuelve a subir.

Salidas: [[ui:exp.outputPort]]. Sin ajustes.

```json
{ "id": "api", "type": "emulator", "x": 270, "y": 80,
  "emulator": { "name": "Orders API", "bind": "127.0.0.1:18080", "protocol": "http",
    "routes": [{ "method": "GET", "path": "/orders/:id", "order": "sequence",
                 "responses": [{ "status": 503 }, { "status": 200, "body": "{\"id\":\"{{request.params.id}}\"}" }] }] } }
```

## Fallos {#faults}

Nodos que rompen cosas a voluntad. Una rama de nodos [[ui:exp.node.delay]] y estos
junto al tráfico se lee como una programación; [Fallos programados](faults.md) muestra
cómo.

### Degradación {#node-impairment}

Un relé de degradación durante toda la ejecución: el sistema bajo prueba envía a (o
se conecta a) [[ui:exp.relayListen]] en lugar del destino real; el relé reenvía a
[[ui:exp.relayTarget]], y las respuestas vuelven por el mismo camino, degradadas por
el perfil. Se abre antes del primer paso y se cierra cuando termina la ejecución, de
cualquier forma, así que nada queda degradado; en el flujo, el paso se supera de
inmediato. Cada decisión se extrae de la semilla de la ejecución: la misma semilla y
el mismo tráfico corren la misma suerte.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.relayListen]] | `listen` | `IP:port` al que envía el sistema bajo prueba | `127.0.0.1:9010` | Solo parámetros |
| [[ui:exp.relayTarget]] | `target` | `IP:port` del destino real, o `host:port` — un nombre de host se resuelve cuando empieza la ejecución, y un nombre que no se encuentra detiene la ejecución en este nodo | `127.0.0.1:9000` | Solo parámetros |
| [[ui:ns.protocol]] | `protocol` | UDP (`udp`): cada datagrama corre su propia suerte; TCP (`tcp`): cada conexión se une a una propia hacia el destino, y ambos flujos se degradan | UDP (`udp` si falta) | No |
| [[ui:ns.preset]] y los valores bajo él | `profile` | Lo que el relé hace al tráfico — consulta [El perfil](#impair-profile) | [[ui:ns.preset.lan]] (sin degradación si falta) | No |

La dirección de escucha de un relé no puede ser otro socket de la ejecución, y los
relés no pueden reenviarse entre sí en círculo. Un destino dado por nombre se sigue
una vez resuelto, así que un círculo a través de un nombre detiene la ejecución al
empezar. El informe cuenta cada fase de un relé por separado.

Salidas: [[ui:exp.outputPort]]. Sin ajustes.

```json
{ "id": "relay", "type": "impairment", "x": 270, "y": 80, "listen": "127.0.0.1:9010", "target": "{{device}}",
  "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } }
```

#### El perfil {#impair-profile}

Un chip de preajuste — [[ui:ns.preset.lan]], [[ui:ns.preset.wifi]],
[[ui:ns.preset.4g]], [[ui:ns.preset.satellite]],
[[ui:ns.preset.intermittent]], [[ui:ns.preset.offline]] — rellena todos los valores;
cambia cualquiera de ellos después. Un relé solo lee los valores de su protocolo; en
un archivo puede omitirse cualquier clave (cero, desactivado).

| Campo | En el archivo | Qué | Límites | Protocolo |
| --- | --- | --- | --- | --- |
| — | `name` | Una etiqueta para la línea de tiempo y el informe: la clave de un preajuste (`lan`, `wifi`, `4g`, `satellite`, `intermittent`, `offline`) o la tuya | como máximo 60 caracteres | ambos |
| [[ui:ns.offline]] | `offline` | No pasa nada | `true` / `false` | ambos |
| [[ui:ns.latency]] | `latency_ms` | Retardo añadido a cada paquete, o trozo de un flujo | 0–60 000 (el deslizador llega a 1 000) | ambos |
| [[ui:ns.jitter]] | `jitter_ms` | Un retardo extra aleatorio de hasta este tiempo; un flujo TCP se mantiene en orden | 0–60 000 (el deslizador llega a 500) | ambos |
| [[ui:ns.rate]] | `rate_kbps` | Un límite de ancho de banda, 0 para ninguno. UDP: pasado un segundo de cola, los datagramas se descartan como limitados; TCP: se frena al emisor, no se descarta nada | 0, o 8–10 000 000 | ambos |
| [[ui:ns.loss]] | `loss` | La probabilidad de que se descarte un datagrama | 0–1 (el deslizador muestra %) | UDP |
| [[ui:ns.burst]], [[ui:ns.burstLength]] | `burst_start`, `burst_length` | La probabilidad de que empiece una ráfaga de pérdidas, y cuántos datagramas dura de media | 0–1; 1–1 000 cuando las ráfagas están activadas | UDP |
| [[ui:ns.duplicate]] | `duplicate` | La probabilidad de que un datagrama se envíe dos veces | 0–1 | UDP |
| [[ui:ns.corrupt]] | `corrupt` | La probabilidad de que se invierta un bit de un datagrama | 0–1 | UDP |
| [[ui:ns.reorder]] | `reorder` | La probabilidad de que un datagrama se retenga, para que los posteriores lo adelanten | 0–1 | UDP |
| [[ui:ns.reset]] | `reset` | La probabilidad de que un trozo de un flujo reinicie su conexión en su lugar — ambos extremos reciben un reinicio | 0–1 | TCP |
| [[ui:ns.stall]] | `stall` | La probabilidad de que un trozo deje su conexión a medias: no pasa nada más en ningún sentido, y no se avisa a ninguno de los dos extremos | 0–1 | TCP |

Más sobre relés, preajustes y lo que modelan en la página
[[[ui:nav.netsim]]](../tools/impairment.md).

### Cambiar degradación {#node-impairment_change}

Cambia uno de los nodos [[ui:exp.node.impairment]] de la ejecución a otro perfil a
partir de este paso, sin soltar su puerto. La fase hasta ahora se cierra y se cuenta
en el informe.

| Campo | En el archivo | Qué | Predeterminado | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.relay]] | `relay` | El id de un nodo [[ui:exp.node.impairment]] de este experimento | el primero | No |
| [[ui:ns.preset]] y los valores bajo él | `profile` | Aquello con lo que degrada a partir de ahora — consulta [El perfil](#impair-profile); el relé lee los valores de su propio protocolo | [[ui:ns.preset.offline]] (sin degradación si falta) | No |

El paso falla si el relé no está en marcha — porque no logró reenviar, por ejemplo.
Salidas: [[ui:exp.outputPort]]. Sin ajustes.

```json
{ "id": "cut", "type": "impairment_change", "x": 730, "y": 200, "relay": "relay",
  "profile": { "name": "offline", "offline": true } }
```

### Bajar/subir emulador {#node-emulator_state}

Baja uno de los emuladores de la ejecución, o lo vuelve a subir. Mientras está caído,
un emulador HTTP responde como dice [[ui:exp.downFault]]; un dispositivo TCP y un
bróker MQTT cortan sus conexiones y rechazan las nuevas; los dispositivos OSC y UDP
no responden nada. Al subir de nuevo, el emulador sigue su propia programación de
caídas, si la tiene.

| Campo | En el archivo | Qué | Predeterminado |
| --- | --- | --- | --- |
| [[ui:exp.emulatorNode]] | `emulator` | El id de un nodo [[ui:exp.node.emulator]] de este experimento | el primero |
| [[ui:exp.emulatorDownState]] | `down` | [[ui:exp.emulatorGoesDown]] (`true`) o [[ui:exp.emulatorComesUp]] (`false`) | caído (`false` si falta) |
| [[ui:exp.downFault]] | `fault` | Solo HTTP: [[ui:emu.outageFault.unavailable]] (`unavailable`), [[ui:emu.outageFault.reset]] (`reset`: la conexión se cierra sin respuesta) o [[ui:emu.outageFault.timeout]] (`timeout`: la solicitud se retiene hasta que el cliente se rinde, 120 s como máximo) | `unavailable` (también si falta) |

Salidas: [[ui:exp.outputPort]]. Sin ajustes. Nada es una plantilla.

```json
{ "id": "down", "type": "emulator_state", "x": 500, "y": 200, "emulator": "api", "down": true, "fault": "unavailable" }
```
## Datos {#data}

### Extraer un valor {#node-extract}

Guarda una parte de la última respuesta HTTP de su ruta como una variable, para
campos posteriores (`{{token}}`), comprobaciones y ramas. Una solicitud HTTP debe
precederlo en todas las rutas. Hacer clic en un valor de una respuesta de
[[ui:exp.sendNow]] añade uno por ti.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.variable]] | `variable` | El nombre: letras, dígitos y `_`, sin empezar por un dígito, ni una palabra reservada, ni el nombre de un parámetro | `token` | No |
| [[ui:exp.extractFrom]] | `from` | [[ui:exp.from.json]] (`json`), [[ui:exp.from.header]] (`header`), [[ui:exp.from.status]] (`status`), [[ui:exp.from.body]] (`body`) o [[ui:exp.from.regex]] (`regex`) | `json` | No |
| [[ui:exp.jsonPath]], [[ui:exp.headerName]] o [[ui:exp.pattern]] | `expr` | Una ruta JSON (`$.data.token`, `$.items[0]`, `$["first name"]`), el nombre de un encabezado (en cualquier caso), o una expresión regular — su primer grupo, o toda la coincidencia | `$.token`; no se usa para el estado y el cuerpo | No |

El paso falla cuando no hay nada que tomar: el cuerpo no es JSON, falta la ruta o el
encabezado, la expresión no coincide o — para un campo JSON o todo el cuerpo — el
cuerpo era más largo que los 256 KiB conservados. Un estado se guarda como número; el
resto, como texto, o como el valor JSON encontrado. Salidas:
[[ui:exp.outputPort]]. Sin ajustes.

```json
{ "id": "token", "type": "extract", "x": 500, "y": 80, "variable": "token", "from": "json", "expr": "$.data.token" }
```

Más sobre variables en [Datos y plantillas](data.md).

## Comprobaciones {#checks}

Una comprobación se supera, o falla la ejecución. Las cuatro comprobaciones de
respuesta leen la última respuesta HTTP de su ruta, así que una solicitud HTTP — no
una bajo carga — debe precederlas en todas las rutas.

### Estado HTTP {#node-assert_status}

Se supera cuando el estado de la última respuesta es exactamente el indicado.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.expectedStatus]] | `status` | | 200; 100–599 | No |

Salidas: [[ui:exp.outputPort]]. Sin ajustes.

```json
{ "id": "ok", "type": "assert_status", "x": 500, "y": 80, "status": 200 }
```

### Texto de la respuesta {#node-assert_body}

Se supera cuando el cuerpo de la última respuesta contiene el texto, exactamente
(incluidas las mayúsculas y minúsculas). Solo se conservan los primeros 256 KiB de un
cuerpo: un texto que no se encuentra en un cuerpo que se cortó falla con ese motivo.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.contains]] | `contains` | | `ok`; obligatorio | Sí |

Salidas: [[ui:exp.outputPort]]. Sin ajustes.

```json
{ "id": "ready", "type": "assert_body", "x": 500, "y": 80, "contains": "ready" }
```

### Encabezado de la respuesta {#node-assert_header}

Se supera cuando la última respuesta tiene el encabezado y su valor contiene el texto.
El nombre del encabezado coincide en cualquier caso; el valor, exactamente.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.headerName]] | `name` | | `content-type`; obligatorio | Sí |
| [[ui:exp.contains]] | `contains` | Lo que debe contener su valor; vacío: el encabezado solo tiene que estar | `application/json` | Sí |

Salidas: [[ui:exp.outputPort]]. Sin ajustes.

```json
{ "id": "json", "type": "assert_header", "x": 500, "y": 80, "name": "Content-Type", "contains": "json" }
```

### Tiempo de respuesta {#node-assert_latency}

Se supera cuando la última respuesta tardó como máximo este tiempo, desde el envío de
la solicitud hasta el final de su cuerpo.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.maxLatency]] | `max_ms` | | 1 000; 1–120 000 | No |

Salidas: [[ui:exp.outputPort]]. Sin ajustes.

```json
{ "id": "fast", "type": "assert_latency", "x": 500, "y": 80, "max_ms": 250 }
```

### Comprobar un valor {#node-assert_value}

Compara un valor — normalmente una variable, escrita como plantilla — con uno
esperado, y se supera cuando se cumple la comparación.

| Campo | En el archivo | Qué | Predeterminado | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.value]] | `value` | Lo que se compara: `{{token}}`, `{{reply.args[0]}}` | `{{token}}` | Sí |
| [[ui:exp.operator]] | `op` | Consulta [Comparaciones](#comparisons) | [[ui:exp.op.not_empty]] | No |
| [[ui:exp.expected]] | `expected` | No lo usan [[ui:exp.op.empty]] ni [[ui:exp.op.not_empty]] | vacío (también si falta) | Sí |

Salidas: [[ui:exp.outputPort]]. Sin ajustes.

```json
{ "id": "state", "type": "assert_value", "x": 730, "y": 80, "value": "{{state}}", "op": "eq", "expected": "ready" }
```

### Comparaciones {#comparisons}

[[ui:exp.node.assert_value]], [[ui:exp.node.branch_value]], la condición de salida de
un [[ui:exp.node.loop]], las reglas de argumentos de OSC y las condiciones HTTP
comparan del mismo modo:

| Opción | En el archivo | Se cumple cuando el valor |
| --- | --- | --- |
| [[ui:exp.op.eq]] | `eq` | es igual al esperado — como números cuando ambos son números (`200` = `200.0`), si no como texto exacto |
| [[ui:exp.op.ne]] | `ne` | no es igual a él, por la misma regla |
| [[ui:exp.op.lt]], [[ui:exp.op.le]], [[ui:exp.op.gt]], [[ui:exp.op.ge]] | `lt`, `le`, `gt`, `ge` | es menor, como máximo, mayor, como mínimo — ambos deben ser números: si no, una comprobación, una rama o un [[ui:exp.node.loop]] falla el paso, y una regla de argumentos o una condición HTTP no se cumple |
| [[ui:exp.op.contains]] | `contains` | contiene el texto esperado |
| [[ui:exp.op.matches]] | `matches` | coincide con la expresión regular esperada |
| [[ui:exp.op.empty]], [[ui:exp.op.not_empty]] | `empty`, `not_empty` | está vacío (los espacios cuentan como vacío) / no lo está |
## Flujo {#flow}

Nodos que deciden adónde va la ejecución. Más sobre ramas, uniones y bucles en
[Flujo](flow.md).

### Inicio {#node-start}

Donde empieza la ejecución; cada experimento tiene exactamente uno. No tiene entrada
ni campos. La primera fila de la línea de tiempo da la semilla de la ejecución.

Salidas: [[ui:exp.outputPort]], obligatoria. Varios cables desde él inician ramas
paralelas a la vez.

```json
{ "id": "start", "type": "start", "x": 40, "y": 80 }
```

### Fin {#node-end}

Donde se completa la ejecución; cada experimento tiene exactamente uno, y no tiene
salidas. Varias ramas pueden conducir a él: la ejecución se supera una vez, después
de que termine la última rama, y solo si ninguna falló. Una ejecución que nunca llega
a [[ui:exp.node.end]] falla.

```json
{ "id": "end", "type": "end", "x": 960, "y": 80 }
```

### Retardo {#node-delay}

Espera un tiempo fijo antes del siguiente paso.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.delayMs]] | `ms` | | 300; 0–60 000 | No |

Salidas: [[ui:exp.outputPort]]. Sin ajustes. Para esperas más largas, pon varios en
fila o en un [[ui:exp.node.loop]].

```json
{ "id": "pause", "type": "delay", "x": 500, "y": 80, "ms": 500 }
```

### Rama por estado {#node-branch_status}

Elige [[ui:exp.yes]] cuando la última respuesta HTTP tiene este estado, si no
[[ui:exp.no]]. Una solicitud HTTP debe precederla en todas las rutas.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.expectedStatus]] | `status` | | 200; 100–599 | No |

Salidas: [[ui:exp.yes]] y [[ui:exp.no]], ambas obligatorias. Sin ajustes.

```json
{ "id": "branch", "type": "branch_status", "x": 500, "y": 80, "status": 200 }
```

### Rama por valor {#node-branch_value}

Elige [[ui:exp.yes]] cuando se cumple una comparación, si no [[ui:exp.no]]. Sus campos
y sus [comparaciones](#comparisons) son los de
[[[ui:exp.node.assert_value]]](#node-assert_value); una comparación que no se puede
hacer (`lt` sobre texto) falla el paso.

| Campo | En el archivo | Qué | Predeterminado | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.value]] | `value` | Lo que se compara | `{{token}}` | Sí |
| [[ui:exp.operator]] | `op` | | [[ui:exp.op.eq]] | No |
| [[ui:exp.expected]] | `expected` | | vacío (también si falta) | Sí |

Salidas: [[ui:exp.yes]] y [[ui:exp.no]], ambas obligatorias. Sin ajustes.

```json
{ "id": "ok", "type": "branch_value", "x": 730, "y": 80, "value": "{{reply.args[0]}}", "op": "eq", "expected": "ok" }
```

### Rama paralela {#node-fork}

Ejecuta lo que sigue a [[ui:exp.branch1]] y [[ui:exp.branch2]] al mismo tiempo, cada
rama con su propia copia de las variables. Cada salida puede tener más cables para más
ramas. Sin campos.

Salidas: [[ui:exp.branch1]] y [[ui:exp.branch2]], ambas obligatorias.

```json
{ "id": "split", "type": "fork", "x": 270, "y": 80 }
```

### Unir ramas {#node-join}

Espera hasta que se haya alcanzado cada cable que entra en él, luego continúa una vez,
con las variables de las ramas fusionadas — cuando dos ramas establecen la misma
variable, gana la cuyo cable aparece más tarde en el archivo — y la última respuesta
HTTP de la última de ellas que tuvo una. Sin campos.

Solo se encuentran aquí las ramas que se ejecutan todas: un [[ui:exp.node.join]] detrás
de un [[ui:exp.node.branch_status]], cuyos [[ui:exp.yes]] y [[ui:exp.no]] nunca ocurren
ambos, nunca continúa; cuando ninguna otra ruta llega a [[ui:exp.node.end]], la
ejecución falla en este nodo, indicando cuántas ramas seguía esperando.

Salidas: [[ui:exp.outputPort]], obligatoria.

Cualquier otro nodo con varios cables que entran en él se ejecuta una vez por cada
llegada.

```json
{ "id": "joined", "type": "join", "x": 730, "y": 80 }
```

### Bucle {#node-loop}

Ejecuta los pasos de [[ui:exp.portBody]] — que vuelven a él — una y otra vez: como
máximo un número de veces y, cuando tiene una condición de salida, hasta que se
cumple.

| Campo | En el archivo | Qué | Predeterminado y límites | Plantillas |
| --- | --- | --- | --- | --- |
| [[ui:exp.loopMax]] | `max` | Iteraciones como máximo | 5; 1–1 000 | No |
| [[ui:exp.loopUntilOn]] | `until` | Condición de salida opcional `{ "value", "op", "expected" }`, como en [[[ui:exp.node.assert_value]]](#node-assert_value) | desactivada | Valor y esperado: sí |

- El cuerpo siempre se ejecuta al menos una vez. La condición de salida se lee después
  de cada iteración, así que el cuerpo puede establecer lo que prueba.
- [[ui:exp.portDone]] sigue cuando se cumple la condición — o, sin condición, tras la
  última iteración.
- [[ui:exp.portLimit]] sigue cuando se agotaron las iteraciones antes de que se
  cumpliera la condición. Sin un cable en ella, eso falla el paso.
- Dentro del cuerpo, `{{counter}}` es el número de la iteración.
- Un cuerpo se ejecuta como una sola rama: cada salida dentro de él tiene un cable; no
  contiene [[ui:exp.node.start]], [[ui:exp.node.end]], [[ui:exp.node.fork]],
  [[ui:exp.node.join]] ni otro [[ui:exp.node.loop]]; se entra en él solo por
  [[ui:exp.portBody]]; y cada cable dentro de él conduce hacia adelante en el cuerpo o
  de vuelta al bucle.

Salidas: [[ui:exp.portBody]] y [[ui:exp.portDone]] (obligatorias),
[[ui:exp.portLimit]] (opcional). Sin ajustes.

```json
{ "id": "poll", "type": "loop", "x": 270, "y": 80, "max": 10,
  "until": { "value": "{{status.args[0]}}", "op": "eq", "expected": "ready" } }
```

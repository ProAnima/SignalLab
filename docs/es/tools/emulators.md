---
title: Emuladores
description: Haz que Signal Lab sea la otra parte — una API HTTP, un dispositivo OSC, UDP o TCP, o un bróker MQTT — que responde según tus reglas, falla cuando se lo pides y cuenta lo que llega.
---

# Emuladores

Un emulador es Signal Lab haciendo el papel de la API, el dispositivo o el servicio con el que
habla tu sistema. Escucha en una dirección y responde según reglas: una API HTTP, por rutas; un
dispositivo OSC, UDP o TCP, con “ante esto, responde aquello”; un bróker MQTT, como cualquier
bróker y además con reglas propias. Puede ser lento, fallar o caer de vez en cuando, para que
pruebes qué hace tu sistema cuando su dependencia se porta mal. Cada intercambio se cuenta, se
muestra en una lista y se envía al [Inspector](inspector.md).

Un emulador es un solo documento. La pantalla [[ui:nav.emulators]] guarda una biblioteca de
ellos; el mismo documento funciona dentro de un experimento como nodo
[[ui:exp.node.emulator]], desde la línea de comandos con `signallab emulate`, y a través de la
[API](../api/commands.md) y de [MCP](../automation/mcp.md), y responde igual en todas partes.

## La pantalla {#screen}

A la izquierda está la biblioteca ([[ui:emu.library]]): cada emulador con su protocolo y su
dirección, y un punto que late y un recuento de solicitudes en los que están en marcha. A la
derecha están los ajustes y las reglas del emulador seleccionado y, debajo, lo que ha recibido
([[ui:emu.live]]).

## Crear un emulador {#create}

1. Pulsa uno de los botones de la parte superior de la biblioteca:

   | Botón | Crea | Escucha en | Con una regla que funciona tal cual |
   | --- | --- | --- | --- |
   | ＋ [[ui:emu.new.http]] | Una API HTTP | `127.0.0.1:18080` | `GET /health` → 200 `{"status":"ok"}` |
   | ＋ [[ui:emu.new.osc]] | Un dispositivo OSC | `127.0.0.1:9100` | `/ping` → `/pong` con el recuento como int |
   | ＋ [[ui:emu.new.udp]] | Un dispositivo UDP | `127.0.0.1:7100` | un datagrama que contiene `PING` → `PONG 1`, `PONG 2`, … |
   | ＋ [[ui:emu.new.tcp]] | Un dispositivo TCP | `127.0.0.1:7200` | una línea que contiene `PING` → `PONG` |
   | ＋ [[ui:emu.new.mqtt]] | Un bróker MQTT | `127.0.0.1:1883` | una publicación en `lab/<name>/set` → la misma carga útil, retenida, en `lab/<name>/state` |

   Si otro emulador de la biblioteca ya usa ese puerto, se toma el siguiente libre.
2. Dale un [[ui:emu.name]] (como máximo 120 caracteres).
3. Rellena el campo [[ui:emu.bind]] con `IP:port`. `127.0.0.1` solo responde a este equipo; `0.0.0.0`
   responde también a la red.
4. Cambia las reglas (más abajo) e indica en el campo [[ui:emu.note]] a qué sustituye.

Los cambios se guardan solos. [[ui:emu.duplicate]] crea una copia en el siguiente puerto libre.
[[ui:emu.delete]] vuelve a preguntar ([[ui:emu.confirmDelete]]), detiene el emulador si está en
marcha y lo quita de la biblioteca.

Las reglas se prueban en orden, de la primera a la última; responde la primera que coincide. La
cabecera de cada regla muestra un resumen de una línea; haz clic en ella para abrir o plegar la
regla. Los botones ↑ y ↓ mueven una regla, y × la quita.

## Ponerlo en marcha {#run}

1. Selecciona el emulador y pulsa [[ui:emu.start]]. Su puerto se abre antes de que el botón vuelva
   a estar disponible: un puerto ya ocupado, o un emulador con un problema, se rechaza ahí mismo
   con el motivo.
2. Apunta tu sistema hacia él. En una API HTTP, [[ui:emu.copyUrl]] copia su dirección
   (`http://127.0.0.1:18080`), y cada ruta tiene un botón [[ui:emu.copyRouteUrl]] para la suya
   (salvo cuando su ruta de acceso contiene una plantilla `{{…}}`).
3. Observa cómo se llena la lista [[ui:emu.received]].
4. Pulsa [[ui:emu.stop]], o detén su tarea desde la franja de la consola.

El estado junto a los botones indica [[ui:emu.notRunning]], dónde responde o que está caído.

Un emulador sigue respondiendo con las reglas con las que se inició. Si lo cambias mientras
funciona, aparece [[ui:emu.restart]]: púlsalo para volver a iniciarlo con las reglas tal como
están ahora. Hasta entonces, los recuentos de coincidencias de las reglas se ocultan, porque
pertenecen a las reglas antiguas.

El botón [[ui:emu.takeDown]] hace que un emulador en marcha no esté disponible hasta que pulsas
[[ui:emu.bringUp]]: una solicitud HTTP recibe 503, un dispositivo TCP y un bróker MQTT cortan sus
conexiones y rechazan las nuevas, y un dispositivo OSC o UDP no responde nada. Consulta
[Caídas](#outage).

Dos emuladores del mismo transporte no pueden compartir un puerto: los emuladores HTTP, TCP y MQTT
escuchan en puertos TCP, y los OSC y UDP en puertos UDP. Una API HTTP y un dispositivo OSC pueden
usar ambos el puerto 8080; dos API HTTP, no. Un segundo emulador en un puerto ocupado se rechaza
al iniciarse.

::: tip
En un navegador conectado a un [servidor](../server/index.md), el emulador funciona en el
servidor. A uno que escucha en `0.0.0.0` se llega por el nombre del servidor, y
[[ui:emu.copyUrl]] copia esa dirección; uno en `127.0.0.1` solo responde a programas del propio
servidor.
:::

## Lo que llegó {#received}

Mientras funciona, el panel [[ui:emu.live]] cuenta:

| Recuento | Qué |
| --- | --- |
| [[ui:emu.total]] | Todo lo que llegó: solicitudes, mensajes, líneas. |
| [[ui:emu.unmatched]] | Lo que ninguna regla aceptó. Una solicitud HTTP sin ruta recibe igualmente su respuesta (consulta [Solicitudes que ninguna ruta acepta](#fallback)); lo demás no recibe ninguna. |
| [[ui:emu.failed]] | Intercambios en los que no se pudo crear o enviar una respuesta. |
| [[ui:emu.down]] | Lo que llegó mientras el emulador estaba caído. Se muestra cuando tiene una caída programada o algo lo encontró caído. Nunca se cuenta en [[ui:emu.unmatched]]. |
| [[ui:emu.missed]] | Solo MQTT, cuando ocurre: mensajes que un cliente no pudo recibir porque iba demasiado atrasado. |

La cabecera de cada regla muestra cuántas veces coincidió desde el inicio.

La lista [[ui:emu.received]] muestra los 300 intercambios más recientes, el más reciente primero:

| Columna | Qué |
| --- | --- |
| [[ui:emu.col.time]] | Cuándo llegó. |
| [[ui:emu.col.from]] | La dirección del cliente. |
| [[ui:emu.col.request]] | Lo que llegó, en notación del protocolo: `GET /users/7`, `/ping 1`, `POWER?`. |
| [[ui:emu.col.rule]] | La regla que lo aceptó (`#2`), o `—`. |
| [[ui:emu.col.reply]] | Lo que se devolvió: `200 OK · 37 B`, `/pong 3`, una carga útil; [[ui:emu.held]] o [[ui:emu.closed]] en caso de fallo; el error, cuando la respuesta falló; [[ui:emu.wasDown]], cuando llegó mientras estaba caído. |
| [[ui:emu.col.ms]] | Desde la llegada hasta que salió la respuesta, retardo incluido. |

El botón ⌕ de una fila ([[ui:emu.inspectFrame]]) abre ese intercambio en el Inspector, si la
captura estaba activada. Cuando llegan más de 200 intercambios en una quinta parte de segundo, la
lista omite algunos e indica cuántos. El motor conserva los 500 intercambios más recientes de cada
emulador en marcha, con lo que llegó, para la línea de comandos, la API y MCP.

## API HTTP {#http}

Un servidor HTTP/1.1. Cada solicitud la responde la primera ruta que la acepta.

### Rutas {#routes}

Una ruta acepta una solicitud cuando coinciden su método, su ruta de acceso y todas sus
condiciones.

| Campo | Qué |
| --- | --- |
| [[ui:emu.method]] | `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS` o [[ui:emu.methodAny]]. Una ruta `GET` responde también a `HEAD`. |
| [[ui:emu.path]] | Empieza por `/`. Un segmento `:name` acepta un segmento cualquiera, que se lee como `{{request.params.name}}`; un último segmento `*` acepta todo lo que hay debajo. Una `/` final no cambia nada; la cadena de consulta no forma parte de la ruta de acceso. |
| [[ui:emu.conditions]] | Deben cumplirse todas. Añade una con ＋ [[ui:emu.addCondition]]. |

Ejemplos de ruta de acceso:

| Ruta de acceso | Acepta | No acepta |
| --- | --- | --- |
| `/health` | `/health`, `/health/` | `/health/db`, `/Health` |
| `/users/:id` | `/users/7` (`params.id` es `7`), `/users/a%20b` (`a b`) | `/users`, `/users/7/orders` |
| `/files/*` | `/files`, `/files/a`, `/files/a/b/c` | `/file`, `/other/files/a` |

Una condición lee una parte de la solicitud ([[ui:emu.on]]) y la compara:

| [[ui:emu.on]] | Nombre | Lee |
| --- | --- | --- |
| [[ui:emu.on.header]] | El nombre de un encabezado, en mayúsculas o minúsculas | El valor del encabezado; si el encabezado se envía varias veces, sus valores unidos con `, `. |
| [[ui:emu.on.query]] | Un parámetro de consulta | Su valor, decodificado; el primero, si se repite. |
| [[ui:emu.on.body]] | — | Todo el cuerpo como texto. |
| [[ui:emu.on.json]] | Una ruta JSON, como `$.user.id` | Ese campo de un cuerpo JSON. |

Las comparaciones son [[ui:exp.op.eq]], [[ui:exp.op.ne]], [[ui:exp.op.lt]],
[[ui:exp.op.le]], [[ui:exp.op.gt]], [[ui:exp.op.ge]], [[ui:exp.op.contains]],
[[ui:exp.op.matches]], [[ui:exp.op.empty]] y [[ui:exp.op.not_empty]]. Los números se comparan como
números; el texto, exactamente. Un encabezado, parámetro o campo que no existe está vacío. Una
comparación que no se puede hacer (texto frente a un número) no se cumple.

### Respuestas {#responses}

Una ruta tiene de una a 16 respuestas ([[ui:emu.responses]]).

| Campo | Qué | Predeterminado |
| --- | --- | --- |
| [[ui:emu.status]] | 100–599. | 200 |
| [[ui:emu.fault]] | Algo distinto de una respuesta; consulta [Fallos](#faults). | [[ui:emu.fault.none]] |
| [[ui:emu.delay]] | Cuánto esperar antes de responder, 0–60 000 ms. | 0 |
| [[ui:emu.jitter]] | Hasta este tiempo más, al azar, 0–60 000 ms. | 0 |
| [[ui:emu.weight]] | Su proporción cuando la ruta responde al azar. Solo se muestra entonces. | 1 |
| [[ui:emu.headers]] | Hasta 32. Los nombres pueden usar parámetros; los valores son [plantillas](#templates). | ninguno |
| [[ui:emu.body]] | Una [plantilla](#templates), de hasta 256 KiB tal como se escribe. | vacío |

Sin un encabezado `Content-Type`, un cuerpo que es JSON válido se envía como `application/json`, y
cualquier otro como `text/plain; charset=utf-8`.

Con dos respuestas o más, el campo [[ui:emu.order]] indica cuál recibe cada solicitud:

| [[ui:emu.order]] | Las solicitudes reciben | Para |
| --- | --- | --- |
| [[ui:emu.order.sequence]] | La primera, la segunda, …, y a partir de ahí la última: 500, 500, 200, 200, 200… | Reintentos: fallar dos veces y luego funcionar. |
| [[ui:emu.order.cycle]] | Otra vez la primera después de la última: 200, 500, 200, 500… | Una dependencia que falla de vez en cuando, con regularidad. |
| [[ui:emu.order.random]] | Cada una, sorteada según su peso. Con pesos 8 y 2, la primera sale aproximadamente el 80 % de las veces. Al menos un peso debe ser mayor que 0. | Una proporción realista de fallos. |

El menú [[ui:emu.preset]] añade a la ruta una respuesta ya preparada:

| Preajuste | Añade |
| --- | --- |
| [[ui:emu.preset.ok]] | 200, `{"ok":true}` |
| [[ui:emu.preset.created]] | 201, `{"id":"{{uuid}}"}`, encabezado `Location: {{request.path}}/{{counter}}` |
| [[ui:emu.preset.notFound]] | 404, `{"error":"not found"}` |
| [[ui:emu.preset.error]] | 500, `{"error":"internal"}` |
| [[ui:emu.preset.unavailable]] | 503, `{"error":"unavailable"}`, encabezado `Retry-After: 1` |
| [[ui:emu.preset.slow]] | 200, `{"ok":true}` al cabo de 2000 ms |
| [[ui:emu.preset.timeout]] | El fallo [[ui:emu.fault.timeout]] |
| [[ui:emu.preset.reset]] | El fallo [[ui:emu.fault.reset]] |
| [[ui:emu.preset.malformed]] | 200, `{"items":[{"id":1},{"id":2}]}` con el fallo [[ui:emu.fault.malformed]] |

### Fallos {#faults}

| [[ui:emu.fault]] | Lo que encuentra el cliente |
| --- | --- |
| [[ui:emu.fault.none]] | La respuesta. |
| [[ui:emu.fault.timeout]] | Nada. La solicitud se retiene hasta 2 minutos y luego se cierra la conexión, así que lo que se prueba es el propio tiempo de espera del cliente. El retardo no se aplica. |
| [[ui:emu.fault.reset]] | La conexión se cierra sin respuesta, tras el retardo. |
| [[ui:emu.fault.malformed]] | Una respuesta HTTP completa, con el estado y los encabezados definidos, cuyo cuerpo se corta a la mitad: un JSON que no se puede analizar. Cuando todo el cuerpo era JSON, el tipo de contenido sigue indicando `application/json`. |

### Solicitudes que ninguna ruta acepta {#fallback}

La opción [[ui:emu.fallback]] decide qué recibe una solicitud que no coincide con ninguna ruta:

- [[ui:emu.fallbackDefault]]: 404 con el cuerpo `{"error":"no_route"}`;
- [[ui:emu.fallbackCustom]]: una respuesta que defines tú, con todo lo que tiene la respuesta de
  una ruta. Su `{{counter}}` cuenta las solicitudes que ninguna ruta aceptó.

En ambos casos, la solicitud se cuenta en [[ui:emu.unmatched]].

### Lo que puede leer una respuesta HTTP {#http-request}

| Plantilla | Es |
| --- | --- |
| `{{request.method}}` | `GET`, `POST`, … |
| `{{request.path}}` | La ruta de acceso, sin la consulta. |
| `{{request.params.id}}` | El segmento de la ruta de acceso llamado `:id`. |
| `{{request.query.page}}` | Un parámetro de consulta, decodificado. |
| `{{request.headers.x-key}}` | Un encabezado; los nombres, en minúsculas. |
| `{{request.body}}` | El cuerpo como texto: sus primeros 64 KiB. |
| `{{request.json.name}}` | Un campo de un cuerpo JSON, cuando el cuerpo es JSON y no pasa de 64 KiB. |
| `{{request.from}}` | El `IP:port` del cliente. |

Una solicitud con un cuerpo de más de 1 MiB recibe 413 y se cuenta en [[ui:emu.failed]]. Una
respuesta que no se puede crear (una plantilla que nombra algo que la solicitud no tiene) recibe
500 con el error en el cuerpo, y se cuenta en [[ui:emu.failed]].

## Dispositivo OSC {#osc}

Cada mensaje que llega (cada mensaje de un bundle, por separado) lo responde la primera regla con
la que coincide. Un datagrama que no es OSC se cuenta en [[ui:emu.unmatched]].

| Campo | Qué |
| --- | --- |
| [[ui:emu.address]] | Un patrón de dirección OSC 1.0: `*` cualquier carácter, `?` uno, `[a-z]` un conjunto, `{a,b}` cualquiera de los dos, cada uno dentro de un segmento (consulta [OSC](../protocols/osc.md#patterns)). |
| [[ui:exp.argRules]] | Hasta 16 condiciones sobre los argumentos, como en [[ui:exp.node.wait_osc]] (consulta [Nodos](../experiments/nodes.md#node-wait_osc)). |
| [[ui:emu.replyOn]] | Desactivado: aceptar el mensaje y no responder nada. |
| [[ui:emu.replyAddress]] | La dirección de la respuesta, una [plantilla](#templates). |
| [[ui:emu.replyArgs]] | Hasta 16 argumentos, cada uno con un [[ui:emu.argType]] (`int`, `float`, `str`, `long`, `double`, `bool`, `blob`, `nil`) y una plantilla de [[ui:emu.argValue]]. |
| [[ui:emu.to]] | Vacío: de vuelta a la dirección y el puerto del emisor. Si no, `IP:port`. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 0–60 000 ms cada uno. |

El valor de un argumento se lee según su tipo una vez rellenada la plantilla:
`{{request.args[0]}}` repite el primer argumento como número cuando el tipo es numérico. Un `bool`
acepta `true`, `1`, `yes`, `on` o `false`, `0`, `no`, `off`; un `blob`, bytes en hex; un valor
vacío es el cero del tipo.

Las respuestas salen del propio puerto del emulador, así que un cliente que escucha en el puerto
desde el que envió las oye.

Una respuesta OSC puede leer `{{request.address}}`, `{{request.args[0]}}` y `{{request.from}}`.

## Dispositivo UDP {#udp}

Cada datagrama lo responde la primera regla con la que coincide.

| Campo | Qué |
| --- | --- |
| [[ui:emu.match]] | [[ui:exp.mode.any]], [[ui:exp.mode.contains]], [[ui:exp.mode.regex]] o [[ui:exp.mode.hex]]. |
| [[ui:emu.pattern]] | El texto, la expresión regular o los bytes que buscar. |
| [[ui:emu.reply]] | [[ui:emu.replyOff]], [[ui:emu.replyText]] o [[ui:emu.replyHex]], y luego la respuesta en sí como [plantilla](#templates). |
| [[ui:emu.to]] | Vacío: de vuelta al emisor. Si no, `IP:port`. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 0–60 000 ms cada uno. |

Una respuesta UDP o TCP puede leer:

| Plantilla | Es |
| --- | --- |
| `{{request.text}}` | La carga útil como texto. |
| `{{request.match}}` | Lo que coincidió: el texto, el primer grupo de una expresión regular (o toda la coincidencia), los bytes. |
| `{{request.hex}}` | La carga útil como bytes en hex, los primeros 1024. |
| `{{request.bytes}}` | El tamaño de la carga útil. |
| `{{request.from}}` | El `IP:port` del emisor. |

Una respuesta de texto tiene como máximo 65 507 bytes.

## Dispositivo TCP {#tcp}

Un dispositivo que habla por líneas en una conexión TCP, como un proyector o un conmutador
matricial. Cada mensaje que envía un cliente lo responde la primera regla con la que coincide; la
respuesta vuelve por la misma conexión.

| Campo | Qué |
| --- | --- |
| [[ui:emu.delimiter]] | Lo que termina un mensaje, y se añade tras cada respuesta y tras el saludo: [[ui:emu.delimiter.lf]] (se descarta un `\r` delante), [[ui:emu.delimiter.crlf]], [[ui:emu.delimiter.cr]] o [[ui:emu.delimiter.none]]. Las líneas vacías se omiten. |
| [[ui:emu.greeting]] | Se envía cuando se conecta un cliente; vacío para ninguno. Puede leer `{{request.from}}`. |
| [[ui:emu.match]], [[ui:emu.pattern]], [[ui:emu.reply]] | Como en un [dispositivo UDP](#udp). |
| [[ui:emu.close]] | Cerrar la conexión tras la respuesta de esta regla; por ejemplo, ante `QUIT`. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 0–60 000 ms cada uno. |

Un mensaje de más de 64 KiB sin su delimitador se toma tal cual.

## Bróker MQTT {#mqtt}

Un pequeño bróker MQTT 3.1.1 sobre TCP sin cifrar. Hace lo que hace un bróker: los clientes se
conectan, se suscriben con `+` y `#`, publican con QoS 0, 1 y 2, los mensajes retenidos y los de
última voluntad funcionan, y una segunda conexión con el id de un cliente sustituye a la primera.
Las sesiones son siempre limpias: un cliente que pide conservar su sesión recibe una nueva, y no
se guarda nada en cola para un cliente ausente.

Además, cada mensaje que se le publica se comprueba con las reglas: la primera que coincide
publica también una respuesta, como un dispositivo que informa de lo que ha hecho.

| Campo | Qué |
| --- | --- |
| [[ui:emu.username]], [[ui:emu.password]] | Si se define un nombre de usuario, un cliente debe conectarse con él y con la contraseña; vacío: cualquiera puede conectarse. Una contraseña sin nombre de usuario se rechaza, porque MQTT 3.1.1 no puede transportarla. |
| [[ui:emu.retained]] | Hasta 64 mensajes ([[ui:emu.topic]], [[ui:emu.payload]], [[ui:emu.qos]]) guardados desde el inicio, como si se hubieran publicado con retain: un cliente que se suscribe los recibe primero. |
| [[ui:emu.topicFilter]] | Qué temas acepta una regla: `+` un nivel, `#` el resto; por ejemplo, `lab/+/set`. |
| [[ui:emu.match]], [[ui:emu.pattern]] | Una condición sobre la carga útil, como en un [dispositivo UDP](#udp). |
| [[ui:emu.replyOn]] | Desactivado: aceptar el mensaje y no publicar nada más. |
| [[ui:emu.replyTopic]], [[ui:emu.replyPayload]] | [Plantillas](#templates). El tema no puede contener `+` ni `#`. |
| [[ui:emu.qos]], [[ui:emu.retain]] | Los de la respuesta. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 0–60 000 ms cada uno. |

Una respuesta MQTT puede leer `{{request.topic}}`, `{{request.levels[1]}}` (los niveles del tema,
desde 0), `{{request.payload}}`, `{{request.json.state}}`,
`{{request.match}}`, `{{request.qos}}`, `{{request.retain}}`,
`{{request.client}}` (el id del cliente) y `{{request.from}}`.

## Plantillas en las respuestas {#templates}

Las respuestas se escriben en el mismo [lenguaje de plantillas](../experiments/data.md#templates)
que los experimentos, así que un campo significa lo mismo aquí y allí. Una respuesta puede leer:

- `request`: lo que llegó, según se indica para cada protocolo más arriba;
- `{{counter}}`: cuántos mensajes ha aceptado esta regla desde que se inició el emulador, este
  incluido;
- los [generadores](../experiments/data.md#generators): `{{uuid}}`, `{{now.iso}}`, valores
  aleatorios y los demás; los aleatorios se sortean a partir de la semilla del emulador;
- parámetros, cuando el emulador funciona en un experimento o se inicia con
  `signallab emulate --param`.

Una respuesta nunca lee secretos, y un nombre desconocido es un error, no un texto vacío.

Algunos campos quedan fijados al iniciarse el emulador, antes de que llegue nada: una ruta de
acceso, una condición, un patrón de dirección, un patrón de carga útil, un filtro de temas,
[[ui:emu.to]], el nombre de un encabezado, los mensajes retenidos y el inicio de sesión del
bróker. Solo aceptan texto y parámetros, ni `request` ni generadores.

La semilla rige el orden aleatorio de las respuestas, el jitter y los generadores aleatorios. En
la pantalla [[ui:nav.emulators]], cada inicio toma una semilla nueva; un experimento usa la
semilla de la ejecución, y `signallab emulate --seed` toma la que le des.

## Caídas {#outage}

Para probar qué hace tu sistema cuando una dependencia va y viene, marca
[[ui:emu.outage]]:

| Campo | Qué | Predeterminado |
| --- | --- | --- |
| [[ui:emu.outageUp]] | Cuánto tiempo responde, 10–3 600 000 ms. | 10 000 |
| [[ui:emu.outageDown]] | Cuánto tiempo está caído, 10–3 600 000 ms. | 3000 |
| [[ui:emu.outageFault]] | Solo HTTP: lo que encuentra una solicitud mientras está caído. | [[ui:emu.outageFault.unavailable]] |

El programa empieza cuando se inicia el emulador y se repite: activo, caído, activo, caído…
Mientras está caído:

| Emulador | Encuentra |
| --- | --- |
| HTTP | [[ui:emu.outageFault.unavailable]]: 503 con `Retry-After` igual a los segundos que faltan para que vuelva (al menos 1). [[ui:emu.outageFault.reset]]: la conexión se cierra sin respuesta. [[ui:emu.outageFault.timeout]]: se retiene hasta 2 minutos y luego se cierra. |
| Dispositivo TCP | Las conexiones abiertas se cortan en menos de 0,1 s; las nuevas se cierran según llegan. |
| Bróker MQTT | Se cortan todas las conexiones; las nuevas se rechazan (CONNACK con código de retorno 3, servidor no disponible). |
| Dispositivo OSC, UDP | No se responde nada. |

Lo que llega mientras está caído se cuenta en [[ui:emu.down]], no en [[ui:emu.unmatched]], y no se
consulta a sus reglas.

El botón [[ui:emu.takeDown]] hace lo mismo a petición, diga lo que diga el programa, hasta que pulsas
[[ui:emu.bringUp]]; HTTP recibe entonces 503 sin `Retry-After`. En un experimento, el nodo
[[ui:exp.node.emulator_state]] lo hace en un paso de la ejecución (consulta
[Nodos](../experiments/nodes.md#node-emulator_state) y [Fallos](../experiments/faults.md)).

## Problemas {#problems}

Mientras editas, el emulador se comprueba un momento después de cada cambio, y un problema
aparece bajo sus botones antes de que pulses [[ui:emu.start]]. Un problema indica dónde está (la
regla, la respuesta o el mensaje retenido, y el campo) y qué está mal: una ruta de acceso sin su
`/`, una expresión regular que no compila, una plantilla de respuesta que nombra algo que no es
`request`, parámetros o generadores, un valor fuera de rango. [[ui:emu.start]] rechaza un emulador con un problema.

## Límites {#limits}

| Qué | Límite | Al llegar al límite |
| --- | --- | --- |
| Rutas o reglas por emulador | 64 | Se rechaza al comprobarlo. |
| Respuestas por ruta | 16 | Se rechaza. |
| Condiciones por ruta | 16 | Se rechaza. |
| Encabezados por respuesta | 32 | Se rechaza. |
| Condiciones de argumentos, argumentos de respuesta (OSC) | 16 de cada | Se rechaza. |
| Mensajes retenidos (MQTT) | 64 | Se rechaza. |
| Un cuerpo, una respuesta o un saludo, tal como se escribe | 256 KiB | Se rechaza. |
| Un retardo o un jitter | 60 000 ms | Se rechaza. |
| Cuerpo de una solicitud HTTP | 1 MiB | 413. |
| Conexiones HTTP a la vez | 512 | Las demás se cierran según llegan. |
| Cabecera de una solicitud HTTP | 30 s | Un cliente debe enviarla en este tiempo. |
| Conexiones TCP a la vez | 256 | Las demás se cierran según llegan. |
| Respuestas OSC y UDP a la espera de su retardo | 1024 | Las demás se descartan y se cuentan en [[ui:emu.failed]]. |
| Clientes MQTT a la vez | 256 | Los demás se cierran según llegan. |
| Paquete MQTT | 256 KiB | La conexión del cliente termina. |
| Suscripciones MQTT por cliente | 100 | Las demás se rechazan. |
| Temas MQTT retenidos | 1000 temas, 16 MiB | Un nuevo mensaje retenido se encamina, pero no se retiene. |
| Mensajes MQTT a la espera de un cliente lento | 1024 mensajes, 8 MiB | El cliente se los pierde; se cuentan en [[ui:emu.missed]]. |

## Simular esto {#mock-this}

Para crear un emulador a partir de una respuesta que funcionó:

1. En la pantalla [[ui:nav.http]], envía una solicitud y obtén una respuesta, o usa
   [[ui:exp.sendNow]] en un nodo HTTP de un experimento.
2. Pulsa ⧉ [[ui:http.mockThis]] junto a la respuesta. El diálogo [[ui:http.mockTitle]] muestra la
   ruta que creará.
3. En [[ui:http.mockInto]], elige uno de tus emuladores HTTP, o [[ui:http.mockNew]].
4. Pulsa [[ui:http.mockAdd]]. La pantalla [[ui:nav.emulators]] se abre en ese emulador.

La ruta responde al método y la ruta de acceso de la solicitud (sin la consulta) con el estado, los
encabezados y el cuerpo de la respuesta. Los encabezados propios de ese único intercambio
(`Content-Length`, `Date`, `Server`, `ETag` y similares) se omiten, y el cuerpo se envía tal como
era, aunque contenga `{{`. Un emulador nuevo contiene solo esta ruta. Si se añade a un emulador
existente, la ruta va la primera, de modo que responde antes que una ruta más general; uno en
marcha la incorpora cuando pulsas [[ui:emu.restart]].

Desde un experimento, una URL escrita con plantillas se convierte en un patrón: su base
(`{{api}}`) se elimina, un segmento que es una sola plantilla (`/orders/{{order_id}}`) pasa a ser
`:order_id`, y un segmento que solo en parte es plantilla termina la ruta de acceso con `*`.

## El conjunto inicial {#starter-set}

La primera vez que Signal Lab no encuentra ninguna biblioteca de emuladores, escribe cinco, todos
en este equipo. Sus nombres y notas se escriben en el idioma que tenga la interfaz en ese momento.

| Emulador | Escucha en | Hace |
| --- | --- | --- |
| [[ui:seed.emu.demo-api.name]] | `127.0.0.1:8080` | `GET /health` → `{"status":"ok","time":…}`; `GET /users/:id` → un usuario con ese id; `POST /users` → 201 con un `Location`; `GET /slow` → al cabo de 1500 ms; `/flaky` → 503, 503 y luego 200 desde entonces. |
| [[ui:seed.emu.osc-device.name]] | `127.0.0.1:9100` | `/ping` → `/pong` con el recuento; `/fader/*` → `/ack` con la dirección que recibió; `/cue/*` se acepta sin respuesta. |
| [[ui:seed.emu.udp-device.name]] | `127.0.0.1:7100` | `PING` → `PONG` y el recuento; cualquier otra cosa → `ACK` y su tamaño en bytes. |
| [[ui:seed.emu.tcp-device.name]] | `127.0.0.1:7200` | Líneas que terminan en CR LF. Saluda con `READY`; `POWER?` → `POWER=ON`; `POWER ON` o `POWER OFF` → `OK ON` / `OK OFF`; `QUIT` → `BYE`, y cuelga. |
| [[ui:seed.emu.mqtt-broker.name]] | `127.0.0.1:1883` | Retiene `online` en `lab/status`; `ON` u `OFF` publicado en `lab/<name>/set` → lo mismo, retenido, en `lab/<name>/state`. |

La señal inicial [[ui:seed.http-reachable.name]] de la
[biblioteca de señales](signals.md#starter-set) pregunta a `http://127.0.0.1:8080/`, la dirección
de la [[ui:seed.emu.demo-api.name]]: como esta no tiene ninguna ruta para `/`, recibe
404.

## El archivo de la biblioteca {#file}

La biblioteca es `emulators.json`, en la carpeta de datos (consulta
[Archivos](../reference/files.md)); pasa el puntero por encima del recuento bajo la lista para ver
su ruta. Se escribe entero 0,7 s después del último cambio, a través de un archivo temporal, así
que una escritura fallida deja el anterior. Si el archivo no se puede leer, la lista muestra el
error con la ruta, la línea y la columna, y el archivo se deja como está: corrígelo y pulsa
[[ui:emu.reload]]. Pulsa también [[ui:emu.reload]] después de editarlo a mano. Si no hay archivo,
se vuelve a escribir el conjunto inicial.

```json
{
  "version": 1,
  "emulators": [
    {
      "id": "orders-api",
      "note": "Stands in for the orders service.",
      "emulator": {
        "name": "Orders API",
        "bind": "127.0.0.1:18080",
        "protocol": "http",
        "routes": [
          { "method": "GET", "path": "/orders/:id",
            "responses": [{ "body": "{\"id\":\"{{request.params.id}}\",\"state\":\"open\"}" }] },
          { "method": "POST", "path": "/orders", "order": "sequence",
            "responses": [{ "status": 503 }, { "status": 201, "body": "{\"id\":\"{{uuid}}\"}" }] }
        ],
        "outage": { "up_ms": 20000, "down_ms": 2000, "fault": "unavailable" }
      }
    }
  ]
}
```

El objeto `emulator` por sí solo es un documento que también lee `signallab emulate`.

## En experimentos y scripts {#elsewhere}

- En un experimento, un nodo [[ui:exp.node.emulator]] abre su emulador antes del primer paso y
  responde hasta que termina la ejecución; lo que recibió se cuenta en el informe. Un emulador HTTP
  es ahí también lo que escucha [[ui:exp.node.wait_http]]
  ([Nodos](../experiments/nodes.md#node-wait_http)), y un emulador OSC o UDP comparte su
  puerto con las esperas de la ejecución. Dos emuladores del mismo transporte en un experimento no
  pueden compartir un puerto. Consulta [Nodos](../experiments/nodes.md#node-emulator) y
  [Fallos](../experiments/faults.md).
- `signallab emulate` ejecuta emuladores desde archivos o desde esta biblioteca hasta
  <kbd>Ctrl</kbd>+<kbd>C</kbd> o `--for`, mostrando lo que responden; consulta
  [La línea de comandos](../automation/cli.md#cli-emulate).

## Véase también {#related}

- [Inspector](inspector.md): cada intercambio, decodificado.
- [Degradación](impairment.md): una red deficiente entre tu sistema y un emulador.
- [Datos y plantillas](../experiments/data.md#templates)

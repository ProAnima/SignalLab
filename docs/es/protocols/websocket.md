---
title: WebSocket
description: Conéctate a un servicio ws:// o wss:// con los encabezados y subprotocolos que espera, envía mensajes de texto o binarios y lee cada mensaje según llega.
---

# WebSocket

La pantalla [[ui:nav.ws]] es un cliente WebSocket: abre una conexión a un
servicio — con los encabezados y subprotocolos que espera el servicio — envía texto
o bytes, y lista cada mensaje que va y viene, el más reciente al final. Úsala para
probar una API en vivo, una superficie de control o un dispositivo que habla WebSocket antes de
programarlo en un experimento.

## Conectar {#connect}

1. Abre [[ui:nav.ws]].
2. En [[ui:field.url]], escribe la dirección: `ws://127.0.0.1:9001/` o
   `wss://example.com/socket`.
3. Si el servicio los pide, escribe [[ui:exp.wsProtocols]] y añade
   [[ui:http.headers]] (un encabezado `Authorization` con un token, una cookie).
4. Pulsa [[ui:ws.connect]]. Mientras se ejecuta el upgrade, el botón dice
   [[ui:ws.connecting]]; cuando termina, los campos se bloquean y el botón pasa a ser
   [[ui:ws.disconnect]].

| Campo | Qué | Predeterminado |
| --- | --- | --- |
| [[ui:field.url]] | `ws://` o `wss://`, un host, un puerto opcional (80 para `ws`, 443 para `wss`) y una ruta de acceso | `ws://127.0.0.1:9001/` |
| [[ui:exp.wsProtocols]] | Subprotocolos que ofrecer, separados por comas, en orden de preferencia; el servidor elige uno. Un nombre no tiene espacios, comas ni barras. | ninguno |
| [[ui:http.headers]] | Encabezados extra para la solicitud de upgrade; [[ui:http.addHeader]] añade una fila. Una fila sin nombre se omite. | ninguno |

Conectar — la resolución del nombre, la conexión TCP, TLS para `wss://` y el
upgrade — tiene 10 segundos. La URL se conserva cuando cambias de pantalla y cuando
reinicias la aplicación; los encabezados y subprotocolos no.

Cuando está conectada, el panel muestra:

| Elemento | Qué |
| --- | --- |
| [[ui:ws.state]] | [[ui:ws.open]], o una vez termina: cerrada por ti o por el servidor, con el código de cierre, o [[ui:ws.lost]] cuando la línea se rompió sin un cierre |
| [[ui:field.reason]] | El motivo que dio el lado que cierra, si lo hay |
| [[ui:ws.subprotocol]] | El subprotocolo que eligió el servidor, o — |
| [[ui:ws.peer]] | El `IP:port` del servidor |
| [[ui:ws.upgradeTime]] | Cuánto tardaron la conexión y el upgrade, en milisegundos |

La conexión es una tarea: aparece en la franja de la consola y también se puede detener
desde ahí.

### Conexiones seguras {#wss}

`wss://` confía en los mismos certificados que HTTPS en este sistema: un servidor cuyo
certificado este sistema no confía (autofirmado, caducado, otro nombre) se
rechaza con "A secure connection to … could not be made". No hay ningún ajuste
para saltarse la comprobación.

## Enviar un mensaje {#send}

1. En [[ui:ws.message]], elige [[ui:exp.wsText]] o [[ui:exp.wsBinary]].
2. Escribe el mensaje. Para binario, escribe los bytes como pares de dígitos hex:
   `de ad be ef`.
3. Pulsa [[ui:common.send]], o <kbd>Ctrl</kbd>+<kbd>Enter</kbd> en el mensaje.

Un mensaje es de 16 MiB como máximo (16 777 216 bytes, contados como bytes para binario, no
como dígitos hex), el mismo límite que para un mensaje que llega y para el
paso [[ui:exp.node.ws_send]]. Uno más largo se rechaza con `Too long: at most
16777216` antes de enviar nada, y la conexión sigue abierta.

Cuando un mensaje de texto es JSON, [[ui:http.formatJson]] lo formatea con sangrías
antes de que lo envíes. El mensaje se conserva cuando cambias de pantalla y cuando
reinicias la aplicación.

## Leer los mensajes {#messages}

[[ui:ws.messages]] lista lo que se recibió (↓) y se envió (↑), el más reciente al final, con
la hora, el inicio del mensaje (300 caracteres) y su tamaño; un mensaje
binario muestra sus bytes en hex y se marca [[ui:ws.binary]]. Sobre la lista están
cuántos se recibieron y se enviaron. La lista sigue los mensajes nuevos mientras está
desplazada hasta el final; desplázala hacia arriba y se queda donde estás.

Haz clic en un mensaje para verlo entero bajo la lista: JSON formateado, binario como hex.
[[ui:ws.editHere]] lo copia al campo del mensaje, para enviarlo otra vez o cambiarlo.
Un mensaje muy largo se muestra en parte — texto hasta 64 KiB, binario hasta
4096 bytes — y entonces no se puede copiar, porque se enviaría cortado.

La pantalla conserva los 2000 mensajes más recientes; [[ui:common.clear]] vacía la
lista. Si un servicio envía más rápido de lo que la pantalla puede asumir — más de 2000 en una
décima de segundo — los más antiguos de esos se omiten de la lista y se cuentan como
"sin mostrar". El Inspector todavía los tiene mientras la captura está activada.

## Cerrar {#close}

[[ui:ws.disconnect]] envía una trama de cierre con el código 1000 (normal) y espera
hasta 2 segundos la respuesta del servidor antes de colgar. El estado lee entonces
cerrada con 1000. Cuando el servidor cierra, el estado muestra su código y motivo;
cuando la conexión se rompe sin una trama de cierre, lee [[ui:ws.lost]] y
la consola dice por qué.

Signal Lab responde por sí mismo a los pings del servidor; los pings y pongs no se
listan. Una conexión también termina cuando llega un mensaje de más de 16 MiB, o cuando
enviar uno tarda más de 10 segundos porque el servidor dejó de leer.
(Enviar uno de más de 16 MiB tú mismo se rechaza y no termina nada.)

## En el Inspector {#inspector}

Con la captura activada, el tráfico de la conexión aparece con el protocolo `ws`
y el origen `websocket`:

| Resumen | Qué |
| --- | --- |
| `CONNECT ws://… (subprotocol)` | La conexión se abrió |
| `TEXT …` | Un mensaje de texto y su inicio |
| `BINARY n B …` | Un mensaje binario, su tamaño y sus primeros 16 bytes |
| `CLOSE code reason` | Una trama de cierre, enviada o recibida |

Cada trama de mensaje conserva sus bytes. Mientras el tráfico es ligero se
captura cada mensaje; una conexión ocupada se limita a 200 tramas por segundo, y la siguiente trama
capturada dice cuántas se omitieron (`+n not shown`). Consulta
[Inspector](../tools/inspector.md).

## En experimentos {#experiments}

Cuatro pasos programan una conversación WebSocket. Un paso abre una conexión
y los demás la nombran:

| Paso | Qué hace |
| --- | --- |
| [[ui:exp.node.ws_connect]] | Abre una conexión para el resto de la ejecución. Su URL y encabezados aceptan `{{templates}}`, así que un token extraído antes puede ir en ellos. [Detalles](../experiments/nodes.md#node-ws_connect) |
| [[ui:exp.node.ws_send]] | Envía un mensaje de texto o binario por una conexión. [Detalles](../experiments/nodes.md#node-ws_send) |
| [[ui:exp.node.wait_ws]] | Espera un mensaje cuya carga útil coincida, como hace [[ui:exp.node.wait_udp]]; los mensajes JSON se pueden leer campo por campo después. [Detalles](../experiments/nodes.md#node-wait_ws) |
| [[ui:exp.node.ws_close]] | Cierra una conexión con un apretón de cierre: código 1000, o 3000–4999 para los propios de una aplicación, y un motivo de hasta 123 bytes. [Detalles](../experiments/nodes.md#node-ws_close) |

Una conexión que la ejecución aún tiene abierta cuando termina — o se detiene — se
cierra correctamente. La plantilla [[ui:exp.templateWsEcho]] es un ejemplo resuelto.

## Desde la línea de comandos {#cli}

`signallab send ws` hace un intercambio: conectar, enviar un mensaje, esperar la
respuesta si la pides, cerrar.

```bash
signallab send ws ws://127.0.0.1:9001/ --text '{"type":"ping"}' --expect pong
```

El apretón de manos y lo que se envió van a la salida de error estándar, la respuesta a la
salida estándar:

```text
Connected to ws://127.0.0.1:9001/ in 4 ms
Sent 15 bytes
{"type":"pong"}
```

`--hex` envía un mensaje binario; `-H` añade un encabezado y `--protocol` ofrece un
subprotocolo (ambos repetibles). `--expect TEXT`, `--expect-regex RE` o `--wait`
(cualquier mensaje) dicen qué respuesta esperar, durante `--timeout` milisegundos (2000
por defecto). Sale con 1 cuando la respuesta no llega o la conexión
falla. Consulta [Línea de comandos](../automation/cli.md#cli-send-ws).

## Problemas {#troubleshooting}

| Lo que ves | Causa habitual |
| --- | --- |
| `… is not a WebSocket address` | La URL no empieza por `ws://` o `wss://`, o no tiene host. |
| `… answered HTTP n instead of switching to WebSocket` | El servidor rechazó el upgrade: una ruta de acceso equivocada (404), un token ausente o incorrecto (401, 403). El inicio de su respuesta está bajo los detalles técnicos. |
| `… did not take any of the subprotocols offered` | Ofreciste subprotocolos y el servidor no eligió ninguno, o respondió con uno que no ofreciste. |
| `… is not a subprotocol name` | Un nombre con un espacio, una coma o una barra. |
| `The header … cannot be sent with the upgrade` | Un nombre o valor de encabezado con caracteres que HTTP no permite. |
| `… refused the connection` | Nada escucha en ese puerto. |
| `A secure connection to … could not be made` | El certificado no es de confianza aquí, o TLS falló. Consulta [Conexiones seguras](#wss). |
| `The connection with … broke: the server did not keep to the WebSocket protocol` | El servidor envió algo que no es WebSocket válido. |
| `A WebSocket message is limited to … bytes` | El servidor envió un mensaje de más de 16 MiB, que termina la conexión. |
| `Too long: at most 16777216` | El mensaje que intentaste enviar supera los 16 MiB. No se envió nada; la conexión está abierta. |

Cada mensaje de error está en [Mensajes de error](../reference/errors.md#ws).

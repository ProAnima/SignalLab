---
title: Inspector
description: Captura cada trama que envían y reciben las herramientas de Signal Lab, fíltralas y léelas decodificadas y como bytes, expórtalas y guarda una como señal.
---

# Inspector

El Inspector es una sola línea de tiempo para todas las herramientas: cada
mensaje OSC, datagrama, intercambio HTTP, publicación MQTT, mensaje WebSocket,
paquete retransmitido e intercambio de emulador aterriza ahí, decodificado, con
los bytes de los que estaba hecho. Úsalo para ver qué pasó realmente por el
cable, en qué orden y qué le ocurrió.

Vive en el panel inferior, como la pestaña [[ui:dock.inspector]] junto a la
consola, así que está en todas las pantallas. Haz clic en la pestaña para
abrirlo; el panel se abre con altura suficiente para unas filas y el detalle de
una trama. El botón ⤢ ([[ui:dock.maximise]]) hace que el panel ocupe toda la
altura de la ventana. El Inspector conserva su lista y su selección mientras
cierras el panel o cambias de pantalla.

## Capturar {#capture}

La captura está desactivada cuando Signal Lab arranca, y mientras está
desactivada no cuesta nada: las herramientas ni siquiera construyen tramas.

1. Abre la pestaña [[ui:dock.inspector]].
2. Pulsa [[ui:ins.arm]]. El punto de la pestaña se vuelve rojo y late.
3. Usa cualquier herramienta. Las tramas aparecen en la parte superior de la
   lista, la más reciente primero.
4. Pulsa [[ui:ins.disarm]] cuando tengas lo que necesitas.

La pestaña muestra cuántas tramas se han capturado, desde cualquier pantalla.

Las tramas se capturan desde el momento en que activas la captura, nunca antes:
actívala primero y luego envía.

En un [servidor](../server/index.md), la captura pertenece al servidor: todas las
páginas conectadas a él ven las mismas tramas, y activarla o vaciarla en una
página lo hace para todas.

### Qué se captura {#sources}

| Herramienta | Tramas | Cuántas |
| --- | --- | --- |
| [OSC](../protocols/osc.md): envío | Cada mensaje enviado | Todos |
| [OSC](../protocols/osc.md): monitor | Cada paquete recibido; uno que no se decodifica se marca con el error de decodificación | Todos |
| [OSC](../protocols/osc.md): generador de señales | Mensajes enviados | Como máximo uno cada 100 ms, marcado `sampled` |
| [Difusión](../protocols/broadcast.md): enviar una vez | Cada datagrama, uno por destino; un envío fallido con su error | Todos |
| [Difusión](../protocols/broadcast.md): baliza | Datagramas enviados | Como máximo uno cada 50 ms |
| [Difusión](../protocols/broadcast.md): escucha de descubrimiento | Sondeos recibidos | Como máximo uno cada 40 ms |
| [Difusión](../protocols/broadcast.md): escucha de descubrimiento | Sus respuestas (`auto-reply`) | Todas |
| [HTTP](../protocols/http.md): envío, señales, solicitudes de experimentos | Cada intercambio: la línea de solicitud, el estado y el tiempo, los encabezados de la respuesta y el inicio del cuerpo | Todos |
| [HTTP](../protocols/http.md): ráfaga de carga y solicitudes bajo carga | Intercambios | Como máximo uno cada 100 ms |
| [MQTT](../protocols/mqtt.md): conexión | Publicaciones enviadas | Todas |
| [MQTT](../protocols/mqtt.md): conexión | Mensajes recibidos | Como máximo uno cada 200 ms |
| [MQTT](../protocols/mqtt.md): una señal MQTT enviada mientras la pantalla no está conectada a su bróker | La publicación | Todas |
| [WebSocket](../protocols/websocket.md) | Mensajes enviados y recibidos | Todos mientras el tráfico es ligero; como máximo 200 por segundo |
| [Emuladores](emulators.md) | Lo que llega y la respuesta, juntos | Como máximo un intercambio cada 10 ms |
| [Degradación](impairment.md) | Cada datagrama o fragmento retransmitido, en ambos sentidos, con su suerte | Como máximo uno cada 25 ms para los dos sentidos juntos |
| [Tormenta](storm.md) (UDP) | Paquetes de inundación, todos idénticos | Uno por segundo, marcado `sampled 1/s`; una tormenta TCP no captura ninguno |
| [Escáner](scanner.md) | Cada puerto abierto, con su banner | Todos |
| [Experimentos](../experiments/index.md) | Lo que envían los pasos de una ejecución (un mensaje TCP: la carga útil escrita y la respuesta leída) y lo que reciben sus esperas | Como la herramienta que usa |

Una herramienta que muestrea deja el resto fuera a propósito y los cuenta: la
siguiente trama que sí dibuja dice cuántas se guardó, en su veredicto como
`+n not shown` (`sampled · +5 not shown`). El recuento es de lo que la
herramienta habría dibujado, no de lo que la propia captura descartó (consulta
[Recuentos y huecos](#counts)).

## La lista de tramas {#list}

| Columna | Qué |
| --- | --- |
| [[ui:common.time]] | Cuándo se capturó, al milisegundo. |
| [[ui:ins.dir]] | → enviada (`tx`), ← recibida (`rx`). Para un relé, → es del cliente al destino y ← del destino al cliente. |
| [[ui:bc.proto]] | `osc`, `udp`, `tcp`, `http`, `mqtt` o `ws`. |
| [[ui:bc.peer]] | El otro extremo: un `IP:port`, una URL, un bróker. |
| [[ui:common.bytes]] | El tamaño de la trama. |
| [[ui:ins.summary]] | Una línea en la notación propia del protocolo, como `/fader/1 0.75` o `GET http://127.0.0.1:8080/ → 200 in 3ms`. |
| [[ui:ins.verdict]] | Qué le ocurrió, cuando hay algo que decir. |

El veredicto es verde, ámbar o rojo. Rojo es una pérdida o un fallo
(`dropped (loss)`, `failed`, `error: …`); ámbar es una trama alterada o solo una
muestra de muchas (`corrupted`, `copy 2/2`, `sampled`, `+n not shown`); verde es
el resto. Algunos veredictos que verás:

| Veredicto | De | Significa |
| --- | --- | --- |
| `forwarded +42ms` | Degradación | Reenviada tras ese retardo; pueden seguir `· corrupted`, `· reordered` o `· copy 1/2`. |
| `dropped (loss)`, `dropped (burst)`, `dropped (offline)` | Degradación | Perdida a propósito, y por qué. |
| `throttled` | Degradación | Descartada por el límite de ancho de banda. |
| `· client→target`, `· target→client` | Degradación | Cierra el veredicto de cada trama retransmitida, antes de cualquier `+n not shown`: hacia dónde iba. |
| `#2 → 200 OK · 37 B`, `— → 404 …` | Emuladores | Qué regla respondió (`—`: ninguna) y la respuesta. |
| `down`, `down → 503` | Emuladores | Llegó mientras el emulador estaba caído. |
| `200 OK`, `failed` | HTTP | El estado de la respuesta, o ninguna respuesta. |
| `open` | Escáner | Un puerto abierto. |
| `auto-reply` | Descubrimiento | Una respuesta que la escucha envió a un sondeo. |
| `clears retained` | MQTT | Una publicación retenida vacía. |
| `+n not shown` | Cualquier herramienta que muestree | Se dejaron fuera esas tramas desde la anterior; sigue al otro veredicto de la trama tras un `·`. |

La lista conserva las 4000 tramas más recientes y dibuja las 300 más recientes
que coinciden con los filtros; bajo los filtros dice cuántas muestra de cuántas
coinciden.

### Filtrar {#filter}

- Escribe en el campo de texto ([[ui:ins.filterPlaceholder]]) para conservar las
  tramas cuyo resumen, extremo, origen, protocolo o veredicto contenga el texto.
- Haz clic en las etiquetas de protocolo (`osc`, `udp`, `tcp`, `http`, `mqtt`,
  `ws`) para mostrar solo esos protocolos. Sin ninguna activada, se muestran
  todos los protocolos.
- Haz clic en `tx` o `rx` para mostrar solo las tramas enviadas o solo las
  recibidas.
- [[ui:common.reset]] limpia los tres.

Los filtros cambian solo lo que muestra la lista. La captura, los recuentos y una
exportación siempre abarcan todo.

### Pausar y vaciar {#pause}

[[ui:ins.pause]] congela la lista para que puedas leerla mientras el tráfico
continúa; la captura sigue. [[ui:ins.resume]] deja entrar tramas nuevas otra vez.
Las tramas que llegaron mientras la vista estaba en pausa no se añaden a la
lista, pero están en la captura y en una exportación.

[[ui:common.clear]] vacía la lista y la captura, y restablece sus recuentos.

### Recuentos y huecos {#counts}

La barra de la parte superior cuenta las tramas capturadas y sus bytes, y cuán
llena está la captura (tramas retenidas de 8192).

Cuando las tramas llegan más rápido de lo que la lista puede aceptarlas —más de
250 en una octava parte de segundo—, la lista omite las más antiguas. Una
etiqueta ámbar cuenta entonces las tramas no mostradas, y una fila en la lista
marca dónde faltan. Esas tramas siguen en la captura, a menos que otras más
nuevas las hayan desplazado desde entonces: expórtala para verlas.

## El detalle de una trama {#detail}

Haz clic en una fila para ver la trama a la derecha.

| Campo | Qué |
| --- | --- |
| [[ui:ins.seq]] | El número de la trama. Los números suben en orden de captura y nunca se reutilizan. |
| [[ui:common.time]] | Cuándo se capturó. |
| [[ui:ins.direction]] | Enviada o recibida. |
| [[ui:common.protocol]] | Como en la lista. |
| [[ui:ins.source]] | La herramienta que la capturó (`osc-send`, `osc-monitor`, `netsim`, `emulator`, `experiment-wait`, …), y su número de tarea cuando pertenece a una. |
| [[ui:ins.local]] | La dirección de este lado, cuando la hay. Para una trama retransmitida, la dirección en la que escucha el relé. |
| [[ui:bc.peer]] | El otro extremo. Para una trama retransmitida, hacia dónde iba: el destino, o el cliente al que volvió la respuesta. |
| [[ui:ins.size]] | Su tamaño en bytes. |
| [[ui:ins.verdict]] | Como en la lista. |

Bajo [[ui:ins.decoded]] está la trama leída en su protocolo: cada mensaje de un
bundle OSC con sus argumentos, los encabezados de una respuesta HTTP y el inicio
de su cuerpo, la solicitud y la respuesta de un emulador.

Bajo [[ui:ins.rawBytes]] hay un volcado hex: desplazamiento, 16 bytes en hex y
los mismos bytes como texto. La lista lleva el primer KiB de cada trama; cuando
una trama es más larga, un botón bajo el volcado carga todo.

### Qué conserva una trama {#limits}

| Límite | Valor | Al llegar al límite |
| --- | --- | --- |
| Bytes que conserva una trama | 256 KiB | Una trama más larga conserva sus primeros 256 KiB y dice cuánto del total conservó. |
| Tramas en la captura | 8192 | La trama más antigua hace sitio. |
| Bytes que conserva la captura en total | 64 MiB | Las tramas más antiguas hacen sitio. |

Algunas tramas no conservan bytes: los intercambios HTTP (su tamaño se registra,
y los encabezados de la respuesta y el inicio del cuerpo están en el texto
decodificado) y los puertos abiertos del Escáner.

Una trama MQTT conserva la carga útil del mensaje, no el paquete del protocolo
que lo rodea; el tema, el QoS y el indicador de retención están en su resumen.

### Secretos {#secrets}

Mientras una ejecución de experimento o [[ui:exp.sendNow]] usa
[secretos](../experiments/data.md#secrets), sus valores se enmascaran en cada
trama antes de capturarla: `••••` en el resumen, el texto decodificado, las
direcciones y el veredicto, y `*` por cada byte de la carga útil, para que los
desplazamientos del volcado sigan siendo ciertos. Las credenciales de la pantalla
HTTP tampoco aparecen nunca: una trama HTTP contiene la respuesta, no el
encabezado `Authorization` que se envió.

## Guardar una trama como señal {#save-as-signal}

Para guardar un paquete que has capturado y volver a enviarlo más tarde, cuando
el dispositivo que lo envió ya no está:

1. Selecciona la trama.
2. Pulsa [[ui:sig.fromFrame]].

La señal va a la carpeta [[ui:sig.capturedFolder]] de la
[biblioteca de señales](signals.md) con todos los bytes de la trama, tomados de
lo que conservó la captura, no del texto decodificado. Se nombra a partir del
resumen de la trama, y su nota dice de qué trama viene.

Lo que obtienes depende de la trama:

| Trama | Señal |
| --- | --- |
| Un datagrama OSC o UDP | Una señal UDP sin procesar con los bytes de la trama, en hex. |
| Una publicación MQTT —enviada, recibida o de un emulador— | Una señal MQTT con el bróker, el tema, el QoS y el indicador de retención de la trama, y su carga útil como texto, exactamente como estaba. Una carga útil vacía se conserva, así que limpiar un valor retenido se puede guardar. |
| Cualquier otra cosa: un flujo TCP (incluido uno que llevó un relé, o el del nodo TCP), un intercambio HTTP, un mensaje WebSocket, un paquete MQTT que no es una publicación (la suscripción de un cliente en un emulador) | Nada: el botón está desactivado y su consejo dice por qué. Una señal envía un datagrama o una publicación; estas no se pueden volver a enviar como estaban. |

Un datagrama se envía a:

- una trama recibida: la dirección que la recibió (el lado [[ui:ins.local]]),
  para que la señal haga el papel del emisor;
- una trama enviada: el extremo al que se envió;
- una trama retransmitida, en cualquier sentido: la dirección hacia la que iba:
  el destino para una trama que va desde el cliente, el cliente para una
  respuesta.

Cuando esa dirección es todas las direcciones de este equipo —un monitor que
escucha en `0.0.0.0:9000` o `[::]:9000`—, la señal se dirige a este equipo en su
lugar: `127.0.0.1:9000` o `[::1]:9000`. Abre la señal y cambia su destino si
quieres otra dirección.

Una señal MQTT va al bróker que indica la trama. Se llega a un bróker que escucha
en todas las direcciones por `127.0.0.1` de la misma manera.

El botón también está desactivado para:

- una trama que no se conservó entera: una mayor de 256 KiB, o una que no
  conservó ningún byte;
- un mensaje MQTT cuya carga útil no es texto: la carga útil de una señal es
  texto, así que sus bytes no se podrían volver a enviar como estaban;
- una trama recibida que no nombra ningún socket de este lado, así que no hay
  dirección a la que enviar.

Una trama que la captura ya ha soltado tampoco se puede guardar; la consola lo
indica. Mientras no se puede leer el archivo de la biblioteca,
[[ui:sig.fromFrame]] también está desactivado, y el consejo muestra el error del
archivo (consulta [Señales](signals.md#file)).

## Exportar {#export}

[[ui:ins.exportJsonl]] y [[ui:ins.exportTxt]] escriben toda la captura —hasta
8192 tramas, cada byte que conservó cada una, digan lo que digan los filtros— en
un archivo `capture-<time>.jsonl` o `capture-<time>.txt` en la carpeta de datos
(consulta [Archivos](../reference/files.md)). La consola dice dónde. En un
navegador conectado a un servidor, el archivo se escribe en el servidor y tu
navegador lo descarga.

Una captura vacía no se escribe; la consola dice que no hay nada que guardar.

- **`.jsonl`** — un objeto JSON por línea, una línea por trama: `seq`, `ts`
  (milisegundos desde 1970), `proto`, `dir`, `source`, `job_id`, `local`,
  `remote`, `bytes`, `kept`, `summary`, `detail`, `hex` (el volcado del primer
  KiB), `verdict` y `data`, los bytes que conservó en base64.
- **`.txt`** — para leer: una línea por trama con su número, hora, dirección,
  protocolo, extremo, tamaño y veredicto, y luego su resumen, su texto
  decodificado y un volcado hex de cada byte que conservó.

```json
{"seq":12,"ts":1767225600123,"proto":"osc","dir":"rx","source":"osc-monitor","job_id":3,"local":"0.0.0.0:9000","remote":"127.0.0.1:53211","bytes":20,"summary":"/fader/1 0.75","detail":"/fader/1 0.75","hex":"0000  2f 66 61 64 65 72 2f 31  00 00 00 00 2c 66 00 00  |/fader/1....,f..|\n0010  3f 40 00 00                                       |?@..|\n","verdict":null,"kept":20,"data":"L2ZhZGVyLzEAAAAALGYAAD9AAAA="}
```

## Venir de otros sitios {#reveal}

Otras pantallas apuntan a tramas: una espera en la línea de tiempo de un
experimento enlaza la trama con la que coincidió, y la lista de lo recibido de un
emulador tiene un botón ⌕ ([[ui:emu.inspectFrame]]) en cada intercambio. Seguir
uno abre el Inspector con esa trama seleccionada, los filtros limpios y la vista
reanudada.

## Véase también {#related}

- [Señales](signals.md): en qué se convierte una trama guardada.
- [Degradación](impairment.md): los veredictos del relé.
- [Emuladores](emulators.md): lo que recibió un emulador.

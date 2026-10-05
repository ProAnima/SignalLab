---
title: MQTT
description: Conéctate a un bróker MQTT 3.1.1, observa cada tema que guarda como un árbol en vivo, publica con QoS 0, 1 o 2, anuncia una última voluntad y borra valores retenidos atascados.
---

# MQTT

La pantalla [[ui:nav.mqtt]] es un cliente MQTT para mirar un bróker y cambiar lo que
contiene. Conéctate y, de forma predeterminada, se suscribe a `#`: cada tema que guarda
el bróker se forma como un árbol con su último valor. Desde ahí publicas, borras un valor
retenido, guardas un tema como señal o lo conviertes en un paso de un experimento.

Signal Lab habla **MQTT 3.1.1 sobre TCP sin cifrar**, con QoS 0, 1 y 2 para suscribirse,
publicar y la última voluntad. No hay MQTT 5 ni TLS: un bróker que solo acepte clientes
`mqtts://` o MQTT 5 no se puede alcanzar.

## Conectar {#connect}

1. Abre [[ui:nav.mqtt]].
2. Escribe [[ui:mq.host]] y [[ui:common.port]].
3. Deja [[ui:mq.clientId]] como está, salvo que el bróker espere uno concreto. Añade
   [[ui:mq.username]] y [[ui:mq.password]] solo si el bróker los pide.
4. Pulsa [[ui:mq.connect]].

Conectar abre la conexión TCP y completa la negociación MQTT antes que nada, así que una
contraseña equivocada o un puerto cerrado se indica ahí mismo. Los campos de conexión se
bloquean mientras está conectado; [[ui:mq.disconnect]] la cierra. La conexión es una tarea
en la franja de la consola y también se puede detener allí.

| Campo | Qué | Predeterminado |
| --- | --- | --- |
| [[ui:mq.host]] | La dirección IP o el nombre de host del bróker | `127.0.0.1` |
| [[ui:common.port]] | El puerto del bróker | `1883` |
| [[ui:mq.clientId]] | El nombre de tu cliente en el bróker. No puede estar vacío y debe ser único allí: un segundo cliente con el mismo id expulsa al primero. | `signal-lab-` y seis dígitos hexadecimales aleatorios, nuevos cada vez que se inicia la aplicación |
| [[ui:mq.username]], [[ui:mq.password]] | Se envían solo si el bróker los necesita — en texto plano, ya que no hay TLS. Una contraseña sin nombre de usuario no se envía: MQTT 3.1.1 no puede transportarla. | vacío |
| [[ui:mq.keepAlive]] | Segundos que la conexión puede estar en silencio. Signal Lab hace ping al bróker cada mitad de ese tiempo; un bróker descarta a un cliente que calla 1,5 veces ese tiempo. 0 desactiva el ping. | `60` |
| [[ui:mq.cleanSession]] | Activado: cada conexión empieza sin suscripciones guardadas ni mensajes en cola. Desactivado pide al bróker que las conserve para este id de cliente entre conexiones. | activado |
| [[ui:mq.scanFilter]] y su [[ui:mq.qos]] | Un filtro al que se suscribe en cuanto la conexión está lista; `#` es todos los temas. Vacío: ninguno. | `#`, QoS 0 |
| [[ui:mq.willEnable]] | Dar una última voluntad al bróker ([más abajo](#will)) | desactivado |

El bróker tiene 6 segundos para aceptar la conexión TCP y 6 más para responder a la
negociación.

### La última voluntad {#will}

Una última voluntad es un mensaje que el bróker guarda por ti y publica por su cuenta si tu
conexión muere sin una despedida correcta. La presencia suele construirse así: un
dispositivo publica `online` en su tema de estado, y su voluntad pone ese mismo tema en
`off`.

Con [[ui:mq.willEnable]] marcada, define [[ui:mq.willTopic]] y
[[ui:mq.willPayload]] (`off` de forma predeterminada). La voluntad se publica con QoS 2 y
retenida. Sin un tema, no se envía ninguna voluntad.

## Suscribirse {#subscribe}

El filtro de exploración se suscribe al conectar. Para más:

1. En [[ui:mq.addSubscription]], escribe un filtro de temas.
2. Elige su [[ui:mq.qos]].
3. Pulsa [[ui:mq.subscribe]] o <kbd>Enter</kbd>.

Un filtro es un tema con comodines:

| Comodín | Representa | Ejemplo |
| --- | --- | --- |
| `+` | exactamente un nivel | `sensors/+/state` coincide con `sensors/door/state` |
| `#` | todos los niveles inferiores, solo como último carácter | `sensors/#` coincide con `sensors/door/state` y `sensors` |

[[ui:mq.subscriptions]] enumera cada filtro con lo que concedió el bróker:
`qos0`, `qos1` o `qos2` — el bróker puede conceder menos de lo que pediste — o
[[ui:mq.refused]]. [[ui:mq.unsubscribe]] cancela la suscripción a uno.

| QoS | Entrega |
| --- | --- |
| 0 | Como mucho una vez: se envía y se olvida |
| 1 | Al menos una vez: se confirma, puede llegar dos veces |
| 2 | Exactamente una vez: una negociación en dos pasos; una reentrega no se muestra dos veces |

## El árbol de temas {#topics}

Cada mensaje que llega va a [[ui:mq.topics]], un árbol de los niveles de tema. Un tema
muestra su último valor, una **R** cuando ese valor está retenido, y cuántos mensajes ha
tenido cuando hay más de uno. Haz clic en un nivel para abrirlo o cerrarlo.

- Escribe en el campo sobre el árbol para listar solo los temas cuya ruta o último valor
  contenga el texto.
- Sobre el árbol están el número de temas, cuántos guardan un valor retenido y, mientras
  estás conectado, el bróker en el que escuchas.
- Las cargas útiles se muestran como texto; los bytes que no son UTF-8 se muestran como
  caracteres de reemplazo.
- [[ui:common.clear]] vacía el árbol. Nada más lo hace: se queda como está cuando cambias
  de pantalla o te desconectas, hasta que se cierra la aplicación.

Los mensajes llegan a la pantalla en lotes, diez veces por segundo. Cuando un bróker envía
más de 4000 mensajes en una décima de segundo, los más antiguos de ese lote se dejan fuera
del árbol y se cuentan como "no mostrados" sobre él.

### El panel de un tema {#topic}

Elige un tema con un valor para verlo bajo el árbol: [[ui:mq.value]],
[[ui:mq.qos]], [[ui:mq.retain]], [[ui:common.bytes]], [[ui:mq.messages]] y
[[ui:mq.lastAt]]. Sus botones:

| Botón | Hace |
| --- | --- |
| [[ui:mq.editHere]] | Copia el tema, el valor, el QoS y el indicador de retención en [[ui:mq.publish]] |
| [[ui:mq.waitForThis]] | Añade un paso [Esperar un mensaje MQTT](../experiments/nodes.md#node-wait_mqtt) sobre este tema, en este bróker, cualquier carga útil, con 2000 ms de tiempo de espera, al experimento abierto |
| [[ui:mq.clearRetained]] | Quita el valor retenido ([más abajo](#clear-retained)) |
| [[ui:sig.fromFrame]] | Guarda el tema y su último valor como señal en la carpeta [[ui:sig.capturedFolder]] |

## Publicar {#publish}

1. Conéctate.
2. En [[ui:mq.publish]], escribe el [[ui:mq.topic]] y la
   [[ui:sig.payload]].
3. Elige el [[ui:mq.qos]] y marca [[ui:mq.retain]] si el bróker debe guardar
   el mensaje como valor del tema para cada cliente que se suscriba después.
4. Pulsa [[ui:mq.publishBtn]].

La consola confirma cada publicación: al instante para QoS 0, cuando el bróker la ha
confirmado para QoS 1 y 2. Un tema para publicar no tiene comodines y no está vacío: un
tema con `+` o `#` se rechaza antes de enviar nada, con el mismo mensaje que dan una señal,
un paso y `signallab send mqtt`, y la conexión se queda como estaba. Solo una suscripción
acepta filtros con comodines.

### Borrar un valor retenido {#clear-retained}

Un valor retenido permanece en el bróker hasta que se sustituye, y cada cliente que se
suscribe lo recibe primero — uno obsoleto es un motivo clásico de que un dispositivo
arranque en el estado equivocado. La única forma de quitarlo es publicar una carga útil
vacía con retain activado.

[[ui:mq.clearRetained]] en el panel de un tema hace eso: púlsalo y luego
[[ui:mq.clearConfirm]]. Publica la carga útil retenida vacía con QoS 1 en tu conexión. Solo
está disponible mientras estás conectado y cuando el último valor del tema está retenido.
Puedes hacer lo mismo a mano: una [[ui:sig.payload]] vacía con [[ui:mq.retain]] marcado.

::: warning
Borrar cambia el bróker para todos los clientes a la vez.
:::

## En el Inspector {#inspector}

Con la captura activada, el tráfico MQTT aparece con el protocolo `mqtt`:

| Origen | Qué | Cuántos |
| --- | --- | --- |
| `mqtt` | Lo que publica la conexión de la pantalla; una publicación retenida vacía tiene el veredicto `clears retained` | todos |
| `mqtt` | Mensajes que recibe la conexión | como máximo uno cada 200 ms |
| `mqtt-send` | Una publicación que trajo su propia conexión: una señal enviada mientras la pantalla no está conectada al bróker de la señal, un paso, `signallab send mqtt` (veredicto `one-shot`) | todas |
| `experiment-wait` | Mensajes que recibe la suscripción de un paso [[ui:exp.node.wait_mqtt]], aparte de las repeticiones retenidas | todos |

El resumen dice `topic = payload`, con el QoS y `retained` cuando corresponden. Consulta
[Inspector](../tools/inspector.md).

## Guardar y reutilizar {#library}

- **Guardar como señal.** [[ui:sig.saveNew]] bajo [[ui:mq.publish]] guarda el
  bróker (el [[ui:mq.host]] y el [[ui:common.port]] de la conexión), el tema,
  la carga útil, el QoS y el indicador de retención en la biblioteca de señales;
  <kbd>Ctrl</kbd>+<kbd>S</kbd> en el panel de publicación hace lo mismo, y
  actualiza la señal una vez que el panel está vinculado a ella. Consulta
  [Señales](../tools/signals.md).
- **Enviar una señal MQTT.** Mientras esta pantalla está conectada al bróker que
  nombra la señal (mismo host, sin distinguir mayúsculas, y mismo puerto; `1883`
  cuando la señal no da ninguno), una señal enviada desde la biblioteca sale por
  esa conexión, con su id de cliente y sus credenciales. En caso contrario — no
  estás conectado, o lo estás a otro bróker — abre una conexión propia a su
  propio bróker — un id de cliente nuevo, sin nombre de usuario — publica, espera
  la confirmación que su QoS requiere y se desconecta. Los nombres no se
  resuelven, así que `localhost` y `127.0.0.1` cuentan como brókeres distintos.
  La biblioteca no guarda ninguna contraseña.
- **En un experimento.** Una señal MQTT guardada se puede elegir en
  [[ui:exp.group.signals]] en el menú [[ui:exp.addNode]] del experimento, lo que
  la convierte en un paso [[ui:exp.node.mqtt]].

## En experimentos {#experiments}

| Paso | Qué hace |
| --- | --- |
| [[ui:exp.node.mqtt]] | Conecta, publica un mensaje y se desconecta — sin nombre de usuario ni contraseña, una sesión limpia, en 15 segundos. [Detalles](../experiments/nodes.md#node-mqtt) |
| [[ui:exp.node.wait_mqtt]] | Se suscribe cuando empieza la ejecución y espera un mensaje en un filtro de temas cuya carga útil coincida; los valores retenidos repetidos al suscribirse se ignoran. [Detalles](../experiments/nodes.md#node-wait_mqtt) |
| [[ui:exp.node.emulator]] | Un bróker MQTT propio de la ejecución. [Detalles](../experiments/nodes.md#node-emulator) |

Ninguno de los dos pasos inicia sesión, así que necesitan un bróker que acepte clientes sin
nombre de usuario.

### El emulador de bróker {#broker-emulator}

Signal Lab también puede ser el bróker: un emulador [[ui:emu.new.mqtt]] encamina lo que
publican los clientes a quien se haya suscrito — 3.1.1, TCP sin cifrar, QoS 0, 1 y 2,
mensajes retenidos, últimas voluntades, un inicio de sesión opcional — y responde según
reglas, como un dispositivo. Apunta tu equipo y esta pantalla a él para probar sin un bróker
real. Consulta [Emuladores](../tools/emulators.md).

## Desde la línea de comandos {#cli}

`signallab send mqtt` publica un mensaje con una conexión propia:

```bash
signallab send mqtt 127.0.0.1:1883 lab/light/1/set on --qos 1
signallab send mqtt 127.0.0.1:1883 lab/light/1/state "" --retain
```

```text
✔ lab/light/1/set → 127.0.0.1:1883 · 2 B · qos1
```

La segunda línea borra un valor retenido. Sin un puerto, el bróker está en 1883. No usa
credenciales. Termina con 0 cuando el bróker aceptó el mensaje, 1 cuando no se pudo
alcanzar o lo rechazó. Consulta
[Línea de comandos](../automation/cli.md#cli-send-mqtt).

## Problemas {#troubleshooting}

| Qué ves | Causa habitual |
| --- | --- |
| `… refused the connection — nothing is listening on that port` | Ningún bróker en esa dirección y puerto. |
| `… accepted the connection but did not answer in time — is it an MQTT broker?` | Algo escucha allí, pero no habla MQTT, o lo habla sobre TLS. |
| `… answered with something other than MQTT 3.1.1` | No es un bróker MQTT, o un bróker que envió algo que Signal Lab no puede leer. |
| `… does not accept MQTT 3.1.1 clients` | El bróker solo acepta MQTT 5. |
| `… rejected the client ID — choose another one` | El id es demasiado largo o tiene caracteres que el bróker no acepta. |
| `… rejected the username or password` | Credenciales incorrectas, o una contraseña sin nombre de usuario. |
| `… did not authorize this client — check its access rules` | Las reglas de acceso del bróker rechazan a este cliente. |
| `… is unavailable right now — try again later` | El bróker está activo pero no acepta clientes. |
| `Enter a client ID — brokers refuse an empty one` | [[ui:mq.clientId]] está vacío. |
| `A publish topic cannot contain the wildcards + or #` | El tema al que publicar tiene un `+` o un `#`. Esos son para suscribirse; publica en un tema a la vez. |
| Un filtro muestra [[ui:mq.refused]] | Las reglas de acceso del bróker lo prohíben, o el filtro está mal formado (`#` no al final, `+` compartiendo nivel con otros caracteres). |
| La conexión se corta un momento después de conectar | Otro cliente se conectó con el mismo [[ui:mq.clientId]]. |
| No aparece nada en el árbol | El filtro de exploración está vacío, o el bróker no deja ver nada a este cliente. |

Cada mensaje de error se enumera en [Mensajes de error](../reference/errors.md#mqtt).

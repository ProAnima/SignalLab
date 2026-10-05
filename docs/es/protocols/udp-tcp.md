---
title: UDP y TCP
description: Dónde envía y recibe Signal Lab datagramas UDP y datos TCP sin formato — pasos de experimento, señales, la línea de comandos, las herramientas de carga y escaneo y los dispositivos emulados — y cómo se escriben las cargas útiles.
---

# UDP y TCP

UDP y TCP sin formato no tienen una pantalla propia. Son lo que usas para un
dispositivo con su propio protocolo de texto o binario — un proyector, un servidor de
medios, un sensor — y aparecen en varios sitios:

| Para… | Usa |
| --- | --- |
| enviar un datagrama o un mensaje TCP como paso, y esperar la respuesta | los [pasos de experimento](#experiments) |
| guardar un datagrama para enviarlo otra vez, o reproducir uno que capturaste | una [señal UDP](#signals) |
| enviar un datagrama desde un script | [`signallab send udp`](#cli) |
| enviar a muchos hosts a la vez, a una dirección de difusión o a un grupo de multidifusión | la pantalla [Difusión](broadcast.md) |
| cargar un servidor o un enlace con tráfico | [Tormenta](#storm) |
| averiguar qué puertos TCP tiene abiertos un host | [Escáner](#scanner) |
| hacer el papel del dispositivo | un emulador de [dispositivo UDP o TCP](#emulators) |
| empeorar la red entre dos extremos | un [relé de degradación](#impairment) |

OSC es un formato transportado en datagramas UDP; tiene su propia página:
[OSC](osc.md).

## Cargas útiles {#payloads}

Allí donde escribas una carga útil sin formato, es de uno de dos tipos:

| Tipo | Qué se envía | Ejemplo |
| --- | --- | --- |
| Texto | Los caracteres como UTF-8, exactamente como se escriben: sin terminador, sin añadir fin de línea. Un protocolo de líneas necesita su fin de línea en el texto. | `PING` |
| Bytes en hex | Byte a byte, escritos como pares de dígitos hex. Se permiten espacios, `:`, `-` y `,` entre pares y un `0x` delante de ellos. | `de ad be ef`, `DEADBEEF`, `0xde,0xad` |

Un datagrama lleva como máximo 65 507 bytes. Un número impar de dígitos hex, o
ninguno, es un error antes de enviar nada.

::: info
UDP no tiene acuse de recibo. "Enviado" significa que el datagrama salió de este equipo, no
que algo lo recibiera. Para saber que un dispositivo te oyó, espera su respuesta.
:::

## Adónde va {#destinations}

Un destino es una dirección IP y un puerto, o un nombre de host y un puerto:
`192.0.2.20:9000`, `[2001:db8::20]:9000` (una dirección IPv6 va entre corchetes),
`projector.local:9000`. Esto vale para el destino de un paso UDP, un destino OSC,
una señal UDP, `signallab send udp` y `signallab send osc`, la lista de
[Difusión](broadcast.md), el destino de [Tormenta](../tools/storm.md) y el host del
paso TCP.

Un nombre de host se resuelve cada vez que se usa. Cuando tiene una dirección IPv4,
se usa esa — así `localhost` llega a un servicio que escucha en `127.0.0.1`,
donde la primera dirección que un sistema lista para él puede ser `::1` — y un nombre que
solo tiene direcciones IPv6 se alcanza desde un socket IPv6. Un nombre que no
resuelve falla con `Cannot resolve …`; un destino sin puerto, o que no tiene
ninguna de las dos formas, con `… is not a valid address`. Las direcciones en las que un
servicio *escucha* (las de un monitor, una espera, un emulador) son siempre `IP:port`.

## En experimentos {#experiments}

| Paso | Qué hace |
| --- | --- |
| [[ui:exp.node.udp]] | Envía su [[ui:exp.payload]] como texto a [[ui:common.target]]. El destino es `IP:port` o `host:port` ([arriba](#destinations)), y varios destinos separados por comas reciben cada uno el datagrama. Con [[ui:exp.expectReply]] envía desde [[ui:exp.replyOn]] y espera ahí una respuesta en el mismo paso. [Detalles](../experiments/nodes.md#node-udp) |
| [[ui:exp.node.wait_udp]] | Escucha en [[ui:exp.listenOn]] (`IP:port`) y espera un datagrama cuya carga útil coincida. [Detalles](../experiments/nodes.md#node-wait_udp) |
| [[ui:exp.node.tcp]] | Se conecta a [[ui:exp.host]] y [[ui:exp.port]], escribe su [[ui:exp.payload]] como texto, escucha 250 ms por si hay respuesta y cierra. El paso dice cuántos bytes volvieron, no qué eran. [Detalles](../experiments/nodes.md#node-tcp) |

La carga útil y el destino de UDP, el host y la carga útil de TCP, y el patrón de una
espera aceptan `{{templates}}`, así que un datagrama puede llevar el id de la ejecución o un
valor que extrajo un paso anterior. Consulta [Datos y plantillas](../experiments/data.md).

### Hacer coincidir un datagrama {#matching}

[[ui:exp.node.wait_udp]] y la respuesta de [[ui:exp.node.udp]] eligen un datagrama
por su carga útil:

| [[ui:exp.waitMode]] | Acepta un datagrama cuando |
| --- | --- |
| [[ui:exp.mode.any]] | siempre: el primero que llegue |
| [[ui:exp.mode.contains]] | su carga útil, leída como texto, contiene el patrón |
| [[ui:exp.mode.regex]] | su carga útil, leída como texto, coincide con la expresión regular |
| [[ui:exp.mode.hex]] | sus bytes contienen los bytes del patrón, escritos en hex |

Lo que coincidió se guarda en la variable del paso ([[ui:exp.replyVariable]],
`reply` salvo que la renombres): `text`, `hex` (los primeros 1024 bytes), `bytes`
(el tamaño), `from` (el `IP:port` del emisor), `ms` (cuánto tardó) y `match`
(el texto o los bytes encontrados, o el primer grupo de una expresión regular).

Una espera empieza a escuchar cuando empieza la ejecución, no cuando se llega al paso, así que
una respuesta que llega muy rápido no se pierde. Solo toma lo que llegó después
del último envío en su camino.

### Límites y valores predeterminados {#limits}

| Ajuste | Predeterminado | Rango |
| --- | --- | --- |
| Paso TCP: [[ui:common.timeoutMs]] (conectar, escribir y la respuesta juntos) | 4000 ms | 1–120 000 ms |
| [[ui:exp.waitTimeout]] de una espera o una respuesta | 2000 ms | 1–120 000 ms |
| Carga útil UDP | — | como máximo 65 507 bytes |
| Dirección de escucha | — | `IP:port` con un puerto; el [[ui:exp.replyOn]] de una respuesta puede usar el puerto 0 (cualquier puerto libre) |

En el Inspector, el datagrama de un paso UDP aparece con el origen `broadcast`
(o `experiment` cuando el paso espera una respuesta), y cada datagrama que llega
al puerto de una espera con el origen `experiment-wait`. Un paso TCP aparece como dos
tramas `tcp` con el origen `experiment`: la carga útil que escribió y, cuando
llegó una, la respuesta que leyó. Su entrada en la línea de tiempo dice qué envió y cuán grande fue la
respuesta.

## Señales {#signals}

Una señal [[ui:sig.tr.udp]] de la biblioteca es un destino y una carga útil, como texto o
bytes en hex. Envíala desde [[ui:nav.signals]], o con <kbd>Ctrl</kbd>+<kbd>K</kbd>
desde cualquier pantalla. Su destino puede ser un nombre de host.

Cualquier datagrama que el Inspector conservara entero puede convertirse en una: [[ui:sig.fromFrame]] crea una
señal UDP hex, en la carpeta [[ui:sig.capturedFolder]], que reproduce esos bytes exactos
— al destino de la trama si la trama se envió, a la dirección que
la recibió si llegó (en este equipo, cuando esa era todas las direcciones).
Una porción TCP no puede: es un trozo de un flujo. Consulta [Señales](../tools/signals.md) y
[Inspector](../tools/inspector.md#save-as-signal).

Una señal UDP de texto se puede añadir a un experimento como paso [[ui:exp.node.udp]];
una hex no, porque el paso envía texto.

## Desde la línea de comandos {#cli}

`signallab send udp` envía un datagrama:

```bash
signallab send udp 127.0.0.1:9000 --text "PING"
signallab send udp 127.0.0.1:9000 --hex "de ad be ef"
```

```text
✔ sent 4 bytes → 127.0.0.1:9000
```

Indica exactamente uno de `--text` y `--hex`. El destino es `IP:port` o
`host:port` ([arriba](#destinations)). Sale con 0 cuando el datagrama salió, 1 cuando el envío falló,
y 2 cuando el destino o el hex no son válidos. No hay `send tcp`. Consulta [Línea de comandos](../automation/cli.md#cli-send-udp).

## Tormenta {#storm}

[[ui:nav.storm]] es una fuente de carga para tus propios servidores y enlaces: un
[[ui:st.udp]] envía datagramas de un tamaño fijo a una tasa fija, un [[ui:st.tcp]]
abre una conexión, escribe la carga útil y cierra, una y otra vez. El caudal
se mide en vivo. Consulta [Tormenta](../tools/storm.md).

## Escáner {#scanner}

[[ui:nav.scan]] prueba una conexión TCP a cada puerto de un rango y lista los
que aceptan, con lo que dice primero el servicio si pides banners. Consulta
[Escáner](../tools/scanner.md).

::: danger
Tormenta y Escáner envían tráfico real a hosts reales. Apúntalos solo a sistemas
que sean tuyos o que puedas probar: una tormenta puede saturar un enlace, y ambos pueden activar
la detección de intrusiones.
:::

## Dispositivos emulados {#emulators}

En la pantalla [[ui:nav.emulators]], Signal Lab puede ser el dispositivo:

- un [[ui:emu.new.udp]] responde a los datagramas según reglas sobre su carga útil — cualquiera,
  que contenga un texto, que coincida con una expresión regular, que contenga bytes — con una
  respuesta de texto o hex construida a partir de lo que llegó, al emisor o a otro
  `IP:port`, tras un retardo si lo defines;
- un [[ui:emu.new.tcp]] acepta conexiones, parte lo que llega en mensajes
  con un fin de línea que elijas (LF, CR LF, CR, o cada fragmento según llega),
  responde a cada uno con el mismo tipo de reglas, puede enviar un saludo cuando un cliente
  se conecta, y puede cerrar la conexión tras una respuesta.

Ambos pueden funcionar también como paso [[ui:exp.node.emulator]] durante una ejecución.
Consulta [Emuladores](../tools/emulators.md).

## Degradación {#impairment}

El relé de [[ui:nav.netsim]] se sitúa entre un cliente y su destino y degrada
lo que pasa: por datagrama sobre UDP (retardo, pérdida, duplicados, reordenación, un
límite de ancho de banda) o por flujo sobre TCP (retardo, un límite de ancho de banda, conexiones
restablecidas o dejadas semiabiertas). Consulta [Degradación](../tools/impairment.md) y, dentro
de un experimento, [Fallos](../experiments/faults.md).

## "Puerto inalcanzable" en Windows {#port-unreachable}

Cuando un datagrama llega a un puerto donde nada escucha, el equipo receptor
suele responder con un mensaje ICMP "puerto inalcanzable". Windows informa de esa
respuesta en la siguiente recepción del socket emisor, como si la conexión se hubiera
restablecido — aunque UDP no tiene conexión.

Signal Lab cuenta con ello. Sus escuchas — el monitor OSC, la escucha de
descubrimiento, las esperas y respuestas de los experimentos, los emuladores UDP y OSC, el relé
de degradación — lo detectan y siguen escuchando. Un dispositivo que se ha ido no las
detiene. Una escucha que de verdad no puede recibir más termina su tarea, y la
consola dice por qué.

## Problemas {#troubleshooting}

| Lo que ves | Causa habitual |
| --- | --- |
| `… is not a valid address` | El destino no tiene puerto, o no es ni `IP:port` ni `host:port`. |
| `Cannot resolve …` | El nombre de host no resuelve en este equipo. |
| `… refused the connection — nothing is listening on that port` (TCP) | Nada escucha en ese puerto, o un firewall lo rechaza. |
| `No answer from … in time` (TCP) | El host no responde en absoluto — dirección equivocada, o un firewall que descarta en lugar de rechazar. |
| Una espera agota el tiempo aunque el dispositivo responda | El dispositivo responde al puerto desde el que vino el datagrama, no al puerto de la espera. Deja que el envío espere él mismo la respuesta con [[ui:exp.expectReply]]: entonces sale desde el puerto al que vuelve la respuesta. |
| Los datagramas de otros equipos nunca llegan | En Windows, el firewall puede impedirlo: permite Signal Lab cuando la aplicación lo ofrezca. Consulta [Solución de problemas](../reference/troubleshooting.md). |

Cada mensaje de error está en [Mensajes de error](../reference/errors.md#transport).

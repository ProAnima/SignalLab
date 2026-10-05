---
title: OSC
description: Envía mensajes Open Sound Control con tipo, observa lo que llega a un puerto y dirige una onda continua a un dispositivo desde la pantalla OSC.
---

# OSC

La pantalla [[ui:nav.osc]] es donde hablas Open Sound Control (OSC 1.0) por
UDP a mano. Tiene tres partes:

- [[ui:osc.sender]]: un mensaje, argumentos con tipo, enviado cuando pulsas Enter.
- [[ui:osc.monitor]]: escucha en un puerto y decodifica cada paquete que llega.
- [[ui:osc.generator]]: envía un valor que sigue una forma de onda, muchas veces
  por segundo, y lo dibuja.

Signal Lab codifica y decodifica OSC por sí mismo. Lo que envía es lo que el
[Inspector](../tools/inspector.md) muestra, byte a byte.

## Enviar un mensaje {#send}

1. Abre [[ui:nav.osc]].
2. En [[ui:common.target]], escribe la dirección IP o el nombre de host del
   dispositivo y su puerto, por ejemplo `127.0.0.1:9000` o `stage-mixer.local:9000`.
3. En [[ui:common.address]], escribe la dirección que espera el dispositivo, por
   ejemplo `/mixer/fader/1`.
4. En [[ui:common.arguments]], define el tipo y el valor de cada argumento. Pulsa
   [[ui:common.addArgument]] para añadir otro; ✕ quita uno.
5. Pulsa [[ui:common.send]], o <kbd>Enter</kbd> en cualquier campo del emisor.

La línea bajo los botones dice lo que salió: la dirección, su tamaño en bytes
y el destino. Volver a enviar el mismo mensaje suma (×2, ×3…), así que puedes
ver que un envío repetido hizo algo. Un fallo se muestra ahí en su lugar, y
en la consola.

El destino, la dirección y los argumentos se conservan cuando cambias de pantalla y cuando
reinicias la aplicación.

### Campos {#send-fields}

| Campo | Qué | Predeterminado |
| --- | --- | --- |
| [[ui:common.target]] | `IP:port` o `host:port` del receptor. Una dirección IPv6 va entre corchetes: `[::1]:9000`. Un nombre de host se resuelve cada vez que envías; cuando tiene una dirección IPv4, se usa esa (así `localhost` llega a un receptor que escucha en `127.0.0.1`), si no, su IPv6. Un destino sin puerto se rechaza. | `127.0.0.1:9000` |
| [[ui:common.address]] | La dirección OSC, que empieza por `/`, con las partes separadas por `/`. Una sin la `/` inicial se rechaza antes de enviar nada. | `/hello/avatar/1` |
| [[ui:common.arguments]] | Valores con tipo tras la dirección, en orden. Un mensaje puede no tener ninguno. | un `float`, `1.0` |

### Tipos de argumento {#types}

El tipo de cada argumento forma parte del mensaje (su etiqueta de tipo), así que un dispositivo
que espera un float puede ignorar un int con el mismo valor.

| Tipo en la lista | Etiqueta OSC | Valor | Cómo lo escribes |
| --- | --- | --- | --- |
| `int` | `i` | entero con signo de 32 bits | un número entero |
| `float` | `f` | coma flotante de 32 bits | un número, `0.75` |
| `str` | `s` | texto | cualquier texto, enviado como UTF-8 |
| `bool` | `T` / `F` | verdadero o falso | `true` o `false` de una lista; no lleva bytes, solo la etiqueta |
| `long` | `h` | entero con signo de 64 bits | un número entero |
| `double` | `d` | coma flotante de 64 bits | un número |
| `nil` | `N` | nada | ningún valor |
| `blob` | `b` | bytes | no se escribe aquí: aparece, de solo lectura y en hex, cuando abres una señal que tiene uno |

Un campo numérico que no contiene un número envía `0`.

::: tip
Un `true` de OSC es la etiqueta `T`, no el texto `"true"`. Un dispositivo que espera un
bool ignora en silencio una cadena.
:::

## Bundles {#bundles}

El emisor envía mensajes sueltos, no bundles. Cuando llega un bundle (`#bundle`),
el monitor, las esperas de los experimentos y los emuladores lo desempaquetan: cada
mensaje que contiene se trata por su cuenta, y su etiqueta de tiempo se ignora.

## Observar un puerto {#monitor}

Para ver lo que envía un dispositivo o un controlador de espectáculo:

1. En [[ui:common.bind]], escribe la dirección y el puerto donde escuchar. `0.0.0.0:9000`
   (el predeterminado) escucha en todas las tarjetas de red; `127.0.0.1:9000` solo en este
   equipo.
2. Pulsa [[ui:osc.listen]]. El campo se bloquea mientras el monitor funciona.
3. Apunta el emisor a la dirección IP de este equipo y a ese puerto.

Cada paquete se convierte en una fila, el más reciente arriba:

| Columna | Qué |
| --- | --- |
| [[ui:common.time]] | Cuándo llegó, al milisegundo |
| [[ui:osc.from]] | El `IP:port` del emisor |
| [[ui:osc.address]] | La dirección OSC, o [[ui:osc.decodeError]] cuando el paquete no es OSC válido |
| [[ui:osc.args]] | Los valores de los argumentos; un blob se muestra como `blob[n]`, `nil` como `nil`. Para un paquete que no se pudo decodificar, el motivo. |

Un bundle da una fila por mensaje. La lista conserva las 300 filas más recientes;
[[ui:common.clear]] la vacía. Pulsa [[ui:common.stop]] para cerrar el puerto. El
monitor es también una tarea en la franja de la consola, así que se puede detener desde ahí.

El monitor lee paquetes de hasta 64 KiB. Decodifica las etiquetas `i f s S b h d T F N I`
(`S` se lee como texto, `I` como nil); un paquete con cualquier otra etiqueta, o cortado,
se muestra como un error de decodificación en lugar de descartarse.

### Convertir un mensaje en una espera {#wait-for-this}

Cada fila tiene un botón ⇠, [[ui:osc.waitForThis]]. Añade un
paso [Esperar OSC](../experiments/nodes.md#node-wait_osc) al experimento abierto
que escucha en el [[ui:common.bind]] del monitor para esta dirección,
con una regla "es igual a" por cada argumento de texto, entero y verdadero/falso —
los float, blob y nil no tienen ninguna — hasta 16 reglas. Su tiempo de espera es 2000 ms. El
editor se abre con el nuevo paso seleccionado.

::: warning
Detén el monitor antes de ejecutar ese experimento. La ejecución abre el mismo puerto
ella misma, y dos escuchas no pueden compartirlo.
:::

## Dirigir una onda {#generator}

El [[ui:osc.generator]] envía un mensaje tras otro a una dirección, con un
único argumento cuyo valor sigue una forma de onda — un fader, un nivel de luz, una
posición. Úsalo para ver cómo sigue un dispositivo un valor en movimiento, o para cargar un
receptor con un flujo constante.

1. Define [[ui:common.target]] y [[ui:osc.address]]. Ambos se comprueban cuando
   pulsas [[ui:osc.startGen]], como los comprueba el emisor.
2. Elige una [[ui:osc.waveform]], su [[ui:osc.freq]] y la [[ui:osc.rate]].
3. Define [[ui:osc.min]] y [[ui:osc.max]], el rango del valor.
4. Pulsa [[ui:osc.startGen]]. Funciona hasta que pulses [[ui:osc.stopGen]] o detengas
   su tarea en la franja de la consola.

| Campo | Qué | Predeterminado |
| --- | --- | --- |
| [[ui:common.target]] | `IP:port` o `host:port` del receptor; un nombre se resuelve una vez, cuando arranca el generador | `127.0.0.1:9000` |
| [[ui:osc.address]] | La dirección a la que se envía cada mensaje; empieza por `/` | `/hello/lfo` |
| [[ui:osc.waveform]] | La forma del valor a lo largo del tiempo (abajo) | [[ui:wave.sine]] |
| [[ui:osc.freq]] | Ciclos de la forma de onda por segundo | `1` |
| [[ui:osc.rate]] | Mensajes por segundo, de 0,1 a 5000; un valor fuera se ajusta a ese rango | `60` |
| [[ui:osc.min]], [[ui:osc.max]] | El valor más bajo y el más alto. Si Máx. es menor que Mín., el valor no se mueve. | `0`, `1` |
| [[ui:osc.asInt]] | Redondear al número entero más cercano y enviar un `int` en lugar de un `float` | desactivado |

| Forma de onda | Qué hace el valor en cada ciclo |
| --- | --- |
| [[ui:wave.sine]] | Oscila suavemente entre Mín. y Máx., empezando en el medio y subiendo |
| [[ui:wave.triangle]] | Sube de Mín. a Máx., luego baja de nuevo a Mín., empezando en Mín. |
| [[ui:wave.saw]] | Una sierra descendente: empieza en Máx., baja a Mín. y salta de nuevo a Máx. |
| [[ui:wave.ramp]] | Una sierra ascendente: empieza en Mín., sube a Máx. y salta de nuevo a Mín. |
| [[ui:wave.square]] | Máx. durante la primera mitad, Mín. durante la segunda |
| [[ui:wave.random]] | Un valor aleatorio nuevo entre Mín. y Máx. con cada mensaje; la frecuencia no se usa |
| [[ui:wave.constant]] | Máx., siempre; la frecuencia no se usa |

### El osciloscopio {#scope}

Junto a los campos, el osciloscopio dibuja el valor tal como se envía: los últimos 300
puntos, escalados para encajar. Debajo están la forma de onda, la frecuencia y la tasa, y el
último valor enviado. El osciloscopio se actualiza unas 30 veces por segundo por rápido que
envíe el generador, así que a tasas altas muestra una muestra de los mensajes, no cada
uno.

Si un envío falla, el generador se detiene y la consola dice por qué.

## En el Inspector {#inspector}

Con la captura activada ([[ui:ins.arm]] en el [[ui:dock.inspector]]), el tráfico OSC
aparece con el protocolo `osc`:

| Origen | Qué | Notas |
| --- | --- | --- |
| `osc-send` | Cada mensaje que envían el emisor, una señal de la biblioteca, un paso [[ui:exp.node.osc]] o `signallab send osc` | Un paso que espera una respuesta se muestra como `experiment` |
| `osc-monitor` | Cada paquete que recibe el monitor | Un bundle se resume por su primer mensaje y `+n more in bundle`; un paquete mal formado tiene el veredicto `decode error: …` |
| `osc-gen` | Los mensajes del generador | Se captura como máximo uno cada 100 ms, con el veredicto `sampled`; el siguiente después de los mensajes omitidos añade cuántos no se dibujaron, como `sampled · +5 not shown` |

Cada trama conserva los bytes con los que se construyó. Consulta [Inspector](../tools/inspector.md).

## Guardar y reutilizar {#library}

- **Guardar como señal.** [[ui:sig.saveNew]] bajo los botones guarda el mensaje —
  destino, dirección y argumentos — en la biblioteca de señales, en una carpeta que elijas.
  A partir de entonces el emisor queda vinculado a esa señal: [[ui:sig.save]] (o
  <kbd>Ctrl</kbd>+<kbd>S</kbd> en el emisor) la actualiza, [[ui:sig.saveAs]]
  hace una copia, y la etiqueta a su lado la abre en [[ui:nav.signals]]. Envíala
  más tarde desde [[ui:nav.signals]] o con <kbd>Ctrl</kbd>+<kbd>K</kbd> desde cualquier
  pantalla. Consulta [Señales](../tools/signals.md).
- **Añadir a un experimento.** [[ui:common.toExperiment]] añade un
  paso [Mensaje OSC](../experiments/nodes.md#node-osc) con el mismo destino,
  dirección y argumentos al experimento abierto — justo antes de Fin, o después del
  paso seleccionado — y lo abre. Mientras el experimento se ejecuta, no se puede añadir nada;
  la consola lo dice.

## En experimentos {#experiments}

| Paso | Qué hace |
| --- | --- |
| [[ui:exp.node.osc]] | Envía un mensaje. Su destino, dirección y argumentos de texto aceptan `{{templates}}`. Con [[ui:exp.expectReply]] envía desde un puerto propio y espera ahí la respuesta en el mismo paso. [Detalles](../experiments/nodes.md#node-osc) |
| [[ui:exp.node.wait_osc]] | Espera un mensaje cuya dirección coincide con un patrón y cuyos argumentos pasan las reglas. [Detalles](../experiments/nodes.md#node-wait_osc) |
| [[ui:exp.node.emulator]] | Un dispositivo OSC que responde según reglas durante toda la ejecución. Consulta [Emuladores](../tools/emulators.md). |

El paso de mensaje OSC acepta `IP:port` o `host:port` como la pantalla, y su
dirección debe empezar por `/`. Un nombre de host se resuelve cada vez que el paso envía.

### Patrones de dirección {#patterns}

[[ui:exp.node.wait_osc]], la respuesta de [[ui:exp.node.osc]] y las reglas de un
emulador OSC comparan direcciones con patrones OSC 1.0:

| Patrón | Coincide con |
| --- | --- |
| `*` | cualquier serie de caracteres, también ninguno |
| `?` | exactamente un carácter |
| `[0-9]`, `[a-c]` | un carácter del conjunto o rango |
| `[!0-9]` | un carácter que no está en el conjunto |
| `{ping,pong}` | una de las palabras |

Los comodines se quedan dentro de una parte entre barras: `/cue/*` coincide con `/cue/7` pero
no con `/cue/7/go`, y un patrón solo coincide con una dirección con el mismo número de
partes. La comparación distingue mayúsculas de minúsculas. Un patrón empieza por `/`, no tiene ninguna parte vacía
(`//`), ni espacios, ni `#` ni caracteres fuera de ASCII, y tiene como máximo 512
caracteres.

Las reglas de argumentos comparan el argumento número 0–63 con un valor (es igual a, menor que,
contiene, coincide con una expresión regular, etc.); una espera tiene como máximo 16. Cuando
llega un bundle, la espera lo toma si cualquiera de sus mensajes coincide. Consulta
[Datos y plantillas](../experiments/data.md) para saber qué da un mensaje coincidente a
los pasos siguientes.

## Desde la línea de comandos {#cli}

`signallab send osc` envía un mensaje como lo hace el emisor:

```bash
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main
```

```text
✔ sent /cue/go (24 bytes) → 127.0.0.1:9000
```

Cada argumento es `tag:value`: `i:3`, `f:0.5`, `d:1.5`, `h:64`, `s:text`,
`b:de ad be ef` (bytes en hex), o `T`, `F`, `N` solos. Sin etiqueta, un
número entero es `i`, un número con punto decimal es `f`, y cualquier otra cosa es
`s`; escribe `s:7` para enviar el texto `7`. El destino es `IP:port` o `host:port`. Sale
con 0 cuando el mensaje salió, 1 cuando el envío falló (un nombre de host que
no resuelve incluido), y 2 cuando un argumento, la dirección o el destino
no son válidos. [`signallab fire`](../automation/cli.md#cli-fire) envía una señal guardada. Consulta
[Línea de comandos](../automation/cli.md#cli-send-osc).

::: tip
En Git Bash en Windows, un argumento que empieza por `/` se convierte en una ruta de
archivo antes de que `signallab` lo vea. Ejecuta el comando con `MSYS_NO_PATHCONV=1` delante,
o usa PowerShell o `cmd`.
:::

## Problemas {#troubleshooting}

| Lo que ves | Causa habitual |
| --- | --- |
| `… is not a valid address` al enviar | El destino no tiene puerto, o no es ni `IP:port` ni `host:port`. |
| `Cannot resolve …` al enviar | El nombre de host no resuelve en este equipo. Compruébalo, o usa la dirección IP. |
| `OSC addresses start with / (…)` | La dirección no tiene la `/` inicial. |
| El mensaje se envía pero el dispositivo no hace nada | Puerto o dirección equivocados; un tipo distinto del que espera (`int` en lugar de `float`, un texto `"true"` en lugar de un bool). Obsérvalo en el Inspector, o apunta el destino al monitor de este equipo para ver qué sale. |
| `… is already in use by another program` en [[ui:osc.listen]] | Otro programa — o un experimento, emulador o un segundo monitor en marcha — tiene el puerto. |
| `… is not an address of this computer` | La IP de [[ui:common.bind]] pertenece a otro equipo. Usa `0.0.0.0` o una de las direcciones de este equipo. |
| Los paquetes de otros equipos nunca llegan | En Windows, el firewall puede impedirlo: permite Signal Lab cuando la aplicación lo ofrezca. El tráfico local (`127.0.0.1`) no se ve afectado. Consulta [Solución de problemas](../reference/troubleshooting.md). |
| Filas [[ui:osc.decodeError]] | El emisor no habla OSC 1.0 en ese puerto, o usa una etiqueta de tipo que Signal Lab no decodifica. |

En un servidor, la pantalla funciona en la red del servidor: `127.0.0.1` es el
servidor mismo, y el monitor escucha en los puertos del servidor. Consulta
[Servidor](../server/index.md).

Cada mensaje de error está en [Mensajes de error](../reference/errors.md#transport).

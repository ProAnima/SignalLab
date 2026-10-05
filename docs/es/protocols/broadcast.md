---
title: Difusión
description: Envía una carga útil a una lista de hosts, una dirección de difusión, un grupo de multidifusión o cada host de una subred, una vez o como baliza, y escucha quién responde con la escucha de descubrimiento.
---

# Difusión

La pantalla [[ui:nav.broadcast]] ([[ui:bc.title]]) envía una carga útil UDP a muchos
destinos a la vez, y escucha al otro lado para ver quién responde. Úsala para encontrar
dispositivos en una red, para comprobar que un flujo de multidifusión llega a un receptor,
o para simular un dispositivo que responde a sondeos de descubrimiento.

- La [[ui:bc.emitter]] envía a una lista de hosts, una dirección de difusión, un
  grupo de multidifusión o cada host de una subred — una vez, o una y otra vez como
  baliza.
- La [[ui:bc.discovery]] escucha en un puerto, se une a grupos de multidifusión, enumera cada
  par que habla con ella y puede responder a los sondeos.

::: danger
La difusión, la multidifusión y un barrido llegan a todos los dispositivos del segmento de
red, no solo al que tienes en mente, y una baliza sigue haciéndolo. Comprueba primero en
qué red estás, y envía solo a redes que sean tuyas o que tengas permiso para probar. Los
límites de abajo son barandillas de protección, no permiso.
:::

## Enviar {#send}

1. Elige el [[ui:bc.mode]] (más abajo).
2. Escribe el destino: el nombre del campo cambia con el modo. En una red
   con una dirección IPv4, [[ui:bc.useSubnet]] lo rellena a partir de la dirección de esta
   máquina.
3. Elige la [[ui:bc.payload]] y escríbela.
4. Pulsa [[ui:bc.sendOnce]]: va un datagrama a cada destino.

### Modos {#modes}

| [[ui:bc.mode]] | Destino | Qué pasa | Predeterminado |
| --- | --- | --- | --- |
| [[ui:bc.mode.list]] | [[ui:bc.targetList]]: entradas `IP:port` o `host:port` separadas por comas, puntos y coma o líneas nuevas — un espacio no las separa. Un nombre se resuelve, y se toma su dirección IPv4 cuando tiene una. | Un datagrama a cada uno | `127.0.0.1:9000, 127.0.0.1:9001` |
| [[ui:bc.mode.broadcast]] | [[ui:bc.targetAddress]]: `255.255.255.255:port`, o una dirección terminada en `.255` | Un datagrama que recibe cada host de la red local. Los enrutadores no lo reenvían. | `255.255.255.255:9000` |
| [[ui:bc.mode.multicast]] | [[ui:bc.targetAddress]]: un grupo de `224.0.0.0` a `239.255.255.255`, con un puerto | Un datagrama al grupo; solo lo reciben los receptores que se han unido a él | `239.1.1.1:9000` |
| [[ui:bc.mode.sweep]] | [[ui:bc.targetCidr]], `a.b.c.d/nn`, y un [[ui:bc.port]] | Un datagrama a cada host utilizable del bloque, como unidifusión — para dispositivos que ignoran la difusión | `192.168.1.0/24`, puerto 9000 |

En un barrido se omiten las direcciones de red y de difusión (salvo en un `/31` o `/32`), y
una base que no es la de la propia red se redondea hacia abajo hasta ella:
`192.0.2.77/30` barre `192.0.2.77` y `192.0.2.78`. Un barrido llega a 1024 hosts como
máximo, así que el bloque más ancho es un `/22` (1022 hosts); uno más ancho se rechaza con
el prefijo al que reducirlo.

La difusión, los grupos de multidifusión, los barridos y las uniones de la escucha de
descubrimiento son solo IPv4: IPv6 no tiene difusión. Una [[ui:bc.mode.list]] también puede
nombrar hosts IPv6 (consulta [Opciones de socket](#socket-options)).

[[ui:bc.useSubnet]] rellena `x.y.z.10:9000, x.y.z.11:9000` en [[ui:bc.mode.list]],
`x.y.z.255:9000` en [[ui:bc.mode.broadcast]] y `x.y.z.0/24` en
[[ui:bc.mode.sweep]], a partir de la dirección de esta máquina `x.y.z.w`. Supone una red
`/24`.

### Carga útil {#payload}

| [[ui:bc.payload]] | Qué se envía |
| --- | --- |
| [[ui:bc.payload.osc]] | Un mensaje OSC: [[ui:common.address]] y [[ui:common.arguments]] con tipo, como en la pantalla [OSC](osc.md). De forma predeterminada `/hello/discover` con el texto `who-is-there`. |
| [[ui:bc.payload.text]] | El [[ui:bc.text]] como UTF-8, exactamente como se escribió, sin terminador. Predeterminado `HELLO-PROBE`. |
| [[ui:bc.payload.hex]] | El [[ui:bc.hex]] byte a byte — para reproducir una trama capturada o hablar un protocolo de descubrimiento binario. Pares de dígitos hexadecimales; cualquier otra cosa entre ellos se ignora. Predeterminado `48 45 4c 4c 4f`. |

### Opciones de socket {#socket-options}

[[ui:bc.socketOptions]] abre tres ajustes más:

| Opción | Qué | Predeterminado |
| --- | --- | --- |
| [[ui:bc.bindSource]] | El `IP:port` local desde el que salen los datagramas. Fíjalo para elegir la tarjeta de red, o un puerto de origen al que responde un dispositivo. `0.0.0.0:0`: cualquiera. | `0.0.0.0:0` |
| [[ui:bc.ttl]] | Cuántos enrutadores puede cruzar un datagrama, 1–255. Para multidifusión es el límite de saltos de multidifusión: 1 lo mantiene en esta red. | `1` |
| [[ui:bc.mcastLoop]] | Solo multidifusión: entregar también los datagramas del grupo a esta máquina, para que un receptor de aquí los oiga | activado |

Con el [[ui:bc.bindSource]] predeterminado, los datagramas salen de un socket IPv4, o de
uno IPv6 cuando todos los destinos son IPv6. Una lista que mezcla los dos se envía desde el
socket IPv4, y sus destinos IPv6 fallan; envíalos como una lista propia, o fija
[[ui:bc.bindSource]] a una dirección IPv6.

### El resultado {#result}

[[ui:bc.lastEmit]] muestra lo que salió: [[ui:bc.targets]], [[ui:common.packets]],
[[ui:common.volume]] y [[ui:common.errors]], y los primeros ocho destinos
que alcanzó ("… +n más" para el resto). Un error a un destino no detiene
a los demás; la línea de la consola indica cuántos fallaron.

## Repetir como baliza {#beacon}

Una baliza envía la misma ronda — un datagrama a cada destino — según un programa, hasta
que la detienes. Los dispositivos que esperan un anuncio periódico necesitan una.

1. Configura el modo, el destino y la carga útil como para un envío único.
2. En [[ui:bc.beacon]], define [[ui:bc.beaconRate]], y si debe detenerse por
   sí sola, [[ui:bc.beaconRounds]] o [[ui:bc.beaconSeconds]].
3. Pulsa [[ui:bc.startBeacon]]. [[ui:bc.stopBeacon]] — o detener su tarea en la
   franja de la consola — la termina.

| Campo | Qué | Predeterminado |
| --- | --- | --- |
| [[ui:bc.beaconRate]] | Rondas por segundo; debe ser mayor que 0 | `2` |
| [[ui:bc.beaconRounds]] | Detenerse tras estas rondas; 0 — sin límite | `0` |
| [[ui:bc.beaconSeconds]] | Detenerse tras estos segundos; 0 — sin límite | `0` |

La tasa multiplicada por el número de destinos puede ser como máximo 50 000 datagramas por
segundo. Un barrido de un `/24` (254 hosts) puede, por tanto, repetirse unas 196 veces por
segundo como máximo. Mientras la baliza funciona, [[ui:bc.lastEmit]] muestra
[[ui:bc.targets]] (los destinos de cada ronda, desde el primer informe),
los totales, [[ui:bc.rounds]] y [[ui:bc.pps]] (datagramas por segundo), actualizados
cuatro veces por segundo, y [[ui:bc.sendOnce]] no está disponible. Una baliza que ha tenido
más de 32 datagramas fallidos y ni uno enviado — sin ruta, difusión no permitida — se
detiene sola y dice por qué.

## Escuchar dispositivos {#discovery}

La [[ui:bc.discovery]] enlaza un puerto UDP y registra cada par que le envía algo: lo que
responde a un sondeo, o lo que un dispositivo anuncia por su cuenta.

1. Define [[ui:common.bind]], el puerto al que envían los dispositivos.
2. Para multidifusión, enumera los grupos en [[ui:bc.joinGroups]].
3. Pulsa [[ui:bc.startListen]]. Los ajustes se bloquean hasta que pulses
   [[ui:bc.stopListen]].

| Campo | Qué | Predeterminado |
| --- | --- | --- |
| [[ui:common.bind]] | Dónde escuchar, `IP:port`. `0.0.0.0` escucha en todas las tarjetas de red. | `0.0.0.0:9000` |
| [[ui:bc.joinGroups]] | Grupos de multidifusión IPv4 a los que unirse, separados por comas; vacío — solo unidifusión y difusión | `239.1.1.1` |
| [[ui:bc.interface]] | La dirección IPv4 de la tarjeta de red en la que unirse a los grupos; vacío — el sistema elige | vacío |
| [[ui:bc.reuse]] | Escuchar en un puerto que otro programa también usa (`SO_REUSEADDR`). Funciona solo si ese programa permite compartirlo. | activado |
| [[ui:bc.respond]] | Responder a los sondeos, como haría un dispositivo ([más abajo](#auto-reply)) | desactivado |

### Pares {#peers}

[[ui:bc.peers]] enumera quién ha enviado algo, el más reciente primero, con el número de
pares vistos, los paquetes oídos y las respuestas enviadas por encima:

| Columna | Qué |
| --- | --- |
| [[ui:bc.peer]] | El `IP:port` del emisor; su punto muestra si se le oyó en los últimos 3 segundos |
| [[ui:bc.proto]] | `osc` cuando su último datagrama se decodificó como OSC, si no `udp` |
| [[ui:common.packets]] | Cuántos envió |
| [[ui:bc.age]] | Segundos desde su último datagrama |
| [[ui:bc.lastMessage]] | Su último datagrama: la dirección y los argumentos OSC, o el inicio del texto |

La lista se actualiza unas cuantas veces por segundo y guarda hasta 512 pares; más allá, los
paquetes se siguen contando pero los pares nuevos no obtienen fila.

### Responder a los sondeos {#auto-reply}

Con [[ui:bc.respond]], la escucha interpreta un dispositivo: responde a cada datagrama que
recibe, desde el puerto de escucha, a la dirección y el puerto del emisor.

| Campo | Qué | Predeterminado |
| --- | --- | --- |
| [[ui:bc.payload]] | La respuesta: OSC, texto o hex, como para enviar | OSC `/hello/here` con el texto `signal-lab` |
| [[ui:bc.replyDelay]] | Esperar este tiempo antes de responder, como haría un dispositivo lento | `0` |
| [[ui:bc.matchContains]] | Responder solo a los datagramas cuyo texto decodificado contenga esto — la dirección y los argumentos OSC, o el inicio del texto; vacío — todos | vacío |

La escucha nunca responde a un datagrama idéntico a su propia respuesta, así que dos
escuchas apuntadas una a otra no se responden sin parar.

### Firewalls y puertos compartidos {#firewall}

El tráfico de difusión y multidifusión de otras máquinas lo bloquean de forma
predeterminada la mayoría de los firewalls de Windows: permite Signal Lab en redes privadas
cuando la aplicación lo ofrezca. La difusión nunca cruza un enrutador. Para escuchar en un
puerto que ya tiene el servicio real, ambos lados deben permitir compartirlo
([[ui:bc.reuse]] aquí); sin eso, un puerto ocupado se rechaza con una sugerencia para
activarlo. Consulta [Solución de problemas](../reference/troubleshooting.md).

## En el Inspector {#inspector}

Con la captura activada, el tráfico de la pantalla aparece con el protocolo `osc` o `udp`,
según su carga útil:

| Origen | Qué | Cuántos |
| --- | --- | --- |
| `broadcast` | [[ui:bc.sendOnce]]: cada datagrama, con el veredicto `fan-out`, `broadcast`, `multicast` o `sweep`; uno fallido con `error: …` | todos |
| `beacon` | Las rondas de una baliza | como máximo una ronda cada 50 ms |
| `discovery` | Datagramas que recibe la escucha | como máximo uno cada 40 ms |
| `discovery` | Sus respuestas, con el veredicto `auto-reply` | todas |

Lo que el Inspector deja fuera de una baliza o de la escucha se cuenta: la siguiente trama
que dibuja lleva el número en su veredicto, `+n not shown` (para una baliza, una trama por
cada destino de cada ronda omitida). Consulta
[Inspector](../tools/inspector.md).

## En un servidor o en Docker {#server}

En un [servidor](../server/index.md), la pantalla envía y escucha en la red del servidor.
En Docker, la difusión, la multidifusión y el descubrimiento llegan a la red local solo
cuando el contenedor usa la red del host (`--network host`) en un host Linux. Con la red
puente predeterminada de Docker, o Docker Desktop, solo funciona la unidifusión a hosts que
el contenedor puede alcanzar.

## En otros lugares {#elsewhere}

- [`signallab send udp`](../automation/cli.md#cli-send-udp) envía un datagrama a un host; no
  hay difusión, multidifusión ni barrido desde la línea de comandos.
- Un paso [[ui:exp.node.udp]] envía un datagrama de texto a uno o más hosts, y un
  paso [[ui:exp.node.wait_udp]] espera uno. Consulta [UDP y TCP](udp-tcp.md).
- Para interpretar un dispositivo que responde según reglas — varias reglas, respuestas
  construidas a partir de lo que llegó — usa un emulador [[ui:emu.new.udp]] o
  [[ui:emu.new.osc]]. Consulta [Emuladores](../tools/emulators.md).

## Problemas {#troubleshooting}

| Qué ves | Causa habitual |
| --- | --- |
| `… is not a broadcast address` | [[ui:bc.mode.broadcast]] acepta `255.255.255.255:port` o una dirección terminada en `.255`. Para otra máscara de subred, usa [[ui:bc.mode.sweep]]. |
| `… is not a multicast group` | La dirección está fuera de `224.0.0.0`–`239.255.255.255`. |
| `… spans … addresses, and a sweep reaches at most 1024 hosts` | El bloque es más ancho que un `/22`; redúcelo. |
| `Set the port to sweep` | [[ui:bc.port]] es 0. |
| `… is over the … pps limit` | La tasa multiplicada por el número de destinos supera 50 000 por segundo: baja [[ui:bc.beaconRate]], o reduce el destino. |
| `… is already in use — turn on “share the port” to listen alongside it` | Otro programa tiene el puerto; marca [[ui:bc.reuse]]. |
| `Cannot join the multicast group …` | El grupo o [[ui:bc.interface]] no se pueden usar en esta máquina — ninguna tarjeta de red con esa dirección, o ninguna ruta de multidifusión. |
| Se envió, pero nadie responde | Los dispositivos escuchan en otro puerto; el firewall de aquí deja fuera sus respuestas; hay un enrutador entre vosotros; o, en Docker, el contenedor no está en la red del host. |
| Salen los sondeos, pero la escucha de descubrimiento no oye respuestas | Muchos dispositivos responden a la dirección y el puerto desde los que salió un sondeo — el propio socket del emisor, que la pantalla no lee. Escucha en el puerto al que responden los dispositivos, o envía el sondeo desde un experimento: un paso [[ui:exp.node.udp]] con [[ui:exp.expectReply]] envía y escucha en el mismo puerto. |

Cada mensaje de error se enumera en [Mensajes de error](../reference/errors.md#broadcast).

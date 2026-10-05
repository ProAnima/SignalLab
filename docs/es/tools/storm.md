---
title: Tormenta
description: Envía una inundación controlada de datagramas UDP o conexiones TCP a un servidor que sea tuyo, y observa el ritmo, el rendimiento y los errores en vivo.
---

# Tormenta

La pantalla [[ui:nav.storm]] es una fuente de carga: envía datagramas UDP, o abre
conexiones TCP, contra un destino tan rápido como pidas, durante tanto tiempo
como pidas, y mide lo que realmente salió. Úsala para ver cómo aguanta tu propio
servidor, dispositivo o enlace bajo una inundación: si sigue respondiendo, si
descarta paquetes o si se cae.

::: danger Uso responsable
Una tormenta envía tráfico real a un host real. Apúntala solo a hosts y redes que
sean tuyos o que estés autorizado a probar. Los ritmos altos pueden saturar los
enlaces para todos los que los usan y disparar la detección de intrusiones. El
motor no limita el ritmo de una tormenta: 0 significa tan rápido como este equipo
pueda enviar.
:::

## Iniciar una tormenta {#start}

1. Define [[ui:common.target]]: el `IP:port` o `host:port` al que enviar, como
   `127.0.0.1:9000` o `test-rig.local:9000`. Un nombre de host se resuelve cuando
   la lanzas, y se toma su dirección IPv4 cuando la tiene.
2. Elige el [[ui:common.protocol]]: [[ui:st.udp]] o [[ui:st.tcp]].
3. Define [[ui:st.payloadSize]], [[ui:st.rate]] y [[ui:st.duration]].
4. Pulsa [[ui:st.launch]].

La tormenta funciona como una tarea: aparece en la franja de la consola, y se
detiene cuando termina su duración, cuando pulsas [[ui:st.stop]] o con
[[ui:app.stopAll]]. Los campos quedan fijos mientras funciona.

| Campo | Qué | Predeterminado |
| --- | --- | --- |
| [[ui:common.target]] | `IP:port` o `host:port` del destino. | `127.0.0.1:9000` |
| [[ui:common.protocol]] | [[ui:st.udp]]: datagramas separados. [[ui:st.tcp]]: una conexión TCP nueva por cada «paquete». | [[ui:st.udp]] |
| [[ui:st.payloadSize]] | Bytes en cada datagrama o conexión, 1–65 507. Un valor mayor o menor se lleva a ese intervalo. | 512 |
| [[ui:st.rate]] | Paquetes (o conexiones) por segundo a los que aspirar. 0: tan rápido como sea posible. | 1000 |
| [[ui:st.duration]] | Segundos que dura. 0: hasta que la detengas. | 10 |

### Inundación UDP {#udp}

Signal Lab envía datagramas del tamaño de la carga útil, con cada byte `0x55`,
desde un puerto propio al destino. Un envío que el sistema rechaza cuenta como
error, por ejemplo después de que el destino respondiera que no hay nada
escuchando en ese puerto.

### Inundación de conexiones TCP {#tcp}

Por cada «paquete», Signal Lab abre una conexión TCP al destino, escribe la carga
útil y la cierra. Las conexiones se hacen una tras otra, no a la vez, así que el
ritmo que alcanza una tormenta TCP está limitado por la rapidez con la que el
destino las acepta. Una conexión que se rechaza, o que no se acepta en 500 ms,
cuenta como error; una conexión que aceptó la carga útil cuenta como paquete.

### Cómo se mantiene el ritmo {#pacing}

El ritmo es un calendario. El primer paquete sale de inmediato, y el paquete *n*
vence *n* ÷ [[ui:st.rate]] segundos después del inicio. Cada vez que la tormenta
se despierta, envía lo que vence, y luego duerme hasta que venza el siguiente
paquete. Así, 50 por segundo son 50 por segundo y 250 son 250 —una ejecución de un
segundo envía más o menos esa cantidad—, sea cual sea la granularidad del
temporizador del sistema. Una [[ui:st.duration]] termina la tormenta a tiempo
incluso cuando el siguiente paquete vencería más tarde.

Una tormenta nunca envía por encima de su ritmo para recuperar el tiempo perdido.
Si este equipo, o un destino TCP que acepta despacio, se retrasa más de 256
paquetes, los más antiguos se descartan del calendario en lugar de enviarse tarde
en una ráfaga. [[ui:st.pps]] muestra lo que se alcanzó.

Con [[ui:st.rate]] 0 no hay calendario: la tormenta envía 256 paquetes, deja que
se ejecute otro trabajo y envía los siguientes 256, tan rápido como este equipo
pueda.

## Leer el rendimiento {#metrics}

[[ui:st.throughput]] se actualiza cuatro veces por segundo:

| Métrica | Qué |
| --- | --- |
| [[ui:common.packets]] | Datagramas enviados, o conexiones que aceptaron la carga útil, desde el inicio. |
| [[ui:st.pps]] | Paquetes por segundo en el último cuarto de segundo. |
| [[ui:st.rateLabel]] | Megabits por segundo de carga útil en el último cuarto de segundo. Los encabezados (IP, UDP, TCP) no se cuentan. |
| [[ui:common.volume]] | Bytes de carga útil enviados desde el inicio. |
| [[ui:common.errors]] | Envíos o conexiones que fallaron. |

El gráfico que hay debajo representa [[ui:st.pps]] durante el último minuto.

Cuando termina la tormenta, los contadores muestran los totales finales, y
[[ui:st.pps]] y [[ui:st.rateLabel]] pasan a 0.

Lo que te dicen los números:

- [[ui:st.pps]] muy por debajo de [[ui:st.rate]] en UDP: este equipo no puede
  enviar más rápido. Baja el ritmo o el tamaño de la carga útil.
- [[ui:common.errors]] subiendo en UDP: el puerto del destino está cerrado, o la
  red rechaza el tráfico.
- [[ui:common.errors]] subiendo en TCP: el destino rechaza conexiones o tarda más
  de 500 ms en aceptarlas; puede que haya alcanzado su límite.

Una tormenta dice cuánto salió, no cuánto llegó. Para ver lo que recibió el
destino, obsérvalo: sus propios registros, un monitor [[ui:nav.osc]] o un
[emulador](emulators.md) en su lugar.

## En el Inspector {#inspector}

Con la captura activada, una tormenta UDP pone uno de sus datagramas en el
[Inspector](inspector.md) cada segundo, marcado `sampled 1/s`: son todos iguales.
El veredicto también dice cuántos se dejaron fuera desde el anterior, como
`sampled 1/s · +999 not shown`. Una tormenta TCP no pone ninguno.

## Véase también {#related}

- [Degradación](impairment.md): un enlace lento o con pérdidas en lugar de una
  inundación.
- [HTTP](../protocols/http.md): ráfagas de carga de solicitudes HTTP, con
  latencias.
- [Carga](../experiments/load.md): carga HTTP en un experimento, con umbrales.

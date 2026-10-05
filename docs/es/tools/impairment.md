---
title: Degradación
description: Pon un relé entre un cliente y su destino que retrasa, descarta, duplica, reordena o limita datagramas UDP, o retrasa, limita, restablece o deja a medias flujos TCP.
---

# Degradación

La pantalla [[ui:nav.netsim]] ejecuta un relé que se sitúa entre un cliente y el
destino con el que habla, y degrada lo que pasa por él en ambos sentidos: un
Wi-Fi malo, un enlace móvil, un salto por satélite, un enlace que se cae. Úsalo
para ver cómo se comportan tu cliente y tu dispositivo en una red que no tienes a
mano.

Apunta el cliente al relé en lugar del destino real; el relé reenvía todo al
destino, y las respuestas de vuelta al cliente, después de hacer con cada
datagrama o flujo lo que dice su perfil. Puedes cambiar el perfil mientras
funciona, sin soltar el puerto.

## Iniciar un relé {#start}

1. Elige el [[ui:ns.protocol]]: UDP para datagramas (OSC, la mayor parte del
   tráfico de control de espectáculos y de sensores), TCP para flujos (HTTP,
   MQTT, un dispositivo TCP).
2. Define [[ui:ns.listen]]: el `IP:port` en el que escucha el relé.
   `0.0.0.0:9010` acepta tráfico de la red; `127.0.0.1:9010`, solo de este
   equipo.
3. Define [[ui:ns.target]]: el `IP:port` del destino real, o un nombre de host y
   un puerto como `device.local:9000`.
4. Elige un [preajuste](#presets) o define los valores del [perfil](#profile).
5. Pulsa [[ui:ns.startRelay]].
6. Apunta tu cliente al puerto del relé en lugar del destino, por ejemplo
   `127.0.0.1:9010` en vez de `127.0.0.1:9000`.

[[ui:ns.stopRelay]] cierra el relé; nada de lo que aún retenía sale después de
eso.

Valores predeterminados: UDP, escucha en `0.0.0.0:9010`, destino
`127.0.0.1:9000`, 40 ms de latencia, 15 ms de jitter y 2 % de pérdida.

[[ui:ns.listen]] es una dirección a la que enlazarse, siempre `IP:port`. Un
nombre de host en [[ui:ns.target]] se resuelve una vez, cuando pulsas
[[ui:ns.startRelay]], igual que un destino [OSC](../protocols/osc.md); un nombre
que no se encuentra se rechaza y no arranca ningún relé. [[ui:ns.protocol]],
[[ui:ns.listen]] y [[ui:ns.target]] quedan fijos mientras el relé funciona;
detenlo para cambiarlos.

```text
client ──► relay (0.0.0.0:9010) ──► target (127.0.0.1:9000)
client ◄── relay ◄───────────────── target
```

### UDP {#udp}

El relé envía cada datagrama al destino desde un puerto propio, y envía lo que
responde el destino de vuelta al cliente. Cada datagrama corre su propia suerte,
decidida cuando llega.

Las respuestas van al cliente que envió el datagrama más reciente: el relé
atiende a un cliente a la vez.

### TCP {#tcp}

Cada conexión que un cliente hace al relé se une a una conexión nueva del propio
relé hacia el destino. Ambos flujos de cada conexión —del cliente al destino y
del destino al cliente— se degradan fragmento a fragmento, a medida que el relé
los lee (hasta 16 KiB a la vez). Un flujo siempre llega en orden, sea cual sea el
jitter.

Cuando el destino rechaza la conexión, la conexión del cliente se restablece.

## El perfil {#profile}

Los valores que aplica un relé, en ambos sentidos. Un relé UDP lee los valores
del datagrama, uno TCP los del flujo; los demás se ocultan.

### Sobre UDP {#profile-udp}

| Valor | Qué hace | Intervalo |
| --- | --- | --- |
| [[ui:ns.offline]] | Cada datagrama se descarta, en ambos sentidos, hasta que lo desmarques. | activado / desactivado |
| [[ui:ns.latency]] | Se añade a cada datagrama. | 0–1000 ms |
| [[ui:ns.jitter]] | Un retardo extra aleatorio, de 0 a este, para cada datagrama, así que los datagramas pueden llegar desordenados. | 0–500 ms |
| [[ui:ns.loss]] | La proporción de datagramas que nunca llegan, cada uno por su cuenta. | 0–100 % |
| [[ui:ns.burst]] | La probabilidad de que un datagrama inicie una ráfaga de pérdidas: un enlace que se cae un momento, a diferencia de una pérdida dispersa. | 0–20 % |
| [[ui:ns.burstLength]] | Cuántos datagramas pierde una ráfaga de media. Se muestra cuando [[ui:ns.burst]] es mayor que 0; 5 cuando lo subes por primera vez. | 1–1000 |
| [[ui:ns.duplicate]] | La proporción de datagramas entregados dos veces. | 0–100 % |
| [[ui:ns.corrupt]] | La proporción de datagramas con un bit de un byte invertido. | 0–100 % |
| [[ui:ns.reorder]] | La proporción de datagramas retenidos —por la latencia, y al menos 20 ms— para que los que van detrás lleguen antes. | 0–100 % |
| [[ui:ns.rate]] | Un límite de ancho de banda, en kilobits por segundo; 0 es ninguno. Los datagramas se ponen en cola para el enlace a este ritmo; uno que esperaría más de un segundo se descarta por limitación. | 0, u 8–10 000 000 |

Una ráfaga funciona así: un datagrama que no está en una ráfaga inicia una con
la probabilidad [[ui:ns.burst]]; todos los datagramas de una ráfaga se pierden, y
cada uno la termina con una probabilidad de 1 entre [[ui:ns.burstLength]], así que
una ráfaga dura eso de media.

Un datagrama se decide en este orden: sin conexión, ráfaga, pérdida, ancho de
banda y luego duplicación, corrupción, retardo y reordenación para cada copia.

### Sobre TCP {#profile-tcp}

| Valor | Qué hace | Intervalo |
| --- | --- | --- |
| [[ui:ns.offline]] | No fluye nada, en ningún sentido, y las conexiones nuevas esperan al destino, hasta que lo desmarques; entonces todo continúa. | activado / desactivado |
| [[ui:ns.latency]] | Se añade a cada fragmento de un flujo, en ambos sentidos. | 0–1000 ms |
| [[ui:ns.jitter]] | Un retardo extra aleatorio, de 0 a este, para cada fragmento, nunca por delante del fragmento anterior. | 0–500 ms |
| [[ui:ns.reset]] | La proporción de fragmentos que restablecen su conexión en lugar de pasar: tanto el cliente como el destino reciben un restablecimiento. | 0–100 % |
| [[ui:ns.stall]] | La proporción de fragmentos que dejan su conexión semiabierta: no pasa nada más, en ningún sentido, y a ninguno de los dos lados se le dice. El relé mantiene ambos lados abiertos, intactos, hasta que se detiene. | 0–100 % |
| [[ui:ns.rate]] | Un límite de ancho de banda para cada flujo de cada conexión, en kilobits por segundo; 0 es ninguno. Pasado un segundo de cola, el relé deja de leer, así que el emisor se ralentiza, como en un enlace lento. No se descarta nada. | 0, u 8–10 000 000 |

La pérdida, las ráfagas, la duplicación, la corrupción y la reordenación no se
aplican a TCP: un flujo TCP real retransmite lo que pierde y se pone en orden, así
que lo que un cliente encuentra en un enlace malo es retardo, un emisor lento,
restablecimientos y conexiones que dejan de responder.

::: tip
Los deslizadores llegan hasta 1000 ms de latencia y 500 ms de jitter. El relé en
sí admite hasta 60 000 ms de cada uno, por ejemplo desde un archivo de
experimento.
:::

## Preajustes {#presets}

Un preajuste define todos los valores con un clic. Su etiqueta permanece
encendida mientras los valores siguen siendo los del preajuste; cambia cualquier
valor y se apaga.

| Preajuste | Sobre UDP | Sobre TCP |
| --- | --- | --- |
| [[ui:ns.preset.lan]] | 1 ms ±1 | 1 ms ±1 |
| [[ui:ns.preset.wifi]] | 20 ms ±30, 1 % de pérdida, ráfagas 1 % × 3, 0,5 % duplicados, 2 % reordenados | 20 ms ±30 |
| [[ui:ns.preset.4g]] | 60 ms ±25, 0,5 % de pérdida, 0,5 % reordenados, 20 000 kbit/s | 60 ms ±25, 20 000 kbit/s |
| [[ui:ns.preset.satellite]] | 300 ms ±30, 1 % de pérdida, 2000 kbit/s | 300 ms ±30, 2000 kbit/s |
| [[ui:ns.preset.intermittent]] | 30 ms ±20, ráfagas 3 % × 15 | 30 ms ±20, 0,2 % de los fragmentos quedan semiabiertos |
| [[ui:ns.preset.offline]] | No pasa nada | No fluye nada |

## Cambiarlo mientras funciona {#live}

Cambia cualquier valor, o elige otro preajuste, mientras el relé funciona: se
aplica un cuarto de segundo después de tu último cambio, sin soltar el puerto ni
las conexiones. La consola anota cada perfil nuevo, y la línea bajo
[[ui:ns.live]] dice qué hace ahora el relé: el nombre del preajuste, o los valores
en resumen, como `60 ms ±25 · loss 2% · 20000 kbps`.

Un datagrama o fragmento se decide con el perfil vigente cuando llega; uno que ya
va de camino conserva su retardo.

## Los contadores {#counters}

[[ui:ns.live]] muestra lo que ha hecho el relé desde que arrancó, los dos sentidos
juntos, actualizado cuatro veces por segundo.

Sobre UDP:

| Contador | Qué |
| --- | --- |
| [[ui:ns.received]] | Datagramas que llegaron al relé. |
| [[ui:ns.forwarded]] | Datagramas reenviados; uno duplicado cuenta dos veces. |
| [[ui:ns.dropped]] | Perdidos a propósito: sin conexión, una ráfaga o pérdida. |
| [[ui:ns.throttled]] | Descartados por el límite de ancho de banda, o porque ya iban 10 000 de camino. |
| [[ui:ns.duplicated]] | Datagramas enviados dos veces. |
| [[ui:ns.corrupted]] | Copias con un bit invertido. |
| [[ui:ns.reordered]] | Copias retenidas para que otras posteriores las adelantaran. |
| [[ui:common.volume]] | Bytes reenviados. |

Sobre TCP:

| Contador | Qué |
| --- | --- |
| [[ui:ns.connections]] | Conexiones que los clientes hicieron al relé. |
| [[ui:ns.received]] | Fragmentos que leyó el relé, en ambos sentidos. |
| [[ui:ns.forwarded]] | Fragmentos que reenvió. |
| [[ui:ns.resets]] | Conexiones restablecidas. |
| [[ui:ns.stalled]] | Conexiones que quedaron semiabiertas. |
| [[ui:ns.held]] | Cuántas veces un flujo esperó por el límite de ancho de banda: el emisor se ralentizó, no se descartó nada. |
| [[ui:common.volume]] | Bytes reenviados. |

Con la captura activada, el [Inspector](inspector.md) muestra los datagramas y
fragmentos retransmitidos —como máximo uno cada 25 ms, los dos sentidos juntos—,
cada uno con su suerte: `forwarded +43ms`, `· corrupted`, `· reordered`,
`· copy 2/2`, `dropped (loss)`, `dropped (burst)`, `dropped (offline)`,
`throttled` y, sobre TCP, `reset` y `half-open`. → es del cliente al destino, ←
del destino al cliente.

Una trama retransmitida nombra sockets reales: [[ui:ins.local]] es la dirección en
la que escucha el relé y [[ui:bc.peer]] es hacia dónde iba ese datagrama o
fragmento: el destino, o el cliente al que volvió una respuesta. El tramo cierra
el veredicto: `forwarded +43ms · client→target`,
`dropped (loss) · target→client`. Así que un datagrama retransmitido se puede
guardar con [[ui:sig.fromFrame]] y se dirige a la dirección hacia la que iba; un
fragmento TCP es un trozo de un flujo y no se puede (consulta
[Inspector](inspector.md#save-as-signal)).

## El mismo tráfico, la misma suerte {#seed}

Cada decisión —qué datagrama se pierde, cuánto se retrasa, dónde se corrompe— se
sortea a partir de la semilla del relé, por separado para cada sentido y, sobre
TCP, para cada conexión. Con la misma semilla y el mismo tráfico, el relé
descarta los mismos datagramas y los retrasa igual.

La pantalla [[ui:nav.netsim]] toma una semilla nueva cada vez que inicias un relé.
Para repetir una ejecución exactamente, usa un nodo [[ui:exp.node.impairment]] en
un experimento: sortea a partir de la semilla de la ejecución, que puedes fijar
(consulta [Repetir una ejecución con fallos](../experiments/faults.md#seed)).

## En experimentos {#experiments}

Dos nodos meten el mismo relé en un experimento:

- [[ui:exp.node.impairment]] abre un relé antes del primer paso y lo cierra
  cuando termina la ejecución, termine como termine. Sus direcciones de escucha y
  destino solo aceptan parámetros; el destino puede ser un nombre de host,
  resuelto cuando empieza la ejecución. Consulta
  [Nodos](../experiments/nodes.md#node-impairment).
- [[ui:exp.node.impairment_change]] cambia un relé de la ejecución a otro perfil a
  partir de ese paso —limpio, con pérdidas, sin conexión, limpio otra vez— y el
  informe de la ejecución cuenta lo que hizo cada fase. Consulta
  [Nodos](../experiments/nodes.md#node-impairment_change).

En un nodo OSC o UDP, [[ui:exp.routeThrough]] coloca un
[[ui:exp.node.impairment]] delante y apunta el nodo al relé. Consulta
[Fallos](../experiments/faults.md).

## Véase también {#related}

- [Emuladores](emulators.md): el destino que poner detrás del relé.
- [Inspector](inspector.md): la suerte de cada datagrama.
- [UDP y TCP](../protocols/udp-tcp.md)

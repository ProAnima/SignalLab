---
title: Cómo se mueve una ejecución
description: Inicio y Fin, salidas y cables, ramas paralelas y unión, bifurcación, Reintentar, Repetir y Bucle, esperas que escuchan desde el principio de la ejecución, y qué se comprueba antes de una ejecución.
---

# Cómo se mueve una ejecución

Una ejecución empieza en [[ui:exp.node.start]], sigue los cables de nodo en nodo y termina cuando
todas las ramas han acabado y se ha llegado a [[ui:exp.node.end]]. Esta página explica las reglas
que sigue; lo que hace cada nodo está en [la referencia de los nodos](nodes.md), y los valores que
viajan con ella, en [datos](data.md).

## Inicio y Fin {#start-end}

Un experimento tiene exactamente un [[ui:exp.node.start]] y un [[ui:exp.node.end]].

- [[ui:exp.node.start]] no tiene entrada. Se supera de inmediato, y su fila en la línea de tiempo da
  la semilla de la ejecución. Su salida puede tener varios cables: el experimento empieza entonces
  con ramas paralelas.
- Toda rama que llega a [[ui:exp.node.end]] se detiene ahí. Fin se muestra como en ejecución desde
  la primera llegada y se supera una vez, después de que haya terminado la última rama — y no lo
  hace en absoluto si algún paso falló. Esa superación es lo que hace que la ejecución se considere
  [[ui:exp.passed]].
- Una ejecución en la que todas las ramas terminaron sin error pero ninguna llegó a Fin falla con
  `run.no_end`.

## Salidas y cables {#outputs}

El paso de un nodo termina eligiendo una salida, y la ejecución sigue cada cable de esa salida. La
mayoría de los nodos tienen una salida, [[ui:exp.outputPort]]; algunos eligen entre varias:

| Nodo | Salidas que deben conectarse | Salidas que pueden conectarse |
| --- | --- | --- |
| [[ui:exp.node.end]] | — | — |
| [[ui:exp.node.fork]] | [[ui:exp.branch1]], [[ui:exp.branch2]] | — |
| [[ui:exp.node.branch_status]], [[ui:exp.node.branch_value]] | [[ui:exp.yes]], [[ui:exp.no]] | — |
| Todas las esperas ([[ui:exp.node.wait_osc]], [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_http]], [[ui:exp.node.wait_ws]]) | [[ui:exp.portMatched]] | [[ui:exp.portTimeout]] |
| [[ui:exp.node.loop]] | [[ui:exp.portBody]], [[ui:exp.portDone]] | [[ui:exp.portLimit]] |
| Todos los demás nodos | [[ui:exp.outputPort]] | — |

Para conectar, arrastra desde una salida hasta un nodo; si la sueltas en un lienzo vacío, añade ahí
un nodo nuevo. Arrastrar desde una salida que ya tiene un cable añade otro. [[ui:exp.addNext]], la
tecla <kbd>A</kbd> y el ＋ de un cable insertan un nodo en el cable existente.

Un grafo sin terminar es un borrador: se guarda, pero no se ejecuta. [[ui:exp.needsLinks]] en la
barra de herramientas dice qué falta y muestra el nodo. Consulta
[qué se comprueba antes de una ejecución](#validation).

## Ramas paralelas {#parallel}

### Varios cables desde una salida {#fan-out}

Cuando una salida tiene varios cables — los de [[ui:exp.node.start]] incluidos — cada nodo al que
llevan se ejecuta a la vez. El primer cable continúa la rama; cada cable adicional inicia una rama
paralela. Cada rama lleva su propia copia de las variables y de la última respuesta HTTP, así que lo
que una rama define o recibe no lo ven las demás.

### Rama paralela y unión {#fork-join}

[[ui:exp.node.fork]] se supera de inmediato y sale por [[ui:exp.branch1]] y [[ui:exp.branch2]] — lo
mismo que dos cables desde una salida, dibujado como un nodo.

[[ui:exp.node.join]] espera **todos** los cables que llegan a él, y luego continúa como una sola
rama con las copias fusionadas en el orden de esos cables:

- las variables de todos ellos — en un nombre que definan dos ramas, gana el cable listado después
  en el experimento;
- la respuesta HTTP del último cable, en ese orden, que traiga una;
- para las esperas posteriores, la acción más temprana de sus últimas acciones.

El orden de los cables decide, nunca qué rama terminó primero por casualidad.

::: warning Une solo lo que se ejecuta en paralelo
Una unión cuenta sus cables, sin importar cómo llegaron a ejecutarse en paralelo: desde un
[[ui:exp.node.fork]], desde varios cables de una salida, desde caminos separados. Detrás de
[[ui:exp.yes]] y [[ui:exp.no]] de una bifurcación solo se ejecuta un camino, así que una unión
alimentada por ambos espera una rama que nunca llega: la ejecución falla con `run.join_waiting`,
indicando cuántos cables nunca se siguieron. Para reunir caminos alternativos, conéctalos
directamente al siguiente nodo.
:::

Un nodo que no es una unión, alcanzado por dos ramas paralelas, se ejecuta una vez por cada una de
ellas.

### Cuando falla un paso {#failure}

El primer fallo hace fallar la ejecución. Las demás ramas no inician ningún paso nuevo: una
repetición o una carga termina antes de tiempo, cualquier otro paso en el que estén se ejecuta hasta
su final. Un fallo que encuentran mientras tanto se informa en la línea de tiempo, pero no es el
error de la ejecución. Un paso que llegó a un tiempo de espera mientras había un cable
[[ui:exp.portTimeout]] no falló — consulta [esperas](#timeout).

## Bifurcación {#branching}

| Nodo | Sale por [[ui:exp.yes]] cuando |
| --- | --- |
| [[ui:exp.node.branch_status]] | la última respuesta HTTP en este camino tiene el estado dado |
| [[ui:exp.node.branch_value]] | se cumple su comparación — consulta [comparar valores](data.md#compare) |

En caso contrario, cada uno sale por [[ui:exp.no]]. Una bifurcación por estado necesita una
solicitud HTTP antes en todos los caminos; una bifurcación por valor necesita que los nombres que
lee se conozcan allí. Cuando los caminos posteriores a [[ui:exp.yes]] y [[ui:exp.no]] se
reencuentran, el nodo donde se reencuentran se ejecuta una vez, y allí solo se conocen las variables
definidas en ambos caminos ([dónde se conoce una variable](data.md#visibility)).

## Reintentar {#retry}

Un paso que envía o escucha puede volver a intentarlo cuando falla: activa [[ui:exp.retryOn]] en sus
propiedades.

| Ajuste | Qué | Rango | Valor inicial |
| --- | --- | --- | --- |
| [[ui:exp.attempts]] | intentos en total, el primero incluido | 1–10 | 3 |
| [[ui:exp.retryDelay]] | la pausa antes del segundo intento | 0–60 000 ms | 500 |
| [[ui:exp.backoff]] | [[ui:exp.backoff.fixed]]: todas las pausas iguales; [[ui:exp.backoff.exponential]]: cada pausa el doble que la anterior | — | [[ui:exp.backoff.fixed]] |

- Reintentar se aplica a [[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.mqtt]],
  [[ui:exp.node.osc]], [[ui:exp.node.udp]], [[ui:exp.node.ws_connect]], [[ui:exp.node.ws_send]] y
  todas las esperas. Los demás nodos lo rechazan (`node.retry_unsupported`), y una solicitud HTTP
  bajo [carga](load.md) no admite Reintentar.
- Ninguna pausa dura más de 60 s, diga lo que diga la duplicación.
- Cada intento fallido aparece en la línea de tiempo como [[ui:exp.retry]], con su número y su
  motivo. El paso pasa entonces, o falla con el motivo del último intento.
- Solo se repite la ejecución. Un campo cuya plantilla no se resuelve falla de inmediato.
- Un envío que espera su respuesta vuelve a enviar. Una espera vuelve a esperar, contando desde la
  última acción de la rama como antes.
- Una espera con un cable [[ui:exp.portTimeout]] no falla por un tiempo de espera, así que no se
  reintenta: sigue [[ui:exp.portTimeout]].
- [Detener](#stop) termina una pausa de inmediato.

## Repetir {#repeat}

Un paso que envía puede enviar una y otra vez — un latido, un sondeo, un flujo constante — sin un
bucle en el grafo: activa [[ui:exp.repeatOn]].

| Ajuste | Qué | Rango | Valor inicial |
| --- | --- | --- | --- |
| [[ui:exp.repeatBy]] | [[ui:exp.repeatBy.count]] o [[ui:exp.repeatBy.duration]] | — | [[ui:exp.repeatBy.count]] |
| [[ui:exp.repeatCount]] | envíos en total, el primero incluido | 2–10 000 | 10 |
| [[ui:exp.repeatDuration]] | cuánto tiempo seguir enviando, desde el primer envío | 1–300 000 ms | 10 000 |
| [[ui:exp.repeatInterval]] | la pausa entre dos envíos | 10–60 000 ms | 1000 |
| [[ui:exp.repeatJitter]] | cada pausa es hasta esto más larga, al azar | 0–60 000 ms | 0 |

- Repetir se aplica a [[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.mqtt]],
  [[ui:exp.node.osc]], [[ui:exp.node.udp]] y [[ui:exp.node.ws_send]] (`node.repeat_unsupported` en
  los demás). Una solicitud HTTP tiene Repetir o [carga](load.md), no ambas.
- Cada envío se hace como se haría uno solo: sus plantillas se leen de nuevo — `{{counter}}` es el
  número del envío, `{{now}}` su hora — y Reintentar, cuando está activado, se aplica a cada envío.
  Un envío que espera una respuesta espera la suya.
- Durante un tiempo, un envío solo se hace si puede empezar antes de que se acabe el tiempo.
- El jitter se sortea a partir de la semilla de la ejecución: la misma semilla da las mismas pausas.
- La línea de tiempo informa del progreso como [[ui:exp.repeating]] como máximo una vez por segundo.
  El paso pasa tras el último envío, con el desenlace de ese envío; un envío que falla
  definitivamente hace fallar el paso.
- Un fallo en otra rama termina los envíos; [Detener](#stop) termina una pausa de inmediato.

Debe caber en una ejecución: los envíos y sus pausas más largas (`(count − 1) × (interval + jitter)`)
como máximo 300 s (`node.repeat_too_long`), y una repetición por tiempo como máximo 10 000 envíos
(`node.repeat_too_many`).

## Bucle {#loop}

[[ui:exp.node.loop]] ejecuta los pasos de su salida [[ui:exp.portBody]] una y otra vez; el último de
ellos está conectado de vuelta al Bucle.

| Ajuste | Qué | Rango |
| --- | --- | --- |
| [[ui:exp.loopMax]] | el máximo de iteraciones | 1–1000 |
| [[ui:exp.loopUntilOn]] | una condición de salida: [[ui:exp.value]], [[ui:exp.operator]], [[ui:exp.expected]], como en [[ui:exp.node.assert_value]] | opcional |

1. Alcanzado desde fuera, el Bucle inicia la iteración 1 en [[ui:exp.portBody]].
2. Cada vez que el cuerpo vuelve, se lee la condición de salida — después de la iteración, así que
   el cuerpo siempre se ejecuta al menos una vez y puede definir lo que comprueba.
3. Cuando se cumple la condición, el Bucle sale por [[ui:exp.portDone]].
4. Si no, empieza la siguiente iteración, mientras quede alguna.
5. Cuando las iteraciones se acaban primero, el Bucle sale por [[ui:exp.portLimit]] si está
   conectado, y hace fallar la ejecución con `loop.limit` si no lo está. Sin condición, el cuerpo se
   ejecuta en cada iteración y el Bucle sale por [[ui:exp.portDone]].

Dentro del cuerpo, `{{counter}}` es el número de la iteración, ya que cada nodo cuenta sus propias
ejecuciones. La condición y los pasos posteriores a [[ui:exp.portDone]] o [[ui:exp.portLimit]]
pueden usar lo que define cada iteración del cuerpo — por ejemplo, un estado que el cuerpo extrae;
el cuerpo en sí solo ve lo que se conocía cuando se alcanzó el Bucle.

La plantilla [[ui:exp.templatePoll]] pregunta a un dispositivo por su estado cada 0,3 s hasta que
responde `ready`, como máximo 10 veces.

### Qué puede contener un cuerpo {#loop-body}

Un cuerpo se ejecuta como una sola rama, una iteración tras otra. El cable de vuelta al Bucle es el
único ciclo que puede tener un experimento; cualquier otro es `graph.cycle`.

| Regla | Error |
| --- | --- |
| Algo en [[ui:exp.portBody]] vuelve al Bucle | `loop.no_return` |
| Cada salida del cuerpo tiene un solo cable | `loop.body_parallel` |
| Cada salida del cuerpo sigue en el cuerpo o vuelve al Bucle | `loop.body_leaves` |
| Solo la salida [[ui:exp.portBody]] del Bucle entra en el cuerpo | `loop.body_entered` |
| Ningún [[ui:exp.node.start]], [[ui:exp.node.end]], [[ui:exp.node.fork]], [[ui:exp.node.join]] ni otro [[ui:exp.node.loop]] en el cuerpo | `loop.body_unsupported` |

## Esperas {#waits}

Una espera pasa cuando llega un mensaje que está esperando: [[ui:exp.node.wait_osc]],
[[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_http]] y
[[ui:exp.node.wait_ws]]. Lo que coincide con cada una está en [la referencia de los nodos](nodes.md);
aquí se explica cómo escuchan.

### Escuchar desde el principio {#listening}

La ejecución abre aquello en lo que escuchan sus esperas **antes de su primer paso**, así que una
respuesta más rápida que el paso siguiente no se pierde:

| Espera | Se abre antes del primer paso |
| --- | --- |
| OSC, UDP | un socket UDP por dirección [[ui:exp.listenOn]], compartido por todas las esperas que haya en ella |
| Solicitud HTTP | un escucha por dirección — el [emulador](faults.md#emulator) HTTP de la ejecución cuando hay uno; si no, uno que responde `204` |
| MQTT | una conexión por bróker y filtro de temas, suscrita; los mensajes retenidos que el bróker reproduce entonces se ignoran |
| WebSocket | nada: lee la conexión que abrió un [[ui:exp.node.ws_connect]] cuando se ejecutó |

Como se abren primero, estas direcciones quedan fijadas antes de la ejecución: una espera OSC, UDP o
HTTP escucha en un `IP:port` literal con un puerto distinto de 0, y el bróker y el tema de una
espera MQTT solo aceptan parámetros. Un puerto que no se puede abrir — ocupado, o que no es una
dirección de este equipo — detiene la ejecución antes de cualquier tráfico, en el campo de esa
espera. Todo se cierra cuando termina la ejecución, sea como sea.

### Qué mensajes cuentan {#counting}

Una espera considera los mensajes que llegaron después de que empezara la **última acción en su
rama** — la última solicitud, mensaje, publicación, conexión WebSocket o envío — o, antes de
cualquier acción, después de que empezara la ejecución. Un mensaje anterior a la solicitud no cuenta,
y una Pausa o una Marca de registro entre la solicitud y la espera no oculta su respuesta. Tras una
unión, cuenta la acción más temprana de las últimas acciones de las ramas fusionadas.

Una espera toma el primer mensaje que coincide y lo consume: dos esperas nunca coinciden con el
mismo mensaje.

Cada socket, suscripción o conexión guarda como máximo 1024 mensajes y 64 MiB para sus esperas;
pasado eso, los más antiguos se descartan y se cuentan.

### Tiempo de espera {#timeout}

[[ui:exp.waitTimeout]] es 1–120 000 ms, 2000 al principio. Cuando nada coincide a tiempo:

- con un cable [[ui:exp.portTimeout]], la espera lo sigue;
- sin él, el paso falla con `wait.timeout`, que dice cuántos otros mensajes llegaron mientras tanto —
  un patrón equivocado se ve distinto de un dispositivo en silencio — y, en su detalle, cuántos
  mensajes más antiguos se descartaron cuando la cola estaba llena.

La variable de la espera — `reply`, o `request` para HTTP — solo existe tras
[[ui:exp.portMatched]]. Cuando el [[ui:dock.inspector]] está capturando, el paso también enlaza la
trama con la que coincidió: consulta [la línea de tiempo](runs.md#timeline).

## Una respuesta en el mismo paso {#reply}

Un [[ui:exp.node.osc]] o un [[ui:exp.node.udp]] pueden esperar su propia respuesta: activa
[[ui:exp.expectReply]].

| Ajuste | Qué | Valor inicial |
| --- | --- | --- |
| [[ui:exp.replyOn]] | la dirección en la que se espera la respuesta; el puerto 0 es cualquier puerto libre | `0.0.0.0:0` |
| [[ui:exp.replyAddress]] (OSC), [[ui:exp.replyMode]] (UDP) | qué debe ser la respuesta, como en la espera correspondiente | cualquiera |
| [[ui:exp.waitTimeout]] | 1–120 000 ms | 2000 |
| [[ui:exp.replyVariable]] | la variable en la que se escribe la respuesta | `reply` |

El socket de [[ui:exp.replyOn]] se abre antes del primer paso, como el de una espera, y el mensaje
**sale desde él**: se oye a un dispositivo que responde al propio puerto del emisor, y a uno que
responde a un puerto fijo cuando ese puerto es el indicado. El paso pasa con una respuesta que
coincide, y la variable existe tras su salida. No hay salida [[ui:exp.portTimeout]]: ninguna
respuesta a tiempo hace fallar el paso, y Reintentar puede volver a enviar. Para bifurcar por
silencio, usa una espera aparte.

## Qué se comprueba antes de una ejecución {#validation}

El editor comprueba el experimento mientras lo editas; el botón Ejecutar lo comprueba una vez más.
Un problema nombra el nodo, y el campo cuando lo hay.

| Regla | Error |
| --- | --- |
| El experimento tiene un nombre | `doc.name_required` |
| 1–64 nodos, exactamente un Inicio y un Fin | `doc.node_count`, `doc.start_end_count` |
| Inicio no tiene entrada | `graph.start_input` |
| Un cable lleva a otro nodo que existe | `doc.connection_invalid` |
| El mismo cable no está dos veces | `doc.connection_duplicate` |
| Cada salida que debe conectarse lo está | `graph.outputs_required` |
| Un nodo no tiene cable en una salida que no tiene | `graph.port_unexpected` |
| Se llega a cada nodo desde Inicio | `graph.unreachable` |
| Ningún ciclo salvo el cable de vuelta de un Bucle | `graph.cycle`, y las [reglas del cuerpo](#loop-body) |
| Una comprobación o Extraer tiene una solicitud HTTP antes en todos los caminos; una solicitud bajo carga no cuenta | `graph.needs_http` |
| Cada plantilla se analiza, y cada nombre que usa se conoce en todos los caminos | `template.*`, `name.*` — consulta [datos](data.md#unknown-names) |
| Cada campo está presente y en rango | `node.*` |
| Un nodo que nombra a otro — [[ui:exp.node.impairment_change]], [[ui:exp.node.emulator_state]], los nodos WebSocket — nombra uno que está ahí, y un nodo WebSocket va después de su conexión | `impair.relay_unknown`, `emulator.node_unknown`, `ws.connection_unknown`, `ws.connection_after` |
| Dos de los sockets de la ejecución no comparten un puerto | consulta [fallos](faults.md#ports) |

La ejecución comprueba después lo que necesita para empezar: cada
[secreto](data.md#secret-check) guardado, cada puerto abierto. Hasta que todo se cumple, no se
ejecuta ningún paso y no se envía nada.

## Límite de tiempo {#limit}

Una ejecución dura como máximo 300 s. Una que siga en marcha entonces se detiene y falla con
`run.timeout`. Desde la [línea de comandos](../automation/cli.md#cli-run) y
[la API](../api/run.md) el límite puede ser más corto, 1–300 s.

## Detener {#stop}

Mientras una ejecución avanza, el botón Ejecutar es [[ui:common.stop]]. Detener termina la ejecución
de inmediato: cada rama, cada pausa de Reintentar o Repetir, cada espera y cada carga — las
solicitudes en curso se descartan. Sus sockets, suscripciones, emuladores y relés se cierran, y sus
conexiones WebSocket envían una trama de cierre. [[ui:app.stopAll]] en la cabecera hace lo mismo con
todas las tareas. Una ejecución detenida no guarda ningún informe; consulta
[ejecuciones](runs.md#stop).

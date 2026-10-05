---
title: Pruebas de carga
description: Envía la solicitud de un nodo HTTP según un perfil de carga — constante, rampa, escalones, pico o llegadas aleatorias —, mide latencias, errores y la tasa alcanzada, júzgalos con umbrales y compara dos ejecuciones.
---

# Prueba de carga de una solicitud HTTP

Un nodo [[ui:exp.node.http]] puede enviar su solicitud muchas veces seguidas, según un perfil de
solicitudes por segundo, muchas a la vez, y medir lo que vuelve: percentiles de latencia,
errores, la tasa que alcanzó. Los umbrales deciden si el paso se supera, y [[ui:exp.compare]] pone
las cifras junto a las de una ejecución anterior.

Una carga es un ajuste del nodo, no un nodo propio: el resto del experimento (emuladores, relés de
degradación, otras ramas) se ejecuta a su alrededor como de costumbre.

## Someter una solicitud a carga {#turn-on}

1. Selecciona un nodo [[ui:exp.node.http]] y rellena su solicitud.
2. En sus propiedades, activa [[ui:exp.loadOn]].
3. Elige un [[ui:exp.loadShape]] y sus cifras. El gráfico de debajo, [[ui:exp.loadChart]], dibuja
   la tasa e indica cuántas solicitudes suma en total y en cuántos segundos.
4. Indica en [[ui:exp.loadConcurrency]] cuántas solicitudes pueden estar en curso al mismo tiempo.
5. Añade o cambia los [[ui:exp.thresholds]].
6. Ejecuta el experimento.

Una carga empieza como una [[ui:exp.loadShape.ramp]] de 0 a 100 solicitudes por segundo durante
30 000 ms, 32 a la vez, con dos umbrales:
[[ui:exp.metric.p95_ms]] < 500 ms y [[ui:exp.metric.error_rate]] < 1 %.

**La carga sustituye a Repetir y Reintentar.** Al activarla se desactivan, y se rechaza un nodo con
carga y cualquiera de los dos (`node.load_alone`): una solicitud fallida se cuenta, no se
reintenta. Solo una solicitud HTTP puede ejecutarse bajo carga (`node.load_unsupported`).

**La solicitud se lee una vez.** Sus plantillas se resuelven cuando empieza el paso, así que todas
las solicitudes de la carga son la misma: `{{counter}}` y `{{uuid}}` toman un solo valor para todas.
Consulta [plantillas](data.md#templates).

**Un solo cliente para toda la carga.** Las solicitudes comparten el almacén de cookies de la
ejecución cuando [[ui:exp.cookies]] está activado, y una sola memoria Digest, así que un único
desafío sirve para todas. Cada solicitud tiene el propio tiempo de espera del nodo.

**El Inspector recibe una muestra:** como máximo un intercambio cada 100 ms, para que una carga no
inunde el [[ui:dock.inspector]].

## Perfiles {#profiles}

| [[ui:exp.loadShape]] | Ajustes | La tasa a lo largo del tiempo |
| --- | --- | --- |
| [[ui:exp.loadShape.constant]] | [[ui:exp.loadRate]], [[ui:exp.loadDuration]] | la tasa durante todo el tiempo |
| [[ui:exp.loadShape.ramp]] | [[ui:exp.loadFrom]], [[ui:exp.loadTo]], [[ui:exp.loadDuration]] | en línea recta de una tasa a la otra |
| [[ui:exp.loadShape.steps]] | [[ui:exp.loadFrom]], [[ui:exp.loadStepBy]], [[ui:exp.loadEvery]], [[ui:exp.loadSteps]] | la primera tasa y luego un escalón más en cada nivel, cada nivel durante el mismo tiempo |
| [[ui:exp.loadShape.spike]] | [[ui:exp.loadBase]], [[ui:exp.loadPeak]], [[ui:exp.loadAt]], [[ui:exp.loadSpikeFor]], [[ui:exp.loadDuration]] | la tasa base, el pico durante un tiempo a partir de un momento dado, y luego otra vez la tasa base |
| [[ui:exp.loadShape.poisson]] | [[ui:exp.loadRate]], [[ui:exp.loadDuration]] | llegadas al azar, con la tasa como promedio |

Al cambiar de forma se conserva lo que se puede trasladar: cuánto dura y la tasa más alta que
alcanza.

### Límites {#limits}

| Ajuste | Rango |
| --- | --- |
| [[ui:exp.loadRate]] de [[ui:exp.loadShape.constant]] y [[ui:exp.loadShape.poisson]], [[ui:exp.loadPeak]] | 0,1–100 000 solicitudes/s |
| [[ui:exp.loadFrom]], [[ui:exp.loadTo]], [[ui:exp.loadBase]] | 0–100 000 solicitudes/s |
| Cada nivel de [[ui:exp.loadShape.steps]], incluido el último | 0–100 000 solicitudes/s; el escalón puede ser negativo |
| [[ui:exp.loadDuration]], [[ui:exp.loadEvery]] | 100–300 000 ms |
| [[ui:exp.loadSteps]] | 1–100, y todos los niveles juntos, como máximo 300 000 ms |
| Un pico | de más de 0 ms, y terminado antes del final de la duración |
| [[ui:exp.loadConcurrency]] | 1–512 |
| [[ui:exp.thresholds]] | como máximo 16, cada valor un número, 0 o mayor |

Se rechaza un perfil que no suma ninguna solicitud
(`load.nothing_planned`). Las tasas son las de la
[ráfaga HTTP](../protocols/http.md).

Un perfil puede durar tanto como una ejecución entera, 300 s, pero el
[límite de tiempo](flow.md#limit) de la ejecución cuenta todos los pasos, así que deja margen para
el resto del experimento.

### Cuántas solicitudes {#planned}

Las solicitudes de un perfil son su tasa sumada a lo largo del tiempo:

| Perfil | Solicitudes |
| --- | --- |
| [[ui:exp.loadShape.constant]], 100/s durante 1000 ms | 100 |
| [[ui:exp.loadShape.ramp]], 0 → 100/s en 2000 ms | 100 |
| [[ui:exp.loadShape.steps]], desde 10/s subiendo de 10/s en 10/s, 3 niveles de 1000 ms | 60 (10 + 20 + 30) |
| [[ui:exp.loadShape.spike]], 10/s con 100/s a partir de los 1000 ms durante 500 ms, 2000 ms en total | 65 |
| [[ui:exp.loadShape.poisson]], 200/s durante 10 000 ms | 2000 de media |

## La planificación {#schedule}

La enésima solicitud toca en el momento en que el recuento del perfil llega a n; la primera, de
inmediato. Cada momento se calcula desde el inicio de la carga, así que un despertar tardío nunca
desplaza las solicitudes siguientes, y la tasa que describe el perfil es la tasa que se pide.

El perfil [[ui:exp.loadShape.poisson]] sortea los intervalos entre llegadas a partir de la semilla
de la ejecución: la misma semilla da los mismos momentos, así que una carga aleatoria se puede
repetir exactamente. Consulta [semillas](runs.md#seeds).

**Solicitudes omitidas.** Nunca hay en curso más solicitudes que las que indica
[[ui:exp.loadConcurrency]]. Cuando todas ellas siguen esperando su respuesta, la siguiente
solicitud espera a que quede un hueco libre. Si fuera a salir más de 50 ms después de su momento,
no se envía tarde: se omite y se cuenta como omitida, junto con todas las demás que tocaban
mientras tanto, y la carga sigue con la primera que aún va a tiempo. Muchas solicitudes omitidas
significan que el servidor, o el valor de [[ui:exp.loadConcurrency]], no pudo seguir el ritmo del
perfil.

## Mientras se ejecuta {#progress}

Una vez por segundo, la línea de tiempo muestra el paso como [[ui:exp.load]], con los segundos
transcurridos, las solicitudes enviadas, la tasa del último segundo, el p95 hasta el momento y las
solicitudes fallidas. [[ui:common.stop]] termina la carga al instante y descarta las solicitudes en
curso; un fallo en otra rama la termina en menos de un segundo.

## Qué se mide {#metrics}

Tras la última respuesta, el paso tiene sus mediciones, guardadas en su último evento de la línea
de tiempo y en el [informe de la ejecución](runs.md#report):

| Medición | Qué |
| --- | --- |
| planned | las solicitudes que suma el perfil ([[ui:exp.loadShape.poisson]]: de media) |
| sent | solicitudes que se respondieron o que fallaron |
| ok | respondidas con un estado 2xx |
| failed | cualquier otro estado, o ninguna respuesta |
| missed | tocaban mientras todos los huecos estaban ocupados, y se omitieron |
| rps | solicitudes enviadas por segundo: sent ÷ la duración del perfil, o ÷ el tiempo hasta que salió la última solicitud, si fue más tarde |
| error_rate | failed, en % de sent |
| min, mean, max | la solicitud más rápida, la media y la más lenta, en ms |
| p50, p90, p95, p99 | la latencia que el 50, 90, 95 y 99 % de las solicitudes no superaron, en ms |
| received_bytes | bytes de cuerpo recibidos en total |
| statuses | solicitudes por estado (`200`, `503`) y, si no lo hay, por causa (`timeout`, `refused`, `reset` …) |
| seconds | cada segundo del perfil: solicitudes enviadas, fallidas y su latencia media |
| histogram | solicitudes por latencia, hasta 1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10 000 ms, y más lentas |

La latencia de una solicitud va desde que se envía hasta que se ha leído toda su respuesta, y una
solicitud fallida cuenta con el tiempo que tardó en fallar. Los percentiles se leen de intervalos
logarítmicos de un 1 % de ancho y quedan a menos de un 0,5 % del valor real, dure lo que dure la
carga.

## Umbrales {#thresholds}

Un umbral es una fila de [[ui:exp.thresholdMetric]], [[ui:exp.thresholdOp]] y
[[ui:exp.thresholdValue]]; el botón [[ui:exp.thresholdAdd]] añade uno.

| [[ui:exp.thresholdMetric]] | Se mide en |
| --- | --- |
| [[ui:exp.metric.p50_ms]], [[ui:exp.metric.p90_ms]], [[ui:exp.metric.p95_ms]], [[ui:exp.metric.p99_ms]] | ms |
| [[ui:exp.metric.mean_ms]], [[ui:exp.metric.max_ms]] | ms |
| [[ui:exp.metric.error_rate]] | % de las solicitudes enviadas |
| [[ui:exp.metric.rps]] | solicitudes por segundo alcanzadas |
| [[ui:exp.metric.missed]] | solicitudes |

La [[ui:exp.thresholdOp]] es una de `<`, `≤`, `>`, `≥`. Algunos umbrales habituales:

| [[ui:exp.thresholdMetric]] | [[ui:exp.thresholdOp]] | [[ui:exp.thresholdValue]] | El paso falla cuando |
| --- | --- | --- | --- |
| [[ui:exp.metric.p95_ms]] | `<` | 300 | una de cada veinte solicitudes, o más, tardó 300 ms o más |
| [[ui:exp.metric.error_rate]] | `<` | 1 | falló el 1 % de las solicitudes o más |
| [[ui:exp.metric.rps]] | `≥` | 180 | el servidor no pudo atender 180 solicitudes por segundo |
| [[ui:exp.metric.missed]] | `≤` | 0 | hubo que omitir aunque fuera una sola solicitud |

En un archivo, un umbral es `{ "metric": "p95_ms", "op": "lt", "value": 300 }`;
las métricas son `p50_ms`, `p90_ms`, `p95_ms`, `p99_ms`, `mean_ms`, `max_ms`,
`error_rate`, `rps` y `missed`, y las comparaciones, `lt`, `le`, `gt` y `ge`.

Los umbrales se leen tras la última respuesta, en su orden. El paso falla en el primero que no se
cumple (`load.threshold`), con un mensaje que da el umbral y el valor medido, y la ejecución falla
con él. Sin umbrales, una carga se supera mida lo que mida. Cuando el fallo de otra rama terminó la
carga antes de tiempo, ese fallo es el de la ejecución, no un umbral.

## El resultado {#result}

Cuando el paso se supera, la línea de tiempo lo resume: las solicitudes, la tasa, el p95 y la
proporción que falló. Selecciona el nodo: sus propiedades muestran [[ui:exp.loadResult]]:

- cada umbral, ✓ [[ui:exp.thresholdHeld]] o ✕ [[ui:exp.thresholdBroken]],
  con el valor medido;
- [[ui:http.sent]], [[ui:exp.loadRps]], [[ui:exp.loadErrors]] con su proporción,
  [[ui:http.missed]];
- [[ui:http.p50]], [[ui:http.p90]], [[ui:http.p95]], [[ui:http.p99]],
  [[ui:http.avg]], [[ui:http.max]];
- [[ui:exp.loadPerSecond]]: las solicitudes de cada segundo, las fallidas en rojo, y su latencia
  media como una línea;
- [[ui:exp.loadLatencies]]: cuántas solicitudes tardaron cuánto;
- los estados y las causas, cada uno con su recuento.

La línea de comandos muestra las mismas cifras y el veredicto de cada umbral; consulta
[`signallab run`](../automation/cli.md#cli-run).

## Comparar dos ejecuciones {#compare}

1. Ejecuta el experimento dos veces, o más.
2. En la línea de tiempo, pulsa [[ui:exp.compare]]. El botón aparece en cuanto una ejecución ha
   guardado su informe, y está desactivado mientras hay una ejecución en curso.
3. La última ejecución es [[ui:exp.compareAfter]], y la anterior, [[ui:exp.compareBefore]];
   cualquiera de las dos listas permite elegir otra ejecución.

Las listas contienen las ejecuciones de este experimento (según su nombre) a partir de los
informes de la carpeta de datos, las más recientes primero, como máximo 50: cada una con su fecha y
hora, cómo terminó y su semilla. También aparecen las ejecuciones de la línea de comandos, si usó
la misma carpeta de datos. Cambiar el nombre del experimento empieza un historial nuevo.

Para cada paso de carga, emparejado por nodo, una tabla muestra cada métrica
[[ui:exp.compareBefore]], [[ui:exp.compareAfter]] y el
[[ui:exp.compareChange]], en la unidad y en %. Un cambio en el sentido malo del 5 % o más (más
lento, más errores, más solicitudes omitidas, una tasa menor) es una regresión y se muestra en
rojo; pasar de nada a algo también cuenta. Bajo la tabla, el veredicto de cada umbral en ambas
ejecuciones. Un paso de carga que solo tiene una de las ejecuciones se marca
[[ui:exp.compareOnlyBefore]] o [[ui:exp.compareOnlyAfter]], sin cambios. Las ejecuciones sin pasos
de carga muestran [[ui:exp.compareNoLoad]].

Desde un script, [`experiment_runs`](../api/commands.md#experiment_runs) enumera las
ejecuciones y [`experiment_compare`](../api/commands.md#experiment_compare) compara dos, por el
nombre del archivo de su informe; `signallab mcp` ofrece lo mismo a un asistente
([MCP](../automation/mcp.md)).

## Comprobaciones después de una carga {#checks-after}

Una carga no deja ninguna respuesta propia: se mide, no se comprueba. Una comprobación o un
[[ui:exp.node.extract]] después de ella necesita antes otra solicitud sin carga, en todos los
caminos, o el experimento no se ejecuta (`graph.needs_http`). Para comprobar una respuesta de la
API bajo carga, pon un [[ui:exp.node.http]] normal después de la carga, o en una rama paralela
junto a ella.

[[ui:exp.sendNow]] en un nodo bajo carga envía su solicitud una sola vez.

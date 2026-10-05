---
title: Ejecuciones y resultados
description: Iniciar una ejecución con los valores del experimento u otros, la línea de tiempo, detenerla, qué significan superada y fallida, el informe de la ejecución, las semillas, probar un nodo y los archivos de experimento con sus versiones.
---

# Ejecuciones y resultados

## Iniciar una ejecución {#start}

Pulsa [[ui:exp.run]] en la barra de herramientas del editor. Antes de que se envíe nada:

1. El experimento se comprueba como lo comprueba el editor — su grafo, campos, nombres y valores
   ([qué se comprueba](flow.md#validation)) — y cada secreto que usa debe estar guardado
   ([secretos](data.md#secret-check)).
2. Se guarda.
3. La ejecución abre lo que necesita para toda su duración: sus emuladores, sus relés de degradación,
   los sockets en los que escuchan sus esperas y sus suscripciones MQTT.

Si algo de esto falla, no se ejecuta nada: se muestra el problema y se selecciona el nodo al que se
refiere. En caso contrario, la línea de tiempo se abre bajo el lienzo y los pasos aparecen en ella
según ocurren. Mientras la ejecución avanza, [[ui:exp.run]] pasa a ser [[ui:common.stop]] y el
experimento no se puede editar.

La ejecución usa los valores del perfil activo, y la semilla fijada en el experimento o una nueva.
Para ejecutar una vez con otros, usa [[ui:exp.runWith]].

### Ejecutar con otros valores {#run-with}

El ▾ junto a [[ui:exp.run]] abre [[ui:exp.runWith]]: otros valores para una ejecución, sin cambiar el
experimento.

| Campo | Qué | Vacío |
| --- | --- | --- |
| [[ui:exp.profile]] | el perfil para esta ejecución; se muestra cuando el experimento tiene perfiles | el activo |
| cada parámetro | un valor solo para esta ejecución | el valor del perfil elegido, mostrado en gris |
| [[ui:exp.seed]] | la semilla para esta ejecución, 0–9 007 199 254 740 991; el botón de al lado rellena la semilla de la última ejecución | la semilla fijada, o una nueva |

[[ui:exp.run]] en el formulario inicia la ejecución; [[ui:exp.resetOverrides]] vacía el formulario.
Lo que escribiste se queda en el formulario durante la sesión, así que el mismo cambio es un clic la
próxima vez. Un perfil que no se ejecutaría se marca ⚠. La precedencia de los valores está en
[datos](data.md#precedence).

Cuando el experimento tiene perfiles, o una ejecución tuvo valores escritos para ella, la línea de
tiempo dice qué perfil usó la ejecución — o [[ui:exp.runDefaults]] — y, cuando se escribieron
valores, [[ui:exp.overridden]].

## La línea de tiempo {#timeline}

[[ui:exp.timeline]] está bajo el lienzo; ▸ y ▾ la pliegan, y su borde la redimensiona. Contiene una
fila por evento de paso, de más antigua a más reciente: la hora, el nodo, su estado y qué ocurrió —
`HTTP 200 · 41 ms`, `token = abc123`, `/pong 42 ← 127.0.0.1:9000 · 12 ms`. Un fallo dice por qué,
con su detalle técnico en la descripción emergente. Al hacer clic en una fila se selecciona su nodo
en el lienzo.

| Estado | El paso |
| --- | --- |
| [[ui:exp.running]] | ha empezado |
| [[ui:exp.passed]] | terminó bien y eligió su salida |
| [[ui:exp.failed]] | falló; el primer fallo es el de la ejecución |
| [[ui:exp.retry]] | falló un intento y volverá a intentarlo ([Reintentar](flow.md#retry)) |
| [[ui:exp.repeating]] | está enviando una y otra vez, como máximo una fila por segundo ([Repetir](flow.md#repeat)) |
| [[ui:exp.load]] | está bajo carga, una fila por segundo ([carga](load.md#progress)) |

En el lienzo, cada nodo lleva una insignia con su último estado.

La cabecera de la línea de tiempo contiene:

- el resultado: [[ui:exp.running]], [[ui:exp.passed]], [[ui:exp.failed]] con el motivo, o
  [[ui:exp.stopped]];
- [[ui:exp.reportSaved]] una vez escrito el informe — en un navegador, un enlace que lo descarga; en
  la aplicación de escritorio, su ruta en la descripción emergente;
- [[ui:exp.compare]], para poner esta ejecución junto a una anterior
  ([comparar ejecuciones](load.md#compare));
- el perfil y los valores cambiados, como arriba;
- la semilla de la ejecución con [[ui:exp.pinSeed]], o, cuando el experimento tiene una fijada, esa
  semilla con [[ui:exp.unpinSeed]] ([semillas](#seeds)).

**Tramas.** Mientras el [[ui:dock.inspector]] está capturando, una espera — o un envío que espera su
respuesta — que coincidió con un mensaje guarda el número de la trama de ese mensaje. Un botón bajo
las filas nombra el nodo y la trama; abre el Inspector en el panel inferior con esa trama
seleccionada. Consulta el [Inspector](../tools/inspector.md).

La línea de tiempo muestra la última ejecución del experimento en esta sesión; se vacía al abrir
otro experimento.

## Detener {#stop}

Pulsa [[ui:common.stop]], o [[ui:app.stopAll]] en la cabecera para todas las tareas a la vez. La
ejecución termina de inmediato ([qué detiene](flow.md#stop)), la línea de tiempo muestra
[[ui:exp.stopped]], y **no se guarda ningún informe**. Una ejecución iniciada desde la línea de
comandos o desde la API en un servidor es una tarea como cualquier otra: [[ui:app.stopAll]] en ese
servidor también la detiene, y quien la llamó se entera de que se detuvo.

## El resultado {#result}

| Resultado | Significa | Informe |
| --- | --- | --- |
| [[ui:exp.passed]] | terminaron todas las ramas, ningún paso falló y se llegó a Fin | guardado |
| [[ui:exp.failed]] | falló un paso — una comprobación, una espera sin cable [[ui:exp.portTimeout]], un error de red, un umbral — o la ejecución se quedó sin tiempo (`run.timeout`), una unión esperó en vano (`run.join_waiting`), o ninguna rama llegó a Fin (`run.no_end`) | guardado, con el primer fallo |
| [[ui:exp.stopped]] | alguien la detuvo | ninguno |
| no se inició | el experimento no es válido, falta un secreto o no se pudo abrir un puerto | ninguno |

Una ejecución fallida nombra el nodo y el campo de su primer fallo; la
[referencia de errores](../reference/errors.md) enumera cada código. La línea de comandos dice lo
mismo con su código de salida: `0` superada, `1` fallida, `2` el experimento o la llamada no eran
válidos (un secreto que falta cuenta), `3` algo ajeno al experimento impidió que se ejecutara, como
un puerto que no se pudo abrir. Consulta [`signallab run`](../automation/cli.md#cli-run).

## El informe de la ejecución {#report}

Cada ejecución que termina por sí sola — superada o fallida — escribe un informe JSON en la carpeta
`runs` de la carpeta de datos: `Documents/SignalLab/runs` en un escritorio, la propia carpeta de
datos del servidor en un servidor ([archivos](../reference/files.md)). El archivo es
`run-<start time in ms>-<job number>.json`; un informe nunca se escribe sobre otro. Si no se puede
escribir, el editor dice por qué.

| Clave | Qué |
| --- | --- |
| `version` | el formato del informe, ahora 5 |
| `experiment` | el nombre del experimento |
| `document_version` | la versión del experimento, ahora 9 |
| `seed` | la semilla que usó la ejecución |
| `profile` | el perfil con el que se ejecutó, o `null` para los predeterminados |
| `overrides` | los valores escritos en [[ui:exp.runWith]] |
| `params` | cada valor de parámetro que usó la ejecución |
| `started_ms`, `ended_ms` | milisegundos Unix |
| `outcome` | `passed` o `failed` |
| `error` | el primer fallo, o `null` |
| `steps` | cada evento de paso, en orden (más abajo) |
| `emulators` | los recuentos de cada nodo Emulador — presente cuando hay uno ([emuladores](faults.md#emulator)) |
| `impairments` | los recuentos y las fases de cada nodo de Degradación — presente cuando hay uno ([fases](faults.md#change-impairment)) |

Cada evento de paso tiene:

| Clave | Qué |
| --- | --- |
| `job_id`, `node_id` | la ejecución y el nodo |
| `ts` | milisegundos Unix |
| `state` | `running`, `passed`, `failed`, `retry`, `repeating`, `load` |
| `detail` | qué ocurrió, en inglés |
| `message_key`, `message_params` | lo mismo que el texto de la interfaz y sus valores, para que el paso se pueda mostrar en cualquier idioma |
| `vars` | las variables que escribió el paso, si las hay |
| `error` | por qué falló: `code`, `params`, `node`, `field`, `detail` |
| `frame` | la trama del Inspector con la que coincidió una espera, si la captura estaba activada |
| `load` | lo que midió una carga ([mediciones](load.md#metrics)), en su último evento |

Los valores de los secretos nunca aparecen en un informe: se enmascaran como `••••`
([enmascaramiento](data.md#masking)).

El formato del informe creció con las funciones: la versión 3 añadió los recuentos de los emuladores,
la versión 4 las fases de las degradaciones, la versión 5 las mediciones de una carga.

Los informes son el historial de ejecuciones: [[ui:exp.compare]] los lee, y
[`experiment_runs`](../api/commands.md#experiment_runs) también. El `--report` de la línea de
comandos copia el informe de una ejecución donde quieras.

## Semillas {#seeds}

Cada ejecución tiene una semilla, un número entero de 0 a 9 007 199 254 740 991. Es, en orden:

1. la semilla dada a esta ejecución en [[ui:exp.runWith]], en la línea de comandos (`--seed`) o a la
   API;
2. la semilla fijada en el experimento;
3. una semilla aleatoria nueva.

La primera fila de la ejecución en la línea de tiempo la da, y el informe la conserva.

La semilla decide todo lo aleatorio que hace una ejecución: los
[generadores](data.md#generators) de las plantillas, el jitter de [Repetir](flow.md#repeat), las
llegadas de una [carga aleatoria](load.md#schedule), la suerte de cada paquete en un
[relé de degradación](faults.md#seed) y las elecciones aleatorias de un emulador. Cada uno sortea de
un flujo propio, así que las ramas paralelas nunca desplazan los valores de las demás.

Para repetir una ejecución:

1. Pulsa [[ui:exp.pinSeed]] junto a su semilla en la línea de tiempo. La semilla se guarda en el
   experimento, y cada ejecución la usa hasta que pulses [[ui:exp.unpinSeed]]. En
   [[ui:exp.params]], [[ui:exp.seed]] muestra y edita la semilla fijada; vacía, es
   [[ui:exp.seedRandom]].
2. Ejecuta con el mismo perfil y los mismos valores; el informe los enumera.

Lo que una semilla no puede repetir: la hora (`{{now}}`), `{{run.id}}`, y cuándo responden los
dispositivos y la red.

## Probar un solo nodo {#send-now}

Para probar un nodo sin ejecutar el experimento, selecciónalo y pulsa [[ui:exp.sendNow]] — o
<kbd>Ctrl</kbd>+<kbd>Enter</kbd> en sus propiedades — en un nodo [[ui:exp.node.http]],
[[ui:exp.node.tcp]], [[ui:exp.node.osc]], [[ui:exp.node.udp]], [[ui:exp.node.mqtt]],
[[ui:exp.node.ws_connect]] o [[ui:exp.node.ws_send]]. En una espera es [[ui:exp.listenNow]]: escucha
desde ahora hasta que coincida un mensaje o termine su tiempo de espera.

El motor ejecuta el nodo con el código que usa una ejecución, una vez:

- con los valores del perfil activo, los valores de las variables conocidos en esta sesión (de la
  última ejecución y de intentos anteriores) y los secretos guardados;
- con la semilla fijada, o una nueva; `{{run.id}}` es `0` y `{{counter}}` es `1`;
- sin Reintentar, Repetir ni carga — un solo envío;
- **sin cookies**: una sola solicitud, nada definido antes para devolver;
- sin los relés ni los emuladores de la ejecución. Una [[ui:exp.node.wait_http]] escucha en un
  escucha propio, y un envío o una espera WebSocket abre la conexión que describe su
  [[ui:exp.node.ws_connect]], para ese único intento.

Si un nombre que usa el nodo aún no tiene valor, no se envía nada y el resultado dice qué nombres
faltan — ejecuta el experimento, o usa [[ui:exp.sendNow]] primero en el nodo que los define.

El resultado muestra ✓ o ✕ y qué ocurrió. Para una solicitud HTTP también muestra el estado, el
tiempo, el tamaño y la [[ui:http.response]]; en una respuesta JSON se puede hacer clic en cada valor
para [extraerlo](data.md#extract), y [[ui:http.mockThis]] convierte la respuesta en una ruta de un
emulador. Los valores que recibió una espera, o que los nodos [[ui:exp.node.extract]] justo después
de una solicitud tomarían de su respuesta, pasan a ser conocidos por la vista previa y el siguiente
[[ui:exp.sendNow]]. Mientras una ejecución avanza, [[ui:exp.sendNow]] no está disponible.

La vista previa de un nodo con plantilla — lo que [enviará](data.md#preview) — también la resuelve
el motor, sin enviar nada.

## Archivos de experimento {#files}

### El experimento de trabajo {#working-file}

El editor contiene un experimento, guardado por sí solo 0,7 s después de cada cambio en
`experiment.json` en la carpeta de datos; la barra de herramientas dice [[ui:exp.saving]],
[[ui:exp.saved]] o [[ui:exp.saveError]]. Un grafo sin terminar también se guarda. Un archivo que no
se puede leer se informa con su ruta, nunca se reemplaza. Un archivo de experimento ocupa como
máximo 4 MiB.

En un servidor, el archivo está en la carpeta de datos del servidor, así que todos los navegadores
que abren el editor allí trabajan sobre el mismo experimento.

### Abrir, plantillas y exportar {#open-export}

El botón ☰ de la barra de herramientas abre [[ui:exp.documents]]:

- [[ui:exp.templates]]: [[ui:exp.templateEmpty]], [[ui:exp.templateHttp]],
  [[ui:exp.templateBranch]], [[ui:exp.templateParallel]], [[ui:exp.templatePingReply]],
  [[ui:exp.templatePoll]], [[ui:exp.templateFlaky]], [[ui:exp.templateFaults]],
  [[ui:exp.templateOutage]], [[ui:exp.templateWsEcho]]. Sus destinos están en `127.0.0.1`.
- [[ui:exp.importJson]] lee un archivo de hasta 4 MiB — de esta versión del formato de experimento o
  de una anterior, que se pone al día al abrirlo — y lo comprueba antes de mostrar su nombre y
  cuántos nodos y conexiones tiene. El archivo también debe caber en 4 MiB tal como lo escribe el
  editor, con sangría, así que un archivo compacto cerca del límite puede rechazarse. Un archivo
  roto se rechaza con la línea y la columna del problema, un archivo de un Signal Lab más nuevo con
  `doc.version_unsupported`, y el experimento actual se queda.
- [[ui:exp.openDocument]] reemplaza el experimento actual por el elegido. <kbd>Ctrl</kbd>+<kbd>Z</kbd>
  recupera el anterior durante esta sesión. Abrir un experimento no lo ejecuta.
- [[ui:exp.exportJson]] escribe una copia en la carpeta `exports` de la carpeta de datos, como
  `experiment-<time in ms>-<random>.json`, nunca sobre otra copia; en un navegador,
  [[ui:common.download]] la descarga.

La línea de comandos y la API toman los mismos archivos, y las plantillas por su nombre: `empty`,
`http-check`, `status-branch`, `parallel-flows`, `osc-ping-reply`, `poll-until-ready`, `flaky-api`,
`fault-phases`, `dependency-outage`, `websocket-echo`.

### Versiones del documento {#versions}

Un archivo de experimento tiene una `version`; este Signal Lab escribe la versión 9 y abre todas las
anteriores, rellenando lo que el archivo antiguo no podía contener. Un archivo de una versión más
nueva que la 9 se rechaza (`doc.version_unsupported`) en lugar de abrirse sin lo que contiene.

| Versión | Añadió |
| --- | --- |
| 2 | los parámetros y la semilla |
| 3 | los perfiles |
| 4 | Reintentar, y una respuesta esperada por un envío OSC o UDP |
| 5 | Repetir, y [[ui:exp.node.loop]] |
| 6 | [[ui:exp.node.emulator]] y [[ui:exp.node.wait_http]] |
| 7 | [[ui:exp.node.impairment]], [[ui:exp.node.impairment_change]] y [[ui:exp.node.emulator_state]] |
| 8 | los nodos WebSocket, la autenticación HTTP y el almacén de cookies |
| 9 | la carga en una solicitud HTTP, y la degradación sobre TCP |

Un archivo anterior a la versión 8 se abre con [[ui:exp.cookies]] desactivado, así que se ejecuta
como lo hacía; un archivo más nuevo conserva su propio ajuste. Guardado otra vez, cualquier archivo
pasa a ser la versión 9.

## Desde la línea de comandos o un servidor {#automation}

Una ejecución es la misma en todas partes: la línea de comandos y la API del servidor inician la
misma ejecución que el editor, con los mismos pasos, resultado e informe.

```bash
signallab run checkout.json --profile Stage -p api=http://192.0.2.10:8080 --seed 42 --report report.json
```

- [`signallab run`](../automation/cli.md#cli-run) ejecuta archivos de experimento o plantillas en
  este proceso o en un servidor, imprime los pasos como la línea de tiempo y sale con el código del
  resultado.
- [`POST /api/run`](../api/run.md) ejecuta uno en un servidor y responde con el resultado, o emite
  sus pasos según ocurren. Un cliente que se va no detiene la ejecución; se ejecuta hasta el final y
  conserva su informe.
- En CI: [GitHub Actions y otros](../automation/ci.md).

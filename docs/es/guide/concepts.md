---
title: Conceptos
description: Las ideas en las que se basa Signal Lab — pantallas y experimentos, señales, tareas, captura, emuladores, relés de degradación, parámetros, plantillas, secretos, semillas, informes y la carpeta de datos.
---

# Conceptos

Esta página explica las ideas en las que se basa Signal Lab, para que el resto de la
documentación se lea con facilidad. Cada sección enlaza con la página que cubre su tema por
completo.

## Pantallas y experimentos {#screens-and-experiments}

Signal Lab tiene dos formas de trabajar, y usarás las dos.

- **Las pantallas** son herramientas para el trabajo que haces ahora, a mano: envía este
  mensaje, escucha en ese puerto, inicia este emulador, mira qué guarda el bróker. Pruebas,
  miras, cambias algo y vuelves a probar. Cada protocolo y herramienta tiene una pantalla;
  consulta [La ventana](interface.md).
- **Los experimentos** son flujos que construyes una vez y ejecutas de nuevo, siempre igual:
  envía una solicitud, espera la respuesta, compruébala, continúa o ramifica. Cada ejecución
  se informa paso a paso y se guarda. Consulta [Experimentos](../experiments/index.md).

Las dos se encuentran en varios sitios. En las pantallas HTTP y OSC, [[ui:common.toExperiment]]
convierte lo que acabas de enviar en el siguiente paso del experimento. En el monitor OSC y
la pantalla MQTT, [[ui:osc.waitForThis]] convierte un mensaje que recibiste en un paso que
lo espera. Y una señal de la biblioteca puede convertirse en un paso — cualquiera salvo
bytes UDP sin formato escritos en hexadecimal.

## Señales y la biblioteca {#signals}

Una **señal** es un mensaje que guardas: un nombre, una carpeta, una nota sobre lo que
debería provocar y lo que envía — un mensaje OSC, bytes UDP sin formato, una solicitud HTTP
o una publicación MQTT. La **biblioteca de señales** las guarda en carpetas que puedes
anidar, renombrar y arrastrar.

- Guardas una señal desde las pantallas OSC, HTTP y MQTT ([[ui:sig.saveNew]]), creas una en
  la pantalla [[ui:nav.signals]], o guardas una trama que capturó el Inspector
  ([[ui:sig.fromFrame]]), que luego la reproduce byte a byte.
- La envías desde la pantalla [[ui:nav.signals]], desde cualquier lugar con
  <kbd>Ctrl</kbd>+<kbd>K</kbd>, como un paso de un experimento, o con `signallab fire` desde
  un terminal.
- Una señal envía exactamente lo que enviaría su pantalla: los mismos bytes, por el mismo
  camino, mostrados en el Inspector bajo su protocolo real.

La biblioteca es un solo archivo, `signals.json`, en la [carpeta de datos](#data-folder):
JSON sin más que puedes leer, editar, copiar a otra máquina o guardar en un repositorio.
Empieza con un conjunto de señales de ejemplo, todas apuntadas a `127.0.0.1`. Consulta
[Señales](../tools/signals.md).

## Tareas {#jobs}

Todo lo que sigue funcionando después de pulsar su botón es una **tarea**: un monitor o
generador OSC, una conexión con un bróker o WebSocket, una baliza o escucha de
descubrimiento, una ráfaga de carga HTTP, un emulador, un relé de degradación, una tormenta,
un escaneo, una ejecución de experimento.

- Cada tarea tiene una píldora en la franja del panel inferior, con su número, lo que es y
  un botón para detenerla. La barra lateral muestra cuántas tareas en curso tiene cada
  pantalla.
- [[ui:app.stopAll]] en la cabecera detiene todas las tareas a la vez.
- Una tarea que termina sola — un escaneo que acabó, una ejecución que se superó, un monitor
  cuyo puerto falló — deja su píldora, y la consola dice cómo terminó.
- Una tarea continúa mientras trabajas en otras pantallas.
- Instalar una actualización detiene antes todas las tareas.

En un servidor, las tareas pertenecen al servidor: cada página con sesión iniciada en él ve
las mismas tareas y puede detenerlas.

## Captura e Inspector {#capture}

Cada herramienta — emisores, monitores, escuchas, emuladores, relés, ejecuciones de
experimentos — entrega cada trama que envía o recibe a una **captura**, y el
[Inspector](../tools/inspector.md) la muestra en una sola línea de tiempo.

- La captura está **desactivada hasta que la activas** con [[ui:ins.arm]], y no cuesta nada
  mientras está desactivada. Sigue activada, estés en la pantalla que estés, hasta que la
  desactives.
- Guarda hasta 8192 tramas y 64 MiB de sus bytes; las tramas más antiguas dejan sitio a las
  nuevas. Cada trama guarda hasta 256 KiB de sus bytes, y la lista muestra su primer KiB.
- [[ui:ins.pause]] impide que la lista se mueva para que puedas leerla; la captura continúa
  por debajo.
- Los valores secretos que usa una ejecución se enmascaran en cada trama.
- Toda la captura se puede exportar, con cada byte guardado, a un archivo `.jsonl` o `.txt`.

## Emuladores {#emulators}

Un **emulador** interpreta el otro lado: la API, el dispositivo o el servicio con el que
habla tu sistema. Cada uno es un documento con un protocolo, la dirección en la que escucha
y reglas que dicen qué responder:

| Protocolo | Qué emula |
| --- | --- |
| HTTP | Una API: rutas por método y ruta de acceso, respuestas en secuencia, por turnos o al azar, con retardos y fallos |
| OSC | Un dispositivo que responde a mensajes OSC por dirección y argumentos |
| UDP | Un dispositivo que responde a datagramas por su carga útil |
| TCP | Un dispositivo que responde a líneas en una conexión TCP, con un saludo |
| MQTT | Un bróker que encamina lo que publican los clientes y responde según reglas como un dispositivo |

Un emulador puede responder despacio, fallar, cerrar la conexión, enviar un cuerpo mal
formado o caerse según un programa. Cada intercambio se cuenta, se enumera en su pantalla y
se captura para el Inspector.

Inicias uno desde la pantalla [[ui:nav.emulators]], donde funciona como una tarea; desde el
nodo [[ui:exp.node.emulator]] de un experimento, donde responde durante toda la ejecución; o
con `signallab emulate`. La **biblioteca de emuladores** es `emulators.json` en la carpeta
de datos. Empieza con un emulador de cada tipo, todos en `127.0.0.1`:

| Emulador | Escucha en | Qué hace |
| --- | --- | --- |
| [[ui:seed.emu.demo-api.name]] | `127.0.0.1:8080` (HTTP) | Una comprobación de estado, un usuario por id, una creación, una respuesta lenta y una ruta que falla dos veces antes de funcionar |
| [[ui:seed.emu.osc-device.name]] | `127.0.0.1:9100` (OSC) | Responde a `/ping` con `/pong` y un recuento, acusa `/fader/…` con `/ack`, acepta `/cue/…` sin decir nada |
| [[ui:seed.emu.udp-device.name]] | `127.0.0.1:7100` (UDP) | Responde a `PING` con `PONG` y un recuento, a cualquier otra cosa con cuántos bytes recibió |
| [[ui:seed.emu.tcp-device.name]] | `127.0.0.1:7200` (TCP) | Un protocolo por líneas como el de un proyector: saluda con `READY`, informa y conmuta la alimentación, dice `BYE` y cuelga ante `QUIT` |
| [[ui:seed.emu.mqtt-broker.name]] | `127.0.0.1:1883` (MQTT) | Un `lab/status` retenido, y una lámpara: `ON` u `OFF` publicado en `lab/<name>/set` se responde en `lab/<name>/state` |

Consulta [Emuladores](../tools/emulators.md).

## Relés de degradación {#impairment}

Un **relé de degradación** se sitúa entre un cliente y su destino. Apuntas el cliente a la
dirección de escucha del relé en lugar del destino real; el relé reenvía en ambos sentidos y
degrada lo que pasa, según un **perfil**:

- por **UDP**, cada datagrama corre su propia suerte: latencia y jitter, pérdida y ráfagas
  de pérdida, duplicación, corrupción, reordenación, un límite de ancho de banda, o nada en
  absoluto (sin conexión);
- por **TCP**, cada conexión se une a una propia hacia el destino, y ambos flujos se
  retrasan, se limitan a un ancho de banda, se reinician o se dejan medio abiertos.

Los preajustes fijan un perfil con un clic, desde un cable hasta un enlace por satélite. Un
cambio se aplica mientras el relé funciona, sin soltar su puerto. Cada decisión se extrae de
una semilla, así que el mismo tráfico corre la misma suerte otra vez.

En la pantalla [[ui:nav.netsim]] un relé funciona como una tarea. En un experimento, un nodo
[[ui:exp.node.impairment]] abre uno para la ejecución y [[ui:exp.node.impairment_change]]
cambia su perfil a mitad de la ejecución. Consulta [Degradación](../tools/impairment.md) y
[Fallos](../experiments/faults.md).

## Experimentos {#experiments}

### Nodos y cables {#nodes-and-wires}

Un experimento es un grafo de **nodos** unidos por **cables**. Cada nodo es un paso: envía
algo, espera algo, comprueba un valor, extrae uno, cambia el flujo o prepara la ejecución —
un emulador, un relé de degradación. Cada experimento tiene exactamente un
[[ui:exp.node.start]] y un [[ui:exp.node.end]], y admite hasta 64 nodos. El
experimento abierto en el editor se guarda mientras lo editas. Consulta
[Nodos](../experiments/nodes.md).

### Salidas {#outputs}

Un cable va de la **salida** de un nodo a la entrada de otro. La mayoría de los nodos
tienen una salida; otros eligen entre varias: [[ui:exp.yes]] y [[ui:exp.no]] para una
bifurcación, [[ui:exp.portMatched]] y [[ui:exp.portTimeout]] para una espera,
[[ui:exp.portBody]], [[ui:exp.portDone]] y [[ui:exp.portLimit]] para un bucle,
[[ui:exp.branch1]] y [[ui:exp.branch2]] para una rama paralela.

Una salida puede tener varios cables: cada uno corre como una rama propia, en paralelo, y un
[[ui:exp.node.join]] espera todos los cables que llegan a él. Solo el cuerpo de un
[[ui:exp.node.loop]] puede volver atrás; cualquier otro ciclo es un error. Consulta
[Flujo](../experiments/flow.md).

### Parámetros y perfiles {#parameters}

Un **parámetro** es un valor con nombre — un host, un puerto, un nombre de usuario — escrito
una vez en [[ui:exp.params]] y usado en cualquier campo como `{{name}}`. Un **perfil** cambia
algunos parámetros a la vez: uno para el portátil, uno para el escenario, uno para el local.
Eliges el perfil que usan las ejecuciones, o [[ui:exp.runWith]] un perfil, otros valores o
una semilla solo para una ejecución, sin cambiar el experimento. Un experimento admite hasta
64 parámetros y 32 perfiles. Consulta [Datos](../experiments/data.md).

### Plantillas {#templates}

La mayoría de los campos de texto de los nodos son **plantillas**: texto plano con
expresiones entre llaves dobles, que se rellenan mientras se ejecuta el paso.

- `{{host}}` — un parámetro, o una variable definida antes en la ejecución, como un valor
  que un nodo [[ui:exp.node.extract]] tomó de una respuesta, o una respuesta que recibió una
  espera (`{{reply.args[0]}}`).
- `{{secret.API_TOKEN}}` — un secreto.
- `{{run.id}}`, `{{run.seed}}`, `{{now}}`, `{{now.iso}}`, `{{counter}}` — la ejecución y el
  momento.
- `{{uuid}}`, `{{random_int(1, 10)}}`, `{{random_float(0, 1, 2)}}`, `{{pick("a", "b")}}`
  — valores generados.

Solo el motor rellena las plantillas, así que un campo significa lo mismo en una ejecución,
en la vista previa del editor y en [[ui:exp.sendNow]]. Un nombre desconocido es un error,
nunca una cadena vacía. Consulta [Datos](../experiments/data.md).

### Secretos {#secrets}

Un **secreto** es un valor que un experimento usa pero nunca guarda — un token, una
contraseña. El experimento solo guarda su nombre; los campos lo usan como
`{{secret.NAME}}`; y cada texto que informa una ejecución, cada paso, el informe y cada
trama del Inspector, lo muestra enmascarado. Ningún comando devuelve jamás el valor de un
secreto.

Dónde viven los valores depende de dónde funcione Signal Lab:

- **La aplicación de escritorio en Windows** los guarda en el Administrador de credenciales
  de Windows. Los defines en [[ui:exp.params]] → [[ui:exp.secrets]].
- **La aplicación de escritorio en Linux** no tiene ningún almacén de credenciales donde
  guardarlos, así que los experimentos que usan secretos se ejecutan desde la línea de
  comandos o un servidor allí.
- **Un servidor** los lee, en solo lectura, de su entorno (`SIGNALLAB_SECRET_<NAME>`) o de un
  archivo por nombre en su carpeta de secretos (`/run/secrets/signallab/<NAME>` de forma
  predeterminada); no se pueden definir desde un navegador.
- **La línea de comandos** los lee igual que un servidor, o del almacén de credenciales del
  sistema cuando se lo pides. Consulta [La línea de comandos](../automation/cli.md).

### Semillas {#seeds}

Cada ejecución tiene una **semilla**, un número que decide todo lo aleatorio que hay en
ella: valores generados, el jitter de una repetición, la elección aleatoria de respuesta de
un emulador, cada decisión de un relé de degradación. La misma semilla y el mismo tráfico
dan la misma ejecución. Se extrae una semilla nueva para cada ejecución a menos que el
experimento fije una — [[ui:exp.pinSeed]] en la línea de tiempo de la ejecución fija la
semilla de la última, y [[ui:exp.seed]] en [[ui:exp.params]] fija una.

### Ejecuciones e informes {#reports}

Una **ejecución** empieza en [[ui:exp.node.start]], sigue los cables y se supera cuando
llega a [[ui:exp.node.end]] sin que falle ningún paso. Se detiene si tarda más de 300
segundos. Cada paso aparece en la línea de tiempo de la ejecución al empezar y al terminar.

Una ejecución que termina, superada o fallida, escribe un **informe** en la carpeta `runs`
de la carpeta de datos: el nombre del experimento, la semilla, el perfil y los valores
usados, cuándo empezó y terminó, el resultado y su error, cada paso, y lo que contaron sus
emuladores y relés. Dos ejecuciones del mismo experimento se pueden comparar. Consulta
[Ejecuciones e informes](../experiments/runs.md).

## La carpeta de datos {#data-folder}

Todo lo que guarda Signal Lab es un archivo en una sola carpeta: `Documents/SignalLab` en tu
carpeta personal, tanto en Windows como en Linux. Un servidor guarda la suya, que eliges al
iniciarlo (`/data` en la imagen de Docker).

| Archivo o carpeta | Qué guarda |
| --- | --- |
| `experiment.json` | El experimento abierto en el editor |
| `signals.json` | La biblioteca de señales |
| `emulators.json` | La biblioteca de emuladores |
| `runs/` | Un informe por cada ejecución |
| `exports/` | Experimentos exportados desde el diálogo de experimentos |
| `capture-….jsonl`, `capture-….txt` | Exportaciones del Inspector |

Los archivos son JSON, escritos enteros. Si uno no se puede leer, Signal Lab dice qué
archivo y dónde está el error, y lo deja como está en lugar de empezar de cero. Consulta
[Archivos y carpetas](../reference/files.md).

## Escritorio y servidor {#desktop-and-server}

La aplicación de escritorio y un servidor ejecutan el mismo motor tras la misma interfaz. Lo
que cambia:

| | Aplicación de escritorio | Servidor, en un navegador |
| --- | --- | --- |
| Dónde empieza el tráfico, dónde escuchan los monitores | Este equipo | El servidor |
| Carpeta de datos | `Documents/SignalLab` | La del servidor; pasa el puntero por [[ui:app.server]] en la cabecera para verla |
| Secretos | Administrador de credenciales de Windows, definidos en la aplicación; ninguno en Linux | En solo lectura, del entorno del servidor o de archivos de secretos |
| Informes, exportaciones, capturas | Escritos en la carpeta de datos; se muestra la ruta | Los descarga el navegador |
| Iniciar sesión | — | Con el token de acceso del servidor, cuando lo tiene |
| Tareas, el almacén de cookies de la pantalla HTTP | Las de esta aplicación | Las del servidor, compartidas por cada página con sesión iniciada en él |
| Firewall | Un aviso ofrece permitir Signal Lab (Windows) | Signal Lab nunca lo cambia |
| Actualizaciones | Instala versiones firmadas cuando haces clic | Se actualiza con su imagen |

Consulta [El servidor](../server/index.md) y [Seguridad del servidor](../server/security.md).

## Lo que Signal Lab no hace por su cuenta {#on-its-own}

- **Solo envía cuando tú actúas**, y solo a las direcciones que escribes. Iniciar la
  aplicación no envía nada — salvo, en la aplicación de escritorio, la comprobación diaria
  de actualizaciones, que puedes desactivar. Los comentarios salen solo cuando envías el
  formulario.
- **Sus ejemplos se quedan en este equipo.** Las señales iniciales, los emuladores
  iniciales, los emuladores nuevos y las plantillas de experimentos usan todos
  `127.0.0.1`. Las escuchas que inicias — un monitor OSC, la escucha de descubrimiento, un
  relé de degradación — usan `0.0.0.0` de forma predeterminada, todas las tarjetas de red,
  para que otras máquinas puedan alcanzarlas; escribe `127.0.0.1` para mantener una en este
  equipo.
- **Solo cambia el firewall cuando haces clic en** [[ui:fw.allow]] y confirmas el aviso de
  administrador de Windows, o ejecutas `signallab firewall allow`. Un servidor nunca cambia
  el firewall de su host.
- **Un servidor sin token de acceso** escucha solo en `127.0.0.1`, y se niega a iniciarse en
  cualquier otra dirección.
- **Respeta las barandillas de protección**: un barrido de difusión llega a 1024 hosts como
  máximo, y una baliza envía como máximo 50 000 paquetes por segundo entre todos sus
  destinos.

Las barandillas de protección no son permiso: [Tormenta](../tools/storm.md), el
[Escáner](../tools/scanner.md) y [Difusión](../protocols/broadcast.md) envían tráfico real.
Úsalos solo en redes y hosts que sean tuyos o que estés autorizado a probar.

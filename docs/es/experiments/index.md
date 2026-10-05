---
title: El editor
description: Crea, edita y ejecuta experimentos en el lienzo de nodos — añadir y cablear nodos, seleccionar y copiar, el panel de propiedades, Enviar ahora, la validación, la ejecución y los archivos que hay detrás.
---

# El editor de experimentos

Un experimento es una prueba escrita como un grafo: nodos que envían (una solicitud HTTP,
un mensaje OSC, una publicación MQTT…), esperan una respuesta, comprueban lo que llegó,
hacen el otro papel o rompen la red a propósito, unidos por cables que dicen qué se ejecuta
después. Una ejecución empieza en [[ui:exp.node.start]], sigue los cables y se supera cuando
llega a [[ui:exp.node.end]] con todos los pasos superados. Lo construyes en la pantalla
[[ui:nav.experiment]], lo ejecutas ahí y ejecutas el mismo archivo desde la línea de
comandos o un servidor.

El editor contiene un experimento a la vez y lo guarda mientras trabajas. Para conservar
varios, exporta instantáneas o abre otro desde un archivo (consulta
[Guardar y archivos](#files)).

Cada tipo de nodo, con sus campos y sus salidas, está en la
[referencia de los nodos](nodes.md). Cómo fluyen los valores entre nodos está en
[Datos y plantillas](data.md); las ramas paralelas, los bucles, los reintentos y las
repeticiones están en [Flujo](flow.md).

## La pantalla {#screen}

| Zona | Qué contiene |
| --- | --- |
| Barra de herramientas (arriba) | El nombre del experimento y su estado de guardado, añadir nodos, parámetros, perfiles, los paneles, el modo de enfoque, la pantalla completa y Ejecutar |
| Barra del lienzo | Deshacer y rehacer, el buscador de nodos, [[ui:exp.arrange]] y el zoom |
| Lienzo | El grafo: los nodos, los cables y el progreso de la ejecución dibujado sobre ellos |
| [[ui:exp.properties]] (derecha) | Los campos del nodo seleccionado, su vista previa y [[ui:exp.sendNow]]; o lo que permiten varios nodos seleccionados o un cable seleccionado |
| [[ui:exp.timeline]] (abajo) | Los pasos de la ejecución actual o de la última, su resultado, su informe y su semilla |

El panel de propiedades y la línea de tiempo tienen un tirador en su borde interior:
arrástralo, o dale el foco con <kbd>Tab</kbd> y usa las teclas de flecha (<kbd>Shift</kbd>
para pasos mayores); un doble clic o <kbd>Enter</kbd> devuelve al panel su tamaño
predeterminado. Los tamaños se conservan para la próxima vez. La
línea de tiempo permanece plegada hasta que la abre la primera ejecución.

## La barra de herramientas {#toolbar}

| Control | Qué hace |
| --- | --- |
| ☰ [[ui:exp.documents]] | Abre las plantillas, abrir un archivo JSON y exportar ([Guardar y archivos](#files)) |
| [[ui:exp.name]] | El nombre del experimento; una ejecución necesita uno. Los informes de ejecución lo registran, y [[ui:exp.compare]] encuentra ejecuciones anteriores por él |
| [[ui:exp.saved]] / [[ui:exp.saving]] / [[ui:exp.saveError]] | Si el último cambio está en el disco |
| ⚠ [[ui:exp.needsLinks]] | Se muestra mientras el experimento no puede ejecutarse; su descripción emergente dice por qué, y un clic selecciona el nodo al que se refiere ([Validación](#validation)) |
| ＋ [[ui:exp.addNode]] | Abre el menú de añadir, después del nodo seleccionado cuando hay uno (<kbd>A</kbd>) |
| [[ui:exp.profile]] | El perfil que usan las ejecuciones, la vista previa y Enviar ahora; se muestra cuando el experimento tiene perfiles. ⚠ marca un perfil que no se ejecutaría |
| `{ }` [[ui:exp.params]] | Parámetros, perfiles, la semilla, las cookies y los secretos ([Datos y plantillas](data.md)) |
| ☷ [[ui:exp.properties]] | Muestra u oculta el panel de propiedades |
| ▢ [[ui:exp.focus]] | Oculta la barra lateral, la cabecera y el panel inferior ([Modo de enfoque y pantalla completa](#focus)) |
| ⛶ [[ui:exp.fullscreen]] | La ventana en pantalla completa, con modo de enfoque |
| [[ui:exp.run]] | Comprueba, guarda y ejecuta el experimento; mientras se ejecuta, el botón es [[ui:common.stop]] |
| ▾ [[ui:exp.runWith]] | Una ejecución con otro perfil, otros valores de parámetros o una semilla dada, sin cambiar el experimento |

## Moverse por el lienzo {#canvas}

- **Desplazar**: arrastra el lienzo vacío, o usa la rueda.
- **Zoom**: <kbd>Ctrl</kbd> y la rueda del ratón acercan y alejan alrededor del puntero;
  − y ＋ en la barra del lienzo van de 10 % en 10 %. El zoom va del 15 % al 200 %; el
  porcentaje entre los botones indica dónde estás.
- **1:1** ([[ui:exp.resetZoom]], <kbd>Ctrl</kbd>+<kbd>1</kbd>) vuelve al 100 %.
- ⊡ [[ui:exp.fit]] (<kbd>Ctrl</kbd>+<kbd>0</kbd>) muestra todo el grafo, como máximo al
  100 %.
- **Buscar un nodo**: [[ui:exp.nodes]] en la barra del lienzo (muestra cuántos nodos
  hay) o <kbd>Ctrl</kbd>+<kbd>F</kbd>. Escribe palabras del nombre del nodo,
  de su resumen (una URL, una dirección, un tema) o de su id; <kbd>↑</kbd>
  <kbd>↓</kbd> eligen, <kbd>Enter</kbd> selecciona el nodo y lo trae a la
  vista (con un zoom de al menos el 80 %), y <kbd>Esc</kbd> cierra.

No hay minimapa: [[ui:exp.fit]] y el buscador hacen su trabajo.

## Nodos y cables {#nodes-and-wires}

Un nodo muestra su tipo, un resumen de una línea de lo que hace y distintivos para sus
ajustes: ↻ y un número para Reintentar, × y un recuento (o un tiempo) para Repetir, ⚡
para Carga. Su entrada está a la izquierda —todos los nodos menos [[ui:exp.node.start]]
tienen una— y sus salidas a la derecha:

| Salida | En | Se sigue cuando |
| --- | --- | --- |
| [[ui:exp.outputPort]] | La mayoría de los nodos | El paso se superó |
| [[ui:exp.yes]] / [[ui:exp.no]] | [[ui:exp.node.branch_status]], [[ui:exp.node.branch_value]] | La comparación se cumplió / no se cumplió |
| [[ui:exp.branch1]] / [[ui:exp.branch2]] | [[ui:exp.node.fork]] | Siempre, ambas a la vez |
| [[ui:exp.portMatched]] / [[ui:exp.portTimeout]] | Esperas | Llegó un mensaje que coincide / no llegó ninguno a tiempo |
| [[ui:exp.portBody]] / [[ui:exp.portDone]] / [[ui:exp.portLimit]] | [[ui:exp.node.loop]] | Otra iteración / el bucle terminó / se agotaron las iteraciones |

**Una salida puede tener varios cables.** El primer cable continúa la rama;
cada cable siguiente inicia una rama paralela con una copia de las variables conocidas
en ese punto. Un nodo [[[ui:exp.node.join]]](nodes.md#node-join) espera todos los cables que
llegan a él. Los detalles están en [Flujo](flow.md).

[[ui:exp.portTimeout]] y [[ui:exp.portLimit]] son opcionales: si se dejan sin cable, un
tiempo de espera agotado o unas iteraciones agotadas hacen fallar el paso. Cualquier otra
salida debe tener un cable para que el experimento pueda ejecutarse.

El cable desde el cuerpo de un [[ui:exp.node.loop]] de vuelta a él se dibuja como un arco
por encima del cuerpo; es el único cable que puede ir hacia atrás.

## Añadir nodos {#adding}

### El menú de añadir {#add-menu}

El menú de añadir enumera cada tipo de nodo por grupo —[[ui:exp.group.action]],
[[ui:exp.group.observe]], [[ui:exp.group.emulate]], [[ui:exp.group.fault]],
[[ui:exp.group.data]], [[ui:exp.group.check]], [[ui:exp.group.flow]]— y
luego [[ui:exp.group.signals]]: las señales de tu [biblioteca de señales](../tools/signals.md)
que pueden convertirse en un nodo (OSC, HTTP, UDP con una carga útil de texto, MQTT), con sus
campos ya rellenos.

Su campo de búsqueda tiene el foco al abrirse. Escribe unas letras: cada palabra
debe aparecer en el nombre del nodo, en su descripción o en su tipo (`http`,
`wait_osc`); una señal se encuentra por su nombre, su carpeta, su transporte o su destino.
<kbd>↑</kbd> <kbd>↓</kbd> eligen, <kbd>Enter</kbd> añade y <kbd>Esc</kbd>
cierra. La línea de arriba dice dónde va el nodo: después de un nodo, o en
paralelo a uno.

El nodo nuevo queda seleccionado, se abre el panel de propiedades y su campo
principal —la URL, la dirección, el tema, el retardo— tiene el foco con su texto
seleccionado, para que escribas directamente. <kbd>Esc</kbd> en un campo te devuelve
al nodo en el lienzo, listo para la siguiente <kbd>A</kbd>.

### Después del nodo seleccionado {#add-after}

<kbd>A</kbd>, ＋ [[ui:exp.addNode]] en la barra de herramientas y ＋ [[ui:exp.addNext]]
en el panel de propiedades añaden el paso siguiente después del nodo seleccionado:

- Se conecta a la primera salida del nodo que aún no tiene cable (el
  [[ui:exp.no]] libre de un [[ui:exp.node.branch_status]], por ejemplo), o, si no, a su
  primera salida.
- Si esa salida ya tiene un cable, el nodo nuevo se inserta en él (en el
  primero, si tiene varios): el cable pasa ahora por el nodo nuevo, y
  todo lo que va después se desplaza a la derecha para hacer sitio.
- Con [[ui:exp.node.end]] seleccionado, el nodo va antes que él, cuando un único
  cable llega hasta él.
- Sin nada seleccionado, el nodo aparece en el centro de la vista, sin
  cables.

Un nodo con una sola salida que se coloca en el [[ui:exp.portBody]] vacío de un
[[ui:exp.node.loop]] —de esta forma o [en un cable nuevo](#add-branch)— también se cablea
de vuelta a él, así que el cuerpo queda completo de inmediato.

### Dentro de un cable {#insert}

Cada cable tiene un ＋ en su punto medio ([[ui:exp.insertNode]]). Abre el menú de añadir,
y el nodo que eliges se inserta en ese cable. Un [[ui:exp.node.loop]] insertado así
continúa el flujo por su [[ui:exp.portDone]].

### En un cable nuevo {#add-branch}

Arrastra un cable desde una salida y suéltalo en el lienzo vacío: el menú de añadir
se abre ahí, y el nodo nuevo va en un cable nuevo de esa salida —junto a los
cables que ya tiene, así que se ejecuta en paralelo con ellos—. Lo mismo ocurre cuando
haces clic en una salida y luego en el lienzo vacío, o cuando haces doble clic en ella.

### En cualquier parte {#add-anywhere}

Haz doble clic en el lienzo vacío para añadir un nodo en ese punto, sin cables.

### Desde otras pantallas {#from-screens}

- [[ui:common.toExperiment]] en las pantallas [OSC](../protocols/osc.md) y
  [HTTP](../protocols/http.md) añade lo que acabas de probar como paso
  siguiente —justo antes de [[ui:exp.node.end]], cuando un único cable llega hasta él— y
  cambia al editor.
- [[ui:osc.waitForThis]] junto a un mensaje del monitor OSC añade una
  [[ui:exp.node.wait_osc]] que lo reconoce: su dirección y sus argumentos de texto, de
  número entero y de verdadero/falso (los float son medidas que cambian, así que se
  omiten). Detén el monitor antes de ejecutar: la ejecución escucha en ese puerto por sí misma.
- [[ui:mq.waitForThis]] en un tema del árbol [MQTT](../protocols/mqtt.md)
  añade una [[ui:exp.node.wait_mqtt]] en ese tema, en ese bróker.

Mientras hay una ejecución en curso, no se pueden añadir nodos; la consola lo indica.

## Conectar nodos {#connecting}

- **Arrastra** desde una salida hasta un nodo. Basta con soltar a menos de 24 píxeles
  de un nodo.
- **Haz clic** en una salida (o dale el foco y pulsa <kbd>Enter</kbd> o
  <kbd>Space</kbd>): la barra del lienzo indica [[ui:exp.chooseInput]]. Haz clic en el nodo
  o en su entrada para conectar; haz clic en el lienzo vacío para añadir allí un nodo en un
  cable nuevo; [[ui:exp.cancelLink]] o <kbd>Esc</kbd> lo cancela.

Un cable se rechaza cuando crearía un ciclo (que no sea el camino de vuelta de un
[[ui:exp.node.loop]]), cuando llega a [[ui:exp.node.start]] o sale de
[[ui:exp.node.end]], o cuando conectaría un nodo consigo mismo; el editor lo
indica. Dibujar un cable que ya existe no cambia nada.

Para **quitar un cable**, haz clic en él —el panel de propiedades muestra qué
conecta— y pulsa <kbd>Delete</kbd>, o usa [[ui:exp.removeWire]] allí. Pasar el puntero
por encima de un cable también muestra una × encima de su ＋. Las propiedades de un nodo
enumeran sus cables salientes, cada uno con × ([[ui:exp.disconnect]]).

## Seleccionar varios nodos {#selection}

| Para | Haz |
| --- | --- |
| Seleccionar un nodo | Haz clic en él, o muévete hasta él con <kbd>Tab</kbd> |
| Añadir un nodo a la selección, o quitarlo | <kbd>Shift</kbd> o <kbd>Ctrl</kbd> y un clic |
| Seleccionar con un marco | <kbd>Shift</kbd> y arrastra sobre el lienzo vacío: todos los nodos que toca el marco se unen a la selección |
| Seleccionar todos los nodos | <kbd>Ctrl</kbd>+<kbd>A</kbd> |
| Borrar la selección | Haz clic en el lienzo vacío; <kbd>Esc</kbd> cuando hay varios seleccionados |
| Moverlos | Arrastra uno de ellos: se mueven todos |
| Desplazarlos un poco | Con un nodo enfocado, las teclas de flecha mueven la selección 5 píxeles, 20 con <kbd>Shift</kbd> |

El panel de propiedades muestra el último nodo elegido; con varios seleccionados,
muestra cuántos son, con [[ui:exp.copy]], [[ui:exp.duplicate]] y
[[ui:exp.deleteSelected]].

### Copiar, cortar y pegar {#copy-paste}

<kbd>Ctrl</kbd>+<kbd>C</kbd> copia como texto los nodos seleccionados y los cables entre
ellos; <kbd>Ctrl</kbd>+<kbd>X</kbd> además los quita;
<kbd>Ctrl</kbd>+<kbd>V</kbd> los pega —en este experimento, en otro
que abras después, o en otra ventana de Signal Lab—. El texto es JSON, así que
también puedes guardarlo en un archivo o en un mensaje.

- [[ui:exp.node.start]] y [[ui:exp.node.end]] son únicos: nunca se
  copian.
- Los cables entre un nodo copiado y el resto del grafo no se copian; cablea
  tú mismo la copia.
- Cada nodo pegado recibe un id nuevo.
- Un nodo que nombra a otro —[[ui:exp.node.impairment_change]],
  [[ui:exp.node.emulator_state]], [[ui:exp.node.ws_send]],
  [[ui:exp.node.wait_ws]], [[ui:exp.node.ws_close]]— nombra a la copia cuando esta también se copió;
  si no, sigue nombrando al original si está en este experimento, o al
  primer nodo de ese tipo que haya aquí.
- Un [[ui:exp.node.emulator]] o un [[ui:exp.node.impairment]] pegados cuyo puerto
  ya escucha este experimento pasan al siguiente puerto libre. Lo que envía al
  original sigue haciéndolo.
- La copia aparece 32 píxeles a la derecha y una fila por debajo de donde estaba,
  más abajo hasta no cubrir ningún nodo, y queda seleccionada.

<kbd>Ctrl</kbd>+<kbd>D</kbd> ([[ui:exp.duplicate]]) hace lo mismo sin el
portapapeles.

### Eliminar {#deleting}

<kbd>Delete</kbd> o <kbd>Backspace</kbd> ([[ui:exp.delete]],
[[ui:exp.deleteSelected]]) quitan los nodos seleccionados y sus cables. Un nodo
que tenía exactamente un cable de entrada y otro de salida deja un cable en su lugar, del
nodo anterior al siguiente, así que una cadena sigue conectada. [[ui:exp.node.start]]
y [[ui:exp.node.end]] no se pueden eliminar.

## Deshacer y rehacer {#undo}

<kbd>Ctrl</kbd>+<kbd>Z</kbd> deshace; <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd>
o <kbd>Ctrl</kbd>+<kbd>Y</kbd> rehacen (↶ ↷ en la barra del lienzo). Un arrastre, una
serie de desplazamientos con las flechas o lo que se escribe en un campo son un solo paso. Los últimos
100 pasos se conservan mientras la aplicación está abierta, aunque cambies de pantalla; abrir
otro experimento también es un paso, así que <kbd>Ctrl</kbd>+<kbd>Z</kbd> recupera
el anterior. Deshacer y rehacer esperan mientras hay una ejecución en curso.

## Organizar {#arrange}

[[ui:exp.arrange]] coloca el grafo de izquierda a derecha: cada nodo en la
columna siguiente al último nodo que lleva a él, el cuerpo de un [[ui:exp.node.loop]]
en su fila —y luego ajusta la vista—. Es un paso del historial. Un borrador
con un ciclo que no es el de un bucle se deja tal cual.

## El panel de propiedades {#properties}

Con un nodo seleccionado, el panel muestra, de arriba abajo:

1. El nombre del nodo (su descripción emergente dice lo que hace) y, cuando el experimento
   no puede ejecutarse por culpa de este nodo, qué va mal.
2. Sus campos. Los campos que aceptan [plantillas](data.md) sugieren parámetros,
   variables, secretos y generadores mientras escribes `{{`, o con
   <kbd>Ctrl</kbd>+<kbd>Space</kbd>.
3. Sus ajustes: [[ui:exp.loadOn]] en una solicitud HTTP, [[ui:exp.repeatOn]] en
   los nodos que envían, [[ui:exp.retryOn]] en los nodos que envían o escuchan, y
   [[ui:exp.expectReply]] en los mensajes OSC y UDP (consulta [Ajustes de los
   nodos](nodes.md#settings)). La carga sustituye a Repetir y Reintentar mientras está activada.
4. El resultado de carga de la última ejecución, en un nodo HTTP que se ejecutó bajo carga.
5. La vista previa —[[ui:exp.preview]], [[ui:exp.previewWait]] o
   [[ui:exp.previewCheck]]— cuando el nodo tiene plantillas: lo que enviaría,
   esperaría o compararía, resuelto con los parámetros actuales y los valores
   conocidos hasta el momento ([Enviar ahora y la vista previa](#send-now)).
6. [[ui:exp.sendNow]] o [[ui:exp.listenNow]], y lo que hizo la última vez.
7. Sus cables salientes, cada uno con ×.
8. ⚡ [[ui:exp.routeThrough]] en un mensaje OSC o UDP: se coloca un [[ui:exp.node.impairment]]
   delante del nodo —escuchando en un puerto libre de loopback a partir del 9010 y
   reenviando al destino del nodo— y el nodo se apunta a él, así que la
   siguiente ejecución degrada lo que envía ([Fallos](faults.md)).
9. ＋ [[ui:exp.addNext]], [[ui:exp.copy]], [[ui:exp.duplicate]] y
   [[ui:exp.delete]].

Un doble clic en un nodo abre el panel con el cursor en su campo principal.
Con varios nodos seleccionados, el panel ofrece lo que se puede hacer con todos ellos;
con un cable seleccionado, qué conecta y [[ui:exp.removeWire]]. Mientras hay una ejecución
en curso, los campos están bloqueados.

## Enviar ahora y la vista previa {#send-now}

[[ui:exp.sendNow]] (<kbd>Ctrl</kbd>+<kbd>Enter</kbd>, también desde dentro de los
campos del nodo) envía el nodo seleccionado por sí solo, sin ejecutar el
experimento, a través del mismo código que usa una ejecución. Se ofrece en
[[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]],
[[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_connect]] y
[[ui:exp.node.ws_send]].
En una espera es [[ui:exp.listenNow]]: la espera escucha desde ahora hasta que
coincida un mensaje o termine su tiempo de espera.

- El nodo usa el perfil activo, los secretos guardados y los valores de las variables
  conocidos hasta el momento —de la última ejecución y de resultados anteriores de [[ui:exp.sendNow]]—.
  Si una plantilla nombra un valor que aún no ha definido nadie, no se envía nada y se
  enumeran los nombres.
- Se envía una sola vez: Reintentar, Repetir y Carga no se aplican, no se guardan cookies,
  y los emuladores y relés del experimento no se inician.
- Un [[ui:exp.node.ws_send]] o un [[ui:exp.node.wait_ws]] abre la conexión que
  describe su [[ui:exp.node.ws_connect]], para esa única prueba.
- En una solicitud HTTP se muestra la respuesta —el estado, el tiempo, el tamaño y el cuerpo,
  con formato JSON—. Los valores que tomarían los nodos [[ui:exp.node.extract]] después de la
  solicitud se rellenan de inmediato. Haz clic en un valor de una respuesta JSON para
  añadir justo después de la solicitud un nodo [[ui:exp.node.extract]] para él, con su
  variable nombrada y su valor conocido.
  [[ui:http.mockThis]] convierte la respuesta en una ruta de un
  [emulador](../tools/emulators.md).
- El resultado también se escribe en la consola.

La vista previa la resuelve el motor igualmente, alrededor de un cuarto de segundo
después de un cambio. Los valores de los secretos nunca se muestran, ni ahí ni en ningún sitio.

## Validación {#validation}

El editor comprueba el experimento mientras lo editas, y una vez más cuando pulsas
[[ui:exp.run]]:

- Una salida que aún necesita un cable parpadea en ámbar.
- Un nodo al que no llega ningún cable desde [[ui:exp.node.start]] se dibuja con línea discontinua; su descripción emergente lo indica.
- El nodo al que se refiere un problema se resalta con un contorno, y el problema se escribe encima de
  sus campos.
- ⚠ [[ui:exp.needsLinks]] en la barra de herramientas nombra el problema; un clic selecciona
  el nodo.

Un experimento se ejecuta cuando, entre otras cosas:

- tiene un nombre, exactamente un [[ui:exp.node.start]] y un
  [[ui:exp.node.end]], y como máximo 64 nodos;
- todos los nodos son alcanzables desde [[ui:exp.node.start]] y cada salida obligatoria tiene un cable;
- los únicos ciclos son los cuerpos de los nodos [[ui:exp.node.loop]] que vuelven a
  ellos;
- cada comprobación y cada [[ui:exp.node.extract]] tienen una solicitud HTTP antes, en todos los caminos (uno
  bajo carga no cuenta: no deja respuesta);
- un [[ui:exp.node.ws_send]], [[ui:exp.node.wait_ws]] o [[ui:exp.node.ws_close]]
  va después del [[ui:exp.node.ws_connect]] que usa;
- cada campo está relleno y dentro de rango, y cada plantilla nombra algo
  conocido en ese punto.

Los borradores sin terminar se guardan igualmente. Una ejecución se rechaza, antes de cualquier
tráfico, cuando un secreto que necesita no está guardado. Cada mensaje se enumera en la
[referencia de errores](../reference/errors.md).

## Ejecutar {#running}

[[ui:exp.run]] comprueba el experimento, lo guarda y lo inicia. Si no puede
ejecutarse, se muestra el problema y se selecciona su nodo. Si no, se abre la línea de tiempo
y los nodos se iluminan a medida que la ejecución los alcanza: ● en curso, ✓ superado, ✕ fallido,
↻ reintentando, ⟳ repitiendo, ⚡ bajo carga; los cables que llevaron el flujo también se
colorean.

- Las esperas, los emuladores y los relés de degradación abren sus puertos antes del primer
  paso, así que no se pierde nada que llegue temprano. Un puerto que no se puede abrir
  (porque lo tiene otro programa, por ejemplo) impide que la ejecución empiece, y se muestra
  el nodo que lo necesita.
- [[ui:common.stop]] termina la ejecución al instante: pausas, esperas y cargas incluidas.
  La ejecución también es una tarea en la franja de tareas de la consola, que también puede detenerla.
- Una ejecución que tarda más de 300 segundos se detiene y falla.
- Mientras hay una ejecución en curso, el experimento no se puede editar; desplazar, hacer zoom y
  seleccionar siguen funcionando.

▾ [[ui:exp.runWith]] junto al botón ejecuta una vez con otro perfil, otros
valores de parámetros o una semilla dada; el experimento en sí no cambia, y el
formulario conserva lo que escribiste hasta que se cierra la aplicación ([Datos y
plantillas](data.md)).

### La línea de tiempo de la ejecución {#timeline}

La [[ui:exp.timeline]] enumera una fila por paso: la hora, el nodo y lo que
ocurrió —superado, fallido y por qué, un reintento, el progreso de una repetición, las
cifras de una carga una vez por segundo—. Haz clic en una fila para seleccionar su nodo en el lienzo.

Su línea de título contiene:

- el resultado: [[ui:exp.passed]], [[ui:exp.failed]] con el motivo, o
  [[ui:exp.stopped]];
- [[ui:exp.reportSaved]] —el archivo de informe de la ejecución (en un servidor, una descarga)—;
- [[ui:exp.compare]] —esta ejecución junto a otra anterior del mismo experimento—;
- el perfil y los valores cambiados que usó la ejecución, cuando tenía alguno;
- la semilla de la ejecución, con [[ui:exp.pinSeed]] para conservarla en el experimento de modo que las
  siguientes ejecuciones saquen los mismos valores aleatorios, o [[ui:exp.unpinSeed]] una vez
  fijada.

Cuando el [Inspector](../tools/inspector.md) estaba capturando, una espera (o una
respuesta esperada) que coincidió enlaza con la trama con la que coincidió; un clic la abre en
el Inspector. Los informes, las semillas y la comparación de ejecuciones están en [Ejecuciones e
informes](runs.md).

## Modo de enfoque y pantalla completa {#focus}

▢ [[ui:exp.focus]] oculta la barra lateral, la cabecera y el panel inferior (la consola
y el Inspector), dejando la pantalla al editor. ⛶ [[ui:exp.fullscreen]] pone
la ventana en pantalla completa y activa el modo de enfoque; salir de la pantalla completa devuelve el modo de
enfoque a como estaba. Cambiar a otra pantalla abandona el modo de enfoque.

<kbd>Esc</kbd> en el lienzo, cuando no hay nada más que cerrar, sale de la pantalla completa y
luego del modo de enfoque.

## Guardar y archivos {#files}

El experimento se guarda solo, un momento después de cada cambio, en
`experiment.json` dentro de la carpeta de datos (`Documents/SignalLab` en una aplicación de escritorio; un
servidor tiene la suya —consulta [Archivos y carpetas](../reference/files.md)). Los borradores
que aún no pueden ejecutarse también se guardan. Una ejecución guarda primero, así que lo que se ejecutó es lo que está en
el disco.

Si ese archivo no se puede leer —editado a mano hasta dejarlo en un JSON roto, por ejemplo—, el editor
dice qué archivo es y por qué, y no lo toca. Abrir otro experimento desde
☰ [[ui:exp.documents]] lo sustituye entonces en el siguiente guardado.

☰ [[ui:exp.documents]] abre el diálogo de experimentos:

- [[ui:exp.templates]]: elige una y pulsa [[ui:exp.openDocument]]. Todas
  usan direcciones de loopback.
- [[ui:exp.importJson]] lee un archivo de experimento —escrito por esta versión de
  Signal Lab o por una anterior, de hasta 4 MiB— y muestra su nombre y cuántos
  nodos y conexiones tiene antes de que lo abras. Los archivos de versiones anteriores
  se ponen al día al abrirse. Un archivo que no se puede analizar, o que no sería
  un experimento válido, se rechaza con el motivo, y el experimento actual
  se queda.
- [[ui:exp.exportJson]] escribe una instantánea del experimento actual —nodos,
  cables, posiciones, parámetros y perfiles, nunca valores de secretos— en un archivo nuevo
  dentro de `exports`, en la carpeta de datos, y muestra su ruta; en un servidor, con un
  enlace [[ui:common.download]].
- [[ui:exp.openDocument]] sustituye el experimento actual por la plantilla o
  el archivo. No lo ejecuta, y <kbd>Ctrl</kbd>+<kbd>Z</kbd> recupera el anterior.

| Plantilla | Qué hace |
| --- | --- |
| [[ui:exp.templateEmpty]] | [[ui:exp.node.start]] y [[ui:exp.node.end]], para tu propio flujo |
| [[ui:exp.templateHttp]] | Un GET a `http://127.0.0.1:8080/` y una comprobación del estado 200: el experimento con el que empiezas |
| [[ui:exp.templateBranch]] | La misma solicitud; con 200, un mensaje OSC a `127.0.0.1:9000`, y si no, un retardo de 500 ms |
| [[ui:exp.templateParallel]] | Dos ramas a la vez —una solicitud y una línea de registro— unidas antes de [[ui:exp.node.end]] |
| [[ui:exp.templatePingReply]] | Envía `/ping` con el id de la ejecución a `127.0.0.1:9000` y espera en `127.0.0.1:9001` un `/pong` que lo devuelva |
| [[ui:exp.templatePoll]] | Pregunta a un dispositivo por `/status` cada 0,3 s hasta que responde `ready`, como máximo diez veces |
| [[ui:exp.templateFlaky]] | Una API emulada que falla dos veces antes de responder, y un bucle que pregunta hasta que responde |
| [[ui:exp.templateFaults]] | Datagramas a un dispositivo emulado a través de un relé de degradación mientras una rama paralela cambia la red a limpia, con pérdidas, sin conexión y limpia otra vez |
| [[ui:exp.templateOutage]] | Una API emulada caída durante dos segundos por una rama paralela, y un cliente que sigue preguntando hasta que vuelve a responder |
| [[ui:exp.templateWsEcho]] | Se conecta a un servicio de eco en `ws://127.0.0.1:9001/echo`, envía un ping JSON, espera recibirlo sin cambios y se desconecta |

Los mismos archivos se ejecutan sin el editor: `signallab run experiment.json` —consulta
[La línea de comandos](../automation/cli.md).

## Atajos de teclado {#shortcuts}

Las teclas sueltas actúan sobre el lienzo y no se tocan mientras escribes en un campo. En
un Mac, en un navegador, <kbd>Cmd</kbd> funciona donde se escribe <kbd>Ctrl</kbd>.

| Teclas | Acción |
| --- | --- |
| <kbd>A</kbd> | Añade un nodo después del seleccionado, o en el centro de la vista cuando no hay nada seleccionado |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:exp.sendNow]], o [[ui:exp.listenNow]], en el nodo seleccionado |
| <kbd>Ctrl</kbd>+<kbd>Z</kbd> | Deshacer |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd>, <kbd>Ctrl</kbd>+<kbd>Y</kbd> | Rehacer |
| <kbd>Ctrl</kbd>+<kbd>A</kbd> | Seleccionar todos los nodos |
| <kbd>Ctrl</kbd>+<kbd>C</kbd> / <kbd>Ctrl</kbd>+<kbd>X</kbd> / <kbd>Ctrl</kbd>+<kbd>V</kbd> | Copiar / cortar / pegar los nodos seleccionados y los cables entre ellos |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | Duplicar los nodos seleccionados |
| <kbd>Delete</kbd>, <kbd>Backspace</kbd> | Quitar el cable seleccionado, o los nodos seleccionados |
| Teclas de flecha (con un nodo enfocado) | Mueven los nodos seleccionados 5 píxeles; con <kbd>Shift</kbd>, 20 |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | Buscar un nodo |
| <kbd>Ctrl</kbd>+<kbd>0</kbd> | Ajustar el grafo a la vista |
| <kbd>Ctrl</kbd>+<kbd>1</kbd> | Zoom al 100 % |
| <kbd>Ctrl</kbd> + rueda del ratón | Zoom alrededor del puntero |
| <kbd>Shift</kbd> + clic, <kbd>Ctrl</kbd> + clic | Añadir un nodo a la selección, o quitarlo |
| <kbd>Shift</kbd> + arrastrar sobre el lienzo vacío | Seleccionar con un marco |
| Doble clic en un nodo | Editar sus campos |
| Doble clic en el lienzo vacío | Añadir un nodo ahí |
| <kbd>Enter</kbd>, <kbd>Space</kbd> en una salida enfocada | Iniciar un cable desde ella |
| `{{` o <kbd>Ctrl</kbd>+<kbd>Space</kbd> en un campo | Sugerir parámetros, variables, secretos y generadores |
| <kbd>Esc</kbd> en un campo | Volver al nodo en el lienzo |
| <kbd>Esc</kbd> en el lienzo | Cierra el menú de añadir; si no, cancela el cable que se está dibujando; si no, suelta el cable seleccionado; si no, borra una selección de varios; si no, sale de la pantalla completa; si no, sale del modo de enfoque |

En el menú de añadir y el buscador, <kbd>↑</kbd> <kbd>↓</kbd> eligen,
<kbd>Enter</kbd> toma la opción y <kbd>Esc</kbd> cierra. Todos los atajos de la aplicación
están en [Atajos de teclado](../reference/shortcuts.md).

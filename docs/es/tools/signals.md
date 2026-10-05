---
title: Señales
description: Guarda mensajes OSC, UDP, HTTP y MQTT en una biblioteca de carpetas, y vuelve a enviarlos desde la pantalla Señales, desde cualquier pantalla con Ctrl+K o desde la línea de comandos.
---

# Señales

Una señal es un mensaje que has nombrado y guardado: un mensaje OSC, un datagrama
UDP sin procesar, una solicitud HTTP o una publicación MQTT, con su destino. La
construyes una vez —en la pantalla [[ui:nav.signals]], o guardando lo que acabas
de enviar desde otra pantalla— y la vuelves a enviar cuando la necesitas, byte
por byte idéntica.

La biblioteca es un archivo JSON que puedes leer, editar a mano, copiar a otra
máquina o guardar junto a un proyecto. La pantalla [[ui:nav.signals]] la muestra
como un árbol de carpetas a la izquierda y los campos de la señal seleccionada a
la derecha.

## Qué puede enviar una señal {#transports}

Elige el tipo en [[ui:sig.transport]]. Cada tipo tiene sus propios campos:

| [[ui:sig.transport]] | Campos | Qué sale |
| --- | --- | --- |
| [[ui:sig.tr.osc]] | [[ui:common.target]], [[ui:common.address]], [[ui:common.arguments]] | Un mensaje OSC a un `IP:port` o `host:port`, desde un puerto UDP nuevo. Consulta [OSC](../protocols/osc.md#types) para los tipos de argumento. |
| [[ui:sig.tr.udp]] | [[ui:common.target]], [[ui:sig.payloadKind]] ([[ui:sig.payloadText]] o [[ui:sig.payloadHex]]), [[ui:sig.payload]] | Un datagrama con exactamente estos bytes. El texto va tal cual, sin terminador; el hex es pares de dígitos como `de ad be ef` (se permiten espacios y un prefijo `0x`). |
| [[ui:sig.tr.http]] | [[ui:sig.method]], [[ui:sig.url]], [[ui:sig.timeout]], [[ui:sig.headers]], [[ui:field.auth]], [[ui:sig.body]] | Una solicitud HTTP. Consulta [HTTP](../protocols/http.md). |
| [[ui:sig.tr.mqtt]] | [[ui:mq.broker]], [[ui:mq.topic]], [[ui:mq.qos]], [[ui:mq.retain]], [[ui:sig.payload]] | Una publicación. Consulta [MQTT](../protocols/mqtt.md). |

Toda señal tiene además un [[ui:sig.name]], un [[ui:sig.group]] y una
[[ui:sig.note]]: qué debería provocar y qué tiene que coincidir en el otro
extremo.

Los destinos de las señales OSC y UDP son `IP:port` o `host:port` (por ejemplo
`127.0.0.1:9000`); un nombre de host se resuelve cada vez que se envía la señal.
El bróker de una señal MQTT es `host:port`; sin puerto, es `1883`.

Cuando cambias el tipo de una señal, su mensaje vuelve a empezar desde los
valores predeterminados de ese tipo. Solo se conserva el destino, y solo entre
[[ui:sig.tr.osc]] y [[ui:sig.tr.udp]], donde el destino significa lo mismo.

## Enviar una señal {#send}

Para enviar una señal desde la pantalla [[ui:nav.signals]], haz cualquiera de
estas cosas:

- Selecciónala y pulsa [[ui:sig.fire]].
- Pulsa <kbd>Ctrl</kbd>+<kbd>Enter</kbd> mientras trabajas en sus campos.
- Haz doble clic sobre ella en el árbol.

Cada envío escribe una línea en la consola: qué fue a dónde, los bytes enviados
o, para HTTP, el estado y el tiempo que tardó. Un fallo (una conexión rechazada,
un host al que no se llega) es una línea roja con el motivo. La hora del último
envío se muestra junto a los botones.

Una señal sale por los mismos comandos que las pantallas de su protocolo, así que
el [Inspector](inspector.md) la lista bajo la herramienta que la envió, y el otro
extremo no puede distinguirla de una que hayas escrito tú.

Cómo se envía cada tipo:

| Tipo | Cómo sale |
| --- | --- |
| [[ui:sig.tr.osc]] | Como la pantalla [[ui:nav.osc]] envía un mensaje. |
| [[ui:sig.tr.udp]] | Un datagrama al destino. |
| [[ui:sig.tr.http]] | Como la pantalla [[ui:nav.http]] envía una solicitud, con su tarro de cookies mientras [[ui:http.keepCookies]] esté activado allí. Una conexión rechazada o un tiempo de espera cuenta como fallo, no como estado. |
| [[ui:sig.tr.mqtt]] | Mientras la pantalla [[ui:nav.mqtt]] está conectada al bróker de la señal, por esa conexión, con su id de cliente y sus credenciales. Si no —sin conexión, o conectada a otro bróker—, Signal Lab se conecta al bróker de la señal para esta única publicación, con un id de cliente propio, sin nombre de usuario y con una sesión limpia, y luego se desconecta. |

Una conexión es al bróker de la señal cuando el host es el mismo, sin distinguir
mayúsculas, y el puerto es el mismo, entendiendo `1883` para un bróker escrito
sin puerto. Los nombres no se resuelven: `localhost` y `127.0.0.1` son dos
brókeres distintos aquí, así que una señal que nombra uno no se envía por una
conexión hecha al otro.

::: tip
Una señal MQTT no guarda ninguna contraseña. Para publicar en un bróker que pida
una, conéctate antes a ese bróker en la pantalla [[ui:nav.mqtt]]; la señal
entonces viaja por esa conexión.
:::

## Enviar desde cualquier pantalla {#palette}

Pulsa <kbd>Ctrl</kbd>+<kbd>K</kbd> en cualquier pantalla para abrir la paleta,
escribe unas letras del nombre, la carpeta, el destino o el mensaje de una señal
y pulsa <kbd>Enter</kbd>. La paleta se cierra y la señal se envía; sigues en la
pantalla que estabas viendo.

| Tecla | Qué hace |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>K</kbd> | Abre la paleta, o la cierra. |
| <kbd>↑</kbd> <kbd>↓</kbd> | Mueve la elección. |
| <kbd>Enter</kbd> | Envía la señal elegida. |
| <kbd>Esc</kbd> | Cierra la paleta sin enviar. |

La paleta lista como máximo 12 señales: las 12 primeras de la biblioteca
mientras no hayas escrito nada, y luego las 12 primeras que coincidan. Un clic en
una fila la envía; un clic fuera cierra la paleta.

## Crear señales {#create}

### En la pantalla Señales {#create-here}

1. Elige la carpeta a la que pertenece la señal (consulta [carpeta actual](#current-folder)).
2. Pulsa [[ui:sig.new]]. Aparece en esa carpeta, seleccionada, una señal OSC
   nueva a `127.0.0.1:9000` con la dirección `/hello`.
3. Cambia [[ui:sig.name]], [[ui:sig.transport]] y los campos del mensaje.

Cada cambio se guarda solo; no hay botón de guardar en esta pantalla. Mientras no
se puede leer el archivo de la biblioteca, no se guarda nada y los campos son de
solo lectura (consulta [El archivo de la biblioteca](#file)).

[[ui:sig.duplicate]] coloca una copia justo después de la señal seleccionada, con
su nombre seguido de `·`. [[ui:sig.delete]] vuelve a preguntar
([[ui:sig.confirmDelete]]): el segundo clic la quita del archivo. Su carpeta se
queda, aunque ahora esté vacía.

### Desde las pantallas HTTP, OSC y MQTT {#save-from-screens}

La parte de envío de tres pantallas —[[ui:http.request]] en [[ui:nav.http]],
[[ui:osc.sender]] en [[ui:nav.osc]], [[ui:mq.publish]] en [[ui:nav.mqtt]]— puede
guardar lo que enviaría como una señal.

1. Prepara el mensaje y envíalo hasta que haga lo que quieres.
2. Pulsa [[ui:sig.saveNew]] (o <kbd>Ctrl</kbd>+<kbd>S</kbd> en esa parte de la
   pantalla). Se abre el diálogo [[ui:sig.saveTitle]].
3. Revisa [[ui:sig.name]]: se sugiere a partir de lo que se está enviando.
4. Elige o escribe un [[ui:sig.group]]. Se rellena con la carpeta que usaste la
   última vez; una ruta como `Venue/Stage` que aún no existe se crea.
5. Pulsa [[ui:sig.saveConfirm]].

A partir de entonces la pantalla queda vinculada a esa señal. Una etiqueta junto
a los botones indica dónde vive (`❖ Folder / Name`); haz clic en ella para ver la
señal en la pantalla [[ui:nav.signals]].

| Ves | Significa | Qué puedes hacer |
| --- | --- | --- |
| ✓ [[ui:sig.savedState]] (atenuado) | La biblioteca contiene exactamente lo que enviaría la pantalla. | Nada que guardar. |
| [[ui:sig.save]], y [[ui:sig.changed]] en la etiqueta | El mensaje de la pantalla difiere de la señal. | [[ui:sig.save]] o <kbd>Ctrl</kbd>+<kbd>S</kbd> escribe el mensaje de la pantalla en esa señal; su nombre, carpeta y nota se quedan. |
| [[ui:sig.saveAs]] | — | Abre el diálogo otra vez, relleno con el nombre y la carpeta de la señal, y guarda una señal nueva. La pantalla queda entonces vinculada a la nueva. |

La comparación mira el mensaje, no cómo está escrito: el orden de las claves JSON
y los últimos dígitos de un float OSC más allá de la precisión de 32 bits no
cuentan como cambio.

Las pantallas [[ui:nav.http]] y [[ui:nav.osc]] conservan el vínculo cuando Signal
Lab se reinicia; la pantalla [[ui:nav.mqtt]] lo conserva hasta que cierras la
aplicación.

::: warning
Una señal HTTP guarda su [[ui:field.auth]] —nombre de usuario y contraseña, o
token— en el archivo de la biblioteca como texto sin cifrar. Cualquiera que pueda
leer el archivo puede leerlos.
:::

### Abrir una señal en su pantalla {#open-in-screen}

Una señal HTTP, OSC o MQTT seleccionada tiene un botón que la abre en la pantalla
de su protocolo ([[ui:nav.http]], [[ui:nav.osc]] o [[ui:nav.mqtt]]). Los campos de
la pantalla se rellenan con la señal y la pantalla queda vinculada a ella, como
arriba: edita allí, envía y luego [[ui:sig.save]]. Una señal UDP sin procesar no
tiene pantalla propia.

### Desde una trama o un tema {#capture}

- En el [Inspector](inspector.md#save-as-signal), selecciona una trama y pulsa
  [[ui:sig.fromFrame]]. Un datagrama se convierte en una señal UDP sin procesar
  con los bytes exactos de esa trama; una publicación MQTT se convierte en una
  señal MQTT con el mismo bróker, tema, QoS, indicador de retención y carga útil.
  Las demás tramas no se pueden guardar.
- En la pantalla [[ui:nav.mqtt]], selecciona un tema y pulsa [[ui:sig.fromFrame]].
  Obtienes una señal MQTT que publica el último valor del tema, con su QoS y su
  indicador de retención, en el bróker al que estás conectado.

Ambas van a la carpeta [[ui:sig.capturedFolder]] y se nombran a partir de lo
capturado.

### En un experimento {#in-experiments}

Cuando añades un nodo a un experimento, el menú [[ui:exp.addNode]] también lista
tus señales en [[ui:exp.group.signals]]. Elegir una añade un nodo OSC, HTTP, MQTT
o UDP con el mismo mensaje. Una señal UDP sin procesar con carga útil en hex no
se ofrece: el nodo UDP envía texto. Consulta [Nodos](../experiments/nodes.md).

## Carpetas {#folders}

Una carpeta es una ruta de nombres unidos por `/`: `API/Auth` es la carpeta `Auth`
dentro de `API`. El campo [[ui:sig.group]] de una señal contiene la ruta de su
carpeta; vacío significa el nivel superior ([[ui:sig.topLevel]]). Los nombres se
recortan y las partes vacías se descartan al salir del campo, así que
` API / Auth/ ` se convierte en `API/Auth`.

Las carpetas se ordenan por nombre, con los números en orden numérico (`Cue 2`
antes de `Cue 10`); las señales se quedan en el orden del archivo. Cada carpeta
muestra cuántas señales contiene, incluidas sus subcarpetas. Una carpeta vacía se
conserva hasta que la quites.

### La carpeta actual {#current-folder}

La carpeta en la que hiciste clic por última vez, o la carpeta de la señal que
seleccionaste, es la actual: [[ui:sig.new]] y [[ui:sig.newFolder]] ponen las cosas
allí. Una etiqueta sobre el árbol la nombra; haz clic en la etiqueta para volver
al nivel superior.

### Trabajar con carpetas {#folder-tasks}

| Para | Haz esto |
| --- | --- |
| Crear una carpeta | Pulsa ＋ [[ui:sig.newFolder]]. Se crea dentro de la carpeta actual, se llama [[ui:sig.newFolderName]] (con un número detrás cuando ese nombre ya existe) y puedes renombrarla enseguida. |
| Abrir o cerrar una carpeta | Haz clic en ella, o pulsa <kbd>→</kbd> / <kbd>←</kbd> mientras tiene el foco. Signal Lab recuerda qué carpetas están cerradas. |
| Abrirlas o cerrarlas todas | Los botones ⊞ y ⊟ sobre el árbol ([[ui:sig.expandAll]], [[ui:sig.collapseAll]]). |
| Renombrar una carpeta | Pulsa ✎ ([[ui:sig.renameFolder]]) o <kbd>F2</kbd> sobre ella, escribe y pulsa <kbd>Enter</kbd>; <kbd>Esc</kbd> cancela. |
| Mover una señal o una carpeta | Arrástrala a una carpeta, o al espacio vacío del árbol para el nivel superior. |
| Mover una señal escribiendo | Cambia su campo [[ui:sig.group]]. |
| Quitar una carpeta | Pulsa × ([[ui:sig.removeFolder]]) y luego [[ui:sig.confirmRemoveFolder]], o pulsa <kbd>Delete</kbd> dos veces sobre ella. |

Un cambio de nombre nunca fusiona dos carpetas: un nombre con `/`, o uno que ya
tiene una carpeta hermana, se rechaza, y la consola lo indica. Una carpeta no se
puede arrastrar dentro de sí misma ni dentro de una carpeta que está dentro de
ella. Arrastrar una carpeta a una carpeta que ya tiene una del mismo nombre
fusiona las dos.

Quitar una carpeta quita solo la carpeta: sus señales y subcarpetas suben un
nivel. No se borra nada.

### Encontrar una señal {#filter}

Escribe en [[ui:sig.search]] sobre el árbol. Coincide con el nombre, la carpeta,
la nota, el destino y el mensaje. Mientras filtras, todas las carpetas con una
coincidencia están abiertas y las demás se ocultan.

## El conjunto inicial {#starter-set}

La primera vez que Signal Lab no encuentra ningún archivo de biblioteca, escribe
nueve ejemplos, cada uno sobre algo que es fácil hacer mal. Sus nombres y notas
se escriben en el idioma que tenga la interfaz en ese momento; a partir de ahí son
tuyos para cambiarlos. Todos apuntan a este equipo.

| Carpeta | Señal | Envía |
| --- | --- | --- |
| `OSC` | [[ui:seed.osc-fader.name]] | `/fader/1` con el float `0.75` a `127.0.0.1:9000` |
| `OSC` | [[ui:seed.osc-types.name]] | `/types` con int `-7`, float `1.5`, string `hi`, bool true, int64 `4294967296`, double `0.125` y nil |
| `OSC` | [[ui:seed.osc-id-and-value.name]] | `/tag` con los strings `reader-1` y `04a1b2c3` |
| `OSC` | [[ui:seed.osc-trigger.name]] | `/cue/go` sin argumentos |
| `MQTT` | [[ui:seed.mqtt-publish.name]] | `1` a `lab/example/value` en `127.0.0.1:1883`, QoS 0 |
| `MQTT` | [[ui:seed.mqtt-retained.name]] | `night` a `lab/example/config`, QoS 1, retenido |
| `MQTT` | [[ui:seed.mqtt-clear-retained.name]] | Una carga útil retenida vacía a `lab/example/config`, QoS 1 |
| `HTTP` | [[ui:seed.http-reachable.name]] | `GET http://127.0.0.1:8080/`, tiempo de espera 4000 ms |
| [[ui:seed.folder.Raw]] | [[ui:seed.udp-raw.name]] | Los bytes `de ad be ef` a `127.0.0.1:9000` |

Para recuperar el conjunto inicial, mueve o renombra `signals.json` y pulsa
[[ui:sig.reload]]: sin archivo allí, se vuelve a escribir.

## El archivo de la biblioteca {#file}

La biblioteca es `signals.json` en la carpeta de datos: `Documents/SignalLab` en
tu carpeta personal en un escritorio, o la carpeta de datos del servidor (consulta
[Archivos](../reference/files.md)). Pasa el puntero por encima del recuento de
señales bajo el árbol para ver la ruta completa.

- **Se guarda solo.** Cada cambio se escribe 0,7 s después del último, el archivo
  entero de una vez, a través de un archivo temporal en la misma carpeta que luego
  ocupa el lugar del archivo: una escritura cortada a medias deja el archivo
  anterior. Mientras una escritura espera, el pie del árbol indica
  [[ui:sig.saving]]; luego [[ui:sig.saved]]. Una escritura que aún espera se hace
  antes de que una actualización reinicie la aplicación.
- **Editado a mano.** Signal Lab no se entera de que el archivo cambia por debajo.
  Después de editarlo, o de sustituirlo por uno de otra máquina, pulsa
  [[ui:sig.reload]]. Recargar vuelve a leer el archivo y descarta un cambio que
  aún esperaba a escribirse.
- **Nunca se sustituye mientras está roto.** Si el archivo no es JSON válido, o
  no es una biblioteca de señales, el árbol muestra el error con la ruta, la línea
  y la columna del archivo, y la consola dice lo mismo. El archivo se deja tal
  cual, y nada escribe la biblioteca hasta que vuelva a leerse: [[ui:sig.new]],
  renombrar, mover y quitar señales y carpetas, arrastrar, los campos de una
  señal, [[ui:sig.saveNew]], [[ui:sig.save]] y [[ui:sig.saveAs]] en las pantallas
  HTTP, OSC y MQTT, y [[ui:sig.fromFrame]] en el Inspector y en la pantalla MQTT
  están todos desactivados, y su consejo dice qué está mal. Corrige el archivo, o
  quítalo, y pulsa [[ui:sig.reload]]: en cuanto se lea, todo vuelve a funcionar.
- **Un archivo que se rompe mientras la aplicación funciona.** Si editas el
  archivo y lo dejas ilegible y la aplicación guarda luego un cambio, el guardado
  se rechaza con el mismo error, el archivo se deja como lo dejaste y la
  aplicación deja de escribir hasta que lo corrijas y pulses [[ui:sig.reload]]. Un
  archivo que editaste y dejaste válido se sustituye por la lista de la aplicación
  en su siguiente guardado, como arriba: recarga primero.

Un ejemplo breve del archivo:

```json
{
  "version": 2,
  "signals": [
    {
      "id": "fader-value",
      "name": "Fader value",
      "group": "Venue/Stage",
      "note": "Main fader of desk A.",
      "body": {
        "transport": "osc",
        "target": "127.0.0.1:9000",
        "address": "/fader/1",
        "args": [{ "type": "float", "value": 0.75 }]
      }
    }
  ],
  "folders": ["Venue/Stage", "Venue/Empty for now"]
}
```

| Clave | Qué |
| --- | --- |
| `version` | `2`. Un archivo de versión 1 (anterior a las carpetas) se lee igual, sin carpetas vacías. |
| `signals[].id` | Se forma a partir del nombre cuando se crea la señal (`fader-value`, `fader-value-2`, …) y nunca lo cambia un renombrado. `signallab fire` encuentra una señal por él. |
| `signals[].group` | La ruta de la carpeta; `""` es el nivel superior. |
| `signals[].body` | El mensaje. `transport` es `osc`, `udp`, `http` o `mqtt`; las demás claves son los campos de ese tipo. |
| `folders` | Todas las carpetas, para que se conserve una vacía. Se omite cuando no hay ninguna. Un `group` que ninguna entrada lista también es una carpeta. |

## Desde la línea de comandos {#cli}

`signallab fire` envía una señal de una biblioteca, por los mismos comandos que la
aplicación:

```bash
signallab fire "Fader value"
signallab fire fader-value --library ./show/signals.json
```

Encuentra la señal por su id primero, luego por su nombre, sin distinguir
mayúsculas. Cuando varias señales tienen ese nombre, indica sus ids y no envía
nada. Sin `--library` lee el propio `signals.json` de la aplicación; nunca escribe
el archivo. Consulta [La línea de comandos](../automation/cli.md#cli-fire).

## Véase también {#related}

- [Inspector](inspector.md): observa lo que envía una señal y guarda una trama
  capturada como señal.
- [Atajos de teclado](../reference/shortcuts.md)
- [Archivos](../reference/files.md): dónde está la carpeta de datos.

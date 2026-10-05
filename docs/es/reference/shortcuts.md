---
title: Atajos de teclado
description: Todos los atajos de teclado de Signal Lab — en cualquier lugar de la aplicación, en el editor de experimentos, la biblioteca de señales, las pantallas de herramientas, los paneles y el menú de idiomas.
---

# Atajos de teclado

Cada tecla a la que responde Signal Lab, agrupada por dónde funciona. Los atajos
con una sola letra o Suprimir nunca actúan mientras escribes en un campo.

::: tip
En un navegador en un Mac, <kbd>⌘</kbd> funciona dondequiera que abajo se escriba
<kbd>Ctrl</kbd>, excepto <kbd>Ctrl</kbd>+<kbd>Space</kbd> en un campo de
plantilla.
:::

## En cualquier lugar {#anywhere}

| Teclas | Qué hacen |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>K</kbd> | Abre [[ui:sig.paletteTitle]], la paleta de señales, en cualquier pantalla — incluso mientras escribes en un campo. Otra vez la cierra |
| <kbd>F1</kbd> | Abre esta documentación en la página de la pantalla en la que estás, como [[ui:app.docs]] |
| <kbd>Tab</kbd>, <kbd>Shift</kbd>+<kbd>Tab</kbd> | Se mueve entre los controles. El consejo de un control aparece cuando toma el foco; cualquier otra tecla lo oculta |
| <kbd>Space</kbd>, <kbd>Enter</kbd> | Pulsa el botón que tiene el foco — todo lo que se puede pulsar es un botón |
| <kbd>Esc</kbd> | Cierra el diálogo que está abierto |

En la paleta de señales:

| Teclas | Qué hacen |
| --- | --- |
| Escribir | Filtra las señales |
| <kbd>↑</kbd> <kbd>↓</kbd> | Elige una señal |
| <kbd>Enter</kbd> | La envía y cierra la paleta |
| <kbd>Esc</kbd> | Cierra la paleta; también lo hace un clic al lado |

## Editor de experimentos {#editor}

En el lienzo — cuando ningún campo de texto tiene el foco:

| Teclas | Qué hacen |
| --- | --- |
| <kbd>A</kbd> | Abre el menú para añadir un nodo ([[ui:exp.addNode]]). Con un nodo seleccionado, el nuevo va después de él |
| <kbd>Delete</kbd> o <kbd>Backspace</kbd> | Quita la conexión seleccionada, o los nodos seleccionados — nunca [[ui:exp.node.start]] ni [[ui:exp.node.end]] |
| <kbd>Tab</kbd> | Se mueve por los nodos, sus puertos y las conexiones; un nodo o una conexión que toma el foco queda seleccionado |
| <kbd>Enter</kbd> o <kbd>Space</kbd> en una salida, y luego en un nodo o su entrada | Los conecta |
| <kbd>←</kbd> <kbd>→</kbd> <kbd>↑</kbd> <kbd>↓</kbd> | Mueve los nodos seleccionados 5 puntos (20 con <kbd>Shift</kbd>), mientras un nodo tiene el foco. Una pulsación mantenida es un paso del historial |
| <kbd>Esc</kbd> | Cierra el menú de añadir; si no, cancela una conexión que se está dibujando; si no, suelta la conexión seleccionada; si no, borra una selección de varios nodos; si no, sale de [[ui:exp.fullscreen]]; si no, sale de [[ui:exp.focus]] |
| <kbd>Ctrl</kbd>+<kbd>Z</kbd> | [[ui:exp.undo]] |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd>, <kbd>Ctrl</kbd>+<kbd>Y</kbd> | [[ui:exp.redo]] |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | [[ui:exp.duplicate]]: una copia de los nodos seleccionados, con las conexiones entre ellos |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | Busca un nodo por su tipo, lo que hace o su id |
| <kbd>Ctrl</kbd>+<kbd>0</kbd> | [[ui:exp.fit]]: todo el grafo a la vista |
| <kbd>Ctrl</kbd>+<kbd>1</kbd> | [[ui:exp.resetZoom]]: 100 % |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:exp.sendNow]] — para una espera, [[ui:exp.listenNow]]: solo el nodo seleccionado, sin ejecutar el experimento |
| <kbd>Ctrl</kbd>+<kbd>A</kbd> | Selecciona todos los nodos |
| <kbd>Ctrl</kbd>+<kbd>C</kbd> | [[ui:exp.copy]]: los nodos seleccionados y las conexiones entre ellos; [[ui:exp.node.start]] y [[ui:exp.node.end]] quedan fuera, igual que al duplicar |
| <kbd>Ctrl</kbd>+<kbd>X</kbd> | Los corta |
| <kbd>Ctrl</kbd>+<kbd>V</kbd> | Pega nodos copiados aquí o en otro experimento, con ids nuevos |

<kbd>Ctrl</kbd>+<kbd>A</kbd>, <kbd>C</kbd>, <kbd>X</kbd> y <kbd>V</kbd>
funcionan sobre los nodos mientras el editor tiene el foco, o mientras nada lo
tiene y no hay texto seleccionado en la página. Con texto seleccionado en otro
sitio — en la [[ui:console.title]], en un informe — son los propios de la página:
<kbd>Ctrl</kbd>+<kbd>C</kbd> copia ese texto.

Con el ratón:

| Gesto | Qué hace |
| --- | --- |
| Arrastrar en el lienzo vacío | Mueve la vista |
| <kbd>Shift</kbd> + arrastrar en el lienzo vacío | Añade a la selección los nodos que toca un marco |
| <kbd>Shift</kbd> o <kbd>Ctrl</kbd> + clic en un nodo | Lo añade a la selección, o lo quita |
| Arrastrar uno de varios nodos seleccionados | Los mueve todos |
| Doble clic en el lienzo vacío | Abre allí el menú de añadir |
| <kbd>Ctrl</kbd> + rueda | Aumenta o reduce en el puntero |

En el menú de añadir, el buscador de nodos y las propiedades:

| Dónde | Teclas | Qué hacen |
| --- | --- | --- |
| Menú de añadir | Escribir | Filtra los nodos y las señales guardadas |
| Menú de añadir | <kbd>↑</kbd> <kbd>↓</kbd>, <kbd>Enter</kbd>, <kbd>Esc</kbd> | Elegir, añadir, cerrar |
| Buscador de nodos | <kbd>↑</kbd> <kbd>↓</kbd> | Elige un nodo |
| Buscador de nodos | <kbd>Enter</kbd> | Lo muestra en el lienzo y lo selecciona |
| Buscador de nodos | <kbd>Tab</kbd> | Vuelve al campo de búsqueda |
| Buscador de nodos | <kbd>Esc</kbd> | Lo cierra |
| [[ui:exp.properties]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:exp.sendNow]] para el nodo mostrado |
| [[ui:exp.properties]] | <kbd>Esc</kbd> en un campo | Vuelve al lienzo con el nodo seleccionado, donde <kbd>A</kbd> añade el siguiente |

En un campo que acepta plantillas (`{{name}}`):

| Teclas | Qué hacen |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>Space</kbd> | Escribe `{{` en el cursor y lista lo que puede ir ahí: parámetros, variables, secretos, generadores |
| Escribir `{{` | Lista lo mismo |
| <kbd>↑</kbd> <kbd>↓</kbd> | Elige en la lista |
| <kbd>Enter</kbd> o <kbd>Tab</kbd> | Inserta la elección y cierra el `}}` |
| <kbd>Esc</kbd> | Cierra la lista; el campo conserva el foco |

En los diálogos del experimento:

| Dónde | Teclas | Qué hacen |
| --- | --- | --- |
| [[ui:exp.params]] | <kbd>Enter</kbd> en el valor del último parámetro | Añade otro parámetro |
| [[ui:exp.params]], [[ui:exp.runWith]] | <kbd>Esc</kbd> | Cierra el diálogo |
| [[ui:exp.runWith]] | <kbd>Enter</kbd> en un campo | [[ui:exp.run]] con estos valores |
| [[ui:exp.secrets]] | <kbd>Enter</kbd> en un valor | Lo guarda |
| [[ui:exp.secrets]] | <kbd>Esc</kbd> en un valor o un nombre nuevo | Cancela; [[ui:exp.params]] sigue abierto |

## Biblioteca de señales {#signals}

| Dónde | Teclas | Qué hacen |
| --- | --- | --- |
| [[ui:nav.signals]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:sig.fire]]: envía la señal seleccionada |
| [[ui:nav.signals]] | Doble clic en una señal | La envía |
| Una carpeta | <kbd>F2</kbd> | [[ui:sig.renameFolder]] |
| Renombrar una carpeta | <kbd>Enter</kbd>, <kbd>Esc</kbd> | Conserva el nombre nuevo, o cancela |
| Una carpeta | <kbd>Delete</kbd>, dos veces | [[ui:sig.removeFolder]]; lo que hay en ella sube un nivel |
| Una carpeta | <kbd>→</kbd> <kbd>←</kbd> | La abre, la cierra |
| Emisores [[ui:nav.http]], [[ui:nav.osc]], [[ui:nav.mqtt]] | <kbd>Ctrl</kbd>+<kbd>S</kbd> | [[ui:sig.save]] cuando el mensaje está vinculado a una señal de la biblioteca (abierto desde ella o guardado allí) y ha cambiado; [[ui:sig.saveNew]] cuando todavía no está en la biblioteca |
| [[ui:sig.saveTitle]] | <kbd>Enter</kbd>, <kbd>Esc</kbd> | Guarda, o cancela |

## Pantallas de herramientas {#tools}

| Pantalla | Teclas | Qué hacen |
| --- | --- | --- |
| [[ui:nav.http]] | <kbd>Enter</kbd> en la URL, un encabezado, las credenciales o el tiempo de espera | Envía la solicitud |
| [[ui:nav.http]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> en cualquier parte de la solicitud, el cuerpo incluido | Envía la solicitud |
| [[ui:nav.osc]] | <kbd>Enter</kbd> en el destino, la dirección o un argumento | Envía el mensaje |
| [[ui:nav.ws]] | <kbd>Enter</kbd> en la URL | Conecta |
| [[ui:nav.ws]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> en [[ui:ws.message]] | Lo envía |
| [[ui:nav.mqtt]] | <kbd>Enter</kbd> en [[ui:mq.addSubscription]] | Se suscribe |
| [[ui:nav.broadcast]] | <kbd>Enter</kbd> en [[ui:bc.targetAddress]] o [[ui:bc.targetCidr]] — todos los modos salvo [[ui:bc.mode.list]] | [[ui:bc.sendOnce]] |

En [[ui:feedback.title]]: <kbd>Ctrl</kbd>+<kbd>Enter</kbd> en
[[ui:feedback.message]] lo envía (también lo hace <kbd>Enter</kbd> en
[[ui:feedback.email]]), <kbd>Ctrl</kbd>+<kbd>V</kbd> en cualquier parte del
diálogo pega una captura de pantalla del portapapeles, y <kbd>Esc</kbd> lo
cierra.

## Paneles {#panes}

Los tiradores entre paneles — [[ui:layout.console]], [[ui:layout.properties]],
[[ui:layout.timeline]] — toman el foco con <kbd>Tab</kbd>:

| Teclas | Qué hacen |
| --- | --- |
| <kbd>↑</kbd> <kbd>↓</kbd> | La consola y la línea de tiempo: más alto, más bajo, 16 píxeles |
| <kbd>←</kbd> <kbd>→</kbd> | Las propiedades: más ancho, más estrecho, 16 píxeles (al revés cuando la interfaz se lee de derecha a izquierda) |
| <kbd>Shift</kbd> + una flecha | Cuatro veces más lejos |
| <kbd>Home</kbd>, <kbd>End</kbd> | El tamaño más pequeño, el más grande |
| <kbd>Enter</kbd> | El tamaño predeterminado; también lo hace un doble clic en el tirador |

## Menú de idiomas {#language}

La bandera y las letras de la cabecera.

| Dónde | Teclas | Qué hacen |
| --- | --- | --- |
| En el botón | <kbd>↓</kbd> o <kbd>↑</kbd> | Abre la lista |
| En la lista | <kbd>↓</kbd> <kbd>↑</kbd> | El idioma siguiente, el anterior |
| En la lista | <kbd>Home</kbd>, <kbd>End</kbd> | El primero, el último |
| En la lista | Una letra | El siguiente idioma cuyo nombre — en su propio idioma, en el tuyo o en inglés — o cuyas letras empiecen por ella |
| En la lista | <kbd>Enter</kbd> o <kbd>Space</kbd> | Cambia a él |
| En la lista | <kbd>Esc</kbd> o <kbd>Tab</kbd> | Cierra la lista |

---
title: La ventana
description: Oriéntate en la ventana de Signal Lab — las pantallas de la barra lateral, la cabecera, la consola y el Inspector, la paleta de señales, los paneles, las descripciones emergentes y los idiomas.
---

# La ventana

La ventana de Signal Lab tiene cuatro partes: la **barra lateral** de la izquierda enumera las
pantallas, la **cabecera** de la parte superior reúne lo que se aplica en todas partes, la
**pantalla** que elegiste ocupa el centro y el **panel inferior** muestra la consola, las tareas en
curso y el Inspector. La aplicación de escritorio y la página de un servidor en un navegador se ven
igual; las pocas diferencias están en [En un navegador](#browser).

## La barra lateral {#sidebar}

Cada pantalla es una herramienta en sí misma. Haz clic en una para abrirla:

| Pantalla | Para qué sirve |
| --- | --- |
| [[ui:nav.experiment]] | Construir flujos de prueba en un lienzo, ejecutarlos y leer cada ejecución paso a paso. [Experimentos](../experiments/index.md) |
| [[ui:nav.signals]] | La biblioteca de señales: mensajes con nombre en carpetas, para editarlos y volver a enviarlos. [Señales](../tools/signals.md) |
| [[ui:nav.emulators]] | API HTTP simuladas, dispositivos OSC, UDP y TCP y brókeres MQTT que responden según reglas. [Emuladores](../tools/emulators.md) |
| [[ui:nav.osc]] | Enviar mensajes OSC, monitorizar un puerto, enviar una forma de onda a un endpoint. [OSC](../protocols/osc.md) |
| [[ui:nav.mqtt]] | Conectar con un bróker, ver todos los temas que guarda, publicar y borrar valores retenidos. [MQTT](../protocols/mqtt.md) |
| [[ui:nav.broadcast]] | Enviar a muchos hosts a la vez (una lista, difusión, multidifusión, un barrido de subred) y escuchar quién responde. [Difusión y descubrimiento](../protocols/broadcast.md) |
| [[ui:nav.http]] | Una solicitud y toda su respuesta, y luego una ráfaga de carga contra el mismo endpoint. [HTTP](../protocols/http.md) |
| [[ui:nav.ws]] | Conectar con un servicio WebSocket, enviar texto o bytes, leer cada mensaje. [WebSocket](../protocols/websocket.md) |
| [[ui:nav.netsim]] | Un relé que degrada el tráfico UDP o TCP entre un cliente y su destino. [Degradación](../tools/impairment.md) |
| [[ui:nav.storm]] | Carga UDP o TCP sin formato contra tus propios servidores y enlaces. [Tormenta](../tools/storm.md) |
| [[ui:nav.scan]] | Qué puertos TCP de un host están abiertos, con lo primero que dice el servicio. [Escáner](../tools/scanner.md) |

La barra lateral empieza como un riel estrecho con el símbolo de cada pantalla y un nombre corto;
pasa el puntero por encima de uno para ver su nombre completo. El **☰** de la izquierda de la
cabecera ([[ui:app.expandNav]] / [[ui:app.collapseNav]]) alterna entre el riel y la lista completa,
y Signal Lab recuerda lo que elegiste.

Un número en la entrada de una pantalla cuenta las tareas en curso que salen de ella, como un
monitor o un emulador, para que veas qué sigue funcionando sin abrirla. En la parte inferior de la
barra lateral, la versión abre Acerca de, y la línea de debajo cuenta todas las tareas en curso.

Signal Lab se abre en la última pantalla que usaste.

## La cabecera {#header}

De izquierda a derecha:

| Elemento | Qué hace |
| --- | --- |
| **☰** | Muestra la barra lateral como riel o como lista completa. |
| El host | [[ui:app.host]] y, a continuación, el nombre y la dirección de red del equipo en el que funciona el motor: este en la aplicación de escritorio, el servidor en un navegador. En un navegador también indica [[ui:app.server]] (pasa el puntero por encima para ver dónde guarda el servidor sus archivos), y el punto de al lado cambia cuando la página pierde la conexión. |
| El botón de actualización | Aparece en la aplicación de escritorio cuando se ha encontrado una versión más reciente, y abre Acerca de para instalarla. Consulta [Actualizaciones](install.md#updates). |
| El libro | [[ui:app.docs]]: esta documentación, en la página de la pantalla en la que estás. <kbd>F1</kbd> hace lo mismo desde cualquier lugar. |
| **✉** | [[ui:feedback.open]]: un mensaje para los desarrolladores. Consulta [Escribir a los desarrolladores](#feedback). |
| **?** | [[ui:about.open]]: la versión, quién hace Signal Lab, cómo contactar con ellos y las actualizaciones. |
| La bandera | [[ui:app.language]]: el idioma de la interfaz. Consulta [Idiomas](#languages). |
| [[ui:app.signOut]] | En un navegador, cuando el servidor pide un token de acceso: termina la sesión de este navegador. |
| [[ui:app.stopAll]] | Detiene a la vez todas las tareas en curso: monitores, generadores, balizas, emuladores, relés, tormentas, escaneos, ejecuciones. Está atenuado mientras no hay nada en marcha. |

### La documentación {#docs}

La documentación está integrada en la aplicación y en el servidor, así que está disponible sin
conexión a internet, en el idioma de la interfaz. La aplicación de escritorio la muestra en una
ventana propia (si vuelves a pulsar el botón desde otra pantalla, esa ventana pasa a la página de
la otra pantalla) y envía a tu navegador los enlaces que salen de la documentación. En un navegador
se abre en una pestaña propia. Acerca de también tiene un botón [[ui:app.docs]], que abre
[Qué es Signal Lab](index.md).

### Acerca de {#about}

Acerca de muestra la versión, el desarrollador, la dirección de contacto (con un botón que la
copia), el código fuente y la licencia. En la aplicación de escritorio, su sección
[[ui:update.title]] busca, muestra e instala actualizaciones; consulta
[Actualizaciones](install.md#updates). En un navegador indica [[ui:update.server]].
[[ui:about.writeUs]] abre el formulario de comentarios.

### Escribir a los desarrolladores {#feedback}

El **✉** de la cabecera abre un formulario que llega directamente a los desarrolladores:

| Campo | Qué poner |
| --- | --- |
| [[ui:feedback.message]] | Qué pasó y qué esperabas en su lugar. Obligatorio; como máximo 20 000 caracteres. |
| [[ui:feedback.email]] | Opcional: dónde pueden responderte los desarrolladores. Solo lo ven ellos. |
| [[ui:feedback.screenshots]] | Hasta 6 imágenes (PNG, JPEG, WebP o GIF), de 8 MB cada una y 15 MB en total. Pega una con <kbd>Ctrl</kbd>+<kbd>V</kbd>, suelta archivos sobre la ventana o pulsa [[ui:feedback.addScreenshot]]. |
| [[ui:feedback.logs]] | Las líneas de la consola y [[ui:feedback.systemInfo]], cada uno adjunto como un archivo propio. [[ui:feedback.show]] muestra exactamente lo que se envía; desmarca cualquiera de los dos para no incluirlo. |

Los registros adjuntos omiten el nombre de este equipo, su dirección de red y los nombres de las
rutas de tus carpetas. <kbd>Ctrl</kbd>+<kbd>Enter</kbd> envía el formulario; cuando se ha enviado,
recibes un número de referencia, que la consola también guarda. El mensaje viaja a través del
servicio propio del estudio, que lo reenvía por correo; la aplicación no guarda ninguna contraseña
para ello.

## El panel inferior {#bottom-panel}

El panel que hay bajo cada pantalla tiene dos pestañas, [[ui:console.title]] e
[[ui:dock.inspector]], y entre ellas y los botones del panel, la franja de tareas en curso.

- El **chevrón** de su izquierda ([[ui:console.collapse]] / [[ui:console.expand]]) pliega el panel
  hasta su barra o lo vuelve a abrir. Con el panel plegado, la barra sigue mostrando la última
  línea de la consola; haz clic en esa línea para abrir el panel.
- Arrastra el borde superior del panel para hacerlo más alto o más bajo (consulta
  [Cambiar el tamaño de los paneles](#panes)).
- El botón de su derecha lo extiende [[ui:dock.maximise]] y lo devuelve
  ([[ui:dock.restore]]).

Si el panel está abierto, qué pestaña muestra y qué altura tiene se conservan para la próxima vez.

### La consola {#console}

La consola dice qué hizo cada herramienta y qué salió mal, con lo más reciente al final: un mensaje
enviado y su tamaño, un monitor iniciado, el estado y el tiempo de una respuesta, una tarea que
terminó y por qué. Cada línea tiene la hora (al milisegundo), una etiqueta con el nombre de la
herramienta y el mensaje, coloreado según lo que sea: hecho, información, una advertencia o un
error.

- La casilla [[ui:console.autoscroll]] mantiene a la vista la línea más reciente a medida que
  llegan líneas; desmárcala para leer hacia atrás mientras siguen llegando.
- [[ui:common.clear]] la vacía.
- Conserva las últimas 500 líneas.
- Al cambiar el idioma, toda la consola se reescribe en el nuevo.

### Tareas {#jobs-strip}

Cada tarea en curso (un monitor, un generador, una baliza, una conexión con un bróker, un emulador,
un relé, una tormenta, un escaneo, la ejecución de un experimento) tiene una píldora en la franja
con su número y lo que es, y su propio botón para detenerla. Sin nada en marcha, la franja indica
[[ui:console.empty]]. Consulta [Tareas](concepts.md#jobs).

### La pestaña Inspector {#inspector-tab}

La pestaña [[ui:dock.inspector]] muestra cada trama que las herramientas envían y reciben mientras
la captura está activada, junto a la pantalla en la que estés trabajando. Su punto se ilumina
mientras la captura está activada, y un número cuenta las tramas capturadas. El panel se abre con
la altura suficiente para la lista del Inspector y el detalle de una trama; un enlace a una trama
desde otro lugar (en la línea de tiempo de una ejecución o en la lista de un emulador) abre esta
pestaña en esa trama. Una vez abierto, el Inspector conserva su lista y su selección mientras el
panel está cerrado. Cómo usarlo: [Inspector](../tools/inspector.md).

## Enviar una señal desde cualquier lugar {#palette}

Pulsa <kbd>Ctrl</kbd>+<kbd>K</kbd> en cualquier pantalla para abrir la paleta
[[ui:sig.paletteTitle]]: escribe unas letras del nombre, la carpeta o el destino de una señal,
elige con <kbd>↑</kbd> y <kbd>↓</kbd> y pulsa <kbd>Enter</kbd> para enviarla. Muestra hasta 12
señales a la vez. <kbd>Esc</kbd>, un clic fuera de ella o <kbd>Ctrl</kbd>+<kbd>K</kbd> otra vez la
cierran. La consola indica qué se envió y adónde. Consulta [Señales](../tools/signals.md).

## Cambiar el tamaño de los paneles {#panes}

Entre los paneles que puedes redimensionar hay un tirador fino: el borde superior del panel
inferior ([[ui:layout.console]]) y, en la pantalla de experimentos, el borde de las propiedades
([[ui:layout.properties]]) y la parte superior de la línea de tiempo de la ejecución
([[ui:layout.timeline]]).

- Arrastra el tirador.
- O dale el foco con <kbd>Tab</kbd> y usa las flechas: cada pulsación lo mueve 16 píxeles, cuatro
  veces más con <kbd>Shift</kbd>; <kbd>Home</kbd> y <kbd>End</kbd> llevan al tamaño mínimo y al
  máximo.
- Haz doble clic en él, o pulsa <kbd>Enter</kbd> sobre él, para devolver al panel su tamaño
  predeterminado.

Los tamaños se conservan para la próxima vez.

## Descripciones emergentes {#tooltips}

Las pantallas muestran etiquetas, valores y estados, y guardan sus explicaciones en descripciones
emergentes: qué espera un campo, qué significa `0` en él, qué tecla hace lo mismo. Una descripción
emergente aparece cuando dejas el puntero sobre algo durante medio segundo aproximadamente, y al
instante cuando llegas a ello con el teclado; un campo muestra la descripción de su etiqueta.
<kbd>Esc</kbd>, escribir, un clic o desplazarse la ocultan. Los lectores de pantalla leen el mismo
texto.

Los mensajes de error dicen dónde y qué salió mal, y por qué; el texto propio del sistema queda
plegado bajo [[ui:err.details]].

## Las pantallas conservan su estado {#state}

Una pantalla se abre la primera vez que la visitas y luego se queda como está mientras trabajas en
otra: lo que escribiste, la última respuesta, la lista de mensajes de un monitor y la tarea que hay
detrás, incluso la posición de desplazamiento, siguen ahí cuando vuelves. Una tarea en curso
continúa sea cual sea la pantalla que mires.

Algunos valores se conservan también entre reinicios: el mensaje OSC que estabas enviando, la
solicitud HTTP, la dirección WebSocket, el tamaño de los paneles, la pantalla en la que estabas.

## El aviso del firewall {#firewall-notice}

El Firewall de Windows Defender decide, programa por programa, si otros equipos pueden llegar a él.
La primera vez que un programa escucha, Windows pregunta a la persona que está frente a la
pantalla, y un *Cancelar* ahí, o una red que Windows considera pública, descarta en silencio todo
lo que envíen otros equipos. Un monitor que no muestra nada es la señal habitual.

Por eso, en la aplicación de escritorio en Windows, la primera vez que algo empieza a escuchar (un
monitor OSC, la escucha de descubrimiento, un relé de degradación, un emulador o la ejecución de un
experimento), Signal Lab consulta el firewall una vez. Cuando el firewall estorba, un aviso bajo la
cabecera lo indica:

- [[ui:fw.allow]] pide derechos de administrador con el propio aviso de Windows y luego deja que
  otros equipos lleguen a Signal Lab en redes privadas y de dominio.
- En una red que Windows llama pública (a menudo, la Wi-Fi de un recinto), el botón es
  [[ui:fw.allowPublic]].
- [[ui:fw.dismiss]] oculta el aviso durante esta sesión.

Permitirlo sustituye las reglas de entrada del firewall de Signal Lab por una sola regla que lo
permite. El tráfico dentro de este mismo equipo (`127.0.0.1`) nunca se ve afectado, así que puedes
ignorar el aviso mientras trabajas en loopback. Una instalación para todos ya tiene la regla;
consulta [Qué añade el instalador](install.md#windows-setup-adds). `signallab doctor` informa de lo
mismo desde un terminal y `signallab firewall allow` lo arregla allí; consulta
[La línea de comandos](../automation/cli.md). En Linux y en un navegador no hay aviso: el firewall
de un servidor es cosa de su administrador.

## En un navegador {#browser}

La página de un [servidor](../server/index.md) es la misma interfaz, con algunas diferencias:

- El host de la cabecera nombra el servidor, con [[ui:app.server]] al lado.
- Si el servidor pide un token de acceso, inicias sesión una vez, y [[ui:app.signOut]] en la
  cabecera termina la sesión.
- Si la página pierde la conexión con el servidor, una barra indica [[ui:app.connectionLost]]
  hasta que vuelve; la consola anota ambas cosas.
- Los informes de ejecución, las exportaciones y las capturas del Inspector los descarga el
  navegador en lugar de mostrarse como una ruta.
- Acerca de no tiene actualizaciones: el servidor se actualiza con su imagen.

Todo lo que hace la página de un servidor ocurre en el servidor: el tráfico sale de él, los
monitores escuchan en sus puertos, los archivos van a su carpeta de datos. Consulta
[Conceptos](concepts.md#desktop-and-server).

## Idiomas {#languages}

La bandera de la cabecera muestra el idioma actual y sus dos letras. Haz clic en ella para abrir la
lista de todos los idiomas, cada uno con su bandera y su propio nombre, y elige uno. En la lista,
<kbd>↑</kbd>, <kbd>↓</kbd>, <kbd>Home</kbd> y <kbd>End</kbd> mueven la selección, una letra salta
al siguiente idioma que empieza por ella (en su propio nombre o en inglés, así que <kbd>g</kbd>
encuentra Deutsch), <kbd>Enter</kbd> elige y <kbd>Esc</kbd> cierra la lista.

La interfaz cambia al instante, sin reiniciar: todas las pantallas, descripciones emergentes y
errores, y también las líneas anteriores de la consola. La primera vez que Signal Lab se inicia,
elige el primero de los idiomas de tu sistema que tenga, o el inglés, y a partir de entonces
conserva tu elección; en un navegador, para ese navegador.

El árabe pone toda la ventana de derecha a izquierda. Lo que son datos se queda de izquierda a
derecha, tal como se escribe: direcciones, volcados hexadecimales, código y el lienzo del
experimento.

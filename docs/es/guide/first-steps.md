---
title: Primeros pasos
description: Una primera sesión en un solo equipo — envía un mensaje OSC y míralo llegar, guárdalo como señal, pregunta a una API emulada y construye y ejecuta un pequeño experimento.
---

# Primeros pasos

Esta sesión no necesita nada más que Signal Lab: todo va a `127.0.0.1`, este equipo, así que no
intervienen dispositivos, redes ni reglas de firewall. Vas a:

1. enviar un mensaje OSC y verlo llegar;
2. ver el mismo mensaje en el Inspector;
3. guardarlo en la biblioteca y volver a enviarlo desde cualquier lugar;
4. iniciar una API HTTP emulada y preguntarle algo;
5. ejecutar un experimento contra esa API, leer por qué falla, corregirlo y añadir una comprobación.

Si aún no has instalado Signal Lab, consulta [Instalación y actualizaciones](install.md).
¿No sabes dónde está algo en la ventana? Consulta [La ventana](interface.md).

## Enviar un mensaje OSC y verlo llegar {#osc}

Primero, algo que reciba el mensaje: el monitor de la pantalla OSC.

1. Abre [[ui:nav.osc]] en la barra lateral.
2. En [[ui:osc.monitor]], pon [[ui:common.bind]] en `127.0.0.1:9000`, para que el monitor escuche
   solo en este equipo.
3. Pulsa [[ui:osc.listen]]. El botón pasa a ser [[ui:common.stop]], la consola indica que el
   monitor está escuchando y el monitor aparece como tarea en la franja del panel inferior.

Ahora el mensaje, desde el emisor que está al lado:

4. En [[ui:osc.sender]], deja [[ui:common.target]] en `127.0.0.1:9000`, el puerto en el que
   escucha el monitor.
5. Deja [[ui:common.address]] en `/hello/avatar/1` y el único argumento float de
   [[ui:common.arguments]] en `1.0`, o escribe una dirección y unos valores propios.
6. Pulsa [[ui:common.send]], o <kbd>Enter</kbd> en el campo del destino o de la dirección.

Aparece una línea en la tabla del monitor: la [[ui:common.time]] a la que llegó,
[[ui:osc.from]] (`127.0.0.1` y el puerto desde el que se envió), la [[ui:osc.address]] y los
[[ui:osc.args]]. Bajo el emisor, una línea confirma lo que se envió y su tamaño en bytes; vuelve a
enviarlo y cuenta las repeticiones.

::: tip ¿Un aviso sobre el firewall?
En Windows, al iniciar el monitor puede aparecer bajo la cabecera un aviso sobre el Firewall de
Windows. Se refiere a los mensajes de *otros* equipos; el tráfico en `127.0.0.1` nunca se filtra.
Pulsa [[ui:fw.dismiss]] por ahora; [El aviso del firewall](interface.md#firewall-notice) explica
cuándo conviene permitirlo.
:::

## Verlo en el Inspector {#inspector}

El Inspector registra cada trama que envía y recibe cada herramienta, pero solo mientras la captura
está activada.

1. En el panel inferior, abre la pestaña [[ui:dock.inspector]].
2. Pulsa [[ui:ins.arm]]. El punto de la pestaña se ilumina.
3. De vuelta en el emisor, pulsa [[ui:common.send]] una vez más.

Aparecen dos filas, la más reciente primero: el mensaje tal como se envió (→) y tal como lo recibió
el monitor (←), cada una con su protocolo, la dirección del otro extremo, su tamaño y un resumen.
Haz clic en una: [[ui:ins.detail]] muestra qué herramienta la envió o la recibió y en qué
direcciones, el mensaje en la sección [[ui:ins.decoded]] y, en [[ui:ins.rawBytes]], los bytes que lo formaban.

Pulsa [[ui:ins.disarm]] cuando termines; mientras la captura está desactivada no cuesta nada. Más
en [Inspector](../tools/inspector.md).

## Guardarlo como señal y volver a enviarlo {#signal}

Un mensaje que vas a querer otra vez tiene su sitio en la biblioteca de señales.

1. En la pantalla OSC, pulsa [[ui:sig.saveNew]] bajo el emisor.
2. En [[ui:sig.saveTitle]], pon [[ui:sig.name]] en `First message` y [[ui:sig.group]] en
   `Tutorial`; la carpeta nueva se crea al guardar en ella.
3. Pulsa [[ui:sig.saveConfirm]].

El emisor queda ahora vinculado a esa señal: el botón indica [[ui:sig.savedState]] y una etiqueta a
su lado muestra dónde está la señal. Cambia el argumento y la etiqueta anota el cambio;
[[ui:sig.save]] (<kbd>Ctrl</kbd>+<kbd>S</kbd>) actualizaría la señal.

Ahora vuelve a enviarla, de tres formas:

- **Desde la biblioteca.** Haz clic en la etiqueta: se abre [[ui:nav.signals]] con la señal
  seleccionada en la carpeta `Tutorial` (o abre [[ui:nav.signals]] y haz clic en ella allí). Pulsa
  [[ui:sig.fire]], o <kbd>Ctrl</kbd>+<kbd>Enter</kbd>; un doble clic sobre ella en la lista también
  la envía.
- **Desde cualquier lugar.** En cualquier pantalla, pulsa <kbd>Ctrl</kbd>+<kbd>K</kbd>, escribe
  `first` y pulsa <kbd>Enter</kbd>.
- **Desde un experimento.** Cuando añades un nodo, el menú muestra tus señales en
  [[ui:exp.group.signals]], listas para convertirse en un paso que envía una de ellas.

Cada vez, el monitor muestra la llegada del mensaje y la consola nombra la señal. Una señal envía
exactamente lo que habría enviado su pantalla. Más en [Señales](../tools/signals.md).

Cuando termines con OSC, pulsa [[ui:common.stop]] en el monitor.

## Preguntar a una API emulada {#emulator}

Signal Lab trae cinco emuladores, todos en `127.0.0.1`. Uno de ellos, la
[[ui:seed.emu.demo-api.name]], es una API HTTP en `127.0.0.1:8080` con estas rutas:

| Solicitud | Respuesta |
| --- | --- |
| `GET /health` | `200` con `{"status":"ok","time":"…"}`: la hora actual |
| `GET /users/:id` | `200` con el usuario de ese id, por ejemplo `{"id":"42","name":"User 42"}` |
| `POST /users` | `201` con un encabezado `Location` y el nuevo id |
| `GET /slow` | `200` al cabo de 1,5 segundos |
| cualquier método, `/flaky` | `503`, `503` y luego `200` a partir de la tercera solicitud |
| cualquier otra cosa | `404` |

1. Abre [[ui:nav.emulators]]. La [[ui:emu.library]] muestra los cinco; selecciona
   [[ui:seed.emu.demo-api.name]].
2. Pulsa [[ui:emu.start]]. Ahora responde en `127.0.0.1:8080` y funciona como una tarea.
3. Abre [[ui:nav.http]]. El método es `GET`; pon la URL
   `http://127.0.0.1:8080/health`.
4. Pulsa [[ui:common.send]], o <kbd>Enter</kbd> en la URL.

En [[ui:http.response]] ves el [[ui:http.status]] `200`, la [[ui:http.latency]], el
[[ui:http.size]], los encabezados de la respuesta y el cuerpo JSON. Envía
`http://127.0.0.1:8080/flaky` tres veces: dos respuestas `503` y luego `200`, que es como ve un
cliente que reintenta a un servicio que se recupera.

De vuelta en [[ui:nav.emulators]], el panel [[ui:emu.live]] cuenta cada solicitud, y
[[ui:emu.received]] muestra cada una con la [[ui:emu.col.rule]] que la respondió y la
[[ui:emu.col.reply]]. Deja la [[ui:seed.emu.demo-api.name]] en marcha para la parte siguiente. Más
en [Emuladores](../tools/emulators.md).

## Ejecutar un experimento {#experiment}

Un experimento es un flujo de pasos que puedes ejecutar una y otra vez. El que Signal Lab abre la
primera vez, la plantilla [[ui:exp.templateHttp]], envía una solicitud a
`http://127.0.0.1:8080/` y comprueba que la respuesta es `200`.

### Abrir la plantilla {#open-template}

1. Abre [[ui:nav.experiment]].
2. Si el lienzo no muestra cuatro nodos ([[ui:exp.node.start]], [[ui:exp.node.http]],
   [[ui:exp.node.assert_status]], [[ui:exp.node.end]]), pulsa **☰** a la izquierda de la barra de
   herramientas ([[ui:exp.documents]]), elige [[ui:exp.templateHttp]] en la lista de plantillas y
   pulsa [[ui:exp.openDocument]]. Abrirla sustituye el experimento del lienzo;
   <kbd>Ctrl</kbd>+<kbd>Z</kbd> recupera el anterior.

Haz clic en un nodo para ver sus ajustes en [[ui:exp.properties]], a la derecha. Los experimentos
se guardan solos mientras los editas.

### Ejecutarlo y leer por qué falla {#first-run}

3. Pulsa [[ui:exp.run]].

La [[ui:exp.timeline]] se abre bajo el lienzo, con una fila por paso cuando empieza
([[ui:exp.running]]) y otra cuando termina: la hora, el nodo y cómo fue. Esta ejecución falla:

- [[ui:exp.node.start]] se supera e indica la semilla de la ejecución.
- [[ui:exp.node.http]] se supera: la solicitud salió y volvió una respuesta, `HTTP 404`.
- [[ui:exp.node.assert_status]] falla: esperaba `200` y recibió `404`.

La [[ui:seed.emu.demo-api.name]] no tiene ninguna ruta para `/`, así que respondió `404`, y la
comprobación lo detectó. La línea de la parte superior de la línea de tiempo indica
[[ui:exp.failed]] y por qué. Haz clic en una fila para seleccionar su nodo en el lienzo.

::: tip ¿Falló la propia solicitud?
Si el paso [[ui:exp.node.http]] falla con una conexión rechazada, no hay nada escuchando en
`127.0.0.1:8080`: inicia la [[ui:seed.emu.demo-api.name]] en [[ui:nav.emulators]] y vuelve a
ejecutar.
:::

### Corregir la solicitud {#fix}

4. Haz clic en el nodo [[ui:exp.node.http]].
5. En [[ui:exp.properties]], cambia [[ui:sig.url]] a `http://127.0.0.1:8080/health`.
6. Pulsa [[ui:exp.run]].

Esta vez se superan todos los pasos: [[ui:exp.node.assert_status]] indica
[[ui:exp.step.checked]], [[ui:exp.node.end]] indica [[ui:exp.step.complete]] y el título de la
línea de tiempo indica [[ui:exp.passed]].

### Añadir una comprobación {#add-check}

Un estado `200` dice que el servicio respondió; no dice qué respondió. Comprueba también el cuerpo:

7. Haz clic en el nodo [[ui:exp.node.assert_status]].
8. En [[ui:exp.properties]], pulsa [[ui:exp.addNext]], o pulsa <kbd>A</kbd> con el foco en el
   lienzo. Se abre un menú de nodos con un campo de búsqueda.
9. Escribe `assert_body` y pulsa
   <kbd>Enter</kbd>. Se añade un nodo [[ui:exp.node.assert_body]] entre
   [[ui:exp.node.assert_status]] y [[ui:exp.node.end]], ya conectado, con su campo
   [[ui:exp.contains]] listo para escribir.
10. Escribe `"status":"ok"`.
11. Pulsa [[ui:exp.run]].

El paso nuevo se supera. Cambia el texto por algo que el cuerpo no contenga y vuelve a ejecutar
para verlo fallar con el motivo.

### Lo que deja una ejecución {#report}

- **Un informe.** Cuando termina una ejecución, aparece [[ui:exp.reportSaved]] en el título de la
  línea de tiempo; pasa el puntero por encima para ver el archivo. Una ejecución que termina,
  superada o fallida, escribe uno en la carpeta `runs` de tu carpeta de datos, con los valores que
  usó y cada paso. En un navegador es un enlace de descarga.
- **Una semilla.** El título muestra también la semilla de la ejecución con [[ui:exp.pinSeed]]: los
  valores aleatorios de una ejecución siguen su semilla, y fijarla los repite exactamente.

Más en [Ejecuciones e informes](../experiments/runs.md).

## Limpiar {#clean-up}

Pulsa [[ui:app.stopAll]] en la cabecera: detiene la [[ui:seed.emu.demo-api.name]] y todo lo que
siga en marcha. Tu señal, el experimento y sus informes se quedan en tu carpeta de datos.

## Y ahora {#next}

- [Conceptos](concepts.md): las ideas en las que se basan las pantallas, las señales, las tareas,
  los emuladores y los experimentos.
- [Experimentos](../experiments/index.md): el editor completo, y cada tipo de nodo en
  [Nodos](../experiments/nodes.md).
- [OSC](../protocols/osc.md), [HTTP](../protocols/http.md) y las páginas de los demás protocolos,
  cuando apuntes Signal Lab a equipos reales.
- [La línea de comandos](../automation/cli.md): ejecuta el mismo experimento desde un terminal o
  un pipeline.

---
title: Qué es Signal Lab
description: Para qué sirve Signal Lab, qué puedes hacer con él, las dos formas de ejecutarlo y cómo está organizado.
---

# Qué es Signal Lab

Signal Lab es un laboratorio de pruebas para OSC y protocolos de red. Envía los mensajes que hablan
tus equipos y servicios, te muestra lo que vuelve, ocupa el lugar del dispositivo o la API que
aún no existe, degrada a propósito la red entre ellos y convierte todo eso en experimentos
que puedes volver a ejecutar desde la aplicación, un script o un pipeline de CI.

Se creó para poner en marcha una instalación antes de que la instalación exista: el
controlador del espectáculo, el servidor de medios, los sensores y la API en la nube pueden
probarse cada uno contra un sustituto, en un solo portátil, mucho antes de encontrarse in situ.

## Para quién es {#audience}

- **Quienes ponen en marcha control de espectáculos, instalaciones y dispositivos en red**:
  servidores de iluminación y de medios, controladores, sensores, proyectores, todo lo que hable
  OSC, UDP, TCP o MQTT.
- **Quienes prueban API y servicios**: endpoints HTTP y WebSocket, su autenticación, su
  comportamiento bajo carga y cuando falla una dependencia.
- **Quienes automatizan cualquiera de las dos cosas**: los mismos experimentos se ejecutan sin
  ventana en un pipeline, en un servidor de laboratorio junto a los equipos o dirigidos por un
  asistente de IA.

## Qué puedes hacer {#what-you-can-do}

### Enviar y observar tráfico {#send-and-watch}

Cada protocolo tiene su propia pantalla:

| Protocolo | Qué puedes hacer | Página |
| --- | --- | --- |
| OSC | Enviar mensajes con argumentos con tipo, monitorizar un puerto, enviar una forma de onda a un endpoint | [OSC](../protocols/osc.md) |
| UDP y TCP sin formato | Enviar texto o bytes en experimentos y desde la biblioteca, emular dispositivos que los hablan | [UDP y TCP](../protocols/udp-tcp.md) |
| HTTP | Una solicitud con su respuesta completa, autenticación Basic, Bearer o Digest, un almacén de cookies, una ráfaga de carga | [HTTP](../protocols/http.md) |
| WebSocket | Conectar con los encabezados y subprotocolos que espera un servicio, enviar texto o bytes, leer cada mensaje | [WebSocket](../protocols/websocket.md) |
| MQTT 3.1.1 | Conectar con un bróker, ver todos los temas que guarda, publicar con QoS 0, 1 o 2, borrar un valor retenido | [MQTT](../protocols/mqtt.md) |
| Difusión y multidifusión | Enviar a una lista, a una dirección de difusión, a un grupo de multidifusión o a todos los hosts de una subred; escuchar quién responde | [Difusión y descubrimiento](../protocols/broadcast.md) |

### Ver cada trama {#inspect}

El [Inspector](../tools/inspector.md) registra cada trama que las herramientas envían y reciben,
en una sola línea de tiempo: decodificada, con sus bytes, filtrable y exportable. Una trama que
haya capturado puede convertirse en una señal que la reenvía byte a byte.

### Guardar lo que funciona {#library}

Un mensaje que funcionó va a la [biblioteca de señales](../tools/signals.md): con nombre,
ordenado en carpetas y con una nota sobre lo que debería provocar. Lo vuelves a enviar desde su
pantalla, desde la biblioteca o desde cualquier lugar con <kbd>Ctrl</kbd>+<kbd>K</kbd>.

### Hacer el papel de la otra parte {#emulate}

Los [emuladores](../tools/emulators.md) responden como la API, el dispositivo o el servicio con
el que habla tu sistema: una API HTTP con rutas y respuestas, un dispositivo OSC, UDP o TCP con
reglas, un bróker MQTT. Pueden responder despacio, fallar en secuencia, enviar cuerpos mal
formados o caer según un programa, para que pruebes cómo se las arregla tu sistema antes de que
exista lo real.

### Romper la red a propósito {#impair}

Un [relé de degradación](../tools/impairment.md) se sitúa entre un cliente y su destino y añade
latencia, jitter, pérdida, duplicados, reordenación o un límite de ancho de banda a UDP, o
retrasa, limita, restablece y bloquea conexiones TCP. Cada decisión sigue una semilla, así que
el mismo tráfico corre dos veces la misma suerte.

### Convertirlo en una prueba {#experiments}

Un [experimento](../experiments/index.md) es un flujo de pasos que dibujas en un lienzo: enviar
una solicitud, esperar la respuesta, comprobarla, extraer un valor, bifurcar, repetir en bucle,
ejecutar ramas en paralelo, iniciar un emulador o un relé de degradación para la ejecución. Cada
ejecución se informa paso a paso y se guarda como un informe que puedes comparar con uno anterior.

### Someter un servicio a carga {#load}

La solicitud HTTP de un experimento puede ejecutarse [bajo carga](../experiments/load.md) (una
tasa constante, una rampa, escalones, un pico o llegadas aleatorias), medida y juzgada por
umbrales. La pantalla [HTTP](../protocols/http.md) tiene una ráfaga de carga rápida, y
[Tormenta](../tools/storm.md) genera carga UDP o TCP sin formato contra tus propios servidores y
enlaces.

### Encontrar lo que hay en la red {#discover}

[Difusión y descubrimiento](../protocols/broadcast.md) encuentra los dispositivos que responden
a un sondeo, y el [Escáner](../tools/scanner.md) muestra qué puertos TCP de un host están abiertos.

### Automatizarlo {#automate}

La [línea de comandos](../automation/cli.md) `signallab` ejecuta experimentos sin ventana y
termina con un código que un pipeline entiende; escribe informes JUnit para [CI](../automation/ci.md).
`signallab mcp` permite a un [asistente de IA](../automation/mcp.md) leer, escribir y ejecutar
experimentos. Un [servidor](../server/index.md) ofrece lo mismo a través de su [API HTTP](../api/index.md).

## Dos formas de ejecutarlo {#two-ways}

| | Aplicación de escritorio | Servidor, en un navegador |
| --- | --- | --- |
| Funciona en | Windows 10 y 11 (x64), Linux (x86_64) | Linux como imagen de Docker (amd64 y arm64), o compilado desde el código fuente |
| Lo usas | en su propia ventana | en Chrome, Edge o Firefox, desde cualquier equipo que llegue al servidor |
| El tráfico sale de | este equipo | el servidor |
| Incluye | la línea de comandos `signallab` | la línea de comandos `signallab`, en la imagen |
| Se actualiza | sola, cuando tú lo indicas | con su imagen |

Ambas son la misma interfaz sobre el mismo motor: todas las pantallas, el Inspector, los
experimentos y los informes funcionan igual. Un servidor es la opción cuando los equipos están
en una red a la que no llega tu propio PC: un PC de rack o una pequeña máquina Linux junto a
ellos, usado desde un portátil o por un pipeline. Consulta [Instalación y actualizaciones](install.md)
y [El servidor](../server/index.md); lo que cambia entre las dos está en
[Conceptos](concepts.md#desktop-and-server).

## Cómo está organizado {#organisation}

- **Las pantallas** son herramientas para el trabajo que haces ahora, a mano: envía esto, escucha
  allí, inicia ese emulador. La barra lateral las enumera; cada una conserva lo que escribiste y
  lo que recibió mientras cambias a otra. Consulta [La ventana](interface.md).
- **Los experimentos** son flujos que construyes una vez y vuelves a ejecutar, siempre de la misma
  forma, con un informe de cada ejecución.
- **Las bibliotecas** guardan lo que creaste: las señales en la biblioteca de señales y los
  emuladores en la biblioteca de emuladores. Los experimentos usan ambas.
- **Las tareas** son el trabajo de larga duración (un monitor, un emulador, un relé, una
  ejecución) y aparecen en la parte inferior de la ventana, donde detienes una o todas.

[Conceptos](concepts.md) explica cada uno de ellos con más detalle.

## Seguro por defecto {#defaults}

- Signal Lab no envía nada hasta que pulsas un botón, y solo adonde tú le indicas. La única
  excepción es la búsqueda de actualizaciones de la aplicación de escritorio, una vez al día, que
  puedes desactivar (consulta [Actualizaciones](install.md#updates)).
- Las señales iniciales, los emuladores iniciales y las plantillas de experimentos apuntan todos
  a `127.0.0.1`, este equipo. Llegar a la red es una decisión que tomas al escribir una dirección.
- Un servidor iniciado sin token de acceso escucha solo en `127.0.0.1` y rechaza cualquier otra
  dirección.
- El firewall solo cambia cuando haces clic para permitirlo.

## Uso responsable {#responsible-use}

::: danger Tráfico real hacia hosts reales
[Tormenta](../tools/storm.md), el [Escáner](../tools/scanner.md) y
[Difusión](../protocols/broadcast.md) envían tráfico real a hosts reales, y una difusión o un
barrido de subred llega a todos los dispositivos del segmento, no solo al que tenías en mente.
Apúntalos solo a sistemas que sean tuyos o que tengas autorización para probar, y comprueba
antes en qué red estás. Las tasas altas pueden saturar enlaces y activar la detección de intrusiones.
:::

El motor tiene límites de seguridad: un barrido llega como máximo a 1024 hosts y una baliza envía
como máximo 50 000 paquetes por segundo entre todos sus destinos. Son límites de seguridad, no
un permiso.

## Idiomas {#languages}

La interfaz, sus descripciones emergentes y sus mensajes de error, la línea de comandos
`signallab`, la página de inicio de sesión del servidor y el instalador de Windows hablan 11
idiomas: English (inglés), Русский (ruso), Español, Français (francés), Deutsch (alemán),
Português (portugués de Brasil), 中文 (chino simplificado), 日本語 (japonés), 한국어 (coreano),
हिन्दी (hindi) y العربية (árabe, de derecha a izquierda). Lo cambias en vivo desde la cabecera;
consulta [La ventana](interface.md#languages).

## Siguientes pasos {#next}

1. [Instala Signal Lab](install.md).
2. Oriéntate en [la ventana](interface.md).
3. Da los [primeros pasos](first-steps.md) en este equipo.
4. Lee los [conceptos](concepts.md) en los que se basa.

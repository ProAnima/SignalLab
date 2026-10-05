---
title: Asistentes (MCP)
description: signallab mcp permite a un asistente de IA construir, comprobar y ejecutar experimentos, enviar mensajes, escuchar y hacer de emulador, mediante el Model Context Protocol.
---

# Signal Lab para un asistente: `signallab mcp`

`signallab mcp` es un servidor del [Model Context Protocol](https://modelcontextprotocol.io). Un
asistente en Claude Code, Claude Desktop, Cursor, VS Code o cualquier otro cliente MCP lo inicia y
a partir de ahí puede:

- saber de qué se compone un experimento, escribir uno, comprobarlo, ejecutarlo y leer, paso a
  paso, por qué falló;
- enviar un mensaje OSC, un datagrama, una solicitud HTTP, un mensaje WebSocket o una publicación
  MQTT, y escuchar en un puerto lo que envía un dispositivo;
- enviar una señal de tu biblioteca;
- hacer el papel de una dependencia (una API HTTP, un dispositivo OSC, UDP o TCP, un bróker MQTT)
  y leer lo que tu sistema le envió;
- volver a leer ejecuciones anteriores y comparar dos de ellas.

Cada acción pasa por los mismos comandos del motor que usa la aplicación, así que un experimento
que ejecuta el asistente es la misma ejecución que haría la aplicación, con el mismo informe, y
cada fallo se expresa con las mismas palabras que la interfaz.

::: warning
Los envíos, las ejecuciones y los emuladores generan tráfico real en la red. Dile al asistente con
qué dispositivos puede hablar; las plantillas incluidas apuntan a loopback (`127.0.0.1`).
:::

## Configuración {#setup}

`signallab` viene con la aplicación de escritorio y está en tu `PATH` después de instalarla
(consulta [Instalación](cli.md#install)). El cliente inicia `signallab mcp` por sí mismo y habla con
él por stdin y stdout; no lo ejecutas a mano.

### Mostrar la configuración {#print-config}

`--print-config` muestra lo que necesita un cliente, con la ruta completa de este `signallab`:

| Comando | Qué muestra |
| --- | --- |
| `signallab mcp --print-config claude-code` | La línea de comandos `claude mcp add`. |
| `signallab mcp --print-config claude-desktop` | La entrada `mcpServers` para el archivo de configuración de Claude Desktop. |
| `signallab mcp --print-config cursor` | La misma entrada `mcpServers`, para el `mcp.json` de Cursor. |
| `signallab mcp --print-config vscode` | La entrada `servers` para el `.vscode/mcp.json` de VS Code. |

Para un asistente que trabaja en un [servidor de laboratorio](#on-a-server), añade
`--server URL`: la configuración mostrada lo incluye entonces, con un marcador de posición para el
token.

### Claude Code {#claude-code}

Ejecuta la línea que muestra `--print-config claude-code`, por ejemplo:

```bash
claude mcp add signallab -- "C:\Program Files\Signal Lab\signallab.exe" mcp
```

### Claude Desktop y Cursor {#claude-desktop}

Pon la entrada en la configuración del cliente (en Claude Desktop, `claude_desktop_config.json`;
en Cursor, `mcp.json`) y reinicia el cliente:

```json
{
  "mcpServers": {
    "signallab": {
      "command": "C:\\Program Files\\Signal Lab\\signallab.exe",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

### VS Code {#vscode}

```json
{
  "servers": {
    "signallab": {
      "type": "stdio",
      "command": "/usr/bin/signallab",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

### Otros clientes {#other-clients}

Cualquier cliente que inicie un servidor stdio funciona igual: el comando es `signallab` (o su
ruta completa), y los argumentos, `mcp` y cualquiera de las [opciones](#options). En Linux, la
imagen del servidor también puede servir de comando:

```bash
docker run -i --rm --network host --entrypoint signallab ghcr.io/proanima/signallab:[[version]] mcp
```

## Opciones {#options}

| Opción | Qué hace | Predeterminado |
| --- | --- | --- |
| `--server URL` | Ejecuta los experimentos, los envíos y los emuladores en este servidor de Signal Lab (consulta [En un servidor de laboratorio](#on-a-server)). | `SIGNALLAB_SERVER` |
| `--token-file PATH` | Un archivo que contiene el token del servidor. | `SIGNALLAB_TOKEN_FILE`, si no, `SIGNALLAB_TOKEN` |
| `--data-dir PATH` | Dónde se guardan las ejecuciones y sus informes. No junto con `--server`. | la carpeta de datos de la aplicación (`Documents/SignalLab`) |
| `--library PATH` | La biblioteca de señales para `list_signals` y `fire_signal`. | el `signals.json` de la aplicación |
| `--emulators PATH` | La biblioteca de emuladores para `list_emulators` y `start_emulator`. | el `emulators.json` de la aplicación |
| `--secrets files\|system` | De dónde salen los valores secretos para las ejecuciones en este equipo, como en [`run`](cli.md#secrets). No junto con `--server`. | `files` |
| `--secrets-dir PATH` | Una carpeta de archivos de secretos, uno por nombre. No junto con `--server`. | `/run/secrets/signallab`, si existe |
| `--lang <code>` | El idioma de los resultados y los fallos. | `SIGNALLAB_LANG`, si no, la configuración regional y, si no, `en` |
| `--print-config CLIENT` | Muestra la configuración de un cliente y termina: `claude-code`, `claude-desktop`, `cursor` o `vscode`. | |

Las ejecuciones guardan sus informes en la carpeta de datos de la aplicación, donde la aplicación
guarda los suyos, así que se conservan después de la sesión.

## Herramientas {#tools}

Las herramientas que solo leen están marcadas como de solo lectura, para que un cliente pueda
dejar que se ejecuten sin preguntar. Las herramientas que llegan al mundo exterior (envían,
escuchan o inician algo) están marcadas como tales, y un cliente puede preguntarte antes de cada
llamada. Ninguna está marcada como destructiva.

| Herramienta | Qué hace | Llega al mundo exterior |
| --- | --- | --- |
| `describe_nodes` | El documento del experimento, cada tipo de nodo con sus campos, salidas y un ejemplo, el lenguaje `{{template}}`, los perfiles de carga y el documento del emulador. | no |
| `list_templates` | Los experimentos incluidos, con sus parámetros. | no |
| `get_template` | Un experimento incluido como documento. | no |
| `validate_experiment` | Comprueba un experimento como hace el editor antes de una ejecución; no envía nada. | no |
| `run_experiment` | Ejecuta un experimento hasta el final e informa de cada paso. | sí |
| `send_osc` | Un mensaje OSC. | sí |
| `send_udp` | Un datagrama UDP. | sí |
| `send_http` | Una solicitud HTTP. | sí |
| `send_mqtt` | Una publicación MQTT 3.1.1. | sí |
| `send_ws` | Un intercambio WebSocket. | sí |
| `listen` | Lo que llega a un puerto UDP durante un tiempo. | sí |
| `list_signals` | Las señales de tu biblioteca. | no |
| `fire_signal` | Envía una señal de la biblioteca. | sí |
| `list_emulators` | Los emuladores de tu biblioteca. | no |
| `start_emulator` | Inicia un emulador. | sí |
| `emulator_exchanges` | Lo que recibió y respondió un emulador en marcha. | no |
| `set_emulator_down` | Deja caído un emulador en marcha, o lo vuelve a levantar. | sí |
| `list_runs` | Los informes de ejecuciones anteriores. | no |
| `compare_runs` | Dos ejecuciones una junto a otra. | no |
| `list_jobs` | Lo que está en marcha. | no |
| `stop_job` | Detiene una tarea en curso. | sí |

### Experimentos {#tools-experiments}

`describe_nodes` es lo que lee el asistente antes de escribir un experimento; es lo mismo que
[`signallab nodes`](cli.md#cli-nodes). `list_templates` y `get_template` dan ejemplos que
funcionan, para ejecutarlos o adaptarlos.

`validate_experiment` y `run_experiment` reciben el experimento de una de tres formas, exactamente
una de ellas:

| Argumento | Qué es |
| --- | --- |
| `document` | Un documento de experimento, tal como lo guarda la aplicación. |
| `file` | La ruta de un archivo de experimento en el equipo en el que funciona `signallab`. |
| `template` | El nombre de una plantilla incluida. |
| `params` | Valores de parámetros para esta ejecución: `{"name": "value"}`; los números y los booleanos se toman como texto. |
| `profile` | Ejecutar con este perfil del documento; `""` para los valores predeterminados. |
| `seed` | `run_experiment`: la semilla de los valores aleatorios. |
| `timeout` | `run_experiment`: los segundos que puede durar la ejecución, de 1 a 300 (300 por defecto). |

`run_experiment` responde cuando la ejecución ha terminado: superada, fallida o detenida, su
duración y su semilla, cada paso con lo que hizo o por qué falló, lo que se pidió a cada emulador,
lo que hizo cada relé de degradación y la ruta del informe. Una ejecución que falla es una
respuesta normal (los pasos dicen por qué), no una llamada fallida.

### Mensajes sueltos {#tools-send}

| Herramienta | Argumentos |
| --- | --- |
| `send_osc` | `target` (`host:port`), `address`, `args`: números (enteros → int32, o int64 fuera de su rango; si no, float32), cadenas, booleanos, `null`, o `{"type": "int"\|"float"\|"str"\|"long"\|"double"\|"bool"\|"blob"\|"nil", "value": …}`. |
| `send_udp` | `target`, y `text` o `hex` (`"de ad be ef"`). |
| `send_http` | `method`, `url`, `headers` (`{"Name": "value"}`), `body`, `timeout_ms` (10000 por defecto), `auth`: `{"scheme": "basic"\|"digest", "username", "password"}` o `{"scheme": "bearer", "token"}`. Devuelve el estado, el tiempo, los encabezados y el cuerpo (sus primeros 16 KiB). |
| `send_mqtt` | `broker` (`host:port`, puerto 1883 si no se indica), `topic`, `payload`, `qos` (0, 1 o 2), `retain`. Una carga útil vacía con `retain` borra un valor retenido. |
| `send_ws` | `url` (`ws://` o `wss://`), `text` o `hex`, `headers`, `protocols` y, para esperar la respuesta, `expect` (contiene), `expect_regex` o `wait` (cualquier mensaje); `timeout_ms` de 1 a 120000 (2000 por defecto). Devuelve el handshake, lo que se envió y la respuesta, con su JSON analizado cuando es JSON. |

Son los comandos que usan las pantallas de la aplicación; consulta
[`signallab send`](cli.md#cli-send).

### Escuchar {#tools-listen}

`listen` abre un puerto UDP **en el equipo en el que funciona `signallab mcp`**, durante un
tiempo, y devuelve lo que llegó: los mensajes OSC decodificados y los demás datagramas como texto y
hex.

| Argumento | Qué es | Predeterminado |
| --- | --- | --- |
| `bind` | `IP:port`, p. ej. `0.0.0.0:9000`. | obligatorio |
| `protocol` | `osc` o `udp`. | `osc` |
| `seconds` | Cuánto tiempo escuchar, de 0,1 a 60. | 5 |
| `max` | Parar tras este número de datagramas, de 1 a 1000. | 100 |

Cuando no llega nada a `0.0.0.0`, la respuesta recuerda al asistente que compruebe el firewall
([`signallab doctor`](cli.md#cli-doctor)). Con `--server`, se rechaza `listen`: en un servidor,
escucha allí un experimento con un nodo de espera.

### Señales y emuladores {#tools-library}

`list_signals` y `fire_signal` usan tu biblioteca de señales: el `signals.json` de la aplicación,
`--library` o una ruta `library` indicada en la llamada. Una señal se envía por su id o su nombre,
exactamente como la envía la aplicación.

`list_emulators` nombra los emuladores de tu biblioteca. `start_emulator` inicia uno (un documento
en `emulator`, o el id o el nombre de una entrada de la biblioteca en `name`) y devuelve su id de
tarea y su dirección; responde según sus reglas hasta `stop_job`. `bind` lo traslada a otro
`IP:port`, `params` da valores que leen sus plantillas y `seed` fija sus decisiones aleatorias.
`emulator_exchanges` (`job_id`, y `after` para solo los más recientes) muestra lo que llegó y lo
que respondió cada regla. `set_emulator_down` (`job_id`, `down` y `fault`: `unavailable`, `reset`
o `timeout`) desconecta un emulador en marcha hasta que se vuelve a levantar: HTTP encuentra el
fallo (`unavailable` responde 503), un dispositivo TCP y un bróker MQTT cortan las conexiones, y
OSC y UDP no responden nada. Consulta [Emuladores](../tools/emulators.md).

### Ejecuciones y tareas {#tools-runs}

`list_runs` lee los informes de ejecuciones anteriores, los más recientes primero (de un
experimento, cuando `experiment` lo nombra, y como máximo `limit`, de 1 a 500, 50 por defecto), con
las cifras de cada paso de carga. `compare_runs` recibe dos de sus nombres, `a` (antes) y `b`
(después), y pone una junto a otra las latencias, la tasa de errores, la tasa alcanzada y las
solicitudes omitidas de cada paso de carga, y marca como regresión un cambio del 5 % o más en el
sentido malo. Consulta [Ejecuciones e informes](../experiments/runs.md).

`list_jobs` muestra lo que está en marcha (monitores, generadores, emuladores, ejecuciones) y
`stop_job` detiene uno por su id.

## Resultados y errores {#results}

Cada respuesta es texto para el modelo y, además, lo mismo en forma de datos estructurados. Un fallo se marca
como error y lleva el error del motor (un `code` estable, sus valores, el nodo y el campo a los que
se refiere), expresado en el idioma elegido con `--lang`. Si el asistente da mal un argumento,
la respuesta lo explica con palabras que le permiten corregirlo.

## Progreso y cancelación {#progress}

Cuando el cliente pide el progreso de `run_experiment`, cada paso se notifica a medida que ocurre
(el nodo y su estado), así que el asistente (y tú también) puede ver cómo avanza la ejecución. Cancelar una llamada
la detiene; cancelar `run_experiment` detiene la propia ejecución, como hace [[ui:common.stop]] en
la aplicación.

Cuando el cliente cierra la conexión, las llamadas aún en curso terminan, y luego
`signallab mcp` sale.

## En un servidor de laboratorio {#on-a-server}

Con `--server http://192.0.2.10:1430`, los experimentos, los envíos, las señales y los emuladores
ocurren **en ese servidor**, a través de su API (con su red, sus secretos y su carpeta de datos),
así que el asistente llega a equipos a los que solo llega el laboratorio. Indica el token en el
entorno del cliente:

```json
{
  "mcpServers": {
    "signallab": {
      "command": "signallab",
      "args": ["mcp", "--server", "http://192.0.2.10:1430"],
      "env": { "SIGNALLAB_TOKEN": "<the server's token>" }
    }
  }
}
```

Lo que se queda en este equipo: las bibliotecas de señales y de emuladores (las de la aplicación,
o `--library` y `--emulators`) y los archivos que nombra una llamada (`file`, `library`) se leen
aquí, y lo que contienen se envía al servidor; `listen` se rechaza. Consulta
[Ejecutar Signal Lab como servidor](../server/index.md).

## Seguridad {#safety}

- El asistente solo puede hacer lo que hacen las herramientas, y cada herramienta es uno de los
  propios comandos de la aplicación: no puede llegar a nada a lo que no llegue la aplicación.
- Las herramientas que envían, escuchan o inician algo están marcadas como herramientas que llegan
  al mundo exterior; tu cliente decide si te pregunta antes de cada llamada.
- Los valores secretos nunca llegan al asistente: un experimento los nombra como
  `{{secret.NAME}}`, y cada resultado muestra `••••` en su lugar.
- Un emulador o una escucha abre un puerto en el equipo en el que funciona; `list_jobs` y
  `stop_job` muestran y terminan lo que sigue en marcha.

## Protocolo {#protocol}

Para quienes escriben clientes: JSON-RPC 2.0 sobre stdio, un mensaje por línea; stdout solo lleva
mensajes del protocolo, y todo lo dirigido a una persona va a stderr. Versiones del protocolo
`2025-06-18`, `2025-03-26` y `2024-11-05` (la más reciente cuando el cliente pide otra), lotes,
`ping`, `tools/list` y `tools/call`; progreso como `notifications/progress` para una llamada que
envió un `progressToken`, y cancelación mediante `notifications/cancelled`. Las `instructions` del
servidor le explican al modelo cómo encajan las herramientas.

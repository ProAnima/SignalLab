---
title: Línea de comandos
description: signallab ejecuta experimentos sin ventana, envía mensajes sueltos, hace de emulador y comprueba la red, desde un terminal, un script o una tarea de CI.
---

# La línea de comandos: `signallab`

`signallab` es Signal Lab sin ventana. Ejecuta los experimentos hasta el final y
sale con un código que un script entiende, envía un mensaje OSC, un datagrama,
una solicitud HTTP, un mensaje WebSocket o una publicación MQTT, lanza una señal
de tu biblioteca, hace de emulador hasta que lo detienes y dice qué se interpone
entre este equipo y los dispositivos.

Es el mismo motor que la aplicación: una ejecución desde la línea de comandos da
los mismos pasos, escribe el mismo informe y dice las mismas cosas, en los idiomas
de la interfaz. Con `--server`, las ejecuciones ocurren en su lugar en un
[servidor de Signal Lab](../server/index.md), con su red y sus secretos.

```bash
signallab run tests/smoke.json --param api=http://127.0.0.1:8080 --junit junit.xml
signallab run flaky-api                                # a bundled template, by name
signallab validate tests/*.json                        # the editor's check, nothing sent
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab fire "Fader value"
signallab emulate tests/orders-api.json --for 120      # play a dependency for two minutes
signallab doctor                                       # firewall, network, data folder
```

Para pipelines, consulta [Signal Lab en CI](ci.md); para un asistente de IA,
[`signallab mcp`](mcp.md).

## Instalación {#install}

| Dónde | Cómo obtienes `signallab` |
| --- | --- |
| Windows, el instalador (`.exe`) | Se instala junto a la aplicación, y esa carpeta se añade al `PATH`: el tuyo para una instalación «para mí», el del equipo para «para todos». Abre un terminal nuevo después de instalar. El modificador `/NOPATH` del instalador deja el `PATH` intacto. |
| Windows, el `.msi` | Se instala junto a la aplicación; la carpeta de instalación está en el `PATH` del equipo mientras la aplicación está instalada. |
| Linux, `.deb` y `.rpm` | `/usr/bin/signallab`. |
| Linux, AppImage | No se incluye: usa el archivo comprimido de abajo. |
| Cualquier equipo, sin la aplicación | Cada versión incluye `signallab-<version>-windows-x64.zip` y `signallab-<version>-linux-x64.tar.gz`, cada uno con el programa y su licencia; `SHA256SUMS.txt` en la misma página lista sus sumas de comprobación. |
| La imagen del servidor | `/usr/local/bin/signallab` en `ghcr.io/proanima/signallab` (consulta [CI](ci.md#docker)). |

Compruébalo con:

```bash
signallab version
```

## Comandos {#commands}

| Comando | Qué hace |
| --- | --- |
| [`run`](#cli-run) | Ejecuta experimentos uno tras otro; sale con 0 solo cuando todos han pasado. |
| [`validate`](#cli-validate) | Comprueba los experimentos como hace el editor antes de una ejecución; no envía nada. |
| [`send`](#cli-send) | Envía un mensaje: `osc`, `udp`, `http`, `ws` o `mqtt`. |
| [`fire`](#cli-fire) | Envía una señal de una biblioteca de señales, por su id o su nombre. |
| [`emulate`](#cli-emulate) | Hace el papel de una API HTTP, un dispositivo OSC, UDP o TCP, o un bróker MQTT hasta <kbd>Ctrl</kbd>+<kbd>C</kbd> o `--for`. |
| [`emulators`](#cli-emulators) | Lista los emuladores de la biblioteca de la aplicación. |
| [`templates`](#cli-templates) | Lista las plantillas de experimento incluidas. |
| [`nodes`](#cli-nodes) | Describe cada tipo de nodo, como JSON. |
| [`mcp`](#cli-mcp) | Sirve Signal Lab a un asistente de IA mediante el Model Context Protocol. |
| [`doctor`](#cli-doctor) | Comprueba el firewall, la red, la carpeta de datos y un servidor. |
| [`firewall`](#cli-firewall) | `firewall allow`: permite que otras máquinas lleguen a Signal Lab a través del Firewall de Windows. |
| [`version`](#cli-version) | Imprime la versión. |

`signallab help <command>` o `signallab <command> --help` imprime las opciones
de un comando.

## Opciones de todos los comandos {#global-options}

| Opción | Qué hace | Predeterminado |
| --- | --- | --- |
| `--lang <code>` | El idioma de los mensajes: `en`, `ru`, `es`, `fr`, `de`, `pt`, `zh`, `ja`, `ko`, `hi` o `ar`. | `SIGNALLAB_LANG`, si no, la configuración regional y, si no, `en` |
| `--json` | Salida para máquinas por stdout en lugar de texto (consulta [Salida](#output)). | desactivado |
| `-h`, `--help` | La ayuda del comando. | |
| `-V`, `--version` | La versión; antes de cualquier comando, como `signallab --version`. | |

Tanto `--lang` como `--json` pueden ir en cualquier parte de la línea:
`signallab --json run smoke.json` y `signallab run smoke.json --json` son lo mismo.

### Idioma {#language}

Los mensajes, los textos de los pasos, los fallos y el informe de JUnit usan los
propios textos de la interfaz, sus formas plurales y su estilo numérico. El idioma
es el primero de:

1. `--lang`;
2. `SIGNALLAB_LANG` (`ru`, `ru-RU` y `ru_RU.UTF-8` significan todos ruso);
3. la configuración regional: la primera de `LC_ALL`, `LC_MESSAGES` y `LANG` que esté definida;
4. inglés.

::: tip
Los terminales de Windows normalmente no definen ninguna de las variables de
configuración regional, así que `signallab` habla inglés allí a menos que definas
`SIGNALLAB_LANG` o pases `--lang`.
:::

### Salida para personas y para máquinas {#output}

Sin `--json`, los resultados van a **stdout** y todo lo que una persona lee por el
camino va a **stderr**: los pasos de una ejecución a medida que ocurren, por qué
falló algo y dónde está el informe. `signallab run … 2>/dev/null` deja una línea
de veredicto por ejecución.

Con `--json`, stdout lleva JSON y nada más, y stderr permanece en silencio:

| Comando | Qué imprime `--json` en stdout |
| --- | --- |
| `run` | Un objeto por línea: `started`, un `step` por paso, `ended` (el resultado de la ejecución) y luego `summary`; `error` para una ejecución que no pudo empezar. Consulta [Qué imprime `run`](#run-output). |
| `validate` | Un objeto por línea, por experimento: `valid`, y `profile_issues` o `error`. |
| `send`, `fire` | `{"type": "sent", "result": …}`; `send http` imprime `{"type": "response", "response": …}` y `send ws`, `{"type": "exchange", "result": …}`. |
| `emulate` | Un objeto por línea: `started`, un `exchange` por solicitud, un `summary` por emulador; `valid` con `--check`. |
| `emulators` | `{"path": …, "emulators": [{id, name, protocol, bind, rules, note}, …]}`. |
| `templates` | `[{"name": …, "experiment": …}, …]`. |
| `doctor` | Un objeto: `version`, `network`, `data_dir`, `firewall`, `server`, `problems`. |
| `version` | `{"version": "…"}`. |
| `nodes` | Siempre JSON, con o sin `--json`. |

Un fallo es `{"type": "error", "error": {…}, "exit_code": N}`. `error` es el error
del motor: un `code` estable (como `transport.refused` o `secret.missing`), sus
`params`, y el `node` y el `field` a los que se refiere. Un script puede ramificar
según `error.code` en cualquier idioma; los textos de cada código están en
[Mensajes de error](../reference/errors.md).

### Códigos de salida {#exit-codes}

| Código | Significado |
| --- | --- |
| `0` | Todos los experimentos pasaron; el envío se hizo; nada se interpone. |
| `1` | Un experimento se ejecutó y falló, se agotó el tiempo o se detuvo; un envío falló (rechazado, sin respuesta, un estado inesperado); `doctor` encontró algo que se interpone. |
| `2` | El comando o un documento está mal: un argumento, un archivo que no se puede leer, un error de validación, un parámetro desconocido, un secreto que falta. |
| `3` | No se pudo ejecutar nada por una razón ajena al experimento: no se puede alcanzar el servidor o rechaza el token, no se puede abrir un puerto, falla el almacén de credenciales. |

Con varios experimentos, el resultado más grave decide, en este orden: `2`, luego
`3`, luego `1` y luego `0`.

### Variables de entorno {#environment}

| Variable | Qué hace |
| --- | --- |
| `SIGNALLAB_LANG` | El idioma, cuando no se da `--lang`. |
| `LC_ALL`, `LC_MESSAGES`, `LANG` | El idioma, cuando no se define ninguno de los anteriores. |
| `SIGNALLAB_SERVER` | El servidor para `run`, `validate`, `emulate`, `mcp` y `doctor`, como lo da `--server`. |
| `SIGNALLAB_TOKEN` | El token de acceso del servidor, cuando no se da ningún archivo de token. |
| `SIGNALLAB_TOKEN_FILE` | Un archivo que contiene el token del servidor, como lo da `--token-file`. |
| `SIGNALLAB_SECRET_<NAME>` | El valor del secreto `NAME` para las ejecuciones en este proceso (consulta [Secretos](#secrets)). |
| `SIGNALLAB_DATA_DIR` | La carpeta de datos de la aplicación, donde `fire`, `emulators`, `emulate`, `mcp` y `doctor` buscan de forma predeterminada; si no, `Documents/SignalLab` en tu carpeta de inicio. |
| `GITHUB_ACTIONS` | Cuando es `true`, una ejecución que no pasa también se imprime como anotación `::error`, que GitHub muestra en la página de la ejecución. |

## `run` {#cli-run}

```text
signallab run [OPTIONS] <FILE>...
```

Ejecuta experimentos uno tras otro y sale con `0` solo cuando todos han pasado.

| Opción | Qué hace | Predeterminado |
| --- | --- | --- |
| `<FILE>...` | Archivos de experimento, o nombres de [plantillas incluidas](#cli-templates). | obligatorio |
| `-p`, `--param NAME=VALUE` | Un valor de parámetro para esta ejecución; repite para más. | los valores del documento |
| `--profile NAME` | Ejecutar con este perfil; todos los experimentos dados deben tenerlo. `""` ejecuta con los valores predeterminados. | el perfil del documento |
| `-m`, `--matrix NAME=V1,V2` | Ejecutar una vez por valor; repite para más nombres (consulta [Una matriz de ejecuciones](#matrix)). | |
| `--matrix-file PATH` | Combinaciones de un archivo JSON. | |
| `--seed N` | La semilla de los valores aleatorios, de 0 a 9007199254740991 (2⁵³ − 1). | la semilla del documento, si no, una nueva por ejecución |
| `--timeout SECONDS` | Falla una ejecución que tarde más, de 1 a 300. | `300` |
| `--fail-fast` | Detenerse en la primera ejecución que no pase; las demás no empiezan. | desactivado |
| `--junit PATH` | Escribir allí un informe JUnit XML (consulta [Informes](#reports)). | |
| `--report PATH` | Copiar allí el informe de la ejecución: un archivo para una ejecución, una carpeta para varias. | |
| `--data-dir PATH` | La carpeta de datos para las ejecuciones de este proceso; sus informes se quedan allí. | una carpeta temporal, que se elimina al salir |
| `--server URL` | Ejecutar en este servidor en lugar de en este proceso (consulta [En un servidor](#run-on-server)). | `SIGNALLAB_SERVER` |
| `--token-file PATH` | Un archivo que contiene el token del servidor. | `SIGNALLAB_TOKEN_FILE`, si no, `SIGNALLAB_TOKEN` |
| `--secrets files\|system` | De dónde salen los valores secretos en este proceso. | `files` |
| `--secrets-dir PATH` | Una carpeta de archivos de secretos, uno por nombre. | `/run/secrets/signallab` cuando existe |

`--data-dir`, `--secrets` y `--secrets-dir` se refieren a las ejecuciones en este
proceso; no se pueden combinar con `--server`.

### Archivos y plantillas {#run-inputs}

Un `FILE` es un experimento tal como lo guarda y exporta la aplicación
([[ui:exp.exportJson]] en la pantalla [[ui:nav.experiment]]). Sirve cualquier
versión del documento que abra la aplicación; una antigua se pone al día al
leerla, como hace la aplicación al abrirla. Un nombre que no es un archivo se
busca entre las [plantillas incluidas](#cli-templates), con o sin `.json`:

```bash
signallab run tests/stage-cues.json tests/api.json
signallab run osc-ping-reply --param device=192.0.2.20:9000
```

Cada experimento —y cada combinación de una matriz— se comprueba como lo comprueba
el editor antes de que empiece el primero. Un tercer archivo roto detiene también
el primero, antes de enviar nada, con código de salida `2`.

### Parámetros y perfiles {#run-params}

`--param NAME=VALUE` define un parámetro solo para esta ejecución; el archivo no
cambia. El valor es todo lo que sigue al primer `=`, así que
`--param url=http://127.0.0.1/?q=1` funciona, y `--param note=` define un valor
vacío. Un valor se aplica a cada experimento dado que tenga ese parámetro, y debe
nombrar un parámetro de al menos uno de ellos: un nombre mal escrito se rechaza
con código de salida `2`.

`--profile NAME` ejecuta con uno de los perfiles del experimento, como lo hace
elegirlo en [[ui:exp.profile]] en el editor. Consulta
[Datos y plantillas](../experiments/data.md).

### Una matriz de ejecuciones {#matrix}

Una matriz ejecuta el mismo experimento contra cada destino, cada usuario, cada
tamaño de carga útil. Cada combinación es una ejecución propia, con su propio
resultado, su propio informe y su propia suite en el informe de JUnit.

```bash
signallab run smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest \
  --fail-fast --junit junit.xml
```

Eso son cuatro ejecuciones: `device` varía más despacio, `user` más rápido. Se
nombran por el archivo y sus valores: `smoke.json [device=192.0.2.20:9000, user=admin]`.

- `--matrix NAME=V1,V2` añade un eje. El mismo nombre otra vez le añade valores.
  Los espacios alrededor de nombres y valores se descartan; un valor dado dos
  veces se ejecuta una vez.
- `--matrix-file PATH` lee JSON de una de dos formas:

  ```json
  { "device": ["192.0.2.20:9000", "192.0.2.21:9000"], "retries": [1, 3] }
  ```

  añade ejes: cada combinación se ejecuta, estos nombres después de los de
  `--matrix`, en orden alfabético;

  ```json
  [
    { "device": "192.0.2.20:9000", "user": "admin" },
    { "device": "192.0.2.21:9000", "user": "guest" }
  ]
  ```

  enumera las combinaciones mismas, cada una cruzada con los ejes de `--matrix`.
  Los valores son texto, números o `true`/`false`; una coma dentro de un valor en
  un archivo sigue formando parte de él.
- Cada nombre de matriz debe ser un parámetro de al menos un experimento dado. Un
  experimento sin uno de ellos se ejecuta una vez, no una vez por cada valor que
  ignoraría.
- Un nombre definido tanto por `--param` como por la matriz, o tanto por
  `--matrix` como por el archivo, se rechaza.
- Como máximo **256** ejecuciones de un comando; más se rechaza antes de que nada
  se ejecute.

### Ejecutar en un servidor {#run-on-server}

```bash
signallab run tests/stage.json --server http://192.0.2.10:1430 --token-file token.txt
```

El experimento viene de este equipo; el servidor lo ejecuta con su propia red, sus
propios [secretos](../server/index.md#secrets) y su propia carpeta de datos, y
transmite los pasos a medida que ocurren. El informe se queda en el servidor —su
ruta se imprime— y `--report` descarga una copia. Una ejecución en un servidor es
una tarea allí como una iniciada en su interfaz: cada página con sesión iniciada
la muestra. Si `signallab` desaparece a mitad de la ejecución, la ejecución
termina igualmente en el servidor y conserva su informe.

El token se lee de `--token-file` (o `SIGNALLAB_TOKEN_FILE`), si no de
`SIGNALLAB_TOKEN`, y se envía como `Authorization: Bearer`. Un servidor al que no
se puede llegar, que rechaza el token o que deja de responder a mitad de la
ejecución es código de salida `3`. `signallab doctor --server URL` comprueba la
dirección y el token por su cuenta.

### Informes {#reports}

Cada ejecución escribe el mismo informe que escribe la aplicación. Sin
`--data-dir`, las ejecuciones usan una carpeta temporal que se elimina cuando
`signallab` sale, así que guarda lo que necesites:

- `--report PATH` copia el informe: a `PATH` mismo para una ejecución, o a la
  carpeta `PATH` para varias, como `01-<file>.json`, `02-<file>.json`, … en el
  orden en que se ejecutaron.
- `--data-dir PATH` guarda el informe de cada ejecución en esa carpeta (en
  `runs/`), e imprime dónde está cada uno.

`--junit PATH` escribe un informe JUnit XML, el formato que lee cualquier sistema
de CI:

- un `<testsuite>` por ejecución (por combinación de una matriz), con el archivo,
  la semilla, el resultado, el perfil, la ruta del informe y cada valor de la
  matriz (`param.NAME`) como propiedades;
- un `<testcase>` por nodo que se ejecutó, nombrado por el tipo del nodo y su id,
  con su propio tiempo;
- un `<failure>` en el nodo que falló: el mensaje en el idioma elegido, el código
  de error como su `type`, y los pasos del nodo con el detalle técnico;
- un caso `<skipped>` por cada nodo al que la ejecución nunca llegó (el otro lado
  de una rama);
- una suite con un caso `<error>` para un experimento que no pudo empezar, y otra
  con un caso `<skipped>` por cada ejecución que `--fail-fast` no inició.

```xml
<testsuites name="Signal Lab" tests="6" failures="1" errors="0" time="2.006">
  <testsuite name="HTTP to OSC" tests="6" failures="1" errors="0" skipped="4" time="2.006" timestamp="2026-10-04T16:48:03">
    <properties>
      <property name="file" value="status-branch" />
      <property name="seed" value="42" />
      <property name="outcome" value="failed" />
      <property name="report" value="out/report.json" />
    </properties>
    <testcase name="Start (start)" classname="HTTP to OSC" time="0.001">
      <system-out>Started · seed 42</system-out>
    </testcase>
    <testcase name="HTTP request (request)" classname="HTTP to OSC" time="2.004">
      <failure message="http://127.0.0.1:8080/ refused the connection — nothing is listening on that port" type="transport.refused">…</failure>
    </testcase>
    <testcase name="Status branch (branch)" classname="HTTP to OSC" time="0.000">
      <skipped message="not reached in this run" />
    </testcase>
    …
  </testsuite>
</testsuites>
```

Un nodo HTTP bajo [carga](../experiments/load.md) añade una línea bajo su último
paso por cada umbral, cumplido o no (`✕ p95 < 100 ms · 152.58 ms`); un umbral que
no se cumple falla la ejecución y su caso de JUnit (`type="load.threshold"`).

### Secretos {#secrets}

Un experimento lee un secreto como `{{secret.NAME}}`. Para las ejecuciones en este
proceso, el valor viene de:

| `--secrets` | De dónde viene el valor de `NAME` |
| --- | --- |
| `files` (el predeterminado) | La variable de entorno `SIGNALLAB_SECRET_NAME`; si no, un archivo llamado `NAME` en `--secrets-dir` (de forma predeterminada `/run/secrets/signallab`, cuando esa carpeta existe: la disposición de secretos de Docker). |
| `system` | El Administrador de credenciales de Windows, donde la aplicación guarda los valores de sus [[ui:exp.secrets]]. Linux no tiene ningún almacén de credenciales que `signallab` lea: allí `--secrets system` falla con código de salida `3`. |

El salto de línea final de un archivo no forma parte del valor, y un valor vacío
cuenta como no definido. Los nombres son letras, dígitos y `_`, sin empezar por
dígito, de hasta 128 caracteres; un valor es como máximo de 16 KiB.

```bash
SIGNALLAB_SECRET_API_TOKEN="$API_TOKEN" signallab run tests/api.json
```

Un secreto que no está definido detiene la ejecución antes de cualquier tráfico,
con código de salida `2` y el nombre que falta. Un valor nunca se imprime: los
pasos, los errores, los informes y el informe de JUnit muestran `••••` en su
lugar. Con `--server`, los secretos son los del servidor.

### Qué imprime `run` {#run-output}

A medida que avanza una ejecución, cada paso es una línea en stderr —su tiempo
desde el inicio, el nodo, su estado y lo que hizo—, como la línea de tiempo de la
aplicación. Cuando termina, una línea en stdout dice cómo fue:

```text
▶ HTTP check (http-check) · seed 1185927457137919
     0.000  Start         Running
     0.000  Start         Passed · Started · seed 1185927457137919
     0.000  HTTP request  Running
     2.004  HTTP request  Failed · URL — http://127.0.0.1:8080/ refused the connection — nothing is listening on that port
✖ HTTP check failed after 2 s: HTTP request · URL — http://127.0.0.1:8080/ refused the connection — nothing is listening on that port
  Technical details: error sending request for url (http://127.0.0.1:8080/): … (os error 10061)
  To run it again with the same random values: --seed 1185927457137919
```

Una ejecución fallida nombra la semilla que usó: `--seed` con ese número la
ejecuta de nuevo con los mismos valores aleatorios. Tras el veredicto vienen, en
stderr, lo que se pidió a cada emulador de la ejecución —solicitudes, cuántas no
las tomó ninguna regla, cuántas fallaron y los aciertos por regla—:

```text
✔ Retry a flaky API passed in 7 ms
  Flaky API: 3 requests, 0 without a rule, 0 failed · #1 3
```

y lo que hizo cada relé de degradación: lo que recibió, descartó y limitó, y cada
fase como reenviado / recibido:

```text
  127.0.0.1:19110 → 127.0.0.1:19100: 155 datagrams, 38 dropped, 0 throttled · lan 0.0–2.0 s 39/39, wifi 2.0–4.0 s 39/39, offline 4.0–6.0 s 0/38, lan 6.0–8.0 s 38/38
```

Un relé sobre TCP mueve fragmentos de flujos, no datagramas, y no descarta nada,
así que su línea cuenta fragmentos, conexiones, reinicios, conexiones a medio
abrir y las veces que un flujo se vio retenido por el límite de ancho de banda:

```text
  127.0.0.1:19120 → 127.0.0.1:19101: 14 chunks, 2 connections, 1 reset, 0 half-open, 3 held back
```

Un nodo HTTP bajo [carga](../experiments/load.md) informa de su progreso como un
paso como máximo una vez por segundo.

Con varias ejecuciones, una línea por ejecución y un recuento cierran la salida:
`3 runs: 2 passed, 1 failed`. Con `--fail-fast`, las ejecuciones que no inició se
cuentan en stderr.

Con `--json`, cada línea es un objeto con un `type`:

```json
{"type":"started","experiment":"Empty experiment","file":"empty","job_id":1,"overridden":false,"profile":null,"seed":1,"started_ms":1791132204585}
{"type":"step","node_id":"start","state":"passed","detail":"Started","message_key":"exp.step.started","message_params":{"seed":1},"job_id":1,"ts":1791132204585}
{"type":"ended","experiment":"Empty experiment","file":"empty","outcome":"passed","seed":1,"params":{},"steps":[…],"report_path":"…","started_ms":1791132204585,"ended_ms":1791132204585,…}
{"type":"summary","total":1,"passed":1,"failed":0,"not_started":0,"exit_code":0}
```

Las líneas `started`, `ended` y `error` llevan `file` (el argumento tal como se
dio) y, en una matriz, `matrix` (los valores de la combinación). `ended` es todo
el resultado de la ejecución: `outcome` (`passed`, `failed` o `stopped`), `seed`,
`profile`, `params`, `error`, cada paso, `emulators` e `impairments` cuando la
ejecución los tenía, y `report_path`. El último paso de una carga lleva `load`
con cada número que midió: `planned`, `sent`, `ok`, `failed`, `missed`, `rps`,
`error_rate`, `min_ms`, `mean_ms`, `max_ms`, `p50_ms` a `p99_ms`, `statuses`,
cada segundo (`seconds`), el `histogram` y el veredicto de cada umbral
(`thresholds`).

## `validate` {#cli-validate}

```text
signallab validate [OPTIONS] <FILE>...
```

Comprueba los experimentos como lo hace el editor antes de una ejecución —el
grafo, cada campo, las plantillas, los parámetros y los secretos— y no envía
nada. Sale con `0` cuando todos podrían empezar.

Toma las entradas de [`run`](#cli-run): archivos y plantillas, `--param`,
`--profile`, `--matrix`, `--matrix-file`, y `--server`, `--token-file`,
`--secrets`, `--secrets-dir`. Con `--server`, el servidor las comprueba, contra
sus propios secretos.

```text
✔ tests/stage.json: Stage cues would run
  Rehearsal: Would not run: …
```

Un problema que solo tiene otro perfil del documento se lista bajo él, pero no
falla la comprobación. Con `--json`, una línea por experimento (por combinación):
`{"experiment", "file", "valid": true, "profile_issues": […]}`, o
`"valid": false` con `error` y `exit_code`.

## `send` {#cli-send}

```text
signallab send <osc|udp|http|ws|mqtt> …
```

Envía un mensaje a través de los mismos comandos que usan las pantallas de la
aplicación, así que son los mismos bytes en la red. Un envío no lee ninguna
biblioteca ni ningún secreto.

Códigos de salida: `0` enviado, `1` el envío falló, `2` un argumento está mal.

Un `<host:port>` es una dirección IP o un nombre de host, y un puerto:
`127.0.0.1:9000`, `[::1]:9000` (una dirección IPv6 entre corchetes) o
`device.local:9000`. Un nombre se resuelve cuando se ejecuta el comando, y se
toma su dirección IPv4 cuando la tiene, así que `localhost:9000` llega a un
receptor en `127.0.0.1`. Un nombre que no se resuelve falla el envío (`1`); un
destino sin puerto es un argumento no válido (`2`).

### `send osc` {#cli-send-osc}

```text
signallab send osc <host:port> <address> [ARG]...
```

Un mensaje OSC. Los argumentos se tipan con un prefijo, o se infieren:

| Argumento | Tipo OSC |
| --- | --- |
| `i:3` | int32 |
| `f:0.5` | float32 |
| `d:1.5` | float64 (double) |
| `h:64` | int64 |
| `s:text` | cadena |
| `b:de ad be ef` | blob, como bytes hex |
| `T`, `F` | true, false |
| `N` | nil |
| `3`, `-3` | un entero simple es int32 |
| `2.5` | un decimal simple es float32 |
| cualquier otra cosa | cadena |

```bash
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main 3
# ✔ sent /cue/go (32 bytes) → 127.0.0.1:9000
```

Entrecomilla un argumento con espacios: `"s:hello world"`. `s:7` envía el texto
`7`. La dirección debe empezar por `/`; una que no lo haga es un argumento no
válido (`2`).

::: warning Git Bash on Windows
Git Bash reescribe los argumentos que empiezan por `/` a rutas de Windows, así
que `/cue/go` llega como `C:/Program Files/Git/cue/go`. Escribe `//cue/go`, o
ejecuta con `MSYS_NO_PATHCONV=1`. PowerShell y `cmd` no se ven afectados.
:::

Consulta [OSC](../protocols/osc.md).

### `send udp` {#cli-send-udp}

```text
signallab send udp <host:port> (--text TEXT | --hex HEX)
```

Un datagrama UDP. Da la carga útil como `--text`, o como bytes `--hex`:
`"de ad be ef"`, `deadbeef` o `0xDE,0xAD`.

```bash
signallab send udp 127.0.0.1:7000 --text "PLAY 1"
signallab send udp 127.0.0.1:7000 --hex "de ad be ef"
```

### `send http` {#cli-send-http}

```text
signallab send http <METHOD> <URL> [OPTIONS]
```

Una solicitud HTTP. La línea de estado va a stderr —`HTTP 200 OK · 3 ms · 1,234 B`—
y el cuerpo de la respuesta a stdout, para que se pueda canalizar. Un cuerpo de
más de 256 KiB se corta ahí, y stderr lo dice.

| Opción | Qué hace | Predeterminado |
| --- | --- | --- |
| `-H`, `--header "Name: value"` | Un encabezado de la solicitud; repite para más. | |
| `--body TEXT` | El cuerpo de la solicitud. `@FILE` envía el contenido de un archivo de texto. | ninguno |
| `--expect-status STATUS` | Salir con `1` a menos que la respuesta tenga este estado. | cualquier estado es `0` |
| `--timeout MS` | Milisegundos que esperar la respuesta. | `10000` |
| `-u`, `--user NAME:PASSWORD` | Credenciales, enviadas como Basic. | |
| `--digest` | Con `--user`: responder al desafío Digest del servidor en su lugar (MD5 o SHA-256). | desactivado |
| `--bearer TOKEN` | Enviar `Authorization: Bearer TOKEN`. No junto con `--user`. | |

```bash
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab send http POST http://127.0.0.1:8080/cue -H "Content-Type: application/json" --body '{"cue": 1}'
signallab send http GET http://127.0.0.1:8080/admin -u admin:secret --digest
```

Sin `--expect-status`, cualquier respuesta es un éxito, un `500` incluido. Una
solicitud que no obtiene respuesta (rechazada, agotada, un nombre que no se
resuelve) es código de salida `1`, y también un desafío Digest que no se pudo
responder: el motivo se imprime después de la respuesta.

::: tip
Los argumentos son visibles para otros usuarios del mismo equipo. Reserva las
contraseñas reales para los experimentos, donde son [secretos](#secrets).
:::

Consulta [HTTP](../protocols/http.md).

### `send ws` {#cli-send-ws}

```text
signallab send ws <URL> [OPTIONS]
```

Un intercambio WebSocket: conectar a `ws://…` o `wss://…`, enviar un mensaje,
esperar opcionalmente la respuesta, cerrar. El handshake y lo que se envió van a
stderr, la respuesta a stdout (los mensajes binarios como hex).

| Opción | Qué hace | Predeterminado |
| --- | --- | --- |
| `--text TEXT` | Enviar este mensaje de texto. | no se envía nada |
| `--hex HEX` | Enviar estos bytes como mensaje binario. No junto con `--text`. | |
| `-H`, `--header "Name: value"` | Un encabezado para la solicitud de mejora; repite para más. | |
| `--protocol NAME` | Un subprotocolo que ofrecer; repite para más, en orden de preferencia. | |
| `--expect TEXT` | Esperar un mensaje que contenga este texto. | |
| `--expect-regex REGEX` | Esperar un mensaje que coincida con esta expresión regular. | |
| `--wait` | Esperar cualquier mensaje. | |
| `--timeout MS` | Milisegundos que esperar la respuesta. | `2000` |

```bash
signallab send ws ws://127.0.0.1:9001/echo --text '{"ping": 1}' --expect '"ping"'
signallab send ws ws://127.0.0.1:9001/feed --wait        # nothing sent: the server's first message
```

Sin `--expect`, `--expect-regex` ni `--wait`, conecta, envía y cierra sin esperar.
Cuando la respuesta esperada no llega a tiempo, el código de salida es `1`.
Consulta [WebSocket](../protocols/websocket.md).

### `send mqtt` {#cli-send-mqtt}

```text
signallab send mqtt <host:port> <topic> [payload] [--qos 0|1|2] [--retain]
```

Una publicación MQTT 3.1.1 sobre TCP sin cifrar, sin credenciales, como un cliente
propio con un id de cliente nuevo, así que nunca derriba una conexión que ya está.
Sin puerto, el bróker está en `1883`. Un tema es un tema: si contiene un `+` o un
`#`, o está vacío, el comando se rechaza antes de conectar (`2`).

| Opción | Qué hace | Predeterminado |
| --- | --- | --- |
| `[payload]` | La carga útil. | vacía |
| `--qos 0\|1\|2` | La calidad de servicio. | `0` |
| `--retain` | Conservarla como el valor retenido del tema. Una carga útil vacía con `--retain` lo borra. | desactivado |

```bash
signallab send mqtt 127.0.0.1:1883 lab/light/1/set on --qos 1
signallab send mqtt 127.0.0.1 lab/light/1/state "" --retain     # clear the retained value
```

Consulta [MQTT](../protocols/mqtt.md).

## `fire` {#cli-fire}

```text
signallab fire <signal> [--library PATH]
```

Envía una señal de una biblioteca de señales exactamente como la envía la pantalla
[[ui:nav.signals]] de la aplicación: señales OSC, UDP, HTTP y MQTT. La señal se
busca por su id y, si no, por su nombre, sin distinguir mayúsculas; un nombre que
comparten varias señales se rechaza con sus ids.

| Opción | Qué hace | Predeterminado |
| --- | --- | --- |
| `<signal>` | El id o el nombre de la señal. | obligatorio |
| `--library PATH` | El archivo de la biblioteca. | `signals.json` en la carpeta de datos de la aplicación |

```bash
signallab fire "Fader value"
signallab fire go --library show/signals.json
```

La biblioteca solo se lee, nunca se crea ni se cambia. Consulta
[Señales](../tools/signals.md).

## `emulate` {#cli-emulate}

```text
signallab emulate [OPTIONS] <FILE|NAME>...
```

Hace el papel del otro lado —una API HTTP, un dispositivo OSC, UDP o TCP, un
bróker MQTT— hasta <kbd>Ctrl</kbd>+<kbd>C</kbd> o `--for`, e imprime cada
solicitud a medida que la responde. Un `FILE` contiene un emulador, una lista de
ellos o toda una biblioteca tal como la escribe la aplicación; un `NAME` es el id
o el nombre de uno de la biblioteca de la aplicación (la de la pantalla
[[ui:nav.emulators]]).

| Opción | Qué hace | Predeterminado |
| --- | --- | --- |
| `-p`, `--param NAME=VALUE` | Un valor que sus plantillas leen como `{{NAME}}`; repite para más. | |
| `--bind IP:PORT` | Escuchar allí en su lugar. Solo con un emulador. | la del propio emulador |
| `--for SECONDS` | Detenerse tras este tiempo. | hasta <kbd>Ctrl</kbd>+<kbd>C</kbd> |
| `--seed N` | La semilla de sus elecciones aleatorias: un orden aleatorio, jitter, generadores. | |
| `--check` | Comprobar los emuladores y salir, sin abrir ningún puerto. | desactivado |
| `--library PATH` | La biblioteca en la que se buscan los nombres. | `emulators.json` en la carpeta de datos de la aplicación |
| `--server URL`, `--token-file PATH` | Iniciarlos en un servidor, seguirlos a través de su API y detenerlos al final. | |
| `--secrets`, `--secrets-dir` | Como para [`run`](#cli-run). | |

```text
$ signallab emulate tests/orders-api.json --for 60
Orders API (http) answering on 127.0.0.1:18099
answering for 60 s
+   1.209 s Orders API  #1  GET /orders/42 → 200 OK · 12 B  1 ms  ← 127.0.0.1:55744
+   1.209 s Orders API  —  GET /nothing → 404 Not Found · 20 B  2 ms  ← 127.0.0.1:55745
Orders API: 2 requests, 1 without a rule, 0 failed · #1 1
```

Cada línea es el tiempo desde el inicio, el emulador, la regla que respondió
(`#1`, o `—` para ninguna), la solicitud y lo que obtuvo, el tiempo que tardó y
quién la envió. Al final, los recuentos de cada emulador: solicitudes, cuántas no
las tomó ninguna regla, cuántas fallaron, cuántas encontraron una caída o se
quedaron sin entregar cuando las hubo, y los aciertos por regla.

Códigos de salida: `0` cuando se detiene con <kbd>Ctrl</kbd>+<kbd>C</kbd> o
`--for`; `2` cuando un emulador no es válido; `3` cuando su puerto está ocupado o
no se puede abrir, o falla un socket mientras responde.

En un pipeline, inícialo en segundo plano, prueba el sistema contra él y lee los
recuentos al final:

```bash
signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
npm test          # the system under test, configured for the emulator's address
wait              # the last lines of mock.ndjson are the counts
```

En un servidor, un emulador iniciado con `--server` se detiene cuando `signallab`
termina con normalidad; uno que deja atrás un proceso terminado a la fuerza se
puede detener desde la interfaz del servidor. Consulta
[Emuladores](../tools/emulators.md).

## `emulators` {#cli-emulators}

```text
signallab emulators [--library PATH]
```

Lista los emuladores de la biblioteca: id, nombre, protocolo, dirección y cuántas
reglas. `--library PATH` lee otro archivo de biblioteca en lugar de
`emulators.json` en la carpeta de datos de la aplicación. `signallab emulate <id>`
inicia uno.

La biblioteca solo se lee. Si no existe tal archivo —la aplicación crea el de su
carpeta de datos la primera vez que se inicia—, el comando lo dice y sale con `2`.

## `templates` {#cli-templates}

```text
signallab templates
```

Lista las plantillas incluidas, que `run` y `validate` toman por su nombre. Son
las de la propia aplicación:

| Nombre | En la aplicación | Parámetros |
| --- | --- | --- |
| `empty` | [[ui:exp.templateEmpty]] | |
| `http-check` | [[ui:exp.templateHttp]] | |
| `status-branch` | [[ui:exp.templateBranch]] | |
| `parallel-flows` | [[ui:exp.templateParallel]] | |
| `osc-ping-reply` | [[ui:exp.templatePingReply]] | `device` = `127.0.0.1:9000` |
| `poll-until-ready` | [[ui:exp.templatePoll]] | `device` = `127.0.0.1:9000` |
| `flaky-api` | [[ui:exp.templateFlaky]] | `api` = `http://127.0.0.1:18080` |
| `fault-phases` | [[ui:exp.templateFaults]] | |
| `dependency-outage` | [[ui:exp.templateOutage]] | `api` = `http://127.0.0.1:18090` |
| `websocket-echo` | [[ui:exp.templateWsEcho]] | `service` = `ws://127.0.0.1:9001/echo` |

Todas las plantillas apuntan a loopback. `flaky-api`, `fault-phases` y
`dependency-outage` traen sus propios emuladores, así que se ejecutan sin nada más
escuchando: una forma rápida de ver `signallab` funcionar.

## `nodes` {#cli-nodes}

```text
signallab nodes
```

Imprime, como JSON, de qué se compone un experimento: la forma y las reglas del
documento, cada tipo de nodo con su etiqueta, descripción, campos, salidas y un
ejemplo que el motor acepta, el lenguaje `{{template}}`, los perfiles de carga y
el documento del emulador. Es lo que lee un asistente a través de
[`signallab mcp`](mcp.md) para escribir un experimento; para las personas,
[Nodos](../experiments/nodes.md) dice lo mismo con más palabras.

## `mcp` {#cli-mcp}

```text
signallab mcp [OPTIONS]
```

Sirve Signal Lab a un asistente de IA mediante el Model Context Protocol, por
stdin y stdout. Todo —configurarlo en Claude Code, Claude Desktop, Cursor o VS
Code, sus opciones y sus herramientas— está en [Asistentes (MCP)](mcp.md).

## `doctor` {#cli-doctor}

```text
signallab doctor [--server URL] [--token-file PATH]
```

Dice qué podría interponerse entre Signal Lab y los dispositivos, y sale con `1`
cuando algo lo hace. Sus líneas siguen [`--lang`](#language) como el resto de la
línea de comandos:

- la red en la que está este equipo: su nombre y su dirección;
- la carpeta de datos: si se puede escribir en ella (una que aún no existe no pasa
  nada: la aplicación la crea en el primer uso);
- el firewall: en Windows, para `signallab` y para la aplicación de escritorio
  respectivamente, si una regla deja entrar a otras máquinas en el tipo de red en
  el que está el equipo ahora (privada, dominio o pública), o si una regla las
  bloquea —lo que deja un *Cancelar* en el aviso del sistema—; en Linux, si ufw o
  firewalld está activo y el comando que abre un puerto;
- con `--server` (o `SIGNALLAB_SERVER`): si el servidor responde y acepta el
  token.

```text
signallab [[version]]
Network: LAB-PC · 192.0.2.15
Data folder: C:\Users\lab\Documents\SignalLab — writable
Firewall · signallab (C:\…\Signal Lab\signallab.exe) · private network: not allowed yet — signallab firewall allow
Firewall · app (C:\…\Signal Lab\signal-lab.exe) · private network: allowed
✖ 1 thing in the way
```

`--json` imprime lo mismo como un solo objeto. Consulta
[Solución de problemas](../reference/troubleshooting.md).

## `firewall` {#cli-firewall}

```text
signallab firewall allow [--public]
```

En Windows, permite que otras máquinas lleguen a Signal Lab —lo que necesita un
monitor o una espera para oír un dispositivo. Windows pide primero derechos de
administrador; luego las reglas de entrada de `signallab` y de la aplicación de
escritorio (que se encuentra junto a ella, o donde la ponen los instaladores) se
sustituyen por una regla de permiso cada una, incluidas las reglas de bloqueo, en
redes privadas y de dominio.

| Opción | Qué hace |
| --- | --- |
| `--public` | También en redes públicas: el Wi-Fi de un local suele serlo. |

Códigos de salida: `0` hecho; `3` cuando se rechaza el aviso de administrador o
el cambio falla. En Linux no se cambia nada: imprime el comando de ufw o firewalld
que abre los puertos en los que escuchas, y sale con `0`.

El firewall solo cambia cuando ejecutas esto; nada más en `signallab` lo toca.

## `version` {#cli-version}

```text
signallab version
```

Imprime `signallab [[version]]` —con `--json`, `{"version": "[[version]]"}`.
`signallab --version` imprime la misma versión.

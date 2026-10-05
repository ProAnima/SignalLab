---
title: Archivos y carpetas
description: Dónde guarda Signal Lab sus archivos — la carpeta de datos, el experimento, las bibliotecas de señales y emuladores, los informes de ejecución y las exportaciones — sus formatos, y qué es seguro editar, respaldar y mover.
---

# Archivos y carpetas

Todo lo que guarda Signal Lab es JSON (o texto) simple en una sola carpeta, la
carpeta de datos. Los valores de los secretos nunca están en ella.

## La carpeta de datos {#data-folder}

| Dónde se ejecuta Signal Lab | La carpeta de datos |
| --- | --- |
| Aplicación de escritorio, Windows | `Documents\SignalLab` en tu carpeta de usuario: `C:\Users\<you>\Documents\SignalLab` |
| Aplicación de escritorio, Linux | `~/Documents/SignalLab` |
| Servidor | `--data-dir`, o `SIGNALLAB_DATA_DIR`; sin ninguno de los dos, `Documents/SignalLab` en la carpeta personal del usuario con el que se ejecuta |
| Servidor, imagen de Docker | `/data`, un volumen (`signallab-data` en el archivo de compose) |
| `signallab run` | Una carpeta temporal, que se elimina al salir — salvo que `--data-dir` indique una |

La aplicación de escritorio también toma `SIGNALLAB_DATA_DIR` de su entorno
cuando está definida. La carpeta se crea cuando se escribe algo en ella por
primera vez.

::: tip
En Windows la aplicación usa la carpeta `Documents` directamente dentro de tu
carpeta de usuario, incluso cuando Windows guarda tus documentos en otro sitio
(OneDrive).
:::

En un servidor, los archivos se escriben en la máquina del servidor, no en la
tuya. La insignia [[ui:app.server]] de la cabecera dice dónde, en su consejo; la
API lo da como `data_dir` de [`app_info`](../api/commands.md#app_info), y
[`/api/files`](../api/index.md#files) descarga lo que hay en ella.

La línea de comandos `signallab emulate`, `signallab send` y `signallab mcp` leen
las bibliotecas de la aplicación de la misma carpeta que la aplicación de
escritorio.

## Qué contiene {#contents}

| Archivo | Qué es | Se escribe |
| --- | --- | --- |
| `experiment.json` | El experimento abierto en el editor | Poco después de cada cambio |
| `signals.json` | La biblioteca de señales ([[ui:nav.signals]]) | Poco después de cada cambio |
| `emulators.json` | La biblioteca de emuladores ([[ui:nav.emulators]]) | Poco después de cada cambio |
| `runs/run-<ms>-<job>.json` | Un informe por cada ejecución que terminó por su cuenta | Cuando termina la ejecución |
| `exports/experiment-<ms>-<16 hex digits>.json` | Una instantánea del experimento | [[ui:exp.exportJson]] |
| `capture-<ms>.jsonl`, `capture-<ms>.txt` | Las tramas del Inspector | [[ui:ins.exportJsonl]], [[ui:ins.exportTxt]] |
| `token` | El token de acceso de un servidor, legible solo por su usuario | `--generate-token`, en el primer inicio |
| `.experiment-<hex>.tmp`, `.signals-<hex>.tmp`, `.emulators-<hex>.tmp` | Un guardado en camino | Por un momento, y luego se renombra |

`<ms>` es una hora en milisegundos desde 1970; `<job>` es el número de tarea de
la ejecución. En un servidor, cada navegador trabaja con el mismo
`experiment.json`, las mismas bibliotecas y los mismos informes.

## Formatos {#formats}

Todos son JSON en UTF-8, escrito con sangrado para que se lean y se comparen
bien. Cada uno lleva una `version`; un archivo de una versión anterior se lee y
se migra cuando se abre, y se vuelve a escribir en la versión actual la próxima
vez que se guarda — después de lo cual una versión anterior de Signal Lab no
puede abrirlo.

### experiment.json {#experiment-json}

El documento del experimento, versión 9 — el mismo JSON que
[[ui:exp.exportJson]] escribe y [[ui:exp.importJson]] lee:

```json
{
  "version": 9,
  "name": "HTTP check",
  "params": [],
  "profiles": [],
  "profile": null,
  "seed": null,
  "cookies": true,
  "nodes": [ { "id": "start", "type": "start", "x": 40, "y": 80 }, … ],
  "edges": [ { "from": "start", "to": "request", "port": "next" }, … ]
}
```

- Como máximo 4 MiB, y de 1 a 64 nodos (`doc.node_count`).
- Las versiones 1 a 8 se migran al abrir. Un archivo anterior a la versión 8 se
  abre con `cookies` desactivado, para que se ejecute como lo hacía; los demás
  ajustes que añadió cada versión (parámetros en 2, perfiles en 3, reintentos en
  4, repeticiones y bucles en 5, emuladores en 6, degradaciones en 7, WebSocket
  y autenticación HTTP en 8, carga en 9) empiezan vacíos.
- Una versión más reciente que la que conoce este Signal Lab se rechaza
  (`doc.version_unsupported`) en lugar de abrirse sin lo que no puede leer.
- Un archivo que no se analiza se reporta con su ruta, línea y columna, y nunca
  se reemplaza.
- Se escribe en un archivo temporal y se renombra, así que un guardado fallido
  deja el anterior.

Qué son los nodos, los parámetros y los perfiles: [experimentos](../experiments/index.md),
[nodos](../experiments/nodes.md), [datos](../experiments/data.md).

### signals.json {#signals-json}

La biblioteca de señales, versión 2:

```json
{
  "version": 2,
  "signals": [
    {
      "id": "…",
      "name": "Go cue",
      "group": "Stage/Cues",
      "note": "",
      "body": { "transport": "osc", "target": "127.0.0.1:9000", "address": "/cue/go", "args": [ { "type": "int", "value": 1 } ] }
    }
  ],
  "folders": [ "Stage", "Stage/Cues" ]
}
```

- `group` es la carpeta de la señal como una ruta con `/`; vacío es el nivel
  superior. `folders` (añadido en la versión 2) lista cada carpeta, incluidas las
  vacías, y se omite cuando no hay ninguna. Un archivo de la versión 1 se lee
  igual, sin carpetas vacías.
- `body` es uno de `osc`, `udp`, `http` o `mqtt`; sus campos están en
  [`signals_save`](../api/commands.md#signals_save).
- Cuando el archivo no existe, se escribe el conjunto inicial — todos los
  destinos en `127.0.0.1` — y se renombra al idioma de la interfaz.
- Un archivo que no se analiza se reporta con su ruta, línea y columna
  (`signals.json_invalid`) y nunca se reemplaza con el conjunto inicial:
  corrígelo o bórralo. Nada escribe la biblioteca mientras no se lea — un
  guardado se rechaza con el mismo error y el archivo se queda como está —
  hasta que [[ui:sig.reload]] lo lea otra vez. Un guardado pasa por un archivo
  temporal en la misma carpeta, así que una escritura interrumpida deja el
  archivo anterior.

### emulators.json {#emulators-json}

La biblioteca de emuladores, versión 1:

```json
{
  "version": 1,
  "emulators": [
    { "id": "demo-api", "note": "…", "emulator": { "name": "Demo API", "bind": "127.0.0.1:8080", "protocol": "http", "routes": [ … ] } }
  ]
}
```

Cada entrada es un documento de emulador con un `id` y una `note`; el documento
se describe en [emuladores](../tools/emulators.md). Igual que con las señales, un
archivo ausente recibe el conjunto inicial (todos vinculados a `127.0.0.1`), y
uno roto se reporta (`emulators.json_invalid`), nunca se reemplaza. Se escribe a
través de un archivo temporal. `signallab emulate` también lee un archivo propio
que contiene un emulador, una lista de ellos, o una biblioteca como esta.

### Informes de ejecución {#run-reports}

`runs/run-<started ms>-<job>.json`, informe versión 5: un archivo por cada
ejecución que se superó o falló, nunca se escribe encima (una segunda ejecución
con el mismo nombre recibe `-2`, `-3`… añadidos). Una ejecución detenida no
guarda ninguno.

| Campo | Qué es |
| --- | --- |
| `version` | 5 |
| `experiment` | El nombre del experimento |
| `document_version` | La versión del documento que se ejecutó |
| `seed`, `profile` | Con qué se ejecutó |
| `overrides` | Valores dados solo para esta ejecución |
| `params` | Cada valor de parámetro que usó |
| `started_ms`, `ended_ms` | Milisegundos desde 1970 |
| `outcome` | `passed` o `failed` |
| `error` | Su primer fallo, o null |
| `steps` | Cada paso, como [`experiment://step`](../api/events.md#event-experiment-step) |
| `emulators` | Lo que recibió y respondió cada nodo [[ui:exp.node.emulator]] (desde la versión 3); se omite cuando no hay ninguno |
| `impairments` | Lo que hizo el relé de cada nodo [[ui:exp.node.impairment]], fase por fase (desde la versión 4); se omite cuando no hay ninguno |

Las mediciones de un paso de carga están en su último paso (desde la versión 5).
El historial de ejecuciones de la línea de tiempo y [[ui:exp.compare]] leen estos
archivos; un informe que no se puede leer queda fuera de la lista. Consulta
[ejecuciones e informes](../experiments/runs.md).

### Exportaciones {#exports}

- `exports/experiment-…json`: el documento del experimento, como arriba. Cada
  exportación es un archivo nuevo.
- `capture-….jsonl`: una trama del Inspector por línea, con los bytes que
  conserva en `data`, en base64.
- `capture-….txt`: las tramas para leer, cada una con un volcado hexadecimal.

En un servidor, la exportación del Inspector se descarga a tu equipo según se
crea; la exportación de un experimento ofrece [[ui:common.download]], y
[[ui:exp.reportSaved]] de una ejecución en la línea de tiempo es un enlace que
descarga su informe.

## Los secretos no están en estos archivos {#secrets}

Un experimento nombra un secreto — `{{secret.API_TOKEN}}` — y solo se escribe el
nombre. El valor se guarda:

| Dónde se ejecuta Signal Lab | Dónde están los valores de los secretos |
| --- | --- |
| Aplicación de escritorio, Windows | Administrador de credenciales de Windows, bajo `SignalLab` ([[ui:exp.secrets]] en el editor) |
| Aplicación de escritorio, Linux | En ningún sitio: los secretos no se pueden guardar (`secret.unsupported`) |
| Servidor | De solo lectura: la variable de entorno `SIGNALLAB_SECRET_<NAME>`, o el archivo `<NAME>` en `--secrets-dir` (por defecto `/run/secrets/signallab`) |
| `signallab` | Los mismos archivos y variables, o el almacén del sistema con `--secrets system` |

::: warning
Lo que escribes directamente en un campo se guarda tal como lo escribiste. Una
contraseña en las credenciales de una señal HTTP, la contraseña de un emulador
de broker MQTT, un token pegado en un encabezado — todo es texto simple en
`signals.json`, `emulators.json` o `experiment.json`, y en sus exportaciones.
Usa `{{secret.NAME}}` en un experimento para cualquier cosa que no pondrías en
una carpeta compartida.
:::

## Ajustes de la interfaz {#settings}

Lo que recuerda la interfaz — su idioma, los valores escritos por última vez en
cada pantalla, qué panel está abierto y su tamaño, [[ui:http.keepCookies]],
cuándo se buscaron actualizaciones por última vez, y el número aleatorio de la
instalación para las actualizaciones — lo guarda la propia interfaz, no la
carpeta de datos: en el almacenamiento propio de la aplicación en el escritorio,
en el almacenamiento del sitio del navegador para una página de servidor (por
navegador). Las credenciales de la pantalla [[ui:nav.http]] no se guardan ahí.

Signal Lab no escribe archivos de registro; consulta
[resolución de problemas](troubleshooting.md#logs).

## Respaldar, editar, mover {#backup}

- **Respalda** copiando toda la carpeta. Todo lo que hay en ella es JSON
  autocontenido; los valores de los secretos no están ahí, así que vuelve a
  definirlos en una máquina nueva.
- **Edita** `signals.json`, `emulators.json` y `experiment.json` a mano con
  Signal Lab cerrado (o, en un servidor, mientras no haya ninguna página abierta):
  la aplicación escribe el archivo entero desde lo que tiene, así que un cambio
  hecho mientras se ejecuta se sobrescribe con su siguiente guardado. Un error se
  reporta con la línea y la columna la próxima vez que se lee el archivo, nunca
  se reemplaza en silencio — para `signals.json`, también cuando la aplicación
  guarda en él: ese guardado se rechaza y el archivo se queda como lo dejaste.
- **Borra** los archivos `runs/`, `exports/` y `capture-*` en cualquier momento.
  Borrar `signals.json` o `emulators.json` recupera el conjunto inicial; borrar
  `experiment.json` recupera el experimento inicial.
- **Mueve** la carpeta copiándola y apuntando Signal Lab al nuevo sitio:
  `--data-dir` para un servidor, `SIGNALLAB_DATA_DIR` para la aplicación de
  escritorio.
- **Comparte** un experimento exportándolo, o confirmando su JSON junto al
  proyecto que prueba; [`signallab run`](../automation/cli.md#cli-run) lo ejecuta
  desde allí.

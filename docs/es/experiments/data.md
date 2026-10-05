---
title: Datos y plantillas
description: Parámetros y perfiles, el lenguaje de plantillas con sus generadores, los valores extraídos de las respuestas, las comparaciones y los secretos que nunca salen del motor.
---

# Datos en los experimentos

Los valores se mueven por una ejecución: un parámetro elige el destino, un campo de una respuesta se
convierte en un encabezado de la siguiente solicitud, un id generado sale en un comando y vuelve en
una comprobación. Esta página explica de dónde vienen esos valores y cómo los usa un campo.

| Origen | Se escribe como | Se define en |
| --- | --- | --- |
| Parámetro | `{{api}}` o `{{params.api}}` | panel [[ui:exp.params]], un perfil, [[ui:exp.runWith]] |
| Variable | `{{token}}` o `{{vars.token}}` | un nodo durante la ejecución: [[ui:exp.node.extract]], una espera, un envío que espera una respuesta |
| Secreto | `{{secret.API_TOKEN}}` | el almacén de credenciales del equipo, o el entorno y los archivos del servidor |
| Valor integrado | `{{run.seed}}`, `{{now.iso}}`, `{{counter}}` | la propia ejecución |
| Generador | `{{uuid}}`, `{{random_int(1, 100)}}` | sorteados a partir de la semilla de la ejecución |

## Parámetros {#parameters}

Un parámetro es un valor de texto con nombre que cualquier campo con plantilla puede usar. Guarda
los destinos en parámetros, así cambiar una dirección es una sola edición en vez de una por nodo.

### Añadir un parámetro {#add-parameter}

1. Pulsa [[ui:exp.params]] (`{ }`) en la barra de herramientas del editor.
2. En la pestaña [[ui:exp.noProfile]], pulsa [[ui:exp.addParam]].
3. Escribe el [[ui:exp.paramName]] y el [[ui:exp.paramValue]], por ejemplo `api` y
   `http://127.0.0.1:8080`.
4. En el campo de un nodo, escribe `{{api}}/login`.

Cada cambio del panel es una edición del experimento: se guarda con él y se deshace con
<kbd>Ctrl</kbd>+<kbd>Z</kbd> como cualquier otra.

### Reglas {#parameter-rules}

| Regla | Límite |
| --- | --- |
| Nombre | empieza por una letra o `_`, luego letras, dígitos y `_` |
| Nombres reservados | `vars`, `params`, `secret`, `run`, `node`, `now`, `uuid`, `counter`, `random_int`, `random_float`, `pick` |
| Parámetros por experimento | 64 |
| Tamaño de un valor | 64 KiB |
| Nombres | únicos; una variable no puede llevar el nombre de un parámetro |

Un valor es texto plano y se inserta tal como se escribe: los `{{…}}` dentro de un valor no se
resuelven. Cuando un campo pide una parte de un parámetro (`{{config.ports[0]}}`), el valor se lee
como JSON; un valor que no es JSON no tiene partes.

Un parámetro cuyo nombre no es válido, está reservado o aparece dos veces no impide guardar el
experimento, así que puedes seguir escribiendo; el experimento no se ejecuta hasta que se corrija el
nombre.

## Perfiles {#profiles}

Un perfil es un conjunto con nombre de valores de parámetros — *Portátil*, *Escenario*, *Local* —
así cambiar el destino es una elección, no una edición de cada nodo. Un perfil cambia algunos
parámetros; los demás conservan su valor predeterminado.

### Crear un perfil {#make-profile}

1. Abre [[ui:exp.params]] y pulsa [[ui:exp.addProfile]]. Se abre una pestaña nueva.
2. Renómbralo en [[ui:exp.profileName]].
3. Para cada parámetro que cambie el perfil, escribe su valor. Un campo vacío conserva el
   predeterminado, que se muestra en gris en el campo; [[ui:exp.resetToDefault]] (↺) borra un valor.
4. Pulsa [[ui:exp.makeActive]] para ejecutar con él. La pestaña del perfil activo lleva
   ● [[ui:exp.activeProfile]]. [[ui:exp.makeActive]] en la pestaña [[ui:exp.noProfile]] vuelve a
   los predeterminados.

Cuando un experimento tiene perfiles, una lista [[ui:exp.profile]] en la barra de herramientas
permite cambiar entre ellos. El perfil activo lo usan las ejecuciones, la vista previa y
[[ui:exp.sendNow]], y se guarda en el experimento, así que un archivo exportado se abre con los
mismos destinos. [[ui:exp.removeProfile]] elimina el perfil en pantalla.

| Regla | Límite |
| --- | --- |
| Perfiles por experimento | 32 |
| Nombre | 1–64 caracteres, único (los espacios de los extremos no cuentan) |
| Valores | solo parámetros que existan; como máximo 64 |

Renombrar o eliminar un parámetro lo cambia en todos los perfiles a la vez.

### Qué valor usa una ejecución {#precedence}

Gana el último:

1. el valor predeterminado del parámetro, en la pestaña [[ui:exp.noProfile]];
2. el valor del perfil activo, si define uno;
3. un valor escrito en [[ui:exp.runWith]] solo para esta ejecución — consulta
   [ejecutar con otros valores](runs.md#run-with).

[[ui:exp.runWith]] solo puede definir parámetros que tenga el experimento. El informe de la
ejecución registra el perfil, los valores escritos para la ejecución y cada valor que usó.

### Perfiles que no se ejecutarían {#profile-issues}

Cada vez que se comprueba el experimento, también se comprueban los demás perfiles y los valores
predeterminados. Uno que fallaría — por ejemplo, una URL que no es `http://` o `https://` — lleva
⚠ en las pestañas y en la lista de la barra de herramientas, y su descripción emergente dice por
qué. No impide ejecutar con el perfil en uso.

## Plantillas {#templates}

El texto dentro de `{{ }}` es una expresión; todo lo demás de un campo se conserva exactamente como
se escribe.

```text
{{api}}/users/{{user.id}}?trace={{uuid}}
Bearer {{secret.API_TOKEN}}
```

- Los espacios dentro de las llaves no importan: `{{ token }}` es `{{token}}`.
- `\{{` escribe un `{{` literal.
- Un `}}` por sí solo es texto plano.
- Un valor se inserta tal cual, sin comillas. En un cuerpo JSON, escribe las comillas tú:
  `"id": "{{uuid}}"`.

### Nombres {#names}

| Expresión | Valor |
| --- | --- |
| `{{name}}` | la variable `name` si hay una definida en este camino; si no, el parámetro `name` |
| `{{vars.name}}` | solo la variable |
| `{{params.name}}` | solo el parámetro |
| `{{secret.NAME}}` | el secreto guardado `NAME` — consulta [secretos](#secrets) |
| `{{name.field}}` | un campo de un valor JSON |
| `{{name[0]}}` | un elemento de un array JSON |
| `{{name["a b"]}}`, `{{name['a b']}}` | un campo cuyo nombre tiene otros caracteres |

Un nombre de campo después de `.` puede contener letras, dígitos, `_` y `-`. Los pasos se encadenan:
`{{reply.args[0]}}`, `{{order.items[2].sku}}`.

### Cómo se escriben los valores {#value-text}

| Valor | Se escribe como |
| --- | --- |
| texto | el texto |
| número | su forma más corta: `42`, `0.5` |
| `true`, `false` | `true`, `false` |
| `null` | `null` |
| objeto, array | JSON compacto: `["x","y"]` |

### Valores integrados {#built-ins}

| Expresión | Valor |
| --- | --- |
| `{{run.id}}` | el número de tarea de la ejecución; `0` en la vista previa y en [[ui:exp.sendNow]] |
| `{{run.seed}}` | la semilla de esta ejecución |
| `{{node.id}}` | el id del nodo que se está ejecutando |
| `{{now}}` | la hora actual, en milisegundos Unix |
| `{{now.iso}}` | la hora actual en UTC, ISO 8601 con milisegundos: `2026-09-30T12:34:56.789Z` |
| `{{counter}}` | cuántas veces se ha ejecutado este nodo en esta ejecución, esta incluida, desde 1 |

`{{counter}}` cuenta por nodo: en el cuerpo de un [Bucle](flow.md#loop) es el número de la
iteración; en un nodo que [repite](flow.md#repeat), el número del envío. `run`, `node` y `now` solo
tienen los campos enumerados; cualquier otro es un error.

### Generadores {#generators}

| Expresión | Valor |
| --- | --- |
| `{{uuid}}` o `{{uuid()}}` | un UUID versión 4 |
| `{{random_int(min, max)}}` | un número entero de `min` a `max`, ambos incluidos; argumentos enteros, `min` ≤ `max` |
| `{{random_float(min, max)}}` | un número de `min` hasta `max` sin incluirlo, con 3 decimales; `min` < `max` |
| `{{random_float(min, max, digits)}}` | lo mismo con `digits` decimales, 0–9 |
| `{{pick(a, b, c)}}` | uno de los argumentos, al menos uno |

Los argumentos se separan por comas. Uno entre comillas (`"dark blue"` o `'a, b'`) puede contener
cualquier cosa salvo su propia comilla; uno sin comillas puede contener letras, dígitos y
`_ - . : / +`. Un argumento vacío es un error.

Cada generador sortea a partir de la semilla de la ejecución. Los valores de una ejecución de un
nodo dependen solo de la semilla, del id del nodo y de cuántas veces se ha ejecutado el nodo, así
que las ramas paralelas nunca cambian los valores de las demás, y una ejecución con la misma semilla
vuelve a generar los mismos valores. Los sorteos dentro de un nodo siguen el orden de sus campos.
`{{now}}` y `{{run.id}}` no son reproducibles. Consulta [semillas](runs.md#seeds).

### Sugerencias {#suggestions}

Escribir `{{` en un campo con plantilla, o pulsar <kbd>Ctrl</kbd>+<kbd>Space</kbd>, abre una lista
en cuatro grupos: [[ui:exp.suggest.params]] con sus valores, [[ui:exp.suggest.vars]] definidas antes
de este nodo con el nodo que las define (también los campos de una respuesta, como
`reply.args[0]`), [[ui:exp.suggest.secrets]] y [[ui:exp.suggest.generators]]. <kbd>↑</kbd> y
<kbd>↓</kbd> eligen, <kbd>Enter</kbd> o <kbd>Tab</kbd> insertan, <kbd>Esc</kbd> cierra la lista y
conserva el campo.

### Los nombres desconocidos son errores {#unknown-names}

Un nombre sin valor nunca se convierte en una cadena vacía. Antes de una ejecución, cada nombre que
usa un campo debe ser un parámetro, un nombre de secreto válido o una variable definida en
**todos** los caminos que llevan al nodo. El editor señala el nodo y el campo:

| Problema | Antes de la ejecución | Durante la ejecución |
| --- | --- | --- |
| Un nombre que nadie define | `name.unknown` | — |
| Una variable definida solo en algunos caminos | `name.not_on_every_path` | — |
| `{{params.x}}` sin un parámetro `x` | `param.unknown` | — |
| Un campo que un valor no tiene | — | `template.no_field` |
| Un `{{` sin cerrar, un `{{}}` vacío, un argumento mal formado | `template.*`, con la posición | — |

Los textos de estos códigos están en [Errores](../reference/errors.md).

## Qué campos aceptan plantillas {#templated-fields}

| Nodo | Campos con plantilla |
| --- | --- |
| [[ui:exp.node.http]] | URL, nombres y valores de encabezados, cuerpo, el nombre de usuario y la contraseña de Basic y Digest, el token Bearer |
| [[ui:exp.node.osc]] | destino, dirección, argumentos de texto; con respuesta: su patrón de dirección y los valores de sus reglas |
| [[ui:exp.node.udp]] | destino, carga útil; con respuesta: su patrón |
| [[ui:exp.node.tcp]] | host, carga útil |
| [[ui:exp.node.mqtt]] | host del bróker, tema, carga útil |
| [[ui:exp.node.log]] | mensaje |
| [[ui:exp.node.assert_body]] | texto esperado |
| [[ui:exp.node.assert_header]] | nombre del encabezado, texto esperado |
| [[ui:exp.node.assert_value]], [[ui:exp.node.branch_value]], la condición de salida de un [[ui:exp.node.loop]] | valor, valor esperado |
| [[ui:exp.node.wait_osc]] | patrón de dirección, valores de las reglas |
| [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_ws]] | patrón |
| [[ui:exp.node.wait_mqtt]] | bróker y tema (solo parámetros), patrón |
| [[ui:exp.node.wait_http]] | patrón de ruta de acceso, condiciones |
| [[ui:exp.node.impairment]] | escucha y destino (solo parámetros) |
| [[ui:exp.node.ws_connect]] | URL, nombres y valores de encabezados |
| [[ui:exp.node.ws_send]] | carga útil |
| [[ui:exp.node.ws_close]] | motivo |

Los números — puertos, tiempos de espera, retardos, estados, números OSC tipados — y las direcciones
de escucha de las esperas son literales. Un [[ui:exp.node.emulator]] rellena sus propias respuestas
con lo que llegó (`{{request.…}}`) y los parámetros; consulta
[fallos](faults.md#emulator).

**Solo parámetros.** Algunos campos se abren antes del primer paso, cuando todavía no existe ninguna
variable: el bróker y el tema de un [[ui:exp.node.wait_mqtt]], la escucha y el destino de un
[[ui:exp.node.impairment]]. Solo aceptan texto y parámetros, nada más (`node.params_only`).

**Comprobados como literales.** Un campo que usa solo parámetros se resuelve antes de la ejecución y
se comprueba como el texto que enviará la ejecución: una URL debe ser `http://` o `https://`, un
destino OSC `IP:port` o `host:port`, un nombre de encabezado válido. Un campo con variables o
generadores se comprueba cuando se ejecuta.

## Vista previa {#preview}

Cuando el nodo seleccionado tiene una plantilla, sus propiedades muestran qué hará con los valores
conocidos ahora: [[ui:exp.preview]] para un envío, [[ui:exp.previewWait]] para una espera,
[[ui:exp.previewCheck]] para una comparación. La resuelve el motor, con el mismo código que usa una
ejecución, así que la vista previa nunca discrepa de la ejecución.

- Los parámetros vienen del perfil activo.
- Las variables vienen de lo que el editor ha visto en esta sesión: los pasos de la última ejecución
  y [[ui:exp.sendNow]].
- Un secreto guardado se muestra como `••••`.
- Un nombre que aún no tiene valor se queda como se escribió, y la vista previa lo enumera. Un
  secreto que no está guardado se enumera aparte.
- Los generadores usan la semilla fijada del experimento, o `0` cuando no hay ninguna, como la
  primera ejecución del nodo. Con una semilla fijada, la vista previa muestra los valores generados
  que enviará la primera ejecución del nodo en una ejecución.

## Extraer valores {#extract}

[[ui:exp.node.extract]] lee un valor de la última respuesta HTTP en su camino y lo escribe en una
variable.

| Campo | Qué |
| --- | --- |
| [[ui:exp.variable]] | la variable donde escribir; se aplican las reglas de nombres de los parámetros |
| [[ui:exp.extractFrom]] | de dónde viene el valor (más abajo) |
| [[ui:exp.jsonPath]], [[ui:exp.headerName]] o [[ui:exp.pattern]] | qué leer, según el origen |

| [[ui:exp.extractFrom]] | Lee | Valor |
| --- | --- | --- |
| [[ui:exp.from.json]] | el cuerpo como JSON, en una ruta | el valor JSON: texto, número, objeto, array |
| [[ui:exp.from.header]] | el primer encabezado con ese nombre, en cualquier caso | texto |
| [[ui:exp.from.status]] | el código de estado | un número |
| [[ui:exp.from.body]] | todo el cuerpo | texto |
| [[ui:exp.from.regex]] | la primera coincidencia en el cuerpo | el grupo de captura 1 si el patrón tiene uno; si no, toda la coincidencia |

**Rutas JSON.** `$.token`, `$.items[0].id`, `$["a b"]`, `$['a b']['c-d']`; se puede omitir el `$.`
inicial (`token`, `items[0].id`), y `$` solo es todo el cuerpo.

**Las expresiones regulares** usan la sintaxis del motor `regex` de Rust, que no tiene aserciones de
búsqueda ni referencias hacia atrás. La coincidencia se busca en cualquier parte del cuerpo; ancla
con `^` y `$` cuando importe.

El paso falla, indicando qué falta, cuando:

- ninguna solicitud HTTP se ejecutó antes en este camino (`check.no_response`; el editor ya rechaza
  un grafo donde ninguna puede, `graph.needs_http`);
- el cuerpo no es JSON, o la ruta no está en él;
- el encabezado no está, o el patrón no coincide;
- el cuerpo supera los 256 KiB que guarda una respuesta, para una ruta JSON o el cuerpo entero, y
  para un patrón que no coincidió en la parte guardada (`extract.truncated`).

La línea de tiempo muestra el valor escrito: `token = abc123`.

::: tip Extraer con un clic
[[ui:exp.sendNow]] en un [[ui:exp.node.http]] muestra su respuesta JSON. Haz clic en un valor de
ella: se añade un nodo [[ui:exp.node.extract]] después de la solicitud, con la ruta rellenada y un
nombre tomado de la clave, y la vista previa conoce el valor de inmediato.
:::

## Variables {#variables}

Una variable contiene un valor JSON. Estos nodos escriben una:

| Nodo | Escribe | En la salida |
| --- | --- | --- |
| [[ui:exp.node.extract]] | el valor extraído | su salida |
| [[ui:exp.node.wait_osc]], [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_http]], [[ui:exp.node.wait_ws]] | lo que llegó, nombre predeterminado `reply` (`request` para HTTP) | solo [[ui:exp.portMatched]] |
| [[ui:exp.node.osc]], [[ui:exp.node.udp]] con [[ui:exp.expectReply]] | la respuesta, nombre predeterminado `reply` | su salida |

Lo que escribe una espera es un objeto; los campos posteriores leen sus partes:

| Espera | Campos |
| --- | --- |
| OSC | `address`, `args`, `from`, `ms` |
| UDP | `text`, `hex`, `bytes`, `from`, `ms`, y `match` con un patrón |
| MQTT | `topic`, y los campos de UDP |
| WebSocket | los campos de UDP, y `json` cuando el mensaje es JSON |
| Solicitud HTTP | `method`, `path`, `query`, `headers`, `body`, `json`, `params`, `from`, `ms` |

`ms` es el tiempo desde la última acción de la rama hasta la llegada. El contenido exacto está en
[la referencia de los nodos](nodes.md).

### Dónde se conoce una variable {#visibility}

Una variable existe desde la salida que la escribe en adelante, en los caminos que pasan por esa
salida:

- Tras una unión de caminos alternativos — el [[ui:exp.yes]] y el [[ui:exp.no]] de una bifurcación
  que se reencuentran — solo se conoce lo que definió **todos** los caminos.
- Tras [[ui:exp.node.join]], se conoce lo que definió **cualquier** rama que llegue a él: todas se
  ejecutaron.
- Tras el [[ui:exp.portDone]] o el [[ui:exp.portLimit]] de un [[ui:exp.node.loop]], y en su condición
  de salida, se conoce lo que define cada iteración del cuerpo.
- La variable de una espera no se conoce tras su salida [[ui:exp.portTimeout]].

Cada rama paralela trabaja sobre su propia copia de las variables. Una unión fusiona las copias en
el orden de sus cables de entrada; el cable posterior gana un nombre que ambos definan, así que el
resultado nunca depende de qué rama terminó primero. Consulta
[cómo se mueve una ejecución](flow.md#parallel).

## Comparar valores {#compare}

[[ui:exp.node.assert_value]] hace fallar la ejecución cuando una comparación no se cumple;
[[ui:exp.node.branch_value]] sale por [[ui:exp.yes]] o [[ui:exp.no]]; un [[ui:exp.node.loop]] usa la
misma comparación como su condición de salida. Cada uno tiene un [[ui:exp.value]], un
[[ui:exp.operator]] y un valor [[ui:exp.expected]], y ambos textos son plantillas:

| [[ui:exp.value]] | [[ui:exp.operator]] | [[ui:exp.expected]] |
| --- | --- | --- |
| `{{status}}` | [[ui:exp.op.lt]] | `300` |
| `{{reply.args[0]}}` | [[ui:exp.op.eq]] | `{{nonce}}` |

| [[ui:exp.operator]] | Se cumple cuando |
| --- | --- |
| [[ui:exp.op.eq]], [[ui:exp.op.ne]] | los dos son iguales (distintos) — como números cuando ambos son números (`200` es igual a `200.0`); si no, como texto exacto, mayúsculas incluidas |
| [[ui:exp.op.lt]], [[ui:exp.op.le]], [[ui:exp.op.gt]], [[ui:exp.op.ge]] | como números; un lado que no es un número hace fallar el paso (`compare.not_numbers`) en lugar de un *no* silencioso |
| [[ui:exp.op.contains]] | el valor contiene el texto esperado, mayúsculas incluidas |
| [[ui:exp.op.matches]] | la expresión regular del valor esperado coincide en cualquier parte del valor |
| [[ui:exp.op.empty]], [[ui:exp.op.not_empty]] | el valor está vacío, o no, tras recortar espacios; el valor esperado no se usa |

Un número es texto que se lee como tal tras recortar espacios: `42`, `-1.5`, `1e3`. La línea de
tiempo muestra la comparación tal como se hizo, `401 = 200`, cada lado recortado a 120 caracteres.

## Secretos {#secrets}

Un token o una contraseña se escriben en un campo como `{{secret.NAME}}`. El archivo del experimento
solo guarda el nombre; el valor se queda donde está almacenado y nunca llega a la interfaz.

### Dónde viven los secretos {#secret-stores}

| Dónde se ejecuta Signal Lab | Almacén | Desde la interfaz |
| --- | --- | --- |
| Aplicación de escritorio en Windows | el Administrador de credenciales de Windows, bajo el servicio `SignalLab`, una entrada por nombre | definir, reemplazar, eliminar |
| Aplicación de escritorio en Linux | ninguno: una ejecución que necesita un secreto falla con `secret.unsupported` | — |
| Servidor | la variable de entorno `SIGNALLAB_SECRET_<NAME>`; si no, el archivo `<NAME>` en su carpeta de secretos, `/run/secrets/signallab` salvo que se indique otra cosa | solo lectura |
| `signallab` en la línea de comandos | como un servidor, o el Administrador de credenciales de Windows con `--secrets system` | — |

El encabezado de la sección [[ui:exp.secrets]] tiene una descripción emergente que dice cuál de
estos se aplica donde estás: el almacén de Windows, el entorno y los archivos del servidor, o — en
la aplicación de escritorio en Linux — que no hay ningún almacén. Allí, `secret.unsupported` dice
que los secretos se guardan en el Administrador de credenciales de Windows, que este sistema no
tiene.

Un secreto pertenece al equipo o al servidor, no a un experimento: dos experimentos que usan
`{{secret.API_TOKEN}}` usan el mismo valor.

En un servidor, la variable de entorno gana al archivo. El salto de línea final de un archivo no
forma parte del valor, y un archivo vacío cuenta como que no hay secreto. La carpeta del servidor se
define con `--secrets-dir` o `SIGNALLAB_SECRETS_DIR`; consulta
[el servidor](../server/index.md). Para la línea de comandos, consulta
[`signallab run`](../automation/cli.md#cli-run).

| Regla | Límite |
| --- | --- |
| Nombre | empieza por una letra o `_`, luego letras, dígitos y `_`; como máximo 128 caracteres |
| Valor | no vacío, como máximo 16 KiB |

### Definir un secreto {#set-secret}

En Windows:

1. Abre [[ui:exp.params]]. La sección [[ui:exp.secrets]] enumera cada secreto que usan los campos
   del experimento, cada uno [[ui:exp.secretStored]] o [[ui:exp.secretMissing]].
2. Pulsa [[ui:exp.secretSet]] junto al nombre, o [[ui:exp.addSecret]] para un nombre que aún no usa
   ningún campo.
3. Escribe el valor — el campo muestra puntos — y pulsa [[ui:exp.secretSave]] o <kbd>Enter</kbd>. El
   campo se vacía; nada puede volver a leer el valor.

[[ui:exp.secretReplace]] guarda un valor nuevo y [[ui:exp.secretRemove]] lo elimina del almacén de
credenciales. Un nombre guardado en esta sesión también se ofrece en las sugerencias.

En un navegador conectado a un servidor, la sección solo dice [[ui:exp.secretOnServer]] o
[[ui:exp.secretNotOnServer]]: define el valor donde se ejecuta el servidor, de una de dos formas:

```bash
# in the server's environment
SIGNALLAB_SECRET_API_TOKEN='…'
# or as a file in its secrets folder
printf '%s' '…' > /run/secrets/signallab/API_TOKEN
```

Un archivo se lee cada vez que empieza una ejecución, así que un archivo cambiado cuenta desde la
siguiente ejecución; una variable de entorno cambiada necesita que se reinicie el servidor.

### Antes de una ejecución {#secret-check}

Cada secreto que usan los campos de la ejecución debe estar guardado. Si falta uno, la ejecución se
detiene antes de cualquier tráfico, en el primer nodo y campo que lo usan (`secret.missing`).
[[ui:exp.sendNow]] comprueba lo mismo para su nodo.

### Enmascaramiento {#masking}

Mientras una ejecución o un [[ui:exp.sendNow]] usan secretos, cada aparición de sus valores se
reemplaza por `••••` en todo lo que sale del motor:

- textos de los pasos, errores y las variables que escribió un paso;
- el informe de la ejecución;
- el resultado de [[ui:exp.sendNow]], incluida la respuesta HTTP que muestra;
- las tramas del [[ui:dock.inspector]], capturadas mientras dura la ejecución — en un volcado hex
  cada byte de un valor se convierte en `*`, así que los desplazamientos siguen siendo correctos.

La autenticación Basic envía `name:password` en base64; cuando una de las dos partes contiene un
secreto, ese texto base64 también se enmascara. El tráfico en sí lleva el valor real. La vista previa
muestra un secreto guardado como `••••`. Las respuestas de un [[ui:exp.node.emulator]] no pueden usar
secretos.

## Comandos {#commands}

La vista previa es [`experiment_resolve`](../api/commands.md#experiment_resolve); los secretos se
enumeran, se definen y se eliminan con [`secret_status`](../api/commands.md#secret_status),
[`secret_set`](../api/commands.md#secret_set) y
[`secret_delete`](../api/commands.md#secret_delete). Ningún comando devuelve el valor de un secreto.

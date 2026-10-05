---
title: HTTP
description: Envía una solicitud HTTP y lee toda la respuesta, autentícate con Basic, Bearer o Digest, conserva las cookies y mide un endpoint bajo una ráfaga de carga concurrente.
---

# HTTP

La pantalla [[ui:nav.http]] es a la vez un inspector de solicitudes y una herramienta de carga:

- enviar una sola solicitud y ver el estado, el tiempo que tardó, los encabezados y
  el cuerpo;
- autenticarte con Basic, un token Bearer o Digest;
- conservar las cookies que define un servidor, como hace un navegador;
- enviar la misma solicitud muchas veces a la vez — una [[ui:http.burst]] — y leer
  el caudal y los percentiles de latencia.

## Enviar una solicitud {#request}

1. Abre [[ui:nav.http]].
2. Elige el método y escribe la URL, por ejemplo `http://127.0.0.1:8080/health`.
3. Añade [[ui:http.headers]] si el servidor los necesita; [[ui:http.addHeader]] añade
   una fila, ✕ quita una. Una fila sin nombre no se envía.
4. Para un método distinto de GET y HEAD, escribe el [[ui:http.body]]. El texto se queda en el campo mientras cambias a GET o HEAD y vuelve con el otro método, pero no se envía mientras tanto.
5. Pulsa [[ui:common.send]].

La línea bajo los botones da el veredicto al instante — estado, tiempo y tamaño,
o por qué no hubo respuesta — y el panel [[ui:http.response]] muestra el
resto. La solicitud (método, URL, encabezados, cuerpo, tiempo de espera y
[[ui:http.keepCookies]]) se conserva cuando cambias de pantalla y cuando reinicias
la aplicación; las credenciales no.

| Tecla | Dónde | Qué hace |
| --- | --- | --- |
| <kbd>Enter</kbd> | la URL, un encabezado, una credencial, el tiempo de espera | envía |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | cualquier campo de la solicitud, el cuerpo incluido | envía |
| <kbd>Ctrl</kbd>+<kbd>S</kbd> | cualquier campo de la solicitud | la guarda como señal ([abajo](#library)) |

### Campos de la solicitud {#request-fields}

| Campo | Qué | Predeterminado |
| --- | --- | --- |
| [[ui:exp.method]] | GET, POST, PUT, PATCH, DELETE, HEAD u OPTIONS | GET |
| [[ui:field.url]] | Una URL `http://` o `https://` | `http://127.0.0.1:8080/` |
| [[ui:http.headers]] | Pares de nombre y valor, enviados tal como se escriben. Sin un `User-Agent` propio, Signal Lab envía `SignalLab/0.1`. | `Accept: application/json` |
| [[ui:field.auth]] | Cómo se autentica la solicitud ([abajo](#auth)) | [[ui:http.auth.none]] |
| [[ui:http.keepCookies]] | Devolver las cookies que definen los servidores ([abajo](#cookies)) | activado |
| [[ui:http.body]] | Se envía exactamente como se escribe; no se añade ningún `Content-Type`, así que añade el encabezado que corresponda. Un cuerpo vacío no se envía. El campo no se muestra para GET y HEAD, y entonces la solicitud no lleva ningún cuerpo — ni en el envío, ni en una señal guardada, ni en [[ui:common.toExperiment]] — aunque hayas escrito uno con otro método. | vacío |
| [[ui:common.timeoutMs]] | Cuánto puede durar todo el intercambio, respuesta y cuerpo incluidos | 10 000 |

## Autenticación {#auth}

| [[ui:field.auth]] | Campos | Qué se envía |
| --- | --- | --- |
| [[ui:http.auth.none]] | — | ningún encabezado `Authorization` |
| [[ui:http.auth.basic]] | [[ui:field.username]], [[ui:field.password]] | `Authorization: Basic …`, el nombre y la contraseña en base64, con la primera solicitud |
| [[ui:http.auth.bearer]] | [[ui:field.token]] | `Authorization: Bearer <token>` |
| [[ui:http.auth.digest]] | [[ui:field.username]], [[ui:field.password]] | nada al principio; la respuesta al desafío del servidor ([abajo](#digest)) |

Cambiar entre Basic y Digest conserva el nombre y la contraseña.

### Digest {#digest}

Con Digest, Signal Lab envía la solicitud sin credenciales. Cuando el servidor
responde `401` con un desafío Digest, Signal Lab calcula la respuesta a partir del
desafío y tu contraseña y vuelve a enviar la solicitud. La respuesta que ves
es la de esa segunda solicitud, marcada [[ui:http.digestAnswered]], y la
latencia cuenta ambos intercambios — lo que espera un cliente.

- Algoritmos: MD5 y SHA-256, y sus variantes `-sess`. Cuando un servidor ofrece
  ambos, se usa SHA-256.
- Calidad de protección: `auth` y `auth-int`, y la respuesta más antigua sin
  `qop`.
- Cuando el servidor dice que el nonce se agotó (`stale`), o vuelve a pedirlo con uno nuevo,
  la solicitud se responde otra vez, hasta 3 veces más. Cuando rechaza la
  respuesta al nonce que dio en último lugar, el `401` se mantiene: el nombre o la contraseña
  son incorrectos.
- Los redireccionamientos los sigue Signal Lab mismo, así que la URL que pregunta es la que
  se responde. Un desafío de otro origen no se responde: las credenciales escritas
  para un host no van a ningún otro. Pasar de `http://` a `https://` en el mismo
  host y los puertos predeterminados cuenta como el mismo host.

Cuando el desafío no se puede responder, el `401` se mantiene y el panel dice por qué:

| Message | Meaning |
| --- | --- |
| The server answered 401 without asking for Digest | El servidor quiere otro esquema; prueba Basic o Bearer. |
| The server asked for Digest with … | Un algoritmo que Signal Lab no habla; habla MD5 y SHA-256. |
| The server asked for Digest without a realm or nonce | El desafío del servidor está incompleto. |
| The request was sent on to …, which asked for Digest | Un redireccionamiento llevó a otro origen, cuyo desafío no se responde. |

### Dónde van las credenciales {#credentials}

Las credenciales van solo al encabezado `Authorization` de la solicitud cuando se envía.
El Inspector, la consola y los informes de experimento nunca muestran ese encabezado. En
esta pantalla se guardan solo en memoria y desaparecen tras un reinicio — salvo que
la solicitud esté vinculada a una señal guardada, que las trae de vuelta.

::: warning
Una solicitud guardada como señal conserva sus credenciales en el archivo de la biblioteca,
`signals.json`, como texto sin formato. En un experimento, escribe una contraseña como
`{{secret.NAME}}` en su lugar; consulta [Datos y plantillas](../experiments/data.md).
:::

## Cookies {#cookies}

Con [[ui:http.keepCookies]] activado, lo que un servidor define con `Set-Cookie` se conserva
en el almacén de cookies de la pantalla y se devuelve con las solicitudes posteriores a ese servidor,
siguiendo las reglas del navegador (dominio, ruta de acceso, `Secure`, caducidad). El almacén lo usan
las solicitudes de esta pantalla, su ráfaga y las señales HTTP que envías desde la
biblioteca. Desactívalo para enviar solicitudes sin cookies y no conservar ninguna.

El panel de cookies bajo la solicitud y la respuesta lista lo que contiene el almacén:
[[ui:http.cookieName]], [[ui:http.cookieValue]], [[ui:http.cookieWhere]]
(un dominio que empieza por `.` también cubre sus subdominios),
[[ui:http.cookieExpires]] ([[ui:http.cookieSession]] para una cookie sin
caducidad) y [[ui:http.cookieFlags]] (`Secure`, `HttpOnly`, `SameSite`).
Las cookies caducadas no se listan. [[ui:common.clear]] vacía el almacén.

El almacén vive mientras vive la aplicación: un reinicio empieza con uno vacío. En un
[servidor](../server/index.md), hay un almacén por cada página que haya iniciado sesión en él.
Una ejecución de experimento tiene un almacén propio (consulta [Experimentos](../experiments/index.md)),
y `signallab send http` no usa ninguno.

## La respuesta {#response}

| Parte | Qué |
| --- | --- |
| [[ui:http.status]] | El código de estado y su motivo; `ERR` cuando no llegó ninguna respuesta |
| [[ui:http.latency]] | Desde el envío hasta el último byte del cuerpo, en milisegundos |
| [[ui:http.size]] | El tamaño del cuerpo |
| Encabezados de respuesta | Haz clic en la línea con su recuento para mostrarlos u ocultarlos |
| Cuerpo | Formateado cuando es JSON; [[ui:http.rawBody]] y [[ui:http.formatJson]] alternan. Se muestran hasta 256 KiB, luego `… (truncated)`. |

Cuando no hay respuesta, el panel dice por qué, con las mismas palabras que en todas partes
de Signal Lab: rechazada, sin respuesta a tiempo, el nombre no resuelve, un
problema de certificado, etc. El detalle técnico del sistema queda plegado
debajo.

### Redireccionamientos {#redirects}

Los redireccionamientos (301, 302, 303, 307, 308) se siguen, hasta 10; la respuesta que se muestra
es la última. Tras 301, 302 y 303 la solicitud continúa como GET sin
cuerpo (HEAD sigue siendo HEAD); tras 307 y 308 como estaba. El `Authorization` y las
cookies escritas para un host no se envían a otro.

### Conexiones seguras {#tls}

El certificado de un servidor `https://` se comprueba contra los certificados en los que
este sistema confía. Un certificado autofirmado o caducado se rechaza con
"A secure connection to … could not be made"; no hay ningún ajuste para saltarse la
comprobación. Para probar un servidor con tu propio certificado, añádelo a los certificados
de confianza del sistema.

## Ráfaga de carga {#burst}

La [[ui:http.burst]] envía la solicitud en pantalla — con su autenticación y,
mientras [[ui:http.keepCookies]] está activado, el almacén de cookies — muchas veces, y la
mide.

1. Define [[ui:common.concurrency]], [[ui:http.total]], [[ui:http.duration]] y
   [[ui:http.rate]].
2. Pulsa [[ui:http.startBurst]]. La ráfaga es una tarea: [[ui:http.stopBurst]], o
   detenerla en la franja de la consola, la termina.

| Campo | Qué | Predeterminado |
| --- | --- | --- |
| [[ui:common.concurrency]] | Solicitudes en curso a la vez, 1–512 | 20 |
| [[ui:http.total]] | Solicitudes que enviar; 0 — seguir enviando hasta que termine la duración | 500 |
| [[ui:http.duration]] | Segundos que dura; 0 — parar cuando se envíe el total | 0 |
| [[ui:http.rate]] | Solicitudes iniciadas por segundo, 0,1–100 000; 0 — tan rápido como vayan los hilos | 0 |

Con [[ui:http.total]] y [[ui:http.duration]] a 0, la ráfaga funciona hasta que
la detengas.

Hay dos formas de enviar:

- **[[ui:http.rate]] 0.** Cada hilo vuelve a enviar en cuanto tiene una
  respuesta. Esto averigua cuánto aguanta el servidor, pero un servidor lento también frena
  la ráfaga.
- **Una tasa.** Las solicitudes empiezan según un plan fijo — a 10 por segundo, una
  cada 100 ms desde el inicio — por lentas que sean las respuestas. Una solicitud cuyo
  momento llega mientras todos los hilos están ocupados espera como máximo 50 ms por uno; después
  de eso se omite y se cuenta como [[ui:http.missed]], nunca se envía tarde. Las solicitudes
  omitidas significan que la concurrencia es demasiado baja para esta tasa, o que el servidor es
  más lento de lo que la tasa necesita.

| Número | Qué |
| --- | --- |
| [[ui:http.sent]] | Solicitudes que han tenido respuesta o han fallado |
| [[ui:http.ok]] | Respondidas con un estado 2xx |
| [[ui:http.failed]] | Sin respuesta, o cualquier estado fuera de 200–299 |
| [[ui:http.missed]] | Omitidas, como arriba (solo con una tasa) |
| [[ui:http.rps]] | Solicitudes por segundo durante la última décima de segundo; cuando la ráfaga ha terminado, durante toda la ráfaga. Con una tasa, la etiqueta nombra la tasa pedida. |
| [[ui:http.p50]], [[ui:http.p90]], [[ui:http.p95]], [[ui:http.p99]] | El tiempo dentro del cual terminó esa proporción de las solicitudes, fallos incluidos; preciso hasta un 0,5 % |
| [[ui:http.avg]], [[ui:http.min]], [[ui:http.max]] | La media, la más rápida y la más lenta |

Los números se actualizan unas 10 veces por segundo. El gráfico que hay junto a ellos dibuja
las solicitudes por segundo durante los últimos 24 segundos, aproximadamente.

Con Digest, el desafío de la primera solicitud se responde una vez y esa respuesta
sirve para todas las solicitudes de la ráfaga.

::: warning
Una ráfaga es carga real. Apúntala solo a servidores que sean tuyos o que puedas probar.
:::

Para rampas, escalones, picos y umbrales de superado/fallado, ejecuta la solicitud bajo
[carga en un experimento](../experiments/load.md).

## En el Inspector {#inspector}

Con la captura activada, cada intercambio aparece como una trama con el protocolo
`http` y el origen `http`: el método, la URL, el estado y el tiempo en el resumen,
los encabezados de la respuesta y el inicio del cuerpo (2000 caracteres) en su detalle,
el estado como veredicto (`failed` cuando no llegó ninguna respuesta, `· digest after 401`
cuando se respondió un desafío). La trama registra el tamaño del cuerpo, no sus
bytes. El encabezado `Authorization` de la solicitud nunca está en ella. Una ráfaga pone como
mucho un intercambio cada 100 ms en la captura. Consulta
[Inspector](../tools/inspector.md).

## Guardar y reutilizar {#library}

- **Guardar como señal.** [[ui:sig.saveNew]] conserva la solicitud — método, URL,
  encabezados, cuerpo, tiempo de espera y autenticación — en la biblioteca de señales. La pantalla
  queda vinculada a ella: [[ui:sig.save]] (<kbd>Ctrl</kbd>+<kbd>S</kbd>) la actualiza,
  [[ui:sig.saveAs]] la copia, la etiqueta la abre en [[ui:nav.signals]]. Abrir
  una señal HTTP desde la biblioteca la carga aquí de nuevo, credenciales incluidas.
  Consulta [Señales](../tools/signals.md).
- **Añadir a un experimento.** [[ui:common.toExperiment]] añade un
  paso [Solicitud HTTP](../experiments/nodes.md#node-http) con la misma solicitud
  al experimento abierto, justo antes de Fin o después del paso seleccionado, y lo abre.
- **Simular esto.** Bajo una respuesta, [[ui:http.mockThis]] crea una ruta de emulador
  que responde a este método y ruta de acceso con este estado, encabezados y cuerpo. Elige
  un emulador HTTP en [[ui:http.mockInto]], o [[ui:http.mockNew]], y pulsa
  [[ui:http.mockAdd]]; la ruta va la primera en ese emulador, y la
  pantalla [[ui:nav.emulators]] se abre en él. Consulta [Emuladores](../tools/emulators.md).

## En experimentos {#experiments}

| Paso | Qué hace |
| --- | --- |
| [[ui:exp.node.http]] | Envía una solicitud; su URL, encabezados, cuerpo y credenciales aceptan `{{templates}}`. Puede ejecutarse [bajo carga](../experiments/load.md). [Detalles](../experiments/nodes.md#node-http) |
| [[ui:exp.node.assert_status]], [[ui:exp.node.assert_body]], [[ui:exp.node.assert_header]], [[ui:exp.node.assert_latency]] | Comprueban la última respuesta. [Detalles](../experiments/nodes.md#node-assert_status) |
| [[ui:exp.node.extract]] | Guarda un campo JSON, un encabezado, el estado, el cuerpo o la coincidencia de una expresión regular como variable. [Detalles](../experiments/nodes.md#node-extract) |
| [[ui:exp.node.branch_status]] | Continúa por Sí o No según el estado. [Detalles](../experiments/nodes.md#node-branch_status) |
| [[ui:exp.node.wait_http]] | Espera a que llegue una solicitud — del sistema que pruebas — al propio escucha o emulador de la ejecución. [Detalles](../experiments/nodes.md#node-wait_http) |
| [[ui:exp.node.emulator]] | Una API HTTP que responde por rutas durante toda la ejecución. [Detalles](../experiments/nodes.md#node-emulator) |

## Desde la línea de comandos {#cli}

`signallab send http` envía una solicitud, como esta pantalla:

```bash
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab send http POST http://127.0.0.1:8080/api/items \
  -H 'Content-Type: application/json' --body '{"name":"lamp"}'
signallab send http GET http://127.0.0.1:8080/private -u admin:secret --digest
```

La línea de estado va a la salida de error estándar y el cuerpo a la salida estándar:

```text
HTTP 200 OK · 3 ms · 15 B
{"status":"ok"}
```

| Opción | Qué | Predeterminado |
| --- | --- | --- |
| `-H`, `--header 'Name: value'` | Un encabezado; repítelo para más | — |
| `--body TEXT`, `--body @FILE` | El cuerpo, o el contenido de un archivo | — |
| `--expect-status N` | Sale con 1 salvo que el estado sea N | — |
| `--timeout MS` | Cuánto esperar la respuesta | 10 000 |
| `-u`, `--user NAME:PASSWORD` | Autenticación Basic | — |
| `--digest` | Con `--user`: responder en su lugar al desafío Digest del servidor | — |
| `--bearer TOKEN` | `Authorization: Bearer TOKEN` | — |
| `--json` | Imprimir toda la respuesta como JSON en la salida estándar | — |

Sale con 0 cuando llegó una respuesta (y tenía el estado esperado), 1 cuando
no llegó ninguna, el estado no era el esperado o un desafío Digest no se pudo
responder, y 2 cuando una opción no es válida. No conserva ninguna cookie. Consulta
[Línea de comandos](../automation/cli.md#cli-send-http).

## Problemas {#troubleshooting}

| Lo que ves | Causa habitual |
| --- | --- |
| `… refused the connection — nothing is listening on that port` | El servidor no se está ejecutando, o escucha en otro puerto o dirección. |
| `No answer from … in time` | El servidor es lento o inalcanzable; comprueba la dirección, o sube [[ui:common.timeoutMs]]. |
| `Cannot resolve …` | El nombre de host no resuelve en este equipo — una errata, o un nombre que solo conoce otra red. |
| `A secure connection to … could not be made` | El certificado no es de confianza aquí (autofirmado, caducado, otro nombre), o TLS falló. Consulta [Conexiones seguras](#tls). |
| `… is not a valid address` | La URL está mal formada o no empieza por `http://` o `https://`. |
| El servidor dice que falta el cuerpo o es del tipo equivocado | Ningún encabezado `Content-Type` que corresponda al cuerpo, o un cuerpo vacío. |
| `401` con Digest | Lee el mensaje bajo el estado: consulta [Digest](#digest). |
| [[ui:http.missed]] por encima de 0 | Sube [[ui:common.concurrency]], o baja la tasa: el servidor responde más despacio de lo que la tasa necesita. |
| [[ui:http.failed]] alto aunque el servidor responda | Todo estado fuera de 200–299 cuenta como fallido, 404 y 500 incluidos. |

En un servidor, las solicitudes salen del servidor: `127.0.0.1` es el servidor mismo.
Consulta [Servidor](../server/index.md).

Cada mensaje de error está en [Mensajes de error](../reference/errors.md#transport).

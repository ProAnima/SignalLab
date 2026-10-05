---
title: Seguridad del servidor
description: Quién puede usar un servidor de Signal Lab y cómo deja fuera a los demás — tokens, sesiones, comprobaciones de host y de origen, secretos — y qué envía Signal Lab al exterior.
---

# Seguridad del servidor

Un servidor de Signal Lab envía tráfico real desde el equipo en el que funciona: OSC, UDP, HTTP,
MQTT, tormentas, escaneos, difusiones. Quien pueda usarlo puede hacer todo eso desde ese equipo,
así que el servidor está cerrado por defecto y solo se abre con un token.

::: warning
Trata el token de acceso como una contraseña de la red del equipo. Quien lo tenga puede enviar
tráfico desde el servidor a todo lo que el servidor alcanza.
:::

## De un vistazo {#summary}

- **Sin token, solo este equipo.** Sin token, el servidor escucha en loopback y solo responde a
  nombres de host de loopback. En cualquier otra dirección se niega a iniciarse.
- **Un token para todos los demás.** Los navegadores inician sesión una vez y reciben una cookie
  de sesión; los scripts envían el token con cada solicitud.
- **Solo sus propias páginas.** Las solicitudes que cambian algo, y el WebSocket de eventos, deben
  venir del propio origen del servidor; los comandos solo aceptan JSON.
- **Solo sus propios nombres.** Se rechaza un nombre de host al que el servidor no responde, lo que
  impide el DNS rebinding.
- **Los secretos se quedan dentro.** De solo lectura, desde el entorno o desde archivos, nunca
  devueltos, enmascarados dondequiera que fueran a mostrarse.
- **Nada más allá de su función.** Nunca cambia el firewall del host, no sirve ningún archivo de
  fuera de su carpeta de datos y no necesita privilegios.

## Sin token: solo este equipo {#loopback}

Iniciado sin token, el servidor escucha en `127.0.0.1:1430` y no pide iniciar sesión: es una
herramienta para la persona que está en este equipo. Para impedir que una página web abierta en
cualquier navegador de este equipo llegue a él a través de un nombre que se resuelve como
`127.0.0.1` (DNS rebinding), solo responde a solicitudes cuyo `Host` es un nombre de loopback
(`localhost`, un nombre que termina en `.localhost`, `127.x.x.x` o `[::1]`) o un nombre que
permitas con `--allowed-host`.

Si se le pide escuchar en cualquier otra dirección sin token, no se inicia: explica por qué y
termina con el código `2`.

## El token {#token}

Un token tiene al menos 24 caracteres, sin espacios ni saltos de línea.
`signal-lab-server token` muestra uno aleatorio de 64 caracteres hexadecimales, y
`--generate-token` (activado en la imagen) crea uno en el primer inicio, lo guarda en la carpeta de
datos con permiso de lectura solo para el propio usuario del servidor y lo muestra una vez.
Consulta [El token de acceso](index.md#token) para ver todas las formas de indicarlo.

Comparar un token tarda lo mismo sea cual sea el punto en que difiere, y un token incorrecto cuesta
una espera de un segundo y una advertencia en el registro: adivinar es lento y deja rastro.

## Navegadores: sesiones {#sessions}

Un navegador sin sesión iniciada se envía a la página de inicio de sesión. Su token se canjea una
vez por una sesión, guardada en una cookie que es:

- `HttpOnly`: ningún script de una página puede leerla;
- `SameSite=Strict`: ninguna página de otro sitio puede hacer que el navegador la envíe;
- válida durante 7 días;
- `Secure` con `--secure-cookie`, para que solo viaje por HTTPS (actívalo detrás de un proxy
  HTTPS).

Las sesiones viven en la memoria del servidor: un reinicio cierra la sesión de todos, y
[[ui:app.signOut]] termina una al instante. Se conservan como máximo 1024 sesiones; la más antigua
se elimina primero.

## Scripts: el token bearer {#bearer}

Un script, `signallab --server` y CI envían el token con cada solicitud:

```http
Authorization: Bearer <token>
```

Todos los endpoints necesitan el token o una sesión, excepto `GET /api/health` (si el servidor
responde, su versión, si pide un token) y la página de inicio de sesión. Una solicitud a la API sin
ninguno de los dos recibe `401` con el error `auth.required`; una página recibe la página de inicio
de sesión.

## Nombres de host {#hosts}

| Servidor iniciado | Nombres de host a los que responde |
| --- | --- |
| sin token | los nombres de loopback y los de `--allowed-host` |
| con token, sin `--allowed-host` | cualquier nombre |
| con token y `--allowed-host` | los nombres de loopback y los de `--allowed-host` |

`--allowed-host` (`SIGNALLAB_ALLOWED_HOSTS`) acepta nombres separados por comas, que se comparan sin
el puerto y sin distinguir mayúsculas de minúsculas:

```bash
signal-lab-server --listen 0.0.0.0:1430 --token-file token.txt --allowed-host lab-pc.example.com,192.0.2.10
```

Cualquier otro `Host` recibe `403` con el error `auth.host`. Defínelo en un servidor al que se
llega con nombres conocidos, para que una página de otro sitio no pueda llegar a él a través de un
nombre propio.

## Origen y tipo de contenido {#origin}

- Toda solicitud que cambia algo (todo salvo `GET` y `HEAD`), y el WebSocket de eventos, debe
  llegar sin `Origin` o con el del propio servidor: el mismo host y puerto que su `Host`. Una
  página de otro sitio, o una que envía `Origin: null`, recibe `403` con `auth.origin`. Los scripts
  y `curl` no envían `Origin` y no se ven afectados.
- Los comandos solo aceptan `Content-Type: application/json` (si no, `415` con
  `command.json_required`), así que un formulario de otro sitio no puede enviar ninguno.
- El servidor no responde a solicitudes de otros orígenes (CORS).

## Encabezados de respuesta {#headers}

Cada respuesta lleva:

| Encabezado | Valor |
| --- | --- |
| `Content-Security-Policy` | `default-src 'self'; connect-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; object-src 'none'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'` |
| `X-Content-Type-Options` | `nosniff` |
| `X-Frame-Options` | `DENY` |
| `Referrer-Policy` | `same-origin` |
| `Cache-Control` | `no-store` para la API y la página de inicio de sesión |

La interfaz no carga nada salvo lo que sirve el servidor, no habla con nada salvo con el servidor y
ninguna otra página puede incrustarla en un marco.

## Archivos y tamaños {#files}

- Una descarga (`GET /api/files?path=…`) solo se sirve desde dentro de la carpeta de datos, con un
  máximo de 256 MiB; cualquier otra cosa es `404`.
- El cuerpo de una solicitud tiene como máximo 24 MiB.

## Secretos {#secrets}

El valor de un secreto nunca sale del motor:

- En un servidor, los valores son **de solo lectura**: la variable de entorno
  `SIGNALLAB_SECRET_<NAME>`, o el archivo `<NAME>` de la carpeta de secretos
  (`/run/secrets/signallab` por defecto). Definir o quitar uno desde el navegador se rechaza
  (`secret.read_only`), así que un valor escrito en una página nunca acaba guardado en un lugar
  menos seguro. Consulta [Secretos](index.md#secrets).
- Ningún comando devuelve un valor; la interfaz solo sabe si un nombre está definido.
- Los experimentos nombran los secretos como `{{secret.NAME}}`. Mientras una ejecución o un envío
  los usa, todo texto del que informa (pasos, errores, el informe de la ejecución) muestra `••••`
  en su lugar, y las tramas del [[ui:dock.inspector]] se enmascaran byte a byte.
- Las credenciales de un nodo HTTP se convierten en un encabezado `Authorization` solo en el
  momento de enviar la solicitud; los pasos, las tramas y los informes llevan la respuesta, nunca
  ese encabezado.

## Qué se registra {#audit}

El registro anota, con la dirección del cliente: cada tarea iniciada (tormentas, escaneos,
difusiones, monitores, generadores, ejecuciones, emuladores), cada inicio de sesión y cada intento
de inicio de sesión con un token incorrecto. Consulta [Registros](index.md#logs).

## Lo que el servidor nunca hace {#never}

- **Cambiar el firewall del host.** La aplicación de escritorio puede añadir una regla de firewall
  cuando se lo pides; en un servidor ese comando se rechaza (`firewall.server`). El firewall del
  host es cosa de quien gestiona el host. (El script de instalación de un solo comando ofrece abrir
  el puerto del servidor en ufw o firewalld, y pregunta antes; consulta
  [El firewall del host](index.md#firewall).)
- **Iniciarse accesible sin token**, en cualquier dirección que no sea loopback.
- **Servir un archivo de fuera de su carpeta de datos.**
- **Guardar un secreto** escrito en un navegador.
- **Hablar TLS** por sí mismo: pon delante un proxy HTTPS (consulta
  [Detrás de un proxy HTTPS](index.md#https)).

Todos los límites del motor (como máximo 1024 hosts en un barrido, como máximo 50 000 paquetes por
segundo desde una baliza) se mantienen en un servidor igual que en la aplicación. Son límites de
seguridad, no un permiso: envía tráfico solo a sistemas que sean tuyos o que puedas probar.

## El contenedor {#container}

La imagen se ejecuta como un usuario sin privilegios (uid y gid 10001) y solo escribe en `/data`.
Funciona sin cambios con un sistema de archivos raíz de solo lectura, sin capabilities y con
`no-new-privileges`, tal como la inician el script de instalación y `deploy/compose.yaml`. Cada
imagen se publica con un SBOM, la procedencia de su compilación y una atestación de GitHub firmada:

```bash
gh attestation verify oci://ghcr.io/proanima/signallab:[[version]] -R ProAnima/SignalLab
```

## Qué envía Signal Lab al exterior {#outside}

Además del tráfico que envías tú, Signal Lab habla con dos lugares, ambos del estudio.

### Búsqueda de actualizaciones {#update-check}

Solo la **aplicación de escritorio** busca actualizaciones; un servidor y un navegador nunca lo
hacen. Con la casilla [[ui:update.auto]] marcada (en [[ui:about.open]]), una vez al día, y cada vez que
pulsas [[ui:update.check]], la aplicación pregunta al hub del estudio (`hub.proanima.net`), y a la
última versión de GitHub solo cuando no puede llegar al hub. La consulta lleva:

- la versión de la aplicación;
- el sistema operativo y la arquitectura del procesador;
- un número aleatorio de esta instalación (`X-Install-Id`), creado una vez y guardado con los
  ajustes de la aplicación, para que una nueva versión pueda llegar primero a una parte de las
  instalaciones. No dice nada de ti ni del equipo.

Solo se ofrecen versiones publicadas. Una descarga cuya firma no coincide con la clave integrada en
la aplicación no se instala, y no se instala nada hasta que pulsas [[ui:update.install]].

### Comentarios {#feedback}

[[ui:feedback.open]] (el ✉ de la cabecera, también en [[ui:about.open]]) envía un mensaje a los
desarrolladores a través del hub del estudio, que lo reenvía por correo; la aplicación no guarda
ninguna contraseña para ello. Solo envía lo que muestra el formulario: tu mensaje, tu correo si lo
indicas, las capturas de pantalla que añadas y, bajo [[ui:feedback.logs]], el registro de la consola
y [[ui:feedback.systemInfo]], cada uno de los cuales puedes abrir antes de enviar y desmarcar. El
nombre de este equipo, su dirección y tus carpetas se omiten. Desde un navegador, el formulario lo
envía el servidor.

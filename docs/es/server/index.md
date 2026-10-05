---
title: Ejecutar un servidor
description: Ejecuta Signal Lab en una máquina Linux junto al equipo y úsalo desde cualquier navegador de la red, con una API HTTP para scripts y CI.
---

# Ejecutar Signal Lab como servidor

`signal-lab-server` es Signal Lab sin ventana: el mismo motor, que sirve la
misma interfaz a un navegador. Ponlo en una máquina junto al equipo — un PC de
rack, una VM de control de espectáculos, una caja de laboratorio compartida — y
ábrelo desde Chrome, Firefox o Edge en cualquier punto de la red: cada pantalla
funciona como en la aplicación de escritorio, y las ejecuciones, los informes y
las exportaciones se descargan a través del navegador.

Los scripts y los pipelines usan el mismo servidor a través de su
[API HTTP](../api/index.md), y
[`signallab --server`](../automation/cli.md#run-on-server) le envía ejecuciones.

Algunas cosas son distintas de la aplicación de escritorio:

- **Un servidor es un motor.** Cada página con sesión iniciada ve las mismas
  tareas en marcha, las mismas bibliotecas de señales y emuladores y el mismo
  tarro de cookies de la pantalla [[ui:nav.http]] ([[ui:http.keepCookies]]).
  Quienes comparten un servidor comparten todo eso.
- **Los secretos son del servidor**, leídos de su entorno o de archivos; no se
  pueden definir desde el navegador (consulta [Secretos](#secrets)).
- **El firewall es del host.** El servidor nunca lo cambia; el aviso del
  firewall de la aplicación de escritorio no aparece.
- **Se actualiza con su imagen**, no mediante el actualizador de la aplicación
  (consulta [Actualizar](#update)).

## En un host Linux, con un solo comando {#install-script}

En una máquina Linux con acceso a internet:

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh
```

El script:

1. instala Docker si falta — pregunta antes, con el propio instalador de Docker
   (`get.docker.com`);
2. escribe un `compose.yaml` en `/opt/signallab` (`~/signallab` si no eres
   root);
3. descarga la imagen e inicia el servidor con red de host, para que OSC, UDP,
   difusión, multicast y descubrimiento alcancen la red real;
4. espera hasta que el servidor responde a su comprobación de estado (hasta 90
   segundos);
5. imprime las direcciones que abrir, el token de acceso con el que iniciar
   sesión y los comandos para actualizar, leer los registros y quitarlo;
6. cuando ufw o firewalld está activo, ofrece abrir el puerto del servidor
   (consulta [El firewall del host](#firewall)).

Pasa opciones después de `sh -s --`:

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh -s -- --version [[version]] --port 8430
```

| Opción | Qué hace | Valor por defecto |
| --- | --- | --- |
| `--version X.Y.Z` | La versión de la imagen (`latest` o `X.Y.Z`; se quita una `v` inicial). | `latest` |
| `--port N` | El puerto que usan los navegadores. | `1430` |
| `--listen IP:PORT` | Escuchar solo en una dirección. | `0.0.0.0:<port>` |
| `--dir DIR` | Dónde va el archivo de compose. | `/opt/<name>` como root, `~/<name>` en caso contrario |
| `--name NAME` | El contenedor y su volumen de datos: letras minúsculas, dígitos, `-` y `_`. Un segundo servidor en el mismo host necesita un nombre propio. | `signallab` |
| `--image NAME` | Otra imagen o registro; un `NAME:TAG` completo se usa tal cual. | `ghcr.io/proanima/signallab` |
| `--open-udp PORTS` | Con un firewall activo, deja entrar también UDP en estos puertos, para monitores y esperas: `9000,9100:9110`. | |
| `--no-firewall` | No cambiar nunca ufw ni firewalld. | |
| `--yes`, `-y` | Responder sí: instalar Docker, abrir el firewall, borrar datos con `--purge`. | |
| `--uninstall` | Detener y quitar el servidor; los datos se quedan. | |
| `--purge` | Con `--uninstall`: borrar también los datos y el token. | |
| `--help`, `-h` | Imprimir las opciones. | |

Necesita el plugin Compose de Docker (el paquete `docker-compose-plugin`), que
trae el instalador de Docker. La imagen está compilada para x86_64 y arm64.

**Ejecútalo otra vez para actualizar:** el mismo comando descarga la imagen más
reciente (o la `--version` que indiques) y reinicia el servidor; los datos y el
token se quedan. Con `--name`, indica el mismo nombre otra vez.

**Ajustes propios** — secretos de experimentos, `SIGNALLAB_ALLOWED_HOSTS`,
`SIGNALLAB_SECURE_COOKIE` detrás de HTTPS — van en `compose.override.yaml`
junto al archivo de compose. Docker Compose lo fusiona, y el script reescribe
`compose.yaml` en cada ejecución pero nunca toca el override. Un `compose.yaml`
que no escribió él se conserva como `compose.yaml.before-install`.

```yaml
# compose.override.yaml
services:
  signallab:
    environment:
      SIGNALLAB_ALLOWED_HOSTS: lab-pc.example.com,192.0.2.10
      SIGNALLAB_SECRET_API_TOKEN: ${API_TOKEN}
```

**Para quitarlo:**

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh -s -- --uninstall
```

Los datos se quedan en su volumen de Docker, e instalarlo de nuevo los recupera
con el mismo token. `--uninstall --purge` borra también los datos y el token,
después de preguntar.

### El firewall del host {#firewall}

Con red de host, el servidor escucha en los propios puertos del host, así que el
firewall del host decide quién lo alcanza. Cuando ufw o firewalld está activo, el
script pregunta antes de abrir el puerto TCP del servidor y los puertos UDP de
`--open-udp`, anota lo que abrió (`.firewall` junto al archivo de compose), y
`--uninstall` cierra exactamente eso otra vez. Si lo rechazas, los navegadores de
otras máquinas alcanzan el servidor solo cuando el firewall se lo permite, y los
monitores y las esperas oyen a otras máquinas solo en los puertos UDP que abre.

## Docker {#docker}

### La imagen {#image}

`ghcr.io/proanima/signallab`, para `linux/amd64` y `linux/arm64`, publicada con
cada versión:

| Etiqueta | Qué es |
| --- | --- |
| `X.Y.Z` | Esa versión. |
| `X.Y` | La versión estable más reciente de esa línea. |
| `latest` | La versión estable más reciente. |

Contiene `signal-lab-server`, la interfaz compilada y la línea de comandos
`signallab`, e inicia el servidor con estos ajustes:

| Variable | Valor en la imagen |
| --- | --- |
| `SIGNALLAB_LISTEN` | `0.0.0.0:1430` |
| `SIGNALLAB_DATA_DIR` | `/data` |
| `SIGNALLAB_UI_DIR` | `/usr/share/signal-lab/ui` |
| `SIGNALLAB_GENERATE_TOKEN` | `true` |

Se ejecuta como un usuario sin privilegios (uid y gid 10001), escribe solo en
`/data` (un volumen), expone el puerto `1430` y comprueba su propio estado cada
30 segundos.

### Iniciarlo {#docker-run}

```bash
docker run -d --name signallab --network host --restart unless-stopped \
  -v signallab-data:/data --read-only --cap-drop ALL --security-opt no-new-privileges \
  ghcr.io/proanima/signallab:[[version]]
docker logs signallab                    # on the first start: "Sign in with it:" and the token
```

Luego abre `http://<host>:1430` e inicia sesión con el token. Para ver el token
otra vez más tarde:

```bash
docker exec signallab cat /data/token
```

`--read-only`, `--cap-drop ALL` y `no-new-privileges` son opcionales y no
cuestan nada: el servidor no necesita privilegios y escribe solo en `/data`.

### Docker Compose {#compose}

El `deploy/compose.yaml` del repositorio es igual que un archivo de Compose:

```yaml
name: signallab

services:
  signallab:
    image: ${SIGNALLAB_IMAGE:-ghcr.io/proanima/signallab:latest}
    container_name: signallab
    network_mode: host
    environment:
      SIGNALLAB_GENERATE_TOKEN: "true"
    volumes:
      - signallab-data:/data
    read_only: true
    cap_drop: [ALL]
    security_opt: ["no-new-privileges:true"]
    restart: unless-stopped
    stop_grace_period: 15s

volumes:
  signallab-data:
```

```bash
docker compose up -d
docker exec signallab cat /data/token
```

Define `SIGNALLAB_IMAGE=ghcr.io/proanima/signallab:X.Y.Z` para fijar una versión.

### Red {#networking}

| Red de Docker | Qué funciona | Qué no |
| --- | --- | --- |
| `--network host` (`network_mode: host`), en un host Linux | Todo: OSC, UDP, TCP, HTTP, WebSocket y MQTT hacia la LAN, puertos a la escucha, difusión, multicast, descubrimiento. | — |
| Bridge (lo predeterminado), con puertos publicados | Unicast hacia los hosts que alcanza el contenedor; receptores en puertos publicados (`-p 1430:1430 -p 9000:9000/udp`). | Difusión y multicast; respuestas a puertos que no están publicados. |
| Docker Desktop en Windows o macOS | Unicast y receptores publicados. | Red de host hacia la red física. En Windows, usa la aplicación de escritorio. |

### El volumen de datos {#data-volume}

`/data` guarda todo lo que el servidor conserva: el experimento, las bibliotecas
de señales y emuladores, los informes de ejecución, las exportaciones y el token.
Un volumen con nombre, como el de arriba, empieza siendo propiedad del usuario de
la imagen. Una carpeta del host montada ahí debe ser escribible por el uid 10001:

```bash
sudo mkdir -p /srv/signallab && sudo chown 10001:10001 /srv/signallab
docker run -d --name signallab --network host -v /srv/signallab:/data ghcr.io/proanima/signallab:[[version]]
```

## Sin Docker {#binary}

El servidor se publica como imagen. Para ejecutar `signal-lab-server`
directamente, compílalo desde el código fuente (consulta
[Compilar](../../develop/building.md)):

```bash
npm install
npm run build                            # the interface, into dist/
cargo run --release -p signal-lab-server
```

Sirve `dist/` en `http://127.0.0.1:1430`, solo para esta máquina, sin necesidad
de token. Añade `--listen` y un token para abrirlo a la red.

## Opciones {#options}

Cada opción tiene una variable de entorno, para contenedores. Las opciones ganan
a las variables.

| Opción | Variable | Valor por defecto | Qué hace |
| --- | --- | --- | --- |
| `--listen IP:PORT` | `SIGNALLAB_LISTEN` | `127.0.0.1:1430` | Dónde escuchar. Cualquier dirección que no sea loopback necesita un token. |
| `--token-file PATH` | `SIGNALLAB_TOKEN_FILE` | | Un archivo que contiene el token de acceso (por ejemplo, un secreto de Docker). |
| `--token TOKEN` | `SIGNALLAB_TOKEN` | | El propio token de acceso. Prefiere el archivo: los argumentos son visibles para otros usuarios de la máquina. |
| `--generate-token` | `SIGNALLAB_GENERATE_TOKEN` | desactivado | Sin token indicado, en una dirección más allá de loopback: usar el token guardado en `<data folder>/token`, creándolo en el primer inicio. |
| `--data-dir PATH` | `SIGNALLAB_DATA_DIR` | `Documents/SignalLab` en la carpeta personal del usuario | La carpeta de datos. |
| `--secrets-dir PATH` | `SIGNALLAB_SECRETS_DIR` | `/run/secrets/signallab` | Una carpeta de secretos de solo lectura, un archivo por nombre. |
| `--ui-dir PATH` | `SIGNALLAB_UI_DIR` | `ui` junto al programa, si no `./dist` | La interfaz compilada. Sin una, solo se sirve la API. |
| `--allowed-host NAME` | `SIGNALLAB_ALLOWED_HOSTS` | | Nombres de host con los que se puede alcanzar el servidor, separados por comas. Esos y los nombres de loopback se aceptan siempre, con token o sin él; si no se indica ninguno, un servidor con token responde a cualquier nombre y uno sin token solo a los nombres de loopback (consulta [Nombres de host](security.md#hosts)). |
| `--secure-cookie` | `SIGNALLAB_SECURE_COOKIE` | desactivado | Enviar la cookie de sesión solo por HTTPS. Defínelo detrás de un proxy HTTPS. |
| `--log FILTER` | `SIGNALLAB_LOG` | `info` | Qué registrar: `error`, `warn`, `info`, `debug`, o por módulo (`signal_lab_server=debug`). |
| `--log-format text\|json` | `SIGNALLAB_LOG_FORMAT` | `text` | Las líneas de registro como texto, o un objeto JSON por línea. |

Los interruptores toman `true` o `false` de su variable:
`SIGNALLAB_GENERATE_TOKEN=true`.

| Comando | Qué hace |
| --- | --- |
| `signal-lab-server token` | Imprime un nuevo token aleatorio: 64 caracteres hexadecimales. |
| `signal-lab-server healthcheck` | Termina con `0` cuando un servidor responde en `--listen` (la comprobación de estado de la imagen). |
| `signal-lab-server --version` | Imprime la versión. |
| `signal-lab-server --help` | Imprime cada opción. |

**Códigos de salida:** `0` cuando lo detiene <kbd>Ctrl</kbd>+<kbd>C</kbd> o
`SIGTERM`; `2` cuando se rechazan los ajustes (sin token en una dirección
alcanzable, un token demasiado corto, un token indicado dos veces, un archivo de
token que no se puede leer, `--generate-token` sin carpeta de datos); `1` cuando
no puede escuchar en la dirección, o no se puede escribir en la carpeta de
datos. El motivo se imprime en stderr.

## El token de acceso {#token}

Sin token, el servidor escucha solo en loopback y sirve solo a esta máquina. En
cualquier otra dirección necesita uno, y se niega a iniciarse sin él. Hay tres
formas de indicarlo:

| Cómo | Cuándo usarlo |
| --- | --- |
| `--generate-token` (activado en la imagen) | Nada que configurar: en el primer inicio el servidor crea un token, lo guarda en `<data folder>/token` — legible solo por su propio usuario — y lo imprime una vez en el registro. Los inicios posteriores lo reutilizan, así que los navegadores con sesión iniciada y los scripts siguen funcionando entre reinicios y actualizaciones. Necesita una carpeta de datos (`--data-dir`). |
| `--token-file PATH` | Un token propio en un archivo, como un secreto de Docker. Un salto de línea al final no forma parte del token. |
| `SIGNALLAB_TOKEN` | El token en el entorno. |

Un token tiene al menos **24** caracteres y ni espacios ni saltos de línea;
indícalo de una sola forma. `signal-lab-server token` crea uno bueno:

```bash
docker run --rm ghcr.io/proanima/signallab:[[version]] token > signallab_token.txt
```

y Compose lo entrega como secreto (el archivo debe ser legible por el uid 10001):

```yaml
services:
  signallab:
    environment:
      SIGNALLAB_TOKEN_FILE: /run/secrets/signallab_token
    secrets:
      - signallab_token

secrets:
  signallab_token:
    file: ./signallab_token.txt
```

Un token indicado explícitamente gana a `--generate-token`, y entonces no se crea
ninguno. Para cambiar un token creado, detén el servidor, borra `<data folder>/token` e
inícialo otra vez: creará e imprimirá uno nuevo. Un archivo de
token dañado se reporta, nunca se reemplaza.

## Iniciar sesión {#sign-in}

Abre `http://<host>:1430`. Un servidor con token lleva primero un navegador a su
página de inicio de sesión, en el idioma del navegador; pega el token una vez, y
el navegador permanece con la sesión iniciada durante 7 días.
[[ui:app.signOut]] en la interfaz termina la sesión. Las sesiones se guardan en
la memoria del servidor: un reinicio cierra la sesión de todos.

En loopback sin token no hay inicio de sesión.

Los scripts envían el token con cada solicitud, como
`Authorization: Bearer <token>`:

```bash
curl -fsS http://192.0.2.10:1430/api/invoke/app_info \
  -H "Authorization: Bearer $(cat signallab_token.txt)" \
  -H "Content-Type: application/json" -d 'null'
```

Consulta [La API HTTP](../api/index.md) y [Seguridad del servidor](security.md)
para ver las reglas que hay detrás de todo esto.

## La carpeta de datos {#data}

El servidor guarda sus archivos en la carpeta de datos: el experimento, las
bibliotecas de señales y emuladores, los informes de ejecución, las exportaciones
y un token creado. Es `--data-dir` (`SIGNALLAB_DATA_DIR`), `/data` en la imagen y,
si no, `Documents/SignalLab` en la carpeta personal del usuario con el que se
ejecuta. Una carpeta de datos indicada con `--data-dir` se crea si falta y se
comprueba al inicio: si el servidor no puede escribir allí, se detiene y nombra la
carpeta. Consulta [Archivos](../reference/files.md).

Las descargas (informes, exportaciones) solo salen de dentro de esta carpeta.

## Secretos {#secrets}

Los experimentos leen secretos como `{{secret.NAME}}`. En un servidor son de solo
lectura, desde:

1. la variable de entorno `SIGNALLAB_SECRET_NAME`, si no
2. el archivo `NAME` en la carpeta de secretos (`--secrets-dir`, por defecto
   `/run/secrets/signallab` — la disposición de secretos de Docker).

Un salto de línea al final de un archivo no forma parte del valor; un valor vacío
cuenta como no definido; un valor tiene como máximo 16 KiB. Los nombres son
letras, dígitos y `_`, sin empezar por un dígito. La interfaz muestra qué secretos
están definidos en el servidor, pero definir uno desde el navegador se rechaza —
los valores nunca van a un lugar menos seguro, y nunca vuelven a salir (consulta
[Secretos](security.md#secrets)).

Con Compose, un archivo de secreto por nombre:

```yaml
services:
  signallab:
    secrets:
      - source: api_token
        target: /run/secrets/signallab/API_TOKEN

secrets:
  api_token:
    file: ./api_token.txt
```

El archivo debe ser legible por el uid 10001.

## Detrás de un proxy HTTPS {#https}

El servidor habla HTTP simple. Para HTTPS, pon delante un proxy inverso (Caddy,
nginx, Traefik) que:

- pase el encabezado `Host` sin cambios;
- pase las actualizaciones de WebSocket (la interfaz mantiene una abierta, hacia
  `/api/events`);

e inicia el servidor con `--secure-cookie`, para que la cookie de sesión viaje
solo por HTTPS, y con `--allowed-host` puesto al nombre que usa la gente.

## Actualizar {#update}

[[ui:update.server]]: el actualizador de la aplicación de escritorio no tiene
nada que ver con esto.

- Instalado con el script: ejecuta el mismo comando otra vez.
- Con Compose: `docker compose pull && docker compose up -d`.
- Con `docker run`: descarga la nueva imagen, luego quita el contenedor y
  inícialo otra vez con el mismo volumen.

Los datos y el token están en el volumen, así que se quedan.

## Comprobación de estado {#health}

`GET /api/health` responde a todos, sin token:

```bash
curl -s http://127.0.0.1:1430/api/health
# {"auth":true,"status":"ok","version":"[[version]]"}
```

`auth` indica si el servidor pide un token. El comando
`signal-lab-server healthcheck` pregunta lo mismo en esta máquina y termina con
`0` cuando responde; la imagen lo ejecuta cada 30 segundos (5 segundos de
tiempo de espera, 3 intentos), así que `docker ps` muestra el contenedor como
saludable.

## Registros {#logs}

El servidor registra en stdout: `docker logs -f signallab`. En el nivel
predeterminado (`info`) dice dónde escucha y si necesita un token, sus carpetas
de datos e interfaz, cada tarea iniciada — monitores, generadores, tormentas,
escaneos, ejecuciones, emuladores — con la dirección del cliente, cada inicio de
sesión, cada inicio de sesión con un token incorrecto (como advertencia), y
cuándo una página se queda atrás respecto a los eventos. El nivel `--log debug`
añade cada comando. Las líneas se colorean solo en un terminal (nunca con
`NO_COLOR` definido); `--log-format json` escribe un objeto JSON por línea para
un recopilador de registros.

## Detener {#stop}

<kbd>Ctrl</kbd>+<kbd>C</kbd>, `docker stop` o `SIGTERM` cierra la conexión de
cada página, detiene cada tarea — una ejecución en curso termina como detenida —
y sale con `0`. Los archivos de compose le dan 15 segundos para eso.

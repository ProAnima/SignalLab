---
title: Resolución de problemas
description: Problemas comunes con Signal Lab y cómo solucionarlos — no llega nada, un puerto en uso, el firewall, la red de Docker, el servidor, MQTT, los certificados, las actualizaciones y los registros.
---

# Resolución de problemas

Cada fallo que reporta Signal Lab tiene un código; el mensaje de cada uno está en
[mensajes de error](errors.md). Los fallos de red son los códigos
[`transport`](errors.md#transport): `refused`, `timeout`, `dns`,
`unreachable`, `reset`, `address_in_use`, `address_unavailable`, `denied`,
`tls`, `target_invalid`, `failed`. Los problemas de abajo son los habituales.

## No llega nada {#nothing-arrives}

Primero averigua si algo llega siquiera a Signal Lab: abre la pestaña
[[ui:dock.inspector]] en el panel inferior y pulsa [[ui:ins.arm]]. Cada
datagrama, solicitud y mensaje que envía o recibe una herramienta se lista ahí,
con de dónde vino.

### La dirección de escucha {#listen-address}

- Un [[ui:common.bind]] de `0.0.0.0:<port>` escucha en todas las tarjetas de
  red; `127.0.0.1:<port>` oye solo esta máquina. El equipo en la red necesita
  lo primero.
- El dispositivo debe enviar a la dirección de esta máquina y al puerto en el
  que escuchas. La cabecera muestra el nombre y la dirección de esta máquina.
- Una dirección que no es la de esta máquina falla con `address_unavailable`.

### El firewall {#firewall}

El tráfico en `127.0.0.1` nunca se filtra, que es por lo que una prueba en una
máquina funciona mientras la misma prueba desde otra máquina no recibe nada.

**Windows.** El Firewall de Windows decide por programa. Windows normalmente
pregunta una vez, la primera vez que un programa escucha — y un *Cancelar* ahí
deja una regla que lo bloquea, que gana a cualquier regla de permiso. En una red
que Windows llama pública (la Wi-Fi de un local, a menudo), puede que no pregunte
en absoluto.

- La aplicación de escritorio mira el firewall una vez, cuando un monitor, un
  oyente de descubrimiento, un relé, una ejecución o un emulador empieza a
  escuchar. Cuando el firewall se interpone lo dice en un aviso con
  [[ui:fw.allow]] — o [[ui:fw.allowPublic]] en una red pública. Windows pide
  derechos de administrador, y entonces las reglas de entrada del programa,
  incluida una regla de bloqueo, se reemplazan por una regla de permiso.
  [[ui:fw.dismiss]] oculta el aviso.
- Desde un terminal: `signallab doctor` muestra qué se interpone, y
  `signallab firewall allow` lo arregla (`--public` también para redes públicas),
  con el mismo aviso de administrador.
- Un instalador (`.exe`) instalado *para todos* añade las reglas de permiso él
  mismo (redes privadas y de dominio), salvo que se ejecutara con
  `/NOFIREWALL`. Un instalador *para mí* no puede, y el `.msi` deja el firewall
  a quien lo despliegue.
- Un servidor nunca cambia el firewall de su host: su administrador abre los
  puertos (el script de instalación ofrece hacerlo, con ufw o firewalld).

**Linux.** Un firewall como ufw o firewalld funciona por puerto, no por
programa. `signallab doctor` nombra el que está activo y cómo abrir un puerto,
por ejemplo `sudo ufw allow 9000/udp`.

### Difusión y multicast {#broadcast-multicast}

- Los routers no reenvían la difusión: `255.255.255.255` y `x.x.x.255` alcanzan
  solo el segmento de red en el que está la tarjeta emisora. Con varias
  tarjetas, pon la dirección de la tarjeta en [[ui:bc.bindSource]] (bajo
  [[ui:bc.socketOptions]]), por ejemplo `10.0.0.5:0`.
- Un datagrama multicast solo alcanza a los oyentes que se unieron a su grupo —
  en el oyente de descubrimiento, [[ui:bc.joinGroups]]. Con [[ui:bc.ttl]] en 1,
  lo predeterminado, se queda en esta red.
- Para oír tu propio multicast en la misma máquina, deja [[ui:bc.mcastLoop]]
  activado.

### Un dispositivo que no responde {#no-answer}

UDP no tiene acuse de recibo: un datagrama enviado a un puerto en el que nadie
escucha cuenta igualmente como enviado. Windows entonces reporta el ICMP *puerto
inalcanzable* que recibió como un reinicio de conexión en la siguiente recepción
de ese socket; los monitores, oyentes, relés y esperas de Signal Lab lo ignoran y
siguen escuchando. Así que cuando no llega una respuesta, una espera falla con
`wait.timeout` tras su tiempo, no con un error sobre el envío. Comprueba en el
[[ui:dock.inspector]] que el mensaje salió a la dirección correcta, y luego
comprueba el dispositivo.

## Un envío se rechaza antes de salir {#refused-send}

Estos se comprueban primero, y no se envía nada cuando uno falla:

| Código | Por qué | Solución |
| --- | --- | --- |
| `transport.target_invalid` | El destino no tiene puerto, o no es `IP:port` ni `host:port` | Escribe ambos, como `192.0.2.20:9000` |
| `transport.dns` | El nombre de host no resuelve en esta máquina | Comprueba el nombre, o usa la dirección. Un nombre con una dirección IPv4 se alcanza por IPv4, así que `localhost:9000` encuentra un receptor en `127.0.0.1` |
| `node.osc_address` | Una dirección OSC no empieza por `/` — en el emisor, el generador, una señal o un paso | Empieza por `/`, como `/cue/go` |
| `node.topic_wildcard` | El tema de una publicación MQTT tiene un `+` o `#`, en la conexión de la pantalla como en cualquier otro sitio | Publica en un solo tema; los comodines son para suscribirse |
| `node.too_long` en un mensaje WebSocket | El mensaje supera los 16 MiB | Envía menos; la conexión sigue abierta |

## Un puerto ya está en uso {#port-in-use}

`transport.address_in_use`: otro programa — u otra tarea de Signal Lab — ya
escucha en ese puerto.

- **Un monitor y una ejecución.** Una ejecución abre los puertos de sus esperas,
  emuladores y relés antes de su primer paso, así que un puerto ocupado por un
  [[ui:osc.monitor]], un oyente de descubrimiento o una tarea de emulador hace
  que la ejecución falle antes de empezar. Detén esa tarea primero;
  [[ui:app.stopAll]] las detiene todas.
- **Dos emuladores** en una ejecución no pueden compartir un puerto de un mismo
  transporte: los emuladores HTTP, MQTT y TCP escuchan todos en TCP, y los de OSC
  y UDP en UDP (`emulator.bind_taken`).
- **Escuchar junto al servicio real.** El oyente de descubrimiento puede compartir
  un puerto con un programa que ya lo tiene: deja [[ui:bc.reuse]] activado. En
  Linux, ese programa también debe compartir su puerto. Sin eso, un puerto
  ocupado es `broadcast.port_shared`.
- **Acaba de detenerse.** Un puerto que tenía un emulador o una ejecución
  detenidos se libera un momento después; un emulador iniciado de nuevo enseguida
  espera por él brevemente.
- **Los puertos por debajo de 1024** en Linux necesitan derechos de administrador
  (`denied`). La imagen del servidor se ejecuta sin ninguno, así que usa un
  puerto de 1024 o superior.

## El servidor en Docker no alcanza la red {#docker-network}

La difusión, el multicast y el descubrimiento alcanzan la red física solo con red
de host — `network_mode: host` en el archivo de compose, o
`docker run --network host` — y solo en un host Linux. También permite que los
monitores, las esperas y los emuladores escuchen en los propios puertos del host.
Con la red bridge predeterminada de Docker, el contenedor está en una red propia:
la difusión y el multicast nunca salen de ella, y solo los puertos que publicas
la alcanzan.

En Windows y macOS, la red de host de Docker no alcanza la red física: usa la
aplicación de escritorio en Windows, o ejecuta el servidor en un host Linux.

## SmartScreen advierte sobre el instalador {#smartscreen}

Los instaladores todavía no están firmados, así que Windows SmartScreen dice que
no conoce al editor. Elige *Más información*, y luego *Ejecutar de todas formas*.
Descarga los instaladores solo desde las versiones del proyecto en GitHub.

## El servidor no arranca {#server-start}

`signal-lab-server` comprueba sus ajustes antes de escuchar y sale con un mensaje
en su salida de error:

| Código de salida | Mensaje | Solución |
| --- | --- | --- |
| 2 | refusing to listen on … without a token | Un servidor al que otros pueden llegar necesita un token: `--token-file` o `SIGNALLAB_TOKEN` (crea uno con `signal-lab-server token`), o `--generate-token`. O escucha en `127.0.0.1` |
| 2 | the token has N characters; it needs at least 24 | Usa un token más largo |
| 2 | the token must not contain spaces or line breaks | Un archivo de token puede terminar en un salto de línea; nada más |
| 2 | give the token once | Usa `--token`/`SIGNALLAB_TOKEN` o `--token-file`/`SIGNALLAB_TOKEN_FILE`, no ambos |
| 2 | --generate-token keeps the token in the data folder | Define `--data-dir` o `SIGNALLAB_DATA_DIR` |
| 2 | … it does not hold a valid token; remove it to have a new one made | El archivo `token` de la carpeta de datos está dañado |
| 2 | cannot read the token file … | El archivo que indica `--token-file` falta o no es legible por este usuario |
| 2 | cannot save the new token in … | `--generate-token` no pudo escribir `token` en la carpeta de datos: haz que la carpeta sea escribible por este usuario |
| 1 | cannot listen on … | La dirección no es la de esta máquina, o el puerto está ocupado |
| 1 | the data folder … must be writable by this user | En Docker, una carpeta montada debe ser escribible por el uid 10001 |

Un token que creó `--generate-token` se imprime una vez, en el primer inicio
(`docker logs signallab` lo muestra), y se guarda en `token` en la carpeta de
datos: `docker exec signallab cat /data/token`. Consulta
[el servidor](../server/index.md).

## No se puede iniciar sesión en el servidor {#sign-in}

| Qué ves | Por qué | Solución |
| --- | --- | --- |
| *That token is not right.* | Un token incorrecto | Cópialo otra vez de donde lo guarda el servidor; la respuesta tarda un segundo a propósito |
| `auth.host` | El servidor no responde al nombre de la barra de direcciones | Ábrelo por un nombre que acepte. Los nombres de loopback (`localhost`, `127.x.x.x`, `[::1]`) siempre pasan. Sin token el servidor responde solo a esos y a los nombres de `--allowed-host`; con token, a cualquier nombre salvo que `--allowed-host` lo restrinja |
| `auth.origin` | Una solicitud vino de una página de otro origen | Detrás de un proxy inverso, pasa el `Host` del navegador al servidor (nginx: `proxy_set_header Host $host;`), para que `Origin` y `Host` coincidan |
| Vuelve a la página de inicio de sesión tras iniciarla | El navegador no conservó la cookie de sesión | Con `--secure-cookie`, se debe alcanzar el servidor por HTTPS |
| Sesión cerrada tras un rato | Las sesiones duran 7 días y terminan cuando el servidor se reinicia; pasadas 1024 sesiones, se va la más antigua | Inicia sesión otra vez |

## La conexión al servidor se corta continuamente {#connection-lost}

[[ui:app.connectionLost]] significa que el socket de eventos de la página,
`/api/events`, se cerró. La página se reconecta por su cuenta, tras medio segundo
y luego con menos frecuencia, hasta cada 15 s. Lo que pasó mientras tanto no se
reproduce: las tareas en marcha informan de nuevo según continúan. Detrás de un
proxy inverso, asegúrate de que pasa las actualizaciones de WebSocket para
`/api/events` y no cierra las conexiones tranquilas en menos de 20 s (el servidor
hace ping cada 20 s). Una página que dice que el servidor no tiene tal dirección
(`api.not_found`) es más antigua que el servidor: recárgala.

## MQTT no conecta {#mqtt}

Signal Lab habla MQTT 3.1.1 sobre TCP simple. Conecta y espera la respuesta del
broker (CONNACK) antes de informar éxito, así que el motivo está en el botón que
conecta:

| Código | Por qué | Solución |
| --- | --- | --- |
| `transport.refused` | Nada escucha en ese puerto | Comprueba el puerto: 1883 es el habitual |
| `transport.timeout` | Ninguna conexión TCP en 6 s | Comprueba la dirección, la red, el firewall del broker |
| `transport.dns` | El nombre de host no resuelve | Comprueba el nombre, o usa la dirección |
| `transport.unreachable` | No hay ruta al broker | Comprueba la red y la dirección |
| `transport.reset` | El broker cerró la conexión de inmediato | A menudo un puerto TLS (8883) — Signal Lab no habla MQTT sobre TLS |
| `mqtt.no_answer` | El puerto está abierto, pero no llegó ningún CONNACK en 6 s | A menudo un puerto WebSocket — Signal Lab no habla MQTT sobre WebSocket |
| `mqtt.protocol` | Lo que respondió no es un broker MQTT | Comprueba el puerto |
| `mqtt.refused_protocol` | El broker no acepta MQTT 3.1.1 | Activa 3.1.1 en el broker |
| `mqtt.refused_client_id` | El broker rechaza el id de cliente | Usa otro id de cliente |
| `mqtt.refused_unavailable` | El broker no está disponible | Inténtalo más tarde |
| `mqtt.refused_credentials` | El nombre de usuario o la contraseña son incorrectos | Compruébalos |
| `mqtt.refused_not_authorized` | El usuario no puede conectar | Comprueba las reglas de acceso del broker |
| `mqtt.client_id_required` | El id de cliente está vacío | Rellénalo |

Un broker descarta la más antigua de dos conexiones con el mismo id de cliente:
cuando una conexión se cierra continuamente, busca otro cliente que use el mismo
id.

## Un certificado no es de confianza {#tls}

`transport.tls`, en `https://` o `wss://`: el certificado del servidor no es de
confianza, no nombra el host que pediste, o no se pudo acordar TLS. Signal Lab
comprueba los certificados como lo hace el sistema y no tiene ningún interruptor
para saltarse la comprobación. HTTPS y WSS confían en los mismos certificados:
los del sistema operativo donde se ejecuta el motor — el almacén de certificados
de Windows en Windows, los certificados de CA del sistema en Linux y en la imagen
del servidor. Para un certificado autofirmado o tu propia CA, añádelo a los
certificados de confianza de esa máquina (para la imagen del servidor, una imagen
construida sobre ella que añada el certificado), y conecta por el nombre que
lleva el certificado.

## Falta un secreto {#secrets}

`secret.missing`: un experimento usa `{{secret.NAME}}` y no hay ningún valor
guardado con ese nombre donde se ejecuta.

- **Aplicación de escritorio en Windows**: defínelo en [[ui:exp.secrets]] en el
  [[ui:exp.params]] del editor. Se guarda en el Administrador de credenciales de
  Windows, así que una máquina nueva necesita definirlo otra vez.
- **Servidor**: pon el valor en el archivo `/run/secrets/signallab/NAME` (o la
  carpeta que indique `--secrets-dir`) o en la variable de entorno
  `SIGNALLAB_SECRET_NAME`. Un servidor no puede definir secretos desde la página
  (`secret.read_only`).
- **La aplicación de escritorio en Linux** no tiene almacén para secretos
  (`secret.unsupported`). Ejecuta tal experimento con `signallab run`, que lee
  los secretos de archivos y variables, o en un servidor.

Consulta [archivos](files.md#secrets).

## Una ejecución se detiene a los cinco minutos {#run-timeout}

Una ejecución que tarda más de 300 s falla con `run.timeout`; ese es el máximo
que puede durar una ejecución. A una ejecución se le puede dar un límite más
corto a través de la API (`timeout` de [`/api/run`](../api/run.md#request)) o de
la línea de comandos (`signallab run --timeout`).

## La aplicación no se actualiza {#updates}

- La aplicación de escritorio busca una versión nueva una vez al día mientras
  [[ui:update.auto]] está activado, y cuando pulsas [[ui:update.check]] en
  [[ui:about.open]].
- Pregunta primero al hub del estudio y a GitHub cuando no se puede alcanzar el
  hub. Cuando una red bloquea ambos, [[ui:update.check]] da
  [[ui:update.checkFailed]]; la búsqueda diaria falla sin decir nada.
- Solo se le ofrecen versiones publicadas, nunca borradores ni versiones previas.
- Una versión nueva llega a la aplicación cuando el hub la ofrece, lo que puede
  ser algún tiempo después de que aparezca en GitHub: el hub despliega una versión
  a una parte de las instalaciones cada vez.
- Solo instala cuando pulsas [[ui:update.install]] — antes se detienen las tareas
  en marcha — y solo una versión cuya firma se verifica:
  [[ui:update.installFailed]] en caso contrario.
- Un servidor se actualiza con su imagen: `docker compose pull && docker compose up -d`
  en la carpeta de su archivo de compose.

## Dónde están los registros {#logs}

- **Aplicación de escritorio**: no escribe archivos de registro. La pestaña
  [[ui:console.title]] en el panel inferior lista lo que hizo cada herramienta y
  lo que salió mal, y [[ui:feedback.open]] lo adjunta a un mensaje a los
  desarrolladores (sin el nombre de este equipo, su dirección ni tus carpetas). El
  informe de cada ejecución está en `runs/` en la carpeta de datos.
- **Servidor**: registra en su salida y error estándar — `docker logs signallab`
  en Docker. Cada inicio de tarea se registra con la dirección del cliente que lo
  pidió. `--log` (o `SIGNALLAB_LOG`) define el nivel: `error`, `warn`, `info` (lo
  predeterminado) o `debug`; `--log-format json` (o `SIGNALLAB_LOG_FORMAT`)
  escribe un objeto JSON por línea.
- **Línea de comandos**: `signallab` escribe sus mensajes en su salida de error;
  consulta [la línea de comandos](../automation/cli.md).

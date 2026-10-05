---
title: Fallos
description: Relés de degradación y emuladores como nodos de una ejecución — una red deficiente y una dependencia que falla, activadas en el momento elegido, contadas fase a fase en el informe y repetibles con la semilla.
---

# Fallos como nodos

Para ver cómo se las arregla un sistema cuando la red se degrada o una dependencia cae, pon el
fallo en el experimento. Un relé o un emulador se abre con la ejecución, un paso lo cambia en el
momento elegido, y el informe de la ejecución cuenta lo que pasó en cada fase. El final de la
ejecución (superada, fallida o detenida) los cierra, así que nada se queda degradado después.

| Nodo | Qué hace |
| --- | --- |
| [[ui:exp.node.impairment]] | un relé entre el sistema sometido a prueba y su destino, que degrada lo que pasa por él durante toda la ejecución |
| [[ui:exp.node.impairment_change]] | cambia un relé de la ejecución a otro perfil, a partir de este paso |
| [[ui:exp.node.emulator]] | una API, un dispositivo o un bróker cuyo papel hace Signal Lab, durante toda la ejecución |
| [[ui:exp.node.emulator_state]] | deja caído un emulador de la ejecución, o lo vuelve a levantar |

Los cuatro están en el menú de añadir, en [[ui:exp.group.fault]] y
[[ui:exp.group.emulate]]. Sus campos están en
[la referencia de los nodos](nodes.md); el relé en sí se describe en
[Degradación](../tools/impairment.md), y los emuladores, en
[Emuladores](../tools/emulators.md).

## Degradación {#impairment}

El sistema sometido a prueba envía al relé en lugar de a su destino real; el relé reenvía al
destino, devuelve las respuestas y degrada ambos sentidos.

| Campo | Qué |
| --- | --- |
| [[ui:exp.relayListen]] | `IP:port` al que envía o se conecta el sistema sometido a prueba, con un puerto distinto de 0 |
| [[ui:exp.relayTarget]] | `IP:port` del destino real, o `host:port`; el nombre de host se resuelve al iniciar la ejecución |
| [[ui:ns.protocol]] | UDP (cada datagrama corre su propia suerte) o TCP (cada conexión se une a otra propia hacia el destino) |
| el perfil | un [[ui:ns.preset]] o valores propios |

Lo que un relé lee de su perfil depende del protocolo; los demás valores se omiten:

| Protocolo | Degradaciones |
| --- | --- |
| UDP | latencia, jitter, pérdida de paquetes, pérdida en ráfagas, duplicación, corrupción, reordenación, un límite de ancho de banda, sin conexión |
| TCP | latencia y jitter (un flujo se mantiene en orden), un límite de ancho de banda (el emisor se ralentiza, no se descarta nada), conexiones restablecidas, conexiones semiabiertas, sin conexión |

**Se abre antes del primer paso.** Todos los relés del experimento se abren cuando empieza la
ejecución, como los sockets de las esperas, así que sus campos [[ui:exp.relayListen]] y
[[ui:exp.relayTarget]] solo aceptan texto y parámetros
(`node.params_only`): `{{relay}}` con un parámetro `relay`, nunca una variable.
Un puerto que no se puede abrir, o un nombre de destino que no se puede resolver, detiene la ejecución antes de que haya tráfico, en el nodo.

**Se pasa de largo en el flujo.** Cuando la ejecución llega al nodo, este se supera al instante y
la línea de tiempo indica qué degrada y con qué. El relé funciona desde el inicio de la ejecución
hasta su final, esté donde esté el nodo en el grafo.

**Se cierra con la ejecución.** Termine como termine la ejecución, el relé se cierra; las
conexiones de un relé TCP se cierran con él. Un relé que dejó de retransmitir por su cuenta
conserva el motivo: los pasos que lo usan fallan con él, y el informe lo indica.

::: tip Pasar por una degradación
En un nodo [[ui:exp.node.osc]] o [[ui:exp.node.udp]], [[ui:exp.routeThrough]] coloca delante un
nodo de degradación: el relé escucha en un puerto libre de `127.0.0.1`, reenvía al destino del nodo
con el preajuste [[ui:ns.preset.lan]], y el nodo pasa a enviar al relé.
:::

## Cambiar degradación {#change-impairment}

[[ui:exp.node.impairment_change]] nombra uno de los relés del experimento en el campo
[[ui:exp.relay]] e indica el perfil con el que degrada a partir de ese paso. El relé conserva su
puerto y sus conexiones; los nuevos valores se aplican al siguiente paquete o fragmento. La línea
de tiempo muestra el nuevo perfil.

Cada cambio cierra una **fase**. El informe de la ejecución guarda, para cada relé:

- sus direcciones de escucha y de destino, y su protocolo cuando es TCP;
- sus recuentos totales: recibidos, reenviados, descartados, limitados, duplicados, corrompidos,
  reordenados, bytes y, en TCP, las conexiones, las restablecidas y las que quedaron semiabiertas;
- cada fase: el nombre del perfil, cuándo empezó y terminó, en milisegundos desde que se abrió el
  relé, y los mismos recuentos solo para esa fase.

Un paquete se cuenta en la fase que decidió su suerte, aunque su copia retrasada salga después del
cambio. Un relé conserva sus últimas 1000 fases; las más antiguas se cuentan, pero no se guardan.

Se rechaza un [[ui:exp.node.impairment_change]] que no nombra ningún relé del experimento
(`impair.relay_unknown`).

## Emulador {#emulator}

[[ui:exp.node.emulator]] hace el papel de una dependencia durante toda la ejecución: una API HTTP,
un dispositivo OSC, UDP o TCP, o un bróker MQTT. Es el mismo emulador que la pantalla
[Emuladores](../tools/emulators.md) ejecuta por separado:
[[ui:emu.edit]] abre sus reglas, [[ui:emu.toLibrary]] guarda una copia en la biblioteca y
[[ui:emu.fromLibrary]] toma una de ella.

- Se abre antes del primer paso y responde hasta que termina la ejecución; un puerto que no se puede
  abrir detiene la ejecución antes de que haya tráfico. En el flujo se supera al instante.
- Su dirección es un `IP:port` literal. Sus patrones de coincidencia solo aceptan parámetros; sus
  respuestas son plantillas que se leen con lo que llegó (`{{request.…}}`) y los parámetros de la
  ejecución. No puede leer secretos.
- Sus decisiones aleatorias (una mezcla ponderada de respuestas, el jitter del retardo, los
  generadores en las respuestas) parten de la semilla de la ejecución.
- Un emulador HTTP es también lo que escucha un [[ui:exp.node.wait_http]] en la misma dirección:
  comprueba lo que envió el sistema sometido a prueba. Si no hay ningún emulador ahí, la escucha
  propia de la ejecución responde a cada solicitud con `204`.
- Un emulador OSC o UDP comparte su puerto con las esperas de la ejecución en él: ambos ven cada
  datagrama.
- Un emulador MQTT es un bróker que los nodos [[ui:exp.node.mqtt]] y
  [[ui:exp.node.wait_mqtt]] de la ejecución pueden usar como cualquier otro.

El informe de la ejecución guarda, para cada nodo de emulador, su nombre, protocolo y dirección, y
sus recuentos: las solicitudes en total, las que ninguna regla aceptó, las que fallaron, las que lo
encontraron caído, los mensajes que un bróker no pudo entregar a un cliente lento y las
coincidencias de cada regla.

## Emulador caído y activo {#emulator-state}

[[ui:exp.node.emulator_state]] nombra uno de los emuladores de la ejecución en el campo
[[ui:exp.emulatorNode]]; su [[ui:exp.emulatorDownState]] es [[ui:exp.emulatorGoesDown]] o
[[ui:exp.emulatorComesUp]]. Mientras está caído:

| Emulador | Encuentra |
| --- | --- |
| HTTP | lo que indique [[ui:exp.downFault]]: [[ui:emu.outageFault.unavailable]] (`503`, cuerpo `{"error":"unavailable"}`), [[ui:emu.outageFault.reset]] o [[ui:emu.outageFault.timeout]] (la solicitud se retiene hasta que el cliente desiste, como máximo 120 s) |
| Dispositivo TCP, bróker MQTT | las conexiones se cortan y las nuevas se rechazan |
| Dispositivo OSC, UDP | no se responde nada |

Lo que llega mientras está caído se cuenta como `down`, nunca como una solicitud que ninguna regla
aceptó. Un [[ui:exp.node.wait_http]] sigue viendo las solicitudes. El emulador sigue caído hasta
que un paso lo levanta, diga lo que diga su propio programa de caídas, y el final de la ejecución
lo cierra en cualquier caso.

Se rechaza un [[ui:exp.node.emulator_state]] que no nombra ningún emulador del experimento
(`emulator.node_unknown`).

## Caídas programadas {#outage}

Un emulador también puede caer por sí mismo: en sus reglas, [[ui:emu.outage]] define
[[ui:emu.outageUp]] y [[ui:emu.outageDown]], cada uno de 10–3 600 000 ms, y
[[ui:emu.outageFault]] para HTTP. Responde durante el primero, está caído durante el segundo, y así
sucesivamente, contando desde que se abrió (en una ejecución, antes del primer paso). Mientras está
caído según su programa, el `503` de un emulador HTTP lleva
`Retry-After` con los segundos enteros que faltan para que vuelva, al menos 1; un `503`
mientras un paso [[ui:exp.node.emulator_state]] lo mantiene caído no lleva ninguno, porque nadie
sabe cuándo terminará.

Un programa no necesita ningún paso; un paso no necesita ningún programa. Usa el programa para una
dependencia que va y viene, y el paso para una caída en un punto elegido del flujo.

## Ejemplo: una caída detrás de un enlace lento {#example}

Un cliente pide un pedido a una API a través de un relé. Mientras pregunta, una segunda rama
ralentiza el enlace a [[ui:ns.preset.4g]], deja caída la API durante dos segundos, la vuelve a
levantar y deja el enlace limpio otra vez. El cliente debe seguir preguntando hasta obtener su
respuesta.

```text
start → orders → link → split
split ─ branch1 → settle → until_ok ─ done → answered → joined
                           until_ok ─ body → get → status → pause → until_ok
split ─ branch2 → slow → down → outage → up → clean → joined
joined → end
```

Los nombres son los id de los nodos en el archivo de más abajo.

1. Añade un parámetro `api` = `http://127.0.0.1:18091`: el relé, no la API.
2. Añade un nodo [[ui:exp.node.emulator]]: HTTP, `127.0.0.1:18090`, una ruta
   `GET /orders/:id` que responde `200` con `{"order":"{{request.params.id}}"}`.
3. Después, un nodo [[ui:exp.node.impairment]]: [[ui:exp.relayListen]]
   `127.0.0.1:18091`, [[ui:exp.relayTarget]] `127.0.0.1:18090`,
   [[ui:ns.protocol]] TCP, preajuste [[ui:ns.preset.lan]].
4. Después, un nodo [[ui:exp.node.fork]].
5. En [[ui:exp.branch1]], el cliente: una [[ui:exp.node.delay]] de 300 ms y luego un
   [[ui:exp.node.loop]] ([[ui:exp.loopMax]] 40, [[ui:exp.loopUntilOn]]
   `{{status}}` [[ui:exp.op.eq]] `200`). Su cuerpo: una [[ui:exp.node.http]]
   `GET {{api}}/orders/42`, un [[ui:exp.node.extract]] del
   [[ui:exp.from.status]] en `status` y una pausa de 250 ms, conectada de vuelta al
   bucle. En [[ui:exp.portDone]], una
   [[ui:exp.node.log]] `Orders API answers again: HTTP {{status}}`.
6. En [[ui:exp.branch2]], los fallos: un nodo [[ui:exp.node.impairment_change]] que pasa el
   relé a [[ui:ns.preset.4g]]; un nodo [[ui:exp.node.emulator_state]] que pone la Orders API en
   estado [[ui:exp.emulatorGoesDown]] con [[ui:emu.outageFault.unavailable]]; una pausa de
   2000 ms; otro [[ui:exp.node.emulator_state]] que la devuelve al estado
   [[ui:exp.emulatorComesUp]]; y otro [[ui:exp.node.impairment_change]] que vuelve a
   [[ui:ns.preset.lan]].
7. Conecta ambas ramas a un nodo [[ui:exp.node.join]], y este a
   [[ui:exp.node.end]].
8. Ejecútalo.

La línea de tiempo muestra las solicitudes del cliente respondidas con `503` a través del enlace
lento, la API que vuelve y luego `200`, y el bucle que sale por
[[ui:exp.portDone]]. El informe cuenta unas cinco solicitudes que encontraron la API caída y una
respondida por su ruta, y las tres fases del relé ([[ui:ns.preset.lan]] durante un instante,
[[ui:ns.preset.4g]] durante la caída y otra vez [[ui:ns.preset.lan]]), cada una con su propio
tráfico.

::: details El experimento como archivo
Guárdalo como archivo `.json` y ábrelo con [[ui:exp.importJson]] en
[[ui:exp.documents]].

```json
{
  "version": 9,
  "name": "Outage behind a slow link",
  "params": [{ "name": "api", "value": "http://127.0.0.1:18091" }],
  "profiles": [],
  "profile": null,
  "seed": null,
  "nodes": [
    { "id": "start", "type": "start", "x": 40, "y": 270 },
    { "id": "orders", "type": "emulator", "x": 260, "y": 270,
      "emulator": { "name": "Orders API", "bind": "127.0.0.1:18090", "protocol": "http",
        "routes": [{ "method": "GET", "path": "/orders/:id", "when": [], "order": "sequence",
          "responses": [{ "status": 200, "headers": [], "body": "{\"order\":\"{{request.params.id}}\"}", "delay_ms": 0, "jitter_ms": 0, "fault": "none", "weight": 1 }] }],
        "fallback": null } },
    { "id": "link", "type": "impairment", "x": 490, "y": 270, "listen": "127.0.0.1:18091", "target": "127.0.0.1:18090", "protocol": "tcp",
      "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } },
    { "id": "split", "type": "fork", "x": 720, "y": 270 },
    { "id": "settle", "type": "delay", "x": 950, "y": 140, "ms": 300 },
    { "id": "until_ok", "type": "loop", "x": 1180, "y": 140, "max": 40,
      "until": { "value": "{{status}}", "op": "eq", "expected": "200" } },
    { "id": "get", "type": "http", "x": 1410, "y": 20,
      "request": { "method": "GET", "url": "{{api}}/orders/42", "headers": [], "body": null, "timeout_ms": 3000 } },
    { "id": "status", "type": "extract", "x": 1640, "y": 20, "variable": "status", "from": "status", "expr": "" },
    { "id": "pause", "type": "delay", "x": 1870, "y": 20, "ms": 250 },
    { "id": "answered", "type": "log", "x": 1410, "y": 140, "message": "Orders API answers again: HTTP {{status}}" },
    { "id": "slow", "type": "impairment_change", "x": 950, "y": 400, "relay": "link",
      "profile": { "name": "4g", "latency_ms": 60, "jitter_ms": 25, "rate_kbps": 20000 } },
    { "id": "down", "type": "emulator_state", "x": 1180, "y": 400, "emulator": "orders", "down": true, "fault": "unavailable" },
    { "id": "outage", "type": "delay", "x": 1410, "y": 400, "ms": 2000 },
    { "id": "up", "type": "emulator_state", "x": 1640, "y": 400, "emulator": "orders", "down": false, "fault": "unavailable" },
    { "id": "clean", "type": "impairment_change", "x": 1870, "y": 400, "relay": "link",
      "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } },
    { "id": "joined", "type": "join", "x": 2100, "y": 270 },
    { "id": "end", "type": "end", "x": 2330, "y": 270 }
  ],
  "edges": [
    { "from": "start", "to": "orders" },
    { "from": "orders", "to": "link" },
    { "from": "link", "to": "split" },
    { "from": "split", "to": "settle", "port": "branch1" },
    { "from": "split", "to": "slow", "port": "branch2" },
    { "from": "settle", "to": "until_ok" },
    { "from": "until_ok", "to": "get", "port": "body" },
    { "from": "get", "to": "status" },
    { "from": "status", "to": "pause" },
    { "from": "pause", "to": "until_ok" },
    { "from": "until_ok", "to": "answered", "port": "done" },
    { "from": "answered", "to": "joined" },
    { "from": "slow", "to": "down" },
    { "from": "down", "to": "outage" },
    { "from": "outage", "to": "up" },
    { "from": "up", "to": "clean" },
    { "from": "clean", "to": "joined" },
    { "from": "joined", "to": "end" }
  ]
}
```
:::

Dos plantillas de [[ui:exp.documents]] hacen lo mismo de otras formas:
[[ui:exp.templateFaults]] envía datagramas a un dispositivo emulado a través de un relé UDP que se
cambia a limpio, con pérdidas, sin conexión y de nuevo limpio;
[[ui:exp.templateOutage]] deja caída una API emulada durante dos segundos mientras un cliente sigue
preguntando.

## Puertos {#ports}

Los sockets de una ejecución (esperas, respuestas, emuladores, relés) no pueden compartir un puerto
del mismo protocolo; un socket UDP y uno TCP sí pueden usar el mismo número. En el campo
[[ui:exp.relayListen]] de un relé, una dirección en `0.0.0.0` choca con todas las direcciones del
mismo puerto.

| Socket | No puede compartir su puerto con |
| --- | --- |
| un emulador HTTP, TCP o MQTT | otro de ellos (`emulator.bind_taken`) |
| un emulador OSC o UDP | otro de ellos (`emulator.bind_taken`) |
| un emulador TCP o MQTT | un [[ui:exp.node.wait_http]] (`emulator.bind_taken`) |
| el [[ui:exp.relayListen]] de un relé UDP | otro relé UDP, un emulador OSC o UDP, una espera o el socket de una respuesta (`impair.bind_taken`) |
| el [[ui:exp.relayListen]] de un relé TCP | otro relé TCP, un emulador HTTP, TCP o MQTT, un [[ui:exp.node.wait_http]] (`impair.bind_taken`) |

Compartidos a propósito: un emulador HTTP y los pasos [[ui:exp.node.wait_http]] en su
dirección; un emulador OSC o UDP y las esperas en su puerto; las esperas en una misma dirección
entre sí.

Un relé no puede reenviar hacia sí mismo, ni directamente ni a través de otros relés: su tráfico
daría vueltas en loopback (`impair.loop`). Dos relés seguidos delante de un dispositivo no son
ningún problema.

## Repetir una ejecución con fallos {#seed}

Cada decisión que toma un relé (si un paquete se pierde, se duplica, se corrompe o se retiene,
cuánto jitter recibe) se sortea a partir de la semilla de la ejecución, por separado para cada
sentido y paquete a paquete. Las decisiones aleatorias de un emulador también parten de ella.
Vuelve a ejecutar con la misma semilla y el mismo tráfico, y los mismos paquetes correrán la misma
suerte: un fallo visto una vez se puede volver a ver.

Para conservar la semilla, pulsa [[ui:exp.pinSeed]] junto a ella en la línea de tiempo, o ejecuta
con ella desde [[ui:exp.runWith]]; consulta [semillas](runs.md#seeds). Lo que la semilla no puede
fijar es el momento: cuándo envía el sistema sometido a prueba y, por tanto, en qué fase cae un
paquete.

---
title: Escáner
description: Averigua qué puertos TCP de un host aceptan conexiones, con el saludo que envía cada servicio, usando un escaneo de conexión TCP concurrente.
---

# Escáner

La pantalla [[ui:nav.scan]] intenta abrir una conexión TCP a cada puerto de un
intervalo en un host y lista los puertos que aceptan. Úsala para confirmar qué
servicios están escuchando realmente en un dispositivo —el puerto de control de
un proyector, la interfaz web de un conmutador, el puerto que tu propio servicio
debería haber abierto— y, para los servicios que saludan primero, lo que dicen.

::: danger Uso responsable
Un escaneo se conecta a cada puerto del intervalo. Escanea solo hosts que sean
tuyos o que estés autorizado a probar: en otras redes, un escaneo puede disparar
la detección de intrusiones y a menudo va contra las normas.
:::

## Escanear un host {#scan}

1. Introduce el [[ui:sc.host]]: un host, una dirección IP o un nombre.
2. Define [[ui:sc.fromPort]] y [[ui:sc.toPort]], o elige un [[ui:sc.preset]].
3. Ajusta [[ui:common.concurrency]] y [[ui:common.timeoutMs]] si lo necesitas.
4. Deja [[ui:sc.grabBanner]] activado para leer lo que dice primero cada
   servicio.
5. Pulsa [[ui:sc.startScan]].

Una barra bajo el botón muestra cuántos puertos se han probado, de cuántos, y
cuántos están abiertos. Los puertos abiertos aparecen a la derecha a medida que
se encuentran. El escaneo termina cuando se ha probado cada puerto;
[[ui:sc.stopScan]] lo termina antes, y los puertos aún no probados no se prueban.
Los campos quedan fijos mientras funciona.

| Campo | Qué | Predeterminado |
| --- | --- | --- |
| [[ui:sc.host]] | El host que escanear. | `127.0.0.1` |
| [[ui:sc.fromPort]], [[ui:sc.toPort]] | El intervalo, ambos extremos incluidos, 1–65 535. Cuando el primero es mayor, se intercambian. | 1–1024 |
| [[ui:common.concurrency]] | Cuántos intentos de conexión están abiertos a la vez, 1–1024. | 400 |
| [[ui:common.timeoutMs]] | Cuánto esperar por cada conexión, 50–10 000 ms. | 500 |
| [[ui:sc.grabBanner]] | Después de conectar, espera hasta 400 ms a que el servicio envíe algo y conserva sus primeros 256 bytes. | activado |

Un [[ui:common.concurrency]] o [[ui:common.timeoutMs]] fuera de su intervalo se
lleva a él cuando empieza el escaneo.

[[ui:sc.preset]] rellena el intervalo:

| Preajuste | Puertos |
| --- | --- |
| [[ui:sc.preset.wellKnown]] | 1–1024 |
| [[ui:sc.preset.common]] | 1–10 000 |
| [[ui:sc.preset.osc]] | 8000–9100 |
| [[ui:sc.preset.full]] | 1–65 535 |

## Cuánto tarda un escaneo {#duration}

Un puerto que acepta responde de inmediato. Un puerto que no, puede costar hasta
todo el tiempo de espera: un firewall que descarta los intentos de conexión nunca
responde, y en Windows incluso un rechazo puede tardar más que el tiempo de
espera predeterminado. Así que un escaneo de un host que no responde nada tarda
aproximadamente:

```text
ports ÷ concurrency × timeout
```

El intervalo completo con los valores predeterminados:
65 535 ÷ 400 × 0,5 s ≈ 82 s. Sube [[ui:common.concurrency]] o baja
[[ui:common.timeoutMs]] para ir más rápido; baja el tiempo de espera demasiado y
los puertos abiertos de un host lento se pasan por alto.

## Leer los resultados {#results}

[[ui:sc.openPorts]] lista cada puerto que aceptó una conexión, en orden de puerto:

| Columna | Qué |
| --- | --- |
| [[ui:sc.port]] | El puerto abierto. |
| [[ui:common.time]] | Cuándo se encontró. |
| [[ui:sc.banner]] | Lo que envió primero el servicio, con los saltos de línea convertidos en espacios, o `—`. |

Un puerto que rechazó y uno que no respondió dentro del tiempo de espera se dejan
ambos fuera: el escáner no distingue cerrado de filtrado. La lista conserva hasta
2000 puertos abiertos.

Solo los servicios que hablan primero tienen un banner — SSH, SMTP, FTP, muchos
protocolos de control de dispositivos. Un servidor web espera una solicitud, así
que su puerto muestra `—`. Capturar banners hace que cada puerto abierto tarde
hasta 400 ms más.

Una conexión que el escáner abre se cierra inmediatamente. El escáner no envía
nada por ella.

Con la captura activada, cada puerto abierto está también en el
[Inspector](inspector.md), con su banner y el veredicto `open`.

## Véase también {#related}

- [Tormenta](storm.md): carga sobre un puerto que hayas encontrado.
- [UDP y TCP](../protocols/udp-tcp.md): habla con él.
- [Solución de problemas](../reference/troubleshooting.md)

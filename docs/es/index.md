---
layout: home
title: Signal Lab
description: Signal Lab es un laboratorio de pruebas para OSC y protocolos de red que envía, captura, emula y degrada el tráfico de equipos de control de espectáculos, dispositivos y servicios, y lo convierte en pruebas repetibles.
hero:
  name: Signal Lab
  text: Un laboratorio de pruebas para OSC y protocolos de red
  tagline: Envía y observa el tráfico que hablan tus equipos y servicios, ocupa el lugar del dispositivo o la API que aún no existe, rompe la red a propósito y vuelve a ejecutar las mismas comprobaciones desde la aplicación, un script o CI.
  actions:
    - theme: brand
      text: Empezar
      link: /es/guide/
    - theme: alt
      text: Descargar
      link: https://github.com/ProAnima/SignalLab/releases/latest
features:
  - title: Todos los protocolos del rack
    details: OSC con argumentos con tipo, UDP y TCP sin formato, HTTP con autenticación Basic, Bearer y Digest, WebSocket y MQTT 3.1.1, además de difusión, multidifusión, barridos de subred y una escucha de descubrimiento.
    link: /es/protocols/osc
  - title: Un solo Inspector para todo
    details: Cada trama que las herramientas envían y reciben, en una sola línea de tiempo, decodificada y con sus bytes. Fíltrala, expórtala y reenvía byte a byte una trama capturada.
    link: /es/tools/inspector
  - title: Una biblioteca de señales
    details: Pon nombre al mensaje que funcionó, guárdalo en una carpeta y vuelve a enviarlo desde cualquier lugar con Ctrl+K, ya sea OSC, UDP sin formato, una solicitud HTTP o una publicación MQTT.
    link: /es/tools/signals
  - title: Emuladores
    details: Haz el papel de la otra parte (una API HTTP simulada, un dispositivo OSC, UDP o TCP, un bróker MQTT) con reglas, secuencias, retardos, fallos y caídas programadas.
    link: /es/tools/emulators
  - title: Degradación de red
    details: Un relé entre un cliente y su destino que añade latencia, jitter, pérdida, duplicados, reordenación o un límite de ancho de banda, o restablece conexiones TCP, todo a partir de una semilla, así que el mismo tráfico corre la misma suerte.
    link: /es/tools/impairment
  - title: Experimentos
    details: Flujos de prueba visuales (enviar, esperar la respuesta, comprobarla, bifurcar, repetir en bucle, ejecutar ramas en paralelo) con parámetros, perfiles, secretos y un informe de cada ejecución.
    link: /es/experiments/
  - title: Pruebas de carga
    details: Somete una solicitud HTTP a una tasa constante, una rampa, escalones, un pico o llegadas aleatorias, mide de p50 a p99, los errores y la tasa alcanzada, y haz fallar la ejecución según umbrales.
    link: /es/experiments/load
  - title: Automatización
    details: Ejecuta experimentos sin ventana con la línea de comandos signallab (códigos de salida, informes JUnit, una GitHub Action) o pásaselos a un asistente de IA mediante MCP.
    link: /es/automation/cli
  - title: Servidor y API
    details: La misma interfaz en un navegador y el mismo motor en un PC del laboratorio o en Docker, con inicio de sesión mediante un token, y una API HTTP para cada comando y cada ejecución.
    link: /es/server/
---

Signal Lab funciona como aplicación de escritorio en Windows y Linux, o como un servidor que abres
en un navegador. ¿Es la primera vez? Lee [qué es Signal Lab](guide/index.md), [instálalo](guide/install.md)
y da los [primeros pasos](guide/first-steps.md): un mensaje enviado y recibido, una señal
guardada, una API emulada que responde y un pequeño experimento ejecutado, todo en este equipo.

---
layout: home
title: Signal Lab
description: Signal Lab é um laboratório de testes para OSC e protocolos de rede que envia, captura, emula e degrada o tráfego de equipamentos de controle de show, dispositivos e serviços, e o transforma em testes repetíveis.
hero:
  name: Signal Lab
  text: Um laboratório de testes para OSC e protocolos de rede
  tagline: Envie e observe o tráfego que seus equipamentos e serviços falam, substitua o dispositivo ou a API que ainda não existe, quebre a rede de propósito — e depois execute as mesmas verificações de novo pelo aplicativo, por um script ou na CI.
  actions:
    - theme: brand
      text: Começar
      link: /pt/guide/
    - theme: alt
      text: Baixar
      link: https://github.com/ProAnima/SignalLab/releases/latest
features:
  - title: Todos os protocolos do rack
    details: OSC com argumentos tipados, UDP e TCP brutos, HTTP com autenticação Basic, Bearer e Digest, WebSocket e MQTT 3.1.1 — além de broadcast, multicast, varreduras de sub-rede e um ouvinte de descoberta.
    link: /pt/protocols/osc
  - title: Um só Inspetor para tudo
    details: Cada quadro que as ferramentas enviam e recebem, em uma única linha do tempo, decodificado e com seus bytes. Filtre, exporte e reproduza um quadro capturado byte a byte.
    link: /pt/tools/inspector
  - title: Uma biblioteca de sinais
    details: Dê nome à mensagem que funcionou, guarde-a em uma pasta e dispare-a de novo de qualquer lugar com Ctrl+K — OSC, UDP bruto, uma requisição HTTP ou uma publicação MQTT.
    link: /pt/tools/signals
  - title: Emuladores
    details: Faça o papel do outro lado — uma API HTTP simulada, um dispositivo OSC, UDP ou TCP, um broker MQTT — com regras, sequências, atrasos, falhas e quedas programadas.
    link: /pt/tools/emulators
  - title: Degradação de rede
    details: Um retransmissor entre um cliente e seu destino que acrescenta latência, jitter, perda, duplicatas, reordenação ou um limite de banda, ou redefine conexões TCP — com semente, para que o mesmo tráfego tenha sempre a mesma sorte.
    link: /pt/tools/impairment
  - title: Experimentos
    details: Fluxos de teste visuais — enviar, aguardar a resposta, verificá-la, desviar, repetir em laço, executar ramos em paralelo — com parâmetros, perfis, segredos e um relatório de cada execução.
    link: /pt/experiments/
  - title: Testes de carga
    details: Submeta uma requisição HTTP a uma taxa constante, uma rampa, degraus, um pico ou chegadas aleatórias, meça de p50 a p99, os erros e a taxa alcançada, e reprove a execução por limiares.
    link: /pt/experiments/load
  - title: Automação
    details: Execute experimentos sem interface com a linha de comando signallab — códigos de saída, relatórios JUnit, uma GitHub Action — ou entregue-os a um assistente de IA por MCP.
    link: /pt/automation/cli
  - title: Servidor e API
    details: A mesma interface em um navegador e o mesmo motor em um PC do laboratório ou no Docker, com acesso por token — e uma API HTTP para cada comando e cada execução.
    link: /pt/server/
---

O Signal Lab funciona como aplicativo de desktop no Windows e no Linux, ou como um servidor que você
abre no navegador. É novo por aqui? Leia [o que é o Signal Lab](guide/index.md), [instale-o](guide/install.md)
e depois siga os [primeiros passos](guide/first-steps.md): uma mensagem enviada e recebida, um sinal
salvo, uma API emulada respondendo e um pequeno experimento executado — tudo neste computador.

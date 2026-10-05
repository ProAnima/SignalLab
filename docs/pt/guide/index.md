---
title: O que é o Signal Lab
description: Para que serve o Signal Lab, o que você pode fazer com ele, as duas formas de executá-lo e como ele se organiza.
---

# O que é o Signal Lab

O Signal Lab é um laboratório de testes para OSC e protocolos de rede. Ele envia as mensagens que
seus equipamentos e serviços falam, mostra o que volta, substitui o dispositivo ou a API que ainda
não existe, degrada de propósito a rede entre eles e transforma tudo isso em experimentos que você
pode executar de novo — pelo aplicativo, por um script ou em um pipeline de CI.

Ele foi feito para colocar uma instalação no ar antes que a instalação exista: o controlador do
show, o servidor de mídia, os sensores e a API na nuvem podem ser testados, cada um, contra um
substituto, em um só notebook, muito antes de se encontrarem no local.

## Para quem é {#audience}

- **Quem coloca no ar controle de show, instalações e dispositivos em rede**: servidores de
  iluminação e de mídia, controladores, sensores, projetores, qualquer coisa que fale OSC, UDP,
  TCP ou MQTT.
- **Quem testa APIs e serviços**: endpoints HTTP e WebSocket, sua autenticação, seu comportamento
  sob carga e quando uma dependência falha.
- **Quem automatiza um ou outro**: os mesmos experimentos são executados sem interface em um
  pipeline, em um servidor de laboratório ao lado dos equipamentos ou conduzidos por um assistente
  de IA.

## O que você pode fazer {#what-you-can-do}

### Enviar e observar tráfego {#send-and-watch}

Cada protocolo tem uma tela própria:

| Protocolo | O que você pode fazer | Página |
| --- | --- | --- |
| OSC | Enviar mensagens com argumentos tipados, monitorar uma porta, levar uma forma de onda a um endpoint | [OSC](../protocols/osc.md) |
| UDP e TCP brutos | Enviar texto ou bytes em experimentos e a partir da biblioteca, emular dispositivos que falam esses protocolos | [UDP e TCP](../protocols/udp-tcp.md) |
| HTTP | Uma requisição com a resposta completa, autenticação Basic, Bearer ou Digest, um armazenamento de cookies, uma rajada de carga | [HTTP](../protocols/http.md) |
| WebSocket | Conectar com os cabeçalhos e subprotocolos que um serviço espera, enviar texto ou bytes, ler cada mensagem | [WebSocket](../protocols/websocket.md) |
| MQTT 3.1.1 | Conectar a um broker, observar todos os tópicos que ele guarda, publicar com QoS 0, 1 ou 2, limpar um valor retido | [MQTT](../protocols/mqtt.md) |
| Broadcast e multicast | Enviar para uma lista, um endereço de broadcast, um grupo multicast ou todos os hosts de uma sub-rede; escutar quem responde | [Broadcast e descoberta](../protocols/broadcast.md) |

### Ver cada quadro {#inspect}

O [Inspetor](../tools/inspector.md) registra cada quadro que as ferramentas enviam e recebem, em
uma única linha do tempo: decodificado, com seus bytes, filtrável e exportável. Um quadro que ele
capturou pode virar um sinal que o reproduz byte a byte.

### Guardar o que funciona {#library}

Uma mensagem que funcionou vai para a [biblioteca de sinais](../tools/signals.md): com nome,
organizada em pastas, com uma nota sobre o que ela deve provocar. Você a dispara de novo pela tela
dela, pela biblioteca ou de qualquer lugar com <kbd>Ctrl</kbd>+<kbd>K</kbd>.

### Fazer o papel do outro lado {#emulate}

Os [emuladores](../tools/emulators.md) respondem como a API, o dispositivo ou o serviço com que o
seu sistema conversa: uma API HTTP com rotas e respostas, um dispositivo OSC, UDP ou TCP com
regras, um broker MQTT. Eles podem responder devagar, falhar em sequência, enviar corpos malformados
ou sair do ar em horários programados — para que você teste como o seu sistema se vira antes que a
coisa real esteja lá.

### Quebrar a rede de propósito {#impair}

Um [retransmissor de degradação](../tools/impairment.md) fica entre um cliente e seu destino e
acrescenta latência, jitter, perda, duplicatas, reordenação ou um limite de banda ao UDP, ou atrasa,
estrangula, redefine e trava conexões TCP. Cada decisão segue uma semente, então o mesmo tráfego
tem a mesma sorte duas vezes.

### Transformar tudo em teste {#experiments}

Um [experimento](../experiments/index.md) é um fluxo de etapas que você desenha em um canvas:
enviar uma requisição, aguardar a resposta, verificá-la, extrair um valor, desviar,
repetir em laço, executar ramos em paralelo, iniciar um emulador ou um retransmissor de degradação
para a execução. Cada execução é relatada etapa por etapa e salva como um relatório que você pode
comparar com um anterior.

### Colocar carga em um serviço {#load}

A requisição HTTP de um experimento pode ser executada [sob carga](../experiments/load.md) — uma
taxa constante, uma rampa, degraus, um pico ou chegadas aleatórias —, medida e julgada por
limiares. A tela [HTTP](../protocols/http.md) tem uma rajada de carga rápida, e a
[Tempestade](../tools/storm.md) gera carga UDP ou TCP bruta contra os seus próprios servidores e
enlaces.

### Descobrir o que há na rede {#discover}

[Broadcast e descoberta](../protocols/broadcast.md) encontra dispositivos que respondem a uma
sonda, e o [Scanner](../tools/scanner.md) mostra quais portas TCP de um host estão abertas.

### Automatizar {#automate}

A [linha de comando](../automation/cli.md) `signallab` executa experimentos sem janela e termina
com um código que um pipeline entende; ela escreve relatórios JUnit para a [CI](../automation/ci.md).
`signallab mcp` permite que um [assistente de IA](../automation/mcp.md) leia, escreva e execute
experimentos. Um [servidor](../server/index.md) oferece o mesmo pela sua [API HTTP](../api/index.md).

## Duas formas de executá-lo {#two-ways}

| | Aplicativo de desktop | Servidor, no navegador |
| --- | --- | --- |
| Roda em | Windows 10 e 11 (x64), Linux (x86_64) | Linux como imagem Docker (amd64 e arm64), ou compilado a partir do código-fonte |
| Você usa | em uma janela própria | no Chrome, Edge ou Firefox, de qualquer máquina que alcance o servidor |
| O tráfego sai de | este computador | o servidor |
| Vem com | a linha de comando `signallab` | a linha de comando `signallab`, na imagem |
| Atualizações | ele mesmo, quando você mandar | com a imagem |

Os dois são a mesma interface sobre o mesmo motor: todas as telas, o Inspetor, os experimentos e os
relatórios funcionam do mesmo jeito. Um servidor é a escolha quando os equipamentos estão em uma
rede que o seu computador não alcança — um PC de rack ou um pequeno computador Linux ao lado deles,
usado de um notebook ou por um pipeline. Veja [Instalação e atualização](install.md) e
[O servidor](../server/index.md); o que muda entre os dois está em
[Conceitos](concepts.md#desktop-and-server).

## Como ele se organiza {#organisation}

- **Telas** são ferramentas para o trabalho que você faz agora, à mão: enviar isto, escutar ali,
  iniciar aquele emulador. A barra lateral as lista; cada uma guarda o que você digitou e o que
  recebeu enquanto você passa para outra. Veja [A janela](interface.md).
- **Experimentos** são fluxos que você monta uma vez e executa de novo, sempre do mesmo jeito, com
  um relatório para cada execução.
- **Bibliotecas** guardam o que você criou: sinais na biblioteca de sinais, emuladores na
  biblioteca de emuladores. Os experimentos usam as duas.
- **Tarefas** são o trabalho de longa duração — um monitor, um emulador, um retransmissor, uma
  execução —, listadas na parte de baixo da janela, onde você para uma delas ou todas.

[Conceitos](concepts.md) explica cada um deles com mais profundidade.

## Seguro por padrão {#defaults}

- O Signal Lab não envia nada até você apertar um botão, e só para onde você mandar. A única
  exceção é a verificação de atualizações do aplicativo de desktop, uma vez por dia, que você pode
  desativar (veja [Atualizações](install.md#updates)).
- Os sinais iniciais, os emuladores iniciais e os modelos de experimento apontam todos para
  `127.0.0.1`, este computador. Alcançar a rede é uma escolha que você faz ao digitar um endereço.
- Um servidor iniciado sem token de acesso escuta só em `127.0.0.1` e recusa qualquer outro
  endereço.
- O firewall só muda quando você clica para permitir.

## Uso responsável {#responsible-use}

::: danger Tráfego real para hosts reais
A [Tempestade](../tools/storm.md), o [Scanner](../tools/scanner.md) e o
[Broadcast](../protocols/broadcast.md) enviam tráfego real para hosts reais, e um broadcast ou uma
varredura de sub-rede alcança todos os dispositivos do segmento, não só aquele que você tinha em
mente. Aponte-os apenas para sistemas que são seus ou que você tem autorização para testar, e
confira antes em qual rede você está. Taxas altas podem saturar enlaces e disparar sistemas de
detecção de intrusão.
:::

O motor tem limites de proteção: uma varredura alcança no máximo 1.024 hosts, e um beacon envia no
máximo 50.000 pacotes por segundo somando todos os seus destinos. São limites de proteção, não
permissão.

## Idiomas {#languages}

A interface, suas dicas e suas mensagens de erro, a linha de comando `signallab`, a página de
entrada do servidor e o instalador do Windows falam 11 idiomas: English (inglês), Русский (russo),
Español (espanhol), Français (francês), Deutsch (alemão), Português (do Brasil), 中文 (chinês
simplificado), 日本語 (japonês), 한국어 (coreano), हिन्दी (híndi) e العربية (árabe, da direita para a
esquerda). Você troca o idioma ao vivo pelo cabeçalho; veja [A janela](interface.md#languages).

## Próximos passos {#next}

1. [Instale o Signal Lab](install.md).
2. Oriente-se pela [janela](interface.md).
3. Siga os [primeiros passos](first-steps.md) neste computador.
4. Leia os [conceitos](concepts.md) por trás dele.

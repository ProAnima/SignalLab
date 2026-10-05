---
title: A janela
description: Oriente-se pela janela do Signal Lab — as telas da barra lateral, o cabeçalho, o console e o Inspetor, a paleta de sinais, os painéis, as dicas e os idiomas.
---

# A janela

A janela do Signal Lab tem quatro partes: a **barra lateral**, à esquerda, lista as telas; o
**cabeçalho**, no alto, reúne o que vale em todo lugar; a **tela** que você escolheu ocupa o meio;
e o **painel inferior** mostra o console, as tarefas em execução e o Inspetor. O aplicativo de
desktop e a página de um servidor no navegador têm a mesma aparência; as poucas diferenças estão
em [No navegador](#browser).

## A barra lateral {#sidebar}

Cada tela é uma ferramenta própria. Clique em uma para abri-la:

| Tela | Para que serve |
| --- | --- |
| [[ui:nav.experiment]] | Montar fluxos de teste em um canvas, executá-los e ler cada execução etapa por etapa. [Experimentos](../experiments/index.md) |
| [[ui:nav.signals]] | A biblioteca de sinais: mensagens com nome, em pastas, para editar e enviar de novo. [Sinais](../tools/signals.md) |
| [[ui:nav.emulators]] | APIs HTTP, dispositivos OSC, UDP e TCP e brokers MQTT simulados, que respondem por regras. [Emuladores](../tools/emulators.md) |
| [[ui:nav.osc]] | Enviar mensagens OSC, monitorar uma porta, levar uma forma de onda a um endpoint. [OSC](../protocols/osc.md) |
| [[ui:nav.mqtt]] | Conectar a um broker, ver todos os tópicos que ele guarda, publicar e limpar valores retidos. [MQTT](../protocols/mqtt.md) |
| [[ui:nav.broadcast]] | Enviar para muitos hosts ao mesmo tempo — uma lista, broadcast, multicast, uma varredura de sub-rede — e escutar quem responde. [Broadcast e descoberta](../protocols/broadcast.md) |
| [[ui:nav.http]] | Uma requisição e a resposta inteira, depois uma rajada de carga contra o mesmo endpoint. [HTTP](../protocols/http.md) |
| [[ui:nav.ws]] | Conectar a um serviço WebSocket, enviar texto ou bytes, ler cada mensagem. [WebSocket](../protocols/websocket.md) |
| [[ui:nav.netsim]] | Um retransmissor que degrada o tráfego UDP ou TCP entre um cliente e seu destino. [Degradação](../tools/impairment.md) |
| [[ui:nav.storm]] | Carga UDP ou TCP bruta contra os seus próprios servidores e enlaces. [Tempestade](../tools/storm.md) |
| [[ui:nav.scan]] | Quais portas TCP de um host estão abertas, com o que o serviço diz primeiro. [Scanner](../tools/scanner.md) |

A barra lateral começa como um trilho estreito com o símbolo e um nome curto de cada tela; passe o
ponteiro sobre um para ver o nome completo. O **☰** à esquerda do cabeçalho
([[ui:app.expandNav]] / [[ui:app.collapseNav]]) alterna entre o trilho e a lista completa, e o
Signal Lab lembra qual você escolheu.

Um número na entrada de uma tela conta as tarefas em execução a partir dela, como um monitor ou um
emulador, para que você veja o que ainda está rodando sem abri-la. No pé da barra lateral, a
versão abre a janela Sobre, e a linha abaixo dela conta todas as tarefas em execução.

O Signal Lab abre na última tela que você usou.

## O cabeçalho {#header}

Da esquerda para a direita:

| Item | O que faz |
| --- | --- |
| **☰** | Mostra a barra lateral como trilho ou como lista completa. |
| O host | [[ui:app.host]] e depois o nome e o endereço de rede do computador em que o motor roda: este, no aplicativo de desktop; o servidor, no navegador. No navegador também aparece [[ui:app.server]] — passe o ponteiro sobre ele para ver onde o servidor guarda seus arquivos —, e o ponto ao lado muda quando a página perde a conexão. |
| O botão de atualização | Aparece no aplicativo de desktop quando uma versão mais nova foi encontrada e abre a janela Sobre para instalá-la. Veja [Atualizações](install.md#updates). |
| O livro | [[ui:app.docs]]: esta documentação, na página da tela em que você está. <kbd>F1</kbd> faz o mesmo de qualquer lugar. |
| **✉** | [[ui:feedback.open]]: uma mensagem para os desenvolvedores. Veja [Escrever aos desenvolvedores](#feedback). |
| **?** | [[ui:about.open]]: a versão, quem faz o Signal Lab, como falar com eles e as atualizações. |
| A bandeira | [[ui:app.language]]: o idioma da interface. Veja [Idiomas](#languages). |
| [[ui:app.signOut]] | No navegador, quando o servidor pede um token de acesso: encerra a sessão deste navegador. |
| [[ui:app.stopAll]] | Para de uma vez todas as tarefas em execução: monitores, geradores, beacons, emuladores, retransmissores, tempestades, varreduras, execuções. Fica acinzentado enquanto nada está rodando. |

### A documentação {#docs}

A documentação vem embutida no aplicativo e no servidor, então está disponível sem conexão com a
internet, no idioma da interface. O aplicativo de desktop a mostra em uma janela própria —
pressionar o botão de novo a partir de outra tela leva essa janela à página da outra tela — e
envia ao seu navegador os links que saem da documentação. No navegador, ela abre em uma aba
própria. A janela Sobre também tem um botão [[ui:app.docs]], que abre
[O que é o Signal Lab](index.md).

### Sobre {#about}

A janela Sobre mostra a versão, o desenvolvedor, o endereço de contato (com um botão que o copia),
o código-fonte e a licença. No aplicativo de desktop, a seção [[ui:update.title]] dela procura,
mostra e instala atualizações — veja [Atualizações](install.md#updates). No navegador, ela diz
[[ui:update.server]]. O botão [[ui:about.writeUs]] abre o formulário de comentários.

### Escrever aos desenvolvedores {#feedback}

O **✉** do cabeçalho abre um formulário que vai direto para os desenvolvedores:

| Campo | O que colocar ali |
| --- | --- |
| [[ui:feedback.message]] | O que aconteceu e o que você esperava no lugar. Obrigatório; no máximo 20.000 caracteres. |
| [[ui:feedback.email]] | Opcional: onde os desenvolvedores podem responder a você. Só eles o veem. |
| [[ui:feedback.screenshots]] | Até 6 imagens (PNG, JPEG, WebP ou GIF), 8 MB cada e 15 MB ao todo. Cole uma com <kbd>Ctrl</kbd>+<kbd>V</kbd>, solte arquivos na janela ou pressione [[ui:feedback.addScreenshot]]. |
| [[ui:feedback.logs]] | As linhas do console e [[ui:feedback.systemInfo]], cada um anexado como arquivo próprio. O botão [[ui:feedback.show]] mostra exatamente o que é enviado; desmarque qualquer um deles para deixá-lo de fora. |

Os logs anexados deixam de fora o nome deste computador, o endereço de rede dele e os nomes nos
caminhos das suas pastas. <kbd>Ctrl</kbd>+<kbd>Enter</kbd> envia o formulário; quando ele sai, você
recebe um número de referência, que o console também guarda. A mensagem passa pelo serviço do
próprio estúdio, que a encaminha por e-mail; o aplicativo não guarda senha nenhuma para isso.

## O painel inferior {#bottom-panel}

O painel abaixo de cada tela tem duas abas, [[ui:console.title]] e [[ui:dock.inspector]], e, entre
elas e os botões do painel, a faixa das tarefas em execução.

- A **seta** à esquerda dele ([[ui:console.collapse]] / [[ui:console.expand]]) recolhe o painel
  até a barra dele ou o abre de novo. Recolhida, a barra continua mostrando a linha mais recente do
  console; clique nessa linha para abrir o painel.
- Arraste a borda superior do painel para deixá-lo mais alto ou mais baixo (veja
  [Redimensionar painéis](#panes)).
- O botão à direita dele o deixa [[ui:dock.maximise]] e o traz de volta
  ([[ui:dock.restore]]).

Se o painel está aberto, qual aba ele mostra e qual a altura dele ficam guardados para a próxima
vez.

### O console {#console}

O console diz o que cada ferramenta fez e o que deu errado, com o mais recente por último: uma
mensagem enviada e o tamanho dela, um monitor iniciado, o status e o tempo de uma resposta, uma
tarefa que terminou e por quê. Cada linha tem a hora (até o milissegundo), uma etiqueta com o nome
da ferramenta e a mensagem, colorida conforme o que ela é — concluído, informação, aviso ou erro.

- A opção [[ui:console.autoscroll]] mantém a linha mais recente à vista à medida que as linhas chegam;
  desmarque para ler o que veio antes enquanto chegam mais.
- O botão [[ui:common.clear]] o esvazia.
- Ele guarda as últimas 500 linhas.
- Trocar o idioma reescreve o console inteiro no novo idioma.

### Tarefas {#jobs-strip}

Cada tarefa em execução — um monitor, um gerador, um beacon, uma conexão com um broker, um
emulador, um retransmissor, uma tempestade, uma varredura, a execução de um experimento — tem uma
pílula na faixa com o número dela e o que ela é, e um botão próprio para pará-la. Sem nada
rodando, a faixa diz [[ui:console.empty]]. Veja [Tarefas](concepts.md#jobs).

### A aba Inspetor {#inspector-tab}

A aba [[ui:dock.inspector]] mostra cada quadro que as ferramentas enviam e recebem enquanto a
captura está ativa, ao lado da tela em que você estiver trabalhando. O ponto dela acende enquanto
a captura está ativa, e um número conta os quadros capturados. O painel abre alto o suficiente
para a lista do Inspetor e os detalhes de um quadro; um link para um quadro em outro lugar — na
linha do tempo de uma execução ou na lista de um emulador — abre esta aba nesse quadro. Depois de
aberto, o Inspetor mantém a lista e a seleção enquanto o painel está fechado. Como usá-lo:
[Inspetor](../tools/inspector.md).

## Enviar um sinal de qualquer lugar {#palette}

Pressione <kbd>Ctrl</kbd>+<kbd>K</kbd> em qualquer tela para abrir a paleta [[ui:sig.paletteTitle]]: digite
algumas letras do nome, da pasta ou do destino de um sinal, escolha com <kbd>↑</kbd> e
<kbd>↓</kbd> e pressione <kbd>Enter</kbd> para enviá-lo. A paleta lista até 12 sinais por vez.
<kbd>Esc</kbd>, um clique fora dela ou <kbd>Ctrl</kbd>+<kbd>K</kbd> de novo a fecham. O console
diz o que foi enviado e para onde. Veja [Sinais](../tools/signals.md).

## Redimensionar painéis {#panes}

Uma alça fina fica entre os painéis que você pode redimensionar: a borda superior do painel
inferior ([[ui:layout.console]]) e, na tela de experimentos, a borda das propriedades
([[ui:layout.properties]]) e o topo da linha do tempo da execução ([[ui:layout.timeline]]).

- Arraste a alça.
- Ou dê foco a ela com <kbd>Tab</kbd> e use as setas: cada toque a move 16 pixels, quatro vezes
  mais com <kbd>Shift</kbd>; <kbd>Home</kbd> e <kbd>End</kbd> levam ao menor e ao maior tamanho.
- Clique duas vezes nela, ou pressione <kbd>Enter</kbd> sobre ela, para devolver ao painel o
  tamanho padrão.

Os tamanhos ficam guardados para a próxima vez.

## Dicas {#tooltips}

As telas mostram rótulos, valores e estados, e guardam as explicações em dicas: o que um campo
espera, o que `0` significa ali, qual tecla faz o mesmo. Uma dica aparece quando você deixa o
ponteiro sobre algo por cerca de meio segundo, e imediatamente quando chega a ele pelo teclado; um
campo mostra a dica do seu rótulo. <kbd>Esc</kbd>, digitar, um clique ou a rolagem a escondem. Os
leitores de tela leem o mesmo texto.

As mensagens de erro dizem onde e o que deu errado, e por quê; o texto do próprio sistema fica
recolhido em [[ui:err.details]].

## As telas guardam o estado {#state}

Uma tela abre na primeira vez que você a visita e depois permanece como está enquanto você
trabalha em outra: o que você digitou, a última resposta, a lista de mensagens de um monitor e a
tarefa por trás dela, até a posição de rolagem — tudo está lá quando você volta. Uma tarefa em
execução continua, seja qual for a tela que você estiver olhando.

Alguns valores também são guardados entre reinícios: a mensagem OSC que você estava enviando, a
requisição HTTP, o endereço WebSocket, os tamanhos dos painéis, a tela em que você estava.

## O aviso do firewall {#firewall-notice}

O Firewall do Windows Defender decide, programa por programa, se outras máquinas podem alcançá-lo.
Na primeira vez que um programa escuta, o Windows pergunta a quem está diante da tela — e um
*Cancelar* ali, ou uma rede que o Windows considera pública, descarta em silêncio tudo o que outras
máquinas enviarem. Um monitor que não mostra nada é o sinal de costume.

Por isso, no aplicativo de desktop no Windows, na primeira vez que algo começa a escutar — um
monitor OSC, o ouvinte de descoberta, um retransmissor de degradação, um emulador ou a execução de
um experimento —, o Signal Lab consulta o firewall uma vez. Quando o firewall está no caminho, um
aviso abaixo do cabeçalho diz isso:

- O botão [[ui:fw.allow]] pede direitos de administrador com o próprio aviso do Windows e depois deixa
  outras máquinas alcançarem o Signal Lab em redes privadas e de domínio.
- Em uma rede que o Windows chama de pública — o Wi-Fi de um local de evento, muitas vezes —, o
  botão passa a ser [[ui:fw.allowPublic]].
- O botão [[ui:fw.dismiss]] esconde o aviso nesta sessão.

Permitir substitui as regras de entrada do firewall do Signal Lab por uma única regra que
permite. O tráfego dentro deste computador (`127.0.0.1`) nunca é afetado, então você pode ignorar
o aviso enquanto trabalha em loopback. Uma instalação para todos já tem a regra; veja
[O que o instalador acrescenta](install.md#windows-setup-adds). `signallab doctor` informa o mesmo
em um terminal, e `signallab firewall allow` resolve ali — veja
[A linha de comando](../automation/cli.md). Não há aviso no Linux nem no navegador: o firewall de
um servidor é assunto do administrador dele.

## No navegador {#browser}

A página de um [servidor](../server/index.md) é a mesma interface, com algumas diferenças:

- O host no cabeçalho dá o nome do servidor, com [[ui:app.server]] ao lado.
- Se o servidor pede um token de acesso, você entra uma vez, e [[ui:app.signOut]] no cabeçalho
  encerra a sessão.
- Se a página perde a conexão com o servidor, uma barra diz [[ui:app.connectionLost]] até que ela
  volte; o console registra as duas coisas.
- Relatórios de execução, exportações e capturas do Inspetor são baixados pelo navegador em vez de
  aparecerem como um caminho.
- A janela Sobre não tem atualizações: o servidor é atualizado com a sua imagem.

Tudo o que a página de um servidor faz acontece no servidor: o tráfego sai dele, os monitores
escutam nas portas dele, os arquivos vão para a pasta de dados dele. Veja
[Conceitos](concepts.md#desktop-and-server).

## Idiomas {#languages}

A bandeira no cabeçalho mostra o idioma atual e as duas letras dele. Clique nela para abrir a
lista de todos os idiomas, cada um com a sua bandeira e o seu próprio nome, e escolha um. Na lista,
<kbd>↑</kbd>, <kbd>↓</kbd>, <kbd>Home</kbd> e <kbd>End</kbd> movem a seleção, uma letra salta para o
próximo idioma que começa com ela — no nome próprio dele ou em inglês, então <kbd>g</kbd> encontra
Deutsch —, <kbd>Enter</kbd> escolhe e <kbd>Esc</kbd> fecha a lista.

A interface muda na hora, sem reiniciar: todas as telas, dicas e erros, e também as linhas
anteriores do console. Na primeira vez que o Signal Lab inicia, ele escolhe o primeiro dos idiomas
do seu sistema que ele tem, ou o inglês, e daí em diante mantém a sua escolha — no navegador, para
aquele navegador.

O árabe vira a janela inteira da direita para a esquerda. O que é dado continua da esquerda para a
direita, como está escrito: endereços, dumps hexadecimais, código e o canvas do experimento.

---
title: O editor
description: Monte, edite e execute experimentos no canvas de nós — adicionar e ligar nós, selecionar e copiar, o painel de propriedades, Enviar agora, validação, execução e os arquivos por trás disso.
---

# O editor de experimentos

Um experimento é um teste escrito como um grafo: nós que enviam (uma requisição HTTP,
uma mensagem OSC, uma publicação MQTT…), aguardam uma resposta, verificam o que voltou,
fazem o papel do outro lado ou quebram a rede na hora certa, unidos por fios que dizem
o que executa em seguida. Uma execução começa em [[ui:exp.node.start]], segue os fios e é aprovada quando
chega a [[ui:exp.node.end]] com todas as etapas aprovadas. Você monta o experimento na tela [[ui:nav.experiment]],
executa por lá e executa o mesmo arquivo pela linha de comando ou em um servidor.

O editor guarda um experimento por vez e o salva enquanto você trabalha. Para manter
vários, exporte instantâneos ou abra outro a partir de um arquivo (veja
[Salvamento e arquivos](#files)).

Cada tipo de nó, com seus campos e saídas, está na [referência dos nós](nodes.md).
Como os valores passam de um nó a outro está em [Dados e modelos](data.md); ramos
paralelos, laços, novas tentativas e repetições estão em [Fluxo](flow.md).

## A tela {#screen}

| Área | O que contém |
| --- | --- |
| Barra de ferramentas (no alto) | O nome do experimento e o estado de salvamento, a adição de nós, parâmetros, perfis, os painéis, o modo foco, a tela cheia e Executar |
| Barra do canvas | Desfazer e refazer, o localizador de nós, [[ui:exp.arrange]] e o zoom |
| Canvas | O grafo: nós, fios e o progresso da execução desenhado sobre eles |
| [[ui:exp.properties]] (à direita) | Os campos do nó selecionado, a prévia dele e [[ui:exp.sendNow]]; ou o que vários nós selecionados ou um fio selecionado permitem |
| [[ui:exp.timeline]] (embaixo) | As etapas da execução atual ou da última, o resultado, o relatório e a semente |

O painel de propriedades e a linha do tempo têm uma alça na borda interna: arraste-a,
ou foque nela com <kbd>Tab</kbd> e use as setas (<kbd>Shift</kbd>
para passos maiores); um clique duplo ou <kbd>Enter</kbd> devolve ao painel o
tamanho padrão. Os tamanhos são mantidos para a próxima vez. A
linha do tempo fica recolhida até a primeira execução abri-la.

## A barra de ferramentas {#toolbar}

| Controle | O que faz |
| --- | --- |
| ☰ [[ui:exp.documents]] | Abre os modelos, a abertura de um arquivo JSON e a exportação ([Salvamento e arquivos](#files)) |
| [[ui:exp.name]] | O nome do experimento; uma execução exige um. Os relatórios de execução o registram, e [[ui:exp.compare]] encontra execuções anteriores por ele |
| [[ui:exp.saved]] / [[ui:exp.saving]] / [[ui:exp.saveError]] | Se a última alteração está no disco |
| ⚠ [[ui:exp.needsLinks]] | Aparece enquanto o experimento não pode executar; a dica explica por quê, e um clique seleciona o nó em questão ([Validação](#validation)) |
| ＋ [[ui:exp.addNode]] | Abre o menu de adição, depois do nó selecionado quando há um (<kbd>A</kbd>) |
| [[ui:exp.profile]] | Qual perfil a execução, a prévia e Enviar agora usam; aparece quando o experimento tem perfis. ⚠ marca um perfil que não executaria |
| `{ }` [[ui:exp.params]] | Parâmetros, perfis, a semente, cookies e segredos ([Dados e modelos](data.md)) |
| ☷ [[ui:exp.properties]] | Mostra ou oculta o painel de propriedades |
| ▢ [[ui:exp.focus]] | Oculta a barra lateral, o cabeçalho e o painel inferior ([Modo foco e tela cheia](#focus)) |
| ⛶ [[ui:exp.fullscreen]] | A janela em tela cheia, com o modo foco |
| [[ui:exp.run]] | Verifica, salva e executa o experimento; durante a execução, o botão passa a ser [[ui:common.stop]] |
| ▾ [[ui:exp.runWith]] | Uma execução com outro perfil, outros valores de parâmetro ou uma semente dada, sem alterar o experimento |

## Movimento no canvas {#canvas}

- **Deslocar**: arraste o canvas vazio, ou role.
- **Zoom**: <kbd>Ctrl</kbd> e a roda do mouse ampliam em torno do ponteiro;
  − e ＋ na barra do canvas dão passos de 10 %. O zoom vai de 15 % a 200 %; a
  porcentagem entre os botões mostra onde você está.
- **1:1** ([[ui:exp.resetZoom]], <kbd>Ctrl</kbd>+<kbd>1</kbd>) volta a 100 %.
- ⊡ [[ui:exp.fit]] (<kbd>Ctrl</kbd>+<kbd>0</kbd>) mostra o grafo inteiro, a
  100 % no máximo.
- **Localizar um nó**: [[ui:exp.nodes]] na barra do canvas (mostra quantos nós
  há) ou <kbd>Ctrl</kbd>+<kbd>F</kbd>. Digite palavras do nome do nó,
  do resumo dele (uma URL, um endereço, um tópico) ou do id; <kbd>↑</kbd>
  <kbd>↓</kbd> escolhem, <kbd>Enter</kbd> seleciona o nó e o traz à
  vista (com zoom de pelo menos 80 %), <kbd>Esc</kbd> fecha.

Não há minimapa: [[ui:exp.fit]] e o localizador fazem o trabalho dele.

## Nós e fios {#nodes-and-wires}

Um nó mostra o tipo, um resumo de uma linha do que ele faz e distintivos para suas
configurações: ↻ e um número para Nova tentativa, × e uma contagem (ou um tempo) para Repetição, ⚡
para Carga. A entrada fica à esquerda — todo nó, exceto [[ui:exp.node.start]], tem uma — e as
saídas à direita:

| Saída | Em | Seguida quando |
| --- | --- | --- |
| [[ui:exp.outputPort]] | A maioria dos nós | A etapa foi aprovada |
| [[ui:exp.yes]] / [[ui:exp.no]] | [[ui:exp.node.branch_status]], [[ui:exp.node.branch_value]] | A comparação valeu / não valeu |
| [[ui:exp.branch1]] / [[ui:exp.branch2]] | [[ui:exp.node.fork]] | Sempre, os dois ao mesmo tempo |
| [[ui:exp.portMatched]] / [[ui:exp.portTimeout]] | Esperas | Chegou uma mensagem correspondente / nenhuma chegou a tempo |
| [[ui:exp.portBody]] / [[ui:exp.portDone]] / [[ui:exp.portLimit]] | [[ui:exp.node.loop]] | Outra iteração / o laço terminou / as iterações acabaram |

**Uma saída pode ter vários fios.** O primeiro fio continua o ramo;
cada fio adicional inicia um ramo paralelo com uma cópia das variáveis conhecidas
naquele ponto. Um nó [[[ui:exp.node.join]]](nodes.md#node-join) aguarda todos os fios que
chegam a ele. Os detalhes estão em [Fluxo](flow.md).

[[ui:exp.portTimeout]] e [[ui:exp.portLimit]] são opcionais: sem fio, um
tempo esgotado ou o fim das iterações reprova a etapa. Toda outra saída precisa
ter um fio antes que o experimento possa executar.

O fio do corpo de um [[ui:exp.node.loop]] de volta a ele é desenhado como um arco
sobre o corpo; é o único fio que pode voltar para trás.

## Adição de nós {#adding}

### O menu de adição {#add-menu}

O menu de adição lista todos os tipos de nó por grupo — [[ui:exp.group.action]],
[[ui:exp.group.observe]], [[ui:exp.group.emulate]], [[ui:exp.group.fault]],
[[ui:exp.group.data]], [[ui:exp.group.check]], [[ui:exp.group.flow]] — e
depois [[ui:exp.group.signals]]: os sinais da sua [biblioteca](../tools/signals.md)
que podem virar um nó (OSC, HTTP, UDP com carga de texto, MQTT), com os campos
já preenchidos.

O campo de busca recebe o foco ao abrir. Digite algumas letras: cada palavra
deve aparecer no nome do nó, na descrição ou no tipo (`http`,
`wait_osc`); um sinal é encontrado pelo nome, pela pasta, pelo transporte ou pelo destino.
<kbd>↑</kbd> <kbd>↓</kbd> escolhem, <kbd>Enter</kbd> adiciona, <kbd>Esc</kbd>
fecha. A linha no alto diz onde o nó vai: depois de um nó, ou em
paralelo a um.

Um nó novo fica selecionado, o painel de propriedades abre e o campo principal dele — a
URL, o endereço, o tópico, o atraso — recebe o foco com o texto
selecionado, para você digitar de imediato. <kbd>Esc</kbd> em um campo leva de
volta ao nó no canvas, pronto para o próximo <kbd>A</kbd>.

### Depois do nó selecionado {#add-after}

<kbd>A</kbd>, ＋ [[ui:exp.addNode]] na barra de ferramentas e ＋ [[ui:exp.addNext]]
no painel de propriedades adicionam a próxima etapa depois do nó selecionado:

- O novo nó se liga à primeira saída do nó que ainda não tem fio (o
  [[ui:exp.no]] livre de um [[ui:exp.node.branch_status]], por exemplo) ou, na falta dela, à primeira
  saída.
- Se essa saída já tem um fio, o novo nó é inserido nele (no
  primeiro, se houver vários): o fio passa a atravessar o novo nó, e
  tudo o que vem depois se desloca para a direita para dar espaço.
- Com [[ui:exp.node.end]] selecionado, o nó entra antes dele, quando um fio
  chega a ele.
- Sem nada selecionado, o nó cai no meio da vista, sem
  fios.

Um nó com uma única saída colocado no [[ui:exp.portBody]] vazio de um
[[ui:exp.node.loop]] — assim ou [em um fio novo](#add-branch) — também é ligado
de volta a ele, de modo que o corpo fica completo de imediato.

### Em um fio {#insert}

Todo fio tem um ＋ no meio ([[ui:exp.insertNode]]). Ele abre o menu de adição,
e o nó escolhido é inserido nesse fio. Um [[ui:exp.node.loop]] inserido
assim continua o fluxo por [[ui:exp.portDone]].

### Em um fio novo {#add-branch}

Arraste um fio para fora de uma saída e solte no canvas vazio: o menu de adição
abre ali, e o novo nó vai em um fio novo dessa saída — ao lado dos
fios que ela já tem, de modo que executa em paralelo com eles. O mesmo acontece quando
você clica em uma saída e depois no canvas vazio, ou clica duas vezes nela.

### Em qualquer lugar {#add-anywhere}

Clique duas vezes no canvas vazio para adicionar um nó naquele ponto, sem fios.

### A partir de outras telas {#from-screens}

- [[ui:common.toExperiment]] nas telas [OSC](../protocols/osc.md) e
  [HTTP](../protocols/http.md) adiciona o que você acabou de testar como a próxima
  etapa — logo antes de [[ui:exp.node.end]], quando um fio chega a ele — e
  passa para o editor.
- [[ui:osc.waitForThis]] ao lado de uma mensagem no monitor OSC adiciona um
  [[ui:exp.node.wait_osc]] que a reconhece: o endereço e o texto dela, os argumentos inteiros e
  verdadeiro/falso (os de ponto flutuante são medições que mudam, então ficam
  de fora). Pare o monitor antes de executar: a execução escuta essa porta por conta própria.
- [[ui:mq.waitForThis]] em um tópico da árvore [MQTT](../protocols/mqtt.md)
  adiciona um [[ui:exp.node.wait_mqtt]] nesse tópico, nesse broker.

Enquanto uma execução está em andamento, não é possível adicionar nós; o console avisa.

## Ligação de nós {#connecting}

- **Arraste** de uma saída até um nó. Soltar a até 24 pixels de um nó
  basta.
- **Clique** em uma saída (ou foque nela e pressione <kbd>Enter</kbd> ou
  <kbd>Space</kbd>): a barra do canvas diz [[ui:exp.chooseInput]]. Clique em um nó
  ou na entrada dele para ligar; clique no canvas vazio para adicionar um nó ali em um fio
  novo; [[ui:exp.cancelLink]] ou <kbd>Esc</kbd> desiste.

Um fio é recusado quando criaria um ciclo (que não seja o retorno de um
[[ui:exp.node.loop]]), quando chega a [[ui:exp.node.start]] ou sai de
[[ui:exp.node.end]], ou quando ligaria um nó a ele mesmo; o editor avisa.
Desenhar um fio que já existe não muda nada.

Para **remover um fio**, clique nele — o painel de propriedades mostra o que ele liga —
e pressione <kbd>Delete</kbd>, ou use [[ui:exp.removeWire]] ali. Passar o ponteiro sobre um
fio também mostra um × acima do ＋. As propriedades de um nó listam os fios
de saída, cada um com × ([[ui:exp.disconnect]]).

## Seleção de vários nós {#selection}

| Para | Faça |
| --- | --- |
| Selecionar um nó | Clique nele, ou vá até ele com <kbd>Tab</kbd> |
| Incluir um nó na seleção, ou tirá-lo | <kbd>Shift</kbd> ou <kbd>Ctrl</kbd> e um clique |
| Selecionar com um quadro | <kbd>Shift</kbd> e arraste no canvas vazio: todo nó que o quadro tocar entra na seleção |
| Selecionar todos os nós | <kbd>Ctrl</kbd>+<kbd>A</kbd> |
| Limpar a seleção | Clique no canvas vazio; <kbd>Esc</kbd> quando vários estão selecionados |
| Movê-los | Arraste um deles: todos se movem |
| Deslocá-los de leve | Com um nó em foco, as setas movem a seleção 5 pixels, 20 com <kbd>Shift</kbd> |

O painel de propriedades mostra o último nó escolhido; com vários selecionados,
mostra quantos são, com [[ui:exp.copy]], [[ui:exp.duplicate]] e
[[ui:exp.deleteSelected]].

### Copiar, recortar e colar {#copy-paste}

<kbd>Ctrl</kbd>+<kbd>C</kbd> copia os nós selecionados e os fios entre
eles como texto; <kbd>Ctrl</kbd>+<kbd>X</kbd> também os remove;
<kbd>Ctrl</kbd>+<kbd>V</kbd> os cola — neste experimento, em outro
aberto depois, ou em outra janela do Signal Lab. O texto é JSON, então
você também pode guardá-lo em um arquivo ou em uma mensagem.

- [[ui:exp.node.start]] e [[ui:exp.node.end]] são únicos: nunca são
  copiados.
- Os fios entre um nó copiado e o resto do grafo não são copiados; ligue
  a cópia você mesmo.
- Cada nó colado recebe um id novo.
- Um nó que cita outro — [[ui:exp.node.impairment_change]],
  [[ui:exp.node.emulator_state]], [[ui:exp.node.ws_send]],
  [[ui:exp.node.wait_ws]], [[ui:exp.node.ws_close]] — passa a citar a cópia quando ela também foi copiada;
  caso contrário continua citando o original, se ele estiver neste experimento, ou o
  primeiro nó desse tipo daqui.
- Um [[ui:exp.node.emulator]] ou [[ui:exp.node.impairment]] colado, cuja porta
  este experimento já escuta, passa para a próxima porta livre. O que envia para
  o original continua enviando.
- A cópia cai 32 pixels à direita e uma linha abaixo de onde veio,
  mais para baixo até não cobrir nenhum nó, e fica selecionada.

<kbd>Ctrl</kbd>+<kbd>D</kbd> ([[ui:exp.duplicate]]) faz o mesmo sem a
área de transferência.

### Exclusão {#deleting}

<kbd>Delete</kbd> ou <kbd>Backspace</kbd> ([[ui:exp.delete]],
[[ui:exp.deleteSelected]]) remove os nós selecionados e seus fios. Um nó
que tinha exatamente um fio de entrada e um de saída deixa um fio no lugar, do
nó anterior ao nó seguinte, para que a cadeia continue ligada. [[ui:exp.node.start]]
e [[ui:exp.node.end]] não podem ser excluídos.

## Desfazer e refazer {#undo}

<kbd>Ctrl</kbd>+<kbd>Z</kbd> desfaz; <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd>
ou <kbd>Ctrl</kbd>+<kbd>Y</kbd> refaz (↶ ↷ na barra do canvas). Um arrasto, uma
série de deslocamentos pelas setas ou a digitação em um campo é um passo. Os últimos
100 passos são mantidos enquanto o app está aberto, entre trocas de tela; abrir
outro experimento também é um passo, então <kbd>Ctrl</kbd>+<kbd>Z</kbd> traz
o anterior de volta. Desfazer e refazer ficam em espera enquanto uma execução está em andamento.

## Organizar {#arrange}

[[ui:exp.arrange]] dispõe o grafo da esquerda para a direita: cada nó na
coluna seguinte à do último nó que leva a ele, o corpo de um [[ui:exp.node.loop]]
na linha dele — e depois enquadra a vista. É um passo do histórico. Um rascunho
com um ciclo que não é de um laço fica como está.

## O painel de propriedades {#properties}

Com um nó selecionado, o painel mostra, de cima para baixo:

1. O nome do nó (a dica diz o que ele faz) e, quando o experimento
   não pode executar por causa desse nó, o que está errado.
2. Os campos dele. Os campos que aceitam [modelos](data.md) sugerem parâmetros,
   variáveis, segredos e geradores conforme você digita `{{`, ou com
   <kbd>Ctrl</kbd>+<kbd>Space</kbd>.
3. As configurações dele: [[ui:exp.loadOn]] em uma requisição HTTP, [[ui:exp.repeatOn]] nos
   nós que enviam, [[ui:exp.retryOn]] nos nós que enviam ou escutam e
   [[ui:exp.expectReply]] em mensagens OSC e UDP (veja [Configurações do
   nó](nodes.md#settings)). A carga substitui Repetição e Nova tentativa enquanto está ativa.
4. O resultado de carga da última execução, em um nó HTTP que executou sob carga.
5. A prévia — [[ui:exp.preview]], [[ui:exp.previewWait]] ou
   [[ui:exp.previewCheck]] — quando o nó tem modelos: o que ele enviaria,
   aguardaria ou compararia, resolvido com os parâmetros atuais e os valores
   conhecidos até agora ([Enviar agora e a prévia](#send-now)).
6. [[ui:exp.sendNow]] ou [[ui:exp.listenNow]], e o que fez da última vez.
7. Os fios de saída dele, cada um com ×.
8. ⚡ [[ui:exp.routeThrough]] em uma mensagem OSC ou UDP: um [[ui:exp.node.impairment]] é posto
   na frente do nó — escutando em uma porta de loopback livre a partir de 9010,
   encaminhando para o destino do nó — e o nó é apontado para ele, de modo que a
   próxima execução degrada o que ele envia ([Falhas](faults.md)).
9. ＋ [[ui:exp.addNext]], [[ui:exp.copy]], [[ui:exp.duplicate]] e
   [[ui:exp.delete]].

Um clique duplo em um nó abre o painel com o cursor no campo principal dele.
Com vários nós selecionados, o painel oferece o que pode ser feito com todos;
com um fio selecionado, o que ele liga e [[ui:exp.removeWire]]. Enquanto uma execução
está em andamento, os campos ficam travados.

## Enviar agora e a prévia {#send-now}

[[ui:exp.sendNow]] (<kbd>Ctrl</kbd>+<kbd>Enter</kbd>, também de dentro dos
campos do nó) envia o nó selecionado sozinho, sem executar o
experimento, pelo mesmo código que uma execução usa. Está disponível em
[[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]],
[[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_connect]] e
[[ui:exp.node.ws_send]].
Em uma espera, é [[ui:exp.listenNow]]: a espera escuta a partir de agora até que uma mensagem
corresponda ou o tempo limite dela acabe.

- O nó usa o perfil ativo, os segredos guardados e os valores de variável
  conhecidos até agora — da última execução e de resultados anteriores de [[ui:exp.sendNow]].
  Se um modelo cita um valor que ninguém definiu ainda, nada é enviado e os nomes
  são listados.
- Ele é enviado uma vez: Nova tentativa, Repetição e Carga não se aplicam, nenhum cookie é mantido,
  e os emuladores e retransmissores do experimento não são iniciados.
- Um [[ui:exp.node.ws_send]] ou [[ui:exp.node.wait_ws]] abre a conexão que o
  [[ui:exp.node.ws_connect]] dele descreve, para esse único teste.
- Em uma requisição HTTP, a resposta é mostrada — status, tempo, tamanho e o corpo,
  formatado como JSON. Os valores que os nós [[ui:exp.node.extract]] depois da
  requisição tomariam são preenchidos de imediato. Clique em um valor em uma resposta JSON para
  adicionar um nó [[ui:exp.node.extract]] para ele logo depois da requisição, com a
  variável já nomeada e o valor conhecido.
  [[ui:http.mockThis]] transforma a resposta em uma rota de um
  [emulador](../tools/emulators.md).
- O resultado também é escrito no console.

A prévia também é resolvida pelo motor, cerca de um quarto de segundo
depois de uma alteração. Valores de segredos nunca são mostrados, ali nem em lugar nenhum.

## Validação {#validation}

O editor verifica o experimento enquanto você edita, e mais uma vez quando você pressiona
[[ui:exp.run]]:

- Uma saída que ainda precisa de um fio pulsa em âmbar.
- Um nó que nenhum fio de [[ui:exp.node.start]] alcança é desenhado tracejado; a dica dele diz isso.
- O nó de que um problema trata fica contornado, e o problema é escrito acima dos
  campos dele.
- ⚠ [[ui:exp.needsLinks]] na barra de ferramentas nomeia o problema; um clique seleciona
  o nó.

Um experimento executa quando, entre outras coisas:

- tem um nome, exatamente um [[ui:exp.node.start]] e um
  [[ui:exp.node.end]], e no máximo 64 nós — o editor não adiciona, cola nem
  duplica nada além disso, e avisa;
- todo nó é alcançável a partir de [[ui:exp.node.start]] e toda saída obrigatória tem um fio;
- os únicos ciclos são os corpos de nós [[ui:exp.node.loop]] que voltam
  a eles;
- toda verificação e todo [[ui:exp.node.extract]] tem uma requisição HTTP antes dele em cada caminho (uma
  sob carga não conta: ela não deixa resposta);
- um [[ui:exp.node.ws_send]], [[ui:exp.node.wait_ws]] ou [[ui:exp.node.ws_close]]
  vem depois do [[ui:exp.node.ws_connect]] que usa;
- todo campo está preenchido e dentro do intervalo, e todo modelo cita algo
  conhecido naquele ponto.

Rascunhos inacabados são salvos do mesmo jeito. Uma execução é recusada, antes de qualquer
tráfego, quando um segredo de que ela precisa não está guardado. Todas as mensagens estão listadas na
[referência de erros](../reference/errors.md).

## Execução {#running}

[[ui:exp.run]] verifica o experimento, o salva e o inicia. Se ele não pode
executar, o problema é mostrado e o nó correspondente é selecionado. Caso contrário, a linha do tempo abre
e os nós se acendem conforme a execução chega a eles: ● em execução, ✓ aprovado, ✕ com falha,
↻ tentando de novo, ⟳ repetindo, ⚡ sob carga; os fios que levaram o fluxo também
são coloridos.

- Esperas, emuladores e retransmissores de degradação abrem as portas deles antes da primeira
  etapa, então nada que chegue cedo se perde. Uma porta que não pode ser aberta
  (outro programa a ocupa, por exemplo) impede a execução de começar, e o nó
  que precisa dela é mostrado.
- [[ui:common.stop]] encerra a execução na hora: pausas, esperas e cargas incluídas.
  A execução também é uma tarefa na faixa de tarefas do console, que também pode pará-la.
- Uma execução que demora mais de 300 segundos é parada e falha.
- Enquanto uma execução está em andamento, o experimento não pode ser editado; deslocar,
  ampliar e selecionar continuam funcionando.

▾ [[ui:exp.runWith]] ao lado do botão executa uma vez com outro perfil, outros
valores de parâmetro ou uma semente dada; o experimento em si não é alterado, e o
formulário mantém o que você digitou até o app fechar ([Dados e
modelos](data.md)).

### A linha do tempo da execução {#timeline}

A [[ui:exp.timeline]] lista uma linha por etapa: a hora, o nó e o que
aconteceu — aprovada, com falha e por quê, uma nova tentativa, o progresso de uma repetição, os
números de uma carga a cada segundo. Clique em uma linha para selecionar o nó dela no canvas.

A linha de título dela contém:

- o resultado: [[ui:exp.passed]], [[ui:exp.failed]] com o motivo, ou
  [[ui:exp.stopped]];
- [[ui:exp.reportSaved]] — o arquivo de relatório da execução (em um servidor, um download);
- [[ui:exp.compare]] — esta execução ao lado de uma anterior do mesmo experimento;
- o perfil e os valores alterados que a execução usou, quando houve;
- a semente da execução, com [[ui:exp.pinSeed]] para guardá-la no experimento, de modo que as
  próximas execuções sorteiem os mesmos valores aleatórios, ou [[ui:exp.unpinSeed]] depois que
  ela foi fixada.

Quando o [Inspetor](../tools/inspector.md) estava capturando, uma espera (ou uma
resposta esperada) que correspondeu leva ao quadro com que correspondeu; um clique o abre no
Inspetor. Relatórios, sementes e a comparação de execuções estão em [Execuções e
relatórios](runs.md).

## Modo foco e tela cheia {#focus}

▢ [[ui:exp.focus]] oculta a barra lateral, o cabeçalho e o painel inferior (console
e Inspetor), deixando a tela para o editor. ⛶ [[ui:exp.fullscreen]] coloca
a janela em tela cheia e liga o modo foco; sair da tela cheia devolve o modo foco
ao que estava. Passar para outra tela sai do modo foco.

<kbd>Esc</kbd> no canvas, sem mais nada a fechar, sai da tela cheia e
depois do modo foco.

## Salvamento e arquivos {#files}

O experimento é salvo sozinho, um instante depois de cada alteração, em
`experiment.json` na pasta de dados (`Documents/SignalLab` em um desktop; um
servidor tem a sua — veja [Arquivos e pastas](../reference/files.md)). Rascunhos
que ainda não podem executar também são salvos. Uma execução salva antes, de modo que o que executou é o que está no
disco.

Se esse arquivo não puder ser lido — editado à mão e com JSON quebrado, por exemplo — o editor
diz qual arquivo e por quê, e o deixa em paz. Abrir outro experimento em
☰ [[ui:exp.documents]] o substitui no próximo salvamento.

☰ [[ui:exp.documents]] abre a caixa de diálogo dos experimentos:

- [[ui:exp.templates]]: escolha um e pressione [[ui:exp.openDocument]]. Todos
  usam endereços de loopback.
- [[ui:exp.importJson]] lê um arquivo de experimento — escrito por esta versão do
  Signal Lab ou por uma anterior, de até 4 MiB — e mostra o nome e quantos
  nós e conexões ele tem antes de você abri-lo. Arquivos de versões anteriores
  são atualizados ao abrir. Um arquivo que não pode ser lido, ou que não seria
  um experimento válido, é recusado com o motivo, e o experimento atual
  permanece.
- [[ui:exp.exportJson]] grava um instantâneo do experimento atual — nós,
  fios, posições, parâmetros e perfis, nunca valores de segredos — em um novo
  arquivo em `exports` na pasta de dados e mostra o caminho dele; em um servidor, com um
  link [[ui:common.download]].
- [[ui:exp.openDocument]] substitui o experimento atual pelo modelo ou
  arquivo. Não o executa, e <kbd>Ctrl</kbd>+<kbd>Z</kbd> traz o anterior
  de volta.

| Modelo | O que faz |
| --- | --- |
| [[ui:exp.templateEmpty]] | [[ui:exp.node.start]] e [[ui:exp.node.end]], para o seu próprio fluxo |
| [[ui:exp.templateHttp]] | Um GET para `http://127.0.0.1:8080/` e uma verificação do status 200 — o experimento com que você começa |
| [[ui:exp.templateBranch]] | A mesma requisição; em caso de 200, uma mensagem OSC para `127.0.0.1:9000`, caso contrário um atraso de 500 ms |
| [[ui:exp.templateParallel]] | Dois ramos ao mesmo tempo — uma requisição e uma linha de log — unidos antes de [[ui:exp.node.end]] |
| [[ui:exp.templatePingReply]] | Envia `/ping` com o id da execução para `127.0.0.1:9000` e aguarda em `127.0.0.1:9001` um `/pong` que o traga de volta |
| [[ui:exp.templatePoll]] | Pergunta a um dispositivo por `/status` a cada 0,3 s até que ele responda `ready`, dez vezes no máximo |
| [[ui:exp.templateFlaky]] | Uma API emulada que falha duas vezes antes de responder, e um laço que pergunta até ela responder |
| [[ui:exp.templateFaults]] | Datagramas para um dispositivo emulado por um retransmissor de degradação enquanto um ramo paralelo põe a rede em estado limpo, com perdas, desligada e limpa de novo |
| [[ui:exp.templateOutage]] | Uma API emulada derrubada por dois segundos por um ramo paralelo, e um cliente que continua perguntando até ela responder de novo |
| [[ui:exp.templateWsEcho]] | Conecta a um serviço de eco em `ws://127.0.0.1:9001/echo`, envia um ping em JSON, espera recebê-lo de volta sem alteração e fecha |

Os mesmos arquivos executam sem o editor: `signallab run experiment.json` — veja
[A linha de comando](../automation/cli.md).

## Atalhos de teclado {#shortcuts}

As teclas simples agem no canvas e são deixadas em paz enquanto você digita em um campo. Em
um Mac, em um navegador, <kbd>Cmd</kbd> vale onde está escrito <kbd>Ctrl</kbd>.

| Teclas | Ação |
| --- | --- |
| <kbd>A</kbd> | Adicionar um nó depois do selecionado, ou no meio da vista quando nada está selecionado |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:exp.sendNow]], ou [[ui:exp.listenNow]], no nó selecionado |
| <kbd>Ctrl</kbd>+<kbd>Z</kbd> | Desfazer |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd>, <kbd>Ctrl</kbd>+<kbd>Y</kbd> | Refazer |
| <kbd>Ctrl</kbd>+<kbd>A</kbd> | Selecionar todos os nós |
| <kbd>Ctrl</kbd>+<kbd>C</kbd> / <kbd>Ctrl</kbd>+<kbd>X</kbd> / <kbd>Ctrl</kbd>+<kbd>V</kbd> | Copiar / recortar / colar os nós selecionados e os fios entre eles |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | Duplicar os nós selecionados |
| <kbd>Delete</kbd>, <kbd>Backspace</kbd> | Remover o fio selecionado, ou os nós selecionados |
| Setas (um nó em foco) | Mover os nós selecionados 5 pixels; com <kbd>Shift</kbd>, 20 |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | Localizar um nó |
| <kbd>Ctrl</kbd>+<kbd>0</kbd> | Enquadrar o grafo |
| <kbd>Ctrl</kbd>+<kbd>1</kbd> | Zoom a 100 % |
| <kbd>Ctrl</kbd> + roda do mouse | Zoom em torno do ponteiro |
| <kbd>Shift</kbd> + clique, <kbd>Ctrl</kbd> + clique | Incluir um nó na seleção, ou tirá-lo |
| <kbd>Shift</kbd> + arrastar no canvas vazio | Selecionar com um quadro |
| Clique duplo em um nó | Editar os campos dele |
| Clique duplo no canvas vazio | Adicionar um nó ali |
| <kbd>Enter</kbd>, <kbd>Space</kbd> em uma saída em foco | Iniciar um fio a partir dela |
| `{{` ou <kbd>Ctrl</kbd>+<kbd>Space</kbd> em um campo | Sugerir parâmetros, variáveis, segredos e geradores |
| <kbd>Esc</kbd> em um campo | De volta ao nó no canvas |
| <kbd>Esc</kbd> no canvas | Fechar o menu de adição; senão cancelar o fio que está sendo desenhado; senão soltar o fio selecionado; senão limpar uma seleção de vários; senão sair da tela cheia; senão sair do modo foco |

No menu de adição e no localizador, <kbd>↑</kbd> <kbd>↓</kbd> escolhem,
<kbd>Enter</kbd> aceita a escolha e <kbd>Esc</kbd> fecha. Todos os atalhos do app estão em
[Atalhos de teclado](../reference/shortcuts.md).

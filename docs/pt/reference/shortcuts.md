---
title: Atalhos de teclado
description: Todos os atalhos de teclado do Signal Lab — em qualquer lugar do aplicativo, no editor de experimentos, na biblioteca de sinais, nas telas de ferramentas, nos painéis e no menu de idioma.
---

# Atalhos de teclado

Todas as teclas a que o Signal Lab responde, agrupadas pelo lugar onde funcionam. Atalhos com
uma única letra ou Delete nunca agem enquanto você digita em um campo.

::: tip
Em um navegador em um Mac, <kbd>⌘</kbd> funciona onde quer que <kbd>Ctrl</kbd> esteja escrito
abaixo, exceto <kbd>Ctrl</kbd>+<kbd>Space</kbd> em um campo de modelo.
:::

## Em qualquer lugar {#anywhere}

| Teclas | O que fazem |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>K</kbd> | Abre [[ui:sig.paletteTitle]], a paleta de sinais, em qualquer tela — mesmo enquanto você digita em um campo. De novo, a fecha |
| <kbd>F1</kbd> | Abre esta documentação na página da tela em que você está, como [[ui:app.docs]] |
| <kbd>Tab</kbd>, <kbd>Shift</kbd>+<kbd>Tab</kbd> | Move entre os controles. A dica de um controle aparece quando ele recebe o foco; qualquer outra tecla a oculta |
| <kbd>Space</kbd>, <kbd>Enter</kbd> | Pressiona o botão que tem o foco — tudo o que é clicável é um botão |
| <kbd>Esc</kbd> | Fecha a janela que está aberta |

Na paleta de sinais:

| Teclas | O que fazem |
| --- | --- |
| Digitar | Filtra os sinais |
| <kbd>↑</kbd> <kbd>↓</kbd> | Escolhe um sinal |
| <kbd>Enter</kbd> | Envia-o e fecha a paleta |
| <kbd>Esc</kbd> | Fecha a paleta; um clique ao lado dela também |

## Editor de experimentos {#editor}

No canvas — quando nenhum campo de texto tem o foco:

| Teclas | O que fazem |
| --- | --- |
| <kbd>A</kbd> | Abre o menu para adicionar um nó ([[ui:exp.addNode]]). Com um nó selecionado, o novo vai depois dele |
| <kbd>Delete</kbd> ou <kbd>Backspace</kbd> | Remove a conexão selecionada, ou os nós selecionados — nunca [[ui:exp.node.start]] nem [[ui:exp.node.end]] |
| <kbd>Tab</kbd> | Percorre os nós, as portas deles e as conexões; um nó ou uma conexão que recebe o foco é selecionado |
| <kbd>Enter</kbd> ou <kbd>Space</kbd> em uma saída, depois em um nó ou na entrada dele | Conecta os dois |
| <kbd>←</kbd> <kbd>→</kbd> <kbd>↑</kbd> <kbd>↓</kbd> | Move os nós selecionados 5 pontos (20 com <kbd>Shift</kbd>), enquanto um nó tem o foco. Um pressionar e segurar é um passo do histórico |
| <kbd>Esc</kbd> | Fecha o menu de adicionar; senão cancela uma conexão que está sendo desenhada; senão solta a conexão selecionada; senão limpa uma seleção de vários nós; senão sai de [[ui:exp.fullscreen]]; senão sai de [[ui:exp.focus]] |
| <kbd>Ctrl</kbd>+<kbd>Z</kbd> | [[ui:exp.undo]] |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd>, <kbd>Ctrl</kbd>+<kbd>Y</kbd> | [[ui:exp.redo]] |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | [[ui:exp.duplicate]]: uma cópia dos nós selecionados, com as conexões entre eles |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | Encontra um nó pelo tipo, pelo que faz ou pelo id |
| <kbd>Ctrl</kbd>+<kbd>0</kbd> | [[ui:exp.fit]]: o grafo inteiro à vista |
| <kbd>Ctrl</kbd>+<kbd>1</kbd> | [[ui:exp.resetZoom]]: 100 % |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:exp.sendNow]] — para uma espera, [[ui:exp.listenNow]]: só o nó selecionado, sem executar o experimento |
| <kbd>Ctrl</kbd>+<kbd>A</kbd> | Seleciona todos os nós |
| <kbd>Ctrl</kbd>+<kbd>C</kbd> | [[ui:exp.copy]]: os nós selecionados e as conexões entre eles; [[ui:exp.node.start]] e [[ui:exp.node.end]] ficam de fora, como ao duplicar |
| <kbd>Ctrl</kbd>+<kbd>X</kbd> | Recorta-os |
| <kbd>Ctrl</kbd>+<kbd>V</kbd> | Cola nós copiados aqui ou em outro experimento, com ids novos |

<kbd>Ctrl</kbd>+<kbd>A</kbd>, <kbd>C</kbd>, <kbd>X</kbd> e <kbd>V</kbd> agem sobre nós enquanto
o editor tem o foco, ou enquanto nada o tem e nenhum texto está selecionado na página. Com um
texto selecionado em outro lugar — no [[ui:console.title]], em um relatório —, valem as teclas
da própria página: <kbd>Ctrl</kbd>+<kbd>C</kbd> copia esse texto.

Com o mouse:

| Gesto | O que faz |
| --- | --- |
| Arrastar no canvas vazio | Move a vista |
| <kbd>Shift</kbd> + arrastar no canvas vazio | Acrescenta à seleção os nós que um quadro toca |
| <kbd>Shift</kbd> ou <kbd>Ctrl</kbd> + clique em um nó | Acrescenta-o à seleção, ou o tira dela |
| Arrastar um de vários nós selecionados | Move todos eles |
| Clique duplo no canvas vazio | Abre ali o menu de adicionar |
| <kbd>Ctrl</kbd> + roda do mouse | Dá zoom no ponteiro |

No menu de adicionar, no localizador de nós e nas propriedades:

| Onde | Teclas | O que fazem |
| --- | --- | --- |
| Menu de adicionar | Digitar | Filtra os nós e os sinais salvos |
| Menu de adicionar | <kbd>↑</kbd> <kbd>↓</kbd>, <kbd>Enter</kbd>, <kbd>Esc</kbd> | Escolhem, adicionam, fecham |
| Localizador de nós | <kbd>↑</kbd> <kbd>↓</kbd> | Escolhem um nó |
| Localizador de nós | <kbd>Enter</kbd> | Mostra-o no canvas e o seleciona |
| Localizador de nós | <kbd>Tab</kbd> | Volta ao campo de busca |
| Localizador de nós | <kbd>Esc</kbd> | Fecha-o |
| [[ui:exp.properties]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:exp.sendNow]] para o nó mostrado |
| [[ui:exp.properties]] | <kbd>Esc</kbd> em um campo | Volta ao canvas com o nó selecionado, onde <kbd>A</kbd> adiciona o próximo |

Em um campo que aceita modelos (`{{name}}`):

| Teclas | O que fazem |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>Space</kbd> | Digita `{{` no cursor e lista o que pode entrar ali: parâmetros, variáveis, segredos, geradores |
| Digitar `{{` | Lista o mesmo |
| <kbd>↑</kbd> <kbd>↓</kbd> | Escolhem na lista |
| <kbd>Enter</kbd> ou <kbd>Tab</kbd> | Inserem a escolha e fecham o `}}` |
| <kbd>Esc</kbd> | Fecha a lista; o campo mantém o foco |

Nas janelas do experimento:

| Onde | Teclas | O que fazem |
| --- | --- | --- |
| [[ui:exp.params]] | <kbd>Enter</kbd> no valor do último parâmetro | Adiciona outro parâmetro |
| [[ui:exp.params]], [[ui:exp.runWith]] | <kbd>Esc</kbd> | Fecha a janela |
| [[ui:exp.runWith]] | <kbd>Enter</kbd> em um campo | [[ui:exp.run]] com esses valores |
| [[ui:exp.secrets]] | <kbd>Enter</kbd> em um valor | Salva-o |
| [[ui:exp.secrets]] | <kbd>Esc</kbd> em um valor ou em um nome novo | Cancela; [[ui:exp.params]] continua aberta |

## Biblioteca de sinais {#signals}

| Onde | Teclas | O que fazem |
| --- | --- | --- |
| [[ui:nav.signals]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | [[ui:sig.fire]]: envia o sinal selecionado |
| [[ui:nav.signals]] | Clique duplo em um sinal | Envia-o |
| Uma pasta | <kbd>F2</kbd> | [[ui:sig.renameFolder]] |
| Renomear uma pasta | <kbd>Enter</kbd>, <kbd>Esc</kbd> | Mantém o novo nome, ou cancela |
| Uma pasta | <kbd>Delete</kbd>, duas vezes | [[ui:sig.removeFolder]]; o que há nela sobe um nível |
| Uma pasta | <kbd>→</kbd> <kbd>←</kbd> | Abre-a, fecha-a |
| Envios de [[ui:nav.http]], [[ui:nav.osc]], [[ui:nav.mqtt]] | <kbd>Ctrl</kbd>+<kbd>S</kbd> | [[ui:sig.save]] quando a mensagem está ligada a um sinal da biblioteca (aberta a partir dele ou salva ali) e foi alterada; [[ui:sig.saveNew]] quando ainda não está na biblioteca |
| [[ui:sig.saveTitle]] | <kbd>Enter</kbd>, <kbd>Esc</kbd> | Salva, ou cancela |

## Telas de ferramentas {#tools}

| Tela | Teclas | O que fazem |
| --- | --- | --- |
| [[ui:nav.http]] | <kbd>Enter</kbd> na URL, em um cabeçalho, nas credenciais ou no tempo limite | Envia a requisição |
| [[ui:nav.http]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> em qualquer ponto da requisição, o corpo incluído | Envia a requisição |
| [[ui:nav.osc]] | <kbd>Enter</kbd> no destino, no endereço ou em um argumento | Envia a mensagem |
| [[ui:nav.ws]] | <kbd>Enter</kbd> na URL | Conecta |
| [[ui:nav.ws]] | <kbd>Ctrl</kbd>+<kbd>Enter</kbd> em [[ui:ws.message]] | Envia-a |
| [[ui:nav.mqtt]] | <kbd>Enter</kbd> em [[ui:mq.addSubscription]] | Assina |
| [[ui:nav.broadcast]] | <kbd>Enter</kbd> em [[ui:bc.targetAddress]] ou [[ui:bc.targetCidr]] — todos os modos, exceto [[ui:bc.mode.list]] | [[ui:bc.sendOnce]] |

Em [[ui:feedback.title]]: <kbd>Ctrl</kbd>+<kbd>Enter</kbd> em [[ui:feedback.message]] envia a
mensagem (<kbd>Enter</kbd> em [[ui:feedback.email]] também), <kbd>Ctrl</kbd>+<kbd>V</kbd> em
qualquer ponto da janela cola uma captura de tela da área de transferência, e <kbd>Esc</kbd>
a fecha.

## Painéis {#panes}

As alças entre os painéis — [[ui:layout.console]], [[ui:layout.properties]],
[[ui:layout.timeline]] — recebem o foco com <kbd>Tab</kbd>:

| Teclas | O que fazem |
| --- | --- |
| <kbd>↑</kbd> <kbd>↓</kbd> | O console e a linha do tempo: mais alto, mais baixo, de 16 em 16 pixels |
| <kbd>←</kbd> <kbd>→</kbd> | As propriedades: mais largas, mais estreitas, de 16 em 16 pixels (ao contrário quando a interface é lida da direita para a esquerda) |
| <kbd>Shift</kbd> + uma seta | Quatro vezes mais longe |
| <kbd>Home</kbd>, <kbd>End</kbd> | O menor tamanho, o maior |
| <kbd>Enter</kbd> | O tamanho padrão; um clique duplo na alça também |

## Menu de idioma {#language}

A bandeira e as letras no cabeçalho.

| Onde | Teclas | O que fazem |
| --- | --- | --- |
| No botão | <kbd>↓</kbd> ou <kbd>↑</kbd> | Abre a lista |
| Na lista | <kbd>↓</kbd> <kbd>↑</kbd> | O próximo idioma, o anterior |
| Na lista | <kbd>Home</kbd>, <kbd>End</kbd> | O primeiro, o último |
| Na lista | Uma letra | O próximo idioma cujo nome — no próprio idioma, no seu ou em inglês — ou cujas letras começam com ela |
| Na lista | <kbd>Enter</kbd> ou <kbd>Space</kbd> | Muda para ele |
| Na lista | <kbd>Esc</kbd> ou <kbd>Tab</kbd> | Fecha a lista |

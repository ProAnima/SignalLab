---
title: Primeiros passos
description: Uma primeira sessão em um só computador — envie uma mensagem OSC e veja-a chegar, salve-a como sinal, consulte uma API emulada e monte e execute um pequeno experimento.
---

# Primeiros passos

Esta sessão não precisa de nada além do Signal Lab: tudo vai para `127.0.0.1`, este computador,
então nenhum dispositivo, rede ou regra de firewall entra em jogo. Você vai:

1. enviar uma mensagem OSC e vê-la chegar;
2. ver a mesma mensagem no Inspetor;
3. salvá-la na biblioteca e enviá-la de novo de qualquer lugar;
4. iniciar uma API HTTP emulada e perguntar algo a ela;
5. executar um experimento contra essa API, ler por que ele falha, corrigi-lo e acrescentar uma
   verificação.

Se ainda não instalou o Signal Lab, veja [Instalação e atualização](install.md). Não sabe onde
fica alguma coisa na janela? Veja [A janela](interface.md).

## Enviar uma mensagem OSC e vê-la chegar {#osc}

Primeiro, algo que receba a mensagem: o monitor da tela OSC.

1. Abra [[ui:nav.osc]] na barra lateral.
2. Na seção [[ui:osc.monitor]], defina o campo [[ui:common.bind]] como `127.0.0.1:9000`, para
   que o monitor escute só neste computador.
3. Pressione [[ui:osc.listen]]. O botão passa a ser [[ui:common.stop]], o console diz que o
   monitor está escutando, e o monitor aparece como tarefa na faixa do painel inferior.

Agora a mensagem, a partir do envio ao lado:

4. Na seção [[ui:osc.sender]], deixe o campo [[ui:common.target]] em `127.0.0.1:9000`, a porta
   em que o monitor escuta.
5. Deixe o campo [[ui:common.address]] em `/hello/avatar/1` e o único argumento float em
   [[ui:common.arguments]] em `1.0` — ou digite um endereço e valores seus.
6. Pressione [[ui:common.send]], ou <kbd>Enter</kbd> no campo de destino ou de endereço.

Uma linha aparece na tabela do monitor: a [[ui:common.time]] de chegada, a [[ui:osc.from]]
(`127.0.0.1` e a porta de onde a mensagem saiu), o [[ui:osc.address]] e os [[ui:osc.args]].
Abaixo do envio, uma linha confirma o que foi enviado e o tamanho em bytes; envie de novo e ela
conta as repetições.

::: tip Um aviso sobre o firewall?
No Windows, iniciar o monitor pode fazer aparecer, abaixo do cabeçalho, um aviso sobre o Firewall
do Windows. Ele trata de mensagens vindas de *outras* máquinas; o tráfego em `127.0.0.1` nunca é
filtrado. Pressione [[ui:fw.dismiss]] por enquanto — [O aviso do firewall](interface.md#firewall-notice)
explica quando permitir.
:::

## Vê-la no Inspetor {#inspector}

O Inspetor registra cada quadro que cada ferramenta envia e recebe — mas só enquanto a captura
está ativa.

1. No painel inferior, abra a aba [[ui:dock.inspector]].
2. Pressione [[ui:ins.arm]]. O ponto da aba acende.
3. De volta ao envio, pressione [[ui:common.send]] mais uma vez.

Aparecem duas linhas, a mais recente primeiro: a mensagem como foi enviada (→) e como o monitor a
recebeu (←), cada uma com o protocolo, o endereço da outra ponta, o tamanho e um resumo. Clique em
uma: [[ui:ins.detail]] mostra qual ferramenta a enviou ou recebeu e em quais endereços, a mensagem
em [[ui:ins.decoded]] e os [[ui:ins.rawBytes]] que a compõem.

Pressione [[ui:ins.disarm]] quando terminar; com a captura desativada, ela não custa nada. Mais em
[Inspetor](../tools/inspector.md).

## Salvá-la como sinal e enviá-la de novo {#signal}

Uma mensagem que você vai querer de novo pertence à biblioteca de sinais.

1. Na tela OSC, pressione [[ui:sig.saveNew]] abaixo do envio.
2. Na caixa [[ui:sig.saveTitle]], defina [[ui:sig.name]] como `First message` e
   [[ui:sig.group]] como `Tutorial` — uma pasta nova é criada quando você salva nela.
3. Pressione [[ui:sig.saveConfirm]].

O envio agora está ligado a esse sinal: o botão diz [[ui:sig.savedState]], e um chip ao lado
mostra onde o sinal está. Mude o argumento e o chip registra a alteração; [[ui:sig.save]]
(<kbd>Ctrl</kbd>+<kbd>S</kbd>) atualizaria o sinal.

Agora envie-o de novo, de três jeitos:

- **Pela biblioteca.** Clique no chip: a tela [[ui:nav.signals]] abre com o sinal selecionado na
  pasta `Tutorial` (ou abra [[ui:nav.signals]] e clique nele ali). Pressione [[ui:sig.fire]], ou
  <kbd>Ctrl</kbd>+<kbd>Enter</kbd>; um clique duplo nele na lista também o envia.
- **De qualquer lugar.** Em qualquer tela, pressione <kbd>Ctrl</kbd>+<kbd>K</kbd>, digite `first`
  e pressione <kbd>Enter</kbd>.
- **De um experimento.** Quando você adiciona um nó, o menu lista os seus sinais em
  [[ui:exp.group.signals]], prontos para virar uma etapa que envia um deles.

Em todas as vezes, o monitor mostra a mensagem chegando e o console dá o nome do sinal. Um sinal
envia exatamente o que a tela dele teria enviado. Mais em [Sinais](../tools/signals.md).

Quando terminar com o OSC, pressione [[ui:common.stop]] no monitor.

## Consultar uma API emulada {#emulator}

O Signal Lab vem com cinco emuladores, todos em `127.0.0.1`. Um deles, a
[[ui:seed.emu.demo-api.name]], é uma API HTTP em `127.0.0.1:8080` com estas rotas:

| Requisição | Resposta |
| --- | --- |
| `GET /health` | `200` com `{"status":"ok","time":"…"}` — a hora atual |
| `GET /users/:id` | `200` com o usuário desse id, como `{"id":"42","name":"User 42"}` |
| `POST /users` | `201` com um cabeçalho `Location` e o novo id |
| `GET /slow` | `200` depois de 1,5 segundo |
| qualquer método, `/flaky` | `503`, `503` e depois `200` a partir da terceira requisição |
| qualquer outra coisa | `404` |

1. Abra [[ui:nav.emulators]]. A [[ui:emu.library]] lista os cinco; selecione
   [[ui:seed.emu.demo-api.name]].
2. Pressione [[ui:emu.start]]. Agora ela responde em `127.0.0.1:8080` e roda como tarefa.
3. Abra [[ui:nav.http]]. O método é `GET`; defina a URL como `http://127.0.0.1:8080/health`.
4. Pressione [[ui:common.send]], ou <kbd>Enter</kbd> na URL.

Em [[ui:http.response]] você vê o [[ui:http.status]] `200`, a [[ui:http.latency]], o
[[ui:http.size]], os cabeçalhos da resposta e o corpo JSON. Envie
`http://127.0.0.1:8080/flaky` três vezes: duas respostas `503` e depois `200` — é assim que um
serviço que se recupera aparece para um cliente que tenta de novo.

De volta a [[ui:nav.emulators]], o painel [[ui:emu.live]] conta cada requisição, e a lista
[[ui:emu.received]] mostra cada uma com a [[ui:emu.col.rule]] que a respondeu e a
[[ui:emu.col.reply]]. Deixe a [[ui:seed.emu.demo-api.name]] rodando para a próxima parte. Mais em
[Emuladores](../tools/emulators.md).

## Executar um experimento {#experiment}

Um experimento é um fluxo de etapas que você pode executar de novo e de novo. Aquele com que o
Signal Lab abre na primeira vez — o modelo [[ui:exp.templateHttp]] — envia uma requisição para
`http://127.0.0.1:8080/` e verifica se a resposta é `200`.

### Abrir o modelo {#open-template}

1. Abra [[ui:nav.experiment]].
2. Se o canvas não mostrar quatro nós — [[ui:exp.node.start]], [[ui:exp.node.http]],
   [[ui:exp.node.assert_status]], [[ui:exp.node.end]] —, pressione **☰** à esquerda da barra
   de ferramentas ([[ui:exp.documents]]), escolha [[ui:exp.templateHttp]] na lista de modelos e
   pressione [[ui:exp.openDocument]]. Abrir substitui o experimento do canvas;
   <kbd>Ctrl</kbd>+<kbd>Z</kbd> traz o anterior de volta.

Clique em um nó para ver as configurações dele em [[ui:exp.properties]], à direita. Os
experimentos se salvam sozinhos enquanto você edita.

### Executá-lo e ler por que ele falha {#first-run}

3. Pressione [[ui:exp.run]].

A [[ui:exp.timeline]] abre abaixo do canvas, com uma linha por etapa quando ela começa
([[ui:exp.running]]) e outra quando termina: a hora, o nó e como foi. Esta execução falha:

- [[ui:exp.node.start]] passa e informa a semente da execução.
- [[ui:exp.node.http]] passa: a requisição saiu e uma resposta voltou, `HTTP 404`.
- [[ui:exp.node.assert_status]] falha: esperava `200` e recebeu `404`.

A [[ui:seed.emu.demo-api.name]] não tem rota para `/`, então respondeu `404` — e a verificação
pegou isso. A linha no alto da linha do tempo diz [[ui:exp.failed]] e o porquê. Clique em uma
linha para selecionar o nó dela no canvas.

::: tip A própria requisição falhou?
Se a etapa [[ui:exp.node.http]] falhar com uma conexão recusada, nada está escutando em
`127.0.0.1:8080`: inicie a [[ui:seed.emu.demo-api.name]] em [[ui:nav.emulators]] e execute de
novo.
:::

### Corrigir a requisição {#fix}

4. Clique no nó [[ui:exp.node.http]].
5. Em [[ui:exp.properties]], mude [[ui:sig.url]] para `http://127.0.0.1:8080/health`.
6. Pressione [[ui:exp.run]].

Desta vez todas as etapas passam: [[ui:exp.node.assert_status]] diz [[ui:exp.step.checked]],
[[ui:exp.node.end]] diz [[ui:exp.step.complete]], e o título da linha do tempo diz
[[ui:exp.passed]].

### Acrescentar uma verificação {#add-check}

Um status `200` diz que o serviço respondeu; não diz o que ele respondeu. Verifique o corpo
também:

7. Clique no nó [[ui:exp.node.assert_status]].
8. Em [[ui:exp.properties]], pressione [[ui:exp.addNext]] — ou pressione <kbd>A</kbd> com o
   canvas em foco. Abre-se um menu de nós com um campo de busca.
9. Digite `assert_body` e pressione
   <kbd>Enter</kbd>. Um nó [[ui:exp.node.assert_body]] é adicionado entre
   [[ui:exp.node.assert_status]] e [[ui:exp.node.end]], já ligado, com o campo
   [[ui:exp.contains]] pronto para digitar.
10. Digite `"status":"ok"`.
11. Pressione [[ui:exp.run]].

A nova etapa passa. Mude o texto para algo que o corpo não contém e execute de novo para vê-la
falhar com o motivo.

### O que uma execução deixa {#report}

- **Um relatório.** Quando uma execução termina, [[ui:exp.reportSaved]] aparece no título da
  linha do tempo; passe o ponteiro sobre ele para ver o arquivo. Uma execução que termina,
  aprovada ou com falha, grava um relatório na pasta `runs` da sua pasta de dados, com os valores
  que usou e todas as etapas. No navegador, ele é um link de download.
- **Uma semente.** O título também mostra a semente da execução, com o botão [[ui:exp.pinSeed]]:
  os valores aleatórios de uma execução seguem a semente dela, e fixá-la os repete exatamente.

Mais em [Execuções e relatórios](../experiments/runs.md).

## Arrumar a casa {#clean-up}

Pressione [[ui:app.stopAll]] no cabeçalho: isso para a [[ui:seed.emu.demo-api.name]] e
qualquer outra coisa que ainda esteja rodando. O seu sinal, o experimento e os relatórios dele
ficam na sua pasta de dados.

## Para onde ir agora {#next}

- [Conceitos](concepts.md): as ideias por trás das telas, sinais, tarefas, emuladores e
  experimentos.
- [Experimentos](../experiments/index.md): o editor completo, e cada tipo de nó em
  [Nós](../experiments/nodes.md).
- [OSC](../protocols/osc.md), [HTTP](../protocols/http.md) e as páginas dos outros protocolos,
  quando você apontar o Signal Lab para equipamentos reais.
- [A linha de comando](../automation/cli.md): execute o mesmo experimento em um terminal ou em um
  pipeline.

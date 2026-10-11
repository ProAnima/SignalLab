---
title: Como uma execução se move
description: Início e Fim, saídas e fios, ramos paralelos e União, desvios, Nova tentativa, Repetição e Laço, esperas que escutam desde o início da execução e o que é verificado antes de uma execução.
---

# Como uma execução se move

Uma execução começa em [[ui:exp.node.start]], segue os fios de nó em nó e está
completa quando todos os ramos terminaram e [[ui:exp.node.end]] foi
alcançado. Esta página explica as regras que ela segue; o que cada nó faz está na
[referência dos nós](nodes.md), e os valores que viajam com ela em
[dados](data.md).

## Início e Fim {#start-end}

Um experimento tem exatamente um [[ui:exp.node.start]] e um
[[ui:exp.node.end]].

- [[ui:exp.node.start]] não tem entrada. Ele é aprovado na hora, e a linha dele na
  linha do tempo dá a semente da execução. A saída dele pode ter vários fios: o
  experimento então começa com ramos paralelos.
- Todo ramo que chega a [[ui:exp.node.end]] para ali. O Fim aparece como
  em execução desde a primeira chegada e é aprovado uma vez, depois que o último ramo
  terminou — e de modo nenhum se alguma etapa falhou. Essa aprovação é o que faz a
  execução ficar [[ui:exp.passed]].
- Uma execução em que todos os ramos terminaram sem erro mas nenhum chegou ao Fim
  falha com `run.no_end`.

## Saídas e fios {#outputs}

A etapa de um nó termina escolhendo uma saída, e a execução segue todos os fios dessa
saída. A maioria dos nós tem uma saída, [[ui:exp.outputPort]]; alguns escolhem
entre várias:

| Nó | Saídas que precisam de fio | Saídas que podem ter fio |
| --- | --- | --- |
| [[ui:exp.node.end]] | — | — |
| [[ui:exp.node.fork]] | [[ui:exp.branch1]], [[ui:exp.branch2]] | — |
| [[ui:exp.node.branch_status]], [[ui:exp.node.branch_value]] | [[ui:exp.yes]], [[ui:exp.no]] | — |
| Toda espera ([[ui:exp.node.wait_osc]], [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_http]], [[ui:exp.node.wait_ws]]) | [[ui:exp.portMatched]] | [[ui:exp.portTimeout]] |
| [[ui:exp.node.loop]] | [[ui:exp.portBody]], [[ui:exp.portDone]] | [[ui:exp.portLimit]] |
| Todos os outros nós | [[ui:exp.outputPort]] | — |

Para ligar, arraste de uma saída até um nó; solto no canvas vazio, ele adiciona
ali um novo nó. Arrastar de uma saída que já tem um fio adiciona outro.
[[ui:exp.addNext]], a tecla <kbd>A</kbd> e o ＋ em um fio inserem um nó
no fio existente em vez disso.

Um grafo inacabado é um rascunho: ele é salvo, mas não executa.
[[ui:exp.needsLinks]] na barra de ferramentas diz o que falta e mostra o nó.
Veja [o que é verificado antes de uma execução](#validation).

## Ramos paralelos {#parallel}

### Vários fios de uma saída {#fan-out}

Quando uma saída tem vários fios — a de [[ui:exp.node.start]] inclusive — todos os
nós a que eles levam executam ao mesmo tempo. O primeiro fio continua o ramo;
cada fio adicional inicia um ramo paralelo. Cada ramo leva a própria cópia das
variáveis e da última resposta HTTP, de modo que o que um ramo define ou
recebe não é visto pelos outros.

### Ramo paralelo e União {#fork-join}

[[ui:exp.node.fork]] é aprovado na hora e sai por [[ui:exp.branch1]] e
[[ui:exp.branch2]] — o mesmo que dois fios de uma
saída, desenhado como um nó.

[[ui:exp.node.join]] espera **todos** os fios que chegam a ele e então
continua como um ramo só, com as cópias mescladas na ordem desses fios:

- as variáveis de todos — em um nome que dois ramos definem, vale o fio
  listado depois no experimento;
- a resposta HTTP do último fio, nessa ordem, que traz uma;
- para as esperas depois dele, a mais antiga das últimas ações delas.

A ordem dos fios decide, nunca qual ramo calhou de terminar primeiro.

::: warning Una só o que executa em paralelo
Uma União conta os fios dela, não importa como passaram a executar em paralelo: de um
[[ui:exp.node.fork]], de vários fios de uma saída, de caminhos separados.
Atrás de [[ui:exp.yes]] e [[ui:exp.no]] de um desvio só um caminho executa, então uma
União alimentada pelos dois espera um ramo que nunca vem: a execução falha com
`run.join_waiting`, dizendo quantos fios nunca foram seguidos. Para reunir
caminhos alternativos, ligue-os direto ao nó seguinte.
:::

Um nó que não é uma União, alcançado por dois ramos paralelos, executa uma vez
para cada um deles. Ligado camada após camada, isso dobra a cada camada, então uma execução
inicia no máximo 1.000 ramos paralelos no total; mais um a faz falhar com
`run.too_many_branches` no nó que o teria iniciado. Reúna esses
ramos com um [[ui:exp.node.join]].

### Quando uma etapa falha {#failure}

A primeira falha reprova a execução. Os outros ramos não iniciam nenhuma etapa nova: uma
repetição ou uma carga termina antes, qualquer outra etapa em que estejam executa até o fim. Uma
falha que eles encontram nesse meio-tempo é informada na linha do tempo, mas não é o erro da execução.
Uma etapa que esbarrou em um tempo esgotado havendo um fio
[[ui:exp.portTimeout]] não falhou — veja [esperas](#timeout).

## Desvios {#branching}

| Nó | Sai por [[ui:exp.yes]] quando |
| --- | --- |
| [[ui:exp.node.branch_status]] | a última resposta HTTP neste caminho tem o status dado |
| [[ui:exp.node.branch_value]] | a comparação dele vale — veja [comparação de valores](data.md#compare) |

Caso contrário, cada um sai por [[ui:exp.no]]. Um desvio por status precisa de uma requisição
HTTP antes dele em todos os caminhos; um desvio por valor precisa que os nomes que ele lê
sejam conhecidos ali. Quando os caminhos depois de [[ui:exp.yes]] e [[ui:exp.no]] se encontram de novo, o nó em que se encontram executa uma vez, e só as variáveis definidas
nos dois caminhos são conhecidas ali ([onde uma variável é conhecida](data.md#visibility)).

## Nova tentativa {#retry}

Uma etapa que envia ou escuta pode tentar de novo quando falha: ative
[[ui:exp.retryOn]] nas propriedades dela.

| Configuração | O quê | Intervalo | Valor inicial |
| --- | --- | --- | --- |
| [[ui:exp.attempts]] | tentativas no total, a primeira incluída | 1–10 | 3 |
| [[ui:exp.retryDelay]] | a pausa antes da segunda tentativa | 0–60.000 ms | 500 |
| [[ui:exp.backoff]] | [[ui:exp.backoff.fixed]]: todas as pausas iguais; [[ui:exp.backoff.exponential]]: cada pausa o dobro da anterior | — | [[ui:exp.backoff.fixed]] |

- A Nova tentativa vale para [[ui:exp.node.http]], [[ui:exp.node.tcp]],
  [[ui:exp.node.mqtt]], [[ui:exp.node.osc]], [[ui:exp.node.udp]],
  [[ui:exp.node.ws_connect]], [[ui:exp.node.ws_send]] e toda espera. Os outros
  nós a recusam (`node.retry_unsupported`), e uma requisição HTTP sob
  [carga](load.md) não aceita Nova tentativa.
- Nenhuma pausa é maior que 60 s, seja qual for o resultado da duplicação.
- Cada tentativa que falha aparece na linha do tempo como [[ui:exp.retry]], com o
  número e o motivo. A etapa então é aprovada, ou falha com o motivo da última
  tentativa.
- Só a execução é repetida. Um campo cujo modelo não se resolve
  falha na hora.
- Um envio que aguarda a resposta envia de novo. Uma espera aguarda de novo, contando
  a partir da última ação do ramo como antes.
- Uma espera com um fio [[ui:exp.portTimeout]] não falha por tempo esgotado, então
  não é tentada de novo: ela segue [[ui:exp.portTimeout]].
- [Parar](#stop) encerra uma pausa na hora.

## Repetição {#repeat}

Uma etapa que envia pode enviar de novo e de novo — um sinal de vida, uma consulta, um fluxo
contínuo — sem um laço no grafo: ative [[ui:exp.repeatOn]].

| Configuração | O quê | Intervalo | Valor inicial |
| --- | --- | --- | --- |
| [[ui:exp.repeatBy]] | [[ui:exp.repeatBy.count]] ou [[ui:exp.repeatBy.duration]] | — | [[ui:exp.repeatBy.count]] |
| [[ui:exp.repeatCount]] | envios no total, o primeiro incluído | 2–10.000 | 10 |
| [[ui:exp.repeatDuration]] | por quanto tempo continuar enviando, a partir do primeiro envio | 1–300.000 ms | 10.000 |
| [[ui:exp.repeatInterval]] | a pausa entre dois envios | 10–60.000 ms | 1000 |
| [[ui:exp.repeatJitter]] | cada pausa é até este tanto mais longa, ao acaso | 0–60.000 ms | 0 |

- A Repetição vale para [[ui:exp.node.http]], [[ui:exp.node.tcp]],
  [[ui:exp.node.mqtt]], [[ui:exp.node.osc]], [[ui:exp.node.udp]] e
  [[ui:exp.node.ws_send]] (`node.repeat_unsupported` nos demais). Uma requisição HTTP
  tem Repetição ou [carga](load.md), não as duas.
- Cada envio é feito como seria um envio isolado: os modelos dele são lidos de novo —
  `{{counter}}` é o número do envio, `{{now}}` o horário dele — e a Nova tentativa, quando ativa,
  vale para cada envio. Um envio que aguarda uma resposta aguarda a própria.
- Por um tempo, um envio só é feito se puder começar antes de o tempo acabar.
- O jitter é sorteado a partir da semente da execução: a mesma semente dá as mesmas
  pausas.
- A linha do tempo informa o progresso como [[ui:exp.repeating]] no máximo uma vez por segundo.
  A etapa é aprovada depois do último envio, com o resultado desse envio; um envio que
  falha de vez reprova a etapa.
- Uma falha em outro ramo encerra os envios; [Parar](#stop) encerra uma pausa na
  hora.

Ela precisa caber em uma execução: os envios e as pausas mais longas deles
(`(count − 1) × (interval + jitter)`) no máximo 300 s (`node.repeat_too_long`),
e uma repetição por tempo no máximo 10.000 envios (`node.repeat_too_many`).

## Laço {#loop}

[[ui:exp.node.loop]] executa as etapas da saída [[ui:exp.portBody]] de novo e de
novo; a última delas é ligada de volta ao Laço.

| Configuração | O quê | Intervalo |
| --- | --- | --- |
| [[ui:exp.loopMax]] | o máximo de iterações | 1–1000 |
| [[ui:exp.loopUntilOn]] | uma condição de saída: [[ui:exp.value]], [[ui:exp.operator]], [[ui:exp.expected]], como em [[ui:exp.node.assert_value]] | opcional |

1. Alcançado de fora, o Laço inicia a iteração 1 em [[ui:exp.portBody]].
2. Cada vez que o corpo volta, a condição de saída é lida — depois da
   iteração, de modo que o corpo sempre executa pelo menos uma vez e pode definir o que ela testa.
3. Quando a condição vale, o Laço sai por [[ui:exp.portDone]].
4. Caso contrário, a próxima iteração começa, enquanto houver alguma.
5. Quando as iterações acabam antes, o Laço sai por
   [[ui:exp.portLimit]] se ela tem fio, e reprova a execução com `loop.limit`
   se não tem. Sem uma condição, o corpo executa todas as iterações e o
   Laço sai por [[ui:exp.portDone]].

Dentro do corpo, `{{counter}}` é o número da iteração, já que cada nó
conta as próprias execuções. A condição e as etapas depois de [[ui:exp.portDone]] ou [[ui:exp.portLimit]] podem
usar o que toda iteração do corpo define — um status que o corpo extrai, por
exemplo; o próprio corpo vê só o que era conhecido quando o Laço foi alcançado.

O modelo [[ui:exp.templatePoll]] pergunta a um dispositivo pelo status dele a cada 0,3 s
até que ele responda `ready`, no máximo 10 vezes.

### O que um corpo pode conter {#loop-body}

Um corpo executa como um ramo, uma iteração depois da outra. O fio de volta ao
Laço é o único ciclo que um experimento pode ter; qualquer outro é `graph.cycle`.

| Regra | Erro |
| --- | --- |
| Algo em [[ui:exp.portBody]] leva de volta ao Laço | `loop.no_return` |
| Cada saída no corpo tem um fio | `loop.body_parallel` |
| Toda saída no corpo segue adiante no corpo ou volta ao Laço | `loop.body_leaves` |
| Só a saída [[ui:exp.portBody]] do Laço leva ao corpo | `loop.body_entered` |
| Nenhum [[ui:exp.node.start]], [[ui:exp.node.end]], [[ui:exp.node.fork]], [[ui:exp.node.join]] nem outro [[ui:exp.node.loop]] no corpo | `loop.body_unsupported` |

## Esperas {#waits}

Uma espera é aprovada quando chega uma mensagem que ela aguarda:
[[ui:exp.node.wait_osc]], [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]],
[[ui:exp.node.wait_http]] e [[ui:exp.node.wait_ws]]. Com o que cada uma corresponde está
na [referência dos nós](nodes.md); aqui está como elas escutam.

### Escuta desde o início {#listening}

A execução abre aquilo em que as esperas escutam **antes da primeira etapa**, de modo que uma resposta
mais rápida que a etapa seguinte não se perde:

| Espera | Aberto antes da primeira etapa |
| --- | --- |
| OSC, UDP | um socket UDP por endereço [[ui:exp.listenOn]], compartilhado por toda espera nele |
| Requisição HTTP | um ouvinte por endereço — o [emulador](faults.md#emulator) HTTP da execução quando há um ali, senão um que responde `204` |
| MQTT | uma conexão por broker e filtro de tópicos, assinada; as mensagens retidas que o broker reenvia então são ignoradas |
| WebSocket | nada: ela lê a conexão que um [[ui:exp.node.ws_connect]] abriu quando executou |

Como abrem primeiro, esses endereços são fixados antes da execução: uma espera OSC,
UDP ou HTTP escuta em um `IP:port` literal com uma porta diferente de 0, e
o broker e o tópico de uma espera MQTT aceitam só parâmetros. Uma porta que não pode ser
aberta — ocupada, ou que não é um endereço deste computador — interrompe a execução antes de qualquer
tráfego, no campo dessa espera. Tudo é fechado quando a execução termina, de qualquer
jeito.

### Quais mensagens contam {#counting}

Uma espera considera as mensagens que chegaram depois que a **última ação no
ramo dela** começou — a última requisição, mensagem, publicação, conexão WebSocket ou
envio — ou, antes de qualquer ação, depois que a execução começou. Uma mensagem de antes da
requisição não conta, e um Atraso ou um Log entre a requisição e a espera
não esconde a resposta dela. Depois de uma União, vale a mais antiga das últimas ações dos
ramos mesclados.

Uma espera toma a primeira mensagem que corresponde e a consome: duas esperas nunca
correspondem à mesma mensagem.

Cada socket, assinatura ou conexão guarda no máximo 1024 mensagens e 64 MiB
para as esperas dele; além disso, as mais antigas são descartadas e contadas.

### Tempo esgotado {#timeout}

[[ui:exp.waitTimeout]] é de 1–120.000 ms, 2000 no início. Quando nada corresponde a
tempo:

- com um fio [[ui:exp.portTimeout]], a espera o segue;
- sem um, a etapa falha com `wait.timeout`, que diz quantas outras
  mensagens chegaram nesse meio-tempo — um padrão errado tem outra cara que um dispositivo
  mudo — e, no detalhe, quantas mensagens mais antigas foram descartadas quando a
  fila estava cheia.

A variável da espera — `reply`, ou `request` para HTTP — só existe depois de
[[ui:exp.portMatched]]. Quando o [[ui:dock.inspector]] está capturando, a etapa
também liga ao quadro com que correspondeu: veja [a linha do tempo](runs.md#timeline).

## Uma resposta na mesma etapa {#reply}

Um [[ui:exp.node.osc]] ou um [[ui:exp.node.udp]] pode aguardar a própria resposta:
ative [[ui:exp.expectReply]].

| Configuração | O quê | Valor inicial |
| --- | --- | --- |
| [[ui:exp.replyOn]] | o endereço em que a resposta é aguardada; a porta 0 é qualquer porta livre | `0.0.0.0:0` |
| [[ui:exp.replyAddress]] (OSC), [[ui:exp.replyMode]] (UDP) | o que a resposta deve ser, como na espera correspondente | qualquer |
| [[ui:exp.waitTimeout]] | 1–120.000 ms | 2000 |
| [[ui:exp.replyVariable]] | a variável em que a resposta é escrita | `reply` |

O socket em [[ui:exp.replyOn]] é aberto antes da primeira etapa, como o de uma
espera, e a mensagem **sai dele**: um dispositivo que responde à porta do
próprio remetente é ouvido, e um que responde a uma porta fixa é ouvido quando
essa porta é a dada. A etapa é aprovada com uma resposta correspondente, e a
variável existe depois da saída dela. Não há saída [[ui:exp.portTimeout]]: nenhuma resposta a
tempo reprova a etapa, e a Nova tentativa pode enviar de novo. Para desviar pelo silêncio, use uma
espera separada.

## O que é verificado antes de uma execução {#validation}

O editor verifica o experimento enquanto você o edita; o botão Executar o verifica mais
uma vez. Um problema nomeia o nó e, quando há, o campo.

| Regra | Erro |
| --- | --- |
| O experimento tem um nome | `doc.name_required` |
| 1–64 nós, exatamente um Início e um Fim | `doc.node_count`, `doc.start_end_count` |
| O Início não tem entrada | `graph.start_input` |
| Um fio leva a outro nó que existe | `doc.connection_invalid` |
| O mesmo fio não está lá duas vezes | `doc.connection_duplicate` |
| Toda saída que precisa de fio tem um | `graph.outputs_required` |
| Um nó não tem fio em uma saída que ele não tem | `graph.port_unexpected` |
| Todo nó pode ser alcançado a partir do Início | `graph.unreachable` |
| Nenhum ciclo além do fio de volta de um Laço | `graph.cycle`, e as [regras do corpo](#loop-body) |
| Uma verificação ou um Extrair tem uma requisição HTTP antes dele em todos os caminhos; uma requisição sob carga não conta | `graph.needs_http` |
| Todo modelo é lido sem erro, e todo nome que ele usa é conhecido em todos os caminhos | `template.*`, `name.*` — veja [dados](data.md#unknown-names) |
| Todo campo está presente e dentro do intervalo | `node.*` |
| Um nó que nomeia outro — [[ui:exp.node.impairment_change]], [[ui:exp.node.emulator_state]], os nós WebSocket — nomeia um que existe, e um nó WebSocket vem depois do connect dele | `impair.relay_unknown`, `emulator.node_unknown`, `ws.connection_unknown`, `ws.connection_after` |
| Dois sockets da execução não compartilham uma porta | veja [falhas](faults.md#ports) |

A execução então verifica o que precisa para começar: todo [segredo](data.md#secret-check)
guardado, toda porta aberta. Enquanto nem tudo vale, nenhuma etapa executa e nada é
enviado.

## Limite de tempo {#limit}

Uma execução dura no máximo 300 s. Uma que ainda esteja em andamento então é parada e falha com
`run.timeout`. Pela [linha de comando](../automation/cli.md#cli-run) e
pela [API](../api/run.md), o limite pode ser menor, de 1–300 s.

## Parar {#stop}

Enquanto uma execução está em andamento, o botão Executar é [[ui:common.stop]]. Parar encerra a execução
na hora: todos os ramos, todas as pausas de Nova tentativa ou Repetição, todas as esperas e todas as cargas —
as requisições em andamento são descartadas. Os sockets, assinaturas, emuladores e
retransmissores dela se fecham, e as conexões WebSocket enviam um quadro de fechamento.
[[ui:app.stopAll]] no cabeçalho faz o mesmo com todas as tarefas. Uma execução parada
não salva relatório; veja [execuções](runs.md#stop).

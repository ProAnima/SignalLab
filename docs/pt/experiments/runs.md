---
title: Execuções e resultados
description: Iniciar uma execução com os valores do experimento ou outros, a linha do tempo, parar, o que significam aprovado e com falha, o relatório da execução, sementes, testar um nó e os arquivos de experimento com suas versões.
---

# Execuções e resultados

## Início de uma execução {#start}

Pressione [[ui:exp.run]] na barra de ferramentas do editor. Antes de qualquer coisa ser enviada:

1. O experimento é verificado como o editor o verifica — o grafo, os campos,
   os nomes e os valores ([o que é verificado](flow.md#validation)) — e todo segredo
   que ele usa precisa estar guardado ([segredos](data.md#secret-check)).
2. Ele é salvo.
3. A execução abre o que precisa para toda a sua duração: os emuladores dela, os
   retransmissores de degradação, os sockets em que as esperas escutam e as assinaturas
   MQTT.

Se algo disso falha, nada executa: o problema é mostrado, e o nó a que ele
se refere é selecionado. Caso contrário, a linha do tempo abre sob o canvas e as etapas
aparecem nela conforme acontecem. Enquanto a execução está em andamento, [[ui:exp.run]] vira
[[ui:common.stop]] e o experimento não pode ser editado.

A execução usa os valores do perfil ativo e a semente fixada no
experimento ou uma nova. Para executar uma vez com outros, use [[ui:exp.runWith]].

### Execução com outros valores {#run-with}

O ▾ ao lado de [[ui:exp.run]] abre [[ui:exp.runWith]]: outros valores para uma
execução, sem alterar o experimento.

| Campo | O quê | Vazio |
| --- | --- | --- |
| [[ui:exp.profile]] | o perfil desta execução; aparece quando o experimento tem perfis | o ativo |
| cada parâmetro | um valor só para esta execução | o valor do perfil escolhido, mostrado em cinza |
| [[ui:exp.seed]] | a semente desta execução, 0–9.007.199.254.740.991; o botão ao lado preenche a semente da última execução | a semente fixada, ou uma nova |

[[ui:exp.run]] no formulário inicia a execução; [[ui:exp.resetOverrides]] esvazia o
formulário. O que você digitou fica no formulário durante a sessão, de modo que a mesma alteração
é um clique na próxima vez. Um perfil que não executaria é marcado com ⚠. A
precedência dos valores está em [dados](data.md#precedence).

Quando o experimento tem perfis, ou uma execução teve valores digitados para ela, a
linha do tempo diz qual perfil a execução usou — ou [[ui:exp.runDefaults]] — e,
quando valores foram digitados, [[ui:exp.overridden]].

## A linha do tempo {#timeline}

A [[ui:exp.timeline]] fica sob o canvas; ▸ e ▾ a recolhem, a borda dela a
redimensiona. Ela tem uma linha por evento de etapa, da mais antiga à mais nova: a hora, o nó, o
estado dele e o que aconteceu — `HTTP 200 · 41 ms`, `token = abc123`,
`/pong 42 ← 127.0.0.1:9000 · 12 ms`. Uma falha diz por quê, e o detalhe técnico dela
fica na dica. Um clique em uma linha seleciona o nó dela no canvas.

| Estado | A etapa |
| --- | --- |
| [[ui:exp.running]] | começou |
| [[ui:exp.passed]] | terminou bem e escolheu a saída dela |
| [[ui:exp.failed]] | falhou; a primeira falha é a da execução |
| [[ui:exp.retry]] | falhou em uma tentativa e vai tentar de novo ([Nova tentativa](flow.md#retry)) |
| [[ui:exp.repeating]] | está enviando de novo e de novo, no máximo uma linha por segundo ([Repetição](flow.md#repeat)) |
| [[ui:exp.load]] | está sob carga, uma linha por segundo ([carga](load.md#progress)) |

No canvas, cada nó traz um distintivo com o último estado dele.

**Execuções longas.** Uma execução mostra as primeiras 5.000 etapas uma a uma. Depois delas — um Laço
de mil iterações sobre um corpo longo produz mais de cem mil — ela
mostra uma etapa por segundo e toda falha, e quando termina, as últimas 500. Uma
linha em âmbar diz quantas foram omitidas entre duas outras, e a contagem da
linha do tempo recolhida é todas as etapas que a execução fez. O relatório guarda as mesmas etapas.

O cabeçalho da linha do tempo tem:

- o resultado: [[ui:exp.running]], [[ui:exp.passed]], [[ui:exp.failed]] com o
  motivo, ou [[ui:exp.stopped]];
- [[ui:exp.reportSaved]] quando o relatório é escrito — em um navegador, um link que
  o baixa; no aplicativo de desktop, o caminho dele na dica;
- [[ui:exp.compare]], para pôr esta execução ao lado de uma anterior
  ([comparação de execuções](load.md#compare));
- o perfil e os valores alterados, como acima;
- a semente da execução com [[ui:exp.pinSeed]] ou, quando o experimento tem uma
  fixada, essa semente com [[ui:exp.unpinSeed]] ([sementes](#seeds)).

**Quadros.** Enquanto o [[ui:dock.inspector]] está capturando, uma espera — ou um envio
que aguarda a resposta — que correspondeu a uma mensagem guarda o número do quadro dessa
mensagem. Um botão sob as linhas nomeia o nó e o quadro; ele
abre o Inspetor no painel inferior com esse quadro selecionado. Veja o
[Inspetor](../tools/inspector.md).

A linha do tempo mostra a última execução do experimento nesta sessão; ela é
esvaziada quando outro experimento é aberto.

## Parar {#stop}

Pressione [[ui:common.stop]], ou [[ui:app.stopAll]] no cabeçalho para todas as tarefas de
uma vez. A execução termina na hora ([o que para](flow.md#stop)), a linha do tempo mostra
[[ui:exp.stopped]] e **nenhum relatório é salvo**. Uma execução iniciada pela linha de
comando ou pela API em um servidor é uma tarefa como qualquer outra: [[ui:app.stopAll]] nesse
servidor também a para, e quem a chamou fica sabendo que foi parada.

## O resultado {#result}

| Resultado | Significa | Relatório |
| --- | --- | --- |
| [[ui:exp.passed]] | todos os ramos terminaram, nenhuma etapa falhou e o Fim foi alcançado | salvo |
| [[ui:exp.failed]] | uma etapa falhou — uma verificação, uma espera sem fio [[ui:exp.portTimeout]], um erro de rede, um limiar — ou a execução ficou sem tempo (`run.timeout`), uma União esperou em vão (`run.join_waiting`) ou nenhum ramo chegou ao Fim (`run.no_end`) | salvo, com a primeira falha |
| [[ui:exp.stopped]] | alguém a parou | nenhum |
| não começou | o experimento é inválido, falta um segredo ou uma porta não pôde ser aberta | nenhum |

Uma execução com falha nomeia o nó e o campo da primeira falha dela; a
[referência de erros](../reference/errors.md) lista todos os códigos. A linha de comando
diz o mesmo com o código de saída: `0` aprovada, `1` com falha, `2` o experimento ou
a chamada era inválida (um segredo ausente conta), `3` algo fora do
experimento a impediu de executar, como uma porta que não pôde ser aberta. Veja
[`signallab run`](../automation/cli.md#cli-run).

## O relatório da execução {#report}

Toda execução que termina sozinha — aprovada ou com falha — escreve um relatório JSON na
pasta `runs` da pasta de dados: `Documents/SignalLab/runs` em um desktop, a
própria pasta de dados do servidor em um servidor ([arquivos](../reference/files.md)). O arquivo
se chama `run-<start time in ms>-<job number>.json`; um relatório nunca é escrito sobre
outro. Se ele não pode ser escrito, o editor diz por quê.

| Chave | O quê |
| --- | --- |
| `version` | o formato do relatório, atualmente 6 |
| `experiment` | o nome do experimento |
| `document_version` | a versão do experimento, atualmente 9 |
| `seed` | a semente que a execução usou |
| `profile` | o perfil com que executou, ou `null` para os padrões |
| `overrides` | os valores digitados em [[ui:exp.runWith]] |
| `params` | todo valor de parâmetro que a execução usou |
| `started_ms`, `ended_ms` | milissegundos Unix |
| `outcome` | `passed` ou `failed` |
| `error` | a primeira falha, ou `null` |
| `steps` | os eventos de etapa, em ordem (abaixo): os primeiros 5.000, depois um por segundo e toda falha, depois os últimos 500 |
| `emulators` | as contagens de cada nó Emulador — presente quando há um ([emuladores](faults.md#emulator)) |
| `impairments` | as contagens e fases de cada nó Degradação — presente quando há um ([fases](faults.md#change-impairment)) |

Cada evento de etapa tem:

| Chave | O quê |
| --- | --- |
| `job_id`, `node_id` | a execução e o nó |
| `ts` | milissegundos Unix |
| `state` | `running`, `passed`, `failed`, `retry`, `repeating`, `load` |
| `detail` | o que aconteceu, em inglês |
| `message_key`, `message_params` | o mesmo que o texto da interface e os valores dele, de modo que a etapa pode ser mostrada em qualquer idioma |
| `vars` | as variáveis que a etapa escreveu, se houver |
| `error` | por que falhou: `code`, `params`, `node`, `field`, `detail` |
| `frame` | o quadro do Inspetor com que uma espera correspondeu, se a captura estava ativa |
| `load` | o que uma carga mediu ([métricas](load.md#metrics)), no último evento dela |
| `omitted` | quantas etapas logo antes desta foram omitidas, quando algumas foram |

Valores de segredos nunca aparecem em um relatório: são mascarados como `••••`
([mascaramento](data.md#masking)).

O formato do relatório cresceu com os recursos: a versão 3 acrescentou as contagens dos
emuladores, a versão 4 as fases das degradações, a versão 5 as medições de uma carga,
a versão 6 as etapas de execuções longas como uma amostra com `omitted`.

Os relatórios são o histórico de execuções: [[ui:exp.compare]] os lê, e
[`experiment_runs`](../api/commands.md#experiment_runs) também. A opção
`--report` da linha de comando copia o relatório de uma execução para onde você quiser.

## Sementes {#seeds}

Toda execução tem uma semente, um número inteiro de 0 a 9.007.199.254.740.991. Ela é,
em ordem:

1. a semente dada a esta execução em [[ui:exp.runWith]], na linha de comando
   (`--seed`) ou à API;
2. a semente fixada no experimento;
3. uma nova semente aleatória.

A primeira linha da execução na linha do tempo a mostra, e o relatório a guarda.

A semente decide tudo o que uma execução faz de aleatório: os
[geradores](data.md#generators) nos modelos, o jitter da
[Repetição](flow.md#repeat), as chegadas de uma [carga aleatória](load.md#schedule), o
destino de cada pacote em um [retransmissor de degradação](faults.md#seed) e as
escolhas aleatórias de um emulador. Cada um sorteia de um fluxo próprio, de modo que ramos paralelos
nunca alteram os valores uns dos outros.

Para repetir uma execução:

1. Pressione [[ui:exp.pinSeed]] ao lado da semente dela na linha do tempo. A semente é guardada
   no experimento, e toda execução a usa até você pressionar
   [[ui:exp.unpinSeed]]. Em [[ui:exp.params]], [[ui:exp.seed]] mostra e edita
   a semente fixada; vazia, ela é [[ui:exp.seedRandom]].
2. Execute com o mesmo perfil e os mesmos valores; o relatório os lista.

O que uma semente não pode repetir: o horário (`{{now}}`), `{{run.id}}` e quando os dispositivos
e a rede respondem.

## Teste de um nó {#send-now}

Para testar um nó sem executar o experimento, selecione-o e pressione
[[ui:exp.sendNow]] — ou <kbd>Ctrl</kbd>+<kbd>Enter</kbd> nas propriedades dele —
em um nó [[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]],
[[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_connect]] ou
[[ui:exp.node.ws_send]]. Em uma espera, é [[ui:exp.listenNow]]: ela escuta a partir de agora até que uma mensagem
corresponda ou o tempo limite dela acabe.

O motor executa o nó com o código que uma execução usa, uma vez:

- com os valores do perfil ativo, os valores de variável conhecidos nesta sessão
  (da última execução e de testes anteriores) e os segredos guardados;
- com a semente fixada, ou uma nova; `{{run.id}}` é `0` e `{{counter}}`
  é `1`;
- sem Nova tentativa, Repetição nem carga — um envio;
- **sem cookies**: uma requisição, nada definido antes dela para enviar de volta;
- sem os retransmissores e emuladores da execução. Um [[ui:exp.node.wait_http]] escuta
  em um ouvinte próprio, e um envio ou espera WebSocket abre a conexão que o
  [[ui:exp.node.ws_connect]] dele descreve, para esse único teste.

Se um nome que o nó usa ainda não tem valor, nada é enviado e o resultado diz
quais nomes faltam — execute o experimento, ou use antes [[ui:exp.sendNow]] no
nó que os define.

O resultado mostra ✓ ou ✕ e o que aconteceu. Para uma requisição HTTP, ele também mostra
o status, o tempo, o tamanho e a [[ui:http.response]]; em uma resposta JSON,
cada valor pode ser clicado para [extraí-lo](data.md#extract), e
[[ui:http.mockThis]] transforma a resposta em uma rota de um emulador. Os
valores que uma espera recebeu, ou que os nós [[ui:exp.node.extract]] logo depois de uma
requisição tomariam da resposta dela, passam a ser conhecidos da prévia e do
próximo [[ui:exp.sendNow]]. Enquanto uma execução está em andamento, [[ui:exp.sendNow]] não está
disponível.

A prévia de um nó com modelo — o que ele [enviará](data.md#preview) — também é
resolvida pelo motor, sem enviar nada.

## Arquivos de experimento {#files}

### O experimento de trabalho {#working-file}

O editor guarda um experimento, salvo sozinho 0,7 s depois de cada alteração em
`experiment.json` na pasta de dados; a barra de ferramentas diz [[ui:exp.saving]],
[[ui:exp.saved]] ou [[ui:exp.saveError]]. Um grafo inacabado também é salvo. Um arquivo
que não pode ser lido é informado com o caminho dele, nunca substituído. Um arquivo de experimento
tem no máximo 4 MiB.

Em um servidor, o arquivo fica na pasta de dados do servidor, de modo que todo navegador que
abre o editor ali trabalha no mesmo experimento.

### Abertura, modelos e exportação {#open-export}

O botão ☰ na barra de ferramentas abre [[ui:exp.documents]]:

- [[ui:exp.templates]]: [[ui:exp.templateEmpty]], [[ui:exp.templateHttp]],
  [[ui:exp.templateBranch]], [[ui:exp.templateParallel]],
  [[ui:exp.templatePingReply]], [[ui:exp.templatePoll]],
  [[ui:exp.templateFlaky]], [[ui:exp.templateFaults]],
  [[ui:exp.templateOutage]], [[ui:exp.templateWsEcho]]. Os destinos deles ficam em
  `127.0.0.1`.
- [[ui:exp.importJson]] lê um arquivo de até 4 MiB — desta versão do formato de
  experimento ou de uma anterior, que é atualizada ao abrir — e o
  verifica antes de mostrar o nome e quantos nós e conexões ele
  tem. O arquivo também precisa caber em 4 MiB como o editor o escreve, indentado, então um
  arquivo compacto perto do limite pode ser recusado. Um arquivo quebrado é recusado com
  a linha e a coluna do problema, um arquivo de um Signal Lab mais novo com
  `doc.version_unsupported`, e o experimento atual permanece.
- [[ui:exp.openDocument]] substitui o experimento atual pelo escolhido.
  <kbd>Ctrl</kbd>+<kbd>Z</kbd> traz o anterior de volta durante esta
  sessão. Abrir um experimento não o executa.
- [[ui:exp.exportJson]] escreve uma cópia na pasta `exports` da pasta de
  dados, como `experiment-<time in ms>-<random>.json`, nunca sobre outra cópia;
  em um navegador, [[ui:common.download]] a baixa.

A linha de comando e a API aceitam os mesmos arquivos, e os modelos pelo nome:
`empty`, `http-check`, `status-branch`, `parallel-flows`, `osc-ping-reply`,
`poll-until-ready`, `flaky-api`, `fault-phases`, `dependency-outage`,
`websocket-echo`.

### Versões do documento {#versions}

Um arquivo de experimento tem uma `version`; este Signal Lab escreve a versão 9 e abre
todas as anteriores, preenchendo o que o arquivo mais antigo não podia guardar. Um arquivo de
uma versão mais nova que a 9 é recusado (`doc.version_unsupported`) em vez de aberto
sem o que ele contém.

| Versão | Acrescentou |
| --- | --- |
| 2 | parâmetros e a semente |
| 3 | perfis |
| 4 | Nova tentativa, e uma resposta aguardada por um envio OSC ou UDP |
| 5 | Repetição, e [[ui:exp.node.loop]] |
| 6 | [[ui:exp.node.emulator]] e [[ui:exp.node.wait_http]] |
| 7 | [[ui:exp.node.impairment]], [[ui:exp.node.impairment_change]] e [[ui:exp.node.emulator_state]] |
| 8 | os nós WebSocket, a autenticação HTTP e o armazenamento de cookies |
| 9 | carga em uma requisição HTTP e degradação sobre TCP |

Um arquivo anterior à versão 8 abre com [[ui:exp.cookies]] desligado, de modo que executa como
executava; um arquivo mais novo mantém a própria configuração. Salvo de novo, qualquer arquivo vira a versão
9.

## Pela linha de comando ou por um servidor {#automation}

Uma execução é a mesma em todo lugar: a linha de comando e a API do servidor iniciam a
mesma execução que o editor, com as mesmas etapas, o mesmo resultado e o mesmo relatório.

```bash
signallab run checkout.json --profile Stage -p api=http://192.0.2.10:8080 --seed 42 --report report.json
```

- [`signallab run`](../automation/cli.md#cli-run) executa arquivos de experimento ou
  modelos neste processo ou em um servidor, imprime as etapas como a linha do tempo
  e sai com o código do resultado.
- [`POST /api/run`](../api/run.md) executa um em um servidor e responde com o
  resultado, ou transmite as etapas dele conforme acontecem. Um cliente que some não
  para a execução; ela executa até o fim e guarda o relatório.
- Em CI: [GitHub Actions e outros](../automation/ci.md).

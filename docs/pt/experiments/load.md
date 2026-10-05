---
title: Testes de carga
description: Envie a requisição de um nó HTTP segundo um perfil de carga — constante, rampa, degraus, pico ou chegadas aleatórias —, meça latências, erros e a taxa alcançada, julgue-os com limiares e compare duas execuções.
---

# Teste de carga de uma requisição HTTP

Um nó [[ui:exp.node.http]] pode enviar a sua requisição muitas e muitas vezes, segundo um perfil de
requisições por segundo, muitas ao mesmo tempo — e medir o que volta: percentis de latência,
erros, a taxa que alcançou. Limiares decidem se a etapa passa, e [[ui:exp.compare]] coloca os
números ao lado dos de uma execução anterior.

Uma carga é uma configuração do nó, não um nó à parte: o restante do experimento — emuladores,
retransmissores de degradação, outros ramos — roda em volta dela como de costume.

## Colocar uma requisição sob carga {#turn-on}

1. Selecione um nó [[ui:exp.node.http]] e preencha a requisição dele.
2. Nas propriedades dele, marque [[ui:exp.loadOn]].
3. Escolha um [[ui:exp.loadShape]] e os números dele. O gráfico abaixo deles,
   [[ui:exp.loadChart]], desenha a taxa e diz quantas requisições ela soma, em quantos segundos.
4. Defina o campo [[ui:exp.loadConcurrency]] — quantas requisições podem estar em andamento ao mesmo
   tempo.
5. Acrescente ou altere [[ui:exp.thresholds]].
6. Execute o experimento.

Uma carga começa como uma [[ui:exp.loadShape.ramp]] de 0 a 100 requisições por segundo em
30.000 ms, 32 simultâneas, com dois limiares: [[ui:exp.metric.p95_ms]] < 500 ms e
[[ui:exp.metric.error_rate]] < 1%.

**A carga substitui a Repetição e a Nova tentativa.** Ativá-la desativa as duas, e um nó com carga
e qualquer uma delas é recusado (`node.load_alone`): uma requisição que falha é contada, não
tentada de novo. Só uma requisição HTTP pode rodar sob carga (`node.load_unsupported`).

**A requisição é lida uma vez.** Os modelos dela são resolvidos quando a etapa começa, então todas
as requisições da carga são a mesma: `{{counter}}` e `{{uuid}}` têm um único valor para todas.
Veja [modelos](data.md#templates).

**Um único cliente para a carga inteira.** As requisições compartilham o armazenamento de cookies
da execução quando a opção [[ui:exp.cookies]] está ativada, e uma única memória Digest, de modo que um só
desafio vale para todas. Cada requisição tem o timeout do próprio nó.

**O Inspetor recebe uma amostra:** no máximo uma troca a cada 100 ms, para que uma carga não
inunde o [[ui:dock.inspector]].

## Perfis {#profiles}

| [[ui:exp.loadShape]] | Configurações | A taxa ao longo do tempo |
| --- | --- | --- |
| [[ui:exp.loadShape.constant]] | [[ui:exp.loadRate]], [[ui:exp.loadDuration]] | a taxa o tempo todo |
| [[ui:exp.loadShape.ramp]] | [[ui:exp.loadFrom]], [[ui:exp.loadTo]], [[ui:exp.loadDuration]] | em linha reta de uma taxa à outra |
| [[ui:exp.loadShape.steps]] | [[ui:exp.loadFrom]], [[ui:exp.loadStepBy]], [[ui:exp.loadEvery]], [[ui:exp.loadSteps]] | a primeira taxa e depois um degrau a mais em cada nível, cada nível pelo mesmo tempo |
| [[ui:exp.loadShape.spike]] | [[ui:exp.loadBase]], [[ui:exp.loadPeak]], [[ui:exp.loadAt]], [[ui:exp.loadSpikeFor]], [[ui:exp.loadDuration]] | a taxa base, o pico por um tempo a partir de um dado momento e depois a taxa base de novo |
| [[ui:exp.loadShape.poisson]] | [[ui:exp.loadRate]], [[ui:exp.loadDuration]] | chegadas ao acaso, com a taxa em média |

Trocar o perfil mantém o que pode ser aproveitado: quanto tempo ele dura e a maior taxa que
alcança.

### Limites {#limits}

| Configuração | Intervalo |
| --- | --- |
| [[ui:exp.loadRate]] de [[ui:exp.loadShape.constant]] e [[ui:exp.loadShape.poisson]], [[ui:exp.loadPeak]] | 0,1–100.000 requisições/s |
| [[ui:exp.loadFrom]], [[ui:exp.loadTo]], [[ui:exp.loadBase]] | 0–100.000 requisições/s |
| Cada nível de [[ui:exp.loadShape.steps]], incluindo o último | 0–100.000 requisições/s; o degrau pode ser negativo |
| [[ui:exp.loadDuration]], [[ui:exp.loadEvery]] | 100–300.000 ms |
| [[ui:exp.loadSteps]] | 1–100, e todos os níveis juntos no máximo 300.000 ms |
| Um pico | mais de 0 ms, e encerrado até o fim da duração |
| [[ui:exp.loadConcurrency]] | 1–512 |
| [[ui:exp.thresholds]] | no máximo 16, cada valor um número, 0 ou mais |

Um perfil que não soma requisição nenhuma é recusado (`load.nothing_planned`). As taxas são as da
[rajada HTTP](../protocols/http.md).

Um perfil pode durar tanto quanto uma execução inteira, 300 s — mas o
[limite de tempo](flow.md#limit) da execução conta todas as etapas, então deixe espaço para o resto
do experimento.

### Quantas requisições {#planned}

As requisições de um perfil são a taxa dele somada ao longo do tempo:

| Perfil | Requisições |
| --- | --- |
| [[ui:exp.loadShape.constant]], 100/s por 1.000 ms | 100 |
| [[ui:exp.loadShape.ramp]], 0 → 100/s em 2.000 ms | 100 |
| [[ui:exp.loadShape.steps]], de 10/s subindo 10/s, 3 níveis de 1.000 ms | 60 (10 + 20 + 30) |
| [[ui:exp.loadShape.spike]], 10/s com 100/s a partir de 1.000 ms por 500 ms, 2.000 ms ao todo | 65 |
| [[ui:exp.loadShape.poisson]], 200/s por 10.000 ms | 2.000 em média |

## O cronograma {#schedule}

A n-ésima requisição tem como hora o momento em que a contagem do perfil chega a n — a primeira,
imediatamente. Cada momento é calculado a partir do início da carga, então um despertar atrasado
nunca desloca as requisições seguintes, e a taxa que o perfil descreve é a taxa pedida.

[[ui:exp.loadShape.poisson]] sorteia os intervalos entre as chegadas a partir da semente da
execução: a mesma semente dá os mesmos momentos, então uma carga aleatória pode ser repetida
exatamente. Veja [sementes](runs.md#seeds).

**Requisições puladas.** Ficam em andamento no máximo tantas requisições quanto diz o campo
[[ui:exp.loadConcurrency]]. Quando todas elas ainda estão esperando a resposta, a requisição seguinte espera um lugar livre.
Se ela fosse sair mais de 50 ms depois do seu momento, não é enviada atrasada: é pulada e entra
na contagem de puladas, junto com todas as outras requisições cuja hora chegou nesse meio-tempo, e a carga
continua com a primeira que ainda está no horário. Muitas requisições puladas significam que o
servidor, ou o valor de [[ui:exp.loadConcurrency]], não conseguiu acompanhar o perfil.

## Durante a execução {#progress}

Uma vez por segundo, a linha do tempo mostra a etapa como [[ui:exp.load]], com os segundos
decorridos, as requisições enviadas, a taxa no último segundo, o p95 até ali e as requisições com
falha. O botão [[ui:common.stop]] encerra a carga na hora e descarta as requisições em andamento; uma falha
em outro ramo a encerra em até um segundo.

## O que é medido {#metrics}

Depois da última resposta, a etapa tem as suas medições, guardadas no último evento dela na linha
do tempo e no [relatório da execução](runs.md#report):

| Medição | O quê |
| --- | --- |
| planned | as requisições que o perfil soma ([[ui:exp.loadShape.poisson]]: em média) |
| sent | requisições que foram respondidas ou falharam |
| ok | respondidas com um status 2xx |
| failed | qualquer outro status, ou nenhuma resposta |
| missed | com hora marcada enquanto todos os lugares estavam ocupados, e puladas |
| rps | requisições enviadas por segundo: sent ÷ a duração do perfil — ou ÷ o tempo até a última requisição sair, quando isso foi mais tarde |
| error_rate | failed, em % de sent |
| min, mean, max | a requisição mais rápida, a média e a mais lenta, em ms |
| p50, p90, p95, p99 | a latência em que ou abaixo da qual ficaram 50, 90, 95 e 99% das requisições, em ms |
| received_bytes | bytes de corpo recebidos ao todo |
| statuses | requisições por status (`200`, `503`) e, sem status, por causa (`timeout`, `refused`, `reset` …) |
| seconds | cada segundo do perfil: requisições enviadas, com falha, a latência média delas |
| histogram | requisições por latência, até 1, 2, 5, 10, 20, 50, 100, 200, 500, 1.000, 2.000, 5.000, 10.000 ms, e mais lentas |

A latência de uma requisição vai do envio até a leitura da resposta inteira, e uma requisição que
falha conta com o tempo que levou para falhar. Os percentis são lidos de faixas logarítmicas de 1%
de largura e ficam a menos de 0,5% do valor real, por mais que a carga dure.

## Limiares {#thresholds}

Um limiar é uma linha com [[ui:exp.thresholdMetric]], [[ui:exp.thresholdOp]] e
[[ui:exp.thresholdValue]]; o botão [[ui:exp.thresholdAdd]] acrescenta um.

| [[ui:exp.thresholdMetric]] | Medida em |
| --- | --- |
| [[ui:exp.metric.p50_ms]], [[ui:exp.metric.p90_ms]], [[ui:exp.metric.p95_ms]], [[ui:exp.metric.p99_ms]] | ms |
| [[ui:exp.metric.mean_ms]], [[ui:exp.metric.max_ms]] | ms |
| [[ui:exp.metric.error_rate]] | % das requisições enviadas |
| [[ui:exp.metric.rps]] | requisições por segundo alcançadas |
| [[ui:exp.metric.missed]] | requisições |

A [[ui:exp.thresholdOp]] é uma de `<`, `≤`, `>`, `≥`. Alguns comuns:

| [[ui:exp.thresholdMetric]] | [[ui:exp.thresholdOp]] | [[ui:exp.thresholdValue]] | A etapa falha quando |
| --- | --- | --- | --- |
| [[ui:exp.metric.p95_ms]] | `<` | 300 | uma requisição em cada vinte, ou mais, levou 300 ms ou mais |
| [[ui:exp.metric.error_rate]] | `<` | 1 | 1% ou mais das requisições falharam |
| [[ui:exp.metric.rps]] | `≥` | 180 | o servidor não conseguiu receber 180 requisições por segundo |
| [[ui:exp.metric.missed]] | `≤` | 0 | uma única requisição precisou ser pulada |

Em um arquivo, um limiar é `{ "metric": "p95_ms", "op": "lt", "value": 300 }`; as métricas são
`p50_ms`, `p90_ms`, `p95_ms`, `p99_ms`, `mean_ms`, `max_ms`, `error_rate`, `rps` e `missed`, e as
comparações, `lt`, `le`, `gt` e `ge`.

Os limiares são lidos depois da última resposta, na ordem deles. A etapa falha no primeiro que não
se cumprir (`load.threshold`), com uma mensagem que traz o limiar e o valor medido, e a execução
falha junto. Sem limiares, uma carga passa seja o que for que tenha medido. Quando a falha de
outro ramo encerrou a carga antes da hora, essa falha é a da execução, não um limiar.

## O resultado {#result}

Quando a etapa passa, a linha do tempo a resume: as requisições, a taxa, o p95 e a parcela que
falhou. Selecione o nó: as propriedades dele mostram [[ui:exp.loadResult]] —

- cada limiar, ✓ [[ui:exp.thresholdHeld]] ou ✕ [[ui:exp.thresholdBroken]], com o valor medido;
- [[ui:http.sent]], [[ui:exp.loadRps]], [[ui:exp.loadErrors]] com a sua parcela,
  [[ui:http.missed]];
- [[ui:http.p50]], [[ui:http.p90]], [[ui:http.p95]], [[ui:http.p99]], [[ui:http.avg]],
  [[ui:http.max]];
- [[ui:exp.loadPerSecond]]: as requisições de cada segundo, as que falharam em vermelho, e a
  latência média delas como uma linha;
- [[ui:exp.loadLatencies]]: quantas requisições levaram quanto tempo;
- os status e as causas, cada um com a sua contagem.

A linha de comando imprime os mesmos números e o veredito de cada limiar; veja
[`signallab run`](../automation/cli.md#cli-run).

## Comparar duas execuções {#compare}

1. Execute o experimento duas vezes, ou mais.
2. Na linha do tempo, pressione [[ui:exp.compare]]. O botão aparece assim que uma execução salvou
   o relatório e fica desativado enquanto uma execução está em andamento.
3. A execução mais recente é [[ui:exp.compareAfter]]; a anterior a ela, [[ui:exp.compareBefore]];
   qualquer uma das listas escolhe outra execução.

As listas contêm as execuções deste experimento — pelo nome dele — a partir dos relatórios na
pasta de dados, as mais recentes primeiro, no máximo 50: cada uma com a data e a hora, como
terminou e a semente. As execuções da linha de comando também aparecem quando ela usou a mesma
pasta de dados. Renomear o experimento começa um histórico novo.

Para cada etapa com carga, pareada pelo nó, uma tabela mostra cada métrica em
[[ui:exp.compareBefore]], em [[ui:exp.compareAfter]] e a [[ui:exp.compareChange]], na unidade e em
%. Uma mudança para o lado ruim de 5% ou mais — mais lenta, mais erros, mais requisições puladas,
uma taxa menor — é uma regressão e aparece em vermelho; passar de nada para alguma coisa também
conta. Abaixo da tabela, o veredito de cada limiar nas duas execuções. Uma etapa com carga que só
uma das execuções tem é marcada como [[ui:exp.compareOnlyBefore]] ou [[ui:exp.compareOnlyAfter]],
sem variações. Execuções sem etapas com carga mostram [[ui:exp.compareNoLoad]].

Em um script, [`experiment_runs`](../api/commands.md#experiment_runs) lista as execuções e
[`experiment_compare`](../api/commands.md#experiment_compare) compara duas, pelo nome do arquivo
do relatório; `signallab mcp` oferece o mesmo a um assistente ([MCP](../automation/mcp.md)).

## Verificações depois de uma carga {#checks-after}

Uma carga não deixa resposta própria: ela é medida, não verificada. Uma verificação ou um
[[ui:exp.node.extract]] depois dela precisa de outra requisição sem carga antes, em todos os
caminhos, ou o experimento não é executado (`graph.needs_http`). Para verificar uma resposta da API
sob carga, coloque um [[ui:exp.node.http]] simples depois da carga, ou em um ramo paralelo ao lado
dela.

O botão [[ui:exp.sendNow]] em um nó sob carga envia a requisição dele uma vez.

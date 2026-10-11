---
title: Conceitos
description: As ideias por trás do Signal Lab — telas e experimentos, sinais, tarefas, captura, emuladores, retransmissores de degradação, parâmetros, modelos, segredos, sementes, relatórios e a pasta de dados.
---

# Conceitos

Esta página explica as ideias em que o Signal Lab se baseia, para que o resto da
documentação seja fácil de ler. Cada seção aponta para a página que trata do assunto por
completo.

## Telas e experimentos {#screens-and-experiments}

O Signal Lab tem duas formas de trabalhar, e você vai usar as duas.

- **Telas** são ferramentas para o trabalho que você faz agora, à mão: enviar esta
  mensagem, escutar nesta porta, iniciar este emulador, ver o que o broker guarda. Você
  tenta, observa, muda algo e tenta de novo. Cada protocolo e cada ferramenta tem uma
  tela; veja [A janela](interface.md).
- **Experimentos** são fluxos que você monta uma vez e executa de novo, sempre do mesmo
  jeito: enviar uma requisição, aguardar a resposta, verificá-la, seguir em frente ou
  desviar. Cada execução é relatada etapa por etapa e salva. Veja
  [Experimentos](../experiments/index.md).

As duas se encontram em vários pontos. Nas telas HTTP e OSC, [[ui:common.toExperiment]]
transforma o que você acabou de enviar na próxima etapa do experimento. No monitor OSC e
na tela MQTT, [[ui:osc.waitForThis]] transforma uma mensagem recebida em uma etapa que
a aguarda. E um sinal da biblioteca pode virar uma etapa — qualquer um, exceto bytes UDP
brutos escritos em hexadecimal.

## Sinais e a biblioteca {#signals}

Um **sinal** é uma mensagem que você guarda: um nome, uma pasta, uma nota sobre o que ela
deve provocar e o que ela envia — uma mensagem OSC, bytes UDP brutos, uma requisição HTTP
ou uma publicação MQTT. A **biblioteca de sinais** os reúne em pastas que você pode
aninhar, renomear e arrastar.

- Você salva um sinal nas telas OSC, HTTP e MQTT ([[ui:sig.saveNew]]), cria um na tela
  [[ui:nav.signals]], ou salva um quadro que o Inspetor capturou
  ([[ui:sig.fromFrame]]), que então o reproduz byte a byte.
- Você o envia pela tela [[ui:nav.signals]], de qualquer lugar com
  <kbd>Ctrl</kbd>+<kbd>K</kbd>, como etapa de um experimento ou com `signallab fire` em um
  terminal.
- Um sinal envia exatamente o que a sua tela enviaria: os mesmos bytes, pelo mesmo
  caminho, mostrados no Inspetor sob o protocolo real.

A biblioteca é um único arquivo, `signals.json`, na [pasta de dados](#data-folder): JSON
simples que você pode ler, editar, copiar para outra máquina ou guardar em um repositório.
Ela começa com um conjunto de sinais de exemplo, todos apontados para `127.0.0.1`. Veja
[Sinais](../tools/signals.md).

## Tarefas {#jobs}

Tudo o que continua em andamento depois de você pressionar o botão é uma **tarefa**: um
monitor ou gerador OSC, uma conexão com um broker ou WebSocket, um beacon ou um ouvinte de
descoberta, uma rajada de carga HTTP, um emulador, um retransmissor de degradação, uma
tempestade, uma varredura, a execução de um experimento.

- Cada tarefa tem uma pílula na faixa do painel inferior, com seu número, o que ela é e um
  botão para pará-la. A barra lateral mostra quantas tarefas cada tela tem em andamento.
- [[ui:app.stopAll]], no cabeçalho, para todas as tarefas de uma vez.
- Uma tarefa que termina sozinha — uma varredura concluída, uma execução aprovada, um
  monitor cuja porta falhou — sai da faixa, e o console diz como ela terminou.
- Uma tarefa continua enquanto você trabalha em outras telas.
- Instalar uma atualização para todas as tarefas antes.

Em um servidor, as tarefas pertencem ao servidor: toda página conectada a ele vê as mesmas
tarefas e pode pará-las.

## Captura e o Inspetor {#capture}

Toda ferramenta — envios, monitores, ouvintes, emuladores, retransmissores, execuções de
experimentos — entrega cada quadro que envia ou recebe a uma única **captura**, e o
[Inspetor](../tools/inspector.md) a mostra em uma só linha do tempo.

- A captura fica **desligada até você ativá-la** com [[ui:ins.arm]], e não custa nada
  enquanto está desligada. Depois de ativada, ela continua ligada, em qualquer tela, até
  você desativá-la.
- Ela guarda até 8192 quadros e 64 MiB dos bytes deles; os quadros mais antigos dão lugar
  aos novos. Cada quadro guarda até 256 KiB dos seus bytes, e a lista mostra os primeiros
  256 bytes.
- [[ui:ins.pause]] impede a lista de se mover para que você possa lê-la; a captura continua
  por baixo.
- Os valores dos segredos usados por uma execução são mascarados em todos os quadros.
- A captura inteira pode ser exportada, com todos os bytes, para um arquivo `.jsonl` ou
  `.txt`.

## Emuladores {#emulators}

Um **emulador** faz o papel do outro lado: a API, o dispositivo ou o serviço com que o seu
sistema conversa. Cada um é um documento com um protocolo, o endereço em que escuta e
regras que dizem o que responder:

| Protocolo | O que ele emula |
| --- | --- |
| HTTP | Uma API: rotas por método e caminho, respostas em sequência, em rodízio ou aleatórias, com atrasos e falhas |
| OSC | Um dispositivo que responde a mensagens OSC por endereço e argumentos |
| UDP | Um dispositivo que responde a datagramas pela carga útil |
| TCP | Um dispositivo que responde a linhas em uma conexão TCP, com uma saudação |
| MQTT | Um broker que encaminha o que os clientes publicam e responde por regras, como um dispositivo |

Um emulador pode responder devagar, falhar, fechar a conexão, enviar um corpo malformado ou
ficar fora do ar em um horário programado. Cada troca é contada, listada na sua tela e
capturada para o Inspetor.

Você inicia um na tela [[ui:nav.emulators]], onde ele roda como tarefa; no nó
[[ui:exp.node.emulator]] de um experimento, onde ele responde durante a execução inteira;
ou com `signallab emulate`. A **biblioteca de emuladores** é o `emulators.json` na pasta de
dados. Ela começa com um emulador de cada tipo, todos em `127.0.0.1`:

| Emulador | Escuta em | O que faz |
| --- | --- | --- |
| [[ui:seed.emu.demo-api.name]] | `127.0.0.1:8080` (HTTP) | Uma verificação de saúde, um usuário por id, uma criação, uma resposta lenta e uma rota que falha duas vezes antes de funcionar |
| [[ui:seed.emu.osc-device.name]] | `127.0.0.1:9100` (OSC) | Responde `/ping` com `/pong` e uma contagem, confirma `/fader/…` com `/ack`, recebe `/cue/…` sem dizer nada |
| [[ui:seed.emu.udp-device.name]] | `127.0.0.1:7100` (UDP) | Responde `PING` com `PONG` e uma contagem, e qualquer outra coisa com quantos bytes recebeu |
| [[ui:seed.emu.tcp-device.name]] | `127.0.0.1:7200` (TCP) | Um protocolo de linhas como o de um projetor: saúda com `READY`, informa e muda o estado de energia, diz `BYE` e desliga em `QUIT` |
| [[ui:seed.emu.mqtt-broker.name]] | `127.0.0.1:1883` (MQTT) | Um `lab/status` retido e uma lâmpada: `ON` ou `OFF` publicado em `lab/<name>/set` é respondido em `lab/<name>/state` |

Veja [Emuladores](../tools/emulators.md).

## Retransmissores de degradação {#impairment}

Um **retransmissor de degradação** fica entre um cliente e o destino dele. Você aponta o
cliente para o endereço de escuta do retransmissor em vez do destino real; o retransmissor
encaminha nos dois sentidos e degrada o que passa, segundo um **perfil**:

- em **UDP**, cada datagrama tem o seu próprio destino: latência e jitter, perda e rajadas
  de perda, duplicação, corrupção, reordenação, um limite de largura de banda, ou nada
  (desconectado);
- em **TCP**, cada conexão é ligada a uma conexão própria com o destino, e os dois fluxos
  são atrasados, limitados a uma largura de banda, redefinidos ou deixados semiabertos.

Predefinições aplicam um perfil com um clique, de um cabo a um enlace de satélite. Uma
mudança vale enquanto o retransmissor roda, sem soltar a porta. Toda decisão sai de uma
semente, então o mesmo tráfego encontra o mesmo destino de novo.

Na tela [[ui:nav.netsim]], um retransmissor roda como tarefa. Em um experimento, um nó
[[ui:exp.node.impairment]] abre um para a execução e [[ui:exp.node.impairment_change]]
troca o perfil dele no meio da execução. Veja [Degradação](../tools/impairment.md) e
[Falhas](../experiments/faults.md).

## Experimentos {#experiments}

### Nós e fios {#nodes-and-wires}

Um experimento é um grafo de **nós** ligados por **fios**. Cada nó é uma etapa: envia algo,
aguarda algo, verifica um valor, extrai um, muda o fluxo ou prepara a execução — um
emulador, um retransmissor de degradação. Todo experimento tem exatamente um nó
[[ui:exp.node.start]] e um [[ui:exp.node.end]], e comporta até 64 nós. O experimento
aberto no editor é salvo enquanto você o edita. Veja [Nós](../experiments/nodes.md).

### Saídas {#outputs}

Um fio vai da **saída** de um nó à entrada de outro. A maioria dos nós tem uma saída; os
outros escolhem entre várias: [[ui:exp.yes]] e [[ui:exp.no]] em um desvio,
[[ui:exp.portMatched]] e [[ui:exp.portTimeout]] em uma espera, [[ui:exp.portBody]],
[[ui:exp.portDone]] e [[ui:exp.portLimit]] em um laço, [[ui:exp.branch1]] e
[[ui:exp.branch2]] em um ramo paralelo.

Uma saída pode ter vários fios: cada um roda como um ramo próprio, em paralelo, e um nó
[[ui:exp.node.join]] aguarda todos os fios que chegam a ele. Só o corpo de um nó
[[ui:exp.node.loop]] pode voltar; qualquer outro ciclo é um erro. Veja
[Fluxo](../experiments/flow.md).

### Parâmetros e perfis {#parameters}

Um **parâmetro** é um valor com nome — um host, uma porta, um nome de usuário — escrito uma
vez em [[ui:exp.params]] e usado em qualquer campo como `{{name}}`. Um **perfil** muda
vários parâmetros de uma vez: um para o notebook, um para o palco, um para o local do
evento. Você escolhe o perfil que as execuções usam, ou usa [[ui:exp.runWith]] para um
perfil, outros valores ou uma semente em uma só execução, sem alterar o experimento. Um
experimento comporta até 64 parâmetros e 32 perfis. Veja [Dados](../experiments/data.md).

### Modelos {#templates}

A maioria dos campos de texto dos nós são **modelos**: texto simples com expressões entre
chaves duplas, preenchidas quando a etapa é executada.

- `{{host}}` — um parâmetro, ou uma variável definida antes na execução, como um valor que
  um nó [[ui:exp.node.extract]] tirou de uma resposta, ou uma resposta que uma espera
  recebeu (`{{reply.args[0]}}`).
- `{{secret.API_TOKEN}}` — um segredo.
- `{{run.id}}`, `{{run.seed}}`, `{{now}}`, `{{now.iso}}`, `{{counter}}` — a execução e o
  momento.
- `{{uuid}}`, `{{random_int(1, 10)}}`, `{{random_float(0, 1, 2)}}`, `{{pick("a", "b")}}`
  — valores gerados.

Só o motor preenche modelos, então um campo significa a mesma coisa em uma execução, na
prévia do editor e em [[ui:exp.sendNow]]. Um nome desconhecido é um erro, nunca uma string
vazia. Veja [Dados](../experiments/data.md).

### Segredos {#secrets}

Um **segredo** é um valor que um experimento usa mas nunca guarda — um token, uma senha. O
experimento guarda apenas o nome; os campos o usam como `{{secret.NAME}}`; e todo texto
que uma execução relata, cada etapa, o relatório e cada quadro do Inspetor, o mostra
mascarado. Nenhum comando devolve o valor de um segredo.

Onde os valores ficam depende de onde o Signal Lab roda:

- **O aplicativo de desktop no Windows** os guarda no Gerenciador de Credenciais do
  Windows. Você os define em [[ui:exp.params]] → [[ui:exp.secrets]].
- **O aplicativo de desktop no Linux** não tem um repositório de credenciais para guardá-los,
  então os experimentos que usam segredos rodam por lá na linha de comando ou em um
  servidor.
- **Um servidor** os lê, somente leitura, do seu ambiente (`SIGNALLAB_SECRET_<NAME>`) ou de
  um arquivo por nome na sua pasta de segredos (`/run/secrets/signallab/<NAME>` por
  padrão); eles não podem ser definidos em um navegador.
- **A linha de comando** os lê como um servidor, ou do repositório de credenciais do sistema
  quando solicitado. Veja [A linha de comando](../automation/cli.md).

### Sementes {#seeds}

Toda execução tem uma **semente**, um número que decide tudo o que é aleatório nela: os
valores gerados, o jitter de uma repetição, a escolha aleatória de resposta de um emulador,
cada decisão de um retransmissor de degradação. A mesma semente e o mesmo tráfego dão a
mesma execução. Uma nova semente é sorteada para cada execução, a menos que o experimento
fixe uma — [[ui:exp.pinSeed]], na linha do tempo da execução, fixa a semente da última
execução, e [[ui:exp.seed]], em [[ui:exp.params]], define uma.

### Execuções e relatórios {#reports}

Uma **execução** começa em [[ui:exp.node.start]], segue os fios e é aprovada quando chega a
[[ui:exp.node.end]] sem nenhuma etapa que tenha falhado. Ela é interrompida se passar de
300 segundos. Cada etapa aparece na linha do tempo da execução quando começa e quando
termina.

Uma execução que termina, aprovada ou com falha, grava um **relatório** na pasta `runs` da
pasta de dados: o nome do experimento, a semente, o perfil e os valores usados, quando
começou e terminou, o resultado e o seu erro, cada etapa e o que os seus emuladores e
retransmissores contaram. Duas execuções do mesmo experimento podem ser comparadas. Veja
[Execuções e relatórios](../experiments/runs.md).

## A pasta de dados {#data-folder}

Tudo o que o Signal Lab guarda é um arquivo em uma única pasta: `Documents/SignalLab` na
sua pasta pessoal, tanto no Windows quanto no Linux. Um servidor guarda a sua própria, que
você escolhe ao iniciá-lo (`/data` na imagem Docker).

| Arquivo ou pasta | O que contém |
| --- | --- |
| `experiment.json` | O experimento aberto no editor |
| `signals.json` | A biblioteca de sinais |
| `emulators.json` | A biblioteca de emuladores |
| `runs/` | Um relatório para cada execução |
| `exports/` | Experimentos exportados pela janela de experimentos |
| `capture-….jsonl`, `capture-….txt` | Exportações do Inspetor |

Os arquivos são JSON, gravados por inteiro. Se um deles não puder ser lido, o Signal Lab
diz qual arquivo e onde está o erro, e o deixa como está em vez de recomeçar do zero. Veja
[Arquivos e pastas](../reference/files.md).

## Desktop e servidor {#desktop-and-server}

O aplicativo de desktop e um servidor rodam o mesmo motor por trás da mesma interface. O
que muda:

| | Aplicativo de desktop | Servidor, em um navegador |
| --- | --- | --- |
| Onde o tráfego começa, onde os monitores escutam | Este computador | O servidor |
| Pasta de dados | `Documents/SignalLab` | A do servidor; passe o mouse sobre [[ui:app.server]] no cabeçalho para vê-la |
| Segredos | Gerenciador de Credenciais do Windows, definidos no aplicativo; nenhum no Linux | Somente leitura, do ambiente do servidor ou de arquivos de segredos |
| Relatórios, exportações, capturas | Gravados na pasta de dados; o caminho é mostrado | Baixados pelo navegador |
| Entrar | — | Com o token de acesso do servidor, quando ele tem um |
| Tarefas, o armazenamento de cookies da tela HTTP | Deste aplicativo | Do servidor, compartilhados por toda página conectada a ele |
| Firewall | Um aviso oferece permitir o Signal Lab (Windows) | Nunca alterado pelo Signal Lab |
| Atualizações | Instala versões assinadas quando você clica | Atualizado com a sua imagem |

Veja [O servidor](../server/index.md) e [Segurança do servidor](../server/security.md).

## O que o Signal Lab não faz por conta própria {#on-its-own}

- **Ele só envia quando você age**, e só para os endereços que você digita. Iniciar o
  aplicativo não envia nada — exceto, no aplicativo de desktop, a verificação diária de
  atualização, que você pode desligar. O feedback só sai quando você envia o formulário.
- **Os exemplos ficam neste computador.** Os sinais iniciais, os emuladores iniciais, os
  novos emuladores e os modelos de experimento usam todos `127.0.0.1`. Os ouvintes que você
  inicia — um monitor OSC, o ouvinte de descoberta, um retransmissor de degradação — usam
  `0.0.0.0` por padrão, todas as placas de rede, para que outras máquinas possam alcançá-los;
  digite `127.0.0.1` para mantê-lo neste computador.
- **Ele só altera o firewall quando você clica em** [[ui:fw.allow]] e confirma o aviso de
  administrador do Windows, ou executa `signallab firewall allow`. Um servidor nunca altera
  o firewall do seu host.
- **Um servidor sem token de acesso** escuta só em `127.0.0.1` e se recusa a iniciar em
  qualquer outro endereço.
- **Ele respeita limites de proteção**: uma varredura de broadcast alcança no máximo 1024
  hosts, e um beacon envia no máximo 50.000 pacotes por segundo somando todos os seus
  destinos.

Os limites de proteção não são uma permissão: [Tempestade](../tools/storm.md), o
[Scanner](../tools/scanner.md) e o [Broadcast](../protocols/broadcast.md) enviam tráfego
real. Use-os só em redes e hosts que são seus ou que você tem autorização para testar.

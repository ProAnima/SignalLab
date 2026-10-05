---
title: Emuladores
description: Faça do Signal Lab o outro lado — uma API HTTP, um dispositivo OSC, UDP ou TCP, ou um broker MQTT — que responde pelas suas regras, falha quando você quer e conta o que chega.
---

# Emuladores

Um emulador é o Signal Lab fazendo o papel da API, do dispositivo ou do serviço com que o seu
sistema conversa. Ele escuta em um endereço e responde por regras: uma API HTTP, por rotas; um
dispositivo OSC, UDP ou TCP, por "quando chegar isto, responda aquilo"; um broker MQTT, como
qualquer broker, mais regras próprias. Ele pode ser lento, falhar ou sair do ar de vez em quando,
para que você teste o que o seu sistema faz quando uma dependência se comporta mal. Cada troca é
contada, listada e enviada ao [Inspetor](inspector.md).

Um emulador é um documento. A tela [[ui:nav.emulators]] guarda uma biblioteca deles; o mesmo
documento é executado dentro de um experimento como nó [[ui:exp.node.emulator]], pela linha de
comando com `signallab emulate` e pela [API](../api/commands.md) e pelo [MCP](../automation/mcp.md),
e responde do mesmo jeito em todo lugar.

## A tela {#screen}

À esquerda fica a biblioteca ([[ui:emu.library]]): cada emulador com o protocolo e o endereço, e um
ponto pulsante e uma contagem de requisições nos que estão rodando. À direita ficam as
configurações e as regras do emulador selecionado e, abaixo delas, o que ele recebeu
([[ui:emu.live]]).

## Criar um emulador {#create}

1. Pressione um dos botões no alto da biblioteca:

   | Botão | Cria | Escuta em | Com uma regra que funciona como está |
   | --- | --- | --- | --- |
   | ＋ [[ui:emu.new.http]] | Uma API HTTP | `127.0.0.1:18080` | `GET /health` → 200 `{"status":"ok"}` |
   | ＋ [[ui:emu.new.osc]] | Um dispositivo OSC | `127.0.0.1:9100` | `/ping` → `/pong` com a contagem como int |
   | ＋ [[ui:emu.new.udp]] | Um dispositivo UDP | `127.0.0.1:7100` | um datagrama que contém `PING` → `PONG 1`, `PONG 2`, … |
   | ＋ [[ui:emu.new.tcp]] | Um dispositivo TCP | `127.0.0.1:7200` | uma linha que contém `PING` → `PONG` |
   | ＋ [[ui:emu.new.mqtt]] | Um broker MQTT | `127.0.0.1:1883` | uma publicação em `lab/<name>/set` → a mesma carga útil, retida, em `lab/<name>/state` |

   Quando outro emulador da biblioteca já usa essa porta, a próxima porta livre é usada.
2. Dê a ele um [[ui:emu.name]] (no máximo 120 caracteres).
3. Preencha o campo [[ui:emu.bind]] com `IP:port`. `127.0.0.1` responde só a este computador;
   `0.0.0.0` responde também à rede.
4. Altere as regras (abaixo) e diga na [[ui:emu.note]] o que ele substitui.

As alterações são salvas sozinhas. O botão [[ui:emu.duplicate]] faz uma cópia na próxima porta livre.
O botão [[ui:emu.delete]] pergunta mais uma vez ([[ui:emu.confirmDelete]]), para o emulador se ele
estiver rodando e o remove da biblioteca.

As regras são testadas em ordem, da primeira à última; a primeira que corresponde responde. O
cabeçalho de cada regra mostra um resumo de uma linha; clique nele para abrir ou recolher a regra.
Os botões ↑ e ↓ movem uma regra, e × a remove.

## Executá-lo {#run}

1. Selecione o emulador e pressione [[ui:emu.start]]. A porta dele abre antes que o botão volte:
   se a porta já estiver ocupada, ou se o emulador tiver um problema, o início é recusado ali,
   com o motivo.
2. Aponte o seu sistema para ele. Para uma API HTTP, [[ui:emu.copyUrl]] copia o endereço dela
   (`http://127.0.0.1:18080`), e cada rota tem um botão [[ui:emu.copyRouteUrl]] para o seu próprio
   endereço (não quando o caminho dela contém um modelo `{{…}}`).
3. Veja a lista [[ui:emu.received]] se encher.
4. Pressione [[ui:emu.stop]], ou pare a tarefa dele na faixa do console.

O estado ao lado dos botões diz [[ui:emu.notRunning]], onde ele responde ou que está fora do ar.

Um emulador continua respondendo com as regras com que foi iniciado. Quando você o altera enquanto
ele roda, aparece o botão [[ui:emu.restart]]: pressione-o para iniciá-lo de novo com as regras como estão
agora. Até lá, as contagens de correspondências das regras ficam escondidas, pois pertencem às
regras antigas.

O botão [[ui:emu.takeDown]] deixa indisponível um emulador em execução até você pressionar
[[ui:emu.bringUp]]: uma requisição HTTP recebe 503, um dispositivo TCP e um broker MQTT derrubam
as conexões e recusam novas, um dispositivo OSC ou UDP não responde nada. Veja
[Fora do ar](#outage).

Dois emuladores do mesmo transporte não podem compartilhar uma porta: os emuladores HTTP, TCP e
MQTT escutam em portas TCP; os OSC e UDP, em portas UDP. Uma API HTTP e um dispositivo OSC podem
ambos usar a porta 8080; duas APIs HTTP, não. Um segundo emulador em uma porta ocupada é
recusado quando é iniciado.

::: tip
Em um navegador conectado a um [servidor](../server/index.md), o emulador roda no servidor. Um emulador
que escute em `0.0.0.0` é alcançado pelo nome do servidor, e [[ui:emu.copyUrl]] copia esse endereço;
um em `127.0.0.1` responde só a programas no próprio servidor.
:::

## O que chegou {#received}

Enquanto ele roda, o painel [[ui:emu.live]] conta:

| Contagem | O quê |
| --- | --- |
| [[ui:emu.total]] | Tudo o que chegou: requisições, mensagens, linhas. |
| [[ui:emu.unmatched]] | O que nenhuma regra aceitou. Uma requisição HTTP sem rota ainda recebe a sua resposta (veja [Requisições que nenhuma rota aceita](#fallback)); as outras não recebem nenhuma. |
| [[ui:emu.failed]] | Trocas em que não foi possível montar ou enviar uma resposta. |
| [[ui:emu.down]] | O que chegou enquanto o emulador estava fora do ar. Aparece quando ele tem uma queda programada ou quando algo o encontrou fora do ar. Nunca conta como [[ui:emu.unmatched]]. |
| [[ui:emu.missed]] | Só MQTT, quando acontece: mensagens que um cliente estava atrasado demais para receber. |

O cabeçalho de cada regra mostra quantas vezes ela correspondeu desde o início.

A lista [[ui:emu.received]] mostra as 300 trocas mais recentes, a mais nova primeiro:

| Coluna | O quê |
| --- | --- |
| [[ui:emu.col.time]] | Quando chegou. |
| [[ui:emu.col.from]] | O endereço do cliente. |
| [[ui:emu.col.request]] | O que chegou, em notação do protocolo: `GET /users/7`, `/ping 1`, `POWER?`. |
| [[ui:emu.col.rule]] | A regra que a aceitou (`#2`), ou `—`. |
| [[ui:emu.col.reply]] | O que voltou: `200 OK · 37 B`, `/pong 3`, uma carga útil; [[ui:emu.held]] ou [[ui:emu.closed]], para uma falha; o erro, quando a resposta falhou; [[ui:emu.wasDown]], quando chegou com o emulador fora do ar. |
| [[ui:emu.col.ms]] | Da chegada até a saída da resposta, com o atraso incluído. |

O botão ⌕ de uma linha ([[ui:emu.inspectFrame]]) abre essa troca no Inspetor, quando a captura
estava ativa. Quando chegam mais de 200 trocas em um quinto de segundo, a lista pula algumas e diz
quantas. O motor guarda as 500 trocas mais recentes de cada emulador em execução, com o que
chegou, para a linha de comando, a API e o MCP.

## API HTTP {#http}

Um servidor HTTP/1.1. Cada requisição é respondida pela primeira rota que a aceita.

### Rotas {#routes}

Uma rota aceita uma requisição quando o método, o caminho e todas as condições dela correspondem.

| Campo | O quê |
| --- | --- |
| [[ui:emu.method]] | `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS` ou [[ui:emu.methodAny]]. Uma rota `GET` responde também a `HEAD`. |
| [[ui:emu.path]] | Começa com `/`. Um segmento `:name` aceita qualquer segmento, lido como `{{request.params.name}}`; um último segmento `*` aceita tudo o que vem abaixo. Uma `/` no final não faz diferença; a query string não faz parte do caminho. |
| [[ui:emu.conditions]] | Todas precisam se cumprir. Acrescente uma com ＋ [[ui:emu.addCondition]]. |

Exemplos de caminho:

| Caminho | Aceita | Não aceita |
| --- | --- | --- |
| `/health` | `/health`, `/health/` | `/health/db`, `/Health` |
| `/users/:id` | `/users/7` (`params.id` é `7`), `/users/a%20b` (`a b`) | `/users`, `/users/7/orders` |
| `/files/*` | `/files`, `/files/a`, `/files/a/b/c` | `/file`, `/other/files/a` |

Uma condição lê uma parte da requisição ([[ui:emu.on]]) e a compara:

| [[ui:emu.on]] | Nome | Lê |
| --- | --- | --- |
| [[ui:emu.on.header]] | O nome de um cabeçalho, sem distinção de maiúsculas | O valor do cabeçalho; para um cabeçalho enviado várias vezes, os valores unidos com `, `. |
| [[ui:emu.on.query]] | Um parâmetro da query | O valor dele, decodificado; o primeiro, quando ele se repete. |
| [[ui:emu.on.body]] | — | O corpo inteiro como texto. |
| [[ui:emu.on.json]] | Um caminho JSON, como `$.user.id` | Esse campo de um corpo JSON. |

As comparações são [[ui:exp.op.eq]], [[ui:exp.op.ne]], [[ui:exp.op.lt]], [[ui:exp.op.le]],
[[ui:exp.op.gt]], [[ui:exp.op.ge]], [[ui:exp.op.contains]], [[ui:exp.op.matches]],
[[ui:exp.op.empty]] e [[ui:exp.op.not_empty]]. Números são comparados como números; texto, de forma
exata. Um cabeçalho, parâmetro ou campo que não existe está vazio. Uma comparação que não pode ser
feita — texto contra número — não se cumpre.

### Respostas {#responses}

Uma rota tem de uma a 16 respostas ([[ui:emu.responses]]).

| Campo | O quê | Padrão |
| --- | --- | --- |
| [[ui:emu.status]] | 100–599. | 200 |
| [[ui:emu.fault]] | Algo diferente de uma resposta; veja [Falhas](#faults). | [[ui:emu.fault.none]] |
| [[ui:emu.delay]] | Quanto esperar antes de responder, 0–60.000 ms. | 0 |
| [[ui:emu.jitter]] | Até este tanto a mais, ao acaso, 0–60.000 ms. | 0 |
| [[ui:emu.weight]] | A participação dela quando a rota responde ao acaso. Só aparece nesse caso. | 1 |
| [[ui:emu.headers]] | Até 32. Os nomes podem usar parâmetros; os valores são [modelos](#templates). | nenhum |
| [[ui:emu.body]] | Um [modelo](#templates), até 256 KiB como escrito. | vazio |

Sem um cabeçalho `Content-Type`, um corpo que é JSON válido vai como `application/json`, e qualquer
outro corpo como `text/plain; charset=utf-8`.

Com duas respostas ou mais, o campo [[ui:emu.order]] diz qual delas uma requisição recebe:

| [[ui:emu.order]] | As requisições recebem | Para |
| --- | --- | --- |
| [[ui:emu.order.sequence]] | A primeira, a segunda, …, e daí em diante a última: 500, 500, 200, 200, 200… | Novas tentativas: falhar duas vezes e depois funcionar. |
| [[ui:emu.order.cycle]] | A primeira de novo depois da última: 200, 500, 200, 500… | Uma dependência que falha de vez em quando, com regularidade. |
| [[ui:emu.order.random]] | Cada uma sorteada pelo seu peso. Pesos 8 e 2 dão a primeira em cerca de 80% das vezes. Pelo menos um peso precisa ser maior que 0. | Uma proporção realista de falhas. |

O menu [[ui:emu.preset]] acrescenta à rota uma resposta pronta:

| Predefinição | Acrescenta |
| --- | --- |
| [[ui:emu.preset.ok]] | 200, `{"ok":true}` |
| [[ui:emu.preset.created]] | 201, `{"id":"{{uuid}}"}`, cabeçalho `Location: {{request.path}}/{{counter}}` |
| [[ui:emu.preset.notFound]] | 404, `{"error":"not found"}` |
| [[ui:emu.preset.error]] | 500, `{"error":"internal"}` |
| [[ui:emu.preset.unavailable]] | 503, `{"error":"unavailable"}`, cabeçalho `Retry-After: 1` |
| [[ui:emu.preset.slow]] | 200, `{"ok":true}` depois de 2.000 ms |
| [[ui:emu.preset.timeout]] | A falha [[ui:emu.fault.timeout]] |
| [[ui:emu.preset.reset]] | A falha [[ui:emu.fault.reset]] |
| [[ui:emu.preset.malformed]] | 200, `{"items":[{"id":1},{"id":2}]}` com a falha [[ui:emu.fault.malformed]] |

### Falhas {#faults}

| [[ui:emu.fault]] | O que o cliente encontra |
| --- | --- |
| [[ui:emu.fault.none]] | A resposta. |
| [[ui:emu.fault.timeout]] | Nada. A requisição fica segurada por até 2 minutos e depois a conexão é fechada — assim, o que se testa é o próprio timeout do cliente. O atraso não se aplica. |
| [[ui:emu.fault.reset]] | A conexão fecha sem resposta, depois do atraso. |
| [[ui:emu.fault.malformed]] | Uma resposta HTTP completa, com o status e os cabeçalhos definidos, cujo corpo para no meio: um JSON que não pode ser interpretado. Quando o corpo inteiro era JSON, o tipo de conteúdo continua dizendo `application/json`. |

### Requisições que nenhuma rota aceita {#fallback}

A opção [[ui:emu.fallback]] decide o que recebe uma requisição que não corresponde a nenhuma rota:

- [[ui:emu.fallbackDefault]] — 404 com o corpo `{"error":"no_route"}`;
- [[ui:emu.fallbackCustom]] — uma resposta que você define, com tudo o que a resposta de uma rota
  tem. O `{{counter}}` dela conta as requisições que nenhuma rota aceitou.

De um jeito ou de outro, a requisição conta como [[ui:emu.unmatched]].

### O que uma resposta HTTP pode ler {#http-request}

| Modelo | É |
| --- | --- |
| `{{request.method}}` | `GET`, `POST`, … |
| `{{request.path}}` | O caminho, sem a query. |
| `{{request.params.id}}` | O segmento do caminho chamado `:id`. |
| `{{request.query.page}}` | Um parâmetro da query, decodificado. |
| `{{request.headers.x-key}}` | Um cabeçalho; nomes em minúsculas. |
| `{{request.body}}` | O corpo como texto: os primeiros 64 KiB dele. |
| `{{request.json.name}}` | Um campo de um corpo JSON, quando o corpo é JSON e cabe em 64 KiB. |
| `{{request.from}}` | O `IP:port` do cliente. |

Um corpo de requisição maior que 1 MiB recebe 413 e conta como [[ui:emu.failed]]. Uma resposta que
não pode ser montada — um modelo que cita algo que a requisição não tem — recebe 500 com o erro no
corpo e conta como [[ui:emu.failed]].

## Dispositivo OSC {#osc}

Cada mensagem que chega — cada mensagem de um bundle, separadamente — é respondida pela primeira
regra a que ela corresponde. Um datagrama que não é OSC conta como [[ui:emu.unmatched]].

| Campo | O quê |
| --- | --- |
| [[ui:emu.address]] | Um padrão de endereço OSC 1.0: `*` quaisquer caracteres, `?` um caractere, `[a-z]` um conjunto, `{a,b}` um ou outro, cada um dentro de um segmento (veja [OSC](../protocols/osc.md#patterns)). |
| [[ui:exp.argRules]] | Até 16 condições sobre os argumentos, como em [[ui:exp.node.wait_osc]] (veja [Nós](../experiments/nodes.md#node-wait_osc)). |
| [[ui:emu.replyOn]] | Desmarcado: aceita a mensagem e não responde nada. |
| [[ui:emu.replyAddress]] | O endereço da resposta, um [modelo](#templates). |
| [[ui:emu.replyArgs]] | Até 16 argumentos, cada um com um [[ui:emu.argType]] (`int`, `float`, `str`, `long`, `double`, `bool`, `blob`, `nil`) e um modelo em [[ui:emu.argValue]]. |
| [[ui:emu.to]] | Vazio: de volta ao endereço e à porta do remetente. Caso contrário, `IP:port`. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 0–60.000 ms cada. |

O valor de um argumento é lido como o tipo dele depois que o modelo é preenchido:
`{{request.args[0]}}` devolve o primeiro argumento como número quando o tipo é numérico. Um `bool`
aceita `true`, `1`, `yes`, `on` ou `false`, `0`, `no`, `off`; um `blob` aceita bytes em hex; um
valor vazio é o zero do tipo.

As respostas saem da própria porta do emulador, então um cliente que escuta na porta de onde
enviou as ouve.

Uma resposta OSC pode ler `{{request.address}}`, `{{request.args[0]}}` e `{{request.from}}`.

## Dispositivo UDP {#udp}

Cada datagrama é respondido pela primeira regra a que ele corresponde.

| Campo | O quê |
| --- | --- |
| [[ui:emu.match]] | [[ui:exp.mode.any]], [[ui:exp.mode.contains]], [[ui:exp.mode.regex]] ou [[ui:exp.mode.hex]]. |
| [[ui:emu.pattern]] | O texto, a expressão regular ou os bytes a procurar. |
| [[ui:emu.reply]] | [[ui:emu.replyOff]], [[ui:emu.replyText]] ou [[ui:emu.replyHex]] e depois a própria resposta, como [modelo](#templates). |
| [[ui:emu.to]] | Vazio: de volta ao remetente. Caso contrário, `IP:port`. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 0–60.000 ms cada. |

Uma resposta UDP ou TCP pode ler:

| Modelo | É |
| --- | --- |
| `{{request.text}}` | A carga útil como texto. |
| `{{request.match}}` | O que correspondeu: o texto, o primeiro grupo de uma expressão regular (ou a correspondência inteira), os bytes. |
| `{{request.hex}}` | A carga útil em bytes hex, os primeiros 1.024 deles. |
| `{{request.bytes}}` | O tamanho da carga útil. |
| `{{request.from}}` | O `IP:port` do remetente. |

Uma resposta em texto tem no máximo 65.507 bytes.

## Dispositivo TCP {#tcp}

Um dispositivo que fala por linhas em uma conexão TCP, como fazem um projetor ou um switcher
matricial. Cada mensagem que um cliente envia é respondida pela primeira regra a que ela
corresponde; a resposta volta pela mesma conexão.

| Campo | O quê |
| --- | --- |
| [[ui:emu.delimiter]] | O que termina uma mensagem e é acrescentado depois de cada resposta e da saudação: [[ui:emu.delimiter.lf]] (um `\r` antes dele é descartado), [[ui:emu.delimiter.crlf]], [[ui:emu.delimiter.cr]] ou [[ui:emu.delimiter.none]]. Linhas vazias são ignoradas. |
| [[ui:emu.greeting]] | Enviada quando um cliente se conecta; vazia para nenhuma. Pode ler `{{request.from}}`. |
| [[ui:emu.match]], [[ui:emu.pattern]], [[ui:emu.reply]] | Como em um [dispositivo UDP](#udp). |
| [[ui:emu.close]] | Fecha a conexão depois da resposta desta regra — para `QUIT`, por exemplo. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 0–60.000 ms cada. |

Uma mensagem com mais de 64 KiB sem o delimitador é aceita como está.

## Broker MQTT {#mqtt}

Um pequeno broker MQTT 3.1.1 sobre TCP simples. Ele faz o que um broker faz: os clientes se
conectam, assinam com `+` e `#`, publicam com QoS 0, 1 e 2, mensagens retidas e mensagens de última
vontade (will) funcionam, e uma segunda conexão com o id de um cliente assume o lugar da primeira.
As sessões são sempre limpas: um cliente que pede para manter a sessão recebe uma nova, e nada fica
na fila para um cliente ausente.

Além disso, cada mensagem publicada nele é comparada com as regras: a primeira que corresponde
também publica uma resposta — um dispositivo informando o que fez.

| Campo | O quê |
| --- | --- |
| [[ui:emu.username]], [[ui:emu.password]] | Quando um nome de usuário está definido, um cliente precisa se conectar com ele e com a senha; vazio: qualquer um pode se conectar. Uma senha sem nome de usuário é recusada, pois o MQTT 3.1.1 não consegue transportá-la. |
| [[ui:emu.retained]] | Até 64 mensagens ([[ui:emu.topic]], [[ui:emu.payload]], [[ui:emu.qos]]) guardadas desde o início, como se publicadas com retain: um cliente que assina as recebe primeiro. |
| [[ui:emu.topicFilter]] | Quais tópicos uma regra aceita: `+` um nível, `#` o resto — `lab/+/set`. |
| [[ui:emu.match]], [[ui:emu.pattern]] | Uma condição sobre a carga útil, como em um [dispositivo UDP](#udp). |
| [[ui:emu.replyOn]] | Desmarcado: aceita a mensagem e não publica mais nada. |
| [[ui:emu.replyTopic]], [[ui:emu.replyPayload]] | [Modelos](#templates). O tópico não pode conter `+` nem `#`. |
| [[ui:emu.qos]], [[ui:emu.retain]] | Da resposta. |
| [[ui:emu.delay]], [[ui:emu.jitter]] | 0–60.000 ms cada. |

Uma resposta MQTT pode ler `{{request.topic}}`, `{{request.levels[1]}}` (os níveis do tópico, a
partir de 0), `{{request.payload}}`, `{{request.json.state}}`, `{{request.match}}`,
`{{request.qos}}`, `{{request.retain}}`, `{{request.client}}` (o id do cliente) e
`{{request.from}}`.

## Modelos nas respostas {#templates}

As respostas são escritas na mesma [linguagem de modelos](../experiments/data.md#templates) dos
experimentos, então um campo significa a mesma coisa aqui e lá. Uma resposta pode ler:

- `request` — o que chegou, conforme listado acima para cada protocolo;
- `{{counter}}` — quantas mensagens esta regra aceitou desde que o emulador foi iniciado,
  incluindo esta;
- os [geradores](../experiments/data.md#generators) — `{{uuid}}`, `{{now.iso}}`, valores
  aleatórios e os demais; os aleatórios são sorteados a partir da semente do emulador;
- parâmetros, quando o emulador roda em um experimento ou é iniciado com
  `signallab emulate --param`.

Uma resposta nunca lê segredos, e um nome desconhecido é um erro, não um texto vazio.

Alguns campos são fixados quando o emulador é iniciado, antes que algo chegue: um caminho, uma
condição, um padrão de endereço, um padrão de carga útil, um filtro de tópicos, [[ui:emu.to]], o
nome de um cabeçalho, as mensagens retidas e o login do broker. Eles aceitam só texto e
parâmetros, sem `request` e sem geradores.

A semente comanda a ordem aleatória das respostas, o jitter e os geradores aleatórios. Na tela
[[ui:nav.emulators]], cada início usa uma semente nova; um experimento usa a semente da execução, e
`signallab emulate --seed` usa a que você informar.

## Fora do ar {#outage}

Para testar o que o seu sistema faz quando uma dependência oscila, marque a opção
[[ui:emu.outage]]:

| Campo | O quê | Padrão |
| --- | --- | --- |
| [[ui:emu.outageUp]] | Por quanto tempo ele responde, 10–3.600.000 ms. | 10.000 |
| [[ui:emu.outageDown]] | Por quanto tempo ele fica fora do ar, 10–3.600.000 ms. | 3.000 |
| [[ui:emu.outageFault]] | Só HTTP: o que uma requisição encontra enquanto ele está fora do ar. | [[ui:emu.outageFault.unavailable]] |

O ciclo começa quando o emulador é iniciado e se repete: no ar, fora do ar, no ar, fora do ar…
Enquanto ele está fora do ar:

| Emulador | O que se encontra |
| --- | --- |
| HTTP | [[ui:emu.outageFault.unavailable]]: 503 com `Retry-After` definido com os segundos que faltam para ele voltar (pelo menos 1). [[ui:emu.outageFault.reset]]: a conexão fecha sem resposta. [[ui:emu.outageFault.timeout]]: segurada por até 2 minutos e depois fechada. |
| Dispositivo TCP | As conexões abertas caem em até 0,1 s; as novas são fechadas assim que chegam. |
| Broker MQTT | Todas as conexões caem; as novas são recusadas (código de retorno 3 no CONNACK, servidor indisponível). |
| Dispositivo OSC, UDP | Nada é respondido. |

O que chega enquanto ele está fora do ar conta como [[ui:emu.down]], não como
[[ui:emu.unmatched]], e as regras dele não são consultadas.

O botão [[ui:emu.takeDown]] faz o mesmo quando você quiser, diga o ciclo o que disser, até você pressionar
[[ui:emu.bringUp]]; o HTTP então encontra 503 sem `Retry-After`. Em um experimento, o nó
[[ui:exp.node.emulator_state]] faz isso em uma etapa da execução (veja
[Nós](../experiments/nodes.md#node-emulator_state) e [Falhas](../experiments/faults.md)).

## Problemas {#problems}

Enquanto você edita, o emulador é verificado um instante depois de cada alteração, e um problema
aparece abaixo dos botões dele antes que você pressione [[ui:emu.start]]. Um problema diz onde
está — a regra, a resposta ou a mensagem retida, e o campo — e o que está errado: um caminho sem a
sua `/`, uma expressão regular que não compila, um modelo de resposta que cita algo além de
`request`, parâmetros e geradores, um valor fora do intervalo. O botão [[ui:emu.start]] recusa um emulador com um problema.

## Limites {#limits}

| O quê | Limite | No limite |
| --- | --- | --- |
| Rotas ou regras por emulador | 64 | Recusado na verificação. |
| Respostas por rota | 16 | Recusado. |
| Condições por rota | 16 | Recusado. |
| Cabeçalhos por resposta | 32 | Recusado. |
| Condições de argumento, argumentos da resposta (OSC) | 16 cada | Recusado. |
| Mensagens retidas (MQTT) | 64 | Recusado. |
| Um corpo, uma resposta ou uma saudação, como escrito | 256 KiB | Recusado. |
| Um atraso ou um jitter | 60.000 ms | Recusado. |
| Corpo de requisição HTTP | 1 MiB | 413. |
| Conexões HTTP simultâneas | 512 | As excedentes são fechadas assim que chegam. |
| Cabeçalho de requisição HTTP | 30 s | Um cliente precisa enviá-lo dentro desse prazo. |
| Conexões TCP simultâneas | 256 | As excedentes são fechadas assim que chegam. |
| Respostas OSC e UDP aguardando o atraso | 1.024 | As excedentes são descartadas e contadas como [[ui:emu.failed]]. |
| Clientes MQTT simultâneos | 256 | Os excedentes são fechados assim que chegam. |
| Pacote MQTT | 256 KiB | A conexão do cliente termina. |
| Assinaturas MQTT por cliente | 100 | As excedentes são recusadas. |
| Tópicos retidos MQTT | 1.000 tópicos, 16 MiB | Uma nova mensagem retida é encaminhada, mas não retida. |
| Mensagens MQTT aguardando um cliente lento | 1.024 mensagens, 8 MiB | Ele as perde; contadas como [[ui:emu.missed]]. |

## Simular isto {#mock-this}

Para criar um emulador a partir de uma resposta que funcionou:

1. Na tela [[ui:nav.http]], envie uma requisição e obtenha uma resposta — ou use
   [[ui:exp.sendNow]] em um nó HTTP de um experimento.
2. Pressione ⧉ [[ui:http.mockThis]] ao lado da resposta. A caixa de diálogo
   [[ui:http.mockTitle]] mostra a rota que será criada.
3. No campo [[ui:http.mockInto]], escolha um dos seus emuladores HTTP, ou [[ui:http.mockNew]].
4. Pressione [[ui:http.mockAdd]]. A tela [[ui:nav.emulators]] abre nesse emulador.

A rota responde ao método e ao caminho da requisição (sem a query) com o status, os cabeçalhos e o
corpo da resposta. Os cabeçalhos que pertencem àquela troca específica (`Content-Length`, `Date`,
`Server`, `ETag` e afins) ficam de fora, e o corpo é enviado como era, mesmo que contenha `{{`. Um
emulador novo contém só essa rota. Acrescentada a um emulador existente, a rota vai para o início,
para responder antes de uma rota mais ampla; um emulador em execução a adota quando você pressiona
[[ui:emu.restart]].

A partir de um experimento, uma URL escrita com modelos vira um padrão: a base dela (`{{api}}`) é
descartada, um segmento que é um único modelo (`/orders/{{order_id}}`) vira `:order_id`, e um
segmento só em parte com modelo termina o caminho com `*`.

## O conjunto inicial {#starter-set}

Na primeira vez que o Signal Lab não encontra uma biblioteca de emuladores, ele grava cinco, todos
neste computador. Os nomes e as notas deles são escritos no idioma que a interface tem nesse
momento.

| Emulador | Escuta em | Faz |
| --- | --- | --- |
| [[ui:seed.emu.demo-api.name]] | `127.0.0.1:8080` | `GET /health` → `{"status":"ok","time":…}`; `GET /users/:id` → um usuário com esse id; `POST /users` → 201 com um `Location`; `GET /slow` → depois de 1.500 ms; `/flaky` → 503, 503 e depois 200 daí em diante. |
| [[ui:seed.emu.osc-device.name]] | `127.0.0.1:9100` | `/ping` → `/pong` com a contagem; `/fader/*` → `/ack` com o endereço recebido; `/cue/*` aceito sem resposta. |
| [[ui:seed.emu.udp-device.name]] | `127.0.0.1:7100` | `PING` → `PONG` e a contagem; qualquer outra coisa → `ACK` e o tamanho dela em bytes. |
| [[ui:seed.emu.tcp-device.name]] | `127.0.0.1:7200` | Linhas terminadas em CR LF. Cumprimenta com `READY`; `POWER?` → `POWER=ON`; `POWER ON` ou `POWER OFF` → `OK ON` / `OK OFF`; `QUIT` → `BYE` e depois desliga. |
| [[ui:seed.emu.mqtt-broker.name]] | `127.0.0.1:1883` | Retém `online` em `lab/status`; `ON` ou `OFF` publicado em `lab/<name>/set` → o mesmo, retido, em `lab/<name>/state`. |

O sinal inicial [[ui:seed.http-reachable.name]] da [biblioteca de sinais](signals.md#starter-set)
consulta `http://127.0.0.1:8080/`, o endereço da [[ui:seed.emu.demo-api.name]]: como ela não tem
rota para `/`, o sinal recebe
404.

## O arquivo da biblioteca {#file}

A biblioteca é o `emulators.json` na pasta de dados (veja [Arquivos](../reference/files.md));
passe o ponteiro sobre a contagem abaixo da lista para ver o caminho dele. Ele é gravado inteiro
0,7 s depois da última alteração, por meio de um arquivo temporário, então uma gravação que falha
deixa o anterior. Se o arquivo não puder ser lido, a lista mostra o erro com o caminho, a linha e
a coluna, e o arquivo fica como está: corrija-o e pressione [[ui:emu.reload]]. Pressione
[[ui:emu.reload]] também depois de editá-lo à mão. Sem arquivo, o conjunto inicial é gravado de
novo.

```json
{
  "version": 1,
  "emulators": [
    {
      "id": "orders-api",
      "note": "Stands in for the orders service.",
      "emulator": {
        "name": "Orders API",
        "bind": "127.0.0.1:18080",
        "protocol": "http",
        "routes": [
          { "method": "GET", "path": "/orders/:id",
            "responses": [{ "body": "{\"id\":\"{{request.params.id}}\",\"state\":\"open\"}" }] },
          { "method": "POST", "path": "/orders", "order": "sequence",
            "responses": [{ "status": 503 }, { "status": 201, "body": "{\"id\":\"{{uuid}}\"}" }] }
        ],
        "outage": { "up_ms": 20000, "down_ms": 2000, "fault": "unavailable" }
      }
    }
  ]
}
```

Só o objeto `emulator` já é um documento que o `signallab emulate` também lê.

## Em experimentos e scripts {#elsewhere}

- Em um experimento, um nó [[ui:exp.node.emulator]] abre o emulador dele antes da primeira etapa e
  responde até o fim da execução; o que ele recebeu é contado no relatório. Um emulador HTTP ali é
  também o que [[ui:exp.node.wait_http]] ([Nós](../experiments/nodes.md#node-wait_http)) escuta, e
  um emulador OSC ou UDP compartilha a porta dele com as esperas da execução. Dois emuladores do
  mesmo transporte em um mesmo experimento não podem compartilhar uma porta. Veja
  [Nós](../experiments/nodes.md#node-emulator) e [Falhas](../experiments/faults.md).
- `signallab emulate` executa emuladores a partir de arquivos ou desta biblioteca até
  <kbd>Ctrl</kbd>+<kbd>C</kbd> ou `--for`, imprimindo o que eles respondem; veja
  [A linha de comando](../automation/cli.md#cli-emulate).

## Relacionados {#related}

- [Inspetor](inspector.md) — cada troca, decodificada.
- [Degradação](impairment.md) — uma rede ruim entre o seu sistema e um emulador.
- [Dados e modelos](../experiments/data.md#templates)

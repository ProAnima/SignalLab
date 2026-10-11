---
title: Nós
description: Cada tipo de nó de experimento — o que faz, os campos com padrões e limites, as saídas, as configurações que aceita e como aparece em um arquivo de experimento.
---

# Referência dos nós

Todos os tipos de nó que um experimento pode ter, nos grupos do menu de adição:
[ações](#actions), [esperas](#waits), [emulação](#emulation),
[falhas](#faults), [dados](#data), [verificações](#checks) e [fluxo](#flow). Como
adicioná-los e ligá-los está em [O editor](index.md); `signallab nodes` imprime o
mesmo catálogo em JSON, para scripts e assistentes ([A linha de
comando](../automation/cli.md)).

## Como ler esta página {#reading}

Cada nó tem uma tabela de campos:

- **Campo** é o nome no painel de propriedades; **No arquivo** é a chave no
  JSON do experimento.
- **Padrão** é o que um nó recebe quando você o adiciona no editor. Onde um arquivo
  pode omitir uma chave, o valor que ela então assume é dado como *se ausente*;
  as demais chaves são obrigatórias em um arquivo.
- **Modelos**: *sim* — o campo aceita `{{templates}}`: parâmetros,
  variáveis definidas antes, segredos e geradores, resolvidos quando a etapa executa
  ([Dados e modelos](data.md)). *Só parâmetros* — ele é aberto antes da
  primeira etapa, quando só os parâmetros são conhecidos. *Não* — o valor é tomado
  como foi escrito.

Os tempos são em milissegundos. Os limites são verificados antes de uma execução começar; um campo
fora do intervalo impede o experimento de executar e é mostrado no nó.

## Um nó em um arquivo {#file-shape}

Em um arquivo de experimento, um nó é um objeto com um `id` (único no
experimento), o `type`, o lugar dele no canvas (`x`, `y`, zero ou mais), os
campos e as configurações que usa (`retry`, `repeat`, `load`, omitidas quando
desligadas). Um fio é uma aresta da saída de um nó (`port`, `next` se ausente) para
outro nó:

```json
{
  "nodes": [
    { "id": "start", "type": "start", "x": 40, "y": 80 },
    { "id": "ping", "type": "udp", "x": 270, "y": 80, "target": "127.0.0.1:9000", "text": "PING",
      "retry": { "attempts": 3, "delay_ms": 500, "backoff": "fixed" } },
    { "id": "end", "type": "end", "x": 500, "y": 80 }
  ],
  "edges": [
    { "from": "start", "to": "ping", "port": "next" },
    { "from": "ping", "to": "end", "port": "next" }
  ]
}
```

Os exemplos a seguir mostram um nó cada, como um arquivo o guarda.

## Configurações compartilhadas por vários nós {#settings}

Elas são ativadas na parte de baixo das propriedades de um nó. Qual nó aceita
qual está listado em cada nó.

| Configuração | Quem aceita | O que faz |
| --- | --- | --- |
| [Nova tentativa](#retry) | Nós que enviam ou escutam: [[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]], [[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_connect]], [[ui:exp.node.ws_send]] e todas as esperas | Tenta de novo quando a etapa falha |
| [Repetição](#repeat) | Nós que enviam: [[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]], [[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_send]] | Envia de novo e de novo, um número de vezes ou por um tempo |
| [Carga](#load) | [[ui:exp.node.http]] | Envia a requisição segundo um perfil de carga, medida e julgada por limiares |
| [Espera por uma resposta](#reply) | [[ui:exp.node.osc]], [[ui:exp.node.udp]] | Envia e aguarda a resposta na mesma etapa |

### Nova tentativa {#retry}

[[ui:exp.retryOn]]: quando a etapa falha — sem conexão, um tempo esgotado, uma espera
sem nenhuma correspondência — ela faz uma pausa e executa de novo. Cada tentativa que falha é uma linha
na linha do tempo; a etapa falha quando a última tentativa falha. Um modelo que
não pode ser resolvido não é tentado de novo. [[ui:common.stop]] também encerra uma pausa.

| Campo | No arquivo | O quê | Padrão e limites |
| --- | --- | --- | --- |
| [[ui:exp.attempts]] | `retry.attempts` | Tentativas no total, a primeira incluída | 3; 2–10 no editor (um arquivo também pode dizer 1) |
| [[ui:exp.retryDelay]] | `retry.delay_ms` | A pausa antes da segunda tentativa | 500; 0–60.000 |
| [[ui:exp.backoff]] | `retry.backoff` | [[ui:exp.backoff.fixed]] (`fixed`): a mesma pausa todas as vezes; [[ui:exp.backoff.exponential]] (`exponential`): o dobro do tempo depois de cada falha | `fixed` (também se ausente) |

Nenhuma pausa passa de 60 segundos, por mais que dobre. Uma espera cuja
saída [[ui:exp.portTimeout]] tem um fio não falha por tempo esgotado — ela
sai por essa saída — então nesse caso não é tentada de novo.

### Repetição {#repeat}

[[ui:exp.repeatOn]]: o nó envia de novo e de novo — um sinal de vida, uma consulta, um
fluxo contínuo — sem um laço no grafo. Cada envio lê os modelos dele
de novo (`{{counter}}` é o número dele, `{{now}}` o horário), e a Nova tentativa, quando ativa,
vale para cada envio. A etapa é aprovada quando todos os envios foram; um envio que falha
de vez reprova a etapa. A linha do tempo informa o progresso no máximo uma vez por segundo.

| Campo | No arquivo | O quê | Padrão e limites |
| --- | --- | --- | --- |
| [[ui:exp.repeatBy]] | `repeat.until` | [[ui:exp.repeatBy.count]] (`count`) ou [[ui:exp.repeatBy.duration]] (`duration`) | `count` (também se ausente) |
| [[ui:exp.repeatCount]] | `repeat.count` | Envios no total, o primeiro incluído | 10 (também se ausente); 2–10.000 |
| [[ui:exp.repeatDuration]] | `repeat.duration_ms` | Por quanto tempo continuar enviando, a partir do primeiro envio | 10.000 (também se ausente); 1–300.000 |
| [[ui:exp.repeatInterval]] | `repeat.interval_ms` | A pausa entre dois envios | 1.000; 10–60.000; obrigatório em um arquivo |
| [[ui:exp.repeatJitter]] | `repeat.jitter_ms` | Cada pausa até este tanto mais longa, sorteada a partir da semente da execução | 0 (também se ausente); 0–60.000 |

As repetições precisam caber nos 300 segundos de uma execução, e *por um tempo* exige
menos de 10.000 envios (o tempo dividido pelo intervalo).

### Carga {#load}

[[ui:exp.loadOn]], apenas em um [[ui:exp.node.http]]: a requisição é enviada segundo um perfil —
uma taxa constante, uma rampa, degraus, um pico ou chegadas aleatórias — com até 512
em andamento ao mesmo tempo (32 por padrão), e medida: latências, erros, a taxa
alcançada. Os limiares decidem se a etapa é aprovada. A carga substitui Repetição e
Nova tentativa (uma requisição que falha é contada, não repetida) e não deixa resposta para
as verificações que vêm depois. Os campos e os resultados estão em [Teste de carga](load.md).

### Espera por uma resposta {#reply}

[[ui:exp.expectReply]], em um [[ui:exp.node.osc]] ou um [[ui:exp.node.udp]]: a mensagem é enviada
a partir da porta em que a resposta é aguardada, de modo que um dispositivo que responde ao remetente
é ouvido, e a etapa só é aprovada quando uma resposta correspondente chega a tempo. Sem resposta,
a etapa falha — a Nova tentativa envia de novo. A resposta é guardada em uma variável, como a de
uma espera.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.replyOn]] | `reply.bind` | `IP:port` de onde enviar e em que escutar; a porta 0 pega qualquer porta livre | `0.0.0.0:0` | Não |
| [[ui:exp.replyAddress]] (OSC) | `reply.address` | O padrão de endereço da resposta, como em [[[ui:exp.node.wait_osc]]](#node-wait_osc) | `/*` | Sim |
| [[ui:exp.argRules]] (OSC) | `reply.args` | Regras de argumento, como em [[ui:exp.node.wait_osc]] | nenhuma; no máximo 16 | Valores: sim |
| [[ui:exp.replyMode]] (UDP) | `reply.mode` | `any`, `contains`, `regex` ou `hex` — veja [Correspondência de carga útil](#payload-matching) | `any` (também se ausente) | Não |
| [[ui:field.pattern]] (UDP) | `reply.pattern` | O que a resposta deve conter ou com o que deve corresponder | vazio; obrigatório, exceto com `any` | Sim |
| [[ui:exp.waitTimeout]] | `reply.timeout_ms` | Quanto tempo aguardar | 2.000 (também se ausente); 1–120.000 | Não |
| [[ui:exp.replyVariable]] | `reply.variable` | A variável em que a resposta é guardada | `reply` (também se ausente) | Não |

A porta da resposta é aberta antes da primeira etapa, como a de uma espera.

## Ações {#actions}

Nós que enviam. Uma espera depois de uma ação conta as mensagens a partir do momento em que a
ação começou.

### Requisição HTTP {#node-http}

Envia uma requisição HTTP e guarda a resposta para as verificações, os desvios e os
nós [[ui:exp.node.extract]] que vêm depois.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.method]] | `request.method` | GET, HEAD, POST, PUT, PATCH, DELETE ou OPTIONS (um arquivo pode dar qualquer método) | `GET` | Não |
| URL | `request.url` | Uma URL `http://` ou `https://` | `http://127.0.0.1:8080/` | Sim |
| [[ui:common.timeoutMs]] | `request.timeout_ms` | Para a troca inteira | 4.000 (10.000 se ausente); 1–120.000 | Não |
| [[ui:exp.headers]] | `request.headers` | `[[name, value], …]`; uma linha com nome vazio é ignorada | nenhum | Sim, nomes e valores |
| [[ui:exp.body]] | `request.body` | Texto, ou `null` para nenhum | `null` | Sim |
| [[ui:field.auth]] | `request.auth` | [[ui:http.auth.none]], [[ui:http.auth.basic]], [[ui:http.auth.bearer]] ou [[ui:http.auth.digest]], com [[ui:field.username]] e [[ui:field.password]], ou [[ui:field.token]] | nenhuma | Sim |

- Qualquer resposta aprova a etapa, inclusive 404 e 500: verifique o status com
  [[[ui:exp.node.assert_status]]](#node-assert_status) ou desvie por ele com
  [[[ui:exp.node.branch_status]]](#node-branch_status). Uma requisição que não obtém resposta — recusada, um
  tempo esgotado, um nome que não resolve, um certificado em que não se confia —
  reprova a etapa.
- Os redirecionamentos são seguidos, dez no máximo. Os certificados `https://` são verificados.
- O corpo da resposta é guardado até 256 KiB para as verificações; um corpo maior é cortado
  nesse ponto (as verificações avisam quando o que procuram pode estar depois do corte).
- O Digest responde ao desafio 401 do servidor e envia a requisição de novo. As
  credenciais vão só na requisição: etapas, relatórios e o Inspetor nunca
  mostram o cabeçalho `Authorization`. Escreva uma senha como `{{secret.NAME}}`.
- Enquanto o experimento mantém cookies (ativado por padrão, em
  [[ui:exp.params]]), o que os servidores definem é enviado de volta nas requisições
  posteriores da execução a eles.

Saídas: [[ui:exp.outputPort]]. Configurações: Nova tentativa, Repetição, Carga.

```json
{ "id": "cue", "type": "http", "x": 270, "y": 80,
  "request": { "method": "POST", "url": "{{api}}/cue", "headers": [["Content-Type", "application/json"]],
               "body": "{\"cue\": 1}", "timeout_ms": 5000,
               "auth": { "scheme": "bearer", "token": "{{secret.API_TOKEN}}" } } }
```

Veja também [HTTP](../protocols/http.md).

### Mensagem TCP {#node-tcp}

Conecta a um host por TCP, escreve a carga útil, aguarda até 250 ms pelos
primeiros bytes de uma resposta (lê no máximo 1.024 bytes, uma vez) e fecha a
conexão. O tamanho da resposta é informado, não verificado.

No [Inspetor](../tools/inspector.md), a etapa são dois quadros `tcp` com a
origem `experiment`: a carga útil escrita e, quando veio uma, a resposta lida.
Os segredos em uso são mascarados nos dois, como em qualquer quadro.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.host]] | `host` | Um nome de host ou um endereço IP | `127.0.0.1` | Sim |
| [[ui:exp.port]] | `port` | | 9000; 1–65.535 | Não |
| [[ui:common.timeoutMs]] | `timeout_ms` | Para conectar, escrever e a resposta juntos | 4.000 (também se ausente); 1–120.000 | Não |
| [[ui:exp.payload]] | `payload` | O texto escrito depois de conectar, em UTF-8 | `hello` | Sim |

A etapa falha quando a conexão é recusada, o nome não resolve ou o
tempo acaba. Saídas: [[ui:exp.outputPort]]. Configurações: Nova tentativa, Repetição.
[[ui:exp.sendNow]] conecta e escreve a carga útil uma vez, e o resultado do nó
diz quantos bytes foram enviados e voltaram.

```json
{ "id": "go", "type": "tcp", "x": 270, "y": 80, "host": "127.0.0.1", "port": 5000, "payload": "GO\r\n", "timeout_ms": 2000 }
```

### Mensagem OSC {#node-osc}

Envia uma mensagem OSC 1.0 por UDP.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:common.target]] | `target` | `IP:port` ou `host:port`; um nome de host é consultado quando a etapa envia, e o endereço IPv4 dele é usado quando há um | `127.0.0.1:9000` | Sim |
| [[ui:common.address]] | `address` | Começa com `/` | `/test` | Sim |
| [[ui:exp.arguments]] | `args` | `[{ "type", "value" }, …]` — `int`, `float`, `str`, `long`, `double`, `bool`, `blob` (bytes), `nil` (sem valor) | nenhum | Valores de texto (`str`): sim |
| [[ui:exp.expectReply]] | `reply` | Opcional: enviar e aguardar a resposta — veja [Espera por uma resposta](#reply) | desligado | |

Saídas: [[ui:exp.outputPort]]; com uma resposta esperada, ela só é seguida quando
a resposta veio. Configurações: Nova tentativa, Repetição, uma resposta. ⚡ [[ui:exp.routeThrough]] nas
propriedades dele coloca um [[[ui:exp.node.impairment]]](#node-impairment) na frente dele.

```json
{ "id": "fader", "type": "osc", "x": 270, "y": 80, "target": "{{device}}", "address": "/fader/1",
  "args": [{ "type": "float", "value": 0.75 }] }
```

Veja também [OSC](../protocols/osc.md).

### Datagrama UDP {#node-udp}

Envia uma carga útil de texto como um datagrama UDP a um ou mais destinos.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:common.target]] | `target` | `IP:port` ou `host:port`; vários separados por vírgulas, pontos e vírgulas ou quebras de linha recebem cada um o datagrama. Um nome de host é consultado quando a etapa envia, e o endereço IPv4 dele é usado quando há um | `127.0.0.1:9000` | Sim |
| [[ui:exp.payload]] | `text` | A carga útil, em UTF-8 | `hello`; no máximo 65.507 bytes | Sim |
| [[ui:exp.expectReply]] | `reply` | Opcional: enviar e aguardar a resposta — veja [Espera por uma resposta](#reply) | desligado | |

A etapa falha se algum destino não puder ser alcançado. Saídas: [[ui:exp.outputPort]].
Configurações: Nova tentativa, Repetição, uma resposta.

```json
{ "id": "ping", "type": "udp", "x": 270, "y": 80, "target": "{{device}}", "text": "PING {{run.id}}",
  "reply": { "bind": "0.0.0.0:0", "mode": "contains", "pattern": "PONG", "timeout_ms": 1000, "variable": "pong" } }
```

### Publicação MQTT {#node-mqtt}

Conecta a um broker MQTT, publica uma mensagem e desconecta. A
conexão é MQTT 3.1.1 sobre TCP simples, com sessão limpa e sem nome de usuário
nem senha. Conectar, publicar e a confirmação do broker precisam acontecer
em 15 segundos.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.broker]] | `host` | O nome de host ou endereço do broker | `127.0.0.1` | Sim |
| [[ui:exp.port]] | `port` | | 1883; 1–65.535 | Não |
| [[ui:exp.topic]] | `topic` | Sem curingas (`+`, `#`) | `lab/test` | Sim |
| [[ui:exp.payload]] | `payload` | A mensagem, como texto | `hello` | Sim |
| QoS | `qos` | 0, 1 ou 2 | 0 | Não |
| [[ui:exp.retain]] | `retain` | `true`: o broker a guarda como o valor do tópico | `false` | Não |

As seis chaves são obrigatórias em um arquivo. A etapa falha quando o broker não pode ser
alcançado ou recusa a conexão ou a mensagem. Saídas:
[[ui:exp.outputPort]]. Configurações: Nova tentativa, Repetição.

```json
{ "id": "light", "type": "mqtt", "x": 270, "y": 80, "host": "{{broker}}", "port": 1883,
  "topic": "lab/light/1/set", "payload": "on", "qos": 1, "retain": false }
```

Veja também [MQTT](../protocols/mqtt.md).

### Conectar WebSocket {#node-ws_connect}

Abre um WebSocket pelo resto da execução, ou até um
[[[ui:exp.node.ws_close]]](#node-ws_close). O que chega a partir daí é guardado para as etapas
[[[ui:exp.node.wait_ws]]](#node-wait_ws) nele. A URL e os cabeçalhos são resolvidos quando
a etapa executa, então um token extraído antes pode estar neles. Executado de novo — em um
[[ui:exp.node.loop]] — ele fecha a conexão anterior primeiro e abre uma nova. Quando a execução
termina, de qualquer jeito, as conexões dela são fechadas com um quadro de fechamento.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| URL | `url` | Uma URL `ws://` ou `wss://` | `ws://127.0.0.1:9001/` | Sim |
| [[ui:exp.headers]] | `headers` | `[[name, value], …]` enviados com a requisição de upgrade | nenhum | Sim, nomes e valores |
| [[ui:exp.wsProtocols]] | `protocols` | Subprotocolos a oferecer, em ordem de preferência; o servidor escolhe um | nenhum | Não |
| [[ui:common.timeoutMs]] | `timeout_ms` | Para conectar e fazer o upgrade | 5.000 (10.000 se ausente); 1–120.000 | Não |

`wss://` confia nos mesmos certificados que `https://`. A etapa falha quando a
conexão ou o upgrade falha; o status do servidor está no motivo.
Saídas: [[ui:exp.outputPort]]. Configurações: Nova tentativa (não Repetição).

```json
{ "id": "socket", "type": "ws_connect", "x": 270, "y": 80, "url": "ws://127.0.0.1:9001/chat",
  "headers": [["Authorization", "Bearer {{token}}"]], "protocols": ["chat.v1"], "timeout_ms": 5000 }
```

Veja também [WebSocket](../protocols/websocket.md).

### Enviar por WebSocket {#node-ws_send}

Envia uma mensagem pela conexão que um [[ui:exp.node.ws_connect]] abriu.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | O id de um nó [[ui:exp.node.ws_connect]] deste experimento | o primeiro | Não |
| [[ui:exp.wsFormat]] | `binary` | [[ui:exp.wsText]] (`false`) ou [[ui:exp.wsBinary]] (`true`): a carga útil são bytes escritos em hex, `de ad be ef` | `false` (também se ausente) | Não |
| [[ui:exp.payload]] | `text` | A mensagem | `hello`; no máximo 16 MiB | Sim |

O connect precisa vir antes do envio no caminho dele; um envio cuja conexão
não está aberta falha. As respostas contam a partir do momento em que a mensagem é escrita.
Saídas: [[ui:exp.outputPort]]. Configurações: Nova tentativa, Repetição.

```json
{ "id": "hello", "type": "ws_send", "x": 500, "y": 80, "connection": "socket",
  "text": "{\"type\":\"ping\",\"id\":\"{{uuid}}\"}", "binary": false }
```

### Fechar WebSocket {#node-ws_close}

Fecha uma conexão com um handshake de fechamento. A linha do tempo diz quem a fechou:
esta etapa, o servidor antes (com o código dele) ou uma conexão que tinha caído.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | O id de um nó [[ui:exp.node.ws_connect]] | o primeiro | Não |
| [[ui:field.code]] | `code` | 1000 (normal), ou 3000–4999 para os próprios de uma aplicação | 1000 (também se ausente) | Não |
| [[ui:field.reason]] | `reason` | Enviado com o código | vazio; no máximo 123 bytes, depois dos modelos | Sim |

Saídas: [[ui:exp.outputPort]]. Sem configurações.

```json
{ "id": "bye", "type": "ws_close", "x": 960, "y": 80, "connection": "socket", "code": 1000, "reason": "done" }
```

### Marcador de log {#node-log}

Escreve uma linha na linha do tempo e no relatório — um ponto de controle, ou os valores que uma
execução alcançou.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.logMessage]] | `message` | O texto | `Check point`; no máximo 10.000 caracteres | Sim |

Saídas: [[ui:exp.outputPort]]. Sem configurações.

```json
{ "id": "ready", "type": "log", "x": 500, "y": 80, "message": "device {{device}} ready" }
```

## Esperas {#waits}

O grupo [[ui:exp.group.observe]]: nós que aguardam a chegada de algo.
Eles compartilham estas regras:

- **Escutam desde o início da execução.** A porta ou a assinatura de broker
  de uma espera é aberta antes da primeira etapa, de modo que um dispositivo que responde
  mais rápido do que a etapa seguinte começa não se perde. Duas esperas no mesmo endereço
  compartilham um socket.
- **Contam a partir da última ação no ramo delas.** Uma mensagem que
  chegou antes da última requisição do ramo não é resposta a ela; antes de qualquer
  ação, tudo o que veio desde o início da execução conta.
- **A primeira mensagem correspondente é tomada.** Uma mensagem que uma espera tomou não é vista
  por outra.
- **[[ui:exp.portMatched]] ou [[ui:exp.portTimeout]].** Em uma correspondência, a
  mensagem é guardada na variável da espera e o fluxo segue
  [[ui:exp.portMatched]]. Quando o tempo acaba, ele segue
  [[ui:exp.portTimeout]] se essa saída tem um fio; caso contrário a etapa falha,
  dizendo quantas outras mensagens chegaram.
- Cada socket guarda as últimas 1.024 mensagens (e 64 MiB); as mais antigas são
  descartadas, e um tempo esgotado diz quantas foram.
- [[ui:exp.listenNow]] escuta só com essa etapa, a partir de agora.

Saídas: [[ui:exp.portMatched]] (obrigatória), [[ui:exp.portTimeout]] (opcional).
Configurações: Nova tentativa.

### Correspondência de carga útil {#payload-matching}

[[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_ws]]
e uma resposta UDP escolhem como a carga útil deve ser:

| Opção | No arquivo | Corresponde quando a carga útil |
| --- | --- | --- |
| [[ui:exp.mode.any]] | `any` | é qualquer coisa |
| [[ui:exp.mode.contains]] | `contains` | lida como texto UTF-8, contém o padrão (diferencia maiúsculas de minúsculas) |
| [[ui:exp.mode.regex]] | `regex` | lida como texto UTF-8, corresponde à expressão regular |
| [[ui:exp.mode.hex]] | `hex` | contém os bytes, escritos como pares hex: `de ad be ef`, `deadbeef`, `0xde,0xad`, `DE:AD` |

A mensagem correspondente é guardada como um objeto. As etapas seguintes leem os campos dela como
`{{reply.text}}` (com o nome da variável no lugar de `reply`):

| Campo | O quê |
| --- | --- |
| `text` | A carga útil como texto |
| `hex`, `bytes` | A carga útil em hex (os primeiros 1.024 bytes dela) e o tamanho em bytes |
| `match` | O que correspondeu: o texto, o primeiro grupo da expressão regular (ou a correspondência inteira), ou os bytes |
| `from` | O `IP:port` do remetente |
| `ms` | Milissegundos desde a última ação do ramo (ou o início da execução) até a mensagem |
| `topic` | [[ui:exp.node.wait_mqtt]]: o tópico em que foi publicada |
| `json`, `kind` | [[ui:exp.node.wait_ws]]: a mensagem lida como JSON (`null` quando não é) e `text` ou `binary` |

### Aguardar OSC {#node-wait_osc}

Aguarda uma mensagem OSC cujo endereço corresponde a um padrão e cujos argumentos
atendem a todas as regras. Em um bundle, a primeira mensagem que corresponde é a tomada.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | `IP:port` em que escutar; `0.0.0.0` para todas as placas de rede | `127.0.0.1:9001` | Não |
| [[ui:exp.addressPattern]] | `address` | `*` quaisquer caracteres, `?` um, `[0-9]` um conjunto (`[!0-9]` fora dele), `{ping,pong}` qualquer um dos dois; os curingas ficam dentro de um segmento entre `/` | `/pong`; no máximo 512 caracteres | Sim |
| [[ui:exp.argRules]] | `args` | `[{ "index", "op", "value" }, …]`: o argumento `index` comparado com `value` por `op` ([Comparações](#comparisons)); todas devem valer | nenhuma; no máximo 16, índice 0–63 | Valores: sim |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2.000 (também se ausente); 1–120.000 | Não |
| [[ui:exp.replyVariable]] | `variable` | Onde a mensagem é guardada | `reply` (também se ausente) | Não |

Um argumento é comparado como texto: números como foram escritos, strings sem aspas,
`true`/`false`, um blob em hex. Uma regra sobre um argumento que a mensagem não tem
não vale. A mensagem guardada tem `address`, `args` (`{{reply.args[0]}}`),
`from` e `ms`.

```json
{ "id": "status", "type": "wait_osc", "x": 500, "y": 80, "bind": "0.0.0.0:9001", "address": "/status",
  "args": [{ "index": 0, "op": "eq", "value": "ready" }], "timeout_ms": 5000, "variable": "reply" }
```

### Aguardar UDP {#node-wait_udp}

Aguarda um datagrama UDP cuja carga útil corresponde.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | `IP:port` em que escutar | `127.0.0.1:9001` | Não |
| [[ui:exp.waitMode]] | `mode` | Veja [Correspondência de carga útil](#payload-matching) | `contains` (`any` se ausente) | Não |
| [[ui:field.pattern]] | `pattern` | O que a carga útil deve conter ou com o que deve corresponder | `pong`; obrigatório, exceto com `any` | Sim |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2.000 (também se ausente); 1–120.000 | Não |
| [[ui:exp.replyVariable]] | `variable` | | `reply` (também se ausente) | Não |

```json
{ "id": "ready", "type": "wait_udp", "x": 500, "y": 80, "bind": "0.0.0.0:9002", "mode": "contains",
  "pattern": "READY", "timeout_ms": 5000, "variable": "reply" }
```

### Aguardar MQTT {#node-wait_mqtt}

Aguarda uma mensagem publicada em um tópico de um broker, cuja carga útil corresponde.
A execução conecta e assina antes da primeira etapa. As mensagens retidas que o
broker reenvia ao assinar são ignoradas: só conta o que é publicado depois que a
execução começou.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.broker]] | `host` | O broker | `127.0.0.1` | Só parâmetros |
| [[ui:exp.port]] | `port` | | 1883; 1–65.535 | Não |
| [[ui:exp.topicFilter]] | `topic` | Um filtro: `+` é qualquer nível, `#` tudo abaixo (só no fim) | `lab/#` | Só parâmetros |
| [[ui:exp.waitMode]] | `mode` | Veja [Correspondência de carga útil](#payload-matching) | `any` (também se ausente) | Não |
| [[ui:field.pattern]] | `pattern` | | vazio; obrigatório, exceto com `any` | Sim |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2.000 (também se ausente); 1–120.000 | Não |
| [[ui:exp.replyVariable]] | `variable` | | `reply` (também se ausente) | Não |

```json
{ "id": "state", "type": "wait_mqtt", "x": 500, "y": 80, "host": "{{broker}}", "port": 1883,
  "topic": "lab/+/state", "mode": "contains", "pattern": "on", "timeout_ms": 5000, "variable": "reply" }
```

### Aguardar requisição HTTP {#node-wait_http}

Aguarda uma requisição HTTP — um webhook, um callback — ao
[[[ui:exp.node.emulator]]](#node-emulator) da execução nesse endereço ou, quando a execução não tem emulador
HTTP ali, a um ouvinte próprio da execução que responde a toda requisição com
204. A requisição precisa corresponder ao método, ao caminho e a todas as condições.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | `IP:port` | `127.0.0.1:18080` — onde um novo [[ui:exp.node.emulator]] escuta | Não |
| [[ui:exp.method]] | `method` | Um método, ou [[ui:emu.methodAny]] (`ANY`); GET também aceita HEAD | `ANY` (também se ausente) | Não |
| [[ui:exp.path]] | `path` | `/hooks/:name` nomeia um segmento (`{{request.params.name}}`); um `/*` final pega o resto | `/*` (também se ausente); no máximo 512 caracteres | Sim |
| [[ui:emu.conditions]] | `when` | `[{ "on", "name", "op", "value" }, …]` sobre um `header`, um parâmetro de `query`, o `body` ou um caminho `json`; todas devem valer | nenhuma; no máximo 16 | Sim, nomes e valores |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 5.000 (2.000 se ausente); 1–120.000 | Não |
| [[ui:exp.replyVariable]] | `variable` | | `request` (também se ausente) | Não |

A requisição guardada tem `method`, `path`, `query`, `headers`, `body`, `json`,
`params`, `from` e `ms`: `{{request.json.event}}`, `{{request.headers.x-key}}`.

```json
{ "id": "hook", "type": "wait_http", "x": 500, "y": 80, "bind": "127.0.0.1:18081", "method": "POST",
  "path": "/hooks/:name", "when": [{ "on": "json", "name": "$.event", "op": "eq", "value": "deploy" }],
  "timeout_ms": 5000, "variable": "request" }
```

### Aguardar WebSocket {#node-wait_ws}

Aguarda uma mensagem na conexão que um [[ui:exp.node.ws_connect]] abriu, cuja carga útil
corresponde. As mensagens desde a última ação no ramo contam — o próprio connect,
um envio ou qualquer outra requisição.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | O id de um nó [[ui:exp.node.ws_connect]] | o primeiro | Não |
| [[ui:exp.waitMode]] | `mode` | Veja [Correspondência de carga útil](#payload-matching) | `any` (também se ausente) | Não |
| [[ui:field.pattern]] | `pattern` | | vazio; obrigatório, exceto com `any` | Sim |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2.000 (também se ausente); 1–120.000 | Não |
| [[ui:exp.replyVariable]] | `variable` | | `reply` (também se ausente) | Não |

Uma mensagem JSON pode ser lida campo a campo: `{{reply.json.type}}`. O connect
precisa vir antes da espera no caminho dela.

```json
{ "id": "pong", "type": "wait_ws", "x": 730, "y": 80, "connection": "socket", "mode": "contains",
  "pattern": "pong", "timeout_ms": 3000, "variable": "reply" }
```

## Emulação {#emulation}

### Emulador {#node-emulator}

Faz o papel de uma dependência — uma API HTTP, um dispositivo OSC, UDP ou TCP, um broker MQTT —
durante toda a execução. Ele abre antes da primeira etapa e responde até a execução
terminar; no fluxo, a etapa é aprovada na hora. O que ele recebeu é contado, regra por
regra, no relatório da execução.

| Campo | No arquivo | O quê | Padrão |
| --- | --- | --- | --- |
| [[ui:emu.edit]] | `emulator` | O emulador: `name`, `bind` (`IP:port`), `protocol` (`http`, `osc`, `udp`, `tcp`, `mqtt`), as rotas ou regras dele e uma `outage` opcional | Uma API HTTP chamada *API* em `127.0.0.1:18080` que responde a `/health` |

As propriedades mostram em uma linha o que ele interpreta. [[ui:emu.edit]] abre as regras dele,
o mesmo editor da tela [[[ui:nav.emulators]]](../tools/emulators.md);
[[ui:emu.toLibrary]] guarda uma cópia na biblioteca de emuladores, e
[[ui:emu.fromLibrary]] substitui este por uma cópia dela. As regras — rotas,
respostas, falhas, quedas — estão descritas lá.

- Um emulador HTTP também é o que um [[[ui:exp.node.wait_http]]](#node-wait_http) no
  endereço dele lê; um emulador OSC ou UDP divide a porta dele com as esperas da execução ali.
- Dois emuladores do mesmo transporte não podem compartilhar uma porta em uma execução.
- [[[ui:exp.node.emulator_state]]](#node-emulator_state) o derruba e o reativa.

Saídas: [[ui:exp.outputPort]]. Sem configurações.

```json
{ "id": "api", "type": "emulator", "x": 270, "y": 80,
  "emulator": { "name": "Orders API", "bind": "127.0.0.1:18080", "protocol": "http",
    "routes": [{ "method": "GET", "path": "/orders/:id", "order": "sequence",
                 "responses": [{ "status": 503 }, { "status": 200, "body": "{\"id\":\"{{request.params.id}}\"}" }] }] } }
```

## Falhas {#faults}

Nós que quebram as coisas na hora certa. Um ramo de nós [[ui:exp.node.delay]] e estes ao lado do
tráfego funciona como um cronograma; [Falhas em um cronograma](faults.md) mostra como.

### Degradação {#node-impairment}

Um retransmissor de degradação para a execução inteira: o sistema em teste envia para (ou
se conecta a) [[ui:exp.relayListen]] em vez do destino real; o retransmissor
encaminha para [[ui:exp.relayTarget]], e as respostas voltam pelo mesmo caminho,
degradadas pelo perfil. Ele abre antes da primeira etapa e fecha quando a execução
termina, de qualquer jeito, de modo que nada fica degradado; no fluxo, a etapa é aprovada na hora.
Toda decisão sorteia a partir da semente da execução: a mesma semente e o mesmo tráfego
têm o mesmo destino.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.relayListen]] | `listen` | `IP:port` para o qual o sistema em teste envia | `127.0.0.1:9010` | Só parâmetros |
| [[ui:exp.relayTarget]] | `target` | `IP:port` do destino real, ou `host:port` — um nome de host é consultado quando a execução começa, e um nome que não é encontrado interrompe a execução neste nó | `127.0.0.1:9000` | Só parâmetros |
| [[ui:ns.protocol]] | `protocol` | UDP (`udp`): cada datagrama tem o seu próprio destino; TCP (`tcp`): cada conexão é ligada a uma própria até o destino, e os dois fluxos são degradados | UDP (`udp` se ausente) | Não |
| [[ui:ns.preset]] e os valores abaixo dela | `profile` | O que o retransmissor faz com o tráfego — veja [O perfil](#impair-profile) | [[ui:ns.preset.lan]] (sem degradação se ausente) | Não |

O endereço de escuta de um retransmissor não pode ser outro socket da execução, e os retransmissores não
podem encaminhar uns aos outros em círculo. Um destino dado por nome é seguido depois que
é consultado, então um círculo por um nome interrompe a execução quando ela começa. O relatório conta cada fase de um retransmissor
à parte.

Saídas: [[ui:exp.outputPort]]. Sem configurações.

```json
{ "id": "relay", "type": "impairment", "x": 270, "y": 80, "listen": "127.0.0.1:9010", "target": "{{device}}",
  "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } }
```

#### O perfil {#impair-profile}

Um chip de predefinição — [[ui:ns.preset.lan]], [[ui:ns.preset.wifi]],
[[ui:ns.preset.4g]], [[ui:ns.preset.satellite]],
[[ui:ns.preset.intermittent]], [[ui:ns.preset.offline]] — preenche todos os valores;
altere qualquer um depois. Um retransmissor lê só os valores do protocolo dele; em um
arquivo, toda chave pode ser omitida (zero, desligado).

| Campo | No arquivo | O quê | Limites | Protocolo |
| --- | --- | --- | --- | --- |
| — | `name` | Um rótulo para a linha do tempo e o relatório: a chave de uma predefinição (`lan`, `wifi`, `4g`, `satellite`, `intermittent`, `offline`) ou a sua própria | no máximo 60 caracteres | ambos |
| [[ui:ns.offline]] | `offline` | Nada passa | `true` / `false` | ambos |
| [[ui:ns.latency]] | `latency_ms` | Atraso somado a cada pacote, ou trecho de um fluxo | 0–60.000 (o controle deslizante vai até 1.000) | ambos |
| [[ui:ns.jitter]] | `jitter_ms` | Um atraso extra aleatório até este valor; um fluxo TCP continua em ordem | 0–60.000 (o controle deslizante vai até 500) | ambos |
| [[ui:ns.rate]] | `rate_kbps` | Um limite de largura de banda, 0 para nenhum. UDP: depois de um segundo de fila, os datagramas são descartados por limitação; TCP: o remetente é desacelerado, nada é descartado | 0, ou 8–10.000.000 | ambos |
| [[ui:ns.loss]] | `loss` | A chance de um datagrama ser descartado | 0–1 (o controle deslizante mostra %) | UDP |
| [[ui:ns.burst]], [[ui:ns.burstLength]] | `burst_start`, `burst_length` | A chance de começar uma rajada de perda e quantos datagramas ela dura em média | 0–1; 1–1.000 quando as rajadas estão ativas | UDP |
| [[ui:ns.duplicate]] | `duplicate` | A chance de um datagrama ser enviado duas vezes | 0–1 | UDP |
| [[ui:ns.corrupt]] | `corrupt` | A chance de um bit de um datagrama ser invertido | 0–1 | UDP |
| [[ui:ns.reorder]] | `reorder` | A chance de um datagrama ser retido, de modo que os seguintes o ultrapassem | 0–1 | UDP |
| [[ui:ns.reset]] | `reset` | A chance de um trecho de um fluxo redefinir a conexão em vez de passar — os dois lados recebem uma redefinição | 0–1 | TCP |
| [[ui:ns.stall]] | `stall` | A chance de um trecho deixar a conexão semiaberta: nada mais passa em nenhum sentido, e nenhum dos lados é avisado | 0–1 | TCP |

Mais sobre retransmissores, predefinições e o que eles modelam na
página [[[ui:nav.netsim]]](../tools/impairment.md).

### Alterar degradação {#node-impairment_change}

Passa um dos nós [[ui:exp.node.impairment]] da execução para outro perfil a partir desta etapa,
sem soltar a porta dele. A fase até aqui é encerrada e contada no relatório.

| Campo | No arquivo | O quê | Padrão | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.relay]] | `relay` | O id de um nó [[ui:exp.node.impairment]] deste experimento | o primeiro | Não |
| [[ui:ns.preset]] e os valores abaixo dela | `profile` | Com o que ele degrada de agora em diante — veja [O perfil](#impair-profile); o retransmissor lê os valores do protocolo dele | [[ui:ns.preset.offline]] (sem degradação se ausente) | Não |

A etapa falha se o retransmissor não está em execução — falhou ao retransmitir, por exemplo.
Saídas: [[ui:exp.outputPort]]. Sem configurações.

```json
{ "id": "cut", "type": "impairment_change", "x": 730, "y": 200, "relay": "relay",
  "profile": { "name": "offline", "offline": true } }
```

### Derrubar/reativar emulador {#node-emulator_state}

Derruba um dos emuladores da execução, ou o traz de volta ao ar. Enquanto está fora do ar, um
emulador HTTP responde como [[ui:exp.downFault]] diz; um dispositivo TCP e um broker
MQTT derrubam as conexões e recusam as novas; dispositivos OSC e UDP não respondem
nada. De volta ao ar, o emulador segue o próprio cronograma de queda, se tiver um.

| Campo | No arquivo | O quê | Padrão |
| --- | --- | --- | --- |
| [[ui:exp.emulatorNode]] | `emulator` | O id de um nó [[ui:exp.node.emulator]] deste experimento | o primeiro |
| [[ui:exp.emulatorDownState]] | `down` | [[ui:exp.emulatorGoesDown]] (`true`) ou [[ui:exp.emulatorComesUp]] (`false`) | fora do ar (`false` se ausente) |
| [[ui:exp.downFault]] | `fault` | Só HTTP: [[ui:emu.outageFault.unavailable]] (`unavailable`), [[ui:emu.outageFault.reset]] (`reset`: a conexão é fechada sem resposta) ou [[ui:emu.outageFault.timeout]] (`timeout`: a requisição é retida até o cliente desistir, 120 s no máximo) | `unavailable` (também se ausente) |

Saídas: [[ui:exp.outputPort]]. Sem configurações. Nada tem modelo.

```json
{ "id": "down", "type": "emulator_state", "x": 500, "y": 200, "emulator": "api", "down": true, "fault": "unavailable" }
```

## Dados {#data}

### Extrair valor {#node-extract}

Guarda uma parte da última resposta HTTP no caminho dele como uma variável, para campos
posteriores (`{{token}}`), verificações e desvios. Uma requisição HTTP precisa vir antes dele em
todos os caminhos. Clicar em um valor em uma resposta de [[ui:exp.sendNow]] adiciona um para você.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.variable]] | `variable` | O nome: letras, dígitos e `_`, sem começar por dígito, não uma palavra reservada, não o nome de um parâmetro | `token` | Não |
| [[ui:exp.extractFrom]] | `from` | [[ui:exp.from.json]] (`json`), [[ui:exp.from.header]] (`header`), [[ui:exp.from.status]] (`status`), [[ui:exp.from.body]] (`body`) ou [[ui:exp.from.regex]] (`regex`) | `json` | Não |
| [[ui:exp.jsonPath]], [[ui:exp.headerName]] ou [[ui:exp.pattern]] | `expr` | Um caminho JSON (`$.data.token`, `$.items[0]`, `$["first name"]`), um nome de cabeçalho (qualquer caixa) ou uma expressão regular — o primeiro grupo dela, ou a correspondência inteira | `$.token`; não usado para status e corpo | Não |

A etapa falha quando não há nada a tomar: o corpo não é JSON, o caminho ou o
cabeçalho não existe, a expressão não corresponde ou — para um campo JSON ou o
corpo inteiro — o corpo era maior que os 256 KiB guardados. Um status é guardado como
número; o resto como texto, ou como o valor JSON encontrado. Saídas:
[[ui:exp.outputPort]]. Sem configurações.

```json
{ "id": "token", "type": "extract", "x": 500, "y": 80, "variable": "token", "from": "json", "expr": "$.data.token" }
```

Mais sobre variáveis em [Dados e modelos](data.md).

## Verificações {#checks}

Uma verificação é aprovada, ou reprova a execução. As quatro verificações de resposta leem a última
resposta HTTP no caminho delas, então uma requisição HTTP — não uma sob carga — precisa vir
antes delas em todos os caminhos.

### Status HTTP {#node-assert_status}

Aprova quando o status da última resposta é exatamente o dado.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.expectedStatus]] | `status` | | 200; 100–599 | Não |

Saídas: [[ui:exp.outputPort]]. Sem configurações.

```json
{ "id": "ok", "type": "assert_status", "x": 500, "y": 80, "status": 200 }
```

### Texto da resposta {#node-assert_body}

Aprova quando o corpo da última resposta contém o texto, exatamente (maiúsculas
e minúsculas incluídas). Só os primeiros 256 KiB de um corpo são guardados: um texto
não encontrado em um corpo cortado reprova com esse motivo.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.contains]] | `contains` | | `ok`; obrigatório | Sim |

Saídas: [[ui:exp.outputPort]]. Sem configurações.

```json
{ "id": "ready", "type": "assert_body", "x": 500, "y": 80, "contains": "ready" }
```

### Cabeçalho da resposta {#node-assert_header}

Aprova quando a última resposta tem o cabeçalho e o valor dele contém o texto.
O nome do cabeçalho é comparado em qualquer caixa; o valor, exatamente.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.headerName]] | `name` | | `content-type`; obrigatório | Sim |
| [[ui:exp.contains]] | `contains` | O que o valor dele deve conter; vazio: o cabeçalho só precisa estar presente | `application/json` | Sim |

Saídas: [[ui:exp.outputPort]]. Sem configurações.

```json
{ "id": "json", "type": "assert_header", "x": 500, "y": 80, "name": "Content-Type", "contains": "json" }
```

### Tempo de resposta {#node-assert_latency}

Aprova quando a última resposta levou no máximo este tempo, do envio da requisição
até o fim do corpo dela.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.maxLatency]] | `max_ms` | | 1.000; 1–120.000 | Não |

Saídas: [[ui:exp.outputPort]]. Sem configurações.

```json
{ "id": "fast", "type": "assert_latency", "x": 500, "y": 80, "max_ms": 250 }
```

### Verificar valor {#node-assert_value}

Compara um valor — em geral uma variável, escrita como modelo — com um valor esperado
e aprova quando a comparação vale.

| Campo | No arquivo | O quê | Padrão | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.value]] | `value` | O que é comparado: `{{token}}`, `{{reply.args[0]}}` | `{{token}}` | Sim |
| [[ui:exp.operator]] | `op` | Veja [Comparações](#comparisons) | [[ui:exp.op.not_empty]] | Não |
| [[ui:exp.expected]] | `expected` | Não usado por [[ui:exp.op.empty]] e [[ui:exp.op.not_empty]] | vazio (também se ausente) | Sim |

Saídas: [[ui:exp.outputPort]]. Sem configurações.

```json
{ "id": "state", "type": "assert_value", "x": 730, "y": 80, "value": "{{state}}", "op": "eq", "expected": "ready" }
```

### Comparações {#comparisons}

[[ui:exp.node.assert_value]], [[ui:exp.node.branch_value]], a condição de saída
de um [[ui:exp.node.loop]], as regras de argumento OSC e as condições HTTP comparam do
mesmo modo:

| Opção | No arquivo | Vale quando o valor |
| --- | --- | --- |
| [[ui:exp.op.eq]] | `eq` | é igual ao esperado — como números quando ambos são números (`200` = `200.0`), senão como texto exato |
| [[ui:exp.op.ne]] | `ne` | não é igual a ele, pela mesma regra |
| [[ui:exp.op.lt]], [[ui:exp.op.le]], [[ui:exp.op.gt]], [[ui:exp.op.ge]] | `lt`, `le`, `gt`, `ge` | é menor, no máximo, maior, no mínimo — ambos precisam ser números: caso contrário uma verificação, um desvio ou um [[ui:exp.node.loop]] reprova a etapa, e uma regra de argumento ou uma condição HTTP não vale |
| [[ui:exp.op.contains]] | `contains` | contém o texto esperado |
| [[ui:exp.op.matches]] | `matches` | corresponde à expressão regular esperada |
| [[ui:exp.op.empty]], [[ui:exp.op.not_empty]] | `empty`, `not_empty` | está vazio (espaços contam como vazio) / não está |

## Fluxo {#flow}

Nós que decidem para onde a execução vai. Mais sobre ramos, uniões e laços em
[Fluxo](flow.md).

### Início {#node-start}

Onde a execução começa; todo experimento tem exatamente um. Ele não tem entrada nem
campos. A primeira linha da linha do tempo dá a semente da execução.

Saídas: [[ui:exp.outputPort]], obrigatória. Vários fios saindo dela iniciam ramos
paralelos de uma vez.

```json
{ "id": "start", "type": "start", "x": 40, "y": 80 }
```

### Fim {#node-end}

Onde a execução se completa; todo experimento tem exatamente um, e ele não tem saídas.
Vários ramos podem chegar a ele: a execução é aprovada uma vez, depois que o último ramo
terminou, e só se nenhum falhou. Uma execução que nunca chega a [[ui:exp.node.end]] falha.

```json
{ "id": "end", "type": "end", "x": 960, "y": 80 }
```

### Atraso {#node-delay}

Espera um tempo fixo antes da etapa seguinte.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.delayMs]] | `ms` | | 300; 0–60.000 | Não |

Saídas: [[ui:exp.outputPort]]. Sem configurações. Para esperas mais longas, ponha vários em sequência
ou em um [[ui:exp.node.loop]].

```json
{ "id": "pause", "type": "delay", "x": 500, "y": 80, "ms": 500 }
```

### Desvio por status {#node-branch_status}

Escolhe [[ui:exp.yes]] quando a última resposta HTTP tem este status, senão
[[ui:exp.no]]. Uma requisição HTTP precisa vir antes dele em todos os caminhos.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.expectedStatus]] | `status` | | 200; 100–599 | Não |

Saídas: [[ui:exp.yes]] e [[ui:exp.no]], ambas obrigatórias. Sem configurações.

```json
{ "id": "branch", "type": "branch_status", "x": 500, "y": 80, "status": 200 }
```

### Desvio por valor {#node-branch_value}

Escolhe [[ui:exp.yes]] quando uma comparação vale, senão [[ui:exp.no]]. Os campos dele
e as [comparações](#comparisons) são os de
[[[ui:exp.node.assert_value]]](#node-assert_value); uma comparação que não pode ser
feita (`lt` em texto) reprova a etapa.

| Campo | No arquivo | O quê | Padrão | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.value]] | `value` | O que é comparado | `{{token}}` | Sim |
| [[ui:exp.operator]] | `op` | | [[ui:exp.op.eq]] | Não |
| [[ui:exp.expected]] | `expected` | | vazio (também se ausente) | Sim |

Saídas: [[ui:exp.yes]] e [[ui:exp.no]], ambas obrigatórias. Sem configurações.

```json
{ "id": "ok", "type": "branch_value", "x": 730, "y": 80, "value": "{{reply.args[0]}}", "op": "eq", "expected": "ok" }
```

### Ramo paralelo {#node-fork}

Executa o que vem depois de [[ui:exp.branch1]] e [[ui:exp.branch2]] ao mesmo tempo,
cada ramo com a própria cópia das variáveis. Cada saída pode ter mais fios
para mais ramos. Sem campos.

Saídas: [[ui:exp.branch1]] e [[ui:exp.branch2]], ambas obrigatórias.

```json
{ "id": "split", "type": "fork", "x": 270, "y": 80 }
```

### Unir ramos {#node-join}

Espera até que todos os fios que chegam a ele tenham sido alcançados e então continua uma vez, com as
variáveis dos ramos mescladas — onde dois ramos definem a mesma variável, vale a do
ramo cujo fio vem depois no arquivo — e a última resposta HTTP do
último deles que teve uma. Sem campos.

Só se encontram aqui ramos que executam todos: um [[ui:exp.node.join]] atrás de um
[[ui:exp.node.branch_status]], cujos [[ui:exp.yes]] e [[ui:exp.no]] nunca acontecem os dois,
nunca continua; quando nenhum outro caminho chega a [[ui:exp.node.end]], a execução
falha neste nó, dizendo por quantos ramos ainda esperava.

Saídas: [[ui:exp.outputPort]], obrigatória.

Qualquer outro nó com vários fios chegando a ele executa uma vez a cada chegada.

```json
{ "id": "joined", "type": "join", "x": 730, "y": 80 }
```

### Laço {#node-loop}

Executa as etapas em [[ui:exp.portBody]] — que voltam a ele — de novo e de novo:
no máximo um número de vezes e, quando há uma condição de saída, até que ela
valha.

| Campo | No arquivo | O quê | Padrão e limites | Modelos |
| --- | --- | --- | --- | --- |
| [[ui:exp.loopMax]] | `max` | No máximo este número de iterações | 5; 1–1.000 | Não |
| [[ui:exp.loopUntilOn]] | `until` | Condição de saída opcional `{ "value", "op", "expected" }`, como em [[[ui:exp.node.assert_value]]](#node-assert_value) | desligada | Valor e esperado: sim |

- O corpo sempre executa pelo menos uma vez. A condição de saída é lida depois de cada
  iteração, de modo que o corpo pode definir o que ela testa.
- [[ui:exp.portDone]] é seguida quando a condição vale — ou, sem
  condição, depois da última iteração.
- [[ui:exp.portLimit]] é seguida quando as iterações acabaram antes de a condição
  valer. Sem um fio nela, isso reprova a etapa.
- Dentro do corpo, `{{counter}}` é o número da iteração.
- Um corpo executa como um ramo: cada saída dentro dele tem um fio; ele não contém
  [[ui:exp.node.start]], [[ui:exp.node.end]], [[ui:exp.node.fork]],
  [[ui:exp.node.join]] nem outro [[ui:exp.node.loop]]; só se entra nele
  por [[ui:exp.portBody]]; e todo fio dentro dele segue adiante no corpo ou
  volta ao laço.

Saídas: [[ui:exp.portBody]] e [[ui:exp.portDone]] (obrigatórias),
[[ui:exp.portLimit]] (opcional). Sem configurações.

```json
{ "id": "poll", "type": "loop", "x": 270, "y": 80, "max": 10,
  "until": { "value": "{{status.args[0]}}", "op": "eq", "expected": "ready" } }
```

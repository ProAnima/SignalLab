---
title: Eventos
description: O WebSocket de eventos do motor em um servidor do Signal Lab, e cada canal com a carga útil dele e quando é enviado.
---

# Eventos

Tudo o que acontece enquanto uma tarefa executa — as etapas de uma execução, as
mensagens de um monitor, os números de uma rajada, os quadros do Inspetor, o fim de uma tarefa — é enviado como
um evento. Em um navegador, a página do servidor os recebe em um único WebSocket,
`/api/events`; um script pode escutar no mesmo socket. O aplicativo de desktop recebe os
mesmos eventos, com os mesmos nomes e cargas úteis, dentro do app.

## Assinatura {#subscribe}

Abra um WebSocket para `/api/events` no servidor:

```bash
websocat -H "Authorization: Bearer $TOKEN" ws://127.0.0.1:1430/api/events
```

- **A autenticação** é a do resto da API: o token como
  `Authorization: Bearer`, ou o cookie de sessão de um navegador. Sem ela, o
  upgrade é recusado com `401` `auth.required`.
- **Origin**: um cliente que envia um cabeçalho `Origin` precisa enviar o do próprio
  servidor (host e porta iguais a `Host`), ou o upgrade é recusado com `403`
  `auth.origin`. A maioria das bibliotecas de WebSocket fora de um navegador não envia nenhum.
- **Todo evento para todo cliente.** Não há nada a assinar: cada
  socket recebe todo evento de toda tarefa, não importa quem a iniciou. Escolha o que precisa
  por `event`, e por `job_id` na carga útil.
- **Só escuta.** O servidor ignora o que um cliente envia, exceto um fechamento; uma
  mensagem maior que 64 KiB fecha o socket.
- **Keep-alive.** O servidor faz ping a cada 20 s, de modo que um socket quieto continua aberto
  através de proxies. Quando o servidor para, ele fecha todos os sockets.
- **Nada é reenviado.** Os eventos enviados enquanto um cliente não estava conectado
  se perdem para ele. Um cliente que se reconecta deve ler o estado atual com
  comandos (`jobs_list`, `inspect_snapshot`, `emulator_exchanges`…).
- **Ficar para trás.** Até 4096 eventos esperam por um socket. Um cliente que
  fica mais para trás recebe [`server://lagged`](#event-server-lagged) com quantos
  perdeu. Um cliente que para de ler de vez — nada que ele receba em 10 s — é desconectado;
  reconecte e leia o estado de novo.

## Formato da mensagem {#format}

Cada evento é uma mensagem de texto com um objeto JSON:

```json
{ "event": "scan://open", "payload": { "job_id": 9, "ts": 1759600000123, "port": 8080, "banner": null } }
```

| Campo | O que é |
| --- | --- |
| `event` | O canal, abaixo |
| `payload` | Os valores do evento; a forma dele depende do canal |

Os tempos (`ts`, `first_ms` e `last_ms` de um par) são milissegundos desde 1970;
as latências e outras durações (`*_latency_ms`, `p50_ms`…, `ms`) são
milissegundos. Os erros nas cargas úteis são objetos
[`EngineError`](index.md#errors); os códigos deles estão listados em
[mensagens de erro](../reference/errors.md).

## Canais {#channels}

| Canal | Enviado por | Quando |
| --- | --- | --- |
| [`experiment://step`](#event-experiment-step) | Uma execução | Uma etapa começa, é aprovada, falha, tenta de novo, repete ou informa carga |
| [`experiment://ended`](#event-experiment-ended) | Uma execução | Uma vez, quando a execução termina sozinha |
| [`job://ended`](#event-job-ended) | Toda tarefa | Uma vez, quando a tarefa termina sozinha ou falha |
| [`osc://message`](#event-osc-message) | Monitor OSC | Todo pacote, amostrado a 200/s sob uma enxurrada |
| [`osc://monitor-stat`](#event-osc-monitor-stat) | Monitor OSC | A cada 250 ms, e no fim |
| [`osc://gen-tick`](#event-osc-gen-tick) | Gerador OSC | Toda mensagem, ou 30 a 45 vezes por segundo acima de 60 mensagens por segundo |
| [`http://burst-progress`](#event-http-burst-progress) | Rajada HTTP | A cada 100 ms, e no fim |
| [`ws://state`](#event-ws-state) | Conexão WebSocket | Conectada, fechada |
| [`ws://messages`](#event-ws-messages) | Conexão WebSocket | A cada 100 ms com algo novo |
| [`mqtt://state`](#event-mqtt-state) | Conexão MQTT | Conectada, assinada, fechada |
| [`mqtt://messages`](#event-mqtt-messages) | Conexão MQTT | A cada 100 ms com algo novo |
| [`mqtt://ack`](#event-mqtt-ack) | Conexão MQTT | Uma publicação QoS 1/2 foi concluída; um cancelamento de assinatura foi respondido |
| [`broadcast://emit-stat`](#event-broadcast-emit-stat) | Beacon | A cada 250 ms, e no fim |
| [`broadcast://peers`](#event-broadcast-peers) | Ouvinte de descoberta | A cada 400 ms |
| [`netsim://stat`](#event-netsim-stat) | Retransmissor de degradação | A cada 250 ms |
| [`storm://stat`](#event-storm-stat) | Tempestade | A cada 250 ms, e no fim |
| [`scan://open`](#event-scan-open) | Scanner | Toda porta aberta |
| [`scan://progress`](#event-scan-progress) | Scanner | Cerca de a cada 1 % do intervalo, e no fim |
| [`emulator://activity`](#event-emulator-activity) | Tarefa de emulador | A cada 200 ms com algo novo |
| [`inspect://batch`](#event-inspect-batch) | Inspetor | A cada 120 ms com quadros novos, cerca de uma vez por segundo quando quieto, enquanto a captura está armada |
| [`server://lagged`](#event-server-lagged) | O servidor | Um cliente ficou para trás |

### `experiment://step` {#event-experiment-step}

Uma etapa de uma execução: um nó começando, sendo aprovado, falhando, esperando para tentar de novo,
repetindo ou informando o progresso de uma carga. Uma execução iniciada com `/api/run` envia
as mesmas etapas na resposta dela (veja [execuções](run.md)).

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa da execução |
| `ts` | número | Quando |
| `node_id` | string | O nó |
| `state` | string | `running`, `passed`, `failed`, `retry` (uma tentativa falhou e a etapa executa de novo depois de uma pausa), `repeating` (o progresso de uma ação que se repete, no máximo uma vez por segundo) ou `load` (o progresso de uma carga, no máximo uma vez por segundo) |
| `detail` | string | O que aconteceu, em inglês; vazio para `running` e `failed` (veja `error`) |
| `message_key` | string ou null | O texto da interface para isso, como uma chave do dicionário dela |
| `message_params` | objeto ou null | Os valores que `message_key` nomeia |
| `vars` | objeto | Variáveis que a etapa escreveu; omitido quando nenhuma |
| `error` | `EngineError` | Por que falhou, ou por que a tentativa falhou (`retry`); omitido nos demais casos |
| `frame` | número | O quadro do Inspetor da mensagem com que uma espera (ou a resposta esperada de um envio) correspondeu, quando a captura estava armada; omitido nos demais casos |
| `load` | objeto | O que uma carga mediu, com os limiares lidos — no último evento de uma etapa de carga, aprovada ou com falha; omitido nos demais casos. Veja [carga](../experiments/load.md) |
| `omitted` | número | Etapas logo antes desta que foram omitidas; omitido quando nenhuma. Uma execução informa as primeiras 5.000 etapas uma a uma, depois uma por segundo (e toda falha) e, ao terminar, as últimas 500 — de modo que um Laço de mil iterações continua legível |

O nó [[ui:exp.node.end]] de uma execução mostra `running` quando o primeiro ramo chega a ele e
`passed` quando todos os ramos terminaram sem falha. Os valores de segredos são
mascarados em todos os campos.

### `experiment://ended` {#event-experiment-ended}

Uma execução terminou sozinha: foi aprovada, falhou ou ficou sem tempo. Enviado logo depois
da mesma carga útil em [`job://ended`](#event-job-ended). Uma execução parada com
`job_stop` ou [[ui:app.stopAll]] não envia nenhum dos dois e não salva relatório.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa da execução |
| `kind` | string | `experiment` |
| `seed` | número | A semente com que executou |
| `profile` | string ou null | O perfil dela |
| `overridden` | booleano | Alguns valores de parâmetros vieram de [[ui:exp.runWith]] ou de `overrides` |
| `error` | `EngineError` ou null | A primeira falha da execução; null quando foi aprovada |
| `report_path` | string ou null | O relatório dela, em `runs/` da pasta de dados |
| `report_error` | `EngineError` ou null | Por que o relatório não pôde ser escrito |

### `job://ended` {#event-job-ended}

Uma tarefa terminou sozinha ou falhou. Uma tarefa parada com `job_stop` ou
`jobs_stop_all` não o envia.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa |
| `kind` | string | `osc-monitor`, `osc-gen`, `http-burst`, `netsim`, `storm`, `scan`, `beacon`, `discovery`, `mqtt`, `websocket`, `emulator` ou `experiment` |
| `error` | `EngineError` ou null | Por que terminou, quando algo deu errado |

O `job://ended` de uma execução carrega também os campos de
[`experiment://ended`](#event-experiment-ended). O que encerra cada tipo:

| `kind` | Termina quando | `error` |
| --- | --- | --- |
| `osc-monitor` | O socket não pode mais receber | `wait.receive_failed` |
| `osc-gen` | A duração dele acabou, ou um envio falha | null, ou `transport.*` |
| `http-burst` | O total ou a duração dele é alcançado | null |
| `storm` | A duração dele acabou | null |
| `scan` | Todas as portas do intervalo foram tentadas | null |
| `beacon` | As rodadas ou a duração dele acabaram, ou mais de 32 envios falharam sem nenhum enviado | null, ou `transport.*` |
| `discovery` | O socket não pode mais receber | `wait.receive_failed` |
| `mqtt` | O broker fechou a conexão ou ela se perdeu | `transport.*` (`transport.reset` quando o broker a fechou), ou `mqtt.protocol` |
| `websocket` | A conexão fechou | null, ou por que se perdeu |
| `netsim` | O retransmissor não pode mais funcionar | por quê |
| `emulator` | O socket dele falha | por quê |
| `experiment` | A execução termina | a falha da execução, ou null |

### `osc://message` {#event-osc-message}

Um pacote UDP que um monitor OSC recebeu, decodificado. Os pacotes decodificados são amostrados a
no máximo 200 eventos por segundo, de modo que uma enxurrada não sobrecarregue o enlace com a interface
([`osc://monitor-stat`](#event-osc-monitor-stat) diz quantos foram recebidos e
quantos não foram encaminhados); um pacote malformado é sempre enviado.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa do monitor |
| `ts` | número | Quando chegou |
| `from` | string | O remetente, `IP:port` |
| `bytes` | número | O tamanho do pacote |
| `messages` | objeto[] | Cada mensagem do pacote (um bundle tem várias): `address` e `args` ([`OscArg`](commands.md#type-oscarg)`[]`) |
| `error` | `EngineError` ou null | `osc.packet_malformed` quando o pacote não decodificou (então `messages` fica vazio) |

### `osc://monitor-stat` {#event-osc-monitor-stat}

Os totais acumulados de um monitor OSC, a cada 250 ms e mais uma vez quando ele termina, de modo que uma
lista amostrada mostre quanto tráfego ela representa.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa do monitor |
| `ts` | número | Quando |
| `packets` | número | Pacotes recebidos, decodificados ou não |
| `bytes` | número | Bytes recebidos |
| `not_shown` | número | Pacotes recebidos mas não encaminhados como [`osc://message`](#event-osc-message) (a amostra de taxa os reteve) |
| `malformed` | número | Pacotes que não decodificaram como OSC |

### `osc://gen-tick` {#event-osc-gen-tick}

O progresso de um gerador OSC: para cada mensagem abaixo de 60 mensagens por segundo;
acima disso, para cada n-ésima, sendo n a taxa dividida por 30 e arredondada para baixo —
30 a 45 vezes por segundo.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa do gerador |
| `ts` | número | Quando |
| `value` | número | O valor recém-enviado, antes de ser arredondado para um inteiro ou para um float de 32 bits |
| `sent` | número | Mensagens enviadas até agora |

### `http://burst-progress` {#event-http-burst-progress}

Os números de uma rajada HTTP, a cada 100 ms enquanto ela executa, e mais uma vez com
`done: true` quando termina sozinha.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa da rajada |
| `ts` | número | Quando |
| `sent` | número | Requisições respondidas ou falhas até agora |
| `ok` | número | Dessas, respondidas com um status 2xx e o corpo inteiro |
| `failed` | número | Dessas, qualquer outro status, um corpo cortado ou nenhuma resposta — nunca uma que esta máquina não pôde enviar |
| `local` | número | Dessas, requisições que esta máquina não pôde enviar: ficou sem sockets ou portas locais (`transport.exhausted`). Não entram em `failed` nem nas latências |
| `missed` | número | Requisições de uma rajada com ritmo que esperaram demais por um worker livre e foram puladas |
| `rps` | número | Requisições por segundo nos últimos 100 ms; no último evento, na rajada inteira |
| `last_latency_ms`, `min_latency_ms`, `max_latency_ms`, `avg_latency_ms` | número | Latências até agora |
| `p50_ms`, `p90_ms`, `p95_ms`, `p99_ms` | número | Percentis de todas as requisições até agora, falhas incluídas, com precisão de 0,5 % |
| `done` | booleano | O último evento da rajada |

### `ws://state` {#event-ws-state}

Uma conexão WebSocket aberta por `ws_connect` conectou ou fechou. Uma
conexão cuja tarefa foi parada não envia `closed`.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa da conexão |
| `ts` | número | Quando |
| `state` | string | `connected` ou `closed` |
| `handshake` | objeto | `url`, `peer`, `local`, `protocol` (o subprotocolo que o servidor escolheu, ou null) e `ms` (a conexão e o upgrade) |
| `closed` | objeto ou null | Com `closed`: `code`, `reason`, `by` (`client`, `server` ou `lost`) e `error` |

### `ws://messages` {#event-ws-messages}

O que uma conexão WebSocket enviou e recebeu desde o último evento, a cada
100 ms quando há algo.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa da conexão |
| `ts` | número | Quando |
| `messages` | objeto[] | Em ordem: `ts`, `dir` (`rx` recebida, `tx` enviada), `kind` (`text` ou `binary`), `text` (os primeiros 64 KiB como UTF-8; de uma mensagem binária, os primeiros 4096 bytes; bytes que não são UTF-8 viram `�`), `hex` (os primeiros 4096 bytes de uma mensagem binária em hex, senão null), `bytes` (o tamanho completo) e `truncated` (mais do que foi mostrado: além de 64 KiB de texto, além de 4096 bytes de binário) |
| `dropped` | número | Mensagens deixadas de fora deste evento porque havia mais de 2000, ou mais de 1 MiB de `text` e `hex`; as mais antigas saem primeiro |
| `received`, `sent` | número | Mensagens recebidas e enviadas na conexão até agora, mostradas ou não |

### `mqtt://state` {#event-mqtt-state}

O estado de uma conexão MQTT mudou.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa da conexão |
| `ts` | número | Quando |
| `state` | string | `connected`; `subscribed` depois de cada resposta a uma assinatura; `closed` quando a conexão terminou (não quando a tarefa dela foi parada) |
| `broker` | string | `host:port` |
| `error` | `EngineError` ou null | Por que uma conexão `closed` terminou (`transport.reset` quando o broker a fechou); null nos demais casos |
| `grants` | objeto[] | Com `subscribed`: cada filtro pedido, com `filter`, `qos` (concedido) e `accepted`; vazio nos demais casos |

### `mqtt://messages` {#event-mqtt-messages}

O que uma conexão MQTT recebeu desde o último evento, a cada 100 ms quando há
algo. Uma mensagem QoS 2 reenviada é mostrada uma vez.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa da conexão |
| `ts` | número | Quando |
| `messages` | objeto[] | Uma por tópico, a mais recente que chegou nele: `ts`, `topic`, `payload` (os primeiros 4096 bytes como UTF-8; bytes que não são viram `�`), `bytes` (o tamanho da carga útil inteira), `truncated` (`payload` é só o começo), `qos`, `retain`, `dup` e `count` (mensagens no tópico desde o último evento, esta incluída) |
| `dropped` | número | Sempre 0: quando um evento carregaria mais de 4000 tópicos ou 2 MiB de carga útil, a conexão para de ler até ele sair, e o broker espera |

### `mqtt://ack` {#event-mqtt-ack}

O broker concluiu algo que a conexão pediu.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa da conexão |
| `ts` | número | Quando |
| `kind` | string | `published` (uma publicação QoS 1 ou 2 está completa) ou `unsubscribed` |
| `packet_id` | número | O id de pacote MQTT |
| `topic` | string ou null | O tópico publicado; null para `unsubscribed` |

### `broadcast://emit-stat` {#event-broadcast-emit-stat}

Os contadores de um beacon, a cada 250 ms, e mais uma vez quando ele termina sozinho com
`pps` 0.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa do beacon |
| `ts` | número | Quando |
| `targets` | número | Destinos em cada rodada |
| `rounds` | número | Rodadas enviadas |
| `packets`, `bytes` | número | Datagramas e bytes enviados |
| `errors` | número | Envios que falharam |
| `local` | número | Envios que esta máquina não pôde fazer (sem buffers ou sockets), fora de `errors`; omitido enquanto 0 |
| `pps` | número | Datagramas por segundo nos últimos 250 ms |

### `broadcast://peers` {#event-broadcast-peers}

O que um ouvinte de descoberta ouviu, a cada 400 ms.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa do ouvinte |
| `ts` | número | Quando |
| `peers` | objeto[] | O ouvido mais recentemente primeiro: `addr`, `proto`, `packets`, `bytes`, `first_ms`, `last_ms`, `last_summary`, `responded` (os pacotes dele que foram respondidos, contados conforme chegaram); no máximo 512 |
| `packets`, `bytes` | número | Tudo o que foi recebido |
| `responses` | número | Respostas enviadas |
| `unlisted` | número | Pares distintos além dos 512 que a tabela comporta: contados aqui, não em `peers` |
| `replies_skipped` | número | Sondagens não respondidas porque já havia respostas automáticas demais (1024) em andamento — o teto que impede uma enxurrada de sondagens de ser amplificada sem limite |

### `netsim://stat` {#event-netsim-stat}

Os contadores de um retransmissor de degradação, a cada 250 ms. Os retransmissores dos nós
[[ui:exp.node.impairment]] de uma execução informam no relatório da execução.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa do retransmissor |
| `ts` | número | Quando |
| `received`, `forwarded` | número | Datagramas ou trechos que entraram e saíram |
| `dropped` | número | Perdidos por `loss`, rajadas ou `offline` (UDP; um retransmissor TCP retém um fluxo enquanto está sem rede e não descarta nada) |
| `throttled` | número | UDP: descartados pelo limite de largura de banda, ou porque já havia demais a caminho. TCP: trechos que seguraram o fluxo por causa do limite de largura de banda |
| `duplicated`, `corrupted`, `reordered` | número | O que o perfil fez com eles |
| `bytes` | número | Bytes repassados |
| `connections`, `reset`, `stalled` | número | TCP: conexões aceitas, redefinidas, deixadas semiabertas; omitido enquanto 0 |
| `refused` | número | TCP: conexões que o destino recusou ou não respondeu, repassadas ao cliente como uma redefinição; omitido enquanto 0 |
| `failed` | número | O que esta máquina não pôde repassar: um datagrama que o sistema não quis enviar, uma conexão para a qual não havia porta local nem socket (tentada de novo por cerca de dois segundos antes), ou uma além das 1.024 conexões simultâneas do retransmissor; omitido enquanto 0 |
| `profile` | string | O perfil com que ele degrada agora, como a linha do tempo o nomeia: o nome dele, ou o que ele faz (`60 ms ±25 · loss 2%`) |

### `storm://stat` {#event-storm-stat}

Os contadores de uma tempestade, a cada 250 ms, e mais uma vez quando ela termina sozinha com
`pps` e `mbps` 0.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa da tempestade |
| `ts` | número | Quando |
| `packets`, `bytes` | número | Datagramas (ou conexões TCP) e bytes enviados |
| `errors` | número | Envios ou conexões que o destino recusou, redefiniu ou não respondeu |
| `local` | número | Unidades que esta máquina não pôde enviar — ficou sem portas locais, sockets ou buffers, e não o destino recusando; nunca contadas como `errors` |
| `pps` | número | Por segundo nos últimos 250 ms |
| `mbps` | número | Megabits por segundo nos últimos 250 ms |

### `scan://open` {#event-scan-open}

O scanner encontrou uma porta aberta.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa da varredura |
| `ts` | número | Quando |
| `port` | número | A porta |
| `banner` | string ou null | O que o serviço enviou primeiro, quando banners foram pedidos e ele disse algo em 400 ms |

### `scan://progress` {#event-scan-progress}

Até onde uma varredura foi: cerca de a cada 1 % do intervalo, e quando ela termina sozinha
com `done` igual a `total` (esse pode chegar duas vezes).

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa da varredura |
| `ts` | número | Quando |
| `done` | número | Portas tentadas |
| `total` | número | Portas no intervalo |
| `open` | número | Portas abertas encontradas |
| `unchecked` | número | Portas que esta máquina não pôde tentar (ficou sem portas locais ou sockets), portanto nem abertas nem fechadas; 0 normalmente |

### `emulator://activity` {#event-emulator-activity}

O que um emulador iniciado com `emulator_start` recebeu e respondeu desde o
último evento, a cada 200 ms quando algo mudou (uma troca, ser derrubado
ou reativado, ou uma mensagem que o broker MQTT não pôde entregar). Os nós [[ui:exp.node.emulator]] de uma execução não o enviam; os contadores deles
estão no relatório da execução.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa do emulador |
| `ts` | número | Quando |
| `counts` | objeto | `total`, `unmatched`, `failed`, `down`, `hits` (por regra), `missed` (MQTT; omitido enquanto 0) e `refused` (conexões além do limite; omitido enquanto 0) — como em [`emulator_exchanges`](commands.md#emulator_exchanges) |
| `forced` | string | `unavailable`, `reset` ou `timeout` enquanto está derrubado; omitido nos demais casos |
| `exchanges` | objeto[] | As novas trocas, como `emulator_exchanges` as lista, mas sem `data`; no máximo 200 |
| `dropped` | número | Trocas além das primeiras 200 do intervalo, não enviadas aqui; `emulator_exchanges` ainda tem as últimas 500 |

### `inspect://batch` {#event-inspect-batch}

Novos quadros do Inspetor. Enviado apenas enquanto a captura está armada: a cada 120 ms quando
há quadros novos, e cerca de uma vez por segundo quando não há, de modo que os
contadores continuem atualizados.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `frames` | objeto[] | Os [quadros](commands.md#type-frame) novos, do mais antigo ao mais novo, no máximo 250; sem os bytes deles (use `inspect_payload`) |
| `stats` | objeto | Os contadores da captura, [`CaptureStats`](commands.md#type-frame) |
| `skipped_now` | número | Quadros capturados desde o último lote mas que não estão neste — chegaram mais de 250, ou o buffer os liberou. Eles ainda entram em uma exportação enquanto o buffer os guarda |

### `server://lagged` {#event-server-lagged}

Só no servidor. Este cliente ficou mais de 4096 eventos para trás e perdeu alguns.
Leia o estado de novo com comandos.

| Campo | Tipo | Significado |
| --- | --- | --- |
| `skipped` | número | Quantos eventos ele perdeu |

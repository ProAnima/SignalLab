---
title: Degradação
description: Ponha um retransmissor entre um cliente e o destino dele que atrasa, descarta, duplica, reordena ou limita datagramas UDP, ou atrasa, limita, redefine ou trava fluxos TCP.
---

# Degradação

A tela [[ui:nav.netsim]] executa um retransmissor que fica entre um cliente e o destino com
que ele conversa e degrada o que passa por ele nos dois sentidos: um Wi-Fi ruim, um enlace
móvel, um salto de satélite, um enlace que cai. Use-o para ver como o seu cliente e o seu
dispositivo se comportam em uma rede que você não tem à mão.

Você aponta o cliente para o retransmissor em vez do destino real; o retransmissor repassa
tudo ao destino, e as respostas de volta ao cliente, depois de fazer com cada datagrama ou
fluxo o que o perfil dele diz. Você pode mudar o perfil enquanto ele roda, sem soltar a
porta.

## Iniciar um retransmissor {#start}

1. Escolha o [[ui:ns.protocol]]: UDP para datagramas (OSC, a maior parte do tráfego de
   controle de show e de sensores), TCP para fluxos (HTTP, MQTT, um dispositivo TCP).
2. Defina [[ui:ns.listen]]: o `IP:port` em que o retransmissor escuta. `0.0.0.0:9010` recebe
   tráfego da rede; `127.0.0.1:9010`, só deste computador.
3. Defina [[ui:ns.target]]: o `IP:port` do destino real, ou um nome de host e uma porta como
   `device.local:9000`.
4. Escolha uma [predefinição](#presets) ou defina os valores do [perfil](#profile).
5. Pressione [[ui:ns.startRelay]].
6. Aponte o seu cliente para a porta do retransmissor em vez do destino — por exemplo
   `127.0.0.1:9010` em vez de `127.0.0.1:9000`.

[[ui:ns.stopRelay]] fecha o retransmissor; nada que ele ainda segurava sai depois disso.

Padrões: UDP, escuta em `0.0.0.0:9010`, destino `127.0.0.1:9000`, latência de 40 ms, jitter
de 15 ms e perda de 2 %.

[[ui:ns.listen]] é um endereço a vincular, sempre `IP:port`. Um nome de host em
[[ui:ns.target]] é resolvido uma vez, quando você pressiona [[ui:ns.startRelay]], como é o
destino [OSC](../protocols/osc.md); um nome que não é encontrado é recusado e nenhum
retransmissor inicia. [[ui:ns.protocol]], [[ui:ns.listen]] e [[ui:ns.target]] ficam fixos
enquanto o retransmissor roda; pare-o para alterá-los.

```text
client ──► relay (0.0.0.0:9010) ──► target (127.0.0.1:9000)
client ◄── relay ◄───────────────── target
```

### UDP {#udp}

O retransmissor envia cada datagrama ao destino a partir de uma porta própria e devolve ao
cliente o que o destino responde. Cada datagrama tem o seu próprio destino, decidido quando
ele chega.

As respostas vão para o cliente que enviou o datagrama mais recente: o retransmissor atende
um cliente por vez.

### TCP {#tcp}

Cada conexão que um cliente faz com o retransmissor é ligada a uma nova conexão do próprio
retransmissor com o destino. Os dois fluxos de cada conexão — do cliente para o destino e do
destino para o cliente — são degradados bloco a bloco, conforme o retransmissor os lê (até
16 KiB por vez). Um fluxo sempre chega em ordem, seja qual for o jitter.

Quando o destino recusa a conexão, a conexão do cliente é redefinida.

## O perfil {#profile}

Os valores que um retransmissor aplica, nos dois sentidos. Um retransmissor UDP lê os valores
do datagrama, um TCP os do fluxo; os outros ficam ocultos.

### Em UDP {#profile-udp}

| Valor | O que faz | Faixa |
| --- | --- | --- |
| [[ui:ns.offline]] | Todo datagrama é descartado, nos dois sentidos, até você desmarcar. | ligado / desligado |
| [[ui:ns.latency]] | Somada a todo datagrama. | 0–1000 ms |
| [[ui:ns.jitter]] | Um atraso extra aleatório, de 0 até este valor, para cada datagrama — de modo que os datagramas podem chegar fora de ordem. | 0–500 ms |
| [[ui:ns.loss]] | A fração de datagramas que nunca chega, cada um por conta própria. | 0–100 % |
| [[ui:ns.burst]] | A chance de um datagrama iniciar uma rajada de perdas: um enlace que cai por um momento, ao contrário de perdas dispersas. | 0–20 % |
| [[ui:ns.burstLength]] | Quantos datagramas uma rajada perde em média. Aparece quando [[ui:ns.burst]] está acima de 0; 5 quando você o aumenta pela primeira vez. | 1–1000 |
| [[ui:ns.duplicate]] | A fração de datagramas entregues duas vezes. | 0–100 % |
| [[ui:ns.corrupt]] | A fração de datagramas com um bit de um byte invertido. | 0–100 % |
| [[ui:ns.reorder]] | A fração de datagramas retidos — pela latência, e por pelo menos 20 ms — para que os seguintes cheguem primeiro. | 0–100 % |
| [[ui:ns.rate]] | Um limite de largura de banda, em quilobits por segundo; 0 é nenhum. Os datagramas entram na fila do enlace a essa taxa; um que esperaria mais de um segundo é descartado como limitado. | 0, ou 8–10.000.000 |

Uma rajada funciona assim: um datagrama que não está em uma rajada inicia uma com a chance de
[[ui:ns.burst]]; todo datagrama de uma rajada é perdido, e cada um a encerra com chance de 1
em [[ui:ns.burstLength]], então uma rajada dura em média esse número de datagramas.

Um datagrama é decidido nesta ordem: sem rede, rajada, perda, largura de banda, depois
duplicação, corrupção, atraso e reordenação para cada cópia.

### Em TCP {#profile-tcp}

| Valor | O que faz | Faixa |
| --- | --- | --- |
| [[ui:ns.offline]] | Nada flui, em nenhum sentido, e as novas conexões esperam o destino, até você desmarcar — então tudo continua. | ligado / desligado |
| [[ui:ns.latency]] | Somada a todo bloco de um fluxo, nos dois sentidos. | 0–1000 ms |
| [[ui:ns.jitter]] | Um atraso extra aleatório, de 0 até este valor, para cada bloco — nunca à frente do bloco anterior. | 0–500 ms |
| [[ui:ns.reset]] | A fração de blocos que redefinem a conexão em vez de passar: tanto o cliente quanto o destino recebem uma redefinição. | 0–100 % |
| [[ui:ns.stall]] | A fração de blocos que deixam a conexão semiaberta: nada mais passa, em nenhum sentido, e nenhum dos lados é avisado. O retransmissor mantém os dois lados abertos, intocados, até parar. | 0–100 % |
| [[ui:ns.rate]] | Um limite de largura de banda para cada fluxo de cada conexão, em quilobits por segundo; 0 é nenhum. Passado um segundo de fila, o retransmissor para de ler, de modo que o remetente desacelera, como em um enlace lento. Nada é descartado. | 0, ou 8–10.000.000 |

Perda, rajadas, duplicação, corrupção e reordenação não se aplicam ao TCP: um fluxo TCP real
retransmite o que perde e se põe em ordem, então o que um cliente encontra em um enlace ruim é
atraso, um remetente lento, redefinições e conexões que param de responder.

::: tip
Os controles deslizantes param em 1000 ms de latência e 500 ms de jitter. O próprio
retransmissor aceita até 60.000 ms de cada, por exemplo a partir de um arquivo de experimento.
:::

## Predefinições {#presets}

Uma predefinição ajusta todos os valores com um clique. O chip dela continua aceso enquanto os
valores ainda são os da predefinição; altere qualquer valor e ele se apaga.

| Predefinição | Em UDP | Em TCP |
| --- | --- | --- |
| [[ui:ns.preset.lan]] | 1 ms ±1 | 1 ms ±1 |
| [[ui:ns.preset.wifi]] | 20 ms ±30, perda de 1 %, rajadas de 1 % × 3, 0,5 % duplicados, 2 % reordenados | 20 ms ±30 |
| [[ui:ns.preset.4g]] | 60 ms ±25, perda de 0,5 %, 0,5 % reordenados, 20.000 kbit/s | 60 ms ±25, 20.000 kbit/s |
| [[ui:ns.preset.satellite]] | 300 ms ±30, perda de 1 %, 2000 kbit/s | 300 ms ±30, 2000 kbit/s |
| [[ui:ns.preset.intermittent]] | 30 ms ±20, rajadas de 3 % × 15 | 30 ms ±20, 0,2 % dos blocos deixados semiabertos |
| [[ui:ns.preset.offline]] | Nada passa | Nada flui |

## Mudar enquanto roda {#live}

Altere qualquer valor, ou escolha outra predefinição, enquanto o retransmissor roda: a mudança
vale um quarto de segundo depois da sua última alteração, sem soltar a porta nem as conexões.
O console registra cada novo perfil, e a linha sob [[ui:ns.live]] diz o que o retransmissor
faz agora — o nome da predefinição, ou os valores em resumo, como
`60 ms ±25 · loss 2% · 20000 kbps`.

Um datagrama ou bloco é decidido pelo perfil em vigor quando ele chega; um que já está a
caminho mantém o seu atraso.

## Os contadores {#counters}

[[ui:ns.live]] mostra o que o retransmissor fez desde que iniciou, os dois sentidos juntos,
atualizado quatro vezes por segundo.

Em UDP:

| Contador | O que é |
| --- | --- |
| [[ui:ns.received]] | Datagramas que chegaram ao retransmissor. |
| [[ui:ns.forwarded]] | Datagramas repassados; um duplicado conta duas vezes. |
| [[ui:ns.dropped]] | Perdidos de propósito: sem rede, uma rajada ou perda. |
| [[ui:ns.throttled]] | Descartados pelo limite de largura de banda, ou porque já havia 10.000 — ou 64 MB — a caminho. |
| [[ui:ns.duplicated]] | Datagramas enviados duas vezes. |
| [[ui:ns.corrupted]] | Cópias com um bit invertido. |
| [[ui:ns.reordered]] | Cópias retidas para que as seguintes as ultrapassassem. |
| [[ui:common.volume]] | Bytes repassados. |
| [[ui:ns.notRelayed]] | Aparece quando há algum: datagramas que o sistema não quis enviar. Nunca é obra do perfil. |

Em TCP:

| Contador | O que é |
| --- | --- |
| [[ui:ns.connections]] | Conexões que os clientes fizeram com o retransmissor. |
| [[ui:ns.received]] | Blocos que o retransmissor leu, nos dois sentidos. |
| [[ui:ns.forwarded]] | Blocos que ele escreveu adiante. |
| [[ui:ns.refused]] | Conexões que o destino recusou ou não respondeu; o cliente recebeu uma redefinição. |
| [[ui:ns.resets]] | Conexões redefinidas. |
| [[ui:ns.stalled]] | Conexões deixadas semiabertas. |
| [[ui:ns.held]] | Quantas vezes um fluxo esperou pelo limite de largura de banda: o remetente foi desacelerado, nada foi descartado. Um retransmissor também segura no máximo 64 MB a caminho, todas as conexões somadas, e espera do mesmo modo além disso. |
| [[ui:common.volume]] | Bytes escritos adiante. |
| [[ui:ns.notRelayed]] | Aparece quando há algum: conexões que este computador não conseguiu fazer com o destino — sem porta local ou socket restante, ainda depois de dois segundos de tentativas — e conexões além de 1.024 ao mesmo tempo. Cada uma recebeu uma redefinição. Nunca é a resposta do destino. |

Um retransmissor que fecha o seu lado de uma conexão primeiro mantém uma porta local por
alguns minutos depois (o TIME_WAIT do sistema), como qualquer programa. Muitas centenas de
conexões curtas por segundo por um só retransmissor esgotam as portas deste computador; o
retransmissor então espera e tenta de novo, e o que ainda não consegue passar é contado em
[[ui:ns.notRelayed]], nunca passado por recusa do destino.

Com a captura ligada, o [Inspetor](inspector.md) mostra os datagramas e blocos retransmitidos
— no máximo um a cada 25 ms, os dois sentidos juntos — cada um com o seu destino:
`forwarded +43ms`, `· corrupted`, `· reordered`, `· copy 2/2`, `dropped (loss)`,
`dropped (burst)`, `dropped (offline)`, `throttled` e, em TCP, `reset` e `half-open`. → é do
cliente para o destino, ← do destino para o cliente.

Um quadro retransmitido nomeia sockets reais: [[ui:ins.local]] é o endereço em que o
retransmissor escuta e [[ui:bc.peer]] é para onde esse datagrama ou bloco ia — o destino, ou o
cliente a que uma resposta voltou. O trecho encerra o veredito:
`forwarded +43ms · client→target`, `dropped (loss) · target→client`. Assim, um datagrama
retransmitido pode ser guardado com [[ui:sig.fromFrame]] e é apontado para o endereço a que ia;
um bloco TCP é um pedaço de um fluxo e não pode (veja
[Inspetor](inspector.md#save-as-signal)).

## O mesmo tráfego, o mesmo destino {#seed}

Toda decisão — qual datagrama é perdido, atrasado em quanto, corrompido onde — sai da semente
do retransmissor, separadamente para cada sentido e, em TCP, para cada conexão. Com a mesma
semente e o mesmo tráfego, o retransmissor descarta os mesmos datagramas e os atrasa do mesmo
modo.

A tela [[ui:nav.netsim]] usa uma nova semente cada vez que você inicia um retransmissor. Para
repetir uma execução exatamente, use um nó [[ui:exp.node.impairment]] em um experimento: ele
sorteia a partir da semente da execução, que você pode fixar (veja
[Repetir uma execução com falhas](../experiments/faults.md#seed)).

## Em experimentos {#experiments}

Dois nós põem o mesmo retransmissor em um experimento:

- [[ui:exp.node.impairment]] abre um retransmissor antes da primeira etapa e o fecha quando a
  execução termina, seja como for. Os endereços de escuta e de destino dele aceitam apenas
  parâmetros; o destino pode ser um nome de host, resolvido quando a execução começa. Veja
  [Nós](../experiments/nodes.md#node-impairment).
- [[ui:exp.node.impairment_change]] passa um retransmissor da execução para outro perfil a
  partir dessa etapa — limpo, com perdas, sem rede, limpo de novo — e o relatório da execução
  conta o que cada fase fez. Veja [Nós](../experiments/nodes.md#node-impairment_change).

Em um nó OSC ou UDP, [[ui:exp.routeThrough]] põe um [[ui:exp.node.impairment]] na frente dele
e aponta o nó para o retransmissor. Veja [Falhas](../experiments/faults.md).

## Relacionados {#related}

- [Emuladores](emulators.md) — o destino a pôr atrás do retransmissor.
- [Inspetor](inspector.md) — o destino de cada datagrama.
- [UDP e TCP](../protocols/udp-tcp.md)

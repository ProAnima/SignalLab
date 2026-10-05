---
title: Falhas
description: Retransmissores de degradação e emuladores como nós de uma execução — uma rede ruim e uma dependência que falha, acionadas na hora certa, contadas fase por fase no relatório e repetíveis com a semente.
---

# Falhas como nós

Para ver como um sistema se vira quando a rede se degrada ou uma dependência sai do ar, coloque a
falha no experimento. Um retransmissor ou um emulador abre com a execução, uma etapa o aciona na
hora certa, e o relatório da execução conta o que aconteceu em cada fase. O fim da execução —
aprovada, com falha ou parada — os fecha, então nada fica degradado depois dela.

| Nó | O que faz |
| --- | --- |
| [[ui:exp.node.impairment]] | um retransmissor entre o sistema em teste e o destino dele, degradando o que passa, durante toda a execução |
| [[ui:exp.node.impairment_change]] | passa um retransmissor da execução para outro perfil, desta etapa em diante |
| [[ui:exp.node.emulator]] | uma API, um dispositivo ou um broker interpretado pelo Signal Lab, durante toda a execução |
| [[ui:exp.node.emulator_state]] | tira do ar um emulador da execução, ou o traz de volta |

Os quatro estão no menu de adicionar, em [[ui:exp.group.fault]] e [[ui:exp.group.emulate]]. Os
campos deles estão na [referência dos nós](nodes.md); o retransmissor em si é descrito em
[Degradação](../tools/impairment.md), e os emuladores, em [Emuladores](../tools/emulators.md).

## Degradação {#impairment}

O sistema em teste envia para o retransmissor em vez do destino real; o retransmissor encaminha
para o destino, traz as respostas de volta e degrada os dois sentidos.

| Campo | O quê |
| --- | --- |
| [[ui:exp.relayListen]] | `IP:port` para o qual o sistema em teste envia ou ao qual se conecta, porta diferente de 0 |
| [[ui:exp.relayTarget]] | `IP:port` do destino real |
| [[ui:ns.protocol]] | UDP — cada datagrama tem a sua própria sorte — ou TCP — cada conexão é ligada a uma conexão própria até o destino |
| o perfil | uma [[ui:ns.preset]] ou valores seus |

O que um retransmissor lê do perfil depende do protocolo; os demais valores ficam de fora:

| Protocolo | Degradações |
| --- | --- |
| UDP | latência, jitter, perda de pacotes, perda em rajada, duplicação, corrupção, reordenação, um limite de banda, sem rede |
| TCP | latência e jitter (um fluxo continua em ordem), um limite de banda (o remetente é freado, nada é descartado), conexões redefinidas, conexões deixadas semiabertas, sem rede |

**Aberto antes da primeira etapa.** Todo retransmissor do experimento abre quando a execução
começa, como os sockets das esperas, então os campos [[ui:exp.relayListen]] e
[[ui:exp.relayTarget]] dele aceitam só texto e parâmetros (`node.params_only`) — `{{relay}}` com
um parâmetro `relay`, nunca uma variável. Uma porta que não pode ser aberta interrompe a execução
antes de qualquer tráfego, no nó.

**De passagem no fluxo.** Quando a execução chega ao nó, ele passa imediatamente, e a linha do
tempo diz o que ele degrada e com quê. O retransmissor funciona do início ao fim da execução, seja
qual for a posição do nó no grafo.

**Fechado com a execução.** Seja como for que a execução termine, o retransmissor fecha; as
conexões de um retransmissor TCP fecham com ele. Um retransmissor que parou de retransmitir por
conta própria guarda o motivo: as etapas que o usam falham com ele, e o relatório informa isso.

::: tip Passar pela degradação
Em um nó [[ui:exp.node.osc]] ou [[ui:exp.node.udp]], o botão [[ui:exp.routeThrough]] coloca uma
Degradação na frente dele: o retransmissor escuta em uma porta livre de `127.0.0.1`, encaminha para
o destino do nó com a predefinição [[ui:ns.preset.lan]], e o nó passa a enviar para o
retransmissor.
:::

## Alterar degradação {#change-impairment}

O nó [[ui:exp.node.impairment_change]] indica um dos retransmissores do experimento em
[[ui:exp.relay]] e dá o perfil com que ele degrada daquela etapa em diante. O retransmissor mantém
a porta e as conexões; os novos valores valem a partir do próximo pacote ou bloco. A linha do tempo
mostra o novo perfil.

Cada alteração encerra uma **fase**. O relatório da execução guarda, para cada retransmissor:

- os endereços de escuta e de destino, e o protocolo, quando é TCP;
- as contagens totais: recebidos, encaminhados, descartados, estrangulados, duplicados,
  corrompidos, reordenados, bytes — e, para TCP, as conexões, as redefinidas e as deixadas
  semiabertas;
- cada fase: o nome do perfil, quando ela começou e terminou, em milissegundos a partir do momento
  em que o retransmissor abriu, e as mesmas contagens só daquela fase.

Um pacote é contado na fase que decidiu a sorte dele, mesmo quando a cópia atrasada dele sai
depois da troca. Um retransmissor guarda as suas últimas 1.000 fases; as mais antigas são
contadas, não guardadas.

Um nó [[ui:exp.node.impairment_change]] que não indica nenhum retransmissor do experimento é
recusado (`impair.relay_unknown`).

## Emulador {#emulator}

O nó [[ui:exp.node.emulator]] faz o papel de uma dependência durante toda a execução: uma API
HTTP, um dispositivo OSC, UDP ou TCP, ou um broker MQTT. É o mesmo emulador que a tela
[Emuladores](../tools/emulators.md) executa sozinha: o botão [[ui:emu.edit]] abre as regras dele,
[[ui:emu.toLibrary]] guarda uma cópia na biblioteca, [[ui:emu.fromLibrary]] pega um de lá.

- Ele abre antes da primeira etapa e responde até o fim da execução; uma porta que não pode ser
  aberta interrompe a execução antes de qualquer tráfego. No fluxo, ele passa imediatamente.
- O endereço dele é um `IP:port` literal. Os padrões de correspondência dele aceitam só
  parâmetros; as respostas são modelos lidos com o que chegou (`{{request.…}}`) e os parâmetros da
  execução. Ele não pode ler segredos.
- As escolhas aleatórias dele — uma mistura ponderada de respostas, o jitter dos atrasos, os
  geradores nas respostas — são sorteadas a partir da semente da execução.
- Um emulador HTTP é também o que um nó [[ui:exp.node.wait_http]] no mesmo endereço escuta: ele
  verifica o que o sistema em teste enviou. Sem um emulador ali, o próprio ouvinte da execução
  responde a toda requisição com `204`.
- Um emulador OSC ou UDP compartilha a porta dele com as esperas da execução nessa porta: os dois
  veem cada datagrama.
- Um emulador MQTT é um broker que os nós [[ui:exp.node.mqtt]] e [[ui:exp.node.wait_mqtt]] da
  execução podem usar como qualquer outro.

O relatório da execução guarda, para cada nó de emulador, o nome, o protocolo e o endereço, e as
contagens: requisições ao todo, as que nenhuma regra aceitou, as que falharam, as que o
encontraram fora do ar, as mensagens que um broker não conseguiu entregar a um cliente lento e as
correspondências de cada regra.

## Emulador fora do ar e de volta {#emulator-state}

O nó [[ui:exp.node.emulator_state]] indica um dos emuladores da execução em
[[ui:exp.emulatorNode]]; o [[ui:exp.emulatorDownState]] é [[ui:exp.emulatorGoesDown]] ou
[[ui:exp.emulatorComesUp]]. Enquanto ele está fora do ar:

| Emulador | O que se encontra |
| --- | --- |
| HTTP | o que diz o campo [[ui:exp.downFault]]: [[ui:emu.outageFault.unavailable]] (`503`, corpo `{"error":"unavailable"}`), [[ui:emu.outageFault.reset]] ou [[ui:emu.outageFault.timeout]] — a requisição fica segurada até o cliente desistir, no máximo 120 s |
| Dispositivo TCP, broker MQTT | as conexões caem e as novas são recusadas |
| Dispositivo OSC, UDP | nada é respondido |

O que chega enquanto ele está fora do ar conta como `down`, nunca como uma requisição que nenhuma
regra aceitou. Um nó [[ui:exp.node.wait_http]] continua vendo as requisições. O emulador fica fora do
ar até que uma etapa o traga de volta, diga o que disser o ciclo de quedas dele, e o fim da
execução o fecha de qualquer forma.

Um nó [[ui:exp.node.emulator_state]] que não indica nenhum emulador do experimento é recusado
(`emulator.node_unknown`).

## Quedas programadas {#outage}

Um emulador também pode sair do ar sozinho: nas regras dele, a opção [[ui:emu.outage]] define
[[ui:emu.outageUp]] e [[ui:emu.outageDown]], cada um de 10–3.600.000 ms, e [[ui:emu.outageFault]]
para HTTP. Ele responde durante o primeiro, fica fora do ar durante o segundo, e assim por diante,
contando a partir de quando abriu — em uma execução, antes da primeira etapa. Fora do ar pelo
ciclo, o `503` de um emulador HTTP leva `Retry-After` com os segundos inteiros que faltam para ele
voltar, pelo menos 1; um `503` enquanto uma etapa [[ui:exp.node.emulator_state]] o mantém fora do
ar não leva nenhum, já que ninguém sabe quando isso termina.

Um ciclo não precisa de etapa; uma etapa não precisa de ciclo. Use o ciclo para uma dependência
que oscila, e a etapa para uma queda em um ponto escolhido do fluxo.

## Exemplo: uma queda atrás de um enlace lento {#example}

Um cliente pede um pedido a uma API por meio de um retransmissor. Enquanto ele pede, um segundo
ramo deixa o enlace lento, em [[ui:ns.preset.4g]], tira a API do ar por dois segundos, a traz de
volta e deixa o enlace limpo de novo. O cliente precisa continuar pedindo até receber a resposta.

```text
start → orders → link → split
split ─ branch1 → settle → until_ok ─ done → answered → joined
                           until_ok ─ body → get → status → pause → until_ok
split ─ branch2 → slow → down → outage → up → clean → joined
joined → end
```

Os nomes são os ids dos nós no arquivo abaixo.

1. Acrescente um parâmetro `api` = `http://127.0.0.1:18091` — o retransmissor, não a API.
2. Acrescente um [[ui:exp.node.emulator]]: HTTP, `127.0.0.1:18090`, uma rota `GET /orders/:id`
   que responde `200` com `{"order":"{{request.params.id}}"}`.
3. Depois dele, uma [[ui:exp.node.impairment]]: [[ui:exp.relayListen]] `127.0.0.1:18091`,
   [[ui:exp.relayTarget]] `127.0.0.1:18090`, [[ui:ns.protocol]] TCP, predefinição
   [[ui:ns.preset.lan]].
4. Depois dela, um nó [[ui:exp.node.fork]].
5. Em [[ui:exp.branch1]], o cliente: um [[ui:exp.node.delay]] de 300 ms e depois um
   [[ui:exp.node.loop]] — [[ui:exp.loopMax]] 40, [[ui:exp.loopUntilOn]] `{{status}}`
   [[ui:exp.op.eq]] `200`. O corpo dele: uma [[ui:exp.node.http]] `GET {{api}}/orders/42`, um nó
   [[ui:exp.node.extract]] que leva o [[ui:exp.from.status]] para `status`, um atraso de 250 ms,
   ligado de volta ao Laço. Em [[ui:exp.portDone]], um
   [[ui:exp.node.log]] `Orders API answers again: HTTP {{status}}`.
6. Em [[ui:exp.branch2]], as falhas: um nó [[ui:exp.node.impairment_change]] que passa o
   retransmissor para [[ui:ns.preset.4g]]; um nó [[ui:exp.node.emulator_state]] que deixa a
   Orders API [[ui:exp.emulatorGoesDown]] com [[ui:emu.outageFault.unavailable]]; um atraso de
   2.000 ms; outro [[ui:exp.node.emulator_state]] que a deixa [[ui:exp.emulatorComesUp]] de novo;
   outro [[ui:exp.node.impairment_change]] de volta para [[ui:ns.preset.lan]].
7. Ligue os dois ramos a um nó [[ui:exp.node.join]], e esse nó ao [[ui:exp.node.end]].
8. Execute.

A linha do tempo mostra as requisições do cliente respondidas com `503` pelo enlace lento, a API
voltando e então `200` e o Laço saindo por [[ui:exp.portDone]]. O relatório conta umas cinco
requisições que encontraram a API fora do ar e uma respondida pela rota dela, e as três fases do
retransmissor — [[ui:ns.preset.lan]] por um instante, [[ui:ns.preset.4g]] durante a queda,
[[ui:ns.preset.lan]] de novo —, cada uma com o seu próprio tráfego.

::: details O experimento como arquivo
Salve-o como arquivo `.json` e abra-o com [[ui:exp.importJson]] em [[ui:exp.documents]].

```json
{
  "version": 9,
  "name": "Outage behind a slow link",
  "params": [{ "name": "api", "value": "http://127.0.0.1:18091" }],
  "profiles": [],
  "profile": null,
  "seed": null,
  "nodes": [
    { "id": "start", "type": "start", "x": 40, "y": 270 },
    { "id": "orders", "type": "emulator", "x": 260, "y": 270,
      "emulator": { "name": "Orders API", "bind": "127.0.0.1:18090", "protocol": "http",
        "routes": [{ "method": "GET", "path": "/orders/:id", "when": [], "order": "sequence",
          "responses": [{ "status": 200, "headers": [], "body": "{\"order\":\"{{request.params.id}}\"}", "delay_ms": 0, "jitter_ms": 0, "fault": "none", "weight": 1 }] }],
        "fallback": null } },
    { "id": "link", "type": "impairment", "x": 490, "y": 270, "listen": "127.0.0.1:18091", "target": "127.0.0.1:18090", "protocol": "tcp",
      "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } },
    { "id": "split", "type": "fork", "x": 720, "y": 270 },
    { "id": "settle", "type": "delay", "x": 950, "y": 140, "ms": 300 },
    { "id": "until_ok", "type": "loop", "x": 1180, "y": 140, "max": 40,
      "until": { "value": "{{status}}", "op": "eq", "expected": "200" } },
    { "id": "get", "type": "http", "x": 1410, "y": 20,
      "request": { "method": "GET", "url": "{{api}}/orders/42", "headers": [], "body": null, "timeout_ms": 3000 } },
    { "id": "status", "type": "extract", "x": 1640, "y": 20, "variable": "status", "from": "status", "expr": "" },
    { "id": "pause", "type": "delay", "x": 1870, "y": 20, "ms": 250 },
    { "id": "answered", "type": "log", "x": 1410, "y": 140, "message": "Orders API answers again: HTTP {{status}}" },
    { "id": "slow", "type": "impairment_change", "x": 950, "y": 400, "relay": "link",
      "profile": { "name": "4g", "latency_ms": 60, "jitter_ms": 25, "rate_kbps": 20000 } },
    { "id": "down", "type": "emulator_state", "x": 1180, "y": 400, "emulator": "orders", "down": true, "fault": "unavailable" },
    { "id": "outage", "type": "delay", "x": 1410, "y": 400, "ms": 2000 },
    { "id": "up", "type": "emulator_state", "x": 1640, "y": 400, "emulator": "orders", "down": false, "fault": "unavailable" },
    { "id": "clean", "type": "impairment_change", "x": 1870, "y": 400, "relay": "link",
      "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } },
    { "id": "joined", "type": "join", "x": 2100, "y": 270 },
    { "id": "end", "type": "end", "x": 2330, "y": 270 }
  ],
  "edges": [
    { "from": "start", "to": "orders" },
    { "from": "orders", "to": "link" },
    { "from": "link", "to": "split" },
    { "from": "split", "to": "settle", "port": "branch1" },
    { "from": "split", "to": "slow", "port": "branch2" },
    { "from": "settle", "to": "until_ok" },
    { "from": "until_ok", "to": "get", "port": "body" },
    { "from": "get", "to": "status" },
    { "from": "status", "to": "pause" },
    { "from": "pause", "to": "until_ok" },
    { "from": "until_ok", "to": "answered", "port": "done" },
    { "from": "answered", "to": "joined" },
    { "from": "slow", "to": "down" },
    { "from": "down", "to": "outage" },
    { "from": "outage", "to": "up" },
    { "from": "up", "to": "clean" },
    { "from": "clean", "to": "joined" },
    { "from": "joined", "to": "end" }
  ]
}
```
:::

Dois modelos em [[ui:exp.documents]] fazem o mesmo de outras maneiras:
[[ui:exp.templateFaults]] envia datagramas a um dispositivo emulado por um retransmissor UDP que
passa por limpo, com perdas, sem rede e limpo de novo; [[ui:exp.templateOutage]] tira uma API
emulada do ar por dois segundos enquanto um cliente continua perguntando.

## Portas {#ports}

Os sockets de uma execução — esperas, respostas, emuladores, retransmissores — não podem
compartilhar uma porta do mesmo protocolo; um socket UDP e um TCP podem usar o mesmo número. No
campo [[ui:exp.relayListen]] de um retransmissor, um endereço em `0.0.0.0` entra em conflito com
qualquer endereço na mesma porta.

| Socket | Não pode compartilhar a porta com |
| --- | --- |
| um emulador HTTP, TCP ou MQTT | outro deles (`emulator.bind_taken`) |
| um emulador OSC ou UDP | outro deles (`emulator.bind_taken`) |
| um emulador TCP ou MQTT | um [[ui:exp.node.wait_http]] (`emulator.bind_taken`) |
| o [[ui:exp.relayListen]] de um retransmissor UDP | outro retransmissor UDP, um emulador OSC ou UDP, uma espera ou o socket de uma resposta (`impair.bind_taken`) |
| o [[ui:exp.relayListen]] de um retransmissor TCP | outro retransmissor TCP, um emulador HTTP, TCP ou MQTT, um [[ui:exp.node.wait_http]] (`impair.bind_taken`) |

Compartilhados de propósito: um emulador HTTP e as etapas [[ui:exp.node.wait_http]] no endereço
dele; um emulador OSC ou UDP e as esperas na porta dele; esperas em um mesmo endereço entre si.

Um retransmissor não pode encaminhar para si mesmo, diretamente ou por outros retransmissores: o
tráfego dele daria voltas no loopback (`impair.loop`). Dois retransmissores em sequência na frente
de um dispositivo não são problema.

## Repetir uma execução com falhas {#seed}

Cada decisão que um retransmissor toma — se um pacote é perdido, duplicado, corrompido ou retido,
quanto jitter ele recebe — é sorteada a partir da semente da execução, separadamente para cada
sentido e pacote a pacote. As escolhas aleatórias de um emulador também saem dela. Execute de novo
com a mesma semente e o mesmo tráfego, e os mesmos pacotes têm a mesma sorte: uma falha vista uma
vez pode ser vista de novo.

Para manter a semente, pressione [[ui:exp.pinSeed]] ao lado dela na linha do tempo, ou execute com
ela em [[ui:exp.runWith]]; veja [sementes](runs.md#seeds). O que a semente não consegue fixar é o
tempo: quando o sistema em teste envia e, portanto, em qual fase cai um pacote.

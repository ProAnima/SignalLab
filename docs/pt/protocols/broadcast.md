---
title: Broadcast
description: Envie uma carga útil a uma lista de hosts, a um endereço de broadcast, a um grupo multicast ou a todos os hosts de uma sub-rede, uma vez ou como beacon, e escute quem responde com o ouvinte de descoberta.
---

# Broadcast

A tela [[ui:nav.broadcast]] ([[ui:bc.title]]) envia uma carga útil UDP a muitos destinos de
uma vez e escuta do outro lado para ver quem responde. Use-a para encontrar dispositivos em
uma rede, para verificar se um fluxo multicast chega a um receptor, ou para fazer o papel de
um dispositivo que responde a sondas de descoberta.

- O [[ui:bc.emitter]] envia a uma lista de hosts, a um endereço de broadcast, a um grupo
  multicast ou a todos os hosts de uma sub-rede — uma vez, ou repetidamente como beacon.
- O [[ui:bc.discovery]] escuta em uma porta, entra em grupos multicast, lista todo par que
  fala com ele e pode responder a sondas.

::: danger
Broadcast, multicast e uma varredura alcançam todos os dispositivos do segmento de rede, não
só aquele em que você pensa, e um beacon continua fazendo isso. Verifique primeiro em que
rede você está e envie apenas para redes que são suas ou que você tem autorização para
testar. Os limites abaixo são proteções, não permissão.
:::

## Enviar {#send}

1. Escolha o [[ui:bc.mode]] (abaixo).
2. Informe o destino: o nome do campo muda com o modo. Em uma rede com endereço IPv4,
   [[ui:bc.useSubnet]] o preenche a partir do endereço desta máquina.
3. Escolha a [[ui:bc.payload]] e escreva-a.
4. Pressione [[ui:bc.sendOnce]]: um datagrama vai para cada destino.

### Modos {#modes}

| [[ui:bc.mode]] | Destino | O que acontece | Padrão |
| --- | --- | --- | --- |
| [[ui:bc.mode.list]] | [[ui:bc.targetList]]: entradas `IP:port` ou `host:port` separadas por vírgulas, ponto e vírgulas ou quebras de linha — um espaço não as separa. Um nome é resolvido, e o seu endereço IPv4 é usado quando ele tem um. | Um datagrama para cada um | `127.0.0.1:9000, 127.0.0.1:9001` |
| [[ui:bc.mode.broadcast]] | [[ui:bc.targetAddress]]: `255.255.255.255:port`, ou um endereço terminado em `.255` | Um datagrama que todos os hosts da rede local recebem. Os roteadores não o repassam. | `255.255.255.255:9000` |
| [[ui:bc.mode.multicast]] | [[ui:bc.targetAddress]]: um grupo de `224.0.0.0` a `239.255.255.255`, com uma porta | Um datagrama para o grupo; só os ouvintes que entraram nele o recebem | `239.1.1.1:9000` |
| [[ui:bc.mode.sweep]] | [[ui:bc.targetCidr]], `a.b.c.d/nn`, e uma [[ui:bc.port]] | Um datagrama para cada host utilizável do bloco, em unicast — para dispositivos que ignoram broadcast | `192.168.1.0/24`, porta 9000 |

Em uma varredura, os endereços de rede e de broadcast são pulados (exceto em um `/31` ou
`/32`), e uma base que não é a própria da rede é arredondada para baixo até ela:
`192.0.2.77/30` varre `192.0.2.77` e `192.0.2.78`. Uma varredura alcança no máximo 1024
hosts, então o maior bloco é um `/22` (1022 hosts); um bloco maior é recusado, com o prefixo
a que reduzi-lo.

Broadcast, grupos multicast, varreduras e as entradas em grupos do ouvinte de descoberta são
somente IPv4: o IPv6 não tem broadcast. Uma [[ui:bc.mode.list]] também pode nomear hosts
IPv6 (veja [Opções de socket](#socket-options)).

[[ui:bc.useSubnet]] preenche `x.y.z.10:9000, x.y.z.11:9000` em [[ui:bc.mode.list]],
`x.y.z.255:9000` em [[ui:bc.mode.broadcast]] e `x.y.z.0/24` em [[ui:bc.mode.sweep]], a partir
do endereço `x.y.z.w` desta máquina. Ele presume uma rede `/24`.

### Carga útil {#payload}

| [[ui:bc.payload]] | O que é enviado |
| --- | --- |
| [[ui:bc.payload.osc]] | Uma mensagem OSC: [[ui:common.address]] e [[ui:common.arguments]] tipados, como na tela [OSC](osc.md). Padrão: `/hello/discover` com o texto `who-is-there`. |
| [[ui:bc.payload.text]] | O [[ui:bc.text]] em UTF-8, exatamente como digitado, sem terminador. Padrão: `HELLO-PROBE`. |
| [[ui:bc.payload.hex]] | Os [[ui:bc.hex]] byte a byte — para reproduzir um quadro capturado ou falar um protocolo de descoberta binário. Pares de dígitos hexadecimais; qualquer outra coisa entre eles é ignorada. Padrão: `48 45 4c 4c 4f`. |

### Opções de socket {#socket-options}

[[ui:bc.socketOptions]] abre mais três configurações:

| Opção | O que é | Padrão |
| --- | --- | --- |
| [[ui:bc.bindSource]] | O `IP:port` local de onde os datagramas saem. Fixe-o para escolher a placa de rede, ou uma porta de origem para a qual um dispositivo responde. `0.0.0.0:0`: qualquer uma. | `0.0.0.0:0` |
| [[ui:bc.ttl]] | Quantos roteadores um datagrama pode atravessar, 1–255. Para multicast, é o limite de saltos multicast: 1 o mantém nesta rede. | `1` |
| [[ui:bc.mcastLoop]] | Só multicast: entrega os datagramas do grupo também a esta máquina, para que um ouvinte aqui os ouça | ligado |

Com o [[ui:bc.bindSource]] padrão, os datagramas saem de um socket IPv4, ou de um IPv6
quando todos os destinos são IPv6. Uma lista que mistura os dois é enviada pelo socket IPv4,
e os seus destinos IPv6 falham; envie-os como uma lista à parte, ou fixe [[ui:bc.bindSource]]
em um endereço IPv6.

### O resultado {#result}

[[ui:bc.lastEmit]] mostra o que saiu: [[ui:bc.targets]], [[ui:common.packets]],
[[ui:common.volume]] e [[ui:common.errors]], e os primeiros oito destinos alcançados ("… +n
more" para o resto). Um erro em um destino não impede os outros; a linha do console diz
quantos falharam.

## Repetir como beacon {#beacon}

Um beacon envia a mesma rodada — um datagrama para cada destino — em um cronograma, até você
pará-lo. Dispositivos que escutam um anúncio periódico precisam de um.

1. Configure o modo, o destino e a carga útil como para um envio único.
2. Em [[ui:bc.beacon]], defina [[ui:bc.beaconRate]] e, se ele deve parar sozinho,
   [[ui:bc.beaconRounds]] ou [[ui:bc.beaconSeconds]].
3. Pressione [[ui:bc.startBeacon]]. [[ui:bc.stopBeacon]] — ou parar a sua tarefa na faixa do
   console — o encerra.

| Campo | O que é | Padrão |
| --- | --- | --- |
| [[ui:bc.beaconRate]] | Rodadas por segundo; precisa ser maior que 0 | `2` |
| [[ui:bc.beaconRounds]] | Parar depois deste número de rodadas; 0 — sem limite | `0` |
| [[ui:bc.beaconSeconds]] | Parar depois deste número de segundos; 0 — sem limite | `0` |

A taxa vezes o número de destinos pode ser no máximo 50.000 datagramas por segundo. Uma
varredura de um `/24` (254 hosts) pode, portanto, se repetir cerca de 196 vezes por segundo
no máximo. Enquanto o beacon roda, [[ui:bc.lastEmit]] mostra [[ui:bc.targets]] (os destinos
de cada rodada, a partir do primeiro relatório), os totais, [[ui:bc.rounds]] e [[ui:bc.pps]]
(datagramas por segundo), atualizados quatro vezes por segundo, e [[ui:bc.sendOnce]] fica
indisponível. Um beacon que teve mais de 32 datagramas com falha e nenhum enviado — sem rota,
broadcast não permitido — para sozinho e diz por quê.

## Escutar dispositivos {#discovery}

O [[ui:bc.discovery]] vincula uma porta UDP e registra todo par que envia a ela: o que
responde a uma sonda, ou o que um dispositivo anuncia por conta própria.

1. Defina [[ui:common.bind]], a porta para a qual os dispositivos enviam.
2. Para multicast, liste os grupos em [[ui:bc.joinGroups]].
3. Pressione [[ui:bc.startListen]]. As configurações ficam travadas até você pressionar
   [[ui:bc.stopListen]].

| Campo | O que é | Padrão |
| --- | --- | --- |
| [[ui:common.bind]] | Onde escutar, `IP:port`. `0.0.0.0` escuta em todas as placas de rede. | `0.0.0.0:9000` |
| [[ui:bc.joinGroups]] | Grupos multicast IPv4 em que entrar, separados por vírgula; vazio — só unicast e broadcast | `239.1.1.1` |
| [[ui:bc.interface]] | O endereço IPv4 da placa de rede em que entrar nos grupos; vazio — o sistema escolhe | vazio |
| [[ui:bc.reuse]] | Escutar em uma porta que outro programa também usa (`SO_REUSEADDR`). Só funciona se esse programa também permitir o compartilhamento. | ligado |
| [[ui:bc.respond]] | Responder a sondas, como um dispositivo faria ([abaixo](#auto-reply)) | desligado |

### Pares {#peers}

[[ui:bc.peers]] lista quem enviou algo, o mais recente primeiro, com o número de pares vistos,
os pacotes ouvidos e as respostas enviadas acima:

| Coluna | O que é |
| --- | --- |
| [[ui:bc.peer]] | O `IP:port` do remetente; o ponto dele mostra se foi ouvido nos últimos 3 segundos |
| [[ui:bc.proto]] | `osc` quando o último datagrama dele foi decodificado como OSC, senão `udp` |
| [[ui:common.packets]] | Quantos ele enviou |
| [[ui:bc.age]] | Segundos desde o último datagrama dele |
| [[ui:bc.lastMessage]] | O último datagrama dele: o endereço e os argumentos OSC, ou o início do texto |

A lista é atualizada algumas vezes por segundo e comporta até 512 pares; além disso, os
pacotes ainda são contados, mas pares novos não ganham uma linha.

### Responder a sondas {#auto-reply}

Com [[ui:bc.respond]], o ouvinte faz o papel de um dispositivo: responde a cada datagrama que
recebe, da porta de escuta, ao endereço e à porta do remetente.

| Campo | O que é | Padrão |
| --- | --- | --- |
| [[ui:bc.payload]] | A resposta: OSC, texto ou hex, como no envio | OSC `/hello/here` com o texto `signal-lab` |
| [[ui:bc.replyDelay]] | Aguardar este tempo antes de responder, como faria um dispositivo lento | `0` |
| [[ui:bc.matchContains]] | Responder só a datagramas cujo texto decodificado contenha isto — o endereço e os argumentos OSC, ou o início do texto; vazio — todos | vazio |

O ouvinte nunca responde a um datagrama idêntico à sua própria resposta, então dois ouvintes
apontados um para o outro não ficam respondendo um ao outro para sempre.

### Firewalls e portas compartilhadas {#firewall}

O tráfego de broadcast e multicast vindo de outras máquinas é bloqueado por padrão pela
maioria dos firewalls do Windows: permita o Signal Lab em redes privadas quando o aplicativo
oferecer. O broadcast nunca atravessa um roteador. Para escutar em uma porta que o serviço
real já tem, os dois lados precisam permitir o compartilhamento ([[ui:bc.reuse]] aqui); sem
isso, uma porta ocupada é recusada com uma dica para ligá-lo. Veja
[Solução de problemas](../reference/troubleshooting.md).

## No Inspetor {#inspector}

Com a captura ativada, o tráfego da tela aparece com o protocolo `osc` ou `udp`, conforme a
carga útil:

| Origem | O que é | Quantas |
| --- | --- | --- |
| `broadcast` | [[ui:bc.sendOnce]]: cada datagrama, com o veredito `fan-out`, `broadcast`, `multicast` ou `sweep`; um que falhou, com `error: …` | todas |
| `beacon` | As rodadas de um beacon | no máximo uma rodada a cada 50 ms |
| `discovery` | Os datagramas que o ouvinte recebe | no máximo um a cada 40 ms |
| `discovery` | As respostas dele, com o veredito `auto-reply` | todas |

O que o Inspetor deixa de fora de um beacon ou do ouvinte é contado: o próximo quadro que ele
desenha traz o número no veredito, `+n not shown` (para um beacon, um quadro para cada
destino de cada rodada deixada de fora). Veja [Inspetor](../tools/inspector.md).

## Em um servidor ou no Docker {#server}

Em um [servidor](../server/index.md), a tela envia e escuta na rede do servidor. No Docker,
broadcast, multicast e descoberta só alcançam a rede local quando o contêiner usa a rede do
host (`--network host`) em um host Linux. Com a rede bridge padrão do Docker, ou o Docker
Desktop, só funciona unicast para hosts que o contêiner consegue alcançar.

## Em outros lugares {#elsewhere}

- [`signallab send udp`](../automation/cli.md#cli-send-udp) envia um datagrama a um host; não
  há broadcast, multicast nem varredura pela linha de comando.
- Uma etapa [[ui:exp.node.udp]] envia um datagrama de texto a um ou mais hosts, e uma etapa
  [[ui:exp.node.wait_udp]] aguarda um. Veja [UDP e TCP](udp-tcp.md).
- Para fazer o papel de um dispositivo que responde por regras — várias regras, respostas
  montadas a partir do que chegou — use um emulador [[ui:emu.new.udp]] ou [[ui:emu.new.osc]].
  Veja [Emuladores](../tools/emulators.md).

## Problemas {#troubleshooting}

| O que você vê | Causa habitual |
| --- | --- |
| `… is not a broadcast address` | [[ui:bc.mode.broadcast]] aceita `255.255.255.255:port` ou um endereço terminado em `.255`. Para outra máscara de sub-rede, use [[ui:bc.mode.sweep]]. |
| `… is not a multicast group` | O endereço está fora de `224.0.0.0`–`239.255.255.255`. |
| `… spans … addresses, and a sweep reaches at most 1024 hosts` | O bloco é maior que um `/22`; reduza-o. |
| `Set the port to sweep` | [[ui:bc.port]] é 0. |
| `… is over the … pps limit` | A taxa vezes o número de destinos passa de 50.000 por segundo: reduza [[ui:bc.beaconRate]], ou estreite o destino. |
| `… is already in use — turn on “share the port” to listen alongside it` | Outro programa está com a porta; marque [[ui:bc.reuse]]. |
| `Cannot join the multicast group …` | O grupo ou [[ui:bc.interface]] não é utilizável nesta máquina — nenhuma placa de rede com esse endereço, ou nenhuma rota multicast. |
| Enviado, mas ninguém responde | Os dispositivos escutam em outra porta; o firewall daqui barra as respostas deles; há um roteador no meio; ou, no Docker, o contêiner não está na rede do host. |
| As sondas saem, mas o ouvinte de descoberta não ouve respostas | Muitos dispositivos respondem ao endereço e à porta de onde veio a sonda — o socket do próprio emissor, que a tela não lê. Escute na porta para a qual os dispositivos respondem, ou envie a sonda de um experimento: uma etapa [[ui:exp.node.udp]] com [[ui:exp.expectReply]] envia e escuta na mesma porta. |

Todas as mensagens de erro estão listadas em [Mensagens de erro](../reference/errors.md#broadcast).

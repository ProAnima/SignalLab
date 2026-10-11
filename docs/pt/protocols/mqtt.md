---
title: MQTT
description: Conecte-se a um broker MQTT 3.1.1, observe ao vivo, como uma árvore, todos os tópicos que ele guarda, publique em QoS 0, 1 ou 2, anuncie uma última vontade e limpe valores retidos que ficaram presos.
---

# MQTT

A tela [[ui:nav.mqtt]] é um cliente MQTT para olhar um broker e mudar o que há nele. Ao
conectar, ela assina `#` por padrão: todos os tópicos que o broker guarda formam uma árvore
com o último valor de cada um. A partir daí você publica, limpa um valor retido, salva um
tópico como sinal ou o transforma em uma etapa de experimento.

O Signal Lab fala **MQTT 3.1.1 sobre TCP simples**, com QoS 0, 1 e 2 para assinar, publicar
e para a última vontade. Não há MQTT 5 nem TLS: um broker que só aceita `mqtts://` ou
clientes MQTT 5 não pode ser alcançado.

## Conectar {#connect}

1. Abra [[ui:nav.mqtt]].
2. Informe [[ui:mq.host]] e [[ui:common.port]].
3. Deixe [[ui:mq.clientId]] como está, a menos que o broker espere um em particular.
   Acrescente [[ui:mq.username]] e [[ui:mq.password]] só se o broker pedir.
4. Pressione [[ui:mq.connect]].

Conectar abre a conexão TCP e conclui o handshake MQTT antes de qualquer outra coisa, de modo
que uma senha errada ou uma porta fechada é relatada ali mesmo. Os campos da conexão ficam
travados enquanto conectado; [[ui:mq.disconnect]] a fecha. A conexão é uma tarefa na faixa
do console e também pode ser parada de lá.

| Campo | O que é | Padrão |
| --- | --- | --- |
| [[ui:mq.host]] | O endereço IP ou nome de host do broker | `127.0.0.1` |
| [[ui:common.port]] | A porta do broker | `1883` |
| [[ui:mq.clientId]] | O nome do seu cliente no broker. Não pode ser vazio e precisa ser único ali: um segundo cliente com o mesmo id derruba o primeiro. | `signal-lab-` e seis dígitos hexadecimais aleatórios, novos a cada início do aplicativo |
| [[ui:mq.username]], [[ui:mq.password]] | Enviados só se o broker precisar — em texto simples, já que não há TLS. Uma senha sem nome de usuário não é enviada de jeito nenhum: o MQTT 3.1.1 não consegue levá-la. | vazio |
| [[ui:mq.keepAlive]] | Segundos que a conexão pode ficar em silêncio. O Signal Lab envia um ping ao broker a cada metade desse tempo; um broker derruba um cliente que fica em silêncio por 1,5 vez esse tempo. 0 desliga os pings. | `60` |
| [[ui:mq.cleanSession]] | Ligado: cada conexão começa sem assinaturas guardadas e sem mensagens na fila. Desligado, pede ao broker que as mantenha para este id de cliente entre as conexões. | ligado |
| [[ui:mq.scanFilter]] e o seu [[ui:mq.qos]] | Um filtro assinado assim que a conexão está no ar; `#` é todo tópico. Vazio: nenhum. | `#`, QoS 0 |
| [[ui:mq.willEnable]] | Dá ao broker uma última vontade ([abaixo](#will)) | desligado |

O broker tem 6 segundos para aceitar a conexão TCP e mais 6 para responder ao handshake.

### A última vontade {#will}

Uma última vontade é uma mensagem que o broker guarda para você e publica por conta própria
se a sua conexão morrer sem uma despedida adequada. A presença costuma ser montada assim: um
dispositivo publica `online` no seu tópico de status, e a sua última vontade põe o mesmo
tópico em `off`.

Com [[ui:mq.willEnable]] marcado, defina [[ui:mq.willTopic]] e [[ui:mq.willPayload]] (`off`
por padrão). A última vontade é publicada em QoS 2 e retida. Sem um tópico, nenhuma última
vontade é enviada.

## Assinar {#subscribe}

O filtro de varredura é assinado ao conectar. Para mais:

1. Em [[ui:mq.addSubscription]], digite um filtro de tópico.
2. Escolha o seu [[ui:mq.qos]].
3. Pressione [[ui:mq.subscribe]] ou <kbd>Enter</kbd>.

Um filtro é um tópico com curingas:

| Curinga | Significa | Exemplo |
| --- | --- | --- |
| `+` | exatamente um nível | `sensors/+/state` corresponde a `sensors/door/state` |
| `#` | todos os níveis abaixo, só como último caractere | `sensors/#` corresponde a `sensors/door/state` e a `sensors` |

[[ui:mq.subscriptions]] lista cada filtro com o que o broker concedeu: `qos0`, `qos1` ou
`qos2` — o broker pode conceder menos do que você pediu — ou [[ui:mq.refused]].
[[ui:mq.unsubscribe]] cancela uma.

| QoS | Entrega |
| --- | --- |
| 0 | No máximo uma vez: enviada e esquecida |
| 1 | Pelo menos uma vez: confirmada, pode chegar duas vezes |
| 2 | Exatamente uma vez: um handshake em duas etapas; uma reentrega não é mostrada duas vezes |

## A árvore de tópicos {#topics}

Toda mensagem que chega entra em [[ui:mq.topics]], uma árvore dos níveis dos tópicos. Um
tópico mostra o último valor, um **R** quando esse valor é retido e quantas mensagens ele
teve quando mais de uma. Clique em um nível para abri-lo ou fechá-lo.

- Digite no campo acima da árvore para listar só os tópicos cujo caminho ou último valor
  contém o texto.
- Acima da árvore ficam o número de tópicos, quantos têm um valor retido e, enquanto
  conectado, o broker em que você escuta.
- As cargas úteis são mostradas como texto; bytes que não são UTF-8 aparecem como caracteres
  de substituição. De uma carga útil acima de 4 KB mostra-se o início, com o tamanho total;
  ela não pode ser copiada para [[ui:mq.publish]] nem salva como sinal, pois seria enviada
  cortada.
- [[ui:common.clear]] esvazia a árvore. Nada mais o faz: ela fica como está quando você troca
  de tela ou desconecta, até o aplicativo fechar.

As mensagens chegam à tela em lotes, dez vezes por segundo, cada lote com a última mensagem
de cada tópico que teve alguma e quantas teve: uma enxurrada em poucos tópicos não perde
nada, e toda contagem está certa. Quando um lote enche — 4000 tópicos, ou 2 MB de cargas
úteis, como os valores retidos de um broker ao assinar `#` o fazem —, a conexão só continua a
ler depois que ele saiu, de modo que o broker espera em vez de um tópico enviado uma única
vez ser perdido. A árvore comporta até 100.000 tópicos; mensagens em tópicos além disso são
contadas como "não exibidas" acima dela.

### O painel de um tópico {#topic}

Escolha um tópico com um valor para vê-lo abaixo da árvore: [[ui:mq.value]], [[ui:mq.qos]],
[[ui:mq.retain]], [[ui:common.bytes]], [[ui:mq.messages]] e [[ui:mq.lastAt]]. Os botões dele:

| Botão | O que faz |
| --- | --- |
| [[ui:mq.editHere]] | Copia o tópico, o valor, o QoS e o indicador retain para [[ui:mq.publish]] |
| [[ui:mq.waitForThis]] | Acrescenta ao experimento aberto uma etapa [Aguardar MQTT](../experiments/nodes.md#node-wait_mqtt) neste tópico, neste broker, com qualquer carga útil e tempo limite de 2000 ms |
| [[ui:mq.clearRetained]] | Remove o valor retido ([abaixo](#clear-retained)) |
| [[ui:sig.fromFrame]] | Guarda o tópico e o último valor dele como um sinal na pasta [[ui:sig.capturedFolder]] |

## Publicar {#publish}

1. Conecte.
2. Em [[ui:mq.publish]], informe o [[ui:mq.topic]] e a [[ui:sig.payload]].
3. Escolha o [[ui:mq.qos]] e marque [[ui:mq.retain]] se o broker deve guardar a mensagem
   como o valor do tópico para todo cliente que assinar depois.
4. Pressione [[ui:mq.publishBtn]].

O console confirma cada publicação: na hora em QoS 0, quando o broker a confirmou em QoS 1 e
2. Um tópico para publicar não tem curingas e não é vazio: um tópico com `+` ou `#` é
recusado antes de qualquer envio, com a mesma mensagem que um sinal, uma etapa e
`signallab send mqtt` dão, e a conexão continua como estava. Só uma assinatura aceita filtros
com curingas.

### Limpar um valor retido {#clear-retained}

Um valor retido fica no broker até ser substituído, e todo cliente que assina o recebe
primeiro — um valor obsoleto é um motivo clássico para um dispositivo iniciar no estado
errado. A única forma de removê-lo é publicar uma carga útil vazia com retain definido.

[[ui:mq.clearRetained]], no painel de um tópico, faz isso: pressione-o e depois
[[ui:mq.clearConfirm]]. Ele publica a carga útil retida vazia em QoS 1 na sua conexão. Está
disponível só enquanto conectado e quando o último valor do tópico é retido. Você pode fazer
o mesmo à mão: uma [[ui:sig.payload]] vazia com [[ui:mq.retain]] marcado.

::: warning
Limpar altera o broker para todos os clientes de uma vez.
:::

## No Inspetor {#inspector}

Com a captura ativada, o tráfego MQTT aparece com o protocolo `mqtt`:

| Origem | O que é | Quantas |
| --- | --- | --- |
| `mqtt` | O que a conexão da tela publica; uma publicação retida vazia tem o veredito `clears retained` | todas |
| `mqtt` | As mensagens que a conexão recebe | no máximo uma a cada 200 ms |
| `mqtt-send` | Uma publicação que trouxe a sua própria conexão: um sinal disparado enquanto a tela não está conectada ao broker do sinal, uma etapa, `signallab send mqtt` (veredito `one-shot`) | todas |
| `experiment-wait` | As mensagens que a assinatura de uma etapa [[ui:exp.node.wait_mqtt]] recebe, afora as reproduções de retidos | todas |

O resumo diz `topic = payload`, com o QoS e `retained` quando se aplicam. Veja
[Inspetor](../tools/inspector.md).

## Salvar e reutilizar {#library}

- **Salvar como sinal.** [[ui:sig.saveNew]], sob [[ui:mq.publish]], guarda o broker
  ([[ui:mq.host]] e [[ui:common.port]] da conexão), o tópico, a carga útil, o QoS e o
  indicador retain na biblioteca de sinais; <kbd>Ctrl</kbd>+<kbd>S</kbd> no painel de
  publicação faz o mesmo, e atualiza o sinal quando o painel está ligado a ele. Veja
  [Sinais](../tools/signals.md).
- **Disparar um sinal MQTT.** Enquanto esta tela está conectada ao broker que o sinal nomeia
  (mesmo host, ignorando maiúsculas, e mesma porta; `1883` quando o sinal não dá nenhuma),
  um sinal disparado da biblioteca sai por essa conexão, com o id de cliente e as
  credenciais dela. Caso contrário — sem conexão, ou conectada a outro broker — ele abre uma
  conexão própria com o seu próprio broker — um id de cliente novo, sem nome de usuário —,
  publica, aguarda a confirmação que o seu QoS pede e desconecta. Os nomes não são
  resolvidos, então `localhost` e `127.0.0.1` contam como brokers diferentes. A biblioteca
  não guarda senha.
- **Em um experimento.** Um sinal MQTT salvo pode ser escolhido em [[ui:exp.group.signals]],
  no menu [[ui:exp.addNode]] do experimento, o que o torna uma etapa [[ui:exp.node.mqtt]].

## Em experimentos {#experiments}

| Etapa | O que faz |
| --- | --- |
| [[ui:exp.node.mqtt]] | Conecta, publica uma mensagem e desconecta — sem nome de usuário nem senha, com sessão limpa, em até 15 segundos. [Detalhes](../experiments/nodes.md#node-mqtt) |
| [[ui:exp.node.wait_mqtt]] | Assina quando a execução começa e aguarda uma mensagem em um filtro de tópico cuja carga útil corresponda; valores retidos reproduzidos ao assinar são ignorados. [Detalhes](../experiments/nodes.md#node-wait_mqtt) |
| [[ui:exp.node.emulator]] | Um broker MQTT próprio da execução. [Detalhes](../experiments/nodes.md#node-emulator) |

Nenhuma das duas etapas faz login, então elas precisam de um broker que aceite clientes sem
nome de usuário.

### O emulador de broker {#broker-emulator}

O Signal Lab também pode ser o broker: um emulador [[ui:emu.new.mqtt]] encaminha o que os
clientes publicam a quem assinou — 3.1.1, TCP simples, QoS 0, 1 e 2, mensagens retidas,
últimas vontades, um login opcional — e responde por regras, como um dispositivo. Aponte o
seu equipamento e esta tela para ele para testar sem um broker real. Veja
[Emuladores](../tools/emulators.md).

## Pela linha de comando {#cli}

`signallab send mqtt` publica uma mensagem com uma conexão própria:

```bash
signallab send mqtt 127.0.0.1:1883 lab/light/1/set on --qos 1
signallab send mqtt 127.0.0.1:1883 lab/light/1/state "" --retain
```

```text
✔ lab/light/1/set → 127.0.0.1:1883 · 2 B · qos1
```

A segunda linha limpa um valor retido. Sem porta, o broker está na 1883. O comando não usa
credenciais. Termina com 0 quando o broker aceitou a mensagem, 1 quando não foi possível
alcançá-lo ou ele a recusou. Veja [Linha de comando](../automation/cli.md#cli-send-mqtt).

## Problemas {#troubleshooting}

| O que você vê | Causa habitual |
| --- | --- |
| `… refused the connection — nothing is listening on that port` | Nenhum broker nesse endereço e porta. |
| `… accepted the connection but did not answer in time — is it an MQTT broker?` | Algo escuta ali, mas não fala MQTT, ou fala sobre TLS. |
| `… answered with something other than MQTT 3.1.1` | Não é um broker MQTT, ou é um broker que enviou algo que o Signal Lab não consegue ler. |
| `… does not accept MQTT 3.1.1 clients` | O broker só aceita MQTT 5. |
| `… rejected the client ID — choose another one` | O id é longo demais ou tem caracteres que o broker não aceita. |
| `… rejected the username or password` | Credenciais erradas, ou uma senha sem nome de usuário. |
| `… did not authorize this client — check its access rules` | As regras de acesso do broker recusam este cliente. |
| `… is unavailable right now — try again later` | O broker está no ar, mas não aceita clientes. |
| `Enter a client ID — brokers refuse an empty one` | [[ui:mq.clientId]] está vazio. |
| `A publish topic cannot contain the wildcards + or #` | O tópico de publicação tem um `+` ou `#`. Eles servem para assinar; publique em um tópico por vez. |
| Um filtro mostra [[ui:mq.refused]] | As regras de acesso do broker o proíbem, ou o filtro está malformado (`#` fora da última posição, `+` dividindo um nível com outros caracteres). |
| A conexão cai um instante depois de conectar | Outro cliente conectou com o mesmo [[ui:mq.clientId]]. |
| Nada aparece na árvore | O filtro de varredura está vazio, ou o broker não deixa este cliente ver nada. |

Todas as mensagens de erro estão listadas em [Mensagens de erro](../reference/errors.md#mqtt).

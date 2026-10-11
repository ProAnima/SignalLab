---
title: UDP e TCP
description: Onde o Signal Lab envia e recebe datagramas UDP brutos e dados TCP — etapas de experimento, sinais, a linha de comando, as ferramentas de carga e de varredura e dispositivos emulados — e como as cargas úteis são escritas.
---

# UDP e TCP

UDP e TCP brutos não têm tela própria. São o que você usa para um dispositivo com protocolo
próprio, de texto ou binário — um projetor, um servidor de mídia, um sensor — e aparecem em
vários lugares:

| Para… | Use |
| --- | --- |
| enviar um datagrama ou uma mensagem TCP como etapa e aguardar a resposta | as [etapas de experimento](#experiments) |
| guardar um datagrama para enviar de novo, ou reproduzir um que você capturou | um [sinal UDP](#signals) |
| enviar um datagrama a partir de um script | [`signallab send udp`](#cli) |
| enviar a muitos hosts de uma vez, a um endereço de broadcast ou a um grupo multicast | a tela [Broadcast](broadcast.md) |
| carregar um servidor ou um enlace com tráfego | [Tempestade](#storm) |
| descobrir quais portas TCP um host tem abertas | [Scanner](#scanner) |
| fazer o papel do dispositivo | um emulador de [dispositivo UDP ou TCP](#emulators) |
| piorar a rede entre duas pontas | um [retransmissor de degradação](#impairment) |

OSC é um formato transportado em datagramas UDP; ele tem a sua própria página:
[OSC](osc.md).

## Cargas úteis {#payloads}

Sempre que você escreve uma carga útil bruta, ela é de um de dois tipos:

| Tipo | O que é enviado | Exemplo |
| --- | --- | --- |
| Texto | Os caracteres em UTF-8, exatamente como digitados: sem terminador, sem fim de linha acrescentado. Um protocolo de linhas precisa do fim de linha no texto. | `PING` |
| Bytes em hex | Byte a byte, escritos como pares de dígitos hexadecimais. Espaços, `:`, `-` e `,` entre os pares e um `0x` antes deles são permitidos. | `de ad be ef`, `DEADBEEF`, `0xde,0xad` |

Um datagrama carrega no máximo 65.507 bytes. Um número ímpar de dígitos hexadecimais, ou
nenhum, é um erro antes de qualquer envio.

::: info
O UDP não tem confirmação de entrega. "Enviado" quer dizer que o datagrama saiu desta
máquina, não que alguém o recebeu. Para saber que um dispositivo o ouviu, aguarde a resposta
dele.
:::

## Para onde vai {#destinations}

Um destino é um endereço IP e uma porta, ou um nome de host e uma porta:
`192.0.2.20:9000`, `[2001:db8::20]:9000` (um endereço IPv6 vai entre colchetes),
`projector.local:9000`. Isso vale para o destino de uma etapa UDP, para um destino OSC, para
um sinal UDP, para `signallab send udp` e `signallab send osc`, para a lista do
[Broadcast](broadcast.md), para o destino da [Tempestade](../tools/storm.md) e para o host da
etapa TCP.

Um nome de host é resolvido a cada uso. Quando tem um endereço IPv4, esse é usado — então
`localhost` alcança um serviço que escuta em `127.0.0.1`, mesmo que o primeiro endereço que
um sistema lista para ele seja `::1` — e um nome que só tem endereços IPv6 é alcançado por um
socket IPv6. Um nome que não resolve falha com `Cannot resolve …`; um destino sem porta, ou
que não tem nenhuma das duas formas, com `… is not a valid address`. Os endereços em que um
serviço *escuta* (o de um monitor, de uma espera, de um emulador) são sempre `IP:port`.

## Em experimentos {#experiments}

| Etapa | O que faz |
| --- | --- |
| [[ui:exp.node.udp]] | Envia a sua [[ui:exp.payload]] como texto para [[ui:common.target]]. O destino é `IP:port` ou `host:port` ([acima](#destinations)), e vários destinos separados por vírgulas recebem cada um o datagrama. Com [[ui:exp.expectReply]], envia de [[ui:exp.replyOn]] e aguarda ali uma resposta na mesma etapa. [Detalhes](../experiments/nodes.md#node-udp) |
| [[ui:exp.node.wait_udp]] | Escuta em [[ui:exp.listenOn]] (`IP:port`) e aguarda um datagrama cuja carga útil corresponda. [Detalhes](../experiments/nodes.md#node-wait_udp) |
| [[ui:exp.node.tcp]] | Conecta-se a [[ui:exp.host]] e [[ui:exp.port]], escreve a sua [[ui:exp.payload]] como texto, escuta por 250 ms uma resposta e fecha. A etapa diz quantos bytes voltaram, não quais eram. [Detalhes](../experiments/nodes.md#node-tcp) |

A carga útil e o destino UDP, o host e a carga útil TCP e o padrão de uma espera aceitam
`{{templates}}`, de modo que um datagrama pode levar o id da execução ou um valor que uma
etapa anterior extraiu. Veja [Dados e modelos](../experiments/data.md).

### Casar um datagrama {#matching}

[[ui:exp.node.wait_udp]] e a resposta de [[ui:exp.node.udp]] escolhem um datagrama pela
carga útil:

| [[ui:exp.waitMode]] | Aceita um datagrama quando |
| --- | --- |
| [[ui:exp.mode.any]] | sempre: o primeiro que chegar |
| [[ui:exp.mode.contains]] | a carga útil dele, lida como texto, contém o padrão |
| [[ui:exp.mode.regex]] | a carga útil dele, lida como texto, corresponde à expressão regular |
| [[ui:exp.mode.hex]] | os bytes dele contêm os bytes do padrão, escritos em hex |

O que correspondeu fica guardado na variável da etapa ([[ui:exp.replyVariable]], `reply` a
menos que você a renomeie): `text`, `hex` (os primeiros 1024 bytes), `bytes` (o tamanho),
`from` (o `IP:port` do remetente), `ms` (quanto tempo levou) e `match` (o texto ou os bytes
encontrados, ou o primeiro grupo de uma expressão regular).

Uma espera começa a escutar quando a execução começa, não quando a etapa é alcançada, de modo
que uma resposta muito rápida não se perde. Ela só aceita o que chegou depois do último envio
no seu caminho.

### Limites e padrões {#limits}

| Configuração | Padrão | Faixa |
| --- | --- | --- |
| Etapa TCP: [[ui:common.timeoutMs]] (conectar, escrever e a resposta juntos) | 4000 ms | 1–120.000 ms |
| [[ui:exp.waitTimeout]] de uma espera ou de uma resposta | 2000 ms | 1–120.000 ms |
| Carga útil UDP | — | no máximo 65.507 bytes |
| Endereço de escuta | — | `IP:port` com porta; o [[ui:exp.replyOn]] de uma resposta pode usar a porta 0 (qualquer porta livre) |

No Inspetor, o datagrama de uma etapa UDP aparece com a origem `broadcast` (ou `experiment`
quando a etapa aguarda uma resposta), e todo datagrama que chega à porta de uma espera, com a
origem `experiment-wait`. Uma etapa TCP aparece como dois quadros `tcp` com a origem
`experiment`: a carga útil que ela escreveu e, quando veio uma, a resposta que ela leu. A
entrada dela na linha do tempo diz o que enviou e qual era o tamanho da resposta.

## Sinais {#signals}

Um sinal [[ui:sig.tr.udp]] na biblioteca é um destino e uma carga útil, como texto ou bytes
em hex. Dispare-o em [[ui:nav.signals]], ou com <kbd>Ctrl</kbd>+<kbd>K</kbd> de qualquer
tela. O destino dele pode ser um nome de host.

Qualquer datagrama que o Inspetor guardou inteiro pode virar um: [[ui:sig.fromFrame]] cria
um sinal UDP em hex, na pasta [[ui:sig.capturedFolder]], que reproduz exatamente esses bytes
— para o destino do quadro, no caso de um quadro enviado; para o endereço que o recebeu, no
caso de um que chegou (neste computador, quando era todo endereço). Um bloco TCP não pode: é
um pedaço de um fluxo. Veja [Sinais](../tools/signals.md) e
[Inspetor](../tools/inspector.md#save-as-signal).

Um sinal UDP de texto pode ser acrescentado a um experimento como etapa [[ui:exp.node.udp]];
um em hex não pode, porque a etapa envia texto.

## Pela linha de comando {#cli}

`signallab send udp` envia um datagrama:

```bash
signallab send udp 127.0.0.1:9000 --text "PING"
signallab send udp 127.0.0.1:9000 --hex "de ad be ef"
```

```text
✔ sent 4 bytes → 127.0.0.1:9000
```

Informe exatamente um entre `--text` e `--hex`. O destino é `IP:port` ou `host:port`
([acima](#destinations)). O comando termina com 0 quando o datagrama saiu, 1 quando o envio
falhou e 2 quando o destino ou o hex é inválido. Não existe `send tcp`. Veja
[Linha de comando](../automation/cli.md#cli-send-udp).

## Tempestade {#storm}

[[ui:nav.storm]] é uma fonte de carga para os seus próprios servidores e enlaces: uma
[[ui:st.udp]] envia datagramas de um tamanho definido a uma taxa definida, uma [[ui:st.tcp]]
abre uma conexão, escreve a carga útil e fecha, de novo e de novo. A vazão é medida ao vivo.
Veja [Tempestade](../tools/storm.md).

## Scanner {#scanner}

[[ui:nav.scan]] tenta uma conexão TCP com cada porta de uma faixa e lista as que aceitam,
com o que o serviço diz primeiro se você pedir os banners. Veja
[Scanner](../tools/scanner.md).

::: danger
A Tempestade e o Scanner enviam tráfego real a hosts reais. Aponte-os só para sistemas que
são seus ou que você tem autorização para testar: uma tempestade pode saturar um enlace, e
ambos podem disparar a detecção de intrusão.
:::

## Dispositivos emulados {#emulators}

Na tela [[ui:nav.emulators]], o Signal Lab pode ser o dispositivo:

- um [[ui:emu.new.udp]] responde a datagramas por regras sobre a carga útil — qualquer uma,
  contendo um texto, correspondendo a uma expressão regular, contendo bytes — com uma
  resposta em texto ou hex montada a partir do que chegou, para o remetente ou para outro
  `IP:port`, depois de um atraso se você definir um;
- um [[ui:emu.new.tcp]] aceita conexões, divide o que chega em mensagens em um fim de linha
  à sua escolha (LF, CR LF, CR, ou cada bloco como vem), responde a cada uma pelo mesmo
  tipo de regras, pode enviar uma saudação quando um cliente se conecta e pode fechar a
  conexão depois de uma resposta.

Os dois também podem rodar como uma etapa [[ui:exp.node.emulator]] durante uma execução.
Veja [Emuladores](../tools/emulators.md).

## Degradação {#impairment}

O retransmissor de [[ui:nav.netsim]] fica entre um cliente e o destino dele e degrada o que
passa: por datagrama em UDP (atraso, perda, duplicatas, reordenação, um limite de largura de
banda) ou por fluxo em TCP (atraso, um limite de largura de banda, conexões redefinidas ou
deixadas semiabertas). Veja [Degradação](../tools/impairment.md) e, dentro de um experimento,
[Falhas](../experiments/faults.md).

## "Porta inacessível" no Windows {#port-unreachable}

Quando um datagrama chega a uma porta em que nada escuta, a máquina receptora costuma
responder com uma mensagem ICMP de "porta inacessível". O Windows relata essa resposta na
próxima leitura do socket de envio, como se a conexão tivesse sido redefinida — embora o UDP
não tenha conexão.

O Signal Lab espera por isso. Os seus ouvintes — o monitor OSC, o ouvinte de descoberta, as
esperas e respostas de experimentos, os emuladores UDP e OSC, o retransmissor de degradação —
registram o fato e continuam escutando. Um dispositivo que sumiu não os para. Um ouvinte que
realmente não consegue mais receber encerra a sua tarefa, e o console diz por quê.

## Problemas {#troubleshooting}

| O que você vê | Causa habitual |
| --- | --- |
| `… is not a valid address` | O destino não tem porta, ou não é nem `IP:port` nem `host:port`. |
| `Cannot resolve …` | O nome de host não resolve nesta máquina. |
| `… refused the connection — nothing is listening on that port` (TCP) | Nada escuta nessa porta, ou um firewall a rejeita. |
| `No answer from … in time` (TCP) | O host não responde de jeito nenhum — endereço errado, ou um firewall que descarta em vez de rejeitar. |
| Uma espera expira embora o dispositivo responda | O dispositivo responde à porta de onde o datagrama saiu, não à porta da espera. Deixe o envio aguardar a resposta por conta própria com [[ui:exp.expectReply]]: ele então sai da porta para onde a resposta volta. |
| Datagramas de outras máquinas nunca chegam | No Windows, o firewall pode barrá-los: permita o Signal Lab quando o aplicativo oferecer. Veja [Solução de problemas](../reference/troubleshooting.md). |

Todas as mensagens de erro estão listadas em [Mensagens de erro](../reference/errors.md#transport).

---
title: WebSocket
description: Conecte-se a um serviço ws:// ou wss:// com os cabeçalhos e subprotocolos que ele espera, envie mensagens de texto ou binárias e leia cada mensagem à medida que chega.
---

# WebSocket

A tela [[ui:nav.ws]] é um cliente WebSocket: abre uma conexão com um serviço — com os
cabeçalhos e subprotocolos que o serviço espera —, envia texto ou bytes e lista cada
mensagem que vai e vem, a mais recente por último. Use-a para experimentar uma API ao vivo,
uma superfície de controle ou um dispositivo que fala WebSocket antes de automatizá-lo em um
experimento.

## Conectar {#connect}

1. Abra [[ui:nav.ws]].
2. Em [[ui:field.url]], digite o endereço: `ws://127.0.0.1:9001/` ou
   `wss://example.com/socket`.
3. Se o serviço pedir, informe [[ui:exp.wsProtocols]] e acrescente [[ui:http.headers]] (um
   cabeçalho `Authorization` com um token, um cookie).
4. Pressione [[ui:ws.connect]]. Enquanto o upgrade acontece, o botão diz
   [[ui:ws.connecting]]; quando termina, os campos travam e o botão passa a ser
   [[ui:ws.disconnect]].

| Campo | O que é | Padrão |
| --- | --- | --- |
| [[ui:field.url]] | `ws://` ou `wss://`, um host, uma porta opcional (80 para `ws`, 443 para `wss`) e um caminho | `ws://127.0.0.1:9001/` |
| [[ui:exp.wsProtocols]] | Subprotocolos a oferecer, separados por vírgula, em ordem de preferência; o servidor escolhe um. Um nome não tem espaços, vírgulas nem barras. | nenhum |
| [[ui:http.headers]] | Cabeçalhos extras para a requisição de upgrade; [[ui:http.addHeader]] acrescenta uma linha. Uma linha sem nome é deixada de fora. Um `Host` (ou outro cabeçalho que o próprio upgrade escreve, como `Sec-WebSocket-Version`) substitui o que o upgrade enviaria. | nenhum |

Conectar — a consulta do nome, a conexão TCP, o TLS para `wss://` e o upgrade — tem 10
segundos. A URL é mantida quando você troca de tela e quando reinicia o aplicativo;
cabeçalhos e subprotocolos não.

Quando conectado, o painel mostra:

| Item | O que é |
| --- | --- |
| [[ui:ws.state]] | [[ui:ws.open]], ou, quando termina: fechada por você ou pelo servidor, com o código de fechamento, ou [[ui:ws.lost]] quando a linha caiu sem um fechamento |
| [[ui:field.reason]] | O motivo dado por quem fechou, se houver |
| [[ui:ws.subprotocol]] | O subprotocolo que o servidor escolheu, ou — |
| [[ui:ws.peer]] | O `IP:port` do servidor |
| [[ui:ws.upgradeTime]] | Quanto tempo a conexão e o upgrade levaram, em milissegundos |

A conexão é uma tarefa: aparece na faixa do console e também pode ser parada de lá.

### Conexões seguras {#wss}

`wss://` confia nos mesmos certificados do HTTPS neste sistema: um servidor cujo certificado
este sistema não aceita (autoassinado, expirado, outro nome) é recusado com "Não foi
possível estabelecer uma conexão segura com …". Não há configuração para pular a
verificação.

## Enviar uma mensagem {#send}

1. Em [[ui:ws.message]], escolha [[ui:exp.wsText]] ou [[ui:exp.wsBinary]].
2. Escreva a mensagem. Para binário, escreva os bytes como pares de dígitos hexadecimais:
   `de ad be ef`.
3. Pressione [[ui:common.send]], ou <kbd>Ctrl</kbd>+<kbd>Enter</kbd> na mensagem.

Uma mensagem tem no máximo 16 MiB (16.777.216 bytes, contados como bytes no caso binário, não
como dígitos hexadecimais), o mesmo limite de uma mensagem que chega e da etapa
[[ui:exp.node.ws_send]]. Uma mais longa é recusada com `Too long: at most 16777216` antes de
qualquer envio, e a conexão continua aberta.

Quando uma mensagem de texto é JSON, [[ui:http.formatJson]] a organiza com recuos antes do
envio. A mensagem é mantida quando você troca de tela e quando reinicia o aplicativo.

## Ler as mensagens {#messages}

[[ui:ws.messages]] lista o que foi recebido (↓) e enviado (↑), a mais recente por último, com
a hora, o início da mensagem (300 caracteres) e o tamanho; uma mensagem binária mostra os
bytes em hex e é marcada como [[ui:ws.binary]]. Acima da lista ficam quantas foram recebidas
e enviadas. A lista acompanha as novas mensagens enquanto está rolada até o fim; role para
cima e ela fica onde você está.

Clique em uma mensagem para vê-la inteira abaixo da lista: JSON organizado, binário em hex.
[[ui:ws.editHere]] a copia para o campo de mensagem, para enviar de novo ou alterar. Uma
mensagem muito longa é mostrada em parte — texto até 64 KiB, binário até 4096 bytes — e então
não pode ser copiada, pois seria enviada cortada.

A tela guarda as últimas 2000 mensagens, menos quando são longas (8 milhões de caracteres no
total); [[ui:common.clear]] esvazia a lista. As contagens de recebidas e enviadas acima dela
são as da própria conexão, todas as mensagens incluídas. Se um serviço envia mais rápido do
que a tela consegue receber — mais de 2000 mensagens, ou 1 MB de texto, em um décimo de
segundo —, as mais antigas desse grupo ficam de fora da lista e são contadas como "não
exibidas". O Inspetor ainda as tem enquanto a captura está ativada. Por mais que um serviço
inunde, [[ui:ws.disconnect]] e uma parada continuam valendo na hora.

## Fechar {#close}

[[ui:ws.disconnect]] envia um quadro de fechamento com o código 1000 (normal) e aguarda até 2
segundos pela resposta do servidor antes de desligar. O estado passa então a
fechada com 1000. Quando o servidor fecha, o estado mostra o código e o motivo dele; quando a conexão cai
sem um quadro de fechamento, ele diz [[ui:ws.lost]] e o console explica por quê.

O Signal Lab responde sozinho aos pings do servidor; pings e pongs não são listados. Uma
conexão também termina quando chega uma mensagem acima de 16 MiB, ou quando o envio de uma
leva mais de 10 segundos porque o servidor parou de ler. (Enviar você mesmo uma acima de 16
MiB é recusado e não encerra nada.)

## No Inspetor {#inspector}

Com a captura ativada, o tráfego da conexão aparece com o protocolo `ws` e a origem
`websocket`:

| Resumo | O que é |
| --- | --- |
| `CONNECT ws://… (subprotocol)` | A conexão foi aberta |
| `TEXT …` | Uma mensagem de texto e o início dela |
| `BINARY n B …` | Uma mensagem binária, o tamanho e os primeiros 16 bytes |
| `CLOSE code reason` | Um quadro de fechamento, enviado ou recebido |

Cada quadro de mensagem guarda os seus bytes. Enquanto o tráfego é leve, toda mensagem é
capturada; uma conexão movimentada é limitada a 200 quadros por segundo, e o próximo quadro
capturado diz quantos ficaram de fora (`+n not shown`). Veja [Inspetor](../tools/inspector.md).

## Em experimentos {#experiments}

Quatro etapas automatizam uma conversa WebSocket. Uma conexão é aberta por uma etapa e
nomeada pelas outras:

| Etapa | O que faz |
| --- | --- |
| [[ui:exp.node.ws_connect]] | Abre uma conexão para o resto da execução. A URL e os cabeçalhos aceitam `{{templates}}`, então um token extraído antes pode entrar neles. [Detalhes](../experiments/nodes.md#node-ws_connect) |
| [[ui:exp.node.ws_send]] | Envia uma mensagem de texto ou binária em uma conexão. [Detalhes](../experiments/nodes.md#node-ws_send) |
| [[ui:exp.node.wait_ws]] | Aguarda uma mensagem cuja carga útil corresponda, como faz [[ui:exp.node.wait_udp]]; mensagens JSON podem ser lidas campo a campo depois. [Detalhes](../experiments/nodes.md#node-wait_ws) |
| [[ui:exp.node.ws_close]] | Fecha uma conexão com um handshake de fechamento: código 1000, ou 3000–4999 para os de uso próprio de um aplicativo, e um motivo de até 123 bytes. [Detalhes](../experiments/nodes.md#node-ws_close) |

Uma conexão que a execução ainda mantém aberta quando termina — ou é interrompida — é
fechada como deve ser. O modelo [[ui:exp.templateWsEcho]] é um exemplo prático.

## Pela linha de comando {#cli}

`signallab send ws` faz uma troca: conecta, envia uma mensagem, aguarda a resposta se você
pedir uma e fecha.

```bash
signallab send ws ws://127.0.0.1:9001/ --text '{"type":"ping"}' --expect pong
```

O handshake e o que foi enviado vão para a saída de erro padrão, a resposta para a saída
padrão:

```text
Connected to ws://127.0.0.1:9001/ in 4 ms
Sent 15 bytes
{"type":"pong"}
```

`--hex` envia uma mensagem binária; `-H` acrescenta um cabeçalho e `--protocol` oferece um
subprotocolo (ambos repetíveis). `--expect TEXT`, `--expect-regex RE` ou `--wait` (qualquer
mensagem) dizem que resposta aguardar, por `--timeout` milissegundos (2000 por padrão). O
comando termina com 1 quando a resposta não vem ou a conexão falha. Veja
[Linha de comando](../automation/cli.md#cli-send-ws).

## Problemas {#troubleshooting}

| O que você vê | Causa habitual |
| --- | --- |
| `… is not a WebSocket address` | A URL não começa com `ws://` nem `wss://`, ou não tem host. |
| `… answered HTTP n instead of switching to WebSocket` | O servidor recusou o upgrade: um caminho errado (404), um token ausente ou errado (401, 403). O início do corpo da resposta dele (por `ws://`, o que chega em 0,3 s) está nos detalhes técnicos. |
| `… did not take any of the subprotocols offered` | Você ofereceu subprotocolos e o servidor não escolheu nenhum deles, ou respondeu com um que você não ofereceu. |
| `… is not a subprotocol name` | Um nome com espaço, vírgula ou barra. |
| `The header … cannot be sent with the upgrade` | Um nome ou valor de cabeçalho com caracteres que o HTTP não permite. |
| `… refused the connection` | Nada escuta nessa porta. |
| `A secure connection to … could not be made` | O certificado não é confiável aqui, ou o TLS falhou. Veja [Conexões seguras](#wss). |
| `The connection with … broke: the server did not keep to the WebSocket protocol` | O servidor enviou algo que não é WebSocket válido. |
| `A WebSocket message is limited to … bytes` | O servidor enviou uma mensagem acima de 16 MiB, o que encerra a conexão. |
| `Too long: at most 16777216` | A mensagem que você tentou enviar passa de 16 MiB. Nada foi enviado; a conexão está aberta. |

Todas as mensagens de erro estão listadas em [Mensagens de erro](../reference/errors.md#ws).

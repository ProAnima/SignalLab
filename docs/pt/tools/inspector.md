---
title: Inspetor
description: Capture todos os quadros que as ferramentas do Signal Lab enviam e recebem, filtre-os e leia-os decodificados e como bytes, exporte-os e guarde um como sinal.
---

# Inspetor

O Inspetor é uma única linha do tempo para todas as ferramentas: cada mensagem OSC,
datagrama, troca HTTP, publicação MQTT, mensagem WebSocket, pacote retransmitido e troca de
emulador cai ali, decodificado, com os bytes de que era feito. Use-o para ver o que realmente
passou pelo fio, em que ordem, e o que aconteceu com isso.

Ele fica no painel inferior, como a aba [[ui:dock.inspector]] ao lado do console, e por isso
está presente em todas as telas. Clique na aba para abri-lo; o painel abre alto o bastante
para algumas linhas e o detalhe de um quadro. O botão ⤢ ([[ui:dock.maximise]]) deixa o painel
tão alto quanto a janela. O Inspetor mantém a lista e a seleção enquanto você fecha o painel
ou troca de tela.

## Capturar {#capture}

A captura fica desligada quando o Signal Lab inicia e, enquanto está desligada, não custa
nada: as ferramentas nem sequer montam quadros.

1. Abra a aba [[ui:dock.inspector]].
2. Pressione [[ui:ins.arm]]. O ponto na aba fica vermelho e pulsa.
3. Use qualquer ferramenta. Os quadros aparecem no topo da lista, o mais recente primeiro.
4. Pressione [[ui:ins.disarm]] quando tiver o que precisa.

A aba mostra quantos quadros foram capturados, de qualquer tela.

Os quadros são capturados a partir do momento em que você ativa a captura, nunca antes:
ative-a primeiro, depois envie.

Em um [servidor](../server/index.md), a captura pertence ao servidor: toda página conectada a
ele vê os mesmos quadros, e ativá-la ou limpá-la em uma página o faz para todas.

### O que é capturado {#sources}

| Ferramenta | Quadros | Quantos |
| --- | --- | --- |
| [OSC](../protocols/osc.md): envio | Cada mensagem enviada | Todas |
| [OSC](../protocols/osc.md): monitor | Cada pacote recebido; um que não decodifica é marcado com o erro de decodificação | Todos |
| [OSC](../protocols/osc.md): gerador de sinal | Mensagens enviadas | No máximo uma a cada 100 ms, marcada `sampled` |
| [Broadcast](../protocols/broadcast.md): enviar uma vez | Cada datagrama, um por destino; um envio que falhou, com o erro | Todos |
| [Broadcast](../protocols/broadcast.md): beacon | Datagramas enviados | No máximo um a cada 50 ms |
| [Broadcast](../protocols/broadcast.md): ouvinte de descoberta | Sondas recebidas | No máximo uma a cada 40 ms |
| [Broadcast](../protocols/broadcast.md): ouvinte de descoberta | As respostas dele (`auto-reply`) | Todas |
| [HTTP](../protocols/http.md): envio, sinais, requisições de experimento | Cada troca: a linha da requisição, o status e o tempo, os cabeçalhos da resposta e o início do corpo | Todas |
| [HTTP](../protocols/http.md): rajada de carga e requisições sob carga | Trocas | No máximo uma a cada 100 ms |
| [MQTT](../protocols/mqtt.md): conexão | Publicações enviadas | Todas |
| [MQTT](../protocols/mqtt.md): conexão | Mensagens recebidas | No máximo uma a cada 200 ms |
| [MQTT](../protocols/mqtt.md): um sinal MQTT enviado enquanto a tela não está conectada ao broker dele | A publicação | Todas |
| [WebSocket](../protocols/websocket.md) | Mensagens enviadas e recebidas | Todas enquanto o tráfego é leve; no máximo 200 por segundo |
| [Emuladores](emulators.md) | O que chega e a resposta, juntos | No máximo uma troca a cada 10 ms |
| [Degradação](impairment.md) | Cada datagrama ou bloco retransmitido, nos dois sentidos, com o seu destino | No máximo um a cada 25 ms para os dois sentidos juntos |
| [Tempestade](storm.md) (UDP) | Pacotes da inundação, todos idênticos | Um por segundo, marcado `sampled 1/s`; uma tempestade TCP não captura nenhum |
| [Scanner](scanner.md) | Cada porta aberta, com o banner dela | Todas |
| [Experimentos](../experiments/index.md) | O que as etapas de uma execução enviam (uma mensagem TCP: a carga útil escrita e a resposta lida) e o que as esperas dela recebem | Como a ferramenta que usa |

Uma ferramenta que faz amostragem deixa o resto de fora de propósito, e o conta: o próximo
quadro que ela desenha diz quantos segurou, no veredito, como `+n not shown`
(`sampled · +5 not shown`). A contagem é do que a ferramenta teria desenhado, não do descarte
da própria captura (veja [Contagens e lacunas](#counts)).

## A lista de quadros {#list}

| Coluna | O que é |
| --- | --- |
| [[ui:common.time]] | Quando foi capturado, ao milissegundo. |
| [[ui:ins.dir]] | → enviado (`tx`), ← recebido (`rx`). Para um retransmissor, → é do cliente para o destino e ← do destino para o cliente. |
| [[ui:bc.proto]] | `osc`, `udp`, `tcp`, `http`, `mqtt` ou `ws`. |
| [[ui:bc.peer]] | O outro lado: um `IP:port`, uma URL, um broker. |
| [[ui:common.bytes]] | O tamanho do quadro. |
| [[ui:ins.summary]] | Uma linha na notação do próprio protocolo, como `/fader/1 0.75` ou `GET http://127.0.0.1:8080/ → 200 in 3ms`. |
| [[ui:ins.verdict]] | O que aconteceu com ele, quando há algo a dizer. |

O veredito é verde, âmbar ou vermelho. Vermelho é uma perda ou uma falha (`dropped (loss)`,
`failed`, `error: …`); âmbar é um quadro alterado ou apenas uma amostra de muitos
(`corrupted`, `copy 2/2`, `sampled`, `+n not shown`); verde é o resto. Alguns vereditos que
você vai encontrar:

| Veredito | De | Significa |
| --- | --- | --- |
| `forwarded +42ms` | Degradação | Repassado depois desse atraso; `· corrupted`, `· reordered` ou `· copy 1/2` podem vir em seguida. |
| `dropped (loss)`, `dropped (burst)`, `dropped (offline)` | Degradação | Perdido de propósito, e por quê. |
| `throttled` | Degradação | Descartado pelo limite de largura de banda. |
| `· client→target`, `· target→client` | Degradação | Termina o veredito de todo quadro retransmitido, antes de qualquer `+n not shown`: para que lado ele ia. |
| `#2 → 200 OK · 37 B`, `— → 404 …` | Emuladores | Qual regra respondeu (`—`: nenhuma) e a resposta. |
| `down`, `down → 503` | Emuladores | Chegou enquanto o emulador estava fora do ar. |
| `200 OK`, `failed` | HTTP | O status da resposta, ou nenhuma resposta. |
| `open` | Scanner | Uma porta aberta. |
| `auto-reply` | Descoberta | Uma resposta que o ouvinte enviou a uma sonda. |
| `clears retained` | MQTT | Uma publicação retida vazia. |
| `+n not shown` | Qualquer ferramenta que faz amostragem | Esse número de quadros desde o anterior ficou de fora; vem depois do outro veredito do quadro, após um `·`. |

A lista guarda os 4000 quadros mais recentes e desenha os 300 mais recentes que passam pelos
filtros; sob os filtros ela diz quantos mostra de quantos correspondem.

### Filtrar {#filter}

- Digite no campo de texto ([[ui:ins.filterPlaceholder]]) para manter os quadros cujo
  resumo, par, origem, protocolo ou veredito contenha o texto.
- Clique nos chips de protocolo (`osc`, `udp`, `tcp`, `http`, `mqtt`, `ws`) para mostrar só
  esses protocolos. Sem nenhum chip ligado, todos os protocolos aparecem.
- Clique em `tx` ou `rx` para mostrar só os quadros enviados ou só os recebidos.
- [[ui:common.reset]] limpa os três.

Os filtros mudam apenas o que a lista mostra. A captura, as contagens e uma exportação sempre
cobrem tudo.

### Pausar e limpar {#pause}

[[ui:ins.pause]] congela a lista para que você a leia enquanto o tráfego continua; a captura
prossegue. [[ui:ins.resume]] deixa os quadros novos entrarem de novo. Os quadros que chegaram
enquanto a exibição estava pausada não são acrescentados à lista, mas estão na captura e em
uma exportação.

[[ui:common.clear]] esvazia a lista e a captura e zera as suas contagens.

### Contagens e lacunas {#counts}

A barra no topo conta os quadros capturados e os seus bytes, e o quanto a captura está cheia
(quadros guardados de 8192).

Quando os quadros chegam mais rápido do que a lista consegue receber — mais de 250 em cerca
de um oitavo de segundo —, a lista pula os mais antigos deles. Um chip âmbar passa então a
contar os quadros não exibidos, e uma linha na lista marca onde eles faltam. Esses quadros
continuam na captura, a menos que quadros mais novos os tenham empurrado para fora: exporte-a
para vê-los.

## O detalhe de um quadro {#detail}

Clique em uma linha para ver o quadro à direita.

| Campo | O que é |
| --- | --- |
| [[ui:ins.seq]] | O número do quadro. Os números sobem na ordem da captura e nunca são reutilizados. |
| [[ui:common.time]] | Quando foi capturado. |
| [[ui:ins.direction]] | Enviado ou recebido. |
| [[ui:common.protocol]] | Como na lista. |
| [[ui:ins.source]] | A ferramenta que o capturou (`osc-send`, `osc-monitor`, `netsim`, `emulator`, `experiment-wait`, …) e o número da tarefa dela, quando pertence a uma. |
| [[ui:ins.local]] | O endereço deste lado, quando há um. Para um quadro retransmitido, o endereço em que o retransmissor escuta. |
| [[ui:bc.peer]] | O outro lado. Para um quadro retransmitido, para onde ele ia: o destino, ou o cliente a que a resposta voltou. |
| [[ui:ins.size]] | O tamanho em bytes. |
| [[ui:ins.verdict]] | Como na lista. |

Em [[ui:ins.decoded]] está o quadro lido no seu protocolo: cada mensagem de um bundle OSC com
os seus argumentos, os cabeçalhos de uma resposta HTTP e o início do corpo dela, a requisição
de um emulador e a resposta dele.

Em [[ui:ins.rawBytes]] está um dump hexadecimal: o deslocamento, 16 bytes em hex e os mesmos
bytes como texto. A lista traz os primeiros 256 bytes de cada quadro; quando um quadro é mais
longo, um botão sob o dump carrega tudo.

### O que um quadro guarda {#limits}

| Limite | Valor | No limite |
| --- | --- | --- |
| Bytes que um quadro guarda | 256 KiB | Um quadro mais longo guarda os primeiros 256 KiB e diz quanto do total guardou. |
| Quadros na captura | 8192 | O quadro mais antigo dá lugar. |
| Bytes que a captura guarda no total | 64 MiB | Os quadros mais antigos dão lugar. |

Alguns quadros não guardam bytes: as trocas HTTP (o tamanho delas é registrado, e os
cabeçalhos da resposta e o início do corpo ficam no texto decodificado) e as portas abertas
do Scanner.

Um quadro MQTT guarda a carga útil da mensagem, não o pacote do protocolo em volta dela; o
tópico, o QoS e o indicador retain estão no resumo.

### Segredos {#secrets}

Enquanto uma execução de experimento ou [[ui:exp.sendNow]] usa [segredos](../experiments/data.md#secrets),
os valores deles são mascarados em todo quadro antes de ser capturado: `••••` no resumo, no
texto decodificado, nos endereços e no veredito, e `*` para cada byte da carga útil, de modo
que os deslocamentos no dump continuam verdadeiros. As credenciais da tela HTTP também nunca
aparecem: um quadro HTTP contém a resposta, não o cabeçalho `Authorization` que foi enviado.

## Salvar um quadro como sinal {#save-as-signal}

Para guardar um pacote que você capturou e enviá-lo de novo depois — quando o dispositivo que
o enviou não está mais lá:

1. Selecione o quadro.
2. Pressione [[ui:sig.fromFrame]].

O sinal vai para a pasta [[ui:sig.capturedFolder]] da [biblioteca de sinais](signals.md) com
todos os bytes do quadro, tirados do que a captura guardou, não do texto decodificado. Ele
recebe o nome do resumo do quadro, e a nota dele diz de que quadro veio.

O que você obtém depende do quadro:

| Quadro | Sinal |
| --- | --- |
| Um datagrama OSC ou UDP | Um sinal UDP bruto com os bytes do quadro, em hex. |
| Uma publicação MQTT — enviada, recebida, ou de um emulador | Um sinal MQTT com o broker, o tópico, o QoS e o indicador retain do quadro, e a carga útil dele como texto, exatamente como era. Uma carga útil vazia é mantida, de modo que limpar um valor retido pode ser salvo. |
| Qualquer outra coisa: um fluxo TCP (inclusive um que um retransmissor carregou, ou o do nó TCP), uma troca HTTP, uma mensagem WebSocket, um pacote MQTT que não é uma publicação (o subscribe de um cliente em um emulador) | Nada: o botão fica desativado, e a dica diz por quê. Um sinal envia um datagrama ou uma publicação; estes não podem ser reenviados como eram. |

Um datagrama é enviado para:

- um quadro recebido — o endereço que o recebeu (o lado [[ui:ins.local]]), de modo que o
  sinal faz as vezes do remetente;
- um quadro enviado — o par para o qual foi enviado;
- um quadro retransmitido, nos dois sentidos — o endereço para onde ia: o destino, para um
  quadro que vai do cliente; o cliente, para uma resposta.

Quando esse endereço é todo endereço deste computador — um monitor que escuta em
`0.0.0.0:9000` ou `[::]:9000` —, o sinal é apontado para este computador: `127.0.0.1:9000` ou
`[::1]:9000`. Abra o sinal e altere o destino dele se você quer outro endereço.

Um sinal MQTT vai para o broker que o quadro indica. Um broker que escuta em todos os
endereços é alcançado em `127.0.0.1` do mesmo modo.

O botão também fica desativado para:

- um quadro não guardado inteiro: um maior que 256 KiB, ou um que não guardou bytes;
- uma mensagem MQTT cuja carga útil não é texto — a carga útil de um sinal é texto, então os
  bytes dela não poderiam ser reenviados como eram;
- um quadro recebido que não nomeia nenhum socket deste lado, de modo que não há endereço para
  onde enviar.

Um quadro que a captura já descartou também não pode ser salvo; o console avisa. Enquanto o
arquivo da biblioteca não puder ser lido, [[ui:sig.fromFrame]] também fica desativado, e a
dica mostra o erro do arquivo (veja [Sinais](signals.md#file)).

## Exportar {#export}

[[ui:ins.exportJsonl]] e [[ui:ins.exportTxt]] gravam a captura inteira — até 8192 quadros,
todos os bytes que cada um guardou, o que quer que os filtros mostrem — em um arquivo
`capture-<time>.jsonl` ou `capture-<time>.txt` na pasta de dados (veja
[Arquivos](../reference/files.md)). O console diz onde. Em um navegador conectado a um
servidor, o arquivo é gravado no servidor e o seu navegador o baixa.

Uma captura vazia não é gravada; o console diz que não há nada a salvar.

- **`.jsonl`** — um objeto JSON por linha, uma linha por quadro: `seq`, `ts` (milissegundos
  desde 1970), `proto`, `dir`, `source`, `job_id`, `local`, `remote`, `bytes`, `kept`,
  `summary`, `detail`, `hex` (o dump dos primeiros 256 bytes), `verdict` e `data`, os bytes que
  ele guardou em base64.
- **`.txt`** — para leitura: uma linha por quadro com o número, a hora, a direção, o
  protocolo, o par, o tamanho e o veredito, depois o resumo, o texto decodificado e um dump
  hexadecimal de todos os bytes que ele guardou.

```json
{"seq":12,"ts":1767225600123,"proto":"osc","dir":"rx","source":"osc-monitor","job_id":3,"local":"0.0.0.0:9000","remote":"127.0.0.1:53211","bytes":20,"summary":"/fader/1 0.75","detail":"/fader/1 0.75","hex":"0000  2f 66 61 64 65 72 2f 31  00 00 00 00 2c 66 00 00  |/fader/1....,f..|\n0010  3f 40 00 00                                       |?@..|\n","verdict":null,"kept":20,"data":"L2ZhZGVyLzEAAAAALGYAAD9AAAA="}
```

## Vindo de outros lugares {#reveal}

Outras telas apontam para quadros: uma espera na linha do tempo de um experimento liga ao
quadro com que correspondeu, e a lista de recebidos de um emulador tem um botão ⌕
([[ui:emu.inspectFrame]]) em cada troca. Segui-lo abre o Inspetor com esse quadro
selecionado, os filtros limpos e a exibição retomada.

## Relacionados {#related}

- [Sinais](signals.md) — no que um quadro salvo se transforma.
- [Degradação](impairment.md) — os vereditos do retransmissor.
- [Emuladores](emulators.md) — o que um emulador recebeu.

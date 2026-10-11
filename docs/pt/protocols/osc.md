---
title: OSC
description: Envie mensagens Open Sound Control tipadas, veja o que chega a uma porta e conduza uma forma de onda contínua até um dispositivo, tudo pela tela OSC.
---

# OSC

A tela [[ui:nav.osc]] é onde você fala Open Sound Control (OSC 1.0) sobre UDP à mão. Ela
tem três partes:

- [[ui:osc.sender]]: uma mensagem, argumentos tipados, enviada quando você pressiona Enter.
- [[ui:osc.monitor]]: escuta em uma porta e decodifica cada pacote que chega.
- [[ui:osc.generator]]: envia um valor que segue uma forma de onda, muitas vezes por
  segundo, e o desenha.

O Signal Lab codifica e decodifica OSC por conta própria. O que ele envia é o que o
[Inspetor](../tools/inspector.md) mostra, byte a byte.

## Enviar uma mensagem {#send}

1. Abra [[ui:nav.osc]].
2. Em [[ui:common.target]], informe o endereço IP ou o nome de host do dispositivo e a porta
   dele, por exemplo `127.0.0.1:9000` ou `stage-mixer.local:9000`.
3. Em [[ui:common.address]], informe o endereço que o dispositivo escuta, por exemplo
   `/mixer/fader/1`.
4. Em [[ui:common.arguments]], defina o tipo e o valor de cada argumento. Pressione
   [[ui:common.addArgument]] para outro; ✕ remove um.
5. Pressione [[ui:common.send]], ou <kbd>Enter</kbd> em qualquer campo do envio.

A linha sob os botões diz o que saiu: o endereço, o tamanho em bytes e o destino. Enviar a
mesma mensagem de novo faz a contagem subir (×2, ×3…), de modo que você vê que o reenvio
fez alguma coisa. Uma falha aparece ali no lugar disso, e também no console.

O destino, o endereço e os argumentos são mantidos quando você troca de tela e quando
reinicia o aplicativo.

### Campos {#send-fields}

| Campo | O que é | Padrão |
| --- | --- | --- |
| [[ui:common.target]] | `IP:port` ou `host:port` do receptor. Um endereço IPv6 vai entre colchetes: `[::1]:9000`. Um nome de host é resolvido a cada envio; quando tem um endereço IPv4, esse é usado (assim `localhost` alcança um receptor que escuta em `127.0.0.1`), senão o IPv6. Um destino sem porta é recusado. | `127.0.0.1:9000` |
| [[ui:common.address]] | O endereço OSC, começando com `/`, com as partes separadas por `/`. Um endereço sem a `/` inicial é recusado antes de qualquer envio. | `/hello/avatar/1` |
| [[ui:common.arguments]] | Valores tipados depois do endereço, em ordem. Uma mensagem pode não ter nenhum. | um `float`, `1.0` |

### Tipos de argumento {#types}

O tipo de cada argumento faz parte da mensagem (a sua etiqueta de tipo), então um
dispositivo que espera um float pode ignorar um int com o mesmo valor.

| Tipo na lista | Etiqueta OSC | Valor | Como você o informa |
| --- | --- | --- | --- |
| `int` | `i` | inteiro de 32 bits com sinal | um número inteiro |
| `float` | `f` | ponto flutuante de 32 bits | um número, `0.75` |
| `str` | `s` | texto | qualquer texto, enviado em UTF-8 |
| `bool` | `T` / `F` | verdadeiro ou falso | `true` ou `false` em uma lista; não carrega bytes, só a etiqueta |
| `long` | `h` | inteiro de 64 bits com sinal | um número inteiro |
| `double` | `d` | ponto flutuante de 64 bits | um número |
| `nil` | `N` | nada | sem valor |
| `blob` | `b` | bytes | não se digita aqui: aparece, somente leitura e em hexadecimal, ao abrir um sinal que tem um |

Um campo numérico que não contém um número envia `0`.

::: tip
Um `true` OSC é a etiqueta `T`, não o texto `"true"`. Um dispositivo que espera um bool
ignora em silêncio uma string.
:::

## Bundles {#bundles}

O envio manda mensagens avulsas, não bundles. Quando um bundle (`#bundle`) chega, o
monitor, as esperas dos experimentos e os emuladores o desempacotam: cada mensagem dentro
dele é tratada à parte, e a sua marca de tempo é ignorada.

## Observar uma porta {#monitor}

Para ver o que um dispositivo ou um controlador de show envia:

1. Em [[ui:common.bind]], informe o endereço e a porta em que escutar. `0.0.0.0:9000` (o
   padrão) escuta em todas as placas de rede; `127.0.0.1:9000`, só nesta máquina.
2. Pressione [[ui:osc.listen]]. O campo fica travado enquanto o monitor roda.
3. Aponte o envio para o endereço IP desta máquina e para essa porta.

Cada pacote vira uma linha, a mais recente no topo:

| Coluna | O que é |
| --- | --- |
| [[ui:common.time]] | Quando chegou, ao milissegundo |
| [[ui:osc.from]] | O `IP:port` do remetente |
| [[ui:osc.address]] | O endereço OSC, ou [[ui:osc.decodeError]] quando o pacote não é OSC válido |
| [[ui:osc.args]] | Os valores dos argumentos; um blob aparece como `blob[n]`, `nil` como `nil`, uma etiqueta de tempo como `t:` e seus segundos desde 1900 (`t:immediate` para o "agora" do OSC), um char como `'A'`, uma cor como `#rrggbbaa`, uma mensagem MIDI como `midi[01 90 3c 7f]`, um array entre `[ ]`. Para um pacote que não pôde ser decodificado, o motivo. |

Um bundle gera uma linha por mensagem. A lista guarda as últimas 300 linhas;
[[ui:common.clear]] a esvazia. Pressione [[ui:common.stop]] para fechar a porta. O monitor
também é uma tarefa na faixa do console, e por isso pode ser parado de lá.

O monitor lê pacotes de até 64 KiB. Ele decodifica todas as etiquetas de tipo do OSC 1.0 —
`i f s S b h d T F N I t c r m` e arrays `[ ]` (`S` é lido como texto, `I` como
`impulse`). Uma mensagem de uma implementação antiga, sem string de etiquetas de tipo, é lida
como uma mensagem sem argumentos, como a especificação pede. Um pacote com qualquer outra
etiqueta, cortado, ou cujo endereço não começa com `/`, aparece como erro de decodificação em
vez de ser descartado. As esperas e os emuladores leem os mesmos tipos: em um valor de resposta,
uma etiqueta de tempo é o número de 64 bits dela como texto, uma cor `#rrggbbaa`, uma mensagem
MIDI os quatro bytes dela, um array uma lista.

### Transformar uma mensagem em uma espera {#wait-for-this}

Cada linha tem um botão ⇠, [[ui:osc.waitForThis]]. Ele acrescenta ao experimento aberto uma
etapa [Aguardar OSC](../experiments/nodes.md#node-wait_osc) que escuta no
[[ui:common.bind]] do monitor por este endereço, com uma regra "igual a" para cada argumento
de texto, número inteiro e verdadeiro/falso — floats, blobs e nil não ganham nenhuma —, até
16 regras. O tempo limite dela é 2000 ms. O editor abre com a nova etapa selecionada.

::: warning
Pare o monitor antes de executar esse experimento. A execução abre a mesma porta por conta
própria, e dois ouvintes não podem dividi-la.
:::

## Conduzir uma forma de onda {#generator}

O [[ui:osc.generator]] envia uma mensagem após a outra para um endereço, com um único
argumento cujo valor segue uma forma de onda — um fader, um nível de luz, uma posição. Use-o
para ver como um dispositivo acompanha um valor em movimento, ou para carregar um receptor
com um fluxo constante.

1. Defina [[ui:common.target]] e [[ui:osc.address]]. Os dois são verificados quando você
   pressiona [[ui:osc.startGen]], como o envio os verifica.
2. Escolha uma [[ui:osc.waveform]], a sua [[ui:osc.freq]] e a [[ui:osc.rate]].
3. Defina [[ui:osc.min]] e [[ui:osc.max]], a faixa do valor.
4. Pressione [[ui:osc.startGen]]. Ele roda até você pressionar [[ui:osc.stopGen]] ou parar a
   sua tarefa na faixa do console.

| Campo | O que é | Padrão |
| --- | --- | --- |
| [[ui:common.target]] | `IP:port` ou `host:port` do receptor; um nome é resolvido uma vez, quando o gerador inicia | `127.0.0.1:9000` |
| [[ui:osc.address]] | O endereço para o qual toda mensagem é enviada; começa com `/` | `/hello/lfo` |
| [[ui:osc.waveform]] | O formato do valor ao longo do tempo (abaixo) | [[ui:wave.sine]] |
| [[ui:osc.freq]] | Ciclos da forma de onda por segundo | `1` |
| [[ui:osc.rate]] | Mensagens por segundo, de 0,1 a 5000; um valor fora da faixa é levado a ela | `60` |
| [[ui:osc.min]], [[ui:osc.max]] | O menor e o maior valor. Se Máx. ficar abaixo de Mín., o valor não se move. | `0`, `1` |
| [[ui:osc.asInt]] | Arredonda para o inteiro mais próximo e envia um `int` em vez de um `float` | desligado |

| Forma de onda | O que o valor faz em cada ciclo |
| --- | --- |
| [[ui:wave.sine]] | Oscila suavemente entre Mín. e Máx., começando no meio e subindo |
| [[ui:wave.triangle]] | Sobe de Mín. a Máx., depois desce de volta a Mín., começando em Mín. |
| [[ui:wave.saw]] | Um dente de serra descendente: começa em Máx., cai até Mín., depois salta de volta a Máx. |
| [[ui:wave.ramp]] | Um dente de serra ascendente: começa em Mín., sobe até Máx., depois salta de volta a Mín. |
| [[ui:wave.square]] | Máx. na primeira metade, Mín. na segunda |
| [[ui:wave.random]] | Um novo valor aleatório entre Mín. e Máx. a cada mensagem; a frequência não é usada |
| [[ui:wave.constant]] | Máx., todas as vezes; a frequência não é usada |

### O osciloscópio {#scope}

Ao lado dos campos, o osciloscópio desenha o valor à medida que é enviado: os últimos 300
pontos, em escala para caber. Abaixo dele ficam a forma de onda, a frequência e a taxa, e o
último valor enviado. O osciloscópio é atualizado cerca de 30 vezes por segundo, por mais
rápido que o gerador envie, então em taxas altas ele mostra uma amostra das mensagens, não
cada uma.

Se um envio falha, o gerador para e o console diz por quê.

## No Inspetor {#inspector}

Com a captura ativada ([[ui:ins.arm]] em [[ui:dock.inspector]]), o tráfego OSC aparece com o
protocolo `osc`:

| Origem | O que é | Observações |
| --- | --- | --- |
| `osc-send` | Toda mensagem que o envio, um sinal da biblioteca, uma etapa [[ui:exp.node.osc]] ou `signallab send osc` envia | Uma etapa que aguarda uma resposta aparece como `experiment` |
| `osc-monitor` | Todo pacote que o monitor recebe | Um bundle é resumido pela primeira mensagem e `+n more in bundle`; um pacote malformado tem o veredito `decode error: …` |
| `osc-gen` | As mensagens do gerador | No máximo uma a cada 100 ms é capturada, com o veredito `sampled`; a seguinte, depois de mensagens puladas, acrescenta quantas não foram desenhadas, como `sampled · +5 not shown` |

Cada quadro guarda os bytes com que foi montado. Veja [Inspetor](../tools/inspector.md).

## Salvar e reutilizar {#library}

- **Salvar como sinal.** [[ui:sig.saveNew]], sob os botões, guarda a mensagem — destino,
  endereço e argumentos — na biblioteca de sinais, em uma pasta à sua escolha. A partir daí o
  envio fica ligado a esse sinal: [[ui:sig.save]] (ou <kbd>Ctrl</kbd>+<kbd>S</kbd> no envio)
  o atualiza, [[ui:sig.saveAs]] faz uma cópia, e o chip ao lado deles o abre em
  [[ui:nav.signals]]. Dispare-o depois em [[ui:nav.signals]] ou com
  <kbd>Ctrl</kbd>+<kbd>K</kbd> de qualquer tela. Veja [Sinais](../tools/signals.md).
- **Adicionar a um experimento.** [[ui:common.toExperiment]] acrescenta ao experimento
  aberto uma etapa [Mensagem OSC](../experiments/nodes.md#node-osc) com o mesmo destino,
  endereço e argumentos — logo antes do Fim, ou depois da etapa selecionada — e o abre.
  Enquanto o experimento roda, nada pode ser adicionado; o console avisa.

## Em experimentos {#experiments}

| Etapa | O que faz |
| --- | --- |
| [[ui:exp.node.osc]] | Envia uma mensagem. O destino, o endereço e os argumentos de texto aceitam `{{templates}}`. Com [[ui:exp.expectReply]], envia de uma porta própria e aguarda ali a resposta na mesma etapa. [Detalhes](../experiments/nodes.md#node-osc) |
| [[ui:exp.node.wait_osc]] | Aguarda uma mensagem cujo endereço corresponda a um padrão e cujos argumentos passem nas regras. [Detalhes](../experiments/nodes.md#node-wait_osc) |
| [[ui:exp.node.emulator]] | Um dispositivo OSC que responde por regras durante a execução inteira. Veja [Emuladores](../tools/emulators.md). |

A etapa de mensagem OSC aceita `IP:port` ou `host:port` como a tela, e o endereço dela
precisa começar com `/`. Um nome de host é resolvido a cada envio da etapa.

### Padrões de endereço {#patterns}

[[ui:exp.node.wait_osc]], a resposta de [[ui:exp.node.osc]] e as regras de um emulador OSC
casam endereços com padrões do OSC 1.0:

| Padrão | Corresponde a |
| --- | --- |
| `*` | qualquer sequência de caracteres, inclusive nenhuma |
| `?` | exatamente um caractere |
| `[0-9]`, `[a-c]` | um caractere do conjunto ou da faixa |
| `[!0-9]` | um caractere fora do conjunto |
| `{ping,pong}` | uma das palavras |

Os curingas ficam dentro de uma parte entre barras: `/cue/*` corresponde a `/cue/7`, mas
não a `/cue/7/go`, e um padrão só corresponde a um endereço com o mesmo número de partes. A
correspondência diferencia maiúsculas de minúsculas. Um padrão começa com `/`, não tem
parte vazia (`//`), nem espaços, nem `#`, nem caracteres fora do ASCII, e tem no máximo 512
caracteres.

As regras de argumento comparam o argumento de número 0–63 com um valor (igual, menor que,
contém, corresponde a uma expressão regular e assim por diante); uma espera tem no
máximo 16. Quando um bundle chega, a espera o aceita se qualquer mensagem dentro dele corresponder.
Veja [Dados e modelos](../experiments/data.md) para o que uma mensagem correspondente dá às
etapas seguintes.

## Pela linha de comando {#cli}

`signallab send osc` envia uma mensagem como o envio da tela:

```bash
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main
```

```text
✔ sent /cue/go (24 bytes) → 127.0.0.1:9000
```

Cada argumento é `tag:value`: `i:3`, `f:0.5`, `d:1.5`, `h:64`, `s:text`, `b:de ad be ef`
(bytes em hexadecimal), ou `T`, `F`, `N` sozinhos. Sem etiqueta, um número inteiro é `i`, um
número com ponto decimal é `f`, e qualquer outra coisa é `s`; escreva `s:7` para enviar o
texto `7`. O destino é `IP:port` ou `host:port`. O comando termina com 0 quando a
mensagem saiu, 1 quando o envio falhou (inclusive um nome de host que não resolve) e 2
quando um argumento, o endereço ou o destino é inválido.
[`signallab fire`](../automation/cli.md#cli-fire) envia um sinal salvo. Veja
[Linha de comando](../automation/cli.md#cli-send-osc).

::: tip
No Git Bash do Windows, um argumento que começa com `/` é transformado em um caminho de
arquivo antes de o `signallab` vê-lo. Execute o comando com `MSYS_NO_PATHCONV=1` na frente,
ou use o PowerShell ou o `cmd`.
:::

## Problemas {#troubleshooting}

| O que você vê | Causa habitual |
| --- | --- |
| `… is not a valid address` ao enviar | O destino não tem porta, ou não é nem `IP:port` nem `host:port`. |
| `Cannot resolve …` ao enviar | O nome de host não resolve nesta máquina. Verifique-o, ou use o endereço IP. |
| `OSC addresses start with / (…)` | O endereço não tem a `/` inicial. |
| A mensagem é enviada mas o dispositivo não faz nada | Porta ou endereço errados; um tipo diferente do esperado (`int` em vez de `float`, um texto `"true"` em vez de um bool). Observe-a no Inspetor, ou aponte o destino para o monitor desta máquina para ver o que sai. |
| `… is already in use by another program` em [[ui:osc.listen]] | Outro programa — ou um experimento em execução, um emulador ou um segundo monitor — está com a porta. |
| `… is not an address of this computer` | O IP de [[ui:common.bind]] pertence a outra máquina. Use `0.0.0.0` ou um dos endereços desta máquina. |
| Pacotes de outras máquinas nunca chegam | No Windows, o firewall pode barrá-los: permita o Signal Lab quando o aplicativo oferecer. O tráfego local (`127.0.0.1`) não é afetado. Veja [Solução de problemas](../reference/troubleshooting.md). |
| Linhas [[ui:osc.decodeError]] | O remetente não fala OSC 1.0 nessa porta, ou usa uma etiqueta de tipo que o Signal Lab não decodifica. |

Em um servidor, a tela trabalha na rede do servidor: `127.0.0.1` é o próprio servidor, e o
monitor escuta nas portas do servidor. Veja [Servidor](../server/index.md).

Todas as mensagens de erro estão listadas em [Mensagens de erro](../reference/errors.md#transport).

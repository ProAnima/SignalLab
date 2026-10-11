---
title: Tempestade
description: Envie uma inundação controlada de datagramas UDP ou de conexões TCP a um servidor que é seu e acompanhe ao vivo a taxa, a vazão e os erros.
---

# Tempestade

A tela [[ui:nav.storm]] é uma fonte de carga: envia datagramas UDP, ou abre conexões TCP, a um
destino tão rápido quanto você pedir, pelo tempo que pedir, e mede o que de fato saiu. Use-a
para ver como o seu próprio servidor, dispositivo ou enlace se sai sob uma inundação — se
continua respondendo, perde pacotes ou cai.

::: danger Uso responsável
Uma tempestade envia tráfego real a um host real. Aponte-a só para hosts e redes que são seus
ou que você tem autorização para testar. Taxas altas podem saturar enlaces para todos que os
usam e disparar a detecção de intrusão. O motor não limita a taxa de uma tempestade: 0 quer
dizer o mais rápido que este computador conseguir enviar.
:::

## Iniciar uma tempestade {#start}

1. Defina [[ui:common.target]]: o `IP:port` ou `host:port` para onde enviar, como
   `127.0.0.1:9000` ou `test-rig.local:9000`. Um nome de host é resolvido quando você inicia, e o
   endereço IPv4 dele é usado quando ele tem um.
2. Escolha o [[ui:common.protocol]]: [[ui:st.udp]] ou [[ui:st.tcp]].
3. Defina [[ui:st.payloadSize]], [[ui:st.rate]] e [[ui:st.duration]].
4. Pressione [[ui:st.launch]].

A tempestade roda como uma tarefa: aparece na faixa do console e para quando a duração acaba,
quando você pressiona [[ui:st.stop]] ou com [[ui:app.stopAll]]. Os campos ficam fixos enquanto
ela roda.

| Campo | O que é | Padrão |
| --- | --- | --- |
| [[ui:common.target]] | `IP:port` ou `host:port` do destino. | `127.0.0.1:9000` |
| [[ui:common.protocol]] | [[ui:st.udp]]: datagramas separados. [[ui:st.tcp]]: uma nova conexão TCP para cada "pacote". | [[ui:st.udp]] |
| [[ui:st.payloadSize]] | Bytes em cada datagrama ou conexão, 1–65.507. Um valor maior ou menor é levado a essa faixa. | 512 |
| [[ui:st.rate]] | Pacotes (ou conexões) por segundo a atingir. 0: o mais rápido possível. | 1000 |
| [[ui:st.duration]] | Segundos de execução. 0: até você pará-la. | 10 |

### Inundação UDP {#udp}

O Signal Lab envia datagramas do tamanho da carga útil, com todos os bytes `0x55`, de uma
porta própria para o destino. Um envio que o sistema recusa conta como erro — por exemplo,
depois que o destino respondeu que nada escuta nessa porta.

### Inundação de conexões TCP {#tcp}

Para cada "pacote", o Signal Lab abre uma conexão TCP com o destino, escreve a carga útil e a
fecha. As conexões são feitas uma após a outra, não ao mesmo tempo, então a taxa que uma
tempestade TCP alcança é limitada pela rapidez com que o destino as aceita. Uma conexão
recusada, ou não aceita em 500 ms, conta como erro; uma conexão que recebeu a carga útil conta
como pacote.

### Como a taxa é mantida {#pacing}

A taxa é um cronograma. O primeiro pacote sai na hora, e o pacote *n* vence *n* ÷
[[ui:st.rate]] segundos depois do início. Cada vez que a tempestade acorda, envia o que
venceu e depois dorme até o próximo pacote vencer. Assim, 50 por segundo são 50 por segundo e
250 são 250 — uma execução de um segundo envia mais ou menos esse número —, seja qual for a
granularidade do temporizador do sistema. Uma [[ui:st.duration]] encerra a tempestade no
prazo, mesmo quando o próximo pacote venceria mais tarde.

Uma tempestade nunca envia acima da sua taxa para compensar o tempo perdido. Se este
computador, ou um destino TCP que aceita devagar, ficar atrasado em mais de 256 pacotes, os
mais antigos são retirados do cronograma em vez de enviados atrasados em uma rajada.
[[ui:st.pps]] mostra o que foi alcançado.

Com [[ui:st.rate]] 0 não há cronograma: a tempestade envia 256 pacotes, deixa outro trabalho
rodar e envia os 256 seguintes, o mais rápido que este computador conseguir.

## Ler a vazão {#metrics}

[[ui:st.throughput]] é atualizada quatro vezes por segundo:

| Métrica | O que é |
| --- | --- |
| [[ui:common.packets]] | Datagramas enviados, ou conexões que receberam a carga útil, desde o início. |
| [[ui:st.pps]] | Pacotes por segundo no último quarto de segundo. |
| [[ui:st.rateLabel]] | Megabits por segundo de carga útil no último quarto de segundo. Os cabeçalhos (IP, UDP, TCP) não são contados. |
| [[ui:common.volume]] | Bytes de carga útil enviados desde o início. |
| [[ui:common.errors]] | Envios ou conexões que falharam. |

O gráfico abaixo delas traça [[ui:st.pps]] no último minuto.

Quando a tempestade termina, os contadores mostram os totais finais, e [[ui:st.pps]] e
[[ui:st.rateLabel]] vão a 0.

O que os números dizem:

- [[ui:st.pps]] bem abaixo de [[ui:st.rate]] em UDP: este computador não consegue enviar mais
  rápido. Reduza a taxa ou o tamanho da carga útil.
- [[ui:common.errors]] subindo em UDP: a porta do destino está fechada, ou a rede recusa o
  tráfego.
- [[ui:common.errors]] subindo em TCP: o destino recusa conexões ou leva mais de 500 ms para
  aceitá-las — ele pode ter atingido o seu limite.

Uma tempestade diz quanto saiu, não quanto chegou. Para ver o que o destino recebeu,
observe-o: os registros dele, um monitor [[ui:nav.osc]] ou um [emulador](emulators.md) no
lugar dele.

## No Inspetor {#inspector}

Com a captura ligada, uma tempestade UDP põe um dos seus datagramas no
[Inspetor](inspector.md) por segundo, marcado `sampled 1/s` — são todos iguais. O veredito
também diz quantos ficaram de fora desde o anterior, como `sampled 1/s · +999 not shown`. Uma
tempestade TCP não põe nenhum.

## Relacionados {#related}

- [Degradação](impairment.md) — um enlace lento ou com perdas em vez de uma inundação.
- [HTTP](../protocols/http.md) — rajadas de carga de requisições HTTP, com latências.
- [Carga](../experiments/load.md) — carga HTTP em um experimento, com limiares.

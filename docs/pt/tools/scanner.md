---
title: Scanner
description: Descubra quais portas TCP de um host aceitam conexões, com a saudação que cada serviço envia, usando uma varredura de conexão TCP concorrente.
---

# Scanner

A tela [[ui:nav.scan]] tenta abrir uma conexão TCP com cada porta de uma faixa em um host e
lista as portas que aceitam. Use-a para confirmar quais serviços de fato escutam em um
dispositivo — a porta de controle de um projetor, a interface web de um switch, a porta que o
seu próprio serviço deveria abrir — e, para serviços que saúdam primeiro, o que eles dizem.

::: danger Uso responsável
Uma varredura conecta-se a cada porta da faixa. Varra só hosts que são seus ou que você tem
autorização para testar: em outras redes uma varredura pode disparar a detecção de intrusão e
muitas vezes contraria as regras.
:::

## Varrer um host {#scan}

1. Informe o [[ui:sc.host]]: um host, um endereço IP ou um nome. Um nome é resolvido uma vez,
   quando a varredura começa; um que não resolve a interrompe ali, com o motivo.
2. Defina [[ui:sc.fromPort]] e [[ui:sc.toPort]], ou escolha uma [[ui:sc.preset]].
3. Ajuste [[ui:common.concurrency]] e [[ui:common.timeoutMs]] se precisar.
4. Deixe [[ui:sc.grabBanner]] ligado para ler o que cada serviço diz primeiro.
5. Pressione [[ui:sc.startScan]].

Uma barra sob o botão mostra quantas portas já foram tentadas, de quantas, e quantas estão
abertas. As portas abertas aparecem à direita à medida que são encontradas. A varredura
termina quando todas as portas foram tentadas; [[ui:sc.stopScan]] a encerra antes, e as
portas ainda não tentadas não são tentadas. Os campos ficam fixos enquanto ela roda.

| Campo | O que é | Padrão |
| --- | --- | --- |
| [[ui:sc.host]] | O host a varrer. | `127.0.0.1` |
| [[ui:sc.fromPort]], [[ui:sc.toPort]] | A faixa, com as duas pontas incluídas, 1–65.535: um número digitado além de qualquer uma das pontas para ali. Quando a primeira é maior, elas são trocadas. | 1–1024 |
| [[ui:common.concurrency]] | Quantas tentativas de conexão ficam abertas ao mesmo tempo, 1–1024. | 400 |
| [[ui:common.timeoutMs]] | Quanto esperar por cada conexão, 50–10.000 ms. | 500 |
| [[ui:sc.grabBanner]] | Depois de conectar, aguardar até 400 ms para o serviço enviar algo e guardar os primeiros 256 bytes. | ligado |

Uma [[ui:common.concurrency]] ou [[ui:common.timeoutMs]] fora da sua faixa é levada a ela
quando a varredura começa.

[[ui:sc.preset]] preenche a faixa:

| Predefinição | Portas |
| --- | --- |
| [[ui:sc.preset.wellKnown]] | 1–1024 |
| [[ui:sc.preset.common]] | 1–10.000 |
| [[ui:sc.preset.osc]] | 8000–9100 |
| [[ui:sc.preset.full]] | 1–65.535 |

## Quanto tempo leva uma varredura {#duration}

Uma porta que aceita responde na hora. Uma que não aceita pode custar até o tempo limite
inteiro: um firewall que descarta tentativas de conexão nunca responde, e no Windows até uma
recusa pode levar mais do que o tempo limite padrão. Assim, uma varredura de um host que não
responde nada leva cerca de (no Windows uma porta recusada leva cerca de 2 s, porque o
Windows tenta de novo, então ali o tempo limite decide):

```text
ports ÷ concurrency × timeout
```

A faixa completa nos padrões: 65.535 ÷ 400 × 0,5 s ≈ 82 s. Aumente [[ui:common.concurrency]] ou
reduza [[ui:common.timeoutMs]] para ir mais rápido; reduza demais o tempo limite e as portas
abertas de um host lento passam despercebidas.

## Ler os resultados {#results}

[[ui:sc.openPorts]] lista toda porta que aceitou uma conexão, em ordem de porta:

| Coluna | O que é |
| --- | --- |
| [[ui:sc.port]] | A porta aberta. |
| [[ui:common.time]] | Quando foi encontrada. |
| [[ui:sc.banner]] | O que o serviço enviou primeiro, com as quebras de linha viradas em espaços, ou `—`. |

Uma porta que recusou e uma que não respondeu dentro do tempo limite ficam ambas de fora: o
scanner não distingue fechada de filtrada. A lista guarda toda porta aberta, sejam quantas
forem.

Poucas portas abertas é o resultado habitual: um PC com Windows costuma escutar nas portas
135 e 445 (e 139 no seu endereço de rede) entre as primeiras mil, e a maioria dos dispositivos
em um punhado. Para comparar, liste o que este computador escuta com `netstat -an` (ou
`Get-NetTCPConnection -State Listen` no PowerShell).

### Portas não verificadas {#unchecked}

Cada conexão usa uma das portas deste próprio computador. Quando elas se esgotam — ou os
sockets —, uma tentativa falha aqui, antes de chegar ao host; o scanner espera e tenta de
novo, e uma porta que ainda assim não consegue tentar é contada como *não verificada*,
mostrada em âmbar ao lado da contagem de abertas, nunca como fechada. Se você a vir, varra de
novo com uma [[ui:common.concurrency]] menor.

Só serviços que falam primeiro têm banner — SSH, SMTP, FTP, muitos protocolos de controle de
dispositivos. Um servidor web espera uma requisição, então a porta dele mostra `—`. Capturar
banners faz cada porta aberta levar até 400 ms a mais.

Uma conexão que o scanner abre é fechada de novo na hora, com uma redefinição, para não
segurar a porta deste computador por minutos depois. O scanner não envia nada por ela.

Com a captura ligada, toda porta aberta também está no [Inspetor](inspector.md), com o seu
banner e o veredito `open`.

## Relacionados {#related}

- [Tempestade](storm.md) — carga em uma porta que você encontrou.
- [UDP e TCP](../protocols/udp-tcp.md) — converse com ela.
- [Solução de problemas](../reference/troubleshooting.md)

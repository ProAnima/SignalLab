---
title: HTTP
description: Envie uma requisição HTTP e leia a resposta inteira, autentique-se com Basic, Bearer ou Digest, mantenha cookies e meça um endpoint sob uma rajada de carga concorrente.
---

# HTTP

A tela [[ui:nav.http]] é um inspetor de requisições e uma ferramenta de carga ao mesmo
tempo:

- envie uma única requisição e veja o status, o tempo que levou, os cabeçalhos e o corpo;
- autentique-se com Basic, um token Bearer ou Digest;
- mantenha os cookies que um servidor define, como faz um navegador;
- envie a mesma requisição muitas vezes ao mesmo tempo — uma [[ui:http.burst]] — e leia a
  vazão e os percentis de latência.

## Enviar uma requisição {#request}

1. Abra [[ui:nav.http]].
2. Escolha o método e digite a URL, por exemplo `http://127.0.0.1:8080/health`.
3. Acrescente [[ui:http.headers]] se o servidor precisar deles; [[ui:http.addHeader]]
   acrescenta uma linha, ✕ remove uma. Uma linha sem nome não é enviada.
4. Para um método diferente de GET e HEAD, escreva o [[ui:http.body]]. O texto permanece no
   campo enquanto você passa para GET ou HEAD e volta com o outro método, mas não é enviado
   nesse meio-tempo.
5. Pressione [[ui:common.send]].

A linha sob os botões dá o veredito na hora — status, tempo e tamanho, ou por que não houve
resposta — e o painel [[ui:http.response]] mostra o resto. A requisição (método, URL,
cabeçalhos, corpo, tempo limite e [[ui:http.keepCookies]]) é mantida quando você troca de
tela e quando reinicia o aplicativo; as credenciais não.

| Tecla | Onde | O que faz |
| --- | --- | --- |
| <kbd>Enter</kbd> | a URL, um cabeçalho, uma credencial, o tempo limite | envia |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | qualquer campo da requisição, inclusive o corpo | envia |
| <kbd>Ctrl</kbd>+<kbd>S</kbd> | qualquer campo da requisição | salva como sinal ([abaixo](#library)) |

### Campos da requisição {#request-fields}

| Campo | O que é | Padrão |
| --- | --- | --- |
| [[ui:exp.method]] | GET, POST, PUT, PATCH, DELETE, HEAD ou OPTIONS | GET |
| [[ui:field.url]] | Uma URL `http://` ou `https://` | `http://127.0.0.1:8080/` |
| [[ui:http.headers]] | Pares de nome e valor, enviados como escritos. Sem um `User-Agent` seu, o Signal Lab envia `SignalLab/0.1`. | `Accept: application/json` |
| [[ui:field.auth]] | Como a requisição se autentica ([abaixo](#auth)) | [[ui:http.auth.none]] |
| [[ui:http.keepCookies]] | Devolver os cookies que os servidores definem ([abaixo](#cookies)) | ligado |
| [[ui:http.body]] | Enviado exatamente como escrito; nenhum `Content-Type` é acrescentado, então inclua o cabeçalho correspondente. Um corpo vazio não é enviado. O campo não aparece para GET e HEAD, e então a requisição não leva corpo algum — nem no envio, nem em um sinal salvo, nem em [[ui:common.toExperiment]] — mesmo que você tenha digitado um sob outro método. | vazio |
| [[ui:common.timeoutMs]] | Quanto tempo a troca inteira pode levar, resposta e corpo incluídos: de 1 ms a 120.000 (dois minutos), como na etapa HTTP | 10.000 |

## Autenticação {#auth}

| [[ui:field.auth]] | Campos | O que é enviado |
| --- | --- | --- |
| [[ui:http.auth.none]] | — | nenhum cabeçalho `Authorization` |
| [[ui:http.auth.basic]] | [[ui:field.username]], [[ui:field.password]] | `Authorization: Basic …`, o nome e a senha em base64, já na primeira requisição |
| [[ui:http.auth.bearer]] | [[ui:field.token]] | `Authorization: Bearer <token>` |
| [[ui:http.auth.digest]] | [[ui:field.username]], [[ui:field.password]] | nada no início; a resposta ao desafio do servidor ([abaixo](#digest)) |

Alternar entre Basic e Digest mantém o nome e a senha.

### Digest {#digest}

Com Digest, o Signal Lab envia a requisição sem credenciais. Quando o servidor responde
`401` com um desafio Digest, o Signal Lab calcula a resposta a partir do desafio e da sua
senha e envia a requisição de novo. A resposta que você vê é a dessa segunda requisição,
marcada com [[ui:http.digestAnswered]], e a latência soma as duas trocas — o que um cliente
espera.

- Algoritmos: MD5 e SHA-256, e as suas variantes `-sess`. Quando um servidor oferece os dois,
  usa-se SHA-256.
- Qualidade de proteção: `auth` e `auth-int`, e a resposta mais antiga sem `qop`.
- Quando o servidor diz que o nonce expirou (`stale`), ou pede de novo com um nonce novo, a
  requisição é respondida outra vez, até 3 vezes a mais. Quando ele recusa a resposta ao
  último nonce que deu, o `401` permanece: o nome ou a senha está errado.
- Os redirecionamentos são seguidos pelo próprio Signal Lab, então a URL que faz o pedido é
  a que é respondida. Um desafio de outra origem não é respondido: as credenciais digitadas
  para um host não vão a nenhum outro. Passar de `http://` para `https://` no mesmo host e
  nas portas padrão conta como o mesmo host.

Quando o desafio não pode ser respondido, o `401` permanece e o painel diz por quê:

| Mensagem | Significado |
| --- | --- |
| O servidor respondeu 401 sem pedir Digest | O servidor quer outro esquema; tente Basic ou Bearer. |
| O servidor pediu Digest com … | Um algoritmo que o Signal Lab não suporta; ele suporta MD5 e SHA-256. |
| O servidor pediu Digest sem realm ou nonce | O desafio do servidor está incompleto. |
| A requisição foi encaminhada para …, que pediu Digest | Um redirecionamento levou a outra origem, cujo desafio não é respondido. |

### Para onde vão as credenciais {#credentials}

As credenciais vão apenas no cabeçalho `Authorization` da requisição, no momento em que ela
é enviada. O Inspetor, o console e os relatórios de experimento nunca mostram esse
cabeçalho. Nesta tela elas ficam só na memória e somem depois de uma reinicialização — a
menos que a requisição esteja ligada a um sinal salvo, que as traz de volta.

::: warning
Uma requisição salva como sinal guarda as suas credenciais no arquivo da biblioteca,
`signals.json`, em texto simples. Em um experimento, escreva uma senha como
`{{secret.NAME}}`; veja [Dados e modelos](../experiments/data.md).
:::

## Cookies {#cookies}

Com [[ui:http.keepCookies]] ligado, o que um servidor define com `Set-Cookie` é guardado no
armazenamento de cookies da tela e devolvido nas requisições seguintes a esse servidor,
seguindo as regras do navegador (domínio, caminho, `Secure`, validade). O armazenamento é
usado pelas requisições desta tela, pela sua rajada e pelos sinais HTTP que você dispara da
biblioteca. Desligue-o para enviar requisições sem cookies e não guardar nenhum.

O painel de cookies sob a requisição e a resposta lista o que o armazenamento contém:
[[ui:http.cookieName]], [[ui:http.cookieValue]], [[ui:http.cookieWhere]] (um domínio que
começa com `.` cobre também os seus subdomínios), [[ui:http.cookieExpires]]
([[ui:http.cookieSession]] para um cookie sem validade) e [[ui:http.cookieFlags]]
(`Secure`, `HttpOnly`, `SameSite`). Cookies expirados não são listados. [[ui:common.clear]]
esvazia o armazenamento.

O armazenamento guarda no máximo 3000 cookies, mais ou menos o que um navegador guarda. Um
servidor que define um cookie novo a cada resposta o enche até esse ponto e não além: depois
dele um cookie novo não é guardado, enquanto um que o armazenamento já tem ainda é
atualizado ou removido.

O armazenamento vive enquanto o aplicativo: uma reinicialização começa com um vazio. Em um
[servidor](../server/index.md), há um só para todas as páginas conectadas a ele. Uma
execução de experimento tem um armazenamento próprio (veja
[Experimentos](../experiments/index.md)), e `signallab send http` não usa nenhum.

## A resposta {#response}

| Parte | O que é |
| --- | --- |
| [[ui:http.status]] | O código de status e o seu motivo; `ERR` quando nenhuma resposta veio |
| [[ui:http.latency]] | Do envio ao último byte do corpo, em milissegundos |
| [[ui:http.size]] | O tamanho do corpo |
| Cabeçalhos da resposta | Clique na linha com a contagem deles para mostrar ou ocultar |
| Corpo | Formatado quando é JSON; [[ui:http.rawBody]] e [[ui:http.formatJson]] alternam. Até 256 KiB são mostrados, depois `… (truncated)`; [[ui:http.size]] continua sendo o do corpo inteiro. |

O corpo é lido à medida que chega e só os primeiros 256 KiB são guardados, então um corpo de
gigabytes — ou um que nunca termina — não custa mais memória do que isso; ele termina no
tempo limite. Um corpo que para no meio (o tempo limite, ou um servidor que desliga) é uma
falha, dita com as mesmas palavras de qualquer outra, com o status e o que chegou ainda
mostrados: um `200` com metade de um corpo não é um sucesso.

Quando não há resposta, o painel diz por quê, com as mesmas palavras de todo o Signal Lab:
recusada, sem resposta a tempo, o nome não resolve, um problema de certificado e assim por
diante. O detalhe técnico do sistema fica recolhido embaixo.

### Redirecionamentos {#redirects}

Os redirecionamentos (301, 302, 303, 307, 308) são seguidos, até 10; a resposta mostrada é a
última. Depois de 301, 302 e 303 a requisição continua como GET sem corpo (HEAD continua
HEAD); depois de 307 e 308, como estava. O `Authorization` e os cookies digitados para um
host não são enviados a outro.

### Conexões seguras {#tls}

O certificado de um servidor `https://` é verificado contra os certificados em que este
sistema confia. Um certificado autoassinado ou expirado é recusado com "Não foi possível
estabelecer uma conexão segura com …"; não há configuração para pular a verificação. Para
testar um servidor com o seu próprio certificado, acrescente-o aos certificados confiáveis
do sistema.

## Rajada de carga {#burst}

A [[ui:http.burst]] envia a requisição que está na tela — com a sua autenticação e, enquanto
[[ui:http.keepCookies]] estiver ligado, o armazenamento de cookies — muitas vezes, e a mede.

1. Defina [[ui:common.concurrency]], [[ui:http.total]], [[ui:http.duration]] e
   [[ui:http.rate]].
2. Pressione [[ui:http.startBurst]]. A rajada é uma tarefa: [[ui:http.stopBurst]], ou
   pará-la na faixa do console, a encerra.

| Campo | O que é | Padrão |
| --- | --- | --- |
| [[ui:common.concurrency]] | Requisições em andamento ao mesmo tempo, 1–512 | 20 |
| [[ui:http.total]] | Requisições a enviar; 0 — continuar enviando até a duração acabar | 500 |
| [[ui:http.duration]] | Segundos de execução; 0 — parar quando o total for enviado | 0 |
| [[ui:http.rate]] | Requisições iniciadas por segundo, 0,1–100.000; 0 — o mais rápido que os trabalhadores aguentarem | 0 |

Com [[ui:http.total]] e [[ui:http.duration]] ambos em 0, a rajada roda até você pará-la.

Há duas formas de enviar:

- **[[ui:http.rate]] 0.** Cada um dos trabalhadores envia de novo assim que tem uma
  resposta. Isso descobre quanto o servidor aguenta, mas um servidor lento também deixa a
  rajada mais lenta.
- **Uma taxa.** As requisições começam em um cronograma fixo — a 10 por segundo, uma a
  cada 100 ms desde o início —, por mais lentas que sejam as respostas. Uma requisição cujo
  momento chega enquanto todos os trabalhadores estão ocupados espera no máximo 50 ms por
  um; depois disso ela é pulada e contada como [[ui:http.missed]], nunca enviada atrasada.
  Requisições puladas significam que a concorrência está baixa para essa taxa, ou que o
  servidor é mais lento do que a taxa exige.

| Número | O que é |
| --- | --- |
| [[ui:http.sent]] | Requisições que tiveram uma resposta ou falharam |
| [[ui:http.ok]] | Respondidas com um status 2xx e o corpo inteiro |
| [[ui:http.failed]] | Sem resposta, um corpo cortado ou qualquer status fora de 200–299 |
| [[ui:http.local]] | Requisições que este computador não conseguiu enviar: ficou sem sockets ou portas locais (cada conexão fechada segura uma por até dois minutos no Windows). O servidor nunca as viu, então elas estão em [[ui:http.sent]], mas não em [[ui:http.failed]] nem nas latências. Aparece só quando há alguma. |
| [[ui:http.missed]] | Puladas, como acima (só com uma taxa) |
| [[ui:http.rps]] | Requisições por segundo no último décimo de segundo; quando a rajada terminou, na rajada inteira. Com uma taxa, o rótulo nomeia a taxa pedida. |
| [[ui:http.p50]], [[ui:http.p90]], [[ui:http.p95]], [[ui:http.p99]] | O tempo em que essa fração das requisições terminou, falhas incluídas; preciso dentro de 0,5 % |
| [[ui:http.avg]], [[ui:http.min]], [[ui:http.max]] | A média, a mais rápida e a mais lenta |

Os números são atualizados cerca de 10 vezes por segundo. O gráfico ao lado desenha as
requisições por segundo dos últimos 24 segundos, mais ou menos.

Com Digest, o desafio da primeira requisição é respondido uma vez e essa resposta serve a
todas as requisições da rajada.

::: warning
Uma rajada é carga real. Aponte-a só para servidores que são seus ou que você tem
autorização para testar.
:::

Para rampas, degraus, picos e limiares de aprovação ou reprovação, execute a requisição sob
[carga em um experimento](../experiments/load.md).

## No Inspetor {#inspector}

Com a captura ativada, cada troca aparece como um quadro com o protocolo `http` e a origem
`http`: o método, a URL, o status e o tempo no resumo, os cabeçalhos da resposta e o início
do corpo (2000 caracteres) no detalhe, o status como veredito (`failed` quando nenhuma
resposta veio, `· digest after 401` quando um desafio foi respondido). O quadro registra o
tamanho do corpo, não os seus bytes. O cabeçalho `Authorization` da requisição nunca está
nele. Uma rajada põe na captura no máximo uma troca a cada 100 ms. Veja
[Inspetor](../tools/inspector.md).

## Salvar e reutilizar {#library}

- **Salvar como sinal.** [[ui:sig.saveNew]] guarda a requisição — método, URL, cabeçalhos,
  corpo, tempo limite e autenticação — na biblioteca de sinais. A tela continua ligada a ele:
  [[ui:sig.save]] (<kbd>Ctrl</kbd>+<kbd>S</kbd>) o atualiza, [[ui:sig.saveAs]] o copia, o
  chip o abre em [[ui:nav.signals]]. Abrir um sinal HTTP da biblioteca o carrega de volta
  aqui, credenciais incluídas. Veja [Sinais](../tools/signals.md).
- **Adicionar a um experimento.** [[ui:common.toExperiment]] acrescenta ao experimento aberto
  uma etapa [Requisição HTTP](../experiments/nodes.md#node-http) com a mesma requisição,
  logo antes do Fim ou depois da etapa selecionada, e o abre.
- **Simular isto.** Abaixo de uma resposta, [[ui:http.mockThis]] cria uma rota de emulador que
  responde a este método e caminho com este status, estes cabeçalhos e este corpo. Escolha
  um emulador HTTP em [[ui:http.mockInto]], ou [[ui:http.mockNew]], e pressione
  [[ui:http.mockAdd]]; a rota vai em primeiro lugar nesse emulador, e a tela
  [[ui:nav.emulators]] abre nela. Veja [Emuladores](../tools/emulators.md).

## Em experimentos {#experiments}

| Etapa | O que faz |
| --- | --- |
| [[ui:exp.node.http]] | Envia uma requisição; a URL, os cabeçalhos, o corpo e as credenciais aceitam `{{templates}}`. Pode rodar [sob carga](../experiments/load.md). [Detalhes](../experiments/nodes.md#node-http) |
| [[ui:exp.node.assert_status]], [[ui:exp.node.assert_body]], [[ui:exp.node.assert_header]], [[ui:exp.node.assert_latency]] | Verificam a última resposta. [Detalhes](../experiments/nodes.md#node-assert_status) |
| [[ui:exp.node.extract]] | Guarda como variável um campo JSON, um cabeçalho, o status, o corpo ou a correspondência de uma expressão regular. [Detalhes](../experiments/nodes.md#node-extract) |
| [[ui:exp.node.branch_status]] | Segue por Sim ou Não conforme o status. [Detalhes](../experiments/nodes.md#node-branch_status) |
| [[ui:exp.node.wait_http]] | Aguarda a chegada de uma requisição — do sistema que você testa — ao ouvinte ou emulador da própria execução. [Detalhes](../experiments/nodes.md#node-wait_http) |
| [[ui:exp.node.emulator]] | Uma API HTTP que responde por rotas durante a execução inteira. [Detalhes](../experiments/nodes.md#node-emulator) |

## Pela linha de comando {#cli}

`signallab send http` envia uma requisição, como esta tela:

```bash
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab send http POST http://127.0.0.1:8080/api/items \
  -H 'Content-Type: application/json' --body '{"name":"lamp"}'
signallab send http GET http://127.0.0.1:8080/private -u admin:secret --digest
```

A linha de status vai para a saída de erro padrão e o corpo para a saída padrão:

```text
HTTP 200 OK · 3 ms · 15 B
{"status":"ok"}
```

| Opção | O que faz | Padrão |
| --- | --- | --- |
| `-H`, `--header 'Name: value'` | Um cabeçalho; repita para mais | — |
| `--body TEXT`, `--body @FILE` | O corpo, ou o conteúdo de um arquivo | — |
| `--expect-status N` | Termina com 1, a menos que o status seja N | — |
| `--timeout MS` | Quanto tempo aguardar a resposta | 10.000 |
| `-u`, `--user NAME:PASSWORD` | Autenticação Basic | — |
| `--digest` | Com `--user`: responde ao desafio Digest do servidor em vez disso | — |
| `--bearer TOKEN` | `Authorization: Bearer TOKEN` | — |
| `--json` | Imprime a resposta inteira como JSON na saída padrão | — |

O comando termina com 0 quando uma resposta veio (e tinha o status esperado), 1 quando
nenhuma veio, o status não era o esperado ou um desafio Digest não pôde ser respondido, e 2
quando uma opção é inválida. Ele não guarda cookies. Veja
[Linha de comando](../automation/cli.md#cli-send-http).

## Problemas {#troubleshooting}

| O que você vê | Causa habitual |
| --- | --- |
| `… refused the connection — nothing is listening on that port` | O servidor não está rodando, ou escuta em outra porta ou endereço. |
| `No answer from … in time` | O servidor está lento ou inalcançável; verifique o endereço, ou aumente [[ui:common.timeoutMs]]. |
| `Cannot resolve …` | O nome de host não resolve nesta máquina — um erro de digitação, ou um nome que só outra rede conhece. |
| `A secure connection to … could not be made` | O certificado não é confiável aqui (autoassinado, expirado, outro nome), ou o TLS falhou. Veja [Conexões seguras](#tls). |
| `… is not a valid address` | A URL está malformada ou não começa com `http://` nem `https://`. |
| O servidor diz que o corpo está ausente ou é do tipo errado | Nenhum cabeçalho `Content-Type` que corresponda ao corpo, ou um corpo vazio. |
| `401` com Digest | Leia a mensagem sob o status: veja [Digest](#digest). |
| [[ui:http.missed]] acima de 0 | Aumente [[ui:common.concurrency]], ou reduza a taxa: o servidor responde mais devagar do que a taxa exige. |
| [[ui:http.failed]] alto embora o servidor responda | Todo status fora de 200–299 conta como falha, 404 e 500 inclusive. |
| [[ui:http.local]] acima de 0 | Este computador ficou sem portas locais: o servidor fecha cada conexão, então cada requisição precisa de uma nova. Reduza [[ui:common.concurrency]] ou a taxa, ou aguarde um minuto para as portas voltarem. |

Em um servidor, as requisições saem do servidor: `127.0.0.1` é o próprio servidor. Veja
[Servidor](../server/index.md).

Todas as mensagens de erro estão listadas em [Mensagens de erro](../reference/errors.md#transport).

---
title: Dados e modelos
description: Parâmetros e perfis, a linguagem de modelos com seus geradores, valores extraídos de respostas, comparações e segredos que nunca saem do motor.
---

# Dados nos experimentos

Os valores circulam por uma execução: um parâmetro escolhe o destino, um campo de uma
resposta vira um cabeçalho da requisição seguinte, um id gerado sai em um
comando e volta em uma verificação. Esta página trata de onde esses valores
vêm e de como um campo os usa.

| Origem | Escrita como | Definida onde |
| --- | --- | --- |
| Parâmetro | `{{api}}` ou `{{params.api}}` | painel [[ui:exp.params]], um perfil, [[ui:exp.runWith]] |
| Variável | `{{token}}` ou `{{vars.token}}` | um nó durante a execução: [[ui:exp.node.extract]], uma espera, um envio que aguarda uma resposta |
| Segredo | `{{secret.API_TOKEN}}` | o repositório de credenciais do computador, ou o ambiente e os arquivos do servidor |
| Valor embutido | `{{run.seed}}`, `{{now.iso}}`, `{{counter}}` | a própria execução |
| Gerador | `{{uuid}}`, `{{random_int(1, 100)}}` | sorteado a partir da semente da execução |

## Parâmetros {#parameters}

Um parâmetro é um valor de texto com nome que qualquer campo com modelo pode usar. Guarde
os destinos em parâmetros, de modo que uma mudança de endereço seja uma edição em vez de uma por
nó.

### Adição de um parâmetro {#add-parameter}

1. Pressione [[ui:exp.params]] (`{ }`) na barra de ferramentas do editor.
2. Na aba [[ui:exp.noProfile]], pressione [[ui:exp.addParam]].
3. Digite o [[ui:exp.paramName]] e o [[ui:exp.paramValue]], por exemplo
   `api` e `http://127.0.0.1:8080`.
4. No campo de um nó, escreva `{{api}}/login`.

Toda alteração no painel é uma edição do experimento: é salva com ele
e desfeita com <kbd>Ctrl</kbd>+<kbd>Z</kbd> como qualquer outra.

### Regras {#parameter-rules}

| Regra | Limite |
| --- | --- |
| Nome | começa com uma letra ou `_`, depois letras, dígitos e `_` |
| Nomes reservados | `vars`, `params`, `secret`, `run`, `node`, `now`, `uuid`, `counter`, `random_int`, `random_float`, `pick` |
| Parâmetros por experimento | 64 |
| Tamanho de um valor | 64 KiB |
| Nomes | únicos; uma variável não pode ter o nome de um parâmetro |

Um valor é texto simples e é inserido como foi escrito: `{{…}}` dentro de um valor não
é resolvido. Quando um campo pede uma parte de um parâmetro
(`{{config.ports[0]}}`), o valor é lido como JSON; um valor que não é JSON
não tem partes.

Um parâmetro cujo nome é inválido, reservado ou repetido não impede o
experimento de ser salvo, então você pode continuar digitando; o experimento não executa
até o nome ser corrigido.

## Perfis {#profiles}

Um perfil é um conjunto nomeado de valores de parâmetros — *Notebook*, *Palco*, *Local do evento* — de modo que
trocar o destino é uma escolha, não uma edição de todos os nós. Um perfil
altera alguns parâmetros; os outros mantêm o padrão.

### Criação de um perfil {#make-profile}

1. Abra [[ui:exp.params]] e pressione [[ui:exp.addProfile]]. Uma nova aba abre.
2. Renomeie-o em [[ui:exp.profileName]].
3. Para cada parâmetro que o perfil altera, digite o valor. Um campo vazio mantém
   o padrão, mostrado em cinza no campo; [[ui:exp.resetToDefault]] (↺) limpa
   um valor.
4. Pressione [[ui:exp.makeActive]] para executar com ele. A aba do perfil ativo traz
   ● [[ui:exp.activeProfile]]. [[ui:exp.makeActive]] na
   aba [[ui:exp.noProfile]] volta aos padrões.

Quando um experimento tem perfis, uma lista [[ui:exp.profile]] na barra de ferramentas
alterna entre eles. O perfil ativo é usado pelas execuções, pela prévia e por
[[ui:exp.sendNow]], e é salvo no experimento, de modo que um arquivo exportado abre
com os mesmos destinos. [[ui:exp.removeProfile]] exclui o perfil em exibição.

| Regra | Limite |
| --- | --- |
| Perfis por experimento | 32 |
| Nome | 1–64 caracteres, único (espaços nas pontas não contam) |
| Valores | só parâmetros que existem; no máximo 64 |

Renomear ou remover um parâmetro o altera em todos os perfis de uma vez.

### Qual valor uma execução usa {#precedence}

O último vence:

1. o padrão do parâmetro, na aba [[ui:exp.noProfile]];
2. o valor do perfil ativo, se ele define um;
3. um valor digitado em [[ui:exp.runWith]] só para esta execução — veja
   [executar com outros valores](runs.md#run-with).

[[ui:exp.runWith]] só pode definir parâmetros que o experimento tem. O relatório da execução
registra o perfil, os valores digitados para a execução e todos os valores que ela usou.

### Perfis que não executariam {#profile-issues}

A cada verificação do experimento, os outros perfis e os padrões também são
verificados. Um que falharia — digamos, uma URL que não é `http://` nem
`https://` — traz ⚠ nas abas e na lista da barra de ferramentas, e a dica dele
diz por quê. Isso não impede as execuções com o perfil em uso.

## Modelos {#templates}

O texto dentro de `{{ }}` é uma expressão; todo o resto de um campo é mantido
exatamente como foi escrito.

```text
{{api}}/users/{{user.id}}?trace={{uuid}}
Bearer {{secret.API_TOKEN}}
```

- Espaços dentro das chaves não importam: `{{ token }}` é `{{token}}`.
- `\{{` escreve um `{{` literal.
- Um `}}` sozinho é texto simples.
- Um valor é inserido como está, sem aspas. Em um corpo JSON, escreva as
  aspas você mesmo: `"id": "{{uuid}}"`.

### Nomes {#names}

| Expressão | Valor |
| --- | --- |
| `{{name}}` | a variável `name`, se uma está definida neste caminho, senão o parâmetro `name` |
| `{{vars.name}}` | só a variável |
| `{{params.name}}` | só o parâmetro |
| `{{secret.NAME}}` | o segredo guardado `NAME` — veja [segredos](#secrets) |
| `{{name.field}}` | um campo de um valor JSON |
| `{{name[0]}}` | um elemento de um array JSON |
| `{{name["a b"]}}`, `{{name['a b']}}` | um campo cujo nome tem outros caracteres |

Um nome de campo depois de `.` pode ter letras, dígitos, `_` e `-`. Os passos se encadeiam:
`{{reply.args[0]}}`, `{{order.items[2].sku}}`.

### Como os valores são escritos {#value-text}

| Valor | Escrito como |
| --- | --- |
| texto | o texto |
| número | a forma mais curta: `42`, `0.5` |
| `true`, `false` | `true`, `false` |
| `null` | `null` |
| objeto, array | JSON compacto: `["x","y"]` |

Um campo, depois de escritos os valores nele, tem no máximo 16 MiB; além disso a etapa
falha com `template.too_large` — um valor grande usado vezes e vezes não pode fazer um
sem limite.

### Valores embutidos {#built-ins}

| Expressão | Valor |
| --- | --- |
| `{{run.id}}` | o número da tarefa da execução; `0` na prévia e em [[ui:exp.sendNow]] |
| `{{run.seed}}` | a semente desta execução |
| `{{node.id}}` | o id do nó em execução |
| `{{now}}` | a hora atual, em milissegundos Unix |
| `{{now.iso}}` | a hora atual em UTC, ISO 8601 com milissegundos: `2026-09-30T12:34:56.789Z` |
| `{{counter}}` | quantas vezes este nó executou nesta execução, esta incluída, a partir de 1 |

`{{counter}}` conta por nó: no corpo de um [Laço](flow.md#loop) é o número da
iteração, em um nó que [repete](flow.md#repeat) é o número do envio.
`run`, `node` e `now` têm só os campos listados; qualquer outro é um erro.

### Geradores {#generators}

| Expressão | Valor |
| --- | --- |
| `{{uuid}}` ou `{{uuid()}}` | um UUID versão 4 |
| `{{random_int(min, max)}}` | um número inteiro de `min` a `max`, ambos incluídos; argumentos inteiros, `min` ≤ `max` |
| `{{random_float(min, max)}}` | um número de `min` até `max`, sem incluí-lo, com 3 casas decimais; `min` < `max` |
| `{{random_float(min, max, digits)}}` | o mesmo com `digits` casas decimais, 0–9 |
| `{{pick(a, b, c)}}` | um dos argumentos, pelo menos um |

Os argumentos são separados por vírgulas. Um entre aspas (`"dark blue"` ou `'a, b'`)
pode conter qualquer coisa, menos a própria aspa; um sem aspas pode conter letras,
dígitos e `_ - . : / +`. Um argumento vazio é um erro.

Todo gerador sorteia a partir da semente da execução. Os valores de uma execução de um
nó dependem só da semente, do id do nó e de quantas vezes o nó
executou, de modo que ramos paralelos nunca alteram os valores uns dos outros, e uma execução com
a mesma semente gera os mesmos valores de novo. Os sorteios dentro de um nó seguem
a ordem dos campos dele. `{{now}}` e `{{run.id}}` não são reproduzíveis. Veja
[sementes](runs.md#seeds).

### Sugestões {#suggestions}

Digitar `{{` em um campo com modelo, ou pressionar <kbd>Ctrl</kbd>+<kbd>Space</kbd>,
abre uma lista em quatro grupos: [[ui:exp.suggest.params]] com seus valores,
[[ui:exp.suggest.vars]] definidas antes deste nó com o nó que as define
(os campos de uma resposta também, como `reply.args[0]`), [[ui:exp.suggest.secrets]] e
[[ui:exp.suggest.generators]]. <kbd>↑</kbd> e <kbd>↓</kbd> escolhem,
<kbd>Enter</kbd> ou <kbd>Tab</kbd> inserem, <kbd>Esc</kbd> fecha a lista e
mantém o campo.

### Nomes desconhecidos são erros {#unknown-names}

Um nome sem valor nunca vira uma string vazia. Antes de uma execução, todo
nome que um campo usa precisa ser um parâmetro, um nome de segredo válido ou uma variável definida
em **todos** os caminhos que levam ao nó. O editor aponta o nó e
o campo:

| Problema | Antes da execução | Durante a execução |
| --- | --- | --- |
| Um nome que ninguém define | `name.unknown` | — |
| Uma variável definida só em alguns caminhos | `name.not_on_every_path` | — |
| `{{params.x}}` sem um parâmetro `x` | `param.unknown` | — |
| Um campo que um valor não tem | — | `template.no_field` |
| Um `{{` sem fechar, um `{{}}` vazio, um argumento malformado | `template.*`, com a posição | — |

Os textos desses códigos estão em [Erros](../reference/errors.md).

## Quais campos aceitam modelos {#templated-fields}

| Nó | Campos com modelo |
| --- | --- |
| [[ui:exp.node.http]] | URL, nomes e valores de cabeçalhos, corpo, o nome de usuário e a senha de Basic e Digest, o token Bearer |
| [[ui:exp.node.osc]] | destino, endereço, argumentos de texto; com uma resposta: o padrão de endereço e os valores das regras dela |
| [[ui:exp.node.udp]] | destino, carga útil; com uma resposta: o padrão dela |
| [[ui:exp.node.tcp]] | host, carga útil |
| [[ui:exp.node.mqtt]] | host do broker, tópico, carga útil |
| [[ui:exp.node.log]] | mensagem |
| [[ui:exp.node.assert_body]] | texto esperado |
| [[ui:exp.node.assert_header]] | nome do cabeçalho, texto esperado |
| [[ui:exp.node.assert_value]], [[ui:exp.node.branch_value]], a condição de saída de um [[ui:exp.node.loop]] | valor, valor esperado |
| [[ui:exp.node.wait_osc]] | padrão de endereço, valores das regras |
| [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_ws]] | padrão |
| [[ui:exp.node.wait_mqtt]] | broker e tópico (só parâmetros), padrão |
| [[ui:exp.node.wait_http]] | padrão de caminho, condições |
| [[ui:exp.node.impairment]] | escuta e destino (só parâmetros) |
| [[ui:exp.node.ws_connect]] | URL, nomes e valores de cabeçalhos |
| [[ui:exp.node.ws_send]] | carga útil |
| [[ui:exp.node.ws_close]] | motivo |

Números — portas, tempos limite, atrasos, status, números OSC tipados — e os
endereços de escuta das esperas são literais. Um [[ui:exp.node.emulator]] gera
as próprias respostas com o que chegou (`{{request.…}}`) e os parâmetros; veja
[falhas](faults.md#emulator).

**Só parâmetros.** Alguns campos são abertos antes da primeira etapa, quando nenhuma
variável existe ainda: o broker e o tópico de um [[ui:exp.node.wait_mqtt]], a escuta
e o destino de um [[ui:exp.node.impairment]]. Eles aceitam texto e parâmetros,
nada mais (`node.params_only`).

**Verificados como literais.** Um campo que usa só parâmetros é resolvido
antes da execução e verificado como o texto que a execução enviará: uma URL deve ser
`http://` ou `https://`, um destino OSC `IP:port` ou `host:port`, um nome de cabeçalho válido. Um campo
com variáveis ou geradores é verificado quando executa.

## Prévia {#preview}

Quando o nó selecionado tem um modelo, as propriedades dele mostram o que ele fará
com os valores conhecidos agora: [[ui:exp.preview]] para um envio,
[[ui:exp.previewWait]] para uma espera, [[ui:exp.previewCheck]] para uma comparação.
O motor a resolve, com o mesmo código que uma execução usa, de modo que a prévia nunca
discorda da execução.

- Os parâmetros vêm do perfil ativo.
- As variáveis vêm do que o editor viu nesta sessão: as etapas da última
  execução e [[ui:exp.sendNow]].
- Um segredo guardado aparece como `••••`.
- Um nome que ainda não tem valor fica como foi escrito, e a prévia o lista. Um
  segredo que não está guardado é listado à parte.
- Os geradores usam a semente fixada do experimento, ou `0` quando nenhuma está fixada, como
  a primeira execução do nó. Com uma semente fixada, a prévia mostra os valores
  gerados que a primeira execução do nó em uma execução enviará.

## Extração de valores {#extract}

[[ui:exp.node.extract]] lê um valor da última resposta HTTP no caminho dele
e o escreve em uma variável.

| Campo | O quê |
| --- | --- |
| [[ui:exp.variable]] | a variável a escrever; valem as regras de nome dos parâmetros |
| [[ui:exp.extractFrom]] | de onde vem o valor (abaixo) |
| [[ui:exp.jsonPath]], [[ui:exp.headerName]] ou [[ui:exp.pattern]] | o que ler, conforme a origem |

| [[ui:exp.extractFrom]] | Lê | Valor |
| --- | --- | --- |
| [[ui:exp.from.json]] | o corpo como JSON, em um caminho | o valor JSON: texto, número, objeto, array |
| [[ui:exp.from.header]] | o primeiro cabeçalho com esse nome, em qualquer caixa | texto |
| [[ui:exp.from.status]] | o código de status | um número |
| [[ui:exp.from.body]] | o corpo inteiro | texto |
| [[ui:exp.from.regex]] | a primeira correspondência no corpo | o grupo de captura 1 se o padrão tem um, senão a correspondência inteira |

**Caminhos JSON.** `$.token`, `$.items[0].id`, `$["a b"]`, `$['a b']['c-d']`;
o `$.` inicial pode ser omitido (`token`, `items[0].id`), e `$` sozinho é
o corpo inteiro.

**As expressões regulares** usam a sintaxe do motor `regex` do Rust, que não tem
look-around nem referências anteriores. A correspondência é procurada em qualquer ponto do
corpo; ancore-a com `^` e `$` quando isso importar.

A etapa falha, dizendo o que falta, quando:

- nenhuma requisição HTTP executou antes dela neste caminho (`check.no_response`; o editor
  já recusa um grafo em que nenhuma pode ter executado, `graph.needs_http`);
- o corpo não é JSON, ou o caminho não está nele;
- o cabeçalho não existe, ou o padrão não corresponde;
- o corpo passa dos 256 KiB que uma resposta guarda, para um caminho JSON ou o corpo
  inteiro, e para um padrão que não correspondeu a nada na parte guardada
  (`extract.truncated`).

A linha do tempo mostra o valor escrito: `token = abc123`.

::: tip Extrair com um clique
[[ui:exp.sendNow]] em um [[ui:exp.node.http]] mostra a resposta JSON dele. Clique em um
valor nela: um nó [[ui:exp.node.extract]] é adicionado depois da requisição, com
o caminho preenchido e um nome tirado da chave, e o valor fica conhecido da
prévia na hora.
:::

## Variáveis {#variables}

Uma variável guarda um valor JSON. Estes nós escrevem uma:

| Nó | Escreve | Na saída |
| --- | --- | --- |
| [[ui:exp.node.extract]] | o valor extraído | a saída dele |
| [[ui:exp.node.wait_osc]], [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_http]], [[ui:exp.node.wait_ws]] | o que chegou, nome padrão `reply` (`request` para HTTP) | só [[ui:exp.portMatched]] |
| [[ui:exp.node.osc]], [[ui:exp.node.udp]] com [[ui:exp.expectReply]] | a resposta, nome padrão `reply` | a saída dele |

O que uma espera escreve é um objeto; os campos seguintes leem as partes dele:

| Espera | Campos |
| --- | --- |
| OSC | `address`, `args`, `from`, `ms` |
| UDP | `text`, `hex`, `bytes`, `from`, `ms` e `match` com um padrão |
| MQTT | `topic` e os campos de UDP |
| WebSocket | os campos de UDP e `json` quando a mensagem é JSON |
| Requisição HTTP | `method`, `path`, `query`, `headers`, `body`, `json`, `params`, `from`, `ms` |

`ms` é o tempo da última ação do ramo até a chegada. O conteúdo exato
está na [referência dos nós](nodes.md).

### Onde uma variável é conhecida {#visibility}

Uma variável existe a partir da saída que a escreve, nos caminhos que
passam por essa saída:

- Depois de uma junção de caminhos alternativos — o [[ui:exp.yes]] e o [[ui:exp.no]]
  de um desvio que se encontram de novo — só é conhecido o que **todos** os caminhos definiram.
- Depois de [[ui:exp.node.join]], é conhecido o que **qualquer** ramo que chega a ele definiu: todos
  eles executaram.
- Depois de [[ui:exp.portDone]] ou [[ui:exp.portLimit]] de um [[ui:exp.node.loop]],
  e na condição de saída dele, é conhecido o que toda iteração do corpo define.
- A variável de uma espera não é conhecida depois da saída [[ui:exp.portTimeout]] dela.

Cada ramo paralelo trabalha com a própria cópia das variáveis. Uma união mescla as
cópias na ordem dos fios de entrada, e o fio posterior vence em um nome que ambos
definem, de modo que o resultado nunca depende de qual ramo terminou primeiro. Veja
[como uma execução se move](flow.md#parallel).

## Comparação de valores {#compare}

[[ui:exp.node.assert_value]] reprova a execução quando uma comparação não vale;
[[ui:exp.node.branch_value]] sai por [[ui:exp.yes]] ou [[ui:exp.no]];
um [[ui:exp.node.loop]] usa a mesma comparação como condição de saída. Cada um tem
um [[ui:exp.value]], um [[ui:exp.operator]] e um valor [[ui:exp.expected]], e
os dois textos são modelos:

| [[ui:exp.value]] | [[ui:exp.operator]] | [[ui:exp.expected]] |
| --- | --- | --- |
| `{{status}}` | [[ui:exp.op.lt]] | `300` |
| `{{reply.args[0]}}` | [[ui:exp.op.eq]] | `{{nonce}}` |

| [[ui:exp.operator]] | Vale quando |
| --- | --- |
| [[ui:exp.op.eq]], [[ui:exp.op.ne]] | os dois são iguais (diferentes) — como números quando ambos são números (`200` é igual a `200.0`), senão como texto exato, maiúsculas e minúsculas incluídas |
| [[ui:exp.op.lt]], [[ui:exp.op.le]], [[ui:exp.op.gt]], [[ui:exp.op.ge]] | como números; um lado que não é número reprova a etapa (`compare.not_numbers`) em vez de um *não* silencioso |
| [[ui:exp.op.contains]] | o valor contém o texto esperado, maiúsculas e minúsculas incluídas |
| [[ui:exp.op.matches]] | a expressão regular no valor esperado corresponde em qualquer ponto do valor |
| [[ui:exp.op.empty]], [[ui:exp.op.not_empty]] | o valor está vazio, ou não, depois de remover os espaços; o valor esperado não é usado |

Um número é um texto que se lê como tal depois de remover os espaços: `42`, `-1.5`,
`1e3`. A linha do tempo mostra a comparação como foi feita, `401 = 200`, cada lado cortado
em 120 caracteres.

## Segredos {#secrets}

Um token ou uma senha entra em um campo como `{{secret.NAME}}`. O arquivo do
experimento guarda só o nome; o valor fica onde está guardado e nunca
chega à interface.

### Onde os segredos ficam {#secret-stores}

| Onde o Signal Lab executa | Repositório | A partir da interface |
| --- | --- | --- |
| Aplicativo de desktop no Windows | o Gerenciador de Credenciais do Windows, sob o serviço `SignalLab`, uma entrada por nome | definir, substituir, remover |
| Aplicativo de desktop no Linux | nenhum: uma execução que precisa de um segredo falha com `secret.unsupported` | — |
| Servidor | a variável de ambiente `SIGNALLAB_SECRET_<NAME>`, senão o arquivo `<NAME>` na pasta de segredos dele, `/run/secrets/signallab` a menos que se defina outra | somente leitura |
| `signallab` na linha de comando | como um servidor, ou o Gerenciador de Credenciais do Windows com `--secrets system` | — |

O título da seção [[ui:exp.secrets]] tem uma dica que diz qual destes se aplica onde você
está: o repositório do Windows, o ambiente e os arquivos do servidor ou — no
aplicativo de desktop no Linux — que não há repositório. Ali,
`secret.unsupported` diz que os segredos são guardados no Gerenciador de Credenciais
do Windows, que o sistema não tem.

Um segredo pertence ao computador ou ao servidor, não a um experimento: dois
experimentos que usam `{{secret.API_TOKEN}}` usam o mesmo valor.

Em um servidor, a variável de ambiente vence o arquivo. A quebra de linha final de um arquivo
não faz parte do valor, e um arquivo vazio conta como nenhum segredo.
A pasta do servidor é definida com `--secrets-dir` ou `SIGNALLAB_SECRETS_DIR`;
veja [o servidor](../server/index.md). Para a linha de comando, veja
[`signallab run`](../automation/cli.md#cli-run).

| Regra | Limite |
| --- | --- |
| Nome | começa com uma letra ou `_`, depois letras, dígitos e `_`; no máximo 128 caracteres |
| Valor | não vazio, no máximo 16 KiB |

### Definição de um segredo {#set-secret}

No Windows:

1. Abra [[ui:exp.params]]. A seção [[ui:exp.secrets]] lista todos os segredos que
   os campos do experimento usam, cada um [[ui:exp.secretStored]] ou
   [[ui:exp.secretMissing]].
2. Pressione [[ui:exp.secretSet]] ao lado do nome, ou [[ui:exp.addSecret]] para um
   nome que nenhum campo usa ainda.
3. Digite o valor — o campo mostra pontos — e pressione [[ui:exp.secretSave]] ou
   <kbd>Enter</kbd>. O campo é limpo; nada pode ler o valor de volta.

[[ui:exp.secretReplace]] guarda um novo valor e [[ui:exp.secretRemove]]
o exclui do repositório de credenciais. Um nome guardado nesta sessão também é oferecido
nas sugestões.

Em um navegador conectado a um servidor, a seção só diz
[[ui:exp.secretOnServer]] ou [[ui:exp.secretNotOnServer]]: defina o valor onde
o servidor executa, de uma de duas maneiras:

```bash
# in the server's environment
SIGNALLAB_SECRET_API_TOKEN='…'
# or as a file in its secrets folder
printf '%s' '…' > /run/secrets/signallab/API_TOKEN
```

Um arquivo é lido a cada início de execução, então um arquivo alterado vale a partir da próxima
execução; uma variável de ambiente alterada exige reiniciar o servidor.

### Antes de uma execução {#secret-check}

Todo segredo que os campos da execução usam precisa estar guardado. Um que falta interrompe a execução
antes de qualquer tráfego, no primeiro nó e campo que o usam
(`secret.missing`). [[ui:exp.sendNow]] faz a mesma verificação para o nó dele.

### Mascaramento {#masking}

Enquanto uma execução ou um [[ui:exp.sendNow]] usa segredos, toda ocorrência dos valores
deles é substituída por `••••` em tudo o que sai do motor:

- textos de etapas, erros e as variáveis que uma etapa escreveu;
- o relatório da execução;
- o resultado de [[ui:exp.sendNow]], inclusive a resposta HTTP que ele mostra;
- os quadros do [[ui:dock.inspector]], capturados enquanto a execução dura — em um despejo hex,
  cada byte de um valor vira `*`, de modo que os deslocamentos continuam verdadeiros.

A autenticação Basic envia `name:password` em base64; quando uma das partes contém um
segredo, esse texto em base64 também é mascarado. O mesmo vale para as formas que uma requisição e um
servidor que a ecoa dão a um valor: codificado com porcentagem como em uma URL ou um
formulário, com o `+` de uma query lido de volta como espaço, e com escapes dentro de JSON.
O tráfego em si leva o valor real. A prévia mostra um segredo guardado como `••••`. As respostas de um
[[ui:exp.node.emulator]] não podem usar segredos.

## Comandos {#commands}

A prévia é [`experiment_resolve`](../api/commands.md#experiment_resolve);
os segredos são listados, definidos e removidos com
[`secret_status`](../api/commands.md#secret_status),
[`secret_set`](../api/commands.md#secret_set) e
[`secret_delete`](../api/commands.md#secret_delete). Nenhum comando devolve o valor
de um segredo.

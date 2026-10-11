---
title: Linha de comando
description: signallab executa experimentos sem janela, envia mensagens avulsas, faz o papel de emuladores e verifica a rede, a partir de um terminal, de um script ou de um job de CI.
---

# A linha de comando: `signallab`

`signallab` é o Signal Lab sem janela. Ele executa experimentos até o fim e
sai com um código que um script entende, envia uma mensagem OSC, um datagrama, uma requisição
HTTP, uma mensagem WebSocket ou uma publicação MQTT, dispara um sinal da sua biblioteca,
faz o papel de um emulador até você parar e diz o que há entre esta máquina
e o equipamento.

É o mesmo motor do app: uma execução pela linha de comando segue as mesmas
etapas, escreve o mesmo relatório e diz as mesmas coisas, nos idiomas
da interface. Com `--server`, as execuções acontecem em um [servidor do Signal Lab](../server/index.md),
com a rede e os segredos dele.

```bash
signallab run tests/smoke.json --param api=http://127.0.0.1:8080 --junit junit.xml
signallab run flaky-api                                # um modelo embutido, pelo nome
signallab validate tests/*.json                        # a verificação do editor, nada é enviado
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab fire "Fader value"
signallab emulate tests/orders-api.json --for 120      # faz o papel de uma dependência por dois minutos
signallab doctor                                       # firewall, rede, pasta de dados
```

Para pipelines, veja [Signal Lab em CI](ci.md); para um assistente de IA,
[`signallab mcp`](mcp.md).

## Instalação {#install}

| Onde | Como obter `signallab` |
| --- | --- |
| Windows, o instalador (`.exe`) | Instalado ao lado do app, e essa pasta é adicionada ao `PATH` — o seu, em uma instalação "para mim", o da máquina em "para todos". Abra um novo terminal depois de instalar. A opção `/NOPATH` do instalador deixa o `PATH` em paz. |
| Windows, o `.msi` | Instalado ao lado do app; a pasta de instalação fica no `PATH` da máquina enquanto o app estiver instalado. |
| Linux, `.deb` e `.rpm` | `/usr/bin/signallab`. |
| Linux, AppImage | Não incluído: use o arquivo compactado abaixo. |
| Qualquer máquina, sem o app | Cada versão traz `signallab-<version>-windows-x64.zip` e `signallab-<version>-linux-x64.tar.gz`, cada um com o programa e a licença dele; `SHA256SUMS.txt` na mesma página lista as somas de verificação deles. |
| A imagem do servidor | `/usr/local/bin/signallab` em `ghcr.io/proanima/signallab` (veja [CI](ci.md#docker)). |

Confira com:

```bash
signallab version
```

## Comandos {#commands}

| Comando | O que faz |
| --- | --- |
| [`run`](#cli-run) | Executa experimentos um após o outro; sai com 0 só quando todos foram aprovados. |
| [`validate`](#cli-validate) | Verifica experimentos como o editor faz antes de uma execução; não envia nada. |
| [`send`](#cli-send) | Envia uma mensagem: `osc`, `udp`, `http`, `ws` ou `mqtt`. |
| [`fire`](#cli-fire) | Envia um sinal de uma biblioteca de sinais, pelo id ou pelo nome. |
| [`emulate`](#cli-emulate) | Faz o papel de uma API HTTP, de um dispositivo OSC, UDP ou TCP, ou de um broker MQTT até <kbd>Ctrl</kbd>+<kbd>C</kbd> ou `--for`. |
| [`emulators`](#cli-emulators) | Lista os emuladores da biblioteca do app. |
| [`templates`](#cli-templates) | Lista os modelos de experimento embutidos. |
| [`nodes`](#cli-nodes) | Descreve todos os tipos de nó, em JSON. |
| [`mcp`](#cli-mcp) | Serve o Signal Lab a um assistente de IA pelo Model Context Protocol. |
| [`doctor`](#cli-doctor) | Verifica o firewall, a rede, a pasta de dados e um servidor. |
| [`firewall`](#cli-firewall) | `firewall allow`: deixa outras máquinas alcançarem o Signal Lab pelo Firewall do Windows. |
| [`version`](#cli-version) | Imprime a versão. |

`signallab help <command>` ou `signallab <command> --help` imprime as opções de um
comando.

## Opções de todos os comandos {#global-options}

| Opção | O que faz | Padrão |
| --- | --- | --- |
| `--lang <code>` | O idioma das mensagens: `en`, `ru`, `es`, `fr`, `de`, `pt`, `zh`, `ja`, `ko`, `hi` ou `ar`. | `SIGNALLAB_LANG`, senão o idioma do sistema, senão `en` |
| `--json` | Saída para máquinas em stdout em vez de texto (veja [Saída](#output)). | desligado |
| `-h`, `--help` | A ajuda do comando. | |
| `-V`, `--version` | A versão; antes de qualquer comando, como `signallab --version`. | |

`--lang` e `--json` podem ir em qualquer lugar da linha:
`signallab --json run smoke.json` e `signallab run smoke.json --json` são a mesma coisa.

### Idioma {#language}

As mensagens, os textos de etapas, as falhas e o relatório JUnit usam os textos,
as formas de plural e o estilo de números da própria interface. O idioma é o primeiro destes:

1. `--lang`;
2. `SIGNALLAB_LANG` (`ru`, `ru-RU` e `ru_RU.UTF-8` significam todos russo);
3. o idioma do sistema: a primeira de `LC_ALL`, `LC_MESSAGES` e `LANG` que estiver definida;
4. inglês.

::: tip
Os terminais do Windows em geral não definem nenhuma das variáveis de idioma, então `signallab`
fala inglês ali, a menos que você defina `SIGNALLAB_LANG` ou passe `--lang`.
:::

### Saída para pessoas e para máquinas {#output}

Sem `--json`, os resultados vão para **stdout** e tudo o que uma pessoa lê ao longo
do caminho vai para **stderr**: as etapas de uma execução conforme acontecem, por que algo
falhou, onde está o relatório. `signallab run … 2>/dev/null` deixa uma linha de veredicto
por execução.

Com `--json`, stdout carrega JSON e nada mais, e stderr fica quieto:

| Comando | O que `--json` imprime em stdout |
| --- | --- |
| `run` | Um objeto por linha: `started`, um `step` por etapa, `ended` (o resultado da execução) e depois `summary`; `error` para uma execução que não pôde começar. Veja [O que `run` imprime](#run-output). |
| `validate` | Um objeto por linha, por experimento: `valid`, e `profile_issues` ou `error`. |
| `send`, `fire` | `{"type": "sent", "result": …}`; `send http` imprime `{"type": "response", "response": …}`, `send ws` `{"type": "exchange", "result": …}`. |
| `emulate` | Um objeto por linha: `started`, um `exchange` por requisição, um `summary` por emulador; `valid` com `--check`. |
| `emulators` | `{"path": …, "emulators": [{id, name, protocol, bind, rules, note}, …]}`. |
| `templates` | `[{"name": …, "experiment": …}, …]`. |
| `doctor` | Um objeto: `version`, `network`, `data_dir`, `firewall`, `server`, `problems`. |
| `version` | `{"version": "…"}`. |
| `nodes` | Sempre JSON, com ou sem `--json`. |

Uma falha é `{"type": "error", "error": {…}, "exit_code": N}`. `error` é o erro do
motor: um `code` estável (como `transport.refused` ou
`secret.missing`), os `params` dele e o `node` e o `field` a que se refere. Um
script pode decidir por `error.code` em qualquer idioma; os textos de cada código estão
listados em [Mensagens de erro](../reference/errors.md).

### Códigos de saída {#exit-codes}

| Código | Significado |
| --- | --- |
| `0` | Todos os experimentos foram aprovados; o envio teve êxito; nada está no caminho. |
| `1` | Um experimento executou e falhou, ficou sem tempo ou foi parado; um envio falhou (recusado, sem resposta, um status inesperado); `doctor` achou algo no caminho. |
| `2` | O comando ou um documento está errado: um argumento, um arquivo que não pode ser lido, um erro de validação, um parâmetro desconhecido, um segredo ausente. |
| `3` | Nada pôde executar por um motivo fora do experimento: o servidor não pode ser alcançado ou recusa o token, uma porta não pode ser aberta, o repositório de credenciais falha. |

Com vários experimentos, o resultado mais grave decide, nesta ordem: `2`,
depois `3`, depois `1`, depois `0`.

### Variáveis de ambiente {#environment}

| Variável | O que faz |
| --- | --- |
| `SIGNALLAB_LANG` | O idioma, quando `--lang` não é dado. |
| `LC_ALL`, `LC_MESSAGES`, `LANG` | O idioma, quando nenhuma das anteriores está definida. |
| `SIGNALLAB_SERVER` | O servidor para `run`, `validate`, `emulate`, `mcp` e `doctor`, como `--server` o dá. |
| `SIGNALLAB_TOKEN` | O token de acesso do servidor, quando nenhum arquivo de token é dado. |
| `SIGNALLAB_TOKEN_FILE` | Um arquivo com o token do servidor, como `--token-file` o dá. |
| `SIGNALLAB_SECRET_<NAME>` | O valor do segredo `NAME` para as execuções neste processo (veja [Segredos](#secrets)). |
| `SIGNALLAB_DATA_DIR` | A pasta de dados do app, onde `fire`, `emulators`, `emulate`, `mcp` e `doctor` procuram por padrão; senão `Documents/SignalLab` na sua pasta pessoal. |
| `GITHUB_ACTIONS` | Quando é `true`, uma execução que não é aprovada também é impressa como uma anotação `::error`, que o GitHub mostra na página da execução. |

## `run` {#cli-run}

```text
signallab run [OPTIONS] <FILE>...
```

Executa experimentos um após o outro e sai com `0` só quando todos foram aprovados.

| Opção | O que faz | Padrão |
| --- | --- | --- |
| `<FILE>...` | Arquivos de experimento, ou nomes de [modelos embutidos](#cli-templates). | obrigatório |
| `-p`, `--param NAME=VALUE` | Um valor de parâmetro para esta execução; repita para mais. | os valores do documento |
| `--profile NAME` | Executa com este perfil; todo experimento dado precisa tê-lo. `""` executa com os padrões. | o perfil do documento |
| `-m`, `--matrix NAME=V1,V2` | Executa uma vez por valor; repita para mais nomes (veja [Uma matriz de execuções](#matrix)). | |
| `--matrix-file PATH` | Combinações de um arquivo JSON. | |
| `--seed N` | A semente dos valores aleatórios, de 0 a 9007199254740991 (2⁵³ − 1). | a semente do documento, senão uma nova por execução |
| `--timeout SECONDS` | Reprova uma execução que demora mais, 1–300. | `300` |
| `--fail-fast` | Para na primeira execução que não é aprovada; as demais não são iniciadas. | desligado |
| `--junit PATH` | Escreve ali um relatório JUnit XML (veja [Relatórios](#reports)). | |
| `--report PATH` | Copia ali o relatório da execução: um arquivo para uma execução, uma pasta para várias. | |
| `--data-dir PATH` | A pasta de dados para as execuções deste processo; os relatórios delas ficam ali. | uma pasta temporária, removida ao sair |
| `--server URL` | Executa neste servidor em vez de neste processo (veja [Em um servidor](#run-on-server)). | `SIGNALLAB_SERVER` |
| `--token-file PATH` | Um arquivo com o token do servidor. | `SIGNALLAB_TOKEN_FILE`, senão `SIGNALLAB_TOKEN` |
| `--secrets files\|system` | De onde vêm os valores de segredos neste processo. | `files` |
| `--secrets-dir PATH` | Uma pasta de arquivos de segredo, um por nome. | `/run/secrets/signallab` quando existe |

`--data-dir`, `--secrets` e `--secrets-dir` dizem respeito às execuções neste processo;
não podem ser combinados com `--server`.

### Arquivos e modelos {#run-inputs}

Um `FILE` é um experimento como o app o salva e exporta
([[ui:exp.exportJson]] na tela [[ui:nav.experiment]]). Funciona qualquer versão de
documento que o app abra; uma mais antiga é atualizada ao ser lida,
como o app faz ao abri-la. Um nome que não é um arquivo é procurado entre
os [modelos embutidos](#cli-templates), com ou sem `.json`:

```bash
signallab run tests/stage-cues.json tests/api.json
signallab run osc-ping-reply --param device=192.0.2.20:9000
```

Todo experimento — e toda combinação de uma matriz — é verificado como o editor
verifica, antes de o primeiro executar. Um terceiro arquivo quebrado impede também o primeiro,
antes de qualquer envio, com o código de saída `2`.

### Parâmetros e perfis {#run-params}

`--param NAME=VALUE` define um parâmetro só para esta execução; o arquivo não é
alterado. O valor é tudo depois do primeiro `=`, então
`--param url=http://127.0.0.1/?q=1` funciona, e `--param note=` define um valor
vazio. Um valor vale para todo experimento dado que tenha esse parâmetro, e
precisa nomear um parâmetro de pelo menos um deles — um nome com erro de grafia é recusado
com o código de saída `2`.

`--profile NAME` executa com um dos perfis do experimento, como escolhê-lo
em [[ui:exp.profile]] no editor faz. Veja
[Dados e modelos](../experiments/data.md).

### Uma matriz de execuções {#matrix}

Uma matriz executa o mesmo experimento contra todos os destinos, todos os usuários, todos os
tamanhos de carga útil. Cada combinação é uma execução à parte, com resultado próprio,
relatório próprio e suíte própria no relatório JUnit.

```bash
signallab run smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest \
  --fail-fast --junit junit.xml
```

São quatro execuções: `device` varia mais devagar, `user` mais depressa. Elas têm o nome
do arquivo e dos valores: `smoke.json [device=192.0.2.20:9000, user=admin]`.

- `--matrix NAME=V1,V2` adiciona um eixo. O mesmo nome de novo soma valores a ele.
  Os espaços em volta de nomes e valores são descartados; um valor dado duas vezes executa uma vez.
- `--matrix-file PATH` lê JSON em uma de duas formas:

  ```json
  { "device": ["192.0.2.20:9000", "192.0.2.21:9000"], "retries": [1, 3] }
  ```

  adiciona eixos — toda combinação executa, esses nomes depois dos de `--matrix`, em
  ordem alfabética;

  ```json
  [
    { "device": "192.0.2.20:9000", "user": "admin" },
    { "device": "192.0.2.21:9000", "user": "guest" }
  ]
  ```

  lista as próprias combinações, cada uma cruzada com os eixos de `--matrix`.
  Os valores são texto, números ou `true`/`false`; uma vírgula dentro de um valor em um arquivo
  continua fazendo parte dele.
- Todo nome da matriz precisa ser um parâmetro de pelo menos um experimento dado. Um
  experimento sem um deles executa uma vez, não uma vez por valor que
  ignoraria.
- Um nome definido por `--param` e pela matriz, ou por `--matrix` e pelo
  arquivo, é recusado.
- No máximo **256** execuções por comando; mais que isso é recusado antes de qualquer coisa
  executar.

### Execução em um servidor {#run-on-server}

```bash
signallab run tests/stage.json --server http://192.0.2.10:1430 --token-file token.txt
```

O experimento vem desta máquina; o servidor o executa com a rede dele, os
[segredos](../server/index.md#secrets) dele e a pasta de dados dele,
e transmite as etapas de volta conforme acontecem. O relatório fica no servidor —
o caminho dele é impresso — e `--report` baixa uma cópia. Uma execução em um servidor é uma
tarefa ali como uma iniciada na interface dele: toda página conectada a ele
a mostra. Se `signallab` some no meio da execução, ela ainda termina no servidor e
guarda o relatório.

O token é lido de `--token-file` (ou `SIGNALLAB_TOKEN_FILE`), senão de
`SIGNALLAB_TOKEN`, e enviado como `Authorization: Bearer`. Um servidor que não pode ser
alcançado, recusa o token ou para de responder no meio da execução é o código de saída `3`.
`signallab doctor --server URL` verifica o endereço e o token por si sós.

### Relatórios {#reports}

Cada execução escreve o mesmo relatório que o app escreve. Sem `--data-dir`, as execuções
usam uma pasta temporária que é removida quando `signallab` sai, então guarde o que
precisar:

- `--report PATH` copia o relatório: para o próprio `PATH` no caso de uma execução, ou para a
  pasta `PATH` no caso de várias, como `01-<file>.json`, `02-<file>.json`, … na
  ordem em que executaram.
- `--data-dir PATH` guarda o relatório de cada execução nessa pasta (em `runs/`)
  e imprime onde está cada um.

`--junit PATH` escreve um relatório JUnit XML, o formato que todo sistema de CI lê:

- uma `<testsuite>` por execução (por combinação de uma matriz), com o arquivo, a
  semente, o resultado, o perfil, o caminho do relatório e cada valor da matriz
  (`param.NAME`) como propriedades;
- um `<testcase>` por nó que executou, com o nome do tipo do nó e o id dele,
  com o tempo próprio;
- uma `<failure>` no nó que falhou: a mensagem no idioma escolhido,
  o código de erro como `type` e as etapas do nó com o detalhe técnico;
- um caso `<skipped>` para cada nó que a execução nunca alcançou (o outro lado de um
  desvio);
- uma suíte com um caso `<error>` para um experimento que não pôde começar, e
  uma com um caso `<skipped>` para cada execução que `--fail-fast` não iniciou.

```xml
<testsuites name="Signal Lab" tests="6" failures="1" errors="0" time="2.006">
  <testsuite name="HTTP to OSC" tests="6" failures="1" errors="0" skipped="4" time="2.006" timestamp="2026-10-04T16:48:03">
    <properties>
      <property name="file" value="status-branch" />
      <property name="seed" value="42" />
      <property name="outcome" value="failed" />
      <property name="report" value="out/report.json" />
    </properties>
    <testcase name="Start (start)" classname="HTTP to OSC" time="0.001">
      <system-out>Started · seed 42</system-out>
    </testcase>
    <testcase name="HTTP request (request)" classname="HTTP to OSC" time="2.004">
      <failure message="http://127.0.0.1:8080/ refused the connection — nothing is listening on that port" type="transport.refused">…</failure>
    </testcase>
    <testcase name="Status branch (branch)" classname="HTTP to OSC" time="0.000">
      <skipped message="not reached in this run" />
    </testcase>
    …
  </testsuite>
</testsuites>
```

Um nó HTTP sob [carga](../experiments/load.md) acrescenta uma linha sob a última
etapa dele para cada limiar, cumprido ou não (`✕ p95 < 100 ms · 152.58 ms`); um
limiar que não é cumprido reprova a execução e o caso JUnit dela (`type="load.threshold"`).

### Segredos {#secrets}

Um experimento lê um segredo como `{{secret.NAME}}`. Para as execuções neste processo, o
valor vem de:

| `--secrets` | De onde vem o valor de `NAME` |
| --- | --- |
| `files` (o padrão) | A variável de ambiente `SIGNALLAB_SECRET_NAME`; senão um arquivo chamado `NAME` em `--secrets-dir` (por padrão `/run/secrets/signallab`, quando essa pasta existe — o layout dos segredos do Docker). |
| `system` | O Gerenciador de Credenciais do Windows — onde o app guarda os valores dos seus [[ui:exp.secrets]]. O Linux não tem um repositório de credenciais que `signallab` leia: ali `--secrets system` falha com o código de saída `3`. |

A quebra de linha final de um arquivo não faz parte do valor, e um valor vazio
conta como não definido. Os nomes são letras, dígitos e `_`, sem começar por
dígito, até 128 caracteres; um valor tem no máximo 16 KiB.

```bash
SIGNALLAB_SECRET_API_TOKEN="$API_TOKEN" signallab run tests/api.json
```

Um segredo que não está definido interrompe a execução antes de qualquer tráfego, com o código de saída `2`
e o nome que falta. Um valor nunca é impresso: etapas, erros, relatórios
e o relatório JUnit mostram `••••` no lugar dele. Com `--server`, os segredos são
os do servidor.

### O que `run` imprime {#run-output}

Conforme uma execução avança, cada etapa é uma linha em stderr — o tempo desde o início,
o nó, o estado dele e o que fez — como a linha do tempo do app. Quando ela termina, uma
linha em stdout diz como foi:

```text
▶ HTTP check (http-check) · seed 1185927457137919
     0.000  Start         Running
     0.000  Start         Passed · Started · seed 1185927457137919
     0.000  HTTP request  Running
     2.004  HTTP request  Failed · URL — http://127.0.0.1:8080/ refused the connection — nothing is listening on that port
✖ HTTP check failed after 2 s: HTTP request · URL — http://127.0.0.1:8080/ refused the connection — nothing is listening on that port
  Technical details: error sending request for url (http://127.0.0.1:8080/): … (os error 10061)
  To run it again with the same random values: --seed 1185927457137919
```

Uma execução com falha nomeia a semente que usou: `--seed` com esse número a executa de novo
com os mesmos valores aleatórios. Depois do veredicto vem, em stderr, o que cada
emulador da execução recebeu — requisições, quantas nenhuma regra tomou, quantas
falharam e os acertos por regra:

```text
✔ Retry a flaky API passed in 7 ms
  Flaky API: 3 requests, 0 without a rule, 0 failed · #1 3
```

e o que cada retransmissor de degradação fez: o que recebeu, descartou e limitou,
e cada fase como encaminhado / recebido:

```text
  127.0.0.1:19110 → 127.0.0.1:19100: 155 datagrams, 38 dropped, 0 throttled · lan 0.0–2.0 s 39/39, wifi 2.0–4.0 s 39/39, offline 4.0–6.0 s 0/38, lan 6.0–8.0 s 38/38
```

Um retransmissor sobre TCP move trechos de fluxos, não datagramas, e não descarta nada, de modo que
a linha dele conta trechos, conexões, redefinições, conexões semiabertas e as
vezes em que um fluxo foi retido pelo limite de largura de banda:

```text
  127.0.0.1:19120 → 127.0.0.1:19101: 14 chunks, 2 connections, 1 reset, 0 half-open, 3 held back
```

Um nó HTTP sob [carga](../experiments/load.md) informa o progresso dele como uma
etapa no máximo uma vez por segundo.

Com várias execuções, uma linha por execução e uma contagem encerram a saída:
`3 runs: 2 passed, 1 failed`. Com `--fail-fast`, as execuções que ele não iniciou são
contadas em stderr.

Com `--json`, toda linha é um objeto com um `type`:

```json
{"type":"started","experiment":"Empty experiment","file":"empty","job_id":1,"overridden":false,"profile":null,"seed":1,"started_ms":1791132204585}
{"type":"step","node_id":"start","state":"passed","detail":"Started","message_key":"exp.step.started","message_params":{"seed":1},"job_id":1,"ts":1791132204585}
{"type":"ended","experiment":"Empty experiment","file":"empty","outcome":"passed","seed":1,"params":{},"steps":[…],"report_path":"…","started_ms":1791132204585,"ended_ms":1791132204585,…}
{"type":"summary","total":1,"passed":1,"failed":0,"not_started":0,"exit_code":0}
```

As linhas `started`, `ended` e `error` carregam `file` (o argumento como foi dado) e,
em uma matriz, `matrix` (os valores da combinação). `ended` é o resultado inteiro da
execução: `outcome` (`passed`, `failed` ou `stopped`), `seed`, `profile`,
`params`, `error`, todas as etapas, `emulators` e `impairments` quando a execução os
teve, e `report_path`. A última etapa de uma carga carrega `load` com todos os
números que ela mediu: `planned`, `sent`, `ok`, `failed`, `missed`, `rps`,
`error_rate`, `min_ms`, `mean_ms`, `max_ms`, `p50_ms` até `p99_ms`, `statuses`,
cada segundo (`seconds`), o `histogram` e o veredicto de cada limiar
(`thresholds`).

## `validate` {#cli-validate}

```text
signallab validate [OPTIONS] <FILE>...
```

Verifica experimentos como o editor faz antes de uma execução — o grafo, todos os
campos, modelos, parâmetros e segredos — e não envia nada. Sai com `0` quando
todos iniciariam.

Aceita as entradas de [`run`](#cli-run): arquivos e modelos, `--param`,
`--profile`, `--matrix`, `--matrix-file`, e `--server`, `--token-file`,
`--secrets`, `--secrets-dir`. Com `--server`, o servidor os verifica, contra
os segredos dele.

```text
✔ tests/stage.json: Stage cues would run
  Rehearsal: Would not run: …
```

Um problema que só outro perfil do documento tem é listado abaixo dele,
mas não reprova a verificação. Com `--json`, uma linha por experimento (por
combinação): `{"experiment", "file", "valid": true, "profile_issues": […]}`,
ou `"valid": false` com `error` e `exit_code`.

## `send` {#cli-send}

```text
signallab send <osc|udp|http|ws|mqtt> …
```

Envia uma mensagem pelos mesmos comandos que as telas do app usam, de modo que são
os mesmos bytes no fio. Um envio não lê biblioteca nem segredos.

Códigos de saída: `0` enviado, `1` o envio falhou, `2` um argumento está errado.

Um `<host:port>` é um endereço IP ou um nome de host, e uma porta: `127.0.0.1:9000`,
`[::1]:9000` (um endereço IPv6 entre colchetes) ou `device.local:9000`. Um nome é
consultado quando o comando executa, e o endereço IPv4 dele é usado quando há um, de modo que
`localhost:9000` alcança um receptor em `127.0.0.1`. Um nome que não
resolve reprova o envio (`1`); um destino sem porta é um argumento inválido
(`2`).

### `send osc` {#cli-send-osc}

```text
signallab send osc <host:port> <address> [ARG]...
```

Uma mensagem OSC. Os argumentos são tipados com um prefixo, ou inferidos:

| Argumento | Tipo OSC |
| --- | --- |
| `i:3` | int32 |
| `f:0.5` | float32 |
| `d:1.5` | float64 (double) |
| `h:64` | int64 |
| `s:text` | string |
| `b:de ad be ef` | blob, como bytes em hex |
| `T`, `F` | true, false |
| `N` | nil |
| `3`, `-3` | um inteiro simples é int32 |
| `3000000000` | um inteiro além de 32 bits é int64 |
| `2.5`, `1e3` | um decimal simples, ou um número com expoente, é float32 (float64 além do alcance do float32) |
| qualquer outra coisa | string |

```bash
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main 3
# ✔ sent /cue/go (32 bytes) → 127.0.0.1:9000
```

Ponha entre aspas um argumento com espaços: `"s:hello world"`. `s:7` envia o texto `7`. O
endereço precisa começar com `/`; um que não começa é um argumento inválido (`2`).

::: warning Git Bash no Windows
O Git Bash reescreve argumentos que começam com `/` em caminhos do Windows, então
`/cue/go` chega como `C:/Program Files/Git/cue/go`. Escreva `//cue/go`, ou execute
com `MSYS_NO_PATHCONV=1`. PowerShell e `cmd` não são afetados.
:::

Veja [OSC](../protocols/osc.md).

### `send udp` {#cli-send-udp}

```text
signallab send udp <host:port> (--text TEXT | --hex HEX)
```

Um datagrama UDP. Dê a carga útil como `--text`, ou como bytes em `--hex`:
`"de ad be ef"`, `deadbeef` ou `0xDE,0xAD`.

```bash
signallab send udp 127.0.0.1:7000 --text "PLAY 1"
signallab send udp 127.0.0.1:7000 --hex "de ad be ef"
```

### `send http` {#cli-send-http}

```text
signallab send http <METHOD> <URL> [OPTIONS]
```

Uma requisição HTTP. A linha de status vai para stderr — `HTTP 200 OK · 3 ms · 1,234 B`
— e o corpo da resposta para stdout, de modo que pode ser encadeado adiante. Um corpo maior que
256 KiB é cortado nesse ponto, e stderr avisa.

| Opção | O que faz | Padrão |
| --- | --- | --- |
| `-H`, `--header "Name: value"` | Um cabeçalho da requisição; repita para mais. | |
| `--body TEXT` | O corpo da requisição. `@FILE` envia o conteúdo de um arquivo de texto. | nenhum |
| `--expect-status STATUS` | Sai com `1` a menos que a resposta tenha este status. | qualquer status é `0` |
| `--timeout MS` | Milissegundos a aguardar a resposta. | `10000` |
| `-u`, `--user NAME:PASSWORD` | Credenciais, enviadas como Basic. | |
| `--digest` | Com `--user`: responde ao desafio Digest do servidor em vez disso (MD5 ou SHA-256). | desligado |
| `--bearer TOKEN` | Envia `Authorization: Bearer TOKEN`. Não com `--user`. | |

```bash
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab send http POST http://127.0.0.1:8080/cue -H "Content-Type: application/json" --body '{"cue": 1}'
signallab send http GET http://127.0.0.1:8080/admin -u admin:secret --digest
```

Sem `--expect-status`, qualquer resposta é um sucesso, um `500` incluído. Uma
requisição que não obtém resposta (recusada, tempo esgotado, um nome que não
resolve) é o código de saída `1`, e também é um desafio Digest que não pôde ser
respondido — o motivo é impresso depois da resposta.

::: tip
Os argumentos são visíveis para outros usuários da mesma máquina. Guarde as senhas reais
para os experimentos, onde são [segredos](#secrets).
:::

Veja [HTTP](../protocols/http.md).

### `send ws` {#cli-send-ws}

```text
signallab send ws <URL> [OPTIONS]
```

Uma troca WebSocket: conecta a `ws://…` ou `wss://…`, envia uma mensagem,
opcionalmente aguarda a resposta, fecha. O handshake e o que foi enviado vão para
stderr, a resposta para stdout (mensagens binárias em hex).

| Opção | O que faz | Padrão |
| --- | --- | --- |
| `--text TEXT` | Envia esta mensagem de texto. | nada enviado |
| `--hex HEX` | Envia estes bytes como mensagem binária. Não com `--text`. | |
| `-H`, `--header "Name: value"` | Um cabeçalho da requisição de upgrade; repita para mais. | |
| `--protocol NAME` | Um subprotocolo a oferecer; repita para mais, em ordem de preferência. | |
| `--expect TEXT` | Aguarda uma mensagem que contenha este texto. | |
| `--expect-regex REGEX` | Aguarda uma mensagem que corresponda a esta expressão regular. | |
| `--wait` | Aguarda qualquer mensagem. | |
| `--timeout MS` | Milissegundos a aguardar a resposta. | `2000` |

```bash
signallab send ws ws://127.0.0.1:9001/echo --text '{"ping": 1}' --expect '"ping"'
signallab send ws ws://127.0.0.1:9001/feed --wait        # nada enviado: a primeira mensagem do servidor
```

Sem `--expect`, `--expect-regex` nem `--wait`, ele conecta, envia e fecha
sem aguardar. Quando a resposta esperada não chega a tempo, o código de saída
é `1`. Veja [WebSocket](../protocols/websocket.md).

### `send mqtt` {#cli-send-mqtt}

```text
signallab send mqtt <host:port> <topic> [payload] [--qos 0|1|2] [--retain]
```

Uma publicação MQTT 3.1.1 sobre TCP simples, sem credenciais, como cliente
próprio com um id de cliente novo, de modo que nunca derruba uma conexão que já
esteja lá. Sem porta, o broker está em `1883`. Um tópico é um tópico só: com um `+`
ou `#` nele, ou vazio, o comando é recusado antes de conectar (`2`).

| Opção | O que faz | Padrão |
| --- | --- | --- |
| `[payload]` | A carga útil. | vazia |
| `--qos 0\|1\|2` | A qualidade de serviço. | `0` |
| `--retain` | Guarda como o valor retido do tópico. Uma carga útil vazia com `--retain` o limpa. | desligado |

```bash
signallab send mqtt 127.0.0.1:1883 lab/light/1/set on --qos 1
signallab send mqtt 127.0.0.1 lab/light/1/state "" --retain     # limpa o valor retido
```

Veja [MQTT](../protocols/mqtt.md).

## `fire` {#cli-fire}

```text
signallab fire <signal> [--library PATH]
```

Envia um sinal de uma biblioteca de sinais exatamente como a tela [[ui:nav.signals]] do app
o dispara: sinais OSC, UDP, HTTP e MQTT. O sinal é encontrado pelo
id, senão pelo nome, sem diferenciar maiúsculas de minúsculas; um nome que vários sinais compartilham é recusado
com os ids deles.

| Opção | O que faz | Padrão |
| --- | --- | --- |
| `<signal>` | O id ou o nome do sinal. | obrigatório |
| `--library PATH` | O arquivo da biblioteca. | `signals.json` na pasta de dados do app |

```bash
signallab fire "Fader value"
signallab fire go --library show/signals.json
```

A biblioteca só é lida, nunca criada nem alterada. Veja
[Sinais](../tools/signals.md).

## `emulate` {#cli-emulate}

```text
signallab emulate [OPTIONS] <FILE|NAME>...
```

Faz o papel do outro lado — uma API HTTP, um dispositivo OSC, UDP ou TCP, um broker MQTT —
até <kbd>Ctrl</kbd>+<kbd>C</kbd> ou `--for`, e imprime cada requisição conforme é
respondida. Um `FILE` contém um emulador, uma lista deles ou uma biblioteca inteira como o
app a escreve; um `NAME` é o id ou o nome de um da biblioteca do app (a da
tela [[ui:nav.emulators]]).

| Opção | O que faz | Padrão |
| --- | --- | --- |
| `-p`, `--param NAME=VALUE` | Um valor que os modelos dele leem como `{{NAME}}`; repita para mais. | |
| `--bind IP:PORT` | Escuta ali em vez disso. Só com um emulador. | o próprio do emulador |
| `--for SECONDS` | Para depois deste tempo. | até <kbd>Ctrl</kbd>+<kbd>C</kbd> |
| `--seed N` | A semente das escolhas aleatórias dele: uma ordem aleatória, jitter, geradores. | |
| `--check` | Verifica os emuladores e sai, sem abrir uma porta. | desligado |
| `--library PATH` | A biblioteca em que os nomes são procurados. | `emulators.json` na pasta de dados do app |
| `--server URL`, `--token-file PATH` | Inicia-os em um servidor, acompanha-os pela API dele e os para no fim. | |
| `--secrets`, `--secrets-dir` | Como em [`run`](#cli-run). | |

```text
$ signallab emulate tests/orders-api.json --for 60
Orders API (http) answering on 127.0.0.1:18099
answering for 60 s
+   1.209 s Orders API  #1  GET /orders/42 → 200 OK · 12 B  1 ms  ← 127.0.0.1:55744
+   1.209 s Orders API  —  GET /nothing → 404 Not Found · 20 B  2 ms  ← 127.0.0.1:55745
Orders API: 2 requests, 1 without a rule, 0 failed · #1 1
```

Cada linha é o tempo desde o início, o emulador, a regra que respondeu
(`#1`, ou `—` para nenhuma), a requisição e o que ela obteve, o tempo que levou e quem a
enviou. No fim, as contagens de cada emulador: requisições, quantas nenhuma regra tomou,
quantas falharam, quantas encontraram uma queda ou ficaram sem entrega, quando
houve, e os acertos por regra.

Códigos de saída: `0` quando para em <kbd>Ctrl</kbd>+<kbd>C</kbd> ou `--for`; `2`
quando um emulador não é válido; `3` quando a porta dele está ocupada ou não pode ser aberta,
ou um socket falha enquanto ele responde.

Em um pipeline, inicie-o em segundo plano, teste o sistema contra ele e leia
as contagens no fim:

```bash
signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
npm test          # o sistema em teste, configurado para o endereço do emulador
wait              # as últimas linhas de mock.ndjson são as contagens
```

Em um servidor, um emulador iniciado com `--server` é parado quando `signallab`
termina normalmente; um deixado para trás por um processo morto pode ser parado pela
interface do servidor. Veja [Emuladores](../tools/emulators.md).

## `emulators` {#cli-emulators}

```text
signallab emulators [--library PATH]
```

Lista os emuladores da biblioteca: id, nome, protocolo, endereço e quantas
regras. `--library PATH` lê outro arquivo de biblioteca em vez de `emulators.json`
na pasta de dados do app. `signallab emulate <id>` inicia um.

A biblioteca só é lida. Se não há esse arquivo — o app cria o da
pasta de dados dele na primeira vez que inicia — o comando diz isso e sai com `2`.

## `templates` {#cli-templates}

```text
signallab templates
```

Lista os modelos embutidos, que `run` e `validate` aceitam pelo nome. São
os do próprio app:

| Nome | No app | Parâmetros |
| --- | --- | --- |
| `empty` | [[ui:exp.templateEmpty]] | |
| `http-check` | [[ui:exp.templateHttp]] | |
| `status-branch` | [[ui:exp.templateBranch]] | |
| `parallel-flows` | [[ui:exp.templateParallel]] | |
| `osc-ping-reply` | [[ui:exp.templatePingReply]] | `device` = `127.0.0.1:9000` |
| `poll-until-ready` | [[ui:exp.templatePoll]] | `device` = `127.0.0.1:9000` |
| `flaky-api` | [[ui:exp.templateFlaky]] | `api` = `http://127.0.0.1:18080` |
| `fault-phases` | [[ui:exp.templateFaults]] | |
| `dependency-outage` | [[ui:exp.templateOutage]] | `api` = `http://127.0.0.1:18090` |
| `websocket-echo` | [[ui:exp.templateWsEcho]] | `service` = `ws://127.0.0.1:9001/echo` |

Todo modelo aponta para o loopback. `flaky-api`, `fault-phases` e
`dependency-outage` trazem os próprios emuladores, então executam sem mais nada
escutando — um jeito rápido de ver `signallab` funcionar.

## `nodes` {#cli-nodes}

```text
signallab nodes
```

Imprime, em JSON, do que um experimento é feito: a forma e as regras do documento,
cada tipo de nó com o rótulo, a descrição, os campos, as saídas e um exemplo
que o motor aceita, a linguagem de `{{template}}`, os perfis de carga e o documento de
emulador. É o que um assistente lê por [`signallab mcp`](mcp.md) para
escrever um experimento; para pessoas, [Nós](../experiments/nodes.md) diz o mesmo
com mais palavras.

## `mcp` {#cli-mcp}

```text
signallab mcp [OPTIONS]
```

Serve o Signal Lab a um assistente de IA pelo Model Context Protocol, em stdin
e stdout. Tudo isso — configurá-lo no Claude Code, no Claude Desktop, no Cursor ou
no VS Code, as opções e as ferramentas dele — está em [Assistentes (MCP)](mcp.md).

## `doctor` {#cli-doctor}

```text
signallab doctor [--server URL] [--token-file PATH]
```

Diz o que pode estar entre o Signal Lab e o equipamento, e sai com `1` quando
algo está. As linhas dele seguem [`--lang`](#language) como o resto da
linha de comando:

- a rede em que esta máquina está: o nome e o endereço dela;
- a pasta de dados: se pode ser escrita (uma ainda não criada não é problema — o
  app a cria no primeiro uso);
- o firewall: no Windows, para `signallab` e para o aplicativo de desktop, cada um, se uma
  regra deixa outras máquinas entrarem no tipo de rede em que a máquina está agora
  (privada, de domínio ou pública), ou se uma regra as bloqueia — o que um *Cancelar* no
  aviso do sistema deixa para trás; no Linux, se ufw ou firewalld está ativo e
  o comando que abre uma porta;
- com `--server` (ou `SIGNALLAB_SERVER`): se o servidor responde e
  aceita o token.

```text
signallab [[version]]
Network: LAB-PC · 192.0.2.15
Data folder: C:\Users\lab\Documents\SignalLab — writable
Firewall · signallab (C:\…\Signal Lab\signallab.exe) · private network: not allowed yet — signallab firewall allow
Firewall · app (C:\…\Signal Lab\signal-lab.exe) · private network: allowed
✖ 1 thing in the way
```

`--json` imprime o mesmo como um objeto. Veja
[Solução de problemas](../reference/troubleshooting.md).

## `firewall` {#cli-firewall}

```text
signallab firewall allow [--public]
```

No Windows, deixa outras máquinas alcançarem o Signal Lab — o que um monitor ou uma espera
precisa para ouvir um dispositivo. O Windows pede primeiro direitos de administrador; depois as
regras de entrada de `signallab` e do aplicativo de desktop (encontrado ao lado dele, ou onde
os instaladores o colocaram) são substituídas por uma regra de permissão cada,
regras de bloqueio incluídas, em redes privadas e de domínio.

| Opção | O que faz |
| --- | --- |
| `--public` | Também em redes públicas — o Wi-Fi de um local de evento costuma ser uma. |

Códigos de saída: `0` feito; `3` quando o aviso de administrador é recusado ou a
alteração falha. No Linux nada é alterado: ele imprime o comando do ufw ou do firewalld
que abre as portas em que você escuta e sai com `0`.

O firewall só muda quando você executa isto; nada mais em `signallab` mexe
nele.

## `version` {#cli-version}

```text
signallab version
```

Imprime `signallab [[version]]` — com `--json`, `{"version": "[[version]]"}`. `signallab --version`
imprime a mesma versão.

---
title: Comandos
description: Todos os comandos do motor do Signal Lab, chamados como POST /api/invoke/<command>, com seus argumentos, resultado e erros.
---

# Comandos

Todos os comandos que o motor tem, agrupados pelo que operam. Cada um é chamado como
`POST /api/invoke/<command>` com um objeto JSON de argumentos e responde `200`
com o resultado ou `422` com um [`EngineError`](index.md#errors). Como
autenticar e o que os status significam está na [visão geral da API](index.md).

## Convenções {#conventions}

- **Os nomes de argumentos** são camelCase (`jobId`, `nodeId`). Um argumento que um comando
  não conhece, ou um obrigatório que foi omitido, é recusado com
  `command.args_invalid`; todo comando que aceita argumentos pode falhar com ele.
  Um comando sem argumentos não aceita corpo, ou `null`, ou `{}`; qualquer outra coisa
  também é recusada com `command.args_invalid`.
- **Os objetos passados como argumento** — `config`, `request`, `document`,
  `library`, `emulator`, `profile` — usam os nomes de campo do próprio motor, em geral
  snake_case (`timeout_ms`). Dentro deles, um campo que o motor não conhece é
  **ignorado**, então um campo opcional com erro de grafia mantém discretamente o padrão. Só o
  formulário de `feedback_send` recusa campos desconhecidos.
- Os argumentos e campos **opcionais** podem ser omitidos ou enviados como `null`; as
  tabelas dão os padrões deles.
- Os **resultados** são JSON. "null" significa que o comando não tem nada a devolver.
- Os **endereços** escritos `IP:port` aceitam um endereço numérico e uma porta
  (`127.0.0.1:9000`, `[::1]:9000`); um nome de host ali é recusado. Onde uma tabela
  diz `IP:port` ou `host:port`, um nome de host também funciona: ele é
  consultado quando o comando executa, e o endereço IPv4 dele é usado quando há um
  (então `localhost:9000` é `127.0.0.1:9000`).
- **Tarefas**: um comando marcado como *Inicia uma tarefa* devolve um
  [`JobInfo`](#type-jobinfo); o trabalho continua até terminar ou ser parado com
  [`job_stop`](#job_stop). Veja [tarefas](index.md#jobs).
- Os **caminhos** nos resultados estão na máquina em que o motor executa — em um servidor,
  dentro da pasta de dados dele; baixe-os com [`/api/files`](index.md#files).

Os exemplos usam esta função de shell:

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
invoke() {
  curl -sS -X POST "$SERVER/api/invoke/$1" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    --data "${2:-}"
}
```

## Aplicativo {#application}

### app_info {#app_info}

O que o motor é e onde executa. Sem argumentos.

**Resultado**

| Campo | Tipo | Significado |
| --- | --- | --- |
| `version` | string | A versão do Signal Lab, `[[version]]` |
| `mode` | string | `desktop` ou `server` |
| `secrets_writable` | booleano | Se [`secret_set`](#secret_set) e [`secret_delete`](#secret_delete) podem funcionar aqui: `false` em um servidor |
| `data_dir` | string | A pasta de dados, na máquina em que o motor executa |
| `os` | string | `windows`, `linux`… |
| `arch` | string | `x86_64`, `aarch64`… |

### get_host_info {#get_host_info}

O nome da máquina e o endereço de que ela enviaria. Sem argumentos.

**Resultado**: `{ "local_ip": string, "hostname": string }`. `local_ip` é o
endereço IPv4 que o sistema escolhe para o tráfego à internet (descoberto sem
enviar nada), ou `127.0.0.1` quando não há nenhum. `hostname` é o nome do
computador, ou `localhost` quando o sistema não o informa.

### firewall_status {#firewall_status}

Se o firewall do sistema deixa outras máquinas alcançarem este programa. Só o
Windows tem um firewall por programa para ler; nos demais, `applies` é `false` e,
do resto, só `program` é preenchido. Sem argumentos.

**Resultado**

| Campo | Tipo | Significado |
| --- | --- | --- |
| `applies` | booleano | Há um firewall por programa aqui (Windows) |
| `program` | string | O programa de que as regras tratam |
| `enabled` | booleano | O firewall está ativo para a rede em que a máquina está agora |
| `networks` | string[] | Os tipos de rede em que a máquina está: `domain`, `private`, `public` |
| `allowed` | booleano | Uma regra de entrada deixa o UDP entrar para este programa na rede atual |
| `blocked` | booleano | Uma regra de entrada bloqueia este programa na rede atual; ela vence qualquer regra de permissão |
| `rules` | número | Regras de entrada para este programa, de qualquer tipo |

**Erros**: `firewall.failed`.

### firewall_allow {#firewall_allow}

Deixa outras máquinas alcançarem o Signal Lab: o sistema mostra o próprio aviso de administrador
e então as regras de entrada do programa (uma regra de bloqueio incluída) são substituídas
por uma regra de permissão cada para o Signal Lab e a linha de comando `signallab` ao lado
dele. Só no aplicativo de desktop no Windows; um servidor recusa, já que ninguém está diante da
tela dele para responder ao aviso.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `public` | booleano | sim | Permite também em redes públicas, não só nas privadas e de domínio |

**Resultado**: o novo [`firewall_status`](#firewall_status).

**Erros**: `firewall.server` (em um servidor), `firewall.unsupported` (não é
Windows), `firewall.declined` (o aviso foi respondido com Não), `firewall.failed`.

### feedback_send {#feedback_send}

Envia uma mensagem aos desenvolvedores do Signal Lab, pelo hub do estúdio, que a
encaminha por e-mail a eles.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `form` | objeto | sim | A mensagem, abaixo. Campos desconhecidos são recusados |

| Campo de `form` | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `message` | string | — | O que aconteceu; obrigatório, no máximo 20.000 caracteres |
| `email` | string | nenhum | Para onde uma resposta pode ir |
| `meta` | objeto de strings | `{}` | O que o app diz sobre si mesmo (version, os, arch, mode, lang, screen) |
| `screenshots` | `{ name, data }[]` | `[]` | Imagens, `data` em base64; no máximo 6, 8 MiB cada |
| `logs` | `{ name, text }[]` | `[]` | Arquivos de texto; no máximo 4, 2 MiB cada |

Tudo junto tem no máximo 15 MiB.

**Resultado**: `{ "id": string }`, a referência que os desenvolvedores recebem.

**Erros**: `feedback.message_required`, `feedback.message_too_long`,
`feedback.too_many_files`, `feedback.file_too_large`, `feedback.too_large`,
`feedback.invalid`, as recusas do hub (`feedback.email_invalid`,
`feedback.file_type`, `feedback.rate_limited`, `feedback.disabled`,
`feedback.send_failed`, `feedback.failed`) e os `transport.*` da rede.

```bash
invoke app_info
# {"version":"[[version]]","mode":"server","secrets_writable":false,"data_dir":"/data","os":"linux","arch":"x86_64"}
```

## Tarefas {#jobs}

### jobs_list {#jobs_list}

As tarefas em execução, da mais antiga à mais nova. Sem argumentos.

**Resultado**: [`JobInfo`](#type-jobinfo)`[]`.

### job_stop {#job_stop}

Para uma tarefa na hora: os sockets dela se fecham, o retransmissor, servidor ou conexão dela some.
A resposta vem depois que eles se foram — um início logo em seguida usa a mesma porta.
Uma tarefa parada não envia [`job://ended`](events.md#event-job-ended); uma
execução parada não salva relatório.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `id` | número | sim | O `id` da tarefa |

**Resultado**: `true` quando uma tarefa com esse id estava em execução, `false` caso contrário.

### jobs_stop_all {#jobs_stop_all}

Para todas as tarefas em execução, não importa quem as iniciou, e responde depois que elas
soltaram as portas, como faz [`job_stop`](#job_stop). Uma tarefa que ainda está iniciando
(uma execução abrindo as portas dela) é parada ao iniciar. Sem argumentos.

**Resultado**: null.

```bash
invoke jobs_list
# [{"id":3,"kind":"osc-monitor","label":"OSC monitor 0.0.0.0:9000","params":{"bind":"0.0.0.0:9000"},"started_ms":1759600000000}]
invoke job_stop '{"id":3}'
# true
```

## Experimentos e execuções {#experiments}

Estes comandos aceitam e devolvem um documento de experimento (`Experiment`): o JSON
que o editor salva e exporta, com `version`, `name`, `params`, `profiles`,
`profile`, `seed`, `cookies`, `nodes` e `edges`. Os nós dele estão em
[nós](../experiments/nodes.md); os parâmetros, perfis e modelos dele em
[dados](../experiments/data.md). Um documento tem no máximo 4 MiB
(`file.too_large`). Para executar um experimento e aguardar o resultado dele, use
[`POST /api/run`](run.md) em vez de [`experiment_start`](#experiment_start).

### experiment_load {#experiment_load}

O experimento de trabalho: `experiment.json` na pasta de dados — em um servidor, o
que a interface dele mostra. Quando não há nenhum, o experimento inicial. As versões
antigas de documento são migradas. Sem argumentos.

**Resultado**: `Experiment`.

**Erros**: `file.io`, `file.json_invalid` (com o `path`, a `line`
e a `column` do arquivo), `file.too_large`, `doc.version_unsupported` e as demais
verificações `doc.*`.

### experiment_save {#experiment_save}

Substitui o experimento de trabalho, `experiment.json` na pasta de dados. Ele é
escrito antes em um arquivo temporário, de modo que uma gravação que falha deixa o anterior.

::: warning
Em um servidor, este é o documento em que o editor de todo navegador trabalha.
:::

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `document` | `Experiment` | sim | O documento |

**Resultado**: string, o caminho escrito.

**Erros**: `doc.*`, as verificações de tamanho do documento (`param.*`, `params.too_many`,
`profile.*`, `profiles.too_many`, `seed.range`), `file.too_large`, `file.io`.

### experiment_parse {#experiment_parse}

Lê um experimento de um texto JSON, como [[ui:exp.importJson]] faz. As versões 1
a 8 são migradas para a versão 9, a atual; um arquivo anterior à versão 8
abre com `cookies` desligado, de modo que executa como executava. Uma marca de ordem de bytes é ignorada.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `text` | string | sim | O texto do arquivo |

**Resultado**: `Experiment`.

**Erros**: `file.json_invalid` (`line`, `column`), `file.too_large`,
`doc.version_unsupported`, `doc.*`, as verificações de tamanho de
[`experiment_save`](#experiment_save).

### experiment_export {#experiment_export}

Escreve um instantâneo de um documento em `exports/experiment-<ms>-<16 hex digits>.json`
na pasta de dados. Toda exportação é um arquivo novo.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `document` | `Experiment` | sim | O documento |

**Resultado**: string, o caminho escrito.

**Erros**: os de [`experiment_save`](#experiment_save).

### experiment_validate {#experiment_validate}

Verifica se um documento executaria com o perfil ativo dele (ou os padrões):
o grafo, todos os campos, os parâmetros, e se todo segredo que ele nomeia está guardado.
Um problema que bloqueia é o erro. Em caso de sucesso, diz quais dos *outros*
perfis falhariam, para você saber antes de trocar.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `document` | `Experiment` | sim | O documento |
| `overrides` | objeto de strings | não | Valores de parâmetros só para esta verificação, como [[ui:exp.runWith]] os dá |

**Resultado**: `{ "profile": string or null, "error": EngineError }[]` — cada
outro perfil que não validaria (`null`: os padrões, sem
perfil). Uma lista vazia significa que todos os perfis estão bem.

**Erros**: qualquer código de validação (`doc.*`, `graph.*`, `node.*`, `param.*`,
`profile.*`, `template.*`, `loop.*`…), `run.override_unknown` (uma substituição
para um parâmetro que o documento não tem), `secret.missing`,
`secret.store`, `secret.unsupported`.

### experiment_resolve {#experiment_resolve}

Um nó com os modelos preenchidos, como a prévia do editor o mostra: os valores do
perfil ativo e os valores de variável que você der. Os segredos aparecem
como `••••`, nunca os valores deles. Os nomes que não têm valor ficam como foram escritos e
são listados.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `document` | `Experiment` | sim | O documento |
| `nodeId` | string | sim | O nó |
| `vars` | objeto | sim | Valores de variável a usar, por nome; `{}` para nenhum |

**Resultado**: `{ "node": node, "missing": string[] }`.

**Erros**: `node.not_found`, `template.*`, `secret.store`,
`secret.unsupported`.

### experiment_send_node {#experiment_send_node}

[[ui:exp.sendNow]]: executa um nó sozinho, pelo mesmo código que uma execução
usa. Uma ação é enviada; uma espera escuta a partir de agora até corresponder ou o tempo
acabar. Um nó [[ui:exp.node.ws_send]] ou [[ui:exp.node.wait_ws]] abre a
conexão que o nó [[ui:exp.node.ws_connect]] dele descreve. Nada é enviado com
cookies, e os nós [[ui:exp.node.impairment]] e [[ui:exp.node.emulator]]
da execução não são abertos.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `document` | `Experiment` | sim | O documento; o perfil ativo dele dá os valores dos parâmetros |
| `nodeId` | string | sim | Uma ação ou uma espera |
| `vars` | objeto | sim | Valores de variável que os modelos do nó leem; `{}` para nenhum |

**Resultado**

| Campo | Tipo | Significado |
| --- | --- | --- |
| `detail` | string | O que aconteceu, em inglês |
| `response` | [`HttpResponse`](#type-httpresponse) ou null | A resposta de um nó HTTP |
| `vars` | objeto | O que a etapa definiu: a resposta de uma espera, ou o que os nós [[ui:exp.node.extract]] depois de uma requisição tomam da resposta dela |

Os valores de segredos são mascarados em tudo isso.

**Erros**: `node.not_found`, `run.not_an_action` (nem uma ação nem uma
espera), `ws.connection_unknown`, `secret.missing`, `template.*` e com o que quer que a
etapa falhe: `transport.*`, `wait.timeout`, `check.*`…

### experiment_start {#experiment_start}

Inicia uma execução, como [[ui:exp.run]] faz, e retorna na hora. As etapas dela chegam como
eventos [`experiment://step`](events.md#event-experiment-step), o fim dela como
[`experiment://ended`](events.md#event-experiment-ended), e o relatório dela é
salvo em `runs/` na pasta de dados. As esperas, emuladores, retransmissores de degradação e
assinaturas MQTT abrem antes da primeira etapa, então uma porta ocupada falha
aqui. Uma execução de mais de 300 s falha com `run.timeout`. *Inicia uma tarefa*
(`experiment`).

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `document` | `Experiment` | sim | O documento |
| `overrides` | objeto de strings | não | Valores de parâmetros só para esta execução |
| `seed` | número | não | A semente da execução, 0 a 9007199254740991; padrão: a do documento, senão uma nova |

**Resultado**: [`JobInfo`](#type-jobinfo), com `params.name` o nome do experimento.

**Erros**: tudo o que [`experiment_validate`](#experiment_validate) informa,
`seed.range`, `transport.address_in_use` e as demais falhas de bind,
`emulator.*`, `impair.*`, `node.params_only` (um endereço de escuta ou o broker ou tópico de uma
espera MQTT que não é fixo quando a execução começa) e os
erros de [`mqtt_connect`](#mqtt_connect) de um broker que uma espera MQTT não consegue alcançar.

### experiment_runs {#experiment_runs}

As execuções lidas de volta a partir dos relatórios em `runs/`, da mais nova à mais antiga. Um relatório que
não pode ser lido é omitido.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `name` | string | não | Só as execuções do experimento com exatamente este nome |
| `limit` | número | não | No máximo este tanto; padrão 50, no máximo 500 |

**Resultado**: resumos de execuções:

| Campo | Tipo | Significado |
| --- | --- | --- |
| `name` | string | O nome de arquivo do relatório, `run-<ms>-<job>.json`: o que [`experiment_compare`](#experiment_compare) aceita |
| `experiment` | string | O nome do experimento |
| `started_ms`, `ended_ms` | número | Milissegundos desde 1970 |
| `outcome` | string | `passed` ou `failed` |
| `seed` | número | A semente da execução |
| `profile` | string ou null | O perfil dela |
| `loads` | objeto[] | Cada etapa de carga: `node`, `sent`, `rps`, `p95_ms`, `error_rate`, `held` (todo limiar cumprido) |

**Erros**: `file.io`.

### experiment_compare {#experiment_compare}

Duas execuções lado a lado, etapa de carga por etapa de carga, como o [[ui:exp.compare]] da
linha do tempo as mostra. As etapas são pareadas pelo id do nó.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `a` | string | sim | O nome de arquivo do relatório da execução anterior |
| `b` | string | sim | O nome de arquivo do relatório da execução posterior |

**Resultado**: `{ "a": summary, "b": summary, "steps": [...] }`, cada etapa com
`node`, `missing_in` (`a` ou `b`, quando só uma execução a tem), `metrics`,
`sent` (`[a, b]`), `thresholds_a` e `thresholds_b` (cada limiar como
`{ metric, op, value, actual, held }`). `metrics` lista nove métricas, cada uma como
`{ metric, a, b, change, percent, worse }`: `metric` é `p50_ms`, `p90_ms`,
`p95_ms`, `p99_ms`, `mean_ms`, `max_ms`, `error_rate`, `rps` ou `missed`;
`change` é `b − a`; `percent` a variação em % de `a` (null quando `a` é 0);
`worse` que se moveu no sentido errado — para cima, ou para baixo no caso de `rps` — em 5 % ou
mais, ou de 0 para qualquer coisa. Uma etapa que só uma execução tem nunca é `worse`. Veja
[carga](../experiments/load.md).

**Erros**: `runs.name_invalid` (qualquer coisa que não seja o nome de arquivo de um relatório, sem pastas),
`runs.not_found`, `file.json_invalid`, `file.io`.

```bash
invoke experiment_validate "$(jq '{document: .}' experiment.json)"
# []
```

## Segredos {#secrets}

Os valores de segredos são usados pelos experimentos como `{{secret.NAME}}` e nunca saem do
motor: nenhum comando devolve um. Onde eles ficam depende de onde o motor
executa:

| Onde | Repositório | Definição e remoção |
| --- | --- | --- |
| Aplicativo de desktop, Windows | Gerenciador de Credenciais do Windows | Sim |
| Aplicativo de desktop, Linux | Nenhum | `secret.unsupported` |
| Servidor | `SIGNALLAB_SECRET_<NAME>`, ou o arquivo `<NAME>` em `--secrets-dir` (padrão `/run/secrets/signallab`) | Não: `secret.read_only` |

Um nome começa com uma letra latina ou `_`, continua com letras latinas, dígitos
e `_`, e tem no máximo 128 caracteres (`secret.name_invalid`).

### secret_status {#secret_status}

Quais dos nomes dados têm um valor guardado.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `names` | string[] | sim | Os nomes a consultar |

**Resultado**: um objeto, nome → `true` (guardado) ou `false`.

**Erros**: `secret.name_invalid` (em um servidor), `secret.store`,
`secret.too_large` (um arquivo de servidor acima de 16 KiB), `secret.unsupported`.

### secret_set {#secret_set}

Guarda um valor sob um nome, substituindo o que está ali. Só no aplicativo de desktop.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `name` | string | sim | O nome |
| `value` | string | sim | Não vazio; no máximo 16 KiB |

**Resultado**: null.

**Erros**: `secret.read_only` (em um servidor), `secret.unsupported`,
`secret.name_invalid`, `secret.empty`, `secret.too_large`, `secret.store`.

### secret_delete {#secret_delete}

Remove um valor guardado. Remover um que não está guardado não é um erro.
Só no aplicativo de desktop.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `name` | string | sim | O nome |

**Resultado**: null.

**Erros**: `secret.read_only` (em um servidor), `secret.unsupported`,
`secret.name_invalid`, `secret.store`.

```bash
invoke secret_status '{"names":["API_TOKEN","MQTT_PASSWORD"]}'
# {"API_TOKEN":true,"MQTT_PASSWORD":false}
```

## OSC {#osc}

Veja [OSC](../protocols/osc.md) para a tela a que estes comandos servem.

### osc_send {#osc_send}

Envia uma mensagem OSC em um datagrama UDP, a partir de um socket novo.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `target` | string | sim | `IP:port` ou `host:port` de destino; um nome é consultado, e o endereço IPv4 dele é usado quando há um |
| `address` | string | sim | O endereço OSC, `/mixer/fader/1`; começa com `/` |
| `args` | [`OscArg`](#type-oscarg)`[]` | sim | Os argumentos; `[]` para nenhum |

**Resultado**: número, os bytes enviados.

**Erros**: `node.osc_address` (sem `/` inicial; campo `address`),
`transport.target_invalid` (sem porta, ou nenhuma das duas formas), `transport.dns` (o
nome não resolve), `transport.*`.

### osc_monitor_start {#osc_monitor_start}

Escuta OSC em uma porta UDP e decodifica todo pacote. Cada um chega como um evento
[`osc://message`](events.md#event-osc-message). *Inicia uma tarefa*
(`osc-monitor`, `params.bind`).

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `bind` | string | sim | `IP:port` em que escutar: `0.0.0.0:9000` todas as placas de rede, `127.0.0.1:9000` só esta máquina |

**Resultado**: [`JobInfo`](#type-jobinfo).

**Erros**: `node.bind_invalid`, `transport.address_in_use`,
`transport.address_unavailable`, `transport.denied`, `wait.bind_failed`. A
tarefa termina com `wait.receive_failed` se o socket não pode mais receber.

### osc_generator_start {#osc_generator_start}

Envia um fluxo de mensagens OSC cujo único argumento segue uma forma de onda.
O progresso chega como [`osc://gen-tick`](events.md#event-osc-gen-tick). *Inicia
uma tarefa* (`osc-gen`, `params.target`, `params.address`).

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `config` | objeto | sim | Abaixo |

| Campo de `config` | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `target` | string | — | `IP:port` ou `host:port` de destino; um nome é consultado uma vez, quando a tarefa inicia |
| `address` | string | — | O endereço OSC; começa com `/` |
| `rate` | número | — | Mensagens por segundo, mantido entre 0,1 e 5000 |
| `waveform` | string | — | `sine`, `triangle`, `saw` (descendente: de `max` a `min`, depois volta de uma vez), `ramp` (ascendente: de `min` a `max`, depois volta de uma vez), `square`, `random` ou `constant` (`max`) |
| `freq` | número | — | Ciclos da forma de onda por segundo |
| `min`, `max` | número | — | O intervalo do valor |
| `as_int` | booleano | `false` | Arredonda e envia um int em vez de um float |
| `duration_s` | número | `0` | Para depois deste número de segundos; 0 executa até ser parado |

**Resultado**: [`JobInfo`](#type-jobinfo).

**Erros**: `node.osc_address`, `transport.target_invalid`, `transport.dns`,
`transport.*`. A tarefa termina com um erro `transport.*` se um envio falha.

```bash
invoke osc_send '{"target":"127.0.0.1:9000","address":"/cue/go","args":[{"type":"int","value":1}]}'
# 16
invoke osc_monitor_start '{"bind":"0.0.0.0:9000"}'
```

## HTTP e cookies {#http}

Veja [HTTP](../protocols/http.md).

### http_request {#http_request}

Envia uma requisição HTTP e devolve a resposta. Uma requisição que não obtém
resposta — recusada, tempo esgotado, um nome que não resolve, um certificado
em que não se confia — **não** é um erro do comando: a resposta diz isso
em `error` e `cause`.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `request` | [`HttpRequest`](#type-httprequest) | sim | A requisição |
| `cookies` | booleano | não | Envia o armazenamento de cookies da tela [[ui:nav.http]] e guarda o que a resposta define; padrão `false` |

**Resultado**: [`HttpResponse`](#type-httpresponse).

**Erros**: `node.method_invalid` (campo `method`: não é um token HTTP, como
`BAD METHOD`), `http.client_failed` (a requisição nem pôde ser preparada).

### http_burst_start {#http_burst_start}

Envia uma requisição muitas vezes, várias ao mesmo tempo, e a mede. Sem uma
`rate`, cada worker envia de novo assim que tem uma resposta; com uma,
as requisições começam em um cronograma fixo, por mais lentas que sejam as respostas, e uma requisição
que esperou mais de 50 ms depois do seu momento por um worker livre é pulada e
contada como puladas. O progresso chega como
[`http://burst-progress`](events.md#event-http-burst-progress) dez vezes por
segundo. *Inicia uma tarefa* (`http-burst`, `params.method`, `params.url` e
`params.rate` quando há ritmo).

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `config` | objeto | sim | Os campos de [`HttpRequest`](#type-httprequest) e os abaixo, em um só objeto |

| Campo de `config` | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `concurrency` | número | — | No máximo este número em andamento, mantido entre 1 e 512 |
| `total` | número | `0` | Para depois deste número de requisições; 0: sem contagem |
| `duration_s` | número | `0` | Para depois deste número de segundos; 0: sem limite de tempo |
| `rate` | número | `0` | Requisições iniciadas por segundo, 0,1 a 100.000; 0: o mais rápido que as respostas chegam |
| `cookies` | booleano | `false` | Usa o armazenamento de cookies da tela [[ui:nav.http]] |

Sem `total` nem `duration_s`, a rajada executa até ser parada.

**Resultado**: [`JobInfo`](#type-jobinfo).

**Erros**: `http.rate_invalid`, `http.duration_invalid`, `node.method_invalid`, `http.client_failed`.

### http_cookies {#http_cookies}

O armazenamento de cookies da tela [[ui:nav.http]]: todo cookie que não expirou. Em um servidor
há um armazenamento para todas as páginas e scripts. Sem argumentos.

**Resultado**: cookies, cada um com `name`, `value`, `domain`, `host_only` (sem
atributo Domain: só o host que o definiu o recebe de volta), `path`, `expires`
(segundos Unix, null para um cookie de sessão), `secure`, `http_only` e
`same_site` (string ou null).

### http_cookies_clear {#http_cookies_clear}

Esvazia o armazenamento de cookies da tela [[ui:nav.http]]. Sem argumentos.

**Resultado**: null.

```bash
invoke http_request '{"request":{"method":"GET","url":"http://127.0.0.1:8080/health","headers":[["Accept","application/json"]],"body":null,"timeout_ms":5000}}' \
  | jq '{status, latency_ms, body}'
```

## WebSocket {#websocket}

Veja [WebSocket](../protocols/websocket.md). Uma conexão que `ws_connect`
abre é uma tarefa; os outros comandos a nomeiam por `jobId`.

### ws_connect {#ws_connect}

Abre um WebSocket e o mantém aberto. O que chega e o que é enviado vem como
[`ws://messages`](events.md#event-ws-messages) a cada 100 ms; o estado da conexão
como [`ws://state`](events.md#event-ws-state). *Inicia uma tarefa*
(`websocket`, `params.url`).

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | sim | Onde e como conectar |

**Resultado**: [`JobInfo`](#type-jobinfo).

**Erros**: `ws.url_invalid`, `ws.header_invalid`, `ws.protocol_invalid`,
`ws.handshake_status` (o servidor respondeu ao upgrade com outro status),
`ws.subprotocol_refused`, `ws.handshake_failed`, `transport.*`.

### ws_send {#ws_send}

Envia uma mensagem em uma conexão aberta.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `jobId` | número | sim | A tarefa da conexão |
| `message` | objeto | sim | `{ "text": "…" }` para uma mensagem de texto, ou `{ "hex": "de ad be ef" }` para uma binária; exatamente uma das duas |

**Resultado**: número, os bytes enviados.

**Erros**: `ws.not_connected`, `ws.payload_required` (nenhuma ou as duas),
`hex.invalid` (um `hex` vazio também), `node.too_long` (acima de 16 MiB, campo
`payload`; nada é enviado e a conexão continua aberta), `ws.closed`, `transport.*` (`transport.timeout` quando o servidor parou de
ler por 10 s).

### ws_close {#ws_close}

Fecha uma conexão com um handshake de fechamento e aguarda até 2 s pela resposta do
servidor; a tarefa então termina. Depois que uma conexão terminou, a tarefa dela some e
fechá-la é `ws.not_connected`.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `jobId` | número | sim | A tarefa da conexão |
| `code` | número | não | 1000, ou 3000 a 4999 para os próprios de uma aplicação; padrão 1000 |
| `reason` | string | não | No máximo 123 bytes; padrão vazio |

**Resultado**: `{ "code", "reason", "by", "error" }` — `by` é `client`,
`server` ou `lost`; `code` é 1005 quando o fechamento não trouxe nenhum e 1006 quando
não houve quadro de fechamento.

**Erros**: `ws.close_code`, `node.too_long`, `ws.not_connected`.

### ws_exchange {#ws_exchange}

Uma troca sem tarefa: conecta, envia uma mensagem se dada, aguarda uma
resposta se pedido, fecha.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | sim | Onde e como conectar |
| `message` | objeto | não | `{ "text" }` ou `{ "hex" }`, como em [`ws_send`](#ws_send) |
| `expect` | objeto | não | O que aguardar: `mode` (`any`, `contains`, `regex`, `hex`; padrão `any`), `pattern` (padrão vazio), `timeout_ms` (padrão 2000) |

Com `expect` e sem `message`, a primeira mensagem correspondente depois de conectar
conta — uma saudação.

**Resultado**: `{ "handshake", "sent", "reply", "closed" }` — `handshake` é
`{ url, peer, local, protocol, ms }`; `sent` os bytes enviados ou null; `reply`
`{ kind, text, hex, bytes, json, ms }` ou null (`json`: uma resposta de texto lida,
senão null; `ms`: desde o envio, ou desde a conexão quando nada foi enviado);
`closed` como [`ws_close`](#ws_close) devolve.

**Erros**: os de [`ws_connect`](#ws_connect) e [`ws_send`](#ws_send),
`wait.timeout` (com `ms`, `unmatched` e `target`), `regex.invalid` e
`hex.invalid` (um `pattern` que não é lido).

```bash
invoke ws_exchange '{"config":{"url":"ws://127.0.0.1:9001/"},"message":{"text":"{\"type\":\"ping\"}"},"expect":{"mode":"contains","pattern":"pong"}}' \
  | jq .reply.text
```

## MQTT {#mqtt}

MQTT 3.1.1 sobre TCP simples, QoS 0, 1 e 2. Veja [MQTT](../protocols/mqtt.md).

### mqtt_connect {#mqtt_connect}

Conecta a um broker e mantém a conexão. O comando retorna quando o
broker aceitou a conexão (CONNACK), então uma senha errada ou uma porta fechada é o
erro dele. As mensagens chegam como
[`mqtt://messages`](events.md#event-mqtt-messages) a cada 100 ms; as mudanças de estado
como [`mqtt://state`](events.md#event-mqtt-state); as publicações e cancelamentos de assinatura QoS 1/2 concluídos
como [`mqtt://ack`](events.md#event-mqtt-ack).
*Inicia uma tarefa* (`mqtt`, `params.broker`, `params.client`).

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | sim | O broker e como conectar |

**Resultado**: [`JobInfo`](#type-jobinfo).

**Erros**: `mqtt.client_id_required`, `transport.*` (recusada, inalcançável,
`dns`, `timeout` após 6 s), `mqtt.no_answer` (nenhum CONNACK em 6 s),
`mqtt.protocol`, `mqtt.refused_protocol`, `mqtt.refused_client_id`,
`mqtt.refused_unavailable`, `mqtt.refused_credentials`,
`mqtt.refused_not_authorized`, `mqtt.refused`; antes de discar,
`node.topic_filter` (um filtro de `subscribe` que o 3.1.1 proíbe), `node.too_long`
(um nome de usuário, senha, ID de cliente ou will além dos 65.535 bytes do MQTT) e
`node.topic_wildcard` / `mqtt.topic_invalid` (um will em um tópico em que não poderia ser
publicado). Uma conexão cujo broker não diz nada, nem aos pings dela, durante
1,5 × `keep_alive_s` termina com `transport.timeout` em `job://ended`.

### mqtt_publish {#mqtt_publish}

Publica em uma conexão aberta.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `jobId` | número | sim | A tarefa da conexão |
| `topic` | string | sim | O tópico: não vazio, e sem `+` nem `#` |
| `payload` | string | sim | A carga útil, enviada como UTF-8 |
| `qos` | número | sim | 0, 1 ou 2 (acima de 2 é enviado como 2) |
| `retain` | booleano | sim | Pede ao broker que a guarde; uma carga útil vazia com `retain` limpa um valor retido |

**Resultado**: null. Uma publicação QoS 1 ou 2 é confirmada depois por
[`mqtt://ack`](events.md#event-mqtt-ack).

**Erros**: `node.topic_wildcard`, `mqtt.topic_required`, `mqtt.topic_invalid`
(um NUL nele) e `node.too_long` (além de 65.535 bytes), todos com o campo `topic`,
como em [`mqtt_publish_once`](#mqtt_publish_once) — o comando os recusa
antes de procurar a conexão; `mqtt.not_connected`.

### mqtt_subscribe {#mqtt_subscribe}

Assina filtros em uma conexão aberta. O que o broker concede chega como
[`mqtt://state`](events.md#event-mqtt-state) com `state: "subscribed"`.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `jobId` | número | sim | A tarefa da conexão |
| `filters` | `{ filter, qos }[]` | sim | Pelo menos um; `qos` assume 0 por padrão. `+` e `#` são curingas |

**Resultado**: null.

**Erros**: `mqtt.filter_required`, `node.topic_filter` (campo `topic`: um filtro que o
3.1.1 proíbe, como `a/#/b` ou `a+` — os brokers fecham a conexão por causa de um),
`mqtt.not_connected`.

### mqtt_unsubscribe {#mqtt_unsubscribe}

Cancela a assinatura de filtros em uma conexão aberta.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `jobId` | número | sim | A tarefa da conexão |
| `filters` | string[] | sim | Pelo menos um |

**Resultado**: null. A resposta do broker chega como
[`mqtt://ack`](events.md#event-mqtt-ack) com `kind: "unsubscribed"`.

**Erros**: `mqtt.filter_required`, `node.topic_filter` (campo `topic`), `mqtt.not_connected`.

### mqtt_publish_once {#mqtt_publish_once}

Conecta, publica uma mensagem, aguarda a confirmação que o QoS dela pede
(até 6 s) e desconecta. Ele traz a própria conexão, com um id de cliente
próprio — os primeiros 12 caracteres de `client_id`, `-o` e um número — de modo que
nunca derruba do broker uma conexão ativa com esse id.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | sim | O broker; `subscribe` não é usado |
| `topic` | string | sim | Não vazio e sem `+` nem `#` |
| `payload` | string | sim | Enviada como UTF-8 |
| `qos` | número | sim | 0, 1 ou 2 (acima de 2 é enviado como 2) |
| `retain` | booleano | sim | Pede ao broker que a guarde |

**Resultado**: string, um resumo escrito pelo Signal Lab:
`<topic> → <broker> · <bytes> B · qos<n>`, com ` retained` quando retida.

**Erros**: `mqtt.topic_required`, `node.topic_wildcard` e os de
[`mqtt_connect`](#mqtt_connect), exceto `mqtt.client_id_required`: um
`client_id` vazio é aceito aqui.

```bash
invoke mqtt_publish_once '{"config":{"host":"127.0.0.1","port":1883,"client_id":"lab"},"topic":"lab/lamp/set","payload":"ON","qos":1,"retain":false}'
# "lab/lamp/set → 127.0.0.1:1883 · 2 B · qos1"
```

## Broadcast, multicast e descoberta {#broadcast}

Veja [broadcast e descoberta](../protocols/broadcast.md).

::: danger
O broadcast e uma varredura alcançam todos os hosts de um segmento de rede. Envie só em
redes pelas quais você é responsável.
:::

### broadcast_send {#broadcast_send}

Envia um datagrama a cada destino, uma vez.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `config` | objeto | sim | Abaixo |

| Campo de `config` | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `mode` | string | — | `list`, `broadcast`, `multicast` ou `sweep` |
| `target` | string | — | Conforme o modo, abaixo |
| `port` | número | `0` | A porta, só para `sweep` |
| `payload` | [`Payload`](#type-payload) | — | O que cada datagrama carrega |
| `bind` | string | qualquer | O `IP:port` local de que é enviado; vazio ou null: `0.0.0.0:0` (`[::]:0` quando todos os destinos são IPv6) |
| `ttl` | número | `1` | TTL de IP, ou o limite de saltos do multicast; 1 a 255 |
| `multicast_loop` | booleano | `true` | O multicast também volta a esta máquina |
| `rate`, `count`, `duration_s` | número | `0` | Só para [`broadcast_beacon_start`](#broadcast_beacon_start) |

| `mode` | `target` |
| --- | --- |
| `list` | Entradas `IP:port` ou `host:port` separadas por vírgulas, pontos e vírgulas ou quebras de linha (não por espaços); um nome é consultado, e o endereço IPv4 dele é usado quando há um |
| `broadcast` | `255.255.255.255:port`, ou um endereço terminado em `.255` com a porta dele |
| `multicast` | Um grupo de 224.0.0.0 a 239.255.255.255 com a porta dele |
| `sweep` | Um bloco CIDR, `192.0.2.0/24`: todo host utilizável em `port`; no máximo 1024 hosts, então `/22` ou mais estreito |

**Resultado**

| Campo | Tipo | Significado |
| --- | --- | --- |
| `targets` | número | Destinos |
| `packets`, `bytes` | número | O que saiu |
| `errors` | número | Datagramas que não puderam ser enviados |
| `resolved` | string[] | Os 8 primeiros destinos |
| `summary` | string | A carga útil em uma linha |
| `error` | `EngineError` | Por que o primeiro datagrama que falhou falhou; omitido quando nenhum falhou |

**Erros**: `broadcast.target_required`, `broadcast.not_broadcast`,
`broadcast.ipv6`, `broadcast.not_multicast`, `broadcast.sweep_port`,
`broadcast.cidr_invalid`, `broadcast.prefix_invalid`,
`broadcast.sweep_too_large`, `node.osc_address` (um endereço OSC precisa começar
com `/`), `hex.empty`, `hex.invalid`, `node.bind_invalid`,
`socket.option_failed`, `transport.target_invalid`, `transport.dns`, falhas de
bind.

### broadcast_beacon_start {#broadcast_beacon_start}

Envia a mesma rodada — um datagrama por destino — de novo e de novo. Os contadores dele
chegam como [`broadcast://emit-stat`](events.md#event-broadcast-emit-stat) a cada
250 ms. Depois de mais de 32 envios falhos sem nenhum enviado, ele para com
o motivo. *Inicia uma tarefa* (`beacon`, `params.mode`, `params.target`,
`params.targets`, `params.rate`).

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `config` | objeto | sim | Como em [`broadcast_send`](#broadcast_send), com os três abaixo |

| Campo de `config` | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `rate` | número | — | Rodadas por segundo; acima de 0, e rodadas × destinos no máximo 50.000 datagramas por segundo |
| `count` | número | `0` | Para depois deste número de rodadas; 0: sem contagem |
| `duration_s` | número | `0` | Para depois deste número de segundos; 0: até ser parado |

**Resultado**: [`JobInfo`](#type-jobinfo).

**Erros**: os de [`broadcast_send`](#broadcast_send),
`broadcast.rate_invalid`, `broadcast.rate_limit`.

### discovery_start {#discovery_start}

Escuta em uma porta UDP, mantém uma lista de todo par que envia algo e
pode responder a sondagens como um dispositivo faria. Os pares chegam como
[`broadcast://peers`](events.md#event-broadcast-peers) a cada 400 ms. *Inicia
uma tarefa* (`discovery`, `params.bind`, `params.groups`, `params.joined`).

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `config` | objeto | sim | Abaixo |

| Campo de `config` | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `bind` | string | — | `IP:port` em que escutar |
| `groups` | string[] | `[]` | Grupos multicast a que se juntar (IPv4) |
| `interface` | string | qualquer | O endereço IPv4 local em que se juntar aos grupos |
| `reuse` | booleano | `true` | Compartilha a porta com um programa que já escuta nela (`SO_REUSEADDR`) |
| `respond` | booleano | `false` | Responde ao que chega |
| `response` | [`Payload`](#type-payload) | nenhuma | A resposta; necessária com `respond` |
| `respond_delay_ms` | número | `0` | Espera este tempo antes de responder |
| `match_contains` | string | nenhuma | Responde só a datagramas cujo texto contém isto |

No máximo 512 pares são listados; os posteriores não são adicionados.

**Resultado**: [`JobInfo`](#type-jobinfo).

**Erros**: `node.bind_invalid`, `broadcast.port_shared` (a porta está ocupada
e `reuse` está desligado), `broadcast.interface_invalid`,
`broadcast.not_multicast`, `broadcast.join_failed`,
`broadcast.reply_missing`, `node.osc_address`, `hex.*`, falhas de bind. A
tarefa termina com `wait.receive_failed` se o socket não pode mais receber.

```bash
invoke broadcast_send '{"config":{"mode":"list","target":"127.0.0.1:9000, 127.0.0.1:9001","payload":{"kind":"text","text":"PING"}}}' \
  | jq '{packets, errors}'
```

## Degradação {#impairment}

Um retransmissor entre um cliente e o servidor dele que atrasa, descarta, duplica,
corrompe, reordena ou limita o que passa, sobre UDP ou TCP. Veja
[degradação](../tools/impairment.md).

### netsim_start {#netsim_start}

Inicia um retransmissor: o que chega em `listen` segue para `target`, e as respostas
voltam pelo mesmo caminho, ambos degradados pelo perfil. Os contadores dele chegam como
[`netsim://stat`](events.md#event-netsim-stat) a cada 250 ms. *Inicia uma tarefa*
(`netsim`, `params.listen`, `params.target` e `params.protocol` para TCP).

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `config` | objeto | sim | Abaixo |

| Campo de `config` | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `listen` | string | — | `IP:port` em que o retransmissor escuta; aponte o cliente para cá |
| `target` | string | — | `IP:port` do servidor real, ou `host:port` — um nome de host é consultado uma vez, quando o retransmissor inicia |
| `profile` | [`ImpairProfile`](#type-impairprofile) | — | O que fazer com o tráfego |
| `seed` | número | nova | A semente dos sorteios: a mesma semente e o mesmo tráfego dão os mesmos descartes |
| `protocol` | string | `udp` | `udp` (datagramas) ou `tcp` (fluxos) |

**Resultado**: [`JobInfo`](#type-jobinfo).

**Erros**: `node.range` (um valor do perfil fora do intervalo, com
`min`, `max` e o campo), `node.too_long`, `node.bind_invalid`,
`transport.target_invalid`, `transport.dns` (um nome de destino que não pode ser
encontrado), falhas de bind. A tarefa termina com
`wait.receive_failed` se um socket não pode mais receber.

### netsim_set_profile {#netsim_set_profile}

Um retransmissor em execução degrada com outro perfil a partir de agora, sem fechar os
sockets.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `jobId` | número | sim | A tarefa do retransmissor |
| `profile` | [`ImpairProfile`](#type-impairprofile) | sim | O novo perfil |

**Resultado**: null.

**Erros**: `netsim.not_running`, `node.range`, `node.too_long`.

```bash
invoke netsim_start '{"config":{"listen":"127.0.0.1:9010","target":"127.0.0.1:9000","profile":{"latency_ms":80,"jitter_ms":20,"loss":0.02}}}'
```

## Tempestade e scanner {#storm-scanner}

::: danger
Uma tempestade carrega um destino tão forte quanto você pedir, e uma varredura sonda todas as portas de um
intervalo. Aponte-as só para hosts pelos quais você é responsável.
:::

### storm_start {#storm_start}

Envia uma carga constante de datagramas UDP ou conexões TCP a um destino. Os
contadores dela chegam como [`storm://stat`](events.md#event-storm-stat) a cada 250 ms.
*Inicia uma tarefa* (`storm`, `params.protocol`, `params.target`, `params.rate`).

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `config` | objeto | sim | Abaixo |

| Campo de `config` | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `target` | string | — | `IP:port` ou `host:port`; um nome é consultado uma vez, quando a tarefa inicia |
| `protocol` | string | — | `udp`: datagramas; `tcp`: uma conexão por unidade que escreve a carga útil e fecha (cada conexão pode levar 500 ms) |
| `size` | número | — | Bytes da carga útil, mantido entre 1 e 65.507 |
| `rate` | número | — | Unidades por segundo, limitado a 10.000.000, em um cronograma: a unidade *n* é devida *n* / `rate` segundos depois do início, e cada despertar envia o que é devido (no máximo 256; um cronograma mais atrasado pula as unidades mais antigas); 0 envia o mais rápido que puder |
| `duration_s` | número | `0` | Para depois deste número de segundos; 0 (ou um número negativo) até ser parada |

**Resultado**: [`JobInfo`](#type-jobinfo).

**Erros**: `transport.target_invalid`, `transport.dns` e, para UDP, uma falha de bind ou
de conexão (`transport.*`) quando o socket não pode ser aberto de modo algum. Os envios
que o destino recusa são contados como `errors` nos eventos; o que esta máquina
não pôde enviar (sem portas, sockets ou buffers) é contado como `local`, nunca
como erro do destino.

### scan_start {#scan_start}

Tenta uma conexão TCP em todas as portas de um intervalo e informa as abertas,
com o que o serviço diz primeiro quando pedido. As portas abertas chegam como
[`scan://open`](events.md#event-scan-open), o progresso como
[`scan://progress`](events.md#event-scan-progress). *Inicia uma tarefa* (`scan`,
`params.host`, `params.from`, `params.to`).

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `config` | objeto | sim | Abaixo |

| Campo de `config` | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `host` | string | — | Um nome de host ou um endereço |
| `port_start`, `port_end` | número | — | O intervalo, ambos incluídos; dados na ordem errada, são trocados |
| `concurrency` | número | `256` | Tentativas ao mesmo tempo, 1 a 1024 |
| `timeout_ms` | número | `600` | Por porta, 50 a 10.000 |
| `grab_banner` | booleano | `false` | Lê até 256 bytes que o serviço envia em 400 ms depois de conectar |

**Resultado**: [`JobInfo`](#type-jobinfo).

**Erros**: `scan.host_required`.

```bash
invoke scan_start '{"config":{"host":"127.0.0.1","port_start":8000,"port_end":9100,"grab_banner":true}}'
```

## Inspetor {#inspector}

O Inspetor registra o que as ferramentas enviam e recebem, como quadros, enquanto a
captura está armada. Em um servidor há um Inspetor para todas as páginas e
scripts. Veja [o Inspetor](../tools/inspector.md).

### inspect_set_enabled {#inspect_set_enabled}

Arma ou desarma a captura. Enquanto desarmada, nada é registrado.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `enabled` | booleano | sim | Arma (`true`) ou desarma |

**Resultado**: [`CaptureStats`](#type-frame).

### inspect_stats {#inspect_stats}

Os contadores da captura. Sem argumentos.

**Resultado**: [`CaptureStats`](#type-frame).

### inspect_snapshot {#inspect_snapshot}

Os quadros mais novos, do mais antigo ao mais novo.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `limit` | número | sim | Quantos, 1 a 8192 |

**Resultado**: [`Frame`](#type-frame)`[]`, sem os bytes deles (veja
[`inspect_payload`](#inspect_payload)).

### inspect_clear {#inspect_clear}

Esvazia a captura e os contadores dela. Sem argumentos.

**Resultado**: [`CaptureStats`](#type-frame).

### inspect_export {#inspect_export}

Escreve todos os quadros guardados em `capture-<ms>.jsonl` ou `capture-<ms>.txt` na
pasta de dados. Em `jsonl`, cada linha é um quadro com os bytes que ele guarda em
`data`, em base64; `txt` é para leitura, com um despejo hex de cada quadro.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `format` | string | sim | `txt`; qualquer outra coisa escreve `jsonl` |

**Resultado**: string, o caminho escrito.

**Erros**: `inspect.empty`, `file.io`.

### inspect_payload {#inspect_payload}

Os bytes que um quadro guarda, além da prévia de 256 bytes que o lote dele carregava.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `seq` | número | sim | O número do quadro |

**Resultado**: `{ "seq", "bytes", "kept", "dump", "hex" }` — `bytes` o tamanho do
quadro, `kept` quantos deles são guardados (até 256 KiB), `dump` cada linha como
`offset  hex  |ascii|`, `hex` o hex simples que um reenvio envia.

**Erros**: `inspect.frame_gone` (quadros mais novos tomaram o lugar dele),
`inspect.no_payload` (só o tamanho foi registrado).

```bash
invoke inspect_set_enabled '{"enabled":true}'
invoke inspect_snapshot '{"limit":20}' | jq '.[] | {seq, proto, dir, summary}'
```

## Biblioteca de sinais {#signals}

A biblioteca é `signals.json` na pasta de dados. Ela é só armazenamento: um sinal
é enviado com o comando do transporte dele (`osc_send`, `broadcast_send`,
`http_request`, `mqtt_publish` ou `mqtt_publish_once`). Veja
[sinais](../tools/signals.md) e [arquivos](../reference/files.md#signals-json).

### signals_load {#signals_load}

Lê a biblioteca. Quando o arquivo não existe, o conjunto inicial é escrito
primeiro. Sem argumentos.

**Resultado**: `{ "path": string, "library": library, "seeded": boolean, "stamp": string }` —
`seeded` é verdadeiro quando o conjunto inicial acabou de ser escrito; `stamp` nomeia o que o
arquivo continha, para o `base` do próximo salvamento. A biblioteca é
`{ "version", "signals": [...], "folders": [...] }`: `version` 2 (um arquivo da versão 1
volta como está), `folders` omitido quando não há nenhuma. Cada sinal
tem `id`, `name`, `group` (a pasta dele, `"A/B"`; vazio para nenhuma), `note` e
`body`.

**Erros**: `signals.json_invalid` (com `path`, `line`, `column`; o arquivo
nunca é substituído), `file.io`.

### signals_save {#signals_save}

Substitui o arquivo inteiro da biblioteca, por meio de um arquivo temporário na mesma pasta.
Um arquivo que existe e não é lido como uma biblioteca é deixado como está. Com
`base`, um arquivo alterado desde que foi lido — por outra página de um servidor, ou por uma
pessoa em um editor — também é deixado como está, e o salvamento é recusado, para que a
alteração dela não se perca sob uma lista que nunca a viu.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `library` | objeto | sim | `{ version, signals, folders }` como `signals_load` o devolve |
| `base` | string | não | O `stamp` com que a biblioteca foi lida (ou que o último salvamento devolveu); `""` quando não havia arquivo |

**Resultado**: string, o caminho escrito; com `base`, `{ "path", "stamp" }` — o stamp do que o
arquivo contém agora, o `base` do próximo salvamento.

**Erros**: `signals.json_invalid` (o arquivo que está agora no disco não é lido, com
`path`, `line`, `column`; nada é escrito), `file.changed` (com `path`: o
arquivo já não contém `base`; nada é escrito — leia-o de novo),
`signals.encode`, `file.io`.

O `body` de um sinal conforme o `transport` dele:

| `transport` | Campos |
| --- | --- |
| `osc` | `target`, `address`, `args` ([`OscArg`](#type-oscarg)`[]`) |
| `udp` | `target`, `payload`: `{ "kind": "text", "text" }` ou `{ "kind": "hex", "hex" }` |
| `http` | `request` ([`HttpRequest`](#type-httprequest)) |
| `mqtt` | `broker` (`host:port`), `topic`, `payload`, `qos`, `retain` |

```bash
invoke signals_load | jq '.library.signals[] | {name, transport: .body.transport}'
```

## Emuladores {#emulators}

Um emulador é o Signal Lab fazendo o papel do outro lado: uma API HTTP, um dispositivo OSC, UDP ou
TCP, um broker MQTT. O documento dele — `name`, `bind`, `protocol`, as
regras do protocolo e uma `outage` opcional — está descrito em
[emuladores](../tools/emulators.md). A biblioteca é `emulators.json` na
pasta de dados.

### emulators_load {#emulators_load}

Lê a biblioteca de emuladores. Quando o arquivo não existe, o conjunto inicial é
escrito primeiro. Sem argumentos.

**Resultado**: `{ "path", "library": { "version": 1, "emulators": [{ "id", "note", "emulator" }] }, "seeded", "stamp" }` —
`stamp` nomeia o que o arquivo continha, para o `base` do próximo salvamento.

**Erros**: `emulators.json_invalid` (com `path`, `line`, `column`; nunca
substituído), `file.io`.

### emulators_save {#emulators_save}

Substitui a biblioteca de emuladores inteira, por meio de um arquivo temporário. Um arquivo que
existe e não é lido como uma biblioteca é deixado como está; com `base`, também o é um
arquivo alterado desde que foi lido, como em [`signals_save`](#signals_save).

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `library` | objeto | sim | `{ version, emulators }` como `emulators_load` o devolve |
| `base` | string | não | O `stamp` com que a biblioteca foi lida (ou que o último salvamento devolveu); `""` quando não havia arquivo |

**Resultado**: string, o caminho escrito; com `base`, `{ "path", "stamp" }`.

**Erros**: `emulators.json_invalid` (o arquivo que está agora no disco não é lido;
nada é escrito), `file.changed` (o arquivo já não contém `base`; nada
é escrito), `emulators.encode`, `file.io`.

### emulator_check {#emulator_check}

Se um emulador iniciaria: tudo o que [`emulator_start`](#emulator_start)
verifica antes de fazer o bind.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `emulator` | objeto | sim | O documento do emulador |
| `params` | objeto de strings | não | Valores que os modelos dele leem como parâmetros |

**Resultado**: null quando iniciaria.

**Erros**: `emulator.*`; `node.*` para um campo ausente, fora do intervalo, longo demais
ou malformado (`node.required`, `node.range`, `node.too_long`,
`node.bind_invalid`, `node.target_invalid`, `node.method_invalid`…);
`param.unknown`, `template.*`, `osc.pattern_*`, `regex.invalid`,
`hex.invalid`. Cada um tem `rule`, `retained` ou `response` em `params` quando o
problema está em um deles.

### emulator_start {#emulator_start}

Inicia um emulador como uma tarefa própria. O socket dele está aberto quando o comando
retorna. O que ele recebe e responde chega como
[`emulator://activity`](events.md#event-emulator-activity) a cada 200 ms quando
algo mudou. *Inicia uma tarefa* (`emulator`, `params.name`,
`params.protocol`, `params.local` e `params.source` quando `source` é
dado).

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `emulator` | objeto | sim | O documento do emulador |
| `params` | objeto de strings | não | Valores que os modelos dele leem como parâmetros |
| `seed` | número | não | A semente dele, 0 a 9007199254740991; padrão: uma nova |
| `source` | string | não | A entrada da biblioteca de onde ele vem, guardada na tarefa como `params.source` |

**Resultado**: [`JobInfo`](#type-jobinfo).

**Erros**: os de [`emulator_check`](#emulator_check), `seed.range`,
`transport.address_in_use` e as demais falhas de bind.

### emulator_exchanges {#emulator_exchanges}

O que um emulador em execução recebeu e respondeu. Ele guarda as últimas 500
trocas.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `jobId` | número | sim | A tarefa do emulador |
| `after` | número | não | Só as trocas numeradas acima disto; padrão 0 |
| `limit` | número | não | No máximo este tanto, 1 a 500; padrão 500 |

**Resultado**

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa |
| `name`, `protocol`, `local` | string | O emulador, o protocolo dele e o endereço em que escuta |
| `counts` | objeto | `total`, `unmatched` (nenhuma regra a tomou), `failed`, `down` (chegou enquanto estava fora do ar: a queda dele ou [`emulator_down`](#emulator_down)), `hits` (por regra), `missed` (MQTT: mensagens que um cliente muito atrasado não recebeu; omitido enquanto 0) e `refused` (HTTP, TCP, MQTT: conexões fechadas ao chegar, além do limite; omitido enquanto 0) |
| `forced` | string | `unavailable`, `reset` ou `timeout` enquanto está derrubado; omitido nos demais casos |
| `exchanges` | objeto[] | Cada uma: `seq`, `ts`, `from`, `request`, `rule` (começando em 1; omitido quando nenhuma a tomou), `reply`, `status`, `fault`, `ms`, `error`, `frame`, `down` e `data` (a requisição como os modelos a leem) |

**Erros**: `emulator.not_running`.

### emulator_down {#emulator_down}

Derruba um emulador em execução até ele ser reativado, seja o que for que o cronograma de
queda dele diga, ou o traz de volta. Enquanto fora do ar, um emulador HTTP recebe cada
requisição com `fault`, um dispositivo TCP e um broker MQTT derrubam as conexões
e recusam as novas, e dispositivos OSC e UDP não respondem nada.

| Argumento | Tipo | Obrigatório | Significado |
| --- | --- | --- | --- |
| `jobId` | número | sim | A tarefa do emulador |
| `down` | booleano | sim | Fora do ar (`true`) ou no ar |
| `fault` | string | não | O que as requisições HTTP encontram: `unavailable` (503, sem `Retry-After`: não se sabe quando volta), `reset` (a conexão se fecha), `timeout` (nenhuma resposta); padrão `unavailable` |

**Resultado**: null.

**Erros**: `emulator.not_running`.

```bash
invoke emulator_exchanges '{"jobId":5,"after":0}' | jq '.counts, (.exchanges[] | {request, rule, status})'
```

## Tipos compartilhados {#types}

### JobInfo {#type-jobinfo}

O que um comando que inicia uma tarefa devolve, e o que [`jobs_list`](#jobs_list)
lista: `id`, `kind`, `label` (em inglês, para logs), `params` (os valores que o
rótulo nomeia; omitido quando nenhum) e `started_ms`. Veja
[tarefas](index.md#jobs).

### OscArg {#type-oscarg}

Um argumento OSC, o tipo e o valor dele:

| `type` | `value` | Tag OSC |
| --- | --- | --- |
| `int` | inteiro de 32 bits | `i` |
| `float` | número, enviado como float de 32 bits | `f` |
| `str` | string | `s` |
| `long` | inteiro de 64 bits | `h` |
| `double` | número, de 64 bits | `d` |
| `bool` | `true` ou `false` | `T` ou `F` |
| `blob` | array de bytes, `[222, 173]` | `b` |
| `nil` | nenhum: `{ "type": "nil" }` | `N` |
| `time` | uma etiqueta de tempo, os 64 bits dela como texto decimal: `"1"` é "imediatamente" | `t` |
| `char` | um caractere, `"A"` | `c` |
| `color` | `[red, green, blue, alpha]` | `r` |
| `midi` | `[port, status, data1, data2]` | `m` |
| `impulse` | nenhum | `I` |
| `array` | uma lista de OscArg | `[` … `]` |

Os seis últimos são o que as mensagens recebidas carregam (o `osc://message` do monitor OSC);
um comando também pode enviá-los.

### HttpRequest {#type-httprequest}

| Campo | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `method` | string | — | `GET`, `POST`… |
| `url` | string | — | `http://` ou `https://` |
| `headers` | `[name, value][]` | `[]` | Cabeçalhos da requisição |
| `body` | string ou null | null | O corpo |
| `timeout_ms` | número | `10000` | Para a troca inteira, corpo incluído: 1 a 120.000; fora disso o comando falha com `node.range` (campo `timeout_ms`) |
| `auth` | objeto | nenhuma | `{ "scheme": "basic", "username", "password" }`, `{ "scheme": "digest", "username", "password" }` ou `{ "scheme": "bearer", "token" }` |

Até 10 redirecionamentos são seguidos. As credenciais e os cookies digitados para um host
nunca passam a outro. Uma requisição Digest responde ao desafio 401 do servidor
e envia de novo.

### HttpResponse {#type-httpresponse}

| Campo | Tipo | Significado |
| --- | --- | --- |
| `ok` | booleano | Um status 2xx, e o corpo inteiro chegou |
| `status`, `status_text` | número, string | O status; 0 e vazio sem resposta |
| `latency_ms` | número | Até o corpo inteiro chegar |
| `headers` | `[name, value][]` | Cabeçalhos da resposta |
| `body` | string | O corpo como texto, no máximo 256 KiB |
| `body_bytes` | número | O tamanho total do corpo |
| `truncated` | booleano | `body` foi cortado em 256 KiB |
| `error` | string ou null | Por que não houve resposta, ou por que o corpo parou antes do fim (então `status`, `headers` e o corpo até ali também estão presentes), cada camada da causa |
| `cause` | string ou null | Que tipo de falha: `refused`, `timeout`, `dns`, `unreachable`, `reset`, `address_in_use`, `address_unavailable`, `denied`, `tls`, `target_invalid`, `exhausted` (esta máquina ficou sem sockets ou portas locais), `failed` — o mesmo que os códigos `transport.*` |
| `digest` | objeto | Só uma requisição Digest que encontrou um 401; omitido nos demais casos. `challenged`: o desafio foi respondido e a requisição enviada de novo. `error`: por que não pôde ser, um `EngineError` (`http.digest_not_offered`, `http.digest_unsupported`, `http.digest_invalid`, `http.digest_other_origin`) ou null |

### Payload {#type-payload}

O que um datagrama de broadcast ou de descoberta carrega:
`{ "kind": "osc", "address", "args" }`, `{ "kind": "text", "text" }` (enviado como
está, sem zero final) ou `{ "kind": "hex", "hex" }` (`de ad be ef`,
`deadbeef`, `0xDE,0xAD` — tudo o que não for dígito hex é ignorado).

### MqttConfig {#type-mqttconfig}

| Campo | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `host` | string | — | O nome ou endereço do broker |
| `port` | número | — | Em geral 1883 |
| `client_id` | string | — | Não vazio; outra conexão com o mesmo id é derrubada pelo broker |
| `username`, `password` | string | vazio | `username` é enviado quando não vazio; `password` só junto com um `username` |
| `keep_alive_s` | número | `60` | Os pings saem na metade dele; 0: nenhum |
| `clean_session` | booleano | `true` | A flag do CONNECT |
| `will` | objeto ou null | null | `{ topic, payload, qos, retain }`, publicada pelo broker se a conexão se perde |
| `subscribe` | `{ filter, qos }[]` | `[]` | Assinados assim que a conexão está ativa |

### WsConfig {#type-wsconfig}

| Campo | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `url` | string | — | `ws://` ou `wss://` (`wss://` confia no que o sistema confia para HTTPS) |
| `headers` | `[name, value][]` | `[]` | Enviados com a requisição de upgrade |
| `protocols` | string[] | `[]` | Subprotocolos a oferecer, em ordem de preferência |
| `timeout_ms` | número | `10000` | Para a conexão, o TLS e o upgrade juntos |

As mensagens têm no máximo 16 MiB nos dois sentidos.

### ImpairProfile {#type-impairprofile}

Todo campo é opcional; o que é omitido não faz nada. As probabilidades vão de 0
a 1.

| Campo | Intervalo | Significado | UDP | TCP |
| --- | --- | --- | --- | --- |
| `name` | no máximo 60 caracteres | Um rótulo para a linha do tempo e o relatório | sim | sim |
| `latency_ms` | 0 a 60.000 | Atraso somado a tudo | sim | sim |
| `jitter_ms` | 0 a 60.000 | Até este tanto a mais, sorteado a cada vez | sim | sim |
| `loss` | 0 a 1 | Um datagrama é descartado | sim | — |
| `duplicate` | 0 a 1 | Um datagrama é enviado duas vezes | sim | — |
| `corrupt` | 0 a 1 | Um bit de um datagrama é invertido | sim | — |
| `reorder` | 0 a 1 | Um datagrama é retido de modo que os seguintes o ultrapassem | sim | — |
| `rate_kbps` | 0, ou 8 a 10.000.000 | Limite de largura de banda, quilobits por segundo; 0: nenhum | sim | sim |
| `burst_start` | 0 a 1 | Um datagrama inicia uma rajada de perdas | sim | — |
| `burst_length` | 1 a 1000 | Datagramas que uma rajada dura em média (necessário com `burst_start`) | sim | — |
| `offline` | `true` ou `false` | Nada passa | sim | sim |
| `reset` | 0 a 1 | Um trecho de um fluxo redefine a conexão dele | — | sim |
| `stall` | 0 a 1 | Um trecho de um fluxo deixa a conexão dele semiaberta | — | sim |

### Frame e CaptureStats {#type-frame}

Um `Frame` é um pacote, requisição ou mensagem capturado:

| Campo | Significado |
| --- | --- |
| `seq` | O número dele, crescente |
| `ts` | Quando, em milissegundos desde 1970 |
| `proto` | `osc`, `udp`, `tcp`, `http`, `mqtt`, `ws`… |
| `dir` | `tx` (enviado) ou `rx` (recebido) |
| `source` | A ferramenta que o capturou: `osc-monitor`, `broadcast`, `netsim`… |
| `job_id` | A tarefa dele, ou null |
| `local`, `remote` | Os endereços: `IP:port` deste lado e do outro (uma URL ou um broker para HTTP, WebSocket e MQTT). O `local` de um quadro retransmitido é o endereço em que o retransmissor escuta e o `remote` para onde o quadro ia; o trecho termina o `verdict` dele (`· client→target`, `· target→client`) |
| `bytes` | O tamanho dele |
| `summary` | Uma linha |
| `detail` | Uma decodificação em várias linhas, ou null |
| `hex` | Um despejo hex dos primeiros 256 bytes, ou null |
| `verdict` | O que foi feito dele — `dropped`, `sampled`, um status — ou null |
| `kept` | De `bytes`, quantos são guardados (até 256 KiB); 0 quando só o tamanho foi registrado |
| `publish` | Só uma publicação MQTT: `{ broker, topic, qos, retain, text }` — o broker como `host:port` e se os bytes guardados, que são a carga útil da mensagem, são texto UTF-8. Ausente em todos os outros quadros |

`CaptureStats`: `enabled`, `total` (quadros registrados), `bytes`, `skipped`
(registrados mas nunca enviados à interface), `buffered` (quadros guardados),
`capacity` (8192), `held` (bytes de carga útil guardados) e `held_limit` (64 MiB). Os
quadros mais antigos cedem lugar além de qualquer dos limites.

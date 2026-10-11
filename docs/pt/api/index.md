---
title: API HTTP
description: Controle um servidor do Signal Lab a partir de scripts, de CI e de outras ferramentas, com os mesmos comandos e eventos que a própria interface dele usa.
---

# A API HTTP

Para controlar o Signal Lab a partir de um script, de um pipeline de CI ou de outra ferramenta, fale com um
servidor do Signal Lab (`signal-lab-server`, ou a imagem Docker) por HTTP. A
interface que o servidor mostra em um navegador usa exatamente esta API: cada botão é uma
chamada a `/api/invoke/<command>`, cada número ao vivo chega em `/api/events`. Então
tudo o que uma pessoa pode fazer na página do servidor, um script também pode.

O aplicativo de desktop não tem API HTTP: a janela dele alcança o motor dentro do app.
Para automatizar em um desktop, execute um servidor na mesma máquina (veja
[o servidor](../server/index.md)) ou use a linha de comando
[`signallab`](../automation/cli.md).

## Endpoints {#endpoints}

| Método e caminho | O que faz | Token |
| --- | --- | --- |
| `GET /api/health` | Se o servidor responde, a versão dele, se ele quer um token | não precisa |
| `POST /api/invoke/<command>` | Executa um comando do motor: argumentos JSON na entrada, resultado JSON na saída ([comandos](commands.md)) | precisa |
| `POST /api/run` | Executa um experimento até o fim: o resultado, ou as etapas dele como linhas ([execuções](run.md)) | precisa |
| `GET /api/events` | WebSocket de todos os eventos do motor ([eventos](events.md)) | precisa |
| `GET /api/files?path=…` | Um arquivo que o motor escreveu na pasta de dados, como download | precisa |
| `GET /api/openapi.json` | Esta API descrita em OpenAPI 3.1 | precisa |
| `GET /login`, `POST /login` | A página e o formulário de entrada para navegadores | não precisa |
| `POST /logout` | Encerra a sessão de um navegador | uma sessão |

Qualquer outro caminho em `/api/` responde `404` com o código `api.not_found`; um
método que um caminho não aceita (`GET /api/invoke/…`) é `405`, com corpo vazio.
Todo o resto é a interface; em um servidor com token, um navegador sem
sessão é enviado primeiro a `/login`.

## URL base {#base-url}

Um servidor escuta em `http://127.0.0.1:1430`, a menos que se diga outra coisa com
`--listen` (ou `SIGNALLAB_LISTEN`). A imagem Docker escuta em todas as placas de
rede, `0.0.0.0:1430`. Os exemplos destas páginas usam:

```bash
SERVER=http://127.0.0.1:1430
```

O servidor fala HTTP simples. Para HTTPS, ponha na frente dele um proxy reverso que termine
o TLS e inicie o servidor com `--secure-cookie`.

## Autenticação {#authentication}

Um servidor iniciado sem token escuta só no loopback e não precisa de
autenticação: qualquer pessoa naquela máquina pode usá-lo. Um servidor que outros
possam alcançar sempre tem um token, e então toda requisição, exceto `/api/health` e
`/login`, precisa trazê-lo.

**Scripts** enviam o token no cabeçalho `Authorization`:

```bash
TOKEN=$(cat token.txt)
curl -fsS "$SERVER/api/invoke/jobs_list" -X POST \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json"
```

O cabeçalho precisa ser exatamente `Bearer`, um espaço e o token. Um token ausente ou
errado é `401` com o código `auth.required`.

**Navegadores** entram uma vez em `/login` com o token e recebem um cookie de sessão,
`signallab_session`: `HttpOnly`, `SameSite=Strict`, mantido por 7 dias e `Secure`
quando o servidor executa com `--secure-cookie`. `POST /logout` o encerra. Um token
errado no formulário de entrada custa um segundo antes da resposta, o que torna
a adivinhação lenta. As sessões vivem na memória do servidor: um reinício desconecta todos
os navegadores, enquanto scripts com o token não são afetados. O servidor guarda
no máximo 1024 sessões; além disso, a mais antiga sai.

De onde vem o token é questão da configuração do servidor (`--token-file`,
`SIGNALLAB_TOKEN`, ou o arquivo `token` que `--generate-token` cria na pasta de
dados): veja [segurança do servidor](../server/security.md). Um token tem pelo menos 24
caracteres e nenhum espaço; `signal-lab-server token` imprime um novo.

::: warning
Quem tem o token pode fazer o servidor enviar tráfego. Mantenha o arquivo em que ele está
legível só por você e nunca o ponha em uma URL — o servidor não lê um
token ali de qualquer modo.
:::

## Host e Origin {#host-origin}

Duas verificações executam antes de qualquer outra coisa, em todo caminho, `/api/health` incluído.

**`Host`.** O cabeçalho `Host` precisa nomear este servidor:

- Os nomes de loopback sempre passam: `localhost`, nomes terminados em `.localhost`,
  `127.x.x.x` e `[::1]`.
- Os nomes dados com `--allowed-host` (ou `SIGNALLAB_ALLOWED_HOSTS`) passam.
- Um servidor com token e sem `--allowed-host` responde a qualquer nome.

Qualquer outra coisa é `403` com `auth.host`. Acesse o servidor por um nome que ele
aceita: `curl` envia o host da URL que você der a ele.

**`Origin`.** Uma requisição que altera algo (qualquer método além de `GET` e
`HEAD`) e o upgrade de WebSocket precisam vir da página do próprio servidor quando
trazem um cabeçalho `Origin`: o host e a porta dele precisam ser iguais a `Host`. Caso contrário,
a resposta é `403` com `auth.origin`; `Origin: null` também é recusado. Scripts
e `curl` não enviam `Origin` e passam nesta verificação — ainda precisam do token.

**Só JSON.** `POST /api/invoke/…` e `POST /api/run` aceitam
`Content-Type: application/json` (parâmetros como `; charset=utf-8` são
aceitos). Qualquer outra coisa é `415` com `command.json_required`. Uma página web de
outro site não consegue enviar isso sem perguntar antes ao servidor, e o servidor
nunca concorda.

## Chamada de um comando {#invoke}

```http
POST /api/invoke/<command>
Content-Type: application/json

{ "argument": "value", … }
```

- O corpo é um objeto JSON com os argumentos do comando. Um comando sem
  argumentos aceita `{}` ou um corpo vazio (o cabeçalho `Content-Type` ainda
  é necessário).
- Os nomes dos argumentos são camelCase, como a interface os envia: `jobId`,
  `nodeId`. Os objetos passados como argumento (`config`, `request`, `document`,
  `library`…) mantêm os nomes de campo que o motor escreve, que em geral são
  snake_case: `timeout_ms`, `port_start`.
- Um argumento que um comando não conhece é um erro, nunca ignorado: `422` com
  `command.args_invalid` nomeando o comando, e as palavras do analisador em
  `detail`. O mesmo vale para um obrigatório que foi omitido. Um comando sem argumentos não
  lê o corpo de modo algum.
- Um argumento opcional pode ser omitido ou enviado como `null`.
- A resposta é `200` com o resultado do comando como JSON. Um comando que não tem
  nada a devolver responde `null`.

Todos os comandos, com seus argumentos e resultado, estão em [comandos](commands.md).

## Erros {#errors}

| Status | Quando | Corpo |
| --- | --- | --- |
| `200` | O comando executou; `/api/run` iniciou a execução | O resultado |
| `400` | O corpo não é JSON; uma requisição de execução não pode ser lida | `EngineError`: `command.args_invalid`, `api.run_invalid`, `api.run_source` |
| `400` | `/api/files` sem `path` | Texto simples do servidor web, não um `EngineError` |
| `401` | Nenhum token, ou um errado | `EngineError`: `auth.required` |
| `403` | Um `Host` ou `Origin` que o servidor recusa | `EngineError`: `auth.host`, `auth.origin` |
| `404` | Nenhum caminho de API desse tipo; um arquivo que não está na pasta de dados | `EngineError`: `api.not_found`, `file.not_found` |
| `405` | Um método que o caminho não aceita | Vazio |
| `413` | Um corpo de requisição acima de 24 MiB | Texto simples do servidor web, não um `EngineError` |
| `413` | Um download acima de 256 MiB | `EngineError`: `file.too_large` |
| `415` | Não é `application/json` | `EngineError`: `command.json_required` |
| `422` | O comando falhou, ou a execução não pôde começar | `EngineError`: qualquer código do motor |
| `500` | Um arquivo não pôde ser lido | `EngineError`: `file.io` |

Um nome de comando desconhecido é `422` com `command.unknown`.

Toda falha que o motor informa tem uma só forma, o `EngineError`:

```json
{
  "code": "transport.refused",
  "params": { "target": "http://127.0.0.1:8080/" },
  "node": "request",
  "field": { "key": "url" },
  "detail": "error sending request for url (http://127.0.0.1:8080/): tcp connect error: Connection refused (os error 111)"
}
```

| Campo | O que é |
| --- | --- |
| `code` | O que deu errado: um identificador estável. Todo código e a mensagem dele estão listados em [mensagens de erro](../reference/errors.md), agrupados pela parte antes do ponto (por exemplo [`transport`](../reference/errors.md#transport)) |
| `params` | Os valores que a mensagem nomeia, todos como strings. Omitido quando não há nenhum |
| `node` | O nó do experimento a que se refere. Omitido quando não há nenhum |
| `field` | O campo a que se refere: `key` (nomeado em [campos](../reference/errors.md#fields)) e `index`, começando em 1, para campos repetidos como um cabeçalho. Omitido quando não há nenhum |
| `detail` | As próprias palavras do sistema operacional, de um analisador ou de uma biblioteca, em inglês. Omitido quando não há nenhuma |

Decida por `code`, nunca por `detail`. Os valores de segredos que uma execução ou um único envio
usa são mascarados (`••••`) em todo erro que ele informa.

Uma requisição que alcança o servidor dela mas recebe um status de erro, ou nenhum,
não é um comando que falhou: `http_request` responde `200` com a resposta, e
`ok`, `error` e `cause` dizem o que aconteceu. Veja
[`http_request`](commands.md#http_request).

## Limites {#limits}

| O quê | Limite | No limite |
| --- | --- | --- |
| Um corpo de requisição | 24 MiB | `413` |
| Um documento de experimento | 4 MiB | `file.too_large` |
| Um download de `/api/files` | 256 MiB | `413`, `file.too_large` |
| A duração de uma execução | 300 s | A execução falha com `run.timeout` |
| O formulário de feedback (`feedback_send`) | 15 MiB no total | `feedback.too_large` |
| Eventos esperando por um WebSocket | 4096 | Ele recebe [`server://lagged`](events.md#event-server-lagged) com quantos perdeu |

## Eventos {#events}

`GET /api/events` elevado a um WebSocket transmite todo evento que o motor
envia — as etapas de uma execução, o fim de uma tarefa, as mensagens de um monitor, os
quadros do Inspetor — a todos os clientes conectados, como mensagens de texto:

```json
{ "event": "job://ended", "payload": { "job_id": 7, "kind": "storm", "error": null } }
```

Ele segue as mesmas regras de token e `Origin` do resto. Todo canal e a carga útil dele
estão em [eventos](events.md).

## Arquivos {#files}

`GET /api/files?path=<path>` baixa um arquivo que o motor escreveu na pasta de
dados do servidor: um relatório de execução (`report_path` do resultado de uma execução), uma exportação do
experimento ou do Inspetor, a biblioteca de sinais ou de emuladores. `path` é o
caminho que o motor deu a você, no servidor (codificado como URL):

```bash
curl -fsS -G "$SERVER/api/files" --data-urlencode "path=/data/runs/run-1759600000000-3.json" \
  -H "Authorization: Bearer $TOKEN" -o report.json
```

- Só os arquivos dentro da pasta de dados são servidos. Qualquer outra coisa, uma pasta ou um
  arquivo que não existe, é `404` com `file.not_found`.
- A resposta é `application/octet-stream` com
  `Content-Disposition: attachment`. Os caracteres do nome do arquivo que não sejam
  letras, dígitos, `.`, `_` e `-` viram `_`.
- Um arquivo acima de 256 MiB é `413` com `file.too_large`.

O que a pasta de dados contém está em [arquivos e pastas](../reference/files.md).

## Saúde {#health}

`GET /api/health` é aberto: não precisa de token, só de um `Host` que o servidor aceite.

```bash
curl -fsS "$SERVER/api/health"
```

```json
{ "status": "ok", "version": "[[version]]", "auth": true }
```

`auth` diz se as requisições precisam de um token. `signal-lab-server healthcheck`
consulta o mesmo endereço em que o servidor escuta e sai com `0` quando ele responde;
a verificação de saúde da imagem Docker o executa.

## Descrição OpenAPI {#openapi}

`GET /api/openapi.json` descreve esta API em OpenAPI 3.1: os endpoints, todos os
comandos com seus argumentos, e a requisição e o resultado da execução. Precisa do token
como o resto de `/api/`. Estas páginas são a referência completa; onde as duas
diferem, estas páginas seguem o motor.

## Tarefas {#jobs}

O trabalho de longa duração — um monitor, um gerador, uma rajada, um retransmissor, uma conexão, um
emulador, uma execução — é uma tarefa. Um comando que inicia uma devolve o `JobInfo` dela
assim que ela executa:

```json
{ "id": 4, "kind": "osc-monitor", "label": "OSC monitor 0.0.0.0:9000", "params": { "bind": "0.0.0.0:9000" }, "started_ms": 1759600000000 }
```

| Campo | O que é |
| --- | --- |
| `id` | O número da tarefa, único enquanto o servidor executa; outros comandos o aceitam como `jobId` ou `id` |
| `kind` | `experiment`, `osc-monitor`, `osc-gen`, `http-burst`, `netsim`, `storm`, `scan`, `beacon`, `discovery`, `mqtt`, `websocket` ou `emulator` |
| `label` | Uma linha em inglês, para logs |
| `params` | Os valores de que o rótulo é feito (destino, bind, host…). Omitido quando não há nenhum |
| `started_ms` | Quando começou, em milissegundos desde 1970 |

Estes comandos iniciam uma tarefa: `experiment_start`, `osc_monitor_start`,
`osc_generator_start`, `http_burst_start`, `netsim_start`, `storm_start`,
`scan_start`, `broadcast_beacon_start`, `discovery_start`, `mqtt_connect`,
`ws_connect` e `emulator_start`. O servidor registra cada início com o endereço
do cliente que o pediu; `POST /api/run` também inicia uma tarefa.

- [`jobs_list`](commands.md#jobs_list) lista as tarefas em execução,
  [`job_stop`](commands.md#job_stop) para uma,
  [`jobs_stop_all`](commands.md#jobs_stop_all) para todas.
- Uma tarefa que termina sozinha, ou falha, envia
  [`job://ended`](events.md#event-job-ended). Uma tarefa que você para não envia mais nada:
  `job_stop` respondendo `true` é a confirmação.
- As tarefas pertencem ao servidor, não ao cliente que as iniciou. Fechar a
  página ou terminar o script não as para, todo cliente as vê e pode
  pará-las, e um servidor que desliga as para todas.

## Um exemplo completo {#example}

Pergunte se o servidor está no ar, inicie um pequeno emulador HTTP com um comando,
execute nele o experimento embutido `http-check` e aguarde o resultado,
depois pare o emulador. `jq` pega campos das respostas.

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)            # omita com um servidor em loopback sem token
AUTH="Authorization: Bearer $TOKEN"
JSON="Content-Type: application/json"

# 1. No ar? Qual versão? Quer um token?
curl -fsS "$SERVER/api/health"
# {"status":"ok","version":"[[version]]","auth":true}

# 2. Um comando: um emulador HTTP em 127.0.0.1:8080 que responde a GET / com 200
EMULATOR=$(curl -fsS -X POST "$SERVER/api/invoke/emulator_start" -H "$AUTH" -H "$JSON" -d '{
  "emulator": { "name": "Example", "bind": "127.0.0.1:8080", "protocol": "http",
                "routes": [ { "method": "GET", "path": "/", "responses": [ { "body": "ok" } ] } ] }
}' | jq .id)

# 3. Executa o experimento embutido que espera 200 de http://127.0.0.1:8080/ e aguarda
curl -sS -X POST "$SERVER/api/run" -H "$AUTH" -H "$JSON" -d '{"template":"http-check"}' \
  | jq '{outcome, error, report_path}'
# {"outcome":"passed","error":null,"report_path":"/data/runs/run-1759600000000-2.json"}

# 4. Para o emulador
curl -fsS -X POST "$SERVER/api/invoke/job_stop" -H "$AUTH" -H "$JSON" -d "{\"id\":$EMULATOR}"
# true
```

`curl -f` transforma um status de erro em um comando que falhou; omita-o (como na
etapa 3) para ver o `EngineError` no corpo.

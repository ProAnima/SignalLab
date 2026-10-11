---
title: Execuções
description: POST /api/run executa um experimento em um servidor do Signal Lab e responde com o resultado, ou transmite as etapas como linhas NDJSON.
---

# Execução de um experimento

Para executar um experimento em um servidor a partir de um script ou de um pipeline e saber como
foi, envie-o a `POST /api/run`. O servidor o executa até o fim e responde
com o resultado — ou, se você pedir, com cada etapa conforme acontece. É
o que [`signallab run --server`](../automation/cli.md#cli-run) usa.

É a mesma execução de [[ui:exp.run]] do editor e de
[`experiment_start`](commands.md#experiment_start): uma tarefa que toda página aberta
vê e pode parar, os mesmos eventos, o mesmo relatório na pasta de dados.

## A requisição {#request}

```http
POST /api/run
Authorization: Bearer <token>
Content-Type: application/json

{ "template": "osc-ping-reply", "overrides": { "device": "192.0.2.20:9000" }, "seed": 42, "timeout": 30 }
```

O corpo nomeia um experimento — um `document` ou um `template`, não os dois — e
com o que executá-lo:

| Campo | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `document` | objeto | — | Um experimento como o editor o salva e exporta. As versões antigas são migradas, como quando um arquivo é aberto |
| `template` | string | — | Um modelo embutido pelo nome de arquivo, com ou sem `.json` (abaixo) |
| `overrides` | objeto | `{}` | Valores de parâmetros só para esta execução. Os valores podem ser strings, números ou booleanos; cada um precisa ser um parâmetro do experimento |
| `profile` | string | o do documento | Executa com este perfil; `""` executa com os padrões |
| `seed` | número | o do documento, senão um novo | 0 a 9007199254740991; a mesma semente sorteia os mesmos valores aleatórios |
| `timeout` | número | `300` | Segundos antes de a execução falhar com `run.timeout`; 1 a 300 |

Um campo que o servidor não conhece é recusado (`400`, `api.run_invalid`). O
próprio documento é lido como um arquivo aberto, então tem no máximo 4 MiB.

Os modelos embutidos — os experimentos que o editor oferece em [[ui:exp.templates]]:

| `template` | O que é |
| --- | --- |
| `empty` | Início e Fim |
| `http-check` | Um GET de `http://127.0.0.1:8080/`, depois uma verificação do status 200 |
| `status-branch` | Um GET de `http://127.0.0.1:8080/`; em caso de 200, uma mensagem OSC, caso contrário um atraso de 500 ms |
| `parallel-flows` | Um [[ui:exp.node.fork]] para um GET de `http://127.0.0.1:8080/` e uma entrada de log, lado a lado, depois [[ui:exp.node.join]] |
| `osc-ping-reply` | Um `/ping` OSC para o parâmetro `device` (`127.0.0.1:9000`), depois uma espera de 2 s por `/pong` em `127.0.0.1:9001` |
| `poll-until-ready` | Um [[ui:exp.node.loop]] que pergunta a `device` por `/status` via OSC até ele responder `ready`, no máximo 10 vezes |
| `flaky-api` | Uma API emulada (parâmetro `api`) que falha antes de funcionar, consultada em um [[ui:exp.node.loop]] até responder 200 |
| `fault-phases` | Um dispositivo UDP atrás de um retransmissor de degradação, com envios por 8 s enquanto o retransmissor passa a limpo, com perdas, desligado e limpo de novo |
| `dependency-outage` | Uma API emulada (parâmetro `api`) derrubada por 2 s enquanto um [[ui:exp.node.loop]] a consulta até ela responder 200 de novo |
| `websocket-echo` | Uma conexão ao parâmetro `service` (`ws://127.0.0.1:9001/echo`), uma mensagem, uma espera pelo eco dela, uma verificação, um fechamento |

Abra um em [[ui:exp.templates]] para ver os nós e parâmetros dele; veja
[experimentos](../experiments/index.md).

## O resultado {#result}

Por padrão, a resposta é `200` com `Content-Type: application/json`, enviada
quando a execução terminou: um objeto JSON, o resultado da execução.

```json
{
  "job_id": 12,
  "experiment": "OSC ping → reply",
  "outcome": "passed",
  "seed": 42,
  "profile": null,
  "overridden": true,
  "params": { "device": "192.0.2.20:9000" },
  "started_ms": 1759600000000,
  "ended_ms": 1759600000310,
  "steps": [ { "job_id": 12, "ts": 1759600000001, "node_id": "start", "state": "running", "detail": "", "message_key": null, "message_params": null }, … ],
  "report_path": "/data/runs/run-1759600000000-12.json"
}
```

| Campo | Tipo | Significado |
| --- | --- | --- |
| `job_id` | número | A tarefa da execução |
| `experiment` | string | O nome do experimento |
| `outcome` | string | `passed`, `failed` ou `stopped` |
| `seed` | número | A semente com que executou: devolva-a como `seed` para sortear os mesmos valores |
| `profile` | string ou null | O perfil com que executou |
| `overridden` | booleano | Alguns valores vieram de `overrides` |
| `params` | objeto | Todo valor de parâmetro que a execução usou |
| `started_ms`, `ended_ms` | número | Milissegundos desde 1970 |
| `error` | `EngineError` | Por que falhou: a primeira falha dela. Omitido quando foi aprovada |
| `steps` | objeto[] | Todas as etapas na ordem em que aconteceram, como [`experiment://step`](events.md#event-experiment-step) |
| `emulators` | objeto[] | O que cada nó [[ui:exp.node.emulator]] recebeu e respondeu: `node`, `name`, `protocol`, `local`, `counts`. Omitido quando não há nenhum |
| `impairments` | objeto[] | O que o retransmissor de cada nó [[ui:exp.node.impairment]] fez, fase por fase. Omitido quando não há nenhum |
| `report_path` | string | O relatório da execução no servidor; baixe-o com [`/api/files`](index.md#files). Omitido quando nenhum foi escrito |
| `report_error` | `EngineError` | Por que o relatório não pôde ser escrito. Omitido nos demais casos |

Os valores de segredos são mascarados em tudo isso. O arquivo de relatório contém as mesmas etapas;
veja [execuções e relatórios](../experiments/runs.md).

Enquanto a execução prossegue, o servidor envia um espaço a cada 15 s. O JSON ignora
espaços em branco antes de um valor, então o resultado ainda é lido, e um proxy não
toma uma execução longa e silenciosa por uma conexão morta.

## Acompanhamento das etapas {#lines}

Para ver as etapas conforme acontecem, peça NDJSON:

```http
Accept: application/x-ndjson
```

A resposta é `200` com `Content-Type: application/x-ndjson`: um objeto JSON
por linha, cada um com um `type`.

| `type` | Quando | O resto da linha |
| --- | --- | --- |
| `started` | Primeiro, uma vez | `job_id`, `experiment`, `seed`, `profile`, `overridden`, `started_ms` |
| `step` | A cada etapa | A etapa, como [`experiment://step`](events.md#event-experiment-step) |
| `heartbeat` | A cada 15 s | Nada |
| `ended` | Por último, uma vez | O resultado, como acima |

```text
{"job_id":12,"experiment":"OSC ping → reply","seed":42,"profile":null,"overridden":true,"started_ms":1759600000000,"type":"started"}
{"job_id":12,"ts":1759600000001,"node_id":"start","state":"running","detail":"","message_key":null,"message_params":null,"type":"step"}
…
{"job_id":12,"experiment":"OSC ping → reply","outcome":"passed",…,"type":"ended"}
```

Leia as linhas até `ended`; ignore um `type` que você não conhece. O servidor
envia `X-Accel-Buffering: no`, de modo que um proxy nginx repassa cada linha na hora.

## Status e resultado {#status}

Um status de erro HTTP significa que **nenhuma execução começou**; o corpo é um
[`EngineError`](index.md#errors):

| Status | Código | Por quê |
| --- | --- | --- |
| `400` | `api.run_invalid` | O corpo não é uma requisição de execução: não é JSON, um campo desconhecido, uma substituição que não é string, número nem booleano |
| `400` | `api.run_source` | Nem `document` nem `template`, ou os dois |
| `415` | `command.json_required` | Não é `Content-Type: application/json` |
| `422` | `api.template_unknown` | Nenhum modelo embutido com esse nome |
| `422` | `file.json_invalid`, `file.too_large`, `doc.*` | O documento não pode ser lido |
| `422` | qualquer código de validação, `run.override_unknown`, `profile.active_missing`, `run.limit_range`, `seed.range`, `secret.missing`, `transport.address_in_use`… | O experimento não pode começar: não valida, um valor está fora do intervalo, um segredo não está guardado, uma porta em que ele escuta está ocupada |

Depois que a execução começou, o status é `200`, aconteça o que acontecer: leia
`outcome` no resultado.

| `outcome` | Significado |
| --- | --- |
| `passed` | Todas as etapas foram aprovadas e [[ui:exp.node.end]] foi alcançado |
| `failed` | Uma etapa falhou, ou a execução levou mais que o `timeout` dela (`run.timeout`); `error` diz qual e por quê |
| `stopped` | Foi parada antes de terminar: por `job_stop`, [[ui:app.stopAll]] ou pelo desligamento do servidor. `steps` tem as etapas a que chegou; nenhum relatório é salvo |

## Um cliente que some {#disconnect}

Fechar a conexão não para a execução. Ela é uma tarefa no servidor: executa
até o fim e salva o relatório, como uma execução iniciada em um navegador faz quando
a aba é fechada. Encontre-a com [`jobs_list`](commands.md#jobs_list), pare-a
com [`job_stop`](commands.md#job_stop) e leia o relatório dela depois com
[`experiment_runs`](commands.md#experiment_runs). Quando o servidor desliga,
a execução é parada e um cliente ainda conectado recebe `"outcome": "stopped"`.

## Exemplos {#examples}

Execute um modelo embutido e aguarde o resultado:

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
curl -sS -X POST "$SERVER/api/run" \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"template":"osc-ping-reply","overrides":{"device":"192.0.2.20:9000"},"timeout":30}' \
  | jq -r .outcome
```

Envie o seu próprio experimento com um parâmetro alterado e imprima cada etapa conforme
acontece:

```bash
jq '{document: ., overrides: {api: "http://192.0.2.10:8080"}, profile: ""}' smoke.json |
  curl -sSN -X POST "$SERVER/api/run" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    -H "Accept: application/x-ndjson" --data @- |
  jq -r 'select(.type == "step") | "\(.node_id)  \(.state)  \(.detail)"'
```

`-N` impede `curl` de reter as linhas. Para reprovar um pipeline em uma execução
com falha, verifique `outcome`:

```bash
outcome=$(curl -sS -X POST "$SERVER/api/run" -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" -d '{"template":"http-check"}' | jq -r .outcome)
[ "$outcome" = "passed" ]
```

## Pela linha de comando {#cli}

[`signallab run`](../automation/cli.md#cli-run) com `--server <url>` executa em um
servidor por este endpoint: envia o experimento que leu, com
`overrides`, `seed` e `timeout`, pede NDJSON e imprime cada etapa conforme a
linha dela chega. O token vem de `--token-file`, senão de `SIGNALLAB_TOKEN`.
Com `--report`, ele baixa o relatório por `/api/files`. Um servidor que
não envia nada por 60 s — nem mesmo um heartbeat — é considerado perdido.

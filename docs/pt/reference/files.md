---
title: Arquivos e pastas
description: Onde o Signal Lab guarda seus arquivos — a pasta de dados, o experimento, as bibliotecas de sinais e de emuladores, os relatórios de execução e as exportações — seus formatos, e o que é seguro editar, copiar e mover.
---

# Arquivos e pastas

Tudo o que o Signal Lab guarda é JSON simples (ou texto) em uma única pasta, a pasta de
dados. Os valores dos segredos nunca estão nela.

## A pasta de dados {#data-folder}

| Onde o Signal Lab roda | A pasta de dados |
| --- | --- |
| Aplicativo de desktop, Windows | `Documents\SignalLab` na sua pasta de usuário: `C:\Users\<you>\Documents\SignalLab` |
| Aplicativo de desktop, Linux | `~/Documents/SignalLab` |
| Servidor | `--data-dir`, ou `SIGNALLAB_DATA_DIR`; sem nenhum dos dois, `Documents/SignalLab` na pasta pessoal do usuário com que ele roda |
| Servidor, imagem Docker | `/data`, um volume (`signallab-data` no arquivo compose) |
| `signallab run` | Uma pasta temporária, removida quando ele termina — a menos que `--data-dir` nomeie uma |

O aplicativo de desktop também lê `SIGNALLAB_DATA_DIR` do seu ambiente quando ela está
definida. A pasta é criada quando algo é gravado nela pela primeira vez.

::: tip
No Windows o aplicativo usa a pasta `Documents` diretamente dentro da sua pasta de usuário,
mesmo quando o Windows guarda os seus documentos em outro lugar (OneDrive).
:::

Em um servidor, os arquivos são gravados na máquina do servidor, não na sua. O selo
[[ui:app.server]] no cabeçalho diz onde, na dica dele; a API informa o caminho como `data_dir`
de [`app_info`](../api/commands.md#app_info), e [`/api/files`](../api/index.md#files) baixa o
que há nela.

Os comandos `signallab emulate`, `signallab send` e `signallab mcp` da linha de comando leem as
bibliotecas do aplicativo da mesma pasta que o aplicativo de desktop.

## O que há nela {#contents}

| Arquivo | O que é | Gravado |
| --- | --- | --- |
| `experiment.json` | O experimento aberto no editor | Logo depois de cada alteração |
| `signals.json` | A biblioteca de sinais ([[ui:nav.signals]]) | Logo depois de cada alteração |
| `emulators.json` | A biblioteca de emuladores ([[ui:nav.emulators]]) | Logo depois de cada alteração |
| `runs/run-<ms>-<job>.json` | Um relatório por execução que terminou por conta própria | Quando a execução termina |
| `exports/experiment-<ms>-<16 hex digits>.json` | Um instantâneo do experimento | [[ui:exp.exportJson]] |
| `capture-<ms>.jsonl`, `capture-<ms>.txt` | Os quadros do Inspetor | [[ui:ins.exportJsonl]], [[ui:ins.exportTxt]] |
| `token` | O token de acesso de um servidor, legível só pelo usuário dele | `--generate-token`, na primeira inicialização |
| `.experiment-<hex>.tmp`, `.signals-<hex>.tmp`, `.emulators-<hex>.tmp` | Uma gravação a caminho | Por um instante, depois renomeado |

`<ms>` é um instante em milissegundos desde 1970; `<job>` é o número da tarefa da execução.
Em um servidor, todo navegador trabalha no mesmo `experiment.json`, nas mesmas bibliotecas e
nos mesmos relatórios.

## Formatos {#formats}

Todos são JSON em UTF-8, gravados com recuos para que se leiam e se comparem bem. Cada um
traz uma `version`; um arquivo de versão mais antiga é lido e migrado quando é aberto, e
gravado de volta na versão atual na próxima vez que for salvo — depois do que um Signal Lab
mais antigo não consegue abri-lo. Um arquivo de versão mais nova do que este Signal Lab conhece
é recusado (`doc.version_unsupported`) e deixado como está: lido mesmo assim, o que a versão
mais nova acrescentou se perderia no próximo salvamento. Isso vale para o experimento e para as
duas bibliotecas.

### experiment.json {#experiment-json}

O documento do experimento, versão 9 — o mesmo JSON que [[ui:exp.exportJson]] grava e
[[ui:exp.importJson]] lê:

```json
{
  "version": 9,
  "name": "HTTP check",
  "params": [],
  "profiles": [],
  "profile": null,
  "seed": null,
  "cookies": true,
  "nodes": [ { "id": "start", "type": "start", "x": 40, "y": 80 }, … ],
  "edges": [ { "from": "start", "to": "request", "port": "next" }, … ]
}
```

- No máximo 4 MiB, e de 1 a 64 nós (`doc.node_count`).
- As versões 1 a 8 são migradas ao abrir. Um arquivo anterior à versão 8 abre com `cookies`
  desligado, para rodar como rodava; as outras configurações que cada versão acrescentou
  (parâmetros na 2, perfis na 3, novas tentativas na 4, repetições e laços na 5, emuladores na
  6, degradações na 7, WebSocket e autenticação HTTP na 8, carga na 9) começam vazias.
- Uma versão mais nova do que este Signal Lab conhece é recusada (`doc.version_unsupported`)
  em vez de aberta sem o que ele não consegue ler.
- Um arquivo que não é lido é relatado com o seu caminho, linha e coluna, e nunca substituído.
- Ele é gravado em um arquivo temporário e renomeado, de modo que uma gravação que falha deixa
  o anterior.

O que são os nós, os parâmetros e os perfis: [experimentos](../experiments/index.md),
[nós](../experiments/nodes.md), [dados](../experiments/data.md).

### signals.json {#signals-json}

A biblioteca de sinais, versão 2:

```json
{
  "version": 2,
  "signals": [
    {
      "id": "…",
      "name": "Go cue",
      "group": "Stage/Cues",
      "note": "",
      "body": { "transport": "osc", "target": "127.0.0.1:9000", "address": "/cue/go", "args": [ { "type": "int", "value": 1 } ] }
    }
  ],
  "folders": [ "Stage", "Stage/Cues" ]
}
```

- `group` é a pasta do sinal como um caminho com `/`; vazio é o nível superior. `folders`
  (acrescentado na versão 2) lista todas as pastas, as vazias incluídas, e é omitido quando não
  há nenhuma. Um arquivo versão 1 é lido igual, sem pastas vazias.
- `body` é um entre `osc`, `udp`, `http` ou `mqtt`; os campos deles estão em
  [`signals_save`](../api/commands.md#signals_save).
- Quando o arquivo não existe, o conjunto inicial é gravado — todos os destinos em
  `127.0.0.1` — e renomeado para o idioma da interface.
- Um arquivo que não é lido é relatado com o seu caminho, linha e coluna
  (`signals.json_invalid`) e nunca substituído pelo conjunto inicial: corrija-o ou exclua-o.
  Nada grava a biblioteca enquanto ela não é lida — um salvamento é recusado com o mesmo erro e
  o arquivo fica como está — até que [[ui:sig.reload]] a leia de novo. Um salvamento passa por
  um arquivo temporário na mesma pasta, de modo que uma gravação interrompida deixa o arquivo
  anterior.

### emulators.json {#emulators-json}

A biblioteca de emuladores, versão 1:

```json
{
  "version": 1,
  "emulators": [
    { "id": "demo-api", "note": "…", "emulator": { "name": "Demo API", "bind": "127.0.0.1:8080", "protocol": "http", "routes": [ … ] } }
  ]
}
```

Cada entrada é um documento de emulador com um `id` e uma `note`; o documento está descrito em
[emuladores](../tools/emulators.md). Como nos sinais, um arquivo ausente recebe o conjunto
inicial (todos vinculados a `127.0.0.1`), e um quebrado é relatado (`emulators.json_invalid`),
nunca substituído. Ele é gravado por um arquivo temporário. `signallab emulate` também lê um
arquivo próprio que contém um emulador, uma lista deles ou uma biblioteca como esta.

### Relatórios de execução {#run-reports}

`runs/run-<started ms>-<job>.json`, relatório versão 6: um arquivo por execução aprovada ou com
falha, nunca sobrescrito (uma segunda execução com o mesmo nome ganha `-2`, `-3`… no fim). Uma
execução que foi interrompida não salva nenhum.

| Campo | O que é |
| --- | --- |
| `version` | 6 |
| `experiment` | O nome do experimento |
| `document_version` | A versão do documento que rodou |
| `seed`, `profile` | Com o que rodou |
| `overrides` | Valores dados só para esta execução |
| `params` | Todo valor de parâmetro que ela usou |
| `started_ms`, `ended_ms` | Milissegundos desde 1970 |
| `outcome` | `passed` ou `failed` |
| `error` | A primeira falha dela, ou null |
| `steps` | As etapas, como [`experiment://step`](../api/events.md#event-experiment-step): depois das primeiras 5.000, uma por segundo e as últimas 500, com uma lacuna contada em `omitted` (desde a versão 6) |
| `emulators` | O que cada nó [[ui:exp.node.emulator]] recebeu e respondeu (desde a versão 3); omitido quando não há nenhum |
| `impairments` | O que o retransmissor de cada nó [[ui:exp.node.impairment]] fez, fase por fase (desde a versão 4); omitido quando não há nenhum |

As medições de uma etapa de carga ficam na última etapa dela (desde a versão 5). O histórico
de execuções da linha do tempo e [[ui:exp.compare]] leem esses arquivos; um relatório que não
pode ser lido fica de fora da lista. Veja [execuções e relatórios](../experiments/runs.md).

### Exportações {#exports}

- `exports/experiment-…json`: o documento do experimento, como acima. Toda exportação é um
  arquivo novo.
- `capture-….jsonl`: um quadro do Inspetor por linha, com os bytes que ele guarda em `data`,
  em base64.
- `capture-….txt`: os quadros para leitura, cada um com um dump hexadecimal.

Em um servidor, a exportação do Inspetor é baixada para o seu computador à medida que é feita;
a exportação de um experimento oferece [[ui:common.download]], e o [[ui:exp.reportSaved]] de
uma execução, na linha do tempo, é um link que baixa o relatório dela.

## Os segredos não estão nestes arquivos {#secrets}

Um experimento nomeia um segredo — `{{secret.API_TOKEN}}` — e só o nome é gravado. O valor fica
guardado:

| Onde o Signal Lab roda | Onde ficam os valores dos segredos |
| --- | --- |
| Aplicativo de desktop, Windows | Gerenciador de Credenciais do Windows, em `SignalLab` ([[ui:exp.secrets]] no editor) |
| Aplicativo de desktop, Linux | Em lugar nenhum: segredos não podem ser guardados (`secret.unsupported`) |
| Servidor | Somente leitura: a variável de ambiente `SIGNALLAB_SECRET_<NAME>`, ou o arquivo `<NAME>` em `--secrets-dir` (padrão `/run/secrets/signallab`) |
| `signallab` | Os mesmos arquivos e variáveis, ou o repositório do sistema com `--secrets system` |

::: warning
O que você digita direto em um campo é guardado como digitou. Uma senha nas credenciais de um
sinal HTTP, a senha de um emulador de broker MQTT, um token colado em um cabeçalho — tudo isso
é texto simples em `signals.json`, `emulators.json` ou `experiment.json`, e nas exportações
deles. Use `{{secret.NAME}}` em um experimento para tudo o que você não poria em uma pasta
compartilhada.
:::

## Configurações da interface {#settings}

O que a interface lembra — o idioma, os valores digitados por último em cada tela, qual painel
está aberto e de que tamanho, [[ui:http.keepCookies]], quando as atualizações foram procuradas
pela última vez e o número aleatório da instalação para atualizações — é guardado pela própria
interface, não na pasta de dados: no armazenamento do próprio aplicativo no desktop, no
armazenamento do site no navegador para a página de um servidor (por navegador). As
credenciais da tela [[ui:nav.http]] não são guardadas ali.

O Signal Lab não grava arquivos de registro; veja [solução de problemas](troubleshooting.md#logs).

## Copiar, editar, mover {#backup}

- **Faça backup** copiando a pasta inteira. Tudo nela é JSON autocontido; os valores dos
  segredos não estão nela, então defina-os de novo em uma máquina nova.
- **Edite** `signals.json`, `emulators.json` e `experiment.json` à mão enquanto o Signal Lab
  está fechado (ou, em um servidor, enquanto nenhuma página está aberta): o aplicativo grava o
  arquivo inteiro a partir do que guarda. Para `signals.json` e `emulators.json`, um
  salvamento sobre um arquivo alterado desde que o aplicativo o leu é recusado (`file.changed`)
  e a sua alteração é mantida — pressione Recarregar para lê-la; `experiment.json` é
  sobrescrito pelo próximo salvamento do editor. Um erro é relatado com a linha e a coluna
  quando o arquivo é lido na vez seguinte, nunca substituído em silêncio — no caso de
  `signals.json` e `emulators.json`, também quando o aplicativo salva neles: esse salvamento é
  recusado e o arquivo fica como você o deixou.
- **Exclua** `runs/`, `exports/` e os arquivos `capture-*` quando quiser. Excluir
  `signals.json` ou `emulators.json` traz de volta o conjunto inicial; excluir
  `experiment.json` traz de volta o experimento inicial.
- **Mova** a pasta copiando-a e apontando o Signal Lab para o novo lugar: `--data-dir` para um
  servidor, `SIGNALLAB_DATA_DIR` para o aplicativo de desktop.
- **Compartilhe** um experimento exportando-o, ou versionando o JSON dele junto do projeto que
  ele testa; [`signallab run`](../automation/cli.md#cli-run) o executa a partir dali.

---
title: Assistentes (MCP)
description: O signallab mcp permite que um assistente de IA monte, verifique e execute experimentos, envie mensagens, escute e faça o papel de emuladores, pelo Model Context Protocol.
---

# O Signal Lab para um assistente: `signallab mcp`

`signallab mcp` é um servidor [Model Context Protocol](https://modelcontextprotocol.io). Um
assistente no Claude Code, no Claude Desktop, no Cursor, no VS Code ou em qualquer outro cliente
MCP o inicia e pode então:

- aprender do que um experimento é feito, escrever um, verificá-lo, executá-lo e ler, etapa por
  etapa, por que ele falhou;
- enviar uma mensagem OSC, um datagrama, uma requisição HTTP, uma mensagem WebSocket ou uma
  publicação MQTT, e escutar em uma porta o que um dispositivo envia;
- disparar um sinal da sua biblioteca;
- fazer o papel de uma dependência — uma API HTTP, um dispositivo OSC, UDP ou TCP, um broker MQTT
  — e ler o que o seu sistema enviou a ela;
- reler execuções anteriores e comparar duas delas.

Toda ação passa pelos mesmos comandos do motor que o aplicativo usa, então um experimento que o
assistente executa é a mesma execução que o aplicativo faria, com o mesmo relatório, e toda falha
é redigida como a interface a redige.

::: warning
Envios, execuções e emuladores colocam tráfego real na rede. Diga ao assistente quais
dispositivos são seus, para que ele saiba com quais pode falar; os modelos incluídos apontam para o loopback
(`127.0.0.1`).
:::

## Configuração {#setup}

O `signallab` vem com o aplicativo de desktop e fica no seu `PATH` depois da instalação (veja
[Instalação](cli.md#install)). O cliente inicia o `signallab mcp` ele mesmo e conversa com ele por
stdin e stdout; você não o executa à mão.

### Imprimir a configuração {#print-config}

`--print-config` imprime o que um cliente precisa, com o caminho completo deste `signallab`:

| Comando | O que imprime |
| --- | --- |
| `signallab mcp --print-config claude-code` | A linha de comando `claude mcp add`. |
| `signallab mcp --print-config claude-desktop` | A entrada `mcpServers` para o arquivo de configuração do Claude Desktop. |
| `signallab mcp --print-config cursor` | A mesma entrada `mcpServers`, para o `mcp.json` do Cursor. |
| `signallab mcp --print-config vscode` | A entrada `servers` para o `.vscode/mcp.json` do VS Code. |

Para um assistente que trabalha em um [servidor de laboratório](#on-a-server), acrescente
`--server URL`: a configuração impressa passa a levá-lo, com um marcador no lugar do token.

### Claude Code {#claude-code}

Execute a linha que `--print-config claude-code` imprime, por exemplo:

```bash
claude mcp add signallab -- "C:\Program Files\Signal Lab\signallab.exe" mcp
```

### Claude Desktop e Cursor {#claude-desktop}

Coloque a entrada na configuração do cliente — para o Claude Desktop, `claude_desktop_config.json`;
para o Cursor, `mcp.json` — e reinicie o cliente:

```json
{
  "mcpServers": {
    "signallab": {
      "command": "C:\\Program Files\\Signal Lab\\signallab.exe",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

### VS Code {#vscode}

```json
{
  "servers": {
    "signallab": {
      "type": "stdio",
      "command": "/usr/bin/signallab",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

### Outros clientes {#other-clients}

Qualquer cliente que inicia um servidor stdio funciona do mesmo jeito: o comando é `signallab` (ou
o caminho completo dele), e os argumentos, `mcp` e qualquer uma das [opções](#options). No Linux,
a imagem do servidor também pode servir como comando:

```bash
docker run -i --rm --network host --entrypoint signallab ghcr.io/proanima/signallab:[[version]] mcp
```

## Opções {#options}

| Opção | O que faz | Padrão |
| --- | --- | --- |
| `--server URL` | Executa experimentos, envios e emuladores neste servidor Signal Lab (veja [Em um servidor de laboratório](#on-a-server)). | `SIGNALLAB_SERVER` |
| `--token-file PATH` | Um arquivo com o token do servidor. | `SIGNALLAB_TOKEN_FILE`, senão `SIGNALLAB_TOKEN` |
| `--data-dir PATH` | Onde as execuções e os relatórios delas são guardados. Não com `--server`. | a pasta de dados do aplicativo (`Documents/SignalLab`) |
| `--library PATH` | A biblioteca de sinais para `list_signals` e `fire_signal`. | o `signals.json` do aplicativo |
| `--emulators PATH` | A biblioteca de emuladores para `list_emulators` e `start_emulator`. | o `emulators.json` do aplicativo |
| `--secrets files\|system` | De onde vêm os valores dos segredos para execuções nesta máquina, como em [`run`](cli.md#secrets). Não com `--server`. | `files` |
| `--secrets-dir PATH` | Uma pasta de arquivos de segredos, um por nome. Não com `--server`. | `/run/secrets/signallab`, quando existe |
| `--lang <code>` | O idioma dos resultados e das falhas. | `SIGNALLAB_LANG`, senão a localidade, senão `en` |
| `--print-config CLIENT` | Imprime a configuração de um cliente e sai: `claude-code`, `claude-desktop`, `cursor` ou `vscode`. | |

As execuções guardam os relatórios na pasta de dados do aplicativo, onde o aplicativo guarda os
seus, então eles ficam depois da sessão.

## Ferramentas {#tools}

As ferramentas que só leem são marcadas como somente leitura, para que um cliente possa deixá-las
rodar sem perguntar. As ferramentas que alcançam o mundo exterior — que enviam, escutam ou iniciam
algo — são marcadas como tal, e um cliente pode perguntar a você antes de cada chamada. Nenhuma é
marcada como destrutiva.

| Ferramenta | O que faz | Alcança o mundo exterior |
| --- | --- | --- |
| `describe_nodes` | O documento de experimento, cada tipo de nó com os campos, as saídas e um exemplo, a linguagem `{{template}}`, os perfis de carga e o documento de emulador. | não |
| `list_templates` | Os experimentos incluídos, com os parâmetros deles. | não |
| `get_template` | Um experimento incluído, como documento. | não |
| `validate_experiment` | Verifica um experimento como o editor faz antes de uma execução; não envia nada. | não |
| `run_experiment` | Executa um experimento até o fim e relata cada etapa. | sim |
| `send_osc` | Uma mensagem OSC. | sim |
| `send_udp` | Um datagrama UDP. | sim |
| `send_http` | Uma requisição HTTP. | sim |
| `send_mqtt` | Uma publicação MQTT 3.1.1. | sim |
| `send_ws` | Uma troca WebSocket. | sim |
| `listen` | O que chega em uma porta UDP durante um tempo. | sim |
| `list_signals` | Os sinais da sua biblioteca. | não |
| `fire_signal` | Envia um sinal da biblioteca. | sim |
| `list_emulators` | Os emuladores da sua biblioteca. | não |
| `start_emulator` | Inicia um emulador. | sim |
| `emulator_exchanges` | O que um emulador em execução recebeu e respondeu. | não |
| `set_emulator_down` | Tira do ar um emulador em execução, ou o traz de volta. | sim |
| `list_runs` | Os relatórios de execuções anteriores. | não |
| `compare_runs` | Duas execuções lado a lado. | não |
| `list_jobs` | O que está rodando. | não |
| `stop_job` | Para uma tarefa em execução. | sim |

### Experimentos {#tools-experiments}

`describe_nodes` é o que o assistente lê antes de escrever um experimento; é o mesmo que
[`signallab nodes`](cli.md#cli-nodes). `list_templates` e `get_template` dão exemplos que
funcionam, para executar ou adaptar.

`validate_experiment` e `run_experiment` recebem o experimento de uma de três formas — exatamente
uma delas:

| Argumento | O que é |
| --- | --- |
| `document` | Um documento de experimento, como o aplicativo o salva. |
| `file` | O caminho de um arquivo de experimento na máquina em que o `signallab` roda. |
| `template` | O nome de um modelo incluído. |
| `params` | Valores de parâmetros para esta execução: `{"name": "value"}`; números e booleanos são lidos como texto. |
| `profile` | Executar com este perfil do documento; `""` para os padrões. |
| `seed` | `run_experiment`: a semente dos valores aleatórios. |
| `timeout` | `run_experiment`: segundos que a execução pode levar, de 1 a 300 (padrão 300). |

`run_experiment` responde quando a execução terminou: aprovada, com falha ou parada, a duração e a
semente, cada etapa com o que fez ou por que falhou, o que foi pedido a cada emulador, o que cada
retransmissor de degradação fez e o caminho do relatório. Uma execução que falha é uma resposta
normal — as etapas dizem por quê —, não uma chamada que falhou.

### Mensagens avulsas {#tools-send}

| Ferramenta | Argumentos |
| --- | --- |
| `send_osc` | `target` (`host:port`), `address`, `args`: números (inteiros → int32, ou int64 além desse intervalo; senão float32), strings, booleanos, `null` ou `{"type": "int"\|"float"\|"str"\|"long"\|"double"\|"bool"\|"blob"\|"nil", "value": …}`. |
| `send_udp` | `target`, e `text` ou `hex` (`"de ad be ef"`). |
| `send_http` | `method`, `url`, `headers` (`{"Name": "value"}`), `body`, `timeout_ms` (padrão 10.000), `auth`: `{"scheme": "basic"\|"digest", "username", "password"}` ou `{"scheme": "bearer", "token"}`. Devolve o status, o tempo, os cabeçalhos e o corpo — os primeiros 16 KiB dele. |
| `send_mqtt` | `broker` (`host:port`, porta 1883 quando não há nenhuma), `topic`, `payload`, `qos` (0, 1 ou 2), `retain`. Uma carga útil vazia com `retain` limpa um valor retido. |
| `send_ws` | `url` (`ws://` ou `wss://`), `text` ou `hex`, `headers`, `protocols` e, para aguardar a resposta, `expect` (contém), `expect_regex` ou `wait` (qualquer mensagem); `timeout_ms` de 1 a 120.000 (padrão 2.000). Devolve o handshake, o que foi enviado e a resposta, com o JSON interpretado quando é JSON. |

São os comandos que as telas do aplicativo usam; veja [`signallab send`](cli.md#cli-send).

### Escuta {#tools-listen}

`listen` abre uma porta UDP **na máquina em que o `signallab mcp` roda**, durante um tempo, e
devolve o que chegou: mensagens OSC decodificadas, outros datagramas como texto e hex.

| Argumento | O que é | Padrão |
| --- | --- | --- |
| `bind` | `IP:port`, por exemplo `0.0.0.0:9000`. | obrigatório |
| `protocol` | `osc` ou `udp`. | `osc` |
| `seconds` | Por quanto tempo escutar, de 0,1 a 60. | 5 |
| `max` | Parar depois deste número de datagramas, de 1 a 1.000. | 100 |

Quando nada chega em `0.0.0.0`, a resposta lembra o assistente de verificar o firewall
([`signallab doctor`](cli.md#cli-doctor)). Com `--server`, `listen` é recusado: em um servidor, um
experimento com um nó de espera escuta ali.

### Sinais e emuladores {#tools-library}

`list_signals` e `fire_signal` usam a sua biblioteca de sinais — o `signals.json` do aplicativo,
`--library` ou um caminho `library` passado na chamada. Um sinal é disparado pelo id ou pelo nome,
exatamente como o aplicativo o dispara.

`list_emulators` dá os nomes dos emuladores da sua biblioteca. `start_emulator` inicia um — um
documento em `emulator`, ou o id ou o nome de uma entrada da biblioteca em `name` — e devolve o id
da tarefa e o endereço dele; ele responde pelas suas regras até `stop_job`. `bind` o move para
outro `IP:port`, `params` dá valores que os modelos dele leem, `seed` fixa as escolhas aleatórias
dele. `emulator_exchanges` (`job_id`, e `after` para só as mais novas) lista o que chegou e o que
cada regra respondeu. `set_emulator_down` (`job_id`, `down` e `fault`: `unavailable`, `reset` ou
`timeout`) derruba um emulador em execução até que ele seja reativado: o HTTP encontra a falha
(`unavailable` responde 503), um dispositivo TCP e um broker MQTT derrubam as conexões, OSC e UDP
não respondem nada. Veja [Emuladores](../tools/emulators.md).

### Execuções e tarefas {#tools-runs}

`list_runs` lê os relatórios de execuções anteriores, os mais recentes primeiro — de um
experimento, quando `experiment` o indica, no máximo `limit` (de 1 a 500, padrão 50) —, com os
números de cada etapa com carga. `compare_runs` recebe dois dos nomes deles, `a` (antes) e `b`
(depois), e coloca lado a lado as latências, a taxa de erros, a taxa alcançada e as requisições
puladas de cada etapa com carga, marcando como regressão uma mudança de 5% ou mais para o lado
ruim. Veja [Execuções e relatórios](../experiments/runs.md).

`list_jobs` lista o que está rodando — monitores, geradores, emuladores, execuções — e `stop_job`
para um deles pelo id.

## Resultados e erros {#results}

Toda resposta é texto para o modelo e o mesmo como dados estruturados. Uma falha é marcada como
erro e traz o erro do motor — um `code` estável, os valores dele, o nó e o campo a que se refere
—, redigido no idioma escolhido com `--lang`. Argumentos que o assistente errou voltam em palavras
que ele consegue corrigir.

## Progresso e cancelamento {#progress}

Quando o cliente pede progresso em `run_experiment`, cada etapa é relatada à medida que acontece
(o nó e o estado dele), então o assistente — e você — veem a execução andar. Cancelar uma chamada
a interrompe; cancelar `run_experiment` para a própria execução, como faz o botão [[ui:common.stop]] no
aplicativo.

Quando o cliente fecha a conexão, as chamadas ainda em andamento terminam, e então o
`signallab mcp` sai.

## Em um servidor de laboratório {#on-a-server}

Com `--server http://192.0.2.10:1430`, os experimentos, os envios, os sinais e os emuladores
acontecem **nesse servidor**, pela API dele — com a rede, os segredos e a pasta de dados dele —,
então o assistente alcança equipamentos que só o laboratório alcança. Informe o token no ambiente
do cliente:

```json
{
  "mcpServers": {
    "signallab": {
      "command": "signallab",
      "args": ["mcp", "--server", "http://192.0.2.10:1430"],
      "env": { "SIGNALLAB_TOKEN": "<the server's token>" }
    }
  }
}
```

O que fica nesta máquina: as bibliotecas de sinais e de emuladores (as do aplicativo, ou
`--library` e `--emulators`) e os arquivos que uma chamada indica (`file`, `library`) são lidos
aqui, e o que eles contêm é enviado ao servidor; `listen` é recusado. Veja
[Executar o Signal Lab como servidor](../server/index.md).

## Segurança {#safety}

- O assistente só pode fazer o que as ferramentas fazem, e cada ferramenta é um dos próprios
  comandos do aplicativo: ele não alcança nada que o aplicativo não alcançaria.
- As ferramentas que enviam, escutam ou iniciam algo são marcadas como ferramentas que alcançam o
  mundo exterior; o seu cliente decide se pergunta a você antes de cada chamada.
- Os valores dos segredos nunca chegam ao assistente: um experimento os indica como
  `{{secret.NAME}}`, e todo resultado mostra `••••` no lugar deles.
- Um emulador ou um ouvinte abre uma porta na máquina em que roda; `list_jobs` e `stop_job`
  mostram e encerram o que ainda está rodando.

## Protocolo {#protocol}

Para quem escreve clientes: JSON-RPC 2.0 sobre stdio, uma mensagem por linha; o stdout leva só
mensagens do protocolo, e tudo o que é para uma pessoa vai para o stderr. Versões do protocolo
`2025-06-18`, `2025-03-26` e `2024-11-05` (a mais nova, quando o cliente pede outra), lotes,
`ping`, `tools/list` e `tools/call`; progresso como `notifications/progress` para uma chamada que
enviou um `progressToken`, cancelamento por `notifications/cancelled`. As `instructions` do
servidor dizem ao modelo como as ferramentas se encaixam.

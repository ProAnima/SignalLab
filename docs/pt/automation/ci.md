---
title: Pipelines de CI
description: Execute experimentos do Signal Lab no GitHub Actions, no GitLab CI ou em qualquer pipeline, reprove o job quando um falhar e guarde um relatório JUnit.
---

# Signal Lab em CI

Os experimentos que colocam uma instalação no ar também são os testes de regressão dela.
Em um pipeline, [`signallab run`](cli.md#cli-run) os executa sem janela,
imprime cada etapa, escreve um relatório JUnit que todo sistema de CI mostra e sai com
um código que o job entende:

| Código de saída | O pipeline deve |
| --- | --- |
| `0` | seguir em frente: todos os experimentos foram aprovados |
| `1` | falhar: um experimento executou e falhou |
| `2` | falhar: um experimento ou o comando está errado (um erro de validação, um parâmetro desconhecido, um segredo ausente) |
| `3` | falhar ou tentar de novo: nada pôde executar (o servidor não pode ser alcançado ou recusa o token, uma porta não pode ser aberta) |

Há quatro maneiras de entrar:

| Maneira | Onde executa |
| --- | --- |
| [A GitHub Action](#github-actions) | Um runner Linux, a partir da imagem do servidor. |
| [A imagem](#docker) | Qualquer CI que execute contêineres: GitLab, Jenkins, um shell com Docker. |
| [O binário](#binary) | Qualquer runner, Windows incluído. |
| [Um servidor de laboratório](#lab-server) | As execuções acontecem em um servidor do Signal Lab ao lado do equipamento; o pipeline só as envia. |

## GitHub Actions {#github-actions}

O repositório também é uma GitHub Action. Ela executa `signallab` a partir da imagem
`ghcr.io/proanima/signallab`, reprova o job quando um experimento falha e
deixa um relatório JUnit:

```yaml
jobs:
  signallab:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: ProAnima/SignalLab@v[[version]]
        with:
          version: [[version]]
          experiments: tests/signallab/*.json
          params: |
            api=http://127.0.0.1:8080
        env:
          SIGNALLAB_SECRET_API_TOKEN: ${{ secrets.API_TOKEN }}

      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: signallab-junit
          path: signallab-junit.xml
```

Uma execução que não é aprovada também vira uma anotação de erro na página da execução, ao lado
da etapa que falhou, com o experimento e o motivo da falha.

### Entradas {#action-inputs}

| Entrada | O que faz | Padrão |
| --- | --- | --- |
| `experiments` | Arquivos de experimento ou nomes de modelos embutidos, separados por espaços ou linhas. Padrões como `tests/*.json` são expandidos; um caminho com espaços não é aceito. | obrigatório |
| `params` | Valores de parâmetros, `NAME=VALUE`, um por linha. | |
| `profile` | Executa com este perfil dos experimentos. | |
| `matrix` | Executa uma vez por combinação: `NAME=V1,V2`, um nome por linha. | |
| `matrix-file` | Combinações de um arquivo JSON (veja [Uma matriz de execuções](#matrix)). | |
| `fail-fast` | `"true"`: para na primeira execução que não é aprovada. | `"false"` |
| `server` | Executa neste servidor do Signal Lab em vez de no job, por exemplo `http://192.0.2.10:1430`. | |
| `token` | O token de acesso do servidor. Passe um segredo. | |
| `junit` | Para onde vai o relatório JUnit. | `signallab-junit.xml` |
| `timeout` | Segundos que uma execução pode levar, 1 a 300. | `300` |
| `version` | A tag da imagem. | `latest` |
| `image` | Outro registro ou uma imagem construída localmente; `version` é a tag dela. | `ghcr.io/proanima/signallab` |
| `lang` | O idioma das mensagens e do relatório: `en`, `ru`, `es`, `fr`, `de`, `pt`, `zh`, `ja`, `ko`, `hi` ou `ar`. | `en` |
| `fail-on-error` | `"false"`: não reprova a etapa; leia `exit-code` em vez disso. | `"true"` |

Os caminhos (`experiments`, `matrix-file`, `junit`) são relativos ao diretório de trabalho
da etapa.

::: tip
Fixe `version` na versão com que você testou. `latest` passa para cada nova
versão estável.
:::

### Saídas {#action-outputs}

| Saída | O que é |
| --- | --- |
| `junit` | O caminho do relatório JUnit. |
| `exit-code` | O código de saída de `signallab`: `0`, `1`, `2` ou `3`. |

### Seguir adiante depois de uma falha {#fail-on-error}

Uma etapa que falha não entrega saídas ao resto do job. Para decidir você mesmo,
defina `fail-on-error: "false"` e ramifique por `exit-code`:

```yaml
      - id: lab
        uses: ProAnima/SignalLab@v[[version]]
        with:
          experiments: tests/signallab/smoke.json
          fail-on-error: "false"

      - if: steps.lab.outputs.exit-code == '1'
        run: echo "an experiment failed; the report is ${{ steps.lab.outputs.junit }}"

      - if: steps.lab.outputs.exit-code != '0'
        run: exit 1
```

### Segredos na action {#action-secrets}

O `{{secret.NAME}}` de um experimento lê `SIGNALLAB_SECRET_NAME`. Defina essas
variáveis na etapa (`env:`), a partir dos segredos do repositório. A action entrega
a `signallab`, pelo nome, toda variável `SIGNALLAB_SECRET_…` da etapa; os
valores viajam no ambiente, nunca em uma linha de comando, e todo relatório e
linha de log mostra `••••` no lugar deles. Um segredo de que o experimento precisa e que não está
definido reprova a etapa com o código de saída `2`, antes de qualquer envio.

A entrada `token` viaja do mesmo modo, como `SIGNALLAB_TOKEN`.

### Como a action executa {#action-runs}

- Ela precisa de um **runner Linux** com Docker (`ubuntu-latest` tem). Em um
  runner Windows ou macOS ela para com o código de saída `2` e uma anotação; ali,
  use [o binário](#binary).
- `signallab` executa na imagem com **rede do host**: o que o runner
  alcança, ele alcança. Um serviço que o job iniciou no runner — o seu sistema
  em teste, ou um contêiner de `services:` com uma porta publicada — está em
  `127.0.0.1`.
- Ela executa como o usuário do runner, com o workspace montado no mesmo caminho,
  de modo que o relatório pertence ao job.
- Ela passa a `run` exatamente as entradas acima e `--junit`. Para qualquer outra coisa que
  `signallab` faça — `--report`, `--seed`, `--json`, `emulate` — use
  [a imagem](#docker) diretamente.

## A imagem, em qualquer CI {#docker}

A imagem do servidor traz `signallab` como `/usr/local/bin/signallab`. Substitua o
entrypoint para usá-lo.

**GitLab CI:**

```yaml
signallab:
  image:
    name: ghcr.io/proanima/signallab:[[version]]
    entrypoint: [""]
  script:
    - signallab run tests/signallab/*.json --junit signallab-junit.xml
  artifacts:
    when: always
    reports:
      junit: signallab-junit.xml
```

Os segredos são variáveis de CI/CD chamadas `SIGNALLAB_SECRET_<NAME>` (marque-as como mascaradas);
o ambiente do job as entrega a `signallab` como estão.

**Docker, a partir de um shell ou de qualquer agendador** (cron, Jenkins, um script de deploy):

```bash
docker run --rm --network host \
  --user "$(id -u):$(id -g)" \
  -v "$PWD:/work" -w /work \
  -e SIGNALLAB_SECRET_API_TOKEN \
  --entrypoint signallab \
  ghcr.io/proanima/signallab:[[version]] \
  run tests/smoke.json --junit junit.xml
echo "signallab exited with $?"
```

- `--network host` deixa as execuções alcançarem o que o host alcança, broadcast e
  multicast incluídos — em um host Linux. Sem ele, o contêiner alcança outros
  hosts só por unicast.
- A imagem executa como um usuário sem privilégios (uid 10001). `--user` a executa como você,
  de modo que ela pode escrever o relatório na sua pasta; sem ele, a pasta
  precisa ser gravável para o uid 10001.
- `-e NAME` sem valor passa essa variável do seu ambiente.

## O binário {#binary}

Cada versão traz `signallab` sozinho:
`signallab-<version>-linux-x64.tar.gz` e `signallab-<version>-windows-x64.zip`,
listados no `SHA256SUMS.txt` da versão. O de Linux é construído no Ubuntu
22.04 e usa o OpenSSL 3 do sistema (`libssl3`): executa nessa versão ou em uma
distribuição mais nova.

```yaml
      - name: Signal Lab
        run: |
          curl -fsSL -o signallab.tar.gz https://github.com/ProAnima/SignalLab/releases/download/v[[version]]/signallab-[[version]]-linux-x64.tar.gz
          tar -xzf signallab.tar.gz
          ./signallab run tests/signallab/smoke.json --junit signallab-junit.xml
```

Em um runner Windows, descompacte o zip e execute `signallab.exe` do mesmo modo. Uma
máquina com o aplicativo de desktop instalado já tem `signallab` no `PATH`.

## Execuções em um servidor de laboratório {#lab-server}

O equipamento na rede de uma instalação é alcançável do laboratório, não de um runner
na nuvem. Execute ali um [servidor do Signal Lab](../server/index.md), guarde o token dele como
segredo de CI e envie as execuções a ele:

```bash
signallab run tests/stage.json --server http://192.0.2.10:1430 --token-file token.txt --junit junit.xml --report reports/
```

```yaml
      - uses: ProAnima/SignalLab@v[[version]]
        with:
          experiments: tests/signallab/stage.json
          server: http://192.0.2.10:1430
          token: ${{ secrets.SIGNALLAB_TOKEN }}
```

O experimento vem do checkout do pipeline; o servidor o executa com a
rede, os segredos e a pasta de dados dele, transmite as etapas de volta e guarda o
relatório (`--report` baixa uma cópia). O token vem de `--token-file` ou
`SIGNALLAB_TOKEN`. Os códigos de saída são os mesmos; um servidor que não pode ser alcançado
ou recusa o token é `3`. O runner precisa conseguir alcançar o servidor — um
runner próprio no laboratório, ou um endereço de servidor que o runner possa abrir.

Para executar no servidor sem `signallab` nenhum, um script pode chamar a API HTTP dele
diretamente: veja [Execução de um experimento por HTTP](../api/run.md).

## Uma matriz de execuções {#matrix}

Um experimento, todos os destinos: cada combinação é uma execução à parte e uma suíte de
testes à parte no relatório JUnit, com o nome dos valores dela.

```bash
signallab run tests/smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest
```

Na action, um nome por linha:

```yaml
        with:
          experiments: tests/signallab/smoke.json
          matrix: |
            device=192.0.2.20:9000,192.0.2.21:9000
            user=admin,guest
          fail-fast: "true"
```

Ou a partir de um arquivo, com `--matrix-file` (`matrix-file` na action):

```json
[
  { "device": "192.0.2.20:9000", "user": "admin" },
  { "device": "192.0.2.21:9000", "user": "guest" }
]
```

Toda combinação é verificada antes de a primeira enviar qualquer coisa, no máximo 256
execuções saem de um comando, e `--fail-fast` deixa o resto sem iniciar — elas
aparecem no relatório JUnit como ignoradas. As regras estão na
[página da linha de comando](cli.md#matrix).

Para testar todos os perfis de um experimento, execute uma vez por perfil — uma etapa cada, ou
uma matriz de jobs do seu CI — com `--profile`.

## Relatórios JUnit {#junit}

`--junit PATH` (a action sempre escreve um) contém uma suíte de testes por execução e um
caso de teste por nó: a falha onde aconteceu, no idioma escolhido, com
o código de erro e o detalhe técnico; os nós que uma execução nunca alcançou como ignorados;
a semente, o resultado, o arquivo e os valores da matriz como propriedades. O GitHub
(com uma action de relatórios), o GitLab (`artifacts:reports:junit`), o Jenkins e
o Azure DevOps o mostram como resultados de testes. A estrutura dele está na
[página da linha de comando](cli.md#reports).

A semente de uma execução com falha está nas propriedades da suíte dela e no log:
`--seed <that number>` a executa de novo com os mesmos valores aleatórios.

## Dependências que o sistema em teste chama {#emulators}

Para testar o seu próprio sistema contra uma API, um dispositivo ou um broker que não existe
no CI, deixe o Signal Lab fazer o papel dele:

- **Dentro de um experimento**, um nó [[ui:exp.node.emulator]] faz o papel da
  dependência por uma execução, e [[ui:exp.node.wait_http]] verifica o que o seu sistema
  enviou a ela. A saída da execução termina com o que cada emulador recebeu. Veja
  [Emuladores](../tools/emulators.md).
- **Em torno dos seus próprios testes**, `signallab emulate` responde em segundo plano
  enquanto eles executam, e as contagens dele dizem o que foi chamado:

  ```bash
  signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
  npm test
  wait
  ```

## Saída para um script {#json}

`--json` imprime um objeto JSON por linha em stdout e nada mais: `started`,
cada `step`, `ended` com o resultado inteiro e um `summary` com `total`,
`passed`, `failed`, `not_started` e `exit_code`. Os erros carregam o `code` estável
do motor, de modo que um script pode decidir por ele em qualquer idioma. Veja
[Saída](cli.md#output).

```bash
signallab run tests/smoke.json --json | jq -c 'select(.type == "ended") | {file, outcome, seed}'
```

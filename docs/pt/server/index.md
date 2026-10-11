---
title: Execução de um servidor
description: Execute o Signal Lab em uma máquina Linux ao lado do equipamento e use-o de qualquer navegador da rede, com uma API HTTP para scripts e CI.
---

# Execução do Signal Lab como servidor

`signal-lab-server` é o Signal Lab sem janela: o mesmo motor, servindo a mesma
interface a um navegador. Coloque-o em uma máquina ao lado do equipamento — um PC de rack, uma
VM de controle de show, uma máquina de laboratório compartilhada — e abra-o no Chrome, Firefox ou Edge
de qualquer lugar da rede: todas as telas funcionam como no aplicativo de desktop, e execuções,
relatórios e exportações são baixados pelo navegador.

Scripts e pipelines usam o mesmo servidor pela [API HTTP](../api/index.md) dele,
e [`signallab --server`](../automation/cli.md#run-on-server) envia execuções a ele.

Algumas coisas são diferentes do aplicativo de desktop:

- **Um servidor é um motor.** Toda página conectada a ele vê as mesmas tarefas em
  execução, as mesmas bibliotecas de sinais e de emuladores e o mesmo armazenamento de cookies da
  tela [[ui:nav.http]] ([[ui:http.keepCookies]]). Quem compartilha um servidor
  compartilha tudo isso.
- **Os segredos são os do servidor**, lidos do ambiente ou de arquivos dele; não
  podem ser definidos pelo navegador (veja [Segredos](#secrets)).
- **O firewall é o do host.** O servidor nunca o altera; o aviso de firewall do
  aplicativo de desktop não aparece.
- **Ele se atualiza com a imagem dele**, não pelo atualizador do app (veja
  [Atualização](#update)).

## Em um host Linux, em um comando {#install-script}

Em uma máquina Linux com acesso à internet:

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh
```

O script:

1. instala o Docker se ele não existir — pergunta antes, usando o instalador do
   próprio Docker (`get.docker.com`);
2. escreve um `compose.yaml` em `/opt/signallab` (`~/signallab` quando você não é
   root);
3. baixa a imagem e inicia o servidor com rede do host, de modo que OSC, UDP,
   broadcast, multicast e descoberta alcancem a rede real;
4. aguarda até o servidor responder à verificação de saúde dele (até 90 segundos);
5. imprime os endereços a abrir, o token de acesso para entrar e os
   comandos para atualizar, ler os logs e removê-lo;
6. quando ufw ou firewalld está ativo, oferece abrir a porta do servidor (veja
   [O firewall do host](#firewall)).

Passe opções depois de `sh -s --`:

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh -s -- --version [[version]] --port 8430
```

| Opção | O que faz | Padrão |
| --- | --- | --- |
| `--version X.Y.Z` | A versão da imagem (`latest` ou `X.Y.Z`; um `v` inicial é descartado). | `latest` |
| `--port N` | A porta que os navegadores usam. | `1430` |
| `--listen IP:PORT` | Escuta em um só endereço. | `0.0.0.0:<port>` |
| `--dir DIR` | Onde vai o arquivo compose. | `/opt/<name>` como root, `~/<name>` nos demais casos |
| `--name NAME` | O contêiner e o volume de dados dele: letras minúsculas, dígitos, `-` e `_`. Um segundo servidor no mesmo host precisa de um nome próprio. | `signallab` |
| `--image NAME` | Outra imagem ou registro; um `NAME:TAG` completo é usado como está. | `ghcr.io/proanima/signallab` |
| `--open-udp PORTS` | Com um firewall ativo, também deixa o UDP entrar nestas portas, para monitores e esperas: `9000,9100:9110`. | |
| `--no-firewall` | Nunca altera ufw nem firewalld. | |
| `--yes`, `-y` | Responde sim: instala o Docker, abre o firewall, apaga os dados com `--purge`. | |
| `--uninstall` | Para e remove o servidor; os dados ficam. | |
| `--purge` | Com `--uninstall`: apaga também os dados e o token. | |
| `--help`, `-h` | Imprime as opções. | |

Ele precisa do plugin Compose do Docker (o pacote `docker-compose-plugin`), que
o instalador do Docker traz. A imagem é construída para x86_64 e arm64.

**Execute de novo para atualizar:** o mesmo comando baixa a imagem mais nova (ou a
`--version` que você der) e reinicia o servidor; os dados e o token ficam.
Com `--name`, dê o mesmo nome de novo.

**Configurações suas** — segredos de experimentos, `SIGNALLAB_ALLOWED_HOSTS`,
`SIGNALLAB_SECURE_COOKIE` atrás de HTTPS — vão em `compose.override.yaml` ao lado
do arquivo compose. O Docker Compose o mescla, e o script reescreve
`compose.yaml` a cada execução mas nunca toca no override. Um `compose.yaml` que ele
não escreveu é guardado como `compose.yaml.before-install`.

```yaml
# compose.override.yaml
services:
  signallab:
    environment:
      SIGNALLAB_ALLOWED_HOSTS: lab-pc.example.com,192.0.2.10
      SIGNALLAB_SECRET_API_TOKEN: ${API_TOKEN}
```

**Para removê-lo:**

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh -s -- --uninstall
```

Os dados ficam no volume Docker dele, e instalar de novo os traz de volta com o
mesmo token. `--uninstall --purge` apaga também os dados e o token, depois de
perguntar.

### O firewall do host {#firewall}

Com rede do host, o servidor escuta nas portas do próprio host, então o firewall
do host decide quem o alcança. Quando ufw ou firewalld está ativo, o script pergunta
antes de abrir a porta TCP do servidor e as portas UDP de `--open-udp`, anota
o que abriu (`.firewall` ao lado do arquivo compose), e `--uninstall`
fecha exatamente isso de novo. Se você recusar, navegadores em outras máquinas alcançam o
servidor só depois que o firewall os deixa, e monitores e esperas ouvem outras
máquinas só nas portas UDP que ele abre.

## Docker {#docker}

### A imagem {#image}

`ghcr.io/proanima/signallab`, para `linux/amd64` e `linux/arm64`, publicada
a cada versão:

| Tag | O que é |
| --- | --- |
| `X.Y.Z` | Essa versão. |
| `X.Y` | A versão estável mais nova dessa linha. |
| `latest` | A versão estável mais nova. |

Ela contém `signal-lab-server`, a interface construída e a linha de comando
`signallab`, e inicia o servidor com estas configurações:

| Variável | Valor na imagem |
| --- | --- |
| `SIGNALLAB_LISTEN` | `0.0.0.0:1430` |
| `SIGNALLAB_DATA_DIR` | `/data` |
| `SIGNALLAB_UI_DIR` | `/usr/share/signal-lab/ui` |
| `SIGNALLAB_GENERATE_TOKEN` | `true` |

Ela executa como um usuário sem privilégios (uid e gid 10001), escreve só em `/data` (um
volume), expõe a porta `1430` e verifica a própria saúde a cada 30 segundos.

### Como iniciá-la {#docker-run}

```bash
docker run -d --name signallab --network host --restart unless-stopped \
  -v signallab-data:/data --read-only --cap-drop ALL --security-opt no-new-privileges \
  ghcr.io/proanima/signallab:[[version]]
docker logs signallab                    # no primeiro início: "Sign in with it:" e o token
```

Depois abra `http://<host>:1430` e entre com o token. Para ver o token
de novo mais tarde:

```bash
docker exec signallab cat /data/token
```

`--read-only`, `--cap-drop ALL` e `no-new-privileges` são opcionais e não custam
nada: o servidor não precisa de privilégios e escreve só em `/data`.

### Docker Compose {#compose}

O `deploy/compose.yaml` do repositório é o mesmo, como arquivo Compose:

```yaml
name: signallab

services:
  signallab:
    image: ${SIGNALLAB_IMAGE:-ghcr.io/proanima/signallab:latest}
    container_name: signallab
    network_mode: host
    environment:
      SIGNALLAB_GENERATE_TOKEN: "true"
    volumes:
      - signallab-data:/data
    read_only: true
    cap_drop: [ALL]
    security_opt: ["no-new-privileges:true"]
    restart: unless-stopped
    stop_grace_period: 15s

volumes:
  signallab-data:
```

```bash
docker compose up -d
docker exec signallab cat /data/token
```

Defina `SIGNALLAB_IMAGE=ghcr.io/proanima/signallab:X.Y.Z` para fixar uma versão.

### Rede {#networking}

| Rede do Docker | O que funciona | O que não funciona |
| --- | --- | --- |
| `--network host` (`network_mode: host`), em um host Linux | Tudo: OSC, UDP, TCP, HTTP, WebSocket e MQTT para a LAN, portas de escuta, broadcast, multicast, descoberta. | — |
| Bridge (o padrão), com portas publicadas | Unicast para os hosts que o contêiner alcança; ouvintes em portas publicadas (`-p 1430:1430 -p 9000:9000/udp`). | Broadcast e multicast; respostas a portas que não estão publicadas. |
| Docker Desktop no Windows ou macOS | Unicast e ouvintes publicados. | Rede do host para a rede física. No Windows, use o aplicativo de desktop. |

### O volume de dados {#data-volume}

`/data` guarda tudo o que o servidor mantém: o experimento, as bibliotecas de sinais e de
emuladores, os relatórios de execução, as exportações e o token. Um volume nomeado, como
acima, começa pertencendo ao usuário da imagem. Uma pasta do host montada ali
precisa ser gravável pelo uid 10001:

```bash
sudo mkdir -p /srv/signallab && sudo chown 10001:10001 /srv/signallab
docker run -d --name signallab --network host -v /srv/signallab:/data ghcr.io/proanima/signallab:[[version]]
```

## Sem Docker {#binary}

O servidor é publicado como a imagem. Para executar `signal-lab-server` diretamente, construa-o
a partir do código-fonte (veja [Compilação](../../develop/building.md)):

```bash
npm install
npm run build                            # a interface, em dist/
cargo run --release -p signal-lab-server
```

Ele serve `dist/` em `http://127.0.0.1:1430`, só para esta máquina, sem precisar
de token. Acrescente `--listen` e um token para abri-lo à rede.

## Opções {#options}

Toda opção tem uma variável de ambiente, para contêineres. As opções vencem as
variáveis.

| Opção | Variável | Padrão | O que faz |
| --- | --- | --- | --- |
| `--listen IP:PORT` | `SIGNALLAB_LISTEN` | `127.0.0.1:1430` | Onde escutar. Qualquer endereço que não seja loopback exige um token. |
| `--token-file PATH` | `SIGNALLAB_TOKEN_FILE` | | Um arquivo com o token de acesso (um segredo do Docker, por exemplo). |
| `--token TOKEN` | `SIGNALLAB_TOKEN` | | O próprio token de acesso. Prefira o arquivo: os argumentos são visíveis para outros usuários da máquina. |
| `--generate-token` | `SIGNALLAB_GENERATE_TOKEN` | desligado | Sem token dado, em um endereço além do loopback: usa o token guardado em `<data folder>/token`, criando-o no primeiro início. |
| `--data-dir PATH` | `SIGNALLAB_DATA_DIR` | `Documents/SignalLab` na pasta pessoal do usuário | A pasta de dados. |
| `--secrets-dir PATH` | `SIGNALLAB_SECRETS_DIR` | `/run/secrets/signallab` | Uma pasta de segredos somente leitura, um arquivo por nome. |
| `--ui-dir PATH` | `SIGNALLAB_UI_DIR` | `ui` ao lado do programa, senão `./dist` | A interface construída. Sem uma, só a API é servida. |
| `--allowed-host NAME` | `SIGNALLAB_ALLOWED_HOSTS` | | Nomes de host pelos quais o servidor pode ser alcançado, separados por vírgulas. Eles e os nomes de loopback são sempre aceitos, com token ou sem; sem nenhum dado, um servidor com token responde a qualquer nome e um sem token só a nomes de loopback (veja [Nomes de host](security.md#hosts)). |
| `--secure-cookie` | `SIGNALLAB_SECURE_COOKIE` | desligado | Envia o cookie de sessão só por HTTPS. Defina-o atrás de um proxy HTTPS. |
| `--log FILTER` | `SIGNALLAB_LOG` | `info` | O que registrar: `error`, `warn`, `info`, `debug`, ou por módulo (`signal_lab_server=debug`). |
| `--log-format text\|json` | `SIGNALLAB_LOG_FORMAT` | `text` | Linhas de log como texto, ou um objeto JSON por linha. |

As chaves aceitam `true` ou `false` na variável delas:
`SIGNALLAB_GENERATE_TOKEN=true`.

| Comando | O que faz |
| --- | --- |
| `signal-lab-server token` | Imprime um novo token aleatório: 64 caracteres hexadecimais. |
| `signal-lab-server healthcheck` | Sai com `0` quando um servidor responde em `--listen` (a verificação de saúde da imagem). |
| `signal-lab-server --version` | Imprime a versão. |
| `signal-lab-server --help` | Imprime todas as opções. |

**Códigos de saída:** `0` quando parado por <kbd>Ctrl</kbd>+<kbd>C</kbd> ou `SIGTERM`;
`2` quando as configurações são recusadas (nenhum token em um endereço alcançável, um token curto
demais, um token dado duas vezes, um arquivo de token que não pode ser lido,
`--generate-token` sem pasta de dados); `1` quando não consegue escutar no
endereço, ou a pasta de dados não pode ser escrita. O motivo é impresso em stderr.

## O token de acesso {#token}

Sem um token, o servidor escuta só no loopback e atende só esta
máquina. Em qualquer outro endereço ele precisa de um e se recusa a iniciar sem ele.
Há três maneiras de dá-lo:

| Como | Quando usar |
| --- | --- |
| `--generate-token` (ativo na imagem) | Nada a configurar: no primeiro início o servidor cria um token, o salva em `<data folder>/token` — legível só pelo próprio usuário dele — e o imprime uma vez no log. Os inícios seguintes o reutilizam, de modo que navegadores e scripts conectados continuam funcionando entre reinícios e atualizações. Exige uma pasta de dados (`--data-dir`). |
| `--token-file PATH` | Um token seu em um arquivo, como um segredo do Docker. Uma quebra de linha no fim não faz parte do token. |
| `SIGNALLAB_TOKEN` | O token no ambiente. |

Um token tem pelo menos **24** caracteres e nenhum espaço nem quebra de linha; dê-o de
uma só maneira. `signal-lab-server token` cria um bom:

```bash
docker run --rm ghcr.io/proanima/signallab:[[version]] token > signallab_token.txt
```

e o Compose o entrega como um segredo (o arquivo precisa ser legível pelo uid 10001):

```yaml
services:
  signallab:
    environment:
      SIGNALLAB_TOKEN_FILE: /run/secrets/signallab_token
    secrets:
      - signallab_token

secrets:
  signallab_token:
    file: ./signallab_token.txt
```

Um token dado explicitamente vence `--generate-token`, e nada é criado então.
Para trocar um token criado, pare o servidor, apague `<data folder>/token` e inicie-o
de novo: ele cria e imprime um novo. Um arquivo de token danificado é
informado, nunca substituído.

## Entrada {#sign-in}

Abra `http://<host>:1430`. Um servidor com token leva o navegador primeiro à página
de entrada, no idioma do navegador; cole o token uma vez, e o navegador
fica conectado por 7 dias. [[ui:app.signOut]] na interface encerra a
sessão. As sessões ficam na memória do servidor: um reinício desconecta
todos.

No loopback sem token, não há entrada.

Os scripts enviam o token a cada requisição, como `Authorization: Bearer <token>`:

```bash
curl -fsS http://192.0.2.10:1430/api/invoke/app_info \
  -H "Authorization: Bearer $(cat signallab_token.txt)" \
  -H "Content-Type: application/json" -d 'null'
```

Veja [A API HTTP](../api/index.md) e [Segurança do servidor](security.md) para
as regras por trás de tudo isso.

## A pasta de dados {#data}

O servidor guarda os arquivos dele na pasta de dados: o experimento, as bibliotecas de sinais e de
emuladores, os relatórios de execução, as exportações e um token criado. Ela é
`--data-dir` (`SIGNALLAB_DATA_DIR`), `/data` na imagem, e, nos demais casos,
`Documents/SignalLab` na pasta pessoal do usuário com que ele executa. Uma pasta de dados
dada com `--data-dir` é criada se não existir e verificada na inicialização: se o
servidor não pode escrever ali, ele para e nomeia a pasta. Veja
[Arquivos](../reference/files.md).

Os downloads (relatórios, exportações) vêm só de dentro desta pasta.

## Segredos {#secrets}

Os experimentos leem segredos como `{{secret.NAME}}`. Em um servidor, eles são somente leitura,
vindos de:

1. a variável de ambiente `SIGNALLAB_SECRET_NAME`, senão
2. o arquivo `NAME` na pasta de segredos (`--secrets-dir`, por padrão
   `/run/secrets/signallab` — o layout dos segredos do Docker).

A quebra de linha final de um arquivo não faz parte do valor; um valor vazio conta como
não definido; um valor tem no máximo 16 KiB. Os nomes são letras, dígitos e `_`, sem
começar por dígito. A interface mostra quais segredos estão definidos no servidor,
mas defini-los pelo navegador é recusado — os valores nunca vão para um lugar mais fraco
e nunca saem de volta (veja [Segredos](security.md#secrets)).

Com o Compose, um arquivo de segredo por nome:

```yaml
services:
  signallab:
    secrets:
      - source: api_token
        target: /run/secrets/signallab/API_TOKEN

secrets:
  api_token:
    file: ./api_token.txt
```

O arquivo precisa ser legível pelo uid 10001.

## Atrás de um proxy HTTPS {#https}

O servidor fala HTTP simples. Para HTTPS, ponha na frente dele um proxy reverso
(Caddy, nginx, Traefik) que:

- repasse o cabeçalho `Host` sem alterá-lo;
- repasse os upgrades de WebSocket (a interface mantém um aberto, para `/api/events`);

e inicie o servidor com `--secure-cookie`, de modo que o cookie de sessão viaje só por
HTTPS, e com `--allowed-host` definido com o nome que as pessoas usam.

## Atualização {#update}

[[ui:update.server]]: o atualizador do aplicativo de desktop não tem nada a ver com isso.

- Instalado com o script: execute o mesmo comando de novo.
- Com Compose: `docker compose pull && docker compose up -d`.
- Com `docker run`: baixe a nova imagem, depois remova o contêiner e inicie-o
  de novo com o mesmo volume.

Os dados e o token estão no volume, então ficam.

## Verificação de saúde {#health}

`GET /api/health` responde a todos, sem token:

```bash
curl -s http://127.0.0.1:1430/api/health
# {"auth":true,"status":"ok","version":"[[version]]"}
```

`auth` diz se o servidor pede um token. O comando
`signal-lab-server healthcheck` pergunta o mesmo nesta máquina e sai com `0` quando
ele responde; a imagem o executa a cada 30 segundos (tempo limite de 5 segundos, 3 tentativas), de modo que
`docker ps` mostra o contêiner como saudável.

## Logs {#logs}

O servidor registra em stdout: `docker logs -f signallab`. No nível padrão
(`info`) ele diz onde escuta e se precisa de um token, as pastas de dados e de
interface, cada tarefa iniciada — monitores, geradores, tempestades, varreduras,
execuções, emuladores — com o endereço do cliente, cada entrada, cada entrada com um
token errado (como aviso) e quando uma página fica para trás nos eventos. O nível
`--log debug` acrescenta cada comando. As linhas só são coloridas em um terminal (nunca com
`NO_COLOR` definido); `--log-format json` escreve um objeto JSON por linha para um coletor
de logs.

## Parada {#stop}

<kbd>Ctrl</kbd>+<kbd>C</kbd>, `docker stop` ou `SIGTERM` fecha a conexão de cada
página, para todas as tarefas — uma execução em andamento termina como parada — e sai com
`0`. Os arquivos compose dão a ele 15 segundos para isso.

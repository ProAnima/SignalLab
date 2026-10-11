---
title: Solução de problemas
description: Problemas comuns do Signal Lab e como corrigi-los — nada chega, uma porta em uso, o firewall, a rede do Docker, o servidor, MQTT, certificados, atualizações e registros.
---

# Solução de problemas

Toda falha que o Signal Lab relata tem um código; a mensagem de cada um está em
[mensagens de erro](errors.md). As falhas de rede são os códigos [`transport`](errors.md#transport):
`refused`, `timeout`, `dns`, `unreachable`, `reset`, `address_in_use`, `address_unavailable`,
`denied`, `tls`, `target_invalid`, `exhausted`, `failed`. Os problemas abaixo são os comuns.

## Nada chega {#nothing-arrives}

Primeiro descubra se algo chega ao Signal Lab: abra a aba [[ui:dock.inspector]] no painel
inferior e pressione [[ui:ins.arm]]. Todo datagrama, requisição e mensagem que uma ferramenta
envia ou recebe é listado ali, com a sua origem.

### O endereço de escuta {#listen-address}

- Um [[ui:common.bind]] de `0.0.0.0:<port>` escuta em todas as placas de rede;
  `127.0.0.1:<port>` ouve só esta máquina. O equipamento na rede precisa do primeiro.
- O dispositivo deve enviar para o endereço desta máquina e para a porta em que você escuta. O
  cabeçalho mostra o nome e o endereço desta máquina.
- Um endereço que não é desta máquina falha com `address_unavailable`.

### O firewall {#firewall}

O tráfego em `127.0.0.1` nunca é filtrado, e é por isso que um teste em uma máquina funciona
enquanto o mesmo teste a partir de outra máquina não recebe nada.

**Windows.** O Firewall do Windows decide por programa. O Windows costuma perguntar uma vez, na
primeira vez que um programa escuta — e um *Cancelar* ali deixa uma regra que o bloqueia, e que
vale mais que qualquer regra de permissão. Em uma rede que o Windows chama de pública (o Wi-Fi
de um local de evento, muitas vezes), ele pode nem perguntar.

- O aplicativo de desktop olha o firewall uma vez, quando um monitor, um ouvinte de descoberta,
  um retransmissor, uma execução ou um emulador começa a escutar. Quando o firewall atrapalha,
  ele avisa em uma notificação com [[ui:fw.allow]] — ou [[ui:fw.allowPublic]] em uma rede
  pública. O Windows pede direitos de administrador, e então as regras de entrada do programa,
  inclusive uma de bloqueio, são substituídas por uma única regra de permissão.
  [[ui:fw.dismiss]] oculta o aviso.
- Em um terminal: `signallab doctor` mostra o que atrapalha, e `signallab firewall allow` o
  corrige (`--public` também para redes públicas), com o mesmo pedido de administrador.
- Um instalador (`.exe`) instalado *para todos* acrescenta ele mesmo as regras de permissão
  (redes privadas e de domínio), a menos que tenha sido executado com `/NOFIREWALL`. Um
  instalador *só para mim* não pode, e o `.msi` deixa o firewall a cargo de quem o implanta.
- Um servidor nunca altera o firewall do seu host: o administrador dele abre as portas (o
  script de instalação se oferece para isso, com ufw ou firewalld).

**Linux.** Um firewall como ufw ou firewalld trabalha por porta, não por programa.
`signallab doctor` nomeia o que está ativo e como abrir uma porta, por exemplo
`sudo ufw allow 9000/udp`.

### Broadcast e multicast {#broadcast-multicast}

- Os roteadores não encaminham broadcast: `255.255.255.255` e `x.x.x.255` alcançam só o
  segmento de rede em que a placa de envio está. Com várias placas, ponha o endereço da placa
  em [[ui:bc.bindSource]] (em [[ui:bc.socketOptions]]), por exemplo `10.0.0.5:0`.
- Um datagrama multicast alcança só os ouvintes que entraram no seu grupo — no ouvinte de
  descoberta, [[ui:bc.joinGroups]]. Com [[ui:bc.ttl]] em 1, o padrão, ele fica nesta rede.
- Para ouvir o seu próprio multicast na mesma máquina, deixe [[ui:bc.mcastLoop]] ligado.

### Um dispositivo que não responde {#no-answer}

O UDP não tem confirmação de entrega: um datagrama enviado a uma porta em que ninguém escuta
ainda conta como enviado. O Windows então relata o *port unreachable* ICMP que recebeu de volta
como uma redefinição de conexão na próxima leitura desse socket; os monitores, ouvintes,
retransmissores e esperas do Signal Lab o ignoram e continuam escutando. Assim, quando uma
resposta não vem, uma espera falha com `wait.timeout` depois do seu tempo, não com um erro sobre
o envio. Confira no [[ui:dock.inspector]] que a mensagem saiu para o endereço certo e depois
verifique o dispositivo.

## Um envio é recusado antes de sair {#refused-send}

Estes itens são verificados primeiro, e nada é enviado quando um falha:

| Código | Por quê | Correção |
| --- | --- | --- |
| `transport.target_invalid` | O destino não tem porta, ou não é nem `IP:port` nem `host:port` | Escreva os dois, como `192.0.2.20:9000` |
| `transport.dns` | O nome de host não resolve nesta máquina | Verifique o nome, ou use o endereço. Um nome com endereço IPv4 é alcançado por IPv4, então `localhost:9000` encontra um receptor em `127.0.0.1` |
| `node.osc_address` | Um endereço OSC não começa com `/` — no envio, no gerador, em um sinal ou em uma etapa | Comece-o com `/`, como `/cue/go` |
| `node.topic_wildcard` | O tópico de uma publicação MQTT tem um `+` ou `#`, na conexão da tela como em qualquer outro lugar | Publique em um só tópico; os curingas servem para assinar |
| `node.too_long` em uma mensagem WebSocket | A mensagem passa de 16 MiB | Envie menos; a conexão continua aberta |

## Uma porta já está em uso {#port-in-use}

`transport.address_in_use`: outro programa — ou outra tarefa do Signal Lab — já escuta nessa
porta.

- **Um monitor e uma execução.** Uma execução abre as portas das suas esperas, emuladores e
  retransmissores antes da primeira etapa, então uma porta mantida por um [[ui:osc.monitor]],
  um ouvinte de descoberta ou uma tarefa de emulador faz a execução falhar antes de começar.
  Pare essa tarefa primeiro; [[ui:app.stopAll]] para todas.
- **Dois emuladores** em uma execução não podem dividir uma porta de um mesmo transporte: os
  emuladores HTTP, MQTT e TCP escutam todos em TCP, e os OSC e UDP em UDP
  (`emulator.bind_taken`).
- **Escutar ao lado do serviço real.** O ouvinte de descoberta pode dividir uma porta com um
  programa que já a mantém: deixe [[ui:bc.reuse]] ligado. No Linux, esse programa também precisa
  compartilhar a porta. Sem isso, uma porta ocupada é `broadcast.port_shared`.
- **Acabou de parar.** Uma porta que um emulador ou uma execução parada mantinha é liberada
  um instante depois; um emulador iniciado de novo logo em seguida a espera por pouco tempo.
- **Portas abaixo de 1024** no Linux precisam de direitos de administrador (`denied`). A imagem
  do servidor roda sem nenhum, então use uma porta de 1024 ou mais.

## O servidor no Docker não alcança a rede {#docker-network}

Broadcast, multicast e descoberta só alcançam a rede física com a rede do host —
`network_mode: host` no arquivo compose, ou `docker run --network host` — e só em um host
Linux. Ela também deixa monitores, esperas e emuladores escutarem nas portas do próprio host.
Com a rede bridge padrão do Docker, o contêiner fica em uma rede própria: broadcast e multicast
nunca saem dela, e só as portas que você publica o alcançam.

No Windows e no macOS, a rede de host do Docker não alcança a rede física: use o aplicativo de
desktop no Windows, ou rode o servidor em um host Linux.

## O SmartScreen avisa sobre o instalador {#smartscreen}

Os instaladores ainda não são assinados, então o Windows SmartScreen diz que não conhece o
editor. Escolha *Mais informações* e depois *Executar assim mesmo*. Baixe instaladores só das
versões do projeto no GitHub.

## O servidor não inicia {#server-start}

O `signal-lab-server` verifica as suas configurações antes de escutar e termina com uma
mensagem na saída de erro:

| Código de saída | Mensagem | Correção |
| --- | --- | --- |
| 2 | refusing to listen on … without a token | Um servidor que outros podem alcançar precisa de um token: `--token-file` ou `SIGNALLAB_TOKEN` (crie um com `signal-lab-server token`), ou `--generate-token`. Ou escute em `127.0.0.1` |
| 2 | the token has N characters; it needs at least 24 | Use um token mais longo |
| 2 | the token must not contain spaces or line breaks | Um arquivo de token pode terminar com uma quebra de linha; nada além disso |
| 2 | give the token once | Use `--token`/`SIGNALLAB_TOKEN` ou `--token-file`/`SIGNALLAB_TOKEN_FILE`, não os dois |
| 2 | --generate-token keeps the token in the data folder | Defina `--data-dir` ou `SIGNALLAB_DATA_DIR` |
| 2 | … it does not hold a valid token; remove it to have a new one made | O arquivo `token` da pasta de dados está danificado |
| 2 | cannot read the token file … | O arquivo que `--token-file` nomeia não existe ou não é legível por este usuário |
| 2 | cannot save the new token in … | `--generate-token` não conseguiu gravar `token` na pasta de dados: torne a pasta gravável por este usuário |
| 1 | cannot listen on … | O endereço não é desta máquina, ou a porta está ocupada |
| 1 | the data folder … must be writable by this user | No Docker, uma pasta montada por bind precisa ser gravável pelo uid 10001 |

Um token que `--generate-token` criou é impresso uma vez, na primeira inicialização
(`docker logs signallab` o mostra), e guardado em `token` na pasta de dados:
`docker exec signallab cat /data/token`. Veja [o servidor](../server/index.md).

## Não consigo entrar no servidor {#sign-in}

| O que você vê | Por quê | Correção |
| --- | --- | --- |
| *That token is not right.* | Um token errado | Copie-o de novo de onde o servidor o guarda; a resposta leva um segundo de propósito |
| `auth.host` | O servidor não responde ao nome na barra de endereço | Abra-o por um nome que ele aceita. Nomes de loopback (`localhost`, `127.x.x.x`, `[::1]`) sempre passam. Sem token, o servidor só responde a esses e aos nomes de `--allowed-host`; com token, a qualquer nome, a menos que `--allowed-host` o restrinja |
| `auth.origin` | Uma requisição veio de uma página de outra origem | Atrás de um proxy reverso, repasse o `Host` do navegador ao servidor (nginx: `proxy_set_header Host $host;`), para que `Origin` e `Host` coincidam |
| De volta à página de entrada depois de entrar | O navegador não guardou o cookie de sessão | Com `--secure-cookie`, o servidor precisa ser alcançado por HTTPS |
| Desconectado depois de um tempo | As sessões duram 7 dias e terminam quando o servidor reinicia; passadas 1024 sessões, a mais antiga sai | Entre de novo |

## A conexão com o servidor não para de cair {#connection-lost}

[[ui:app.connectionLost]] significa que o socket de eventos da página, `/api/events`, fechou. A
página se reconecta sozinha, depois de meio segundo e então com menos frequência, até a cada
15 s. O que aconteceu nesse meio-tempo não é reproduzido: as tarefas em andamento voltam a
relatar à medida que continuam. Atrás de um proxy reverso, garanta que ele repasse upgrades de
WebSocket para `/api/events` e não feche conexões quietas em menos de 20 s (o servidor envia um
ping a cada 20 s). Uma página que diz que o servidor não tem tal endereço (`api.not_found`) é
mais antiga que o servidor: recarregue-a.

## O MQTT não conecta {#mqtt}

O Signal Lab fala MQTT 3.1.1 sobre TCP simples. Ele conecta e espera a resposta do broker
(CONNACK) antes de relatar sucesso, então o motivo aparece no botão que conecta:

| Código | Por quê | Correção |
| --- | --- | --- |
| `transport.refused` | Nada escuta nessa porta | Verifique a porta: 1883 é a habitual |
| `transport.timeout` | Nenhuma conexão TCP em 6 s | Verifique o endereço, a rede e o firewall do broker |
| `transport.dns` | O nome de host não resolve | Verifique o nome, ou use o endereço |
| `transport.unreachable` | Sem rota até o broker | Verifique a rede e o endereço |
| `transport.reset` | O broker fechou a conexão na hora | Muitas vezes é uma porta TLS (8883) — o Signal Lab não fala MQTT sobre TLS |
| `mqtt.no_answer` | A porta está aberta, mas nenhum CONNACK veio em 6 s | Muitas vezes é uma porta WebSocket — o Signal Lab não fala MQTT sobre WebSocket |
| `mqtt.protocol` | O que respondeu não é um broker MQTT | Verifique a porta |
| `mqtt.refused_protocol` | O broker não aceita MQTT 3.1.1 | Ative o 3.1.1 no broker |
| `mqtt.refused_client_id` | O broker recusa o id de cliente | Use outro id de cliente |
| `mqtt.refused_unavailable` | O broker está indisponível | Tente de novo mais tarde |
| `mqtt.refused_credentials` | O nome de usuário ou a senha estão errados | Verifique-os |
| `mqtt.refused_not_authorized` | O usuário não pode conectar | Verifique as regras de acesso do broker |
| `mqtt.client_id_required` | O id de cliente está vazio | Preencha-o |

Um broker derruba a mais antiga de duas conexões com o mesmo id de cliente: quando uma conexão
não para de fechar, procure outro cliente usando o mesmo id.

## Um certificado não é confiável {#tls}

`transport.tls`, em `https://` ou `wss://`: o certificado do servidor não é confiável, não
nomeia o host que você pediu, ou não foi possível acertar o TLS. O Signal Lab verifica os
certificados como o sistema faz e não tem chave para pular a verificação. HTTPS e WSS confiam
nos mesmos certificados: os do sistema operacional em que o motor roda — o repositório de
certificados do Windows no Windows, os certificados de CA do sistema no Linux e na imagem do
servidor. Para um certificado autoassinado ou a sua própria CA, acrescente-o aos certificados
confiáveis dessa máquina (para a imagem do servidor, uma imagem construída sobre ela que
acrescente o certificado) e conecte pelo nome que o certificado carrega.

## Falta um segredo {#secrets}

`secret.missing`: um experimento usa `{{secret.NAME}}` e nenhum valor está guardado sob esse
nome onde ele roda.

- **Aplicativo de desktop no Windows**: defina-o em [[ui:exp.secrets]], dentro de
  [[ui:exp.params]] no editor. Ele é guardado no Gerenciador de Credenciais do Windows, então uma
  máquina nova precisa que ele seja definido de novo.
- **Servidor**: ponha o valor no arquivo `/run/secrets/signallab/NAME` (ou na pasta que
  `--secrets-dir` nomeia) ou na variável de ambiente `SIGNALLAB_SECRET_NAME`. Um servidor não
  pode definir segredos pela página (`secret.read_only`).
- **Aplicativo de desktop no Linux** não tem repositório para segredos (`secret.unsupported`).
  Execute esse experimento com `signallab run`, que lê segredos de arquivos e variáveis, ou em
  um servidor.

Veja [arquivos](files.md#secrets).

## Uma execução para depois de cinco minutos {#run-timeout}

Uma execução que passa de 300 s falha com `run.timeout`; esse é o máximo que uma execução pode
levar. Um limite menor pode ser dado a uma execução pela API (`timeout` de
[`/api/run`](../api/run.md#request)) ou pela linha de comando (`signallab run --timeout`).

## O aplicativo não atualiza {#updates}

- O aplicativo de desktop procura uma nova versão uma vez por dia enquanto [[ui:update.auto]]
  está ligado, e quando você pressiona [[ui:update.check]] em [[ui:about.open]].
- Ele pergunta primeiro ao hub do estúdio e ao GitHub quando o hub não pode ser alcançado.
  Quando uma rede bloqueia os dois, [[ui:update.check]] dá [[ui:update.checkFailed]]; a
  verificação diária falha sem uma palavra.
- Só lhe são oferecidas versões publicadas, nunca rascunhos nem pré-lançamentos.
- Uma nova versão chega ao aplicativo quando o hub a oferece, o que pode levar algum tempo
  depois de ela aparecer no GitHub: o hub distribui uma versão a uma parte das instalações de
  cada vez.
- Ele só instala quando você pressiona [[ui:update.install]] — as tarefas em andamento são
  paradas antes — e só uma versão cuja assinatura confere: caso contrário,
  [[ui:update.installFailed]].
- Um servidor se atualiza com a sua imagem: `docker compose pull && docker compose up -d` na
  pasta do arquivo compose dele.

## Onde ficam os registros {#logs}

- **Aplicativo de desktop**: não grava arquivos de registro. A aba [[ui:console.title]] no
  painel inferior lista o que cada ferramenta fez e o que deu errado, e [[ui:feedback.open]] a
  anexa a uma mensagem aos desenvolvedores (sem o nome deste computador, o endereço dele nem as
  suas pastas). O relatório de cada execução está em `runs/` na pasta de dados.
- **Servidor**: registra na saída padrão e na saída de erro — `docker logs signallab` no
  Docker. Todo início de tarefa é registrado com o endereço do cliente que a pediu. `--log` (ou
  `SIGNALLAB_LOG`) define o nível: `error`, `warn`, `info` (o padrão) ou `debug`;
  `--log-format json` (ou `SIGNALLAB_LOG_FORMAT`) grava um objeto JSON por linha.
- **Linha de comando**: o `signallab` grava as suas mensagens na saída de erro; veja
  [a linha de comando](../automation/cli.md).

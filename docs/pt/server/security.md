---
title: Segurança do servidor
description: Quem pode usar um servidor Signal Lab e como ele mantém os outros do lado de fora — tokens, sessões, verificações de host e de origem, segredos — e o que o Signal Lab envia para o mundo exterior.
---

# Segurança do servidor

Um servidor Signal Lab envia tráfego real a partir da máquina em que roda: OSC, UDP, HTTP, MQTT,
tempestades, varreduras, broadcasts. Quem puder usá-lo pode fazer tudo isso a partir daquela
máquina, por isso o servidor vem fechado por padrão e só se abre com um token.

::: warning
Trate o token de acesso como uma senha para a rede da máquina. Quem o tiver pode enviar tráfego do
servidor para qualquer coisa que o servidor alcance.
:::

## Em resumo {#summary}

- **Sem token, só esta máquina.** Sem um token, o servidor escuta no loopback e só responde a nomes
  de host de loopback. Em qualquer outro endereço, ele se recusa a iniciar.
- **Um token para todos os demais.** Os navegadores entram uma vez e recebem um cookie de sessão;
  os scripts enviam o token em toda requisição.
- **Só as próprias páginas.** As requisições que alteram algo, e o WebSocket de eventos, precisam
  vir da origem do próprio servidor; os comandos só aceitam JSON.
- **Só os próprios nomes.** Um nome de host ao qual o servidor não responde é recusado, o que
  impede o DNS rebinding.
- **Os segredos ficam lá dentro.** Somente leitura, vindos do ambiente ou de arquivos, nunca
  devolvidos, mascarados onde quer que apareceriam.
- **Nada além da sua função.** Ele nunca altera o firewall do host, não serve nenhum arquivo fora
  da pasta de dados e não precisa de privilégios.

## Sem token: só esta máquina {#loopback}

Iniciado sem token, o servidor escuta em `127.0.0.1:1430` e não exige entrada: é uma ferramenta
para quem está diante desta máquina. Para impedir que uma página web em qualquer navegador desta
máquina o alcance por meio de um nome que resolve para `127.0.0.1` (DNS rebinding), ele só responde
a requisições cujo `Host` seja um nome de loopback — `localhost`, um nome terminado em
`.localhost`, `127.x.x.x` ou `[::1]` — ou um nome que você permita com `--allowed-host`.

Se lhe pedirem para escutar em qualquer outro endereço sem token, ele não inicia: diz por quê e
sai com o código `2`.

## O token {#token}

Um token tem pelo menos 24 caracteres, sem espaços nem quebras de linha.
`signal-lab-server token` imprime um aleatório de 64 caracteres hexadecimais, e `--generate-token`
(ativado na imagem) cria um no primeiro início, guarda-o na pasta de dados com leitura permitida só
ao próprio usuário do servidor e o imprime uma vez. Veja [O token de acesso](index.md#token) para
todas as formas de informar um.

A comparação de um token leva o mesmo tempo, seja qual for o ponto em que ele difere, e um token
errado custa uma espera de um segundo e um aviso no log — adivinhar é lento e deixa rastro.

## Navegadores: sessões {#sessions}

Um navegador que não entrou é enviado para a página de entrada. O token dele é trocado uma única
vez por uma sessão, guardada em um cookie que é:

- `HttpOnly` — nenhum script de uma página consegue lê-lo;
- `SameSite=Strict` — nenhuma página de outro site consegue fazer o navegador enviá-lo;
- válido por 7 dias;
- `Secure` com `--secure-cookie`, de modo que só trafega por HTTPS (ative atrás de um proxy HTTPS).

As sessões ficam na memória do servidor: um reinício desconecta todo mundo, e o botão [[ui:app.signOut]]
encerra uma na hora. São guardadas no máximo 1.024 sessões; a mais antiga sai primeiro.

## Scripts: o token Bearer {#bearer}

Um script, o `signallab --server` e a CI enviam o token em toda requisição:

```http
Authorization: Bearer <token>
```

Todo endpoint exige o token ou uma sessão, exceto `GET /api/health` (se o servidor responde, a
versão dele, se ele pede token) e a página de entrada. Uma requisição à API sem nenhum dos dois
recebe `401` com o erro `auth.required`; uma página recebe a página de entrada.

## Nomes de host {#hosts}

| Servidor iniciado | Nomes de host aos quais responde |
| --- | --- |
| sem token | nomes de loopback e os nomes em `--allowed-host` |
| com token, sem `--allowed-host` | qualquer nome |
| com token e `--allowed-host` | nomes de loopback e os nomes em `--allowed-host` |

`--allowed-host` (`SIGNALLAB_ALLOWED_HOSTS`) aceita nomes separados por vírgulas, comparados sem a
porta e sem distinção de maiúsculas e minúsculas:

```bash
signal-lab-server --listen 0.0.0.0:1430 --token-file token.txt --allowed-host lab-pc.example.com,192.0.2.10
```

Qualquer outro `Host` recebe `403` com o erro `auth.host`. Defina-o em um servidor alcançável por
nomes conhecidos, para que uma página de outro site não consiga alcançá-lo por um nome próprio.

## Origem e tipo de conteúdo {#origin}

- Toda requisição que altera algo (qualquer uma exceto `GET` e `HEAD`), e o WebSocket de eventos,
  precisa vir sem `Origin` ou com a do próprio servidor — o mesmo host e a mesma porta do `Host`
  dela. Uma página de outro site, ou uma que envie `Origin: null`, recebe `403` com `auth.origin`.
  Scripts e o `curl` não enviam `Origin` e não são afetados.
- Os comandos só aceitam `Content-Type: application/json` (caso contrário, `415` com
  `command.json_required`), então um formulário de outro site não consegue enviar um.
- O servidor não responde a requisições de outras origens (CORS).

## Cabeçalhos de resposta {#headers}

Toda resposta leva:

| Cabeçalho | Valor |
| --- | --- |
| `Content-Security-Policy` | `default-src 'self'; connect-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; object-src 'none'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'` |
| `X-Content-Type-Options` | `nosniff` |
| `X-Frame-Options` | `DENY` |
| `Referrer-Policy` | `same-origin` |
| `Cache-Control` | `no-store` para a API e a página de entrada |

A interface só carrega o que o servidor serve, só conversa com o servidor e não pode ser embutida
em um frame por outra página.

## Arquivos e tamanhos {#files}

- Um download (`GET /api/files?path=…`) só é servido de dentro da pasta de dados, com no máximo
  256 MiB; qualquer outra coisa é `404`.
- O corpo de uma requisição tem no máximo 24 MiB.

## Segredos {#secrets}

O valor de um segredo nunca sai do motor:

- Em um servidor, os valores são **somente leitura**: a variável de ambiente
  `SIGNALLAB_SECRET_<NAME>`, ou o arquivo `<NAME>` na pasta de segredos (`/run/secrets/signallab`
  por padrão). Definir ou remover um pelo navegador é recusado (`secret.read_only`), então um
  valor digitado em uma página nunca acaba guardado em um lugar menos seguro. Veja
  [Segredos](index.md#secrets).
- Nenhum comando devolve um valor; a interface só fica sabendo se um nome está definido.
- Os experimentos indicam os segredos como `{{secret.NAME}}`. Enquanto uma execução ou um envio os
  usa, todo texto que ele relata — etapas, erros, o relatório da execução — mostra `••••` no lugar
  deles, e os quadros no [[ui:dock.inspector]] são mascarados byte a byte.
- As credenciais de um nó HTTP só viram um cabeçalho `Authorization` no momento em que a
  requisição é enviada; etapas, quadros e relatórios levam a resposta, nunca esse cabeçalho.

## O que é registrado {#audit}

O log registra, com o endereço do cliente: cada tarefa iniciada — tempestades, varreduras,
broadcasts, monitores, geradores, execuções, emuladores —, cada entrada e cada tentativa de entrada
com um token errado. Veja [Logs](index.md#logs).

## O que o servidor nunca faz {#never}

- **Alterar o firewall do host.** O aplicativo de desktop pode acrescentar uma regra de firewall
  quando você pede; em um servidor, esse comando é recusado (`firewall.server`). O firewall do host
  é assunto de quem administra o host. (O script de instalação em um só comando se oferece para
  abrir a porta do servidor no ufw ou no firewalld, e pergunta antes — veja
  [O firewall do host](index.md#firewall).)
- **Iniciar alcançável sem token**, em qualquer endereço que não seja o loopback.
- **Servir um arquivo de fora da sua pasta de dados.**
- **Guardar um segredo** digitado em um navegador.
- **Falar TLS** ele mesmo: coloque um proxy HTTPS na frente dele (veja
  [Atrás de um proxy HTTPS](index.md#https)).

Todos os limites do motor — no máximo 1.024 hosts em uma varredura, no máximo 50.000 pacotes por
segundo a partir de um beacon — valem em um servidor como no aplicativo. São limites de proteção,
não permissão: envie tráfego só para sistemas que são seus ou que você pode testar.

## O contêiner {#container}

A imagem roda como um usuário sem privilégios (uid e gid 10001) e só escreve em `/data`. Ela roda
sem alterações com um sistema de arquivos raiz somente leitura, sem capabilities e com
`no-new-privileges` — como o script de instalação e o `deploy/compose.yaml` a iniciam. Cada imagem
é publicada com um SBOM, a proveniência do build e uma atestação assinada do GitHub:

```bash
gh attestation verify oci://ghcr.io/proanima/signallab:[[version]] -R ProAnima/SignalLab
```

## O que o Signal Lab envia para o mundo exterior {#outside}

Além do tráfego que você envia, o Signal Lab conversa com dois lugares, ambos do estúdio.

### Verificação de atualizações {#update-check}

Só o **aplicativo de desktop** procura atualizações; um servidor e um navegador nunca o fazem. Com
a opção [[ui:update.auto]] ativada (na janela [[ui:about.open]]), uma vez por dia, e sempre que
você pressiona [[ui:update.check]], o aplicativo consulta o hub do estúdio (`hub.proanima.net`) —
e a versão mais recente do GitHub só quando o hub não pode ser alcançado. A consulta leva:

- a versão do aplicativo;
- o sistema operacional e a arquitetura do processador;
- um número aleatório desta instalação (`X-Install-Id`), criado uma vez e guardado com as
  configurações do aplicativo, para que uma versão nova possa chegar primeiro a uma parte das
  instalações. Ele não diz nada sobre você nem sobre o computador.

Só versões publicadas são oferecidas. Um download cuja assinatura não confere com a chave embutida
no aplicativo não é instalado, e nada é instalado até você pressionar [[ui:update.install]].

### Comentários {#feedback}

[[ui:feedback.open]] (o ✉ do cabeçalho, também na janela [[ui:about.open]]) envia uma mensagem aos
desenvolvedores pelo hub do estúdio, que a encaminha por e-mail; o aplicativo não guarda senha
nenhuma para isso. Ele envia só o que o formulário mostra: a sua mensagem, o seu e-mail, se você
informar um, as capturas de tela que você anexar e — em [[ui:feedback.logs]] — o log do console e
[[ui:feedback.systemInfo]], cada um dos quais você pode abrir antes de enviar e desmarcar. O nome
deste computador, o endereço dele e as suas pastas ficam de fora deles. A partir de um navegador,
quem envia o formulário é o servidor.

---
title: Sinais
description: Guarde mensagens OSC, UDP, HTTP e MQTT em uma biblioteca de pastas e envie-as de novo pela tela Sinais, de qualquer tela com Ctrl+K ou pela linha de comando.
---

# Sinais

Um sinal é uma mensagem que você nomeou e guardou: uma mensagem OSC, um datagrama UDP bruto,
uma requisição HTTP ou uma publicação MQTT, com o seu destino. Você o monta uma vez — na tela
[[ui:nav.signals]], ou salvando o que acabou de enviar em outra tela — e o envia de novo
sempre que precisar, byte a byte igual.

A biblioteca é um arquivo JSON que você pode ler, editar à mão, copiar para outra máquina ou
versionar junto de um projeto. A tela [[ui:nav.signals]] a mostra como uma árvore de pastas à
esquerda e os campos do sinal selecionado à direita.

## O que um sinal pode enviar {#transports}

Escolha o tipo em [[ui:sig.transport]]. Cada tipo tem os seus próprios campos:

| [[ui:sig.transport]] | Campos | O que sai |
| --- | --- | --- |
| [[ui:sig.tr.osc]] | [[ui:common.target]], [[ui:common.address]], [[ui:common.arguments]] | Uma mensagem OSC para um `IP:port` ou `host:port`, a partir de uma porta UDP nova. Veja [OSC](../protocols/osc.md#types) para os tipos de argumento. |
| [[ui:sig.tr.udp]] | [[ui:common.target]], [[ui:sig.payloadKind]] ([[ui:sig.payloadText]] ou [[ui:sig.payloadHex]]), [[ui:sig.payload]] | Um datagrama com exatamente esses bytes. O texto vai como escrito, sem terminador; hex são pares de dígitos como `de ad be ef` (espaços e um prefixo `0x` são permitidos). |
| [[ui:sig.tr.http]] | [[ui:sig.method]], [[ui:sig.url]], [[ui:sig.timeout]], [[ui:sig.headers]], [[ui:field.auth]], [[ui:sig.body]] | Uma requisição HTTP. Veja [HTTP](../protocols/http.md). |
| [[ui:sig.tr.mqtt]] | [[ui:mq.broker]], [[ui:mq.topic]], [[ui:mq.qos]], [[ui:mq.retain]], [[ui:sig.payload]] | Uma publicação. Veja [MQTT](../protocols/mqtt.md). |

Todo sinal também tem um [[ui:sig.name]], uma [[ui:sig.group]] e uma [[ui:sig.note]] — o que
ele deve provocar e o que precisa corresponder do outro lado.

Os destinos dos sinais OSC e UDP são `IP:port` ou `host:port` (por exemplo
`127.0.0.1:9000`); um nome de host é resolvido a cada envio do sinal. O broker de um sinal
MQTT é `host:port`; sem porta, é `1883`.

Quando você muda o tipo de um sinal, a mensagem dele recomeça dos padrões desse tipo. Só o
destino é mantido, e apenas entre [[ui:sig.tr.osc]] e [[ui:sig.tr.udp]], onde o destino
significa a mesma coisa.

## Enviar um sinal {#send}

Para enviar um sinal pela tela [[ui:nav.signals]], faça qualquer uma destas coisas:

- Selecione-o e pressione [[ui:sig.fire]].
- Pressione <kbd>Ctrl</kbd>+<kbd>Enter</kbd> enquanto trabalha nos campos dele.
- Dê um clique duplo nele na árvore.

Cada envio escreve uma linha no console: o que foi para onde, os bytes enviados ou, no HTTP,
o status e o tempo que levou. Uma falha (uma conexão recusada, um host inalcançável) é uma
linha vermelha com o motivo. A hora do último envio aparece ao lado dos botões.

Um sinal sai pelos mesmos comandos das telas do seu protocolo, então o
[Inspetor](inspector.md) o lista sob a ferramenta que o enviou, e a outra ponta não o
distingue de um que você digitou.

Como cada tipo é enviado:

| Tipo | Como sai |
| --- | --- |
| [[ui:sig.tr.osc]] | Como a tela [[ui:nav.osc]] envia uma mensagem. |
| [[ui:sig.tr.udp]] | Um datagrama para o destino. |
| [[ui:sig.tr.http]] | Como a tela [[ui:nav.http]] envia uma requisição, com o armazenamento de cookies dela enquanto [[ui:http.keepCookies]] estiver ligado ali. Uma conexão recusada ou um tempo limite conta como falha, não como status. |
| [[ui:sig.tr.mqtt]] | Enquanto a tela [[ui:nav.mqtt]] estiver conectada ao broker do sinal, por essa conexão, com o id de cliente e as credenciais dela. Caso contrário — sem conexão, ou conectada a outro broker — o Signal Lab se conecta ao broker do sinal para essa única publicação, com um id de cliente próprio, sem nome de usuário e com sessão limpa, e depois desconecta. |

Uma conexão é com o broker do sinal quando o host é o mesmo, ignorando maiúsculas, e a porta
é a mesma, sendo `1883` a de um broker escrito sem porta. Os nomes não são resolvidos:
`localhost` e `127.0.0.1` são dois brokers diferentes aqui, então um sinal que nomeia um não é
enviado por uma conexão feita com o outro.

::: tip
Um sinal MQTT não guarda senha. Para publicar em um broker que a peça, conecte-se primeiro a
esse broker na tela [[ui:nav.mqtt]]; o sinal então segue por essa conexão.
:::

## Enviar de qualquer tela {#palette}

Pressione <kbd>Ctrl</kbd>+<kbd>K</kbd> em qualquer tela para abrir a paleta, digite algumas
letras do nome, da pasta, do destino ou da mensagem de um sinal e pressione
<kbd>Enter</kbd>. A paleta fecha e o sinal é enviado; você continua na tela que estava
observando.

| Tecla | O que faz |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>K</kbd> | Abre a paleta, ou a fecha. |
| <kbd>↑</kbd> <kbd>↓</kbd> | Move a escolha. |
| <kbd>Enter</kbd> | Envia o sinal escolhido. |
| <kbd>Esc</kbd> | Fecha a paleta sem enviar. |

A paleta lista no máximo 12 sinais: os 12 primeiros da biblioteca enquanto você não digitou
nada, depois os 12 primeiros que correspondem. Um clique em uma linha a envia; um clique fora
fecha a paleta.

## Criar sinais {#create}

### Na tela Sinais {#create-here}

1. Escolha a pasta a que o sinal pertence (veja [pasta atual](#current-folder)).
2. Pressione [[ui:sig.new]]. Um novo sinal OSC para `127.0.0.1:9000`, endereço `/hello`,
   aparece nessa pasta, selecionado.
3. Altere [[ui:sig.name]], [[ui:sig.transport]] e os campos da mensagem.

Cada alteração é salva por conta própria; não há botão de salvar nesta tela. Enquanto o
arquivo da biblioteca não puder ser lido, nada é salvo e os campos ficam somente leitura
(veja [O arquivo da biblioteca](#file)).

[[ui:sig.duplicate]] põe uma cópia logo depois do sinal selecionado, com o nome seguido de
`·`. [[ui:sig.delete]] pergunta mais uma vez ([[ui:sig.confirmDelete]]): o segundo clique o
remove do arquivo. A pasta dele permanece, mesmo que agora esteja vazia.

### Nas telas HTTP, OSC e MQTT {#save-from-screens}

A parte de envio de três telas — [[ui:http.request]] em [[ui:nav.http]], [[ui:osc.sender]]
em [[ui:nav.osc]], [[ui:mq.publish]] em [[ui:nav.mqtt]] — pode guardar como sinal o que
enviaria.

1. Monte a mensagem e envie-a até que faça o que você quer.
2. Pressione [[ui:sig.saveNew]] (ou <kbd>Ctrl</kbd>+<kbd>S</kbd> nessa parte da tela). A
   janela [[ui:sig.saveTitle]] abre.
3. Confira [[ui:sig.name]]: ele é sugerido a partir do que está sendo enviado.
4. Escolha ou digite uma [[ui:sig.group]]. A pasta que você usou da última vez já vem
   preenchida; um caminho como `Venue/Stage` que ainda não existe é criado.
5. Pressione [[ui:sig.saveConfirm]].

A partir daí a tela fica ligada a esse sinal. Um chip ao lado dos botões diz onde ele mora
(`❖ Folder / Name`); clique nele para ver o sinal na tela [[ui:nav.signals]].

| Você vê | Significa | O que você pode fazer |
| --- | --- | --- |
| ✓ [[ui:sig.savedState]] (esmaecido) | A biblioteca contém exatamente o que a tela enviaria. | Nada a salvar. |
| [[ui:sig.save]], e [[ui:sig.changed]] no chip | A mensagem da tela difere do sinal. | [[ui:sig.save]] ou <kbd>Ctrl</kbd>+<kbd>S</kbd> grava a mensagem da tela nesse sinal; o nome, a pasta e a nota permanecem. |
| [[ui:sig.saveAs]] | — | Abre a janela de novo, preenchida com o nome e a pasta do sinal, e salva um sinal novo. A tela passa então a ficar ligada ao novo. |

A comparação olha para a mensagem, não para como ela é escrita: a ordem das chaves do JSON e
os últimos dígitos de um float OSC além da precisão de 32 bits não contam como alteração.

As telas [[ui:nav.http]] e [[ui:nav.osc]] mantêm a ligação quando o Signal Lab reinicia; a
tela [[ui:nav.mqtt]] a mantém até você fechar o aplicativo.

::: warning
Um sinal HTTP guarda a sua [[ui:field.auth]] — nome de usuário e senha, ou token — no arquivo
da biblioteca em texto simples. Quem puder ler o arquivo pode ler esses dados.
:::

### Abrir um sinal na sua tela {#open-in-screen}

Um sinal HTTP, OSC ou MQTT selecionado tem um botão que o abre na tela do seu protocolo
([[ui:nav.http]], [[ui:nav.osc]] ou [[ui:nav.mqtt]]). Os campos da tela são preenchidos a
partir do sinal e a tela fica ligada a ele, como acima: edite ali, envie e depois use
[[ui:sig.save]]. Um sinal UDP bruto não tem tela própria.

### A partir de um quadro ou de um tópico {#capture}

- No [Inspetor](inspector.md#save-as-signal), selecione um quadro e pressione
  [[ui:sig.fromFrame]]. Um datagrama vira um sinal UDP bruto com os bytes exatos desse
  quadro; uma publicação MQTT vira um sinal MQTT com o mesmo broker, tópico, QoS, indicador
  retain e carga útil. Outros quadros não podem ser salvos.
- Na tela [[ui:nav.mqtt]], selecione um tópico e pressione [[ui:sig.fromFrame]]. Você obtém
  um sinal MQTT que publica o último valor do tópico, com o seu QoS e indicador retain, no
  broker a que você está conectado.

Os dois vão para a pasta [[ui:sig.capturedFolder]] e recebem o nome do que foi capturado.

### Em um experimento {#in-experiments}

Quando você acrescenta um nó a um experimento, o menu [[ui:exp.addNode]] também lista os seus
sinais em [[ui:exp.group.signals]]. Escolher um acrescenta um nó OSC, HTTP, MQTT ou UDP com a
mesma mensagem. Um sinal UDP bruto com carga útil em hex não é oferecido: o nó UDP envia
texto. Veja [Nós](../experiments/nodes.md).

## Pastas {#folders}

Uma pasta é um caminho de nomes unidos por `/`: `API/Auth` é a pasta `Auth` dentro de `API`.
O campo [[ui:sig.group]] de um sinal guarda o caminho da pasta dele; vazio significa o nível
superior ([[ui:sig.topLevel]]). Os nomes são aparados e as partes vazias descartadas quando
você sai do campo, então ` API / Auth/ ` vira `API/Auth`.

As pastas são ordenadas por nome, com números em ordem numérica (`Cue 2` antes de `Cue 10`);
os sinais ficam na ordem do arquivo. Cada pasta mostra quantos sinais contém, subpastas
incluídas. Uma pasta vazia é mantida até você removê-la.

### A pasta atual {#current-folder}

A pasta em que você clicou por último, ou a pasta do sinal que você selecionou, é a atual:
[[ui:sig.new]] e [[ui:sig.newFolder]] põem as coisas nela. Um chip acima da árvore a nomeia;
clique no chip para voltar ao nível superior.

### Trabalhar com pastas {#folder-tasks}

| Para | Faça isto |
| --- | --- |
| Criar uma pasta | Pressione ＋ [[ui:sig.newFolder]]. Ela é criada dentro da pasta atual, com o nome [[ui:sig.newFolderName]] (com um número depois quando esse nome já existe), e você a renomeia na hora. |
| Abrir ou fechar uma pasta | Clique nela, ou pressione <kbd>→</kbd> / <kbd>←</kbd> enquanto ela tem o foco. O Signal Lab lembra quais pastas estão fechadas. |
| Abrir ou fechar todas | Os botões ⊞ e ⊟ acima da árvore ([[ui:sig.expandAll]], [[ui:sig.collapseAll]]). |
| Renomear uma pasta | Pressione ✎ ([[ui:sig.renameFolder]]) ou <kbd>F2</kbd> nela, digite e depois <kbd>Enter</kbd>; <kbd>Esc</kbd> cancela. |
| Mover um sinal ou uma pasta | Arraste-o para uma pasta, ou para um espaço vazio da árvore para o nível superior. |
| Mover um sinal digitando | Altere o campo [[ui:sig.group]] dele. |
| Remover uma pasta | Pressione × ([[ui:sig.removeFolder]]) e depois [[ui:sig.confirmRemoveFolder]], ou pressione <kbd>Delete</kbd> duas vezes nela. |

Renomear nunca une duas pastas: um nome com `/` ou um que uma pasta irmã já tem é recusado, e
o console avisa. Uma pasta não pode ser arrastada para dentro de si mesma nem para uma pasta
dentro dela. Arrastar uma pasta para uma pasta que já tem uma com o mesmo nome une as duas.

Remover uma pasta remove só a pasta: os sinais e as subpastas dela sobem um nível. Nada é
excluído.

### Encontrar um sinal {#filter}

Digite em [[ui:sig.search]], acima da árvore. O filtro compara o nome, a pasta, a nota, o
destino e a mensagem. Enquanto você filtra, toda pasta com uma correspondência fica aberta e
as outras ficam ocultas.

## O conjunto inicial {#starter-set}

Na primeira vez que o Signal Lab não encontra um arquivo de biblioteca, ele grava nove
exemplos, cada um sobre algo fácil de errar. Os nomes e as notas deles são escritos no
idioma da interface naquele momento; depois disso são seus para alterar. Todos apontam para
este computador.

| Pasta | Sinal | Envia |
| --- | --- | --- |
| `OSC` | [[ui:seed.osc-fader.name]] | `/fader/1` com o float `0.75` para `127.0.0.1:9000` |
| `OSC` | [[ui:seed.osc-types.name]] | `/types` com int `-7`, float `1.5`, string `hi`, bool true, int64 `4294967296`, double `0.125` e nil |
| `OSC` | [[ui:seed.osc-id-and-value.name]] | `/tag` com as strings `reader-1` e `04a1b2c3` |
| `OSC` | [[ui:seed.osc-trigger.name]] | `/cue/go` sem argumentos |
| `MQTT` | [[ui:seed.mqtt-publish.name]] | `1` em `lab/example/value` em `127.0.0.1:1883`, QoS 0 |
| `MQTT` | [[ui:seed.mqtt-retained.name]] | `night` em `lab/example/config`, QoS 1, retido |
| `MQTT` | [[ui:seed.mqtt-clear-retained.name]] | Uma carga útil retida vazia em `lab/example/config`, QoS 1 |
| `HTTP` | [[ui:seed.http-reachable.name]] | `GET http://127.0.0.1:8080/`, tempo limite de 4000 ms |
| [[ui:seed.folder.Raw]] | [[ui:seed.udp-raw.name]] | Os bytes `de ad be ef` para `127.0.0.1:9000` |

Para recuperar o conjunto inicial, mova ou renomeie `signals.json` e pressione
[[ui:sig.reload]]: sem arquivo ali, ele é gravado de novo.

## O arquivo da biblioteca {#file}

A biblioteca é o `signals.json` na pasta de dados: `Documents/SignalLab` na sua pasta pessoal
em um desktop, ou a pasta de dados do servidor (veja [Arquivos](../reference/files.md)). Passe
o mouse sobre a contagem de sinais sob a árvore para ver o caminho completo.

- **Salvo por conta própria.** Cada alteração é gravada 0,7 s depois da última, o arquivo
  inteiro de uma vez, por um arquivo temporário na mesma pasta que depois toma o lugar do
  arquivo — uma gravação interrompida deixa o arquivo anterior. Enquanto uma gravação espera,
  o pé da árvore diz [[ui:sig.saving]]; depois, [[ui:sig.saved]]. Uma gravação ainda pendente
  é feita antes de uma atualização reiniciar o aplicativo.
- **Editado à mão.** O Signal Lab não percebe quando o arquivo muda por baixo dele. Depois de
  editá-lo, ou de substituí-lo por um de outra máquina, pressione [[ui:sig.reload]]. Recarregar
  lê o arquivo de novo e descarta uma alteração que ainda esperava para ser gravada.
- **Alterado em outro lugar.** Cada salvamento diz qual arquivo leu. Se o arquivo mudou desde
  então — você o editou, ou outra página do mesmo servidor salvou uma alteração própria — o
  salvamento é recusado, o arquivo mantém essa alteração, e a árvore mostra o erro e para de
  gravar, como para um arquivo quebrado, abaixo. Pressione [[ui:sig.reload]] para ver o que há
  ali agora; a alteração que esta página não conseguiu salvar some com o recarregamento, então
  refaça-a.
- **Nunca substituído enquanto está quebrado.** Se o arquivo não é JSON válido, ou não é uma
  biblioteca de sinais, a árvore mostra o erro com o caminho, a linha e a coluna do arquivo, e
  o console diz o mesmo. O arquivo é deixado como está, e nada grava a biblioteca até que ele
  seja lido de novo: [[ui:sig.new]], renomear, mover e remover sinais e pastas, arrastar, os
  campos de um sinal, [[ui:sig.saveNew]], [[ui:sig.save]] e [[ui:sig.saveAs]] nas telas HTTP,
  OSC e MQTT, e [[ui:sig.fromFrame]] no Inspetor e na tela MQTT ficam todos desativados, e a
  dica deles diz o que está errado. Corrija o arquivo, ou remova-o, e pressione
  [[ui:sig.reload]]: assim que ele for lido, tudo volta a funcionar.
- **Um arquivo que quebra enquanto o aplicativo roda.** Se você edita o arquivo até deixá-lo
  ilegível e o aplicativo então salva uma alteração, o salvamento é recusado com o mesmo erro,
  o arquivo é deixado como você o fez, e o aplicativo para de gravar até você corrigi-lo e
  pressionar [[ui:sig.reload]]. Um arquivo que você editou e deixou válido também não é
  substituído: o próximo salvamento é recusado como alterado em outro lugar, e
  [[ui:sig.reload]] lê a sua edição.

Um pequeno exemplo do arquivo:

```json
{
  "version": 2,
  "signals": [
    {
      "id": "fader-value",
      "name": "Fader value",
      "group": "Venue/Stage",
      "note": "Main fader of desk A.",
      "body": {
        "transport": "osc",
        "target": "127.0.0.1:9000",
        "address": "/fader/1",
        "args": [{ "type": "float", "value": 0.75 }]
      }
    }
  ],
  "folders": ["Venue/Stage", "Venue/Empty for now"]
}
```

| Chave | O que é |
| --- | --- |
| `version` | `2`. Um arquivo versão 1 (anterior às pastas) é lido igual, sem pastas vazias. |
| `signals[].id` | Formado a partir do nome quando o sinal é criado (`fader-value`, `fader-value-2`, …) e nunca alterado por uma renomeação. `signallab fire` encontra um sinal por ele. |
| `signals[].group` | O caminho da pasta; `""` é o nível superior. |
| `signals[].body` | A mensagem. `transport` é `osc`, `udp`, `http` ou `mqtt`; as outras chaves são os campos desse tipo. |
| `folders` | Todas as pastas, para que uma vazia seja mantida. Omitido quando não há nenhuma. Um `group` que nenhuma entrada lista também é uma pasta. |

## Pela linha de comando {#cli}

`signallab fire` envia um sinal de uma biblioteca, pelos mesmos comandos do aplicativo:

```bash
signallab fire "Fader value"
signallab fire fader-value --library ./show/signals.json
```

Ele encontra o sinal primeiro pelo id, depois pelo nome, ignorando maiúsculas. Quando vários
sinais têm esse nome, ele nomeia os ids e não envia nada. Sem `--library`, lê o próprio
`signals.json` do aplicativo; nunca grava o arquivo. Veja
[A linha de comando](../automation/cli.md#cli-fire).

## Relacionados {#related}

- [Inspetor](inspector.md) — observe o que um sinal envia e guarde um quadro capturado como
  sinal.
- [Atalhos de teclado](../reference/shortcuts.md)
- [Arquivos](../reference/files.md) — onde fica a pasta de dados.

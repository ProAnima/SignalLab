---
title: Instalação e atualização
description: Instale o aplicativo de desktop Signal Lab no Windows ou no Linux, confira um download, mantenha-o atualizado e remova-o de novo.
---

# Instalação e atualização

O Signal Lab é publicado no GitHub: cada versão traz o aplicativo de desktop para Windows e Linux,
a linha de comando `signallab` avulsa e uma lista de somas de verificação. Para executá-lo em um
servidor e usá-lo pelo navegador, veja [O servidor](../server/index.md).

## O que baixar {#downloads}

Abra a [versão mais recente](https://github.com/ProAnima/SignalLab/releases/latest) e escolha o
arquivo do seu sistema. `<version>` é a versão publicada, como `[[version]]`.

| Arquivo | Para |
| --- | --- |
| `Signal.Lab_<version>_x64-setup.exe` | Windows 10 e 11, x64 — o instalador que a maioria das pessoas quer |
| `Signal.Lab_<version>_x64_en-US.msi` | Windows, implantação pela TI (Intune, Política de Grupo): instala para todos os usuários |
| `Signal.Lab_<version>_amd64.deb` | Linux x86_64: Debian, Ubuntu e derivados |
| `Signal.Lab-<version>-1.x86_64.rpm` | Linux x86_64: Fedora, RHEL, openSUSE e derivados |
| `Signal.Lab_<version>_amd64.AppImage` | Linux x86_64, qualquer distribuição, sem instalar |
| `signallab-<version>-windows-x64.zip` | só a linha de comando, para Windows |
| `signallab-<version>-linux-x64.tar.gz` | só a linha de comando, para Linux |
| `SHA256SUMS.txt` | a soma de verificação SHA-256 de cada arquivo acima |

Os arquivos `.sig` e o `latest.json` ao lado deles são para o atualizador do próprio aplicativo;
você não precisa deles.

## Windows {#windows}

O Signal Lab roda no Windows 10 e 11, x64. Ele usa o runtime WebView2, que faz parte de ambos.

### Instalação com o instalador {#windows-setup}

1. Execute `Signal.Lab_<version>_x64-setup.exe`.
2. Se o Windows SmartScreen disser que protegeu o seu computador, escolha **Mais informações** e
   depois **Executar assim mesmo** (veja [SmartScreen](#smartscreen) abaixo).
3. Escolha o idioma do instalador. Ele oferece os mesmos 11 idiomas do aplicativo.
4. Escolha para quem é a instalação:
   - Só para você: instala em `%LOCALAPPDATA%\Signal Lab` e não precisa de direitos de
     administrador.
   - Para todos que usam este computador: instala em `C:\Program Files\Signal Lab` e pede
     direitos de administrador.
5. Confirme a pasta e instale.
6. A última página oferece iniciar o Signal Lab imediatamente e criar um atalho na área de
   trabalho.

Executar o instalador de uma versão mais nova sobre uma já instalada atualiza a instalação no
lugar.

### O que o instalador acrescenta {#windows-setup-adds}

Além do aplicativo, o instalador coloca duas coisas no lugar, e o desinstalador remove as duas:

- **A linha de comando.** O `signallab.exe` vai para junto do aplicativo, e a pasta dele, para o
  `PATH` — o seu `PATH`, em uma instalação só para você, o do computador, em uma instalação para
  todos —, de modo que `signallab` funciona em todo terminal que você abrir depois. Veja
  [A linha de comando](../automation/cli.md).
- **Uma regra de firewall**, só na instalação para todos. O Firewall do Windows Defender só deixa
  outras máquinas alcançarem um programa quando uma regra permite, e pergunta a quem está diante
  da tela na primeira vez que o programa escuta; um *Cancelar* ali descarta em silêncio tudo o que
  outras máquinas enviarem. Uma instalação para todos, que tem direitos de administrador,
  acrescenta uma regra de entrada que permite o `signal-lab.exe` (o aplicativo) e outra para o
  `signallab.exe` (a linha de comando), em redes privadas e de domínio. Uma instalação só para
  você não pode alterar o firewall: o aplicativo se oferece para fazê-lo, com o próprio aviso de
  administrador do Windows, na primeira vez que algo escuta — veja
  [o aviso do firewall](interface.md#firewall-notice).

O tráfego dentro deste computador (`127.0.0.1`) nunca é filtrado, então tudo o que você faz em
loopback funciona sem regra nenhuma.

### Instalações autônomas {#windows-unattended}

O instalador aceita estas opções na linha de comando:

| Opção | O que faz |
| --- | --- |
| `/S` | Instala em silêncio, sem perguntar nada. |
| `/P` | Instala com uma barra de progresso e sem perguntas. |
| `/ALLUSERS` | Instala para todos (exige um prompt elevado). |
| `/CURRENTUSER` | Instala só para o usuário atual. |
| `/NS` | Não cria atalhos. |
| `/D=C:\Tools\Signal Lab` | Instala nesta pasta. Precisa vir por último e não leva aspas. |
| `/NOPATH` | Não mexe no `PATH`: o `signallab` é instalado, mas não fica no `PATH`. |
| `/NOFIREWALL` | Não acrescenta regras de firewall. |

Por exemplo, uma instalação silenciosa para todos, sem regras de firewall, a partir de um prompt
elevado:

```powershell
.\Signal.Lab_[[version]]_x64-setup.exe /S /ALLUSERS /NOFIREWALL
```

### O MSI {#windows-msi}

O `Signal.Lab_<version>_x64_en-US.msi` é para implantação pela TI. Ele instala para todos os
usuários e coloca o `signallab.exe` junto do aplicativo e essa pasta no `PATH` do computador, mas
não acrescenta regra de firewall:
isso fica a cargo de quem o implanta (pela Política de Grupo, por exemplo). O aplicativo ainda
oferece a regra na primeira vez que escuta. Para instalá-lo em silêncio:

```powershell
msiexec /i "Signal.Lab_[[version]]_x64_en-US.msi" /qn
```

### SmartScreen {#smartscreen}

Os instaladores ainda não têm assinatura de código, então o Windows SmartScreen não conhece o
editor deles e pode interromper a instalação com um aviso. Escolha **Mais informações**, confira se
o arquivo é o que você baixou da página de versões e depois escolha **Executar assim mesmo**. Para
ter certeza de que o arquivo não foi alterado, [confira-o com o `SHA256SUMS.txt`](#checksums)
antes.

As atualizações do próprio aplicativo são assinadas e verificadas à parte; veja
[Atualizações](#updates).

## Conferir um download {#checksums}

O `SHA256SUMS.txt` lista a soma de verificação SHA-256 de cada arquivo da versão, uma por linha,
seguida do nome do arquivo. Baixe-o ao lado do arquivo e compare.

No Windows, no PowerShell:

```powershell
Get-FileHash .\Signal.Lab_[[version]]_x64-setup.exe -Algorithm SHA256
Select-String "Signal.Lab_[[version]]_x64-setup.exe" .\SHA256SUMS.txt
```

Os dois hashes precisam ser iguais (o `Get-FileHash` o imprime em maiúsculas; maiúsculas e
minúsculas não importam).

No Linux, na pasta que contém os dois arquivos:

```bash
sha256sum --check --ignore-missing SHA256SUMS.txt
```

Ele imprime `OK` depois de cada arquivo que encontrou. Qualquer outra coisa significa que o
download não é o arquivo publicado: baixe-o de novo.

## Linux {#linux}

O aplicativo para Linux é compilado para x86_64 (no Ubuntu 22.04) e precisa do WebKitGTK 4.1, a
web view com que ele desenha a janela. Não há build para processadores Arm; em uma máquina Arm,
execute o [servidor](../server/index.md).

### O pacote .deb {#linux-deb}

```bash
sudo apt install ./Signal.Lab_[[version]]_amd64.deb
```

O `apt` instala junto o que o pacote precisa. O pacote também coloca a linha de comando em
`/usr/bin/signallab`.

### O pacote .rpm {#linux-rpm}

```bash
sudo dnf install ./Signal.Lab-[[version]]-1.x86_64.rpm
```

No openSUSE, use `sudo zypper install` com o mesmo arquivo. Como o `.deb`, o pacote coloca a linha
de comando em `/usr/bin/signallab`.

### O AppImage {#linux-appimage}

O AppImage roda sem instalar:

```bash
chmod +x Signal.Lab_[[version]]_amd64.AppImage
./Signal.Lab_[[version]]_amd64.AppImage
```

Ele não inclui a linha de comando `signallab`; pegue-a no
[arquivo da linha de comando](#cli-archives) se precisar dela.

### O firewall no Linux {#linux-firewall}

No Linux, o aplicativo não mostra aviso de firewall: o `ufw` e o `firewalld` funcionam por porta,
não por programa. Se um deles estiver ativo e outras máquinas precisarem alcançar um monitor, um
ouvinte, um emulador ou um retransmissor, abra a porta dele ali, por exemplo:

```bash
sudo ufw allow 9000/udp
```

O tráfego de loopback não é afetado.

## A linha de comando avulsa {#cli-archives}

Os instaladores de desktop trazem o `signallab` com eles. Para uma máquina sem o aplicativo — um
agente de build, um servidor, um PC do laboratório —, cada versão também traz a linha de comando
avulsa:

- `signallab-<version>-windows-x64.zip` contém o `signallab.exe` e a licença.
- `signallab-<version>-linux-x64.tar.gz` contém o `signallab` e a licença.

Descompacte-o em uma pasta que esteja no seu `PATH`. No Linux:

```bash
tar -xzf signallab-[[version]]-linux-x64.tar.gz signallab
sudo install signallab /usr/local/bin/
signallab version
```

A imagem Docker do servidor também a contém. Como usá-la: [A linha de comando](../automation/cli.md).

## Atualizações {#updates}

### O aplicativo de desktop {#updates-desktop}

O aplicativo de desktop se atualiza apenas a partir de versões publicadas, e apenas quando você
mandar.

- **Ele verifica uma vez por dia.** Com a opção [[ui:update.auto]] marcada (ela vem marcada) o
  aplicativo procura uma versão mais nova logo depois de iniciar, quando a última verificação foi
  há um dia ou mais, e de novo sempre que mais um dia passa com ele aberto. Desmarque-a na janela Sobre
  para que ele pare de verificar por conta própria.
- **Você pode verificar agora.** Abra a janela Sobre — o **?** no cabeçalho — e pressione
  [[ui:update.check]] em [[ui:update.title]].
- **O que a verificação envia.** Ela pergunta ao serviço de atualizações do estúdio
  (`hub.proanima.net`) qual versão esta instalação deve receber, e só recorre à versão mais recente
  do GitHub quando esse serviço não pode ser alcançado. A requisição leva a versão do aplicativo, o
  sistema e o tipo de processador, e um número aleatório criado para esta instalação, o que permite
  que uma versão nova chegue primeiro a uma parte das instalações. Nada que identifique você ou
  este computador é enviado.
- **Quando uma versão é encontrada**, o console avisa e o cabeçalho mostra um botão de atualização
  que abre a janela Sobre. Ali, [[ui:update.notes]] mostra as notas da versão.
- **Nada é instalado até você clicar.** Pressione [[ui:update.install]] (quando há tarefas em
  execução, o botão avisa que vai pará-las). O Signal Lab salva tudo o que ainda estiver pendente,
  para todas as tarefas em execução, baixa a atualização e verifica a assinatura dela com a chave
  embutida no aplicativo — uma atualização que não foi assinada pelo processo de publicação é
  recusada —, depois a instala e reinicia já na versão nova.

No Windows, a atualização executa o instalador sem perguntas e mantém a sua pasta, o `PATH` e as
regras de firewall. No Linux, ela substitui o AppImage ou instala o novo pacote `.deb` ou `.rpm` —
no caso de um pacote, o sistema pede antes a sua senha.

Versões prévias e rascunhos nunca são oferecidos. Para impedir que todas as instalações de uma
máquina verifiquem por conta própria — quando a TI implanta as atualizações ela mesma, por exemplo
—, defina a variável de ambiente `SIGNALLAB_NO_UPDATE_CHECK` com qualquer valor;
[[ui:update.check]] continua funcionando quando pressionado. Builds de desenvolvimento nunca
verificam por conta própria.

### Um servidor {#updates-server}

Um servidor, e a página que um navegador mostra a partir dele, se atualiza com a sua imagem
Docker, nunca por conta própria: a janela Sobre informa isso. Como atualizar um servidor:
[O servidor](../server/index.md).

## Desinstalação {#uninstall}

**Windows.** Abra *Configurações → Aplicativos → Aplicativos instalados*, encontre o Signal Lab e
escolha *Desinstalar*. O desinstalador tira o `signallab.exe` do `PATH` e, quando é executado com
direitos de administrador (como acontece em uma instalação para todos), remove todas as regras de
entrada do firewall para o aplicativo e a linha de comando, inclusive as que o aviso do Windows
criou. Ele oferece uma caixa para excluir também os dados do aplicativo: são as configurações do
próprio aplicativo — o idioma, os tamanhos dos painéis, a tela em que você estava, os últimos
valores digitados nas telas —, não os seus arquivos. Uma instalação MSI é removida do mesmo jeito;
ela leva a sua entrada no `PATH` junto e deixa o firewall a cargo de quem a implantou.

**Linux.** Remova o pacote com o gerenciador de pacotes com que você o instalou, ou exclua o
arquivo AppImage.

Nenhum dos dois mexe na sua **pasta de dados**: experimentos, as bibliotecas de sinais e de
emuladores, relatórios de execução e exportações ficam em `Documents/SignalLab`, na sua pasta
pessoal. Exclua essa pasta você mesmo se quiser apagá-los.

## Onde ficam os seus dados {#data}

Tudo o que você cria é guardado como arquivos simples em uma única pasta, `Documents/SignalLab`
na sua pasta pessoal, tanto no Windows quanto no Linux: o experimento atual, a biblioteca de
sinais, a biblioteca de emuladores, os relatórios de execução, as exportações e as capturas do
Inspetor. Cada arquivo e seu formato estão descritos em [Arquivos e pastas](../reference/files.md).

## Executar como servidor {#server}

Para usar o Signal Lab pelo navegador — em um PC do laboratório ao lado dos equipamentos, em um
computador Linux, no Docker —, veja [O servidor](../server/index.md). Em uma máquina Linux com
Docker, um único comando o instala e inicia.

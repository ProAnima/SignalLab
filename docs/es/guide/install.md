---
title: Instalación y actualizaciones
description: Instala la aplicación de escritorio Signal Lab en Windows o Linux, comprueba una descarga, mantenla al día y vuelve a quitarla.
---

# Instalación y actualizaciones

Signal Lab se publica en GitHub: cada versión incluye la aplicación de escritorio para Windows y
Linux, la línea de comandos `signallab` por separado y una lista de sumas de comprobación. Para
ejecutarlo en un servidor y usarlo desde un navegador, consulta [El servidor](../server/index.md).

## Qué descargar {#downloads}

Abre la [última versión](https://github.com/ProAnima/SignalLab/releases/latest) y elige el
archivo para tu sistema. `<version>` es el número de la versión, por ejemplo `[[version]]`.

| Archivo | Para |
| --- | --- |
| `Signal.Lab_<version>_x64-setup.exe` | Windows 10 y 11, x64: el instalador que quiere la mayoría |
| `Signal.Lab_<version>_x64_en-US.msi` | Windows, despliegue por parte de TI (Intune, directiva de grupo): instala para todos los usuarios |
| `Signal.Lab_<version>_amd64.deb` | Linux x86_64: Debian, Ubuntu y sus derivadas |
| `Signal.Lab-<version>-1.x86_64.rpm` | Linux x86_64: Fedora, RHEL, openSUSE y sus derivadas |
| `Signal.Lab_<version>_amd64.AppImage` | Linux x86_64, cualquier distribución, sin instalar |
| `signallab-<version>-windows-x64.zip` | solo la línea de comandos, para Windows |
| `signallab-<version>-linux-x64.tar.gz` | solo la línea de comandos, para Linux |
| `SHA256SUMS.txt` | la suma de comprobación SHA-256 de cada archivo anterior |

Los archivos `.sig` y `latest.json` que los acompañan son para el actualizador de la propia
aplicación; no los necesitas.

## Windows {#windows}

Signal Lab funciona en Windows 10 y 11, x64. Usa el entorno de ejecución WebView2, que forma parte
de ambos.

### Instalar con el instalador {#windows-setup}

1. Ejecuta `Signal.Lab_<version>_x64-setup.exe`.
2. Si Windows SmartScreen dice que ha protegido tu equipo, elige **Más información** y luego
   **Ejecutar de todas formas** (consulta [SmartScreen](#smartscreen) más abajo).
3. Elige el idioma del instalador. Ofrece los mismos 11 idiomas que la aplicación.
4. Elige para quién es:
   - Solo para ti: se instala en `%LOCALAPPDATA%\Signal Lab` y no necesita derechos de
     administrador.
   - Para todos los que usan este equipo: se instala en `C:\Program Files\Signal Lab` y pide
     derechos de administrador.
5. Confirma la carpeta e instala.
6. La última página ofrece iniciar Signal Lab enseguida y crear un acceso directo en el escritorio.

Ejecutar el instalador de una versión más reciente sobre una ya instalada la actualiza en el mismo
lugar.

### Qué añade el instalador {#windows-setup-adds}

Además de la aplicación, el instalador coloca dos cosas, y el desinstalador quita ambas:

- **La línea de comandos.** `signallab.exe` se coloca junto a la aplicación y su carpeta se añade
  al `PATH` (tu propio `PATH` en una instalación solo para ti, el del equipo en una instalación
  para todos), así que `signallab` funciona en cada terminal que abras después. Consulta
  [La línea de comandos](../automation/cli.md).
- **Una regla de firewall**, solo en una instalación para todos. El Firewall de Windows Defender
  deja que otros equipos lleguen a un programa solo cuando una regla lo permite, y pregunta a la
  persona que está frente a la pantalla la primera vez que el programa escucha; un *Cancelar* ahí
  descarta en silencio todo lo que envíen otros equipos. Una instalación para todos, que tiene
  derechos de administrador, añade una regla de entrada de permiso para `signal-lab.exe` (la
  aplicación) y otra para `signallab.exe` (la línea de comandos), en redes privadas y de dominio.
  Una instalación solo para ti no puede cambiar el firewall: la aplicación se ofrece a hacerlo,
  con el propio aviso de administrador de Windows, la primera vez que algo escucha; consulta
  [el aviso del firewall](interface.md#firewall-notice).

El tráfico dentro de este mismo equipo (`127.0.0.1`) nunca se filtra, así que todo lo que haces en
loopback funciona sin ninguna regla.

### Instalaciones desatendidas {#windows-unattended}

El instalador acepta estos modificadores en su línea de comandos:

| Modificador | Qué hace |
| --- | --- |
| `/S` | Instala en silencio, sin preguntar nada. |
| `/P` | Instala con una barra de progreso y sin preguntas. |
| `/ALLUSERS` | Instala para todos (necesita un símbolo del sistema elevado). |
| `/CURRENTUSER` | Instala solo para el usuario actual. |
| `/NS` | No crea accesos directos. |
| `/D=C:\Tools\Signal Lab` | Instala en esta carpeta. Debe ir al final y sin comillas. |
| `/NOPATH` | No toca el `PATH`: `signallab` se instala, pero no queda en el `PATH`. |
| `/NOFIREWALL` | No añade reglas de firewall. |

Por ejemplo, una instalación silenciosa para todos sin reglas de firewall, desde un símbolo del
sistema elevado:

```powershell
.\Signal.Lab_[[version]]_x64-setup.exe /S /ALLUSERS /NOFIREWALL
```

### El MSI {#windows-msi}

`Signal.Lab_<version>_x64_en-US.msi` es para el despliegue por parte de TI. Instala para todos los
usuarios y coloca `signallab.exe` junto a la aplicación y esa carpeta en el `PATH` del equipo,
pero no añade ninguna regla de firewall:
eso queda en manos de quien lo despliega (con una directiva de grupo, por ejemplo). La aplicación
sigue ofreciendo la regla la primera vez que escucha. Para instalarlo en silencio:

```powershell
msiexec /i "Signal.Lab_[[version]]_x64_en-US.msi" /qn
```

### SmartScreen {#smartscreen}

Los instaladores aún no tienen firma de código, así que Windows SmartScreen no conoce a su editor
y puede detener la instalación con una advertencia. Elige **Más información**, comprueba que el
archivo es el que descargaste de la página de versiones y luego elige **Ejecutar de todas formas**.
Para asegurarte de que el archivo no ha cambiado, [compáralo antes con `SHA256SUMS.txt`](#checksums).

Las actualizaciones de la propia aplicación están firmadas y se comprueban por separado; consulta
[Actualizaciones](#updates).

## Comprobar una descarga {#checksums}

`SHA256SUMS.txt` enumera la suma de comprobación SHA-256 de cada archivo de la versión, una por
línea, seguida del nombre del archivo. Descárgalo junto al archivo y compara.

En Windows, en PowerShell:

```powershell
Get-FileHash .\Signal.Lab_[[version]]_x64-setup.exe -Algorithm SHA256
Select-String "Signal.Lab_[[version]]_x64-setup.exe" .\SHA256SUMS.txt
```

Los dos hashes deben coincidir (`Get-FileHash` lo muestra en mayúsculas; las mayúsculas y
minúsculas no importan).

En Linux, en la carpeta que contiene ambos archivos:

```bash
sha256sum --check --ignore-missing SHA256SUMS.txt
```

Muestra `OK` tras cada archivo que encontró. Cualquier otra cosa significa que la descarga no es
el archivo publicado: vuelve a descargarlo.

## Linux {#linux}

La aplicación para Linux está compilada para x86_64 (en Ubuntu 22.04) y necesita WebKitGTK 4.1, la
vista web con la que dibuja su ventana. No hay compilación para procesadores Arm; en un equipo Arm,
ejecuta el [servidor](../server/index.md).

### El paquete .deb {#linux-deb}

```bash
sudo apt install ./Signal.Lab_[[version]]_amd64.deb
```

`apt` instala junto con él lo que el paquete necesita. El paquete también coloca la línea de
comandos en `/usr/bin/signallab`.

### El paquete .rpm {#linux-rpm}

```bash
sudo dnf install ./Signal.Lab-[[version]]-1.x86_64.rpm
```

En openSUSE, usa `sudo zypper install` con el mismo archivo. Como el `.deb`, el paquete coloca la
línea de comandos en `/usr/bin/signallab`.

### El AppImage {#linux-appimage}

El AppImage se ejecuta sin instalar:

```bash
chmod +x Signal.Lab_[[version]]_amd64.AppImage
./Signal.Lab_[[version]]_amd64.AppImage
```

No incluye la línea de comandos `signallab`; tómala del
[archivo de la línea de comandos](#cli-archives) si la necesitas.

### El firewall en Linux {#linux-firewall}

En Linux la aplicación no muestra ningún aviso del firewall: `ufw` y `firewalld` funcionan por
puerto, no por programa. Si uno de ellos está activo y otros equipos deben llegar a un monitor, una
escucha, un emulador o un relé, abre allí su puerto, por ejemplo:

```bash
sudo ufw allow 9000/udp
```

El tráfico de loopback no se ve afectado.

## La línea de comandos por separado {#cli-archives}

Los instaladores de escritorio traen `signallab` consigo. Para un equipo sin la aplicación (un
agente de compilación, un servidor, un PC del laboratorio), cada versión incluye también la línea
de comandos por separado:

- `signallab-<version>-windows-x64.zip` contiene `signallab.exe` y la licencia.
- `signallab-<version>-linux-x64.tar.gz` contiene `signallab` y la licencia.

Descomprímelo en una carpeta de tu `PATH`. En Linux:

```bash
tar -xzf signallab-[[version]]-linux-x64.tar.gz signallab
sudo install signallab /usr/local/bin/
signallab version
```

La imagen de Docker del servidor también la contiene. Cómo usarla: [La línea de comandos](../automation/cli.md).

## Actualizaciones {#updates}

### La aplicación de escritorio {#updates-desktop}

La aplicación de escritorio se actualiza solo a partir de versiones publicadas, y solo cuando tú
lo indicas.

- **Busca una vez al día.** Con la casilla [[ui:update.auto]] marcada (lo está por defecto), la
  aplicación busca una versión más reciente poco después de iniciarse, si la última búsqueda fue
  hace un día o más, y otra vez cada vez que pasa otro día mientras está abierta. Desmárcala en
  Acerca de para que deje de buscar por su cuenta.
- **Puedes buscar ahora.** Abre Acerca de (el **?** de la cabecera) y pulsa
  [[ui:update.check]] en la sección [[ui:update.title]].
- **Qué envía la búsqueda.** Pregunta al servicio de actualizaciones del estudio
  (`hub.proanima.net`) qué versión debe recibir esta instalación, y solo acude a la última versión
  de GitHub cuando no puede llegar a ese servicio. La solicitud lleva la versión de la aplicación,
  el sistema y el tipo de procesador, y un número aleatorio creado para esta instalación, que
  permite que una nueva versión llegue primero a una parte de las instalaciones. No se envía nada
  que te identifique a ti ni a este equipo.
- **Cuando se encuentra una versión**, la consola lo indica y la cabecera muestra un botón de
  actualización que abre Acerca de. Allí, [[ui:update.notes]] muestra las notas de la versión.
- **No se instala nada hasta que haces clic.** Pulsa [[ui:update.install]] (si hay tareas en
  curso, el botón avisa de que las detendrá). Signal Lab guarda todo lo que aún está pendiente,
  detiene todas las tareas en curso, descarga la actualización y comprueba su firma con la clave
  integrada en la aplicación (se rechaza una actualización que no haya firmado el proceso de
  publicación), y luego la instala y se reinicia en la nueva versión.

En Windows la actualización ejecuta el instalador sin preguntas y conserva tu carpeta, el `PATH`
y las reglas de firewall. En Linux sustituye el AppImage o instala el nuevo paquete `.deb` o
`.rpm`; con un paquete, el sistema pide antes tu contraseña.

Nunca se ofrecen versiones preliminares ni borradores. Para que ninguna instalación de un equipo
busque por su cuenta (cuando TI despliega las actualizaciones por sí mismo, por ejemplo), define la
variable de entorno `SIGNALLAB_NO_UPDATE_CHECK` con cualquier valor; [[ui:update.check]] sigue
funcionando al pulsarlo. Las compilaciones de desarrollo nunca buscan por su cuenta.

### Un servidor {#updates-server}

Un servidor, y la página que un navegador muestra desde él, se actualiza con su imagen de Docker,
nunca por su cuenta: Acerca de lo indica. Cómo actualizar uno: [El servidor](../server/index.md).

## Desinstalar {#uninstall}

**Windows.** Abre *Configuración → Aplicaciones → Aplicaciones instaladas*, busca Signal Lab y
elige *Desinstalar*. El desinstalador quita `signallab.exe` del `PATH` y, cuando se ejecuta con
derechos de administrador (como en una instalación para todos), elimina todas las reglas de
entrada del firewall para la aplicación y la línea de comandos, incluidas las que creó el aviso de
Windows. Ofrece una casilla para borrar también los datos de la aplicación: son los ajustes propios
de la aplicación (el idioma, el tamaño de los paneles, la pantalla en la que estabas, los últimos
valores escritos en las pantallas), no tus archivos. Una instalación MSI se quita de la misma
forma; se lleva consigo su entrada del `PATH` y deja el firewall a quien la desplegó.

**Linux.** Quita el paquete con el gestor de paquetes con el que lo instalaste, o borra el archivo
AppImage.

Ninguno de los dos toca tu **carpeta de datos**: los experimentos, las bibliotecas de señales y de
emuladores, los informes de ejecución y las exportaciones se quedan en `Documents/SignalLab`, en tu
carpeta personal. Borra tú esa carpeta si quieres eliminarlos.

## Dónde se guardan tus datos {#data}

Todo lo que creas se guarda como archivos normales en una sola carpeta, `Documents/SignalLab`, en
tu carpeta personal, tanto en Windows como en Linux: el experimento actual, la biblioteca de
señales, la biblioteca de emuladores, los informes de ejecución, las exportaciones y las capturas
del Inspector. Cada archivo y su formato se describen en [Archivos y carpetas](../reference/files.md).

## Ejecutarlo como servidor {#server}

Para usar Signal Lab desde un navegador (en un PC del laboratorio junto a los equipos, en un
equipo Linux, en Docker), consulta [El servidor](../server/index.md). En un equipo Linux con
Docker, un solo comando lo instala y lo inicia.

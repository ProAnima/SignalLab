---
title: Pipelines de CI
description: Ejecuta experimentos de Signal Lab en GitHub Actions, GitLab CI o cualquier pipeline, falla la tarea cuando uno falla y conserva un informe de JUnit.
---

# Signal Lab en CI

Los experimentos que ponen en marcha una instalación son también sus pruebas de
regresión. En un pipeline, [`signallab run`](cli.md#cli-run) los ejecuta sin
ventana, imprime cada paso, escribe un informe de JUnit que muestra cualquier
sistema de CI y sale con un código que la tarea entiende:

| Código de salida | El pipeline debería |
| --- | --- |
| `0` | continuar: todos los experimentos pasaron |
| `1` | fallar: un experimento se ejecutó y falló |
| `2` | fallar: un experimento o el comando está mal (un error de validación, un parámetro desconocido, un secreto que falta) |
| `3` | fallar o reintentar: no se pudo ejecutar nada (no se puede alcanzar el servidor o rechaza el token, no se puede abrir un puerto) |

Hay cuatro formas de entrar:

| Forma | Dónde se ejecuta |
| --- | --- |
| [La GitHub Action](#github-actions) | Un runner de Linux, desde la imagen del servidor. |
| [La imagen](#docker) | Cualquier CI que ejecute contenedores: GitLab, Jenkins, un shell con Docker. |
| [El binario](#binary) | Cualquier runner, Windows incluido. |
| [Un servidor de laboratorio](#lab-server) | Las ejecuciones ocurren en un servidor de Signal Lab junto a los dispositivos; el pipeline solo las envía. |

## GitHub Actions {#github-actions}

El repositorio es también una GitHub Action. Ejecuta `signallab` desde la imagen
`ghcr.io/proanima/signallab`, falla la tarea cuando falla un experimento y deja un
informe de JUnit:

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

Una ejecución que no pasa es también una anotación de error en la página de la
ejecución, junto al paso que falló, con el experimento y por qué falló.

### Inputs {#action-inputs}

| Entrada | Qué hace | Predeterminado |
| --- | --- | --- |
| `experiments` | Archivos de experimento o nombres de plantillas incluidas, separados por espacios o líneas. Los patrones como `tests/*.json` se expanden; una ruta con espacios no se admite. | obligatorio |
| `params` | Valores de parámetros, `NAME=VALUE`, uno por línea. | |
| `profile` | Ejecutar con este perfil de los experimentos. | |
| `matrix` | Ejecutar una vez por combinación: `NAME=V1,V2`, un nombre por línea. | |
| `matrix-file` | Combinaciones de un archivo JSON (consulta [Una matriz de ejecuciones](#matrix)). | |
| `fail-fast` | `"true"`: detenerse en la primera ejecución que no pase. | `"false"` |
| `server` | Ejecutar en este servidor de Signal Lab en lugar de en la tarea, p. ej. `http://192.0.2.10:1430`. | |
| `token` | El token de acceso del servidor. Pasa un secreto. | |
| `junit` | Dónde va el informe de JUnit. | `signallab-junit.xml` |
| `timeout` | Segundos que puede durar una ejecución, de 1 a 300. | `300` |
| `version` | La etiqueta de la imagen. | `latest` |
| `image` | Otro registro o una imagen construida localmente; `version` es su etiqueta. | `ghcr.io/proanima/signallab` |
| `lang` | El idioma de los mensajes y del informe: `en`, `ru`, `es`, `fr`, `de`, `pt`, `zh`, `ja`, `ko`, `hi` o `ar`. | `en` |
| `fail-on-error` | `"false"`: no fallar el paso; leer `exit-code` en su lugar. | `"true"` |

Las rutas (`experiments`, `matrix-file`, `junit`) son relativas al directorio de
trabajo del paso.

::: tip
Fija `version` a la versión con la que probaste. `latest` se mueve a cada nueva
versión estable.
:::

### Outputs {#action-outputs}

| Salida | Qué es |
| --- | --- |
| `junit` | La ruta del informe de JUnit. |
| `exit-code` | El código de salida de `signallab`: `0`, `1`, `2` o `3`. |

### Continuar tras un fallo {#fail-on-error}

Un paso que falla no entrega ninguna salida al resto de la tarea. Para decidir tú
mismo, pon `fail-on-error: "false"` y ramifica según `exit-code`:

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

### Secretos en la action {#action-secrets}

El `{{secret.NAME}}` de un experimento lee `SIGNALLAB_SECRET_NAME`. Define esas
variables en el paso (`env:`), a partir de los secretos del repositorio. La
action entrega cada variable `SIGNALLAB_SECRET_…` de su paso a `signallab` por su
nombre; los valores viajan en el entorno, nunca en una línea de comandos, y cada
informe y línea de registro muestra `••••` en su lugar. Un secreto que necesita
el experimento y no está definido falla el paso con código de salida `2`, antes de
enviar nada.

La entrada `token` viaja de la misma forma, como `SIGNALLAB_TOKEN`.

### Cómo se ejecuta la action {#action-runs}

- Necesita un **runner de Linux** con Docker (`ubuntu-latest` lo tiene). En un
  runner de Windows o macOS se detiene con código de salida `2` y una anotación;
  allí, usa [el binario](#binary).
- `signallab` se ejecuta en la imagen con **red del host**: lo que alcanza el
  runner, lo alcanza. Un servicio que la tarea inició en el runner —tu sistema
  bajo prueba, o un contenedor `services:` con un puerto publicado— está en
  `127.0.0.1`.
- Se ejecuta como el usuario del runner, con el espacio de trabajo montado en la
  misma ruta, así que el informe pertenece a la tarea.
- Pasa a `run` exactamente las entradas de arriba, y `--junit`. Para cualquier
  otra cosa que pueda hacer `signallab` —`--report`, `--seed`, `--json`,
  `emulate`—, usa [la imagen](#docker) directamente.

## La imagen, en cualquier CI {#docker}

La imagen del servidor lleva `signallab` como `/usr/local/bin/signallab`.
Sobrescribe el entrypoint para usarlo.

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

Los secretos son variables de CI/CD llamadas `SIGNALLAB_SECRET_<NAME>` (márcalas
como enmascaradas); el entorno de la tarea se las entrega a `signallab` tal cual.

**Docker, desde un shell o cualquier planificador** (cron, Jenkins, un script de
despliegue):

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

- `--network host` permite que las ejecuciones alcancen lo que alcanza el host,
  difusión y multidifusión incluidas, en un host Linux. Sin ella, el contenedor
  alcanza otros hosts solo por unidifusión.
- La imagen se ejecuta como un usuario sin privilegios (uid 10001). `--user` la
  ejecuta como tú en su lugar, para que pueda escribir el informe en tu carpeta;
  sin ella, la carpeta debe ser escribible para uid 10001.
- `-e NAME` sin un valor pasa esa variable desde tu entorno.

## El binario {#binary}

Cada versión incluye `signallab` por separado:
`signallab-<version>-linux-x64.tar.gz` y `signallab-<version>-windows-x64.zip`,
listados en el `SHA256SUMS.txt` de la versión. El de Linux se construye en Ubuntu
22.04 y usa el OpenSSL 3 del sistema (`libssl3`): se ejecuta en esa versión o en
una distribución más reciente.

```yaml
      - name: Signal Lab
        run: |
          curl -fsSL -o signallab.tar.gz https://github.com/ProAnima/SignalLab/releases/download/v[[version]]/signallab-[[version]]-linux-x64.tar.gz
          tar -xzf signallab.tar.gz
          ./signallab run tests/signallab/smoke.json --junit signallab-junit.xml
```

En un runner de Windows, descomprime el zip y ejecuta `signallab.exe` de la misma
forma. Un equipo con la aplicación de escritorio instalada ya tiene `signallab`
en su `PATH`.

## Ejecuciones en un servidor de laboratorio {#lab-server}

Los dispositivos en la red de una instalación son alcanzables desde el
laboratorio, no desde un runner en la nube. Ejecuta allí un
[servidor de Signal Lab](../server/index.md), guarda su token como secreto de CI
y envíale las ejecuciones:

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

El experimento viene del checkout del pipeline; el servidor lo ejecuta con su
propia red, sus secretos y su carpeta de datos, transmite los pasos de vuelta y
conserva el informe (`--report` descarga una copia). El token viene de
`--token-file` o `SIGNALLAB_TOKEN`. Los códigos de salida son los mismos; un
servidor al que no se puede llegar o que rechaza el token es `3`. El runner debe
poder alcanzar el servidor: un runner autoalojado en el laboratorio, o una
dirección de servidor que el runner pueda abrir.

Para ejecutar en el servidor sin `signallab` en absoluto, un script puede llamar
directamente a su API HTTP: consulta
[Ejecutar un experimento por HTTP](../api/run.md).

## Una matriz de ejecuciones {#matrix}

Un experimento, cada destino: cada combinación es una ejecución propia y una suite
de pruebas propia en el informe de JUnit, nombrada por sus valores.

```bash
signallab run tests/smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest
```

En la action, un nombre por línea:

```yaml
        with:
          experiments: tests/signallab/smoke.json
          matrix: |
            device=192.0.2.20:9000,192.0.2.21:9000
            user=admin,guest
          fail-fast: "true"
```

O desde un archivo, con `--matrix-file` (`matrix-file` en la action):

```json
[
  { "device": "192.0.2.20:9000", "user": "admin" },
  { "device": "192.0.2.21:9000", "user": "guest" }
]
```

Cada combinación se comprueba antes de que la primera envíe nada, como máximo 256
ejecuciones vienen de un comando, y `--fail-fast` deja el resto sin iniciar:
aparecen en el informe de JUnit como omitidas. Las reglas están en
[la página de la línea de comandos](cli.md#matrix).

Para probar cada perfil de un experimento, ejecuta una vez por perfil —un paso
cada uno, o una matriz de tareas de tu CI— con `--profile`.

## Informes de JUnit {#junit}

`--junit PATH` (la action siempre escribe uno) contiene una suite de pruebas por
ejecución y un caso de prueba por nodo: el fallo donde ocurrió, en el idioma
elegido, con su código de error y su detalle técnico; los nodos a los que una
ejecución nunca llegó como omitidos; la semilla, el resultado, el archivo y los
valores de la matriz como propiedades. GitHub (con una action de informes),
GitLab (`artifacts:reports:junit`), Jenkins y Azure DevOps lo muestran como
resultados de pruebas. Su estructura está en
[la página de la línea de comandos](cli.md#reports).

La semilla de una ejecución fallida está en las propiedades de su suite y en el
registro: `--seed <that number>` la ejecuta de nuevo con los mismos valores
aleatorios.

## Dependencias a las que llama el sistema bajo prueba {#emulators}

Para probar tu propio sistema contra una API, un dispositivo o un bróker que no
está en CI, deja que Signal Lab haga su papel:

- **Dentro de un experimento**, un nodo [[ui:exp.node.emulator]] hace el papel de
  la dependencia durante una ejecución, y [[ui:exp.node.wait_http]] comprueba lo
  que tu sistema le envió. La salida de la ejecución termina con lo que se pidió a
  cada emulador. Consulta [Emuladores](../tools/emulators.md).
- **Alrededor de tus propias pruebas**, `signallab emulate` responde en segundo
  plano mientras se ejecutan, y sus recuentos te dicen qué se llamó:

  ```bash
  signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
  npm test
  wait
  ```

## Salida para un script {#json}

`--json` imprime un objeto JSON por línea en stdout y nada más: `started`, cada
`step`, `ended` con todo el resultado y un `summary` con `total`, `passed`,
`failed`, `not_started` y `exit_code`. Los errores llevan el `code` estable del
motor, así que un script puede ramificar según él en cualquier idioma. Consulta
[Salida](cli.md#output).

```bash
signallab run tests/smoke.json --json | jq -c 'select(.type == "ended") | {file, outcome, seed}'
```

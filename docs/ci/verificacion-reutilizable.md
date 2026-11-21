# Verificación reutilizable en CI (F5-1)

TASK-041. Dos piezas que **solo llaman al comando de verificación del
proyecto** (ADR [F1-7](../adr/F1-7-verificacion-ci-perfiles-cache-runners.md),
decisión 1; RR-15): no conocen sus pasos, sus grupos ni sus herramientas.

| Plataforma | Pieza | Cómo se usa |
|---|---|---|
| GitHub Actions | [`.github/workflows/verificacion.yml`](../../.github/workflows/verificacion.yml) (`on: workflow_call`) | un job del proyecto con `uses: charliebk/TaskCodeTwo/.github/workflows/verificacion.yml@<sha>` |
| GitLab CI | [`plantillas/gitlab/verificacion.gitlab-ci.yml`](../../plantillas/gitlab/verificacion.gitlab-ci.yml) (`spec:inputs`) | `include: remote` a la URL raw de la plantilla fijada por SHA |

Fuera de F5-1: `carril` y `revision-ia` (F6), y la definición de CI en otro
proyecto de GitLab (F2-9). El informe JSON para el revisor (F5-4), la caché
por árbol (F5-2), el runner propio (F5-5) y el atajo local con confianza
(F5-6) se describen más abajo.

## Entradas

Las mismas en las dos plataformas (lo comprueba
`workflow_y_plantilla_tienen_las_mismas_entradas` en
[`xtask/src/workflows.rs`](../../xtask/src/workflows.rs)); solo cambia cómo
se elige el runner.

| Entrada | Por defecto | Qué es |
|---|---|---|
| `comando` | (obligatoria) | El comando de verificación del proyecto, tal cual se escribe en su shell |
| `argumentos-rapida` | `--perfil rapida` | Lo que se añade al comando en ese perfil |
| `argumentos-impacto` | `--perfil impacto` | Ídem |
| `argumentos-completa` | `--perfil completa` | Ídem; vacío (`""`) llama al comando sin argumentos |
| `argumentos-pesada` | `--perfil pesada` | Ídem |
| `perfiles` | `""` (los del evento) | Perfiles forzados, separados por espacios |
| `huella-arbol` | `""` | Rutas versionadas que entran en la clave de la caché (lockfiles, toolchain), separadas por espacios (F5-2) |
| `huella-local` | `""` | Orden que imprime lo que la suite lee fuera del árbol en el runner del perfil (F5-2); vacía, `local = sha256("")` |
| `cola-de-merge` | `false` | Booleana. `true` solo si la rama base tiene cola de merge (GitHub) o merge trains (GitLab): el PR corre la impacto y la cola la completa (F5-3) |
| GitHub: `runner-<perfil>` | `'"ubuntu-latest"'` | `runs-on` del perfil, en JSON: una cadena o una lista de etiquetas (`'["self-hosted","windows","toolsaweb"]'`) |
| GitHub: `shell` | `bash` | Shell en el que corre el comando (`cmd` para un `.bat`) |
| GitLab: `tags-<perfil>` | `[]` | Etiquetas del runner del perfil |
| GitLab: `stage` | `test` | Etapa de los jobs |

> **Aviso:** `comando` y `argumentos-*` se interpolan tal cual en la línea del
> shell (`run:` en GitHub, `script:` en GitLab). Son literales que escribe el
> proyecto: **nunca** les pases datos del evento (`${{ github.head_ref }}`, el
> título del PR, `$CI_COMMIT_REF_NAME`…), porque quien nombra la rama o escribe
> el título ejecutaría código en el runner. El lint de lista blanca revisa el
> workflow y la plantilla, no al llamador: esto no lo detecta.

**Perfil según el evento** (F1-7, decisión 4), si `perfiles` va vacío:

| Evento | GitHub | GitLab |
|---|---|---|
| PR / MR que no es la release | completa | completa (`verificacion-completa`) |
| PR / MR que no es la release, con `cola-de-merge: true` | impacto | impacto (`verificacion-impacto`) |
| PR / MR de `develop` a `main` (release), con o sin cola | completa y pesada | completa y pesada |
| `merge_group` / pipeline de `merge_train` | completa | completa con `cola-de-merge: true`; sin ella, la del MR (completa) |
| `workflow_dispatch` | completa | — |
| `push` de rama | nada | nada |

El comando recibe la base del PR en `RAMA_BASE` y el perfil en `TC2_PERFIL`.
Resultado: código de salida (0 verde, otro rojo).

**Cola de merge** (TASK-043, F5-3; [adenda de F1-7](../adr/F1-7-verificacion-ci-perfiles-cache-runners.md#adenda-f5-3-task-043--impacto-en-el-pr-solo-con-cola-de-merge)):
con `cola-de-merge: true` el PR corre solo la impacto y la completa la corre
la cola sobre el árbol que acaba en `develop`. La impacto **nunca se cachea**
(depende de `RAMA_BASE`): no deja línea `arbol=` y se ejecuta siempre. El
resumen del check lleva `cola-de-merge=<true|false>`.

> **Aviso:** pon `cola-de-merge: true` solo si la rama base tiene de verdad
> una regla de cola (GitHub) o merge trains (GitLab). Sin cola, `develop`
> integraría con solo la impacto, sin la completa. Nada lo detecta todavía
> (F3-8); el check lo deja escrito en `cola-de-merge=`. En GitLab el tren
> (`CI_MERGE_REQUEST_EVENT_TYPE == "merge_train"`) es de pago: no verificado.

## Qué publica

- **GitHub**: tres jobs. `buscar` (Linux, `contents: read`) elige los perfiles;
  `ejecutar` corre uno por perfil en su runner, con `contents: read`, sin
  secretos y con `actions/checkout` fijado por SHA y `persist-credentials:
  false`; `verificacion` (sin checkout, `checks: write`) suma los resultados y
  crea con `GITHUB_TOKEN` un check-run llamado exactamente **`verificacion`**,
  de la App `github-actions` (`app_id` 15368), en el head del PR. Ese es el
  check obligatorio de F1-5 y el vehículo de `output.summary` para F5-2. Los
  jobs salen además como checks `<job que llama> / <job>`.
- **GitLab**: un job `verificacion-<perfil>` por perfil del evento; el estado
  es el del pipeline.

El job que llama tiene que conceder `contents: read`, `checks: write` y
`actions: read` (un workflow reutilizable no puede tener más permisos que su
llamador; `actions: read` lo pide `buscar` desde F5-2). En un PR desde un fork el
`GITHUB_TOKEN` es de solo lectura: el publicador falla y el PR queda en rojo
(falla cerrado).

## ToolsaWeb (sin scripts nuevos)

`scripts\verificar.bat` no acepta `--perfil`: sin argumentos es la completa y
admite `--rapida`, `--selectiva` y `--pesada` (líneas 68-79; otro argumento,
`exit /b 2`). Se declaran los argumentos de cada perfil:

```yaml
# .github/workflows/verificacion.yml de ToolsaWeb
name: verificacion
on:
  pull_request:
    branches: [develop, main]
permissions: {}
jobs:
  verificacion:
    permissions:
      contents: read
      checks: write
      actions: read
    uses: charliebk/TaskCodeTwo/.github/workflows/verificacion.yml@<sha>
    with:
      comando: scripts\verificar.bat
      shell: cmd
      argumentos-rapida: --rapida
      argumentos-impacto: --selectiva
      argumentos-completa: ""
      argumentos-pesada: --pesada
      runner-completa: '"windows-latest"'
      runner-pesada: '["self-hosted","windows","toolsaweb"]'
```

La pesada lee `exampleData` (34 GB) y solo corre en la máquina de Carlos
(F5-5); hasta entonces un PR de release se queda pendiente esperando ese
runner (falla cerrado, F1-7 decisión 5).

## OpenDockTrading (sin scripts nuevos)

`scripts\verificar.bat` no lee argumentos y es una sola verificación: basta el
comando, con la completa sin argumentos.

```yaml
jobs:
  verificacion:
    permissions:
      contents: read
      checks: write
      actions: read
    uses: charliebk/TaskCodeTwo/.github/workflows/verificacion.yml@<sha>
    with:
      comando: scripts\verificar.bat
      shell: cmd
      argumentos-completa: ""
      runner-completa: '"windows-latest"'
```

## GitLab

```yaml
# .gitlab-ci.yml del proyecto
include:
  - remote: https://raw.githubusercontent.com/charliebk/TaskCodeTwo/<sha>/plantillas/gitlab/verificacion.gitlab-ci.yml
    inputs:
      comando: scripts/verificar.sh
      argumentos-completa: ""
      tags-completa: [saas-linux-small-amd64]
```

Un MR abre `verificacion-completa`; un push de rama no abre nada salvo que se
fuerce con `perfiles`. La plantilla no define `workflow:rules`: si el proyecto
quiere evitar el pipeline de rama duplicado con el de MR, lo pone en su
`.gitlab-ci.yml`.

## Lint de lista blanca (F1-8, decisión 1)

[`xtask/src/workflows.rs`](../../xtask/src/workflows.rs) corre con
`cargo test` (y por tanto en `cargo xtask verificar --perfil rapida`). Sobre el
workflow: eventos admitidos (`workflow_call`, `pull_request`, `push`,
`merge_group`, `workflow_dispatch`), `permissions` arriba y en cada job (sin
`write-all`; de lectura solo `contents`, `checks` y `actions`, F5-2), las
únicas acciones admitidas son `actions/checkout` fijada por
SHA, con `persist-credentials: false`, sin `clean:` distinto de `true` (F5-5)
y nunca en un job que escribe (F5-2:
antes solo se prohibían `ref:` y `repository:`) y `actions/upload-artifact` fijada por SHA y solo en un job sin
escritura (F5-4), ningún `secrets: inherit`, ningún secreto en un job sin escritura y
ningún `github.event.*`, `github.head_ref` ni `toJSON(github…)` en `run:`
(también con corchetes, `github['event']`, `secrets['X']`, o con espacios);
`env` y `defaults` de la raíz sin secretos (y `defaults` sin datos del evento);
un `uses:` a nivel de job fijado por SHA. Sobre la plantilla: `spec:inputs`,
ningún `secrets:`, ninguna variable que escribe el autor (`CI_COMMIT_MESSAGE`,
`CI_MERGE_REQUEST_TITLE`…) en `script:` y ningún `rules:if` que lea una
variable de job. El lector falla en cerrado: lo que no entiende (sangría
inesperada, continuación de un escalar) es un error, no se descarta. Cada
regla tiene su mutación negativa en las pruebas.

## Informe JSON (F5-4)

TASK-044. El comando de verificación del proyecto escribe, si sabe, un informe
JSON en la ruta de la variable **`TC2_INFORME`**, también cuando acaba en rojo.
La plataforma solo lo transporta (F1-7, decisión 1: no conoce los pasos):

| Plataforma | `TC2_INFORME` | Transporte |
|---|---|---|
| GitHub | `${{ runner.temp }}/informe-verificacion.json` (env del paso) | `actions/upload-artifact` con `if: always()` e `if-no-files-found: ignore`, artefacto `informe-verificacion-<perfil>`; el publicador añade `informe=informe-verificacion-<perfil>[,…]` a `output.summary` |
| GitLab | `$CI_PROJECT_DIR/.tc2/informe-verificacion.json` | `artifacts: {name: informe-verificacion-$TC2_PERFIL, when: always, paths: [.tc2/informe-verificacion.json]}` |

- **Contrato**: [`crates/tc2/esquemas/informe-verificacion.schema.json`](../../crates/tc2/esquemas/informe-verificacion.schema.json),
  `version: 1`: `perfil`, `resultado` (`verde`/`rojo`), `commit` (40 hex o
  `null`), `inicio` y `fin` (UTC), `pasos[{nombre, resultado, segundos}]` y
  `tests{total, fallidos}` opcional (F1-11, práctica 8). Una clave desconocida
  u otra versión es error.
- **Lector**: `tc2::verificacion::informe::leer` y `seccion_revisor(&informe,
  artefacto)`, el texto con el que el revisor IA (F6-3) cita el informe y se
  compromete a no relanzar los pasos en verde.
- **Productor**: `cargo xtask verificar` (fmt, clippy, test y, en `completa`,
  reproducible), sin recuento de tests. Su prueba compara el JSON con los
  fixtures `crates/tc2/tests/fixtures/informe-verificacion/` que lee tc2.
- **Quién decide**: el informe lo escribe código del PR; es detalle para citar.
  El hecho autoritativo para no repetir la suite es el check `verificacion` de
  `github-actions` (`app_id` 15368). Si el comando no escribe el informe, no
  hay artefacto y el revisor solo tiene el resumen del check, que basta para no
  relanzar un perfil en verde. El publicador no abre el artefacto.

## Caché por árbol (F5-2)

TASK-042. Cero repeticiones sobre el mismo árbol (F1-7, decisiones 2 y 3;
F1-8, decisión 7), sin componentes nuevos: los tres jobs de siempre.

- **`buscar`** (Linux; `contents`, `checks` y `actions` de lectura; no
  ejecuta nada del PR): checkout del commit verificado `V` (el merge ref en un
  PR) con `fetch-depth: 3`; `V`, sus padres y sus abuelos (como mucho 7) salen
  de `git`. Para cada uno pide
  `GET commits/{c}/check-runs?check_name=verificacion&app_id=15368&filter=all`
  (el más reciente primero) y solo cuenta un check-run en verde cuya
  ejecución (su check suite →
  `GET actions/runs?check_suite_id={id}`) tenga en `referenced_workflows` este
  workflow (`charliebk/TaskCodeTwo/.github/workflows/verificacion.yml@…`) con
  el mismo `sha` que la ejecución actual (la versión del workflow, leída con
  `GET actions/runs/{run_id}`). De su resumen toma las líneas `arbol=`
  con el árbol de `V` y el perfil, el runner y la clave actuales: son los
  candidatos de cada perfil, en el orden de los commits. Si el diff de `V`
  frente a su primer padre (la base) toca `.github/workflows/**` o
  `.taskcode/**`, no hay candidatos.
- **`ejecutar`** (runner del perfil): calcula la huella local (la orden de
  `huella-local` en el shell del proyecto, y `sha256sum` en `bash`); el primer
  candidato con el mismo `local` es el **acierto** y el paso del comando sale
  `skipped`. Corre siempre en el runner del perfil: un runner propio caído deja
  el check pendiente aunque haya candidato (falla cerrado).
- **`verificacion`** (publicador): lee el árbol de `V` con `GET commits/{V}`;
  si no es el de `buscar`, el check sale rojo y sin líneas. Con el resultado en
  verde añade al resumen una línea por perfil.

**La línea** (contrato para F8-1), una por perfil, debajo de la de siempre
(`perfiles= resultado= informe= ejecucion=`) y de `cola-de-merge=` (F5-3);
la impacto no tiene línea:

```
arbol=<árbol de V> perfil=<p> runner=<r> clave=<k> local=<h> origen=<sha> ejecutada
arbol=<árbol de V> perfil=<p> runner=<r> clave=<k> local=<h> origen=<sha> reutilizada de <commit>
```

- `runner`: el `runner-<perfil>` declarado, sin comillas ni espacios (una
  lista de etiquetas, unida por comas).
- `clave = sha256(perfil|runner|huella-arbol|versión del workflow|órdenes)` (la
  versión es el `sha` de `referenced_workflows` de la ejecución), con
  `huella-arbol = sha256(git ls-tree -r HEAD -- <rutas>)` (o de nada sin rutas)
  y `órdenes = sha256` del JSON `[comando, argumentos-<perfil>, shell,
  huella-local]`: un verde de otras órdenes (p. ej. pasadas por `vars` o por
  un `workflow_dispatch`) no se reutiliza. La regla de rutas usa
  `git diff --no-renames`, para que mover un fichero fuera de
  `.github/workflows/` o `.taskcode/` cuente como tocarlo (revisión de la
  ronda 1).
- `local = sha256` de lo que imprime `huella-local` en el runner del perfil.
- `origen`: el commit (head del PR) donde se ejecutó de verdad; una línea
  reutilizada conserva el del candidato. `reutilizada de <commit>`: el commit
  en cuyo check estaba la línea que dio el acierto.

**Cómo leer un acierto**: en el job `<llamador> / <perfil>`, el paso «cache
por arbol» dice `reutilizada de <commit>` y el paso del comando sale
`skipped`; en el check `verificacion`, la línea del perfil termina en
`reutilizada de <commit>`. Solo cuenta un check en verde: una línea de un
check rojo no es candidata.

**Coste**: hasta 7 lecturas de check-runs más una de la ejecución por cada
check en verde encontrado, y una de `commits/{V}` en el publicador. Un acierto
despierta igualmente el runner del perfil (huella local).

**GitLab** (D2): sin búsqueda; cada pipeline verifica entero (degradación
declarada, RR-15). Con el comando en verde, `after_script` escribe la línea
(`origen=$CI_COMMIT_SHA ejecutada`, `runner` = `CI_RUNNER_TAGS`, versión vacía en
la clave) en `.tc2/cache-verificacion-<perfil>.txt`, dentro del artefacto
`informe-verificacion-<perfil>` (un job de GitLab tiene un solo archivo de
artefactos). `after_script` no cambia el estado del job: en un runner sin `sh`
no hay línea.

## Runner propio (F5-5)

TASK-045. ADR [F1-8](../adr/F1-8-seguridad-modelo-de-amenazas.md), decisión 6
(D3b) y su [adenda F5-5](../adr/F1-8-seguridad-modelo-de-amenazas.md#adenda-f5-5-task-045--runner-propio-solo-en-la-release);
[F1-7](../adr/F1-7-verificacion-ci-perfiles-cache-runners.md), decisión 5
(solo repos privados; runner caído, falla cerrado). No hay entradas nuevas:
el runner propio se elige con `runner-<perfil>`.

**Regla de uso** (D1): un perfil en un runner propio solo corre en la
**release**, un `pull_request` de `develop` a `main` de un **repo privado** con
el head **en el mismo repo**. Lo comprueba el paso «Runner propio» de `buscar`
(Linux, sin código del PR) antes de encolar nada en el runner. Cuenta como
runner propio un `runner-<perfil>` con la etiqueta `self-hosted` (sin distinguir
mayúsculas) o que no es cadena ni lista (un grupo de runners, por ejemplo). Si
falta una condición, `buscar` sale con `exit 2`, el motivo (`repo publico`,
`no consta si el repo es privado`, `solo en la release, un pull_request de
develop a main` con el evento, la base y el origen, o `el head del PR viene de
otro repo`) y qué hacer: poner ese `runner-<perfil>` en un runner de GitHub. Un
`runner-<perfil>` o unos perfiles que no son JSON también fallan. Con todos los
perfiles en runners de GitHub el paso solo escribe `Runner propio: ningun
perfil de este evento lo usa` y todo sigue igual.

Por eso solo la **pesada** va en el runner propio: la completa corre también
en los PR de tarea y en `workflow_dispatch`, que fallarían. Para ToolsaWeb:

```yaml
    with:
      comando: scripts\verificar.bat
      shell: cmd
      runner-completa: '"windows-latest"'
      runner-pesada: '["self-hosted","windows","toolsaweb"]'
```

**Etiquetas** (D4): `["self-hosted","windows","<proyecto>"]`, siempre con
`self-hosted` en el `runner-<perfil>`. «Runner propio» de `buscar` solo
reconoce el runner propio por esa etiqueta: con `'["windows","<proyecto>"]'`
(o un runner registrado con `--no-default-labels`) el job llega a tu máquina y
lo para el aislamiento de abajo, que repite la regla del evento y no depende
de la etiqueta, antes de sacar el código.

**Aislamiento** (D3b a): el primer paso de `ejecutar`, antes del checkout,
lleva `if: runner.environment == 'self-hosted'` (en un runner de GitHub vale
`github-hosted` y el paso se salta) y corre en Windows PowerShell con
`-NoProfile -ExecutionPolicy Bypass` solo para ese proceso: con la directiva
`AllSigned` de la máquina, el script sin firmar que genera el runner no llega a
ejecutarse (lo encontró el E2E de TASK-045). Falla, con su motivo, si el
runner no es Windows; si el evento no es la release de un repo privado con el
head en el mismo repo (la misma regla de `buscar`, con los datos del evento por
`env:`); si la cuenta es `SYSTEM` (SID `S-1-5-18`) o si pertenece al grupo
Administradores (SID `S-1-5-32-544`, también cuando UAC lo deja solo para
denegar); y si no encuentra `bash` o `jq` en el `PATH` (la caché por árbol usa
`jq`). El script hace `exit 2`, pero con `-Command` Windows PowerShell lo
devuelve como 1: el paso falla igual. Si todo va bien escribe el usuario y el
`bash`. Que la cuenta no lea tu perfil y que los datos sean de solo lectura no
lo comprueba: depende del alta.

**Directorio limpio** (D3b b): `actions/checkout` con su `clean: true` por
defecto; el lint da error si un checkout lleva `clean:` con otro valor.

**Directorio de `main` aparte** (D3b c): se cumple por construcción, porque en
el runner propio solo corre la release (adenda F5-5 de F1-8).

### Alta mínima (D2a, unos 15 minutos)

En una consola de PowerShell **de administrador** en tu máquina, para un repo
**privado**:

1. **Usuario de servicio sin administración**:
   `New-LocalUser -Name tc2-runner -Password (Read-Host -AsSecureString "Contraseña") -PasswordNeverExpires`
   (si la contraseña caduca, el servicio deja de arrancar).
   No lo añadas a ningún grupo más.
2. **Carpeta del runner** fuera de tu perfil, una por repo, con permiso de
   modificación para ese usuario:
   `New-Item -ItemType Directory C:\ProgramData\tc2-runner\<repo>` y
   `icacls C:\ProgramData\tc2-runner\<repo> /grant "tc2-runner:(OI)(CI)M"`.
3. **Descarga**: en GitHub, *Settings → Actions → Runners → New self-hosted
   runner*, Windows x64. Sigue en esa carpeta los pasos de descarga y
   comprobación del hash que da la página; de ahí sale también el token de
   registro (dura poco). Si la página da 404 (repo privado y el navegador con
   otra cuenta), descarga el zip de `actions/runner` de su release, compara el
   SHA-256 con el que publica GitHub y pide el token con
   `gh api -X POST repos/<dueño>/<repo>/actions/runners/registration-token --jq .token`.
   **bash y jq**: el job usa el bash de Git y `jq`, que Windows no trae. Deja
   `jq.exe` (release de `jqlang/jq`, con su SHA-256) en
   `C:\ProgramData\tc2-runner\bin` y crea en la carpeta del runner un `.env`
   con `PATH=C:\ProgramData\tc2-runner\bin;C:\Program Files\Git\bin;C:\Program Files\Git\usr\bin;C:\WINDOWS\system32;C:\WINDOWS;C:\WINDOWS\System32\WindowsPowerShell\v1.0`.
   El runner lo lee al arrancar: si lo cambias, reinicia el servicio. El paso
   de aislamiento falla si no encuentra `bash` o `jq`.
4. **Registro como servicio con ese usuario**:

   ```powershell
   .\config.cmd --url https://github.com/<dueño>/<repo> --token <token> `
     --name <máquina>-<repo> --labels <proyecto> `
     --runasservice --windowslogonaccount .\tc2-runner
   ```

   No escribas la contraseña en la línea de órdenes
   (`--windowslogonpassword`): queda en el historial.
5. **Datos de solo lectura**: `icacls <ruta de los datos> /grant
   "tc2-runner:(OI)(CI)RX"` (en ToolsaWeb, `exampleData`). Comprueba con
   `icacls C:\Users\<tú>` que ni `tc2-runner` ni `Usuarios` tienen acceso a tu
   perfil.
6. **Una instancia por máquina y repo**: un runner ejecuta un job a la vez,
   así que dos jobs nunca comparten `target/` ni `_work` (corrige la
   divergencia 6). No registres una segunda instancia para el mismo repo en
   la misma máquina.
7. **Comprobación**: en *Settings → Actions → Runners* el runner sale
   `Idle` con `self-hosted`, `Windows` y `<proyecto>`. En el primer PR de
   release, el paso «aislamiento del runner propio» escribe `Runner propio:
   usuario <máquina>\tc2-runner sin administracion; bash: ...`.

### Retirada

1. Cambia antes `runner-pesada` del llamador a un runner de GitHub (o quítalo):
   si no, la pesada de la release espera en cola hasta que GitHub la cancela a
   las 24 h, y el PR no se mergea (falla cerrado, F1-7 decisión 5).
2. *Settings → Actions → Runners →* el runner *→ Remove*, y en la máquina la
   orden que muestra la página (`.\config.cmd remove --token <token>`): quita
   el registro, la configuración local y el servicio. Sin acceso a la
   máquina, *Force remove this runner*.
3. Borra la carpeta del runner y el usuario: `Remove-LocalUser tc2-runner`.

### Qué está comprobado y qué no

- **En la documentación de GitHub** (2026-11-20): `runner.environment` vale
  `github-hosted` o `self-hosted` (contexto `runner`); en un workflow
  reutilizable el contexto `github` es el del llamador, y el reutilizable
  puede usar los runners propios del repo llamador; un job sin runner espera
  en cola y falla a las 24 h; las etiquetas no distinguen mayúsculas.
- **En el código del runner** (`actions/runner`, `Constants.cs` y
  `CommandSettings.cs`; la página de GitHub no las lista): `config` acepta
  `--runasservice`, `--windowslogonaccount`, `--windowslogonpassword`,
  `--labels`, `--name`, `--url`, `--token` y `--no-default-labels`, y
  `remove` acepta `--token`.
- **Probado en local**: el `run:` de «Runner propio» con `bash` y datos
  simulados (pruebas de `xtask/src/workflows.rs`), y el de aislamiento con
  Windows PowerShell y el shell exacto del paso: un PR de fork, un repo
  público u `origen Develop` se rechazan por la regla del evento; con una
  cuenta del grupo Administradores falla (código 1).
- **E2E con un runner real** (TASK-045, 2026-11-21, repo privado de pruebas):
  la pesada de la release corrió en el runner propio con `tc2-runner` sin
  administración y los datos de solo lectura; `github.event.repository.private`
  y `github.event.pull_request.head.repo.full_name` llegan con valor al
  reutilizable; la etiqueta `self-hosted` se pone sola. Encontró la directiva
  `AllSigned`, la falta de `jq` y la huella con `\` en Windows, ya corregidos.
  En un repo público el runner propio se rechaza en `buscar` sin encolar nada.
- **No verificado**: que `config.cmd` pida la contraseña si no se le pasa, y
  que conceda solo el derecho de iniciar sesión como servicio; un acierto de la
  caché por árbol en el runner propio (en el E2E el diff tocaba el workflow y
  la regla de rutas no reutiliza, por diseño).

## Hook local rápido, opcional y con confianza (F5-6)

TASK-046. Fuera de la CI, la misma verificación tiene un atajo local: un
**comando determinista** y, si la persona quiere, un hook de Claude Code que
lo llama (ADR [F1-11](../adr/F1-11-extensiones-declaradas.md), decisión 10).
Quitar el hook no cambia ninguna garantía: las puertas son el check
`verificacion`, el check `estado` y la transición.

```text
tc2 confiar verificacion.comando        # una vez, tras revisar la orden y sus scripts
tc2 verificar --perfil rapida [--json]  # cualquier perfil declarado (D6)
tc2 verificar --perfil rapida --hook    # lo mismo, devuelto como lo espera el evento Stop
```

**Nada del repo se ejecuta sin confianza** (ADR
[F1-8](../adr/F1-8-seguridad-modelo-de-amenazas.md), decisión 5).
`tc2 verificar` lee `verificacion` de `.taskcode/config.yml` **del árbol de
trabajo** (D3a), calcula la huella de la orden (con la shell) y de lo que
ejecuta (los scripts versionados que nombra y, a su vez, los que nombran
estos, con `%~dp0` resuelto, más los globs de `confianza_incluye`) y la
compara con la confiada:

| Estado | Qué hace `tc2 verificar` |
|---|---|
| `nunca` (sin confianza en este repo) | no ejecuta nada; código 3, `motivo=sin-confianza confianza=nunca` y el aviso dice que hay que revisar y ejecutar `tc2 confiar verificacion.comando` |
| `cambiado` (la orden, un script del cierre o un fichero de `confianza_incluye` cambió) | igual, con `confianza=cambiado` |
| `confiado` | lanza `verificacion.shell` con `<comando> --perfil <p>`, `TC2_PERFIL=<p>` y el `timeout` del perfil; la salida del comando va a stderr |

La confianza vive **fuera del árbol**, en
`%LOCALAPPDATA%\taskcodetwo\confianza\` (Windows) o
`$XDG_CACHE_HOME/taskcodetwo/confianza/` (`~/.cache/...` si no está
definida), un fichero por repo con nombre el sha256 del directorio común de
Git: los worktrees del repo la comparten y otro clon no. Un fichero del repo
que diga «confiado» no cuenta. Si se borra la carpeta, todo vuelve a `nunca`;
`tc2 desinstalar --maquina` la borra.

**Resultado** (F1-11, decisión 8): código 0 del comando, `resultado=verde`;
otro código, `motivo=rojo` y código 3; un `timeout` agotado es rojo
(`motivo=timeout`) salvo en `rapida`, donde es `resultado=excedido` con
código 0 (presupuesto que no bloquea). Al agotarse se mata el comando y lo
que haya lanzado (`taskkill /T` en Windows, el grupo de procesos en Unix).

### Activar el hook (opcional, D4a y D5a)

No hay orden que lo instale: quien lo quiera añade esta entrada a su
`.claude/settings.local.json` del proyecto (personal, no se versiona) o a su
`~/.claude/settings.json`:

```json
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "tc2 verificar --perfil rapida --hook",
            "timeout": 630
          }
        ]
      }
    ]
  }
}
```

`timeout` (segundos, por defecto 600 en Claude Code) tiene que ser algo
mayor que `verificacion.perfiles.rapida.timeout`, o Claude Code corta el hook
antes que `tc2`. El evento `Stop` no admite `matcher`. Comportamiento con
`--hook` (la decisión es la misma que sin él; solo cambia cómo se devuelve):

| Caso | Código | Qué ve el agente |
|---|---|---|
| `stop_hook_active: true` en el JSON de stdin (Claude Code ya continúa por un hook) | 0 | nada; no se ejecuta. Se mira antes que los argumentos |
| entrada mal escrita (perfil inexistente, opción desconocida) | 0 | no bloquea; una línea en stderr dice que se corrija en `settings.json` |
| sin confianza, sin perfil `rapida` o con la configuración inválida | 0 | no bloquea; una línea en stderr dice qué hacer (`tc2 confiar verificacion.comando`, o el arreglo de la configuración); no se ejecuta |
| rápida en verde o `excedido` | 0 | nada |
| rápida en rojo | 2 | bloquea la parada; en stderr el motivo y las últimas 15 líneas de la salida del comando |

Con otro perfil (`--perfil completa --hook`, fuera de lo documentado), su
rojo y su `timeout` agotado también salen con 2. La salida del comando se
recoge en un fichero temporal de nombre aleatorio, creado en exclusiva, del
que solo se lee el final y que se borra al terminar (si Claude Code mata el
hook por su `timeout`, puede quedar en la carpeta temporal del usuario).
`--hook` lee el JSON de stdin hasta que se cierra: Claude Code lo cierra tras
escribirlo; un lanzador que lo deje abierto hace esperar a `tc2` (aceptado,
MENOR-3 de la ronda 1).

`desinstalar` no edita `settings.json`: avisa de los hooks que nombran `tc2`
y los deja (F2-14).

### Presupuesto de 1 s (D1a)

«≤ 1 s» es el coste propio de `tc2` hasta decidir (leer la configuración,
cierre, huella, almacén y lanzar o rechazar), en frío y en caliente; cuando no
se ejecuta nada es el tiempo total. La rápida misma está acotada por su
`timeout`. La huella reutiliza el sha de cada fichero cuyo `(tamaño, mtime)`
no cambió (D2a); la orden se hashea siempre y `tc2 confiar` rehashea todo.

Medido con la prueba pesada
`presupuesto_de_un_segundo_con_un_cierre_como_el_de_toolsaweb` (51 scripts
versionados, 1 210 462 bytes, orden que sale enseguida; tiempo de pared de
`tc2 verificar --perfil rapida` entero, con el arranque de `tc2`, de `git` y
de `cmd`), en Windows 11, binario `--release`, cuatro ejecuciones el
2026-11-21 sin controlar la carga de la máquina:

| Caso | Tiempo (ms) |
|---|---|
| sin confiar (cierre entero hasheado, no se ejecuta) | 250 – 684 |
| en frío (todos los `mtime` cambiados: 1 210 462 bytes rehasheados) | 318 – 595 |
| en caliente (0 bytes rehasheados) | 249 – 419 |

Con el binario de depuración, el peor caso fue 530 ms (en frío). Tras la
ronda 2 de revisión (scripts de nombre raro por su nombre), dos ejecuciones
en release: 284 – 401 ms sin confiar, 322 – 368 ms en frío y 258 – 292 ms en
caliente; y con un script de nombre raro más en el cierre (52 ficheros, los
textos se releen también en caliente, medida a mano sin dejarla en la
prueba): 239 – 306, 306 – 319 y 254 – 279 ms. Se lanza a
mano: `cargo test -p tc2 --release --test verificar -- --ignored --nocapture`
(la verificación del repo no corre las pruebas ignoradas).

**Cómo se forma el cierre.** Se siguen los tokens con forma de ruta (llevan
`.` o `/`, con letras también no ASCII, como `verificación.bat`) y, enteros,
los textos entre comillas de una línea (`call "%~dp0lib\mi paso.bat"`), con
`%~dp0` resuelto a la carpeta del script. Como emparejar comillas falla con
un apóstrofo suelto (`echo Don't & call "%~dp0lib\mi paso.bat"`) o con
comillas anidadas (`"$(dirname "$0")/lib/mi paso.sh"`), además todo script
versionado cuyo nombre lleve un carácter que el tokenizador no junta
(espacio, apóstrofo…) entra en el cierre si su nombre de fichero aparece
literalmente en la orden o en el texto de un fichero del cierre, se siga o
no la ruta: puede sumar de más a la huella, nunca dejar fuera uno nombrado.
Si el repo tiene scripts así, los textos del cierre se releen (sin
rehashear) también en caliente. En Windows y macOS las rutas se
comparan sin distinguir mayúsculas. Los globs de `confianza_incluye` admiten
`./` al principio y siguen la misma regla de mayúsculas; un glob que no
compila es error de la configuración (no se confía ni se ejecuta nada) y uno
que no casa con ningún fichero versionado se avisa en `tc2 confiar`
(`globs-sin-ficheros=` y una línea en stderr).

**Límites** (F1-8, decisión 5), riesgos residuales aceptados:

- No entra en el cierre el código al que se llega por herramientas
  (`build.rs`, tests, dependencias) ni lo que se carga por nombre de módulo y
  no por ruta; eso se declara en `confianza_incluye`.
- Solo cuentan los ficheros **versionados**: un script sin versionar o
  ignorado que la orden llame se ejecuta sin entrar en la huella.
- Un enlace simbólico a fichero se hashea por su destino; un enlace a
  **carpeta** o un **submódulo** dejan sus scripts fuera (`git ls-files` no
  los lista dentro). No probado en esta máquina (sin permiso para `mklink`).
- Un cambio que conserve tamaño y `mtime` no se detecta en caliente (D2a);
  si el sistema no da `mtime`, no hay caché y se rehashea siempre.
- Nombres que el cierre no reconoce como el nombre literal del fichero (en
  todos, la salida es nombrar el script en `confianza_incluye`):
  - el espacio escapado con barra en sh (`. ./lib/mi\ paso.sh`) o con `^`
    en cmd (`call lib\mi^ paso.bat`);
  - nombres partidos por comillas o formados con variables (`%N%.bat`,
    `"$DIR/$NOMBRE.sh"`);
  - referencias sin extensión que cmd completa por `PATHEXT`
    (`call "lib\mi paso"`);
  - scripts en UTF-16: se leen con `from_utf8_lossy` y sus rutas no se
    reconocen.
- Entre hashear y ejecutar hay una ventana (TOCTOU): la shell relee los
  scripts al ejecutarlos y quien pueda escribir en el árbol en ese instante
  ejecuta lo suyo.

Este repo aún no puede estrenar el hook: su `.taskcode/config.yml` es de la
0.16 (sin `formato: 2`) y `tc2 verificar` la rechaza; con `--hook` sale con 0
y avisa.

## Divergencias con los ADR

1. **Argumentos por perfil** (D1): F1-7, decisión 1, fija `--perfil
   <p>` como contrato. Para que ToolsaWeb lo use sin un envoltorio, cada perfil
   acepta sus argumentos, con `--perfil <p>` por defecto. Es transitorio hasta
   F9 (migración).
2. **`carril`, `revision-ia` y `ci_config_path`** (D4): F1-13 los ponía en la
   plantilla de GitLab; quedan para F2-9 y F6.
3. **Línea de la caché en GitLab** (D3): irá en un artefacto del job (F5-2),
   no en el estado del commit (255 caracteres, F1-13 divergencia 5).
4. **`workflow_run` para el publicador** (F1-8 pedía valorarlo): no se usa. El
   publicador no hace checkout ni lee el PR; solo suma resultados de `needs`.
   Moverlo a `workflow_run` lo sacaría del contexto del PR y obligaría a buscar
   el head por la API; se reconsidera con F5-2 si el resumen empieza a leer
   datos del PR.
5. **Un solo llamador por repo** (observado en el E2E): si dos workflows del
   mismo repo llaman al reutilizable en el mismo commit, salen dos check-runs
   `verificacion` (uno por llamador) y GitHub puede listar el creado con
   `GITHUB_TOKEN` en la suite de otra ejecución (el rojo de un llamador
   apareció en la lista de jobs del otro). El proyecto declara un único
   llamador; desde F5-2 además `buscar` enlaza cada check-run con su ejecución
   por la check suite, y con dos llamadores ese enlace dejaría de ser fiable.
6. **Concurrencia por PR (o ref) y perfil**: el grupo es
   `verificacion-<repo>-<github.event.pull_request.number || github.ref>-<perfil>`
   (revisión de la ronda 1: por repo y perfil, un tercer PR cancelaba la
   ejecución pendiente de otro y lo dejaba en rojo sin culpa). Ahora solo se
   encolan las ejecuciones del mismo PR; con `cancel-in-progress: false` GitHub
   cancela la pendiente más antigua del grupo cuando llega una tercera, que es
   un commit ya superado del mismo PR. PR distintos corren en paralelo: un
   runner propio ejecuta un job a la vez y los serializa por sí mismo. F5-5
   fija **una instancia por máquina y repo** (sección «Runner propio»), así
   que dos jobs del mismo repo nunca comparten `target/` en la misma máquina.
   Comprobado solo por la prueba `la_concurrencia_es_por_pr_o_ref`; no se
   reprodujo en el E2E.
7. **Segunda acción en la lista blanca** (F5-4, D3): F1-8 y esta guía decían
   que la única acción admitida es `actions/checkout`. Se admite también
   `actions/upload-artifact`, fijada por SHA y solo en clase ejecuta (sin
   escritura ni secretos), porque el artefacto es el canal que F1-8 fija para
   el revisor y existe en las dos plataformas (RR-15).
8. **E2E del informe pendiente** (F5-4, D6): TASK-044 no hace push. Se valida
   en local (pruebas de xtask y tc2) y la plantilla con la API `ci/lint` de
   GitLab (contenido en línea, entradas sustituidas por valores de ejemplo y
   sin la cabecera `spec`). La ejecución real en GitHub (artefacto presente,
   `informe=` en el resumen) y la descarga del artefacto de otro run con
   `actions: read` se hacen en TASK-049, que consume el informe.
9. **"No repite tests"** (criterio 1): solo es comprobable con el revisor de
   F6-3. TASK-049 debe usar `seccion_revisor` y demostrarlo.
10. **ToolsaWeb y OpenDockTrading** no producen el informe hasta migrar (F9):
    su `.estado/verificar-ultima.json` tiene otra forma. Hasta entonces solo
    existe el resumen del check.
11. **Versión del workflow en la clave** (F1-7, decisión 2; plan de F5-2):
    `github.job_workflow_sha` llega vacío dentro del reutilizable (E2E). La
    versión es el `sha` de `referenced_workflows` de la propia ejecución
    (`GET actions/runs/{run_id}`, `actions: read`); si no hay exactamente uno,
    no se busca y se verifica entero.
12. **`filter=all` en vez de `filter=latest`** (F1-7, decisión 3): «Re-run all
    jobs» saca de `latest` (y de la lista de la suite) el check-run del intento
    anterior, así que con `latest` una re-ejecución no encontraba nada (E2E,
    run 44607389843 intento 2). Se leen todos los del commit, el más reciente
    primero. Una línea en verde de un intento anterior sigue siendo válida: el
    resultado es función del árbol, la clave y la huella local.
13. **Origen del check-run por su check suite, no por `details_url`** (F1-8,
    decisión 7; D3 del plan): GitHub ignora el `details_url` de un check-run
    creado con `GITHUB_TOKEN` (queda en `…/runs/<id del check-run>`). `buscar`
    enlaza el check-run con su ejecución por `check_suite.id`
    (`GET actions/runs?check_suite_id=`) y exige ahí este workflow en la misma
    versión.
14. **Huella local volátil en los runners de GitHub**: con `huella-local: uname
    -sr` dos patas de la misma ejecución, y la misma pata entre intentos,
    sacaron `local` distintos (imágenes con distinto kernel; runs 44607057098
    y 44607389843): sin acierto. La huella local debe nombrar solo lo que de
    verdad cambia el resultado (versiones de herramientas fijadas), no el
    sistema del runner. Se documenta para F1-11.
15. **PR del E2E a una rama base con el llamador, no a `main`**: la regla de
    rutas rechaza toda línea si el PR toca `.github/workflows/**`, y el
    llamador del E2E vive ahí; contra `main` (sin llamador) nunca habría
    acierto. La base `cache-f5-2-base` lleva el mismo llamador que el PR.
16. **Lint más estricto** (F1-8, decisión 1): un job que escribe ya no puede
    hacer checkout (antes solo se prohibían `ref:` y `repository:`), y solo se
    admiten lecturas de `contents`, `checks` y `actions`. El lector YAML
    admite además `- |` (bloque literal como elemento de lista: el
    `after_script` de GitLab).
17. **GitLab: línea en un fichero del artefacto del informe** (D2): un job de
    GitLab tiene un solo archivo de artefactos; la línea va en
    `.tc2/cache-verificacion-<perfil>.txt` dentro de
    `informe-verificacion-<perfil>`, sin la versión del workflow en la clave.
    Sin pipeline real (D2): solo el lint de la plantilla.

## Prueba E2E (2026-11-19, workflow y plantilla en `790b72d`)

Repos de prueba públicos, ramas `ci-f5-1` borradas al terminar y PR cerrado
sin mergear.

| Caso | Dónde | Resultado |
|---|---|---|
| Stub con el contrato de ToolsaWeb (`scripts\verificar.bat`, `shell: cmd`, `windows-latest`, `argumentos-completa: ""`), PR a `main` | [run 44551304303](https://github.com/charliebk/taskcodetwo-e2e-github/actions/runs/44551304303) | verde; `RAMA_BASE=main`, perfil completa |
| Stub con la forma de OpenDockTrading (no lee argumentos), mismo PR | [run 44551304248](https://github.com/charliebk/taskcodetwo-e2e-github/actions/runs/44551304248) | verde; `argumentos recibidos: []` |
| ToolsaWeb, `workflow_dispatch` con `perfiles: completa pesada` | [run 44551659290](https://github.com/charliebk/taskcodetwo-e2e-github/actions/runs/44551659290) | verde; `--pesada` en su job, completa sin argumentos |
| ToolsaWeb forzado a fallar (commit con `fallar.txt`) | [run 44552422966](https://github.com/charliebk/taskcodetwo-e2e-github/actions/runs/44552422966) | rojo: `completa` sale con 1 y el check `verificacion` en `failure` |
| Plantilla de GitLab con `include: remote` por SHA (primer intento) | [pipeline 3089899840](https://gitlab.com/taskcodetwo-e2e/taskcodetwo-e2e-gitlab/-/pipelines/3089899840) | no ejecutado: «The pipeline failed due to the user not being verified» (0 jobs, 0 minutos) |
| Plantilla de GitLab en `944a974`, push de rama con `perfiles: rapida` | [pipeline 3090338668](https://gitlab.com/taskcodetwo-e2e/taskcodetwo-e2e-gitlab/-/pipelines/3090338668) | verde: `verificacion-rapida` llama al stub con `--perfil rapida` (15 s); la regla lee `TC2_PERFILES` global |
| La misma, stub forzado a fallar (`fallar.txt`) | [pipeline 3090341084](https://gitlab.com/taskcodetwo-e2e/taskcodetwo-e2e-gitlab/-/pipelines/3090341084) | rojo: `exit code 1` |

`gh api --method GET repos/…/commits/<head>/check-runs?check_name=verificacion&app_id=15368`
devuelve el check-run `verificacion` de la App `github-actions` (`app_id`
15368) en el **head** del PR (el workflow corre sobre el merge ref), con
`output.summary` escrito; los jobs salen como `verificacion / buscar`,
`verificacion / completa` y `verificacion / verificacion`.

Suposiciones del plan:

- **Nombre del check de un reutilizable**: confirmado `<job que llama> /
  <job llamado>`; por eso el publicador crea además el check-run
  `verificacion` con ese nombre exacto (D2).
- **Shell por entrada**: `defaults.run.shell: ${{ inputs.shell }}` funciona
  (el paso corrió con `cmd.EXE /D /E:ON /V:OFF /S /C "CALL "{0}""`).
- **`spec:inputs` con `include: remote` en GitLab.com Free**: verificado con
  la API de lint (`POST /projects/:id/ci/lint`, `include_merged_yaml`): las
  entradas se interpolan, los cuatro jobs salen con su `script` y `tags: []`
  es válido. `TC2_PERFILES` era una variable del job y `rules:if` no ve las
  `variables:` de los jobs (revisión de la ronda 1): ahora es una variable
  **global** de la plantilla, que sí se evalúa en las reglas; el lint de
  `workflows.rs` rechaza un `rules:if` que lea una variable de job. La
  regla casa en un pipeline real: verificado el 2026-11-19 con un push de
  rama y `perfiles: rapida` (pipelines 3090338668 en verde y 3090341084 en
  rojo), tras verificar la persona la identidad de la cuenta en GitLab.com
  (requisito de sus runners compartidos).
- **`charliebk/TaskCodeTwo` público**: sí; el reutilizable se resolvió desde
  otro repo por SHA de una rama que no es `develop` ni `main`.

## Prueba E2E de la caché por árbol (2026-11-19, workflow en `132a9dd`)

TASK-042, en `charliebk/taskcodetwo-e2e-github`: llamador único con un stub
Linux (`sh scripts/verificar.sh`), `perfiles: completa pesada` (dos patas de
la matriz) y `huella-local: echo e2e-f5-2`. PR #13 de `cache-f5-2` a
`cache-f5-2-base` (rama base con el mismo llamador: ver divergencia 15),
cerrado sin mergear y ramas borradas al terminar.

| Caso | Ejecución | Paso del comando (completa / pesada) | Línea del perfil |
|---|---|---|---|
| (a) PR nuevo | [44607911390, intento 1](https://github.com/charliebk/taskcodetwo-e2e-github/actions/runs/44607911390/attempts/1) | `success` / `success` | `ejecutada` |
| (a) «Re-run all jobs» (`gh run rerun`) | [44607911390, intento 2](https://github.com/charliebk/taskcodetwo-e2e-github/actions/runs/44607911390/attempts/2) | `skipped` / `skipped` | `reutilizada de 10d5e76` (el head) |
| (b) commit vacío | [44608246992](https://github.com/charliebk/taskcodetwo-e2e-github/actions/runs/44608246992) | `skipped` / `skipped` | `reutilizada de 10d5e76` (el head anterior, abuelo de V) |
| (c) cambiar `datos.txt` | [44608436324](https://github.com/charliebk/taskcodetwo-e2e-github/actions/runs/44608436324) | `success` / `success` | `ejecutada` |
| (c) tocar `.github/workflows/verificacion.yml` | [44608660673](https://github.com/charliebk/taskcodetwo-e2e-github/actions/runs/44608660673) | `success` / `success` | `ejecutada` («Regla de rutas») |
| (c) commit vacío tras tocar el workflow | [44608801152](https://github.com/charliebk/taskcodetwo-e2e-github/actions/runs/44608801152) | `success` / `success` | `ejecutada`: mismo árbol, pero la regla de rutas rechaza toda línea |

Medición (criterio 1) con
`medicion/medir.sh` de la carpeta de TASK-042 en `tareas/` (solo `gh api --method GET`;
salida en `salida.txt` junto a él): por árbol y perfil, el paso del comando no
salió `skipped` **una vez** en los árboles `5b30777` y `91a0f2f`; dos veces en
`555e6b4`, el árbol cuyo PR toca `.github/workflows/**`, donde la regla de
rutas obliga a verificar entero.

Suposiciones del plan:

- **`github.job_workflow_sha` en el reutilizable**: **falsa**, llega vacío
  (run 44606759639, `version=` vacío). La versión se lee de
  `referenced_workflows` de la propia ejecución (divergencia 11).
- **Salidas con nombre fijo por perfil en la matriz**: confirmado; con dos
  patas, el publicador recibió `local-completa` y `local-pesada` sin pisarse.
- **Merge ref con `fetch-depth: 3`**: confirmado; da V, sus dos padres (la
  base primero) y los abuelos, y el diff frente a `HEAD^1` es el del PR.
- **Una re-ejecución deja el check anterior como `latest` en el head**:
  **falsa**: «Re-run all jobs» saca el check-run del intento anterior de la
  suite y de `filter=latest` (sigue existiendo por id y con `filter=all`)
  (divergencia 12).
- **`bash` en el runner del perfil**: solo se probó Linux; en Windows el paso
  «cache por arbol» usa el `bash` de Git for Windows (no verificado aquí).
- **`GET commits/{V}` sobre el merge ref desde el publicador**: funciona con
  `contents: read`; el árbol coincidió con el de `buscar` en todas las
  ejecuciones.

# Verificación reutilizable en CI (F5-1)

TASK-041. Dos piezas que **solo llaman al comando de verificación del
proyecto** (ADR [F1-7](../adr/F1-7-verificacion-ci-perfiles-cache-runners.md),
decisión 1; RR-15): no conocen sus pasos, sus grupos ni sus herramientas.

| Plataforma | Pieza | Cómo se usa |
|---|---|---|
| GitHub Actions | [`.github/workflows/verificacion.yml`](../../.github/workflows/verificacion.yml) (`on: workflow_call`) | un job del proyecto con `uses: charliebk/TaskCodeTwo/.github/workflows/verificacion.yml@<sha>` |
| GitLab CI | [`plantillas/gitlab/verificacion.gitlab-ci.yml`](../../plantillas/gitlab/verificacion.gitlab-ci.yml) (`spec:inputs`) | `include: remote` a la URL raw de la plantilla fijada por SHA |

Fuera de F5-1: el runner propio (F5-5), `carril` y `revision-ia` (F6), y la
definición de CI en otro proyecto de GitLab (F2-9). El informe JSON para el
revisor (F5-4) y la caché por árbol (F5-2) se describen más abajo.

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
| PR / MR a cualquier rama | completa | completa (`verificacion-completa`) |
| PR / MR de `develop` a `main` (release) | completa y pesada | completa y pesada |
| `merge_group` | completa | — |
| `workflow_dispatch` | completa | — |
| `push` de rama | nada | nada |

El comando recibe la base del PR en `RAMA_BASE` y el perfil en `TC2_PERFIL`.
Resultado: código de salida (0 verde, otro rojo).

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
SHA, con `persist-credentials: false` y nunca en un job que escribe (F5-2:
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
  `GET commits/{c}/check-runs?check_name=verificacion&app_id=15368&filter=latest`
  y solo cuenta un check-run en verde cuya ejecución (`details_url` →
  `GET actions/runs/{id}`) tenga en `referenced_workflows` este workflow
  (`charliebk/TaskCodeTwo/.github/workflows/verificacion.yml@…`) con el mismo
  `sha` que `github.job_workflow_sha`. De su resumen toma las líneas `arbol=`
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
(`perfiles= resultado= informe= ejecucion=`):

```
arbol=<árbol de V> perfil=<p> runner=<r> clave=<k> local=<h> origen=<sha> ejecutada
arbol=<árbol de V> perfil=<p> runner=<r> clave=<k> local=<h> origen=<sha> reutilizada de <commit>
```

- `runner`: el `runner-<perfil>` declarado, sin comillas ni espacios (una
  lista de etiquetas, unida por comas).
- `clave = sha256(perfil|runner|huella-arbol|github.job_workflow_sha)`, con
  `huella-arbol = sha256(git ls-tree -r HEAD -- <rutas>)` (o de nada sin rutas).
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
   apareció en la lista de jobs del otro). Con `filter=latest` solo cuenta el
   último: el proyecto declara un único llamador.
6. **Concurrencia por PR (o ref) y perfil**: el grupo es
   `verificacion-<repo>-<github.event.pull_request.number || github.ref>-<perfil>`
   (revisión de la ronda 1: por repo y perfil, un tercer PR cancelaba la
   ejecución pendiente de otro y lo dejaba en rojo sin culpa). Ahora solo se
   encolan las ejecuciones del mismo PR; con `cancel-in-progress: false` GitHub
   cancela la pendiente más antigua del grupo cuando llega una tercera, que es
   un commit ya superado del mismo PR. PR distintos corren en paralelo: un
   runner propio con un único ejecutor los serializa por sí mismo, y varios
   ejecutores en la misma máquina (que compartirían `target/`) quedan para
   F5-5. Comprobado solo por la prueba `la_concurrencia_es_por_pr_o_ref`; no se
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

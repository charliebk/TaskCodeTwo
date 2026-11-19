# Verificación reutilizable en CI (F5-1)

TASK-041. Dos piezas que **solo llaman al comando de verificación del
proyecto** (ADR [F1-7](../adr/F1-7-verificacion-ci-perfiles-cache-runners.md),
decisión 1; RR-15): no conocen sus pasos, sus grupos ni sus herramientas.

| Plataforma | Pieza | Cómo se usa |
|---|---|---|
| GitHub Actions | [`.github/workflows/verificacion.yml`](../../.github/workflows/verificacion.yml) (`on: workflow_call`) | un job del proyecto con `uses: charliebk/TaskCodeTwo/.github/workflows/verificacion.yml@<sha>` |
| GitLab CI | [`plantillas/gitlab/verificacion.gitlab-ci.yml`](../../plantillas/gitlab/verificacion.gitlab-ci.yml) (`spec:inputs`) | `include: remote` a la URL raw de la plantilla fijada por SHA |

Fuera de F5-1: la caché por árbol y la línea `arbol=` (F5-2), el informe JSON
para el revisor (F5-4), el runner propio (F5-5), `carril` y `revision-ia`
(F6), y la definición de CI en otro proyecto de GitLab (F2-9).

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
| GitHub: `runner-<perfil>` | `'"ubuntu-latest"'` | `runs-on` del perfil, en JSON: una cadena o una lista de etiquetas (`'["self-hosted","windows","toolsaweb"]'`) |
| GitHub: `shell` | `bash` | Shell en el que corre el comando (`cmd` para un `.bat`) |
| GitLab: `tags-<perfil>` | `[]` | Etiquetas del runner del perfil |
| GitLab: `stage` | `test` | Etapa de los jobs |

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

El job que llama tiene que conceder `checks: write` (un workflow reutilizable
no puede tener más permisos que su llamador). En un PR desde un fork el
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
`write-all`), la única acción admitida es `actions/checkout` fijada por SHA y
con `persist-credentials: false` (sin `ref:` ni `repository:` en un job que
escribe), ningún `secrets: inherit`, ningún secreto en un job sin escritura y
ningún `github.event.*`, `github.head_ref` ni `toJSON(github…)` en `run:`. Sobre
la plantilla: `spec:inputs`, ningún `secrets:` y ninguna variable que escribe
el autor (`CI_COMMIT_MESSAGE`, `CI_MERGE_REQUEST_TITLE`…) en `script:`. Cada
regla tiene su mutación negativa en las pruebas.

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

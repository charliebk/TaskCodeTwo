# Revisor IA en CI (F6-3)

TASK-049. El check `revision-ia` lo publica la **App del revisor** desde un
workflow reutilizable que corre cuando termina `verificacion` en un PR
([`.github/workflows/revision-ia.yml`](../../.github/workflows/revision-ia.yml)).
Qué revisoras se lanzan y con qué modelo no lo decide el modelo: lo decide
`tc2 revisoras` con las tablas del ADR
[F1-15](../adr/F1-15-seleccion-de-skills-agentes-y-modelo.md) (adenda F6-3)
y el catálogo de la base. Clases de jobs y secretos: ADR
[F1-8](../adr/F1-8-seguridad-modelo-de-amenazas.md) (adenda F6-3).

## Qué hace

| Job | Clase (F1-8) | Permisos | Qué hace |
|---|---|---|---|
| `preparar` | ejecuta | `contents: read`, `actions: read`; sin secretos | saca la base (`base`, por defecto `develop`), el plugin taskcode a un SHA fijado y el código de `tc2` a la versión del workflow, y compila `tc2`; trae el head del PR **como dato** (`git fetch` de su SHA, sin checkout, autenticado con el `GITHUB_TOKEN` del job; ver «Credenciales del fetch»); baja `informe-verificacion-<perfil>` del run de `verificacion` (tope de 1 MiB del zip y 64 KiB del JSON); saca las rutas con `git -c core.quotepath=false diff --name-only -z` y corre `tc2 revisoras --rutas <fichero> --nul` (rutas literales separadas por NUL, también las no ASCII), y escribe un prompt por revisora; sube el artefacto `revision-ia-prompts` |
| `publicar` (`revision-ia`) | privilegio, por el entorno `revisor` | `actions: read`; sin checkout | llama a la API de Anthropic una vez por revisora y publica el check-run `revision-ia` con la App del revisor, conclusión `neutral` |

El prompt de cada revisora lleva su `SKILL.md`, la línea de la selección,
la sección del informe de verificación (`seccion_revisor` de F5-4, con la
regla de no relanzar lo que ya pasó en verde) y el diff de **sus** rutas con
un tope (`tope-diff`, 120 000 bytes por defecto; si se corta, se le dice).
El diff va entre cercas de acentos graves más largas que la secuencia más
larga del propio diff (mínimo tres): ninguna línea del diff puede cerrar el
bloque. La revisora de dominio recibe las rutas que casan con sus patrones; la
genérica, las sueltas (S08) o todas (S09). Nadie ejecuta la suite: el
modelo no tiene herramientas.

El resumen del check dice `revisoras= agentes= modelo= modelo-api= reglas=
avisos= carril= complejidad= informe=` y una línea por revisora (`ok`,
`negativa`, `error-http-<n>` o `sin-clave-del-modelo`); el texto, los
informes del modelo. Si `publicar` no encuentra o no puede leer el artefacto
`revision-ia-prompts`, lo avisa en el log y publica igual el check `neutral`
con `preparar=<resultado> sin-prompts`. Un fallo de la App (no instalada,
id o clave que no valen, sin token de instalación) se dice con un `::error::`
explícito y no publica.

### Credenciales del fetch

Los checkouts llevan `persist-credentials: false` (lo exige el lint). El
paso «Head del PR como dato» necesita credencial para `git fetch` del SHA
del head y de `taskcodetwo/tareas` en un repo privado: usa el
`GITHUB_TOKEN` del propio job (`contents: read`, solo lectura), que llega
por `env:` y se pasa a git como cabecera `AUTHORIZATION: basic` con
`GIT_CONFIG_COUNT`/`GIT_CONFIG_KEY_0`/`GIT_CONFIG_VALUE_0` exportadas solo
en ese paso. Así no aparece en la línea de órdenes, no se escribe en
`.git/config` (no queda para pasos siguientes) y su base64 se enmascara con
`::add-mask::` antes de usarse.

## Cómo se invoca el modelo

`curl` a la Messages API (`POST https://api.anthropic.com/v1/messages`) en
el job `publicar`, con `x-api-key: $ANTHROPIC_API_KEY` y
`anthropic-version: 2026-10-21` (decisión D1 de TASK-049: clave de API como
secreto del entorno, no Claude Code en el runner). Cuerpo:
`{model, max_tokens: 16000, output_config: {effort: "high"}, messages}`,
sin `temperature` (la versión 2026-10-21 la rechaza). Con Sonnet y Opus va
además `fallbacks: "default"` con `anthropic-beta:
server-side-fallback-2026-07-01`, por si el clasificador del modelo se
niega; Haiku no tiene fallback del servidor. Se comprueba `stop_reason`
antes de leer el texto (`refusal` sale como `negativa`).

Modelo: la columna «Revisión» de `tabla-modelo` (carril efectivo y
complejidad de la tarea) y S15. Ids de la API: `haiku` →
`claude-haiku-5-5`, `sonnet` → `claude-sonnet-5-5`, `opus` →
`claude-opus-5-5`. La tarea sale de la rama del PR (`feature/task-NNN-...`)
leída en `taskcodetwo/tareas`; sin tarea, complejidad `media` con aviso.

## Lo que hay que crear (la persona)

1. **La App del revisor** (D2): permisos `checks: write`, `contents: read`,
   `pull_requests: read`, `metadata: read`; instalada en el repo. Su id va a
   `plataforma.revisor_id` de `.taskcode/config.yml` en la base, para que
   `tc2 review` lea su check (F6-2).
2. **El entorno `revisor`** en el repo, con la regla de despliegue limitada
   a la rama por defecto (`workflow_run` corre ahí) y tres secretos de
   entorno:
   - `TC2_APP_REVISOR_ID`: id numérico de la App;
   - `TC2_APP_REVISOR_CLAVE`: su clave privada PEM completa;
   - `ANTHROPIC_API_KEY`: la clave de la API de Anthropic.
   Las claves no pasan por el chat ni por ningún fichero del repo.
3. **El llamador** en la rama por defecto. No pasa `secrets:` ni usa
   `secrets: inherit` (F1-8, R9): el reutilizable declara los tres secretos
   en `on.workflow_call.secrets` como opcionales y su job `publicar`, con
   `environment: revisor`, los toma del entorno del repo llamador. Sin esa
   declaración el contexto `secrets` del reutilizable llega vacío aunque el
   job tenga entorno (comprobado en el E2E de TASK-049):

```yaml
name: revision-ia
on:
  workflow_run:
    workflows: [verificacion]
    types: [completed]
permissions: {}
jobs:
  revision-ia:
    # Los PR de forks llegan sin pull_requests: no se revisan en F6-3.
    if: github.event.workflow_run.event == 'pull_request' && github.event.workflow_run.pull_requests[0] != null
    permissions:
      contents: read
      actions: read
    uses: charliebk/TaskCodeTwo/.github/workflows/revision-ia.yml@<sha>
    with:
      verificacion-run: ${{ github.event.workflow_run.id }}
      head-sha: ${{ github.event.workflow_run.head_sha }}
      pr: ${{ github.event.workflow_run.pull_requests[0].number }}
      rama: ${{ github.event.workflow_run.head_branch }}
      base: ${{ github.event.workflow_run.pull_requests[0].base.ref }}
```

`workflows: [verificacion]` es el `name:` del workflow del proyecto que
llama a `verificacion.yml`. `<sha>` es el commit de TaskCodeTwo con este
workflow (40 hexadecimales, nunca una rama); `tc2` se compila de ese mismo
commit (o del de la entrada `tc2-sha`).

**Sin secretos sale limpio**: sin `TC2_APP_REVISOR_ID` o
`TC2_APP_REVISOR_CLAVE`, `publicar` termina en verde con un aviso y no
publica nada. Con la App y sin `ANTHROPIC_API_KEY`, publica el check
`neutral` con la selección y `sin-clave-del-modelo` por revisora. Este repo
no activa aún el llamador: hasta que existan la App y el entorno, solo
daría ejecuciones sin check.

## Entradas

| Entrada | Por defecto | Qué es |
|---|---|---|
| `verificacion-run` | (obligatoria) | id del run de `verificacion` |
| `head-sha` | (obligatoria) | SHA verificado; no se exige que siga siendo el head del PR (seguimiento de TASK-044) |
| `pr` | (obligatoria) | número del PR |
| `rama` | `""` | rama del PR; solo para sacar `TASK-NNN` con una expresión fija |
| `base` | `develop` | rama base: configuración y skills del proyecto |
| `perfil` | `completa` | perfil del informe que se cita |
| `plugin-repo`, `plugin-sha`, `plugin-ruta`, `plugin-nombre` | `charliebk/TaskCode`, `b044e75...` (v0.16.2), `taskcode-marketplace/plugins/taskcode-plugin`, `taskcode-plugin` | el plugin con las revisoras |
| `tc2-sha` | `""` | commit de TaskCodeTwo para compilar `tc2`; vacío, el del workflow |
| `tope-diff` | `120000` | bytes de diff por revisora |

## Probarlo de punta a punta (repo E2E)

`workflow_run` toma el llamador de la rama por defecto, así que el E2E se
hace en el repo E2E (rama por defecto `main`), no en TaskCodeTwo.

1. Crear la App del revisor e instalarla en el repo E2E; crear el entorno
   `revisor` con los tres secretos (arriba).
2. En `main` del repo E2E: el workflow `verificacion` que llama a
   `verificacion.yml@<sha>` (ya existe desde F5-1/F5-2) y el llamador de
   arriba con `revision-ia.yml@<sha>`, ambos con el SHA del commit de
   TaskCodeTwo que trae este workflow (empujado a GitHub).
3. Abrir un PR contra `develop` desde una rama `feature/task-NNN-...` que
   toque un `.rs` y un `.vue` (y nada más).
4. Esperar a `verificacion` y después a `revision-ia` (pestaña Actions).
   Comprobar:
   - el check-run `revision-ia` del head lo publica la App del revisor
     (`app.id` = su id), conclusión `neutral`;
   - el resumen dice `revisoras=taskcode-plugin:angular-vue-reviewer,taskcode-plugin:rust-wasm-reviewer`,
     `reglas=` con `S08` y `S12`, el modelo y `informe=verde` (o `rojo`);
   - el artefacto `revision-ia-prompts` tiene un prompt por revisora con la
     sección `## Informe de verificacion (F5-4)`;
   - el log de `preparar` no ejecuta la suite ni nada del PR (solo `cargo
     build -p tc2` sobre el código de TaskCodeTwo).
5. Repetir sin secretos de la App (o en un repo sin el entorno): `publicar`
   en verde con el aviso y sin check.
6. Cerrar el PR y borrar la rama.

## Límites conocidos

- `neutral` no bloquea hasta F6-5; lo contiene la aprobación de la persona.
- Sin el detector de dominios (F4-6) ni rondas (F6-6): S10, S11 y S13 no se
  aplican.
- El diff y el informe son datos del autor del PR que llegan al modelo; la
  defensa contra inyección de instrucciones es de F6-7.
- Coste: una llamada por revisora y ronda (2 o 3 en un PR Rust + Vue).

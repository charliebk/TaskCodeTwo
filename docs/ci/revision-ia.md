# Revisor IA en CI (F6-3, F6-4, F6-5, F6-6)

TASK-049, TASK-050, TASK-051 y TASK-052. El check `revision-ia` lo publica la **App del revisor** desde un
workflow reutilizable que corre cuando termina `verificacion` en un PR
([`.github/workflows/revision-ia.yml`](../../.github/workflows/revision-ia.yml)).
Qué revisoras se lanzan y con qué modelo no lo decide el modelo: lo decide
`tc2 revisoras` con las tablas del ADR
[F1-15](../adr/F1-15-seleccion-de-skills-agentes-y-modelo.md) (adenda F6-3)
y el catálogo de la base. Qué hallazgos cuentan tampoco: `tc2 hallazgos
validar` rechaza todo hallazgo sin evidencia de su tipo anclada a una línea
del diff (F6-4, «Hallazgos con evidencia»). Ni el color del check: lo saca
`tc2 hallazgos veredicto` (F6-5, «Veredicto del check»). Ni qué se mira en
cada ronda: `tc2 hallazgos ronda` (F6-6, «Rondas»). Clases de jobs y secretos: ADR
[F1-8](../adr/F1-8-seguridad-modelo-de-amenazas.md) (adendas F6-3 y F6-4).

## Qué hace

| Job | Clase (F1-8) | Permisos | Qué hace |
|---|---|---|---|
| `preparar` | ejecuta | `contents: read`, `actions: read`; sin secretos | saca la base (`base`, por defecto `develop`), el plugin taskcode a un SHA fijado y el código de `tc2` a la versión del workflow, y compila `tc2`; trae el head del PR **como dato** (`git fetch` de su SHA, sin checkout, autenticado con el `GITHUB_TOKEN` del job; ver «Credenciales del fetch»); baja `informe-verificacion-<perfil>` del run de `verificacion`, o del run que ejecutó el perfil si ese run lo reutilizó de la caché (ver «Informe de una verificación reutilizada»), con tope de 1 MiB del zip y 64 KiB del JSON; saca las rutas con `git -c core.quotepath=false diff --name-only -z` y corre `tc2 revisoras --rutas <fichero> --nul` (rutas literales separadas por NUL, también las no ASCII), y escribe un prompt por revisora; sube el artefacto `revision-ia-prompts` (prompts, el diff recortado que vio cada revisora en `diffs/<i>.txt` y el informe) |
| `consultar` | privilegio, por el entorno `revisor` | `actions: read`; sin checkout; solo `ANTHROPIC_API_KEY` | llama a la API de Anthropic una vez por revisora y pasa cada respuesta a `validar` por `outputs` (ver «Hallazgos con evidencia») |
| `validar` | ejecuta | `contents: read`, `actions: read`; sin secretos | compila `tc2` de la misma versión que `preparar` (no se pasan binarios entre jobs), pasa cada respuesta por `tc2 hallazgos validar` contra el diff de su revisora, las junta con `tc2 hallazgos veredicto` (`check.json` y lotes `anotaciones-<k>.json`) y sube `revision-ia-hallazgos` |
| `publicar` (`revision-ia`) | privilegio, por el entorno `revisor` | `actions: read`; sin checkout; solo la App del revisor | lee `revision-ia-hallazgos` como dato y publica tal cual su `check.json` como check-run `revision-ia` con la App del revisor (conclusión `success` o `failure`), y después cada lote de anotaciones con un `PATCH`; sin `check.json` legible, `failure` con veredicto `incompleta` |

El prompt de cada revisora lleva su `SKILL.md`, la línea de la selección,
la sección del informe de verificación (`seccion_revisor` de F5-4, con la
regla de no relanzar lo que ya pasó en verde) y el diff de **sus** rutas con
un tope (`tope-diff`, 120 000 bytes por defecto; si se corta, se le dice).
El diff va entre cercas de acentos graves más largas que la secuencia más
larga del propio diff (mínimo tres): ninguna línea del diff puede cerrar el
bloque. La revisora de dominio recibe las rutas que casan con sus patrones; la
genérica, las sueltas (S08) o todas (S09). Nadie ejecuta la suite: el
modelo no tiene herramientas. La sección «Lo que devuelves» del prompt es la
salida de `tc2 hallazgos instrucciones`, hecha con las mismas constantes que
el validador.

El resumen del check (F6-5) empieza por el veredicto, los recuentos y una
fila por revisora (ver «Veredicto del check»); después, como líneas de
texto plano y en este orden, la de la selección que deja `preparar`
(`revisoras= agentes= modelo= modelo-api= reglas= avisos= carril=
complejidad= informe=`, más `informe-de=run-<id>`, el run del que sale el
informe, cuando hay informe), la que añade `validar` con `preparar=success
consultar=<r>` (`preparar=success` va fijo: `validar` solo corre si
`preparar` terminó bien) y una línea por revisora: `<id>: <estado>; hallazgos ok revisora=<id>
aceptados=N rechazados=M motivos=<quien:motivo,...>`, con el estado de la
llamada (`ok(<stop_reason>) tokens=...`, `negativa`, `error-http-<n> ...`,
`sin-clave-del-modelo` o `sin-respuesta`; `cortada=<bytes>` si pasó de 64
KiB). El texto lleva, por revisora, las tablas de aceptados y rechazados y
después el informe del modelo. Si `publicar` no encuentra o no puede leer el
artefacto `revision-ia-hallazgos` (o su `check.json`), lo avisa en el log y
publica el check en `failure` con el título `revision-ia: incompleta (sin
veredicto de validar)` y `preparar= consultar= validar=` en el resumen. Si
la API rechaza el check con el veredicto de `validar` (por ejemplo, una
anotación fuera de sus límites), lo avisa en el log y publica en su lugar el
check en `failure`, sin anotaciones, con el título `revision-ia: incompleta
(la API rechazo el veredicto)`: el check existe y dice por qué. `tc2
hallazgos veredicto` ya evita los rechazos conocidos: un mensaje que queda
vacío pasa a `descripcion sin texto legible` y el `raw_details` se recorta
hasta caber en los 64 KB de la API. Un fallo de la App (no instalada, id o
clave que no valen, sin token de instalación) se dice con un `::error::`
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
el job `consultar`, con `x-api-key: $ANTHROPIC_API_KEY` y
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
3. **El llamador** en la rama por defecto, con `secrets: inherit`. Es la
   única excepción a la regla R9 de F1-8 (adenda F6-3): GitHub solo entrega
   los secretos del entorno `revisor` a los jobs `consultar` y `publicar` del
   reutilizable si el llamador hereda. Declararlos en `on.workflow_call.secrets` no basta:
   llegan vacíos (las dos cosas comprobadas en el E2E de TASK-049). El lint
   (perfil `llamador-revision`) solo admite `inherit` hacia
   `charliebk/TaskCodeTwo/.github/workflows/revision-ia.yml` fijado por SHA,
   y el del reutilizable impide usar secretos fuera de esos jobs (y el test
   del workflow, cada secreto fuera del suyo: la clave del modelo solo en
   `consultar`, la App solo en `publicar`). Ojo:
   `inherit` entrega al reutilizable todos los secretos de repositorio y de
   organización, no solo los del entorno, y ese lint corre en los tests de
   TaskCodeTwo, no en tu repo: revisa el llamador a mano y fija el SHA:

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
      # F6-6: preparar lee el check revision-ia de la ronda anterior.
      checks: read
    uses: charliebk/TaskCodeTwo/.github/workflows/revision-ia.yml@<sha>
    secrets: inherit
    with:
      verificacion-run: ${{ github.event.workflow_run.id }}
      head-sha: ${{ github.event.workflow_run.head_sha }}
      pr: ${{ github.event.workflow_run.pull_requests[0].number }}
      rama: ${{ github.event.workflow_run.head_branch }}
      base: ${{ github.event.workflow_run.pull_requests[0].base.ref }}
```

Sin `checks: read` en el llamador, el reutilizable no puede pedirlo (no gana
permisos que el llamador no da) y GitHub rechaza el run entero al validarlo,
antes de arrancar ningún job: el workflow de revisión sale en rojo con un
error de permisos («... is requesting 'checks: read', but is only allowed
'checks: none'») y **no se publica ningún check `revision-ia`**, ni siquiera
`incompleta` (falla en cerrado: si `revision-ia` es un check requerido, el PR
no se puede integrar). Un llamador que ya fijaba una versión anterior por SHA
tiene que añadir `checks: read` al subir el SHA. No verificado aún en un run
real.

`workflows: [verificacion]` es el `name:` del workflow del proyecto que
llama a `verificacion.yml`. `<sha>` es el commit de TaskCodeTwo con este
workflow (40 hexadecimales, nunca una rama); `tc2` se compila de ese mismo
commit (o del de la entrada `tc2-sha`).

**Sin secretos sale limpio**: sin `TC2_APP_REVISOR_ID` o
`TC2_APP_REVISOR_CLAVE`, `publicar` termina en verde con un aviso y no
publica nada. Con la App y sin `ANTHROPIC_API_KEY`, publica el check
en `failure` (veredicto `incompleta`: no hubo revisión) con la selección y, por revisora, `sin-clave-del-modelo` y su
respuesta vacía rechazada (`respuesta:sin-bloque`). Este repo
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
     (`app.id` = su id), conclusión `success` o `failure` (sin saldo en la
     cuenta del modelo, `failure` con título `revision-ia: incompleta ...`);
   - el resumen dice `revisoras=taskcode-plugin:angular-vue-reviewer,taskcode-plugin:rust-wasm-reviewer`,
     `reglas=` con `S08` y `S12`, el modelo y `informe=verde` (o `rojo`);
   - el artefacto `revision-ia-prompts` tiene un prompt por revisora con la
     sección `## Informe de verificacion (F5-4)` y las instrucciones de
     `tc2 hallazgos instrucciones`, y `diffs/<i>.txt` por revisora;
   - el resumen tiene, por revisora, la línea `hallazgos ok ... aceptados=N
     rechazados=M motivos=...`; sin saldo en la cuenta del modelo, el estado
     es `error-http-400 ...` y la respuesta vacía sale `respuesta:sin-bloque`
     (F6-4: también así se publica el check que lo dice);
   - el log de `validar` compila `tc2` y no ejecuta nada del PR, y el de
     `consultar` avisa si una respuesta se cortó;
   - el log de `preparar` no ejecuta la suite ni nada del PR (solo `cargo
     build -p tc2` sobre el código de TaskCodeTwo).
5. Repetir sin secretos de la App (o en un repo sin el entorno): `publicar`
   en verde con el aviso y sin check.
6. Cerrar el PR y borrar la rama.

## Informe de una verificación reutilizada

Con acierto de la caché por árbol (F5-2), el run de `verificacion` se salta
el comando y no sube `informe-verificacion-<perfil>`. Entonces `preparar`
(TASK-137, con los mismos permisos):

1. toma el job del perfil de ese run (`<perfil>` o `... / <perfil>`) y busca
   en su log la última línea `Perfil <perfil>: reutilizada de <sha> (origen
   <sha>, ...)` que escribe el paso `cache por arbol`;
2. lista los runs en verde del mismo workflow con `head_sha` igual al
   origen y cita el más reciente con el artefacto sin caducar (solo los runs
   que ejecutaron el perfil lo suben);
3. la sección del prompt nombra el artefacto con su run y su origen, y el
   resumen añade `informe-de=run-<id>`.

El código del PR podría imitar esa línea en el log (la huella local corre
antes en el mismo job), pero el informe ya lo escribe código del PR y solo se
cita como dato: lo más que consigue es citar otro informe del mismo workflow.

## Hallazgos con evidencia (F6-4)

Cada revisora termina su respuesta con **un único** bloque ```` ```json ````
con la lista de sus hallazgos (`[]` si no hay). Contrato:
[`crates/tc2/esquemas/hallazgos.schema.json`](../../crates/tc2/esquemas/hallazgos.schema.json),
versión 2 (`$defs/respuesta` es lo que va en el bloque). Cada hallazgo:

| Clave | Qué es |
|---|---|
| `id` | único en la lista; letras, dígitos, `.`, `_`, `-`, hasta 64 |
| `severidad` | `CRITICO`, `IMPORTANTE` o `MENOR` |
| `clase` | F6-5: `puerta-seguridad`, `contrato`, `correccion`, `fidelidad-evidencia` o `reproducibilidad` (lista cerrada de F1-15; si encaja en varias, la primera); si falta o no es de la lista, `hallazgo-invalido` |
| `fichero` | ruta de la cabecera `+++ b/<ruta>` del diff, sin `b/` |
| `linea` | línea del **lado nuevo** del diff que vio esa revisora (contexto o añadida) |
| `descripcion` | qué falla y por qué |
| `evidencia` | obligatoria en **todos**, también MENOR (decisión de la persona) |

Evidencia mínima por tipo (`tipo` y sus campos, llenos, ninguno de otro tipo):

| `tipo` | Campos |
|---|---|
| `reproduccion` | `pasos` (lista no vacía) y `fallo_esperado`; o `paso_rojo`, un paso en rojo del informe F5-4 |
| `mutante` | `linea_original` (igual al texto de esa línea en el diff, sin espacios a los lados), `mutacion` (distinta) y `test` (el que debería detectarla) |
| `borde` | `entrada`, `esperado` y `obtenido`, con `esperado` distinto de `obtenido` |

Motivos de rechazo (ids estables, en el orden en que se comprueban):
`sin-bloque`, `bloque-repetido`, `json-invalido` (no se cierra, no es JSON o
no es una lista), `fuera-de-tope` (más de 100 hallazgos),
`hallazgo-invalido`, `id-repetido`, `sin-evidencia`, `evidencia-incompleta`,
`linea-fuera-del-diff`, `mutante-no-coincide`, `mutante-sin-cambio`,
`borde-sin-diferencia` y `paso-rojo-no-esta`. Que haya rechazos no es un
fallo de la orden: sale `ok`, y qué hallazgos ponen el check en rojo lo
decide F6-5 (abajo).

```
tc2 hallazgos validar --respuesta <md> --diff <txt> --revisora <id>
    [--informe <informe.json>] [--salida <hallazgos.json>] [--resumen <seccion.md>]
tc2 hallazgos instrucciones
```

La evidencia **se declara y se valida, no se ejecuta**: la ejecución
(decisión 3 de F1-8) pasa a F6-6 (adenda F6-4). Lo contienen el anclaje al
diff que vio la revisora (lo que el tope dejó fuera se rechaza), que
`linea_original` coincida con el texto de esa línea, la aprobación de la
persona y F6-6.

Transporte: `consultar` (privilegio) no puede subir artefactos (lint), así
que cada respuesta va a `validar` por `outputs`, en base64 y en un hueco por
revisora (`r0`..`r5`, con su estado en `e0`..`e5`). Con 64 KiB de texto por
respuesta, cada hueco ocupa unos 87 KB: seis caben en el MiB de `outputs`
de un job y cada uno en los 128 KiB de una variable de entorno. Una séptima
revisora saldría `sin-respuesta` con un aviso (hoy la selección da como
mucho tres de dominio o la genérica). En `validar` las respuestas entran por
`env:`, nunca interpoladas en `run:` (lo comprueba el test del workflow).

## Veredicto del check (F6-5)

`tc2 hallazgos veredicto --salida <dir> [--hallazgos <salida-de-validar.json>]... [--extra <lineas.txt>]`
junta las salidas de `validar` de todas las revisoras (una que falta, no se
lee o no es de la versión 2 cuenta como revisora sin respuesta válida) y
decide, sin GitHub:

| Veredicto | Cuándo | Conclusión |
|---|---|---|
| `incompleta` | no hay revisoras, o alguna no dejó JSON o su respuesta entera se rechazó (`sin-bloque`, `json-invalido`...) | `failure` |
| `rojo` | algún CRITICO o IMPORTANTE **abierto** (nuevo o de una ronda anterior sin cierre válido) | `failure` |
| `tope` | pasado el tope de rondas con un CRITICO o IMPORTANTE abierto (F6-6) | `failure` |
| `verde` | el resto (solo MENOR, o nada) | `success` |

Un CRITICO rechazado (sin evidencia, fuera del diff...) no pone el check en
rojo. Nunca sale `neutral`: GitHub da por cumplido un check obligatorio en
`neutral`, y una revisión que no ocurrió (sin saldo, o una respuesta sin
bloque por una inyección) pasaría la puerta (decisión 1 de TASK-051, adenda
F6-5 de F1-15).

Escribe en `<dir>`:

- `check.json` (`$defs/veredicto`): `conclusion`, `output.title`
  (`revision-ia: <veredicto> (ronda=R criticos=N importantes=M menores=K)`,
  con los abiertos; `tc2` sigue leyendo el título v2 de F6-5, sin `ronda=`),
  `output.summary` (veredicto, recuentos, tabla por revisora y las líneas de
  `--extra` por `plano`) y las primeras 50 anotaciones;
- `anotaciones-<k>.json` (`$defs/lote`), el cuerpo de cada `PATCH` con las
  50 siguientes (la API admite 50 por petición).

Cada aceptado es una **anotación** del check (se ve en «Files changed»; no
hacen falta comentarios de PR ni permisos nuevos): `path` y `start_line` del
hallazgo, nivel `failure` para CRITICO e IMPORTANTE, `warning` para MENOR y
`notice` para los cerrados en esa ronda (F6-6), título `<id> <SEVERIDAD>
<clase> (<revisora>)` y mensaje por `plano`, y en `raw_details` el aceptado
con su revisora, su ronda y su estado en JSON (`$defs/aceptado-de`; uno de
F6-5 sin ronda ni estado se lee como ronda 1 y abierto). Línea: `hallazgos
ok veredicto=<v> conclusion=<c> criticos=N importantes=M menores=K
revisoras=R incompletas=I lotes=L ronda=R modo=<m> nuevos=N persisten=P
cerrados=C`; un rojo no es un fallo de la orden.

`publicar` no calcula nada: añade `name`, `head_sha`, `status`,
`details_url` y `output.text`, y acepta la conclusión de `check.json` solo si
es `success` o `failure`. El lint del workflow exige que `validar` corra
`tc2 hallazgos veredicto`, que `publicar` lea la conclusión de
`"$dir/check.json"`, que solo nombre `success` y `failure` y que `success`
aparezca una sola vez, en el `case` que copia `check.json` (contando la
palabra, escriba como escriba jq la clave).

### Informe en la carpeta de la tarea

`tc2 review TASK-NNN`, cuando `revision-ia` ya terminó (`revision-ia=ok` o
`fallo`), lee el check con la identidad del agente (`checks:read`; operación
`leer-revision` del puerto, adenda F6-5 de F1-13) y guarda por el punto único
de escritura de la rama de estado:

- `TASK-NNN/revision/revision-ia-<sha12>.json` (`$defs/informe-tarea`):
  tarea, PR, SHA, conclusión, veredicto, recuentos, el resumen del check y
  los aceptados leídos de `raw_details`. El título del check tiene que casar
  con su conclusión y con esos aceptados (los recuentos del título son los
  de las anotaciones leídas; un `incompleta` sin anotaciones vale con
  cualquier título); si no, por ejemplo porque un lote de anotaciones no
  llegó, `leer-revision` falla con `respuesta-invalida` y `review` da
  `hallazgos=sin-leer` sin guardar nada;
- `TASK-NNN/revision/revision-ia-<sha12>.md`: el veredicto y la tabla de
  aceptados, generados por `tc2` con `plano`, **sin** la prosa del modelo.

La línea de `review` gana `hallazgos=<rojo|verde|incompleta|tope|sin-leer>`,
`informe=<ruta>` y (F6-6) `ronda=R nuevos=N cerrados=C`. Antes de empujar,
si el PR de la rama ya existe y su check `revision-ia` terminó, `review`
guarda el informe de ese head (el de la ronda anterior): después del push el
PR apunta al commit nuevo y el check anterior ya no se lee por el PR. Repetirla con el mismo check sale `sin-cambios`. Con el
check pendiente no se lee ni se escribe nada: el informe se guarda al
relanzar `review` cuando el check ya terminó. La CI no escribe en la rama de
estado (necesitaría `contents: write`, que solo tiene el agente). Los
rechazados quedan solo como recuento y motivos del resumen.

## Rondas (F6-6)

TASK-052, RF-33. Cada check `revision-ia` describe su ronda (título con
`ronda=`) y lleva en sus anotaciones **todos** los hallazgos que siguen
abiertos. La ronda siguiente solo necesita ese check, publicado por la App
del revisor; no la rama de estado, que escribe el agente (F1-8): si el
agente borrara un IMPORTANTE de su informe, la ronda siguiente lo seguiría
viendo.

En `preparar`, el paso «Ronda anterior» recorre los commits del PR (`git
rev-list --first-parent origin/<base>..<head>`, sin el head, como mucho 100;
si hay más, falla y el check sale `incompleta`) del más nuevo al más viejo y
vuelca en crudo todos sus check-runs `revision-ia` (`GITHUB_TOKEN` con
`checks: read`; `filter=all`, no solo el último de cada commit). Tras cada
volcado, `tc2 hallazgos anterior` dice si ya hay ronda anterior: de cada
commit, del más nuevo al más viejo, el `revision-ia` terminado de la App del
revisor (`plataforma.revisor_id` de la configuración de la base; sin él, no
hay ronda anterior) más nuevo que dice `rojo` o `verde`. Una ronda
`incompleta` no cuenta ni sirve de base (si en el mismo commit hay antes un
`rojo`, cuenta ese); una `tope` tampoco (no revisó nada) y una `ligera`
tampoco (no la vio ningún modelo; se reconoce por la línea `Ronda N (ligera)`
de su resumen). Del elegido se bajan sus páginas de anotaciones y `tc2
hallazgos ronda` exige que título, conclusión y abiertos casen (la regla de
`leer-revision`); si no, falla (falla en cerrado) y el check sale
`incompleta`, con un mensaje que dice cómo salir: si el check se publicó a
medias (falló un lote de anotaciones), relanzar el workflow `revision-ia`
del commit de ese check publica en él uno completo, que pasa delante.

El diff incremental sale de `tc2 hallazgos desde --anterior <sha> --head
<head> --base origin/<base>`: desde el commit anterior, **sin lo que entró
de la base después** (un merge de la base o el botón «Update branch»), que
no es del PR. Si desde entonces no entró nada de la base, es `<sha>..<head>`;
si entró, es el diff desde el árbol de `<sha>` con lo que trajo la base
(`git merge-tree --write-tree <sha> <merge-base>`) hasta el head. Tras un
rebase que deja el commit anterior fuera de la historia, o si esa mezcla
tiene conflictos, el diff vuelve a `base...head` (falla en cerrado: se revisa
todo el PR) y los abiertos se arrastran igual. Con los ficheros cambiados de
ese diff decide `ronda.json`:

| Modo | Cuándo | Qué pasa |
|---|---|---|
| `completa` | sin ronda anterior | ronda 1: todo el diff contra la base |
| `incremental` | el resto | la ronda sube; cada revisora ve el diff desde el commit anterior y sus abiertos CRITICO e IMPORTANTE |
| `ligera` | anterior `verde`, sin otra `ligera` desde él, y los cambios solo tocan ficheros con MENOR abiertos | sin modelo; verde; la ronda no sube; no sirve de base del diff siguiente |
| `tope` | anterior `rojo` y la ronda siguiente pasa del tope | sin modelo; dictamen `tope` (`failure`) con los abiertos arrastrados |

El tope efectivo es `tope_rondas` de la configuración de la base más uno por
cada elemento de lista de primer nivel con texto (`- ...` o `* ...`, fuera de
los bloques de código) en `## Rondas extra` (o `## Ronda extra`) de la tarea
(rama de estado). La ronda extra la decide la persona (F1-16, D6a):
F4-4 traducirá `tope` a la parada `preguntar` de F1-10.

Cierre (falla en cerrado): en modo incremental la respuesta es un objeto
`{"hallazgos": [...], "cierres": [...]}` (la clave que falte cuenta como
lista vacía). Un abierto CRITICO o IMPORTANTE solo
se cierra con un cierre `{id, fichero, linea, motivo}` cuya línea esté en el
lado nuevo del diff incremental que vio la revisora; si el abierto traía
`paso_rojo`, ese paso tiene que estar ya en verde en el informe F5-4 del head
nuevo. Motivos nuevos: `cierre-desconocido`, `cierre-fuera-del-diff`,
`paso-rojo-sigue-rojo`. Un hallazgo nuevo no puede reutilizar el id de un
abierto. Los MENOR no pasan por el cierre del modelo: se cierran cuando su
fichero cambia. `tc2 hallazgos abiertos` reparte los abiertos graves: cada
uno a su revisora si sigue elegida y, si no, a la que tiene su fichero entre
sus rutas (los ficheros de los abiertos también entran en las rutas de `tc2
revisoras --ronda <n>`, con lo que S13 se aplica desde la ronda 2).

`veredicto --ronda` cuenta `nuevos` (aceptados en esta ronda), `persisten`
(abiertos heredados sin cierre) y `cerrados` (cerrados en esta ronda) y los
deja en el título, el resumen, las anotaciones y el informe de la carpeta de
la tarea. La ejecución de la evidencia (adenda F6-4 de F1-8) no entra en
F6-6 (decisión 3 de TASK-052).

## Límites conocidos

- Sin saldo en la API de Anthropic, `revision-ia` sale siempre en rojo
  `incompleta` (es lo honesto: no hubo revisión); el título lo distingue de
  un rojo por hallazgos.
- Si un `PATCH` de un lote falla a medias, el check ya está publicado con su
  conclusión y le faltan anotaciones; el job `publicar` sale en rojo.
- La evidencia de cada hallazgo se valida pero no se ejecuta (fuera de
  F6-6; decisión 3 de TASK-052). En el cierre solo se contrasta `paso_rojo`
  con el informe F5-4.
- Ronda `ligera` (decisión 4 de TASK-052): da verde sin consultar al modelo
  a un cambio que solo toca ficheros con MENOR abiertos, sea cual sea el
  cambio (también uno que meta un CRITICO), y da esos MENOR por cerrados sin
  mirar si se arreglaron. Lo acota que no sirve de base: la ronda siguiente
  sale desde el último commit revisado por modelo, y no puede ser otra
  `ligera`, así que lo que entró en la ligera lo ve el modelo en el push
  siguiente. Si la ligera es el último push antes de integrar, ese cambio
  se integra sin revisión IA (la revisión por pares de `review` sigue).
- El ancla de un cierre solo prueba que esa línea está en el lado nuevo del
  diff incremental (de cualquier fichero: un arreglo puede estar en otro,
  p. ej. en el test); que esa línea arregle el abierto lo juzga el modelo.
  Los CRITICO e IMPORTANTE sin cierre válido siguen abiertos.
- Rondas (F6-6): sin saldo en la API la cadena real no pasa de la ronda 1
  (`incompleta` no cuenta). Una revisora que reporta un abierto con otro id
  infla `nuevos`; la anotación de un abierto que persiste conserva la línea
  de la ronda en que se encontró, aunque el código se haya desplazado. Un
  force-push que reescribe los commits del PR deja fuera el check anterior:
  la ronda vuelve a ser la 1, completa. El coste de buscar la ronda anterior
  es una llamada a la API por commit del PR hasta encontrarla (tope 100).
- Que GitHub deje pasar las respuestas por `outputs` sin enmascarar ni
  truncar no se ha comprobado aún en un run real (E2E de TASK-050).
- Las tablas de hallazgos escapan el Markdown del modelo; su informe en
  prosa, debajo, va tal cual (como en F6-3): la defensa es de F6-7.
- Sin el detector de dominios (F4-6): S10 y S11 no se aplican (S13 sí, con
  la ronda de F6-6).
- El diff y el informe son datos del autor del PR que llegan al modelo; la
  defensa contra inyección de instrucciones es de F6-7.
- Coste: una llamada por revisora y ronda (2 o 3 en un PR Rust + Vue).
- El informe de una verificación reutilizada se encuentra por la línea
  `Perfil <p>: reutilizada de ... (origen <sha>, ...)` del log de
  `verificacion.yml`: si cambia su formato, `preparar` se queda sin informe.
  Si el artefacto del run de origen ha caducado (retención de artefactos del
  repositorio, 90 días por defecto), el paso lo avisa con un `::warning::`,
  el check dice `informe=sin-informe` y las revisoras van sin la sección
  (TASK-137).
- Un error de la API sale en el check con su tipo y mensaje recortados
  (por ejemplo, la cuenta sin saldo da `error-http-400
  invalid_request_error: Your credit balance is too low...`).

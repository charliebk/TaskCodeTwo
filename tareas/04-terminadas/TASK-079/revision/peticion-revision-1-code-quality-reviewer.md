# Peticion de revision — TASK-079 (ronda 1)

- Tarea: TASK-079 — F3-5b status y roadmap siempre frescos
- Rama revisada: feature/task-079-f3-5b-status-y-roadmap-siempre-frescos
- Rama base: develop
- Commit revisado (HEAD): 67b8a3455d064a006a5464c96db0e3877b4a4834
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-079 (criterios de aceptacion y plan)
- Alcance: completa (motivo: primera ronda de revision)

## Instrucciones para el agente revisor

Eres un revisor INDEPENDIENTE del agente que implemento. Tu trabajo es
reproducir empiricamente, no leer el diff y opinar: clona el repo a un
directorio temporal, corre la suite tu mismo (salvo que esta
peticion ya traiga la verificacion de la ronda en verde para el commit revisado;
entonces vale ese resultado) y construye el caso que
rompe el codigo antes de reportarlo. Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. Vuelca tu salida en el
informe de esta ronda (informe-revision-1-code-quality-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-079/tarea.md
- tareas/03-en-revision/TASK-079/planificacion/
- docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
- docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md

## Commits a revisar (git log develop..HEAD)

````
67b8a34 docs(TASK-079): autorrevision con rust-wasm-reviewer y code-quality-reviewer
69d39bd refactor(TASK-079): formato de frescura.rs con cargo fmt
dfd9270 docs(TASK-079): adendas de F1-4 y F1-2 por la lectura fresca
26ceae8 test(TASK-079): criterios C1 a C9 de la lectura fresca
aedaf7a feat(TASK-079): la transaccion renueva la marca tras publicar
c76837f feat(TASK-079): status, board, next y roadmap traen antes de mostrar
07396f2 feat(TASK-079): lectura fresca de la rama de estado
716af9d refactor(TASK-079): traer_estado extraido de la transaccion
0a01917 feat(TASK-079): marca de sincronizacion y formato de la frescura
d2ac114 feat(TASK-079): fichero de la marca de sincronizacion en la huella
26330f7 chore(TASK-079): tarea en curso
````

## Diff de tu dominio (2 fichero(s) de 22; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
index bd27bbc..2d3c301 100644
--- a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
+++ b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
@@ -5,6 +5,7 @@
 - Requisitos: RR-01, RR-09, RR-12, RN-05, RN-06 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); H3 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md); se apoya en RF-02 y RF-07
 - Depende de: [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (carpeta fija `TASK-NNN/` en la rama huérfana `taskcode/tareas`, montada en `tareas/`)
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2b, D3a, D4a del plan de TASK-003)
+- Adenda (2026-11-17, TASK-079, F3-5b): la lectura fresca de los puntos 5 y 6 queda hecha; ver [la adenda del final](#adenda-f3-5b-task-079--lectura-fresca-y-hora-de-sincronización)
 - Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
@@ -534,3 +535,43 @@ Evidencia: `crates/tc2/tests/ciclo.rs`
 (`c8_sin_red_descartar_falla_y_la_pausa_queda_pendiente`,
 `c4_el_id_de_la_descartada_no_se_reutiliza`) y los unitarios de
 `ciclo/operacion.rs` (`anotacion_ida_y_vuelta`).
+
+## Adenda (F3-5b, TASK-079) — lectura fresca y hora de sincronización
+
+Cierra lo que los puntos 5 y 6 dejaban a F3-5b (opciones recomendadas
+D1a..D6a del plan de TASK-079, aceptadas por Carlos para todo el H3):
+
+- **Las cuatro órdenes de lectura traen antes de mostrar** (`status`,
+  `board`, `next` y `roadmap`): un `fetch` de la rama de estado por orden,
+  el **mismo** que hace cada intento de la transacción
+  (`estado::frescura::traer_estado`, extraído de `transaccion.rs` sin cambiar
+  su comportamiento). No toman el cerrojo ni escriben nada más que
+  `refs/remotes/origin/taskcodetwo/tareas` y la marca. `estado::leer` sigue
+  sin `fetch` (lo usan `doctor`, `import` y `start`, que ya traen por su
+  cuenta o no deben hablar con el remoto).
+- **Hora de la última sincronización** en `.git/taskcodetwo/sincronizacion`
+  (segundos desde 1970, temporal y `rename`; un fichero que no se entiende
+  es «nunca»). La renueva cualquier `fetch` con éxito de la rama de estado,
+  de una lectura o de una transacción (así un `new` con éxito deja la hora
+  de ese `new`). Un `git fetch` a mano no la renueva: la hora es
+  conservadora, nunca más fresca de lo real. `desinstalar` la borra con la
+  carpeta propia.
+- **Sin red no es un error**: se muestra lo último traído con
+  `remoto=sin-red` y un aviso con la hora y «hace N»; igual con
+  `remoto=denegado` (credenciales o URL), `error-local` o `sin-rama`. Un
+  repo sin remoto no hace `fetch` (`remoto=sin-remoto`), como ya decía el
+  punto 5.
+- **`status` dice cuántas pendientes hay** (`pendientes=N`, las refs de
+  `refs/taskcodetwo/pendientes/`) y avisa, con código 0, lo que la adenda de
+  TASK-018 dejó a F3-5b: `tareas=diverge` (la rama local tiene commits que no
+  están en lo publicado) y `tareas=con-cambios` (`tareas/` está atrasada y
+  tiene cambios sin publicar, que impedirán adelantarla).
+- **Riesgos aceptados**: ~1 s más por orden contra GitHub; sin tiempo límite
+  en el puerto, un host mudo cuelga la lectura lo que tarde TCP (igual que la
+  transacción); dos `fetch` a la vez en el mismo clon pueden dar `cannot lock
+  ref`, que se clasifica como `sin-red` (se muestra lo último traído con
+  aviso; no se escribe nada).
+
+Evidencia: `crates/tc2/tests/frescura.rs` (C1..C9 de TASK-079), los
+unitarios de `estado/frescura.rs` y la suite de la transacción sin cambios
+(`estado_*`, `alta`, `toma`, `ciclo`, `importar`).
diff --git a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
index b522b7c..548be13 100644
--- a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
+++ b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
@@ -9,6 +9,7 @@
 - Adenda (2026-11-17, TASK-033, F3-6): `start [--json] TASK-NNN` compone `fetch`, `validar-base`, `comprobar-id`, `commit-estado`, `crear-rama-y-worktree` y `reanudar` sin operaciones nuevas (la base de un `hotfix` es el último tag por versión de `refs/tags/` contenido en `origin/main`, leído con `for-each-ref --merged`, o su punta). Línea de éxito: `start ok id=… rama=… worktree=… base=… estado=<commit de la toma> asignado=<login>`; dueño con la tarea empezada: `sin-cambios` o `ok creado=rama,worktree` sin escribir estado. Motivos de `rechazada` (3): `no-existe`, `formato-1` (con `arreglo=tc2 actualizar`), `ilegible`, `tomada` (con `asignado=<login>`), `asignado-heredado`, `terminada`, `descartada`, `estado-no-valido`, `dependencias-pendientes` (con `dependencias=`), `rama-invalida`, `tipo-release`, `ocupada` (con `ramas=`), `carpeta-ocupada` (la carpeta del worktree existe y no es el worktree registrado de la rama; se comprueba antes de publicar la toma, sin `estado=`) e `identidad`; `uso` (2): `sin-id`, `id-invalido`, `argumentos-de-mas`; sin red, `sinred pendiente=no` (4)
 - Adenda (2026-11-17, TASK-034, F3-7): `descartar [--json] TASK-NNN "<motivo>"`, `pausa [--json] TASK-NNN` y `reanudar [--json] TASK-NNN` componen `fetch` (solo `descartar`), `commit-estado` y `quitar-worktree` (solo `descartar`, tras publicar) sin operaciones nuevas. Líneas: `descartar ok id=… commit=… intentos=… tareas=… worktree=quitado|ninguno|conservado [worktree-motivo=cambios-sin-guardar|sin-publicar|sin-rama|en-uso|error]` (el worktree conservado no hace fallar la orden: el descarte ya está publicado y la rama no se toca nunca); `pausa ok id=… commit=… intentos=… tareas=…` y `reanudar ok id=… commit=… intentos=… tareas=…` (las claves de la transacción, como `descartar`, sin `worktree`). `sin-cambios` (0): descartar una descartada (vale el motivo de entonces y se intenta quitar el worktree), pausar una en pausa o reanudar una que no lo está. Motivos de `rechazada` (3): `no-existe`, `formato-1` (con `arreglo=tc2 actualizar`), `ilegible`, `de-otra-persona` (con `asignado=<login>`; D2: puede quien la lleva o cualquiera si no la lleva nadie), `asignado-heredado`, `terminada`, `descartada` (pausa y reanudar), `estado-no-valido` e `identidad`; `start` gana `pausada` (con `arreglo=tc2 reanudar`). `uso` (2): `sin-id`, `id-invalido`, `argumentos-de-mas` y, en `descartar`, `sin-motivo` y `motivo-invalido` (vacío, solo espacios, saltos de línea o caracteres de control, o más de 500 caracteres). Sin red: `descartar sinred pendiente=no` (4); `pausa`/`reanudar sinred pendiente=si` (4), que la siguiente transacción con red publica (adenda de TASK-034 en F1-2)
 - Adenda (2026-11-17, TASK-035, F3-8): `doctor [--json]` compone lecturas sin escribir nada (ni `fetch`): `git version` entra en la lista permitida (decisión 3; solo lectura), y la lista cerrada de la plataforma gana `leer-plan` (detalle en la [adenda de F1-13](F1-13-adaptador-de-plataforma.md)). Salida: primera línea `doctor <ok|rechazada> fallos=N avisos=N no-comprobado=N plataforma=ok|no-configurada|sin-credenciales|sin-red|error [plan=… impuestas=N sustituidas=N no-disponibles=N [no-comprobadas=N]] [commit=…]` (`no-comprobadas` solo si alguna garantía cuya configuración se lee no se pudo leer: no cuenta como impuesta; ronda 1 de TASK-035); detrás, los hallazgos que no son `ok` (`<zona> <id> <falla|aviso|no-comprobado> [motivo=…] [claves] [arreglo=…]`), los peores primero, y si no caben en 5 líneas, 3 y `y N mas: tc2 doctor --json`. `--json`: una línea con las claves y `hallazgos` (todos, también los `ok`). Código 3 (`rechazada`) con alguna `falla` (D5a de TASK-035) y 0 con avisos o no-comprobados; fuera de un repo, `doctor rechazada motivo=no-es-repo` (3); argumentos de más, `uso motivo=argumentos` (2; `--recursos` llega con F3-9). Git por debajo de 2.56: `aviso motivo=git-no-verificado` (D3a de TASK-035); sin git o con una versión que no se entiende, `falla`. Diagnóstico de una línea en stderr solo con alguna `falla`
+- Adenda (2026-11-17, TASK-079, F3-5b): `status`, `board`, `next` y `roadmap` (y `roadmap --comparar`) traen antes la rama de estado con el mismo `fetch -q origin refs/heads/taskcodetwo/tareas:refs/remotes/origin/taskcodetwo/tareas` de la transacción (sin operaciones nuevas, sin cerrojo) y solo escriben esa ref remota y `.git/taskcodetwo/sincronizacion`. Claves nuevas en las cuatro, detrás de las que ya tenían: `remoto=consultado|sin-red|denegado|error-local|sin-rama|sin-remoto` (cómo fue el `fetch`; sin `origin` no se hace) y `sincronizado=<AAAA-MM-DDTHH:MM:SSZ>|nunca` (hora UTC del último `fetch` con éxito, de una lectura o de una transacción). Un fallo del remoto no cambia el resultado: sale `ok` (0) con lo último traído y un aviso de una línea en stderr con la hora y «hace N» (o que no se sincronizó nunca). `status` gana además `pendientes=N` (refs de `refs/taskcodetwo/pendientes/`) y, si la carpeta montada no se puede adelantar, `tareas=con-cambios|diverge` con su aviso (código 0). En texto, la hora va al final de la línea de `status` (`· sincronizado …`) y de la cabecera de `board`; en `roadmap`, a stderr (`estado sincronizado con origin el … (hace …)`), porque su stdout sigue siendo el `ROADMAP.md` byte a byte. `doctor` no hace `fetch` y añade `sincronizado=` (de la marca) detrás de `commit=`
 - Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-079/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-079/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-079/tarea.md                                    | 29 +++++++++++++++++++++++++++--
 3 files changed, 27 insertions(+), 2 deletions(-)
````

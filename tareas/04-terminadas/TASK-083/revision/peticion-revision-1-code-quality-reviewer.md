# Peticion de revision — TASK-083 (ronda 1)

- Tarea: TASK-083 — F3-13 Estimacion por fase de cada tarea al planificar
- Rama revisada: feature/task-083-f3-13-estimacion-por-fase-de-cada-tarea
- Rama base: develop
- Commit revisado (HEAD): 2792c7e1f3c33f6fd9ab426df543f15c6fee3c6f
- Fecha: 2026-11-18
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-083 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-083/tarea.md
- tareas/03-en-revision/TASK-083/planificacion/
- crates/tc2/tests/fixtures/estimacion/paridad-poc-f1-17.json
- docs/adr/F1-17-progreso-y-estimacion.md
- docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md

## Commits a revisar (git log develop..HEAD)

````
2792c7e docs(TASK-083): autorrevision y correccion del criterio C1
e9a6879 docs(TASK-083): adendas de F1-17 y F1-4 por la estimacion
a1c4841 test(TASK-083): paridad con poc-f1-17.py y E2E de C1 a C7
bc21575 feat(TASK-083): recalcular en new, import y start, tc2 estimar y doctor
1a5e8fb feat(TASK-083): lector del historico y git log de develop
9387345 feat(TASK-083): forma del campo estimacion, su lectura y su linea
c21b687 feat(TASK-083): estimar, restante y error como funciones puras
3ee6b69 feat(TASK-083): real por fase de una tarea terminada (estimacion::real)
b92ff20 chore(TASK-083): tarea en curso
````

## Diff de tu dominio (3 fichero(s) de 23; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-17-progreso-y-estimacion.md b/docs/adr/F1-17-progreso-y-estimacion.md
index 9bbb010..4fcf8dd 100644
--- a/docs/adr/F1-17-progreso-y-estimacion.md
+++ b/docs/adr/F1-17-progreso-y-estimacion.md
@@ -6,6 +6,15 @@
 - Requisitos: RR-19 y RR-12 ([catálogo](../plan/REQUISITOS-FUNCIONALES.md)); en lo que toca, RR-01 (lo hace el CLI, sin tokens del agente)
 - Depende de: [ADR F1-9](F1-9-formato-de-tarea-plan-y-especificacion.md) (campo `estimacion`, `## Duracion`, línea `Duracion estimada`), [ADR F1-10](F1-10-carriles-y-modo-automatico.md) (carril declarado y carril mínimo por ficheros), [ADR F1-2](F1-2-escrituras-atomicas-ids-y-toma.md) (punto único de escritura, `SinRed`, `Rechazada`), [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md) (`start`, worktree por tarea, merge sin fast-forward), [ADR F1-14](F1-14-distribucion-e-instalacion.md) (la status line llama al binario de Rust), [ADR F1-15](F1-15-seleccion-de-skills-agentes-y-modelo.md) (texto de la tarea no confiable), [ADR F1-16](F1-16-lotes-desatendidos.md) (CAS de procedencia, mediana de una ronda) y [ADR F1-18](F1-18-hito-de-decisiones.md) (presupuesto con mediana y p75)
 - Decide: Carlos, el 2026-11-14 y el 2026-11-15: RR-19 imprescindible; requisitos de la persona (1) `resumen` tipo dashboard, (2) tareas en paralelo en terminales distintas con `start` que se niega si una predecesora no está terminada y avisa del solape, y (3) reparto por camino crítico con `--personas N` y zonas declaradas; seguridad y trabajo en grupo imprescindibles; comandos deterministas sobre hooks; nombres `tc2` y `taskcodetwo`; sin trocear. Respuestas al plan de TASK-075: **D1a a D8a**; D3a y D4a se le preguntaron expresamente por cambiar su forma de trabajar y respondió «Apruebo las dos». El 2026-11-15, durante la implementación, redujo el alcance de la PoC: completa en el lector y en la estimación fuera de muestra, mínima en lo demás.
+- Adenda (2026-11-18, TASK-083, F3-13; decisiones D1a a D5a de su plan, con la consigna del H3 «la opción recomendada»):
+  - **Cuándo se recalcula (D2a, concreta el punto 6)**: en `new`, `import` y `start` (dentro de la operación, sobre el estado fresco de cada intento y con un solo `git log` de `develop` leído antes de la transacción) y en `tc2 estimar`; en `plan`, `approve`, `review` y `finish` cuando existan (F4-1, F4-2, F6-2, F8-7), por el mismo `estimacion::recalcular`. `pausa`, `reanudar`, `liberar`, `reasignar` y `descartar` **no** tocan `estimacion` (la conservan byte a byte). Un `import` calcula el histórico una vez para todo el lote.
+  - **Sin semillas (D3a, concreta el punto 3)**: la cascada de la 2.0 tiene cuatro niveles (carril + complejidad + tamaño, carril + complejidad, carril, repo); sin ningún nivel con 5 muestras, `nivel: "sin-datos"`, `n` = muestras disponibles, `por_fase: null` y `total: null`, y la vista dice «sin tareas parecidas (n=k<5)»; nunca un número inventado. Las semillas de otros proyectos quedan fuera hasta que una tarea las pida con datos.
+  - **Carril de las muestras (D5a)**: el declarado. Una tarea de la 0.16 (sin `carril`) solo cuenta en el nivel `repo`, y una tarea sin carril declarado se estima solo en `repo`. La complejidad `null` se compara igual a `null`, como la PoC.
+  - **Error (D4a)**: `tc2 estimar --error TASK-NNN` (solo lectura) da, por fase y en total, el error absoluto (min), el relativo (|estimado − real| / real, real de al menos 1 min) y la razón estimado/real de la mediana guardada frente al real recalculado; guardarlo al cerrar es de F8-7.
+  - **Forma del punto 7**: los pares son minutos enteros (redondeo al más cercano); `agente` queda `null` hasta que exista `plan`. La tarea estimada nunca es muestra de sí misma (ni dentro de un grupo de troceo).
+  - **Commits de una tarea (punto 1)**: los de `git log <origin/develop | develop>` cuyo asunto la nombra (`TASK-N`, sin mayúsculas, normalizado a tres cifras) y los de `M^1..M^2` del merge `merge(feature|fix): feature|fix/task-nnn-…` de la 0.16; sin `develop`, ninguno. Verificado con los asuntos de los merges de este repo.
+  - **Caso de C1 de TASK-083**: con `start` 10:00, commit 10:20, `pausa` 10:30, `reanudar` 11:30, commit 11:40, `review` 13:00 y 13:30 y `finish` 13:40, la regla de este ADR (y la PoC) da implementación **85** (20 + 10 + 10 + 45), rondas 30 y cierre 10; el 75 del enunciado es el de la misma tarea sin el commit de las 11:40. Se sigue la regla, con paridad exacta con `poc-f1-17.py` (fixture de `tests/estimacion.rs`).
+  - **`doctor`**: el hallazgo `estado historico ok usables=N descartadas=M` (sin `git log`: los descartes no dependen de los commits).
 
 Este ADR fija **qué estima la 2.0, con qué datos, cuándo lo recalcula y dónde
 lo enseña sin gastar tokens**. Hay un solo lector del histórico,
diff --git a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
index a1019de..beccbc7 100644
--- a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
+++ b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
@@ -10,6 +10,7 @@
 - Adenda (2026-11-17, TASK-034, F3-7): `descartar [--json] TASK-NNN "<motivo>"`, `pausa [--json] TASK-NNN` y `reanudar [--json] TASK-NNN` componen `fetch` (solo `descartar`), `commit-estado` y `quitar-worktree` (solo `descartar`, tras publicar) sin operaciones nuevas. Líneas: `descartar ok id=… commit=… intentos=… tareas=… worktree=quitado|ninguno|conservado [worktree-motivo=cambios-sin-guardar|sin-publicar|sin-rama|en-uso|error]` (el worktree conservado no hace fallar la orden: el descarte ya está publicado y la rama no se toca nunca); `pausa ok id=… commit=… intentos=… tareas=…` y `reanudar ok id=… commit=… intentos=… tareas=…` (las claves de la transacción, como `descartar`, sin `worktree`). `sin-cambios` (0): descartar una descartada (vale el motivo de entonces y se intenta quitar el worktree), pausar una en pausa o reanudar una que no lo está. Motivos de `rechazada` (3): `no-existe`, `formato-1` (con `arreglo=tc2 actualizar`), `ilegible`, `de-otra-persona` (con `asignado=<login>`; D2: puede quien la lleva o cualquiera si no la lleva nadie), `asignado-heredado`, `terminada`, `descartada` (pausa y reanudar), `estado-no-valido` e `identidad`; `start` gana `pausada` (con `arreglo=tc2 reanudar`). `uso` (2): `sin-id`, `id-invalido`, `argumentos-de-mas` y, en `descartar`, `sin-motivo` y `motivo-invalido` (vacío, solo espacios, saltos de línea o caracteres de control, o más de 500 caracteres). Sin red: `descartar sinred pendiente=no` (4); `pausa`/`reanudar sinred pendiente=si` (4), que la siguiente transacción con red publica (adenda de TASK-034 en F1-2)
 - Adenda (2026-11-17, TASK-035, F3-8): `doctor [--json]` compone lecturas sin escribir nada (ni `fetch`): `git version` entra en la lista permitida (decisión 3; solo lectura), y la lista cerrada de la plataforma gana `leer-plan` (detalle en la [adenda de F1-13](F1-13-adaptador-de-plataforma.md)). Salida: primera línea `doctor <ok|rechazada> fallos=N avisos=N no-comprobado=N plataforma=ok|no-configurada|sin-credenciales|sin-red|error [plan=… impuestas=N sustituidas=N no-disponibles=N [no-comprobadas=N]] [commit=…]` (`no-comprobadas` solo si alguna garantía cuya configuración se lee no se pudo leer: no cuenta como impuesta; ronda 1 de TASK-035); detrás, los hallazgos que no son `ok` (`<zona> <id> <falla|aviso|no-comprobado> [motivo=…] [claves] [arreglo=…]`), los peores primero, y si no caben en 5 líneas, 3 y `y N mas: tc2 doctor --json`. `--json`: una línea con las claves y `hallazgos` (todos, también los `ok`). Código 3 (`rechazada`) con alguna `falla` (D5a de TASK-035) y 0 con avisos o no-comprobados; fuera de un repo, `doctor rechazada motivo=no-es-repo` (3); argumentos de más, `uso motivo=argumentos` (2; `--recursos` llega con F3-9). Git por debajo de 2.56: `aviso motivo=git-no-verificado` (D3a de TASK-035); sin git o con una versión que no se entiende, `falla`. Diagnóstico de una línea en stderr solo con alguna `falla`
 - Adenda (2026-11-17, TASK-079, F3-5b): `status`, `board`, `next` y `roadmap` (y `roadmap --comparar`) traen antes la rama de estado con el mismo `fetch origin refs/heads/taskcodetwo/tareas:refs/remotes/origin/taskcodetwo/tareas` de la transacción y de `montar` (sin `-q`, para que git no calle el rechazo non-fast-forward; sin operaciones nuevas, sin cerrojo) y solo escriben esa ref remota y `.git/taskcodetwo/sincronizacion`. Claves nuevas en las cuatro, detrás de las que ya tenían: `remoto=consultado|sin-red|rechazado|denegado|error-local|sin-rama|sin-remoto` (cómo fue el `fetch`; sin `origin` no se hace; `rechazado`: la rama de estado de `origin` está reescrita y no desciende de la traída; `error-local` incluye la ref remota bloqueada por un `.lock`, que la lectura reintenta dos veces antes y nombra en su aviso; una escritura que encuentra la rama reescrita sale `error` con `motivo=rama-reescrita`, y la ref bloqueada, `error` con `motivo=error-local`, sin quedar pendiente) y `sincronizado=<AAAA-MM-DDTHH:MM:SSZ>|nunca` (hora UTC del último `fetch` con éxito, de una lectura, de una transacción o de `montar`; una marca con hora posterior a ahora cuenta como `nunca`). Un fallo del remoto no cambia el resultado: sale `ok` (0) con lo último traído y un aviso de una línea en stderr con la hora y «hace N» (o que no se sincronizó nunca). `status` gana además `pendientes=N` (refs de `refs/taskcodetwo/pendientes/`) y, si la carpeta montada no se puede adelantar, `tareas=con-cambios|diverge` con su aviso (código 0). En texto, la hora va al final de la línea de `status` (`· sincronizado …`, seguida de `· pendientes N` solo si N > 0) y de la cabecera de `board`; en `roadmap`, a stderr (`estado sincronizado con origin el … (hace …)`), porque su stdout sigue siendo el `ROADMAP.md` byte a byte. `doctor` no hace `fetch` y añade `sincronizado=` (de la marca) detrás de `commit=`
+- Adenda (2026-11-18, TASK-083, F3-13): `log` entra en el puerto `Git` (ya estaba en la lista permitida de la decisión 3; solo lectura): un único `git log --format=%H%x1f%P%x1f%ct%x1f%s <origin/develop | develop> --` por `new`, `import`, `start` y `tc2 estimar` para el histórico de la estimación (F1-17). `new ok` y `start ok` ganan al final `estimacion=<mediana total en min>|sin-datos`, `estimacion-p80=<min>` (solo con datos), `estimacion-n=<n>` y `estimacion-nivel=<nivel>`; `import` no (cada `tarea.md` lleva la suya). Orden nueva `tc2 estimar [--json] [TASK-NNN | --error TASK-NNN]`, que no es una operación Git (no choca con D1a): sin argumentos, transacción de la rama de estado que estima las tareas abiertas de formato 2 con `estimacion: null` (`estimar ok actualizadas=N ids=… commit=… intentos=… tareas=…`, `estimar sin-cambios actualizadas=0`, sin red `sinred pendiente=no`); con `TASK-NNN`, solo lectura del estado conocido (sin `fetch`): `estimar ok id=… estimacion=… [estimacion-p80=…] estimacion-n=… estimacion-nivel=…` y, sin `--json`, una segunda línea para la persona `estimacion: ≈M min (p80 P), n=K, nivel=X` o `estimacion: sin tareas parecidas (n=k<5)`; con `--error TASK-NNN`, solo lectura: `estimar ok id=… error-<fase>=<min> relativo-<fase>=<0.00> razon-<fase>=<0.00>…` (fases con cifra y `total`) y la línea `error: …`; rechazos (3) `no-existe`, `formato-1`, `ilegible`, `sin-estimacion` («sin estimacion guardada», sin cifras), `estimacion-ilegible`, `sin-cifras` (guardada `sin-datos`) y `sin-real` (con `causa=` del descarte); `uso` (2) `sin-id`, `id-invalido`, `argumentos-de-mas`
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
tareas/{01-en-diseno => 02-en-curso}/TASK-083/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-083/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-083/tarea.md                                    | 35 ++++++++++++++++++++++++++++++++---
 3 files changed, 32 insertions(+), 3 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/fixtures/estimacion/paridad-poc-f1-17.json"
````

````
crates/tc2/tests/fixtures/estimacion/paridad-poc-f1-17.json | 605 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 605 insertions(+)
````

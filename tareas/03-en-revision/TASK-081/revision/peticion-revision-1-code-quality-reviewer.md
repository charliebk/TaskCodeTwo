# Peticion de revision — TASK-081 (ronda 1)

- Tarea: TASK-081 — F3-10 Liberar y reasignar tareas abandonadas
- Rama revisada: feature/task-081-f3-10-liberar-y-reasignar-tareas-abandon
- Rama base: develop
- Commit revisado (HEAD): 6f41ef2e6595a749fa6ea41439152efc1dfb5182
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-081 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-081/tarea.md
- tareas/03-en-revision/TASK-081/planificacion/
- crates/tc2/esquemas/config.schema.json
- docs/adr/F1-11-extensiones-declaradas.md
- docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
- docs/adr/F1-3-flujo-de-ramas-worktrees-release.md
- docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md

## Commits a revisar (git log develop..HEAD)

````
6f41ef2 docs(TASK-081): autorrevision con rust-wasm-reviewer y code-quality-reviewer
bfc11c6 docs(TASK-081): adendas de F1-2, F1-3, F1-9 y F1-11 por liberar y reasignar
e809973 feat(TASK-081): tareas sin actividad y aviso ya-no-tuyas en status y board
3da3c0a feat(TASK-081): tc2 liberar y tc2 reasignar
c7c8b81 refactor(TASK-081): actividad de la tarea, fechas de ramas y worktree publicado
6a864b5 chore(TASK-081): tarea en curso
````

## Diff de tu dominio (5 fichero(s) de 25; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/esquemas/config.schema.json b/crates/tc2/esquemas/config.schema.json
index 4ec41f7..4586fc5 100644
--- a/crates/tc2/esquemas/config.schema.json
+++ b/crates/tc2/esquemas/config.schema.json
@@ -43,7 +43,9 @@
               "review",
               "finish",
               "descartar",
-              "pausa"
+              "pausa",
+              "liberar",
+              "reasignar"
             ],
             "description": "en que transiciones corre (por defecto, en todas)",
             "items": {
@@ -55,7 +57,9 @@
                 "review",
                 "finish",
                 "descartar",
-                "pausa"
+                "pausa",
+                "liberar",
+                "reasignar"
               ],
               "type": "string"
             },
@@ -86,6 +90,13 @@
       },
       "type": "array"
     },
+    "dias_sin_actividad": {
+      "default": 7,
+      "description": "dias sin actividad tras los que otra persona puede liberar o reasignar una tarea y status y board la marcan [F1-11 adenda de TASK-081; F3-10 D1a]",
+      "maximum": 365,
+      "minimum": 1,
+      "type": "integer"
+    },
     "formato": {
       "description": "version del formato del proyecto; con 2 manda la 2.0 [F1-12 D4a, decision 6]",
       "maximum": 2,
diff --git a/docs/adr/F1-11-extensiones-declaradas.md b/docs/adr/F1-11-extensiones-declaradas.md
index 392f427..317b830 100644
--- a/docs/adr/F1-11-extensiones-declaradas.md
+++ b/docs/adr/F1-11-extensiones-declaradas.md
@@ -6,6 +6,7 @@
 - Depende de: [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (carpeta fija, rama de estado huérfana, roadmap con un Mermaid por hito) y [ADR F1-7](F1-7-verificacion-ci-perfiles-cache-runners.md) (un comando con perfiles, huella del árbol y local, runner por perfil, job programado); cierra costuras abiertas por [F1-8](F1-8-seguridad-modelo-de-amenazas.md) (confianza local única, globs extra, `comando_*` como entrada no confiable, rutas de Windows en YAML), [F1-9](F1-9-formato-de-tarea-plan-y-especificacion.md) (claves desconocidas del frontmatter) y [F1-10](F1-10-carriles-y-modo-automatico.md) (configuración que falla cerrada, tope de 12 claves); cambia la lista de checks obligatorios de [F1-5](F1-5-puertas-en-github.md) (ver «Divergencias con F1-5 y F1-7»)
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a, D5a y D6a del plan de TASK-012, y las correcciones de la revisión 1), «no trocear», la restricción de que el plugin sea **eficiente y con ventajas medibles** y prefiera **comandos deterministas** a hooks, y la decisión del 2026-11-14: «todo lo que hace funcionar al plugin 2 debe estar en su repo y debe ser generalista; al actualizar a la 2 hay que quitar los parches que se aplicaron para mejorar la 0.16 dentro del repo»
 - Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
+- Adenda (2026-11-17, TASK-081, F3-10): el tope de claves de primer nivel sube a 13 con `dias_sin_actividad`; ver [la adenda del final](#adenda-f3-10-task-081--tope-de-13-claves-y-dias_sin_actividad)
 
 ## Contexto
 
@@ -833,3 +834,28 @@ Este ADR se planificó con un solo rol (arquitectura); quedan sin cubrir:
   hace falta, F5-3 o F2-7.
 - **Criterio 1 del plan**: no se revisaron `docs/` de OpenDockTrading ni
   hooks de Git de los pilotos (en ninguno de los dos se vio uno).
+
+## Adenda (F3-10, TASK-081) — tope de 13 claves y `dias_sin_actividad`
+
+Decisión D1a del plan de TASK-081 (opción recomendada, aceptada por Carlos
+para todo el H3):
+
+- **Clave nueva `dias_sin_actividad`** (entero de 1 a 365, por defecto 7):
+  días sin actividad tras los que otra persona puede `liberar` o `reasignar`
+  una tarea que no es suya, y a partir de los cuales `status` y `board` la
+  cuentan como inactiva. Se lee de la configuración de la base, como el
+  resto; sin configuración o si no vale, 7.
+- **El tope de claves de primer nivel sube de 12 a 13**
+  (`config::TOPE_CLAVES`): la tabla ya tenía 12 y la alternativa (meter el
+  umbral dentro de otra clave) mezclaba conceptos que no tienen que ver.
+  Cada clave sigue citando su ADR de origen.
+- **`liberar` y `reasignar` entran en las fases de los derivados**
+  (`derivados[].fases`): escriben en la rama de estado como `descartar` y
+  `pausa`.
+
+Evidencia: el unitario de `config/claves.rs`
+(`claves_de_primer_nivel_no_pasan_del_tope_y_cada_una_cita_su_adr`),
+`crates/tc2/tests/config.rs` (`esquema_publicado_es_el_generado`, con
+`crates/tc2/esquemas/config.schema.json` regenerado) y el umbral leído de la
+configuración en `crates/tc2/tests/liberar.rs`
+(`c5_c6_c10_tarea_de_otra_persona_por_actividad`, `umbral=3`).
diff --git a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
index 5ae690e..1a574f4 100644
--- a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
+++ b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
@@ -6,6 +6,7 @@
 - Depende de: [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (carpeta fija `TASK-NNN/` en la rama huérfana `taskcode/tareas`, montada en `tareas/`)
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2b, D3a, D4a del plan de TASK-003)
 - Adenda (2026-11-17, TASK-079, F3-5b): la lectura fresca de los puntos 5 y 6 queda hecha; ver [la adenda del final](#adenda-f3-5b-task-079--lectura-fresca-y-hora-de-sincronización)
+- Adenda (2026-11-17, TASK-081, F3-10): `liberar` y `reasignar` implementadas y el punto 4 admite la rama propia de la tarea en `start`; ver [la adenda del final](#adenda-f3-10-task-081--liberar-y-reasignar)
 - Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
@@ -603,3 +604,49 @@ la revisión 1, rama reescrita y `.lock` huérfano), los unitarios de
 `marca_del_futuro_es_nunca`) y de `estado/lectura.rs`
 (`reintenta_solo_la_ref_bloqueada`), y la suite de la transacción
 (`estado_*`, `alta`, `toma`, `ciclo`, `importar`).
+
+## Adenda (F3-10, TASK-081) — liberar y reasignar
+
+Cierra lo que el punto 5 y las consecuencias dejaban a F3-10 (opciones
+recomendadas D1a..D6a del plan de TASK-081, aceptadas por Carlos para todo el
+H3):
+
+- **`liberar` y `reasignar` exigen red** (D2b, como ya decía el punto 5):
+  `fetch` completo (también las ramas) y, sin red, `sinred pendiente=no` (4)
+  sin commit ni ref pendiente. Son operaciones de la rama de estado por el
+  punto único de escritura, con el `ROADMAP.md` del mismo commit; anotación
+  `liberar|TASK-NNN|<id>|<login>|<actividad o ->|<dias>|<ahora>` y
+  `reasignar|TASK-NNN|<id>|<login>|<id destino>|<login destino>|<actividad o
+  ->|<dias>|<ahora>`.
+- **Quién (D3a)**: el dueño siempre (también su agente); una tarea sin dueño,
+  cualquiera; la de otra persona de la lista, solo con `dias_sin_actividad`
+  días o más sin actividad (`rechazada motivo=activa dias=K umbral=N`) y
+  lanzada por una persona en un terminal (`requiere-persona`: el agente no
+  quita a nadie una tarea). Un `asignado_a` heredado de la 0.16 no espera los
+  días, pero también lo decide una persona.
+- **Actividad (D2a)**: la más reciente de la última fila de
+  `## Transiciones` (también `pausa`/`reanudar`: la pausa no exime) y del
+  último commit de `origin/<rama>` tras el `fetch`. Sin ninguna de las dos no
+  se sabe y no frena.
+- **`liberar` (D6a)**: `en-curso` → `planificada`, `en-diseno` y
+  `planificada` se quedan; `asignado_a: null`; `rama:` y `plan_aprobado`
+  igual; `en-revision` se rechaza (se reasigna). **`reasignar`**: solo
+  `asignado_a`, a una persona de `personas.yml` de la base (si no,
+  `fuera-de-lista`); conserva el estado.
+- **Trabajo sin publicar (D4a)**: si el worktree de la tarea en el clon que
+  lanza la orden tiene cambios sin guardar o commits sin publicar,
+  `rechazada motivo=sin-publicar` antes de la transacción; publicada, el
+  worktree se quita como en `descartar`. La rama no se toca nunca.
+- **Punto 4, la rama propia**: `start` ya no da `ocupada` por la rama que la
+  tarea tiene en su `rama:`; cualquier otra `*/task-nnn-*` sigue siendo
+  `ocupada`. La siguiente persona la retoma desde `origin/<rama>` con su
+  historia (`git::reanudar`).
+- **Riesgos aceptados**: la foto de actividad se toma antes de la
+  transacción (si el dueño publica entre el `fetch` y el push del estado, la
+  liberación sigue: la misma ventana que `comprobar-id` en `start`); el
+  aviso al dueño anterior es sin plataforma (D5a): la fila, el commit de
+  estado y `ya-no-tuyas=` en su `status`/`board`.
+
+Evidencia: `crates/tc2/tests/liberar.rs` (C1..C8, C11, C12 y
+`la_rama_propia_no_debilita_ocupada`), `crates/tc2/tests/tablero.rs`
+(`c9_c10_inactivas_y_ya_no_tuyas`) y los unitarios de `ciclo/liberar.rs`.
diff --git a/docs/adr/F1-3-flujo-de-ramas-worktrees-release.md b/docs/adr/F1-3-flujo-de-ramas-worktrees-release.md
index b5281dc..9786e9c 100644
--- a/docs/adr/F1-3-flujo-de-ramas-worktrees-release.md
+++ b/docs/adr/F1-3-flujo-de-ramas-worktrees-release.md
@@ -6,6 +6,7 @@
 - Depende de: [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (estado en `taskcode/tareas`, montada en `tareas/`; límite de un worktree por rama) y [ADR F1-2](F1-2-escrituras-atomicas-ids-y-toma.md) (punto único de escritura, re-ejecución en vez de rebase, `start` publica el `asignado` antes de crear la rama)
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a del plan de TASK-004)
 - Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
+- Adenda (2026-11-17, TASK-081, F3-10): cierra la pregunta abierta del worktree tras `liberar` o `reasignar`; ver [la adenda del final](#adenda-f3-10-task-081--el-worktree-tras-liberar-o-reasignar)
 
 ## Contexto
 
@@ -461,8 +462,9 @@ Este ADR se planificó con un solo rol (arquitectura). Quedan sin cubrir:
   ignorados) igual que borra `tareas/` (hueco abierto desde F1-1); lo
   commiteado y publicado lo recupera el `start` del dueño (PoC 1f), lo no
   publicado se pierde. F2-2/F2-3 tienen que avisar o protegerse.
-- **Worktree de tarea abierto tras `liberar` o `reasignar`** (F3-10): qué
-  pasa con el worktree y lo no publicado de quien la tenía.
+- ~~**Worktree de tarea abierto tras `liberar` o `reasignar`** (F3-10): qué
+  pasa con el worktree y lo no publicado de quien la tenía.~~ Cerrada por la
+  adenda de TASK-081 (abajo).
 - **Hotfix cuyo PR contra `develop` choca** con trabajo nuevo: en la PoC no
   chocó (tocaban líneas distintas); quién resuelve el conflicto y en qué rama
   (¿una rama de vuelta propia?) está sin decidir.
@@ -478,3 +480,30 @@ Este ADR se planificó con un solo rol (arquitectura). Quedan sin cubrir:
   quizá un `taskctl podar` para ramas de tareas terminadas hace más de N
   meses.
 - **Plataforma**: la PoC no usa GitHub ni GitLab; F1-5, F1-13 y F9-4.
+
+## Adenda (F3-10, TASK-081) — el worktree tras `liberar` o `reasignar`
+
+Cierra la pregunta abierta del worktree de quien tenía la tarea (D4a y D5a
+del plan de TASK-081, aceptadas por Carlos para todo el H3):
+
+- **La rama no se toca nunca**: ni la local ni `origin/<rama>`. La
+  siguiente persona hace `start` y sigue en ella desde `origin/<rama>`, con
+  la historia de la anterior (`reanudar`, caso c de la decisión 2; `start`
+  ya no da `ocupada` por la rama propia de la tarea, adenda de TASK-081 en
+  F1-2).
+- **Quien la deja desde su clon** (`liberar` o `reasignar` del dueño): con
+  cambios sin guardar, commits sin publicar o HEAD suelto en el worktree,
+  `rechazada motivo=sin-publicar` antes de escribir el estado; limpio y
+  publicado, el worktree se quita como en `descartar` (`worktree=quitado`).
+- **Quien la tenía en otro clon** (la libera o reasigna otra persona tras
+  N días sin actividad): su worktree y su rama local se quedan como están.
+  `status` y `board` en su clon le avisan `ya-no-tuyas=TASK-NNN` con quien la
+  lleva ahora (por su cache de identidad y `git worktree list`, sin
+  plataforma ni red). Lo que no publicó lo sigue teniendo él; nadie más lo
+  ve (riesgo aceptado: «sin perder su trabajo» solo vale para lo publicado).
+
+Evidencia: `crates/tc2/tests/liberar.rs`
+(`c8_trabajo_sin_publicar_no_se_suelta`,
+`c1_c2_el_dueno_libera_y_otra_persona_la_retoma`,
+`c5_c6_c10_tarea_de_otra_persona_por_actividad`) y
+`crates/tc2/tests/tablero.rs` (`c9_c10_inactivas_y_ya_no_tuyas`).
diff --git a/docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md b/docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md
index 41e805c..8df6dd0 100644
--- a/docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md
+++ b/docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md
@@ -7,6 +7,7 @@
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a y D5a del plan de TASK-010)
 - Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 - Adenda (2026-11-17, TASK-029): clave opcional `lote` para el import en bloque; ver [la adenda al final](#adenda-f3-2-task-029--clave-opcional-lote)
+- Adenda (2026-11-17, TASK-081): filas `liberar` y `reasignar`; `reasignar` resuelve los `asignado_a` heredados; ver [la adenda al final](#adenda-f3-10-task-081--filas-liberar-y-reasignar)
 
 ## Contexto
 
@@ -539,3 +540,27 @@ y sin `lote`), `crates/tc2/tests/tarea.rs` (`ida_y_vuelta_byte_a_byte`, sin
 cambios) y `crates/tc2/tests/importar.rs`
 (`c1_c3_importa_37_en_un_commit_y_repetir_no_crea_nada`,
 `c4_dos_entradas_mas_crean_solo_esas_dos`).
+
+## Adenda (F3-10, TASK-081) — filas `liberar` y `reasignar`
+
+Sin claves ni estados nuevos (la lista de estados sigue cerrada):
+
+- **Filas de `## Transiciones`**: `liberar` y `reasignar` añaden su fila con
+  la fase del mismo nombre, como `descartar` o `pausa`, y `actualizado` de
+  hoy; el resto del `tarea.md`, byte a byte.
+- **`liberar`** escribe `asignado_a: null` y, si estaba `en-curso`,
+  `estado: planificada` (lista para empezar); conserva `rama:` y
+  `plan_aprobado`. **`reasignar`** escribe solo `asignado_a` con el objeto
+  `{id, login}` de la persona de `personas.yml` y conserva el estado.
+- **Heredados**: `reasignar` es lo que resuelve un `asignado_a` heredado de
+  la 0.16 que no casa con ningún `correos:` (la nota de TASK-077 de arriba):
+  lo sustituye por el objeto de la persona elegida.
+- **Actividad**: la última fila fechada de `## Transiciones` (también
+  `pausa` y `reanudar`) cuenta como actividad de la tarea para F3-10, junto
+  al último commit de su rama publicada.
+
+Evidencia: `crates/tc2/tests/liberar.rs`
+(`c1_c2_el_dueno_libera_y_otra_persona_la_retoma`,
+`c3_reasignar_y_el_nuevo_sigue_en_la_rama`, `c12_heredado_queda_como_objeto`)
+y los unitarios de `ciclo/liberar.rs` y `tarea/cuerpo.rs`
+(`ultima_fecha_de_las_transiciones`).
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-081/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-081/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-081/tarea.md                                    | 35 +++++++++++++++++++++++++++++++++--
 3 files changed, 33 insertions(+), 2 deletions(-)
````

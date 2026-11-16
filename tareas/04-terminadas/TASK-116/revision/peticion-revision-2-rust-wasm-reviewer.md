# Peticion de revision — TASK-116 (ronda 2)

- Tarea: TASK-116 — Nombres de la 2.0 alineados con F1-12 D1a
- Rama revisada: fix/task-116-nombres-de-la-2-0-alineados-con-f1-12-d1
- Rama base: develop
- Commit revisado (HEAD): 727d2c560c1126a56d130b778cb830a46d211e21
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-116 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 92900522ace4f8328546b838b04358cb15a3d298 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (7 hallazgo(s)))

## Instrucciones para el agente revisor (verificacion acotada)

Eres un revisor INDEPENDIENTE del agente que implemento. La rama ya paso una
revision completa: esta ronda verifica las correcciones de la ronda anterior,
no vuelve a revisar la rama entera ni los MENORES de la ronda anterior (tu
tabla solo trae CRITICOS e IMPORTANTES). Haz esto, y nada mas:

1. Comprueba, uno por uno, que cada hallazgo pendiente de tu tabla de abajo
   queda resuelto por el diff de esta ronda (los marcados `corregido` tambien:
   verificalo, no lo des por bueno). Para un CRITICO o IMPORTANTE, reproduce el
   fallo original y comprueba que ya no ocurre; puedes leer el repo entero si la
   causa no esta en el diff.
2. Ejecuta la suite (o el comando de verificacion del proyecto) sobre el commit
   revisado y di el resultado: verificar sin ejecutar no vale. Si esta peticion
   ya trae la verificacion de la ronda en verde para ese mismo commit, no la
   repitas: cita ese resultado y corre solo los tests de lo que verificas.
3. Busca en el diff de esta ronda el fallo que la correccion abre: es justo
   donde se cuelan los fallos nuevos. Si encuentras uno, construye el caso que
   lo rompe antes de reportarlo.

Solo abre un hallazgo NUEVO si es CRITICO o si lo introduce la propia correccion
(algo que el diff de esta ronda rompe). Lo demas que veas fuera de eso va como
nota al final del informe, no en la tabla: no abre otra ronda.

Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. En la tabla, cada hallazgo pendiente que ya este
resuelto va como `corregido`; el que no, `abierto`. Vuelca tu salida en el
informe de esta ronda (informe-revision-2-rust-wasm-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-116/tarea.md
- tareas/03-en-revision/TASK-116/planificacion/
- crates/tc2/src/git/puerto.rs
- crates/tc2/tests/convivencia_016.rs
- docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md
- docs/adr/F1-10-carriles-y-modo-automatico.md
- docs/adr/F1-11-extensiones-declaradas.md
- docs/adr/F1-12-actualizacion-entre-versiones.md
- docs/adr/F1-16-lotes-desatendidos.md
- docs/adr/F1-5-puertas-en-github.md
- docs/adr/F1-6-identidades-agente-revisor-persona.md
- docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md
- docs/adr/F1-8-seguridad-modelo-de-amenazas.md
- docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md
- docs/adr/README.md

## Hallazgos de la ronda 1 pendientes de verificar

Tus 3 hallazgo(s) MENOR de la ronda 1 no se reverifican (los comprueba
quien coordina): recibes esta ronda porque la correccion
toca ficheros de tu dominio. Revisa en el diff de esta ronda lo que la correccion
cambia en tu terreno y si abre un fallo.

## Commits a revisar (git log 92900522ace4f8328546b838b04358cb15a3d298..HEAD)

````
727d2c5 chore(TASK-116): la ronda 1 pasa a cambios solicitados para revisar los ADR anadidos
75bb6b2 fix(TASK-116): marca como intencionado el literal del nombre viejo en puerto
76be757 fix(TASK-116): convivencia con la 0.16 tambien por el camino sin red
973642c docs(TASK-116): adenda de nombres en las cabeceras de los ADR aceptados
baf2a45 chore(TASK-116): veredictos y coste de la revision ronda 1
d2ac8b9 chore(TASK-116): informes de revision ronda 1
0097ddc chore(TASK-116): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 92900522ace4f8328546b838b04358cb15a3d298..HEAD)

````diff
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
index 927497d..7e71078 100644
--- a/crates/tc2/src/git/puerto.rs
+++ b/crates/tc2/src/git/puerto.rs
@@ -511,6 +511,7 @@ mod tests {
             // El prefijo viejo ya no es propio (TASK-116).
             (
                 Subcomando::UpdateRef,
+                // intencionado: caso de rechazo del nombre viejo
                 &["refs/taskcode/pendientes/000001", "abc"],
             ),
             (
diff --git a/docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md b/docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md
index ffe7761..3eafe1e 100644
--- a/docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md
+++ b/docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md
@@ -4,6 +4,7 @@
 - Tarea: TASK-002 · Plan: [F1-1 en PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) (§4 P1 y tabla de la Fase 1)
 - Requisitos: RR-10, RR-11, RR-12 (la parte de estado), RN-05 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md)
 - Decide: Carlos, con la consigna «la opción recomendada» (resuelve el `[DECIDE CARLOS]` de P1)
+- Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
 
diff --git a/docs/adr/F1-10-carriles-y-modo-automatico.md b/docs/adr/F1-10-carriles-y-modo-automatico.md
index a87baad..a702675 100644
--- a/docs/adr/F1-10-carriles-y-modo-automatico.md
+++ b/docs/adr/F1-10-carriles-y-modo-automatico.md
@@ -5,6 +5,7 @@
 - Requisitos: la fila F1-10 del plan cita P7; este ADR responde a RR-02, RR-04, RR-07, RR-08, RR-17, RR-18, RF-21, RF-27, RF-33, RF-50, RF-51 y RN-06 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md) (RR-18 viene de la [ampliación del 2026-11-14](../plan/AMPLIACION-2026-11-14.md)) y a H5 y H6 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
 - Depende de: [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md) (release y hotfix), [ADR F1-5](F1-5-puertas-en-github.md) (tabla de carriles, check `carril`, carril efectivo, errata solo con `docs/`), [ADR F1-6](F1-6-identidades-agente-revisor-persona.md) (operación → identidad, lista de firmantes en `.taskcode/`), [ADR F1-7](F1-7-verificacion-ci-perfiles-cache-runners.md) (perfil por evento), [ADR F1-8](F1-8-seguridad-modelo-de-amenazas.md) (`carril` en `pull_request_target`, D1b) y [ADR F1-9](F1-9-formato-de-tarea-plan-y-especificacion.md) (campo `carril`, modo congelado en la tarea)
 - Decide: Carlos, con la consigna «la opción recomendada» (D0a, D1a, D2a, D3a, D4a, D5a, D6a y D7a del plan de TASK-011) y «no trocear»
+- Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
 
diff --git a/docs/adr/F1-11-extensiones-declaradas.md b/docs/adr/F1-11-extensiones-declaradas.md
index 8504c9d..392f427 100644
--- a/docs/adr/F1-11-extensiones-declaradas.md
+++ b/docs/adr/F1-11-extensiones-declaradas.md
@@ -5,6 +5,7 @@
 - Requisitos: la fila F1-11 del plan cita P8; este ADR responde a RN-04, RF-07, RF-08, RF-40, RF-41, RF-42, RF-70, RF-71 y RR-11 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md) y a H2 y H16 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
 - Depende de: [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (carpeta fija, rama de estado huérfana, roadmap con un Mermaid por hito) y [ADR F1-7](F1-7-verificacion-ci-perfiles-cache-runners.md) (un comando con perfiles, huella del árbol y local, runner por perfil, job programado); cierra costuras abiertas por [F1-8](F1-8-seguridad-modelo-de-amenazas.md) (confianza local única, globs extra, `comando_*` como entrada no confiable, rutas de Windows en YAML), [F1-9](F1-9-formato-de-tarea-plan-y-especificacion.md) (claves desconocidas del frontmatter) y [F1-10](F1-10-carriles-y-modo-automatico.md) (configuración que falla cerrada, tope de 12 claves); cambia la lista de checks obligatorios de [F1-5](F1-5-puertas-en-github.md) (ver «Divergencias con F1-5 y F1-7»)
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a, D5a y D6a del plan de TASK-012, y las correcciones de la revisión 1), «no trocear», la restricción de que el plugin sea **eficiente y con ventajas medibles** y prefiera **comandos deterministas** a hooks, y la decisión del 2026-11-14: «todo lo que hace funcionar al plugin 2 debe estar en su repo y debe ser generalista; al actualizar a la 2 hay que quitar los parches que se aplicaron para mejorar la 0.16 dentro del repo»
+- Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
 
diff --git a/docs/adr/F1-12-actualizacion-entre-versiones.md b/docs/adr/F1-12-actualizacion-entre-versiones.md
index 2f19974..0168ee3 100644
--- a/docs/adr/F1-12-actualizacion-entre-versiones.md
+++ b/docs/adr/F1-12-actualizacion-entre-versiones.md
@@ -549,10 +549,11 @@ Las órdenes que esos ADR nombran para la 2.0 (`taskctl actualizar`,
 La nota anterior solo cubría la rama y `.git/taskcode`. Al implementar
 F2-2..F2-13 quedaron en el código otros nombres que chocan con la 0.16 o no
 se reconocen como de la 2.0; TASK-116 los alinea con D1a. Son la **única
-lectura válida** de F1-2, F1-3, F1-4 y F1-16 donde nombran estas piezas, y
-salen todos de un único módulo, `crates/tc2/src/huella.rs`:
+lectura válida** de F1-1, F1-2, F1-3, F1-4, F1-5, F1-6, F1-7, F1-8, F1-9,
+F1-10, F1-11 y F1-16 donde nombran estas piezas (cada uno lo dice en su
+cabecera), y salen todos de un único módulo, `crates/tc2/src/huella.rs`:
 
-| Pieza | Nombre en F1-2/F1-3/F1-4/F1-16 y en el código anterior | Nombre de la 2.0 |
+| Pieza | Nombre en esos ADR y en el código anterior | Nombre de la 2.0 |
 |---|---|---|
 | Rama de estado | `taskcode/tareas` | **`taskcodetwo/tareas`** |
 | Cerrojo local del clon | `.git/taskcode-estado.lock` | **`.git/taskcodetwo/estado.lock`** |
diff --git a/docs/adr/F1-16-lotes-desatendidos.md b/docs/adr/F1-16-lotes-desatendidos.md
index 81b0c3c..1b3fd08 100644
--- a/docs/adr/F1-16-lotes-desatendidos.md
+++ b/docs/adr/F1-16-lotes-desatendidos.md
@@ -6,6 +6,7 @@
 - Requisitos: RR-18 y RF-27 ([catálogo](../plan/REQUISITOS-FUNCIONALES.md)); en lo que toca, RR-01 (el CLI decide, no el agente), RR-08 (garantías de la plataforma), RR-09 y RR-12 (equipo) y RF-33 (tope de rondas)
 - Depende de: [ADR F1-2](F1-2-escrituras-atomicas-ids-y-toma.md) (CAS en la rama de estado y toma), [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md) (prefijo reservado `lote/*`, worktrees), [ADR F1-4](F1-4-operaciones-git-y-contrato-de-salida.md) (lista cerrada de operaciones), [ADR F1-5](F1-5-puertas-en-github.md) (rulesets, D2a), [ADR F1-6](F1-6-identidades-agente-revisor-persona.md) (identidades, decisión 4), [ADR F1-8](F1-8-seguridad-modelo-de-amenazas.md) (A5, A16), [ADR F1-10](F1-10-carriles-y-modo-automatico.md) (columna «Lote», `tope_rondas`), [ADR F1-11](F1-11-extensiones-declaradas.md) (derivados en la rama de estado, D2a), [ADR F1-13](F1-13-adaptador-de-plataforma.md) (garantías G01–G20) y [ADR F1-15](F1-15-seleccion-de-skills-agentes-y-modelo.md) (`seleccionar`)
 - Decide: Carlos, el 2026-11-14: RR-18 imprescindible (comando determinista del CLI, `lote/<nombre>` desde `develop`, solo carriles permitidos, apartar lo rojo con sus dependientes, nunca integrar en rojo, presupuesto, `--reanudar` sin repetir, informe y equivalente en GitLab); seguridad y trabajo en grupo imprescindibles, con su caso real (escribir en el prompt mientras una tarea trabajaba cambió el repo y provocó rondas extra); comandos deterministas sobre hooks; nombres `tc2`, `taskcodetwo` y rama de estado `taskcodetwo/tareas`; sin trocear. Respuestas al plan de TASK-074: **D1a a D8a**; D2a, D6a y D7a se le preguntaron expresamente por cambiar su forma de trabajar y respondió «Apruebo».
+- Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 Este ADR fija **cómo corre un lote desatendido**: `tc2 lote` es un bucle
 del CLI que, en cada vuelta, reconstruye los hechos desde Git (la cadena de
diff --git a/docs/adr/F1-5-puertas-en-github.md b/docs/adr/F1-5-puertas-en-github.md
index 4e8a805..44181e0 100644
--- a/docs/adr/F1-5-puertas-en-github.md
+++ b/docs/adr/F1-5-puertas-en-github.md
@@ -5,6 +5,7 @@
 - Requisitos: RR-04, RR-07, RR-08, RR-15, RF-50, RF-51, RF-64 y RN-06 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); H5 y H6 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
 - Depende de: [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (rama de estado con push directo del CLI), [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md) (nombres de rama, merge commit, ramas conservadas, tags `vX.Y.Z`) y [ADR F1-4](F1-4-operaciones-git-y-contrato-de-salida.md) (puerto `Plataforma` y su lista cerrada)
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a, D5a y D6a del plan de TASK-006)
+- Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
 
diff --git a/docs/adr/F1-6-identidades-agente-revisor-persona.md b/docs/adr/F1-6-identidades-agente-revisor-persona.md
index 975b30a..594cbd6 100644
--- a/docs/adr/F1-6-identidades-agente-revisor-persona.md
+++ b/docs/adr/F1-6-identidades-agente-revisor-persona.md
@@ -5,6 +5,7 @@
 - Requisitos: RR-04, RR-08, RR-12, RR-15, RF-30, RF-32, RF-51, RF-64, RN-01 y RN-06 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); H5 y H6 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
 - Depende de: [ADR F1-4](F1-4-operaciones-git-y-contrato-de-salida.md) (puerto `Plataforma` y su lista cerrada) y [ADR F1-5](F1-5-puertas-en-github.md) (check `carril`, firma verificada del `approve` atada al blob con el trailer `Plan-Blob`, `integration_id` de `revision-ia`, CODEOWNERS, `strict`, sin `bypass_actors`); nombres de rama y tags de [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md)
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a —FIDO si hay llave; si no, D4b—, D5a y D6a del plan de TASK-007)
+- Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
 
diff --git a/docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md b/docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md
index 84eb4a9..7b2f7ab 100644
--- a/docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md
+++ b/docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md
@@ -5,6 +5,7 @@
 - Requisitos: RR-07, RR-15, RF-41, RF-42 y RN-02 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); las cuatro viñetas de P6 del plan; H2, H16 y H18 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
 - Depende de: [ADR F1-5](F1-5-puertas-en-github.md) (check `verificacion` obligatorio atado a GitHub Actions, `integration_id` 15368; `strict` en `develop` y no en `main`), [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md) (merge commit sin fast-forward, release por merge de `develop` en `main`), [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (el roadmap y las tareas fuera del árbol de código) y [ADR F1-6](F1-6-identidades-agente-revisor-persona.md) (el agente lee checks y no los escribe; no toca workflows)
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a, D5a y D6a del plan de TASK-008)
+- Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
 
diff --git a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
index 20deeaa..e42c706 100644
--- a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
+++ b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
@@ -5,6 +5,7 @@
 - Requisitos: la fila F1-8 del plan no cita ninguno; este ADR responde a RR-03, RR-08, RR-13, RR-15, RR-18, RF-70, RN-02 y RN-06 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md), a las viñetas 3 y 4 de P6 del plan y a H2 y H9 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
 - Depende de: [ADR F1-5](F1-5-puertas-en-github.md) (checks `carril`, `verificacion` y `revision-ia`, `integration_id` 15368, CODEOWNERS en `/.github/` y `/.taskcode/`), [ADR F1-6](F1-6-identidades-agente-revisor-persona.md) (dos Apps, el agente sin `workflows` ni `secrets`, la clave del revisor en Actions —D5a—, firma de la persona con FIDO o frase sin caché) y [ADR F1-7](F1-7-verificacion-ci-perfiles-cache-runners.md) (caché por árbol en el resumen del check, runner por perfil, runner propio solo en privados, hook con la rápida)
 - Decide: Carlos, con la consigna «la opción recomendada» (D1b, D2b, D3b, D4b y D5a del plan de TASK-009)
+- Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
 
diff --git a/docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md b/docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md
index c6a119c..ff75790 100644
--- a/docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md
+++ b/docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md
@@ -5,6 +5,7 @@
 - Requisitos: la fila F1-9 del plan no cita ninguno; este ADR responde a RR-10, RR-13, RR-14, RR-17, RR-19, RF-01, RF-07, RF-22, RF-34, RF-51, RF-62 y RF-71 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md) y a H11 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
 - Depende de: [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (carpeta fija `TASK-NNN/`, estado como campo), [ADR F1-2](F1-2-escrituras-atomicas-ids-y-toma.md) (ID y toma con el «quién»), [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md) (nombre de rama), [ADR F1-5](F1-5-puertas-en-github.md) (decisión 4: la aprobación es el commit firmado con `Plan-Blob` sobre el blob de `plan-final.md`) y [ADR F1-8](F1-8-seguridad-modelo-de-amenazas.md) (zona Z2 y activo `A10`: el texto de las tareas es dato no confiable)
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a y D5a del plan de TASK-010)
+- Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
 
diff --git a/docs/adr/README.md b/docs/adr/README.md
index 1201cc1..0f04fa8 100644
--- a/docs/adr/README.md
+++ b/docs/adr/README.md
@@ -8,7 +8,7 @@ directo a la tabla de la Fase 1 y a su tarea.
 
 | ADR | Tema | Tarea | Estado |
 |---|---|---|---|
-| [F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) | Carpeta fija por tarea y rama de estado `taskcode/tareas` | TASK-002 | aceptada |
+| [F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) | Carpeta fija por tarea y rama de estado `taskcodetwo/tareas` (nombre de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)) | TASK-002 | aceptada |
 | [F1-2](F1-2-escrituras-atomicas-ids-y-toma.md) | Escrituras atómicas en la rama de estado, reserva de IDs y toma de tareas | TASK-003 | aceptada |
 | [F1-3](F1-3-flujo-de-ramas-worktrees-release.md) | Flujo de ramas `develop`/`main`, rama por tarea con worktree, release y hotfix; qué queda de Git-Flow | TASK-004 | aceptada |
 | [F1-4](F1-4-operaciones-git-y-contrato-de-salida.md) | Capa de operaciones Git/GitHub deterministas: lista cerrada, contrato de salida, códigos y objetivo de tokens por tarea | TASK-005 | aceptada |
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 92900522ace4f8328546b838b04358cb15a3d298..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-116/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-116/planificacion/plan-final.md                 |   0
 tareas/03-en-revision/TASK-116/revision/informe-revision-1-code-quality-reviewer.md         |  88 +++++++++++++
 tareas/03-en-revision/TASK-116/revision/informe-revision-1-rust-wasm-reviewer.md            |  94 ++++++++++++++
 tareas/03-en-revision/TASK-116/revision/peticion-revision-1-code-quality-reviewer.md        | 181 +++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-116/revision/peticion-revision-1-rust-wasm-reviewer.md           | 677 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-116/tarea.md                                    |  16 ++-
 7 files changed, 1053 insertions(+), 3 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 92900522ace4f8328546b838b04358cb15a3d298..HEAD -- ":(literal)crates/tc2/tests/convivencia_016.rs"
````

````
crates/tc2/tests/convivencia_016.rs | 41 ++++++++++++++++++++++++++++++++++++-----
 1 file changed, 36 insertions(+), 5 deletions(-)
````

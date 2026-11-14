# Peticion de revision — TASK-013 (ronda 3)

- Tarea: TASK-013 — F1-12 ADR actualizacion de proyectos entre versiones
- Rama revisada: feature/task-013-f1-12-adr-actualizacion-de-proyectos-ent
- Rama base: develop
- Commit revisado (HEAD): 3b62dcff555fdf7c29e9f5763c0a4d8693e905be
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-013 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde b706b33020e14c52351e27cd2167c8d2e42b016e (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (4 hallazgo(s)))

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
informe de esta ronda (informe-revision-3.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-013/tarea.md
- tareas/03-en-revision/TASK-013/planificacion/
- docs/adr/F1-12-actualizacion-entre-versiones.md

## Hallazgos de la ronda 2 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 2 hallazgo(s) MENOR de la ronda 2 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-12-actualizacion-entre-versiones.md:277 | informe-revision-2.md |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-12-actualizacion-entre-versiones.md:476 | informe-revision-2.md |

## Commits a revisar (git log b706b33020e14c52351e27cd2167c8d2e42b016e..HEAD)

````
3b62dcf chore(TASK-013): ronda extra 3
f08b409 chore(TASK-013): informe de revision 2 y constancia de autorrevision ronda 3
d4799bc docs(adr): F1-12 corrige los hallazgos de la revision 2 (aprobacion de nombres registrada, comillas, guard bajo LOCALAPPDATA)
7fd5b8d chore(TASK-013): hallazgos de la ronda 1 marcados como corregidos en 3c44c18
f448fc1 chore(TASK-013): peticion de revision ronda 2
````

## Diff desde la ronda anterior (git diff b706b33020e14c52351e27cd2167c8d2e42b016e..HEAD)

````diff
diff --git a/docs/adr/F1-12-actualizacion-entre-versiones.md b/docs/adr/F1-12-actualizacion-entre-versiones.md
index 5c0432b..3862255 100644
--- a/docs/adr/F1-12-actualizacion-entre-versiones.md
+++ b/docs/adr/F1-12-actualizacion-entre-versiones.md
@@ -133,7 +133,8 @@ tareas tras migrar. La PoC fuerza la migración para medir el resto.
 El `subtree split` recorre los 1350 commits de `develop` y deja 1151 en la
 rama de estado. Es el paso caro de la migración y su tiempo de reloj varía
 entre ejecuciones en la misma máquina, así que no entra en la tabla
-comprobada. Rango observado: 594 s en la entrega, 614 s en la reejecución
+comprobada. Rango observado: 594 s en la entrega (su log solo guardó el
+cubo «menos de 600 s», línea 29 del log del commit 7f431ff), 614 s en la reejecución
 de la revisión 1 y 647 s en la ejecución que deja el log de esta
 corrección (su stderr va junto al log, en `poc-f1-12.tiempos.log`).
 
@@ -396,7 +397,7 @@ algo: sin `tareas-json` la migración se niega (código y ficheros cambiados:
    | `tareas/NN-estado/.gitkeep` | se borran (las carpetas de estado desaparecen) |
    | frontmatter de la 0.16 | formato 2 de F1-9: `formato: 2`, cadenas y fechas entre comillas dobles, `modelo_sugerido` → `roles.implementacion.modelo`, `skills_recomendados` y `regla_seleccion_skill` → `roles.implementacion`, `agente_revisor` → `roles.revision.agente`; fuera `revision_codex` y `ultimo_commit_revisado`; `carril` y `estimacion` a `null`; las claves desconocidas se conservan (F1-9 D4a). El cuerpo, byte a byte |
    | `tareas/ROADMAP.md` | se lleva tal cual a la raíz de la rama de estado; la 2.0 lo regenera con el contenido de F1-1 en su primera transición (F3-4) |
-   | `.taskcode/config.yml` plana | la misma ruta (D4a) con `---` en la primera línea, `formato: 2`, las claves base de F2-7 y los bloques de F1-11 (`verificacion`, `derivados`, `confianza_incluye`); los valores de la 0.16 pasan tal cual (`comando_verificacion` → `verificacion.comando`, `timeout_verificacion` → `perfiles.completa.timeout`) y lo que la 0.16 no tenía lo declara el proyecto (F1-11, «valores a rellenar»). **Comillas**: las claves que la 0.16 también lee (`modo_flujo` y las demás claves base heredadas) van entre **comillas dobles**, como pide F1-9 para las cadenas y porque la 0.16 quita las dobles pero conserva las simples dentro del valor (variante D de la PoC C); las órdenes y rutas de los bloques nuevos, que la 0.16 no lee, van entre **simples** o con `/`, como fija F1-11 (decisión 1), para que el `\v` de una ruta de Windows no sea un escape |
+   | `.taskcode/config.yml` plana | la misma ruta (D4a) con `---` en la primera línea, `formato: 2`, las claves base de F2-7 y los bloques de F1-11 (`verificacion`, `derivados`, `confianza_incluye`); los valores de la 0.16 pasan tal cual (`comando_verificacion` → `verificacion.comando`, `timeout_verificacion` → `perfiles.completa.timeout`) y lo que la 0.16 no tenía lo declara el proyecto (F1-11, «valores a rellenar»). **Comillas**: las claves que la 0.16 también lee (hoy solo `modo_flujo`) van entre **comillas dobles**, porque la 0.16 quita las dobles pero conserva las simples dentro del valor (variante D de la PoC C); es el mismo criterio que F1-9 fija para las cadenas del frontmatter de `tarea.md`, aplicado aquí por analogía a `config.yml`; las órdenes y rutas de los bloques nuevos, que la 0.16 no lee, van entre **simples** o con `/`, como fija F1-11 (decisión 1), para que el `\v` de una ruta de Windows no sea un escape |
    | `.gitignore` | añade `tareas/` |
    | `.git/taskcode/` (estado interno de la 0.16) | no migra: lo borra la desinstalación de la 0.16 (F9-10) |
 
@@ -473,7 +474,7 @@ algo: sin `tareas-json` la migración se niega (código y ficheros cambiados:
      desinstalar la 0.16 (F9-10); nunca con una tarea en curso; opcional y
      por proyecto.
 
-7. **Nombres definitivos** (D1a), aprobados por Carlos el 2026-11-14.
+7. **Nombres definitivos** (D1a), aprobados por Carlos el 2026-11-14 en la conversación de la tarea; queda registrado en la sección «Decisiones de la persona» del `tarea.md` de TASK-013.
    Distintos de los de la 0.16 y los mismos tras desinstalarla:
 
    | Pieza | 0.16 | 2.0 |
````

## Excluido del diff (5 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff b706b33020e14c52351e27cd2167c8d2e42b016e..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/03-en-revision/TASK-013/poc/poc-f1-12.py                |   6 +++
 tareas/03-en-revision/TASK-013/revision/informe-revision-1.md  |  14 +++----
 tareas/03-en-revision/TASK-013/revision/informe-revision-2.md  |  99 +++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-013/revision/peticion-revision-2.md | 263 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-013/tarea.md                        |  10 +++++
 5 files changed, 385 insertions(+), 7 deletions(-)
````

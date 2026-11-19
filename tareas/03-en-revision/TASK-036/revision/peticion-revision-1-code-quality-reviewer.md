# Peticion de revision — TASK-036 (ronda 1)

- Tarea: TASK-036 — F4-1 plan con brainstorm de roles
- Rama revisada: feature/task-036-f4-1-plan-con-brainstorm-de-roles
- Rama base: develop
- Commit revisado (HEAD): 6ecf89825e4a41e13abdd7f70a2fa08bcbf65ff6
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-036 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-036/tarea.md
- tareas/03-en-revision/TASK-036/planificacion/
- docs/adr/F1-10-carriles-y-modo-automatico.md

## Commits a revisar (git log develop..HEAD)

````
6ecf898 chore(TASK-036): autorrevision
dbe7b5f feat(TASK-036): tc2 plan con brainstorm de roles y tc2 plan guardar
a963125 chore(TASK-036): tarea en curso
````

## Diff de tu dominio (1 fichero(s) de 14; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-10-carriles-y-modo-automatico.md b/docs/adr/F1-10-carriles-y-modo-automatico.md
index a702675..60cfa04 100644
--- a/docs/adr/F1-10-carriles-y-modo-automatico.md
+++ b/docs/adr/F1-10-carriles-y-modo-automatico.md
@@ -691,3 +691,36 @@ Este ADR se planificó con un solo rol (arquitectura); quedan sin cubrir:
 - La salida de `siguiente --json` con los campos nuevos y lo que pinta el
   check `carril`: diseño, sin implementación (F4-4, F6).
 - `taskctl doctor` no se ejecutó en la implementación de esta tarea.
+
+## Adenda (F4-1, TASK-036) — `tc2 plan` y `tc2 plan guardar`
+
+`plan` congela el carril declarado y el modo y lanza los roles de la columna
+«Roles de diseño» (`Carril::roles_diseno` en `crates/tc2/src/carril.rs`).
+Lo que el plan de TASK-036 no fijaba, o fija de otra forma:
+
+- **Orden del tope de `hotfix` y de la configuración**: el tope de 1 rol de
+  un `hotfix` (fuera de riesgo alto) se aplica antes y
+  `politica_carriles.<carril>.roles_diseno` después, porque la
+  configuración solo endurece (decisión 2). El plan los citaba al revés.
+- **`roles_diseno: 5`**: el esquema admite hasta 5 y la escalera tiene 4; el
+  quinto es el condicional (rendimiento) aunque ninguna etiqueta lo pida.
+- **Modo congelado**: el de la última fila `plan` de `## Transiciones`; la
+  primera vez, `modo_flujo` de la base. No hay campo nuevo en el
+  frontmatter.
+- **Rondas**: solo se añaden (la rama de estado no borra). Una ronda nueva se
+  abre cuando cambia el juego de roles (por ejemplo, al subir el carril con
+  `--carril`); repetir `plan` con los mismos roles y el mismo carril es
+  `sin-cambios`. Volver a planificar con los mismos roles tras «pedir
+  cambios» queda para `approve` (TASK-037), que tendrá que cerrar la ronda.
+  `plan-final.md` solo se escribe si no existe: el esqueleto no se rehace al
+  cambiar de roles.
+- **Sin catálogo**: `plan` y `plan guardar` exigen red y nunca quedan
+  pendientes (como `import`), así que no se reconstruyen desde su anotación.
+- **Rechazos añadidos**: `plan` rechaza una tarea en pausa (`pausada`) y los
+  huecos de la plantilla cuentan como objetivo o criterio vacíos; `plan
+  guardar` exige la petición del rol cuya salida guarda (`sin-peticion`),
+  rechaza un fichero vacío y pasa CRLF a LF. Su línea es `plan ...
+  destino=...`.
+
+Evidencia: `crates/tc2/tests/plan.rs` y los unitarios de `carril.rs`
+(`roles_de_diseno_por_carril`) y de `crates/tc2/src/plan/`.
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-036/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-036/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-036/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````

# Peticion de revision — TASK-128 (ronda 1)

- Tarea: TASK-128 — Seguimiento de F3-11 y F3-12: aviso de solape contra uno mismo, textos al catalogo y test intermitente de progreso
- Rama revisada: fix/task-128-seguimiento-de-f3-11-y-f3-12-aviso-de-so
- Rama base: develop
- Commit revisado (HEAD): 6fdb49100d3066da1c76a255b171cb8ed232d052
- Fecha: 2026-11-18
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-128 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-128/tarea.md
- tareas/03-en-revision/TASK-128/planificacion/
- docs/adr/F1-17-progreso-y-estimacion.md

## Commits a revisar (git log develop..HEAD)

````
6fdb491 chore(TASK-128): autorrevision
78f965e fix(TASK-128): el anterior del progreso es el fichero que habia, byte a byte
9df6b42 fix(TASK-128): sin aviso de solape contra uno mismo y textos al catalogo
06a1d7d chore(TASK-128): tarea en curso
````

## Diff de tu dominio (1 fichero(s) de 10; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-17-progreso-y-estimacion.md b/docs/adr/F1-17-progreso-y-estimacion.md
index eb31b90..624580f 100644
--- a/docs/adr/F1-17-progreso-y-estimacion.md
+++ b/docs/adr/F1-17-progreso-y-estimacion.md
@@ -29,6 +29,11 @@
   - **Módulo = carpeta padre (D1a)**: dos tareas solapan si tocan el mismo fichero o la misma carpeta padre de un fichero; los ficheros de la raíz no tienen módulo (`Cargo.toml` frente a `README.md` no avisa; el mismo `Cargo.toml` sí). Un módulo en el que ya hay un fichero común no se repite como cruce de módulo. Da falsos positivos en carpetas con muchos ficheros: es un aviso, nunca una garantía (D4a).
   - **Cálculo local tras el `fetch` de `start`**: sin red, con lo que sabe el clon (la misma frescura que la toma). Un `for-each-ref` de `refs/remotes/origin/`, el `git diff --name-only -z --no-renames <base>...<rama>` de la rama propia (`refs/heads/<rama>`, ya creada) y uno por cada otra tarea con `origin/<rama>`: **1 + k + 1 procesos de git**. Las tareas en curso salen de la misma lectura de todos los `tarea.md` que ya hace la toma para el histórico (sin otro `cat-file`); cuando `start` no escribe (reanudar), de un `cat-file --batch` sobre el estado leído. Sin ninguna otra tarea en curso no se lanza nada; sin ninguna ref de `origin` (repo sin remoto) no sale ninguna clave.
   - **Salida (D3b, D4a, D5a)**: solo en `start`, con código 0 y el resultado de siempre. Claves, listas planas `TASK-NNN:valor` (`:` no se escapa en un valor de lista, decisión 4 de F1-4, y el ID no lo lleva: el primer `:` separa): `solape`, `solape-ficheros`, `solape-modulos`, `solape-asignados`, `solape-sin-rama` (en curso sin `origin/<rama>`: no se sabe qué toca), `solape-incompleto` (un `diff` que falló; no está en D5a, se añade para no callar un fallo) y, en el primer `start` con la rama propia sin cambios, `en-curso-modulos` (como mucho 5 módulos por tarea). Cada lista, como mucho 20 elementos, y `solape-truncado=si` si se cortó alguna. Para la persona, en stderr y fuera del JSON (segunda excepción a «diagnóstico solo si falla» de F1-4, como el aviso de `new`): una línea `aviso:` por tarea que cruza, con su ID, quién la lleva y los ficheros o módulos; en el primer `start`, una línea `nota:` con las tareas en curso con rama publicada, su @ y sus módulos. Login, rutas e IDs pasan por `tablero::sanear` (punto 8). `review` y `status` podrán llamar a `solape::calcular`, que es pública; hoy no lo hacen (D4a).
+- Adenda (2026-11-18, TASK-128; decisiones D1a a D5a de su plan, con la consigna de Carlos «la opción recomendada» para las cuatro tareas del H0 antes del H4):
+  - **Nadie se avisa contra sí mismo (restringe el punto 11)**: el solape de `start` ya no cruza las tareas en curso o en revisión que lleva la misma persona que toma la tarea (por id de cuenta, `Asignado::misma_persona`, como decide la toma si una tarea es suya). No salen en el aviso, ni en la `nota:` del primer `start`, ni en `en-curso-modulos`, ni en `solape-sin-rama`. Las sin asignar y las heredadas de la 0.16 siguen contando. El trabajo en paralelo de una misma persona en varios clones o worktrees (F9-4) no se avisa.
+  - **La rotación del anterior ya no se calla (F3-12)**: si el progreso que había no pasa a `.anterior.json` (no se pudo leer entero o no se pudo escribir; bajo carga en Windows, un lector o el antivirus con el fichero abierto), el progreso nuevo se escribe igual (R1-04 de TASK-084) y la línea de la transición lleva `progreso=sin-anterior`, sin stderr y sin cambiar el código ni el resultado.
+  - **El anterior es el fichero que había, byte a byte (divergencia con el plan)**: la causa del `progreso::c3` intermitente no era un `rename` bloqueado, como suponía el plan, sino que la rotación escribía el progreso leído y vuelto a serializar. `serde_json` sin `float_roundtrip` no devuelve igual un `f64` en cerca del 13 % de los valores probados, así que `transcurrido` podía cambiar en la última cifra. Ahora se copian los bytes del fichero (si son un progreso válido). Con eso, `c3` pasó 10 de 10 bajo carga; antes, 9 de 10.
+  - Los textos `AVISO_Y_MAS` y `NOTA_Y_MAS` del solape pasan al catálogo de `toma`.
 
 Este ADR fija **qué estima la 2.0, con qué datos, cuándo lo recalcula y dónde
 lo enseña sin gastar tokens**. Hay un solo lector del histórico,
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-128/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-128/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-128/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````

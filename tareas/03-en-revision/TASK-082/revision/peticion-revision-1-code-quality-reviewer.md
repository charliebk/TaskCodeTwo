# Peticion de revision — TASK-082 (ronda 1)

- Tarea: TASK-082 — F3-11 Aviso de solape entre tareas en curso
- Rama revisada: feature/task-082-f3-11-aviso-de-solape-entre-tareas-en-cu
- Rama base: develop
- Commit revisado (HEAD): 1866457c7b052bf73ffa8205973f0d364b28b111
- Fecha: 2026-11-18
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-082 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-082/tarea.md
- tareas/03-en-revision/TASK-082/planificacion/
- docs/adr/F1-17-progreso-y-estimacion.md

## Commits a revisar (git log develop..HEAD)

````
1866457 docs(TASK-082): autorrevision con rust-wasm-reviewer y code-quality-reviewer
277a5b9 docs(TASK-082): adenda de F1-17 con las divergencias del solape sin zonas
57bd1fb test(TASK-082): solape en start con dos clones y remoto bare (C1 a C10)
7cece03 feat(TASK-082): aviso y nota de solape en stderr, saneados y con catalogo
14bf30a feat(TASK-082): start calcula el solape y anade las claves solape a la linea
6516ff0 feat(TASK-082): ficheros_de_rama con un solo git diff --name-only sin red
d4f04e5 feat(TASK-082): nucleo puro del solape (modulo, cruzar y tareas en curso)
25f3d0a chore(TASK-082): tarea en curso
````

## Diff de tu dominio (1 fichero(s) de 10; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-17-progreso-y-estimacion.md b/docs/adr/F1-17-progreso-y-estimacion.md
index c64e9bd..eb31b90 100644
--- a/docs/adr/F1-17-progreso-y-estimacion.md
+++ b/docs/adr/F1-17-progreso-y-estimacion.md
@@ -24,6 +24,11 @@
   - **Saturada (R1-05 de la revisión 1 de TASK-084)**: pasados los 480 min de la tabla, lo que falta es el del último punto y la línea añade `estimacion saturada: mas de 480 min en la fase`; la tabla no se alarga (más de 8 h activas en una fase ya no se parece a sus parecidas y el fichero crecería sin aportar).
   - **Instalación (D2A)**: `tc2 init` solo propone la `statusLine` (línea `clase=status-line decision=pendiente motivo=pegar-a-mano orden=tc2 estado --stdin`, con el bloque JSON en `--json`); nunca lee ni escribe un `settings.json`, así que no sabe si ya está pegada y la propone en cada ejecución como `pendiente` (no `persona`: la segunda ejecución de `init` no pide nada nuevo, C2 de TASK-028). Se recomienda pegarla en `.claude/settings.local.json` del proyecto para no sustituir la de la 0.16 en `~/.claude/settings.json`; `tc2 desinstalar` solo reconoce la de `~/.claude/settings.json`.
   - **Lote (D3A)**: queda en F4-9; el fichero lleva `version` para añadirlo. **Tarea real (D4A)**: se acepta con el E2E de las transiciones existentes; la comprobación durante una tarea real queda para F9-7.
+- Adenda (2026-11-18, TASK-082, F3-11; decisiones D1a, D2a, D3b, D4a y D5a de su plan, con la consigna del H3 «la opción recomendada»). Divergencias con el punto 11 y con el plan:
+  - **Sin zonas hasta F4-1**: Carlos decidió el 2026-11-18 hacer F3-11 **sin depender de F4-1** (`plan`, que aún no existe) y se quitó esa dependencia de la tarea. Sin zonas declaradas, el solape **solo cruza el diff real**: los ficheros que cambia la rama propia frente a los de cada otra tarea en curso o en revisión (D2a; una `terminada` con su rama conservada no cuenta) con la rama publicada en `origin`. Lo que F3-11 llama «según sus planes» queda fuera; el punto de extensión es `solape::Toca` (ganará las `zonas`) y `solape::cruzar` (ganará la regla de prefijo del punto 11).
+  - **Módulo = carpeta padre (D1a)**: dos tareas solapan si tocan el mismo fichero o la misma carpeta padre de un fichero; los ficheros de la raíz no tienen módulo (`Cargo.toml` frente a `README.md` no avisa; el mismo `Cargo.toml` sí). Un módulo en el que ya hay un fichero común no se repite como cruce de módulo. Da falsos positivos en carpetas con muchos ficheros: es un aviso, nunca una garantía (D4a).
+  - **Cálculo local tras el `fetch` de `start`**: sin red, con lo que sabe el clon (la misma frescura que la toma). Un `for-each-ref` de `refs/remotes/origin/`, el `git diff --name-only -z --no-renames <base>...<rama>` de la rama propia (`refs/heads/<rama>`, ya creada) y uno por cada otra tarea con `origin/<rama>`: **1 + k + 1 procesos de git**. Las tareas en curso salen de la misma lectura de todos los `tarea.md` que ya hace la toma para el histórico (sin otro `cat-file`); cuando `start` no escribe (reanudar), de un `cat-file --batch` sobre el estado leído. Sin ninguna otra tarea en curso no se lanza nada; sin ninguna ref de `origin` (repo sin remoto) no sale ninguna clave.
+  - **Salida (D3b, D4a, D5a)**: solo en `start`, con código 0 y el resultado de siempre. Claves, listas planas `TASK-NNN:valor` (`:` no se escapa en un valor de lista, decisión 4 de F1-4, y el ID no lo lleva: el primer `:` separa): `solape`, `solape-ficheros`, `solape-modulos`, `solape-asignados`, `solape-sin-rama` (en curso sin `origin/<rama>`: no se sabe qué toca), `solape-incompleto` (un `diff` que falló; no está en D5a, se añade para no callar un fallo) y, en el primer `start` con la rama propia sin cambios, `en-curso-modulos` (como mucho 5 módulos por tarea). Cada lista, como mucho 20 elementos, y `solape-truncado=si` si se cortó alguna. Para la persona, en stderr y fuera del JSON (segunda excepción a «diagnóstico solo si falla» de F1-4, como el aviso de `new`): una línea `aviso:` por tarea que cruza, con su ID, quién la lleva y los ficheros o módulos; en el primer `start`, una línea `nota:` con las tareas en curso con rama publicada, su @ y sus módulos. Login, rutas e IDs pasan por `tablero::sanear` (punto 8). `review` y `status` podrán llamar a `solape::calcular`, que es pública; hoy no lo hacen (D4a).
 
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
tareas/{01-en-diseno => 02-en-curso}/TASK-082/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-082/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-082/tarea.md                                    | 32 ++++++++++++++++++++++++++++++--
 3 files changed, 30 insertions(+), 2 deletions(-)
````

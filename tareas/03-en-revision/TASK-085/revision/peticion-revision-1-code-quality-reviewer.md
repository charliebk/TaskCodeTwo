# Peticion de revision — TASK-085 (ronda 1)

- Tarea: TASK-085 — F4-6 plan propone skills, agentes y modelo por rol
- Rama revisada: feature/task-085-f4-6-plan-propone-skills-agentes-y-model
- Rama base: develop
- Commit revisado (HEAD): 0689c2cec9429cd0695cd68458dcdad880695989
- Fecha: 2026-11-22
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-085 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-085/tarea.md
- tareas/03-en-revision/TASK-085/planificacion/
- crates/tc2/esquemas/config.schema.json
- crates/tc2/tests/fixtures/muestra-f1-15/tareas.json
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-046/rutas.txt
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-046/tarea.md
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-052/rutas.txt
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-052/tarea.md
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-053/rutas.txt
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-053/tarea.md
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-077/rutas.txt
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-077/tarea.md
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-078/rutas.txt
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-078/tarea.md
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-095/rutas.txt
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-095/tarea.md
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-118/rutas.txt
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-118/tarea.md
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-125/rutas.txt
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-125/tarea.md
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-128/rutas.txt
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-128/tarea.md
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-137/rutas.txt
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-137/tarea.md
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/estratos.txt
- docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md
- docs/ci/revision-ia.md

## Commits a revisar (git log develop..HEAD)

````
0689c2c docs(TASK-085): tabla de 10 tareas de TaskCodeTwo para confirmar por Carlos
7345f6a feat(TASK-085): orden tc2 seleccion, adenda F4-6 en F1-15 y 10 tareas de TaskCodeTwo
4dde725 feat(TASK-085): plan anota la seleccion, approve la muestra y la revision usa dominios
8244644 feat(TASK-085): mapa roles ampliado en tarea.md
a45118d feat(TASK-085): seleccionar por rol (diseno, implementacion, revision) y catalogo ampliado
7b4efa6 feat(TASK-085): tabla-dominios, tres columnas de modelo y detector dominios()
4287a9b chore(TASK-085): tarea en curso
````

## Diff de tu dominio (25 fichero(s) de 48; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/esquemas/config.schema.json b/crates/tc2/esquemas/config.schema.json
index 52f2c59..bf7057b 100644
--- a/crates/tc2/esquemas/config.schema.json
+++ b/crates/tc2/esquemas/config.schema.json
@@ -395,15 +395,34 @@
             "minItems": 1,
             "type": "array"
           },
+          "etiquetas": {
+            "default": [],
+            "description": "etiquetas con las que compite una skill de ejecucion (S07)",
+            "items": {
+              "description": "texto no vacio, sin caracteres de control ni espacios en los extremos",
+              "minLength": 1,
+              "type": "string"
+            },
+            "minItems": 1,
+            "type": "array"
+          },
           "id": {
             "description": "nombre de la skill, el agente o el ejecutable. texto no vacio, sin caracteres de control ni espacios en los extremos",
             "minLength": 1,
             "type": "string"
           },
+          "prioridad": {
+            "description": "desempate de S07: mayor primero",
+            "maximum": 1000,
+            "minimum": 0,
+            "type": "integer"
+          },
           "rol": {
-            "description": "rol que cubre",
+            "description": "rol que cubre; sin rol no se elige (F1-15 decision 8)",
             "enum": [
-              "revisor"
+              "revisor",
+              "diseno",
+              "ejecucion"
             ],
             "type": "string"
           },
diff --git a/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md b/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md
index 98a06d1..e065676 100644
--- a/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md
+++ b/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md
@@ -887,3 +887,89 @@ la App, el prompt con la sección del informe y sin ejecutar la suite).
   título v2 de F6-5 (sin `ronda=`) como ronda 1. Veredicto nuevo `tope`
   (`failure`): pasado el tope de rondas con un CRITICO o IMPORTANTE abierto.
   Contrato de hallazgos v3; guía: `docs/ci/revision-ia.md`, «Rondas (F6-6)».
+
+## Adenda (F4-6, TASK-085) — diseño, implementación y dominios activos; `plan` anota y la revisión los usa
+
+F4-6 completa la decisión 1 sobre lo que dejó F6-3:
+
+- **`seleccion::seleccionar`** (`crates/tc2/src/seleccion/mod.rs`): la misma
+  función pura para los tres roles. Recorre las filas de su rol y de
+  `todos` de `tabla-seleccion` y la columna de su rol de `tabla-modelo`
+  (Diseño, Implementación o Revisión; «—» en diseño de `errata` es sin
+  modelo). `seleccion::revision` queda como envoltura de F6-3 sin dominios.
+  Los agentes de diseño son los ids de la columna «Ids del inventario» de
+  S02 a S05 (suelo de S03, rol de S04, unificador de S05); los no
+  instalados se quitan de `agentes` con `no-instalado` (S14), pero siguen en
+  los roles y `plan` escribe su petición igual.
+- **`seleccion::dominios`** (`crates/tc2/src/seleccion/dominios.rs`): el
+  detector con `tabla-dominios` leída de este fichero (lector nuevo en
+  `tabla.rs`, mismo error por vocabulario). Etiqueta y palabra plegadas (sin
+  tildes ni mayúsculas), palabra al principio de palabra, ruta con glob sin
+  mayúsculas; `dominios_extra` solo añade. Con él, S03, S04, S06, S10, S11 y
+  la excepción de S13 (`dominio=puertas-seguridad`) se evalúan; S10 y S11 se
+  evalúan una vez por dominio detectado.
+- **Catálogo** (decisión 8): el bloque yaml y `recursos:` admiten `rol` ∈
+  {`diseno`, `ejecucion`, `revisor`}, `etiquetas`, `dominios` y
+  `prioridad`; `recursos:` de la configuración de la base entra en el
+  catálogo. S07 elige por solape de etiquetas y dominios, después
+  prioridad, después id, con aviso `empate-por-id`.
+- **`plan`** lee el inventario y el catálogo antes de la operación (la
+  misma `seleccion::orden::fuentes` de `review`), selecciona diseño e
+  implementación y anota `roles.diseno` y `roles.implementacion`; cada
+  petición dice «lánzalo con el agente `<id>` y el modelo `<m>`» y la línea
+  gana `dominios=`, `modelo-diseno=`, `modelo-implementacion=` y `avisos=`.
+  `approve` muestra a la persona `dominios=` (las señales, o `ninguno`),
+  `roles=` y los dos modelos, leídos de la tarea sin recalcular.
+- **Revisión** (`review` y `tc2 revisoras`, el camino de CI): los dominios
+  son la unión de las señales congeladas en `roles.diseno.senales`, las del
+  texto vivo y las de las rutas del diff, con `dominios_extra` de la base;
+  `roles.revision` anota además `reglas`, `senales` y `avisos`.
+- **`tc2 seleccion [TASK-NNN] [--tarea-fichero <tarea.md>] [--rutas
+  <fichero>] [--json]`**: los tres roles sin escribir nada (contrato para
+  F4-4 y F1-16); `--tarea-fichero` acepta los formatos 1 y 2.
+- **`tc2 doctor --recursos`** recoge también `roles.*.agentes`.
+
+**Divergencias con el plan de TASK-085 y con este ADR**:
+
+1. **Mapa `roles` (divergencia 1 de este ADR)**: claves opcionales
+   `agentes`, `reglas`, `senales`, `avisos`, `inventario`, `catalogo` y, en
+   revisión, `ronda`, escritas solo con valor (una tarea de antes se
+   reescribe byte a byte igual). `regla` sigue con el vocabulario de la
+   0.16 y tc2 no la toca; las filas van en `reglas`.
+2. **«Texto aprobado» (decisión 3)**: no se anota el blob de `tarea.md` en
+   `roles.<rol>.texto` (el blob del commit de `approve` contiene la propia
+   anotación); se congelan las `senales` que `plan` calcula, que es lo que
+   la unión necesita. Residual: quien reescriba `roles.diseno.senales` y el
+   texto tras `approve` quita el suelo en la revisión salvo por las rutas
+   del diff (mismo residual que el caso 2 de la PoC S).
+3. **Rol de rendimiento con un solo rol**: S04 lo añade aunque la
+   escalera tenga un rol, y entonces S05 pone el unificador (la tabla manda);
+   `plan::roles::seleccionar` de F4-1 solo lo sumaba con dos o más.
+4. **Señales en `plan`**: `plan` calcula los dominios sin rutas (aún no hay
+   diff); la PoC T los calculó con los ficheros del merge. Por eso la
+   reproducción de la muestra (test `la_muestra_del_adr_se_reproduce`) pasa
+   las rutas a los tres roles, como la PoC.
+5. **Columna «Dominios» de la muestra**: la PoC muestra la primera señal de
+   cada dominio; `roles.<rol>.senales` las anota todas (TASK-403 de ToolsaWeb
+   tiene además `ruta:.taskcode/**`).
+6. **Árbol del proyecto en `plan`**: las skills del proyecto se leen de la
+   base (`origin/develop`), como en `review`, no del commit en que se
+   calcula (decisión 2); el hash del inventario y del catálogo va anotado.
+7. **`start` que para por recursos** y el consumo por el orquestador quedan
+   fuera (F3-9/F1-14 y F4-4): `doctor --recursos` lo dice.
+
+**Evidencia**: unitarios de `seleccion::` (las 10 filas de `tabla-muestra`
+con la fixture `crates/tc2/tests/fixtures/muestra-f1-15/`, leída en solo
+lectura de los commits fijos de los pilotos; M01 a M07 en los tres roles,
+S13 con y sin `puertas-seguridad`, S15 en diseño e implementación; sin red
+ni modelo y S14 en diseño; tabla adversaria del detector), `tests/tarea.rs`
+(mapa ampliado de ida y vuelta), `tests/plan.rs` (modelos distintos por
+carril y complejidad, agente y modelo en cada petición), `tests/aprobar.rs`
+(`plan`, `approve`, revisión con el texto reescrito y `doctor --recursos`) y
+`tests/seleccion.rs` (10 tareas terminadas de TaskCodeTwo por regla fija,
+deterministas).
+
+**No verificado**: que las 10 tareas de TaskCodeTwo coincidan con lo que
+haría una persona (lo marca Carlos tras el cierre, D10a); el orquestador
+(F4-4) consumiendo la selección; el revisor en CI con dominios en un PR real
+(E2E).
diff --git a/docs/ci/revision-ia.md b/docs/ci/revision-ia.md
index 194b29a..133e945 100644
--- a/docs/ci/revision-ia.md
+++ b/docs/ci/revision-ia.md
@@ -480,8 +480,12 @@ F6-6 (decisión 3 de TASK-052).
   de 32 KiB para el bloque entero, cercas incluidas (`tc2 hallazgos validar
   --prosa`, F6-7): se lee como código, sin enlaces, imágenes, menciones ni
   HTML.
-- Sin el detector de dominios (F4-6): S10 y S11 no se aplican (S13 sí, con
-  la ronda de F6-6).
+- Dominios (F4-6): `tc2 revisoras` los calcula con las rutas del diff y,
+  con `--tarea`, con el texto vivo y las señales congeladas en
+  `roles.diseno.senales`, siempre con `dominios_extra` de la base. Una
+  revisora declarada para el dominio entra (S10); si no hay, el aviso
+  `dominio-sin-revisora` (S11), y con `puertas-seguridad` S13 no baja el
+  modelo en las rondas de verificación.
 - Inyección de instrucciones (F6-7; adenda F6-7 de F1-8): el prompt lo
   compone `tc2 hallazgos prompt` y todo dato del PR, de la tarea o de una
   ronda anterior va en una cerca que no puede cerrar, rotulado como dato no
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/02-en-curso/TASK-085/evidencias/tabla-10-tareas-taskcodetwo.md                     | 16 ++++++++++++++++
 tareas/{01-en-diseno => 02-en-curso}/TASK-085/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-085/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-085/tarea.md                                    |  3 ++-
 4 files changed, 18 insertions(+), 1 deletion(-)
````

## Tests del diff (22 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(glob)**/test/**" ":(glob)**/tests/**" ":(glob)**/__tests__/**" ":(glob)**/*.test.*" ":(glob)**/*.spec.*" ":(exclude,glob)**/dist/**" ":(exclude,glob)**/*.lock" ":(exclude,glob)**/*-lock.json" ":(exclude,glob)**/*-lock.yaml" ":(exclude,glob)**/*-lock.yml" ":(exclude,glob)**/*-lock.toml" ":(exclude,glob)tareas/**" ":(exclude,glob)tareas/*/TASK-*/aceptacion/**"
````

````
crates/tc2/tests/fixtures/muestra-f1-15/tareas.json                | 335 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-046/rutas.txt |  17 +++++++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-046/tarea.md  | 142 +++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-052/rutas.txt |  25 ++++++++++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-052/tarea.md  |  97 ++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-053/rutas.txt |  13 +++++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-053/tarea.md  | 100 ++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-077/rutas.txt |  15 ++++++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-077/tarea.md  |  97 ++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-078/rutas.txt |  20 ++++++++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-078/tarea.md  |  91 ++++++++++++++++++++++++++++++++++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-095/rutas.txt |   2 +
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-095/tarea.md  |  93 +++++++++++++++++++++++++++++++++++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-118/rutas.txt |  17 +++++++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-118/tarea.md  |  91 ++++++++++++++++++++++++++++++++++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-125/rutas.txt |   2 +
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-125/tarea.md  |  67 +++++++++++++++++++++++++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-128/rutas.txt |  10 ++++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-128/tarea.md  |  80 ++++++++++++++++++++++++++++++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-137/rutas.txt |   3 ++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-137/tarea.md  |  72 +++++++++++++++++++++++++++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/estratos.txt       |  10 ++++
 22 files changed, 1399 insertions(+)
````

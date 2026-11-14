# Peticion de revision — TASK-011 (ronda 4)

- Tarea: TASK-011 — F1-10 ADR carriles y modo automatico
- Rama revisada: feature/task-011-f1-10-adr-carriles-y-modo-automatico
- Rama base: develop
- Commit revisado (HEAD): 8ef8213643323201b94815602d928e4cffb6fb8a
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-011 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 24d05a006aff0a62482dfdd1cd49fcc70088c154 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (2 hallazgo(s)))

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
informe de esta ronda (informe-revision-4.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-011/tarea.md
- tareas/03-en-revision/TASK-011/planificacion/
- docs/adr/F1-10-carriles-y-modo-automatico.md

## Hallazgos de la ronda 3 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-10-carriles-y-modo-automatico.md:385 | informe-revision-3.md |
| IMP-2 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-011/poc/poc-f1-10.py:256 | informe-revision-3.md |

## Commits a revisar (git log 24d05a006aff0a62482dfdd1cd49fcc70088c154..HEAD)

````
8ef8213 chore(TASK-011): autorrevision y ronda extra 4
4f79940 docs(adr): F1-10 corrige IMP-2 de la revision 3 (union con la politica configurada)
a54736d chore(TASK-011): peticion de revision ronda 3
````

## Diff desde la ronda anterior (git diff 24d05a006aff0a62482dfdd1cd49fcc70088c154..HEAD)

````diff
diff --git a/docs/adr/F1-10-carriles-y-modo-automatico.md b/docs/adr/F1-10-carriles-y-modo-automatico.md
index 666ccf2..af05495 100644
--- a/docs/adr/F1-10-carriles-y-modo-automatico.md
+++ b/docs/adr/F1-10-carriles-y-modo-automatico.md
@@ -224,8 +224,13 @@ paradas nuevas por tipo: approve=9, merge-pr=9, tope=2
   union: release funcionalidad desde develop efectivo=funcionalidad -> release+funcionalidad, True, 2, [approve, merge-pr, tag] ok
 -- barrido: approve, merge-pr y tag en todo carril, tipo, base, modo y configuracion valida de D
   combinaciones=1260 ; continuar=0 ; tipo y base que fallan cerrado (sin fila)=160
-  filas de main por debajo del carril efectivo: 0/8
-RESULTADO C ... ; tipo y base 11/11 ; continuar sobre approve/merge-pr/tag=0 ; paradas distintas de la politica=0
+  filas de main por debajo del carril efectivo configurado (5 configuraciones aceptadas de D): 0/40
+-- union con la politica configurada (revision 3, IMP-2)
+   (cada resultado: fila, plan, aprobaciones, roles minimos, paradas; revisoras = por dominio del diff en los tres)
+  X1 hotfix/* -> main, fix + plan_aprobado                   -> hotfix+fix, True, 2, 1, [approve, merge-pr, tag] ok
+  X2 release develop -> main, funcionalidad + review, roles 3 -> release+funcionalidad, True, 2, 3, [approve, merge-pr, review, tag] ok
+  X3 hotfix/* -> main, funcionalidad + review, roles 3       -> hotfix+funcionalidad, True, 2, 3, [approve, merge-pr, review, tag] ok
+RESULTADO C ... ; tipo y base 14/14 ; continuar sobre approve/merge-pr/tag=0 ; paradas distintas de la politica=0
 ```
 
 Lectura:
@@ -239,9 +244,12 @@ Lectura:
   `hotfix/*` → `main`; cualquier otro PR contra `main` falla cerrado (M1) y
   contra `develop` manda el carril efectivo. Además, una fila de `main`
   nunca baja de su carril efectivo: sus puertas son la unión de las de la
-  fila y las del carril (M3: un hotfix declarado funcionalidad lleva plan,
-  `approve` y dos aprobaciones; 0 de 8 filas de `main` por debajo de su
-  carril; 11/11 casos).
+  fila y la política del carril **ya endurecida por la configuración** (M3:
+  un hotfix declarado funcionalidad lleva plan, `approve` y dos
+  aprobaciones; X1–X3: un `plan_aprobado`, una `paradas_extra` o unos
+  `roles_diseno` del carril también valen contra `main`; 0 de 40 filas de
+  `main` por debajo de su carril configurado con las 5 configuraciones
+  aceptadas de D; 14/14 casos).
 - **Las paradas por tarea suben** (0,85 → 2,52 en ToolsaWeb). No las añade
   este ADR: son las dos puertas de F1-5 y F1-6 que la 0.16 no tiene. Hoy, en
   automático, el agente mergea en local (`finish=156` en ToolsaWeb, sin
@@ -333,6 +341,16 @@ RESULTADO D aceptadas=5/5 ; rechazadas=17/17
    una fila de `main` nunca baja del carril efectivo (unión de puertas;
    decisión 1; PoC C, M1, M3 y la unión, 11/11). Nota menor: las rutas se
    normalizan (`./.github/x` sale riesgo alto; antes, fix).
+10. **Revisión 3** (Carlos, con la consigna «la opción recomendada»):
+    **IMP-2**, la unión de una fila de `main` tomaba la fila de la tabla del
+    carril, sin `politica_carriles`: con `fix.plan_aprobado: true` un hotfix
+    a `main` salía sin plan (X1), y con `funcionalidad` endurecida
+    (`paradas_extra: [review]`, `roles_diseno: 3`) una release o un hotfix
+    perdían la parada y los roles (X2, X3). Ahora la unión toma
+    `politica(carril efectivo, config)` en todas sus columnas, también
+    «Revisoras» y `revisoras_extra` (nota menor), y la comprobación de
+    filas por debajo del carril usa las 5 configuraciones aceptadas de D
+    (0/40; PoC C, 14/14).
 
 ### No verificado
 
@@ -389,10 +407,14 @@ existe aún):
    fila es la del carril efectivo; cualquier otro PR contra `main` (un
    `release` desde `feature/*`, un `hotfix` desde otra rama, otro tipo) no
    tiene fila y `carril` falla cerrado. **Una fila de `main` nunca baja de
-   lo que exige el carril efectivo**: sus puertas son la unión de las de la
-   fila y las de la fila del carril (plan y `approve` si el carril los
-   exige, el mayor número de aprobaciones y de roles, la unión de checks y
-   de paradas, y fuera del lote si cualquiera de las dos lo está). Una
+   lo que exige el carril efectivo**: sus puertas son la unión, columna a
+   columna, de las de la fila y de `politica(carril efectivo, config)`, es
+   decir, la política del carril **ya endurecida por `politica_carriles`**
+   (plan y `approve` si el carril configurado los exige, el mayor número de
+   aprobaciones y de roles, la unión de checks y de paradas —también las
+   `paradas_extra`—, el perfil mayor, las revisoras de las dos —«ninguna»
+   de la release no anula las «por dominio del diff» del carril— más sus
+   `revisoras_extra`, y fuera del lote si cualquiera de las dos lo está). Una
    release cuyo diff toca `.taskcode/` lleva así lo de riesgo alto (decisión
    4), y un hotfix declarado funcionalidad, su plan firmado. `Fase` = `plan | approve | start | review | merge-pr | tag`;
    `politica` recibe la política del carril **efectivo**. La usan
@@ -587,7 +609,7 @@ Los ADR aceptados no se modifican; lo que cambia queda aquí y en
 | RF-33 tope de rondas | parada `preguntar` con las salidas de la 0.16 | PoC C (tope=69 / 40 / 2) |
 | RF-51 `## Resultado` | sin cambio: lo exige `carril` (F1-5) | — |
 | RN-06 una persona con agentes | D4a de F1-5 y D7a | D7a no verificado (F2-6) |
-| RR-08 / F1-8 `A10` lo escrito en `tarea.md` no baja una puerta (revisiones 1 y 2, IMP-1) | `tipo` solo elige fila con base real `main` y su rama de origen (`develop` en release, `hotfix/*` en hotfix); contra `develop`, el carril efectivo; las puertas de una fila de `main` son la unión con las del carril efectivo | PoC C (11/11: `tipo: hotfix` con diff en `.github/` sale riesgo alto contra `develop` y hotfix + riesgo alto contra `main`; M1, `release` desde `feature/*` → falla cerrado; M3, hotfix funcionalidad → plan y `approve`; release funcionalidad desde `develop` → unión; 0 de 8 filas de `main` por debajo de su carril) |
+| RR-08 / F1-8 `A10` lo escrito en `tarea.md` no baja una puerta (revisiones 1, 2 y 3, IMP-1 e IMP-2) | `tipo` solo elige fila con base real `main` y su rama de origen (`develop` en release, `hotfix/*` en hotfix); contra `develop`, el carril efectivo; las puertas de una fila de `main` son la unión con la política configurada del carril efectivo | PoC C (14/14: `tipo: hotfix` con diff en `.github/` sale riesgo alto contra `develop` y hotfix + riesgo alto contra `main`; M1, `release` desde `feature/*` → falla cerrado; M3, hotfix funcionalidad → plan y `approve`; release funcionalidad desde `develop` → unión; X1–X3, la configuración del carril también vale contra `main`; 0 de 40 filas de `main` por debajo de su carril configurado) |
 | Derivación con renombrados (F1-5) | los dos nombres; nunca funcionalidad | PoC A (203 tareas: 0 funcionalidad; 23/23 casos, `.github/` → `docs/` = riesgo alto) |
 | Preguntas abiertas de F1-3, F1-4, F1-5, F1-6, F1-8 y F1-9 | tabla «Preguntas heredadas» (15 filas, ninguna sin destino) | este ADR |
````

## Excluido del diff (6 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 24d05a006aff0a62482dfdd1cd49fcc70088c154..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/03-en-revision/TASK-011/poc/poc-f1-10.log               |  10 +++--
 tareas/03-en-revision/TASK-011/poc/poc-f1-10.py                |  96 +++++++++++++++++++++++++++++------------
 tareas/03-en-revision/TASK-011/revision/informe-revision-2.md  |  44 ++++++++++++++++---
 tareas/03-en-revision/TASK-011/revision/informe-revision-3.md  |  21 +++++++++
 tareas/03-en-revision/TASK-011/revision/peticion-revision-3.md | 301 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-011/tarea.md                        |   6 ++-
 6 files changed, 441 insertions(+), 37 deletions(-)
````

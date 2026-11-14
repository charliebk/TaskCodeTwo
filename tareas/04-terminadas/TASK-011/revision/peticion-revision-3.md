# Peticion de revision — TASK-011 (ronda 3)

- Tarea: TASK-011 — F1-10 ADR carriles y modo automatico
- Rama revisada: feature/task-011-f1-10-adr-carriles-y-modo-automatico
- Rama base: develop
- Commit revisado (HEAD): 24d05a006aff0a62482dfdd1cd49fcc70088c154
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-011 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 08e34e710ec49d3d7dbe31e6302b8fd79f2809c5 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (1 hallazgo(s)))

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

- tareas/03-en-revision/TASK-011/tarea.md
- tareas/03-en-revision/TASK-011/planificacion/
- docs/adr/F1-10-carriles-y-modo-automatico.md

## Hallazgos de la ronda 2 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-10-carriles-y-modo-automatico.md:369 | informe-revision-2.md |

## Commits a revisar (git log 08e34e710ec49d3d7dbe31e6302b8fd79f2809c5..HEAD)

````
24d05a0 chore(TASK-011): autorrevision y ronda extra 3
e8e3306 docs(adr): F1-10 corrige IMP-1 de la revision 2 (release solo desde develop y union de puertas)
8b0f4e0 chore(TASK-011): peticion de revision ronda 2
````

## Diff desde la ronda anterior (git diff 08e34e710ec49d3d7dbe31e6302b8fd79f2809c5..HEAD)

````diff
diff --git a/docs/adr/F1-10-carriles-y-modo-automatico.md b/docs/adr/F1-10-carriles-y-modo-automatico.md
index fba1785..666ccf2 100644
--- a/docs/adr/F1-10-carriles-y-modo-automatico.md
+++ b/docs/adr/F1-10-carriles-y-modo-automatico.md
@@ -121,7 +121,8 @@ renombrados fuera de tareas/: 0 ; que cambian de clase con los dos nombres: 0
   .taskcode exacto (fichero o enlace)      -> riesgo-alto  ok
   .githubx/ no es el suelo                 -> fix          ok
   PR sin ficheros                          -> ERROR        ok
-  (y 11 casos más: 22/22)
+  ./.github/ se normaliza                  -> riesgo-alto  ok
+  (y 11 casos más: 23/23)
 ```
 
 Lectura:
@@ -209,30 +210,38 @@ paradas nuevas por tipo: approve=9, merge-pr=9, tope=2
   release  paradas=2 [('merge-pr', 1), ('tag', 1)] politica=['merge-pr', 'tag'] -> ok
   hotfix   paradas=2 [('merge-pr', 1), ('tag', 1)] politica=['merge-pr', 'tag'] -> ok
 -- tipo de la tarea frente a la base real del PR (revision 1, IMP-1)
-  hotfix con .github/ contra develop     efectivo=riesgo-alto  -> ('riesgo-alto', True, 2, ['approve', 'merge-pr']) ok
-  hotfix con .github/ contra main        efectivo=riesgo-alto  -> ('hotfix+riesgo-alto', True, 2, ['approve', 'merge-pr', 'tag']) ok
-  hotfix solo src/ contra main           efectivo=fix          -> ('hotfix', False, 1, ['merge-pr', 'tag']) ok
-  release con .github/ contra develop    efectivo=riesgo-alto  -> ('riesgo-alto', True, 2, ['approve', 'merge-pr']) ok
-  release con .taskcode/ contra main     efectivo=riesgo-alto  -> ('release+riesgo-alto', True, 2, ['approve', 'merge-pr', 'tag']) ok
-  release solo src/ contra main          efectivo=fix          -> ('release', False, 1, ['merge-pr', 'tag']) ok
+   (cada resultado: fila, plan, aprobaciones, paradas; checks = carril, revision-ia, verificacion en todos)
+  hotfix con .github/ contra develop     efectivo=riesgo-alto  -> riesgo-alto, True, 2, [approve, merge-pr] ok
+  hotfix con .github/ contra main        efectivo=riesgo-alto  -> hotfix+riesgo-alto, True, 2, [approve, merge-pr, tag] ok
+  hotfix solo src/ contra main           efectivo=fix          -> hotfix+fix, False, 1, [merge-pr, tag] ok
+  release con .github/ contra develop    efectivo=riesgo-alto  -> riesgo-alto, True, 2, [approve, merge-pr] ok
+  release con .taskcode/ contra main     efectivo=riesgo-alto  -> release+riesgo-alto, True, 2, [approve, merge-pr, tag] ok
+  release solo docs/ contra main         efectivo=errata       -> release+errata, False, 1, [merge-pr, tag] ok
   hotfix contra main desde feature/*     efectivo=fix          -> ERROR ok
   feature contra main                    efectivo=funcionalidad -> ERROR ok
+  M1 release contra main desde feature/* efectivo=funcionalidad -> ERROR ok
+  M3 hotfix funcionalidad contra main    efectivo=funcionalidad -> hotfix+funcionalidad, True, 2, [approve, merge-pr, tag] ok
+  union: release funcionalidad desde develop efectivo=funcionalidad -> release+funcionalidad, True, 2, [approve, merge-pr, tag] ok
 -- barrido: approve, merge-pr y tag en todo carril, tipo, base, modo y configuracion valida de D
-  combinaciones=1440 ; continuar=0 ; tipo y base que fallan cerrado (sin fila)=140
-RESULTADO C ... ; tipo y base 8/8 ; continuar sobre approve/merge-pr/tag=0 ; paradas distintas de la politica=0
+  combinaciones=1260 ; continuar=0 ; tipo y base que fallan cerrado (sin fila)=160
+  filas de main por debajo del carril efectivo: 0/8
+RESULTADO C ... ; tipo y base 11/11 ; continuar sobre approve/merge-pr/tag=0 ; paradas distintas de la politica=0
 ```
 
 Lectura:
 
 - **Nunca hay `continuar` sobre `approve`, `merge-pr` o `tag`**, ni en las
-  tareas reales ni en las 1440 combinaciones de carril, tipo, base, modo y
+  tareas reales ni en las 1260 combinaciones de carril, tipo, base, modo y
   configuración válida; y en modo automático las paradas fijas que ve cada
   tarea son exactamente las de su fila de la tabla.
-- **El tipo no baja la puerta** (revisión 1, IMP-1): `tipo: hotfix` o
-  `tipo: release` solo eligen su fila con la base real `main` (y en hotfix,
-  la rama `hotfix/*`); contra `develop` manda el carril efectivo, y un diff
-  que toca `.github/` o `.taskcode/` exige también contra `main` el plan
-  firmado y las dos aprobaciones de riesgo alto (8/8 casos).
+- **El tipo no baja la puerta** (revisiones 1 y 2, IMP-1): `tipo: release`
+  solo elige su fila en un PR `develop` → `main`, y `tipo: hotfix` en uno
+  `hotfix/*` → `main`; cualquier otro PR contra `main` falla cerrado (M1) y
+  contra `develop` manda el carril efectivo. Además, una fila de `main`
+  nunca baja de su carril efectivo: sus puertas son la unión de las de la
+  fila y las del carril (M3: un hotfix declarado funcionalidad lleva plan,
+  `approve` y dos aprobaciones; 0 de 8 filas de `main` por debajo de su
+  carril; 11/11 casos).
 - **Las paradas por tarea suben** (0,85 → 2,52 en ToolsaWeb). No las añade
   este ADR: son las dos puertas de F1-5 y F1-6 que la 0.16 no tiene. Hoy, en
   automático, el agente mergea en local (`finish=156` en ToolsaWeb, sin
@@ -317,6 +326,13 @@ RESULTADO D aceptadas=5/5 ; rechazadas=17/17
    exactos también). **MEN-3**, la PoC imprime los recuentos de `finish` y
    `approve` que cita el ADR y la cota inferior. **MEN-4**, el número de
    roles lo fija `plan`, no `politica` (decisión 1).
+9. **Revisión 2** (Carlos, con la consigna «la opción recomendada»):
+   **IMP-1** seguía abierto para `release`, que no exigía rama de origen (un
+   PR `feature/*` → `main` con `tipo: release` quitaba plan, `approve` y
+   `revision-ia`). Ahora la fila release solo vale desde `develop` y, además,
+   una fila de `main` nunca baja del carril efectivo (unión de puertas;
+   decisión 1; PoC C, M1, M3 y la unión, 11/11). Nota menor: las rutas se
+   normalizan (`./.github/x` sale riesgo alto; antes, fix).
 
 ### No verificado
 
@@ -366,15 +382,19 @@ existe aún):
 
    `Carril` = `errata | fix | funcionalidad | riesgo-alto` (ordenados);
    `Tipo` ∈ los de la tarea. **El tipo es texto de `tarea.md` (no
-   confiable) y solo elige fila con la base real del PR** (la de la API en
-   `carril`; la del PR que abre `review` en local): `release` con base
-   `main`, y `hotfix` con base `main` y rama `hotfix/*`. En un PR contra
-   `develop` el tipo se ignora y la fila es la del carril efectivo; un PR
-   contra `main` que no sea release ni hotfix (o un hotfix desde otra rama)
-   no tiene fila y `carril` falla cerrado. En una fila de `main`, si el
-   carril efectivo es riesgo alto (mínimo por ficheros, decisión 4), se
-   exige además lo de riesgo alto: plan con `approve` firmado, dos
-   aprobaciones, `revision-ia` y la parada `approve`. `Fase` = `plan | approve | start | review | merge-pr | tag`;
+   confiable) y solo elige fila con la base y la rama reales del PR** (las
+   de la API en `carril`; las del PR que abre `review` en local):
+   `release` solo en un PR `develop` → `main` (D3a), y `hotfix` solo en uno
+   `hotfix/*` → `main`. En un PR contra `develop` el tipo se ignora y la
+   fila es la del carril efectivo; cualquier otro PR contra `main` (un
+   `release` desde `feature/*`, un `hotfix` desde otra rama, otro tipo) no
+   tiene fila y `carril` falla cerrado. **Una fila de `main` nunca baja de
+   lo que exige el carril efectivo**: sus puertas son la unión de las de la
+   fila y las de la fila del carril (plan y `approve` si el carril los
+   exige, el mayor número de aprobaciones y de roles, la unión de checks y
+   de paradas, y fuera del lote si cualquiera de las dos lo está). Una
+   release cuyo diff toca `.taskcode/` lleva así lo de riesgo alto (decisión
+   4), y un hotfix declarado funcionalidad, su plan firmado. `Fase` = `plan | approve | start | review | merge-pr | tag`;
    `politica` recibe la política del carril **efectivo**. La usan
    `siguiente` y el orquestador (F4-4), `plan` (F4-1, F4-6), el check
    `carril` (F6), `lote` (F1-16) y `doctor` (F3-8). `siguiente --json`
@@ -388,8 +408,8 @@ existe aún):
    | `fix` | `develop` | 1 (arquitectura) | no | por dominio del diff | completa | `verificacion`, `revision-ia`, `carril` | 1 (PR) | `merge-pr` | sí |
    | `funcionalidad` | `develop` | 1 a 4, según la complejidad | sí, `approve` firmado | por dominio del diff | completa | `verificacion`, `revision-ia`, `carril` | 2 (plan y PR) | `approve`, `merge-pr` | sí, con el plan aprobado antes de lanzar el lote |
    | `riesgo-alto` | `develop` | todos (rendimiento si la etiqueta lo pide) | sí, `approve` firmado | por dominio del diff | completa | `verificacion`, `revision-ia`, `carril` | 2 (plan y PR; D6) | `approve`, `merge-pr` | no |
-   | `release` (tipo) | `main` (base real del PR) | 0 (sin plan) | no | ninguna | completa y pesada | `verificacion`, `carril` | 1 (PR) | `merge-pr`, `tag` | no |
-   | `hotfix` (tipo) | `main` (base real del PR), rama `hotfix/*` | 1 (arquitectura) | no | por dominio del diff | completa | `verificacion`, `revision-ia`, `carril` | 1 (PR) | `merge-pr`, `tag` | no |
+   | `release` (tipo) | `main`, solo desde `develop` (base y rama reales del PR); puertas ∪ las del carril efectivo | 0 (sin plan) | no | ninguna | completa y pesada | `verificacion`, `carril` | 1 (PR) | `merge-pr`, `tag` | no |
+   | `hotfix` (tipo) | `main`, solo desde `hotfix/*` (base y rama reales del PR); puertas ∪ las del carril efectivo | 1 (arquitectura) | no | por dominio del diff | completa | `verificacion`, `revision-ia`, `carril` | 1 (PR) | `merge-pr`, `tag` | no |
    <!-- tabla-politica:fin -->
 
    Columnas: «Checks» y «Aprobaciones» son las de F1-5 (no cambian); el
@@ -425,15 +445,17 @@ existe aún):
    de la tarea; el check `carril` la lee de la base por la API desde
    `pull_request_target` (F1-8 D1b, parcial). El carril declarado en
    `tarea.md` es texto no confiable (F1-8, `A10`; F1-9): solo sirve para
-   subir; el `tipo`, igual, solo elige la fila de release o hotfix con la
-   base real `main` (decisión 1). Las paradas locales del orquestador son de usabilidad (evitar
+   subir; el `tipo`, igual, solo elige la fila de release (desde `develop`)
+   o de hotfix (desde `hotfix/*`) con la base real `main`, y nunca por
+   debajo del carril efectivo (decisión 1). Las paradas locales del orquestador son de usabilidad (evitar
    trabajo que la puerta rechazará); quien impone es `carril`, con la misma
    función. Un PR que cambia `.taskcode/` sale riesgo alto por el suelo, y su
    configuración nueva no se aplica hasta que esté en la base (PoC D).
 4. **Derivación del carril mínimo.** Entrada: los ficheros del PR por la API
    (en local, `git diff -M --name-status` contra la base); en cada
    renombrado o copia, **los dos nombres** (`previous_filename`). Por cada
-   ruta: primer componente `.github` o `.taskcode` (también la ruta exacta:
+   ruta, normalizada antes (sin `./` ni componentes vacíos: `./.github/x`
+   es `.github/x`): primer componente `.github` o `.taskcode` (también la ruta exacta:
    un fichero, enlace o submódulo con ese nombre) o un `CODEOWNERS` en
    cualquier carpeta (comparando sin mayúsculas) → riesgo alto; un glob de
    `rutas_carril.riesgo-alto` → riesgo alto; uno de `rutas_carril.fix` → fix;
@@ -442,7 +464,9 @@ existe aún):
    una ruta con `..` o una lista que no se puede leer entera (la API corta
    a partir de 3000 ficheros, no verificado) → `carril` falla, no decide. El
    mínimo se aplica a todos los PR, también a los de release y hotfix contra
-   `main` (decisión 1: suben a lo de riesgo alto). El
+   `main` (decisión 1: unión con las puertas del carril efectivo). Un
+   cambio solo de `docs/` que tenga que ir directo a `main` no tiene fila
+   de errata allí: va como hotfix, desde `hotfix/*`. El
    tamaño del diff no cuenta: un PR enorme de solo `docs/` sigue siendo
    errata (lo revisa `revision-ia`; F1-17 usa el tamaño para estimar).
 5. **Qué congela la tarea.** En `plan` se congelan el **modo** (como en la
@@ -465,14 +489,16 @@ existe aún):
 
    `accion` resuelve `approve`, `merge-pr` y `tag` **antes** de mirar la
    política o el modo, así que ninguna configuración las convierte en
-   `continuar` (PoC C: 1440 combinaciones, 0 `continuar`). Fuera de esas y de
+   `continuar` (PoC C: 1260 combinaciones, 0 `continuar`). Fuera de esas y de
    las `paradas_extra`, en `automatico` todo se encadena; `manual` devuelve
    `detener` en cada fase y `semiautomatico`, `preguntar`. Las guardas del CLI
    de la 0.16 que no son de política (lo que se mergearía no coincide con lo
    revisado, revisor independiente) siguen en `review` y `finish`.
 7. **Lo que cierra además** (D1–D7 del plan):
    - **D1a** `carril` ∈ `errata < fix < funcionalidad < riesgo-alto`;
-     release y hotfix, por `tipo`, solo con base real `main` (decisión 1).
+     release y hotfix, por `tipo`, solo con base real `main` y su rama de
+     origen (`develop`, `hotfix/*`), sin bajar del carril efectivo
+     (decisión 1).
    - **D2a** funcionalidad solo se declara en `plan`.
    - **D3a** la release la lanza la persona: `release` abre el PR
      `develop`→`main` con la identidad del agente (F1-6), y el `merge-pr` y
@@ -539,6 +565,12 @@ Los ADR aceptados no se modifican; lo que cambia queda aquí y en
 4. **F1-9, tabla de campos**: `complejidad` figura como «entrada del carril
    (F1-10)»; aquí no decide el carril, solo cuántos roles lleva una
    funcionalidad.
+5. **F1-5, decisión 2** (revisión 2): F1-5 no exige `revision-ia` en la
+   release («una release no la necesita»). Con la unión de puertas, la
+   release la lleva siempre, porque todas las filas de carril incluyen
+   `revision-ia` (también errata, con perfil rápido): un PR `develop` →
+   `main` pasa por el revisor. Endurece, no baja nada; su coste en minutos
+   lo mide F1-16 o F7.
 
 ## Requisitos y evidencia
 
@@ -548,15 +580,15 @@ Los ADR aceptados no se modifican; lo que cambia queda aquí y en
 | P7 / RR-07 carriles proporcionales; el automático para solo en las puertas del carril | paradas fijas (decisión 6); errata y fix sin `approve` | PoC C (paradas = fila de la tabla en todas las tareas; 0 `continuar` en fijas); las paradas por tarea suben (divergencia 3) |
 | RF-21 modos con encadenado | `accion` en `siguiente --json`, modo congelado | PoC C (manual, semiautomático y automático en las tareas reales) |
 | RR-08 garantías en la plataforma | `carril` impone con la misma función; política de la base | decisión 3; PoC D (PR que cambia `.taskcode/`); GitHub: no verificado (F2-6) |
-| RR-04 / RF-50 aprobación de una persona antes del merge | `merge-pr` y `approve` como paradas fijas | PoC C (1440 combinaciones, 0 `continuar`) |
+| RR-04 / RF-50 aprobación de una persona antes del merge | `merge-pr` y `approve` como paradas fijas | PoC C (1260 combinaciones, 0 `continuar`) |
 | H5 / H6 aprobación comprobable; el agente no se salta puertas | `approve` firmado y `merge-pr` de persona en cualquier configuración | PoC C (los 114 `approve` con `decidido_por: automatico` de ToolsaWeb pasan a `detener`); PoC D |
 | RR-02 / RR-17 roles y revisoras proporcionales al carril | columnas «Roles» y «Revisoras» | PoC B (uno en fix, todos en riesgo alto); contenido en F1-15 |
 | RR-18 / RF-27 lotes con revisión humana al final | columna «Lote» | tabla; diseño en F1-16 |
 | RF-33 tope de rondas | parada `preguntar` con las salidas de la 0.16 | PoC C (tope=69 / 40 / 2) |
 | RF-51 `## Resultado` | sin cambio: lo exige `carril` (F1-5) | — |
 | RN-06 una persona con agentes | D4a de F1-5 y D7a | D7a no verificado (F2-6) |
-| RR-08 / F1-8 `A10` lo escrito en `tarea.md` no baja una puerta (revisión 1, IMP-1) | `tipo` solo elige fila con base real `main` (y rama `hotfix/*`); contra `develop`, el carril efectivo; el mínimo por ficheros también en release y hotfix | PoC C (8/8: `tipo: hotfix` con diff en `.github/` sale riesgo alto contra `develop` y hotfix + riesgo alto contra `main`; tipo y base sin fila fallan cerrado) |
-| Derivación con renombrados (F1-5) | los dos nombres; nunca funcionalidad | PoC A (203 tareas: 0 funcionalidad; 22/22 casos, `.github/` → `docs/` = riesgo alto) |
+| RR-08 / F1-8 `A10` lo escrito en `tarea.md` no baja una puerta (revisiones 1 y 2, IMP-1) | `tipo` solo elige fila con base real `main` y su rama de origen (`develop` en release, `hotfix/*` en hotfix); contra `develop`, el carril efectivo; las puertas de una fila de `main` son la unión con las del carril efectivo | PoC C (11/11: `tipo: hotfix` con diff en `.github/` sale riesgo alto contra `develop` y hotfix + riesgo alto contra `main`; M1, `release` desde `feature/*` → falla cerrado; M3, hotfix funcionalidad → plan y `approve`; release funcionalidad desde `develop` → unión; 0 de 8 filas de `main` por debajo de su carril) |
+| Derivación con renombrados (F1-5) | los dos nombres; nunca funcionalidad | PoC A (203 tareas: 0 funcionalidad; 23/23 casos, `.github/` → `docs/` = riesgo alto) |
 | Preguntas abiertas de F1-3, F1-4, F1-5, F1-6, F1-8 y F1-9 | tabla «Preguntas heredadas» (15 filas, ninguna sin destino) | este ADR |
 
 ## Consecuencias
````

## Excluido del diff (6 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 08e34e710ec49d3d7dbe31e6302b8fd79f2809c5..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/03-en-revision/TASK-011/poc/poc-f1-10.log               |  25 +++++----
 tareas/03-en-revision/TASK-011/poc/poc-f1-10.py                |  67 ++++++++++++++++--------
 tareas/03-en-revision/TASK-011/revision/informe-revision-1.md  |  65 +++++++++++++++++++++--
 tareas/03-en-revision/TASK-011/revision/informe-revision-2.md  |  21 ++++++++
 tareas/03-en-revision/TASK-011/revision/peticion-revision-2.md | 365 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-011/tarea.md                        |   9 +++-
 6 files changed, 515 insertions(+), 37 deletions(-)
````

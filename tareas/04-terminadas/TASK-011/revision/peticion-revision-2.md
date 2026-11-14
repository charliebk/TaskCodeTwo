# Peticion de revision — TASK-011 (ronda 2)

- Tarea: TASK-011 — F1-10 ADR carriles y modo automatico
- Rama revisada: feature/task-011-f1-10-adr-carriles-y-modo-automatico
- Rama base: develop
- Commit revisado (HEAD): 08e34e710ec49d3d7dbe31e6302b8fd79f2809c5
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-011 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde e83f838c05980a2115e24949afbd7896e042bdc2 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (5 hallazgo(s)))

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
informe de esta ronda (informe-revision-2.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-011/tarea.md
- tareas/03-en-revision/TASK-011/planificacion/
- docs/adr/F1-10-carriles-y-modo-automatico.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 4 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-10-carriles-y-modo-automatico.md:330 | informe-revision-1.md |

## Commits a revisar (git log e83f838c05980a2115e24949afbd7896e042bdc2..HEAD)

````
08e34e7 chore(TASK-011): constancia de autorrevision ronda 2
232628f docs(adr): F1-10 corrige los hallazgos de la revision 1 (tipo solo con base main, suelo por componente, menores)
25a2365 chore(TASK-011): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff e83f838c05980a2115e24949afbd7896e042bdc2..HEAD)

````diff
diff --git a/docs/adr/F1-10-carriles-y-modo-automatico.md b/docs/adr/F1-10-carriles-y-modo-automatico.md
index df93cfd..fba1785 100644
--- a/docs/adr/F1-10-carriles-y-modo-automatico.md
+++ b/docs/adr/F1-10-carriles-y-modo-automatico.md
@@ -118,8 +118,10 @@ renombrados fuera de tareas/: 0 ; que cambian de clase con los dos nombres: 0
   Docs/ en mayusculas no es errata         -> fix          ok
   copia desde .github/                     -> riesgo-alto  ok
   rutas_carril sube src/auth/**            -> riesgo-alto  ok
+  .taskcode exacto (fichero o enlace)      -> riesgo-alto  ok
+  .githubx/ no es el suelo                 -> fix          ok
   PR sin ficheros                          -> ERROR        ok
-  (y 10 casos más: 19/19)
+  (y 11 casos más: 22/22)
 ```
 
 Lectura:
@@ -192,38 +194,61 @@ Para cada tarea terminada de los tres repos, sus filas de `## Transiciones`:
 
 ```text
 --- ToolsaWeb  tareas=160  carril efectivo: fix=5, funcionalidad=151, riesgo-alto=4
-paradas humanas: hoy=136 (0.85/tarea) ; nuevo=403 (2.52/tarea)
+filas en automatico: finish=156 ; approve decidido_por automatico=114 ; approve decidido_por persona=43
+paradas humanas: hoy=136 (0.85/tarea) ; nuevo=403 (2.52/tarea) ; cota inferior con declarado = minimo por ficheros=244 (1.52/tarea)
 paradas nuevas por tipo: approve=159, merge-pr=160, modo-manual=11, subida=4, tope=69
 --- OpenDockTrading  tareas=43  carril efectivo: funcionalidad=42, riesgo-alto=1
-paradas humanas: hoy=91 (2.12/tarea) ; nuevo=137 (3.19/tarea)
+filas en automatico: finish=40 ; approve decidido_por automatico=5 ; approve decidido_por persona=35
+paradas humanas: hoy=91 (2.12/tarea) ; nuevo=137 (3.19/tarea) ; cota inferior con declarado = minimo por ficheros=93 (2.16/tarea)
 paradas nuevas por tipo: approve=44, merge-pr=43, modo-semiautomatico=9, subida=1, tope=40
 --- TaskCodeTwo  tareas=9  carril efectivo: funcionalidad=9
-paradas humanas: hoy=11 (1.22/tarea) ; nuevo=20 (2.22/tarea)
+filas en automatico: finish=9 ; approve decidido_por automatico=0 ; approve decidido_por persona=9
+paradas humanas: hoy=11 (1.22/tarea) ; nuevo=20 (2.22/tarea) ; cota inferior con declarado = minimo por ficheros=11 (1.22/tarea)
 paradas nuevas por tipo: approve=9, merge-pr=9, tope=2
 -- tareas sinteticas de tipo release y hotfix (la 0.16 no tiene ninguna terminada)
   release  paradas=2 [('merge-pr', 1), ('tag', 1)] politica=['merge-pr', 'tag'] -> ok
   hotfix   paradas=2 [('merge-pr', 1), ('tag', 1)] politica=['merge-pr', 'tag'] -> ok
--- barrido: approve, merge-pr y tag en todo carril, tipo, modo y configuracion valida de D
-  combinaciones=900 ; continuar=0
-RESULTADO C ... ; continuar sobre approve/merge-pr/tag=0 ; paradas distintas de la politica=0
+-- tipo de la tarea frente a la base real del PR (revision 1, IMP-1)
+  hotfix con .github/ contra develop     efectivo=riesgo-alto  -> ('riesgo-alto', True, 2, ['approve', 'merge-pr']) ok
+  hotfix con .github/ contra main        efectivo=riesgo-alto  -> ('hotfix+riesgo-alto', True, 2, ['approve', 'merge-pr', 'tag']) ok
+  hotfix solo src/ contra main           efectivo=fix          -> ('hotfix', False, 1, ['merge-pr', 'tag']) ok
+  release con .github/ contra develop    efectivo=riesgo-alto  -> ('riesgo-alto', True, 2, ['approve', 'merge-pr']) ok
+  release con .taskcode/ contra main     efectivo=riesgo-alto  -> ('release+riesgo-alto', True, 2, ['approve', 'merge-pr', 'tag']) ok
+  release solo src/ contra main          efectivo=fix          -> ('release', False, 1, ['merge-pr', 'tag']) ok
+  hotfix contra main desde feature/*     efectivo=fix          -> ERROR ok
+  feature contra main                    efectivo=funcionalidad -> ERROR ok
+-- barrido: approve, merge-pr y tag en todo carril, tipo, base, modo y configuracion valida de D
+  combinaciones=1440 ; continuar=0 ; tipo y base que fallan cerrado (sin fila)=140
+RESULTADO C ... ; tipo y base 8/8 ; continuar sobre approve/merge-pr/tag=0 ; paradas distintas de la politica=0
 ```
 
 Lectura:
 
 - **Nunca hay `continuar` sobre `approve`, `merge-pr` o `tag`**, ni en las
-  tareas reales ni en las 900 combinaciones de carril, tipo, modo y
+  tareas reales ni en las 1440 combinaciones de carril, tipo, base, modo y
   configuración válida; y en modo automático las paradas fijas que ve cada
   tarea son exactamente las de su fila de la tabla.
+- **El tipo no baja la puerta** (revisión 1, IMP-1): `tipo: hotfix` o
+  `tipo: release` solo eligen su fila con la base real `main` (y en hotfix,
+  la rama `hotfix/*`); contra `develop` manda el carril efectivo, y un diff
+  que toca `.github/` o `.taskcode/` exige también contra `main` el plan
+  firmado y las dos aprobaciones de riesgo alto (8/8 casos).
 - **Las paradas por tarea suben** (0,85 → 2,52 en ToolsaWeb). No las añade
   este ADR: son las dos puertas de F1-5 y F1-6 que la 0.16 no tiene. Hoy, en
-  automático, el agente mergea en local (los 156 `finish` en automático de
-  ToolsaWeb, sin parada) y 114 de los 157 `approve` en automático de
-  ToolsaWeb los hizo el propio flujo (`decidido_por: automatico`, H5). La 2.0 suma `merge-pr` de persona en toda
-  tarea y el `approve` firmado en funcionalidad y riesgo alto.
-- Lo que este ADR sí puede bajar: una tarea declarada errata o fix no pasa
-  por `approve` (23 tareas de solo `docs/` en A), y un lote (F1-16) agrupa
-  las de errata, fix y funcionalidad con plan aprobado en un solo `merge-pr`
-  al final (RR-18). Ver «Divergencia con el plan», 3.
+  automático, el agente mergea en local (`finish=156` en ToolsaWeb, sin
+  parada) y en 114 tareas el `approve` lo hizo el propio flujo
+  (`decidido_por: automatico`, H5; 43 los decidió una persona). Recuentos
+  sobre las tareas terminadas del commit fijo, tal como los imprime la PoC.
+  La 2.0 suma `merge-pr` de persona en toda tarea y el `approve` firmado en
+  funcionalidad y riesgo alto.
+- **Cota inferior**: si cada tarea se hubiera declarado con su carril mínimo
+  por ficheros (solo `docs/` → errata; código → fix; sin `approve` salvo
+  riesgo alto), las paradas serían 1,52 por tarea en ToolsaWeb, 2,16 en
+  OpenDockTrading y 1,22 en TaskCodeTwo: la diferencia con 2,52 / 3,19 /
+  2,22 es el `approve` de las tareas que se declaren funcionalidad. El lote
+  (F1-16) puede además agrupar errata, fix y funcionalidad con plan
+  aprobado en un solo `merge-pr` al final (RR-18). Ver «Divergencia con el
+  plan», 3.
 
 ### D — Configuración que solo endurece
 
@@ -244,11 +269,12 @@ Lectura:
   riesgo-alto entra en lote              -> rechazada: ...lote: solo admite false
   fix con perfil rapida                  -> rechazada: 'rapida' por debajo de 'completa'
   politica de release (es un tipo)       -> rechazada: carril desconocido
+  paradas_extra con una lista dentro     -> rechazada: ...paradas_extra: solo fases de ['review', 'start']
   (y 7 más)
 -- PR que cambia .taskcode/config.yml para bajar la politica
   carril minimo=riesgo-alto (.taskcode/config.yml) ; config de la rama valida=False ;
   politica con la de la base: plan=True paradas=['approve', 'merge-pr']
-RESULTADO D aceptadas=5/5 ; rechazadas=16/16
+RESULTADO D aceptadas=5/5 ; rechazadas=17/17
 ```
 
 ### Divergencia con el plan
@@ -260,8 +286,9 @@ RESULTADO D aceptadas=5/5 ; rechazadas=16/16
    calcular la política del carril efectivo cuando sube; en el núcleo la
    calcula quien llama y `accion` recibe la del efectivo (decisión 1).
 3. **Las paradas humanas por tarea suben** frente a hoy (C: 0,85 → 2,52 en
-   ToolsaWeb, 2,12 → 3,19 en OpenDockTrading, 1,22 → 2,22 en TaskCodeTwo). El
-   plan esperaba medirlo, no este resultado. Va contra la letra de RR-07
+   ToolsaWeb, 2,12 → 3,19 en OpenDockTrading, 1,22 → 2,22 en TaskCodeTwo;
+   cota inferior, con cada tarea declarada con su carril mínimo: 1,52, 2,16
+   y 1,22). El plan esperaba medirlo, no este resultado. Va contra la letra de RR-07
    («pocas puertas humanas»), pero las dos paradas que lo explican
    (`merge-pr` y `approve` firmado) son puertas ya aceptadas en F1-5 y F1-6
    que la 0.16 se salta (H5, H6). Lo contienen la errata y el fix sin plan
@@ -279,6 +306,17 @@ RESULTADO D aceptadas=5/5 ; rechazadas=16/16
    carril»): decide cuántos roles lleva una funcionalidad, no el carril, que
    declara el plan (D2a) y sube por ficheros. Ver «Divergencias con F1-5,
    F1-6 y F1-9».
+8. **Revisión 1** (Carlos, con la consigna «la opción recomendada»):
+   **IMP-1**, el `tipo` de `tarea.md` (no confiable) elegía la fila de
+   release o hotfix en cualquier PR y podía quitar el plan firmado y la
+   segunda aprobación; ahora solo vale con la base real `main` (y la rama
+   `hotfix/*` en hotfix), y el mínimo por ficheros también se aplica a
+   release y hotfix (decisiones 1 y 4; PoC C, 8/8). **MEN-1**, `paradas_extra`
+   con un valor que no es texto se rechaza con mensaje (PoC D). **MEN-2**, el
+   suelo compara el primer componente de la ruta (`.github` y `.taskcode`
+   exactos también). **MEN-3**, la PoC imprime los recuentos de `finish` y
+   `approve` que cita el ADR y la cota inferior. **MEN-4**, el número de
+   roles lo fija `plan`, no `politica` (decisión 1).
 
 ### No verificado
 
@@ -320,15 +358,23 @@ existe aún):
      Cambio = {estado: A|M|D|R|C|T, rutas: [ruta] | [ruta_vieja, ruta_nueva]}
    carrilEfectivo(declarado: Carril, minimo: Carril) -> Carril          # el mayor; solo sube
    validarConfig(config) -> Error[]                                      # vacía = válida
-   politica(carril: Carril, tipo: Tipo, config: ConfigValidada) -> Politica
+   politica(carril: Carril, tipo: Tipo, base: Rama, rama: Rama, config: ConfigValidada) -> Politica | ErrorCarril
      Politica = {base, roles_min, plan, revisoras, perfil, checks, aprobaciones, paradas, lote}
-   accion(fase: Fase, tarea: {declarado, efectivo, tipo, ronda, tope}, politica: Politica, modo: Modo)
+   accion(fase: Fase, tarea: {declarado, efectivo, tipo, base, rama, ronda, tope}, politica: Politica, modo: Modo)
      -> {accion: detener|preguntar|continuar, motivo}
    ```
 
    `Carril` = `errata | fix | funcionalidad | riesgo-alto` (ordenados);
-   `Tipo` ∈ los de la tarea, y con `release` o `hotfix` la fila es la del
-   tipo; `Fase` = `plan | approve | start | review | merge-pr | tag`;
+   `Tipo` ∈ los de la tarea. **El tipo es texto de `tarea.md` (no
+   confiable) y solo elige fila con la base real del PR** (la de la API en
+   `carril`; la del PR que abre `review` en local): `release` con base
+   `main`, y `hotfix` con base `main` y rama `hotfix/*`. En un PR contra
+   `develop` el tipo se ignora y la fila es la del carril efectivo; un PR
+   contra `main` que no sea release ni hotfix (o un hotfix desde otra rama)
+   no tiene fila y `carril` falla cerrado. En una fila de `main`, si el
+   carril efectivo es riesgo alto (mínimo por ficheros, decisión 4), se
+   exige además lo de riesgo alto: plan con `approve` firmado, dos
+   aprobaciones, `revision-ia` y la parada `approve`. `Fase` = `plan | approve | start | review | merge-pr | tag`;
    `politica` recibe la política del carril **efectivo**. La usan
    `siguiente` y el orquestador (F4-4), `plan` (F4-1, F4-6), el check
    `carril` (F6), `lote` (F1-16) y `doctor` (F3-8). `siguiente --json`
@@ -342,14 +388,16 @@ existe aún):
    | `fix` | `develop` | 1 (arquitectura) | no | por dominio del diff | completa | `verificacion`, `revision-ia`, `carril` | 1 (PR) | `merge-pr` | sí |
    | `funcionalidad` | `develop` | 1 a 4, según la complejidad | sí, `approve` firmado | por dominio del diff | completa | `verificacion`, `revision-ia`, `carril` | 2 (plan y PR) | `approve`, `merge-pr` | sí, con el plan aprobado antes de lanzar el lote |
    | `riesgo-alto` | `develop` | todos (rendimiento si la etiqueta lo pide) | sí, `approve` firmado | por dominio del diff | completa | `verificacion`, `revision-ia`, `carril` | 2 (plan y PR; D6) | `approve`, `merge-pr` | no |
-   | `release` (tipo) | `main` | 0 (sin plan) | no | ninguna | completa y pesada | `verificacion`, `carril` | 1 (PR) | `merge-pr`, `tag` | no |
-   | `hotfix` (tipo) | `main` | 1 (arquitectura) | no | por dominio del diff | completa | `verificacion`, `revision-ia`, `carril` | 1 (PR) | `merge-pr`, `tag` | no |
+   | `release` (tipo) | `main` (base real del PR) | 0 (sin plan) | no | ninguna | completa y pesada | `verificacion`, `carril` | 1 (PR) | `merge-pr`, `tag` | no |
+   | `hotfix` (tipo) | `main` (base real del PR), rama `hotfix/*` | 1 (arquitectura) | no | por dominio del diff | completa | `verificacion`, `revision-ia`, `carril` | 1 (PR) | `merge-pr`, `tag` | no |
    <!-- tabla-politica:fin -->
 
    Columnas: «Checks» y «Aprobaciones» son las de F1-5 (no cambian); el
    «Perfil» es el de F1-7 por la base del PR; «Roles» sigue F4-1 del plan
-   («uno en fix, todos en riesgo alto»; en funcionalidad, el número lo da la
-   complejidad, como hoy en la 0.16); «Paradas» son las fijas de la decisión
+   («uno en fix, todos en riesgo alto») y es el **mínimo** que devuelve
+   `politica` (`roles_min`): el número exacto de roles lo fija `plan` (F4-1)
+   según la complejidad, fuera de `politica` y nunca por debajo de ese
+   mínimo (en funcionalidad, de 1 a 4, como hoy en la 0.16; divergencia 7); «Paradas» son las fijas de la decisión
    6 que tocan a la fila (además, en todas: tope de rondas y subida de
    carril); «Lote» dice si la tarea puede ir en un lote de F1-16. La PoC B
    compara esta tabla con F1-5 y F1-7.
@@ -377,20 +425,24 @@ existe aún):
    de la tarea; el check `carril` la lee de la base por la API desde
    `pull_request_target` (F1-8 D1b, parcial). El carril declarado en
    `tarea.md` es texto no confiable (F1-8, `A10`; F1-9): solo sirve para
-   subir. Las paradas locales del orquestador son de usabilidad (evitar
+   subir; el `tipo`, igual, solo elige la fila de release o hotfix con la
+   base real `main` (decisión 1). Las paradas locales del orquestador son de usabilidad (evitar
    trabajo que la puerta rechazará); quien impone es `carril`, con la misma
    función. Un PR que cambia `.taskcode/` sale riesgo alto por el suelo, y su
    configuración nueva no se aplica hasta que esté en la base (PoC D).
 4. **Derivación del carril mínimo.** Entrada: los ficheros del PR por la API
    (en local, `git diff -M --name-status` contra la base); en cada
    renombrado o copia, **los dos nombres** (`previous_filename`). Por cada
-   ruta: `.github/`, `.taskcode/` o un `CODEOWNERS` en cualquier carpeta
-   (comparando sin mayúsculas) → riesgo alto; un glob de
+   ruta: primer componente `.github` o `.taskcode` (también la ruta exacta:
+   un fichero, enlace o submódulo con ese nombre) o un `CODEOWNERS` en
+   cualquier carpeta (comparando sin mayúsculas) → riesgo alto; un glob de
    `rutas_carril.riesgo-alto` → riesgo alto; uno de `rutas_carril.fix` → fix;
    bajo `docs/` exacto → errata; el resto → fix. El carril mínimo es el mayor
    de todas las rutas. **Nunca sale funcionalidad** (D2a). Una lista vacía,
    una ruta con `..` o una lista que no se puede leer entera (la API corta
    a partir de 3000 ficheros, no verificado) → `carril` falla, no decide. El
+   mínimo se aplica a todos los PR, también a los de release y hotfix contra
+   `main` (decisión 1: suben a lo de riesgo alto). El
    tamaño del diff no cuenta: un PR enorme de solo `docs/` sigue siendo
    errata (lo revisa `revision-ia`; F1-17 usa el tamaño para estimar).
 5. **Qué congela la tarea.** En `plan` se congelan el **modo** (como en la
@@ -413,14 +465,14 @@ existe aún):
 
    `accion` resuelve `approve`, `merge-pr` y `tag` **antes** de mirar la
    política o el modo, así que ninguna configuración las convierte en
-   `continuar` (PoC C: 900 combinaciones, 0 `continuar`). Fuera de esas y de
+   `continuar` (PoC C: 1440 combinaciones, 0 `continuar`). Fuera de esas y de
    las `paradas_extra`, en `automatico` todo se encadena; `manual` devuelve
    `detener` en cada fase y `semiautomatico`, `preguntar`. Las guardas del CLI
    de la 0.16 que no son de política (lo que se mergearía no coincide con lo
    revisado, revisor independiente) siguen en `review` y `finish`.
 7. **Lo que cierra además** (D1–D7 del plan):
    - **D1a** `carril` ∈ `errata < fix < funcionalidad < riesgo-alto`;
-     release y hotfix, por `tipo`.
+     release y hotfix, por `tipo`, solo con base real `main` (decisión 1).
    - **D2a** funcionalidad solo se declara en `plan`.
    - **D3a** la release la lanza la persona: `release` abre el PR
      `develop`→`main` con la identidad del agente (F1-6), y el `merge-pr` y
@@ -492,18 +544,19 @@ Los ADR aceptados no se modifican; lo que cambia queda aquí y en
 
 | Requisito | Decisión | Evidencia |
 |---|---|---|
-| Criterio de F1-10: política por carril, configurable | tabla única (decisión 1) y dos claves que solo endurecen (decisión 2) | PoC B (6 filas, 0 fallos, 12/12 desvíos detectados); PoC D (5/5 aceptadas, 16/16 rechazadas) |
+| Criterio de F1-10: política por carril, configurable | tabla única (decisión 1) y dos claves que solo endurecen (decisión 2) | PoC B (6 filas, 0 fallos, 12/12 desvíos detectados); PoC D (5/5 aceptadas, 17/17 rechazadas) |
 | P7 / RR-07 carriles proporcionales; el automático para solo en las puertas del carril | paradas fijas (decisión 6); errata y fix sin `approve` | PoC C (paradas = fila de la tabla en todas las tareas; 0 `continuar` en fijas); las paradas por tarea suben (divergencia 3) |
 | RF-21 modos con encadenado | `accion` en `siguiente --json`, modo congelado | PoC C (manual, semiautomático y automático en las tareas reales) |
 | RR-08 garantías en la plataforma | `carril` impone con la misma función; política de la base | decisión 3; PoC D (PR que cambia `.taskcode/`); GitHub: no verificado (F2-6) |
-| RR-04 / RF-50 aprobación de una persona antes del merge | `merge-pr` y `approve` como paradas fijas | PoC C (900 combinaciones, 0 `continuar`) |
-| H5 / H6 aprobación comprobable; el agente no se salta puertas | `approve` firmado y `merge-pr` de persona en cualquier configuración | PoC C (114 `approve` automáticos de hoy pasan a `detener`); PoC D |
+| RR-04 / RF-50 aprobación de una persona antes del merge | `merge-pr` y `approve` como paradas fijas | PoC C (1440 combinaciones, 0 `continuar`) |
+| H5 / H6 aprobación comprobable; el agente no se salta puertas | `approve` firmado y `merge-pr` de persona en cualquier configuración | PoC C (los 114 `approve` con `decidido_por: automatico` de ToolsaWeb pasan a `detener`); PoC D |
 | RR-02 / RR-17 roles y revisoras proporcionales al carril | columnas «Roles» y «Revisoras» | PoC B (uno en fix, todos en riesgo alto); contenido en F1-15 |
 | RR-18 / RF-27 lotes con revisión humana al final | columna «Lote» | tabla; diseño en F1-16 |
 | RF-33 tope de rondas | parada `preguntar` con las salidas de la 0.16 | PoC C (tope=69 / 40 / 2) |
 | RF-51 `## Resultado` | sin cambio: lo exige `carril` (F1-5) | — |
 | RN-06 una persona con agentes | D4a de F1-5 y D7a | D7a no verificado (F2-6) |
-| Derivación con renombrados (F1-5) | los dos nombres; nunca funcionalidad | PoC A (203 tareas: 0 funcionalidad; 19/19 casos, `.github/` → `docs/` = riesgo alto) |
+| RR-08 / F1-8 `A10` lo escrito en `tarea.md` no baja una puerta (revisión 1, IMP-1) | `tipo` solo elige fila con base real `main` (y rama `hotfix/*`); contra `develop`, el carril efectivo; el mínimo por ficheros también en release y hotfix | PoC C (8/8: `tipo: hotfix` con diff en `.github/` sale riesgo alto contra `develop` y hotfix + riesgo alto contra `main`; tipo y base sin fila fallan cerrado) |
+| Derivación con renombrados (F1-5) | los dos nombres; nunca funcionalidad | PoC A (203 tareas: 0 funcionalidad; 22/22 casos, `.github/` → `docs/` = riesgo alto) |
 | Preguntas abiertas de F1-3, F1-4, F1-5, F1-6, F1-8 y F1-9 | tabla «Preguntas heredadas» (15 filas, ninguna sin destino) | este ADR |
 
 ## Consecuencias
@@ -564,6 +617,10 @@ Este ADR se planificó con un solo rol (arquitectura); quedan sin cubrir:
 - **Configuración inválida en la base**: falla cerrado bloquea todo PR,
   también el que la arregla; el que toca `.taskcode/` tendría que poder
   pasar con las puertas de riesgo alto. Lo concreta F2-7.
+- **Release con riesgo alto**: una release cuyo diff acumulado toca
+  `.taskcode/` (4 de las 160 tareas de ToolsaWeb lo cambiaron) exige plan
+  firmado, y F1-3 no le da tarea; F1-5 ya hace fallar todo PR sin tarea.
+  Cómo se da tarea y plan a una release lo concretan F1-3 y F7.
 - **GitLab**: la misma derivación sobre el diff del MR y la lectura de la
   configuración de la rama destino (F1-13).
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff e83f838c05980a2115e24949afbd7896e042bdc2..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-011/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-011/planificacion/plan-final.md                 |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-011/poc/poc-f1-10.log                           |  34 ++++--
 tareas/{02-en-curso => 03-en-revision}/TASK-011/poc/poc-f1-10.py                            | 143 +++++++++++++++++-----
 tareas/03-en-revision/TASK-011/revision/informe-revision-1.md                               |  21 ++++
 tareas/03-en-revision/TASK-011/revision/peticion-revision-1.md                              | 667 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-011/tarea.md                                    |  11 +-
 7 files changed, 833 insertions(+), 43 deletions(-)
````

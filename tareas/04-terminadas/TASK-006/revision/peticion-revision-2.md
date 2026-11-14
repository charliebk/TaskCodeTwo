# Peticion de revision — TASK-006 (ronda 2)

- Tarea: TASK-006 — F1-5 ADR puertas en GitHub
- Rama revisada: feature/task-006-f1-5-adr-puertas-en-github
- Rama base: develop
- Commit revisado (HEAD): 3e277420be9417232556523146d737d5f2468aa3
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-006 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde d05805a37a2b99f58dbfe9c0180713914bbb188d (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (6 hallazgo(s)))

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

- tareas/03-en-revision/TASK-006/tarea.md
- tareas/03-en-revision/TASK-006/planificacion/
- docs/adr/F1-5-puertas-en-github.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 4 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-5-puertas-en-github.md:283 | informe-revision-1.md |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-5-puertas-en-github.md:265 | informe-revision-1.md |

## Commits a revisar (git log d05805a37a2b99f58dbfe9c0180713914bbb188d..HEAD)

````
3e27742 chore(TASK-006): constancia de autorrevision ronda 2
cbe4d2d docs(adr): F1-5 corrige los hallazgos de la revision 1 (carril no falsificable, strict en main, menores)
f5b3eac chore(TASK-006): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff d05805a37a2b99f58dbfe9c0180713914bbb188d..HEAD)

````diff
diff --git a/docs/adr/F1-5-puertas-en-github.md b/docs/adr/F1-5-puertas-en-github.md
index f06e008..12bdbcf 100644
--- a/docs/adr/F1-5-puertas-en-github.md
+++ b/docs/adr/F1-5-puertas-en-github.md
@@ -43,6 +43,7 @@ qué pasa en una cuenta gratuita, sin cola de merge.
 | **Rulesets** | Reglas de repositorio (`/repos/{o}/{r}/rulesets`): varias por repo, por patrón de rama o de tag, con checks atados a una integración. | **Elegida**: cubren ramas y tags con el mismo modelo, admiten `integration_id` y `allowed_merge_methods`, y se leen con `GET /rules/branches/{rama}` (lo que `doctor` necesita). |
 | Protección clásica de ramas | `/branches/{rama}/protection`, una por rama. | Descartada: no protege tags ni patrones con la misma API y no tiene método de merge por rama; además, en el repo privado gratuito responde el mismo 403 que los rulesets (PoC A), así que no salva D2. |
 | **Carril en un check** | El ruleset pone un suelo común por rama; un check obligatorio `carril` (workflow) impone lo propio de cada carril. | **Elegida**: un ruleset filtra por rama, no por PR ni por tarea; el carril vive en la tarea (rama de estado, F1-1). |
+| Carril declarado solo en la tarea | `carril` cree el carril y la marca de plan aprobado del frontmatter de `taskcode/tareas`. | Descartada (revisión 1): el agente puede escribir en esa rama y bajar su carril o marcar el plan como aprobado; se sustituye por la firma verificada del `approve` y el carril mínimo por ficheros (decisión 4). |
 | Carril en el nombre de la rama | Un ruleset por prefijo (`errata/*`, `riesgo/*`…) con sus aprobaciones. | Descartada: el agente elige el nombre de la rama y con él la puerta; y los rulesets se aplican a la rama **destino** del PR, no a la de origen. |
 | Carril con etiquetas del PR | Reglas según la etiqueta. | Descartada: los rulesets no filtran por etiqueta y el agente puede poner o quitar etiquetas. |
 | **D1a** Evidencia de solo lectura | `GET` de rulesets y esquema OpenAPI; nada se escribe en GitHub. | **Elegida**. Descartadas: (b) crear un ruleset en el repo temporal, (c) hacerlo público, (d) aplicarlos en este repo (rompe el flujo 0.16, RR-14). |
@@ -50,7 +51,7 @@ qué pasa en una cuenta gratuita, sin cola de merge.
 | **D3a** Cola de merge | `taskctl merge` por la API; la cola (`merge_queue`) solo si el repo es de una organización. | **Elegida**. |
 | **D4a** Riesgo alto con una persona | Cuentan como las dos aprobaciones la de la persona en el plan y la suya en el PR (RN-06). | **Elegida**. |
 | **D5a** `revision-ia` en errata | También se ejecuta, con perfil rápido (P3 la pide en todo PR). | **Elegida**. |
-| **D6a** Rama al día sin cola | `strict_required_status_checks_policy: true` en `develop` (y en `main`). | **Elegida**. |
+| **D6a** Rama al día sin cola | `strict_required_status_checks_policy: true` en `develop`. | **Elegida**. En `main`, `false` (decisión 2; revisión 1). |
 | Checks sin origen | Solo el nombre (`context`) del check. | Descartada: cualquier identidad con permiso de escribir checks (también la del agente) publicaría un `revision-ia` verde con ese nombre. |
 
 ## Pruebas de concepto
@@ -59,14 +60,14 @@ Guion `poc-f1-5.sh`, su salida completa `poc-f1-5.log` y los JSON generados
 (`poc/rulesets/*.json`), en la carpeta `poc/` de TASK-006 (sin enlace: la
 carpeta de la tarea cambia de estado al cerrarse). Ejecutado en Git Bash con
 `gh` 2.95.0 y Python 3.12 (`python -I`) bajo
-`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-006\poc\` (borrado al
-terminar); el esquema se descarga en su propia carpeta vacía (`esquema/`).
+`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-006\poc\` y, tras la
+revisión 1, `...\task-006-c1\` (borradas al terminar); el esquema se descarga en su propia carpeta vacía (`esquema/`).
 **Solo lectura en GitHub**: toda llamada pasa por una función que usa
 `gh api --method GET`, y el guion las registra.
 
 ```bash
-bash poc-f1-5.sh "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-006/poc" \
-  docs/adr/F1-5-puertas-en-github.md tareas/02-en-curso/TASK-006/poc/rulesets > poc-f1-5.log 2>&1
+bash poc-f1-5.sh "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-006-c1" \
+  docs/adr/F1-5-puertas-en-github.md tareas/<estado>/TASK-006/poc/rulesets > poc-f1-5.log 2>&1
 ```
 
 ### A — Estado real en GitHub (solo `GET`)
@@ -84,7 +85,7 @@ rules/branches/develop:       HTTP 200 []
 branches/develop/protection:  HTTP 404 {"message":"Branch not protected", ...}
 TaskCodeTwo codeowners/errors: HTTP 404 {"message":"Not Found", ...}
 id de la App GitHub Actions: 15368
-RESULTADO A+B llamadas gh api con GET=10 ; gh api con otro metodo=0 ; gh repo view=2
+RESULTADO A+B llamadas gh api con GET=11 ; gh api con otro metodo=0 ; gh repo view=2   (incluye el GET de la sección F)
 ```
 
 Lectura:
@@ -129,7 +130,7 @@ comprueban que el validador rechaza lo que debe.
 develop          branch refs/heads/develop   reglas=deletion,non_fast_forward,pull_request,required_status_checks
                  checks=verificacion@15368,revision-ia@0,carril@15368 -> VALIDO
 main             branch refs/heads/main      reglas=deletion,non_fast_forward,pull_request,required_status_checks
-                 checks=verificacion@15368,carril@15368 -> VALIDO
+                 checks=verificacion@15368,carril@15368 -> VALIDO   (strict=false, decisión 2)
 ramas-de-tarea   branch refs/heads/feature/**,refs/heads/fix/**,refs/heads/hotfix/**  reglas=deletion,non_fast_forward -> VALIDO
 taskcode-tareas  branch refs/heads/taskcode/tareas  reglas=deletion,non_fast_forward -> VALIDO
 tags-version     tag    refs/tags/v*         reglas=deletion,update -> VALIDO
@@ -142,9 +143,19 @@ develop-cola-org branch refs/heads/develop   reglas=...,required_status_checks,m
   clave desconocida en la raiz (lane)                          -> rechazado: neg: clave desconocida lane
   falta strict_required_status_checks_policy                   -> rechazado: neg.rules[3]: oneOf casa con 0 variantes
   enforcement inventado (required)                             -> rechazado: neg.enforcement: 'required' fuera del enum
-RESULTADO C rulesets=6 errores=0 ; negativos rechazados=7/7 ; integration_id actions=15368 revisor=0 (relleno)
+-- JSON versionados en la carpeta de artefactos (antes de sobrescribirlos): validacion y comparacion
+  develop            VALIDO ; igual al generado
+  main               VALIDO ; igual al generado
+  (ramas-de-tarea, taskcode-tareas, tags-version, develop-cola-org: igual)
+RESULTADO C rulesets=6 errores=0 ; negativos rechazados=7/7 ; integration_id actions=15368 revisor=0 (relleno) ; versionados validos=6/6 iguales=6/6
 ```
 
+Los `rulesets/*.json` versionados son **salida del guion**, no fuente: antes de
+sobrescribirlos, el guion los valida con el mismo validador y los compara con
+los generados (revisión 1). En la primera ejecución tras pasar `main` a
+`strict=false`, `main.json` salió «DISTINTO del generado (se sobrescribe)»,
+como se esperaba; el log versionado es el de la segunda, con 6/6 iguales.
+
 `revision-ia@0`: la App del revisor aún no existe (la decide F1-6); el `0` es
 un relleno que el guion toma de `APP_REVISOR_ID` (divergencia 2).
 
@@ -152,28 +163,83 @@ un relleno que el guion toma de `APP_REVISOR_ID` (divergencia 2).
 
 El guion lee la tabla «Quién aprueba cada carril» de este ADR y, para cada
 fila, compara sus checks con los del ruleset de la base del PR más lo que
-impone el check `carril` (solo `revision-ia` en hotfix); exige además que la
-base tenga `carril` obligatorio, que «quién aprueba» diga «persona» y no
+impone el check `carril` (solo `revision-ia` en hotfix), y el **número de
+aprobaciones** (primer número de «Quién aprueba») con las del PR del ruleset
+más la del plan que exige `carril` (funcionalidad y riesgo alto; mínimo 2 en
+riesgo alto, D4a). Exige además que la base tenga `carril` obligatorio, que
+«Quién aprueba» diga «persona», mencione el plan solo donde se exige y no
 nombre ninguna identidad de IA, y que la base no tenga `bypass_actors`.
 
 ```text
-errata         base=develop  tabla=carril,revision-ia,verificacion  ruleset+carril=carril,revision-ia,verificacion  checks:si aprueba:persona, sin IA
-fix            base=develop  (igual)                                                                               checks:si aprueba:persona, sin IA
-funcionalidad  base=develop  (igual)                                                                               checks:si aprueba:persona, sin IA
-riesgo alto    base=develop  (igual)                                                                               checks:si aprueba:persona, sin IA
-release        base=main     tabla=carril,verificacion              ruleset+carril=carril,verificacion              checks:si aprueba:persona, sin IA
-hotfix         base=main     tabla=carril,revision-ia,verificacion  ruleset+carril=carril,revision-ia,verificacion  checks:si aprueba:persona, sin IA
+errata         base=develop  checks:si aprobaciones=1 (PR 1 + plan 0) aprueba:persona, sin IA, numero cuadra bypass_actors=[]
+fix            base=develop  checks:si aprobaciones=1 (PR 1 + plan 0) aprueba:persona, sin IA, numero cuadra bypass_actors=[]
+funcionalidad  base=develop  checks:si aprobaciones=2 (PR 1 + plan 1) aprueba:persona, sin IA, numero cuadra bypass_actors=[]
+riesgo alto    base=develop  checks:si aprobaciones=2 (PR 1 + plan 1) aprueba:persona, sin IA, numero cuadra bypass_actors=[]
+release        base=main     checks:si aprobaciones=1 (PR 1 + plan 0) aprueba:persona, sin IA, numero cuadra bypass_actors=[]
+hotfix         base=main     checks:si aprobaciones=1 (PR 1 + plan 0) aprueba:persona, sin IA, numero cuadra bypass_actors=[]
 -- controles negativos (tablas alteradas que la comprobacion tiene que detectar)
-  release con revision-ia              -> detectado (1 fallos)
-  fix sin carril                       -> detectado (1 fallos)
-  errata aprobada por el revisor IA    -> detectado (1 fallos)
-RESULTADO D filas=6 fallos=0 ; negativos detectados=3/3
+  release con revision-ia                  -> detectado (1 fallos)
+  fix sin carril                           -> detectado (1 fallos)
+  riesgo alto con 1 persona                -> detectado (1 fallos)
+  funcionalidad sin aprobacion del plan    -> detectado (1 fallos)
+  errata aprobada por el revisor IA        -> detectado (1 fallos)
+RESULTADO D filas=6 fallos=0 ; negativos detectados=5/5
 ```
 
 Lectura: la tabla y los rulesets dicen lo mismo, y la comparación detecta una
 tabla que se desvíe. Es una comprobación **de coherencia del documento**, no
 del comportamiento de GitHub.
 
+### E — Por qué `main` va sin `strict` (git local)
+
+Repo temporal, release por merge commit sin fast-forward como en F1-3:
+
+```text
+develop y main iguales: main ES ancestro de develop
+tras la release 1 (merge commit en main): main NO es ancestro de develop
+tras d2 en develop: main NO es ancestro de develop -> con strict, el PR develop->main de la release 2 saldria desactualizado
+RESULTADO E main ancestro de develop tras release por merge commit: no
+```
+
+Lectura: el hecho de ancestría está comprobado; que GitHub, con `strict`,
+marque ese PR como desactualizado es **no verificado** (conocimiento general;
+ningún ruleset aplicado). Por eso `main` lleva `strict=false` (decisión 2).
+
+### F — Carril no falsificable por el agente
+
+Un `GET` a un commit real y el esquema dicen qué da la API para la regla 1
+de la decisión 4; una simulación en Python aplica las dos reglas a casos
+fijos. La regla de derivación de la simulación es un **ejemplo** (la fija
+F1-10).
+
+```text
+commit de develop en TaskCodeTwo: sha=e8ee970802d9 verified=false reason=unsigned committer=charliebk commit.committer=carlos gallardo
+GET /repos/{o}/{r}/commits/{ref}: commit.verification = ['payload', 'reason', 'signature', 'verified', 'verified_at'] required [...]
+  campos de primer nivel con la cuenta de GitHub: ['author', 'committer']
+  plan: approve firmado y verificado, committer persona    -> cuenta
+  plan: approve sin firma (lo escribe el agente)           -> no cuenta
+  plan: approve verificado pero de la App del agente       -> no cuenta
+  plan: approve verificado por GitHub (web-flow, por API)  -> no cuenta
+  carril: declarado=errata       ficheros=docs/guia.md                  -> errata
+  carril: declarado=errata       ficheros=src/main.rs                   -> fix
+  carril: declarado=fix          ficheros=.github/workflows/carril.yml  -> riesgo alto
+  carril: declarado=riesgo alto  ficheros=docs/guia.md                  -> riesgo alto
+  carril: declarado=None         ficheros=README.md                     -> errata
+  carril: declarado=None         ficheros=src/main.rs                   -> FALLA (necesita tarea de carril >= fix)
+RESULTADO F esquema verification=si ; plan 4/4 casos ; carril efectivo 6/6 casos (el agente solo puede subirlo)
+```
+
+Lectura:
+
+- El commit real de `develop` **no está firmado** (`unsigned`) y aun así la
+  API le asigna la cuenta `committer=charliebk` (por el correo): la cuenta
+  sola no prueba nada; la regla exige `verified: true` **y** cuenta de
+  persona. El flujo de la 0.16 no firma, así que F4-2 tiene que añadir la
+  firma.
+- La simulación comprueba la lógica de las reglas, no GitHub: que la API
+  marque `verified` como se espera en un commit firmado por la persona es
+  **no verificado** (no hay ningún commit firmado que leer).
+
 ### Divergencias con el plan
 
 1. **Claves que el esquema exige y el plan no decidía.** `pull_request` exige
@@ -192,6 +258,18 @@ del comportamiento de GitHub.
 4. **La protección clásica también da 403** en el repo privado gratuito (el
    plan solo lo esperaba de los rulesets): no hay alternativa dentro de la
    cuenta gratuita, lo que refuerza D2a.
+5. **`main` sin `strict`** (revisión 1, IMP-2). El plan decía «`main`:
+   igual» que `develop`, y D6a solo hablaba de `develop`; con releases por
+   merge commit, `strict` en `main` bloquearía cada release desde la segunda
+   (PoC E). Se corrige en el modelo, en `main.json` y en la decisión 2.
+6. **El check `carril` no se fía de la rama de estado** (revisión 1, IMP-1).
+   El plan decía que `carril` «lee el carril de la tarea en
+   `taskcode/tareas`», que el agente puede escribir. Carlos decide, con la
+   consigna «la opción recomendada», la firma verificada del `approve` y el
+   carril efectivo por ficheros (decisión 4; PoC F).
+7. **La tabla de carriles dice el número de aprobaciones** y la PoC D lo
+   comprueba (revisión 1, MEN-2); la errata sale de `fix/errata-<slug>`
+   (MEN-4).
 
 ### No verificado
 
@@ -211,6 +289,14 @@ del comportamiento de GitHub.
   por target): validados contra el esquema, no enviados.
 - **Lo que pinta la interfaz de GitHub** (sección siguiente): conocimiento
   general, sin ruleset aplicado que mirar.
+- **Que GitHub marque desactualizado el PR `develop`→`main` con `strict`**
+  tras una release, y que sin `strict` lo deje mergear (PoC E solo prueba la
+  ancestría).
+- **La regla 1 del carril en GitHub real**: que un commit de `approve`
+  firmado por la persona salga `verified: true` con su cuenta, y que no haya
+  forma de que otra identidad lo consiga (PoC F solo simula la regla).
+- **La regla de derivación del carril mínimo**: la de la PoC F es un ejemplo
+  (F1-10).
 
 ## Cómo se ve en GitHub
 
@@ -225,7 +311,9 @@ ha aplicado ningún ruleset):
 |---|---|
 | Settings → Rules → Rulesets | cinco rulesets activos: `taskcode-develop`, `taskcode-main`, `taskcode-ramas-de-tarea`, `taskcode-taskcode-tareas` y `taskcode-tags-version` (más `taskcode-develop-cola` en vez de `taskcode-develop` si el repo es de una organización y usa la cola), sin actores con bypass |
 | Caja de merge de un PR a `develop` | `verificacion`, `revision-ia` y `carril` marcados como obligatorios; «Review required» hasta la aprobación de una persona; «Code owner review required» si el PR toca `.github/` o `.taskcode/`; «This branch is out-of-date» con el botón de actualizar si `develop` avanzó (D6a); un solo método: merge commit |
-| Check `carril` | un check más en la lista, con un resumen de una línea: carril de la tarea y qué le falta («falta la aprobación del plan», «falta `## Resultado`») o «no aplica» |
+| Check `carril` | un check más en la lista, con un resumen de una línea: carril efectivo (declarado y mínimo por ficheros) y qué le falta («falta la aprobación del plan», «falta `## Resultado`») o «no aplica» |
+| PR `develop`→`main` de una release | sin aviso de rama desactualizada aunque `main` lleve merges de releases anteriores (sin `strict`; no verificado) |
+| Commit de `approve` en `taskcode/tareas` | con la etiqueta «Verified» si lo firmó la persona; sin ella, `carril` no cuenta la aprobación del plan |
 | Push directo a `develop` o `main` | rechazado por el ruleset (también el de la persona: no hay bypass) |
 | `git push --force` o borrar `feature/task-…`, `taskcode/tareas` o un tag `v*` | rechazado |
 | Push del CLI a `taskcode/tareas` | aceptado (sin PR ni checks), siempre fast-forward |
@@ -240,8 +328,8 @@ ha aplicado ningún ruleset):
    develop          rama  sin_borrado sin_forzar solo_pr(aprobaciones=1, descartar_viejas, ultimo_push,
                           propietarios, metodos=[merge]) al_dia checks=[verificacion<-actions,
                           revision-ia<-app-revisor, carril<-actions]
-   main             rama  igual que develop, checks=[verificacion<-actions, carril<-actions]
-   ramas de tarea   rama  feature/**, fix/**, hotfix/**: sin_borrado sin_forzar
+   main             rama  igual que develop pero sin al_dia, checks=[verificacion<-actions, carril<-actions]
+   ramas de tarea   rama  feature/**, fix/** (también la errata), hotfix/**: sin_borrado sin_forzar
    taskcode/tareas  rama  sin_borrado sin_forzar (push directo del CLI)
    v*               tag   sin_borrado sin_mover
    ```
@@ -263,11 +351,21 @@ ha aplicado ningún ruleset):
      F1-3); checks obligatorios `verificacion`, `revision-ia` y `carril`;
      rama al día (`strict`, D6a).
    - **`main`**: igual, con `verificacion` (perfil completo; el workflow elige
-     el perfil por la base del PR) y `carril`. `revision-ia` no es
-     obligatorio en el ruleset de `main` porque una release no la necesita;
-     en un hotfix la exige `carril`.
+     el perfil por la base del PR) y `carril`, pero **sin rama al día**
+     (`strict_required_status_checks_policy: false`). Motivo: con releases
+     por merge commit (D3a de F1-3), tras la primera release `main` deja de
+     ser ancestro de `develop` (PoC E); con `strict`, el PR `develop`→`main`
+     de cada release siguiente saldría desactualizado, y actualizarlo es
+     empujar a `develop`, que solo admite PR: cada release exigiría antes un
+     PR `main`→`develop` con todas sus puertas. Cómo trata GitHub ese PR sin
+     `strict`: no verificado. `revision-ia` no es obligatorio en el ruleset de
+     `main` porque una release no la necesita; en un hotfix la exige `carril`.
    - **`feature/**`, `fix/**`, `hotfix/**`**: sin borrado ni force-push (las
-     ramas se conservan, F1-3; la capa no fuerza nunca, F1-4).
+     ramas se conservan, F1-3; la capa no fuerza nunca, F1-4). **La errata
+     sale de una rama `fix/errata-<slug>`** (prefijo de F1-3, sin ID de
+     tarea), así que queda cubierta por `fix/**`. Una rama con otro prefijo
+     no la crea el CLI y no está protegida; `carril` trata su PR como
+     cualquier otro (carril efectivo de la decisión 4).
    - **`taskcode/tareas`**: sin borrado ni force-push; sin PR ni checks; push
      directo del CLI (F1-1, F1-2).
    - **Tags `v*`**: ni borrar ni mover (`deletion` + `update`).
@@ -280,13 +378,38 @@ ha aplicado ningún ruleset):
 4. **El carril no se expresa con un ruleset** (filtra por rama destino, no por
    PR ni por tarea). El ruleset pone el suelo común de la rama y el check
    **`carril`**, un workflow, impone lo propio de cada carril:
-   - lee el carril de la tarea en `taskcode/tareas`; un PR sin tarea es
-     errata (sus límites los fija F1-10);
+   - **no se fía de lo que el agente puede escribir.** `taskcode/tareas` no
+     tiene PR ni restricción de quién empuja (cualquier identidad con
+     escritura, también la del agente, hace un push fast-forward), así que
+     ni el carril declarado ni una marca de «plan aprobado» en el
+     frontmatter bastan. Dos reglas (decididas por Carlos con la consigna
+     «la opción recomendada», a raíz de la revisión 1):
+     1. **Aprobación del plan = commit firmado.** Solo cuenta si el commit de
+        `approve` en `taskcode/tareas` tiene **firma verificada por la
+        plataforma** (`commit.verification.verified: true` en
+        `GET /repos/{o}/{r}/commits/{sha}`) y su firmante (la cuenta
+        `committer` que devuelve la API) es una identidad de persona del
+        proyecto; una identidad de agente o de IA, un commit sin firma o uno
+        firmado por la propia plataforma (`web-flow`) no valen. Es también
+        la base de la segunda aprobación de riesgo alto (D4a).
+     2. **Carril efectivo = el mayor** entre el declarado en la tarea y un
+        **carril mínimo derivado de los ficheros del PR** (que `carril` lee
+        de la API, no de la rama): por ejemplo, un cambio en `.github/`,
+        `.taskcode/`, CODEOWNERS o fuera de `docs/` no puede ser errata. El
+        agente solo puede **subir** el carril, nunca bajarlo. Orden:
+        errata < fix < funcionalidad < riesgo alto. Un PR sin tarea solo es
+        errata si su mínimo es errata; si no, falla cerrado («necesita
+        tarea»). La regla de derivación concreta la fija F1-10.
+
+     La PoC F simula las dos reglas y comprueba en el esquema que la API da
+     `verification.verified`; el comportamiento real en GitHub, **no
+     verificado**;
    - se relanza con cada review, cada edición de la descripción y cada push;
-   - comprueba: plan aprobado por una persona (F4-2) en funcionalidad y
-     riesgo alto; las dos aprobaciones de persona de riesgo alto (D4a);
-     `## Resultado` en la descripción del PR en los carriles con tarea
-     (RF-51); en un PR a `main` desde `hotfix/*`, `revision-ia` en verde;
+   - comprueba, sobre el carril efectivo: plan aprobado (regla 1, F4-2) en
+     funcionalidad y riesgo alto; las dos aprobaciones de persona de riesgo
+     alto (D4a); `## Resultado` en la descripción del PR en los carriles con
+     tarea (RF-51); en un PR a `main` desde `hotfix/*`, `revision-ia` en
+     verde; si no puede decidir algo (API caída, tarea ilegible), falla;
    - **solo cuenta aprobaciones de personas** (no de Apps ni cuentas de
      máquina);
    - siempre informa: si no aplica, termina en verde con «no aplica»; nunca
@@ -297,14 +420,20 @@ ha aplicado ningún ruleset):
 6. **CODEOWNERS** en las ramas de código (`.github/CODEOWNERS`): `/.github/`
    (workflows, scripts de CI y el propio CODEOWNERS) y `/.taskcode/`
    (configuración) a cargo de la persona, con
-   `require_code_owner_review: true` en `develop` y `main`. Así el agente no
-   rebaja una puerta editando el workflow de `carril` o de `verificacion` en
-   su propio PR sin que la persona lo apruebe expresamente. Con una sola
-   persona coincide con la aprobación de siempre; en un equipo, obliga al
-   propietario (no verificado en la plataforma).
+   `require_code_owner_review: true` en `develop` y `main`. Es una
+   **mitigación parcial**: un cambio en un workflow o en la configuración no
+   se integra sin la aprobación del propietario, que lo ve en el diff; pero
+   con `pull_request` GitHub ejecuta el workflow de la rama del PR, así que
+   `carril` o `verificacion` pueden haber salido en verde con el workflow ya
+   modificado, y con una sola persona (RN-06) esa aprobación es la misma que
+   ya exige el PR. Lo cierra F1-8 (por ejemplo, `carril` con
+   `pull_request_target` o un workflow que solo se lee de la base); ver
+   «Preguntas abiertas». En un equipo, obliga al propietario (no verificado
+   en la plataforma).
 7. **Cola de merge y rama al día.** Sin organización, `finish` mergea con
    `merge-pr` (F1-4) por la API cuando las puertas están en verde, con
-   `strict` activado: si `develop` avanzó, primero `actualizar-rama` (merge de
+   `strict` activado **solo en `develop`** (en `main`, no: decisión 2): si
+   `develop` avanzó, primero `actualizar-rama` (merge de
    la base, sin rebase, F1-4) y vuelta a verificar. En un repo de
    organización, el ruleset `taskcode-develop-cola` (fichero `develop-cola-org.json`) sustituye a `taskcode-develop` y añade `merge_queue` con
    `merge_method: MERGE` y `finish` pone el PR en la cola (D3a). La
@@ -321,8 +450,8 @@ ha aplicado ningún ruleset):
 |---|---|---|---|
 | errata | `develop` | `verificacion`, `revision-ia` (perfil rápido, D5a), `carril` | 1 persona en el PR |
 | fix | `develop` | `verificacion`, `revision-ia`, `carril` | 1 persona en el PR |
-| funcionalidad | `develop` | `verificacion`, `revision-ia`, `carril` | la persona aprueba el plan y el PR |
-| riesgo alto | `develop` | `verificacion`, `revision-ia`, `carril` | 2 aprobaciones de persona: con una sola persona, la del plan y la del PR (D4a) |
+| funcionalidad | `develop` | `verificacion`, `revision-ia`, `carril` | 2 aprobaciones de persona: el plan (commit de `approve` con firma verificada) y el PR |
+| riesgo alto | `develop` | `verificacion`, `revision-ia`, `carril` | 2 aprobaciones de persona: con una sola persona, el plan (commit de `approve` con firma verificada) y el PR (D4a) |
 | release | `main` | `verificacion` (perfil completo), `carril` | 1 persona en el PR |
 | hotfix | `main` | `verificacion`, `revision-ia` (lo exige `carril`), `carril` | 1 persona en el PR |
 <!-- tabla-carriles:fin -->
@@ -331,7 +460,14 @@ En ningún carril cuenta como aprobación una identidad de IA (agente
 implementador o revisor IA): el PR lo abre el agente, que no puede aprobar su
 propio PR; `carril` solo cuenta aprobaciones de personas; y ningún ruleset
 tiene `bypass_actors`. La sección D de la PoC compara esta tabla, leída de
-este fichero, con los checks de los JSON.
+este fichero, con los checks de los JSON y con el número de aprobaciones:
+el primer número de «Quién aprueba» tiene que ser las aprobaciones del PR
+del ruleset de la base (`required_approving_review_count`) más la del plan
+que exige `carril` (funcionalidad y riesgo alto), con al menos 2 en riesgo
+alto (D4a), y la celda menciona el plan solo donde se exige.
+
+El carril de la tabla es el **carril efectivo** de la decisión 4 (el mayor
+entre el declarado y el mínimo por ficheros), no el que el agente declara.
 
 ## Requisitos y evidencia
 
@@ -339,11 +475,12 @@ este fichero, con los checks de los JSON.
 |---|---|---|
 | P3 / RR-08 las garantías las impone la plataforma | rulesets sin bypass; checks atados a su integración; push directo a `develop`/`main` imposible | PoC C (seis JSON válidos contra el esquema, `bypass_actors: []`); comportamiento real: no verificado (F2-6, F9-4) |
 | RR-04 / RF-50 revisión independiente y aprobación de una persona antes del merge | `revision-ia` obligatorio en `develop`; 1 aprobación, último push aprobado, aprobaciones viejas descartadas; `carril` solo cuenta personas | PoC C (`pull_request` y `required_status_checks` válidos), D (ninguna fila aprueba con IA); no verificado en GitHub |
-| P7 / RR-07 carriles proporcionales, pocas puertas humanas | suelo común por rama + check `carril`; tabla de carriles | PoC D (tabla = rulesets + `carril`, 6 filas, 3/3 desvíos detectados) |
+| P7 / RR-07 carriles proporcionales, pocas puertas humanas | suelo común por rama + check `carril`; tabla de carriles; `strict` solo en `develop` | PoC D (checks y número de aprobaciones = rulesets + `carril`, 6 filas, 5/5 desvíos detectados); PoC E (`main` deja de ser ancestro de `develop` tras una release) |
+| P3 / RR-04 el agente no puede rebajar su propia puerta | carril efectivo = mayor entre el declarado y el mínimo por ficheros (solo sube); aprobación del plan solo con commit de `approve` con firma verificada de una persona | PoC F (esquema: `commit.verification` con `verified`; simulación: 4/4 casos de plan y 6/6 de carril); comportamiento real: no verificado (F2-6); regla de derivación: F1-10 |
 | RF-51 `## Resultado` exigido por un check | lo comprueba `carril` en los carriles con tarea | decisión 4; no implementado ni verificado (F6) |
 | P11 / RR-15 / RF-64 mismas garantías; `doctor` dice lo que falta | modelo neutral; `leer-puertas`; D2a | PoC A (403 con aviso de plan en el privado, `[]` en el público); GitLab: F1-13 |
 | RN-06 una persona con agentes igual que un equipo | D4a: plan + PR cuentan como las dos aprobaciones de riesgo alto | decisión 4; riesgo de `require_last_push_approval` en «Preguntas abiertas» |
-| H5 la aprobación humana no se puede comprobar | aprobación del PR por una identidad de persona en GitHub; `carril` comprueba la del plan (F4-2) | razonado; identidades: F1-6 |
+| H5 la aprobación humana no se puede comprobar | aprobación del PR por una identidad de persona en GitHub; la del plan, por la firma verificada del commit de `approve` (F4-2) | PoC F (campo `verification` en el esquema; commit real de `develop` sin firma: el flujo 0.16 no firma); firma y claves: F1-6; no verificado en GitHub |
 | H6 el agente se salta revisión y verificación | sin `bypass_actors`; checks obligatorios en el servidor; `finish` no puede mergear con puertas pendientes | PoC C; no verificado en GitHub |
 | F1-1 reglas de la rama de estado | `taskcode/tareas`: sin borrado ni force, push directo | PoC C (`taskcode-tareas.json`) |
 | F1-3 merge commit, ramas conservadas, tags | `allowed_merge_methods: [merge]`; `deletion` en ramas de tarea; tags sin borrar ni mover | PoC C (enum `merge`/`squash`/`rebase` del esquema; negativo `fast-forward` rechazado) |
@@ -356,11 +493,20 @@ este fichero, con los checks de los JSON.
   reglas ocurre en cualquier push directo a `develop`).
 - **F1-6** decide la App del revisor (su `integration_id` sustituye al
   relleno `0`), con qué identidad se abren y actualizan los PR y que ninguna
-  IA tenga permiso de administrar el repo ni de aprobar.
+  IA tenga permiso de administrar el repo ni de aprobar; además, la
+  identidad de la persona **con su clave de firma** registrada en la
+  plataforma, la lista de identidades de persona que `carril` acepta como
+  firmantes y que la clave no esté al alcance del agente.
+- **F4-2** (`approve`) crea el commit de aprobación en `taskcode/tareas`
+  **firmado** con la clave de la persona; sin firma verificada no hay plan
+  aprobado.
+- **F1-8** cubre la seguridad del check `carril` (que su workflow no lo
+  pueda cambiar el propio PR) y del acceso del agente a la clave de firma.
 - **F1-7** fija los perfiles de `verificacion` por base (rápida en `develop`,
   completa en `main`) y la caché por SHA que abarata `strict` sin cola.
-- **F1-10** fija qué es una errata (límites de un PR sin tarea), la política
-  de cada carril en configuración y quién lanza una release.
+- **F1-10** fija la regla de derivación del carril mínimo por ficheros (qué
+  es una errata), la política de cada carril en configuración y quién lanza
+  una release.
 - **F1-13** traduce el modelo neutral a GitLab y verifica la cola y los
   planes de pago.
 - **F2-6** implementa `aplicar-puertas` y `leer-puertas` y prueba en E2E,
@@ -418,5 +564,17 @@ riesgos sin prueba:
   puede crear un tag; quién publica una release es de F1-6 y F1-10.
 - **Riesgo alto en un equipo**: D4a cubre una sola persona; con varias, si
   hacen falta dos personas distintas en el PR lo decide F1-10.
+- **Regla de derivación del carril mínimo** (F1-10): qué rutas suben a qué
+  carril (la de la PoC F es un ejemplo), cómo se trata un PR enorme de solo
+  documentación y si el tamaño del diff cuenta.
+- **La clave de firma de la persona al alcance del agente**: el agente corre
+  en la misma máquina y sesión que la persona; si la clave está en un
+  `ssh-agent` o en un llavero desbloqueado, el agente puede firmar como ella
+  y la regla 1 no la distingue. Clave con frase de paso o con toque físico
+  (FIDO) lo contendrían; lo deciden F1-6 y F1-8. No verificado.
+- **Qué cuenta devuelve la API como firmante**: se usa la cuenta `committer`
+  del commit verificado; si GitHub la asocia por el correo del commit y una
+  identidad de agente puede usar el correo de la persona, falta comprobar que
+  `verified` exige la clave de esa cuenta. No verificado.
 - **Pilotos privados**: ToolsaWeb y OpenDockTrading quedarían sin puertas
   impuestas en una cuenta gratuita (D2a); su plan real no se ha comprobado.
````

## Excluido del diff (13 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff d05805a37a2b99f58dbfe9c0180713914bbb188d..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-006/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-006/planificacion/plan-final.md                 |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-006/poc/poc-f1-5.log                            |  64 ++++++++++---
 tareas/{02-en-curso => 03-en-revision}/TASK-006/poc/poc-f1-5.sh                             | 126 +++++++++++++++++++++++--
 tareas/{02-en-curso => 03-en-revision}/TASK-006/poc/rulesets/develop-cola-org.json          |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-006/poc/rulesets/develop.json                   |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-006/poc/rulesets/main.json                      |   2 +-
 tareas/{02-en-curso => 03-en-revision}/TASK-006/poc/rulesets/ramas-de-tarea.json            |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-006/poc/rulesets/tags-version.json              |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-006/poc/rulesets/taskcode-tareas.json           |   0
 tareas/03-en-revision/TASK-006/revision/informe-revision-1.md                               |  21 +++++
 tareas/03-en-revision/TASK-006/revision/peticion-revision-1.md                              | 514 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-006/tarea.md                                    |  11 ++-
 13 files changed, 709 insertions(+), 29 deletions(-)
````

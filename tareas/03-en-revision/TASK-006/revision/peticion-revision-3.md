# Peticion de revision — TASK-006 (ronda 3)

- Tarea: TASK-006 — F1-5 ADR puertas en GitHub
- Rama revisada: feature/task-006-f1-5-adr-puertas-en-github
- Rama base: develop
- Commit revisado (HEAD): 38166f29fef407ae4ef1c93b19b8d98bea0499a4
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-006 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 3e277420be9417232556523146d737d5f2468aa3 (el commit revisado en la ronda anterior)
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
informe de esta ronda (informe-revision-3.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-006/tarea.md
- tareas/03-en-revision/TASK-006/planificacion/
- docs/adr/F1-5-puertas-en-github.md

## Hallazgos de la ronda 2 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-5-puertas-en-github.md:387 | informe-revision-2.md |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-5-puertas-en-github.md:355 | informe-revision-2.md |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-5-puertas-en-github.md:387 | informe-revision-2.md |
| IMP-4 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-006/poc/poc-f1-5.sh:346 | informe-revision-2.md |
| IMP-5 | IMPORTANTE | corregido | docs/adr/F1-5-puertas-en-github.md:365 | informe-revision-2.md |

## Commits a revisar (git log 3e277420be9417232556523146d737d5f2468aa3..HEAD)

````
38166f2 chore(TASK-006): autorrevision y ronda extra 3
2cd1f20 docs(adr): F1-5 corrige los hallazgos de la revision 2 (plan atado al blob, errata solo docs, errata con tarea)
eb4ea53 chore(TASK-006): peticion de revision ronda 2
````

## Diff desde la ronda anterior (git diff 3e277420be9417232556523146d737d5f2468aa3..HEAD)

````diff
diff --git a/docs/adr/F1-5-puertas-en-github.md b/docs/adr/F1-5-puertas-en-github.md
index 12bdbcf..636e8b5 100644
--- a/docs/adr/F1-5-puertas-en-github.md
+++ b/docs/adr/F1-5-puertas-en-github.md
@@ -216,17 +216,23 @@ F1-10).
 commit de develop en TaskCodeTwo: sha=e8ee970802d9 verified=false reason=unsigned committer=charliebk commit.committer=carlos gallardo
 GET /repos/{o}/{r}/commits/{ref}: commit.verification = ['payload', 'reason', 'signature', 'verified', 'verified_at'] required [...]
   campos de primer nivel con la cuenta de GitHub: ['author', 'committer']
-  plan: approve firmado y verificado, committer persona    -> cuenta
-  plan: approve sin firma (lo escribe el agente)           -> no cuenta
-  plan: approve verificado pero de la App del agente       -> no cuenta
-  plan: approve verificado por GitHub (web-flow, por API)  -> no cuenta
-  carril: declarado=errata       ficheros=docs/guia.md                  -> errata
+  blob de plan-final.md: v1=44121046fc34 v2=ec2837e49822
+  plan: firmado, persona, blob = plan actual             -> cuenta
+  plan: sin firma (lo escribe el agente)                 -> no cuenta
+  plan: verificado pero de la App del agente             -> no cuenta
+  plan: verificado por GitHub (web-flow, por API)        -> no cuenta
+  plan: firmado, pero el agente cambio el plan despues   -> no cuenta
+  plan: firmado, sin blob registrado                     -> no cuenta
+  plan: firmado de nuevo sobre el plan cambiado          -> cuenta
+  carril: declarado=errata       ficheros=docs/a.md                     -> errata
+  carril: declarado=errata       ficheros=src/x.md                      -> fix
+  carril: declarado=errata       ficheros=CLAUDE.md                     -> fix
   carril: declarado=errata       ficheros=src/main.rs                   -> fix
   carril: declarado=fix          ficheros=.github/workflows/carril.yml  -> riesgo alto
   carril: declarado=riesgo alto  ficheros=docs/guia.md                  -> riesgo alto
-  carril: declarado=None         ficheros=README.md                     -> errata
-  carril: declarado=None         ficheros=src/main.rs                   -> FALLA (necesita tarea de carril >= fix)
-RESULTADO F esquema verification=si ; plan 4/4 casos ; carril efectivo 6/6 casos (el agente solo puede subirlo)
+  carril: declarado=None         ficheros=docs/a.md                     -> FALLA (PR sin tarea)
+  carril: declarado=None         ficheros=src/main.rs                   -> FALLA (PR sin tarea)
+RESULTADO F esquema verification=si ; plan 7/7 casos ; carril efectivo 8/8 casos (el agente solo puede subirlo)
 ```
 
 Lectura:
@@ -236,6 +242,11 @@ Lectura:
   sola no prueba nada; la regla exige `verified: true` **y** cuenta de
   persona. El flujo de la 0.16 no firma, así que F4-2 tiene que añadir la
   firma.
+- La aprobación queda **atada al contenido**: el commit firmado registra el
+  blob de `plan-final.md` aprobado, y un plan cambiado después (blob v2) deja
+  de contar hasta un nuevo `approve` firmado sobre él.
+- Errata solo si **todos** los ficheros están bajo `docs/`: un `.md` fuera de
+  `docs/` (`src/x.md`, `CLAUDE.md`) sube a fix. Un PR sin tarea falla siempre.
 - La simulación comprueba la lógica de las reglas, no GitHub: que la API
   marque `verified` como se espera en un commit firmado por la persona es
   **no verificado** (no hay ningún commit firmado que leer).
@@ -268,8 +279,16 @@ Lectura:
    consigna «la opción recomendada», la firma verificada del `approve` y el
    carril efectivo por ficheros (decisión 4; PoC F).
 7. **La tabla de carriles dice el número de aprobaciones** y la PoC D lo
-   comprueba (revisión 1, MEN-2); la errata sale de `fix/errata-<slug>`
-   (MEN-4).
+   comprueba (revisión 1, MEN-2).
+8. **La errata también tiene tarea** (revisión 2, IMP-5; sustituye a la rama
+   `fix/errata-<slug>` de la corrección de MEN-4). P7 dice «errata → PR
+   directo»; aquí «directo» queda como «sin plan aprobado», no «sin tarea»:
+   la rama es `fix/task-NNN-<slug>` como el resto (D1a de F1-3: unicidad por
+   ID; RR-12: el estado de todas las tareas a la vista), y un PR sin tarea
+   falla cerrado siempre. Si eso es demasiado peso para una errata lo puede
+   revisar F1-10 sin tocar las puertas.
+9. **Aprobación del plan atada al contenido y errata solo en `docs/`**
+   (revisión 2, IMP-3 e IMP-4): ver decisión 4 y PoC F.
 
 ### No verificado
 
@@ -296,7 +315,10 @@ Lectura:
   firmado por la persona salga `verified: true` con su cuenta, y que no haya
   forma de que otra identidad lo consiga (PoC F solo simula la regla).
 - **La regla de derivación del carril mínimo**: la de la PoC F es un ejemplo
-  (F1-10).
+  por encima de errata (F1-10).
+- **Cómo registra y lee `carril` el blob del plan aprobado**: el trailer
+  `Plan-Blob` y la lectura del blob actual por la API son una propuesta; se
+  implementan en F4-2 y F6.
 
 ## Cómo se ve en GitHub
 
@@ -362,10 +384,10 @@ ha aplicado ningún ruleset):
      `main` porque una release no la necesita; en un hotfix la exige `carril`.
    - **`feature/**`, `fix/**`, `hotfix/**`**: sin borrado ni force-push (las
      ramas se conservan, F1-3; la capa no fuerza nunca, F1-4). **La errata
-     sale de una rama `fix/errata-<slug>`** (prefijo de F1-3, sin ID de
-     tarea), así que queda cubierta por `fix/**`. Una rama con otro prefijo
-     no la crea el CLI y no está protegida; `carril` trata su PR como
-     cualquier otro (carril efectivo de la decisión 4).
+     también tiene tarea** y sale de `fix/task-NNN-<slug>` como cualquier fix
+     (D1a de F1-3), cubierta por `fix/**`. Una rama con otro prefijo no la
+     crea el CLI y no está protegida; su PR, sin tarea, lo rechaza `carril`
+     (decisión 4).
    - **`taskcode/tareas`**: sin borrado ni force-push; sin PR ni checks; push
      directo del CLI (F1-1, F1-2).
    - **Tags `v*`**: ni borrar ni mover (`deletion` + `update`).
@@ -390,16 +412,26 @@ ha aplicado ningún ruleset):
         `GET /repos/{o}/{r}/commits/{sha}`) y su firmante (la cuenta
         `committer` que devuelve la API) es una identidad de persona del
         proyecto; una identidad de agente o de IA, un commit sin firma o uno
-        firmado por la propia plataforma (`web-flow`) no valen. Es también
-        la base de la segunda aprobación de riesgo alto (D4a).
+        firmado por la propia plataforma (`web-flow`) no valen. Además, la
+        aprobación **se ata al contenido**: el commit firmado de `approve`
+        registra el sha del blob de `plan-final.md` aprobado (por ejemplo,
+        un trailer `Plan-Blob: <sha>`), y `carril` exige que el blob actual
+        de `plan-final.md` en `taskcode/tareas` sea ese mismo (equivale a que
+        el último commit que toca `plan-final.md` sea el firmado de
+        `approve`). Si el agente cambia el plan después, la aprobación deja
+        de contar hasta un nuevo `approve` firmado. Es también la base de la
+        segunda aprobación de riesgo alto (D4a).
      2. **Carril efectivo = el mayor** entre el declarado en la tarea y un
         **carril mínimo derivado de los ficheros del PR** (que `carril` lee
-        de la API, no de la rama): por ejemplo, un cambio en `.github/`,
-        `.taskcode/`, CODEOWNERS o fuera de `docs/` no puede ser errata. El
+        de la API, no de la rama): **solo es errata si todos los ficheros
+        del PR están bajo `docs/`** (un `.md` fuera de `docs/`, como
+        `src/x.md` o `CLAUDE.md`, no lo es); un cambio en `.github/`,
+        `.taskcode/` o CODEOWNERS sube más (en la PoC, a riesgo alto). El
         agente solo puede **subir** el carril, nunca bajarlo. Orden:
-        errata < fix < funcionalidad < riesgo alto. Un PR sin tarea solo es
-        errata si su mínimo es errata; si no, falla cerrado («necesita
-        tarea»). La regla de derivación concreta la fija F1-10.
+        errata < fix < funcionalidad < riesgo alto. **Todo PR necesita
+        tarea**, también la errata: un PR sin tarea falla cerrado siempre.
+        El resto de la regla de derivación (qué sube a qué por encima de
+        errata) lo fija F1-10.
 
      La PoC F simula las dos reglas y comprueba en el esquema que la API da
      `verification.verified`; el comportamiento real en GitHub, **no
@@ -476,7 +508,7 @@ entre el declarado y el mínimo por ficheros), no el que el agente declara.
 | P3 / RR-08 las garantías las impone la plataforma | rulesets sin bypass; checks atados a su integración; push directo a `develop`/`main` imposible | PoC C (seis JSON válidos contra el esquema, `bypass_actors: []`); comportamiento real: no verificado (F2-6, F9-4) |
 | RR-04 / RF-50 revisión independiente y aprobación de una persona antes del merge | `revision-ia` obligatorio en `develop`; 1 aprobación, último push aprobado, aprobaciones viejas descartadas; `carril` solo cuenta personas | PoC C (`pull_request` y `required_status_checks` válidos), D (ninguna fila aprueba con IA); no verificado en GitHub |
 | P7 / RR-07 carriles proporcionales, pocas puertas humanas | suelo común por rama + check `carril`; tabla de carriles; `strict` solo en `develop` | PoC D (checks y número de aprobaciones = rulesets + `carril`, 6 filas, 5/5 desvíos detectados); PoC E (`main` deja de ser ancestro de `develop` tras una release) |
-| P3 / RR-04 el agente no puede rebajar su propia puerta | carril efectivo = mayor entre el declarado y el mínimo por ficheros (solo sube); aprobación del plan solo con commit de `approve` con firma verificada de una persona | PoC F (esquema: `commit.verification` con `verified`; simulación: 4/4 casos de plan y 6/6 de carril); comportamiento real: no verificado (F2-6); regla de derivación: F1-10 |
+| P3 / RR-04 el agente no puede rebajar su propia puerta | carril efectivo = mayor entre el declarado y el mínimo por ficheros (solo sube); aprobación del plan solo con commit de `approve` con firma verificada de una persona | PoC F (esquema: `commit.verification` con `verified`; simulación: 7/7 casos de plan, con el blob del plan, y 8/8 de carril); comportamiento real: no verificado (F2-6); regla de derivación: F1-10 |
 | RF-51 `## Resultado` exigido por un check | lo comprueba `carril` en los carriles con tarea | decisión 4; no implementado ni verificado (F6) |
 | P11 / RR-15 / RF-64 mismas garantías; `doctor` dice lo que falta | modelo neutral; `leer-puertas`; D2a | PoC A (403 con aviso de plan en el privado, `[]` en el público); GitLab: F1-13 |
 | RN-06 una persona con agentes igual que un equipo | D4a: plan + PR cuentan como las dos aprobaciones de riesgo alto | decisión 4; riesgo de `require_last_push_approval` en «Preguntas abiertas» |
@@ -498,14 +530,16 @@ entre el declarado y el mínimo por ficheros), no el que el agente declara.
   plataforma, la lista de identidades de persona que `carril` acepta como
   firmantes y que la clave no esté al alcance del agente.
 - **F4-2** (`approve`) crea el commit de aprobación en `taskcode/tareas`
-  **firmado** con la clave de la persona; sin firma verificada no hay plan
+  **firmado** con la clave de la persona y con el blob de `plan-final.md`
+  aprobado; sin firma verificada, o con el plan cambiado después, no hay plan
   aprobado.
 - **F1-8** cubre la seguridad del check `carril` (que su workflow no lo
   pueda cambiar el propio PR) y del acceso del agente a la clave de firma.
 - **F1-7** fija los perfiles de `verificacion` por base (rápida en `develop`,
   completa en `main`) y la caché por SHA que abarata `strict` sin cola.
-- **F1-10** fija la regla de derivación del carril mínimo por ficheros (qué
-  es una errata), la política de cada carril en configuración y quién lanza
+- **F1-10** fija la regla de derivación del carril mínimo por encima de
+  errata (errata = solo `docs/`, decidido aquí), si una errata necesita
+  tarea con menos ceremonia, la política de cada carril en configuración y quién lanza
   una release.
 - **F1-13** traduce el modelo neutral a GitLab y verifica la cola y los
   planes de pago.
@@ -565,7 +599,7 @@ riesgos sin prueba:
 - **Riesgo alto en un equipo**: D4a cubre una sola persona; con varias, si
   hacen falta dos personas distintas en el PR lo decide F1-10.
 - **Regla de derivación del carril mínimo** (F1-10): qué rutas suben a qué
-  carril (la de la PoC F es un ejemplo), cómo se trata un PR enorme de solo
+  carril por encima de errata (la de la PoC F es un ejemplo), cómo se trata un PR enorme de solo
   documentación y si el tamaño del diff cuenta.
 - **La clave de firma de la persona al alcance del agente**: el agente corre
   en la misma máquina y sesión que la persona; si la clave está en un
````

## Excluido del diff (6 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 3e277420be9417232556523146d737d5f2468aa3..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/03-en-revision/TASK-006/poc/poc-f1-5.log                |  24 +++---
 tareas/03-en-revision/TASK-006/poc/poc-f1-5.sh                 |  58 +++++++++------
 tareas/03-en-revision/TASK-006/revision/informe-revision-1.md  |  62 ++++++++++++++--
 tareas/03-en-revision/TASK-006/revision/informe-revision-2.md  |  21 ++++++
 tareas/03-en-revision/TASK-006/revision/peticion-revision-2.md | 520 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-006/tarea.md                        |   9 ++-
 6 files changed, 657 insertions(+), 37 deletions(-)
````

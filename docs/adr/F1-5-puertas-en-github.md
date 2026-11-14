# ADR F1-5 — Puertas en GitHub por carril: rulesets, checks obligatorios y CODEOWNERS

- Estado: **aceptada** (2026-11-14)
- Tarea: TASK-006 · Plan: [F1-5 en PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) (§4 P3, P7, P11 y tabla de la Fase 1)
- Requisitos: RR-04, RR-07, RR-08, RR-15, RF-50, RF-51, RF-64 y RN-06 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); H5 y H6 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
- Depende de: [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (rama de estado con push directo del CLI), [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md) (nombres de rama, merge commit, ramas conservadas, tags `vX.Y.Z`) y [ADR F1-4](F1-4-operaciones-git-y-contrato-de-salida.md) (puerto `Plataforma` y su lista cerrada)
- Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a, D5a y D6a del plan de TASK-006)

## Contexto

F1-1 dejó a este ADR las reglas de la rama de estado; F1-3, las de `develop`,
`main` y las ramas de tarea, el método de merge permitido y la cola; F1-4, los
endpoints del puerto `Plataforma` y el caso de un `push` rechazado por reglas.
El marco:

- **P3 / RR-08**: las garantías las impone la plataforma, no las instrucciones
  al agente. Código solo por PR contra `develop` con checks obligatorios
  (verificación, revisión IA) y la aprobación de una persona; el agente y el
  revisor IA tienen identidad propia (H5); sin identidad de persona no hay
  atajos (H6).
- **P7 / RR-07**: carriles proporcionales. Errata → PR directo; fix → tarea y
  PR; funcionalidad → plan aprobado y PR; riesgo alto → diseño con todos los
  roles y dos aprobaciones. Pocas puertas humanas, todas con valor.
- **RR-04 / RF-50**: revisión independiente y aprobación de una persona antes
  del merge; cerrar integrando solo con revisión aprobada y verificación en
  verde, «merge del PR por la cola, con rulesets».
- **RF-51**: `## Resultado` obligatorio, como «sección obligatoria de la
  descripción del PR (check que la exige)».
- **P11 / RR-15 / RF-64**: las mismas garantías en GitHub y GitLab; donde la
  plataforma o su plan no las den, se documenta y `doctor` lo dice.
- **RN-06**: un proyecto de una sola persona con agentes funciona igual que un
  equipo (los tres repos de hoy son de una persona).

Preguntas que responde: cómo se describen las puertas sin atarse a GitHub,
qué reglas lleva cada rama y cada tag, qué checks son obligatorios y de dónde
tienen que venir, cómo se expresa el carril, quién aprueba en cada carril y
qué pasa en una cuenta gratuita, sin cola de merge.

## Alternativas

| | Descripción | Veredicto |
|---|---|---|
| **Rulesets** | Reglas de repositorio (`/repos/{o}/{r}/rulesets`): varias por repo, por patrón de rama o de tag, con checks atados a una integración. | **Elegida**: cubren ramas y tags con el mismo modelo, admiten `integration_id` y `allowed_merge_methods`, y se leen con `GET /rules/branches/{rama}` (lo que `doctor` necesita). |
| Protección clásica de ramas | `/branches/{rama}/protection`, una por rama. | Descartada: no protege tags ni patrones con la misma API y no tiene método de merge por rama; además, en el repo privado gratuito responde el mismo 403 que los rulesets (PoC A), así que no salva D2. |
| **Carril en un check** | El ruleset pone un suelo común por rama; un check obligatorio `carril` (workflow) impone lo propio de cada carril. | **Elegida**: un ruleset filtra por rama, no por PR ni por tarea; el carril vive en la tarea (rama de estado, F1-1). |
| Carril declarado solo en la tarea | `carril` cree el carril y la marca de plan aprobado del frontmatter de `taskcode/tareas`. | Descartada (revisión 1): el agente puede escribir en esa rama y bajar su carril o marcar el plan como aprobado; se sustituye por la firma verificada del `approve` y el carril mínimo por ficheros (decisión 4). |
| Carril en el nombre de la rama | Un ruleset por prefijo (`errata/*`, `riesgo/*`…) con sus aprobaciones. | Descartada: el agente elige el nombre de la rama y con él la puerta; y los rulesets se aplican a la rama **destino** del PR, no a la de origen. |
| Carril con etiquetas del PR | Reglas según la etiqueta. | Descartada: los rulesets no filtran por etiqueta y el agente puede poner o quitar etiquetas. |
| **D1a** Evidencia de solo lectura | `GET` de rulesets y esquema OpenAPI; nada se escribe en GitHub. | **Elegida**. Descartadas: (b) crear un ruleset en el repo temporal, (c) hacerlo público, (d) aplicarlos en este repo (rompe el flujo 0.16, RR-14). |
| **D2a** Repos privados gratuitos | La garantía exige GitHub Pro, Team o repo público; si no, `doctor` declara «puertas no impuestas por la plataforma». | **Elegida** (PoC A: 403 con aviso de plan). |
| **D3a** Cola de merge | `taskctl merge` por la API; la cola (`merge_queue`) solo si el repo es de una organización. | **Elegida**. |
| **D4a** Riesgo alto con una persona | Cuentan como las dos aprobaciones la de la persona en el plan y la suya en el PR (RN-06). | **Elegida**. |
| **D5a** `revision-ia` en errata | También se ejecuta, con perfil rápido (P3 la pide en todo PR). | **Elegida**. |
| **D6a** Rama al día sin cola | `strict_required_status_checks_policy: true` en `develop`. | **Elegida**. En `main`, `false` (decisión 2; revisión 1). |
| Checks sin origen | Solo el nombre (`context`) del check. | Descartada: cualquier identidad con permiso de escribir checks (también la del agente) publicaría un `revision-ia` verde con ese nombre. |

## Pruebas de concepto

Guion `poc-f1-5.sh`, su salida completa `poc-f1-5.log` y los JSON generados
(`poc/rulesets/*.json`), en la carpeta `poc/` de TASK-006 (sin enlace: la
carpeta de la tarea cambia de estado al cerrarse). Ejecutado en Git Bash con
`gh` 2.95.0 y Python 3.12 (`python -I`) bajo
`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-006\poc\` y, tras la
revisión 1, `...\task-006-c1\` (borradas al terminar); el esquema se descarga en su propia carpeta vacía (`esquema/`).
**Solo lectura en GitHub**: toda llamada pasa por una función que usa
`gh api --method GET`, y el guion las registra.

```bash
bash poc-f1-5.sh "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-006-c1" \
  docs/adr/F1-5-puertas-en-github.md tareas/<estado>/TASK-006/poc/rulesets > poc-f1-5.log 2>&1
```

### A — Estado real en GitHub (solo `GET`)

```text
--- charliebk/taskcodetwo-poc-f1-1
visibilidad: PRIVATE
rulesets:                     HTTP 403 {"message":"Upgrade to GitHub Pro or make this repository public to enable this feature.", ...}
rules/branches/develop:       HTTP 403 {"message":"Upgrade to GitHub Pro or make this repository public to enable this feature.", ...}
branches/develop/protection:  HTTP 403 {"message":"Upgrade to GitHub Pro or make this repository public to enable this feature.", ...}
--- charliebk/TaskCodeTwo
visibilidad: PUBLIC
rulesets:                     HTTP 200 []
rules/branches/develop:       HTTP 200 []
branches/develop/protection:  HTTP 404 {"message":"Branch not protected", ...}
TaskCodeTwo codeowners/errors: HTTP 404 {"message":"Not Found", ...}
id de la App GitHub Actions: 15368
RESULTADO A+B llamadas gh api con GET=11 ; gh api con otro metodo=0 ; gh repo view=2   (incluye el GET de la sección F)
```

Lectura:

- En el repo **privado** de una cuenta personal gratuita, rulesets, reglas
  por rama **y también la protección clásica** responden 403 con el aviso de
  plan: la plataforma no impondría allí ninguna puerta (D2a). Es lo que les
  pasaría a los pilotos privados (ToolsaWeb, OpenDockTrading) en una cuenta
  gratuita; su plan real no se ha comprobado.
- En el repo **público** la API está disponible y vacía: este repo no tiene
  rulesets, `develop` no está protegida y no hay CODEOWNERS (el 404 de
  `codeowners/errors` se lee como «sin fichero»; no verificado contra la
  documentación). Es coherente con la 0.16, que integra con merge local
  (RR-14); este ADR no lo cambia.
- `integration_id` de GitHub Actions: **15368** (`GET /apps/github-actions`).

### B — Esquema

Descripción OpenAPI pública de GitHub, `github/rest-api-description` en el
commit `2a33d9cc5d1d` (2026-11-14T00:44:38Z), fichero
`descriptions/api.github.com/api.github.com.json` (14 274 027 bytes, OpenAPI
3.0.3, versión 1.1.4). Se usa el cuerpo de `POST /repos/{owner}/{repo}/rulesets`:

```text
rules[].type admitidos (25): creation update deletion required_linear_history merge_queue
  required_deployments required_signatures pull_request required_status_checks non_fast_forward ...
allowed_merge_methods: ['merge', 'squash', 'rebase']
status check: {'context': 'string', 'integration_id': 'integer'} required ['context']
additionalProperties en el cuerpo: (no declarado)
```

### C — Modelo neutral → JSON de ruleset por destino → validación

El guion describe las puertas en un modelo neutral (sin nombres de GitHub),
lo traduce a un ruleset por destino y valida cada JSON contra el esquema con
un validador **más estricto que el esquema**: una clave que el esquema no
declara es fallo; `oneOf` tiene que casar con una sola variante; tipos,
enums, claves obligatorias, mínimos y máximos. Siete controles negativos
comprueban que el validador rechaza lo que debe.

```text
develop          branch refs/heads/develop   reglas=deletion,non_fast_forward,pull_request,required_status_checks
                 checks=verificacion@15368,revision-ia@0,carril@15368 -> VALIDO
main             branch refs/heads/main      reglas=deletion,non_fast_forward,pull_request,required_status_checks
                 checks=verificacion@15368,carril@15368 -> VALIDO   (strict=false, decisión 2)
ramas-de-tarea   branch refs/heads/feature/**,refs/heads/fix/**,refs/heads/hotfix/**  reglas=deletion,non_fast_forward -> VALIDO
taskcode-tareas  branch refs/heads/taskcode/tareas  reglas=deletion,non_fast_forward -> VALIDO
tags-version     tag    refs/tags/v*         reglas=deletion,update -> VALIDO
develop-cola-org branch refs/heads/develop   reglas=...,required_status_checks,merge_queue -> VALIDO
-- controles negativos (el validador tiene que rechazarlos)
  rules[].type inventado (required_approvals)                  -> rechazado: neg.rules[4]: oneOf casa con 0 variantes
  allowed_merge_methods fast-forward                           -> rechazado: neg.rules[2]: oneOf casa con 0 variantes
  integration_id como texto                                    -> rechazado: neg.rules[3]: oneOf casa con 0 variantes
  clave desconocida en pull_request (required_reviewers_count) -> rechazado: neg.rules[2]: oneOf casa con 0 variantes
  clave desconocida en la raiz (lane)                          -> rechazado: neg: clave desconocida lane
  falta strict_required_status_checks_policy                   -> rechazado: neg.rules[3]: oneOf casa con 0 variantes
  enforcement inventado (required)                             -> rechazado: neg.enforcement: 'required' fuera del enum
-- JSON versionados en la carpeta de artefactos (antes de sobrescribirlos): validacion y comparacion
  develop            VALIDO ; igual al generado
  main               VALIDO ; igual al generado
  (ramas-de-tarea, taskcode-tareas, tags-version, develop-cola-org: igual)
RESULTADO C rulesets=6 errores=0 ; negativos rechazados=7/7 ; integration_id actions=15368 revisor=0 (relleno) ; versionados validos=6/6 iguales=6/6
```

Los `rulesets/*.json` versionados son **salida del guion**, no fuente: antes de
sobrescribirlos, el guion los valida con el mismo validador y los compara con
los generados (revisión 1). En la primera ejecución tras pasar `main` a
`strict=false`, `main.json` salió «DISTINTO del generado (se sobrescribe)»,
como se esperaba; el log versionado es el de la segunda, con 6/6 iguales.

`revision-ia@0`: la App del revisor aún no existe (la decide F1-6); el `0` es
un relleno que el guion toma de `APP_REVISOR_ID` (divergencia 2).

### D — La tabla de carriles frente a los JSON

El guion lee la tabla «Quién aprueba cada carril» de este ADR y, para cada
fila, compara sus checks con los del ruleset de la base del PR más lo que
impone el check `carril` (solo `revision-ia` en hotfix), y el **número de
aprobaciones** (primer número de «Quién aprueba») con las del PR del ruleset
más la del plan que exige `carril` (funcionalidad y riesgo alto; mínimo 2 en
riesgo alto, D4a). Exige además que la base tenga `carril` obligatorio, que
«Quién aprueba» diga «persona», mencione el plan solo donde se exige y no
nombre ninguna identidad de IA, y que la base no tenga `bypass_actors`.

```text
errata         base=develop  checks:si aprobaciones=1 (PR 1 + plan 0) aprueba:persona, sin IA, numero cuadra bypass_actors=[]
fix            base=develop  checks:si aprobaciones=1 (PR 1 + plan 0) aprueba:persona, sin IA, numero cuadra bypass_actors=[]
funcionalidad  base=develop  checks:si aprobaciones=2 (PR 1 + plan 1) aprueba:persona, sin IA, numero cuadra bypass_actors=[]
riesgo alto    base=develop  checks:si aprobaciones=2 (PR 1 + plan 1) aprueba:persona, sin IA, numero cuadra bypass_actors=[]
release        base=main     checks:si aprobaciones=1 (PR 1 + plan 0) aprueba:persona, sin IA, numero cuadra bypass_actors=[]
hotfix         base=main     checks:si aprobaciones=1 (PR 1 + plan 0) aprueba:persona, sin IA, numero cuadra bypass_actors=[]
-- controles negativos (tablas alteradas que la comprobacion tiene que detectar)
  release con revision-ia                  -> detectado (1 fallos)
  fix sin carril                           -> detectado (1 fallos)
  riesgo alto con 1 persona                -> detectado (1 fallos)
  funcionalidad sin aprobacion del plan    -> detectado (1 fallos)
  errata aprobada por el revisor IA        -> detectado (1 fallos)
RESULTADO D filas=6 fallos=0 ; negativos detectados=5/5
```

Lectura: la tabla y los rulesets dicen lo mismo, y la comparación detecta una
tabla que se desvíe. Es una comprobación **de coherencia del documento**, no
del comportamiento de GitHub.

### E — Por qué `main` va sin `strict` (git local)

Repo temporal, release por merge commit sin fast-forward como en F1-3:

```text
develop y main iguales: main ES ancestro de develop
tras la release 1 (merge commit en main): main NO es ancestro de develop
tras d2 en develop: main NO es ancestro de develop -> con strict, el PR develop->main de la release 2 saldria desactualizado
RESULTADO E main ancestro de develop tras release por merge commit: no
```

Lectura: el hecho de ancestría está comprobado; que GitHub, con `strict`,
marque ese PR como desactualizado es **no verificado** (conocimiento general;
ningún ruleset aplicado). Por eso `main` lleva `strict=false` (decisión 2).

### F — Carril no falsificable por el agente

Un `GET` a un commit real y el esquema dicen qué da la API para la regla 1
de la decisión 4; una simulación en Python aplica las dos reglas a casos
fijos. La regla de derivación de la simulación es un **ejemplo** (la fija
F1-10).

```text
commit de develop en TaskCodeTwo: sha=e8ee970802d9 verified=false reason=unsigned committer=charliebk commit.committer=carlos gallardo
GET /repos/{o}/{r}/commits/{ref}: commit.verification = ['payload', 'reason', 'signature', 'verified', 'verified_at'] required [...]
  campos de primer nivel con la cuenta de GitHub: ['author', 'committer']
  blob de plan-final.md: v1=44121046fc34 v2=ec2837e49822
  plan: firmado, persona, blob = plan actual             -> cuenta
  plan: sin firma (lo escribe el agente)                 -> no cuenta
  plan: verificado pero de la App del agente             -> no cuenta
  plan: verificado por GitHub (web-flow, por API)        -> no cuenta
  plan: firmado, pero el agente cambio el plan despues   -> no cuenta
  plan: firmado, sin blob registrado                     -> no cuenta
  plan: firmado de nuevo sobre el plan cambiado          -> cuenta
  carril: declarado=errata       ficheros=docs/a.md                     -> errata
  carril: declarado=errata       ficheros=src/x.md                      -> fix
  carril: declarado=errata       ficheros=CLAUDE.md                     -> fix
  carril: declarado=errata       ficheros=src/main.rs                   -> fix
  carril: declarado=fix          ficheros=.github/workflows/carril.yml  -> riesgo alto
  carril: declarado=riesgo alto  ficheros=docs/guia.md                  -> riesgo alto
  carril: declarado=None         ficheros=docs/a.md                     -> FALLA (PR sin tarea)
  carril: declarado=None         ficheros=src/main.rs                   -> FALLA (PR sin tarea)
RESULTADO F esquema verification=si ; plan 7/7 casos ; carril efectivo 8/8 casos (el agente solo puede subirlo)
```

Lectura:

- El commit real de `develop` **no está firmado** (`unsigned`) y aun así la
  API le asigna la cuenta `committer=charliebk` (por el correo): la cuenta
  sola no prueba nada; la regla exige `verified: true` **y** cuenta de
  persona. El flujo de la 0.16 no firma, así que F4-2 tiene que añadir la
  firma.
- La aprobación queda **atada al contenido**: el commit firmado registra el
  blob de `plan-final.md` aprobado, y un plan cambiado después (blob v2) deja
  de contar hasta un nuevo `approve` firmado sobre él.
- Errata solo si **todos** los ficheros están bajo `docs/`: un `.md` fuera de
  `docs/` (`src/x.md`, `CLAUDE.md`) sube a fix. Un PR sin tarea falla siempre.
- La simulación comprueba la lógica de las reglas, no GitHub: que la API
  marque `verified` como se espera en un commit firmado por la persona es
  **no verificado** (no hay ningún commit firmado que leer).

### Divergencias con el plan

1. **Claves que el esquema exige y el plan no decidía.** `pull_request` exige
   `required_review_thread_resolution` (se fija `false`: los hilos abiertos no
   bloquean; la puerta es la aprobación) y la regla `update` de los tags exige
   `update_allows_fetch_and_merge` (se fija `false`). Los tags se traducen a
   `deletion` + `update` («ni borrar ni mover»); `non_fast_forward` sobraría.
2. **`integration_id` de `revision-ia` sin valor real.** La App del revisor
   no existe todavía (F1-6). Los JSON llevan `0` como relleno, y
   `aplicar-puertas` tiene que negarse a aplicar un `integration_id` de
   relleno.
3. **El esquema no declara `additionalProperties: false`**: por sí solo no
   rechaza una clave inventada. La validación estricta es una regla del
   guion; si la API real rechaza o ignora claves desconocidas queda **no
   verificado** (no se ha hecho ningún `POST`).
4. **La protección clásica también da 403** en el repo privado gratuito (el
   plan solo lo esperaba de los rulesets): no hay alternativa dentro de la
   cuenta gratuita, lo que refuerza D2a.
5. **`main` sin `strict`** (revisión 1, IMP-2). El plan decía «`main`:
   igual» que `develop`, y D6a solo hablaba de `develop`; con releases por
   merge commit, `strict` en `main` bloquearía cada release desde la segunda
   (PoC E). Se corrige en el modelo, en `main.json` y en la decisión 2.
6. **El check `carril` no se fía de la rama de estado** (revisión 1, IMP-1).
   El plan decía que `carril` «lee el carril de la tarea en
   `taskcode/tareas`», que el agente puede escribir. Carlos decide, con la
   consigna «la opción recomendada», la firma verificada del `approve` y el
   carril efectivo por ficheros (decisión 4; PoC F).
7. **La tabla de carriles dice el número de aprobaciones** y la PoC D lo
   comprueba (revisión 1, MEN-2).
8. **La errata también tiene tarea** (revisión 2, IMP-5; sustituye a la rama
   `fix/errata-<slug>` de la corrección de MEN-4). P7 dice «errata → PR
   directo»; aquí «directo» queda como «sin plan aprobado», no «sin tarea»:
   la rama es `fix/task-NNN-<slug>` como el resto (D1a de F1-3: unicidad por
   ID; RR-12: el estado de todas las tareas a la vista), y un PR sin tarea
   falla cerrado siempre. Si eso es demasiado peso para una errata lo puede
   revisar F1-10 sin tocar las puertas.
9. **Aprobación del plan atada al contenido y errata solo en `docs/`**
   (revisión 2, IMP-3 e IMP-4): ver decisión 4 y PoC F.

### No verificado

- **El comportamiento real de las puertas**: que GitHub bloquee el merge sin
  aprobación o con un check pendiente, que descarte aprobaciones viejas, que
  exija la aprobación del último push, que rechace un check con el nombre
  correcto y otro `integration_id`, que CODEOWNERS exija al propietario y que
  solo ofrezca merge commit. **No se ha aplicado ningún ruleset** (D1a). Pasa
  como criterio de aceptación a **F2-6** (E2E contra un repo de pruebas) y
  **F9-4**.
- **Que la cola de merge solo exista en repos de organización** (D3a): no
  comprobado por la PoC; el esquema no lo dice. Lo verifica F1-13.
- **Que GitHub Pro o Team dé rulesets en repos privados** (D2a): solo se
  observa el texto del 403 («Upgrade to GitHub Pro or make this repository
  public»).
- **Que la API acepte los seis JSON tal cual** (claves, combinación de reglas
  por target): validados contra el esquema, no enviados.
- **Lo que pinta la interfaz de GitHub** (sección siguiente): conocimiento
  general, sin ruleset aplicado que mirar.
- **Que GitHub marque desactualizado el PR `develop`→`main` con `strict`**
  tras una release, y que sin `strict` lo deje mergear (PoC E solo prueba la
  ancestría).
- **La regla 1 del carril en GitHub real**: que un commit de `approve`
  firmado por la persona salga `verified: true` con su cuenta, y que no haya
  forma de que otra identidad lo consiga (PoC F solo simula la regla).
- **La regla de derivación del carril mínimo**: la de la PoC F es un ejemplo
  por encima de errata (F1-10).
- **Cómo registra y lee `carril` el blob del plan aprobado**: el trailer
  `Plan-Blob` y la lectura del blob actual por la API son una propuesta; se
  implementan en F4-2 y F6.

## Cómo se ve en GitHub

Hoy (observado, PoC A): este repo es público, sin rulesets, con `develop` sin
proteger y sin CODEOWNERS; el único check que corre es `comprobar-repo` de
`ci.yml`, sin ser obligatorio. Sigue así hasta el relevo (RR-14).

Con este ADR aplicado (conocimiento general de GitHub, **no verificado**: no se
ha aplicado ningún ruleset):

| Dónde | Qué se ve |
|---|---|
| Settings → Rules → Rulesets | cinco rulesets activos: `taskcode-develop`, `taskcode-main`, `taskcode-ramas-de-tarea`, `taskcode-taskcode-tareas` y `taskcode-tags-version` (más `taskcode-develop-cola` en vez de `taskcode-develop` si el repo es de una organización y usa la cola), sin actores con bypass |
| Caja de merge de un PR a `develop` | `verificacion`, `revision-ia` y `carril` marcados como obligatorios; «Review required» hasta la aprobación de una persona; «Code owner review required» si el PR toca `.github/` o `.taskcode/`; «This branch is out-of-date» con el botón de actualizar si `develop` avanzó (D6a); un solo método: merge commit |
| Check `carril` | un check más en la lista, con un resumen de una línea: carril efectivo (declarado y mínimo por ficheros) y qué le falta («falta la aprobación del plan», «falta `## Resultado`») o «no aplica» |
| PR `develop`→`main` de una release | sin aviso de rama desactualizada aunque `main` lleve merges de releases anteriores (sin `strict`; no verificado) |
| Commit de `approve` en `taskcode/tareas` | con la etiqueta «Verified» si lo firmó la persona; sin ella, `carril` no cuenta la aprobación del plan |
| Push directo a `develop` o `main` | rechazado por el ruleset (también el de la persona: no hay bypass) |
| `git push --force` o borrar `feature/task-…`, `taskcode/tareas` o un tag `v*` | rechazado |
| Push del CLI a `taskcode/tareas` | aceptado (sin PR ni checks), siempre fast-forward |
| Repo privado en cuenta gratuita | nada de lo anterior: `doctor` dice «puertas no impuestas por la plataforma» (D2a) |

## Decisión

1. **Modelo de puertas neutral y su traducción.** El núcleo describe cada
   destino (rama o tag) con reglas abstractas, sin nombres de GitHub:

   ```text
   develop          rama  sin_borrado sin_forzar solo_pr(aprobaciones=1, descartar_viejas, ultimo_push,
                          propietarios, metodos=[merge]) al_dia checks=[verificacion<-actions,
                          revision-ia<-app-revisor, carril<-actions]
   main             rama  igual que develop pero sin al_dia, checks=[verificacion<-actions, carril<-actions]
   ramas de tarea   rama  feature/**, fix/** (también la errata), hotfix/**: sin_borrado sin_forzar
   taskcode/tareas  rama  sin_borrado sin_forzar (push directo del CLI)
   v*               tag   sin_borrado sin_mover
   ```

   El adaptador de GitHub lo traduce a un ruleset por destino (los seis JSON
   de la PoC); el de GitLab, a ramas protegidas y reglas de aprobación
   (F1-13). Al puerto `Plataforma` de F1-4 se añaden dos operaciones a su
   lista cerrada:

   | Operación | Comando dueño | Idempotencia y resultados |
   |---|---|---|
   | `aplicar-puertas` | `init` (F3-1), con la identidad de la persona; exige administrar el repo | lee los rulesets `taskcode-*`; igual al modelo → `sin-cambios`; distinto → los actualiza; sin rulesets en el plan (403) → `rechazada` (3) con `motivo=plan`; se niega con un `integration_id` de relleno (no verificado: ninguna escritura en GitHub) |
   | `leer-puertas` | `doctor` (F3-8), `review` | lectura (`GET /rulesets` y `/rules/branches/{rama}`); compara con el modelo e informa lo que falta o sobra; 403 de plan → «puertas no impuestas por la plataforma» (PoC A) |

2. **Reglas por rama** (cierra lo que dejaron F1-1 y F1-3):
   - **`develop`**: sin borrado ni force-push; solo por PR; 1 aprobación;
     aprobaciones viejas descartadas con cada push y aprobación del último
     push; revisión del propietario de código; **solo merge commit** (D4a de
     F1-3); checks obligatorios `verificacion`, `revision-ia` y `carril`;
     rama al día (`strict`, D6a).
   - **`main`**: igual, con `verificacion` (perfil completo; el workflow elige
     el perfil por la base del PR) y `carril`, pero **sin rama al día**
     (`strict_required_status_checks_policy: false`). Motivo: con releases
     por merge commit (D3a de F1-3), tras la primera release `main` deja de
     ser ancestro de `develop` (PoC E); con `strict`, el PR `develop`→`main`
     de cada release siguiente saldría desactualizado, y actualizarlo es
     empujar a `develop`, que solo admite PR: cada release exigiría antes un
     PR `main`→`develop` con todas sus puertas. Cómo trata GitHub ese PR sin
     `strict`: no verificado. `revision-ia` no es obligatorio en el ruleset de
     `main` porque una release no la necesita; en un hotfix la exige `carril`.
   - **`feature/**`, `fix/**`, `hotfix/**`**: sin borrado ni force-push (las
     ramas se conservan, F1-3; la capa no fuerza nunca, F1-4). **La errata
     también tiene tarea** y sale de `fix/task-NNN-<slug>` como cualquier fix
     (D1a de F1-3), cubierta por `fix/**`. Una rama con otro prefijo no la
     crea el CLI y no está protegida; su PR, sin tarea, lo rechaza `carril`
     (decisión 4).
   - **`taskcode/tareas`**: sin borrado ni force-push; sin PR ni checks; push
     directo del CLI (F1-1, F1-2).
   - **Tags `v*`**: ni borrar ni mover (`deletion` + `update`).
   - **Ningún ruleset tiene `bypass_actors`**: ni la persona ni ninguna App se
     las salta por configuración.
3. **Origen de cada check.** Cada check obligatorio va atado a su
   integración con `integration_id`: `verificacion` y `carril` a GitHub
   Actions (15368, PoC A); `revision-ia` a la App del revisor (F1-6). Otra
   identidad que publique un check con el mismo nombre no cuenta.
4. **El carril no se expresa con un ruleset** (filtra por rama destino, no por
   PR ni por tarea). El ruleset pone el suelo común de la rama y el check
   **`carril`**, un workflow, impone lo propio de cada carril:
   - **no se fía de lo que el agente puede escribir.** `taskcode/tareas` no
     tiene PR ni restricción de quién empuja (cualquier identidad con
     escritura, también la del agente, hace un push fast-forward), así que
     ni el carril declarado ni una marca de «plan aprobado» en el
     frontmatter bastan. Dos reglas (decididas por Carlos con la consigna
     «la opción recomendada», a raíz de la revisión 1):
     1. **Aprobación del plan = commit firmado.** Solo cuenta si el commit de
        `approve` en `taskcode/tareas` tiene **firma verificada por la
        plataforma** (`commit.verification.verified: true` en
        `GET /repos/{o}/{r}/commits/{sha}`) y su firmante (la cuenta
        `committer` que devuelve la API) es una identidad de persona del
        proyecto; una identidad de agente o de IA, un commit sin firma o uno
        firmado por la propia plataforma (`web-flow`) no valen. Además, la
        aprobación **se ata al contenido**: el commit firmado de `approve`
        registra el sha del blob de `plan-final.md` aprobado (por ejemplo,
        un trailer `Plan-Blob: <sha>`), y `carril` exige que el blob actual
        de `plan-final.md` en `taskcode/tareas` sea ese mismo (equivale a que
        el último commit que toca `plan-final.md` sea el firmado de
        `approve`). Si el agente cambia el plan después, la aprobación deja
        de contar hasta un nuevo `approve` firmado. Es también la base de la
        segunda aprobación de riesgo alto (D4a).
     2. **Carril efectivo = el mayor** entre el declarado en la tarea y un
        **carril mínimo derivado de los ficheros del PR** (que `carril` lee
        de la API, no de la rama): **solo es errata si todos los ficheros
        del PR están bajo `docs/`** (un `.md` fuera de `docs/`, como
        `src/x.md` o `CLAUDE.md`, no lo es); un cambio en `.github/`,
        `.taskcode/` o CODEOWNERS sube más (en la PoC, a riesgo alto). El
        agente solo puede **subir** el carril, nunca bajarlo. Orden:
        errata < fix < funcionalidad < riesgo alto. **Todo PR necesita
        tarea**, también la errata: un PR sin tarea falla cerrado siempre.
        El resto de la regla de derivación (qué sube a qué por encima de
        errata) lo fija F1-10.

     La PoC F simula las dos reglas y comprueba en el esquema que la API da
     `verification.verified`; el comportamiento real en GitHub, **no
     verificado**;
   - se relanza con cada review, cada edición de la descripción y cada push;
   - comprueba, sobre el carril efectivo: plan aprobado (regla 1, F4-2) en
     funcionalidad y riesgo alto; las dos aprobaciones de persona de riesgo
     alto (D4a); `## Resultado` en la descripción del PR en los carriles con
     tarea (RF-51); en un PR a `main` desde `hotfix/*`, `revision-ia` en
     verde; si no puede decidir algo (API caída, tarea ilegible), falla;
   - **solo cuenta aprobaciones de personas** (no de Apps ni cuentas de
     máquina);
   - siempre informa: si no aplica, termina en verde con «no aplica»; nunca
     se salta por filtros de rutas o de ramas (un check obligatorio que no
     corre deja el PR pendiente para siempre).
5. **Quién aprueba cada carril**: la tabla de abajo. Es la lista exacta del
   criterio de F1-5 y la comprueba la PoC (D).
6. **CODEOWNERS** en las ramas de código (`.github/CODEOWNERS`): `/.github/`
   (workflows, scripts de CI y el propio CODEOWNERS) y `/.taskcode/`
   (configuración) a cargo de la persona, con
   `require_code_owner_review: true` en `develop` y `main`. Es una
   **mitigación parcial**: un cambio en un workflow o en la configuración no
   se integra sin la aprobación del propietario, que lo ve en el diff; pero
   con `pull_request` GitHub ejecuta el workflow de la rama del PR, así que
   `carril` o `verificacion` pueden haber salido en verde con el workflow ya
   modificado, y con una sola persona (RN-06) esa aprobación es la misma que
   ya exige el PR. Lo cierra F1-8 (por ejemplo, `carril` con
   `pull_request_target` o un workflow que solo se lee de la base); ver
   «Preguntas abiertas». En un equipo, obliga al propietario (no verificado
   en la plataforma).
7. **Cola de merge y rama al día.** Sin organización, `finish` mergea con
   `merge-pr` (F1-4) por la API cuando las puertas están en verde, con
   `strict` activado **solo en `develop`** (en `main`, no: decisión 2): si
   `develop` avanzó, primero `actualizar-rama` (merge de
   la base, sin rebase, F1-4) y vuelta a verificar. En un repo de
   organización, el ruleset `taskcode-develop-cola` (fichero `develop-cola-org.json`) sustituye a `taskcode-develop` y añade `merge_queue` con
   `merge_method: MERGE` y `finish` pone el PR en la cola (D3a). La
   verificación repetida tras cada merge la abarata la caché por SHA (F1-7).
8. **Plan de la cuenta** (D2a): la garantía exige repo público, GitHub Pro o
   Team (u organización con plan que dé rulesets en privados). Sin eso,
   `init` no aplica nada (`rechazada`, `motivo=plan`) y `doctor` declara
   «puertas no impuestas por la plataforma» en cada ejecución (RR-15, RF-64).

### Quién aprueba cada carril

<!-- tabla-carriles:inicio -->
| Carril | Base del PR | Checks obligatorios | Quién aprueba |
|---|---|---|---|
| errata | `develop` | `verificacion`, `revision-ia` (perfil rápido, D5a), `carril` | 1 persona en el PR |
| fix | `develop` | `verificacion`, `revision-ia`, `carril` | 1 persona en el PR |
| funcionalidad | `develop` | `verificacion`, `revision-ia`, `carril` | 2 aprobaciones de persona: el plan (commit de `approve` con firma verificada) y el PR |
| riesgo alto | `develop` | `verificacion`, `revision-ia`, `carril` | 2 aprobaciones de persona: con una sola persona, el plan (commit de `approve` con firma verificada) y el PR (D4a) |
| release | `main` | `verificacion` (perfil completo), `carril` | 1 persona en el PR |
| hotfix | `main` | `verificacion`, `revision-ia` (lo exige `carril`), `carril` | 1 persona en el PR |
<!-- tabla-carriles:fin -->

En ningún carril cuenta como aprobación una identidad de IA (agente
implementador o revisor IA): el PR lo abre el agente, que no puede aprobar su
propio PR; `carril` solo cuenta aprobaciones de personas; y ningún ruleset
tiene `bypass_actors`. La sección D de la PoC compara esta tabla, leída de
este fichero, con los checks de los JSON y con el número de aprobaciones:
el primer número de «Quién aprueba» tiene que ser las aprobaciones del PR
del ruleset de la base (`required_approving_review_count`) más la del plan
que exige `carril` (funcionalidad y riesgo alto), con al menos 2 en riesgo
alto (D4a), y la celda menciona el plan solo donde se exige.

El carril de la tabla es el **carril efectivo** de la decisión 4 (el mayor
entre el declarado y el mínimo por ficheros), no el que el agente declara.

## Requisitos y evidencia

| Requisito | Decisión | Evidencia |
|---|---|---|
| P3 / RR-08 las garantías las impone la plataforma | rulesets sin bypass; checks atados a su integración; push directo a `develop`/`main` imposible | PoC C (seis JSON válidos contra el esquema, `bypass_actors: []`); comportamiento real: no verificado (F2-6, F9-4) |
| RR-04 / RF-50 revisión independiente y aprobación de una persona antes del merge | `revision-ia` obligatorio en `develop`; 1 aprobación, último push aprobado, aprobaciones viejas descartadas; `carril` solo cuenta personas | PoC C (`pull_request` y `required_status_checks` válidos), D (ninguna fila aprueba con IA); no verificado en GitHub |
| P7 / RR-07 carriles proporcionales, pocas puertas humanas | suelo común por rama + check `carril`; tabla de carriles; `strict` solo en `develop` | PoC D (checks y número de aprobaciones = rulesets + `carril`, 6 filas, 5/5 desvíos detectados); PoC E (`main` deja de ser ancestro de `develop` tras una release) |
| P3 / RR-04 el agente no puede rebajar su propia puerta | carril efectivo = mayor entre el declarado y el mínimo por ficheros (solo sube); aprobación del plan solo con commit de `approve` con firma verificada de una persona | PoC F (esquema: `commit.verification` con `verified`; simulación: 7/7 casos de plan, con el blob del plan, y 8/8 de carril); comportamiento real: no verificado (F2-6); regla de derivación: F1-10 |
| RF-51 `## Resultado` exigido por un check | lo comprueba `carril` en los carriles con tarea | decisión 4; no implementado ni verificado (F6) |
| P11 / RR-15 / RF-64 mismas garantías; `doctor` dice lo que falta | modelo neutral; `leer-puertas`; D2a | PoC A (403 con aviso de plan en el privado, `[]` en el público); GitLab: F1-13 |
| RN-06 una persona con agentes igual que un equipo | D4a: plan + PR cuentan como las dos aprobaciones de riesgo alto | decisión 4; riesgo de `require_last_push_approval` en «Preguntas abiertas» |
| H5 la aprobación humana no se puede comprobar | aprobación del PR por una identidad de persona en GitHub; la del plan, por la firma verificada del commit de `approve` (F4-2) | PoC F (campo `verification` en el esquema; commit real de `develop` sin firma: el flujo 0.16 no firma); firma y claves: F1-6; no verificado en GitHub |
| H6 el agente se salta revisión y verificación | sin `bypass_actors`; checks obligatorios en el servidor; `finish` no puede mergear con puertas pendientes | PoC C; no verificado en GitHub |
| F1-1 reglas de la rama de estado | `taskcode/tareas`: sin borrado ni force, push directo | PoC C (`taskcode-tareas.json`) |
| F1-3 merge commit, ramas conservadas, tags | `allowed_merge_methods: [merge]`; `deletion` en ramas de tarea; tags sin borrar ni mover | PoC C (enum `merge`/`squash`/`rebase` del esquema; negativo `fast-forward` rechazado) |

## Consecuencias

- **F1-4**: el puerto `Plataforma` suma `aplicar-puertas` y `leer-puertas` a
  su lista cerrada; un `push` rechazado por un ruleset debe dar un resultado
  propio, no `sinred` (la pregunta abierta de F1-4 queda confirmada: con estas
  reglas ocurre en cualquier push directo a `develop`).
- **F1-6** decide la App del revisor (su `integration_id` sustituye al
  relleno `0`), con qué identidad se abren y actualizan los PR y que ninguna
  IA tenga permiso de administrar el repo ni de aprobar; además, la
  identidad de la persona **con su clave de firma** registrada en la
  plataforma, la lista de identidades de persona que `carril` acepta como
  firmantes y que la clave no esté al alcance del agente.
- **F4-2** (`approve`) crea el commit de aprobación en `taskcode/tareas`
  **firmado** con la clave de la persona y con el blob de `plan-final.md`
  aprobado; sin firma verificada, o con el plan cambiado después, no hay plan
  aprobado.
- **F1-8** cubre la seguridad del check `carril` (que su workflow no lo
  pueda cambiar el propio PR) y del acceso del agente a la clave de firma.
- **F1-7** fija los perfiles de `verificacion` por base (rápida en `develop`,
  completa en `main`) y la caché por SHA que abarata `strict` sin cola.
- **F1-10** fija la regla de derivación del carril mínimo por encima de
  errata (errata = solo `docs/`, decidido aquí), si una errata necesita
  tarea con menos ceremonia, la política de cada carril en configuración y quién lanza
  una release.
- **F1-13** traduce el modelo neutral a GitLab y verifica la cola y los
  planes de pago.
- **F2-6** implementa `aplicar-puertas` y `leer-puertas` y prueba en E2E,
  contra un repo de pruebas, todo lo marcado «no verificado».
- **F3-1** (`init`) aplica las puertas o explica por qué no puede; **F3-8**
  (`doctor`) las compara con el modelo.
- **F6** implementa el workflow `carril` y el check `revision-ia`.
- **Este repo no cambia** (RR-14): sin rulesets ni CODEOWNERS hasta el relevo;
  `ci.yml` y `comprobar-repo` siguen.

## Fuera de alcance

- Identidades, permisos de las Apps y tokens: F1-6, F6-1.
- Perfiles de verificación, caché por SHA y runners: F1-7.
- Seguridad de los workflows (qué código del PR ejecutan, secretos): F1-8.
- Política configurable por carril y modo automático: F1-10.
- Equivalencias en GitLab y qué depende del plan de pago: F1-13.
- Implementación y E2E: F2-6, F3-1, F3-8, F6, F9-4.

## Preguntas abiertas

Este ADR se planificó con un solo rol (arquitectura). Quedan sin cubrir, como
riesgos sin prueba:

- **`require_last_push_approval` con una sola persona**: si el último push de
  la rama lo hace la identidad de la persona (por ejemplo, `actualizar-rama`
  lanzado con su token, o el botón «Update branch»), ella misma no puede
  aprobarlo y el PR se queda sin aprobación posible. Los pushes a ramas de
  tarea tienen que hacerse con la identidad del agente (F1-6). Igual con el
  PR de release y el de vuelta del hotfix: si los abre la persona, no puede
  aprobarlos. No verificado.
- **`strict` y aprobaciones descartadas sin cola**: cada merge en `develop`
  deja los demás PR desactualizados; al actualizarlos, si GitHub cuenta ese
  merge como push nuevo, descarta su aprobación y la persona aprueba otra vez.
  Con 1–3 personas se acepta; F1-7 lo mide. No verificado si GitHub descarta
  la aprobación en un merge de la base sin cambios en el diff.
- **Un check obligatorio que nunca corre** (workflow roto, App caída) deja el
  PR pendiente para siempre: falla cerrado, pero `doctor` tiene que decir cuál
  falta y por qué.
- **Workflow modificado en el propio PR**: con `pull_request`, GitHub ejecuta
  el workflow de la rama del PR; CODEOWNERS obliga a la persona a aprobarlo,
  pero el check ya habrá salido verde. Ejecutar `carril` con
  `pull_request_target` (el workflow de la base, sin código del PR) lo
  cerraría; lo decide F1-8. No verificado.
- **Cómo se relanza `carril`** al terminar `revision-ia` (evento de check de
  otra App) y qué ejecución cuenta si hay varias con el mismo nombre en el
  mismo commit: no verificado.
- **App del revisor con `checks:write` de más**: podría publicar también
  `verificacion` o `carril`; los contiene `integration_id` (Actions), no
  verificado. Sus permisos mínimos: F1-6.
- **La persona como administradora**: sin `bypass_actors` no debería saltarse
  las reglas, pero puede desactivar el ruleset en Settings; `doctor` lo
  detecta al compararlo con el modelo, no lo impide.
- **Creación de tags `v*`**: cualquiera con escritura (también el agente)
  puede crear un tag; quién publica una release es de F1-6 y F1-10.
- **Riesgo alto en un equipo**: D4a cubre una sola persona; con varias, si
  hacen falta dos personas distintas en el PR lo decide F1-10.
- **Regla de derivación del carril mínimo** (F1-10): qué rutas suben a qué
  carril por encima de errata (la de la PoC F es un ejemplo), cómo se trata un PR enorme de solo
  documentación y si el tamaño del diff cuenta.
- **La clave de firma de la persona al alcance del agente**: el agente corre
  en la misma máquina y sesión que la persona; si la clave está en un
  `ssh-agent` o en un llavero desbloqueado, el agente puede firmar como ella
  y la regla 1 no la distingue. Clave con frase de paso o con toque físico
  (FIDO) lo contendrían; lo deciden F1-6 y F1-8. No verificado.
- **Qué cuenta devuelve la API como firmante**: se usa la cuenta `committer`
  del commit verificado; si GitHub la asocia por el correo del commit y una
  identidad de agente puede usar el correo de la persona, falta comprobar que
  `verified` exige la clave de esa cuenta. No verificado.
- **Pilotos privados**: ToolsaWeb y OpenDockTrading quedarían sin puertas
  impuestas en una cuenta gratuita (D2a); su plan real no se ha comprobado.

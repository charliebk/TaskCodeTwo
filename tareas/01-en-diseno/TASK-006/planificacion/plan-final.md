# Plan — TASK-006: F1-5 ADR puertas en GitHub (rulesets, checks obligatorios, CODEOWNERS) por carril

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 60 min

Senales: un ADR nuevo y una fila en el indice, sin codigo de la 2.0; evidencia de plataforma solo de lectura (API y esquema de rulesets) y sin PoC de Git local (las ramas ya las probo F1-3); 2 criterios. Precedentes: TASK-004 estimo 55 y tardo 48; TASK-005 estimo 55. Cabe en 60 min: no se trocea.

### Que se crea y que no se toca

1. **Se crea** `docs/adr/F1-5-puertas-en-github.md` con la plantilla de `docs/adr/README.md`, incluida «Como se ve en GitHub».
2. **Se extiende** `docs/adr/README.md` con la fila F1-5 / TASK-006.
3. **Se crean** `poc/poc-f1-5.sh` y `poc/poc-f1-5.log`: solo `GET` a la API y validacion local de los JSON de rulesets, en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-006\` (borrada al terminar); el esquema descargado va en su propia carpeta vacia.
4. **No se tocan** F1-1..F1-4 (se citan) ni `.github/` de este repo (`ci.yml` y su check `comprobar-repo` siguen hasta el relevo, RR-14). No se escribe nada en GitHub (D1).

### Que decide el ADR

- **Modelo de puertas neutral y su traduccion**: el nucleo describe cada rama con reglas abstractas (sin borrado, sin forzar, solo por PR con aprobaciones y metodos, checks con su origen); el adaptador lo traduce a rulesets de GitHub (GitLab, F1-13). Al puerto `Plataforma` de F1-4 se anaden `aplicar-puertas` (idempotente; la lanza `init` con la identidad de la persona, exige administrar el repo) y `leer-puertas` (la usa `doctor`, F3-8).
- **Reglas por rama** (cierra lo que dejaron F1-1 y F1-3):
  - `develop`: sin borrado ni force-push, solo por PR; 1 aprobacion; aprobaciones viejas descartadas con cada push y aprobacion del ultimo push; solo merge commit (D4a de F1-3); checks obligatorios `verificacion`, `revision-ia`, `carril`.
  - `main`: igual, con `verificacion` (perfil completo) y `carril`.
  - `feature/**`, `fix/**`, `hotfix/**`: sin borrado ni force-push (las ramas se conservan, F1-3).
  - `taskcode/tareas`: sin borrado ni force-push, sin PR ni checks; push directo del CLI permitido (F1-1, F1-2).
  - Tags `v*`: ni borrar ni mover.
- **Cada check obligatorio atado a su origen** con el `integration_id` de la regla: `verificacion` a GitHub Actions, `revision-ia` a la App del revisor (F1-6). Sin esto, la identidad del agente podria publicar un check con ese nombre.
- **El carril no se expresa con un ruleset** (filtra por rama, no por PR): el ruleset pone un suelo comun por rama y el check `carril` (un workflow) impone lo propio de cada carril: lee el carril de la tarea en `taskcode/tareas`, se relanza con cada review y comprueba plan aprobado por una persona (F4-2) en funcionalidad, 2 aprobaciones en riesgo alto, `## Resultado` en el PR (RF-51) y, en PR a `main` desde `hotfix/*`, `revision-ia` en verde. Un check obligatorio siempre informa (si no aplica, «no aplica»); nunca se salta por filtros de rutas.
- **Tabla exacta del criterio 1** (carril → checks → quien aprueba):

  | Carril | Checks | Quien aprueba |
  |---|---|---|
  | errata | `verificacion`, `revision-ia` (D5), `carril` | 1 persona |
  | fix | `verificacion`, `revision-ia`, `carril` | 1 persona |
  | funcionalidad | lo mismo | la persona aprueba el plan y el PR |
  | riesgo alto | lo mismo | 2 aprobaciones (D4) |
  | release | `verificacion` (completa), `carril` | 1 persona |
  | hotfix | `verificacion`, `revision-ia`, `carril` | 1 persona |

  En ningun carril cuenta como aprobacion una identidad de IA.
- **CODEOWNERS** en las ramas de codigo: `.github/**`, `CODEOWNERS` y `.taskcode/` a cargo de la persona, con revision del propietario exigida: el agente no puede rebajar una puerta editando el workflow de su propio PR.
- **Cola de merge y rama al dia** segun D3 y D6.

### Orden

1. `poc-f1-5.sh`: `gh api` solo `GET` sobre los rulesets de `charliebk/taskcodetwo-poc-f1-1` y `charliebk/TaskCodeTwo` (disponibilidad segun visibilidad); descarga de la descripcion OpenAPI de rulesets y validacion de los JSON de cada rama; log guardado.
2. ADR: modelo neutral, reglas por rama, tabla de carriles, origen de los checks, CODEOWNERS; tabla de requisitos (P3, P7, P11, RR-04, RR-07, RR-08, RR-15, RF-50, RF-51, RF-64, H5, H6), comprobando cada ID antes de citarlo.
3. Fila del indice.
4. Borrar la carpeta temporal tras comprobar que no queda nada que conservar.

## Lo que el rol no cubrio

Solo arquitectura. Sin revisar:

- **Riesgos** (criterio 1): check obligatorio que nunca se ejecuta deja el PR pendiente para siempre; workflow modificado en el propio PR (lo contiene CODEOWNERS, sin probar); App con `checks:write` de mas; reglas que el administrador (la persona) puede saltarse.
- **Dominio**: si una errata necesita revision IA (D5), como se cuentan dos personas en un proyecto individual (D4), quien lanza una release (F1-6, F1-10).
- **Testing**: el E2E real de las puertas es de F2-6 y F9-4; aqui solo esquema y disponibilidad.
- **Sin verificar por el rol**: si `charliebk/TaskCodeTwo` es publico; que la cola de merge solo existe en repos de organizacion; que en un repo privado de cuenta gratuita no hay rulesets. Lo comprueba la PoC o la documentacion, y se marca en el ADR.

## Riesgos aceptados y que los contiene

- **Pilotos privados** (ToolsaWeb, OpenDockTrading, REQUISITOS §1): en cuenta gratuita GitHub no impondria alli las puertas; lo contienen D2 y el aviso de `doctor` (RR-15).
- **Sin cola en cuenta personal**: merge por `taskctl merge` con la rama al dia; re-verificar los PR abiertos tras cada merge crece con los PR concurrentes; se acepta para 1–3 personas; F1-7 lo mide con cache por SHA.
- **Logica de carril en un workflow**: si falla, la puerta queda pendiente, nunca abierta; F1-10 la hace configurable.
- **Evidencia solo de esquema y disponibilidad**: el comportamiento real (aprobar, descartar aprobaciones viejas, origen del check) queda «no verificado» y pasa a F2-6 y F9-4.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio). Secciones de `poc/poc-f1-5.sh` en Git Bash, solo lectura en GitHub, sin crear repos ni gists; carpeta borrada al terminar. Ninguno pesado.

- **Lista exacta de checks y quien aprueba cada carril → 2**
  1. Un JSON de ruleset por rama (`develop`, `main`, ramas de tarea, `taskcode/tareas`, tags) valida contra el esquema descargado: claves, enums de `rules[].type`, `allowed_merge_methods`, `integration_id`.
  2. Las filas de la tabla de carriles coinciden con los checks de los JSON (script que compara nombres); ningun carril aprueba con una identidad de IA.
- **Cumple F1-5 y los requisitos que cita → 2**
  1. `GET /repos/charliebk/taskcodetwo-poc-f1-1/rulesets` y `GET /repos/charliebk/TaskCodeTwo/rulesets`: se anota codigo y mensaje (se espera 403 con aviso de plan en el privado y `[]` en el publico). Evidencia de D2.
  2. Tabla requisito → decision → evidencia; al cerrar: fila del indice, `bash .github/scripts/comprobar-repo.sh` en verde y `taskctl doctor` sin errores.

## Lo que necesita decision de una persona

Decisiones respondidas por la persona (Carlos) el 2026-11-14 con la consigna «responder la opcion recomendada a todas las preguntas»:

- **D1. Evidencia en GitHub** → **(a)** solo lectura: `GET` de rulesets y esquema OpenAPI; no se escribe nada en GitHub. Descartadas (b) crear un ruleset en el repo temporal, (c) hacerlo publico, (d) aplicarlos en `charliebk/TaskCodeTwo` (rompe el flujo 0.16, RR-14).
- **D2. Repos privados en cuenta gratuita** → **(a)** el ADR exige GitHub Pro, Team o repo publico para la garantia; sin eso, `doctor` declara «puertas no impuestas por la plataforma».
- **D3. Cola de merge** → **(a)** `taskctl merge` por la API; la cola solo si el repo es de una organizacion.
- **D4. Riesgo alto con una sola persona** (RN-06) → **(a)** cuentan como las dos aprobaciones la de la persona en el plan y la de la persona en el PR.
- **D5. `revision-ia` en errata** → **(a)** tambien se ejecuta, con perfil rapido (P3 la exige en todo PR).
- **D6. Rama al dia (`strict`) en `develop` sin cola** → **(a)** activada.

Sin troceo: la duracion estimada no pasa de 60 min.

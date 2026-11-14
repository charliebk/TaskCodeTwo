# Informe de revision — TASK-006 (ronda 3)

- Commit revisado: 38166f29fef407ae4ef1c93b19b8d98bea0499a4
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-5-puertas-en-github.md:407 | plan |
| IMP-2 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-006/poc/rulesets/main.json:37 | plan |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-5-puertas-en-github.md:418 | plan |
| IMP-4 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-006/poc/poc-f1-5.sh:359 | implementacion |
| IMP-5 | IMPORTANTE | corregido | docs/adr/F1-5-puertas-en-github.md:387 | plan |
| MEN-5 | MENOR | abierto | docs/adr/F1-5-puertas-en-github.md:419 | plan |
| MEN-6 | MENOR | abierto | docs/adr/F1-5-puertas-en-github.md:287 | plan |

Ningun CRITICO. Ningun IMPORTANTE nuevo. MEN-5 y MEN-6 los introduce el texto de esta correccion; no abren ronda (son MENOR) y no bloquean el cierre, pero conviene corregir el texto antes de F4-2.

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el enunciado o en los criterios de aceptacion de la tarea.

## Verificacion por hallazgo

- **IMP-1 — corregido.** La regla 1 (ADR:407-411) exige commit de `approve` con `commit.verification.verified: true` y committer de persona; el frontmatter no cuenta ("ni el carril declarado ni una marca de plan aprobado en el frontmatter bastan"). La seccion F de la PoC lo recoge: `sin firma`, `verificado de la App` y `web-flow` dan «no cuenta» (7/7 casos de plan). No verificado en GitHub: que la API devuelva `verified: true` para una firma de la persona es no verificado, como declara el ADR.
- **IMP-2 — corregido.** `main.json:37` tiene `strict_required_status_checks_policy: false`; `develop.json:37` y `develop-cola-org.json:37` lo tienen en `true`. Tras la ejecucion, los JSON generados por la PoC son byte a byte iguales a los versionados (`cmp`), asi que los versionados son los generados. La PoC E confirma que `main` deja de ser ancestro de `develop` tras una release por merge commit (ADR:193-206). Coherente con D6a y decision 2 (ADR:54, :272-274).
- **IMP-3 — corregido.** La regla ahora exige que el commit firmado de `approve` registre el blob de `plan-final.md` (trailer `Plan-Blob`, ADR:418) y que el blob actual coincida. La PoC F reproduce el escenario del IMP-3 a nivel de estado: «firmado, pero el agente cambio el plan despues» → no cuenta; «firmado de nuevo sobre el plan cambiado» → cuenta. Limite: la simulacion trabaja con un unico commit y un blob, no con una cadena de commits; la cadena real (T1 approve, T2 cambio del agente) queda como no verificado. La formulacion «equivale a que el ultimo commit...» es incorrecta; ver MEN-5.
- **IMP-4 — corregido.** Ejecutada la PoC (seccion F): `docs/a.md` → errata; `src/x.md` → fix; `CLAUDE.md` → fix; `src/main.rs` → fix; `.github/workflows/carril.yml` → riesgo alto. El `.md` fuera de `docs/` ya no es errata. El texto (ADR:248, :426) y el codigo (poc-f1-5.sh:359) coinciden.
- **IMP-5 — corregido (a nivel de diseno).** La errata sale de `fix/task-NNN-<slug>` (ADR:387, :386-387), que casa con D1a de F1-3 (`<tipo>/task-NNN-<slug>`, tipo ∈ feature, fix, hotfix; F1-3:63) y con la unicidad por ID de `start` (F1-3:344). Dos erratas con el mismo slug ya no colisionan en la rama, porque el ID es distinto; dos con el mismo ID las rechaza la comprobacion de F1-2/F1-3 (RR-12, F1-3:412). Esa comprobacion contra el remoto (F1-3 PoC 1d) no se ha reejecutado en esta ronda; se cita como verificada en F1-3.

## Verificacion realizada

- Commit: HEAD era `32ceda5`. El diff `38166f2..HEAD` solo toca `revision/` y una linea de `tarea.md`; ADR, PoC y rulesets no cambian. Por eso el commit revisado es `38166f2`.
- Puerta determinista: el repo es documental (ADR, tareas, PoC); no hay build ni suite de tests que ejecutar. Verificacion ejecutable:
  - `bash .github/scripts/comprobar-repo.sh`: exit 0, «Tareas: 93; enlaces relativos: 40; fallos: 0».
  - PoC `poc-f1-5.sh` ejecutada con carpeta de trabajo fuera del repo (`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-006-r3`) y carpeta de artefactos vacia; exit 0. Solo GET: 11 llamadas `gh api --method GET`, 0 con otro metodo, 2 `gh repo view`. Resultados: A (rulesets de `taskcodetwo-poc-f1-1` y `TaskCodeTwo`: 403 con aviso de plan en el privado, `[]` en el publico; id de GitHub Actions 15368); C rulesets=6 errores=0, negativos 7/7; D filas=6 fallos=0, negativos 5/5; E «no»; F plan 7/7, carril efectivo 8/8. Coinciden con los valores citados en ADR:235 y en el log del repo.
  - Carpeta de trabajo y de artefactos borradas al terminar.
- Reproduccion de MEN-5 y del caso de rename (repo git temporal, borrado tras la prueba):
  - Cadena T1 `approve` firmado (blob v1), T2 agente cambia a v2, T3 agente restaura v1 sin firma. Blob actual = blob de T1 (`047575825627`), pero el ultimo commit que toca `plan-final.md` es T3 (sin firma). La regla por blob cuenta la aprobacion; la formulacion «ultimo commit firmado» no. Las dos no son equivalentes.
  - Rename `.github/workflows/carril.yml` → `docs/carril.yml`: `git diff -M --name-status` da `R100 .github/workflows/carril.yml docs/carril.yml`. Un `minimo` que lea solo el nombre nuevo da `errata` (ver nota 1).
- `git status` del repo limpio salvo `informe-revision-3.md`. No se ha hecho ningun commit ni ninguna escritura en GitHub.

## Notas (fuera de la tabla)

1. **Rename hacia `docs/` (no es hallazgo nuevo abierto).** Si la implementacion de `carril` deriva el carril minimo solo del nombre nuevo que devuelve la API para cada fichero, un PR que mueve `.github/**` a `docs/` sale como errata. El texto del ADR (ADR:426, «todos los ficheros del PR») cubre el nombre de origen, pero no lo dice; y la API de PR devuelve `previous_filename` en los renombrados. No es errata por el texto; es una trampa para F4-2/F1-10. Recomiendo que el ADR diga «rutas de origen y destino». Severidad si se implementa mal: IMPORTANTE. Reproducido en git; la API de GitHub no se ha consultado (no verificado).
2. **Carril minimo que nunca sube a funcionalidad (pre-existente).** `minimo()` (poc-f1-5.sh:357-361) devuelve `errata`, `fix` o `riesgo alto`, nunca `funcionalidad`. Un cambio de codigo con carril declarado `fix` no exige plan aprobado. El ADR deja la derivacion a F1-10 (ADR:317-318, :425-433), pero la clausula P3 («el agente no puede rebajar su propia puerta») se cumple solo a medias en esta version. Ya existia en ronda 2 y no es un fallo de la correccion; se deja anotado para F1-10.
3. **Caso de mayusculas (pre-existente).** `minimo(['.Github/workflows/carril.yml'])` da `fix`, no `riesgo alto`, porque la comparacion es sensible a mayusculas. En GitHub (Linux) `.Github/` no es un workflow ejecutado, pero en un clon de Windows o macOS puede colisionar con `.github/`. Sin accion obligatoria; conviene normalizar a minusculas en la regla de F1-10.
4. **Errata y funcionalidad comparten puertas humanas.** Segun la tabla de carriles (ADR:483-488), errata y fix exigen 1 persona y los mismos checks; solo cambia el plan. Por eso el rename de la nota 1 solo baja puertas cuando lo que se rebaja es `riesgo alto` (2 aprobaciones) a errata (1). Pesa por eso en la nota 1.
5. **Pendiente de la ronda 2 no tratado en esta ronda (MENOR, no abre ronda).** Sin cambios respecto a la ronda 2: la comprobacion «funcionalidad sin aprobacion del plan» de la PoC D se detecta por numero de aprobaciones, no por ausencia de «plan» (poc-f1-5.sh:264).

## Descripcion de los MENOR nuevos

- **MEN-5** (`docs/adr/F1-5-puertas-en-github.md:419`): «blob actual de `plan-final.md` ... sea ese mismo (equivale a que el ultimo commit que toca `plan-final.md` sea el firmado de `approve`)». No es equivalente: con la reversion T1→T2→T3 del apartado de verificacion, el blob coincide y el ultimo commit no es el firmado. Correccion sugerida: quitar la equivalencia y dejar la regla por blob, o cambiar la regla a «ultimo commit firmado»; elegir una. La PoC F no tiene el caso de reversion; anadirlo si se elige la regla por blob.
- **MEN-6** (`docs/adr/F1-5-puertas-en-github.md:287`): cita «RR-12: el estado de todas las tareas a la vista». RR-12 (REQUISITOS-FUNCIONALES.md:29) es «nadie pisa nombres, tareas ni trabajo»: incluye nombres de rama unicos (la parte que aplica aqui), y el estado mas reciente es solo una de sus partes. Correccion sugerida: citar «RR-12: nombres de rama unicos» (F1-3:30, :412).

## Lecturas fuera del contexto permitido

- `C:\Users\nullcad2025\.claude\plugins\cache\taskcode-marketplace\taskcode-plugin\0.16.1\skills\task-workflow\revision.md` y la skill `code-quality-reviewer`: procedimiento comun exigido por la skill.
- `docs/adr/F1-3-flujo-de-ramas-worktrees-release.md` (lineas 5, 30, 63, 344, 361, 412): coherencia de nombres de rama y unicidad, pedida en la peticion.
- `docs/plan/REQUISITOS-FUNCIONALES.md` (linea 29) y `docs/plan/PLAN-TASKCODE-2.md` (lineas 40 y similares, por grep): definicion de RR-12 para la nota MEN-6.
- `tareas/03-en-revision/TASK-006/revision/informe-revision-2.md` y `peticion-revision-3.md`: lectura de la peticion y de los hallazgos a verificar.
- `tareas/03-en-revision/TASK-006/poc/rulesets/*.json` (grep y `cmp` contra los generados): verificacion de IMP-2.
- `docs/adr/F1-5-puertas-en-github.md` leido por tramos, no completo: solo las partes del diff de esta ronda y las citadas arriba.

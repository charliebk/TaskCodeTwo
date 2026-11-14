# Informe de revision — TASK-011 (ronda 2)

- Commit revisado: 08e34e710ec49d3d7dbe31e6302b8fd79f2809c5
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-10-carriles-y-modo-automatico.md:369 | plan |

Sin hallazgos nuevos CRITICO ni IMPORTANTE introducidos por la correccion. IMP-1 no esta resuelto: la correccion cierra el caso `tipo` con base `develop` y el de `hotfix` desde otra rama, pero no el de `release` desde una rama distinta de `develop`.

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### IMP-1 — `tipo: release` contra `main` desde una rama `feature/*` quita plan, approve y revision-ia (correccion incompleta)

- Donde: docs/adr/F1-10-carriles-y-modo-automatico.md:369 (regla «solo elige fila con la base real del PR», que solo exige la base para release); fila `release` en docs/adr/F1-10-carriles-y-modo-automatico.md:391; codigo en tareas/03-en-revision/TASK-011/poc/poc-f1-10.py:221-222 (`fila_politica`, rama `release` sin comprobar la rama de origen).
- Que pasa: para `hotfix` la correccion exige la rama `hotfix/*`; para `release` no exige ninguna rama. Un PR `feature/*` → `main` con `tipo: release` en `tarea.md` cae en la fila `release`: sin plan firmado, sin parada `approve`, una sola aprobacion, sin `revision-ia` y 0 roles, aunque su carril efectivo sea `funcionalidad`.
- Reproduccion: script en el scratchpad de la sesion (fuera del repo) que carga la PoC con `importlib` y llama a `politica` y `cadena_nueva` con la tabla leida del ADR. Salida:
  - M0 `funcionalidad`, base `develop`: fila=funcionalidad, plan=True, aprob=2, checks=carril,revision-ia,verificacion, paradas=approve,merge-pr. Cadena: plan, approve, start, review, merge-pr.
  - M1 `funcionalidad`, `tipo: release`, base `main`, rama `feature/task-011-x`: fila=release, plan=False, aprob=1, checks=carril,verificacion, paradas=merge-pr,tag. Cadena: plan, start, review, merge-pr, tag (sin approve).
  - M2 `fix` con diff en `src/`, `tipo: release`, base `main`, rama `feature/*`: fila=release, sin `revision-ia`, roles>=0 (la fila fix exige revision-ia y 1 rol).
  - M3 `funcionalidad`, `tipo: hotfix`, base `main`, rama `hotfix/task-012-x`: fila=hotfix, plan=False, sin parada approve (mantiene revision-ia).
  - M5 `funcionalidad`, `tipo: release`, base `main`, rama `develop`: igual que M1 (la rama `develop` es la unica que el ADR describe para release, D3a).
  - Comprobado sin fallo: `tipo: errata` contra `main` (M4) falla cerrado; `hotfix` desde `feature/*` contra `main` falla cerrado; `tipo` ignorado contra `develop` (M6); barrido de la PoC C (1440 combinaciones, 0 `continuar`).
- Impacto: el ADR afirma «El tipo no baja la puerta» (8/8 casos) y que el texto de `tarea.md` (no confiable, A10) no baja ninguna puerta. Eso es falso para un cambio `funcionalidad` que llegue a `main` con `tipo: release` o, con rama `hotfix/*`, `tipo: hotfix`: se salta el plan firmado y la parada `approve`. Lo que lo frena hoy es solo lo que imponga GitHub (ruleset) y la persona en `merge-pr`, que el ADR no cuenta como puerta de carril.
- Correccion sugerida: exigir la rama de origen para `release` (`develop`, como fija D3a), igual que `hotfix` exige `hotfix/*`, y rechazar con `ErrorCarril` cualquier otro caso; o, si se quiere, que una fila de `main` nunca baje por debajo del carril efectivo `funcionalidad` (solo puede añadir exigencias). Escribirlo en la decision 1 y en la columna «Base del PR» de la tabla, y añadir a la PoC C los casos M1 y M3 con el resultado esperado (ERROR o plan y approve). No es CRITICO: no hay perdida de datos ni estado corrupto; se mantiene IMPORTANTE porque un cambio funcionalidad llega a `main` sin su puerta de carril por un campo de texto.
- Origen `plan`: la decision 1 fija la rama de `hotfix` y no la de `release`; el codigo sigue lo escrito.

## Verificacion realizada

- Alcance: verificacion acotada de IMP-1 y del delta de la correccion (commit 232628f, y 08e34e7 que solo añade la constancia de autorrevision). Los demas hallazgos de la ronda 1 (MEN-1 a MEN-4) los comprueba quien coordina y no se han vuelto a revisar.
- No se clono el repo. El arbol estaba limpio en HEAD 8b0f4e0, y `git diff --stat 08e34e7 8b0f4e0` solo toca `revision/` y `tarea.md`: el ADR y la PoC revisados son los del commit 08e34e7.
- Puerta: no hay suite de tests de codigo en el diff (docs y una PoC en Python). La verificacion ejecutable es la PoC y `comprobar-repo.sh`.
- PoC: `python -I -B tareas/03-en-revision/TASK-011/poc/poc-f1-10.py <carpeta> <ADR> <carpeta Personal>` con carpeta `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-011-r2` (vacia; borrada al terminar). Salida: exit 0, «todo como se espera: A, B, C y D». A: 22/22 casos; B: 6 filas, 0 fallos, 12/12 desvios detectados; C: 8/8 tipo y base, 1440 combinaciones con 0 `continuar`, 140 combinaciones de tipo o base sin fila que fallan cerrado; D: 5/5 aceptadas, 17/17 rechazadas. Los recuentos de la PoC (finish=156; approve automatico=114, persona=43 en ToolsaWeb) coinciden con lo que cita el ADR.
- `bash .github/scripts/comprobar-repo.sh`: «Tareas: 93; enlaces relativos: 82; fallos: 0», exit 0.
- Derivacion (MEN-2 de la ronda 1): `.github`, `.taskcode` exactos, `.GitHub/` en mayusculas y `.githubx/` (fix) dan lo esperado. `docs//x.md` da errata.
- Mutaciones de politica (M0 a M6) y de rutas: en el scratchpad de la sesion, sin tocar el repo.

## Notas (MENOR, no abren ronda)

- `./.github/workflows/ci.yml` clasifica como `fix` y no como `riesgo-alto` (`clase_ruta` mira el primer componente, que aqui es `.`). Git y la API de GitHub no emiten el prefijo `./`, asi que no es alcanzable por el flujo real. Es defensa en profundidad: normalizar o rechazar el prefijo `./`. La afirmacion del informe de la ronda 1 de que esa ruta da error no es exacta: se acepta como `fix`.
- `tipo: errata` contra `main` falla cerrado (no hay fila): un cambio solo de `docs/` que vaya directo a `main` necesita `tipo: hotfix` y rama `hotfix/*`. Es seguro, pero el ADR no lo dice.
- El minimo por ficheros solo sube a riesgo-alto en release y hotfix (decision 4, M2 incluido). Es lo que dice el ADR, pero conviene saberlo: un release con diff en `src/` no lleva `revision-ia`.

## Lecturas fuera del contexto permitido

- `~/.claude/plugins/cache/.../skills/task-workflow/revision.md`: reglas comunes de la revision (plantilla del informe, rondas, veredicto). Lo exige la skill revisora.
- `docs/adr/F1-5-puertas-en-github.md` y `docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md`: los lee la PoC (tablas F1-5 y perfiles F1-7); no se han abierto a mano.
- `ToolsaWeb`, `OpenDockTrading`, `TaskCodeTwo` (repos vecinos): solo `git log`, `git show`, `git archive` de commits fijos desde la PoC; no se han extraido ni ejecutado.
- `.github/scripts/comprobar-repo.sh`: ejecutado (lo pide la peticion).

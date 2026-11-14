# Informe de revision — TASK-006 (ronda 2)

- Commit revisado: 3e277420be9417232556523146d737d5f2468aa3
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-5-puertas-en-github.md:387 | plan |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-5-puertas-en-github.md:355 | plan |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-5-puertas-en-github.md:387 | plan |
| IMP-4 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-006/poc/poc-f1-5.sh:346 | implementacion |
| IMP-5 | IMPORTANTE | corregido | docs/adr/F1-5-puertas-en-github.md:365 | plan |

IMP-1 e IMP-2 son de la ronda 1 (verificacion acotada). IMP-3, IMP-4 e IMP-5 son
nuevos: los introduce la propia correccion de esta ronda, que es el unico caso
en que esta ronda abre hallazgos.

## Verificacion por hallazgo

- **IMP-1 — corregido.** El frontmatter ya no cuenta: la regla 1 exige un commit
  de `approve` con `commit.verification.verified: true` y committer de persona;
  la regla 2 solo sube el carril (mayor entre declarado y minimo por ficheros),
  asi que el agente no lo puede bajar. El residuo (la aprobacion no queda atada al
  contenido del plan) es IMP-3; la clave de firma al alcance del agente ya figura
  en «Preguntas abiertas» como no verificado.
- **IMP-2 — corregido.** `main.json` tiene `strict_required_status_checks_policy:
  false`; `develop.json` y `develop-cola-org.json` lo tienen en `true`. La decision 2
  y D6a (ADR:54, :355) lo dicen igual. La PoC E reproduce que `main` deja de ser
  ancestro de `develop` tras una release por merge commit. Que GitHub marque como
  desactualizado el PR `develop`→`main` con `strict` es **no verificado**, como
  declara el ADR.

### IMP-3 — La aprobacion del plan no queda atada al contenido aprobado

- Donde: docs/adr/F1-5-puertas-en-github.md:387 (regla 1); tareas/03-en-revision/TASK-006/poc/poc-f1-5.sh:330-331 (`plan_aprobado`).
- Que pasa: la regla 1 cuenta un commit de `approve` firmado por una persona y no
  dice nada de los commits posteriores que cambien `plan-final.md`. Si despues el
  agente empuja a `taskcode/tareas` (fast-forward, sin PR) un cambio de alcance o
  de criterios, `carril` sigue viendo la aprobacion valida.
- Reproduccion: sobre la regla escrita y la simulacion. `plan_aprobado(commit)`
  recibe un unico commit, y los 4 casos de la seccion F (poc-f1-5.sh:333-336) son
  de un commit aislado; ninguno tiene commits posteriores. Secuencia que la regla
  deberia rechazar y no rechaza: T1, approve firmado por la persona sobre
  `plan-final.md` con alcance X; T2, push del agente que cambia `plan-final.md` a
  alcance Y sin tocar el frontmatter; resultado de `carril`: plan aprobado = si.
  No ejecutado en GitHub (no verificado).
- Impacto: en funcionalidad y riesgo alto, la aprobacion del plan (y la segunda
  de D4a en riesgo alto) cubre un alcance que la persona no aprobo. El PR sigue
  necesitando aprobacion humana sobre el diff, lo que atenua el riesgo, pero el
  gate del plan ya no hace lo que la tabla dice.
- Sugerencia: atar la aprobacion al contenido. El commit de `approve` registra el
  sha del blob de `plan-final.md`, y `carril` exige que el ultimo commit que toca
  ese fichero sea el `approve` (o que el blob coincida). Anadir a la seccion F un
  caso con commit posterior que la regla debe rechazar.

### IMP-4 — La regla de derivacion del carril minimo trata cualquier `.md` como errata

- Donde: poc-f1-5.sh:346 (`f.endswith('.md')`); contradice el texto normativo de ADR:398 («fuera de `docs/` no puede ser errata»).
- Que pasa: la simulacion da carril errata a cualquier PR cuyos ficheros acaben en
  `.md`, esten donde esten.
- Reproduccion: se ejecuto con `python -I` la misma funcion `minimo` del guion:
  `['src/x.md']` -> errata; `['CLAUDE.md']` -> errata; `['.claude/agents/revisor.md']` -> errata;
  `['src/main.rs']` -> fix. La seccion F no tiene ningun caso con un `.md` fuera de
  `docs/`, por eso su «carril efectivo 6/6» no lo detecta.
- Impacto: un PR que cambia las instrucciones del agente (`CLAUDE.md`,
  `.claude/**/*.md`) o documentacion fuera de `docs/` puede ir con carril errata
  (una persona, sin plan). Es justo lo que la decision 4 quiere impedir. Como
  ejemplo de F1-10 es discutible, pero el ejemplo contradice el texto que lo
  define.
- Sugerencia: que la regla de errata sea «solo bajo `docs/`» (o la lista que fije
  F1-10), no «cualquier `.md`». Anadir a la seccion F los casos `src/x.md` -> fix,
  `CLAUDE.md` -> fix y `docs/guia.md` -> errata, y que texto y codigo coincidan.

### IMP-5 — La rama de errata `fix/errata-<slug>` no cumple el patron ni la unicidad de F1-3

- Donde: docs/adr/F1-5-puertas-en-github.md:365 (y :271). Incoherente con
  docs/adr/F1-3-flujo-de-ramas-worktrees-release.md:63 (D1a: `<tipo>/task-NNN-<slug>`,
  unicidad por el ID) y :412 (RR-12: comprobacion `*/task-NNN-*` contra el remoto).
- Que pasa: una errata no tiene ID de tarea, asi que la unicidad por ID no la cubre
  y no hay comprobacion de nombre. Dos erratas con el mismo slug comparten rama.
- Reproduccion (git local, remoto bare en carpeta temporal):
  1. `fix/errata-typo` subida desde `develop` con un commit, sin mergear: aceptada.
  2. Rama nueva desde `develop` con otro commit, empujada a `fix/errata-typo`:
     rechazada («non-fast-forward»; el ruleset impide forzar).
  3. Tras mergear la primera, la segunda push con el mismo nombre se acepta como
     fast-forward sobre la rama conservada: su PR arrastra el historial de la
     primera errata.
  No ejecutado contra GitHub.
- Impacto: la segunda errata queda bloqueada (hay que renombrarla a mano) o se
  mezcla con el historial de otra. No hay perdida de datos, pero F1-3 promete
  ramas unicas y conservadas, y esta errata queda fuera de esa garantia.
- Sugerencia: decidir en F1-3 (o en F1-10) un patron con unicidad para las ramas
  sin tarea (por ejemplo, un sufijo con la fecha o el sha corto) comprobado contra
  el remoto, o exigir que toda errata lleve ID de tarea. Mientras tanto, anotar la
  divergencia en el ADR.

## Veredicto propuesto

`cambios-solicitados`: quedan abiertos IMP-3, IMP-4 e IMP-5 (IMPORTANTE). IMP-1 e
IMP-2 estan corregidos. No se ha escrito la linea de veredicto (la peticion pide
dejarla tal cual; la escribe `taskctl veredicto`).

## Verificacion realizada

- Commit: HEAD al empezar era `eb4ea53`, que solo anade la peticion y el informe
  de ronda 2 y cambia una linea de `tarea.md`; el ADR y los JSON no cambian entre
  `3e27742` y `eb4ea53`. El diff de la ronda se tomo de la peticion (no se regenero).
- Puerta: el repo es documental (ADR, tareas y PoC). No hay build ni linter que
  ejecutar (no comprobado). La verificacion ejecutable es `comprobar-repo.sh` y la PoC.
- `bash .github/scripts/comprobar-repo.sh`: exit 0, «Tareas: 93; enlaces relativos: 40; fallos: 0».
- PoC `poc-f1-5.sh` (carpeta de trabajo fuera del repo, carpeta de artefactos
  vacia), exit 0:
  - A: 11 llamadas `gh api --method GET` y 0 con otro metodo; `gh repo view` 2.
    Repos privado 403, publico `[]`/404, id de GitHub Actions 15368.
  - B: esquema descargado de `github/rest-api-description` (commit 2a33d9cc).
  - C: 6 rulesets validos, 0 errores, negativos 7/7. Versionados: FALTA (carpeta vacia; esperado).
  - D: 6 filas, 0 fallos, negativos 5/5.
  - E: `main` no es ancestro de `develop` tras la release 1.
  - F: esquema con `verification`; plan 4/4; carril efectivo 6/6.
- PoC segunda ejecucion, con copias de `poc/rulesets/*.json` del repo como
  artefactos: C «versionados validos=6/6 iguales=6/6». Confirma MEN-3 y que
  `main.json` (strict=false) es el generado por el modelo.
- Repro de IMP-4 (funcion `minimo` con `python -I`): 4 casos, resultados arriba.
- Repro de IMP-5 (git local, remoto bare): comandos y salidas arriba.
- IMP-3: analisis de la regla escrita y de `plan_aprobado` (poc-f1-5.sh:330-336). No es una ejecucion en GitHub.
- El repo quedo limpio (`git status` vacio) tras las ejecuciones.
- Las carpetas temporales de la ronda (`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-006-r2` y `task-006-r2-log`) se borran al terminar.
- No se ejecuto ninguna llamada de escritura a GitHub; la PoC solo hace GET.

## Notas (fuera de la tabla)

- MENOR, no abre ronda: en la seccion D, el control «funcionalidad sin aprobacion del plan» se detecta por el numero de aprobaciones, no por la ausencia de «plan» (esa comprobacion existe, poc-f1-5.sh:264, pero el control no la ejercita por separado). Sin accion obligatoria.
- Las rutas de F1-3 citadas (D1a, RR-12) se consultaron por lineas (grep), no completas; el ADR se leyo solo en las lineas del diff de la peticion.

## Lecturas fuera del contexto permitido

- `C:\Users\nullcad2025\.claude\plugins\cache\taskcode-marketplace\taskcode-plugin\0.16.1\skills\task-workflow\revision.md`: procedimiento comun y plantilla de informe, exigido por la skill.
- `docs/adr/F1-3-flujo-de-ramas-worktrees-release.md` (lineas 63, 338-345, 361, 412): coherencia de nombres de rama y unicidad, pedida en la peticion.
- `tareas/03-en-revision/TASK-006/revision/informe-revision-2.md`: lectura de la plantilla de cabecera.

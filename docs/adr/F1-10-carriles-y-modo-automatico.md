# ADR F1-10 — Carriles y modo automático: qué para y dónde

- Estado: **aceptada** (2026-11-14)
- Tarea: TASK-011 · Plan: [F1-10 en PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) (§4 P7 y tabla de la Fase 1)
- Requisitos: la fila F1-10 del plan cita P7; este ADR responde a RR-02, RR-04, RR-07, RR-08, RR-17, RR-18, RF-21, RF-27, RF-33, RF-50, RF-51 y RN-06 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md) (RR-18 viene de la [ampliación del 2026-11-14](../plan/AMPLIACION-2026-11-14.md)) y a H5 y H6 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
- Depende de: [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md) (release y hotfix), [ADR F1-5](F1-5-puertas-en-github.md) (tabla de carriles, check `carril`, carril efectivo, errata solo con `docs/`), [ADR F1-6](F1-6-identidades-agente-revisor-persona.md) (operación → identidad, lista de firmantes en `.taskcode/`), [ADR F1-7](F1-7-verificacion-ci-perfiles-cache-runners.md) (perfil por evento), [ADR F1-8](F1-8-seguridad-modelo-de-amenazas.md) (`carril` en `pull_request_target`, D1b) y [ADR F1-9](F1-9-formato-de-tarea-plan-y-especificacion.md) (campo `carril`, modo congelado en la tarea)
- Decide: Carlos, con la consigna «la opción recomendada» (D0a, D1a, D2a, D3a, D4a, D5a, D6a y D7a del plan de TASK-011) y «no trocear»

## Contexto

P7 pide carriles proporcionales: «errata → PR directo; fix → tarea + PR;
funcionalidad → plan aprobado + PR; riesgo alto → diseño con todos los roles y
dos aprobaciones», y que el modo automático (RF-21: 88–97 % del uso real)
encadene las fases y pare «solo en las puertas del carril». RR-07 pide pocas
puertas humanas, todas con valor; RR-08, que las garantías las imponga la
plataforma y no las instrucciones al agente.

Lo que ya está decidido y este ADR no cambia:

- **F1-5**: las puertas por carril (tabla de seis filas: errata, fix,
  funcionalidad, riesgo alto, release y hotfix), el check `carril` que las
  impone, el **carril efectivo** como el mayor entre el declarado y un mínimo
  por ficheros, y errata solo si todo el PR está bajo `docs/`. Dejó a F1-10 el
  resto de la regla de derivación, los renombrados, la política en
  configuración, la ceremonia de la errata, quién lanza una release y el
  riesgo alto con varias personas.
- **F1-6**: `merge-pr` y `tag-anotado` son de la persona; ninguna IA aprueba
  ni mergea; la lista de firmantes vive en `.taskcode/` bajo CODEOWNERS (D6a).
- **F1-8**: `carril` corre en `pull_request_target` (mitigación parcial, D1b).
- **F1-9**: la tarea gana el campo `carril` (`null` hasta el plan); el modo se
  congela en `plan`.

El modo automático de la 0.16 (`skills/task-workflow/avance.md` del plugin
0.16.1) ya tiene la forma que hace falta: `taskctl siguiente --json` devuelve
`fase`, `comando`, `accion` (`detener`, `preguntar` o `continuar`) y
`motivo`; el modo se congela en la tarea al planificar; en `automatico` solo
pregunta en sus guardas (aprobar sin modo congelado, cerrar hotfix o release,
cerrar si lo que se mergearía no coincide con lo revisado, otra ronda pasado
`tope_rondas` y el veredicto de la segunda opinión). Lo que no tiene es una
política por carril: la 0.16 no conoce carriles.

Preguntas que responde: qué valores tiene `carril`, qué exige cada uno, de
dónde sale su política y quién puede cambiarla, cómo se deriva el carril
mínimo de los ficheros, qué se congela en la tarea, dónde para el modo
automático en cualquier configuración, y las once preguntas que F1-3, F1-4,
F1-5, F1-6, F1-8 y F1-9 le dejaron (tabla «Preguntas heredadas»).

## Alternativas

| | Descripción | Veredicto |
|---|---|---|
| **Tabla única de política** | Una fila por carril con roles, plan, revisoras, perfil, checks, aprobaciones, paradas y lote; la leen `siguiente`, `plan`, el check `carril`, `lote` y `doctor` con la misma función pura. | **Elegida** (D0a: no se trocea para no tener dos fuentes). |
| Política repartida por comando | Cada comando decide lo suyo (el orquestador sus paradas, `carril` sus puertas). | Descartada: dos fuentes que pueden divergir; la PoC B compara una sola tabla con F1-5 y F1-7. |
| **D1a** Cuatro carriles ordenados + tipo | `errata < fix < funcionalidad < riesgo-alto`; release y hotfix los da `tipo` (base `main`). | **Elegida**. Reinterpreta las seis filas de F1-5 sin cambiar sus puertas. |
| D1b Seis valores de `carril` | release y hotfix también como carril. | Descartada: no tienen orden con los otros cuatro (otra base) y el carril efectivo «el mayor» dejaría de estar definido. |
| **D2a** Funcionalidad nunca por ficheros | Solo se declara en `plan`; por ficheros salen errata, fix o riesgo alto. | **Elegida**. |
| D2b Funcionalidad por rutas o tamaño | `src/` nuevo o un diff grande → funcionalidad. | Descartada: el diff no dice si un cambio es funcionalidad (en A, 134 de 160 tareas de ToolsaWeb tocan código y casi todas son `feature`; con D2b todas pedirían plan aprobado por ficheros, también los fix). |
| **Configuración que solo endurece** | El suelo va en el binario; la configuración sube un carril, añade rutas o paradas; lo que baja lo rechaza el esquema. | **Elegida**. |
| Configuración libre | La política entera en `.taskcode/config.yml`. | Descartada: CODEOWNERS obliga a aprobarla, pero con una sola persona (RN-06) un cambio que baja la puerta pasa con la misma aprobación que ya da el PR. |
| **Política leída de la base** | `origin/develop:.taskcode/config.yml` en el CLI; la base por la API en `carril`. | **Elegida**. Descartado leerla del worktree de la tarea (el agente la escribe). |
| Congelar la política en `plan` | Como el modo. | Descartada: una tarea larga conservaría una política más laxa; como la configuración solo endurece, la vigente nunca es más blanda. Sí se congelan el modo y el carril declarado. |
| **`accion` en `siguiente --json`** | Se extiende el contrato `fase/accion/motivo` de la 0.16. | **Elegida**. Descartado un comando nuevo para lo mismo. |
| **D3a** La release la lanza la persona | `merge-pr` y `tag-anotado` de persona (F1-6); al agente no se le impide crear un tag `v*`, `doctor` lo detecta y la publicación solo publica tags firmados por una persona de la lista. | **Elegida**. Descartada D3b, una regla `creation` en tags sin bypass: impediría también a la persona (F1-6). |
| **D4a** Merge del agente: límite escrito | `finish` exige identidad de persona; no se añade nada más. | **Elegida**. Descartado el fork o la cola en organización como única vía (F1-6, D3b): no se puede exigir a los pilotos de una persona. |
| **D5a** Errata y hotfix con tarea propia | `taskctl errata "<título>"` crea tarea, rama y PR sin plan. | **Elegida**. Descartada la errata sin tarea: F1-5 hace fallar todo PR sin tarea. |
| **D6a** Riesgo alto con varias personas | Con 2 o más en la lista, el firmante del plan distinto del aprobador del PR; con 1, D4a de F1-5. Sin clave nueva. | **Elegida**. Descartadas dos aprobaciones en el PR (exigiría `required_approving_review_count: 2` en todo `develop`, porque el ruleset no filtra por carril). |
| **D7a** Tarea que toca `.github/workflows/` con una persona | Empuja la persona y después el agente hace un commit propio encima, para que el último push sea suyo. | **Elegida** (no verificado). Descartadas dar `workflows` al agente (F1-8) y una segunda persona (no existe con RN-06). |

## Pruebas de concepto

Guion `poc-f1-10.py` y su salida completa `poc-f1-10.log`, en la carpeta
`poc/` de TASK-011 (sin enlace: la carpeta de la tarea cambia de estado al
cerrarse). Ejecutado con Python 3.12.10 en modo aislado (`python -I`) y Git
2.56, con la carpeta de trabajo en
`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-011\poc\` (borrada al
terminar). **Solo lectura**: ToolsaWeb, OpenDockTrading y TaskCodeTwo se leen
con `git log`, `git show` y `git archive` de commits fijos (los mismos que la
PoC de F1-9; el archivo se lee en memoria, no se extrae ni se ejecuta nada);
no hay llamadas a GitHub. Dos ejecuciones seguidas dan el mismo log.

```bash
cd sources/Personal
python -I TaskCodeTwo/tareas/<estado>/TASK-011/poc/poc-f1-10.py \
  "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-011/poc" \
  TaskCodeTwo/docs/adr/F1-10-carriles-y-modo-automatico.md . > poc-f1-10.log
```

El guion implementa las funciones de la decisión 1 (`carril_minimo`,
`carril_efectivo`, `politica`, `accion`, `validar_config`) y **lee la tabla
de política de este fichero** entre los marcadores `tabla-politica`: lo que C
y D simulan sale de la misma tabla que B compara.

### A — Carril mínimo sobre el historial real

Merges de primer padre en `develop` con rama de tarea; por cada uno,
`git log -M --name-status M^1..M^2`. Los cambios cuyas rutas están todas bajo
`tareas/` son estado de la 0.16 (en la 2.0 van a `taskcode/tareas`, F1-1) y
no cuentan.

```text
--- ToolsaWeb (commit 93c9618ac)
merges de primer padre con rama de tarea: 160 (por prefijo: feature=155, fix=5; sin ID de tarea: 1)
cambios solo de tareas/ (estado de la 0.16, fuera del PR en la 2.0) excluidos: 6230
carril minimo: errata=22 ; fix=134 ; funcionalidad=0 ; riesgo-alto=4
  riesgo-alto (4): TASK-297, TASK-298, TASK-344, TASK-403 (.taskcode/config.yml)
renombrados fuera de tareas/: 6 ; que cambian de clase con los dos nombres: 0
--- OpenDockTrading (commit fab8d7e05)
merges de primer padre con rama de tarea: 43 (por prefijo: feature=43; sin ID de tarea: 0)
carril minimo: errata=1 ; fix=41 ; funcionalidad=0 ; riesgo-alto=1
  riesgo-alto (1): TASK-014 (.taskcode/config.yml)
renombrados fuera de tareas/: 0 ; que cambian de clase con los dos nombres: 0
-- casos sinteticos
  renombrado .github/ -> docs/             -> riesgo-alto  ok
  renombrado docs/ -> .taskcode/           -> riesgo-alto  ok
  renombrado src/ -> docs/                 -> fix          ok
  docs/CODEOWNERS                          -> riesgo-alto  ok
  .GitHub/ en mayusculas                   -> riesgo-alto  ok
  Docs/ en mayusculas no es errata         -> fix          ok
  copia desde .github/                     -> riesgo-alto  ok
  rutas_carril sube src/auth/**            -> riesgo-alto  ok
  .taskcode exacto (fichero o enlace)      -> riesgo-alto  ok
  .githubx/ no es el suelo                 -> fix          ok
  PR sin ficheros                          -> ERROR        ok
  ./.github/ se normaliza                  -> riesgo-alto  ok
  (y 11 casos más: 23/23)
```

Lectura:

- **Por ficheros nunca sale funcionalidad** (0 de 203), y la mayoría de las
  tareas sale fix (134 + 41): el carril de una tarea con código lo pone su
  plan, no sus ficheros (D2a).
- **23 tareas solo tocan `docs/`** (22 de ToolsaWeb, con la serie de
  documentación de TASK-173 a TASK-191 (14 de ellas), y TASK-067 de
  OpenDockTrading). Hoy
  son `feature` con plan y aprobación; con la 2.0 podían haber ido como
  errata.
- **Los 5 riesgo alto salen de `.taskcode/config.yml`**; ningún piloto tiene
  `.github/` (coherente con la PoC D de F1-8: no tienen workflows). Un
  cambio de configuración pide hoy el mismo plan que cualquier tarea; con la
  2.0, todas las paradas de riesgo alto.
- **Los renombrados reales no cruzan de clase** (6 en ToolsaWeb, todos dentro
  de la misma clase); el caso que importa (`.github/` → `docs/`) solo existe
  en los sintéticos, y sale riesgo alto porque se miran los dos nombres.

### B — La tabla de política frente a F1-5 y F1-7

Por cada fila: base, checks, número de aprobaciones y si exige el plan, contra
la tabla `tabla-carriles` de F1-5; el perfil, contra la tabla «Perfil según el
evento» de F1-7; y la coherencia interna (parada `approve` si y solo si hay
plan aprobado; `merge-pr` en todas; `tag` solo en release y hotfix; ninguna
parada fuera de las fijas; riesgo alto, release y hotfix fuera del lote;
`revision-ia` si hay revisoras; uno o todos los roles según F4-1 del plan; y
cada carril al menos tan estricto como el anterior).

```text
perfiles de F1-7: {'develop': 'completa', 'release': 'completa y pesada', 'hotfix': 'completa'}
errata         base=develop  roles>=0 plan=no aprob=1 perfil='completa' checks=carril,revision-ia,verificacion paradas=merge-pr lote=si
fix            base=develop  roles>=1 plan=no aprob=1 perfil='completa' checks=carril,revision-ia,verificacion paradas=merge-pr lote=si
funcionalidad  base=develop  roles>=1 plan=si aprob=2 perfil='completa' checks=carril,revision-ia,verificacion paradas=approve,merge-pr lote=si
riesgo-alto    base=develop  roles>=4 plan=si aprob=2 perfil='completa' checks=carril,revision-ia,verificacion paradas=approve,merge-pr lote=no
release        base=main     roles>=0 plan=no aprob=1 perfil='completa y pesada' checks=carril,verificacion paradas=merge-pr,tag lote=no
hotfix         base=main     roles>=1 plan=no aprob=1 perfil='completa' checks=carril,revision-ia,verificacion paradas=merge-pr,tag lote=no
-- desvios inyectados (la comparacion tiene que detectarlos)
  errata sin revision-ia               -> detectado (2 fallos)
  funcionalidad sin parada approve     -> detectado (1 fallos)
  riesgo-alto con 1 aprobacion         -> detectado (2 fallos)
  release sin parada tag               -> detectado (1 fallos)
  fix con plan aprobado                -> detectado (2 fallos)
  hotfix con base develop              -> detectado (1 fallos)
  riesgo-alto en lote                  -> detectado (1 fallos)
  fix con perfil rapida                -> detectado (1 fallos)
  errata sin parada merge-pr           -> detectado (1 fallos)
  fila errata quitada                  -> detectado (2 fallos)
  riesgo-alto con 1 rol                -> detectado (1 fallos)
  parada inventada en fix (review)     -> detectado (2 fallos)
RESULTADO B filas=6 fallos=0 ; desvios detectados=12/12
```

Es una comprobación de coherencia de los documentos, no del comportamiento de
GitHub.

### C — `accion` sobre las transiciones reales de la 0.16

Para cada tarea terminada de los tres repos, sus filas de `## Transiciones`:

- **hoy** (0.16): en `manual` y `semiautomatico`, cada transición salvo la
  primera es una parada; en `automatico`, el `approve` con
  `decidido_por: persona`, las `pausa` y las rondas por encima de
  `tope_rondas` (2 en los tres repos).
- **nuevo**: la cadena `plan → [approve] → start → review × N → merge-pr →
  [tag]` recorrida con `accion()` en el modo congelado de la tarea, con
  carril declarado `fix` si el tipo es `fix` y `funcionalidad` si no (la 0.16
  no tiene carril) y carril efectivo el mayor entre ese y el mínimo de A.

```text
--- ToolsaWeb  tareas=160  carril efectivo: fix=5, funcionalidad=151, riesgo-alto=4
filas en automatico: finish=156 ; approve decidido_por automatico=114 ; approve decidido_por persona=43
paradas humanas: hoy=136 (0.85/tarea) ; nuevo=403 (2.52/tarea) ; cota inferior con declarado = minimo por ficheros=244 (1.52/tarea)
paradas nuevas por tipo: approve=159, merge-pr=160, modo-manual=11, subida=4, tope=69
--- OpenDockTrading  tareas=43  carril efectivo: funcionalidad=42, riesgo-alto=1
filas en automatico: finish=40 ; approve decidido_por automatico=5 ; approve decidido_por persona=35
paradas humanas: hoy=91 (2.12/tarea) ; nuevo=137 (3.19/tarea) ; cota inferior con declarado = minimo por ficheros=93 (2.16/tarea)
paradas nuevas por tipo: approve=44, merge-pr=43, modo-semiautomatico=9, subida=1, tope=40
--- TaskCodeTwo  tareas=9  carril efectivo: funcionalidad=9
filas en automatico: finish=9 ; approve decidido_por automatico=0 ; approve decidido_por persona=9
paradas humanas: hoy=11 (1.22/tarea) ; nuevo=20 (2.22/tarea) ; cota inferior con declarado = minimo por ficheros=11 (1.22/tarea)
paradas nuevas por tipo: approve=9, merge-pr=9, tope=2
-- tareas sinteticas de tipo release y hotfix (la 0.16 no tiene ninguna terminada)
  release  paradas=2 [('merge-pr', 1), ('tag', 1)] politica=['merge-pr', 'tag'] -> ok
  hotfix   paradas=2 [('merge-pr', 1), ('tag', 1)] politica=['merge-pr', 'tag'] -> ok
-- tipo de la tarea frente a la base real del PR (revision 1, IMP-1)
   (cada resultado: fila, plan, aprobaciones, paradas; checks = carril, revision-ia, verificacion en todos)
  hotfix con .github/ contra develop     efectivo=riesgo-alto  -> riesgo-alto, True, 2, [approve, merge-pr] ok
  hotfix con .github/ contra main        efectivo=riesgo-alto  -> hotfix+riesgo-alto, True, 2, [approve, merge-pr, tag] ok
  hotfix solo src/ contra main           efectivo=fix          -> hotfix+fix, False, 1, [merge-pr, tag] ok
  release con .github/ contra develop    efectivo=riesgo-alto  -> riesgo-alto, True, 2, [approve, merge-pr] ok
  release con .taskcode/ contra main     efectivo=riesgo-alto  -> release+riesgo-alto, True, 2, [approve, merge-pr, tag] ok
  release solo docs/ contra main         efectivo=errata       -> release+errata, False, 1, [merge-pr, tag] ok
  hotfix contra main desde feature/*     efectivo=fix          -> ERROR ok
  feature contra main                    efectivo=funcionalidad -> ERROR ok
  M1 release contra main desde feature/* efectivo=funcionalidad -> ERROR ok
  M3 hotfix funcionalidad contra main    efectivo=funcionalidad -> hotfix+funcionalidad, True, 2, [approve, merge-pr, tag] ok
  union: release funcionalidad desde develop efectivo=funcionalidad -> release+funcionalidad, True, 2, [approve, merge-pr, tag] ok
-- barrido: approve, merge-pr y tag en todo carril, tipo, base, modo y configuracion valida de D
  combinaciones=1260 ; continuar=0 ; tipo y base que fallan cerrado (sin fila)=160
  filas de main por debajo del carril efectivo: 0/8
RESULTADO C ... ; tipo y base 11/11 ; continuar sobre approve/merge-pr/tag=0 ; paradas distintas de la politica=0
```

Lectura:

- **Nunca hay `continuar` sobre `approve`, `merge-pr` o `tag`**, ni en las
  tareas reales ni en las 1260 combinaciones de carril, tipo, base, modo y
  configuración válida; y en modo automático las paradas fijas que ve cada
  tarea son exactamente las de su fila de la tabla.
- **El tipo no baja la puerta** (revisiones 1 y 2, IMP-1): `tipo: release`
  solo elige su fila en un PR `develop` → `main`, y `tipo: hotfix` en uno
  `hotfix/*` → `main`; cualquier otro PR contra `main` falla cerrado (M1) y
  contra `develop` manda el carril efectivo. Además, una fila de `main`
  nunca baja de su carril efectivo: sus puertas son la unión de las de la
  fila y las del carril (M3: un hotfix declarado funcionalidad lleva plan,
  `approve` y dos aprobaciones; 0 de 8 filas de `main` por debajo de su
  carril; 11/11 casos).
- **Las paradas por tarea suben** (0,85 → 2,52 en ToolsaWeb). No las añade
  este ADR: son las dos puertas de F1-5 y F1-6 que la 0.16 no tiene. Hoy, en
  automático, el agente mergea en local (`finish=156` en ToolsaWeb, sin
  parada) y en 114 tareas el `approve` lo hizo el propio flujo
  (`decidido_por: automatico`, H5; 43 los decidió una persona). Recuentos
  sobre las tareas terminadas del commit fijo, tal como los imprime la PoC.
  La 2.0 suma `merge-pr` de persona en toda tarea y el `approve` firmado en
  funcionalidad y riesgo alto.
- **Cota inferior**: si cada tarea se hubiera declarado con su carril mínimo
  por ficheros (solo `docs/` → errata; código → fix; sin `approve` salvo
  riesgo alto), las paradas serían 1,52 por tarea en ToolsaWeb, 2,16 en
  OpenDockTrading y 1,22 en TaskCodeTwo: la diferencia con 2,52 / 3,19 /
  2,22 es el `approve` de las tareas que se declaren funcionalidad. El lote
  (F1-16) puede además agrupar errata, fix y funcionalidad con plan
  aprobado en un solo `merge-pr` al final (RR-18). Ver «Divergencia con el
  plan», 3.

### D — Configuración que solo endurece

```text
-- aceptadas (endurecen o no cambian nada)
  V1 {"politica_carriles": {"fix": {"lote": false, "plan_aprobado": true, "roles_diseno": 2}}} -> aceptada, no baja nada
  V2 {"politica_carriles": {"funcionalidad": {"paradas_extra": ["start", "review"], "perfil": "completa y pesada"}},
      "rutas_carril": {"riesgo-alto": ["src/auth/**"]}} -> aceptada, no baja nada
  (V0, V3 y V4 igual)
  efecto de V1 en fix: plan=True paradas=['approve', 'merge-pr'] roles>=2 lote=False
-- rechazadas (bajan un carril, quitan una parada o no se reconocen: falla cerrado)
  funcionalidad sin plan aprobado        -> rechazada: ...plan_aprobado: solo admite true
  riesgo-alto con 1 aprobacion           -> rechazada: ...aprobaciones: clave no admitida
  quitar merge-pr de fix                 -> rechazada: ...paradas_quitar: clave no admitida
  rutas que dan errata                   -> rechazada: rutas_carril.errata: solo fix o riesgo-alto
  rutas que dan funcionalidad            -> rechazada: rutas_carril.funcionalidad: solo fix o riesgo-alto
  glob negado en riesgo-alto             -> rechazada: glob no admitido '!docs/**'
  riesgo-alto entra en lote              -> rechazada: ...lote: solo admite false
  fix con perfil rapida                  -> rechazada: 'rapida' por debajo de 'completa'
  politica de release (es un tipo)       -> rechazada: carril desconocido
  paradas_extra con una lista dentro     -> rechazada: ...paradas_extra: solo fases de ['review', 'start']
  (y 7 más)
-- PR que cambia .taskcode/config.yml para bajar la politica
  carril minimo=riesgo-alto (.taskcode/config.yml) ; config de la rama valida=False ;
  politica con la de la base: plan=True paradas=['approve', 'merge-pr']
RESULTADO D aceptadas=5/5 ; rechazadas=17/17
```

### Divergencia con el plan

1. **`politica` recibe también `tipo`.** El plan la firma como
   `politica(carril, config)`; con D1a, release y hotfix no son carriles sino
   tipos, y su fila se elige por `tipo`. Firma final en la decisión 1.
2. **En la PoC, `accion` recibe además la configuración y la tabla** para
   calcular la política del carril efectivo cuando sube; en el núcleo la
   calcula quien llama y `accion` recibe la del efectivo (decisión 1).
3. **Las paradas humanas por tarea suben** frente a hoy (C: 0,85 → 2,52 en
   ToolsaWeb, 2,12 → 3,19 en OpenDockTrading, 1,22 → 2,22 en TaskCodeTwo;
   cota inferior, con cada tarea declarada con su carril mínimo: 1,52, 2,16
   y 1,22). El plan esperaba medirlo, no este resultado. Va contra la letra de RR-07
   («pocas puertas humanas»), pero las dos paradas que lo explican
   (`merge-pr` y `approve` firmado) son puertas ya aceptadas en F1-5 y F1-6
   que la 0.16 se salta (H5, H6). Lo contienen la errata y el fix sin plan
   aprobado y el lote (F1-16); si no basta, se revisa en F1-16, no aquí.
4. **La cifra «hoy» es aproximada**: no cuenta las preguntas que la 0.16 hace
   dentro de `plan`, ni un cierre en la última ronda sin ronda extra (no
   deja fila), y `decidido_por` no es fiable (H5).
5. **A excluye `tareas/`**: 6 230 cambios en ToolsaWeb y 1 674 en
   OpenDockTrading son estado de la 0.16, que en la 2.0 no va en el PR
   (F1-1). Sin esa exclusión ninguna tarea saldría errata.
6. **B compara también los perfiles con F1-7 y los roles con F4-1** del plan
   (el plan solo pedía F1-5): un perfil o un número de roles cambiado en la
   tabla también se detecta.
7. **`complejidad` no entra en el carril** (F1-9 la lista como «entrada del
   carril»): decide cuántos roles lleva una funcionalidad, no el carril, que
   declara el plan (D2a) y sube por ficheros. Ver «Divergencias con F1-5,
   F1-6 y F1-9».
8. **Revisión 1** (Carlos, con la consigna «la opción recomendada»):
   **IMP-1**, el `tipo` de `tarea.md` (no confiable) elegía la fila de
   release o hotfix en cualquier PR y podía quitar el plan firmado y la
   segunda aprobación; ahora solo vale con la base real `main` (y la rama
   `hotfix/*` en hotfix), y el mínimo por ficheros también se aplica a
   release y hotfix (decisiones 1 y 4; PoC C, 8/8). **MEN-1**, `paradas_extra`
   con un valor que no es texto se rechaza con mensaje (PoC D). **MEN-2**, el
   suelo compara el primer componente de la ruta (`.github` y `.taskcode`
   exactos también). **MEN-3**, la PoC imprime los recuentos de `finish` y
   `approve` que cita el ADR y la cota inferior. **MEN-4**, el número de
   roles lo fija `plan`, no `politica` (decisión 1).
9. **Revisión 2** (Carlos, con la consigna «la opción recomendada»):
   **IMP-1** seguía abierto para `release`, que no exigía rama de origen (un
   PR `feature/*` → `main` con `tipo: release` quitaba plan, `approve` y
   `revision-ia`). Ahora la fila release solo vale desde `develop` y, además,
   una fila de `main` nunca baja del carril efectivo (unión de puertas;
   decisión 1; PoC C, M1, M3 y la unión, 11/11). Nota menor: las rutas se
   normalizan (`./.github/x` sale riesgo alto; antes, fix).

### No verificado

- **El comportamiento en GitHub**: que `GET /pulls/{n}/files` dé
  `previous_filename` en cada renombrado y que corte la lista a partir de
  3000 ficheros; que `carril` en `pull_request_target` lea la configuración
  de la base y no la del PR. Ningún PR real se ha leído: criterio de F2-6.
- **D7a**: que GitHub admita que la App del agente empuje un commit propio
  encima de un cambio en `.github/workflows/` sin el permiso `workflows`
  (el commit del agente no toca workflows, pero la rama sí los cambia).
  F2-6.
- **El carril declarado de las tareas de la 0.16** en C es una suposición
  (`fix` si el tipo es `fix`, `funcionalidad` si no): la 0.16 no tiene carril.
- **Las reglas de negocio de la tabla** (roles por carril, qué entra en un
  lote, revisoras): sin rol de dominio en el plan; van como propuesta que
  F1-15 y F1-16 pueden endurecer.
- **`taskctl doctor`** no se ejecutó en esta implementación (la tarea pidió
  no lanzar `taskctl`); queda para el cierre.

## Cómo se ve en GitHub

Conocimiento general de GitHub, **no verificado** (ningún check `carril`
existe aún):

| Dónde | Qué se ve |
|---|---|
| Check `carril` | `carril: funcionalidad (declarado funcionalidad; mínimo fix por src/x.rs)` y lo que falta: «falta la aprobación del plan», o `carril: riesgo-alto (declarado fix; mínimo riesgo-alto por .taskcode/config.yml): vuelve a plan` |
| Check `carril` con configuración inválida en la base | en rojo: «política no válida: `politica_carriles.fix.plan_aprobado` solo admite true» |
| PR que cambia `.taskcode/config.yml` | carril mínimo riesgo alto; «Code owner review required» (F1-5) |
| Terminal (`siguiente --json`) | `{"fase": "merge-pr", "accion": "detener", "motivo": "merge-pr solo lo lanza una persona (finish)", "carril": "fix", "carril_minimo": "fix", "paradas": ["merge-pr"]}` |

## Decisión

1. **Una sola tabla de política, una función pura.** La tabla de abajo es la
   fuente única. El núcleo, sin E/S:

   ```text
   carrilMinimo(cambios: Cambio[], reglas: RutasCarril) -> {carril: errata|fix|riesgo-alto, ruta} | ErrorCarril
     Cambio = {estado: A|M|D|R|C|T, rutas: [ruta] | [ruta_vieja, ruta_nueva]}
   carrilEfectivo(declarado: Carril, minimo: Carril) -> Carril          # el mayor; solo sube
   validarConfig(config) -> Error[]                                      # vacía = válida
   politica(carril: Carril, tipo: Tipo, base: Rama, rama: Rama, config: ConfigValidada) -> Politica | ErrorCarril
     Politica = {base, roles_min, plan, revisoras, perfil, checks, aprobaciones, paradas, lote}
   accion(fase: Fase, tarea: {declarado, efectivo, tipo, base, rama, ronda, tope}, politica: Politica, modo: Modo)
     -> {accion: detener|preguntar|continuar, motivo}
   ```

   `Carril` = `errata | fix | funcionalidad | riesgo-alto` (ordenados);
   `Tipo` ∈ los de la tarea. **El tipo es texto de `tarea.md` (no
   confiable) y solo elige fila con la base y la rama reales del PR** (las
   de la API en `carril`; las del PR que abre `review` en local):
   `release` solo en un PR `develop` → `main` (D3a), y `hotfix` solo en uno
   `hotfix/*` → `main`. En un PR contra `develop` el tipo se ignora y la
   fila es la del carril efectivo; cualquier otro PR contra `main` (un
   `release` desde `feature/*`, un `hotfix` desde otra rama, otro tipo) no
   tiene fila y `carril` falla cerrado. **Una fila de `main` nunca baja de
   lo que exige el carril efectivo**: sus puertas son la unión de las de la
   fila y las de la fila del carril (plan y `approve` si el carril los
   exige, el mayor número de aprobaciones y de roles, la unión de checks y
   de paradas, y fuera del lote si cualquiera de las dos lo está). Una
   release cuyo diff toca `.taskcode/` lleva así lo de riesgo alto (decisión
   4), y un hotfix declarado funcionalidad, su plan firmado. `Fase` = `plan | approve | start | review | merge-pr | tag`;
   `politica` recibe la política del carril **efectivo**. La usan
   `siguiente` y el orquestador (F4-4), `plan` (F4-1, F4-6), el check
   `carril` (F6), `lote` (F1-16) y `doctor` (F3-8). `siguiente --json`
   conserva `fase`, `comando`, `accion` y `motivo` de la 0.16 y añade
   `carril`, `carril_minimo` y `paradas`.

   <!-- tabla-politica:inicio -->
   | Carril | Base del PR | Roles de diseño | Plan aprobado | Revisoras | Perfil de `verificacion` | Checks obligatorios | Aprobaciones de persona | Paradas del automático | Lote |
   |---|---|---|---|---|---|---|---|---|---|
   | `errata` | `develop` | 0 (sin plan) | no | genérica, perfil rápido (D5a de F1-5) | completa | `verificacion`, `revision-ia`, `carril` | 1 (PR) | `merge-pr` | sí |
   | `fix` | `develop` | 1 (arquitectura) | no | por dominio del diff | completa | `verificacion`, `revision-ia`, `carril` | 1 (PR) | `merge-pr` | sí |
   | `funcionalidad` | `develop` | 1 a 4, según la complejidad | sí, `approve` firmado | por dominio del diff | completa | `verificacion`, `revision-ia`, `carril` | 2 (plan y PR) | `approve`, `merge-pr` | sí, con el plan aprobado antes de lanzar el lote |
   | `riesgo-alto` | `develop` | todos (rendimiento si la etiqueta lo pide) | sí, `approve` firmado | por dominio del diff | completa | `verificacion`, `revision-ia`, `carril` | 2 (plan y PR; D6) | `approve`, `merge-pr` | no |
   | `release` (tipo) | `main`, solo desde `develop` (base y rama reales del PR); puertas ∪ las del carril efectivo | 0 (sin plan) | no | ninguna | completa y pesada | `verificacion`, `carril` | 1 (PR) | `merge-pr`, `tag` | no |
   | `hotfix` (tipo) | `main`, solo desde `hotfix/*` (base y rama reales del PR); puertas ∪ las del carril efectivo | 1 (arquitectura) | no | por dominio del diff | completa | `verificacion`, `revision-ia`, `carril` | 1 (PR) | `merge-pr`, `tag` | no |
   <!-- tabla-politica:fin -->

   Columnas: «Checks» y «Aprobaciones» son las de F1-5 (no cambian); el
   «Perfil» es el de F1-7 por la base del PR; «Roles» sigue F4-1 del plan
   («uno en fix, todos en riesgo alto») y es el **mínimo** que devuelve
   `politica` (`roles_min`): el número exacto de roles lo fija `plan` (F4-1)
   según la complejidad, fuera de `politica` y nunca por debajo de ese
   mínimo (en funcionalidad, de 1 a 4, como hoy en la 0.16; divergencia 7); «Paradas» son las fijas de la decisión
   6 que tocan a la fila (además, en todas: tope de rondas y subida de
   carril); «Lote» dice si la tarea puede ir en un lote de F1-16. La PoC B
   compara esta tabla con F1-5 y F1-7.

2. **La configuración solo endurece.** El suelo (esta tabla, y que errata
   exige todo bajo `docs/` y que `.github/`, `.taskcode/` y cualquier
   `CODEOWNERS` suben a riesgo alto) va en el binario. Dos claves nuevas en
   `.taskcode/config.yml` (bajo CODEOWNERS, F1-6 D6a), que suman dos al tope
   de 12 de F2-7:

   | Clave | Admite | Rechaza |
   |---|---|---|
   | `politica_carriles.<carril>` (solo los cuatro carriles) | `roles_diseno` (≥ suelo, ≤ 5), `plan_aprobado: true`, `paradas_extra` (`start`, `review`), `revisoras_extra`, `lote: false`, `perfil` (≥ suelo) | cualquier otra clave (`aprobaciones`, `paradas`, `paradas_quitar`, `modo`…), un valor por debajo del suelo, `release` o `hotfix` como carril |
   | `rutas_carril.<carril>` (solo `fix` y `riesgo-alto`) | lista no vacía de globs (`**`, `*`, `?`) | `errata` (no se da por rutas), `funcionalidad` (nunca por ficheros, D2a), globs con `!`, `/` inicial o `..` |

   Una clave que baja algo o que no se reconoce la rechaza el esquema y **se
   falla cerrado**: el CLI no avanza ninguna fase, el check `carril` sale en
   rojo y `doctor` dice la clave. Un glob mal escrito solo puede subir
   carriles (más paradas, nunca menos). Con `plan_aprobado: true` en un
   carril que no lo exige, la fila gana la parada `approve` y la segunda
   aprobación.

3. **Confianza de lo que se lee.** El CLI lee la política de
   `origin/develop:.taskcode/config.yml` tras el `fetch`, nunca del worktree
   de la tarea; el check `carril` la lee de la base por la API desde
   `pull_request_target` (F1-8 D1b, parcial). El carril declarado en
   `tarea.md` es texto no confiable (F1-8, `A10`; F1-9): solo sirve para
   subir; el `tipo`, igual, solo elige la fila de release (desde `develop`)
   o de hotfix (desde `hotfix/*`) con la base real `main`, y nunca por
   debajo del carril efectivo (decisión 1). Las paradas locales del orquestador son de usabilidad (evitar
   trabajo que la puerta rechazará); quien impone es `carril`, con la misma
   función. Un PR que cambia `.taskcode/` sale riesgo alto por el suelo, y su
   configuración nueva no se aplica hasta que esté en la base (PoC D).
4. **Derivación del carril mínimo.** Entrada: los ficheros del PR por la API
   (en local, `git diff -M --name-status` contra la base); en cada
   renombrado o copia, **los dos nombres** (`previous_filename`). Por cada
   ruta, normalizada antes (sin `./` ni componentes vacíos: `./.github/x`
   es `.github/x`): primer componente `.github` o `.taskcode` (también la ruta exacta:
   un fichero, enlace o submódulo con ese nombre) o un `CODEOWNERS` en
   cualquier carpeta (comparando sin mayúsculas) → riesgo alto; un glob de
   `rutas_carril.riesgo-alto` → riesgo alto; uno de `rutas_carril.fix` → fix;
   bajo `docs/` exacto → errata; el resto → fix. El carril mínimo es el mayor
   de todas las rutas. **Nunca sale funcionalidad** (D2a). Una lista vacía,
   una ruta con `..` o una lista que no se puede leer entera (la API corta
   a partir de 3000 ficheros, no verificado) → `carril` falla, no decide. El
   mínimo se aplica a todos los PR, también a los de release y hotfix contra
   `main` (decisión 1: unión con las puertas del carril efectivo). Un
   cambio solo de `docs/` que tenga que ir directo a `main` no tiene fila
   de errata allí: va como hotfix, desde `hotfix/*`. El
   tamaño del diff no cuenta: un PR enorme de solo `docs/` sigue siendo
   errata (lo revisa `revision-ia`; F1-17 usa el tamaño para estimar).
5. **Qué congela la tarea.** En `plan` se congelan el **modo** (como en la
   0.16) y el **carril declarado** (campo `carril` de F1-9; en errata, lo
   escribe `taskctl errata`). La **política no se congela**: se evalúa con la
   configuración vigente en la base, que solo puede ser más estricta. Si en
   `review` el carril efectivo supera al declarado, `accion` devuelve
   `detener`; si el nuevo exige plan (o más roles), con el motivo «vuelve a
   plan», y la tarea repite `plan` con el carril nuevo y, si toca, `approve`
   firmado.
6. **Paradas fijas del modo automático, en cualquier configuración y modo**:

   | Parada | Carril o tipo | `accion` | Por qué |
   |---|---|---|---|
   | `approve` firmado | funcionalidad, riesgo alto (y donde la configuración añada `plan_aprobado`) | `detener` | la firma es de la persona, con TTY (F1-5, F1-6) |
   | `merge-pr` | todos | `detener` | ninguna IA mergea (F1-6); `finish` exige identidad de persona (D4a) |
   | tag de release y de hotfix | `release`, `hotfix` | `detener` | `tag-anotado` es de la persona (D3a) |
   | tope de rondas | todos | `preguntar` | como en la 0.16: `cerrar`, `ronda-extra` o `volver-a-diseno` (RF-33) |
   | subida de carril | todos | `detener` | decisión 5 |

   `accion` resuelve `approve`, `merge-pr` y `tag` **antes** de mirar la
   política o el modo, así que ninguna configuración las convierte en
   `continuar` (PoC C: 1260 combinaciones, 0 `continuar`). Fuera de esas y de
   las `paradas_extra`, en `automatico` todo se encadena; `manual` devuelve
   `detener` en cada fase y `semiautomatico`, `preguntar`. Las guardas del CLI
   de la 0.16 que no son de política (lo que se mergearía no coincide con lo
   revisado, revisor independiente) siguen en `review` y `finish`.
7. **Lo que cierra además** (D1–D7 del plan):
   - **D1a** `carril` ∈ `errata < fix < funcionalidad < riesgo-alto`;
     release y hotfix, por `tipo`, solo con base real `main` y su rama de
     origen (`develop`, `hotfix/*`), sin bajar del carril efectivo
     (decisión 1).
   - **D2a** funcionalidad solo se declara en `plan`.
   - **D3a** la release la lanza la persona: `release` abre el PR
     `develop`→`main` con la identidad del agente (F1-6), y el `merge-pr` y
     el `tag-anotado` son de la persona. Un tag `v*` que cree el agente no
     se impide (F1-6, divergencia 5); `doctor` lista los tags `v*` cuyo
     commit de tag no firmó una persona de la lista, y el workflow de
     publicación (F7) solo publica tags firmados por una persona de la
     lista.
   - **D4a** merge del agente con todo en verde: límite escrito; `finish`
     exige identidad de persona.
   - **D5a** errata y hotfix con tarea propia: `taskctl errata "<título>"`
     crea la tarea con `carril: errata`, su rama `fix/task-NNN-<slug>` y el
     PR, sin plan; el hotfix es una tarea de tipo `hotfix` (F1-3).
   - **D6a** riesgo alto con 2 o más personas en la lista de firmantes: el
     firmante del `approve` y quien aprueba el PR tienen que ser personas
     distintas (lo comprueba `carril`); con una, D4a de F1-5. La lista es la
     de F1-6 (D6a), en `.taskcode/`, y la mantiene la persona por PR.
   - **D7a** tarea que toca `.github/workflows/` con una sola persona:
     empuja la persona (el agente no tiene `workflows`, F1-8) y después el
     agente hace un commit propio encima (que no toca workflows), para que el último push sea suyo y la persona pueda
     aprobar (`require_last_push_approval`, F1-5). No verificado (F2-6).
   - **Ramas y releases que el agente alcanza** (F1-6): no se añade ningún
     ruleset; las ramas que importan ya las protege F1-5, otra rama no la
     crea el CLI y su PR falla en `carril` por no tener tarea, y una release
     de GitHub borrada se rehace desde su tag, que no se puede borrar.

## Preguntas heredadas

Lo que los ADR anteriores dejaron a F1-10, con su decisión aquí.

<!-- preguntas-heredadas:inicio -->
| Pregunta heredada | Decisión | Origen |
|---|---|---|
| ¿El hotfix necesita tarea propia o basta un carril de errata? | tarea propia de tipo `hotfix`; la errata también tiene tarea (D5a) | F1-3 decisión 2 y consecuencias |
| ¿Quién puede lanzar `release` y `hotfix`? | release: la persona (`merge-pr`, `tag-anotado`); hotfix: el agente lo trabaja, la persona lo cierra (D3a) | F1-3 preguntas abiertas (dominio) |
| ¿Quién puede lanzar `merge-pr`, `tag-anotado` y `release`? | la persona; paradas fijas en cualquier modo (decisión 6) | F1-4 fuera de alcance |
| Regla de derivación del carril mínimo por encima de errata | decisión 4: suelo en el binario, `rutas_carril` solo sube, nunca funcionalidad | F1-5 decisión 4, consecuencias y preguntas abiertas |
| Renombrados: un `.github/` → `docs/` no puede salir errata | se miran los dos nombres (`previous_filename`) | F1-5 preguntas abiertas (revisión 3) |
| PR enorme de solo documentación; ¿cuenta el tamaño? | no cuenta: sigue siendo errata (decisión 4) | F1-5 preguntas abiertas |
| ¿La errata necesita tarea con menos ceremonia? | `taskctl errata` crea tarea, rama y PR sin plan (D5a) | F1-5 divergencia 8 y consecuencias |
| Política de cada carril en configuración | `politica_carriles` y `rutas_carril`, solo endurecen (decisión 2) | F1-5 consecuencias; criterio de F1-10 |
| Riesgo alto en un equipo: ¿dos personas distintas? | con 2 o más, firmante del plan distinto del aprobador del PR (D6a) | F1-5 preguntas abiertas; F1-6 fuera de alcance y preguntas abiertas |
| ¿Quién mantiene la lista de personas? | la persona, por PR, en `.taskcode/` bajo CODEOWNERS (D6a de F1-6) | F1-6 preguntas abiertas |
| Tags `v*` y releases que el agente puede crear | no se impiden; `doctor` y la publicación solo aceptan tags firmados por una persona de la lista (D3a) | F1-5 preguntas abiertas; F1-6 divergencia 5 y consecuencias |
| Renombrar o borrar ramas no protegidas y releases; ¿más rulesets? | ninguno nuevo (decisión 7) | F1-6 consecuencias |
| Merge por el agente con todo en verde | límite escrito: `finish` exige persona (D4a) | F1-6 preguntas abiertas; F1-8 `P9` |
| Tarea que toca `.github/workflows/` con una sola persona | empuja la persona y el agente pone un commit propio encima (D7a), no verificado | F1-6 preguntas abiertas; F1-8 `P8` y consecuencias |
| Valores de `carril` y qué para cada uno | cuatro valores (D1a) y la tabla de política | F1-9 decisión y consecuencias |
<!-- preguntas-heredadas:fin -->

## Divergencias con F1-5, F1-6 y F1-9

Los ADR aceptados no se modifican; lo que cambia queda aquí y en
«Consecuencias»:

1. **F1-5, tabla de carriles**: sus seis filas pasan a ser cuatro carriles y
   dos tipos (D1a); las puertas (checks, aprobaciones, base) son las mismas
   (PoC B).
2. **F1-5, PoC F**: allí `.github/` sube a riesgo alto «en la PoC» como
   ejemplo; aquí es el suelo, con `.taskcode/`, cualquier `CODEOWNERS` y la
   comparación sin mayúsculas.
3. **F1-6, divergencia 5**: el agente puede crear tags; aquí no se impide,
   se detecta y no se publica (D3a).
4. **F1-9, tabla de campos**: `complejidad` figura como «entrada del carril
   (F1-10)»; aquí no decide el carril, solo cuántos roles lleva una
   funcionalidad.
5. **F1-5, decisión 2** (revisión 2): F1-5 no exige `revision-ia` en la
   release («una release no la necesita»). Con la unión de puertas, la
   release la lleva siempre, porque todas las filas de carril incluyen
   `revision-ia` (también errata, con perfil rápido): un PR `develop` →
   `main` pasa por el revisor. Endurece, no baja nada; su coste en minutos
   lo mide F1-16 o F7.

## Requisitos y evidencia

| Requisito | Decisión | Evidencia |
|---|---|---|
| Criterio de F1-10: política por carril, configurable | tabla única (decisión 1) y dos claves que solo endurecen (decisión 2) | PoC B (6 filas, 0 fallos, 12/12 desvíos detectados); PoC D (5/5 aceptadas, 17/17 rechazadas) |
| P7 / RR-07 carriles proporcionales; el automático para solo en las puertas del carril | paradas fijas (decisión 6); errata y fix sin `approve` | PoC C (paradas = fila de la tabla en todas las tareas; 0 `continuar` en fijas); las paradas por tarea suben (divergencia 3) |
| RF-21 modos con encadenado | `accion` en `siguiente --json`, modo congelado | PoC C (manual, semiautomático y automático en las tareas reales) |
| RR-08 garantías en la plataforma | `carril` impone con la misma función; política de la base | decisión 3; PoC D (PR que cambia `.taskcode/`); GitHub: no verificado (F2-6) |
| RR-04 / RF-50 aprobación de una persona antes del merge | `merge-pr` y `approve` como paradas fijas | PoC C (1260 combinaciones, 0 `continuar`) |
| H5 / H6 aprobación comprobable; el agente no se salta puertas | `approve` firmado y `merge-pr` de persona en cualquier configuración | PoC C (los 114 `approve` con `decidido_por: automatico` de ToolsaWeb pasan a `detener`); PoC D |
| RR-02 / RR-17 roles y revisoras proporcionales al carril | columnas «Roles» y «Revisoras» | PoC B (uno en fix, todos en riesgo alto); contenido en F1-15 |
| RR-18 / RF-27 lotes con revisión humana al final | columna «Lote» | tabla; diseño en F1-16 |
| RF-33 tope de rondas | parada `preguntar` con las salidas de la 0.16 | PoC C (tope=69 / 40 / 2) |
| RF-51 `## Resultado` | sin cambio: lo exige `carril` (F1-5) | — |
| RN-06 una persona con agentes | D4a de F1-5 y D7a | D7a no verificado (F2-6) |
| RR-08 / F1-8 `A10` lo escrito en `tarea.md` no baja una puerta (revisiones 1 y 2, IMP-1) | `tipo` solo elige fila con base real `main` y su rama de origen (`develop` en release, `hotfix/*` en hotfix); contra `develop`, el carril efectivo; las puertas de una fila de `main` son la unión con las del carril efectivo | PoC C (11/11: `tipo: hotfix` con diff en `.github/` sale riesgo alto contra `develop` y hotfix + riesgo alto contra `main`; M1, `release` desde `feature/*` → falla cerrado; M3, hotfix funcionalidad → plan y `approve`; release funcionalidad desde `develop` → unión; 0 de 8 filas de `main` por debajo de su carril) |
| Derivación con renombrados (F1-5) | los dos nombres; nunca funcionalidad | PoC A (203 tareas: 0 funcionalidad; 23/23 casos, `.github/` → `docs/` = riesgo alto) |
| Preguntas abiertas de F1-3, F1-4, F1-5, F1-6, F1-8 y F1-9 | tabla «Preguntas heredadas» (15 filas, ninguna sin destino) | este ADR |

## Consecuencias

- **F1-15** usa el carril como entrada de roles y revisoras (columnas de la
  tabla) y puede endurecerlas con `politica_carriles`.
- **F1-16** diseña el lote sobre la columna «Lote» (errata, fix y
  funcionalidad con plan aprobado; nunca riesgo alto, release ni hotfix) y
  revisa si el lote compensa las paradas que suben (divergencia 3).
- **F1-17** estima con el carril efectivo y el tamaño del diff.
- **F2-7** suma `politica_carriles` y `rutas_carril` al esquema (dos de las
  12 claves) con el validador de la decisión 2.
- **F3-8** (`doctor`) dice si la política de la base es válida, lista los
  tags `v*` sin firma de persona de la lista y repite «puertas no impuestas
  por la plataforma» donde no haya rulesets (F1-5 D2a): sin ellos, las
  paradas locales son lo único que hay.
- **F4-1** (`plan`) lanza los roles de la fila y congela modo y carril
  declarado; **F4-2** (`approve`) solo existe en los carriles con plan
  aprobado; **F4-4** (orquestador) usa `accion` y no decide nada por su
  cuenta; **F4-7** (`lote`) respeta la columna «Lote».
- **F6** implementa `carril` con las mismas funciones (carril mínimo,
  efectivo, política de la base, D6a).
- **F7** publica solo tags firmados por una persona de la lista (D3a).
- **Comando nuevo `taskctl errata`** (D5a): sin fila en el plan; acción en el
  plan (la hace Carlos al revisar el plan): añadirlo a la fila de altas
  correspondiente de la Fase 3.
- **Este repo no cambia** (RR-14): sigue con la 0.16, sin carriles, hasta el
  relevo.

## Fuera de alcance

- Puertas de la plataforma, rulesets y el check `carril` en sí: F1-5, F6.
- Identidades y permisos: F1-6.
- Perfiles y caché de `verificacion`: F1-7.
- Seguridad de los workflows: F1-8.
- Selección de skills, agentes y modelo por rol: F1-15.
- Lote: rama, presupuesto, qué aparta una tarea: F1-16.
- Estimación: F1-17.
- GitLab: F1-13.
- Implementación y E2E: F2-6, F2-7, F3-8, F4-1, F4-2, F4-4, F4-7, F6.

## Preguntas abiertas

Este ADR se planificó con un solo rol (arquitectura); quedan sin cubrir:

- **Reglas de negocio de la tabla**: cuántos roles según la complejidad, qué
  revisoras por carril y si una funcionalidad entra en el lote no las revisó
  un rol de dominio. Son propuesta; F1-15 y F1-16 las fijan.
- **`.claude/` y otros ficheros que cambian el comportamiento del agente**
  (hooks, settings) no están en el suelo de riesgo alto; ToolsaWeb lo tocó
  una vez en el historial. Un proyecto lo puede subir con `rutas_carril`; si
  debe ir en el suelo, lo decide F1-8 o F1-11.
- **Paradas por tarea frente a RR-07** (divergencia 3): si el lote no las
  compensa, cabe medir otra vez con datos de la 2.0 (P9).
- **Modos de fallo de la derivación** (sin rol de riesgos): globs ambiguos,
  rutas con mayúsculas en sistemas que no las distinguen, PR de más de 3000
  ficheros, submódulos y enlaces simbólicos.
- **Configuración inválida en la base**: falla cerrado bloquea todo PR,
  también el que la arregla; el que toca `.taskcode/` tendría que poder
  pasar con las puertas de riesgo alto. Lo concreta F2-7.
- **Release con riesgo alto**: una release cuyo diff acumulado toca
  `.taskcode/` (4 de las 160 tareas de ToolsaWeb lo cambiaron) exige plan
  firmado, y F1-3 no le da tarea; F1-5 ya hace fallar todo PR sin tarea.
  Cómo se da tarea y plan a una release lo concretan F1-3 y F7.
- **GitLab**: la misma derivación sobre el diff del MR y la lectura de la
  configuración de la rama destino (F1-13).

## No verificado

- `previous_filename` en `GET /pulls/{n}/files`, el corte a 3000 ficheros y
  la lectura de la configuración de la base desde `pull_request_target`:
  F2-6.
- D7a (commit del agente encima de un cambio en workflows sin `workflows`):
  F2-6.
- El carril declarado de las tareas de la 0.16 usado en la PoC C es una
  suposición; las cifras «hoy» son aproximadas (divergencia 4).
- La salida de `siguiente --json` con los campos nuevos y lo que pinta el
  check `carril`: diseño, sin implementación (F4-4, F6).
- `taskctl doctor` no se ejecutó en la implementación de esta tarea.

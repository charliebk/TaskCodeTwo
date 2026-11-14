# Plan — TASK-011: F1-10 ADR carriles y modo automatico

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 75 min

Senales: complejidad simple por la heuristica (1 punto, depende de TASK-006), pero el ADR cierra once preguntas abiertas de F1-3, F1-5, F1-6 y F1-8 y lleva una PoC de lectura sobre el historial real. **No se trocea** (ver «Motivo para no trocear»): derivacion del carril, politica por carril y paradas del modo automatico salen de la misma tabla.

**Que se crea**: `docs/adr/F1-10-carriles-y-modo-automatico.md` con la plantilla de `docs/adr/README.md` y su fila en el indice; guion y log en `poc/` de la tarea, ejecutados en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-011\` (borrada al terminar).
**Que no se toca**: ningun ADR aceptado (lo afectado va a «Consecuencias»), `.taskcode/config.yml`, codigo ni GitHub (solo `GET`). ToolsaWeb y OpenDockTrading solo se leen con `git log`.

### Que decide el ADR

1. **Una sola tabla de politica por carril, fuente unica**: por fila, roles de diseno, aprobacion del plan, revisoras, perfil de verificacion, en que fases para el modo automatico y si entra en un lote. Funcion pura del nucleo sin E/S: `carrilMinimo(cambios, reglas)`, `carrilEfectivo(declarado, minimo)`, `politica(carril, config)`, `accion(fase, tarea, politica, modo) -> detener|preguntar|continuar`. La usan `siguiente`/orquestador (F4-4), `plan` (F4-1, F4-6), el check `carril` (F6), `lote` (F1-16) y `doctor`. Se extiende el contrato `fase/accion/motivo` de `siguiente --json` de la 0.16 (`skills/task-workflow/avance.md`) en vez de crear otro.
2. **La configuracion solo endurece**: el suelo de F1-5 (errata solo con `docs/`; `.github/`, `.taskcode/` y CODEOWNERS suben a riesgo alto) va en el binario. La configuracion en `.taskcode/` (bajo CODEOWNERS, F1-6 D6a) puede subir un carril, anadir rutas o anadir paradas; una clave que baja algo la rechaza el esquema y se falla cerrado. Suma dos claves al tope de 12 de F2-7: `politica_carriles` y `rutas_carril`.
3. **Confianza de lo que se lee**: el CLI lee la politica de `origin/develop:.taskcode/config.yml` y el check `carril` desde `pull_request_target` (F1-8 D1b); nunca del worktree de la tarea. Las paradas locales del orquestador son de usabilidad; quien impone es `carril`, con la misma funcion.
4. **Derivacion del carril minimo**: ficheros del PR por la API; en cada renombrado se miran los dos nombres (`previous_filename`, pregunta abierta de F1-5). Da errata, fix o riesgo alto; por ficheros nunca sale funcionalidad (diferencia semantica, D2).
5. **Que congela la tarea**: en `plan` se congelan el modo (como en la 0.16) y el carril declarado (campo `carril` de F1-9). La politica no se congela: se evalua con la configuracion vigente en la base. Si en `review` el carril efectivo supera al declarado y exige plan aprobado, `accion` devuelve `detener` con «vuelve a plan».
6. **Paradas fijas del modo automatico, en cualquier configuracion**: `approve` firmado en funcionalidad y riesgo alto; `merge-pr` solo por una persona (F1-6); tag de release y de hotfix; tope de rondas; subida de carril. Fuera de esas, todo se encadena.
7. Cierra ademas: valores de `carril` (D1), quien lanza la release y tags del agente (D3), merge del agente con todo en verde (D4), errata y hotfix (D5), riesgo alto con varias personas (D6), tarea que toca workflows con una sola persona (D7).

**Orden**: 1) tabla de politica y su contraste con la de F1-5; 2) regla de derivacion y PoC sobre el historial; 3) funcion `accion` y contraste con las transiciones reales; 4) claves de configuracion y controles negativos; 5) Consecuencias (F1-15, F1-16, F1-17, F2-7, F4-4, F6) y fila del indice.

### Motivo para no trocear

La persona (Carlos) indico expresamente al lanzar el H1 el 2026-11-14: «NO troceamos ninguna tarea». Ademas, partirla daria dos fuentes para la misma tabla de politica (D0). Se aprueba entera aunque la estimacion pase de 60 min.

## Lo que el rol no cubrio

Solo arquitectura.

- «Politica por carril, configurable»: cubierta la forma; las reglas de negocio (que complejidad lleva a que carril, cuantos roles por carril) no las reviso el rol de dominio; van como propuesta.
- «Cumple F1-10»: sin revision de riesgos; los modos de fallo de la derivacion (globs ambiguos, PR enorme, la API pagina ficheros a partir de 3000) solo se nombran.
- Estrategia de pruebas sin rol de testing: el plan es el minimo de arquitectura.
- No se leyo `docs/auditoria/` (ninguna afirmacion del plan depende de ella).

## Riesgos aceptados y que los contiene

- **El agente puede saltarse las paradas locales** (editando su config o sin llamar a `siguiente`): lo contiene el check `carril`, que lee la base; sin rulesets (privado gratuito, F1-5 D2a) no hay contencion y `doctor` lo dice.
- **Configuracion distinta entre base y rama** en un PR que cambia `.taskcode/`: ese PR sale riesgo alto por ruta (suelo) y CODEOWNERS.
- **Globs mal escritos**: solo pueden subir el carril (la config no baja nada): mas paradas, nunca menos.
- **Comportamiento real en GitHub** (`previous_filename`, `pull_request_target` leyendo la config de la base): no verificado; criterio de F2-6.

## Plan de pruebas

PoC de solo lectura (`poc-f1-10.py` con `python -I` y su `.log`): A derivacion sobre `git log -M --name-status` de las ramas de tarea integradas en ToolsaWeb y OpenDockTrading (cuantas saldrian errata, fix o riesgo alto; renombrados `R` con sus dos nombres); B tabla de politica del ADR entre marcadores frente a la de F1-5; C `accion` simulada sobre las `## Transiciones` reales de la 0.16 (paradas humanas por tarea con la politica nueva frente a hoy, RR-07); D configuraciones que bajan algo como controles negativos.

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio). Ninguno pesado.

- **Politica por carril, configurable → 2**
  1. PoC B: tabla del ADR frente a F1-5 con 0 fallos y desvios inyectados detectados.
  2. PoC D: toda clave que baja un carril o quita una parada es rechazada (N/N).
- **Cumple F1-10 y lo que cita → 3**
  1. PoC A: un renombrado de `.github/` a `docs/` no sale errata y ninguna tarea se deriva como funcionalidad.
  2. PoC C: las paradas del automatico coinciden con las puertas del carril y nunca hay `continuar` sobre `approve`, `merge-pr` o un tag.
  3. Una fila en «Requisitos y evidencia» por cada pregunta abierta heredada (F1-3, F1-5, F1-6, F1-8), ninguna sin destino; al cerrar, `bash .github/scripts/comprobar-repo.sh` en verde y `taskctl doctor` sin errores.

## Lo que necesita decision de una persona

Decisiones respondidas por la persona (Carlos) el 2026-11-14 con la consigna «responder la opcion recomendada a todas las preguntas» y «no trocear»:

- **D0. Trocear** → **(a)** un ADR entero (~75 min); ver «Motivo para no trocear».
- **D1. Valores de `carril`** → **(a)** cuatro valores ordenados `errata < fix < funcionalidad < riesgo-alto`; release y hotfix los da `tipo` (reglas de base `main`). Reinterpreta la tabla de 6 filas de F1-5 sin cambiar sus puertas.
- **D2. Funcionalidad por ficheros** → **(a)** nunca se deriva por ficheros; solo se declara en `plan`.
- **D3. Release y tags** → **(a)** la release la lanza la persona (`merge-pr` y `tag-anotado`, de persona en F1-6); al agente no se le impide crear un tag `v*`, pero `doctor` lo detecta y el workflow de publicacion solo publica tags firmados por una persona de la lista.
- **D4. Merge del agente con todo en verde** → **(a)** limite escrito: `finish` exige identidad de persona; no se anade nada mas.
- **D5. Errata y hotfix** → **(a)** las dos con tarea propia; `taskctl errata "<titulo>"` crea tarea, rama y PR sin plan.
- **D6. Riesgo alto con varias personas** → **(a)** con 2 o mas personas en la lista, el firmante del plan distinto del aprobador del PR; con 1, D4a de F1-5. Sin clave nueva.
- **D7. Tarea que toca `.github/workflows/` con una sola persona** → **(a)** empuja la persona y despues el agente hace un commit propio encima para que el ultimo push sea suyo; no verificado que GitHub lo admita sin `workflows` (F2-6).

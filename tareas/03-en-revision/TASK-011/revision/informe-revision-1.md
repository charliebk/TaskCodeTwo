# Informe de revision — TASK-011 (ronda 1)

- Commit revisado: e83f838c05980a2115e24949afbd7896e042bdc2
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-10-carriles-y-modo-automatico.md:330 | implementacion |
| MEN-1 | MENOR | corregido | tareas/03-en-revision/TASK-011/poc/poc-f1-10.py:180 | implementacion |
| MEN-2 | MENOR | corregido | docs/adr/F1-10-carriles-y-modo-automatico.md:387 | implementacion |
| MEN-3 | MENOR | corregido | docs/adr/F1-10-carriles-y-modo-automatico.md:220 | implementacion |
| MEN-4 | MENOR | corregido | docs/adr/F1-10-carriles-y-modo-automatico.md:330 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Revision hecha con el pase generico (diff de solo documentacion y una PoC en Python; sin dominios con revisor propio).

## Reproduccion y correccion

### IMP-1 — `tipo` (texto de `tarea.md`, no confiable) elige la fila y puede bajar la puerta

ADR, decision 1 (linea 330): «con `release` o `hotfix` la fila es la del tipo». Filas en lineas 345-346. La decision 3 dice que lo declarado en `tarea.md` «solo sirve para subir», pero `tipo` no esta sometido al «solo sube»: no hay regla que exija que el PR tenga base `main` para usar esas filas, ni que `carril` lea `tipo` de otra fuente que la tarea.

Reproduccion (funciones de la PoC cargadas con `importlib`, tabla leida del ADR; PR con `.github/workflows/ci.yml` y `src/a.rs`, declarado `fix`):

```text
carril_minimo -> riesgo-alto ; carril_efectivo(fix, riesgo-alto) -> riesgo-alto
politica(efectivo, tipo=feature) -> fila riesgo-alto  plan=True aprob=2 paradas=[approve, merge-pr] base=develop
politica(efectivo, tipo=hotfix)  -> fila hotfix       plan=False aprob=1 paradas=[merge-pr, tag]     base=main
politica(efectivo, tipo=release) -> fila release      plan=False aprob=1 checks=[carril, verificacion] (sin revision-ia)
```

Con el mismo diff, escribir `tipo: hotfix` o `tipo: release` en `tarea.md` quita el plan aprobado y firmado, la segunda aprobacion y (en release) `revision-ia`. Lo unico que lo frenaria es que el ruleset de `develop` exige `revision-ia` y la base, y eso no esta escrito en el ADR como regla de `carril` (y F1-8 A10 trata el texto de la tarea como no confiable; RR-08). Segundo hueco relacionado: un PR de release `develop`→`main` no tiene tarea (F1-3) pero F1-5 hace fallar todo PR sin tarea, y su diff acumulado sale `riesgo-alto` por el suelo en cuanto `.taskcode/config.yml` cambio desde la ultima release (4 de 160 tareas de ToolsaWeb): el ADR no dice si el minimo por ficheros se aplica a release y hotfix. En la PoC, `accion` en `review` de un hotfix con efectivo `riesgo-alto` devuelve «el carril sube de fix a riesgo-alto: vuelve a plan», aunque el hotfix no tiene fase `plan`.

Correccion: anadir a la decision 1 (y a la columna «Base del PR») una regla de `carril`: las filas `release` y `hotfix` solo se aplican si la base real del PR (API) es `main` y, para hotfix, la rama es `hotfix/*`; en cualquier PR contra `develop` se ignora `tipo` y manda el carril efectivo. Decir ademas que el minimo por ficheros no sube la fila de release (diff acumulado) y que en hotfix lo que toque `.github/`, `.taskcode/` o CODEOWNERS exige la puerta de riesgo alto (o queda expresamente fuera, con su motivo). Anadir a la PoC B/C un caso de ese diff con `tipo: hotfix` sobre base `develop` que tiene que seguir saliendo `riesgo-alto`.

### MEN-1 — la validacion revienta con valores no hashables en vez de rechazarlos

`poc-f1-10.py:180`: `any(f not in FASES_EXTRA for f in x)` con `paradas_extra: [["review"]]` lanza `TypeError: unhashable type: 'list'` en lugar de devolver el error de esquema. Falla cerrado igualmente, pero el ADR promete mensaje dirigido («`doctor` dice la clave»). Correccion: `any(not isinstance(f, str) or f not in FASES_EXTRA ...)`, y un caso en D.

### MEN-2 — el suelo de riesgo alto compara `.github/` con barra final

ADR linea 387 y PoC linea 112 (`baja.startswith((".github/", ".taskcode/"))`). Una ruta exacta `.github` o `.taskcode` (fichero, enlace simbolico o submodulo con ese nombre) sale `fix` (reproducido: `clase_ruta('.taskcode') -> fix`). El ADR deja los enlaces y submodulos como pregunta abierta, pero el suelo debia cubrirlo sin coste. Correccion: comparar el primer componente (`ruta.lower().split('/')[0] in ('.github', '.taskcode')`).

### MEN-3 — cifras de la comparacion «hoy» sin respaldo en la PoC y no reproducibles exactamente

ADR lineas 218-221 y 500 («156 `finish`», «114 de los 157 `approve` en automatico», «114 `approve` automaticos de hoy pasan a `detener`», citados como evidencia de PoC C): ni el guion ni el log imprimen 156, 157 ni 114. Contando las filas `## Transiciones` de ToolsaWeb en el commit fijo 93c9618ac (solo lectura): `finish automatico persona` = 156, `approve automatico automatico` = 115, `approve automatico persona` = 44 (159 en total, no 157). Correccion: que la PoC C imprima esos tres recuentos (y el ADR cite lo impreso) o quitar la cifra. Ademas, la cifra «nuevo» 2,52 supone que ninguna tarea de la 0.16 se habria declarado errata o fix (el guion declara `funcionalidad` salvo tipo `fix`); la frase «lo contienen la errata y el fix sin plan» (linea 262) no esta cuantificada. La divergencia con RR-07 esta bien justificada y comunicada (divergencia 3, preguntas abiertas, consecuencias de F1-16), pero conviene anadir la cota inferior (23 tareas de solo `docs/` como errata) al lado del 2,52.

### MEN-4 — la funcion `politica` no recibe la complejidad que la tabla usa

Linea 330: la fila `funcionalidad` dice «1 a 4, segun la complejidad», pero `politica(carril, tipo, config)` no tiene `complejidad` y la PoC fija `roles_min=1`. Correccion: o anadir `complejidad` a la firma, o decir que el numero de roles lo resuelve `plan` (F4-1) por encima del minimo de la tabla.

## Verificacion realizada

- PoC ejecutada con `python -I` y carpeta de trabajo `%LOCALAPPDATA%/pruebas-claude/taskcodetwo/task-011-r1` (vacia): salida identica al `poc-f1-10.log` salvo la linea de la ruta de trabajo; `RESUMEN: todo como se espera: A, B, C y D`.
- Mutacion de la tabla del ADR (copias en el scratchpad, no en el repo): errata con 2 aprobaciones y riesgo-alto sin parada `approve`; la PoC B falla con «aprobaciones 2 != F1-5 1», «riesgo-alto: parada approve False sin casar con plan True» y «es menos estricto que el anterior». La comparacion discrimina.
- Casos fuera de la PoC: IMP-1, MEN-1, MEN-2 (arriba); rutas `./.github/x`, `docs//x`, `DOCS/x`, `docs/.github/workflows/a.yml`, `docs/../.github/x` (error); `rutas_carril` con `docs/**` en fix y riesgo alto a la vez (acepta, solo sube); `perfil` y globs con tipos raros (rechazados); tope de rondas en los tres modos (`preguntar`, como en la 0.16).
- La configuracion solo endurece: revisadas todas las claves (`roles_diseno`, `plan_aprobado`, `paradas_extra`, `revisoras_extra`, `lote`, `perfil`, `rutas_carril`); ninguna combinacion relaja una puerta. El hueco real es `tipo`, no la configuracion (IMP-1).
- Paradas fijas: `accion` resuelve `approve`, `merge-pr` y `tag` antes de politica y modo; 900 combinaciones con 0 `continuar` reproducidas. Son paradas locales; la imposicion es de `carril` y el ADR lo dice.
- Coherencia con F1-5 (tabla de carriles, derivacion, errata solo `docs/`), F1-6 (operacion→identidad, `merge-pr` y `tag-anotado` de persona), F1-8 (`carril` en `pull_request_target`; IDs A10, P8, P9, D1b existen) y F1-9 (`carril: null`, `complejidad`: divergencia 4 y 7 documentada). IDs citados (RR-02/04/07/08/17/18, RF-21/27/33/50/51, RN-06, H5, H6, F2-6, F2-7, F3-8, F4-1/2/4/7, F6, F7) existen. La tabla de 15 filas de preguntas heredadas esta completa.
- `bash .github/scripts/comprobar-repo.sh`: «Tareas: 93; enlaces relativos: 82; fallos: 0».
- Criterios de `tarea.md`: «Politica por carril, configurable» cubierto (tabla unica y dos claves que solo endurecen) con la salvedad de IMP-1; «Cumple lo que pide F1-10» cubierto en lo demas. `taskctl doctor` no ejecutado (el ADR lo declara).
- No verificado: comportamiento de GitHub (`previous_filename`, corte a 3000 ficheros, `pull_request_target`), D7a; el ADR ya lo marca como F2-6.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-011/poc/poc-f1-10.py` y `poc-f1-10.log`: lo pide el encargo (reproducir la PoC).
- `docs/adr/F1-3-...`, `F1-5-...`, `F1-6-...`, `F1-8-...`, `F1-9-...`, `F1-7-...` (solo greps y fragmentos): comprobar coherencia y que los IDs citados existen.
- `docs/plan/PLAN-TASKCODE-2.md`, `REQUISITOS-FUNCIONALES.md`, `AMPLIACION-2026-11-14.md`, `docs/auditoria/AUDITORIA-EXTERNA-2026-11-13.md` (greps): comprobar IDs y la fila F1-10.
- `ToolsaWeb` (solo lectura, `git grep` en el commit 93c9618ac): recuento de transiciones para MEN-3.
- `.github/scripts/comprobar-repo.sh` (ejecutado).

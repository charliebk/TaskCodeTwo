# Informe de revision — TASK-074 (ronda 3)

- Commit revisado: e4490f72fa66678ada8a660462e4f1df76b24e2d (HEAD de la peticion: 7675fb0, solo añade la peticion)
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

Verificacion acotada al diff de las correcciones (`08a98c8..HEAD`: ADR F1-16, `docs/adr/README.md`, PoC y su log). Profundidad: a fondo `nucleo_tarea`/`partes_tarea`, `carril_minimo`/`es_riesgo_alto`, `comprobar_primer_padre` y el evento `ajeno-aceptado` (C4c, C4d, C4e, C6c-e, C9b); por encima el resto del ADR y las filas del indice.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-16-lotes-desatendidos.md y tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:239 (`carril_minimo`) y C9, C9b | informe-revision-2.md |
| IMP-2 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:367 (`comprobar_primer_padre`), C5c | informe-revision-2.md |
| IMP-3 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:332 (`nucleo_tarea`), C6c | informe-revision-2.md |
| IMP-4 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-074/poc/poc-f1-16.py (C10, C10b) | informe-revision-2.md |
| IMP-5 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-074/poc/poc-f1-16.py (`develop_agente`, C11) | informe-revision-2.md |
| NUE-1 | IMPORTANTE | corregido | docs/adr/F1-16-lotes-desatendidos.md:494-507 y tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:308-338 | implementacion |
| NUE-2 | MENOR | corregido | docs/adr/F1-16-lotes-desatendidos.md y tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:223-246 | implementacion |
| NUE-3 | MENOR | corregido | docs/adr/F1-16-lotes-desatendidos.md:640-650 y tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:367-405 | plan |
| NUE-4 | MENOR | corregido | docs/adr/F1-16-lotes-desatendidos.md:602-612 y :815-821 | implementacion |
| NUE-5 | IMPORTANTE | corregido | docs/adr/F1-16-lotes-desatendidos.md:138-143 y :494-507 y tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:308-310 (`CLAVES_CLI`, `SECCIONES_CLI`) y :1368-1412 (`medir_escrituras_cli`) | implementacion |
| NUE-6 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:1885-1955 (C4c-C4e) y :386-390 | implementacion |
| NUE-7 | MENOR | corregido | tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:570-573 y :1000-1001 (`por in PERSONAS_LOTE`) | implementacion |
| NUE-8 | MENOR | corregido | docs/adr/F1-16-lotes-desatendidos.md:640-650 y :815-826 y tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:386-390 | implementacion |
| NUE-9 | MENOR | corregido | tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:313-338 (`partes_tarea`) | implementacion |

Los NUE-5..NUE-9 son nuevos y los abre la propia correccion de la ronda 2 (NUE-1 y NUE-3). No hay CRITICO. Abiertos: 2 IMPORTANTE (NUE-5, NUE-6) y 3 MENOR (NUE-7, NUE-8, NUE-9).

## Ejecucion

- PoC completa, una sola vez, sobre un clon de 7675fb0 (`python -I -B tareas/03-en-revision/TASK-074/poc/poc-f1-16.py`, 3088 s): `RESULTADO: todo como se espera`, exit 0, `errores del ADR: 0`, carpeta de trabajo borrada. Su salida es identica al `poc-f1-16.log` commiteado tras normalizar CRLF (28083 caracteres cada uno). Con eso quedan verificados en verde IMP-1 a IMP-5 (C9, C9b, C5c, C6c, C6d, C6e, C10, C11, C4c, C4d y C4e pasan). La suite del proyecto es esta PoC; el ADR no tiene otra.
- IMP-1..IMP-5: no se repitieron las mutaciones de la ronda 2 (siguen en el codigo sin cambios fuera de las zonas de abajo); la ejecucion integra es la comprobacion.

## Verificacion de NUE-1..NUE-4

### NUE-1 — corregido en lo que pedia

Con `nucleo_tarea` de la PoC sobre el `tarea.md` real de TASK-074: cambian el hash (apartan) el texto de un criterio, un criterio nuevo, una `## Escenarios` nueva, `carril` nuevo en el frontmatter, `agente_revisor`, `revision_codex`, `skills_recomendados`, `modelo_sugerido`, `complejidad`, un preambulo, CRLF. No lo cambian, y asi debe ser: marcar una casilla, `asignado_a`, `rama`, filas de `## Transiciones`/`## Coste`. Mutacion propia M1 (añadir `Escenarios` a `SECCIONES_CLI`, `seccion_c` entera): falla C6d y solo C6d. Cambiar el texto de un criterio no pasa (solo se ignoran las marcas). `rama` o `asignado_a` cambiados por un ajeno no aparta, pero el lote decide con el diario y con Git, no con ellos, como dice el ADR.

### NUE-2 — corregido

`carril_minimo` da `riesgo-alto` para `.github/`, `.GitHub/`, `./.github/`, `.\.github\x`, `.taskcode` (fichero o enlace), `.TaskCode/config.yml`, cualquier `CODEOWNERS`, `.gitlab-ci.yml` (cualquier mayuscula y subcarpeta), `.gitlab/`, un renombrado con los dos nombres y `rutas_carril`; `.githubx/` y `Docs/` dan `fix`, como en F1-10. Solo `.//.github/x` da `fix` (F1-10 normaliza los componentes vacios; git no emite esa ruta en un diff), sin consecuencia.

### NUE-3 — corregido en sustancia, con dos huecos de prueba (NUE-6, NUE-7) y uno de texto (NUE-8)

`ajeno-aceptado` con `decidido_por: agente` se rechaza (C4d); la reversion total por una persona desbloquea sin evento (C4e). Con la funcion real: la reversion parcial (arbol distinto) y un commit de persona de otro arbol no desbloquean; la reversion con autor agente tampoco.

### NUE-4 — corregido

El ADR lo dice en la decision 6 y en `No verificado`, con el caso a ejecutar en F2-6, y la guarda falla cerrado (`comprobar_primer_padre` rechaza un arbol distinto de `arbol_esperado`).

## Hallazgos nuevos

### NUE-5 — La lista cerrada de lo que escribe el CLI no es la de la 0.16.1: aparta tareas legitimas (IMPORTANTE)

- Donde: ADR:138-143 («fuera de esa lista, ninguno») y :494-507 («La lista sale de lo que escriben de verdad los commits del CLI de la 0.16»); PoC:308-310 y `medir_escrituras_cli` (:1368-1412), que solo mide las 17 tareas de ToolsaWeb.
- Reproduccion (ejecutada): `nucleo_tarea` sobre cada commit `chore(TASK-NNN)` que toca un `tarea.md` en este mismo repo (101 con antes y despues). El nucleo cambia en 54. Las secciones fuera de la lista: `## Autorrevision` (36 commits; la escribe quien implementa antes de `review`, `core/autorrevision.js` de la 0.16.1 avisa si falta y F1-9 la tiene como RF-22), `## Verificacion ligera` (13; la escribe el CLI, `commands/verificar-menores.js`), `## Rondas extra` (7; p. ej. e4490f7 de esta misma tarea) y `## Decisiones de la persona` (2). Ejemplos: b983479 (autorrevision de TASK-074), ccb28cb, 72901f6. Cada una cambia el hash y aparta la tarea con `manifiesto-cambiado`.
- Por que no lo ve la PoC: las 17 tareas de ToolsaWeb no tienen ninguno de esos commits (`otros: 0`), y C6c simula solo las escrituras de la lista. TaskCodeTwo es uno de los tres repos objetivo del ADR y trabaja con la 0.16.1.
- Impacto: falla cerrado, pero en un caso real y no de borde: en un repo con el flujo de la 0.16.1, la tarea que escribe `## Autorrevision` antes de `review` se aparta a mitad de noche sin motivo. La lista blanca de la ronda 2 no lo hacia; lo introduce el paso a exclusion. El informe de la ronda 2 ya nombraba `## Autorrevision` entre lo que hay que excluir y la correccion no lo recogio.
- Correccion: medir la lista tambien en TaskCodeTwo y OpenDockTrading, añadir a la exclusion lo que escribe el CLI o el implementador por flujo (al menos `Autorrevision`, `Verificacion ligera`, `Rondas extra`; decidir `Decisiones de la persona`), añadir a C6c una escritura de `## Autorrevision` y de `## Verificacion ligera`, y corregir la frase «fuera de esa lista, ninguno».

### NUE-6 — La guarda de reversion no tiene ningun test que se ponga rojo al quitar su condicion central (IMPORTANTE)

- Donde: PoC:386-390 (`cadena[j].get("arbol") == arbol_previo`) y C4e (:1942-1955).
- Reproduccion (ejecutada): mutacion propia M4 (cambiar esa comparacion por `True`) y `seccion_c` entera: **0 fallos**. Es la condicion que impide que un commit cualquiera de una persona desbloquee el lote sin revertir nada (reversion parcial, commit sin relacion). La funcion real si lo rechaza (probado a mano con una cadena sintetica: arbol parcial y arbol distinto «paran»), pero C4e solo prueba la reversion total.
- Impacto: una proteccion nueva, la que hace que «revertir» no sea un `ajeno-aceptado` encubierto, sin red de regresion.
- Correccion: añadir a C4e dos casos: la persona empuja un commit con el arbol del ajeno mas un cambio (reversion parcial) y un commit de persona sin relacion; `--reanudar` tiene que seguir parando en los dos.

### NUE-7 — `por in PERSONAS_LOTE` no esta cubierto (MENOR)

- Donde: PoC:570-573 y :1000-1001; C4d (:1927).
- Reproduccion (ejecutada): mutacion propia M2 (quitar `por in PERSONAS_LOTE` en los dos sitios): 0 fallos. C4d usa `decidido_por: agente` y ya lo rechaza la primera condicion; nadie prueba un evento `decidido_por: persona` con `por` de otra identidad (por ejemplo el compañero).
- Impacto: MENOR porque `por` es un campo que escribe quien escribe el evento y la defensa real es la firma del commit de estado, que el ADR deja en `No verificado`.
- Correccion: añadir a C4d un evento con `decidido_por: persona` y `por` = el compañero.

### NUE-8 — «Una persona de la lista» en la reversion se comprueba por el autor del commit y no esta dicho (MENOR)

- Donde: ADR:640-650 y :815-826; PoC:386-390.
- Reproduccion (ejecutada, cadena sintetica con `comprobar_primer_padre`): un commit con el arbol de antes, `autor` = la persona y `committer` = la App del agente, desbloquea sin evento. El autor de un commit lo pone quien lo crea.
- Impacto: el arbol queda igual que antes, asi que no queda contenido ajeno; lo que la guarda deja de ver es que hubo un commit ajeno (y los workflows que un push a `lote/*` pudo ejecutar). El ADR exige firma para `ajeno-aceptado`, la declara no verificada, y no dice nada de la reversion.
- Correccion: en el ADR, exigir para la reversion la misma firma de persona que para `ajeno-aceptado` y añadirlo a `No verificado`.

### NUE-9 — `partes_tarea` pierde las secciones y claves con el mismo nombre: un duplicado anterior no cambia el hash (MENOR)

- Donde: PoC:313-338 (`secc[...] = ...` y `claves[k] = ...` por nombre).
- Reproduccion (ejecutada): sobre el `tarea.md` de TASK-074, un segundo `## Objetivo` o un segundo `## Criterios de aceptacion` colocado **antes** del original, o un segundo `titulo:` antes del original: el hash es el mismo (el ultimo gana). Colocado despues si cambia el hash. La 0.16 (`rangoDeSeccion`) tambien se queda con la ultima, pero quien lee el fichero, el agente, ve las dos.
- Impacto: MENOR, hace falta intencion de evadir el hash, y la misma via de texto libre existe en las secciones excluidas (ver notas).
- Correccion: hashear la lista ordenada de pares (titulo, cuerpo) sin colapsar, o parar la tarea si hay un encabezado o una clave repetidos.

## Mutaciones propias

| Mutacion | Resultado de `seccion_c` | Lectura |
|---|---|---|
| M1: `SECCIONES_CLI` + `Escenarios` (NUE-1) | falla C6d y solo C6d | protegido |
| M3: reversion sin `autor in personas` (NUE-3) | fallan C4, C4c, C4d, C4e, C5, C5b, C7b | protegido |
| M4: reversion sin comparar el arbol (NUE-3) | 0 fallos | NUE-6 |
| M2: sin `por in PERSONAS_LOTE` (NUE-3) | 0 fallos | NUE-7 |
| duplicado de seccion o clave anterior al original (hash directo) | mismo hash | NUE-9 |
| `## Resultado` / `## Transiciones` / `## Coste` con texto inyectado (hash directo) | mismo hash | nota 1 |

## Notas (no abren hallazgo)

1. Las secciones excluidas (`Resultado`, `Transiciones`, `Coste`) y las claves excluidas admiten texto libre que el agente lee (el implementador lee `## Resultado` para corregir hallazgos). Un ajeno puede escribir ahi sin apartar la tarea. Ya era asi en la ronda 2 y el ADR justifica la exclusion («el lote nunca decide con lo excluido»), pero no dice que el agente si lo lee.
2. ADR:605-607 dice que, si el arbol del merge real no coincide, «el informe lista las rutas en que difieren los dos arboles»; la PoC solo imprime los ids de los dos arboles. Es diseño del CLI, sin comprobar.
3. El carril del diff de la PoC (:1057) llama a `carril_minimo(rutas)` sin `rutas_carril`; el ADR dice que entran. Solo lo prueba una llamada suelta (:1534).
4. Lecturas fuera del contexto permitido: ver abajo.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-074/poc/poc-f1-16.py` y `poc-f1-16.log`: mi parte del diff (excluida del embebido); sin ellos no se reproduce ni se muta.
- `tareas/03-en-revision/TASK-074/revision/informe-revision-2.md` (lista de NUE a verificar).
- `docs/adr/F1-10-carriles-y-modo-automatico.md` y `tareas/04-terminadas/TASK-011/poc/poc-f1-10.py` (`clase_ruta`, `carril_minimo`): la regla que NUE-2 pedia contrastar.
- `git log`/`git show` de los commits `chore(TASK-NNN)` de este repo y los fuentes de la 0.16.1 en `~/.claude/plugins/cache/taskcode-marketplace/taskcode-plugin/0.16.1/dist/src` (`core/autorrevision.js`, `core/transiciones.js`, `commands/verificar-menores.js`, `commands/review.js`): lo que la 0.16 escribe de verdad, base de NUE-5.
- ToolsaWeb: solo `git clone --no-hardlinks --no-checkout` a la carpeta temporal por la propia PoC; nada ejecutado dentro.

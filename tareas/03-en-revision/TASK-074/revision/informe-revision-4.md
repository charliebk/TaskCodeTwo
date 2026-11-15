# Informe de revision — TASK-074 (ronda 4)

- Commit revisado: d57f5cc2451a17816ee38f9763f150a3349ebf15 (HEAD de la peticion: 2f23459, solo añade la peticion)
- Revisor: code-quality-reviewer
- Veredicto: aprobada

Verificacion acotada al diff de las correcciones (`e4490f7..HEAD`: ADR F1-16, `docs/adr/README.md`, PoC y su log). Profundidad: a fondo `medir_escrituras_cli` y su clasificacion, `partes_ordenadas`/`nucleo_tarea`, `comprobar_primer_padre` (reversion) y los casos C4f..C4i, C5c y C9c; por encima el resto del ADR.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| NUE-5 | IMPORTANTE | corregido | docs/adr/F1-16-lotes-desatendidos.md:138-143 y :494-507 y tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:308-310 (`CLAVES_CLI`, `SECCIONES_CLI`) y :1368-1412 (`medir_escrituras_cli`) | informe-revision-3.md |
| NUE-6 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:2044-2075 (C4g, C4h, C4i, C4e) y :415-418 | informe-revision-3.md |
| NUE-7 | MENOR | corregido | tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:602-604 y :1036 (`por in PERSONAS_LOTE`), C4f | informe-revision-3.md |
| NUE-8 | MENOR | corregido | docs/adr/F1-16-lotes-desatendidos.md:693-714 y :881-885 y tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:418 (committer), C4i | informe-revision-3.md |
| NUE-9 | MENOR | corregido | tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:338-352 (`partes_ordenadas`) y :355-361 (`nucleo_tarea`) | informe-revision-3.md |

No hay hallazgos abiertos ni nuevos. Los IMP-1..IMP-5 y NUE-1 de la peticion (ya `corregido` en la ronda 3) siguen en verde: C5c, C6c, C6d, C6e, C9, C9b, C10, C11, C4c, C4d y C4e pasan en la ejecucion integra.

## Ejecucion

- PoC completa, una sola vez, sobre un clon de 2f23459 en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-074-r4rev` (`python -I -B tareas/03-en-revision/TASK-074/poc/poc-f1-16.py`, 3738 s con las mutaciones corriendo a la vez): `RESULTADO: todo como se espera`, `errores del ADR: 0`, carpeta de trabajo borrada. Su salida es identica al `poc-f1-16.log` commiteado tras normalizar CRLF (31649 caracteres cada uno). Es la suite del proyecto; el ADR no tiene otra.
- ToolsaWeb y OpenDockTrading: solo `git clone --no-checkout` de ToolsaWeb a la carpeta temporal (lo hace la PoC) y `git log`/`git show`/`git cat-file` de lectura. Nada ejecutado dentro de ellos.

## Verificacion de NUE-5 (medicion de escrituras del CLI en los tres repos)

- Cifras: 2394 = 1698 (ToolsaWeb) + 163 (TaskCodeTwo) + 533 (OpenDockTrading); por fase 664 = 500+38+126 (antes), 1442 = 939+32+70+42+282+77 (durante: CLI + ciclo), 208 = 159+7+42 (despues), 80 = 68+6+6 (a mano). Cambian el nucleo 23 de antes (21+0+2), 1 de despues y 60 a mano (57+2+1), y 0 de las 1442 de durante. Todo cuadra con el log (:56-69) y con el ADR (:138-156).
- Reproducible: la salida de L es identica a la commiteada en la ejecucion integra, y sobre los tres SHA fijos recalcule por separado la clasificacion de todos los commits `chore(TASK-NNN)` que tocan su `tarea.md`.
- Clasificacion por asunto: muestreados con `git show --stat` `06bd2410b` (peticion de revision, solo informes), `93c9618ac` (tarea en curso), `0c00d1e78` y `1a7ef2ac9` (autorrevision, solo `tarea.md`), `ab6e57a3c` (plan aprobado, en `01-en-diseno`), `22ff81ad4` (coste de revision) y `c051b200f` (cierre): cada uno cae en su grupo. Los asuntos de «durante» son los de `mensajeChore` y del ciclo; ninguno de los 1442 mezcla una replanificacion de la persona (0 de 1442 cambian el nucleo, y no es circular: la lista de asuntos no se ha ajustado para que salga 0, los 60 de «a mano» que cambian el nucleo son todos dependencias, sprint, hito, objetivo, criterios o `## Decisiones de la persona`).
- Lista de secciones excluidas: `Transiciones`, `Coste`, `Resultado`, `Autorrevision`, `Verificacion ligera`, `Verificacion omitida`, `Duracion`, `Revisores no relanzados`, `Rondas extra`, `Ronda extra`. Ninguna define la tarea (objetivo, criterios, escenarios, dependencias y `## Decisiones de la persona` quedan dentro del nucleo, y el ADR explica por que).
- Texto no confiable: el ADR (:549-562) dice que lo excluido es texto libre que el agente puede leer, que un ajeno puede escribir ahi sin apartar la tarea, que el CLI lo entrega como dato no confiable y que ninguna decision del lote sale de ello. Queda cubierto el caso de la nota 1 de la ronda 3.

## Verificacion de NUE-6, NUE-7, NUE-8 (mutaciones propias sobre la seccion C)

Cada mutacion se aplico sobre una copia de la PoC y se ejecuto `seccion_l` + `seccion_c` enteras:

| Mutacion | Casos de C que se ponen en rojo | Lectura |
|---|---|---|
| M4: quitar `arbol == arbol_previo` en la reversion | C4g y C4h | protegido (NUE-6) |
| M2: quitar `por in PERSONAS_LOTE` (los 3 sitios) | C4f y solo C4f | protegido (NUE-7) |
| M5: usar el autor en vez del committer en la reversion | C4i y C4e | protegido (NUE-8) |
| M6: el carril del diff sin `rutas_carril` | C9c y solo C9c | protegido (nota 3 de la ronda 3) |

El ADR (decision 8, :693-714, y `No verificado`, :881-885) exige ahora la misma persona de la lista para revertir y para `ajeno-aceptado`, nombra el committer, la firma verificada en la 2.0 y la limitacion de la PoC (committer y `por` son texto).

## Verificacion de NUE-9

`nucleo_tarea` hashea la lista ordenada de pares sin colapsar. Con el `tarea.md` real de TASK-074, mismo hash base: un segundo `## Objetivo` o `## Criterios de aceptacion` antes del original, un `## objetivo` en minuscula, un segundo `titulo:` antes o despues: cambian el hash. No lo cambian, como debe ser: marcar una casilla, un `estado:` duplicado, un `## Resultado` duplicado o una `## Autorrevision` nueva (excluidas). `## Decisiones de la persona` nueva si lo cambia. Los casos nuevos de P (`## Objetivo` duplicado, `titulo` duplicado, `## Autorrevision`, `## Verificacion ligera`) estan en el log y pasan.

## Notas sin hallazgo

1. Rutas que difieren al parar por arbol distinto: implementado y probado (C5c imprime `difieren: scripts/instalar-malo.sh`, igual que el ADR). `rutas_carril` en el carril del diff: implementado y probado (C9c, M6). Las dos notas de la ronda 3 quedan resueltas.
2. Cifras nuevas del ADR: 55 cortes (log :139-140 y ADR :253-255 y :371), 29 inyecciones de C (29 filas `C*` en la tabla frente a 24 antes: C4f, C4g, C4h, C4i y C9c), hash del manifiesto `baa00915b31d` (log :189, ADR :415): cuadran.
3. Detalle de redaccion, sin consecuencia: el ADR (:150-151) describe los 80 de «a mano» como replanificaciones de la persona, «depende de TASK-NNN» y sprint, pero ~13 son escrituras del ciclo con asuntos que la regex no reconoce («constancia de la autorrevision», «resultado, coste y nota de MEN-N»). No cambian el nucleo salvo 2 con `## Decisiones de la persona`; el efecto es que las 1442 son un suelo, no un techo, y la conclusion (0 cambios) no se debilita.
4. Que el CLI entregue las secciones excluidas al agente marcadas como no confiables es diseño de la 2.0, no probado por la PoC; el ADR lo presenta como regla de diseño, no como medido.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-074/poc/poc-f1-16.py` y `poc-f1-16.log`: mi parte del diff (excluida del embebido); sin ellos no se reproduce ni se muta.
- `tareas/03-en-revision/TASK-074/revision/informe-revision-3.md`: lista de NUE a verificar y sus mutaciones.
- `git log`/`git show --stat`/`git cat-file` de solo lectura en ToolsaWeb (`93c9618ac`), TaskCodeTwo (`02142712`) y OpenDockTrading (`fab8d7e05`): muestreo de la clasificacion de NUE-5.

# Informe de revision — TASK-010 (ronda 1)

- Commit revisado: 43678e65ce225547cfdef2586a1fadce4baf0438
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | tareas/03-en-revision/TASK-010/poc/poc-f1-9.sh:195 | implementacion |
| MEN-2 | MENOR | abierto | docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md:262 | implementacion |
| MEN-3 | MENOR | abierto | docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md:340 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Sin CRITICO ni IMPORTANTE. Propuesta de veredicto: aprobada-con-correcciones (tres MENOR, ninguno bloquea).

### MEN-1 — El log de la PoC no es reproducible byte a byte (orden de empates)
- Donde: tareas/03-en-revision/TASK-010/poc/poc-f1-9.sh:195
- Que pasa: las secciones se ordenan por total descendente a partir de `set().union(...)`; las que empatan (`Criterios de aceptacion` y `Objetivo`, 404/96/93; `Autorrevision`, `Descartada*`, etc.) salen en un orden que depende de la semilla de hash de cadenas de Python. Las cifras son las mismas.
- Reproduccion: ejecutado el guion en `$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-010-r1` (sale 0, `OK: A..G como se espera, mutaciones detectadas, solo GET`) y `diff` contra `poc-f1-9.log`: solo difieren la linea `HEAD actual` y el orden de las filas empatadas (`Criterios de aceptacion` y `Descartada*` cambian de sitio).
- Impacto: quien repita la PoC y compare el log con `diff` ve diferencias que no lo son; la tarea promete que «la medida se pueda repetir».
- Correccion: ordenar con clave estable, `key=lambda n: (-total(n), n)`, y regenerar el log.

### MEN-2 — La PoC E no comprueba la tabla de secciones ni la linea «Recuento»
- Donde: docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md:262
- Que pasa: la sentencia «cada veredicto cita su cifra» y las cuatro copias mutadas solo cubren la tabla de campos (punto 3). La tabla de secciones del punto 4 (`## Resultado` 159 / 43 / 9, `## Duracion`, `## Descartada…`, etc.) y la frase `Recuento: 13 se conservan, 7 …` (linea 248) no se cotejan con la medida; hoy los valores son correctos, pero nada los protege.
- Reproduccion: copia del ADR en `.../task-010-r1/m/<caso>/` con repo minimo (`git archive HEAD docs .github tareas/...`) y mutacion, y el guion contra esa copia:
  - `| \`## Resultado\` | 159 / 43 / 9` cambiado a `999 / 43 / 9`: sale 0.
  - `| \`## Descartada…\` | 6 / 2 / 0` cambiado a `7 / 2 / 0`: sale 0.
  - `| \`## Duracion\` | 23 / 33 / 9` cambiado a `99 / 33 / 9`: sale 0.
  - `13 se conservan,` cambiado a `14 se conservan,`: sale 0 (el guion imprime su propio recuento, pero no lo compara con la prosa).
  - `PoC A: 0 de 593` cambiado a `0 de 999`: sale 0.
  - Controles que si fallan: veredicto de `tipo` cambiado a «se retira» (FALLO en E), `RF-51` cambiado a `RF-99` (FALLO en G) y la seccion de Spec Kit «Dependencies & Execution Order» inventada (FALLO en F).
- Impacto: las cifras de secciones y el recuento pueden desviarse del log en una edicion futura sin que la PoC lo note. No es una afirmacion falsa hoy (la medida en el log y en mi ejecucion coincide: 159/43/9, 92/44/10, 33/43/9, 23/33/9, 6/2/0, 162/44/10, y 13/7/2/4).
- Correccion: o bien extender E con marcadores `secciones-tarea` y la misma comprobacion «fila = cifra medida», mas comparar `Recuento` con el diccionario de veredictos; o bien decir en el ADR (linea 417 y PoC E) que la mutacion solo protege la tabla de campos.

### MEN-3 — «Sin cambios» respecto a `planTemplate` no es exacto
- Donde: docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md:340
- Que pasa: el punto 7 dice que la plantilla es el esqueleto que genera la 0.16 «sin cambios, mas `## Troceo`». La 0.16 (`src/commands/plan.ts`, `planTemplate`, y `LINEA_PLANTILLA_DURACION` en `src/core/duracion-estimada.ts:14`) emite `Duracion estimada:  min (de la rama abierta al cierre con merge; senales: …)`, no `Duracion estimada: N min`, y la cabecera de origen es una de tres frases segun el numero de roles. La PoC D solo coteja titulos `##` y la presencia de una linea «Duracion estimada».
- Reproduccion: lectura de `plan.ts` lineas 230-275 y `duracion-estimada.ts:14-15`, frente al bloque de `plantilla-plan` del ADR.
- Impacto: bajo. Los titulos que consume la 0.16 coinciden (PoC D: ninguno falta, ninguno sobra salvo `Troceo`) y `approve` exige un numero, que ni una plantilla ni la otra traen. Quien implemente F2-5 podria creer que reproduce byte a byte el esqueleto de la 0.16 y que el detector de «esqueleto sin rellenar», que compara por lineas, lo seguira reconociendo.
- Correccion: cambiar «sin cambios» por «mismos titulos y orden; la linea de duracion se simplifica a `Duracion estimada: N min`», o decir que el detector de esqueleto es cosa de F2-5.

## Verificacion realizada

- Puerta: no hay build ni linter propios (ADR y guion). `bash .github/scripts/comprobar-repo.sh`: `Tareas: 93; enlaces relativos: 71; fallos: 0`.
- Reproduccion de la PoC: `poc-f1-9.sh` en `$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-010-r1`, con las mismas rutas de los tres repos y `task.ts` de la 0.16.1. Sale 0; mismas cifras que el log (solo difieren `HEAD actual` y el orden de empates, MEN-1). Solo `GET` a GitHub (cuatro llamadas registradas).
- Los 22 campos: `TASK_FIELD_ORDER` de `task.ts:107-130` contiene exactamente los 22 de la tabla (ni uno mas ni uno menos); recuento 13 / 7 / 2 / 4 verificado a mano contra la tabla. Coherentes con F1-1..F1-8: `estado` como campo y sin carpeta (F1-1), `asignado_a` ligado a la toma (F1-2), aprobacion firmada con `Plan-Blob` y `carril` (F1-5, F1-6), texto no confiable `Z2`/`A10` (F1-8).
- `ultimo_commit_revisado`: `grep` en el plugin 0.16.1 solo lo escribe a `null` (`new.ts:276`, `troceo.ts:213`); `review.ts:16` dice que se actualizaria al terminar una revision, pero ningun codigo lo hace. El ADR («la 0.16 solo lo pone a `null`») es exacto.
- Plantilla de `tarea.md` extraida del ADR: carga con PyYAML (`python -I`), 21 claves, `roles` como mapas de flujo, `creado` como texto. Casos extra fuera del guion, con titulo `C:\\ruta\\` (acabado en `\`), `a\"b` y `tab\t`: devuelven `C:\ruta\`, `a"b` y tabulador; y `tipo` duplicado: PyYAML acepta y se queda con `hotfix`, tal como dice el ADR (por eso el CLI debe rechazarlas).
- Spec Kit: las seccion citadas existen en las plantillas descargadas por la PoC; ademas `**Feature Branch**: [###-feature-name]` y `specs/[###-feature]/` (afirmacion de D1c) y los `[NEEDS CLARIFICATION]` y `FR-/SC-NNN` estan en las plantillas. La ida y vuelta real figura como no verificada en el ADR, y asi consta.
- IDs de requisitos: RR-17, RR-19, RF-07 (9 + 2 archivadas), RF-34 (R, 0 usos), RF-71 (E, cubierto por RF-07) y H11 coinciden con `REQUISITOS-FUNCIONALES.md` y la auditoria; F1-9 existe en el plan con criterios «Plantillas» y «qué campos se conservan» (ambos cubiertos). Los 12 IDs y las 21 tareas las comprueba el guion (G).
- Plan aprobado y divergencias: el ADR sigue D1a..D5a y documenta las cuatro divergencias (22 campos clasificados, `estado` una sola vez, `## Duracion` y `## Descartada`, commits fijos). Criterios de `tarea.md`: los tres cubiertos (plantillas, tabla de campos, requisitos citados).
- Sin hallazgos en: tabla de campos, plantilla de `tarea.md`, contrato del texto, correspondencia con Spec Kit y lo marcado no verificado.

## Lecturas fuera del contexto permitido

- `C:\Users\nullcad2025\.claude\plugins\cache\taskcode-marketplace\taskcode-plugin\0.16.1\src\core\task.ts`: lo exigia la peticion del orquestador (los 22 de `TASK_FIELD_ORDER`).
- `...\0.16.1\src\commands\plan.ts` y `...\src\core\duracion-estimada.ts`: comparar la plantilla del plan con `planTemplate` (MEN-3).
- `...\0.16.1\src\commands\review.ts`, `new.ts` y `troceo.ts` (grep): saber quien escribe `ultimo_commit_revisado` y `agente_revisor`.
- `...\0.16.1\skills\task-workflow\revision.md`: procedimiento comun de revision.
- `tareas/03-en-revision/TASK-010/poc/poc-f1-9.sh` y `poc-f1-9.log`: reproducir la PoC.
- `docs/adr/F1-1`, `F1-2`, `F1-5`, `F1-6`, `F1-8`, `docs/plan/PLAN-TASKCODE-2.md`, `docs/plan/REQUISITOS-FUNCIONALES.md`, `docs/auditoria/AUDITORIA-EXTERNA-2026-11-13.md`: comprobar la coherencia de los veredictos y los IDs citados.
- `.github/scripts/comprobar-repo.sh` (ejecutado), y `/c/Users/nullcad2025/Documents/sources/Personal/{ToolsaWeb,OpenDockTrading,TaskCodeTwo}` (solo `git archive` de lectura, hecho por el propio guion).

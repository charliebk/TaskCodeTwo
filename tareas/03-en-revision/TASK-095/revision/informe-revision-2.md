# Informe de revision — TASK-095 (ronda 2)

- Commit revisado: 63e9c84280307befcf399d9343db83e84189c140
- Revisor: code-quality-reviewer (agente general-purpose independiente, modelo haiku; verificacion acotada)
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-19-memoria-del-proyecto.md (PoC 1 «Dato real», decision 6, tabla RF-78/RF-79) | plan |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-19-memoria-del-proyecto.md (PoC 2 «6 ms», decision 2, tabla F10-6) | plan |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-19-memoria-del-proyecto.md:111-117 y 254-256 (decision 3, PoC 1, tabla RF-78) | implementacion |

**IMP-3 (nuevo, introducido por la correccion de IMP-1).** Las cifras que anade la correccion no se reproducen con la regla que el propio ADR enuncia.

- ADR: «467 de 482 atribuidos y 15 sin atribuir, todos `docs:`».
- Reproducido sobre e1bf456 (482 commits): regla 1 (asunto `(TASK-NNN)`, tareas terminadas) = 362, coincide. Mas la regla del rango `merge^1..merge^2` de los 30 merges `feature/task-NNN` de tareas terminadas, incluido el merge, deduplicado: **447 atribuidos y 35 sin atribuir**, no 467 y 15.
- Los 35 sin atribuir: 9 `docs:` y 26 `chore:` (16 `chore(taskctl): import de N tarea(s)`, `chore: alta de 18 tareas`, `chore: dependencias…`, `chore: ignorar .iml`, `chore: config de taskctl`, `chore: estructura inicial`, y 3 de TASK-095, no terminada). «Todos `docs:`» es falso.
- Otras cifras de «Dato real»: «18 `merge(feature)`» son **30**; «unos 34 `docs(adr): F1-N corrige`» son **36** (55 `docs(adr)` en total).
- Correccion sugerida: recalcular con la regla fijada (447/35 o la variante exacta) y corregir 30 y 36.

## Verificaciones realizadas

- (a) Atribucion: 482 commits en e1bf456. Regla 1 = 362. Regla 1 + rango = 447. No reproduce 467/15.
- (b) ADR por merge: para las 19 tareas F1- terminadas, `git diff --name-only merge^1...merge^2` contiene `docs/adr/F1-*` en 18 de 18 con ADR propio. TASK-001 no tiene ADR. Reproducido.
- (c) Etiquetas: 112 `tarea.md` en e1bf456, las 112 con `etiquetas: []`. Reproducido.
- IMP-1 e IMP-2: el diff de 63e9c84 fija la regla en la decision 3 y reformula el «6 ms» (6-7 ms dentro del proceso; proceso completo 177-289 ms, 200 ms no demostrados). Se dan por corregidos.
- Sin hallazgos nuevos CRITICO.
- Suite: `cargo xtask verificar --perfil rapida` en un clon temporal en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-095-r2\` (63e9c84): EXIT 0, verificacion Rapida en verde (fmt y clippy limpios; E2E contra GitHub y GitLab ignorados por credenciales). La tarea solo toca docs.

## Reproduccion

Solo lectura sobre e1bf456: listado de 482 commits, asuntos `(TASK-NNN)` cruzados con las 30 carpetas de `tareas/04-terminadas/`, los 30 merges `feature/task-NNN`, `git rev-list merge^1..merge^2` por merge, union deduplicada con prioridad a la regla 1, y clasificacion por tipo de asunto de los 35 no atribuidos.

Comprobacion del implementador tras el informe (misma regla, solo asunto): con cualquier tarea, 449 atribuidos y 33 sin atribuir (24 `chore`, 9 `docs`); solo terminadas, 447 y 35; entre los 120 huerfanos de la regla 1, 30 `merge(feature)`, 36 `docs(adr): F1-N corrige` y 16 `chore(taskctl): import`. La cifra 467 del ADR salia de un `git log --grep`, que busca tambien en el cuerpo del mensaje.

## Lecturas fuera del contexto permitido

- `task-workflow/revision.md` del plugin (reglas de veredicto y plantilla), para fijar el veredicto.
- `.taskcode/config.yml` en 63e9c84 (comando de verificacion).
- `revision/informe-revision-1.md` y `peticion-revision-2.md` (contexto de la ronda).

## Notas (no son hallazgos)

- MEN-2 de la ronda 1 se marco `corregido`, pero el criterio 2 de `tarea.md` sigue con el texto original; el ADR lo deja a Carlos en «Textos que cambian»: decision pendiente, no fallo de la correccion.
- Carpeta temporal borrada; `git status` del repo limpio.

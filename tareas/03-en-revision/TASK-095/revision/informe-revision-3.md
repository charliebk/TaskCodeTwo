# Informe de revision — TASK-095 (ronda 3)

- Commit revisado: 1e73ccf4fc6afe95b42d1e8d87a46f8febf857bb
- Revisor: code-quality-reviewer (agente general-purpose independiente, modelo haiku; verificacion acotada a IMP-3, ronda extra decidida por Carlos)
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-19-memoria-del-proyecto.md (PoC 1 «Dato real», decision 6, RF-78/RF-79); no tocado por 1e73ccf | plan |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-19-memoria-del-proyecto.md (PoC 2 «6 ms», decision 2, F10-6); no tocado por 1e73ccf | plan |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-19-memoria-del-proyecto.md:111-126, 263 y 404 (decision 3, PoC 1, RF-78) | implementacion |

Sin hallazgos nuevos. Ninguno es CRITICO ni lo introduce la correccion de 1e73ccf.

IMP-3 queda resuelto: el diff de 1e73ccf toca solo el ADR (18 inserciones, 10 borrados) y todas sus cifras se reproducen con git. No quedan «467», «15 sin atribuir», «18 merge(feature)» ni «34 docs(adr)» vigentes; la unica mencion de 467 es la nota historica de la linea 124. IMP-1 e IMP-2 no estan en el diff de 1e73ccf y se mantienen corregidos segun la ronda 2.

## Reproduccion

Solo lectura sobre e1bf456, mirando unicamente el asunto (%s), sin cuerpos ni --grep. Suite no ejecutada (la peticion lo indica: el diff desde 63e9c84 solo toca docs y alli estaba en verde).

1. Commits en e1bf456: 482.
2. Regla 1 (asunto con `(TASK-NNN)`): cualquier tarea 364; solo las 30 terminadas 362.
3. Merges con dos padres: 32; 30 `merge(feature): feature/task-NNN` (todos de terminadas) y 2 `merge(TASK-024)` con rango de un solo commit ya atribuido.
4. Regla 1 + rango `merge^1..merge^2` + el propio merge, deduplicado: solo terminadas 447 atribuidos y 35 sin atribuir (16 `chore(taskctl)`, 10 `chore`, 9 `docs`); cualquier tarea 449 y 33 (24 `chore`, 9 `docs`). Coincide con el ADR.
5. Los 120 no atribuidos por la regla 1 a terminadas: 30 `merge(feature)`, 36 `docs(adr): F1-N corrige`, 16 `chore(taskctl): import`, 19 `docs(adr)` distintos, 9 `docs`, 8 `chore` y 2 `chore(TASK-095)`. Coincide con el ADR.
6. 467: solo en la nota historica de la linea 124 del ADR; el resto de menciones estan en informes y peticiones de revision, historicas.
7. Limitacion: la regla «asunto o rama» de la decision 3 se midio solo por asunto; en e1bf456 no hay merges `hotfix` y los 32 merges quedan cubiertos, asi que la cifra no cambia. El origen de 467 (un `--grep` sobre el cuerpo) no se comprobo porque la peticion prohibia mirar cuerpos; es nota historica.

## Lecturas fuera del contexto permitido

- `revision/informe-revision-2.md`: origen de IMP-1 a IMP-3 y de las cifras.
- Nombres de `tareas/04-terminadas/` y de `revision/`.
- `git show 1e73ccf -- docs/adr/F1-19-memoria-del-proyecto.md`.

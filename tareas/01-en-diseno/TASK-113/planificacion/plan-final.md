# Plan — TASK-113: Puerta de valor F10: informe, datos y descarte del H10

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 60 min

Tarea solo de documentos (sin codigo), en dos partes sobre la rama de TASK-113.

**(a) Evidencia** en `docs/mediciones/fase10-puerta-valor/`:
1. `.gitattributes`: regla `docs/mediciones/** -text`. Este clon tiene `core.autocrlf=true` (comprobado): sin la regla, el checkout cambiaria los saltos de linea y el sha256 dejaria de coincidir.
2. Copia byte a byte (Copy-Item, nunca editor) de `informe.md` (sha256 b50f22cb…) y `corridas.tsv` (b98f1769…) desde `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\puerta-valor\`. El informe no se toca: el contexto que le falte va al README.
3. `README.md` nuevo: que se midio y como, las 19 columnas de `corridas.tsv`, el criterio, el veredicto y la decision de Carlos (descartar la Fase 10).
4. `docs/INDEX.md`: seccion `## Mediciones` antes de `## Tareas terminadas` (finish inserta bajo esa cabecera y conserva el resto).

**(b) Descarte del H10**:
5. Aviso unico «**Descartado el 2026-11-16** (TASK-113): la puerta de valor dio NO-GO y Carlos descarto la Fase 10; ver [informe](../mediciones/fase10-puerta-valor/informe.md)», sin borrar texto, en:
   - `docs/plan/REQUISITOS-FUNCIONALES.md`: fila RR-20 y seccion RF-77..RF-84;
   - `docs/plan/PLAN-TASKCODE-2.md`: fila RR-20, seccion «Fase 10» y punto 6 del orden;
   - `docs/adr/F1-19-memoria-del-proyecto.md`: aviso en cabecera (el campo de estado no se cambia);
   - `docs/adr/README.md`: fila F1-19 dice «descartada»;
   - `docs/plan/AMPLIACION-2026-11-16.md`: aviso en cabecera.
6. `git rm -r` de `tareas/00-planificadas/TASK-096` .. `TASK-112`.
7. Zona manual de `tareas/ROADMAP.md`: quitar `### H10 — Memoria del proyecto` y su linea.
8. `sprint: 1` en TASK-113 (hoy `sprint: 0` genera un «H0 — H0» vacio; H1 es el hito de decisiones y esta tarea cierra la de F1-19).
9. Regenerar con `taskctl roadmap --escribir` y `taskctl board --escribir`; despues `taskctl roadmap --comparar` y `taskctl doctor`.
10. Barrido de referencias colgantes solo en documentos vivos. Se dejan a proposito: `04-terminadas/TASK-095/**` y CHANGELOG (historia), `docs/auditoria/*`, F1-9, F1-17 y F1-18 (citan IDs de la 0.16, OpenDockTrading o ToolsaWeb, no estas tareas).

## Lo que el rol no cubrio

- Riesgos, pruebas y dominio con un solo rol: la tarea es de documentos y borrado; la revision por pares reproduce cada criterio.
- Si `taskctl new` podria reutilizar los IDs 096..112: TASK-113 ya existe por encima; se comprueba en la implementacion como reserva los IDs.
- Si los hooks de la 0.16 bloquean el `git rm` de tareas planificadas o la edicion de la zona manual: se comprueba en la rama.
- Las memorias fuera del repo (MEMORY.md y engram) se actualizan aparte, fuera de esta tarea.

## Riesgos aceptados y que los contiene

- Romper el sha256 por saltos de linea: lo contiene la regla `-text` y la comprobacion sobre el blob (`git show HEAD:<ruta>`), no solo sobre la copia de trabajo.
- Que `finish` reescriba INDEX y se pierda la seccion manual: `appendEntry` inserta bajo `## Tareas terminadas` y conserva el resto; se comprueba tras el cierre.
- RF-77..RF-84 no se renumeran: quedan marcados para no romper referencias.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):
Sin tests de codigo (no hay codigo). Comprobaciones reproducibles por criterio:
- C1: `sha256sum` de la copia y de `git show HEAD:docs/mediciones/fase10-puerta-valor/{informe.md,corridas.tsv}` = b50f22cb… / b98f1769….
- C2: el informe contiene «Criterio fijado antes de medir», «Resultado (medianas)», «NO-GO», «Uso real del índice» y «Factores que contaminan».
- C3: el README nombra las 19 columnas de la cabecera de `corridas.tsv`.
- C4: `docs/INDEX.md` enlaza `mediciones/fase10-puerta-valor/informe.md` y la ruta existe.
- C5: no existe ninguna carpeta `tareas/*/TASK-09[6-9]|TASK-10[0-9]|TASK-11[0-2]` y el grep de esos IDs en `tareas/0[0-3]*` sale vacio.
- C6: grep de esos IDs y de `H10 —` en `tareas/ROADMAP.md` y `docs/BOARD.md` vacio; `taskctl roadmap --comparar` sale con 0.
- C7: «Descartado el 2026-11-16» aparece en cada uno de los 7 sitios y cada enlace resuelve a un fichero existente.

## Lo que necesita decision de una persona

Decidido por Carlos el 2026-11-16: descartar la Fase 10; borrar las tareas y el H10 del roadmap; marcar como descartados (sin borrar) los documentos. Aprobo este plan («Aprobado»). Decisiones menores tomadas por quien orquesta: `sprint: 1` para TASK-113 y no cambiar el campo de estado del ADR F1-19 (solo aviso en cabecera).

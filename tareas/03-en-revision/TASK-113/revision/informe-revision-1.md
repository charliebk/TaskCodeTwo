# Informe de revision — TASK-113 (ronda 1)

- Commit revisado: faaaba2f9fcda500f56fc13d74d758f762ffd4be
- Revisor: code-quality-reviewer
- Veredicto: PENDIENTE (escribelo con: taskctl veredicto TASK-113 aprobada | aprobada-con-correcciones | cambios-solicitados)

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | tareas/ROADMAP.md:71 | implementacion |
| MEN-2 | MENOR | abierto | docs/BOARD.md:77 | implementacion |
| MEN-3 | MENOR | abierto | docs/mediciones/fase10-puerta-valor/informe.md:324 | especificacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Debajo, la reproduccion de cada hallazgo, o "sin hallazgos" explicito.

Sin CRITICOS ni IMPORTANTES. Tarea solo de documentos: no hay suite de codigo; se
reprodujo cada criterio sobre un clon limpio (core.autocrlf=true) de la rama en el commit 8b1e673.
Los dos commits de contenido (75574ec, faaaba2) son los revisados; 8b1e673 solo anade la peticion.

### Criterios reproducidos

- C1 sha256: informe.md b50f22cb...7e2b y corridas.tsv b98f1769...67f1 coinciden en los tres sitios
  (original en pruebas-claude/puerta-valor, blob `git show HEAD:...` y checkout del clon con autocrlf=true).
  `git check-attr text` da `unset` (regla `-text` efectiva).
- C2 informe: contiene criterio fijado antes de medir, medianas por repo y modo, NO-GO, uso real del indice y
  factores que contaminan. Recalculado desde corridas.tsv: mediana de tokens ToolsaWeb 563116 / 826825
  (563 k / 827 k), denegadas 56 (indice) frente a 10 (grep), 26 de 32 corridas de indice con intento de grep
  (13 por repo), las 32 usaron buscar.py (mediana 3). Todo coincide salvo MEN-3.
- C3 README: la cabecera de corridas.tsv tiene 19 columnas y las 19 aparecen en la tabla del README
  (comprobado una a una, incluida `denegadas`); 65 lineas, todas con 19 campos (64 corridas + cabecera).
- C4 enlaces: los 10 enlaces nuevos (INDEX x3, adr/README, F1-19, PLAN, REQUISITOS, AMPLIACION, README->informe y
  corridas) resuelven a un fichero existente.
- C5 tareas: ninguna carpeta TASK-096..112 en tareas/*; grep de esos IDs en todo el arbol (salvo la carpeta de
  TASK-113) vacio. Ninguna tarea viva depende de ellas.
- C6 H10: `H10` solo aparece en la auditoria externa (otro H10, hallazgo), citas historicas de TASK-095, el aviso de
  AMPLIACION (historico) y el titulo de TASK-113. ROADMAP y BOARD sin hito H10 ni esas tareas.
- C7 marcas: "Descartado el 2026-11-16" con enlace al informe en PLAN (fila RR-20, Fase 10, punto 6), REQUISITOS
  (fila RR-20 y seccion RF-77..84), ADR F1-19, adr/README y AMPLIACION. RF-77..84 no se renumeran.
- `taskctl doctor`: 0 errores. Quedan menciones de F10-N/RR-20 dentro del propio ADR F1-19, AMPLIACION y la
  seccion Fase 10, todas bajo el aviso de descartado: aceptable.
- Mutacion: no aplica (sin codigo). La proteccion `-text` se comprobo por el efecto (sha256 igual con autocrlf=true).

### MEN-1 — ROADMAP desactualizado respecto al estado real
`taskctl roadmap --comparar` da 1 diferencia: TASK-113 figura "en curso" y taskctl genera "en revision"
(la tarea ya esta en 03-en-revision). Se regenera solo al cerrar; basta `taskctl roadmap --escribir`
tras la revision si se quiere el estado exacto.

### MEN-2 — BOARD con la tarea en "En curso"
docs/BOARD.md:77 lista TASK-113 en "En curso (02-en-curso)"; esta en 03-en-revision. Mismo origen que MEN-1;
`taskctl board --escribir` lo deja bien.

### MEN-3 — Dato del informe original impreciso (no corregible por la regla de byte a byte)
informe.md:324 dice que el indice se uso "de 1 a 6 veces"; en corridas.tsv W4-indice-r2 tiene `usos_indice` = 7.
La mediana (3) es correcta. Como el informe debe ser copia exacta, lo razonable es anotarlo en el README
(una linea: "el maximo real es 7") o dejarlo asi y aceptarlo; no cambia el veredicto.

## Lecturas fuera del contexto permitido

ninguna

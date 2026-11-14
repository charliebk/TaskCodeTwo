# MENORES pendientes — TASK-015

MENORES abiertos de la ronda 2 que no se corrigieron en la verificacion ligera (`taskctl verificar-menores`). Son backlog de la tarea: no abren otra ronda ni impiden `finish`.

| ID | Fichero | Motivo |
|---|---|---|
| MEN-6 | tareas/03-en-revision/TASK-015/poc/poc-f1-13.py:209 | no se corrigio en la verificacion ligera |

Nota de cierre: MEN-6 si esta corregido en el commit b6e0181 (normalizacion y rechazo de `.`/`..` codificados, NEG-12..16); `verificar-menores` no reconoce los ficheros de la carpeta de la tarea (`poc/`).

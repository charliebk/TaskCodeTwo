# MENORES pendientes — TASK-073

MENORES abiertos de la ronda 2 que no se corrigieron en la verificacion ligera (`taskctl verificar-menores`). Son backlog de la tarea: no abren otra ronda ni impiden `finish`.

| ID | Fichero | Motivo |
|---|---|---|
| MEN-9 | tareas/03-en-revision/TASK-073/poc/poc-f1-15.py:1273 | no se corrigio en la verificacion ligera |
| MEN-10 | tareas/03-en-revision/TASK-073/poc/poc-f1-15.py:208 | no se corrigio en la verificacion ligera |

Nota de cierre: MEN-9 (cobertura por atomo, 50/50) y MEN-10 (ruta en minusculas, caso `.Claude/settings.json`) si estan corregidos en el commit 71bc716; `verificar-menores` no reconoce los ficheros de la carpeta de la tarea (`poc/`).

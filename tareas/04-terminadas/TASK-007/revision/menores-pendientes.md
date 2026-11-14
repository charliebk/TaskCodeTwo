# MENORES pendientes — TASK-007

MENORES abiertos de la ronda 2 que no se corrigieron en la verificacion ligera (`taskctl verificar-menores`). Son backlog de la tarea: no abren otra ronda ni impiden `finish`.

| ID | Fichero | Motivo |
|---|---|---|
| MEN-7 | tareas/03-en-revision/TASK-007/poc/poc-f1-6.sh:98 | no se corrigio en la verificacion ligera |
| MEN-8 | tareas/03-en-revision/TASK-007/poc/poc-f1-6.sh:285 | no se corrigio en la verificacion ligera |

**Nota posterior al cierre (2026-11-14):** MEN-7 y MEN-8 **si estan corregidos**, en el commit `1797712` (cambia `poc/poc-f1-6.sh` y `poc/poc-f1-6.log` de esta tarea junto al ADR). `taskctl verificar-menores` 0.16.1 no los reconocio, probablemente porque no cuenta como corregidos los ficheros que viven dentro de la carpeta de la tarea. No queda nada pendiente.

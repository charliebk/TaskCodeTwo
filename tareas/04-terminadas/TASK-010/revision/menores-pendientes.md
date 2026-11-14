# MENORES pendientes — TASK-010

MENORES abiertos de la ronda 1 que no se corrigieron en la verificacion ligera (`taskctl verificar-menores`). Son backlog de la tarea: no abren otra ronda ni impiden `finish`.

| ID | Fichero | Motivo |
|---|---|---|
| MEN-1 | tareas/03-en-revision/TASK-010/poc/poc-f1-9.sh:195 | no se corrigio en la verificacion ligera |

**Nota de quien orquesta (2026-11-14):** MEN-1 **si esta corregido**, en el commit `9522391` (cambia `poc/poc-f1-9.sh` y su log junto al ADR). `taskctl verificar-menores` 0.16.1 no reconoce como corregidos los ficheros que viven dentro de la carpeta de la tarea. No queda nada pendiente.

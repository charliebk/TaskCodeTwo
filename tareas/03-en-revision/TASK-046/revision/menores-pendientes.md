# MENORES pendientes — TASK-046

MENORES abiertos de la ronda 3 que no se corrigieron en la verificacion ligera (`taskctl verificar-menores`). Son backlog de la tarea: no abren otra ronda ni impiden `finish`.

| ID | Fichero | Motivo |
|---|---|---|
| MEN-7 | crates/tc2/src/confianza/cierre.rs:`calcular` (`pendientes.extend(indice.por_nombre(orden))`) | menores de la ronda 3: test de raro en la orden y de mayusculas en por_nombre, limites del cierre en la guia |

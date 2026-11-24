# MENORES pendientes — TASK-148c

MENORES abiertos de la ronda 1 que no se corrigieron en la verificacion ligera (`taskctl verificar-menores`). Son backlog de la tarea: no abren otra ronda ni impiden `finish`.

| ID | Fichero | Motivo |
|---|---|---|
| MEN-1 | crates/tc2/tests/comun/http_mock.rs:212 | MEN-1 solo cabe en http_mock.rs con un test en su propio modulo, que se replicaria en los ~40 binarios que declaran mod comun; el sitio de los tests del soporte es soporte.rs, fuera del fichero citado: queda en el backlog |

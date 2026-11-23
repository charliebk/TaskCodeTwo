# MENORES pendientes — TASK-148a

MENORES abiertos de la ronda 1 que no se corrigieron en la verificacion ligera (`taskctl verificar-menores`). Son backlog de la tarea: no abren otra ronda ni impiden `finish`.

| ID | Fichero | Motivo |
|---|---|---|
| MEN-1 | crates/tc2/tests/actualizar_cli.rs:265 | MEN-1 no cabe en el fichero citado: lanzar una copia del binario con la base comun exige una variante orden_tc2_de(ruta) en tests/comun/lanzador.rs; queda en el backlog de la tarea |

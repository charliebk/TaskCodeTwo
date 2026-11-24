# MENORES pendientes — TASK-141

MENORES abiertos de la ronda 1 que no se corrigieron en la verificacion ligera (`taskctl verificar-menores`). Son backlog de la tarea: no abren otra ronda ni impiden `finish`.

| ID | Fichero | Motivo |
|---|---|---|
| MEN-3 | crates/tc2/src/importar/mod.rs:247 | MEN-3 (traduccion de lectura::Motivo repetida en plan e import) exige tocar plan/mod.rs y el catalogo, fuera del fichero citado; va al backlog, candidato natural para TASK-150 (utilidades de E/S). MEN-4 anade la variante publica NoListo::Plan para motivos desconocidos de validar, hoy inalcanzable |

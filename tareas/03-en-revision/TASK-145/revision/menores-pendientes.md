# MENORES pendientes — TASK-145

MENORES abiertos de la ronda 1 que no se corrigieron en la verificacion ligera (`taskctl verificar-menores`). Son backlog de la tarea: no abren otra ronda ni impiden `finish`.

| ID | Fichero | Motivo |
|---|---|---|
| MEN-3 | crates/tc2/src/desinstalar/mod.rs:528 | MEN-3 (ruta repetida en el fallo de desinstalar al reescribir un fichero cambiado) es cosmetico y su arreglo exige tocar el catalogo, fuera del fichero citado; va al backlog |

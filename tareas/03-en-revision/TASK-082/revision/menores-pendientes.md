# MENORES pendientes — TASK-082

MENORES abiertos de la ronda 1 que no se corrigieron en la verificacion ligera (`taskctl verificar-menores`). Son backlog de la tarea: no abren otra ronda ni impiden `finish`.

| ID | Fichero | Motivo |
|---|---|---|
| H2 | crates/tc2/src/git/diff.rs:96 | no se corrigio en la verificacion ligera |

Notas de quien orquesta (`verificar-menores` solo mira si cambio el fichero citado, y aqui se equivoca en los dos sentidos):

- H2 si esta corregido: el hallazgo pedia un test de comportamiento, no un cambio en `git/diff.rs`; es `tests/solape.rs::h2_renombrado_y_borrado_dan_los_dos_nombres` (commit 5bb2855), y quitar `--no-renames` lo pone rojo.
- R1-02 NO esta corregido aunque `verificar-menores` lo marque: `solape.rs` cambio por H3, no por R1-02. En la toma nueva la lista de tareas en curso se lee antes de aplicar la toma y no trae el `quien` propio; arreglarlo exige tocar `toma/`, fuera de los ficheros citados. Pasa a TASK-128, junto con mover `AVISO_Y_MAS` y `NOTA_Y_MAS` al catalogo y el test intermitente de progreso.

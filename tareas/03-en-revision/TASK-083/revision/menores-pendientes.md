# MENORES pendientes — TASK-083

MENORES abiertos de la ronda 1 que no se corrigieron en la verificacion ligera (`taskctl verificar-menores`). Son backlog de la tarea: no abren otra ronda ni impiden `finish`.

| ID | Fichero | Motivo |
|---|---|---|
| R1-01 | crates/tc2/src/git/mod.rs:143 | no se corrigio en la verificacion ligera |

Nota de quien orquesta: R1-01 si esta corregido. El hallazgo pedia un test con commits reales, no un cambio en `git/mod.rs`; ese test es `tests/estimacion.rs::r1_01_un_commit_de_develop_cambia_el_real` (commit 7be0767), y la mutacion `FORMATO_HISTORIA = "--format=%H"` lo pone rojo (45 en vez de 55). `verificar-menores` lo deja aqui porque solo mira si cambio el fichero citado.

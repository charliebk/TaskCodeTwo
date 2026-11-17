# Informe de revisión — TASK-034 (ronda 2)

- Commit revisado: f31d627 (codigo de 7fffc65)
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| 1 (ronda 1) | IMPORTANTE | corregido | crates/tc2/src/ciclo/mod.rs:109 (`fetch` previo en `paso_descartar`) | implementacion |
| 2 (ronda 1) | MENOR | corregido | crates/tc2/tests/ciclo.rs `c5_sin_publicar_o_con_head_suelto_conserva_el_worktree` | implementacion |
| 3 (ronda 1) | MENOR | corregido | docs/adr/F1-4 (adenda TASK-034); `c9_`/`c10_` | implementacion |
| 4 (ronda 1) | MENOR | corregido | crates/tc2/src/estado/tareas.rs (`por_estado`, `de_otra_persona`) | implementacion |
| R2-1 | MENOR | aceptado | crates/tc2/src/estado/tareas.rs (`de_otra_persona`, rama `asignado-heredado`) | implementacion |

R2-1: quitar `.con("orden", orden)` del texto de `asignado-heredado` sobrevive en `rechazos_comunes_por_estado_y_por_asignado`, que no ejercita la rama heredada. Aceptado: solo afecta a la redaccion del diagnostico de stderr; el motivo `asignado-heredado` lo cubren los unitarios de `Tomar` y de `ciclo`.

## Verificacion

Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-034-r2-calidad` (borrado). `--test ciclo` 15/15, `--test toma` y `--test config` en verde, `--lib rechazos_comunes` 1/1.

| Mutacion | Test que se pone rojo |
|---|---|
| `if false && hay_remoto(...)` en `paso_descartar` | `c5_rama_publicada_desde_otro_clon_quita_el_worktree` |
| sin `--full-tree` en `identidad.rs` | `desde_una_subcarpeta_del_clon_o_del_worktree` |
| sin `--full-tree` en `config/mod.rs` | `de_base_lee_origin_develop_y_no_el_worktree` |
| `por_estado`: `DESCARTADA` devuelve `terminada` | `rechazos_comunes_por_estado_y_por_asignado` |
| `de_otra_persona`: `dueno = None` | `rechazos_comunes_por_estado_y_por_asignado` |

La adenda de F1-4 coincide con la salida real; sin decision duplicada entre `toma/` y `ciclo/` (textos `TAREA_*` solo en `catalogo/comun.rs`, `grafo::DESCARTADA`, `EMPEZABLES`/`ABIERTAS`). Sin fallos nuevos.

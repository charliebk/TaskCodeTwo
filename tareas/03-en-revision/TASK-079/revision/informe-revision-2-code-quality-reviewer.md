# Informe de revisión — TASK-079 (ronda 2)

- Commit revisado: ef977f1
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| M5 | MENOR | abierto | crates/tc2/src/estado/montaje.rs:233 | implementacion |

### M5: los brazos nuevos de `montar` para `Reescrita` y `Bloqueada` no tienen test

Cambiar `Traida::Reescrita { detalle } => return Err(rama_reescrita(OP, &detalle))` por `Traida::Reescrita { .. } => crear = true` deja verdes `--lib`, `estado_bordes`, `estado_casos` y `frescura` (C10 y C11 solo ejercitan `new`). Se alcanza al montar una rama de estado que no existe en local con la ref remota rechazada o bloqueada. Falta un caso de `montar` con rama reescrita.

## Verificacion

Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-079-r2-calidad` (borrado). `--lib` 204 ok, 1 ignorado; frescura 12/12; roadmap 13/13; siguiente 6/6; tablero 5/5 (H5 no fallo).

| Mutacion | Resultado |
|---|---|
| M1: `comparar` sin `anotar` | rojo `c9_...` |
| M2: sin `· pendientes N` | rojo `c7_pendientes_y_avisos_de_tareas` |
| M3: `montar` borra la marca tras `Hecha` | rojo `c4_...` |
| `montar` trata `Reescrita` como `crear` | verde (M5) |

M1, M2, M3, M4, H1 (lectura `remoto=rechazado`, escritura `error` con `motivo=rama-reescrita` sin pendiente; ref bloqueada `error-local`; clasificacion en `LC_ALL=C`), H2, H3 y H4 corregidos; H5 atendido (diagnostico ampliado, no reproducido). Adendas de F1-2 y F1-4 coinciden con la salida real; avisos nuevos en es y en; sin decision duplicada (refspec y clasificacion en `traer_estado` y `clasificar_traida`).

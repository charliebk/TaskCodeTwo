# Informe de revision — TASK-020 (ronda 2)

- Commit revisado: 28ac126257f93838179e03a8e9905745575499a1
- Revisor: general-purpose (sonnet) con rust-wasm-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/tarea/yaml.rs:44 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/tarea/cuerpo.rs:69 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### Reproduccion de cada hallazgo

- **MEN-1. `canonicos` rechaza YAML válido cuando una línea en blanco o un comentario sigue al escalar** (`valor_plano` en yaml.rs y `canonicos` en modelo.rs). `partir` pega las líneas en blanco y los comentarios a la entrada anterior, y `valor_plano` devuelve `None` si el `crudo` tiene más de una línea. Medido con `tarea::leer`:
  - `sprint: 0\n# nota\n` y `sprint: 0\n\n` dan «tiene que ir en una sola linea»;
  - `sprint: 0\t# c` da «no es un entero» (solo se corta ` #`, no con tabulador);
  - en una tarea 0.16 real, `plan_aprobado: true\n# x` da el mismo error.

  Es un falso rechazo con mensaje claro: no hay pérdida de datos y ninguna tarea del corpus lo dispara (404 + 97).
- **MEN-2. Ramas protectoras nuevas sin test que las ponga en rojo (mutación).** Ficheros: `cuerpo.rs` y `tests/tarea.rs`.
  - En `cierra`, cambiar `n >= abierta.1` por `n == abierta.1` sobrevive: falta un caso de cierre con más acentos que la apertura (`"## A\n````\n## B\n`````\n## C\n"`).
  - En `escape_ambiguo` sobreviven las ramas de comilla simple y de comentario. Solo importan con un `"` dentro: `titulo: 'a"b\c'` o `titulo: "a" # "\`.
  - `strict_booleans`, `underscored_numbers_as_strings` y `leading_zero_numbers_as_strings` a `false` sobreviven como mutantes equivalentes, porque `canonicos` cubre todos los campos. Son defensa en profundidad y no cuentan como hallazgo.

Verificación de la ronda 1:
- **IMP-1 corregido.** `yes`, `On`, `True`, `0x2`, `2.0`, `00`, `-0`, `!!int 3`, `'7'`, `Null`, `~`, el valor vacío y `2#a` dan error. `null # x` y `0 # c` pasan.
- **MEN-2, MEN-3, MEN-4, MEN-7 y MEN-8 corregidos.**
- **MEN-6 corregido.** Única divergencia de borde: una cerca dentro de un ítem de lista no se reconoce. No se reporta.
- **MEN-5 aceptado: razonable.** La documentación de `escribir` lo declara, el cuerpo se conserva byte a byte y ninguna de las 595 tareas reales lleva CRLF ni BOM.

## Alcance

- Ficheros revisados: `crates/tc2/src/tarea/{cuerpo,modelo,yaml}.rs` y `crates/tc2/tests/tarea.rs`.
- Límite JS-WASM: no aplica (CLI nativo). Sin `unsafe`, sin `f64` y sin `unwrap` ni `expect` sobre entrada externa.

## Reproduccion

- Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-020-r2\repo` (detached en 28ac126), borrado al terminar.
- `cargo test -p tc2 --locked`, `fmt --check` y `clippy -D warnings` en verde. `TC2_CORPUS` (ToolsaWeb y OpenDockTrading): 404 y 97 tareas leídas.
- Unos 50 casos hostiles en un fichero temporal.
- 17 mutantes: 12 detectados, 3 equivalentes y 2 reales (MEN-2).

## Lecturas fuera del contexto permitido

- Copias de solo lectura de ToolsaWeb (`tareas/04-terminadas/TASK-001/tarea.md`, base de casos 0.16) y OpenDockTrading para el test con `TC2_CORPUS`.
- `crates/tc2/src/tarea/mod.rs` y `crates/tc2/src/alta.rs`, para construir los casos.

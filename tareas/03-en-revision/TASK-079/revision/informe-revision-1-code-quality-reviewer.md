# Informe de revisión — TASK-079 (ronda 1)

- Commit revisado: ba24ba8
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

Nota de quien orquesta: el revisor escribio «aprobada» (sin CRITICOS ni IMPORTANTES, cuatro MENORES). Carlos ha pedido corregir todos los MENORES; corregir estos y los cinco del revisor de Rust toca ficheros que ningun MENOR cita (adendas de los ADR, catalogo i18n), asi que `verificar-menores` no los admite y los cambios deben pasar por una segunda ronda de revision. Por eso se registra `cambios-solicitados`; el texto de los hallazgos es el del revisor.

## Verificacion

Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-079-r1-calidad` (borrado). `cargo test --workspace`: 378 ok, 0 fallos.

| Mutacion | Resultado |
|---|---|
| quitar `frescura.anotar` en `board_en` | rojo C1, C3, C6 |
| quitar `pendientes` de `status` | rojo C7 |
| invertir `es_ancestro` en `local_de` | rojo C7 |
| hora tambien sin `origin` en `en_texto` | rojo `en_texto_y_linea` |
| quitar `frescura.anotar` en `roadmap::comparar` | sigue verde (M1) |

Tests anteriores no debilitados: `tests/siguiente.rs` pasa de igualdad exacta a prefijo `esperada()` mas sufijo estricto y conserva stderr vacio; `next_sin_red_usa_la_copia_del_clon` se endurece (`remoto=sin-red`, `sincronizado=nunca`, aviso); `tests/tablero.rs` solo enmascara las horas (`sin_hora`) y la foto excluye la marca; `board_con_la_plataforma_de_produccion` anade `GIT_SSH_COMMAND` coherente con el fetch nuevo. Salida coincide con la adenda de F1-4 salvo M1. Formato de la frescura solo en `estado/frescura.rs`; un solo `fetch` (`traer_estado`) para transaccion y lectura.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| M1 | MENOR | corregido | crates/tc2/src/roadmap/mod.rs:392 | implementacion |
| M2 | MENOR | corregido | crates/tc2/src/tablero.rs:198 | implementacion |
| M3 | MENOR | corregido | crates/tc2/src/estado/montaje.rs:233 | implementacion |
| M4 | MENOR | corregido | crates/tc2/src/estado/frescura.rs:46 | implementacion |

### M1: `roadmap --comparar` con `remoto=` y `sincronizado=` sin test

Cambiar `(frescura.anotar(i), Some(frescura))` por `(i, Some(frescura))` deja `--test frescura --test roadmap` en verde. Anadir la asercion en C9.

### M2: `status` en texto no muestra `pendientes=N`

`linea_status` no lo incluye; solo sale en `--json`. Decidir si el texto lo muestra con N>0 o dejarlo escrito en la adenda como solo JSON.

### M3: `montar` repite el refspec y no renueva la marca

`montaje.rs:233-240` hace el mismo `fetch` que `traer_estado` a mano. Opcional: que use `traer_estado`.

### M4: brazo muerto en `Traida::remoto`

`Fallo { fallo: FalloRemoto::RamaInexistente }` es inalcanzable: `traer_estado` ya lo convierte en `Traida::SinRama`.

## Ya declarados y confirmados

Sin tiempo limite en el puerto; dos `fetch` a la vez pueden dar `cannot lock ref` clasificado como `sin-red` (adenda de F1-2 y autorrevision).

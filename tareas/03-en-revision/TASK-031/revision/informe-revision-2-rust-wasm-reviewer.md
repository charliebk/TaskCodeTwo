# Informe de revision — TASK-031 (ronda 2)

- Commit revisado: 48d4390b5a72a8a4311ad4007f5b77284bb0300e (HEAD de la rama: 5fb22c0, solo anade la peticion)
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/roadmap/mod.rs:188 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### IMP-1 — corregido

`Regenerar::aplicar` compara ahora la zona publicada con la de la que partio la edicion (la del `HEAD` de la carpeta montada) y con la editada; si no coincide con ninguna, `rechazada motivo=zona-divergente` sin pisar nada.

Verificado con el binario de la rama y dos clones de un bare (el test `zona_editada_a_la_vez_por_dos_no_se_pisa`, que monta A y B en el mismo commit, ejecuta `tc2 roadmap --escribir` de verdad y mira el bare):
- A publica `NOTA DE A`; B, que parte del commit anterior, edita (`NOTA DE B`) y ejecuta `--escribir`: sale con codigo 3, `roadmap rechazada ... motivo=zona-divergente`, el bare conserva `NOTA DE A` y no tiene `NOTA DE B`, el fichero de B sigue con su edicion byte a byte, `status` de `tareas/` sigue en ` M ROADMAP.md` y la huella de refs/worktrees no cambia salvo la ref remota que trajo el fetch.
- B descarta su edicion (`checkout --`) y ejecuta `--escribir`: sin conflicto, trae lo de A.
- Lectura del codigo: si B no toco la zona (solo cambios fuera), `editado_en` devuelve `FueraDeZona` y se usa `Regenerar::nueva()`, que no pisa nada (cubierto por `cambio_fuera_de_la_zona_se_avisa`).
- Mutacion: con la condicion de divergencia anulada (`if false && ...`) el test `zona_editada_a_la_vez_por_dos_no_se_pisa` se pone rojo (roadmap.rs:201); el resto de la suite de `roadmap` sigue verde. Mutacion revertida.

### Notas (no abren ronda)

- NOTA-1 (MENOR, sin tabla): `restaurar_indice` (mod.rs:430) no tiene red de pruebas. Solo se usa sin remoto (con remoto `preparado` es `None`) y la transaccion sin remoto solo falla por un error de git; `escribir_que_falla_deja_el_indice_como_estaba` usa un remoto inalcanzable y por tanto no pasa por ella. Mutacion: anular la llamada (`i.resultado == Ok && false`) deja las 13 pruebas de `roadmap` en verde. La lectura del codigo es correcta (lee lo de la persona en memoria, escribe el contenido previo, `git add`, devuelve lo de la persona). La ventana en la que el fichero de trabajo contiene el contenido previo son milisegundos y es la aceptada; si el proceso muere ahi, la edicion queda solo en el proceso (no en disco): es el unico camino de perdida y esta documentado como aceptado.
- NOTA-2 (MENOR, sin tabla): `es_fin` exige la marca exacta `<!-- manual:fin -->`. Un ROADMAP de 0.16 con `<!-- manual:fin ... -->` con texto pasa a `roadmap-roto` (antes se aceptaba por prefijo). Es la consecuencia buscada de MEN-2; solo afecta a ficheros con una marca que 0.16 no generaba.

## Alcance
- Ficheros revisados: crates/tc2/src/roadmap/mod.rs, roadmap/zona.rs, situacion.rs, siguiente.rs, i18n/catalogo/roadmap.rs, tests/roadmap.rs (diff 8a54049..48d4390).
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios; el diff no toca `#[wasm_bindgen]`.

## Reproduccion
- Clon: C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-031-r2-rust, rama feature/task-031-f3-4-roadmap-la-vista-para-todos (HEAD 5fb22c0). Borrado al terminar.
- Build: nativo (`cargo test`, `cargo clippy`); no hay objetivo wasm32 en este diff.
- Formato y lints: `cargo clippy --all-targets --locked -- -D warnings`: sin avisos (exit 0).
- Suite: `cargo test -p tc2`: todos los binarios en verde, 0 fallos (el bloque `roadmap`: 13 pruebas ok, incluidas `zona_editada_a_la_vez_por_dos_no_se_pisa`, `escribir_que_falla_deja_el_indice_como_estaba`, `zona_editada_sin_remoto`, `cambio_fuera_de_la_zona_se_avisa`, `personas_ilegibles_se_avisan`).
- Casos construidos: dos clones de un bare con A publicando antes que B (IMP-1); B sin editar la zona; B con solo cambios fuera de zona; sin remoto; remoto inalcanzable.
- Mutacion: (1) divergencia anulada: test rojo. (2) restauracion del indice anulada: todo verde (NOTA-1).
- Medidas: no aplican.

## Revisado sin hallazgos
- `editado_en`: lee con `hash-object -w` + `cat-file blob` sin tocar el indice, valida la zona antes de nada y distingue `FueraDeZona` de `Zona`; `base` sale del `HEAD` de la carpeta. Sin `git add` previo, el rechazo por zona rota ya no deja nada preparado.
- `preparar` / `restaurar_indice`: solo sin remoto; recuerda el contenido previo del indice, y tras un fallo lo restaura y devuelve la edicion de la persona (leida en memoria antes). Con remoto, el `git add` ocurre solo tras publicar (`adelantar_editada`). Una edicion de la persona no se pierde salvo en la ventana de milisegundos aceptada.
- MEN-2: `es_inicio`/`es_fin` reconocen la marca completa (con espacios alrededor y CRLF); `<!-- manual:inicio2 -->`, `<!-- manual:finalizado -->` y `<!-- manual:iniciox -->` ya no cuentan. Cubierto por `marcas_completas_no_prefijos`.
- `situacion.rs:97`: sin ROADMAP o sin zona se usan los hitos de `zona::POR_DEFECTO` (que ahora lleva `### H0`); con zona rota se sigue usando el texto tolerante; el primer ROADMAP generado queda `ok` en `--comparar` (tests de toolsaweb y `roadmap_y_next_coinciden...` en verde).
- Produccion sin `unwrap`, `expect`, `panic!`, `unreachable!` ni `unsafe` en mod.rs, zona.rs y situacion.rs (los hay solo en `#[cfg(test)]`).

## Lecturas fuera del contexto permitido

- crates/tc2/tests/comun/mod.rs: para saber como la fixture monta los clones y reproducir IMP-1 con el mismo entorno aislado de git.

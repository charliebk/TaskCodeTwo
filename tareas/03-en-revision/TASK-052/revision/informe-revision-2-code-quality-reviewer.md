# Informe de revision — TASK-052 (ronda 2)

- Commit revisado: 3df9643e52dc72400bde144775a4131f7e726ca0
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

sin hallazgos

IMP-1 de la ronda 1: corregido de verdad (reproducido, ver abajo). No hay hallazgos nuevos ni regresiones en lo cambiado.

## Reproduccion

Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-052-r2-calidad` (checkout 3df9643, `target` propio). La verificacion de la ronda ya estaba en verde en d032160 (mismo codigo): no repetida la suite completa. Ejecutado por mi:

- `cargo test -p xtask`: 34 ok.
- `cargo test -p tc2 --test hallazgos`: 8 ok.
- `cargo test -p tc2 --lib hallazgos`: 21 ok (incluye `tras_una_ligera...`, `una_ligera_no_sirve...`, `en_un_commit_cuenta_el_mas_nuevo...`, `el_informe_de_la_tarea_cuenta_como_el_check`).

### IMP-1 (diff incremental arrastraba lo de la base): corregido

Repos Git reales de juguete con el binario `tc2` real (`tc2 hallazgos desde --anterior --head --base origin/develop`), mismo escenario de la ronda 1 (A en la rama, develop avanza tocando `otro.rs`, merge de develop en la rama, commit B):

| Caso | Resultado |
|---|---|
| Merge de la base | `desde=<arbol> motivo=merge`; `git diff --name-only <desde> HEAD` da solo `pr.rs`. El rango viejo `A HEAD` daba `otro.rs` y `pr.rs`. |
| Sin merge nuevo (anterior = head, o head = A) | `motivo=directo`, desde = el propio anterior. |
| Rebase (anterior no es ancestro) | `desde=base motivo=no-ancestro` (vuelve a `base...head`). |
| Conflicto (develop toca las mismas lineas que A) | `desde=base motivo=conflicto` (falla en cerrado), rc 0. |
| Argumentos invalidos (sha mal formado, base `-x`) | `uso` con `sha-invalido` / `base-invalida`, rc 2. |

`git merge-tree --write-tree` solo escribe objetos: tras las pruebas el repo de juguete no tenia ramas ni worktree modificados. El test de integracion `hallazgos.rs:859` cubre el mismo caso con git real.

Lo que la correccion podria romper, mirado y sin fallo: (1) la extraccion de `desde=` con `sed` casa SHA-1 y SHA-256 y deja vacio con `desde=base`, y el rango vuelve a `base...head`; (2) `desde.txt` se escribe en `$RUNNER_TEMP/revision-ia/ronda`, que es donde lo lee el paso siguiente (`$salida/ronda/desde.txt`), y vacio tambien cuando no hay anterior; (3) el arbol de `merge-tree` se crea en `base`, el mismo repo donde se hace luego el `git diff` (clon con `fetch-depth: 0`); (4) un git sin `--write-tree` (< 2.38) cae en `motivo=git` y se revisa todo el PR (falla en cerrado); (5) si `tc2 hallazgos desde` sale con error, `set -e` aborta el paso. Texto del prompt coherente con ambos casos (con y sin `desde`).

Revision rapida del resto del diff de la ronda (workflow `filter=all`, `elegir` por commit/ligera, `modo_de_resumen`, `rondas_extra` con vallas de codigo, `validar_con` sin `hallazgos`): sin fallo; cubierto por los tests anteriores.

## Lecturas fuera del contexto permitido

- `.github/workflows/revision-ia.yml` (lineas 165-175, 295-380) y `crates/tc2/tests/hallazgos.rs:859`: ya en la lista; solo contexto adicional del paso (fetch-depth, rutas de `desde.txt`).

Notas (fuera de la tabla): ninguna.

Veredicto propuesto: aprobada

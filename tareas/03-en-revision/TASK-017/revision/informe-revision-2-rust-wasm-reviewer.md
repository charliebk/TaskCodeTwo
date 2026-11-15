# Informe de revision — TASK-017 (ronda 2)

- Commit revisado: c943b1c8a5f3575a0fb320fbed783ac55f3ebb1a
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido | crates/tc2/src/git/worktree.rs:45 | implementacion |
| IMPORTANTE-2 | IMPORTANTE | corregido | crates/tc2/src/git/guardar.rs:121 | implementacion |
| IMPORTANTE-3 | IMPORTANTE | corregido | crates/tc2/src/git/puerto.rs:219 | implementacion |

Sin hallazgos nuevos (ni CRITICO ni introducido por la correccion). Los MENORES
de la ronda 1 no se reverifican en esta ronda.

## Verificacion de los IMPORTANTE de la ronda 1

- IMPORTANTE-1: `quitar` rechaza `sin-rama` con HEAD suelto antes de tocar nada. Test `quitar_worktree_con_head_suelto_no_pierde_commits` verde; mutante I1 (saltar la comprobacion) → ROJO.
- IMPORTANTE-2: `repartir` no filtra por disco; `entradas_con_origen` aporta el origen del renombrado como `D `. Test `guardar_renombrados_y_borrados_ya_en_el_indice` verde; mutantes I2a (sin origen) e I2b (`D ` a `add`) → ROJOS.
- IMPORTANTE-3: tabla `rechaza_cada_argumento_prohibido_sin_lanzar_git`; mutantes P1–P9 (push -d/--delete/--prune/--all, branch -d/--delete, tag --delete/-f, worktree -f) → ROJOS.

## Alcance
- Diff ca600a5 frente a 7d2d4c06: guardar.rs, puerto.rs, remoto.rs, tag.rs, worktree.rs, tests/git_casos_limite.rs.
- Corte en `--` (add, commit, status, diff) y salto del valor de `-m` (commit, tag): mutados (P10, P11) → ROJOS; no se encontro argumento prohibido que escape.
- Limite JS-WASM: no aplica.

## Reproduccion
- Clon `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-017-r2` (c943b1c), borrado al terminar.
- `cargo xtask verificar --perfil rapida`: verde (lib 11, cli 2, git_binario 1, git_casos_limite 5, git_ciclo 1, git_conflicto 1, git_push 1, xtask 3).
- 20 mutantes, todos con su test en rojo, revertidos.

## Notas (no abren ronda)
- Un worktree con HEAD suelto sin commits propios ahora se rechaza con `sin-rama` (antes se quitaba). Sin perdida de datos; decision de diseno. Lectura de codigo, no reproducido.

## Lecturas fuera del contexto permitido

- crates/tc2/src/git/mod.rs (tipo de `Worktree.rama`)

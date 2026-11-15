# Informe de revision — TASK-017 (ronda 1)

- Commit revisado: 7d2d4c06fe9dd81556612abcb562219d65e6fa8a
- Revisor: rust-wasm-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido | crates/tc2/src/git/worktree.rs:45 | implementacion |
| IMPORTANTE-2 | IMPORTANTE | corregido | crates/tc2/src/git/guardar.rs:121 | implementacion |
| IMPORTANTE-3 | IMPORTANTE | corregido | crates/tc2/src/git/puerto.rs:219 | implementacion |
| MENOR-1 | MENOR | corregido | crates/tc2/src/git/remoto.rs:84 | plan |
| MENOR-2 | MENOR | corregido | crates/tc2/src/git/puerto.rs:219 | implementacion |
| MENOR-3 | MENOR | corregido | crates/tc2/src/git/guardar.rs:110 | implementacion |
| MENOR-4 | MENOR | corregido | crates/tc2/src/git/remoto.rs:240 | implementacion |

Lineas del commit revisado (el HEAD bcc43ba solo anade la peticion).

### IMPORTANTE-1 — `quitar-worktree` borra commits sin publicar con HEAD suelto
La comprobacion `sin-publicar` solo corre dentro de `if let Some(rama) = &w.rama`.
Reproduccion: TASK-007, `checkout --detach` en el worktree, commit de
`perdido.txt`, `quitar_worktree` → `quitar-worktree ok id=TASK-007`; el commit
queda inalcanzable. Sugerencia: con HEAD suelto, rechazar (`motivo=sin-rama`)
o exigir que HEAD este en alguna `refs/remotes/origin/*`; test.

### IMPORTANTE-2 — `guardar` con renombrado o borrado ya en el indice
`conocidas` filtra por existencia en disco y `entradas_status` descarta la
ruta de origen de los renombrados.
(a) `git mv m.txt n.txt` + `guardar "mv" m.txt n.txt` → `ok ... ficheros=1
sin-guardar=2`; el borrado de `m.txt` queda en el indice (`D  m.txt`).
(b) `git rm z.txt` + `guardar "rm z" z.txt` → `guardar error git=add`
(`pathspec 'z.txt' did not match any files`).
(c) borrado en disco sin stage: funciona.
Sugerencia: no filtrar por disco; quitar solo lo que git no conoce; tests de
`git mv` y `git rm`.

### IMPORTANTE-3 — Los tests del puerto no cubren varios argumentos prohibidos
Mutantes que sobreviven al quitar la regla: `push --delete`, `push -d`,
`push --prune`, `push --all`, `branch -d`, `branch --delete`, `tag --delete`,
`tag -f`, `worktree -f`. La decision 3 de F1-4 prohibe borrar ramas; la regla
puede desaparecer sin que un test lo note. Sugerencia: test por tabla con cada
variante prohibida, formas largas y cortas.

### MENOR-1 — Rechazo persistente del remoto acaba en `sinred` "reintenta con red"
Hook pre-receive que siempre rechaza → `push sinred ... intentos=3` (codigo
4). Coherente con F1-4 (B1b, linea 509), pero el agente puede reintentar en
bucle. Sugerencia: documentarlo u opcionalmente `agotada` (5).

### MENOR-2 — `--force*` se rechaza en cualquier argumento, tambien mensajes y rutas
`guardar "--force no se usa" b.txt` → `guardar error git=commit`; fichero
`--force-x.txt` → `guardar error git=status`. Sugerencia: no mirar tras `--`
ni el valor de `-m`.

### MENOR-3 — Falta test de `--only` en `guardar`
Sin `--only -- <rutas>` todos los tests siguen verdes (mutante M3b). El codigo
actual respeta un fichero ajeno ya en el indice. Sugerencia: test con
`git add otro.txt` previo.

### MENOR-4 — `actualizar-rama` hace merge commit aunque la remota sea fast-forward
`--no-ff` tambien sobre `origin/<rama>`: con la local solo por detras se crea
un merge (3 tokens en `rev-list --parents`). Ruido en el historial, sin
perdida. Sugerencia: `--no-ff` solo para la base.

## Alcance
- Revisados: contrato.rs, git/{diff,guardar,mod,puerto,rama,remoto,tag,worktree}.rs, lib.rs, tests/comun/mod.rs, tests/git_{binario,ciclo,conflicto,push}.rs; main.rs leido.
- Limite JS-WASM: no aplica (CLI nativo, sin wasm32).

## Reproduccion
- Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-017-r1` (bcc43ba), borrado al terminar.
- `cargo xtask verificar --perfil rapida` una vez: verde (lib 10, cli 2, git_binario 1, git_ciclo 1, git_conflicto 1, git_push 1, xtask 3).
- Tests temporales (`tests/rev_extra.rs`, solo en el clon): HEAD suelto, indice con fichero ajeno, borrado/`git rm`/`git mv`/subdirectorio, mensaje y ruta `--force...`, rechazo persistente, remota adelantada, conflicto en directorio. Sin hallazgo: borrado en disco (X8a) y `../b.txt` desde subdirectorio (X7).
- Mutantes (`perl -0pi`, revertidos):
  - M1 push reintenta non-fast-forward: MATADO (git_push.rs:66).
  - M2 puerto sin `--force`, `+`, `:`, `--mirror`, pull sin `--ff-only`, `merge --abort`, `commit --amend`, `tag -d`, `push -f`, `branch -D`: MATADOS. `worktree -f`, `push -d/--delete/--prune/--all`, `branch -d/--delete`, `tag --delete/-f`: SOBREVIVEN (IMPORTANTE-3).
  - M3a `commit -a`: MATADO; M3c `add -A`: MATADO; M3b sin `--only`: SOBREVIVE (MENOR-3).
  - M4 actualizar-rama sin guarda de merge abierto: MATADO.

## Revisado sin hallazgos
- contrato.rs (codigos 0–7 iguales a la ADR, escapado reversible, JSON, stderr solo si falla); diff.rs; rama.rs; tag.rs; fetch y actualizar-principal. Sin `unwrap`/`expect`/`panic` fuera de tests.

## Lecturas fuera del contexto permitido

- docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md (contrastar lista cerrada, prohibidas y codigos)
- crates/tc2/src/main.rs (como se escribe la salida)
- task-workflow/revision.md del plugin

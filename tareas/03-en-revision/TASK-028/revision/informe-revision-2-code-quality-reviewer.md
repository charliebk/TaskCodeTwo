# Informe de revision — TASK-028 (ronda 2)

- Commit revisado: 2da874858cbdd8149b00e8c3fa4fe864bd3a560a
- Revisor: code-quality-reviewer
- Veredicto: PENDIENTE (escribelo con: taskctl veredicto TASK-028 aprobada | aprobada-con-correcciones | cambios-solicitados)

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/init/inventario.rs:309 | implementacion |
| MEN-1 | MENOR | corregido | crates/tc2/src/init/inventario.rs:389 | plan |
| MEN-2 | MENOR | corregido | crates/tc2/src/init/inventario.rs:537 | plan |
| MEN-4 | MENOR | corregido | crates/tc2/src/init/inventario.rs:42 | implementacion |
| MEN-5 | MENOR | corregido | docs/adr/F1-14-distribucion-e-instalacion.md:8 | plan |
| MEN-6 | MENOR | corregido | docs/adr/F1-5-puertas-en-github.md:368 | implementacion |
| MEN-7 | MENOR | corregido | tareas/03-en-revision/TASK-028/tarea.md:33 | especificacion |

Sin hallazgos nuevos CRITICO ni IMPORTANTE: ninguno de los que se ven fuera de la tabla lo es o lo introduce la correccion (ver Notas).

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea. (Origen tomado de la ronda 1.)

## Reproduccion y verificacion por hallazgo

### IMP-1 (corregido)

Reproduccion con `tc2.exe` del clon (`TC2_IDIOMA=es`), repos de prueba en `...\pruebas-claude\taskcodetwo\r2-imp1`:

1. `semilla`: `git init -b main`, commit vacio, push a `rem.git` (bare). El remoto tiene solo `refs/heads/main`.
2. `loc`: `git init -b main`, sin commits, `git remote add origin <rem.git>`.
3. `tc2 init --simular` en `loc`: `refs/heads/main` sale `persona motivo=en-el-remoto` con `git fetch origin main && git branch main origin/main`. `refs/heads/develop` sale `persona motivo=remoto-con-historia` con `git fetch origin main && git branch develop origin/main && git push origin develop`. Ninguna accion `crea` de ramas.
4. `tc2 init` real (sin `--simular`): no se crea `develop` ni local ni remoto, y `rem.git` sigue con `refs/heads/main` igual (7c0f597). Lo unico nuevo en el remoto es la rama de estado, ver Nota N-2.

Tests: `remoto_con_historia_no_es_repo_vacio` (`crates/tc2/tests/init.rs:754`) pasa, con `main` y con `trunk`.

Mutacion M1: `crates/tc2/src/init/inventario.rs:309`, `let vacio = !hay_commits && historia_remota.is_empty();` cambiado a `let vacio = !hay_commits;`. Falla `remoto_con_historia_no_es_repo_vacio` en `init.rs:785`: `develop` sale `crea` en vez de `persona`. Restaurado con `git checkout`.

### MEN-1 (corregido): linea de versionar

`versionar()` (`inventario.rs:389`) se anade tras `ramas()` solo si el repo no estaba vacio y config o `.gitignore` salen `crea` o tienen cambios sin versionar (`git status --porcelain`). En la reproduccion del IMP-1 sale: `accion ... clase=versionar decision=persona motivo=sin-versionar orden=git add -- .taskcode/config.yml .gitignore && git commit -m "chore: configura TaskCodeTwo"`. Tras el commit, la linea no sale (lo cubre `con_historia_pide_versionar_la_configuracion`).

Mutacion M2: `(los_toca || sin_versionar)` cambiado a `(false)`. Fallan `con_historia_pide_versionar_la_configuracion` y `remoto_con_historia_no_es_repo_vacio`. Restaurado.

### MEN-2 (corregido por el motivo)

El motivo de la linea `ruleset` pasa de `sin-checks` a `aplicar-con-task-118` (`inventario.rs:537`, `mod.rs:473-477`). La orden sigue siendo `tc2 init --aplicar-puertas` (sin cambios en el diff), que falla hasta TASK-118. El motivo avisa de eso, que es la opcion que proponia la ronda 1.

### MEN-4 (corregido): duplicados en `.gitignore` y `exclude`

`misma_linea()` (`inventario.rs:42`) compara sin espacios y sin la barra final. Lo usan `faltan()` (`inventario.rs:33`) y `anadir()` (`aplicar.rs:97`).

Test `gitignore_sin_barra_final_no_se_duplica`: pasa.

Mutacion M3: se quita `.trim_end_matches('/')` de `misma_linea()`. Falla `gitignore_sin_barra_final_no_se_duplica` (`init.rs:876`). Restaurado.

### MEN-5 (corregido en cabecera)

Adendas de 2026-11-17 en `docs/adr/F1-14-distribucion-e-instalacion.md:8` y `docs/adr/F1-18-hito-de-decisiones.md:8`, con la decision D4. El cuerpo de ambos ADR no se toca (F1-14 sigue diciendo en sus lineas de manifiesto que `init` lo comprueba). Como aviso basta, pero un lector del cuerpo no ve la excepcion salvo que lea la cabecera.

### MEN-6 (corregido)

`docs/adr/F1-5-puertas-en-github.md:368`: la fila `aplicar-puertas` lleva la adenda, y el punto de F3-1 (lineas ~505-507) dice que `init` propone y TASK-118 aplica. Verificado en el diff.

### MEN-7 (corregido)

`tarea.md:32-33`. C1 dice `.gitignore` con `/tareas/` y `/.taskcodetwo/`, y `exclude` con `/tareas/` y `/.taskcodetwo/worktrees/`. Coincide con `LINEAS_GITIGNORE` y `LINEAS_EXCLUDE` (`crates/tc2/src/huella.rs:66` y `:70`). C2 dice `decision=existe` para lo creado y `pendiente` para el resto, lo que asevera `init_dos_veces_sin_cambios`.

## Metodo y entorno

- Clon en `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-028-r2-calidad`, rama `feature/task-028-f3-1-taskctl-init-setup`. HEAD del clon es `ed7869f`, que solo anade la peticion; el diff revisado es el de `2da8748`. Todos los comandos git y cargo, con esa ruta absoluta o con la de mis repos de prueba. El repo original no se ha tocado.
- `cargo test -p tc2 --test init`: 17 passed, 0 failed (35,25 s de ejecucion). Sin cuelgues de 127.0.0.1 ni `LNK1104`.
- Mutaciones M1, M2, M3, cada una restaurada con `git checkout`. El clon queda sin cambios.
- Limite: la suite completa del workspace no se ha corrido en esta ronda. La peticion pide `--test init` y no trae verificacion de la ronda en verde para `2da8748`. La ronda 1 corrio la suite entera (todo en verde).

## Notas (no abren ronda, no van en la tabla)

- N-1 (preexistente, no la introduce la correccion; decision de quien coordina). Con historia **local** sin relacion con `origin/main`, la orden de `develop` es `git branch develop HEAD && git push origin develop` (rama `_ => HEAD` de `ramas()`, `inventario.rs` ~333-336). Reproducido en `...\taskcodetwo\r2-imp1b`: local `master` con un commit propio y `origin` con `main` de otra historia. Si la persona sigue la orden, publica un `develop` sin relacion con `origin/main`, el mismo daño que IMP-1 pero con la orden en manos de la persona (no se publica sola). El comentario de `ramas()` dice lo contrario. Sugerencia: con `origin` con historia y sin `develop` remoto, la orden debe partir de `origin/<rama>`, o avisar. Por mi lectura seria IMPORTANTE si la orden se sigue tal cual; lo decide quien coordina.
- N-2. `init` publica en `origin` con historia la rama de estado `taskcodetwo/tareas` (huerfana; en la reproduccion pasa a `9b7ef5c`). El test `ramas_de` la excluye a proposito. Conviene confirmar que "init no debe publicar nada" se refiere a ramas de codigo.
- N-3. Sin verificar, fuera de la peticion: MEN-3 (separadores de ruta en `t()`), que no aparece en la lista de esta ronda.
- N-4. La frase de `PLAN-TASKCODE-2.md:551` que citaba la ronda 1 (MEN-6) no aparece con ese texto (grep sin coincidencias). No lo he investigado mas.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/huella.rs:66-70` y `:116-119`: nombres exactos de `LINEAS_GITIGNORE` y `LINEAS_EXCLUDE`, para comprobar MEN-4 y MEN-7.
- `docs/plan/PLAN-TASKCODE-2.md`: busqueda de la frase de MEN-6 (sin coincidencias).
- `docs/adr/F1-14-distribucion-e-instalacion.md` y `docs/adr/F1-18-hito-de-decisiones.md`: grep de `init` para ver si el cuerpo contradice la adenda (MEN-5).
- `revision.md` y la skill `code-quality-reviewer`: instrucciones del proceso, no codigo revisado.

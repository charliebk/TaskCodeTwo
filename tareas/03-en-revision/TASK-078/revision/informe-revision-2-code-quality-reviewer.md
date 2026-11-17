# Informe de revision — TASK-078 (ronda 2)

- Commit revisado: b0f38bd9378c4b8d05c63255a0ec7c3e89f834ab
- Revisor: code-quality-reviewer
- Veredicto: aprobada

Propuesta del revisor: **aprobada** (0 hallazgos nuevos; IMP-1 e IMP-2 corregidos).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/desinstalar/mod.rs:521 | implementacion |
| IMP-2 | IMPORTANTE | corregido | crates/tc2/tests/desinstalar.rs:355 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

## Verificacion ejecutada

Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-078-r2-calidad`, rama de la tarea (HEAD 07c30a8, el codigo es el de b0f38bd).
- `cargo test -p tc2 --test desinstalar`: 18 de 18 verdes. `cargo test -p tc2 --lib`: 97 verdes. Sin caidas por LNK1104 ni por 127.0.0.1.
- No he corrido fmt/clippy ni el resto del workspace (verificacion acotada a lo que toca el diff).
- HOME, USERPROFILE, LOCALAPPDATA y APPDATA falsos en todas las pruebas a mano; `--maquina` solo con `--simular`.

## Reproduccion

**IMP-1 — corregido.** `mod.rs:512-521` busca primero la accion de clase `binario` en uso y solo si no hay recurre a la primera. El test `binario_en_uso_por_otro_queda_pendiente_sin_nada_a_medias` ahora asevera `h.informe.valor("ruta") == exe`, que `fin` contiene `motivo=en-uso ruta=` y que stderr dice `<exe> esta en uso`; antes solo miraba `contains("motivo=en-uso")`. El fixture tiene `.old` y temporales, es decir, el caso que fallaba.

**IMP-2 — corregido.** Test nuevo `dos_directorios_en_el_path_con_uno_en_uso` (dos directorios en el PATH, el de `TC2_INSTALL_DIR` en uso, el de por defecto no). Mutacion: `maquina.rs:198` `path = path_antes;` cambiado por `let _ = &path_antes;` deja **17 verdes y 1 roja**, la nueva (rechazada esperada, cae en el `assert_eq!` de la linea 376 porque el resultado deja de ser `Rechazada`). Revertida con `git checkout`. El test comprueba ademas el PATH final (`C:\a;<propio>;C:\b`), que el binario en uso sigue ahi y que el del directorio por defecto se quita.

**MEN-1 — corregido.** El resumen es coherente con `rechazada`: `mod.rs:445-454` pasa las `borra` a `pendiente motivo=no-aplicada` antes de contar (test `worktree_bloqueado_o_carpeta_actual_dentro_rechaza`). En «en uso», el grupo se acumula en `del_grupo` y no en `quitados`; el test asevera `carpeta=[pendiente]` y `borra=1`.

**MEN-2 — corregido.** Repo con remoto bare y rama de estado local adelantada (commit `64d4e35`): `--proyecto --simular --borrar-tareas taskcodetwo/tareas` imprime `decision=borra motivo=adelantada-del-remoto sha=64d4e35...` y en stderr el aviso con `git branch taskcodetwo/tareas <sha>`. El modo real hace lo mismo, borra la rama local y el remoto queda intacto (`git ls-remote`). Segunda pasada `sin-cambios`. Sin `--borrar-tareas` la rama sale `conserva`.

**MEN-3 — corregido** (aunque no estaba en la tabla): `bloqueo()` lee `w.bloqueado` y rechaza la carpeta actual dentro del worktree; test cubierto.

**MEN-4 — corregido.** La deteccion usa `config::leer` (`proyecto.rs`): `Ok`, `ConfigInvalida` y `FormatoNuevo` cuentan como de la 2.0; `Formato`, `SinConfig` y `Yaml` como no suya (se conserva, y `personas.yml` tambien). El test cubre `---\nformato: 2 # version\n` (linea 612) y el caso de la 0.16. Nota: tratar `ConfigInvalida` como de la 2.0 permite que `--config` borre un fichero con `formato: 2` pero esquema roto; es defendible (el formato lo declara como suyo) y solo con la opcion explicita.

**MEN-5 — corregido.** `grep -cP "\t" docs/distribucion/instalar.md` da 0. La seccion tiene los avisos de uso (linea 204: `--config`/`--borrar-tareas` sin `--proyecto` dan `uso` (2)).

**MEN-6 — corregido.** `main.rs` no llama a `limpiar_old` si el primer argumento es `desinstalar`. Visto en Windows con un `tc2.exe` copiado a una carpeta propia y un `tc2.exe.old`: tras `desinstalar --maquina --simular` el `.old` sigue ahi; tras `tc2 version` desaparece.

**MEN-8 — corregido.** `plugins()` filtra clave y scope con `seguro()` y cita la clave con comillas simples (`claude plugin uninstall 'taskcodetwo@taskcodetwo' --scope user`, asertado en el test de plugin). `installed_plugins.json` se lee una vez (`instalados`).

Cambios no pedidos que revise por si abrian un fallo nuevo, sin encontrarlo: `es_tc2` (un `tc2` ajeno queda `ajeno`, test `un_tc2_ajeno_no_se_borra`) y `escribir_atomico` (temporal en la misma carpeta, permisos conservados, limpia el temporal si falla, sigue enlaces simbolicos); la rama `adelantada` trata un remoto ausente como no publicado, que es el lado seguro.

## Notas (no abren ronda)

- MEN-9 de la ronda 1 sigue: rutas con `/` y `\` mezclados (`...\repo/.git\taskcodetwo` en la salida real). Cosmetico.
- No ejecute `probar-instalador.sh` ni Windows real para el autoborrado (solo CI): sigue sin verificar por mi.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/config/mod.rs`: para confirmar que `config::leer` devuelve `Formato`/`SinConfig`/`Yaml` para una config de la 0.16 (MEN-4).
- `tareas/.../informe-revision-1-code-quality-reviewer.md`: el informe de la ronda 1, pedido por quien coordina.

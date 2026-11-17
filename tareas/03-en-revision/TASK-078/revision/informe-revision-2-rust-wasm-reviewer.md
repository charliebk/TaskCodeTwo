# Informe de revision — TASK-078 (ronda 2)

- Commit revisado: b0f38bd9378c4b8d05c63255a0ec7c3e89f834ab
- Revisor: rust-wasm-reviewer
- Veredicto: PENDIENTE (escribelo con: taskctl veredicto TASK-078 aprobada | aprobada-con-correcciones | cambios-solicitados)

Veredicto propuesto por el revisor: aprobada (con tres notas MENORES al final, que no abren ronda).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/desinstalar/puertos.rs:175 | implementacion |
| IMP-2 | IMPORTANTE | corregido | crates/tc2/src/desinstalar/proyecto.rs:283 | implementacion |
| MEN-1 | MENOR | corregido | crates/tc2/src/desinstalar/texto.rs:17 | implementacion |
| MEN-2 | MENOR | corregido | crates/tc2/src/desinstalar/puertos.rs:99 | implementacion |
| MEN-3 | MENOR | corregido | crates/tc2/src/desinstalar/proyecto.rs:48 | implementacion |
| MEN-4 | MENOR | corregido | crates/tc2/src/desinstalar/mod.rs:589 | implementacion |
| MEN-5 | MENOR | corregido | crates/tc2/src/desinstalar/maquina.rs:84 | implementacion |

Sin hallazgos nuevos. Verificacion, uno por uno:

- IMP-1: `FLAGS = CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP` (sin `DETACHED_PROCESS`). Se ejecuto `el_auxiliar_real_borra_tras_salir_el_proceso` (`puertos.rs`, `#[cfg(all(test, windows))]`): pasa. Mutacion: se repuso `0x0000_0008` en `FLAGS` y el test cae ("tras 10.06s siguen el fichero o la carpeta"), es decir, reproduce el fallo original y detecta su vuelta. El test usa una ruta con espacios, `'` y `$`, un PID ya muerto y solo toca su carpeta temporal.
- IMP-2: `de_la_016` se calcula con `config::leer` (formato 2 reconocido, o `ConfigInvalida`/`FormatoNuevo` tratados como de la 2.0) y `personas.yml` queda `taskcode-016` + conserva aunque se pase `--config`. Test `config_solo_con_la_opcion` en verde.
- MEN-1: `linea_del_instalador` exige la linea completa de `install.sh:169` con el mismo `DIR` en los dos sitios; `esac; rm -rf ~/x` y otro DIR dan `Tocadas` (test nuevo). Coincide byte a byte con el `printf` del instalador. El salto de linea final anadido a un perfil sin el esta documentado.
- MEN-2: `PS_ESCRIBIR_PATH` usa `[string]$env:TC2_PATH_VIEJO` y `[string]$env:TC2_PATH_NUEVO`: un `$null` pasa a `''`. No se ejecuto contra el registro (regla de seguridad); es una revision de lectura del script, que ya no pasa `$null` a `SetValue`.
- MEN-3: `bloqueo` rechaza `worktree-bloqueado` (`locked` del porcelain) y `carpeta-actual-dentro` (canonicalizando); el rechazo convierte los `borra` en `pendiente motivo=no-aplicada`. Test `worktree_bloqueado_o_carpeta_actual_dentro_rechaza` en verde.
- MEN-4: `escribir_atomico` (`mod.rs:589`): temporal en la misma carpeta, permisos copiados, rename, limpia el temporal si falla; si la ruta es un enlace simbolico escribe el destino canonico. Sin `unwrap`/`panic`/`unsafe`.
- MEN-5: `es_tc2` (`maquina.rs:334`): rechaza enlaces simbolicos y exige `version --json` con `version` (cadena) y `protocolo` (u64), con `ejecutar_con_limite` a 10 s (reutiliza la funcion de `actualizar_cli`, ahora `pub(crate)`, que mata al hijo al vencer el plazo y limita la salida a 1 MiB). Test `un_tc2_ajeno_no_se_borra` en verde. Tambien se verifica la clave del plugin con `seguro` (letras, digitos, `.`, `_`, `-`, `@`) y va entre comillas simples.

## Notas (no abren ronda)

1. `un_tc2_ajeno_no_se_borra` escribe un script `#!/bin/sh` posiblemente sin permiso de ejecucion: el test no distingue "se ejecuta y no es tc2" de "no se puede ejecutar" (ambos dan `false`). Y la rama positiva de `es_tc2` (un `tc2` en `~/.local/bin` distinto del exe en marcha que si responde) no tiene test; `es_exe` o `dir_propio` la cubren en los demas tests. `crates/tc2/tests/desinstalar.rs:683`.
2. `es_tc2` ejecuta el binario del directorio de instalacion, tambien con `--simular` (que se anuncia como "no toca nada"). Es un `tc2` que la persona ya tendria en el PATH, con 10 s de limite y sin escribir, asi que el riesgo es bajo, pero conviene dejarlo escrito en `instalar.md`.
3. `escribir_atomico` rompe un enlace duro (el rename sustituye el inodo) y exige permiso de escritura en la carpeta, no solo en el fichero; antes bastaba lo segundo. El error se informa, no se pierde nada. Las ordenes con comillas simples de `claude plugin uninstall '...'` no valen en `cmd.exe` (si en PowerShell y sh).

## Alcance
- Ficheros revisados: actualizar_cli.rs (visibilidad), desinstalar/{maquina,mod,proyecto,puertos,texto}.rs, git/mod.rs, i18n/catalogo/desinstalar.rs, main.rs, tests/desinstalar.rs, docs/distribucion/instalar.md (solo el diff de la ronda).
- Contexto adicional pedido: `instalar/install.sh` (linea 169, para juzgar `linea_del_instalador`) y `crates/tc2/src/version.rs`/`lib.rs` (formato de `version --json`). Fuera del contexto permitido por la peticion; motivo: contrastar las inversas.
- Limite JS-WASM: no aplica (sin `wasm_bindgen` ni `wasm32` en el diff).

## Reproduccion
- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-078-r2-rust`, rama `feature/task-078-f2-14-taskctl-desinstalar-quitar-taskcod` (HEAD 07c30a8, el codigo es el de b0f38bd). Borrado al terminar. No se hizo checkout en el repo original.
- Build y suite: `cargo test -p tc2 --locked`: todos los bloques `test result: ok` (97 unitarios incluido el auxiliar real, mas integracion), 0 fallos, a la primera. Sin `LNK1104` ni cuelgues de 127.0.0.1.
- Lints: `cargo clippy --all-targets --locked -- -D warnings`: sin avisos. `cargo fmt --check` no se paso.
- Mutacion: `DETACHED_PROCESS` repuesto en `FLAGS` -> `el_auxiliar_real_borra_tras_salir_el_proceso` rojo; revertido con `git checkout`.
- Seguridad: nunca se ejecuto `tc2 desinstalar` contra el HOME, `~/.claude`, el registro ni el PATH reales; solo los tests con HOME falso y la maquina en memoria.
- Sin `unwrap`, `expect`, `panic!` ni `unsafe` en produccion de los ficheros del diff.

## Revisado sin hallazgos
- `ejecutar_con_limite` hecha `pub(crate)`: sin cambio de comportamiento.
- `del_grupo`/`quitados`: lo que borra un grupo en uso ya no cuenta para vaciar carpetas (el grupo rechazado restaura el PATH).
- Aviso `adelantada-del-remoto` con el sha: `es_ancestro` contra la remota; sin remota se avisa.
- `main.rs`: `desinstalar` ya no llama a `limpiar_old` (el `--simular` no toca nada).

## Lecturas fuera del contexto permitido

`instalar/install.sh` (funcion `escribir_bloque`), `crates/tc2/src/version.rs` y `crates/tc2/src/lib.rs`: para contrastar la linea exacta del instalador y el formato de `version --json`.

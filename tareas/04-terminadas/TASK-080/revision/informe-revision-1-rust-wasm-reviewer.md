# Informe de revisión — TASK-080 (ronda 1)

- Commit revisado: 642ec1c
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| H1 | MENOR | abierto | crates/tc2/src/inventario.rs:671 | implementacion |
| H2 | MENOR | abierto | crates/tc2/src/inventario.rs:358 | implementacion |
| H3 | MENOR | abierto | crates/tc2/src/inventario.rs:601 | implementacion |
| H4 | MENOR | abierto | crates/tc2/src/inventario.rs:536 | implementacion |
| H5 | MENOR | abierto | crates/tc2/src/doctor/recursos.rs:137 | implementacion |

### H1: entrada de plugin que no es lista se descarta sin incidencia

`leer_plugins` hace `continue` si `plugins.<clave>` no es una lista. Con `{"version":2,"plugins":{"x@m":{"installPath":...}}}` (objeto) o `"d@m": null` el plugin desaparece con `ilegibles=[]` y un recomendado suyo sale `falla no-instalado` en vez de `no-comprobado`. Incoherente con C8. Tampoco se mira `version`.

### H2: error de lectura de carpeta tragado

`hijos()` hace `read_dir(...).unwrap_or_default()`: un `skills/` o `agents/` ilegible (reproducido con `icacls ... /deny`) cuenta como vacio con `completo=true`. Nombres no UTF-8 tambien se descartan sin aviso.

### H3: `name` del manifiesto sincronizado sin validar

`cuenta.join(nombre)` con `name: "../../fuera"` lee fuera de `synced/` y da el id `../../fuera:malo`. Un `installPath` relativo se resuelve contra el cwd del proceso (linea 686).

### H4: `orden_de_fecha` compara tipos mezclados

Con `lastUpdated` numero en una cuenta e ISO en otra gana la ISO aunque sea mas antigua; negativos o NaN ordenan por texto.

### H5: filtro de tareas descartadas sin test

Quitar `|| t.estado == DESCARTADA` deja verdes `--test recursos` y `--lib`.

### Notas (sin ID)

`name: foo # comentario` da el id con el comentario; un skill de usuario `name: a:b` puede satisfacer el id de un plugin; frontmatter sin cierre toma un `name:` del cuerpo; nombre de mas de 16 KiB truncado sin aviso.

## Reproduccion

Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-080-r1-rust` (borrado). Sin tocar el `~/.claude` real. recursos 12/12, doctor 12/12, desinstalar 19/19, `--lib inventario recursos` 14/14; `fmt --check` y `clippy -D warnings` limpios. Mutaciones: `aplica` sin `projectPath` rojo; quitar `DESCARTADA` verde (H5); `completo` sin `ilegibles` rojo; desactivar `~gN` rojo. Sin unwrap/expect/panic ni indexado en codigo no de test; solo lectura.

## Revisado sin hallazgos

C11 y `leer_opciones`; linea de `--recursos`; JSON en una linea; escapado en `Hallazgo::linea` y `json`; huella determinista; `desinstalar` con `leer_instalados`; sincronizado que cede, `.trash`, `~gN`, `syncClaudeAiPlugins:false`; `evaluar_recursos` pura. Limite JS-WASM: no aplica.

# Informe de revision — TASK-078 (ronda 1)

- Commit revisado: b855c2057a8375e142ebdec0a665bce971e0af33 (HEAD de la rama al clonar: 2fa75ca, que solo anade la peticion)
- Revisor: rust-wasm-reviewer
- Veredicto: cambios-solicitados

Veredicto propuesto por el revisor: cambios-solicitados (IMP-1 incumple C8 en Windows con el proceso auxiliar real).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | abierto | crates/tc2/src/desinstalar/puertos.rs:172 | implementacion |
| IMP-2 | IMPORTANTE | abierto | crates/tc2/src/desinstalar/proyecto.rs:249 | implementacion |
| MEN-1 | MENOR | abierto | crates/tc2/src/desinstalar/texto.rs:48 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/desinstalar/puertos.rs:97 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/src/desinstalar/proyecto.rs:88 | implementacion |
| MEN-4 | MENOR | abierto | crates/tc2/src/desinstalar/mod.rs:528 | implementacion |
| MEN-5 | MENOR | abierto | crates/tc2/src/desinstalar/maquina.rs:77 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### IMP-1 — El proceso auxiliar de autoborrado no se ejecuta con `DETACHED_PROCESS` (C8)

`MaquinaReal::programar_borrado` lanza `powershell.exe` con `FLAGS = DETACHED_PROCESS | CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP`. Reproducido en esta maquina (Windows 11, PowerShell 5.1), con el script `PS_AUTOBORRADO` literal:

- Sonda de marca (`'ok' | Out-File marca`): con `0x8|0x08000000|0x200` el hijo sale con codigo 0 sin ejecutar el comando (no existe la marca). Con `0x8|0x200` igual. Con `0x08000000|0x200` (sin DETACHED_PROCESS) y sin flags, la marca existe.
- Autoborrado con PID ya muerto: con los flags actuales, tras 10 s el fichero y la carpeta siguen ahi. Con `0x08000000|0x200`, se borran en 0,77 s.
- Autoborrado con PID vivo (ping 4 s) y el proceso padre acabando sin esperar: con `0x08000000|0x200` el auxiliar sobrevive al padre y borra fichero y carpeta. DETACHED_PROCESS no hace falta para sobrevivir.
- El script en si es correcto: ejecutado sin DETACHED_PROCESS borra rutas con espacios, `'`, `$`, backtick, `;`, `&`, `(` y acentos. Las rutas viajan por variables de entorno y el script es constante, asi que no hay inyeccion.

Consecuencia: en Windows `desinstalar --maquina` imprime `decision=borra motivo=al-salir`, sale `ok` y el binario y su carpeta se quedan. Una segunda ejecucion vuelve a "borrarlos" sin efecto. Los tests no lo ven porque solo usan `MaquinaEnMemoria`; `MaquinaReal` no tiene ningun test (C8 queda "runner Windows", no verificado). Correccion sugerida: quitar `0x8` de `FLAGS` y anadir un test `#[cfg(windows)]` del auxiliar real sobre ficheros temporales.

### IMP-2 — `--config` borra `personas.yml` aunque la config sea de la 0.16 y se conserve

Con `.taskcode/config.yml` sin `formato: 2` (0.16) y `personas.yml`, `tc2 desinstalar --proyecto --config --simular` da:

```
config.yml    clase=config dueno=taskcode-016 decision=conserva
personas.yml  clase=config dueno=taskcodetwo  decision=borra
```

`personas.yml` esta compartido con la 0.16 (ADR F1-12 D4a: "compartida con la 0.16"), pero el plan lo marca `dueno=taskcodetwo` sin mirar de quien es el proyecto. Resultado: queda una config 0.16 sin su `personas.yml`, contra D6 y el espiritu de C10. Correccion: si la config es de la 0.16, `personas.yml` se conserva con `dueno=taskcode-016` aunque se pase `--config`. El test C10 no lo pilla porque no usa `--config`.

### MEN-1 — `quitar_bloque` solo comprueba el prefijo de la linea del bloque

`texto.rs:48` usa `starts_with(LINEA_PERFIL)`. Sonda: entre las marcas, `case ":$PATH:" in ...esac; rm -rf ~/x` devuelve `Ok(Some(""))`, es decir, se borra una linea que la persona modifico. El reconocedor se declara cerrado, pero esa linea tiene forma conocida (`case ":$PATH:" in *":DIR:"*) ;; *) export PATH="DIR:$PATH" ;; esac`): comprobarla entera y devolver `Tocadas` si no coincide. Ademas, si el perfil original no acababa en salto de linea, `install.sh` anade uno y el resultado no queda identico byte a byte (inherente; conviene documentarlo).

### MEN-2 — Un PATH de usuario que queda vacio no se puede escribir

`PS_ESCRIBIR_PATH` lee `$env:TC2_PATH_NUEVO`. Comprobado: una variable de entorno de valor vacio llega a PowerShell como `$null` (la sonda imprimio `NULL`). Si la entrada de tc2 era la unica del PATH, `SetValue('Path', $null, ...)` lanza (ArgumentNullException segun la documentacion de .NET; no probado contra el registro por la regla de seguridad) y la orden acaba en `error`. Antes ya se programo el autoborrado, asi que quedan el binario borrado y la entrada colgando. Solucion: tratar `$null` como `''` en el script.

### MEN-3 — Ejecutar desde dentro de un worktree de tarea limpio deja el borrado a medias (Windows)

Sonda con un repo falso: desde `.taskcodetwo/worktrees/TASK-1` (limpio y publicado), `tc2 desinstalar --proyecto` hace fallar a `git worktree remove` con `failed to delete` por ser el directorio actual; el registro del worktree ya no existe, la carpeta queda vacia y las ejecuciones siguientes la ven `no-vacia`. Se informa `error` y no se pierde nada, pero queda a medias. Rechazar (`rechazada motivo=cwd-en-worktree`) si `c.dir` esta dentro de un worktree que se va a quitar.

### MEN-4 — `Reescribir` trunca y escribe (no atomico)

`aplicar` hace `std::fs::write(ruta, despues)` tras comprobar `antes`. Si el proceso muere entre truncar y escribir, se pierde el perfil o el `settings.json` de la persona. `install.sh` hace lo mismo (`cat >`), asi que es coherente con el instalador; se anota por la clase de fichero. Mejora: temporal vecino y renombrar.

### MEN-5 — El binario se borra por nombre, sin comprobar que sea de tc2

En `maquina.rs:77`, `<dir>/tc2[.exe]` de `~/.local/bin` o de `TC2_INSTALL_DIR` se borra por ruta; no se verifica que sea el binario de tc2 (un enlace a un gestor, o una herramienta ajena llamada `tc2`), tampoco en modo plugin o gestor. Aceptable como diseno (ruta exacta de F2-11), pero conviene dejarlo anotado o no borrar enlaces simbolicos que apunten fuera de las carpetas propias.

## Alcance
- Ficheros revisados: crates/tc2/src/desinstalar/{mod,maquina,proyecto,puertos,texto}.rs, crates/tc2/src/instalacion.rs, el diff de crates/tc2/src/git/puerto.rs y de actualizar_cli.rs, lib.rs, contrato.rs, crates/distribucion/src/lib.rs y xtask/src/instaladores.rs.
- Contexto adicional pedido: `instalar/install.sh` e `install.ps1` (funciones `escribir_bloque` y `Add-PathUsuario`, para juzgar las inversas), `docs/adr/F1-12` y `planificacion/plan-final.md` (D4a, D6).
- Limite JS-WASM: no aplica. Sin `wasm_bindgen` ni objetivo `wasm32` en el diff.

## Reproduccion
- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-078-r1-rust`, rama `feature/task-078-f2-14-taskctl-desinstalar-quitar-taskcod`. Borrado al terminar, igual que la carpeta de sondas.
- Build: `cargo test -p tc2 --locked` compila en nativo. No se compilo `wasm32` (el diff no lo toca).
- Formato y lints: `cargo clippy --all-targets --locked -- -D warnings` limpio, sin avisos. `cargo fmt --check` no se paso.
- Suite: `cargo test -p tc2 --locked`: 23 bloques `test result: ok`, 0 fallos, a la primera (96 unitarios, mas la integracion de `desinstalar`: `simular_no_toca_nada`, `status_line_solo_la_de_la_2`, `hooks_nunca_se_editan_y_se_avisa`, `huella_de_la_016_intacta`, `segunda_ejecucion_sin_cambios`...). Sin `LNK1104` ni cuelgues de 127.0.0.1.
- Casos construidos (HOME, USERPROFILE, LOCALAPPDATA y CLAUDE_CONFIG_DIR falsos; nunca se ejecuto `--maquina` contra la maquina real ni se toco el registro):
  - `quitar_miembro` con clave repetida (primera nuestra y ambas nuestras), `statusLine` anidada sola y con la de raiz, coma final, comentario, BOM, ultima clave con espacios, clave unica y valor numerico. Los invalidos o ambiguos dan `None` (el llamador deja `persona` o `json-invalido`, nunca reescribe). Los validos conservan el resto de bytes (solo queda algun espacio sobrante, cosmetico).
  - `quitar_del_path`: entrada unica da `""`; `C:/x/bin/` con barra se reconoce; entradas ajenas y vacias intactas.
  - `desinstalar --proyecto --config --simular` con config 0.16 y `personas.yml` (IMP-2); desde dentro de un worktree de tarea (MEN-3).
  - Autoborrado real con PowerShell y rutas con caracteres especiales (IMP-1).
- Mutacion: no se paso `cargo mutants`; no se mutaron las protecciones del diff.
- Medidas: no aplican.

## Revisado sin hallazgos
- `Git::borrar_rama_de_estado`: la ref sale de `huella::RAMA_ESTADO` dentro de la funcion (el llamador no la elige), `viejo` debe ser hexadecimal de 40 o mas caracteres (no puede empezar por `-`) y `update-ref -d <ref> <viejo>` solo borra si la rama sigue en ese sha. Salta `comprobar_argumentos` a proposito (su `forma_de_update_ref` rechaza `refs/heads/*`), pero el unico camino es esta funcion cerrada. Es segura como unica excepcion a F1-4 D3.
- `GIT_OPTIONAL_LOCKS=0` en `ENTORNO_FIJO`: solo evita que `status` y similares refresquen el indice (puede re-hashear mas en repos enormes, pero es correcto). Las operaciones que escriben toman los cerrojos que necesitan; la suite de git y de estado sigue en verde.
- Sin `unwrap`, `expect`, `panic!`, `unsafe` ni `todo!` en produccion de los ficheros del diff. Los indexados (`b[i]`, `lineas[i]`, `&resto[..i]`) estan protegidos por el bucle o por una comprobacion previa.
- Reconocedor de propiedad cerrado en `statusLine` (`command == "tc2 estado --stdin"` tras trim; la de la 0.16 y las ajenas quedan en `conserva`) y en hooks (nunca se editan). Una `statusLine` duplicada o anidada no se reescribe.
- Inyeccion y comillas en el autoborrado: scripts constantes, rutas y PID por variables de entorno, `-LiteralPath`, sin interpolar texto de la persona; `orden_borrar` escapa las comillas simples. Salvo IMP-1, sin hallazgos.
- Inversa de `Add-PathUsuario`: solo quita entradas cuya expansion (`%LOCALAPPDATA%`, `%USERPROFILE%`) coincide con el directorio; conserva orden, entradas vacias y tipo REG; comprueba el valor viejo (`-cne`) antes de escribir.
- `info/exclude`: solo se quitan las dos lineas propias exactas; el resto queda byte a byte.

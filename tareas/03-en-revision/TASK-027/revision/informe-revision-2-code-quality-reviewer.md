# Informe de revision — TASK-027 (ronda 2)

- Commit revisado: 39ef7a1862bb2047f844adb168309dc4dd92d19c
- Revisor: code-quality-reviewer
- Veredicto: PENDIENTE (escribelo con: taskctl veredicto TASK-027 aprobada | aprobada-con-correcciones | cambios-solicitados)

Veredicto propuesto por el revisor: **cambios-solicitados** (IMP-2 abierto; IMP-1 y los MENOR de la ronda 1 verificados).

Alcance: verificacion acotada (ver `## Verificacion de la ronda 1`). El codigo de `crates/` de `39ef7a1` y de `HEAD` (`b3d2e77`, que solo anade la peticion) es identico: `git diff 39ef7a1 b3d2e77 -- crates` vacio.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/actualizar_cli.rs:451 | implementacion |
| IMP-2 | IMPORTANTE | abierto | crates/tc2/src/descarga.rs:278 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### IMPORTANTE-1 (IMP-2) — la correccion del reintento deja sin reintento los cortes antes de las cabeceras

- Donde: crates/tc2/src/descarga.rs:278 (clasificacion de `Sinred` en `a_escritor`: `FalloHttp::Sinred | Tls | Otro | RespuestaInvalida` pasan a `sinred(e, false)`) y crates/tc2/src/actualizar_cli.rs:646 (`bajar_con_un_reintento` solo reintenta `Fallo::Sinred { corte: true, .. }`).
- Que pasa: en `991b24c` cualquier `Fallo::Sinred` de la descarga del binario se reintentaba una vez. En `39ef7a1` solo se reintenta el corte a mitad del cuerpo (`corte: true`, que llega de `read` en el bucle). Un corte que llega antes de las cabeceras (`ureq::Error::Io`, `ConnectionFailed`, `Timeout`, `BodyStalled` durante `call()`) ya no se reintenta: `actualizar --cli` sale con codigo 4 al primer intento. El comentario dice "no conectar ... se repetiria igual", pero `Io` con conexion ya establecida (el `Peer disconnected` de este equipo) no es "no conectar".
- Reproduccion: servidor de ensayo en `127.0.0.1` (script de prueba fuera del repo, `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-027-r2-calidad-work\servidor_corte.py`) que sirve `tc2-9.9.9-win-x64.exe` (un `tc2_falso`, 196 KB) y SHA256SUMS correcto, y corta la PRIMERA peticion al `.exe` cerrando el socket sin cabeceras. Mismo comando en ambas versiones: `TC2_BASE_URL=http://127.0.0.1:<p> TC2_IDIOMA=es tc2 actualizar --cli`.
  - `39ef7a1` (nuevo): `actualizar sinred url=http://127.0.0.1:8771/tc2-9.9.9-win-x64.exe` con `no se pudo bajar ...: io: Peer disconnected`; codigo 4; una sola peticion al `.exe`. Nada cambiado.
  - `991b24c` (viejo, compilado en un worktree): `actualizar ok anterior=0.0.0 version=9.9.9 sha256=612a70cb...`; codigo 0; el segundo intento se sirve.
  - Ninguna prueba del repo cubre el reintento (ni el de corte a mitad ni el de antes de cabeceras), por eso la suite sigue en verde.
- Impacto: en el equipo donde TASK-027 vio los cortes (red local a 127.0.0.1, antivirus), una actualizacion que antes salia bien a la segunda pasada ahora falla con `sinred` y la persona tiene que repetirla a mano. No hay perdida de datos: la guarda borra el temporal y `tc2` no se sustituye.
- Sugerencia: volver a reintentar una vez los fallos transitorios de transporte (`Io`, `ConnectionFailed`, `Timeout`, `BodyStalled`) de la llamada, y mantener sin reintento TLS, estado HTTP, limite, redireccion y `RespuestaInvalida`. Hacer coincidir el comentario de `bajar_con_un_reintento` y `instalar.md` ("se repite una vez; los demas fallos de red no se reintentan") con lo que se hace. Anadir un test con un servidor que corte la primera peticion antes de cabeceras y espere exito en el segundo intento (el mismo servidor de ensayo sirve de modelo).

## Verificacion de la ronda 1

- IMP-1 (attestation con `gh`), corregido. `instalar()` llama a `comprobar_attestation(gh, &temporal)` (actualizar_cli.rs:451) despues del sha256 y antes de `hacer_ejecutable` y de `comprobar_nuevo`, asi que no se ejecuta el binario nuevo sin la comprobacion. Argumentos de `gh attestation verify <descarga> --repo charliebk/TaskCodeTwo --signer-workflow charliebk/TaskCodeTwo/.github/workflows/release.yml`: los mismos que `install.sh:286` e `install.ps1:216`, y el test los fija con el registro de `gh_falso`. Cubre sin `gh` (`attestation=sin-gh`), con `gh` valido (`comprobada`) y con `gh` que rechaza (codigo 3, `motivo=attestation-no-valida`, nada cambiado). Mutacion: `comprobar_attestation(None, ...)` pone en rojo `attestation_con_gh_ausente_valido_y_que_falla` (crates/tc2/tests/actualizar_cli.rs:424). Limite: `gh` real no ejecutado (sin red a GitHub y sin release publicada); la prueba usa el doble `gh_falso`.
- MEN-1 (`detectar_modo`), corregido. Ahora cada gestor exige su estructura (`node_modules/<pkg>`, `Cellar/<f>/<v>`, `winget\packages|links`, `scoop\apps|shims`, `.cargo\bin`). Los casos de la ronda 1 (`Scoop\bin`, `~/.cargo/otra`, `cellar` suelto) dan `Instalador` en el unitario. Residuo (nota N-2).
- MEN-2 (mensaje en espanol), corregido: "este tc2 lo ha instalado {gestor}".
- MEN-3 (`.old` al arrancar), corregido: `main.rs` llama a `limpiar_old` en Windows y `la_siguiente_ejecucion_borra_el_old` lo fija. Mutacion (quitar la llamada de `main.rs`): ese test pasa a rojo.
- MEN-4 (`doctor`), corregido: la doc y el modulo dicen "lo llamara `doctor` (F3-8)".
- MEN-6 (`--comprobar --version` anterior), corregido: `ok ... cambio=anterior`, cubierto por `comprobar_no_modifica_nada_y_sin_red_da_sinred_con_la_url`.
- MEN-5 (`arreglo` con significados distintos), sin corregir a proposito (decision del coordinador). No revisado en esta ronda; sigue abierto y no bloquea.

## Ejecucion

- Clon nuevo en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-027-r2-calidad`, rama `feature/task-027-f2-12-taskctl-actualizar-cli-y-aviso-de`, HEAD `b3d2e77` (codigo igual a `39ef7a1`). Toolchain 1.100.0.
- `cargo test -p tc2 --test actualizar_cli`: primera pasada **4 de 7** (`actualizar_sustituye_vx_por_vy...`, `attestation_con_gh_ausente...` y `sha256_que_no_casa...` caen con "falta ...\examples\tc2_falso.exe: lanza `cargo test -p tc2`"). Causa: los ejemplos no se compilan con `--test actualizar_cli`. Tras `cargo build -p tc2 --examples`, **7 de 7** en 21,6 s. No hubo cortes de red en estas pasadas.
- `cargo test -p tc2 --lib`: 64/64. `cargo test -p distribucion`: 7/7.
- No ejecutado en esta ronda (fuera de lo pedido): el resto de tests de `tc2`, `cargo clippy` y `cargo fmt`. La suite completa de la ronda 1 no se repite.

## Notas (no abren hallazgo)

- N-1: `cargo test -p tc2 --test actualizar_cli` solo, sin ejemplos compilados, falla (ver Ejecucion). Ya pasaba antes de esta ronda con `tc2_falso`; `gh_falso` anade otro ejemplo. El comando documentado, `cargo test -p tc2`, funciona. Verificado.
- N-2 (residuo de MEN-1, por lectura del codigo, no ejecutado): `cellar` con dos componentes detras (p. ej. `/home/ana/cellar/a/b/tc2`) se toma por Homebrew. Un caso muy raro.
- N-3 (lectura): el rechazo por attestation dice "no es valida ... avisa en issues" tambien cuando `gh` no responde en 60 s o `gh` es una version sin `attestation verify`. Es el mismo comportamiento que `install.ps1`; el mensaje podria decir el detalle sin acusar a la attestation.
- N-4 (lectura): `ejecutar_con_limite` (tiempos de 60 s para `gh` y 30 s para `version --json`) no tiene test: ni el timeout real ni un hijo que escribe mas de 1 MiB.
- N-5: el reintento no tiene test (ver IMP-2).

## Lecturas fuera del contexto permitido

- crates/tc2/src/plataforma/http.rs (funcion `clasificar`, lineas ~343-360): para saber a que fase corresponde cada `ureq::Error` y justificar IMP-2.
- crates/tc2/src/main.rs: para verificar MEN-3 (llamada a `limpiar_old` en Windows).
- instalar/install.sh y instalar/install.ps1 (solo el bloque de attestation): para comparar argumentos y el comportamiento sin `gh`.
- tareas/03-en-revision/TASK-027/revision/informe-revision-1-code-quality-reviewer.md y el `revision.md` del plugin: para el protocolo y la lista de hallazgos de la ronda 1.

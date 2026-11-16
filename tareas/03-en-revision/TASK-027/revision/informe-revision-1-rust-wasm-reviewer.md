# Informe de revision — TASK-027 (ronda 1)

- Commit revisado: 991b24cb3fbdf8c89c7441744cea1e1d03cc4f96
- Revisor: rust-wasm-reviewer
- Veredicto: PENDIENTE (escribelo con: taskctl veredicto TASK-027 aprobada | aprobada-con-correcciones | cambios-solicitados)

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/actualizar_cli.rs:543 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/descarga.rs:258 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/src/actualizar_cli.rs:362 | implementacion |
| MEN-4 | MENOR | abierto | crates/tc2/src/actualizar_cli.rs:494 | implementacion |
| MEN-5 | MENOR | abierto | crates/tc2/tests/actualizar_cli.rs:192 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Sin CRITICOS ni IMPORTANTES.

### MEN-1 — un `.old` bloqueado impide actualizar (Windows)
`sustituir_con` borra `<exe>.old` ignorando el error (`let _ = remove_file`) y luego
`rename(viejo, old)`. Si hay otro `tc2` de la version anterior en marcha desde ese
`.old` (p. ej. de una actualizacion previa), el borrado falla y el renombrado da
«Acceso denegado». Reproducido: `inst\tc2.exe.old` = copia de `ping.exe` en ejecucion,
`tc2 actualizar --cli` contra una release de ensayo → `actualizar error
motivo=no-se-sustituye ... Acceso denegado (os error 5). Queda el binario anterior;
cierra los tc2 abiertos y repite`, exit 1, `tc2.exe` intacto y sin `.descarga`. Cerrado el
proceso, la misma orden actualiza bien (`tc2 9.9.9`). El estado no se corrompe y el mensaje
guia; solo es una limitacion. Opcional: nombre unico para el `.old` o decir cual es el
proceso que bloquea.

### MEN-2 — el reintento no se limita a `sinred` de verdad
`a_escritor` convierte `FalloHttp::Sinred`, `Tls`, `Otro` y `RespuestaInvalida` en
`Fallo::Sinred`, y `bajar_con_un_reintento` reintenta todo `Fallo::Sinred`. Un fallo
determinista (certificado invalido, TLS) se reintenta tambien y, con `ESPERA_DESCARGA` de 120 s
por intento, puede tardar hasta unos 240 s. No reproducido (necesita un servidor TLS malo):
sale de la lectura del codigo. Lo comprobado con un servidor local: corte a mitad de cuerpo →
2 peticiones y `sinred` (4); 404 → 1 peticion `rechazada` (3); 500 → 1 peticion; redireccion y
cuerpo grande → 1 peticion. Es decir, 404, 5xx, hash y limite no se reintentan, como se pide.
Tambien `Descargador::texto` informa de `Grande` cuando `SHA256SUMS` no es UTF-8 (descarga.rs:234),
mensaje engañoso.

### MEN-3 — disco lleno se informa como `sin-permiso`
Un `write_all` que falla por falta de espacio (`Fallo::Escritura`) pasa por `sin_permiso`
(`motivo=sin-permiso`, texto «sin permiso»). El temporal se borra (guarda `Borrar`) y el binario
no se toca, pero el diagnostico lleva a mirar permisos y no el espacio. No reproducido (no se
llena el disco en esta maquina): lectura del codigo. El caso de ruta sin permiso de escritura si
es correcto: `create_new` del temporal falla antes de bajar nada → `rechazada sin-permiso` con
la carpeta.

### MEN-4 — `comprobar_nuevo` sin tiempo limite
`Command::output()` del `version --json` del binario nuevo espera sin limite (y sin tope de
salida). El hash ya se comprobo contra `SHA256SUMS`, asi que solo importa con una release rota;
aun asi bloquearia `actualizar` indefinidamente con el temporal en disco. Lectura del codigo.
Relacionado, tambien MENOR: un `.tc2-<v>-<pid>.descarga` queda huerfano si se mata el proceso
(el pid no se repite, nadie lo limpia), y `--comprobar` ejecuta `limpiar_old` (actualizar_cli.rs:988)
aunque promete no modificar nada (inocuo: Windows ya lo hace en `main`).

### MEN-5 — sin test de integracion del reintento, la redireccion y el limite
`tests/actualizar_cli.rs` cubre sustitucion, hash que no casa, `--comprobar`, plugin y uso, pero
nada ejerce de extremo a extremo el reintento, el rechazo de una redireccion a otro host ni el
limite de tamano (solo hay test unitario de `redireccion_permitida`). Los reproduje a mano (ver
abajo) y todo se comporta bien; faltan como red de regresion.

## Alcance
- Ficheros revisados: Cargo.toml, crates/distribucion/{Cargo.toml,src/lib.rs,src/sha256.rs},
  crates/tc2/{Cargo.toml,examples/tc2_falso.rs,src/actualizar_cli.rs,src/descarga.rs,src/main.rs,
  src/plataforma/http.rs (agente y clasificar),tests/actualizar_cli.rs}.
- Contexto adicional pedido: ninguno (leida la peticion y el diff; `tarea.md` y el codigo del clon).
  No se leyo `planificacion/plan-final.md` entero: la peticion y los criterios de `tarea.md` bastaron.
- Limite JS-WASM: sin cambios; el diff no toca `#[wasm_bindgen]` ni codigo `wasm32`.

## Reproduccion
- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-027-r1-rust`, rama
  `feature/task-027-f2-12-taskctl-actualizar-cli-y-aviso-de` (HEAD 4ecddba, codigo = 991b24c). Borrado al terminar.
- Build: `cargo clippy --all-targets --locked` y `cargo test --locked` compilan el workspace nativo.
  No hay objetivo `wasm32` en este diff (es CLI nativo).
- Formato y lints: `cargo fmt --check` ok; `cargo clippy --all-targets --locked -- -D warnings` ok.
  `unsafe_code = "forbid"` en el workspace; sin `unwrap`/`expect`/`panic!` en produccion de los
  ficheros del diff (solo `unwrap_or_default` y similares).
- Suite: `cargo test --locked --workspace` → todo verde (tc2 lib 63, `tests/actualizar_cli.rs` 5,
  xtask 9, distribucion; 0 fallos). Sin caidas atribuibles a la red local de esta maquina.
- Casos construidos (servidor Python en 127.0.0.1, `tc2.exe` de debug copiado fuera del repo con
  `TC2_BASE_URL`): redireccion a `https://evil.example` → `rechazada redireccion-no-permitida` (3),
  a `http://github.com` → igual; cuerpo chunked de ~72 MB → `rechazada descarga-grande` (3), el
  cliente corta la conexion tras ~64 MiB; corte a mitad → 2 peticiones y `sinred` (4); 404 →
  `rechazada no-publicada`, 1 peticion; 500 → `sinred motivo=http`, 1 peticion. En todos, la
  carpeta quedo sin `.descarga` ni `.old`. Sustitucion real con `tc2_falso` 9.9.9: `ok`, luego
  `tc2 --version` = 9.9.9 y `.old` presente; con `.old` bloqueado, ver MEN-1.
- Hosts permitidos: `redireccion_permitida` exige `https://`, host exacto de la lista, sin
  usuario ni puerto (`github.com.evil.example`, `user@github.com`, `:8443`, relativas: rechazadas);
  `max_redirects(0)` en el agente, asi que ureq no sigue nada por su cuenta; tope de `MAX_SALTOS`.
- Temporal con pid: `.tc2-<version>-<pid>.descarga`, `create_new` (no pisa); guarda `Borrar` lo
  elimina en cualquier salida salvo la correcta.
- Carreras: en Unix la sustitucion es un `rename` atomico; en Windows, si dos `actualizar`
  coinciden, el segundo falla en el primer renombrado (destino `.old` existente o `exe` ausente)
  antes de tocar nada, sin vuelta atras necesaria. Por lectura, no reproducido con dos procesos.
- Mutacion: no se paso `cargo mutants`. El test `fallo_en_el_segundo_renombrado_deja_el_original`
  inyecta el fallo del segundo renombrado y comprueba el original y la ausencia de `.old`.

## Revisado sin hallazgos
- Vuelta atras de `sustituir_con` (rollback correcto; si tambien falla, el error cita donde queda el binario anterior).
- Verificacion antes de ejecutar: sha256 calculado por este proceso y contra `SHA256SUMS` antes de
  `hacer_ejecutable` y de lanzar `version --json`; `.old` y `.descarga` no quedan si el hash no casa.
- Limites de tamano: `Content-Length` declarado y `take(limite + 1)` sobre el cuerpo.
- `distribucion` (tabla de targets, `leer_sumas`/`sumas`, `cli_lock`, `clase_de_tag`) y `sha256.rs`
  (vectores FIPS 180-4 con relleno y multiples bloques); `comparar` de versiones con prereleases.
- `detectar_modo` para rutas de los tres sistemas.

# Informe de revision — TASK-027 (ronda 1)

- Commit revisado: 991b24cb3fbdf8c89c7441744cea1e1d03cc4f96
- Revisor: code-quality-reviewer
- Veredicto: PENDIENTE (escribelo con: taskctl veredicto TASK-027 aprobada | aprobada-con-correcciones | cambios-solicitados)

Veredicto propuesto por el revisor: **aprobada-con-correcciones** (ningun CRITICO; 1 IMPORTANTE y 6 MENOR).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | abierto | crates/tc2/src/actualizar_cli.rs:332 | implementacion |
| MEN-1 | MENOR | abierto | crates/tc2/src/actualizar_cli.rs:47 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/i18n/catalogo/actualizar.rs:34 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/src/main.rs:18 | implementacion |
| MEN-4 | MENOR | abierto | docs/distribucion/instalar.md:91 | implementacion |
| MEN-5 | MENOR | abierto | crates/tc2/src/actualizar_cli.rs:245 | plan |
| MEN-6 | MENOR | abierto | crates/tc2/src/actualizar_cli.rs:309 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

## Verificacion de la puerta y de los criterios

Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-027-r1-calidad`, rama `feature/task-027-...`, HEAD 4ecddba (el codigo es el de 991b24c; despues solo cambia la peticion).

- `cargo test -p tc2 -p distribucion`: distribucion 7/7, `tc2` lib 63/63, `tests/cli.rs` 4/4, `alta` 3/3, `config` 8/8, `plataforma` 4/4. En la primera pasada `tests/actualizar_cli.rs` dio **3 de 5**: `actualizar_sustituye_vx_por_vy...` y `sha256_que_no_casa...` cayeron con `no se pudo bajar http://127.0.0.1:<p>/tc2-9.9.9-win-x64.exe: Peer disconnected` (codigo 4). Es el corte de descargas locales de este equipo que describe el encargo: se repitio dos veces `cargo test -p tc2 --test actualizar_cli` y dio **5/5 las dos veces** (8,4 s y 23 s). No es un fallo del codigo, pero se anota que el reintento unico de `bajar_con_un_reintento` no bastó en esa primera pasada. No he corrido el resto de ficheros de `tests/` de `tc2` (git*, estado*, tarea, plataforma_*): el diff no los toca.
- Criterio 1: `actualizar_sustituye_vx_por_vy_servida_en_127_0_0_1` existe, comprueba la salida exacta, `tc2 --version` = 9.9.9 y el hash del binario instalado; pasa. La vY es un doble (`tc2_falso`), no un tc2 real; el caso real solo lo cubre el job de CI, que sigue sin verificarse hasta el push (la tarea y la doc lo dicen).
- Criterio 2: `sha256_que_no_casa_es_rechazada_y_no_toca_nada` (es y en): codigo 3, `motivo=sha256-no-casa`, hash intacto, carpeta sin restos. Mutacion: desactivar la comparacion `obtenido != publicado.sha256` pone el test en rojo. Pasa y discrimina.
- Criterio 3: `comprobar_no_modifica_nada_y_sin_red_da_sinred_con_la_url` cubre `ok`, `sin-cambios` y `sinred` (4) con la URL, en es y en; el unitario `comparar_versiones_con_prerelease` cubre el orden. Pasa. A mano: `TC2_BASE_URL=http://127.0.0.1:1 tc2 actualizar --cli --comprobar` da `actualizar sinred url=http://127.0.0.1:1/SHA256SUMS`, codigo 4, 2 s.
- Criterio 4: el unitario `version_json_exactamente_...` y `cli.rs::version_json_del_binario_con_su_sha256` (otro proceso calcula el hash). Pasan. A mano: `{"version":"0.0.0","protocolo":1,"target":"win-x64","sha256":"037c..."}`.
- Criterio 5: `compatible_fuera_de_rango_es_rechazada_con_el_arreglo` (es, en y `--json`). Pasa. A mano con `--protocolo 2-3`: codigo 3; stdout `compatible rechazada protocolo=1 rango=2-3 motivo=protocolo-incompatible arreglo=tc2%20actualizar%20--cli`; stderr (es y en) con `motivo=protocolo-incompatible arreglo="tc2 actualizar --cli"`. La linea literal del criterio esta en stderr; en stdout el valor sale percent-encoded por la regla del contrato (`escapar_valor`, la misma de `nota=con%20espacio`). Desviacion aceptable y coherente con el contrato; el test la fija.
- Criterio 6: `detectar_modo_con_rutas_de_los_tres_sistemas` (unitario) y `el_tc2_de_la_cache_del_plugin_no_se_sustituye` (integracion, sin peticiones al servidor). Pasan.
- Criterio 7: `cargo xtask cli-lock` probado con un `SHA256SUMS` sintetico de 6 targets: con `--tag v0.0.0` genera el `cli.lock`; con `--tag v2.0.0` falla con `el tag «v2.0.0» no es v0.0.0 ni v0.0.0-<sufijo>`, como dice `releases.md`. No he corrido `cargo xtask verificar` entero (queda fuera de la parte revisada).
- Mutaciones: (a) quitar el borrado en `limpiar_old` pone rojos el unitario de vuelta atras y el test del criterio 1; (b) quitar la llamada de `main.rs` **no pone nada en rojo** (ver MEN-3).

## Desviaciones del plan

- `tc2::descarga` en lugar de `tc2::distribucion::descarga`: sin consecuencias, la caja `distribucion` queda como logica pura sin E/S, y el modulo de red va en `tc2`. Correcto.
- `Modo::Gestor(&'static str)` en lugar de `Desconocido`: mejora (da el mensaje con el gestor) y la doc lo recoge, pero genera MEN-1 y MEN-2.
- `.old` borrado en `main.rs` (Windows): necesario para que la «siguiente ejecucion» lo limpie; documentado, pero sin test local que lo proteja (MEN-3).
- Arreglo percent-encoded en stdout: ver criterio 5; sin hallazgo. Si hay hallazgo en que el valor de `arreglo` no tiene el mismo significado en las dos ordenes (MEN-5).
- Del plan **no esta** la attestation opcional con `gh` (decision 4 de «Lo que necesita decision de una persona»): IMP-1.

## Detalle y reproduccion

### IMP-1 — `actualizar --cli` no comprueba la attestation aunque el plan lo decidio
- El plan (decision 4, aceptada por Carlos) dice «Attestation opcional si hay `gh` (paridad con F2-11)» y los riesgos aceptados: «Attestation solo si hay `gh`». `instalar/install.sh` e `install.ps1` la comprueban con `gh attestation verify --signer-workflow ...` cuando hay `gh` con sesion. En `crates/tc2` y `xtask` no hay ninguna referencia a attestation (`grep -rn -i attestation crates xtask/src` solo da resultados en `instalar/`): `instalar()` termina en el sha256 de `SHA256SUMS`.
- Efecto: el camino de actualizacion tiene menos garantia que el de instalacion. Lo unico que protege es un hash de la misma release, es decir, el riesgo que el plan acepto solo para el caso sin `gh`.
- La doc lo dice a medias: `instalar.md:118` abre con «Como en los instaladores, el hash prueba...», y acaba con «La attestation se comprueba con `gh attestation verify`» (manual). No dice que `tc2 actualizar --cli` no la hace ni que es una decision aplazada. Ni la tarea ni `## Autorrevision` lo recogen como desviacion.
- Correccion posible: implementarla (invocar `gh` si existe, igual que el instalador) o registrar la desviacion en la tarea/ADR, y corregir la frase de la doc. Si la persona acepta aplazarla, baja a MENOR.
- Reproduccion: lectura mas `grep` (hecho). No he podido ejercitar el caso con `gh` real (sin release publicada).

### MEN-1 — `detectar_modo` mira cualquier componente de la ruta
- `actualizar_cli.rs:47-72` recorre todas las partes de la ruta, y `scoop`, `cellar`, `homebrew`, `winget`, `.cargo`, `node_modules` en cualquier posicion bastan para dar `Gestor`. Reproduccion: copiar `tc2.exe` a `...\taskcodetwo\fp-r1\Scoop\bin\tc2.exe` y lanzar `actualizar --cli --comprobar` da `actualizar rechazada motivo=gestionado-por-gestor gestor=scoop`, codigo 3, aunque no lo gestiona Scoop. Un usuario cuyo directorio se llame `scoop` o que instale con `TC2_INSTALL_DIR` bajo `~/.cargo/` se queda sin poder actualizar. Es un caso de borde; el sistema de rutas del test no cubre rutas «normales» con esos nombres como padres lejanos. Mejor anclar cada gestor a su patron (p. ej. `scoop/apps/`, `Cellar/<formula>/`, `WinGet/Packages/`, `node_modules/<paquete>/`, `.cargo/bin/`).

### MEN-2 — Mensaje `GESTIONADO_POR_GESTOR` en espanol con un verbo equivocado
- `catalogo/actualizar.rs:34`: «este tc2 lo instalo {gestor}» se lee «yo lo instalo»; deberia ser «lo ha instalado» o «lo instalo» con tilde segun la convencion del catalogo (el catalogo entero esta sin tildes, comprobado con `grep`; por eso conviene «ha instalado»). Visto en la salida real de la reproduccion de MEN-1. El ingles esta bien.

### MEN-3 — El borrado del `.old` en `main.rs` no lo protege ningun test local
- Mutacion: sustituir `tc2::actualizar_cli::limpiar_old(&exe);` de `main.rs:18` por `let _ = exe;` deja `cargo test -p tc2 --test actualizar_cli` en 5/5 verde. El test del criterio 1 acepta `tc2.exe.old` tras `tc2 --version` («quedan.iter().all(|r| r == "tc2.exe.old")») y lo borra el mismo con `limpiar_old`. La promesa de la doc («el `tc2.exe.old` lo borra la siguiente ejecucion de `tc2`») solo la comprueba `probar-actualizar.sh` (lineas 181-183), que esta sin verificar en CI. Sugerencia: en Windows, tras la segunda ejecucion (`--version`), exigir `restos(&dir, &exe).is_empty()`.
- (La limpieza dentro de `actualizar()` y tras `instalar()` si esta cubierta: la mutacion de `limpiar_old` da rojo.)

### MEN-4 — La doc y el modulo dicen que `doctor` llama a `--comprobar`; `doctor` no existe en `tc2`
- `instalar.md:91` («lo llaman `doctor` y, si se quiere, un hook de inicio de sesion») y `actualizar_cli.rs:10`. En `crates/tc2/src` solo hay comentarios que hablan de `doctor` (F3-8); no hay orden. Es una intencion, presentada como hecho. Reescribir como «lo llamara `doctor` (F3-8)» o similar.

### MEN-5 — `arreglo` significa cosas distintas segun la orden
- `compatible` pone `arreglo=tc2 actualizar --cli` (un comando), pero el rechazo del plugin pone `arreglo=actualizar-el-plugin` (`actualizar_cli.rs:245`, un identificador) y el texto humano trae el comando real en stderr. Un consumidor (el lanzador del plugin) que lea `arreglo` en stdout recibira un comando en un caso y un slug en otro. El criterio 6 pide «el arreglo del plugin» sin precisar el formato (ambiguedad del plan). Unificar: o siempre texto humano/comando, o documentar el vocabulario cerrado.

### MEN-6 — `--comprobar --version <anterior>` da `ok ... disponible=<mas vieja>`
- `actualizar_cli.rs:309-315`: con `--version`, `nada_que_hacer` solo es cierto si las versiones son iguales, asi que `--comprobar --version 0.0.1` desde la 2.0.0 imprime `actualizar ok actual=2.0.0 disponible=0.0.1`. La doc dice que `ok` es «si hay una mas nueva». El `actualizar --cli --version <anterior>` real si baja (documentado como «tambien hacia atras»); solo el texto de `--comprobar` es enganoso. Documentarlo o separar el caso. No lo cubre ningun test.

## Mensajes es/en
Revisados los de `catalogo/actualizar.rs` y los de `compatible`/`actualizar` en las dos lenguas (`TC2_IDIOMA=en`): paridad de claves y de significado, mismo stdout y mismo codigo. Los terminos de orden (`actualizar`, `motivo=`, `arreglo=`) quedan literales a proposito. Solo MEN-2.

## Documentacion frente a comportamiento
`instalar.md` y `releases.md` coinciden con lo ejecutado: `version --json`, `compatible` (rechazo y codigos), `--comprobar` sin red (`sinred`, 4), `sin-cli` (`uso`, 2), `TC2_BASE_URL` invalida (`uso motivo=base-url`) y `cli-lock` (6 targets, tag que case). Desajustes: IMP-1 (attestation), MEN-4 (`doctor`) y MEN-6. Sin red real no pude comprobar `actualizar --cli --comprobar` contra GitHub: con el proxy/permisos de este equipo da `sinred` con la URL de `releases/latest/download/SHA256SUMS` (`os error 10013`), que es el comportamiento documentado.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/actualizar_cli.rs`, `descarga.rs`, `version.rs`, `lib.rs` (diff), `main.rs`, `i18n/catalogo/actualizar.rs`, `tests/actualizar_cli.rs`, `crates/distribucion/src/lib.rs`, `xtask/src/main.rs` (diff), `examples/tc2_falso.rs`, `instalar/install.sh` e `install.ps1` (solo la busqueda de la attestation y el directorio por defecto): el encargo pedia comprobar criterio por criterio que cada test existe y prueba lo citado, las desviaciones del plan (`tc2::descarga`, `Modo::Gestor`, `.old` en `main.rs`, arreglo escapado) y la paridad con los instaladores, y nada de eso se ve desde el diff de los 4 ficheros del contexto.

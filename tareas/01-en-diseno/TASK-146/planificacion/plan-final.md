# Plan — TASK-146: Auditoria 4D: A8 transversal, actualizar_cli, instalacion, contrato, dato y xtask (7 IMPORTANTE)

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 110 min (de la rama abierta al cierre con merge; senales: 6 ficheros de codigo previstos, todos de test salvo el doble `tc2_falso`; dos crates, `tc2` y `xtask`; 8 criterios; `cargo xtask verificar` largo y la revision reproduce 9 mutantes)

Regla de una hora: pasa de 60 min, pero por la consigna de Carlos del 2026-11-23 (modo automatico, opcion recomendada) la tarea se hace sin trocear.

No cambia codigo de produccion. Son 7 arreglos sueltos en los tests de cada modulo y una extension pequena del doble `tc2_falso`. Sin abstraccion nueva entre modulos. Un commit por ID:

1. **A11-03** (`crates/tc2/src/dato.rs:207-227`): `plano_deja_el_texto_inerte` anade `assert_eq!` con `plano("&#x202E; **n** _x_ ~~t~~", 400)` y su salida exacta (`\&\#x202E; \*\*n\*\* \_x\_ \~\~t\~\~`, a confirmar ejecutando). Escapes en `dato.rs:76` (`*`), `:77` (`_`), `:86` (`~`), `:88` (`&`).
2. **A11-04** (`crates/tc2/src/contrato.rs:376-397`): `linea_de_contrato_y_json_con_las_mismas_claves` anade un valor `"a\u{1b}[31m\u{7}b"`; exige `"a\u001b[31m\u0007b"` en el JSON y que `serde_json::from_str::<serde_json::Value>` sea `Ok`. El brazo a matar esta ahora en `contrato.rs:345` (la auditoria decia `:336`).
3. **A8-15 y A11-02** (`crates/tc2/src/instalacion.rs:280-374`): la tabla de `:301-315` pasa a `(ruta, gestor, orden)` y el `assert!(!…is_empty())` de `:317` se cambia por `assert_eq!(orden_del_gestor(gestor), orden)` con los cinco textos de `:79-83`; dos filas nuevas: `/opt/homebrew/bin/tc2` → homebrew y `C:\Users\ana\AppData\Local\Microsoft\WinGet\Links\tc2.exe` → winget. En `rutas_del_entorno_por_sistema` un `Entorno` de Windows con `TC2_INSTALL_DIR` igual a la carpeta por defecto exige un solo elemento en `dirs_de_instalacion()`.
4. **A8-13** (`crates/tc2/src/actualizar_cli.rs:787-852`): ayudante privado `fn shell(windows: &str, unix: &str) -> Command` en el `mod tests` (`cmd /c` o `sh -c`, con `TC2_PRUEBA=1`), sustituye las tres copias.
5. **A9-02** (`crates/tc2/tests/actualizar_cli.rs` y `crates/tc2/examples/tc2_falso.rs`): `tc2_falso` responde `TC2_FALSO_SHA256` si existe (precedente `GH_FALSO` en `gh_falso.rs`). Test nuevo `el_binario_nuevo_que_no_responde_bien_es_rechazado_y_no_toca_nada` con tres casos (no arranca, otra version, otro sha256) que exigen `actualizar rechazada motivo=binario-no-arranca`, codigo 3, sha256 del instalado intacto y sin restos.
6. **A9-03** (`xtask/src/workflows.rs:2712-2721`, `:2743`): se borra el cierre `runs_de` y se usa `runs_del_job` (`:2162-2173`).
7. `cargo xtask verificar` y el `## Resultado` con los 7 IDs y su commit.

## Reutilizacion (Regla 4D)

Lo que ya existe y se usa: en `crates/tc2/tests/actualizar_cli.rs`, `Carpeta`, `instalar_vx`, `sha256`, `binario_vy`, `release`, `servir`, `lanzar_con`, `restos`, `VERSION_Y`; `comun::http_mock::{serie, Mock}` y `comun::texto`; el doble `tc2_falso` (se extiende, no se crea otro); `runs_del_job` y `normalizar` de `xtask/src/workflows.rs`; las tablas de tests de `instalacion.rs` (se amplian).

Nuevo, con motivo: `fn shell` en `actualizar_cli::tests` — `verificacion::ejecutar::orden_de_shell` es privada de produccion, elige shell por configuracion y usa `raw_arg` con otra forma del guion; hacerla `pub(crate)` acoplaria modulos; `tests/comun` no es accesible desde tests unitarios. El test de A9-02 (no hay ninguno que llegue a `binario-no-arranca`). La variable `TC2_FALSO_SHA256` (sin ella no se prueba otro sha256).

No se crea `pasos_del_job` (cinco lecturas por debajo del umbral; fuera de alcance).

## Lo que el rol no cubrio

- Comprobacion empirica de los mutantes (rol de testing): se hace en la implementacion.
- La salida exacta de `plano`: se fija ejecutandola.
- Tiempos en Windows del test de A9-02 y el arranque de `b"binario-y"` en macOS/Linux (rol de riesgos).
- Si los cinco textos de `orden_del_gestor` son los de F1-12 D1a (rol de dominio): el test fija los de hoy.
- Criterio 8 no ejecutado en el diseno.

## Riesgos aceptados y que los contiene

- Modo «mentiroso» del doble por variable de entorno: `tc2_falso` no se distribuye y solo la pone el test con `lanzar_con`.
- Rutas de `TC2_INSTALL_DIR` comparadas como `PathBuf`: misma forma con `/` que el caso existente, que ya pasa en Windows y Linux.
- A9-03 cambia «algun run contiene» por «el texto unido contiene»: equivalente para `tc2 hallazgos validar`, igual o mas estricto para `needs.` tras `normalizar`; se pierde el detalle del `run` concreto, el mensaje sigue nombrando el job.
- Tres actualizaciones completas mas en `tests/actualizar_cli.rs`: binario pequeno en el caso que no arranca y `serie()` ya serializa.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- A8-15: se amplia `detectar_modo_con_rutas_de_los_tres_sistemas` (2 filas). Mutantes: quitar `instalacion.rs:56`; dejar `:62` solo con `packages`.
- A9-02: 1 test nuevo con 3 casos. Mutantes: `:406` → `Ok::<(), String>(())`; `:738` → `if false`. `cargo test -p tc2 --test actualizar_cli`.
- A11-02: se amplian `detectar_modo_…` (columna orden) y `rutas_del_entorno_por_sistema` (deduplicacion). Mutantes: `:79` cambiado; `:229-231` → `v.push(d)`.
- A11-03: 1 asercion en `plano_deja_el_texto_inerte`. Mutantes: quitar `dato.rs:88`; quitar `:76`.
- A11-04: se amplia `linea_de_contrato_y_json_con_las_mismas_claves`. Mutante: quitar `contrato.rs:345`.
- A8-13 y A9-03: refactorizacion sin test nuevo; siguen en verde `cargo test -p tc2 --lib actualizar_cli::tests` y `cargo test -p xtask`.
- Ningun test pesado. Final: `cargo xtask verificar` con `CARGO_TARGET_DIR` propio.

## Lo que necesita decision de una persona

1. Como provocar «otro sha256» en A9-02: recomendada variable `TC2_FALSO_SHA256` en `tc2_falso`.
2. `pasos_del_job`: recomendada fuera de alcance.
3. `docs/auditoria-4d.md`: recomendado no tocarlo; los IDs se cierran en el `## Resultado`.

Respuestas de la persona (Carlos, consigna del 2026-11-23: modo automatico con la opcion recomendada): 1) `TC2_FALSO_SHA256`; 2) fuera de alcance; 3) no se toca la auditoria.

# Plan — TASK-150: Auditoria 4D: utilidades comunes de E-S y tiempo (4 IMPORTANTE)

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 150 min (de la rama abierta al cierre con merge; senales: unos 16 ficheros de codigo, tres modulos raiz privados nuevos o ampliados (`lectura`, `escritura`, `proceso`) y uno publico (`fecha`), 5 criterios)

Regla de una hora: pasa de 60 min, pero por la consigna de Carlos del 2026-11-23 (modo automatico, opcion recomendada) la tarea se hace sin trocear.

Las utilidades comunes van a modulos hoja de la raiz del crate (precedente `mod lectura;`, `lib.rs:51`): cada dueno depende de ellos y ellos de ninguno. Ningun contrato de salida cambia salvo la decision 3. Un commit por ID:

1. **A8-04 (absorbe A7-02).** `crates/tc2/src/escritura.rs` (privado) con `pub(crate) fn atomica(ruta: &Path, contenido: &[u8]) -> io::Result<()>` y `atomica_con(.., esperas)` privada para tests. Cuerpo de `huella.rs:89-117` (con `ESPERAS_RENAME`) mas lo de `desinstalar/mod.rs:537-554`: sigue enlaces, conserva permisos del destino si existe, crea la carpeta padre, temporal con un solo nombre (`<nombre>.tc2-tmp-<pid>`), y borra el temporal si falla `write`, `set_permissions` o `rename`. Lo usan `estado/frescura.rs:192`, `identidad.rs:275`, `progreso/fichero.rs:207,215`, `confianza/mod.rs:172-177` y `desinstalar/mod.rs:530`. Se borran `huella::escribir_local`/`_con`, `desinstalar::escribir_atomico` y la escritura en linea de `confianza`; los tests de `huella.rs:237-283` pasan a `escritura.rs`; se corrige la doc de `huella.rs:15` y `:83-88`.
2. **A8-17.** `lectura.rs` gana `acotado(r: impl Read, tope) -> io::Result<Option<Vec<u8>>>` (unico `take(tope.saturating_add(1))`), `con_tope(ruta, tope)` y `texto_con_tope(ruta, tope)` (UTF-8 estricto, `InvalidData`). `lectura::fichero` usa `acotado`. Sustituye `identidad.rs:229-240`, `estado/frescura.rs:172-181`, las dos `leer_con_tope` (`hallazgos/orden.rs:216-224`, `seleccion/orden.rs:284-292`), `progreso/fichero.rs:124-133`, `progreso/mod.rs:51-59` y el stdin de `seleccion` (`lib.rs:586-594`, `seleccion/orden.rs:430-434`). No se tocan las lecturas que truncan sin rechazar (`inventario.rs:379-385`, `verificacion/ejecutar.rs:141,182`, `progreso/mod.rs:371`) ni `descarga.rs:377-390`.
3. **A9-01.** `crates/tc2/src/proceso.rs` (privado) con `grupo_propio(c: &mut Command)` (`process_group(0)` en Unix), `esperar_con_plazo(hijo, plazo) -> io::Result<Option<ExitStatus>>` (sondeo `SONDEO` 10 ms; al plazo o si falla `try_wait`, `matar_arbol`, `kill` y `wait`) y `matar_arbol` privada movida de `ejecutar.rs:396-417`. `verificacion::lanzar` traduce a `Fin::Codigo/Timeout`; `actualizar_cli::ejecutar_con_limite` usa `grupo_propio` y `esperar_con_plazo`, conserva sus hilos lectores y su doc se reescribe con lo que hace de verdad.
4. **A8-03.** `crates/tc2/src/fecha.rs` (`pub mod fecha`) con `pub fn instante(segundos: u64) -> String` y `pub(crate) fn fecha_de_dias(i64)` (cuerpos de `alta.rs:724-749`). Se borran `alta::instante_utc`, `alta::fecha_de_dias` y `toma::operacion::instante`; la reexportacion de `toma/mod.rs:44` queda en `Tomar, desde_anotacion`. Llamadores a `crate::fecha::instante` en produccion (`ciclo/operacion.rs`, `plan/aprobar.rs`, `plan/operacion.rs`, `revision/operacion.rs`, `estado/frescura.rs`, `estimacion/mod.rs`) y tests (`ciclo/prueba.rs`, `progreso/calculo.rs`, `tests/comun/remoto.rs`, `tests/comun/tareas.rs`, `tests/soporte.rs`, `tests/toma.rs`). `alta::hoy` usa `fecha::fecha_de_dias`. En `tests/identidades.rs` se borra `iso` y `iso_formatea_como_github` usa `tc2::fecha::instante`. `xtask/src/main.rs:428-448` (`fecha_utc`) no cambia: una linea de doc con el motivo (xtask solo depende de `distribucion`).
5. `cargo xtask verificar` en verde sin solapar, y el `## Resultado` con A8-04 (y A7-02), A8-17, A9-01 y A8-03, cada uno con su commit.

## Reutilizacion (Regla 4D)

Se usa: `huella::escribir_local_con` y `ESPERAS_RENAME`; la rama de enlace y permisos de `desinstalar`; `lectura::fichero`, `Motivo`, `NoSeLeyo`; `matar_arbol`, `SONDEO` y `process_group(0)` de `verificacion/ejecutar.rs`; `alta::fecha_de_dias` e `instante_utc` (se mueven). Tests: `huella.rs:237-283`, `lectura.rs` `no_utf8_se_rechaza`, los dos del nieto de `actualizar_cli.rs`, los asserts de fechas de `alta.rs` y `toma/operacion.rs` (se juntan en `fecha.rs`), `iso_formatea_como_github`.

Nuevo, con motivo: `escritura.rs` (`huella` es el catalogo de nombres del repo, ADR F1-12 D1a; el almacen de confianza y los ficheros de la persona no lo son); `proceso.rs` (`verificacion` no debe depender de `actualizar_cli` ni al reves); `fecha.rs` (fecha repartida entre `alta` y `toma`; los tests necesitan ruta publica). En `lectura` solo se anaden funciones.

Se borran 2 `leer_con_tope`, 4 lecturas en linea, 3 escrituras atomicas, 1 espera con plazo, 2 formateos de instante y la copia de Hinnant del test.

## Lo que el rol no cubrio

- Riesgos: en Unix `process_group(0)` saca a `gh`/`tc2` del grupo de primer plano (Ctrl+C no les llega directamente); `taskkill` anade latencia al matar por plazo; seguir enlaces y conservar permisos vale ahora tambien para `CARPETA_COMUN` y el almacen; los lectores `lossy` ahora rechazan.
- Testing: los tests `cfg(unix)` no corren en esta maquina Windows y no hay push.
- Dominio: no validado si un informe no UTF-8 debe ser `invalido` o `sin-informe`.
- Criterio 5 solo se comprueba ejecutando.

## Riesgos aceptados y que los contiene

- UTF-8 estricto en `hallazgos`/`seleccion` (antes U+FFFD): decision 3 y un test por camino; `seleccion` ya era estricta por stdin.
- Nombre del temporal distinto: nadie lo busca por nombre (grep de `.tmp-`/`tc2-tmp` en `crates/`).
- Un `symlink_metadata` y un `metadata` de mas por escritura de progreso: despreciable frente al `rename`.
- `ejecutar_con_limite` ya no deja vivo al nieto al cumplirse el plazo: los dos tests existentes y uno nuevo del arbol.
- No se tocan: `inventario::dias_desde_epoca`, `fecha_utc` de xtask, las lecturas que truncan, el lugar de `ejecutar_con_limite`.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- A8-04 (`escritura.rs`): (a) y (b) movidos de `huella`; (c) nuevo: `rename` que falla (destino carpeta no vacia, `atomica_con(.., &[])`) da `Err`, carpeta igual y sin temporal; (d) nuevo `cfg(unix)`: conserva `0o600` y el enlace. Mutantes: quitar el reintento (b), quitar `remove_file` (c), quitar `set_permissions` o no seguir el enlace (d, solo Unix).
- A8-17 (`lectura.rs`): (a) frontera `tope`/`tope+1`; (b) `texto_con_tope` con `[b'a', 0xff]` da `InvalidData` y `con_tope` da los bytes; (c) 1 test de `informe_revisor` con informe no UTF-8 → `invalido`. Mutantes: `>` por `>=` y quitar `+1` (a); `from_utf8_lossy` (b).
- A9-01 (`proceso.rs`): hijo con nieto en segundo plano que retiene stdout; `esperar_con_plazo(1 s)` da `None` y la tuberia llega a EOF en menos de 5 s. Mutante: quitar `matar_arbol`.
- A8-03: (a) asserts de fechas juntos en `fecha.rs`; (b) `iso_formatea_como_github` con `tc2::fecha::instante`. Mutantes: `i64::from(m <= 2)` por 0 (a con `2000-02-29`, b con `2099-01-01`); `%` por `/` en los segundos (a).
- Criterio 5: `cargo xtask verificar` sin solapar. Ningun test pesado ni de red.

## Lo que necesita decision de una persona

1. Donde va la escritura atomica: recomendado `crate::escritura`.
2. Nombre del temporal: recomendado `<nombre>.tc2-tmp-<pid>`.
3. Bytes no UTF-8: recomendado rechazo estricto, con `informe=invalido` en los lectores de informe.
4. Stdin de `seleccion` en el `acotado` comun: recomendado si.
5. Espera con plazo: recomendado `crate::proceso`, `ejecutar_con_limite` se queda en `actualizar_cli`.
6. Fechas: recomendado `pub mod fecha`, sin reexportar en `toma`.

Respuestas de la persona (Carlos, consigna del 2026-11-23: modo automatico con la opcion recomendada): 1) `escritura`; 2) `.tc2-tmp-<pid>`; 3) estricto, con `invalido` en los informes; 4) si; 5) `proceso`, y `ejecutar_con_limite` se queda; 6) `pub mod fecha`, sin reexportacion.

# Plan — TASK-145: Auditoria 4D: A7 config, init, desinstalar, i18n, confianza y ciclo (8 IMPORTANTE)

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 210 min

Señales: unos 18 ficheros, 3 nuevos (`ciclo/prueba.rs`, `plan_dato.rs`, el test de A7-11); 3 capas (tests unitarios; producción de `init`, `desinstalar`, `confianza`, `config`; i18n y guarda); 7 criterios y 8 IDs, un commit por ID. A7-01 y A7-07 tocan el bucle de aplicar de dos órdenes.

**Fichas vivas (develop):** A7-04 en parte (TASK-148 ya llevó `estado`/`motivo`/`texto` a `estado::prueba`; siguen copiados `md`, `fila`, `AHORA` y `ana()` en `liberar.rs` y `operacion.rs`); A7-05 (`ciclo/mod.rs:473-496` y `:538-559`: montaje bare + clon + commit + push + fetch copiado); A7-01 (`init/mod.rs` y `desinstalar/mod.rs`, mismos tramos); A7-03 (`confianza/mod.rs:198-242` y `:246-275`); A7-07 (cinco ubicaciones en `init/aplicar.rs`, `init/mod.rs:470`, `desinstalar/proyecto.rs:373-376`, `desinstalar/mod.rs:542`); A7-09 (`config/mod.rs:534-544`: `Err(_)` → `SinConfig`); A7-10 (`guarda.rs` con 4 formas en `PROHIBIDOS`); A7-11 (`tests/init.rs:1599-1660` solo provoca `ramas-por-delante`; `inventario.rs:347-352` sin ejercitar). Fuera: A7-02 (→ A8-04, TASK-150), A7-06 (absorbida por `tests/comun` instantánea), A7-08 (MENOR).

**Orden** (un commit por ID; cada D4 con su test primero):
1. A7-04 (divergente): `#[cfg(test)] pub(crate) mod prueba` en `ciclo/` con `AHORA`, `ana()`, `fila(fase, por)` y `md(id, estado, asignado, Extra)` (`Extra { plan_aprobado, hace }` con `Default`). **Cambia:** el motivo del fixture `descartada` queda en un solo literal, `"el de entonces"` (decisión 3); los esperados se derivan de `antes` (se confirma corriendo los 23 usos de `md(`).
2. A7-05: `fn repo_con_remoto(nombre, primer_commit, ramas)` privada en el `mod tests` de `ciclo/mod.rs`, sobre `git::prueba::{aislar, sh, entorno}`.
3. A7-03: cálculo privado `evaluar(repo, almacen, clave, orden, incluye, op, cache) -> Result<Calculo, Informe>` con `Calculo::guardar` y `Calculo::evaluacion`; `estado` y `confiar` mantienen su firma; `escribir_almacen` no se toca (A8-04).
4. A7-01: `src/plan_dato.rs` (privado) con `trait Paso { aplicable, aplazar, ruta }`, `rechazar` y `aplicar_en_orden`; `init::Accion` y `desinstalar::Accion` implementan `Paso`; `leer` y `no_se_lee` pasan a `crate::lectura` (`opcional(op, ruta)`, `no_se_lee(op, ruta, e)`); `init::NO_SE_LEE` y `desinstalar::NO_SE_LEE` se funden en `catalogo::comun::NO_SE_LEE`; el `current_dir` usa `tablero::carpeta_actual(NOMBRE)`; una sola `t()`. Se actualiza el `//!` de `lectura`.
5. A7-10 y A7-07 (test primero): ampliar `ningun_literal_fuera_del_catalogo` con (a) `Err("` con un literal con espacio (prosa) y (b) toda mención de `EXTERNO` fuera de `i18n/` en el recuento cerrado; rojo contra el código actual (`desinstalar/mod.rs:542`, `init/aplicar.rs:57`) y con el mutante de la ficha. Después A7-07: `aplicar` de init y desinstalar y los ayudantes de `desinstalar/proyecto.rs` devuelven `Result<(), Mensaje>` (`.con_mensaje("error", m)`); `CAMBIO_DESDE_EL_PLAN` sube a `comun`; `"aplicar-puertas sin plataforma"` pasa al catálogo; errores de E/S con un único `plan_dato::de_io(&io::Error) -> Mensaje` (listado en `EXTERNOS`); los tres `texto(i)` en `Idioma::Es` se sustituyen por `Informe::mensaje(&self) -> Mensaje` (`pub(crate)`, en `contrato.rs`), que generaliza `estado/lectura.rs::fallo_de_lectura` (el permiso de `EXTERNOS` pasa a `contrato.rs`). **Cambia:** el stderr con `TC2_IDIOMA=en` deja de mezclar español; stdout, motivos y códigos no cambian.
6. A7-09 (test primero): unitario con `.taskcode/config.yml` en UTF-16LE con BOM (hoy `sin-config`); después `NotFound` → `SinConfig` y cualquier otro error → `lectura::no_se_lee(NOMBRE, ruta, &e)` (`config error ruta=.taskcode/config.yml`, «no se puede leer {ruta}: {error}»). **Cambia:** con un fichero ilegible, `tc2 verificar` y `tc2 confiar` pasan de `rechazada motivo=sin-config` (3) a `error` (1), sin el arreglo de `tc2 init` (decisión 1).
7. A7-11 (test primero): caso «develop sin publicar» (`repo_listo` y `update-ref -d refs/heads/develop` en el bare): `motivo=ramas-sin-remoto`, `faltan` con `develop`, sin escrituras en el mock; se corrige el docstring; rojo con `if false && !sin_remoto.is_empty()`.

## Reutilizacion (Regla 4D)

Lo que ya existe y se usa (helpers, funciones, fixtures, tests), buscado en el repo antes de disenar; y lo que se crea nuevo, con el motivo de no reutilizar:

- `estado::prueba::{estado, motivo, texto, EnMemoria}`, `git::prueba::{R, aislar, entorno, sh}`, `contrato::LineaAccion` (precedente de plan como dato), `estado/lectura.rs::fallo_de_lectura` (se generaliza), `Mensaje::con_mensaje`, `tablero::carpeta_actual`, `crate::lectura`, `catalogo::init::CAMBIO_DESDE_EL_PLAN` (se mueve); en tests `repo_listo`, `aplicar`, `escrituras`, `github_mock`, `Fixture::crudo`, `comun::*`.
- Nuevo: `ciclo/prueba.rs` (no hay soporte común de tests para `ciclo`); `src/plan_dato.rs` (el bucle de aplicar no está en ningún sitio común; `contrato` es la capa de salida); `Informe::mensaje` (sustituye 4 conversiones); `lectura::opcional`/`no_se_lee` (amplían un módulo existente).
- Fuera de A7 (se ven y se dejan): `Carpeta` en `ciclo/mod.rs:448` y `plan/mod.rs:543`; los `md(` de `plan`, `revision`, `toma`; los `Err(format!(` de `actualizar_cli` y `verificacion/informe.rs`.

## Lo que el rol no cubrio

- Testing: cobertura y huecos de la heurística «literal con espacio».
- Riesgos: permisos en Windows, otros `ErrorKind`, si el `error` de código 1 en `config` rompe al hook de verificar (`verificacion/ejecutar.rs:66`), carrera en «cambió desde el plan».
- Dominio: si F1-4 permite un `error` sin `motivo` en `config`; código 1 frente a 3 en `verificar`/`confiar`.
- Criterios: A7-09 dice «otro motivo con la causa»; la propuesta da `Resultado::Error` sin `motivo` (decisión 1). El inglés entero de A7-07 se cubre con tests unitarios.

## Riesgos aceptados y que los contiene

- Se mantiene la API pública (`init::{Accion, Decision, Hecho}`, `desinstalar::{Accion, Decision, Efecto, Hecho}`, `confianza::{estado, confiar}`, `config::del_arbol`): no hay `Accion<D, E>` genérico; quedan los builders `motivo`/`orden` (4 líneas) y `Hecho::solo`, bajo el umbral de D1; el trait `Paso` comparte lo que puede divergir.
- Dependencia nueva `init`/`desinstalar` → `tablero` por `carpeta_actual` (ya pública; su sitio natural es A8-14, MENOR).
- La guarda textual es heurística: lista cerrada de excepciones con motivo (decisión 2).
- Caché de A7-03: `estado` sigue sin propagar el fallo de escritura de la caché (condición explícita y comentario).

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- A7-04: 0 (tests de `ciclo::liberar` y `ciclo::operacion` en verde).
- A7-01 y A7-03: 1 unitario de `plan_dato` (tras el primer fallo, lo de detrás queda `pendiente motivo=no-aplicada`, la que falla `error`; `rechazar` aplaza las aplicables).
- A7-05: 0 (dos unitarios de `ciclo/mod.rs`, pesados).
- A7-07 y A7-10: guarda ampliada (rojo hoy y con el mutante); unitario en `desinstalar` (`aplicar(Reescribir)` con el fichero cambiado, `FALLO_AL_APLICAR` en inglés exacto); unitario en `init` (`escribir_nuevo` sobre un existente e `Informe::mensaje` sin diagnóstico, en inglés).
- A7-09: 1 unitario UTF-16LE con BOM (rojo hoy).
- A7-11: 1 caso de integración en `tests/init.rs` (pesado), rojo con el mutante.
- Final: `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar`.

## Lo que necesita decision de una persona

1. Contrato de A7-09 para un `config.yml` ilegible: (a) `config error ruta=...` (1) con «no se puede leer {ruta}: {error}». Recomendado.
2. Literales `Err("...")` en prosa de otros grupos (`actualizar_cli.rs:742`, `verificacion/informe.rs:116`): lista cerrada `LITERALES_ERR` con su motivo en vez de arreglarlos aquí. Recomendado.
3. Literal único del fixture `descartada`: `"el de entonces"`. Recomendado.

**Respuestas de la persona (Carlos, consigna del 2026-11-23: modo automatico con la opcion recomendada):** se aplican las recomendaciones 1 a 3.

**Regla de una hora:** 210 min, sin trocear: Carlos descartó trocear las tareas de corrección 4D (consigna del 2026-11-23).

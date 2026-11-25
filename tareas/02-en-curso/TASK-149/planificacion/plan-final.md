# Plan — TASK-149

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 170 min (de la rama abierta al cierre con merge; senales: unos 17 ficheros de codigo, una capa (ordenes y operaciones de la rama de estado), 6 criterios mas las copias de `valor`)

Regla de una hora: pasa de 60 min, pero por la consigna de Carlos del 2026-11-23 (modo automatico, opcion recomendada) la tarea se hace sin trocear.

Todo lo comun va a `estado`, del que ya depende cada modulo afectado y que ya importa `contrato` (`estado/transaccion.rs:20`). Excepcion: `escribir` lee el `ROADMAP` y `roadmap` depende de `estado` (seria un ciclo). Ningun contrato publico cambia. Un commit por ID, en este orden:

1. **A8-06 (absorbe A6-01 y A2-10).** `impl Fallo { pub fn informe(self, comando: &str) -> Informe }` en `estado/transaccion.rs:47`, junto a `Fallo::rechazada`. Sustituye la conversion a mano en `roadmap/mod.rs:303-313`, `estimacion/mod.rs:245-254`, `siguiente.rs:78-84`, `tablero.rs:443-451`, `seleccion/orden.rs:597-607`, `ciclo/mod.rs:245-256`, `importar/mod.rs:405-418`, `revision/mod.rs:233-245` y `:247-254`, `toma/mod.rs:157-164`. Lo que cada orden anade va encima en el mismo orden de claves que hoy (`estimacion` mete `id` en la posicion 0 solo si es `Rechazada`; `importar` mete `simular` en la 0; `ciclo` envuelve con `con_rechazo`, `toma` con `rechazo_de_toma`, `revision` anade `asignado` con su `dueno()`). El `de_fallo` privado de `estado/transaccion.rs:264` no se toca (convierte el rechazo en `error`).
2. **A8-08.** `pub fn como_orden(i: Informe, nombre: &str, clave: &str, valor: &str) -> Informe` en `estado/transaccion.rs`, junto a `informe()` (`:172`). Forma general de `plan/mod.rs:525-535`. Sustituye `de_review` (`revision/mod.rs:201-210`), `de_start` (`toma/mod.rs:106-115`), `ciclo::de_la_orden` (`ciclo/mod.rs:425-434`) y `plan::de_la_orden`. Los dos `rechazada()` identicos (`revision/mod.rs:212-216`, `toma/mod.rs:117-121`) pasan a `Fallo::rechazada(m, d).informe(NOMBRE)`.
3. **A8-09.** `con_dueno(i, dueno)` una vez en el modulo nuevo `estado/dueno.rs` (se borran `ciclo/mod.rs:378-388` y `plan/mod.rs:509-519`). `escribir` pasa a `impl toma::Entorno<'_> { pub(crate) fn escribir(&self, op: &dyn Operacion) -> Informe }` en `toma/mod.rs:55`; se borran `ciclo/mod.rs:369-374` y `plan/mod.rs:469-474`, y lo usa la copia en linea de `toma/mod.rs:211-214`.
4. **A8-10.** `pub fn de_base_o_suelo(g, dir, orden: &str) -> Result<Config, Informe>` en `config/mod.rs`, junto a `de_base` (`:526`). `plan::config_de_base` (`plan/mod.rs:419-430`) y `ciclo::umbral_de_inactividad` (`ciclo/mod.rs:168-179`, se conserva como funcion porque la usan `tablero.rs:464` y los tests) la usan.
5. **A8-07 (absorbe A2-01 y A3-04).** `pub fn tarea_propia(estado, id, quien: &Asignado, orden, ajena: (&str, &'static Texto), dueno: &RefCell<Option<String>>) -> Result<(Tarea, Cuerpo), Fallo>` en `estado/dueno.rs` (la `fn tarea` de `ciclo/operacion.rs:77-91` hecha funcion libre). La usan `revision/guardar.rs:97-108`, `revision/operacion.rs:132-143`, `plan/guardar.rs:111-122`, `plan/operacion.rs:368-379` y `plan/aprobar.rs:522-533` (con `APROBAR_DE_OTRA_PERSONA`). `Lanzada::tarea` delega en ella.
6. **A8-19.** `#[derive(Debug, PartialEq)] pub struct AnotacionConDueno<'a> { orden, id, asignado: Asignado, resto: Option<&'a str> }` en `estado/dueno.rs`, con `escribir() -> String` y `leer(a) -> Option<Self>` (valida `es_id`, la cuenta y `Asignado::persona`). Escritores: los `anotacion()` de `revision/operacion.rs:116`, `revision/guardar.rs:80`, `plan/guardar.rs:94`, `plan/operacion.rs:350`, `plan/aprobar.rs:156`, `:361` (hito) y `:506`, `toma/operacion.rs:145`, `ciclo/operacion.rs:116` (y por encima `liberar.rs:234,311`). Lectores: `revision/operacion.rs:185-196`, `toma/operacion.rs:266-282`, `ciclo/operacion.rs:241-264`, `ciclo/liberar.rs:351-384`; cada uno interpreta solo su `resto`.
7. **Copias de `valor` (de TASK-140).** `pub(crate) fn valor_de<'a>(it: &mut impl Iterator<Item = &'a String>, uso: impl FnOnce(&str) -> Informe) -> Result<String, Informe>` en `contrato.rs`; `falta-valor` en un solo sitio. Sitios: `seleccion/orden.rs:355` y `:566-570`, `alta.rs:184-186` y `hallazgos/orden.rs:160`.
8. `cargo xtask verificar` y el `## Resultado` con los 6 IDs y las copias de `valor`, con su commit.

## Reutilizacion (Regla 4D)

Se usa: `estado::tarea_de`, `de_otra_persona`, `por_estado` (`estado/tareas.rs:53,76,104`); `Fallo::rechazada`; `Informe::{nuevo, con, valor, diagnostico}`; `config::de_base` / `Config::suelo`; `toma::Entorno`; la forma general de `plan::de_la_orden` y la `fn tarea` de `ciclo` como modelo. Tests: `estado::prueba::{estado, motivo, texto}`, `ciclo::prueba::{ana, md, fila, AHORA}`, los `anotacion_ida_y_vuelta` existentes (sin tocar sus aserciones), `tests/comun`.

Nuevo, con motivo: `estado/dueno.rs` (grupo «operacion con dueno»; `tareas.rs` lee `tarea.md` y no sabe de informes); `Fallo::informe` y `como_orden` (no hay equivalente publico; `de_fallo` hace otra cosa); `de_base_o_suelo` (`identidad.rs:417` devuelve `Ok(None)`, no vale); `valor_de` (`opciones()` de `hallazgos/orden.rs:145` no admite opciones sin valor ni posicionales).

Se borran: `de_review`, `de_start`, los 2 `de_la_orden`, los 2 `con_dueno`, los 2 `escribir`, los 2 `rechazada`, `roadmap::de_fallo` y 6 preambulos.

## Lo que el rol no cubrio

- Riesgos: si el lector comun con `splitn(5)` cambia lo que hoy acepta `toma` con `splitn(4)` (p. ej. `start|TASK-001|7|ana|x`); si `Config::suelo().dias_sin_actividad` y `DIAS_POR_DEFECTO` (`liberar.rs:47`) pueden divergir.
- Testing: cobertura de cada sitio sustituido no valorada (plan minimo abajo).
- Dominio: nadie consume por fuera las lineas `rechazada`/`error` de `tablero`/`siguiente`/`seleccion` (sin comprobar); `revision` sigue sin `arreglo=tc2 actualizar` para `formato-1` (se conserva).
- Criterio 6 solo se comprueba ejecutando.

## Riesgos aceptados y que los contiene

- Orden de claves al pasar a la funcion comun: inserciones explicitas en la posicion actual y tests de integracion sin tocar.
- Lector de anotaciones en casos raros: cada lector rechaza un `resto` que no espera (`toma` exige `resto == None`); las anotaciones malas existentes siguen en `None`.
- `umbral_de_inactividad` depende de `Config::suelo()`: test de `ciclo/mod.rs:580-615`.
- Toque de una linea en `hallazgos/orden.rs`: su suite cubre `falta-valor`.
- No se tocan: preambulo de `toma` (`toma/operacion.rs:160-190`), copia en linea de `revision/mod.rs:298-300`, `lib.rs:738-741`, el `dueno()` repetido en 7 tipos, los `retain` de `plan/mod.rs:272` y `estimacion/mod.rs:214`.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; sin D4, sin mutantes obligatorios):

- A8-07: 1 unitario en `estado/dueno.rs` (propia, ajena con login en el `RefCell`, heredada, inexistente; un segundo intento limpia el dueno).
- A8-19: 1 unitario de ida y vuelta (`assert_eq!(leer(&a.escribir()), Some(a))`, con y sin `resto`, `resto` con `|`, anotaciones malas). Los 4 existentes sin cambios.
- A8-06: 1 unitario de `Fallo::informe` (rechazada y error con diagnostico) y 1 que fija el orden `id,motivo` de `estimacion`.
- A8-08 y A8-09: 1 de `como_orden` y 1 de `con_dueno`.
- A8-10: 1 unitario con Git real (sin base da el suelo; config mala sale con el nombre de la orden), reutilizando el montaje de `ciclo/mod.rs:580`.
- Copias de `valor`: los casos `falta-valor` existentes, sin cambios.
- Criterio 6: integracion de `review`, `start`, `plan`, `approve`, `liberar` y `roadmap` sin tocar aserciones, y `cargo xtask verificar` en verde. Ningun test pesado.

## Lo que necesita decision de una persona

1. Donde vive lo comun: recomendado `estado` (`estado/dueno.rs` y `transaccion.rs`), `escribir` como metodo de `toma::Entorno`.
2. Parametro del dueno en `tarea_propia`: recomendado `&RefCell<Option<String>>`.
3. Donde va `valor_de`: recomendado `contrato.rs`.
4. Rechazo de `revision`: recomendado mantener su `asignado` sin `arreglo` para `formato-1` (criterio 6).
5. Marcar la auditoria: recomendado no; el cierre va en `## Resultado`.

Respuestas de la persona (Carlos, consigna del 2026-11-23: modo automatico con la opcion recomendada): 1) `estado`; 2) `&RefCell<Option<String>>`; 3) `contrato.rs`; 4) se mantiene; 5) no se marca.

# Plan — TASK-148e: soporte comun de los tests unitarios (A8-18, A8-20 unitarios) (parte e de TASK-148)

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 50 min

Señales: 2 ficheros nuevos y 12 migrados, todos dentro de `#[cfg(test)]`; una capa (tests unitarios de `crates/tc2/src`); 3 criterios; 1 test nuevo; trabajo mecánico (mover y borrar), sin cambios de producción.

Dos módulos de soporte, cada uno en el módulo del que depende lo que contiene (dependencias hacia abajo, `estado` → `git`):

1. `crates/tc2/src/git/prueba.rs`, declarado en `git/mod.rs` como `#[cfg(test)] pub(crate) mod prueba;` (precedente: `#[cfg(test)] mod guarda;` de `i18n/mod.rs:18`). Contiene `CONFIG`, `entorno(raiz)` y `sh(raiz, dir, args) -> R<String>`, movidos desde `ciclo/mod.rs:446-485`, y `aislar(raiz) -> R<()>`, que escribe `CONFIG` en `raiz/gitconfig` (hoy repetido en `ciclo/mod.rs:511` y `:576`; decisión 2).
2. `crates/tc2/src/estado/prueba.rs`, declarado en `estado/mod.rs` igual. Contiene `EnMemoria(pub(crate) BTreeMap<String, String>)` con su `impl Estado` (movido desde `roadmap/mod.rs:579-588`), `estado(&[(&str, String)])` (guarda cada tarea bajo `{id}/tarea.md`), `motivo(&Result<Vec<Cambio>, Fallo>)`, `EnMemoria::de_rutas(&[(&str, String)])` (para el estado por ruta de `plan/operacion.rs:544`) y `texto(&Result<..>)` (idéntico en `ciclo/operacion.rs:313` y `ciclo/liberar.rs:455`; decisión 1).

Orden (un commit por hallazgo):
1. A8-20: `git::prueba` con su unitario. `ciclo/mod.rs` borra sus copias e importa `crate::git::prueba::*`. `estado/montaje.rs`: su `sh(dir, args)` pasa a `git::prueba::sh(raiz, dir, args)`, sin los cuatro `-c` (339-348); `escenario` llama a `aislar(&raiz)`; `commit_en_bare` recibe `raiz`. `git/estado.rs`: `repo()` llama a `aislar(&d)` y hace `git init` con `sh`; el entorno a mano de `plan_blob_es_el_del_arbol_del_propio_commit` (676-679) pasa a `Git::nuevo().con_entorno(entorno(&d))` (ese test sigue escribiendo su `gitconfig` con la clave de firma).
2. A8-18: `estado::prueba`; migrar `ciclo/operacion`, `ciclo/liberar`, `toma/operacion`, `plan/operacion`, `plan/aprobar`, `plan/guardar`, `revision/operacion` (sus 7 llamadas a `estado(t)` pasan a `estado(&[("TASK-001", t)])`), `roadmap` y `siguiente` (borra su copia, 140-149). `roadmap::tests` sigue `pub(crate)` porque `tablero.rs:830` usa `estado_de_referencia` y `personas`.
3. `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar`.

**Suposición del plan padre comprobada:** `estado/montaje.rs` no depende de `init.defaultBranch=main` (`escenario` fija `refs/heads/main` y `HEAD` con `update-ref`/`symbolic-ref`, 397-400, antes de clonar). En `git/estado.rs` ningún test nombra una rama.

**API pública:** no cambia; todo es `#[cfg(test)] pub(crate)` en módulos existentes.

## Reutilizacion (Regla 4D)

Lo que ya existe y se usa (helpers, funciones, fixtures, tests), buscado en el repo antes de disenar; y lo que se crea nuevo, con el motivo de no reutilizar:

- `ciclo::tests::{CONFIG, entorno, sh}` se mueven a `git::prueba`; `roadmap::tests::EnMemoria` a `estado::prueba`; `estado` y `motivo` (mismo sha en ciclo/operacion, ciclo/liberar y toma/operacion) se mueven y se borran las copias.
- `Git::con_entorno` sustituye al entorno armado a mano en `git/estado.rs:676-679`.
- Nuevo: `aislar` (sin él, la escritura de `CONFIG` queda en 4 sitios); `EnMemoria::de_rutas` (para que no quede un `estado` fuera de `estado::prueba`).
- `CONFIG` existe también en `tests/comun/mod.rs:22`: duplicado aceptado por la frontera de crate (decisión 5 del plan padre).

## Lo que el rol no cubrio

- Modos de fallo (no lanzado): si el `gitconfig` que deja `aislar` dentro del repo de `git/estado.rs` molesta como fichero sin seguimiento; si `core.autocrlf` cambia algo al pasar de `-c` a `CONFIG`.
- Testing (no lanzado): traslados literales, sin valorar fuerza de cada comprobación.
- Fuera de los criterios: en `montaje.rs` quedan `Command::new("git")` directos (`mktree` 365, `rama_local` 423); los `Git::nuevo()` sin entorno de `montaje.rs:416/454` y `git/estado.rs:574/727` siguen leyendo la configuración del usuario (decisión 3). Los ayudantes de carpeta temporal se parecen pero ninguna ficha los nombra.

## Riesgos aceptados y que los contiene

- La rama inicial de `repo()` en `git/estado.rs` pasa a `develop`: ningún test nombra una rama; lo contiene la suite completa.
- `commit_en_bare` cambia de firma (privada del módulo de tests).
- Lo no aislado (decisión 3) sigue igual que hoy; queda nombrado.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- Criterio 1 (A8-18, 0): traslados idénticos cubiertos por sus suites; `rg "struct EnMemoria|fn estado\(|fn motivo\(" crates/tc2/src` solo en `estado/prueba.rs` (los `fn estado` de producción tienen otra firma).
- Criterio 2 (A8-20, 1, medio, lanza git): en `git::prueba`, `aislar(raiz)` y `sh(raiz, raiz, ["config","user.email"])` da `prueba@example.com`.
- Criterio 3 (pesados): `cargo test -p tc2 --no-fail-fast` (recuento antes + 1) y `cargo xtask verificar`, sin solapar.

## Lo que necesita decision de una persona

1. Mover también `texto` a `estado::prueba`. Recomendado: sí.
2. Añadir `aislar(raiz)` a `git::prueba`. Recomendado: sí.
3. Aislar más de lo que exige el criterio (`rama_local`, los `Git::nuevo()` de montaje y git/estado). Recomendado: no en esta parte; consta como pendiente en el `## Resultado`.

**Respuestas de la persona (Carlos, consigna del 2026-11-23: modo automatico con la opcion recomendada):** 1 sí, 2 sí, 3 no (pendiente anotado).

# Plan — TASK-140: Auditoria 4D: A2 hallazgos y revision (7 IMPORTANTE)

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 180 min

Señales: 4 ficheros (`hallazgos/mod.rs`, `hallazgos/ronda.rs`, `hallazgos/orden.rs`, `tests/hallazgos.rs`); una capa; 7 criterios; 4 mutantes en rojo; un test de caracterización del binario y un caso con Git real; `cargo xtask verificar` al final.

**Las 7 fichas siguen vivas (develop 21824c9):** A2-02 (`mod.rs:695-704` y `:828-837`), A2-03 (`mod.rs:956-977` y `:980-1013`), A2-04 (`mod.rs:1193-1203`, `:1866-1880`, `ronda.rs:325-336`, con columnas en distinto orden), A2-05 (7 copias de `let mut valor` en `orden.rs:121, 294, 440, 495, 668, 721, 766`; la 7.ª es `prompt` con un array de rutas), A2-11 (`mod.rs:1561-1564`; el test no mira `v.dictamen`), A2-12 (`orden.rs:653-657`; el test solo prueba `merge`, `directo` y `no-ancestro`), A2-13 (`mod.rs:982-993`; el test no pasa de 2 cierres).

**Orden** (los D4 primero, para aplicar cada mutante sobre su línea literal antes del refactor):
1. A2-13: en `cada_regla_del_cierre_rechaza_con_su_motivo`, caso con `TOPE_HALLAZGOS + 1` cierres de `I1`: último motivo `#100:fuera-de-tope` y `v.cierres.len() == 1`. Rojo con `j >= TOPE_HALLAZGOS * 10`.
2. A2-11: en `el_informe_de_la_tarea_cuenta_como_el_check`, `assert_eq!(v.dictamen, Dictamen::Verde)` (ligera) y `Dictamen::Tope` (tope). Rojo con cada mutante de la ficha.
3. A2-12: ampliar `el_diff_incremental_no_arrastra_lo_que_trae_la_base`: `develop` cambia `pr.rs`, `feat` mezcla con `merge -X ours` (commit C) y `desde_incremental(.., &b, &c) == (None, "conflicto")`. Rojo con `1 => (Some(arbol), "merge")`.
4. A2-02: `fn fallo_de(v: &Value) -> impl Fn(&'static str, String) -> Fallo` privada, usada por `validar_uno` y `validar_cierre`.
5. A2-03: `impl Fallo { fn rechazo(self, indice) -> Rechazo }` y `fn cortar(lista, primero, que) -> (&[Value], Option<Rechazo>)` privadas; el orden de `rechazados` queda igual.
6. A2-04: `fn celdas(h: &Aceptado) -> [String; 6]` y `fn fila(celdas) -> String` privadas en `mod.rs`; las tres funciones insertan sus columnas extra donde ya van; `ronda.rs` las importa con `use super::{celdas, fila}`; salida idéntica byte a byte.
7. A2-05: test de caracterización (tabla de las 7 subórdenes × opción desconocida, repetida, sin valor al final, obligatoria ausente) verde antes; después un analizador privado `fn opciones(a, unicas, repetibles) -> Result<Opciones, Informe>` con `una`/`todas`, que comprueba la repetición antes de consumir el valor; las validaciones de valor (`es_revisora`, `git::es_id`, `id_de_app`, `numero`) van justo después del análisis y antes de `sin-*`/`faltan-argumentos` (sigue `validar --revisora "no vale"` → `revisora-invalida`).
8. `cargo xtask verificar`; `## Resultado` con los 7 IDs, sus commits y el rojo de los 4 mutantes.

No cambian líneas de contrato con un solo error por entrada, ni el Markdown, ni firmas `pub`; todo lo nuevo es privado de `hallazgos`.

## Reutilizacion (Regla 4D)

Lo que ya existe y se usa (helpers, funciones, fixtures, tests), buscado en el repo antes de disenar; y lo que se crea nuevo, con el motivo de no reutilizar:

- `crate::dato::plano`, `es_id` de `hallazgos/mod.rs:615`, `motivo::FUERA_DE_TOPE`, `Fallo`/`Rechazo` sin campos nuevos, `uso()`, `id_de_app`, `es_revisora`, `crate::git::es_id`.
- Tests: `comun::Fixture`, `f.crudo`, `comun::escribir`, `comun::registro_permitido`, el helper `hallazgos(&[..])` y los helpers unitarios de `mod.rs`. A2-12 amplía el test existente en vez de copiar su arranque.
- Nuevo (privado): `fallo_de`, `Fallo::rechazo`, `cortar`, `celdas`, `fila`, `opciones`; no hay equivalente común en el crate (los demás analizadores son de una sola orden).
- Fuera de alcance: copias de `valor` en `seleccion/orden.rs:350`, `:561-565` y `alta.rs` (decisión 1; las recoge XO).

## Lo que el rol no cubrio

- Riesgos: qué entradas con dos errores cambian de motivo con A2-05; si `merge -X ours` da el mismo conflicto en todas las versiones de Git del CI; captura de lifetimes de `impl Fn` en edition 2024.
- Testing: cobertura del test de caracterización no valorada.
- Dominio: consumidores externos del texto «N cierres pasan del tope» o del orden de rechazos (workflow `revision-ia.yml`); se conservan tal cual.

## Riesgos aceptados y que los contiene

- A2-05 cambia el motivo cuando una entrada trae dos errores (p. ej. `--revisora mal --raro`: de `revisora-invalida` a `argumento-desconocido`); con un error la salida es idéntica y la fija el test de caracterización (decisión 2).
- Los mutantes de los D4 dejan de aplicarse literalmente tras los D1: el rojo se demuestra antes y `cortar` queda cubierto por dos tests.
- Markdown: salida byte a byte; si alguna de las tres funciones no tiene aserción de fila completa, se añade.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- A2-02, A2-03: 0 (cubiertos por los tests del cierre, del tope y el caso de A2-13).
- A2-04: 0 si los tests de Markdown miran una fila; si no, hasta 3 aserciones.
- A2-05: 1 test de caracterización por tabla (algo pesado: ~28 lanzamientos del binario).
- A2-11: 2 aserciones en el test existente.
- A2-12: 1 caso ampliado (pesado: Git real).
- A2-13: 1 caso con 101 cierres en el test existente.
- Cierre: `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar`, sin solapar.

## Lo que necesita decision de una persona

1. Analizador de A2-05 privado en `hallazgos/orden.rs` (lo de `seleccion` y `alta` queda para XO). Recomendado.
2. Validaciones de valor después del análisis (cambia el motivo solo con dos errores a la vez). Recomendado.
3. No marcar A2 como corregido en `docs/auditoria-4d.md` (como TASK-139); el cierre va en `## Resultado`. Recomendado.

**Respuestas de la persona (Carlos, consigna del 2026-11-23: modo automatico con la opcion recomendada):** se aplican las recomendaciones 1 a 3.

**Regla de una hora:** 180 min, sin trocear: Carlos descartó trocear las tareas de corrección 4D (consigna del 2026-11-23).

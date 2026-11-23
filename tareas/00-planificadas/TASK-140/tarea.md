---
id: TASK-140
titulo: "Auditoria 4D: A2 hallazgos y revision (7 IMPORTANTE)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: fix/task-140-auditoria-4d-a2-hallazgos-y-revision-7-i
asignado_a: null
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-23
dependencias: []
---
## Objetivo

Corregir los 7 casos IMPORTANTE del grupo A2 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 4 Duplicaciones en `crates/tc2/src/hallazgos/` y 3 Disfraces de test con mutante confirmado. A2-01 y A2-06 se fusionaron en las tareas de abstracción común (A8-07 y A8-05) y aquí no se piden. Tampoco los MENOR (A2-07 a A2-10). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion
- [ ] A2-02 y A2-03: la extracción del id válido y la closure del `Fallo` (`hallazgos/mod.rs:695-704` ~ `:828-837`) y el corte por `TOPE_HALLAZGOS` con su conversión `Fallo -> Rechazo` (`mod.rs:956-977` ~ `:980-1013`) quedan escritos una vez cada uno.
- [ ] A2-04: la fila Markdown de un hallazgo y sus topes de `plano` (64, 200, 400) están en una sola función, que usan `mod.rs:1193-1204`, `mod.rs:1866-1881` y `ronda.rs:325-337`.
- [ ] A2-05: los siete analizadores de opciones de `hallazgos/orden.rs` (`:117-139`, `:287-302`, `:435-446`, `:488-511`, `:664-678`, `:717-731`, `:759-798`) usan un analizador común. `uso` y `argumento-desconocido` siguen saliendo igual, y lo comprueba `cargo test -p tc2 --test hallazgos`.
- [ ] A2-11: hay un test que comprueba el dictamen de `veredicto_ronda` en modo `tope` y en modo `ligera` (`mod.rs:1561-1564`). Con los dos mutantes de la ficha aplicados, `cargo test -p tc2 --lib hallazgos` sale en rojo; hoy sale en verde.
- [ ] A2-12: hay un test que ejecuta la rama «mezcla con conflictos -> `None`» de `desde_incremental` (`orden.rs:653-657`). Con el mutante de la ficha aplicado, `cargo test -p tc2 --test hallazgos el_diff_incremental_no_arrastra_lo_que_trae_la_base` (o el test nuevo) sale en rojo.
- [ ] A2-13: `cada_regla_del_cierre_rechaza_con_su_motivo` (o un test nuevo) prueba el tope de cierres `fuera-de-tope` (`mod.rs:982-993`). Con el mutante de la ficha aplicado, `cargo test -p tc2 --lib hallazgos::tests::cada_regla_del_cierre_rechaza_con_su_motivo` sale en rojo.
- [ ] `cargo xtask verificar` en verde, y el `## Resultado` lista los 7 IDs con su commit.

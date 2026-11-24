---
id: TASK-148f
titulo: "dobles de Plataforma con Parcial y Doble (A8-05, absorbe A2-06) (parte f de TASK-148)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: false
rama: fix/task-148-auditoria-4d-soporte-de-tests-compartido
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-24
dependencias: [TASK-148e]
---
## Objetivo

Parte f del troceo de TASK-148 (Auditoria 4D: soporte de tests compartido en tests/comun y modulos prueba (9 IMPORTANTE)), partes TASK-148a, TASK-148b, TASK-148c, TASK-148d, TASK-148e, TASK-148f en la rama fix/task-148-auditoria-4d-soporte-de-tests-compartido.

Crea `plataforma::prueba::{Parcial, Doble}` y `comun/plataforma.rs` con `panic!` ante un método no esperado, y migra `identidad.rs`, `revision/mod.rs`, `tablero.rs`, `tests/revision.rs` y `tests/tablero.rs`.

## Criterios de aceptacion
- [ ] `git grep -n 'impl Plataforma for' -- crates/tc2` solo devuelve `github.rs`, `gitlab.rs` y los dos `Doble<P>`, y no queda ningún `sin_uso`.
- [ ] Dos tests `#[should_panic(expected = "merge-pr")]` sobre un `Doble` vacío, uno en `plataforma::prueba` y otro en `tests/soporte.rs`.
- [ ] `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar` en verde; el `## Resultado` lista A8-05 y A2-06 con su commit.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-24T08:18:58Z | plan | automatico | persona | 0.17.1 |

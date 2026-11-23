---
id: TASK-148c
titulo: "servidor HTTP de los tests con comun::http_mock (A8-11) (parte c de TASK-148)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: fix/task-148-auditoria-4d-soporte-de-tests-compartido
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
dependencias: [TASK-148b]
---
## Objetivo

Parte c del troceo de TASK-148 (Auditoria 4D: soporte de tests compartido en tests/comun y modulos prueba (9 IMPORTANTE)), partes TASK-148a, TASK-148b, TASK-148c, TASK-148d, TASK-148e, TASK-148f en la rama fix/task-148-auditoria-4d-soporte-de-tests-compartido.

`tests/identidad.rs` y `tests/init.rs` usan `comun::http_mock::Mock` y `serie()` en lugar de su `Servidor` propio.

## Criterios de aceptacion
- [ ] `git grep -n -e 'Servidor::nuevo' -e 'struct Servidor' -- crates/tc2/tests` no devuelve nada.
- [ ] `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar` en verde; el `## Resultado` lista A8-11 con su commit.

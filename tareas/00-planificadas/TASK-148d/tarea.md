---
id: TASK-148d
titulo: "instantanea de carpeta comun (A8-12, absorbe A7-06) (parte d de TASK-148)"
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
dependencias: [TASK-148c]
---
## Objetivo

Parte d del troceo de TASK-148 (Auditoria 4D: soporte de tests compartido en tests/comun y modulos prueba (9 IMPORTANTE)), partes TASK-148a, TASK-148b, TASK-148c, TASK-148d, TASK-148e, TASK-148f en la rama fix/task-148-auditoria-4d-soporte-de-tests-compartido.

Crea `comun/instantanea.rs` (recursiva, anota carpetas, `"<len> <sha256>"`, con lista de exclusión) y migra init, desinstalar, importar, tablero, frescura y doctor.

## Criterios de aceptacion
- [ ] Ninguno de `tests/init.rs`, `tests/desinstalar.rs`, `tests/importar.rs`, `tests/tablero.rs`, `tests/frescura.rs` y `tests/doctor.rs` recorre carpetas por su cuenta: usan `comun::instantanea`.
- [ ] Un test de `tests/soporte.rs` comprueba que una carpeta vacía nueva y un fichero en una subcarpeta cambian la foto, y que un nombre excluido no la cambia.
- [ ] `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar` en verde; el `## Resultado` lista A8-12 y A7-06 con su commit.

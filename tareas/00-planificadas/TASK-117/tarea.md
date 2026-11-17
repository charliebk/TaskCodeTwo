---
id: TASK-117
titulo: "Seguimiento de F2-14: menores aceptados de tc2 desinstalar"
tipo: fix
sprint: 0
etiquetas: [distribucion, tests]
complejidad: simple
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: fix/task-117-seguimiento-de-f2-14-menores-aceptados-d
asignado_a: null
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-17
actualizado: 2026-11-17
dependencias: []
---
## Objetivo

Cerrar los MENORES aceptados en TASK-078.

## Criterios de aceptacion
- [ ] un_tc2_ajeno_no_se_borra distingue un binario que se ejecuta y no es tc2 de uno que no arranca, y hay test de la rama positiva de es_tc2 (crates/tc2/tests/desinstalar.rs)
- [ ] es_tc2 no ejecuta binarios con --simular, o instalar.md lo documenta
- [ ] escribir_atomico preserva enlaces duros o lo documenta, y las ordenes de claude plugin uninstall valen en cmd.exe y en sh (test)
- [ ] Una config con formato: 2 y esquema roto no se borra con --config sin aviso (test)
- [ ] Rutas de la salida con un solo separador por sistema (test)

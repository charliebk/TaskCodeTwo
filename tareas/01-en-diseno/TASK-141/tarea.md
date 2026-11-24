---
id: TASK-141
titulo: "Auditoria 4D: A3 plan, importar y alta (5 IMPORTANTE)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: false
rama: fix/task-141-auditoria-4d-a3-plan-importar-y-alta-5-i
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
dependencias: []
---
## Objetivo

Corregir los 5 casos IMPORTANTE del grupo A3 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 2 Duplicaciones (una `divergente`), 1 Desviación que cambia el comportamiento y 2 Disfraces de test con mutante confirmado. A3-04 y A3-06 van en las tareas de abstracción común (A8-07 y soporte de tests). No se piden los MENOR (A3-07, A3-09). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion
- [ ] A3-03 (`divergente`, va primero): `leer_fichero` de `plan/mod.rs:139-170` e `importar/mod.rs:246-271` es una sola función con un solo tope (`plan/guardar.rs:33-34` ~ `importar/fichero.rs:32-33`). `import` sanea la ruta y el error antes del diagnóstico (F1-8) y rechaza el fichero vacío, como ya hace `plan`, y lo prueba un test de `import` con una ruta que lleva un salto de línea.
- [ ] A3-05: la secuencia de `approve` y la de `approve --hito` (`plan/mod.rs:283-295` ~ `:349-362`) comparten una sola función. `cargo test -p tc2 --test aprobar` sigue en verde.
- [ ] A3-08: `new` e `import` rechazan al dar de alta un enunciado que no está listo según la misma regla que `plan` (`alta.rs:75-108` frente a `plan/heuristica.rs:130-144`, RF-01). Lo prueba un test de `new` con un enunciado que hoy acepta y `plan` rechaza.
- [ ] A3-01: `anotacion_ida_y_vuelta` (`alta.rs:1051-1058`) compara la operación reconstruida campo a campo, y `desde_anotacion` (`alta.rs:567-571`) deja de perder lo que no es el título, o documenta y prueba por qué no lo necesita. Con el mutante de la ficha aplicado, `cargo test -p tc2 --lib alta::tests::anotacion_ida_y_vuelta` sale en rojo.
- [ ] A3-02: un test llama a `es_esqueleto` con rutas que eligen roles por dominio, la rama con unificador de `plan/aprobar.rs:117`. Con el mutante de la ficha aplicado, `cargo test -p tc2 --lib plan::aprobar` sale en rojo.
- [ ] `cargo xtask verificar` en verde, y el `## Resultado` lista los 5 IDs con su commit.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-24T19:39:12Z | plan | automatico | persona | 0.17.1 |

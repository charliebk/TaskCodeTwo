---
id: TASK-150
titulo: "Auditoria 4D: utilidades comunes de E-S y tiempo (4 IMPORTANTE)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
rama: fix/task-150-auditoria-4d-utilidades-comunes-de-e-s-y
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1470926
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-25
dependencias: []
---
## Objetivo

Extraer las utilidades de E/S y de tiempo que la auditoría 4D (`docs/auditoria-4d.md`, commit auditado d82ea9d) encontró copiadas entre dueños: 4 casos IMPORTANTE (tres `divergente`) de los grupos escritura-atomica, lectura-con-tope, espera-con-plazo y fechas-utc, más la ficha fusionada A7-02. Toca `huella`, `confianza`, `desinstalar`, `identidad`, `estado`, `hallazgos`, `seleccion`, `progreso`, `actualizar_cli`, `verificacion`, `alta`, `toma` y `xtask`. No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion
- [ ] A8-04 (`divergente`, absorbe A7-02; va primero): una sola escritura atómica, con el reintento del `rename` en Windows de `huella.rs:89-116`, la usan `confianza/mod.rs:160-178` y `desinstalar/mod.rs:571-588`. Fija el mismo nombre de temporal, conserva los permisos y borra el temporal si falla `write` o `rename`, y lo prueba un test de la propia función.
- [ ] A8-17 (`divergente`): un solo «leer con tope» (`take(tope + 1)` y rechazar si se pasa) sustituye a los de `identidad.rs:229-240`, `estado/frescura.rs:171-181`, `hallazgos/orden.rs:153-160`, `seleccion/orden.rs:280-287`, `progreso/fichero.rs:125-133`, `progreso/mod.rs:51-59` y el resto de la ficha. Se decide y se prueba qué hace con los bytes que no son UTF-8.
- [ ] A9-01 (`divergente`): «esperar a un hijo con plazo y matarlo» es una sola función (`actualizar_cli.rs:574-592` ~ `verificacion/ejecutar.rs:380-393`) que mata el árbol entero (`matar_arbol`), y la documentación de `actualizar_cli.rs:516-518` dice lo que hace.
- [ ] A8-03: el formateo `AAAA-MM-DDTHH:MM:SSZ` y el algoritmo civil están una sola vez en `tc2` (`alta.rs:673-698` ~ `toma/operacion.rs:250-261`), y el test `iso` de `tests/identidades.rs:366-384` usa la función del crate. `xtask/src/main.rs:430-448` (`fecha_utc`) se queda como está si xtask sigue sin depender de tc2, y se anota el motivo.
- [ ] `cargo xtask verificar` en verde, y el `## Resultado` lista los 4 IDs con su commit.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-25T18:33:25Z | plan | automatico | persona | 0.17.1 |
| 2026-11-25T18:41:47Z | approve | automatico | persona | 0.17.1 |
| 2026-11-25T18:41:49Z | start | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-25T18:41:46Z | diseno | agente:afaf9edd246673d34 | 52 | 89610 | 1355310 | 25954 | 51 | 1470926 | - | - |

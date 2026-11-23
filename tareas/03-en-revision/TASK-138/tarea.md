---
id: TASK-138
titulo: "Auditoria 4D de todo el codigo de TaskCodeTwo"
tipo: feature
sprint: 0
etiquetas: []
complejidad: media
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-138-auditoria-4d-de-todo-el-codigo-de-taskco
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2375381
tokens_implementacion: 287015785
tokens_revision: null
creado: 2026-11-22
actualizado: 2026-11-23
dependencias: []
---
## Objetivo

Auditar todo el codigo existente de TaskCodeTwo (crates/tc2/src, crates/tc2/tests, xtask, crates/distribucion y .github/workflows) contra la regla 4D de Carlos, recomendada por TaskCode 0.17 al actualizar: 1) Duplicacion: dos funciones o tipos que hacen lo mismo con pequenas diferencias. 2) Desviacion: codigo que se sale del objetivo de su tarea o de su ADR (o un criterio dado por cumplido sin estarlo). 3) Diluvio: lineas y tipos de mas donde bastan pocos. 4) Disfraz: codigo o tests que aparentan resolver algo y no lo comprueban. La auditoria solo diagnostica; las correcciones van en tareas de seguimiento.

## Criterios de aceptacion
- [x] docs/auditoria-4d.md lista cada caso con su regla (D1 a D4), fichero:linea, la evidencia que lo prueba (los dos fragmentos duplicados, el mutante que sobrevive, el criterio desviado) y su severidad CRITICO, IMPORTANTE o MENOR
- [x] Cubre todos los modulos de crates/tc2/src, los tests, xtask, crates/distribucion y los workflows; el documento lleva una tabla de cobertura por modulo con el numero de casos
- [x] Los casos de Disfraz en tests se prueban con un mutante que sobrevive, ejecutado de verdad

### Tras el cierre
- [ ] Cada caso IMPORTANTE o CRITICO tiene su tarea de seguimiento en tareas/00-planificadas, agrupadas por modulo (aplazado por decision de Carlos, 2026-11-22: se crea cuando el plugin TaskCode publique su version con arreglos; las 12 tareas estan preparadas en `seguimiento/`)

## Resultado

Auditoria 4D (Duplicacion, Desviacion, Diluvio, Disfraz) de todo el codigo de TaskCodeTwo sobre el commit d82ea9d, solo diagnostico. Informe: `docs/auditoria-4d.md`.

- **108 casos**: 82 IMPORTANTE (18 `divergente`), 0 CRITICO, 26 MENOR. Por regla: D1 54, D2 12, D3 17, D4 25.
- **Cobertura total**: los 283 ficheros de `git ls-tree d82ea9d` fuera de `tareas/` y `docs/` estan en la tabla de cobertura; lectura completa de todo el codigo, scripts, instaladores, workflows y configuracion; lectura mecanica declarada solo para `Cargo.lock` y 37 fixtures de datos grandes.
- **Metodo**: pase 0 determinista (ast-grep, clippy con lints de complejidad, scripts propios; sin descargas), 11 auditores con rutas exclusivas (A1-A7 por dominio, A8 transversal y duplicacion entre modulos, A9-A11 para cerrar los huecos detectados por el cruce de cobertura), mutacion dirigida y en serie en un clon propio, y consolidacion con fusion de duplicados y normalizacion de severidades.
- **Disfraz probado**: los 18 D4 de test con su mutante en verde y su control en rojo, ejecutados de verdad; los D4 de scripts (A10-01, A10-02, A11-01) con arneses que ejecutan las lineas del commit, guardados en `seguimiento/arneses/`. A10-05 (wget) queda como no verificado: no hay wget y el plan prohibe descargarlo.
- **Seguimiento**: 12 tareas preparadas en `seguimiento/` (9 por dueno y 3 por abstraccion comun: soporte de tests, capa de ordenes y utilidades de E/S y tiempo), sin crear por decision de Carlos (2026-11-22): las correcciones esperan a la nueva version del plugin TaskCode.

Decisiones (opcion recomendada, delegada por Carlos, 2026-11-22): sin descargas; mutacion dirigida y serializada; D1 divergente = IMPORTANTE con marca; criterio 3 aplazado.

Revision por pares:
- Ronda 1 (14de27c): code-quality-reviewer, aprobada-con-correcciones, sin CRITICO ni IMPORTANTE. Reprodujo por script 605 de 606 lineas de evidencia contra d82ea9d (la otra es una cita recortada de un ADR), 8 casos D1/D2/D3 a mano, 3 mutaciones (A11-04, A1-08, A3-01) y el arnes de A10-01, y cuadro la cobertura. Cinco MENOR corregidos en 1882024: textos de severidad de 8 fichas, salida mal rotulada de A10-01, arneses copiados al repo, cabecera de las tareas preparadas y nota de los 11 ficheros de configuracion. `verificar-menores` reconocio MEN-1, MEN-2 y MEN-5; MEN-3 y MEN-4 viven dentro de la carpeta de la tarea, donde el comando no mira (anotado en `menores-pendientes.md`).

Sin comprobar: A10-05 con wget real; los E2E con tokens reales quedan «no verificables por mutacion».

## Divergencias

- La auditoria tuvo 11 auditores en vez de 8: A9, A10 y A11 se anadieron al detectar ficheros leidos por zonas o fuera del reparto (exigencia de Carlos: ningun fichero sin leer).
- El criterio 3 (crear las tareas) se aplaza por decision de Carlos; las tareas quedan preparadas en `seguimiento/`.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-22T21:02:15Z | plan | automatico | persona | 0.17.0 |
| 2026-11-22T21:07:27Z | approve | automatico | persona | 0.17.0 |
| 2026-11-22T21:07:28Z | start | automatico | persona | 0.17.0 |
| 2026-11-23T00:11:13Z | review | automatico | persona | 0.17.0 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-22T21:07:27Z | diseno | agente:a957eb47a4d94c2cf,a5101993684efad53,ad6bbe593f787e65b | 50 | 215649 | 2127531 | 32151 | 2 | 2375381 | - | - |
| 2026-11-23T00:11:13Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 102 | 104678 | 27541115 | 40513 | 0 | 27686408 | - | - |
| 2026-11-23T00:11:13Z | implementacion | agente:a18cd64ab147999e9 | 30 | 75468 | 1390395 | 3874 | 1 | 1469767 | - | - |
| 2026-11-23T00:11:13Z | implementacion | agente:a252db4ebab6f785e | 152 | 277206 | 16724153 | 55861 | 26 | 17057372 | - | - |
| 2026-11-23T00:11:13Z | implementacion | agente:a30dda9761936fdc2 | 144 | 241591 | 14067429 | 54987 | 15 | 14364151 | - | - |
| 2026-11-23T00:11:13Z | implementacion | agente:a3f0f3372602ffae9 | 152 | 387554 | 21957387 | 57361 | 7 | 22402454 | - | - |
| 2026-11-23T00:11:13Z | implementacion | agente:a45b202e3a9ed7c55 | 132 | 360392 | 17420877 | 74167 | 34 | 17855568 | - | - |
| 2026-11-23T00:11:13Z | implementacion | agente:a5385ded908c3c7f9 | 110 | 253769 | 10637664 | 74853 | 3 | 10966396 | - | - |
| 2026-11-23T00:11:13Z | implementacion | agente:a66fce62a384beb00 | 216 | 413288 | 31391657 | 75133 | 24 | 31880294 | - | - |
| 2026-11-23T00:11:13Z | implementacion | agente:a6c4608b7a0d26af7 | 88 | 413488 | 12216550 | 42916 | 31 | 12673042 | - | - |
| 2026-11-23T00:11:13Z | implementacion | agente:a9a962b9657590196 | 114 | 333284 | 14629652 | 65912 | 44 | 15028962 | - | - |
| 2026-11-23T00:11:13Z | implementacion | agente:a9ca621ffa449c7c2 | 88 | 356887 | 10871041 | 50423 | 12 | 11278439 | - | - |
| 2026-11-23T00:11:13Z | implementacion | agente:a9dfde5f6065860db | 152 | 278404 | 16480677 | 61392 | 2 | 16820625 | - | - |
| 2026-11-23T00:11:13Z | implementacion | agente:ac374be0c6e9b4218 | 194 | 233782 | 12860078 | 49964 | 3 | 13144018 | - | - |
| 2026-11-23T00:11:13Z | implementacion | agente:ac53e6f11e6e46672 | 74 | 137133 | 4903737 | 50354 | 3 | 5091298 | - | - |
| 2026-11-23T00:11:13Z | implementacion | agente:ad898e7773b31091c | 152 | 341059 | 11029136 | 37794 | 14 | 11408141 | - | - |
| 2026-11-23T00:11:13Z | implementacion | agente:ad9affd01f2ab3b25 | 86 | 370243 | 11038877 | 50715 | 27 | 11459921 | - | - |
| 2026-11-23T00:11:13Z | implementacion | agente:ae16802ea1d1aba78 | 166 | 398106 | 25773898 | 72088 | 51 | 26244258 | - | - |
| 2026-11-23T00:11:13Z | implementacion | agente:ae2610917fc669d59 | 140 | 449679 | 19675028 | 59824 | 57 | 20184671 | - | - |

## Verificacion ligera

- 2026-11-23T01:35:34Z: ronda 1; commit revisado 14de27c9ff1a6be919beb9b56e9f3c2bdbc2f7af; commit verificado 1882024a158e7e965c34e537d5f3721ba2f9382b; ficheros: docs/auditoria-4d.md; corregidos: MEN-1, MEN-2, MEN-5; pendientes: MEN-3, MEN-4

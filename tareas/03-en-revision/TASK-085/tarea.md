---
id: TASK-085
titulo: "F4-6 plan propone skills, agentes y modelo por rol"
tipo: feature
sprint: 4
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-085-f4-6-plan-propone-skills-agentes-y-model
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 4322228
tokens_implementacion: 64605497
tokens_revision: 13836895
creado: 2026-11-14
actualizado: 2026-11-22
dependencias: [TASK-036, TASK-049, TASK-080]
---
## Objetivo

`plan` propone skills, agentes y modelo para diseño, implementación y revisión; se anotan en la tarea; el orquestador y la revisión los usan (RF-12, RF-13)

Tarea F4-6 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F4-1, F3-9, F6-3.

Contexto (Carlos, 2026-11-14, RR-17, imprescindible): para cada tarea y cada rol (diseño, implementación, revisión) hay que elegir las skills y los agentes adecuados y el modelo según complejidad y carril, solo entre lo que está instalado en la máquina y el proyecto, anotar la elección en la tarea y medir si acierta (hallazgos por revisora y coste por modelo). En la 0.16 existe con `catalogo-skills.yml`, `skills_recomendados` y `modelo_sugerido`, pero se usó en 21 / 0 / 2 tareas de ToolsaWeb / OpenDockTrading / TaskCode porque el catálogo apenas se configuró: hoy la elección la hace el agente sin criterio registrado. Nada debe decidirlo el modelo «a ojo»: reglas deterministas. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §1–4.

## Criterios de aceptacion
- [x] el modelo cambia según complejidad y carril
- [x] Cumple lo que pide la tarea F4-6 del plan y los requisitos que cita

### Tras el cierre
- [ ] En 10 tareas reales la propuesta coincide con la que haría una persona (dentro de muestra, comprobado por test; fuera de muestra, lo confirma Carlos en `evidencias/tabla-10-tareas-taskcodetwo.md`)

## Resultado

Seleccion determinista de skills, agentes y modelo por rol (F4-6, RF-12, RF-13, RR-17). No hay selector nuevo: `seleccion::revision` (F6-3) se generaliza a `seleccionar_con` por rol (diseno, implementacion, revision) con las filas S01-S07 y las columnas D/I/R de `tabla-modelo` del ADR F1-15; `seleccion/dominios.rs` lee `tabla-dominios` (S10, S11 y la excepcion de S13 activas).
- `plan` lee el inventario (F3-9) y el catalogo de la base, anota `roles.diseno` y `roles.implementacion` (agentes, reglas, senales congeladas, avisos, huellas de inventario y catalogo) y cada peticion dice que agente y que modelo lanzar; un agente no instalado no rompe `plan`: queda el aviso `no-instalado`.
- `approve` muestra lo que se firma (dominios, roles, modelos); `review` y `tc2 revisoras` suman los dominios (congelado + texto vivo + rutas); `doctor --recursos` recoge `roles.*.agentes`.
- Orden nueva `tc2 seleccion` (tambien `--json`); el mapa `roles` de `tarea.md` gana claves opcionales que solo se escriben con valor (una tarea antigua se reescribe byte a byte igual).
- Adenda F4-6 en F1-15 (las senales congeladas sustituyen al «texto aprobado» de la decision 3; la columna Implementacion sale «—» porque ninguna skill de ejecucion trae metadatos sin `recursos:`).

Decisiones (opcion recomendada, delegada por Carlos, 2026-11-22): las del plan; sin trocear.

Evidencia (2026-11-22):
- C1 dentro de muestra: `seleccion::tests::la_muestra_del_adr_se_reproduce` reproduce exactamente las 10 filas de `tabla-muestra` del ADR F1-15 que Carlos marco «Coincide las 10» (fixture extraida en solo lectura de ToolsaWeb, OpenDockTrading y TaskCode).
- C1 fuera de muestra: `tests/seleccion.rs::diez_tareas_de_taskcodetwo` genera con el binario la tabla de 10 tareas terminadas de TaskCodeTwo y la compara con la tabla dorada `tests/fixtures/seleccion-taskcodetwo/tabla.md`; la misma tabla, con la columna «Comprobado a mano (Carlos)» vacia, esta en `evidencias/tabla-10-tareas-taskcodetwo.md`. Pendiente de que Carlos la confirme (`### Tras el cierre`).
- C2: `seleccion::tests::modelo_por_rol_carril_y_complejidad` (opus/sonnet segun carril y complejidad por rol) y `tests/plan.rs::plan_anota_modelos_distintos`; en la tabla, sonnet/sonnet/sonnet en las simples y opus/sonnet/opus en las medias.
- C3: `tests/aprobar.rs::plan_approve_review_usan_la_seleccion` (plan, approve con firma, revision y doctor con remoto bare real), `tests/tarea.rs::roles_ampliados_ida_y_vuelta`, `tabla_adversaria` del detector. Mutaciones por test en rojo (ver revisiones).
- `cargo xtask verificar --perfil rapida` en verde sobre 7345f6a (1305 s) y sobre 2e2d33d tras las correcciones (la primera ejecucion de esa ronda cayo por el entorno: un instalador de Visual Studio quito `cl.exe` y dejo Git a medias durante la actualizacion; no fue del codigo).

Sin comprobar: que la propuesta fuera de muestra coincida con la de Carlos; el orquestador (F4-4) consumiendo `roles.*`; el revisor de CI con dominios en un PR real con modelo (sin saldo); la normalizacion de rutas con `\` fuera de Windows. La deteccion por palabra marca `puertas-seguridad` en tareas de TaskCodeTwo por «merge», «check» o «workflow» (lista de F1-15 tal cual).

Revision por pares (con la regla 4D de Carlos):
- Ronda 1 (0689c2c): cambios solicitados.
  - Rust: IMP-1 la huella del catalogo dependia de la ruta absoluta del `SKILL.md` (otra maquina reescribia la tarea); IMP-2 S06 y el orden de S07 sin tests, tres mutantes vivos (Disfraz); IMP-3 seis campos y su lectura/escritura duplicados en `Rol` y `RolRevision` (Duplicacion); MEN-1 helpers reimplementados y dos `dominios_de`; MEN-2 heuristica calculada dos veces.
  - Calidad: MEN-1 la tabla de 10 tareas no se comparaba con nada (Disfraz); MEN-2 regla de estratos no ejecutable (aceptado: queda escrita en `estratos.txt`; hacerla mecanica exige fixtures de todas las tareas); MEN-3 senales de dominio construidas en dos sitios y error de tabla tragado; MEN-4 no se avisaba de que Implementacion sale «—».
  - Corregidos en 9d43951, 1443600, 3e949ae (un solo `tarea::Rol`), 73818be y 2e2d33d, con test y mutacion.
- Ronda 2 (1fc787b, acotada): las dos revisoras, aprobada, sin hallazgos y la regla 4D cumplida en las cuatro D. Notas sin ronda: `revision::seleccion_de` sigue con `unwrap_or_default` sobre la tabla (inalcanzable con la tabla compilada) y la rama `TABLA_ILEGIBLE` no tiene test.

## Divergencias

- La revision con dominios y `doctor --recursos` entraron en el paso 4 (el plan los ponia en el 5) porque el test del criterio 3 los ejercita juntos; ese test vive en `tests/aprobar.rs` por la firma SSH.
- El rol de rendimiento entra con un solo rol (manda la tabla del ADR, F4-1 solo lo sumaba con dos).
- `plan` no tiene diff: los dominios de diseno salen del texto; la reproduccion de la muestra pasa las rutas a los tres roles como la PoC.
- En la muestra de TaskCodeTwo el estrato `alta` estaba vacio y lo sustituye `sustituto-H5` (TASK-046).

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-22T17:13:20Z | plan | automatico | persona | 0.16.1 |
| 2026-11-22T17:19:30Z | approve | automatico | persona | 0.16.1 |
| 2026-11-22T17:19:31Z | start | automatico | persona | 0.16.1 |
| 2026-11-22T18:43:47Z | review | automatico | persona | 0.16.1 |
| 2026-11-22T20:24:21Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-22T17:19:29Z | diseno | agente:a4af724a5e04d173f | 58 | 199960 | 4097817 | 24393 | 1 | 4322228 | - | - |
| 2026-11-22T18:43:47Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 34 | 23126 | 6411833 | 8631 | 0 | 6443624 | - | - |
| 2026-11-22T18:43:47Z | implementacion | agente:ab73915092e08af44 | 362 | 408808 | 57599241 | 153462 | 65 | 58161873 | - | - |
| 2026-11-22T19:03:54Z | revision | agente:a2e01d7a4a39a342a | 92 | 218701 | 7855057 | 27288 | 4 | 8101138 | 1 | rust-wasm-reviewer |
| 2026-11-22T19:03:54Z | revision | agente:ad3cafaa4f13575f2 | 44 | 154328 | 2672568 | 18913 | 2 | 2845853 | 1 | code-quality-reviewer |
| 2026-11-22T20:35:54Z | revision | agente:ae74baa6bed97ac4f | 32 | 129108 | 1747238 | 8411 | 3 | 1884789 | 2 | rust-wasm-reviewer |
| 2026-11-22T20:35:54Z | revision | agente:a7c136ad23e88224d | 18 | 120637 | 878068 | 6392 | 5 | 1005115 | 2 | code-quality-reviewer |

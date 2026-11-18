---
id: TASK-128
titulo: "Seguimiento de F3-11 y F3-12: aviso de solape contra uno mismo, textos al catalogo y test intermitente de progreso"
tipo: fix
sprint: 0
etiquetas: [solape, progreso]
complejidad: simple
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: fix/task-128-seguimiento-de-f3-11-y-f3-12-aviso-de-so
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1157361
tokens_implementacion: 20000464
tokens_revision: 4873148
creado: 2026-11-18
actualizado: 2026-11-18
dependencias: []
---
## Objetivo

Cerrar lo que quedo fuera de los ficheros citados en la revision de TASK-082 y el test intermitente que aparecio en ella.

## Criterios de aceptacion
- [x] start no avisa de solape entre dos tareas de la misma persona (la propia lleva su quien hasta solape::calcular en la toma nueva); el test de liberar c5_c6_c10 pasa a exigir stderr vacio (test)
- [x] AVISO_Y_MAS y NOTA_Y_MAS viven en i18n/catalogo/toma.rs y los recorre el test del catalogo
- [x] progreso::c3_transiciones_reales_reescriben_el_progreso pasa 10 de 10 bajo carga: la rotacion de .anterior.json no se ignora en silencio o el test no depende de ella

## Resultado

Hecho:
- C1: `solape::calcular` recibe la cuenta de quien toma la tarea y quita las tareas en curso o en revision que lleva la misma persona (`otras`, por id de cuenta con `Asignado::misma_persona`; `EnCurso` gana `asignado`). No salen en el aviso, ni en la `nota:`, ni en `en-curso-modulos`, ni en `solape-sin-rama`. `liberar::c5_c6_c10` exige stderr vacio; `solape::c10` pasa a k = 1 (TASK-002 es del propio luis); test E2E nuevo `misma_persona_no_se_avisa_contra_si_misma`.
- C2: `AVISO_Y_MAS` y `NOTA_Y_MAS` en `i18n/catalogo/toma.rs` (los recorre `catalogo_completo`).
- C3: la causa del `progreso::c3` intermitente no era la que suponia el plan (un `rename` bloqueado en Windows), sino que la rotacion de `.anterior.json` escribia el progreso leido y vuelto a serializar. `serde_json` sin `float_roundtrip` no devuelve igual un `f64` en un 13 % de los valores de una sonda (26 481 de 199 999), y `transcurrido` podia cambiar en la ultima cifra. Ahora se copian los bytes del fichero si son un progreso valido (test `el_anterior_es_el_fichero_que_habia_byte_a_byte`, rojo con la re-serializacion). Ademas, lo que decidio el plan (D1a, D2a): si el anterior no se escribe, la linea de la transicion lleva `progreso=sin-anterior` sin stderr. Diagnostico bajo carga (c3 diez veces con la suite de solape en paralelo): 9 de 10 con solo `sin-anterior`, 10 de 10 tras copiar los bytes.
- Adenda en el ADR F1-17 (D5a) con la regla de C1, `progreso=sin-anterior` y la divergencia de C3. D1a a D5a por la consigna de Carlos; sin trocear (150 min estimados) por su decision.
- `cargo xtask verificar --perfil rapida` en verde.

No verificado: la mutacion del filtro de C1 contra `liberar::c5_c6_c10` por E2E (al revisor le fallo el enlazado de `tc2.exe`); la cubren el unitario de `otras` y el E2E de solape.

Revision ronda 1: aprobada con correcciones por los dos, sin CRITICOS ni IMPORTANTES. MEN-1 de rust-wasm-reviewer (`toma/mod.rs:234`) y MEN-1 de code-quality-reviewer (`progreso/mod.rs:266`): `progreso=sin-anterior` sin test que lo fuerce en `start` ni en una transicion. Corregidos en 5bd3020 con el E2E `sin_anterior_en_start_y_en_una_transicion` (los dos mutantes lo ponen rojo); `verificar-menores` los deja en `menores-pendientes.md` porque el arreglo es un test y no cambia el fichero citado.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-18T17:54:47Z | plan | automatico | persona | 0.16.1 |
| 2026-11-18T18:00:59Z | approve | automatico | persona | 0.16.1 |
| 2026-11-18T18:01:00Z | start | automatico | persona | 0.16.1 |
| 2026-11-18T19:33:18Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-18T18:00:44Z | diseno | agente:acc352a6b97b6ff67 | 44 | 73228 | 1068011 | 16078 | 31 | 1157361 | - | - |
| 2026-11-18T19:33:18Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 114 | 75729 | 19890312 | 34309 | 1 | 20000464 | - | - |
| 2026-11-18T19:53:11Z | revision | agente:aad07a7702b52cb5d | 52 | 127517 | 2644008 | 9962 | 4 | 2781539 | 1 | rust-wasm-reviewer |
| 2026-11-18T19:53:11Z | revision | agente:ac476662f6d4e5db6 | 38 | 123611 | 1957052 | 10908 | 3 | 2091609 | 1 | code-quality-reviewer |

## Autorrevision

- rust-wasm-reviewer — 78f965e
- code-quality-reviewer — 78f965e

## Verificacion ligera

- 2026-11-18T19:57:26Z: ronda 1; commit revisado 6fdb49100d3066da1c76a255b171cb8ed232d052; commit verificado 5bd30202122bcc294ee819e46da6da2421c304c8; ficheros: crates/tc2/tests/progreso.rs; corregidos: ninguno; pendientes: MEN-1, MEN-1

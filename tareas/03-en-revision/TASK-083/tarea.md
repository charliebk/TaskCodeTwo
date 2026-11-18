---
id: TASK-083
titulo: "F3-13 Estimacion por fase de cada tarea al planificar"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-083-f3-13-estimacion-por-fase-de-cada-tarea
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1502907
tokens_implementacion: 57714511
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-18
dependencias: [TASK-029, TASK-075]
---
## Objetivo

Estimación por tarea al planificar: tiempo por fase a partir de tareas parecidas; se recalcula al terminar cada fase (RR-19)

Tarea F3-13 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-17, F3-2.

Contexto (Carlos, 2026-11-14, RR-19, imprescindible): mientras se trabaja lo importante no es un porcentaje sino qué fases faltan y el tiempo estimado de cada tarea, como información para el programador, visible en el terminal de trabajo (status line de Claude Code y `taskctl status --seguir`), escrito por el CLI sin gastar tokens; en un lote, además, tareas que faltan y hora estimada de fin. Al cerrar, métricas finales: estimado frente a real por fase, tokens, rondas y hallazgos, con la calibración de la estimación en el informe por versión. Base en la 0.16: `taskctl estado [--seguir]`, la status line (fase, paso, x/N, minutos) y la «Duración estimada» del plan. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 quater.

## Criterios de aceptacion
- [ ] C1: real por fase determinista: con `start` 10:00, commit 10:20, `pausa` 10:30, `reanudar` 11:30, commit 11:40, `review` 13:00, `review` 13:30 y `finish` 13:40 sale implementacion 85 (20 + 10 + 10 + 45; el 75 del plan era un error de suma que olvidaba los 10 min antes de la pausa), rondas 30 y cierre 10; una duracion negativa descarta la tarea con `duracion-negativa`
- [ ] C2: con 6 tareas falsas terminadas del mismo carril, complejidad y tamano (implementacion 10..60) la mediana y el p80 de `implementacion` coinciden exactamente con `poc-f1-17.py` sobre el mismo corpus, con `nivel=carril+complejidad+tamano` y `n=6`; con 4 en ese nivel y 5 del mismo carril baja a `carril`
- [ ] C3: con menos de 5 muestras en todos los niveles `estimacion` queda con `nivel: sin-datos`, `n`, `por_fase: null` y `total: null`, y la salida dice «sin tareas parecidas (n=k<5)»; nunca un numero inventado
- [ ] C4: tras `tc2 new` y `tc2 import` cada `tarea.md` creado tiene `estimacion` no nula (aunque sea `sin-datos`) y se relee sin error; tras `tc2 estimar` ninguna abierta queda con `estimacion: null`; las terminadas quedan byte a byte
- [ ] C5: tras terminar una muestra mas, `start` recalcula: `calculada` a la hora del `start` y `n` sube en 1, con el resto del `tarea.md` byte a byte; `pausa`, `reanudar`, `liberar` y `reasignar` no cambian `estimacion`
- [ ] C6: con `por_fase.implementacion: [30, 52]` guardado y real 75, `tc2 estimar --error` da error absoluto 45 min, relativo 0,60 y razon estimado/real 0,40 por fase y en total; sin estimacion guardada dice «sin estimacion guardada» sin cifras
- [ ] C7: la estimacion de una tarea nunca la usa como muestra; todo sale de funciones puras, del estado y de un `git log`, sin llamar a ningun modelo

## Autorrevision

- rust-wasm-reviewer — e9a6879
- code-quality-reviewer — e9a6879

Hallazgos propios (ninguno CRITICO ni IMPORTANTE):

- Correccion del criterio C1: el plan sumaba 20 + 0 + 10 + 45 = 75 y
  olvidaba los 10 min entre el commit de las 10:20 y la pausa de las 10:30.
  La regla de F1-17 y la PoC dan 85; se corrige el enunciado (no la regla)
  y queda anotado en la adenda de F1-17.
- MENOR: los tres primeros commits anaden ficheros que no se compilan hasta
  `bc21575` (donde entra `mod.rs`); sus tests solo se ejecutan desde ahi.
- MENOR: si no se puede leer el `git log`, la estimacion sigue sin commits
  en vez de fallar.
- Desviaciones del plan: la estimacion va como claves en la linea de `new`
  y `start` (F1-4 exige una sola linea); `tc2 estimar TASK-NNN` muestra sin
  escribir; `tc2 estimar` exige red; `agente` queda `null`.
- Tests anteriores ajustados: `tests/toma.rs` c7 y dos unitarios de
  `toma/operacion.rs` (solo por el bloque de estimacion que escribe
  `start`); el detector de `tests/comun` admite `log`.
- Mutaciones comprobadas (7, rojo y restauradas): tope de 45 min, descarte
  por duracion negativa, tarea como muestra de si misma, `estimar` sin
  filtrar terminadas, `start` sin escribir, tarea sin carril en niveles de
  carril, rango `M^1..M^2` ignorado.
- Verificacion propia: `--test estimacion --test toma` 22 de 22.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-18T00:08:14Z | plan | automatico | persona | 0.16.1 |
| 2026-11-18T00:17:17Z | approve | automatico | persona | 0.16.1 |
| 2026-11-18T00:17:19Z | start | automatico | persona | 0.16.1 |
| 2026-11-18T01:38:25Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-18T00:17:14Z | diseno | agente:abbdb11dbd8806e3a | 42 | 98077 | 1382374 | 22414 | 36 | 1502907 | - | - |
| 2026-11-18T01:38:24Z | implementacion | agente:a6c4a1524715407d2 | 286 | 511109 | 54013678 | 171803 | 75 | 54696876 | - | - |
| 2026-11-18T01:38:25Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 12 | 579547 | 2433823 | 4253 | 0 | 3017635 | - | - |

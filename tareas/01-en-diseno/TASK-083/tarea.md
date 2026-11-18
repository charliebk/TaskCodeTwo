---
id: TASK-083
titulo: "F3-13 Estimacion por fase de cada tarea al planificar"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: true
rama: feature/task-083-f3-13-estimacion-por-fase-de-cada-tarea
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1502907
tokens_implementacion: null
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
- [ ] C1: real por fase determinista: con `start` 10:00, commit 10:20, `pausa` 10:30, `reanudar` 11:30, commit 11:40, `review` 13:00, `review` 13:30 y `finish` 13:40 sale implementacion 75, rondas 30 y cierre 10; una duracion negativa descarta la tarea con `duracion-negativa`
- [ ] C2: con 6 tareas falsas terminadas del mismo carril, complejidad y tamano (implementacion 10..60) la mediana y el p80 de `implementacion` coinciden exactamente con `poc-f1-17.py` sobre el mismo corpus, con `nivel=carril+complejidad+tamano` y `n=6`; con 4 en ese nivel y 5 del mismo carril baja a `carril`
- [ ] C3: con menos de 5 muestras en todos los niveles `estimacion` queda con `nivel: sin-datos`, `n`, `por_fase: null` y `total: null`, y la salida dice «sin tareas parecidas (n=k<5)»; nunca un numero inventado
- [ ] C4: tras `tc2 new` y `tc2 import` cada `tarea.md` creado tiene `estimacion` no nula (aunque sea `sin-datos`) y se relee sin error; tras `tc2 estimar` ninguna abierta queda con `estimacion: null`; las terminadas quedan byte a byte
- [ ] C5: tras terminar una muestra mas, `start` recalcula: `calculada` a la hora del `start` y `n` sube en 1, con el resto del `tarea.md` byte a byte; `pausa`, `reanudar`, `liberar` y `reasignar` no cambian `estimacion`
- [ ] C6: con `por_fase.implementacion: [30, 52]` guardado y real 75, `tc2 estimar --error` da error absoluto 45 min, relativo 0,60 y razon estimado/real 0,40 por fase y en total; sin estimacion guardada dice «sin estimacion guardada» sin cifras
- [ ] C7: la estimacion de una tarea nunca la usa como muestra; todo sale de funciones puras, del estado y de un `git log`, sin llamar a ningun modelo

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-18T00:08:14Z | plan | automatico | persona | 0.16.1 |
| 2026-11-18T00:17:17Z | approve | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-18T00:17:14Z | diseno | agente:abbdb11dbd8806e3a | 42 | 98077 | 1382374 | 22414 | 36 | 1502907 | - | - |

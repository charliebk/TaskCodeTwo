---
id: TASK-075
titulo: "F1-17 ADR progreso en vivo y estimacion de tiempo"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-075-f1-17-adr-progreso-en-vivo-y-estimacion
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1196909
tokens_implementacion: 30003517
tokens_revision: 7778821
creado: 2026-11-14
actualizado: 2026-11-15
dependencias: [TASK-010, TASK-011]
---
## Objetivo

ADR: progreso en vivo y estimación (RR-19): estimación por fase (carril, complejidad, tamaño y tareas parecidas), cuándo se recalcula, dónde se muestra y cómo sin tokens

Tarea F1-17 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-9, F1-10.

Contexto (Carlos, 2026-11-14, RR-19, imprescindible): mientras se trabaja lo importante no es un porcentaje sino qué fases faltan y el tiempo estimado de cada tarea, como información para el programador, visible en el terminal de trabajo (status line de Claude Code y `taskctl status --seguir`), escrito por el CLI sin gastar tokens; en un lote, además, tareas que faltan y hora estimada de fin. Al cerrar, métricas finales: estimado frente a real por fase, tokens, rondas y hallazgos, con la calibración de la estimación en el informe por versión. Base en la 0.16: `taskctl estado [--seguir]`, la status line (fase, paso, x/N, minutos) y la «Duración estimada» del plan. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 quater.

## Criterios de aceptacion
- [ ] Estimación probada contra las tareas terminadas de ToolsaWeb y OpenDockTrading, con su error medio
- [ ] Cumple lo que pide la tarea F1-17 del plan y los requisitos que cita

## Autorrevision

- code-quality-reviewer — 18f6f7c

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-15T08:34:51Z | plan | automatico | persona | 0.16.1 |
| 2026-11-15T09:52:36Z | approve | automatico | persona | 0.16.1 |
| 2026-11-15T09:52:37Z | start | automatico | persona | 0.16.1 |
| 2026-11-15T10:42:24Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T11:05:45Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-15T09:52:50Z | diseno | agente:ab43b73ad5531941b | 30 | 83008 | 423394 | 9857 | 25 | 516289 | - | - |
| 2026-11-15T09:52:51Z | diseno | agente:a239447af7c4a96b2 | 38 | 65974 | 504714 | 10523 | 27 | 581249 | - | - |
| 2026-11-15T09:52:52Z | diseno | agente:a059144b1b74f552f | 6 | 42446 | 43735 | 13184 | 6 | 99371 | - | - |
| 2026-11-15T10:42:24Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 44 | 19267 | 3726998 | 9490 | 2 | 3755799 | - | - |
| 2026-11-15T10:42:24Z | implementacion | agente:a7e03519fedef64a8 | 190 | 446624 | 25635350 | 165554 | 21 | 26247718 | - | - |
| 2026-11-15T10:53:59Z | revision | agente:a402a78144d0fcb05 | 66 | 185156 | 5647744 | 42994 | 7 | 5875960 | 1 | - |
| 2026-11-15T11:10:43Z | revision | agente:a3943e371d3901a2e | 32 | 95304 | 1794896 | 12629 | 4 | 1902861 | 2 | - |

## Verificacion ligera

- 2026-11-15T11:10:46Z: ronda 2; commit revisado b516dee870028718a61296eec40038c470640855; commit verificado a5448948a20eda7e713baf436ccd693e1d88e0bf; ficheros: ninguno; corregidos: ninguno; pendientes: ninguno

## Resultado

- **Implementado**: ADR [F1-17](../../../docs/adr/F1-17-progreso-y-estimacion.md) (aceptada) y su fila en `docs/adr/README.md`. Lector unico del historico (real recalculado desde `## Transiciones` con huecos de mas de 45 min topados a 45 y pausas fuera); nucleo puro `estimar`/`restante` por fases medibles (implementacion, rondas, cierre; diseno sin calibrar) con cascada de tareas parecidas, mediana y p80, y fase «desbordada» con cota inferior; campo `estimacion` recalculado por el CLI en cada transicion; status line O(1) sobre `estado.json` derivado con ultimo valor bueno y texto ajeno saneado; `tc2 resumen` (dashboard de terminadas/pendientes, tiempo y tokens parcial y total, lo que falta con mediana y p80); `repartir --personas N` por camino critico con zonas (sin zona = choca con todas, siempre aviso); `start` que se niega con predecesora sin integrar (por ascendencia `merge-base --is-ancestor`) o sin red, y avisa de solape y de rama fuera de su zona; coste con un fichero por fuente; hora de fin de lote con p50/p80.
- **Medido (PoC minima, ~70 s, sobre los repos piloto solo leidos)**: 164 tareas ToolsaWeb y 48 OpenDockTrading sin descartes. Error relativo fuera de muestra: ToolsaWeb modelo 0,45 frente a 0,52 de la mediana ingenua pero 0,56 frente a 0,32 de la «Estimada» de la 0.16 (n=21); OpenDockTrading 0,43 frente a 0,41 ingenua y 0,37 frente a 1,68 de la 0.16 (que sobreestima 2,68x). **El modelo no gana a las dos lineas base en ningun repo**: se muestra como «rango de tareas parecidas» junto a la estimacion del agente y la calibracion se publica por version (F8-7). TASK-096 ODT 414 → 76 min. Prototipo de reparto reproducido exacto (16,6 h; 77,1/38,6/25,8/19,3 h). `fugas()` 0 fuera de muestra, 2515/734 con fuga a proposito. Rename en Windows falla con lector Python/Node abierto (reintento).
- **Decisiones de la persona**: D1a..D8a; D3a y D4a preguntadas expresamente a Carlos («Apruebo las dos»); sin troceo. Alcance de la PoC reducido por Carlos (2026-11-15): «Tenemos que reducir los PoC».
- **Divergencias** (en el ADR): F1-9 (forma de `estimacion`, campo `zonas`, `estado.json` sin versionar); F3-3 (`--personas`); F1-16 (mediana de ronda desde `estimar`); F1-14 (presupuesto O(1) de status line); F1-18 (p75 → p80); D6a concretado como tope; «tamano» = numero de criterios; quedan para F3-11, F9-4, F3-6, F3-12 y F4-9.
- **No verificado**: lo recortado por alcance (sensibilidad por tarea a otros umbrales, `start` con merge sin empujar o cache sin fetch, aviso de cambios ajenos, estres con procesos reales, mutaciones de `start`/solape/status line); status line real de Claude Code y de la 0.16.1; rename en Linux/macOS y con lector Rust; ascendencia con falso negativo tras squash/rebase o commits posteriores al merge; ramas antiguas en el remoto de los pilotos; estimacion de tokens.
- **Revision**: ronda 1 cambios (IMP-1 control de fuga que no detectaba fugas; IMP-2 `start` exigia un trailer que ningun merge real tiene; MEN-1..6), corregido en b516dee; ronda 2 aprobada sin hallazgos nuevos.
- Implementacion por un agente implementador; revision por dos agentes independientes. `taskctl doctor` con 0 errores. Pilotos solo leidos. Temporales borrados.

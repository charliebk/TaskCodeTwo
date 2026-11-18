---
id: TASK-084
titulo: "F3-12 Progreso en vivo: fases que faltan y tiempo restante"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: false
rama: feature/task-084-f3-12-progreso-en-vivo-fases-que-faltan
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-18
dependencias: [TASK-018, TASK-083]
---
## Objetivo

Progreso en vivo en el terminal: fase actual, **fases que faltan** y **tiempo estimado restante**, en la status line de Claude Code y en `taskctl status --seguir`, escrito por el CLI (RR-19)

Tarea F3-12 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F3-13, F2-3.

Contexto (Carlos, 2026-11-14, RR-19, imprescindible): mientras se trabaja lo importante no es un porcentaje sino qué fases faltan y el tiempo estimado de cada tarea, como información para el programador, visible en el terminal de trabajo (status line de Claude Code y `taskctl status --seguir`), escrito por el CLI sin gastar tokens; en un lote, además, tareas que faltan y hora estimada de fin. Al cerrar, métricas finales: estimado frente a real por fase, tokens, rondas y hallazgos, con la calibración de la estimación en el informe por versión. Base en la 0.16: `taskctl estado [--seguir]`, la status line (fase, paso, x/N, minutos) y la «Duración estimada» del plan. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 quater.

## Criterios de aceptacion
- [ ] C1: `tc2 estado --stdin` con `{"workspace":{"current_dir":<worktree>}}` no lanza procesos ni abre red (su firma no recibe `Git`); con 10 y con 1000 tareas en la rama de estado da la misma salida leyendo los mismos ficheros; tiempo medido en Windows anotado en el Resultado y el test falla por encima de 200 ms
- [ ] C2: una tarea en curso con muestras sinteticas conocidas y 20 min transcurridos da exactamente la linea esperada (fase, fases que faltan, ≈mediana, p80, n y nivel)
- [ ] C3: en un E2E con remoto local, tras `start`, `pausa` y `reanudar` el fichero se reescribe, la linea cambia y el anterior queda en `.anterior.json`; tras `descartar` o `liberar` la linea dice «sin tarea en curso»
- [ ] C4: `calcular` sobre una tarea `en-revision` da la fase rondas con solo «cierre» por delante
- [ ] C5: sin tareas parecidas la linea dice `sin estimacion (n=N)`
- [ ] C6: pasado lo de todas las parecidas la linea dice `≥X min (fase en curso por encima de sus parecidas)`
- [ ] C7: sin tarea en curso o fuera de un repo sale `tc2 · sin tarea en curso` con codigo 0 y stderr vacio
- [ ] C8: un fichero corrupto o a medias da el ultimo valor bueno con su antiguedad
- [ ] C9: `tc2 status --seguir` imprime la linea inicial, vuelve a imprimir al reescribirse el fichero y termina cuando `parar()` es verdadero (test de biblioteca con reloj inyectado); en el test de proceso, la salida contiene las dos lineas
- [ ] C10: avisos con caracteres de control o bidi salen saneados y con longitud acotada
- [ ] C11: ninguna orden nueva usa modelos ni red; `init` propone la status line y deja `settings.json` igual byte a byte

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-18T02:49:02Z | plan | automatico | persona | 0.16.1 |

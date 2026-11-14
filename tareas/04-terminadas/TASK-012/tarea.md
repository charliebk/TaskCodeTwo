---
id: TASK-012
titulo: "F1-11 ADR extensiones declaradas"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-012-f1-11-adr-extensiones-declaradas
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2006835
tokens_implementacion: 17876333
tokens_revision: 14689719
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-002, TASK-008]
---
## Objetivo

ADR: extensiones declaradas (P8)

Tarea F1-11 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-1, F1-7.

## Criterios de aceptacion
- [x] Contrato que cubre lo que hoy hacen ToolsaWeb y OpenDockTrading a mano
- [x] Cumple lo que pide la tarea F1-11 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T11:39:46Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T11:48:00Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T11:48:01Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T12:12:41Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T12:58:19Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T13:17:14Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T13:22:29Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T11:47:59Z | diseno | agente:a7a81b541b5f7788e | 76 | 94799 | 1889130 | 22830 | 72 | 2006835 | - | - |
| 2026-11-14T12:12:41Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 32 | 31131 | 7143929 | 15007 | 0 | 7190099 | - | - |
| 2026-11-14T12:12:41Z | implementacion | agente:a64e221658539d488 | 116 | 479790 | 10096105 | 110223 | 17 | 10686234 | - | - |
| 2026-11-14T12:27:28Z | revision | agente:a69450eddd3b1991b | 98 | 199118 | 8462467 | 58566 | 8 | 8720249 | 1 | - |
| 2026-11-14T13:05:25Z | revision | agente:a6be17e5ef0fa24af | 52 | 221490 | 4262871 | 44442 | 11 | 4528855 | 2 | - |
| 2026-11-14T13:21:51Z | revision | agente:a790aa171bab31ad6 | 22 | 153713 | 1268401 | 18479 | 9 | 1440615 | 3 | - |

## Autorrevision

- code-quality-reviewer — 8b01847
- code-quality-reviewer — 81aee93
- code-quality-reviewer — 8f63e07

## Rondas extra

- Ronda 3: Carlos (2026-11-14: «me interesa que los ADR de este plugin sean lo mas fiel posibles a los datos recolectados»); la correccion de la ronda 1 introdujo IMPORTANTE-5 (check estado ejecutaba el comprobador del PR), que se corrige siempre y se verifica en una ronda mas.

## Verificacion ligera

- 2026-11-14T13:21:56Z: ronda 3; commit revisado 72901f6338a1ace9a6f84d7bdbc379517af85c28; commit verificado 485052d4fa22e7267899865caace8b03d117c4da; ficheros: docs/adr/F1-11-extensiones-declaradas.md; corregidos: MEN-1; pendientes: ninguno

## Resultado

- **Implementado**: ADR [F1-11](../../../docs/adr/F1-11-extensiones-declaradas.md) (aceptada) y su fila en `docs/adr/README.md`. Inventario de las 14 practicas que ToolsaWeb y OpenDockTrading hacen hoy a mano, con 43 citas fichero:linea comprobadas por el guion y una columna «Clasificacion»: 7 parches de TaskCode que `actualizar` (F1-12/F9-2) retirara del proyecto y 7 propias del proyecto que se quedan. Contrato en tres bloques (`verificacion`, `derivados`, `confianza_incluye`; 5 de 12 claves con las de F1-10), todos los `comando` bajo la confianza de F1-8, derivados `genera`/`comprueba` ejecutados por `taskctl` en la transicion (comando explicito, ningun punto de extension depende de un hook), `escribe` normalizado y comparado por contenido, check `estado` obligatorio y no cacheado que toma config y comprobadores de la base, partes generadas de `hitos.md` en la rama de estado y `taskctl tareas --json` como lectura estable.
- **Ventaja medida** (separando medido de proyectado): lineas que se retiran frente a las que se quedan ToolsaWeb 3360 / 1093 (de su `verificar.bat`, 192 / 491) y OpenDockTrading 91 / 973; claves ToolsaWeb 10 → 2 bloques, pero valores a rellenar 10 → 16 en ToolsaWeb y 2 → 13 en OpenDockTrading (donde no gana); lectura del estado 114 → 11 lineas (proyectado); el `--comprobar` real de ToolsaWeb, que hoy sale con codigo 1 tras una transicion, da 0 con la disposicion nueva.
- **Decisiones de la persona**: D1a..D6a (consigna: opcion recomendada), no trocear, comandos deterministas como contrato (Carlos, 2026-11-14) y retirada de los parches de la 0.16 al migrar con lo propio del proyecto conservado (Carlos, 2026-11-14). Ronda extra 3 (ver `## Rondas extra`).
- **Divergencias** (en el ADR): con F1-5 (check `estado` obligatorio anadido), con RF-70 (sin job de CI ni hook) y RF-71 (sin estados configurables); ToolsaWeb necesita que `verificar.bat` acepte `--perfil` y una orden de huella local, lo que pone en duda el «sin scripts nuevos» de F5-1; el clasificador de ToolsaWeb solo es parche si F5-3 da el impacto generico (pregunta abierta).
- **No verificado**: `tareas --json` real, el check `estado` en GitHub, la concurrencia de derivados, enlaces simbolicos en `escribe` (sin privilegios en Windows) y un derivado que cree un enlace durante su ejecucion, la cache de ToolsaWeb con `RAMA_BASE` (solo con cache vacia), shell en Linux y macOS.
- **Revision**: ronda 1 cambios (IMP-1 las configuraciones traducidas no funcionaban con D2a, IMP-2 `escribe` se saltaba, IMP-3 «82 → 3» sin medir, IMP-4 ventaja medida a favor; MEN-1..5), todo corregido junto con la clasificacion parche/propio; ronda 2 cambios (IMPORTANTE-5 introducido: el check `estado` ejecutaba el comprobador del PR); ronda extra 3 aprobada con correcciones: MEN-1 (enlace creado por un derivado) anadido a «No verificado».
- Implementacion por un agente implementador; revision por tres agentes independientes. `taskctl doctor` con 0 errores. Pilotos solo leidos. Temporales borrados.

## Duracion

- Estimada: 170 min
- Real (de la rama abierta al cierre con merge, sin pausas): 94 min
- Paso de la hora: 94 min reales frente a 170 estimados
- Cierre (finish): 12 s, de 2026-11-14T13:22:17Z a 2026-11-14T13:22:29Z

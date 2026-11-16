# Puerta de valor de la Fase 10 (2026-11-16)

Evidencia del experimento que decidió si la Fase 10 (memoria del proyecto:
índice SQLite, `tc2 buscar`, grafo y visor) aportaba valor a Claude. Resultado:
**NO-GO** por el criterio fijado antes de medir. Carlos descartó la Fase 10 el
mismo día (TASK-113).

- [`informe.md`](informe.md): montaje, cifras, criterio, uso real del índice,
  factores que contaminan la medida y recomendación.
- [`corridas.tsv`](corridas.tsv): una fila por corrida (64).

Los dos ficheros son copia byte a byte de los del experimento (sha256
`b50f22cb…` el informe y `b98f1769…` las corridas); por eso no se editan y
llevan `-text` en `.gitattributes`.

## Cómo se midió

- **Preguntas**: 16 multicapa (necesitan juntar tarea, historial de git y
  código o ADR), 8 de TaskCodeTwo en `288f332` (T1..T8) y 8 de ToolsaWeb en
  `882c8bd7` (W1..W8), cada una con su clave de hechos y trampas.
- **Modos**: `grep` (Grep, Glob, Read y `git`) e `indice` (prototipo
  `buscar.py` de `tc2 buscar` con FTS5/BM25 sobre tareas, commits con ficheros
  y docs; más Read y `git`; Grep y Glob prohibidos).
- **Corridas**: 16 preguntas × 2 modos × 2 repeticiones = 64 subagentes sonnet,
  sobre copias fijas de los repos.
- **Métricas**: sacadas de la transcripción jsonl de cada subagente.
- **Juez**: un agente por pregunta, a ciegas (respuestas barajadas como A-D y
  sin nombres de herramienta), contra la clave.

## Columnas de `corridas.tsv`

| Columna | Qué es |
|---|---|
| `corrida` | Identificador `<pregunta>-<modo>-r<repetición>` |
| `repo` | `taskcodetwo` o `toolsaweb` |
| `pregunta` | T1..T8 o W1..W8 |
| `modo` | `grep` o `indice` |
| `rep` | Repetición, 1 o 2 |
| `agent_id` | Id del subagente en la sesión que lo lanzó |
| `tokens_total` | Suma, por llamada a la API, de entrada + caché creada + caché leída + salida |
| `tokens_ponderados` | Igual, ponderado por precio relativo: entrada ×1, caché creada ×1,25, caché leída ×0,1, salida ×5 |
| `tokens_salida` | Tokens de salida |
| `contexto_final` | Tamaño del contexto en la última llamada |
| `llamadas_api` | Llamadas a la API (mensajes distintos del asistente) |
| `segundos` | Del primer al último evento de la transcripción |
| `llamadas_herr` | Llamadas a herramientas |
| `bytes_herr` | Bytes devueltos por las herramientas al contexto |
| `usos_indice` | Llamadas a `buscar.py` |
| `intentos_grep` | Grep, Glob o grep/rg/find/findstr/Select-String por shell (en modo `indice`, intentos de volver a grep) |
| `lecturas` | Read o lecturas por shell (cat, head, sed -n…) |
| `git` | Comandos de shell que usan `git` |
| `denegadas` | Llamadas rechazadas por el sistema de permisos (sobre todo el clasificador del modo auto) |

La nota del juez no está en esta tabla: el agregado por repo y modo está en el
informe.

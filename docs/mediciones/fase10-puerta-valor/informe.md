# Puerta de valor de la Fase 10 — informe (2026-11-16)

## Montaje
- 16 preguntas multicapa con clave (8 TaskCodeTwo @288f332, 8 ToolsaWeb @882c8bd7), 2 modos × 2 repeticiones = 64 corridas sonnet.
- Modo grep: Grep/Glob/Read + git. Modo índice: `buscar.py` (prototipo de `tc2 buscar`: FTS5/BM25 sobre tareas troceadas, commits con ficheros, docs .md; ficha de tarea, búsqueda por fichero y etiqueta) + Read + git; Grep/Glob prohibidos.
- Juez a ciegas (16 agentes, respuestas barajadas A-D y saneadas) contra la clave: aciertos, errores, cobertura extra, nota 0-10.
- Métricas de la transcripción de cada subagente (`metricas.py`): tokens procesados (suma de entrada+caché+salida por llamada), tokens ponderados por precio relativo, segundos, llamadas, bytes devueltos por herramientas.

## Resultado (medianas)

| | TaskCodeTwo grep | TaskCodeTwo índice | ToolsaWeb grep | ToolsaWeb índice |
|---|---:|---:|---:|---:|
| tokens procesados | 427 k | 606 k (+42 %) | 563 k | 827 k (+47 %) |
| tokens ponderados | 115 k | 153 k (+33 %) | 146 k | 199 k (+36 %) |
| segundos | 103 | 119 (+16 %) | 151 | 179 (+19 %) |
| llamadas a herramientas | 8,5 | 12 | 10,5 | 14,5 |
| bytes de herramientas | 21 k | 31 k | 40 k | 66 k (+66 %) |
| acierto (hechos clave) | 1,00 | 1,00 | 0,83 | 1,00 |
| nota del juez | 9,0 | 9,0 | 8,0 | 9,0 |

Medias: acierto TaskCodeTwo 0,98 / 0,98, ToolsaWeb 0,89 / 0,94; nota TaskCodeTwo 9,1 / 8,8, ToolsaWeb 8,3 / 8,9; errores TaskCodeTwo 2 / 6, ToolsaWeb 4 / 3.

Por pregunta (media de 2 repeticiones), el índice gasta menos tokens en 2 de 16 (T8, W1) y menos tiempo en 2 de 16; en ToolsaWeb acierta más en 4 de 8 (W2, W3, W5, W7) y menos en ninguna.

## Criterio fijado antes de medir
«Si en ToolsaWeb el índice no baja la mediana de tokens o de tiempo sin perder acierto, la Fase 10 no se construye como está planteada.»
ToolsaWeb: tokens +47 %, tiempo +19 %, acierto no baja (sube). **No baja ni tokens ni tiempo → NO-GO.**

## Uso real del índice
- Todas las corridas en modo índice usaron `buscar.py` (de 1 a 6 veces; mediana 3).
- 26 de 32 (13 por repo) intentaron aun así volver a grep (Grep/Glob o grep por shell) pese a la prohibición.

## Por qué cuesta más
- El índice no sustituye la lectura: devuelve fragmentos y rutas y el agente lee igual los ficheros (más lecturas, +66 % de bytes en ToolsaWeb).
- Cada llamada extra cuesta el contexto entero (~47 k tokens fijos de arranque por subagente y creciendo), así que +4 llamadas pesan más que lo que ahorra encontrar antes.
- Lo que sí aporta: en el repo grande da mejor respuesta (más hechos de historia: tarea, commit, hallazgo de revisión).

## Factores que contaminan el resultado (no ocultos)
1. Denegaciones del clasificador de auto mode («Undermining Oversight») en `git show/log`: 56 en modo índice frente a 10 en grep. Infla el coste del índice. Estimación gruesa: unas 1,5 llamadas por corrida ≈ 80 k tokens; descontadas, el índice sigue más caro en tokens (≈ +13 % en ToolsaWeb) y en tiempo queda cerca de empate. No cambia el veredicto del criterio en tokens; en tiempo, sí lo deja dudoso.
2. Prototipo, no `tc2 buscar`: sin la regla 2 de atribución de F1-19, merges sin ficheros, salida no optimizada para tokens.
3. Preguntas redactadas por un agente que buscó con grep: pueden favorecer al grep.
4. El juez es un solo agente por pregunta; las diferencias de acierto son pequeñas (n = 16 respuestas por modo y repo).
5. No se midió el escenario real: índice **y** grep disponibles a la vez.

## Recomendación
**NO-GO a la Fase 10 tal como está planteada** (índice como motor de búsqueda que sustituye a Grep/Read, y todo lo que cuelga de él: grafo y visor F10-11..F10-17).
Si se quiere rescatar algo, que sea una propuesta nueva y acotada, con su propia puerta medida:
- `tc2 buscar` como **brújula de historia** (tarea ↔ commits ↔ ficheros ↔ hallazgos), con salida compacta, **junto a** Grep, no en su lugar;
- medirlo en el escenario mixto y sin el ruido de denegaciones, con el mismo criterio (bajar tokens o tiempo en ToolsaWeb sin perder acierto).

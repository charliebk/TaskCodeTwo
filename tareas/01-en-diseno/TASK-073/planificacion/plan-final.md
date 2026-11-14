# Plan — TASK-073: F1-15 ADR seleccion de skills, agentes y modelo

(Respuesta del unico rol de brainstorm, arquitectura. Con un solo rol no hay unificador: el plan dice que lados no reviso ese rol. Lo afirmado sobre la 0.16.1, los ADR y los informes se ha leido en los ficheros citados; lo no leido va a «Suposiciones no verificadas».)

## Enfoque propuesto

Duracion estimada: 150 min

Mas ancho que la mediana de un ADR de la Fase 1 (45,5 a 50 min, F1-18) y mas estrecho que F1-13 (180) o F1-14 (210): un ADR con tres tablas (reglas, modelo, dominios), una PoC que reutiliza lo hecho en F1-10 y dos lecturas medidas (los 15 informes de revision de la Fase 1 y 10 tareas reales de ToolsaWeb y OpenDockTrading). **No se trocea.**

### Decisiones ya tomadas por la persona (Carlos, 2026-11-14)

1. **RR-17 imprescindible**: eleccion por tarea y por rol (diseno, implementacion, revision) solo entre lo instalado en la maquina y el proyecto; reglas deterministas, nada «a ojo»; anotada en la tarea y con medida del acierto (hallazgos por revisora, coste por modelo).
2. **Manifiesto de recursos (F1-14)**: esta tarea define que skills y agentes son «necesarios para el proyecto» y como se eligen; no duplica `evaluar_recursos` ni `tc2 doctor --recursos`. engram no es obligatorio.
3. **Dato medido**: los planes de la Fase 1 se hicieron con un solo rol; en tareas de puertas y seguridad los revisores encontraron IMPORTANTES que ningun rol de riesgos miro. La regla de numero de roles debe considerar el dominio, no solo la complejidad.
4. **Nombres y forma**: `tc2`, `taskcodetwo`; CLI en Rust (F1-14); comandos deterministas sobre hooks; seguridad y trabajo en grupo imprescindibles.
5. **Sin trocear y PoC medida**: temporales en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-073\` (borrados al terminar); ToolsaWeb y OpenDockTrading leidos de copias `git clone --no-hardlinks`; GitHub solo lectura; sin instalar ni descargar toolchains.

### Como lo hace hoy la 0.16.1

- `scripts/catalogo-skills.yml` aplanado (`skill_N_*`), 6 entradas del plugin, ninguna externa.
- `seleccionarSkill` cruza etiquetas, ordena por solape y prioridad y, si empatan, **llama a un modelo** (`plan-desempate-skill.ts`): contradice RR-17.
- `patrones_archivo` duplicado en el catalogo y en cada `SKILL.md`; `revisores.ts` (lineas 17-21) cuenta que divergieron dos veces.
- `clasificarPorDominio` reparte revisoras solo por rutas del diff (globs; si nada casa o casan demasiados, generico); con renombrados solo la ruta final, mientras F1-10 pide las dos.
- `modelo_sugerido` = `sonnet` por defecto, nada lo calcula; en verificacion baja un escalon.
- Numero de roles: `heuristica-complejidad.yml` suma etiquetas, palabras de riesgo (×2), dependencias, tipo y criterios; tabla fija 0–4 roles en orden arquitectura, riesgos, testing, dominio; rendimiento condicional.
- Instalados y nunca elegidos para los ADR de seguridad: la skill `security-review` y el agente `revisor-seguridad-deploy` (los ADR solo tocan `docs/`).

### Dato medido: roles del plan frente a IMPORTANTES (lectura preliminar; la PoC la fija con lista cerrada y cita por hallazgo)

| Tarea | ADR | Roles | Puntos | IMP total | De puerta o seguridad |
|---|---|---|---|---|---|
| TASK-002, TASK-003 | F1-1, F1-2 | 1 | 1 | 0 | 0 |
| TASK-004 | F1-3 | 1 | 1 | 1 | 0 |
| TASK-005 | F1-4 | 1 | 1 | 3 | 0 |
| TASK-006 | F1-5 | 1 | 1 | 5 | 3 |
| TASK-007 | F1-6 | 1 | 1 | 3 | 2 |
| TASK-008 | F1-7 | 1 | 1 | 2 | 1 |
| TASK-009 | F1-8 | 2 (arquitectura y riesgos) | 4 | 4 | 4 |
| TASK-010 | F1-9 | 1 | 1 | 0 | 0 |
| TASK-011 | F1-10 | 1 | 1 | 2 | 2 |
| TASK-012 | F1-11 | 1 | 2 | 5 | 1 |
| TASK-013 | F1-12 | 1 | 2 | 2 | 0 |
| TASK-014 | F1-14 | 1 | 1 | 1 | 0 |
| TASK-015 | F1-13 | 1 | 1 | 2 | 1 |
| TASK-094 | F1-18 | 1 | 2 | 3 | 1 |

Lectura: 15 de 33 IMPORTANTES son de puerta o seguridad, 11 de ellos en planes de un rol; la heuristica mide dependencias, no dominio (14 de 15 «simple»); el reparto por rutas no ve el dominio de un ADR (revisor generico en los 6 informes leidos). Correccion honesta: F1-14 no encaja (su IMP es de reproducibilidad); F1-10 y F1-11 si; F1-8 tuvo riesgos y aun asi salio con 4 IMPORTANTES de seguridad: el suelo por dominio sube la probabilidad de verlos, no la garantiza; por eso tambien actua en la revision (D6).

### Donde encaja: seis piezas, tres se extienden y tres se crean

1. **Funcion pura `seleccionar`** (se crea), patron de `politica` de F1-10, tabla unica entre marcadores; la usan `plan` (F4-6), `review` y el revisor en CI (F6-3), el orquestador (F4-4), `lote` (F1-16) y `doctor --recursos`:
   ```text
   seleccionar(rol: Diseno|Implementacion|Revision, t: TareaLeida, pol: Politica, cx: Complejidad,
               cambios: [Cambio], inv: Inventario, cat: Catalogo, cfg: ConfigValidada) -> Seleccion
   Seleccion { agentes: [id], skills: [id], modelo: Modelo, reglas: [IdRegla], avisos: [Aviso] }
   dominios(t, cambios, cfg) -> [{dominio, senal}]   # lista cerrada en el binario; la config solo anade
   ```
   Nunca llama a un modelo; empate por `id` lexicografico con aviso (divergencia con la 0.16).
2. **Inventario de F3-9** (se extiende), unica fuente de lo que existe: plugin, `~/.claude/skills`, `~/.claude/agents`, otros plugins, `.claude/skills` y `.claude/agents` del proyecto. Lo que no esta no se elige; un catalogo que nombra algo inexistente es error en `doctor`.
3. **Metadatos en la propia skill** (se extiende el bloque yaml de F1-11 D5a) con `etiquetas`, `dominios`, `prioridad` y `rol ∈ {diseno, ejecucion, revisor}`; las ajenas en `recursos:` de F1-14; desaparece `catalogo-skills.yml` y su doble copia. YAML estandar (F1-9 D1a).
4. **Detector de dominio y suelo de roles** (se crea): roles de diseno = max(`roles_min` del carril de F1-10, tabla de complejidad, suelo por dominio). Si `dominios()` da `puertas-seguridad`, suelo 2 roles (arquitectura, riesgos) mas unificador; en riesgo alto, todos (F1-10). Senales en el binario (etiquetas; palabras de objetivo y criterios: puerta, check, ruleset, permiso, identidad, firma, token, secreto, aprobacion, carril, merge, workflow, runner, amenaza, seguridad; rutas), la config solo anade. Cierra lo que F1-10 dejo abierto (lineas 368-369 y 660-661).
5. **Revisoras** (se extiende `clasificarPorDominio`): los dos nombres de cada renombrado; `dominios()` anade la revisora del dominio si esta instalada y declarada, si no generica y aviso «dominio sin revisora» medido; la seleccion de revision se calcula con config y skills de **la base**, nunca del worktree (decision 3 de F1-10).
6. **Tabla de modelo y anotacion** (se crea la tabla, se extiende `roles` de F1-9): fila por (carril, complejidad, rol); `modelos_permitidos` en la config solo restringe; verificacion un escalon menos. Se anota `roles.<rol>.{modelo, skills, agentes, regla}` con id de fila, senales y hash de inventario y catalogo. Diseno e implementacion se calculan en `plan` y se congelan con `approve` firmado; revision en cada ronda con el diff real. Divergencia con F1-9 (`roles.revision.agente` singular → lista).

**Necesario para el proyecto (F1-14)**: lo que `seleccionar` elige para la tarea y el proyecto declara; `start` para si falta; lo del plugin es «obligatorio»; engram «conveniente».

**Medicion (RF-15, F8-6)**: columna **«Clase»** en la tabla de hallazgos con lista cerrada (puerta-seguridad, fidelidad-evidencia, contrato, correccion, reproducibilidad); hallazgos por revisora y clase, «escapados» (IMPORTANTES de una clase cuyo rol no se lanzo), tokens y resultado por modelo y rol.

**Se crea**: `docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md` con `tabla-seleccion`, `tabla-modelo` y `tabla-dominios` entre marcadores, su fila en el indice y la PoC (`poc-f1-15.py`, tabla de clasificacion de hallazgos con cita, log). **No se toca**: ADR F1-1..F1-18, el plan, pilotos ni la 0.16; las correcciones (F1-9 `roles`, F1-11 bloque yaml, F1-10 roles, F5-4 columna «Clase») van a «Divergencias».

### Coste que fija la forma

Ninguna llamada a un modelo para elegir; recalculo barato (globs en memoria, inventario una vez por comando); el suelo por dominio sube de 1 a 2 roles (mas unificador) en como mucho 8 de 15 tareas con los datos; el ADR da la cifra y el coste medio de un rol desde F1-18 (13,9 a 19,1 M tokens por ADR).

### Orden de construccion

1. Medida retrospectiva: clasificar los IMPORTANTES de los 15 informes con lista cerrada y cita; roles de cada plan.
2. Detector y suelo: `tabla-dominios` aplicada a las 15 tareas con recall y precision frente a «tiene al menos un IMPORTANTE de puerta o seguridad», antes de escribir la decision.
3. Inventario real de esta maquina (solo lectura) y de las copias de ToolsaWeb y OpenDockTrading (commits de F1-9/F1-10: `93c9618ac` y `fab8d7e05`, o los vigentes, anotados).
4. Las 10 tareas reales con muestreo estratificado fijado antes de mirar resultados (D9): carril efectivo (F1-10), complejidad (0.16), propuesta por rol, lo que uso la 0.16, y columna «comprobado a mano» que marca Carlos (D10).
5. Escritura: decision, divergencias, tabla requisito → evidencia, indice y borrado de temporales.

### Motivo para no trocear

Consigna de Carlos; roles, revisoras y modelo salen de la misma funcion y senales.

## Lo que el rol no cubrio

- Criterio 1: «probadas a mano» es juicio de una persona (D10); las 10 tareas concretas no estan elegidas.
- Criterio 2: determinismo cubierto; no revisado si algun consumidor (orquestador, F6-3) puede saltarse la seleccion.
- Criterio 3: RR-17, RF-12..RF-15, parte de RR-05 y lo cedido por F1-9, F1-10, F1-11 y F1-14; no se disenan `recursos:` (F2-5), el informe JSON (F5-4) ni una revisora de puertas.
- Riesgos sin enumerar: skill del proyecto que reclama todos los patrones; inventario distinto entre maquina y runner; modelo retirado; dos personas con inventarios distintos; etiquetas puestas para forzar o evitar el suelo.
- Dominio: si «puertas y seguridad» es el unico dominio con suelo (datos, migraciones, fidelidad); la lista de clases es propuesta.
- Testing: cobertura del plan de pruebas no discutida.

## Riesgos aceptados y que los contiene

- Clasificacion manual de hallazgos: cita por hallazgo contrastable por la revision.
- Muestra pequena (15 tareas, un proyecto, todas ADR): declarada; F8-6 recalibra; recall y precision declarados.
- Sin contrafactual (F1-8 con riesgos tuvo 4 IMPORTANTES): suelo en diseno y revisora o aviso en revision (D6), medidos como «escapados».
- Modelos dependen de la cuenta: `modelos_permitidos` solo restringe; valor por defecto del binario dicho en `doctor`.
- Sin revisora de puertas instalada: generica con aviso anotado y medido, sin inventarla.

## Plan de pruebas

`poc-f1-15.py` (`python -I -B`, log determinista, precedente F1-10..F1-14) lee `tabla-seleccion`, `tabla-modelo` y `tabla-dominios` del ADR y sale con 1 si una fila no tiene id, un id se repite, un modelo esta fuera de la escala o una regla da algo que no esta en el inventario; mutaciones de control; tiempo de reloj a stderr.

- **Criterio 1 → 3 pruebas**: (a) las 10 tareas con columnas «0.16» y «a mano» completas y la muestra fijada en el ADR antes de los resultados; (b) retrospectiva de la Fase 1 con recuentos del log y recall y precision del detector; (c) el modelo cambia con carril y complejidad en al menos dos filas reales (criterio de F4-6).
- **Criterio 2 → 3 pruebas**: (a) dos ejecuciones con el mismo hash; inventario y catalogo permutados dan salida identica; (b) quitar una skill del inventario la saca de la seleccion con aviso; empate por `id` lexicografico; (c) el guion no tiene camino a la red ni a un modelo (grep y copia del ADR mutada con empate total resuelto sin peticion).
- **Criterio 3 → 1 prueba**: tabla requisito → decision → evidencia (RR-17, RF-12..RF-15, RR-05, F1-9 `roles`, F1-10, F1-11 D5a, F1-14 «necesario») con IDs comprobados y fila del indice con enlace que resuelve; al cerrar, `taskctl doctor` sin errores y temporales borrados.

## Lo que necesita decision de una persona

Respuestas (Carlos, consigna «opcion recomendada»): **D1a, D2a, D3a, D4a, D5a, D6a, D7a, D8a, D9a, D10a**.

- **D1. Numero de roles de diseno**: **(a, recomendada)** max(`roles_min` del carril, tabla de complejidad, suelo por dominio = 2 si salta `puertas-seguridad`); (b) sumar puntos de dominio a la heuristica (descartada: «seguridad» ya suma 2 y F1-18 y F1-13 salieron «simple»); (c) 2 roles en toda funcionalidad.
- **D2. Detector de dominio**: **(a, recomendada)** lista cerrada en el binario que la config solo amplia; (b) que lo clasifique un modelo (descartada: «a ojo»).
- **D3. Desempate**: **(a, recomendada)** solape, prioridad, `id` lexicografico con aviso; (b) desempate por modelo de la 0.16.
- **D4. Modelo**: **(a, recomendada)** tabla por carril, complejidad y rol con `modelos_permitidos` que solo restringe y un escalon menos en verificacion; (b) un `modelo_sugerido` por tarea.
- **D5. Cuando se calcula**: **(a, recomendada)** diseno e implementacion en `plan`, congelados con `approve`; revision en cada ronda con diff real y config de la base; (b) todo recalculado siempre.
- **D6. Revision cuando el dominio no se ve en las rutas**: **(a, recomendada)** revisora del dominio si esta instalada y declarada; si no, generica con aviso «dominio sin revisora» medido; (b) solo por rutas.
- **D7. Medicion**: **(a, recomendada)** columna «Clase» con lista cerrada mas modelo y tokens por rol (divergencia con F5-4); (b) solo por severidad.
- **D8. Metadatos y anotacion**: **(a, recomendada)** en el bloque yaml de cada skill (mas `recursos:` para las ajenas) y `roles.<rol>.agentes` como lista con `regla` = id de fila y hash (divergencia con F1-9); (b) catalogo aparte.
- **D9. Las 10 tareas**: **(a, recomendada)** 5 de ToolsaWeb y 5 de OpenDockTrading estratificadas antes de mirar resultados (solo docs, fix, funcionalidad multidominio, riesgo alto, una con `skills_recomendados` de la 0.16 y una de seguridad); (b) las 10 mas recientes.
- **D10. Quien comprueba «a mano»**: **(a, recomendada)** Carlos marca la columna en la tabla de las 10 y el revisor por pares contrasta la aritmetica; (b) el revisor IA (ya no seria una persona).

## Suposiciones no verificadas

- Que ToolsaWeb y OpenDockTrading conservan los commits `93c9618ac` y `fab8d7e05`.
- Que la 0.16 no registra el modelo real usado por fase (mirar `metricas.ts`).
- Que los 15 informes usaron `code-quality-reviewer` (leido en 6).
- Que `.claude/skills` y `.claude/agents` de los pilotos estan vacios o no existen.
- Que los 33 IMPORTANTES no cuentan dos veces un hallazgo reabierto.

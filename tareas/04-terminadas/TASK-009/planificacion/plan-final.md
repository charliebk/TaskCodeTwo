# Plan — TASK-009: F1-8 ADR seguridad

(Lo consolida el agente unificador a partir de 2 roles de brainstorm
lanzados en paralelo: arquitectura, riesgos.
Los desacuerdos entre roles se senalan, no se promedian.)

## Enfoque propuesto

Duracion estimada: 60 min

Señales: 1 ADR nuevo + 1 fila en el índice, sin código de producción; unas 15 preguntas heredadas de F1-5/F1-6/F1-7 que cerrar o delegar; 2 PoC locales pequeñas y lecturas `GET`; 2 criterios; complejidad media (heurística, 4 puntos). Queda en el límite: si las PoC se alargan, el troceo natural sería ADR primero y PoC después, pero no se propone porque el ADR sin PoC no tiene evidencia.

- Se crea `docs/adr/F1-8-seguridad-modelo-de-amenazas.md` con la plantilla de `docs/adr/README.md` y una sección «Modelo de amenazas» entre Contexto y Alternativas (arquitectura).
- El modelo se ordena por cinco fronteras de confianza (Z1 máquina de la persona, Z2 contenido del repo como dato no confiable, Z3 runners de GitHub, Z4 runner propio, Z5 plataforma), con una matriz «activo → quién lo lee o escribe → mitigación → ADR o tarea dueña» (arquitectura). La tabla de modos de fallo por gravedad de riesgos entra como contenido de esa matriz.
- Workflows en dos clases: **ejecutan código del PR** (`pull_request`, `permissions: contents: read`, sin secretos) y **tienen privilegio** (publican checks o usan la clave de la App, sin código del PR; el diff y los artefactos entran como datos). La proponen los dos roles (arquitectura y riesgos).
- Tabla «pregunta heredada → decisión → origen» para F1-5 dec. 6, F1-6 (repository_dispatch, claves en la misma máquina, tarea que toca workflows) y F1-7 (runner propio, `verificacion` falsificable, hook), más una sección «Divergencias con F1-5/F1-6/F1-7» (arquitectura).
- Orden: lecturas `GET` → PoC locales → ADR → fila en el índice.

## Desacuerdos entre roles, y como se resuelven

- **`carril` y el workflow modificado en el propio PR** — arquitectura: `carril` con `pull_request_target` o `workflow_run` y checkout solo de la base / riesgos: `pull_request_target` no cubre `pull_request_review`, que corre el workflow del ref del PR (riesgos lo marca no verificado), así que no cierra el hueco por sí solo, y la migración tiene un estado intermedio con dos `carril` en el mismo commit. **Gana riesgos** en lo que el ADR afirma: arquitectura no respondió a lo de la review, y con GitHub en solo lectura no se puede verificar. Va a D1.
- **¿Basta F1-7 dec. 5 para el runner propio?** — arquitectura: basta con «solo repo privado y sin PR de forks» / riesgos: no basta, porque el job corre donde están `ssh-agent`, el token de la persona y `exampleData`, y un runner no efímero contamina la pesada de `main`; pide usuario de servicio, directorio limpio y separar `main` de las tareas, y si no se garantiza, quitar la pesada en PR. **Gana riesgos**: da un escenario concreto que arquitectura no rebate. Va a D3, porque cuesta configurar la máquina de Carlos.
- **Alcance del hash de confianza local** — arquitectura: una sola interfaz `confianza(repo, config) -> confiado | cambiado | nunca`, con el hash de la config fuera del árbol / riesgos: el hash cubre la orden **y lo que ejecuta**; si no, un `git pull` cambia el script y se ejecuta sin preguntar (H2). **Gana riesgos** en el alcance y arquitectura en la forma: no se contradicen en lo demás, y la interfaz única de arquitectura se conserva. Va a D4, porque el coste de hashear «lo que ejecuta» no lo midió nadie.
- **¿Se presentan las clases de workflow como mitigación cerrada?** — arquitectura: las clases «quedan fijadas por la estructura» / riesgos: que `GITHUB_TOKEN` pueda crear check-runs con cualquier nombre no se verifica con `GET`; el ADR lo marca «no verificado» y lo manda al E2E de F2-6. **Gana riesgos**: CLAUDE.md dice «si no se ha ejecutado, no se afirma».
- Desacuerdos que arquitectura previó y no llegaron a darse: riesgos no pidió STRIDE ni detección de inyección. Los dos defienden la inyección por capacidades: el veredicto sale de comprobaciones estructuradas y no de la prosa (riesgos), y ninguna IA aprueba ni integra (arquitectura, P3).
- En lo que coinciden sin discutir: partir el revisor en un job que ejecuta sin secretos y otro que publica, y que ningún workflow escuche `repository_dispatch`, con `doctor` comprobándolo.

## Riesgos aceptados y que los contiene

- Check falsificado (`carril` o `verificacion`) por scripts del PR con `checks:write`, todos con `integration_id` 15368 (riesgos): lo contienen las dos clases de job y que `doctor` exija permisos por defecto en solo lectura. Queda «no verificado» hasta F2-6.
- Caché por árbol de F1-7 envenenada desde el head del PR (riesgos): una línea `arbol=` solo cuenta si la publicó el job publicador (por job o `run_id`, no solo por `app_id`). Cambia F1-7, va a D5.
- Prompt injection en `taskcode/tareas`, que se empuja sin PR (riesgos): veredicto estructurado y defensa por capacidades (arquitectura). La batería de casos es de F6-7.
- Secreto filtrado o `approve` falsificado con la clave de la persona: **no tienen vuelta atrás** (riesgos). Lo contiene un procedimiento de rotación en el ADR. No se ha verificado si el «Verified» sobrevive a retirar la clave.
- Rotación de la clave de una App a medias, o runner caído: fallan cerrado (riesgos, D3a de F1-7), y `doctor` dice la causa.
- `git clean -ffdx` lanzado desde código del PR en local borra el worktree de `tareas/` (riesgos): ningún rol propone mitigación. Se acepta como riesgo nombrado en el ADR.

## Plan de pruebas

Toda PoC es local en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-009\` y se borra al cerrar. En GitHub, solo `GET`.

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- **Criterio 1 — Modelo de amenazas con mitigaciones**
  - T1.1 PoC del hash de confianza: config sin cambiar da `confiado`; config cambiada da `cambiado`; script ejecutado cambiado con la config igual da `cambiado` según D4 (arquitectura los dos primeros casos, riesgos el tercero).
  - T1.2 Clasificación estática de workflows de ejemplo: clase de job, `permissions`, ninguno con `repository_dispatch`, sin `${{ }}` de datos del evento en `run:` (arquitectura y riesgos).
  - T1.3 Revisión del ADR: cada fila de la matriz tiene mitigación y tarea dueña, o «no verificado → F2-6» (riesgos).
- **Criterio 2 — Cumple F1-8 y los requisitos que cita**
  - T2.1 `GET` de `actions/permissions/workflow` y de los ajustes de Actions para fijar el punto de partida (arquitectura).
  - T2.2 Cada pregunta heredada de F1-5, F1-6 y F1-7 queda cerrada o delegada con dueño en la tabla, y cada cambio a un ADR aceptado aparece en «Divergencias» (arquitectura).
  - T2.3 Las cuatro partes del enunciado (local y CI, confianza, prompt injection, runner propio) tienen sección propia, y hay fila en el índice de `docs/adr/README.md`.

## Lo que no cubrieron los roles

- Los requisitos que cita F1-8: el plan (l. 312) no cita ninguno explícito. Ninguno de los dos roles cruzó `REQUISITOS-FUNCIONALES.md` entero; solo se citan RR-03, RR-13, RR-18, RF-70 y H2. Se hace en T2.3.
- Lo que corre en local con el agente en lotes de RR-18: arquitectura no leyó `AMPLIACION-2026-11-14.md` entero. Hay que leerlo antes de redactar la frontera Z1.
- Compatibilidad: no se han leído los workflows de ToolsaWeb y OpenDockTrading (riesgos). El ADR deja dicho que `actualizar` (RR-13) quizá tenga que reescribirlos.
- Salidas que faltaron: ninguna. Se lanzaron 2 roles por complejidad media (seguridad +2, dos dependencias +2). No hubo rol de rendimiento ni de dominio.

## Suposiciones no verificadas

- El revisor necesita una clave de API del modelo en Actions; ningún ADR la menciona (arquitectura).
- `workflow_run` recibe los artefactos del run de `pull_request` en repos privados sin forks (arquitectura).
- `pull_request_review` ejecuta el workflow del ref del PR (riesgos).
- F0-2 se aplica a la 0.16 y F5-6 reutilizará su mecanismo (arquitectura).
- Este repo (RR-14) no sirve de prueba: sigue con `ci.yml` y la cuenta de la persona (riesgos).

## Lo que necesita decision de una persona

Este plan no vale hasta que una persona lo apruebe con `taskctl approve`.

- **D1 — Qué afirma el ADR sobre `carril`** (desacuerdo entre arquitectura y riesgos)
  - a) `carril` en `pull_request_target`/`workflow_run` con checkout de la base, presentado como mitigación (arquitectura).
  - b) Lo mismo, pero marcado como parcial: no cubre `pull_request_review`, la migración borra el workflow viejo en el mismo commit y se manda al E2E de F2-6 (riesgos). **Recomendada**: arquitectura no rebate el caso de la review y no se puede verificar en solo lectura.
- **D2 — Dónde vive la clave del modelo del revisor IA** (la dejó abierta arquitectura)
  - a) En el job 1 (`pull_request`, ejecuta código del PR): el LLM reproduce directamente, pero el código del PR puede robar la clave (arquitectura).
  - b) En el job 2 (sin código del PR): el LLM lee como dato el artefacto de la reproducción, que se hizo sin secretos (arquitectura, opción 2, con la regla de riesgos «la clave solo en un job que no ejecuta nada del PR»). **Recomendada**: es el único reparto en el que ningún secreto comparte job con código hostil.
  - c) No se decide aquí y pasa a F6-3, a la espera de confirmar si hace falta clave (suposición de arquitectura).
- **D3 — Condiciones del runner propio** (desacuerdo)
  - a) Basta F1-7 dec. 5: repo privado y sin forks (arquitectura).
  - b) Además usuario de servicio sin acceso al perfil, directorio limpio por job y ningún job de `main` en el runner de tareas (o runner efímero); si no se garantiza, no hay pesada en PR (riesgos). **Recomendada**.
- **D4 — Alcance del hash de confianza local** (desacuerdo)
  - a) Hash de la config de `.taskcode/` (arquitectura).
  - b) Hash de la orden y de lo que ejecuta, con la misma interfaz única (riesgos sobre la forma de arquitectura). **Recomendada**: (a) no cubre el escenario H2. Su coste se mide en T1.1.
- **D5 — Cambiar F1-7: `arbol=` solo vale si lo publicó el job publicador** (lo propone riesgos; arquitectura no lo trató)
  - a) Sí, registrado en «Divergencias con F1-7». **Recomendada**: sin esto, un check falso deja la caché envenenada de forma persistente.
  - b) No se cambia y se confía solo en la separación de jobs.

### Respuestas de la persona

Decisiones respondidas por la persona (Carlos) el 2026-11-14 con la consigna «responder la opcion recomendada a todas las preguntas»: **D1 (b)** `carril` con `pull_request_target` como mitigacion parcial, con E2E en F2-6; **D2 (b)** la clave del modelo del revisor vive en el job publicador sin codigo del PR, que lee como dato el artefacto de la reproduccion; **D3 (b)** runner propio con usuario de servicio, directorio limpio y `main` separado (si no se garantiza, no hay pesada en PR); **D4 (b)** el hash de confianza cubre la orden y lo que ejecuta, con la interfaz unica de arquitectura; **D5 (a)** se cambia F1-7 para que `arbol=` solo valga si lo publico el job publicador, anotado en «Divergencias». Sin troceo.

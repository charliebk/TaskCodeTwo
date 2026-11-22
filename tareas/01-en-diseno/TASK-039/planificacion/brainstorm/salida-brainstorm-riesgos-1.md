# Brainstorm — TASK-039, rol riesgos (ronda 1)

- Rol: `brainstorm-riesgos`
- Agente: subagente con la definición `agents/brainstorm-riesgos.md` de taskcode-plugin 0.16.2 (solo lectura; sin enfoque de arquitectura a la vista)

## Modos de fallo, ordenados por gravedad, con el escenario concreto de cada uno

- **El orquestador o el agente que lanza integran o aprueban** — la sesión desatendida corre con el token de `gh` de la persona al alcance (A5 de F1-8, abierto; el scope `workflow` se observó el 2026-11-14) y el agente de implementación hace `gh pr merge` o reescribe un workflow porque el texto de la tarea se lo pide — merge en `develop` sin persona, irreversible en la práctica — el orquestador no llama nunca a `finish`/`merge-pr`/`approve` y para en `merge-pr` como parada fija; la exposición a A5 se declara riesgo aceptado y no verificado, no se maquilla.
- **Desviación: paradas decididas fuera de `accion()`** — verificado: `accion()`/`politica()` de F1-10 (decisión 1) no existen en el código (`grep` sin resultados; `next` no da `accion`, `carril` ni `paradas`) y solo existe la clave `paradas_extra` (`config/mod.rs:86`) — si F4-4 resuelve las paradas en su propio bucle, una configuración o un tipo convierte `merge-pr`/`approve` en `continuar` y la PoC C (0 `continuar` en 1260 combinaciones) deja de valer para el código real — F4-4 implementa primero `accion()` como función pura única y el bucle solo la consulta (F1-10: «F4-4 usa `accion` y no decide nada por su cuenta").
- **Duplicación con `lote` (F4-7/F1-16)** — F1-16 ya diseña un bucle que reconstruye desde Git y el diario y llama a `siguiente_paso`; si F4-4 escribe otro bucle con otro modelo de estado («fase actual» guardada), habrá dos máquinas de estados que divergen al primer cambio — una sola función de paso (tarea suelta = lote de uno); nunca guardar «paso actual».
- **Bucle de correcciones sin fin o gastando rondas por el entorno** — verificado en TASK-085: un instalador de Visual Studio quitó `cl.exe` y dejó Git a medias durante la verificación — el orquestador lee el rojo como fallo del código, lanza correcciones que «arreglan» lo que no estaba roto y consume el tope de rondas — clasificar antes de corregir: fallo de entorno (sin compilador, Git a medias, `LNK1104`, timeout) → `detener` con `entorno-no-responde` sin gastar ronda (como F1-16 B3).
- **Verificaciones solapadas sobre el mismo `target/`** — verificado: `tc2 verificar` (`verificacion/ejecutar.rs`) no tiene cerrojo y la misma orden cuelga del `Stop` con `--hook`; la rápida tardó 1305 s en TASK-085, más que el tope de 600 s de una llamada de herramienta — el orquestador la lanza en segundo plano, el turno termina, el hook arranca otra en el mismo worktree: `LNK1104` o dos cortes por tiempo (pasó en TASK-118, 127, 045, 050) — cerrojo por directorio `target/` (o `CARGO_TARGET_DIR` propio) y el orquestador no devuelve el control con una verificación viva.
- **Disfraz: «PR listo» demostrado con un modelo falso** — no hay saldo en la API de Anthropic (TASK-052 y 085 lo dejan «sin comprobar»); `revision-ia` real devolverá `fallo`/`sin-revisor` — el E2E pasa con un revisor simulado y la tarea se cierra como «sin intervención» sin haberlo hecho nunca con el modelo — el `## Resultado` debe decir «no verificado con modelo real» y el orquestador tratar `sin-revisor`/`fallo` como parada, nunca como verde.
- **Diluvio de contexto** — cada fase vuelca su salida, los diffs y los hallazgos en la conversación que orquesta; en una tarea con 3 rondas se agota el contexto o se compacta a mitad y se pierde qué ronda iba — la sesión solo recibe la línea del contrato (`fase ok ...`) y los detalles quedan en ficheros de la tarea.

## Estados intermedios y fallos parciales

- **Corte de la sesión a mitad (ha pasado varias veces)**: la reanudación solo es segura si cada fase es idempotente; verificado que `start` (TASK-033 C2: `sin-cambios`, recrea worktree o rama) y `review` (TASK-048: «repetir converge») lo son; `plan`, la implementación y las correcciones, no consta.
- Corte entre que el agente edita y el commit: worktree sucio al reanudar; ¿se conserva, se descarta o se commitea? RF-23 prohíbe `git add -A`, así que alguien tiene que saber qué ficheros son «del agente» — sin regla, la reanudación mezcla basura o pierde trabajo.
- Corte entre el commit de corrección y el push: el PR sigue en el head viejo y `revision-ia` aprueba lo que no es; la guarda «se revisa el head» de `review`/`finish` debe repetirse al reanudar.
- Corte entre abrir el PR y pasar a `en-revision`: TASK-048 hace el push y el PR antes de la transacción; al reanudar, el PR ya existe — debe converger, no abrir otro (verificado que converge solo en el E2E de una ejecución repetida).
- Subida de carril en `review` (el diff toca `.taskcode/` o `.github/`) → `detener` «vuelve a plan» (F1-10 decisión 5): en una tarea fix el criterio de aceptación pasa a imposible sin persona; el orquestador no debe reintentar en bucle.
- Tope de rondas → `preguntar` (F1-10 decisión 6; `revision-ia` sale `tope`, docs/ci/revision-ia.md:425): sin persona delante, «preguntar» en automático es parar con informe, no elegir `ronda-extra`.
- Dos orquestadores sobre la misma tarea (dos sesiones, o la persona tecleando en el mismo worktree): la toma de `start` es atómica, pero `plan` y las correcciones no tienen dueño exclusivo (supuesto: no he visto cerrojo en `plan guardar`).

## Compatibilidad hacia atras

- Verificado: `TASK-039/tarea.md` y `TASK-085/tarea.md` no tienen `carril:` ni `roles:` en el frontmatter (formato 0.16); el orquestador sobre una tarea así debe fallar cerrado (`detener` con motivo) y no deducir el carril ni el modelo «a ojo» (RR-17).
- Las tareas planificadas antes de TASK-085 no traen `roles.*`; usar el `modelo_sugerido: sonnet` heredado de la 0.16 como si fuera la selección de F4-6 sería Desviación silenciosa.
- La fila de transición de la 0.16 lleva `modo: automatico` decidido por la persona con `version 0.16.1`; un modo congelado por la 0.16 no es un modo congelado por `tc2 plan` y no debe habilitar el encadenado.
- El propio repo sigue con taskctl 0.16 hasta F4-5: probar el orquestador sobre TaskCodeTwo escribiría en `taskcodetwo/tareas` y en `tareas/` a la vez; las pruebas, en un repo de pruebas.

## Vuelta atras

- **Sin retorno**: los commits en la rama de estado (`taskcode/tareas` no admite borrado, F1-8 A4) y el PR abierto por la App (se cierra, no se borra; ya notificó).
- Reversible: rama y worktree de la tarea (`liberar`, `ciclo/liberar.rs`), cerrar el PR, devolver la tarea a `planificada`.
- Si sale mal tras desplegar F4-4: desactivar el automático por configuración (modo `manual` → `detener` en cada fase) basta, siempre que no haya un segundo camino de decisión fuera de `accion()`.

## El riesgo que mas te preocupa (UNO solo)

Que el orquestador decida las paradas en su propio código en vez de en una `accion()` única (que hoy no existe): es a la vez Desviación de F1-10, Duplicación con `lote` y la vía por la que un día un `continuar` cae sobre `merge-pr` sin que ningún test de la PoC C lo vea. Desacuerdo previsto con arquitectura si propone un bucle con estado propio o una skill que decida las fases con el modelo (eso sería Disfraz de «determinista», RR-17).

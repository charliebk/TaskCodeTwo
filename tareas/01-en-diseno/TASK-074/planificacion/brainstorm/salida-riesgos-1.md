# Salida del rol de riesgos — TASK-074 (ronda 1)

(Volcada por el orquestador. Se lanzan arquitectura y riesgos y un unificador por la regla de F1-15 para tareas de puertas y seguridad, aunque la peticion generada por la 0.16 diga «1 rol, sin unificador».)

## Riesgos, de mayor a menor dano

1. **Tarea en rojo o sin revisar en `lote/*` sin que ninguna plataforma lo impida.** Los rulesets solo actuan sobre la rama destino y ToolsaWeb y OpenDockTrading son privados gratuitos (F1-5 D2a: 403, «puertas no impuestas»); nada impide al agente un push directo o un `git merge` local a `lote/<n>`; la persona aprobaria por la manana un PR con codigo que nadie verifico. **Mitigacion**: antes de abrir el PR final, el CLI comprueba que la cadena de primer padre de `lote/*` esta formada solo por merges suyos, cada uno con el trailer `Tarea:` y el SHA del head verificado; cualquier commit ajeno impide el PR y va al informe. **PoC**: meter un push directo y ver que se detecta.
2. **Interferencia** (la persona escribe en el prompt, o un companero empuja a la rama de tarea, al worktree o a `lote/*`): es el caso real de la 0.16 (se revisa X y se integra Y, o rondas extra). **Mitigacion**: integracion por compare-and-swap: solo si el head de la tarea es igual a `ultimo_commit_revisado` y `lote/*` sigue en el SHA anotado al empezar la tarea (merge por API con SHA esperado); antes de `review` y de integrar, `status --porcelain` limpio y HEAD esperado; si no, la tarea se aparta como «interferida», sin re-revisar en bucle. **PoC**: escribir a mitad de una tarea y medir que se detecta antes de revisar.
3. **Corte entre «merge en `lote/*`» y «estado escrito en la rama de estado»** (limite de uso, red, apagado): `--reanudar` repetiria la tarea (doble merge y gasto) o se saltaria puertas. **Mitigacion**: la fuente de verdad es Git (merges de `lote/*` con su trailer); `--reanudar` reconcilia desde ahi; cada paso tiene una precondicion comprobable, nunca un «paso actual» guardado. **PoC**: matar el proceso en cada frontera entre pasos.
4. **Limite de uso del proveedor tomado por fallo** (paso en esta sesion): distinguir «el entorno no responde» (para limpio y reanudable, sin apartar) de «rojo» (apartar); una ronda cortada se repite entera, nunca da verde, y sus tokens cuentan.
5. **Prompt injection nocturna**: una tarea reescribe en la rama de estado el `tarea.md` de otra, su carril, la lista o el contador de presupuesto. **Mitigacion**: manifiesto fijado al lanzar (lista, hash de cada `tarea.md`, carril, presupuesto); hash cambiado → la tarea se aparta; carril efectivo del diff que sube a riesgo-alto (`.taskcode/`, workflows) → se aparta; el presupuesto lo mide el proceso del CLI, nunca un fichero escribible por el agente.
6. **Apartar arrastra al resto**: cadenas largas (como la Fase 1) y dependencias implicitas (B uso codigo de A por estar en el lote, sin declararla); la rama de A, construida sobre `lote@sha`, ya no se integra sola en develop. **Mitigacion**: cierre transitivo sobre dependencias declaradas (tambien las de fuera del lote); dependencia parcial cuenta; la rama apartada se marca con su base de lote para rehacerla desde develop. **PoC**: con un lote real de ToolsaWeb, medir que porcentaje se aparta en cascada.
7. **Coste sin techo.** Medido: TASK-011 ~21 M tokens, 12,4 M en 4 rondas; en TASK-006, 009, 011, 012 y 013 la correccion introdujo un IMPORTANTE nuevo y la ronda extra la decidio Carlos. **Mitigacion**: tope de rondas fijo por tarea, sin ronda extra automatica (es decision de la persona por la manana); presupuesto por tarea ademas del de lote; no empezar ronda si lo que queda es menor que la mediana de una ronda; mismo hallazgo reaparecido → se aparta.
8. **PR final contra un develop que avanzo**: rama al dia (D6a), sin cache de F1-7 para el arbol nuevo, conflictos sin nadie. **Mitigacion**: de noche no se resuelve ningun conflicto; al final `merge-tree` contra `origin/develop`, conflictos al informe; verificacion completa del arbol final obligatoria.

## Puntos sin retorno

- Escrituras del lote en la rama de estado (no admite borrado, F1-8 A4): un estado falso por inyeccion solo se corrige hacia delante.
- Tokens y tiempo gastados: por eso el corte va antes de cada ronda.
- Rechazar una sola tarea dentro del PR final: las siguientes ya vieron su codigo; solo cabe revertirla en `lote/*` y volver a verificar todo el lote.

## Descartado a proposito

- Dos lotes con el mismo nombre en dos maquinas: basta reservar el lote en la rama de estado con el CAS de F1-2.
- Disco y rutas largas por worktrees: uno por tarea, se quita al integrar; rutas largas ya en F1-3.
- Detectar texto hostil por patrones: F1-8 dice que solo vale como senal.

## Desacuerdos previstos

- Con arquitectura y P13 (PR unico todo-o-nada): el ADR debe nombrar el rechazo parcial como riesgo aceptado con su procedimiento (revert y verificacion completa del lote).
- Con F1-8 A16 (su mitigacion depende de A5): el lote nocturno no se habilita mientras A5 este abierto, salvo que la sesion corra sin el token de la persona.
- Con una ronda extra automatica: en contra; la evidencia dice que la decidio una persona en las cinco tareas.

## Suposiciones no verificadas

- Que `verificacion` y `revision-ia` se disparan en PR con base `lote/*` (filtro `branches` de F5-1).
- Que el CLI puede medir los tokens de un agente muerto a mitad de ronda (origen de `tokens_*` en la 0.16.1).
- Que el merge de cada tarea en `lote/*` va por PR y API como el `finish` de F1-3 (lo da arquitectura).
- Que el plan real de ToolsaWeb es gratuito (confirmar con un GET de rulesets).
- GitLab Free: sin aprobaciones impuestas (F1-13) tiene el mismo hueco que el riesgo 1.

Fuentes leidas: F1-5 (D2a), F1-8 (A5, A10, A15, A16, decision 4), F1-3, F1-10 (columna «Lote»), tareas/04-terminadas/TASK-011/tarea.md.

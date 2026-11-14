# Plan — TASK-003: F1-2 ADR escrituras atomicas, reserva de IDs y toma de tareas

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 45 min

De la rama abierta al cierre con merge. Senales: un ADR nuevo y una fila en el indice de `docs/adr/README.md`, sin codigo de la 2.0; un guion de PoC con concurrencia real de procesos en segundo plano y un remoto inalcanzable, sin GitHub; 3 criterios. Cabe en 60 min: no se trocea. Precedente: TASK-002 estimo 60 y tardo ~12.

### Que se crea y que no se toca

1. **Se crea** `docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md` con la plantilla de `docs/adr/README.md`. «Como se ve en GitHub»: «no aplica: el protocolo solo usa `fetch` y `push` sin force, igual en GitHub y GitLab (RR-15); las reglas de la rama son de F1-5».
2. **Se extiende** `docs/adr/README.md` con una fila del indice; la plantilla no cambia.
3. **Se crean** en `poc/` de la tarea `poc-f1-2.sh` y `poc-f1-2.log`, con la forma de `poc-f1-1.sh` de TASK-002 (argumento = carpeta de trabajo, funciones `tarea` y `roadmap`, secciones `===`). Se ejecuta en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-003\` con remotos bare locales; la carpeta se borra al terminar.
4. **No se tocan** `docs/adr/F1-1-*.md`, `.github/`, `.taskcode/` ni el flujo de la 0.16 (RR-14).

### Que decide el ADR

- **Donde vive.** Un unico punto de escritura sobre `taskcode/tareas` (modulo de la rama de estado, F2-3); todos los comandos que cambian estado pasan por el: `new`/`import` (F2-4), `start` (F3-6), transiciones, `liberar`/`reasignar` (F3-10). Transaccion optimista (firmas orientativas, no codigo):
  ```
  operacion(estado_actual) -> cambios | Rechazo(motivo)
  transaccion(operacion) -> Publicada | Rechazada(motivo) | SinRed(pendiente) | Agotada
  ```
- **Protocolo de push con reintento** (criterio 1):
  1. Cerrojo local del clon `.git/taskcode-estado.lock` (persona y agente del mismo clon no se pisan el indice del worktree `tareas/`).
  2. `fetch` de `taskcode/tareas` y worktree en `origin/taskcode/tareas`.
  3. Ejecutar la operacion sobre ese estado fresco (puede rechazar: la tarea ya tiene `asignado`).
  4. Regenerar `ROADMAP.md`, commit pequeno y `push` sin force. **El rechazo non-fast-forward es el compare-and-swap.**
  5. Si se rechaza: descartar el commit local, espera exponencial con jitter y volver al paso 2 **re-ejecutando la operacion** (no rebase). Tras N intentos, `Agotada` con codigo de salida propio (RR-01).
  6. Soltar el cerrojo. El `ROADMAP.md` no se mergea nunca: se regenera (F1-1).
- **Reserva de IDs (H3).** ID = maximo de las carpetas `TASK-*` + 1, calculado dentro de la operacion. Las carpetas no se borran nunca (F1-1) y las descartadas conservan su ID (RF-07): no se reutiliza ninguno. Un `import` de N tareas (RF-02) es **una** operacion: un commit, un push, N IDs consecutivos. La rama de codigo sale del ID y hereda su unicidad; F2-4b anade la comprobacion contra el remoto.
- **Toma de tareas.** `start` exige `asignado` vacio o igual a quien la lanza; escribe `asignado` y estado y **solo con el push publicado** crea rama y worktree. Si al re-ejecutar ve que otro la tomo: `Rechazada("la lleva X")`, sin crear nada. Idempotente: si ya la lleva la misma persona, no escribe estado y solo crea lo que falte. El «quien» lo llena F2-13.
- **Sin red** (criterio 2, D2): las operaciones que reservan algo compartido (`new`, `import`, `start`, `liberar`, `reasignar`) exigen red; sin red devuelven `SinRed` con su codigo y sin commit local. Las de una tarea que ya lleva quien las lanza (transiciones, plan, informes) se commitean en local como «pendientes de publicar» y la siguiente sincronizacion las publica re-ejecutandolas. `status` dice cuantas hay y la hora de la ultima sincronizacion (F3-5b). **Un repo sin remoto no esta «sin red»**: el commit local es la transaccion (RN-06).
- **Frescura (RR-12).** Toda escritura empieza con `fetch`; la lectura fresca es de F3-5b.

### Orden

1. `poc-f1-2.sh` en `poc/`, ejecutado en la carpeta temporal guardando el log.
2. Redactar el ADR con comandos, salida observada y tabla de requisitos.
3. Fila del indice en `docs/adr/README.md`.
4. Borrar la carpeta temporal tras comprobar que no queda nada que conservar.

## Lo que el rol no cubrio

Un solo rol (arquitectura). No revisado:

- **Riesgos** (criterios 1 y 2): error de autenticacion confundido con «sin red»; hook de push que rechaza; proceso que muere entre el push de `start` y la creacion de la rama (solo cubierto por la idempotencia); cerrojo local huerfano; worktree `tareas/` desmontado o borrado con `git clean -ffdx` (abierto de F1-1); conflicto de una operacion pendiente sin red con una reasignacion hecha por otro. El ADR los deja como preguntas abiertas para F2-3 y F3-10.
- **Testing**: plan de pruebas minimo por criterio.
- **Dominio** (criterio 3): que significa «lista para empezar», si un agente puede tomar una tarea en nombre de una persona, si las transiciones propias sin red rompen lo que el equipo espera ver (RR-12).
- **Rendimiento**: contencion razonada en forma y medida solo con 3 clones.

## Riesgos aceptados y que los contiene

- **El ADR decide sobre la 2.0 sin cambiar este repo (RR-14)**: «Consecuencias» nombra quien lo ejecuta (F2-3, F2-4, F2-4b, F3-6, F3-10, F3-5b, F1-5 para prohibir force-push en la rama de estado).
- **Concurrencia simulada con procesos en segundo plano de Git Bash en Windows**, no con maquinas distintas. El bare local rechaza non-fast-forward igual que GitHub (el log de F1-1 lo mostro). Que GitHub se comporte igual ante pushes simultaneos queda «no verificado» y pasa a F9-4.
- **N intentos y tiempos de espera** son valores del ADR, no medidos: parametro con valor por defecto; F9-4 lo valida.
- **La cola de pendientes sin red** solo admite operaciones sobre la tarea propia y se publica con el mismo mecanismo, sin segundo camino.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio). Todos son secciones de `poc/poc-f1-2.sh` en repos temporales con remotos bare bajo `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-003\`, sin GitHub; el ADR copia comandos y salida; la carpeta se borra al terminar. Ninguno pesado.

- **Protocolo de push con reintento → 3**
  1. **Altas concurrentes**: 3 clones lanzan a la vez, en segundo plano, 4 altas cada uno. Observar: 12 IDs distintos y consecutivos, 12 carpetas en `origin/taskcode/tareas`, `ROADMAP.md` con 12 entradas, numero de reintentos, ningun push forzado.
  2. **Toma concurrente**: 2 clones `start` a la vez sobre la misma tarea. Observar: uno publica `asignado`, el otro recibe «la lleva X», la rama de codigo existe solo en el ganador.
  3. **Rebase frente a re-ejecucion**: 2 clones crean «la siguiente» desde el mismo estado. Con `pull --rebase`: conflicto add/add en `TASK-00N/tarea.md` o mismo ID. Con re-ejecucion: IDs distintos sin conflicto. Evidencia de D1.
- **Que pasa sin red → 3**
  1. Remoto inalcanzable (URL a ruta inexistente): `new` y `start` terminan con el codigo «sin red», sin commit local, `git status` del worktree `tareas/` limpio.
  2. Transicion de una tarea propia sin red: queda commit pendiente; al restaurar el remoto, la siguiente operacion la publica re-ejecutandola y `origin` la contiene.
  3. Repo sin remoto: altas y toma funcionan como commit local (RN-06).
- **Cumple F1-2 y los requisitos que cita → 1**
  - Tabla del ADR que enlaza RR-09, RR-12 (IDs y ramas unicos, una persona por tarea, estado fresco), H3, RN-05, RN-06 y RR-01 con decision y evidencia. Al cerrar: fila del indice, `bash .github/scripts/comprobar-repo.sh` en verde y `taskctl doctor` sin errores.

## Lo que necesita decision de una persona

Decisiones respondidas por la persona (Carlos) el 2026-11-14 con la consigna «responder la opcion recomendada a todas las preguntas»:

- **D1. Como se rehace el cambio tras un push rechazado** → **(a)** re-ejecutar la operacion sobre el estado nuevo tras descartar el commit local (recalcula ID y precondiciones). Descartada (b) `pull --rebase`: no recalcula el ID ni ve que otro tomo la tarea.
- **D2. Que pasa sin red** → **(b)** las operaciones compartidas (`new`, `import`, `start`, `liberar`, `reasignar`) exigen red; las de la tarea propia se commitean en local y se publican re-ejecutandolas en la siguiente sincronizacion; `status` muestra pendientes y ultima sincronizacion. Descartadas (a) todo exige red y (c) IDs provisionales renumerados (rompe la unicidad de ramas y las referencias).
- **D3. Tope de reintentos** → **(a)** 5 intentos con espera exponencial y jitter; despues `Agotada` con su codigo de salida (RR-01). Parametro con valor por defecto.
- **D4. Formato del ID** → **(a)** `TASK-NNN` consecutivo reservado con el push (H3). Descartado (b) ULID o prefijo por persona.

Sin troceo: la duracion estimada no pasa de 60 min.

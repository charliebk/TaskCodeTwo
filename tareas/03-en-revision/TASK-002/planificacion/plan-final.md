# Plan — TASK-002: F1-1 ADR carpeta fija por tarea y rama de estado

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 60 min

De la rama abierta al cierre con merge; senales: un ADR nuevo y un README de indice, sin codigo de la 2.0; dos PoC locales con guion y una comprobacion en un remoto temporal; 3 criterios. Cabe en 60 min: no se trocea. Precedente: TASK-001 estimo 30 min y tardo 10, pero no tuvo PoC.

### Que se crea y que no se toca

1. **Se crea `docs/adr/`** (hoy no existe) con dos ficheros:
   - `docs/adr/README.md`: indice de los ADR y plantilla minima que copiaran F1-2..F1-17 (Contexto · Alternativas · Pruebas de concepto · Como se ve en GitHub · Decision · Consecuencias · Fuera de alcance). Al ser el primer ADR, fija la convencion para todos los demas.
   - `docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md`: el ADR (nombre segun D2).
2. **No se toca** `.github/`, `tareas/`, `.taskcode/config.yml` ni el flujo de la 0.16. Por RR-14 este repo sigue con `tareas/` en `develop` hasta el relevo de la Fase 4. El ADR decide sobre la 2.0 y deja escrito que cambia aqui cuando llegue (F2-3 y la migracion de F1-12). Retirar las comprobaciones a)–c) de `.github/scripts/comprobar-repo.sh` es una consecuencia del ADR, no trabajo de esta tarea.

### Contenido del ADR

- **Alternativas comparadas**:
  - **A (recomendada, P1).** Rama **huerfana** `taskcode/tareas`, sin historia comun con `develop`, montada con `git worktree` en `tareas/` del worktree principal; `tareas/` en el `.gitignore` de las ramas de codigo. En la raiz de la rama: `TASK-NNN/` fija (`tarea.md`, plan, informes, resultado) y `ROADMAP.md`. El estado es un campo del frontmatter.
  - **A' (variante descartada).** La misma rama pero nacida de `develop`: GitHub dejaria abrir un PR de `taskcode/tareas` contra `develop` y mezclar historias; la huerfana lo impide por estructura.
  - **B.** Carpeta fija por tarea con estado como campo, **en `develop`**, con commits atomicos del CLI. Exige que la identidad del CLI salte la proteccion de `develop` (choca con P3/RR-08); cada transicion cambia el arbol de `develop` (invalida la cache por SHA de arbol, P6/RR-11, y dispara la CI); una rama de codigo puede volver a llevar cambios de tareas (H8, caso TASK-172).
  - **C (descartada en una linea).** Estado en Issues/Projects de GitHub o en refs fuera de `refs/heads`: RR-10 deja los issues como espejo opcional y las refs ocultas no se ven en GitHub ni en el editor.
- **Escalabilidad de la forma**: con A una transicion es un commit y un push en una rama de texto pequena y el arbol verificado no cambia nunca; con B cada transicion es un push a la rama mas protegida y verificada. Limite de A que el ADR escribe: **una rama solo puede estar en un worktree por clon**; los worktrees de tarea (F1-3, P4) no tienen su propio `tareas/` y leen/escriben el estado del worktree principal a traves del CLI. Esa frontera pasa a F1-3 y F2-3.
- **Contenido del ROADMAP**: secciones de P1 (en curso con quien, en revision, planificadas por hito con dependencias, terminadas, descartadas, grafo Mermaid); se conserva la zona manual `<!-- manual:inicio/fin -->` y la cabecera «formato N» de la 0.16; se regenera en la rama de estado y no se mergea nunca (P2). Escala segun D3 (ToolsaWeb: 403 tareas, 363 con dependencias).
- **Fuera de alcance**: protocolo de push con reintento y reserva de IDs (F1-2); formato de `tarea.md` y del plan (F1-9); worktrees de tarea y flujo de ramas (F1-3); migracion desde la 0.16 (F1-12); como leen el plan la CI o el revisor desde la rama de estado (F6).

### Orden

1. Guiones de las PoC A y B fuera del repo, en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-002\`, ejecutados guardando la salida.
2. Comprobacion en GitHub segun D1.
3. Redactar el ADR con comandos y salida observada, y la tabla A/B contra RR-10, RR-11, RR-12 (parte de estado), P1 y P3.
4. Crear `docs/adr/README.md` con el indice.
5. Borrar carpetas temporales (comprobando antes que no hay nada que conservar); el repo remoto temporal se borra o, si el token no lo permite, se deja anotado para que Carlos lo borre.

## Lo que el rol no cubrio

Planificacion con un solo rol (arquitectura). No se han revisado:

- **Riesgos**: modos de fallo de la rama de estado (worktree desmontado o corrupto, `tareas/` borrada a mano, trabajo sin red, clones 0.16 con `tareas/` en `develop`, rutas largas en Windows). Afecta al criterio 1: el ADR puede decidir A sin haber mirado como falla A. El ADR los lista como preguntas abiertas para F1-2 y F2-3.
- **Testing**: el plan de pruebas es el minimo por criterio; no ha pasado por un rol de testing.
- **Dominio**: no se ha contrastado si los estados de P1 bastan a ToolsaWeb, que tiene `05-archivadas` (RF-71), ni que debe ver «todo el equipo» en el ROADMAP. Afecta al criterio 3 (contenido del ROADMAP).
- **Rendimiento**: coste por transicion razonado a nivel de forma, sin medir.

## Riesgos aceptados y que los contiene

- **El ADR decide sobre la 2.0 sin cambiar este repo**: la seccion «Consecuencias» nombra las tareas que la ejecutan (F2-3, F3-4, F1-12) y lo que hay que retirar de `comprobar-repo.sh`.
- **«Un push a la rama huerfana no dispara la CI»** se apoya en que GitHub lee el workflow del arbol del commit empujado y la rama huerfana no tiene `.github/`: se comprueba en el remoto temporal (D1).
- **La plantilla de ADR condiciona a los demas**: es minima y cualquier ADR posterior puede ampliarla diciendo por que.
- **La PoC no prueba concurrencia real ni el protocolo de reintento** (es de F1-2): aqui solo se muestra que dos escrituras en carpetas distintas hacen rebase sin conflicto.
- **Version de Git**: `git worktree add --orphan` necesita Git >= 2.42; local es 2.56.0 (comprobado con `git --version`). El ADR anota la version minima.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo). Todos son PoC con guion en repos Git temporales bajo `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-002\`; el ADR deja comandos y salida observada; las carpetas se borran al terminar. Ninguno pesado.

- **Decision con pruebas de concepto de las dos → 3**
  1. **PoC A.** Repo con `develop` y remoto bare local; rama huerfana `taskcode/tareas` montada como worktree en `tareas/`, ignorada en `develop`. Observar: `git status` limpio en `develop` tras una transicion; `git log develop` sin commits de tareas; `git rev-parse develop^{tree}` igual antes y despues de diez transiciones.
  2. **PoC B.** Mismo escenario con el estado en `develop`. Observar: el SHA de arbol cambia en cada transicion; conflicto en `ROADMAP.md` al integrar una rama de tarea que lo regenero mientras `develop` avanzaba; reproduccion del caso TASK-172 (un revert que viaja con el cierre de una tarea borra otra). Con A el mismo revert no puede tocar `tareas/`.
  3. **Dos clones escriben en A** en carpetas `TASK-` distintas: el segundo push se rechaza, `pull --rebase` sin conflictos, el ROADMAP se regenera en vez de mergearse.
- **Como se ve en GitHub → 2**
  1. Push de las dos alternativas al repo temporal y registro de: lista de ramas; arbol de `taskcode/tareas` y URL `/blob/taskcode/tareas/TASK-NNN/tarea.md`; que Actions no corre en el push a la rama huerfana; que «Compare» entre `develop` y `taskcode/tareas` dice que no hay historia comun (no hay PR posible); que el diff de PR de una tarea en A no lleva ficheros de `tareas/` y en B si.
  2. Un `ROADMAP.md` sintetico de 400 tareas en GitHub: Mermaid renderizado o error de tamano. Decide D3 con datos.
- **Cumple F1-1 y los requisitos que cita → 1**
  - Tabla en el ADR que enlaza RR-10, RR-11, RR-12 (parte de estado), P1 y sus consecuencias y el contenido del ROADMAP con su seccion y evidencia; al cerrar, `bash .github/scripts/comprobar-repo.sh` en verde y `taskctl doctor` sin errores.

## Lo que necesita decision de una persona

Decisiones respondidas por la persona (Carlos) el 2026-11-14 con la consigna «responder la opcion recomendada a todas las preguntas»:

- **D1. Como se evidencia «como se ve en GitHub»** → **(b) repo temporal**, con un ajuste de quien orquesta: **privado** en lugar de publico (`charliebk/taskcodetwo-poc-f1-1`). Motivo: el token de `gh` tiene los permisos `gist, read:org, repo, workflow` y **no** `delete_repo` (comprobado con `gh auth status`), asi que no se podria borrar desde el CLI; un repo privado no deja nada publico mientras Carlos lo borra a mano. Lo que necesita F1-1 (Actions, Compare, diff de PR, render de Mermaid) funciona igual en privado; la proteccion de ramas, que es lo unico que exige publico en cuenta gratuita, es de F1-5. Si Actions no estuviera disponible en el repo privado, esa afirmacion queda «no verificada» y pasa como criterio a F2-3.
- **D2. Nombre de los ADR** → **(a)** `docs/adr/F1-N-<tema>.md`, con el F-id del plan.
- **D3. Escala del ROADMAP con cientos de tareas** → **(a)** un Mermaid por hito, solo de los hitos no cerrados; las terminadas como recuento mas lista plegada en `<details>`. Se confirma con el dato del test 2.2.
- **D4. Confirmar la alternativa (el `[DECIDE CARLOS]` de P1)** → **(a)** A, rama huerfana `taskcode/tareas` montada en `tareas/`. La PoC B se hace igual completa para dejar la evidencia de por que se descarta.

Sin troceo: la duracion estimada no pasa de 60 min.

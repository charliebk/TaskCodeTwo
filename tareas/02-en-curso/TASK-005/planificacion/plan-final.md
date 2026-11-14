# Plan — TASK-005: F1-4 ADR capa de operaciones Git/GitHub deterministas y su contrato

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 55 min

De la rama abierta al cierre con merge. Senales: un ADR nuevo y una fila en el indice, sin codigo de la 2.0; PoC local en dos partes: (A) linea base de tokens leyendo las transcripciones de TASK-002..004 (solo lectura) y (B) remoto bare con salida cruda frente a porcelana de cada operacion; 3 criterios. Precedentes: TASK-003 estimo 45 y tardo 48; TASK-004 estimo 55 y tardo 48. Cabe en 60 min: no se trocea.

### Que se crea y que no se toca

1. **Se crea** `docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md` con la plantilla de `docs/adr/README.md`.
2. **Se extiende** `docs/adr/README.md` con la fila F1-4 / TASK-005.
3. **Se crean** `poc/poc-f1-4.sh` y `poc/poc-f1-4.log` con la forma de `poc-f1-3.sh`, ejecutados en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-005\` (borrada al terminar).
4. **No se tocan** F1-1, F1-2 ni F1-3 (se citan y extienden sin reabrirlos) ni el flujo 0.16 de este repo (RR-14).

### Que decide el ADR

- **Donde vive la capa**: modulo interno del CLI con dos puertos. `Git` lanza el binario `git` con entorno fijo (`LC_ALL=C`, `GIT_TERMINAL_PROMPT=0`, `-c color.ui=never`, `--porcelain` o `-q`; nunca lee texto traducido). `Plataforma`: PR, estado de las puertas y merge, por la API HTTP y no por `gh`/`glab`, porque P12/RR-16 dejan `git` como unica dependencia externa (sustituye a `CLI_PLATAFORMA` de `plataforma-remota.js` de la 0.16.1). Su equivalencia en GitLab es de F1-13. Los comandos de fase (`start`, `review`, `finish`, `release`, `hotfix`, `init`, `doctor`) componen operaciones; **el agente ve solo la salida del comando**; las operaciones no son subcomandos publicos (P8), salvo D1.
- **Lista cerrada** (cada operacion con comando dueno e idempotencia; hereda la tabla de scripts de F1-3 y `_gitflow-common.sh`):
  - Locales: `fetch`, `validar-base`, `comprobar-id` (`ls-remote --heads` + `branch --list`, patron `*/task-NNN-*`), `crear-rama-y-worktree`, `reanudar`, `guardar` (commit de rutas explicitas, sin `add -A`, RF-23), `commit-estado` (el punto unico de F1-2, referenciado), `push` (con reintento, sin force), `actualizar-rama` (merge de la base, sin rebase), `quitar-worktree`, `actualizar-principal` (`pull --ff-only`), `localizar-principal`, `tag-anotado`, `notas-release`, `diff-tarea` (la base la calcula el CLI).
  - De plataforma: `abrir-pr` (si existe, devuelve el que hay), `estado-puertas`, `merge-pr` (merge commit, D4a de F1-3).
  - **Prohibidas** para la capa y el agente: `rebase`, `push --force`, `reset --hard`, `stash`, `clean`, merge local en `develop`/`main`, borrar ramas.
- **Contrato de salida**: caso normal, como mucho 5 lineas; primera linea `<comando> <resultado> clave=valor…`, sin ANSI; diagnostico solo en stderr y solo si falla; `--json` con la misma informacion para CI.
- **Codigos de salida**: extiende F1-2 (0 `Publicada`, 3 `Rechazada`, 4 `SinRed`, 5 `Agotada`) con 1 error interno, 2 uso, 6 `Conflicto` (`actualizar-rama`: el agente resuelve ficheros y lanza `guardar`) y 7 `Esperando` (puertas pendientes). Cada codigo con su «que hace quien lo recibe». Propuestos: los fija F2-3.
- **Objetivo de tokens por tarea** (criterio 2; forma segun D3). Base en el frontmatter `tokens_*` de las tareas terminadas:

  | Tarea | Diseno | Implementacion | Revision |
  |---|---|---|---|
  | TASK-002 | 0,23 M | 3,1 M | 1,3 M |
  | TASK-003 | 0,24 M | 7,7 M | 2,0 M |
  | TASK-004 | 0,57 M | 6,3 M | 4,2 M |

  Domina `cache_lectura` (TASK-004: 5,26 M de 5,47 M en implementacion): cada byte que una operacion muestra al agente se relee en cada turno siguiente; de ahi el tope de 5 lineas. La linea base exacta de operaciones Git sale de la PoC (A) (llamadas Bash del agente con `git`/`gh` y bytes de su resultado por tarea; precedente: `coste-transcripcion.js` de la 0.16). El ADR fija las cifras con esa medicion y F8-2 las comprueba.

### Orden

1. `poc-f1-4.sh`: (A) linea base, (B) operaciones crudo frente a porcelana, en bytes y en estabilidad con `LC_ALL`. Guardar el log.
2. ADR: capa y puertos, lista cerrada, contrato, codigos, objetivo con las cifras de (A), tabla de requisitos (RR-01, RF-25, RF-23, RF-24, P5, P12; cada ID se comprueba antes de citarlo).
3. Fila del indice.
4. Borrar la carpeta temporal tras comprobar que no queda nada que conservar.

## Lo que el rol no cubrio

Un solo rol (arquitectura). No revisado:

- **Riesgos** (criterio 1): conflicto a mitad de `actualizar-rama`; PR abierto dos veces a la vez; token de la API caducado; `push` rechazado por reglas de F1-5; salida de `git` que cambia entre versiones; worktree con cambios ajenos al llamar a `guardar`. Van como preguntas abiertas.
- **Dominio**: quien puede lanzar `merge-pr`, `tag` y `release` (F1-6, F1-10); formato del mensaje de commit de `guardar`.
- **Testing**: como prueba F2-2 cada operacion contra repos reales; aqui solo el contrato.
- **Sin verificar por el rol**: que las transcripciones de TASK-002..004 sigan en disco y que en ellas el agente lanzara `git` a mano. Lo comprueba la PoC (A); si no estan, se dice y la linea base se apoya en lo que haya.

## Riesgos aceptados y que los contiene

- **Operaciones no publicas**: si un flujo no previsto necesita Git, el agente no tiene salida; lo contiene `doctor` y la lista se amplia por ADR o extension (P8).
- **API HTTP sin `gh`**: exige gestionar credenciales en el CLI; se acepta por P12/RR-16; identidad y token son de F1-6/F6-1.
- **Codigos 6 y 7 propuestos sin implementar**: los fija F2-3.
- **Linea base medida en tareas de ADR, no de codigo**: puede quedarse corta frente a ToolsaWeb/OpenDockTrading; la recalibran F8-2 y F9-7.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio). Secciones de `poc/poc-f1-4.sh` en Git Bash, remoto bare local, sin GitHub ni gists, carpeta borrada al terminar. Ninguno pesado.

- **Lista cerrada de operaciones → 2**
  1. Cada operacion local con el entorno fijo sobre un remoto bare produce salida porcelana igual con `LC_ALL=C` y `LC_ALL=es_ES`; la segunda ejecucion no cambia nada (mismo HEAD, mismas ramas).
  2. Ciclo `start`→`guardar`→`push`→`actualizar-rama` (con conflicto provocado → `Conflicto`)→`quitar-worktree` sin `rebase`, `force`, `stash` ni `reset`, comprobado con un `grep` del log de comandos.
- **Objetivo de tokens por tarea → 2**
  1. (A) Por tarea TASK-002..004: llamadas Bash con `git`/`gh`, bytes de su resultado y turnos restantes tras cada una.
  2. (B) Bytes de la salida cruda (`push`, `worktree add`, `fetch`, `merge`) frente a la linea de contrato equivalente (≤5 lineas).
- **Cumple F1-4 y los requisitos que cita → 1**
  - Tabla requisito → decision → evidencia; al cerrar: fila del indice, `bash .github/scripts/comprobar-repo.sh` en verde y `taskctl doctor` sin errores.

## Lo que necesita decision de una persona

Decisiones respondidas por la persona (Carlos) el 2026-11-14 con la consigna «responder la opcion recomendada a todas las preguntas»:

- **D1. Superficie publica de las operaciones** → **(a)** solo los comandos de fase, mas `guardar` y `actualizar-rama` como comandos porque ninguna fase los cubre.
- **D2. Commits de codigo del agente** → **(a)** `taskctl guardar "<mensaje>" <rutas…>`, rutas explicitas, sin `add -A` (RF-23).
- **D3. Forma del objetivo de tokens** → **(a)** tres cifras: 0 llamadas `git`/`gh` de escritura o red del agente por tarea; salida visible de operaciones por tarea bajo un tope fijado con la linea base de (A); tokens totales por tarea por debajo de la 0.16 (criterio de salida §9).
- **D4. Lecturas de Git por el agente** → **(a)** el flujo usa `diff-tarea` y `status`; las lecturas crudas no se bloquean pero cuentan en la metrica; un hook bloquea las de escritura y red.
- **D5. Formato de la primera linea** → **(a)** `comando resultado clave=valor`, con `--json` opcional.

Sin troceo: la duracion estimada no pasa de 60 min.

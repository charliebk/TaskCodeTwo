# Plan — TASK-013: F1-12 ADR actualizacion de proyectos entre versiones

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 150 min

Senales: dos dependencias aceptadas (F1-1, F1-9) y F1-11 aporta las entradas medidas. La PoC trabaja sobre una copia real de OpenDockTrading (96 tareas: 33 planificadas, 61 terminadas, 2 archivadas; ninguna en curso), comprueba la convivencia con el plugin 0.16.1 instalado y decide los nombres. Precedente: TASK-012 estimo 170 y tardo 94; este ADR anade migracion y convivencia. **No se trocea** (ver «Motivo para no trocear»).

### Decisiones ya tomadas por la persona (Carlos, 2026-11-14)

1. Fidelidad a los datos sobre el coste; ventajas medibles; comandos deterministas como contrato (ningun paso de la actualizacion depende de un hook).
2. **Nombres propios y definitivos de la version two** (CLI, plugin, prefijo de skills, hooks, status line, ficheros bajo `.git`), distintos de los de la 0.16 y que se quedan asi tras desinstalarla. El ADR elige el nombre (D1) y anota que donde F1-1..F1-11 dicen `taskctl` para la 2.0 se lee el nombre definitivo.
3. **Estudio de adopcion antes de migrar**: `actualizar --simular` inventaria lo hecho a mano, mide el uso real y enumera huecos y parches (el hito en si es de F1-18/TASK-094).
4. **Retirada de parches de la 0.16**: `actualizar` retira los parches de la columna «Clasificacion» de F1-11 solo cuando la 2.0 cubre cada uno; se queda lo propio del proyecto como comando unico con perfiles (F1-7).
5. **Orden y condiciones**: desinstalar la 0.16 despues de migrar y verificar; nunca con una tarea en curso; criterio de salida medible con vuelta atras automatica; opcional y por proyecto; bloqueo de escritura cruzada segun `formato`; `doctor` detecta los dos instalados y dice cual manda.
6. **Primera migracion real: OpenDockTrading**; despues el relevo de TaskCodeTwo (F4-5) y por ultimo ToolsaWeb (dependencias en D3).
7. No trocear.

### Forma central: el plan de migracion es un dato

Una funcion pura `planificar(proyecto) -> PlanMigracion` sirve para tres cosas: `actualizar --simular` la ejecuta y muestra el resultado (JSON y vista legible); `actualizar` aplica exactamente ese plan y antes comprueba que su hash no ha cambiado; idempotencia: con `formato: 2` el plan sale vacio y si la migracion quedo a medias se recalcula desde el estado real.

Cuatro bloques, todos de cosas que ya existen:

1. **Uso medido**: la tabla de campos de F1-9 con las tareas de cada campo fuera del valor por defecto (el contador de su PoC A convertido en informe).
2. **Hecho a mano**: las 14 practicas de F1-11 con su clasificacion parche/propio y fichero:lineas. Un parche entra en «retirar» solo si declara `cubierto_por` con una capacidad que la 2.0 instalada anuncia; si no, se queda y aparece como hueco.
3. **Movimientos**: aplanar `NN-estado/TASK-NNN` a `TASK-NNN` (estado como campo); frontmatter a `formato: 2` segun F1-9 (campos a `roles`, se quitan `revision_codex` y `ultimo_commit_revisado`, `05-archivadas` → `descartada` con `motivo`); config plana a los bloques de F1-11.
4. **Criterio de salida**: comparacion estructural de dos instantaneas `{id -> titulo, estado normalizado, dependencias, sprint, etiquetas}` (carpetas 0.16 frente a rama de estado) y del roadmap por estructura (precedente: `core/roadmap-comparar.js` de la 0.16, TASK-098).

**Historial**: `git subtree split -P tareas` genera commits reescritos sin antepasado comun con `develop` (propiedad «huerfana» de F1-1) y conserva el historial de cada fichero; encima, un commit que aplana con trailer `Migrado-desde: <sha>`. La alternativa (instantanea con trailer) es mas barata pero pierde `git log --follow`; se elige la fiel.

**Transaccion**: todo primero en local (ref de la rama de estado, worktree y una rama `actualizar/...` con el commit que quita `tareas/` de `develop` y lo anade al `.gitignore`; SHA previo apuntado bajo `.git/<nombre-nuevo>/actualizar/`). Se evalua el criterio; si falla, vuelta atras automatica y local (se borran ref y worktree y se vuelve al SHA). Solo con el criterio en verde se hace push; despues, `actualizar --deshacer` revierte por PR (F1-5 protege `develop`) y la rama de estado no se borra (F1-5): queda congelada con un commit que lo dice. Precondicion: ninguna tarea en curso. La desinstalacion de la 0.16 (F9-10) solo con el criterio en verde.

**Convivencia**: por proyecto manda el `formato` de `.taskcode/config.yml` (sin el, la 0.16; con 2, la two). La 2.0 rechaza escribir en un proyecto sin formato 2 salvo `actualizar` y `doctor`. `doctor` detecta los dos instalados (cache de `taskcode-plugin` y `taskctl` en el PATH) y dice cual manda. El lado 0.16 no se puede modificar: lo mide la PoC (D2). Hechos leidos en el plugin 0.16.1: estado en `.git/taskcode/*` (`cadena.lock`, `puertas.log`, `freno-ultimo-verde.json`, `gitflow/`); status line con `taskctl estado --stdin` en el `settings.json` de usuario (una sola ranura); hooks Stop, SubagentStop y PreToolUse en `hooks/hooks.json`; skills con prefijo `taskcode-plugin:`; una clave desconocida en la config solo avisa (`core/config.js:122`), asi que `formato: 2` **no** frena a la 0.16.

**Que se crea**: `docs/adr/F1-12-actualizacion-entre-versiones.md`, su fila en `docs/adr/README.md` y `poc/` (guion y log) en la carpeta de la tarea, ejecutados en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-013\` (borrada al terminar).
**Que no se toca**: los ADR F1-1..F1-11 (la nota de nombres va en «Divergencias» del F1-12), el plan (las dependencias se cambian con taskctl al cerrar), este repo (RR-14), codigo de la 2.0 o de la 0.16, y los pilotos (solo lectura; se trabaja sobre copias de un commit fijo).

**Que decide el ADR**: que migra y con que transformacion; la forma del informe de `--simular` (instrumento del estudio de adopcion); transaccion y vuelta atras; criterio de salida; reglas de convivencia; nombres; propuesta de dependencias (D3).

**Orden**: 1) PoC de nombres y decision (aparecen en todo lo demas); 2) PoC de migracion de la copia de OpenDockTrading (subtree split, aplanado, formato 2, config, criterio, vuelta atras, segunda pasada con plan vacio); 3) PoC de convivencia (`board`, `doctor`, `new` de la 0.16 y sus hooks con stdin sobre la copia migrada); 4) informe de adopcion de OpenDockTrading con los detectores de F1-11; 5) ADR, nota de nombres y propuesta de dependencias.

### Motivo para no trocear

La persona (Carlos) indico al lanzar el H1 el 2026-11-14: «NO troceamos ninguna tarea». Ademas, nombres, migracion y convivencia se deciden con la misma PoC sobre la misma copia.

## Lo que el rol no cubrio

Un solo rol (arquitectura):

- **Riesgos**: modos de fallo de la migracion (corte a medias, push parcial, clon sin la rama de estado, rutas largas en Windows, tareas 0.16 mal formadas, historia reescrita por subtree split frente a firmas). «Vuelta atras» cubierto en forma, no en fallos.
- **Testing**: comprobaciones de la PoC por criterio; la bateria de ida y vuelta es de F2-5, F9-1 y F9-2.
- **Dominio**: si `descartada` con `motivo` basta para archivadas sin `## Descartada` (motivo a pedir) y si «hito» es lo mismo que `sprint`.
- **Rendimiento**: solo se mide el subtree split en OpenDockTrading; ToolsaWeb (404 tareas) es de F9-2.

## Riesgos aceptados y que los contiene

- **La 0.16 escribe en un proyecto ya migrado** (puede crear `tareas/00-planificadas/` dentro del worktree de la rama de estado): lo contienen la medicion de la PoC y D2; mientras no se resuelva, `doctor` de la two lo detecta como estructura mixta y el criterio de salida falla.
- **Hooks de la 0.16 activos en un proyecto migrado** hasta desinstalarla: la PoC ejecuta `pre-tool-merge.js` y `stop-verificacion.js` con stdin sobre la copia; si no son inertes, el ADR exige desactivar el plugin en el proyecto antes de la primera tarea con la 2.0.
- **La historia reescrita cambia los SHA del historial de `tareas/`**: trailer con el SHA de origen en el commit de aplanado; `develop` conserva la historia original intacta.
- **Nombre decidido con datos de una sola maquina**: comprobacion del PATH en Windows y en Linux (WSL o runner, si esta disponible; si no, «no verificado») y consulta de solo lectura de npm, winget y Homebrew (P12 preve esos paquetes).

## Plan de pruebas

El guion de `poc/` falla si falta una cifra medida o si una no coincide con lo medido (precedente F1-11). `python -I -B`, log determinista.

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio). Ninguno pesado salvo el subtree split, que se mide.

- **Que migra, simulacion, vuelta atras, convivencia 0.16/2.0 → 3**
  1. Copia de OpenDockTrading migrada: mismas 96 tareas, IDs y dependencias; 2/2 archivadas → `descartada`; todos los frontmatter con `formato: 2` se leen con YAML estandar; `merge-base develop <rama>` sin salida; para cada tarea, `git log --follow` en la rama de estado tiene al menos tantos commits como su ruta en `develop`.
  2. Criterio roto a proposito (dependencia borrada en la copia): la vuelta atras deja la copia en el SHA previo, sin ref ni worktree y `git status` limpio; segunda pasada sobre la copia migrada da plan vacio; `--simular` no cambia ningun byte (hash del arbol antes y despues).
  3. Convivencia: salida y codigo de `taskctl board`, `doctor` y `new` de la 0.16 y de sus hooks sobre la copia migrada; `where`/`command -v` de `taskctl` y del nombre nuevo dan rutas distintas; `.git/taskcode/` y `.git/<nuevo>/` no comparten ningun fichero.
- **Cumple F1-12 y los requisitos que cita → 2**
  1. Tabla requisito → decision → evidencia: RR-13 (simulacion, idempotencia, reversibilidad), RR-16/F9-10 (orden de desinstalacion), F1-1, F1-9 (tabla de campos aplicada), F1-11 (parches enumerados con fichero y lineas en el informe de OpenDockTrading; 91 lineas que se retiran frente a 973 que se quedan, a recalcular); cada ID comprobado antes de citarlo.
  2. Fila del indice con enlace que resuelve; «Divergencias» nombra cada `taskctl` de F1-1..F1-11 que pasa a leerse con el nombre definitivo y la rama de F1-1 renombrada (D1). Al cerrar, `bash .github/scripts/comprobar-repo.sh` en verde y `taskctl doctor` sin errores.

## Lo que necesita decision de una persona

Decisiones respondidas por la persona (Carlos) el 2026-11-14 con la consigna «responder la opcion recomendada a todas las preguntas» (ademas de las siete ya tomadas de arriba):

- **D1. Nombre definitivo** → **(a)** `tc2` como CLI; plugin y prefijo de skills `taskcodetwo`; `.git/taskcodetwo/`; rama de estado `taskcodetwo/tareas` (se renombra la `taskcode/tareas` de F1-1 por coherencia). Descartadas (b) `tctwo`, (c) `taskcodetwo` como CLI (largo) y (d) `tc` (choca con `tc` de iproute2 en Linux; la PoC lo confirma o lo marca «no verificado»). Eleccion de quien orquesta con la consigna: queda senalada en el Resultado para que Carlos la confirme o la cambie.
- **D2. Bloqueo del lado 0.16** → **(a)** usar un fallo cerrado que la 0.16 ya tenga y que la PoC mida (p. ej. que su parser plano aborte con bloques anidados). Si la PoC no lo encuentra, el ADR **no elige solo**: deja la decision abierta para Carlos entre (b) publicar una 0.16.2 en el repo TaskCode que respete `formato` (trabajo fuera de este repo) y (c) solo avisar con `doctor` de la two.
- **D3. Dependencias de la decision 6** → **(a)** quitar F9-6 de F9-7; F4-5 depende de F9-7 y F9-8 depende de F4-5 (orden OpenDockTrading → TaskCodeTwo → ToolsaWeb); se aplicara con taskctl al cerrar.
- **D4. Ruta de la config de la 2.0** → **(a)** la misma `.taskcode/config.yml` con `formato: 2` (el bloqueo se ata al `formato` de una unica config).

# Brainstorm — TASK-138, rol riesgos (ronda 1)

- Rol: `brainstorm-riesgos`
- Agente: brainstorm-riesgos (TaskCode 0.17.0), solo lectura; ninguna orden de cargo ejecutada salvo `cargo mutants --version`

## Modos de fallo, ordenados por gravedad, con el escenario concreto de cada uno

- **Mutante ejecutado contra GitHub/GitLab reales** — un auditor muta `plataforma/github.rs` y lanza `plataforma_e2e`/`plataforma_gitlab_e2e` (18 tests `--ignored` que leen `TC2_E2E_TOKEN`/`TC2_E2E_APP_CLAVE`, visto en `tests/plataforma_e2e.rs:7-36,61-62`) con las identidades reales de la memoria — PR, aprobaciones o ramas reales creadas por código roto, no deshacibles por git — los E2E quedan fuera de la mutación y se marcan «no verificable por mutación» en la tabla de cobertura.
- **La auditoría como Disfraz** — `cargo mutants` no está instalado (verificado: `error: no such command: mutants`); un auditor afirma «el mutante sobrevive» sin ejecutarlo, o el mutante no compila / el test está `#[ignore]` y «sobrevive» por no correr — con la regla 0.17 (Disfraz real ≥ IMPORTANTE, `skills/task-workflow/revision.md:93`) cada falso caso abre tarea — cada D4 lleva commit base, diff del mutante, orden exacta y salida pegada; el revisor de la tarea reejecuta una muestra.
- **Mutación en el checkout principal + freno Stop** — el mutante se aplica sobre `develop` y el proceso muere antes de revertir; o el hook Stop lanza `cargo xtask verificar` sobre el mismo `target/` mientras el auditor compila (LNK1104 ya visto en TASK-118 y TASK-127, memoria `no-solapar-verificaciones`) — código de producción mutado sin commit, o suite cortada — mutar solo en clon bajo `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-138-*` con `CARGO_TARGET_DIR` propio; `git status` limpio del principal al cerrar.
- **Disco y tiempo por targets en paralelo** — `target/` pesa 15 GB (verificado con `du`); N auditores con clon y target propio = N×15 GB y N compilaciones completas a la vez de ~96 k líneas — disco lleno a mitad, cortes por tiempo, resultados parciales — paralelo solo para D1-D3 (lectura); D4 serializado en un único clon con un único target.
- **Falsos positivos de Duplicación** — `plataforma/github.rs` (1195 l.) y `gitlab.rs` (1385 l.) implementan el mismo `trait Plataforma` (`plataforma/mod.rs:413`) por ADR F1-13, y los E2E van en pareja — tareas de «unificar» que rompen la frontera puerto/adaptador decidida — un caso D1 solo cuenta si la lógica repetida no depende de la API; los descartados se listan con el ADR que los justifica.
- **Falsos negativos entre módulos** — un auditor por módulo no ve `src/inventario.rs` (1602 l.) frente a `src/init/inventario.rs` (905 l.), ni `xtask/src/workflows.rs` (3150 l.) frente a `.github/workflows/*.yml` (supuesto: no he comprobado si una genera la otra) — tabla de cobertura «0 casos» que miente — un pase transversal determinista (similitud de bloques, sin API de pago) antes de repartir por módulo.
- **Desviación y diluvio de tareas** — el auditor «arregla de paso», o abre una tarea por caso; ya hay 46 en `tareas/00-planificadas` (verificado) y una cadena H4 en curso — la auditoría incumple su propio objetivo y el tablero se inunda de tareas de 10 líneas — diff de la tarea limitado a `docs/auditoria-4d.md` y a tareas nuevas; una tarea por módulo con todos sus IMPORTANTES; los MENOR solo en el documento.

## Estados intermedios y fallos parciales

- Muere tras crear 5 de 12 tareas de seguimiento: IDs consumidos, documento sin el ID de cada caso. Sin idempotencia, la reejecución duplica tareas. Mitigación: cada caso guarda el ID de su tarea; antes de crear una, se busca por módulo + regla.
- `fichero:linea` caduca en cuanto otra tarea toca el código: el documento se ancla a un commit (`Commit auditado: <sha>`), y la línea se lee siempre con ese commit.
- Clones de 15 GB abandonados en `pruebas-claude` si el proceso muere (CLAUDE.md exige borrarlos). El cierre comprueba que no quedan carpetas `task-138-*`.
- D2 exige cruzar el código con 19 ADR de 26-71 KB y con los criterios de las tareas terminadas. Si el contexto se agota a mitad, unos módulos acaban con D2 revisado y otros no. La tabla lleva una columna por D para que el hueco se vea.

## Compatibilidad hacia atras

- El documento es nuevo y no hay consumidores. Las tareas nuevas, en cambio, entran en ROADMAP, BOARD y grafo de dependencias, y compiten con la cadena H4 y con TASK-040, que espera al piloto: crearlas con `taskctl import` desde ficheros fuera del repo (CLAUDE.md, punto 5) y sin dependencias que reordenen lo ya aprobado.
- `docs/auditoria/` ya existe (`AUDITORIA-EXTERNA-2026-11-13.md`). El criterio pide `docs/auditoria-4d.md` en la raíz de `docs/`. Hay que decidir la ubicación sin mover lo existente.

## Vuelta atras

- El documento se deshace con `git revert`: reversible entero.
- Las tareas creadas se cancelan en vez de borrarse. Supuesto no verificado: taskctl no reutiliza los IDs, así que crearlas es un punto sin retorno parcial.
- Instalar `cargo-mutants` (descarga de crates.io, gratuita) cambia `~/.cargo/bin`. Se deshace con `cargo uninstall`, pero hay que pedir permiso antes de instalarlo. La alternativa es la mutación manual en el clon.
- Una acción E2E sobre GitHub o GitLab real **no se deshace** con git. Por eso los E2E van excluidos de la mutación (ver el primer riesgo).
- Desacuerdo previsto con arquitectura: si propone un agente por módulo que muta y compila cada uno por su lado, mi posición es paralelo solo para leer, y mutación serializada en un único target.

## El riesgo que mas te preocupa (UNO solo)

- Que la auditoría sea ella misma un Disfraz. Mutantes que «sobreviven» porque el test no se ejecutó (`#[ignore]`, el mutante no compila, se filtró mal el `--test`) se convierten en casos D4 IMPORTANTES, cada uno con su tarea, sin evidencia que nadie pueda reproducir. Así la 4D acaba fabricando el mismo defecto que debía detectar. Línea roja: un D4 sin orden reejecutable, salida pegada y commit base no entra en el documento.

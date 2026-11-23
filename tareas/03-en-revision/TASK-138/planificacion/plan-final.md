# Plan — TASK-138: Auditoria 4D de todo el codigo de TaskCodeTwo

(Lo consolida el agente unificador a partir de 2 roles de brainstorm
lanzados en paralelo: arquitectura, riesgos.
Los desacuerdos entre roles se senalan, no se promedian.)

## Enfoque propuesto

Complejidad: declarada **media**, heurística **trivial** (0 señales). Se lanzaron 2 roles (el mayor de los dos). La heurística se queda corta: el alcance son ~95 k líneas (`src` ≈ 62 k, tests 28 375, xtask + distribución 4 626; medido por arquitectura con `wc -l`) y el enunciado no trae ninguna señal que lo refleje.

La auditoría solo diagnostica. El diff de la tarea es `docs/auditoria-4d.md` más las tareas nuevas en `tareas/00-planificadas`; no se corrige código (riesgos; decisión delegada).

1. **Commit auditado** (riesgos): el documento empieza con `Commit auditado: <sha>` y todo `fichero:línea` se lee contra ese sha, porque las líneas caducan en cuanto otra tarea toca el código.
2. **Pase 0 transversal y determinista** (arquitectura): lo hace un solo agente, sin descargas. Usa `ast-grep` 0.44.1 (instalado), que extrae cada `fn`; un script propio normaliza y hashea los cuerpos y agrupa los iguales entre módulos. Suma grep de nombres repetidos, `cargo clippy --all-targets` con `cognitive_complexity`, `too_many_lines`, `too_many_arguments`, `type_complexity`, `redundant_clone` y `needless_pass_by_value` (con `CARGO_TARGET_DIR` propio), y reglas `sg` de Disfraz: `#[test]` sin `assert*`, `let _ =` en tests, aserciones que solo miran `is_ok()`/`is_some()`, `#[ignore]` y `Ok(())` de una línea. Los candidatos y los scripts van a `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-138-auditoria\`, fuera del repo. Meter tooling en `xtask/` sería Desviación (arquitectura).
3. **8 auditores en paralelo, cada uno dueño exclusivo de unas rutas** (arquitectura). El reparto A1-A8 es el de la salida de arquitectura, y cada test va con el módulo que prueba. A8 es transversal: `lib.rs`, `main.rs`, los ficheros sueltos de `src/`, `xtask/`, `crates/distribucion/`, los 4 workflows y `tests/comun/`. Además es dueño de toda Duplicación entre dueños: le llegan las pistas de los otros siete en `pistas-entre-modulos.txt` y hace una pasada por conceptos (escritura atómica, frontmatter/YAML, rutas de tarea, dobles de `Plataforma`, invocación de `git` y fechas, sacados de F1-2 y F1-4). D1, D2 y D3 se buscan leyendo y con las herramientas del pase 0, **sin compilar en paralelo**. D2 se mide contra la tabla `módulo → ADR → TASK` sin reabrir ningún ADR (arquitectura).
4. **Mutación dirigida, manual y serializada** (riesgos gana; ver desacuerdos). Solo se muta en los candidatos D4 que los auditores confirman leyendo. Se hace en **un único clon** con un único `CARGO_TARGET_DIR` bajo `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-138-mutantes\`, nunca en el checkout principal. Los E2E `#[ignore]` con tokens reales (`plataforma_e2e`, `plataforma_gitlab_e2e`) quedan fuera: en la cobertura salen como «no verificable por mutación» (riesgos).
5. **Formato y fusión**: hay un fichero por hallazgo, `hallazgos/A<n>-<nn>.md`, con los campos de arquitectura (`id`, `regla`, `ubicacion`, `modulo`, `severidad`, `motivo_severidad`, `evidencia`, `grupo_seguimiento`). Un script de A8 los concatena en `docs/auditoria-4d.md` con la tabla `ID | Severidad | Regla | fichero:línea | Resumen | Evidencia | Verificado`, que es la de `docs/auditoria/AUDITORIA-EXTERNA-2026-11-13.md` con dos columnas más. Después genera la tabla de cobertura por módulo, con una columna por D (riesgos) y 0 explícito donde no hay casos (arquitectura). Los descartes justificados por un ADR se citan en su propia sección: p. ej. `trait Plataforma` con `github.rs`/`gitlab.rs`, por F1-13 (riesgos).
6. **Seguimiento**: una tarea por grupo/dueño A1-A7 con todos sus IMPORTANTE y CRITICO. Las D1 entre módulos van agrupadas por la abstracción común que hay que extraer (arquitectura). Los MENOR solo constan en el documento (riesgos). Las tareas se crean con `taskctl import` desde ficheros en `task-138-import\`, fuera del repo, y sin dependencias que reordenen la cadena H4 ni TASK-040 (riesgos). Cada caso guarda el ID de su tarea, y antes de crear una se busca por grupo + regla, para que reejecutar no duplique (riesgos).
7. **Cierre**: `git status` limpio en el checkout principal y ninguna carpeta `task-138-*` en `pruebas-claude`. Se borran tras comprobar que no queda nada que conservar (riesgos; CLAUDE.md global).

Duracion estimada: 300 min

Señales: 0 ficheros de código, 1 documento nuevo, entre 8 y 12 tareas de seguimiento previstas (sin medir), ~95 k líneas leídas por 8 auditores, mutación serial sobre un workspace con `target/` de 15 GB (riesgos, `du`), 4 criterios. No hay precedente de auditoría completa con `taskctl` en este repo; la cifra es una suposición del unificador.

### Evidencia por regla (sin Disfraz en la propia auditoría)

- D1: los dos fragmentos con `fichero:línea`, de 10 líneas como máximo cada uno, y su diff normalizado diciendo en qué difieren. Por debajo de 10 líneas o 50 tokens no hay D1 (arquitectura). Solo cuenta lógica que no dependa de la API del adaptador (riesgos).
- D2: la cita del ADR o del criterio junto a la línea de código que se aparta (arquitectura).
- D3: la versión reducida (5 líneas como máximo) con las líneas de antes y de después. Un aviso de clippy por sí solo no es hallazgo (arquitectura).
- D4 en código: la entrada con la que el código no hace el trabajo (arquitectura).
- D4 en tests: commit base, diff del mutante, orden exacta `cargo test -p tc2 --test <x> <nombre>` y su salida pegada en verde con el mutante puesto. Además, el mismo test en rojo con el código roto de verdad, para probar que el test corre: no vale un test `#[ignore]`, un mutante que no compila ni un `--test` mal filtrado (arquitectura y riesgos). Sin estas piezas, el caso no entra (decisión delegada).

## Troceo

Pasa de 60 min (300). **La persona decidió no trocear** (decisión de Carlos, 2026-11-22). Contiene el riesgo el orden de pasos: el pase 0 y la lectura (D1-D3) quedan completos antes de mutar, y las tareas de seguimiento se crean al final, cuando el documento está cerrado. Un corte a medias deja candidatos fuera del repo y ningún cambio en él.

## Desacuerdos entre roles, y como se resuelven

- **Dónde y cuántos se compila a la vez**. Arquitectura: un `git worktree` por auditor (`wt-A<n>`), cada uno con su `CARGO_TARGET_DIR`, y como mucho dos `cargo test` de mutación a la vez con un cerrojo de fichero. Riesgos: paralelo solo para leer, y D4 serializado en un único clon con un único target, porque N targets son N×15 GB, N compilaciones de ~96 k líneas y, con el freno Stop, LNK1104 (TASK-118, TASK-127). **Gana riesgos** (opción recomendada, delegada por Carlos, 2026-11-22): el coste en disco está medido (15 GB) y el solape ya rompió verificaciones antes. El cerrojo de arquitectura mitiga el solape, pero no el disco.
- **Herramientas descargadas**. Arquitectura: `npx jscpd` y `cargo install cargo-mutants`, pidiendo permiso, y si no, `sg`, grep y clippy. Riesgos: instalar `cargo-mutants` exige permiso, y la alternativa es la mutación manual en el clon. **No se descarga nada** (opción recomendada, delegada por Carlos, 2026-11-22): queda la rama sin descargas que ambos ya aceptaban como alternativa.
- **Alcance de la mutación**. Arquitectura había previsto que riesgos pediría `cargo mutants` sobre todo el workspace, pero riesgos no lo propuso. No hay desacuerdo real: los dos defienden mutación dirigida. Queda dicho para que nadie lo lea como un acuerdo forzado.
- **Enlace desde `docs/auditoria/`**. Arquitectura: el documento va en `docs/auditoria-4d.md`, como fija el criterio, y se enlaza desde `docs/auditoria/`. Riesgos: hay que decidir la ubicación sin mover lo que ya existe. **Gana `docs/auditoria-4d.md` sin enlace**: enlazarlo tocaría un fichero más, y el diff está limitado al documento y a las tareas nuevas (opción recomendada, delegada por Carlos, 2026-11-22).
- **Ruta temporal**. Arquitectura: `pruebas-claude\taskcodetwo\task-138\` con subcarpetas. Riesgos: `pruebas-claude\TaskCodeTwo\task-138-*`. **Gana `taskcodetwo\task-138-*`** (decisión delegada): una carpeta por motivo, que el cierre comprueba por patrón.

## Riesgos aceptados y que los contiene

- Mutante ejecutado contra GitHub o GitLab reales: crea PR, aprobaciones o ramas que git no deshace (riesgos). Lo contiene excluir los E2E de la mutación y marcarlos «no verificable por mutación».
- La auditoría como Disfraz: mutantes que «sobreviven» sin haberse ejecutado y acaban en casos D4 IMPORTANTES con su tarea (riesgos, el que más le preocupa). Lo contienen la línea roja de evidencia (commit base, diff, orden y salida), el contraste en rojo con el código roto y que el revisor por pares reejecute una muestra.
- Falsos negativos de Duplicación entre módulos: un clon semántico (`match` en un sitio, `if let` en otro) escapa al hash normalizado (arquitectura, la decisión que más le preocupa). Riesgos añade dos pares concretos: `src/inventario.rs` frente a `src/init/inventario.rs`, y `xtask/src/workflows.rs` frente a `.github/workflows/*.yml`. Lo contienen las pistas obligatorias de los siete auditores y la pasada por conceptos de A8. Queda un residual que se acepta, y se anota en el documento.
- Falsos positivos de Duplicación entre `github.rs` y `gitlab.rs`, que implementan el mismo `trait Plataforma` por F1-13 (riesgos). Lo contienen que solo cuente la lógica que no depende de la API y que los descartes se listen con su ADR.
- Diluvio de tareas sobre las 46 ya planificadas y la cadena H4 (riesgos). Lo contienen una tarea por grupo, los MENOR solo en el documento y que las tareas entren sin dependencias que reordenen lo aprobado.
- D2 a medias si se agota el contexto, con 19 ADR de 26-71 KB (riesgos). Lo contiene la columna por D en la tabla de cobertura, que deja el hueco a la vista en lugar de un «0» que mienta.
- La agrupación de las D1 entre módulos por abstracción (arquitectura) no es literalmente «agrupadas por módulo», como dice el criterio 3 (lo señala el unificador). Lo contiene que cada una de esas tareas liste los módulos que toca y que la tabla de cobertura remita a ella desde cada módulo.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

No hay tests de código: la tarea no escribe código. Aquí las «pruebas» son la reproducibilidad de la evidencia y la cobertura. Las comprueba el revisor por pares reejecutando, nunca leyendo el informe.

Criterio «docs/auditoria-4d.md lista cada caso con regla, fichero:línea, evidencia y severidad» (2 comprobaciones):
1. Un script recorre el documento y verifica que todo caso tiene los cuatro campos y que todo `fichero:línea` existe en el `Commit auditado` (`git show <sha>:<fichero>` con esa línea dentro del rango). Contra el propio documento y el repo (arquitectura y riesgos).
2. El revisor reabre una muestra de D1 y D2 (como mínimo una por auditor) y comprueba que los dos fragmentos son los citados en ese sha y que la cita del ADR existe (riesgos).

Criterio «cubre todos los módulos; tabla de cobertura por módulo con el número de casos» (2 comprobaciones):
1. Se cruza la lista de módulos de `src/`, de los ficheros de `tests/`, de `xtask/`, de `distribucion/` y de los 4 workflows (listados con `git ls-tree` en el sha) con las filas de la tabla: no puede faltar ninguna fila, y la que no tiene casos lleva 0 explícito (arquitectura).
2. Las filas de la tabla suman exactamente el número de casos del documento, y los E2E salen como «no verificable por mutación» (riesgos).

Criterio «cada IMPORTANTE o CRITICO tiene su tarea en tareas/00-planificadas, agrupadas por módulo» (2 comprobaciones):
1. Cada caso IMPORTANTE o CRITICO lleva un `TASK-NNN` que existe en `tareas/00-planificadas`, y cada una de esas tareas nombra sus casos (riesgos).
2. Hay una sola tarea por grupo/dueño y por abstracción D1, y ninguna duplicada al reejecutar la importación: se busca por grupo + regla (riesgos).

Criterio «los Disfraz en tests se prueban con un mutante que sobrevive, ejecutado de verdad» (2 comprobaciones):
1. El revisor reejecuta en un clon propio bajo `pruebas-claude` al menos dos D4 de test, con su diff y su orden, y obtiene la misma salida en verde con el mutante. Luego comprueba el rojo con el código roto de verdad (riesgos, línea roja).
2. Ningún D4 cita un test `#[ignore]` ni un mutante sin compilar. Se comprueba sobre la salida pegada: la salida tiene que traer `test result: ok. 1 passed` del test nombrado (riesgos).

Pesado: la mutación serial y la reejecución del revisor, cada una con un `target` propio de unos 15 GB. Nunca en paralelo con `cargo xtask verificar` sobre el mismo target (memoria «no solapar verificaciones»).

## Lo que el plan no cubre

- No se lanzó el rol de testing. Por eso nadie revisó si las comprobaciones de arriba bastan para sostener la cobertura, ni si la reejecución por muestra es suficiente. El plan de pruebas lo armó el unificador con lo que pidieron arquitectura y riesgos.
- No se lanzó el rol de dominio. Nadie juzgó si la lectura de la regla 4D (umbral de 10 líneas o 50 tokens para D1, CRITICO por datos o seguridad) es la que haría Carlos. Solo él puede juzgarlo.
- Arquitectura propone un CRITICO extra cuando las copias D1 ya divergen; resuelto en la decision 5 (IMPORTANTE con marca `divergente`).
- Ningún rol dijo cómo se audita D2 en `.github/workflows` ni en `crates/distribucion` frente a sus ADR. A8 lo cubre con la misma tabla `módulo → ADR → TASK`, sin método propio.

## Suposiciones no verificadas

- Que el reparto A1-A8 queda equilibrado, con unas 9-11 k líneas por auditor. Lo supone arquitectura, que no midió los tests módulo a módulo; habría que medirlos con `wc -l` por grupo antes de lanzar.
- Que `xtask/src/workflows.rs` y `.github/workflows/*.yml` son una D1 entre ficheros. Riesgos no comprobó si uno genera el otro; si lo genera, no hay D1. Habría que leer `xtask/src/workflows.rs`.
- Que `taskctl` no reutiliza los IDs de las tareas canceladas, y que por eso crearlas es parcialmente irreversible (riesgos). Habría que verlo en `taskctl import` y en su código de numeración.
- Que caben en disco un clon con un target de 15 GB para la mutación y otro para el revisor. Nadie midió el espacio libre: hay que mirarlo antes de clonar.
- Que el hash normalizado con `ast-grep` saca los mismos grupos que habría sacado `jscpd`. Sin `jscpd` no se puede contrastar, y se acepta la pérdida.

## Lo que necesita decision de una persona

Todas decididas con la opción recomendada, delegada por Carlos, 2026-11-22; sujetas a `taskctl approve`.

1. No se descarga ni instala nada (ni `cargo-mutants` ni `jscpd`). Quedan el pase 0 con `ast-grep`, clippy con lints de complejidad y scripts propios. La mutación se dirige a los candidatos D4, es manual y va serializada en un único clon con `CARGO_TARGET_DIR` propio bajo `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-138-*`, nunca en el checkout principal. Los E2E `#[ignore]` con tokens reales quedan fuera de mutación (opción recomendada, delegada por Carlos, 2026-11-22).
2. Habrá 8 auditores en paralelo, dueños exclusivos de sus rutas, con A8 transversal: Duplicación entre módulos, con las pistas de los otros siete y una pasada por conceptos. D1-D3 se buscan leyendo y con herramientas, sin compilar en paralelo (opción recomendada, delegada por Carlos, 2026-11-22).
3. Ningún caso entra sin evidencia reproducible. En D1, los dos fragmentos con `fichero:línea`; en D4, commit base, diff del mutante, orden y salida. Los descartes que justifica un ADR se citan; p. ej. el `trait Plataforma`, por F1-13 (opción recomendada, delegada por Carlos, 2026-11-22).
4. Severidad: D1 y D4 reales son IMPORTANTE como mínimo, y CRITICO si tapan un fallo de datos o de seguridad; D2 y D3 son MENOR salvo que cambien el comportamiento. Se abre una tarea por grupo/dueño, y las D1 entre módulos se agrupan por la abstracción común. Los MENOR solo constan en el documento. El diff es el documento más las tareas nuevas (opción recomendada, delegada por Carlos, 2026-11-22).
5. Una D1 cuyas copias ya divergen en comportamiento NO sube a CRITICO por eso: queda IMPORTANTE con la marca `divergente` (su tarea va primera en su grupo); CRITICO solo si tapa un fallo de datos o de seguridad, como en la decision 4 y en la clasificacion de la guia de revision (opcion recomendada, delegada por Carlos, 2026-11-22).
6. Sin trocear (decisión de Carlos, 2026-11-22).

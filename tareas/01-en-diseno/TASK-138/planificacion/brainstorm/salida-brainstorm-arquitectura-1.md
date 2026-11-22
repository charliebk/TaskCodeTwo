# Brainstorm — TASK-138, rol arquitectura (ronda 1)

- Rol: `brainstorm-arquitectura`
- Agente: taskcode-plugin:brainstorm-arquitectura (Opus 5.5), 2026-11-22

## Enfoque propuesto, con rutas y nombres concretos

Dos fases. **Pase 0 transversal y determinista** (un solo agente, antes del reparto) genera los candidatos de todo el repo en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-138\candidatos\`; después **8 auditores en paralelo, cada uno dueño exclusivo de unas rutas**, y uno de ellos (A8) dueño además de toda Duplicación cuyos dos extremos caen en dueños distintos. Un candidato no es hallazgo hasta que su dueño lo reproduce.

- Pase 0, herramientas (todas locales, sin API de pago):
  - Clones por tokens: `npx jscpd --format rust --min-tokens 50 --reporters json crates xtask` (npx descarga el paquete: pedir permiso; si no, sigue sin él).
  - Funciones casi iguales: `ast-grep` 0.44.1 (instalado, `sg`) extrae cada `fn` con su cuerpo; un script normaliza identificadores y literales, hashea y agrupa los cuerpos iguales entre módulos.
  - Nombres repetidos: `grep -rhoE "fn [a-z_]+"` agrupado. Ya da pistas reales: tres escrituras atómicas (`confianza/mod.rs:174`, `desinstalar/mod.rs:571`, `huella.rs:105`) y cinco dobles de `Plataforma` (`identidad.rs:666`, `revision/mod.rs:556`, `tablero.rs:1013`, `tests/revision.rs:118`, `tests/tablero.rs:134`) cuando existe `tests/comun/`.
  - Diluvio: `cargo clippy --all-targets -- -W clippy::cognitive_complexity -W clippy::too_many_lines -W clippy::too_many_arguments -W clippy::type_complexity -W clippy::redundant_clone -W clippy::needless_pass_by_value`, con `CARGO_TARGET_DIR` propio (el workspace ya tiene `clippy::all`, `xtask/src/main.rs:295`).
  - Disfraz: reglas `sg` para `#[test]` sin `assert*`, `let _ =` en tests, aserciones solo con `is_ok()` o `is_some()`, `#[ignore]` y `Ok(())` en cuerpos de una línea.
  - Mutación: `cargo-mutants` **no está instalado** (comprobado). `cargo install cargo-mutants` es una descarga: pedir permiso. Sin él, mutantes a mano. En los dos casos, mutación dirigida solo a los candidatos D4 y a las funciones que cubren, nunca a todo el repo.
- Reparto (líneas medidas con `wc -l`; `src` ≈ 62 k, tests 28 375, xtask + distribucion 4 626):
  - A1 `plataforma/` + `identidad.rs` + tests `plataforma*`, `identidad*`.
  - A2 `hallazgos/` + `revision/` + tests `hallazgos`, `revision`, `revisoras`.
  - A3 `plan/` + `importar/` + `tarea/` + `alta.rs` + tests `plan`, `importar`, `tarea`, `aprobar`.
  - A4 `git/` + `estado/` + `toma/` + tests `git_*`, `estado_*`, `toma`, `liberar`, `agente_git/`.
  - A5 `seleccion/` + `inventario.rs` + `recursos.rs` + `doctor/` + sus tests.
  - A6 `estimacion/` + `progreso/` + `roadmap/` + `tablero.rs` + `grafo.rs` + `solape.rs` + `siguiente.rs` + `situacion.rs` + sus tests.
  - A7 `config/` + `init/` + `desinstalar/` + `i18n/` + `confianza/` + `ciclo/` + `verificacion/` + sus tests.
  - A8 transversal: `lib.rs`, `main.rs`, `actualizar_cli.rs`, `descarga.rs`, `instalacion.rs`, `huella.rs`, `version.rs`, `contrato.rs`, `dato.rs`, `carril.rs`, `yaml.rs`, `xtask/`, `crates/distribucion/`, los 4 workflows (los pasos repetidos entre `revision-ia.yml` y `verificacion.yml` son D1 entre ficheros), `tests/comun/` y **todos los grupos D1 entre dueños** que salgan del pase 0.
- Regla anti-solape: cada `fichero:línea` tiene un único dueño, el de su ruta. Si un auditor ve una duplicación con código de otro dueño, la apunta como pista en `candidatos/pistas-entre-modulos.txt`, no como hallazgo, y A8 la cierra.
- Cada test va con el módulo que prueba, para que el D4 de un test se juzgue junto al código que dice comprobar.
- Desviación: cada auditor recibe la tabla `módulo → ADR → TASK`, sacada de `docs/adr/README.md` y de `git log --format=%s -- <ruta>`. Cita la línea del ADR o del criterio y la línea de código que se aparta.
- Formato común: un fichero por hallazgo, `hallazgos/A<n>-<nn>.md`, con estos campos:
  - `id`, `regla` (D1–D4), `ubicacion` (fichero:línea; en D1 las dos), `modulo`, `severidad`, `motivo_severidad`;
  - `evidencia`: comando exacto, salida literal y fragmentos de 10 líneas como máximo;
  - `grupo_seguimiento`.
- Un script de A8 concatena los ficheros en `docs/auditoria-4d.md` y genera la tabla de cobertura por módulo. Un módulo sin casos sale con 0 explícito, nunca como fila ausente.
- Evidencia sin Disfraz, por regla:
  - D1: los dos fragmentos y su diff normalizado, diciendo en qué difieren.
  - D2: la cita del ADR o del criterio junto a la línea de código.
  - D3: la versión reducida (5 líneas como máximo) y las líneas de antes y de después. Un aviso de clippy por sí solo no es hallazgo.
  - D4 en código: la entrada con la que no hace el trabajo.
  - D4 en tests: el diff del mutante, `cargo test -p tc2 --test <x> <nombre>` en verde con el mutante puesto, y el test en rojo cuando el código está bien roto.
  - Los mutantes se aplican en un `git worktree` bajo `task-138\wt-A<n>\`, nunca en el árbol del repo.
- Umbrales:
  - D1 y D4 reales: IMPORTANTE como mínimo (`revision.md` §Regla 4D).
  - CRITICO si el Disfraz tapa un fallo en datos, seguridad o identidad (estado, atómicas, `identidad`, `verificacion`), o si las copias D1 ya divergen en su comportamiento.
  - D2 y D3: MENOR salvo que cambien el comportamiento o escondan un fallo.
  - Por debajo de 10 líneas o 50 tokens equivalentes no hay D1.
- Seguimiento:
  - Una tarea por grupo de dueño (A1–A7), con todos sus IMPORTANTE y CRITICO.
  - Las D1 entre módulos se agrupan por la abstracción común que hay que extraer (p. ej. «una sola escritura atómica», «doble de `Plataforma` en `tests/comun`»), no por módulo, para que la extracción no se haga dos veces.
  - Se importan con `taskctl import` desde `task-138\import\`, fuera del repo (CLAUDE.md, regla 5).

## Que se extiende y que se crea

- `docs/auditoria/` — se extiende. El precedente es `AUDITORIA-EXTERNA-2026-11-13.md`, con la tabla `ID | Severidad | Área | Resumen | Evidencia | Verificado`. Se reutiliza esa tabla y se le añaden las columnas `Regla` y `fichero:línea`. El criterio fija `docs/auditoria-4d.md`: se respeta y se enlaza desde `docs/auditoria/`.
- `tests/comun/` (`mod.rs`, `ops.rs`, `http_mock.rs`) — no se toca. Es el destino natural de los dobles duplicados, pero eso queda para el seguimiento.
- Se crean scripts del pase 0 (normalizador `sg` y fusionador de hallazgos). Viven en el directorio temporal, no en el repo: la tarea solo diagnostica, y meter tooling nuevo en `xtask/` sería Desviación.
- Se crea `docs/auditoria-4d.md`, el único fichero del repo que se escribe, más las tareas en `tareas/00-planificadas` vía `taskctl`.

## Limites que cruza

- Ningún contrato público ni formato de fichero: solo lectura del código, más un documento y tareas nuevas.
- El árbol de trabajo: las mutaciones sí tocan código, por eso van en worktrees temporales con `CARGO_TARGET_DIR` propio por auditor. Además, como mucho dos `cargo test` de mutación a la vez (cerrojo de fichero en `task-138\cerrojo`), porque 8 builds en paralelo del workspace saturan la máquina y solapan verificaciones.
- Coherencia con los ADR: la D2 se mide contra F1-1 … F1-19 tal como están en `docs/adr/`, sin reabrir ninguno; si un ADR está mal, va como nota, no como hallazgo.
- Desacuerdo previsto con riesgos: si propone `cargo mutants` sobre todo el workspace, mi posición es mutación dirigida a los candidatos. Un barrido completo de ~62 k líneas son horas de build y cuesta más de lo que añade a un diagnóstico.
- Supuesto: que `npx jscpd` y `cargo install cargo-mutants` se puedan descargar. Si la persona no lo autoriza, el pase 0 queda en `sg`, grep y clippy, y los mutantes se hacen a mano.
- Supuesto: que el reparto quede equilibrado (unas 9–11 k líneas por auditor). No he medido los tests por módulo uno a uno.

## La decision de diseño que mas te preocupa (UNA sola)

Que toda la Duplicación entre módulos dependa de la cobertura del pase 0. A8 solo ve los grupos que las herramientas o las pistas le llevan. Un clon semántico con otra estructura (la misma lógica escrita con `match` en un sitio y con `if let` en otro) escapa al hash normalizado y a `jscpd`, y ningún auditor por módulo lo va a reclamar. Mitigación propuesta: las pistas de los siete auditores entran a A8 como fuente obligatoria, y A8 hace una pasada dirigida por concepto sobre los conceptos que se repiten por diseño: escritura atómica, frontmatter y YAML, rutas de tarea, dobles de `Plataforma`, invocación de `git` y fechas. Esa lista sale de los ADR F1-2 y F1-4.

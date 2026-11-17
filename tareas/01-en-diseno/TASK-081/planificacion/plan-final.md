# Plan — TASK-081: F3-10 liberar y reasignar tareas

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 300 min

Ordenes de `tc2` (no de `taskctl`): `tc2 liberar [--json] TASK-NNN` y `tc2 reasignar [--json] TASK-NNN <login>`. F1-2 (D2b y adenda de TASK-034) ya las trata como operaciones compartidas que exigen red.

Forma calcada de `descartar`: `fetch` completo (tambien ramas); quien la lanza con `identidad::resolver`; comprobaciones previas fuera de la transaccion; operacion por `estado::transaccion` con el `ROADMAP.md` del mismo commit; ya publicada, el worktree con `con_worktree`/`quitar_worktree`. La rama no se toca nunca.

Orden de construccion:
1. `tarea::Cuerpo::ultima_fecha() -> Option<u64>` (instante de la ultima fila de `## Transiciones`, hermana de `en_pausa()`).
2. `situacion.rs`: `Ficha` gana `ultima_transicion`; funciones puras `actividad(ficha, commit_rama)` (maximo de la ultima fila y del ultimo commit de `origin/<rama>`) y `dias_sin_actividad(ultima, ahora)`.
3. `git/rama.rs`: `fechas_de_ramas(g, dir)` con un solo `for-each-ref --format='%(refname:short) %(committerdate:unix)' refs/remotes/origin/` (sin red).
4. `git/worktree.rs`: extraer de `quitar` la comprobacion de «publicado» como `worktree_publicado(g, dir, id) -> Paso<Option<&'static str>>`, compartida por `quitar_worktree` y la comprobacion previa de `liberar`.
5. `ciclo/liberar.rs` (nuevo): `Lanzada` pasa a `pub(super)`; `Liberar { base, actividad_rama, dias }` y `Reasignar { base, a: Asignado, actividad_rama, dias }` con `desde_anotacion` (anotaciones `liberar|TASK|id|login|<actividad o ->|<ahora>` y `reasignar|TASK|id|login|<id_dest>|<login_dest>|<actividad>|<ahora>`), `exige_red() = true`. Permiso con `de_otra_persona`; si la tarea es de otro, dias sin actividad contra N (por debajo: `rechazada motivo=activa dias=K umbral=N`). `Liberar`: `en-curso` → `planificada`, `en-diseno` se queda, `asignado_a: null`, conserva `rama` y `plan_aprobado`, fila `liberar`. `Reasignar`: solo `asignado_a`, conserva el estado, fila `reasignar`. Rechazos por estado con `por_estado`.
6. `ciclo/mod.rs` y `lib.rs`: `orden_liberar` y `orden_reasignar` reutilizan `escribir`, `con_dueno`, `con_worktree` y `de_la_orden`; dos ramas en `lib.rs` con `de_una_tarea`; `ciclo::desde_anotacion` en el catalogo. Destino validado con `identidad::personas_de_base` y `Personas::por_login` (nuevo, sin distinguir mayusculas) en `plataforma/personas.rs`; fuera de la lista: `rechazada motivo=fuera-de-lista`. Salida con `asignado-anterior=` y, en `reasignar`, `asignado=`.
7. `start` acepta la rama propia de la tarea (`toma/mod.rs` y `git::comprobar_id(g, p, id, propia)`): hoy cualquier `*/task-nnn-*` da `ocupada` (`git/rama.rs:56`, `toma/mod.rs:174`), lo que bloquearia a la siguiente persona. Con la rama propia ignorada, `Destino::crear_paso` entra por `git::reanudar`, que ya crea la rama local `--no-track` desde `origin/<rama>` y su worktree.
8. `tablero.rs`: `status` con `inactivas=N` (JSON) y `· sin actividad N` (texto, N>0); `board` marca «sin actividad hace D dias» en filas en curso y en revision; aviso sin plataforma al dueno anterior: `status`/`board` cruzan `git::worktrees` con la situacion y avisan `ya-no-tuyas=TASK-NNN` con quien la lleva. `doctor` no cambia.
9. `config/claves.rs`: clave `dias_sin_actividad` (1-365, defecto 7); la tabla ya tiene 12 claves (`TOPE_CLAVES = 12`), asi que se sube a 13 con adenda en F1-11 (D1a). `liberar` y `reasignar` en `FASES`.
10. i18n en `i18n/catalogo/ciclo.rs` y `tablero.rs`.
11. Adendas: F1-2 (punto 4: `start` admite la rama propia; `liberar`/`reasignar` implementadas), F1-3 (cierra «worktree tras liberar/reasignar»), F1-9 (filas `liberar`/`reasignar`; `reasignar` resuelve los heredados), F1-11 (tope 13).

Actividad (D2a): maximo de la ultima fila de `## Transiciones` (incluidas `pausa`/`reanudar`) y del ultimo commit de `origin/<rama>`; la pausa no exime. `status`/`board` usan las refs remotas tal como estan (avisos de mas, nunca de menos); `liberar`/`reasignar` deciden tras el `fetch` completo.

## Lo que el rol no cubrio

- Dominio: si «lista para empezar» tras liberar es `planificada` o `en-diseno`; si F1-17 corta la medicion en la fila `liberar`; si el dueno anterior pierde algun derecho.
- Riesgos: carrera entre el `liberar` de un tercero y un push del dueno; `committerdate` falseable o adelantado; el dueno anterior que sigue trabajando en su rama local y diverge.
- Testing: suficiencia de la bateria sin revisar.
- Criterio «sin perder su trabajo»: solo garantizado para lo publicado; lo no subido en el clon del dueno anterior no lo ve nadie (C8 y C10 lo contienen, no lo resuelven).
- Suposiciones a verificar: que «lista para empezar» incluye `planificada` sin asignado con dependencias cumplidas (`grafo.rs`, `siguiente.rs`); que `status` lee `config.yml` de la base sin red (`config/mod.rs`); que la cache de identidad se lee sin efectos (`identidad::Cache`).

## Riesgos aceptados y que los contiene

- Refs remotas sin traer en `status`/`board`: aviso de mas hasta el siguiente `fetch` completo; la decision siempre tras el `fetch` completo.
- Foto de actividad antes de la transaccion: si el dueno publica entre el `fetch` y el push del estado, la liberacion sigue; misma ventana que `comprobar_id` en `start`; la rama no se toca y C10 avisa.
- Sin aviso por plataforma (D5a): visible solo cuando el dueno anterior ejecuta `status`/`board`; queda constancia en la fila y en la anotacion.
- Coste: un `for-each-ref` y un `worktree list` por `status`/`board`, sin red.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- Unitarias en `ciclo/liberar.rs`: escribe solo lo suyo por estado de origen (C1, C3); rechazos `activa`, `requiere-persona`, `fuera-de-lista` y por estado (C5, C6, C11); heredado (C12); ida y vuelta de anotaciones.
- Unitarias de `Cuerpo::ultima_fecha`, `situacion::actividad`, `git::fechas_de_ramas` y `comprobar_id` con `propia`.
- E2E en `crates/tc2/tests/liberar.rs` con los helpers de `tests/ciclo.rs` (remoto bare, clones de `ana` y `beto`, `personas.yml` en `develop`): C2 (`merge-base --is-ancestor` del commit de `ana` en el worktree de `beto`), C3, C4, C5 (envejecer filas con el helper `Escribir` y la rama con `GIT_COMMITTER_DATE`), C7 (sin red con el ssh que rechaza), C8; C9 y C10 en `tests/tablero.rs`.
- Regresion: `toma.rs`, `ciclo.rs`, `tablero.rs`, `frescura.rs`, `config.rs` y `estado_*`.

## Lo que necesita decision de una persona

Carlos dejo dicho que en el H3 se acepta la opcion recomendada y que no se trocea; se aplican:

- D1 donde se configura N: (a) clave `dias_sin_actividad` en `.taskcode/config.yml` (defecto 7), `TOPE_CLAVES` 12 → 13 con adenda a F1-11. **Elegida.**
- D2 que es actividad: (a) maximo de la ultima fila de Transiciones y del ultimo commit de `origin/<rama>`; la pausa no exime. **Elegida.**
- D3 quien libera y reasigna: (a) el dueno siempre (tambien su agente); cualquier persona de `personas.yml` solo con N dias o mas y con terminal (el agente no quita a nadie una tarea); heredadas de la 0.16, cualquier persona verificada sin esperar N. **Elegida.**
- D4 trabajo sin publicar del dueno que libera: (a) `rechazada motivo=sin-publicar` antes de escribir. **Elegida.**
- D5 aviso a quien la tenia: (a) sin plataforma (linea, fila, commit de estado y `ya-no-tuyas=` en su `status`/`board`). **Elegida.**
- D6 estados: (a) `liberar` acepta `en-curso` (→ `planificada`) y `en-diseno` (se queda) y rechaza `en-revision` (tiene PR: `reasignar`); `reasignar` acepta los cuatro abiertos y conserva el estado. **Elegida.**

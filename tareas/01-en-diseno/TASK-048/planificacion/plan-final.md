# Plan — TASK-048: F6-2 review abre el PR con la identidad del agente

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 90 min
Senales: comando de fase nuevo (`revision/mod.rs`, `revision/operacion.rs`), un metodo nuevo en el trait `Plataforma` con dos implementaciones, catalogo i18n, despacho en `lib.rs`, adenda F6-2; 3 criterios.

`tc2 review TASK-NNN` es un comando de fase nuevo con el patron de `start` (`crates/tc2/src/toma/`); une piezas existentes sin capacidades de plataforma nuevas:
1. `push` de la rama de la tarea con el token del agente: `git::remoto::push` + `Git::con_entorno_extra` + `plataforma::entorno_git`.
2. `abrir_pr(rama, "develop", titulo, cuerpo)` del adaptador (`github.rs:411`, `gitlab.rs:560`), idempotente (`sin-cambios pr=N`).
3. `en-curso` -> `en-revision` por `estado::transaccion`, DESPUES de tener el PR, para que el estado nunca diga "en revision" sin PR.
4. `estado_puertas(Rol::Agente, pr, modelo)` para leer el check `revision-ia` y ponerlo en la linea.

Piezas y limites:
- `crates/tc2/src/revision/mod.rs` (nuevo): orquestacion `review_en(e, id)`, como `toma/mod.rs`. Depende de `git`, `plataforma` y `estado`; nadie depende de el (`git` sigue sin depender de `plataforma`).
- `crates/tc2/src/revision/operacion.rs` (nuevo): `Operacion` `Revisar` (`en-curso` -> `en-revision`; ya `en-revision` -> `sin-cambios`) con `desde_anotacion`, como `toma::Tomar` y `ciclo::Pausar`.
- Trait `Plataforma` (`plataforma/mod.rs`): un metodo `entorno_push(&self, url_remoto: &str) -> Paso<Vec<(OsString, OsString)>>`, por defecto `no-soportada`; GitHub y GitLab lo implementan con `self.con.token(op, Rol::Agente)` y `entorno_git`. Asi el token solo sale como cabecera limitada a la URL del remoto, sin duplicar la acunacion ni exponer el `Secreto`.
- `plataforma::entorno_git`: `autor` pasa a `Option<&AutorGit>` (review solo empuja; sin `GET /app`).
- `lib.rs` + `i18n/catalogo/revision.rs`: despacho `review [--json] TASK-NNN` como `start`; el `Entorno` de review es el de `toma::Entorno` mas `conectar` siempre con rol agente (no `conectar_produccion`).
- Adenda F6-2 en `docs/adr/F1-6-identidades-agente-revisor-persona.md`: `entorno_push` en la lista cerrada del puerto (F1-4), decision D1 y linea de salida.

Linea: `review ok id=TASK-NNN rama=<r> pr=<n> estado=en-revision revision-ia=<pendiente|sin-revisor|...>`.

"PR enlazado a la tarea" sin tocar el esquema: titulo `TASK-NNN: <titulo>`; cuerpo de una funcion pura `cuerpo_pr(&Tarea)` con el ID, la ruta de la carpeta de la tarea y `plan-final.md`; el enlace PR -> tarea ya lo resuelve `board` por la rama.

Orden de construccion:
1. `entorno_git` con autor opcional; `entorno_push` en el trait y los dos adaptadores.
2. `revision/operacion.rs` y su registro en el catalogo de operaciones.
3. `revision/mod.rs`: validar id -> fetch -> leer tarea (`en-curso`, con `rama`) -> push con entorno del agente -> `abrir_pr` -> transaccion -> `estado_puertas`. Repetir converge.
4. Despacho en `lib.rs` y textos.
5. Adenda F6-2 y E2E real ignorado.

Alternativas descartadas:
- `pr:` en el frontmatter: cambia el formato 2 (F1-9) por un dato derivable de `rama:`.
- Que el agente publique `revision-ia` como pendiente: prohibido por F1-6 (D2c descartada; el agente solo tiene `checks:read`; un check con su `integration_id` no contaria, G09).

## Troceo

No se trocea: Carlos pidio estas tareas sin trocear (2026-11-18).

## Lo que el rol no cubrio

- Modos de fallo (push rechazado, PR de fork, worktree sucio, carrera de dos `review`): rol de riesgos, no lanzado.
- Si leer e informar `revision-ia` basta para el criterio: lo resuelve D1.
- Quien puede lanzar `review` y si exige ser el asignado: se sigue la regla de `start`.

## Riesgos aceptados y que los contiene

- `revision-ia` no sale en verde en esta tarea: el ruleset ya lo exige por origen (TASK-118, `plataforma.revisor_id`) y `review` dice su estado. El revisor es de F6-3.
- Commits del agente con el autor local: la aprobacion que cuenta es la de la persona (D4 de F6-1) y `require_last_push_approval` mira quien hizo el ultimo push, que si es el agente.
- `commit-estado` con la identidad local, igual que en `start`.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- **PR enlazado:** (1) unitario de `cuerpo_pr` y del titulo; (2) integracion con remoto bare y `Plataforma` simulado: el push llega, `abrir_pr` recibe rama, `develop` y titulo, la tarea queda `en-revision` y la segunda ejecucion sale `sin-cambios`; (3) **pesado**, E2E `#[ignore]` contra `charliebk/taskcodetwo-e2e-github` (App 5832411) y `taskcodetwo-e2e/taskcodetwo-e2e-gitlab`, con el arnes de TASK-047.
- **check `revision-ia`:** 1 unitario: con `estado_puertas` simulado -> `revision-ia=pendiente`; sin `revisor_id` -> `revision-ia=sin-revisor` con resultado `ok`.
- **Requisitos F6-2:** el test de integracion comprueba que el push lleva `Authorization: Basic` del agente.

## Lo que necesita decision de una persona

Consigna de Carlos (2026-11-18): la opcion recomendada y sin trocear.

- **D1. check `revision-ia`.** Elegida: (A) `review` abre el PR, lee `revision-ia` e informa (`pendiente`/`sin-revisor`); lo publica el revisor en F6-3. Divergencia anotada en la adenda F6-2. (B, crear ya la App del revisor y un workflow, exigiria recursos externos y adelanta F6-3/F5-1.)
- **D2. autor `[bot]` en commits del agente.** Elegida: fuera; F6-2 no lo pide.
- **D3. carril `hotfix`.** Elegida: solo `develop`; `hotfix` sale `rechazada motivo=carril-no-soportado` hasta F7.
- **D4. E2E real.** Elegida: lo ejecuta quien orquesta antes de `finish`, con las identidades E2E existentes (TASK-047).

Suposiciones por verificar en la implementacion:
- Que `Git` se compone por peticion sin afectar al de la transaccion (`git/puerto.rs`).
- Que `git::remoto::push` funciona con `dir` = worktree de la tarea (`git/remoto.rs:364`).
- Que `estado_puertas` expone el estado del check `revision-ia` (`plataforma/puertas.rs`).

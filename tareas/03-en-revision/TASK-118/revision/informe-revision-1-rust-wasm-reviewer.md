# Informe de revision — TASK-118 (ronda 1)

- Commit revisado: 34a6b7b21259c24709fa626eb1939edb7a053ecc
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | corregido | crates/tc2/src/plataforma/gitlab.rs:1096 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — En GitLab, un fallo tras proteger las ramas deja los ajustes del proyecto sin aplicar y la repeticion no converge

- Donde: `GitLab::aplicar_puertas` (cierre de `aplicar_en_lote`, bloque `if let Some(pr) = p.solo_pr`) y `leer_para_aplicar` (estado `Difiere` por `!ajustes_bien`).
- Que pasa: para `develop` y `main` el orden es: POST de la rama protegida, luego PUT de ajustes del proyecto (`merge_method` y pipeline obligatorio) y luego la regla de aprobacion (Premium). Los ajustes y la regla solo se escriben mientras la puerta sale `Falta`. Si el PUT (o la regla) falla despues de que la rama ya quedo protegida, en la repeticion la rama existe y la puerta pasa a `Difiere` (por `!ajustes_bien` o por `premium && !con_aprobacion`), que "no se toca": los ajustes y la regla no se aplican nunca. El plan prometia "repeticion idempotente" que converge. Ademas, la primera ejecucion informa `aplicadas=` vacio aunque las ramas ya se protegieron.
- Reproduccion: prueba temporal en el clon (no commiteada) sobre `gitlab_premium` con `only_allow_merge_if_pipeline_succeeds=false`. Ejecucion 1: POST `protected_branches` 201, PUT del proyecto 500: `aplicar-puertas error motivo=http estado=500 aplicadas= faltan=taskcodetwo-develop,taskcodetwo-main,taskcodetwo-ramas-de-tarea,taskcodetwo-tareas,taskcodetwo-tags-version`. Ejecucion 2 con `develop` y `main` ya protegidas como el modelo (push 0, merge 40): `preparar_puertas` da `develop=Difiere, main=Difiere` y no hay ninguna orden que reintente el PUT.
- Impacto: develop y main quedan protegidas sin "pipelines must succeed" ni regla de aprobacion. No es silencioso: la linea sale `existe motivo=difiere` y el fallo de la primera ejecucion si se vio. Es una degradacion de la convergencia, no perdida de datos.
- Sugerencia: aplicar los ajustes y la regla aunque la puerta salga `Difiere` solo por ellos (son escrituras idempotentes que no tocan una proteccion existente), o dejar el caso documentado como `difiere` conocido en el `## Resultado`.

## Divergencias declaradas, valoradas

- GitLab `develop`/`main` con push=0 y merge=Maintainer (40): correcto frente a G01 (el merge es de la persona; el agente es Developer). Los niveles son siempre explicitos (`niveles_de`, `solo_nivel` exige un unico nivel sin usuarios ni grupos y `allow_force_push=false`); la mutacion `push: NADIE -> DEVELOPER` pone rojo `aplicar_puertas_en_gitlab_protege_con_niveles_explicitos`.
- GitLab `verificacion`/`carril` como checks del pipeline sin origen fijo (`del_pipeline`, vistos publicados por cualquiera): coherente con F1-13 G09 (el autor del estado es quien lanzo el pipeline). Queda la debilidad ya aceptada de que cualquiera con permiso de pipeline los publica; no es nuevo de esta tarea.
- GitHub reconoce lo existente por nombre sin marcar `difiere`: coherente con D3 (nunca se compara ni se toca). El informe no avisa si un `taskcodetwo-*` existente difiere, pero el plan lo declara. No es hallazgo.
- Nota sin hallazgo: el PUT de ajustes del proyecto GitLab (`merge_method=merge`, pipeline obligatorio) cambia una configuracion existente si difiere; esta en el alcance del plan (parte c) y solo ocurre con la puerta `Falta`.

## Reproduccion

- Clon: `%LOCALAPPDATA%/pruebas-claude/TaskCodeTwo/task-118-r1-rust`, checkout de 34a6b7b. `git diff --stat a7f212e 34a6b7b -- crates docs` vacio (codigo identico a a7f212e).
- Suite: no repetida; `cargo xtask verificar --perfil rapida` salio en verde (exit=0) sobre a7f212e segun `verif-118.log` ("xtask: verificacion Rapida en verde").
- Tests concretos en el clon: `cargo test -p tc2 --lib puertas` 5 ok; `--test init aplicar_puertas` 4 ok (169 s); `--test plataforma aplicar_puertas` 1 ok; `--test plataforma_gitlab aplicar_puertas` 1 ok. `cargo clippy -p tc2 --all-targets` sin avisos; `cargo fmt --check` limpio.
- Mutacion 1: `push: NADIE` -> `DEVELOPER` en `niveles_de`: `aplicar_puertas_en_gitlab_protege_con_niveles_explicitos` rojo.
- Mutacion 2: anular la comprobacion D2 (`no_vistos`) en `GitHub::preparar_puertas`: el test de adaptador `aplicar_puertas_carrera_corte_y_plan` sigue verde, pero `init aplicar_puertas_sin_origen_se_rechaza_sin_escribir` se pone rojo. Hay red, solo a nivel init.
- Casos construidos: el de MEN-1 (prueba temporal, descartada con `git checkout`).

## Revisado sin hallazgos

- Errores sin panic: sin `unwrap`/`expect`/indexado sobre datos externos en el codigo nuevo; `faltan[k..]` acotado por el bucle; respuestas de la plataforma por `parsear` con `Result`; `unwrap_or_default` solo sobre listas.
- Ningun camino de rechazo escribe en la plataforma: en `init`, el primer `Rechaza` devuelve antes de aplicar nada (disco incluido); en los adaptadores, `preparar_puertas` solo hace GET y el cuerpo con `_nota` nunca se envia (`sin_origen_visto`).
- Sin `DELETE`; no hay `PUT` de rulesets (GitHub solo `POST /rulesets`; GitLab `POST` de ramas, tags y reglas, y un `PUT` del proyecto). 422 (GitHub) y 409 (GitLab) se tratan como "otro llego antes" releyendo.
- Precondicion de origenes: `origenes()` es pura, 0 cuenta como ausente; `revision-ia` sin `revisor_id` rechaza en ambas plataformas; `integration_id` 15368 de Actions correcto.
- Precondicion de ramas (D5): `merge-base --is-ancestor`; si el remoto no esta traido sale `ramas-por-delante` (fail-closed, el mensaje lo dice).
- Idempotencia en GitHub: segunda ejecucion sin escrituras (existentes por nombre).

## Lecturas fuera del contexto permitido

- `crates/tc2/src/plataforma/gitlab.rs` (structs `RamaProtegidaApi`, `NivelApi`, `EstadoApi`, `ReglaAprobacionApi`) y `crates/tc2/src/plataforma/puertas.rs` (`modelo()`): para comprobar tipos y patrones que el diff usa sin mostrar.
- `crates/tc2/tests/plataforma_gitlab.rs` (helper `gitlab_premium`): para construir el caso de MEN-1.

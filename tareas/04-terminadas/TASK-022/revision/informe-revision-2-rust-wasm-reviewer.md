# Informe de revision — TASK-022 (ronda 2)

- Commit revisado: 7b4697f67ade1f22276dd31d65c166ba74f6224d
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/plataforma/gitlab.rs:581 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

IMP-1 (ronda 1, `merge-pr` sin `exigir_persona`): **corregido**. La llamada esta de nuevo en
`merge_pr` (gitlab.rs:581). Mutante que la quita: `protecciones_de_gitlab` pasa a rojo
(`merge-pr denegada pr=12 motivo=no-existe-o-sin-acceso rol=persona` en vez de
`merge-pr rechazada motivo=identidad rol=persona`, plataforma_gitlab.rs:700). El test nuevo
(token de persona 4242 fuera de la lista, sin PUT) mata el mutante.

Sin hallazgos CRITICO ni IMPORTANTE nuevos. Ningun hallazgo nuevo introduce la correccion.

## Alcance
- Ficheros revisados: crates/tc2/src/plataforma/gitlab.rs (G01, `sin_conflictos`, `merge_pr`,
  `estado_puertas`, `traducir` de la rama 401), crates/tc2/src/plataforma/mod.rs (tipo `Clave`),
  crates/tc2/tests/plataforma_gitlab.rs (tests nuevos del diff de la ronda).
- Contexto adicional pedido: ninguno fuera del permitido, salvo lo listado abajo.
- Limite JS-WASM: sin cambios de firma `#[wasm_bindgen]`; el crate `tc2` es nativo. No aplica.

## Reproduccion
- Clon: `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-022-r2-rust`, detached en 7b4697f, con su propio `target/` dentro. Borrado al terminar.
- Build: `cargo test -p tc2 --locked` compila sin errores en nativo. No hay objetivo wasm32 en este crate.
- Formato y lints: no ejecutados en esta ronda (no es el foco de la verificacion acotada).
- Suite (`cargo test -p tc2 --locked --no-fail-fast`, sobre 7b4697f): **verde**. lib 41, alta 3, cli 2, estado_bordes 9, estado_casos 4, estado_concurrencia 3, git_binario 1, git_casos_limite 5, git_ciclo 1, git_conflicto 1, git_push 1, plataforma 4, plataforma_gitlab 3, tarea 5 (1 ignorado). Los E2E (`plataforma_e2e`, `plataforma_gitlab_e2e`) quedan `ignored` y no se ejecutaron.
- Atasco de red local (`sinred`): en la primera pasada de `plataforma_gitlab` (paralela) fallaron los 3 tests con `estado-puertas sinred` tras 127 s. Repetido en solitario con `--test-threads=1`: 3 passed en 30 s. Es el atasco preexistente descrito en la peticion, no un defecto del diff.
- Mutacion (tests `plataforma_gitlab`, `--test-threads=1`, sobre el clon; cada mutante revertido despues):
  - A. Quitar `exigir_persona` de `merge_pr` (gitlab.rs:581-583): **MUERTO** (`protecciones_de_gitlab` rojo).
  - B. Quitar `n.user_id.is_none() && n.group_id.is_none()` de G01 (gitlab.rs:500-502): **MUERTO** (`protecciones_de_gitlab` rojo: `estado-puertas ok` donde se esperaba `rechazada ... G01`, plataforma_gitlab.rs:683).
  - C. Quitar `p.merge_access_levels.len() == 1` de G01 (gitlab.rs:499): **VIVO** (suite verde). Ver nota 1.
- Casos construidos: entrada con `merge_access_levels` de dos entradas (Maintainer + Developers) y con `user_id` concreto, a traves del fixture de los tests.

## Revisado sin hallazgos
- `merge_pr`: la comprobacion de `exigir_persona` se ejecuta antes de cualquier lectura o escritura; el 401 del merge (rama nueva) devuelve `denegada motivo=permisos` sin reintentos ni PUT extra.
- `estado_puertas` / `sin_conflictos`: el estado `checking`/`unchecked`/`preparing`/`conflict` falla cerrado (no se da por libre); la rama de `has_conflicts` original se conserva.
- `FirmaApi`/`Clave`: el `signature_type` decide el espacio de ids (SSH o GPG); un id GPG ya no se compara contra una clave SSH de la lista (test nuevo con `PGP` y `gpg_key_id` igual a una clave SSH registrada).
- `estados?all=true`: sin cambios de semantica en la seleccion del ultimo estado por id.

## Notas (no son hallazgos de la tabla)
1. **Mutante C vivo (MENOR, cobertura del MEN-1 de la ronda 1).** La condicion `merge_access_levels.len() == 1` no tiene test que la ejerza. El test nuevo de G01 anade una segunda entrada con `access_level: 30` (Developers), que ya falla por la clausula `all(== MAINTAINER)`, asi que no distingue la condicion de longitud. Un caso que la mataria es dos entradas con `access_level: 40` y sin `user_id`/`group_id`. Es cobertura de un MENOR de la ronda 1 que el orquestador comprueba; no se abre hallazgo nuevo porque no lo introduce esta correccion.
2. **Mapeo nuevo del 401 en `merge_pr` (observacion).** Antes un 401 en el merge caia en `traducir` como `motivo=credenciales` («el token ... no vale o ha caducado»). Ahora se informa como `motivo=permisos`. Es coherente con el comentario del codigo (el token ya se valido con `GET /user` en la misma llamada), pero si el token caduca entre ambas peticiones, el diagnostico apunta a permisos y no a credenciales. La ventana es estrecha y el fallo es cerrado (denegada), por eso no se abre hallazgo. No hay test de este caso.
3. MEN-2 de la ronda 1 (403 de `approval_rules` como `motivo=plan`) queda aceptado por el orquestador y no se revisa aqui.

## Lecturas fuera del contexto permitido
- `crates/tc2/src/plataforma/errores.rs` (lineas 80-120): para comprobar cómo `traducir` trata el 401 antes y despues de la rama nueva (nota 2).
- `tareas/03-en-revision/TASK-022/revision/informe-revision-1-rust-wasm-reviewer.md` y la peticion de la ronda 2: son la entrada de la verificacion, no codigo; se leyeron para cerrar la tabla.

# Informe de revisión — TASK-035 (ronda 1)

- Commit revisado: fe169a0
- Revisor: rust-wasm-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| H1 | IMPORTANTE | corregido | crates/tc2/src/doctor/plataforma.rs:287-295; crates/tc2/src/doctor/capacidades.rs:376-383 | implementacion |
| H2 | IMPORTANTE | corregido | crates/tc2/src/doctor/capacidades.rs:309-319 (tests :459-620) | implementacion |
| H3 | MENOR | corregido | crates/tc2/src/plataforma/github.rs:~655-667; crates/tc2/src/plataforma/gitlab.rs:~359-402 | implementacion |
| H4 | MENOR | corregido | crates/tc2/src/plataforma/gitlab.rs:~814-826; crates/tc2/src/doctor/capacidades.rs:300 | implementacion |
| H5 | MENOR | corregido | crates/tc2/src/plataforma/github.rs:~248-262 | implementacion |

### H1 (IMPORTANTE): sin leer las puertas, doctor da por impuestas garantias cuya configuracion no leyo

Si `leer_puertas` falla, `de_lo_leido(..., develop = None)` no ejecuta `falta_config` y G01, G02, G03, G06 y G09 salen `ok capacidad=impuesta` con `proteccion ... no-comprobado`; contradice C9 y `capacidades.rs:11-15`. Reproducido: 403 en las puertas (GitHub `impuestas=19`, GitLab `impuestas=17`); `rules/branches` con `required_status_checks: null` (`plataforma=error`, `impuestas=19`); GitLab `approval_rules` con `approvals_required: null` (`impuestas=17`). Arreglo: sin puertas, esas garantias salen `no-comprobado` (o se cuentan aparte), no `impuesta`.

### H2 (IMPORTANTE): la rama `config` de `capacidades` casi sin test

Mutaciones que sobreviven: G03 `r.aprobaciones < 1`→`Some(false)`, G09 `checks_requeridos…is_empty`→`Some(false)`, G01 `deletion && non_fast_forward`→`Some(false)`. Solo G06 muere. Falta caso GitHub sin `deletion`/`non_fast_forward`, con `aprobaciones=0` y `checks_requeridos=Some(vec![])`.

### H3 (MENOR): errores tragados con `.ok()` en checks publicados

403/500/`check_runs:null`/sin `status`/rama sin `commit` acaban en `no-comprobado motivo=checks-no-leidos` sin decir el permiso ni la causa.

### H4 (MENOR): un 403 en `approval_rules` de GitLab se lee como config ausente o plan Free

Con `namespace.plan=premium` sale `garantia G03 aviso motivo=config capacidad=no-disponible`; sin namespace legible, `plan_de` deduce Free del 403 (ya aceptado en F1-13).

### H5 (MENOR): `#[serde(default)]` no cubre `null`

`"required_status_checks": null` hace fallar todo `leer_puertas`. Usar `Option<Vec<_>>` o `deserialize_with`.

## Reproduccion

Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-035-r1-rust` (borrado). `fmt --check` y `clippy -D warnings` limpios; `--test doctor` 11/11 (213,9 s); `--lib doctor` 9/9. Sin unwrap/expect/panic ni indexado sobre entrada externa. Limite de 5 lineas protegido (mutacion `take(hueco)` en rojo); `v <= GIT_VERIFICADO` en rojo; `check-sin-publicar` aviso→falla en rojo. 22 sondas de respuestas raras sin panic, todas degradan a `no-comprobado` salvo H1. Solo GET. `proveedor()`/`leer_plan()` por defecto correctos.

## Revisado sin hallazgos

`version_de_git`, `hallazgo_git`, agregador, `proteccion` y `checks` puros, `Consulta::de_fallo` y el movimiento a `identidad.rs`, escape en `Hallazgo::linea`. Limite JS-WASM: no aplica.

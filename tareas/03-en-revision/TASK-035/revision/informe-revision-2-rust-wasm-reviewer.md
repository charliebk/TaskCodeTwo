# Informe de revisión — TASK-035 (ronda 2)

- Commit revisado: e1924a6 (codigo de f5b75c0)
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| H1 | IMPORTANTE | corregido | crates/tc2/src/doctor/capacidades.rs:176-183, 376-383; crates/tc2/src/doctor/plataforma.rs:254-270 | implementacion |
| H2 | IMPORTANTE | corregido | crates/tc2/src/doctor/capacidades.rs:309-319 (tests :459-620) | implementacion |
| R2-3 | MENOR | aceptado | crates/tc2/src/plataforma/gitlab.rs (`nulo_por_defecto` en `approvals_required`, `code_owner_approval_required`) | implementacion |
| R2-4 | MENOR | aceptado | crates/tc2/src/plataforma/mod.rs (`leer_puertas_de` por defecto) | implementacion |

Los MENORES de la ronda 1 (H3, H4, H5, R1-01..R1-06) comprobados por lectura y, cuando habia test, ejecutados. R2-3: con `null` sale `no-disponible motivo=config arreglo=tc2-init` en vez de `no-comprobado` (conservador; aceptado). R2-4: la implementacion por defecto no pone `checks_publicados: None` fuera de `checks_de` como dice su doc (inocuo: `checks()` solo mira `RAMA_CHECKS`; aceptado).

## Reproduccion

Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-035-r2-rust` (borrado). `fmt --check` y `clippy -D warnings` limpios; `--lib doctor` 15/15; `--test doctor` dos veces, 12/12 ambas (436,8 s y 160,1 s).

Mutaciones en rojo: H1 (`return None` en vez de `NoLeida`) → `sin_puertas_lo_que_se_lee_no_esta_comprobado`; H2 G03, G09 y G01 `Some(false)` → `config_de_github_garantia_a_garantia`; R1-01 tier GitLab de G05 → `la_tabla_compilada_es_la_de_f1_13` (y con ruta de ADR inexistente tambien falla); H4 guarda `!r.plan_da_reglas` → `el_403_de_aprobaciones_con_premium_es_un_permiso`.

## Revision especifica

`nulo_por_defecto` es conservador (nunca da por impuesta lo no leido). `leer_puertas_de`: orden y longitud de ramas preservados, una rama invalida no tumba las demas, el primer `sinred`/`sin-credenciales` corta igual que antes. `FalloLectura` sin panics y sin filtrar tokens.

Flaky `capacidades_por_plan_en_las_dos_plataformas` con `plataforma=sin-red`: no reproducido en dos ejecuciones completas. Causa compatible con la documentada en TASK-115 (paradas de red del antivirus en Windows que acumulan mas de 30 s; `serie()` solo excluye dentro del mismo binario de tests). El mock no tiene fallo propio y los cambios de esta ronda reducen peticiones. No es hallazgo.

Nota: `PERMISO_CHECKS = "checks:read"` no verificado contra la documentacion en esta ronda (el implementador lo cito de la tabla de permisos de GitHub Apps).

# Informe de revision — TASK-042 (ronda 1)

- Commit revisado: aee1621fc055573ce506eeeb31dda3082f1027a2
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | aceptado (seguimiento: LECTURAS_ADMITIDAS por clase de job; hoy ejecutar no las declara y el test lo fija) | xtask/src/workflows.rs:471 | implementacion |

**MEN-1.** `LECTURAS_ADMITIDAS` (l.340) se aplica igual a todos los jobs: un job de clase ejecuta (el que corre codigo del PR) podria declarar `checks: read` o `actions: read` y el lint no lo vería. Hoy `ejecutar` no los declara y el test `la_cache_por_arbol_respeta_las_clases` (l.~1140) lo fija sobre el workflow real (`permiso("ejecutar","checks") == None`), asi que no hay defecto actual; es solo que la regla del lint es mas laxa que la prosa del plan ("`buscar` acepta..."). Sin accion obligatoria.

Resto: sin hallazgos CRITICO ni IMPORTANTE.

## Alcance
- Ficheros revisados: xtask/src/workflows.rs (unico .rs del diff).
- Contexto adicional pedido: ninguno (leidos tarea.md, plan-final.md y la peticion).
- Limite JS-WASM: no aplica; sin `#[wasm_bindgen]` en el diff.

## Reproduccion
- Clon: C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-042-r1-a, commit fc8caf0 (rama de la tarea; sin commits ni push).
- Build: lo cubre `cargo test`/`clippy` de xtask (nativo; no hay wasm32 en este diff).
- Formato y lints: `cargo clippy -p xtask --all-targets -- -D warnings` sin avisos.
- Suite: `cargo test -p xtask`, 26 pasados, 0 fallos (incluye `la_cache_por_arbol_respeta_las_clases` y `el_lector_entiende_el_subconjunto` con el caso `- |`). Verificacion completa del implementador en verde, citada de la peticion, no repetida.
- Casos construidos: lector `- |` (l.194-197) leido contra el codigo: la sangria pasada a `bloque_literal` es la del guion, por lo que el bloque acaba en el siguiente item o clave de menor/igual sangria; documento que acaba en `- |` da cadena vacia sin panic (`lineas[*i..]` con i<=len); sin `unwrap`/`panic` nuevos en codigo de produccion (los `panic!` estan en helpers de test). Regla `lectura no admitida` solo mira `read`: `write` ya cae en `escribe` -> clase privilegio, y `read-all`/otros valores dan error en `escribe`; un job sin `permissions` sigue dando error, asi que los permisos de la raiz no se heredan sin pasar por el lint. Checkout en privilegio ahora siempre error (subsume la regla de `ref`/`repository`, que queda como segundo mensaje).
- Mutacion: `if privilegio {` -> `if false && privilegio {` (afecta al checkout y a la subida): se ponen rojos `la_cache_por_arbol_respeta_las_clases` y `el_informe_se_sube_siempre_y_solo_desde_ejecutar`. Revertida con `git checkout`.
- Medidas: no aplican.

## Revisado sin hallazgos
- Lector YAML: item `- |`/`- >` con y sin lineas vacias intermedias; el test nuevo del caso `j:` compara el valor exacto.
- Reglas de clases: checkout prohibido en privilegio, lecturas admitidas, negativas con `checks: write` y `pull-requests: read` reales sobre el workflow.
- Errores: el lint devuelve `Vec<String>` / `Result<_, String>`, sin `unwrap`/`expect`/indexado sobre entrada externa en lo anadido.

## Lecturas fuera del contexto permitido

ninguna

# Informe de revision — TASK-085 (ronda 1)

- Commit revisado: 0689c2cec9429cd0695cd68458dcdad880695989
- Revisor: rust-wasm-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/seleccion/catalogo.rs:89 | implementacion |
| IMP-2 | IMPORTANTE | corregido | crates/tc2/src/seleccion/mod.rs:270 | implementacion |
| IMP-3 | IMPORTANTE | corregido | crates/tc2/src/tarea/modelo.rs:301 | implementacion |
| MEN-1 | MENOR | corregido | crates/tc2/src/seleccion/orden.rs:637 | implementacion |
| MEN-2 | MENOR | corregido | crates/tc2/src/seleccion/orden.rs:609 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Debajo, la reproduccion de cada hallazgo, o "sin hallazgos" explicito.

### IMP-1 — `Catalogo::huella` depende de la ruta absoluta de cada skill
`huella` hashea `format!("{:?}{:?}", revisoras, otros)` y `Revisora` incluye `ruta`, que para lo instalado es la ruta del `SKILL.md` en disco (`de_carpeta`, `f.display()`). Caso: dos `Catalogo` con la misma skill y el mismo contenido y distinta ruta (`C:/u1/.claude/x/SKILL.md` frente a `/home/u2/.claude/x/SKILL.md`) dan `942d97385c9b` y `92f6edb3e1c7` (test temporal en el clon, rojo). La doc dice «forma canonica» y `Inventario::huella` no usa rutas. Efecto real: `plan` anota `roles.<rol>.catalogo` y `cambia_tarea` compara `t.roles != nuevos` (`plan/operacion.rs`), asi que repetir `plan` desde otra maquina o con otro `CLAUDE_CONFIG_DIR` con el mismo contenido reescribe la tarea y abre fila de transicion en vez de dar `sin-cambios`. Arreglo: excluir `ruta` de la huella (hashear id, rol, etiquetas, dominios, prioridad, patrones, fallback y umbral).

### IMP-2 — S06 y el orden de S07 sin ningun test (Disfraz)
`skills_diseno()` (S06, `mod.rs:270`) y `ejecucion()` (S07, `mod.rs:281`: solape, prioridad, id y aviso `empate-por-id`) no los ejerce ningun test: no hay una sola entrada de catalogo con `rol: diseno` y la unica de `ejecucion` (`task-workflow`) esta sola. Mutacion: (a) invertir `prioridad` e `id` en el `sort_by`; (b) hacer que `skills-diseno-del-dominio` no anada nada; (c) desactivar el aviso `empate-por-id` (`empate.len() > 99`). Los tres mutantes dejan `cargo test -p tc2 --lib seleccion::` en 14 passed, 0 failed, y los tests de integracion no declaran skills de esos roles. Son dos de las filas nuevas de la tabla y la regla de desempate de la decision 8.

### IMP-3 — Duplicacion: seis campos y su lectura/escritura repetidos en `Rol` y `RolRevision` (4D)
Se anaden los mismos `agentes, reglas, senales, avisos, inventario, catalogo` a `Rol` y a `RolRevision` (`modelo.rs:274` y `:301`), el mismo bloque de `textos(...)`/`texto_opcional(...)` en `rol()` y en la lectura de `roles.revision`, y `ampliado()` se llama con los mismos seis argumentos dos veces. Son dos tipos que hacen lo mismo con la diferencia `agente`/`ronda`: bastaba un struct compartido (flatten o campo) y reutilizarlo; hoy cada campo futuro hay que tocarlo en cuatro sitios.

### MEN-1 — Duplicacion menor en `orden_seleccion`
El cierre `nombres` (`orden.rs:637`) es `plan::lista_o_ninguno` (`plan/mod.rs:512`) y el cierre `modelo` es `plan::o_guion`; ambos helpers ya son `pub(crate)` y se reimplementan. Ademas `plan::operacion::dominios_de(senales)` (`operacion.rs:315`) comparte nombre con `seleccion::orden::dominios_de`, que hace otra cosa.

### MEN-2 — Heuristica calculada dos veces en `paso_seleccion`
`de_tarea` ya resuelve la heuristica y devuelve `(carril, complejidad)`; `paso_seleccion` descarta la complejidad (`let (carril, _)`) y vuelve a llamar a `heuristica::resolver` y `complejidad(&r)` (`orden.rs:609`) para sacar `r.roles`. No cambia el resultado, pero es un camino duplicado que puede desviarse.

## Alcance
- Ficheros revisados: los de la peticion (config, doctor/recursos, i18n, lib, plan/*, revision/*, seleccion/*, tarea/modelo, tests aprobar/comun/plan/seleccion/tarea).
- Limite JS-WASM: no aplica (crate nativo sin `#[wasm_bindgen]`; ninguna firma exportada cambia).
- Sin `unsafe`, sin `unwrap`/`expect`/`panic!`/indexado sobre datos de entrada en el codigo nuevo; los `unwrap_or_default` sobre las tablas del ADR compilado no se pueden disparar (`las_tablas_del_adr_se_leen`).

## Reproduccion
- Clon: `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-085-r1-rust`, `git checkout 0689c2cec9429cd0695cd68458dcdad880695989`, target propio; borrado al terminar.
- Linea base: `cargo xtask verificar --perfil rapida` en verde en 7345f6a segun la peticion; no repetida.
- Suite de mi parte: `cargo test -p tc2 --lib seleccion` 16 passed; `cargo test -p tc2 --test seleccion --test tarea --test plan --test aprobar`: aprobar 10, plan 7, seleccion 2, tarea 8 (1 ignorado, pesado), todo ok.
- Casos construidos: IMP-1 con un test temporal en el clon (misma skill, dos rutas, huellas distintas), revertido.
- Mutacion: ver IMP-2 (3 mutantes de S06/S07 sobreviven). Lectura de las filas S01-S15 contra `aplicar`/`atomo`: sin diferencias con el ADR.
- Medidas y wasm32: no aplican.

## Revisado sin hallazgos
- Detector de dominios (`dominios.rs`): plegado de tildes y mayusculas, palabra a principio, etiqueta exacta, `union` solo suma, dominio fuera de lista descartado.
- Tabla de modelo y S13/S15 (`restringir`, `modelo-baja`), S14 con aviso, `plan` con agente no instalado (peticion escrita igual con aviso), `approve` sin recalcular, `revision` con senales congeladas, `tc2 seleccion` (tope de bytes, argumentos, formato 1 y 2).

## Regla 4D
- Duplicacion: IMP-3 (y MEN-1)
- Desviacion: cumple
- Diluvio: cumple
- Disfraz: IMP-2 (S06/S07 sin cobertura; `diez_tareas_de_taskcodetwo` solo afirma contrato y determinismo fuera de tres estratos, y lo declara en su doc)

## Lecturas fuera del contexto permitido

ninguna. La peticion estaba truncada tras la linea 1144; el resto del diff lo lei del propio clon (los mismos ficheros de la lista, mas `plan/roles.rs` y `tarea/texto.rs` para comprobar `UNIFICADOR` y `plegar`, y `docs/adr/F1-15` para las tablas).

Veredicto propuesto: aprobada-con-correcciones

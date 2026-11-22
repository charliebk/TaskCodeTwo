# Informe de revision — TASK-085 (ronda 2)

- Commit revisado: 1fc787b2571f855dd609564c17b18d55c7713123
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|

IMP-1, IMP-2, IMP-3, MEN-1 y MEN-2 de la ronda 1 quedan corregidos de verdad (ver abajo). La correccion no abre ningun fallo nuevo.

### Verificacion de la ronda 1

- IMP-1 (huella con ruta): corregido. `Catalogo::huella` hashea ahora id, rol, etiquetas, dominios, prioridad, patrones, fallback y umbral, sin `ruta`. El test `el_catalogo_no_depende_del_orden` fija que dos rutas distintas dan la misma huella y que cambiar `prioridad` la cambia. Mutacion: anadir `&r.ruta` a la tupla deja ese test en rojo.
- IMP-2 (S06/S07 sin test): corregido. El test `skills_de_diseno_y_ejecucion_por_dominio_solape_prioridad_e_id` cubre S06 (diseno del dominio detectado, y vacio sin dominio), el solape por etiquetas y por dominio, la prioridad frente al id y el aviso `empate-por-id:e-m,e-n`. Los tres mutantes de la ronda 1 (orden prioridad/id, S06 que no anade, aviso desactivado) caen ahora en rojo, y tambien el desempate por id invertido (cuarto mutante).
- IMP-3 (duplicacion Rol/RolRevision): corregido. Hay un solo `Rol`, con `agente` y `ronda` opcionales; `rol(clave, r, revision)` y `escribir` comparten lectura, validacion y escritura, y `ampliado(&Rol)` ya no recibe seis argumentos sueltos. `agente`/`ronda` en diseno o implementacion se rechazan al leer («clave desconocida») y no se escriben; el orden de escritura de revision (agente, modelo, skills, regla, ampliado, ronda) no cambia. Mutacion: desactivar el rechazo (`if false && hay && !revision`) pone en rojo `roles_ampliados_ida_y_vuelta`.
- MEN-1 (reimplementar `lista_o_ninguno`/`o_guion`; nombre repetido): corregido. `paso_seleccion` usa `plan::lista_o_ninguno` y `plan::o_guion`; el de senales pasa a `dominios_de_senales`.
- MEN-2 (heuristica dos veces): corregido. `de_tarea` devuelve `(Carril, Resolucion)` y `paso_seleccion` no vuelve a llamar a `heuristica::resolver`; `seleccion_de` y `paso` sacan la complejidad con `complejidad(&r)`.

### Regresiones buscadas en el diff de la ronda

- `dominios_vivos` unifica el detector de dominios entre `plan` y `orden`: mismo texto (objetivo, criterios, etiquetas) y mismos `dominios_extra`; con `Err` en `plan`, `paso` y `paso_seleccion` (informe de error con `TABLA_ILEGIBLE`). La tabla del ADR va compilada, asi que el `Err` no se dispara hoy (lo cubre `las_tablas_del_adr_se_leen`). Los tests de `plan` y `aprobar` pasan con el cambio.
- La tabla dorada `tabla.md` se compara con la salida del binario (`diez_tareas_de_taskcodetwo`, verde) y la regla de muestra queda escrita en `estratos.txt`.

## Alcance
- Ficheros revisados: los de la peticion (plan/operacion, revision/mod y operacion, seleccion/catalogo, mod y orden, tarea/mod y modelo, tests seleccion y tarea, fixtures, ADR F1-15).
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: no aplica (crate nativo, sin `#[wasm_bindgen]`; ninguna firma exportada cambia).

## Reproduccion
- Clon: `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-085-r2-rust`, `git checkout 1fc787b2571f855dd609564c17b18d55c7713123`, target propio; borrado al terminar.
- Linea base: la rapida esta en verde en 2e2d33d (mismo codigo) segun la peticion; no repetida.
- Suite de mi parte: `cargo test -p tc2 --lib seleccion` 17 passed; `cargo test -p tc2 --test seleccion --test tarea --test plan --test aprobar`: aprobar 10, plan 7, seleccion 2, tarea 8 (1 ignorado, pesado), todo ok.
- Mutacion (todas restauradas con `git checkout`): ruta dentro de la huella (rojo en catalogo), orden prioridad/id invertido (rojo), aviso `empate-por-id` desactivado (rojo), `skills_diseno` vacio (rojo), desempate por id invertido (rojo), rechazo de `agente`/`ronda` en diseno desactivado (rojo en `tarea`).
- Build wasm32, formato, clippy y medidas: no repetidos; la rapida de la peticion los cubre.

## Revisado sin hallazgos
- Lectura y escritura de `tarea.md` con el `Rol` unico (ida y vuelta, tarea antigua byte a byte).
- Propagacion del `Err` de las tablas en `plan`, `tc2 seleccion` y `tc2 revision`; sin `unwrap`/`expect`/`panic!`/`unsafe` nuevos.

## Notas fuera de la tabla (no abren ronda)
- `seleccion/orden.rs:491-494` sigue construyendo a mano el informe `TABLA_ILEGIBLE` que el helper `tabla_ilegible` (orden.rs:233) ya encapsula; una linea de duplicacion menor.
- `revision/mod.rs:229` convierte el `Err` de `dominios_de` en vacio con `unwrap_or_default()`: no es alcanzable hoy (tabla compilada), pero es el unico sitio que traga ese error mientras los demas lo muestran.
- La rama `Err` de `dominios_vivos` no tiene test (no se puede forzar sin inyectar la tabla).

## Regla 4D
- Duplicacion: cumple (IMP-3 y MEN-1 resueltos; resto menor en las notas)
- Desviacion: cumple
- Diluvio: cumple
- Disfraz: cumple (S06/S07 y la huella con tests que caen con mutacion; la tabla dorada se compara)

Veredicto propuesto: aprobada

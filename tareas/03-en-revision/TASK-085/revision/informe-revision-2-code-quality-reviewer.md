# Informe de revision — TASK-085 (ronda 2)

- Commit revisado: 1fc787b2571f855dd609564c17b18d55c7713123
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|

Verificacion de la ronda 1 (los MENORES no abren ronda; se comprueba su estado):

- MEN-1 (test sin golden): corregido. `diez_tareas_de_taskcodetwo` compara la tabla que sale del binario con `tests/fixtures/seleccion-taskcodetwo/tabla.md` (`assert_eq!(tabla, dorada...)`, seleccion.rs:171). La tabla dorada de la fixture es byte a byte la de la evidencia (solo cambia el preambulo). Mutacion: cambiar `opus / sonnet / opus` por `sonnet / sonnet / opus` en `tabla.md` pone rojo el test (1 passed, 1 failed). Antes la mutacion de modelo pasaba en verde.
- MEN-2: aceptado en la ronda 1; la regla queda escrita en la cabecera de `estratos.txt` (orden de asignacion, exclusion de tareas ya elegidas, estrato `alta` vacio y sustituto), aunque sigue sin ser ejecutable. El test filtra las lineas `#`, y `filas.len() == 10` lo protege.
- MEN-3 (doble construccion de dominios y error tragado): corregido. `Planificar::seleccion` y `dominios_de` llaman a una sola `orden::dominios_vivos`; `seleccionar` y la tabla propagan `Err` (`TABLA_ILEGIBLE`) en `plan`, `tc2 seleccion` y la orden de revision; `roles_de` perdio el camino de respaldo a `roles::seleccionar`. `de_tarea` devuelve la resolucion y ya no se recalcula la heuristica en `paso_seleccion`.
- MEN-4 (implementacion sin discriminar): corregido. La adenda del ADR avisa de que la columna «Implementacion» sale «—» en las 10 y que no hay skills de diseno; ademas el unitario nuevo `skills_de_diseno_y_ejecucion_por_dominio_solape_prioridad_e_id` cubre S06 y el orden de S07 (solape, prioridad, id, `empate-por-id`).

Regresiones buscadas en el diff (huella del catalogo, `Rol` unico, refactor de orden.rs): no encontradas.

## Puerta y reproduccion

- Puerta: la peticion trae la rapida en verde en 2e2d33d; el commit revisado (1fc787b) solo anade estado de hallazgos. No se repitio.
- Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-085-r2-calidad` en 1fc787b con target propio.
- `cargo test -p tc2 --test seleccion --test plan`: seleccion 2 passed, plan 7 passed, 0 failed.
- Mutacion 1 (golden alterado): rojo en `tests/seleccion.rs`, como debe.
- Mutacion 2 (`rol()` deja de rechazar `agente`/`ronda` en diseno e implementacion): rojo en `tests/tarea.rs::roles_ampliados_ida_y_vuelta`.
- Mutacion 3 (huella del catalogo vuelve a incluir `ruta`): rojo en `seleccion::catalogo::tests::el_catalogo_no_depende_del_orden`.
- Todas restauradas; el repo original no se toco (se comprobo `git status` limpio).

## Notas (no abren ronda)

- `revision/mod.rs:504` hace `dominios_de(...).unwrap_or_default()`: en la revision de `seleccion_de` un error de tabla vuelve a tragarse, mientras `plan`, `tc2 seleccion` y la orden de revision lo propagan. Hoy no se dispara (la tabla del ADR esta embebida y `las_tablas_del_adr_se_leen` la cubre), pero es la unica via que mantiene el patron de MEN-3. MENOR.
- El camino `TABLA_ILEGIBLE` de `plan` y de `tc2 seleccion` no tiene test que lo ejerza (la tabla no se puede corromper desde fuera); mismo motivo.

## Regla 4D

- Duplicacion: cumple (dominios en `dominios_vivos`, heuristica una vez, un solo tipo `Rol`; `lista_o_ninguno` y `o_guion` reutilizados)
- Desviacion: cumple (la adenda del ADR dice lo no verificado: implementacion sin skills y muestra no comprobada por Carlos)
- Diluvio: cumple (un solo selector `seleccionar`, sin caminos paralelos tras quitar el respaldo de `roles_de`)
- Disfraz: cumple (el test de las 10 tareas ahora discrimina contra la tabla dorada, comprobado por mutacion; solo la nota del `unwrap_or_default` de revision, menor)

## Lecturas fuera del contexto permitido

- tareas/03-en-revision/TASK-085/evidencias/tabla-10-tareas-taskcodetwo.md: para comparar la tabla dorada con la de la evidencia.
- crates/tc2/src/revision/mod.rs y src/plan/operacion.rs completos alrededor del diff: para comprobar los caminos de error.

Veredicto propuesto: aprobada

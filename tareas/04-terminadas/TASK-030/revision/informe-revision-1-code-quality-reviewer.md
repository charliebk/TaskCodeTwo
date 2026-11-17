# Informe de revision — TASK-030 (ronda 1)

- Commit revisado: fc1f1887ff81feae206dda283f77e319fdfb12de (el clon quedo en 0fdce38, que solo anade la peticion de revision)
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/grafo.rs:355 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/siguiente.rs:110 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/src/estado/tareas.rs:13 | implementacion |
| MEN-4 | MENOR | abierto | crates/tc2/src/grafo.rs:3 | plan |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Alcance: pase completo (la peticion pide un unico dominio, el generico). Primer pase de la ronda, sin verificacion previa en verde en la peticion, asi que la suite la corri yo.

Puerta determinista: `cargo test --workspace` en un clon limpio de la rama: todo en verde (0 fallos; los `#[ignore]` se quedan ignorados). Test `#[ignore]` `toolsaweb_reales_se_leen_y_proyectan_como_la_fixture` con `TC2_TOOLSAWEB` apuntando a un clon de ToolsaWeb en `1fc94425964a5d33b5e2c3c4c39691dbd580c641`: 1 passed (los 404 `tarea.md` reales se leen y se proyectan igual que `tareas.tsv`).

### Criterios

- C1: `grafo::tests::toolsaweb_listas_y_en_marcha_como_su_roadmap` existe, compara el vector entero de 54 listas y `en_marcha` contra `esperado.txt`, y pasa. Fixture comprobada contra ToolsaWeb: la linea `- **Listas para empezar:**` de `git show 1fc94425:tareas/ROADMAP.md` es byte a byte la de `esperado.txt` (`cmp` igual) y `En marcha` tambien; `hitos.txt` tiene los mismos 19 titulos `### Hn — ...` y en el mismo orden que `docs/planificacion/hitos.md` de ese SHA (de ahi salen en el oraculo); `tareas.tsv` tiene 404 tareas, las mismas que `tarea.md` en ese SHA; `python -I scripts/grafo-tareas.py --comprobar` sale 0 en el clon. Mutacion: desempate por numero invertido (`u64::MAX - num`) en la clave de `orden_de_trabajo` pone en rojo ese test (y otros 4 del modulo).
- C2: un test por regla, todos existen y pasan: inexistente (`dependencia_inexistente_no_bloquea_y_avisa`), descartada (`dependencia_descartada_bloquea_y_avisa`), ciclo (`ciclo_deja_atascadas`, incluye autociclo y dependiente de ciclo), terminada de hito posterior (`dependencia_terminada_de_un_hito_posterior_retrasa`, con contraste sin retraso), sufijo y 4 cifras (`ids_con_sufijo_y_de_cuatro_cifras`), hito sin titulo (`hito_sin_titulo_va_detras_y_avisa`). Mutacion adicional: tratar un ilegible como terminado en `listas` pone en rojo `dependencia_ilegible_bloquea`.
- C3: `tests/siguiente.rs` (`next_en_es_y_en`, `next_json_valido`, `next_sin_red_usa_la_copia_del_clon`, `next_sin_rama_de_estado_es_rechazada`) existen, usan el binario real contra un remoto bare, comprueban stdout, stderr vacio y codigo, y que `next` no escribe (huella del clon y del bare antes y despues). Pasan. Prueba a mano abajo.
- C4: `siguiente::tests::ilegible_sale_en_ilegibles` (unitario) y la linea `ilegibles=TASK-005` del E2E. Mutacion: descartar el `Err` y no anadir el nodo ilegible pone en rojo el unitario. Pasa.

### Prueba a mano (binario `tc2` de la rama)

Repo con rama de estado local (sin remoto), 10 tareas: legibles, un ciclo, una dependencia inexistente, una descartada, un `tarea.md` roto y un `ROADMAP.md` con zona manual.

- es y en dan la misma linea: `next ok listas=TASK-002,TASK-004 en-marcha=TASK-003 commit=362e4d172809 ilegibles=TASK-009 ciclos=TASK-005,TASK-006 dependencias-inexistentes=TASK-004>TASK-999 depende-de-descartada=TASK-008>TASK-007`, codigo 0.
- `--json`: una linea, JSON valido, listas como array, claves con guiones.
- TASK-010 depende del ilegible TASK-009: no sale en `listas` (bloquea).
- Quitando el `ROADMAP.md`: se anade `hitos-sin-titulo=H0,H1`.
- Repo sin rama de estado: `next rechazada motivo=sin-rama-de-estado`, codigo 3, diagnostico en es (`monta la rama de estado en tareas/ primero`) y en en (`mount the state branch in tareas/ first`), tambien con `--json`.
- Argumentos de mas: `uso`, codigo 2.
- Sin red: cubierto por `next_sin_red_usa_la_copia_del_clon` (remoto inalcanzable, ve la foto del clon).

### Valoracion de las desviaciones

- Claves con guiones: correcto. El contrato F1-4 (`tests/comun::contrato`) manda sobre el texto de C3 y del plan; el criterio de la tarea ya esta corregido. Sin hallazgo (el plan, con `_`, es el que estaba equivocado: ver MEN-4).
- Un ilegible bloquea: correcto y mejor que tratarlo como inexistente (que lo dejaba como lista sobre algo que quiza no esta terminado). Esta cubierto por unitario, por mutacion y por el E2E (TASK-010).
- Aviso `duplicadas=`: razonable, coherente con "un fallo silencioso es peor que una caida"; cubierto por `id_repetido_avisa`. Sin hallazgo.
- Aviso `hitos-sin-titulo=`: razonable y comprobado por unitario, por el unitario de `siguiente` y a mano. Sin hallazgo.
- Diferencia con el oraculo (dependencia con etiqueta `descartada` y estado `terminada`): esta anotada en la tarea, no se da en el SHA de la fixture, y la regla D3 es la aprobada. Aceptable; solo falta un test que la fije (MEN-1).

### Hallazgos

MEN-1 (grafo.rs:355, `listas`). La diferencia conocida con el oraculo (descartada y terminada a la vez en formato 1 bloquea) no tiene test propio: C2 prueba "dependencia descartada bloquea" con estado `descartada`, pero en `Nodo::de` formato 1 un `etiquetas: [descartada]` con `estado: terminada` pasa por la rama `Fase::Descartada` y la unica prueba de eso es `nodo_de_los_dos_formatos` con estado `planificada`. Sin efecto en el resultado normal. Sugerencia: un caso en `nodo_de_los_dos_formatos` con `estado: terminada` y etiqueta `descartada` que compruebe `Fase::Descartada`.

MEN-2 (siguiente.rs:110, `calcular`). Los IDs de los ilegibles se toman de la carpeta (`ruta.split('/').next()`), que `es_tarea_md` acepta en minuscula (`task-013/tarea.md`). Si hay un ilegible en `task-013/`, su ID queda `task-013`, y quien dependa de `TASK-013` no lo encuentra: lo ve como inexistente (no bloquea, y se avisa como `dependencias-inexistentes=`) en vez de bloqueado. No verificado: no pude crear la carpeta en minuscula en un checkout de Windows (git mv dio `Invalid argument`); el caso a ejecutar seria una rama de estado con `task-013/tarea.md` roto y `TASK-014` con `dependencias: [TASK-013]`; esperado hoy: TASK-014 en `listas=`. Caso de borde sin efecto en el resultado normal (los `tarea.md` reales usan `TASK-`).

MEN-3 (estado/tareas.rs:13 y grafo.rs:`en_marcha`). Una tarea `en-curso` que esta en un ciclo (o depende de uno) queda en `atascadas`, fuera de `orden`, y por tanto `en_marcha` no la lista aunque alguien la lleva entre manos; solo sale en `ciclos=`. Reproducible por lectura de `en_marcha` (filtra sobre `o.orden`); no hay test que fije el comportamiento deseado. Poco probable (ToolsaWeb exige grafo sin ciclos por `--comprobar`) y el aviso `ciclos=` lo hace visible. Decidir si `en_marcha` debe mirar `nodos` y no `orden`, y fijarlo con un test.

MEN-4 (grafo.rs:3, plan y criterio C3 de la tarea). El plan decia claves con `_` (`en_marcha=`, `dependencias_inexistentes=`) y el criterio C3 lo repetia; la implementacion se ajusto al contrato F1-4 con guiones. Los comentarios del modulo y del plan-final todavia hablan de `dependencias_inexistentes=` y `depende_de_descartada` en las lineas de plan; el codigo y `tarea.md` ya dicen guiones. No bloquea; conviene dejar el plan coherente o dejar la nota de la desviacion (ya esta en `## Notas de implementacion`).

## Lecturas fuera del contexto permitido

- `crates/tc2/src/grafo.rs`, `crates/tc2/src/siguiente.rs`, `crates/tc2/tests/siguiente.rs`, `crates/tc2/src/estado/tareas.rs` y `crates/tc2/src/estado/lectura.rs`: son el codigo que se revisa, necesarios para reproducir C1..C4 y las mutaciones.
- ToolsaWeb (solo lectura, en un clon temporal en `1fc94425`): `tareas/ROADMAP.md`, `docs/planificacion/hitos.md`, `scripts/grafo-tareas.py`, para comprobar la fixture contra su origen.

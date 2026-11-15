# Informe de revision — TASK-022 (ronda 1)

- Commit revisado: 304782b8e59e4293ddbdd512d739b3979a0ac971
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/plataforma/gitlab.rs:552 | implementacion |
| MEN-1 | MENOR | corregido | crates/tc2/src/plataforma/errores.rs:94 | implementacion |
| MEN-2 | MENOR | corregido | crates/tc2/src/plataforma/gitlab.rs:100 | implementacion |
| MEN-3 | MENOR | corregido | crates/tc2/src/plataforma/gitlab.rs:741 | plan |
| MEN-4 | MENOR | corregido | crates/tc2/src/plataforma/gitlab.rs:489 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

## Puerta y alcance

- Pase generico completo sobre el adaptador GitLab (`gitlab.rs`, `conexion.rs`, `errores.rs`, cambios de `http.rs`, `mod.rs`, `puertas.rs`, `tests/plataforma_gitlab.rs`, fixtures). Profundidad: `gitlab.rs` y sus tests a fondo; `conexion.rs`, `errores.rs` y `http.rs` a fondo en lo que toca GitLab; `github.rs` solo para contrastar.
- Clon en carpeta temporal, en el commit 304782b, con su propio `CARGO_TARGET_DIR`. Suite `cargo test -p tc2`: todos los binarios en verde (41 unitarios, `plataforma` 4/4, `plataforma_gitlab` 3/3, `tarea` 5 y 1 ignorado, E2E ignorados sin ejecutar).
- Una primera pasada, solapada con otra ejecucion, dio dos fallos en `tests/plataforma.rs` (`el_envoltorio_rechaza_antes_de_la_red` y `errores_dirigidos_a_la_persona`) con `sinred`: es el atasco intermitente de red local indicado en la peticion. Repetido, `plataforma` pasa 4/4. No es hallazgo.
- Mutantes sobre `gitlab.rs` con `cargo test -p tc2 --test plataforma_gitlab` (10): matan el test el filtro de autor de la aprobacion, el origen del check, el sha de `merge-pr`, la rama Premium, `has_conflicts`, `diverged_commits_count`, `verified`, y `tiene_commits`. `canceled` como fallo lo cubre el unitario de `src` (`solo_success_es_verde`). Sobrevive uno: ver MEN-4.

### IMP-1 — `estado-puertas` da `ok` mientras GitLab aun calcula la mergeabilidad
- Donde: crates/tc2/src/plataforma/gitlab.rs:552 (`sin_conflictos: !m.has_conflicts`, y `MrPuertasApi` en :142)
- Que pasa: el adaptador toma `has_conflicts == false` como «sin conflictos». El propio E2E lo dejo anotado (E2E-GL-07, `observado-e2e.json`): «has_conflicts pasa a true en segundo plano (hace falta repetir la lectura)». Mientras `detailed_merge_status`/`merge_status` valen `checking` o `unchecked`, `has_conflicts` aun es `false`. El adaptador ignora esos campos y no repite la lectura, asi que un MR recien abierto, o con la rama destino recien movida, puede dar `estado-puertas ok` con conflictos reales. Los fixtures traen `detailed_merge_status` (siempre `mergeable`), pero el struct no lo lee y ningun test lo toca.
- Reproduccion: en el clon, test `rev_merge_status_pendiente` (copia de `plataforma_gitlab.rs` con el fixture `verde` y `mr_de(f)["detailed_merge_status"]="checking"`, `merge_status="checking"`, `has_conflicts=false`). Salida: `estado-puertas ok pr=12 aprobaciones=1 ignoradas=0`. Esperado: `esperando ... falta=sin-conflictos` (o un reintento). La parte de la plataforma real (que GitLab devuelva `checking` justo despues de abrir) no se ha ejecutado aqui (sin permiso de red a GitLab): sale de la observacion del E2E de la tarea (no verificado por mi).
- Impacto: la linea `ok` es la que usa quien orquesta para decidir que el MR esta listo; `merge-pr` falla cerrado despues (405 `no-mergeable`), asi que no se mergea un conflicto, pero la linea ya no es la misma que daria GitHub (RR-15) y el diagnostico falso «todo en verde» llega a la persona. Arreglo sugerido: leer `detailed_merge_status` y tratar `checking`, `unchecked` y `preparing` como `falta=sin-conflictos` (con diagnostico de reintentar), mas un test con ese valor.

### MEN-1 — Un 401 en `merge-pr` se diagnostica como token caducado
- Donde: crates/tc2/src/plataforma/errores.rs:94
- Que pasa: `traducir` manda todo 401 a `motivo=credenciales` («el token ... no vale o ha caducado: renueva TC2_...»). Segun la documentacion de la API de GitLab (de memoria, no he podido abrirla: la herramienta de acceso web fallo), el `PUT .../merge` responde 401 cuando el usuario no tiene permiso para mergear en esa rama (un Developer ante una rama protegida solo Maintainer, justo G01). El E2E uso una cuenta propietaria y no lo ejercito; los tests solo prueban 401 sobre `identidad`.
- Reproduccion: no verificado. Caso a ejecutar: con un token de rol Developer, `merge-pr` sobre un MR en `develop` protegida con merge=40. Esperado en el adaptador: `denegada motivo=permisos` con `PERMISO_MERGE`; si GitLab da 401 saldra `credenciales`.
- Impacto: mensaje que manda a renovar un token valido. Mitigacion barata: en `merge_pr`, un 401 tras haber leido el MR con ese mismo token se traduce como permisos.

### MEN-2 — La lectura de `statuses` no pide `all=true`
- Donde: crates/tc2/src/plataforma/gitlab.rs:100
- Que pasa: sin `all=true`, GitLab devuelve solo el ultimo estado por nombre (y por ref/stage), no todos. El codigo luego ordena por id y busca el ultimo con el autor de origen, como si tuviera el historial. Un estado del mismo nombre publicado despues por otra cuenta oculta el del origen y el check queda `pendiente` (falla cerrado, no se salta nada), pero el rojo del origen tambien se pierde (`falta=` en vez de `check-rojo`).
- Reproduccion: la lista de llamadas del clon muestra `GET .../statuses?per_page=100` sin `all` (salida `REV2` del test de comprobacion). El efecto en la plataforma real no se ejecuto (sin acceso a GitLab); marcado como no verificado.
- Impacto: diagnostico menos preciso, nunca un verde falso. Anadir `&all=true` o corregir el comentario de :510.

### MEN-3 — Ids de clave SSH y GPG en un mismo conjunto
- Donde: crates/tc2/src/plataforma/gitlab.rs:741 (y `Personas::con_claves` en mod.rs)
- Que pasa: `clave = key.id (SSH) o gpg_key_id`, y `Personas.claves` es un unico `BTreeSet<u64>`. Son tablas distintas de GitLab con numeraciones independientes: el id 5 de una clave GPG de un bot y el id 5 de la clave SSH de una persona son indistinguibles, y `cuenta_como_aprobacion` daria `true` para el commit del bot. Lo decidio el plan (decision 4) y el ADR habla de «ids de clave», sin tipo.
- Reproduccion: no verificado en GitLab; en el codigo basta `Personas::nuevas([P]).con_claves([5])` y una `Firma { verificada: true, clave: Some(5), .. }` desde una clave GPG: `cuenta_como_aprobacion` es `true`.
- Impacto: colision poco probable pero posible, y su efecto es contar como persona a quien no lo es. Corregirlo en F2-13 (la lista de personas con el tipo de clave) o anotarlo en el ADR como limite conocido.

### MEN-4 — Proteccion G01 sin test que la mate
- Donde: crates/tc2/src/plataforma/gitlab.rs:489 (`p.merge_access_levels.len() == 1`)
- Que pasa: es la unica comprobacion que impide que una lista vacia `merge_access_levels: []` pase por el `all(== MAINTAINER)` (cierto en vacio) y se tome por G01 cumplida. Mutante M2 (quitar `len() == 1`): `plataforma_gitlab` sigue 3/3 en verde. El test de G01 solo prueba 404, `allow_force_push` y un nivel 30.
- Reproduccion: `sed` de la condicion en el clon y `cargo test -p tc2 --test plataforma_gitlab`: `test result: ok. 3 passed`.
- Impacto: sin red de regresion para la proteccion; el plan pedia `merge_access_levels != [40]`. Anadir los casos `[]` y `[40, 40]` al test.

## Lecturas fuera del contexto permitido

- crates/tc2/src/plataforma/gitlab.rs, conexion.rs, errores.rs, mod.rs, puertas.rs: codigo bajo revision (el diff solo embebia fixtures).
- crates/tc2/src/plataforma/http.rs y deteccion.rs (diff contra develop): cambios de envoltorio de `%2F` y prefijo.
- crates/tc2/tests/plataforma_gitlab.rs: tests que cubren lo anterior (mutantes).
- tareas/03-en-revision/TASK-022/evidencia/e2e-gitlab.log: evidencia del E2E que cita la peticion.
- docs/adr/F1-13-adaptador-de-plataforma.md: no lo abri; el contraste con el ADR se hizo con las citas del plan-final.

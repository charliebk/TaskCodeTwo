# Plan — TASK-142: Auditoria 4D: A4 git, rama de estado y toma (9 IMPORTANTE)

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 270 min

Señales: 8 ficheros de producción (`git/{rama,remoto,tag,estado,mod,guardar,puerto}.rs`, `estado/transaccion.rs`), 5 a 7 de tests, 2 capas (git y estado), 8 criterios, 2 `divergente` con cambio de comportamiento, un D4 con test primero y `cargo xtask verificar` completo.

**Fichas vivas (develop 54d515f):** las 8 de la tarea (A4-01 `rama.rs:88-110`/`:160-196`, `remoto.rs:175-181`, `tag.rs:100-114`; A4-02 `estado.rs:243-265`/`:269-298`/`:135-150`, `mod.rs:54-91`; A4-03 `transaccion.rs:353-371`/`:533-551`; A4-04 `tag.rs:81-92`, `remoto.rs:516-527` y `push_paso`/`push_fallo`; A4-05 `estado.rs:451-473`; A4-06 `rama.rs:318-380`/`:388-456`; A4-07 `guardar.rs:158-176`/`mod.rs:218-230`; A4-10 `puerto.rs:613-646`). A4-08 no está resuelto (`rapida`, `escribir_estado`, `rama_en` y el repo sin remoto siguen copiados en `estado_bordes`, `estado_casos`, `estado_concurrencia`, y hay copias idénticas en `frescura.rs:416-419` y `roadmap.rs:259-262`). A4-09 se fusionó en TASK-148.

**Orden** (un commit por ID):
1. A4-01 (divergente): `remoto::fallo_fetch` pasa a `pub(crate) fallo_del_remoto(op, &SalidaGit)`, única clasificación; nace `pub(crate) consultar_remoto(git, dir, op, args) -> Paso<String>` (`ls-remote` + clasificación). En `rama.rs`, una sola `ramas_de_tarea(git, principal, op, patron)` (mira `hay_remoto`, `consultar_remoto`, `branch --list`, `nombres`), usada por `ramas_del_id` e `ids_con_rama`. `tag::commit_remoto` usa `consultar_remoto` sin mirar `hay_remoto`. **Cambia:** con un remoto que deniega, `comprobar-id` y `tag-anotado` pasan de `sinred` (4) a `denegada motivo=…` (8) o `error motivo=error-local`; sin `origin`, `comprobar-id` sale `ok`/`rechazada` con lo local (decisión 1).
2. A4-02 (divergente): trait `pub(crate) Salida` (`codigo`, `stderr`) para `SalidaGit` y `SalidaCruda`; la primera línea de error en una función libre (la API de `SalidaGit::primera_linea_error` no cambia); `lanzado(op, sub, r)` y `exigida<S: Salida>(op, sub, r)` en `mod.rs`; `git_fallo` recibe `&impl Salida`; `correr`, `exigir`, `con_indice`, `con_entrada`, `leer_blobs` y el `map_err` de `construir_commit_sellado` (decisión 4) quedan en una línea. **Cambia:** el `error=` del diagnóstico de `leer_blobs` (`estado.rs:148`) pasa de la primera línea cruda a la primera no vacía y recortada (o `sin mensaje`).
3. A4-05: `hay_remoto` = `Ok(url_origin(git, dir, op)?.is_some())`.
4. A4-07: un solo analizador `registros_status(salida) -> Vec<(xy, ruta, Option<origen>)>` en `mod.rs`; `entradas_status` es su proyección; `guardar::entradas_con_origen` se queda como proyección que añade `("D ", origen)` en los `R`.
5. A4-03: `fn aplicar_en(vista, op) -> Result<Option<Intento>, Informe>` en `transaccion.rs`.
6. A4-04: `pub(crate) informe_publicacion(op, p, clave, texto)` en `remoto.rs` para `tag_paso`, `actualizar` y las ramas de fallo de `push_paso` (desaparece `push_fallo`); mismo orden de claves, ninguna línea cambia.
7. A4-06: `struct DestinoWt { rama, principal, wt, rel }` con `preparar`, `rechazo`, `carpeta_ocupada` y `anadir` (carpeta y `anadir` separados para conservar el orden de `crear`, donde `base-no-existe` va entre las dos).
8. A4-08: `rapida()` y `escribir_estado()` a `tests/comun/ops.rs`; `rama_en` y `repo_sin_remoto` como métodos de `Fixture`; se migran los tres ficheros de la ficha y `frescura`/`roadmap` (decisión 3).
9. A4-10 (D4): commit 1, las tres filas de la ficha en `rechaza_cada_argumento_prohibido_sin_lanzar_git`, rojo contra el código actual (`Err(ErrorGit::Lanzar(_))`); commit 2, `es_prohibido` despliega cada argumento `-xyz` (un guion) en `-x`, `-y`, `-z` y rechaza si alguno es prohibido (cierra ante la duda); se corrige el comentario de `puerto.rs:555-556`; con el mutante (sin desplegar), rojo.

**API pública:** no cambia; todo lo nuevo es `pub(crate)` o privado.

## Reutilizacion (Regla 4D)

Lo que ya existe y se usa (helpers, funciones, fixtures, tests), buscado en el repo antes de disenar; y lo que se crea nuevo, con el motivo de no reutilizar:

- Se reutiliza: `remoto::fallo_fetch`, `sinred`, `error_local`, `denegada`, `clasificar_fallo_remoto`; `rama::nombres`, `estado::url_origin`, `mod::entradas_status`, `git_fallo`; `SalidaGit::primera_linea_error`, `remoto::que_hacer_al_publicar`, `Publicacion`. Tests: `comun::{Fixture, escribir, contrato, ops::catalogo}`, `Fixture::crudo`, el escenario «origin a una ruta que no es repo» de `tests/git_push.rs:115-131`, `git_sin_binario`. El git falso de `estado.rs::git_que_pierde_shas` se generaliza en `git_falso(d, guion_cmd, guion_sh)`.
- Nuevo (forma única de lo duplicado): `consultar_remoto`, `ramas_de_tarea`, `Salida`, `lanzado`, `exigida`, `registros_status`, `aplicar_en`, `informe_publicacion`, `DestinoWt`.
- Descartado: el analizador de opciones de `hallazgos/orden.rs` (opciones del CLI de tc2, no de git); `crate::lectura` y `plataforma/operaciones.rs` (otra zona).
- Fuera a propósito (forma distinta, se documentan en `## Resultado`, decisión 5): `init/inventario.rs:463-477`, `estado/montaje.rs:139-147`, `toma::carpeta_libre`.

## Lo que el rol no cubrio

- Riesgos: stderr reales de `ls-remote` que ahora se clasifican (la clasificación es la de `fetch`); falsos positivos de la guarda desplegada más allá de los citados.
- Testing: si la suite detecta todos los mutantes de las extracciones sin cambio de comportamiento (A4-03, A4-04, A4-06).
- Dominio: si «`comprobar-id` sin `origin` = `ok`» respeta RN-05 y F1-3 (decisión 1).

### Suposiciones no verificadas
- Que algún test fija ya la línea exacta de `actualizar-rama` cuando su publicar falla (si no, se añade uno).
- Que `ls-remote` contra una ruta inexistente da en stderr «does not appear to be a git repository».

## Riesgos aceptados y que los contiene

- Con A4-02 cada llamada a git pasa por `lanzado`/`exigida`: lo contiene la suite completa; ningún informe cambia salvo `leer_blobs`.
- Cambios de comportamiento limitados a `rama.rs:97-101`, `tag.rs:110-114`, la falta de `origin` en `comprobar-id` y `estado.rs:148`; cada uno con su test.
- La guarda desplegada rechaza también formas legítimas con valor pegado (`-mfoo` → `-f`, `-qm <msg>`); ningún llamador de la capa las usa (buscado en `src`); falla cerrando.
- Coste: A4-01 añade un `ls-remote --get-url` local por cada `comprobar-id`.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- A4-01 (2): en `tests/git_push.rs`, bloque «remoto que no es un repositorio», `comprobar_id` y `tag_anotado` → `denegada motivo=remoto-no-existe` (8), hoy `sinred` (rojo); `comprobar_id` sin `origin` → `ok`.
- A4-02 (1): unitario en `estado.rs` con `git_falso`: `cat-file` con código 1 y stderr `"\nfatal: x"`; el diagnóstico de `leer_blobs` contiene `fatal: x` (hoy vacío: rojo).
- A4-05 y A4-07 (1): `status_z_con_renombrado_y_espacios` con una entrada `C`; unitario de `entradas_con_origen` (`R` añade `D origen`, `C` no).
- A4-03, A4-04, A4-06 (0): extracciones; las protegen `estado_casos`, `git_push` y `git_ciclo`.
- A4-08 (0): tests migrados en verde.
- A4-10: tres filas en el test existente (rojo hoy) y una fila permitida (`Push`, `["-qv", "--porcelain", "origin", "a:b"]`) en `mensajes_y_rutas_no_son_opciones`; rojo con el mutante.
- Pesados: `estado_concurrencia`, `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar`, sin solapar.

## Lo que necesita decision de una persona

1. A4-01: `comprobar-id` sin `origin` mira `hay_remoto` y sale `ok`/`rechazada` con lo local. Recomendado.
2. A4-10: desplegar los grupos cortos (cierra ante la duda) y nombrar en `## Resultado` que la desviación del ADR F1-4 D3 (lista de prohibidas, no permitida) se mantiene. Recomendado.
3. A4-08: migrar también las copias de `tests/frescura.rs:416-419` y `tests/roadmap.rs:259-262`. Recomendado.
4. A4-02: incluir el `map_err` de `construir_commit_sellado`. Recomendado.
5. Observaciones fuera de alcance: solo documentarlas en `## Resultado`. Recomendado.

**Respuestas de la persona (Carlos, consigna del 2026-11-23: modo automatico con la opcion recomendada):** se aplican las recomendaciones 1 a 5.

**Regla de una hora:** 270 min, sin trocear: Carlos descartó trocear las tareas de corrección 4D (consigna del 2026-11-23).

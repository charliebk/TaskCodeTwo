# Plan — TASK-052: F6-6 Rondas incrementales

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Cada check `revision-ia` describe su propia ronda y lleva todos los hallazgos que siguen abiertos. La ronda siguiente solo necesita el check anterior, publicado por la App del revisor (no la rama de estado, que escribe el agente). Como en F6-4 y F6-5, `tc2` decide con funciones puras y el workflow transporta.

En `preparar`, el workflow recorre los commits del PR del mas nuevo al mas viejo con el `GITHUB_TOKEN` (`checks: read`, lectura ya admitida por la lista blanca de F1-8) y vuelca el ultimo check de la App del revisor que no sea `incompleta`. Con ese volcado, `tc2 hallazgos ronda` calcula: numero de ronda; modo `completa` (ronda 1), `incremental`, `ligera` (solo MENOR, sin modelo) o `tope` (sin modelo); SHA anterior; y abiertos heredados.

En modo `incremental` cada revisora recibe el diff `sha_anterior..head` de sus rutas y los abiertos que le tocan, y contesta con hallazgos nuevos y `cierres` explicitos. Un abierto solo se cierra con un cierre valido; si no, sigue abierto (falla en cerrado). `veredicto` cuenta `nuevos`, `persisten` y `cerrados` y los deja en titulo, resumen, anotaciones e informe de la carpeta de la tarea.

Duracion estimada: 260 min

Senales: ~12 ficheros de codigo (`hallazgos/mod.rs`, `hallazgos/orden.rs`, nuevo `hallazgos/ronda.rs`, esquema y fixtures, `seleccion/orden.rs`, `revision-ia.yml`, `xtask/src/workflows.rs`, `plataforma/github.rs`, `revision/mod.rs`, catalogo i18n, guia y adendas), 4 capas (`tc2` puro, workflow+lint, puerto de plataforma, `review` con rama de estado), 2 criterios. Precedentes: TASK-051 estimada 220 / real ~130; TASK-050 140 / 169.

### Piezas y limites

- `crates/tc2/src/hallazgos/ronda.rs` (nuevo, puro):
  - `Anterior { ronda, sha, dictamen, abiertos: Vec<AceptadoDe> }` desde el check-run volcado y sus anotaciones, validado con `dictamen_publicado` (la misma regla de `leer-revision`).
  - `decidir(anterior, rutas_cambiadas, tope_efectivo) -> Decision { ronda, modo, sha_anterior, abiertos }`.
  - `repartir(abiertos, detalle) -> Vec<Vec<AceptadoDe>>`: cada abierto a su `revisora` si esta elegida; si no, a la que tiene su `fichero` entre sus rutas.
- `crates/tc2/src/hallazgos/mod.rs` (contrato v3, `VERSION` = 3):
  - El bloque admite la lista de v2 (ronda 1) o el objeto `{hallazgos, cierres}`.
  - `Cierre { id, fichero, linea, motivo }` anclado al lado nuevo del diff incremental; si el abierto traia evidencia `paso_rojo`, ese paso debe haber dejado de estar en rojo en el informe F5-4 del head nuevo (que `validar` ya recibe).
  - Motivos nuevos: `cierre-desconocido`, `cierre-fuera-del-diff`, `paso-rojo-sigue-rojo`.
  - `AceptadoDe` gana `ronda` y `estado` (`abierto|cerrado`) con `#[serde(default)]` (los `raw_details` v2 se siguen leyendo).
  - `Veredicto` gana `ronda`, `nuevos`, `persisten`, `cerrados`, `ligera`; `Dictamen` gana `Tope` (`failure`).
  - Titulo `revision-ia: <dictamen> (ronda=R criticos=N importantes=M menores=K)`; `dictamen_publicado` acepta tambien el titulo v2 (ronda 1) y cuenta solo los `abierto`.
  - `veredicto(revisoras)` queda como envoltorio de `veredicto_ronda(revisoras, Option<&Decision>)`; `validar` gana `abiertos: &[AceptadoDe]`; `anotaciones()` emite persistentes y, con nivel `notice`, los cerrados en esta ronda; `InformeTarea` gana los campos de ronda.
- `crates/tc2/src/hallazgos/orden.rs`:
  - `tc2 hallazgos ronda --check-runs <json>... --anotaciones <json>... --rutas <nul> [--tarea TASK-NNN] --salida <dir>`: escribe `ronda.json` (`$defs/ronda`) y la linea `hallazgos ok ronda=R modo=<m> anterior=<sha12|ninguno> abiertos=N tope=T`. Tope efectivo = `tope_rondas` de la config de la base (`config::de_base`) + elementos de `## Rondas extra` de la tarea en `taskcodetwo/tareas` (como `tc2 revisoras --tarea`).
  - `tc2 hallazgos abiertos --ronda <json> --detalle <json> --salida <dir>`: por revisora `abiertos-<i>.json` y `abiertos-<i>.md` (por `plano`, para el prompt).
  - `validar --abiertos`, `veredicto --ronda`; `instrucciones` gana la seccion de cierres desde las mismas constantes.
- `crates/tc2/src/seleccion/orden.rs`: `tc2 revisoras --ronda N` (hoy fija a 1 en `orden.rs:394`), con lo que S13 se aplica (adenda F6-3 de F1-15). Rutas = diff incremental + ficheros de los abiertos.
- `crates/tc2/src/plataforma/github.rs`: el filtro «el mas nuevo de `revision-ia` de la App `origen`, terminado» (`github.rs:633-639`) pasa a funcion pura de `hallazgos`, compartida por `leer_revision` y `tc2 hallazgos ronda`.
- `.github/workflows/revision-ia.yml` (sin jobs nuevos):
  - `preparar` gana `checks: read` y el paso «Ronda anterior»: `git rev-list --first-parent origin/<base>..<head>` sin el head, tope 100 commits; por commit `gh api .../commits/<c>/check-runs?check_name=revision-ia` volcado en crudo (decide `tc2`); del elegido, sus paginas de anotaciones.
  - `modo=incremental`: diff `<sha_anterior>..<head>` por revisora y seccion «Hallazgos abiertos de la ronda anterior» en el prompt. `ligera`/`tope`: sin prompts, `lista.tsv` vacia, `consultar` no llama al modelo.
  - `validar` pasa `--abiertos` y `--ronda`; `publicar` no cambia. El llamador de la guia gana `checks: read`.
- `xtask/src/workflows.rs`: perfil `revision-ia` (`preparar` corre `tc2 hallazgos ronda` y `tc2 revisoras --ronda`; `validar` pasa `--ronda`); perfil `llamador-revision` admite `checks: read` y nada mas nuevo; sigue un solo `success` en `publicar`.
- `crates/tc2/src/revision/mod.rs`: antes del push, si `prs_abiertos(Rol::Agente)` da un PR de la rama con `revision-ia` terminado en su head, `review` guarda su informe con `GuardarRevision` (hoy solo despues de empujar, cuando el check anterior ya se pierde). La linea gana `ronda=R nuevos=N cerrados=C` y `hallazgos=tope`.
- Docs: `docs/ci/revision-ia.md` seccion «Rondas (F6-6)»; adendas F6-6 en F1-15 (S13 activa, titulo con `ronda`), F1-8 (`checks: read` en `preparar` y llamador; ejecucion de la evidencia, decision 3), F1-10 y F1-16 (`tope` en CI frente a la parada `preguntar` y `## Rondas extra`).

### Orden de construccion

1. Contrato v3 puro: `Cierre`, campos nuevos de `AceptadoDe` compatibles con v2, `Dictamen::Tope`, `veredicto_ronda`, titulo y `dictamen_publicado` con los dos formatos, esquema. Con fixtures, la cadena ronda 1 -> ronda 2 sin GitHub.
2. `hallazgos/ronda.rs` (`decidir`, `repartir`, filtro compartido con `github.rs`), ordenes `ronda` y `abiertos`, `validar --abiertos`, `veredicto --ronda`, `instrucciones`, catalogo i18n.
3. `tc2 revisoras --ronda`.
4. Workflow (`preparar`, `validar`), llamador de la guia y lint.
5. `review`: informe del head anterior antes del push y linea con la ronda; guia y adendas.

Cada paso deja el arbol en verde. Pasos 1-2 cumplen «hallazgos nuevos por ronda, medidos» en `tc2`; 3-5, F6-6 en CI y en la carpeta de la tarea.

### Alternativas descartadas

- Leer la ronda anterior del informe `revision-ia-<sha12>.json` de la rama de estado: la escribe el agente; si borrara un IMPORTANTE, la ronda siguiente no lo miraria (F1-8). El check lo publica la App del revisor y se filtra por `app.id`.
- Cierre por silencio o por tocar la linea: falla en abierto (un olvido del modelo deja escapar un IMPORTANTE). Se exige cierre explicito anclado al diff.
- Que `tc2` llame a la API en CI por el puerto con el `GITHUB_TOKEN` como rol agente: mezcla identidades (F1-6) y mete red en `hallazgos`, modulo puro.
- Revisar el diff completo en cada ronda: incumple «mira lo cambiado» y gasta tokens (RR-07).

## Troceo

Pasa de 60 min (260). **La persona decidio no trocear**: consigna de Carlos para el H6 (2026-11-21), como en TASK-049, TASK-050 y TASK-051. El orden de construccion deja la regla de rondas probada en `tc2` antes de tocar el workflow, y este antes de `review`; con solo los pasos 1-2 el workflow actual sigue dando ronda 1.

## Lo que el rol no cubrio

- Criterio 1 («medidos»), por donde se rompe (rol de riesgos, no revisado): una revisora que reporta un abierto con otro `id` infla «nuevos»; una linea persistente desplazada deja la anotacion en el sitio equivocado o la API la rechaza; PR con mas de 100 commits en la ronda 1; rebase o force-push que deja el SHA anterior fuera de la historia (decision de arquitectura: si `sha_anterior` no es ancestro del head, el diff vuelve a `base...head` y los abiertos se arrastran igual).
- Criterio 2, reglas de dominio no revisadas: que una ronda `incompleta` no cuente ni sirva de base; que la `ligera` se dispare con anterior `verde` y cambios solo en ficheros con MENOR abiertos; formato de `## Rondas extra` (un elemento de lista por ronda extra); que los MENOR no pasen por el cierre del modelo. Son lecturas de arquitectura de RF-33, F1-10 y F1-16.
- Ejecucion de la evidencia (decision 3 de F1-8, adenda F6-4 «F6-6 la ejecuta»): no entra (ver decision 3). Solo se contrasta `paso_rojo` con el informe F5-4. RR-03 se cumple solo en parte.
- La estrategia de pruebas no la reviso el rol de testing.
- GitLab: la revision en CI solo existe en GitHub.
- Inyeccion en los abiertos reinyectados al prompt: pasan por `plano`; la defensa es F6-7.
- La seleccion anotada en local (`roles.revision` en `review`) sigue con `ronda: 1`; la fuente de la ronda es el check de CI.

## Riesgos aceptados y que los contiene

- Sin saldo en la API la cadena real no pasa de la ronda 1 (`incompleta` no cuenta). Lo contienen: E2E con checks publicados por la App desde fixtures, `ligera` y `tope` sin modelo, y que es lo honesto.
- Permiso nuevo en el llamador (`checks: read`): sin el, `preparar` falla y el check sale `incompleta` (falla en cerrado). Lo contienen la guia, el lint `llamador-revision` y que ya era lectura admitida (`buscar` de F5-2).
- Coste de la ronda 1: hasta una llamada a la API por commit del PR (tope 100); pasado el tope, `incompleta`.
- El agente podria escribir `## Rondas extra`: solo gana rondas (tokens), no salta puertas; la aprobacion de la persona sigue siendo obligatoria (F1-16).
- Cambio de formato del titulo: `dictamen_publicado` lee los dos y el E2E de TASK-051 sigue valiendo.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

Criterio «Hallazgos nuevos por ronda, medidos» (3 tests):
1. `crates/tc2/tests/hallazgos.rs` con binario real y fixtures: la ronda 1 da `check.json` con 1 IMPORTANTE y 1 MENOR; `tc2 hallazgos ronda` sobre ese check (volcado como la API) da `ronda=2 modo=incremental abiertos=2`; respuestas de ronda 2 con un cierre valido del IMPORTANTE y un hallazgo nuevo: `veredicto --ronda` da titulo `ronda=2` con `nuevos=1 cerrados=1 persisten=1` y el cerrado como anotacion `notice`. Un cierre fuera del diff incremental se rechaza (`cierre-fuera-del-diff`) y el IMPORTANTE sigue abierto (rojo).
2. `crates/tc2/tests/revision.rs` con remoto bare (Git real) y plataforma doble con `prs_abiertos` y `leer_revision`: `review` repetido guarda el informe del head anterior antes de empujar, con `ronda` y `nuevos`; quedan dos `revision-ia-<sha12>.json` (ronda 1 y 2) y la linea lleva `ronda=`.
3. **Pesado** (E2E real, `#[ignore]`, red y App del revisor 5846307) en `charliebk/taskcodetwo-e2e-github`, sin el modelo: en el commit A de un PR temporal se publica con la App el `check.json` de ronda 1 (fixture rojo); se empuja B y corre el workflow; el artefacto `revision-ia-prompts` trae `ronda.json` con `ronda=2`, `sha_anterior` = A y diff `A..B`; `consultar` sin saldo da `incompleta`. Limpia PR y rama. Comprueba que el `GITHUB_TOKEN` con `checks: read` lee el check de la App. Va en `## Resultado`; no se lanza con cargo.

Criterio «Cumple F6-6 y lo que cita (RF-33)» (3 tests):
1. Unitarios de `hallazgos::ronda::decidir`: anterior rojo y ronda por encima de `tope_rondas` da `modo=tope` sin prompts y con abiertos arrastrados (`Dictamen::Tope`, `failure`); un elemento en `## Rondas extra` sube el tope; anterior verde con cambios solo en ficheros de MENOR da `ligera` (verde, la ronda no sube); un cambio fuera de ellos da `incremental`; un anterior `incompleta` no cuenta.
2. Test de esquema ampliado: `cierre`, `ronda`, `informe-tarea` v3 y motivos nuevos coinciden entre esquema y codigo; un `raw_details` v2 y un titulo v2 se siguen leyendo; la salida de `ronda` y `veredicto --ronda` valida contra el esquema.
3. Lint ampliado de `xtask/src/workflows.rs`: `preparar` con `checks: read`, corre `tc2 hallazgos ronda` y pasa `--ronda` a `revisoras`; `validar` pasa `--ronda`; el llamador admite `checks: read` y rechaza `checks: write`. Una mutacion por regla da rojo.

Pesado: solo el E2E (criterio 1, test 3). El de `revision.rs` usa Git real en local, como TASK-048 y TASK-051.

## Lo que necesita decision de una persona

Todas decididas con la opcion recomendada, delegada por Carlos (2026-11-22); sujetas a `taskctl approve`.

1. Fuente de la ronda anterior: el check `revision-ia` de la App del revisor en el commit anterior del PR, leido en `preparar` con `checks: read` (tambien en el llamador). No el informe de la rama de estado.
2. Cierre: falla en cerrado. Solo cierra un `cierre` explicito anclado al diff incremental; con evidencia `paso_rojo`, el paso debe estar ya en verde en el informe F5-4. Los MENOR no pasan por cierre.
3. Ejecucion de la evidencia (adenda F6-4 de F1-8, «F6-6 la ejecuta»): no entra en F6-6. Exigiria un comando de test por proyecto (clave nueva de F1-11), un job que ejecute codigo del PR con la mutacion del modelo y saldo para probarlo; ademas pasa de la fila F6-6. La adenda F6-6 en F1-8 lo dice y queda como accion de Carlos crear una fila F6-x en el plan. Mientras, RR-03 se cumple en parte.
4. MENORES sin ronda: verificacion ligera determinista sin modelo cuando el anterior fue `verde` y los cambios solo tocan ficheros con MENOR abiertos; la ronda no sube.
5. Tope: `tope_rondas` de la base + `## Rondas extra` de la tarea; pasado con CRITICO o IMPORTANTE abierto, dictamen `tope` (`failure`) sin modelo y con abiertos arrastrados; F4-4 lo traducira a la parada `preguntar` de F1-10.
6. Ronda `incompleta`: no cuenta ni sirve de base del diff incremental.
7. Titulo v3 con `ronda=`, lectura compatible con v2.
8. `review` guarda el informe del head anterior antes de empujar.
9. Sin trocear.

## Suposiciones no verificadas

- Que el `GITHUB_TOKEN` de un reutilizable lanzado por `workflow_run`, con `checks: read`, lista los check-runs de otra App en un repo privado (E2E).
- Que GitHub acepta anotaciones de persistentes cuya linea ya no corresponde al mismo texto en el head nuevo (E2E).
- Que `gh api .../commits/<sha>/check-runs` admite `check_name` y devuelve `head_sha` y `output.title`.
- Que el perfil `llamador-revision` del lint fija hoy la lista exacta de permisos.
- Que `## Rondas extra` no tiene formato fijado en ningun ADR mas alla de nombrarla en F1-16 (decision 7 de la PoC L).

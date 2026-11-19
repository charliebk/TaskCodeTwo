# Informe de revisión — TASK-041 (ronda 2)

- Commit revisado: 5fe9c4fca6475b66f6bbe86d8025263d7efd485e
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Resumen

Verificación acotada de las correcciones de la ronda 1 (944a974 y 5fe9c4f). Se ha trabajado en el clon `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-041-r2-b` en `5a73f44`; el código es idéntico a `5fe9c4f` (la diferencia entre ambos solo toca `tareas/`). Los cuatro hallazgos de la ronda 1 quedan resueltos y comprobados. No queda ningún CRITICO ni IMPORTANTE abierto. Hay un MENOR nuevo, no bloqueante.

Alcance: diff de la corrección y los tests de `xtask` relacionados. No se ha corrido la suite completa ni `cargo xtask verificar`, como pedía el encargo.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido (verificado) | xtask/src/workflows.rs (test `leer_normaliza_crlf`), .gitattributes | implementacion |
| IMPORTANTE-2 | IMPORTANTE | corregido (verificado en GitLab real) | plantillas/gitlab/verificacion.gitlab-ci.yml:56-59 | implementacion |
| IMPORTANTE-3 | IMPORTANTE | corregido (verificado) | .github/workflows/verificacion.yml:136 | plan |
| MENOR-1 | MENOR | corregido (verificado) | docs/ci/verificacion-reutilizable.md:36 | implementacion |
| MENOR-2 | MENOR | aceptado (seguimiento: push a rama de PR y pull_request en grupos distintos; impacto bajo con runners de GitHub) | .github/workflows/verificacion.yml:136; xtask/src/workflows.rs:980-992 | implementacion |

### MENOR-2 — El grupo de concurrencia no une un push con el pull_request de la misma rama
- Donde: .github/workflows/verificacion.yml:136 (clave del grupo); xtask/src/workflows.rs:980-992 (test `la_concurrencia_es_por_pr_o_ref`)
- Que pasa: la clave es `verificacion-<repo>-<github.event.pull_request.number || github.ref>-<perfil>`. Un push a la rama de un PR abierto cae en `refs/heads/<rama>` y el `pull_request` en su número, así que son grupos distintos y pueden correr a la vez sobre el mismo código. El test solo busca la subcadena `github.event.pull_request.number || github.ref`; no comprueba el perfil ni la forma completa del grupo.
- Reproduccion: lectura de la clave. Mutación ejecutada: quitar `${{ github.event.pull_request.number || github.ref }}` del grupo, `cargo test -p xtask` da `la_concurrencia_es_por_pr_o_ref ... FAILED` (22 passed, 1 failed), así que esa parte sí está protegida. No se ha probado la mutación del perfil. El solapamiento push/PR en GitHub real no se ha reproducido (requiere GitHub).
- Impacto: bajo. Con runners de GitHub no hay estado compartido. Con un runner propio de un solo ejecutor, el propio runner serializa. Solo importa con varios ejecutores que compartan `target/`, caso que el doc deja para F5-5.
- Sugerencia: añadir una frase al apartado 6 de `docs/ci/verificacion-reutilizable.md` que diga que push y pull_request de la misma rama no se serializan entre sí, y reforzar el test con la forma completa del grupo (incluido el perfil). No bloquea el cierre. Queda como backlog de MENORES.

## Verificación de los hallazgos de la ronda 1

### IMPORTANTE-1 (CRLF en el lint) — corregido
- Reproducción: en el clon `5a73f44`, `rtk proxy cargo test -p xtask` da `test result: ok. 23 passed; 0 failed`. La ronda 1 daba 16 de 17 con un fallo.
- Con `plantillas/gitlab/verificacion.gitlab-ci.yml` convertida a CRLF (con `sed`, y restaurada después con `git checkout`), la suite sigue en `23 passed; 0 failed`. El lector normaliza CRLF (test `leer_normaliza_crlf`) y `.gitattributes` fija `*.yml` y `*.yaml` a `eol=lf`.

### IMPORTANTE-2 (la plantilla de GitLab no se había ejecutado; duda sobre `rules:if`) — corregido y verificado en GitLab real
Lectura sola de la API de GitLab (`glab api`):
- Pipeline 3090338668 (`sha f35e736d`, ref `ci-f5-1`): `status: success`. Job `verificacion-rapida` (18191983313): `success`, 15 s. Traza: `$ sh scripts/verificar.sh --perfil rapida` y `stub verificar args: --perfil rapida`. Esto confirma que `rules:if` lee la variable global `TC2_PERFILES` y que el job se crea con `perfiles: rapida`.
- Pipeline 3090341084 (`sha bcff47cb`): `status: failed`. Job `verificacion-rapida` (18192006238): `failed`. Traza: `fallo forzado` y `ERROR: Job failed: exit code 1`.
- El `.gitlab-ci.yml` de `taskcodetwo-e2e-gitlab` en `f35e736d` incluye la plantilla de `charliebk/TaskCodeTwo` en el SHA `944a9746b62cf055aedaf210cacdf270f62fc63f`, la versión con la corrección.
- La variable global está en `plantillas/gitlab/verificacion.gitlab-ci.yml:56-59`. El test `las_reglas_de_gitlab_no_leen_variables_de_job` existe y pasa en la suite.

### IMPORTANTE-3 (concurrencia por repo y perfil) — corregido
- El grupo ahora incluye `github.event.pull_request.number || github.ref` (`verificacion.yml:136`). Los PR distintos tienen grupos distintos. Las ejecuciones del mismo PR siguen encoladas, con `cancel-in-progress: false`.
- Mutación: quitar la parte del PR o de la ref hace fallar `la_concurrencia_es_por_pr_o_ref` (ver MENOR-2 para lo que no se ha probado).
- No reproducido en GitHub real. La afirmación de que `github.event` en un workflow reutilizable es el del llamador es el comportamiento documentado de GitHub, no comprobado aquí.

### MENOR-1 (aviso sobre interpolación en `comando` y `argumentos-*`) — corregido
- `docs/ci/verificacion-reutilizable.md:36` añade el aviso: las entradas se interpolan en la línea del shell y nunca deben recibir datos del evento. Además señala que el lint no lo detecta. Se cumple lo pedido.

## Comprobado sin hallazgo

- `cargo test -p xtask` en `5a73f44`: 23 passed, 0 failed (la lista incluye `leer_normaliza_crlf`, `la_concurrencia_es_por_pr_o_ref`, `las_reglas_de_gitlab_no_leen_variables_de_job` y `la_plantilla_de_gitlab_rechaza_datos_del_autor_y_secretos`).
- Mutación del grupo de concurrencia: `la_concurrencia_es_por_pr_o_ref` FAILED (ver MENOR-2).
- Plantilla en CRLF: 23 passed, 0 failed.
- Pipelines 3090338668 (success) y 3090341084 (failed) con sus jobs y trazas (ver IMPORTANTE-2).
- El doc (apartado 6 y tabla E2E) coincide con el YAML del workflow y con los pipelines reales.
- No se han hecho commits ni push. El clon de revisión queda sin cambios respecto a `5a73f44`.

## Lecturas fuera del contexto permitido

- `xtask/src/workflows.rs`: lectura de los tests `la_concurrencia_es_por_pr_o_ref` y `leer_normaliza_crlf`, y ejecución de la suite, para comprobar IMPORTANTE-1 y IMPORTANTE-3.
- `C:\Users\nullcad2025\.claude\plugins\cache\taskcode-marketplace\taskcode-plugin\0.16.2\skills\task-workflow\revision.md`: plantilla del informe y linea del veredicto.
- Pipelines, jobs y trazas de GitLab (`glab api`, solo lectura): lo pedía el encargo.
- `.gitlab-ci.yml` del repo `taskcodetwo-e2e/taskcodetwo-e2e-gitlab` en `f35e736d` (`glab api`, solo lectura): para comprobar a qué SHA de la plantilla apunta el E2E.

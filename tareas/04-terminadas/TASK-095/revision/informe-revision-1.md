# Informe de revision — TASK-095 (ronda 1)

- Commit revisado: 6e70621ac2c507bb073b6cb6643785dce88ee9a7
- Revisor: code-quality-reviewer (agente general-purpose independiente, modelo sonnet; revision de un ADR, sin codigo de la 2.0 en el diff)
- Veredicto: cambios-solicitados
- Nota del implementador: el revisor propuso `aprobada-con-correcciones`, pero con dos IMPORTANTES abiertos la regla de la fase de revision exige `cambios-solicitados`; se registra asi con `taskctl veredicto`.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-19-memoria-del-proyecto.md (PoC 1 «Dato real», decision 6, tabla de evidencia RF-78/RF-79) | plan |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-19-memoria-del-proyecto.md (PoC 2 «6 ms», decision 2, tabla de evidencia F10-6) | plan |
| MEN-1 | MENOR | corregido | docs/adr/README.md (fila F1-19, linea tras F1-18) | implementacion |
| MEN-2 | MENOR | corregido | docs/adr/F1-19-memoria-del-proyecto.md (decision 4, Consecuencias «Texto de RR-20») y tarea.md criterio 2 | especificacion |
| MEN-3 | MENOR | corregido | docs/adr/F1-19-memoria-del-proyecto.md (Contexto «94 tareas», PoC 1 hash del .db, mediciones) | implementacion |
| MEN-4 | MENOR | corregido | docs/adr/F1-19-memoria-del-proyecto.md (Consecuencias F1-4 y F3-5b) | plan |
| MEN-5 | MENOR | corregido | docs/adr/F1-19-memoria-del-proyecto.md (decisiones 4, 5, 6 frente a lo que cubre la PoC) | implementacion |

Origen: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el plan era ambiguo o equivocado y el ADR lo siguio; `especificacion` si el fallo esta en el enunciado o los criterios de la tarea.

Ninguna de las dos PoC falseo el nucleo del ADR: lo reproducido coincide con lo afirmado en huella, control de arbol sucio, control de cambio, merge en plataforma, force-push y remoto inaccesible. Los hallazgos son de cifras y afirmaciones no respaldadas por lo medido.

## Reproduccion

Entorno: copia de `poc-indice` en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-095-r1\`, `cargo build --release` con `CARGO_TARGET_DIR` dentro; git 2.56.0.windows.4, rustc 1.100.0. Carpeta borrada al terminar; `task-095\poc-indice` sin tocar.

### Lo que SI se reproduce (sin hallazgo)

- PoC 1: `develop` = e1bf456, 482 commits. Dos reconstrucciones: `tareas=30 commits=362 archivos=1307 huerfanos=120 huella=9527d653…a994fe4`, identica al ADR. Los `.db` coinciden byte a byte entre si (md5 `c0802185…`). Control de arbol sucio con `core.autocrlf=true`: huella igual. Control de commit que toca una tarea: cambia la huella, `commits=363`.
- PoC 2: bare local, clon A con `core.hooksPath` vacio, clon B. B publica TASK-999 con un merge de dos padres (hecho con `commit-tree` y `update-ref` porque un hook del proyecto bloqueo `git merge`; ver notas). Antes de buscar, A en e1bf456 y el bare en 94ec091. Dentro del TTL (`--ttl-s 60`): `fresco=hace-25s remoto=no-consultado(ttl 60s) indexado=e1bf456`, sin resultados, 8 ms. Con `--ttl-s 0`: `fresco=si remoto=consultado(687 ms) … indexado=94ec091` y aparece TASK-999. TTL real con `--ttl-s 4` y `sleep 5`: vuelve a consultar. Force-push: `reindexado=completa`, sin error. Remoto inaccesible: `fresco=desconocido remoto=no-consultado(364 ms)`, sin pedir credencial.
- Cifras comprobadas: 482 commits; 365 asuntos con `(TASK-`, 362 atribuidos a las 30 terminadas; cita de `lectura.rs`; decision 10 de F1-11; engram «conveniente» en F1-14; `Cerrojo` con `taskcode-estado.lock`; `Subcomando` sin `CatFile`.

### IMP-1 — La atribucion por `(TASK-NNN)` pierde el trabajo real de las tareas y el ADR lo describe mal

El ADR atribuye los 120 huerfanos a commits como `docs: ROADMAP regenerado…` o `chore: alta…` y a tareas no terminadas. Con `git log develop`: 117 asuntos sin `(TASK-`; los mas frecuentes son 18 `merge(feature): feature/task-N-…`, 16 `chore(taskctl): import de N tarea(s)` y unos 34 `docs(adr): F1-N corrige hallazgos…`; 87 no-merge y 30 merges sin `(TASK-`. Las no terminadas son 3 commits (365-362).

```
sqlite3 a.db "select count(distinct tarea) from tarea_archivo where ruta like 'docs/adr/F1-%'"  -> 2
sqlite3 a.db "select count(*) from tarea where titulo like 'F1-%'"                            -> 19
sqlite3 a.db "select tarea,ruta from tarea_archivo where ruta like 'docs/adr/F1-17%'"        -> (vacio)
```

De 19 tareas ADR, solo 2 tienen su ADR entre los ficheros tocados. La regla de atribucion solo vive en la PoC; las decisiones 3 y 6 no la fijan. Correccion: cifras reales, regla de atribucion fijada (p. ej. rama `feature/task-NNN` de los merges `--no-ff` y su rango `merge^1..merge^2`) y RF-79 marcado con 2 de 19.

### IMP-2 — «De 6 a 7 ms dentro del TTL» no respalda los 200 ms de F10-6

La cifra es el tiempo interno de `responder` con `--db` explicito, sin lanzar `git` ni comparar refs locales. Proceso completo desde bash:

```
git rev-parse --git-common-dir : 253 200 154 145 170 168 ms
git --version                  : 150 258 203 157 152 175 ms
buscar dentro de TTL (--ttl-s 600), proceso completo, 6 tomas: 177 203 243 223 235 289 ms
```

La decision 2 (calcula la ruta con `git rev-parse`) y la decision 4 (sin lanzar `git`) se contradicen. Correccion: decir que incluye la cifra, declarar no verificado resolver directorio comun y refs sin `git`, y fijar si los 200 ms de F10-6 incluyen el arranque. Ademas, `ls-remote` midio aqui 609 ms contra el bare y 1440 ms contra GitHub: citar las cifras como «en esa maquina, en ese momento».

### MEN-1 — Fila F1-19 desligada de la tabla del README

Linea en blanco entre F1-18 y F1-19: la fila se renderiza como parrafo.

### MEN-2 — Criterios y plan con texto que contradice la politica elegida

Con TTL de 60 s el criterio 2 («se refleja en la siguiente busqueda») no se cumple literalmente (la PoC lo muestra). Siguen con el texto antiguo RR-20, la fila F1-19, la fila F10-2 y el criterio 2 de `tarea.md`. Listar las cuatro o declarar que el criterio 2 se cumple en la primera busqueda tras caducar el TTL.

### MEN-3 — Cifras sueltas no reproducibles o desactualizadas

- «94 tareas con `etiquetas: []`»: en `develop` hay 112 `tarea.md`, todas con `etiquetas: []`.
- Hash concreto de los `.db`: la coincidencia se reproduce, el valor no (`c0802185…` frente a `8a352130…`).
- «4 a 12 s» de reconstruccion: aqui 15-19 s; citar la dispersion.

### MEN-4 — Efectos sobre otros ADR que Consecuencias no recoge

- F1-4: la lista permitida del detector no tiene `cat-file` ni `ls-tree`; hay que ampliarla junto al puerto.
- F3-5b dice «traen la rama de estado antes de mostrar»; con TTL, `status`/`roadmap` no traen nada dentro del TTL. Declararlo como cambio a F3-5b y confirmarlo con Carlos.

### MEN-5 — Diseno no ejercitado por las PoC, no marcado como no verificado

- Decision 4: la comparacion en cada lectura con las refs de seguimiento locales no existe en la PoC.
- Decision 5: «`rename` falla en Windows» y «WAL» enunciados como hechos; la PoC usa journal por defecto.
- Decision 5: la PoC escribe `sincronizacion` antes de reindexar y en otra transaccion (si falla el reindexado, el TTL queda marcado con el indice viejo); el diseno del ADR lo evita, la PoC no lo demuestra.
- Decision 6: la huella solo se probo en una maquina y un sistema; la PoC cubre 5 tablas, no `etiqueta`, aristas ni `cambio`.

## Lecturas fuera del contexto permitido

Todas de solo lectura, para contrastar el ADR con lo que cita (lo pedia el encargo):
- `docs/plan/PLAN-TASKCODE-2.md` (filas RR-20, F1-19, F3-5b, F10-2, F10-6).
- `docs/adr/F1-2`, `F1-4`, `F1-9`, `F1-10`, `F1-11` y `F1-14`, con busquedas por texto.
- `crates/tc2/src/estado/cerrojo.rs`, `crates/tc2/src/git/puerto.rs` y nombres de `crates/tc2/src/git/`.
- `git log` y `git ls-tree` de `develop`.
- El codigo de `poc-indice` (`src/main.rs`).

## Notas del proceso

- Un hook del proyecto bloqueo `git merge feature/task-999` en el montaje temporal de la PoC 2; el revisor hizo el merge de dos padres con `commit-tree` y `update-ref` en el clon temporal y lo declara.
- Carpeta `task-095-r1` borrada y `git status` del repo limpio.

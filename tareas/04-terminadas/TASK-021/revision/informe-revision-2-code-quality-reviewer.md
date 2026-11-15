# Informe de revision — TASK-021 (ronda 2)

- Commit revisado: 80b51da9ce8382316c6717f8bee44e45c9b1feae
- Revisor: code-quality-reviewer (verificacion acotada de la ronda 2)
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/tests/plataforma.rs:377 | implementacion |
| IMP-2 | IMPORTANTE | corregido | crates/tc2/src/plataforma/github.rs:660 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Sin hallazgos nuevos: el diff de la ronda (aa704e8..80b51da) no abre ningun CRITICO ni introduce
un fallo de regresion comprobable.

## Reproduccion

Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-021-r2-calidad`, rama
`feature/task-021-f2-6-adaptador-github`. La rama esta en 94c2a8b, que solo anade la peticion y el
informe de la ronda 2 sobre 80b51da (el diff 80b51da..94c2a8b toca solo `tareas/`). El codigo
revisado es el de 80b51da.

**Puerta determinista.** `cargo test -p tc2` en el clon: lib 39 ok; `tests/plataforma.rs` 4 ok;
`tests/plataforma_e2e.rs` 2 ignorados (E2E, no ejecutados: su evidencia es `evidencia/e2e.log`);
el resto de binarios en verde (alta 3, cli 2, estado_bordes 9, estado_casos 4, estado_concurrencia 3,
git_* 9, tarea 5 ok y 1 ignorado pesado). 0 fallos.

### IMP-1 — test `el_envoltorio_rechaza_antes_de_la_red` (crates/tc2/tests/plataforma.rs:377)

Estado: **corregido**.

- Repetido 10 veces con `cargo test -p tc2 --test plataforma el_envoltorio`: 10 pasadas, 0 fallos.
- La suite `--test plataforma` entera repetida 10 veces: 10 pasadas (4 ok en cada una), 0 fallos.
- Causa de la correccion: el mock (`tests/comun/http_mock.rs`) responde con `Content-Length`, y el
  adaptador (`http.rs`, nuevo bloque `declarado`) rechaza un cuerpo declarado por encima de
  `LIMITE_CUERPO` antes de leerlo. Ya no hay corte a medias ni `sinred`.
- Nota no bloqueante (fuera de la tabla, no abre ronda): un cuerpo mayor de 1 MiB sin
  `Content-Length` (chunked) sigue dependiendo del limite de lectura y no hay test que lo ejercite.
  El caso de la prueba es el declarado, que es el que cubre el test.

### IMP-2 — protecciones sin test offline

Estado: **corregido**. Se anadio `protecciones_y_bordes` (tests/plataforma.rs:623 en adelante), que
cubre G09, reglas 3, 4 y 6 (`web-flow`, `Bot`, comentario posterior), `al_dia`/`sin_conflictos`,
PR cerrado, y el aviso de plan. Mutantes aplicados en el clon, uno cada vez, y `cargo test -p tc2
--no-fail-fast --lib --test plataforma`:

| Mutante | Resultado |
|---|---|
| M1 G16: `leer_permisos_ci` con `if !p.can_approve_pull_request_reviews` (github.rs:680) | muerto (`protecciones_y_bordes`, plataforma.rs:752) |
| M2 G09: `estado_puertas` sin `a.id == *origen` (github.rs:514) | muerto (plataforma.rs:641) |
| M3 `es_persona` sin la exclusion `web-flow` (mod.rs:65) | muerto (plataforma.rs:675) |
| M4 `es_persona` sin comprobar `tipo == "User"` (mod.rs:65) | muerto (plataforma.rs:675) |
| M5 `firma_commit` con `verified` negado (github.rs:707) | muerto (plataforma.rs:782) |
| M6 `Firma::cuenta_como_aprobacion` sin exigir `verificada` (mod.rs:96) | muerto (plataforma.rs:782) |
| M7 `leer_puertas` con `plan_da_reglas: true` en el aviso de plan (github.rs:649) | muerto (plataforma.rs:798) |

Los siete mutantes los mata el mismo test (`protecciones_y_bordes`). Tras cada mutante se restauro
el fichero con `git checkout`; el clon quedo limpio (`git status` vacio). Observacion: en el mutante
M7 la bateria tardo 32,75 s en vez de ~0,06 s; no es un fallo del codigo real (es la rama mutada), pero
conviene vigilarlo si reaparece en una pasada normal.

### Revision de las correcciones (busqueda de fallos nuevos)

- `deteccion.rs`: una clave con host conocido ahora exige la base oficial (`api_oficial`). El caso
  MEN-4 (`github.com` con `api.evil.example`) queda rechazado. Los tests de `falla_cerrado` lo cubren.
- `errores.rs`: `es_limite` suma `429` y el texto «rate limit». Cubre MEN-6 sin cambiar el 403 con
  cabeceras. Sin regresion observada.
- `github.rs`: checks `neutral`/`skipped` como OK (MEN-3); reviews solo cuentan `APPROVED`,
  `CHANGES_REQUESTED`, `DISMISSED` (MEN-2); `sha` opcional en `merge-pr` con `merged` (sin
  inventar commit); `sin_conflictos` con `dirty`. Revisado: la expectativa del diagnostico
  «lanza actualizar-rama» para conflictos es coherente, porque `actualizar-rama` es el paso local que
  resuelve conflictos (`git/guardar.rs`, `git/remoto.rs`).
- `http.rs`: host sin `:` con puerto validado (`puerto_valido`, 1..65535); rechazo de `?` y `#`
  en el prefijo. Las URLs validas habituales (`https://api.github.com`, `https://gitlab.com/api/v4`,
  `http://127.0.0.1:PUERTO`) siguen aceptadas por los tests.
- `puertas.rs`: nuevo campo `sin_conflictos` y su mensaje; los tests de `evaluar` lo actualizan.

Profundidad: revisado a fondo el diff de la ronda y los tres puntos de IMP-1 y IMP-2. Los demas
cambios (MEN-1 a MEN-6) solo por lectura, sin verificacion contra GitHub real, como ya indicaba la
ronda 1. No se ejecuto el E2E (prohibido escribir en GitHub).

## Lecturas fuera del contexto permitido

- `crates/tc2/src/plataforma/github.rs`, `http.rs`, `mod.rs` (fragmentos de las lineas de los
  mutantes y del bloque de lectura de cuerpo): para mutar los puntos de IMP-2 y verificar la correccion
  de IMP-1.
- `crates/tc2/tests/comun/http_mock.rs`: para entender por que el mock envia `Content-Length`.
- `crates/tc2/src/contrato.rs` y `crates/tc2/src/git/remoto.rs`, `git/guardar.rs` (solo `git grep`,
  para confirmar el papel de `actualizar-rama` ante conflictos).
- `tareas/03-en-revision/TASK-021/revision/informe-revision-1-code-quality-reviewer.md`: para
  contrastar los hallazgos de la ronda 1.

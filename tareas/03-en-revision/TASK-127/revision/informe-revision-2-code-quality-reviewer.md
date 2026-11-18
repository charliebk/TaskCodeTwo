# Informe de revision — TASK-127 (ronda 2)

- Commit revisado: 0d318d505400cf903b38867e6846e8355967785c
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/git/estado.rs:348 | implementacion |
| MEN-1 | MENOR | corregido | docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:578 | implementacion |
| MEN-2 | MENOR | corregido | crates/tc2/src/git/estado.rs:269 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

## Puerta y alcance (verificacion acotada)

- Suite: no repetida. La peticion trae la verificacion de la ronda en verde sobre 4589b99: log `verif-127-r2.log` (en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo`), con `xtask: verificacion Rapida en verde` y `exit=0` en el fichero `.fin`. `git diff --stat 4589b99 0d318d5 -- crates docs` esta vacio: el codigo revisado es identico.
- Tests ejecutados por mi en un clon de 0d318d5 (carpeta borrada al terminar):
  - `cargo test -p tc2 --lib tests_stdin`: 2 pasan (`shas_completos_exige_uno_por_fichero`, `con_entrada_da_error_si_git_falla`).
  - `cargo test -p tc2 --test estado_casos`: 7 pasan, incluido `mil_ficheros_en_una_transaccion_con_procesos_acotados` (119 s).
- Mutaciones (cada una en su clon, restaurada despues; `git status` limpio):
  - A: en `shas_completos`, `(shas.len() == n).then_some(shas)` pasa a `Some(shas)`. `tests_stdin`: ROJO (`shas_completos_exige_uno_por_fichero`, panic en estado.rs:508).
  - B: en `con_entrada`, `if s.codigo == 0 {` pasa a `if true {`. `tests_stdin`: ROJO (`con_entrada_da_error_si_git_falla`, panic en estado.rs:528).
  - C (adicional): en `construir_commit`, `shas_completos(&shas, cambios.len())` pasa a `shas_completos(&shas, shas.lines().count())`, es decir, se anula la comprobacion que impide commitear con salida incompleta de `hash-object`. `tests_stdin`: VERDE (2/2). `estado_casos`: VERDE (7/7).

## Verificacion de cada hallazgo de la ronda 1

### IMP-1 — abierto (cubierto en parte)
- Donde: `crates/tc2/src/git/estado.rs:348` (`construir_commit`, la comprobacion que sustituyo a `HASH_OBJECT_INCOMPLETO`).
- Lo que la correccion cubre: la rama de error de `con_entrada` (mutante B) y la decision de `shas_completos` (mutante A) ya tienen un test que se pone rojo.
- Lo que no cubre: el cableado en `construir_commit`. Con el mutante C la proteccion desaparece y ningun test se entera, porque `shas_completos_exige_uno_por_fichero` prueba la funcion y no que `construir_commit` la use. El impacto declarado en la ronda 1 sigue vigente: un arbol con blobs sin escribir se commitearia en silencio.
- Reproduccion: el mutante C arriba, con su salida en verde.
- Sugerencia: un test que ejercite `construir_commit` con un `hash-object` que devuelva menos lineas que rutas (por ejemplo, un git falso o un fichero que desaparece entre la escritura y el `hash-object`), que compruebe `Resultado::Error` con `HASH_OBJECT_INCOMPLETO` y que la rama de estado no avanza. Hasta entonces el hallazgo queda abierto.

### MEN-1 — corregido
- Verificado: la adenda de `docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:578` dice ahora que `cat-file` tiene sus argumentos comprobados como los de cualquier llamada. Es cierto: `lanzar_con_entrada` (`puerto.rs:347`) llama a `comprobar_argumentos` antes de lanzar git para todos los subcomandos, `cat-file` incluido. La frase de las tres formas exactas se corrige para `cat-file`.

### MEN-2 — corregido
- Verificado: `con_entrada` (`estado.rs:269`) ya no reimplementa `GIT_FALLO`: construye un `SalidaGit` sin stdout y llama a `git_fallo` (`git/mod.rs:67`), que usa `primera_linea_error`. Queda un unico camino de diagnostico. No se encuentra ningun diagnostico divergente.

## Hallazgos nuevos

Sin hallazgos nuevos. Se busco el fallo que abre la correccion: `shas_completos` (`estado.rs:303`) cuenta lineas tras `trim`; con salida vacia y n = 0 devuelve `Some(vacio)`, correcto. La copia de `SalidaGit` en `con_entrada` solo afecta al camino de error. No se abre ningun CRITICO.

Nota sin abrir hallazgo: la copia de `SalidaGit` para descartar stdout en `con_entrada` es un rodeo que se podria evitar pasando solo el stderr; es estilo, no un fallo.

## Veredicto propuesto

`cambios-solicitados`: IMP-1 sigue abierto (el cableado de la comprobacion de `HASH_OBJECT_INCOMPLETO` no tiene red). MEN-1 y MEN-2 quedan corregidos. La linea de veredicto se deja como PENDIENTE para que la escriba `taskctl veredicto`.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/git/puerto.rs` (clon, lineas 290-340 y 455-490): para verificar que `cat-file` pasa por `comprobar_argumentos` (MEN-1) y las formas exactas de stdin.
- `crates/tc2/src/git/mod.rs` (clon, lineas 60-85): para `git_fallo` y `exigir` (MEN-2).
- `informe-revision-1-code-quality-reviewer.md`: para los hallazgos de la ronda 1 que hay que verificar.
- `revision.md` de la skill `task-workflow`: reglas comunes de la revision (lectura obligatoria del propio skill).
- `verif-127-r2.log` y `.fin` en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo`: para citar la pasada de la suite.

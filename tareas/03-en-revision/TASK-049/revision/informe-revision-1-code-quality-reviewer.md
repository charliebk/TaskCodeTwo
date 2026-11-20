# Informe de revision — TASK-049 (ronda 1)

- Commit revisado: 609cca1 (codigo igual que 17ad62c; solo cambia la peticion)
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Alcance y verificacion

Revision del workflow `revision-ia.yml` contra F1-8 y de la seleccion (criterio Rust + Vue). Ejecutado en el clon `task-049-r1-b`: `cargo test -p xtask` (28 ok), `cargo test -p tc2 --test revisoras` (3 ok), `cargo test -p tc2 --lib seleccion` (12 ok, incluye C1 Rust + Vue = dos revisoras de dominio sin generica, con S08, S12 y M04). No se lanzo ningun workflow ni la suite completa. El workflow no se ha podido ejecutar (E2E pendiente, declarado en la adenda): lo que sigue sobre GitHub Actions es lectura, salvo el hallazgo 1, reproducido con git.

## Hallazgos

| # | Severidad | Estado | Fichero:linea | Hallazgo |
|---|---|---|---|---|
| 1 | IMPORTANTE | corregido | `.github/workflows/revision-ia.yml:207` (tambien 241-250 y 214-217) | `git diff --name-only` sin `-z` y con `core.quotepath` por defecto: git escapa y entrecomilla los nombres con caracteres no ASCII, tabulador, comilla o barra invertida. Reproducido: un fichero `é.rs` sale como `"\303\251.rs"` (con comillas literales). `rutas.txt` lo lleva asi a `tc2 revisoras` (no hay des-escapado en `crates/tc2/src`), donde `*.rs` no casa por la comilla final, y a los `git --literal-pathspecs ... -- "${rutas[@]}"` (linea 244), donde la ruta no existe y el diff sale vacio. Consecuencia: un PR con ficheros de nombre no ASCII (legitimo, o puesto a proposito) se enruta a la generica y su diff no llega a ninguna revisora ni se avisa. Fix: `git -c core.quotepath=false diff --name-only -z` y leer con separador NUL (o `mapfile -d ''`), y que `tc2 revisoras --rutas` acepte NUL. Falta un test con una ruta no ASCII. |
| 2 | IMPORTANTE | corregido | `.github/workflows/revision-ia.yml:131-136` y `165-166` | Los checkouts usan `persist-credentials: false`, pero el paso "Head del PR como dato" hace `git -C base fetch origin <sha>` y `fetch ... taskcodetwo/tareas` despues, sin credencial. En un repo privado (el caso normal de un proyecto) esos fetch fallan (el primero aborta por `set -e`: `preparar` en rojo y check `neutral` con `preparar=failure`); la rama de estado ni siquiera avisa bien (cae al `|| echo`). Solo funciona en repos publicos. Fix: pasar `GH_TOKEN` y `git -c http.extraheader="AUTHORIZATION: basic $(printf 'x-access-token:%s' "$GH_TOKEN" \| base64 -w0)"` solo en esos fetch (el token es de solo lectura y no hay secretos en el job), o dejar `persist-credentials` solo en `base` con el riesgo documentado (no hay codigo del PR ejecutado). No verificado en CI. |
| 3 | MENOR | corregido | `.github/workflows/revision-ia.yml:357-359` | `instalacion=$(api ... \| jq -r .id)` y `token=$(api ... \| jq -r .token)`: sin `pipefail`, un fallo de `curl -f` no aborta en el sitio; `instalacion` queda vacio y el siguiente `curl` falla con una URL rara (el paso acaba en rojo, no publica, pero el mensaje no dice que fallo la App). No hay fuga ni publicacion falsa. Fix: `set -o pipefail` o comprobar que `$token` no esta vacio ni es `null` antes de `add-mask`. |
| 4 | MENOR | corregido | `.github/workflows/revision-ia.yml:302` | `art=$(... '.artifacts[0].id // ""')` seguido de `gh api .../artifacts/$art/zip`: si por cualquier motivo no hay artefacto, `$art` vacio produce un error confuso. Tampoco hay `if-no-files-found` que lo evite en `publicar`. Poco probable (`preparar` = success implica que se subio). |
| 5 | MENOR | corregido | `.github/workflows/revision-ia.yml:241-250` | El diff del autor va entre fences de tres comillas: un diff con una linea que contenga tres comillas cierra el bloque y lo siguiente se lee como instruccion. La adenda y la guia ya declaran que la inyeccion de instrucciones es de F6-7 y la salida solo va a un check `neutral`, asi que no es un fallo del alcance; se anota para F6-7 (usar un fence mas largo que cualquier secuencia del diff, o un delimitador con nonce). |

## Lo comprobado y sin hallazgo

- **Clase y privilegio (F1-8)**: `preparar` solo `contents` y `actions` de lectura, sin secretos; `publicar` solo `actions: read` con `environment: revisor`; permisos de raiz `{}`; sin escritura con GITHUB_TOKEN; el lint (perfil `revision-ia`) lo impone y sus tests pasan.
- **Nada del head ejecutado**: ningun checkout del head; el head solo entra por `git fetch` de su SHA (validado `[0-9a-f]` en 225-227) y `git diff`. El `cargo build` es del codigo de TaskCodeTwo a un SHA, no del PR. La seleccion y las skills salen de la base. Las lecturas de `tc2 revisoras` son datos.
- **Secretos solo en `publicar`**: `ANTHROPIC_API_KEY`, `TC2_APP_REVISOR_*` solo en `env:` de ese paso; no aparecen en `run:` ni en el artefacto; la clave PEM va a fichero con umask 077 y se borra tras firmar; la clave de la API solo va a `api.anthropic.com`.
- **Token de la App**: JWT RS256 con `iat` -60 s y `exp` +540 s (dentro del maximo de 10 min); token de instalacion con `permissions: {checks: write}` y `repositories` restringido al repo; `::add-mask::` antes de usarlo y nunca se imprime.
- **Inyeccion en `run:`**: ningun `${{ }}` dentro de `run:`; todos los datos del PR (`rama`, `base`, `head-sha`, `pr`, `perfil`, `tope-diff`) pasan por `env:`; la rama solo se usa con `sed` de expresion fija. Los datos del modelo y del diff no llegan a una linea de comandos: van por `jq --arg/--rawfile`. Rutas con `--literal-pathspecs`.
- **Sin secretos sale limpio**: sin `APP_ID` o `APP_CLAVE`, `exit 0` con aviso antes de tocar nada; sin `ANTHROPIC_API_KEY` publica el check con `sin-clave-del-modelo`.
- **Conclusion neutral** fija (`conclusion: "neutral"`), tope de diff por revisora con aviso en el prompt, tope del informe (zip 1 MiB, JSON 64 KiB + tope en tc2), resumen truncado a 65000.
- **Seleccion**: el criterio (Rust + Vue = angular-vue + rust-wasm, sin generica) lo cubre el unitario C1 y la integracion; mas de tres dominios y README a la generica tambien con test.
- **No verificable aqui (sin cambio de veredicto)**: la version de API `2026-10-21`, la cabecera beta `server-side-fallback-2026-07-01` y `fallbacks: "default"`, y que los secretos del entorno `revisor` lleguen a un job de un reutilizable sin `secrets:` declarado. Todo ello queda en el E2E pendiente que la adenda ya declara.

## Lecturas fuera del contexto permitido

- `xtask/src/workflows.rs` (perfil y lint): para confirmar lo que el lint impone.
- `crates/tc2/src/seleccion/mod.rs`, `crates/tc2/src/revision/mod.rs`, `crates/tc2/src/lib.rs` (busqueda): para ver si `tc2` des-escapa rutas entrecomilladas (no lo hace).

veredicto: cambios

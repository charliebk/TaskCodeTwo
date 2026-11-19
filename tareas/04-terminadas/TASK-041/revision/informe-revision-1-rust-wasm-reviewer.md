# Informe de revision — TASK-041 (ronda 1)

- Commit revisado: 171ec22 (codigo identico al 648720a2f6f21211bc3b31ff5e612126c6c7ce61; solo cambia la peticion)
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | xtask/src/workflows.rs:145-164 (`leer_yaml`), 184-219 (`mapa`), 221-255 (`lista`) | implementacion |
| IMP-2 | IMPORTANTE | corregido | xtask/src/workflows.rs:441-447 (raiz sin inspeccionar), 377-383 y 465-474 (patrones) | implementacion |
| IMP-3 | IMPORTANTE | corregido | xtask/src/workflows.rs:705-711 (`la_plantilla_de_gitlab_rechaza_datos_del_autor_y_secretos`), `.gitattributes` | implementacion |
| MEN-1 | MENOR | corregido | xtask/src/workflows.rs:477-481 y 448-506 (jobs con `uses:` a nivel de job) | implementacion |

### IMP-1 — el lector de YAML falla en abierto: lo que no entiende se descarta sin error

`mapa` y `lista` hacen `break` cuando una linea tiene otra sangria de la esperada, y `leer_yaml` nunca
comprueba que haya consumido todo el documento. Una linea de continuacion de un escalar plano
(valida en YAML) o cualquier sangria atipica corta el analisis en silencio: todo lo que sigue
desaparece del arbol y el lint lo da por bueno.

Reproduccion (prueba temporal en mi clon, revertida): sobre `BUENO`, sustituir
`      - run: echo "$TITULO"` por

```
      - run: echo uno
          dos
  malo:
    permissions: write-all
    steps:
      - run: echo ${{ github.event.issue.title }}
```

`lint_github` devuelve `[]`. Un job con `write-all` e inyeccion por `github.event` pasa el lint.
Segundo caso: un paso `- run: echo a` seguido de una linea `         b` (sangria impar) y a continuacion
otro job, igual. Arreglo propuesto: al terminar `leer_yaml`, si queda alguna linea no vacia sin consumir,
fallar (el modulo es solo de test, `panic!` vale) o devolver un error de lint "documento no entendido".
Fail-closed es lo que pide una lista blanca.

### IMP-2 — falsos negativos de lo prohibido (todos con `lint_github` devolviendo `[]`)

Probados sobre `BUENO` con una sola mutacion cada uno:

1. `env:` a nivel de workflow con `${{ secrets.TOKEN }}`: el lint solo mira `jobs`, y el `env` raiz llega a
   todos los jobs, tambien a los de clase ejecuta. Es la fuga de secretos que F1-8 quiere impedir.
2. Sintaxis de corchetes: `${{ github['event']['pull_request']['title'] }}` en `run:` (los patrones solo
   buscan `github.event.`, `github.event[`, `github.head_ref`, `tojson(github`), y
   `${{ secrets['TOKEN'] }}` en clase ejecuta (solo se busca `secrets.`).
3. `toJSON( github)` con espacio tras el parentesis.

El lint cumple lo que el plan decia, pero el plan lo vende como lista blanca y estas formas son triviales de
escribir por descuido. Arreglo: inspeccionar tambien `env`/`defaults` de la raiz con las mismas reglas;
normalizar el valor antes de buscar (quitar espacios y comillas: `github['event']` pasa a `github.event`,
`secrets['x']` a `secrets.x`), o buscar con regex `github\s*(\.|\[)\s*['"]?(event|head_ref)` y
`secrets\s*(\.|\[)`.

### IMP-3 — la prueba de GitLab se pone roja en una copia con CRLF

En mi clon (Windows, `core.autocrlf=true`) los `.yml` salen con CRLF (`.gitattributes` fija LF para
`*.rs`, `*.toml`, `*.sh`, pero no para `*.yml`). La prueba de GitLab busca
`"    - $[[ inputs.comando ]] $[[ inputs.argumentos-completa ]]\n"` y falla con
`assertion failed: texto.contains(original)`. Resultado real: `cargo test -p xtask workflows` da
6 pasan, 1 falla. El lector si tolera CRLF (el resto de pruebas pasa con los ficheros reales en CRLF);
solo falla la busqueda literal del test. Plataforma principal del proyecto: Windows. Arreglo: anadir
`*.yml text eol=lf` a `.gitattributes` y/o normalizar `\r\n` a `\n` en `leer()`.

### MEN-1 — `uses:` a nivel de job no se valida

Un job que llama a otro workflow reutilizable (`uses: x/y/.github/workflows/z.yml@v1`, sin SHA) pasa el lint:
solo se validan los `uses` de `steps`. Solo se vigila `secrets: inherit`. Probado: `[]`. Conviene exigir SHA
(y repositorio conocido) tambien ahi, o dejar escrito en el modulo que queda fuera de alcance.

## Alcance
- Ficheros revisados: xtask/src/main.rs (solo el `mod workflows` bajo `cfg(test)`), xtask/src/workflows.rs.
  Leido como contexto del E2E y del lint: .github/workflows/verificacion.yml y
  plantillas/gitlab/verificacion.gitlab-ci.yml (dentro del contexto de la tarea).
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios de firma exportada (no hay `wasm_bindgen` en el diff).

## Reproduccion
- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-041-r1-a`, rama de la tarea, commit 171ec22. `CARGO_TARGET_DIR` propio dentro del clon.
- Build: lo compila `cargo test`/`clippy` de abajo (nativo; sin wasm32, el xtask no es WASM).
- Formato y lints: `cargo clippy -p xtask --all-targets -- -D warnings`: sin avisos. `cargo fmt --check` no ejecutado (fuera de lo pedido).
- Suite: `cargo test -p xtask workflows`: 6 pasan, 1 falla (IMP-3, por CRLF; no es un fallo del codigo del lint). No se paso la suite completa, como se pidio; la verificacion completa en verde de la peticion es de un arbol con LF.
- Casos construidos: la sonda de IMP-1 e IMP-2 (prueba temporal anadida a `workflows.rs`, revertida con `git checkout`; `git status` limpio al final).
- Mutacion (sobre los ficheros reales, revertida): (a) `persist-credentials: false` a `true` en el workflow: la prueba da rojo con "job ejecutar: checkout sin persist-credentials: false"; (b) anadir `pull_request_target:` al `on`: rojo con "evento no admitido: pull_request_target"; (c) cambiar el script de verificacion-pesada de la plantilla por `echo $CI_COMMIT_TITLE`: rojo con "job verificacion-pesada: CI_COMMIT_TITLE en script". El lint reacciona a las mutaciones directas.
- Medidas: no aplican.

## Revisado sin hallazgos
- Errores sin unwrap/panic fuera de tests: `workflows.rs` esta bajo `#[cfg(test)]`; los `panic!` solo estan en pruebas, con `allow` justificado. Sin `unwrap`/`expect` en el lector. Los indexados `&texto[..p]`, `valor[..p]`, `linea.texto[1..]` caen en limites de caracter (separadores ASCII).
- `escribe`: falla en cerrado ante `write-all`, `read-all`, escalar desconocido, permiso sin bloque (probado con `write-all` en las mutaciones).
- Lista blanca de eventos y de acciones: `pull_request_target`, `workflow_run`, acciones de terceros, `actions/checkout@v4` sin SHA, `ref:` en clase privilegio. Todos rojos en `cada_mutacion_prohibida_da_su_error`.
- El lector tolera CRLF y comentarios dentro de bloques `|` (cubierto por `el_lector_entiende_el_subconjunto` y por los ficheros reales en CRLF).

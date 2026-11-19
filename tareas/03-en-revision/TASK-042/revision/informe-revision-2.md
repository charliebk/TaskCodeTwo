# Informe de revision — TASK-042 (ronda 2)

- Commit revisado: 4f4a62d3640e923a9eb5296a8889503e5967fd17 (la verificacion se hizo en el clon de revision, en 0f3beec, que solo anade la peticion sobre 4f4a62d)
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | .github/workflows/verificacion.yml:197 | implementacion |
| MEN-1 | MENOR | corregido | .github/workflows/verificacion.yml:179 | plan |

Sin hallazgos nuevos. La correccion no introduce ningun CRITICO ni IMPORTANTE.

### IMP-1 — corregido: la regla de rutas ve los renombrados

Reproduccion en un repo git temporal (borrado al terminar). Base con `.github/workflows/c.yml` y `.taskcode/config.yml`; commit siguiente con `git mv .taskcode/config.yml config.yml` y `git mv .github/workflows/c.yml otro.yml`.

- `git diff --name-status HEAD^1 HEAD` muestra `R100 .taskcode/config.yml -> config.yml` y `R100 .github/workflows/c.yml -> otro.yml`.
- Linea original (sin `--no-renames`, ronda 1): no imprime nada. La regla no detecta el movimiento. Fallo reproducido.
- Linea actual del workflow (con `--no-renames`): imprime `.github/workflows/c.yml` y `.taskcode/config.yml`. La regla detecta el movimiento. Corregido.

### MEN-1 — corregido: las ordenes entran en la clave

Validado con jq 1.8.2 y un JSON de ejemplo que replica la forma de `toJSON(inputs)`:

- Cambiar `huella-local`, `argumentos-completa`, `comando` o `shell` cambia el hash de `ordenes` y, por tanto, la `clave` del perfil `completa`. Corregido.
- Cambiar `argumentos-completa` no altera la `clave` del perfil `rapida` (el hash de `rapida` queda igual). Correcto: solo entran las ordenes del perfil.
- `runner` como lista (`["self-hosted","win"]`) se une con comas y la clave sigue siendo hex de 64 caracteres.
- Las claves de `buscar` y del publicador salen de la misma variable `claves` (publicador: linea 384 con `jq -r --arg p "$p" '.[$p]'`), asi que el cambio llega a los dos lados.
- Los nombres de las entradas `comando`, `argumentos-<perfil>`, `shell` y `huella-local` coinciden con los `inputs` declarados en el `workflow_call` del workflow.

### Revision de la propia correccion

- La linea nueva de `ordenes` es bash y jq validos. Sintaxis y comillas correctas, y `sha256sum` en la misma tuberia que el resto del calculo.
- La clave cambia para todos los perfiles. Los verdes publicados con la formula anterior dejan de contar: solo se pierden aciertos de cache, nunca se reutiliza algo que no debe.
- `--no-renames` solo cambia la deteccion de renombrados; el formato de salida (una ruta por linea) sigue igual y el `grep -E` posterior no cambia.

### Verificacion ejecutada

- `cargo test -p xtask` en el clon de revision: 26 passed, 0 failed (mismo resultado que la ronda 1).
- No se ejecutaron la suite completa ni `cargo xtask verificar`, porque la peticion lo excluye.

## Notas (no abren ronda)

1. La guia (`docs/ci/verificacion-reutilizable.md`, linea ~240) describe `órdenes = sha256 del JSON`, pero el hash real se calcula sobre la salida de `jq -c` mas el salto de linea final de `printf`/`sha256sum`. Quien reimplemente la clave a partir de la guia sin ese salto obtiene otra clave. Imprecision de documentacion, la misma que ya tenia `huella-arbol`.
2. La guia cita "(revisión de la ronda 1)" dentro de la documentacion del producto. Es texto del proceso de revision que quedara obsoleto; conviene quitarlo.
3. Si `jq` falla dentro de la tuberia de `ordenes` (el workflow no usa `pipefail`), el paso no aborta y `ordenes` sale como el sha de la cadena vacia. Con `toJSON(inputs)` el JSON siempre es valido, asi que hoy no es alcanzable. Es el mismo patron de la linea 178, ya aceptado en la ronda 1.
4. MEN-2 (la regla de rutas fuera de PR y `merge_group` solo mira `HEAD^1`) no se ha revisado en esta ronda, porque la peticion deja los MENORES de la ronda 1 a quien coordina.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-042/revision/informe-revision-1-code-quality-reviewer.md`: citado en la peticion como fuente de los hallazgos a verificar.
- `git show 4f4a62d` y `git show --stat 0f3beec`: para acotar el diff de la correccion y confirmar que el clon solo anade la peticion.

## Notas de la ejecucion

- La peticion que se pidio se llama `peticion-revision-2.md`, no `peticion-revision-2-code-quality-reviewer.md`. Se uso el fichero que existe. La plantilla `informe-revision-2.md` queda sin tocar; el informe se escribio en la ruta indicada por la orden.
- El clon `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-042-r2` se conserva, sin borrar, porque la orden de trabajo lo usaba; quien coordina decide si se borra.

# Informe de revision — TASK-042 (ronda 1)

- Commit revisado: fc8caf0 (contenido de codigo identico a aee1621fc055573ce506eeeb31dda3082f1027a2; fc8caf0 solo anade la peticion)
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | .github/workflows/verificacion.yml:197 | implementacion |
| MEN-1 | MENOR | corregido | .github/workflows/verificacion.yml:179 | plan |
| MEN-2 | MENOR | aceptado (seguimiento: fuera de PR y merge_group la regla mira solo HEAD^1; impacto bajo) | .github/workflows/verificacion.yml:196 | plan |

Alcance: pase generico sobre `buscar`, `ejecutar` y el publicador del workflow (la guia y la plantilla de GitLab solo en lo que describen). Profundidad: lectura completa del bash, reproduccion local de los fragmentos dudosos (git/awk/jq) y lectura de los logs reales de las ejecuciones 44607911390 (intento 2), 44608246992 y 44608801152 en `charliebk/taskcodetwo-e2e-github`. `cargo test -p xtask`: 26 passed, 0 failed. No se ejecuto la suite completa ni `cargo xtask verificar` (lo pedia la peticion). El bash del workflow no tiene pruebas unitarias (lo declara el plan): la red de regresion es el E2E manual, que no cubre el fallo de la API ni los renombrados.

### IMPORTANTE-1 — La regla de rutas no ve los renombrados que sacan un fichero de `.github/workflows/` o `.taskcode/`
- Donde: .github/workflows/verificacion.yml:197
- Que pasa: `git diff --name-only HEAD^1 HEAD` detecta renombrados (por defecto) y para un renombrado puro lista solo el nombre nuevo. Mover `.taskcode/config.yml` a `config.yml` (o un workflow a otra carpeta) no casa con `^(\.github/workflows/|\.taskcode/)`: la regla de F1-8 decision 7 («un PR que toca los workflows o .taskcode no reutiliza nada») no se aplica y `buscar` sigue ofreciendo candidatos.
- Reproduccion: repo temporal con `.github/workflows/c.yml` y `.taskcode/config.yml`; `git mv .taskcode/config.yml config.yml; git mv .github/workflows/c.yml otro.yml; commit`. `git diff --name-only HEAD^1 HEAD | grep -E '^(\.github/workflows/|\.taskcode/)'` no imprime nada (la regla no detecta). Con `--no-renames` imprime `.github/workflows/c.yml` y `.taskcode/config.yml`.
- Impacto: un PR puede cambiar la configuracion de CI/taskcode moviendola fuera de las rutas protegidas y conservar el acierto de la cache. Alcance real acotado (el arbol debe coincidir con el de un check verde anterior), pero la garantia declarada falla con una entrada trivial y el arreglo es de una linea.
- Sugerencia: `git diff --name-only --no-renames` (o `--name-status` con ambas rutas del renombrado) en la linea 197; anadir el caso a la medicion E2E o a una prueba de la regla.

### MENOR-1 — La clave no incluye los valores de las entradas del llamador
- Donde: .github/workflows/verificacion.yml:179
- Que pasa: `clave = sha256(perfil|runner|huella-arbol|version)`. `comando`, `argumentos-<perfil>`, `shell` y `huella-local` no entran. Hoy quedan cubiertos solo porque viven en el workflow llamador, que forma parte del arbol que se compara. Si el llamador pasa un valor que no esta en el arbol (por ejemplo `argumentos-completa: ${{ vars.X }}` o un input de `workflow_dispatch`), cambiarlo no cambia ni arbol ni clave y se reutiliza un verde obtenido con otras ordenes. No reproducido (no hay llamador asi en el E2E): razonado a partir del codigo.
- Impacto: acierto sobre un resultado de otra orden de verificacion, en un llamador que use variables.
- Sugerencia: meter en la clave un hash de `ENTRADAS` (ya disponible en el paso) o documentar que las entradas deben ser literales.

### MENOR-2 — Fuera de PR y merge_group la regla de rutas solo mira el ultimo commit
- Donde: .github/workflows/verificacion.yml:196
- Que pasa: la regla compara `V` con `HEAD^1`. En un PR o `merge_group` es la base y cubre todo el diff; en un `workflow_dispatch` sobre una rama, es solo el padre. Un commit vacio encima de otro que toco `.github/workflows/**` (mismo arbol) acepta la linea producida bajo el workflow modificado. No reproducido en GitHub (el E2E de ese caso fue en PR).
- Impacto: bajo (mismo arbol implica el mismo llamador), pero la guia dice «la base».
- Sugerencia: documentar que solo es la base en PR y merge_group, o limitar la busqueda a esos eventos.

## Comprobado sin hallazgos

- Acierto sobre un arbol distinto: exige `arbol` de la linea igual al de `V` (awk, linea 231), perfil, runner y clave actuales (jq), check-run en verde, `app.id == 15368` y nombre `verificacion`, y ejecucion enlazada por check suite que llama al workflow en la misma version; el publicador relee el arbol por la API y falla si difiere de `buscar`. Los logs de las ejecuciones citadas coinciden con la guia (regla de rutas, candidatos duplicados por intento sin efecto).
- Fallo cerrado: error de la API en check-runs o runs da «no cuenta» (verifica entero); fallo de `commits/{V}` o de `buscar` aborta el publicador sin check; un JSON de candidatos roto aborta el paso `cache` (job rojo). Un `jq` que falle dentro de una tuberia sin `pipefail` (linea 178) solo dejaria el runner vacio, y `runs-on` falla antes por el mismo dato.
- Inyeccion: los datos del evento (`head_ref`, `base_ref`, `inputs`) van por `env`, nunca interpolados en `run:`; las rutas de `huella-arbol` son del llamador. `buscar` hace checkout sin credenciales y solo lee con git. Los outputs del job ejecutor los fija el paso `cache` (cada paso tiene su propio fichero de salidas), el codigo del PR no puede escribirlos.
- Riesgo residual ya aceptado en el plan: `local` lo produce el job ejecutor.

## Lecturas fuera del contexto permitido

- `skills/task-workflow/revision.md` del plugin y `tareas/03-en-revision/TASK-042/medicion/salida.txt`: plantilla del informe, criterio del veredicto y cifras citadas por la guia.
- Logs de tres ejecuciones del repo E2E (solo lectura, `gh run view --log`): comprobar que lo que la guia afirma coincide con lo ejecutado.

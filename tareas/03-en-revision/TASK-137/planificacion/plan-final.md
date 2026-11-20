# Plan — TASK-137: F6-3 seguimiento: revision-ia cita el informe del run de origen cuando la verificacion sale de cache

(Esta tarea no lanza brainstorm: su complejidad resuelve cero roles.
El plan se redacta directamente a partir del enunciado.)

## Enfoque propuesto

Duracion estimada: 120 min
Senales: un paso de `.github/workflows/revision-ia.yml` y su guia; sin codigo Rust ni cambio del lint; E2E real en el repo de GitHub con dos runs de verificacion y dos de revision-ia; 4 criterios.

Hoy, con acierto de la cache por arbol, el job `ejecutar` de `verificacion.yml` se salta el comando y su `upload-artifact` (con `if-no-files-found: ignore`) no sube nada; el paso `Informe de verificacion` de `revision-ia.yml` solo mira el run `verificacion-run` y sale con `sin artefacto`.

Alternativas valoradas:

1. **(recomendada) `preparar` sigue al run de origen.** Solo cambia el paso `Informe de verificacion` de `revision-ia.yml`, con los permisos que ya tiene (`contents: read`, `actions: read`):
   - si el run `verificacion-run` no tiene `informe-verificacion-<perfil>`, lee sus jobs (`actions/runs/<id>/jobs`), toma el del perfil (nombre `<perfil>` o `... / <perfil>`) y busca en su log (`actions/jobs/<id>/logs`) la linea `Perfil <perfil>: reutilizada de <sha40> (origen <sha40>, ...)` que escribe el paso `cache por arbol`;
   - con ese `origen` lista los runs del mismo workflow (`workflow_id` del run de verificacion) con `head_sha=<origen>` y se queda con el mas reciente que tenga el artefacto sin caducar (solo los runs que ejecutaron lo suben);
   - lo baja con el mismo tope de 1 MiB / 64 KiB de hoy, y el resumen del check anade `informe-de=run-<id>` para que se vea de donde sale; la seccion cita el artefacto con su run y su origen (`--artefacto`, texto libre en tc2).
   - Si no hay linea, no hay `origen` o el artefacto ha caducado (retencion del repo), `::warning::` con el motivo y se sigue sin informe (`informe=sin-informe`), como hoy.
2. **`verificacion` vuelve a subir el informe del origen.** Todo run tendria su artefacto y cualquier consumidor lo encontraria. Pero `ejecutar` es clase ejecuta (solo `contents: read`, corre `huella-local` del PR) y no puede leer artefactos; hacerlo en el publicador `verificacion` exige darle `actions: read` y `upload-artifact` en clase privilegio, que el lint de F1-8 prohibe (`SUBIDA` solo en clase ejecuta): habria que cambiar el lint y una decision del ADR. Ademas `buscar` tendria que llevar el run del candidato hasta el publicador (campo nuevo en las lineas `arbol=`), cambiando el formato de la cache.
3. **`preparar` lee el check-run `verificacion`** (su resumen trae `origen=`): necesita `checks: read` en `preparar` y, por tanto, en el llamador (`permissions` del job que llama); rompe los llamadores existentes. Descartada.

Por que la 1: arregla el hueco en el unico consumidor afectado sin tocar F1-8, la cache ni los llamadores. Seguridad: la linea de log la puede imitar el codigo del PR (`huella-local` corre antes en el mismo job), pero el informe ya lo escribe codigo del PR y solo se cita como dato; lo peor que consigue es citar otro informe real del mismo repo y workflow. Se valida que el `origen` y los ids son hexadecimales/numericos antes de usarlos.

Piezas:
- `.github/workflows/revision-ia.yml`: paso `Informe de verificacion` (busqueda del origen y aviso de caducidad) y paso `Seleccion y prompts` (texto de `--artefacto` y campo `informe-de=` del resumen); comentario de cabecera.
- `docs/ci/revision-ia.md`: se sustituye el limite de la cache por el comportamiento nuevo y su limite (caducidad del artefacto del origen; runs anteriores al arreglo en cadena siguen funcionando porque `origen` siempre es el run que ejecuto).

## Riesgos aceptados y que los contiene

- Formato del log: depende de la linea `Perfil <p>: reutilizada de ...` de `verificacion.yml`; si cambia, el paso avisa y sigue sin informe (no rompe). Se deja un comentario en ambos sitios.
- Caducidad: si el artefacto del origen ha caducado, sin informe con `::warning::`.
- El E2E necesita fijar en el llamador del repo E2E el SHA de la rama de la tarea (`revision-ia.yml@<sha>` o `tc2-sha`), como en TASK-049. La cuenta de Anthropic sin saldo hace que las revisoras den 400: no afecta, lo que se comprueba es el artefacto `revision-ia-prompts` (seccion `## Informe de verificacion (F5-4)` del run de origen) y el resumen del check.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):
- Criterio 1 (E2E real, sin test automatico: es comportamiento de GitHub Actions): en `charliebk/taskcodetwo-e2e-github`, un PR cuya verificacion ejecuta (run A con artefacto) y un commit vacio que la reutiliza (run B sin artefacto, log `reutilizada de`); el run de revision-ia de B baja el informe de A: `informe=legible ... informe-de=run-<A>` y prompts con la seccion. Se anotan los ids de los runs en el `## Resultado`.
- Criterio 2: el mismo paso con un origen sin artefacto (por ejemplo, el run de verificacion 44740343425 de TASK-049 o forzando un perfil inexistente) da el `::warning::` y `informe=sin-informe`. Se comprueba con el script del paso ejecutado en local contra la API (bash + gh) si el E2E no lo cubre.
- Criterio 4: `cargo xtask verificar --perfil rapida` (incluye el lint de workflows) en verde.

## Lo que necesita decision de una persona

- Aprobar la alternativa 1 (o elegir la 2, que cambia el lint de F1-8 y el formato de la cache).

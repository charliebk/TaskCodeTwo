# Informe de revision — TASK-041 (ronda 1)

- Commit revisado: 171ec22 (contenido de 648720a mas la peticion)
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Resumen

Revision del workflow reutilizable y de la plantilla de GitLab contra F1-7 y F1-8, en el clon `...\pruebas-claude\taskcodetwo\task-041-r1-b`. El workflow de GitHub esta bien resuelto en seguridad:

- Permisos `{}` arriba y por job. `buscar` y `ejecutar` solo tienen `contents: read`, y el publicador solo `checks: write`.
- El publicador no hace checkout y `ejecutar` no usa secretos.
- Ningun `github.event.*` ni `head_ref` va directo a `run:`; todo pasa por `env:`.
- Los perfiles forzados se validan contra lista blanca.
- `actions/checkout` va fijado por SHA con `persist-credentials: false`.

Quedan dos IMPORTANTES: un test del lint que falla en un clon de Windows, y la parte de GitLab sin ejecutar y con una duda de correccion sin resolver.

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido | xtask/src/workflows.rs:709 | implementacion |
| IMPORTANTE-2 | IMPORTANTE | corregido | plantillas/gitlab/verificacion.gitlab-ci.yml:528 | implementacion |
| IMPORTANTE-3 | IMPORTANTE | corregido | .github/workflows/verificacion.yml:190 | plan |
| MENOR-1 | MENOR | corregido | docs/ci/verificacion-reutilizable.md:301 | implementacion |

## Hallazgos

### IMPORTANTE-1 — `cargo test -p xtask` falla en un clon de Windows (CRLF)
- Donde: xtask/src/workflows.rs:709-710 (y `.gitattributes`, sin regla para `*.yml`)
- Que pasa: el test `la_plantilla_de_gitlab_rechaza_datos_del_autor_y_secretos` busca la cadena `"    - $[[ inputs.comando ]] $[[ inputs.argumentos-completa ]]\n"` en el texto leido del fichero. Con `core.autocrlf=true` (valor por defecto de Git para Windows, aqui en `C:/Program Files/Git/etc/gitconfig`) la plantilla se extrae con CRLF y el `contains` falla. `.gitattributes` fija LF para `*.sh`, `*.rs` y `*.toml`, pero no para `*.yml`.
- Reproduccion: en el clon, `cargo test -p xtask` da `test result: FAILED. 16 passed; 1 failed` con `assertion failed: texto.contains(original)` en `xtask\src\workflows.rs:710:5`. `file plantillas/gitlab/verificacion.gitlab-ci.yml` dice `ASCII text, with CRLF line terminators`. La copia del repo principal tambien esta en CRLF, asi que debe fallar igual alli.
- Impacto: `cargo xtask verificar --perfil rapida` (que corre este lint) se pone rojo en cualquier clon nuevo en Windows. Es la plataforma de este proyecto (`windows-latest` en los ejemplos). Un revisor o un runner con clon limpio ve la puerta determinista en rojo por una causa que no es el codigo.
- Sugerencia: normalizar en `leer` (`.replace("\r\n", "\n")`), o fijar `*.yml text eol=lf` en `.gitattributes` (o las dos). Mejor las dos, para que el lint no dependa de la configuracion de Git de quien lo corra. Comprobar que el resto de mutaciones (lineas 636-689) siguen pasando con el fichero en CRLF.

### IMPORTANTE-2 — Criterio "la plantilla de GitLab pasa en un proyecto de pruebas" sin cumplir, con duda real de correccion
- Donde: plantillas/gitlab/verificacion.gitlab-ci.yml:518-528 (y 538, 548, 559); docs/ci/verificacion-reutilizable.md:443-451
- Que pasa:
  - La plantilla nunca se ejecuto. El pipeline 3089899840 termino con «user not being verified» y 0 jobs; el propio doc lo declara «pendiente». El segundo criterio de aceptacion de `tarea.md` sigue sin marcar y sin base para marcarlo.
  - Hay un riesgo de correccion concreto sin resolver. `TC2_PERFILES` se define en `variables:` del job (via `.tc2-verificacion`) y se lee en `rules:if`. Segun mi conocimiento de GitLab, `rules:if` se evalua al crear el pipeline con las variables globales, predefinidas y de proyecto, y no con las `variables:` del job. Eso **no esta verificado** (ni por mi ni por el autor).
  - Si es asi, `$TC2_PERFILES =~ / rapida /` nunca casa. La entrada `perfiles` queda muerta y las reglas `merge_request_event` son las unicas que funcionan. `rapida`, `impacto` y los pushes forzados no se podrian pedir.
- Reproduccion: no verificado. Caso a ejecutar: con la cuenta de pruebas ya verificada, un `.gitlab-ci.yml` que haga `include: remote` de la plantilla con `perfiles: rapida` y empuje una rama sin MR. Esperado si el diseno es correcto: aparece `verificacion-rapida`. Mi prediccion: no aparece ningun job. Se puede adelantar con la API de lint con `dry_run` en cuanto la cuenta este verificada.
- Impacto: la mitad GitLab del entregable (y la unica forma de forzar perfiles en GitLab) puede no funcionar, y nadie lo sabe. El criterio de aceptacion no se puede dar por cumplido con lo hecho.
- Sugerencia: verificar la identidad de la cuenta de GitLab y ejecutar un verde y un rojo reales, como ya dice el doc. Si falla la hipotesis, mover `TC2_PERFILES` a un `variables:` global de la plantilla o evaluar `perfiles` con interpolacion en `rules`. Mientras tanto, no marcar el criterio.

### IMPORTANTE-3 — La concurrencia por repo y perfil da rojos falsos entre PR distintos
- Donde: .github/workflows/verificacion.yml:187-192
- Que pasa: el grupo es `verificacion-<repo>-<perfil>`, sin PR ni rama, y `cancel-in-progress: false`. El autor ya documenta que, con un PR en marcha y uno pendiente, un tercero hace que GitHub cancele el pendiente mas antiguo. Ese PR queda con `ejecutar` cancelado y el publicador escribe `verificacion` en `failure`. En el E2E no se reprodujo.
- Reproduccion: no verificado. Caso: tres PR abiertos a la vez sobre el mismo repo con el perfil completa. Esperado por la semantica documentada de `concurrency`: el PR intermedio queda en rojo sin que su codigo falle.
- Impacto: con dos o tres PR a la vez, que es lo normal en un equipo, hay rojos que hay que relanzar a mano. La motivacion («un runner propio comparte `target/`») no la cubre bien. Un runner propio ya ejecuta un job a la vez, y el caso de varios runners en la misma maquina no esta pedido por F1-7. En runners de GitHub (`windows-latest`) la concurrencia solo estorba. Es un efecto del plan (divergencia 6), no un error de transcripcion.
- Sugerencia: quitar la concurrencia por defecto. Si se quiere para runners propios, hacerla opcional (entrada) o con una clave que incluya el PR o la rama (`github.event.pull_request.number || github.ref`), de modo que solo se encolen las ejecuciones del mismo PR. Si se decide mantenerla, aceptarlo con el motivo en `## Resultado`.

### MENOR-1 — El doc no avisa de que `comando` y `argumentos-*` se interpolan en el shell
- Donde: docs/ci/verificacion-reutilizable.md:301 (seccion de entradas) y .github/workflows/verificacion.yml:204
- Que pasa: `run: ${{ inputs.comando }} ${{ inputs[...] }}` es inyeccion de comandos por diseno, y es aceptable porque el job `ejecutar` no tiene secretos y las entradas las escribe el proyecto. Pero nada dice que el llamador no debe pasar ahi valores del evento (`github.head_ref`, el titulo del PR…). El lint de lista blanca mira el workflow reutilizable, no al llamador, asi que no lo detectaria.
- Reproduccion: lectura. Un llamador con `argumentos-completa: ${{ github.head_ref }}` ejecuta codigo de quien nombre la rama.
- Impacto: bajo, porque `ejecutar` no tiene secretos ni escritura. Riesgo de una mala practica copiada.
- Sugerencia: una linea en el doc («las entradas son literales del proyecto, nunca datos del evento»). No bloquea.

## Comprobado sin hallazgo

- **Seguridad F1-8, workflow:**
  - Permisos, checkout, secretos y `workflow_run` cumplen lo que pide F1-8, y la ausencia de `workflow_run` esta justificada en el doc (divergencia 4).
  - `HEAD_SHA` va por `env:` y `GH_TOKEN` solo existe en el job publicador.
  - Si `buscar` falla, no hay check `verificacion`: el required check queda sin publicar y el PR no se puede mergear, o sea falla cerrado.
  - En un PR de fork el token es de solo lectura y el publicador falla en rojo (documentado).
- **Seleccion de perfiles por evento (F1-7, decision 4):** coincide. PR a cualquier rama da completa. `develop` a `main` da completa y pesada. `merge_group` y `workflow_dispatch` dan completa. `push` da nada. Un perfil fuera de lista da exit 2.
- **ToolsaWeb:** se leyo `scripts\verificar.bat` (lineas 68-79). Sin argumentos es la completa. `--rapida`, `--selectiva` y `--pesada` existen, y otro argumento da `exit /b 2`. La tabla del doc (rapida, `--rapida`; impacto, `--selectiva`; completa, `""`; pesada, `--pesada`) es correcta. No hace falta ningun script nuevo. `RAMA_BASE` esta soportado, que es lo que usa la selectiva.
- **OpenDockTrading:** `scripts\verificar.bat` es un unico flujo sin lectura de argumentos. Basta `comando` con `argumentos-completa: ""`.
- **Tests de xtask:** `cargo test -p xtask` da 16 de 17 en verde y un fallo, el de IMPORTANTE-1. Los tests de mutacion del lint de GitHub pasan.
- **E2E leido:** el run 44552422966 existe y esta en fallo, lo esperado para el caso rojo forzado. No se relanzo nada ni se leyeron los logs de los verdes.
- **No ejecutado:** la suite completa ni `cargo xtask verificar`, como pedia el encargo. Las mutaciones se hicieron solo por lectura de los tests existentes, sin edicion.

## Lecturas fuera del contexto permitido

- docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md y docs/adr/F1-8-seguridad-modelo-de-amenazas.md: pedidos en el encargo, para contrastar decisiones.
- xtask/src/workflows.rs: para entender el lint y el test que falla.
- `.gitattributes`: causa de IMPORTANTE-1.
- `scripts\verificar.bat` de ToolsaWeb y de OpenDockTrading: pedidos en el encargo, solo lectura.
- `gh run view 44552422966`: solo lectura del E2E.

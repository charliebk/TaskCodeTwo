# Informe de revision — TASK-045 (ronda 1)

- Commit revisado: addfcb2da94056304689989a9516567c77a7c466
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

Pase generico completo (el diff de .github/workflows/verificacion.yml, la guia
docs/ci/verificacion-reutilizable.md y la adenda F5-5 de F1-8; el Rust de
xtask/src/workflows.rs lo cubre el revisor de Rust y aqui solo se ejecuta su
suite y se mutan las protecciones del workflow).

## Que se ejecuto

- Clon propio en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-045-r1-calidad`, checkout de addfcb2. No se compilo en el checkout original.
- Verificacion completa del commit: NO repetida (verde a solas el 2026-11-21, segun la peticion).
- `cargo test -p xtask` en el clon: **34 passed, 0 failed** (incluye `el_runner_propio_solo_en_la_release_de_un_privado`, `el_aislamiento_va_antes_del_checkout_en_el_runner_propio`, `sha256sum_solo_por_la_entrada_estandar`).
- El `run:` real del paso «Runner propio» extraido del YAML y ejecutado con bash y jq 1.8.2 con 22 eventos simulados (release valida, repo publico, `private` vacio, fork, head vacio, dispatch, merge_group, PR de tarea, hotfix, `Self-Hosted`, grupo, objeto, JSON ilegible, entrada ausente, numero, booleano, etiqueta suelta, PERFILES vacio y basura).
- El `run:` real del aislamiento ejecutado con Windows PowerShell 5.1 y el shell exacto del paso (`-NoProfile -ExecutionPolicy Bypass -Command ". 'x.ps1'"`), con `RUNNER_OS=Windows` y `Linux`.
- Siete mutaciones del workflow, cada una con `cargo test -p xtask workflows`.
- Consulta de solo lectura con `gh` de los E2E: 44922050486 verde (log: `Runner propio en pesada: release de develop a main en charliebk/taskcodetwo-e2e-privado (privado)` y `Runner propio: usuario nullcad\tc2-runner sin administracion; bash: C:\Program Files\Git\bin\bash.exe`), 44894375522 y 44901386742 rojos solo en `pesada` (los fallos ya corregidos), y el negativo 44831249131 (publico, `workflow_dispatch`): `buscar` en rojo y `matrix.perfil` y `verificacion` saltados, sin encolar nada en el runner.

Mutaciones (todas sobre el workflow del clon, restauradas tras cada una):

| Mutacion | Resultado |
|---|---|
| quitar la comprobacion `REPO_HEAD != REPO` | test rojo |
| quitar la comprobacion de repo privado | test rojo |
| quitar el `if: runner.environment == 'self-hosted'` | test rojo |
| cambiar el SID `S-1-5-32-544` | test rojo |
| quitar `-ExecutionPolicy Bypass` | test rojo |
| `sha256sum "$HUELLA"` (con ruta) | test rojo |
| **desactivar la comprobacion de `jq`** | **verde: sin test que la proteja** (MENOR-3) |

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido (e141ed1) | .github/workflows/verificacion.yml:94-101 | implementacion |
| MENOR-1 | MENOR | corregido (e141ed1) | .github/workflows/verificacion.yml:94,102 | implementacion |
| MENOR-2 | MENOR | corregido (e141ed1) | .github/workflows/verificacion.yml:107-117 | implementacion |
| MENOR-3 | MENOR | corregido (e141ed1) | .github/workflows/verificacion.yml:148 | implementacion |
| MENOR-4 | MENOR | corregido (e141ed1) | .github/workflows/verificacion.yml:137,143,146 y docs/ci/verificacion-reutilizable.md | implementacion |
| MENOR-5 | MENOR | corregido (e141ed1) | .github/workflows/verificacion.yml:11,13-14 | implementacion |

### IMPORTANTE-1 — La guarda de `buscar` solo reconoce el literal `self-hosted`: un runs-on con la etiqueta propia del runner la salta
- Donde: `.github/workflows/verificacion.yml:95` (el `jq` que decide `es`); `docs/ci/verificacion-reutilizable.md` («Etiquetas», que solo advierte de `--no-default-labels`); adenda F5-5 de F1-8, «Lo que queda».
- Que pasa: un `runner-<perfil>` es «propio» solo si es la cadena `self-hosted` o una lista que la contiene. Un runner registrado como indica la guia (`--labels <proyecto>`, con las etiquetas por defecto, que incluyen `self-hosted`) tambien se alcanza con `runs-on: "toolsaweb"` o `["windows","toolsaweb"]`, y entonces el paso dice que ningun perfil usa runner propio, sale con 0 y el job se encola en la maquina de la persona desde cualquier evento y repo. Lo que la adenda y la guia declaran como residual es solo `--no-default-labels`; este caso, con el alta recomendada, no consta.
- Reproduccion: con el `run:` real extraido y `caso.sh PERFILES ENTRADAS EVENTO PRIVADO BASE ORIGEN REPO REPO_HEAD`:
  - `'["pesada"]' '{"runner-pesada":"\"toolsaweb\""}' pull_request false main feat o/r f/r` (repo publico, PR de un fork) → `Runner propio: ningun perfil de este evento lo usa`, exit=0.
  - lo mismo con `'{"runner-pesada":"[\"windows\",\"toolsaweb\"]"}'` → exit=0.
  - Con `["self-hosted","windows","toolsaweb"]` el mismo evento sale con exit=2 («repo publico»). Es decir, la unica diferencia es escribir la etiqueta del runner sin `self-hosted`.
  - Despues del paso en `buscar` solo queda el aislamiento de `ejecutar` (comprueba usuario, no evento), que pasa con `tc2-runner`; el job corre.
- Impacto: la regla «solo repo privado, solo release del mismo repo» (D1, criterio 2 de la tarea) se salta con una configuracion corriente y de buena fe, sin aviso; un error de configuracion deja el runner propio abierto a PR de tarea y de forks. Un llamador hostil siempre puede escribir su propio `runs-on` fuera del reutilizable (eso no lo cubre ninguna guarda de este workflow), pero aqui el fallo es de quien intenta cumplir la regla.
- Sugerencia: cerrar por la otra punta. O bien tratar como propio todo runs-on que no sea una etiqueta de runner de GitHub conocida (`ubuntu-*`, `windows-*`, `macos-*`, con el coste de los runners grandes), o bien repetir la regla del evento (evento, base, origen, repo privado, head del mismo repo) en el paso de aislamiento de `ejecutar`, que corre con `runner.environment == 'self-hosted'` y no depende de la etiqueta, antes del checkout y con los datos del evento por `env:`. Añadir en el test un caso con etiqueta suelta y, en la guia, la limitacion si se mantiene.

### MENOR-1 — Fallo abierto si `PERFILES` no es JSON, y el mensaje de «repo publico» miente cuando `private` llega vacio
- Donde: `.github/workflows/verificacion.yml:94` y `:107-108`.
- Que pasa: `for p in $(printf '%s' "$PERFILES" | jq -r '.[]' ...)` no se ve afectado por `set -e` si el `jq` de la sustitucion falla: la lista queda vacia y el paso termina con «ningun perfil de este evento lo usa», exit 0.
- Reproduccion: `caso.sh 'basura' ... ` → `jq: parse error: Invalid numeric literal...`, `Runner propio: ningun perfil de este evento lo usa`, exit=0; `PERFILES=''` igual. En la practica `PERFILES` lo genera el paso anterior tras validar cada perfil, asi que hoy no es alcanzable.
- Tambien: con `PRIVADO=''` (valor que no llegara) el mensaje dice «repo publico (F1-7, decision 5)» aunque el dato sea desconocido (cae cerrado, bien, pero manda a la persona al sitio equivocado). El E2E 44922050486 demuestra que el valor si llega con `true`, por lo que queda en cosmetica.
- Impacto: ninguno hoy; la guarda es fail-closed en todo lo demas.
- Sugerencia: leer la lista con una asignacion previa (`lista=$(jq ...) || exit 2`) antes del `for`; distinguir `false` de vacio en el mensaje.

### MENOR-2 — Los mensajes dicen por que, pero no que hacer, y el de «no es la release» omite la causa mas probable
- Donde: `.github/workflows/verificacion.yml:108-112`.
- Que pasa: los tres motivos terminan en `F1-8, decision 6`, sin la accion («pon `runner-<perfil>` en un runner de GitHub para este evento; solo la pesada va en el propio»). En un PR de tarea a `main` desde `feature/x` el mensaje es `(evento pull_request, base main)`: la base es correcta y lo que falla es el origen, que el mensaje no imprime; en un `workflow_dispatch` sale `base ` vacia.
- Reproduccion: `caso.sh '["pesada"]' ... pull_request true main feature/x o/r o/r` → `solo en la release, un pull_request de develop a main (evento pull_request, base main)`, exit=2.
- Impacto: quien ve el rojo tiene que ir a la guia para saber la salida; el de la base correcta ve «base main» y no sabe que sobra el origen.
- Sugerencia: imprimir evento, base y origen (o `-` si vacios) y cerrar con la accion en una frase. Se puede dejar como esta y cubrirlo con la guia.

### MENOR-3 — La comprobacion de `jq` del aislamiento no tiene test que se ponga rojo al quitarla
- Donde: `.github/workflows/verificacion.yml:148`.
- Que pasa: es la unica proteccion nueva de la ronda sin mutacion que la delate; el resto de mutaciones de arriba dan rojo.
- Reproduccion: sustituir `if (-not (Get-Command jq` por `if ($false -and -not (Get-Command jq` y correr `cargo test -p xtask workflows` → `23 passed; 0 failed`.
- Impacto: bajo. Sin el paso el job tambien falla cerrado mas tarde (`jq: command not found` en el paso de cache), solo que sin el mensaje que manda a la guia; y lo cubrio el E2E 44901386742. 
- Sugerencia: añadir al test del aislamiento que el texto del paso contenga la comprobacion de `jq`, como ya hace con el SID y el `-ExecutionPolicy`. Lo mismo sirve para `bash` (ver MENOR-4).

### MENOR-4 — El codigo de salida del aislamiento no es el que dicen el comentario, la guia y la adenda; y falta un fallo explicito si no hay `bash`
- Donde: `.github/workflows/verificacion.yml:137-148`; `docs/ci/verificacion-reutilizable.md` («Aislamiento», «Probado en local: ... sale `exit 2`»); adenda F5-5.
- Que pasa: con el shell exacto del paso (`powershell -Command ". 'x.ps1'"`), `exit 2` del script se convierte en codigo de salida **1**. El paso falla igualmente (no cero), asi que la proteccion funciona; pero la guia afirma `exit 2` como probado.
- Reproduccion (Windows PowerShell 5.1): con la cuenta administradora de esta maquina, `RUNNER_OS=Windows powershell.exe -NoProfile -ExecutionPolicy Bypass -Command ". 'ais.ps1'"` → `verificacion: nullcad\nullcad2025 es del grupo Administradores; F1-8, decision 6`, exit=1. Con `RUNNER_OS=Linux` → mensaje de «fuera de Windows», exit=1. Control: un `.ps1` con `exit 2` lanzado con `-File` da 2 y con `-Command ". 'e.ps1'"` da 1.
- Ademas: si no hay `bash`, el paso solo escribe `bash: (ninguno)` y sigue, y el fallo llega despues como error del runner al buscar el shell `bash`, sin el «que hacer». En un runner que no es Windows el paso ni llega a su comprobacion de `RUNNER_OS` (`powershell` no existe en Linux y el runner falla al lanzarlo). Esto ultimo no esta verificado: no hay runner Linux propio.
- Impacto: documental. Si alguien depende del 2 (p. ej. un lint o una prueba que lo espere) se equivoca.
- Sugerencia: decir «sale con error» en la guia y la adenda, o escribir `exit 2` de forma que el codigo sobreviva (`$host.SetShouldExit(2)`); y convertir el `(ninguno)` en fallo con la misma instruccion del alta.

### MENOR-5 — Detalles de redaccion
- Donde: `.github/workflows/verificacion.yml:11` (linea de comentario de 150 columnas, la unica de la cabecera que se pasa del resto); guia: «ningún perfil de este evento lo usa» con tilde, mientras el mensaje real del paso es `ningun`; el alta pide `New-LocalUser` sin `-PasswordNeverExpires`, de modo que la contraseña puede caducar segun la politica local y el servicio dejaria de arrancar (no verificado: no se probo la caducidad).
- Impacto: ninguno funcional.
- Sugerencia: recolocar la linea del comentario; citar el mensaje tal cual; valorar `-PasswordNeverExpires` en el alta.

## Lo comprobado y no hallado

- La regla de la release no se puede saltar por los caminos que lee: repo publico, `private` vacio, fork, head vacio (fork borrado), `workflow_dispatch`, `merge_group`, PR de tarea, hotfix, base distinta, `Self-Hosted` en mayusculas, grupo/objeto, entrada ausente o ilegible: todos `exit 2` con motivo; en un runner de GitHub no altera nada. Y `runs-on` en `ejecutar` se evalua con la misma entrada (`inputs[format('runner-{0}', ...)]`) que lee la guarda.
- La guia coincide con el codigo en: motivos, condicion de «propio», `-NoProfile -ExecutionPolicy Bypass`, SID de SYSTEM y Administradores, linea de exito del aislamiento (literal verificado en el log del E2E), `jq` y `.env`, una instancia por maquina y repo. La seccion «Qué está comprobado y qué no» no sobreafirma salvo lo de `exit 2` (MENOR-4).
- Huella local por la entrada: correcta, con test que falla al revertirla.

## Lecturas fuera del contexto permitido

- `skills/code-quality-reviewer` y `task-workflow/revision.md` del plugin (plantilla y linea del veredicto).
- `xtask/src/workflows.rs` (test `el_runner_propio_solo_en_la_release_de_un_privado`), para juzgar la cobertura de la guarda; y las ejecuciones `gh run view` de solo lectura de los E2E de la peticion.

# Informe de revision — TASK-045 (ronda 2)

- Commit revisado: e141ed154087ea00e2e7fa9536bed2fb42c093f7
- Revisor: code-quality-reviewer
- Veredicto: aprobada
- Alcance: verificacion acotada al diff de las correcciones (IMPORTANTE-1 y MENOR-1..5 de la ronda 1, cada uno verificado; no se revisa la rama entera)

El veredicto aprueba porque IMPORTANTE-1 queda corregido, los MENOR-1..5 quedan corregidos y ningun CRITICO ni IMPORTANTE queda abierto. El unico hallazgo nuevo (MENOR-6, de la propia correccion) es cosmetico y no bloquea el cierre.

## Que se ejecuto

- Clon propio en `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-045-r2-calidad`, checkout de `e141ed154087ea00e2e7fa9536bed2fb42c093f7` (HEAD comprobado). Compilado dentro del clon, no en el checkout original. El clon se borra al terminar.
- `cargo test -p xtask` en el clon: **34 passed, 0 failed** (16 s), incluidos `el_runner_propio_solo_en_la_release_de_un_privado`, `el_aislamiento_va_antes_del_checkout_en_el_runner_propio` y `sha256sum_solo_por_la_entrada_estandar`.
- `cargo fmt -p xtask -- --check`: exit 0.
- Alcance de la puerta: solo el paquete xtask, como pidio la coordinacion. NO se ejecuto el comando de verificacion del proyecto (`cargo xtask verificar --perfil rapida`, en `.taskcode/config.yml`) ni clippy, y no hay un resultado de la ronda en verde para este commit en la peticion.
- `run:` real del paso «Runner propio» (buscar), extraido del YAML y ejecutado con bash 5.3 y jq 1.8.2, con eventos simulados por variables de entorno:

| Caso | Resultado |
|---|---|
| Release valida (pull_request, develop a main, privado, mismo repo) | exit 0: `Runner propio en pesada: release de develop a main en o/r (privado)` |
| Repo publico (`private=false`) | exit 2: `repo publico (F1-7, decision 5)` |
| `private` vacio | exit 2: `no consta si el repo es privado (private=-)` |
| `PERFILES` basura | exit 2: `perfiles ilegibles (basura)` |
| `PERFILES` vacio | exit 0: `ningun perfil de este evento lo usa` |
| Origen `feature/x` | exit 2: incluye `origen feature/x` y `Que hacer:` |
| Base `develop` | exit 2: incluye `base develop, origen develop` |
| `workflow_dispatch` | exit 2: `base -, origen -` |
| Head de fork (`f/r`) | exit 2: `el head del PR viene de otro repo (f/r)` |
| Head vacio | exit 2: `el head del PR viene de otro repo ()` (ver nota 2) |
| `Self-Hosted` en mayusculas, release valida | exit 0 |
| Etiqueta `["windows","toolsaweb"]` sin `self-hosted`, repo publico | exit 0, `ningun perfil...` (limite conocido, lo para el aislamiento; ver IMPORTANTE-1) |
| `runner-pesada` ilegible | exit 2: `runner-pesada ilegible` |

- `run:` real del paso «aislamiento del runner propio», extraido del YAML y ejecutado con `powershell.exe -NoProfile -ExecutionPolicy Bypass -Command ". 'x.ps1'"`:

| Caso | Resultado |
|---|---|
| Release valida, cuenta administradora de esta maquina | exit 1: `nullcad\nullcad2025 es del grupo Administradores` (esperado) |
| Release valida, guarda de Administradores anulada (simulacion de cuenta sin administracion) | exit 0: `Runner propio: usuario nullcad\nullcad2025 sin administracion; bash: C:\Program Files\Git\usr\bin\bash.exe` |
| Repo publico | exit 1: `runner propio fuera de la release de un repo privado` |
| `PRIVADO` vacio | exit 1 (mensaje con `privado ,`, ver MENOR-6) |
| Evento `push` | exit 1 |
| Origen `feature/x` | exit 1 |
| Head de fork | exit 1 |
| `RUNNER_OS=Linux` | exit 1: `fuera de Windows (Linux)` |
| Control: `exit 2` con `-File` / con `-Command ". 'x.ps1'"` | codigo 2 / codigo 1 (confirma MENOR-4, documentado) |

- Mutaciones del workflow del clon, cada una con `cargo test -p xtask workflows`, y restauracion tras cada una (el workflow queda igual que en `e141ed1`):

| Mutacion | Resultado |
|---|---|
| M1: quitar la comprobacion de `jq` del aislamiento | rojo: `el_aislamiento_va_antes_del_checkout_en_el_runner_propio` (MENOR-3 cubierto) |
| M2: quitar la comprobacion de `bash` del aislamiento | rojo: mismo test (MENOR-4 cubierto) |
| M3: quitar la comprobacion `REPO_HEAD` de la regla del evento en el aislamiento | rojo: mismo test (IMPORTANTE-1 cubierto) |
| M4: `elif [ "$EVENTO" != pull_request ]` pasa a `elif false` en buscar | rojo: `el_runner_propio_solo_en_la_release_de_un_privado` |
| M5: el fallo cerrado de la lista de perfiles pasa a `|| true` | rojo: mismo test (MENOR-1 cubierto) |

- E2E, solo lectura con `gh`, repo privado `charliebk/taskcodetwo-e2e-privado`:
  - Run **44926606100** (`pull_request`, develop a main, verde): buscar, `Runner propio en pesada: release de develop a main en charliebk/taskcodetwo-e2e-privado (privado)`; pesada, aislamiento verde con `Runner propio: usuario nullcad\tc2-runner sin administracion; bash: C:\Program Files\Git\bin\bash.exe`, y checkout; completa, aislamiento `skipped` (runner de GitHub). El caller del commit de E2E (`306109b`) apunta a `charliebk/TaskCodeTwo/.github/workflows/verificacion.yml@e141ed154087ea00e2e7fa9536bed2fb42c093f7`, asi que corrio el codigo revisado.
  - Run **44926601827** (`workflow_dispatch`, `runner-pesada` `["windows","tc2-e2e"]` sin `self-hosted`, rojo esperado): buscar, `Runner propio: ningun perfil de este evento lo usa`; pesada, aislamiento en rojo con `verificacion: runner propio fuera de la release de un repo privado (evento workflow_dispatch, base , origen , privado true, head ); ... Que hacer: ...`, y checkout `skipped`. El texto con «Que hacer» solo existe desde e141ed1, asi que este run tambien ejecuto el codigo corregido. No pude leer el caller del commit de E2E `93f41a2` con `gh api` (404), por lo que ese punto se basa en el texto del log, no en el caller.

## Hallazgos

Estado de los hallazgos de la ronda 1:

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido | .github/workflows/verificacion.yml:364-374 (env y regla del evento en «aislamiento del runner propio») | implementacion |
| MENOR-1 | MENOR | corregido | .github/workflows/verificacion.yml:173 (lista antes del `for`, fallo cerrado) y :190 (`private` vacio) | implementacion |
| MENOR-2 | MENOR | corregido | .github/workflows/verificacion.yml:192-197 (base, origen y accion en el motivo) | implementacion |
| MENOR-3 | MENOR | corregido | .github/workflows/verificacion.yml:385 (comprobacion de `jq`, con test que se pone rojo) | implementacion |
| MENOR-4 | MENOR | corregido | .github/workflows/verificacion.yml:381-382 (comprobacion de `bash`) y docs/ci/verificacion-reutilizable.md (codigo de salida 1) | implementacion |
| MENOR-5 | MENOR | corregido | .github/workflows/verificacion.yml:11 (linea de 72 bytes), docs/ci/verificacion-reutilizable.md:353 (`-PasswordNeverExpires`) | implementacion |
| MENOR-6 | MENOR | corregido (76daad5) | .github/workflows/verificacion.yml:374 (mensaje del aislamiento con valores vacios) | implementacion |

### Verificacion de cada hallazgo

**IMPORTANTE-1 — corregido.** Reproduccion de la ronda 1: `runs-on` `["windows","toolsaweb"]` en un PR de fork con repo publico. Con la correccion: `buscar` sigue diciendo «ningun perfil» (limite conocido y documentado), pero el paso «aislamiento» de `ejecutar` (con `runner.environment == 'self-hosted'`, antes del checkout y con la regla del evento por `env:`) para el job con exit 1 antes de sacar codigo. Lo confirma el run 44926601827 en vivo y la simulacion del script con evento `push`, origen `feature/x`, fork o repo publico. La regla quedo cubierta por test (M3 en rojo).

**MENOR-1 — corregido.** `lista=$(...) || {...}` antes del `for`: con `PERFILES=basura` sale exit 2 con `perfiles ilegibles`. Mutacion M5 en rojo. `private` vacio ya no dice «repo publico»: dice `no consta si el repo es privado` (test).

**MENOR-2 — corregido.** El motivo imprime evento, base y origen (`-` si vacios) y cierra con `Que hacer: ...`. Confirmado en los casos de la tabla.

**MENOR-3 — corregido.** M1 (quitar la comprobacion de `jq`) pone rojo `el_aislamiento_va_antes_del_checkout_en_el_runner_propio`. En la ronda 1 esa mutacion daba verde.

**MENOR-4 — corregido.** Existe la comprobacion de `bash` (M2 en rojo) y la guia dice que el paso falla con codigo 1 (`-Command` convierte el `exit 2`). El control confirma 2 con `-File` y 1 con `-Command`, asi que la guia es correcta.

**MENOR-5 — corregido.** La linea 11 mide 72 bytes. La guia incluye `-PasswordNeverExpires` en el alta. (Las lineas 86, 90 y 94 de 138-140 bytes son descripciones de `workflow_call` ya presentes en `addfcb2`; no son comentarios, no se tocan en esta ronda.)

**MENOR-6 — abierto, nuevo, lo introduce la correccion (no bloquea).** El mensaje de `aislamiento` imprime las variables tal cual llegan. Con datos vacios (p. ej. `private` vacio) escribe `privado , head o/r` y `origen ` en blanco; `buscar` usa `-` para esos casos. Sugerencia: aplicar el mismo `${VAR:--}` en PowerShell (`$(if ($env:PRIVADO) { $env:PRIVADO } else { '-' })`) o dejarlo y anotarlo en la guia. Se puede dejar como backlog de `menores-pendientes.md`.

## Notas (fuera de la tabla, no abren hallazgo)

1. El limite de `buscar` con una etiqueta sin `self-hosted` queda documentado en el test y en la guia; lo cubre el paso de aislamiento (IMPORTANTE-1), que es la defensa real.
2. Pre-existente, no de esta correccion: con head vacio (fork borrado), `buscar` dice `viene de otro repo ()`. El texto es correcto como efecto (el head no se puede comparar), pero induce a error. Puede ser MENOR en otra ronda; no se abre aqui porque no lo introduce la correccion.
3. La puerta del proyecto (`cargo xtask verificar --perfil rapida`) no se ejecuto; la verificacion de esta ronda es la de `cargo test -p xtask` mas las mutaciones y los casos de los pasos de workflow.

## Lo comprobado y no hallado

- Ningun camino de la regla de la release se salta con los casos de la tabla de `buscar` ni del aislamiento: los fallos que cubre (repo publico, `private` vacio, fork, head vacio, `workflow_dispatch`, `push`, origen o base distintos, Linux, SYSTEM, Administradores) dan exit no nulo antes del checkout.
- En un runner de GitHub el paso de aislamiento se salta (`skipped` en el run 44926606100 para la completa). No cambia nada.
- No se reproduce una cuenta sin administracion real: solo con la simulacion de la guarda anulada. La prueba de cuenta sin administracion es el E2E 44926606100, verde.
- No hallado: CRITICO, IMPORTANTE nuevo, ni regresion de la suite de xtask.

## Lecturas fuera del contexto permitido

- `.taskcode/config.yml` del repo: para saber el comando de verificacion del proyecto (no ejecutado).
- `C:\Users\nullcad2025\.claude\plugins\cache\taskcode-marketplace\taskcode-plugin\0.16.2\skills\task-workflow\revision.md` y la skill `code-quality-reviewer`: plantilla de hallazgos, reglas de ronda y linea del veredicto.
- Caller del repo E2E (`charliebk/taskcodetwo-e2e-privado`, `.github/workflows/verificacion.yml` en `306109b`, leido con `gh api` de solo lectura): para confirmar el commit pinneado del reutilizable.
- Logs de los runs 44926606100 y 44926601827 (`gh run view --log`, solo lectura): para las lineas de buscar y aislamiento.

# Informe de revision — TASK-028 (ronda 1)

- Commit revisado: 2c3706b7cb68b2e455377ec33ab0766aae68db39 (HEAD de la rama: 9fff220, solo anade la peticion)
- Revisor: code-quality-reviewer
- Veredicto: PENDIENTE (escribelo con: taskctl veredicto TASK-028 aprobada | aprobada-con-correcciones | cambios-solicitados)

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | abierto | crates/tc2/src/init/inventario.rs:302 | implementacion |
| MEN-1 | MENOR | abierto | crates/tc2/src/init/inventario.rs:292 | plan |
| MEN-2 | MENOR | abierto | crates/tc2/src/init/mod.rs:39 | plan |
| MEN-3 | MENOR | abierto | crates/tc2/src/init/mod.rs:304 | implementacion |
| MEN-4 | MENOR | abierto | crates/tc2/src/init/inventario.rs:31 | implementacion |
| MEN-5 | MENOR | abierto | docs/adr/F1-18-hito-de-decisiones.md:455 | plan |
| MEN-6 | MENOR | abierto | docs/adr/F1-5-puertas-en-github.md:368 | implementacion |
| MEN-7 | MENOR | abierto | tareas/03-en-revision/TASK-028/tarea.md:33 | especificacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

## Metodo y entorno

- Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-028-r1-calidad`, rama `feature/task-028-f3-1-taskctl-init-setup`. Todos los comandos git y cargo, con la ruta de ese clon o de mis repos de prueba (`...\taskcodetwo\m\*`). El repo original no se ha tocado.
- `cargo test -p tc2 --test init`: 12 passed, 0 failed (18,5 s). Sin `LNK1104` ni cuelgues de 127.0.0.1 en mis ejecuciones.
- Mutaciones (cada una restaurada con `git checkout`), todas ponen algun test en rojo:
  - C1: quitar `.gitignore` de los ficheros del commit raiz -> 3 tests fallan (entre ellos `repo_vacio_sin_remoto`).
  - C2: `.gitignore` siempre `crea` -> 2 fallan (`init_dos_veces_sin_cambios` entre ellos).
  - C3: `if !op.simular` -> `if true` -> 1 falla (`simular_no_toca_nada_y_coincide`).
  - C4: ignorar las acciones `rechaza` -> 2 fallan (los tests de 0.16 / config invalida).
  - C7: `desinstalar` deja de reconocer las lineas de `.gitignore` -> falla `init_y_desinstalar_se_reconocen`.
- Prueba manual de `tc2 init` (binario del clon, `TC2_IDIOMA=es|en`): repo vacio con y sin `--simular` (plan igual, `simular=si`, nada escrito); segunda ejecucion `sin-cambios`; repo con historia en `master`; repo con config 0.16 (`rechazada motivo=proyecto-016`, mensaje en ambos idiomas, nada escrito; tambien en `--json`); `origin` sin red (`sinred pendiente=no`, codigo 4, nada escrito); fuera de un repo (`no-es-repo`); desde un subdirectorio; `--aplicar-puertas`; `desinstalar --proyecto --simular` tras `init`.
- Suite completa del workspace, clippy y `fmt --check`: ver el apartado final.

## Verificacion criterio a criterio

- C1: `repo_vacio_sin_remoto` existe, aserta rama montada en `git worktree list`, `main == develop` con 1 commit, `ls-tree` exacto, `config::leer == Config::suelo()`, `.gitignore`, `exclude`, HEAD y `status` limpio. Pasa y la mutacion lo pone rojo. Nota: el criterio dice que `exclude` lleva "las dos lineas" y el test (y el codigo) usa `/tareas/` y `/.taskcodetwo/worktrees/`, no `/.taskcodetwo/` (MEN-7).
- C2: `init_dos_veces_sin_cambios` aserta `sin-cambios`, refs, worktrees, `exclude` y arbol (instantanea de ficheros y huella de git) iguales. Pasa y muta. Lo que se aserta es mas debil que el enunciado (ver desviacion 1).
- C3: `simular_no_toca_nada_y_coincide`: arbol y `.git` iguales, no crea `.git/taskcodetwo/`, comprueba `--json` y compara `linea()` del plan simulado con el real. Pasa y muta.
- C4: `init_en_repo_016_rechaza` (config y `tareas/` normal, codigo 3, arreglo, en/es, nada escrito) y `init_con_config_invalida_no_la_pisa` (`formato-nuevo`, `config-invalida`). Pasan; la mutacion los pone rojos.
- C5: `sin_remoto_completa_al_anadirlo` y `sin_red_no_escribe` existen y prueban lo dicho (con y sin `--simular`). Pasan. No mutado (no se pedia).
- C6: no tiene test propio con nombre; lo cubre un tramo de `sin_remoto_completa_al_anadirlo`: 5 lineas `persona` con la orden exacta, propuesta en JSON y todas las peticiones del doble HTTP son `GET`. Prueba lo que dice. Pasa.
- C7: `init_y_desinstalar_se_reconocen`: lo `crea` con `dueno=taskcodetwo` aparece en `desinstalar`, y el `.gitignore` como `persona`. Pasa y muta. No cubre `main`/`develop` (son `dueno=proyecto`, desviacion 2).

## Valoracion de las desviaciones declaradas

1. C2 deja `pendiente` en la segunda ejecucion: aceptable. Son `personas` (`sin-remoto`), `workflow` (`sin-plantilla`) y los rulesets; no pueden pasar a `existe` sin remoto ni plantilla. La que falla es la redaccion del criterio (MEN-7). El test aserta que las acciones que no son `crea` salen identicas.
2. `main` y `develop` con `dueno=proyecto`: aceptable. Son ramas de la persona y `desinstalar` no debe borrarlas. Consecuencia: C7 no puede cubrirlas; `desinstalar` no las lista (comprobado a mano).
3. Con historia, las ramas quedan como `persona`: aceptable por D3. Pero el flujo queda incompleto, ver MEN-1.
4. HEAD en `master`: aceptable y probado (`head_en_master_queda_para_la_persona` ejecuta la orden de la linea y deja el worktree limpio).
5. `personas.yml` al disco y no al commit raiz: aceptable. La misma consecuencia que MEN-1 para `config.yml` y `.gitignore` con historia.
6. `tc2 init --aplicar-puertas` no existe: aceptable por plan (TASK-118), pero la linea la anuncia como si existiera, ver MEN-2.
7. `desinstalar` muestra los rulesets como `persona motivo=si-existe` sin consultar la red: aceptable; `desinstalar` es offline por diseno y el motivo lo dice.
8. D4 sin anotar en ningun ADR: confirmado, ver MEN-5. El plan decia «con la divergencia anotada» y no lo esta.

La adenda de F1-5: es correcta y consistente con el ADR (checks inexistentes hasta F5-1/F6, `integration_id` de relleno que el propio ADR prohibe aplicar, linea 263-265). Solo esta en la cabecera; ver MEN-6.

## IMP-1 — `init` crea y publica un `develop` sin historia comun con el `main` que ya tiene `origin`

Fichero: `crates/tc2/src/init/inventario.rs:302` (la rama `else` de `ramas()`, que elige `Crea` mirando solo `hay_commits` local).

Reproduccion (hecha, con repos de prueba en `...\taskcodetwo\m`):

1. `git init --bare -b main rem.git` con un commit en `main` y sin `develop`.
2. `git init -b main loc` (sin commits) y `git remote add origin ../rem.git`.
3. `tc2 init` desde `loc`.

Resultado: `main` sale `persona motivo=en-el-remoto`, pero `develop` sale `decision=crea` y se aplica: commit raiz nuevo y `git push origin develop`. En `rem.git`, `git merge-base main develop` devuelve 1 (sin ancestro comun): el remoto compartido queda con un `develop` de historia no relacionada con su `main`, y la rama de estado tambien se publica. No hay force, no se pierden datos, pero es la secuencia clasica `git init` + `git remote add` sobre un proyecto que ya existe, y deja un estado dificil de arreglar (un `develop` de historia ajena publicado que el resto del equipo puede ya haber bajado). Lo mismo, simetrico, si el remoto solo tiene `develop`.

Arreglo propuesto: si `remoto` y alguna de `main`/`develop` esta en el remoto (`remotas` no vacio), la que falta debe ser `persona` (motivo `en-el-remoto-sin-la-otra` o similar, con orden `git fetch ... && git branch develop origin/main`) y nunca `Crea`; anadir un test con un `origin` bare que ya tenga `main`.

## Menores

- MEN-1 (`inventario.rs:292`): con historia, `init` deja `.taskcode/config.yml` y `.gitignore` sin versionar (`git status`: `?? .gitignore`, `?? .taskcode/`) y la orden que da a la persona, `git branch main HEAD`, crea ramas **sin** esos ficheros. Ninguna linea ni el aviso dicen que hay que hacer commit. El equipo que clone no tendra la config. Anadir una linea `persona` (`git add .taskcode .gitignore && git commit ...`) o decirlo en el aviso final.
- MEN-2 (`mod.rs:39`): cada linea `ruleset` da `orden=tc2 init --aplicar-puertas`, y ejecutarla hoy sale `init uso motivo=argumentos` (codigo 2, comprobado). Esta previsto (TASK-118), pero la persona recibe una orden que falla. Anadir al motivo o al aviso que la orden llega con TASK-118.
- MEN-3 (`mod.rs:304`): `t()` usa `Path::display()` sobre rutas montadas con `join` y `/`, y en Windows las lineas salen con separadores mezclados: `ruta=C:/.../repo\.taskcode/config.yml`, `.../.git\info\exclude`. Es estable pero feo y no casa a ojo con la salida de `git`; si el contrato quiere rutas normalizadas, normalizar en `t()`. Los tests ya compensan con `replace('\\', "/")` en varios sitios.
- MEN-4 (`inventario.rs:31`): `faltan()` compara linea a linea; un `.gitignore` con `/tareas` (sin barra final) recibe ademas `/tareas/` (duplicado inocuo, comprobado). Mejor comparar tambien la forma sin la barra final.
- MEN-5 (`docs/adr/F1-18-hito-de-decisiones.md:455`): F1-18 (contrato en `init`: crear `hito-decisiones.yml` abierto) y F1-14 (manifiesto de recursos en `init`, lineas 578-594) siguen diciendo que `init` lo hace; D4 lo saca de F3-1 pero ningun ADR lleva la adenda. Solo esta en el plan de la tarea y en la memoria. Debe anotarse (o crear las tareas que lo recojan) antes de cerrar.
- MEN-6 (`docs/adr/F1-5-puertas-en-github.md:368`): la adenda esta solo en la cabecera; la tabla de operaciones (`aplicar-puertas` | `init` (F3-1)...) y `PLAN-TASKCODE-2.md:551` («F3-1 aplica las puertas o explica por que no puede») siguen igual. Un lector que llegue a la tabla no ve la excepcion. Anadir una nota en la fila o referencia a la adenda.
- MEN-7 (`tareas/03-en-revision/TASK-028/tarea.md:33`): el enunciado de C2 dice «todas las acciones `decision=existe`» y el de C1 «`exclude` con las dos lineas» sin decir cuales; implementacion y tests usan `existe|pendiente` y `/.taskcodetwo/worktrees/`. Corregir el texto del criterio para que diga lo que se prueba.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/init/*.rs`, `crates/tc2/tests/init.rs`, `crates/tc2/src/desinstalar/proyecto.rs` y `crates/tc2/src/huella.rs`: necesarios para comprobar criterio a criterio y hacer las mutaciones pedidas por la peticion de esta ronda.
- `docs/adr/F1-18-hito-de-decisiones.md`, `docs/adr/F1-14-distribucion-e-instalacion.md`, `docs/plan/PLAN-TASKCODE-2.md` (lineas citadas) y `tareas/00-planificadas/TASK-118/tarea.md` (cabecera): para valorar la desviacion 8 y la adenda de F1-5.
- `tareas/03-en-revision/TASK-028/planificacion/plan-final.md`: contexto permitido.

## Suite completa, clippy y fmt

- `cargo clippy --workspace --all-targets`: sin avisos. `cargo fmt --check`: limpio.
- `cargo test --workspace`, primera pasada: todos los binarios en verde salvo uno (3 passed, 1 failed, 94 s) que no pude identificar porque mi filtro de salida solo guardo las lineas `test result`. Coincidio con otro `cargo` de otro revisor corriendo en la maquina (el enlazador `LNK1104` y los cuelgues de 127.0.0.1 son conocidos en este entorno, por eso lo trato como fallo de entorno, no demostrado). Segunda pasada completa con `--no-fail-fast` y filtro de `FAILED|failed|panicked|LNK`: sin ninguna coincidencia, es decir, sin fallos. Dejo constancia de que el fallo de la primera pasada no se reprodujo y no se identifico.

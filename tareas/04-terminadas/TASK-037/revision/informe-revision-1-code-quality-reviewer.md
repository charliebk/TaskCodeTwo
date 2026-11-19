# Informe de revision — TASK-037 (ronda 1)

- Commit revisado: b9a6a55f86af56fe615da1af20a1480ea2d9ae0a
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/estado/transaccion.rs:323 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/plan/plantillas.rs:75 | plan |
| MEN-3 | MENOR | abierto | docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:91 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Sin CRITICO ni IMPORTANTE. Veredicto propuesto: aprobada-con-correcciones (los tres MENOR no bloquean).

### MEN-1 — D4a (pendientes fuera del commit firmado) sin ningun test que se ponga rojo

- **Donde**: `crates/tc2/src/estado/transaccion.rs:323` (`if remoto && sello == Sello::Normal`).
- **Que pasa**: el codigo es correcto, pero la garantia «un commit firmado lleva solo lo que firma la persona» no la protege ningun test. El plan de pruebas de la tarea no la incluia y `tests/aprobar.rs` no menciona pendientes.
- **Reproduccion**: en el clon, mutar la linea a `if remoto {` y correr `cargo test -p tc2 --lib estado` (27 pasan, 0 fallan). `grep -n pendiente crates/tc2/tests/aprobar.rs` no da nada, asi que el E2E tampoco lo cubre (no lo ejecute con la mutacion, pero no tiene ninguna asercion sobre pendientes).
- **Impacto**: una regresion futura mezclaria operaciones pendientes de otra persona o del agente dentro de un commit firmado por la persona, sin que la suite avise. Hoy no hay comportamiento incorrecto, por eso MENOR (la skill lo pondria IMPORTANTE solo si fuera comportamiento incorrecto en un caso real).
- **Sugerencia**: un unitario con un `Git` simulado o un E2E con una pendiente previa (escritura sin red) que compruebe que tras `approve` la pendiente sigue en `refs/taskcodetwo/pendientes/` y el arbol firmado no la contiene.

### MEN-2 — Los cambios pedidos no llegan a la peticion de la ronda siguiente

- **Donde**: `crates/tc2/src/plan/plantillas.rs:75` (y las plantillas de peticion que no los referencian).
- **Que pasa**: `approve --pedir-cambios` cierra de verdad el hueco declarado en TASK-036 (verificado: el unitario `cambios_pedidos_cierran_la_ronda` y el E2E `d1a_pedir_cambios_y_plan_abre_la_ronda_2` abren la ronda 2 con los mismos roles). Pero `peticion-plan-<n+1>.md` no menciona `cambios-pedidos-<n>.md` (`grep cambios-pedidos` solo da codigo de plan/ y tests). Quien replanifica solo lo encuentra si la salida `approve ok ... cambios-pedidos=...` se lo pasa a mano.
- **Reproduccion**: `grep -rn "cambios-pedidos" crates --include=*.rs` fuera de `aprobar.rs` y tests: solo `plan/mod.rs`, `operacion.rs` y `plantillas.rs` (nombre y testigo), ninguna plantilla de peticion.
- **Impacto**: la retroalimentacion de la persona queda guardada pero no enrutada; probablemente es del orquestador (F4-4), por eso MENOR y no bloquea el criterio de la tarea.
- **Sugerencia**: una linea en la peticion de la ronda `n+1` («lee `brainstorm/cambios-pedidos-<n>.md`») o anotarlo explicitamente como pendiente de F4-4 en la adenda de F1-10.

### MEN-3 — La adenda de F1-4 omite un motivo `uso`

- **Donde**: `docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:91`.
- **Que pasa**: la lista de `uso` (2) de `approve` es `sin-id`, `id-invalido`, `sin-fichero`, `decidido-por`, `argumentos-de-mas`, pero `approve TASK-001 --pedir-cambios nofichero` da `approve uso motivo=no-se-lee` (exit 2).
- **Reproduccion**: `tc2.exe approve TASK-001 --pedir-cambios nofichero </dev/null` en el clon: `approve uso motivo=no-se-lee`, codigo 2.
- **Impacto**: documentacion incompleta del contrato de salida; no cambia el comportamiento.
- **Sugerencia**: anadir `no-se-lee` a la lista de la adenda (y, si se quiere, a la de `plan guardar` por coherencia).

## Que se ejecuto (resumen de la revision, sin mas hallazgos)

- Suite de la ronda: `cargo xtask verificar --perfil rapida` en verde (exit=0) sobre 7b98d98, log `%LOCALAPPDATA%/pruebas-claude/TaskCodeTwo/verif-037.log` (termina en «verificacion Rapida en verde»); `git diff --stat 7b98d98 b9a6a55 -- crates docs` vacio (codigo identico al revisado). No se repitio.
- En el clon de b9a6a55 con `-- --test-threads=4`: `--test aprobar` 6/6, `--test plan` 6/6, `--lib plan` 29/29.
- CLI real (`target/debug/tc2.exe`, sin TTY): `approve TASK-001` -> `rechazada motivo=identidad rol=persona` (3) con diagnostico que dice que lo lanza una persona desde su terminal con `TC2_TOKEN_PERSONA` y clave de firma; sin tocar nada. `--decidido-por automatico` -> `uso motivo=decidido-por` (2); `--pedir-cambios` sin fichero -> `sin-fichero`; sin id -> `sin-id`.
- Mutaciones: quitar `--gpg-sign`/`-S<clave>` de la lista prohibida del puerto -> 2 tests rojos; quitar `!cerrada` en `Planificar` -> 1 test rojo; quitar la exclusion de pendientes con sello firmado -> verde (MEN-1).
- Git 2.56 en Windows, firma: `commit-tree -S` sin `user.signingkey` sale con 128, con clave ssh inexistente con 1 y con `gpg.format` por defecto sin clave con 1; coincide con lo que dice la adenda de F1-6. El mensaje `FIRMA_FALLO` dice que configurar (gpg.format y user.signingkey) y desde donde lanzarlo.
- Adendas de F1-6, F1-4 y F1-10 contrastadas con el codigo: dicen la verdad salvo MEN-3. No verificado (lo declara la propia adenda): la peticion de frase de paso y el toque FIDO con una persona real en el terminal de Windows.
- Fuera de alcance, no hallazgo: lote F4-3 (TASK-038), orquestador F4-4, comprobacion de la firma frente a la plataforma (check `carril`, F6).

## Lecturas fuera del contexto permitido

- `docs/plan/PLAN-TASKCODE-2.md` (linea de F4-2) y `crates/tc2/**` (codigo completo de la rama, no solo el diff de docs): necesarios para reproducir y mutar los tests nuevos, como pide la peticion.
- `tareas/03-en-revision/TASK-037/tarea.md` y `planificacion/plan-final.md`: dentro del contexto permitido.
- `C:\Users\nullcad2025\.claude\plugins\cache\...\skills\task-workflow\revision.md`: no lo lei; segui la skill del revisor y las instrucciones de la peticion.

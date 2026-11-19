# Informe de revision — TASK-036 (ronda 1)

- Commit revisado: 6ecf89825e4a41e13abdd7f70a2fa08bcbf65ff6
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/plan/operacion.rs:270 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/plan/operacion.rs:270 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/src/i18n/catalogo/plan.rs:5 | plan |
| MEN-4 | MENOR | abierto | crates/tc2/src/plan/plantillas.rs:171 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Propuesta de veredicto: aprobada (no hay CRITICO ni IMPORTANTE; los cuatro MENOR son corregibles sin otra ronda).

### MEN-1 — `plan` no comprueba `plan_aprobado`, `plan guardar` si

- Donde: `crates/tc2/src/plan/operacion.rs:267-272` (la validacion de estado/pausa) frente a `crates/tc2/src/plan/guardar.rs:129`.
- Que pasa: `plan guardar` rechaza con `plan-aprobado` («el plan de la tarea ya esta aprobado: no se cambia su planificacion»), pero `plan` sobre una tarea `en-diseno` con `plan_aprobado: true` (aprobada y todavia sin `start`) escribe peticiones nuevas, una fila `plan` y deja `plan_aprobado: true`: la aprobacion pasa a cubrir un juego de roles y un carril que nadie aprobo.
- Reproduccion: test unitario temporal en el clon (`Planificar::nueva("TASK-001", ana, Some(Carril::RiesgoAlto), ..)` sobre una `tarea.md` con `estado: en-diseno` y `plan_aprobado: true`). Resultado: `Ok` con 5 peticiones, `plan-final.md` y `tarea.md`; el `tarea.md` resultante sigue con `plan_aprobado: true`. Borrado despues.
- Impacto: bajo hoy (hace falta pedirlo a proposito y `approve` es de TASK-037, que puede cerrarlo), pero es una incoherencia entre las dos ordenes de la misma tarea.
- Sugerencia: rechazar `plan` con `plan-aprobado` (mismo texto) o dejarlo declarado en la adenda del ADR para que TASK-037 lo decida.

### MEN-2 — Protecciones nuevas sin un test que se ponga rojo al quitarlas

- Donde: `operacion.rs:270` (`pausada`), `guardar.rs:116-125` (`de-otra-persona` en `plan guardar`), `guardar.rs:129` (`plan-aprobado`), `carril.rs:95-103` (orden del tope de `hotfix` y la configuracion), `mod.rs:128-143` (`vacio`, `no-utf8`, `grande`).
- Que pasa: mutadas en el clon (`cargo test -p tc2 --lib -- plan carril`, 21 tests), siguen en verde: quitar el rechazo `pausada`; quitar el `return Err` de `de-otra-persona` en `guardar`; desactivar `plan-aprobado`; invertir el orden (aplicar la configuracion y despues el tope de `hotfix`). El orden esta ademas afirmado en la adenda del ADR («el tope ... se aplica antes y `politica_carriles` despues»): la tabla de `roles_de_diseno_por_carril` no tiene ningun caso `hotfix` con configuracion. En `tests/plan.rs` tampoco hay caso con fichero vacio, no UTF-8 o grande (`grep` sin resultados); a mano, el vacio da `plan uso motivo=vacio` y un fichero inexistente `no-se-lee`, ambos correctos.
- Matan la mutacion: CRLF a LF (`guarda_solo_en_diseno_con_peticion`), ceros de `salida-...-0.md`, hueco de criterio, `abierta` siempre falsa y `--carril` que baja (todos en rojo).
- Reproduccion: `sed -i 's/if cuerpo.en_pausa() {/if false \&\& cuerpo.en_pausa() {/'` en `operacion.rs`, y analogas; `cargo test -p tc2 --lib -- plan carril` da `21 passed`.
- Impacto: el comportamiento actual es correcto (comprobado en el CLI real y por lectura); solo falta la red de regresion. Es MENOR y no IMPORTANTE porque no hay comportamiento incorrecto, aunque la skill clasifica una proteccion sin test como IMPORTANTE: lo dejo a criterio de quien cierra.
- Sugerencia: un caso por rechazo en `rechazos`/`guarda_solo_en_diseno_con_peticion` y una fila `hotfix` con `politica_carriles` en la tabla de carril.

### MEN-3 — El uso anuncia `--carril errata`, que nunca puede aceptarse

- Donde: `crates/tc2/src/i18n/catalogo/plan.rs:5` (`USO_PLAN`) y `Carril::desde` en `operacion.rs:284-295`.
- Que pasa: el carril declarado es como poco `fix`, y `--carril` solo sube, asi que `errata` siempre sale `carril-baja`.
- Reproduccion: en un repo con remoto, `tc2 plan TASK-002 --carril errata` da `plan rechazada id=TASK-002 motivo=carril-baja` (codigo 3). El mensaje dice que hacer, pero la ayuda ofrecia el valor.
- Impacto: confusion menor. Sugerencia: quitar `errata` del texto de uso, o decirlo en el rechazo.

### MEN-4 — Textos de las peticiones algo imprecisos

- Donde: `crates/tc2/src/plan/plantillas.rs:171` y `:188`.
- Que pasa: (a) «Numero final con el carril `riesgo-alto`: 5 roles» cuenta el rol condicional, mientras la frase que sigue habla de «el carril y `politica_carriles`»; (b) la ronda 2 dice «Ya existe un plan en `plan-final.md` de una ronda anterior» aunque el fichero sea solo el esqueleto vacio (si nadie ejecuto `plan guardar`).
- Reproduccion: salida de `tc2 plan TASK-001 --carril riesgo-alto` tras una primera ronda con etiqueta `rendimiento` (peticion-unificador-2.md).
- Impacto: cosmetico; el agente lo lee y puede buscar un plan que no existe. Sugerencia: decir «el esqueleto o el plan» o distinguir si el fichero tiene contenido.

## Lo comprobado sin hallazgos

- Suite: se cita `cargo xtask verificar --perfil rapida` en verde (exit=0) sobre dbe7b5f, log `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\verif-036.log` (acaba en «verificacion Rapida en verde»); `git diff --stat dbe7b5f 6ecf898 -- crates docs` vacio. No se repitio. En el clon con `-- --test-threads=4`: `cargo test -p tc2 --test plan` 5 passed (71 s), `--lib plan` 19 passed, `--lib carril` 3 passed.
- Criterio 1 (mismos roles que la 0.16): los 57 textos de `roles-brainstorm.ts` (0.16.2) de mas de 20 caracteres (preguntas, mira, no mira, entregables) estan literales en `plan/roles.rs`; ids, orden de la escalera, etiquetas del condicional (`rendimiento`, `wasm`, `workers`, `3d`), minimo 2 para condicionales y `brainstorm-unificador` coinciden. La divergencia (n=5 suma el condicional) esta declarada en el codigo y en la adenda.
- Criterio 2 (uno en fix, todos en riesgo alto): en el CLI real, fix da `roles=arquitectura forma=redaccion` (un `peticion-plan-1.md`); `--carril riesgo-alto` da los cuatro mas rendimiento si hay etiqueta (5), con `peticion-unificador-N.md`; un `hotfix` queda en 1 y con `--carril riesgo-alto` en 4. Coincide con la fila de la tabla de F1-10 y con F4-1/RR-02 (proporcional al carril).
- Heuristica: pesos (1/2/1/1/2/1), umbral 5, cortes 0/3/5/7, roles 0..4, las 14 palabras y el tope de hotfix (minimo, no sustitucion) coinciden con `heuristica-complejidad.yml` de la 0.16.2; el test `senales_y_puntos` y `resolucion` los fijan.
- Rondas: solo se anaden; repetir con los mismos roles y carril da `plan sin-cambios`; subir el carril abre la ronda 2 sin pisar `plan-final.md`; `plan` sin `--carril` tras `riesgo-alto` conserva el carril (sin-cambios). El hueco para re-planificar con los mismos roles tras «pedir cambios» (peticiones con el enunciado antiguo, `sin-cambios`) esta declarado en la adenda para `approve` (TASK-037): no es hallazgo.
- Adenda del ADR F1-10: cada punto contrastado con el codigo (orden del tope, n=5, modo congelado, rondas, sin catalogo con `pendiente=no`, rechazos `pausada`, huecos de plantilla, `sin-peticion`, fichero vacio, CRLF, linea `destino=`); dice la verdad.
- Mensajes: los rechazos dicen que hacer (`reanudar` existe como orden, `escribelo antes de planificarla`, `--carril solo sube`, `guarda la salida de un rol que tc2 plan haya lanzado`); solo `plan-aprobado` y `carril-baja` no ofrecen salida (menor).
- Entrada externa: `plan guardar` solo admite `plan-final.md` o `brainstorm/salida-<rol>-<n>.md` (sin `..`, `\`, ceros, subcarpetas, desbordes), tope de 1 MiB con lectura acotada, UTF-8 y no vacio; sin recorrido de directorios posible.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/plan/*.rs`, `crates/tc2/src/carril.rs`, `crates/tc2/tests/plan.rs`, `crates/tc2/src/i18n/catalogo/plan.rs`: el codigo a revisar (mi dominio es el ADR, pero la peticion pide mutar y ejercitar el CLI).
- `C:\Users\nullcad2025\.claude\plugins\cache\taskcode-marketplace\taskcode-plugin\0.16.2\src\core\roles-brainstorm.ts` y `scripts\heuristica-complejidad.yml`: la peticion manda compararlos.
- `docs/plan/PLAN-TASKCODE-2.md` (fila F4-1) y `docs/adr/F1-10-...` (tabla de politica y decisiones 1 y 2): criterios F4-1 y RR-02.
- `tareas/.../TASK-036/tarea.md`, `planificacion/plan-final.md`: enunciado y plan de la tarea.

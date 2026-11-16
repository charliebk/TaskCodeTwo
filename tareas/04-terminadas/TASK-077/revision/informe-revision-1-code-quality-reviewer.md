# Informe de revision — TASK-077 (ronda 1)

- Commit revisado: 7a8d6e5 (la peticion cita eebf5f7; el HEAD de la rama solo anade la peticion, sin codigo)
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | abierto | crates/tc2/src/identidad.rs:359 | implementacion |
| MEN-1 | MENOR | abierto | crates/tc2/src/plataforma/personas.rs:282 | implementacion |

### IMP-1 — `personas.yml` se lee de `origin/develop` sin fetch y el rechazo manda a pedir un PR que ya esta fusionado

`personas_de_base` (y por tanto `tc2 identidad` y `quien_ejecuta`) lee `origin/develop` tal y como esta en local. No hace fetch ni avisa de que puede estar atrasado. Cuando una persona acaba de entrar en la lista por un PR ya fusionado y todavia no ha hecho `git fetch`, `tc2 identidad` la rechaza como si no estuviera.

Reproduccion (real, con consola TTY y mock de GitHub en 127.0.0.1:18765):
1. Remoto con `develop` y `personas.yml` solo con `{id: 9}`. El clon A lo trae y queda con `origin/develop` en ese commit.
2. Otro clon empuja a `develop` un `personas.yml` que anade `{id: 1001, login: "carlos"}`. El clon A no hace fetch.
3. `TC2_TOKEN_PERSONA=x tc2 identidad` en el clon A, con el mock devolviendo `carlos`/1001/`User`, sale:
   ```
   identidad rechazada motivo=identidad rol=persona cuenta=fuera-de-lista
   la cuenta carlos (id 1001) no esta en .taskcode/personas.yml: pide que la anadan con un PR, o usa el token de una persona de la lista
   ```
El mensaje (`CUENTA_FUERA_DE_LISTA`, `i18n/catalogo/identidad.rs:53`) manda a la persona en la direccion equivocada: el PR ya existe. Lo mismo vale para `CACHE_FUERA_DE_LISTA` en el camino del agente.

Arreglo propuesto, a elegir: anadir al mensaje «si ya la anadieron, haz `git fetch origin`», o hacer el fetch dentro de `tc2 identidad` (con TTY) antes de leer la lista, con `sinred` si falla. Pide un test que cubra el caso.

### MEN-1 — Aserto debil en el test de correo repetido (C3)

`personas.rs:282`: `assert!(error(correo).contains("x@y") || error(correo).contains("X@Y"))` no comprueba que se nombren las dos entradas, que es lo que pide C3. Si el diagnostico nombrara solo una, o ninguna entrada, el test seguiria en verde. Para el `id` y el `login` el test si comprueba `1 (..)` y `2 (..)`; el del correo deberia hacer lo mismo. (La mutacion «sin deteccion de correo repetido» si lo pone en rojo, asi que hay red de regresion; el defecto es solo la precision.)

## Reproduccion y evidencia (resumen)

Puerta determinista, en clon limpio en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-077-r1-calidad` (rama de la tarea, `CARGO_TARGET_DIR` aparte):
- `cargo fmt --check`: limpio. `cargo clippy -p tc2 --all-targets -- -D warnings`: sin avisos.
- `cargo test -p tc2 --lib --test tarea`: 87 pasan, 1 ignorado (el ignorado ya existia).
- `cargo test -p tc2 --test identidad`: 4/4 en verde en 4 de 5 ejecuciones (2 en serie con `--test-threads=1`, 2 en paralelo). La primera ejecucion, en paralelo y justo tras compilar, fallo en 2 tests con `identidad sinred` tras mas de 60 s: es el cuelgue de 127.0.0.1 de esta maquina (el cliente agota el tiempo y devuelve `sinred`), no un fallo del codigo. Repetido, pasa. No es un hallazgo.
- No se ejecuto la suite entera del workspace; solo los suites que toca el diff (`--lib`, `tarea`, `identidad`). La peticion no traia verificacion de la ronda.

Criterio por criterio (el test existe, prueba lo que dice y pasa):
- C1: `tests/identidad.rs::identidad_con_terminal_guarda_la_cache_en_github_y_gitlab`: linea exacta `identidad ok rol=persona id=1001 login=carlos`, una sola peticion `GET /user`, cache con proveedor/id/login/verificado, y JSON. En GitHub y GitLab. OK.
- C2: `cuenta_fuera_de_lista_bot_o_web_flow_no_toca_la_cache` (3 casos x 2 plataformas, compara los bytes de la cache antes y despues) y el unitario `bot_nunca_asignado_y_sin_tty_es_agente`. OK.
- C3: `id_o_login_repetido_se_rechaza_nombrando_las_dos_entradas` (ver MEN-1 para el correo). OK.
- C4: `se_ensena_el_login_y_el_nombre_no_lo_sustituye`. OK.
- C5: `tests/tarea.rs::asignado_a_persona_ida_y_vuelta_y_comparacion_por_id` (byte a byte, parser estandar, `u64::MAX`, comparacion por id, 9 formas invalidas). OK.
- C6: unitario de `identidad.rs`. OK.
- C7: `sin_red_no_toca_la_cache_y_sin_cache_se_pide_tc2_identidad` (sinred codigo 4, cache intacta, `cache=falta` y `cache=caducada` con diagnostico en es y en en que cita `tc2 identidad`) mas el conteo de peticiones en C1 y el unitario `cache_vigencia_y_rechazos`. OK. La linea `identidad sinred` coincide con el formato del contrato F1-4 reajustado en C1 y C7.
- C8: `sin_plataforma_reconocida_se_rechaza`. OK.
- C9: `asignado_a_de_texto_de_la_0_16_es_heredado` (heredado, conversion por correo sin distinguir mayusculas, vacio es `null`, reescritura de un heredado en formato 2 byte a byte). OK. Nota: `convertir_heredado` y `quien_ejecuta` no tienen aun llamador de produccion (solo la orden `identidad` usa `quien_ejecuta`); el plan lo dice: los conectan F3-6 y F3-10. No es un hallazgo; solo conviene que esas tareas lo recojan.

Mutaciones (se aplico el cambio, se ejecuto `--lib --test tarea --test identidad` y se revirtio; todas se detectaron):
- C2: quitar el rechazo de `web-flow`, quitar el de `Bot` (`tipo != "User"`), quitar el de fuera de lista: en rojo en `bot_nunca_asignado_y_sin_tty_es_agente`. Tocar la cache antes de verificar: en rojo `cuenta_fuera_de_lista_...` y `sin_red_...`.
- C5: `misma_persona` comparando por login, y `la_lleva` que acepte cualquier id: en rojo `asignado_a_persona_ida_y_vuelta_y_comparacion_por_id`.
- C6: `decidido_por` devolviendo `persona` para el agente, y `quien_ejecuta` sin TTY devolviendo `Quien::Persona`: en rojo `bot_nunca_asignado_y_sin_tty_es_agente`.
- C7: TTL doblado: en rojo `cache_vigencia_y_rechazos`. Llamar a la plataforma sin TTY: en rojo `bot_nunca_asignado_...` (cuenta las llamadas).
- C1: no guardar la cache: en rojo `identidad_con_terminal_guarda_la_cache_...`.
- C9: no convertir por correo: en rojo `asignado_a_de_texto_de_la_0_16_es_heredado`.

Ejecucion a mano de `tc2 identidad` con el binario compilado y un mock en 127.0.0.1 (script propio de Python, fuera del repo), en un clon con `origin/develop` que lleva `config.yml` y `personas.yml`:
- Con una consola real (TTY, lanzado por `cmd` en ventana oculta; `winpty` no sirve, dice que stdin no es TTY): `identidad ok rol=persona id=1001 login=carlos`, `--json` con las mismas claves, y se crea `.git/tc2/identidad.yml` con `verificado`.
- Sin TTY, en es y en en (con `LANG=en_US.UTF-8`; `TC2_LANG` no cambia el idioma): `identidad rechazada motivo=identidad rol=persona` mas el diagnostico traducido, codigo 3.
- Con argumentos de mas: `identidad uso motivo=argumentos-de-mas`, codigo 2.
- Sin red (mock parado, TTY): `identidad sinred` mas diagnostico en es y en; el hash de la cache no cambia.
- Con `origin/develop` atrasado: ver IMP-1.
- Observacion fuera del diff: el diagnostico de `sinred` incluye el texto del error del sistema, que en esta maquina sale con la codificacion rota (`conexiÃ³n`). Viene de la capa de plataforma preexistente; no se abre hallazgo aqui.

Nota del ADR F1-9 (`docs/adr/F1-9-...md`, lineas 467-474): es correcta y coherente con el codigo. `asignado_a` es `null` o `{id: <entero>, login: "<login>"}`, se compara por `id`, el `login` es copia que refresca `personas.yml`, y el texto de la 0.16 es `Heredado`. La frase «no reabre este ADR» es razonable: el ADR ya dejaba el valor a F2-13 (linea 237 y 464). Una precision que no es hallazgo: dice que la 2.0 «nunca crea» un heredado pero que lo conserva en formato 2; esta bien explicado y es lo que hace el codigo.

Desviaciones del implementador:
- Claves `cuenta=` y `cache=` de mas en los rechazos: razonables. F1-4 admite pares `clave=valor` tras `motivo`, el contrato (`comun::contrato`) las acepta y distinguen el motivo del rechazo para quien parsea la linea. Aceptable.
- `personas.yml` leido de `origin/develop`, con `motivo=sin-personas`: decision coherente con F1-6 (cambiarla exige PR aprobado) y con que el agente no pueda editarla en su worktree. Su coste es IMP-1 (lista atrasada sin aviso). Sin esa salvedad, aceptable.
- `Heredado` reescrito tal cual: aceptable y necesario para no perder el dato al reescribir una tarea; esta probado byte a byte. Esta documentado en el ADR.
- `claves_ssh` y `claves_gpg` anadidas: son alcance extra respecto al enunciado de F2-13, pero vienen de mover `Personas` desde `plataforma/mod.rs` (que ya tenia `claves`), y F1-13 las exige para la firma de GitLab. Sin ellas se perderia la lectura de esas claves al mover el tipo. Aceptable. Tienen un test (`correos_y_claves`); no se valida que una misma clave este en dos personas, pero hoy es un conjunto y no tiene efecto observable.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/identidad.rs`, `plataforma/personas.rs`, `tarea/modelo.rs`, `tests/identidad.rs`, `tests/tarea.rs`, `lib.rs`, `git/estado.rs` y el catalogo `i18n/catalogo/identidad.rs`: es mi parte del diff, necesario para reproducir y mutar.
- `docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md` y `F1-6-identidades-agente-revisor-persona.md` (solo `grep`): para comprobar que las claves de mas en los rechazos y el formato de `identidad ok` / `sinred` encajan en el contrato, y que la lectura de `personas.yml` de la base es la de F1-6.

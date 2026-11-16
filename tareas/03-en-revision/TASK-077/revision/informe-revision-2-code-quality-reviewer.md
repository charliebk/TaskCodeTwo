# Informe de revision — TASK-077 (ronda 2)

- Commit revisado: 2fae3e4b7d864f66f5668dac349b21c345ce3f9a (HEAD de la rama: 8ea5cb4, que solo anade la peticion; el codigo es identico a 2fae3e4)
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/identidad.rs:470 | implementacion |
| MEN-1 | MENOR | corregido | crates/tc2/src/plataforma/personas.rs:288 | implementacion |
| IMP-2 | IMPORTANTE | abierto | crates/tc2/src/i18n/catalogo/identidad.rs:58 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/git/remoto.rs:177 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### Verificacion de las correcciones de la ronda 1

**IMP-1 — corregido.** Con terminal, `orden_paso` llama a `git::remoto::traer_rama` (`git fetch -q --end-of-options origin develop`) antes de leer `personas.yml`; si el fetch falla, `sinred` y la cache sin tocar. Sin terminal, `orden_paso` rechaza antes de cualquier fetch (`NECESITA_TERMINAL`), y `quien_ejecuta` sin terminal no trae nada: el rechazo por cache (`cache=fuera-de-lista`) lleva la pista `tc2 identidad` en es y en en.
- Test `persona_recien_anadida_se_identifica_desde_otro_clon_sin_fetch_previo`: en verde con el codigo del commit revisado.
- Mutacion: quitar la llamada a `traer_rama` (dejando `let _ = rama_base;`) deja en rojo `persona_recien_anadida_se_identifica_desde_otro_clon_sin_fetch_previo` (linea 495: la identidad no sale `ok`) y tambien `sin_red_no_toca_la_cache_y_sin_cache_se_pide_tc2_identidad` (linea 356). Restaurado; el clon quedo sin cambios.
- Limite conocido, no hallazgo: el fetch usa el refspec de `origin`. Con el refspec por defecto (el de los clones normales) actualiza `origin/develop`; no se ha comprobado con clones de refspec estrecho.

**MEN-1 — corregido.** El test de correo repetido (`personas.rs:288-291`) aserta ahora `1 (a, id 7)`, `2 (b, id 9)` y `X@Y`, como los de id y login. Verificado por lectura y en verde; no se ha mutado el mensaje.

**Fixtures (`origin` a `ssh://git@git.ejemplo.test/...` con `GIT_SSH_COMMAND`).** No debilitan los tests anteriores:
- Como `tc2 identidad` con terminal ahora hace fetch, los tests con terminal usan `git_con_ssh`, que sirve el remoto bare de la fixture; el fetch funciona y la prueba sigue llegando a la API. La fixture no quita ninguna aserción existente.
- `sin_red_no_toca_la_cache_y_sin_cache_se_pide_tc2_identidad` mantiene su primera parte intacta (la API cerrada da `identidad sinred`, cache sin tocar, `cache=falta` y `cache=caducada`) y anade una segunda parte con el fetch caido (`identidad sinred remoto=origin`, cache intacta, cero peticiones a la API).
- Ningun test se ha convertido en uno que pase por un motivo distinto: los 5 de `tests/identidad.rs` y las pruebas `--lib` pasan por las aserciones que enunciaban.

### IMP-2 — El mensaje de lista con terminal manda a repetir el comando que se acaba de ejecutar

- Donde: crates/tc2/src/i18n/catalogo/identidad.rs:58-59 (`CUENTA_FUERA_DE_LISTA`), usado desde `NoAsignable::FueraDeLista` en crates/tc2/src/identidad.rs.
- Que pasa: con terminal, `tc2 identidad` ya ha traido `origin/develop` antes de leer la lista. Si la cuenta sigue fuera, el diagnostico dice «si ya la anadieron, ejecuta `tc2 identidad` (actualiza la lista)», es decir, el mismo comando que acaba de correr, con la lista ya actualizada.
- Reproduccion: test temporal (no commiteado, borrado) con `Servidor` que devuelve `otra`/4242 y `lanzar(&f, &clon, true)` sobre un clon con `personas.yml` sin esa cuenta. Salida:
  ```
  LINEA=identidad rechazada motivo=identidad rol=persona cuenta=fuera-de-lista
  ES="la cuenta otra (id 4242) no esta en .taskcode/personas.yml: pide que la anadan con un PR, o usa el token de una persona de la lista; si ya la anadieron, ejecuta `tc2 identidad` (actualiza la lista)"
  ```
- Impacto: la persona que ve el rechazo con terminal recibe una instruccion que no cambia nada (repetir el mismo comando). Es el mismo tipo de mensaje que IMP-1: manda en la direccion equivocada. La frase es correcta solo en la ruta sin terminal, donde `tc2 identidad` no trae nada. Introducido por la correccion de la ronda 1 (el texto es nuevo).
- Arreglo sugerido (pequeno): que `CUENTA_FUERA_DE_LISTA` solo lleve la frase de «ya la anadieron» cuando la ruta no ha traido la base (por ejemplo, un mensaje distinto para el camino con terminal, o quitar la frase en esta ruta), y un test que fije el texto en ambos idiomas con terminal.

### MEN-2 — Rama base inexistente en origin se diagnostica como «sin red»

- Donde: crates/tc2/src/git/remoto.rs:177 (`fallo_fetch`: `FalloRemoto::RamaInexistente => sinred(op)`), llamado desde `traer_rama` en `orden_paso`.
- Que pasa: si `origin` no tiene `develop`, el fetch falla con `couldn't find remote ref` (RamaInexistente) y `tc2 identidad` sale como sinred.
- Reproduccion: test temporal con la fixture estandar y `update-ref -d refs/heads/develop` en el remoto bare; `lanzar(&f, &clon, true)` da `identidad sinred remoto=origin`, con `ES="identidad: no se pudo leer origin; reintenta con red"`. Verificado.
- Impacto: el diagnostico manda a revisar la red cuando el problema es la rama base. Antes de la correccion, la ruta de `traer` no llegaba aqui por este caso. Es un caso raro (develop borrado o nunca subido), por eso MENOR. Sugerencia: para `RamaInexistente` en `traer_rama`, devolver un diagnostico propio (la base no existe en `origin`) en vez de `sinred`.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/plataforma/github.rs`, `crates/tc2/src/plataforma/gitlab.rs` y `crates/tc2/src/plataforma/mod.rs` (solo `grep` de `tipo`): para comprobar que el tipo de cuenta que llega a `asignable` es `User` o `Bot` en GitLab y que `plataforma/mod.rs:79` no contradice el nuevo `NoPersona`.
- `crates/tc2/src/git/remoto.rs` (`clasificar_fallo_remoto`, `sinred`): para ver que `RamaInexistente` se clasifica como sinred (MEN-2).
- Las pruebas temporales de IMP-2 y MEN-2 se hicieron en el clon de pruebas, con un `tests/zz_scratch.rs` copia de `identidad.rs` que se borro tras ejecutarlo.

## Puerta y suite (commit revisado, 2fae3e4)

- `cargo test -p tc2` (suite completa del crate, todos los targets): 83 unitarios en verde, `identidad` 5/5, `tarea` 7/7 (1 ignorado, ya existia), `plataforma` 4/4, `plataforma_gitlab` 3/3, y el resto de integracion en verde. Sin fallos. La ejecucion no tuvo ningun cuelgue de 127.0.0.1.
- `cargo fmt --check`: limpio. No se ejecuto clippy en esta ronda (no estaba en la peticion).
- Repetida `tests/identidad` en paralelo: 5/5 en verde.

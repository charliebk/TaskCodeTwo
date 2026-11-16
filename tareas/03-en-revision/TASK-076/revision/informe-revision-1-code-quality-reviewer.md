# Informe de revision — TASK-076 (ronda 1)

- Commit revisado: ebc7fa5647f8386bf0266e16eab63a8a34d7e8a8 (HEAD de la rama es 710bc92, que solo anade la peticion)
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

Propuesta del revisor: **aprobada-con-correcciones** (sin CRITICO ni IMPORTANTE; 3 MENOR; el primero pide una adenda de documentacion antes del cierre).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:425 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/tarea/texto.rs:16 | plan |
| MEN-3 | MENOR | abierto | crates/tc2/src/i18n/catalogo/estado.rs:72 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

## Verificacion empirica

Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-076-r1-calidad`, rama de la tarea. `cargo test --workspace` (una sola vez, 4 min 14 s): **todo en verde**, 0 fallos (72 unitarios de `tc2`, `tests/alta.rs` con sus 9 tests, y el resto de suites). No cayo ningun test por los cuelgues de 127.0.0.1, asi que no hubo que repetir nada.

### Criterio a criterio (el test existe, prueba lo dicho, pasa; mutacion en rojo)

| Criterio | Test | Mutacion aplicada | Resultado |
|---|---|---|---|
| C1 | `alta.rs::mismo_titulo_a_la_vez_desde_dos_clones` (dos clones con barrera; IDs 001/002, ramas distintas en lo publicado, la segunda con `parecidas=TASK-001`) | `aplicar` deja `parecidas` vacio | rojo (tests/alta.rs:442) |
| C2 | `rama_solo_en_el_remoto_ocupa_su_id` + unitario `siguiente_id_es_el_maximo_mas_uno` | quitar `.chain(ocupados...)` de `siguiente_id` | rojo los dos |
| C3 | `rama_solo_en_local_ocupa_su_id` (con y sin remoto) | `ids_con_rama` ignora las ramas locales | rojo |
| C4 | `titulo_duplicado_de_una_abierta_avisa_y_de_una_terminada_no` (codigo 0, `(duplicado)`, las terminadas no avisan) | `ABIERTOS` incluye `terminada` | rojo (tests/alta.rs:367) |
| C5 | `texto.rs::umbral_de_casi_igual` (par `Exportar/Exprotar` = exactamente 0,80; `Login con Google/GitHub` = 0,61) | `>=` pasa a `>`; umbral 80 pasa a 85 | rojo las dos (texto.rs:261) |
| C6 | `ls_remote_previo_sin_red_o_denegado_no_deja_nada` (`sinred`, sin `fetch` ni `push` en el registro, refs iguales; remoto inexistente = 8) | `ids_con_rama` ignora el fallo del remoto | rojo |
| C7 | `rama_derivada_del_id_pasa_rama_valida`, `slug_ascii_cortado_en_guion_y_vacio_es_tarea` | slug vacio `tarea` pasa a `t`; `rama = None` | rojo (2 tests cada una) |

El borde del umbral (0,80 exacto) queda fijado por el par de ejemplo de C5, y la red de regresion discrimina. `id_de_rama` tambien tiene tabla (`task-7-x` sin tipo no cuenta, `a/b/task-1-x` si, desbordes de u32).

### `tc2 new` a mano (repo con remoto bare local, binario del clon)

| Caso | Salida observada |
|---|---|
| Titulo duplicado (`exportar  INFORMES!` tras `Exportar informes`) | `new ok id=TASK-002 rama=feature/task-002-exportar-informes parecidas=TASK-001 ...`; stderr `aviso: ya hay tareas abiertas con un titulo igual o parecido: TASK-001 "Exportar informes" (duplicado)`; codigo 0 |
| Titulo parecido (`Exprotar informes`) | `parecidas=TASK-001,TASK-002`, `(parecido 0,80)`, codigo 0 |
| `Login con Google` | sin `parecidas`, stderr vacio |
| `Задача без латиницы` y `漢字漢字` | `rama=feature/task-005-tarea` y `feature/task-006-tarea` (slug `tarea`) |
| Rama local a mano `feature/task-050-x` | el siguiente `new` da TASK-051 |
| Rama solo en remoto `feature/task-070-remota` | el siguiente `new` da TASK-071 |
| `feature/task-500` (sin slug) y `task-900-raiz` (sin tipo) | el siguiente da TASK-501: la primera cuenta, la segunda no |
| Ingles (`TC2_IDIOMA=en`) | `warning: there are already open tasks with the same or a similar title: ... (duplicate); ... (similarity 0.80)`, con punto decimal |
| `--json` | `"parecidas":["TASK-001",...]` en el JSON y el aviso en texto plano por stderr |
| Remoto caido (`127.0.0.1:1`) | `new sinred operacion=new pendiente=no`, codigo 4, diagnostico en es y en en |
| Remoto inexistente | `new denegada ... motivo=remoto-no-existe`, codigo 8 |

Comprobado con Git 2.56.0.windows.4 lo que afirma la adenda: `ls-remote --heads origin '*/task-*'` devuelve tambien `refs/heads/task-7-x` y `refs/heads/a/b/task-1-x`, y `id_de_rama` filtra `task-7-x` como dice.

### Adenda al ADR F1-2 punto 3

Correcta y completa en lo que cubre: nuevo texto del punto, que cuenta como rama de tarea, sin remoto (RN-06), fallos del remoto con sus codigos (4 y 8, verificados), rama escrita en `tarea.md`, riesgo aceptado (la carrera entre `ls-remote` y push, que cubrira `comprobar-id` de F3-6) y coste (un viaje por `new`). La evidencia citada existe y se pone en rojo por mutacion. Sin objeciones.

### Desviacion frente al ADR F1-4 punto 4

El ADR F1-4 §4 dice «Diagnostico solo en stderr y solo si falla». El alta escribe un aviso por stderr con codigo 0 (`crates/tc2/src/lib.rs:91`, con el comentario «unica excepcion ... (F2-4b, D1)»). Valoracion: la desviacion es razonable y esta bien acotada (la informacion de contrato va en stdout como `parecidas=` y en `--json`; el stderr es solo para la persona; no cambia el codigo de salida ni el maximo de 5 lineas de stdout; la decision D1 es de Carlos). Pero queda sin adenda en F1-4, y el test C4 aserta `err.is_empty()` en el caso sin parecidas, de modo que quien lea el ADR sigue viendo la regla vieja. Se recoge como MEN-1.

## Detalle de los hallazgos

### MEN-1 — la excepcion «aviso por stderr con codigo 0» no tiene adenda en F1-4
`docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:425` sigue diciendo «Diagnostico solo en stderr y solo si falla». El codigo ya lo contradice en un caso (`crates/tc2/src/lib.rs:91`) y la regla solo esta justificada en un comentario. Reproduccion: `tc2 new "Exportar informes"` dos veces; la segunda sale con codigo 0 y una linea `aviso:` por stderr. Con `--json`, stdout es JSON y stderr texto plano, algo que un consumidor que trate stderr como «fallo» no espera. Correccion propuesta: una adenda corta en F1-4 (como la de F1-2) que fije «avisos informativos: una linea por stderr, codigo 0, y la clave `parecidas=` en stdout/JSON como canal fiable».

### MEN-2 — el umbral 0,80 da falsos positivos con titulos numerados y el aviso no tiene tope
Reproduccion: `new "Fase 1 de migracion"` y luego `new "Fase 2 de migracion"` avisa `(parecido 0,86)`; `Задача 5` / `Задача 6` avisa 0,83. En un plan con titulos tipo `F2-4 ...` / `F2-4b ...` pasara a menudo. Ademas la lista del aviso crece sin limite (con 5 parecidas ya van las 5 en una linea). Esta aceptado en el plan (D2: constante hasta que F9-4 mida el ruido), por eso es MENOR y de origen `plan`; queda anotado para que F9-4 lo mida y se plantee un tope (p. ej. las 3 mas parecidas) y no contar digitos sueltos.

### MEN-3 — el diagnostico de `denegada` de una lectura dice «the remote denies the write»
`crates/tc2/src/i18n/catalogo/estado.rs:72` (`ESCRITURA_DENEGADA`). Reproduccion: `origin` apuntando a una ruta inexistente y `tc2 new "x"` da `new denegada ... motivo=remoto-no-existe` con el diagnostico «...does not appear to be a git repository; the remote denies the write: check credentials, permissions or rules...». El fallo es del `ls-remote` previo, una lectura, y no se ha intentado ninguna escritura. Es un matiz de redaccion (el consejo sigue valiendo) y viene del catalogo compartido con otras operaciones; no afecta a codigo ni a contrato.

## Lo revisado sin hallazgos

- `leer_varios` / `leer_blobs` / `partir_batch`: orden preservado, rutas con salto de linea van aparte, las superpuestas (`encima`) y las ausentes no lanzan nada, un objeto por ruta o error; la entrada se escribe en otro hilo (sin interbloqueo); `ejecutar_con_entrada` solo admite `cat-file` y lo prueba un test.
- `siguiente_id`: desborde de `u32` con `checked_add`, probado; ramas con numero que no cabe en `u32` se ignoran.
- Alta con reintentos: `parecidas` e `id` se recalculan sobre el estado fresco de cada intento; el aviso solo sale con codigo 0.
- Tipo y rama salen de la misma constante `TIPO` (correccion de la autorrevision verificada).
- Sin `unwrap`, `expect` ni `panic` nuevos en produccion.

## Lecturas fuera del contexto permitido

- `docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md` (§4 y adenda F2-3): el encargo pedia valorar la desviacion del aviso por stderr.
- Codigo fuente del clon (`crates/tc2/src/**`, `crates/tc2/tests/alta.rs`): necesario para las mutaciones y la reproduccion; es el diff de la tarea.

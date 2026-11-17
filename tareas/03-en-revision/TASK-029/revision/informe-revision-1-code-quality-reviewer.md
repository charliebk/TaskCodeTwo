# Informe de revision — TASK-029 (ronda 1)

- Commit revisado: 80f72ac09fe9e660606b55f88dc82c61307d5b56
- Revisor: code-quality-reviewer
- Veredicto: PENDIENTE (escribelo con: taskctl veredicto TASK-029 aprobada | aprobada-con-correcciones | cambios-solicitados)

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/importar/mod.rs:109 | plan |
| MEN-2 | MENOR | abierto | crates/tc2/src/importar/mod.rs:263 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/src/importar/mod.rs:115 | implementacion |
| MEN-4 | MENOR | abierto | crates/tc2/src/importar/mod.rs:121 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Sin CRITICO ni IMPORTANTE. Los cuatro MENOR son casos de borde o decisiones sin
registrar que no cambian lo que se escribe ni lo que se publica en un uso normal.

## Alcance y puerta

- Clon limpio en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-029-r1-calidad`, rama de la tarea (HEAD `b96b694`, que solo anade la peticion; el codigo es el de `80f72ac`). La peticion no traia verificacion de la ronda: corri yo la puerta.
- `cargo build` ok, `cargo fmt --check` ok, `cargo clippy --all-targets` sin avisos, `cargo test --no-fail-fast` entero: todos los binarios `test result: ok`, 0 fallos.
- Pase generico completo: el diff toca solo Rust y un ADR, asi que no llego por exceso de dominios; la parte Rust la cubre ademas la revisora de rust-wasm. Profundidad: `importar/fichero.rs`, `importar/mod.rs` y `alta.rs` a fondo; `lib.rs`, `tarea/*` y los catalogos de i18n por encima.

## Criterios (el test existe, prueba lo que dice y pasa)

| Criterio | Test | Resultado |
|---|---|---|
| C1 | `importar.rs::c1_c3_importa_37_en_un_commit_y_repetir_no_crea_nada` | verde; comprueba 1 commit, 37 carpetas consecutivas, sprint, objetivo, criterios y rama. Mutacion: quitar `e.sprint = sprint` -> rojo |
| C2 | `importar.rs::c2_dependencias_externas_validadas_y_errores_sin_commit` (+ resolucion local en C1) | verde; codigo 2 y refs del remoto sin cambios en inexistente, local inexistente y ciclo. Mutaciones: no apuntar la dependencia resuelta -> rojo; desactivar la deteccion de ciclos -> rojo |
| C3 | mismo test que C1 | verde; `creadas=0 existentes=37`, codigo 0, sin commit nuevo. Mutacion: `partition(\|_\| false)` (nada cuenta como existente) -> rojo |
| C4 | `importar.rs::c4_dos_entradas_mas_crean_solo_esas_dos` | verde, prueba lo que dice (sin mutar, no pedido) |
| C5 | `c5_entrada_invalida_en_la_30_no_escribe_nada` y `c5_dos_clones_importando_a_la_vez` | verde. Mutaciones: quitar la regla `SinCriterios` -> rojo; ids con hueco (`i*2`) -> rojo en el de dos clones |
| C6 | `importar.rs::c6_simular_no_cambia_ni_un_byte` | verde; compara instantanea del clon y del remoto. Mutacion: desactivar la rama `--simular` -> rojo |
| C7 | `importar.rs::c7_limites_y_sin_red` | verde. A mano: fichero de 1,1 MB da `uso motivo=fichero-grande limite=1048576` |
| C8 | `importar.rs::c8_fichero_016_de_la_ampliacion_se_importa_sin_tocarlo` | verde; lee el bloque de §5 del propio documento, no una copia |
| C9 | `alta.rs::c9_new_con_los_campos_de_rf_01_y_rechazo_de_un_enunciado_no_listo` | verde. Mutacion: quitar `SinCriterios` -> rojo |

Nota de metodo: tras las mutaciones el binario `target/debug/tc2.exe` quedo compilado con la ultima mutacion; lo detecte (un import manual dio un hueco de IDs), recompile y repeti. Los resultados manuales de abajo son sobre el binario sano.

## Prueba a mano (remoto bare propio, es y en)

- `import --simular` sin rama de estado: `import ok simular=si creadas=2 existentes=0 ids=TASK-001,TASK-002`, sin `base=`; con estado: `base=<sha>`. Nada escrito en local ni en remoto.
- `import` real: `creadas=2 ... commit=`; repetido: `sin-cambios creadas=0 existentes=2`, codigo 0, sin commit.
- Errores, con diagnostico en `es` y `en`: ciclo, dependencia local inexistente, `TASK-999` inexistente, entrada sin criterios, solo titulo, `lote: x/y`, clave duplicada por slug: todos `uso` (2), «No se ha importado nada» / «Nothing was imported», rama de estado sin cambiar.
- Remoto inexistente: `denegada motivo=remoto-no-existe`, rc 8, tambien con `--simular`.
- `new` con `--tipo fix --objetivo --criterio x2 --etiqueta --complejidad --sprint --dependencia`: rama `fix/task-004-...`. Rechazos con `causa=`: `sin-criterios`, `plantilla`, `criterio-repetido`. `new --json` ok.
- Un lote de 3 con el binario sano: `TASK-012,TASK-013,TASK-014`, sin huecos.

## Valoracion de las decisiones del implementador

- **«Enunciado no listo»**: razonable. Solo titulo es borrador valido en `new` (como hasta F2-4) y no en `import` (como la 0.16); con algo escrito exige criterio, sin vacios, repetidos ni huecos `<...>` ni control. Acepta objetivo vacio con criterios (discutible, coherente con la 0.16). Sin hallazgo.
- **`--simular` sin fetch y con `base=`**: correcto y honesto: dice sobre que commit calcula. Matiz: sigue haciendo `ls-remote` (solo lectura) para las ramas ocupadas, asi que sin red `--simular` falla con `denegada`/`sinred`. No cambia ni un byte, pero «sin fetch» no es «sin red». Sin hallazgo; conviene decirlo en la ayuda.
- **Dependencias a tareas descartadas**: aceptadas sin aviso ni test. Es la pregunta que el plan dejo abierta y nadie cerro. Ver MEN-1.
- **Lote invalido rechazado entero**: correcto (F1-2, todo o nada) y mejor que la 0.16, que saltaba entradas; cubierto por C5 con mutacion.
- **Lote sin cabecera = slug del nombre del fichero**: funciona y es determinista, pero la identidad depende del nombre del fichero. Ver MEN-3.
- **Titulo que conserva el `###` completo (clave incluida)**: coherente con la 0.16 y con C8; el coste es una rama y un titulo largos (`feature/task-001-h1-tarea-hito37-numero-1`). Decision, no fallo.

### MEN-1 — Dependencia a una tarea descartada aceptada en silencio
- Donde: crates/tc2/src/importar/mod.rs:109 (y `alta.rs`, `Alta::aplicar`, la misma regla en `new --dependencia`)
- Que pasa: la dependencia externa solo comprueba que exista `TASK-NNN/tarea.md`; el estado no cuenta. El plan dejo abierta la regla de dominio y no hay decision registrada ni test.
- Reproduccion: se descarto a mano TASK-002 (`estado: "descartada"` con `motivo`); `import` de `### X1 Nueva ... Dependencias: TASK-002` dio `import ok creadas=1`, y `new ... --dependencia TASK-002` dio `new ok`.
- Impacto: una tarea nueva nace bloqueada por una que nunca terminara; F3-3 (`next`) tendra que decidirlo. MENOR: no se pierde ni corrompe nada. Resolver: decidirlo y dejarlo escrito (plan de F3-3 o un aviso).

### MEN-2 — El error de dependencia externa inexistente no lleva `referencia=`
- Donde: crates/tc2/src/importar/mod.rs:263 (`Fallo::rechazada` y `referencia_a_uso`)
- Que pasa: la local inexistente da `uso motivo=referencia-inexistente linea=1 referencia=Z9`; la externa da `uso operacion=import motivo=referencia-inexistente`, sin `referencia=` ni `linea=` (el texto solo va en el diagnostico).
- Reproduccion: `Dependencias: TASK-999` frente a `Dependencias: Z9`, mismo `motivo=`.
- Impacto: quien lea solo la linea de contrato no sabe cual falla. Inconsistencia de claves, sin efecto en lo escrito.

### MEN-3 — La identidad del lote sin cabecera es el nombre del fichero (colisiones y renombrados)
- Donde: crates/tc2/src/importar/mod.rs:115
- Que pasa: sin `lote:` el nombre es `slug(nombre del fichero)`. Nombres distintos con el mismo slug (`hito-37.md` y `hito_37.md`; un nombre sin letras ASCII da `tarea`) comparten lote, y la segunda importacion trataria como `existentes` entradas con la misma clave aunque su contenido sea otro. Y renombrar el fichero duplica todo.
- Reproduccion: `lote.md` importado y copiado a `lote-copia.md` crea 2 tareas nuevas, con el aviso `parecidas=TASK-001 (duplicado)`. `¡¡.md` dio `lote: "tarea/A1"`.
- Impacto: el aviso de parecidas lo mitiga y la adenda de F1-9 ya dice que sin cabecera se usa el nombre del fichero; no dice que renombrar duplica ni que el slug puede colisionar. Falta en el ADR y en la ayuda: recomendar siempre `lote:` en la cabecera.

### MEN-4 — Sin `sprint` en cabecera ni `--sprint`, el hito queda en 0 sin avisar
- Donde: crates/tc2/src/importar/mod.rs:121
- Que pasa: `(c, f) => c.or(f).unwrap_or(0)`: un fichero sin cabecera importado sin `--sprint` crea tareas con `sprint: 0`.
- Reproduccion: `import lote.md` (sin cabecera): `TASK-002/tarea.md` con `sprint: 0`.
- Impacto: el hito (RF-02) se pierde en silencio y, como un import no sincroniza, repetirlo no lo corrige. Coherente con `new` (sprint 0 por defecto) y con la 0.16, por eso MENOR; un aviso o `sprint=` en la salida lo haria visible.

## Adenda de F1-9

- Comprobado: 21 claves del punto 3 en la plantilla; `lote` es la unica opcional (`OPCIONALES`), se escribe despues de `dependencias` y antes de las desconocidas, y solo con valor; el esquema (texto sin control con una `/` entre dos partes no vacias) es el de `modelo.rs::lote`; los tests citados existen y pasan (`plantilla_escapa_el_titulo_y_sigue_f1_9` fija el orden con y sin `lote`; `ida_y_vuelta_byte_a_byte` y `tests/tarea.rs` no estan en el diff, asi que la dorada de F2-5 no cambio); el ancla `#adenda-f3-2-task-029--clave-opcional-lote` coincide con el titulo.
- Matiz sin hallazgo: una tarea con `lote: null` explicito se lee bien pero se reescribe sin la clave; la adenda dice «si falta», asi que no la contradice.
- Falta (ver MEN-3): una linea sobre renombrar el fichero sin cabecera.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/importar/fichero.rs`, `importar/mod.rs`, `alta.rs`, `tarea/modelo.rs`, `tests/importar.rs` y `tests/alta.rs`: para comprobar los criterios y las mutaciones (mi parte del diff era solo el ADR).
- `docs/plan/REQUISITOS-FUNCIONALES.md`: texto de RF-01..RF-03 y RF-07 para valorar las decisiones.
- `task-workflow/revision.md` del plugin: plantilla del informe.

# Informe de revision — TASK-023 (ronda 1)

- Commit revisado: 25e7d2ce796319aac3b8ab5b4d26518c6c197f78
- Revisor: agente general-purpose (sonnet) con la skill rust-wasm-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/config/validar.rs:115 | implementacion |
| MEN-1 | MENOR | corregido | crates/tc2/src/config/validar.rs:307 | implementacion |
| MEN-2 | MENOR | corregido | crates/tc2/src/config/validar.rs:219 | implementacion |
| MEN-3 | MENOR | corregido | crates/tc2/src/config/validar.rs:231 | implementacion |
| MEN-4 | MENOR | corregido | crates/tc2/src/config/validar.rs:243 | implementacion |
| MEN-5 | MENOR | aceptado | crates/tc2/src/config/claves.rs:504 | plan |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### IMP-1 — La prohibicion de `escribe` en un derivado `comprueba` no tiene test

`validar.rs:115` (`Some("comprueba") if escribe => ...`) es la regla cruzada
«prohibida en comprueba» (claves.rs la documenta en `derivados[].escribe`).
Mutacion: sustituir la guarda por `if false`. Resultado: `cargo test -p tc2
--test config --lib` sigue en verde (mutante SUPERVIVE). Ningun caso de
`configs_hostiles_rechazadas_con_la_ruta` ni de `config.rs` ejerce
`tipo: comprueba` con `escribe`; el simetrico (`genera` sin `escribe`) si esta
cubierto (mutante M6 muerto). Un `comprueba` con `escribe: [...]` pasaria
como valido y llegaria a `Derivado.escribe`. Arreglo: anadir el caso a la tabla
de hostiles (ruta `derivados[0].escribe`, fragmento «prohibida»).

### MEN-1 — `escribe` no ve los alias de Windows (punto o espacio final, 8.3)

`ruta_tareas` compara segmentos exactos (`.git` en `validar.rs:307`,
`ROADMAP.md`, `TASK-NNN`). Caso construido con `config::leer`, todos aceptados:
`tareas/ROADMAP.md.`, `tareas/ROADMAP.md ` (espacio final), `tareas/.git./x`,
`tareas/.git /x`, `tareas/TASK-001./x`, `tareas/aux /x`, `tareas/ROADMA~1.MD`,
`tareas/GIT~1/x`. En Windows, que es la plataforma principal del proyecto, el
punto y el espacio finales se descartan y esas rutas caen sobre `ROADMAP.md`,
`.git` o la carpeta de tarea que el plan declara reservados «sin distinguir
mayusculas». Solo `tareas/CON./x` se rechaza (por `reservado_en_windows`, que
parte por `.`). Fallar cerrado en el validador no depende de que el
comparador posterior normalice igual. Arreglo: rechazar segmentos que acaben en
`.` o espacio y los que contengan `~`.

### MEN-2 — `comando: |` se rechaza con un mensaje que apunta a otra causa

Una orden en bloque literal YAML (`comando: |` + linea) produce `"cargo
test\n"`; `comprobar(Regla::Orden)` lo rechaza por caracter de control con el
texto «¿un \v de una ruta de Windows entre comillas dobles?». Reproducido en
`verificacion.comando`. Es fallar cerrado (correcto) pero el diagnostico induce
a error: `|-` o una linea simple funcionan. Arreglo: mensaje especifico si el
unico control es un `\n` final o interior.

### MEN-3 — «nunca todo el arbol» en `confianza_incluye` es una lista de cadenas exactas

`validar.rs:231` solo rechaza `*`, `**`, `**/*`, `**/**`, `./**`, `./*`. Aceptados
(reproducido): `**/`, `**/**/*`, `*/**`, `./**/*`, `***`, `.`. Todos cubren el
arbol (o casi). Arreglo: decidir por la forma (todos los segmentos son
comodines o `.`) en vez de por lista.

### MEN-4 — `cron` y listas permisivos

- `validar.rs:243`: el cron solo exige 5 campos de `[0-9*,/-]`; `99 99 99 99 99` y
  `- , / - ,` son validos (reproducido).
- `derivados[].fases: [new, new]` y las demas listas cerradas aceptan
  duplicados (reproducido); `fases` duplicada llega a `Derivado.fases`.
- El esquema publicado es mas laxo que el validador en lo que no sabe expresar
  (espacios en `Nombre`, caracteres de control, cron, globs y las reglas
  cruzadas `escribe`/`id` repetido): un editor con el esquema acepta lo que
  `leer` rechaza. Esta documentado en `description`, no en el esquema.

### MEN-5 — Se aparta del plan: claves planas, 12 de 12 y `modo_flujo` por defecto `manual`

El plan (D1, inventario) agrupa `modo_flujo` y `tope_rondas` en la clave `flujo`
con `modo: automatico` por defecto, deja 11 de 12 claves y lo cita como riesgo
aceptado («queda una sola clave libre»). La implementacion deja `modo_flujo` y
`tope_rondas` en el primer nivel (`claves.rs:504`, `:511`), con `modo_flujo`
por defecto `manual`: 12 de 12, ninguna libre, y el valor por defecto cambia.
La razon es plausible (F1-11 y F1-12 los tratan como claves planas que tambien
lee la 0.16, y el plan D10 pedia comprobar el valor en la 0.16), pero la
desviacion no consta en `tarea.md` ni en el plan. El criterio «≤ 12 claves» se
cumple. Accion: dejar constancia (nota en `## Resultado`) de la decision y del
valor por defecto verificado.

## Alcance

- Ficheros revisados: crates/tc2/src/carril.rs, config/claves.rs, config/mod.rs, config/validar.rs, git/estado.rs (solo `reservado_en_windows` a `pub(crate)`), lib.rs, tarea/yaml.rs, yaml.rs, tests/config.rs.
- Contexto adicional pedido: ninguno fuera de la lista (ver «Lecturas fuera del contexto permitido»).
- Limite JS-WASM: no aplica. El diff no usa `wasm_bindgen` ni compila a `wasm32`; es el CLI nativo `tc2`. No hay `unsafe`, `unwrap` ni indexado `[]` sobre entrada en el codigo nuevo (los `unwrap_or*` llevan valor por defecto; el indice de `y_mayores` no puede salir de rango).

## Reproduccion

- Clon: `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-023-r1-rust`, rama `feature/task-023-f2-7-configuracion-minima-con-esquema-va` (HEAD 05eb4d3, que solo anade la peticion sobre 25e7d2c).
- Build: `cargo test --locked` compila en nativo; no se compila a `wasm32` (no aplica).
- Formato y lints: `cargo fmt --check` limpio; `cargo clippy -p tc2 --all-targets --locked -- -D warnings` sin avisos.
- Suite: `cargo test -p tc2 --test config --lib --locked`: lib 46 pasan, config 8 pasan, 0 fallos. No se corrieron los `#[ignore]` ni la suite del workspace entero (peticion acotada).
- Casos construidos (con `config::leer`, test temporal borrado): los de MEN-1, MEN-2, MEN-3, MEN-4; ademas rechazados correctamente: documento YAML multiple, entero > u64 (`tope_rondas: 18446744073709551616`), clave con `ESC` y `\n` (la linea de contrato y stderr salen saneadas: `claves=x%1B[31m%0Aconfig%20ok`).
- Mutacion (cada una sobre el commit revisado, suite de config y `lib` de config/carril/tarea):
  - MUERTOS: M1 `roles_min` riesgo-alto 4→3; M2 sin rechazo de `.git`; M3 sin nombres reservados de Windows; M4 glob sin rechazo de `..`; M5 `GlobAcotado` sin su lista; M6 `genera` sin `escribe`; M7 `recursos[].id` repetido; M8 claves duplicadas YAML ignoradas.
  - SUPERVIVE: M6b (`comprueba` con `escribe`), es IMP-1.
  - SUPERVIVE, equivalente: M8b (`max_aliases` 0→100): `max_anchors = 0` ya impide que exista un alias, no es hallazgo.
  - NO EJECUTADOS por tiempo (la maquina estaba cargada, unos 6 min por mutante): limites de `Entero`, `minItems`, carpeta `TASK-NNN`, `ROADMAP.md`, defectos en listas, normalizacion de `escribe`, `\r\n`, `formato`. Los casos de `configs_hostiles...` y `rutas_de_escribe` los ejercen por lectura, pero no consta mutacion.
- Medidas: no aplican (sin rendimiento ni `.wasm`).

## Revisado sin hallazgos

- `yaml.rs` y su uso desde `tarea/yaml.rs`: la extraccion es identica linea a linea y la suite de `tarea` sigue en verde.
- `carril.rs`: el suelo (`roles_min` 0/1/1/4 y perfil `completa`) coincide con la tabla de F1-10; `y_mayores` no indexa fuera de rango.
- `de_base`: lee `origin/develop` con `ls-tree` + `show`, probado contra commit local y cambio sin commitear; sin `origin/develop` da `sin-base` sin panico.
- Saneado de rutas y claves hostiles en la linea de contrato y en stderr (via `Informe`).
- Tipos de `Config`: los anchos (`u8`, `u16`, `u32`) estan dentro de los maximos que valida la tabla; `u64::MAX` en `lote` cabe.
- Esquema publicado: igual que el generado (test) y cerrado en cada objeto.

## Lecturas fuera del contexto permitido

- `docs/adr/F1-10-carriles-y-modo-automatico.md`, `F1-11-extensiones-declaradas.md`, `F1-12-actualizacion-entre-versiones.md` (solo `grep`): para contrastar el suelo de `carril.rs` y el estado de `modo_flujo` (MEN-5).
- `crates/tc2/src/contrato.rs` y `crates/tc2/src/git/estado.rs` (fragmentos): para comprobar el saneado de la linea de contrato y `leer_blob`/`reservado_en_windows`.

## Estado tras la ronda (orquestador)

Corregidos en 8671970 con test. MEN-5: aceptado: se sigue a F1-12 (decision 6, tabla de la config migrada) y F1-11, que nombran `modo_flujo` y `tope_rondas` planas y dicen que la 0.16 sigue leyendo `modo_flujo` en la convivencia; agruparlas rompe esa lectura. El defecto `manual` es el de la 0.16 (CONFIG_DEFAULTS de taskctl 0.16.1, verificado; D10 lo dejaba sin verificar). Quedan 12 de 12 claves (criterio <= 12 cumplido). Consta en el Resultado como divergencia con el plan.

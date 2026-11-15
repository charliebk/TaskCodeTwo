# Informe de revision — TASK-023 (ronda 1)

- Commit revisado: 25e7d2ce796319aac3b8ab5b4d26518c6c197f78
- Revisor: agente general-purpose (modelo sonnet) con la skill code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/config/validar.rs:276 | implementacion |
| MEN-1 | MENOR | corregido | crates/tc2/src/config/validar.rs:227 | implementacion |
| MEN-2 | MENOR | corregido | crates/tc2/src/config/mod.rs:385 | implementacion |
| MEN-3 | MENOR | aceptado | crates/tc2/src/config/claves.rs:504 | implementacion |
| MEN-4 | MENOR | corregido | crates/tc2/tests/config.rs:1 | implementacion |
| MEN-5 | MENOR | corregido | crates/tc2/src/config/validar.rs:243 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

## Entorno y puerta

- Clon limpio en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-023-r1-calidad` (rama de la tarea, HEAD 25e7d2c). No he tocado el repo original.
- `cargo test -p tc2 --test config --test tarea --lib`: 59 passed, 1 ignored, 0 failed (una sola ejecucion de la suite).
- Casos borde reales contra `config::leer` con un test temporal en el clon (no forma parte de la entrega): vacio, solo `---`, multidocumento, `...`, clave no string, `lote:` nulo, `formato` 3 y 2.0, `tope_rondas: 3.0`, entero desbordado, anidamiento de 5000 niveles, tabulaciones, clave con salto de linea, listas con duplicados, rutas de `escribe` con `.` o espacio finales y nombre 8.3, globs equivalentes a todo el arbol. El camino de errores y el contrato se comportan bien en todos salvo lo que sigue.
- Mutantes a mano: 22 intentados; 19 aplicados y ejecutados con `--test config --lib` (M7 no se aplico y M19 no compilo: no cuentan). Muertos (14): suelo de 4 roles en riesgo alto, `ROADMAP.md` sin distinguir mayusculas, `tareas/` entera, `.git`, `..` en globs, valor por defecto de `modo_flujo`, maximo de `tope_rondas`, `formato` distinto de 2, regla cruzada de `genera` sin `escribe`, id de `recursos` repetido, hint `tc2 actualizar`, `*` en `GlobAcotado`, recursion de valores por defecto, rama de `y_mayores`. **Supervivientes (5)**: ver MEN-4.

## Hallazgos

### IMP-1 — `derivados[].escribe`: la reserva de `ROADMAP.md`, `TASK-NNN/` y `.git` se salta con punto o espacio finales (Windows)

F1-11 (decision 1) pide rechazar `tareas/ROADMAP.md`, `tareas/TASK-NNN/` y `.git`, y `ruta_tareas` ya trata Windows (`reservado_en_windows`, rechaza `aux.`). Pero las tres reservas comparan el segmento tal cual, y Windows ignora los puntos y espacios finales de un nombre.

Reproduccion (test temporal sobre `config::leer`, `derivados: [{comando: x, tipo: genera, escribe: ['<ruta>']}]`):

- `tareas/ROADMAP.md.` -> aceptada (esperado: rechazada, es `ROADMAP.md` en Windows)
- `tareas/ROADMAP.md ` -> aceptada
- `tareas/TASK-001./tarea.md` y `tareas/TASK-001 /x` -> aceptadas
- `tareas/.git./x` y `tareas/.git /x` -> aceptadas
- `tareas/aux./x` -> rechazada (el unico caso cubierto)

Por que importa: `escribe` es la lista blanca que el `check estado` compara con lo que toca un `genera`. Un derivado que escriba `tareas/ROADMAP.md.` sobrescribe el ROADMAP o una tarea ajena sin que la configuracion lo declare. Los tests de `rutas_de_escribe` cubren las mayusculas, no esta variante. Arreglo propuesto: recortar `.` y espacios finales de cada segmento antes de comparar (o rechazar los segmentos que los llevan) y anadir los casos a `rutas_de_escribe`. Los nombres cortos 8.3 (`ROADMA~1.MD` pasa) se pueden aceptar y documentar.

### MEN-1 — `confianza_incluye`: la guarda "nunca todo el arbol" es una lista de cadenas literales

`Regla::GlobAcotado` solo rechaza `*`, `**`, `**/*`, `**/**`, `./**` y `./*` (`validar.rs:227-234`). Formas equivalentes pasan: `**/`, `./**/*`, `**/**/*`, `.`, `./`, `*/**` y, casi equivalentes, `*.*` y `**/*.*`. Reproducido con `config::leer`: las ocho dan `OK`. F1-11 (dec. 1) pide "nunca todo el arbol (`**`, `*`)" y la intencion solo se cumple para las grafias que alguien penso. Arreglo razonable: normalizar el glob (quitar `./`, colapsar `**/**`, quitar `/` final) antes de comparar, y ampliar `reglas_de_texto`.

### MEN-2 — El arreglo `tc2 actualizar` se decide buscando texto dentro del mensaje de error

`Invalida::informe` (`mod.rs:385`) elige "quedan claves de la 0.16" con `e.mensaje.contains("tc2 actualizar")`, y los mensajes incluyen el valor de la persona. Reproduccion: `modo_flujo: tc2 actualizar` da el diagnostico `... Arreglo: quedan claves de la 0.16: tc2 actualizar las traduce`, que manda en la direccion equivocada. Ademas `Motivo::Formato` siempre dice "es un proyecto de la 0.16: migralo con tc2 actualizar", tambien con `formato: 3` (formato futuro: lo que toca es actualizar `tc2`). Poco frecuente. Arreglo: un campo `hint_016: bool` en `ErrorConfig` que fija `no_admitida`, en vez de buscar en el texto, y tratar `formato` mayor que 2 aparte.

### MEN-3 — El codigo se aparta del plan aprobado en las claves de primer nivel y no queda dicho

El plan (inventario y D1) agrupa `modo_flujo` y `tope_rondas` en una clave `flujo` y deja 11 de 12, con "queda una sola libre" como riesgo aceptado. La implementacion usa `modo_flujo` y `tope_rondas` planas: son 12 de 12, sin margen, y el esquema publicado ya las fija. Hay motivo defendible (F1-11 las cita como claves base de F2-7 y la 0.16 lee `modo_flujo`), pero la desviacion no consta como decision. El valor por defecto de `modo_flujo` es `manual` donde D10 proponia `automatico` (D10 pedia comprobarlo en la 0.16; el resultado tampoco consta). No bloquea: conviene anotarlo en `## Resultado` y recordar que no queda ninguna clave libre.

### MEN-4 — Cinco mutantes sobreviven: protecciones sin test que se ponga rojo

Aplicados en el clon, la suite siguio verde en todos:

- `normalizar_escribe` (`mod.rs:466`) eliminado: ningun test comprueba que `leer` guarda `escribe` normalizado (`./tareas//x/../hitos.md` a `tareas/hitos.md`). Hay test de `ruta_tareas`, no del cableado.
- `Some("comprueba") if escribe` desactivado: la mitad "prohibida en comprueba" de la regla cruzada no tiene caso hostil (la mitad `genera` si).
- `primero: Some("self-hosted")` desactivado: `runner: [windows]` se acepta sin que ningun test lo note.
- `TOPE_CLAVES` de 12 a 13: el test compara con la propia constante, subirla no lo rompe. Un `assert_eq!(config::TOPE_CLAVES, 12)` lo cubre.
- Cron con `>= 5` campos: solo se prueba un caso de 4.

Cada uno se arregla con una linea de test.

### MEN-5 — El cron admite valores fuera de rango

`Regla::Cron` (`validar.rs:243`) acepta cualquier cifra y `*,/-`: `99 99 99 99 99` es valida (reproducido). Basta si F1-11 solo exige 5 campos; se reporta porque el mensaje dice "cron no admitido" y puede dar a entender que se valido el rango. Aceptable si se documenta que lo valida quien lo consume.

## Lo que he comprobado y esta bien

- Contrato de salida: `config rechazada motivo=... errores=N claves=...` con rutas escapadas, codigo 3, diagnostico en stderr con el primer error y el arreglo.
- `leer` es pura. BOM, `\r\n`, `---` con espacio o comentario, multidocumento, `formato` como `2.0`, clave no string, nulo donde va un mapa, anidamiento enorme, enteros desbordados: rechazo con motivo y ruta, sin panico.
- `de_base` lee `origin/develop` y no el worktree (test de integracion con commit local y cambio sin commitear), y distingue `sin-config`, `sin-base` y fallo de git.
- El esquema publicado se compara byte a byte con el generado (`.gitattributes` con `eol=lf`) y su sincronia con la tabla de F1-11 se comprueba contra el ADR. Mover `opciones()` a `crate::yaml` no cambia el comportamiento: la suite de `tarea` sigue en verde.
- No hay duplicacion de decisiones: validador, valores por defecto y esquema salen de la misma tabla.

## Lecturas fuera del contexto permitido

Para contrastar con los ADR que pide el encargo: `docs/adr/F1-11-extensiones-declaradas.md` (lineas 265-292, 564-567, 810-818) y, por `grep` de `modo_flujo` y `tope_rondas`, `F1-10`, `F1-12` y `F1-16`. Ademas, el codigo completo de `config/`, `carril.rs`, `yaml.rs` y `tests/config.rs` del clon (es mi parte del diff).

## Estado tras la ronda (orquestador)

Corregidos en 8671970 con test. MEN-3: aceptado: se sigue a F1-12 (decision 6, tabla de la config migrada) y F1-11, que nombran `modo_flujo` y `tope_rondas` planas y dicen que la 0.16 sigue leyendo `modo_flujo` en la convivencia; agruparlas rompe esa lectura. El defecto `manual` es el de la 0.16 (CONFIG_DEFAULTS de taskctl 0.16.1, verificado; D10 lo dejaba sin verificar). Quedan 12 de 12 claves (criterio <= 12 cumplido). Consta en el Resultado como divergencia con el plan.

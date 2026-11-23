# Informe de revision — TASK-138 (ronda 1)

- Commit revisado: 14de27c9ff1a6be919beb9b56e9f3c2bdbc2f7af
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | docs/auditoria-4d.md:3733 | implementacion |
| MEN-2 | MENOR | abierto | docs/auditoria-4d.md:4738 | implementacion |
| MEN-3 | MENOR | abierto | tareas/03-en-revision/TASK-138/seguimiento/09-A10-scripts-ci-instaladores.md:5 | implementacion |
| MEN-4 | MENOR | abierto | tareas/03-en-revision/TASK-138/seguimiento/01-A1-plataforma.md:1 | implementacion |
| MEN-5 | MENOR | abierto | docs/auditoria-4d.md:7472 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Debajo, la reproduccion de cada hallazgo, o "sin hallazgos" explicito.

Ningún CRITICO ni IMPORTANTE. Ninguna ficha de la muestra es falsa: los fragmentos, las líneas, los mutantes y los controles que he reproducido coinciden con lo que dice el documento. Los cinco hallazgos son MENOR y tratan de la coherencia interna del documento y de la reproducibilidad a futuro de tres arneses.

### MEN-1 — El `motivo_severidad` de 8 fichas IMPORTANTE concluye lo contrario de su severidad

Las fichas que el consolidador subió de nivel o confirmó por mutación conservan el motivo antiguo del auditor:

- A7-07 (`docs/auditoria-4d.md:3733`): «… Por eso se deja en MENOR.», con severidad IMPORTANTE.
- A10-01 (`:4721`): «Por eso MENOR y no IMPORTANTE.»
- A10-05 (`:4814`): «No llega a IMPORTANTE: …»
- A8-15, A9-02, A11-02, A11-03 y A11-04: «Sin ejecutar: queda `pendiente-mutacion`», aunque la sección «Mutación» de cada una confirma el resultado.

Cada una lleva su línea `normalizacion:`, que explica el cambio, así que no se pierde información. Pero la ficha se contradice a sí misma, y quien la lea suelta (por ejemplo, la tarea de seguimiento que la cite) se encuentra dos severidades. Reproducción:

```
awk '/^#### A[0-9]+-[0-9]+ · D. · IMPORTANTE/{id=$2} /^#### .* · MENOR/{id=""} /^- motivo_severidad:/ && id!="" && /(Por eso (se deja en )?MENOR|No llega a IMPORTANTE|queda .pendiente-mutacion)/{print id}' docs/auditoria-4d.md
-> A7-07 A8-15 A9-02 A11-02 A11-03 A11-04 A10-01 A10-05
```

Arreglo: reescribir o tachar la conclusión del motivo en esas 8 fichas.

### MEN-2 — La salida pegada en A10-01 atribuye a la línea 60 un estado que no es el suyo

La ficha (`:4738`) pega `SIGUE: … (estado de la linea: 0)`. En el arnés original (`pruebas-claude/taskcodetwo/task-138/a10/demo/d4-cmp.sh`, leído), ese `$?` es el de `[ "$LINEA" = … ] && echo …`, no el de `! cmp -s`. Con dos ficheros iguales, `! cmp -s` da 1. Lo he reproducido con la línea 60 del commit:

```
linea 60: ! cmp -s "$tmp/srv/y/SHA256SUMS" "$tmp/srv/malo/SHA256SUMS"
estado de la linea: 1
SIGUE tras la guarda con ficheros iguales
salida=0
```

La conclusión del caso es correcta: `set -e` no para. Lo que está mal es la cifra rotulada. Es un Disfraz pequeño en la evidencia: una salida pegada que no dice lo que su etiqueta afirma.

### MEN-3 — Los arneses de los D4 de script solo existen fuera del repo, en una carpeta que el cierre del plan borra

A10-01, A10-02 y A11-01 citan `task-138/a10/demo/d4-cmp.sh`, `d4-sin-version.sh` y `task-138/a11/d4-release-284.sh`, pero no incluyen su texto. Estos arneses viven en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-138\`. El paso 7 del plan manda borrar las carpetas `task-138-*`, y el CLAUDE.md global obliga a borrarlas al cerrar la tarea. La tarea de seguimiento A10 (`seguimiento/09-…md:5-7`) usa como criterio de aceptación «el arnés de la ficha … termina con un código distinto de 0». Una vez cerrada TASK-138, ese arnés ya no existe. Se puede reconstruir a partir de la descripción (yo lo hice para A10-01 en 10 líneas), pero el documento promete evidencia reproducible. Arreglo: pegar en cada ficha los 10-15 líneas del arnés.

### MEN-4 — Las tareas preparadas dicen estar «fuera del repo» y están versionadas dentro

La primera línea de los 12 ficheros de `seguimiento/` dice «(fuera del repo; para `taskctl import --tipo fix`)», y están commiteados en `tareas/03-en-revision/TASK-138/seguimiento/`. Ubicarlos ahí lo decidió Carlos, y lo acepto. Lo que no es cierto es la cabecera, que además choca con la regla 5 del CLAUDE.md del proyecto: los ficheros que se pasan a `taskctl import` viven fuera del repo. Arreglo: cambiar la cabecera por «copiar fuera del repo antes de `taskctl import`».

### MEN-5 — La fila «configuración y manifiestos» de la cobertura no enumera sus 11 ficheros

`docs/auditoria-4d.md:7472` cuenta 11 ficheros sin nombrarlos, y `:7585` afirma que todos están cubiertos «por ruta exacta». Al cruzar `git ls-tree -r --name-only d82ea9d` (283 ficheros fuera de `tareas/` y `docs/`) con las filas, 9 ficheros no casan con ninguna fila por ruta ni por carpeta: `.cargo/config.toml`, `.gitattributes`, `.gitignore`, `.taskcode/config.yml`, `Cargo.lock`, `Cargo.toml`, `crates/tc2/Cargo.toml`, `rust-toolchain.toml` y `rustfmt.toml`. Las filas `crates/distribucion/` (dice 2, la carpeta tiene 3) y `xtask/` (dice 3, tiene 4) dejan fuera su `Cargo.toml`. 9 + 2 = 11 cuadra, y el total de 283 también, pero solo por inferencia. La lista exacta está en `alcance.txt`, fuera del repo y destinada a borrarse. Arreglo: enumerar los 11 en la fila o en una nota.

## Reproducción de las comprobaciones

Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-138-r1\repo`, con `git checkout d82ea9dba1f77577d39b32a4f5caea2e140ae8da` y `CARGO_TARGET_DIR` propio en `task-138-r1\target`. Antes de empezar comprobé que no había ningún `cargo` ni `rustc` en marcha, y todas las órdenes `cargo` se lanzaron en serie. El repo principal no se tocó. El clon se borró al terminar.

### 1. Muestra de D1/D2/D3 contra `git show d82ea9d:<fichero>`

Comprobación automática previa:
- Script que recorre todas las líneas de evidencia con forma `ruta:N: contenido` y las compara con `git show d82ea9d:ruta`. Resultado: `ok 605 distinto 1 fichero inexistente 0`. La única diferencia está en A5-03 y es una cita del ADR F1-15:555 recortada por la derecha (es prefijo literal de la línea real).
- Script sobre la columna `fichero:línea` de las 116 filas de la tabla principal: todas existen en d82ea9d con la línea dentro del fichero.

Casos revisados a mano (8):

| Caso | Comprobado | Veredicto |
|---|---|---|
| A1-05 D1 | `tests/plataforma.rs:32-44` y `:568-580`: 13 líneas cada uno; solo cambian la firma y `Personas::nuevas(...)` | real, supera el umbral, no lo impone ningún ADR (son ayudantes de test) |
| A4-05 D1 | `git/estado.rs:451-460` y `:463-473`: el mismo `correr(... LsRemote, ["--get-url", REMOTO])` y la misma condición | real; 10 y 11 líneas, en el límite del umbral |
| A8-09 D1 | `ciclo/mod.rs:367-386` frente a `plan/mod.rs:487-492` y `:527-537`: `escribir` y `con_dueno` son texto idéntico salvo el comentario de doc | real, entre dos dueños |
| A7-07 D2 | `init/aplicar.rs:25-27`, `:82-84` (`Idioma::Es` fijo), `desinstalar/mod.rs:541-542` (literal «cambio desde el plan»), que entra en `.con("error", e)` (`:478`) dentro de `FALLO_AL_APLICAR` con plantilla `en` | real; sube a IMPORTANTE porque cambia la salida (decisión 4 del plan); la entrada no se ejecutó, y así lo declara la ficha |
| A11-05 D2 | `main.rs:34` `ExitCode::from(2)` y `:75` `ExitCode::from(1)`; `git grep -n "ExitCode::from([0-9]" d82ea9d -- crates/tc2/src` da solo esas dos; `contrato.rs:8-10` «vive aqui y solo aqui» | real, MENOR coherente (hoy los valores coinciden) |
| A5-13 D3 | en `seleccion/mod.rs:1051-1113`, `hash` solo aparece en las líneas 1066 y 1105; `cat2` no se vuelve a seleccionar | real |
| A9-04 D3 | `distribucion/src/lib.rs:258-279`: el predicado de `:271` es el mismo de `:173` y el mensaje sale dos veces; el `git grep` citado da exactamente las 4 líneas que dice la ficha | real, MENOR |
| A1-08 (búsqueda) | `git grep '"https://[-.]' d82ea9d -- crates/tc2` da 0 líneas | coincide con la ficha |

### 2. D4 de test con mutación real (3, entre ellos A11-04)

**A11-04** (`contrato.rs:336`, se borra el brazo de los controles):
```
diff: -            c if (c as u32) < 0x20 => s.push_str(&format!("\\u{:04x}", c as u32)),
cargo test -p tc2 --no-fail-fast  -> exit=0
47 binarios, todos 'test result: ok'; 763 passed, 0 failed, 13 ignored
```
Las cifras son idénticas a las de la ficha (47 binarios, 763/0/13). Control (`'"' => s.push('"'),` en `:331`):
```
cargo test -p tc2 --lib contrato::tests::el_diagnostico_va_a_stderr_solo_si_falla
test contrato::tests::el_diagnostico_va_a_stderr_solo_si_falla ... FAILED
panicked at crates\tc2\src\contrato.rs:402:9
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 398 filtered out   rc=101
```

**A1-08** (`plataforma/http.rs:100`, se borra `&& !host.starts_with(['.', '-'])`):
```
cargo test -p tc2 --lib plataforma::http::tests::bases_validas_e_invalidas
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 398 filtered out   rc=0
cargo test -p tc2 --test plataforma el_envoltorio_rechaza_antes_de_la_red
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out     rc=0
```
Control (`:105` pasa a `if false {`):
```
test plataforma::http::tests::bases_validas_e_invalidas ... FAILED
panicked at crates\tc2\src\plataforma\http.rs:494:13
test result: FAILED. 0 passed; 1 failed ...   rc=101
```

**A3-01** (`alta.rs:570`, `Alta::nueva(titulo)` pasa a `Alta::nueva("otro")`):
```
cargo test -p tc2 --lib alta::tests::anotacion_ida_y_vuelta
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 398 filtered out   rc=0
```
Control (`:570` pasa a `None::<Box<dyn Operacion>>`):
```
test alta::tests::anotacion_ida_y_vuelta ... FAILED
panicked at crates\tc2\src\alta.rs:1055:9
test result: FAILED. 0 passed; 1 failed ...   rc=101
```

Los tres mutantes sobreviven en verde y los tres controles dan rojo, como dicen las fichas. Tras la serie, `git status --short` del clon salió vacío.

Nota sobre el control de A11-04: rompe otro brazo de `cadena_json` (`"`). Demuestra que el test corre y que mira `cadena_json`, pero no que haya un test de controles; precisamente eso es lo que denuncia el caso. Es válido según el plan.

### 3. D4 de script: A10-01

Arnés propio en `task-138-r1\a10`. Toma `sed -n 60p` de `git show d82ea9d:.github/scripts/probar-actualizar.sh` (y su `set -eu` de `:19`) y la ejecuta con `sh` (GNU bash 5.3.15, cygwin) sobre dos `SHA256SUMS` iguales, sin red:
```
linea 60: ! cmp -s "$tmp/srv/y/SHA256SUMS" "$tmp/srv/malo/SHA256SUMS"
estado de la linea: 1
SIGUE tras la guarda con ficheros iguales
salida=0
--- control con la forma corregida ("|| { ...; exit 3; }")
PARA: la forma corregida detecta ficheros iguales
salida=3
```
Caso confirmado; la cifra rotulada de la ficha la recojo en MEN-2. A11-01 lo comprobé solo por lectura: `release.yml:283-285` tiene el mismo `! cmp -s` en mitad del paso.

### 4. Cobertura

- `git ls-tree -r --name-only d82ea9d | grep -v '^tareas/\|^docs/' | wc -l` da **283**. La suma de la columna «Ficheros» también da 283.
- 274 ficheros casan con una fila por ruta exacta o por carpeta. Los 9 restantes, junto con 2 `Cargo.toml` de carpetas, explican los 11 de «configuración y manifiestos»; el detalle está en MEN-5.
- Lectura mecánica declarada: `Cargo.lock` más 37 fixtures. Recuento a mano: `estimacion/paridad*` 1, `muestra-f1-15/` 1, `plataforma/` sin `observado-e2e.json` 13, `roadmap/referencia.md` 1, `seleccion-taskcodetwo/TASK-*` 20 y `toolsaweb/tareas.tsv` 1, que suman 37. Cuadra con los 38 declarados.
- Totales de la tabla principal, recontados por script: 108 casos distintos y 8 fusionados (116 fichas). D1 son 54 IMPORTANTE; D2, 3 IMPORTANTE y 9 MENOR (12); D3, 17 MENOR; D4, 25 IMPORTANTE. Hay 18 `divergente`. 108 = 54 + 12 + 17 + 25 cuadra, igual que los 82 IMPORTANTE y los 26 MENOR. Hay 18 filas «sí: mutante en verde y control en rojo» y 18 secciones `##### Mutación`.
- Seguimiento: cada caso `pendiente [G]` de la tabla aparece en el fichero de `seguimiento/` de su grupo (comprobación por script, sin ausencias).

### 5. Casos falsos, severidad y Disfraz en la propia auditoría

- En la muestra no hay ningún caso falso.
- Las severidades son coherentes con las decisiones 4 y 5 del plan. Los D1 y D4 son IMPORTANTE, los D2 y D3 MENOR salvo tres D2 que cambian el comportamiento (A3-08, A5-03, A7-07), y `divergente` no sube a CRITICO. Las incoherencias que encontré están en el texto y no en la severidad (MEN-1).
- A10-05 entra como IMPORTANTE **no verificado**, y su tarea lo reproduce primero o lo descarta. Lo dice abiertamente en la tabla, en «Límites» y en la ficha, así que no lo cuento como Disfraz. A1-07, A7-09 y A7-07 dan entradas derivadas del código sin ejecutar y lo declaran.
- La cabecera de «Tareas de seguimiento» apunta a `tareas/04-terminadas/TASK-138/seguimiento/`. Es la ruta que tendrán después de `finish`; hoy están en `03-en-revision`. Es correcta tras el cierre y no la cuento.
- Criterio 3 (crear las tareas en `00-planificadas`): queda aplazado por decisión de Carlos (2026-11-22). Las 12 tareas están preparadas en `seguimiento/`. No lo cuento como incumplimiento, pero la tarea no puede darse por cerrada respecto a ese criterio hasta crearlas.

## Regla 4D

- Duplicacion: cumple
- Desviacion: MEN-4
- Diluvio: cumple
- Disfraz: MEN-2

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-138/seguimiento/*.md`: forman parte de lo revisado según el encargo.
- El commit d82ea9d (`git show`/`git grep` de los ficheros citados por las fichas de la muestra): para reproducir los casos.
- `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-138\a10\demo\d4-cmp.sh`: el arnés original de A10-01, para explicar el estado 0 de MEN-2.

Veredicto propuesto: aprobada-con-correcciones

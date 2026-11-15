# Plan — TASK-024: F2-8 i18n de mensajes

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 165 min

Señales: unos 22 ficheros de código tocados (2 nuevos: `crates/tc2/src/i18n/mod.rs` y `crates/tc2/src/i18n/catalogo.rs`; los otros 20 son los que hoy emiten mensajes), 4 capas (contrato, CLI, git/estado, plataforma/config), 2 criterios. Pasa de 60 min; la persona decidió no trocear (ver `## Motivo para no trocear`).

**La idea.** El idioma se resuelve en un solo punto, el borde del proceso. El resto del código deja de construir textos en español y pasa a construir un `Mensaje` neutro: una clave del catálogo más sus argumentos. El texto se genera al escribir la salida.

**Lo que no cambia en ningún idioma** (RR-01 y ADR F1-4): la línea de stdout `<comando> <resultado> clave=valor`, las claves del JSON, los valores de `motivo` y los códigos de salida. Hoy el diagnóstico va aparte, a stderr y en una sola línea (`crates/tc2/src/contrato.rs:96-97` y `:201-212`), así que la frontera ya está separada en el código. Solo se traduce `diagnostico`, también dentro de `--json`.

**Qué hay hoy (con evidencia):**
- No hay i18n: todos los mensajes son literales en `.diagnostico("...")` o `format!` (`crates/tc2/src/lib.rs:98-152`). No se usa thiserror (`crates/tc2/Cargo.toml:21-29`).
- `.diagnostico(` aparece 118 veces en 20 ficheros. Tres son tests de `contrato.rs:294-321`, así que quedan **115 llamadas de producción en 19 ficheros**.
- A eso se suman los textos que no pasan por `.diagnostico(`:
  - unos 20 de `crates/tc2/src/config/validar.rs:13-136` (`ErrorConfig::nuevo` y `err(...)`);
  - unos 12 de `crates/tc2/src/config/mod.rs:381-450` (arreglos y `Invalida::una`);
  - 7 de `Regla::texto` en `crates/tc2/src/config/claves.rs:92-113`;
  - 4 que van directos a stderr: `main.rs:12`, `main.rs:25` y `lib.rs:40,77,85` (`USO`, «falta la orden», «orden desconocida»).
- En total son unos 155 sitios. Al quitar duplicados (por ejemplo `git/remoto.rs:345` y `git/diff.rs:20` dicen lo mismo) saldrán unas 120 entradas de catálogo. Esta cifra es una estimación, no un recuento.

**Qué se crea:**
1. `i18n/mod.rs`:
   - `enum Idioma { Es, En }`.
   - `Idioma::de_entorno(leer: impl Fn(&str) -> Option<String>) -> Idioma`. El orden es `TC2_IDIOMA` (`es` o `en`), luego `LC_ALL`, `LC_MESSAGES` y `LANG` (se mira el prefijo `en` o `es`; `C` y `POSIX` se ignoran), y si no hay nada, `Es` (RF-73).
   - `struct Mensaje { clave, args }` con `.con("ruta", v)` y `fn texto(&self, Idioma) -> String`, que sustituye los huecos `{nombre}`.
   - El entorno se pasa como función de lectura porque, en edition 2024, `set_var` exige `unsafe` y el workspace tiene `unsafe_code = "forbid"` (`Cargo.toml:16`). Así se puede probar sin tocar el entorno.
2. `i18n/catalogo.rs`: una tabla estática hecha con un `macro_rules!` que genera `enum Clave`, `Clave::TODAS` y dos `match` exhaustivos, `es(Clave)` y `en(Clave)`. Si falta una entrada en un idioma, ni compila. Coste de ejecución nulo y ninguna dependencia nueva.

**Qué se extiende:**
- `contrato::Informe`: `diagnostico` pasa de `Option<String>` a `Option<Mensaje>`. Se añade `Informe::salida_en(json, Idioma)`; `salida(json)` se queda como atajo en `Es`, para no reescribir los tests que ya existen.
- `lib.rs`: se añade `ejecutar_en(idioma, args)`, y `ejecutar(args)` pasa a delegar en ella con `Es`.
- `main.rs`: es el único sitio que llama a `Idioma::de_entorno(|k| std::env::var(k).ok())`.
- `config::ErrorConfig.mensaje` pasa a ser `Mensaje`.
- `Mensaje::externo(texto)` solo sirve para meter como argumento un texto que viene de fuera (stderr de git, error del sistema, HTTP). No hay `From<String>`, para que ningún literal en español se cuele sin pasar por el catálogo.

**Por qué esta forma:**
- Si se pasara `Idioma` por parámetro, habría que cambiar la firma de unas 60 funciones de `git/`, `estado/` y `plataforma/`.
- Un `OnceLock` global es estado oculto y mezcla idiomas entre tests que corren en paralelo dentro del mismo proceso.
- Aplazar el texto hasta la salida deja el núcleo sin saber nada del idioma, y así el idioma solo se decide en `main.rs`.

**Crate externo frente a catálogo propio: recomiendo el catálogo propio.**
- `fluent` arrastra fluent-bundle, unic-langid, intl-memoizer y self_cell. Además analiza los `.ftl` al arrancar, sus claves son texto y una clave que falta solo se descubre en tiempo de ejecución. Eso va contra los criterios 1.º (arranque) y 2.º (tamaño) de D1a en ADR F1-14 (`docs/adr/F1-14-distribucion-e-instalacion.md:40`). F1-14 midió solo 3 dependencias directas (`:202`).
- `rust-i18n` mete una macro procedural, un idioma global mutable (`set_locale`) y claves de texto sin tipo.

**La elección de idioma no va en la configuración.**
- `claves::tabla()` ya tiene 12 claves de primer nivel (`claves.rs:386-529`), y el tope es `TOPE_CLAVES = 12` (`claves.rs:10`, con su test en `:636`). Una clave más rompería el tope que fija el plan.
- Además, la configuración se lee de `origin/develop` y es del proyecto (`config/mod.rs:39-40`), mientras que el idioma es de cada persona: en un mismo equipo puede haber dos idiomas.

**Coherencia con los ADR:**
- No hay ningún ADR sobre idioma (la búsqueda en `docs/adr` no da nada).
- F1-4 B1 fija `LC_ALL=C` para las llamadas a git (`F1-4…md:125`). Eso es el entorno de los procesos hijos y no afecta a cómo `tc2` lee su propio entorno.
- No contradice ningún ADR.

**Orden:**
1. `i18n` con el catálogo vacío más las entradas de `lib.rs` y `main.rs`.
2. `Informe` y `salida_en`.
3. `ejecutar_en` y `main.rs`.
4. Tests de los dos criterios.
5. Migrar módulo a módulo. Hay un test de guarda con una lista de ficheros aún sin migrar que se va vaciando hasta quedar en cero.

## Motivo para no trocear

Decisión de la persona (2026-11-15): la tarea se hace entera en TASK-024, unos
165 min, sin `taskctl trocear`. La migración de los mensajes no aporta nada
por partes: hasta que no están todos en el catálogo, F2-8 no queda cumplida
(RF-73 pide es/en en todos los mensajes). Para contener el tamaño se implementa
en tres bloques, en este orden, con la suite en verde al acabar cada uno.

## Bloques de implementacion (en la misma tarea)

### Bloque a: infraestructura y borde del CLI (unos 65 min)
- Crear `i18n/mod.rs` y `i18n/catalogo.rs`.
- Cambiar `Informe` y añadir `salida_en`.
- `ejecutar_en` y la resolución de idioma en `main.rs`.
- Migrar `lib.rs`, `main.rs` y `estado/lectura.rs` (unos 13 mensajes).
- Test del catálogo completo y test de guarda; la lista de excepciones arranca con los 17 ficheros pendientes.
- Test de punta a punta con el binario.

Al terminar este bloque, F2-8 ya funciona de punta a punta. Solo queda traducir el resto de mensajes.

### Bloque b: `git/` y `estado/` (unos 50 min)
- Unos 66 mensajes en `git/{mod,estado,rama,remoto,guardar,diff,tag,worktree}.rs` y `estado/{montaje,transaccion}.rs`.
- Cuando el mensaje lleva el stderr de git, ese texto va como argumento con `Mensaje::externo`.
- Se sacan de la lista de excepciones los ficheros migrados.

### Bloque c: `plataforma/` y `config/` (unos 50 min)
- Unos 41 mensajes de `plataforma/{errores,conexion,puertas,github,gitlab,deteccion}.rs`.
- Unos 40 de `config/{mod,validar,claves}.rs`, incluido `ErrorConfig` y los textos de `Regla`.
- La lista de excepciones queda vacía y el test de guarda pasa a exigir que todo vaya por el catálogo.

## Lo que el rol no cubrio

Comparado con los criterios de aceptación:
- **«Test que falla si falta una traducción»**: he diseñado el mecanismo (compilación más test). No he analizado con un rol de testing si el test basta, ni lo he probado con mutantes.
- **«Cumple F2-8 y los requisitos que cita»**: RF-73 pide además que cada mensaje diga «el siguiente paso» (`docs/plan/REQUISITOS-FUNCIONALES.md:198`). **No he revisado** si los 155 textos actuales lo cumplen. Es una regla de dominio y la migración no la arregla por sí sola.
- Casos límite, sin analizar (son del rol de riesgos): valores raros de `LANG` (`en_US@euro`, `es-419`, vacío), `TC2_IDIOMA` con un valor no válido, y argumentos que contengan `{`.
- Calidad de la traducción al inglés: no la revisa nadie nativo.
- Qué cuenta como «mensaje dirigido a la persona»: he tomado los diagnósticos, los errores de configuración y el uso. Hay cosas que he dejado fuera **sin la validación del dominio**:
  - el contenido de `tarea.md` y los commits que genera `alta`, y el esquema JSON publicado, porque son artefactos compartidos;
  - las descripciones de `claves.rs`, que solo van al esquema.

## Riesgos aceptados y que los contiene

- **Quien usa Windows sin `LANG` ni `TC2_IDIOMA` ve los mensajes en español** aunque su sistema esté en inglés. Leer el idioma de Windows pide FFI (prohibido por `unsafe_code = "forbid"`) o un crate como `sys-locale`. Lo contiene: `TC2_IDIOMA`, documentado en el `--help` y en el README.
- **Los textos de git, del sistema y de HTTP salen en su idioma de origen** dentro del mensaje traducido. Lo contiene: van marcados como `externo`, y git va con `LC_ALL=C` (F1-4).
- **Una prueba que corra el binario en una máquina con `LANG=en_*` vería los diagnósticos en inglés.** Lo contiene: los tests en el mismo proceso usan `ejecutar` (siempre `Es`), y los de binario que comprueban stderr fijan `TC2_IDIOMA` (los tests de `tests/` que comprueban diagnósticos solo son 4).
- **El test de guarda es un análisis de texto y se puede esquivar.** Lo contiene: cuenta también los usos de `Mensaje::externo` contra una lista cerrada de excepciones.
- **El catálogo es un solo fichero grande**, con riesgo de conflictos si hay tareas en paralelo. Se acepta a cambio de tener un solo sitio que traducir y probar.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- **Criterio 1, «Test que falla si falta una traducción»: 2 tests unitarios, ninguno pesado.**
  - (1) `catalogo_completo`: recorre `Clave::TODAS` y comprueba que `es` y `en` no están vacíos, que el inglés es distinto del español (salvo una lista cerrada, por ejemplo los mensajes que solo nombran un comando) y que los dos tienen los mismos huecos `{…}`. Que falte una entrada entera ya lo para el `match` exhaustivo al compilar.
  - (2) `ningun_literal_fuera_del_catalogo`: lee `src/` con `CARGO_MANIFEST_DIR`, falla si hay `.diagnostico("`, `.diagnostico(format!` o `ErrorConfig::nuevo(…, "…")` fuera de `i18n/` y de los tests, y lleva la cuenta de `Mensaje::externo`.
- **Criterio 2, «F2-8 y RF-73 (es/en)»: 3 tests, ninguno pesado.**
  - (1) `resolucion_de_idioma`: tabla de casos con lectura de entorno inyectada: orden de prioridad, `C`/`POSIX`, valor no válido y el español por defecto.
  - (2) `mismo_informe_dos_idiomas`: stdout y código iguales y stderr distinto, en línea y en `--json`.
  - (3) punta a punta en `tests/cli.rs`: `TC2_IDIOMA=en tc2 nada` da un stderr en inglés con el mismo código 2. Lanza un proceso pero no usa git ni red.
- **Bloques b y c**: sin tests nuevos. La migración la cubren los tests de los criterios y la lista de excepciones que se vacía; la suite actual tiene que seguir en verde con el español por defecto.

## Lo que necesita decision de una persona

1. **Cómo se elige el idioma**: `TC2_IDIOMA`, luego `LC_ALL`/`LC_MESSAGES`/`LANG`, y si no, español. *Recomiendo esto.* No meterlo en la configuración: rompe el tope de 12 claves y el idioma es de cada persona, no del proyecto.
2. **Detectar el idioma de Windows**: *recomiendo que no en esta tarea.* Pide una dependencia (`sys-locale`) o `unsafe`. Si se quiere, mejor una tarea aparte que pase la regla D1a.
3. **Catálogo propio o crate**: *recomiendo el catálogo propio* con `macro_rules!` (cero dependencias, sin coste al arrancar y comprobado al compilar), por delante de `fluent` y `rust-i18n`.
4. **Alcance y troceo**: *recomiendo* hacer la parte a en TASK-024 y abrir las partes b y c como tareas propias, que dependan de ella. La otra opción es todo en TASK-024, unos 165 min.
5. **Qué se queda siempre en español**: *recomiendo* dejar en español el esquema JSON publicado, el contenido de `tarea.md` y los commits. Son artefactos compartidos: si dependieran de quién los genera, dejarían de ser deterministas.
6. **Tildes en el catálogo en español**: *recomiendo* mantener los textos actuales tal cual (sin tildes, como `lib.rs:134`) para no tocar los tests de ahora, y poner las tildes en una tarea aparte. La regla de no usar tildes vale solo para los commits.
7. **El `diagnostico` del `--json` se traduce y no se añade ningún campo nuevo**: *recomiendo que sí.* Para la máquina ya están el `motivo` y el código.

Respuestas de la persona (2026-11-15):

- Punto 4 (alcance y troceo): **todo en TASK-024**, sin trocear (ver `## Motivo para no trocear`). No se sigue la recomendación.
- Puntos 1, 2, 3, 5 y 7: **aceptadas las recomendaciones** tal cual.
- Punto 6 (tildes): **textos tal cual, sin tildes**; las tildes del catálogo en español van en una tarea aparte.

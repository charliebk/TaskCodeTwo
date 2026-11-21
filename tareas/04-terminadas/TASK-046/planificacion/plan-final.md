# Plan — TASK-046: F5-6 Hook local rapido, opcional y con confianza

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

El nucleo es un **comando determinista**; el hook es solo una forma de llamarlo (F1-11 dec. 10, memoria «comandos deterministas sobre hooks"). Se crean tres cosas:
- la interfaz unica de confianza de F1-8 dec. 5, `confianza(repo, clave) -> Confiado | Cambiado | Nunca`;
- `tc2 confiar <clave>`;
- `tc2 verificar --perfil <p>`, que consulta la confianza antes de lanzar nada.

El hook es la misma orden con `--hook`, colgada del evento `Stop` de Claude Code: solo cambia como devuelve el resultado, la decision es la misma.

Precedente comprobado por el rol: en `crates/tc2/src` nada ejecuta todavia una orden de la config ni hay codigo de confianza (`grep confian` solo encuentra `confianza_incluye` en `config/`). Esta tarea es la primera que ejecuta algo del repo y deja la costura de la que colgaran los `derivados` (F1-11).

**Piezas y limites:**
- `crates/tc2/src/confianza/` — **se crea**, generica por `clave` (no sabe de hooks ni de perfiles):
  - cierre de los scripts versionados que nombra la orden (tokens con forma de ruta, `%~dp0` resuelto) mas los globs de `confianza_incluye`;
  - huella `sha256` de la orden y del cierre con `ring::digest` (ya dependencia de `crates/tc2/Cargo.toml`);
  - almacen fuera del arbol, en el directorio de datos de `instalacion.rs` (`%LOCALAPPDATA%\taskcodetwo\confianza\` o `$XDG_CACHE_HOME/taskcodetwo/confianza/`), un fichero por repo nombrado con el `sha256` de la ruta canonica de `git::estado::dir_comun`: los worktrees comparten confianza y otro clon no. Cada entrada: `{clave, huella, ficheros: [(ruta, tamano, mtime, sha)]}`.
  - API: `pub fn estado(repo, clave, orden, incluye) -> Result<Estado, Informe>` y `pub fn confiar(repo, clave, orden, incluye) -> Result<Huella, Informe>`.
- `crates/tc2/src/verificacion/ejecutar.rs` — **se crea** junto a `verificacion/informe.rs` (F5-4): lee `verificacion` de `.taskcode/config.yml` **del arbol de trabajo** con `config::leer`; pregunta a `confianza::estado` por la clave `verificacion.comando`; lanza `verificacion.shell` + `comando --perfil <p>` con `TC2_PERFIL=<p>` y el `timeout` del perfil; aplica F1-11 dec. 8 (un `timeout` agotado no es rojo en `rapida`).
- `lib.rs::ejecutar_en` — **se extiende** con `verificar` y `confiar` y sus catalogos i18n en `i18n/catalogo/verificacion.rs`.
- Adaptador `--hook` — **se crea** en `verificacion/ejecutar.rs`: lee el JSON de stdin de `Stop`; sale con 0 si `stop_hook_active` es `true`; sin confianza, sin perfil `rapida` o con la config invalida sale con 0 y una linea en stderr que nombra `tc2 confiar verificacion.comando` (nunca bloquea por eso); con la rapida en rojo sale con 2 y un resumen corto.
- Instalacion del hook — **sin codigo**: se documenta la entrada de `settings.json` (`tc2 verificar --perfil rapida --hook`), que la persona anade solo si quiere. `desinstalar/texto.rs::hooks_con_tc2` ya detecta los hooks que nombran `tc2`.

**Orden de construccion** (cada paso probado por separado): 1) `confianza` y `tc2 confiar`; 2) `verificacion::ejecutar` con la puerta, la shell, el `timeout` y los codigos; 3) el adaptador `--hook`; 4) documentacion de la entrada del hook, cifra medida del presupuesto de 1 s y aviso en la guia.

Duracion estimada: 270 min (de la rama abierta al cierre con merge; senales: unos 6 ficheros de codigo nuevos o extendidos, 3 capas —confianza, verificacion, CLI/i18n—, 3 criterios, 7 tests E2E contra repos temporales).

## Troceo

No se trocea, por la consigna de Carlos del 2026-11-20 («no troceamos tareas»). Ademas, la confianza sin quien la use no tiene prueba E2E del criterio 2, y partir comando y hook dejaria un tramo intermedio que ejecuta codigo del repo sin la puerta probada de punta a punta.

## Lo que el rol no cubrio

- **Criterio «≤ 1 s»:** la forma (D1a, D2a) es de este rol; no hubo rol de rendimiento. Lo cubre el test pesado P1.
- **Criterio «nada del repo sin confianza»:** los modos de fallo (enlaces simbolicos, mayusculas en rutas de Windows, `mtime` manipulado, TOCTOU entre hashear y ejecutar) son del rol de riesgos, que no se lanzo: **no revisados**. El criterio queda comprobable asi: `tc2 verificar --perfil rapida` sobre una orden no confiada sale con codigo distinto de 0 y **no crea** el fichero testigo; tras `tc2 confiar verificacion.comando` si (P3-P5).
- **Criterio «cumple F5-6»:** reglas de dominio no validadas con su rol (que es «opcional» para la persona, que es «repo ajeno»).
- **Estrategia de pruebas:** minima, sin rol de testing.

## Riesgos aceptados y que los contiene

- **Fuera del cierre** queda el codigo al que se llega por herramientas (`build.rs`, tests, dependencias) o cargado por nombre de modulo: limite aceptado en F1-8 dec. 5; lo contienen `confianza_incluye` y que el hook solo corre la rapida, opcional.
- **Cache por `(tamano, mtime)`:** un cambio que conserve los dos no se detecta en caliente; la orden se hashea siempre y `tc2 confiar` rehashea todo.
- **Este repo no puede estrenar el hook todavia:** su config es de la 0.16 (sin `formato: 2`) y `config::leer` la rechaza; el hook sale con 0 y avisa. La prueba real va sobre repos temporales.
- **Almacen en el directorio de cache:** si se borra, la confianza vuelve a `nunca` (falla cerrado).

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

Todos contra **repos git temporales reales**, sin mocks, con `LOCALAPPDATA`/`XDG_CACHE_HOME` temporal propio y las ordenes del proyecto como scripts triviales (nunca `cargo`). Cada test con la mutacion que lo pone rojo.

**Criterio 1 — ≤ 1 s** (2 tests)
- P1 (**pesado**, mide tiempo, solo en `completa`): cierre de 51 scripts versionados y 1,2 MB, como ToolsaWeb, orden confiada que sale enseguida: `tc2 verificar --perfil rapida` ≤ 1 s en frio y en caliente; sin confiar, ≤ 1 s en total. Rojo si se quita la cache de D2a o se hashea el arbol entero.
- P2 (ligero): en caliente y sin cambios, los bytes hasheados valen 0; al tocar un script, su tamano. Rojo si se ignora `(tamano, mtime)`.

**Criterio 2 — nada del repo sin confianza** (3 tests)
- P3: orden que crea `PWNED`. Sin confiar: codigo distinto de 0, aviso con `tc2 confiar verificacion.comando`, `PWNED` no existe. Tras confiar: codigo 0 y `PWNED` existe. Rojo si se salta la puerta.
- P4: con la orden confiada, cambiar un script del cierre, un fichero de `confianza_incluye` y la orden: cada vez `cambiado` y no se ejecuta; al volver al contenido original, `confiado`. Rojo si se deja fuera `%~dp0` o los globs.
- P5: un fichero del repo que diga «confiado» no cuenta; un worktree comparte la confianza; un segundo clon da `nunca`; el almacen esta en `<datos>/taskcodetwo/confianza/` y nada nuevo en el arbol. Rojo si la clave sale del toplevel y no de `dir_comun`.

**Criterio 3 — cumple F5-6 (opcional, rapida, mismo comando)** (2 tests)
- P6: `--hook` con `stop_hook_active: true` sale con 0 sin ejecutar; sin confianza o sin `rapida`, 0 con aviso y sin ejecutar; con la rapida en rojo, 2. Rojo si `--hook` bloquea sin confianza.
- P7: `--hook` y la orden sin hook toman la misma decision y motivo; en `rapida` un `timeout` agotado no es rojo. Rojo si los dos caminos se separan.

## Lo que necesita decision de una persona

Carlos acepto el 2026-11-20 todas las recomendadas y sin trocear; quedan anotadas:
- **D1a** (recomendada, aceptada): «≤ 1 s» es el coste propio de `tc2` hasta decidir (leer config, cierre, huella, almacen, lanzar o rechazar), en frio y en caliente; cuando no se ejecuta, es el tiempo total. La rapida queda acotada por `perfiles.rapida.timeout` (la de este repo tiene 2400 s; ningun hook la hace caber en 1 s). Descartada D1b (rapida en segundo plano): ya no seria el mismo comando.
- **D2a** (recomendada, aceptada): cache del `sha` por `(tamano, mtime)`. Descartada D2b (rehashear siempre: 0,8-0,9 s solo de huella con un cierre como el de ToolsaWeb).
- **D3a** (recomendada, aceptada): la orden se lee del arbol de trabajo (F1-8 dec. 5), no de `config::de_base`: la huella cubre lo que se ejecuta de verdad.
- **D4a** (recomendada, aceptada): evento `Stop` de Claude Code; descartados `pre-push`/`pre-commit` (los push los hace `tc2`) y `post-checkout` (F1-8 prohibe ejecutar al cambiar a una rama ajena).
- **D5a** (recomendada, aceptada): solo la entrada documentada, sin `tc2 hook activar`.
- **D6** (recomendada, aceptada): `--perfil` acepta cualquier perfil declarado; el hook fija `rapida`.

## Suposiciones no verificadas

- F0-2 no tiene tarea en `tareas/`: F5-6 asume la implementacion de la confianza que F1-8 asignaba a F0-2. Se comprueba con `taskctl roadmap` al implementar y se anota en el Resultado.
- `desinstalar` borra todo `%LOCALAPPDATA%\taskcodetwo`, almacen incluido: comprobar en `desinstalar/maquina.rs`.
- La forma exacta del JSON de stdin de `Stop` (`stop_hook_active`): comprobar en la documentacion de hooks de Claude Code.
- El guion del prototipo de cierre de la PoC A1 de F1-8 como referencia para resolver rutas: localizarlo en `docs/adr/` o sus anexos.

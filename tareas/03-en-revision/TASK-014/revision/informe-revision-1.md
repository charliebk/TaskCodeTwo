# Informe de revision — TASK-014 (ronda 1)

- Commit revisado: 36828cc4bd191733e0f3aa8fcc8b29c1c6f0ac7e
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

Pase generico (el diff es documentacion y una PoC en Python, Rust y TypeScript sin revisor de dominio unico). Profundidad: a fondo el guion, el ADR y sus cifras, la aplicacion de D1a y la coherencia con el plan y los ADR previos; por encima el codigo de `poc-rust/src/main.rs` y `poc-ts/src/main.ts` (su comportamiento se juzgo por la salida identica sobre datos reales, no por lectura linea a linea).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-014/poc/poc-f1-14.py:247 | implementacion |
| MEN-1 | MENOR | corregido | docs/adr/F1-14-distribucion-e-instalacion.md:483 | implementacion |
| MEN-2 | MENOR | corregido | tareas/03-en-revision/TASK-014/poc/poc-f1-14.py:761 | implementacion |
| MEN-3 | MENOR | corregido | docs/adr/F1-14-distribucion-e-instalacion.md:407 | plan |
| MEN-4 | MENOR | corregido | docs/adr/F1-14-distribucion-e-instalacion.md:359 | implementacion |
| MEN-5 | MENOR | corregido | docs/adr/F1-14-distribucion-e-instalacion.md:97 | implementacion |

## Reproduccion

### IMP-1 — El guion no se puede ejecutar fuera de `02-en-curso`

El ADR (linea 71) documenta `python -I -B tareas/<estado>/TASK-014/poc/poc-f1-14.py ...` como valido en cualquier estado, pero el guion fija la ruta `tareas/02-en-curso/TASK-014/poc` en cuatro sitios (lineas 247, 285, 558 y 622). La tarea esta ahora en `03-en-revision` (y acabara en `04-terminadas`).

Reproduccion: `python -I -B <repo>/tareas/03-en-revision/TASK-014/poc/poc-f1-14.py <trabajo> <repo> <Personal>` cae en la seccion B con `NotADirectoryError: [WinError 267] El nombre del directorio no es válido` (el `cwd` de `cargo build` no existe), sin ningun mensaje que diga la causa. El log entregado no se puede regenerar tal cual desde el estado en el que lo recibe un revisor ni despues de cerrar la tarea; la evidencia (RR «evidencia, no suposicion») deja de ser reproducible desde el repo.

Arreglo: derivar la carpeta `poc/` de `os.path.dirname(os.path.abspath(__file__))` en vez de la ruta fija (y la ruta relativa del `git status` de la linea 285 de ella), o comprobar que existe y salir con un mensaje que lo diga.

Para continuar la revision movi la tarea a `02-en-curso` solo dentro de mi clon temporal (commit local en el clon; el repo real no se toco) y ejecute el guion completo desde ahi.

### Lo que reproduje (todo en `%LOCALAPPDATA%/pruebas-claude/taskcodetwo/task-014-r1/`, ya borrado)

- **Guion completo** (`python -I -B ...`, compila los dos binarios, clona ToolsaWeb y OpenDockTrading en `8bebfd049` y `8141a08a7`): `RESULTADO: todo como se espera`, exit 0, carpeta de trabajo del guion borrada. Comparado con `poc-f1-14.log` entregado con `Compare-Object`: **la unica diferencia son las lineas de la seccion T** (tiempos de reloj); todo lo demas coincide linea a linea, incluidos tamanos (2720768 y 87430656 bytes), hashes iguales/distintos, 404/0 y 96/0, firma, X, M, D, D1 y las 14 mutaciones de control (cada una falla por su motivo).
- **Tiempos de la nueva pasada**: Rust caliente 68 / 372 / 146 ms frente a TypeScript 156 / 703 / 328 ms (vacio / ToolsaWeb / OpenDockTrading); frio Rust 106 / 402 / 181 frente a 1163 / 1698 / 1339. Misma relacion que en el ADR (Rust mas rapido en los 3 repos, en frio y caliente; con 404 tareas Rust tarda 0,53 veces lo de TypeScript; diferencia en vacio unos 88 ms frente a los «unos 85» del ADR).
- **Recuentos con PyYAML por mi cuenta** (script propio, otro codigo que el del guion, cargador que detecta claves duplicadas): ToolsaWeb 404 `tarea.md`, 404 validas, estados `planificada 241, terminada 162, en-diseno 1`; OpenDockTrading 96, 96 validas, `planificada 52, terminada 44`. Coinciden con el ADR y F1-9.
- **Mutaciones**: las 14 del guion fallan. Mutaciones propias en prosa (ver MEN-2) no se detectan.
- **`/Brepro` entre rutas distintas** (comprobacion extra): dos builds Rust con `-C link-arg=/Brepro` desde dos carpetas fuente y dos `CARGO_TARGET_DIR` distintos dan el mismo sha256 (`9E0D2D75...47B5`, 2720768 bytes). El ADR solo afirma «misma carpeta y misma maquina» (lo declara en «No verificado»): es mas prudente que lo medido.
- **Hooks de la 0.16.1**: `bin/taskctl` empieza por `#!/usr/bin/env node` y `hooks/hooks.json` tiene cinco hooks `"command": "node"`, como dice el contexto del ADR.
- **Metadatos de paquetes**: `npm view postject time` da `1.0.0-alpha.6` el 2023-05-19 y nada despues; `yaml` 2.9.1 (2026-09-11); `Cargo.lock` fija `serde-saphyr` 2.1.1 y `ureq` 3.5.2. Coincide con los `DEPENDENCIAS.md` y con el ADR.
- **Fuentes oficiales, solo lectura (WebFetch)**: Apple Developer Program 99 USD por ano (coincide); GitHub 50 MiB aviso, 100 MiB bloqueo, releases sin limite total (coincide); Azure Artifact Signing: personas fisicas solo EE. UU. y Canada (coincide); winget: `InstallerUrl` debe ser el sitio de releases del ISV, sin requisito de firma en las politicas (coincide, ver MEN-1 por la fecha). No pude consultar los importes de Azure (el ADR ya lo marca «no verificado»).

### MEN-1 — Fechas y alcance de dos fuentes citadas

- ADR linea 483: «las politicas del repositorio (documento con fecha 2026-08-30)». La fecha del propio documento es **22 de mayo de 2021** («Document date» y «Effective date»); 2026-08-30 es la ultima actualizacion de la pagina (`ms.date`). Cambiar a «pagina actualizada el 2026-08-30; documento de 2021-05-22».
- ADR linea 479: «para organizaciones tambien en la UE» es cierto pero incompleto: la pagina lista ademas Reino Unido, Australia, Nueva Zelanda, Japon, Corea del Sur, Singapur, Suiza, Noruega e Israel. No cambia la conclusion (una persona en Espana no puede), pero conviene no recortar la fuente.

### MEN-2 — La comprobacion del ADR no cubre las cifras de la prosa

El ADR dice (linea 104 y siguientes) que el guion sale con 1 si una cifra no es la medida. Eso vale para la tabla `cifras` y los numeros en negrita del `resumen`; las mismas cifras repetidas en Alternativas, Decision, «Cómo se ve en GitHub» y las lecturas (32,1 veces, 15,6 y 500,3 MiB, 147 frente a 215 lineas, «en los 3 repos», «2.º y 4.º») no estan vigiladas. Mutaciones propias aplicadas con `errores_adr` sobre la copia del ADR (500,3 → 600,3 en el punto 4; 32,1 → 3,2 en Alternativas; «en los 3 repos» → «en 2 repos»; «rechazan» → «aceptan» la clave duplicada; «2.º y 4.º» → «otro mas»): **ninguna se detecta**. Revise a mano todas las cifras de la prosa contra el log y hoy coinciden (por eso es MENOR), pero la garantia que describe el ADR es mas estrecha que la real: acotar la frase a «tabla y resumen» o extender la comprobacion.

### MEN-3 — El coste del lanzador no esta medido ni anotado como no verificado

La decision 1 elige Rust por el arranque en caliente (RR-19, la status line), medido con el binario llamado directamente. La decision 4 pone delante de cada llamada un lanzador (`bin/tc2` en sh, `tc2.cmd`/`tc2.ps1`) que detecta target, busca la cache y ejecuta; los hooks y la status line pasaran por el. Su coste por llamada (en especial un `.ps1`, o comprobar el hash de la cache en cada arranque) puede ser del orden de los 85 ms que separan a Rust de TypeScript, y no aparece ni en la PoC ni en «No verificado» (que solo dice que el lanzador «no existe»). Tampoco dice si el binario ya en cache se vuelve a verificar contra `cli.lock` en cada uso o solo al descargarlo: lo primero cuesta arranque, lo segundo deja el binario en cache sin proteccion frente a otro proceso del mismo usuario (F1-8, zona Z1). No cambia el ganador (la relacion sigue a favor de Rust), pero la afirmacion «Rust por arranque en caliente» describe el binario, no el camino real de invocacion. Anadir a «No verificado» el coste del lanzador y fijar en el contrato si se reverifica la cache.

### MEN-4 — Criterios 4 y 5 de D1a: aplicados con datos flojos, bien declarados pero no reflejados arriba

- El criterio 4 se decide por postject (herramienta de build en alfa) y no por las librerias YAML de runtime, que cumplen F1-9 las dos; ademas el ADR describe el YAML de Rust como «movedizo» y a `serde-saphyr` como «joven». La asignacion a Rust es defendible y el ADR lo explica con una sensibilidad honesta («sin el 4 la regla no decide»), pero esa salvedad solo esta en la Decision y el resumen: la fila L1 de «Alternativas» (linea 38) y la fila del indice (`docs/adr/README.md:23`) dicen «por la regla D1a» sin ella. El «Rust» del criterio 4 y el «ninguno» del 5 son constantes del guion (`CRITERIO_4`, `CRITERIO_5`, lineas 55-59), no medidas, aunque la tabla del ADR las lista como cifras de la seccion D1.
- Criterio 5 (linea 360): «~8 frente a ~9 min» no cuenta las tres iteraciones extra de compilar y comparar que necesito TypeScript para igualar a Rust (`poc-ts/NOTAS.md`) ni las lineas 147 frente a 215; por otro lado la version de Rust tuvo un fallo funcional propio. «ninguno» es una lectura prudente, pero la frase deberia citar las iteraciones extra.
- Comparacion de condiciones: Rust se compilo con LTO, `codegen-units = 1`, `strip` y `panic = abort`; el SEA de TypeScript se midio sin probar `useCodeCache` ni `useSnapshot` (no los he verificado; la documentacion de Node SEA los ofrece). El ADR no lo dice en «No verificado». El margen medido (2 a 3 veces en caliente con 404 tareas, 32,1 veces de tamano) hace improbable que cambie el resultado.

### MEN-5 — Matices de redaccion de lo «no descargado»

- ADR linea 97: «Nada descargado: sin bun, zig, targets de rustup ni imagenes Docker». Cierto, y la linea siguiente declara la red usada (`npm ci`, crates, GET). Falta decir que el guion tambien baja con `cargo build` las crates propias de Linux y macOS **sin** `--offline` en la seccion X (lo dice la cabecera del guion, linea 12, no el ADR salvo en la lista de red). No es incumplimiento de la regla de Carlos (crates de dependencias, no toolchains), pero conviene que el ADR lo diga con las mismas palabras que el guion. Menor.

## Coherencia con ADR previos y plan (sin hallazgos)

- F1-4 (git como proceso: el comando lanza `git log -1` como hijo; codigo de precondicion sin numero fijo y choque del 4 remitido a F1-18), F1-6 (GET HTTPS, TLS con almacen del sistema, medido en las dos implementaciones), F1-9 (404 y 96, duplicadas rechazadas, PyYAML las acepta en silencio: reproducido), F1-10 (suelo en el binario), F1-11 (decision 10 comandos sobre hooks), F1-12 (`tc2`, `taskcodetwo`, npm `taskcodetwo` libre segun su PoC) y F1-18 (IDs y choque del 4) son coherentes con el ADR.
- Divergencias con el plan anotadas y no editadas (F2-1 TypeScript, P12 «trae el CLI ya compilado», P12/F2-11 firma o hash, F2-10 «firmados», F2-11 y F2-12 con `taskctl`, F3-8, criterio de «tres sistemas», D2a con bun): completas y con texto propuesto. El plan no se modifica.
- Manifiesto de recursos: `engram` esta en «conveniente» («engram no es obligatorio» presente; la mutacion `engram-obligatorio` falla); obligatorio solo git, `tc2` compatible, skills de fase y version minima.
- Seguridad de la distribucion: hash de `cli.lock` antes de ejecutar, renombrado atomico, borrado de la descarga si no casa, nunca ejecutar sin comprobar; honesto en que hash e integridad no son autenticidad y en que attestation, SmartScreen, MOTW y Gatekeeper no estan verificados. Salvedad en MEN-3 (cache) y que «comprometer una sola de las dos cosas no basta» no cubre a quien controle a la vez el repo y las releases de la misma cuenta; el ADR ya lo matiza con «no da autenticidad del autor».
- Lo no verificado esta marcado: Linux, macOS, Windows ARM64, TypeScript fuera de Windows, compilacion cruzada, firma real, importes de Azure, scope npm, attestation, tokens por implementacion.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-014/poc/` (guion, log, especificacion, `poc-rust/`, `poc-ts/`) y `revision/peticion-revision-1.md`: lo pedia el encargo de la revision (reproducir la PoC).
- `docs/plan/PLAN-TASKCODE-2.md` (P12, F2-1, F2-10..12, F3-8, RR-16), `docs/plan/AMPLIACION-2026-11-14.md` y los ADR F1-4, F1-6, F1-8, F1-9, F1-11, F1-12 y F1-18: contrastar coherencia, pedido en el encargo.
- Cache del plugin 0.16.1 (`bin/taskctl`, `hooks/hooks.json`) en solo lectura: comprobar lo observado en el contexto del ADR.
- Clones temporales de ToolsaWeb y OpenDockTrading (solo lectura, nada ejecutado dentro) en commits fijos: recuentos independientes.
- Paginas oficiales de Apple, GitHub y Microsoft y metadatos de npm: contrastar fuentes citadas.

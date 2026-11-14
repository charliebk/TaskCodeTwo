# Plan — TASK-014: F1-14 ADR distribucion e instalacion

(Respuesta del unico rol de brainstorm, arquitectura, volcada con ajustes del orquestador marcados. Con un solo rol no hay unificador; el plan dice que lados no reviso ese rol.)

## Enfoque propuesto

Duracion estimada: 210 min

Mas ancha que F1-18 (120 estimados, 77 reales) y F1-12 (150 / 165) porque la PoC se hace dos veces: el mismo comando en Rust y en TypeScript empaquetado, compilado para varios sistemas y firmado en Windows. Mediana de un ADR de la Fase 1: 45,5 a 50 min (F1-18). **No se trocea.**

### Decisiones ya tomadas por la persona (Carlos, 2026-11-14)

1. **El lenguaje del CLI se elige midiendo Rust frente a TypeScript** (pedido y confirmado por Carlos). PoC del MISMO comando en los dos: TypeScript empaquetado sin Node instalado y Rust. El comando lee las tareas de una copia de ToolsaWeb (404) y de OpenDockTrading (96) con un parser YAML estandar, pinta un tablero y hace una operacion git de lectura. Se mide: tamano, arranque en frio y en caliente, tiempo por comando, tiempo de compilacion, compilacion cruzada, librerias (YAML: `serde_yaml` archivado; GitHub por HTTP) y coste para un agente de escribirlo. Si gana Rust, el ADR propone cambiar F2-1 como divergencia; el plan no se edita. Lo que no se pueda medir va a «No verificado».
2. **Manifiesto de recursos al arrancar**: `init`, `doctor` y la entrada de `start` lo comprueban con un comando determinista: **obligatorio** (git, CLI, skills de fase, version minima: para con el arreglo), **necesario para el proyecto** (lo que declara su config: para antes de `start` si la tarea lo usa), **conveniente** (engram, context7, rtk: avisa y sigue; engram NO es obligatorio) y **estorba** (hooks de terceros sobre las mismas herramientas, skills con nombres que chocan, la 0.16 instalada a la vez). Aqui se fija el contrato; lo implementan F3-8 e `init`.
3. **Nombres aprobados (F1-12, D1a)**: CLI `tc2`, plugin y prefijo `taskcodetwo`, paquete npm `taskcodetwo` con binario `tc2`.
4. **Comandos deterministas sobre hooks** (un hook solo como atajo opcional del mismo comando). Seguridad y trabajo en grupo imprescindibles: firma, hash, cadena de suministro.
5. **Limites**: sin trocear; PoC en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-014\`, borrada al terminar; pilotos solo leidos (copias `git clone --no-hardlinks` en commit fijo); GitHub solo lectura (sin releases, paquetes ni repos). **Ajuste del orquestador**: no se descarga ni instala ninguna toolchain, binario ni imagen Docker nueva (bun, zig, cargo-zigbuild, targets de rustup, imagenes) sin permiso expreso de Carlos; si no lo hay, se usa lo instalado y lo que falte queda «No verificado».

### Herramientas en esta maquina (miradas por ruta, sin ejecutar)

- Encontradas: Node (`C:\Program Files\nodejs`, 22.x segun F1-12); Rust (`~\.cargo\bin`: cargo, rustc, rustup, cargo-audit, cargo-binstall; toolchains `stable` y `1.99.0` x86_64-pc-windows-msvc, solo targets Windows y `wasm32`: sin std de Linux ni macOS); `signtool.exe` (Windows Kits 10.0.26100.0); `gh`; Docker Desktop; `wsl.exe` y `winget.exe` (sin distro Ubuntu encontrada).
- No encontradas: bun, deno, zig, cargo-zigbuild, cosign.
- La implementacion empieza confirmandolo con `where` y `--version` en el log.

### Donde encaja: cinco piezas, tres se extienden y dos se crean

1. **El ejecutable** (se crea), acotado por decisiones previas: git como proceso (F1-4 descarto libgit2); YAML estandar que rechaza claves duplicadas (F1-9, punto 4), comprobado con una tarea copiada y mutada; GitHub por HTTP con JWT de App (F1-6): la PoC hace un GET anonimo a la API para que el tamano incluya la pila TLS.
2. **Como llega el ejecutable con el plugin** (se extiende): hoy el plugin 0.16.1 trae `bin/taskctl` (`#!/usr/bin/env node`), `bin/taskctl.cmd` y `dist/` en JS, y sus hooks lanzan `node`: la 0.16 necesita Node. Propuesta (D3a): lanzador sin runtime (`bin/tc2` en sh, `tc2.cmd`/`.ps1`) y `cli.lock` con version exacta y sha256 por target; descarga comprobada en el primer uso y cache; los hooks llaman a `tc2`. Sin binarios en el repo del marketplace (la PoC mide cuanto pesarian).
3. **Compatibilidad ejecutable–plugin** (se crea): `tc2 version --json` → `{version, protocolo, target, sha256}`; el CLI del plugin fijado por hash; uno instalado aparte se acepta si su `protocolo` cae en el rango del plugin; si no, codigo de precondicion de F1-4 con el arreglo (`tc2 actualizar --cli`, F2-12).
4. **Canales y firma** (se extiende P12 sin cambiar su orden): marketplace principal (RR-16); releases con binarios, `SHA256SUMS` y attestation; instaladores de una linea (F2-11) con hash; npm, winget y tap de Homebrew al ser estable. Forma del paquete npm segun lenguaje (Rust: paquetes por plataforma, nombres a reservar, D5). Firma en D4.
5. **Contrato del manifiesto** (se crea), reutilizando el inventario de F3-9 (RF-14) y anadiendo hooks y ejecutables con version:
   ```
   evaluar_recursos(inv: Inventario, m: Manifiesto, momento: Init | Doctor | Start(tarea)) -> InformeRecursos
   Recurso { id, nivel: obligatorio|necesario|conveniente|estorba, comprobacion, arreglo }
   ```
   Funcion pura; obligatorio, conveniente y reglas de «estorba» en el binario (patron de F1-10); «necesario» derivado de F1-11 (`verificacion`, revisoras, runner de F1-7) mas un bloque opcional `recursos:` que solo anade. Un solo comando `tc2 doctor --recursos [--json]`; `init` y `start` llaman a la misma funcion; hook SessionStart opcional solo lo ejecuta. Salida de 5 lineas como maximo (F1-4). En `start` solo obligatorio y necesario de la tarea.

**El arranque inclina la eleccion**: la status line (RR-19) llama al CLI en cada refresco y la 0.16 ya necesito una via rapida para `estado`. Por eso el arranque en caliente es el primer criterio de D1 y el tamano el segundo.

**Se crea**: `docs/adr/F1-14-distribucion-e-instalacion.md`, su fila en `docs/adr/README.md` y la PoC (`poc-rust/`, `poc-ts/`, guion y log) en la carpeta de la tarea, con los binarios fuera del repo. **No se toca**: ADR F1-1..F1-18, el plan (F2-1, P12, F2-10/11/12 van a «Divergencias»), el plugin 0.16.1 y los pilotos.

### Orden de construccion

1. Inventario de herramientas al log; copias de ToolsaWeb y OpenDockTrading en commit fijo con hash del arbol antes y despues.
2. Especificacion unica del comando: recorre `tareas/**/tarea.md`, YAML estandar que rechaza duplicados, tablero por estado, `git log -1` en la copia, GET anonimo a `api.github.com`; salida identica byte a byte en las dos implementaciones.
3. Dos implementaciones, cada una por un subagente distinto, anotando tokens, minutos, iteraciones de compilar y corregir, lineas y dependencias (con su estado de mantenimiento consultado en solo lectura).
4. Matriz de compilacion: win-x64 nativo en los dos; el resto con lo instalado o con lo que Carlos autorice descargar (ver ajuste); lo que no, «No verificado».
5. Medicion y firma: tamano, arranque en frio (primera ejecucion de copia nueva) y caliente (mediana de 20), tiempo sobre 404 y 96 tareas, compilacion en limpio e incremental; `signtool sign` con un PFX desechable en la carpeta temporal (sin tocar el almacen de certificados) y ejecucion tras firmar. Despues: regla de D1, ADR, indice y divergencias.

### Motivo para no trocear

Consigna de Carlos («NO troceamos ninguna tarea»); lenguaje, empaquetador, canales y firma salen de la misma PoC.

## Lo que el rol no cubrio

- Criterio 1: Windows se ejecuta; Linux y macOS segun lo disponible, y si no «no verificado» (su ejecucion pasa al criterio de F2-10).
- Criterio 2: estructura cubierta; el coste en euros de Authenticode y de Developer ID es decision de negocio (D4).
- Criterio 3: cubiertos RR-16, P12, F1-12 y F1-4/F1-9/F1-6; la desinstalacion (F2-14, F9-10) no se disena aqui.
- Riesgos no enumerados: primer uso sin red, falsos positivos de antivirus con binarios empaquetados, MOTW/Gatekeeper con `irm`/`curl`, secuestro de nombres npm por plataforma, hash en la misma release que el binario (integridad, no autenticidad).
- Testing: solo la PoC; la bateria real es de F2-1 y F2-10.
- Dominio: que cuenta como «la tarea usa» un recurso; que hooks de terceros estorban de verdad; si rtk es conveniente o estorba; umbrales de D1.
- Rendimiento fino: se mide, no se afina.

## Riesgos aceptados y que los contiene

- Sesgo por quien escribe cada version: misma especificacion, salida identica y un subagente por lenguaje con tokens anotados.
- macOS sin ejecutar: marca «no verificado» y criterio de F2-10.
- Arranque en frio aproximado en Windows: definido como primera ejecucion de copia nueva.
- Una sola maquina y una persona: semilla, recalibracion en la CI de F2-10.

## Plan de pruebas

`poc-f1-14.py` (`python -I -B`, log determinista; precedente F1-11/F1-12/F1-18) lee la tabla `cifras` del ADR y sale con 1 si una cifra no es la medida, falta o sobra; copias mutadas del ADR fallan cada una por su motivo; el tiempo de reloj va a stderr.

- **Criterio 1 → 3 pruebas**: (1) salidas Rust y TypeScript identicas sobre 404 y 96 tareas, con recuentos iguales a PyYAML (F1-9), y las dos rechazan la tarea con clave duplicada; (2) los binarios de los targets compilados existen con su sha256 y los ejecutables se ejecutan sin Node en el PATH; (3) hash del arbol de las copias igual antes y despues.
- **Criterio 2 → 2 pruebas**: (1) el `.exe` firmado con el PFX desechable se sigue ejecutando y `signtool verify /pa` informa de la firma (no de confianza, lo esperado); (2) tabla canal → requisito → firma/hash → fase sin huecos.
- **Criterio 3 → 1 prueba**: tabla requisito → decision → evidencia (RR-16, P12, F1-4, F1-6, F1-9, F1-12, F2-1, F2-10) con IDs comprobados y la fila del indice con enlace que resuelve; al cerrar, `taskctl doctor` sin errores.

## Lo que necesita decision de una persona

Respuestas (Carlos, consigna «opcion recomendada»): **D1a, D2a, D3a, D4a, D5a, D6a, D7a, D8a, D9a**, con el ajuste de descargas del punto 5.

- **D1. Regla para elegir lenguaje**: **(a, recomendada)** regla declarada antes de medir: 1º arranque en caliente, 2º tamano, 3º compilacion cruzada reproducible, 4º librerias mantenidas que cumplan F1-9, 5º coste del agente; gana quien gane el 1º y al menos dos mas; (b) TypeScript siempre; (c) Rust directamente.
- **D2. Empaquetador de TypeScript**: **(a, recomendada)** bun compile (si Carlos autoriza descargarlo) mas Node SEA en Windows como referencia; sin descarga, solo Node SEA en Windows; (b) Node SEA en los tres. Descartado deno compile (misma forma que bun).
- **D3. Como llega el CLI con el plugin**: **(a, recomendada)** lanzador sin runtime + `cli.lock` + descarga comprobada en el primer uso; (b) binarios en el repo del marketplace; (c) instalar el CLI aparte.
- **D4. Firma**: **(a, recomendada)** desde la primera release `SHA256SUMS`, attestation de GitHub (Sigstore) y firma ad hoc en macOS; Authenticode y Developer ID con notarizacion al entrar en winget y Homebrew (el ADR pone el coste anual); (b) todo firmado desde el primer dia; (c) solo hash.
- **D5. Canales y nombres npm**: **(a, recomendada)** marketplace, releases e instaladores de una linea en la 2.0; npm `taskcodetwo` (paquetes por plataforma si gana Rust), winget y tap de Homebrew al ser estable; (b) todos desde la primera release.
- **D6. Donde vive el manifiesto**: **(a, recomendada)** suelo en el binario, «necesario» derivado de F1-11 mas `recursos:` que solo anade, con `tc2 doctor --recursos`; (b) fichero en el plugin; (c) todo en la config.
- **D7. Que hace «estorba»**: **(a, recomendada)** avisa en `doctor`; para `start` solo si un hook de terceros intercepta herramientas que la fase usa (Bash/PowerShell sobre `git`/`tc2`) o si la 0.16 escribe en el mismo proyecto; (b) solo avisar.
- **D8. Ejecucion fuera de Windows**: **(a, recomendada)** Linux en un contenedor Docker limpio si hay imagen disponible o Carlos autoriza bajarla (borrada al terminar); macOS compilado si se puede, «no verificado»; (b) workflow puntual en GitHub (rompe solo lectura; exigiria permiso expreso).
- **D9. Divergencia en F2-1 y P12**: **(a, recomendada)** el ADR la anota y propone el texto; (b) editar el plan aqui.

## Suposiciones no verificadas

- Versiones y PATH reales (confirmar con `where` y `--version`).
- Estado de las librerias YAML de Rust (`serde_yml`, `serde_norway`, `saphyr`, `serde-saphyr`) y si `yaml` de npm rechaza duplicados por defecto: mirar en crates.io y npm y probar con la tarea mutada.
- Compilacion cruzada de Rust a macOS desde Windows (requiere herramientas que no estan).
- Que Docker Desktop arranque y tenga imagenes utilizables.
- Coste y disponibilidad de Azure Artifact Signing para una persona en Espana, y si winget exige binarios firmados (documentacion oficial en solo lectura).

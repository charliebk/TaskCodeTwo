# Peticion de revision — TASK-014 (ronda 1)

- Tarea: TASK-014 — F1-14 ADR distribucion e instalacion
- Rama revisada: feature/task-014-f1-14-adr-distribucion-e-instalacion
- Rama base: develop
- Commit revisado (HEAD): 36828cc4bd191733e0f3aa8fcc8b29c1c6f0ac7e
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-014 (criterios de aceptacion y plan)
- Alcance: completa (motivo: primera ronda de revision)

## Instrucciones para el agente revisor

Eres un revisor INDEPENDIENTE del agente que implemento. Tu trabajo es
reproducir empiricamente, no leer el diff y opinar: clona el repo a un
directorio temporal, corre la suite tu mismo (salvo que esta
peticion ya traiga la verificacion de la ronda en verde para el commit revisado;
entonces vale ese resultado) y construye el caso que
rompe el codigo antes de reportarlo. Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. Vuelca tu salida en el
informe de esta ronda (informe-revision-1.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-014/tarea.md
- tareas/03-en-revision/TASK-014/planificacion/
- docs/adr/F1-14-distribucion-e-instalacion.md
- docs/adr/README.md

## Commits a revisar (git log develop..HEAD)

````
36828cc chore(TASK-014): autorrevision
10027fd docs(adr): F1-14 distribucion e instalacion, Rust frente a TypeScript medido, con PoC
3425a08 chore(TASK-014): tarea en curso
````

## Diff completo (git diff develop..HEAD)

````diff
diff --git a/docs/adr/F1-14-distribucion-e-instalacion.md b/docs/adr/F1-14-distribucion-e-instalacion.md
new file mode 100644
index 0000000..dc83e35
--- /dev/null
+++ b/docs/adr/F1-14-distribucion-e-instalacion.md
@@ -0,0 +1,647 @@
+# ADR F1-14 — Distribución e instalación: CLI en Rust medido frente a TypeScript, lanzador sin runtime con `cli.lock` y descarga comprobada, compatibilidad por protocolo, canales y firma por etapas, y contrato del manifiesto de recursos
+
+- Estado: **aceptada** (2026-11-14)
+- Tarea: TASK-014 · Plan: fila F1-14 del [PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) y su apartado P12
+- Requisitos: RR-16 (instalación fácil en cualquier ordenador) y RR-19 (progreso en vivo: la status line llama al CLI en cada refresco) de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); RR-12 (equipo) para la cadena de suministro; RF-14 (inventario de skills y agentes) para el manifiesto
+- Depende de: [ADR F1-4](F1-4-operaciones-git-y-contrato-de-salida.md) (git como proceso, contrato de salida y códigos), [ADR F1-6](F1-6-identidades-agente-revisor-persona.md) (GitHub por HTTP con JWT de App: el binario lleva TLS), [ADR F1-8](F1-8-seguridad-modelo-de-amenazas.md) (el CLI en la zona Z1 de la máquina de la persona), [ADR F1-9](F1-9-formato-de-tarea-plan-y-especificacion.md) (YAML estándar que rechaza claves duplicadas), [ADR F1-10](F1-10-carriles-y-modo-automatico.md) (suelo en el binario que la config solo endurece), [ADR F1-11](F1-11-extensiones-declaradas.md) (decisión 10: comandos sí, hooks solo como atajo opcional; `verificacion` y revisoras) y [ADR F1-12](F1-12-actualizacion-entre-versiones.md) (nombres `tc2` y `taskcodetwo`, npm `taskcodetwo` libre, convivencia con la 0.16)
+- Decide: Carlos, con sus decisiones del 2026-11-14 (medir Rust frente a TypeScript con el mismo comando; manifiesto de recursos con cuatro niveles y engram no obligatorio; nombres de F1-12; comandos deterministas sobre hooks; no trocear; no descargar toolchains, targets ni imágenes: «haz la comparación con lo que ya está instalado») y la consigna «la opción recomendada» (D1a a D9a del plan de TASK-014)
+
+## Contexto
+
+RR-16 pide que TaskCodeTwo se instale en un minuto en cualquier ordenador, y
+P12 lo concreta: el repo público es su propio marketplace de Claude Code, el
+plugin trae el CLI ya compilado y, como complemento, hay un ejecutable por
+sistema (Windows x64/ARM64, macOS Intel/Apple, Linux x64/ARM64) sin Node,
+instaladores de una línea con hash o firma y, cuando sea estable, winget,
+Homebrew y npm.
+
+Hoy la 0.16.1 no cumple eso: su plugin trae `bin/taskctl` con
+`#!/usr/bin/env node` y `bin/taskctl.cmd`, y los cinco hooks de
+`hooks/hooks.json` lanzan `node` (observado en la caché del plugin el
+2026-11-14). Sin Node instalado no funciona nada.
+
+Faltaban cinco decisiones que esta tarea toma con datos: **en qué lenguaje
+se escribe el CLI** (F2-1 dice TypeScript; Carlos pidió medirlo contra
+Rust), **cómo llega el binario con el plugin**, **cómo se sabe que el
+ejecutable y el plugin son compatibles**, **por qué canales y con qué
+firma**, y el **contrato del manifiesto de recursos** que `init`, `doctor` y
+`start` comprueban al arrancar.
+
+La status line (RR-19, F3-12) llama al CLI en cada refresco y la 0.16 ya
+necesitó una vía rápida para `estado`: por eso el arranque en caliente es el
+primer criterio de la regla D1a, declarada antes de medir.
+
+## Alternativas
+
+| | Descripción | Veredicto |
+|---|---|---|
+| **L1** | CLI en **Rust**, binario nativo por target | **Elegida** por la regla D1a (PoC: gana el criterio 1 y otros dos; ver «Decisión», punto 1) |
+| L2 | CLI en TypeScript empaquetado con Node SEA (o bun compile) | Descartada: pierde el arranque en caliente en los 3 repos y pesa 32,1 veces más; su cadena de empaquetado depende de postject en alfa |
+| **D1a** | Regla declarada antes de medir: 1.º arranque en caliente, 2.º tamaño, 3.º compilación cruzada reproducible, 4.º librerías mantenidas que cumplan F1-9, 5.º coste del agente; gana quien gane el 1.º y al menos dos más | **Elegida** (consigna) |
+| D1b / D1c | TypeScript siempre / Rust directamente | Descartadas: decidir sin medir |
+| **D2a** | bun compile si se autorizaba descargarlo, Node SEA en Windows como referencia; sin descarga, solo Node SEA en Windows | **Elegida** en su rama «sin descarga»: bun no está instalado y no se descargó |
+| **D3a** | Lanzador sin runtime en el plugin + `cli.lock` (versión exacta y sha256 por target) + descarga comprobada en el primer uso, con caché | **Elegida** |
+| D3b | Binarios dentro del repo del marketplace | Descartada: con Rust, 6 targets serían 15,6 MiB por versión (estimado) que el historial de git guarda para siempre; con TypeScript, 500,3 MiB por versión y cada `.exe` por encima del aviso de 50 MiB de GitHub (PoC M) |
+| D3c | Instalar el CLI aparte, sin el plugin | Descartada: rompe «el plugin trae el CLI» de P12 |
+| **D4a** | Desde la primera release: `SHA256SUMS`, attestation de GitHub (Sigstore) y firma ad hoc en macOS; Authenticode y Developer ID con notarización al entrar en winget y Homebrew | **Elegida**; Authenticode probado con un certificado desechable (PoC F) |
+| D4b / D4c | Todo firmado desde el primer día / solo hash | Descartadas: D4b paga certificados antes de tener canales que los pidan (y Azure Artifact Signing no admite personas físicas fuera de EE. UU. y Canadá, ver «Decisión», punto 6); D4c no da procedencia |
+| **D5a** | Marketplace, releases e instaladores de una línea en la 2.0; npm `taskcodetwo` (con paquetes por plataforma, porque gana Rust), winget y tap de Homebrew al ser estable | **Elegida** |
+| D5b | Todos los canales desde la primera release | Descartada: winget y Homebrew piden mantenimiento por versión antes de que haya versiones estables |
+| **D6a** | Manifiesto: suelo en el binario; «necesario» derivado de F1-11 más un bloque `recursos:` que solo añade; un solo comando `tc2 doctor --recursos` | **Elegida** |
+| D6b / D6c | Fichero en el plugin / todo en la config | Descartadas: D6b separa el suelo del binario que lo aplica; D6c deja quitar lo obligatorio |
+| **D7a** | «Estorba» avisa en `doctor`; para `start` solo si un hook de terceros intercepta herramientas que la fase usa o si la 0.16 escribe en el mismo proyecto | **Elegida** |
+| D7b | Solo avisar | Descartada: un hook que reescribe `git` o `tc2`, o la 0.16 escribiendo en el mismo proyecto (la convivencia que F1-12 limita), cambian lo que hace la fase; un aviso no lo evita |
+| **D8a** | Linux en un contenedor Docker limpio si hay imagen; macOS compilado si se puede; si no, «no verificado» | **Elegida**; resultado: los dos «no verificado» (daemon de Docker sin respuesta, WSL sin distribución de usuario, sin targets de Rust para Linux ni macOS) |
+| D8b | Workflow puntual en GitHub | Descartada: rompe «GitHub solo lectura» de la tarea |
+| **D9a** | El ADR anota la divergencia con F2-1 y P12 y propone el texto | **Elegida**: el plan no se edita aquí |
+
+## Pruebas de concepto
+
+Guion `poc-f1-14.py` y su salida `poc-f1-14.log`, en la carpeta `poc/` de
+TASK-014 (sin enlace: la carpeta de la tarea cambia de estado al cerrarse),
+junto a la especificación del comando (`especificacion-comando.md`) y las dos
+implementaciones con sus notas: `poc-rust/` (`NOTAS.md`, `DEPENDENCIAS.md`)
+y `poc-ts/` (ídem). Python 3.12.10 (`python -I -B`), PyYAML 6.0.3,
+Git 2.56.0, Rust 1.100.0 y Node 22.23.4. Carpeta de trabajo
+`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-014\medicion\` con marca y
+guarda (el guion se niega a borrar una carpeta sin su marca o fuera de
+`pruebas-claude/<proyecto>/<motivo>`), borrada al empezar y al terminar.
+
+```bash
+python -I -B tareas/<estado>/TASK-014/poc/poc-f1-14.py \
+  "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-014/medicion" . .. > poc-f1-14.log
+```
+
+- **Mismo comando, dos lenguajes**: `tablero <raíz> [--sin-red]` recorre
+  `tareas/**/tarea.md`, lee el frontmatter con una librería YAML estándar que
+  rechaza claves duplicadas (F1-9), pinta un tablero por estado, lanza
+  `git log -1` como proceso hijo (F1-4) y hace un GET anónimo HTTPS a
+  `api.github.com/zen` para que el tamaño incluya la pila TLS (F1-6). Cada
+  versión la escribió un subagente distinto; las dos se compilan desde las
+  fuentes del repo dentro de la carpeta de trabajo. El `.exe` de Rust que se
+  mide es el compilado con `-C link-arg=/Brepro`: el de por defecto cambia de
+  bytes entre builds (en una pasada, su tamaño con gzip varió en 1 byte) y
+  `/Brepro` es el perfil que fija la decisión.
+- **Alineación previa**: las dos primeras versiones divergían en casos
+  límite (código 1 frente a 2 en errores propios, `tareas/` ausente, 10 s
+  frente a 15 s de timeout, escalares como `1.50` leídos como `1.5`, claves de
+  fusión). La de TypeScript se alineó con las interpretaciones de la de Rust
+  en tres iteraciones extra y quedó igual byte a byte y en código de salida en
+  11 casos límite (juguete, `casos`, 27 `tarea.md` límite, sin `tareas/`,
+  `tareas` como fichero, raíz inexistente, sin argumentos y opciones raras),
+  anotado en `poc-ts/NOTAS.md`.
+- **Datos fijos**: copias `git clone --no-hardlinks` de ToolsaWeb en
+  `develop` `8bebfd049` y de OpenDockTrading en `develop` `8141a08a7` (el de
+  F1-12 y F1-18). Los originales solo se leen; el hash del árbol de las
+  copias, con `.git`, se compara antes y después.
+- **Nada descargado**: sin bun, zig, targets de rustup ni imágenes Docker. La
+  red solo se usa para `npm ci` (dependencias fijadas), para bajar de
+  crates.io las crates propias de Linux y macOS en el intento de compilación
+  cruzada (con `Cargo.lock` fijado) y para el GET del propio comando, fuera
+  de la tabla comprobada.
+- **Determinismo**: dos ejecuciones completas del guion, cada una con sus
+  propias builds y copias, dieron el mismo log salvo la sección T.
+- **Comprobación del ADR (G)**: el guion lee la tabla `cifras` y sale con 1
+  si una cifra no es la medida, falta o sobra, o si su fuente no es la
+  sección que la mide; también el `resumen`, las secciones, el estado, las
+  reglas obligatorias, el ganador de D1 en la decisión, los IDs (en el plan,
+  en los requisitos o con ADR propio), los enlaces y la fila del índice.
+  Catorce copias mutadas del ADR fallan cada una por su motivo.
+- **Tiempos de reloj**: van al log en su sección T y no son cifras
+  comprobadas; en la tabla solo entran las relaciones estables («Rust más
+  rápido en caliente en los 3 repos»).
+
+Salida observada, recortada (la completa está en el log):
+
+### I — Inventario de herramientas
+
+```text
+node     encontrado     v22.23.4
+cargo    encontrado     cargo 1.100.0 (3acc79c0c 2026-11-05)
+rustc    encontrado     rustc 1.100.0 (0ef1bc386 2026-11-09)
+openssl  encontrado     OpenSSL 3.5.9 29 Sep 2026 (Library: OpenSSL 3.5.9 29 Sep 2026)
+docker   encontrado     Docker version 29.9.2, build a82e528
+bun      NO encontrado  no encontrado
+zig      NO encontrado  no encontrado
+signtool encontrado     C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe (no esta en el PATH)
+rustup targets instalados: wasm32-unknown-unknown, x86_64-pc-windows-msvc
+docker daemon: no responde
+wsl distribuciones: docker-desktop
+```
+
+### B y C — Builds y copias
+
+```text
+Rust: dos builds en limpio: sha256 distinto ; tamano 2720768 bytes
+TypeScript: dos builds en limpio: sha256 igual ; tamano 87430656 bytes
+Rust con -C link-arg=/Brepro: dos builds en limpio: sha256 igual ; tamano 2720768 bytes (es el .exe de Rust que se mide desde aqui)
+artefactos de build dentro del repo: 0
+ToolsaWeb: commit 8bebfd049 ; ficheros (con .git) 4579
+OpenDockTrading: commit 8141a08a7 ; ficheros (con .git) 5326
+...
+ToolsaWeb: hash del arbol (con .git) antes y despues: igual
+OpenDockTrading: hash del arbol (con .git) antes y despues: igual
+```
+
+### S — Salidas, contraste con PyYAML, clave duplicada y sin Node
+
+```text
+ToolsaWeb: codigos 0 / 0 ; stdout identico byte a byte: si ; tareas 404 ; errores 0 ; estados en-diseno 1, planificada 241, terminada 162
+   PyYAML (cargador estricto): validas 404 ; errores 0 ; coincide con los binarios: si
+OpenDockTrading: codigos 0 / 0 ; stdout identico byte a byte: si ; tareas 96 ; errores 0 ; estados planificada 52, terminada 44
+   PyYAML (cargador estricto): validas 96 ; errores 0 ; coincide con los binarios: si
+repo vacio: codigos 0 / 0 ; stdout identico byte a byte: si ; tareas 0 ; errores 0 ; estados -
+clave duplicada (estado repetido en tareas/00-planificadas/TASK-021/tarea.md de OpenDockTrading): Rust la rechaza si ; TypeScript la rechaza si ; PyYAML safe_load la acepta en silencio: si
+sin node en el PATH (where node vacio: si) ni NODE_*: Rust igual si ; TypeScript igual si
+```
+
+Lectura: los recuentos son los de F1-9 (404 y 96) y coinciden tarea a tarea
+con un lector independiente en PyYAML con cargador estricto. PyYAML por
+defecto acepta la clave duplicada; los dos binarios la rechazan.
+
+### F — Firma con certificado desechable
+
+```text
+Rust: sin firma antes: verify falla ; sign codigo 0 ; ejecucion tras firmar igual: si ; verify /pa: rechazada: raiz no de confianza ; firmante visto: si
+TypeScript: sin firma antes: verify falla ; sign codigo 0 ; ejecucion tras firmar igual: si ; verify /pa: rechazada: raiz no de confianza ; firmante visto: si
+almacen de certificados del usuario (My, Root, CA, TrustedPublisher) sin cambios: si
+```
+
+### X — Compilación cruzada y ejecución fuera de Windows
+
+```text
+Rust --target x86_64-unknown-linux-gnu: no compila: target no instalado (sin la std de destino)
+Rust --target aarch64-apple-darwin: no compila: target no instalado (sin la std de destino)
+Rust --target aarch64-pc-windows-msvc: no compila: target no instalado (sin la std de destino)
+TypeScript (Node SEA) para Linux o macOS: hace falta el node de ese sistema como base; no hay ninguno instalado y no se descarga
+ejecucion en Linux: no verificado: daemon de Docker sin respuesta y WSL sin distribucion de usuario
+ejecucion en macOS: no verificado (no hay macOS)
+```
+
+### M y D — Tamaños, marketplace y dependencias
+
+```text
+Rust: .exe win-x64 2720768 bytes (2,6 MiB) ; gzip -9 1344575 bytes (1,3 MiB)
+   por version (6 targets, estimado con win-x64): 15,6 MiB ; 10 versiones: 155,7 MiB ; frente a 50 / 100 MiB de GitHub: por debajo / por debajo
+TypeScript: .exe win-x64 87430656 bytes (83,4 MiB) ; gzip -9 33103429 bytes (31,6 MiB)
+   por version (6 targets, estimado con win-x64): 500,3 MiB ; 10 versiones: 5002,8 MiB ; frente a 50 / 100 MiB de GitHub: por encima / por debajo
+TypeScript / Rust en tamano: 32.1 veces
+Rust: dependencias directas 3 (serde, serde-saphyr, ureq) ; paquetes en Cargo.lock 90
+TypeScript: dependencias de runtime 1 (yaml) ; paquetes en package-lock.json 53 ; postject 1.0.0-alpha.6
+lineas de main.rs 175 (147 sin blancos ni comentarios de linea) ; main.ts 243 (215) ; build.mjs 105 (89)
+```
+
+Lectura: el `.exe` de TypeScript es casi entero el `node.exe` de Node 22
+(el bundle JS son 262 KB, `poc-ts/NOTAS.md`). El peso por versión es una
+estimación: multiplica el tamaño de win-x64 por los 6 targets de P12,
+porque los demás no se han podido compilar. Los 53 paquetes de
+`package-lock.json` incluyen los binarios opcionales de esbuild y TypeScript
+de otras plataformas; en `node_modules` quedan 9 y en el binario 1 (`yaml`).
+
+### T — Tiempos de reloj (no comprobados; una pasada en esta máquina)
+
+```text
+Rust sobre repo vacio: frio (1.a ejecucion de copia nueva) 87 ms ; caliente mediana de 20 49 ms (min 47, max 52)
+Rust sobre ToolsaWeb: frio (1.a ejecucion de copia nueva) 400 ms ; caliente mediana de 20 354 ms (min 346, max 377)
+Rust sobre OpenDockTrading: frio (1.a ejecucion de copia nueva) 175 ms ; caliente mediana de 20 132 ms (min 122, max 179)
+TypeScript sobre repo vacio: frio (1.a ejecucion de copia nueva) 1180 ms ; caliente mediana de 20 134 ms (min 132, max 154)
+TypeScript sobre ToolsaWeb: frio (1.a ejecucion de copia nueva) 1753 ms ; caliente mediana de 20 671 ms (min 661, max 751)
+TypeScript sobre OpenDockTrading: frio (1.a ejecucion de copia nueva) 1333 ms ; caliente mediana de 20 308 ms (min 299, max 337)
+build en limpio Rust: 38.0 s ; TypeScript (npm ci + tsc + esbuild + SEA + postject): 17.2 s
+```
+
+Lectura: son cifras de una pasada y cambian entre ejecuciones (por eso no
+entran en la tabla, y las del log final pueden diferir); lo que se repitió
+en todas las pasadas completas del guion es la relación: Rust más rápido en
+caliente y en frío en los 3 repos. En caliente, sin tareas, la diferencia
+es el arranque de Node (unos 85 ms en esta pasada); con 404 tareas, Rust
+tarda alrededor de la mitad. El frío de TypeScript pasa del segundo en los
+3 repos (copia nueva de un `.exe` de 83 MiB, con el antivirus de por medio).
+
+### D1 — Regla aplicada
+
+```text
+criterio 1 arranque en caliente: Rust
+criterio 2 tamano: Rust
+criterio 3 compilacion cruzada reproducible: ninguno
+criterio 4 librerias mantenidas que cumplan F1-9: Rust
+criterio 5 coste del agente: ninguno
+gana: Rust (el 1.o y al menos dos mas)
+```
+
+### G — El ADR y sus mutaciones
+
+Las catorce copias mutadas (una cifra cambiada, una fila borrada, una fila
+sin medida, un tiempo de reloj en la tabla, una fuente cambiada, un número
+inventado en el resumen, un ID que no existe, un enlace a `tareas/`, un
+enlace roto, una sección borrada, el estado «propuesta», «engram es
+obligatorio», el ganador de D1 cambiado y «No verificado» sin macOS) fallan
+cada una por su motivo (log, sección G).
+
+### Cifras de la PoC
+
+Cada fila es una medida del guion, con la sección que la produce. El guion
+falla si un valor no es el medido, si falta una medida, si sobra una fila o
+si la fuente no es la sección que la mide.
+
+<!-- cifras:inicio -->
+| Cifra | Valor | Fuente |
+|---|---|---|
+| I: bun / zig / deno encontrados | `no / no / no` | I |
+| I: distribuciones WSL | `docker-desktop` | I |
+| I: docker daemon | `no responde` | I |
+| I: node | `v22.23.4` | I |
+| I: openssl (de Git for Windows) | `OpenSSL 3.5.9 29 Sep 2026 (Library: OpenSSL 3.5.9 29 Sep 2026)` | I |
+| I: rustc | `rustc 1.100.0 (0ef1bc386 2026-11-09)` | I |
+| I: rustup targets instalados | `wasm32-unknown-unknown, x86_64-pc-windows-msvc` | I |
+| I: signtool (Windows SDK 10.0.26100.0) | `si` | I |
+| B: Rust con -C link-arg=/Brepro, dos builds en limpio (win-x64): sha256 | `igual` | B |
+| B: Rust, dos builds en limpio (win-x64): sha256 | `distinto` | B |
+| B: TypeScript, dos builds en limpio (win-x64): sha256 | `igual` | B |
+| B: artefactos de build dentro del repo | `0` | B |
+| C: OpenDockTrading: commit fijo | `8141a08a7` | C |
+| C: OpenDockTrading: hash del arbol (con .git) antes / despues | `igual` | C |
+| C: ToolsaWeb: commit fijo | `8bebfd049` | C |
+| C: ToolsaWeb: hash del arbol (con .git) antes / despues | `igual` | C |
+| S: OpenDockTrading: estados | `planificada 52, terminada 44` | S |
+| S: OpenDockTrading: recuento igual al de PyYAML | `si` | S |
+| S: OpenDockTrading: salida Rust y TypeScript identica (stdout y codigo) | `si` | S |
+| S: OpenDockTrading: tareas validas / errores | `96 / 0` | S |
+| S: ToolsaWeb: estados | `en-diseno 1, planificada 241, terminada 162` | S |
+| S: ToolsaWeb: recuento igual al de PyYAML | `si` | S |
+| S: ToolsaWeb: salida Rust y TypeScript identica (stdout y codigo) | `si` | S |
+| S: ToolsaWeb: tareas validas / errores | `404 / 0` | S |
+| S: clave duplicada en una tarea real: rechazada por Rust / TypeScript | `si / si` | S |
+| S: clave duplicada: PyYAML safe_load la acepta en silencio | `si` | S |
+| S: repo vacio (sin tareas/): tareas / errores | `0 / 0` | S |
+| S: repo vacio: salida Rust y TypeScript identica (stdout y codigo) | `si` | S |
+| S: sin node en el PATH: Rust / TypeScript dan la misma salida | `si / si` | S |
+| F: Rust: se ejecuta igual tras firmar | `si` | F |
+| F: Rust: signtool sign con el PFX desechable | `codigo 0` | F |
+| F: Rust: signtool verify /pa | `rechazada: raiz no de confianza` | F |
+| F: TypeScript: se ejecuta igual tras firmar | `si` | F |
+| F: TypeScript: signtool sign con el PFX desechable | `codigo 0` | F |
+| F: TypeScript: signtool verify /pa | `rechazada: raiz no de confianza` | F |
+| F: almacen de certificados del usuario sin cambios | `si` | F |
+| X: Rust --target aarch64-apple-darwin | `no compila: target no instalado (sin la std de destino)` | X |
+| X: Rust --target aarch64-pc-windows-msvc | `no compila: target no instalado (sin la std de destino)` | X |
+| X: Rust --target x86_64-unknown-linux-gnu | `no compila: target no instalado (sin la std de destino)` | X |
+| X: TypeScript para Linux o macOS (Node SEA o bun) | `no verificado: falta el node de destino y bun` | X |
+| X: ejecucion en Linux | `no verificado: daemon de Docker sin respuesta y WSL sin distribucion de usuario` | X |
+| X: ejecucion en macOS | `no verificado: no hay macOS` | X |
+| M: Rust: .exe frente al aviso de 50 MiB / bloqueo de 100 MiB de GitHub | `por debajo / por debajo` | M |
+| M: Rust: .exe win-x64 comprimido con gzip -9 (bytes) | `1344575` | M |
+| M: Rust: 10 versiones en el repo del marketplace, estimado (MiB) | `155,7` | M |
+| M: Rust: por version con 6 targets de P12, estimado con win-x64 (MiB) | `15,6` | M |
+| M: Rust: tamano del .exe win-x64 (bytes) | `2720768` | M |
+| M: TypeScript: .exe frente al aviso de 50 MiB / bloqueo de 100 MiB de GitHub | `por encima / por debajo` | M |
+| M: TypeScript: .exe win-x64 comprimido con gzip -9 (bytes) | `33103429` | M |
+| M: TypeScript: 10 versiones en el repo del marketplace, estimado (MiB) | `5002,8` | M |
+| M: TypeScript: por version con 6 targets de P12, estimado con win-x64 (MiB) | `500,3` | M |
+| M: TypeScript: tamano del .exe win-x64 (bytes) | `87430656` | M |
+| M: tamano TypeScript / Rust (veces) | `32,1` | M |
+| M: targets de P12 | `win-x64, win-arm64, darwin-x64, darwin-arm64, linux-x64, linux-arm64` | M |
+| D: Rust: dependencias directas | `3 (serde, serde-saphyr, ureq)` | D |
+| D: Rust: paquetes en Cargo.lock | `90` | D |
+| D: TypeScript: dependencias de runtime | `1 (yaml)` | D |
+| D: TypeScript: paquetes en package-lock.json | `53` | D |
+| D: TypeScript: version de postject | `1.0.0-alpha.6` | D |
+| D: lineas de build.mjs (sin blancos ni comentarios de linea) | `89` | D |
+| D: lineas de main.rs / main.ts (sin blancos ni comentarios de linea) | `147 / 215` | D |
+| T: Rust mas rapido en caliente (mediana de 20) en los 3 repos | `si` | T |
+| T: Rust mas rapido en frio en los 3 repos | `si` | T |
+| D1: criterio 1 arranque en caliente | `Rust` | D1 |
+| D1: criterio 2 tamano | `Rust` | D1 |
+| D1: criterio 3 compilacion cruzada reproducible | `ninguno` | D1 |
+| D1: criterio 4 librerias mantenidas que cumplan F1-9 | `Rust` | D1 |
+| D1: criterio 5 coste del agente | `ninguno` | D1 |
+| D1: criterios ganados por el ganador del 1.o sin contar el 4 ni el 5 | `1` | D1 |
+| D1: ganador (el 1.o y al menos dos mas) | `Rust` | D1 |
+<!-- cifras:fin -->
+
+<!-- resumen:inicio -->
+Qué deja medido este ADR: el mismo comando en Rust y en TypeScript con Node
+SEA da la misma salida byte a byte sobre **404 / 0** tareas de ToolsaWeb y
+**96 / 0** de OpenDockTrading, igual que PyYAML, y los dos rechazan la clave
+duplicada. Rust es más rápido en caliente y en frío en los 3 repos y su
+`.exe` pesa **2720768** bytes frente a **87430656** (**32,1** veces menos);
+meter los binarios en el repo del marketplace costaría, estimado, **15,6**
+MiB por versión en Rust y **500,3** en TypeScript. La regla D1a da
+**Rust** (criterios 1, 2 y 4); sin el 4, no decidiría. Los dos se firman con
+un certificado desechable y siguen funcionando; el almacén de certificados
+no cambia. Dónde no llega: Linux, macOS y Windows ARM64 no se compilaron ni
+se ejecutaron, y los importes de Authenticode no se pudieron consultar.
+<!-- resumen:fin -->
+
+## Cómo se ve en GitHub
+
+- **Releases** con un binario por target, `SHA256SUMS` y attestation de
+  procedencia (`gh attestation verify`); los instaladores de una línea y el
+  lanzador del plugin comprueban el sha256 de `cli.lock` antes de ejecutar.
+- **Repo del marketplace sin binarios**: el plugin solo lleva el lanzador y
+  `cli.lock`; un PR que cambia de versión del CLI cambia una línea por target
+  en `cli.lock`, revisable a ojo.
+- **npm** `taskcodetwo` con el binario `tc2` y paquetes por plataforma con
+  nombres reservados antes de publicar (ver «Decisión», punto 7).
+- Nada de esto se ha visto en GitHub: la tarea es de solo lectura (ver «No
+  verificado»).
+
+## Decisión
+
+1. **Lenguaje del CLI: Rust** (L1), por la regla D1a aplicada sin
+   retocarla a los datos (PoC D1):
+
+   | Criterio (orden declarado antes de medir) | Gana | Evidencia |
+   |---|---|---|
+   | 1.º arranque en caliente | **Rust** | más rápido en caliente (mediana de 20) en los 3 repos y en frío en los 3 (T; los milisegundos son de reloj y están en el log, no en la tabla) |
+   | 2.º tamaño | **Rust** | 2 720 768 frente a 87 430 656 bytes en win-x64, 32,1 veces menos (M) |
+   | 3.º compilación cruzada reproducible | ninguno | ninguno de los dos compila para Linux ni macOS con lo instalado (X); en win-x64, dos builds en limpio dan el mismo sha256 en TypeScript, y en Rust solo con `/Brepro` (B) |
+   | 4.º librerías mantenidas que cumplan F1-9 | **Rust** | las dos librerías YAML están mantenidas y rechazan la clave duplicada en una tarea real (S); la cadena de TypeScript depende de postject `1.0.0-alpha.6`, sin publicar desde 2023-05-19 (D, `poc-ts/DEPENDENCIAS.md`) |
+   | 5.º coste del agente | ninguno | ~8 frente a ~9 min de reloj y 0 errores de compilación en los dos (`NOTAS.md`); los tokens de cada implementación no se anotaron |
+
+   Rust gana el 1.º y otros dos (2.º y 4.º): la regla se cumple.
+   **Sensibilidad**: el 4.º se decide por una herramienta de build, no de
+   runtime. Si se descuenta postject (por ejemplo, porque un Node posterior
+   traiga `--build-sea`, no probado), el 4.º queda en empate, Rust gana solo
+   el 1.º y el 2.º y la regla **no decide**. La decisión se sostiene entonces
+   en lo que la regla pone primero y en el margen: Rust es más rápido en los
+   3 repos y en frío y caliente, y 32,1 veces más pequeño. Lo que la PoC
+   deja además, en contra y a favor de Rust, sin cambiar el resultado:
+   - En contra, **build no reproducible por defecto** en MSVC (sha256
+     distinto en dos builds en limpio). Con `-C link-arg=/Brepro`, que
+     sustituye la marca de tiempo del enlazador por un hash del contenido,
+     dos builds en limpio dan el mismo sha256 (PoC B): F2-1 lo fija y lo
+     comprueba en su CI.
+   - En contra, **ecosistema YAML movedizo**: `serde_yaml` archivado, `serde_yml`
+     marcado DEPRECATED, `serde_norway` sin publicar desde 2024-12;
+     `serde-saphyr` (2.1.1, 2026-11-05) está activo y rechaza duplicadas por
+     defecto, pero es joven. El YAML vive detrás de un módulo propio con la
+     batería de F2-5, para poder cambiar de crate sin tocar el resto.
+   - En contra, **`ring` compila C/asm** y necesita MSVC Build Tools para compilar
+     desde fuente en Windows (y un compilador C del destino para cruzar):
+     afecta a la CI y a quien compile, no al binario distribuido.
+   - A favor, **menos código propio para el mismo comportamiento**: con serde, el
+     esquema estricto (campos obligatorios, escalares) sale de un `struct`;
+     TypeScript necesitó una función propia para igualarlo
+     (147 frente a 215 líneas, D).
+
+2. **TLS con el almacén del sistema.** Un binario de escritorio verifica
+   certificados contra el **almacén del sistema** (Windows, llavero de
+   macOS, el del sistema en Linux), no solo con raíces embebidas. Medido en
+   las dos implementaciones: en esta máquina hay una CA local (inspección
+   TLS); Rust con las raíces de Mozilla de `webpki-roots` daba `red: no` y
+   con `rustls-platform-verifier` da `red: si`; el `fetch` de Node daba
+   `red: no` sin las variables `NODE_USE_SYSTEM_CA` / `NODE_EXTRA_CA_CERTS`
+   que define el entorno de la app, y `red: si` tras añadir las raíces
+   `system` en el código (T, informativo; `NOTAS.md` de cada PoC). En un
+   equipo corporativo, el CLI sin esto no hablaría con GitHub.
+
+3. **Empaquetado.** `cargo build --release` por target en la CI de F2-10,
+   en un runner nativo de cada sistema (la compilación cruzada desde Windows
+   no se ha podido verificar); perfil `opt-level = 3`, `lto = "thin"`,
+   `codegen-units = 1`, `strip = true`, `panic = "abort"`, y en Windows
+   `/Brepro`. Seis targets de P12: win-x64, win-arm64, darwin-x64,
+   darwin-arm64, linux-x64 y linux-arm64 (en Linux, musl o glibc lo decide
+   F2-10). La única dependencia externa en ejecución es `git` (F1-4).
+
+4. **Cómo llega el CLI con el plugin (D3a).** El plugin no lleva binarios:
+   - `bin/tc2` (sh), `bin/tc2.cmd` y `bin/tc2.ps1`: **lanzadores sin
+     runtime** que detectan el target, buscan el binario en la caché
+     (`%LOCALAPPDATA%\taskcodetwo\cli\<version>\` o
+     `~/.cache/taskcodetwo/cli/<version>/`) y lo ejecutan con los mismos
+     argumentos y el mismo código de salida;
+   - `cli.lock` en el plugin, revisado en su PR como cualquier fichero:
+
+     ```yaml
+     formato: 1
+     version: "2.0.0"
+     protocolo: [1, 1]            # rango que el plugin acepta
+     base_url: "https://github.com/charliebk/TaskCodeTwo/releases/download/v2.0.0/"
+     targets:
+       win-x64: {fichero: "tc2-2.0.0-win-x64.exe", sha256: "<64 hex>"}
+       # ... un fichero y un sha256 por target de P12
+     ```
+
+   - **primer uso**: si no está en la caché, el lanzador descarga el fichero
+     de su target, comprueba el **sha256 de `cli.lock`** antes de nada, lo
+     deja con un renombrado atómico y lo ejecuta. Si el hash no casa, borra
+     la descarga y sale con el código de precondición y el arreglo; nunca
+     ejecuta un binario sin comprobar;
+   - **sin red en el primer uso**: el lanzador lo dice en una línea, con la
+     URL exacta y la ruta de la caché donde dejar el fichero a mano; un
+     `tc2` instalado aparte se usa si su protocolo es compatible (punto 5);
+   - el hash viene del repo del marketplace y el binario de la release:
+     comprometer una sola de las dos cosas no basta para ejecutar otra cosa
+     (no da autenticidad del autor; eso lo da la firma, punto 6);
+   - **los hooks del plugin llaman a `tc2`**, nunca a `node`: la 2.0 no
+     necesita Node instalado (PoC S: los dos binarios dan la misma salida
+     sin node en el PATH ni variables `NODE_*`).
+   Meter los binarios en el repo del marketplace (D3b) pesaría, estimado con
+   el tamaño de win-x64 para los 6 targets, 15,6 MiB por versión en Rust y
+   500,3 MiB en TypeScript, para siempre en el historial (M).
+
+5. **Compatibilidad ejecutable–plugin.** `tc2 version --json` devuelve
+   `{"version", "protocolo", "target", "sha256"}`. El protocolo es un entero
+   que sube cuando cambia algo que el plugin usa (subcomandos, `--json`,
+   códigos de salida). Reglas:
+   - el CLI que trae `cli.lock` se acepta por hash;
+   - un `tc2` instalado aparte (releases, npm, winget, Homebrew) se acepta si
+     su `protocolo` cae en el rango de `cli.lock`;
+   - si no, sale con el código de precondición del contrato de F1-4 (el
+     número lo fija F2-3; el choque del 4 entre F1-4 y F1-12 ya está anotado
+     en F1-18) y el arreglo `tc2 actualizar --cli` (F2-12), en una línea.
+
+6. **Canales y firma (D4a, D5a).**
+
+   | Canal | Requisito | Integridad y firma | Cuándo |
+   |---|---|---|---|
+   | Marketplace de Claude Code (camino principal) | RR-16, P12 | lanzador + sha256 en `cli.lock` | 2.0 |
+   | Releases de GitHub | P12, F2-10 | binarios, `SHA256SUMS` y attestation de procedencia (Sigstore); macOS con firma ad hoc | 2.0 |
+   | Instaladores de una línea (`irm … \| iex`, `curl … \| sh`) | P12, F2-11 | descargan de la release y comprueban `SHA256SUMS` (y la attestation si hay `gh`) | 2.0 |
+   | npm `taskcodetwo` (binario `tc2`) | P12 | paquete principal con dependencias opcionales por plataforma, cada una con su binario; provenance de npm | al ser estable |
+   | winget | P12 | Authenticode | al ser estable |
+   | Tap de Homebrew | P12 | Developer ID y notarización | al ser estable |
+
+   - **Authenticode probado** con un certificado autofirmado desechable
+     creado con openssl en la carpeta de trabajo, sin tocar el almacén de
+     certificados del usuario (comprobado antes y después): `signtool sign`
+     firma los dos `.exe`, siguen dando la misma salida y `signtool verify
+     /pa` los rechaza por raíz no de confianza, lo esperado (PoC F). Con un
+     certificado real, la diferencia es la cadena; SmartScreen no se probó.
+   - **Coste y requisitos consultados en solo lectura el 2026-11-14**:
+     - Apple Developer Program: 99 USD por año de membresía
+       ([developer.apple.com/programs/enroll](https://developer.apple.com/programs/enroll/)); Developer ID y notarización
+       dependen de él.
+     - Azure Artifact Signing: dos niveles, Basic (5000 firmas al mes) y
+       Premium (100 000); la página de precios no mostró los importes
+       ([azure.microsoft.com, precios](https://azure.microsoft.com/en-us/pricing/details/artifact-signing/)): **importe no verificado**. Su guía
+       de inicio dice que los certificados Public Trust para **personas
+       físicas** solo se emiten en EE. UU. y Canadá, y para organizaciones
+       también en la UE ([learn.microsoft.com, quickstart](https://learn.microsoft.com/en-us/azure/artifact-signing/quickstart)): Carlos
+       como persona en España no puede usarlo; haría falta una organización
+       o un certificado OV de una CA (coste no consultado).
+     - winget: las políticas del repositorio (documento con fecha
+       2026-08-30) piden que `InstallerUrl` sea el sitio de releases del
+       autor y que no haya malware, y **no exigen firma**
+       ([learn.microsoft.com, políticas de winget](https://learn.microsoft.com/en-us/windows/package-manager/package/windows-package-manager-policies)). Se firma al entrar
+       igualmente por SmartScreen y antivirus (D4a); que la validación de
+       winget rechace binarios sin firma no se ha verificado.
+     - GitHub: aviso a partir de 50 MiB por fichero y bloqueo por encima de
+       100 MiB en un repo; las releases no limitan el total
+       ([docs.github.com, ficheros grandes](https://docs.github.com/en/repositories/working-with-files/managing-large-files/about-large-files-on-github)). El `.exe` de TypeScript
+       queda por encima del aviso (M).
+   - Pagar Authenticode y Developer ID es una decisión de negocio de Carlos
+     al entrar en winget y Homebrew; hasta entonces, hash y attestation.
+
+7. **Paquete npm.** `taskcodetwo` con el binario `tc2` (libre según F1-12)
+   y un paquete por plataforma como dependencia opcional; los nombres de los
+   paquetes por plataforma se **reservan antes de la primera publicación**
+   para evitar que otro los ocupe (si `@taskcodetwo` está libre no se ha
+   comprobado). El lanzador de npm es un shim mínimo: quien instala por npm
+   ya tiene Node.
+
+8. **Contrato del manifiesto de recursos (D6a, D7a).** Una función pura
+   que reutiliza el inventario de F3-9 (RF-14) y le añade hooks y
+   ejecutables con versión:
+
+   ```text
+   evaluar_recursos(inv: Inventario, m: Manifiesto, momento: Init | Doctor | Start(tarea)) -> InformeRecursos
+   Recurso { id, nivel: obligatorio | necesario | conveniente | estorba, comprobacion, arreglo }
+   ```
+
+   | Nivel | Qué entra | Qué hace |
+   |---|---|---|
+   | **obligatorio** | `git` con versión mínima, `tc2` con protocolo compatible, skills de fase del plugin, versión mínima del plugin | para con el arreglo en `init`, `doctor` y `start` |
+   | **necesario para el proyecto** | lo que declara su config: el comando de `verificacion` y sus ejecutables (F1-11), las revisoras del proyecto, el runner de F1-7; más el bloque opcional `recursos:`, que **solo añade** | `doctor` lo lista; `start` para antes de tomar la tarea si **esa tarea** lo usa |
+   | **conveniente** | engram, context7, rtk | avisa y sigue: **engram no es obligatorio** |
+   | **estorba** | hooks de terceros sobre las mismas herramientas (PreToolUse sobre Bash o PowerShell que reescriben `git` o `tc2`), skills con nombres que chocan con las de fase, la 0.16 instalada a la vez | avisa en `doctor`; para `start` solo si un hook intercepta herramientas que la fase usa o si la 0.16 escribe en el mismo proyecto (D7a) |
+
+   - El suelo (obligatorio, conveniente y las reglas de «estorba») va en el
+     binario; la config solo añade «necesario» (patrón de F1-10).
+   - **Un solo comando determinista**: `tc2 doctor --recursos [--json]`;
+     `init` y la entrada de `start` llaman a la misma función, y en `start`
+     solo cuentan lo obligatorio y lo necesario de la tarea. Un hook
+     SessionStart opcional solo lo ejecuta (F1-11, decisión 10). Salida de 5
+     líneas como máximo con el arreglo de cada fallo (F1-4).
+   - Un caso real de «estorba» en esta misma tarea: un hook de terceros que
+     reescribe órdenes de Bash hizo que algunas llamadas no se pudieran
+     revisar y hubo que repetirlas; es el tipo de interferencia que D7a
+     convierte en aviso, y en parada solo si toca `git` o `tc2`.
+   - Lo implementan F3-8 (`doctor`) y la `init` de F2-3, con el inventario de
+     F3-9.
+
+## Divergencias con el plan
+
+Este ADR no cambia el plan (D9a); propone estos cambios, para hacerlos con
+`taskctl` al cerrar:
+
+1. **F2-1 dice «Esqueleto: TypeScript»**. Texto propuesto: «Esqueleto: Rust
+   (`Cargo.lock` versionado, sin binarios en el repo, `/Brepro` en Windows),
+   `cargo fmt` y `clippy`, CI en Linux/macOS/Windows», con el criterio «CI
+   verde en los tres desde el primer commit y dos builds en limpio con el
+   mismo sha256 por target».
+2. **P12 dice que el plugin «trae el CLI ya compilado» y «lo pone en el
+   PATH, como hoy»**. Con D3a, el plugin trae el lanzador y `cli.lock`; el
+   binario se descarga en el primer uso y se comprueba por hash. Texto
+   propuesto: «el plugin trae un lanzador sin runtime y el hash exacto del
+   CLI de cada sistema; en el primer uso lo descarga de la release, lo
+   comprueba y lo deja en la caché».
+3. **P12 y F2-11: «comprueban su firma o su hash»**: en la 2.0 es el hash
+   (`SHA256SUMS` y `cli.lock`) más la attestation; Authenticode y Developer
+   ID llegan con winget y Homebrew (D4a).
+4. **F2-10 dice «firmados»**: desde la primera release, «con attestation de
+   procedencia y firma ad hoc en macOS; Authenticode y Developer ID al
+   entrar en winget y Homebrew». Además, F2-10 recibe la ejecución en Linux y
+   macOS que aquí queda «no verificado».
+5. **F2-11 y F2-12 nombran `taskctl`**: con los nombres de F1-12 son `tc2` y
+   `tc2 actualizar --cli`.
+6. **F3-8 (`doctor`)** no nombra el manifiesto de recursos: se añade
+   `tc2 doctor --recursos` con el contrato del punto 8 de la decisión, y
+   `init` (F2-3) y `start` (F3-6) llaman a la misma función.
+7. **Criterio de F1-14, «prueba de concepto de un ejecutable en los tres
+   sistemas»**: solo se ejecutó en Windows. Linux y macOS no se pudieron
+   compilar ni ejecutar sin descargar targets o imágenes (decisión de Carlos:
+   comparar con lo instalado); pasan al criterio de F2-10.
+8. **D2a con bun**: bun no estaba instalado y no se descargó; TypeScript se
+   midió solo con Node SEA en Windows.
+
+## Requisitos y evidencia
+
+| Requisito | Decisión | Evidencia |
+|---|---|---|
+| RR-16 instalación fácil | marketplace con lanzador y `cli.lock`; releases e instaladores con hash (puntos 4 y 6) | PoC S: los dos binarios funcionan sin node en el PATH; PoC M: peso de meter binarios en el repo |
+| RR-19 status line | Rust por arranque en caliente (punto 1) | PoC T: más rápido en caliente y en frío en los 3 repos (relación estable en la tabla; cifras de reloj en el log) |
+| P12 un ejecutable por sistema | 6 targets en runners nativos (punto 3) | PoC X: solo win-x64 compilado y ejecutado; el resto no verificado |
+| P12 firma o hash | hash y attestation en la 2.0; Authenticode y Developer ID al ser estable (punto 6) | PoC F: firma con certificado desechable, ejecución tras firmar y rechazo de verify por raíz no de confianza; almacén sin cambios |
+| F1-4 git como proceso y contrato de salida | `git` como única dependencia externa; código de precondición con arreglo | PoC S: las dos implementaciones lanzan `git log` como proceso hijo |
+| F1-6 GitHub por HTTP | TLS con el almacén del sistema (punto 2) | `NOTAS.md` de las dos PoC: `red: no` con raíces embebidas y una CA local, `red: si` con el almacén |
+| F1-9 YAML estándar que rechaza duplicadas | serde-saphyr con duplicadas como error | PoC S: 404 y 96 tareas iguales a PyYAML; la clave duplicada en una tarea real la rechazan los dos, PyYAML la acepta |
+| F1-11 decisión 10, comandos y no hooks | `tc2 doctor --recursos`; los hooks del plugin llaman a `tc2` | contrato del punto 8; los hooks de la 0.16.1 lanzan `node` (observado) |
+| F1-12 nombres | `tc2`, `taskcodetwo`, npm `taskcodetwo` | npm `taskcodetwo` libre según la PoC N de F1-12 |
+| RR-12 equipo y cadena de suministro | hash en el repo del marketplace y binario en la release; attestation | por diseño; la attestation no se ha ejecutado (GitHub solo lectura) |
+| RF-14 inventario | el manifiesto reutiliza el inventario de F3-9 | contrato del punto 8; sin implementación |
+
+## Consecuencias
+
+- **F2-1**: esqueleto en Rust con el perfil del punto 3, `/Brepro` y la
+  comprobación de dos builds iguales.
+- **F2-5**: el YAML detrás de un módulo con la batería de esquema estricto
+  (duplicadas, escalares, `id` y `titulo` obligatorios), para poder cambiar
+  de crate.
+- **F2-10**: matriz de 6 targets en runners nativos, `SHA256SUMS`,
+  attestation y firma ad hoc en macOS; ejecuta en Linux y macOS lo que aquí
+  no se pudo.
+- **F2-11**: instaladores que comprueban `SHA256SUMS` antes de poner `tc2`
+  en el PATH.
+- **F2-12**: `tc2 version --json`, el protocolo y `tc2 actualizar --cli`;
+  el plugin publica `cli.lock` en cada versión.
+- **F2-3** (`init`), **F3-6** (`start`) y **F3-8** (`doctor`): el manifiesto
+  de recursos con `evaluar_recursos` y `tc2 doctor --recursos`; **F3-9** da
+  el inventario.
+- **F3-12**: la status line llama al binario de Rust; el presupuesto de
+  tiempo por refresco se mide allí.
+- **F2-14**: la desinstalación borra la caché del lanzador.
+- **Carlos**: decide cuándo pagar Authenticode (con organización o CA, no
+  Azure Artifact Signing como persona) y el Apple Developer Program.
+
+## Fuera de alcance
+
+- La desinstalación limpia de la 2.0 y de la 0.16: F2-14 y F9-10.
+- Selección de skills, agentes y modelo (qué hace «necesario» a una
+  revisora): F1-15.
+- Los números definitivos de los códigos de salida: F2-3 (F1-4).
+- La batería real de pruebas por sistema y la CI en los tres: F2-1 y F2-10.
+- El esquema exacto del bloque `recursos:` de la config: F2-5 con F1-11.
+
+## No verificado
+
+- **Linux y macOS**: no se compiló ni se ejecutó nada. Rust no tiene
+  instalados los targets de Linux ni de macOS y no se descargaron; aunque se
+  instalaran, `ring` necesitaría un compilador C del destino. Docker no
+  respondía (daemon parado) y WSL solo tiene la distribución interna
+  `docker-desktop`. No hay macOS.
+- **Windows ARM64**: target `aarch64-pc-windows-msvc` no instalado.
+- **TypeScript fuera de Windows**: Node SEA necesita el `node` del sistema
+  de destino y bun no está instalado.
+- **Compilación cruzada reproducible**: no verificada en ninguno de los dos.
+  Que `/Brepro` deje el build de Rust igual byte a byte se midió solo en
+  win-x64, en la misma carpeta y la misma máquina (PoC B).
+- **Firma con un certificado real**: SmartScreen, reputación, sello de
+  tiempo (`/tr`) y la validación de winget sin firma no se han probado; solo
+  la mecánica con un certificado desechable.
+- **Importe de Azure Artifact Signing**: la página de precios no lo mostró;
+  el coste de un certificado OV de una CA no se consultó.
+- **macOS**: la firma ad hoc, Developer ID, notarización y Gatekeeper con un
+  binario descargado por `curl`.
+- **Marca de la web (MOTW)** en Windows al descargar con `irm` o con el
+  lanzador, y falsos positivos de antivirus en otras máquinas.
+- **El lanzador, `cli.lock`, `tc2 version --json` y `tc2 doctor --recursos`**
+  no existen: son contratos de este ADR.
+- **Attestation de GitHub y provenance de npm**: no ejecutadas (GitHub solo
+  lectura); si el scope npm `@taskcodetwo` está libre, no comprobado.
+- **Tokens de cada implementación**: no se anotaron; el criterio 5 de D1a
+  queda sin decidir.
+- **Arranque en otras máquinas**: una sola máquina, con antivirus y una CA
+  local; los milisegundos del log no se generalizan, solo la relación.
+- **`node --build-sea`** de versiones de Node posteriores a la 22 (que
+  quitaría postject): no probado.
diff --git a/docs/adr/README.md b/docs/adr/README.md
index a0ee67a..27e4acd 100644
--- a/docs/adr/README.md
+++ b/docs/adr/README.md
@@ -20,6 +20,7 @@ directo a la tabla de la Fase 1 y a su tarea.
 | [F1-10](F1-10-carriles-y-modo-automatico.md) | Carriles y modo automático: cuatro carriles ordenados más release y hotfix por tipo, tabla única de política por carril con su función pura (`accion` en `siguiente --json`), configuración que solo endurece (`politica_carriles`, `rutas_carril`), carril mínimo por ficheros con los dos nombres de cada renombrado, y paradas fijas (`approve`, `merge-pr`, tag, tope de rondas, subida de carril) | TASK-011 | aceptada |
 | [F1-11](F1-11-extensiones-declaradas.md) | Extensiones declaradas: inventario comprobado de las 14 prácticas que los pilotos hacen a mano, clasificadas en parche de TaskCode (se retira al migrar) o propio del proyecto, con el desglose de líneas; tres claves (`verificacion`, `derivados`, `confianza_incluye`) con esquema que falla cerrado y rutas normalizadas; derivados que ejecuta `taskctl` en la transición solo en la rama de estado, comparados por contenido; check obligatorio `estado`; `taskctl tareas --json` como lectura estable; sin estados propios; revisoras como skill del proyecto; «comandos sí, hooks solo como atajo opcional»; y la ventaja medida frente a la 0.16, con lo que no gana | TASK-012 | aceptada |
 | [F1-12](F1-12-actualizacion-entre-versiones.md) | Actualización entre versiones: el plan de migración como dato (`tc2 actualizar --simular` sin cambiar un byte, mismo hash, vacío en la segunda pasada), qué migra (aplanado, formato 2, archivadas a `descartada`, config a los bloques de F1-11), historia con `git subtree split`, transacción local con criterio de salida estructural y vuelta atrás automática, retirada de parches con capacidades y huecos bloqueantes, convivencia con la 0.16 (su lector de config aborta con `---`: ninguna orden de escritura entra en un proyecto migrado), nombres definitivos `tc2` y `taskcodetwo`, y orden de pilotos OpenDockTrading → TaskCodeTwo → ToolsaWeb | TASK-013 | aceptada |
+| [F1-14](F1-14-distribucion-e-instalacion.md) | Distribución e instalación: CLI en Rust por la regla D1a medida frente a TypeScript con Node SEA (mismo comando, salida idéntica sobre las 404 tareas de ToolsaWeb y las 96 de OpenDockTrading; más rápido en caliente en los 3 repos y 32,1 veces más pequeño), TLS con el almacén del sistema, lanzador sin runtime con `cli.lock` y descarga comprobada por sha256, compatibilidad por protocolo con `tc2 version --json`, canales y firma por etapas (hash y attestation en la 2.0; Authenticode y Developer ID al entrar en winget y Homebrew) y contrato del manifiesto de recursos (`tc2 doctor --recursos`: obligatorio, necesario, conveniente y estorba); Linux y macOS no verificados | TASK-014 | aceptada |
 | [F1-18](F1-18-hito-de-decisiones.md) | Hito de decisiones (ADR) en `init` y en `actualizar`: estudio de adopción que lo dimensiona (bloque «uso medido» del informe de F1-12 y `proponer_hito` con un elemento por hueco, bloqueantes antes de migrar), proporcionalidad con mediana y p75 de la Fase 1 (45,5–50 min y 13,9–19,1 M tokens por ADR), registro `hito-decisiones.yml` en `taskcodetwo/tareas` cuya lista, descartes y «no necesario» firma una persona, con estado calculado que falla cerrado, y puerta determinista en `tc2 start` y en el check `estado` (código `rechazada`) | TASK-094 | aceptada |
 
 ## Plantilla
````

## Excluido del diff (18 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-014/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{01-en-diseno => 02-en-curso}/TASK-014/planificacion/plan-final.md                 |   0
 tareas/02-en-curso/TASK-014/poc/especificacion-comando.md                                 |  36 ++++
 tareas/02-en-curso/TASK-014/poc/poc-f1-14.log                                             | 119 +++++++++++++
 tareas/02-en-curso/TASK-014/poc/poc-f1-14.py                                              | 927 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/02-en-curso/TASK-014/poc/poc-rust/Cargo.lock                                       | 778 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/02-en-curso/TASK-014/poc/poc-rust/Cargo.toml                                       |  17 ++
 tareas/02-en-curso/TASK-014/poc/poc-rust/DEPENDENCIAS.md                                  |  31 ++++
 tareas/02-en-curso/TASK-014/poc/poc-rust/NOTAS.md                                         |  67 ++++++++
 tareas/02-en-curso/TASK-014/poc/poc-rust/src/main.rs                                      | 175 +++++++++++++++++++
 tareas/02-en-curso/TASK-014/poc/poc-ts/DEPENDENCIAS.md                                    |  34 ++++
 tareas/02-en-curso/TASK-014/poc/poc-ts/NOTAS.md                                           | 132 +++++++++++++++
 tareas/02-en-curso/TASK-014/poc/poc-ts/package-lock.json                                  | 942 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/02-en-curso/TASK-014/poc/poc-ts/package.json                                       |  23 +++
 tareas/02-en-curso/TASK-014/poc/poc-ts/scripts/build.mjs                                  | 105 ++++++++++++
 tareas/02-en-curso/TASK-014/poc/poc-ts/src/main.ts                                        | 243 +++++++++++++++++++++++++++
 tareas/02-en-curso/TASK-014/poc/poc-ts/tsconfig.json                                      |  16 ++
 tareas/{01-en-diseno => 02-en-curso}/TASK-014/tarea.md                                    |   7 +-
 18 files changed, 3651 insertions(+), 1 deletion(-)
````

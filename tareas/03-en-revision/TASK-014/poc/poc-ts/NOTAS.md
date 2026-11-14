# Notas de la PoC TypeScript + Node SEA (F1-14, TASK-014)

Medido por el agente implementador el 2026-11-14 en Windows 11, Node 22.23.4,
npm 10.9.9. Todo lo medido se ejecutó; lo que no, se dice.

## Esfuerzo

- **Minutos**: ~9 de reloj (de leer la especificación a tener el `.exe`
  probado y estas notas escritas; marcas `date +%s` 1794678692 → ~1794679240).
  Incluye crear el repo de juguete (que luego otro agente sobrescribió, ver
  problemas) y consultar versiones en npm.
- **Iteraciones de compilar y corregir**: **1**. El primer `tsc --noEmit` en
  modo estricto pasó sin errores y la primera inyección SEA produjo un `.exe`
  que funcionó. Hubo una segunda build sólo para medir el tiempo en limpio con
  `npm ci`.
- **Líneas de código propias**: `src/main.ts` 198 (178 sin blancos ni
  comentarios); `scripts/build.mjs` 105 (89). Total 303 (267). Más
  `package.json` (23) y `tsconfig.json` (17).

## Build

- Script: `node scripts/build.mjs [carpeta]` (por defecto
  `%LOCALAPPDATA%/pruebas-claude/taskcodetwo/task-014/build-ts`). Copia las
  fuentes fuera del repo, borra `node_modules`, `npm ci`, `tsc --noEmit`,
  esbuild → `dist/tablero.cjs`, `node --experimental-sea-config`, copia
  `node.exe`, `signtool remove /s`, `postject` con el fuse
  `NODE_SEA_FUSE_fce680ab2cc467b6e072b8b5df1996b2`.
- **Paquetes en `node_modules`**: 9 (1 de runtime: `yaml`, sin transitivas).
- **Tiempo de build en limpio** (con lock, `node_modules` borrado):

  | Paso | 1ª build (`npm install`, sin lock) | 2ª build (`npm ci`) |
  |---|---|---|
  | npm | 15,9 s | 8,6 s |
  | tsc --noEmit (TS 7 nativo) | 0,7 s | 0,8 s |
  | esbuild | 0,3 s | 0,3 s |
  | blob SEA | 0,1 s | 0,1 s |
  | copiar node + signtool | 0,6 s | 0,2 s |
  | **postject** | **29,0 s** | **24,9 s** |
  | **Total** | **46,6 s** | **35,0 s** |

  Sin red (con caché de npm) la parte de npm baja, pero no se midió.
  postject es el cuello de botella: dos tercios del tiempo para escribir un
  blob de 262 KB en un PE de 83 MB.
- **Tamaño del `.exe`**: **87 429 632 bytes (83,4 MiB)**. Es casi todo el
  `node.exe` de Node 22; el bundle JS son 262 KB (`yaml` incluido).

## Ejecución

- `.exe` ejecutado con un PATH del que se quitaron todas las entradas con
  `nodejs`, `npm` y `pnpm` (`command -v node` vacío): funciona. `git` sí
  sigue en el PATH (la especificación lo exige como proceso hijo).
- `process.argv` en SEA: `argv[1]` repite el ejecutable, así que
  `process.argv.slice(2)` es correcto (comprobado con errores de uso).
- Sin aviso "experimental SEA" gracias a `disableExperimentalSEAWarning`.
- Arranque: ~0,78-0,90 s de reloj por ejecución con `--sin-red` contra
  TaskCodeTwo (94 tareas), medido con `time` incluyendo el `bash` envoltorio.
  Con red ~2,1 s.
- Salida contra TaskCodeTwo con `--sin-red` **idéntica byte a byte** (`cmp`)
  a `salida-rust-taskcodetwo.txt` que dejó la PoC Rust en la carpeta de
  pruebas. Contra el juguete no hubo salida Rust con la que comparar.
- Códigos de salida: 0 normal; 2 en error de uso, `<raiz>/tareas` ilegible o
  `git log` fallido (repo sin git). Comprobados los tres.

## Decisiones de interpretación de la especificación (a alinear con Rust)

1. **Frontmatter**: la primera línea del fichero (tras quitar un BOM UTF-8 y
   un `\r` final) debe ser `---`; el cierre es la siguiente línea exactamente
   `---`. Si la primera línea no es `---` o no hay cierre → `sin frontmatter`.
   Las líneas CRLF se aceptan.
2. Si `yaml` devuelve algún error `DUPLICATE_KEY` → `clave duplicada`
   (prioridad sobre otros errores); cualquier otro error, o un frontmatter
   que no sea un mapa (vacío, lista, escalar) → `yaml invalido`.
3. `estado` ausente o `null` → `desconocido`; si no es texto (número,
   booleano) se convierte con `String()`. `id` y `titulo` se leen pero no
   invalidan la tarea si faltan (la salida no los usa).
4. Sólo cuentan ficheros regulares llamados exactamente `tarea.md`; no se
   siguen enlaces simbólicos. `<raiz>/tareas` inexistente es error de E/S.
5. Red: `fetch` con timeout de 15 s; cualquier excepción o no-2xx → `no`.

## Problemas encontrados

- **Repo de juguete en carrera**: no existía al empezar y lo creé; el agente
  de la PoC Rust lo creó a la vez (mismo minuto) y el que quedó es el suyo
  (8 tareas, una CRLF, una con `ñ`, `Zeta/` con `Bloqueada`). Lo usé tal cual
  sin modificarlo.
- **Virtualización MSIX de `%LOCALAPPDATA%`**: los procesos lanzados desde la
  app de Claude ven `C:\Users\nullcad2025\AppData\Local\pruebas-claude\...`,
  pero `npm ls` reveló que físicamente está en
  `C:\Users\nullcad2025\AppData\Local\Packages\Claude_pzs8sxrjxfjjc\LocalCache\Local\pruebas-claude\...`.
  Un proceso fuera de la app (Explorador, otra terminal) podría no ver la
  carpeta en la ruta "normal".
- **postject** lleva sin publicar desde mayo de 2023 y sigue en alfa (ver
  DEPENDENCIAS.md); además es lento (~25-29 s por inyección).
- `signtool` no está en el PATH: el script lo busca en el Windows SDK. Sin él
  la inyección funciona pero deja una firma Authenticode inválida.
- El `.exe` resultante no está firmado: SmartScreen/antivirus pueden avisar al
  distribuirlo (no probado en otra máquina).
- `typescript` 7 (nativo) sólo se usa como comprobador de tipos; el JS lo
  genera esbuild, así que el código que corre no pasa por `tsc`.

## Alineación con la PoC Rust (iteraciones extra, implementador final, 2026-11-14)

Las interpretaciones 3 a 5 de arriba y los códigos de salida divergían de
`poc-rust/NOTAS.md`. `main.ts` llegó sin cambios a medias (coincidía con lo
descrito en estas notas). Se alineó con Rust con cambios mínimos y se comparó
stdout byte a byte y código de salida con un arnés de 11 casos (juguete,
`casos`, un repo con 27 `tarea.md` límite y un `TAREA.md`, repo sin `tareas/`, `tareas` como
fichero, raíz inexistente, sin argumentos, solo `--sin-red`, dos raíces,
opción desconocida, `--sin-red` delante). **3 iteraciones extra de compilar y
comparar** (0 errores de tipos):

1. Código de salida 1 (no 2) en errores propios; `tareas/` ausente o que no es
   carpeta = 0 tareas; timeout de red 10 s; un argumento que empieza por `--`
   se toma como raíz (como Rust); `id` y `titulo` obligatorios y escalares no
   nulos, `estado` nulo o ausente = `desconocido`, `estado` lista o mapa =
   `yaml invalido`; el escalar se toma por su **texto fuente** (`1.50`, `0o17`,
   `1e3`, `.inf`, `yes` salen tal cual, como en serde-saphyr; con `toJS()`
   salían `1.5`, `15`, `1000`, `Infinity`); claves de fusión `<<` resueltas
   (`merge: true`). Quedó un caso distinto: la fusión.
2. Con `merge: true` la clave `<<` llega como `Symbol(<<)`: corregido. 11/11
   casos iguales en stdout y código.
3. **Red y almacén de certificados**: el `fetch` de Node usa solo las raíces de
   Mozilla. En esta máquina daba `red: si` únicamente porque el entorno de la
   app define `NODE_USE_SYSTEM_CA` y `NODE_EXTRA_CA_CERTS`; con esas variables
   quitadas daba `red: no` (`UNABLE_TO_VERIFY_LEAF_SIGNATURE`), mientras Rust
   (platform-verifier) seguía en `si`. Corrección: `tls.setDefaultCACertificates`
   con las raíces `default` + `system` antes del `fetch` (API presente en
   Node 22.23.4). Con las variables quitadas, los dos dan `red: si`.

Tras la alineación: `src/main.ts` 243 líneas (215 sin blancos ni comentarios
de línea); build en limpio con `npm ci` 18,5 a 41,1 s según la ejecución
(postject 13,4 a 28,2 s). Tamaño del `.exe`: 87 430 656 bytes.

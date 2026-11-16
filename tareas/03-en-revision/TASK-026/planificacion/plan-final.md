# Plan — TASK-026: F2-11 Instaladores de una linea

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 90 min (de la rama abierta al cierre con merge; senales: 4 ficheros de codigo previstos (`install.sh`, `install.ps1`, `release.yml` y un test en `xtask`) más 1 doc; 3 capas (scripts de usuario, CI de release, doc); 2 criterios; cada vuelta de CI en 6 runners tarda de 10 a 15 min de reloj y hacen falta al menos 2 (ensayo y prerelease). Precedente: TASK-025 se estimó en 170 y tardó 71 con una forma parecida)

**Qué es, en una frase.** Dos scripts sin dependencias, `install.sh` (POSIX `sh`, no bash) e `install.ps1` (compatible con Windows PowerShell 5.1, que es lo que trae un Windows limpio). Hacen para la persona lo mismo que ya hace el job `verificar-release` de `.github/workflows/release.yml` (TASK-025): bajar `SHA256SUMS` y el binario de su target desde la URL pública, exigir una sola línea que case (`grep -F "  $FICHERO"`), comprobar el sha256 y, si hay `gh` con sesión, la attestation con `--signer-workflow`. Después dejan `tc2` en el PATH. Ese job es el precedente interno más cercano y su contrato se reutiliza tal cual. En el repo no hay ningún instalador previo de la 0.16: el grep de `install.sh`, `install.ps1`, `irm` y `curl … | sh` solo da documentos de plan, y el ADR F1-14 deja dicho que la 0.16.1 se instala como plugin con `bin/taskctl` sobre Node.

**Lo que se toma como dado (verificado leyendo):**
- Nombres de los assets: `tc2-<version>-<alias>[.exe]`, con los 6 alias de P12 (`docs/distribucion/releases.md` y la matriz de `release.yml`).
- `SHA256SUMS` en formato `sha256sum`, que hoy lista solo los 6 binarios. `publicar` comprueba `ls dist/tc2-* | wc -l` = 6 y que la release tiene 7 assets.
- Una attestation por binario, firmada por `release.yml`.
- Linux con musl estático. macOS con firma ad hoc.
- Repo público, descarga anónima (P12, decisión del 2026-11-13).

**Qué se crea:**
1. **`instalar/install.sh`** y **`instalar/install.ps1`**. La carpeta es una propuesta y la ruta exacta queda a decisión (pregunta 7).
   - **Contrato de entrada**, solo por variables de entorno, porque con `curl | sh` e `irm | iex` no se pasan argumentos con comodidad:
     - `TC2_VERSION`: por defecto, la última estable, que se saca de la redirección de `https://github.com/<repo>/releases/latest` sin usar la API, que tiene límite anónimo.
     - `TC2_INSTALL_DIR`.
     - `TC2_NO_MODIFY_PATH=1`.
     - `TC2_BASE_URL`: solo para el ensayo de CI contra un servidor local.
   - **Contrato de salida**: código 0 y una línea con la ruta y la versión instaladas; distinto de 0 con una línea de arreglo, siguiendo la regla de 5 líneas del contrato de F1-4. Nunca deja un binario sin comprobar en el destino: descarga en un temporal, comprueba y solo entonces renombra.
   - **Detección del target**:
     - sh: `uname -s` / `uname -m`.
     - ps1: `RuntimeInformation.OSArchitecture`, no la arquitectura del proceso, para que una PowerShell x64 emulada en ARM64 baje el binario arm64.
     - Es la 4.ª y 5.ª copia de la tabla de alias. TASK-025 ya aceptó 3 copias (MEN-6, no corregido); aquí la costura es un test que las compara con `xtask/src/dist.rs`, en vez de generarlas.
   - **Destino**:
     - Windows: `%LOCALAPPDATA%\taskcodetwo\bin\tc2.exe` y PATH de usuario en el registro, sin pedir permisos de administrador.
     - macOS y Linux: `~/.local/bin/tc2`. Si ese directorio no está en el PATH, se añade un bloque con marcas de inicio y fin al perfil del shell de la persona, salvo que esté puesto `TC2_NO_MODIFY_PATH` (pregunta 3).
     - Las marcas y las rutas fijas son la costura con F2-14: desinstalar sabe qué quitar sin un fichero de registro aparte.
   - **Convivencia con la 0.16**: no toca `taskctl` ni nada de la 0.16 (nombres de F1-12 D1a). El plan dice `taskctl`, pero el ADR F1-14, divergencia 5, lo fija como `tc2`.
   - **Todo el script dentro de una función `main` que se llama en la última línea**, para que una descarga cortada no ejecute medio script.
   - **Ni Node ni bash**. Herramientas que usa:
     - en sh: `curl` o, si no está, `wget`; `sha256sum` o, si no está, `shasum -a 256` (lo mismo que `verificar-release`); `uname`, `mkdir`, `mv`, `chmod`;
     - en ps1: `Invoke-WebRequest` y `Get-FileHash`, forzando TLS 1.2 en la 5.1.
2. **Job `instaladores-ensayo`** en `release.yml`. Corre en cada push que toque `instalar/**`, que se añade a los `paths`.
   - Matriz de 6 runners, más un contenedor alpine fijado por digest, el mismo que usa `humo.sh`.
   - Sirve los artefactos `bin-*` del build y un `SHA256SUMS` montado como en `publicar` desde un servidor HTTP local.
   - Ejecuta el instalador con `TC2_BASE_URL`. Se prueba antes de publicar nada.
3. **Job `verificar-instaladores`** después de `publicar`, con la misma matriz: ejecuta la línea real contra la URL pública y mide el tiempo (criterio 1, ver «Plan de pruebas»).
4. **Doc** `docs/distribucion/instalar.md`, con las dos líneas, las variables y qué comprueba cada script. Además, `releases.md` remite a ella.

**Qué se extiende:**
- `release.yml`, job `publicar`:
  - adjunta `install.sh` e `install.ps1` como assets de la release;
  - los incluye en el `subject-path` de la attestation, pero **no** en `SHA256SUMS`, que sigue siendo el contrato de los 6 binarios del que se rellena `cli.lock` (F2-12) (pregunta 2);
  - la comprobación de assets pasa de 7 a 9.
- Las líneas públicas quedan estables sin poner versión en el nombre del asset: `irm https://github.com/charliebk/TaskCodeTwo/releases/latest/download/install.ps1 | iex` y `curl -fsSL …/latest/download/install.sh | sh`.
- `xtask`: un test que lee los dos scripts y compara su tabla de alias con la de `dist.rs`.
- Fila F2-11 del plan: `taskctl` pasa a `tc2` (divergencia 5 del ADR F1-14), igual que TASK-025 recogió la divergencia 4 en la fila F2-10.

**Orden de construcción:**
1. Escribir los scripts.
2. Montar el ensayo local en CI. Esto ya se puede mergear sin publicar nada.
3. Publicar los instaladores como assets y montar `verificar-instaladores`.
4. Prerelease de evidencia.
5. Doc y fila del plan.

**Qué queda fuera, para no solapar con otras tareas:**
- **F2-12 (TASK-027)**: `cli.lock`, el lanzador del plugin (D3a), `tc2 version --json` con el protocolo y `tc2 actualizar --cli`. Volver a ejecutar el instalador reinstala, pero no es «actualizar».
- **F2-14**: `tc2 desinstalar`. Aquí solo se deja la costura: rutas fijas y el bloque con marcas.
- **F9-9a (TASK-071)**: pruebas en máquinas limpias de verdad y «primera tarea en 15 min».
- **F9-9 (TASK-072)**: publicar la 2.0.0.
- Canales winget, Homebrew y npm: llegan al ser estable (D5a).
- Authenticode y Developer ID: llegan con winget y Homebrew (D4a).

## Por que no se trocea

La estimacion (90 min) pasa de la regla de una hora y el rol propuso trocear en
dos partes (abajo, solo como referencia del orden interno). **La persona decidio
no trocear** (2026-11-16). Motivo: es un unico entregable coherente (dos scripts
y su verificacion en la misma `release.yml`), la parte a sola no aporta valor a
nadie hasta que se publica, y el precedente TASK-025, de forma parecida, se
estimo en 170 min y se cerro en 71. Las dos partes se usan como orden de
construccion dentro de la misma rama.

### Orden interno a: scripts y ensayo local (unos 50 min)
- Se crean `instalar/install.sh` e `instalar/install.ps1`, con el contrato de variables, destino, PATH y comprobación del sha256 y de la attestation.
- Job `instaladores-ensayo` en `release.yml`: servidor local con los artefactos del build, matriz de 6 runners más alpine, medida del tiempo.
- Test de alias en `xtask`.
- Se puede mergear sola: no publica nada ni cambia el contrato de la release.

### Orden interno b: publicación y evidencia real (unos 40 min)
- `publicar` adjunta los dos scripts como assets: el conteo pasa a 9 y entran en la attestation, no en `SHA256SUMS`.
- Job `verificar-instaladores` con la línea real contra la URL pública.
- Prerelease `v0.0.0-prueba.N` como evidencia, borrada después con permiso, como hizo TASK-025.
- `docs/distribucion/instalar.md` y la fila F2-11 del plan.

## Lo que el rol no cubrio

- **Riesgos (no hay rol de riesgos)**: no se han recorrido de forma sistemática los modos de fallo. Sale solo lo que se vio de paso en el diseño (siguiente sección). Sin revisar:
  - Rosetta en macOS, que con `uname -m` da `x86_64`;
  - el proxy corporativo y la CA local que el ADR F1-14 punto 2 vio en esta misma máquina;
  - antivirus y SmartScreen;
  - shells de login poco habituales (fish, nushell);
  - PowerShell en modo de lenguaje restringido;
  - un `TC2_BASE_URL` que no sea https.
- **Pruebas (no hay rol de testing)**: el plan de pruebas lo diseña arquitectura y se limita a lo que la forma elegida hace comprobable. No se ha pensado en cobertura de casos límite.
- **Dominio (no hay rol de dominio)**: hay tres lecturas de reglas que no he verificado.
  - Que «poner `taskctl` en el PATH» signifique `tc2`: lo apoyan el ADR F1-12 D1a y la divergencia 5 de F1-14, pero el texto del plan no está corregido.
  - Qué cuenta como «máquina limpia»: se propone una definición, en la pregunta 5.
  - Que «verificar la firma o el hash» se cumpla con el hash obligatorio y la attestation opcional: lo dice el ADR F1-14 punto 6, «y la attestation si hay `gh`».
- **Contraste con los criterios:**
  - Criterio 1: queda comprobable en runners y en alpine, no en máquinas limpias de verdad, que son de F9-9a. Es un cumplimiento parcial y declarado.
  - Criterio 2: queda cubierto en descarga, hash, attestation condicional y PATH, con la salvedad del nombre `tc2`.

## Riesgos aceptados y que los contiene

- **El hash viene del mismo sitio que el binario**: da integridad, no autoría. Lo que la contiene es la attestation cuando hay `gh` con sesión. Sin `gh`, el instalador lo dice en una línea y sigue, como fija el ADR F1-14 D4a.
- **No hay «latest» hasta que exista una release estable.** Hoy no hay ninguna: TASK-025 borró su prerelease. Antes de la 2.0.0, la línea necesita `TC2_VERSION`. Lo contiene un mensaje de error con el arreglo exacto, y la doc lo dice.
- **Los runners no son máquinas limpias**: traen curl, gh, Node y una red rápida. Lo contienen el contenedor alpine en Linux y la prueba real de F9-9a (TASK-071), que ya lo tiene asignado (`## Resultado` de TASK-025).
- **Tabla de alias en 5 sitios.** Lo contiene el test de `xtask` que compara los dos scripts con `dist.rs`. Las 3 matrices de Actions siguen como aceptó MEN-6 de TASK-025.
- **Tocar el perfil del shell en Unix** es invasivo. Lo contienen un bloque con marcas, idempotente y que quita F2-14, y la salida `TC2_NO_MODIFY_PATH=1`.
- **Sin verificar:**
  - la marca de la web (MOTW) con `Invoke-WebRequest` en un Windows con SmartScreen;
  - la cuarentena de macOS: `curl` no la pone en principio, pero nadie lo ha comprobado;
  - el TLS 1.2 forzado en PowerShell 5.1 detrás de un proxy corporativo.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

**Criterio 1, «instalación en una máquina limpia de cada sistema en menos de un minuto»: 2 tests, los dos pesados.** Cómo pasa a ser comprobable (pregunta 5):

- **T1 (pesado), `verificar-instaladores`.**
  - Dónde corre: los 6 runners nativos de la matriz de TASK-025 (con `windows-11-arm`, `macos-15-intel` y `ubuntu-24.04-arm`), más alpine fijado por digest en Linux x64 y arm64.
  - Qué ejecuta:
    - en Windows, la línea **en `powershell.exe` 5.1**, no en `pwsh`;
    - en macOS y en los runners de Linux, la de `sh`;
    - en alpine, la variante con `wget`, porque alpine no trae curl.
  - Cómo mide: el tiempo de reloj desde antes de lanzar la línea hasta que un **proceso nuevo** encuentra `tc2` en el PATH. Ese proceso es `powershell -NoProfile`, que relee el PATH de usuario del registro, o un `sh -l` que lee el perfil. `tc2 --version` tiene que dar exactamente `tc2 <version>`, como en `humo.sh`.
  - Umbral: **menos de 60 s**, y el tiempo medido se escribe en el log.
  - Evidencia: el run de la prerelease de la parte b.
- **T2 (pesado), la misma medida en `instaladores-ensayo`** contra el servidor local: es lo que da evidencia antes de publicar.

**Criterio 2, «cumple F2-11 y los requisitos que cita» (RR-16, P12, ADR F1-14): 3 tests, 2 pesados.**

- **T3 (pesado), hash que no casa.** En `instaladores-ensayo`, celdas de Linux y Windows: con un `SHA256SUMS` que tiene una línea alterada, el instalador sale con un código distinto de 0 y en el destino no queda `tc2`.
- **T4 (pesado), idempotencia y PATH.** En `instaladores-ensayo`: ejecutar el instalador dos veces deja un solo bloque de PATH en el perfil y una sola entrada en el PATH de usuario de Windows. Con `TC2_NO_MODIFY_PATH=1` no toca nada de eso.
- **T5 (ligero), tabla de alias.** Test de `xtask` dentro de `cargo xtask verificar`: la tabla de alias de los dos scripts es la de `dist.rs`.
- **Contraste con el ADR:** la attestation con `gh` presente ya la cubre T1, porque los runners traen `gh` con token.

Total: 5 tests nuevos, 4 de ellos pesados (jobs de CI en matriz). No hace falta pasarse del presupuesto.

## Lo que necesita decision de una persona

1. **¿Los instaladores se sirven como assets de cada release (`releases/latest/download/install.sh`) y no desde `raw.githubusercontent.com/…/main/`?** Recomendación: **sí, como assets**. Quedan inmutables por tag, entran en la attestation y van con la versión de los binarios. Desde `main` serían mutables y sin procedencia.
2. **¿Los instaladores quedan fuera de `SHA256SUMS`, aunque entren en la attestation?** Recomendación: **sí, fuera**. `SHA256SUMS` sigue siendo el contrato de los 6 binarios del que se rellena `cli.lock` (F2-12).
3. **En macOS y Linux, si `~/.local/bin` no está en el PATH, ¿se añade un bloque con marcas al perfil del shell, con salida `TC2_NO_MODIFY_PATH=1`, o solo se imprime la línea que hay que añadir?** Recomendación: **añadir el bloque con marcas**. El criterio dice «pone en el PATH», y las marcas permiten que F2-14 lo quite limpio.
4. **¿La attestation se comprueba solo si hay `gh` con sesión y, si no, se avisa en una línea y se sigue?** Recomendación: **sí**, que es lo que fija el ADR F1-14 punto 6. Hacerla obligatoria exigiría `gh` en una máquina limpia, y eso choca con «un minuto».
5. **¿Se acepta como definición comprobable del criterio 1 «menos de 60 s de reloj en los 6 runners nativos de GitHub (Windows en PowerShell 5.1) y en alpine, medido por el job `verificar-instaladores`», con la máquina limpia de verdad en F9-9a (TASK-071)?** Recomendación: **sí**. Es lo más cerca de una máquina limpia que se puede automatizar sin infraestructura nueva, y el hueco queda declarado.
6. **¿Hay permiso para publicar una prerelease de evidencia `v0.0.0-prueba.N` y borrarla después, release y tag, como en TASK-025?** Recomendación: **sí**. Sin release publicada, la línea real no se puede ejecutar.
7. **¿La carpeta es `instalar/`, en la raíz del repo, y se corrige la fila F2-11 del plan de `taskctl` a `tc2` (divergencia 5 del ADR F1-14)?** Recomendación: **sí a las dos**. La carpeta se puede cambiar sin coste; el nombre `tc2` ya lo decidió F1-12 D1a.
8. **¿Se trocea en las partes a y b de «Troceo», con un merge al final de cada una?** Recomendación: **sí**. La parte a no cambia el contrato de la release y se puede entregar sola.

### Respuestas de la persona (2026-11-16)

1. Sí: los instaladores se sirven como assets de cada release.
2. Sí: fuera de `SHA256SUMS` y dentro de la attestation.
3. Bloque con marcas en el perfil del shell, con la salida `TC2_NO_MODIFY_PATH=1`.
4. Sí: la attestation solo si hay `gh` con sesión; si no, aviso en una línea.
5. Sí: el criterio 1 se comprueba con menos de 60 s en los 6 runners nativos (Windows en PowerShell 5.1) y en alpine, mediante `verificar-instaladores`. La máquina limpia real queda para F9-9a (TASK-071).
6. Sí: hay permiso para publicar la prerelease `v0.0.0-prueba.N` y borrar después la release y el tag.
7. Sí: la carpeta es `instalar/` y se corrige la fila F2-11 de `taskctl` a `tc2`.
8. **No se trocea** (ver «Por que no se trocea»).

# Plan — TASK-025: F2-10 Build de ejecutables por sistema en la CI, firmados

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

La release sale de un workflow nuevo, `.github/workflows/release.yml`. Se lanza con `push` de tag `v*`, que es el tag anotado sobre el merge en `main` de ADR F1-3 D3a, y con `workflow_dispatch` en modo ensayo, que no publica nada. El empaquetado no se escribe en bash dentro del YAML: va a un subcomando nuevo de `xtask`, para que sea el mismo comando en local y en la CI, que es el precedente de F2-1 (`xtask/src/main.rs`: «el mismo en local y en la CI»). `ci.yml` no se toca.

"Firmados" se entiende como lo decidio ADR F1-14 (D4a, decision 6 y divergencia 4): `SHA256SUMS`, attestation de procedencia de GitHub (Sigstore) y firma ad hoc en macOS. Authenticode y Developer ID con notarizacion **no** entran: llegan con winget y Homebrew. Por eso no hace falta ningun secreto, solo `GITHUB_TOKEN` y OIDC.

Piezas, en orden de construccion:

1. **Se extiende `xtask`** con `cargo xtask empaquetar --target <triple> --salida <dir>` y `cargo xtask sumas <dir>`. Hoy ese fichero solo acepta `verificar --perfil …`.
   - `empaquetar` compila con `cargo build --release --locked -p tc2 --target <triple>` y mantiene la guarda contra `RUSTFLAGS`.
   - Copia el binario con el nombre del contrato de `cli.lock` (ADR F1-14, punto 4): `tc2-<version>-<alias>[.exe]`.
   - El alias es uno de los 6 de P12: win-x64, win-arm64, darwin-x64, darwin-arm64, linux-x64, linux-arm64.
   - `sumas` escribe `SHA256SUMS` en el formato de `sha256sum`, reutilizando `xtask/src/sha256.rs`. Asi el formato vive en un solo sitio y luego lo consumen F2-11 (TASK-026) y F2-12 (TASK-027).
   - La tabla alias↔triple va en un modulo nuevo `xtask/src/dist.rs`, para no engordar `main.rs`.
2. **Job `build`** (matriz de 6, `permissions: contents: read`): cada target compila en un runner nativo de su sistema, como pide ADR F1-14, punto 3.
   - Ejecuta `rustup target add <triple>` solo en la CI. **No** se anaden `targets` a `rust-toolchain.toml`, porque obligaria a cualquier `cargo` local a descargarlos.
   - En macOS hace `codesign --force --sign -` y `codesign --verify`.
   - Sube el binario como artefacto.
   - Linux con **musl estatico** (ver decisiones).
3. **Job `humo`** (matriz de los 6, `needs: build`, sin toolchain ni checkout): descarga el artefacto y comprueba su sha256 contra el `SHA256SUMS` de los artefactos.
   - Saca `node` del PATH y exige que `node` **no** se encuentre, igual que la PoC S de F1-14. Los runners de GitHub traen Node de serie, asi que sin este paso la prueba no demostraria nada.
   - En Linux corre dentro de un contenedor minimo (`alpine` o `debian:stable-slim`): es la maquina mas limpia y ademas prueba que el binario musl no depende de la glibc.
   - Ejecuta `tc2 --version` y exige `tc2 <version>` igual al tag y codigo 0. Tambien ejecuta `tc2` sin argumentos y exige el texto de uso.
   - Este job demuestra el criterio 1.
4. **Job `publicar`** (ubuntu, `needs: [build, humo]`, solo con tag): es el unico job con `contents: write`, `id-token: write` y `attestations: write`, y no ejecuta codigo del repo mas alla de `xtask sumas`.
   - Pasos: `xtask sumas`, luego `actions/attest-build-provenance` sobre los 6 binarios, luego `gh release create` con los 6 binarios y `SHA256SUMS`.
   - Comprueba que el tag es igual a `v` mas la version de `Cargo.toml`, o a eso con sufijo `-…`, que se marca como prerelease.
5. **Job `verificar-release`** (6 runners, tras `publicar`): descarga desde la **URL publica** de la release, sin credenciales (P12: repo publico), comprueba `SHA256SUMS` y `gh attestation verify` y vuelve a lanzar `tc2 --version`. Cierra el «publicados en cada release» con la ruta real.

Todas las actions nuevas se fijan por SHA, igual que `actions/checkout` en `ci.yml`. Cada job es de una sola clase (ADR F1-8, «Dos clases de jobs»): ninguno compila y publica a la vez.

Duracion estimada: 170 min (de la rama abierta al cierre con merge)

- Senales: unos 5 ficheros (`release.yml` nuevo, `xtask/src/main.rs` y `xtask/src/dist.rs`, `docs/` con una pagina corta de como se publica y como se verifica, la fila F2-10 del plan si la persona lo aprueba).
- 3 capas: xtask en Rust, workflow, documentacion.
- 2 criterios, uno de ellos solo verificable en CI real.
- Lo que mas pesa es iterar en CI real: 6 runners por pasada y unas 3 o 4 pasadas previsibles hasta tener los runners ARM e Intel y musl en verde.

## Orden interno de implementacion (no se trocea)

La persona decidio **no trocear** la tarea pese a los 170 min estimados (respuesta del 2026-11-16, sin motivo adicional). Las tres partes de abajo se mantienen como orden interno de implementacion dentro de una sola rama.

### Paso a: xtask empaquetar y sumas (local, unos 45 min)

- Subcomandos, tabla alias↔triple, nombre de fichero y `SHA256SUMS`, con tests unitarios.
- Se prueba entero en local en win-x64, el unico target instalado (ADR F1-14, inventario I).
- Entregable que se puede mergear solo: no cambia nada de la CI.

### Paso b: release.yml con build y humo en modo ensayo (CI real, unos 70 min)

- `workflow_dispatch` sin publicar: matriz de 6, firma ad hoc en macOS y humo sin Node en los tres sistemas.
- Al cerrar esta parte el criterio 1 queda demostrado con el run de evidencia.

### Paso c: publicar, attestation y verificar-release (CI real, unos 55 min)

- Job `publicar` y job `verificar-release`.
- Una release de prueba con tag de prerelease, si la persona la autoriza (ver decisiones).

## Lo que el rol no cubrio

- **Pruebas** (rol de testing): el plan de pruebas de abajo lo esboza la arquitectura y no lo ha revisado nadie con ese punto de vista.
- **Modos de fallo** (rol de riesgos), sin revisar:
  - tag con version que no casa;
  - un target que falla y deja la release a medias;
  - re-ejecutar un job de publicar ya hecho;
  - MOTW y SmartScreen al descargar;
  - Gatekeeper con un binario bajado por `curl`.
- **Dominio y reglas**: no se ha revisado si «arranca» basta con `--version` y el texto de uso, o si debe ejecutar un subcomando real (`tc2 new` en un repo git temporal). Se anota como suposicion.
- Frente a los criterios:
  - El criterio 1 se cubre con el job `humo` en runners de GitHub con Node quitado del PATH. Esos runners **no** son maquinas limpias de verdad. La prueba en maquinas limpias reales es de TASK-071 (F9-9a).
  - El criterio 2 se cubre en las piezas: build por sistema, «firmados» segun D4a, hash y publicacion. Que «firmados» quede asi depende de una decision de persona.
- **Queda fuera porque es de otra tarea**:
  - instaladores `install.ps1` e `install.sh` (TASK-026, F2-11);
  - `cli.lock`, `tc2 version --json` con protocolo, y `actualizar --cli` (TASK-027, F2-12);
  - publicar la 2.0.0, changelog y subida de version en `Cargo.toml` (TASK-072);
  - workflow reutilizable de verificacion para otros proyectos (TASK-041, F5-1);
  - npm, winget y Homebrew (D5a: al ser estable).

## Riesgos aceptados y que los contiene

- **Reproducibilidad entre maquinas, no garantizada.** TASK-016 dejo escrito que el sha256 del runner y el local difieren, y aplazo `--remap-path-prefix` a esta tarea. Se acepta porque el hash de referencia es **siempre** el `SHA256SUMS` que produce la CI. Lo que lo contiene: `cli.lock` (TASK-027) se rellena desde ese fichero y nunca desde un build local, y la pagina de docs lo dice. `--remap-path-prefix` sigue sin aplicarse; queda como MENOR anotado.
- **Sin Authenticode ni Developer ID** (D4a). Windows mostrara SmartScreen y macOS pondra en cuarentena lo que se baje con navegador. Lo contienen el hash y la attestation, y la decision de negocio de Carlos al entrar en winget y Homebrew (ADR F1-14, «Consecuencias»).
- **"Limpio" significa runner de GitHub con Node fuera del PATH**, mas un contenedor minimo en Linux, no una maquina virgen. Lo contienen la comprobacion explicita de que `node` no se encuentra, en la linea de la PoC S, y TASK-071.
- **La compilacion de `ring`** (via `ureq` con `rustls`, `crates/tc2/Cargo.toml`) necesita un compilador C en cada runner, y en musl `musl-tools`. Lo contiene que, si un target no compila, ese job falla y `publicar` no corre: no sale una release con menos de 6 binarios.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- **Criterio 1 (arranca en Windows, macOS y Linux sin Node)**: 0 tests unitarios nuevos.
  - La evidencia es el run del job `humo` en sus 6 celdas: comprobacion de que `node` no esta, `tc2 --version` igual al tag y codigo 0, y sha256 contra `SHA256SUMS`.
  - Es pesado y **solo verificable en CI real**: runner de macOS, runners ARM, contenedor Linux.
  - En local solo se puede repetir la celda win-x64, quitando `node` del PATH como en la PoC S.
  - El run ID y el estado de cada celda van al `## Resultado`.
- **Criterio 2 (F2-10: por sistema, firmados segun D4a, con hash, publicados)**: 3 tests nuevos en xtask, ligeros y locales:
  1. Tabla alias↔triple completa con los 6 de P12, y nombre de fichero con `.exe` solo en Windows.
  2. Formato de `SHA256SUMS` (linea `<64 hex>  <fichero>`, orden estable), con un vector conocido.
  3. Validacion tag↔version (`v0.0.0` acepta; `v0.0.0-prueba.1` acepta como prerelease; `v0.0.1` rechaza).
- **Criterio 2, comprobaciones pesadas y solo en CI real**:
  - `codesign --verify` en macOS, dentro del job `build`.
  - Release de prueba con `gh attestation verify` y descarga anonima desde la URL publica, en el job `verificar-release`.
- **Regresion**: `cargo xtask verificar --perfil rapida` y `ci.yml` siguen en verde.

## Lo que necesita decision de una persona

1. **Que significa "firmados" en esta tarea.** Recomendado: lo de D4a (attestation, `SHA256SUMS` y firma ad hoc en macOS) y actualizar el texto de la fila F2-10 del plan con la divergencia 4 de ADR F1-14, que se propuso y esta sin aplicar: la linea 337 de `docs/plan/PLAN-TASKCODE-2.md` sigue diciendo «firmados». Si se quiere Authenticode o Developer ID ya, hacen falta certificado OV o una organizacion (Azure Artifact Signing no sirve a una persona fisica en Espana, segun ADR F1-14, punto 6) y los 99 USD/ano del Apple Developer Program. Los secretos los aporta Carlos y la tarea crece.
2. **Linux: musl o glibc** (ADR F1-14, punto 3, lo deja a F2-10). Recomendado musl estatico: un binario construido en `ubuntu-latest` contra glibc exigiria una glibc tan nueva como la del runner y no arrancaria en distribuciones mas viejas.
3. **Release de prueba.** ADR F1-3 D3a pone los tags sobre el merge en `main`, que tiene 0 tags. Para verificar `publicar` y la attestation hace falta un tag real. Opciones:
   - (a) tag de prerelease `v0.0.0-prueba.1`, y luego borrar tag y release;
   - (b) dejar la parte c verificada solo hasta el ensayo y estrenarla en TASK-072.

   Recomendado (a), con permiso explicito porque toca el repo publico.
4. **Runners.** Hay que aceptar `windows-11-arm` y `ubuntu-24.04-arm` (gratis en repos publicos, sin verificar) y elegir el de darwin-x64: runner Intel nativo si sigue disponible, o compilacion cruzada desde `macos-latest`. En este ultimo caso el humo de darwin-x64 dependeria de Rosetta en el runner, sin verificar, y la celda podria quedar «no verificado».
5. **Alcance de "arranca".** Basta `tc2 --version` mas el texto de uso, o ademas un subcomando real (`tc2 new --json` en un repo git temporal, que ejercita YAML y `git`, la unica dependencia externa segun P12). Recomendado lo primero por ser una PoC minima.

### Respuestas de la persona (2026-11-16)

- **Troceo**: no se trocea; una sola tarea de unos 170 min.
- **1. Firmados**: lo de D4a (SHA256SUMS, attestation Sigstore, firma ad hoc en macOS), sin secretos. Se actualiza la fila F2-10 de `docs/plan/PLAN-TASKCODE-2.md` con la divergencia 4 de ADR F1-14.
- **2. Linux**: musl estatico.
- **3. Release de prueba**: opcion (a), tag de prerelease `v0.0.0-prueba.1` y su release, borrados despues. Antes de crear el tag y antes de borrarlo se pide confirmacion explicita a la persona en ese momento.
- **4. Runners**: `windows-11-arm` y `ubuntu-24.04-arm`; darwin-x64 en runner Intel nativo si existe y, si no, compilacion cruzada desde `macos-latest`, con la celda de humo marcada «no verificado» si Rosetta no la cubre.
- **5. Arranca**: `tc2 --version` mas el texto de uso.

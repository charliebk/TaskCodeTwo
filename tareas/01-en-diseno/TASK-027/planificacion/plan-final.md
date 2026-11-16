# Plan — TASK-027: F2-12 tc2 actualizar --cli y aviso de version nueva

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 240 min

Todo del lado del CLI, en tres capas: una caja comun de distribucion (targets, nombre de fichero, sha256, `SHA256SUMS`) compartida con `xtask`; un modulo de descarga separado del puerto Plataforma; y las ordenes `tc2 version --json`, `tc2 compatible` y `tc2 actualizar --cli [--comprobar] [--version X]`. La version disponible sale de `releases/latest/download/SHA256SUMS` (la misma fuente que los instaladores de F2-11; una peticion, sin el limite de 60/h de la API). En Windows la sustitucion es por renombrados (actual → `.old`, nuevo → su sitio) con vuelta atras.

Piezas:
- `crates/distribucion` (nueva, caja hoja): `Target`/`TARGETS`/`fichero()` de `xtask/src/dist.rs`, `xtask/src/sha256.rs` y lectura de `SHA256SUMS`. `xtask` y `tc2` dependen de ella.
- `tc2::plataforma::http`: solo se extrae el constructor del agente ureq (TLS del sistema, user-agent). `Cliente` no cambia (contrato F1-13: sin redirecciones, 1 MiB, rutas cerradas).
- `tc2::distribucion::descarga` (nuevo): cuerpo binario con limite propio, redirecciones solo a una lista cerrada de hosts (github.com y sus hosts de assets), escritura en `<dir>/.tc2-<v>.descarga`.
- `tc2::actualizar_cli` (nuevo): `detectar_modo(current_exe)` → `Instalador | Plugin | Desconocido`; `comparar(actual, disponible)` con semver y prerelease; `sustituir(viejo, nuevo)` (sha256 comprobado por el proceso viejo sobre el fichero nuevo, `nuevo version --json`, renombrados y vuelta atras; el `.old` lo limpia la siguiente ejecucion).
- `lib.rs`: `pub const PROTOCOLO: u32 = 1`; ordenes `version`, `compatible`, `actualizar` (sin `--cli` da `uso` hasta F9-2), todas con su `Informe` por `contrato`.
- `release.yml` y `probar-instalador.sh`: job de ensayo que instala vX y actualiza a vY con el servidor 127.0.0.1 de F2-11.
- `cargo xtask cli-lock` para que el futuro lanzador del plugin fije la version del CLI.

Orden de construccion:
1. Caja `distribucion` y `xtask` usandola sin cambiar comportamiento (mismos sha256).
2. `PROTOCOLO`, `version --json`, `compatible` (sin red).
3. `descarga` y `actualizar --cli --comprobar` (solo lectura).
4. `actualizar --cli` con sustitucion, vuelta atras y limpieza de `.old`.
5. Job de CI, `cargo xtask cli-lock` y `docs/distribucion/instalar.md` (quitar «es de F2-12»).

Alternativas descartadas: pedir la version a la API `releases/latest` con el `Cliente` actual (limite 60/h, redireccion igualmente necesaria y segunda fuente de verdad); aviso pasivo con consulta de red en cada orden o en la status line (gasta el presupuesto de RR-19 y mete red en el camino caliente).

## Lo que el rol no cubrio

- No hubo roles de riesgos, testing ni dominio: carreras entre dos `actualizar` a la vez, disco lleno o antivirus que bloquea el renombrado no tienen revision adversarial; la revision por pares debe mirarlos.
- Reglas de dominio de «version nueva» con prereleases y redaccion de mensajes: sin especialista.
- Suposiciones sin verificar: que `releases/latest/download/X` redirige a un host de assets estable (no hay release estable aun); que un `.exe` descargado por ureq no lleva MOTW; que la caja nueva no rompe la reproducibilidad de `xtask` (`/Brepro`); lectura de F1-14 punto 4 («actualizar --cli la hace el binario nuevo sobre el viejo») como «la comprobacion la hace un proceso distinto del comprobado».

## Riesgos aceptados y que los contiene

- Red caida o lenta: tiempos de `http.rs` (30 s total, 10 s conectar) y `sinred` con la URL; nada se toca hasta que el hash casa.
- Windows: un `.exe` en ejecucion se renombra pero no se sobrescribe; si otro proceso lo tiene abierto se rechaza y se vuelve atras; sin permisos de escritura se rechaza antes de descargar, con la ruta.
- Integridad: sha256 de `SHA256SUMS` de la misma release, como el instalador; sin Authenticode (D4a). Attestation solo si hay `gh` (paridad con F2-11). Riesgo aceptado: comprometer la release compromete binario y `SHA256SUMS` a la vez, igual que en el instalador.
- Coste: comprobar ~600 B; actualizar ~3 MiB; sin estado nuevo.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):
- C1: `crates/tc2/tests/actualizar_cli.rs` contra servidor local vX → vY; job de CI (no verificable sin push).
- C2: mismo test con binario alterado: sha256 instalado igual antes y despues, sin `.descarga` ni `.old`.
- C3: servidor apagado → `sinred`; unitarios de `comparar` (prerelease < estable, igual → `sin-cambios`).
- C4 y C5: unitarios en `lib.rs`, tambien en ingles con el mismo stdout y codigo.
- C6: unitario de `detectar_modo` con rutas de cache de los tres sistemas.
- C7: `cargo xtask verificar` en verde y mismos sha256 del ensayo de release.
- Vuelta atras: fallo inyectado en el segundo renombrado deja el original intacto.

## Lo que necesita decision de una persona

Carlos acepta las recomendaciones (consigna del 2026-11-16, «aceptamos todas las respuestas de forma recomendada»):
1. El lanzador del plugin (`tc2`, `tc2.cmd`, `tc2.ps1`) y su uso de `cli.lock` van en una tarea nueva; TASK-027 entrega `version --json`, `compatible` y `cargo xtask cli-lock`.
2. El aviso de version nueva es solo la orden explicita `--comprobar` (la usan un hook SessionStart opcional y `doctor`), sin consulta pasiva con TTL.
3. Incompatibilidad con `rechazada` (3) y `motivo=` (precedente F1-18 punto 11).
4. Attestation opcional si hay `gh`.
5. La fila F2-12 del plan dice `tc2`, no `taskctl` (ADR F1-14, divergencia 5): corregida.

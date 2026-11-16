# Plan — TASK-116: Nombres de la 2.0 alineados con F1-12 D1a

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 120 min

Modulo nuevo `crates/tc2/src/huella.rs` como unica fuente de los nombres que la 2.0 deja en un repo (rama, refs, carpeta en el directorio comun de git, worktrees); `estado` y `git` reexportan desde el, y TASK-078 lo usa como lista blanca de «lo mio».

Mapa de nombres:
- `taskcode/tareas` → `taskcodetwo/tareas`
- `.git/taskcode-estado.lock` → `.git/taskcodetwo/estado.lock`
- `.git/tc2/identidad.yml` → `.git/taskcodetwo/identidad.yml`
- `refs/taskcode/{pendientes,rechazadas}/` → `refs/taskcodetwo/...`
- `.taskcode/worktrees` → `.taskcodetwo/worktrees` (excluida por `info/exclude` como hoy)
- `.taskcode/config.yml`, `personas.yml` y `vocabulario` no se mueven: F1-12 D4a fija una config compartida.

Piezas:
- `src/huella.rs` (nuevo): `CARPETA_COMUN="taskcodetwo"`, `RAMA_ESTADO`, `PREFIJO_REFS="refs/taskcodetwo/"`, `CARPETA_WORKTREES=".taskcodetwo/worktrees"` y `fn dir_propio(comun: &Path) -> io::Result<PathBuf>` que crea la carpeta.
- `estado/mod.rs` (`RAMA`, `REFS_PENDIENTES`, `REFS_RECHAZADAS`) y `git/mod.rs::CARPETA_WORKTREES`: `pub use`/derivados de `huella` (API e imports intactos).
- `estado/cerrojo.rs`: abre `dir_propio(comun)?.join("estado.lock")` (hoy `OpenOptions::create` no crea el padre).
- `identidad.rs`: `Cache::en` usa `dir_propio`; desaparece `CARPETA_CACHE="tc2"`.
- LEEME de `estado/montaje.rs` compuesto desde la constante; docs de modulo actualizados.
- Tests: importan las constantes de `tc2`; los literales de salidas esperadas se actualizan a mano porque fijan el contrato.
- Docs: adenda unica que amplia la «Nota de nombres» de F1-12 con la tabla vieja → nueva, y una linea de adenda en la cabecera de F1-2, F1-3 y F1-4 que apunte a ella; `docs/plan/PLAN-TASKCODE-2.md` corregido directamente. F1-19 esta descartado: no se toca.

Orden de construccion:
1. `huella.rs` y reexports sin cambiar valores (refactor puro en verde) + adendas.
2. Cambiar valores, cerrojo e identidad a `dir_propio`, LEEME desde la constante (C1, C2, C5).
3. Test de que `CARPETA_WORKTREES` no empieza por `.taskcode/` ni `.git/taskcode` (C3).
4. `tests/convivencia_016.rs`: fixture con `.git/taskcode/`, `.taskcode/config.yml` y `tareas/` en la historia de `develop`; huella (hashes y SHA) igual tras `alta` y `crear_rama_y_worktree`, y todo lo nuevo bajo los prefijos de `huella.rs` (C4).
5. `cargo xtask verificar --perfil rapida` (C6).

Alternativas descartadas: cambiar los literales en su sitio sin modulo (TASK-078 tendria que reunir 5 nombres de 4 modulos); worktrees en `.git/taskcodetwo/worktrees` (contradice F1-3 D2a «dentro del proyecto»).

## Lo que el rol no cubrio

- Sin roles de riesgos, testing ni dominio: literales de prueba que no se actualicen; procesos de la 2.0 en marcha durante el cambio de nombre del cerrojo.
- Suposiciones sin verificar: que la 2.0 no este desplegada en ningun repo real (solo comprobado en este); que la 0.16 no use `refs/taskcode/` ni nada bajo `.taskcodetwo`, y donde guarda la 0.16 su estado exactamente (para el fixture de C4).

## Riesgos aceptados y que los contiene

- No hay migracion: la 2.0 no esta desplegada; la implementacion lo comprueba en OpenDockTrading y ToolsaWeb (solo lectura) y lo anota.
- Nombres sueltos en el futuro: `huella.rs` es la unica fuente y C3/C4 lo vigilan.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):
- C1: `alta` y `estado_*` con la `RAMA` importada.
- C2: unitario de que cerrojo e identidad escriben en `<comun>/taskcodetwo/` + grep vacio en `src`.
- C3: unitario sobre `CARPETA_WORKTREES`.
- C4: `tests/convivencia_016.rs`.
- C5: unitario de prefijos de refs desde `huella`.
- C6: verificacion rapida.

## Lo que necesita decision de una persona

Carlos acepta las recomendaciones (consigna del 2026-11-16):
- Worktrees en `.taskcodetwo/worktrees`.
- Se incluyen las refs `refs/taskcode/` → `refs/taskcodetwo/` aunque el enunciado no las nombraba (criterio C5 anadido).
- Cerrojo como `.git/taskcodetwo/estado.lock`.
- Adenda unica en F1-12 en vez de reescribir cada ADR.

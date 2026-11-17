# Plan — TASK-080: F3-9 inventario de skills y agentes y doctor --recursos

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 300 min

Leido por el rol: F3-9 (PLAN, linea 358), RF-14, RR-17, AMPLIACION-2026-11-14 §F3-9, F1-15 (decisiones 2, 8, 9, 11; divergencias 2 y 9), F1-14 decision 8 con su adenda; `doctor/mod.rs`, `doctor/entorno.rs`, `instalacion.rs` (`dir_claude()` respeta `CLAUDE_CONFIG_DIR`), `desinstalar/maquina.rs` (`instalados()` lee `plugins/installed_plugins.json`), `config/claves.rs` y `config/validar.rs` (`recursos:` ya existe: `id`, `tipo` skill/agente/ejecutable, `rol`, `dominios`), `Rol`/`RolRevision` de `tarea/modelo.rs`. En la maquina: `installed_plugins.json` version 2 con `installPath` por `nombre@marketplace`, `skills/<n>/SKILL.md` y `agents/<n>.md`, `~/.claude/skills|agents`, `enabledPlugins` (con `false`) en `~/.claude/settings.json`.

En la 2.0 no hay `catalogo-skills.yml` (F1-15 D8a). Lo que puede nombrar algo no instalado: (a) `recursos:` de la configuracion, (b) `roles.<rol>.skills` y `roles.revision.agente` de las tareas abiertas, (c) el suelo obligatorio del binario (F1-14 punto 8).

Orden:
1. `crates/tc2/src/inventario.rs` (nuevo, de primer nivel: lo usaran `init`, `start` y `seleccionar`): `leer(dir_claude, raiz) -> (Inventario, Vec<FuenteIlegible>)`; `Entrada { id, tipo: Skill|Agente, origen: Plugin{nombre,version}|Usuario|Proyecto|Integrado }`; `Inventario::contiene` y `huella()` (12 hex, para `roles.<rol>.inventario` de F1-15). Fuentes: plugins habilitados de `installed_plugins.json` (id `plugin:nombre`), `<dir_claude>/skills|agents`, `<raiz>/.claude/skills|agents` y `general-purpose`. Orden (tipo, id), sin duplicados.
2. Extraer a `inventario` la lectura de `installed_plugins.json` de `desinstalar::maquina::instalados`; `desinstalar` la reutiliza.
3. `crates/tc2/src/recursos.rs` (nuevo): `evaluar_recursos(inv, manifiesto, momento) -> InformeRecursos` puro, contrato de F1-14 punto 8 (obligatorio, necesario, conveniente, estorba), con su propio `Recurso { id, nivel, estado, comprobacion, arreglo }` (no depende de `doctor`).
4. `doctor`: `--recursos`; adaptador `doctor/recursos.rs` de `Recurso` a `Hallazgo` (area `recursos`); primera linea con `skills=N agentes=N inventario=<huella>` sin `plataforma=`; «y N mas» con la orden lanzada; el `doctor` normal anade un hallazgo agregado `recursos no-instalados falla ids=N arreglo=tc2 doctor --recursos`; `USO_DOCTOR` y el test `argumentos_de_mas_son_uso`.
5. Documentacion: `instalar.md` y adenda en F1-14 (`--recursos` llega con TASK-080).

Descartado: inventario dentro de `doctor/` (dependencia al reves); seguir leyendo `catalogo-skills.yml` (F1-15 D8a); que `recursos` devuelva `Hallazgo` (ata `init`/`start` a `doctor`).

Coste: ~100 cabeceras de ficheros pequenos por llamada (53 skills y 46 agentes en esta maquina segun la PoC I de F1-15); sin cache.

## Lo que el rol no cubrio

- Riesgos: `.claude/` del proyecto no es confiable (un PR que cuela una skill), enlaces simbolicos o rutas fuera de `installPath`, ficheros enormes, codificaciones raras.
- Testing: solo las fixtures de C1-C11.
- Dominio: que son exactamente las «skills de fase» obligatorias, que estados cuentan como «tarea abierta», si un plugin deshabilitado cuenta como no instalado (D2).
- Suposiciones a verificar: el prefijo `plugin:skill` de Claude Code (tomado de F1-15, no de su documentacion); `enabledPlugins` tambien en `.claude/settings.json` y `settings.local.json` del proyecto y su precedencia; `projectPath` en entradas con `scope` de proyecto; lectura reutilizable de tareas abiertas en `estado::lectura` o `tablero`; crate de sha2 en `Cargo.toml`.

## Riesgos aceptados y que los contiene

- El inventario del proyecto se lee del sistema de ficheros, no de la base; la comprobacion con la base es de F6-3 (F1-15 decision 2).
- Rutas de Linux y macOS y plugins con `scope` de proyecto o local sin verificar (aqui no hay).
- Deteccion aproximada de engram y context7 (MCP): solo nivel conveniente, que avisa y sigue.
- `skills`/`agents` con ruta propia en `plugin.json` no se leen; solo la estructura estandar.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- Unitarias de `inventario::leer` y `evaluar_recursos` con temporales y `instalacion::Entorno` inyectado: C1, C2, C3, C8, C9, C10.
- Adaptador y lineas: C4, C5, C11.
- Integracion con el binario sobre un repo temporal: C6, C7.
- `cargo test`, `clippy`, `fmt`; ejecucion manual de solo lectura en esta maquina contrastada con la tabla I de F1-15.

## Lo que necesita decision de una persona

Carlos dejo dicho que en el H3 se acepta la opcion recomendada y que no se trocea; se aplican:

- D1 que cuenta como recomendacion: (a) `recursos:` mas `roles.*` de las tareas no terminadas. **Elegida.**
- D2 plugin deshabilitado en `enabledPlugins`: (a) no cuenta como instalado. **Elegida.**
- D3 mientras no exista el plugin `taskcodetwo`: (a) `falla` (honesto, aunque de 3 en todas las maquinas hasta publicarlo). **Elegida.**
- D4 lista completa: (a) solo en `--json`, cuentas en el texto. **Elegida.**
- D5 ids del inventario: (a) `name:` del frontmatter y, si falta, el nombre de carpeta o fichero, con aviso si no coinciden. **Elegida.**

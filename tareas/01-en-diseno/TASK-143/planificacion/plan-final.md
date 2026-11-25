# Plan — TASK-143: Auditoria 4D: A5 seleccion, inventario, recursos y doctor (6 IMPORTANTE)

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 240 min

Señales: 4 ficheros de producción (`inventario.rs`, `seleccion/catalogo.rs`, `seleccion/orden.rs`, `seleccion/mod.rs`) y 3 de test (`tests/doctor.rs`, `tests/recursos.rs`, `tests/comun/lanzador.rs`); 2 capas; 6 criterios; dos pases largos de verificación y una ronda de revisión.

**Las 6 fichas siguen vivas (develop bfe11a5):** A5-01 (`catalogo.rs:220-258` ~ `inventario.rs:365-473`), A5-02 (`catalogo.rs:265-298` ~ `inventario.rs:804-908`), A5-03 (`seleccion/mod.rs:260-267`, `:344-351`), A5-06 (`seleccion/mod.rs:849`; mutante en `tabla.rs:170`), A5-08 (`tests/doctor.rs:951-958` ~ `tests/recursos.rs:371-386`), A5-09 (`tests/doctor.rs:1000-1009` ~ `doctor_contra` `:509-524`; tercera copia del cierre en `:1082-1087`). A5-07 lo cerró TASK-148.

**Orden** (un commit por ID):
1. A5-01: en `inventario`, `pub(crate) struct Leida { entrada, ruta, cabecera }` y `pub(crate) fn skills_de(dir, prefijo, origen, ilegibles, avisos) -> Vec<Leida>`; `hijos` y `nombre_utf8` reciben `&mut Vec<Incidencia>`; el final común queda en una privada `leer_una` (skills y agentes); `catalogo::de_carpeta` llama a `skills_de` y aplica `recurso_de` sobre `Leida::cabecera` sin releer; se borran `catalogo::cabecera` y su `use std::io::Read`; `catalogo::Lectura` gana `ilegibles` y `avisos`; `pub(crate) fn nombre_de(cabecera, carpeta)` quita la tercera copia del id (en `de_base`). **Cambia:** en `orden::fuentes`, rama `--plugin`, las incidencias ilegibles pasan a `Fuentes.avisos` como `ilegible:<motivo>:<id>` (hoy desaparecen sin rastro).
2. A5-02: `inventario::Plugin` gana `pub carpeta: Option<PathBuf>` (la instalación que eligió `aplica`); el `omitir: bool` de `cargar_plugin` pasa a un enum privado `Fuente { Instalado, Sincronizado }` (sincronizado deja `carpeta: None`); `catalogo::de_instalados(inv)` pierde `dir_claude` y lee de `p.carpeta`. **Cambia:** desaparece la elección propia por `version`/«primera con carpeta»: las revisoras salen de la instalación que el inventario dio por habilitada para el `scope` del proyecto. `Inventario::huella` no incluye `carpeta` (depende de la máquina).
3. A5-03: `revisoras_de` filtra también por `self.instalado(&r.id)`. **Cambia:** con una revisora de dominio declarada y no instalada, deja de aplicar S10 y entra S11 con `dominio-sin-revisora:<d>`; ya no salen S14 ni `no-instalado:<id>` por ella (filas de revisión).
4. A5-06 (test primero): reforzar `las_reglas_se_leen_de_la_tabla` sin `unwrap`/`expect` (`assert!(r.is_ok(), "{r:?}")`, `unwrap_or_default`, y `s.reglas` contiene `M04`): verde hoy, rojo con el mutante de la ficha y con el control (`mod.rs:556`).
5. A5-08 y A5-09: `pub fn salida_acotada(lineas, json, detalle)` en `tests/comun/lanzador.rs` para los dos tests; `doctor_contra_en(f, clon, prov, caso)` que usan `doctor_contra` y `doctor_no_escribe_nada`; `conectar(prov, base)` también para el test sin red (`:1082`).

## Reutilizacion (Regla 4D)

Lo que ya existe y se usa (helpers, funciones, fixtures, tests), buscado en el repo antes de disenar; y lo que se crea nuevo, con el motivo de no reutilizar:

- `inventario::{hijos, nombre_utf8, cabecera, nombre_del_frontmatter, aplica, MAX_CABECERA}` (única lectura); `catalogo::recurso_de`; `Fuentes.avisos` (ya llega a la salida); el precedente de carpeta ilegible portátil (`inventario.rs:1390`); `Tmp` y `skill()` de `inventario::tests`; `catalogo()`, `tres()`, `todos()`; en `tests/doctor.rs` `servir`, `adaptador`, `hallazgos`, `con_estado`, `foto`; `tests/comun/lanzador.rs`.
- Nuevo: `skills_de`/`Leida` (la lógica sale de `leer_carpetas`, que mezcla skills y agentes en un `Acumulado` privado); `nombre_de`; enum `Fuente` (sustituye al `bool`); `salida_acotada`, `doctor_contra_en`, `conectar`.
- Descartado: `crate::lectura::fichero` (UTF-8 estricto, rechaza vacío y tope; la cabecera trunca con `from_utf8_lossy`); `comun/adaptadores.rs::servir` (sirve otra cosa).
- API pública (`tc2::inventario` y `tc2::seleccion::catalogo` son `pub mod`): campo nuevo `Plugin::carpeta` (rompe el literal del test `recursos.rs:343`), nueva firma de `de_instalados` (único llamador `orden.rs:124`), campos nuevos en `catalogo::Lectura`.

## Lo que el rol no cubrio

- Riesgos: permiso denegado real en Windows frente al sustituto «fichero en lugar de carpeta»; carreras.
- Testing: suficiencia más allá del presupuesto.
- Dominio: si S11 debe añadir también la genérica (F1-15:553; hoy la añade S09); cómo cuenta F8-6 el nuevo `dominio-sin-revisora`; si `nombre-distinto` del catálogo debería salir a la salida.

## Riesgos aceptados y que los contiene

- Sin `--plugin`, las cabeceras de skills se leen dos veces (como hoy); unificarlo obligaría al inventario a conocer el catálogo (contra la adenda F6-3).
- `Plugin` deriva `PartialEq`/`Ord` con `carpeta`: inventarios de máquinas distintas dejan de ser iguales aunque la huella coincida; lo contiene la suite.
- A5-03 puede mover resultados esperados de `tests/fixtures/seleccion-taskcodetwo`; si se mueven, es lo que exige el ADR y se documenta.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- A5-01 (2): unitario en `inventario::tests` (`<p>/skills` como fichero → `ilegibles == [p:skills no-se-lee <ruta>]`; con carpeta legible, mismas entradas que `inventario::leer`); E2E en `tests/revisoras.rs` (`--plugin p=<dir>` con `skills` ilegible → `ilegible:no-se-lee:p:skills` en `avisos`; pesado).
- A5-02 (1): `installed_plugins.json` con `[{scope: project, projectPath: /otro, installPath: A}, {scope: user, installPath: B}]` y revisoras distintas: `carpeta == Some(B)` y `de_instalados` solo da la de B (sin `version` y con la misma `version`).
- A5-03 (2): con `security-review` (dominio `puertas-seguridad`) sin instalar → `dominio-sin-revisora:puertas-seguridad`, sin `no-instalado`, S11 y no S10; instalada → S10 sin aviso.
- A5-06: refuerzo del test existente; verde/rojo/rojo con el mutante y el control, salida en `## Resultado`.
- A5-08, A5-09: refactor de tests; siguen en verde los cuatro tests (pesados).
- Final: `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar`.

## Lo que necesita decision de una persona

1. Formato del aviso: `ilegible:<motivo>:<id>`. Recomendado.
2. Aviso solo en la rama `--plugin` (la rama normal queda como pendiente anotado). Recomendado.
3. `Plugin::carpeta = None` en los sincronizados. Recomendado.
4. Aceptar el cambio observable de A5-03 (S11 con aviso en vez de S10/S14), con una línea en `## Resultado`. Recomendado.
5. Incluir los extras 4D: `nombre_de` en `de_base` y el tercer `conectar`. Recomendado.

**Respuestas de la persona (Carlos, consigna del 2026-11-23: modo automatico con la opcion recomendada):** se aplican las recomendaciones 1 a 5.

**Regla de una hora:** 240 min, sin trocear: Carlos descartó trocear las tareas de corrección 4D (consigna del 2026-11-23).

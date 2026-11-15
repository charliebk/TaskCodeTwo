# Plan — TASK-019: F2-4 Reserva de IDs por push atomico

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 60 min

Señales: 2 ficheros de código (`alta.rs` nuevo, `lib.rs` extendido), 1 fichero de tests E2E, 1 capa (biblioteca + orden del binario), 3 criterios.

La concurrencia ya está resuelta: `estado::transaccion` (crates/tc2/src/estado/transaccion.rs) pone el cerrojo, hace el `fetch`, re-ejecuta la operación ante non-fast-forward y aplica la regla «sin red y con `exige_red` → `sinred pendiente=no`». F2-4 añade **una operación de producción**, `Alta`, y la expone como el `new` mínimo. No se crea un segundo mecanismo, y F3-2 amplía este mismo verbo en vez de sustituir un subcomando provisional.

Qué se crea y qué se extiende:

- **Nuevo `crates/tc2/src/alta.rs`**, módulo de la biblioteca al lado de `estado`, como pide el comentario de lib.rs para las capas siguientes. La dependencia va `alta → estado`, nunca al revés. `estado/` no se toca: su frontera (trait `Operacion`, `Contexto`, `transaccion`) sirve tal cual.
  - `struct Alta { titulo: String, id: Cell<Option<u32>> }` implementa `Operacion`:
    - `nombre() = "new"`.
    - `exige_red() = true`, porque reserva algo compartido (ADR F1-2, D2b).
    - `anotacion() = "new|<titulo>"`, en una sola línea.
  - `aplicar(&dyn Estado)` llama a `siguiente_id(&estado.rutas())`:
    - toma el primer componente de cada ruta que case con `TASK-<dígitos>`;
    - devuelve el **máximo + 1**, no el recuento (F1-2 punto 3, D4a);
    - cuentan también las descartadas y terminadas, porque sus carpetas no se borran nunca (F1-1; F3-7);
    - ignora los nombres que no casan;
    - coste O(rutas): no lee ningún blob.

    Después guarda el ID en la `Cell` y devuelve un único `Cambio::nuevo("TASK-NNN/tarea.md", texto)`.
  - ID con el formato `TASK-{:03}`: crece a 4 dígitos a partir de 1000 sin renumerar.
  - `pub fn desde_anotacion(&str) -> Option<Box<dyn Operacion>>`, para el `Catalogo`.
- **Formato de `tarea.md`**: la plantilla del ADR F1-9, punto 6 (docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md), en una sola constante de `alta.rs` con el comentario «la sustituye el modelo de F2-5». Contiene:
  - `formato: 2`, `id`, y `titulo` entre comillas dobles, con `\` y `"` escapados (F1-9 punto 5);
  - `estado: "planificada"`;
  - `creado` y `actualizado` con la fecha UTC, calculada sin dependencias;
  - `rama: null` (decisión 2);
  - las secciones de la plantilla.
- **Se extiende `crates/tc2/src/lib.rs`**:
  - un brazo `new <titulo> [--json]` y la línea correspondiente en `USO`;
  - `orden_new`, que monta `Contexto::nuevo(&git, &dir, &catalogo)` con `roadmap: None` (el roadmap es de F3-4) y la política por defecto, y llama a `estado::transaccion`;
  - después renombra el informe a `new` y, si salió bien, añade `id=TASK-NNN`, leído de la `Cell` (es el ID del intento ganador);
  - un título vacío, con saltos de línea o con caracteres de control sale con `uso` (2) antes de abrir la transacción;
  - el catálogo de producción vive en lib.rs, la raíz de composición, y delega en `alta::desde_anotacion`.
- **Precedente que se reutiliza**: `CrearUnico` de tests/comun/ops.rs (aquí la ruta se calcula sobre el estado fresco) y el arnés de tests/estado_concurrencia.rs. El E2E lanza el binario real (`CARGO_BIN_EXE_tc2`).

Orden de construcción:

1. `alta.rs`: `siguiente_id`, el escape del título y la plantilla, con sus tests unitarios.
2. `Alta` como `Operacion`, con `desde_anotacion`.
3. El brazo `new` en lib.rs: catálogo, renombrado e `id=`.
4. E2E de dos clones y `cargo xtask verificar --perfil completa`.

Alternativas descartadas:

- **Subcomando interno `reservar-id` sin tarea.md.** Va contra el ADR F1-4 D1a, que pide una superficie mínima. Además, reservar sin crear la carpeta no reserva nada: lo que hace de compare-and-swap es la carpeta publicada.
- **Probar solo con la biblioteca.** No es un E2E de «crear tarea».
- **Ampliar el trait `Operacion` con `datos()`.** Abre la frontera de `estado/` para un solo consumidor. La `Cell` deja el cambio dentro de `alta`.

## Lo que el rol no cubrio

- **C1, modos de fallo sin revisar** (son del rol de riesgos): un título con `|` en la anotación, la carpeta `tareas/` con cambios, carpetas `TASK-` mal formadas a mano, o que no exista la rama de estado.
- **C2, estabilidad del E2E**: sin revisión del rol de testing. No se ha analizado si es estable en la CI con la base de 2 s.
- **C3, reglas de negocio de RF-01** (son del rol de dominio): la validación del tipo, las etiquetas y la complejidad queda para F3-2, y el vocabulario (`new` frente a `nueva`) tampoco se ha revisado.
- **Fuera de alcance a propósito**:
  - F2-4b: rama única contra el remoto y títulos duplicados;
  - F2-5: modelo y parser de tarea.md;
  - F3-4: `ROADMAP.md`;
  - F3-2: `import`, dependencias e hito.
- La reproducibilidad se mide solo en Windows.

## Riesgos aceptados y que los contiene

- **`agotada` (5) con varias altas a la vez.** Lo contiene el contrato: 5 significa «reintenta» y no deja nada escrito. El E2E relanza en ese caso.
- **`creado` en UTC.** Lo sustituye F2-5, que tiene un solo punto donde cambiarlo.
- **`Alta` no se puede reconstruir desde la anotación con cualquier título.** Con `exige_red=true` nunca queda pendiente, así que nunca se re-ejecuta desde la anotación.
- **Sin roadmap hasta F3-4.** `Contexto.roadmap = None`.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- **C1, dos clones crean a la vez → IDs distintos** (1 test, `crates/tc2/tests/alta_concurrencia.rs`):
  - montaje: remoto bare y dos clones, sincronizados con `Barrier`;
  - cada clon ejecuta 3 veces `tc2 new` con el binario real y relanza si sale 5;
  - en el bare: `TASK-001`..`TASK-006` consecutivas, cada `id` igual a su carpeta, cada título una sola vez, 0 merges y nada en `refs/taskcode/`.
- **C2, E2E** (2 tests):
  - (a) `new` en A y después en B sin `fetch`: B recibe `TASK-002` y la salida es `new ok id=TASK-002`.
  - (b) Sin red: sale 4 con `pendiente=no`, sin commit local ni ref pendiente.
- **C3, F2-4 y requisitos** (3 tests unitarios):
  - (a) `siguiente_id`: máximo + 1 contando las descartadas, ignora los nombres que no casan, estado vacío → 1, y `TASK-999` → `TASK-1000`.
  - (b) Plantilla: escape de `\` y `"`, y orden de claves de F1-9.
  - (c) `new` sin título o con un salto de línea → `uso` (2).

Verificación final: `cargo xtask verificar --perfil completa`.

## Lo que necesita decision de una persona

Decidido por Carlos con su consigna del H2: «aceptamos todas las preguntas con la respuesta recomendada».

1. **Exponer ya `new` como orden pública del binario**, que F3-2 completará → **sí**.
2. **Campo `rama` de tarea.md** → **`rama: null`** hasta F2-4b.
3. **Plantilla de F1-9 completa** frente a un frontmatter mínimo → **completa**.
4. **Ancho del ID** → **`TASK-NNN`, que crece a `TASK-1000` sin renumerar**.

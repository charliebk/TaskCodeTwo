# Plan — TASK-033: F3-6 start: toma atomica de la tarea, rama y worktree desde develop

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 300 min

`start` es una orden nueva con dos mitades separadas por el push (F1-2 decision 4, F1-3 decision 2 fila `start`):

1. Una **operacion pura** `Tomar` que pasa por el punto unico de escritura (`estado::transaccion`) y se reejecuta en cada intento.
2. **Fuera de la transaccion y solo si la toma esta publicada**, `git::crear_rama_y_worktree` (tarea libre) o `git::reanudar` (tarea ya mia). No se escribe Git nuevo: todo esta en F2-2 y F2-3.

Piezas:
- `crates/tc2/src/toma/operacion.rs` (nueva): `struct Tomar { id, asignado, decidido_por, fecha }` implementa `estado::Operacion`, `exige_red() = true`, anotacion `start|TASK-NNN|<id>|<login>` y `desde_anotacion` registrada en `catalogo()` de `lib.rs` (como `alta` y `roadmap`). `aplicar` decide solo con el estado fresco: existe, formato 2, estado de origen valido (D1), dependencias cumplidas (D3), `asignado_a` vacio o yo. Escribe `estado: en-curso`, `asignado_a`, `actualizado`, la fila de `## Transiciones` y `rama` si falta. Si ya soy el dueno devuelve `Ok(vec![])` (`sin-cambios`, sin commit).
- `crates/tc2/src/toma/mod.rs` (nueva): `NOMBRE = "start"`, `Entorno { dir, git, quien, fecha }` inyectable (como `tablero::Entorno`) y `pub fn start_en(e, id) -> Informe`, publica para el orquestador F4-4. Pasos: `fetch` + `validar_base`; resolver quien; `comprobar_id` (ls-remote + ramas locales; fallo de red = `sinred`); separar casos; transaccion con `roadmap::generador`; si queda publicada, rama y worktree; si ya es mia, `reanudar` o crear lo que falte.
- `identidad.rs` (se extiende): se saca `pub fn resolver(e) -> Paso<Quien>` de `orden_paso`, y la usan `identidad`, `start` y `tablero::conectar_produccion` (hoy esta repetido) (D6).
- `grafo.rs` (se extiende): predicado `pub fn dependencia_cumplida(dep: Option<&Nodo>) -> bool` sacado de `listas`; `listas` y `Tomar` usan la misma regla. `Tomar` lee solo su `tarea.md` y las de sus dependencias.
- `tarea/cuerpo.rs`: `Cuerpo::anadir_transicion(...)` (hoy nadie escribe la tabla; F1-9 dice que la escribe el CLI).
- `git/rama.rs`: `partes_de_rama(id, rama)`, inversa de `nombre_de_rama`, para crear la rama con el `rama:` del alta.
- `i18n/catalogo/toma.rs` (nueva): mensajes es/en de los rechazos.

Limites: `toma` depende de `estado`, `git`, `identidad`, `grafo` y `tarea`; nadie depende de `toma`. Solo escribe claves de la lista excluida del nucleo del lote (F1-16). El worktree va en `huella::CARPETA_WORKTREES` (`.taskcodetwo/worktrees`, adenda F1-12).

Orden: (1) extracciones sin cambio de comportamiento con las suites de `next`, `roadmap`, `board` e `identidad` en verde; (2) `anadir_transicion` y `partes_de_rama` con unitarios; (3) `Tomar` y registro en `catalogo()`; (4) `start_en`, entrada `start [--json] TASK-NNN` en `ejecutar_en` e i18n; (5) E2E en `tests/toma.rs`.

Descartado: crear rama y worktree dentro de `aplicar` o antes del push (rompe F1-2 D4 y la pureza de la operacion); decidir con `situacion::calcular` (lee todo el arbol y exige `planificada`, en conflicto con D1).

## Lo que el rol no cubrio

- Modos de fallo finos (proceso que muere a mitad, Windows, rutas largas): sin rol de riesgos.
- Suficiencia del plan de pruebas: sin rol de testing.
- Reglas de negocio (estado de origen, `plan_aprobado`, dependencias): sin rol de dominio; van como decisiones abajo.
- Fuera de alcance contrastado con los criterios: manifiesto de recursos de F1-14 (`evaluar_recursos` no existe), aviso de solape (F3-11), liberar y reasignar (F3-10).
- Suposiciones a verificar en la implementacion: que `Tarea::escribir` conserve byte a byte claves desconocidas y cuerpo; que el `motivo` del rechazo de `aplicar` llegue a la linea de `start`; que el fixture de `tests/identidad.rs` sirva sin TTY; que alta e import escriban siempre `rama:` (si no, la escribe `Tomar`).

## Riesgos aceptados y que los contiene

- Proceso muerto entre el push de la toma y la rama: lo contiene la idempotencia (C2); repetir `start` crea lo que falte. Riesgo ya aceptado en F1-2.
- Rama creada a mano entre `comprobar_id` y `worktree add`: la local la detecta `crear_rama_y_worktree`; la remota saldria en `review`. Se acepta.
- Coste O(n) de regenerar el ROADMAP en cada intento: lo fija el diseno de la transaccion, no esta tarea.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- Unitarias: `Tomar::aplicar` sobre `EnMemoria` (C4, C5, dueno, otra persona, fila de transicion); `dependencia_cumplida`; `partes_de_rama` ida y vuelta con `nombre_de_rama`; `anadir_transicion` conserva el resto del cuerpo.
- E2E con `CARGO_BIN_EXE_tc2` en `tests/toma.rs`, C1..C9: C1 con dos clones contra un remoto bare y `Barrier` (precedente `tests/alta.rs`); C3 con `origin` a una ruta inexistente.
- Regresion: suites de `next`, `roadmap`, `board` e `identidad` tras las extracciones.

## Lo que necesita decision de una persona

Carlos dejo dicho que se acepta la opcion recomendada en todas las tareas del H3 (cadena automatica); se aplican:

- D1 estado de origen: (a) `planificada` o `en-diseno`; la puerta de `plan_aprobado` por carril la anade F4-2/F1-10 en el mismo predicado. **Elegida.**
- D2 tarea en formato 1: (a) rechazar con el arreglo `actualizar` (F9-2). **Elegida.**
- D3 dependencias sin cerrar: (a) rechazo con codigo 3. **Elegida.**
- D4 base del `hotfix`: (a) cumplir F1-3 (ultimo tag de `main` o su punta); `release` se rechaza. **Elegida.**
- D5 `--simular`: (a) no en esta tarea. **Elegida.**
- D6 identidad: (a) sacar `identidad::resolver` y compartirla. **Elegida.**

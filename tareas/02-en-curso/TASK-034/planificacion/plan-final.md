# Plan — TASK-034: F3-7 descartar y pausa

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 210 min

Modulo nuevo `crates/tc2/src/ciclo/` con la forma de `toma/`: `mod.rs` con las ordenes `descartar`, `pausa` y `reanudar`; `operacion.rs` con `Descartar` y `Pausar { reanudar: bool }` que implementan `estado::Operacion` como `toma::Tomar`. Cada una lee solo su `tarea.md` y lo reescribe (frontmatter, fila en `## Transiciones` con `Cuerpo::anadir_transicion`, resto byte a byte).

Reutilizacion y extensiones:
- Lectura comun: las ramas no-existe / formato-1 / ilegible de `Tomar::aplicar` (`toma/operacion.rs:124-139`) salen a `tarea_de(estado, id) -> Result<(Tarea, Cuerpo), Fallo>`; sus mensajes pasan de `i18n::catalogo::toma` a `comun`. `Tomar` la usa.
- `descartar`: escribe `estado: descartada`, `motivo`, `actualizado` y fila `descartar`; conserva `asignado_a` y `rama`. La carpeta nunca se borra: `alta::siguiente_id` (`alta.rs:538-551`) toma el maximo de carpetas `TASK-*` y `ids_con_rama`, asi que el ID no se reutiliza. El modelo ya exige `motivo` con `descartada` (`tarea/modelo.rs:556`). Tras publicar, `git::quitar_worktree` (ya se niega con cambios sin guardar o commits sin publicar); la rama no se toca nunca.
- `pausa` / `reanudar`: sin estado nuevo (F1-9 cierra la lista, `ESTADOS` en `modelo.rs:17`). Escriben filas `pausa` / `reanudar` en `## Transiciones` (las que F1-17 usa para descontar pausas). «En pausa» = la ultima de esas filas es `pausa`, calculado por `Cuerpo::en_pausa()`.
- Lectura: `situacion::Ficha` gana `motivo` y `en_pausa`; `situacion::calcular` quita las pausadas de `listas` (`next` y `roadmap` siguen coincidiendo). `grafo::Nodo` no cambia. `roadmap::generar::plegadas` muestra el motivo en `## Descartadas` y las tablas marcan «en pausa». `Tomar` rechaza una tarea en pausa no empezada (`motivo=pausada`, arreglo `tc2 reanudar`).
- Conexion: `lib.rs` con tres ordenes y `ciclo::desde_anotacion` en `catalogo()`. Anotaciones `descartar|TASK|<id>|<login>|<motivo>` (`splitn(5)`), `pausa|...`, `reanudar|...`. `exige_red`: `descartar` true (cambia `next` y dependencias de todos); `pausa`/`reanudar` false (transicion de la tarea propia, D2b). Adendas: F1-2 punto 5 (`descartar` exige red) y F1-4 (lineas y motivos de las tres).

Orden: (1) extraer `tarea_de` sin cambio de comportamiento (`tests/toma.rs` en verde); (2) `Cuerpo::en_pausa`, `Ficha`, filtro de `listas` y `roadmap::generar`; (3) operaciones puras con unitarios y catalogo; (4) ordenes del CLI, `quitar_worktree`, i18n, `lib.rs`, rechazo `pausada` en `Tomar`; (5) E2E y adendas.

Descartado: estado `pausada` (reabre F1-9, la 0.16 no lo lee, pierde la medicion de pausas de F1-17); etiqueta `pausada` (mezcla datos con la seleccion de F1-15 y no guarda cuando).

## Lo que el rol no cubrio

- Riesgos: carrera `descartar` contra `start` simultaneos (en principio la cubre la reejecucion de la transaccion); motivo con `|` o muy largo en la anotacion; ediciones a mano de `## Transiciones`.
- Testing: cobertura mas alla de la lista de abajo.
- Dominio: si un responsable puede descartar tareas ajenas; si la pausa lleva motivo; que hacer con el PR abierto de una tarea en revision descartada.
- Criterios: «visible en el roadmap» queda en C3 y «el ID no se reutiliza» en C4. RF-08 dice «etiqueta o estado del tablero»: se cumple con roadmap y `next`, no con `board` (D6).
- Suposiciones a verificar: que `Documento::Formato1` lleve `Cuerpo`; que sin red la identidad salga de la cache; que la transaccion haga su propio `fetch`.

## Riesgos aceptados y que los contiene

- El PR de una tarea en revision descartada queda abierto (sin adaptador de plataforma aqui): lo dice el informe; lo cierran F6/F7.
- Una pausa pendiente se reejecuta como `decidido_por: agente` (igual que `Tomar::desde_anotacion`).
- Descartadas de formato 1 sin `motivo` salen «sin motivo» en el ROADMAP; lo resuelve la migracion de F1-12.
- Refactor de `toma`: lo contiene `tests/toma.rs` en el paso 1.
- Pausa calculada del cuerpo: O(filas) por tarea dentro de una lectura que ya existe.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- Unitarias en `ciclo/operacion.rs`: reescritura byte a byte, rechazos, `sin-cambios`, ida y vuelta de la anotacion, `exige_red`.
- Unitarias de `Cuerpo::en_pausa`, filtro de `listas` en `situacion` y `generar` (descartadas con motivo, «en pausa», mismos bytes).
- E2E en `crates/tc2/tests/ciclo.rs` con `comun::Fixture` y los helpers de `toma.rs`, C1..C11.
- Regresion: `toma.rs`, `alta.rs`, `importar.rs`, `roadmap.rs`, `siguiente.rs` y los tests de uso de `lib.rs`.

## Lo que necesita decision de una persona

Carlos dejo dicho que se acepta la opcion recomendada en todas las tareas del H3 (cadena automatica); se aplican:

- D1 que es pausa: (a) filas `pausa`/`reanudar`, sin estado nuevo. **Elegida.**
- D2 quien descarta o pausa: (a) quien lleva la tarea, o cualquiera si no tiene asignado; ajena se rechaza. **Elegida.**
- D3 worktree al descartar: (a) se quita si esta limpio y publicado; si no, se conserva y se avisa; la rama nunca se borra. **Elegida.**
- D4 sin red: (a) `descartar` exige red; `pausa`/`reanudar` quedan pendientes (adenda F1-2). **Elegida.**
- D5 motivo de la pausa: (a) no. **Elegida.**
- D6 pausa en `board`: (a) no en esta tarea. **Elegida.**
- D7 `asignado_a` al descartar: (a) se conserva. **Elegida.**

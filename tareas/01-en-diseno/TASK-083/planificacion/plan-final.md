# Plan — TASK-083: F3-13 estimacion por tarea al planificar

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 420 min

Modulo nuevo `crates/tc2/src/estimacion/` con lo que F1-17 encarga a F3-13 en «Consecuencias»: `historico`, `estimar`, `restante` y la escritura del campo `estimacion`. Las ordenes que ya escriben `## Transiciones` lo llaman por un unico punto, `recalcular`. Sin vista nueva: status line y `status --seguir` son de F3-12.

Estado actual (verificado): la 2.0 escribe transiciones en `start` (`toma/operacion.rs:186`), `descartar`/`pausa`/`reanudar` (`ciclo/operacion.rs:102`) y `liberar`/`reasignar` (`ciclo/liberar.rs`); no existen `plan`, `approve`, `review` ni `finish` (llegan con F4-1, F4-2, F6-2 y F8-7). `Tarea.estimacion` es un `Option<String>` privado (`tarea/modelo.rs:326`) sin metodo para cambiarlo. `alta` e `import` ya leen todos los `tarea.md` (`alta.rs:360`, `importar/mod.rs:109`); `start` solo su tarea y dependencias (`toma/operacion.rs:160`). El puerto de git no tiene `log`. F3-2 (TASK-029) terminada.

Piezas, en orden:
1. `estimacion::real` (puro): de transiciones, filas de `## Coste` y commits, el real por fase (implementacion: `start` → primera `review`; rondas: primera → ultima `review`; cierre: ultima `review` → `finish`), quitando ventanas de `pausa` y contando como 45 min cada hueco de mas de 45 (F1-17 D6a y divergencia 6); duracion negativa → descartada con motivo.
2. `estimacion::estimar`, `restante` y `error` (puros): cascada de F1-17 punto 3 (al menos 5 muestras por nivel: carril+complejidad+tamano, carril+complejidad, carril, repo) con mediana y p80 por fase y total, nivel y `n`; tamano = casillas de `## Criterios de aceptacion`; `restante` del punto 5 con cota inferior si la fase va desbordada; `error` absoluto, relativo y razon estimado/real.
3. `estimacion::forma` y `Tarea::poner_estimacion(&Estimacion)`: YAML con la forma exacta del punto 7 de F1-17 (`calculada`, `nivel`, `n`, `por_fase`, `total`, `agente`) y su lectura; unico camino de escritura.
4. `estimacion::historico(leidas, commits) -> (Vec<Muestra>, Descartes)`: sobre `estado::tareas`, ambos formatos; `Subcomando::Log` en el puerto y `git::historia` con un solo `git log <develop> --format=%H %P %ct %s`; asignacion de commits por ID en el asunto o M^1..M^2 del merge (regla de la PoC); agrupacion de tareas troceadas con la regla de `poc-f1-17.py`; nunca escribe en una terminada.
5. `estimacion::recalcular(&mut Tarea, &Cuerpo, &Muestras, ahora)` conectado a `Alta`/`Importar` (una sola pasada de `estado::tareas`; un import de 37 calcula las muestras una vez) y a `Tomar` (`start`, dentro de `aplicar` con estado fresco; commits leidos antes de la transaccion). Linea nueva `estimacion: ≈M min (p80 P), n=K, nivel=X` con claves en `--json`. Orden nueva `tc2 estimar` (rellena las abiertas con `estimacion: null` en un solo commit de estado) y `tc2 estimar --error TASK-NNN` (solo lectura). `doctor` gana `historico: usables=N descartadas=M`.

Coste: O(N) lecturas de `tarea.md` (un `cat-file --batch`) mas un `git log` por `new`/`import`/`start`/`estimar` (F1-17 punto 6 lo acepta); `start` pasa de 1+d a N lecturas. La status line no recorre el historico.

Descartado: guardar el real en `## Duracion` y leerlo de ahi (dos comportamientos para el mismo hecho; `finish` no existe); tareas parecidas por titulo con `texto::parecido` o por etiquetas (la PoC solo midio la cascada; contradice D8a sin ADR).

## Lo que el rol no cubrio

- Riesgos, pruebas y dominio sin revisar a fondo.
- Fuera de esta tarea: status line, `estado.json` y `status --seguir` (F3-12, TASK-084); hora de fin de un lote (F4-9); resumen de `finish`, `## Duracion` y calibracion en el informe por version (F8-7); transiciones `plan`/`approve`/`review`/`finish` (queda `recalcular` listo para F4-1, F4-2, F6-2 y el cierre); semillas de otros proyectos (D3); `resumen`, `repartir` y zonas.
- `agente` (la «Duracion estimada» del plan) queda `null` hasta que exista `plan`.
- Contra el enunciado: «cada tarea planificada tiene su estimacion» lo cubren C3 y C4; «el error se mide» queda medido por tarea (C6) pero no se guarda al cerrar hasta F8-7.
- Suposiciones a verificar: forma y fecha de las filas de `## Coste` de la 0.16 (`tareas/04-terminadas/TASK-029/tarea.md`); que el asunto de los merges de la 0.16 basta para M^1..M^2 (`poc-f1-17.py`); forma del contrato para la linea y claves de `estimacion` (`contrato.rs`, F1-4); que `tc2 estimar` no choca con F1-4 D1a ni con `FASES` (`config/claves.rs:17`), con adenda si hace falta.

## Riesgos aceptados y que los contiene

- Trabajo de mas de 45 min sin commit ni fila de coste queda corto: lo acepta F1-17; regla unica y probada.
- Poco historial al principio: las tareas de la 0.16 sin carril solo entran en el nivel `repo`; se muestran nivel y `n`.
- `start` mas caro (O(N) y un `git log`, repetido si la transaccion reintenta): solo en transiciones.
- Rango sin calibrar (F1-17 D8a): no se presenta como calibrado.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- Unitarias de las funciones puras: `real` (tope 45, pausas, pausa sin `reanudar`, negativas), `estimar` (cascada, cada caida de nivel, umbral 5), `restante` (cota inferior), `error`, `forma` (ida y vuelta y `Tarea::leer` acepta lo escrito).
- Paridad con la PoC: fixture JSON con el corpus y sus valores esperados sacados una vez de `poc-f1-17.py`.
- Integracion con remoto bare (como `toma` y `alta`): `new`, `import` de 37 (una sola lectura), `start`, `estimar`, `estimar --error` y la linea de `doctor`.
- Regresion: terminadas byte a byte; `pausa`, `liberar` y `descartar` no cambian `estimacion`.

## Lo que necesita decision de una persona

Carlos dejo dicho que en el H3 se acepta la opcion recomendada y que no se trocea; se aplican:

- D1 fuente de eventos del real: (a) transiciones + `## Coste` + commits de `develop` con un solo `git log` (fiel a F1-17 punto 1). **Elegida.**
- D2 cuando se recalcula: (a) en `new`, `import`, `start` y `tc2 estimar` (y en `plan`/`approve`/`review`/`finish` cuando existan); no en `pausa`, `reanudar`, `liberar` ni `reasignar`; adenda a F1-17 punto 6. **Elegida.**
- D3 semillas de otros proyectos: (a) ninguna; sin datos sale `sin-datos`; adenda a F1-17 punto 3. **Elegida.**
- D4 medicion del error: (a) funcion pura y `tc2 estimar --error` de solo lectura; la escritura en `## Duracion` la hace `finish` en F8-7. **Elegida.**
- D5 carril de las muestras de la 0.16: (a) solo el declarado; con `null`, solo nivel `repo`. **Elegida.**

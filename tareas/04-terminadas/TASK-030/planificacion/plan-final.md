# Plan — TASK-030: F3-3 tc2 next: lo listo para empezar por dependencias y prioridad

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 240 min

`tc2 next` lee el estado tal como lo conoce el clon (`estado::leer`, sin fetch, sin escribir), proyecta cada tarea (formato 1 o 2) a un nodo minimo y calcula el orden de trabajo en un nucleo puro: Kahn con prioridad (posicion del hito, numero, sufijo). De ese orden filtra las «listas». El nucleo va en su propio modulo para que F3-4 (`roadmap`) lo reutilice: un solo calculo de orden.

Hecho verificado: el ROADMAP de ToolsaWeb lo genera `scripts/grafo-tareas.py` (linea 3), no la 0.16. El oraculo de C1 es ese ROADMAP congelado en un SHA (D1).

Reglas:
- Lista: tarea `planificada` cuyas dependencias existentes y no descartadas estan todas `terminada`. Las de diseno, curso o revision van en `en_marcha`.
- Prioridad: no hay campo `prioridad`; la clave es (posicion del hito, numero, sufijo) y el orden es la posicion en el Kahn sobre todas las tareas no descartadas, terminadas incluidas (una dependencia terminada de un hito posterior retrasa a la tarea, como el script).
- Dependencia inexistente: no bloquea; aviso `dependencias_inexistentes=`.
- Dependencia descartada: sale del grafo de orden pero bloquea la tarea; aviso `depende_de_descartada` (D3).
- Ciclos: `atascadas`, fuera de las listas, aviso `ciclos=`. `tarea.md` ilegible: `ilegibles=`.
- Salida: `next ok listas=... en_marcha=... commit=<sha-estado> [avisos]`; `--json` con las mismas claves y listas en array.
- Sin red funciona sobre `origin/taskcodetwo/tareas` tal como este; `commit=` dice que foto se uso (frescura en F3-5b). Sin rama de estado: `rechazada`.

Piezas:
- `crates/tc2/src/grafo.rs` (nuevo, puro): `orden_de_trabajo(nodos, hitos) -> Orden { orden, atascadas, ciclos, avisos }` y `listas(&Orden, &nodos)`; `Nodo { id, sprint, estado: Fase, dependencias }` con `Fase = Viva(estado) | Descartada` y `Nodo::de(&Documento)` (formato 1: etiqueta `descartada`; formato 2: estado `descartada`).
- `estado::tareas(&dyn Estado) -> Vec<(ruta, Result<Documento, ErrorTarea>)>` (sube desde `alta::tareas`, que queda como filtro encima; `next` no depende de `alta`).
- Lector del orden de hitos: `### Hn — titulo` de la zona manual de `ROADMAP.md` en la rama de estado (D2); si no hay, numero del hito y aviso.
- `crates/tc2/src/siguiente.rs` (`NOMBRE = "next"`), rama en `lib.rs` y catalogo i18n, con el patron de `importar`.

Orden de construccion:
1. `grafo` con tests sinteticos por regla (Kahn con `BinaryHeap`, O((n+e) log n)).
2. Fixture de ToolsaWeb y test de C1, en rojo hasta pasar.
3. `estado::tareas` con los dos formatos y `alta::tareas` encima sin cambiar comportamiento.
4. Lector del orden de hitos.
5. Orden `next` con `--json`, i18n y E2E en repo temporal sin remoto alcanzable.

Alternativas descartadas: replicar `core/roadmap.ts` de la 0.16 (desempata distinto y no conoce `descartada`; no sirve de oraculo); ordenar hitos por numero (falla C1: H14 va entre H0 y H1).

## Lo que el rol no cubrio

- Riesgos, diseno de pruebas y reglas de negocio sin especialista; las reglas de «lista», descartada y prioridad se aceptan por la consigna.
- Suposiciones sin verificar: que ToolsaWeb no tiene IDs con sufijo ni de 4 cifras; que la linea «Listas» del ROADMAP corresponde al arbol del mismo commit (confirmarlo con `python -I scripts/grafo-tareas.py --comprobar` en el SHA); que `TareaV1::leer` acepta todos los `tarea.md` de ToolsaWeb; que la rama de estado es plana (`TASK-NNN/tarea.md`).
- Como llegan a la rama de estado los hitos de ToolsaWeb (hoy en `docs/planificacion/hitos.md`): toca a F9-2 y a F1-11 D2a.

## Riesgos aceptados y que los contiene

- La fixture se desfasa de ToolsaWeb vivo: a proposito, SHA fijado.
- Copia vieja del estado en el clon: `commit=` en la salida; frescura en F3-5b.
- Escala: un `cat-file --batch` mas O((n+e) log n) con ~400 tareas.
- La fixture compacta no ejercita el lector estricto de formato 1 con los `tarea.md` reales: si se exige, test `#[ignore]` con los reales.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):
- C1: fixture fija y comparacion del vector entero de listas mas `en_marcha`.
- C2: unitarios en `grafo.rs` por regla.
- C3: `tests/siguiente.rs` (es/en, `--json`, sin red, sin rama de estado).
- C4: unitario o E2E con un `tarea.md` ilegible.

## Lo que necesita decision de una persona

Carlos acepta las recomendaciones (consigna del 2026-11-16):
- D1: oraculo de C1 = ROADMAP de ToolsaWeb generado por `grafo-tareas.py`, congelado en un SHA y anotado en la tarea.
- D2: orden de hitos desde los `### Hn — titulo` de la zona manual de `ROADMAP.md` de la rama de estado; si faltan, numero y aviso.
- D3: una dependencia descartada bloquea, con aviso.
- D4: sin campo `prioridad`; la prioridad es (hito, numero).

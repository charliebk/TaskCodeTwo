# Plan — TASK-082: F3-11 aviso de solape

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 180 min

Decision previa de Carlos (2026-11-18): TASK-082 se hace sin depender de TASK-036 (F4-1, `plan`, que no existe): el solape sale de lo que ya tocan las ramas publicadas de las tareas en curso (diff real), sin prediccion del plan; queda el punto de extension para F4-1 y se anota como divergencia. Se quita TASK-036 de las dependencias de esta tarea.

El aviso se monta sobre `start`, que ya tiene el `fetch` completo previo, la lectura del estado y la rama propia creada o reanudada. Nunca cambia el resultado ni el codigo. Precedente: el aviso de titulos parecidos del alta (`con_aviso` en `lib.rs`): stderr con codigo 0.

Piezas, en orden:
1. `crates/tc2/src/solape.rs` (nuevo, puro): `Toca { id, quien, ficheros: BTreeSet<String> }`, `modulo(ruta) -> Option<&str>` (carpeta padre; la raiz no tiene modulo) y `cruzar(propia, otras) -> Vec<Cruce>` (`Cruce { id, quien, ficheros, modulos }`). Punto de extension F4-1: `Toca` ganara `zonas` (globs de F1-17 punto 11) y `cruzar` la regla de prefijo.
2. `crates/tc2/src/git/diff.rs`: `ficheros_de_rama(git, dir, base, rama) -> Paso<BTreeSet<String>>` con un solo `git diff --name-only -z --no-renames <base>...<rama>` (el `...` toma el merge-base; con `--no-renames` salen los dos nombres de un renombrado, como F1-15 S08). `Diff` ya esta en la lista cerrada del puerto.
3. `crates/tc2/src/toma/mod.rs`: tras `destino.crear(...)`, en los tres finales con exito de `paso` (toma nueva, empezada, sin cambios), `con_solape(i, &vista, ...)`: tareas en curso y en revision con asignado y rama de `situacion::calcular(&vista)` (una sola fuente, la de `board`/`status`); `git::fechas_de_ramas` (un `for-each-ref`, sin red) para saber que ramas existen en `origin`; un `ficheros_de_rama` por cada otra tarea en curso con rama publicada (base `git::base_de_rama`); el conjunto propio de `refs/heads/<rama>`.
4. Catalogo `i18n/catalogo/toma.rs` y `lib.rs`: aviso a stderr al estilo de `con_aviso`; login, rutas e IDs por `tablero::sanear` (F1-17 punto 8).
5. Adenda en F1-17: F3-11 solo cruza el diff real (sin zonas) porque F4-1 no existe; «modulo» = carpeta padre; calculo local tras el `fetch` de `start`; se quita la dependencia de F4-1.

Coste: 1 + k + 1 procesos de git sin red (un `for-each-ref`, un `diff` por cada una de las k tareas en curso con rama publicada y el propio) mas una lectura en lote de los `tarea.md`; se filtra por estado antes de lanzar diffs (F1-3 conserva todas las ramas).

Descartado: un unico `git log --source --name-only` (atribuye mal commits compartidos y cuenta cambios revertidos); leer solo los `tarea.md` con rama remota (no ve tareas en curso sin rama y duplica la regla de «en curso»); calcularlo en cada `status`/`board` (D4).

## Lo que el rol no cubrio

- Riesgos sin rol propio: `diff` que falla a mitad (propuesta: saltarla y marcar `solape=incompleto`), base de un `hotfix`, rutas con comas en una lista de la linea.
- Dominio: que estados cuentan (D2) y que es un modulo (D1).
- Testing sin revisar.
- Contra el enunciado: «dos tareas que tocan el mismo modulo → aviso» lo cubren C1 y C2; lo que F3-11 llama «segun sus planes» queda fuera hasta F4-1 por decision de Carlos (divergencia en la adenda).
- Suposiciones a verificar: que `Ficha` lleva el estado para filtrar; que hay un test de `start` con remoto bare reutilizable; que `:` es valido en un valor de lista del contrato de F1-4; que la nota de F1-18 sobre `tarea_archivo` de F10-4 queda anulada (Fase 10 descartada).

## Riesgos aceptados y que los contiene

1. En el primer `start` la rama propia esta vacia: el aviso real aparece al reanudar, retomar o reasignar, y cuando exista F4-1; lo contiene D3b y el punto de extension.
2. «Carpeta padre» da falsos positivos en carpetas con muchos ficheros (`crates/tc2/src`): es solo aviso (F1-17 D4a) y se distingue fichero comun de modulo comun.
3. Las ramas ajenas son las del `fetch` de `start` (segundos): misma frescura que la toma.
4. Coste lineal en k mas una segunda lectura en lote de los `tarea.md` en `start`: aceptable con k del orden de las personas del equipo.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- Unitarias en `solape.rs`: `modulo` (raiz, anidados, separadores) y `cruzar` (fichero, modulo, nada, varias tareas, orden estable).
- Integracion con dos clones y remoto bare: C1-C8, C10 y C11; C5 y C10 con `Git::registro`; `--json` con las mismas claves.
- Regresion: suites de `start`, `liberar` y `reasignar`.

## Lo que necesita decision de una persona

Carlos dejo dicho que en el H3 se acepta la opcion recomendada y que no se trocea; se aplican:

- D1 que es modulo: (a) la carpeta padre del fichero; los de la raiz no tienen modulo. **Elegida.**
- D2 estados que cuentan: (a) en curso y en revision. **Elegida.**
- D3 primer `start` con la rama propia vacia: (b) una linea `nota:` (no `aviso:`) con las tareas en curso, su @ y sus modulos tocados (como mucho 5 por tarea) y `en-curso-modulos=TASK-001:src/x` en la linea. **Elegida.**
- D4 donde mas se avisa: (a) solo en `start`; la funcion queda publica para `review` y `status`. **Elegida.**
- D5 forma de las claves: (a) listas planas `TASK-NNN:valor` (`solape`, `solape-ficheros`, `solape-modulos`, `solape-asignados`, `solape-sin-rama`) cortadas a 20 con `solape-truncado=si`. **Elegida.**

# Plan — TASK-127: Escrituras de estado con muchos ficheros fallan en Windows (git hash-object)

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

`construir_commit` (`crates/tc2/src/git/estado.rs`, l. 269-341) es la funcion por la que pasan todas las escrituras de estado: `estado::transaccion` (transaccion.rs l. 370 y 535) e `init::aplicar` (aplicar.rs l. 129). Tiene **dos** argumentos que crecen con el numero de ficheros, no uno:

- l. 299-301: `hash-object -w --no-filters -- <ruta temporal>` x N. Cada ruta mide unos 80 caracteres, porque esta bajo `<comun>/taskcodetwo/estado-tmp/<pid>-<n>/blob-i`.
- l. 313-318: `update-index --add --cacheinfo 100644,<sha>,<ruta>` x N, unos 80 caracteres por entrada.

Con 1000 ficheros cada uno ronda los 80 KB, por encima del tope de unos 32 K caracteres de `CreateProcess`. Si solo se arregla `hash-object`, el siguiente fallo sale en `update-index`. Las dos llamadas pasan sus datos por stdin:

1. **Se extiende el puerto** (`git/puerto.rs`). Hoy `Git::ejecutar_con_entrada` (l. 294) solo admite `cat-file` y no lleva indice. Pasa a una lista cerrada de formas exactas; cualquier otra combinacion se rechaza sin lanzar git:
   - `cat-file --batch` (la de hoy);
   - `hash-object -w --no-filters --stdin-paths`;
   - `update-index --add -z --index-info`, solo con indice temporal de ruta absoluta.

   Para el indice, `ejecutar_con_entrada` recibe un `indice: Option<&Path>` interno, o se anade `ejecutar_con_indice_y_entrada`. En los dos casos se reutilizan `comando(.., Some(indice))` y el hilo escritor que ya hay contra el interbloqueo de tuberias (precedente F2-4b, `leer_blobs`). No se crea un helper de stdin aparte.
2. **Se reescribe el tramo l. 290-319 de `construir_commit`**:
   - las rutas temporales van a `hash-object` por stdin, una por linea (nombres `blob-i` generados por la propia funcion: sin saltos de linea);
   - las entradas van a `update-index` como `100644 <sha>\t<ruta>\0` con `-z`.

   El resto no cambia: `read-tree`, `write-tree`, `rev-parse`, `commit-tree`, el control `HASH_OBJECT_INCOMPLETO` y la carpeta `Temporal`.
3. Resultado: 6 procesos por commit (4 sin base), sea cual sea N. Cumple el criterio 2 de forma literal.
4. En el mismo cambio se quita el rodeo `chunks(100)` de `crates/tc2/tests/progreso.rs` l. 349-352, que existia solo por este fallo.
5. Adenda de una linea al ADR `docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md`: el puerto admite stdin en esas tres formas exactas; `update-ref --stdin` sigue prohibido.

Orden: puerto con su lista cerrada, despues `construir_commit`, despues el test de 1000 y el rodeo de progreso, y por ultimo la adenda. Cada paso compila y pasa la suite.

Duracion estimada: 45 min
Senales: 2 ficheros de codigo (puerto.rs, estado.rs) mas 2 de test y 1 ADR; una capa (git); 2 criterios. El arreglo toca dos llamadas y amplia el contrato publico del puerto.

## Lo que el rol no cubrio

- Riesgos: no enumero modos de fallo. Se revisan en la implementacion: git que muere a mitad del stdin; salida de `--stdin-paths` con menos lineas de las esperadas (la cubre `HASH_OBJECT_INCOMPLETO`, se comprueba con un test); `-z --index-info` existe en git desde hace mucho (no es riesgo con la version minima del proyecto, se confirma en la implementacion).
- Dominio: no se reviso si alguna operacion real produce mas de 1000 cambios; 1000 es el tamano del criterio.
- Pruebas: el plan de abajo es de arquitectura, no lo diseno un rol de testing.
- Criterio 1 ("en Windows"): razonado leyendo el codigo; se reproduce el fallo en esta maquina (Windows) antes del arreglo con el test nuevo.

## Riesgos aceptados y que los contiene

- **Se amplia lo que admite stdin en el puerto.** La adenda F2-3 del ADR F1-4 (l. 539-553) prohibe `--stdin` solo en `update-ref`, que sigue igual; el docstring y el test del puerto (puerto.rs l. 636-642) fijan "solo cat-file". Lo contienen: formas exactas, `update-index` con indice temporal absoluto obligatorio y la adenda al ADR.
- **Las rutas ya no se ven en `Git::registro()`**: van por stdin y el registro solo guarda argumentos. `registro_permitido` (tests/comun/mod.rs l. 215) ya admite `hash-object` y `update-index`. Se acepta.
- **Descartado: tandas de K rutas.** Los procesos crecen como N/K y hay que calcular un presupuesto de bytes por plataforma.
- **Descartado: `fast-import` o `mktree` por niveles.** Subcomando nuevo y reescritura mayor.
- `ruta_valida` sigue rechazando `,` aunque `--index-info` ya no lo necesita: regla de dominio fuera de esta tarea.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- Criterio 1 (1000 ficheros en una transaccion terminan en ok): **1 test nuevo, pesado** (escala). Integracion en `crates/tc2/tests/estado_*.rs`: `estado::transaccion` con una operacion tipo `Poner` (como la de progreso.rs l. 95-124) que escribe 1000 ficheros de una vez; resultado `Ok` y la rama tiene los 1000 blobs con su contenido (`ls-tree` cuenta las entradas). Se ve rojo antes del arreglo en Windows.
- Criterio 2 (procesos acotados): **2 tests nuevos.**
  - (a) Pesado (escala). Con 10 y con 1000 ficheros, el numero de lineas de `Git::registro()` del commit es el mismo, y ninguna linea de `hash-object` o `update-index` lleva rutas. Comparte el repo de 1000 con el del criterio 1 si cabe.
  - (b) Ligero, unitario en `puerto.rs`: las tres formas admitidas llegan a `Lanzar` y cualquier otra da `Prohibido` sin lanzar git (`hash-object --stdin`, `update-index --index-info` sin indice o con indice relativo, `update-ref --stdin`). Sustituye a la asercion "solo cat-file".
- Regresion: suite actual de `estado_*`, `init` y `progreso` sin el rodeo `chunks(100)`.
- Mutacion: volver a pasar las rutas por argumentos en `construir_commit` pone rojo el test (a).

## Lo que necesita decision de una persona

Decidido por Carlos el 2026-11-18 al planificar:

- D1 contrato del puerto: (a) stdin en `hash-object` y `update-index` con formas exactas y adenda al ADR F1-4. **Elegida.**
- D2 lectura del criterio 2: numero constante de procesos (excluye tandas). **Elegida** (consecuencia de D1).
- D3 rodeo `chunks(100)` de `tests/progreso.rs`: se quita en esta tarea. **Elegida** (opcion recomendada, consigna de Carlos para esta cadena).
- Sin troceo: 45 min, por debajo de 60.

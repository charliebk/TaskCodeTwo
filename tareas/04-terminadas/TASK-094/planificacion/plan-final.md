# Plan — TASK-094: F1-18 ADR hito de decisiones (ADR) en init y en actualizar

(Respuesta del unico rol de brainstorm, arquitectura, volcada tal cual. Con un solo rol no hay unificador; el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 120 min

Senales: dos dependencias aceptadas (F1-11 y F1-12) que aportan entradas ya medidas (inventario parche/propio, informe de `--simular`, 91 / 973 de OpenDockTrading). Dos PoC baratas: (1) leer `## Coste` y `## Duracion` de las 12 tareas del H1; (2) estudio de adopcion sobre una copia de OpenDockTrading, sin `subtree split`. Precedentes: TASK-012 estimo 170 y tardo 94; TASK-013 estimo 150 y tardo 165. Mas estrecha que F1-12 porque reutiliza su informe. **No se trocea**.

### Decisiones ya tomadas por la persona (Carlos, 2026-11-14)

1. Se llama **«Hito de decisiones (ADR)»**, no «Hito 0». Paso imprescindible en todo proyecto: en `init` investiga el contexto y deja los ADR antes de la primera tarea de codigo; en `actualizar` (proyecto con la 0.16 o existente) hace un estudio de adopcion: inventaria lo hecho a mano, mide el uso real (tareas, rondas, tokens, verificacion), contrasta con la 2.0 y deja un ADR y tareas por hueco.
2. **Obligatorio pero proporcional**: su tamano sale del propio estudio. `start` de una tarea de codigo lo exige cerrado o declarado no necesario por una persona, con un **comando determinista**; los hooks solo como atajo opcional que llama al mismo comando (F1-11, decision 10).
3. **Nombres de la 2.0 aprobados**: CLI `tc2`, plugin y prefijo `taskcodetwo`, rama `taskcodetwo/tareas` (donde F1-1..F1-11 dicen `taskctl` para la 2.0, se lee `tc2`).
4. **Seguridad y trabajo en grupo imprescindibles.** El manifiesto de recursos es de F1-14/F1-15: aqui solo se cita.
5. **Orden de migracion**: OpenDockTrading (F9-7), relevo de TaskCodeTwo (F4-5), ToolsaWeb (F9-8), como fija F1-12 (decision 9).
6. No trocear.
7. **PoC con datos medidos**: cifras del H1 (TASK-002..TASK-013) y estudio de adopcion sobre una **copia** de OpenDockTrading (original solo lectura). Pruebas en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-094\`, borradas al terminar.

### Donde encaja: se extienden tres piezas existentes, ninguna paralela

1. **El estudio de adopcion amplia el informe de `tc2 actualizar --simular` (F1-12, PL1).** Al bloque «uso» (hoy solo la tabla de campos de F1-9) se anade el uso medido: tareas por estado, rondas por tarea, tokens por fase (de `## Coste`), minutos (de `## Duracion`) y registros de verificacion. Los bloques «hecho a mano» (F1-11) y «sin clasificar» ya enumeran los huecos. Encima, una funcion pura `proponer_hito(informe) -> PropuestaHito` devuelve ADR y tareas, una por hueco, con presupuesto; su salida es un fichero en formato de `import` (F3-2) fuera del repo. En `init` recibe un informe de contexto: lista fija de areas de decision (pila, verificacion, ramas y puertas, seguridad, equipo, plataforma), cada una «aplica / no aplica» con motivo.
2. **El estado del hito vive en la rama de estado (F1-1/F1-2)**: registro `hito-decisiones.yml` en la raiz de `taskcodetwo/tareas`, escrito de forma atomica:
   ```yaml
   formato: 2
   origen: "init" | "actualizar"      # informe_hash: hash del informe del que sale
   tareas: ["TASK-NNN", ...]
   no_necesario: null | {por: <identidad de persona>, fecha, motivo}
   ```
   **El estado se calcula, no se guarda**: `hito_estado(rama) -> abierto | cerrado | no-necesario | ausente`; `cerrado` = lista no vacia y todas sus tareas `terminada` o `descartada`; `no-necesario` = firmado por una persona. No depende de si «hito» es `sprint` (pregunta abierta de F1-9).
3. **La puerta sigue el patron de F1-10**: parada local en `tc2 start` (si la tarea es de codigo y el hito no esta `cerrado` ni `no-necesario`, sale con el codigo de precondicion de F1-4 sin tomar la tarea, nombrando el comando que lo arregla) e imposicion en el check obligatorio `estado` (F1-11, decision 4), en rojo si el carril efectivo del PR no es `errata` y el hito no esta cerrado. «De codigo» = fuera de `tareas` del hito y carril distinto de `errata`.

**Comandos nuevos** (deterministas): `tc2 hito estado [--json]`, `tc2 hito proponer --desde <informe>`, `tc2 hito no-necesario --motivo` (exige identidad de persona, mismo mecanismo que `approve`; el agente no puede firmarlo). El atajo PreToolUse opcional solo llama a `tc2 hito estado --json`.

**Dos mitades en `actualizar` (F1-12, decision 8)**: los huecos bloqueantes (codigo 5) se resuelven antes de migrar como tareas de la 0.16 del propio proyecto (en OpenDockTrading, sus tres lectores); tras migrar, el resto entra con `tc2 import` y el registro lista tareas migradas y nuevas. `actualizar` no crea el hito en su transaccion (romperia CR1).

**TaskCodeTwo como precedente**: su Fase 1 es su hito de decisiones; en el relevo (F4-5) el registro lista tareas terminadas y sale `cerrado`.

**Se crea**: `docs/adr/F1-18-hito-de-decisiones.md`, su fila en `docs/adr/README.md` y `poc/` (guion y log). **No se toca**: ADR F1-1..F1-12 (lo que cambia va en «Divergencias»), `docs/plan/PLAN-TASKCODE-2.md` (no tiene fila F1-18: se propone en D5), codigo de la 0.16 y de la 2.0, pilotos (solo lectura, copia de un commit fijo).

**Orden**: PoC H (minutos y tokens por ADR de TASK-002..TASK-013) → PoC A (estudio de adopcion sobre la copia de OpenDockTrading y `proponer_hito`) → PoC P (puerta simulada sobre tareas reales del H1 y de OpenDockTrading) → regla de proporcionalidad → ADR, indice y divergencias.

### Regla de proporcionalidad (forma; cifras de la PoC H)

- Tamano = numero de huecos: parche sin capacidad → 1 tarea; hueco bloqueante → 1 tarea de la 0.16 antes de migrar; grupo sin clasificar → 1 ADR; area de `init` que aplica → 1 ADR.
- Presupuesto = ADR × minutos y tokens por ADR del H1, con mediana y p75, no media.
- Cero huecos: propuesta vacia; la persona puede firmar `no-necesario` con `informe_hash` como evidencia.
- Lectura previa sin comprobar: los «Real» de TASK-002..TASK-013 suman unos 739 min en 12 ADR (12 a 165 min, mediana ~45), sin la fase de diseno. En tokens se separa `salida` de `total` (domina `cache_lectura`).

### Motivo para no trocear

Consigna de Carlos al lanzar el H1 (2026-11-14): «NO troceamos ninguna tarea». Contrato, proporcionalidad y puerta salen de las mismas tres PoC y del mismo informe.

## Lo que el rol no cubrio

- Criterio 1: cubierta la forma; la lista fija de areas de `init` es regla de **dominio**, sin revisar.
- Criterio 2: cubierta la forma; umbrales concretos (p75, tope por hito) son de dominio y salen de la PoC.
- Criterio 3: cubierto en estructura (informe ampliado, sin tocar transaccion ni criterio, comandos y no hooks).
- **Riesgos** no enumerados: registro borrado o editado a mano; carril `errata` declarado en una tarea que acaba tocando codigo; clon sin rama de estado en `start`; `no-necesario` sin identidad disponible (F2-13 aun no existe).
- **Testing**: solo la PoC; la bateria real es de F3-6 y F6.
- **Dominio**: si documentacion con codigo de ejemplo es «de codigo»; si `no-necesario` caduca al cambiar el informe; si «hito» es `sprint`.
- **Rendimiento**: `hito_estado` crece con el tamano del hito, no del backlog; no se mide.

## Riesgos aceptados y que los contiene

- Saltar la puerta local declarando `errata`: lo contiene el check `estado` con el carril efectivo por ficheros (F1-10, decision 3).
- Hito de `actualizar` partido en dos fases: lo contienen el codigo 5 de F1-12 antes y el registro despues.
- Cifras del H1 de un solo proyecto y una persona: semilla de la regla; recalibracion en F1-17/F8-7.
- OpenDockTrading sin `## Coste`: ese uso queda «no medido», no se inventa.

## Plan de pruebas

`poc/poc-f1-18.py` (`python -I -B`, log determinista) lee la tabla `cifras` del ADR y sale con 1 si una cifra no es la medida, falta o sobra (precedente F1-11 y F1-12). Copias mutadas del ADR (cifra cambiada, fila borrada, ID inexistente, seccion borrada) fallan cada una por su motivo. El tiempo de reloj va a stderr, fuera de la tabla.

- **Criterio 1 → 2 pruebas**: (1) copia de OpenDockTrading con `git clone --no-hardlinks` en commit fijo: el informe recalculado reproduce los bloques de F1-12 (91 / 973, 15 / 76, sin clasificar); uso medido o «no medido»; `proponer_hito` da un elemento por hueco separando los bloqueantes; hash del arbol igual antes y despues. (2) Registro sintetico en una rama de estado de prueba: `hito_estado` da `ausente`, `abierto`, `cerrado` y `no-necesario`.
- **Criterio 2 → 2 pruebas**: (1) cifras H por ADR (total, salida, mediana y p75); falla si una tarea no tiene tabla. (2) Puerta simulada sobre tareas reales: con hito abierto se niega en las de codigo y deja pasar las del hito y `errata`; con hito cerrado o `no-necesario`, pasan todas; mismo resultado en dos llamadas.
- **Criterio 3 → 1 prueba**: tabla requisito → decision → evidencia (RR-13, RR-12, F1-11 dec. 10, F1-12 PL1, CR1 y dec. 8), IDs comprobados, fila del indice con enlace que resuelve; al cerrar, `comprobar-repo.sh` en verde y `taskctl doctor` sin errores.

## Lo que necesita decision de una persona

Consigna: responder la recomendada. Respuestas (Carlos, consigna «opcion recomendada»): **D1a, D2a, D3a, D4a, D5a**.

- **D1. Donde vive el estado del hito**: **(a, recomendada)** registro `hito-decisiones.yml` en `taskcodetwo/tareas` con estado calculado; (b) clave en la config; (c) solo etiqueta sin registro. Descartado: fichero bajo `.git/taskcodetwo/` (local, rompe el trabajo en grupo).
- **D2. Que es «tarea de codigo»**: **(a, recomendada)** carril distinto de `errata` y fuera del hito, `start` para y el check `estado` impone; (b) por `tipo` (casi no se usa, F1-9: 6 / 0 / 0); (c) todas menos las del hito.
- **D3. Quien declara «no necesario»**: **(a, recomendada)** solo una persona con identidad, `motivo` obligatorio e `informe_hash`; (b) cualquiera con confirmacion interactiva.
- **D4. Como entran las tareas del hito tras `actualizar`**: **(a, recomendada)** `--simular` → `tc2 hito proponer` (fichero fuera del repo) → tras migrar `tc2 import`; bloqueantes antes como tareas de la 0.16; (b) dentro de la transaccion (rompe CR1).
- **D5. Fila F1-18 que falta en el plan**: **(a, recomendada)** el ADR lo anota en «Divergencias» y propone la fila con dependencias F1-11 y F1-12; (b) editar el plan en esta tarea.

## Suposiciones no verificadas

- Que las tareas de OpenDockTrading tengan `## Coste`, `## Duracion` y `revision/`; si no, «no medido».
- Que el hash de `planificar` (F1-12) cubra solo los pasos y no el bloque «uso»; el ADR lo fija.
- La ruta local de OpenDockTrading (la misma de la PoC de F1-12).
- El codigo concreto de precondicion de F1-4 (F1-12 usa 4; F1-4 deja los numeros para F2).

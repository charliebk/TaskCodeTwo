# ADR F1-18 — Hito de decisiones (ADR) en `init` y en `actualizar`: estudio que lo dimensiona, registro en la rama de estado con estado calculado y puerta determinista en `start` y en el check `estado`

- Estado: **aceptada** (2026-11-14)
- Tarea: TASK-094 · Plan: el [PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) **no tiene fila F1-18**; se propone en «Divergencias con el plan»
- Requisitos: RR-13 (actualizar proyectos existentes) y RR-12 (equipo) de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); RR-14 (se construye con la 0.16) como precedente
- Depende de: [ADR F1-11](F1-11-extensiones-declaradas.md) (inventario parche/propio, check `estado`, decisión 10 «comandos sí, hooks solo como atajo opcional») y [ADR F1-12](F1-12-actualizacion-entre-versiones.md) (informe de `tc2 actualizar --simular`, transacción con criterio CR1, huecos bloqueantes con código 5, decisión 8); usa [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) y [ADR F1-2](F1-2-escrituras-atomicas-ids-y-toma.md) (rama de estado y escritura atómica), [ADR F1-6](F1-6-identidades-agente-revisor-persona.md) (ninguna IA firma lo que firma una persona) y [ADR F1-10](F1-10-carriles-y-modo-automatico.md) (carril efectivo por ficheros, errata solo con `docs/`)
- Decide: Carlos, con sus decisiones del 2026-11-14 (se llama «Hito de decisiones (ADR)», no «Hito 0»; obligatorio pero proporcional; comando determinista y hooks solo como atajo; nombres `tc2`, `taskcodetwo` y rama `taskcodetwo/tareas`; orden de pilotos de F1-12; no trocear; PoC con datos medidos) y la consigna «la opción recomendada» (D1a, D2a, D3a, D4a y D5a del plan de TASK-094)

## Contexto

Antes de la primera tarea de código, este repo dedicó su Fase 1 a decidir:
18 ADR (F1-1 a F1-18), 12 terminados al empezar esta tarea. OpenDockTrading
hizo lo mismo sin nombrarlo: cinco tareas `ADR-001` a `ADR-005` (PoC A).
Ninguna de las dos cosas la pide hoy TaskCode: depende de que la persona se
acuerde. Carlos decidió el 2026-11-14 que sea un **paso del producto en todo
proyecto**:

- en un proyecto nuevo (`tc2 init`), investigar el contexto y escribir los
  ADR antes de la primera tarea de código;
- en un proyecto existente o con la 0.16 (`tc2 actualizar`), un **estudio de
  adopción** que inventaría lo que se hace a mano (F1-11), mide el uso real
  (tareas, rondas, tokens, minutos, verificación), lo contrasta con la 2.0 y
  deja un ADR o una tarea por hueco;
- **obligatorio pero proporcional**: su tamaño sale del propio estudio, y
  `tc2 start` de una tarea de código lo exige cerrado o declarado no
  necesario por una persona, con un **comando determinista**.

Dos ADR dan ya las piezas: F1-12 fija el informe de `tc2 actualizar
--simular` (bloques «uso», «hecho a mano» y «sin clasificar», con el
**91 / 973** de OpenDockTrading) y que los huecos que leen la forma de
carpetas de la 0.16 bloquean la migración (código 5); F1-11 fija el check
obligatorio `estado` y que ninguna garantía depende de un hook. Falta el
contrato del hito (entradas, salidas, cuándo está cerrado), la regla de
proporcionalidad con cifras y la puerta.

## Alternativas

| | Descripción | Veredicto |
|---|---|---|
| **N1** | «Hito de decisiones (ADR)», paso del producto en `init` y en `actualizar` | **Elegida** (decisión de Carlos) |
| N2 | «Hito 0», solo en proyectos nuevos | Descartada por Carlos: en un proyecto existente es donde más se hace a mano (PoC A: 9 huecos en OpenDockTrading) |
| **D1a** | Registro `hito-decisiones.yml` en la raíz de la rama `taskcodetwo/tareas`, **estado calculado** | **Elegida**: compartido por el equipo (RR-12), escrito con la escritura atómica de F1-2; cuatro estados y seis registros rotos medidos (PoC R) |
| D1b | Clave en `.taskcode/config.yml` | Descartada: la config viaja en la rama de código y cambia por PR; el estado del hito es estado de tareas (F1-1) |
| D1c | Solo una etiqueta en las tareas, sin registro | Descartada: no distingue «cerrado» de «nadie lo abrió» ni guarda la firma de «no necesario» |
| — | Fichero bajo `.git/taskcodetwo/` | Descartada: es local de cada clon y rompe el trabajo en grupo |
| **D2a** | Tarea de código = fuera de la lista del hito **y** carril efectivo distinto de `errata`; `start` para y el check `estado` impone | **Elegida**: con el hito abierto, 6 de 81 tareas de TaskCodeTwo pueden empezar (las ADR pendientes) y 0 de 50 de OpenDockTrading (PoC P) |
| D2b | Por el campo `tipo` | Descartada: F1-9 midió que casi no se usa (en OpenDockTrading es uno de los 8 campos sin ningún valor propio, PoC A) |
| D2c | Todas menos las del hito | Descartada: bloquearía una errata de documentación sin motivo |
| **D3a** | «No necesario» solo lo firma una persona con identidad, con `motivo` e `informe_hash` obligatorios | **Elegida**: una firma sin motivo o sin `informe_hash` sale `abierto` (PoC R) |
| D3b | Cualquiera, con confirmación interactiva | Descartada: un agente en modo automático confirmaría (F1-6) |
| **D4a** | `--simular` → `tc2 hito proponer` (fichero fuera del repo, formato de `import`) → los bloqueantes antes de migrar como tareas de la 0.16 → tras migrar, `tc2 import` | **Elegida**: no toca la transacción de F1-12 (CR1); el estudio no cambia ningún fichero de la copia (PoC A) |
| D4b | Crear las tareas del hito dentro de la transacción de `actualizar` | Descartada: rompe el criterio de salida estructural de F1-12 (la instantánea de tareas antes y después dejaría de casar) |
| **D5a** | El ADR anota la fila F1-18 que falta en el plan y la propone con dependencias F1-11 y F1-12 | **Elegida**: el plan se cambia con `taskctl` al cerrar, no aquí |
| D5b | Editar el plan en esta tarea | Descartada: los ADR no editan el plan (mismo criterio que F1-12) |
| **PR1** | Presupuesto con **mediana y p75** | **Elegida**: los minutos por ADR van de 12 a 165 con una cola larga (PoC H) |
| PR2 | Presupuesto con la media | Descartada: la media de los 12 ADR (739 / 12 = 61,6 min, cuenta hecha sobre la suma medida) queda por encima de 9 de los 12 |

## Pruebas de concepto

Guion `poc-f1-18.py` y su salida `poc-f1-18.log`, en la carpeta `poc/` de
TASK-094 (sin enlace: la carpeta de la tarea cambia de estado al cerrarse).
Python 3.12.10 (`python -I -B`), PyYAML 6.0.3 y Git 2.56.0, con la carpeta de
trabajo en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-094\poc\`
(borrada al terminar; el guion se niega a borrar una carpeta que no esté bajo
`pruebas-claude/<proyecto>/<motivo>` o que no lleve su marca).

```bash
python -I -B tareas/<estado>/TASK-094/poc/poc-f1-18.py \
  "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-094/poc" . .. > poc-f1-18.log
```

- **Datos fijos**: de TaskCodeTwo se leen objetos de git del commit
  `0cc5bc3` (la rama de esta tarea al empezar), no el árbol de trabajo; de
  OpenDockTrading, una copia `git clone --no-hardlinks` en `develop`
  `8141a08` (el mismo commit que midió F1-12; sigue siendo la punta de
  `develop` el 2026-11-14). El original **solo se lee**. No se lanza
  `taskctl` ni ningún hook; no se usa la red ni GitHub.
- **Mismo código que F1-12**: el guion carga como módulo `poc-f1-12.py`
  (TASK-013) y usa sus lectores de tareas de la 0.16, su inventario «hecho a
  mano» y su uso de campos, de modo que los bloques de F1-12 se recalculan,
  no se copian.
- **Comprobación del ADR (G)**: el guion lee la tabla `cifras` de este ADR y
  sale con 1 si una cifra no es la medida, si falta una medida o si sobra una
  fila; también comprueba el `resumen`, las secciones, el estado, los IDs
  citados (existen en el plan o en los requisitos; F1-18 solo si
  «Divergencias con el plan» propone su fila), los enlaces y la fila del
  índice. Once copias mutadas del ADR fallan cada una por su motivo. El
  tiempo de reloj va a stderr y no es una cifra comprobada.

Salida observada, recortada (la completa está en el log):

### H — Minutos y tokens por ADR de la Fase 1 de TaskCodeTwo

```text
tarea     estimada  real  rondas  diseno?  total_tokens  salida  diseno_total
TASK-002        60    12       1       si       4646117   37863        226256
TASK-003        45    48       1       si       9911010   81213        242980
...
TASK-011        75   147       4       si      22453984  204929        461024
TASK-012       170    94       3       si      34572887  269547       2006835
TASK-013       150   165       3       si      66834095  318235       1028522
minutos Real: suma 739 ; mediana 45.5 ; p75 50 ; de 12 a 165 (excluyen el diseno: «Real» va de la rama abierta al merge)
tokens total (incluye la fila de diseno de ## Coste): mediana 13942446.5 ; p75 19081290 ; sin diseno: mediana 13503447 ; p75 18502746
tokens salida: mediana 147315.5 ; p75 204929
```

Lectura:

- **Minutos sin diseño, tokens con diseño.** «Real» de `## Duracion` se
  mide de la rama abierta al merge: no incluye la fase de diseño, que va
  antes de `start`. `## Coste` sí tiene la fila `diseno` en los 12 ADR; por
  eso el total se da con y sin ella. Los minutos de diseño no se pueden
  medir con lo que guarda la 0.16 (ver «No verificado»).
- **Salida aparte del total.** El total lo domina `cache_lectura`: la
  mediana del total es ~14 M de tokens y la de salida ~147 mil, unas 95 veces
  menos (cuenta sobre los dos valores medidos).
- **Cola larga**: 3 de 12 ADR tardaron más de lo estimado (TASK-003, TASK-011
  y TASK-013); el más largo, 165 min, es el de F1-12. De ahí mediana y p75 y
  no la media.
- El p75 es por rango más cercano (el valor en la posición ⌈0,75 n⌉ de la
  lista ordenada); la mediana de una lista par es la media de los dos
  centrales (por eso 45.5).

### A — Estudio de adopción de una copia de OpenDockTrading

```text
tareas por carpeta 00..05: 50 / 0 / 0 / 0 / 44 / 2 ; total 96
-- hecho a mano, sin 2.0: se retiran / se quedan 0 / 1064 ; solas / con sustitucion 0 / 0 ; lineas en huecos bloqueantes 76
-- hecho a mano, 2.0.0 prevista: se retiran / se quedan 91 / 973 ; solas / con sustitucion 15 / 76 ; lineas en huecos bloqueantes 0
-- sin clasificar: ficheros que nombran TaskCode fuera de tareas/ 18 ; sin clasificar 15
-- uso medido: tareas con ## Coste 44/96 ; con minutos Real 34/96
   tokens por fase (total / salida): diseno 57760762 / 1332563 ; implementacion 528147546 / 2689016 ; revision 457621183 / 5046536
   tokens por tarea: mediana 19383656.5 ; p75 27935332
   rondas de revision por tarea (rondas: tareas): 1: 1, 2: 21, 3: 16, 4: 4, 5: 1, 15: 1
   minutos Real: suma 2965 ; mediana 56 ; p75 77
   verificacion: tareas con ## Verificacion ligera 3 ; registro de verificacion por tarea: no medido (la 0.16 no lo guarda en tarea.md)
-- ADR propios de OpenDockTrading: TASK-006, TASK-007, TASK-008, TASK-009, TASK-067 ; con ## Coste 5 ; con minutos Real 3
   TASK-006: Real None min ; tokens total 13990532 ; rondas 3
   TASK-007: Real 40 min ; tokens total 16478511 ; rondas 3
   TASK-008: Real None min ; tokens total 46495955 ; rondas 15
   TASK-009: Real 299 min ; tokens total 20577242 ; rondas 4
   TASK-067: Real 15 min ; tokens total 7523701 ; rondas 3
-- proponer_hito, sin 2.0: 9 elementos ; bloqueantes 3 ; tareas 7 ; ADR 2
   TASK-097 tarea BLOQUEANTE antes de migrar (tarea de la 0.16): scripts/lector-tareas.py: leer la carpeta fija o tc2 tareas --json
   TASK-098 tarea BLOQUEANTE antes de migrar (tarea de la 0.16): scripts/comprobar-modulos.py: leer la carpeta fija o tc2 tareas --json
   TASK-099 tarea BLOQUEANTE antes de migrar (tarea de la 0.16): scripts/comprobar-fallos.py: leer la carpeta fija o tc2 tareas --json
   TASK-100 tarea tras migrar (tc2 import): scripts/verificar.bat (3-5): hueco sin puertas-en-ci
   TASK-101 tarea tras migrar (tc2 import): scripts/verificar.bat (16-18, 35-37, 44-47): hueco sin puertas-en-ci
   TASK-102 tarea tras migrar (tc2 import): scripts/verificar.bat (25): hueco sin cli-2
   TASK-103 tarea tras migrar (tc2 import): scripts/verificar.bat (59): hueco sin roadmap-nativo
   TASK-104 ADR   tras migrar (tc2 import): clasificar lo que nombra TaskCode (codigo: 4 ficheros)
   TASK-105 ADR   tras migrar (tc2 import): clasificar lo que nombra TaskCode (documentacion: 11 ficheros)
   presupuesto ADR (min / tokens, mediana-p75 de H): 91-100 / 27884893-38162580 ; tareas (mediana-p75 del propio proyecto): 392-539 / 135685595.5-195547324
-- proponer_hito, 2.0.0 prevista: 5 elementos ; bloqueantes 0 ; tareas 3 ; ADR 2
hash de la copia (con .git) antes y despues del estudio: igual ; ficheros cambiados 0
```

Lectura:

- **Los bloques de F1-12 se reproducen** con su mismo código en el mismo
  commit: **91 / 973** con la 2.0.0 prevista (15 solas y 76 con
  sustitución), 76 líneas en huecos bloqueantes sin 2.0, y 18 ficheros que
  nombran TaskCode, 15 sin clasificar. El informe sale igual en dos llamadas
  y el estudio no cambia ningún fichero de la copia (`.git` incluido).
- **Uso medido**: OpenDockTrading sí tiene `## Coste` (44 de 96 tareas, las
  terminadas) y «Real» en 34; la mediana de una tarea es 56 min y ~19 M de
  tokens. La revisión gasta casi tanto como la implementación en total y casi
  el doble en salida. **La verificación no se puede medir**: la 0.16 no guarda
  un registro por tarea; solo 3 tareas tienen una sección «Verificacion
  ligera».
- **`proponer_hito` da un elemento por hueco** y separa los bloqueantes. Con
  la 2.0 de hoy (ninguna capacidad), 9 elementos: los 3 lectores de
  OpenDockTrading son **bloqueantes**, tareas de la 0.16 que se hacen antes
  de migrar (son las que F1-12 exige antes de F9-7); 4 huecos de
  `verificar.bat` que entran tras migrar; y 2 ADR para lo que nombra
  TaskCode sin clasificar, uno por grupo (código: `comprobar-entorno.py`,
  `comprobar-secretos.py`, `resumen-verificacion.py`, `verificar.rs`;
  documentación: los otros 11). Con la 2.0.0 prevista quedan 5: las 4 líneas
  de `verificar.bat` se retiran solas y no piden trabajo; los 3 lectores
  pasan a sustitución tras migrar.
- **Presupuesto**: 2 ADR × mediana–p75 de H = **91-100** min y 28-38 M de
  tokens; las 7 tareas, con la mediana–p75 del propio proyecto, 392-539 min.
- **Contraste con los ADR propios de OpenDockTrading**: 5 ADR, «Real» solo en
  3 (40, 299 y 15 min; TASK-006 y TASK-008 son anteriores a `## Duracion`) y
  tokens por ADR de mediana ~16 M, del orden de los de TaskCodeTwo. TASK-008
  (`ADR-003`, máquina de estados contra Kraken) tuvo **15** rondas de
  revisión. Con 3 valores, el p75 de OpenDockTrading es su máximo (299): no
  sirve como semilla; la regla usa el de TaskCodeTwo y lo recalibra F1-17.

### R — Registro `hito-decisiones.yml` y `hito_estado`

Rama de estado de prueba `taskcodetwo/tareas` con las 94 tareas de
TaskCodeTwo en `0cc5bc3`, aplanadas a `TASK-NNN/tarea.md` con el estado de su
carpeta, y un registro distinto en cada commit:

```text
sin registro: ausente (no hay hito-decisiones.yml) ; segunda llamada igual
TaskCodeTwo real: las 18 ADR de la Fase 1: abierto (12 de 18 terminadas o descartadas) ; segunda llamada igual
solo las 12 ADR terminadas: cerrado (12 de 12 terminadas o descartadas) ; segunda llamada igual
lista vacia, firmado con motivo e informe_hash: no-necesario (firmado por carlos (persona)) ; segunda llamada igual
roto: firma sin motivo: abierto (no_necesario sin por, fecha, motivo o informe_hash) ; segunda llamada igual
roto: firma sin informe_hash: abierto (no_necesario sin por, fecha, motivo o informe_hash) ; segunda llamada igual
roto: lista vacia sin firma: abierto (lista vacia sin firma de no necesario) ; segunda llamada igual
roto: tarea que no existe: abierto (12 de 13 terminadas o descartadas) ; segunda llamada igual
roto: YAML ilegible: abierto (registro ilegible) ; segunda llamada igual
roto: sin formato 2: abierto (registro sin formato 2 u origen) ; segunda llamada igual
```

Lectura: el estado sale de leer el registro y los `tarea.md` que lista, sin
guardarlo en ningún sitio. **Hoy el hito de TaskCodeTwo está abierto** (12
de 18 ADR terminadas); es el precedente del relevo (F4-5). Un registro
editado a mano o roto **falla cerrado**: sale `abierto` (6 de 6).

### P — Puerta simulada sobre tareas reales

La misma función pura hace de parada local en `tc2 start` (carril
**declarado** en la tarea) y de check `estado` del PR (carril **efectivo por
ficheros**, regla de F1-10: errata si todas las rutas fuera de `tareas/`
están bajo `docs/`). La parada se mide sobre las tareas planificadas y en
curso; el check, retrospectivamente, sobre los PR de tarea ya mergeados en
`develop` (en la realidad el hito no aplica hacia atrás: es un banco de
pruebas con ficheros reales). El hito de TaskCodeTwo es el de R (18 ADR); el
de OpenDockTrading, los 9 elementos de `proponer_hito`.

```text
tareas con carril declarado (la 0.16 no tiene el campo): 0
-- TaskCodeTwo: tareas del hito 18 ; PR de tarea mergeados 13
   hito ausente: start pasa 0 / se niega 81 ; check estado: errata 12, verde 12, rojo 1 ; codigos [0, 4] ; dos llamadas iguales
   hito abierto: start pasa 6 / se niega 75 ; check estado: errata 12, verde 12, rojo 1 ; codigos [0, 4] ; dos llamadas iguales
      start pasa: TASK-014, TASK-015, TASK-073, TASK-074, TASK-075, TASK-094
      PR errata (solo docs/): TASK-002, ..., TASK-013
      PR en rojo: TASK-001
   hito cerrado: start pasa 81 / se niega 0 ; check estado: errata 12, verde 13, rojo 0 ; codigos [0] ; dos llamadas iguales
   hito no-necesario: start pasa 81 / se niega 0 ; check estado: errata 12, verde 13, rojo 0 ; codigos [0] ; dos llamadas iguales
-- OpenDockTrading: tareas del hito 9 ; PR de tarea mergeados 44
   hito ausente: start pasa 0 / se niega 50 ; check estado: errata 1, verde 1, rojo 43 ; codigos [0, 4] ; dos llamadas iguales
   hito abierto: start pasa 0 / se niega 50 ; check estado: errata 1, verde 1, rojo 43 ; codigos [0, 4] ; dos llamadas iguales
      PR errata (solo docs/): TASK-067
   hito cerrado: start pasa 50 / se niega 0 ; check estado: errata 1, verde 44, rojo 0 ; codigos [0] ; dos llamadas iguales
   hito no-necesario: start pasa 50 / se niega 0 ; check estado: errata 1, verde 44, rojo 0 ; codigos [0] ; dos llamadas iguales
```

Lectura:

- Con el hito **abierto**, `start` solo deja empezar las tareas del hito
  (las 6 ADR pendientes de TaskCodeTwo) y niega las otras 75 con el código
  de precondición; en OpenDockTrading niega las 50 planificadas. Con
  **cerrado** o **no-necesario** pasan todas. Sin registro (**ausente**)
  niega como abierto y, además, no hay tareas del hito.
- **Los 12 PR de ADR de TaskCodeTwo son errata por ficheros**: solo tocan
  `docs/` fuera de `tareas/`. Un ADR pasa la puerta por su carril aunque no
  esté en la lista. En OpenDockTrading, solo `ADR-005` (TASK-067) es
  errata; sus otros ADR tocaron código o configuración.
- **Hallazgo**: el PR de TASK-001 (F1-0, crear el repo) sale en rojo con el
  hito abierto. El arranque del repo va antes de que exista el hito: en un
  proyecto nuevo, ese papel lo hace `tc2 init` (punto 2 de la decisión), que
  no pasa por la puerta.
- Las dos llamadas dan lo mismo en los ocho casos (determinista).

### G — El ADR y sus mutaciones

Las once copias mutadas (una cifra cambiada, una fila borrada, una fila sin
medida, el tiempo de reloj en la tabla, un número inventado en el resumen,
un ID que no existe, un enlace a `tareas/`, un enlace roto, una sección
borrada, sin la fila F1-18 propuesta y el estado «propuesta») fallan cada una
por su motivo (log, sección G).

### Cifras de la PoC

Cada fila es una medida del guion, con la sección que la produce. El guion
falla si un valor no es el medido, si falta una medida o si sobra una fila.

<!-- cifras:inicio -->
| Cifra | Valor | Fuente |
|---|---|---|
| H: ADR con ## Coste y minutos Real | `12/12` | H |
| H: ADR con Real mayor que lo estimado | `3/12` | H |
| H: ADR cuyo ## Coste incluye la fila de diseno | `12/12` | H |
| H: minutos Real por ADR (TASK-002..TASK-013) | `12, 48, 48, 50, 38, 36, 31, 43, 27, 147, 94, 165` | H |
| H: minutos Real: minimo / maximo | `12 / 165` | H |
| H: minutos Real: suma / mediana / p75 | `739 / 45.5 / 50` | H |
| H: minutos estimados: suma | `895` | H |
| H: rondas de revision por ADR (rondas: ADR) | `1: 3, 2: 4, 3: 4, 4: 1` | H |
| H: tokens salida por ADR: mediana / p75 | `147315.5 / 204929` | H |
| H: tokens total por ADR (con diseno): mediana / p75 | `13942446.5 / 19081290` | H |
| H: tokens total por ADR sin la fase de diseno: mediana / p75 | `13503447 / 18502746` | H |
| H: tokens total: suma | `237266233` | H |
| ADR de OpenDockTrading (titulo ADR-NNN): tareas | `TASK-006, TASK-007, TASK-008, TASK-009, TASK-067` | A |
| ADR de OpenDockTrading: con ## Coste / con minutos Real | `5 / 3` | A |
| ADR de OpenDockTrading: minutos Real: suma / mediana / p75 | `354 / 40 / 299` | A |
| ADR de OpenDockTrading: rondas de revision (rondas: ADR) | `3: 3, 4: 1, 15: 1` | A |
| ADR de OpenDockTrading: tokens total: mediana / p75 | `16478511 / 20577242` | A |
| F1-12 recalculado, 2.0.0 prevista: lineas en huecos bloqueantes | `0` | A |
| F1-12 recalculado, 2.0.0 prevista: lineas que se retiran / se quedan | `91 / 973` | A |
| F1-12 recalculado, 2.0.0 prevista: retiradas solas / con sustitucion | `15 / 76` | A |
| F1-12 recalculado, sin 2.0: lineas en huecos bloqueantes | `76` | A |
| F1-12 recalculado, sin 2.0: lineas que se retiran / se quedan | `0 / 1064` | A |
| F1-12 recalculado, sin 2.0: retiradas solas / con sustitucion | `0 / 0` | A |
| F1-12 recalculado: campos de la 0.16 sin uso | `8` | A |
| F1-12 recalculado: ficheros que nombran TaskCode / sin clasificar | `18 / 15` | A |
| OpenDockTrading: commit fijo | `8141a08` | A |
| OpenDockTrading: tareas por carpeta (00..05) | `50 / 0 / 0 / 0 / 44 / 2` | A |
| estudio: ficheros cambiados en la copia (con .git) | `0` | A |
| informe: hash en dos llamadas | `igual` | A |
| proponer_hito, 2.0.0 prevista: elementos / bloqueantes / tareas / ADR | `5 / 0 / 3 / 2` | A |
| proponer_hito, 2.0.0 prevista: presupuesto ADR (min / tokens, mediana-p75) | `91-100 / 27884893-38162580` | A |
| proponer_hito, 2.0.0 prevista: presupuesto tareas (min / tokens, mediana-p75) | `168-231 / 58150969.5-83805996` | A |
| proponer_hito, sin 2.0: elementos / bloqueantes / tareas / ADR | `9 / 3 / 7 / 2` | A |
| proponer_hito, sin 2.0: presupuesto ADR (min / tokens, mediana-p75) | `91-100 / 27884893-38162580` | A |
| proponer_hito, sin 2.0: presupuesto tareas (min / tokens, mediana-p75) | `392-539 / 135685595.5-195547324` | A |
| uso medido: minutos Real: suma / mediana / p75 | `2965 / 56 / 77` | A |
| uso medido: rondas de revision por tarea (rondas: tareas) | `1: 1, 2: 21, 3: 16, 4: 4, 5: 1, 15: 1` | A |
| uso medido: tareas con ## Coste / con minutos Real | `44/96 / 34/96` | A |
| uso medido: tareas con ## Verificacion ligera | `3` | A |
| uso medido: tokens de diseno (total / salida) | `57760762 / 1332563` | A |
| uso medido: tokens de implementacion (total / salida) | `528147546 / 2689016` | A |
| uso medido: tokens de revision (total / salida) | `457621183 / 5046536` | A |
| uso medido: tokens total por tarea: mediana / p75 | `19383656.5 / 27935332` | A |
| hito_estado: TaskCodeTwo real: las 18 ADR de la Fase 1 | `abierto (12 de 18 terminadas o descartadas)` | R |
| hito_estado: lista vacia, firmado con motivo e informe_hash | `no-necesario (firmado por carlos (persona))` | R |
| hito_estado: registros rotos que salen abierto | `6/6` | R |
| hito_estado: sin registro | `ausente (no hay hito-decisiones.yml)` | R |
| hito_estado: solo las 12 ADR terminadas | `cerrado (12 de 12 terminadas o descartadas)` | R |
| P OpenDockTrading, hito abierto: check estado verde / rojo | `1 / 43` | P |
| P OpenDockTrading, hito abierto: start pasa | `ninguna` | P |
| P OpenDockTrading, hito abierto: start pasa / se niega | `0 / 50` | P |
| P OpenDockTrading, hito ausente: check estado verde / rojo | `1 / 43` | P |
| P OpenDockTrading, hito ausente: start pasa / se niega | `0 / 50` | P |
| P OpenDockTrading, hito cerrado: check estado verde / rojo | `44 / 0` | P |
| P OpenDockTrading, hito cerrado: start pasa / se niega | `50 / 0` | P |
| P OpenDockTrading, hito no-necesario: check estado verde / rojo | `44 / 0` | P |
| P OpenDockTrading, hito no-necesario: start pasa / se niega | `50 / 0` | P |
| P OpenDockTrading: PR con carril errata por ficheros | `TASK-067` | P |
| P TaskCodeTwo, hito abierto: check estado verde / rojo | `12 / 1` | P |
| P TaskCodeTwo, hito abierto: start pasa | `TASK-014, TASK-015, TASK-073, TASK-074, TASK-075, TASK-094` | P |
| P TaskCodeTwo, hito abierto: start pasa / se niega | `6 / 75` | P |
| P TaskCodeTwo, hito ausente: check estado verde / rojo | `12 / 1` | P |
| P TaskCodeTwo, hito ausente: start pasa / se niega | `0 / 81` | P |
| P TaskCodeTwo, hito cerrado: check estado verde / rojo | `13 / 0` | P |
| P TaskCodeTwo, hito cerrado: start pasa / se niega | `81 / 0` | P |
| P TaskCodeTwo, hito no-necesario: check estado verde / rojo | `13 / 0` | P |
| P TaskCodeTwo, hito no-necesario: start pasa / se niega | `81 / 0` | P |
| P TaskCodeTwo: PR con carril errata por ficheros | `TASK-002, TASK-003, TASK-004, TASK-005, TASK-006, TASK-007, TASK-008, TASK-009, TASK-010, TASK-011, TASK-012, TASK-013` | P |
| P: tareas con carril declarado en TaskCodeTwo y OpenDockTrading | `0` | P |
<!-- cifras:fin -->

<!-- resumen:inicio -->
Qué deja medido este ADR: un ADR de la Fase 1 de TaskCodeTwo cuesta de
mediana **45.5** min (p75 **50**) sin el diseño, y **13942446.5 / 19081290**
tokens de mediana y p75 con el diseño; los 12 suman **739** min. El estudio
de adopción de OpenDockTrading reproduce el **91 / 973** de F1-12, cambia
**0** ficheros de la copia y propone **9 / 3 / 7 / 2** (elementos,
bloqueantes, tareas, ADR) con la 2.0 de hoy. Con el hito abierto, `start`
deja pasar **6 / 75** en TaskCodeTwo y **0 / 50** en OpenDockTrading; con el
hito cerrado o no necesario, todas. Los registros rotos salen abiertos
(**6/6**). Dónde no llega: la verificación por tarea y los minutos de diseño
no se pueden medir con lo que guarda la 0.16, y los ADR propios de
OpenDockTrading solo tienen «Real» en **5 / 3**.
<!-- resumen:fin -->

## Cómo se ve en GitHub

- El registro es un fichero visible en la raíz de la rama
  `taskcodetwo/tareas`: `hito-decisiones.yml`. Su historia (quién lo abrió,
  quién firmó «no necesario») es la de esa rama; F1-5 la protege contra
  borrado y escritura forzada.
- Con el hito abierto, el check obligatorio **`estado`** (F1-11, decisión 4)
  sale en rojo en un PR cuyo carril efectivo no es `errata` y cuya tarea no
  está en el hito, con el mensaje de la puerta («hito de decisiones abierto:
  ciérralo o firma `tc2 hito no-necesario`»). El PR no se puede mergear.
- La propuesta del hito no crea issues ni etiquetas: es un fichero en formato
  de `import` fuera del repo; las tareas aparecen al importarlo.
- Nada de esto se ha visto en GitHub: la PoC no usa GitHub (ver «No
  verificado»).

## Decisión

1. **Nombre y alcance** (N1). «Hito de decisiones (ADR)»: un paso del
   producto en todo proyecto, en `tc2 init` y en `tc2 actualizar`. Su
   trabajo son tareas normales (ADR o tareas), con su ciclo y su revisión.

2. **Contrato en `init`.**
   - Entrada: un **informe de contexto** con una lista fija de áreas de
     decisión: pila, verificación, ramas y puertas, seguridad, equipo y
     plataforma. Cada área, «aplica» o «no aplica» con su motivo (la lista
     es regla de dominio sin revisar: «Preguntas abiertas»).
   - `tc2 init` crea la rama de estado (F2-3) con `hito-decisiones.yml`
     (`origen: "init"`, `tareas: []`), es decir, **abierto** desde el primer
     momento. `init` no pasa por la puerta: es el arranque del proyecto (lo
     que en este repo hizo F1-0).
   - Salida de `tc2 hito proponer --desde <informe>`: un ADR por área que
     aplica, con presupuesto (punto 4), en un fichero de `import` fuera del
     repo; `tc2 import` crea las tareas y las añade a `tareas` del registro.
   - Cerrado: todas las tareas de la lista `terminada` o `descartada`; o
     firma de «no necesario» (punto 5).

3. **Contrato en `actualizar`** (D4a).
   - Entrada: el **informe ampliado** de `tc2 actualizar --simular` (F1-12,
     PL1): los bloques «uso», «hecho a mano» y «sin clasificar» de F1-12 y un
     bloque nuevo **«uso medido»**: tareas por estado, tareas con `## Coste` y
     con «Real», tokens por fase (total y salida), tokens y minutos por tarea
     (mediana y p75), rondas de revisión por tarea y registros de
     verificación. Lo que el proyecto no guarda sale **«no medido»**; no se
     estima. El hash del plan de F1-12 sigue cubriendo solo lo que migra; el
     informe del hito lleva su propio `informe_hash` (sobre el informe entero).
   - `tc2 hito proponer --desde <informe>` es la función pura
     `proponer_hito(informe) -> PropuestaHito`: **un elemento por hueco**:
     - parche con hueco **bloqueante** (código 5 de F1-12) → una **tarea de
       la 0.16 del propio proyecto, antes de migrar**;
     - parche sin capacidad, o con retirada con sustitución → una tarea tras
       migrar;
     - un parche que se retira solo no pide trabajo; lo propio se queda;
     - lo que nombra TaskCode sin clasificar → **un ADR por grupo**, con dos
       grupos fijos: código (`.py`, `.rs`, `.bat`, `.sh`, `.js`, `.ts`,
       `.ps1`) y documentación (el resto).
   - Tras migrar, `tc2 import` del fichero propuesto (F3-2) y escritura del
     registro (`origen: "actualizar"`, `informe_hash`, tareas migradas del
     hito y nuevas). **`actualizar` no crea el registro en su transacción**:
     rompería el criterio CR1 de F1-12. Hasta que existe, el estado es
     `ausente` y la puerta niega (punto 7): un proyecto migrado no empieza
     código sin declarar su hito.
   - Ejemplo medido, OpenDockTrading en `8141a08` con la 2.0 de hoy: **9
     elementos**, 3 bloqueantes (sus tres lectores de tareas, antes de F9-7),
     4 tareas de `verificar.bat` y 2 ADR (PoC A).

4. **Proporcionalidad** (PR1).
   - **Tamaño = número de huecos** (punto 3) o de áreas que aplican (punto
     2). Cero huecos: propuesta vacía; una persona puede firmar «no
     necesario» con el `informe_hash` como evidencia.
   - **Presupuesto por ADR = mediana–p75 de los ADR de referencia**. Semilla:
     la Fase 1 de TaskCodeTwo (PoC H): **45,5–50 min** de implementación y
     revisión (sin diseño) y **13,9–19,1 M de tokens** totales (con diseño),
     de ellos **147–205 mil de salida**. Se dan el total y la salida por
     separado: el total lo domina `cache_lectura`.
   - **Presupuesto por tarea = mediana–p75 del propio proyecto** si su uso
     está medido (OpenDockTrading: 56–77 min y 19,4–27,9 M por tarea); si
     no, «no medido» y la propuesta no da cifra.
   - El presupuesto informa; **no bloquea**. Recalibración con más proyectos:
     F1-17 (estimación) y F8-7.

5. **Registro y estado calculado** (D1a, D3a). `hito-decisiones.yml` en la
   raíz de `taskcodetwo/tareas`, escrito con la escritura atómica de F1-2:

   ```yaml
   formato: 2
   origen: "init"                 # o "actualizar"
   informe_hash: "<sha256 del informe>"
   tareas: ["TASK-097", "TASK-098"]
   no_necesario: null             # o {por: "<identidad de persona>", fecha: "AAAA-MM-DD", motivo: "..."}
   ```

   `hito_estado(rama)` **se calcula, no se guarda**:
   - `ausente`: no hay registro;
   - `no-necesario`: `no_necesario` con `por`, `fecha` y `motivo` no vacíos y
     `informe_hash` no vacío;
   - `cerrado`: lista no vacía y todas sus tareas `terminada` o `descartada`;
   - `abierto`: todo lo demás, **incluido cualquier registro roto** (YAML
     ilegible, sin `formato: 2` u `origen` válido, lista inválida, firma
     incompleta, lista vacía sin firma, tarea listada que no existe). Falla
     cerrado.
   No depende de si «hito» es `sprint` (pregunta abierta de F1-9).

6. **Comandos deterministas** (F1-11, decisión 10):
   - `tc2 hito estado [--json]`: imprime estado y motivo; sale con 0 si
     `cerrado` o `no-necesario` y con el código de precondición si no.
   - `tc2 hito proponer --desde <informe>`: escribe la propuesta fuera del
     repo; no toca el repo.
   - `tc2 hito no-necesario --motivo "<texto>"`: exige identidad de persona,
     con el mismo mecanismo que `approve` (F2-13); **el agente no puede
     firmarlo** (F1-6).
   - El único hook previsto es un atajo PreToolUse opcional que llama a
     `tc2 hito estado --json`; quitarlo no cambia ninguna garantía.

7. **Puerta** (D2a), una sola función pura `puerta(tarea, carril, estado,
   tareas_del_hito)`:
   - pasa si el hito está `cerrado` o `no-necesario`, si la tarea está en la
     lista del hito o si el carril es `errata`; si no, **código de
     precondición** (4, el de F1-12; F1-4 deja los números a F2) con el
     comando que lo arregla;
   - **parada local en `tc2 start`** con el carril **declarado**, antes de la
     toma atómica (F2-4, F3-6): no toma la tarea;
   - **imposición en el check `estado`** con el carril **efectivo por
     ficheros** (F1-10): declarar `errata` en una tarea que toca código no
     salta la puerta, porque el check ve los ficheros.

8. **TaskCodeTwo como precedente.** Su Fase 1 es su hito de decisiones
   (origen `init`, las 18 ADR). Hoy está **abierto** (12 de 18 terminadas,
   PoC R), lo que casa con la regla de este repo de no escribir código de la
   2.0 hasta cerrar la Fase 1. En el relevo (F4-5) el registro lista esas
   tareas terminadas y sale `cerrado`.

## Divergencias con el plan

1. **Falta la fila F1-18 en el plan.** La tarea existe (TASK-094) pero
   `docs/plan/PLAN-TASKCODE-2.md` no tiene su fila. Se propone, para
   añadirla con `taskctl` al cerrar (este ADR no cambia el plan):

   | ID | Tarea | Criterio de aceptación | Dep. |
   |---|---|---|---|
   | F1-18 | ADR: hito de decisiones (ADR) en `init` y en `actualizar` (RR-13): estudio de adopción que lo dimensiona, registro con estado calculado y puerta en `start` y en el check `estado` | Contrato con un ejemplo medido sobre OpenDockTrading; regla de proporcionalidad con cifras de la Fase 1; comando determinista antes de `start` | F1-11, F1-12 |

2. **Lectura previa del plan**: decía «mediana ~45» min por ADR; la medida es
   **45,5** (p75 50, suma 739, de 12 a 165): coincide con la lectura previa.
3. **Grupos sin clasificar**: el plan decía «grupo sin clasificar → 1 ADR»
   sin definir el grupo; se fijan dos (código y documentación) por extensión.
4. **Presupuesto de las tareas**: el plan solo presupuestaba ADR con las
   cifras de la Fase 1; las tareas usan la mediana–p75 del propio proyecto o
   «no medido».
5. **Registros rotos y `ausente`**: el plan no decía qué hace la puerta con
   un registro roto ni sin registro; los dos niegan (fallo cerrado), y `init`
   crea el registro vacío para que el hito esté abierto desde el principio.
6. **Tokens con diseño y sin diseño**: los minutos «Real» no incluyen el
   diseño y `## Coste` sí; se dan las dos cifras de tokens.
7. **Verificación «no medido»**: el plan pedía medir los registros de
   verificación; la 0.16 no guarda uno por tarea (en OpenDockTrading solo 3
   tareas tienen «Verificacion ligera»).
8. **Check `estado` medido hacia atrás**: la PoC lo aplica a PR ya
   mergeados para tener ficheros reales; el hito no se aplica a la historia.
9. **Hallazgo F1-0**: el PR de arranque del repo (TASK-001) saldría en rojo
   con el hito abierto; se resuelve con `init` fuera de la puerta (punto 2).
10. **Contraste con OpenDockTrading**: no estaba en el plan; se añade porque
    da un segundo proyecto con ADR medidos (5 tareas, «Real» en 3).

## Requisitos y evidencia

| Requisito | Decisión | Evidencia |
|---|---|---|
| RR-13 actualizar proyectos existentes | hito en `actualizar` con el informe ampliado (punto 3) | PoC A: 91 / 973 recalculado, 9 elementos, 0 ficheros cambiados en la copia |
| RR-12 equipo | registro en la rama de estado compartida con escritura atómica; firma con identidad | PoC R; escritura atómica de F1-2 no ejercitada aquí |
| RR-14 se construye con la 0.16 | TaskCodeTwo como precedente (punto 8) | PoC R: abierto, 12 de 18 |
| F1-11 decisión 10, comandos y no hooks | `tc2 hito estado / proponer / no-necesario` (punto 6) | PoC R y P: mismas salidas en dos llamadas |
| F1-11 decisión 4, check `estado` | imposición con carril efectivo (punto 7) | PoC P: 12 / 1 en TaskCodeTwo y 1 / 43 en OpenDockTrading con el hito abierto |
| F1-12 PL1, informe de `--simular` | bloque «uso medido» y `informe_hash` (punto 3) | PoC A: informe igual en dos llamadas |
| F1-12 CR1, criterio de salida | el registro se escribe tras migrar, no en la transacción | por diseño; la PoC no ejecuta `actualizar` |
| F1-12 decisión 8, huecos bloqueantes | bloqueantes como tareas de la 0.16 antes de migrar | PoC A: 3 bloqueantes con la 2.0 de hoy, 0 con la 2.0.0 prevista |
| F1-10, errata solo con `docs/` | `errata` pasa la puerta | PoC P: los 12 PR de ADR de TaskCodeTwo y TASK-067 de OpenDockTrading |
| F1-6, ninguna IA firma | `no-necesario` solo persona | PoC R comprueba la firma completa; la identidad, no (F2-13) |
| Proporcionalidad con cifras | punto 4 | PoC H: 45,5–50 min y 13,9–19,1 M tokens por ADR |

## Consecuencias

- **F9-1** añade a `--simular` el bloque «uso medido» y el `informe_hash`, e
  implementa `proponer_hito` y `tc2 hito proponer`.
- **F3-2** acepta en `import` el fichero de la propuesta y añade las tareas a
  la lista del hito.
- **F3-6** (`start`) llama a la puerta antes de la toma atómica; el check
  `estado` (F1-11) la llama con el carril efectivo.
- **F2-3** crea `hito-decisiones.yml` al crear la rama de estado en `init`;
  **F2-13** da la identidad de persona para `tc2 hito no-necesario`;
  **F3-8** (`doctor`) muestra el estado del hito.
- **F1-17** y **F8-7** recalibran la mediana y el p75 con más proyectos.
- **F9-7** (OpenDockTrading): sus tres lectores son las tareas bloqueantes
  del hito, de la 0.16, antes de migrar; **F9-8** (ToolsaWeb) hace su propio
  estudio (no medido aquí).
- **F4-5**: el registro de TaskCodeTwo lista las 18 ADR de la Fase 1.

## Fuera de alcance

- El manifiesto de recursos al arrancar: F1-14 y F1-15 (aquí solo se cita).
- El formato del fichero de `import`: F3-2.
- Los números definitivos de los códigos de salida: F2 (F1-4).
- El estudio de ToolsaWeb: F9-8.
- Selección de modelo y skills para las tareas del hito: F1-15.

## Preguntas abiertas

Este ADR se planificó con un solo rol (arquitectura). Lo que queda:

- **Dominio**: la lista fija de áreas de `init` (pila, verificación, ramas y
  puertas, seguridad, equipo, plataforma) no la ha revisado nadie de
  dominio; si documentación con código de ejemplo es «de código»; si una
  firma de «no necesario» caduca cuando cambia el informe (hoy no: guarda el
  `informe_hash` como evidencia); si «hito» es `sprint` (F1-9).
- **Tope por hito**: no se fija; el presupuesto informa y no bloquea.
- **Riesgos**: registro borrado por un PR a la rama de estado (lo contiene la
  protección de F1-5, no verificado); clon sin la rama de estado al hacer
  `start` (la puerta lo vería `ausente` y negaría); `no-necesario` sin
  identidad disponible mientras F2-13 no exista.
- **Rendimiento**: `hito_estado` lee un `tarea.md` por tarea del hito, no
  del backlog; no se ha medido con hitos grandes.

## No verificado

- **Ningún comando de la 2.0 existe**: `tc2 hito`, `tc2 init`, `tc2 import` y
  la parada en `start` son funciones de la PoC, no código del producto.
- **GitHub**: ni el check `estado` en rojo ni la protección del registro se
  han visto en un PR real.
- **La identidad de persona** de la firma: la PoC solo comprueba que `por`,
  `fecha`, `motivo` e `informe_hash` no están vacíos (F2-13 no existe).
- **Minutos de diseño**: la 0.16 no guarda cuándo empieza el diseño; los
  minutos de la regla excluyen esa fase.
- **Verificación por tarea**: «no medido» en OpenDockTrading (la 0.16 no la
  registra en `tarea.md`).
- **`init` en un proyecto nuevo real**: la lista de áreas no se ha aplicado
  a ningún proyecto; el ejemplo medido es el de `actualizar`.
- **La escritura atómica del registro** (F1-2) no se ejercita: la PoC hace
  commits locales en una rama de prueba.
- **ToolsaWeb** no se ha medido.
- **macOS y Linux**: todo se ejecutó en Windows.
- **Las cifras son de dos proyectos y una persona**: semilla de la regla, no
  una medida general.

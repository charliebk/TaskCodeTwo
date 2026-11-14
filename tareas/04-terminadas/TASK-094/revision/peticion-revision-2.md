# Peticion de revision — TASK-094 (ronda 2)

- Tarea: TASK-094 — F1-18 ADR hito de decisiones (ADR) en init y en actualizar
- Rama revisada: feature/task-094-f1-18-adr-hito-de-decisiones-adr-en-init
- Rama base: develop
- Commit revisado (HEAD): bdc5b032d60f7aa0697c778e8398d62e2c9401a6
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-094 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 70e685d5f42f815aa972ac9890ab5c8771692018 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (7 hallazgo(s)))

## Instrucciones para el agente revisor (verificacion acotada)

Eres un revisor INDEPENDIENTE del agente que implemento. La rama ya paso una
revision completa: esta ronda verifica las correcciones de la ronda anterior,
no vuelve a revisar la rama entera ni los MENORES de la ronda anterior (tu
tabla solo trae CRITICOS e IMPORTANTES). Haz esto, y nada mas:

1. Comprueba, uno por uno, que cada hallazgo pendiente de tu tabla de abajo
   queda resuelto por el diff de esta ronda (los marcados `corregido` tambien:
   verificalo, no lo des por bueno). Para un CRITICO o IMPORTANTE, reproduce el
   fallo original y comprueba que ya no ocurre; puedes leer el repo entero si la
   causa no esta en el diff.
2. Ejecuta la suite (o el comando de verificacion del proyecto) sobre el commit
   revisado y di el resultado: verificar sin ejecutar no vale. Si esta peticion
   ya trae la verificacion de la ronda en verde para ese mismo commit, no la
   repitas: cita ese resultado y corre solo los tests de lo que verificas.
3. Busca en el diff de esta ronda el fallo que la correccion abre: es justo
   donde se cuelan los fallos nuevos. Si encuentras uno, construye el caso que
   lo rompe antes de reportarlo.

Solo abre un hallazgo NUEVO si es CRITICO o si lo introduce la propia correccion
(algo que el diff de esta ronda rompe). Lo demas que veas fuera de eso va como
nota al final del informe, no en la tabla: no abre otra ronda.

Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. En la tabla, cada hallazgo pendiente que ya este
resuelto va como `corregido`; el que no, `abierto`. Vuelca tu salida en el
informe de esta ronda (informe-revision-2.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-094/tarea.md
- tareas/03-en-revision/TASK-094/planificacion/
- docs/adr/F1-18-hito-de-decisiones.md
- docs/adr/README.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 4 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-18-hito-de-decisiones.md:452 | informe-revision-1.md |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-18-hito-de-decisiones.md:476 | informe-revision-1.md |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-18-hito-de-decisiones.md:416 | informe-revision-1.md |

## Commits a revisar (git log 70e685d5f42f815aa972ac9890ab5c8771692018..HEAD)

````
bdc5b03 chore(TASK-094): veredicto de la ronda 1 corregido a cambios-solicitados (habia IMPORTANTE abiertos)
bea8fdc chore(TASK-094): informe de revision 1 corregido en 283d19c y constancia de autorrevision ronda 2
283d19c docs(adr): F1-18 corrige los hallazgos de la revision 1
ced700d chore(TASK-094): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 70e685d5f42f815aa972ac9890ab5c8771692018..HEAD)

````diff
diff --git a/docs/adr/F1-18-hito-de-decisiones.md b/docs/adr/F1-18-hito-de-decisiones.md
index ef7f71e..0c6540e 100644
--- a/docs/adr/F1-18-hito-de-decisiones.md
+++ b/docs/adr/F1-18-hito-de-decisiones.md
@@ -3,7 +3,7 @@
 - Estado: **aceptada** (2026-11-14)
 - Tarea: TASK-094 · Plan: el [PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) **no tiene fila F1-18**; se propone en «Divergencias con el plan»
 - Requisitos: RR-13 (actualizar proyectos existentes) y RR-12 (equipo) de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); RR-14 (se construye con la 0.16) como precedente
-- Depende de: [ADR F1-11](F1-11-extensiones-declaradas.md) (inventario parche/propio, check `estado`, decisión 10 «comandos sí, hooks solo como atajo opcional») y [ADR F1-12](F1-12-actualizacion-entre-versiones.md) (informe de `tc2 actualizar --simular`, transacción con criterio CR1, huecos bloqueantes con código 5, decisión 8); usa [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) y [ADR F1-2](F1-2-escrituras-atomicas-ids-y-toma.md) (rama de estado y escritura atómica), [ADR F1-6](F1-6-identidades-agente-revisor-persona.md) (ninguna IA firma lo que firma una persona) y [ADR F1-10](F1-10-carriles-y-modo-automatico.md) (carril efectivo por ficheros, errata solo con `docs/`)
+- Depende de: [ADR F1-11](F1-11-extensiones-declaradas.md) (inventario parche/propio, check `estado`, decisión 10 «comandos sí, hooks solo como atajo opcional») y [ADR F1-12](F1-12-actualizacion-entre-versiones.md) (informe de `tc2 actualizar --simular`, transacción con criterio CR1, huecos bloqueantes con código 5, decisión 8); usa [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) y [ADR F1-2](F1-2-escrituras-atomicas-ids-y-toma.md) (rama de estado y escritura atómica), [ADR F1-4](F1-4-operaciones-git-y-contrato-de-salida.md) (contrato de salida y códigos), [ADR F1-6](F1-6-identidades-agente-revisor-persona.md) (ninguna IA firma lo que firma una persona) y [ADR F1-10](F1-10-carriles-y-modo-automatico.md) (carril efectivo por ficheros, errata solo con `docs/`)
 - Decide: Carlos, con sus decisiones del 2026-11-14 (se llama «Hito de decisiones (ADR)», no «Hito 0»; obligatorio pero proporcional; comando determinista y hooks solo como atajo; nombres `tc2`, `taskcodetwo` y rama `taskcodetwo/tareas`; orden de pilotos de F1-12; no trocear; PoC con datos medidos) y la consigna «la opción recomendada» (D1a, D2a, D3a, D4a y D5a del plan de TASK-094)
 
 ## Contexto
@@ -38,17 +38,20 @@ proporcionalidad con cifras y la puerta.
 | | Descripción | Veredicto |
 |---|---|---|
 | **N1** | «Hito de decisiones (ADR)», paso del producto en `init` y en `actualizar` | **Elegida** (decisión de Carlos) |
-| N2 | «Hito 0», solo en proyectos nuevos | Descartada por Carlos: en un proyecto existente es donde más se hace a mano (PoC A: 9 huecos en OpenDockTrading) |
-| **D1a** | Registro `hito-decisiones.yml` en la raíz de la rama `taskcodetwo/tareas`, **estado calculado** | **Elegida**: compartido por el equipo (RR-12), escrito con la escritura atómica de F1-2; cuatro estados y seis registros rotos medidos (PoC R) |
+| N2 | «Hito 0», solo en proyectos nuevos | Descartada: Carlos fijó el nombre «Hito de decisiones (ADR)» y que sea un paso de todo proyecto (plan de TASK-094); el motivo que añade este ADR es medido: un proyecto existente también tiene huecos (PoC A: 9 elementos en OpenDockTrading con la 2.0 de hoy, 5 con la 2.0.0 prevista) |
+| **D1a** | Registro `hito-decisiones.yml` en la raíz de la rama `taskcodetwo/tareas`, **estado calculado** | **Elegida**: compartido por el equipo (RR-12), escrito con la escritura atómica de F1-2; cuatro estados, ediciones sin firma, firmas que no cierran y registros rotos medidos (PoC R) |
 | D1b | Clave en `.taskcode/config.yml` | Descartada: la config viaja en la rama de código y cambia por PR; el estado del hito es estado de tareas (F1-1) |
 | D1c | Solo una etiqueta en las tareas, sin registro | Descartada: no distingue «cerrado» de «nadie lo abrió» ni guarda la firma de «no necesario» |
 | — | Fichero bajo `.git/taskcodetwo/` | Descartada: es local de cada clon y rompe el trabajo en grupo |
 | **D2a** | Tarea de código = fuera de la lista del hito **y** carril efectivo distinto de `errata`; `start` para y el check `estado` impone | **Elegida**: con el hito abierto, 6 de 81 tareas de TaskCodeTwo pueden empezar (las ADR pendientes) y 0 de 50 de OpenDockTrading (PoC P) |
 | D2b | Por el campo `tipo` | Descartada: F1-9 midió que casi no se usa (en OpenDockTrading es uno de los 8 campos sin ningún valor propio, PoC A) |
 | D2c | Todas menos las del hito | Descartada: bloquearía una errata de documentación sin motivo |
-| **D3a** | «No necesario» solo lo firma una persona con identidad, con `motivo` e `informe_hash` obligatorios | **Elegida**: una firma sin motivo o sin `informe_hash` sale `abierto` (PoC R) |
+| **D3a** | «No necesario» solo lo firma una persona con identidad, con `motivo` e `informe_hash` obligatorios | **Elegida**: una firma sin motivo, sin `informe_hash`, escrita por el agente o a nombre de otra identidad sale `abierto` (PoC R) |
+| **FI1** | **Firma = el commit de la rama de estado que escribe el registro con identidad de persona** (mismo mecanismo que `approve`, F1-6 y F2-13); vale la **versión firmada**: un registro distinto de ella sale `abierto` y la puerta usa la lista firmada | **Elegida**: las 5 ediciones válidas sin firma salen `abierto` y las 4 mutaciones de control de `hito_estado` las delata algún caso (PoC R) |
+| FI2 | Hash de la lista guardado dentro del registro | Descartada: el agente puede recalcularlo; no hay secreto |
+| FI3 | Solo la protección de la rama de estado (F1-5) | Descartada: el agente escribe en la rama de estado (transiciones), así que la protección no distingue quién cambia la lista |
 | D3b | Cualquiera, con confirmación interactiva | Descartada: un agente en modo automático confirmaría (F1-6) |
-| **D4a** | `--simular` → `tc2 hito proponer` (fichero fuera del repo, formato de `import`) → los bloqueantes antes de migrar como tareas de la 0.16 → tras migrar, `tc2 import` | **Elegida**: no toca la transacción de F1-12 (CR1); el estudio no cambia ningún fichero de la copia (PoC A) |
+| **D4a** | `--simular` → `tc2 hito proponer` (fichero fuera del repo, formato de `import`) → los bloqueantes antes de migrar como tareas de la 0.16 → tras migrar, la persona lanza `tc2 hito abrir --desde <propuesta>` (o `tc2 hito no-necesario`), que crea las tareas y el registro firmado | **Elegida**: no toca la transacción de F1-12 (CR1); el estudio no cambia ningún fichero de la copia (PoC A) y el flujo completo llega a `cerrado` y a `no-necesario` sin atascarse (PoC R2) |
 | D4b | Crear las tareas del hito dentro de la transacción de `actualizar` | Descartada: rompe el criterio de salida estructural de F1-12 (la instantánea de tareas antes y después dejaría de casar) |
 | **D5a** | El ADR anota la fila F1-18 que falta en el plan y la propone con dependencias F1-11 y F1-12 | **Elegida**: el plan se cambia con `taskctl` al cerrar, no aquí |
 | D5b | Editar el plan en esta tarea | Descartada: los ADR no editan el plan (mismo criterio que F1-12) |
@@ -61,7 +64,8 @@ Guion `poc-f1-18.py` y su salida `poc-f1-18.log`, en la carpeta `poc/` de
 TASK-094 (sin enlace: la carpeta de la tarea cambia de estado al cerrarse).
 Python 3.12.10 (`python -I -B`), PyYAML 6.0.3 y Git 2.56.0, con la carpeta de
 trabajo en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-094\poc\`
-(borrada al terminar; el guion se niega a borrar una carpeta que no esté bajo
+(el guion la borra al empezar y al terminar, también si algo falla, y lo
+comprueba; se niega a borrar una carpeta que no esté bajo
 `pruebas-claude/<proyecto>/<motivo>` o que no lleve su marca).
 
 ```bash
@@ -84,7 +88,8 @@ python -I -B tareas/<estado>/TASK-094/poc/poc-f1-18.py \
   fila; también comprueba el `resumen`, las secciones, el estado, los IDs
   citados (existen en el plan o en los requisitos; F1-18 solo si
   «Divergencias con el plan» propone su fila), los enlaces y la fila del
-  índice. Once copias mutadas del ADR fallan cada una por su motivo. El
+  índice, y que estén la regla de la firma de la lista y el código de F1-4. Catorce
+copias mutadas del ADR fallan cada una por su motivo. El
   tiempo de reloj va a stderr y no es una cifra comprobada.
 
 Salida observada, recortada (la completa está en el log):
@@ -175,9 +180,10 @@ Lectura:
   `verificar.bat` que entran tras migrar; y 2 ADR para lo que nombra
   TaskCode sin clasificar, uno por grupo (código: `comprobar-entorno.py`,
   `comprobar-secretos.py`, `resumen-verificacion.py`, `verificar.rs`;
-  documentación: los otros 11). Con la 2.0.0 prevista quedan 5: las 4 líneas
-  de `verificar.bat` se retiran solas y no piden trabajo; los 3 lectores
-  pasan a sustitución tras migrar.
+  documentación: los otros 11). Con la 2.0.0 prevista quedan 5: los 4 huecos
+  de `verificar.bat` (4 tareas con la 2.0 de hoy; 15 líneas en total) se
+  retiran solos y no piden trabajo, y los 3 lectores pasan a sustitución tras
+  migrar.
 - **Presupuesto**: 2 ADR × mediana–p75 de H = **91-100** min y 28-38 M de
   tokens; las 7 tareas, con la mediana–p75 del propio proyecto, 392-539 min.
 - **Contraste con los ADR propios de OpenDockTrading**: 5 ADR, «Real» solo en
@@ -187,54 +193,107 @@ Lectura:
   revisión. Con 3 valores, el p75 de OpenDockTrading es su máximo (299): no
   sirve como semilla; la regla usa el de TaskCodeTwo y lo recalibra F1-17.
 
-### R — Registro `hito-decisiones.yml` y `hito_estado`
+### R — Registro `hito-decisiones.yml`, firma de persona y `hito_estado`
 
-Rama de estado de prueba `taskcodetwo/tareas` con las 94 tareas de
-TaskCodeTwo en `0cc5bc3`, aplanadas a `TASK-NNN/tarea.md` con el estado de su
-carpeta, y un registro distinto en cada commit:
+Ramas de estado de prueba `taskcodetwo/tareas` con las tareas reales
+aplanadas a `TASK-NNN/tarea.md` (el estado, el de su carpeta). La **firma**
+es el autor del commit que escribe el registro: una identidad de persona
+(`carlos@persona.invalid`) o la del agente (`agente@agente.invalid`); en la
+PoC es el correo del autor (ver «No verificado»).
+
+**R1 — TaskCodeTwo** (las 94 tareas en `0cc5bc3`). La persona abre el hito
+con las 18 ADR de la Fase 1 (`tc2 hito abrir --tareas`, que marca
+`hito: true` en sus `tarea.md` y firma la lista). Sobre ese registro, el
+agente hace cinco ediciones **válidas** (YAML correcto) sin firma; la persona
+firma dos registros que no deben cerrar y uno que sí:
+
+```text
+sin registro: ausente (no hay hito-decisiones.yml) ; tareas del hito 0
+TaskCodeTwo real: las 18 ADR de la Fase 1: abierto (12 de 18 cerradas; TASK-014 planificada) ; tareas del hito 18
+edicion valida sin firma: lista cambiada a TASK-001 solo: abierto (registro distinto de la version firmada (5587bb46e292)) ; tareas del hito 18
+edicion valida sin firma: ID de codigo anadido a la lista (TASK-016): abierto (registro distinto de la version firmada (5587bb46e292)) ; tareas del hito 18
+edicion valida sin firma: lista reducida a las 12 terminadas: abierto (registro distinto de la version firmada (5587bb46e292)) ; tareas del hito 18
+edicion valida sin firma: no_necesario escrito por el agente: abierto (registro distinto de la version firmada (5587bb46e292)) ; tareas del hito 18
+edicion valida sin firma: todas las pendientes descartadas en su tarea.md: abierto (12 de 18 cerradas; TASK-014 descartada sin motivo firmado) ; tareas del hito 18
+firmado: lista con TASK-001 (sin hito: true): abierto (0 de 1 cerradas; TASK-001 no es tarea del hito) ; tareas del hito 0
+firmado: pendientes descartadas con motivo firmado: cerrado (18 de 18 terminadas o descartadas con motivo firmado) ; tareas del hito 18
+firmado: no_necesario a nombre de otra identidad: abierto (no_necesario firmado por otra identidad que la del commit) ; tareas del hito 18
+roto (firmado): firma sin motivo | sin informe_hash | lista vacia sin firma | tarea que no existe | YAML ilegible | sin formato 2: abierto (6 de 6)
+transiciones del agente: las 18 terminadas: cerrado (18 de 18 terminadas o descartadas con motivo firmado) ; tareas del hito 18
+```
+
+**R2 — Flujo de `actualizar` en OpenDockTrading**, de principio a fin, sobre
+las 96 tareas en `8141a08` como quedarían tras migrar (commit del agente) y
+la propuesta con la 2.0.0 prevista (5 elementos, ninguno bloqueante: con la
+2.0 de hoy `actualizar` se niega mientras queden los 3 bloqueantes):
 
 ```text
-sin registro: ausente (no hay hito-decisiones.yml) ; segunda llamada igual
-TaskCodeTwo real: las 18 ADR de la Fase 1: abierto (12 de 18 terminadas o descartadas) ; segunda llamada igual
-solo las 12 ADR terminadas: cerrado (12 de 12 terminadas o descartadas) ; segunda llamada igual
-lista vacia, firmado con motivo e informe_hash: no-necesario (firmado por carlos (persona)) ; segunda llamada igual
-roto: firma sin motivo: abierto (no_necesario sin por, fecha, motivo o informe_hash) ; segunda llamada igual
-roto: firma sin informe_hash: abierto (no_necesario sin por, fecha, motivo o informe_hash) ; segunda llamada igual
-roto: lista vacia sin firma: abierto (lista vacia sin firma de no necesario) ; segunda llamada igual
-roto: tarea que no existe: abierto (12 de 13 terminadas o descartadas) ; segunda llamada igual
-roto: YAML ilegible: abierto (registro ilegible) ; segunda llamada igual
-roto: sin formato 2: abierto (registro sin formato 2 u origen) ; segunda llamada igual
+1. migrado (actualizar no escribe el registro): ausente ; start pasa / se niega 0 / 50
+1b. el agente escribe no-necesario: abierto ; start pasa / se niega 0 / 50
+2. la persona abre el hito con la propuesta: abierto ; start pasa / se niega 5 / 50
+3. el agente termina las tareas del hito: cerrado ; start pasa / se niega 50 / 0
+2'. la persona firma no-necesario sin registro previo: no-necesario ; start pasa / se niega 50 / 0
 ```
 
-Lectura: el estado sale de leer el registro y los `tarea.md` que lista, sin
-guardarlo en ningún sitio. **Hoy el hito de TaskCodeTwo está abierto** (12
-de 18 ADR terminadas); es el precedente del relevo (F4-5). Un registro
-editado a mano o roto **falla cerrado**: sale `abierto` (6 de 6).
+**Mutaciones de control de `hito_estado`**: se desactiva una comprobación
+cada vez y algún caso de R tiene que dar otro estado.
+
+```text
+sin la comprobacion «firma»: detectada por 2 casos (edicion valida sin firma: lista reducida a las 12 terminadas)
+sin la comprobacion «marca»: detectada por 1 casos (firmado: lista con TASK-001 (sin hito: true))
+sin la comprobacion «descartes»: detectada por 1 casos (edicion valida sin firma: todas las pendientes descartadas en su tarea.md)
+sin la comprobacion «por»: detectada por 1 casos (firmado: no_necesario a nombre de otra identidad)
+```
+
+Lectura:
+
+- **Sin firma de persona no hay `cerrado` ni `no-necesario`.** Las 5
+  ediciones válidas del agente salen `abierto`; las cuatro que tocan el
+  registro, porque no es la versión firmada; la que descarta tareas en su
+  `tarea.md`, porque los descartes sin motivo firmado no cierran. En todas,
+  las tareas del hito que usa la puerta son las 18 de la versión firmada: el
+  ID de código metido sin firma (TASK-016) no pasa la puerta.
+- **Una firma no basta si la lista no es del hito**: la persona que firma
+  `TASK-001` (una tarea sin `hito: true`) deja el hito `abierto`. Descartar
+  tareas del hito cierra solo con su motivo en el registro firmado.
+- **Hoy el hito de TaskCodeTwo está abierto** (12 de 18 ADR terminadas); con
+  las 18 terminadas por transiciones del agente, y la lista firmada sin
+  cambios, sale `cerrado`.
+- **El flujo de `actualizar` no se atasca**: tras migrar el estado es
+  `ausente` y `start` niega las 50 planificadas; el `no-necesario` que escribe
+  el agente no vale; la persona abre el hito (5 tareas que sí pueden
+  empezar) o firma `no-necesario` sin registro previo, y en los dos caminos se
+  llega a dejar pasar todo.
+- Las 22 comprobaciones dan lo mismo en dos llamadas.
 
 ### P — Puerta simulada sobre tareas reales
 
 La misma función pura hace de parada local en `tc2 start` (carril
-**declarado** en la tarea) y de check `estado` del PR (carril **efectivo por
+**declarado** en la tarea; ninguna tarea real lo declara: la 0.16 no tiene el
+campo) y de check `estado` del PR (carril **efectivo por
 ficheros**, regla de F1-10: errata si todas las rutas fuera de `tareas/`
 están bajo `docs/`). La parada se mide sobre las tareas planificadas y en
 curso; el check, retrospectivamente, sobre los PR de tarea ya mergeados en
 `develop` (en la realidad el hito no aplica hacia atrás: es un banco de
-pruebas con ficheros reales). El hito de TaskCodeTwo es el de R (18 ADR); el
-de OpenDockTrading, los 9 elementos de `proponer_hito`.
+pruebas con ficheros reales). Las tareas del hito son las de la **versión
+firmada** que devuelve `hito_estado`: las 18 ADR de TaskCodeTwo (R1) y las 5
+tareas que abre la persona en OpenDockTrading (R2). La negativa sale con el
+código `rechazada` (3) del contrato de F1-4 y la línea
+`start rechazada id=TASK-016 motivo=hito-abierto`.
 
 ```text
 tareas con carril declarado (la 0.16 no tiene el campo): 0
--- TaskCodeTwo: tareas del hito 18 ; PR de tarea mergeados 13
-   hito ausente: start pasa 0 / se niega 81 ; check estado: errata 12, verde 12, rojo 1 ; codigos [0, 4] ; dos llamadas iguales
-   hito abierto: start pasa 6 / se niega 75 ; check estado: errata 12, verde 12, rojo 1 ; codigos [0, 4] ; dos llamadas iguales
+-- TaskCodeTwo: tareas del hito (version firmada) 18 ; PR de tarea mergeados 13
+   hito ausente: start pasa 0 / se niega 81 ; check estado: errata 12, verde 12, rojo 1 ; codigos [0, 3] ; dos llamadas iguales
+   hito abierto: start pasa 6 / se niega 75 ; check estado: errata 12, verde 12, rojo 1 ; codigos [0, 3] ; dos llamadas iguales
       start pasa: TASK-014, TASK-015, TASK-073, TASK-074, TASK-075, TASK-094
       PR errata (solo docs/): TASK-002, ..., TASK-013
       PR en rojo: TASK-001
    hito cerrado: start pasa 81 / se niega 0 ; check estado: errata 12, verde 13, rojo 0 ; codigos [0] ; dos llamadas iguales
    hito no-necesario: start pasa 81 / se niega 0 ; check estado: errata 12, verde 13, rojo 0 ; codigos [0] ; dos llamadas iguales
--- OpenDockTrading: tareas del hito 9 ; PR de tarea mergeados 44
-   hito ausente: start pasa 0 / se niega 50 ; check estado: errata 1, verde 1, rojo 43 ; codigos [0, 4] ; dos llamadas iguales
-   hito abierto: start pasa 0 / se niega 50 ; check estado: errata 1, verde 1, rojo 43 ; codigos [0, 4] ; dos llamadas iguales
+-- OpenDockTrading: tareas del hito (version firmada) 5 ; PR de tarea mergeados 44
+   hito ausente: start pasa 0 / se niega 50 ; check estado: errata 1, verde 1, rojo 43 ; codigos [0, 3] ; dos llamadas iguales
+   hito abierto: start pasa 0 / se niega 50 ; check estado: errata 1, verde 1, rojo 43 ; codigos [0, 3] ; dos llamadas iguales
       PR errata (solo docs/): TASK-067
    hito cerrado: start pasa 50 / se niega 0 ; check estado: errata 1, verde 44, rojo 0 ; codigos [0] ; dos llamadas iguales
    hito no-necesario: start pasa 50 / se niega 0 ; check estado: errata 1, verde 44, rojo 0 ; codigos [0] ; dos llamadas iguales
@@ -243,8 +302,8 @@ tareas con carril declarado (la 0.16 no tiene el campo): 0
 Lectura:
 
 - Con el hito **abierto**, `start` solo deja empezar las tareas del hito
-  (las 6 ADR pendientes de TaskCodeTwo) y niega las otras 75 con el código
-  de precondición; en OpenDockTrading niega las 50 planificadas. Con
+  (las 6 ADR pendientes de TaskCodeTwo) y niega las otras 75 con
+  `rechazada` (3); en OpenDockTrading niega las 50 planificadas. Con
   **cerrado** o **no-necesario** pasan todas. Sin registro (**ausente**)
   niega como abierto y, además, no hay tareas del hito.
 - **Los 12 PR de ADR de TaskCodeTwo son errata por ficheros**: solo tocan
@@ -259,11 +318,12 @@ Lectura:
 
 ### G — El ADR y sus mutaciones
 
-Las once copias mutadas (una cifra cambiada, una fila borrada, una fila sin
-medida, el tiempo de reloj en la tabla, un número inventado en el resumen,
-un ID que no existe, un enlace a `tareas/`, un enlace roto, una sección
-borrada, sin la fila F1-18 propuesta y el estado «propuesta») fallan cada una
-por su motivo (log, sección G).
+Las catorce copias mutadas (una cifra cambiada, una fila borrada, una fila
+sin medida, el tiempo de reloj en la tabla, un número inventado en el
+resumen, un ID que no existe, un enlace a `tareas/`, un enlace roto, una
+sección borrada, sin la fila F1-18 propuesta, el estado «propuesta», sin la
+regla de la firma de la lista, el código 4 atribuido a F1-12 y la cifra de
+ediciones sin firma cambiada) fallan cada una por su motivo (log, sección G).
 
 ### Cifras de la PoC
 
@@ -316,11 +376,19 @@ falla si un valor no es el medido, si falta una medida o si sobra una fila.
 | uso medido: tokens de implementacion (total / salida) | `528147546 / 2689016` | A |
 | uso medido: tokens de revision (total / salida) | `457621183 / 5046536` | A |
 | uso medido: tokens total por tarea: mediana / p75 | `19383656.5 / 27935332` | A |
-| hito_estado: TaskCodeTwo real: las 18 ADR de la Fase 1 | `abierto (12 de 18 terminadas o descartadas)` | R |
-| hito_estado: lista vacia, firmado con motivo e informe_hash | `no-necesario (firmado por carlos (persona))` | R |
+| hito_estado: TaskCodeTwo real: las 18 ADR de la Fase 1 | `abierto (12 de 18 cerradas)` | R |
+| hito_estado: TaskCodeTwo con las 18 ADR terminadas (lista firmada sin cambios) | `cerrado (18 de 18 terminadas o descartadas con motivo firmado)` | R |
+| hito_estado: ediciones validas sin firma de persona que salen abierto | `5/5` | R |
+| hito_estado: firmado, lista con TASK-001 (sin hito: true) | `abierto (TASK-001 no es tarea del hito)` | R |
+| hito_estado: firmado, pendientes descartadas con motivo firmado | `cerrado` | R |
+| hito_estado: casos comprobados | `22` | R |
+| hito_estado: mutaciones de control detectadas | `4/4` | R |
+| flujo actualizar OpenDockTrading: 1. migrado (actualizar no escribe el registro) | `ausente ; start 0 / 50` | R |
+| flujo actualizar OpenDockTrading: 1b. el agente escribe no-necesario | `abierto ; start 0 / 50` | R |
+| flujo actualizar OpenDockTrading: 2. la persona abre el hito con la propuesta | `abierto ; start 5 / 50` | R |
+| flujo actualizar OpenDockTrading: 3. el agente termina las tareas del hito | `cerrado ; start 50 / 0` | R |
+| flujo actualizar OpenDockTrading: 2'. la persona firma no-necesario sin registro previo | `no-necesario ; start 50 / 0` | R |
 | hito_estado: registros rotos que salen abierto | `6/6` | R |
-| hito_estado: sin registro | `ausente (no hay hito-decisiones.yml)` | R |
-| hito_estado: solo las 12 ADR terminadas | `cerrado (12 de 12 terminadas o descartadas)` | R |
 | P OpenDockTrading, hito abierto: check estado verde / rojo | `1 / 43` | P |
 | P OpenDockTrading, hito abierto: start pasa | `ninguna` | P |
 | P OpenDockTrading, hito abierto: start pasa / se niega | `0 / 50` | P |
@@ -342,6 +410,7 @@ falla si un valor no es el medido, si falta una medida o si sobra una fila.
 | P TaskCodeTwo, hito no-necesario: start pasa / se niega | `81 / 0` | P |
 | P TaskCodeTwo: PR con carril errata por ficheros | `TASK-002, TASK-003, TASK-004, TASK-005, TASK-006, TASK-007, TASK-008, TASK-009, TASK-010, TASK-011, TASK-012, TASK-013` | P |
 | P: tareas con carril declarado en TaskCodeTwo y OpenDockTrading | `0` | P |
+| P: codigo y linea de la negativa | `3, start rechazada id=TASK-016 motivo=hito-abierto` | P |
 <!-- cifras:fin -->
 
 <!-- resumen:inicio -->
@@ -352,8 +421,10 @@ de adopción de OpenDockTrading reproduce el **91 / 973** de F1-12, cambia
 **0** ficheros de la copia y propone **9 / 3 / 7 / 2** (elementos,
 bloqueantes, tareas, ADR) con la 2.0 de hoy. Con el hito abierto, `start`
 deja pasar **6 / 75** en TaskCodeTwo y **0 / 50** en OpenDockTrading; con el
-hito cerrado o no necesario, todas. Los registros rotos salen abiertos
-(**6/6**). Dónde no llega: la verificación por tarea y los minutos de diseño
+hito cerrado o no necesario, todas. Sin firma de persona no se cierra: las
+ediciones válidas sin firma salen abiertas (**5/5**), igual que los
+registros rotos (**6/6**), y las mutaciones de control de `hito_estado` las
+delata algún caso (**4/4**). Dónde no llega: la verificación por tarea y los minutos de diseño
 no se pueden medir con lo que guarda la 0.16, y los ADR propios de
 OpenDockTrading solo tienen «Real» en **5 / 3**.
 <!-- resumen:fin -->
@@ -361,13 +432,15 @@ OpenDockTrading solo tienen «Real» en **5 / 3**.
 ## Cómo se ve en GitHub
 
 - El registro es un fichero visible en la raíz de la rama
-  `taskcodetwo/tareas`: `hito-decisiones.yml`. Su historia (quién lo abrió,
-  quién firmó «no necesario») es la de esa rama; F1-5 la protege contra
-  borrado y escritura forzada.
+  `taskcodetwo/tareas`: `hito-decisiones.yml`. Su historia es la de esa
+  rama: la **firma** es el commit que lo escribe con la identidad de una
+  persona (F1-6), y el check `estado` compara el registro con esa versión
+  firmada. F1-5 protege la rama contra borrado y escritura forzada.
 - Con el hito abierto, el check obligatorio **`estado`** (F1-11, decisión 4)
   sale en rojo en un PR cuyo carril efectivo no es `errata` y cuya tarea no
-  está en el hito, con el mensaje de la puerta («hito de decisiones abierto:
-  ciérralo o firma `tc2 hito no-necesario`»). El PR no se puede mergear.
+  es del hito firmado, con la línea de la puerta (`rechazada`,
+  `motivo=hito-abierto`) y el comando que lo arregla (`tc2 hito estado`). El
+  PR no se puede mergear.
 - La propuesta del hito no crea issues ni etiquetas: es un fichero en formato
   de `import` fuera del repo; las tareas aparecen al importarlo.
 - Nada de esto se ha visto en GitHub: la PoC no usa GitHub (ver «No
@@ -384,13 +457,14 @@ OpenDockTrading solo tienen «Real» en **5 / 3**.
      decisión: pila, verificación, ramas y puertas, seguridad, equipo y
      plataforma. Cada área, «aplica» o «no aplica» con su motivo (la lista
      es regla de dominio sin revisar: «Preguntas abiertas»).
-   - `tc2 init` crea la rama de estado (F2-3) con `hito-decisiones.yml`
-     (`origen: "init"`, `tareas: []`), es decir, **abierto** desde el primer
-     momento. `init` no pasa por la puerta: es el arranque del proyecto (lo
-     que en este repo hizo F1-0).
+   - `tc2 init` lo lanza una persona y crea la rama de estado (F2-3) con
+     `hito-decisiones.yml` firmado (`origen: "init"`, `tareas: []`), es decir,
+     **abierto** desde el primer momento. `init` no pasa por la puerta: es el
+     arranque del proyecto (lo que en este repo hizo F1-0).
    - Salida de `tc2 hito proponer --desde <informe>`: un ADR por área que
      aplica, con presupuesto (punto 4), en un fichero de `import` fuera del
-     repo; `tc2 import` crea las tareas y las añade a `tareas` del registro.
+     repo; la persona lanza `tc2 hito abrir --desde <propuesta>`, que crea las
+     tareas (con `hito: true`) y firma la lista (punto 5).
    - Cerrado: todas las tareas de la lista `terminada` o `descartada`; o
      firma de «no necesario» (punto 5).
 
@@ -413,15 +487,28 @@ OpenDockTrading solo tienen «Real» en **5 / 3**.
      - lo que nombra TaskCode sin clasificar → **un ADR por grupo**, con dos
        grupos fijos: código (`.py`, `.rs`, `.bat`, `.sh`, `.js`, `.ts`,
        `.ps1`) y documentación (el resto).
-   - Tras migrar, `tc2 import` del fichero propuesto (F3-2) y escritura del
-     registro (`origen: "actualizar"`, `informe_hash`, tareas migradas del
-     hito y nuevas). **`actualizar` no crea el registro en su transacción**:
-     rompería el criterio CR1 de F1-12. Hasta que existe, el estado es
-     `ausente` y la puerta niega (punto 7): un proyecto migrado no empieza
-     código sin declarar su hito.
-   - Ejemplo medido, OpenDockTrading en `8141a08` con la 2.0 de hoy: **9
-     elementos**, 3 bloqueantes (sus tres lectores de tareas, antes de F9-7),
-     4 tareas de `verificar.bat` y 2 ADR (PoC A).
+   - Los bloqueantes se hacen **antes** de migrar, como tareas de la 0.16:
+     mientras quede uno, `actualizar` se niega (código 5 de F1-12), así que no
+     entran en el registro de la 2.0.
+   - **`actualizar` no crea el registro en su transacción**: rompería el
+     criterio CR1 de F1-12. Tras migrar el estado es `ausente` y la puerta
+     niega (punto 7): un proyecto migrado no empieza código sin declarar su
+     hito.
+   - **Quién crea el registro tras migrar: la persona**, con uno de dos
+     comandos, sin depender de que exista antes:
+     - `tc2 hito abrir --desde <propuesta>`: importa los elementos de la
+       propuesta (la importación de F3-2) con `hito: true` y escribe el
+       registro (`origen: "actualizar"`, `informe_hash`, `tareas`) en el
+       mismo commit firmado;
+     - `tc2 hito no-necesario --desde <informe> --motivo "..."`: si el
+       registro no existe, lo crea con `tareas: []` y la firma; si existe, la
+       añade.
+     Ninguno de los dos lo puede lanzar el agente (F1-6). El flujo entero,
+     de `ausente` a `cerrado` o a `no-necesario`, está medido (PoC R2).
+   - Ejemplo medido, OpenDockTrading en `8141a08`: con la 2.0 de hoy, **9
+     elementos**: 3 bloqueantes (sus tres lectores de tareas, antes de F9-7),
+     4 tareas de `verificar.bat` y 2 ADR; con la 2.0.0 prevista, **5**: 3
+     sustituciones y 2 ADR (PoC A). El registro de la PoC R2 abre esos 5.
 
 4. **Proporcionalidad** (PR1).
    - **Tamaño = número de huecos** (punto 3) o de áreas que aplican (punto
@@ -438,54 +525,82 @@ OpenDockTrading solo tienen «Real» en **5 / 3**.
    - El presupuesto informa; **no bloquea**. Recalibración con más proyectos:
      F1-17 (estimación) y F8-7.
 
-5. **Registro y estado calculado** (D1a, D3a). `hito-decisiones.yml` en la
-   raíz de `taskcodetwo/tareas`, escrito con la escritura atómica de F1-2:
+5. **Registro, firma y estado calculado** (D1a, D3a, FI1).
+   `hito-decisiones.yml` en la raíz de `taskcodetwo/tareas`, escrito con la
+   escritura atómica de F1-2:
 
    ```yaml
    formato: 2
-   origen: "init"                 # o "actualizar"
+   origen: "actualizar"           # o "init"
    informe_hash: "<sha256 del informe>"
    tareas: ["TASK-097", "TASK-098"]
+   descartadas: {}                # {"TASK-NNN": "motivo"}
    no_necesario: null             # o {por: "<identidad de persona>", fecha: "AAAA-MM-DD", motivo: "..."}
    ```
 
-   `hito_estado(rama)` **se calcula, no se guarda**:
+   - **Firma**: la lista `tareas` la fija una persona con su identidad, igual
+     que los descartes y «no necesario». Firmar es escribir el registro en un
+     commit de la rama de estado con identidad de persona (el mismo mecanismo
+     que `approve`, F1-6 y F2-13); lo hacen `tc2 init`, `tc2 hito abrir`,
+     `tc2 hito descartar` y `tc2 hito no-necesario`, y el agente no puede
+     lanzarlos. La **versión firmada** es la del último commit de persona que
+     escribe el registro.
+   - Una tarea es **del hito** si está en la lista firmada y su `tarea.md`
+     lleva `hito: true` (lo escribe `tc2 hito abrir`). Un ID de una tarea de
+     código en la lista no cuenta, aunque esté firmado.
+
+   `hito_estado(rama)` **se calcula, no se guarda**, y devuelve el estado, el
+   motivo y las tareas del hito de la versión firmada:
    - `ausente`: no hay registro;
-   - `no-necesario`: `no_necesario` con `por`, `fecha` y `motivo` no vacíos y
-     `informe_hash` no vacío;
-   - `cerrado`: lista no vacía y todas sus tareas `terminada` o `descartada`;
-   - `abierto`: todo lo demás, **incluido cualquier registro roto** (YAML
-     ilegible, sin `formato: 2` u `origen` válido, lista inválida, firma
-     incompleta, lista vacía sin firma, tarea listada que no existe). Falla
-     cerrado.
-   No depende de si «hito» es `sprint` (pregunta abierta de F1-9).
+   - `no-necesario`: la versión firmada lleva `no_necesario` con `por`,
+     `fecha` y `motivo`, `por` es la identidad que firmó el commit, y el
+     registro actual es esa versión;
+   - `cerrado`: el registro actual es la versión firmada, su lista no está
+     vacía y cada tarea es del hito y está `terminada`, o `descartada` con su
+     motivo en `descartadas`;
+   - `abierto`: todo lo demás, **incluidos** un registro distinto de la
+     versión firmada, un registro sin firma de persona, una tarea que no es
+     del hito, una descartada sin motivo firmado y cualquier registro roto
+     (YAML ilegible, sin `formato: 2`, `origen` o `informe_hash`, lista
+     inválida, lista vacía sin firma). Falla cerrado.
+   No depende de si «hito» es `sprint` (pregunta abierta de F1-9). Las
+   transiciones de estado de las tareas las sigue haciendo el agente (F1-2):
+   no tocan el registro.
 
 6. **Comandos deterministas** (F1-11, decisión 10):
-   - `tc2 hito estado [--json]`: imprime estado y motivo; sale con 0 si
-     `cerrado` o `no-necesario` y con el código de precondición si no.
+   - `tc2 hito estado [--json] [--exigir]`: imprime estado, motivo y tareas
+     del hito; con `--exigir` sale con `rechazada` (3) si no está `cerrado`
+     ni `no-necesario`.
    - `tc2 hito proponer --desde <informe>`: escribe la propuesta fuera del
      repo; no toca el repo.
-   - `tc2 hito no-necesario --motivo "<texto>"`: exige identidad de persona,
-     con el mismo mecanismo que `approve` (F2-13); **el agente no puede
-     firmarlo** (F1-6).
+   - `tc2 hito abrir (--desde <propuesta> | --tareas <IDs>)`: persona; crea
+     las tareas de la propuesta (o marca las que ya existen) con `hito: true`
+     y escribe el registro firmado. Si el registro existe, la nueva lista
+     sustituye a la anterior con una firma nueva.
+   - `tc2 hito descartar <ID> --motivo "<texto>"`: persona; descarta una
+     tarea del hito y apunta el motivo en `descartadas`, firmado.
+   - `tc2 hito no-necesario [--desde <informe>] --motivo "<texto>"`: persona;
+     crea el registro si no existe (con `tareas: []`) o añade la firma.
    - El único hook previsto es un atajo PreToolUse opcional que llama a
-     `tc2 hito estado --json`; quitarlo no cambia ninguna garantía.
+     `tc2 hito estado --json --exigir`; quitarlo no cambia ninguna garantía.
 
 7. **Puerta** (D2a), una sola función pura `puerta(tarea, carril, estado,
-   tareas_del_hito)`:
-   - pasa si el hito está `cerrado` o `no-necesario`, si la tarea está en la
-     lista del hito o si el carril es `errata`; si no, **código de
-     precondición** (4, el de F1-12; F1-4 deja los números a F2) con el
-     comando que lo arregla;
+   tareas_del_hito)`, con las tareas del hito de la versión firmada:
+   - pasa si el hito está `cerrado` o `no-necesario`, si la tarea es del hito
+     o si el carril es `errata`; si no, sale con el código `rechazada` (3) del
+     contrato de F1-4 (`start rechazada id=TASK-NNN motivo=hito-abierto`, el
+     motivo nombra el estado). No usa el 4, que en F1-2 y F1-4 es `sinred`;
+     los números los fija F2-3;
    - **parada local en `tc2 start`** con el carril **declarado**, antes de la
      toma atómica (F2-4, F3-6): no toma la tarea;
    - **imposición en el check `estado`** con el carril **efectivo por
-     ficheros** (F1-10): declarar `errata` en una tarea que toca código no
-     salta la puerta, porque el check ve los ficheros.
+     ficheros** (F1-10) y comparando el registro con su versión firmada:
+     declarar `errata` en una tarea que toca código, o meter su ID en la
+     lista sin firma, no salta la puerta.
 
 8. **TaskCodeTwo como precedente.** Su Fase 1 es su hito de decisiones
-   (origen `init`, las 18 ADR). Hoy está **abierto** (12 de 18 terminadas,
-   PoC R), lo que casa con la regla de este repo de no escribir código de la
+   (origen `init`, las 18 ADR, abierto en el relevo con `tc2 hito abrir
+   --tareas`). Hoy está **abierto** (12 de 18 terminadas, PoC R1), lo que casa con la regla de este repo de no escribir código de la
    2.0 hasta cerrar la Fase 1. En el relevo (F4-5) el registro lista esas
    tareas terminadas y sale `cerrado`.
 
@@ -520,13 +635,28 @@ OpenDockTrading solo tienen «Real» en **5 / 3**.
    con el hito abierto; se resuelve con `init` fuera de la puerta (punto 2).
 10. **Contraste con OpenDockTrading**: no estaba en el plan; se añade porque
     da un segundo proyecto con ADR medidos (5 tareas, «Real» en 3).
+11. **Código de la negativa** (revisión 1, IMP-2): el plan decía «el código
+    de precondición de F1-4» y suponía que F1-12 usa 4. F1-12 (punto 1) sí
+    lista «**4** precondición» para `actualizar`, pero en F1-2 y F1-4 el 4 es
+    `sinred`, y `start` ya devuelve `sinred` (F1-2). La puerta usa el código
+    `rechazada` (3) del contrato de F1-4, con `motivo=hito-<estado>`. El choque
+    del 4 de F1-12 con F1-4 queda para F2-3, que fija los números.
+12. **Firma de la lista** (revisión 1, IMP-1): el plan solo pedía firma de
+    persona para «no necesario». Con eso, cambiar la lista o descartar las
+    tareas daba `cerrado` sin firma. La lista `tareas` y los descartes pasan
+    a ser parte del registro firmado, y una tarea solo cuenta como del hito si
+    su `tarea.md` lleva `hito: true` (campo nuevo para el formato 2 de F1-9) y
+    está en la lista firmada.
+13. **Comandos nuevos** (revisión 1, IMP-3): `tc2 hito abrir` y
+    `tc2 hito descartar`, y `tc2 hito no-necesario` que crea el registro si no
+    existe; el plan solo nombraba `estado`, `proponer` y `no-necesario`.
 
 ## Requisitos y evidencia
 
 | Requisito | Decisión | Evidencia |
 |---|---|---|
 | RR-13 actualizar proyectos existentes | hito en `actualizar` con el informe ampliado (punto 3) | PoC A: 91 / 973 recalculado, 9 elementos, 0 ficheros cambiados en la copia |
-| RR-12 equipo | registro en la rama de estado compartida con escritura atómica; firma con identidad | PoC R; escritura atómica de F1-2 no ejercitada aquí |
+| RR-12 equipo | registro en la rama de estado compartida con escritura atómica; lista, descartes y «no necesario» firmados con identidad de persona | PoC R: 5/5 ediciones sin firma y 6/6 registros rotos salen abierto; escritura atómica de F1-2 no ejercitada aquí |
 | RR-14 se construye con la 0.16 | TaskCodeTwo como precedente (punto 8) | PoC R: abierto, 12 de 18 |
 | F1-11 decisión 10, comandos y no hooks | `tc2 hito estado / proponer / no-necesario` (punto 6) | PoC R y P: mismas salidas en dos llamadas |
 | F1-11 decisión 4, check `estado` | imposición con carril efectivo (punto 7) | PoC P: 12 / 1 en TaskCodeTwo y 1 / 43 en OpenDockTrading con el hito abierto |
@@ -534,15 +664,18 @@ OpenDockTrading solo tienen «Real» en **5 / 3**.
 | F1-12 CR1, criterio de salida | el registro se escribe tras migrar, no en la transacción | por diseño; la PoC no ejecuta `actualizar` |
 | F1-12 decisión 8, huecos bloqueantes | bloqueantes como tareas de la 0.16 antes de migrar | PoC A: 3 bloqueantes con la 2.0 de hoy, 0 con la 2.0.0 prevista |
 | F1-10, errata solo con `docs/` | `errata` pasa la puerta | PoC P: los 12 PR de ADR de TaskCodeTwo y TASK-067 de OpenDockTrading |
-| F1-6, ninguna IA firma | `no-necesario` solo persona | PoC R comprueba la firma completa; la identidad, no (F2-13) |
+| F1-6, ninguna IA firma | lista, descartes y `no-necesario` solo persona (punto 5) | PoC R: el `no-necesario` del agente y las ediciones del agente salen abierto; la identidad real, no (F2-13) |
+| F1-4, contrato de salida | negativa `rechazada` (3) con `motivo=hito-<estado>` | PoC P: códigos [0, 3] |
+| F1-12 CR1 y huecos, flujo de `actualizar` | la persona crea el registro tras migrar (punto 3) | PoC R2: ausente → abierto (5) → cerrado; ausente → no-necesario |
 | Proporcionalidad con cifras | punto 4 | PoC H: 45,5–50 min y 13,9–19,1 M tokens por ADR |
 
 ## Consecuencias
 
 - **F9-1** añade a `--simular` el bloque «uso medido» y el `informe_hash`, e
   implementa `proponer_hito` y `tc2 hito proponer`.
-- **F3-2** acepta en `import` el fichero de la propuesta y añade las tareas a
-  la lista del hito.
+- **F3-2** acepta en `import` el fichero de la propuesta; `tc2 hito abrir`
+  la usa y escribe el registro firmado.
+- **F2-5** (formato 2) añade el campo `hito` a `tarea.md`.
 - **F3-6** (`start`) llama a la puerta antes de la toma atómica; el check
   `estado` (F1-11) la llama con el carril efectivo.
 - **F2-3** crea `hito-decisiones.yml` al crear la rama de estado en `init`;
@@ -572,8 +705,10 @@ Este ADR se planificó con un solo rol (arquitectura). Lo que queda:
   firma de «no necesario» caduca cuando cambia el informe (hoy no: guarda el
   `informe_hash` como evidencia); si «hito» es `sprint` (F1-9).
 - **Tope por hito**: no se fija; el presupuesto informa y no bloquea.
-- **Riesgos**: registro borrado por un PR a la rama de estado (lo contiene la
-  protección de F1-5, no verificado); clon sin la rama de estado al hacer
+- **Riesgos**: registro borrado o reescrito en la rama de estado (lo
+  contienen la versión firmada y la protección de F1-5; la protección, no
+  verificada); una persona que firma una lista equivocada (la regla solo
+  impide que el agente la cambie); clon sin la rama de estado al hacer
   `start` (la puerta lo vería `ausente` y negaría); `no-necesario` sin
   identidad disponible mientras F2-13 no exista.
 - **Rendimiento**: `hito_estado` lee un `tarea.md` por tarea del hito, no
@@ -585,8 +720,12 @@ Este ADR se planificó con un solo rol (arquitectura). Lo que queda:
   la parada en `start` son funciones de la PoC, no código del producto.
 - **GitHub**: ni el check `estado` en rojo ni la protección del registro se
   han visto en un PR real.
-- **La identidad de persona** de la firma: la PoC solo comprueba que `por`,
-  `fecha`, `motivo` e `informe_hash` no están vacíos (F2-13 no existe).
+- **La identidad de persona** de la firma: en la PoC es el correo del autor
+  del commit, que en local cualquiera puede poner; que sea de verdad una
+  persona lo tienen que garantizar F1-6 y F2-13 (no existe) en GitHub.
+- **La parada local de `start` con carril declarado**: solo se midió sin
+  carril (ninguna tarea real lo declara: 0); el caso `errata` declarado se
+  apoya en la misma función y en F1-10, sin medida propia.
 - **Minutos de diseño**: la 0.16 no guarda cuándo empieza el diseño; los
   minutos de la regla excluyen esa fase.
 - **Verificación por tarea**: «no medido» en OpenDockTrading (la 0.16 no la
diff --git a/docs/adr/README.md b/docs/adr/README.md
index 7a878f1..a0ee67a 100644
--- a/docs/adr/README.md
+++ b/docs/adr/README.md
@@ -20,7 +20,7 @@ directo a la tabla de la Fase 1 y a su tarea.
 | [F1-10](F1-10-carriles-y-modo-automatico.md) | Carriles y modo automático: cuatro carriles ordenados más release y hotfix por tipo, tabla única de política por carril con su función pura (`accion` en `siguiente --json`), configuración que solo endurece (`politica_carriles`, `rutas_carril`), carril mínimo por ficheros con los dos nombres de cada renombrado, y paradas fijas (`approve`, `merge-pr`, tag, tope de rondas, subida de carril) | TASK-011 | aceptada |
 | [F1-11](F1-11-extensiones-declaradas.md) | Extensiones declaradas: inventario comprobado de las 14 prácticas que los pilotos hacen a mano, clasificadas en parche de TaskCode (se retira al migrar) o propio del proyecto, con el desglose de líneas; tres claves (`verificacion`, `derivados`, `confianza_incluye`) con esquema que falla cerrado y rutas normalizadas; derivados que ejecuta `taskctl` en la transición solo en la rama de estado, comparados por contenido; check obligatorio `estado`; `taskctl tareas --json` como lectura estable; sin estados propios; revisoras como skill del proyecto; «comandos sí, hooks solo como atajo opcional»; y la ventaja medida frente a la 0.16, con lo que no gana | TASK-012 | aceptada |
 | [F1-12](F1-12-actualizacion-entre-versiones.md) | Actualización entre versiones: el plan de migración como dato (`tc2 actualizar --simular` sin cambiar un byte, mismo hash, vacío en la segunda pasada), qué migra (aplanado, formato 2, archivadas a `descartada`, config a los bloques de F1-11), historia con `git subtree split`, transacción local con criterio de salida estructural y vuelta atrás automática, retirada de parches con capacidades y huecos bloqueantes, convivencia con la 0.16 (su lector de config aborta con `---`: ninguna orden de escritura entra en un proyecto migrado), nombres definitivos `tc2` y `taskcodetwo`, y orden de pilotos OpenDockTrading → TaskCodeTwo → ToolsaWeb | TASK-013 | aceptada |
-| [F1-18](F1-18-hito-de-decisiones.md) | Hito de decisiones (ADR) en `init` y en `actualizar`: estudio de adopción que lo dimensiona (bloque «uso medido» del informe de F1-12 y `proponer_hito` con un elemento por hueco, bloqueantes antes de migrar), proporcionalidad con mediana y p75 de la Fase 1 (45,5–50 min y 13,9–19,1 M tokens por ADR), registro `hito-decisiones.yml` en `taskcodetwo/tareas` con estado calculado que falla cerrado, y puerta determinista en `tc2 start` y en el check `estado` | TASK-094 | aceptada |
+| [F1-18](F1-18-hito-de-decisiones.md) | Hito de decisiones (ADR) en `init` y en `actualizar`: estudio de adopción que lo dimensiona (bloque «uso medido» del informe de F1-12 y `proponer_hito` con un elemento por hueco, bloqueantes antes de migrar), proporcionalidad con mediana y p75 de la Fase 1 (45,5–50 min y 13,9–19,1 M tokens por ADR), registro `hito-decisiones.yml` en `taskcodetwo/tareas` cuya lista, descartes y «no necesario» firma una persona, con estado calculado que falla cerrado, y puerta determinista en `tc2 start` y en el check `estado` (código `rechazada`) | TASK-094 | aceptada |
 
 ## Plantilla
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 70e685d5f42f815aa972ac9890ab5c8771692018..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-094/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-094/planificacion/plan-final.md                 |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-094/poc/poc-f1-18.log                           |  66 +++++++---
 tareas/{02-en-curso => 03-en-revision}/TASK-094/poc/poc-f1-18.py                            | 429 ++++++++++++++++++++++++++++++++++++++++++++++----------------
 tareas/03-en-revision/TASK-094/revision/informe-revision-1.md                               |  70 +++++++++++
 tareas/03-en-revision/TASK-094/revision/peticion-revision-1.md                              | 688 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-094/tarea.md                                    |   8 +-
 7 files changed, 1132 insertions(+), 129 deletions(-)
````

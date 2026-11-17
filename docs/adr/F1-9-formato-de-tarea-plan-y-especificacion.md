# ADR F1-9 — Formato de tarea, plan y especificación (compatible con la 0.16 y con Spec Kit)

- Estado: **aceptada** (2026-11-14)
- Tarea: TASK-010 · Plan: [F1-9 en PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) (tabla de la Fase 1, §3 y §5)
- Requisitos: la fila F1-9 del plan no cita ninguno; este ADR responde a RR-10, RR-13, RR-14, RR-17, RR-19, RF-01, RF-07, RF-22, RF-34, RF-51, RF-62 y RF-71 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md) y a H11 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
- Depende de: [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (carpeta fija `TASK-NNN/`, estado como campo), [ADR F1-2](F1-2-escrituras-atomicas-ids-y-toma.md) (ID y toma con el «quién»), [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md) (nombre de rama), [ADR F1-5](F1-5-puertas-en-github.md) (decisión 4: la aprobación es el commit firmado con `Plan-Blob` sobre el blob de `plan-final.md`) y [ADR F1-8](F1-8-seguridad-modelo-de-amenazas.md) (zona Z2 y activo `A10`: el texto de las tareas es dato no confiable)
- Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a y D5a del plan de TASK-010)
- Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
- Adenda (2026-11-17, TASK-029): clave opcional `lote` para el import en bloque; ver [la adenda al final](#adenda-f3-2-task-029--clave-opcional-lote)
- Adenda (2026-11-17, TASK-081): filas `liberar` y `reasignar`; `reasignar` resuelve los `asignado_a` heredados; ver [la adenda al final](#adenda-f3-10-task-081--filas-liberar-y-reasignar)

## Contexto

En la 0.16 una tarea es `tarea.md` (frontmatter de 22 claves en un orden
fijo, `TASK_FIELD_ORDER` de `src/core/task.ts`, y secciones `##`) y su plan
es `planificacion/plan-final.md`. El frontmatter lo lee y escribe un parser
propio que solo admite un subconjunto de YAML y que escapa `"` pero no `\`:
un título `C:\ruta` no es YAML válido para un parser estándar (H11). La 2.0
tiene que:

- conservar lo que se usa de verdad (plan §5: «`tarea.md` con frontmatter,
  `## Resultado`») y retirar lo que no (RF-34, segunda opinión con Codex:
  0 usos, marcado R);
- dejar de atar el estado a una carpeta (RR-10, F1-1) y admitir el descarte
  con motivo (RF-07), que hoy ToolsaWeb y OpenDockTrading hacen a mano con
  `05-archivadas` (RF-71);
- anotar por rol la elección de skills, agentes y modelo (RR-17) y dejar
  sitio a la estimación (RR-19);
- leer las tareas de la 0.16 para migrarlas (RR-13) sin romper este repo,
  que sigue en la 0.16 hasta el relevo (RR-14);
- convivir con [GitHub Spec Kit](https://github.com/github/spec-kit), que el
  plan (§3) cita como referencia y critica por sus «artefactos que se
  desincronizan del código».

Preguntas que responde: dónde vive la especificación; qué campos se
conservan, cuáles cambian, cuáles se retiran y cuáles se añaden; qué
secciones tiene una tarea; cuál es el contrato del texto; cómo son las
plantillas de `tarea.md` y `plan-final.md`; cómo se corresponde todo con
Spec Kit.

## Alternativas

### Dónde vive la especificación (D1)

| | Descripción | Veredicto |
|---|---|---|
| **D1a** | Un único modelo canónico, `tarea.md` en la carpeta fija de F1-1: la especificación es el Objetivo, los Criterios de aceptación y una sección `## Escenarios` opcional. Spec Kit es correspondencia de entrada y salida, no almacenamiento. | **Elegida** |
| D1b | `spec.md` aparte dentro de `TASK-NNN/`, junto a `tarea.md`. | Descartada: el Objetivo y los Criterios quedarían en dos ficheros para el mismo hecho, que es lo que el plan §3 critica de Spec Kit; además la aprobación firmada (F1-5) solo se ata al blob de `plan-final.md`, no a un tercer fichero. |
| D1c | La estructura de Spec Kit, `specs/NNN-nombre/{spec,plan,tasks}.md`. | Descartada: choca con F1-1 (carpeta fija `TASK-NNN/` en la rama de estado) y con los nombres de rama de F1-3 (Spec Kit usa `###-feature-name`). |

### El resto de decisiones

| | Elegida | Descartadas y por qué |
|---|---|---|
| **D2** Dónde van las plantillas | **D2a**: dentro de este ADR, en bloques de código; F2-5 las convierte en ficheros | D2b, ficheros en el repo ahora: no se escribe nada de la 2.0 hasta cerrar la Fase 1 (CLAUDE.md, punto 4) |
| **D3** `plan_aprobado` | **D3a**: se conserva como caché informativa que escribe el CLI y que ninguna puerta lee | D3b, retirarlo: se pierde la lectura rápida del tablero y del roadmap; D3c, que sea puerta: F1-5 (decisión 4) ya decidió que una marca en el frontmatter no basta, porque el agente escribe la rama de estado |
| **D4** Claves desconocidas en el frontmatter | **D4a**: se conservan en la ida y vuelta y `doctor` avisa | D4b, rechazarlas: rompe la costura con las extensiones declaradas (F1-11); D4c, borrarlas en silencio: pierde datos del proyecto |
| **D5** `revision_codex` | **D5a**: se retira en el formato 2; F1-12 lo elimina al migrar | Conservarlo: 0 tareas con valor distinto de `false` en los tres repos (tabla de campos) y RF-34 está marcado R |

## Pruebas de concepto

Un guion de solo lectura, ejecutado el 2026-11-14 en
`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-010\` (borrada al terminar)
con Git 2.56.0, gh 2.95.0, Python 3.12.10 y **PyYAML 6.0.3** como parser
YAML estándar (con `python -I`). Lee `tareas/` de cada repo en un commit
fijo con `git archive`, para que la medida se pueda repetir aunque los repos
sigan avanzando: ToolsaWeb `93c9618`, OpenDockTrading `fab8d7e` y
TaskCodeTwo `afc3791` (`develop` antes de empezar esta tarea). A GitHub solo
hace `GET`. Sale con 0 solo si todo sale como se espera y cada control
negativo falla por el error esperado. Salida observada, recortada:

### A — uso real de los campos y de las secciones

Criterio: una tarea cuenta si su valor es distinto del que pone la 0.16 al
crearla (`new` e `import`; `agente_revisor_por_defecto` es
`general-purpose` y ninguno de los tres repos lo cambia). `id`, `titulo` y
`creado` son siempre propios; `rama` cuenta si no es la derivada del tipo,
el ID y el título; `actualizado`, si no es igual a `creado`.

```text
campos de TASK_FIELD_ORDER: 22
ToolsaWeb: 404 tarea.md ; frontmatter no YAML para PyYAML: 0 ; claves fuera de la lista: ninguna ; orden distinto: 0 ; creado leido como fecha (sin comillas): 404 ; titulos con \ o ": 0
   estado distinto de su carpeta: 9 ['TASK-006:05-archivadas/planificada', ... 'TASK-097:05-archivadas/en-diseno', 'TASK-171:05-archivadas/planificada']
   ## Resultado despues de ## Transiciones: 28
OpenDockTrading: 96 tarea.md ; ... no YAML: 0 ; claves fuera de la lista: ninguna ; ... como fecha: 96 ; titulos con \ o ": 0
   estado distinto de su carpeta: 2 ['TASK-015:05-archivadas/planificada', 'TASK-068:05-archivadas/planificada']
   ## Resultado despues de ## Transiciones: 14
TaskCodeTwo: 93 tarea.md ; ... no YAML: 0 ; claves fuera de la lista: ninguna ; ... como fecha: 93 ; titulos con \ o ": 0
   estado distinto de su carpeta: 0 []
   ## Resultado despues de ## Transiciones: 8

campo                          TW      ODT      TC2
modelo_sugerido                 8        0        0
agente_revisor                  0        0        0
skills_recomendados            21        0        0
ultimo_commit_revisado          0        0        0
revision_codex                  0        0        0
tokens_implementacion          54       41        9
...                     (las 22 filas, en la tabla de la decisión)
secciones ## (tareas que la tienen):
Resultado                                     159       43        9
Coste                                          92       44       10
Autorrevision                                  33       43        9
Duracion                                       23       33        9
Descartada*                                     6        2        0
(secciones con un solo uso en total: 26)
```

Frente a REQUISITOS §1 (2026-11-13): `## Resultado` / `## Coste` /
`## Autorrevision` dan 159 / 92 / 33 en ToolsaWeb (allí 159 / 91 / 33) y
43 / 44 / 43 en OpenDockTrading (allí 40 / 41 / 40): las diferencias son las
tareas cerradas desde entonces. La tercera columna de REQUISITOS es el repo
TaskCode, no TaskCodeTwo, y no se compara.

### C — la plantilla de `tarea.md` con un parser estándar

```text
plantilla: 21 claves de primer nivel: formato, id, titulo, tipo, sprint, ... dependencias
  titulo 'C:\\ruta\\'                       -> 'C:\\ruta\\'                         IGUAL
  titulo 'dice "hola"'                      -> 'dice "hola"'                        IGUAL
  titulo 'C:\\ruta\\ "con comillas" \\'     -> 'C:\\ruta\\ "con comillas" \\'       IGUAL
  titulo 'dos puntos: y # almohadilla'      -> 'dos puntos: y # almohadilla'        IGUAL
  control negativo, escape de la 0.16, 'C:\\ruta\\' -> ERROR ParserError (esperado: no da el titulo)
  control negativo, escape de la 0.16, 'C:\\ruta'   -> carga, pero da 'C:\ruta' (esperado: no da el titulo)
  lineas con texto sin comillas: ninguna
  observacion: PyYAML safe_load acepta claves duplicadas sin error y se queda con la ultima: {'a': 2}
```

(`repr` de Python: `'C:\\ruta'` es el texto `C:\ruta`; con el escape de la
0.16, `\r` se convierte en un retorno de carro, que es H11.)

### D — la plantilla del plan frente a la 0.16 y a un plan real

```text
de la 0.16 que faltan en la plantilla: ninguno ; de la plantilla que la 0.16 no tiene (salvo opcionales ['Troceo']): ninguno
de la plantilla (2 roles) que faltan en TASK-009: ninguno
de TASK-009 que la plantilla no nombra (texto libre del unificador): ['Lo que no cubrieron los roles', 'Suposiciones no verificadas']
linea "Duracion estimada": en la plantilla True, en TASK-009 True
```

### E — cada veredicto cita su cifra

La PoC lee la tabla de campos de este ADR entre los marcadores
`campos-tarea` y exige: una fila por cada uno de los 22 campos con la cifra
medida en A; las rutas del formato 2 presentes en la plantilla; los
retirados ausentes; ninguna clave de la plantilla sin fila. Exige también
que la frase «Recuento» del punto 3 coincida con los veredictos de la
tabla, que cada fila `## …` de la tabla de secciones (marcadores
`secciones-tarea`) cite lo medido en A y que «PoC A: 0 de 593» sea lo
medido.

```text
filas leidas: 26 ; campos de la 0.16 con fila: 22 de 22
veredictos: {'se conserva': 13, 'cambia de sentido': 7, 'se retira': 2, 'se añade': 4}
secciones con fila: 9 ; recuento del ADR: ('13', '7', '2', '4') ; PoC A N de M: ('0', '593')
  cifra-cambiada: falla como se espera (ERROR: sprint: cifra del ADR ('9999', '68', '93') distinta de la medida (355, 68, 93))
  fila-borrada: falla como se espera (ERROR: revision_codex: 0 filas (se espera 1))
  ruta-inexistente: falla como se espera (ERROR: titulo: la ruta titulo_inventado no esta en la plantilla)
  sin-cifra: falla como se espera (ERROR: tipo: sin cifra "N / N / N" (medido (6, 0, 0)))
  seccion-resultado: falla como se espera (ERROR: seccion Resultado: cifra del ADR ('999', '43', '9') distinta de la medida (159, 43, 9))
  seccion-descartada: falla como se espera (ERROR: seccion Descartada*: cifra del ADR ('7', '2', '0') distinta de la medida (6, 2, 0))
  recuento: falla como se espera (ERROR: Recuento del ADR ('14', '7', '2', '4') distinto del de la tabla (13, 7, 2, 4))
```

### F — Spec Kit

`GET repos/github/spec-kit/contents/templates` en el commit `2ee2b1e` de
`main` y descarga de `spec-template.md` (4556 bytes), `plan-template.md`
(3770) y `tasks-template.md` (9182) a una carpeta propia vacía. Las 13
secciones citadas en la correspondencia existen; una inventada
(«User Stories Inventadas») falla. Lo que no se ha comprobado: el CLI
`specify`, los comandos de Spec Kit y una ida y vuelta real.

### G — el propio ADR

Los 12 IDs de requisitos, el de la auditoría y las 21 tareas del plan que
cita existen; tiene las secciones de la plantilla, ningún enlace a `tareas/`
ni a otros repos, y su fila en el índice.

### Divergencia con el plan

1. **El plan clasificaba 20 de los 22 campos**: no decía nada de
   `agente_revisor` ni de `ultimo_commit_revisado`. Con la medida: el
   primero (0 / 0 / 0, pero es lo que `review` lanza) pasa a
   `roles.revision.agente` por RR-17; el segundo (0 / 0 / 0; la 0.16 solo lo
   escribe a `null`) se retira.
2. **`estado` aparecía a la vez en «se conservan» y en «cambian de
   sentido»**; la tabla lo pone una vez, en «cambia de sentido».
3. **Secciones que el plan no nombraba**: `## Duracion` (23 / 33 / 9, la
   escribe el CLI) queda como opcional; `## Descartada` (6 / 2 / 0) se
   sustituye por `estado: descartada` y `motivo`.
4. **El plan medía el árbol de trabajo**; la PoC lee commits fijos para que
   la revisión pueda repetir las cifras.

## Cómo se ve en GitHub

Lo que sigue es conocimiento general de GitHub, **no verificado** en esta
tarea: los repos de las tareas reales son privados y la única forma de
pedirle a GitHub que pinte un Markdown sin publicarlo es `POST /markdown`,
y esta tarea solo admite `GET`.

- GitHub pinta el frontmatter YAML de un `.md` como una tabla encima del
  documento; con un YAML inválido muestra un aviso de error. Cómo pinta el
  mapa anidado `roles` está sin verificar.
- La tarea se lee en `.../blob/taskcode/tareas/TASK-NNN/tarea.md` (F1-1).

Pasa como criterio a F2-5 (verlo en el repo de pruebas de F2-6).

## Decisión

1. **Modelo canónico: `tarea.md`** en la carpeta fija `TASK-NNN/` de F1-1.
   La especificación vive en `tarea.md` (Objetivo, Criterios de aceptación y
   `## Escenarios` opcional). Spec Kit es una correspondencia de entrada y
   salida (punto 6): ningún hecho vive en dos ficheros.
2. **`plan-final.md` conserva nombre y ruta** (`TASK-NNN/planificacion/plan-final.md`):
   el trailer `Plan-Blob` del `approve` firmado (F1-5, F1-6) se ata a su
   blob. Su plantilla conserva las secciones que la 0.16 genera y consume
   (punto 5).
3. **Campos del frontmatter.** `formato: 2` marca el formato nuevo; una
   tarea **sin** `formato` es una tarea de la 0.16 (formato 1), que la 2.0
   lee para migrarla (F1-12) pero no vuelve a escribir. La tabla dice, de
   los 22 campos de `TASK_FIELD_ORDER` (plugin 0.16.1), cuántas tareas de
   cada repo tienen un valor distinto del valor por defecto de la 0.16
   (PoC A; commits fijos de la PoC), el veredicto y dónde queda en el
   formato 2. Ninguno de los tres repos añade claves propias (PoC A).

   <!-- campos-tarea:inicio -->
   | Campo | Por defecto en la 0.16 (criterio) | Distintos del defecto TW / ODT / TC2 | Veredicto | En el formato 2 | Motivo |
   |---|---|---|---|---|---|
   | `id` | — (siempre propio) | 404 / 96 / 93 | **se conserva** | `id` | lo reserva F1-2; `TASK-NNN`, entre comillas |
   | `titulo` | — (siempre propio) | 404 / 96 / 93 | **se conserva** | `titulo` | entre comillas dobles con `\` y `"` escapados (H11) |
   | `tipo` | `feature` | 6 / 0 / 0 | **se conserva** | `tipo` | poco uso, pero `hotfix` y `release` mandan en el flujo de ramas (F1-3) |
   | `sprint` | `0` | 355 / 68 / 93 | **se conserva** | `sprint` | las altas en bloque van por hitos; si «hito» es `sprint`, pregunta abierta |
   | `etiquetas` | `[]` | 366 / 96 / 0 | **se conserva** | `etiquetas` | entrada de la selección determinista (RR-17, F1-15) |
   | `complejidad` | `null` | 404 / 0 / 0 | **se conserva** | `complejidad` | ToolsaWeb la declara en todas; entrada del carril (F1-10) y de la estimación (F1-17) |
   | `modelo_sugerido` | `sonnet` | 8 / 0 / 0 | **cambia de sentido** | `roles.diseno.modelo`, `roles.implementacion.modelo`, `roles.revision.modelo` | un modelo por rol (RR-17); el de la 0.16 pasa a `roles.implementacion.modelo`; el contenido lo decide F1-15 |
   | `estado` | `planificada` | 162 / 44 / 10 | **cambia de sentido** | `estado` | deja de corresponder a una carpeta (F1-1) y gana `descartada` (RF-07); hoy 9 / 2 / 0 tareas de `05-archivadas` dicen `planificada` o `en-diseno` |
   | `plan_aprobado` | `false` | 162 / 44 / 10 | **cambia de sentido** | `plan_aprobado` | caché informativa (D3a); la fuente es el commit firmado con `Plan-Blob` (F1-5, decisión 4); `carril` no la lee |
   | `rama` | `<tipo>/<id>-<slug del título>` | 68 / 3 / 0 | **se conserva** | `rama` | la nombra F1-3; la tarea la registra |
   | `asignado_a` | `null` | 162 / 44 / 10 | **cambia de sentido** | `asignado_a` | pasa a ser el usuario de la plataforma (F2-13); es el «asignado» de la toma de F1-2 |
   | `agente_revisor` | `general-purpose` | 0 / 0 / 0 | **cambia de sentido** | `roles.revision.agente` | nadie lo cambia, pero RR-17 pide anotar por rol el agente elegido; contenido en F1-15 |
   | `skills_recomendados` | `[]` | 21 / 0 / 0 | **cambia de sentido** | `roles.diseno.skills`, `roles.implementacion.skills`, `roles.revision.skills` | por rol (RR-17); contenido en F1-15 |
   | `regla_seleccion_skill` | `null` | 21 / 0 / 0 | **cambia de sentido** | `roles.diseno.regla`, `roles.implementacion.regla`, `roles.revision.regla` | por rol (RR-17); contenido en F1-15 |
   | `ultimo_commit_revisado` | `null` | 0 / 0 / 0 | **se retira** | — | ninguna tarea lo tiene; la 0.16 solo lo pone a `null` (al crear y al trocear) y `review` no lo actualiza; en la 2.0 el commit revisado es el head del PR que lleva el check `revision-ia` (F1-5) |
   | `revision_codex` | `false` | 0 / 0 / 0 | **se retira** | — | RF-34 marcado R (D5a) |
   | `tokens_diseno` | `null` | 162 / 44 / 9 | **se conserva** | `tokens_diseno` | coste por fase (RF-62) |
   | `tokens_implementacion` | `null` | 54 / 41 / 9 | **se conserva** | `tokens_implementacion` | coste por fase (RF-62) |
   | `tokens_revision` | `null` | 160 / 43 / 9 | **se conserva** | `tokens_revision` | coste por fase (RF-62) |
   | `creado` | — (siempre propio) | 404 / 96 / 93 | **se conserva** | `creado` | entre comillas: sin ellas PyYAML la lee como fecha y no como texto (PoC A) |
   | `actualizado` | igual a `creado` | 151 / 36 / 0 | **se conserva** | `actualizado` | entre comillas, como `creado` |
   | `dependencias` | `[]` | 364 / 89 / 92 | **se conserva** | `dependencias` | orden del roadmap e import en bloque |
   | `formato` | — | nuevo | **se añade** | `formato` | `2`; ausente = formato 1 de la 0.16 (migración en F1-12) |
   | `carril` | — | nuevo | **se añade** | `carril` | `null` hasta el plan; los valores los decide F1-10 |
   | `motivo` | — | nuevo | **se añade** | `motivo` | obligatorio con `estado: descartada` (RF-07); hoy es una sección `## Descartada` escrita a mano en 6 / 2 / 0 tareas |
   | `estimacion` | — | nuevo | **se añade** | `estimacion` | hueco para la estimación por fase (RR-19); su forma la decide F1-17 |
   <!-- campos-tarea:fin -->

   Valores de `estado`: `planificada`, `en-diseno`, `en-curso`,
   `en-revision`, `terminada` y `descartada`. Recuento: 13 se conservan,
   7 cambian de sentido, 2 se retiran y 4 se añaden.

4. **Secciones de `tarea.md`.** El parser las reconoce **por su título, no
   por su posición**: en 28 / 14 / 8 tareas `## Resultado` va después de
   `## Transiciones` (PoC A). Los títulos se mantienen sin tildes, como en
   la 0.16, para que las tareas migradas no cambien. Tareas que tienen cada
   sección hoy (PoC A, 404 / 96 / 93 tareas):

   <!-- secciones-tarea:inicio -->
   | Sección | TW / ODT / TC2 | Formato 2 |
   |---|---|---|
   | `## Objetivo` | 404 / 96 / 93 | obligatoria |
   | `## Criterios de aceptacion` | 404 / 96 / 93 | obligatoria; casillas comprobables (RF-01) |
   | `## Escenarios` | 0 / 0 / 0 | **nueva**, opcional: Dado / cuando / entonces (correspondencia con Spec Kit) |
   | `## Resultado` | 159 / 43 / 9 | obligatoria al cerrar (RF-51) |
   | `## Transiciones` | 162 / 44 / 10 | la escribe el CLI en cada transición |
   | `## Coste` | 92 / 44 / 10 | opcional, la escribe el CLI (RF-62) |
   | `## Autorrevision` | 33 / 43 / 9 | opcional (RF-22) |
   | `## Duracion` | 23 / 33 / 9 | opcional, la escribe el CLI al cerrar (estimada y real, RR-19) |
   | `## Descartada…` | 6 / 2 / 0 | se sustituye por `estado: descartada` y `motivo` |
   | otras (`Rondas extra`, `Verificacion ligera`, notas de la persona…) | 26 títulos con un solo uso, y unos pocos repetidos | texto libre: se conserva tal cual, sin significado para el CLI |
   <!-- secciones-tarea:fin -->

5. **Contrato del texto.**
   - El frontmatter es **YAML válido para un parser estándar** (H11). Las
     cadenas van **siempre entre comillas dobles**, con `\` y `"`
     escapados; también las fechas, que sin comillas un parser estándar
     convierte en fecha (PoC A: las 404 / 96 / 93 tareas). `null`, `true`,
     `false`, enteros y listas `[]` van sin comillas.
   - **Todo es dato no confiable** (F1-8, zona Z2 y activo `A10`): el CLI
     valida con un esquema estricto los campos conocidos (tipo y valores
     permitidos) y rechaza claves duplicadas, que PyYAML acepta en silencio
     quedándose con la última (PoC C); las claves desconocidas se conservan
     en la ida y vuelta y `doctor` avisa (D4a, costura con F1-11). Ningún
     texto de la tarea se ejecuta ni se interpreta como orden.
   - El cuerpo es Markdown; el CLI solo interpreta las secciones de la tabla
     del punto 4 y conserva las demás byte a byte.

6. **Plantilla de `tarea.md`** (formato 2). F2-5 la convierte en el fichero
   que escribe `new`; los valores son los de una tarea recién creada.

   <!-- plantilla-tarea:inicio -->
   ```markdown
   ---
   formato: 2
   id: "TASK-NNN"
   titulo: "<titulo>"
   tipo: "feature"
   sprint: 0
   etiquetas: []
   complejidad: null
   carril: null
   estado: "planificada"
   motivo: null
   plan_aprobado: false
   rama: "feature/task-nnn-<slug>"
   asignado_a: null
   roles:
     diseno: {modelo: null, skills: [], regla: null}
     implementacion: {modelo: "sonnet", skills: [], regla: null}
     revision: {agente: "general-purpose", modelo: null, skills: [], regla: null}
   estimacion: null
   tokens_diseno: null
   tokens_implementacion: null
   tokens_revision: null
   creado: "2026-11-14"
   actualizado: "2026-11-14"
   dependencias: []
   ---
   ## Objetivo

   <que se quiere y por que>

   ## Criterios de aceptacion

   - [ ] <criterio comprobable>

   ## Escenarios

   <opcional: 1. **Dado** <estado>, **cuando** <accion>, **entonces** <resultado>>

   ## Resultado

   <obligatoria al cerrar: que se hizo, divergencias con el plan, hallazgos no corregidos y por que>

   ## Transiciones

   | fecha | fase | modo | decidido_por | version |
   |---|---|---|---|---|
   ```
   <!-- plantilla-tarea:fin -->

7. **Plantilla de `plan-final.md`.** Es el esqueleto que hoy genera la 0.16
   (`planTemplate` de `src/commands/plan.ts`) con los mismos títulos y en el
   mismo orden; la línea de duración se simplifica a `Duracion estimada: N min`
   (la 0.16 escribe una línea más larga) y la cabecera de origen se resume en
   una (la 0.16 tiene tres frases según el número de roles). Si el detector
   de «esqueleto sin rellenar», que compara por líneas, sigue reconociéndola
   es cosa de F2-5. Se añade la sección
   `## Troceo` opcional que ya consume `trocear`. Según el número de roles
   lleva «Desacuerdos entre roles» (dos o más) o «Lo que el rol no cubrio»
   (uno), nunca las dos. La línea `Duracion estimada: N min` la exige
   `approve` (RR-19); las decisiones de la persona van numeradas `Dn` con la
   opción elegida. Quien consolida puede añadir secciones (el plan de
   TASK-009 añade «Lo que no cubrieron los roles» y «Suposiciones no
   verificadas», PoC D): son texto libre y el CLI no las interpreta.

   <!-- plantilla-plan:inicio -->
   ```markdown
   # Plan — TASK-NNN: <titulo>

   (<de donde sale: roles lanzados y quien consolida>)

   ## Enfoque propuesto

   Duracion estimada: N min

   <senales de la estimacion; que se crea, que se toca y en que orden>

   ## Desacuerdos entre roles, y como se resuelven

   <solo con dos o mas roles>

   ## Lo que el rol no cubrio

   <solo con un rol>

   ## Riesgos aceptados y que los contiene

   ## Plan de pruebas

   Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

   ## Lo que necesita decision de una persona

   - **D1. <pregunta>** → **(a)** <opcion elegida>. Descartadas: (b) <...>

   ## Troceo

   <opcional: ### Parte a: <titulo>, con su propia Duracion estimada>
   ```
   <!-- plantilla-plan:fin -->

8. **Correspondencia con Spec Kit** (plantillas de `github/spec-kit` en el
   commit `2ee2b1e`, leídas el 2026-11-14). `tarea.md` y `plan-final.md`
   son la fuente; la salida a Spec Kit se genera bajo demanda y no se
   versiona como fuente; la entrada crea o completa una tarea. Las secciones
   de Spec Kit entre «» se han comprobado contra las plantillas (PoC F).

   <!-- correspondencia-speckit:inicio -->
   | TaskCode 2 | Spec Kit | Entrada (Spec Kit → tarea) | Salida (tarea → Spec Kit) |
   |---|---|---|---|
   | `titulo` | `spec.md` «Feature Specification» | el nombre tras los dos puntos | el título |
   | `## Objetivo` | `spec.md` «Requirements» | los `FR-NNN` como lista dentro del Objetivo | el Objetivo en prosa y su lista, si la hay |
   | `## Criterios de aceptacion` | `spec.md` «Success Criteria» | cada `SC-NNN`, una casilla | cada casilla, un `SC-NNN` |
   | `## Escenarios` | `spec.md` «User Scenarios & Testing» | Given / When / Then → Dado / cuando / entonces; la prioridad P1… como texto | al revés |
   | texto libre bajo `## Objetivo` | `spec.md` «Edge Cases», `spec.md` «Assumptions», `spec.md` «Key Entities» | subsecciones `###` del Objetivo | no se generan (pérdida declarada) |
   | `## Enfoque propuesto` (plan) | `plan.md` «Summary», `plan.md` «Technical Context» | texto del Enfoque | el Enfoque como Summary |
   | `## Riesgos aceptados y que los contiene` (plan) | `plan.md` «Complexity Tracking» | texto de Riesgos | Riesgos |
   | `## Plan de pruebas` (plan) | `tasks.md` «Tests for User Story 1» | pruebas por criterio | pruebas por criterio |
   | `## Troceo` (plan) y `dependencias` | `tasks.md` «Dependencies & Execution Order», `tasks.md` «Phase 1: Setup» | partes del troceo, sin crear tareas solas | partes y orden |
   | `## Lo que necesita decision de una persona` (plan) | marcas `[NEEDS CLARIFICATION]` del spec (no es una sección) | cada marca, una pregunta `Dn` | cada `Dn` sin responder, una marca |
   | `rama`, `estado`, `creado` | líneas en negrita del spec (`Feature Branch`, `Status`, `Created`), no secciones | se ignoran: la rama la nombra F1-3 | se escriben como texto |
   <!-- correspondencia-speckit:fin -->

   La ida y vuelta real con un proyecto de Spec Kit (su CLI `specify` y sus
   comandos) está **no verificada**: solo se han cotejado los títulos de las
   plantillas. Pasa como criterio a F2-5. La `constitution` de Spec Kit
   (principios del proyecto) no tiene equivalente en una tarea y queda fuera.

## Requisitos y evidencia

| Requisito | Decisión | Evidencia |
|---|---|---|
| Criterio de F1-9: plantillas | puntos 6 y 7 | PoC C (la plantilla de `tarea.md` carga con PyYAML y da el mismo título con `C:\ruta\` y con `"`; el escape de la 0.16 no), PoC D (la del plan tiene los títulos de `planTemplate` y los de TASK-009) |
| Criterio de F1-9: qué campos se conservan | punto 3 | PoC A (cifra por campo, por sección y por repo) y PoC E (cada fila de las tablas de campos y de secciones y el recuento citan su cifra; siete copias mutadas fallan) |
| Criterio de F1-9: compatible con la 0.16 y con Spec Kit | puntos 2, 3 (`formato` ausente = 1) y 8 | PoC D y PoC F; ida y vuelta real: no verificada (F2-5) |
| RR-10 carpeta fija, estado como campo | `estado` sin carpeta, `descartada` | PoC A: 9 / 2 / 0 tareas cuyo estado no casa con `05-archivadas` |
| RR-13 actualizar proyectos | `formato: 2`; sin `formato` = 0.16; destino de cada campo | tabla de campos; la migración es de F1-12 |
| RR-14 se construye con la 0.16 | este repo no cambia de formato | «Consecuencias» |
| RR-17 elección por rol anotada | `roles.<rol>.{modelo, skills, regla}` y `roles.revision.agente` | tabla de campos (21 / 0 / 0 tareas con skills; contenido en F1-15) |
| RR-19 progreso y estimación | `estimacion`, `Duracion estimada` en el plan, `## Duracion` | PoC D; PoC A (23 / 33 / 9 tareas con `## Duracion`) |
| RF-01 alta con criterios comprobables | `## Criterios de aceptacion` obligatoria | PoC A (en todas las tareas) |
| RF-07 descartar con motivo | `estado: descartada` + `motivo` | PoC A (6 / 2 / 0 `## Descartada` a mano) |
| RF-22 autorrevisión | `## Autorrevision` opcional | PoC A (33 / 43 / 9) |
| RF-34 segunda opinión (R) | `revision_codex` se retira | PoC A (0 / 0 / 0) |
| RF-51 `## Resultado` | obligatoria al cerrar, reconocida por título | PoC A (159 / 43 / 9; 28 / 14 / 8 tras `## Transiciones`) |
| RF-62 coste por fase | `tokens_*` y `## Coste` se conservan | PoC A |
| RF-71 `05-archivadas` | cubierto por `descartada` + `motivo` | PoC A; si basta del todo, pregunta abierta |
| H11 YAML válido | contrato del texto (punto 5) | PoC C (control negativo con el escape de la 0.16) |
| F1-5 / F1-6 aprobación atada al blob | `plan-final.md` con el mismo nombre y ruta; `plan_aprobado` solo caché | punto 2 y D3a |
| F1-8 texto como dato | esquema estricto, duplicadas rechazadas, desconocidas conservadas con aviso | PoC C (PyYAML acepta duplicadas: el CLI tiene que rechazarlas) |

## Consecuencias

- **F2-5** implementa el parser y el serializador con estas plantillas y
  este contrato: YAML estándar, comillas siempre, claves duplicadas
  rechazadas, desconocidas conservadas, secciones por título; su batería de
  ida y vuelta incluye las tareas de la 0.16 de los tres repos, títulos con
  `\` y `"`, y un proyecto de Spec Kit.
- **F1-12** decide la migración con esta tabla: `formato: 2`, los tres
  campos de selección a `roles`, `agente_revisor` a `roles.revision.agente`,
  fuera `revision_codex` y `ultimo_commit_revisado`, fechas y cadenas entre
  comillas, y las tareas de `05-archivadas` a `estado: descartada` con el
  `motivo` sacado de su `## Descartada` (las que no la tienen, motivo a
  pedir). La ejecutan F9-1 y F9-2.
- **F1-10** fija los valores de `carril`; **F1-15** el contenido de `roles`;
  **F1-17** la forma de `estimacion`; **F2-13** el valor de `asignado_a`;
  **F3-4** lee `estado` (y no carpetas) para el roadmap.

  > **Nota (TASK-077, F2-13, 2026-11-16)**: `asignado_a` es `null` o el
  > objeto en flujo `{id: <entero sin comillas>, login: "<login>"}` de la
  > cuenta de GitHub o GitLab. Se compara solo por `id` (F1-6, decisión 3);
  > `login` es una copia para mostrar que refresca `.taskcode/personas.yml`.
  > Un texto libre de la 0.16 se lee como `Heredado`: la 2.0 nunca crea
  > uno; si lo lee en formato 2 lo conserva tal cual (cadena entre
  > comillas) hasta que se convierta por los `correos:` de
  > `personas.yml` o lo resuelva `reasignar` (F3-10). No reabre este ADR.
- **F1-11**: las claves desconocidas son la costura de las extensiones.
- **Este repo no cambia de formato** (RR-14): sus tareas siguen en la 0.16
  hasta el relevo (F4-5) y la migración de F1-12.

## Fuera de alcance

- Valores de `carril` y qué para cada uno: F1-10.
- Reglas de selección y contenido de `roles`: F1-15.
- Forma y cálculo de `estimacion`: F1-17.
- Migración y simulación: F1-12; implementación: F2-5, F9-1, F9-2.
- Comando y ubicación de la salida a Spec Kit: F2-5 (o la tarea de comandos
  que la recoja).
- Formato del mensaje de commit de las escrituras: F1-4 lo dejó a F1-9 o
  F2-2; F1-9 no lo decide y pasa a F2-2.

## Preguntas abiertas

Este ADR se planificó con un solo rol (arquitectura). Lo que no cubrió:

- **Riesgos**: los modos de fallo del parser con YAML hostil (anclas y
  alias que crecen, etiquetas `!!`, ficheros enormes, tareas de la 0.16 mal
  formadas) no se enumeran aquí; el contrato los acota (esquema estricto,
  duplicadas rechazadas) y la batería es de F2-5. Hoy ninguna tarea real
  rompe PyYAML (PoC A: 0 de 593) porque ningún título lleva `\` ni `"`: H11
  está latente, no corregido.
- **Testing**: solo pruebas mínimas por criterio; la ida y vuelta completa
  es de F2-5.
- **Dominio**: si `descartada` y `motivo` bastan a los proyectos que usan
  `05-archivadas` (RF-71), y si «hito» es lo mismo que `sprint`.
- **Rendimiento**: sin medir; leer N tareas cuesta N ficheros (el roadmap de
  ToolsaWeb tiene 404). Lo mide F3-4.
- **Spec Kit**: la `constitution` y los comandos de Spec Kit no tienen dueño
  en el plan.

## Adenda (F3-2, TASK-029) — clave opcional `lote`

Se reabre por escrito el **punto 3** (campos del frontmatter) para una clave
nueva, decisión D1 de TASK-029 (aceptada por Carlos con la consigna del
2026-11-16).

- **Qué es**: `lote: "<lote>/<clave>"` en una tarea creada por
  `tc2 import`. `<lote>` es el `lote:` de la cabecera del fichero (o, sin
  cabecera, el slug del nombre del fichero) y `<clave>` la de la entrada
  (el primer token del `###`, como `F3-2`, o el slug del título). Por ella
  un import repetido reconoce lo que ya creó y solo crea lo que falta: no
  depende del título, que la persona puede editar.
- **Opcional**, a diferencia de las 21 claves del punto 3: si falta vale
  `null` y solo se escribe con valor, **después de `dependencias`** (la
  última clave conocida, antes de las desconocidas). Una tarea de antes de
  esta adenda se lee y se reescribe byte a byte igual (la dorada de F2-5
  sigue pasando) y `new` no la escribe.
- **Esquema**: texto sin caracteres de control con una `/` que separa dos
  partes no vacías; cualquier otra cosa es error de lectura, como el resto
  del esquema estricto (punto 5). Ya no cuenta como clave desconocida.
- **Lo que no hace**: un import no sincroniza. La tarea ya importada no se
  actualiza aunque su entrada cambie en el fichero; sale en `existentes=`.
- **Hito**: es `sprint` (D2 de TASK-029); el «hito de decisiones» de F1-18
  sigue fuera y se conserva como clave desconocida.

Evidencia: `crates/tc2/src/alta.rs`
(`plantilla_escapa_el_titulo_y_sigue_f1_9`, que fija el orden de claves con
y sin `lote`), `crates/tc2/tests/tarea.rs` (`ida_y_vuelta_byte_a_byte`, sin
cambios) y `crates/tc2/tests/importar.rs`
(`c1_c3_importa_37_en_un_commit_y_repetir_no_crea_nada`,
`c4_dos_entradas_mas_crean_solo_esas_dos`).

## Adenda (F3-10, TASK-081) — filas `liberar` y `reasignar`

Sin claves ni estados nuevos (la lista de estados sigue cerrada):

- **Filas de `## Transiciones`**: `liberar` y `reasignar` añaden su fila con
  la fase del mismo nombre, como `descartar` o `pausa`, y `actualizado` de
  hoy; el resto del `tarea.md`, byte a byte.
- **`liberar`** escribe `asignado_a: null` y, si estaba `en-curso`,
  `estado: planificada` (lista para empezar); conserva `rama:` y
  `plan_aprobado`. **`reasignar`** escribe solo `asignado_a` con el objeto
  `{id, login}` de la persona de `personas.yml` y conserva el estado.
- **Heredados**: `reasignar` es lo que resuelve un `asignado_a` heredado de
  la 0.16 que no casa con ningún `correos:` (la nota de TASK-077 de arriba):
  lo sustituye por el objeto de la persona elegida.
- **Actividad**: la última fila fechada de `## Transiciones` (también
  `pausa` y `reanudar`) cuenta como actividad de la tarea para F3-10, junto
  al último commit de su rama publicada.

Evidencia: `crates/tc2/tests/liberar.rs`
(`c1_c2_el_dueno_libera_y_otra_persona_la_retoma`,
`c3_reasignar_y_el_nuevo_sigue_en_la_rama`, `c12_heredado_queda_como_objeto`)
y los unitarios de `ciclo/liberar.rs` y `tarea/cuerpo.rs`
(`ultima_fecha_de_las_transiciones`).

# Plan — TASK-010: F1-9 ADR formato de tarea, plan y especificacion

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 45 min

Senales: un ADR nuevo, una fila en el indice, sin codigo de la 2.0; una PoC de lectura que mide el uso real de los campos en tres repos y una consulta publica a Spec Kit; 3 criterios. Tareas parecidas de este repo (`start` a `finish`): TASK-002 13 min, TASK-008 31, TASK-009 43 (3 rondas). Cabe en 60 min: no se trocea.

### Que se crea y que no se toca

1. **Se crea `docs/adr/F1-9-formato-de-tarea-plan-y-especificacion.md`** con la plantilla de `docs/adr/README.md`, y su fila (F1-9 / TASK-010 / aceptada) en el indice. Las plantillas van dentro del ADR en bloques de codigo (D2): `tarea.md` (frontmatter y secciones), `plan-final.md` y la correspondencia con Spec Kit.
2. **Se crean `poc/poc-f1-9.sh` y `poc-f1-9.log`** en la carpeta de la tarea: leen en solo lectura los `tarea.md` de ToolsaWeb, OpenDockTrading y TaskCodeTwo y cuentan, por campo, las tareas con valor distinto del valor por defecto, y las secciones `##`. Carpeta de trabajo `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-010\`, borrada al terminar.
3. **No se toca** `tareas/` (salvo `poc/`), `.taskcode/`, `.github/` ni el plugin 0.16 (RR-14: este repo sigue en formato 0.16 hasta el relevo). La ida y vuelta la hace F2-5 y la migracion F1-12.

### Que decide el ADR

- **Un unico modelo canonico: `tarea.md`** en la carpeta fija de F1-1. Spec Kit es **correspondencia de entrada y salida**, no almacenamiento: Objetivo + Criterios + escenarios opcionales ↔ `spec.md`; `plan-final.md` ↔ `plan.md`; sub-tareas o troceo ↔ `tasks.md`. Sin dos ficheros para el mismo hecho (lo que el plan §3 critica de Spec Kit).
- **`plan-final.md` mantiene nombre y ruta**: el trailer `Plan-Blob` de F1-5/F1-6 ata la aprobacion a su blob. La plantilla conserva las secciones de la 0.16: «Duracion estimada» en la primera linea (RR-19), lo que el rol no cubrio, riesgos, pruebas por criterio, decisiones Dn.
- **Tabla de campos «que se conserva»** sobre los 22 de `TASK_FIELD_ORDER` (`src/core/task.ts` del plugin 0.16.1; ToolsaWeb y OpenDockTrading no anaden claves propias), con la cifra de la PoC en cada veredicto:
  - **Se conservan**: `id` (F1-2), `titulo`, `tipo`, `sprint`, `etiquetas`, `complejidad`, `estado`, `rama` (F1-3), `creado`, `actualizado`, `dependencias`, `tokens_*` (RF-62).
  - **Cambian de sentido**: `asignado_a` pasa a usuario de la plataforma (F2-13); `plan_aprobado` queda como cache informativa (la fuente es el commit firmado, F1-5); `estado` gana `descartada` (RF-07) y deja de corresponder a una carpeta; `modelo_sugerido`, `skills_recomendados`, `regla_seleccion_skill` se agrupan por rol para RR-17 (contenido en F1-15).
  - **Se retira**: `revision_codex` (RF-34 marcado R).
  - **Se anaden**: `formato: 2` (migracion F1-12), `carril` (valores de F1-10), hueco para la estimacion (F1-17), `motivo` al descartar.
- **Secciones**: Objetivo, Criterios de aceptacion, Resultado (RF-51, obligatoria), Transiciones; Coste y Autorrevision opcionales. El parser las reconoce por titulo, no por posicion (en ToolsaWeb TASK-008 tiene `## Resultado` tras `## Transiciones`).
- **Contrato del texto**: YAML valido con un parser estandar (H11), cadenas siempre entre comillas; todo es dato no confiable (F1-8, Z2/A10): esquema estricto en campos conocidos y claves desconocidas conservadas en la ida y vuelta con aviso (costura para F1-11).

### Orden

1. PoC de medicion; guardar el log.
2. Spec Kit en solo lectura: `gh api repos/github/spec-kit/contents/templates` y bajar `spec-template.md`, `plan-template.md`, `tasks-template.md` a la carpeta temporal (o la documentacion publica); lo no comprobado, «no verificado».
3. ADR: alternativas, tabla de campos con cifras, plantillas y correspondencia con Spec Kit.
4. Fila del indice, `comprobar-repo.sh`, `taskctl doctor`, borrar la carpeta temporal.

## Lo que el rol no cubrio

Solo arquitectura.

- **Riesgos** (criterio 3): modos de fallo del parser (YAML hostil, claves duplicadas, tareas 0.16 mal formadas, H11); el ADR fija el contrato pero no enumera los casos.
- **Testing**: pruebas minimas por criterio; la bateria de ida y vuelta es de F2-5.
- **Dominio** (criterio 2): si `descartada` y `motivo` bastan para `05-archivadas` (RF-71) y si «hito» es lo mismo que `sprint`.
- **Rendimiento**: sin medir; leer N tareas cuesta N ficheros; el ROADMAP de 403 tareas lo regenera F3-4.

## Riesgos aceptados y que los contiene

- **Formato real de Spec Kit sin leer**: el paso 2 lo comprueba; si no se puede, la correspondencia queda «no verificada» y pasa como criterio a F2-5.
- **Campos cuyo contenido deciden otros ADR** (`carril` F1-10, seleccion F1-15, estimacion F1-17): F1-9 solo fija nombre, tipo y sitio; «Fuera de alcance» nombra el dueno.
- **Cache `plan_aprobado` que puede mentir**: el ADR dice que no es una puerta (F1-5 decision 4) y que `carril` no la lee.
- **Este repo sigue en formato 0.16**: «Consecuencias» manda la conversion a F1-12 y F9.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio). Ninguno pesado.

- **Plantillas → 2**
  1. La plantilla de `tarea.md` del ADR, rellenada con un titulo `C:\ruta\` y otro con `"`, carga con un parser YAML estandar sin error y da los mismos valores (en la carpeta temporal; salida en el log).
  2. La plantilla del plan conserva las secciones que consume la 0.16, cotejando titulos con un `plan-final.md` terminado de este repo (p. ej. TASK-009).
- **Que campos se conservan → 2**
  1. La PoC cuenta, por campo y por repo, los valores distintos del valor por defecto; cada veredicto de la tabla cita su cifra.
  2. Recuento de secciones `##` por repo (Resultado, Coste, Autorrevision y otras), comparado con REQUISITOS §1.
- **Cumple F1-9 y los requisitos que cita → 1**: tabla requisito → decision → evidencia (RR-10, RR-13, RR-17, RR-19, RF-01, RF-07, RF-51, H11, F1-1, F1-5, F1-8; cada ID comprobado antes de citarlo); al cerrar, `bash .github/scripts/comprobar-repo.sh` en verde y `taskctl doctor` sin errores.

## Lo que necesita decision de una persona

Decisiones respondidas por la persona (Carlos) el 2026-11-14 con la consigna «responder la opcion recomendada a todas las preguntas»:

- **D1. Donde vive la «especificacion»** → **(a)** dentro de `tarea.md` (Objetivo, Criterios y `## Escenarios` opcional), con Spec Kit como correspondencia de entrada y salida. Descartadas (b) `spec.md` aparte y (c) `specs/NNN-x/` (choca con F1-1).
- **D2. Donde van las plantillas** → **(a)** dentro del ADR, en bloques de codigo, que F2-5 convierte en ficheros.
- **D3. `plan_aprobado`** → **(a)** se conserva como cache informativa que escribe el CLI y ninguna puerta lee.
- **D4. Claves desconocidas en el frontmatter** → **(a)** se conservan en la ida y vuelta y `doctor` avisa (costura hacia F1-11).
- **D5. `revision_codex`** → **(a)** se retira en el formato 2 y F1-12 lo elimina al migrar.

Sin troceo: la duracion estimada no pasa de 60 min.

# Peticion de redaccion del plan — TASK-073 (ronda 1)

- Tarea: TASK-073 — F1-15 ADR seleccion de skills, agentes y modelo por tarea y rol
- Tipo: feature · Complejidad declarada: no declarada (decide la heuristica)
- Ronda: 1
- Fecha: 2026-11-14
- Rol: `brainstorm-arquitectura` — lanzalo con el agente de ese mismo nombre
- Quien orquesta vuelca tu respuesta en: `../plan-final.md`

**Esta tarea se planifica con un solo rol y no hay unificador**: tu respuesta ES el plan final. Redactala con las secciones de abajo; quien orquesta la vuelca tal cual en `plan-final.md` (tu no escribes ficheros), sin salida intermedia que consolidar.

## Tu pregunta

> Dado lo que ya existe, ¿donde encaja este cambio y que forma tiene?

## Que miras

- Los modulos y ficheros que ya resuelven algo parecido, para extenderlos en vez de duplicarlos.
- Que se crea nuevo, que se extiende y en que orden se construye.
- Los limites que el cambio cruza: contratos publicos, formatos de fichero, esquemas.
- El precedente interno mas cercano: como se resolvio la ultima vez un problema de esta forma.

## Que NO miras

Que lo mires solo tu no te autoriza a cubrirlo todo: lo que queda fuera de tu pregunta no se mira, pero se DICE. Anota en el plan lo que no cubres, para que quien lo apruebe sepa que ese lado no se ha revisado:

- Como se prueba (es del rol de testing).
- Por donde se rompe (es del rol de riesgos).
- Las reglas de negocio (son del rol de dominio).

## Complejidad: declarada frente a heuristica

- Declarada en la tarea: **no declarada (decide la heuristica)**
- Heuristica (2 puntos): **simple**
- Senales encontradas:
  - dependencias: 2 dependencias (TASK-010, TASK-011) (+2)

Los dos niveles coinciden. Se lanzan 1 rol.

## Que tiene que traer el plan final

- Enfoque propuesto, concreto: que se crea, que se extiende, en que orden.
- Lo que el rol no cubrio, contrastado contra los criterios de aceptacion.
- Riesgos aceptados y que los contiene.
- Plan de pruebas.
- Lo que necesita decision de una persona antes de implementar. **El checkpoint humano es obligatorio**: este plan no vale hasta que alguien lo apruebe con `taskctl approve`.

## Reglas

- **Propones enfoque; no implementas.** No escribas codigo de produccion ni modifiques ficheros del repo: tu salida es el plan.
- Evidencia, no suposicion: si afirmas que algo se comporta de cierta manera, es porque lo has mirado. Di de donde lo sacas.

## Enunciado de la tarea

### Objetivo

````
ADR: selección de skills, agentes y modelo por tarea y por rol (RR-17): fuentes (catálogo del proyecto, skills y agentes instalados, revisoras del plugin), reglas deterministas por etiquetas, rutas, dominio, complejidad y carril, qué se anota en la tarea y cómo se mide el acierto

Tarea F1-15 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-9, F1-10.

Contexto (Carlos, 2026-11-14, RR-17, imprescindible): para cada tarea y cada rol (diseño, implementación, revisión) hay que elegir las skills y los agentes adecuados y el modelo según complejidad y carril, solo entre lo que está instalado en la máquina y el proyecto, anotar la elección en la tarea y medir si acierta (hallazgos por revisora y coste por modelo). En la 0.16 existe con `catalogo-skills.yml`, `skills_recomendados` y `modelo_sugerido`, pero se usó en 21 / 0 / 2 tareas de ToolsaWeb / OpenDockTrading / TaskCode porque el catálogo apenas se configuró: hoy la elección la hace el agente sin criterio registrado. Nada debe decidirlo el modelo «a ojo»: reglas deterministas. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §1–4.
````

### Criterios de aceptacion

````
Reglas probadas a mano sobre 10 tareas reales de ToolsaWeb y OpenDockTrading
nada lo decide el modelo «a ojo»
Cumple lo que pide la tarea F1-15 del plan y los requisitos que cita
````

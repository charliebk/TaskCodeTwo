# Peticion de brainstorm — TASK-138, rol riesgos (ronda 1)

- Tarea: TASK-138 — Auditoria 4D de todo el codigo de TaskCodeTwo
- Tipo: feature · Complejidad declarada: media
- Ronda: 1
- Fecha: 2026-11-22
- Rol: `brainstorm-riesgos` — lanzalo con el agente de ese mismo nombre
- Vuelca tu respuesta en: `salida-brainstorm-riesgos-1.md`

## Tu pregunta

> ¿Por donde se rompe esto?

## Que miras

- Bordes y estados intermedios: que queda a medias si el proceso muere a mitad.
- Fallos parciales y concurrencia: dos ejecuciones, un recurso ocupado, un permiso denegado.
- Compatibilidad hacia atras con los datos y ficheros que YA existen.
- La vuelta atras: si esto sale mal, como se deshace y que queda inservible.

## Que NO miras

No es una lista de cortesia. Si la ignoras y lo cubres todo "por si acaso", el acotado de contexto se deshace sin que se note y esta tarea deja de tener varios puntos de vista:

- Donde encaja el cambio (es del rol de arquitectura).
- Que aserciones escribir (es del rol de testing).
- Las reglas de negocio (son del rol de dominio).

## Enunciado de la tarea

### Objetivo

````
Auditar todo el codigo existente de TaskCodeTwo (crates/tc2/src, crates/tc2/tests, xtask, crates/distribucion y .github/workflows) contra la regla 4D de Carlos, recomendada por TaskCode 0.17 al actualizar: 1) Duplicacion: dos funciones o tipos que hacen lo mismo con pequenas diferencias. 2) Desviacion: codigo que se sale del objetivo de su tarea o de su ADR (o un criterio dado por cumplido sin estarlo). 3) Diluvio: lineas y tipos de mas donde bastan pocos. 4) Disfraz: codigo o tests que aparentan resolver algo y no lo comprueban. La auditoria solo diagnostica; las correcciones van en tareas de seguimiento.
````

### Criterios de aceptacion

````
docs/auditoria-4d.md lista cada caso con su regla (D1 a D4), fichero:linea, la evidencia que lo prueba (los dos fragmentos duplicados, el mutante que sobrevive, el criterio desviado) y su severidad CRITICO, IMPORTANTE o MENOR
Cubre todos los modulos de crates/tc2/src, los tests, xtask, crates/distribucion y los workflows; el documento lleva una tabla de cobertura por modulo con el numero de casos
Cada caso IMPORTANTE o CRITICO tiene su tarea de seguimiento en tareas/00-planificadas, agrupadas por modulo
Los casos de Disfraz en tests se prueban con un mutante que sobrevive, ejecutado de verdad
````

## Como entregas

Escribe en `salida-brainstorm-riesgos-1.md`, con estas secciones y en este orden:

- Modos de fallo, ordenados por gravedad, con el escenario concreto de cada uno
- Estados intermedios y fallos parciales
- Compatibilidad hacia atras
- Vuelta atras
- El riesgo que mas te preocupa (UNO solo)

## Reglas

- **Propones enfoque; no implementas.** No escribas codigo de produccion ni modifiques ficheros del repo: tu salida es un documento.
- Evidencia, no suposicion: si afirmas que algo se comporta de cierta manera, es porque lo has mirado. Di de donde lo sacas.
- Prefiere lo concreto: rutas, nombres y casos reales por encima de recomendaciones genericas.
- Trabajan en paralelo contigo, sin verte: **arquitectura**. No cubras lo suyo — si lo haces, el unificador recibira el mismo punto de vista repetido y lo leera como confirmacion.

# Peticion de brainstorm — TASK-118, rol riesgos (ronda 1)

- Tarea: TASK-118 — Aplicar las puertas (rulesets) desde tc2 init
- Tipo: feature · Complejidad declarada: media
- Ronda: 1
- Fecha: 2026-11-18
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
TASK-028 (F3-1) solo propone los rulesets taskcodetwo-*. Aplicarlos exige los checks verificacion, revision-ia y carril (F5-1, F6) y el integration_id real (F6-1). Esta tarea anade aplicar_puertas al trait Plataforma (GitHub y GitLab) y la opcion de init que las aplica con permiso de la persona.
````

### Criterios de aceptacion

````
tc2 init --aplicar-puertas crea los rulesets taskcodetwo-* en GitHub y GitLab (test con doble HTTP)
Sin los checks ni el integration_id real, la opcion se rechaza con el motivo (test)
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

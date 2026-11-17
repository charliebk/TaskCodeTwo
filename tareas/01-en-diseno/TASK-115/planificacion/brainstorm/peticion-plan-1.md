# Peticion de redaccion del plan — TASK-115 (ronda 1)

- Tarea: TASK-115 — Seguimiento de F2-12: menores de actualizar --cli y mock HTTP estable
- Tipo: fix · Complejidad declarada: simple
- Ronda: 1
- Fecha: 2026-11-17
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

- Declarada en la tarea: **simple**
- Heuristica (2 puntos): **simple**
- Senales encontradas:
  - etiquetas_adicionales: 2 etiquetas (1 mas alla de la primera) (+1)
  - criterios_aceptacion: 5 criterios (umbral: 5) (+1)

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
Cerrar los MENORES aceptados en TASK-027 y estabilizar tests/comun/http_mock.rs, que da sinred intermitente en esta maquina.
````

### Criterios de aceptacion

````
ejecutar_con_limite devuelve la salida ya leida cuando un nieto retiene stdout y su doc dice lo que hace (test con nieto, MEN-7 r3)
Un fallo de TLS no se reintenta y el timeout global no dobla la espera (tests en crates/tc2/src/descarga.rs, MEN-8 y MEN-9 r3)
Tests de integracion de redireccion no permitida y de limite de tamano en crates/tc2/tests/actualizar_cli.rs (MEN-5 r1)
tests/comun/http_mock.rs no manda FIN tras una respuesta completa y plataforma_gitlab pasa 10 de 10 seguidas
El doc de bajar_con_un_reintento esta sobre su funcion y sin lineas duplicadas
````

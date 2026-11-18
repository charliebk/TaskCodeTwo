---
id: TASK-127
titulo: "Escrituras de estado con muchos ficheros fallan en Windows (git hash-object)"
tipo: fix
sprint: 0
etiquetas: [estado]
complejidad: simple
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: false
rama: fix/task-127-escrituras-de-estado-con-muchos-ficheros
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-18
actualizado: 2026-11-18
dependencias: []
---
## Objetivo

Una operacion de estado que escribe muchos ficheros a la vez (p. ej. 1000 en un import grande) falla con git=hash-object en Windows, probablemente por la longitud de la linea de ordenes (visto en TASK-084).

## Criterios de aceptacion
- [ ] Una transaccion que escribe 1000 ficheros en una sola operacion termina en ok en Windows (test)
- [ ] El numero de procesos de git no crece con el numero de ficheros mas alla de un tope fijo (hash-object por stdin o por tandas)

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-18T13:17:32Z | plan | automatico | persona | 0.16.1 |

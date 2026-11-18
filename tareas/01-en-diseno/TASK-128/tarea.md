---
id: TASK-128
titulo: "Seguimiento de F3-11 y F3-12: aviso de solape contra uno mismo, textos al catalogo y test intermitente de progreso"
tipo: fix
sprint: 0
etiquetas: [solape, progreso]
complejidad: simple
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: false
rama: fix/task-128-seguimiento-de-f3-11-y-f3-12-aviso-de-so
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

Cerrar lo que quedo fuera de los ficheros citados en la revision de TASK-082 y el test intermitente que aparecio en ella.

## Criterios de aceptacion
- [ ] start no avisa de solape entre dos tareas de la misma persona (la propia lleva su quien hasta solape::calcular en la toma nueva); el test de liberar c5_c6_c10 pasa a exigir stderr vacio (test)
- [ ] AVISO_Y_MAS y NOTA_Y_MAS viven en i18n/catalogo/toma.rs y los recorre el test del catalogo
- [ ] progreso::c3_transiciones_reales_reescriben_el_progreso pasa 10 de 10 bajo carga: la rotacion de .anterior.json no se ignora en silencio o el test no depende de ella

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-18T17:54:47Z | plan | automatico | persona | 0.16.1 |

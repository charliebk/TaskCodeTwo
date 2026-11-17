---
id: TASK-029
titulo: "F3-2 tc2 new e import en bloque con dependencias e hito"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: false
rama: feature/task-029-f3-2-new-e-import-en-bloque-con-dependen
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-019, TASK-020]
---
## Objetivo

`new` e `import` en bloque con dependencias e hito (RF-01..03)

Tarea F3-2 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-4, F2-5.

## Criterios de aceptacion
- [ ] C1: `tc2 import hito37.md` sobre un remoto bare da un commit, 37 carpetas con IDs consecutivos y cada `tarea.md` con su sprint, objetivo y criterios (`crates/tc2/tests/importar.rs`)
- [ ] C2: las dependencias locales (clave del fichero) quedan resueltas a `TASK-NNN` y las externas validadas; un ciclo o una referencia inexistente dan codigo 2 (uso) sin commit (la rama de estado no cambia de SHA)
- [ ] C3: repetir C1 da codigo 0 con `creadas=0 existentes=37` y ningun commit nuevo
- [ ] C4: el mismo fichero con 2 entradas mas crea solo esas 2, con sus dependencias hacia las ya importadas resueltas
- [ ] C5: una entrada invalida en la posicion 30 no escribe nada; dos clones importando a la vez dan IDs sin solape y sin huecos dentro de cada lote
- [ ] C6: `tc2 import --simular` imprime los IDs previstos y no cambia ni un byte en local ni en el remoto
- [ ] C7: pasar los limites (1 MiB, 200 entradas) da `uso` con su `motivo=`; sin red da `sinred` con `pendiente=no`
- [ ] C8: el fichero de import de formato 0.16 de `docs/plan/AMPLIACION-2026-11-16.md` §5 se importa sin tocarlo
- [ ] C9: `tc2 new` acepta los campos de RF-01 y rechaza un enunciado que no esta listo (test)

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T04:56:46Z | plan | automatico | persona | 0.16.1 |

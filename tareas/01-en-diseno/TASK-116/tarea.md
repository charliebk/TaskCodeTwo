---
id: TASK-116
titulo: "Nombres de la 2.0 alineados con F1-12 D1a"
tipo: fix
sprint: 0
etiquetas: [convivencia, git]
complejidad: simple
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: false
rama: fix/task-116-nombres-de-la-2-0-alineados-con-f1-12-d1
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-16
actualizado: 2026-11-16
dependencias: []
---
## Objetivo

El codigo de tc2 usa nombres que chocan con la 0.16 y contradicen F1-12 D1a (taskcode/tareas, .git/taskcode-estado.lock, .git/tc2/, .taskcode/worktrees). Alinearlos con D1a para que la 2.0 y la 0.16 no compartan nada y tc2 desinstalar (F2-14) pueda reconocer lo suyo por nombre.

## Criterios de aceptacion
- [ ] La rama de estado es taskcodetwo/tareas (estado::RAMA) y los tests de estado y alta pasan con ese nombre
- [ ] El cerrojo, la cache de identidad y lo demas que tc2 guarda en el directorio comun de git viven bajo .git/taskcodetwo/ (grep sin taskcode-estado.lock ni .git/tc2 en crates/tc2/src)
- [ ] La carpeta de worktrees de tarea no coincide con ninguna ruta de la 0.16 (constante en git/mod.rs y test)
- [ ] Un repo con huella de la 0.16 (.git/taskcode/, tareas/ en develop) queda intacto tras new/start de la 2.0 (test)
- [ ] cargo xtask verificar --perfil rapida en verde

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-16T22:37:06Z | plan | automatico | persona | 0.16.1 |

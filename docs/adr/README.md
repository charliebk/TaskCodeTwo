# Decisiones de arquitectura (ADR)

Las tareas de la Fase 1 del [plan](../plan/PLAN-TASKCODE-2.md) dejan cada una
un ADR aquí. Nombre: `F1-N-<tema>.md`, con el F-id del plan, para rastrearlo
directo a la tabla de la Fase 1 y a su tarea.

## Índice

| ADR | Tema | Tarea | Estado |
|---|---|---|---|
| [F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) | Carpeta fija por tarea y rama de estado `taskcode/tareas` | TASK-002 | aceptada |
| [F1-2](F1-2-escrituras-atomicas-ids-y-toma.md) | Escrituras atómicas en la rama de estado, reserva de IDs y toma de tareas | TASK-003 | aceptada |
| [F1-3](F1-3-flujo-de-ramas-worktrees-release.md) | Flujo de ramas `develop`/`main`, rama por tarea con worktree, release y hotfix; qué queda de Git-Flow | TASK-004 | aceptada |
| [F1-4](F1-4-operaciones-git-y-contrato-de-salida.md) | Capa de operaciones Git/GitHub deterministas: lista cerrada, contrato de salida, códigos y objetivo de tokens por tarea | TASK-005 | aceptada |
| [F1-5](F1-5-puertas-en-github.md) | Puertas en GitHub por carril: rulesets por rama y tag, checks obligatorios con su origen, check `carril`, CODEOWNERS, cola y rama al día | TASK-006 | aceptada |

## Plantilla

Cada ADR lleva, en este orden (un ADR puede añadir secciones si dice por qué):

1. **Cabecera**: estado y fecha, tarea, enlace al plan, requisitos que cita,
   quién decide.
2. **Contexto**: el problema y los requisitos que lo enmarcan.
3. **Alternativas**: tabla con descripción y veredicto de cada una.
4. **Pruebas de concepto**: comandos y salida observada, si las hay.
   Se ejecutan en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\<tarea>\` y se
   borran al terminar. Lo que no se pudo comprobar se marca «no verificado».
5. **Cómo se ve en GitHub** (o en GitLab), si aplica.
6. **Decisión**.
7. **Requisitos y evidencia**: tabla requisito → alternativa → evidencia.
8. **Consecuencias**: qué tareas la ejecutan y qué cambia.
9. **Fuera de alcance**: qué decide otro ADR.

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
| [F1-6](F1-6-identidades-agente-revisor-persona.md) | Identidades del agente implementador, del revisor IA y de la persona: operación → identidad, permisos mínimos de dos GitHub Apps, contrato del firmante y por qué ninguna IA aprueba ni mergea | TASK-007 | aceptada |
| [F1-7](F1-7-verificacion-ci-perfiles-cache-runners.md) | Verificación en CI: un comando con perfiles, caché por árbol en el check `verificacion` con búsqueda en dos niveles de padres, perfil por evento, runner por perfil (self-hosted solo en privados, falla cerrado) y coste con ToolsaWeb y OpenDockTrading | TASK-008 | aceptada |
| [F1-8](F1-8-seguridad-modelo-de-amenazas.md) | Seguridad: modelo de amenazas por fronteras de confianza, dos clases de jobs (ejecuta código del PR / tiene privilegio), revisor partido en dos jobs, confianza local única con hash de la orden y de lo que ejecuta, prompt injection por capacidades, runner propio aislado y rotación de secretos | TASK-009 | aceptada |
| [F1-9](F1-9-formato-de-tarea-plan-y-especificacion.md) | Formato de tarea, plan y especificación: `tarea.md` como único modelo canónico (formato 2, YAML estándar, cadenas entre comillas, texto como dato no confiable), tabla de los 22 campos de la 0.16 con su uso medido, plantillas de `tarea.md` y `plan-final.md`, y correspondencia con Spec Kit | TASK-010 | aceptada |
| [F1-10](F1-10-carriles-y-modo-automatico.md) | Carriles y modo automático: cuatro carriles ordenados más release y hotfix por tipo, tabla única de política por carril con su función pura (`accion` en `siguiente --json`), configuración que solo endurece (`politica_carriles`, `rutas_carril`), carril mínimo por ficheros con los dos nombres de cada renombrado, y paradas fijas (`approve`, `merge-pr`, tag, tope de rondas, subida de carril) | TASK-011 | aceptada |
| [F1-11](F1-11-extensiones-declaradas.md) | Extensiones declaradas: inventario comprobado de las 14 prácticas que los pilotos hacen a mano, clasificadas en parche de TaskCode (se retira al migrar) o propio del proyecto, con el desglose de líneas; tres claves (`verificacion`, `derivados`, `confianza_incluye`) con esquema que falla cerrado y rutas normalizadas; derivados que ejecuta `taskctl` en la transición solo en la rama de estado, comparados por contenido; check obligatorio `estado`; `taskctl tareas --json` como lectura estable; sin estados propios; revisoras como skill del proyecto; «comandos sí, hooks solo como atajo opcional»; y la ventaja medida frente a la 0.16, con lo que no gana | TASK-012 | aceptada |
| [F1-12](F1-12-actualizacion-entre-versiones.md) | Actualización entre versiones: el plan de migración como dato (`tc2 actualizar --simular` sin cambiar un byte, mismo hash, vacío en la segunda pasada), qué migra (aplanado, formato 2, archivadas a `descartada`, config a los bloques de F1-11), historia con `git subtree split`, transacción local con criterio de salida estructural y vuelta atrás automática, retirada de parches con capacidades y huecos bloqueantes, convivencia con la 0.16 (su lector de config aborta con `---`: ninguna orden de escritura entra en un proyecto migrado), nombres definitivos `tc2` y `taskcodetwo`, y orden de pilotos OpenDockTrading → TaskCodeTwo → ToolsaWeb | TASK-013 | aceptada |

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

# Brainstorm — TASK-009, rol arquitectura (ronda 1)

- Rol: `brainstorm-arquitectura`
- Agente: brainstorm-arquitectura (a75a8274814f57c60)

## Enfoque propuesto, con rutas y nombres concretos

- Un solo ADR nuevo: `docs/adr/F1-8-seguridad-modelo-de-amenazas.md`. Sigue la plantilla de `docs/adr/README.md` y le anade una seccion «Modelo de amenazas» entre Contexto y Alternativas (la plantilla lo permite si se dice por que).
- El modelo se ordena por **fronteras de confianza**, no por una lista de amenazas. Cinco zonas: Z1 maquina de la persona (CLI, agente, hook, claves); Z2 contenido del repo (`.taskcode/`, `.github/`, texto de tareas y PR), dato no confiable; Z3 runners de GitHub; Z4 runner propio Windows; Z5 plataforma (rulesets, Apps).
- Una matriz «activo → quien lo lee o escribe → mitigacion → ADR o tarea duena» sustituye a la prosa. Activos: clave de la App del agente, secreto del revisor, clave de firma de la persona, checks `verificacion`, `carril` y `revision-ia`, runner propio.
- Los workflows se reparten en dos clases, fijadas por la estructura:
  - **Ejecutan codigo del PR**: `verificacion` y la reproduccion y los mutantes de F6-4. `pull_request`, `permissions: read`, sin secretos. En el runner propio solo si el repo es privado y sin PR de forks (F1-7, decision 5).
  - **Tienen privilegio**: `carril` y la publicacion de `revision-ia` con la clave de la App. `pull_request_target` o `workflow_run`, con checkout solo de la base; el diff y los artefactos entran **como datos**.
- La defensa contra prompt injection va **por capacidades**, no por deteccion: que puede hacer cada LLM que lee texto hostil (el agente en local, en lotes de RR-18, y el revisor en CI). Se apoya en lo que F1-6 ya garantiza: ninguna IA aprueba ni integra.

## Que se extiende y que se crea

- **Se crea**: el ADR F1-8 y una fila en el indice de `docs/adr/README.md`.
- **Se extiende, sin duplicar**: las preguntas abiertas que tres ADR aceptados delegan en F1-8, en una tabla «pregunta heredada → decision → origen»:
  - F1-5, decision 6 (l. 453-465) y l. 589-590: `carril` y CODEOWNERS como mitigacion parcial; `pull_request_target` o workflow leido de la base.
  - F1-6, l. 576-580 y 627-635: `repository_dispatch`, claves de agente y persona en la misma maquina, tarea legitima que toca `.github/workflows/`.
  - F1-7, l. 488-491: runner propio con codigo del PR, `verificacion` falsificable por cualquier workflow con `checks:write` (todos publican como `integration_id` 15368), y la confianza del hook.
- Seccion «Divergencias con F1-5/F1-6/F1-7» (precedente: «Divergencias con F1-5» en F1-7, l. 443): cualquier cambio de una decision aceptada se nombra ahi y no se reabre a escondidas.
- **Un solo modelo de confianza local**, compartido por F0-2 (`taskctl confiar`, hash fuera del repo, PLAN l. 292), F5-6 (hook), RF-70 (`comando_sincronizacion` «con confianza») y H2 de la auditoria. Una sola interfaz, del estilo `confianza(repo, config) -> confiado | cambiado | nunca`, con ubicacion fuera del arbol. Si cada consumidor tuviera la suya, habria tres comportamientos para el mismo hecho.
- Propuesta concreta: ningun workflow de TaskCode se suscribe a `repository_dispatch` y `doctor` (F3-8) lo comprueba; cierra lo que F1-6 deja abierto sin tocar los permisos de la App.

## Limites que cruza

- **Contrato del workflow reutilizable de F5-1**: anade la clase del job (ejecuta o tiene privilegio), eventos admitidos y `permissions` por job. F5-1, F5-5 y F6-3 heredan esta forma; cambiarla despues obliga a rehacer los workflows.
- **Esquema de `.taskcode/` (F1-11)**: los `comando_*` pasan a ser entrada no confiable en local y en CI; F1-8 lo declara, las claves las decide F1-11.
- **Puerto `Plataforma` de F1-4**: quiza suma una lectura para `doctor` (eventos suscritos, aprobacion de workflows de forks); no suma escrituras.
- **Consecuencias que asigna**: F0-2, F5-1, F5-5, F5-6, F6-3, F6-4, F6-7 y F9-5 como duenas de cada mitigacion. F1-10 se queda el merge por el agente con todo en verde y los tags; F1-8 no lo cierra.
- **Pruebas de concepto con evidencia permitida**: en GitHub solo `GET` (permisos de los workflows y ajustes de Actions; F1-6 ya leyo `actions/permissions/workflow`); en local, en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-009\`, el hash de confianza con la config cambiada y sin cambiar, y la clasificacion estatica de workflows de ejemplo por clase de job.
- **Alternativas descartadas**: varios ADR (local, CI, injection) —la misma config y el mismo texto hostil son entrada a la vez del hook, del workflow y del LLM; partirlo separa una frontera que es una sola—; todo con `pull_request_target` —ejecutar el head del PR con secretos es justo el patron a evitar—.
- **Desacuerdos previstos con riesgos**: forma del ADR (riesgos puede pedir STRIDE exhaustivo; posicion: la tabla de amenazas puede ser suya, pero el ADR se ordena por fronteras y cada mitigacion tiene tarea duena); defensa contra inyeccion (puede pedir deteccion; posicion: la garantia es lo que el LLM no puede hacer, P3).

## La decision de diseño que mas te preocupa (UNA sola)

Donde corre el **revisor IA** de F6-3 y F6-4: es el unico componente que a la vez lee la entrada mas hostil (el diff del PR), necesita **ejecutar** codigo del PR (reproduccion y mutantes de F6-4) y tiene secretos (la clave de la App del revisor y, sin verificar, una clave de API del modelo). F1-6 (D5a) solo coloco la clave de la App. Propuesta: partirlo en dos jobs: (1) ejecucion y LLM en `pull_request`, sin la clave de la App, con su salida como artefacto; (2) publicacion con `workflow_run`, que valida el artefacto como datos y publica `revision-ia`. Queda abierto donde vive la clave del modelo: en el job 1 el codigo del PR puede robarla; en el job 2 el LLM no puede reproducir nada. La forma que se elija la heredan F6-3, F6-4 y F6-7.

Suposiciones no verificadas: que F0-2 se aplica a la 0.16 y F5-6 reutilizara su mecanismo; que el revisor necesita una clave de API del modelo en Actions (ningun ADR la menciona); que `workflow_run` recibe los artefactos del run de `pull_request` en repos privados sin forks; no se leyo `AMPLIACION-2026-11-14.md` entero (RR-18 amplia la superficie de inyeccion del agente local).

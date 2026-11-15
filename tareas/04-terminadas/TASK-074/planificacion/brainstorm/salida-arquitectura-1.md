# Salida del rol de arquitectura — TASK-074 (ronda 1)

(Volcada por el orquestador.)

## Enfoque

`tc2 lote` es un planificador determinista del CLI sobre lo existente: `politica`/`accion` de F1-10, `seleccionar` de F1-15 y la capa de operaciones de F1-4; no decide carriles por su cuenta. Estado: diario `lotes/<nombre>.yml` en `taskcodetwo/tareas` (de el salen reanudacion e informe). Cada tarea entra en `lote/<nombre>` por un PR con checks obligatorios; la persona aprueba un solo PR del lote contra `develop`. Para que nadie toque lo que no debe: ruleset sin bypass en `lote/**`, agente lanzado sin terminal (no hay prompt donde escribir) y guarda de procedencia antes de `review` y de integrar.

## Piezas y limites

- **`tc2 lote` + nucleo puro** `planificar(tareas, deps, politica) -> Orden` y `siguiente_paso(diario, orden, presupuesto) -> Paso` (se crea): orden por dependencias (empate por ID), solo lo que admite la columna «Lote» de F1-10, exige `approve` firmado antes de lanzar; paradas de `accion`: el `merge-pr` de cada tarea pasa al PR final y el tope de rondas lo resuelve una regla. Presupuesto entre pasos (se puede pasar como mucho en un paso).
- **Diario `lotes/<nombre>.yml`** (se extiende `commit-estado` de F1-2): cada paso con SHA, resultado, tokens y tiempo; marca `asignado` al lote cada tarea tomada (toma de F1-2) para que un companero no haga `start` sobre ella; `--reanudar` lo contrasta con las ramas del remoto y sigue por el primer paso sin cerrar (operaciones de F1-4 idempotentes).
- **Operacion nueva `integrar-en-lote`** en la lista cerrada de F1-4: abre el PR de la tarea contra `lote/<nombre>` y lo mergea por API con la App del agente solo si la base es `lote/*` y `estado-puertas` esta en verde. `merge-pr` sigue siendo solo de la persona (una vez, PR del lote contra `develop`).
- **Ruleset `taskcodetwo-lote`** en `refs/heads/lote/**` (F1-5): `deletion`, `non_fast_forward`, `pull_request` con 0 aprobaciones y checks `verificacion`, `revision-ia` y `carril`, sin `bypass_actors`; nadie empuja directo y lo rojo no entra (RR-08). GitLab (F1-13, G02 Free): rama protegida `lote/*`, «Allowed to push» nadie, «Allowed to merge» el bot del agente, «Pipelines must succeed». F1-7: fila de perfil para base `lote/*` («completa»).
- **Guarda `comprobar_procedencia(diario, rama) -> Limpia | Ajena{commits, ficheros}`** (se crea), antes de `review` y de `integrar-en-lote`: cabeza remota de la rama de tarea y de `lote/*` igual a la ultima SHA del diario; todo commit de la rama creado por `guardar` en ese paso; worktree limpio al devolver el control. `Ajena` en rama de tarea → se aparta; en `lote/*` → el lote para. Agente lanzado sin stdin sobre `.taskcode/worktrees/`, nunca sobre el principal.
- **`informe(diario) -> Markdown`**, funcion pura (se crea): rama de estado y descripcion del PR del lote; terminadas, apartadas con motivo y dependientes, hallazgos sin corregir, tokens y tiempo.

## Orden de construccion

1. Contratos: esquema del diario, `planificar`/`siguiente_paso`, regla de lote sobre las paradas fijas de F1-10.
2. Divergencias con nombre: F1-6 decision 4 («ninguna IA mergea» → «ninguna IA mergea en `develop`/`main`»), F1-3 (`lote/*`), F1-4 (operacion nueva), F1-5 (ruleset), F1-7 (fila de perfil), F1-13 (fila), F1-8 A16.
3. PoC sobre una copia de ToolsaWeb en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-074\`, remoto bare local y Plataforma simulada. Lote real: serie de documentacion TASK-173..TASK-191 (14 tareas solo `docs/`, errata, segun la PoC A de F1-10) mas dos o tres de codigo con dependencias; lo que hace el agente se simula repitiendo el diff real de cada merge (`M^1..M^2`); rondas de su `tarea.md`.
4. Inyectar: tarea en rojo (se aparta con dependientes), corte a mitad y `--reanudar`, commit ajeno en un worktree y en `lote/*`, presupuesto agotado; comparar el arbol final de `lote/*` con el de `develop` real tras la serie.
5. Informe con el diario de la PoC y tabla requisito → evidencia.

## Alternativas descartadas

- Merge local `--no-ff` del CLI sin PR por tarea: «nunca en rojo» lo impondria el CLI y no la plataforma (RR-08); F1-3 retiro los merges locales; ningun ruleset impediria empujar a la rama del lote.
- Orquestar desde el agente (skill o hook) o integrar cada tarea en `develop`: contra RR-01/RR-18 y «comandos sobre hooks»; contra F1-6 y la revision humana unica.
- Coste de la forma: un CI por tarea mas uno final; tiempo N × (agente + CI) en serie por diseno; paralelizar es de F9-4.

## Desacuerdos previstos

- Riesgos: `integrar-en-lote` con la App del agente reabre F1-6 y A16. Posicion: `lote/*` no es la puerta; la puerta es el PR final; A5 igual que fuera del lote.
- Riesgos: bloquear el sistema de ficheros. Posicion: detectar y lanzar sin terminal, no bloquear (en Windows no hay bloqueo portable).
- Tope de rondas en lote: ronda extra hasta `tope_rondas` y despues apartar, sin preguntar; choca con «fidelidad sobre coste»; que lo decida Carlos.

## Suposiciones no verificadas

- Que el agente sin terminal (p. ej. `claude -p --output-format json`) devuelve tokens por ejecucion.
- Que un ruleset con `pull_request` de 0 aprobaciones en `lote/**` deja mergear por API a la App del agente (sin probar, F2-6), igual en GitLab.
- Reutilizacion de la cache por arbol de F1-7 en el PR final si `develop` no se movio.
- La serie TASK-173..TASK-191 (de la PoC A de F1-10, commit `93c9618ac`): confirmar con `git log` sobre una copia.
- No cubierto: casos limite y amenazas (riesgos), pruebas (testing), reglas de negocio (dominio).

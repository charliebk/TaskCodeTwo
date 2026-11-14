# Plan — TASK-007: F1-6 ADR identidades del agente, del revisor y de la persona

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 60 min

Senales: un ADR nuevo y una fila en el indice, sin codigo de la 2.0; evidencia de plataforma solo de lectura (`GET`, esquema y documentacion publicos de permisos de Apps) y una PoC local corta de firma SSH; 3 criterios. Precedente: TASK-006 estimo 60 y tardo 38 de `start` a `finish`. Cabe en 60 min: no se trocea.

### Que se crea y que no se toca

1. **Se crea** `docs/adr/F1-6-identidades-agente-revisor-persona.md` con la plantilla de `docs/adr/README.md`, incluida «Como se ve en GitHub».
2. **Se extiende** `docs/adr/README.md` con la fila F1-6 / TASK-007.
3. **Se crean** `poc/poc-f1-6.sh` y `poc/poc-f1-6.log`, ejecutados en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-007\` (borrada al terminar); el material descargado va en su propia carpeta vacia y se lee con `python -I`.
4. **No se tocan** F1-1..F1-5 (se citan), `.github/` ni los JSON de rulesets de TASK-006 (`revision-ia@0` sigue de relleno hasta F6-1). Nada se escribe en GitHub.

### Que decide el ADR

- **Ningun modulo nuevo: tabla «operacion → identidad»** sobre la lista cerrada del puerto `Plataforma` (F1-4/F1-5). El puerto recibe un proveedor de credenciales por rol (`agente | revisor | persona`) y cada operacion declara con cual corre:
  - **agente**: push de ramas de tarea (tambien `actualizar-rama`, por el `require_last_push_approval` de F1-5), push a `taskcode/tareas`, `crear-pr` y `actualizar-pr` (tambien el PR de release, para que la persona pueda aprobarlo).
  - **persona** (TTY y token propios): `merge-pr`, `aplicar-puertas`, tag y release, y el commit firmado de `approve`.
  - **revisor**: el check `revision-ia`.
  - `leer-puertas`: cualquiera.
- **Tres identidades con permisos minimos** (nombres de permiso de Apps de GitHub):
  - **Agente (GitHub App)**: `contents:write`, `pull_requests:write`, `metadata:read`. Sin `administration`, sin `workflows` (no puede empujar cambios en `.github/workflows/`: cierra en la plataforma parte del riesgo «workflow modificado en el propio PR» de F1-5) y sin `checks:write`.
  - **Revisor (otra GitHub App)**: `checks:write`, `contents:read`, `pull_requests:read`, `metadata:read`. Sin `pull_requests:write` no crea reviews (tampoco APPROVE); sin `contents:write` no mergea. Su `app_id` es el `integration_id` de `revision-ia`.
  - **Persona**: su cuenta, con una clave de firma registrada fuera del alcance del agente.
- **Contrato del firmante** que usa `carril` (completa la regla 1 de F1-5): `verification.verified=true`, login en la lista de personas, `type` `User` (no `Bot`). La lista vive en `.taskcode/` de la rama de codigo, bajo CODEOWNERS, nunca en `taskcode/tareas`.
- **«Ninguna IA aprueba ni mergea»** en tres capas: el revisor por permisos; el agente por `carril` (solo cuenta personas) y porque GitHub no deja aprobar el PR propio; el merge por `merge-pr`, que solo corre con la identidad de persona. Limite explicito: `contents:write` incluye el merge (D3).
- **GitLab** (RR-15): bot de proyecto con token de proyecto, solo como correspondencia, «no verificado»; lo cierran F1-13 y F6-1.

### Orden

1. PoC, secciones A–E (Plan de pruebas); las tablas de permisos del ADR salen de ahi.
2. ADR: tabla operacion → identidad, permisos por identidad, contrato del firmante, «Como se ve en GitHub», requisitos (P3, RR-04, RR-08, RR-12, RR-15, RN-01, RF-30, H5, H6; cada ID se comprueba antes de citarlo).
3. Fila del indice; `bash .github/scripts/comprobar-repo.sh` y `taskctl doctor`.
4. Borrar la carpeta temporal tras comprobar que no queda nada que conservar.

## Lo que el rol no cubrio

Solo arquitectura. Sin revisar:

- **Riesgos** (criterio 2): si una aprobacion de una App cuenta para `required_approving_review_count`; si `contents:write` del agente basta para `PUT /pulls/{n}/merge` con todo en verde; tags `v*` creados por el agente (abierto en F1-5); acceso del agente a las claves del revisor o de la persona en la misma maquina (F1-8).
- **Dominio**: quien lanza una release (F1-10); si con varias personas se exigen dos distintas; que hace la persona cuando una tarea legitima toca `.github/workflows/`, que el agente ya no puede empujar.
- **Testing**: E2E real de identidades en F2-6, F6-1, F9-4; aqui esquema, documentacion y firma local.
- **Sin verificar por el rol**: la ruta exacta de la tabla de permisos por endpoint en la documentacion publica y que `GET .../actions/permissions/workflow` responda en el repo privado gratuito.

## Riesgos aceptados y que los contiene

- **El agente tecnicamente puede mergear** con la aprobacion de la persona y los checks en verde (`contents:write` = push + merge): lo contienen los rulesets sin bypass, `carril` y `merge-pr` solo con la identidad de persona; la alternativa estricta (fork) esta en D3.
- **El agente puede aprobar PR ajenos** con `pull_requests:write`: lo contiene `carril`, obligatorio y que solo cuenta personas; que la plataforma no cuente esas aprobaciones queda «no verificado».
- **Evidencia sin GitHub real**: no se crea ninguna App; `app_id`, permisos efectivos y `verified` de un commit firmado quedan «no verificado» (F6-1, F2-6).
- **Firma local con clave desechable**: prueba el mecanismo (`git verify-commit`, trailer `Plan-Blob`), no la marca «Verified» de GitHub.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio). Secciones de `poc/poc-f1-6.sh` en Git Bash; toda llamada pasa por una funcion con `gh api --method GET` y el guion cuenta las de otro metodo (debe dar 0). Ninguno pesado.

- **Permisos minimos → 2**
  1. A: permisos de cada identidad validados contra el esquema `app-permissions` de la descripcion OpenAPI publica (nombres y valores `read`/`write`); un permiso inventado sale rechazado.
  2. B: con la tabla publica de permisos por endpoint de las Apps, endpoints que alcanza cada identidad: el revisor no alcanza escrituras de `contents`, `administration` ni `workflows`; el agente no alcanza `administration` (rulesets) ni `workflows`.
- **Que ninguna IA pueda aprobar ni mergear → 3**
  1. B: `POST /repos/{o}/{r}/pulls/{n}/reviews` (`event` `APPROVE`) y `PUT .../pulls/{n}/merge` exigen permisos que el revisor no tiene; para el agente se anota lo que alcanza (merge incluido) como limite documentado.
  2. C: `GET /repos/charliebk/{TaskCodeTwo,taskcodetwo-poc-f1-1}/actions/permissions/workflow` anota `can_approve_pull_request_reviews` (se espera `false`) y `GET /users/{login}` anota `type` (`User` frente a `Bot`).
  3. D: simulacion del contrato del firmante con casos fijos (persona verificada en la lista; Bot verificado; persona fuera de la lista; `web-flow`; sin firma): solo cuenta el primero; control negativo con una tabla alterada donde el revisor tiene `pull_requests:write`, que debe detectarse.
- **Cumple F1-6 y los requisitos que cita → 2**
  1. E: firma SSH local con clave desechable en la carpeta temporal: commit de `approve` con trailer `Plan-Blob`, `git verify-commit` correcto con `allowed_signers` y fallido con otra clave.
  2. Tabla requisito → decision → evidencia, fila del indice, `comprobar-repo.sh` en verde y `taskctl doctor` sin errores.

## Lo que necesita decision de una persona

Decisiones respondidas por la persona (Carlos) el 2026-11-14 con la consigna «responder la opcion recomendada a todas las preguntas»:

- **D1. Evidencia en GitHub** → **(a)** solo lectura (esquema y documentacion de permisos, `GET` en los dos repos). Descartadas (b) crear Apps de prueba y (c) cuenta de maquina.
- **D2. Forma de las identidades de IA** → **(a)** dos GitHub Apps, agente y revisor, con los permisos de arriba. Descartadas (b) cuentas de maquina con PAT y (c) una sola App (el agente podria publicar `revision-ia` en verde).
- **D3. Que significa «no mergear» para el agente** → **(a)** conserva `contents:write` (GitHub no separa push de merge); `merge-pr` solo con la identidad de persona; rulesets y `carril` impiden integrar sin su aprobacion; el ADR lo deja escrito como limite. Descartadas (b) fork (rompe el push directo a `taskcode/tareas`) y (c) solo cola (exige organizacion).
- **D4. Clave de firma de la persona** → **(a)** SSH `ed25519-sk` con toque fisico (FIDO); si la persona no tiene llave, **(b)** SSH o GPG con frase de paso y sin cache en el agente de claves. Descartada (c) `web-flow` (F1-5 no la acepta).
- **D5. Donde corre el revisor y su clave privada** → **(a)** en GitHub Actions, con la clave como secreto de entorno (el agente no tiene `workflows`). Descartada (b) en la maquina del agente. Detalle en F1-8.
- **D6. Lista de personas firmantes** → **(a)** en `.taskcode/` de la rama de codigo, bajo CODEOWNERS. Descartadas (b) en `taskcode/tareas` y (c) derivada de colaboradores admin/maintain.

Sin troceo: la duracion estimada no pasa de 60 min.

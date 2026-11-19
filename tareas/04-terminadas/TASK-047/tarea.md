---
id: TASK-047
titulo: "F6-1 Identidades: agente implementador"
tipo: feature
sprint: 6
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-047-f6-1-identidades-agente-implementador
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1029945
tokens_implementacion: 19539098
tokens_revision: 19057073
creado: 2026-11-14
actualizado: 2026-11-19
dependencias: [TASK-007]
---
## Objetivo

Identidades: agente implementador (commits y PR/MR) y revisor (reviews y checks; nunca aprobar ni mergear), como GitHub App y como usuario bot o token de proyecto en GitLab

Tarea F6-1 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-6.

## Criterios de aceptacion
- [x] La persona aprueba un PR/MR del agente en GitHub y en GitLab
- [x] Cumple lo que pide la tarea F6-1 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-19T08:52:15Z | plan | automatico | persona | 0.16.1 |
| 2026-11-19T09:08:21Z | approve | automatico | persona | 0.16.1 |
| 2026-11-19T09:08:26Z | start | automatico | persona | 0.16.1 |
| 2026-11-19T10:12:00Z | review | automatico | persona | 0.16.1 |
| 2026-11-19T10:56:36Z | review | automatico | persona | 0.16.1 |
| 2026-11-19T12:38:14Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-19T09:08:10Z | diseno | agente:a7e4a4f369272d49c | 34 | 79202 | 932595 | 18114 | 27 | 1029945 | - | - |
| 2026-11-19T10:12:00Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 26 | 14189 | 1875941 | 6450 | 1 | 1896606 | - | - |
| 2026-11-19T10:12:00Z | implementacion | agente:a2f3d88c7f20e826c | 160 | 286417 | 17266883 | 89032 | 15 | 17642492 | - | - |
| 2026-11-19T11:06:56Z | revision | agente:a27c470fa4d68e281 | 52 | 193438 | 3617792 | 33169 | 11 | 3844451 | 1 | rust-wasm-reviewer |
| 2026-11-19T11:06:56Z | revision | agente:a96ccee4dfbe4dd56 | 134 | 183991 | 9307406 | 29243 | 16 | 9520774 | 1 | code-quality-reviewer |
| 2026-11-19T11:06:56Z | revision | agente:ad59151acb5d6cda5 | 54 | 156027 | 3384422 | 29918 | 6 | 3570421 | 2 | code-quality-reviewer |
| 2026-11-19T11:06:56Z | revision | agente:a09d52f1c8eb9842f | 34 | 146847 | 1956490 | 18056 | 14 | 2121427 | 2 | rust-wasm-reviewer |

## Resultado

Identidad propia del agente implementador, sin modulo nuevo: el proveedor de credenciales por rol (ADR F1-6, decision 1) gana `CredencialesApp` (GitHub App: JWT RS256 con ring, instalacion por repo, token reducido al repo y a los permisos de la tabla 2 del ADR por rol, cache con margen), la fabrica `credenciales::de_entorno` (App sobre token, nunca paso entre roles; en GitLab la App cede al `TC2_TOKEN_<ROL>` del mismo rol) y el entorno git del agente (`Git::con_entorno_extra`: cabecera Basic limitada a la URL del remoto y autor `[bot]`, token nunca en argv ni en GIT_TRACE). Adenda F6-1 en `docs/adr/F1-6-identidades-agente-revisor-persona.md` con 6 divergencias del plan; fila F6-1 del plan con "checks".

Evidencia:
- Verificacion completa (`cargo xtask verificar --perfil completa`) en verde antes de la ronda 2; las correcciones de la ronda 1 con tests selectivos (`identidades` 12/12, `--lib plataforma::` 28/28, `--lib git::` 21/21, clippy). `finish` vuelve a verificar antes del merge.
- C1, GitHub real (2026-11-19): E2E `persona_aprueba_pr_del_agente` con la App `tc2-agente-charliebk` (App ID 5832411) instalada solo en `charliebk/taskcodetwo-e2e-github`: commit con autor `tc2-agente-charliebk[bot]`, PR #7 abierto por la App, GitHub rechaza la auto-aprobacion (422 "Can not approve your own pull request"), la persona aprueba y `estado-puertas` da `aprobaciones=1`. PR cerrado y rama borrada.
- C1, GitLab real (2026-11-19): E2E `persona_aprueba_mr_del_agente` con la cuenta de servicio del grupo `taskcodetwo-e2e` (`bot: true`, Developer) sobre `taskcodetwo-e2e/taskcodetwo-e2e-gitlab` (proyecto transferido desde el espacio personal: en GitLab.com Free las cuentas de servicio solo existen en grupos de primer nivel). GitLab SI deja al agente aprobar su propio MR (201); tc2 la cuenta como ignorada (`aprobaciones=0 ignoradas=1`) y la de la persona da `aprobaciones=1`. MR !10 cerrado y rama borrada.
- Suposiciones del plan verificadas: `RsaKeyPair::from_der` con PKCS#1, `access_tokens` con `repositories` y `permissions` (el E2E pide el token reducido), `bot: true` de la cuenta de servicio.

Revision por pares:
- Ronda 1: rust-wasm-reviewer aprobada-con-correcciones (MENOR-1..5); code-quality-reviewer cambios-solicitados (IMP-1: protecciones sin test que las mate; MEN-1..5). Todo corregido en 5f35211, con mutaciones.
- Ronda 2: las dos aprobadas.

Divergencias y pendientes:
- El objetivo nombra tambien al revisor: el proveedor ya lo soporta (`TC2_APP_REVISOR_*`, permisos de la tabla 2), pero crear su App, publicar `revision-ia` y ponerlo en `plataforma.revisor_id` y los rulesets es de F6-2/F6-3, como dice el plan.
- Firma de commits del agente: no se exige (D4, en la adenda).
- `identidad(Rol::Agente)` usa `GET /user`, que no vale con token de instalacion; hoy nadie lo llama asi (nota en la adenda).
- Notas de la ronda 2 sin abrir ronda: la causa `tls` no tiene test; `separar_config` ignora GIT_CONFIG_KEY/VALUE sin GIT_CONFIG_COUNT valido; con `con_entorno` acumulando, `GIT_CONFIG_COUNT=0` ya no anula las del proceso.
- En el grupo de GitLab queda otra cuenta de servicio sin uso (`service_account_group_70922564_...`); decision de Carlos.

## Duracion

- Estimada: 120 min
- Real (de la rama abierta al cierre con merge, sin pausas): 210 min
- Paso de la hora: 210 min reales frente a 120 estimados
- Cierre (finish): 1537 s, de 2026-11-19T12:12:37Z a 2026-11-19T12:38:14Z

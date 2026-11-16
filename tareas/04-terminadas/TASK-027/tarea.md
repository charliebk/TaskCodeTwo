---
id: TASK-027
titulo: "F2-12 taskctl actualizar --cli y aviso de version nueva"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-027-f2-12-taskctl-actualizar-cli-y-aviso-de
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: 53840927
tokens_revision: 20153780
creado: 2026-11-14
actualizado: 2026-11-16
dependencias: [TASK-025]
---
## Objetivo

`taskctl actualizar --cli` y aviso de versión nueva; el plugin del marketplace comprueba la versión del ejecutable

Tarea F2-12 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-10.

## Criterios de aceptacion
- [x] `tc2 actualizar --cli` sustituye una instalacion vX por vY servida en 127.0.0.1 (`TC2_BASE_URL`) sin reinstalar: despues `tc2 --version` da vY (test `crates/tc2/tests/actualizar_cli.rs`; job de ensayo en `.github/workflows/release.yml`, no verificado hasta el push)
- [x] Si el sha256 no casa con `SHA256SUMS`, sale `rechazada` (3) y el sha256 del binario instalado no cambia; no quedan `.descarga` ni `.old` (test)
- [x] `tc2 actualizar --cli --comprobar` da `actualizar ok actual=X disponible=Y` o `sin-cambios` sin modificar nada; sin red da `sinred` (4) con la URL (test)
- [x] `tc2 version --json` devuelve exactamente `{version, protocolo, target, sha256}` (ADR F1-14 punto 5; test)
- [x] `tc2 compatible --protocolo A-B` fuera de rango sale `rechazada` (3) con una linea `motivo=protocolo-incompatible arreglo="tc2 actualizar --cli"` (test, tambien en ingles)
- [x] Un `tc2` en la cache del plugin no se sustituye: `rechazada` con `motivo=gestionado-por-plugin` y el arreglo del plugin (test de `detectar_modo` con rutas de los tres sistemas)
- [x] `xtask` y `tc2` comparten la tabla de targets y el sha256 en `crates/distribucion`; `cargo xtask verificar` sigue en verde

## Autorrevision

- rust-wasm-reviewer — 94fd72e
- code-quality-reviewer — 94fd72e

Lo corregido en 94fd72e: JSON escrito dos veces (`version.rs` reutiliza
`contrato::cadena_json`), mensaje de `TC2_BASE_URL` que hablaba de «la API»,
literal en espanol en `sin_permiso`, y una descarga del binario que no
aguantaba un corte de red (un reintento solo ante `sinred`). Queda como
MENOR sin corregir: `descarga::Fallo::Escritura` da un informe generico con
`ruta=-` (en `actualizar` no se usa: alli se convierte en `sin-permiso` con
la ruta).

## Rondas extra

- Ronda 3: Carlos (consigna del 2026-11-16, «aceptamos todas las respuestas de forma recomendada»); IMP-2 de la ronda 2 es una regresion barata ya corregida y hay que verificarla.

## Resultado

Hecho: caja `crates/distribucion` (targets, sha256, `SHA256SUMS`, `PROTOCOLO`, `RELEASES`) compartida por `xtask` y `tc2` con los mismos sha256 del ensayo de release; `tc2 version --json`, `tc2 compatible`, `tc2 actualizar --cli [--comprobar] [--version X]` con descarga a hosts cerrados, sha256, attestation con `gh` si hay sesion, sustitucion por renombrados con vuelta atras y limpieza del `.old`; `cargo xtask cli-lock`; job `actualizar-ensayo` en `release.yml` y docs de distribucion. `cargo xtask verificar --perfil completa` en verde (implementacion) y `--perfil rapida` en verde tras las correcciones. Cada criterio con su test (ver `tests/actualizar_cli.rs`, `lib.rs`, `actualizar_cli.rs`, `distribucion`).

No verificado: el job `actualizar-ensayo` de CI (sin push; ensayado en local en win-x64 fuera del sandbox), redirecciones reales de GitHub (no hay release estable) y rutas reales de macOS y Linux (solo como cadenas).

Revision (code-quality-reviewer y rust-wasm-reviewer):
- Ronda 1: IMP-1 (attestation con `gh`) y MENORES corregidos en 39ef7a1 (deteccion de gestor, mensaje, test del `.old`, doc de `doctor`, `cambio=anterior`, reintento, disco lleno, limite de `version --json`).
- Ronda 2: IMP-2 (regresion: sin reintento antes de cabeceras) y MEN-6 rust (plazo con nietos) corregidos en 9312b13 y c8d4368, con tests y mutaciones en rojo.
- Ronda 3 (extra, decidida por la consigna de Carlos): aprobada por los dos.

MENORES aceptados sin corregir (van a una tarea de seguimiento):
- MEN-1 rust r1: un `.old` bloqueado por otro `tc2` en marcha da «Acceso denegado» (estado intacto, mensaje claro).
- MEN-5 rust r1: sin tests de integracion de redireccion no permitida ni limite de tamano (probados a mano por el revisor); el reintento si tiene test.
- MEN-5 calidad r1: `arreglo` es comando en `compatible` e identificador en el rechazo del plugin.
- MEN-7 rust r3: si un nieto retiene stdout con el hijo en exito, la salida se pierde y se espera el plazo; la doc de `ejecutar_con_limite` promete lo contrario.
- MEN-8 rust r3: un fallo de TLS en el handshake se clasifica transitorio y se reintenta una vez.
- MEN-9 rust r3: el timeout global de 120 s es transitorio; el peor caso dobla la espera.
- MENOR-1 calidad r3: el doc de `bajar_con_un_reintento` quedo encima de `PAUSA_REINTENTO` (linea duplicada).
- MENOR-2 calidad r3: mismo caso que MEN-7 visto desde la doc.
- Autorrevision: `descarga::Fallo::Escritura` con `ruta=-` (no se usa en `actualizar`).

Entorno: en esta maquina las descargas por 127.0.0.1 se cuelgan a veces ~19 s; el servidor de prueba de esta tarea lo evita no mandando FIN tras una respuesta completa. `tests/comun/http_mock.rs` (TASK-021) sigue con el problema y da `sinred` intermitente en `plataforma_gitlab`: va a la tarea de seguimiento.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-16T16:57:49Z | plan | automatico | persona | 0.16.1 |
| 2026-11-16T17:04:03Z | approve | automatico | persona | 0.16.1 |
| 2026-11-16T17:04:05Z | start | automatico | persona | 0.16.1 |
| 2026-11-16T18:25:07Z | review | automatico | persona | 0.16.1 |
| 2026-11-16T18:49:08Z | review | automatico | persona | 0.16.1 |
| 2026-11-16T19:34:50Z | review | automatico | persona | 0.16.1 |
| 2026-11-16T19:48:42Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-16T18:25:07Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 58 | 20188 | 8428850 | 6762 | 1 | 8455858 | - | - |
| 2026-11-16T18:25:07Z | implementacion | agente:a03e84fce229ca7a2 | 370 | 359244 | 44866127 | 159328 | 70 | 45385069 | - | - |
| 2026-11-16T18:38:00Z | revision | agente:acb5bf97db8f1ac96 | 42 | 133326 | 2259498 | 18875 | 4 | 2411741 | 1 | rust-wasm-reviewer |
| 2026-11-16T18:38:03Z | revision | agente:a3aeef36d07d9c34e | 62 | 158524 | 3735645 | 24248 | 5 | 3918479 | 1 | code-quality-reviewer |
| 2026-11-16T19:00:15Z | revision | agente:ac0bf82d104f1c73e | 62 | 166799 | 3740088 | 46868 | 6 | 3953817 | 2 | code-quality-reviewer |
| 2026-11-16T19:00:17Z | revision | agente:a678c163f4c332436 | 56 | 148516 | 3052927 | 44607 | 2 | 3246106 | 2 | rust-wasm-reviewer |
| 2026-11-16T19:46:26Z | revision | agente:a12ecf0fd484966f3 | 60 | 140707 | 3295583 | 26695 | 15 | 3463045 | 3 | code-quality-reviewer |
| 2026-11-16T19:46:26Z | revision | agente:a5d1f02d3619cad09 | 56 | 121837 | 2997364 | 41335 | 5 | 3160592 | 3 | rust-wasm-reviewer |

## Duracion

- Estimada: 240 min
- Real (de la rama abierta al cierre con merge, sin pausas): 165 min
- Paso de la hora: 165 min reales frente a 240 estimados
- Cierre (finish): 111 s, de 2026-11-16T19:46:51Z a 2026-11-16T19:48:42Z

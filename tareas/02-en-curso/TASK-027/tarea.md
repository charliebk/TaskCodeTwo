---
id: TASK-027
titulo: "F2-12 taskctl actualizar --cli y aviso de version nueva"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
rama: feature/task-027-f2-12-taskctl-actualizar-cli-y-aviso-de
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
actualizado: 2026-11-16
dependencias: [TASK-025]
---
## Objetivo

`taskctl actualizar --cli` y aviso de versión nueva; el plugin del marketplace comprueba la versión del ejecutable

Tarea F2-12 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-10.

## Criterios de aceptacion
- [ ] `tc2 actualizar --cli` sustituye una instalacion vX por vY servida en 127.0.0.1 (`TC2_BASE_URL`) sin reinstalar: despues `tc2 --version` da vY (test `crates/tc2/tests/actualizar_cli.rs`; job de ensayo en `.github/workflows/release.yml`, no verificado hasta el push)
- [ ] Si el sha256 no casa con `SHA256SUMS`, sale `rechazada` (3) y el sha256 del binario instalado no cambia; no quedan `.descarga` ni `.old` (test)
- [ ] `tc2 actualizar --cli --comprobar` da `actualizar ok actual=X disponible=Y` o `sin-cambios` sin modificar nada; sin red da `sinred` (4) con la URL (test)
- [ ] `tc2 version --json` devuelve exactamente `{version, protocolo, target, sha256}` (ADR F1-14 punto 5; test)
- [ ] `tc2 compatible --protocolo A-B` fuera de rango sale `rechazada` (3) con una linea `motivo=protocolo-incompatible arreglo="tc2 actualizar --cli"` (test, tambien en ingles)
- [ ] Un `tc2` en la cache del plugin no se sustituye: `rechazada` con `motivo=gestionado-por-plugin` y el arreglo del plugin (test de `detectar_modo` con rutas de los tres sistemas)
- [ ] `xtask` y `tc2` comparten la tabla de targets y el sha256 en `crates/distribucion`; `cargo xtask verificar` sigue en verde

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

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-16T16:57:49Z | plan | automatico | persona | 0.16.1 |
| 2026-11-16T17:04:03Z | approve | automatico | persona | 0.16.1 |
| 2026-11-16T17:04:05Z | start | automatico | persona | 0.16.1 |

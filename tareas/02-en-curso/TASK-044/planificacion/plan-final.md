# Plan — TASK-044: F5-4 informe de verificacion en JSON para el revisor IA

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 80 min
Senales: esquema JSON nuevo, lector y `seccion_revisor` en tc2, productor en xtask, workflow, plantilla y lint extendidos, guia; 2 criterios.

El comando del proyecto escribe un JSON opcional con sus pasos y su recuento en la ruta de `TC2_INFORME`. La plataforma solo lo transporta: un artefacto `informe-verificacion-<perfil>` por perfil y la linea `informe=` en el `output.summary` del check `verificacion` (F1-7 dec. 1: el workflow no conoce los pasos). El contrato es un esquema v1 versionado; tc2 tiene el lector y `seccion_revisor(&Informe)`, el texto con el que el revisor de F6-3 cita el informe y se compromete a no relanzar lo que ya paso en verde. `cargo xtask verificar` es el primer productor real.

Piezas y limites:
- `crates/tc2/esquemas/informe-verificacion.schema.json` (`version: 1`), precedente `config.schema.json`: `perfil`, `resultado`, `commit`, `inicio`, `fin`, `pasos[{nombre, resultado, segundos}]`, `tests{total, fallidos}` opcional (F1-11 practica 8).
- `crates/tc2/src/verificacion/informe.rs` (nuevo): `leer(&str) -> Result<Informe, String>` (serde, `deny_unknown_fields`) y `seccion_revisor(&Informe, artefacto: &str) -> String`. Debe decir que el hecho autoritativo para no repetir es el check `verificacion` de `github-actions` (app 15368); el JSON lo escribe codigo del PR, es detalle para citar y nunca decide.
- `xtask/src/main.rs` (`verificar`): registra cada paso (fmt, clippy, test, reproducible) con resultado y segundos; con `TC2_INFORME` escribe el JSON tambien en rojo, a mano (xtask no depende de tc2).
- `.github/workflows/verificacion.yml`: `ejecutar` recibe `TC2_INFORME=${{ runner.temp }}/informe-verificacion.json` y `actions/upload-artifact` por SHA con `if: always()` e `if-no-files-found: ignore`; el publicador solo anade `informe=informe-verificacion-<perfil>` al resumen.
- `plantillas/gitlab/verificacion.gitlab-ci.yml`: `TC2_INFORME: $CI_PROJECT_DIR/.tc2/informe-verificacion.json` y `artifacts: {when: always, paths: [...]}`.
- `xtask/src/workflows.rs`: `actions/upload-artifact` admitida por SHA solo en jobs sin escritura ni secretos.
- `docs/ci/verificacion-reutilizable.md`: seccion "Informe JSON (F5-4)" y divergencias. `revision/mod.rs::cuerpo_pr` no se toca (el cuerpo del PR lo controla el autor).

Orden de construccion:
1. Esquema v1, fixture `crates/tc2/tests/fixtures/informe-verificacion/xtask.json`, `leer` y `seccion_revisor`.
2. xtask escribe el informe; su test compara con el mismo fixture (contrato entre las dos cajas).
3. Workflow, plantilla y lint.
4. Guia y divergencias; validacion de la plantilla con la API de lint de GitLab (contenido en linea).

Alternativas descartadas:
- Sobre JSON escrito por el workflow: exige bash/jq en runners Windows o codigo por shell en GitLab, y esos datos ya estan en el check-run.
- `output.text` del check-run u `outputs` de job: en matriz se pisan, GitLab no tiene equivalente y el publicador leeria datos del PR. El artefacto existe en las dos plataformas (RR-15) y es el canal que F1-8 fija para el revisor.

## Troceo

No se trocea: Carlos pidio estas tareas sin trocear (2026-11-18).

## Lo que el rol no cubrio

- Casos limite (riesgos), estrategia de pruebas fina (testing) y semantica de los campos (dominio): sin revisar.
- "No repite tests": solo comprobable con el revisor de F6-3 (TASK-049). Divergencia explicita: TASK-049 debe usar `seccion_revisor` y demostrarlo.
- ToolsaWeb y OpenDockTrading no producen el informe hasta migrar (F9): su `.estado/verificar-ultima.json` tiene otra forma; hasta entonces solo existe el resumen del check.

## Riesgos aceptados y que los contiene

- El artefacto lo controla el codigo del PR: lo contiene que nunca decide (`seccion_revisor`) y F6-7 (prompt injection).
- Sin informe, el revisor solo tiene el resumen del check: el perfil en verde ya basta para no relanzar la suite.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- **C1 (el revisor lo cita):** (1) xtask: el informe de una ejecucion con pasos fijos (verde y rojo) es igual al fixture; (2) tc2: el fixture se lee y `seccion_revisor` contiene artefacto, perfil, pasos y la regla; un campo desconocido o `version` distinta de 1 es error.
- **C2 (F5-4 y lo que cita):** (1) `workflows.rs`: subida con `if: always()` en `ejecutar`, `informe=` en el publicador, `artifacts: when: always` en la plantilla y la accion nueva rechazada en un job con escritura; (2) API `ci/lint` de GitLab con la plantilla en linea: valida.

## Lo que necesita decision de una persona

Consigna de Carlos (2026-11-18): la opcion recomendada y sin trocear.

- **D1. Quien escribe.** Elegida: el comando del proyecto, en `TC2_INFORME`; la plataforma transporta.
- **D2. Transporte.** Elegida: un artefacto por perfil + `informe=` en el resumen.
- **D3. Lista blanca de F1-8.** Elegida: admitir `actions/upload-artifact` por SHA solo en la clase ejecuta; divergencia en la guia (decia que la unica accion admitida es `actions/checkout`).
- **D4. Contrato.** Elegida: esquema + lector + `seccion_revisor` en tc2.
- **D5. Recuento.** Elegida: `tests` opcional; xtask lo omite.
- **D6. E2E.** Elegida por quien orquesta: sin push en esta tarea; validacion local y API de lint de GitLab con la plantilla en linea. La ejecucion real en GitHub (artefacto presente, `informe=` en el resumen) se hace en TASK-049, que consume el informe. Divergencia anotada.

Suposiciones por verificar:
- `artifacts:` y la variable con `$CI_PROJECT_DIR` pasan el lint propio y la API `ci/lint`.
- El revisor de F6-3 puede bajar el artefacto de otro run con `actions: read` (F1-8: no verificado): TASK-049.

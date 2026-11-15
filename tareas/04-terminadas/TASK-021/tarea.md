---
id: TASK-021
titulo: "F2-6 Adaptador GitHub"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-021-f2-6-adaptador-github
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1588800
tokens_implementacion: 11346466
tokens_revision: 19231090
creado: 2026-11-14
actualizado: 2026-11-15
dependencias: [TASK-015, TASK-017]
---
## Objetivo

Adaptador GitHub (PR, checks, reviews, identidad) sobre la API, con errores dirigidos a la persona

Tarea F2-6 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-13, F2-2.

## Criterios de aceptacion
- [x] E2E contra un repo de pruebas en GitHub
- [x] Cumple lo que pide la tarea F2-6 del plan y los requisitos que cita

## Resultado

**Qué se hizo.** Módulo nuevo `crates/tc2/src/plataforma/`, solo en la biblioteca: el binario no gana subcomandos hasta F6-2.
- `mod.rs`: el trait del puerto `Plataforma` (D1a de F1-13), con la lista cerrada `abrir_pr`, `estado_puertas`, `merge_pr`, `leer_puertas`, `leer_permisos_ci`, `firma_commit` e `identidad`. Tipos neutrales: `Personas` por id, `Usuario::es_persona`, `Firma::cuenta_como_aprobacion`, `ModeloPuertas` y `ReglasLeidas`.
- `http.rs`: envoltorio único sobre `ureq` 3.5 con TLS del almacén del sistema.
  - Métodos cerrados por tipo (GET, POST, PUT).
  - Base fija (https; `http://127.0.0.1` solo para tests) y ruta validada antes de la red: sin `.`, `..`, `%2e`, `%25` ni `\`.
  - Sin redirecciones, con límite de 1 MiB (comprueba el `Content-Length` antes de leer) y registro `METODO ruta`.
- `github.rs`: el adaptador.
  - Cada endpoint es una variante de un `enum`.
  - `abrir-pr` es idempotente, también con el 422 «already exists» de una carrera.
  - `merge-pr` exige el sha evaluado (`head-cambiado`, sin PUT).
  - `estado-puertas` hace 4 lecturas y la misma línea que la PoC M: checks por `integration_id` (G09); última review que decide por id (`COMMENTED` no retira una aprobación); `neutral` y `skipped` cuentan como verdes; `behind` y `dirty` salen como `falta=al-dia` y `falta=sin-conflictos`.
  - Paginación por `Link rel=next` solo hacia la misma base, con tope de 10 páginas.
- `puertas.rs`: `evaluar`, puro y común con F2-9.
- `errores.rs`: de HTTP a la línea de contrato, con el arreglo en stderr. El mensaje de la API va saneado y el token no sale nunca.
  - 401 → `denegada motivo=credenciales` (8);
  - 403 con aviso de plan → `rechazada motivo=plan` (3);
  - límite primario y secundario → `agotada` (5) con su espera;
  - 403 → `denegada motivo=permisos` (8); 404 → `denegada motivo=no-existe-o-sin-acceso` (8);
  - 405 y 409 en el merge → `rechazada` (3); 422 sin commits → `rechazada motivo=sin-commits` (3);
  - red → `sinred` (4); TLS, redirección o respuesta inválida → `error` (1).
- `credenciales.rs`: un token por rol (`TC2_TOKEN_AGENTE`, `TC2_TOKEN_REVISOR` y `TC2_TOKEN_PERSONA`, este último solo con TTY), con `Secreto` enmascarado. Sin el token del rol, `rechazada motivo=identidad falta=<rol>` y nunca se prueba con otro. `merge-pr`, `leer-permisos-ci` y `estado-puertas` como persona exigen que su `id` esté en `Personas`.
- `deteccion.rs`: plataforma por el host de `origin` (decisión 7 de F1-13). Con `github.com` o `gitlab.com`, la clave de configuración no puede cambiar la API; otro host exige la clave.
- Dependencias: `ureq` (`rustls`, `platform-verifier`) y `serde_json`. El build sigue reproducible y el binario no cambia (mismo sha256, 464 384 bytes), porque `tc2` todavía no usa el adaptador.

**Evidencia.**
- `cargo xtask verificar --perfil completa` en verde, con build reproducible.
- `tests/plataforma.rs`, 4 tests contra un servidor simulado en 127.0.0.1, sin red y dentro de la verificación:
  - la batería de los 6 casos de la PoC M (misma línea, mismo código y 4/4/4/4/2/1 llamadas) y la idempotencia de `abrir-pr` y `merge-pr`;
  - el envoltorio, que rechaza antes de la red;
  - la tabla de errores, sin que el token aparezca;
  - las protecciones: G09, G16, `web-flow` y Bot en la lista, contrato del firmante, aviso de plan en `leer-puertas`, bordes y tope de páginas.
- Fixtures de la PoC M copiados a `tests/fixtures/plataforma/github/`. Las formas reales observadas en el E2E se añadieron a `observado.json` (E2E-01 a 05).
- **E2E** (`tests/plataforma_e2e.rs`, `--ignored`) contra el repo público `charliebk/taskcodetwo-e2e-github`, ejecutado tres veces en verde. La salida de la última está en `evidencia/e2e.log`.
  - Recorrido: abrir-pr ok y luego sin-cambios con el mismo número; estado-puertas esperando; merge con head distinto rechazado; merge ok y luego sin-cambios.
  - El merge commit lo firma `web-flow` (verificado) y no cuenta como aprobación.
  - `leer-permisos-ci ok ci-aprueba=false`; rama sin commits → `sin-commits`; token falso → 401 dirigido; repo inexistente → 404 dirigido.
  - Puertas reales con un ruleset de 1 aprobación en `develop`: `estado-puertas esperando falta=aprobacion`; el autor no puede aprobar su PR (422); **GitHub bloquea el merge** (405) → `merge-pr rechazada motivo=no-mergeable`. Las dos últimas eran documentales en F1-13 y F1-6.
  - El arnés deja el repo limpio aunque falle a medias (`Drop`).

**Revisión.** Dos rondas, con `rust-wasm-reviewer` y `code-quality-reviewer`.
- **Ronda 1** (cambios solicitados en los dos):
  - IMP-1 de los dos: el test del cuerpo de más de 1 MiB era intermitente (`sinred` en vez de `respuesta-invalida`). Ahora se comprueba el `Content-Length` antes de leer; 0 fallos en 15 repeticiones, y 10 de 10 en las de cada revisor.
  - IMP-2 de rust: `detectar` aceptaba otra URL de API para `github.com`, y los tokens habrían ido a otro host. Ahora falla cerrado.
  - IMP-3 y IMP-4 de rust e IMP-2 de calidad: G09, `es_persona`, G16, la firma y `leer-puertas` no tenían ningún test offline que se pusiera rojo. Los cubre el test nuevo de protecciones; los 8 mutantes mueren.
  - 11 MENORES corregidos:
    - una review `COMMENTED` retiraba una aprobación;
    - `neutral` y `skipped` salían como rojo;
    - faltaban tests de `behind`, `pr-cerrado`, `sha-invalido`, límite con reinicio y tope de páginas;
    - el límite secundario sin cabeceras;
    - `dirty` se daba por bueno: ahora sale `falta=sin-conflictos`;
    - un merge 2xx sin sha salía como `error`;
    - `Base` admitía `?` y puertos fuera de rango;
    - el E2E no limpiaba si fallaba.
- **Ronda 2** (verificación acotada): aprobada por los dos, sin hallazgos nuevos.

**Divergencias con el plan y con los ADR.**
- `aplicar-puertas`, y la comparación de las reglas leídas con el modelo, pasan a F3-1 y F3-8; las Consecuencias de F1-5 las daban a F2-6. El E2E crea el ruleset desde su arnés.
- `leer-permisos-actions` (F1-6) se llama `leer-permisos-ci`, como propone la divergencia 2 de F1-13.
- 401, 403 y 404 van a `denegada` (8), que F1-4 no tenía y añadió F2-3.
- `Personas` se pasa al construir el adaptador y no en cada llamada, como decía el plan; `estado_puertas` y `leer_puertas` reciben el rol (agente o persona).
- La línea nueva `falta=sin-conflictos` no estaba en la PoC M. F2-9 tendrá que dar la misma con `has_conflicts`.
- Una sola cuenta hace de agente y de persona en el E2E (decisión 2 del plan). La separación real llega con las Apps de F6-1, igual que `identidad` con un token de App (`GET /user` no vale para Apps).
- **Riesgo aceptado**: el token de persona en una variable de entorno lo puede leer el agente (A5 de F1-8). El TTY obligatorio solo lo frena en parte; se cierra en F6-1.
- **Sin verificar**:
  - descartar aprobaciones viejas, aprobación del último push, cola y force-push, que necesitan una segunda cuenta y pasan a F6-1 y F9-4;
  - un cuerpo de más de 1 MiB sin `Content-Length` (chunked), que solo lo probó un revisor a mano;
  - `mergeable_state` `unknown`, que se da por bueno y lo vuelve a comprobar el 405 del merge.
- El repo de pruebas `charliebk/taskcodetwo-e2e-github` queda creado y limpio (solo `main` y `develop`). Lo borra Carlos cuando quiera, según su decisión del 2026-11-15; lo pueden reutilizar F3-1 y F6-2.
- Se planificó con 180 min, sin trocear por decisión de Carlos para el H2.

**Tras el cierre.** CI de `develop` en 830de9b (run 43745445064): `comprobar-repo` y `codigo` en ubuntu, macOS y Windows, los 4 jobs en success.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-15T16:50:45Z | plan | automatico | persona | 0.16.1 |
| 2026-11-15T16:58:01Z | approve | automatico | persona | 0.16.1 |
| 2026-11-15T16:58:09Z | start | automatico | persona | 0.16.1 |
| 2026-11-15T17:23:03Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T17:58:36Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T18:12:30Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-15T16:58:01Z | diseno | agente:a14748c38791202ec | 44 | 98344 | 1467204 | 23208 | 33 | 1588800 | - | - |
| 2026-11-15T17:23:03Z | implementacion | sesion:0c02a589-064f-41b2-898e-aecc1141ae21 | 88 | 111688 | 11159050 | 75640 | 8 | 11346466 | - | - |
| 2026-11-15T17:45:39Z | revision | agente:aea0aea00fe5fdb3c | 80 | 196363 | 6112208 | 39711 | 9 | 6348362 | 1 | rust-wasm-reviewer |
| 2026-11-15T17:45:39Z | revision | agente:adc5fa1c079dffb72 | 84 | 206486 | 6530530 | 34463 | 14 | 6771563 | 1 | code-quality-reviewer |
| 2026-11-15T18:09:12Z | revision | agente:ae18fa2d07514a8ff | 48 | 158361 | 2959032 | 28198 | 9 | 3145639 | 2 | rust-wasm-reviewer |
| 2026-11-15T18:09:12Z | revision | agente:a9e675f854847c499 | 54 | 137926 | 2809120 | 18426 | 6 | 2965526 | 2 | code-quality-reviewer |

## Autorrevision

- rust-wasm-reviewer — cd3479a
- code-quality-reviewer — cd3479a
- rust-wasm-reviewer — 229caed
- code-quality-reviewer — 229caed

## Duracion

- Estimada: 180 min
- Real (de la rama abierta al cierre con merge, sin pausas): 74 min
- Paso de la hora: 74 min reales frente a 180 estimados
- Cierre (finish): 143 s, de 2026-11-15T18:10:07Z a 2026-11-15T18:12:30Z

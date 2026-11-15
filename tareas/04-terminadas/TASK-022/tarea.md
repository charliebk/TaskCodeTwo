---
id: TASK-022
titulo: "F2-9 Adaptador GitLab"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-022-f2-9-adaptador-gitlab
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1000511
tokens_implementacion: 29577880
tokens_revision: 26445997
creado: 2026-11-14
actualizado: 2026-11-15
dependencias: [TASK-015, TASK-017]
---
## Objetivo

Adaptador GitLab (MR, pipelines, approvals, identidad) con la misma interfaz y la misma batería E2E (RR-15)

Tarea F2-9 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-13, F2-2.

## Criterios de aceptacion
- [x] E2E contra un proyecto de pruebas en GitLab
- [x] Cumple lo que pide la tarea F2-9 del plan y los requisitos que cita

## Resultado

**Qué se hizo.** Adaptador GitLab del puerto `Plataforma`, solo en la biblioteca: el binario no gana subcomandos.
- `gitlab.rs`: el mismo trait que GitHub sobre la API REST v4, sin `glab`.
  - Endpoints en un `enum` cerrado. El proyecto va como id numérico o `grupo%2Fproyecto`, codificado una sola vez.
  - `estado-puertas` hace 5 lecturas: MR con divergencia, reglas de aprobación, rama protegida, estados del commit (`all=true`, paginados) y aprobaciones. Da la misma línea que GitHub (RR-15).
    - Checks por `author.id` (sustituto de G09); el último estado por id. Solo `success` es verde; `skipped` y `manual` quedan pendientes (falla cerrado, como la PoC M).
    - G01: rama destino protegida, merge solo del rol Maintainer (sin niveles de usuario ni grupo) y sin force-push. Si no se cumple, `rechazada motivo=config garantia=G01`.
    - Rama al día por `diverged_commits_count`. Sin conflictos por `has_conflicts` y `detailed_merge_status` (si GitLab aún comprueba, no se da por libre).
    - La aprobación del autor del MR no cuenta.
  - `abrir-pr`:
    - idempotente;
    - con una carrera (409) relee y da el MR que abrió el otro;
    - antes de crear comprueba los commits nuevos (`ref_name=base..rama`), porque GitLab abre MR vacíos: da `sin-commits`, igual que GitHub.
  - `merge-pr` exige persona de la lista y el sha evaluado (`head-cambiado`, sin PUT). Un 401 en el merge, con el token ya validado, sale como `denegada motivo=permisos`.
  - `firma_commit`: contrato del firmante por clave (decisión 6 de F1-13), con `Clave::Ssh`/`Clave::Gpg` separadas y `verified_system` sin contar.
  - `leer-puertas`, `leer-permisos-ci` (G16: el job token es fijo, `fuente=plataforma`) e `identidad` (`Bot` si `bot: true`).
- `conexion.rs` (nuevo): el transporte que antes era privado de `GitHub` (token por rol, `pedir`, `leer`, paginación con tope, caché de identidades, `exigir_persona`, validadores), compartido por los dos adaptadores.
- `http.rs`:
  - `Cliente::para(base, Proveedor)` con cabeceras por plataforma;
  - `%2F` admitido solo con GitLab y solo exacto, con los segmentos comprobados ya decodificados;
  - prefijo en la base de `127.0.0.1`;
  - cabeceras `RateLimit-*` de GitLab.
- `errores.rs`: nombre de la plataforma en el diagnóstico. 406 y 400 solo para GitLab; el aviso de plan, solo para GitHub.
- `mod.rs`: `Proveedor` pasa al núcleo, `Personas::con_claves`, `Firma.clave` y `Clave`.

**Evidencia.**
- `cargo xtask verificar --perfil completa` en verde, con build reproducible: el mismo sha256 en las dos rutas (`aa2f4253…`, 464 384 bytes).
- `tests/plataforma_gitlab.rs`, 3 tests contra el mock:
  - batería compartida: los 6 casos de la PoC M más `al-dia` y `sin-conflictos`, misma línea y mismo código en GitHub y GitLab, con 4 y 5 llamadas; idempotencia de `abrir-pr` y `merge-pr`;
  - envoltorio y tabla de errores de GitLab, sin que el token aparezca;
  - protecciones.
- Los tests de F2-6 (`tests/plataforma.rs`) no cambian lo que esperan tras el refactor.
- Mutantes: los de las protecciones nuevas mueren (autor del MR, reglas de aprobación, `exigir_persona` en el merge, `detailed_merge_status`). 9 de 10 y 22 de 28 en las rondas 1 de los revisores.
- **E2E** (`tests/plataforma_gitlab_e2e.rs`, `--ignored`) contra el proyecto público `charlie.bk/taskcodetwo-e2e-gitlab`, creado con permiso de Carlos.
  - Verde en 3 de 4 ejecuciones. La otra la cortó un 429 de gitlab.com en `GET /user` antes de probar nada.
  - La última va en `evidencia/e2e-gitlab.log`, sin el token (comprobado con grep).
  - Recorrido: abrir-pr ok y sin-cambios; G01 sin proteger; `falta=verificacion,aprobacion` y luego `falta=aprobacion`; estado de otro origen no cuenta; autoaprobación observada (201) e ignorada; head-cambiado; merge ok y sin-cambios; firma del merge sin firmar; identidad; permisos; reglas; sin-commits; 401 y 404 dirigidos.
  - Puertas reales: conflicto → `falta=aprobacion,al-dia,sin-conflictos` y merge rechazado por GitLab (405) → `no-mergeable`; rama por detrás → `falta=aprobacion,al-dia`.
  - Formas observadas en `tests/fixtures/plataforma/gitlab/observado-e2e.json` (E2E-GL-01 a 09).

**Hallazgos del E2E que cambiaron el código.**
- E2E-GL-01: `approval_rules` responde **200 con `[]` en Free**, no el 403 que suponían F1-13 y la PoC M. Con una garantía Premium exigida, ahora se pide una regla con al menos una aprobación (falla cerrado: `motivo=config`; el 403 sigue siendo `motivo=plan`).
- E2E-GL-02: en Free **el autor puede aprobar su propio MR**. Su aprobación ya no cuenta, como en GitHub.

**Revisión.** Dos rondas, con `rust-wasm-reviewer` y `code-quality-reviewer`.
- **Ronda 1** (cambios solicitados en los dos):
  - IMP-1 de calidad: con `detailed_merge_status=checking`, `estado-puertas` daba `ok`. Corregido.
  - IMP-1 de Rust: ningún test cubría `exigir_persona` en `merge-pr` de GitLab. Corregido.
  - MENORES corregidos: 401 del merge como permisos, `all=true` en estados, claves SSH y GPG separadas, G01 con niveles de usuario o grupo, test de G01 con dos niveles.
  - MEN-2 de Rust aceptado: un 403 de permisos en `approval_rules` sale como `motivo=plan`. Con una sola cuenta no se distingue, falla cerrado igual, y lo afinan F6-1 y F3-8.
- **Ronda 2** (verificación acotada): aprobada por los dos, sin hallazgos nuevos.
  - Nota de los dos: el test de G01 con dos niveles usa uno de 30, que ya falla por otra cláusula, así que el mutante que quita `len() == 1` sigue vivo. Hace falta un caso con dos niveles de 40 (pendiente; el MEN-4 de calidad quedó marcado como corregido sin matarlo).
  - Nota de los dos: un token que caducara entre `GET /user` y el PUT del merge se diagnosticaría como permisos.

**Divergencias con el plan y con los ADR.**
- `abrir-pr` comprueba los commits con `repository/commits?ref_name=base..rama&per_page=1` y no con `compare`, que trae los diffs y podía pasar de 1 MiB.
- `motivo=config` es nuevo en `estado-puertas` (G01 y, en Free, las garantías Premium); GitHub no lo da.
- `leer-puertas` lee 2 endpoints y no 5. En GitLab `plan_da_reglas=true` no prueba Premium (E2E-GL-01): lo afina F3-8.
- `skipped` es verde en GitHub y pendiente en GitLab (como la PoC M). Es decisión de dominio pendiente.
- El sha256 del binario no es el de TASK-021 (`76ac4677…`), con el mismo tamaño, aunque `tc2` no usa el adaptador. No se ha averiguado por qué; el build sigue siendo reproducible.
- **Sin verificar**:
  - un estado publicado por otra cuenta real y la cola, que necesitan una segunda identidad (F6-1, F9-4);
  - el 403 de plan con token en un self-managed;
  - el job token de un pipeline real (sin runners).
- **Tests intermitentes en esta máquina**: los primeros `connect` a 127.0.0.1 de cada proceso de test tardan 10-20 s (también con sockets crudos) y a veces dan `sinred` en `tests/plataforma.rs`. Ya pasaba en `develop` antes de esta tarea (comprobado en un worktree aparte); la CI de TASK-021 dio verde. No es del código.
- El proyecto de pruebas queda limpio (solo `main` y `develop`). Lo borra Carlos cuando quiera.
- Se planificó con 150 min, sin trocear por decisión de Carlos para el H2.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-15T18:32:06Z | plan | automatico | persona | 0.16.1 |
| 2026-11-15T18:39:10Z | approve | automatico | persona | 0.16.1 |
| 2026-11-15T18:39:11Z | start | automatico | persona | 0.16.1 |
| 2026-11-15T20:10:44Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T20:36:40Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T21:05:02Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-15T18:39:10Z | diseno | agente:ad130294f602facaa | 28 | 109657 | 864223 | 26603 | 24 | 1000511 | - | - |
| 2026-11-15T20:10:44Z | implementacion | sesion:0c02a589-064f-41b2-898e-aecc1141ae21 | 204 | 200486 | 29250403 | 126787 | 25 | 29577880 | - | - |
| 2026-11-15T20:36:23Z | revision | agente:a2443859140e2ce0f | 86 | 200966 | 6941326 | 28593 | 14 | 7170971 | 1 | rust-wasm-reviewer |
| 2026-11-15T20:36:23Z | revision | agente:a11f01920ad1f8ca1 | 74 | 173096 | 4948337 | 20690 | 7 | 5142197 | 1 | code-quality-reviewer |
| 2026-11-15T20:54:20Z | revision | agente:a3e36ac08b3259352 | 160 | 154609 | 10006468 | 25216 | 18 | 10186453 | 2 | rust-wasm-reviewer |
| 2026-11-15T20:54:20Z | revision | agente:ab9150263a0ab3016 | 74 | 133961 | 3794863 | 17478 | 10 | 3946376 | 2 | code-quality-reviewer |

## Autorrevision

- rust-wasm-reviewer — 904be48
- code-quality-reviewer — 904be48
- rust-wasm-reviewer — 7b4697f
- code-quality-reviewer — 7b4697f

## Duracion

- Estimada: 150 min
- Real (de la rama abierta al cierre con merge, sin pausas): 146 min
- Paso de la hora: 146 min reales frente a 150 estimados
- Cierre (finish): 385 s, de 2026-11-15T20:58:37Z a 2026-11-15T21:05:02Z

**Tras el cierre.** CI de develop en verde: run 43761739663, los 4 jobs en success (comprobar-repo, codigo en ubuntu, windows y macos). El primer `taskctl finish` cayo en la verificacion por un `sinred` del atasco de red local; el segundo paso entero.

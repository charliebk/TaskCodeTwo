# Plan — TASK-139: Auditoria 4D: A1 plataforma e identidad (11 IMPORTANTE)

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 300 min

Señales: unos 9 ficheros (`github.rs`, `gitlab.rs`, `http.rs`, un módulo neutral nuevo en `src/plataforma/`, `tests/comun/adaptadores.rs` nuevo, `tests/plataforma.rs`, `tests/plataforma_gitlab.rs`, `tests/prs_abiertos.rs`, los dos E2E y un arnés E2E nuevo); dos capas (adaptadores y soporte de tests); 8 criterios; 5 fichas D4 que se demuestran con mutantes; la verificación rápida ya tarda unos 40 min.

**Estado actual (develop 503679c):** ninguna de las 11 fichas está resuelta; TASK-148 no cubre A1-04, A1-05 ni A1-06 (`Parcial`/`Doble` doblan el trait, no prueban adaptadores reales). Siguen las copias de `adaptador`/`con_personas` (`tests/plataforma.rs:32`, `:568`), la tabla de errores (`:468`, `plataforma_gitlab.rs:476`), el servido de fixtures (`plataforma.rs:116`, `plataforma_gitlab.rs:82`) y `Arnes::nuevo`/`intentar*` de los E2E. Hay además copias del constructor en `prs_abiertos.rs:23-53` y `plataforma_gitlab.rs:25-55`, que se recogen en el mismo ayudante.

**Forma.** La lógica neutral sale de los adaptadores a `src/plataforma/operaciones.rs` (`pub(crate)`); cada adaptador solo lee/escribe su API y traduce a un tipo neutral (F1-13).
- A1-02: `abrir_pr` neutral (validación, idempotencia, carrera, línea única) sobre un trait `pub(crate) AbrePr { abierto, antes_de_crear, crear -> Creado::{Pr, YaExiste, Fallo} }`; cada adaptador decide qué es «ya existe» (GitHub 422 `already exists`, GitLab 409); lo neutral emite la misma línea en los dos (decisión 1); el diagnóstico sigue siendo de cada plataforma. `merge-pr` (decisión 2): `entrada_merge` y `merge_segun_estado`, con `puertas::mergeado`/`pr_cerrado` parametrizadas por `op`.
- A1-01: `leer_en_lote(rol, ramas, checks_de, comun, por_rama)` lee lo común perezosamente (desaparece el brazo `None` inalcanzable); `leer_puertas` de cada adaptador es un lote de una rama; una sola `validar_puertas`.
- A1-03: cada adaptador mapea a `PrLeido { numero, rama, sha, borrador, propio }` y `prs_propios` aplica las reglas de la adenda (rechaza al revisor, salta forks y ramas inválidas, `respuesta-invalida` con sha no hex, minúsculas, orden).
- A1-07: `rulesets` se funde con `rulesets_de` (que ya pagina con `leer_paginas`); el 403 de plan vuelve a `Ok(None)` en un solo sitio (decisión 4).
- A1-04 y A1-05: `tests/comun/adaptadores.rs` con `credenciales`, `github`, `gitlab`, `servir(&fixture) -> Mock` y `tabla_de_errores` genérica; lo usan `plataforma.rs`, `plataforma_gitlab.rs` y `prs_abiertos.rs` (el token por parámetro, porque cada batería comprueba que el suyo no se filtra).
- A1-06: `tests/arnes_e2e/mod.rs` (precedente `tests/agente_git/mod.rs`) con `Cliente::nuevo`, `intentar_con`, `sufijo`, `ver`; solo lo incluyen los dos E2E; cada E2E conserva su `Arnes` (F2-9).
- A1-08 a A1-11: casos nuevos en los tests nombrados.

**Orden:** 1) tests D4 contra el código sin refactorizar (A1-09, A1-10, A1-08, A1-11), con el rojo del diff literal de cada ficha; 2) A1-02 (test de la carrera, `operaciones.rs`, `abrir_pr`, luego `merge-pr`); 3) A1-03 y A1-01; 4) A1-07 (test de 100 rulesets + página 2); 5) A1-05, A1-04 y A1-06; luego `cargo xtask verificar`. Un commit por ficha.

## Reutilizacion (Regla 4D)

Lo que ya existe y se usa (helpers, funciones, fixtures, tests), buscado en el repo antes de disenar; y lo que se crea nuevo, con el motivo de no reutilizar:

- `conexion.rs` (`rama_valida`, `sha_valido`, `uso`, `rol_no_valido`, `head_cambiado`, `Conexion::{leer_paginas, invalida_por, traducir, exigir_persona}`): precedente de funciones neutrales libres, se reutiliza tal cual.
- `errores::traducir`/`es_aviso_de_plan`/`mensaje_api` (A1-07).
- `GitHub::rulesets_de` (absorbe a `rulesets`).
- `puertas::mergeado`/`pr_cerrado`, parametrizadas por `op` en vez de crear hermanas.
- `tests/comun/http_mock::{Mock, Simulada::con, serie}` (cabecera `Link`, 403 con `retry-after`).
- Nuevo: `src/plataforma/operaciones.rs` (no mezclar flujo con el transporte de `conexion.rs` ni con la evaluación de `puertas.rs`; se actualiza la doc de `mod.rs:7-8`); `tests/comun/adaptadores.rs` (`comun/plataforma.rs` es de los dobles); `tests/arnes_e2e/mod.rs` (fuera de `comun` para no compilarlo en ~30 binarios).

## Lo que el rol no cubrio

- Riesgos (no lanzado): que el refactor cambie el orden de llamadas al mock que fijan recuentos (p. ej. `bateria_de_la_poc_m_e_idempotencia`); el 403 de plan que además es límite.
- Dominio (no lanzado): la línea de la carrera (decisión 1) es contrato público; `mr-existente` solo aparece en `gitlab.rs:606` y en `plataforma_gitlab.rs:540`.
- Fuera de alcance: constructores `GitHub::nuevo`/`GitLab::nuevo` en `tests/doctor.rs`, `identidad.rs`, `identidades.rs` (posible D1 de otro grupo); paginación de `approval_rules` en GitLab; MENOR de A1-12.

## Riesgos aceptados y que los contiene

- Los mutantes de A1-09 y A1-10 cambian de sitio al refactorizar: los tests van antes; rojo con el diff literal sobre el código previo y con el mutante equivalente en `operaciones.rs` (decisión 5).
- A1-07: un 403 de plan que además es límite pasa de `None` a `agotada` (más fiel; decisión 4).
- Una abstracción más (`AbrePr`, `pub(crate)`, dos implementaciones fijas).
- `tabla_de_errores` genérica: el mensaje del assert lleva la línea esperada.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- A1-02: `carrera_de_abrir_pr_da_la_misma_linea` (GitHub 422 y GitLab 409 con relectura vacía: misma línea y código byte a byte); se adapta `plataforma_gitlab.rs:540`. Ligero.
- A1-01 y A1-03: sin tests nuevos; los protegen A1-09, A1-10 y las baterías existentes.
- A1-04, A1-05, A1-06: sin tests nuevos; baterías en verde y `cargo test -p tc2 --test plataforma_e2e --no-run` / `--test plataforma_gitlab_e2e --no-run`.
- A1-07: `leer_puertas_pagina_los_rulesets` (100 rulesets con `Link rel=next`, la página 2 trae `taskcodetwo-develop`). Ligero.
- A1-08: `https://.evil.example` y `https://-x.example` en `bases_validas_e_invalidas`.
- A1-11: 403 con `retry-after: 30` sin `x-ratelimit-remaining` → `identidad agotada motivo=limite-peticiones reintenta-en=30`.
- A1-09: `sha_invalido_falla_y_rama_rara_se_salta` recorre GitHub y GitLab; `misma_lista…` con un sha en mayúsculas.
- A1-10: token de revisor en `protecciones_de_gitlab` y `protecciones_y_bordes` → `leer-puertas rechazada motivo=identidad rol=revisor`.
- Pesado: `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar`; mutantes limitados al binario nombrado, sin solapar.

## Lo que necesita decision de una persona

1. Línea única de la carrera de `abrir-pr` (RR-15): (a) `abrir-pr conflicto motivo=pr-existente` (código 6) en las dos; (b) `mr-existente`; (c) `error motivo=validacion`. Recomendado (a) (cambia la línea pública de GitLab).
2. Incluir la parte neutral de `merge-pr` en A1-02. Recomendado: sí.
3. Funciones neutrales en `plataforma/operaciones.rs`. Recomendado.
4. A1-07: 403 de plan que además es límite da `agotada`. Recomendado.
5. A1-09 y A1-10: rojo con el diff literal antes del refactor y con el mutante equivalente después, ambos en `## Resultado`. Recomendado.

**Respuestas de la persona (Carlos, consigna del 2026-11-23: modo automatico con la opcion recomendada):** 1 (a), 2 sí, 3, 4 y 5 como se recomiendan.

**Regla de una hora:** 300 min, sin trocear: Carlos descartó trocear las tareas de corrección 4D (consigna del 2026-11-23).

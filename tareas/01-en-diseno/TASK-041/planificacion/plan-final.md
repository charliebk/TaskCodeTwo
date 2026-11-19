# Plan — TASK-041: F5-1 workflow reutilizable de CI y plantilla de GitLab CI

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 100 min
Senales: workflow reutilizable nuevo, plantilla de GitLab nueva, lint en xtask, documento de uso, cambios en los dos repos E2E y dos pruebas reales (GitHub y GitLab); 3 criterios.

F5-1 anade dos piezas de CI que solo llaman al comando de verificacion del proyecto: un workflow reutilizable `.github/workflows/verificacion.yml` (`on: workflow_call`) con los tres jobs de F1-7/F1-8, y una plantilla de GitLab incluida con `include: remote` fijado por SHA y parametrizada con `spec:inputs`. Ninguna conoce los pasos del proyecto. Fuera: cache por arbol (F5-2), informe JSON (F5-4), runner propio (F5-5), `carril`/`revision-ia`.

Choque verificado con F1-7: ToolsaWeb no acepta `--perfil` (`scripts\verificar.bat:68-79` solo admite `--rapida|--selectiva|--pesada`; sin argumentos, completa; otro, `exit /b 2`). Para "sin scripts nuevos", cada perfil acepta argumentos propios, por defecto `--perfil <p>`.

Piezas y limites:
- `.github/workflows/verificacion.yml` (nuevo), tres jobs (F1-8 587-593): `buscar` (Linux, `contents: read`, elige perfiles por evento segun la decision 4 de F1-7); `ejecutar` (matriz por perfil, `runs-on` por perfil, sin secretos, `comando` + `argumentos-<perfil>` en el shell declarado); `verificacion` (publicador, sin checkout del head, suma resultados). Entradas: `comando`, `argumentos-{rapida,impacto,completa,pesada}`, `runner-<perfil>` (JSON para `runs-on`), `shell`.
- `plantillas/gitlab/verificacion.gitlab-ci.yml` (nuevo): mismos `spec:inputs` (con `tags` por perfil); perfil por `rules:` segun `CI_MERGE_REQUEST_TARGET_BRANCH_NAME` y rama de origen; script `$COMANDO $ARGUMENTOS`.
- `xtask/src/workflows.rs` (`#[cfg(test)]`, nuevo, precedente `xtask/src/instaladores.rs`): lint de lista blanca de F1-8 (decision 1): eventos por clase, `permissions` por job, `actions/checkout` sin `ref:`/`repository:`, sin `secrets: inherit`, sin `github.event.*` en `run:`. Entra en `cargo xtask verificar --perfil rapida`.
- `docs/ci/verificacion-reutilizable.md` (nuevo): uso, con los fragmentos literales para ToolsaWeb y OpenDockTrading (un YAML que llama al workflow, sin scripts).
- No se tocan `.github/workflows/ci.yml` ni `.taskcode/config.yml` de este repo (F1-7 "Este repo no cambia", RR-14).
- Repos E2E `charliebk/taskcodetwo-e2e-github` y `taskcodetwo-e2e/taskcodetwo-e2e-gitlab`: workflow que llama al reutilizable, `.gitlab-ci.yml` con el include y stubs del contrato real de argumentos.

Orden de construccion:
1. Workflow reutilizable + lint.
2. Plantilla de GitLab con las mismas entradas; el lint la lee en lo que le aplica.
3. Documento de uso.
4. E2E en GitHub con stubs del contrato real de ToolsaWeb y OpenDockTrading.
5. E2E en GitLab: pipeline real en verde y en rojo.

Alternativas descartadas:
- Exigir `--perfil` sin argumentos por perfil: ToolsaWeb necesitaria un envoltorio (incumple "sin scripts nuevos"). `--perfil` queda por defecto; el ajuste por perfil es transitorio hasta F9.
- Una orden completa por perfil: vuelve a la superficie de seis claves que critica H18.

## Troceo

No se trocea: Carlos pidio estas tareas sin trocear (2026-11-18).

## Lo que el rol no cubrio

- C1: el diseno lo permite; que corran en los runners reales de ToolsaWeb (completa lee `exampleData`, 34 GB; pesada exige la maquina de Carlos, F5-5) y OpenDockTrading no se ve. Se demuestra con stubs del contrato real (D5).
- C2: fallos de GitLab, shells de Windows en GitLab y cuotas, sin revisar.
- C3: seguridad mas alla del lint (forks, `workflow_run`), sin revisar.

## Riesgos aceptados y que los contiene

- Los argumentos por perfil perpetuan la interfaz 0.16: por defecto `--perfil <p>` y F9 los retira.
- El check publicado puede no llamarse `verificacion`: D2 y comprobacion con `gh api GET` en el E2E.
- `target/` compartido en runner propio: `concurrency` por repo y perfil con `cancel-in-progress: false`, mas el cerrojo del propio comando; la verificacion local usa su `CARGO_TARGET_DIR`.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- **C1:** (1) **pesado**, E2E GitHub `windows-latest`, `shell: cmd`: stub con las lineas 68-79 de ToolsaWeb, `argumentos-completa: ""`, `argumentos-pesada: --pesada`, en verde; (2) el mismo stub forzado a fallar deja el check en rojo; (3) stub con la forma de ODT, solo `comando`, en verde.
- **C2:** (1) **pesado**, pipeline real en `taskcodetwo-e2e-gitlab` con `include: remote` por SHA: verde con el stub bueno y rojo con el malo (2 ejecuciones).
- **C3:** (1) lint de `xtask/src/workflows.rs` con caso positivo y negativo; (2) `gh api GET` que confirme el nombre del check y `app_id` 15368.

## Lo que necesita decision de una persona

Consigna de Carlos (2026-11-18): la opcion recomendada y sin trocear.

- **D1. Argumentos por perfil.** Elegida: (a) `comando` + `argumentos-<perfil>`, por defecto `--perfil <p>`. Divergencia con F1-7 decision 1, anotada en el Resultado y en la documentacion.
- **D2. Nombre del check.** Elegida: (a) el publicador crea el check-run `verificacion` con `GITHUB_TOKEN` (`checks: write`, App 15368); vehiculo de `output.summary` para F5-2.
- **D3. Linea de cache en GitLab.** Elegida: (a) artefacto del job, que escribira F5-2.
- **D4. `carril`, `revision-ia`, `ci_config_path` en GitLab.** Elegida: (a) fuera de F5-1 (F2-9/F6), divergencia con F1-13.
- **D5. Demostrar C1.** Elegida: (a) solo el repo E2E de GitHub con stubs del contrato real; no se tocan ToolsaWeb ni OpenDockTrading.
- **D6. Pipeline real en GitLab.com.** Elegida: si (2 ejecuciones, unos 2-6 min de cuota).
- **D7. Publicacion para el E2E.** Decidida por Carlos (2026-11-19): se sube a GitHub SOLO la rama de la tarea de TaskCodeTwo, para que los repos E2E llamen al workflow y a la plantilla por SHA. Ni develop ni main. Los tres repos son publicos.

Suposiciones por verificar en la implementacion:
- Nombre del check de un workflow reutilizable (`<job que llama> / <job llamado>`, F1-7:268): `gh api GET .../check-runs`.
- `shell: ${{ inputs.shell }}` en un paso; si no, un paso por shell con `if:`.
- `spec:inputs` con `include: remote` en GitLab.com Free.
- `charliebk/TaskCodeTwo` es publico: verificado (2026-11-19).

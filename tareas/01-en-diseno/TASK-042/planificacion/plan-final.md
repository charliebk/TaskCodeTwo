# Plan — TASK-042: F5-2 cache por SHA del arbol compartida

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 110 min
Senales: tres jobs del workflow extendidos, entradas nuevas, lint y plantilla, guia, medicion E2E real con varios casos; 2 criterios.

Se amplia `.github/workflows/verificacion.yml`, sin componentes nuevos:
- `buscar` busca, por perfil del evento, candidatos en los check-runs `verificacion` de V, sus padres y abuelos (F1-7 decision 3).
- `ejecutar` calcula la huella local en el runner del perfil y se salta el paso del comando si acierta.
- El publicador escribe una linea `arbol= perfil= runner= clave= local= origen=` por perfil, con el arbol leido por el mismo (F1-8 decision 7).
En GitLab solo se escribe la linea en un artefacto (D3 de TASK-041); la busqueda queda como degradacion declarada (RR-15).

Piezas y limites:
- `buscar`: checkout con `fetch-depth: 3` del merge ref (V, padres, abuelos y arboles por `git` local); hasta 7 `GET check-runs?check_name=verificacion&app_id=15368&filter=latest` (coste fijo por evento); rechaza toda linea si `git diff --name-only HEAD^1 HEAD` toca `.github/workflows/**` o `.taskcode/**`; `clave = sha256(perfil|runner|huella-arbol|github.job_workflow_sha)`; candidatos por perfil en JSON.
- `ejecutar` (matriz): paso `huella-local` en el shell del proyecto y paso `bash` con `sha256sum`; acierto = primer candidato con el mismo `local`; con acierto el paso del comando lleva `if:` y sale `skipped`; corre siempre en el runner del perfil (runner caido falla cerrado); salidas con nombre fijo por perfil.
- Publicador: `GET /commits/{V}` para el arbol, lo compara con el de `buscar`, una linea por perfil con `reutilizada de <sha>` o `ejecutada`; se mantiene la linea `perfiles= resultado= informe= ejecucion=`.
- Entradas nuevas `huella-arbol` (rutas versionadas: lockfiles, toolchain) y `huella-local` (orden opcional), transitorias hasta F1-11 como `argumentos-*`.
- `xtask/src/workflows.rs`: admite `checks: read` y `actions: read` en `buscar`. `plantillas/gitlab/verificacion.gitlab-ci.yml`: artefacto `cache-verificacion-<perfil>` con la linea.
- `docs/ci/verificacion-reutilizable.md`: formato de la linea (contrato para F8-1), entradas, como leer un acierto, divergencias.

Orden de construccion:
1. Formato de la linea y salida por perfil en el publicador, sin busqueda.
2. `buscar`: commits, check-runs, filtros (App, nombre, verde, arbol, perfil, runner, clave), regla de rutas, candidatos.
3. `ejecutar`: huella local, acierto, paso del comando con `if:`.
4. Lint, artefacto de GitLab y guia.
5. Medicion E2E en `charliebk/taskcodetwo-e2e-github` con la rama fijada por SHA.

Alternativas descartadas:
- Logica en un binario Rust llamado por el workflow: compilarlo o bajarlo cuesta minutos y no hay release; el precedente del publicador es bash con `gh api`.
- `actions/cache` o un indice por arbol: descartados por F1-7 (D2b, D2c).

## Troceo

No se trocea: Carlos pidio estas tareas sin trocear (2026-11-18).

## Lo que el rol no cubrio

- Riesgos: `filter=latest` con dos llamadores, relanzamientos repetidos del mismo commit, arbol de `merge_group` frente al del head.
- Reglas de la huella (que declara cada proyecto): F1-11.
- La busqueda en bash dentro del YAML no tiene pruebas unitarias: la prueba real es el E2E; el lint solo cubre la forma.

## Riesgos aceptados y que los contiene

- `local` lo produce el job ejecutor (codigo del PR): riesgo ya aceptado por F1-8 decision 7; falsearlo solo da un acierto sobre el mismo arbol.
- Cola de merge sin probar (exige organizacion, F1-5 D3a): si no acierta se verifica entero.
- GitLab sin busqueda (`CI_JOB_TOKEN` no lista pipelines; un token secreto en jobs que ejecutan lo prohibe F1-8): siempre verifica entero, declarado en la guia.
- Huella local sin declarar: `local = sha256("")`, cuenta solo la clave del arbol.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- **C1 (cero repeticiones, medido):** **pesado**, E2E leido con un guion de solo `gh api --method GET` guardado en la carpeta de la tarea, que cuenta por arbol las veces que el paso del comando no salio `skipped` (debe ser 1): (a) PR nuevo `ejecutada`, "Re-run all jobs" `reutilizada de <head>` con el paso `skipped`; (b) commit vacio en el PR `reutilizada de` el anterior; (c) negativos: cambiar un fichero da `ejecutada`; tocar `.github/workflows/**` da `ejecutada`.
- **C2 (F5-2 y lo que cita):** lint de xtask con el workflow nuevo y una mutacion negativa (`checks: write` en `buscar`); prueba de la plantilla con el artefacto `cache-verificacion-*`; verificacion completa en verde.

## Lo que necesita decision de una persona

Consigna de Carlos (2026-11-18): la opcion recomendada y sin trocear.

- **D1. Publicacion para el E2E.** Decidida por Carlos (2026-11-19): se sube a GitHub SOLO la rama de esta tarea de TaskCodeTwo y se mide en el repo E2E publico. Ni develop ni main.
- **D2. GitLab.** Elegida: solo la linea en el artefacto, sin busqueda y sin pipeline real.
- **D3. `actions: read` del llamador.** Elegida: si; `buscar` acepta solo un check-run cuya ejecucion (`details_url` -> `GET /actions/runs/{id}`) llama al workflow de TaskCode (F1-8 decision 7). Cambian los ejemplos de ToolsaWeb y OpenDockTrading de la guia.

Suposiciones por verificar en el E2E:
- `github.job_workflow_sha` disponible dentro del reutilizable.
- Salidas con nombre fijo por perfil en una matriz sin pisarse con vacios (si no, un artefacto por perfil).
- El merge ref con `fetch-depth: 3` trae abuelos y arboles, y su primer padre es la base.
- Una re-ejecucion deja el check anterior como `latest` en el head.
- `bash` en el runner del perfil (Windows del E2E).

# Plan — TASK-043: F5-3 Verificacion por impacto como paso de CI y completa en la cola

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

F5-3 tal como esta escrita choca con el ADR F1-7. El ADR descarto D1c («impacto en `develop`») y dice en sus consecuencias: «F5-3 cambia de expectativa: la impacto no es el check de los PR». Se hace **condicional y explicita**: una entrada `cola-de-merge` (por defecto `false`) en el workflow reutilizable y en la plantilla de GitLab.
- Con `true`: `pull_request` corre `impacto` y `merge_group` corre `completa`, que ya lo hace hoy.
- Con `false`: todo sigue igual (D1a).

Se recoge en una **adenda a F1-7** (precedente: la adenda F6-1 del ADR F1-6). La objecion de D1c (el resultado no se puede reutilizar en la release) no vale con cola: la completa corre sobre el arbol que acaba en `develop`, y la release la encuentra en los padres de su commit.

Duracion estimada: 180 min

Senales: 2 ficheros de CI (workflow y plantilla), tests del lint, guia y adenda, mas una medicion E2E con 6 PR y la prueba de la cola en GitHub.

### Piezas y limites

- `.github/workflows/verificacion.yml`, paso «Perfiles del evento» de `buscar` (extension):
  - entrada nueva `cola-de-merge`;
  - `pull_request` pasa a `impacto` si `cola-de-merge` es `true` y la PR no es de release;
  - la decision sale solo del evento y de las entradas, sin la API de rulesets.
- Mismo workflow, `buscar` (candidatos) y publicador: el perfil `impacto` ni busca candidatos ni escribe la linea `arbol=`, porque depende de `RAMA_BASE` (F1-7, decision 1) y no solo del arbol. El resumen del check lleva `perfiles=["impacto"]` y `cola-de-merge=`.
- `plantillas/gitlab/verificacion.gitlab-ci.yml` (extension): la misma entrada, que exige el test `workflow_y_plantilla_tienen_las_mismas_entradas`. Con cola, el MR corre `verificacion-impacto` y un `merge_train` corre `verificacion-completa`.
- `xtask/src/workflows.rs`: solo tests. Las clases de F1-8 no cambian: no hay permisos, eventos ni secretos nuevos.
- `docs/ci/verificacion-reutilizable.md` (tabla de eventos) y adenda en `docs/adr/F1-7-*.md` (fila nueva en la tabla de la decision 4; D1c admitida con cola).
- No se tocan `ci.yml`, `xtask/src/main.rs` ni `.taskcode/config.yml` de este repo: F1-7 dice «Este repo no cambia» (RR-14) y aqui no hay cola.

### Orden de construccion

1. Adenda a F1-7 y tabla de la guia.
2. Entrada `cola-de-merge` y su rama en `buscar`; `impacto` fuera de candidatos y de la linea `arbol=`; tests del lint.
3. Plantilla de GitLab con la entrada y las reglas de `merge_train`.
4. Medicion E2E antes y despues en `charliebk/taskcodetwo-e2e-github`, y la cola real en un repo de una organizacion (D2).
5. Cifras en el `## Resultado` y en la guia, con lo no verificado.

### Alternativas descartadas

- **Impacto incondicional en el PR y completa en la cola** (lectura literal del plan). Sin cola, que es el caso de todas las cuentas personales (ToolsaWeb y OpenDockTrading hoy), `develop` integraria sin completa. Es lo que rechazan D1b y D1c.
- **Impacto como job extra antes de la completa en cada PR.** El tiempo de CI por PR sube en vez de bajar.

## Troceo

Pasa de 60 min. **No se trocea**: decision de Carlos del 2026-11-20 de cerrar H5 sin trocear.

## Lo que el rol no cubrio

- Riesgos: no se han enumerado los modos de fallo. Dos concretos: un llamador sin cola que ponga `cola-de-merge: true`, y que el arbol del commit de `merge_group` sea distinto del del head.
- Testing: el plan de pruebas no lo ha revisado el rol de testing.
- Dominio: que entra en la impacto de cada proyecto lo decide el proyecto (F1-7, preguntas abiertas).

## Riesgos aceptados y que los contiene

- **`cola-de-merge: true` sin cola real.** Lo contienen el valor por defecto `false`, un aviso en la guia y la linea `cola-de-merge=` en el resumen del check. Que `doctor` lo detecte queda para F3-8.
- **Rapida cacheada por arbol aunque lee `RAMA_BASE`.** Ya pasaba antes de esta tarea; se anota como pregunta abierta en la adenda.
- **Merge trains de GitLab sin probar.** Son de pago (Premium): esa ruta queda «no verificado». En GitLab solo se prueba la ruta de MR sin tren.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- Criterio 1, tiempo de CI por PR (1 prueba): E2E en `charliebk/taskcodetwo-e2e-github` con un comando stub de duraciones fijas por perfil (completa unos 120 s, impacto unos 20 s). Seis PR: tres con `cola-de-merge: false` y tres con `true`. El tiempo va del `created_at` de la ejecucion al `completed_at` del check `verificacion`, sacado con `gh api` y guardado en la evidencia de la tarea. La ganancia en ToolsaWeb y OpenDockTrading se estima con E1, E2 y E4 de F1-7 y se marca «no medida».
- Criterio 2, cumple F5-3 (3 pruebas):
  1. Tests del lint en `workflows.rs`: el `case` con `cola-de-merge` da `impacto` en `pull_request` y `completa` en `merge_group`; las entradas coinciden en GitHub y GitLab; las clases de F1-8 no cambian.
  2. E2E: una PR con `cola-de-merge: true` deja el check `verificacion` con `perfiles=["impacto"]` y sin linea `arbol=`; una PR de release `develop`->`main` sigue con `completa pesada`.
  3. E2E de la cola (D2): el `merge_group` publica `verificacion` con `completa` y linea `arbol=`, y la PR de release siguiente la reutiliza.

Ninguno es pesado.

## Lo que necesita decision de una persona

Carlos acepto el 2026-11-20 todas las opciones recomendadas:
- **D1** — entrada explicita `cola-de-merge`, sin detectar la cola por la API.
- **D2** — Carlos crea una organizacion gratuita de GitHub con un repo publico E2E y una regla de cola (en los repos publicos de una organizacion la cola existe sin Enterprise). Es accion suya: se le daran pasos concretos cuando llegue la prueba. Si no puede, la cola queda «no verificado».
- **D3** — la impacto nunca se cachea.
- **D4** — la medicion con el stub y 3 + 3 PR.
- **D5** — sin trocear.

## Suposiciones no verificadas

- Que el commit de `merge_group` termina como cabeza de `develop` con el mismo SHA, y que la release lo alcanza en dos niveles.
- Que la cola esta disponible en los repos publicos de una organizacion gratuita.
- Que `CI_MERGE_REQUEST_EVENT_TYPE == "merge_train"` es la variable correcta en GitLab.

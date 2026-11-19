# ADR F1-13 — Adaptador de plataforma GitHub y GitLab: tabla de equivalencias de las 20 garantías con su tier, sustitutos declarados, contrato de `capacidades` y llamadas por operación

- Estado: **aceptada** (2026-11-14)
- Tarea: TASK-015 · Plan: fila F1-13 del [PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) y su apartado P11
- Requisitos: RR-15 (GitHub y GitLab desde la 2.0, con la diferencia documentada y dicha por `doctor`), RF-54 (siempre por PR o MR, GitLab detrás del adaptador), RF-64 (`doctor` con el arreglo de cada fallo), RN-01 (identidad propia del agente) y RR-08 (las garantías las impone la plataforma) de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); RR-12 (equipo) para la interferencia en un MR compartido
- Depende de: [ADR F1-4](F1-4-operaciones-git-y-contrato-de-salida.md) (puerto `Plataforma`, lista cerrada, contrato de salida y códigos 0–7), [ADR F1-5](F1-5-puertas-en-github.md) (modelo neutral de puertas, `aplicar-puertas`, `leer-puertas`, D2a y D3a), [ADR F1-6](F1-6-identidades-agente-revisor-persona.md) (operación → identidad, contrato del firmante, decisión 6), [ADR F1-7](F1-7-verificacion-ci-perfiles-cache-runners.md) (caché en el check `verificacion`), [ADR F1-8](F1-8-seguridad-modelo-de-amenazas.md) (dos clases de jobs, `carril` con la definición de la base, runner propio), [ADR F1-10](F1-10-carriles-y-modo-automatico.md) (carril mínimo con los dos nombres de cada renombrado), [ADR F1-11](F1-11-extensiones-declaradas.md) (comandos sí, hooks solo como atajo opcional), [ADR F1-12](F1-12-actualizacion-entre-versiones.md) (nombres `tc2` y `taskcodetwo`) y [ADR F1-14](F1-14-distribucion-e-instalacion.md) (HTTP con TLS del sistema, sin `gh` ni `glab`)
- Adenda (2026-11-17, TASK-032, F3-5): operación nueva de lectura `prs-abiertos(rol)` → lista de `{número, rama, sha, borrador}` de los PR/MR abiertos **del propio repo**, por número; campos mínimos (D2 y D3 de TASK-032: sin checks ni aprobaciones). Rol `agente` o `persona` (el `revisor` se rechaza); permiso `pull_requests:read` en GitHub (el mismo que ya usa `abrir-pr` para buscar el PR existente) y el permiso de lectura del adaptador en GitLab (`read_api` o `api` y rol Developer, el mismo de `abrir-pr`). Los PR de forks no cuentan: en GitHub se descarta el que tiene `head.repo.id` distinto de `base.repo.id` (o nulo, fork borrado); en GitLab, el que tiene `source_project_id` distinto de `target_project_id`. Borrador: `draft` en GitHub; `draft` o `work_in_progress` (obsoleto) en GitLab. Campos de GitLab comprobados contra la documentación de la API de MR (2026-11-17), no contra la API real. Una rama que no pasa `rama_valida` se salta (no puede casar con una tarea); un sha que no es de 40 hex es `respuesta-invalida`. Paginación con `Link rel=next` hasta `MAX_PAGINAS` (10 × 100); con más, `error motivo=demasiadas-paginas` y `board` sale con `plataforma=error`. El tope HTTP de 30 s es **por llamada**, no por operación: en el peor caso una consulta paginada podría esperar 10 × 30 s; sin red cae en la primera llamada (tope de conexión de 10 s). Probado contra el mock (`tests/prs_abiertos.rs`), no contra GitHub ni GitLab reales
- Adenda (2026-11-17, TASK-035, F3-8): **`leer-puertas` gana lecturas** (decisión 4) y devuelve además `aprobaciones`, `checks_requeridos`, `todos_los_checks` y `checks_publicados`. GitHub: los `parameters` de las reglas que ya trae `GET /rules/branches/{rama}` (`required_status_checks[].context`, `required_approving_review_count`) y una llamada más, `GET /commits/heads/{rama}/check-runs` (la página da la rama como `heads/BRANCH_NAME`: [doc](https://docs.github.com/en/rest/checks/runs), 2026-11-17; esquema de reglas: [doc](https://docs.github.com/en/rest/repos/rules), 2026-11-17, que no trae ejemplo de respuesta: los nombres salen del esquema de rulesets). GitLab: `GET /projects/:id` (`only_allow_merge_if_pipeline_succeeds`: GitLab no exige checks por nombre; con «Pipelines must succeed» cuenta el pipeline entero, `todos_los_checks`), `GET /repository/branches/:rama` (`commit.id`: [doc](https://docs.gitlab.com/api/branches/), 2026-11-17) y `GET /repository/commits/:sha/statuses` (la página documenta `:sha` como «hash of the commit» y no dice que acepte una rama: [doc](https://docs.gitlab.com/api/commits/), 2026-11-17; por eso se lee antes la rama). El campo `only_allow_merge_if_pipeline_succeeds` sale en los ejemplos de respuesta de la [API de proyectos](https://docs.gitlab.com/api/projects/) (2026-11-17), no en su tabla de atributos: si no viene, `checks_requeridos` es `None` y `doctor` dice `no-comprobado`. Lo que no se lee nunca se da por bueno. **Operación nueva de lectura `leer-plan(rol)`** (agente o persona): GitHub, `GET /repos/{o}/{r}` (`private`, `visibility`, `owner.type`; [doc](https://docs.github.com/en/rest/repos/repos), 2026-11-17; `internal` cuenta como privado); GitLab, `GET /projects/:id` (`namespace.id`) y `GET /namespaces/:id` (`plan`: la [API de namespaces](https://docs.gitlab.com/api/namespaces/), 2026-11-17, lo da con los ejemplos `free`, `ultimate` y `default` y no dice quién lo ve). Solo `free`, `premium` y `ultimate` se reconocen; cualquier otro valor, o el namespace ilegible, es «plan no comprobado», salvo el 403 de `approval_rules`, que sigue diciendo Free (decisión 3). **Corrige la decisión 3 en GitLab**: el E2E de F2-9 observó que en Free `approval_rules` da 200 con la lista vacía, no 403; por eso el plan sale del namespace. **`capacidades` en F3-8**: la tabla de la decisión 2 compilada (`doctor/capacidades.rs`); nombres de plan `github-free-publico-usuario`, `github-free-privado-usuario`, `github-pro-privado-usuario`, `github-free-publico-org`, `github-team-privado-org`, `github-free-privado-org` (nuevo: 9/1/10) y `gitlab-free|premium|ultimate`; `enterprise-privado-org` no se distingue de Team con lo observable y sale como Team (la cola sustituida: lo conservador). La configuración (`motivo=config arreglo=tc2-init`) solo se comprueba en las garantías que `leer-puertas` lee: en GitHub, G01 (`deletion` y `non_fast_forward`), G02 (`pull_request`), G03 (`required_approving_review_count`), G04 (`dismiss_stale_reviews_on_push`), G05 (`require_last_push_approval`) y G09 (`required_status_checks`), todos de `GET /rules/branches/{rama}` (los cuatro parámetros de `pull_request` son obligatorios en el esquema de [rulesets](https://docs.github.com/en/rest/repos/rules), 2026-11-17); en GitLab, G02 (`push_access_levels`), G03 (`approval_rules`) y G06 (`code_owner_approval_required`); las demás siguen al plan. Si la configuración de una de esas garantías no se pudo leer (las puertas de `develop` fallaron, o en GitLab el 403 de `approval_rules` con un plan legible que las da), sale `garantia Gnn no-comprobado motivo=puertas-no-leidas|permiso-insuficiente`, no cuenta como impuesta y la primera línea añade `no-comprobadas=N` (ronda 1 de TASK-035, H1 y H4). Un `null` donde la API da una lista, un booleano o un número (`required_status_checks`, `push_access_levels`, `code_owner_approval_required`, `approvals_required`) se lee como vacío, `false` o 0: lo conservador. `doctor` lee las puertas de `develop` y `main` con una sola lectura de lo común al repo (`rulesets`; en GitLab, ramas protegidas, `approval_rules` y proyecto) y los checks publicados solo en `develop`; si esos checks no se leen, `check … no-comprobado` da el motivo y, en un 403, `permiso=checks:read` en GitHub (permiso «Checks» de lectura de una GitHub App: [doc](https://docs.github.com/en/rest/authentication/permissions-required-for-github-apps), 2026-11-17) o `permiso=read_api` en GitLab. Arreglos: `repo-publico-o-github-pro`, `org-publica-o-enterprise-cloud`, `gitlab-premium`, `ninguno-limite-de-la-plataforma` y `tc2-init`. Línea de `doctor`: las cuentas van en su primera línea (`plan=… impuestas=N sustituidas=M no-disponibles=K`, sin la línea aparte `doctor puertas …` de la sección C, que no cabría en las 5) y cada garantía no impuesta es un hallazgo `garantia Gnn aviso motivo=… capacidad=sustituida por=…|no-disponible arreglo=…`. Probado contra el mock (`tests/doctor.rs`), no contra GitHub ni GitLab reales
- Decide: Carlos, con sus decisiones del 2026-11-14 (nombres `tc2`, `taskcodetwo` y rama `taskcodetwo/tareas`; comandos deterministas antes que hooks; seguridad y trabajo en grupo imprescindibles; sin trocear y fiel a los datos; GitHub y GitLab solo lectura) y la consigna «la opción recomendada» (D1a a D8a del plan de TASK-015)

## Contexto

P11 y RR-15 piden las mismas garantías en GitHub y en GitLab desde la 2.0:
rama protegida, pipeline obligatoria, aprobación de una persona, identidad
propia del agente y del revisor, y cola o tren de merge. P11 añade que algunas
funciones de GitLab dependen del plan de pago y que **este ADR lo verifica**, y
que `doctor` dice qué garantía no está disponible y qué la sustituye.

Los ADR anteriores dejaron aquí la mitad GitLab de sus decisiones: F1-4, las
equivalencias del puerto `Plataforma`; F1-5, la traducción del modelo neutral
de puertas, la cola y los planes; F1-6 (decisión 6), la correspondencia de
identidades, «no verificada»; F1-7, si la caché puede buscar en los pipelines
de los padres; F1-8, pipelines de merge request, variables protegidas y
runners por etiqueta; F1-10, el diff del MR con renombrados.

Preguntas que responde: qué API de cada plataforma da cada garantía y en qué
plan; qué sustituto se usa donde no la da; cómo sabe `doctor` lo que hay sin
que nadie lo declare; cuántas lecturas cuesta cada operación en cada
plataforma; y cómo se prueba que los dos adaptadores dicen lo mismo.

## Alternativas

| | Descripción | Veredicto |
|---|---|---|
| **D1a** Puerto común con `capacidades` | El puerto `Plataforma` de F1-4 con dos implementaciones; la tabla de equivalencias es un dato que lee `capacidades` y `doctor` imprime. | **Elegida**: una sola superficie para los comandos de fase y la diferencia como dato comprobable (PoC M y C). |
| D1b Superficies distintas | Cada plataforma con sus propias operaciones. | Descartada: los comandos de fase tendrían dos caminos y la batería no sería la misma (P11). |
| D1c Mínimo común de los planes Free | Solo lo que dan GitHub Free y GitLab Free. | Descartada: en un repo privado gratuito de GitHub no hay ninguna puerta (GH-12, GH-13) y se perdería todo lo que da un plan de pago. |
| **D2a** Sustituto declarado | Si el plan no da una garantía, la tabla dice el sustituto, `init` sigue y `doctor` lo dice en cada ejecución. | **Elegida**. Descartadas: (b) `init` rechaza (deja fuera GitLab Free y los privados gratuitos), (c) solo avisar una vez. |
| **D3a** GitLab.com y self-managed | Misma tabla por tier; verificado solo en GitLab.com. | **Elegida**. (b) solo GitLab.com: descartada, el tier de los tokens de proyecto cambia en self-managed (L11). |
| **D4a** Agente en GitLab = token de proyecto | Usuario bot con rol Developer si el tier lo da; si no, cuenta de servicio. | **Elegida**. (b) token personal: descartada, rompe RN-01. |
| **D5a** PoC en Python con fixtures JSON | `python -I -B`, fixtures que F2-6 y F2-9 pueden reutilizar desde Rust. | **Elegida**. (b) Rust ya: el adaptador real es de F2-6/F2-9. |
| **D6a** Envoltorio único de solo lectura | Toda llamada sale por una función que rechaza, antes de la red, lo que no sea GET o no vaya a un destino de una lista blanca, con registro y prueba negativa. | **Elegida** (PoC N): el token de `gh` tiene escritura (`repo`). (b) disciplina: descartada. |
| **D7a** Divergencias anotadas | El ADR escribe el texto propuesto; no edita F1-4..F1-10. | **Elegida**. |
| **D8a** Plataforma por el host de `origin` | `github.com` y `gitlab.com` se detectan; otro host exige clave explícita; ambiguo, falla cerrado. | **Elegida**. (b) siempre clave explícita: un paso más en `init` sin ganar nada en los dos casos comunes. |

## Pruebas de concepto

Guion `poc-f1-13.py`, su salida `poc-f1-13.log`, `fuentes.md` y los
fixtures (`fixtures/github`, `fixtures/gitlab`, `fixtures/observado.json`),
en la carpeta `poc/` de TASK-015 (sin enlace: la carpeta de la tarea cambia
de estado al cerrarse). Ejecutado con Python 3.12.10 (`python -I -B`), `gh`
2.95.0 y la carpeta de trabajo
`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-015\poc\`, borrada al
terminar. **Nada se escribe en GitHub ni en GitLab**: toda llamada pasa por
el envoltorio `Lector`, que comprueba el método y el destino antes de salir
a la red: con `gh`, solo rutas `repos/charliebk/…` (y `apps/github-actions`),
nunca una que empiece por `-`; ningún segmento `.` o `..`, tampoco codificado
(`%2e`, `%252e`), ni barra invertida; por HTTP, solo la API v4 de GitLab.com para el
proyecto 278964 y el usuario aprobador fijo, y el mock de `127.0.0.1` de esa
ejecución; sin seguir redirecciones.
`glab` está instalado (1.102.0) pero no se usa: los GET a GitLab.com son
anónimos con `urllib`.

```bash
python -I -B poc-f1-13.py "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-015/poc" <repo> > poc-f1-13.log
```

El log es determinista salvo su sección V (hash y tamaño de cada cuerpo,
cabeceras de límite de peticiones), que no se comprueba; los tiempos van a
stderr. Lo estable de cada GET (estado HTTP y presencia o valor de campos) se
compara con `fixtures/observado.json`: si la plataforma cambia, la PoC falla.
Los cuerpos se guardan en la carpeta de trabajo y se borran; en el repo solo
quedan claves de campos, estados y booleanos, sin nombres de usuario.

<!-- resumen:inicio -->
Resumen: 38 GET reales y ninguna escritura; dieciséis peticiones con otro método
o fuera de la lista blanca, rechazadas sin salir a la red. De las **20** garantías, en GitHub **10**
dependen del plan de los rulesets, **9** no dependen del plan y **1** (la cola)
exige organización. En GitLab, **11** se imponen en Free, **6** exigen
Premium, ninguna exige Ultimate y **3** no las impone en ningún tier. Con los dos
adaptadores contra el mock, la misma batería da la misma línea en los 6 casos;
`estado-puertas` lee **4** veces en GitHub y **5** en GitLab.
<!-- resumen:fin -->

### I — Inventario

```text
gh: gh version 2.95.0 (2026-06-17)
glab: glab 1.102.0 (b5a548b)
python: 3.12.10
gh auth status: sesion=si ; ambitos incluyen escritura (repo)=si (la lectura la garantiza el envoltorio, no el token)
```

### N — El envoltorio rechaza todo lo que no es GET

```text
NEG-01 POST: rechazado por el envoltorio
NEG-02 DELETE: rechazado por el envoltorio
NEG-03 PUT: rechazado por el envoltorio
NEG-04 get: rechazado por el envoltorio
NEG-05 GET a otro host: rechazado por el envoltorio
NEG-06 GET a otro proyecto de GitLab: rechazado por el envoltorio
NEG-07 GET con prefijo enganoso: rechazado por el envoltorio
NEG-08 gh GET a un repo ajeno: rechazado por el envoltorio
NEG-09 gh GET con una ruta que es una opcion: rechazado por el envoltorio
NEG-10 GET a otro puerto local: rechazado por el envoltorio
NEG-11 gh GET fuera de repos/: rechazado por el envoltorio
NEG-12 gh GET con ..: rechazado por el envoltorio
NEG-13 GET con ..: rechazado por el envoltorio
NEG-14 GET con %2e%2e: rechazado por el envoltorio
NEG-15 GET con %252e%252e: rechazado por el envoltorio
NEG-16 GET con .: rechazado por el envoltorio
peticiones HTTP que salieron: 0 ; procesos gh: 0 ; peticiones que llegaron al mock: 0 ; rechazos: 16
```

`NEG-02` es un `DELETE` de una rama de este repo con `gh`: no llega a lanzarse
el proceso. `NEG-04` comprueba que el método se compara exacto (`get` en
minúsculas también se rechaza).

### R — GET reales

GitHub con `gh api --method GET --include` sobre `charliebk/TaskCodeTwo`
(público, cuenta personal gratuita) y `charliebk/taskcodetwo-poc-f1-1`
(privado, la misma cuenta); GitLab.com anónimo sobre `gitlab-org/gitlab`
(proyecto 278964), con el MR !271075 y su commit fijos. Extracto (el log
completo da los campos):

```text
GH-03 rules/branches/develop                  -> 200  lista_vacia = true
GH-05 commits/e8ee970…/check-runs             -> 200  runs = ["comprobar-repo<-15368:success"]
GH-07 commits/e8ee970…                        -> 200  verification.verified = false ; reason = "unsigned"
GH-08 pulls?state=all                         -> 200  prs = 0
GH-09 actions/permissions/workflow            -> 200  can_approve_pull_request_reviews = false
GH-11 commits/03f0c42… (cierre de TASK-001)   -> 200  renombrados = 4 ; con_previous_filename = 4
GH-12 taskcodetwo-poc-f1-1/rulesets           -> 403  aviso_de_plan = true
GH-13 taskcodetwo-poc-f1-1/…/protection       -> 403  aviso_de_plan = true
GL-01 projects/278964                         -> 200  merge_method_visible_anonimo = false
GL-03 protected_branches/master               -> 200  allow_force_push = false ; code_owner_approval_required presente
GL-04 protected_tags                          -> 403
GL-05 approval_rules (proyecto)               -> 401
GL-08 merge_requests/271075?include_diverged… -> 200  diverged_commits_count presente
GL-09 merge_requests/271075/approvals         -> 200  aprobador_bot_de_proyecto = true ; campo_bot_en_aprobador = false
GL-12 merge_requests/271075/pipelines         -> 200  ref_de_tren = true
GL-15 repository/commits/e5b0c30…/signature   -> 200  verification_status = "verified_system" ; sin user_id
GL-16 repository/commits/e5b0c30…/statuses    -> 401
GL-17 merge_trains                            -> 200
GL-20 users/26073976 (aprobador bot)          -> 403
GL-21..23 runners, variables, access_tokens   -> 401
llamadas registradas: 38 ; metodos en el registro: ['GET'] ; procesos gh: 15 ; HTTP: 23 ; rechazos: 0
```

Lectura:

- **GitHub privado gratuito**: rulesets y protección clásica dan 403 con el
  aviso de plan (GH-12, GH-13), como en F1-5; el público da `[]` (GH-02,
  GH-03): la API está, sin reglas. No hay ningún PR en el repo (GH-08): las
  reviews no se han podido leer (GH-15 da 404) y su forma sale de la
  documentación (D09).
- **GitLab cuenta aprobaciones de bots**: el MR !271075 tiene entre sus
  aprobadores un usuario bot de proyecto (patrón `project_<id>_bot_…`) y la
  respuesta no dice que lo sea (GL-09: sin campo `bot`); el usuario da 403
  anónimo (GL-20). La regla 6 de F1-6 (contar por `id` contra la lista de
  personas) es imprescindible en GitLab, no una precaución.
- **GitLab firma los commits que crea él** (`verified_system`, GL-15): es el
  equivalente de `web-flow` y no vale como aprobación. La firma no trae el
  usuario: trae la clave (`key.id`).
- **Tren de merge real**: el MR tiene un pipeline sobre la ref `…/train`
  (GL-12) y el endpoint de trenes responde (GL-17); `gitlab-org` es un
  proyecto de pago: no dice nada de Free.
- **Lo que GitLab no enseña sin sesión**: el método de merge y «Pipelines
  must succeed» no salen en el proyecto anónimo (GL-01), ni las reglas de
  aprobación del proyecto, los estados del commit, los runners, las variables
  y los tokens (401). `leer-puertas` y `estado-puertas` necesitan un token con
  lectura de la API.

### M — Misma batería contra el mock, con los dos adaptadores

Un servidor en `127.0.0.1` sirve los fixtures; un adaptador mínimo por
plataforma lee por el mismo envoltorio y devuelve el `EstadoPuertas` neutral:

```text
verde:            [0] estado-puertas ok pr=12 aprobaciones=1 ignoradas=0                       github=4 gitlab=5
esperando-checks: [7] estado-puertas esperando pr=12 aprobaciones=1 ignoradas=0 falta=revision-ia github=4 gitlab=5
sin-aprobacion:   [7] estado-puertas esperando pr=12 aprobaciones=0 ignoradas=0 falta=aprobacion  github=4 gitlab=5
aprobacion-bot:   [7] estado-puertas esperando pr=12 aprobaciones=0 ignoradas=1 falta=aprobacion  github=4 gitlab=5
plan:             [3] estado-puertas rechazada pr=12 motivo=plan garantia=G03                    github=2 gitlab=2
sin-cambios:      [0] estado-puertas sin-cambios pr=12 estado=mergeado                           github=1 gitlab=1
```

- En `esperando-checks` hay además un `revision-ia` **en verde publicado por
  otra identidad** (otra App en GitHub; el bot del agente en GitLab): no
  cuenta en ninguna de las dos (en GitHub por `integration_id`, en GitLab por
  el autor del estado, sustituto de G09).
- `sin-aprobacion` corre con el plan sin reglas de aprobación (403) y la
  política por defecto: el adaptador evalúa igual y espera la aprobación
  (D2a). `plan` es el mismo 403 con la política endurecida
  (`exigir_impuestas: [G03]`): `rechazada`, `motivo=plan`, código 3.
- Los campos de los fixtures son un subconjunto de los observados donde hubo
  respuesta real (GH-05, GL-03, GL-08, GL-09). Sin respuesta real que
  contrastar: el PR y sus reviews de GitHub, los estados del commit de GitLab
  y el 403 de `approval_rules` en Free (no observado: anónimo da 401).

### C — `capacidades` y `doctor` por plan

La PoC lee la tabla de equivalencias de este ADR como dato y calcula, por
plan, impuestas / sustituidas / no disponibles. `doctor` nunca pasa de 5
líneas (F1-4):

```text
-- gitlab free
   doctor puertas plataforma=gitlab plan=free impuestas=11 sustituidas=9 no-disponibles=0
   G01 sustituida motivo=plataforma por=maintainer-solo-persona arreglo=ninguno-limite-de-la-plataforma
   G03 sustituida motivo=plan por=merge-solo-persona arreglo=gitlab-premium
   G04 sustituida motivo=plan por=merge-solo-persona arreglo=gitlab-premium
   y 6 mas: tc2 doctor --json
-- puertas leidas sin CODEOWNERS en GitLab Premium: G06 no_disponible motivo=config arreglo=tc2-init
```

<!-- capacidades:inicio -->
| Plataforma y plan | impuestas / sustituidas / no disponibles |
|---|---|
| `github free-publico-usuario` | `19 / 1 / 0` |
| `github free-privado-usuario` | `9 / 1 / 10` |
| `github pro-privado-usuario` | `19 / 1 / 0` |
| `github free-publico-org` | `20 / 0 / 0` |
| `github team-privado-org` | `19 / 1 / 0` |
| `github enterprise-privado-org` | `20 / 0 / 0` |
| `gitlab free` | `11 / 9 / 0` |
| `gitlab premium` | `17 / 3 / 0` |
| `gitlab ultimate` | `17 / 3 / 0` |
<!-- capacidades:fin -->

Lo que sale: en un repo privado de una cuenta gratuita de GitHub no hay
sustituto para lo que dan los rulesets (D2a de F1-5: «puertas no impuestas
por la plataforma»); en GitLab Free no falta ninguna, pero 9 están
sustituidas y solo 11 las impone la plataforma.

### G — El ADR y sus mutaciones

La sección G comprueba este fichero: la tabla tiene las 20 garantías; cada
celda de GitHub y de GitLab cita una URL oficial con fecha que está en
`fuentes.md` con la misma fecha; el tier de cada celda es el esperado y el de
alguna URL citada; las filas «empírica» citan ids de llamada que están en el
log; los sustitutos son los esperados; el bloque de capacidades y las cifras
de la prosa salen del cálculo; IDs, enlaces, secciones, estado y fila del
índice. Cada copia mutada (fila borrada o de más, URL sin fecha o fuera de
`fuentes.md`, tier cambiado en el ADR o en `fuentes.md`, sustituto borrado,
fila empírica sin id, con un id inexistente o con ids que no prueban esa
fila (los de G15 cambiados por otros; un 401 anónimo en G20), cita de 15
palabras o más,
capacidades, cifras de la prosa, ID, enlace, sección, estado, «No verificado»
y «Divergencias») falla por su motivo (el log lo lista).

## Cómo se ve en GitHub y en GitLab

Hoy (observado): `charliebk/TaskCodeTwo` es público, sin rulesets ni
`develop` protegida, sin CODEOWNERS ni PR (GH-02, GH-03, GH-04, GH-08,
GH-14); el único check es `comprobar-repo` de Actions (GH-05). No hay ningún
proyecto de TaskCodeTwo en GitLab. Nada cambia hasta el relevo (RR-14).

Con este ADR aplicado (documentación citada en la tabla; **no verificado**:
no se ha configurado nada en ninguna plataforma):

| Dónde | GitHub | GitLab |
|---|---|---|
| Reglas de la rama | Settings → Rules → Rulesets `taskcodetwo-*` (F1-5) | Settings → Repository → Protected branches: `develop`, `main`, `feature/*`, `fix/*`, `hotfix/*`, `taskcodetwo/tareas`; «Allowed to push» nadie (salvo `taskcodetwo/tareas`), «Allowed to merge» Maintainers, «Allowed to force push» apagado |
| Aprobaciones | «Review required» en la caja de merge | Premium: regla de aprobación con 1 persona, «Prevent approval by author», «Prevent approvals by users who add commits», «Remove all approvals…»; Free: el widget de aprobación es opcional y solo la persona (único Maintainer) ve el botón de merge |
| Checks | `verificacion`, `revision-ia`, `carril` obligatorios con su App | «Pipelines must succeed»; `revision-ia` como estado de commit del bot del revisor en el mismo pipeline, que `estado-puertas` cuenta solo si lo publicó el revisor; Ultimate: un check externo con «Status checks must succeed» como defensa más (no da el origen) |
| Método de merge | solo «Create a merge commit» en el PR | Settings → Merge requests: «Merge commit», squash «Do not allow», para todo el proyecto |
| Cola | organización: «Merge when ready» con la cola | Premium: «Merge trains»; Free: botón de merge cuando `estado-puertas` dice `ok` |
| Tags `v*` | ruleset de tag | Protected tags `v*` |
| `doctor` | `doctor puertas plataforma=github plan=… impuestas=…` y hasta 4 líneas | lo mismo con `plataforma=gitlab` |

## Decisión

1. **Forma del adaptador (D1a).** El puerto `Plataforma` de F1-4 tiene dos
   implementaciones, GitHub y GitLab, por la API REST (GitLab v4) con el
   cliente HTTP de F1-14, sin `gh` ni `glab`. Las operaciones de la lista
   cerrada (las de F1-4, `aplicar-puertas` y `leer-puertas` de F1-5,
   `leer-permisos-ci` de F1-6 con nombre neutral) tienen el mismo contrato de
   salida y los mismos códigos 0–7 en las dos. Si una plataforma o su plan no
   permite una operación, el resultado es `rechazada` con `motivo=plan` o
   `motivo=plataforma` (código 3), nunca un éxito vacío.

2. **Tabla de equivalencias** (dato; la lee `capacidades` y la comprueba la
   PoC G). Una fila por garantía; la regla neutral es la de F1-5 y F1-6.
   Tiers de GitHub: «Free público; Pro/Team/Enterprise» (repos públicos con
   Free; públicos y privados con Pro, Team y Enterprise Cloud), «org pública;
   Enterprise Cloud» y «todos» (la página no pone restricción de plan). Tiers
   de GitLab (GitLab.com): Free, Premium, Ultimate, y «ninguno» cuando
   ningún tier da la garantía tal cual. El sustituto empieza por su nombre
   corto, que es el que imprime `doctor`. La evidencia es «empírica» (con los
   ids de llamada del log, además de la documentación) o «documental». Un id
   solo vale si la llamada prueba algo de esa fila (un 2xx con el campo, o el
   403 con aviso de plan); GH-03 (`200 []`) solo prueba que el endpoint de
   reglas lee y no hace empírica ninguna fila; los 401 y 403 anónimos solo prueban que la lectura
   necesita sesión y no hacen empírica ninguna fila (la PoC G lo comprueba con
   una tabla de qué prueba cada id).

   <!-- tabla-equivalencias:inicio -->
   | id | garantía | regla neutral | GitHub (API) | tier GitHub | GitLab (API) | tier GitLab | sustituto si falta | evidencia |
   |---|---|---|---|---|---|---|---|---|
   | G01 | sin borrado ni force-push | `sin_borrado sin_forzar` | ruleset `deletion` y `non_fast_forward`, sin `bypass_actors` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | rama protegida con `allow_force_push: false`; borrarla la puede un Maintainer por la interfaz o la API ([doc](https://docs.gitlab.com/user/project/repository/branches/protected/), 2026-11-14) | ninguno | GitHub: — / GitLab: `maintainer-solo-persona` la persona es el único Maintainer u Owner; `doctor` compara la rama con el modelo | empírica: GH-12, GL-03 |
   | G02 | solo por PR o MR | `solo_pr` | ruleset `pull_request` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | rama protegida con «Allowed to push» a nadie (`push_access_levels` 0) ([doc](https://docs.gitlab.com/user/project/repository/branches/protected/), 2026-11-14) | Free | GitHub: — / GitLab: — | empírica: GL-02 |
   | G03 | N aprobaciones de persona | `aprobaciones=N` | `required_approving_review_count` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | regla de aprobación obligatoria ([doc](https://docs.gitlab.com/user/project/merge_requests/approvals/), 2026-11-14; [reglas](https://docs.gitlab.com/user/project/merge_requests/approvals/rules/), 2026-11-14); en Free son opcionales | Premium | GitHub: — / GitLab: `merge-solo-persona` «Allowed to merge» Maintainers con la persona como único Maintainer y `merge-pr` con su identidad; `estado-puertas` exige su aprobación | empírica: GH-12, GL-09 |
   | G04 | descartar aprobaciones en cada push | `descartar_viejas` | `dismiss_stale_reviews_on_push` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | «Remove all approvals when commits are added to the source branch» ([doc](https://docs.gitlab.com/user/project/merge_requests/approvals/settings/), 2026-11-14) | Premium | GitHub: — / GitLab: `merge-solo-persona` el merge lo hace la persona después del último push | documental |
   | G05 | aprobación del último push; ni el autor ni quien añade commits aprueban | `ultimo_push` | `require_last_push_approval` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | «Prevent approval by author» y «Prevent approvals by users who add commits» ([doc](https://docs.gitlab.com/user/project/merge_requests/approvals/settings/), 2026-11-14) | Premium | GitHub: — / GitLab: `merge-solo-persona` el agente es Developer y no puede mergear; `carril` solo cuenta aprobaciones de personas | documental |
   | G06 | propietarios de código | `propietarios` | `require_code_owner_review` y CODEOWNERS ([doc](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners), 2026-11-14) | Free público; Pro/Team/Enterprise | Code Owners y `code_owner_approval_required` en la rama protegida ([doc](https://docs.gitlab.com/user/project/codeowners/), 2026-11-14) | Premium | GitHub: — / GitLab: `carril-propietarios` el job `carril` falla si el MR toca la CI o `.taskcode/` sin aprobación de la persona | empírica: GL-03 |
   | G07 | solo merge commit | `metodos=[merge]` | `allowed_merge_methods: [merge]` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | método del proyecto «Merge commit» y squash «Do not allow», por proyecto y no por rama ([doc](https://docs.gitlab.com/user/project/merge_requests/methods/), 2026-11-14) | Free | GitHub: — / GitLab: — | documental |
   | G08 | rama al día (solo `develop`) | `al_dia` | `strict_required_status_checks_policy` por rama ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | «Merge commit with semi-linear history» es del proyecto entero ([doc](https://docs.gitlab.com/user/project/merge_requests/methods/), 2026-11-14); `diverged_commits_count` del MR ([doc](https://docs.gitlab.com/api/merge_requests/), 2026-11-14); merged results ([doc](https://docs.gitlab.com/ci/pipelines/merged_results_pipelines/), 2026-11-14) | ninguno | GitHub: — / GitLab: `al-dia-por-cli` `estado-puertas` da `falta=al-dia` si el MR va por detrás de `develop` y `finish` no mergea | empírica: GL-08 |
   | G09 | checks obligatorios con su origen | `checks=[nombre<-origen]` | `required_status_checks` con `integration_id` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | «Pipelines must succeed» exige el pipeline entero, sin origen ([doc](https://docs.gitlab.com/user/project/merge_requests/auto_merge/), 2026-11-14); un Developer puede crear estados de commit ([doc](https://docs.gitlab.com/user/permissions/), 2026-11-14); checks de estado externos (Ultimate), que el MR ignora salvo «Status checks must succeed» ([doc](https://docs.gitlab.com/user/project/merge_requests/status_checks/), 2026-11-14) y que puede responder cualquier Developer que pueda aprobar el MR ([API](https://docs.gitlab.com/api/status_checks/), 2026-11-14): ninguno da el origen | ninguno | GitHub: — / GitLab: `autor-del-estado` pipeline obligatoria y `estado-puertas` solo cuenta el `revision-ia` cuyo `author.id` es el del revisor, en todos los tiers (en Ultimate, el check externo con «Status checks must succeed» es una defensa más, no el origen) | empírica: GH-05, GH-10 |
   | G10 | cola o tren de merge | `cola` | merge queue (`merge_group`) ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue), 2026-11-14) | org pública; Enterprise Cloud | merge trains ([doc](https://docs.gitlab.com/ci/pipelines/merge_trains/), 2026-11-14) | Premium | GitHub: `al-dia-por-cli` rama al día y `merge-pr` por la API (D3a de F1-5) / GitLab: `al-dia-por-cli` `estado-puertas` al día y `merge-pr` | empírica: GL-12, GL-17 |
   | G11 | tags `v*` protegidos | `v*: sin_borrado sin_mover` | ruleset de tag `deletion` y `update` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | protected tags ([doc](https://docs.gitlab.com/user/project/protected_tags/), 2026-11-14) | Free | GitHub: — / GitLab: — | documental |
   | G12 | identidad propia del agente | `agente` | GitHub App con tokens de instalación ([doc](https://docs.github.com/en/apps/creating-github-apps/about-creating-github-apps/deciding-when-to-build-a-github-app), 2026-11-14) | todos | token de proyecto (usuario bot) con rol Developer ([doc](https://docs.gitlab.com/user/project/settings/project_access_tokens/), 2026-11-14) | Premium | GitHub: — / GitLab: `cuenta-de-servicio` cuenta de servicio con rol Developer, Free desde 18.11, sin ocupar asiento y hasta 100 por grupo de nivel superior en GitLab.com Free ([doc](https://docs.gitlab.com/user/profile/service_accounts/), 2026-11-14) | empírica: GL-09 |
   | G13 | identidad propia del revisor y cómo publica | `revisor -> revision-ia` | otra App con `checks:write` que publica un check-run ([doc](https://docs.github.com/en/rest/checks/runs), 2026-11-14) | todos | otra cuenta de servicio o token que publica un estado de commit (`POST /projects/:id/statuses/:sha`) ([doc](https://docs.gitlab.com/api/commits/), 2026-11-14; [estados externos](https://docs.gitlab.com/ci/ci_cd_for_external_repos/external_commit_statuses/), 2026-11-14) | Free | GitHub: — / GitLab: — | documental |
   | G14 | firma verificada del commit de `approve` | `firma_verificada(persona)` | `commit.verification.verified` y la cuenta `committer` ([doc](https://docs.github.com/en/authentication/managing-commit-signature-verification/about-commit-signature-verification), 2026-11-14) | todos | `GET /repository/commits/:sha/signature`: `verification_status` `verified` (no `verified_system`) y el id de la clave ([doc](https://docs.gitlab.com/user/project/repository/signed_commits/), 2026-11-14; [API](https://docs.gitlab.com/api/commits/), 2026-11-14) | Free | GitHub: — / GitLab: — | empírica: GH-07, GL-15 |
   | G15 | solo cuentan aprobaciones de personas | `aprobacion(id en lista de personas)` | reviews `APPROVED` con `user.type: User` ([doc](https://docs.github.com/en/rest/pulls/reviews), 2026-11-14) | todos | `approved_by[].user.id` de las aprobaciones del MR ([doc](https://docs.gitlab.com/api/merge_request_approvals/), 2026-11-14); `bot` solo en `GET /users/:id` con sesión ([doc](https://docs.gitlab.com/api/users/), 2026-11-14) | Free | GitHub: — / GitLab: — | empírica: GL-09 |
   | G16 | el token de la CI no aprueba | `ci_no_aprueba` | `can_approve_pull_request_reviews: false` ([doc](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-github-actions-settings-for-a-repository), 2026-11-14) | todos | el job token solo llega a una lista cerrada de endpoints, sin aprobar ([doc](https://docs.gitlab.com/ci/jobs/ci_job_token/), 2026-11-14) | Free | GitHub: — / GitLab: — | empírica: GH-09 |
   | G17 | resultado de la verificación por sha (caché de F1-7) | `resultado_por_sha` | `GET /commits/{sha}/check-runs?check_name&app_id` y `output.summary` ([doc](https://docs.github.com/en/rest/checks/runs), 2026-11-14) | todos | `GET /pipelines?sha=` ([doc](https://docs.gitlab.com/api/pipelines/), 2026-11-14) y estados del commit, con `description` de 255 como mucho ([doc](https://docs.gitlab.com/api/commits/), 2026-11-14) | Free | GitHub: — / GitLab: — | empírica: GH-05, GL-13 |
   | G18 | diff con los dos nombres de cada renombrado | `diff(viejo, nuevo)` | `GET /pulls/{n}/files` con `previous_filename`, 3000 ficheros como mucho ([doc](https://docs.github.com/en/rest/pulls/pulls), 2026-11-14) | todos | `GET /merge_requests/:iid/diffs` con `renamed_file`, `old_path` y `new_path` ([doc](https://docs.gitlab.com/api/merge_requests/), 2026-11-14) | Free | GitHub: — / GitLab: — | empírica: GH-11, GL-14 |
   | G19 | código de forks sin secretos | `forks_sin_secretos` | los secretos no llegan a workflows lanzados desde forks ([doc](https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets), 2026-11-14) | todos | el pipeline de un MR desde un fork corre en el fork ([doc](https://docs.gitlab.com/ci/pipelines/merge_request_pipelines/), 2026-11-14); variables protegidas solo en ramas y tags protegidos ([doc](https://docs.gitlab.com/ci/variables/), 2026-11-14) | Free | GitHub: — / GitLab: — | documental |
   | G20 | runner propio por etiqueta y protegido | `runner(etiquetas, protegido)` | etiquetas en `runs-on` ([doc](https://docs.github.com/en/actions/how-tos/manage-runners/self-hosted-runners/apply-labels), 2026-11-14); nunca en repos públicos ([doc](https://docs.github.com/en/actions/reference/security/secure-use), 2026-11-14) | todos | tags del runner y runner protegido, solo ramas o tags protegidos ([doc](https://docs.gitlab.com/ci/runners/configure_runners/), 2026-11-14) | Free | GitHub: — / GitLab: — | documental |
   <!-- tabla-equivalencias:fin -->

   Lectura de la tabla: en GitHub, 10 garantías dependen del plan de los
   rulesets (en un privado hacen falta Pro, Team o Enterprise Cloud; en un
   público bastan con Free), 9 garantías no dependen del plan en GitHub, y la
   cola exige organización (pública, o privada con Enterprise Cloud); y
   en GitLab Free se imponen 11 garantías; 6 garantías exigen Premium;
   ninguna garantía exige Ultimate; y 3 garantías no las impone GitLab en
   ningún tier tal como las definen F1-5 y F1-6: el borrado de una rama
   protegida (un Maintainer puede, por la interfaz o la API), la rama al día
   solo en `develop` (el método de merge es del proyecto entero; ver la
   divergencia 3) y el origen de un check (un estado de commit lo crea
   cualquier Developer, y el check externo de Ultimate lo responde cualquier
   usuario con rol Developer o superior que pueda aprobar el MR, y por defecto
   no bloquea el merge). De las 20 filas, 13 garantías tienen además evidencia
   empírica en el log; el resto es documental.

3. **Contrato de `capacidades`** (lo implementan F3-8, F2-6 y F2-9):

   ```text
   capacidades(plataforma: GitHub|GitLab, plan_observado: Plan, puertas: PuertasLeidas) -> Vec<Garantia>
   Garantia { id: G01..G20,
              estado: impuesta | sustituida(por: <nombre corto>) | no_disponible,
              motivo: plan | plataforma | config,
              arreglo: <valor sin espacios> }
   ```

   - **Función pura** sobre la tabla de la decisión 2, que el binario lleva
     compilada (no se lee del repo del proyecto: no es configuración).
   - **El plan se observa, no se declara**: GitHub, por la visibilidad y el
     tipo de cuenta del repo (GH-01) y el 403 con aviso de plan de
     `rules/branches` (GH-12); GitLab, por el 403 de las API de Premium
     (`approval_rules`) con el token del proyecto (Ultimate no cambia
     ninguna fila de la tabla). Sin token o con 401, `doctor` dice «plan no comprobado» y
     no supone nada.
   - **`puertas`** son las que `leer-puertas` encuentra: una garantía que el
     plan da pero el proyecto no tiene configurada sale `no_disponible`,
     `motivo=config`, `arreglo=tc2-init`.
   - **`tc2 doctor`** imprime en cada ejecución una línea
     `doctor puertas plataforma=… plan=… impuestas=N sustituidas=M no-disponibles=K`
     y las no impuestas, con 5 líneas como mucho (F1-4); el resto con
     `tc2 doctor --json` (RR-15, RF-64).

4. **Llamadas por operación** (lecturas; las escrituras, **no verificadas**:
   no se ha escrito en ninguna plataforma). Para `estado-puertas`, contadas
   contra el mock en el caso verde: `estado-puertas` hace 4 llamadas en
   GitHub y 5 en GitLab, porque GitLab separa la rama protegida de las reglas
   de aprobación y no tiene un único endpoint de reglas efectivas. Con el MR ya
   mergeado, una en cada una.

   | Operación | GitHub | GitLab |
   |---|---|---|
   | `abrir-pr` | `GET /pulls?head=` y, si no hay, `POST /pulls` | `GET /merge_requests?source_branch=` y, si no hay, `POST /merge_requests` |
   | `estado-puertas` | `GET /pulls/{n}` (con `mergeable_state`), `GET /rules/branches/{base}`, `GET /commits/{sha}/check-runs`, `GET /pulls/{n}/reviews` | `GET /merge_requests/{iid}?include_diverged_commits_count=true`, `GET /approval_rules`, `GET /protected_branches/{destino}`, `GET /repository/commits/{sha}/statuses`, `GET /merge_requests/{iid}/approvals` |
   | `merge-pr` | `PUT /pulls/{n}/merge` con `merge_method=merge`; con cola, la cola | `PUT /merge_requests/{iid}/merge` (el método lo fija el proyecto); Premium con tren: el tren |
   | `aplicar-puertas` (adenda de TASK-118) | `GET /user` (persona de la lista), `GET /rulesets`, `GET /commits/heads/develop/check-runs` y, de las que faltan, `POST /rulesets`; nunca `PUT` ni `DELETE` | `GET /user` (persona de la lista), `GET /protected_branches`, `GET /protected_tags`, `GET /repository/branches/develop` y sus estados, `GET /projects/:id`, el namespace y en Premium `GET /approval_rules`; de lo que falta, `POST /protected_branches`, `POST /protected_tags`, `PUT /projects/:id` (ajustes de merge) y en Premium `POST /approval_rules`; nunca `DELETE` ni `PATCH` |
   | `leer-puertas` | `GET /rulesets`, `GET /rules/branches/{rama}` | `GET /protected_branches`, `GET /protected_tags`, `GET /projects/:id` con sesión, `GET /approval_rules`, `GET /approvals` |
   | `leer-permisos-ci` | `GET /actions/permissions/workflow` | ninguna: el job token es fijo (G16); informa `fuente=plataforma` |
   | firma del `approve` (`carril`) | `GET /commits/{sha}` | `GET /repository/commits/{sha}/signature` |
   | `prs-abiertos` (adenda de TASK-032) | `GET /pulls?state=open&per_page=100`, paginado | `GET /merge_requests?state=opened&per_page=100`, paginado |

   La caché de F1-7 cuesta lo mismo en las dos: una lectura por commit (hasta
   7, F1-7) con `check-runs` o con `pipelines?sha=`.

5. **Cuando el plan no da una garantía (D2a).** La tabla dice el sustituto;
   `init` sigue y lo anota; `doctor` lo repite siempre con `motivo=plan` y el
   arreglo (`gitlab-premium`, `repo-publico-o-github-pro`…). Una garantía que
   no da ningún tier sale `motivo=plataforma`. La política del proyecto puede
   **endurecerse** (F1-10: la configuración solo endurece) con una lista de
   garantías que tienen que estar impuestas; si el plan no la da,
   `estado-puertas` y `aplicar-puertas` dan `rechazada motivo=plan` (PoC M,
   caso `plan`). Un sustituto es una garantía **degradada declarada**: lo
   impone el CLI o un job, no la plataforma, y quien tenga permisos para
   saltarse el CLI se la salta; por eso `doctor` no lo cuenta como impuesta.

6. **Identidades en GitLab (D4a; completa la decisión 6 de F1-6).**
   - **agente**: token de proyecto (usuario bot) con rol Developer en
     GitLab.com Premium o en self-managed; en GitLab.com Free, una cuenta de
     servicio con rol Developer (sustituto `cuenta-de-servicio`). Nunca un
     token personal (RN-01).
   - **revisor**: otra identidad de la misma clase, que publica `revision-ia`
     como estado de commit; `estado-puertas` solo lo cuenta si el `author.id`
     es el suyo (`autor-del-estado`), porque un Developer, también el agente,
     puede crear un estado con ese nombre.
   - **persona**: su usuario, **único Maintainer u Owner** del proyecto (ni
     el agente ni el revisor lo son: no pueden mergear, desproteger ni borrar
     ramas protegidas). En Premium, además, la regla de aprobación con 1
     persona y los ajustes de la decisión 2 (G04, G05).
   - **Contrato del firmante en GitLab**: el commit de `approve` cuenta si la
     firma es `verified` (no `verified_system`, que es la del propio GitLab,
     GL-15) y el `key.id` de la firma está en la lista de personas, que en
     GitLab guarda, además del `id` de usuario, el id de su clave de firma (la
     respuesta de la firma no trae el usuario, GL-15).
   - **Reviews en GitLab**: cuenta `approved_by[].user.id` contra la lista de
     personas; GitLab cuenta como aprobación la de un bot (GL-09) y no lo
     marca en esa respuesta.
   - **Equipo** (RR-12): dos personas o agentes en el mismo MR. La aprobación
     de un tercero que no está en la lista no cuenta (G15); con «Prevent
     approvals by users who add commits» (Premium), quien empuja a la rama de
     otro deja de poder aprobarla; en Free lo sustituye que solo merguea un
     Maintainer. En GitLab.com Free, los grupos privados tienen un límite de
     5 usuarios ([precios](https://about.gitlab.com/pricing/), 2026-11-14);
     las cuentas de servicio no ocupan asiento y en GitLab.com Free caben
     hasta 100 por grupo de nivel superior ([doc](https://docs.gitlab.com/user/profile/service_accounts/), 2026-11-14).

7. **Detección de plataforma (D8a).** Por el host de `origin`:
   `github.com` → GitHub; `gitlab.com` → GitLab. Otro host exige la clave
   `plataforma` (`github` o `gitlab`, con la URL de la API) en
   `.taskcode/config.yml`; sin ella, o si contradice el host, `init` y
   `doctor` fallan cerrado (`rechazada motivo=plataforma`) y dicen qué clave
   falta.

8. **Alcance de GitLab (D3a).** La misma tabla vale para GitLab.com y
   self-managed; los tiers son los de GitLab.com. En self-managed los tokens
   de proyecto están disponibles con cualquier licencia (L11 de
   `fuentes.md`); lo demás, no verificado.

9. **Cómo se prueba cada adaptador.** Los dos adaptadores pasan **la misma
   batería** (los seis casos de la PoC M y los que añadan F2-6 y F2-9) sobre
   fixtures con los nombres de campo observados; el `EstadoPuertas` neutral
   tiene que ser igual byte a byte. Cada campo de un fixture tiene que estar
   en una respuesta real observada, o marcado como no observado. El E2E
   contra un repo de pruebas en GitHub (F2-6) y un proyecto de pruebas en
   GitLab (F2-9) comprueba lo que aquí es documental: que la plataforma
   bloquea el merge, descarta aprobaciones y respeta el tren. La batería se
   ejecuta con un comando (`tc2` en su suite), no con un hook (F1-11,
   decisión 10).

## Divergencias con el plan

1. **Nombres** (decisión de Carlos): `taskcodetwo` y la rama
   `taskcodetwo/tareas` en vez de `taskcode` y `taskcode/tareas`; los
   rulesets de F1-5 se llaman `taskcodetwo-*` (F1-5 y F1-6 dicen
   `taskcode-*`). Texto propuesto para F1-5, decisión 1: «`taskcodetwo/tareas`
   rama sin_borrado sin_forzar (push directo del CLI)».
2. **F1-4**: `estado-puertas` no cuesta lo mismo en las dos plataformas (4 y 5
   lecturas, decisión 4). Y la operación `leer-permisos-actions` de F1-6 pasa
   a llamarse `leer-permisos-ci`: en GitLab no hace ninguna llamada. Texto
   propuesto para la tabla de F1-4: «`estado-puertas` | `review`, `finish` |
   plataforma; lectura (4 lecturas en GitHub, 5 en GitLab; F1-13)».
3. **F1-5, decisión 2** («`main` sin `strict`»): en GitLab el método de merge,
   y con él la rama al día por historia semilineal, es del proyecto entero; no
   se puede pedir en `develop` y no en `main`. Se sustituye por
   `al-dia-por-cli` (G08). **F1-5, D3a**: la cola no basta con «repo de una
   organización»: en un privado de organización exige Enterprise Cloud (D03),
   no Team. **F1-5, «sin `bypass_actors`»**: en GitLab un Maintainer puede
   borrar una rama protegida y desprotegerla; el sustituto es que solo la
   persona lo sea (G01).
4. **F1-6, decisión 6**: el token de proyecto exige Premium en GitLab.com; en
   Free, cuenta de servicio. «Impedir la aprobación del autor y de quien añade
   commits» exige Premium. El firmante se identifica por la clave, no por el
   usuario. GitLab sí cuenta aprobaciones de bots (GL-09): la regla 6 de F1-6
   pasa de precaución a necesaria. Y «publica su estado como estado de commit
   o check externo» no da identidad propia del revisor: los dos los puede
   publicar el agente (L23, L29); lo da `autor-del-estado`. Texto propuesto: «agente = token de
   proyecto (Premium o self-managed) o cuenta de servicio (Free), rol
   Developer; persona = único Maintainer».
5. **F1-7, decisión 3**: la línea `arbol=… perfil=… runner=… clave=… local=…
   origen=…` no cabe en el `description` de un estado de commit de GitLab: con
   hashes completos y `runner=github-windows`, la línea de F1-7 ocupa 274
   caracteres y el límite es 255 (L13); con hashes de 12 caracteres ocupa 114,
   pero 48 bits no bastan como clave de caché. Propuesta para F5-1: en GitLab,
   el estado lleva solo `clave` (64) y el árbol se lee del commit
   (`GET /repository/commits/:sha`), o la línea va en un artefacto del job;
   lo decide F5-1 midiendo las llamadas.
6. **F1-8, decisiones 2 y 7** (`carril` en `pull_request_target`, publicador
   que lee la definición de la base): GitLab no tiene esos eventos; el
   pipeline de un MR del mismo proyecto usa el `.gitlab-ci.yml` de la rama del
   MR. Sustituto propuesto: la definición de CI en **otro proyecto**
   (`ci_config_path` apuntando a un proyecto que solo la persona puede
   cambiar), que la página da en Free ([doc](https://docs.gitlab.com/ci/pipelines/settings/), 2026-11-14); no verificado.
   Los forks quedan cubiertos (G19).
7. **F1-10, decisión 4**: «la API corta a partir de 3000 ficheros, no
   verificado» queda verificado en la documentación de GitHub (D11). En GitLab
   los renombrados vienen con `renamed_file` y `old_path` (GL-14); el límite
   de ficheros del diff de GitLab, no verificado.
8. **P11** dice «cola o tren de merge» en las dos: ninguna de las dos lo da en
   el plan gratuito de una cuenta personal (D03, L07); queda como garantía
   sustituida (G10). Y «identidad propia del agente y del revisor» en GitLab
   Free solo con cuentas de servicio (L12).

## Requisitos y evidencia

| Requisito | Decisión | Evidencia |
|---|---|---|
| RR-15 / P11 mismas garantías; la diferencia documentada y dicha por `doctor` | tabla de equivalencias (decisión 2), `capacidades` (decisión 3), D2a | PoC G (20 filas, URL y fecha por celda, tier contra `fuentes.md`), C (9 planes, `doctor` con 5 líneas como mucho), M (misma línea en los 6 casos) |
| P11 «algunas funciones de GitLab dependen del plan: el ADR lo verifica» | columnas de tier y sustitutos | documentación con fecha (44 URL en `fuentes.md`); empírico solo en GitLab.com de pago (GL-12, GL-17) y anónimo (401/403) |
| RF-54 siempre por PR o MR, GitLab detrás del adaptador | puerto `Plataforma` con dos implementaciones (D1a) | PoC M (mismo `EstadoPuertas` neutral) |
| RF-64 `doctor` con el arreglo | `arreglo` en cada garantía no impuesta | PoC C |
| RN-01 identidad propia del agente | App en GitHub; token de proyecto o cuenta de servicio en GitLab (D4a) | D06, L11, L12; GL-09 (bots de proyecto actuando en MR); creación real: no verificada (F6-1) |
| RR-08 lo impone la plataforma | sustituto ≠ impuesta; `doctor` lo dice | decisión 5; PoC C |
| RR-12 equipo en un MR compartido | aprobaciones por id; quien añade commits no aprueba (Premium) | GL-09, L03; decisión 6 |
| F1-4 / F1-5 / F1-6 / F1-7 / F1-8 / F1-10 lo que dejaron a F1-13 | divergencias 2 a 7 | PoC R y documentación citada |

## Consecuencias

- **F2-6** y **F2-9** implementan los dos adaptadores con esta tabla y la
  batería de la PoC M (fixtures reutilizables), y prueban en E2E lo marcado
  «documental» y «no verificado».
- **F3-8** (`doctor`) implementa `capacidades` y su salida; **F3-1** (`init`)
  aplica las puertas o el sustituto y lo anota.
- **F6-1** crea las identidades: App en GitHub; token de proyecto o cuenta de
  servicio en GitLab, y la lista de personas con el id de su clave de firma.
- **F5-1** escribe la plantilla de GitLab CI con `carril`, `verificacion` y el
  estado de `revision-ia`, decide dónde va la línea de caché (divergencia 5) y
  si la definición de CI vive en otro proyecto (divergencia 6).
- **F1-5, F1-6, F1-7, F1-8 y F1-10** no se editan: las correcciones están en
  «Divergencias».
- **Este repo no cambia** (RR-14).

## Fuera de alcance

- Crear tokens, cuentas de servicio y Apps: F6-1, F3-1.
- La plantilla de GitLab CI y el workflow de GitHub: F5-1.
- El cliente HTTP y el TLS: F1-14.
- GitHub Enterprise Server y GitLab Dedicated.
- Las escrituras de cada operación (forma exacta de los `POST`/`PUT`): F2-6,
  F2-9.

## No verificado

- **El comportamiento de las puertas sin escribir en ninguna plataforma**:
  que GitHub o GitLab bloqueen el merge sin aprobación o con el pipeline en
  rojo, descarten aprobaciones, respeten el tren o la cola, rechacen un
  force-push. Todo eso es documental; pasa como criterio a F2-6 y F2-9.
- **Ningún tier de pago probado**: los tiers salen de la documentación con
  fecha. Lo único observado en un proyecto de pago es que `gitlab-org/gitlab`
  usa trenes y reglas de propietarios (GL-10, GL-12); el plan de ese grupo no
  se ha leído.
- **El 403 de plan en GitLab Free**: anónimo, `approval_rules` da 401
  (GL-05); que con token en Free dé 403 es una suposición del fixture.
- **GitLab self-managed** y versiones anteriores a GitLab.com del
  2026-11-14 (cuentas de servicio en Free desde 18.11).
- **Que un estado de commit externo bloquee el merge** con «Pipelines must
  succeed» en Free (L24 no lo dice).
- **Que el agente, como Developer, no pueda borrar una rama protegida ni
  mergear** con «Allowed to merge» Maintainers (documentado en L01 y L23).
- **Que el rol Developer baste para leer** `protected_branches` y
  `approval_rules` con la identidad agente: las páginas de la API piden
  sesión pero no nombran el rol mínimo; si exigen Maintainer, esas lecturas
  de `estado-puertas` pasan a la identidad de persona (F1-6). Entra en la
  batería E2E de F2-9.
- **La configuración de CI desde otro proyecto** como sustituto de
  `pull_request_target` (divergencia 6).
- **Las reviews de GitHub reales**: no hay ningún PR en el repo (GH-08).
- **Las escrituras de la decisión 4** (`abrir-pr`, `merge-pr`,
  `aplicar-puertas`): solo por la documentación de la API.
- **El límite de ficheros del diff de un MR en GitLab**.

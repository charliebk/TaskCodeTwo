# Peticion de revision — TASK-015 (ronda 2)

- Tarea: TASK-015 — F1-13 ADR adaptador de plataforma GitHub y GitLab
- Rama revisada: feature/task-015-f1-13-adr-adaptador-de-plataforma-github
- Rama base: develop
- Commit revisado (HEAD): dd39c4108a460d33676e2d283ea2012b370bd381
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-015 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 91a8078c162a537a1538137c29ffd2048c1e6a1b (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (5 hallazgo(s)))

## Instrucciones para el agente revisor (verificacion acotada)

Eres un revisor INDEPENDIENTE del agente que implemento. La rama ya paso una
revision completa: esta ronda verifica las correcciones de la ronda anterior,
no vuelve a revisar la rama entera ni los MENORES de la ronda anterior (tu
tabla solo trae CRITICOS e IMPORTANTES). Haz esto, y nada mas:

1. Comprueba, uno por uno, que cada hallazgo pendiente de tu tabla de abajo
   queda resuelto por el diff de esta ronda (los marcados `corregido` tambien:
   verificalo, no lo des por bueno). Para un CRITICO o IMPORTANTE, reproduce el
   fallo original y comprueba que ya no ocurre; puedes leer el repo entero si la
   causa no esta en el diff.
2. Ejecuta la suite (o el comando de verificacion del proyecto) sobre el commit
   revisado y di el resultado: verificar sin ejecutar no vale. Si esta peticion
   ya trae la verificacion de la ronda en verde para ese mismo commit, no la
   repitas: cita ese resultado y corre solo los tests de lo que verificas.
3. Busca en el diff de esta ronda el fallo que la correccion abre: es justo
   donde se cuelan los fallos nuevos. Si encuentras uno, construye el caso que
   lo rompe antes de reportarlo.

Solo abre un hallazgo NUEVO si es CRITICO o si lo introduce la propia correccion
(algo que el diff de esta ronda rompe). Lo demas que veas fuera de eso va como
nota al final del informe, no en la tabla: no abre otra ronda.

Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. En la tabla, cada hallazgo pendiente que ya este
resuelto va como `corregido`; el que no, `abierto`. Vuelca tu salida en el
informe de esta ronda (informe-revision-2.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-015/tarea.md
- tareas/03-en-revision/TASK-015/planificacion/
- docs/adr/F1-13-adaptador-de-plataforma.md
- docs/adr/README.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 3 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-13-adaptador-de-plataforma.md:285 | informe-revision-1.md |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-13-adaptador-de-plataforma.md:287 | informe-revision-1.md |

## Commits a revisar (git log 91a8078c162a537a1538137c29ffd2048c1e6a1b..HEAD)

````
dd39c41 chore(TASK-015): informe de revision 1 corregido en ca2d75f y constancia de autorrevision ronda 2
ca2d75f docs(adr): F1-13 corrige los hallazgos de la revision 1
cd128b5 chore(TASK-015): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 91a8078c162a537a1538137c29ffd2048c1e6a1b..HEAD)

````diff
diff --git a/docs/adr/F1-13-adaptador-de-plataforma.md b/docs/adr/F1-13-adaptador-de-plataforma.md
index 308ad0e..a5955a1 100644
--- a/docs/adr/F1-13-adaptador-de-plataforma.md
+++ b/docs/adr/F1-13-adaptador-de-plataforma.md
@@ -37,7 +37,7 @@ plataforma; y cómo se prueba que los dos adaptadores dicen lo mismo.
 | **D3a** GitLab.com y self-managed | Misma tabla por tier; verificado solo en GitLab.com. | **Elegida**. (b) solo GitLab.com: descartada, el tier de los tokens de proyecto cambia en self-managed (L11). |
 | **D4a** Agente en GitLab = token de proyecto | Usuario bot con rol Developer si el tier lo da; si no, cuenta de servicio. | **Elegida**. (b) token personal: descartada, rompe RN-01. |
 | **D5a** PoC en Python con fixtures JSON | `python -I -B`, fixtures que F2-6 y F2-9 pueden reutilizar desde Rust. | **Elegida**. (b) Rust ya: el adaptador real es de F2-6/F2-9. |
-| **D6a** Envoltorio único de solo lectura | Toda llamada sale por una función que rechaza lo que no sea GET antes de la red, con registro y prueba negativa. | **Elegida** (PoC N): el token de `gh` tiene escritura (`repo`). (b) disciplina: descartada. |
+| **D6a** Envoltorio único de solo lectura | Toda llamada sale por una función que rechaza, antes de la red, lo que no sea GET o no vaya a un destino de una lista blanca, con registro y prueba negativa. | **Elegida** (PoC N): el token de `gh` tiene escritura (`repo`). (b) disciplina: descartada. |
 | **D7a** Divergencias anotadas | El ADR escribe el texto propuesto; no edita F1-4..F1-10. | **Elegida**. |
 | **D8a** Plataforma por el host de `origin` | `github.com` y `gitlab.com` se detectan; otro host exige clave explícita; ambiguo, falla cerrado. | **Elegida**. (b) siempre clave explícita: un paso más en `init` sin ganar nada en los dos casos comunes. |
 
@@ -50,7 +50,11 @@ de estado al cerrarse). Ejecutado con Python 3.12.10 (`python -I -B`), `gh`
 2.95.0 y la carpeta de trabajo
 `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-015\poc\`, borrada al
 terminar. **Nada se escribe en GitHub ni en GitLab**: toda llamada pasa por
-el envoltorio `Lector`, que comprueba el método antes de salir a la red.
+el envoltorio `Lector`, que comprueba el método y el destino antes de salir
+a la red: con `gh`, solo rutas `repos/charliebk/…` (y `apps/github-actions`),
+nunca una que empiece por `-`; por HTTP, solo la API v4 de GitLab.com para el
+proyecto 278964 y el usuario aprobador fijo, y el mock de `127.0.0.1` de esa
+ejecución; sin seguir redirecciones.
 `glab` está instalado (1.102.0) pero no se usa: los GET a GitLab.com son
 anónimos con `urllib`.
 
@@ -66,11 +70,11 @@ Los cuerpos se guardan en la carpeta de trabajo y se borran; en el repo solo
 quedan claves de campos, estados y booleanos, sin nombres de usuario.
 
 <!-- resumen:inicio -->
-Resumen: 38 GET reales y ninguna escritura; cuatro métodos distintos de GET
-rechazados sin salir a la red. De las **20** garantías, en GitHub **10**
+Resumen: 38 GET reales y ninguna escritura; once peticiones con otro método
+o fuera de la lista blanca, rechazadas sin salir a la red. De las **20** garantías, en GitHub **10**
 dependen del plan de los rulesets, **9** no dependen del plan y **1** (la cola)
 exige organización. En GitLab, **11** se imponen en Free, **6** exigen
-Premium, **1** exige Ultimate y **2** no las impone en ningún tier. Con los dos
+Premium, ninguna exige Ultimate y **3** no las impone en ningún tier. Con los dos
 adaptadores contra el mock, la misma batería da la misma línea en los 6 casos;
 `estado-puertas` lee **4** veces en GitHub y **5** en GitLab.
 <!-- resumen:fin -->
@@ -91,7 +95,14 @@ NEG-01 POST: rechazado por el envoltorio
 NEG-02 DELETE: rechazado por el envoltorio
 NEG-03 PUT: rechazado por el envoltorio
 NEG-04 get: rechazado por el envoltorio
-peticiones HTTP que salieron: 0 ; procesos gh: 0 ; peticiones que llegaron al mock: 0 ; rechazos: 4
+NEG-05 GET a otro host: rechazado por el envoltorio
+NEG-06 GET a otro proyecto de GitLab: rechazado por el envoltorio
+NEG-07 GET con prefijo enganoso: rechazado por el envoltorio
+NEG-08 gh GET a un repo ajeno: rechazado por el envoltorio
+NEG-09 gh GET con una ruta que es una opcion: rechazado por el envoltorio
+NEG-10 GET a otro puerto local: rechazado por el envoltorio
+NEG-11 gh GET fuera de repos/: rechazado por el envoltorio
+peticiones HTTP que salieron: 0 ; procesos gh: 0 ; peticiones que llegaron al mock: 0 ; rechazos: 11
 ```
 
 `NEG-02` es un `DELETE` de una rama de este repo con `gh`: no llega a lanzarse
@@ -208,7 +219,7 @@ líneas (F1-4):
 | `github enterprise-privado-org` | `20 / 0 / 0` |
 | `gitlab free` | `11 / 9 / 0` |
 | `gitlab premium` | `17 / 3 / 0` |
-| `gitlab ultimate` | `18 / 2 / 0` |
+| `gitlab ultimate` | `17 / 3 / 0` |
 <!-- capacidades:fin -->
 
 Lo que sale: en un repo privado de una cuenta gratuita de GitHub no hay
@@ -226,7 +237,9 @@ log; los sustitutos son los esperados; el bloque de capacidades y las cifras
 de la prosa salen del cálculo; IDs, enlaces, secciones, estado y fila del
 índice. Cada copia mutada (fila borrada o de más, URL sin fecha o fuera de
 `fuentes.md`, tier cambiado en el ADR o en `fuentes.md`, sustituto borrado,
-fila empírica sin id o con un id inexistente, cita de 15 palabras o más,
+fila empírica sin id, con un id inexistente o con ids que no prueban esa
+fila (los de G15 cambiados por otros; un 401 anónimo en G20), cita de 15
+palabras o más,
 capacidades, cifras de la prosa, ID, enlace, sección, estado, «No verificado»
 y «Divergencias») falla por su motivo (el log lo lista).
 
@@ -244,7 +257,7 @@ no se ha configurado nada en ninguna plataforma):
 |---|---|---|
 | Reglas de la rama | Settings → Rules → Rulesets `taskcodetwo-*` (F1-5) | Settings → Repository → Protected branches: `develop`, `main`, `feature/*`, `fix/*`, `hotfix/*`, `taskcodetwo/tareas`; «Allowed to push» nadie (salvo `taskcodetwo/tareas`), «Allowed to merge» Maintainers, «Allowed to force push» apagado |
 | Aprobaciones | «Review required» en la caja de merge | Premium: regla de aprobación con 1 persona, «Prevent approval by author», «Prevent approvals by users who add commits», «Remove all approvals…»; Free: el widget de aprobación es opcional y solo la persona (único Maintainer) ve el botón de merge |
-| Checks | `verificacion`, `revision-ia`, `carril` obligatorios con su App | «Pipelines must succeed»; `revision-ia` como estado de commit del bot del revisor en el mismo pipeline; Ultimate: además un check de estado externo |
+| Checks | `verificacion`, `revision-ia`, `carril` obligatorios con su App | «Pipelines must succeed»; `revision-ia` como estado de commit del bot del revisor en el mismo pipeline, que `estado-puertas` cuenta solo si lo publicó el revisor; Ultimate: un check externo con «Status checks must succeed» como defensa más (no da el origen) |
 | Método de merge | solo «Create a merge commit» en el PR | Settings → Merge requests: «Merge commit», squash «Do not allow», para todo el proyecto |
 | Cola | organización: «Merge when ready» con la cola | Premium: «Merge trains»; Free: botón de merge cuando `estado-puertas` dice `ok` |
 | Tags `v*` | ruleset de tag | Protected tags `v*` |
@@ -269,44 +282,51 @@ no se ha configurado nada en ninguna plataforma):
    de GitLab (GitLab.com): Free, Premium, Ultimate, y «ninguno» cuando
    ningún tier da la garantía tal cual. El sustituto empieza por su nombre
    corto, que es el que imprime `doctor`. La evidencia es «empírica» (con los
-   ids de llamada del log, además de la documentación) o «documental».
+   ids de llamada del log, además de la documentación) o «documental». Un id
+   solo vale si la llamada prueba algo de esa fila (un 2xx con el campo, o el
+   403 con aviso de plan); los 401 y 403 anónimos solo prueban que la lectura
+   necesita sesión y no hacen empírica ninguna fila (la PoC G lo comprueba con
+   una tabla de qué prueba cada id).
 
    <!-- tabla-equivalencias:inicio -->
    | id | garantía | regla neutral | GitHub (API) | tier GitHub | GitLab (API) | tier GitLab | sustituto si falta | evidencia |
    |---|---|---|---|---|---|---|---|---|
    | G01 | sin borrado ni force-push | `sin_borrado sin_forzar` | ruleset `deletion` y `non_fast_forward`, sin `bypass_actors` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | rama protegida con `allow_force_push: false`; borrarla la puede un Maintainer por la interfaz o la API ([doc](https://docs.gitlab.com/user/project/repository/branches/protected/), 2026-11-14) | ninguno | GitHub: — / GitLab: `maintainer-solo-persona` la persona es el único Maintainer u Owner; `doctor` compara la rama con el modelo | empírica: GH-03, GH-12, GL-03 |
    | G02 | solo por PR o MR | `solo_pr` | ruleset `pull_request` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | rama protegida con «Allowed to push» a nadie (`push_access_levels` 0) ([doc](https://docs.gitlab.com/user/project/repository/branches/protected/), 2026-11-14) | Free | GitHub: — / GitLab: — | empírica: GH-03, GL-02 |
-   | G03 | N aprobaciones de persona | `aprobaciones=N` | `required_approving_review_count` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | regla de aprobación obligatoria ([doc](https://docs.gitlab.com/user/project/merge_requests/approvals/), 2026-11-14; [reglas](https://docs.gitlab.com/user/project/merge_requests/approvals/rules/), 2026-11-14); en Free son opcionales | Premium | GitHub: — / GitLab: `merge-solo-persona` «Allowed to merge» Maintainers con la persona como único Maintainer y `merge-pr` con su identidad; `estado-puertas` exige su aprobación | empírica: GH-12, GL-05, GL-09 |
+   | G03 | N aprobaciones de persona | `aprobaciones=N` | `required_approving_review_count` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | regla de aprobación obligatoria ([doc](https://docs.gitlab.com/user/project/merge_requests/approvals/), 2026-11-14; [reglas](https://docs.gitlab.com/user/project/merge_requests/approvals/rules/), 2026-11-14); en Free son opcionales | Premium | GitHub: — / GitLab: `merge-solo-persona` «Allowed to merge» Maintainers con la persona como único Maintainer y `merge-pr` con su identidad; `estado-puertas` exige su aprobación | empírica: GH-12, GL-09 |
    | G04 | descartar aprobaciones en cada push | `descartar_viejas` | `dismiss_stale_reviews_on_push` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | «Remove all approvals when commits are added to the source branch» ([doc](https://docs.gitlab.com/user/project/merge_requests/approvals/settings/), 2026-11-14) | Premium | GitHub: — / GitLab: `merge-solo-persona` el merge lo hace la persona después del último push | documental |
    | G05 | aprobación del último push; ni el autor ni quien añade commits aprueban | `ultimo_push` | `require_last_push_approval` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | «Prevent approval by author» y «Prevent approvals by users who add commits» ([doc](https://docs.gitlab.com/user/project/merge_requests/approvals/settings/), 2026-11-14) | Premium | GitHub: — / GitLab: `merge-solo-persona` el agente es Developer y no puede mergear; `carril` solo cuenta aprobaciones de personas | documental |
-   | G06 | propietarios de código | `propietarios` | `require_code_owner_review` y CODEOWNERS ([doc](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners), 2026-11-14) | Free público; Pro/Team/Enterprise | Code Owners y `code_owner_approval_required` en la rama protegida ([doc](https://docs.gitlab.com/user/project/codeowners/), 2026-11-14) | Premium | GitHub: — / GitLab: `carril-propietarios` el job `carril` falla si el MR toca la CI o `.taskcode/` sin aprobación de la persona | empírica: GH-14, GL-03 |
-   | G07 | solo merge commit | `metodos=[merge]` | `allowed_merge_methods: [merge]` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | método del proyecto «Merge commit» y squash «Do not allow», por proyecto y no por rama ([doc](https://docs.gitlab.com/user/project/merge_requests/methods/), 2026-11-14) | Free | GitHub: — / GitLab: — | empírica: GL-01 |
+   | G06 | propietarios de código | `propietarios` | `require_code_owner_review` y CODEOWNERS ([doc](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners), 2026-11-14) | Free público; Pro/Team/Enterprise | Code Owners y `code_owner_approval_required` en la rama protegida ([doc](https://docs.gitlab.com/user/project/codeowners/), 2026-11-14) | Premium | GitHub: — / GitLab: `carril-propietarios` el job `carril` falla si el MR toca la CI o `.taskcode/` sin aprobación de la persona | empírica: GL-03 |
+   | G07 | solo merge commit | `metodos=[merge]` | `allowed_merge_methods: [merge]` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | método del proyecto «Merge commit» y squash «Do not allow», por proyecto y no por rama ([doc](https://docs.gitlab.com/user/project/merge_requests/methods/), 2026-11-14) | Free | GitHub: — / GitLab: — | documental |
    | G08 | rama al día (solo `develop`) | `al_dia` | `strict_required_status_checks_policy` por rama ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | «Merge commit with semi-linear history» es del proyecto entero ([doc](https://docs.gitlab.com/user/project/merge_requests/methods/), 2026-11-14); `diverged_commits_count` del MR ([doc](https://docs.gitlab.com/api/merge_requests/), 2026-11-14); merged results ([doc](https://docs.gitlab.com/ci/pipelines/merged_results_pipelines/), 2026-11-14) | ninguno | GitHub: — / GitLab: `al-dia-por-cli` `estado-puertas` da `falta=al-dia` si el MR va por detrás de `develop` y `finish` no mergea | empírica: GL-08 |
-   | G09 | checks obligatorios con su origen | `checks=[nombre<-origen]` | `required_status_checks` con `integration_id` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | «Pipelines must succeed» exige el pipeline entero, sin origen ([doc](https://docs.gitlab.com/user/project/merge_requests/auto_merge/), 2026-11-14); un Developer puede crear estados de commit ([doc](https://docs.gitlab.com/user/permissions/), 2026-11-14); checks de estado externos ([doc](https://docs.gitlab.com/user/project/merge_requests/status_checks/), 2026-11-14) | Ultimate | GitHub: — / GitLab: `autor-del-estado` pipeline obligatoria y `estado-puertas` solo cuenta el `revision-ia` cuyo `author.id` es el del revisor | empírica: GH-05, GH-10, GL-16, GL-18 |
+   | G09 | checks obligatorios con su origen | `checks=[nombre<-origen]` | `required_status_checks` con `integration_id` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | «Pipelines must succeed» exige el pipeline entero, sin origen ([doc](https://docs.gitlab.com/user/project/merge_requests/auto_merge/), 2026-11-14); un Developer puede crear estados de commit ([doc](https://docs.gitlab.com/user/permissions/), 2026-11-14); checks de estado externos (Ultimate), que el MR ignora salvo «Status checks must succeed» ([doc](https://docs.gitlab.com/user/project/merge_requests/status_checks/), 2026-11-14) y que puede responder cualquier Developer que pueda aprobar el MR ([API](https://docs.gitlab.com/api/status_checks/), 2026-11-14): ninguno da el origen | ninguno | GitHub: — / GitLab: `autor-del-estado` pipeline obligatoria y `estado-puertas` solo cuenta el `revision-ia` cuyo `author.id` es el del revisor, en todos los tiers (en Ultimate, el check externo con «Status checks must succeed» es una defensa más, no el origen) | empírica: GH-05, GH-10 |
    | G10 | cola o tren de merge | `cola` | merge queue (`merge_group`) ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue), 2026-11-14) | org pública; Enterprise Cloud | merge trains ([doc](https://docs.gitlab.com/ci/pipelines/merge_trains/), 2026-11-14) | Premium | GitHub: `al-dia-por-cli` rama al día y `merge-pr` por la API (D3a de F1-5) / GitLab: `al-dia-por-cli` `estado-puertas` al día y `merge-pr` | empírica: GL-12, GL-17 |
-   | G11 | tags `v*` protegidos | `v*: sin_borrado sin_mover` | ruleset de tag `deletion` y `update` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | protected tags ([doc](https://docs.gitlab.com/user/project/protected_tags/), 2026-11-14) | Free | GitHub: — / GitLab: — | empírica: GL-04 |
-   | G12 | identidad propia del agente | `agente` | GitHub App con tokens de instalación ([doc](https://docs.github.com/en/apps/creating-github-apps/about-creating-github-apps/deciding-when-to-build-a-github-app), 2026-11-14) | todos | token de proyecto (usuario bot) con rol Developer ([doc](https://docs.gitlab.com/user/project/settings/project_access_tokens/), 2026-11-14) | Premium | GitHub: — / GitLab: `cuenta-de-servicio` cuenta de servicio con rol Developer, Free desde 18.11 ([doc](https://docs.gitlab.com/user/profile/service_accounts/), 2026-11-14) | empírica: GL-09, GL-23 |
+   | G11 | tags `v*` protegidos | `v*: sin_borrado sin_mover` | ruleset de tag `deletion` y `update` ([doc](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets), 2026-11-14) | Free público; Pro/Team/Enterprise | protected tags ([doc](https://docs.gitlab.com/user/project/protected_tags/), 2026-11-14) | Free | GitHub: — / GitLab: — | documental |
+   | G12 | identidad propia del agente | `agente` | GitHub App con tokens de instalación ([doc](https://docs.github.com/en/apps/creating-github-apps/about-creating-github-apps/deciding-when-to-build-a-github-app), 2026-11-14) | todos | token de proyecto (usuario bot) con rol Developer ([doc](https://docs.gitlab.com/user/project/settings/project_access_tokens/), 2026-11-14) | Premium | GitHub: — / GitLab: `cuenta-de-servicio` cuenta de servicio con rol Developer, Free desde 18.11, sin ocupar asiento y hasta 100 por grupo de nivel superior en GitLab.com Free ([doc](https://docs.gitlab.com/user/profile/service_accounts/), 2026-11-14) | empírica: GL-09 |
    | G13 | identidad propia del revisor y cómo publica | `revisor -> revision-ia` | otra App con `checks:write` que publica un check-run ([doc](https://docs.github.com/en/rest/checks/runs), 2026-11-14) | todos | otra cuenta de servicio o token que publica un estado de commit (`POST /projects/:id/statuses/:sha`) ([doc](https://docs.gitlab.com/api/commits/), 2026-11-14; [estados externos](https://docs.gitlab.com/ci/ci_cd_for_external_repos/external_commit_statuses/), 2026-11-14) | Free | GitHub: — / GitLab: — | documental |
    | G14 | firma verificada del commit de `approve` | `firma_verificada(persona)` | `commit.verification.verified` y la cuenta `committer` ([doc](https://docs.github.com/en/authentication/managing-commit-signature-verification/about-commit-signature-verification), 2026-11-14) | todos | `GET /repository/commits/:sha/signature`: `verification_status` `verified` (no `verified_system`) y el id de la clave ([doc](https://docs.gitlab.com/user/project/repository/signed_commits/), 2026-11-14; [API](https://docs.gitlab.com/api/commits/), 2026-11-14) | Free | GitHub: — / GitLab: — | empírica: GH-07, GL-15 |
-   | G15 | solo cuentan aprobaciones de personas | `aprobacion(id en lista de personas)` | reviews `APPROVED` con `user.type: User` ([doc](https://docs.github.com/en/rest/pulls/reviews), 2026-11-14) | todos | `approved_by[].user.id` de las aprobaciones del MR ([doc](https://docs.gitlab.com/api/merge_request_approvals/), 2026-11-14); `bot` solo en `GET /users/:id` con sesión ([doc](https://docs.gitlab.com/api/users/), 2026-11-14) | Free | GitHub: — / GitLab: — | empírica: GL-09, GL-20 |
+   | G15 | solo cuentan aprobaciones de personas | `aprobacion(id en lista de personas)` | reviews `APPROVED` con `user.type: User` ([doc](https://docs.github.com/en/rest/pulls/reviews), 2026-11-14) | todos | `approved_by[].user.id` de las aprobaciones del MR ([doc](https://docs.gitlab.com/api/merge_request_approvals/), 2026-11-14); `bot` solo en `GET /users/:id` con sesión ([doc](https://docs.gitlab.com/api/users/), 2026-11-14) | Free | GitHub: — / GitLab: — | empírica: GL-09 |
    | G16 | el token de la CI no aprueba | `ci_no_aprueba` | `can_approve_pull_request_reviews: false` ([doc](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-github-actions-settings-for-a-repository), 2026-11-14) | todos | el job token solo llega a una lista cerrada de endpoints, sin aprobar ([doc](https://docs.gitlab.com/ci/jobs/ci_job_token/), 2026-11-14) | Free | GitHub: — / GitLab: — | empírica: GH-09 |
-   | G17 | resultado de la verificación por sha (caché de F1-7) | `resultado_por_sha` | `GET /commits/{sha}/check-runs?check_name&app_id` y `output.summary` ([doc](https://docs.github.com/en/rest/checks/runs), 2026-11-14) | todos | `GET /pipelines?sha=` ([doc](https://docs.gitlab.com/api/pipelines/), 2026-11-14) y estados del commit, con `description` de 255 como mucho ([doc](https://docs.gitlab.com/api/commits/), 2026-11-14) | Free | GitHub: — / GitLab: — | empírica: GH-05, GL-13, GL-16 |
+   | G17 | resultado de la verificación por sha (caché de F1-7) | `resultado_por_sha` | `GET /commits/{sha}/check-runs?check_name&app_id` y `output.summary` ([doc](https://docs.github.com/en/rest/checks/runs), 2026-11-14) | todos | `GET /pipelines?sha=` ([doc](https://docs.gitlab.com/api/pipelines/), 2026-11-14) y estados del commit, con `description` de 255 como mucho ([doc](https://docs.gitlab.com/api/commits/), 2026-11-14) | Free | GitHub: — / GitLab: — | empírica: GH-05, GL-13 |
    | G18 | diff con los dos nombres de cada renombrado | `diff(viejo, nuevo)` | `GET /pulls/{n}/files` con `previous_filename`, 3000 ficheros como mucho ([doc](https://docs.github.com/en/rest/pulls/pulls), 2026-11-14) | todos | `GET /merge_requests/:iid/diffs` con `renamed_file`, `old_path` y `new_path` ([doc](https://docs.gitlab.com/api/merge_requests/), 2026-11-14) | Free | GitHub: — / GitLab: — | empírica: GH-11, GL-14 |
-   | G19 | código de forks sin secretos | `forks_sin_secretos` | los secretos no llegan a workflows lanzados desde forks ([doc](https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets), 2026-11-14) | todos | el pipeline de un MR desde un fork corre en el fork ([doc](https://docs.gitlab.com/ci/pipelines/merge_request_pipelines/), 2026-11-14); variables protegidas solo en ramas y tags protegidos ([doc](https://docs.gitlab.com/ci/variables/), 2026-11-14) | Free | GitHub: — / GitLab: — | empírica: GL-22 |
-   | G20 | runner propio por etiqueta y protegido | `runner(etiquetas, protegido)` | etiquetas en `runs-on` ([doc](https://docs.github.com/en/actions/how-tos/manage-runners/self-hosted-runners/apply-labels), 2026-11-14); nunca en repos públicos ([doc](https://docs.github.com/en/actions/reference/security/secure-use), 2026-11-14) | todos | tags del runner y runner protegido, solo ramas o tags protegidos ([doc](https://docs.gitlab.com/ci/runners/configure_runners/), 2026-11-14) | Free | GitHub: — / GitLab: — | empírica: GL-21 |
+   | G19 | código de forks sin secretos | `forks_sin_secretos` | los secretos no llegan a workflows lanzados desde forks ([doc](https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets), 2026-11-14) | todos | el pipeline de un MR desde un fork corre en el fork ([doc](https://docs.gitlab.com/ci/pipelines/merge_request_pipelines/), 2026-11-14); variables protegidas solo en ramas y tags protegidos ([doc](https://docs.gitlab.com/ci/variables/), 2026-11-14) | Free | GitHub: — / GitLab: — | documental |
+   | G20 | runner propio por etiqueta y protegido | `runner(etiquetas, protegido)` | etiquetas en `runs-on` ([doc](https://docs.github.com/en/actions/how-tos/manage-runners/self-hosted-runners/apply-labels), 2026-11-14); nunca en repos públicos ([doc](https://docs.github.com/en/actions/reference/security/secure-use), 2026-11-14) | todos | tags del runner y runner protegido, solo ramas o tags protegidos ([doc](https://docs.gitlab.com/ci/runners/configure_runners/), 2026-11-14) | Free | GitHub: — / GitLab: — | documental |
    <!-- tabla-equivalencias:fin -->
 
    Lectura de la tabla: en GitHub, 10 garantías dependen del plan de los
    rulesets (en un privado hacen falta Pro, Team o Enterprise Cloud; en un
    público bastan con Free), 9 garantías no dependen del plan en GitHub, y la
    cola exige organización (pública, o privada con Enterprise Cloud); y
-   en GitLab Free se imponen 11 garantías; 6 garantías exigen Premium; 1 garantía
-   exige Ultimate (el origen de un check, con los checks de estado externos);
-   y 2 garantías no las impone GitLab en ningún tier tal como las define F1-5:
-   el borrado de una rama protegida (un Maintainer puede, por la interfaz o la
-   API) y la rama al día solo en `develop` (el método de merge es del proyecto
-   entero; ver la divergencia 3). De las 20 filas, 17 garantías tienen además
-   evidencia empírica en el log; el resto es documental.
+   en GitLab Free se imponen 11 garantías; 6 garantías exigen Premium;
+   ninguna garantía exige Ultimate; y 3 garantías no las impone GitLab en
+   ningún tier tal como las definen F1-5 y F1-6: el borrado de una rama
+   protegida (un Maintainer puede, por la interfaz o la API), la rama al día
+   solo en `develop` (el método de merge es del proyecto entero; ver la
+   divergencia 3) y el origen de un check (un estado de commit lo crea
+   cualquier Developer, y el check externo de Ultimate lo responde cualquier
+   usuario con rol Developer o superior que pueda aprobar el MR, y por defecto
+   no bloquea el merge). De las 20 filas, 13 garantías tienen además evidencia
+   empírica en el log; el resto es documental.
 
 3. **Contrato de `capacidades`** (lo implementan F3-8, F2-6 y F2-9):
 
@@ -323,8 +343,8 @@ no se ha configurado nada en ninguna plataforma):
    - **El plan se observa, no se declara**: GitHub, por la visibilidad y el
      tipo de cuenta del repo (GH-01) y el 403 con aviso de plan de
      `rules/branches` (GH-12); GitLab, por el 403 de las API de Premium
-     (`approval_rules`) y de Ultimate (`external_status_checks`) con el token
-     del proyecto. Sin token o con 401, `doctor` dice «plan no comprobado» y
+     (`approval_rules`) con el token del proyecto (Ultimate no cambia
+     ninguna fila de la tabla). Sin token o con 401, `doctor` dice «plan no comprobado» y
      no supone nada.
    - **`puertas`** son las que `leer-puertas` encuentra: una garantía que el
      plan da pero el proyecto no tiene configurada sale `no_disponible`,
@@ -392,7 +412,8 @@ no se ha configurado nada en ninguna plataforma):
      otro deja de poder aprobarla; en Free lo sustituye que solo merguea un
      Maintainer. En GitLab.com Free, los grupos privados tienen un límite de
      5 usuarios ([precios](https://about.gitlab.com/pricing/), 2026-11-14);
-     si las cuentas de servicio cuentan para ese límite, no verificado.
+     las cuentas de servicio no ocupan asiento y en GitLab.com Free caben
+     hasta 100 por grupo de nivel superior ([doc](https://docs.gitlab.com/user/profile/service_accounts/), 2026-11-14).
 
 7. **Detección de plataforma (D8a).** Por el host de `origin`:
    `github.com` → GitHub; `gitlab.com` → GitLab. Otro host exige la clave
@@ -441,7 +462,9 @@ no se ha configurado nada en ninguna plataforma):
    Free, cuenta de servicio. «Impedir la aprobación del autor y de quien añade
    commits» exige Premium. El firmante se identifica por la clave, no por el
    usuario. GitLab sí cuenta aprobaciones de bots (GL-09): la regla 6 de F1-6
-   pasa de precaución a necesaria. Texto propuesto: «agente = token de
+   pasa de precaución a necesaria. Y «publica su estado como estado de commit
+   o check externo» no da identidad propia del revisor: los dos los puede
+   publicar el agente (L23, L29); lo da `autor-del-estado`. Texto propuesto: «agente = token de
    proyecto (Premium o self-managed) o cuenta de servicio (Free), rol
    Developer; persona = único Maintainer».
 5. **F1-7, decisión 3**: la línea `arbol=… perfil=… runner=… clave=… local=…
@@ -473,7 +496,7 @@ no se ha configurado nada en ninguna plataforma):
 | Requisito | Decisión | Evidencia |
 |---|---|---|
 | RR-15 / P11 mismas garantías; la diferencia documentada y dicha por `doctor` | tabla de equivalencias (decisión 2), `capacidades` (decisión 3), D2a | PoC G (20 filas, URL y fecha por celda, tier contra `fuentes.md`), C (9 planes, `doctor` con 5 líneas como mucho), M (misma línea en los 6 casos) |
-| P11 «algunas funciones de GitLab dependen del plan: el ADR lo verifica» | columnas de tier y sustitutos | documentación con fecha (43 URL en `fuentes.md`); empírico solo en GitLab.com de pago (GL-12, GL-17) y anónimo (401/403) |
+| P11 «algunas funciones de GitLab dependen del plan: el ADR lo verifica» | columnas de tier y sustitutos | documentación con fecha (44 URL en `fuentes.md`); empírico solo en GitLab.com de pago (GL-12, GL-17) y anónimo (401/403) |
 | RF-54 siempre por PR o MR, GitLab detrás del adaptador | puerto `Plataforma` con dos implementaciones (D1a) | PoC M (mismo `EstadoPuertas` neutral) |
 | RF-64 `doctor` con el arreglo | `arreglo` en cada garantía no impuesta | PoC C |
 | RN-01 identidad propia del agente | App en GitHub; token de proyecto o cuenta de servicio en GitLab (D4a) | D06, L11, L12; GL-09 (bots de proyecto actuando en MR); creación real: no verificada (F6-1) |
@@ -521,12 +544,14 @@ no se ha configurado nada en ninguna plataforma):
 - **GitLab self-managed** y versiones anteriores a GitLab.com del
   2026-11-14 (cuentas de servicio en Free desde 18.11).
 - **Que un estado de commit externo bloquee el merge** con «Pipelines must
-  succeed» en Free (L24 no lo dice), y que los checks de estado externos de
-  Ultimate solo los pueda responder el servicio configurado.
+  succeed» en Free (L24 no lo dice).
 - **Que el agente, como Developer, no pueda borrar una rama protegida ni
   mergear** con «Allowed to merge» Maintainers (documentado en L01 y L23).
-- **Si las cuentas de servicio cuentan** para el límite de 5 usuarios de
-  GitLab.com Free.
+- **Que el rol Developer baste para leer** `protected_branches` y
+  `approval_rules` con la identidad agente: las páginas de la API piden
+  sesión pero no nombran el rol mínimo; si exigen Maintainer, esas lecturas
+  de `estado-puertas` pasan a la identidad de persona (F1-6). Entra en la
+  batería E2E de F2-9.
 - **La configuración de CI desde otro proyecto** como sustituto de
   `pull_request_target` (divergencia 6).
 - **Las reviews de GitHub reales**: no hay ningún PR en el repo (GH-08).
diff --git a/docs/adr/README.md b/docs/adr/README.md
index a33ef79..865de32 100644
--- a/docs/adr/README.md
+++ b/docs/adr/README.md
@@ -20,7 +20,7 @@ directo a la tabla de la Fase 1 y a su tarea.
 | [F1-10](F1-10-carriles-y-modo-automatico.md) | Carriles y modo automático: cuatro carriles ordenados más release y hotfix por tipo, tabla única de política por carril con su función pura (`accion` en `siguiente --json`), configuración que solo endurece (`politica_carriles`, `rutas_carril`), carril mínimo por ficheros con los dos nombres de cada renombrado, y paradas fijas (`approve`, `merge-pr`, tag, tope de rondas, subida de carril) | TASK-011 | aceptada |
 | [F1-11](F1-11-extensiones-declaradas.md) | Extensiones declaradas: inventario comprobado de las 14 prácticas que los pilotos hacen a mano, clasificadas en parche de TaskCode (se retira al migrar) o propio del proyecto, con el desglose de líneas; tres claves (`verificacion`, `derivados`, `confianza_incluye`) con esquema que falla cerrado y rutas normalizadas; derivados que ejecuta `taskctl` en la transición solo en la rama de estado, comparados por contenido; check obligatorio `estado`; `taskctl tareas --json` como lectura estable; sin estados propios; revisoras como skill del proyecto; «comandos sí, hooks solo como atajo opcional»; y la ventaja medida frente a la 0.16, con lo que no gana | TASK-012 | aceptada |
 | [F1-12](F1-12-actualizacion-entre-versiones.md) | Actualización entre versiones: el plan de migración como dato (`tc2 actualizar --simular` sin cambiar un byte, mismo hash, vacío en la segunda pasada), qué migra (aplanado, formato 2, archivadas a `descartada`, config a los bloques de F1-11), historia con `git subtree split`, transacción local con criterio de salida estructural y vuelta atrás automática, retirada de parches con capacidades y huecos bloqueantes, convivencia con la 0.16 (su lector de config aborta con `---`: ninguna orden de escritura entra en un proyecto migrado), nombres definitivos `tc2` y `taskcodetwo`, y orden de pilotos OpenDockTrading → TaskCodeTwo → ToolsaWeb | TASK-013 | aceptada |
-| [F1-13](F1-13-adaptador-de-plataforma.md) | Adaptador de plataforma GitHub y GitLab: tabla de equivalencias de 20 garantías verificada contra la documentación oficial con fecha (y con GET de solo lectura donde se pudo), tier de cada una (en GitHub, 10 dependen del plan de los rulesets y la cola exige organización; en GitLab, 11 en Free, 6 en Premium, 1 en Ultimate y 2 en ningún tier), sustitutos declarados que `doctor` dice siempre, contrato de `capacidades` con el plan observado y no declarado, identidades en GitLab (token de proyecto o cuenta de servicio, persona único Maintainer, firmante por clave), llamadas por operación (`estado-puertas`: 4 en GitHub, 5 en GitLab) y misma batería para los dos adaptadores; tiers de pago, self-managed y comportamiento de las puertas no verificados | TASK-015 | aceptada |
+| [F1-13](F1-13-adaptador-de-plataforma.md) | Adaptador de plataforma GitHub y GitLab: tabla de equivalencias de 20 garantías verificada contra la documentación oficial con fecha (y con GET de solo lectura donde se pudo), tier de cada una (en GitHub, 10 dependen del plan de los rulesets y la cola exige organización; en GitLab, 11 en Free, 6 en Premium y 3 en ningún tier, entre ellas el origen de un check), sustitutos declarados que `doctor` dice siempre, contrato de `capacidades` con el plan observado y no declarado, identidades en GitLab (token de proyecto o cuenta de servicio, persona único Maintainer, firmante por clave), llamadas por operación (`estado-puertas`: 4 en GitHub, 5 en GitLab) y misma batería para los dos adaptadores; tiers de pago, self-managed y comportamiento de las puertas no verificados | TASK-015 | aceptada |
 | [F1-14](F1-14-distribucion-e-instalacion.md) | Distribución e instalación: CLI en Rust por la regla D1a (gana el 1.º y el 2.º, medidos, y el 4.º, un juicio sobre postject; sin el 4.º la regla no decide) frente a TypeScript con Node SEA (mismo comando, salida idéntica sobre las 404 tareas de ToolsaWeb y las 96 de OpenDockTrading; más rápido en caliente en los 3 repos y 32,1 veces más pequeño), TLS con el almacén del sistema, lanzador sin runtime con `cli.lock` y descarga comprobada por sha256, compatibilidad por protocolo con `tc2 version --json`, canales y firma por etapas (hash y attestation en la 2.0; Authenticode y Developer ID al entrar en winget y Homebrew) y contrato del manifiesto de recursos (`tc2 doctor --recursos`: obligatorio, necesario, conveniente y estorba); Linux y macOS no verificados | TASK-014 | aceptada |
 | [F1-18](F1-18-hito-de-decisiones.md) | Hito de decisiones (ADR) en `init` y en `actualizar`: estudio de adopción que lo dimensiona (bloque «uso medido» del informe de F1-12 y `proponer_hito` con un elemento por hueco, bloqueantes antes de migrar), proporcionalidad con mediana y p75 de la Fase 1 (45,5–50 min y 13,9–19,1 M tokens por ADR), registro `hito-decisiones.yml` en `taskcodetwo/tareas` cuya lista, descartes y «no necesario» firma una persona, con estado calculado que falla cerrado, y puerta determinista en `tc2 start` y en el check `estado` (código `rechazada`) | TASK-094 | aceptada |
````

## Excluido del diff (21 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 91a8078c162a537a1538137c29ffd2048c1e6a1b..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-015/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-015/planificacion/plan-final.md                 |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-015/poc/fixtures/github/aprobacion-bot.json     |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-015/poc/fixtures/github/esperando-checks.json   |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-015/poc/fixtures/github/plan.json               |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-015/poc/fixtures/github/sin-aprobacion.json     |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-015/poc/fixtures/github/sin-cambios.json        |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-015/poc/fixtures/github/verde.json              |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-015/poc/fixtures/gitlab/aprobacion-bot.json     |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-015/poc/fixtures/gitlab/esperando-checks.json   |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-015/poc/fixtures/gitlab/plan.json               |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-015/poc/fixtures/gitlab/sin-aprobacion.json     |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-015/poc/fixtures/gitlab/sin-cambios.json        |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-015/poc/fixtures/gitlab/verde.json              |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-015/poc/fixtures/observado.json                 |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-015/poc/fuentes.md                              |  13 ++-
 tareas/{02-en-curso => 03-en-revision}/TASK-015/poc/poc-f1-13.log                           |  65 ++++++-----
 tareas/{02-en-curso => 03-en-revision}/TASK-015/poc/poc-f1-13.py                            | 113 +++++++++++++++---
 tareas/03-en-revision/TASK-015/revision/informe-revision-1.md                               | 158 +++++++++++++++++++++++++
 tareas/03-en-revision/TASK-015/revision/peticion-revision-1.md                              | 635 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-015/tarea.md                                    |   8 +-
 21 files changed, 946 insertions(+), 46 deletions(-)
````

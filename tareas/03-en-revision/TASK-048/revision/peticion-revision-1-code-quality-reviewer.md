# Peticion de revision — TASK-048 (ronda 1)

- Tarea: TASK-048 — F6-2 review: abre el PR contra develop con la identidad del agente
- Rama revisada: feature/task-048-f6-2-review-abre-el-pr-contra-develop-co
- Rama base: develop
- Commit revisado (HEAD): a30688c63e099c28376a9d6dcd3f9ffd50f0a95d
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-048 (criterios de aceptacion y plan)
- Alcance: completa (motivo: primera ronda de revision)

## Instrucciones para el agente revisor

Eres un revisor INDEPENDIENTE del agente que implemento. Tu trabajo es
reproducir empiricamente, no leer el diff y opinar: clona el repo a un
directorio temporal, corre la suite tu mismo (salvo que esta
peticion ya traiga la verificacion de la ronda en verde para el commit revisado;
entonces vale ese resultado) y construye el caso que
rompe el codigo antes de reportarlo. Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. Vuelca tu salida en el
informe de esta ronda (informe-revision-1-code-quality-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-048/tarea.md
- tareas/03-en-revision/TASK-048/planificacion/
- docs/adr/F1-6-identidades-agente-revisor-persona.md

## Commits a revisar (git log develop..HEAD)

````
a30688c test(TASK-048): GitLab guarda la descripcion del MR sin el salto final
87fe06f feat(TASK-048): tc2 review abre el PR contra develop con la identidad del agente
1710a51 chore(TASK-048): tarea en curso
````

## Diff de tu dominio (1 fichero(s) de 18; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-6-identidades-agente-revisor-persona.md b/docs/adr/F1-6-identidades-agente-revisor-persona.md
index 57f8210..beda6f0 100644
--- a/docs/adr/F1-6-identidades-agente-revisor-persona.md
+++ b/docs/adr/F1-6-identidades-agente-revisor-persona.md
@@ -869,3 +869,113 @@ tiene que fallar: cierra «que GitHub impida a una App aprobar su propio PR»;
 en GitLab, si lo deja, no cuenta) y la persona aprueba hasta
 `aprobaciones=1`. Hasta ejecutarlos, esos puntos de «No verificado» siguen
 abiertos.
+
+## Adenda (F6-2, TASK-048) — `tc2 review` abre el PR con la identidad del agente
+
+Conecta la decisión 1 (`push` y PR con el agente) a un comando de fase:
+`tc2 review [--json] TASK-NNN` (`crates/tc2/src/revision/`). Sin
+capacidades de plataforma nuevas salvo `entorno-push`.
+
+- **Pasos**: `fetch`; lectura del estado; quién lo lanza (la regla de
+  `start`: quien lleva la tarea o cualquiera si no la lleva nadie; de otra
+  persona, `rechazada motivo=de-otra-persona asignado=<login>`); el paso a
+  revisión en seco (los rechazos salen antes de empujar nada); la rama de
+  `rama:` contra `develop`; su worktree registrado en
+  `.taskcodetwo/worktrees/TASK-NNN` (si no, `rechazada motivo=sin-worktree
+  arreglo="tc2 start TASK-NNN"`); el push desde ese worktree con el entorno
+  del agente; `abrir-pr` contra `develop`; `en-curso` → `en-revision` por el
+  punto único de escritura (operación `review|TASK-NNN|<id>|<login>|<instante>`,
+  exige red); y `revision-ia` leído con el agente.
+- **El estado, después del PR**: la transición se escribe solo con el PR
+  abierto, así el estado nunca dice «en revisión» sin PR. Si la transacción
+  falla, la línea lleva `pr=<n>` y repetir converge (push `sin-cambios`,
+  `abrir-pr sin-cambios`, transición `sin-cambios`).
+- **Línea**: `review ok id=TASK-NNN rama=<r> pr=<n> estado=en-revision
+  revision-ia=<ok|pendiente|fallo|sin-revisor|sin-leer>`; repetida sin nada
+  nuevo, `review sin-cambios ...`.
+- **`entorno-push` en la lista cerrada del puerto (F1-4)**:
+  `Plataforma::entorno_push(url_remoto) -> Paso<Vec<(OsString, OsString)>>`,
+  por defecto `rechazada motivo=no-soportada`. GitHub y GitLab lo
+  implementan con `entorno_git::entorno_push`: el token del agente
+  (`Conexion::token(.., Rol::Agente)`, con la App o `TC2_TOKEN_AGENTE`) solo
+  sale del adaptador como la cabecera `http.<url>.extraheader` limitada a la
+  URL de `origin`, sin autor. Solo `https://`: por ssh la cabecera no aplica
+  (empujaría la identidad local) y por `http://` el token iría en claro;
+  si no, `rechazada motivo=remoto-no-https`, sin pedir el token.
+- **`entorno_git` con autor opcional** (`Option<&AutorGit>`): `review` solo
+  empuja (sin `GET /app` para el slug); los commits del agente siguen con el
+  autor local (D2 del plan: el autor `[bot]` queda fuera de F6-2).
+- **«PR enlazado a la tarea» sin tocar el formato 2 (F1-9)**: título
+  `TASK-NNN: <título>` y cuerpo con las rutas
+  `taskcodetwo/tareas:TASK-NNN/tarea.md` y
+  `.../planificacion/plan-final.md`; el enlace tarea → PR ya lo da `board`
+  por la rama.
+- **D1 — check `revision-ia`**: `review` lo lee e informa; no lo publica (el
+  agente solo tiene `checks:read` y un check con su origen no contaría, G09).
+  Con `plataforma.revisor_id` en la configuración de la base, se pide
+  `estado-puertas` con el agente y un modelo con solo `revision-ia` y ese
+  origen: `fallo` si sale en `checks=` de un `check-rojo`, `pendiente` si
+  está en `falta=`, `ok` si no; sin `revisor_id`, `sin-revisor` sin
+  preguntar; si la lectura falla, `sin-leer`. Nunca cambia el resultado de
+  `review`. Lo publica el revisor en F6-3. Divergencia con el criterio
+  «check `revision-ia`» de la tarea: queda leído e informado, no en verde.
+- **D3 — carril**: solo `develop`; una rama `hotfix/` sale `rechazada
+  motivo=carril-no-soportado` hasta F7.
+
+**Divergencias con el plan de TASK-048**:
+
+1. **Suposición 1 falsa: `Git` no se componía por petición.**
+   `con_entorno_extra` consume el puerto y `Git` no es `Clone`. Se añade
+   `Git::derivado(&self, vars)`: un puerto nuevo con el binario y el entorno
+   del original más `vars`, y el registro vacío. El push corre con él; el
+   puerto de la transacción no lleva el token (el test lo comprueba en su
+   registro).
+2. **Suposición 3, a medias: `estado-puertas` no da el estado de un check
+   concreto**, solo las listas `falta=` (esperando) y `checks=` (rojo). Se
+   deduce de ellas con un modelo de un solo check (arriba).
+3. **URL de la cabecera = URL de `origin`** (`ls-remote --get-url`, la de
+   fetch con `insteadOf`). Un `remote.origin.pushurl` distinto no recibe la
+   cabecera: el push iría con lo que tenga git configurado para esa URL. El
+   puerto `Git` no tiene `remote get-url --push` y no se añade un
+   subcomando por esto; queda anotado.
+4. **El E2E no ejecuta `review_en` entero**: escribiría en la rama de estado
+   `taskcodetwo/tareas` del repo de pruebas. Ejecuta lo que `review` hace con
+   la plataforma, la misma función pública `revision::publicar` (push con
+   `entorno-push` y `abrir-pr` contra `develop`) y `revision::revision_ia`;
+   la transición de estado la cubre el test de integración con remoto bare.
+
+**Suposiciones del plan**:
+
+- `Git` se compone por petición sin afectar al de la transacción: **falsa**
+  tal cual; resuelta con `Git::derivado` (divergencia 1).
+- `git::remoto::push` funciona con `dir` = worktree de la tarea:
+  **verificado** (`tests/revision.rs`: la rama del bare queda en la cabeza
+  del worktree; repetir desde el worktree converge).
+- `estado_puertas` expone el estado de `revision-ia`: **parcial**
+  (divergencia 2), verificado con el unitario de `revision_ia`.
+
+**Evidencia** (`crates/tc2/tests/revision.rs`, unitarios de `revision/`
+y de `plataforma/entorno_git.rs`):
+
+- Con remoto bare, el worktree de `start` y una plataforma doble: el push
+  llega, `entorno-push` se pide para la URL de `origin`, `abrir-pr` recibe
+  la rama, `develop`, `TASK-001: Uno` y el cuerpo con las rutas, la tarea
+  queda `en-revision` con su fila `review` y repetir sale `sin-cambios` sin
+  mover la rama de estado.
+- Con `remote.origin.pushurl` en el servidor simulado, el push de `review`
+  manda `Authorization: Basic base64(x-access-token:<token>)` (el del
+  entorno de la plataforma), no abre PR ni cambia el estado al fallar, y el
+  registro del puerto de la transacción no tiene el token.
+- **Mutación**: empujar con `g.derivado(Vec::new())` en lugar del entorno
+  de la plataforma pone en rojo el test de la cabecera (`left: None`).
+
+**E2E ignorados, no ejecutados** (D4: los ejecuta quien orquesta antes de
+`finish`): `review_abre_el_pr_del_agente_contra_develop` en
+`tests/plataforma_e2e.rs` (`TC2_E2E_REPO`, `TC2_E2E_TOKEN`,
+`TC2_E2E_APP_ID`, `TC2_E2E_APP_CLAVE`) y
+`review_abre_el_mr_del_agente_contra_develop` en
+`tests/plataforma_gitlab_e2e.rs` (`TC2_E2E_GITLAB_PROYECTO`,
+`TC2_E2E_GITLAB_TOKEN`, `TC2_E2E_GITLAB_TOKEN_AGENTE`). Empujan con
+`entorno-push` del adaptador de producción, comprueban que el PR/MR lo abre
+el agente contra `develop` con el título y el cuerpo, que repetir converge y
+que `revision-ia` sale `pendiente`; cierran el PR/MR y borran la rama.
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-048/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-048/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-048/tarea.md                                    | 3 ++-
 3 files changed, 2 insertions(+), 1 deletion(-)
````

# Peticion de revision — TASK-037 (ronda 1)

- Tarea: TASK-037 — F4-2 approve con la identidad de la persona
- Rama revisada: feature/task-037-f4-2-approve-con-la-identidad-de-la-pers
- Rama base: develop
- Commit revisado (HEAD): b9a6a55f86af56fe615da1af20a1480ea2d9ae0a
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-037 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-037/tarea.md
- tareas/03-en-revision/TASK-037/planificacion/
- docs/adr/F1-10-carriles-y-modo-automatico.md
- docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
- docs/adr/F1-6-identidades-agente-revisor-persona.md

## Commits a revisar (git log develop..HEAD)

````
b9a6a55 chore(TASK-037): autorrevision
7b98d98 docs(TASK-037): adendas de approve en F1-6, F1-4 y F1-10
4256e92 feat(TASK-037): approve firmado con la identidad de la persona
42083d1 chore(TASK-037): tarea en curso
````

## Diff de tu dominio (3 fichero(s) de 17; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-10-carriles-y-modo-automatico.md b/docs/adr/F1-10-carriles-y-modo-automatico.md
index 60cfa04..201484a 100644
--- a/docs/adr/F1-10-carriles-y-modo-automatico.md
+++ b/docs/adr/F1-10-carriles-y-modo-automatico.md
@@ -724,3 +724,26 @@ Lo que el plan de TASK-036 no fijaba, o fija de otra forma:
 
 Evidencia: `crates/tc2/tests/plan.rs` y los unitarios de `carril.rs`
 (`roles_de_diseno_por_carril`) y de `crates/tc2/src/plan/`.
+
+## Adenda (F4-2, TASK-037) — `tc2 approve` y «pedir cambios»
+
+- **Parada fija**: `approve` no admite `--decidido-por` (ni `automatico` ni
+  otro): `uso motivo=decidido-por`. La fila es siempre `approve | <modo> |
+  persona`, con el modo congelado de la última fila `plan` (decisión 6; la
+  firma es de [F1-6](F1-6-identidades-agente-revisor-persona.md)).
+- **Carriles sin plan aprobado**: `approve` se admite en cualquier carril
+  con la tarea `en-diseno` (D2a: endurece, nunca baja).
+- **Pedir cambios cierra la ronda** (hueco de la adenda de F4-1, D1a):
+  `tc2 approve TASK-NNN --pedir-cambios <fichero>` publica
+  `brainstorm/cambios-pedidos-<n>.md` en la ronda mayor; con ese testigo la
+  ronda queda cerrada y el siguiente `plan`, aunque los roles sean los
+  mismos, abre la `n+1`. No concede nada, así que también lo puede lanzar
+  el agente. Divergencia con el plan de TASK-037: no escribe fila en
+  `## Transiciones` (como `plan guardar`, no es una fase; una fila `approve`
+  se leería como aprobación); quién lo pidió queda en la anotación del
+  commit. Repetir el mismo contenido es `sin-cambios`; otro contenido con la
+  ronda ya cerrada, `ronda-cerrada`.
+
+Evidencia: `crates/tc2/tests/aprobar.rs` y los unitarios de
+`crates/tc2/src/plan/aprobar.rs` y `operacion.rs`
+(`cambios_pedidos_cierran_la_ronda`).
diff --git a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
index c522d20..21f3391 100644
--- a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
+++ b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
@@ -11,6 +11,7 @@
 - Adenda (2026-11-17, TASK-035, F3-8): `doctor [--json]` compone lecturas sin escribir nada (ni `fetch`): `git version` entra en la lista permitida (decisión 3; solo lectura), y la lista cerrada de la plataforma gana `leer-plan` (detalle en la [adenda de F1-13](F1-13-adaptador-de-plataforma.md)). Salida: primera línea `doctor <ok|rechazada> fallos=N avisos=N no-comprobado=N plataforma=ok|no-configurada|sin-credenciales|sin-red|error [plan=… impuestas=N sustituidas=N no-disponibles=N [no-comprobadas=N]] [commit=…]` (`no-comprobadas` solo si alguna garantía cuya configuración se lee no se pudo leer: no cuenta como impuesta; ronda 1 de TASK-035); detrás, los hallazgos que no son `ok` (`<zona> <id> <falla|aviso|no-comprobado> [motivo=…] [claves] [arreglo=…]`), los peores primero, y si no caben en 5 líneas, 3 y `y N mas: tc2 doctor --json`. `--json`: una línea con las claves y `hallazgos` (todos, también los `ok`). Código 3 (`rechazada`) con alguna `falla` (D5a de TASK-035) y 0 con avisos o no-comprobados; fuera de un repo, `doctor rechazada motivo=no-es-repo` (3); argumentos de más, `uso motivo=argumentos` (2; `--recursos` llega con F3-9). Git por debajo de 2.56: `aviso motivo=git-no-verificado` (D3a de TASK-035); sin git o con una versión que no se entiende, `falla`. Diagnóstico de una línea en stderr solo con alguna `falla`
 - Adenda (2026-11-17, TASK-079, F3-5b): `status`, `board`, `next` y `roadmap` (y `roadmap --comparar`) traen antes la rama de estado con el mismo `fetch origin refs/heads/taskcodetwo/tareas:refs/remotes/origin/taskcodetwo/tareas` de la transacción y de `montar` (sin `-q`, para que git no calle el rechazo non-fast-forward; sin operaciones nuevas, sin cerrojo) y solo escriben esa ref remota y `.git/taskcodetwo/sincronizacion`. Claves nuevas en las cuatro, detrás de las que ya tenían: `remoto=consultado|sin-red|rechazado|denegado|error-local|sin-rama|sin-remoto` (cómo fue el `fetch`; sin `origin` no se hace; `rechazado`: la rama de estado de `origin` está reescrita y no desciende de la traída; `error-local` incluye la ref remota bloqueada por un `.lock`, que la lectura reintenta dos veces antes y nombra en su aviso; una escritura que encuentra la rama reescrita sale `error` con `motivo=rama-reescrita`, y la ref bloqueada, `error` con `motivo=error-local`, sin quedar pendiente) y `sincronizado=<AAAA-MM-DDTHH:MM:SSZ>|nunca` (hora UTC del último `fetch` con éxito, de una lectura, de una transacción o de `montar`; una marca con hora posterior a ahora cuenta como `nunca`). Un fallo del remoto no cambia el resultado: sale `ok` (0) con lo último traído y un aviso de una línea en stderr con la hora y «hace N» (o que no se sincronizó nunca). `status` gana además `pendientes=N` (refs de `refs/taskcodetwo/pendientes/`) y, si la carpeta montada no se puede adelantar, `tareas=con-cambios|diverge` con su aviso (código 0). En texto, la hora va al final de la línea de `status` (`· sincronizado …`, seguida de `· pendientes N` solo si N > 0) y de la cabecera de `board`; en `roadmap`, a stderr (`estado sincronizado con origin el … (hace …)`), porque su stdout sigue siendo el `ROADMAP.md` byte a byte. `doctor` no hace `fetch` y añade `sincronizado=` (de la marca) detrás de `commit=`
 - Adenda (2026-11-18, TASK-083, F3-13): `log` entra en el puerto `Git` (ya estaba en la lista permitida de la decisión 3; solo lectura): un único `git log --format=%H%x1f%P%x1f%ct%x1f%s <origin/develop | develop> --` por `new`, `import`, `start` y `tc2 estimar` para el histórico de la estimación (F1-17). `new ok` y `start ok` ganan al final `estimacion=<mediana total en min>|sin-datos`, `estimacion-p80=<min>` (solo con datos), `estimacion-n=<n>` y `estimacion-nivel=<nivel>`; `import` no (cada `tarea.md` lleva la suya). Orden nueva `tc2 estimar [--json] [TASK-NNN | --error TASK-NNN]`, que no es una operación Git (no choca con D1a): sin argumentos, transacción de la rama de estado que estima las tareas abiertas de formato 2 con `estimacion: null` (`estimar ok actualizadas=N ids=… commit=… intentos=… tareas=…`, `estimar sin-cambios actualizadas=0`, sin red `sinred pendiente=no`); con `TASK-NNN`, solo lectura del estado conocido (sin `fetch`): `estimar ok id=… estimacion=… [estimacion-p80=…] estimacion-n=… estimacion-nivel=…` y, sin `--json`, una segunda línea para la persona `estimacion: ≈M min (p80 P), n=K, nivel=X` o `estimacion: sin tareas parecidas (n=k<5)`; con `--error TASK-NNN`, solo lectura: `estimar ok id=… error-<fase>=<min> relativo-<fase>=<0.00> razon-<fase>=<0.00>…` (fases con cifra y `total`) y la línea `error: …`; rechazos (3) `no-existe`, `formato-1`, `ilegible`, `sin-estimacion` («sin estimacion guardada», sin cifras), `estimacion-ilegible`, `sin-cifras` (guardada `sin-datos`) y `sin-real` (con `causa=` del descarte); `uso` (2) `sin-id`, `id-invalido`, `argumentos-de-mas`
+- Adenda (2026-11-19, TASK-037, F4-2): `commit-tree -S` entra en el puerto `Git` como única forma firmada (`Git::ejecutar_firmado`, que pone el `-S` ella misma, hereda la entrada estándar para la frase de paso y captura stdout y stderr); en cualquier otra llamada `-S`, `-S<clave>` y `--gpg-sign[=<clave>]` de `commit-tree` se rechazan sin lanzar nada, y el texto tras `-m` de `commit-tree` no se mira como opción. Solo la usa `approve` (`Sello::Firmado` de la transacción, detalle en la [adenda de F1-6](F1-6-identidades-agente-revisor-persona.md)). `approve [--json] TASK-NNN [--pedir-cambios <fichero>]` compone `fetch` y `commit-estado`; línea `approve ok id=… commit=… intentos=… tareas=…` (con `--pedir-cambios`, además `cambios-pedidos=cambios-pedidos-<n>.md ronda=<n>`); motivos de `rechazada` (3): `identidad` (`rol=persona`), `firma`, `plan-esqueleto`, `sin-plan`, `pausada`, `estado-no-valido`, `de-otra-persona`, `no-existe` y, con `--pedir-cambios`, `sin-ronda`, `ronda-cerrada` y `plan-aprobado`; `uso` (2): `sin-id`, `id-invalido`, `sin-fichero`, `decidido-por`, `argumentos-de-mas`
 - Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
diff --git a/docs/adr/F1-6-identidades-agente-revisor-persona.md b/docs/adr/F1-6-identidades-agente-revisor-persona.md
index 594cbd6..62b8995 100644
--- a/docs/adr/F1-6-identidades-agente-revisor-persona.md
+++ b/docs/adr/F1-6-identidades-agente-revisor-persona.md
@@ -646,3 +646,42 @@ como riesgos sin prueba:
   quién mantiene la lista de personas (F1-10).
 - **GitLab**: si un token de proyecto puede quedar fuera de los aprobadores y
   publicar un estado que la regla de merge exija (F1-13).
+
+## Adenda (F4-2, TASK-037) — `tc2 approve` firmado con `Plan-Blob`
+
+`tc2 approve [--json] TASK-NNN` implementa la consecuencia de F4-2 de este
+ADR. Lo que fija la implementación, con su evidencia en
+`crates/tc2/tests/aprobar.rs` y los unitarios de `crates/tc2/src/plan/`:
+
+- **Identidad**: solo `Quien::Persona` (terminal, `GET /user` con el token de
+  persona y su `id` en la lista de personas de `origin/develop`). Con el
+  agente o con cualquier rechazo de la identidad, `approve rechazada
+  motivo=identidad rol=persona` antes de lanzar git (ni `fetch`, ni
+  transacción, ni `commit-tree`).
+- **Firma**: el commit de `approve` sale de `git commit-tree -S` (la única
+  forma firmada del puerto, adenda de TASK-037 en F1-4) con `gpg.format` y
+  `user.signingkey` de la configuración de la persona, y lleva el trailer
+  `Plan-Blob: <blob>` con el blob de `TASK-NNN/planificacion/plan-final.md`
+  en el árbol de ese mismo commit (`rev-parse <árbol>:<ruta>`). Si la firma
+  falla, `rechazada motivo=firma` y no se publica nada. En cada reintento
+  del CAS se vuelve a firmar. Las pendientes no entran en el commit firmado
+  (D4a): se publican con la siguiente operación normal.
+- **Medido en Windows (Git 2.56.0.windows.4)**: `commit-tree -S` firma con
+  `gpg.format=ssh` y una clave ed25519 desechable sin frase, y `git
+  verify-commit` la valida con `gpg.ssh.allowedSignersFile`. `commit-tree`
+  **no** respeta `commit.gpgSign=true` (es plumbing): ninguna otra operación
+  empieza a firmar por la configuración de la persona, y `approve` pasa `-S`
+  explícito. Sin `user.signingkey` git sale con 128; con una clave que no
+  existe, o con frase de paso y sin terminal, con 1 (los tres, `motivo=firma`).
+- **Terminal**: la entrada estándar de `commit-tree -S` se hereda, para que
+  `ssh-keygen` pueda pedir la frase de paso; stderr se captura (git ya
+  captura el de `ssh-keygen`, así que heredarlo no enseñaría el aviso FIDO) y
+  su primera línea va al diagnóstico. **No verificado**: la petición de la
+  frase de paso y el toque FIDO con una persona real en el terminal de
+  Windows.
+- **Ya aprobado**: `sin-cambios`, sin firmar otra vez. El esqueleto de
+  `plan-final.md` no se aprueba (`plan-esqueleto`); tampoco sin plan
+  (`sin-plan`), en pausa (`pausada`), fuera de `en-diseno`
+  (`estado-no-valido`) o de otra persona (`de-otra-persona`).
+- La comprobación de la firma frente a la plataforma (reglas 1 a 5 de la
+  decisión 3) sigue siendo del check `carril` (F6), no de `approve` (D3a).
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-037/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-037/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-037/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````

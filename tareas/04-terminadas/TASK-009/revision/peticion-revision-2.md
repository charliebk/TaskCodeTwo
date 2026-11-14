# Peticion de revision — TASK-009 (ronda 2)

- Tarea: TASK-009 — F1-8 ADR seguridad
- Rama revisada: feature/task-009-f1-8-adr-seguridad
- Rama base: develop
- Commit revisado (HEAD): fb9e2a9cf4f4761ed65e2f3c1f7f13405ee2607f
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-009 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde cfdcce690f213a71214d169c31c710d25dec6160 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (6 hallazgo(s)))

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

- tareas/03-en-revision/TASK-009/tarea.md
- tareas/03-en-revision/TASK-009/planificacion/
- docs/adr/F1-8-seguridad-modelo-de-amenazas.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 3 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:74 | informe-revision-1.md |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:80 | informe-revision-1.md |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:375 | informe-revision-1.md |

## Commits a revisar (git log cfdcce690f213a71214d169c31c710d25dec6160..HEAD)

````
fb9e2a9 chore(TASK-009): constancia de autorrevision ronda 2
19f74bb docs(adr): F1-8 corrige los hallazgos de la revision 1 (A7 parcial, lint heuristico, salvedad A4/A5, menores)
a9da7bd chore(TASK-009): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff cfdcce690f213a71214d169c31c710d25dec6160..HEAD)

````diff
diff --git a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
index 9fb01ef..2d901cf 100644
--- a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
+++ b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
@@ -58,8 +58,9 @@ entrada a la vez del hook, del workflow y del LLM.
 
 Gravedad: **CRÍTICA** (se salta una puerta o filtra un secreto sin vuelta
 atrás), **ALTA** (verde falso en un check obligatorio), **MEDIA** (pérdida de
-trabajo o de estado). Cada fila tiene mitigación y tarea dueña; lo que no se
-puede comprobar con `GET` lleva «no verificado → F2-6». La PoC C lee esta
+trabajo o de estado). Cada fila tiene mitigación **o un riesgo aceptado
+nombrado** (`A5`, `A15`), y tarea dueña; lo que no se puede comprobar con
+`GET` lleva «no verificado → F2-6». La PoC C lee esta
 tabla entre los marcadores `matriz-amenazas` y comprueba cada fila.
 
 <!-- matriz-amenazas:inicio -->
@@ -69,15 +70,15 @@ tabla entre los marcadores `matriz-amenazas` y comprueba cada fila.
 | `A2` | Clave privada de la App del revisor | Z3 | solo el job publicador de `revision-ia` (clase privilegio) | código del PR la lee si comparte job con él y publica `revision-ia` en verde como revisor (CRÍTICA) | secreto de entorno solo en el job publicador, que no ejecuta nada del PR (decisiones 1 y 3); entorno limitado a las ramas base; rotación (decisión 8) | F6-1, F6-3, no verificado → F2-6 |
 | `A3` | Clave del modelo del revisor IA | Z3 | solo el job publicador (D2b) | código del PR la roba en el job de reproducción (CRÍTICA) | la clave vive en el job publicador; la reproducción corre en otro job sin secretos y le pasa un artefacto que se lee como dato (decisión 3) | F6-3, F6-4, no verificado → F2-6 |
 | `A4` | Clave de firma de la persona | Z1 | `approve` (F4-2) con TTY | el agente, en la misma sesión, firma un `approve` falso si la clave está en un `ssh-agent` o llavero desbloqueado (CRÍTICA, sin vuelta atrás: `taskcode/tareas` no admite borrado) | FIDO con toque (D4a de F1-6) o frase sin caché (D4b de F1-6); nunca cargada en un agente de claves sin confirmación por uso; `doctor` avisa si el tipo de clave registrada no es `sk-` ni tiene frase (no comprobable desde la plataforma); rotación (decisión 8) | F4-2, F3-8, no verificado → F2-6 |
-| `A5` | Token de la persona | Z1 | `merge-pr`, `aplicar-puertas`, `leer-permisos-actions` (F1-6) | el agente lo saca del llavero del mismo usuario del sistema (`gh auth token`) y aprueba o mergea como la persona (CRÍTICA) | F1-6 lo limita a operaciones con TTY, pero eso lo impone el CLI, no la plataforma; queda como riesgo abierto (ver «Preguntas abiertas»): el agente en otro usuario del sistema o sin acceso al llavero, o un token de la persona que no queda guardado | F2-6, F3-8, no verificado → F2-6 |
-| `A6` | `GITHUB_TOKEN` de los jobs que ejecutan código del PR | Z3 | el job y todo lo que el job ejecuta | scripts del PR (`verificar.bat`, `build.rs`, `postinstall`) usan un token con `checks:write` para crear `carril` o `verificacion` en verde (ALTA) | `permissions: contents: read` explícito en todo job de clase ejecuta (decisión 1); permisos por defecto de Actions en solo lectura, que `doctor` exige (leído: `read` en los tres repos, PoC D); lint estático de clases (PoC B) | F5-1, F3-8, no verificado → F2-6 |
-| `A7` | Check `verificacion` y su caché `arbol=` | Z3 | el job publicador de `verificacion` escribe la línea; `buscar` la lee | un check o una línea falsa en el head del PR deja un verde reutilizable para un árbol que no se verificó (ALTA, persistente) | el código del PR corre en un job ejecutor sin escritura; la línea `arbol=` solo cuenta si la publicó el job publicador del workflow de TaskCode, que calcula el árbol él mismo (D5a, divergencia con F1-7) | F5-1, F5-2, no verificado → F2-6 |
-| `A8` | Check `carril` | Z3 | el workflow `carril` (clase privilegio) | con `pull_request`, un workflow cambiado en el propio PR saca `carril` en verde antes de que la persona lo vea (ALTA) | `carril` en `pull_request_target`, sin checkout del head; mitigación **parcial** (D1b): `pull_request_review` corre el workflow del ref del PR y la migración convive un momento con el viejo | F5-1, F6 (`carril` sin fila en el plan), no verificado → F2-6 |
+| `A5` | Token de la persona | Z1 | `merge-pr`, `aplicar-puertas`, `leer-permisos-actions` (F1-6) | el agente lo saca del llavero del mismo usuario del sistema (`gh auth token`) y aprueba o mergea como la persona (CRÍTICA) | F1-6 lo limita a operaciones con TTY, pero eso lo impone el CLI, no la plataforma; queda como riesgo abierto (ver «Preguntas abiertas»): el agente en otro usuario del sistema o sin acceso al llavero, o un token de la persona que no queda guardado | decide un ADR nuevo de Fase 1, «aislamiento del agente frente a las credenciales de la persona» (sin fila en el plan: acción en «Consecuencias»); implementan F2-6, F3-8; no verificado → F2-6 |
+| `A6` | `GITHUB_TOKEN` de los jobs que ejecutan código del PR | Z3 | el job y todo lo que el job ejecuta | scripts del PR (`verificar.bat`, `build.rs`, `postinstall`) usan un token con `checks:write` para crear `carril` o `verificacion` en verde (ALTA) | `permissions: contents: read` explícito en todo job de clase ejecuta (decisión 1); permisos por defecto de Actions en solo lectura, que `doctor` exige (leído: `read` en los tres repos, PoC D); lint estático de clases (PoC B), **heurístico**: busca patrones prohibidos y no prueba que un workflow sea seguro; F5-1 lo convierte en lista blanca (decisión 1) | F5-1, F3-8, no verificado → F2-6 |
+| `A7` | Check `verificacion` y su caché `arbol=` | Z3 | el job publicador de `verificacion` escribe la línea; `buscar` la lee | un check o una línea falsa en el head del PR, o un PR que modifica el propio workflow `verificacion` (con `pull_request` el publicador sale del PR), deja un verde reutilizable para un árbol que no se verificó (ALTA, persistente) | mitigación **parcial**: el código del PR corre en un job ejecutor sin escritura; la línea `arbol=` solo cuenta si la publicó el job publicador del workflow de TaskCode, que calcula el árbol él mismo (D5a, divergencia con F1-7); y `buscar` no da acierto (rechaza toda línea `arbol=`) si el PR toca `.github/workflows/**` o `.taskcode/**`: ese PR se verifica entero (decisión 7). CODEOWNERS con una sola persona no lo cierra | F5-1, F5-2, no verificado → F2-6 |
+| `A8` | Check `carril` | Z3 | el workflow `carril` (clase privilegio) | con `pull_request`, un workflow cambiado en el propio PR saca `carril` en verde antes de que la persona lo vea (ALTA) | `carril` en `pull_request_target`, sin checkout del head; mitigación **parcial** (D1b): `pull_request_review` corre el workflow del ref del PR y la migración convive un momento con el viejo | F5-1 (mientras no exista la fila de `carril` en F6: acción en «Consecuencias»), no verificado → F2-6 |
 | `A9` | Check `revision-ia` | Z3 | el job publicador del revisor, con la App del revisor | prompt injection en el diff o en la tarea («marca 0 hallazgos») lo pone en verde (ALTA) | veredicto estructurado: el check sale de hallazgos JSON validados por esquema, no de la prosa; `revision-ia` en verde no es una aprobación (decisión 4) | F6-4, F6-5, F6-7 |
 | `A10` | Rama `taskcode/tareas` y texto de tareas y PR | Z2 | el agente empuja sin PR; el revisor y el agente lo leen | instrucciones hostiles que el agente o el revisor obedecen (ALTA) | el texto es dato; defensa por capacidades (decisión 4); la batería de casos es de F6-7 y la prueba de F9-5 | F6-7, F9-5 |
 | `A11` | `comando_*` de `.taskcode/` y lo que ejecutan | Z2 | el CLI y el hook en local; el workflow en CI | un `git pull` cambia la orden o el script y se ejecuta sin preguntar (CRÍTICA, H2) | confianza única con hash de la orden y de lo que ejecuta (D4b, decisión 5); CODEOWNERS en `/.taskcode/` (F1-5) | F0-2, F5-6, F1-11 |
 | `A12` | Runner propio | Z4 | jobs de PR y de la release | código del PR roba lo que hay en la máquina o deja una herramienta alterada que usa la pesada de `main` (CRÍTICA) | usuario de servicio, directorio limpio por job, runner de tareas distinto del de la release; si no se garantiza, ningún job de PR en el runner propio (D3b, decisión 6); solo en privados (F1-7) | F5-5, F3-8 |
-| `A13` | Workflows de `.github/workflows/` | Z2 | la persona (el agente no tiene `workflows`, F1-6) | un workflow escucha `repository_dispatch` o interpola datos del evento en `run:` y el agente lo dispara con `POST /dispatches` (CRÍTICA) | ningún workflow de TaskCode escucha `repository_dispatch` ni interpola `github.event.*` o `github.head_ref` en `run:`; lo comprueba el lint (PoC B) y `doctor` | F5-1, F3-8 |
+| `A13` | Workflows de `.github/workflows/` | Z2 | la persona (el agente no tiene `workflows`, F1-6) | un workflow escucha `repository_dispatch` o interpola datos del evento en `run:` y el agente lo dispara con `POST /dispatches` (CRÍTICA) | ningún workflow de TaskCode escucha `repository_dispatch` ni interpola `github.event.*` o `github.head_ref` en `run:`; lo comprueba el lint (PoC B, heurístico: ver «No verificado») y `doctor`; F5-1 lo convierte en lista blanca de eventos y usos de checkout | F5-1, F3-8 |
 | `A14` | Rulesets y ajustes de Actions | Z5 | la persona como administradora | se relaja un ajuste (permisos por defecto en escritura, `can_approve_pull_request_reviews`, workflows de forks en privados) (ALTA) | `doctor` los lee con `GET` y los compara con el modelo (PoC D); `init` los aplica | F3-8, F3-1 |
 | `A15` | Worktrees y estado local sin commitear | Z1 | el CLI y la persona | un script del PR lanza `git clean -ffdx` o borra el worktree de `tareas/` en una verificación local (MEDIA) | ninguna; riesgo aceptado y nombrado (ver «Preguntas abiertas»); la confianza (D4b) reduce cuándo corre código del repo | F5-6, F1-16 |
 | `A16` | Lote desatendido (RR-18) | Z1 | el CLI encadena tareas con el agente | una tarea con texto hostil lleva a las siguientes del lote (ALTA) | el lote nunca integra en `develop` sin el PR final que aprueba la persona (P13); las mismas capacidades que fuera de lote | F1-16, F4-7 |
@@ -92,10 +93,11 @@ una sola clase:
 | Clase | Qué hace | Eventos | `permissions` | Secretos | Entrada del PR |
 |---|---|---|---|---|---|
 | **Ejecuta código del PR** | `verificacion` (job ejecutor), reproducción y mutantes del revisor (F6-4), `comando_sincronizacion` en CI (RF-70) | `pull_request` (y `merge_group`) | `contents: read` y nada más | ninguno | se ejecuta |
-| **Tiene privilegio** | publica `verificacion` (job publicador), `carril`, publica `revision-ia` con la App del revisor | `pull_request_target` o `workflow_run`; el publicador de `verificacion`, en el mismo `pull_request` como job aparte | el mínimo de su escritura | solo los suyos, de entorno | **solo como datos** (diff por la API, artefactos validados); nunca checkout del head ni ejecución de nada del PR |
+| **Tiene privilegio** | publica `verificacion` (job publicador), `carril`, publica `revision-ia` con la App del revisor | `pull_request_target` o `workflow_run`; el publicador de `verificacion`, en el mismo `pull_request` como job aparte (por eso su workflow sale del PR y la mitigación es parcial: decisión 7) | el mínimo de su escritura | solo los suyos, de entorno | **solo como datos** (diff por la API, artefactos validados); nunca checkout del head ni ejecución de nada del PR |
 
 Un job que ejecuta código del PR **y** tiene privilegio es un error de
-diseño, lo detecta el lint (PoC B) y `doctor` lo rechaza.
+diseño; el lint (PoC B, heurístico) detecta los patrones conocidos y
+`doctor` lo rechaza.
 
 ## Alternativas
 
@@ -137,19 +139,20 @@ tabla de preguntas heredadas se leen de este fichero**, entre los marcadores
 de 0 si alguna sección falla.
 
 ```bash
-bash poc-f1-8.sh "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-009/poc" \
+bash poc-f1-8.sh "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-009-c1" \
   docs/adr/F1-8-seguridad-modelo-de-amenazas.md \
   /c/Users/nullcad2025/Documents/sources/Personal > poc-f1-8.log 2>&1
 ```
 
-Resultado de la ejecución registrada en el log (2026-11-14T08:15:45Z, salida 0):
+Resultado de la ejecución registrada en el log, tras la revisión 1 (2026-11-14T08:27:41Z,
+en `...\task-009-c1\`, borrada al terminar; salida 0):
 
 ```text
 RESULTADO A1 casos=7/7 ; controles negativos=8/8 ; limite documentado=1/1 ; almacen fuera del arbol=si
 RESULTADO A2 ordenes medidas=4 ; sin orden o cierre vacio=0
-RESULTADO B1 positivos sin errores y con su clase=5/5 ; controles negativos=14/14
-RESULTADO B2 casos D5a=7/7 ; falsos que la regla de F1-7 aceptaria=2
-RESULTADO C errores ADR real=0 ; activos=16 ; preguntas heredadas=13 ; mutaciones detectadas=12/12 ; no aplicadas=0
+RESULTADO B1 positivos sin errores y con su clase=5/5 ; controles negativos=20/20
+RESULTADO B2 casos decision 7=10/10 ; falsos que aceptaria la regla de F1-7=4 ; solo job publicador=2
+RESULTADO C errores ADR real=0 ; activos=16 ; preguntas heredadas=13 ; mutaciones detectadas=14/14 ; no aplicadas=0
 RESULTADO D comprobaciones=16/16
 RESULTADO llamadas gh api con GET=15 ; gh api con otro metodo=0
 ```
@@ -194,10 +197,10 @@ cierre (cada uno obliga a volver a confiar):
 
 ```text
   repo             clave                       nombrados  cierre      bytes  ms (max)  commits
-  ToolsaWeb        comando_verificacion                1      51    1246280     752,5        8
-  ToolsaWeb        comando_verificacion_freno          1      51    1246280     868,0        8
-  ToolsaWeb        comando_sincronizacion              1      51    1246280     731,8        8
-  OpenDockTrading  comando_verificacion                1      19     731381     421,1        9
+  ToolsaWeb        comando_verificacion                1      51    1246280     778,2        8
+  ToolsaWeb        comando_verificacion_freno          1      51    1246280     763,7        8
+  ToolsaWeb        comando_sincronizacion              1      51    1246280     928,8        8
+  OpenDockTrading  comando_verificacion                1      19     731381     451,2        9
 ```
 
 Lectura: la orden nombra un solo fichero y lo que ejecuta son 51 scripts en
@@ -212,10 +215,14 @@ presupuesto de 1 s del hook de F5-6. Aviso del guion: con YAML estándar,
 
 **B1.** Lint de clases (prototipo del de F5-1 y `doctor`) sobre los cuatro
 workflows de la sección «Cómo se ve en GitHub», el `ci.yml` real de este repo
-y 14 workflows con un fallo cada uno. Un job es de clase ejecuta si hace
-checkout en un evento de PR, hace checkout del head en `pull_request_target`,
-ejecuta un fichero de un artefacto o corre en `pull_request_review`; tiene
-privilegio si escribe, usa secretos o un entorno.
+y 20 workflows con un fallo cada uno (N15–N20 son las variantes que la
+revisión 1 encontró sin detectar). Un job es de clase ejecuta si hace
+checkout en un evento de PR (o llama a un workflow reutilizable en él), hace
+checkout del head por `ref:` o `repository:`, trae el head en `run:` (`gh pr
+checkout`, `git fetch … pull/`) fuera de un evento de PR, ejecuta un fichero
+de un artefacto o corre en `pull_request_review`; tiene privilegio si
+escribe, usa secretos, un entorno o `secrets: inherit`. Es **heurístico**:
+busca patrones conocidos (ver «No verificado»).
 
 ```text
   ok   pos  P1-verificacion            clases={'buscar': 'ejecuta', 'ejecutar': 'ejecuta', 'verificacion': 'privilegio'} errores=0
@@ -237,24 +244,39 @@ privilegio si escribe, usa secretos o un entorno.
   ok   neg  N12 pull_request con un secreto en env: del job    -> R1
   ok   neg  N13 repository_dispatch en la lista de eventos     -> R4
   ok   neg  N14 un job sin permissions junto a otro que si     -> R2
+  ok   neg  N15 pull_request_target, checkout con repository: del head -> R1
+  ok   neg  N16 pull_request_target, gh pr checkout con el numero por env: -> R8 (y R1)
+  ok   neg  N17 titulo del PR en actions/github-script          -> R5 (en script:)
+  ok   neg  N18 github['event']['pull_request']['title'] en run: -> R5
+  ok   neg  N19 toJSON(github.event) en run:                    -> R5
+  ok   neg  N20 workflow reutilizable con secrets: inherit       -> R9 (y R1)
 ```
 
-**B2.** Búsqueda de la caché con la regla de F1-7 (nombre, App 15368, verde y
-`arbol=` en el resumen) frente a D5a (además, el check-run es el job
-publicador del workflow de TaskCode):
+**B2.** Búsqueda de la caché con tres reglas: la de F1-7 (nombre, App 15368,
+verde y `arbol=` en el resumen), «solo job» (además, el check-run es el job
+publicador del workflow de TaskCode; la primera versión de D5a) y la
+decisión 7 tras la revisión 1 (además, el PR no toca `.github/workflows/**`
+ni `.taskcode/**`):
 
 ```text
-  ok   F1-7 acepta   D5a acepta   publicador legitimo (control positivo)
-  ok   F1-7 acepta   D5a rechaza  check-run creado por la API desde un script del PR con checks:write
-  ok   F1-7 rechaza  D5a rechaza  linea escrita en el resumen del job ejecutor
-  ok   F1-7 acepta   D5a rechaza  job verificacion de otro workflow anadido en el PR
-  ok   F1-7 rechaza  D5a rechaza  check verificacion de otra App
-  ok   F1-7 rechaza  D5a rechaza  publicador con otro arbol
-  ok   F1-7 rechaza  D5a rechaza  publicador en rojo
+  ok   F1-7 acepta   solo-job acepta   decision-7 acepta   publicador legitimo (control positivo)
+  ok   F1-7 acepta   solo-job rechaza  decision-7 rechaza  check-run creado por la API desde un script del PR con checks:write
+  ok   F1-7 rechaza  solo-job rechaza  decision-7 rechaza  linea escrita en el resumen del job ejecutor
+  ok   F1-7 acepta   solo-job rechaza  decision-7 rechaza  job verificacion de otro workflow anadido en el PR
+  ok   F1-7 rechaza  solo-job rechaza  decision-7 rechaza  check verificacion de otra App
+  ok   F1-7 rechaza  solo-job rechaza  decision-7 rechaza  publicador con otro arbol
+  ok   F1-7 rechaza  solo-job rechaza  decision-7 rechaza  publicador en rojo
+  ok   F1-7 acepta   solo-job acepta   decision-7 rechaza  publicador de un workflow modificado en el PR (toca .github/workflows/verificacion.yml)
+  ok   F1-7 acepta   solo-job acepta   decision-7 rechaza  publicador con el PR tocando .taskcode/config.yml
+  ok   F1-7 acepta   solo-job acepta   decision-7 acepta   publicador con el PR tocando solo codigo (control positivo)
 ```
 
-Lectura: la regla de F1-7 aceptaría dos verdes falsos; D5a, solo el del
-publicador. Es una simulación: cómo se identifica el job de un check-run en
+Lectura: la regla de F1-7 aceptaría cuatro verdes falsos y la de «solo job»
+dos: un publicador de un workflow modificado en el PR tiene los mismos
+atributos que el legítimo (revisión 1, IMP-1). La decisión 7 no da acierto
+en esos PR y los verifica enteros; sigue siendo parcial (un PR que no toca
+esas rutas no cambia el publicador, pero la regla depende de leer bien el
+diff). Es una simulación: cómo se identifica el job de un check-run en
 la API real queda «no verificado».
 
 ### C — El propio ADR
@@ -266,11 +288,14 @@ heredados obligatorios están; F1-5, F1-6 y F1-7 aparecen como origen (citan
 F1-8 en 5, 6 y 7 líneas); los requisitos de la cabecera existen en el
 catálogo y en la auditoría; están las secciones de la plantilla y una
 decisión por cada parte del enunciado; hay fila en el índice; no hay enlaces
-a `tareas/`. Doce copias mutadas fallan por el error esperado (12/12): dueña
+a `tareas/`; `A7` y `A8` se declaran parciales y la tabla de la decisión 4
+lleva la salvedad A4/A5. Catorce copias mutadas fallan por el error esperado
+(14/14): dueña
 vacía, dueña inexistente (`F5-99`), zona `Z9`, fila del runner propio
 borrada, pregunta de `repository_dispatch` borrada, mitigación vacía,
 requisito `RR-99`, enlace a `tareas/`, gravedad quitada, divergencia con
-F1-7 quitada, decisión del runner propio quitada y marcador borrado.
+F1-7 quitada, decisión del runner propio quitada, marcador borrado, `A7` sin
+«parcial» y salvedad A4/A5 quitada.
 
 ### D — Ajustes reales (solo `GET`)
 
@@ -307,6 +332,15 @@ F1-7 quitada, decisión del runner propio quitada y marcador borrado.
    divergencia 4 con F1-6).
 6. **`taskctl doctor`** no lo ejecuta esta implementación (se le pidió no
    ejecutar `taskctl`); `comprobar-repo.sh` sí, en verde.
+7. **Revisión 1** (decidido por Carlos con la consigna «la opción
+   recomendada»): `A7` pasa a parcial y `buscar` no da acierto en un PR que
+   toca workflows o `.taskcode/` (IMP-1, PoC B2); el lint se declara
+   heurístico, detecta cinco variantes más y F5-1 lo formula como lista
+   blanca (IMP-2, PoC B1); la decisión 4 lleva la salvedad A4/A5 y queda
+   condicionada a resolver `A5` (IMP-3); el límite de la huella nombra las
+   importaciones por nombre de módulo, la matriz admite riesgos aceptados
+   nombrados, `A5` tiene un ADR decisor por crear y `carril`, una acción en
+   el plan (MENOR-1 a 3).
 
 ## Cómo se ve en GitHub
 
@@ -344,7 +378,15 @@ ramas base como únicas ramas permitidas. Ningún workflow con
    (supuesto no verificado): en él no hay escrituras ni secretos. Lo
    comprueba un lint estático (el de la PoC B es el prototipo) en F5-1 y lo
    repite `doctor` (F3-8) sobre los workflows instalados, junto con los
-   permisos por defecto de Actions en solo lectura.
+   permisos por defecto de Actions en solo lectura. **El lint de la PoC es
+   heurístico**: busca patrones prohibidos (checkout del head por `ref:` o
+   por `repository:`, `gh pr checkout` o `git fetch … pull/` en `run:` fuera
+   de un evento de PR, datos del evento en `run:` o en `actions/github-script`
+   también con corchetes o `toJSON(github…)`, `secrets: inherit`, artefacto
+   ejecutado) y deja pasar lo que no conoce. Por eso F5-1 lo formula como
+   **lista blanca**: eventos admitidos por clase y usos de `actions/checkout`
+   permitidos (sin `ref:` ni `repository:` en clase privilegio), y rechaza
+   todo lo demás.
 2. **`carril`** (D1b) corre en `pull_request_target`, sin checkout del head:
    lee el diff, las reviews y la tarea por la API. Es una mitigación
    **parcial**: F1-5 lo relanza con cada review y `pull_request_review`
@@ -372,11 +414,15 @@ ramas base como únicas ramas permitidas. Ningún workflow con
 
    | Quién lee texto hostil | Puede | No puede (y quién lo impide) |
    |---|---|---|
-   | Agente en local (también en lote, RR-18) | ejecutar en la sesión de la persona; empujar ramas de tarea y `taskcode/tareas`; abrir PR | aprobar (`carril` solo cuenta personas, F1-5); integrar sin la aprobación de la persona (rulesets sin bypass); tocar workflows (sin `workflows`, F1-6); publicar checks (sin `checks:write`); firmar un `approve` (FIDO o frase, F1-6) |
+   | Agente en local (también en lote, RR-18) | ejecutar en la sesión de la persona; empujar ramas de tarea y `taskcode/tareas`; abrir PR | **salvo A4/A5**, mientras el token y la clave de la persona estén al alcance de su sesión, la plataforma no lo impide: aprobar (`carril` solo cuenta personas, F1-5); integrar sin la aprobación de la persona (rulesets sin bypass); firmar un `approve` (FIDO o frase, F1-6). Sin salvedad: tocar workflows con su identidad (sin `workflows`, F1-6); publicar checks con su identidad (sin `checks:write`) |
    | Revisor IA en CI | emitir hallazgos JSON | ejecutar nada (job 2 sin código del PR); aprobar ni mergear (App con solo `checks:write`, F1-6); poner el check en verde con prosa: el resultado lo calcula el job con reglas (CRÍTICO o IMPORTANTE abierto → rojo; hallazgo sin evidencia → rechazado, F6-4) |
 
    Un `revision-ia` en verde por inyección no integra nada: la puerta es la
-   aprobación de la persona. Una detección de patrones hostiles se puede
+   aprobación de la persona, salvo A4/A5 (un agente con el token o la clave
+   de la persona al alcance aprueba como ella). **Estado de esta decisión**:
+   garantía de la plataforma para el revisor IA en CI; para el agente en
+   local (y en lote, RR-18), **condicionada a resolver `A5`** (y `A4` si la
+   clave no es FIDO ni con frase sin caché). Una detección de patrones hostiles se puede
    añadir como señal (un hallazgo más), nunca como garantía. La batería de
    casos es de F6-7 y la prueba de un PR malicioso, config hostil e
    inyección en tareas, de F9-5. Lo escrito por la 0.16 se trata igual
@@ -403,7 +449,10 @@ ramas base como únicas ramas permitidas. Ningún workflow con
      historia).
    - **Límite**: el código al que se llega a través de herramientas (tests,
      `build.rs`, scripts de `package.json`, dependencias) no entra en el
-     cierre; hashearlo es hashear el árbol entero y pedir confianza en cada
+     cierre, ni lo que se carga por nombre de módulo y no por ruta (`import
+     helper` en Python, `require('x')`, `source nombre` sin extensión): el
+     cierre solo sigue tokens con forma de ruta; esos ficheros se declaran en
+     `confianza_incluye`; hashearlo es hashear el árbol entero y pedir confianza en cada
      commit. Confiar en una orden de verificación es confiar en ejecutar el
      código de la rama: por eso el hook solo corre la rápida, opcional, y
      nunca al cambiar a una rama ajena sin pedirlo (F5-6). Coste medido en la
@@ -430,7 +479,17 @@ ramas base como únicas ramas permitidas. Ningún workflow con
    `buscar` solo acepta líneas del check-run del **job publicador del
    workflow de TaskCode** (por el job o el `run_id` del check-run, no solo
    por `app_id` y nombre): un check-run creado por la API, el de otro job o
-   el de otro workflow no cuenta (PoC B2). El campo `local` lo calcula el
+   el de otro workflow no cuenta (PoC B2). Es una mitigación **parcial**,
+   como `carril`: con `pull_request` el workflow del publicador sale del
+   propio PR, así que un PR que lo modifique produce un publicador con el
+   mismo nombre, workflow y `run_id` que el legítimo. Por eso `buscar` **no da
+   acierto** (rechaza toda línea `arbol=`, también la del head anterior) si
+   el diff del PR frente a su base toca `.github/workflows/**` o
+   `.taskcode/**`, y ese PR se verifica entero (PoC B2). CODEOWNERS en esas
+   rutas, con una sola persona (RN-06), es la misma aprobación que ya exige
+   el PR y no lo cierra (F1-5, decisión 6). Alternativa para F5-1: mover el
+   publicador a `workflow_run`, que lee el workflow de la base. El campo
+   `local` lo calcula el
    job ejecutor en el runner del perfil y le llega al publicador como dato:
    falsearlo solo puede dar un acierto para el mismo árbol (riesgo aceptado).
 8. **Rotación de secretos.** Ningún secreto filtrado tiene vuelta atrás: se
@@ -455,7 +514,7 @@ La PoC C lee esta tabla entre los marcadores `preguntas-heredadas`.
 <!-- preguntas-heredadas:inicio -->
 | ID | Pregunta heredada | Decisión | Origen |
 |---|---|---|---|
-| `P1` | Workflow modificado en el propio PR: `carril` y `verificacion` salen en verde con `pull_request` | `carril` en `pull_request_target`, parcial (decisión 2); `verificacion` con publicador (decisión 7); E2E en F2-6 | F1-5 decisión 6 y preguntas abiertas; F1-7 preguntas abiertas |
+| `P1` | Workflow modificado en el propio PR: `carril` y `verificacion` salen en verde con `pull_request` | `carril` en `pull_request_target`, parcial (decisión 2); `verificacion` con publicador, también parcial: sin acierto de caché si el PR toca `.github/workflows/**` o `.taskcode/**` (decisión 7); E2E en F2-6 | F1-5 decisión 6 y preguntas abiertas; F1-7 preguntas abiertas |
 | `P2` | La App del revisor con `checks:write` podría publicar `verificacion` o `carril` | se mantiene la contención por `integration_id` 15368 de F1-5; `buscar` exige además el job publicador (decisión 7); no verificado → F2-6 | F1-5 preguntas abiertas |
 | `P3` | Cómo se relanza `carril` al terminar `revision-ia` | el disparador que se use es de clase privilegio (workflow de la base, sin código del PR); cuál, lo fija la implementación de `carril` (F6) con E2E en F2-6 | F1-5 preguntas abiertas |
 | `P4` | Clave de firma de la persona al alcance del agente en la misma máquina | FIDO o frase sin caché (F1-6), nunca en un agente de claves sin confirmación por uso; rotación (decisión 8); no verificado con llave real → F2-6 | F1-5 preguntas abiertas; F1-6 preguntas abiertas |
@@ -466,7 +525,7 @@ La PoC C lee esta tabla entre los marcadores `preguntas-heredadas`.
 | `P9` | Merge por el agente con todo en verde | no lo cierra F1-8: F1-10 (y F1-6, límite D3a) | F1-6 preguntas abiertas |
 | `P10` | Runner propio con código del PR | D3b (decisión 6) | F1-7 decisión 5, consecuencias y preguntas abiertas |
 | `P11` | `pull_request` frente a `pull_request_target` | dos clases de jobs (decisión 1) | F1-7 consecuencias |
-| `P12` | Quién puede publicar un `verificacion` con resumen falso (todo workflow con `checks:write`, `integration_id` 15368) | jobs ejecutores con `contents: read`, permisos por defecto en lectura, línea `arbol=` solo del publicador (decisiones 1 y 7); no verificado → F2-6 | F1-7 consecuencias |
+| `P12` | Quién puede publicar un `verificacion` con resumen falso (todo workflow con `checks:write`, `integration_id` 15368) | jobs ejecutores con `contents: read`, permisos por defecto en lectura, línea `arbol=` solo del publicador y nunca en un PR que toca workflows o `.taskcode/` (decisiones 1 y 7), parcial; no verificado → F2-6 | F1-7 consecuencias |
 | `P13` | Confianza del hook local | confianza única con hash de la orden y de lo que ejecuta (decisión 5) | F1-7 decisión 4 y consecuencias |
 <!-- preguntas-heredadas:fin -->
 
@@ -478,8 +537,9 @@ Los ADR aceptados no se modifican; lo que este ADR cambia queda aquí y en
 1. **F1-7, decisión 3** (D5a): F1-7 acepta como candidata cualquier línea de
    un check `verificacion` en verde de la App 15368. La línea `arbol=` pasa a
    contar **solo si la escribió el job publicador** del workflow de TaskCode, que calcula el árbol él
-   mismo (decisión 7); `verificacion` pasa de un job a dos (`ejecutar` y el
-   publicador), además de `buscar`.
+   mismo (decisión 7), y nunca hay acierto en un PR que toca
+   `.github/workflows/**` o `.taskcode/**`; `verificacion` pasa de un job a
+   dos (`ejecutar` y el publicador), además de `buscar`.
 2. **F1-5, decisión 6**: F1-5 propone `pull_request_target` para `carril`
    como cierre del workflow modificado en el PR. Aquí es **mitigación
    parcial** (D1b): no cubre `pull_request_review`.
@@ -502,9 +562,9 @@ Los ADR aceptados no se modifican; lo que este ADR cambia queda aquí y en
 | Enunciado de F1-8: prompt injection | decisión 4 | tabla de capacidades; batería: F6-7; prueba: F9-5; no verificado |
 | Enunciado de F1-8: runner propio | decisión 6 | PoC D (repos privados, 0 runners, workflows de forks desactivados); condiciones: F5-5, no verificado |
 | H2 nada del repo se ejecuta sin confianza | decisión 5 | PoC A (orden, script nombrado y script transitivo cambiados → `cambiado`); implementación: F0-2, F5-6 |
-| H9 el guardián local de merges se salta | no se defiende en local: integrar exige la plataforma (rulesets sin bypass, F1-5; `carril`, F1-6) | decisión 4 (tabla de capacidades) |
+| H9 el guardián local de merges se salta | no se defiende en local: integrar exige la plataforma (rulesets sin bypass, F1-5; `carril`, F1-6), salvo A4/A5 (el token de la persona al alcance del agente lo salta) | decisión 4 (tabla de capacidades), condicionada a resolver `A5` |
 | RR-03 la revisión reproduce | reproducción en un job sin secretos; el LLM la lee como dato (decisión 3) | PoC B (`revision-reproducir` de clase ejecuta y `revision-ia` de clase privilegio, sin ejecutar el artefacto); `workflow_run` con artefactos: no verificado (F2-6) |
-| RR-08 garantías de la plataforma | capacidades en vez de detección; permisos y eventos comprobables | PoC B y D |
+| RR-08 garantías de la plataforma | capacidades en vez de detección; permisos y eventos comprobables | PoC B (lint heurístico; F5-1, lista blanca) y D; para el agente local, condicionada a `A5` |
 | RR-13 actualizar los proyectos | los pilotos no tienen `.github/`: `actualizar` instala los workflows nuevos, no reescribe ninguno | PoC D (0 ficheros en `.github/` en `develop` y `main` de los dos pilotos) |
 | RR-15 GitHub y GitLab | las dos clases y la confianza local son neutrales | equivalente en GitLab (pipelines de merge request, variables protegidas, runners por etiqueta): no verificado (F1-13) |
 | RR-18 lotes desatendidos | mismas capacidades; el lote no integra en `develop` sin la persona (`A16`) | diseño de F1-16; no verificado |
@@ -523,9 +583,12 @@ Los ADR aceptados no se modifican; lo que este ADR cambia queda aquí y en
   ToolsaWeb) se lee con `\v` como tabulador vertical; el formato de la
   configuración tiene que fijar cómo se escriben las rutas de Windows.
 - **F5-1** implementa las dos clases (jobs `buscar`, `ejecutar` y
-  `verificacion`), el bloque `permissions` por job y el lint estático;
-  **F5-2** escribe la línea `arbol=` en el publicador y `buscar` comprueba el
-  job publicador (decisión 7).
+  `verificacion`), el bloque `permissions` por job y el lint estático como
+  **lista blanca de eventos y de usos de `actions/checkout`** por clase, no
+  como lista de patrones prohibidos; valora mover el publicador de
+  `verificacion` a `workflow_run`. **F5-2** escribe la línea `arbol=` en el
+  publicador, y `buscar` comprueba el job publicador y no da acierto si el
+  PR toca `.github/workflows/**` o `.taskcode/**` (decisión 7).
 - **F5-5** solo registra el runner propio con las tres condiciones de D3b; si
   no, no hay pesada en PR.
 - **F6-1** crea el entorno `revisor` con la clave de la App; **F6-3** parte el
@@ -534,8 +597,13 @@ Los ADR aceptados no se modifican; lo que este ADR cambia queda aquí y en
   reproducciones pedidas por el LLM como datos para la ronda siguiente
   (F6-6); **F6-5** calcula el check desde los hallazgos estructurados; **F6-7**
   hace la batería de inyección.
-- **F6**: `carril` no tiene fila propia en el plan (F1-5 lo asigna a «F6»); hay
-  que añadirla con `pull_request_target` y la migración en un solo commit.
+- **Acciones en el plan** (las hace Carlos al revisar el plan, antes de
+  empezar F5-1): (a) **crear una fila F6-x para `carril`** (F1-5 lo asigna a
+  «F6» sin fila) con `pull_request_target` y la migración en un solo commit;
+  hasta entonces su dueña en la matriz es F5-1; (b) **crear un ADR de Fase 1
+  «aislamiento del agente frente a las credenciales de la persona»**, dueño
+  de la decisión sobre `A5` (agente en otro usuario del sistema o en un
+  contenedor, token sin guardar, confirmación fuera de banda).
 - **F3-8** (`doctor`) compara los permisos por defecto de Actions,
   `can_approve_pull_request_reviews`, los workflows de forks en privados, las
   clases de los workflows instalados, la ausencia de `repository_dispatch` y
@@ -583,6 +651,17 @@ Los ADR aceptados no se modifican; lo que este ADR cambia queda aquí y en
 
 ## No verificado
 
+- **Completitud del lint de clases** (PoC B1): es heurístico. Detecta los
+  20 negativos de la PoC, entre ellos las cinco variantes de la revisión 1
+  (checkout con `repository:` del head, `gh pr checkout` con el número por
+  `env:`, título en `actions/github-script`, `github['event']` y
+  `toJSON(github.event)`, `secrets: inherit`), pero no prueba que un workflow
+  sea seguro: acciones de terceros que hacen checkout del head, scripts que
+  traen el head por otra vía o expresiones en otros campos de `with:` pasan.
+  Por eso F5-1 exige lista blanca.
+- Que `buscar` pueda leer el diff del PR frente a su base para decidir si
+  toca `.github/workflows/**` o `.taskcode/**` en el evento de la búsqueda
+  (decisión 7): F2-6.
 - Que un `GITHUB_TOKEN` con `checks:write` cree check-runs con cualquier
   nombre (`carril`, `verificacion`) y que con `contents: read` no pueda:
   E2E en F2-6.
````

## Excluido del diff (11 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff cfdcce690f213a71214d169c31c710d25dec6160..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-009/planificacion/brainstorm/peticion-brainstorm-arquitectura-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-009/planificacion/brainstorm/peticion-brainstorm-riesgos-1.md      |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-009/planificacion/brainstorm/peticion-unificador-1.md              |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-009/planificacion/brainstorm/salida-brainstorm-arquitectura-1.md   |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-009/planificacion/brainstorm/salida-brainstorm-riesgos-1.md        |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-009/planificacion/plan-final.md                                    |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-009/poc/poc-f1-8.log                                               |  55 ++++---
 tareas/{02-en-curso => 03-en-revision}/TASK-009/poc/poc-f1-8.sh                                                | 148 +++++++++++++++---
 tareas/03-en-revision/TASK-009/revision/informe-revision-1.md                                                  |  21 +++
 tareas/03-en-revision/TASK-009/revision/peticion-revision-1.md                                                 | 692 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-009/tarea.md                                                       |  11 +-
 11 files changed, 877 insertions(+), 50 deletions(-)
````

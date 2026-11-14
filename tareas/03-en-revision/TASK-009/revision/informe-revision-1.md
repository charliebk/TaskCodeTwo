# Informe de revision — TASK-009 (ronda 1)

- Commit revisado: cfdcce690f213a71214d169c31c710d25dec6160
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

Veredicto propuesto por el revisor: **cambios-solicitados** (3 IMPORTANTE abiertos).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:74 | plan |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:80 | implementacion |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:375 | implementacion |
| MENOR-1 | MENOR | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:404 | implementacion |
| MENOR-2 | MENOR | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:72 | implementacion |
| MENOR-3 | MENOR | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:75 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Alcance: completa, un solo pase generico (el diff es solo documentacion mas una PoC, sin dominio con revisor propio). Profundidad: a fondo el ADR, la PoC B (lint) y la PoC A (confianza); por encima las PoC A2 y D (lecturas), que solo se reprodujeron, no se auditaron linea a linea.

### IMP-1 — La caché `arbol=` sigue falsificable si el PR modifica el workflow del publicador
- Donde: docs/adr/F1-8-seguridad-modelo-de-amenazas.md:74 (A7); mismo problema en :95 (clase «publicador… en el mismo `pull_request`»), :425 (decisión 7) y :458 (P1).
- Que pasa: la decisión 7 y la PoC B2 aceptan la línea `arbol=` solo si la escribió «el job publicador del workflow de TaskCode». Pero ese job está definido en un workflow que corre con `pull_request`, y con `pull_request` GitHub ejecuta el workflow del propio PR (lo dice F1-5, decisión 6, y el propio ADR en D1b). Un PR que edita `verificacion.yml` cambia el job publicador (mismo nombre, mismo workflow, mismo `run_id`) para que escriba cualquier `arbol=…`, o le quita el lint de clases. Para `carril` el ADR lo asume y lo declara «parcial»; para `verificacion` lo da por resuelto (P1: «`verificacion` con publicador»), y A7 solo dice «no verificado» sobre el mecanismo de identificar el job, no sobre este hueco lógico.
- Reproduccion: en `poc-f1-8.sh` el caso B2 solo modela «check-run creado por la API», «línea en el resumen del ejecutor» y «job de otro workflow». Ningún caso modela «publicador legítimo de un workflow modificado en el PR»; sus atributos (app 15368, nombre, workflow, job) son idénticos al control positivo, así que cualquier regla basada en ellos lo acepta (se razona sobre el workflow de ejemplo `P1-verificacion.yml`: el job `verificacion` solo hace `echo "arbol=… " >> "$GITHUB_STEP_SUMMARY"`, que el PR puede reescribir). No se pudo ejecutar contra GitHub (solo lectura): marcado **no verificado**, pero el fallo es de diseño, no de plataforma.
- Impacto: un verde reutilizable y persistente (A7, ALTA) con la caché de F1-7 para un árbol que no se verificó; el ADR afirma lo contrario en la tabla de preguntas heredadas.
- Sugerencia: reconocerlo en A7, P1, P12 y decisión 7 como mitigación **parcial** igual que `carril`, y cerrarlo con algo concreto: `buscar` rechaza la línea si el diff del PR (o del commit del check-run frente a la base) toca `.github/workflows/**` o `.taskcode/**`, o el publicador pasa a `workflow_run`/`pull_request_target` leyendo el workflow de la base. Dejar dicho que CODEOWNERS con una sola persona es la misma aprobación del PR (F1-5, decisión 6 ya lo avisa) y no lo cierra.

### IMP-2 — Las mitigaciones A6/A13 se apoyan en un lint que no detecta variantes clásicas, y el ADR no declara su límite
- Donde: docs/adr/F1-8-seguridad-modelo-de-amenazas.md:80 (A13); también :73 (A6), :152 (decisión 1) y la sección PoC B (`poc-f1-8.sh:535`, función `evalua`).
- Que pasa: el ADR presenta «lo comprueba el lint (PoC B) y `doctor`» con «controles negativos 14/14», pero el lint es una heurística de cuatro expresiones regulares. Nueve workflows mutados, uno por variante, contra `clases.py` real de la PoC (`python -I`, carpeta temporal fuera del repo):
  - `pull_request_target` + `actions/checkout` con `repository: ${{ github.event.pull_request.head.repo.full_name }}` y sin `ref:` (el patrón «pwn request» clásico; el job ejecuta el head del fork con `checks: write`): clase `privilegio`, **sin errores**.
  - `pull_request_target` + `gh pr checkout "$PR"` o `git fetch … pull/N/head` pasando el número por `env:` y luego `bash scripts/verificar.sh`: **sin errores** (solo se detecta si el número va interpolado en `run:`, como en el caso M9 que sí salta R5 por otra razón).
  - Título del PR interpolado en `actions/github-script` (`script:`): **sin errores** (el lint solo mira `run:`).
  - `${{ github['event']['pull_request']['title'] }}` y `${{ toJSON(github.event) }}` en `run:`: **sin errores**.
  - Job que llama a un workflow reutilizable con `secrets: inherit` en `pull_request`: **sin errores**.
  Los casos con `ref:` al head, `issue_comment`+`refs/pull/…`, `workflow_run`+`head_sha` y `github.event.pull_request.title` en `run:` sí saltan (R1/R5).
- Impacto: A6 y A13 son CRÍTICA/ALTA y el ADR las deja con «lint estático» como defensa comprobable; quien implemente F5-1 copiará un prototipo que deja pasar el patrón más conocido. Contradice la regla del propio plan («si no se ha ejecutado, no se afirma»): la evidencia 14/14 no cubre las variantes y el ADR no lo dice.
- Sugerencia: (a) añadir al ADR (decisión 1 y «No verificado») que el lint de la PoC es heurístico y lista sus huecos conocidos; (b) pasar a la PoC los casos M1–M5 como controles negativos y ampliar `evalua` (checkout con `repository:` apuntando al head en cualquier evento no-`pull_request`, `gh pr checkout`/`git fetch … pull/`/`refs/pull` en `run:` bajo `pull_request_target`, `${{ … }}` con `github.event`/`github['event']`/`toJSON(github` en cualquier campo de `with:`/`env:`/`run:`, `secrets: inherit`); (c) formular el requisito de F5-1 como lista blanca de eventos y usos de `checkout` permitidos, no como lista de patrones prohibidos.

### IMP-3 — La tabla de capacidades de la decisión 4 afirma como imposible lo que A4/A5 dicen que es posible
- Donde: docs/adr/F1-8-seguridad-modelo-de-amenazas.md:375 (tabla «Agente en local… No puede: aprobar…, integrar sin la aprobación de la persona…, firmar un `approve`»); también :378 («la puerta es la aprobación de la persona») y :505 (fila H9 de «Requisitos y evidencia»).
- Que pasa: la misma ADR dice en `A5` (CRÍTICA) que el agente, con el mismo usuario del sistema, saca el token de la persona del llavero y «aprueba o mergea como la persona», y en `A4` que puede firmar un `approve` falso si la clave está en un agente de claves. Aun así, la decisión 4 lista como «no puede (y quién lo impide)» aprobar (`carril` solo cuenta personas) e integrar sin la persona (rulesets sin bypass): ambas protecciones se saltan con el token de la persona, que es una cuenta de persona, no un bot. La defensa por capacidades (RR-08: lo impone la plataforma) queda así sin garantía para el agente local, que es justo el caso RR-18/RN-06, y la fila H9 cita esa tabla como evidencia.
- Reproduccion: lectura cruzada de las líneas 72 (A5), 71 (A4) y 375; el hecho de base (token en el llavero, PoC D: `gh auth status … (keyring)`) está en el propio log de la PoC. No se ejecutó nada de GitHub fuera de `GET`.
- Impacto: un lector de la decisión 4 concluye que un agente hostil no puede aprobar ni integrar; el ADR sabe que sí puede mientras A5 siga abierto. Es un cierre presentado sin evidencia.
- Sugerencia: añadir a las celdas «No puede» de la fila del agente local la salvedad «salvo A4/A5: mientras el token y la clave de la persona estén al alcance de su sesión, la plataforma no lo impide», corregir la fila H9 y la frase de :378 en el mismo sentido, y declarar el estado real de la decisión 4 (garantía para el revisor IA en CI; para el agente local, condicionada a resolver A5).

### MENOR-1 — El límite de la huella de confianza no nombra los `import`/`source` por nombre de módulo
- Donde: docs/adr/F1-8-seguridad-modelo-de-amenazas.md:404 (decisión 5, «Límite») y `poc-f1-8.sh:78` (`_resuelve`, que descarta tokens sin `.` ni `/`).
- Que pasa: el cierre solo sigue tokens con forma de ruta. Reproducido con `confianza.py` de la PoC en un repo de juguete: `verificar.sh` → `python tools/gen.py`; `gen.py` hace `import helper` (`tools/helper.py`). Cierre calculado: `['scripts/verificar.sh', 'tools/gen.py']`. Tras `confiar`, se reescribe `tools/helper.py` con `os.system("echo PWNED")` y `confianza` sigue devolviendo `confiado`.
- Impacto: bajo. En ToolsaWeb (solo lectura) no hay `import` entre hermanos de `scripts/*.py` (0 coincidencias), y el ADR ya limita «herramientas y rutas construidas en tiempo de ejecución». Pero el texto del límite habla de tests, `build.rs`, `package.json` y dependencias, no de este caso, que parece cubierto por «transitivo».
- Sugerencia: nombrarlo en el límite (importaciones por nombre de módulo, `source nombre` sin extensión) y en `confianza_incluye`; no hace falta código.

### MENOR-2 — «Cada fila tiene mitigación y tarea dueña» no es cierto para A5 ni A15, y la dueña de A5 no puede decidirlo
- Donde: docs/adr/F1-8-seguridad-modelo-de-amenazas.md:72 (A5), y la frase de cabecera de la matriz («Cada fila tiene mitigación y tarea dueña»).
- Que pasa: A5 (CRÍTICA) y A15 (MEDIA) reconocen «queda como riesgo abierto» y «ninguna». La dueña de A5 es F2-6, que en el plan es el adaptador GitHub con E2E, no una tarea de decisión; la decisión sobre usuario del sistema/contenedor/token sin guardar no tiene dueña real. La PoC C valida «dueña existe como fila del plan» pero no que la dueña pueda decidir.
- Impacto: la matriz se lee como cerrada más de lo que está, y la CRÍTICA sin mitigación no tiene tarea que la cierre.
- Sugerencia: reformular la cabecera («cada fila tiene mitigación o riesgo aceptado nombrado») y asignar A5 a un ADR/tarea de decisión (por ejemplo, un ADR nuevo en Fase 1 o F1-6 reabierta), no solo a F2-6 y F3-8.

### MENOR-3 — Dueña «F6» de A8 no es un ID de tarea
- Donde: docs/adr/F1-8-seguridad-modelo-de-amenazas.md:75 (A8).
- Que pasa: la dueña dice «F5-1, F6 (`carril` sin fila en el plan)». «F6» es una fase, no una tarea; el propio ADR dice en Consecuencias que hay que añadir la fila. Aceptable como aviso, pero la matriz queda con una dueña sin tarea hasta que alguien cree la fila.
- Sugerencia: añadir en Consecuencias una acción explícita («añadir al plan una fila F6-x para `carril`») con responsable, o dejar la dueña en F5-1 mientras tanto.

## Verificacion realizada

- Rama `feature/task-009-f1-8-adr-seguridad`, HEAD `a9da7bd` (el commit revisado `cfdcce6` más solo la petición de revisión; el ADR y la PoC no difieren).
- `bash poc-f1-8.sh "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-009-r1" docs/adr/F1-8-seguridad-modelo-de-amenazas.md /c/Users/nullcad2025/Documents/sources/Personal`: salida 0; las siete líneas `RESULTADO` coinciden con el log registrado y con el ADR (A1 7/7 y 8/8 y 1/1; A2 4; B1 5/5 y 14/14; B2 7/7 y 2; C 0 errores, 16 activos, 13 preguntas, 12/12 mutaciones; D 16/16; 15 llamadas `gh api` con GET y 0 de otro método). El primer intento salió con 2 por una carpeta de trabajo mal preparada por mí, no por la PoC.
- `bash .github/scripts/comprobar-repo.sh`: «Tareas: 93; enlaces relativos: 62; fallos: 0».
- Mutaciones fuera del guion: nueve workflows (M1–M9) contra `clases.py` y un repo de juguete contra `confianza.py` (IMP-2, MENOR-1). Los controles que sí debían saltar (M6 título en `run:`, M7 `issue_comment` con `refs/pull`, M8 `workflow_run` con `head_sha`, M9 `github.event.number` en `run:`) saltaron.
- Contraste con los ADR anteriores: F1-5, F1-6 y F1-7 citan F1-8 en 5, 6 y 7 líneas (coincide con la PoC C); los IDs de F-tareas citados como dueñas (F0-2, F1-10, F1-11, F1-13, F1-16, F2-6, F3-1, F3-8, F4-2, F4-7, F5-1, F5-2, F5-5, F5-6, F6-1, F6-3, F6-4, F6-5, F6-6, F6-7, F9-5) existen en el plan; RR-03, RR-08, RR-13, RR-15, RR-18, RF-70, RN-02 y RN-06 existen en REQUISITOS-FUNCIONALES; H2 y H9 existen en la auditoría. Las preguntas heredadas de F1-5, F1-6 y F1-7 que apuntan a F1-8 están cubiertas por P1–P13. Coherentes con D1b, D2b, D3b, D4b y D5a del plan; las divergencias con F1-5/F1-6/F1-7 están declaradas.
- Honestidad: la marca «no verificado → F2-6» se usa de forma consistente salvo en los tres puntos de IMP-1, IMP-2 e IMP-3.
- No ejecutado: `taskctl doctor` (no pedido), nada contra GitHub salvo lo que ya hace el guion con GET; no se invocó `gh auth token` para no manejar el token de la persona.
- Carpeta temporal `pruebas-claude/taskcodetwo/task-009-r1` (y su log) borrada al terminar.

## Lecturas fuera del contexto permitido

- tareas/03-en-revision/TASK-009/poc/poc-f1-8.sh y poc-f1-8.log: son el entregable a reproducir, lo pide el encargo.
- docs/plan/PLAN-TASKCODE-2.md, docs/plan/REQUISITOS-FUNCIONALES.md y docs/auditoria/AUDITORIA-EXTERNA-2026-11-13.md: comprobar que existen los IDs de tarea, requisito y hallazgo citados.
- docs/adr/F1-5-puertas-en-github.md, F1-6-identidades-agente-revisor-persona.md y F1-7-verificacion-ci-perfiles-cache-runners.md: contrastar las preguntas heredadas y las divergencias (en particular F1-5 decisión 6, base de IMP-1).
- .github/workflows/ci.yml: comprobar la afirmación «ya cumple la clase ejecuta».
- ToolsaWeb (`git ls-files` y `git grep`, solo lectura): MENOR-1.
- tareas/03-en-revision/TASK-009/revision/peticion-revision-1.md y la skill/`revision.md` del plugin: instrucciones de la ronda y plantilla del informe.

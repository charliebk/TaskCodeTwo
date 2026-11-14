# Peticion de revision — TASK-009 (ronda 3)

- Tarea: TASK-009 — F1-8 ADR seguridad
- Rama revisada: feature/task-009-f1-8-adr-seguridad
- Rama base: develop
- Commit revisado (HEAD): 15920490783032be788e78412e2adf495fc805ca
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-009 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde fb9e2a9cf4f4761ed65e2f3c1f7f13405ee2607f (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (4 hallazgo(s)))

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
informe de esta ronda (informe-revision-3.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-009/tarea.md
- tareas/03-en-revision/TASK-009/planificacion/
- docs/adr/F1-8-seguridad-modelo-de-amenazas.md

## Hallazgos de la ronda 2 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:75 | informe-revision-2.md |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:381 | informe-revision-2.md |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:417 | informe-revision-2.md |
| IMP-4 | IMPORTANTE | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:417 | informe-revision-2.md |

## Commits a revisar (git log fb9e2a9cf4f4761ed65e2f3c1f7f13405ee2607f..HEAD)

````
1592049 chore(TASK-009): autorrevision y ronda extra 3
18370f1 docs(adr): F1-8 corrige IMP-4 de la revision 2 (salvedad A4/A5 en toda la celda y scope workflow)
441d966 chore(TASK-009): peticion de revision ronda 2
````

## Diff desde la ronda anterior (git diff fb9e2a9cf4f4761ed65e2f3c1f7f13405ee2607f..HEAD)

````diff
diff --git a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
index 2d901cf..6772a45 100644
--- a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
+++ b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
@@ -414,7 +414,7 @@ ramas base como únicas ramas permitidas. Ningún workflow con
 
    | Quién lee texto hostil | Puede | No puede (y quién lo impide) |
    |---|---|---|
-   | Agente en local (también en lote, RR-18) | ejecutar en la sesión de la persona; empujar ramas de tarea y `taskcode/tareas`; abrir PR | **salvo A4/A5**, mientras el token y la clave de la persona estén al alcance de su sesión, la plataforma no lo impide: aprobar (`carril` solo cuenta personas, F1-5); integrar sin la aprobación de la persona (rulesets sin bypass); firmar un `approve` (FIDO o frase, F1-6). Sin salvedad: tocar workflows con su identidad (sin `workflows`, F1-6); publicar checks con su identidad (sin `checks:write`) |
+   | Agente en local (también en lote, RR-18) | ejecutar en la sesión de la persona; empujar ramas de tarea y `taskcode/tareas`; abrir PR | **salvo A4/A5**, mientras el token y la clave de la persona estén al alcance de su sesión, la plataforma no lo impide: aprobar (`carril` solo cuenta personas, F1-5); integrar sin la aprobación de la persona (rulesets sin bypass); firmar un `approve` (FIDO o frase, F1-6); tocar workflows (con su identidad no puede: la App no tiene `workflows`, F1-6; pero el token de la persona en esta máquina sí tiene el scope `workflow`, observado con `gh auth status` el 2026-11-14); publicar checks con su identidad (sin `checks:write`). Todo lo de esta celda queda condicionado a resolver A5 |
    | Revisor IA en CI | emitir hallazgos JSON | ejecutar nada (job 2 sin código del PR); aprobar ni mergear (App con solo `checks:write`, F1-6); poner el check en verde con prosa: el resultado lo calcula el job con reglas (CRÍTICO o IMPORTANTE abierto → rojo; hallazgo sin evidencia → rechazado, F6-4) |
 
    Un `revision-ia` en verde por inyección no integra nada: la puerta es la
@@ -651,6 +651,8 @@ Los ADR aceptados no se modifican; lo que este ADR cambia queda aquí y en
 
 ## No verificado
 
+- Que un agente que use el token de la persona (A5) no pueda empujar cambios en `.github/workflows/`: el token observado tiene el scope `workflow` (`gh auth status`, 2026-11-14, sin mostrar el token), así que lo puede hacer hasta que A5 se resuelva; no se ha probado empujando nada.
+
 - **Completitud del lint de clases** (PoC B1): es heurístico. Detecta los
   20 negativos de la PoC, entre ellos las cinco variantes de la revisión 1
   (checkout con `repository:` del head, `gh pr checkout` con el número por
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff fb9e2a9cf4f4761ed65e2f3c1f7f13405ee2607f..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/03-en-revision/TASK-009/revision/informe-revision-1.md  |  76 ++++++++++++++++++++--
 tareas/03-en-revision/TASK-009/revision/informe-revision-2.md  |  21 ++++++
 tareas/03-en-revision/TASK-009/revision/peticion-revision-2.md | 460 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-009/tarea.md                        |   9 ++-
 4 files changed, 560 insertions(+), 6 deletions(-)
````

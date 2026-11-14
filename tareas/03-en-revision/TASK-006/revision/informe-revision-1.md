# Informe de revision — TASK-006 (ronda 1)

- Commit revisado: d05805a37a2b99f58dbfe9c0180713914bbb188d
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-5-puertas-en-github.md:283 | plan |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-5-puertas-en-github.md:265 | plan |
| MEN-1 | MENOR | corregido | docs/adr/F1-5-puertas-en-github.md:300 | plan |
| MEN-2 | MENOR | corregido | tareas/03-en-revision/TASK-006/poc/poc-f1-5.sh:238 | implementacion |
| MEN-3 | MENOR | corregido | tareas/03-en-revision/TASK-006/poc/poc-f1-5.sh:210 | implementacion |
| MEN-4 | MENOR | corregido | docs/adr/F1-5-puertas-en-github.md:269 | plan |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

## Reproduccion y correcciones

Alcance: completo (ADR, fila del indice y PoC). Pase generico (sin revisor de dominio).

### IMP-1 — El check `carril` lee el carril y la aprobacion del plan de una rama que el agente escribe sin puerta

- Donde: ADR:283 («lee el carril de la tarea en `taskcode/tareas`; un PR sin tarea es errata») y ADR:271 (`taskcode/tareas`: «sin PR ni checks; push directo del CLI»); ADR:286-287 («plan aprobado por una persona (F4-2)»).
- Razonamiento (leido, no ejecutable: no se escribe en GitHub): el ruleset de `taskcode/tareas` solo impide borrar y forzar, y no hay `bypass_actors`, asi que no se puede restringir quien empuja. Cualquier identidad con escritura, tambien la del agente, puede hacer un push fast-forward a esa rama. Lo que `carril` toma de ella (carril de la tarea y marca de plan aprobado) es la unica diferencia entre un fix y una funcionalidad o riesgo alto (los checks de la tabla son identicos en develop). El agente puede (a) cambiar `carril` de la tarea a errata o fix, o (b) escribir la aprobacion del plan, y `carril` sale verde; o abrir el PR sin tarea (= errata, ADR:283). La «segunda aprobacion» de riesgo alto (D4a) es justo la del plan, o sea la misma fuente falsificable. El ADR no lo recoge ni en «Preguntas abiertas», y el requisito H5 («la aprobacion humana no se puede comprobar») queda sin cerrar en este punto, aunque la tabla de requisitos (ADR:346) lo da por razonado.
- Correccion concreta: anadir a la decision 4 que `carril` solo da por buenos el carril y la aprobacion del plan si proceden de un registro que el agente no puede escribir o falsificar, y nombrar el mecanismo. Opciones: (a) `required_signatures` en el ruleset de `taskcode/tareas` y `carril` exige que el commit de aprobacion este firmado por la clave de la persona (no de la App); (b) la aprobacion del plan y la eleccion del carril se registran como review o comentario de la persona (identidad de persona verificada con la API) y no como campo del frontmatter; (c) `carril` ignora el carril declarado y deriva uno minimo de los ficheros del PR (F1-10). Mientras no se elija, anadirlo a «Preguntas abiertas» como riesgo no cerrado y no afirmar que la puerta es inviolable por el agente. Fijar tambien que un PR sin tarea solo es errata si cumple los limites de F1-10 y que `carril` falla cerrado si no puede decidirlo.
- Verificacion: lectura cruzada de ADR:271, :283, :286-290 y de `taskcode-tareas.json` (solo `deletion` y `non_fast_forward`); F1-1 dice «push directo del CLI» sin identidad propia y remite las reglas de la rama de estado a F1-5.

### IMP-2 — `strict` en `main` con releases por merge commit deja `develop` desactualizada tras cada release

- Donde: ADR:265 (`main`: «igual» que develop, incluye `strict`) y ADR:243; con D3a de F1-3 (release = PR `develop`→`main` con merge commit, sin backmerge).
- Reproduccion (git local, carpeta temporal): `develop` y `main` iguales; commit d1 en develop; merge `--no-ff` de develop en main (release 1); commit d2 en develop; `git merge-base --is-ancestor main develop` responde «main NO es ancestro de develop». Con `strict_required_status_checks_policy: true`, GitHub exige que la rama del PR contenga el tip de la base; el segundo PR `develop`→`main` saldria «out-of-date». Actualizarlo es empujar a `develop` (la rama del PR), que su ruleset prohibe (sin bypass, solo por PR), asi que cada release a partir de la segunda exigiria antes un PR extra `main`→`develop` con todas sus puertas. El log de F1-3 no lo vio porque su repo no tenia `strict`. El comportamiento de GitHub no se ha verificado (no se escribe en GitHub); el hecho de ancestria si.
- Correccion concreta: poner `strict_required_status_checks_policy: false` en el ruleset de `main` (en `poc-f1-5.sh` la rama `main` lleva `al_dia=True` por modelo, lineas 140-142: pasarla a `False`), regenerar `rulesets/main.json` y el log, y cambiar el ADR (modelo neutral ADR:243, decision 2 en :265, decision 7 y «Como se ve») para que `strict` sea solo de `develop`. Alternativa: documentar el PR `main`→`develop` previo a cada release y su coste con una sola persona. En todo caso, si no se corrige, anadirlo a «Preguntas abiertas».

### MEN-1 — Decision 6 afirma lo que «Preguntas abiertas» reconoce que no se cumple

- ADR:300-303 dice que asi «el agente no rebaja una puerta editando el workflow de `carril` o de `verificacion` en su propio PR sin que la persona lo apruebe expresamente»; ADR:403-407 reconoce que con `pull_request` el workflow de la rama del PR ya ha dado verde. Ademas, con una sola persona (RN-06) la aprobacion CODEOWNERS es la misma que ya exige el PR y no anade nada «expreso».
- Correccion: reescribir la decision 6 como mitigacion parcial («la persona ve el cambio en el diff; el check puede haber salido verde por el workflow modificado hasta que F1-8 fije `pull_request_target` o un workflow de la base») y remitir a la pregunta abierta; no usar «no rebaja».

### MEN-2 — La PoC D no compara la columna «Quien aprueba» con nada del ruleset

- poc-f1-5.sh:238 solo exige que aparezca «persona» y ninguna palabra de IA. Mutacion en una copia del ADR: fila de riesgo alto cambiada a «1 persona» (sin la doble aprobacion de D4a) da `RESULTADO D filas=6 fallos=0 ; negativos detectados=3/3` (no detectada). Cambiar la fila de errata a «1 persona en el PR y el agente» si se detecta. Las mutaciones de checks (errata sin `revision-ia`, release con base develop, hotfix sin `revision-ia`) tambien se detectan. El ADR (:296, :333, :342) presenta la tabla como comprobada; solo lo estan los checks y la ausencia de IA.
- Correccion: comparar el numero de aprobaciones (carriles de 1 persona = `required_approving_review_count` del JSON; riesgo alto = 2 con plan+PR) o decir en el ADR que la PoC D solo cubre checks y ausencia de IA.

### MEN-3 — Los JSON versionados no se vuelven a validar

- poc-f1-5.sh:210 regenera los JSON y los copia (sobrescribe) a la carpeta de artefactos; nada lee los `rulesets/*.json` ya guardados. Una edicion a mano (clave o enum invalido) en `tareas/.../poc/rulesets/*.json` no la detecta ningun paso, y el ADR habla de «los seis JSON de la PoC» como validados. Establecido por lectura del guion. Los JSON generados en esta ronda son identicos a los versionados salvo finales de linea CRLF del checkout.
- Correccion: anadir al final de C un paso que valide con el mismo validador los ficheros de la carpeta de artefactos y/o compare con los generados, o decir en el ADR que son salida del guion y no fuente.

### MEN-4 — El ruleset de ramas conservadas solo cubre `feature/**`, `fix/**` y `hotfix/**`

- ADR:269 y `ramas-de-tarea.json`: F1-3 fija esos tres tipos y el ADR dice que una errata es un «PR directo» sin tarea, pero no dice de que rama sale. Si sale de otra (`docs/...`, `errata/...`) esa rama no tiene proteccion contra borrado ni force-push, contra «las ramas se conservan» (F1-3).
- Correccion: fijar en el ADR el prefijo de rama de la errata y anadirlo al ruleset, o decir que las ramas fuera de esos tipos no estan protegidas y lo cierra F1-10.

## Verificacion realizada

- `bash poc-f1-5.sh` con carpeta `$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-006-r1`: exit 0; salida identica al `poc-f1-5.log` versionado (A: 403 x3 en el privado, 200 `[]`, `[]` y 404 en el publico, id Actions 15368; C: 6 rulesets, 0 errores, 7/7 negativos; D: 6 filas, 0 fallos, 3/3; 10 GET, 0 otros metodos). Ninguna llamada de escritura propia en GitHub.
- Mutaciones del ADR (5): riesgo alto «1 persona» (no detectada, MEN-2); errata sin `revision-ia`, errata con «y el agente», release con base `develop` y hotfix sin `revision-ia` (detectadas).
- `bash .github/scripts/comprobar-repo.sh`: «Tareas: 93; enlaces relativos: 40; fallos: 0».
- IDs citados: RR-04, RR-07, RR-08, RR-15, RF-50, RF-51, RF-64, RN-06, P3, P7, P11, H5 y H6 existen; citas de RF-50 y RF-51 coherentes con el texto. D1a..D6a del ADR coinciden con plan-final.md. Coherencia con F1-1 (rama de estado), F1-3 (merge commit, ramas conservadas, tags) y F1-4 (puerto `Plataforma`): correcta salvo IMP-2 y MEN-4.
- Bloqueo con una sola persona: cubierto con honestidad en «Preguntas abiertas»; IMP-2 es un bloqueo adicional no recogido.
- Criterios de tarea.md: lista exacta de checks y aprobador por carril presente (ADR:320-327); segundo criterio cumplido salvo los hallazgos. Afirmaciones de plataforma marcadas «no verificado» donde toca.
- Carpeta temporal `task-006-r1` borrada al terminar.

## Lecturas fuera del contexto permitido

- docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md y docs/adr/F1-3-flujo-de-ramas-worktrees-release.md: coherencia con F1-1..F1-3 pedida en la peticion.
- docs/plan/PLAN-TASKCODE-2.md, docs/plan/REQUISITOS-FUNCIONALES.md y docs/auditoria/AUDITORIA-EXTERNA-2026-11-13.md: existencia y texto de los IDs citados.
- tareas/03-en-revision/TASK-006/poc/poc-f1-5.sh, poc-f1-5.log y rulesets/*.json: entregable a reproducir (pedido en el encargo).

# Informe de revision — TASK-015 (ronda 1)

- Commit revisado: 91a8078c162a537a1538137c29ffd2048c1e6a1b
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

Alcance: pase generico (el diff es documentacion y PoC; ningun revisor de
dominio encaja). Profundidad: la tabla de 20 garantias y la PoC a fondo; los
ADR previos (F1-4, F1-5, F1-6, F1-7, F1-10) por contraste puntual con
`grep`, no releidos enteros.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-13-adaptador-de-plataforma.md:285 | implementacion |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-13-adaptador-de-plataforma.md:287 | implementacion |
| MEN-1 | MENOR | corregido | docs/adr/F1-13-adaptador-de-plataforma.md:528 | implementacion |
| MEN-2 | MENOR | corregido | tareas/03-en-revision/TASK-015/poc/poc-f1-13.py:176 | implementacion |
| MEN-3 | MENOR | corregido | docs/adr/F1-13-adaptador-de-plataforma.md:347 | implementacion |

Origen: `implementacion` en todos (el plan pedia fidelidad a la fuente y
sustitutos honestos; el ADR no lo cumple en estos puntos).

### IMP-1 — G09: Ultimate se vende como «origen del check» y la API oficial lo desmiente

Fichero `docs/adr/F1-13-adaptador-de-plataforma.md:285` (fila G09); efecto en
`:210-211` (capacidades `gitlab premium` 17/3/0 y `gitlab ultimate` 18/2/0),
`:303-305` (prosa: «1 garantia exige Ultimate (el origen de un check…)») y en
la salida de `doctor` de la PoC C (`G09 sustituida motivo=plan
arreglo=gitlab-ultimate`).

Reproduccion (documentacion oficial, consultada hoy):
`https://docs.gitlab.com/api/status_checks/`, seccion «Set status of an
external status check»: el token tiene que pertenecer a un usuario con rol
Developer, Maintainer u Owner en el proyecto destino, y la llamada la puede
ejecutar cualquier usuario con derecho a aprobar el MR. Es decir, el agente
(Developer, D4a) puede responder el check externo igual que hoy puede
publicar un estado de commit: el check externo de Ultimate **no da el
origen** que da `integration_id` en GitHub. Ademas
`https://docs.gitlab.com/user/project/merge_requests/status_checks/` dice que
por defecto un MR se puede mergear aunque fallen los checks externos y que
hay que activar «Status checks must succeed»; el ADR no lo menciona. El ADR
solo deja como «no verificado» (lineas 523-525) si los checks externos de
Ultimate solo los puede responder el servicio configurado, pero la
documentacion lo responde: no.

Consecuencia: el ADR cuenta G09 como `impuesta` en Ultimate y dice a `doctor`
que el arreglo es `gitlab-ultimate`; eso vende como la misma garantia (RR-08,
«sustituto != impuesta») algo que sigue siendo degradado, y manda a la
persona a pagar un plan que no lo arregla. Correccion: G09 en GitLab pasa a
tier «ninguno» (sustituida en todos los tiers, `motivo=plataforma`,
`arreglo=ninguno-limite-de-la-plataforma`, como G01 y G08), con
`autor-del-estado` como sustituto tambien en Ultimate (el check externo, como
defensa adicional, no como origen); recalcular capacidades (`ultimate`
17/3/0), cifras de la prosa (0 en Ultimate; 3 en ningun tier), resumen, fila
del indice y esperados de la PoC.

### IMP-2 — Filas marcadas «empirica» cuya unica llamada es un 401/403 anonimo

Fichero `docs/adr/F1-13-adaptador-de-plataforma.md:287` (G11: GL-04),
`:295` (G19: GL-22) y `:296` (G20: GL-21); tambien `:308-309` («17 garantias
tienen ademas evidencia empirica»).

Reproduccion: repeti GL-04 (`GET /projects/278964/protected_tags`) -> 403 y
GL-05 (`/approval_rules`) -> 401, identicos al log; GL-21 y GL-22 dan 401 en
mi ejecucion completa de la PoC. Un 401/403 anonimo prueba que el endpoint
existe, no que protected tags sea Free (G11), que las variables protegidas
solo lleguen a ramas protegidas o que el pipeline de un fork corra en el fork
(G19), ni que el runner se etiquete o proteja (G20). Para G11, G19 y G20 la
unica base real es la documentacion; marcarlas «empirica» infla la cifra de
17 y contradice la regla del plan («lo empirico con id de llamada»,
«documental» para lo no observado). Correccion: pasarlas a «documental»
(14 empiricas; ajustar PoC) o decir en la celda que la llamada solo prueba la
existencia del endpoint. Mismo reparo, menor, para G12 (GL-23, 401), donde
solo GL-09 aporta. Mi mutacion propia lo confirma: cambiar los ids de G15
(`GL-09, GL-20` por `GL-01, GL-04`) **no hace fallar la PoC**, porque solo
comprueba que el id exista, no que sirva para la fila.

### MEN-1 — «Si las cuentas de servicio cuentan para el limite de 5 usuarios: no verificado» ya esta respondido en la fuente

Fichero `docs/adr/F1-13-adaptador-de-plataforma.md:528` (y `:393-395`).
`https://docs.gitlab.com/user/profile/service_accounts/`: las cuentas de
servicio «do not use a seat», no son usuarios facturables, y en Free hay un
tope de 100 por grupo de nivel superior. Se puede cerrar con cita y fecha y
anotar el tope de 100 en el sustituto `cuenta-de-servicio` (G12).

### MEN-2 — La guardia de la PoC solo comprueba el metodo, no el destino

Fichero `tareas/03-en-revision/TASK-015/poc/poc-f1-13.py:176`. `_guardia`
compara el metodo y nada mas: `Lector.http` acepta cualquier URL y
`Lector.gh` pasa `ruta` sin validar a `gh api`. No es explotable hoy
(destinos constantes, argumentos como lista), pero el ADR (D6a) presenta el
envoltorio como la garantia de solo lectura con un token `repo` de escritura.
Recomendable: lista blanca de hosts (`api.github.com`, `gitlab.com`,
`127.0.0.1`) y rechazo de `ruta` que empiece por `-`. La prueba negativa si
discrimina: mutando `!=` por `.upper() !=` la PoC sale con 1 (NEG-04 «get no
se rechazo»).

### MEN-3 — Permisos de la identidad `agente` para las lecturas de GitLab sin comprobar

Fichero `docs/adr/F1-13-adaptador-de-plataforma.md:347`. `estado-puertas`
(identidad agente, F1-6) llama a `protected_branches` y `approval_rules` en
GitLab; las paginas de API (`api/protected_branches`,
`api/merge_request_approvals`) solo dicen que exigen autenticacion y no
nombran el rol minimo. El ADR da por hecho que un Developer las lee (decision
6) sin anotarlo en «No verificado»; si exigen Maintainer, el agente no puede
ejecutar `estado-puertas` y habria que asignarlas a la persona (F1-6).
Anadirlo a «No verificado» y a la bateria E2E de F2-9.

## Reproduccion

- **PoC completa** (`python -I -B`, carpeta
  `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-015-r1\`): sale 0, «todo
  como se espera», 38 GET, 15 procesos `gh`, 23 HTTP. Comparada con
  `poc-f1-13.log` (sin `\r`): igual salvo la seccion V (limites de peticiones
  y dos hashes de cuerpo, GL-11 y GL-17, que cambian como el ADR dice que
  puede pasar).
- **Mutaciones de control**: las 23 del log fallan cada una por su motivo.
  Propias sobre copia del ADR en `--solo-adr`: tier de G06 Premium->Free
  (falla), fecha 2026-11-13 en una celda (falla), tier de G10 en GitHub a
  «todos» (falla), sustituto inventado en G02 (falla), «5 en GitLab» -> «6»
  (falla); ids de evidencia de G15 cambiados (no falla, IMP-2). Mutacion en
  el codigo: la de MEN-2 (falla como debe).
- **GET propios**: GitLab.com anonimo `protected_branches/master` 200,
  `protected_tags` 403, `approval_rules` 401, `merge_requests/271075/approvals`
  200; `gh api --method GET` a `charliebk/TaskCodeTwo/rules/branches/develop`
  200 y a `charliebk/taskcodetwo-poc-f1-1/rulesets` 403 con «Upgrade to GitHub
  Pro or make this repository public». Coinciden con el log.
- **Celdas contrastadas contra la documentacion oficial** (coinciden con el
  ADR): cola de merge de GitHub (repos publicos de organizacion, o privados
  de organizaciones con Enterprise Cloud); rulesets (publicos con Free;
  publicos y privados con Pro, Team, Enterprise Cloud; leido con `curl` de la
  pagina porque el resumidor de WebFetch no veia la frase); tokens de
  proyecto (Premium o Ultimate en GitLab.com, cualquier licencia en
  self-managed); aprobaciones obligatorias Premium (en Free, opcionales);
  «Prevent approval by author», «by users who add commits» y «Remove all
  approvals» Premium; trenes Premium; checks de estado externos Ultimate
  (ver IMP-1); un Maintainer u Owner puede borrar una rama protegida solo por
  interfaz o API (el «por defecto apagado» del force push queda implicito en
  la pagina, no literal).
- **Envoltorio**: rechaza POST, DELETE, PUT y `get` antes de la red (0
  peticiones, 0 procesos `gh`, 0 en el mock); el registro solo contiene GET.
- **Coherencia con ADR previos**: operaciones de la lista cerrada correctas;
  `leer-permisos-actions` -> `leer-permisos-ci` anotado en la divergencia 2;
  `taskcode/tareas` y `taskcode-*` en la divergencia 1; fila F1-13 del indice
  presente con las cifras de la PoC. Sin contradiccion encontrada.
- **Seguridad**: sin token personal en GitLab (D4a), agente Developer y
  persona unica Maintainer, `verified_system` excluida, aprobaciones de bots
  contadas por id. Por mi parte, solo lecturas en ambas plataformas.
- No reproducido por mi: paginas de precios (D15, L26), CODEOWNERS de GitLab
  (L05), `merge_group` y el resto de celdas documentales.

## Lecturas fuera del contexto permitido

- `poc/` (poc-f1-13.py, log, fuentes.md, fixtures) y los ADR F1-4, F1-5,
  F1-6, F1-7 y F1-10 (grep): los pidio el encargo para reproducir y
  contrastar.

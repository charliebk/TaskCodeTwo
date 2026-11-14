# Informe de revision — TASK-009 (ronda 2)

- Commit revisado: fb9e2a9cf4f4761ed65e2f3c1f7f13405ee2607f
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

Veredicto propuesto por el revisor: **cambios-solicitados** (1 IMPORTANTE abierto, IMP-4, introducido por la propia correccion; IMP-1 a IMP-3 verificados como corregidos). La correccion es pequeña: basta con añadir la salvedad a la celda de `tocar workflows` (ver IMP-4).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:75 | plan |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:381 | implementacion |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:417 | implementacion |
| IMP-4 | IMPORTANTE | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:417 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Las lineas de IMP-1 a IMP-3 son donde vive hoy la correccion (en la ronda 1 eran :74, :80 y :375). IMP-4 es el unico hallazgo nuevo: lo introduce la correccion de IMP-3 (la celda de la fila del agente local, linea 417, añade «Sin salvedad» a dos capacidades que dependen del mismo token de la persona). Abierto por ser IMPORTANTE e introducido por la correccion.

### Verificacion de los pendientes

**IMP-1 (corregido).** A7 se declara **parcial** (fila de la matriz, linea 75: «mitigacion **parcial**»), y la decision 7 (lineas 474-490) fija la regla de `buscar`: no hay acierto (se rechaza toda linea `arbol=`) si el diff del PR frente a su base toca `.github/workflows/**` o `.taskcode/**`. El caso que faltaba esta en la PoC B2 y lo he reproducido: «publicador de un workflow modificado en el PR (toca .github/workflows/verificacion.yml)» → la regla de F1-7 acepta, la regla de la decision 7 rechaza; «publicador con el PR tocando .taskcode/config.yml» → rechaza; «publicador con el PR tocando solo codigo» → acepta (control positivo). Resultado: `casos decision 7=10/10 ; falsos que aceptaria la regla de F1-7=4 ; solo job publicador=2`. P1 (linea 517), P12 (linea 528) y la divergencia 1 (lineas 537-542) recogen la regla. La regla de `buscar` choca con F1-7 solo en lo que la divergencia 1 declara (F1-7, decision 3, D5a); no hay choque sin declarar (ver «Verificacion realizada»). Aviso: B2 es una simulacion del calculo de la regla sobre entradas fijadas; que `buscar` lea el diff del PR en el evento real queda «no verificado» (F2-6), como dice el ADR.

**IMP-2 (corregido).** El lint se declara heuristico en la decision 1 (lineas 381-389), en la fila A6 (linea 74), en la fila A13 (linea 81: «heuristico: ver «No verificado»») y en «No verificado» (lineas 654-661, que lista los huecos conocidos). F5-1 lo formula como lista blanca (decision 1, linea 386-389; consecuencias, linea 585-591). Reproduccion de M1–M5 sobre ficheros de workflow propios en la carpeta temporal, con el `clases.py` real de la PoC (`python -I`):
- M1 `pull_request_target` + checkout con `repository:` del head → `R1 ejecuta codigo del PR y tiene privilegio`. Detectado.
- M2 `pull_request_target` + `gh pr checkout "$PR"` con el numero por `env:` → `R8` (y `R1`). Detectado.
- M3 titulo del PR en `actions/github-script` (`script:`) → `R5`. Detectado.
- M4a `github['event']['pull_request']['title']` en `run:` → `R5`. Detectado.
- M4b `toJSON(github.event)` en `run:` → `R5`. Detectado.
- M5 job que llama a workflow reutilizable con `secrets: inherit` en `pull_request` → `R9` (y `R1`). Detectado.
La PoC B1 (20/20 controles negativos, incluidos N15–N20) lo confirma. Mutacion propia distinta (X1): ver «Notas». No es un hallazgo nuevo porque el lint nunca se afirma completo y la lista blanca de F5-1 lo cubre.

**IMP-3 (corregido en lo pedido).** La salvedad «salvo A4/A5» aparece en la celda «No puede» de la fila del agente local (linea 417), en el estado de la decision (lineas 420-425: «garantia de la plataforma para el revisor IA en CI; para el agente en local, condicionada a resolver `A5`»), en H9 (linea 565), en RR-08 (linea 567), en RN-06 (linea 573, `A5` como riesgo abierto), en «Preguntas abiertas» (linea 641) y en la divergencia 4 (lineas 550-553). Las celdas «aprobar», «integrar sin la aprobacion de la persona» y «firmar un approve» quedan cubiertas por la salvedad. Lo que no queda coherente es la parte de «tocar workflows» de la misma celda: IMP-4.

### IMP-4 — La celda de «tocar workflows» declara sin salvedad lo que depende del token de la persona

- Donde: docs/adr/F1-8-seguridad-modelo-de-amenazas.md:417 (fila «Agente en local» de la decision 4): «Sin salvedad: tocar workflows con su identidad (sin `workflows`, F1-6); publicar checks con su identidad (sin `checks:write`)».
- Que pasa: la misma celda dice «**salvo A4/A5**, mientras el token y la clave de la persona estén al alcance de su sesión, la plataforma no lo impide» para aprobar, integrar y firmar, y justo a continuación declara sin salvedad que el agente no puede tocar workflows «con su identidad». Pero A5 (linea 73) es el token de la persona que el agente saca del llavero con `gh auth token` y usa como la persona, y la restriccion que se cita (`workflows`, F1-6) es un permiso de la **App del agente**, no del token de la persona. Si el token de la persona tiene el scope `workflow`, el agente puede empujar cambios en `.github/workflows/**` como la persona, y con ellos cambiar el lint de clases, los `permissions` o la regla de `buscar` que el ADR presenta como garantia.
- Reproduccion: lectura cruzada de las lineas 73 (A5), 417 (celda) y 565/567 (H9 y RR-08 usan la salvedad). Comprobacion en la PoC D: el log muestra `gh auth` con el token en el llavero, pero ningun paso registra los scopes del token (grep de `scope` en el log: sin coincidencias). Por tanto, la ausencia de `workflow` en el token no esta demostrada en el ADR. **No verificado**: caso exacto a ejecutar en la maquina de la persona, sin mostrar el token: `gh auth status` (linea «Token scopes»). Si incluye `workflow`, la celda es falsa tal como esta. La parte de «publicar checks» sí es plausible por diseño (los check-runs los crea una App, no un token de usuario), pero tampoco esta evidenciada en el ADR.
- Impacto: un lector de la decision 4 concluye que el agente no puede alterar workflows con la identidad de la persona, que es justo lo que A5 deja abierto. Es la misma clase de cierre sin evidencia que IMP-3, aplicada a `workflows`, y el ADR la usa como garantia (RR-08, H9).
- Sugerencia: aplicar la salvedad a la celda completa, por ejemplo: «**salvo A4/A5**: tocar workflows con su identidad solo lo impide la plataforma si el token de la persona no tiene el scope `workflow` (no verificado: `gh auth status`); publicar checks con su identidad: no, los check-runs los crea una App (no verificado)». Y añadir a «No verificado» el scope del token de la persona.

## Verificacion realizada

- Rama `feature/task-009-f1-8-adr-seguridad`, HEAD `441d966`. `git diff fb9e2a9 HEAD` solo toca `tareas/` (informes, peticion y `tarea.md`): el ADR `docs/adr/F1-8-seguridad-modelo-de-amenazas.md` y la PoC `tareas/03-en-revision/TASK-009/poc/poc-f1-8.sh` son los del commit revisado.
- PoC ejecutada en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-009-r2` (no en `C:\` ni en el repo): `bash poc-f1-8.sh <carpeta> docs/adr/F1-8-seguridad-modelo-de-amenazas.md /c/Users/nullcad2025/Documents/sources/Personal` → salida 0 (33 s). Resultados: A1 7/7, controles negativos 8/8, limite 1/1; A2 4 ordenes; B1 positivos 5/5, negativos 20/20; B2 decision 7 10/10 (falsos que aceptaria F1-7: 4; solo job publicador: 2); C errores ADR real 0, activos 16, preguntas heredadas 13, mutaciones 14/14, no aplicadas 0; D 16/16; llamadas `gh api` con GET 15, con otro metodo 0. Coincide con las lineas `RESULTADO` registradas en la peticion.
- Mutaciones propias del lint (M1–M5 y X1) sobre ficheros en la carpeta temporal, con `clases.py` de la PoC (`python -I`): M1–M5 detectados; X1 no (ver «Notas»).
- Salvedad «salvo A4/A5»: grep de `A4|A5|salvo|condicion` en todo el ADR y lectura de las lineas 56-85, 360-436, 530-681. Coherente salvo IMP-4.
- Choque de `buscar` con F1-7 (`docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md`, lectura por grep de `arbol=`, `buscar`, `check-run`, `workflows`): F1-7 decision 3 (D2a) acepta como acierto cualquier linea `arbol=` de un check `verificacion` de la App 15368 en el commit, sus padres y los padres de estos, sin mirar que ficheros toca el PR. La regla nueva rechaza esas lineas cuando el PR toca `.github/workflows/**` o `.taskcode/**`. Esto **si** cambia F1-7, y **esta declarado** en la divergencia 1 del ADR (lineas 537-542: «nunca hay acierto en un PR que toca `.github/workflows/**` o `.taskcode/**`»), que cita F1-7 decision 3 y D5a. No hay choque sin declarar. Consecuencia no escrita en la divergencia: esos PR verifican entero y pierden el acierto de cache; es un coste, no una contradiccion de F1-7 (RR-07 se mantiene).
- `bash .github/scripts/comprobar-repo.sh`: «Tareas: 93; enlaces relativos: 62; fallos: 0», salida 0.
- Carpeta temporal `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-009-r2` (con su log) borrada al terminar. No habia commits ni cambios en el repo; no se hizo ningun commit.
- No ejecutado: `taskctl doctor`; ninguna llamada a GitHub salvo lo que hace la PoC con GET; no se leyeron tokens ni se invocó `gh auth token`.

## Notas (fuera de la tabla)

- **X1 (lint, no es hallazgo nuevo)**: un job de clase privilegio en `workflow_run` que descarga un artefacto y ejecuta `bash "$GITHUB_WORKSPACE/artifact/run.sh"` no dispara R7 (`clases.py` solo reconoce la ruta del artefacto sin prefijo). Es un hueco del lint heuristico ya declarado en «No verificado» y cubierto por la lista blanca de F5-1; el codigo del lint no cambia en esta ronda. Se anota para F5-1: el caso debe entrar en su batería.
- **Principio en Contexto (linea 31)**: «la defensa contra prompt injection va por lo que cada LLM no puede hacer» es cierto para el revisor IA y, tras A5, condicionado para el agente local. La decision 4 ya lo dice; no se abre hallazgo.
- Los huecos ya declarados en «No verificado» (scripts que traen el head por otra via, expresiones en otros campos de `with:`) no se han vuelto a revisar.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-009/poc/poc-f1-8.sh` (lineas 569-660, funcion `clases.py` y harness B1, y grep de `decision`/`workflows/` para ver como B2 calcula la regla): para reproducir y entender el lint y la simulacion de B2, que pide la peticion.
- `tareas/03-en-revision/TASK-009/poc/poc-f1-8.log` (grep de `scope`/`workflow`): para comprobar si la PoC D registra scopes del token (no los registra).
- `tareas/03-en-revision/TASK-009/revision/informe-revision-1.md` y `peticion-revision-2.md`: instrucciones de la ronda y hallazgos a verificar, pedidos por la peticion.
- `docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md` (grep de `arbol=`, `buscar`, `check-run`, `workflows`): para el choque de `buscar` con F1-7, que pide la peticion.
- Ni `.github/workflows/ci.yml` ni `docs/plan/` ni `docs/auditoria/` se han leido en esta ronda.

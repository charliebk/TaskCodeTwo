# ADR F1-8 — Seguridad: qué corre en local y en CI, confianza, prompt injection y runner propio

- Estado: **aceptada** (2026-11-14)
- Tarea: TASK-009 · Plan: [F1-8 en PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) (§4 P3, P6 y P13 y tabla de la Fase 1)
- Requisitos: la fila F1-8 del plan no cita ninguno; este ADR responde a RR-03, RR-08, RR-13, RR-15, RR-18, RF-70, RN-02 y RN-06 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md), a las viñetas 3 y 4 de P6 del plan y a H2 y H9 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
- Depende de: [ADR F1-5](F1-5-puertas-en-github.md) (checks `carril`, `verificacion` y `revision-ia`, `integration_id` 15368, CODEOWNERS en `/.github/` y `/.taskcode/`), [ADR F1-6](F1-6-identidades-agente-revisor-persona.md) (dos Apps, el agente sin `workflows` ni `secrets`, la clave del revisor en Actions —D5a—, firma de la persona con FIDO o frase sin caché) y [ADR F1-7](F1-7-verificacion-ci-perfiles-cache-runners.md) (caché por árbol en el resumen del check, runner por perfil, runner propio solo en privados, hook con la rápida)
- Decide: Carlos, con la consigna «la opción recomendada» (D1b, D2b, D3b, D4b y D5a del plan de TASK-009)
- Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)

## Contexto

Tres ADR aceptados dejaron aquí sus preguntas de seguridad: F1-5, el check
`carril` con un workflow que el propio PR puede cambiar; F1-6, dónde vive
cada credencial, `repository_dispatch`, las claves de agente y persona en la
misma máquina y la tarea que toca workflows; F1-7, el runner propio con
código del PR, quién puede publicar un `verificacion` con un resumen falso y
la confianza del hook. El marco:

- **P6 / H2**: hoy el hook `Stop` de la 0.16 ejecuta `comando_verificacion`
  del árbol sin preguntar (la auditoría lo reprodujo con `echo PWNED`). En la
  2.0, nada del repo se ejecuta en local sin confianza explícita (F0-2, F5-6).
- **RF-70**: `comando_sincronizacion` corre «en un job de CI o en un hook
  local con confianza».
- **RR-03**: el revisor reproduce y prueba mutantes, es decir, **ejecuta**
  código del PR; F1-6 (D5a) le dio una clave de App en Actions.
- **RN-02**: runner propio Windows, en la máquina de Carlos, donde están su
  `ssh-agent`, su token y los 34 GB de `exampleData` de ToolsaWeb.
- **RR-18 / P13**: lotes desatendidos; el agente encadena tareas sin
  persona hasta el PR final del lote y lee más texto que nadie revisa.
- **RR-08**: las garantías las impone la plataforma y la CI, no las
  instrucciones al agente. Por eso la defensa contra prompt injection va por
  lo que cada LLM **no puede hacer**, no por detectar el texto hostil.
- **RN-06**: una sola persona con agentes, en la misma máquina.

Preguntas que responde: qué código corre dónde y con qué credenciales; qué
puede publicar un check obligatorio; cómo se da y se pierde la confianza en
local; qué puede hacer un LLM que lee texto hostil; con qué condiciones entra
el runner propio; qué se hace cuando un secreto se filtra.

## Modelo de amenazas

Sección añadida a la plantilla (lo permite el README de `docs/adr/`): el
criterio de F1-8 es un modelo de amenazas, y las alternativas solo se
entienden contra él. Se ordena por **fronteras de confianza**, no por una
lista de ataques, porque la misma configuración y el mismo texto hostil son
entrada a la vez del hook, del workflow y del LLM.

### Zonas

| Zona | Qué hay | Quién manda | Se confía en |
|---|---|---|---|
| **Z1** Máquina de la persona | CLI `taskctl`, agente, hook, worktrees, clave de la App del agente, clave de firma y token de la persona | la persona; el agente corre en su misma sesión (RN-06) | el CLI y lo que la persona confió (D4b); el agente **no** |
| **Z2** Contenido del repo | `.taskcode/` (los `comando_*`), `.github/`, scripts, código, texto de tareas, planes, `## Resultado` y descripción de PR, rama `taskcode/tareas` | quien empuja: el agente, sin PR en `taskcode/tareas` (F1-1, F1-6) | **nada**: es dato no confiable, en local y en CI |
| **Z3** Runners de GitHub | jobs de Actions, `GITHUB_TOKEN`, secretos de entorno | el workflow de la base (clase privilegio) o del PR (clase ejecuta) | solo los jobs de clase privilegio, que no ejecutan nada del PR |
| **Z4** Runner propio | runner Windows en la máquina de Carlos (RN-02) | quien registra el runner | solo con las condiciones de D3b; si no, no corre jobs de PR |
| **Z5** Plataforma | rulesets, CODEOWNERS, Apps, ajustes de Actions, `integration_id` | la persona como administradora | sí; `doctor` lo compara con el modelo |

### Activos, amenazas y mitigaciones

Gravedad: **CRÍTICA** (se salta una puerta o filtra un secreto sin vuelta
atrás), **ALTA** (verde falso en un check obligatorio), **MEDIA** (pérdida de
trabajo o de estado). Cada fila tiene mitigación **o un riesgo aceptado
nombrado** (`A5`, `A15`), y tarea dueña; lo que no se puede comprobar con
`GET` lleva «no verificado → F2-6». La PoC C lee esta
tabla entre los marcadores `matriz-amenazas` y comprueba cada fila.

<!-- matriz-amenazas:inicio -->
| ID | Activo | Zona | Quién lo lee o escribe | Amenaza (gravedad) | Mitigación | Dueña |
|---|---|---|---|---|---|---|
| `A1` | Clave de la App del agente | Z1 | el CLI la lee para firmar sus tokens | código del repo ejecutado en local (hook, `comando_*`) la lee y actúa como agente fuera del CLI (ALTA) | ningún `comando_*` corre sin confianza (D4b); la clave fuera del árbol; permisos mínimos de F1-6 (sin `workflows`, `administration` ni `secrets`), así que robarla no salta ninguna puerta; rotación (decisión 8) | F0-2, F5-6, F6-1 |
| `A2` | Clave privada de la App del revisor | Z3 | solo el job publicador de `revision-ia` (clase privilegio) | código del PR la lee si comparte job con él y publica `revision-ia` en verde como revisor (CRÍTICA) | secreto de entorno solo en el job publicador, que no ejecuta nada del PR (decisiones 1 y 3); entorno limitado a las ramas base; rotación (decisión 8) | F6-1, F6-3, no verificado → F2-6 |
| `A3` | Clave del modelo del revisor IA | Z3 | solo el job publicador (D2b) | código del PR la roba en el job de reproducción (CRÍTICA) | la clave vive en el job publicador; la reproducción corre en otro job sin secretos y le pasa un artefacto que se lee como dato (decisión 3) | F6-3, F6-4, no verificado → F2-6 |
| `A4` | Clave de firma de la persona | Z1 | `approve` (F4-2) con TTY | el agente, en la misma sesión, firma un `approve` falso si la clave está en un `ssh-agent` o llavero desbloqueado (CRÍTICA, sin vuelta atrás: `taskcode/tareas` no admite borrado) | FIDO con toque (D4a de F1-6) o frase sin caché (D4b de F1-6); nunca cargada en un agente de claves sin confirmación por uso; `doctor` avisa si el tipo de clave registrada no es `sk-` ni tiene frase (no comprobable desde la plataforma); rotación (decisión 8) | F4-2, F3-8, no verificado → F2-6 |
| `A5` | Token de la persona | Z1 | `merge-pr`, `aplicar-puertas`, `leer-permisos-actions` (F1-6) | el agente lo saca del llavero del mismo usuario del sistema (`gh auth token`) y aprueba o mergea como la persona (CRÍTICA) | F1-6 lo limita a operaciones con TTY, pero eso lo impone el CLI, no la plataforma; queda como riesgo abierto (ver «Preguntas abiertas»): el agente en otro usuario del sistema o sin acceso al llavero, o un token de la persona que no queda guardado | decide un ADR nuevo de Fase 1, «aislamiento del agente frente a las credenciales de la persona» (sin fila en el plan: acción en «Consecuencias»); implementan F2-6, F3-8; no verificado → F2-6 |
| `A6` | `GITHUB_TOKEN` de los jobs que ejecutan código del PR | Z3 | el job y todo lo que el job ejecuta | scripts del PR (`verificar.bat`, `build.rs`, `postinstall`) usan un token con `checks:write` para crear `carril` o `verificacion` en verde (ALTA) | `permissions: contents: read` explícito en todo job de clase ejecuta (decisión 1); permisos por defecto de Actions en solo lectura, que `doctor` exige (leído: `read` en los tres repos, PoC D); lint estático de clases (PoC B), **heurístico**: busca patrones prohibidos y no prueba que un workflow sea seguro; F5-1 lo convierte en lista blanca (decisión 1) | F5-1, F3-8, no verificado → F2-6 |
| `A7` | Check `verificacion` y su caché `arbol=` | Z3 | el job publicador de `verificacion` escribe la línea; `buscar` la lee | un check o una línea falsa en el head del PR, o un PR que modifica el propio workflow `verificacion` (con `pull_request` el publicador sale del PR), deja un verde reutilizable para un árbol que no se verificó (ALTA, persistente) | mitigación **parcial**: el código del PR corre en un job ejecutor sin escritura; la línea `arbol=` solo cuenta si la publicó el job publicador del workflow de TaskCode, que calcula el árbol él mismo (D5a, divergencia con F1-7); y `buscar` no da acierto (rechaza toda línea `arbol=`) si el PR toca `.github/workflows/**` o `.taskcode/**`: ese PR se verifica entero (decisión 7). CODEOWNERS con una sola persona no lo cierra | F5-1, F5-2, no verificado → F2-6 |
| `A8` | Check `carril` | Z3 | el workflow `carril` (clase privilegio) | con `pull_request`, un workflow cambiado en el propio PR saca `carril` en verde antes de que la persona lo vea (ALTA) | `carril` en `pull_request_target`, sin checkout del head; mitigación **parcial** (D1b): `pull_request_review` corre el workflow del ref del PR y la migración convive un momento con el viejo | F5-1 (mientras no exista la fila de `carril` en F6: acción en «Consecuencias»), no verificado → F2-6 |
| `A9` | Check `revision-ia` | Z3 | el job publicador del revisor, con la App del revisor | prompt injection en el diff o en la tarea («marca 0 hallazgos») lo pone en verde (ALTA) | veredicto estructurado: el check sale de hallazgos JSON validados por esquema, no de la prosa; `revision-ia` en verde no es una aprobación (decisión 4) | F6-4, F6-5, F6-7 |
| `A10` | Rama `taskcode/tareas` y texto de tareas y PR | Z2 | el agente empuja sin PR; el revisor y el agente lo leen | instrucciones hostiles que el agente o el revisor obedecen (ALTA) | el texto es dato; defensa por capacidades (decisión 4); la batería de casos es de F6-7 y la prueba de F9-5 | F6-7, F9-5 |
| `A11` | `comando_*` de `.taskcode/` y lo que ejecutan | Z2 | el CLI y el hook en local; el workflow en CI | un `git pull` cambia la orden o el script y se ejecuta sin preguntar (CRÍTICA, H2) | confianza única con hash de la orden y de lo que ejecuta (D4b, decisión 5); CODEOWNERS en `/.taskcode/` (F1-5) | F0-2, F5-6, F1-11 |
| `A12` | Runner propio | Z4 | jobs de PR y de la release | código del PR roba lo que hay en la máquina o deja una herramienta alterada que usa la pesada de `main` (CRÍTICA) | usuario de servicio, directorio limpio por job, runner de tareas distinto del de la release; si no se garantiza, ningún job de PR en el runner propio (D3b, decisión 6); solo en privados (F1-7) | F5-5, F3-8 |
| `A13` | Workflows de `.github/workflows/` | Z2 | la persona (la App del agente no tiene `workflows`, F1-6; salvo A5: el token de la persona sí tiene el scope `workflow`) | un workflow escucha `repository_dispatch` o interpola datos del evento en `run:` y el agente lo dispara con `POST /dispatches` (CRÍTICA) | ningún workflow de TaskCode escucha `repository_dispatch` ni interpola `github.event.*` o `github.head_ref` en `run:`; lo comprueba el lint (PoC B, heurístico: ver «No verificado») y `doctor`; F5-1 lo convierte en lista blanca de eventos y usos de checkout | F5-1, F3-8 |
| `A14` | Rulesets y ajustes de Actions | Z5 | la persona como administradora | se relaja un ajuste (permisos por defecto en escritura, `can_approve_pull_request_reviews`, workflows de forks en privados) (ALTA) | `doctor` los lee con `GET` y los compara con el modelo (PoC D); `init` los aplica | F3-8, F3-1 |
| `A15` | Worktrees y estado local sin commitear | Z1 | el CLI y la persona | un script del PR lanza `git clean -ffdx` o borra el worktree de `tareas/` en una verificación local (MEDIA) | ninguna; riesgo aceptado y nombrado (ver «Preguntas abiertas»); la confianza (D4b) reduce cuándo corre código del repo | F5-6, F1-16 |
| `A16` | Lote desatendido (RR-18) | Z1 | el CLI encadena tareas con el agente | una tarea con texto hostil lleva a las siguientes del lote (ALTA) | el lote nunca integra en `develop` sin el PR final que aprueba la persona (P13); las mismas capacidades que fuera de lote | F1-16, F4-7 |
<!-- matriz-amenazas:fin -->

### Dos clases de jobs

Toda la superficie que el agente controla en CI **no es el YAML** (no tiene
`workflows`), sino lo que el YAML ejecuta. Por eso cada job de TaskCode es de
una sola clase:

| Clase | Qué hace | Eventos | `permissions` | Secretos | Entrada del PR |
|---|---|---|---|---|---|
| **Ejecuta código del PR** | `verificacion` (job ejecutor), reproducción y mutantes del revisor (F6-4), `comando_sincronizacion` en CI (RF-70) | `pull_request` (y `merge_group`) | `contents: read` y nada más | ninguno | se ejecuta |
| **Tiene privilegio** | publica `verificacion` (job publicador), `carril`, publica `revision-ia` con la App del revisor | `pull_request_target` o `workflow_run`; el publicador de `verificacion`, en el mismo `pull_request` como job aparte (por eso su workflow sale del PR y la mitigación es parcial: decisión 7) | el mínimo de su escritura | solo los suyos, de entorno | **solo como datos** (diff por la API, artefactos validados); nunca checkout del head ni ejecución de nada del PR |

Un job que ejecuta código del PR **y** tiene privilegio es un error de
diseño; el lint (PoC B, heurístico) detecta los patrones conocidos y
`doctor` lo rechaza.

## Alternativas

| | Descripción | Veredicto |
|---|---|---|
| **Un ADR por fronteras** | Un solo modelo con cinco zonas y una matriz de activos. | **Elegida**: la configuración y el texto hostil son entrada del hook, del workflow y del LLM a la vez. |
| Tres ADR (local, CI, inyección) | Uno por parte del enunciado. | Descartada: parte una frontera que es una sola. |
| **Dos clases de jobs** | Ejecuta código del PR sin escrituras ni secretos / tiene privilegio sin código del PR. | **Elegida** (decisión 1). |
| Todo en `pull_request_target` | Workflows de la base con secretos para todo. | Descartada: ejecutar el head del PR con secretos es justo el patrón a evitar. |
| Todo en `pull_request` | Workflows del PR, con escrituras donde haga falta. | Descartada: los scripts del PR tendrían `checks:write` o secretos. |
| D1a `carril` en `pull_request_target` como mitigación cerrada | Lo que propone F1-5. | Descartada: no cubre `pull_request_review`. |
| **D1b** `carril` en `pull_request_target`, mitigación parcial | Igual, marcado parcial y con E2E en F2-6. | **Elegida** (decisión 2). |
| D2a Clave del modelo en el job que ejecuta | El LLM reproduce él mismo. | Descartada: el código del PR roba la clave. |
| **D2b** Clave del modelo en el job publicador | El LLM lee como dato el artefacto de la reproducción, hecha sin secretos. | **Elegida** (decisión 3). |
| D2c Aplazar a F6-3 | Esperar a saber si hace falta clave. | Descartada: la forma de los dos jobs la heredan F6-3, F6-4 y F6-7. |
| Detectar el prompt injection | Filtrar o puntuar el texto hostil. | Descartada como garantía (RR-08): un filtro se esquiva; se puede usar como señal (decisión 4). |
| **Defensa por capacidades** | Lo que cada LLM puede hacer leyendo texto hostil. | **Elegida** (decisión 4). |
| D3a Runner propio con F1-7 dec. 5 | Repo privado y sin PR de forks. | Descartada: no protege la máquina de Carlos ni la pesada de `main`. |
| **D3b** Runner propio aislado | Usuario de servicio, directorio limpio, `main` separado; si no, ningún job de PR. | **Elegida** (decisión 6). |
| D4a Hash de la config | Confianza atada al hash de `.taskcode/`. | Descartada: un `git pull` cambia el script y se ejecuta sin preguntar (H2). |
| **D4b** Hash de la orden y de lo que ejecuta | Misma interfaz única para F0-2, F5-6 y RF-70. | **Elegida** (decisión 5). |
| Confianza por ruta del repo | Se confía en el repo, no en su contenido. | Descartada: es H2 con otro nombre. |
| **D5a** `arbol=` solo del job publicador | Cambia F1-7. | **Elegida** (decisión 7). |
| D5b Solo separar jobs | No tocar F1-7. | Descartada: un check falso dejaría la caché envenenada. |

## Pruebas de concepto

Guion `poc-f1-8.sh` y su salida completa `poc-f1-8.log`, en la carpeta `poc/`
de TASK-009 (sin enlace: la carpeta de la tarea cambia de estado al
cerrarse). Ejecutado en Git Bash con Git 2.56.0, `gh` 2.95.0, Python 3.12
(`python -I`) y PyYAML 6.0.3 bajo
`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-009\poc\` (borrada al
terminar). ToolsaWeb y OpenDockTrading **solo se leen**: el guion lee sus
ficheros para medir el hash de confianza y nunca ejecuta nada de ellos.
**Solo lectura en GitHub**: toda llamada pasa por `gh api --method GET`; el
guion las registra y cuenta las de otro método. **La matriz de activos y la
tabla de preguntas heredadas se leen de este fichero**, entre los marcadores
`matriz-amenazas` y `preguntas-heredadas`. El guion sale con código distinto
de 0 si alguna sección falla.

```bash
bash poc-f1-8.sh "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-009-c1" \
  docs/adr/F1-8-seguridad-modelo-de-amenazas.md \
  /c/Users/nullcad2025/Documents/sources/Personal > poc-f1-8.log 2>&1
```

Resultado de la ejecución registrada en el log, tras la revisión 1 (2026-11-14T08:27:41Z,
en `...\task-009-c1\`, borrada al terminar; salida 0):

```text
RESULTADO A1 casos=7/7 ; controles negativos=8/8 ; limite documentado=1/1 ; almacen fuera del arbol=si
RESULTADO A2 ordenes medidas=4 ; sin orden o cierre vacio=0
RESULTADO B1 positivos sin errores y con su clase=5/5 ; controles negativos=20/20
RESULTADO B2 casos decision 7=10/10 ; falsos que aceptaria la regla de F1-7=4 ; solo job publicador=2
RESULTADO C errores ADR real=0 ; activos=16 ; preguntas heredadas=13 ; mutaciones detectadas=14/14 ; no aplicadas=0
RESULTADO D comprobaciones=16/16
RESULTADO llamadas gh api con GET=15 ; gh api con otro metodo=0
```

### A — Hash de confianza local

**A1.** Prototipo de `confianza(repo, comando)` (decisión 5) sobre un repo de
juguete con `comando_verificacion: "bash scripts/verificar.sh --rapida"`,
donde `verificar.sh` carga `scripts/lib/paso.sh` y recorre `tests/*.sh`. El
almacén vive en la carpeta de trabajo, fuera del árbol:

```text
  ok   caso  nunca     sin confianza dada
  ok   caso  confiado  tras taskctl confiar
  ok   caso  confiado  README cambiado; orden y scripts iguales
  ok   neg   cambiado  orden cambiada en la config
  ok   caso  confiado  config devuelta al contenido confiado
  ok   neg   cambiado  script nombrado por la orden cambiado, config igual (H2)
       D4a (hash de la config) daria: confiado  <- el hueco de H2
  ok   caso  confiado  script devuelto al contenido confiado
  ok   neg   cambiado  script transitivo (lo carga el nombrado) cambiado
  ok   neg   cambiado  script nombrado borrado
  ok   neg   nunca     el repo trae su propio confianza.json; almacen de la persona vacio
  ok   caso  confiado  worktree nuevo del mismo repo (F1-3), mismo contenido
  ok   neg   nunca     otro clon con el mismo contenido
  ok   lim   confiado  LIMITE: test alcanzado por glob (tests/*.sh), sin glob declarado
  ok   neg   cambiado  se declara el glob tests/*.sh en la config
  ok   caso  confiado  confianza dada de nuevo con el glob
  ok   neg   cambiado  test cambiado con el glob declarado
```

Lectura: con el hash de la config (D4a) el script cambiado se ejecutaría sin
preguntar, que es H2; con D4b se vuelve a pedir, también si cambia un script
que solo carga el nombrado. El límite es el código al que se llega por un
glob o por una herramienta: solo entra si se declara.

**A2, coste con los pilotos** (solo se leen sus ficheros; ToolsaWeb en
`93c9618a`, OpenDockTrading en `fab8d7e`). Huella calculada cinco veces por
orden; «cierre» = ficheros versionados que se ejecutan como script;
«commits» = de los últimos 100 de `develop` sin merges, cuántos cambian el
cierre (cada uno obliga a volver a confiar):

```text
  repo             clave                       nombrados  cierre      bytes  ms (max)  commits
  ToolsaWeb        comando_verificacion                1      51    1246280     778,2        8
  ToolsaWeb        comando_verificacion_freno          1      51    1246280     763,7        8
  ToolsaWeb        comando_sincronizacion              1      51    1246280     928,8        8
  OpenDockTrading  comando_verificacion                1      19     731381     451,2        9
```

Lectura: la orden nombra un solo fichero y lo que ejecuta son 51 scripts en
ToolsaWeb (`herramientas.bat`, `scripts/*.py`, …) y 19 en OpenDockTrading;
hashear solo lo nombrado dejaría fuera casi todo. Volver a confiar tocaría
cada 11 o 12 commits. El cálculo en frío cuesta de 0,4 a 0,9 s, casi todo el
presupuesto de 1 s del hook de F5-6. Aviso del guion: con YAML estándar,
`"scripts\verificar.bat"` de ToolsaWeb se lee como `scripts\x0berificar.bat`
(`\v` es un escape); el guion usa la orden literal de la línea.

### B — Clases de jobs y la regla D5a

**B1.** Lint de clases (prototipo del de F5-1 y `doctor`) sobre los cuatro
workflows de la sección «Cómo se ve en GitHub», el `ci.yml` real de este repo
y 20 workflows con un fallo cada uno (N15–N20 son las variantes que la
revisión 1 encontró sin detectar). Un job es de clase ejecuta si hace
checkout en un evento de PR (o llama a un workflow reutilizable en él), hace
checkout del head por `ref:` o `repository:`, trae el head en `run:` (`gh pr
checkout`, `git fetch … pull/`) fuera de un evento de PR, ejecuta un fichero
de un artefacto o corre en `pull_request_review`; tiene privilegio si
escribe, usa secretos, un entorno o `secrets: inherit`. Es **heurístico**:
busca patrones conocidos (ver «No verificado»).

```text
  ok   pos  P1-verificacion            clases={'buscar': 'ejecuta', 'ejecutar': 'ejecuta', 'verificacion': 'privilegio'} errores=0
  ok   pos  P2-carril                  clases={'carril': 'privilegio'} errores=0
  ok   pos  P3-revision-reproducir     clases={'reproducir': 'ejecuta'} errores=0
  ok   pos  P4-revision-ia             clases={'revision-ia': 'privilegio'} errores=0
  ok   pos  P5-ci-real                 clases={'repo': 'ejecuta'} errores=0
  ok   neg  N01 pull_request con checks: write                 -> R1 ejecuta codigo del PR y tiene privilegio
  ok   neg  N02 pull_request con un secreto en run:            -> R1
  ok   neg  N03 pull_request_target con checkout del head      -> R1
  ok   neg  N04 repository_dispatch                            -> R4
  ok   neg  N05 titulo del PR interpolado en run:              -> R5
  ok   neg  N06 sin bloque permissions                         -> R2
  ok   neg  N07 permissions: write-all                         -> R3 (y R1)
  ok   neg  N08 workflow_run que ejecuta el artefacto          -> R7 (y R1)
  ok   neg  N09 pull_request_review con checks: write          -> R6 (corre el workflow del PR; D1b)
  ok   neg  N10 github.head_ref interpolado en run:            -> R5
  ok   neg  N11 pull_request_target con checkout del merge ref -> R1
  ok   neg  N12 pull_request con un secreto en env: del job    -> R1
  ok   neg  N13 repository_dispatch en la lista de eventos     -> R4
  ok   neg  N14 un job sin permissions junto a otro que si     -> R2
  ok   neg  N15 pull_request_target, checkout con repository: del head -> R1
  ok   neg  N16 pull_request_target, gh pr checkout con el numero por env: -> R8 (y R1)
  ok   neg  N17 titulo del PR en actions/github-script          -> R5 (en script:)
  ok   neg  N18 github['event']['pull_request']['title'] en run: -> R5
  ok   neg  N19 toJSON(github.event) en run:                    -> R5
  ok   neg  N20 workflow reutilizable con secrets: inherit       -> R9 (y R1)
```

**B2.** Búsqueda de la caché con tres reglas: la de F1-7 (nombre, App 15368,
verde y `arbol=` en el resumen), «solo job» (además, el check-run es el job
publicador del workflow de TaskCode; la primera versión de D5a) y la
decisión 7 tras la revisión 1 (además, el PR no toca `.github/workflows/**`
ni `.taskcode/**`):

```text
  ok   F1-7 acepta   solo-job acepta   decision-7 acepta   publicador legitimo (control positivo)
  ok   F1-7 acepta   solo-job rechaza  decision-7 rechaza  check-run creado por la API desde un script del PR con checks:write
  ok   F1-7 rechaza  solo-job rechaza  decision-7 rechaza  linea escrita en el resumen del job ejecutor
  ok   F1-7 acepta   solo-job rechaza  decision-7 rechaza  job verificacion de otro workflow anadido en el PR
  ok   F1-7 rechaza  solo-job rechaza  decision-7 rechaza  check verificacion de otra App
  ok   F1-7 rechaza  solo-job rechaza  decision-7 rechaza  publicador con otro arbol
  ok   F1-7 rechaza  solo-job rechaza  decision-7 rechaza  publicador en rojo
  ok   F1-7 acepta   solo-job acepta   decision-7 rechaza  publicador de un workflow modificado en el PR (toca .github/workflows/verificacion.yml)
  ok   F1-7 acepta   solo-job acepta   decision-7 rechaza  publicador con el PR tocando .taskcode/config.yml
  ok   F1-7 acepta   solo-job acepta   decision-7 acepta   publicador con el PR tocando solo codigo (control positivo)
```

Lectura: la regla de F1-7 aceptaría cuatro verdes falsos y la de «solo job»
dos: un publicador de un workflow modificado en el PR tiene los mismos
atributos que el legítimo (revisión 1, IMP-1). La decisión 7 no da acierto
en esos PR y los verifica enteros; sigue siendo parcial (un PR que no toca
esas rutas no cambia el publicador, pero la regla depende de leer bien el
diff). Es una simulación: cómo se identifica el job de un check-run en
la API real queda «no verificado».

### C — El propio ADR

Se leen la matriz (16 activos) y las preguntas heredadas (13) entre sus
marcadores. Cada activo tiene zona Z1–Z5, gravedad, mitigación y dueña que
existe como fila del plan o de la ampliación; los 16 activos y los 13 temas
heredados obligatorios están; F1-5, F1-6 y F1-7 aparecen como origen (citan
F1-8 en 5, 6 y 7 líneas); los requisitos de la cabecera existen en el
catálogo y en la auditoría; están las secciones de la plantilla y una
decisión por cada parte del enunciado; hay fila en el índice; no hay enlaces
a `tareas/`; `A7` y `A8` se declaran parciales y la tabla de la decisión 4
lleva la salvedad A4/A5. Catorce copias mutadas fallan por el error esperado
(14/14): dueña
vacía, dueña inexistente (`F5-99`), zona `Z9`, fila del runner propio
borrada, pregunta de `repository_dispatch` borrada, mitigación vacía,
requisito `RR-99`, enlace a `tareas/`, gravedad quitada, divergencia con
F1-7 quitada, decisión del runner propio quitada, marcador borrado, `A7` sin
«parcial» y salvedad A4/A5 quitada.

### D — Ajustes reales (solo `GET`)

```text
  TaskCodeTwo      public  default_workflow_permissions=read  can_approve=false  forks: first_time_contributors  runners=0
  ToolsaWeb        private default_workflow_permissions=read  can_approve=false  run_workflows_from_fork_pull_requests=false  runners=0
  OpenDockTrading  private default_workflow_permissions=read  can_approve=false  run_workflows_from_fork_pull_requests=false  runners=0
  los tres: allowed_actions=all, sha_pinning_required=false
  ToolsaWeb y OpenDockTrading: 0 ficheros en .github/ en develop y en main (git ls-tree local)
  gh auth status: Logged in to github.com account charliebk (keyring)
```

### Divergencias con el plan

1. **«Lo que ejecuta» se concreta** (D4b, PoC A2): solo los ficheros
   **versionados** que se ejecutan como script, de forma transitiva. La
   primera versión del guion seguía cualquier ruta existente y cogía 190
   ficheros en ToolsaWeb, entre ellos los ignorados de `.estado/` que cambian
   en cada verificación (la confianza se perdería en cada ejecución) y
   documentación; un `git pull` solo cambia lo versionado.
2. **Límite de D4b**: el código al que se llega por un glob o una
   herramienta (tests, `build.rs`, `package.json`) no entra salvo glob
   declarado (PoC A1, caso límite). Confiar en una orden de verificación es
   confiar en ejecutar la rama.
3. **Coste del hash**: de 0,4 a 0,9 s en frío con los pilotos, casi todo el
   presupuesto de 1 s de F5-6; F0-2 tiene que reutilizar el hash de los
   ficheros que no cambiaron (por tamaño y fecha), sin medir.
4. **T2.1 ampliada**: además de los permisos de los workflows se leyeron los
   ajustes de forks y los runners de los tres repos, y los ficheros de
   `.github/` de los pilotos (no tienen ninguno). TaskCodeTwo es **público**:
   aquí no cabe un runner propio.
5. **Riesgo que el plan no trae**: el token de la persona está en el llavero,
   al alcance de esta misma sesión del agente (PoC D; activo `A5`,
   divergencia 4 con F1-6).
6. **`taskctl doctor`** no lo ejecuta esta implementación (se le pidió no
   ejecutar `taskctl`); `comprobar-repo.sh` sí, en verde.
7. **Revisión 1** (decidido por Carlos con la consigna «la opción
   recomendada»): `A7` pasa a parcial y `buscar` no da acierto en un PR que
   toca workflows o `.taskcode/` (IMP-1, PoC B2); el lint se declara
   heurístico, detecta cinco variantes más y F5-1 lo formula como lista
   blanca (IMP-2, PoC B1); la decisión 4 lleva la salvedad A4/A5 y queda
   condicionada a resolver `A5` (IMP-3); el límite de la huella nombra las
   importaciones por nombre de módulo, la matriz admite riesgos aceptados
   nombrados, `A5` tiene un ADR decisor por crear y `carril`, una acción en
   el plan (MENOR-1 a 3).

## Cómo se ve en GitHub

Hoy (observado con `GET`, PoC D): los tres repos tienen los permisos por
defecto de los workflows en `read` y `can_approve_pull_request_reviews` en
`false`; ToolsaWeb y OpenDockTrading son privados, con los workflows de PR de
forks desactivados y ningún runner propio registrado; TaskCodeTwo es público
(aprobación de workflows de forks para quien contribuye por primera vez) y
sin runners propios. Ningún repo exige acciones fijadas por SHA. ToolsaWeb y
OpenDockTrading **no tienen ningún fichero en `.github/`** en `develop` ni en
`main`: no hay workflows suyos que reescribir.

Con este ADR aplicado (conocimiento general de GitHub, **no verificado**):

| Workflow | Clase de cada job | Evento | `permissions` | Secretos |
|---|---|---|---|---|
| `verificacion` | `buscar` (clase ejecuta por conservador: hace checkout para la huella del árbol; solo lectura), `ejecutar` (clase ejecuta), `verificacion` (publicador, sin checkout: escribe `arbol=` en su resumen) | `pull_request`, `merge_group` | `contents: read` en los tres | ninguno |
| `carril` | `carril` (privilegio; diff y reviews por la API) | `pull_request_target` (más el relanzado por review, parcial: D1b) | `contents: read`, `pull-requests: read` | ninguno |
| `revision-reproducir` | `reproducir` (clase ejecuta): tests, mutantes y bordes de F6-4, salida como artefacto | `pull_request` | `contents: read` | ninguno |
| `revision-ia` | `revision-ia` (privilegio): lee el diff y el artefacto como datos, llama al modelo, publica el check con la App del revisor | `workflow_run` de `revision-reproducir` | `actions: read` (artefacto), `contents: read` | entorno `revisor`: clave de la App y clave del modelo |

En Settings → Environments, el entorno `revisor` con sus dos secretos y las
ramas base como únicas ramas permitidas. Ningún workflow con
`repository_dispatch`.

## Decisión

1. **Dos clases de jobs** (sección «Dos clases de jobs»). Todo job de clase
   ejecuta lleva `permissions: contents: read` explícito y ningún secreto;
   todo job de clase privilegio lee el PR solo como datos. Además, en todos
   los workflows de TaskCode: bloque `permissions` explícito (nunca
   `write-all`), ningún `repository_dispatch` y ninguna interpolación de
   `github.event.*` ni `github.head_ref` en `run:` (se pasan por `env:`).
   `pull_request_review` cuenta como evento que ejecuta el workflow del PR
   (supuesto no verificado): en él no hay escrituras ni secretos. Lo
   comprueba un lint estático (el de la PoC B es el prototipo) en F5-1 y lo
   repite `doctor` (F3-8) sobre los workflows instalados, junto con los
   permisos por defecto de Actions en solo lectura. **El lint de la PoC es
   heurístico**: busca patrones prohibidos (checkout del head por `ref:` o
   por `repository:`, `gh pr checkout` o `git fetch … pull/` en `run:` fuera
   de un evento de PR, datos del evento en `run:` o en `actions/github-script`
   también con corchetes o `toJSON(github…)`, `secrets: inherit`, artefacto
   ejecutado) y deja pasar lo que no conoce. Por eso F5-1 lo formula como
   **lista blanca**: eventos admitidos por clase y usos de `actions/checkout`
   permitidos (sin `ref:` ni `repository:` en clase privilegio), y rechaza
   todo lo demás.
2. **`carril`** (D1b) corre en `pull_request_target`, sin checkout del head:
   lee el diff, las reviews y la tarea por la API. Es una mitigación
   **parcial**: F1-5 lo relanza con cada review y `pull_request_review`
   ejecuta el workflow del ref del PR (no verificado), así que un workflow
   cambiado en el propio PR por una persona puede seguir dando un `carril`
   en verde al relanzarse; lo contienen CODEOWNERS en `/.github/` y que la
   App del agente no tenga `workflows` (salvo A5: con el token de la persona
   sí podría). La migración borra el `carril` viejo en el
   mismo commit en que entra el nuevo (si conviven, hay dos check-runs
   `carril` de la misma integración y `filter=latest` toma el último). E2E
   en F2-6.
3. **Revisor IA partido en dos jobs** (D2b, completa D5a de F1-6). Job 1,
   `reproducir`: clase ejecuta, en `pull_request`, sin secretos; corre la
   reproducción, los mutantes y los bordes de F6-4 y deja su salida como
   artefacto. Job 2, `revision-ia`: clase privilegio, en `workflow_run`; con
   la clave de la App del revisor **y la clave del modelo** (secretos del
   entorno `revisor`), lee el diff por la API y el artefacto **como datos no
   fiables** (validados por esquema, nunca ejecutados), y publica el check.
   Consecuencia para RR-03: el LLM no ejecuta código; las reproducciones que
   quiera las pide como datos y las ejecuta el job 1 en la ronda siguiente
   (F6-4, F6-6). Que `workflow_run` reciba los artefactos del run de
   `pull_request` en un repo privado sin forks: no verificado (F2-6).
4. **Prompt injection: defensa por capacidades.** Todo texto de Z2 (tarea,
   plan, `## Resultado`, descripción del PR, diff, comentarios, salida de los
   tests) es dato. Lo que cada LLM puede hacer con él:

   | Quién lee texto hostil | Puede | No puede (y quién lo impide) |
   |---|---|---|
   | Agente en local (también en lote, RR-18) | ejecutar en la sesión de la persona; empujar ramas de tarea y `taskcode/tareas`; abrir PR | **salvo A4/A5**, mientras el token y la clave de la persona estén al alcance de su sesión, la plataforma no lo impide: aprobar (`carril` solo cuenta personas, F1-5); integrar sin la aprobación de la persona (rulesets sin bypass); firmar un `approve` (FIDO o frase, F1-6); tocar workflows (con su identidad no puede: la App no tiene `workflows`, F1-6; pero el token de la persona en esta máquina sí tiene el scope `workflow`, observado con `gh auth status` el 2026-11-14); publicar checks con su identidad (sin `checks:write`). Todo lo de esta celda queda condicionado a resolver A5 |
   | Revisor IA en CI | emitir hallazgos JSON | ejecutar nada (job 2 sin código del PR); aprobar ni mergear (App con solo `checks:write`, F1-6); poner el check en verde con prosa: el resultado lo calcula el job con reglas (CRÍTICO o IMPORTANTE abierto → rojo; hallazgo sin evidencia → rechazado, F6-4) |

   Un `revision-ia` en verde por inyección no integra nada: la puerta es la
   aprobación de la persona, salvo A4/A5 (un agente con el token o la clave
   de la persona al alcance aprueba como ella). **Estado de esta decisión**:
   garantía de la plataforma para el revisor IA en CI; para el agente en
   local (y en lote, RR-18), **condicionada a resolver `A5`** (y `A4` si la
   clave no es FIDO ni con frase sin caché). Una detección de patrones hostiles se puede
   añadir como señal (un hallazgo más), nunca como garantía. La batería de
   casos es de F6-7 y la prueba de un PR malicioso, config hostil e
   inyección en tareas, de F9-5. Lo escrito por la 0.16 se trata igual
   cuando el revisor lo relea.
5. **Confianza local única** (D4b). Una sola interfaz para F0-2
   (`taskctl confiar`), el hook de F5-6, `comando_sincronizacion` (RF-70) y
   cualquier `comando_*` que el CLI ejecute:
   `confianza(repo, comando) -> confiado | cambiado | nunca`, donde `comando` es
   la clave de la configuración y la orden se lee del árbol.
   - **Huella** = `sha256` de la orden y de **lo que ejecuta**: el cierre de
     los ficheros **versionados** que se ejecutan como script (`.bat`,
     `.cmd`, `.sh`, `.ps1`, `.py`, `.js`, `.ts`…) y que la orden nombra y, a
     su vez, los que nombran estos (`%~dp0` resuelto), más los globs que el
     proyecto declare en la configuración (F1-11). Lo ignorado o generado no
     entra (lo que un `git pull` cambia es lo versionado). Se hashea el
     contenido del árbol de trabajo, que es lo que se ejecuta.
   - **Almacén fuera del árbol** (en el perfil de la persona; ruta: F0-2),
     con clave el directorio común de Git: todos los worktrees de un repo
     (F1-3) comparten la confianza; otro clon, no. Un fichero del repo que
     diga «confiado» no cuenta.
   - **Estados**: `nunca` (sin entrada), `confiado` (misma huella),
     `cambiado` (otra huella: no se ejecuta y se vuelve a pedir). Volver al
     contenido confiado vuelve a dar `confiado` (por contenido, no por
     historia).
   - **Límite**: el código al que se llega a través de herramientas (tests,
     `build.rs`, scripts de `package.json`, dependencias) no entra en el
     cierre, ni lo que se carga por nombre de módulo y no por ruta (`import
     helper` en Python, `require('x')`, `source nombre` sin extensión): el
     cierre solo sigue tokens con forma de ruta; esos ficheros se declaran en
     `confianza_incluye`; hashearlo es hashear el árbol entero y pedir confianza en cada
     commit. Confiar en una orden de verificación es confiar en ejecutar el
     código de la rama: por eso el hook solo corre la rápida, opcional, y
     nunca al cambiar a una rama ajena sin pedirlo (F5-6). Coste medido en la
     PoC A2: de 0,4 a 0,9 s en frío con los pilotos; F0-2 reutiliza el hash
     de los ficheros sin cambios para caber en el presupuesto del hook.
6. **Runner propio** (D3b). Además de F1-7 (solo repos privados; falla
   cerrado): (a) corre con un **usuario de servicio** del sistema sin acceso
   al perfil de la persona (ni `ssh-agent`, ni llavero, ni `gh`, ni
   `%APPDATA%`); los datos locales como `exampleData`, en solo lectura para
   ese usuario; (b) **directorio de trabajo limpio en cada job** (runner
   efímero o borrado completo antes del job); (c) **ningún job de `main`
   (pesada de la release) en el mismo runner que los de tarea**: otra
   instancia con otro usuario y otra etiqueta, o runners efímeros. Si no se
   garantizan las tres, **no hay pesada en PR**: el runner propio no se
   registra para jobs de PR y la pesada sigue fuera de la CI, como hoy. Un
   runner que ejecutó código hostil con acceso al perfil de la persona deja
   la máquina comprometida (decisión 8). `doctor` comprueba las etiquetas y
   que el repo es privado (F3-8).
7. **Caché solo del publicador** (D5a, divergencia con F1-7). `verificacion`
   se parte en un job `ejecutar` (clase ejecuta) y el job publicador
   `verificacion`, que no ejecuta nada del PR, calcula él mismo el árbol del
   commit verificado (con `GET /commits/{sha}`) y escribe la línea
   `arbol=… perfil=… runner=… clave=… local=… origen=…` en su resumen.
   `buscar` solo acepta líneas del check-run del **job publicador del
   workflow de TaskCode** (por el job o el `run_id` del check-run, no solo
   por `app_id` y nombre): un check-run creado por la API, el de otro job o
   el de otro workflow no cuenta (PoC B2). Es una mitigación **parcial**,
   como `carril`: con `pull_request` el workflow del publicador sale del
   propio PR, así que un PR que lo modifique produce un publicador con el
   mismo nombre, workflow y `run_id` que el legítimo. Por eso `buscar` **no da
   acierto** (rechaza toda línea `arbol=`, también la del head anterior) si
   el diff del PR frente a su base toca `.github/workflows/**` o
   `.taskcode/**`, y ese PR se verifica entero (PoC B2). CODEOWNERS en esas
   rutas, con una sola persona (RN-06), es la misma aprobación que ya exige
   el PR y no lo cierra (F1-5, decisión 6). Alternativa para F5-1: mover el
   publicador a `workflow_run`, que lee el workflow de la base. El campo
   `local` lo calcula el
   job ejecutor en el runner del perfil y le llega al publicador como dato:
   falsearlo solo puede dar un acierto para el mismo árbol (riesgo aceptado).
8. **Rotación de secretos.** Ningún secreto filtrado tiene vuelta atrás: se
   rota. Procedimiento, con la identidad de la persona:

   | Secreto | Rotar | Después |
   |---|---|---|
   | Clave de la App del agente o del revisor | generar una clave nueva en la App, ponerla donde vive (almacén local o entorno `revisor`), **borrar la vieja** en la App | `doctor` comprueba que la App firma; listar los check-runs y PR de esa App desde la fecha de la fuga y relanzar los checks afectados |
   | Clave del modelo | revocarla en el proveedor y cambiar el secreto del entorno | relanzar `revision-ia` de los PR abiertos |
   | Clave de firma de la persona | quitarla de la cuenta, registrar la nueva | las aprobaciones vigentes se rehacen con `approve` y la clave nueva; que el «Verified» de los commits viejos cambie al quitar la clave: no verificado (F2-6) |
   | Token de la persona | revocarlo en la cuenta, volver a iniciar sesión | revisar en el registro de auditoría qué hizo el token |
   | Runner propio comprometido | quitar el runner del repo, reinstalar la máquina | rotar todo lo que la máquina tuviera al alcance (las filas de arriba) |

   Un fallo a medias de la rotación falla cerrado: sin clave válida
   `revision-ia` se queda pendiente y `doctor` dice la causa.

## Preguntas heredadas

Lo que F1-5, F1-6 y F1-7 dejaron a F1-8, cerrado aquí o delegado con dueña.
La PoC C lee esta tabla entre los marcadores `preguntas-heredadas`.

<!-- preguntas-heredadas:inicio -->
| ID | Pregunta heredada | Decisión | Origen |
|---|---|---|---|
| `P1` | Workflow modificado en el propio PR: `carril` y `verificacion` salen en verde con `pull_request` | `carril` en `pull_request_target`, parcial (decisión 2); `verificacion` con publicador, también parcial: sin acierto de caché si el PR toca `.github/workflows/**` o `.taskcode/**` (decisión 7); E2E en F2-6 | F1-5 decisión 6 y preguntas abiertas; F1-7 preguntas abiertas |
| `P2` | La App del revisor con `checks:write` podría publicar `verificacion` o `carril` | se mantiene la contención por `integration_id` 15368 de F1-5; `buscar` exige además el job publicador (decisión 7); no verificado → F2-6 | F1-5 preguntas abiertas |
| `P3` | Cómo se relanza `carril` al terminar `revision-ia` | el disparador que se use es de clase privilegio (workflow de la base, sin código del PR); cuál, lo fija la implementación de `carril` (F6) con E2E en F2-6 | F1-5 preguntas abiertas |
| `P4` | Clave de firma de la persona al alcance del agente en la misma máquina | FIDO o frase sin caché (F1-6), nunca en un agente de claves sin confirmación por uso; rotación (decisión 8); no verificado con llave real → F2-6 | F1-5 preguntas abiertas; F1-6 preguntas abiertas |
| `P5` | Dónde vive cada credencial y qué workflow puede leer qué secreto | matriz `A1`–`A5`; secretos solo en jobs de clase privilegio, en el entorno `revisor` (decisiones 1 y 3) | F1-6 decisión 5 y consecuencias |
| `P6` | Workflows que el agente puede lanzar con `repository_dispatch` | ningún workflow de TaskCode lo escucha ni interpola datos del evento en `run:`; lint y `doctor` (decisión 1) | F1-6 consecuencias |
| `P7` | Revisor en Actions (D5a): detalle de la clave y de lo que ejecuta | dos jobs, clave de la App y del modelo en el publicador (decisión 3) | F1-6 alternativa D5a |
| `P8` | Una tarea legítima que toca `.github/workflows/` | el agente nunca recibe `workflows`; la empuja la persona; con una sola persona, la salida frente a `require_last_push_approval` la decide F1-10 | F1-6 preguntas abiertas |
| `P9` | Merge por el agente con todo en verde | no lo cierra F1-8: F1-10 (y F1-6, límite D3a) | F1-6 preguntas abiertas |
| `P10` | Runner propio con código del PR | D3b (decisión 6) | F1-7 decisión 5, consecuencias y preguntas abiertas |
| `P11` | `pull_request` frente a `pull_request_target` | dos clases de jobs (decisión 1) | F1-7 consecuencias |
| `P12` | Quién puede publicar un `verificacion` con resumen falso (todo workflow con `checks:write`, `integration_id` 15368) | jobs ejecutores con `contents: read`, permisos por defecto en lectura, línea `arbol=` solo del publicador y nunca en un PR que toca workflows o `.taskcode/` (decisiones 1 y 7), parcial; no verificado → F2-6 | F1-7 consecuencias |
| `P13` | Confianza del hook local | confianza única con hash de la orden y de lo que ejecuta (decisión 5) | F1-7 decisión 4 y consecuencias |
<!-- preguntas-heredadas:fin -->

## Divergencias con F1-5, F1-6 y F1-7

Los ADR aceptados no se modifican; lo que este ADR cambia queda aquí y en
«Consecuencias»:

1. **F1-7, decisión 3** (D5a): F1-7 acepta como candidata cualquier línea de
   un check `verificacion` en verde de la App 15368. La línea `arbol=` pasa a
   contar **solo si la escribió el job publicador** del workflow de TaskCode, que calcula el árbol él
   mismo (decisión 7), y nunca hay acierto en un PR que toca
   `.github/workflows/**` o `.taskcode/**`; `verificacion` pasa de un job a
   dos (`ejecutar` y el publicador), además de `buscar`.
2. **F1-5, decisión 6**: F1-5 propone `pull_request_target` para `carril`
   como cierre del workflow modificado en el PR. Aquí es **mitigación
   parcial** (D1b): no cubre `pull_request_review`.
3. **F1-6, D5a**: la clave de la App del revisor no vive en «el entorno de
   Actions del revisor» en general, sino solo en el job publicador, que no
   ejecuta código del PR; y se le suma la clave del modelo, que F1-6 no
   menciona (decisión 3).
4. **F1-6, decisión 5**: «el token de la persona, solo usado con TTY» no
   impide que el agente, con el mismo usuario del sistema, lo saque del
   llavero (en esta misma sesión, `gh auth status` muestra el token de la
   persona guardado en el llavero: PoC D). Queda como riesgo abierto (`A5`).

## Requisitos y evidencia

| Requisito | Decisión | Evidencia |
|---|---|---|
| Criterio de F1-8: modelo de amenazas con mitigaciones | zonas Z1–Z5 y matriz `A1`–`A16` con mitigación y dueña | PoC C (cada fila con zona válida, mitigación y dueña que existe en el plan; mutaciones detectadas) |
| Enunciado de F1-8: qué corre en local y en CI | dos clases de jobs (decisión 1); confianza local (decisión 5) | PoC B (clasificación de ejemplos y de `ci.yml` real, con controles negativos), PoC A |
| Enunciado de F1-8: confianza | decisión 5 | PoC A (estados, cierre transitivo, almacén fuera del árbol, worktree y clon; coste en los pilotos) |
| Enunciado de F1-8: prompt injection | decisión 4 | tabla de capacidades; batería: F6-7; prueba: F9-5; no verificado |
| Enunciado de F1-8: runner propio | decisión 6 | PoC D (repos privados, 0 runners, workflows de forks desactivados); condiciones: F5-5, no verificado |
| H2 nada del repo se ejecuta sin confianza | decisión 5 | PoC A (orden, script nombrado y script transitivo cambiados → `cambiado`); implementación: F0-2, F5-6 |
| H9 el guardián local de merges se salta | no se defiende en local: integrar exige la plataforma (rulesets sin bypass, F1-5; `carril`, F1-6), salvo A4/A5 (el token de la persona al alcance del agente lo salta) | decisión 4 (tabla de capacidades), condicionada a resolver `A5` |
| RR-03 la revisión reproduce | reproducción en un job sin secretos; el LLM la lee como dato (decisión 3) | PoC B (`revision-reproducir` de clase ejecuta y `revision-ia` de clase privilegio, sin ejecutar el artefacto); `workflow_run` con artefactos: no verificado (F2-6) |
| RR-08 garantías de la plataforma | capacidades en vez de detección; permisos y eventos comprobables | PoC B (lint heurístico; F5-1, lista blanca) y D; para el agente local, condicionada a `A5` |
| RR-13 actualizar los proyectos | los pilotos no tienen `.github/`: `actualizar` instala los workflows nuevos, no reescribe ninguno | PoC D (0 ficheros en `.github/` en `develop` y `main` de los dos pilotos) |
| RR-15 GitHub y GitLab | las dos clases y la confianza local son neutrales | equivalente en GitLab (pipelines de merge request, variables protegidas, runners por etiqueta): no verificado (F1-13) |
| RR-18 lotes desatendidos | mismas capacidades; el lote no integra en `develop` sin la persona (`A16`) | diseño de F1-16; no verificado |
| RF-70 `comando_sincronizacion` con confianza | misma interfaz de confianza (decisión 5); en CI, clase ejecuta | PoC A (cierre de la orden de sincronización de ToolsaWeb) |
| RN-02 runner propio por perfil | D3b (decisión 6) | PoC D; implementación: F5-5 |
| RN-06 una persona con agentes en la misma máquina | FIDO o frase (`A4`); token de la persona, riesgo abierto (`A5`) | PoC D (`gh auth status`: token en el llavero); llave real: no verificado |

## Consecuencias

- **F0-2** implementa `confianza(repo, comando)` con la huella de la decisión
  5 y el almacén fuera del árbol; **F5-6** (hook) y el `comando_sincronizacion`
  de RF-70 la usan sin otra propia.
- **F1-11** declara los globs extra de la huella de confianza y trata los
  `comando_*` como entrada no confiable. Observado en la PoC A: con un
  lector YAML estándar, `"scripts\verificar.bat"` (entre comillas dobles, en
  ToolsaWeb) se lee con `\v` como tabulador vertical; el formato de la
  configuración tiene que fijar cómo se escriben las rutas de Windows.
- **F5-1** implementa las dos clases (jobs `buscar`, `ejecutar` y
  `verificacion`), el bloque `permissions` por job y el lint estático como
  **lista blanca de eventos y de usos de `actions/checkout`** por clase, no
  como lista de patrones prohibidos; valora mover el publicador de
  `verificacion` a `workflow_run`. **F5-2** escribe la línea `arbol=` en el
  publicador, y `buscar` comprueba el job publicador y no da acierto si el
  PR toca `.github/workflows/**` o `.taskcode/**` (decisión 7).
- **F5-5** solo registra el runner propio con las tres condiciones de D3b; si
  no, no hay pesada en PR.
- **F6-1** crea el entorno `revisor` con la clave de la App; **F6-3** parte el
  revisor en `revision-reproducir` y `revision-ia`, con la clave del modelo en
  el entorno; **F6-4** deja la reproducción como artefacto con esquema y las
  reproducciones pedidas por el LLM como datos para la ronda siguiente
  (F6-6); **F6-5** calcula el check desde los hallazgos estructurados; **F6-7**
  hace la batería de inyección.
- **Acciones en el plan** (las hace Carlos al revisar el plan, antes de
  empezar F5-1): (a) **crear una fila F6-x para `carril`** (F1-5 lo asigna a
  «F6» sin fila) con `pull_request_target` y la migración en un solo commit;
  hasta entonces su dueña en la matriz es F5-1; (b) **crear un ADR de Fase 1
  «aislamiento del agente frente a las credenciales de la persona»**, dueño
  de la decisión sobre `A5` (agente en otro usuario del sistema o en un
  contenedor, token sin guardar, confirmación fuera de banda).
- **F3-8** (`doctor`) compara los permisos por defecto de Actions,
  `can_approve_pull_request_reviews`, los workflows de forks en privados, las
  clases de los workflows instalados, la ausencia de `repository_dispatch` y
  las etiquetas del runner propio; avisa del tipo de clave de firma.
- **F2-6** prueba en E2E todo lo marcado «no verificado».
- **F1-10** decide la salida de una tarea que toca workflows con una sola
  persona y el merge por el agente con todo en verde.
- **F1-16** y **F4-7** mantienen que el lote no integre en `develop` sin el PR
  final de la persona.
- **F9-5** prueba un PR malicioso, una config hostil y la inyección en tareas.
- **Este repo no cambia** (RR-14): `ci.yml` ya cumple la clase ejecuta
  (`contents: read`, sin secretos; PoC B).

## Fuera de alcance

- Quién lanza una release, merge por el agente con todo en verde y la tarea
  que toca workflows con una sola persona: F1-10.
- Claves de configuración de la confianza y de la huella: F1-11.
- Equivalentes en GitLab: F1-13.
- Política del lote: F1-16.
- Implementación y E2E: F0-2, F5-1, F5-2, F5-5, F5-6, F6, F2-6, F9-5.

## Preguntas abiertas

- **`git clean -ffdx` desde código del PR** en una verificación local o en el
  hook: borra el worktree de `tareas/` y lo no commiteado (F1-1). Ningún rol
  propuso mitigación; riesgo aceptado y nombrado (`A15`). La confianza
  reduce cuándo corre código del repo, no lo que hace cuando corre.
- **Workflows de los pilotos**: hoy no tienen ninguno (PoC D). Si añaden
  alguno antes del relevo, `actualizar` (RR-13) tendrá que pasarlo por el
  lint de clases y quizá reescribirlo, no solo avisar.
- **RR-18**: en lote, el agente lee más texto hostil sin persona hasta el
  final; el diseño (F1-16) tiene que mantener que el lote no integra en
  `develop` sin el PR final ni salta las puertas de cada tarea.
- **Token de la persona en el llavero** (`A5`, divergencia 4): el agente
  corre con el mismo usuario del sistema y puede usarlo; las salidas
  posibles (agente en otro usuario o en un contenedor, token sin guardar,
  confirmación fuera de banda) las tiene que elegir F2-6 con F3-8. Ningún rol
  lo trató.
- **Acciones de terceros**: los repos no exigen fijarlas por SHA
  (`sha_pinning_required: false`, PoC D); `ci.yml` ya fija `actions/checkout`
  por SHA. Si se exige, es de F5-1 y F3-8.
- **Casos borde del cierre de confianza**: rutas construidas en tiempo de
  ejecución o por glob no entran (PoC A, control de límite).

## No verificado

- Que un agente que use el token de la persona (A5) no pueda empujar cambios en `.github/workflows/`: el token observado tiene el scope `workflow` (`gh auth status`, 2026-11-14, sin mostrar el token), así que lo puede hacer hasta que A5 se resuelva; no se ha probado empujando nada.

- **Completitud del lint de clases** (PoC B1): es heurístico. Detecta los
  20 negativos de la PoC, entre ellos las cinco variantes de la revisión 1
  (checkout con `repository:` del head, `gh pr checkout` con el número por
  `env:`, título en `actions/github-script`, `github['event']` y
  `toJSON(github.event)`, `secrets: inherit`), pero no prueba que un workflow
  sea seguro: acciones de terceros que hacen checkout del head, scripts que
  traen el head por otra vía o expresiones en otros campos de `with:` pasan.
  Por eso F5-1 exige lista blanca.
- Que `buscar` pueda leer el diff del PR frente a su base para decidir si
  toca `.github/workflows/**` o `.taskcode/**` en el evento de la búsqueda
  (decisión 7): F2-6.
- Que un `GITHUB_TOKEN` con `checks:write` cree check-runs con cualquier
  nombre (`carril`, `verificacion`) y que con `contents: read` no pueda:
  E2E en F2-6.
- Que `pull_request_review` ejecute el workflow del ref del PR: F2-6.
- Que `workflow_run` reciba los artefactos del run de `pull_request` en un
  repo privado sin forks: F2-6.
- Que el id del check-run de un job de Actions sirva para leer su job
  (`GET /actions/jobs/{id}`) y su workflow: F2-6 (decisión 7).
- Que el revisor necesite una clave del modelo en Actions (suposición del
  plan): F6-3.
- Que el entorno `revisor` limitado a las ramas base niegue sus secretos a
  un job de `pull_request`: F2-6.
- Que el «Verified» de un commit sobreviva a retirar la clave de firma: F2-6.
- Una llave FIDO real con `approve`: F2-6.
- Que F0-2 se aplique a la 0.16 y que F5-6 reutilice su mecanismo.
- Este repo (RR-14) no sirve de prueba: sigue con `ci.yml` y la cuenta de la
  persona hasta el relevo.

## Adenda (F6-3, TASK-049) — el publicador de `revision-ia`

- **Disparador**: sin F6-4, el revisor corre en `workflow_run` de
  `verificacion` (D3 de TASK-049). `workflow_run` sigue fuera de la lista
  blanca de los workflows reutilizables: lo usa solo el **llamador** del
  proyecto, que GitHub toma de la rama por defecto (un PR no lo cambia).
  `revision-ia.yml` es `workflow_call` y pasa el lint con el perfil
  `revision-ia` de `xtask/src/workflows.rs`.
- **A2 y los secretos del revisor**: `TC2_APP_REVISOR_ID`,
  `TC2_APP_REVISOR_CLAVE` y `ANTHROPIC_API_KEY` viven en el entorno
  `revisor` y solo los ve el job `publicar`, que no hace checkout ni ejecuta
  nada del PR (lee prompts ya hechos y llama a dos APIs). Un job con
  `environment:` es de clase privilegio aunque su GITHUB_TOKEN no escriba;
  el GITHUB_TOKEN del workflow no escribe nada (`actions: read`): el check
  lo publica la App del revisor con un token de instalación limitado a
  `checks: write`.
- **El job `preparar`** (clase ejecuta: `contents` y `actions` de lectura,
  sin secretos) saca la base, el plugin a un SHA fijado y `tc2` a la versión
  del workflow; el head del PR solo se trae como objetos (`git fetch` de su
  SHA) para el diff, nunca se saca ni se ejecuta. El lint lo impone: cada
  checkout lleva `ref:` sin `head` y `run:` no lleva `git checkout`,
  `switch`, `worktree`, `restore`, `reset` ni `inputs.`.
- **Lo que queda**: el diff y el informe de verificación los escribe el
  autor del PR y llegan al modelo; el prompt los marca como datos, no como
  instrucciones, pero la inyección de instrucciones es de F6-7. El check
  sale `neutral` (D4) y no bloquea hasta F6-5: lo contiene la aprobación
  obligatoria de la persona.
- **Excepción a R9 (`secrets: inherit`), decidida por la persona el
  2026-11-20**: el E2E real mostró que GitHub no da los secretos del
  entorno `revisor` al job `publicar` del reutilizable si el llamador no
  hereda: sin `secrets:` llegan vacíos, y también si el reutilizable los
  declara en `on.workflow_call.secrets` como opcionales (run 44735329636);
  con `secrets: inherit` llegan y la App publica el check (run
  44733982088). Se admite `inherit` **solo** en el llamador de
  `charliebk/TaskCodeTwo/.github/workflows/revision-ia.yml` fijado por SHA
  (perfil `llamador-revision` del lint, solo `workflow_run` y sin
  escritura); en cualquier otro workflow sigue siendo un error. Lo que se
  hereda llega a un workflow que fija el SHA y cuyo lint prohíbe secretos
  fuera del job del entorno `revisor`; la alternativa (secretos de
  repositorio pasados uno a uno) perdía la regla de rama del entorno.
  Dos límites, dichos: `inherit` entrega al reutilizable **todos** los
  secretos de repositorio y de organización del llamador, no solo los del
  entorno; y el lint es de tests (`cargo test` de este repo, sobre la guía y
  los workflows de TaskCodeTwo), así que en un proyecto consumidor lo que
  restringe el llamador es su revisión humana y el SHA fijado, hasta que
  `doctor` lo compruebe (F3-8).
- **Verificado en el E2E** (`charliebk/taskcodetwo-e2e-github`, PR 14,
  Rust + Vue): `preparar` sin secretos y sin ejecutar nada del PR elige
  `angular-vue-reviewer` y `rust-wasm-reviewer` (reglas S08, S12); el
  check `revision-ia` lo publica la App del revisor (5846307) con
  conclusión `neutral`.

## Adenda (F6-4, TASK-050) — hallazgos con evidencia

- **Decisión 3, revisada por la persona el 2026-11-20**: «F6-4 deja la
  reproducción como artefacto» pasa a ser **«F6-4 exige y valida la
  evidencia; F6-6 la ejecuta»**. El job `reproducir` (clase ejecuta, en
  `pull_request`) que corre reproducciones, mutantes y bordes no se crea en
  F6-4: sin la ronda siguiente (F6-6) no habría a quién darle su salida. Lo
  que el modelo declara se valida como dato con reglas deterministas en
  `tc2 hallazgos validar` (esquema `crates/tc2/esquemas/hallazgos.schema.json`,
  versión 1): todo hallazgo, también MENOR, trae evidencia de su tipo
  (`reproduccion`: pasos y fallo esperado, o un paso en rojo del informe
  F5-4; `mutante`: `linea_original` igual al texto de esa línea en el diff,
  la mutación y el test; `borde`: entrada, esperado y obtenido distintos)
  anclada a una línea del lado nuevo del diff que vio esa revisora. Lo que
  no cumple se rechaza con un motivo estable y el check lo lista (fila del
  revisor IA de la decisión 4: «hallazgo sin evidencia → rechazado»).
- **Cuatro jobs en `revision-ia.yml`**, cada uno en su clase y cada secreto
  en un solo job: `preparar` (ejecuta, sin secretos) deja en el artefacto el
  diff recortado de cada revisora; `consultar` (privilegio por el entorno
  `revisor`, solo `ANTHROPIC_API_KEY`, sin checkout) llama al modelo;
  `validar` (ejecuta, `contents` y `actions` de lectura, sin secretos)
  compila `tc2` del mismo SHA que `preparar` (no se pasan binarios entre
  jobs) y valida; `publicar` (privilegio, solo la App del revisor, sin
  checkout) lee el artefacto `revision-ia-hallazgos` como dato y publica.
  Ningún job con privilegio ejecuta un artefacto: la validación corre en
  clase ejecuta, sobre código de TaskCodeTwo fijado por SHA. A3 (la clave
  del modelo) ya no comparte job con la clave de la App (A2).
- **Transporte de `consultar` a `validar`**: un job con privilegio no sube
  artefactos (lint), así que las respuestas van por `outputs`, en base64, un
  hueco por revisora (hasta 6) con 64 KiB de texto cada una. En `validar`
  llegan por `env:` y solo las lee `tc2`; el test del workflow comprueba
  que ningún `run:` interpola `needs.*`.
- **Lo que queda**: la evidencia declarada no se ejecuta hasta F6-6 (lo
  contienen el anclaje al diff, la coincidencia de `linea_original`, la
  aprobación de la persona y F6-6); el check sigue `neutral` hasta F6-5; el
  informe en prosa del modelo va al check tal cual (las tablas de hallazgos
  sí escapan su Markdown) y la inyección de instrucciones es de F6-7.
- **No verificado**: que GitHub deje pasar las respuestas por `outputs` de
  un reutilizable sin enmascararlas ni truncarlas, y que la clave del modelo
  y la App funcionen en dos jobs distintos del mismo entorno `revisor` con la
  herencia actual (adenda F6-3): se comprueba en el E2E de TASK-050.

## Adenda (F5-5, TASK-045) — runner propio solo en la release

- **Decisión 6 (D3b), concretada** con las opciones que Carlos aceptó el
  2026-11-20 (D1, D3 y D4 del plan de TASK-045): el runner propio **solo se
  usa en la release**, un `pull_request` de `develop` a `main` de un repo
  privado con el head en el mismo repo. Lo comprueba el paso «Runner propio»
  de `buscar` (Linux, sin código del PR, datos del evento por `env:`), que
  falla cerrado con `exit 2` y su motivo antes de encolar nada; cuenta como
  runner propio un `runner-<perfil>` con la etiqueta `self-hosted` o que no
  es cadena ni lista.
- **D3b (a)**, usuario de servicio: el primer paso de `ejecutar`, antes del
  checkout y solo con `runner.environment == 'self-hosted'`, falla si el
  runner no es Windows, si el evento no es la release de un repo privado con
  el head en el mismo repo (la regla de `buscar`, repetida porque esta no
  depende de la etiqueta; revisión de TASK-045), si la cuenta es `SYSTEM` o si
  es del grupo Administradores. Que no lea el perfil de la persona y que los datos sean de
  solo lectura depende del alta manual
  ([guía](../ci/verificacion-reutilizable.md#runner-propio-f5-5)).
- **D3b (b)**, directorio limpio: `actions/checkout` con `clean: true` por
  defecto, y el lint da error si un checkout lleva `clean:` con otro valor.
- **D3b (c)**, la release aparte de los jobs de tarea: **se cumple por
  construcción**, porque en el runner propio solo corre la release; no hace
  falta otra instancia con otro usuario. Una instancia por máquina y repo.
- **Esto limita dos textos anteriores**, que se leen con esta adenda:
  [F1-7, decisión 5](F1-7-verificacion-ci-perfiles-cache-runners.md)
  admitía cualquier perfil en `[self-hosted, windows, <proyecto>]` en un repo
  privado; ahora solo la pesada, que es el único perfil de la release que no
  corre también en los PR de tarea. Y el ejemplo de ToolsaWeb de
  [F1-11](F1-11-extensiones-declaradas.md) ponía `completa: {runner:
  [self-hosted, windows, toolsaweb]}`: con esta regla, la completa de un PR de
  tarea fallaría en `buscar`; la completa va en un runner de GitHub y solo la
  pesada en el propio.
- **Lo que queda**: `doctor` no comprueba las etiquetas ni la visibilidad del
  repo (leer los runners pide `administration`; F3-8), y la guarda de
  `buscar` reconoce el runner propio por la etiqueta `self-hosted`: un
  `runner-<perfil>` sin ella (o un runner con `--no-default-labels`) llega a
  la máquina y lo para el aislamiento antes del checkout, sin ejecutar código
  del PR. Un llamador hostil puede escribir su propio `runs-on` fuera del
  reutilizable: eso no lo cubre ninguna guarda de este workflow (D1a). Código
  hostil ya integrado en `develop` corre en la release con el usuario de
  servicio (lo contienen D1a y el aislamiento). En GitLab no hay guarda
  equivalente: degradación declarada.
- **Verificado con un runner real** (E2E de TASK-045, 2026-11-21): la pesada
  de la release en el runner propio con el usuario de servicio y los datos de
  solo lectura; `github.event.repository.private` llega con valor al
  reutilizable.

## Adenda (F6-6, TASK-052) — rondas incrementales

- **`checks: read` en `preparar` y en el llamador**: `preparar` lee el check
  `revision-ia` de la ronda anterior con el `GITHUB_TOKEN` (`checks: read`,
  lectura ya admitida por la lista blanca de la decisión 1 para `buscar` de
  F5-2). El llamador tiene que darlo (un reutilizable no gana permisos que
  el llamador no da); sin él, el paso «Ronda anterior» falla y el check sale
  `incompleta` (falla en cerrado). El lint del llamador (`llamador-revision`)
  admite `checks: read` y rechaza `checks: write`. Ningún job nuevo.
- **La ronda anterior sale del check de la App del revisor**, filtrado por
  su `app.id` (`plataforma.revisor_id` de la base), no del informe
  `revision-ia-<sha12>.json` de la rama de estado, que escribe el agente: si
  el agente borrara un IMPORTANTE de su informe, la ronda siguiente lo
  seguiría viendo. Un check que no casa con sus anotaciones (título,
  conclusión y abiertos) para el cálculo de la ronda (falla en cerrado).
- **Cierre explícito**: un abierto CRITICO o IMPORTANTE solo se cierra con un
  cierre anclado al diff incremental (y, si traía `paso_rojo`, con ese paso
  ya en verde en el informe F5-4); el silencio del modelo no cierra nada. Los
  abiertos reinyectados en el prompt pasan por `plano`; la defensa contra la
  inyección sigue siendo de F6-7.
- **Decisión 3 (ejecución de la evidencia), revisada por la persona el
  2026-11-22**: la adenda F6-4 decía «F6-6 la ejecuta»; **no entra en
  F6-6**. Ejecutarla exigiría un comando de test por proyecto (clave nueva de
  F1-11), un job de clase ejecuta que corra código del PR con la mutación del
  modelo y saldo en la API para probarlo, y pasa de la fila F6-6 del plan.
  Queda como acción de Carlos crear una fila F6-x. Mientras, RR-03 se cumple
  en parte: se exige y valida la evidencia y en el cierre solo se contrasta
  `paso_rojo` con el informe F5-4.

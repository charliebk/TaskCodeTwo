# Plan — TaskCodeTwo: TaskCode 2, un plugin profesional para equipos

> Versión 3 del plan, 2026-11-13. Fuentes: la auditoría externa
> ([AUDITORIA-EXTERNA-2026-11-13.md](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)),
> la medición de la 0.16 ([EFECTO-016-2026-11-13.md](../auditoria/EFECTO-016-2026-11-13.md)),
> los requisitos con su uso real ([REQUISITOS-FUNCIONALES.md](REQUISITOS-FUNCIONALES.md))
> y una revisión de proyectos parecidos. Las decisiones marcadas como
> **[DECIDE CARLOS]** se toman antes de la fase que las necesita.

## 1. Diagnóstico y objetivo

TaskCode 0.16 tiene buenas ideas (diseño con varios puntos de vista, revisión
independiente, evidencia antes de cerrar, Git determinista, métricas), pero
**sus garantías son instrucciones al agente, no reglas que algo externo haga
cumplir**, y el estado de las tareas viaja dentro de las ramas, de modo que en
cuanto trabajan dos personas chocan los IDs, el tablero miente y los ficheros
generados dan conflictos.

**Objetivo:** TaskCode 2, en el repo nuevo **TaskCodeTwo**, que conserva lo que
funciona y se usa (requisitos con uso medido), cumple los veinte requisitos
rectores y lo hace **construyéndose con TaskCode 0.16**.

## 2. Requisitos rectores (por encima de todo)

Detalle en [REQUISITOS-FUNCIONALES.md §0](REQUISITOS-FUNCIONALES.md).

| ID | Requisito | Cómo lo cumple este plan |
|---|---|---|
| RR-01 | Git determinista para economizar tokens | P5: toda operación de Git/GitHub la ejecuta el CLI; el agente nunca escribe `git` ni `gh` |
| RR-02 | Varios puntos de vista | Brainstorm de roles con unificador y revisoras por dominio (F5, F6) |
| RR-03 | Búsqueda de errores | Revisión que reproduce, prueba mutantes y busca bordes (F6) |
| RR-04 | Revisión | Revisor independiente con identidad propia y aprobación de una persona antes del merge (P3, F6) |
| RR-05 | Iteración del método | Cifras por versión y auditorías, incluida una externa sin contexto (P9, F8) |
| RR-06 | Merge en `develop` | `develop` de integración, rama por tarea, `main` solo releases (P4) |
| RR-07 | Agilidad | Nada se verifica dos veces sobre el mismo árbol; carriles proporcionales (P6, P7) |
| RR-08 | Calidad | Puertas impuestas por GitHub y la CI (P3) |
| RR-09 | Trabajo en grupo | Estado sin conflictos, IDs únicos, toma de tareas visible (P1, P2) |
| RR-10 | Tareas para el programador, roadmap para todos | Una carpeta fija por tarea, estado como campo, sin carpetas por estado (P1) |
| RR-11 | Roadmap sin interferir en la verificación | El roadmap vive con las tareas, fuera de las ramas de código (P1) |
| RR-12 | Equipo garantizado: nadie pisa nombres, tareas ni trabajo | IDs y ramas únicos, títulos duplicados avisados, identidad de plataforma, liberar y reasignar, aviso de solape, estado siempre fresco (P2, F2-4b, F2-13, F3-10, F3-11, F3-5b) |
| RR-13 | Actualizar los proyectos existentes | `taskctl actualizar` con simulación y vuelta atrás (F9) |
| RR-14 | Hecho con la versión actual de TaskCode | TaskCodeTwo se desarrolla con TaskCode 0.16 (§7) |
| RR-15 | GitHub y GitLab desde la 2.0 | Adaptador de plataforma con las dos implementaciones y la misma batería E2E (P11) |
| RR-16 | Instalación fácil en cualquier ordenador | Repo público que es su propio marketplace de Claude Code, como TaskCode; ejecutable e instalador de una línea como complemento (P12); desinstalación limpia (F2-14, F9-10) |
| RR-17 | Skills, agentes y modelo adecuados a cada tarea | Selección determinista entre lo instalado, anotada en la tarea y medida (F1-15, F3-9, F4-6, F8-6) |
| RR-18 | Lotes desatendidos con revisión humana al final | `taskctl lote`: rama de lote, sin preguntas, un único PR final (P13; F1-16, F4-7, F4-8, F9-4b) |
| RR-19 | Progreso en vivo: fases que faltan y tiempo estimado | Estimación por fase, status line y `status --seguir`, métricas finales (F1-17, F3-12, F3-13, F4-9, F8-7) |
| RR-20 | Memoria del proyecto propia, compartida y nunca desactualizada — **descartado el 2026-11-16** | Git como verdad e índice SQLite reconstruible con frescura por SHA, etiquetado acotado, búsqueda única `tc2 buscar`, conversores de tareas y de engram, visor 3D opcional (F1-19, Fase 10) |

## 3. Qué existe y qué aprendemos

| Proyecto | Qué hace bien | Lo que no hay que copiar |
|---|---|---|
| [GitHub Spec Kit](https://github.github.com/spec-kit/) | Specify → Plan → Tasks → Implement con un artefacto por fase; extensiones; Windows/macOS/Linux | Puertas como prompts; artefactos que se desincronizan del código |
| [CCPM](https://github.com/automazeio/ccpm) | Estado compartido visible por todo el equipo (GitHub Issues); agentes en paralelo con worktrees | Sin puertas reales; depende de que el agente obedezca |
| [Kiro specs](https://kiro.dev/docs/specs/index) | Requisitos (EARS), diseño y tareas con aprobación entre fases | Aprobación local, ligada a su IDE |
| [obra/superpowers](https://github.com/obra/superpowers) | Método forzado brainstorm → plan → TDD → revisión | «Obligatorio» = instrucción; el ciclo entero también para una errata |
| [BMAD Method](https://github.com/bmad-code-org/BMAD-METHOD) | Encaja con sprints de equipos de 5–20 personas | Roles que son prompts; las suposiciones se propagan entre ellos |
| [Task Master](https://github.com/eyaltoledano/claude-task-master) | Grafo de tareas con dependencias | Estado local que el equipo no comparte |
| Protección de ramas y checks obligatorios de GitHub | Lo único que el agente no puede saltarse | Exige identidades distintas para quien implementa y quien aprueba |

Coinciden en nueve convenciones ([vanja.io](https://vanja.io/265000-stars/)),
que TaskCode ya tiene casi todas. Ninguno las hace cumplir en equipo: esa es la
aportación de TaskCodeTwo.

## 4. Principios de diseño

### P1 — Una carpeta fija por tarea para el programador, el roadmap para todos, todo en su propia rama de estado (RR-10, RR-11, RR-12)

- Cada tarea es una carpeta fija, `tareas/TASK-NNN/` (`tarea.md`, plan,
  informes de revisión, resultado), que **no se mueve nunca**: el estado es un
  campo del frontmatter. Sin carpetas por estado, sin renombrados.
- **La vista para todos es el `ROADMAP.md`**: tareas en curso (con quién las
  lleva), en revisión, planificadas por hito con sus dependencias, terminadas
  y descartadas, con su grafo en Mermaid. Se lee en GitHub, en GitLab o en el
  editor.
- **Nada de esto viaja en las ramas de código.** Vive en una rama propia,
  `taskcodetwo/tareas`, que el CLI monta como worktree en la carpeta `tareas/` del
  proyecto (ignorada en las ramas de código).
- Consecuencias:
  - **Ninguna rama de código lleva cambios de tareas**: cero conflictos de
    ROADMAP o `tarea.md` al integrar (H8), y ningún revert de código puede
    borrar una tarea (TASK-172).
  - **El estado real está siempre a la vista** en una sola rama, para todo el
    equipo (H7).
  - **El roadmap no interfiere con la verificación**: no está en el árbol que
    se verifica, así que ni invalida la caché ni dispara la CI (RR-11).
- Alternativa descartada: el estado en `develop` con commits atómicos del CLI.
  Obliga a permitir pushes directos a `develop` y mezcla historial de proceso
  con el de código. **[DECIDE CARLOS]** confirmar en el ADR F1-1.

### P2 — Escrituras de estado atómicas: IDs únicos y nadie pisa a nadie (RR-09, RR-12)

- Cada cambio de estado es un commit pequeño del CLI en `taskcodetwo/tareas` que se
  publica con push inmediato. Si el push se rechaza porque otro escribió
  antes, el CLI rehace el cambio sobre lo nuevo y reintenta; el roadmap se
  regenera tras el rebase, nunca se mergea.
- **El ID se reserva con ese push**: si dos personas crean una tarea a la vez,
  la segunda recibe el siguiente número (H3).
- **Tomar una tarea** (`start`) escribe quién la lleva antes de crear la rama:
  si otro la tomó, se avisa y no se crea.
- Cada operación toca solo la carpeta de su tarea y el roadmap.

### P3 — Las garantías las impone GitHub, no el agente (RR-04, RR-08)

- Código solo por PR contra `develop` con checks obligatorios
  (verificación, revisión IA) y la aprobación de una persona.
- El agente implementador y el revisor IA tienen **identidad propia**
  (GitHub App o cuenta de máquina), distinta de la persona: así la persona puede
  aprobar los PR del agente (GitHub no deja aprobar el PR propio) y queda claro
  quién decidió qué (H5).
- Aprobar el plan lo hace la persona con su identidad (`taskctl approve` firma
  con su usuario de GitHub); el agente no puede, porque no tiene esa identidad.
- Sin TTY ni identidad de persona, los atajos (`--sin-verificacion`, aceptar
  sin revisar) no existen (H6).

### P4 — `develop` de integración, rama por tarea, `main` para releases (RR-06)

- Rama corta por tarea desde `develop` en su worktree; PR contra `develop`;
  merge por la cola de merge o por `taskctl merge` cuando pasan las puertas.
- `main` recibe solo releases (`taskctl release`) y hotfix (`taskctl hotfix`).

### P5 — Git y GitHub deterministas desde el CLI (RR-01)

- Toda operación (rama, worktree, commit de lo que escribe la herramienta,
  push, PR, merge, tag, rama de estado) la hace un comando del CLI:
  idempotente, con reintento y con una salida corta, sin colores y con códigos
  de salida que dicen qué hacer.
- El agente no compone `git` ni `gh`. Se mide en tokens por tarea.

### P6 — Verificación en CI una vez por árbol (RR-07)

- Workflow reutilizable que llama al comando del proyecto con perfiles
  (rápida, impacto, completa, pesada). Caché por SHA del árbol: el PR, la cola
  de merge y la release comparten el resultado.
- El revisor IA lee el informe de la CI y no repite tests.
- Runner propio (self-hosted, Windows) para suites pesadas en repos privados.
- En local, solo un hook rápido y opcional; nada del repo se ejecuta sin
  confianza explícita (H2).

### P7 — Carriles proporcionales

- Errata → PR directo. Fix → tarea + PR. Funcionalidad → plan aprobado + PR.
  Riesgo alto → diseño con todos los roles y dos aprobaciones.
- El modo automático (88–97 % del uso real) encadena las fases y para solo en
  las puertas del carril.

### P8 — Superficie mínima y extensible

- Pocos comandos para el día a día; lo avanzado, en extensiones declaradas
  (estados propios, derivados, verificación propia, revisoras propias) en vez
  de convenciones a mano.

### P9 — El método itera con datos (RR-05)

- Cada versión publica sus cifras por tarea frente a la anterior: tokens,
  tiempo por carril, rondas, hallazgos y verificaciones repetidas.
- Auditoría interna por versión y externa sin contexto por versión mayor.
  Ningún cambio del método sin la cifra que lo motiva.

### P11 — GitHub y GitLab con las mismas garantías (RR-15)

- Un adaptador de plataforma con dos implementaciones desde la 2.0: rama
  protegida, pipeline obligatoria, aprobación de una persona, identidad propia
  del agente y del revisor, cola o tren de merge, en GitHub y en GitLab.
- La verificación se publica como workflow de GitHub Actions y como plantilla
  de GitLab CI que llaman al mismo comando del proyecto.
- Algunas funciones de GitLab dependen del plan de pago (por ejemplo, reglas
  de aprobación obligatorias o trenes de merge): el ADR F1-13 lo verifica, y
  `doctor` dice qué garantía no está disponible y qué la sustituye.

### P12 — Se instala en cualquier ordenador en un minuto (RR-16)

- **Igual que TaskCode**: TaskCodeTwo es un repo **público** que hace de
  marketplace de Claude Code. Se instala con `/plugin marketplace add
  charliebk/TaskCodeTwo` e instalando el plugin; el plugin trae el CLI ya
  compilado (sin `npm install` ni build) y lo pone en el PATH, como hoy.
- Lo que sigue son complementos para quien quiera el CLI fuera de Claude
  Code (CI, otros agentes), no el camino principal:

- **Un ejecutable por sistema** (Windows x64/ARM64, macOS Intel/Apple, Linux
  x64/ARM64), con el runtime dentro: no hace falta instalar Node. Como el CLI
  hace él mismo las operaciones de Git (P5), **tampoco hace falta bash**: la
  única dependencia externa es `git`.
- **Por internet con una línea**: `irm …/install.ps1 | iex` en Windows y
  `curl -fsSL …/install.sh | sh` en macOS y Linux, que descargan el ejecutable
  de la release, comprueban su firma o su hash y lo dejan en el PATH. Además,
  paquetes en winget, Homebrew y npm cuando la 2.0 sea estable.
- **Las skills y los agentes** se instalan por el marketplace de Claude Code,
  como hoy; el plugin comprueba que el ejecutable instalado es compatible.
- **Actualizar** es un comando (`taskctl actualizar --cli`) que avisa si hay
  versión nueva; actualizar un proyecto sigue siendo `taskctl actualizar`
  (RR-13).
- Decidido (Carlos, 2026-11-13): **repo público**, así que el marketplace y
  las releases se descargan sin credenciales.

### P13 — Lotes desatendidos con revisión humana al final (RR-18)

Hoy se hace en ToolsaWeb, OpenDockTrading y TaskCode como instrucción al
agente: «arrancar consecutivamente TASK-xxx a TASK-yyy, sin trocear y
aceptando las recomendaciones en todas las preguntas». Sirve cuando una lista
de tareas es de documentación o de programación que no necesita revisión
humana hasta el final, o para dejar una batería de trabajo por la noche.
Diseño propuesto, a confirmar en la ADR F1-16:

- **Rama de lote**: el lote trabaja sobre `lote/<nombre>` creada desde
  `develop`. Cada tarea va en su rama, pasa sus puertas automáticas
  (verificación en CI y revisión IA sin bloqueantes) y **se integra en la rama
  del lote**, no en `develop`, así que la siguiente tarea ya ve el código de la
  anterior. Al final, **un único PR del lote contra `develop`** que la persona
  revisa y aprueba: la puerta humana de siempre, una sola vez.
- **Política**: solo los carriles permitidos para lote (por ejemplo,
  documentación y programación sin riesgo alto); una tarea de riesgo alto o
  con un `[DECIDE CARLOS]` abierto no entra, o para el lote.
- **Ante un fallo**: un CRÍTICO o IMPORTANTE que no se resuelve en el tope de
  rondas, o una verificación en rojo, aparta esa tarea (y las que dependen de
  ella) y el lote sigue con las independientes; nunca integra algo en rojo.
- **Presupuesto** de tiempo y de tokens para todo el lote; al agotarse, para
  limpio.
- **Reanudable**: si se corta (apagado, error), `taskctl lote --reanudar`
  sigue donde iba sin repetir lo hecho.
- **Informe de la mañana**: qué se terminó, qué se apartó y por qué, hallazgos
  sin corregir, tokens y tiempo; enlazado desde el PR del lote.
- **Nada de esto lo decide el agente**: el lote, su política y sus paradas los
  ejecuta el CLI de forma determinista (P5).

### P10 — El plugin es un producto

- CI en Linux, macOS y Windows sin «rojos conocidos»; tests E2E contra un repo
  de GitHub de pruebas; versión semántica; mensajes en español e inglés.

## 5. Qué se conserva de la 0.16, qué cambia y qué se retira

| Conserva (con uso medido) | Cambia | Retira (sin uso) o pasa a extensión |
|---|---|---|
| `tarea.md` con frontmatter, `## Resultado`, informes de revisión | Carpeta fija por tarea (sin carpetas por estado), en una rama propia, con escrituras atómicas; el roadmap es la vista | `codex-review`, `verificar-lote`, `cadena` (retiran) |
| Import en bloque, dependencias, hitos, ROADMAP | El ROADMAP se regenera en la rama de estado, nunca en la de código | `doctor-deploy`, held-out, `trocear`, documentos con tope (extensiones) |
| Brainstorm de roles + unificador; revisoras por dominio | La revisión se publica en el PR con identidad propia y check obligatorio | Scripts bash de Git-Flow (su función pasa al CLI) |
| Modos manual/semi/automático | Las puertas humanas, por carril y verificables | `pause`/`resume` con stash (sobran con worktrees) |
| Perfiles de verificación y memo por huella | La verificación pasa a la CI, por SHA, compartida | Merge local de `finish` (pasa a PR) |
| Métricas por tarea, versión y repo; coste por fase | Se añaden minutos de CI repetidos y tokens por operación | — |
| Git determinista y commits solo de lo que escribe la herramienta | Mismo principio, para Git y GitHub, con salida para el agente | — |

## 6. Cómo trabajará un equipo con TaskCodeTwo

1. `taskctl import plan-h15.md` → reserva los IDs y crea las tareas del hito en la
   rama de estado (una carpeta por tarea, estado «planificada»), con dependencias. Todo el
   equipo las ve en el roadmap.
2. `taskctl next` → qué está listo para empezar según dependencias y prioridad.
3. `taskctl plan TASK-312` → brainstorm de roles proporcional al carril; el
   plan queda en la carpeta de la tarea. Una persona lo aprueba con
   `taskctl approve` (su identidad; aprobación de un hito entero de una vez).
4. `taskctl start TASK-312` → la tarea queda tomada (visible al momento), rama y
   worktree desde `develop`.
5. El agente implementa; `taskctl review` abre el PR con la identidad del
   agente. La CI verifica una vez por árbol y el revisor IA publica su review y
   su check.
6. Correcciones → la CI corre solo lo afectado; el revisor mira solo lo que
   cambió y si se cerraron los hallazgos.
7. `taskctl finish` → con las puertas en verde y la aprobación de una persona,
   merge en `develop` por la cola; la tarea pasa a «terminada» en la rama de
   estado con su `## Resultado`, y el roadmap lo refleja.
8. El `ROADMAP.md` de la rama de estado es lo que mira el equipo; `taskctl
   status` y `metricas` leen la rama de estado y la plataforma (GitHub o
   GitLab) y enseñan lo mismo a todos.

## 7. Cómo se construye: TaskCodeTwo con TaskCode 0.16 (RR-14)

- **Repo nuevo `TaskCodeTwo`** en GitHub, con `develop` y `main`, y TaskCode
  0.16 instalado y configurado (`tareas/`, `.taskcode/config.yml`, verificación).
  Repo **público**, decidido por Carlos el 2026-11-13; licencia MIT como TaskCode.
- Este plan se convierte en tareas de TaskCode 0.16 con `taskctl import`
  (fichero de import fuera del repo), un sprint por fase, con sus dependencias
  y criterios. Se trabajan con el flujo de siempre: plan, aprobación, rama,
  revisión por pares, finish.
- **Punto de relevo:** cuando la Fase 4 esté terminada (la 2.0 puede crear,
  planificar, tomar, revisar e integrar tareas), TaskCodeTwo se actualiza a sí
  mismo con `taskctl actualizar` y sigue su desarrollo con la 2.0. Es la primera
  prueba de la migración.
- La 0.16.2 (Fase 0) se hace en el repo TaskCode, donde vive la 0.16.

## 8. Plan por fases y tareas

Cada tarea cita los requisitos que cumple y tiene un criterio de aceptación
comprobable. Ninguna pasa de 1–2 h de trabajo; si pasa, se trocea. «Dep.» =
dependencias. Las fases 1 a 9 son tareas de TaskCodeTwo; la 0, del repo TaskCode.

### Fase 0 — Contención en la 0.16.2 (repo TaskCode, urgente)

| ID | Tarea | Criterio de aceptación | Dep. |
|---|---|---|---|
| F0-1 | `veredicto` y `finish` rechazan un informe que sigue siendo la plantilla o no nombra revisor (H1) | Repro de la auditoría → `finish` aborta; test E2E | — |
| F0-2 | Confianza explícita antes de ejecutar cualquier `comando_*` de la config: hash fuera del repo; si cambia, no se ejecuta (H2) | Repro `echo PWNED` → no se ejecuta; `taskctl confiar` lo habilita | — |
| F0-3 | `resume` aplica el stash de su rama (`stash pop "$stash_ref"`, grep anclado) (H4) | Test con dos stashes de ramas distintas | — |
| F0-4 | `decidido_por` = `agente` sin TTY; `persona` exige TTY o confirmación de la persona (H5) | Test con stdin no interactivo | — |
| F0-5 | Los atajos de `finish` exigen la misma confirmación; un hook los bloquea si vienen del agente (H6) | El agente no puede pasarlos; test | F0-4 |
| F0-6 | Frontmatter con escape YAML correcto y test contra un parser estándar (H11) | Título `C:\ruta\` ida y vuelta | — |
| F0-7 | Cerrojo en `verificar-impacto` | Dos ejecuciones a la vez: la segunda espera o avisa | — |
| F0-8 | Publicar 0.16.2 y actualizar ToolsaWeb y OpenDockTrading | Release y actualización en los dos proyectos | F0-1..7 |

### Fase 1 — Arranque de TaskCodeTwo y decisiones (ADR)

| ID | Tarea | Criterio de aceptación | Dep. |
|---|---|---|---|
| F1-0 | Crear TaskCodeTwo: repo en GitHub, `develop` y `main`, TaskCode 0.16 instalado, `tareas/`, config, CI mínima; importar este plan con `taskctl import` (RR-14) | `taskctl doctor` en verde; las tareas F1..F9 planificadas con dependencias | — |
| F1-1 | ADR: carpeta fija por tarea con estado como campo, en rama propia `taskcodetwo/tareas` montada como worktree en `tareas/` (P1), frente a estado en `develop`; contenido del ROADMAP | Decisión con pruebas de concepto de las dos; cómo se ve en GitHub | F1-0 |
| F1-2 | ADR: escrituras atómicas, reserva de IDs y toma de tareas (P2) | Protocolo de push con reintento; qué pasa sin red | F1-1 |
| F1-3 | ADR: flujo de ramas `develop`/`main`, rama por tarea, worktrees, release y hotfix (P4) | Nombres, ciclo completo, qué queda de Git-Flow | F1-0 |
| F1-4 | ADR: capa de operaciones Git/GitHub deterministas y su contrato de salida para el agente (P5) | Lista cerrada de operaciones; objetivo de tokens por tarea | F1-3 |
| F1-5 | ADR: puertas en GitHub (rulesets, checks obligatorios, CODEOWNERS) por carril (P3, P7) | Lista exacta de checks y quién aprueba cada carril | F1-3 |
| F1-6 | ADR: identidades del agente implementador, del revisor IA y de la persona (P3) | Permisos mínimos; que ninguna IA pueda aprobar ni mergear | F1-5 |
| F1-7 | ADR: verificación en CI, perfiles, caché por SHA y runners (GitHub y self-hosted Windows) (P6) | Coste estimado con las suites de ToolsaWeb y OpenDockTrading | F1-5 |
| F1-8 | ADR: seguridad (qué corre en local y en CI, confianza, prompt injection, runner propio) | Modelo de amenazas con mitigaciones | F1-6, F1-7 |
| F1-9 | ADR: formato de tarea, plan y especificación (compatible con la 0.16 y con Spec Kit) | Plantillas; qué campos se conservan | F1-1 |
| F1-10 | ADR: carriles y modo automático (qué para y dónde) (P7) | Política por carril, configurable | F1-5 |
| F1-11 | ADR: extensiones declaradas (P8) | Contrato que cubre lo que hoy hacen ToolsaWeb y OpenDockTrading a mano | F1-1, F1-7 |
| F1-12 | ADR: actualización de proyectos entre versiones (RR-13) | Qué migra, simulación, vuelta atrás, convivencia 0.16/2.0 | F1-1, F1-9 |
| F1-14 | ADR: distribución e instalación (RR-16): ejecutable por sistema (empaquetador, tamaño, arranque), firma de código en Windows y macOS, instaladores de una línea, canales (releases, winget, Homebrew, npm, marketplace), compatibilidad entre ejecutable y plugin | Prueba de concepto de un ejecutable en los tres sistemas; decisión sobre firma y canales | F1-0 |
| F1-13 | ADR: adaptador de plataforma GitHub y GitLab (RR-15): equivalencias (rulesets y ramas protegidas, checks y pipelines, reviews y approvals, cola y tren de merge, Apps y tokens de proyecto) y qué depende del plan de pago | Tabla de equivalencias verificada en las dos; cómo se prueba cada adaptador | F1-4 |
| F1-15 | ADR: selección de skills, agentes y modelo por tarea y por rol (RR-17): fuentes (catálogo del proyecto, skills y agentes instalados, revisoras del plugin), reglas deterministas por etiquetas, rutas, dominio, complejidad y carril, qué se anota en la tarea y cómo se mide el acierto | Reglas probadas a mano sobre 10 tareas reales de ToolsaWeb y OpenDockTrading; nada lo decide el modelo «a ojo» | F1-9, F1-10 |
| F1-16 | ADR: lotes desatendidos (RR-18): rama de lote, política de carriles, qué para el lote y qué aparta una tarea, presupuesto, reanudación, informe y su equivalente en GitLab | Diseño probado a mano con un lote real de ToolsaWeb | F1-5, F1-10 |
| F1-17 | ADR: progreso en vivo y estimación (RR-19): estimación por fase (carril, complejidad, tamaño y tareas parecidas), cuándo se recalcula, dónde se muestra y cómo sin tokens | Estimación probada contra las tareas terminadas de ToolsaWeb y OpenDockTrading, con su error medio | F1-9, F1-10 |
| F1-19 | ADR: memoria del proyecto (RR-20): git como verdad e índice SQLite reconstruible, ubicación del índice, regla de frescura por SHA, esquema, vocabulario y tres capas de etiquetado, conversor de engram (decisión A/B), retirada de engram del manifiesto | PoC: índice de las tareas terminadas de TaskCodeTwo reconstruido dos veces con el mismo hash; un merge hecho en la plataforma se refleja en la siguiente búsqueda sin hooks | F1-1, F1-9, F1-11, F1-14 |

### Fase 2 — Núcleo

| ID | Tarea | Criterio de aceptación | Dep. |
|---|---|---|---|
| F2-1 | Esqueleto: Rust (`Cargo.lock` versionado, sin binarios en el repo, `/Brepro` en Windows), `cargo fmt` y `clippy`, CI en Linux/macOS/Windows (F1-14, divergencia 1) | CI verde en los tres desde el primer commit y dos builds en limpio con el mismo sha256 en Windows (Linux y macOS sin medir, decision de la persona) | F1-* |
| F2-2 | Capa de operaciones Git deterministas: rama, worktree, commit de lo escrito, push con reintento, tag (P5) | Tests contra repos reales; salida ≤ 5 líneas en el caso normal | F1-4, F2-1 |
| F2-3 | Rama de estado: crear, montar como worktree en `tareas/`, leer y escribir de forma atómica (P1, P2) | Dos procesos escriben a la vez sin perder nada; test | F1-1, F1-2, F2-2 |
| F2-4 | Reserva de IDs por push atómico | Dos clones crean tarea a la vez → IDs distintos; test E2E | F2-3 |
| F2-4b | Nombres únicos: rama derivada del ID comprobada también contra el remoto; título duplicado o casi igual a otro abierto → aviso con la tarea parecida (RR-12) | Dos altas con el mismo título a la vez: IDs y ramas distintos y aviso de duplicado | F2-4 |
| F2-5 | Modelo de tarea y parser/serializador de frontmatter YAML válido | Ida y vuelta con un parser estándar; compatible con tareas de la 0.16 | F1-9 |
| F2-6 | Adaptador GitHub (PR, checks, reviews, identidad) sobre la API, con errores dirigidos a la persona | E2E contra un repo de pruebas en GitHub | F1-13, F2-2 |
| F2-9 | Adaptador GitLab (MR, pipelines, approvals, identidad) con la misma interfaz y la misma batería E2E (RR-15) | E2E contra un proyecto de pruebas en GitLab | F1-13, F2-2 |
| F2-7 | Configuración mínima con esquema validado y valores por defecto | ≤ 12 claves (13 desde TASK-081: `dias_sin_actividad`, adenda de F1-11); esquema publicado; tests | F1-10 |
| F2-8 | i18n de mensajes (es, en) | Test que falla si falta una traducción | F2-1 |
| F2-10 | Build de ejecutables por sistema en la CI, con su hash, publicados en cada release (RR-16); firmados desde la primera release con attestation de procedencia y firma ad hoc en macOS, Authenticode y Developer ID al entrar en winget y Homebrew (ADR F1-14, D4a) | El ejecutable arranca en Windows, macOS y Linux limpios sin Node | F1-14, F2-1 |
| F2-11 | Instaladores de una línea (PowerShell y shell), publicados en cada release, que descargan, comprueban el hash y, si hay `gh`, la attestation, y ponen `tc2` en el PATH (ADR F1-14, divergencias 3 y 5) | Instalación en menos de un minuto en cada sistema, medida en la CI en los 6 runners nativos y en alpine; las máquinas limpias de verdad, en F9-9a | F2-10 |
| F2-12 | `tc2 actualizar --cli` y aviso de versión nueva; el plugin del marketplace comprueba la versión del ejecutable | Actualización sin reinstalar; incompatibilidad avisada con el comando que la arregla | F2-10 |
| F2-13 | Identidad de la persona: el «quién» de una tarea es el usuario de GitHub o GitLab autenticado, no un texto libre (RR-12) | Dos personas no pueden figurar con el mismo nombre; el agente figura como agente | F1-6, F2-6, F2-9 |
| F2-14 | `taskctl desinstalar`: quita TaskCodeTwo de forma limpia (plugin, CLI, status line, hooks, worktree de la rama de estado); en un proyecto, la config solo con confirmación y nunca las tareas ni su historial sin confirmación explícita; con `--simular` (RR-16) | Tras desinstalar no queda nada de TaskCodeTwo en la máquina ni en el proyecto salvo lo que la persona decide conservar | F2-3, F2-10, F2-12 |

### Fase 3 — Comandos del flujo

| ID | Tarea | Criterio de aceptación | Dep. |
|---|---|---|---|
| F3-1 | `taskctl init` / `setup`: rama de estado, carpetas, config, workflow de CI y propuesta de rulesets | Repo vacío listo; idempotente; `--dry-run` | F2-3, F2-6 |
| F3-2 | `new` e `import` en bloque con dependencias e hito (RF-01..03) | Importa un hito de 37 tareas; repetirlo no duplica | F2-4, F2-5 |
| F3-3 | `next`: lo listo para empezar por dependencias y prioridad | Coincide con el orden del ROADMAP actual de ToolsaWeb | F3-2 |
| F3-4 | `roadmap`: la vista para todos (en curso con quién, en revisión, planificadas por hito, terminadas, descartadas) con dependencias y Mermaid, regenerada en la rama de estado tras cada cambio (RR-10, RR-11) | Sin conflictos con dos escrituras a la vez; no toca ramas de código | F2-3 |
| F3-5 | `board` y `status`: estado del equipo (quién lleva qué, qué espera revisión) desde la rama de estado y GitHub | Lo mismo para todo el equipo; sin ficheros en ramas de código | F2-3, F2-6 |
| F3-5b | `status` y `roadmap` siempre frescos: traen la rama de estado antes de mostrar y dicen la hora de la última sincronización (RR-12) | Nadie ve un estado de hace horas sin saberlo | F3-5 |
| F3-6 | `start`: toma atómica de la tarea, rama y worktree desde `develop` | Dos personas no pueden tomar la misma tarea; test E2E | F2-3, F2-2 |
| F3-7 | `descartar` y `pausa` (RF-07, RF-08) | Estado «descartada» con motivo, visible en el roadmap; el ID no se reutiliza | F2-3 |
| F3-8 | `doctor`: entorno, rama de estado, permisos, rulesets y checks reales | Detecta una protección de rama ausente | F3-1 |
| F3-9 | Inventario de skills y agentes instalados (máquina y proyecto) y `doctor` que avisa de un catálogo que nombra algo no instalado (RF-14) | Lista lo instalado; una recomendación a algo inexistente es error con el arreglo | F2-1, F1-15 |
| F3-10 | Liberar y reasignar tareas: `taskctl liberar` y `reasignar`, aviso de tareas tomadas sin actividad en N días, con aviso a quien la tenía (RR-12) | Una tarea abandonada vuelve a «lista para empezar» sin perder su trabajo ni su rama | F3-6 |
| F3-11 | Aviso de solape: al tomar una tarea, `start` dice qué tareas en curso (y de quién) tocan los mismos ficheros o módulos (RR-12) | Dos tareas que tocan el mismo módulo → aviso con nombres y tareas | F3-6, F4-1 |
| F3-13 | Estimación por tarea al planificar: tiempo por fase a partir de tareas parecidas; se recalcula al terminar cada fase (RR-19) | Cada tarea planificada tiene su estimación por fase; el error se mide | F1-17, F3-2 |
| F3-12 | Progreso en vivo en el terminal: fase actual, **fases que faltan** y **tiempo estimado restante**, en la status line de Claude Code y en `taskctl status --seguir`, escrito por el CLI (RR-19) | Durante una tarea real se ven las fases que faltan y el tiempo restante, actualizados en cada fase, sin coste de tokens | F3-13, F2-3 |

### Fase 4 — Diseño y aprobación

| ID | Tarea | Criterio de aceptación | Dep. |
|---|---|---|---|
| F4-1 | `plan` con brainstorm de roles (arquitectura, riesgos, testing, dominio, rendimiento) y unificador, proporcional al carril (RR-02) | Mismos roles que la 0.16; uno en fix, todos en riesgo alto | F3-2 |
| F4-2 | `approve` con la identidad de la persona; el agente no puede aprobar (RR-04) | Sin identidad de persona → rechazo; test | F1-6, F4-1 |
| F4-3 | Aprobación de un hito en lote | Diez planes aprobados de una vez, con nombre y fecha | F4-2 |
| F4-4 | Orquestador del modo automático: encadena plan → start → implementación → review → correcciones y para en las puertas del carril (RF-21) | Una tarea fix va de planificada a PR listo sin intervención | F3-6, F4-2, F6-2, F4-6 |
| F4-5 | Punto de relevo: TaskCodeTwo pasa a desarrollarse con la 2.0 | Las tareas pendientes de TaskCodeTwo, migradas con `actualizar` y trabajadas con la 2.0 | F4-4, F9-2, F9-7 |
| F4-6 | `plan` propone skills, agentes y modelo para diseño, implementación y revisión; se anotan en la tarea; el orquestador y la revisión los usan (RF-12, RF-13) | En 10 tareas reales la propuesta coincide con la que haría una persona; el modelo cambia según complejidad y carril | F4-1, F3-9, F6-3 |
| F4-7 | `taskctl lote TASK-a..TASK-b` (o lista): secuencia sin preguntas, sin trocear, aceptando recomendaciones, integración en la rama del lote y PR final contra `develop` (RF-27, RR-18) | Un lote de 5 tareas termina sin intervención y deja un único PR para la persona | F4-4, F4-6, F1-16 |
| F4-8 | Fallos, presupuesto, `--reanudar` e informe del lote | Un lote cortado se reanuda sin repetir; una tarea en rojo se aparta con sus dependientes y el resto sigue; informe completo | F4-7, F8-1 |
| F4-9 | Progreso del lote: tarea actual, tareas que faltan, apartadas y hora estimada de fin (RR-19) | En un lote de 5 tareas se ve cuántas faltan y cuándo acabará | F4-7, F3-12 |

### Fase 5 — Verificación sin redundancia

| ID | Tarea | Criterio de aceptación | Dep. |
|---|---|---|---|
| F5-1 | Workflow reutilizable de GitHub Actions y plantilla de GitLab CI con perfiles (rápida, impacto, completa, pesada) que llaman al comando del proyecto (RR-15) | ToolsaWeb y OpenDockTrading lo usan sin scripts nuevos; la plantilla de GitLab pasa en un proyecto de pruebas | F1-7 |
| F5-2 | Caché por SHA del árbol compartida por PR, cola de merge y release | Cero repeticiones sobre el mismo SHA, medido | F5-1 |
| F5-3 | Verificación por impacto como paso de CI y completa en la cola de merge | Tiempo de CI por PR antes y después | F5-1 |
| F5-4 | Informe de verificación en JSON para el revisor IA | El revisor lo cita y no repite tests | F5-1 |
| F5-5 | Runner self-hosted Windows por perfil | La pesada de ToolsaWeb corre en la máquina de Carlos | F5-1, F1-8 |
| F5-6 | Hook local rápido, opcional y con confianza | ≤ 1 s; nada del repo sin confianza | F1-8 |

### Fase 6 — Revisión con identidad y búsqueda de errores

| ID | Tarea | Criterio de aceptación | Dep. |
|---|---|---|---|
| F6-1 | Identidades: agente implementador (commits y PR/MR) y revisor (reviews y checks; nunca aprobar ni mergear), como GitHub App y como usuario bot o token de proyecto en GitLab | La persona aprueba un PR/MR del agente en GitHub y en GitLab | F1-6 |
| F6-2 | `review`: abre el PR contra `develop` con la identidad del agente y lanza el revisor | PR enlazado a la tarea; check `revision-ia` | F6-1, F2-6 |
| F6-3 | Revisor en CI con las skills de dominio de la 0.16 (angular-vue, rust-wasm, C#, Java, genérica), enrutado por lo que toca el diff y con el modelo que fije F4-6 (RR-02, RR-17) | Un diff Rust + Vue lanza las dos de dominio | F6-2, F5-4 |
| F6-4 | Búsqueda de errores: reproducción, mutantes y bordes con evidencia en cada hallazgo (RR-03) | Hallazgo sin evidencia = rechazado por el propio workflow | F6-3 |
| F6-5 | Hallazgos estructurados (JSON + comentarios); un CRÍTICO o IMPORTANTE abierto deja el check en rojo; informe también en la carpeta de la tarea | El check refleja los hallazgos | F6-4 |
| F6-6 | Rondas incrementales: la siguiente mira lo cambiado y si se cerraron los hallazgos; tope y MENORES sin ronda (RF-33) | Hallazgos nuevos por ronda, medidos | F6-5 |
| F6-7 | Defensa contra prompt injection en PR y tareas | Batería de PR maliciosos | F1-8, F6-3 |

### Fase 7 — Integración y publicación

| ID | Tarea | Criterio de aceptación | Dep. |
|---|---|---|---|
| F7-1 | `finish`: merge en `develop` por la cola cuando pasan las puertas; tarea en estado «terminada» con su `## Resultado` y el roadmap actualizado (RR-06) | Nada se integra con un check en rojo o sin aprobación | F6-5, F2-3 |
| F7-2 | `## Resultado` obligatorio (qué se hizo, divergencias, hallazgos no corregidos) | `finish` lo exige; test | F7-1 |
| F7-3 | `release` y `hotfix` con tag y notas desde los PR | Release reproducible en un repo de pruebas | F2-2 |
| F7-4 | CHANGELOG generado al publicar, no en cada cierre | Sin conflictos entre ramas | F7-3 |

### Fase 8 — Equipo, métricas e iteración del método

| ID | Tarea | Criterio de aceptación | Dep. |
|---|---|---|---|
| F8-1 | `metricas`: tiempo por fase, rondas, hallazgos, tokens y minutos de CI repetidos, por tarea, versión y repo | Reproduce las cifras de la medición de la 0.16 | F5-2, F6-5 |
| F8-2 | Presupuesto de tokens por carril con desglose (diseño, implementación, revisión, verificación, operaciones Git) | Cuadra con la transcripción | F8-1 |
| F8-3 | Ciclo de iteración: informe por versión frente a la anterior y auditoría externa por versión mayor (RR-05) | Ningún cambio del método sin dato | F8-1 |
| F8-4 | Espejo opcional en issues de GitHub o GitLab de la rama de estado | Quien quiera ve el tablero en la plataforma; la fuente siguen siendo las tareas y el roadmap | F3-5 |
| F8-5 | Guía de una página y referencia generada desde el código | Una persona nueva hace su primera tarea en 15 min | F3-*, F7-* |
| F8-6 | Medición del acierto: hallazgos por revisora y por rol, coste y resultado por modelo, en `metricas` y en el informe por versión (RF-15) | Informe que dice qué skill, agente o modelo rinde peor para su coste | F8-1, F4-6 |
| F8-7 | Métricas finales al cerrar cada tarea y cada lote: estimado frente a real por fase, tokens, rondas y hallazgos, en el terminal y guardadas; calibración de las estimaciones en el informe por versión (RR-19) | `finish` termina con su resumen; el informe por versión da el error medio de la estimación | F8-1, F3-13 |

### Fase 9 — Actualización de proyectos y piloto (RR-13)

| ID | Tarea | Criterio de aceptación | Dep. |
|---|---|---|---|
| F9-1 | `taskctl actualizar --simular`: informe de lo que cambiaría al pasar un proyecto de la 0.16 a la 2.0 | Sobre copias de ToolsaWeb y OpenDockTrading, sin tocar nada | F1-12, F2-5 |
| F9-2 | `taskctl actualizar`: lleva `tareas/` a la rama de estado aplanando las carpetas por estado (una carpeta fija por tarea) y conservando historial, migra la config, conserva IDs, dependencias e hitos; vuelta atrás con un comando | Copia de ToolsaWeb migrada: 404 tareas, mismas dependencias, roadmap igual | F9-1, F2-3 |
| F9-3 | Actualizaciones entre versiones de la 2.x (migraciones encadenadas) | 2.0 → 2.1 de prueba sin intervención | F9-2 |
| F9-4 | Suite E2E del flujo con **tres** personas a la vez: altas simultáneas, toma de la misma tarea, tareas que tocan los mismos ficheros, una tarea abandonada y reasignada (RR-12) | Ningún ID, rama ni tarea pisada; tablero correcto; todos ven lo mismo; merge solo con puertas | F7-*, F3-6, F2-4b, F2-13, F3-10, F3-11 |
| F9-4b | Prueba nocturna: lote de 10 tareas reales desatendido en un repo de pruebas (RR-18) | Por la mañana: PR del lote, informe y ninguna integración en rojo | F4-8, F9-4 |
| F9-5 | Prueba de seguridad (PR malicioso, config hostil, inyección en tareas) | Ninguno ejecuta código ni salta una puerta | F6-7 |
| F9-6 | Auditoría externa repetida con el mismo encargo | Las tres preguntas responden «sí» | F9-4, F9-5 |
| F9-7 | Piloto: OpenDockTrading actualizado y 10 tareas con la 2.0 | Métricas frente a la 0.16 | F9-2 |
| F9-8 | ToolsaWeb actualizado y 10 tareas con la 2.0 | Minutos de verificación repetidos ≈ 0 | F9-7, F4-5 |
| F9-8b | Piloto final: TaskCodeTwo actualizado a la 2.0 completa con `taskctl actualizar` y 10 tareas suyas trabajadas con ella, incluida al menos una en paralelo por dos personas o agentes | Sin pérdida de tareas ni historial; métricas frente a ToolsaWeb y OpenDockTrading; lo que falle bloquea la 2.0.0 | F9-8, F4-5 |
| F9-9a | Prueba de instalación en máquinas limpias (Windows, macOS, Linux) por ejecutable, por la línea de internet y por el marketplace | Primera tarea hecha en 15 min desde cero en cada sistema | F2-11, F8-5 |
| F9-9 | Publicar 2.0.0 (ejecutables, instaladores y plugin en el marketplace) | Changelog, guía de actualización y release | F9-6..8, F9-8b, F9-9a, F9-10 |
| F9-10 | Desinstalación limpia de TaskCode 0.16 en la máquina y en los proyectos (plugin, status line, hooks, ficheros de estado bajo `.git`, `logs/`, caché del marketplace), tras actualizar o sin hacerlo, con `--simular` (RR-16) | En una máquina y un proyecto con la 0.16 no queda rastro de ella; las tareas y su historial se conservan | F9-2, F1-12 |

### Fase 10 — Memoria del proyecto (RR-20)

> **Descartado el 2026-11-16** (TASK-113): la puerta de valor dio NO-GO y Carlos descartó la Fase 10; ver [informe](../mediciones/fase10-puerta-valor/informe.md). Se borraron sus tareas (TASK-096..TASK-112) y el hito H10; lo que sigue queda como registro.

Índice de contexto propio que sustituye a engram: git es la fuente de verdad y
una base SQLite local es un índice reconstruible que nunca queda desactualizado;
etiquetado automático acotado a un vocabulario, búsqueda única para el plugin,
conversores de las tareas existentes y de engram, y visor 3D opcional. Diseño y
motivo en [AMPLIACION-2026-11-16](AMPLIACION-2026-11-16.md).

| ID | Tarea | Criterio de aceptación | Dep. |
|---|---|---|---|
| F10-1 | Índice SQLite (rusqlite con FTS5 empaquetado), esquema y migraciones, `tc2 indice reconstruir` y `estado` (RF-77) | Dos máquinas con los mismos SHA → mismo contenido; nunca aparece en `git status` | F1-19, F2-5 |
| F10-2 | Frescura por SHA: comprobación en cada lectura, reindexado incremental, reconstrucción ante historia reescrita, transacción con cerrojo (RR-20) | Merge en la plataforma, push de otra persona, rebase y corte a mitad de reindexado: la siguiente búsqueda responde con lo actual; test | F10-1, F2-3 |
| F10-3 | Conversor de las tareas existentes: todas las `tarea.md` y su historial por `tipo(TASK-NNN)`, con renombrados y ficheros comodín (RF-78) | Las tareas de TaskCodeTwo y una copia de ToolsaWeb (404) indexadas; repetirlo no cambia nada | F10-1 |
| F10-4 | Ficheros tocados en `finish`: sección generada `## Archivos tocados` (ruta, nombre, A/M/D/R, líneas) y su relleno para las tareas antiguas (RF-79) | Coincide con `git diff --name-status` del merge; test | F10-3, F7-1 |
| F10-5 | Conversor de engram por proyecto: copia de solo lectura, alias de proyecto, enlace por `TASK-NNN`, relaciones como aristas, idempotente, `--simular`, filtro de secretos, materializado según F1-19 (RF-80) | Con la base real: 146 observaciones, 116 enlazadas a su tarea, alias resuelto, cero duplicados al repetir | F10-1, F10-3 |
| F10-6 | `tc2 buscar`: texto (BM25), etiqueta, fichero y vecinas de una tarea, con salida JSON (RF-81) | Respuesta en menos de 200 ms con 400 tareas; encuentra lo importado de engram | F10-2, F10-3 |
| F10-7 | El plugin busca en el índice: skills y agentes usan `tc2 buscar`; `doctor` avisa de un índice desfasado o de una skill que busca con grep en `tareas/` (RF-81) | Test que busca algo que solo existe en el índice; `doctor` detecta una skill con grep | F10-6, F3-8 |
| F10-8 | Etiquetado determinista: mapa de rutas a etiquetas, ADR citados, fase y tipo en `vocabulario.yml` (RF-82) | Las tareas terminadas de TaskCodeTwo etiquetadas sin IA; resultado reproducible | F10-3 |
| F10-9 | Vocabulario cerrado, alias y `tc2 tags doctor` (RF-82) | Detecta etiqueta única, comodín (>40 %), casi iguales y alias sin resolver | F10-8 |
| F10-10 | Etiquetado léxico e IA acotada en `finish` con cola de propuestas (RF-82) | La IA nunca escribe una etiqueta fuera del vocabulario; una propuesta entra solo con persona o N apariciones | F10-9, F7-1 |
| F10-11 | Relaciones derivadas: cocambio de ficheros con IDF, etiquetas compartidas, dependencias y engram, con peso y procedencia | Las vecinas de una tarea de TaskCodeTwo coinciden con las que daría una persona en 8 de 10 casos | F10-3, F10-8 |
| F10-12 | `tc2 grafo --json`: nodos, aristas, pesos y proyecciones (completa, por ficheros, por etiquetas, etiquetas y ficheros), con modo foco, agrupación por carpeta y umbral (RF-83) | El JSON de ToolsaWeb en modo foco tiene menos de 300 nodos; mismo resultado con los mismos SHA | F10-11 |
| F10-13 | Medición y retirada de engram: preguntas reales en TaskCodeTwo, ToolsaWeb y OpenDockTrading frente a la búsqueda de engram; si gana, engram sale del manifiesto de F1-14 | Informe con aciertos de cada uno; decisión de Carlos registrada | F10-5..F10-12 |
| F10-14 | Prueba de concepto del motor 3D: `3d-force-graph` frente a Babylon.js con d3-force-3d, con el grafo real de ToolsaWeb (RF-84) | Cifras de FPS, memoria y tamaño del bundle con el grafo completo y en modo foco; halo y pulso funcionando en los dos; decisión registrada | F10-12 |
| F10-15 | `tc2 visor`: servidor local en `127.0.0.1` con token, app embebida en el ejecutable y canal de eventos (`busqueda`, `fichero_cambiado`, `fichero_creado`, `fichero_borrado`, `indice_actualizado`) desde el índice y un vigilante de ficheros (RF-84) | Se abre en el navegador sin Node instalado; un cambio en un fichero llega al visor en menos de 1 s; cerrar el visor no afecta a `tc2` | F10-2, F10-6, F10-14 |
| F10-16 | App Vue del visor con dockview y PrimeVue: paneles de red completa 3D, por ficheros, por etiquetas, etiquetas y ficheros, búsqueda y detalle de tarea, con su disposición guardada (RF-84) | Las cuatro vistas abiertas a la vez con el grafo de ToolsaWeb; la disposición se conserva al volver a abrir | F10-15 |
| F10-17 | Efectos del visor y panel de actividad: pulso de la búsqueda (brillo y partículas en etiquetas y relaciones durante unos cientos de ms), halo verde o rojo en el fichero según líneas añadidas o borradas, aparición de los ficheros creados y apagado de los borrados, y panel de dockview con la pila de búsquedas (consulta, etiquetas, ficheros y tareas) y ficheros cambiados, creados o borrados en tiempo real, sincronizado con el grafo (RF-84) | Una búsqueda ilumina exactamente los nodos y aristas devueltos por `tc2 buscar`; añadir y borrar líneas destella verde y rojo en el fichero correcto; un fichero creado aparece en la red unido a la tarea en curso; cada evento aparece en la pila y en el grafo a la vez; pulsar una entrada de la pila resalta y centra sus nodos | F10-16 |

### Fase 11 — Panel web: dashboard y planificación (RR-11, RR-12)

> Añadida el 2026-11-18 por decisión de Carlos. Nace del dashboard de progreso y coste que se hizo para ToolsaWeb ese día, que sirve de referencia funcional. Ocupa el hito H11 (el H10 quedó descartado con la Fase 10).

`tc2 panel` sirve en local una página Vue de solo lectura con dos pestañas para cada programador: **Dashboard** (progreso, producción diaria, coste por día, por hito y por tarea, y quién hizo cada tarea) y **Planificación** (tareas por persona, en el orden de `tc2 next`, con sus predecesoras).

| ID | Tarea | Criterio de aceptación | Dep. |
|---|---|---|---|
| F11-1 | ADR del panel web: servidor local de solo lectura con token, Vue embebido, origen de los datos y contrato JSON v1 | ADR con alternativas, decisión y motivo | — |
| F11-2 | Datos del panel: tareas con estado, hito, persona, dependencias, tokens por fase y fechas (`tc2 panel --json`) | Con ToolsaWeb, 399 tareas y 171 terminadas a 2026-11-18; lee la 0.16 y la rama de estado | F11-1 |
| F11-3 | Coste por tarea con tarifas por modelo configurables y desglose de entrada, salida y caché | Coste exacto donde F8-1 da el desglose y estimación marcada donde no | F11-2, F8-1 |
| F11-4 | Esqueleto web Vue servido por `tc2 panel`: pestañas, tema y build embebido | Se abre sin Node ni red; build reproducible | F11-1 |
| F11-5 | Pestaña Dashboard con persona de cada tarea y filtros por persona e hito | Reproduce las cifras del dashboard de referencia de ToolsaWeb | F11-3, F11-4 |
| F11-6 | Pestaña Planificación: tareas por persona, predecesoras y sucesoras y grafo de dependencias | Mismo orden que `tc2 next`; avisa de esperas entre personas | F11-2, F11-4, TASK-122 |
| F11-7 | Proyección de fechas por persona y por hito con el ritmo medido | Sin datos de ritmo no inventa fechas | F11-6 |
| F11-8 | Pruebas E2E del panel y paridad con el dashboard de referencia | Pasa en Windows, macOS y Linux | F11-5..F11-7 |

## 9. Criterios de salida de la 2.0

- Los **veinte requisitos rectores** se cumplen, cada uno con su prueba o su
  cifra; todos los requisitos **I** del catálogo, cubiertos.
- Ningún paso del flujo pide al agente escribir `git` o `gh`, y los tokens por
  tarea bajan frente a la 0.16.
- El merge en `develop` y en `main` es imposible sin checks verdes y la
  aprobación de una persona, en GitHub y en GitLab.
- Dos personas trabajan a la vez sin conflictos de IDs, estado ni ficheros
  generados; el roadmap no toca la verificación.
- ToolsaWeb y OpenDockTrading actualizados con `taskctl actualizar`, sin perder
  tareas, dependencias ni historial.
- Cero verificaciones repetidas sobre el mismo SHA.
- Un lote nocturno de 10 tareas acaba en un único PR revisable por la mañana.
- Durante cada tarea se ven en el terminal las fases que faltan y el tiempo estimado.
- Se desinstalan limpiamente TaskCodeTwo y TaskCode 0.16.
- Se instala en una máquina limpia de cada sistema desde el marketplace de
  Claude Code, igual que TaskCode, sin `npm install` ni bash; y con el
  ejecutable o la línea por internet para usos fuera de Claude Code.
- La memoria del proyecto es propia: toda búsqueda del plugin pasa por
  `tc2 buscar`, nunca responde con datos viejos y engram no hace falta (RR-20).
- CI verde en Linux, macOS y Windows; la auditoría externa repetida dice «sí».

## 10. Orden y paralelismo

1. **Fase 0 ya**, en el repo TaskCode.
2. **F1-0** (crear TaskCodeTwo e importar este plan) y después los ADR:
   F1-1 → F1-2, F1-3 → F1-4/F1-5 → F1-6 → F1-8; F1-7, F1-9, F1-10, F1-11,
   F1-12 y F1-13 en cuanto tengan su dependencia.
3. Fase 2 → Fase 3 → Fase 4. En paralelo, F5-1 (workflow de verificación) y
   F6-1 (identidades), que no dependen del núcleo.
4. **F4-5: relevo**, TaskCodeTwo pasa a la 2.0 (requiere F9-2).
5. Fases 6–8, y la Fase 9 al final, con el piloto en OpenDockTrading antes que
   ToolsaWeb.
6. ~~**Fase 10** (memoria del proyecto)~~ — descartada el 2026-11-16 (TASK-113): F1-19 → F10-1 → F10-2 y F10-3 → resto;
   el conversor de tareas va pronto para medir con datos reales antes de la IA.

## Fuentes

- [GitHub Spec Kit](https://github.github.com/spec-kit/) · [anuncio de GitHub](https://github.blog/news-insights/product-news/spec-driven-development-with-ai-get-started-with-a-new-open-source-toolkit/) · [Visual Studio Magazine, 2026](https://visualstudiomagazine.com/articles/2026/05/12/github-spec-kit-takes-off-as-antidote-to-piecemeal-vibe-coding.aspx)
- [CCPM (Algolia DocSearch)](https://docsearch.algolia.com/mcp/docs/repo/automazeio/ccpm) · [Show HN: CCPM](https://hn.svelte.dev/item/44960594)
- [Kiro specs](https://kiro.dev/docs/specs/index)
- [superpowers frente a BMAD](https://www.libhunt.com/compare-superpowers-vs-BMAD-METHOD) · [Marcos de skills agénticas](https://rywalker.com/research/agentic-skills-frameworks) · [BMAD con Claude Code](https://pasqualepillitteri.it/en/news/171/bmad-framework-claude-code-agile-development)
- [«265,000 Stars and I Don't Use Any of It»](https://vanja.io/265000-stars/) · [The Great Agent Skills Land Grab](https://denodell.com/blog/the-great-agent-skills-land-grab)
- [Task Master, PRD](https://glama.ai/mcp/servers/@eyaltoledano/claude-task-master/blob/f961235442aa88e4983b29ab458b6a476fadf370/.taskmaster/docs/prd.txt)
- [GitHub: checks obligatorios](https://docs.github.com/en/enterprise-cloud@latest/pull-requests/how-tos/merge-and-close-pull-requests/troubleshooting-required-status-checks) · [Agentes IA y PR en GitHub Actions](https://tenki.cloud/blog/ai-agent-pr-playbook-github-actions) · [Riesgo de un agente que se salta un check (Civic)](https://www.civic.com/use-cases/claude-code)

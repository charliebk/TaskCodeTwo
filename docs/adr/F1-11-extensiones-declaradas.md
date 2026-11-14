# ADR F1-11 — Extensiones declaradas: tres claves, derivados en la transición y lectura estable del estado

- Estado: **aceptada** (2026-11-14)
- Tarea: TASK-012 · Plan: [F1-11 en PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) (§4 P8 y tabla de la Fase 1)
- Requisitos: la fila F1-11 del plan cita P8; este ADR responde a RN-04, RF-07, RF-08, RF-40, RF-41, RF-42, RF-70, RF-71, RF-72 y RR-11 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md) y a H2 y H16 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
- Depende de: [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (carpeta fija, rama de estado huérfana, roadmap con un Mermaid por hito) y [ADR F1-7](F1-7-verificacion-ci-perfiles-cache-runners.md) (un comando con perfiles, huella del árbol y local, runner por perfil, job programado); cierra costuras abiertas por [F1-8](F1-8-seguridad-modelo-de-amenazas.md) (confianza local única, globs extra, `comando_*` como entrada no confiable, rutas de Windows en YAML), [F1-9](F1-9-formato-de-tarea-plan-y-especificacion.md) (claves desconocidas del frontmatter) y [F1-10](F1-10-carriles-y-modo-automatico.md) (configuración que falla cerrada, tope de 12 claves)
- Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a, D5a y D6a del plan de TASK-012), «no trocear» y una restricción expresa: el plugin tiene que ser **eficiente y con ventajas medibles**, y prefiere **comandos deterministas** que sabe que hacen lo que dicen a hooks que arrancan algo según el contexto

## Contexto

P8 pide «pocos comandos para el día a día; lo avanzado, en extensiones
declaradas (estados propios, derivados, verificación propia, revisoras
propias) en vez de convenciones a mano». RN-04 lo repite como requisito nuevo:
«los proyectos ya extienden el plugin a mano». La fila F1-11 del plan lo
concreta: un **contrato que cubre lo que hoy hacen ToolsaWeb y
OpenDockTrading a mano**.

Lo que ya está decidido y este ADR no cambia:

- **F1-1**: cada tarea es la carpeta fija `TASK-NNN/` en la rama huérfana
  `taskcode/tareas`, montada en `tareas/`; el `ROADMAP.md` lo regenera el CLI
  con **un Mermaid por hito no cerrado**. Dejó a F1-11 si los estados de P1
  bastan a los proyectos con `05-archivadas` (RF-71).
- **F1-7**: un solo comando del proyecto llamado con
  `--perfil rapida|impacto|completa|pesada`; código 0 verde, cualquier otro
  rojo; huella del árbol y huella local; runner por perfil; los pasos que
  dependen de la red salen a un job programado. Dejó a F1-11 las claves, y
  que `completa` y `pesada` no dependan de `RAMA_BASE`.
- **F1-8**: `confianza(repo, comando) -> confiado | cambiado | nunca` es la
  única interfaz para todo `comando_*`; los globs extra de la huella los
  declara el proyecto (F1-11); con un lector YAML estándar,
  `"scripts\verificar.bat"` lee `\v` como tabulador vertical.
- **F1-9**: una clave desconocida del frontmatter de `tarea.md` se conserva
  y `doctor` avisa (D4a, «costura con F1-11»).
- **F1-10**: la configuración se lee de la base, va bajo CODEOWNERS, falla
  cerrada ante lo desconocido y suma al tope de 12 claves de F2-7
  (`politica_carriles` y `rutas_carril`).

La restricción de la persona fija el criterio: el contrato de extensión son
**comandos declarados que `taskctl` ejecuta en puntos explícitos**
(transición, verificación). Ningún punto de extensión depende de un hook; un
hook, si existe, solo es un atajo opcional que llama al mismo comando.

### Inventario de lo que se hace hoy a mano

Leído de commits fijos (ToolsaWeb `cb46c0f1b`, OpenDockTrading `f8a9b5f27`,
los dos del 2026-11-14) y del plugin 0.16.1. La PoC A comprueba cada cita:
el fichero existe en ese commit y las líneas contienen el texto entre «».
`TW` es ToolsaWeb, `ODT` OpenDockTrading y `PL` el plugin 0.16.1. «En la 0.16»
dice cómo se hace hoy: con una clave de la configuración, con algo que pone
el plugin o a mano. «Destino» dice dónde queda en la 2.0: una clave de la
tabla «Esquema» (decisión 1), algo nativo del CLI, una skill o fuera de
alcance.

<!-- inventario:inicio -->
| # | Práctica | Evidencia (repo fichero:línea «texto») | En la 0.16 | Destino | Punto de extensión en la 2.0 |
|---|---|---|---|---|---|
| 1 | Estado propio `05-archivadas`: carpeta, etiqueta `descartada` y un `estado` que no lo dice | `TW tareas/05-archivadas/TASK-171/tarea.md:6` «descartada»; `TW tareas/05-archivadas/TASK-171/tarea.md:9` «estado: planificada»; `ODT tareas/05-archivadas/TASK-068/tarea.md:6` «descartada»; `ODT tareas/05-archivadas/TASK-068/tarea.md:9` «estado: planificada» | a mano | nativo | estado `descartada` con motivo y `pausa` (F3-7); sin clave de estados propios (D1a) |
| 2 | Regenerar el Mermaid de `hitos.md` y `ROADMAP.md` en cada transición | `TW .taskcode/config.yml:29-32` «comando_sincronizacion»; `TW scripts/grafo-tareas.py:4-5` «regenera el Mermaid»; `ODT tareas/05-archivadas/TASK-068/tarea.md:38` «la transición sigue y avisa» | clave | clave | roadmap nativo con un Mermaid por hito (F1-1 dec. 3); lo demás, un derivado `genera` (`derivados[].tipo`) que solo escribe bajo `tareas/` (`derivados[].escribe`, D2a) |
| 3 | Comprobaciones que cruzan código y estado, dentro de la verificación | `TW scripts/verificar.bat:124-127` «grafo-tareas.py --comprobar»; `ODT scripts/verificar.bat:51` «comprobar-modulos.py --comprobar»; `ODT scripts/verificar.bat:53` «comprobar-fallos.py --probar»; `ODT scripts/comprobar-modulos.py:88` «lector-tareas.py»; `ODT scripts/comprobar-fallos.py:302-307` «tareas_vivas» | a mano | clave | derivado `comprueba` (`derivados[].tipo`) en cada transición, fuera del perfil cacheado, más el check `estado` en el PR (D3a) |
| 4 | Lector propio de frontmatter que recorre `tareas/0?-*/TASK-*/tarea.md` | `TW scripts/grafo-tareas.py:99` «raiz_tareas.glob»; `ODT scripts/lector-tareas.py:39` «raiz_tareas.glob» | a mano | nativo | `taskctl tareas --json` como lectura estable; la forma de las carpetas no es contrato |
| 5 | Cuatro perfiles con seis claves de comando y tiempo | `TW .taskcode/config.yml:9` «comando_verificacion:»; `TW .taskcode/config.yml:16` «comando_verificacion_freno»; `TW .taskcode/config.yml:18` «comando_verificacion_selectiva»; `TW .taskcode/config.yml:23` «comando_verificacion_pesada»; `TW docs/verificacion/flujo-de-verificacion.md:9-12` «comando_verificacion_freno» | clave | clave | `verificacion.comando` llamado con `--perfil` (F1-7 dec. 1), con su `verificacion.shell` y `verificacion.perfiles.<p>.runner` |
| 6 | Tiempo máximo por perfil y presupuesto del freno | `TW .taskcode/config.yml:10` «timeout_verificacion: 4800»; `TW .taskcode/config.yml:17` «presupuesto_freno_s: 180»; `TW .taskcode/config.yml:20` «timeout_verificacion_selectiva: 1800»; `ODT .taskcode/config.yml:10` «timeout_verificacion: 1200» | clave | clave | `verificacion.perfiles.<p>.timeout` |
| 7 | Caché propia por paso y «completa del mismo árbol» | `TW scripts/cache-verificacion.py:1` «Caché por paso de verificar.bat»; `TW scripts/verificar.bat:91-93` «completa-leer»; `TW docs/verificacion/flujo-de-verificacion.md:164-165` «mismo árbol exacto» | a mano | clave | `verificacion.huella_arbol` y `verificacion.huella_local` (F1-7 dec. 2); la caché por paso sigue dentro del comando |
| 8 | Canales de texto: el x/N de `TASKCTL_PROGRESO` y la línea «# tests N» | `TW scripts/verificar.bat:51-52` «TASKCTL_PROGRESO»; `TW .taskcode/config.yml:7-8` «# tests N»; `PL src/fs/progreso.ts:33` «TASKCTL_PROGRESO» | plugin | nativo | variable de progreso que pasa `taskctl`; el recuento, al informe JSON (F5-4) |
| 9 | `RAMA_BASE` como entrada del comando | `TW scripts/clasificar-verificacion.py:14-15` «RAMA_BASE (la pasa TaskCode 0.13)»; `TW scripts/clasificar-verificacion.py:54-59` «os.environ.get("RAMA_BASE")» | plugin | nativo | `RAMA_BASE` solo en `rapida` e `impacto` (F1-7 dec. 1); la completa de ToolsaWeb no depende de ella (PoC E) |
| 10 | Pasos que dependen de la red | `ODT scripts/verificar.bat:68-69` «cargo audit --no-yanked --deny unsound»; `ODT scripts/verificar.bat:70` «npm --prefix web audit» | a mano | clave | `verificacion.programada.comando` con `verificacion.programada.cron`: job programado, fuera del check cacheado (F1-7 dec. 2) |
| 11 | Cerrojo propio con códigos 3 y 4 | `TW scripts/cerrojo-verificacion.py:6-7` «3 si hay otra»; `TW docs/verificacion/flujo-de-verificacion.md:182` «código 3»; `ODT scripts/verificar.bat:16-17` «impide dos» | a mano | clave | sigue dentro de `verificacion.comando`; para el contrato, cualquier código distinto de 0 es rojo (D6a) |
| 12 | Ruta de Windows en YAML (`"\v"`) | `ODT .taskcode/config.yml:5-6` «seria un escape»; `TW .taskcode/config.yml:9` «"scripts\verificar.bat"» | a mano | clave | `verificacion.comando` y `derivados[].comando` entre comillas simples o con `/`; el validador rechaza un carácter de control (PoC B) |
| 13 | Revisoras propias: ningún proyecto las declara; el catálogo sale del plugin | `PL src/core/revisores.ts:69-74` «CLAUDE_PLUGIN_ROOT»; `PL src/core/revisores.ts:113-116` «rol: revisor» | plugin | skill | skill del proyecto con bloque `rol: revisor` y `patrones_archivo`, sin clave en la configuración (D5a); la selección, F1-15 |
| 14 | Guardas del arnés en `.claude/settings.json` (`deny git checkout`, `ask --ronda-extra`, hook Stop si HEAD está desligado) | `TW .claude/settings.json:3-9` «git checkout»; `TW .claude/settings.json:11-13` «--ronda-extra»; `TW .claude/settings.json:17-23` «symbolic-ref -q HEAD» | a mano | fuera | se quedan en `.claude/`: no son extensión de TaskCode y la 2.0 no depende de ellas; lo que protegen lo hacen comandos y puertas (F1-3, F1-5) |
<!-- inventario:fin -->

Lectura: de 14 prácticas, la 0.16 cubre 3 con una clave y 3 con algo que
pone el plugin; **8 se hacen a mano**. En la 2.0, 13 tienen un destino
declarado (8 claves del esquema, 4 nativas, 1 skill) y la 14, las guardas
del arnés, queda fuera de alcance por decisión (ver «Fuera de alcance»).

## Alternativas

| | Descripción | Veredicto |
|---|---|---|
| **D1a** | Sin clave de estados propios: `descartada` y `pausa` nativos (F3-7) | **Elegida**: cubren los 11 de 11 casos medidos de `05-archivadas` (PoC A) |
| D1b | Clave `estados` con estados del tablero configurables (lo que sugiere RF-71) | Descartada: una máquina de estados configurable para 11 casos que ya cubre un estado nativo; el validador la rechaza como clave desconocida (PoC B) |
| **D2a** | Los derivados solo escriben bajo `tareas/` (rama de estado) | **Elegida**: el árbol de `develop` no cambia en 7 transiciones (PoC C, RR-11) |
| D2b | Los derivados escriben también en ramas de código (`docs/planificacion/hitos.md`) | Descartada: cada transición cambiaría el árbol verificado (F1-1, RR-11) y la escritura saltaría la protección de `develop` |
| **D3a** | Comprobaciones que cruzan código y estado como derivados `comprueba` en cada transición, más un check `estado` no cacheado en el PR que lee `taskcode/tareas` | **Elegida**: un resultado que depende del estado no puede vivir en un perfil cacheado por árbol (F1-7 dec. 2) |
| D3b | Dejarlas dentro del comando de verificación | Descartada: el check `verificacion` se cachea por árbol y el estado no está en el árbol: un acierto escondería un fallo nuevo |
| **D4a** | Tres bloques: `verificacion`, `derivados`, `confianza_incluye` | **Elegida**: 3 claves de primer nivel; con las 2 de F1-10, 5 de 12 (PoC B) |
| D4b | Claves planas al estilo 0.16 (`comando_verificacion_*`, `timeout_*`, `comando_sincronizacion`…) | Descartada: el catálogo de la 0.16 ya tiene 12 claves solo de verificación y sincronización; no caben en el tope de 12 |
| **D5a** | Revisoras propias como skill del proyecto con bloque `rol: revisor` y `patrones_archivo`, sin clave | **Elegida**: es el formato que ya lee el plugin (`revisores.ts`); F1-15 decide la selección |
| D5b | Clave `revisoras` en la configuración | Descartada: duplica lo que dice la skill y gasta una de las 12 claves para algo que hoy usan 0 proyectos (PoC A) |
| **D6a** | «No se pudo verificar» (códigos 3 y 4 del cerrojo de ToolsaWeb) es rojo como cualquier código distinto de 0 | **Elegida**: el contrato de F1-7 es binario |
| D6b | Un código propio de «no verificado» que no cuenta como rojo | Descartada: abre una tercera salida que el check y el agente tendrían que interpretar |
| E1 | Derivados ejecutados por un hook (fin de turno, `post-checkout`) | Descartada por la restricción de la persona: un hook arranca según el contexto, no en un punto que el agente y la persona ven |
| E2 | Derivados en un job de CI por transición (lo que sugiere RF-70) | Descartada: el push a `taskcode/tareas` no dispara CI (F1-1, «Cómo se ve en GitHub») y añadiría minutos y una espera a cada transición |

## Pruebas de concepto

Guion `poc-f1-11.py` y su salida completa `poc-f1-11.log`, en la carpeta
`poc/` de TASK-012 (sin enlace: la carpeta de la tarea cambia de estado al
cerrarse). Ejecutado con Python 3.12.10 (`python -I -B`), Git 2.56 y PyYAML
6.0.3, con la carpeta de trabajo en
`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-012\poc\` (borrada al
terminar). **Solo lectura**: ToolsaWeb y OpenDockTrading se leen con
`git show`, `git ls-tree`, `git grep` y `git archive` de los commits fijos;
lo que se ejecuta de ellos (D y E) es una **copia** en la carpeta de trabajo,
nunca dentro de esos repos. El plugin 0.16.1 solo se lee. No hay llamadas a
GitHub. Dos ejecuciones seguidas dan el mismo log.

```bash
cd sources/Personal/TaskCodeTwo
python -I -B tareas/<estado>/TASK-012/poc/poc-f1-11.py \
  "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-012/poc" \
  docs/adr/F1-11-extensiones-declaradas.md .. \
  ~/.claude/plugins/cache/taskcode-marketplace/taskcode-plugin/0.16.1 > poc-f1-11.log
```

El guion **lee tres tablas de este fichero** entre marcadores: `inventario`
(A comprueba sus citas), `esquema` (tiene que coincidir clave a clave con el
validador del guion) y `ventaja` (cada cifra tiene que ser la medida). Si
alguna no cuadra, sale con 1.

### A — Cobertura del inventario

```text
claves en la tabla esquema del ADR: 17 ; en el validador: 17 ; iguales: True
citas comprobadas: 43/43
en la 0.16: por clave=3 ; del plugin=3 ; a mano=8 (de 14)
en la 2.0: clave=8 ; nativo=4 ; skill=1 ; fuera de alcance=1 ; sin destino=0
-- estados propios (practica 1): 05-archivadas
  ToolsaWeb: 9 en 05-archivadas ; con etiqueta descartada: 9 ; estado del frontmatter: en-diseno=1 (TASK-097) ; planificada=8 (TASK-006 TASK-010 TASK-030 TASK-031 TASK-054 TASK-055 TASK-056 TASK-171)
  OpenDockTrading: 2 en 05-archivadas ; con etiqueta descartada: 2 ; estado del frontmatter: planificada=2 (TASK-015 TASK-068)
  casos que cubre el estado nativo `descartada`: 11/11
-- revisoras propias (practica 13): skills con `rol: revisor` en los pilotos
  ToolsaWeb: 0
  OpenDockTrading: 0
RESULTADO A practicas=14 ; con destino=13 ; citas=43/43
```

Lectura: las 43 citas existen y dicen lo que el inventario afirma. Las 11
tareas archivadas llevan la etiqueta `descartada` y **ninguna** tiene un
`estado` que lo diga (10 `planificada`, 1 `en-diseno`): el estado nativo
`descartada` con motivo las cubre todas y quita la incoherencia.

### B — Esquema, hostiles, confianza y presupuesto

Configuraciones de ToolsaWeb y OpenDockTrading traducidas al esquema nuevo
(en el guion, `CFG_TW` y `CFG_ODT`), validadas con `yaml.safe_load` y el
validador de la decisión 1:

```text
  ToolsaWeb traducida -> valida ; claves de primer nivel: ['confianza_incluye', 'derivados', 'verificacion']
  OpenDockTrading traducida -> valida ; claves de primer nivel: ['confianza_incluye', 'derivados', 'verificacion']
  TW verificacion.huella_local: scripts/huella-local.py NO EXISTE en el commit fijo (a crear)
  verificar.bat:68-79 acepta ['--selectiva', '--pesada', '--rapida'] ; otro argumento -> exit /b 2: True ; «--perfil completa» lo acepta: False
-- hostiles (falla cerrado)
  clave desconocida (la 0.16 en plano)                               -> rechazada: clave desconocida comando_verificacion_freno
  clave desconocida dentro de verificacion                           -> rechazada: verificacion.presupuesto_freno_s: clave desconocida
  ruta de Windows con \v entre comillas dobles (linea 9 de ToolsaWeb) -> rechazada: verificacion.comando: caracter de control '\x0b' en la orden (¿\v, \t o \n dentro de comillas dobles?): escrib
  derivado que escribe fuera de tareas/ (hitos.md de ToolsaWeb)      -> rechazada: derivados[0].escribe: 'docs/planificacion/hitos.md' fuera de tareas/ (D2a: solo la rama de estado)
  derivado que escribe el ROADMAP nativo                             -> rechazada: derivados[0].escribe: 'tareas/ROADMAP.md' lo escribe taskctl (roadmap o carpeta de tarea)
  derivado que escribe una carpeta de tarea                          -> rechazada: derivados[0].escribe: 'tareas/TASK-001/tarea.md' lo escribe taskctl (roadmap o carpeta de tarea)
  derivado comprueba que escribe                                     -> rechazada: derivados[0].escribe: un derivado comprueba no escribe nada
  derivado con fase inventada                                        -> rechazada: derivados[0].fases: subconjunto no vacio de ['new', 'import', 'plan', 'approve', 'start', 'review', 'finish', 
  estado propio declarado (D1a: sin clave)                           -> rechazada: clave desconocida estados
  perfil fuera de F1-7                                               -> rechazada: verificacion.perfiles.selectiva: perfil desconocido (los de F1-7: ['rapida', 'impacto', 'completa', 'pesada'])
  rapida con runner (corre en local)                                 -> rechazada: verificacion.perfiles.rapida.runner: rapida corre en local (F1-7 dec. 4); sin runner
  confianza_incluye que hashea todo el arbol                         -> rechazada: confianza_incluye: globs sin negacion, sin rutas absolutas ni .., y no todo el arbol
  timeout como texto                                                 -> rechazada: verificacion.perfiles.completa.timeout: entero de 1 a 21600 segundos
  lector YAML estandar sobre la config real de ToolsaWeb: comando_verificacion = 'scripts\x0berificar.bat'
  y sobre la de OpenDockTrading (sin comillas): comando_verificacion = 'scripts\\verificar.bat'
-- confianza (F1-8 dec. 5): una orden sin confianza no se ejecuta
  sin entrada: nunca ; tras confiar: confiado ; script cambiado: cambiado ; vuelta al contenido: confiado ; fichero de confianza_incluye cambiado: cambiado ; orden cambiada: cambiado ; cierre=['scripts/ayuda.py', 'scripts/grafo.py']
-- presupuesto de claves y ventaja frente a la 0.16
  claves del catalogo de la 0.16 (src/core/config.ts, tabla de cabecera): 32
  de ellas, de verificacion y sincronizacion (las sustituyen verificacion y derivados): 12
  ToolsaWeb: claves en su config 0.16 = 12 ; de verificacion y sincronizacion = 10 (comando_sincronizacion, comando_verificacion, comando_verificacion_freno, comando_verificacion_pesada, comando_verificacion_selectiva, presupuesto_freno_s, rutas_sincronizacion, timeout_verificacion, timeout_verificacion_selectiva, verificacion_pesada_en) ; en el esquema nuevo = 2 (derivados, verificacion) ; resto, de F2-7: ['modo_flujo', 'tope_rondas']
  OpenDockTrading: claves en su config 0.16 = 3 ; de verificacion y sincronizacion = 2 (comando_verificacion, timeout_verificacion) ; en el esquema nuevo = 2 (derivados, verificacion) ; resto, de F2-7: ['modo_flujo']
  presupuesto: F1-11 3 + F1-10 2 = 5 de 12 (F1-8 no anade clave propia: sus globs son confianza_incluye)
RESULTADO B traducidas validas=2/2 ; hostiles rechazadas=13/13 ; claves=5/12
```

Lectura: las dos traducciones valen y las 13 hostiles se rechazan con un
mensaje que dice qué arreglar. La línea 9 de ToolsaWeb, leída con un lector
YAML estándar, da `scripts<TAB-vertical>erificar.bat`; el validador lo para.
Lo que la traducción nombra existe en el commit fijo salvo una orden:
ToolsaWeb no tiene hoy una orden de huella local (su lógica está dentro de
`cache-verificacion.py`), y su `verificar.bat` no acepta `--perfil`.
OpenDockTrading **no baja de claves** (2 → 2): gana un derivado `comprueba`
que saca `comprobar-modulos.py` del perfil cacheado (D3a), y el job
programado para sus auditorías de red va dentro de `verificacion`.

### C — Derivados en la rama de estado

Repo temporal con `develop` y la rama huérfana `taskcode/tareas` montada en
`tareas/` (Git 2.56, `worktree add --orphan`). Dos derivados declarados en
`.taskcode/config.yml` (un `genera` estilo `grafo-tareas`, que escribe
`tareas/grafo-hitos.md`, y un `comprueba` que mira que cada `TASK-NNN` citada
en el código exista en el estado) y una función `transicion` que hace lo
que haría `taskctl <fase>`: escribe la tarea, ejecuta los derivados de esa
fase **solo si están confiados**, con `TASKCODE_TAREAS` y `TASKCODE_CODIGO`,
descarta lo que escriban fuera de `escribe`, y commitea todo en un commit de
la rama de estado.

```text
merge-base develop taskcode/tareas: NINGUNA (huerfana)
-- transiciones
  T1 plan TASK-001                               commit=['TASK-001/tarea.md', 'grafo-hitos.md'] ; grafo regenerado=True
  T2 approve TASK-001                            commit=['TASK-001/tarea.md', 'grafo-hitos.md'] ; grafo regenerado=True
  T3 plan TASK-003 con dependencia rota          commit=['TASK-003/tarea.md'] ; grafo regenerado=False
      aviso: derivados[0] fallo (codigo 1: dependencia inexistente: TASK-003 -> TASK-099); la transicion sigue; arreglalo y ejecuta `taskctl derivados`
  T4 review TASK-001 con el generador cambiado   commit=['TASK-001/tarea.md'] ; grafo regenerado=False
      aviso: derivados[0] cambiado: no se ejecuta; revisa la orden y `taskctl confiar derivados[0]`
  T5 finish TASK-001 (generador restaurado)      commit=['TASK-001/tarea.md', 'TASK-003/tarea.md', 'grafo-hitos.md'] ; grafo regenerado=True
-- hostiles en ejecucion
  T6 finish TASK-002, codigo cita TASK-404       commit=['TASK-002/tarea.md', 'grafo-hitos.md'] ; grafo regenerado=True
      aviso: derivados[1] fallo (codigo 1: citadas en el codigo y sin tarea: TASK-404); la transicion sigue; arreglalo y ejecuta `taskctl derivados`
  T7 pausa TASK-003, derivado que escribe en codigo commit=['TASK-003/tarea.md'] ; grafo regenerado=False
      aviso: derivados[0] escribio fuera de `escribe` (['src/intruso.txt']): descartado; la transicion sigue
arbol de develop antes=f7b7e77bd30e despues=f7b7e77bd30e identico=True ; commits en develop 1->1 ; status limpio=True
commits en taskcode/tareas: 8 (alta + 7 transiciones, uno por transicion)
grafo-hitos.md final: TASK-001 fuera (terminada)=True ; TASK-003 en H2=True
RESULTADO C transiciones=7 ; como se espera=7/7 ; develop identico=True
```

Lectura: lo generado va en el mismo commit que la transición; un derivado que
falla, que no está confiado o que escribe fuera de lo declarado no para la
transición, deja su aviso con el comando que lo arregla y no ensucia nada; el
árbol de `develop` es idéntico tras 7 transiciones (RR-11).

### D — Lectura del estado

Los lectores propios **copiados** (`lector-tareas.py` de OpenDockTrading,
`grafo-tareas.py` de ToolsaWeb) sobre las tareas reales de cada piloto, en la
estructura de la 0.16 y en la carpeta fija de F1-1; frente a un
`taskctl tareas --json` simulado (frontmatter leído con YAML estándar; las de
`05-archivadas` migran con estado `descartada`).

```text
  OpenDockTrading (scripts/lector-tareas.py): tareas.md=96 ; vivas hoy (0.16)=94 ; mismo glob en carpeta fija=0 ; desde tareas --json=94 ; mismo conjunto que hoy=True
  ToolsaWeb (scripts/grafo-tareas.py): tareas.md=404 ; vivas hoy (0.16)=395 ; mismo glob en carpeta fija=0 ; desde tareas --json=395 ; mismo conjunto que hoy=True
  lineas que deja de necesitar un lector propio: OpenDockTrading lector-tareas.py entero = 44 ; ToolsaWeb grafo-tareas.py {'quitar_comentario': 7, 'leer_tarea': 24, 'cargar': 7} = 38 ; el consumidor de tareas --json = 3
  ToolsaWeb grafo-tareas.py que solapa con el roadmap nativo (F1-1 dec. 3, F3-4) {'mermaid': 25, 'orden_de_trabajo': 17, 'roadmap': 48} = 90 (no se cuenta como ahorro: falta ver que F3-4 cubra su orden)
  paso de OpenDockTrading que desaparece con el roadmap fuera del arbol (F1-7): verificar.bat:[59]
RESULTADO D ok
```

Lectura: con la carpeta fija los dos lectores **dejan de ver todas las
tareas** (0 de 94 y 0 de 395), sin error: un fallo silencioso. Con
`tareas --json`, el mismo cálculo da el mismo conjunto con un consumidor de 3
líneas.

### E — La completa de ToolsaWeb y `RAMA_BASE`

Copia del commit fijo de ToolsaWeb sin `docs/`, `tareas/` ni `evidencias/`
(958 ficheros), con `develop` y una rama que toca `tw-superficie` (un crate
con tests pesados); `clasificar-verificacion.py plan --completa` con cuatro
valores de `RAMA_BASE`, con la caché vacía en cada uno:

```text
  RAMA_BASE sin definir      : codigo 0 ; modo=selectiva es_completa=True ; grupos=7 ; cacheados=0 ; aviso_pesada=['tw-superficie'] ; rango='develop...HEAD + cambios sin commit'
  RAMA_BASE develop          : codigo 0 ; modo=selectiva es_completa=True ; grupos=7 ; cacheados=0 ; aviso_pesada=['tw-superficie'] ; rango='develop...HEAD + cambios sin commit'
  RAMA_BASE vacia            : codigo 0 ; modo=selectiva es_completa=True ; grupos=7 ; cacheados=0 ; aviso_pesada=[] ; rango=''
  RAMA_BASE main (no existe) : codigo 0 ; modo=selectiva es_completa=True ; grupos=7 ; cacheados=0 ; aviso_pesada=[] ; rango=''
grupos de la completa iguales en las 4 variantes: True ; campos que cambian: ['aviso_pesada', 'ficheros', 'muestra', 'rango'] ; .cmd igual salvo el aviso: True
RESULTADO E grupos iguales=True ; cambia solo=['aviso_pesada', 'ficheros', 'muestra', 'rango']
```

Lectura: con la caché vacía, lo que corre la completa de ToolsaWeb no
depende de `RAMA_BASE`; solo cambia el aviso de que el cambio toca crates con
tests pesados, que no decide el código de salida. Cumple la condición de
F1-7 para cachearla por árbol. (`modo=selectiva` con `es_completa=True` es
como el clasificador escribe la completa con plan por grupos desde TASK-344.)

### Divergencia con el plan

1. **Práctica 1**: el plan dice «`estado: planificada`» en todas las
   archivadas; TASK-097 de ToolsaWeb está en `en-diseno`. Las 11 llevan la
   etiqueta `descartada`, así que D1a no cambia (11 de 11).
2. **Citas reajustadas** a la línea exacta del commit fijo (p. ej. «la
   transición sigue y avisa» está en la línea 38 de `TASK-068/tarea.md`, no
   en 35-39; el aviso de comillas de OpenDockTrading, en 5-6). El inventario
   lleva las que la PoC comprueba.
3. **PoC C con 7 transiciones**, no 5: las 5 del plan más dos hostiles en
   ejecución (un `comprueba` que falla y un derivado que escribe en el
   código).
4. **PoC D con los dos pilotos**: el plan solo pedía `lector-tareas.py`.
5. **B mide también lo que la traducción necesita y hoy no existe**: una
   orden de huella local en ToolsaWeb y la aceptación de `--perfil` en su
   `verificar.bat`. Lo hereda F5-1 (ver «Consecuencias» y «Preguntas
   abiertas»).
6. **OpenDockTrading no baja de claves** (2 → 2); la ventaja en claves es de
   ToolsaWeb (10 → 2). El plan hablaba de ventaja en general.
7. **Nombre propuesto `taskctl derivados`** para volver a ejecutar los
   derivados tras un fallo (el aviso tiene que nombrar un comando, como hoy
   `taskctl roadmap --escribir` en OpenDockTrading). El plan no lo nombraba.

### No verificado

- **`taskctl tareas --json` real**: la PoC D lo simula; la forma exacta de la
  salida la fija F2-7 o F3 (campos mínimos: `id`, `estado`, `etiquetas`,
  `dependencias`, `sprint`).
- **El check `estado` en GitHub** (D3a): diseño, sin workflow ni PR real.
- **La concurrencia de derivados** en la rama de estado (dos clones en
  transición a la vez, un derivado que no termina): sin rol de riesgos; el
  reintento por rebase es de F1-2, y lo generado se **regenera** tras el
  rebase, como el `ROADMAP.md` (F1-1).
- **La caché de la completa de ToolsaWeb con `RAMA_BASE`**: E la prueba con
  la caché vacía; si su clave de caché incluye `RAMA_BASE` (lo dice el
  docstring de `cache-verificacion.py` para los pasos de documentación), un
  cambio de base solo daría un fallo de caché, que es seguro.
- **El shell de los derivados**: la PoC los lanza con `cmd /d /c` en Windows;
  en Linux y macOS no se ha ejecutado (H16).
- **`taskctl doctor`** no se ejecutó en esta implementación (la tarea pidió no
  lanzar `taskctl`); queda para el cierre.

## Cómo se ve en GitHub

No hay PoC en GitHub. Lo que se deduce de F1-1: los commits de los derivados
van en `taskcode/tareas`, una rama sin `.github/`, así que no disparan
Actions; se leen como cualquier fichero de esa rama
(`.../blob/taskcode/tareas/<fichero>`). El check `estado` de D3a es un job del
workflow de TaskCode en el PR (F5-1), no cacheado, que lee la rama de estado.

## Decisión

1. **Tres claves para F2-7** (D4a), en `.taskcode/config.yml`, bajo
   CODEOWNERS y **leídas de la base** como las de F1-10; YAML estándar
   (F1-9); ante una clave o un valor que no reconoce, **falla cerrado**. En
   `tarea.md`, en cambio, una clave desconocida se conserva (F1-9 D4a): son
   dos ficheros con dos reglas.

   <!-- esquema:inicio -->
   | Clave | Tipo | Qué declara | Validación (falla cerrado) |
   |---|---|---|---|
   | `verificacion` | mapa | el contrato de F1-7 | claves conocidas solamente |
   | `verificacion.comando` | orden | el comando del proyecto; `taskctl` y el workflow lo llaman con `--perfil <p>` | texto no vacío, sin caracteres de control |
   | `verificacion.shell` | `cmd`, `bash`, `pwsh` o `sh` | con qué shell se lanza en todas partes (H16); el workflow no la reinterpreta | uno de los cuatro |
   | `verificacion.perfiles.<p>.timeout` | segundos | tiempo máximo del perfil `rapida`, `impacto`, `completa` o `pesada`; sustituye a `timeout_*` y `presupuesto_freno_s` | entero de 1 a 21600; `completa` obligatoria; ningún otro perfil |
   | `verificacion.perfiles.<p>.runner` | `github-windows`, `github-linux`, `github-macos` o `[self-hosted, ...]` | runner de `completa` y `pesada` (F1-7 dec. 5); `rapida` e `impacto` corren en local y no lo llevan | obligatorio en `completa` y `pesada`; prohibido en las otras |
   | `verificacion.huella_arbol` | globs | ficheros versionados que entran en la clave de caché (F1-7 dec. 2) | sin negación, rutas absolutas ni `..` |
   | `verificacion.huella_local` | orden | imprime lo que la suite lee fuera del árbol en el runner | como `comando` |
   | `verificacion.programada.comando` | orden | pasos que dependen de la red, en un job programado fuera del check cacheado | como `comando` |
   | `verificacion.programada.cron` | cron | cuándo corre | expresión de 5 campos |
   | `verificacion.programada.runner` | como `perfiles.<p>.runner` | dónde corre | como `perfiles.<p>.runner` |
   | `derivados` | lista | comandos que `taskctl` ejecuta en las transiciones | lista de mapas |
   | `derivados[].comando` | orden | qué ejecuta | como `verificacion.comando` |
   | `derivados[].tipo` | `genera` o `comprueba` | si escribe ficheros o solo comprueba | uno de los dos |
   | `derivados[].escribe` | rutas | lo único que un `genera` puede escribir, siempre bajo `tareas/` | obligatoria en `genera`, prohibida en `comprueba`; bajo `tareas/`, sin `*` ni `..`; nunca `tareas/ROADMAP.md` ni `tareas/TASK-NNN/` |
   | `derivados[].fases` | lista | en qué transiciones corre (por defecto, en todas) | subconjunto no vacío de las fases de `taskctl` |
   | `derivados[].timeout` | segundos | tiempo máximo (por defecto 60, como `timeout_sincronizacion`) | entero de 1 a 600 |
   | `confianza_incluye` | globs | ficheros que la huella de confianza suma al cierre de cada orden (F1-8 dec. 5) | sin negación, rutas absolutas ni `..`, y nunca todo el árbol (`**`, `*`) |
   <!-- esquema:fin -->

   Las rutas de Windows se escriben **entre comillas simples o con `/`**; una
   orden con un carácter de control (el `\v` de `"scripts\verificar.bat"`)
   es un error con ese arreglo en el mensaje. Ejemplo (la traducción de
   ToolsaWeb que valida la PoC B):

   ```yaml
   verificacion:
     comando: 'scripts\verificar.bat'
     shell: 'cmd'
     perfiles:
       rapida:   {timeout: 180}
       impacto:  {timeout: 1800}
       completa: {timeout: 4800, runner: [self-hosted, windows, toolsaweb]}
       pesada:   {timeout: 4800, runner: [self-hosted, windows, toolsaweb]}
     huella_arbol: ['Cargo.lock', 'web/package-lock.json']
     huella_local: 'python -I scripts/huella-local.py'
   derivados:
     - comando: 'python -I scripts/grafo-tareas.py --comprobar'
       tipo: 'comprueba'
       fases: ['plan', 'approve', 'finish']
   confianza_incluye: ['scripts/cache-verificacion.py', 'scripts/memo_ficheros.py']
   ```

2. **Confianza.** Toda orden de estas claves es **entrada no confiable** y
   pasa por `confianza(repo, clave)` de F1-8 (dec. 5) antes de ejecutarse,
   con `clave` = la ruta de la clave (`verificacion.comando`,
   `derivados[0]`…). `cambiado` o `nunca`: no se ejecuta y el aviso nombra
   `taskctl confiar <clave>` (F0-2). `confianza_incluye` es la única forma de
   sumar ficheros a la huella (lo que se carga por nombre de módulo, como el
   `lector-tareas.py` de OpenDockTrading o el `cache-verificacion.py` de
   ToolsaWeb).

3. **Los derivados los ejecuta `taskctl` en la transición**: un punto
   explícito y determinista del comando que la persona o el agente lanzan,
   **no un hook**. Orden: escribe la tarea, ejecuta los derivados de esa fase
   en el worktree principal con `TASKCODE_TAREAS` (la carpeta montada de la
   rama de estado) y `TASKCODE_CODIGO` (el árbol de código), y commitea la
   tarea y lo generado en **un solo commit** de `taskcode/tareas`. Si un
   derivado falla, no está confiado o escribe fuera de `escribe` (o un
   `comprueba` escribe algo), lo suyo se descarta, **la transición sigue** y
   el aviso dice qué pasó y qué comando lo arregla (`taskctl derivados` para
   volver a ejecutarlos; `taskctl confiar <clave>`), como hoy la 0.16 con el
   roadmap. `doctor` (F3-8) repite el aviso mientras siga pendiente.

4. **Lectura del estado**: `taskctl tareas --json` es la **API estable** para
   los scripts del proyecto; la forma de las carpetas de `tareas/` no es
   contrato (la PoC D muestra por qué: el glob de hoy encuentra 0 tareas en la
   carpeta fija, sin error).

5. **Sin estados propios** (D1a): `descartada` (con motivo) y `pausa` nativos
   (F3-7). La migración (F1-12) lleva cada tarea de `05-archivadas` a
   `descartada`.

6. **Revisoras propias** (D5a): una skill del proyecto con el mismo bloque
   ```` ```yaml ```` que ya lee el plugin (`rol: revisor` y
   `patrones_archivo`), sin clave en la configuración. Dónde se buscan y cómo
   se eligen lo decide F1-15.

7. **Códigos** (D6a): cualquier código distinto de 0 es rojo, también los 3
   y 4 del cerrojo de ToolsaWeb; el cerrojo sigue dentro del comando del
   proyecto.

8. **Principio: comandos sí, hooks solo como atajo opcional.** Ningún punto
   de extensión depende de un hook. El único hook previsto en la 2.0 (F5-6,
   la `rapida` en local) es un atajo opcional que llama a
   `verificacion.comando --perfil rapida`, con confianza; quitarlo no cambia
   ninguna garantía, porque las puertas son la transición, el check
   `verificacion` y el check `estado`. Las guardas que un proyecto ponga en
   `.claude/` (práctica 14) son suyas y la 2.0 no cuenta con ellas.

### Ventaja medida

Cifras de la PoC; el guion falla si alguna de esta tabla no es la medida.

<!-- ventaja:inicio -->
| Cifra | Valor | Fuente |
|---|---|---|
| practicas | `14` | A |
| practicas a mano en la 0.16 | `8` | A |
| practicas con destino declarado en la 2.0 | `13` | A |
| citas comprobadas | `43/43` | A |
| casos de 05-archivadas cubiertos por descartada | `11/11` | A |
| ToolsaWeb: archivadas con estado distinto de descartada | `9/9` | A |
| OpenDockTrading: archivadas con estado distinto de descartada | `2/2` | A |
| claves de la 0.16 en el catalogo | `32` | B |
| claves de la 0.16 de verificacion y sincronizacion | `12` | B |
| ToolsaWeb: claves de verificacion y sincronizacion 0.16 -> 2.0 | `10 -> 2` | B |
| OpenDockTrading: claves de verificacion y sincronizacion 0.16 -> 2.0 | `2 -> 2` | B |
| claves de primer nivel de F1-10 y F1-11 frente al tope de 12 | `5 de 12` | B |
| configuraciones hostiles rechazadas | `13/13` | B |
| ordenes de la traduccion que no existen hoy | `1` | B |
| argumentos de perfil que ToolsaWeb acepta hoy | `--selectiva, --pesada, --rapida` | B |
| transiciones con derivados en la rama de estado | `7` | C |
| arbol de develop tras las transiciones | `identico` | C |
| ToolsaWeb: tareas que encuentra su lector en la carpeta fija | `0` | D |
| OpenDockTrading: tareas que encuentra su lector en la carpeta fija | `0` | D |
| lineas de lectores propios que sustituye tareas --json | `82 -> 3` | D |
| pasos de verificacion que desaparecen (OpenDockTrading) | `1` | D |
| grupos de la completa de ToolsaWeb segun RAMA_BASE | `iguales` | E |
<!-- ventaja:fin -->

En resumen: **de 8 prácticas a mano se pasa a 0** dentro de TaskCode (la que
queda, las guardas de `.claude/`, está fuera por decisión); las 12 claves de
verificación y sincronización de la 0.16 caben en **2 bloques**, y con F1-10
el esquema usa **5 de las 12** claves; los lectores propios de frontmatter
(**82 líneas** en los dos pilotos) se sustituyen por un consumidor de **3**
líneas de `taskctl tareas --json`; y un paso de la verificación de
OpenDockTrading (`taskctl roadmap --comparar`) desaparece. El coste: una
orden de huella local nueva en ToolsaWeb y la adaptación de su
`verificar.bat` a `--perfil`.

## Requisitos y evidencia

| Requisito | Decisión | Evidencia |
|---|---|---|
| P8 / RN-04 extensiones declaradas en vez de convenciones a mano | tres claves, derivados, `tareas --json`, skill revisora (decisiones 1, 3, 4 y 6) | PoC A: 14 prácticas, 13 con destino declarado, 43/43 citas; 8 a mano hoy |
| RF-70 regenerar derivados en cada transición | `derivados` que ejecuta `taskctl` en la transición, en la rama de estado (decisión 3); **no** un job de CI ni un hook, como sugería RF-70 | PoC C: 7 transiciones, lo generado en el commit de la transición, avisos con el comando que lo arregla |
| RR-11 roadmap sin interferir con la verificación | los derivados solo escriben bajo `tareas/` (D2a) | PoC C: árbol de `develop` idéntico, 1 → 1 commits; PoC B: `docs/planificacion/hitos.md` rechazado |
| RF-71 estados propios / RF-07 descartar con motivo / RF-08 pausar | sin clave de estados (D1a): `descartada` y `pausa` nativos (F3-7) | PoC A: 11/11 archivadas llevan `descartada`; 11/11 con un `estado` incoherente hoy; PoC B: `estados` rechazada |
| RF-40 / RF-41 comando del proyecto con perfiles y timeout | `verificacion.comando` + `perfiles.<p>.timeout` (decisión 1) | PoC B: 10 claves de ToolsaWeb → 2 bloques; perfil fuera de F1-7 rechazado |
| RF-42 no repetir sobre el mismo árbol | `huella_arbol` y `huella_local` (F1-7 dec. 2); D3a saca lo que depende del estado del perfil cacheado; `completa` de ToolsaWeb independiente de `RAMA_BASE` | PoC E: 7 grupos en las 4 variantes; PoC A, práctica 3 |
| RF-72 skills instalables sin referencias internas | revisoras propias como skill del proyecto con el bloque que ya lee el plugin (D5a) | PoC A: 0 revisoras propias hoy; `revisores.ts:113-116` |
| H2 ejecución de código desde la config | toda orden pasa por `confianza` (decisión 2) | PoC B: nunca → confiado → cambiado → confiado; PoC C: T4 no ejecuta el generador cambiado |
| H16 shell distinto según quién ejecuta | `verificacion.shell` declarado y usado en todas partes | diseño; Linux y macOS no verificados |

## Consecuencias

- **F2-7** suma `verificacion`, `derivados` y `confianza_incluye` al esquema
  (3 de las 12 claves; con F1-10, 5) con el validador de la decisión 1, y
  fija los nombres finales de las fases de `derivados[].fases`.
- **F0-2** implementa `confianza` con `confianza_incluye` y `taskctl confiar
  <clave>`; **F5-6** (hook) la usa sin otra propia y solo llama a
  `verificacion.comando --perfil rapida`.
- **F3-4** (roadmap) genera el Mermaid por hito que hoy hace
  `grafo-tareas.py` (90 líneas de ToolsaWeb solapan) y decide si adopta su
  orden topológico; **F3-7** da `descartada` con motivo y `pausa`; **F3-8**
  (`doctor`) avisa de derivados pendientes, de órdenes sin confianza y de
  una configuración inválida en la base.
- **F2/F3** añaden `taskctl tareas --json` (lectura estable) y `taskctl
  derivados` (volver a ejecutar los derivados de la fase actual).
- **F5-1** llama a `verificacion.comando --perfil <p>` con `shell`, crea el
  job programado y el check `estado` (D3a). Para ToolsaWeb exige dos cambios
  que hoy no existen: `--perfil` en `verificar.bat` (hoy acepta `--rapida`,
  `--selectiva` y `--pesada`; otro argumento sale con 2) y una orden de huella
  local. El criterio de F5-1 «sin scripts nuevos» queda en duda para
  ToolsaWeb (ver «Preguntas abiertas»).
- **F5-4** recoge el recuento «# tests N» en el informe JSON.
- **F1-12** migra `05-archivadas` a `descartada`, traduce las claves planas
  al esquema nuevo y avisa de los lectores que usan el glob
  `0?-*/TASK-*/tarea.md` (PoC D: 0 tareas sin error).
- **F1-15** decide dónde se buscan las skills revisoras del proyecto y cómo
  se eligen; este ADR solo fija el formato.
- **Este repo no cambia** (RR-14).

## Fuera de alcance

- Las guardas del arnés en `.claude/` (permisos `deny` y `ask`, hooks de
  Claude Code): no son extensión de TaskCode. La 2.0 no depende de ellas; lo
  que protegen lo imponen `taskctl` y las puertas de GitHub (F1-3, F1-5).
- Las claves base de F2-7 (`rama_base`, `modo_flujo`, `tope_rondas`…): el
  presupuesto de este ADR solo cuenta F1-10 y F1-11.
- Selección de skills y revisoras: F1-15. Formato del informe JSON: F5-4.
  Protocolo de escritura en la rama de estado: F1-2. Migración: F1-12.

## Preguntas abiertas

Este ADR se planificó con un solo rol (arquitectura); quedan sin cubrir:

- **Riesgos de los derivados**: concurrencia en la rama de estado, un
  derivado que no termina antes del `timeout`, un `comprueba` que queda viejo
  entre transiciones (lo pilla la siguiente o el check `estado`).
- **Dominio**: si las tablas a mano de `hitos.md` §1 de ToolsaWeb, que viven
  en `docs/` (rama de código), quedan bien como documento manual más un
  `comprueba`, o tienen que ir a la rama de estado.
- **ToolsaWeb y F5-1**: si la orden de huella local y `--perfil` se resuelven
  con un subcomando de sus scripts actuales (`cache-verificacion.py` ya
  calcula la clave) o con un fichero nuevo.
- **RF-43** (`rutas_documentacion`): ningún piloto la usa en su
  configuración (PoC B lista sus claves) y este esquema no le da clave; si
  hace falta, F5-3 o F2-7.
- **Criterio 1 del plan**: no se revisaron `docs/` de OpenDockTrading ni
  hooks de Git de los pilotos (en ninguno de los dos se vio uno).

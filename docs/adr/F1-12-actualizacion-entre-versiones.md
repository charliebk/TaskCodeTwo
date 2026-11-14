# ADR F1-12 — Actualización de proyectos entre versiones: plan de migración como dato, rama de estado con su historia, criterio de salida con vuelta atrás, convivencia 0.16/2.0 y nombres definitivos

- Estado: **aceptada** (2026-11-14)
- Tarea: TASK-013 · Plan: [F1-12 en PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) (tabla de la Fase 1)
- Requisitos: la fila F1-12 cita RR-13; este ADR responde también a RR-14 y RR-16 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md), a RF-07 y RF-71 (archivadas a `descartada`) y a las filas F9-1, F9-2, F9-10, F2-14 y F4-5 del plan
- Depende de: [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (carpeta fija y rama de estado huérfana), [ADR F1-9](F1-9-formato-de-tarea-plan-y-especificacion.md) (formato 2 y tabla de campos) y [ADR F1-11](F1-11-extensiones-declaradas.md) (inventario, clasificación parche/propio, bloques de la configuración); usa [ADR F1-5](F1-5-puertas-en-github.md) (protección de `develop` y de la rama de estado)
- Decide: Carlos, con sus decisiones del 2026-11-14 (fidelidad a los datos, ventajas medibles y comandos deterministas; nombres propios y definitivos; estudio de adopción antes de migrar; retirada de los parches que la 2.0 cubre; orden y condiciones de la desinstalación; OpenDockTrading como primera migración; no trocear) y la consigna «la opción recomendada» (D1a, D2a, D3a y D4a del plan de TASK-013). D2 queda resuelta por la PoC dentro de la opción (a) (ver «Decisión», punto 6)

## Contexto

RR-13 pide «actualizar los proyectos existentes a la versión actual»: un
comando que migra la estructura, la config y las tareas, con simulación
previa, idempotente y reversible, primero ToolsaWeb y OpenDockTrading de la
0.16 a la 2.0. Tres ADR ya fijan el destino:

- **F1-1**: cada tarea es la carpeta fija `TASK-NNN/` en una rama huérfana
  montada como worktree en `tareas/`, con el estado como campo.
- **F1-9**: `tarea.md` en formato 2 (YAML estándar, `formato: 2`); sin
  `formato` es una tarea de la 0.16, que la 2.0 lee para migrarla. Dejó a
  este ADR la migración campo a campo y las archivadas a `descartada` con
  `motivo`.
- **F1-11**: el inventario de lo que los pilotos hacen a mano, con su
  clasificación **parche** (se retira al migrar) o **propio** (se queda), las
  líneas de cada clase, y los tres bloques de la configuración.

Lo que falta decidir: cómo se migra sin perder datos, cómo se sabe que la
migración salió bien y cómo se deshace; qué pasa mientras las dos versiones
están instaladas a la vez en la misma máquina (la 0.16 no se puede cambiar
desde este repo); y con qué nombres convive la 2.0 con la 0.16 y se queda
cuando la 0.16 se desinstala. La persona fijó el criterio: fidelidad a los
datos por encima del coste, ventajas que se puedan medir y **comandos
deterministas** (ningún paso de la actualización depende de un hook).

## Alternativas

| | Descripción | Veredicto |
|---|---|---|
| **PL1** | **El plan de migración es un dato**: `planificar(proyecto)` es una función de lectura que devuelve el plan (JSON) y su hash; `--simular` la muestra; `actualizar` la vuelve a calcular, compara el hash y aplica exactamente ese plan | **Elegida**: la simulación no cambia ningún byte, el plan sale igual en dos llamadas y tras una vuelta atrás, y con `formato: 2` sale vacío (PoC M) |
| PL2 | Guion imperativo que migra y escribe un informe al final | Descartada: la simulación sería otro camino de código que puede divergir del real; no hay idempotencia por construcción |
| **HI1** | **Historia fiel**: `git subtree split -P tareas` (commits reescritos, sin antepasado común con `develop`) y encima dos commits, el aplanado (solo renombres) y el formato 2, con el trailer `Migrado-desde: <sha>` | **Elegida**: 96 de 96 tareas con todos sus commits y `git log --follow` igual o mejor que en `develop` (PoC M) |
| HI2 | Instantánea: un commit con el árbol de `tareas/` ya migrado y el trailer | Descartada: más barata, pero `git log` de una tarea empieza en la migración (pierde su historia) |
| HI3 | `git filter-repo` | Descartada: dependencia externa; RR-16 deja `git` como única dependencia |
| **TR1** | **Transacción local**: ref de la rama de estado, rama `actualizar/...` y worktree en local; criterio de salida; solo con el criterio en verde, push | **Elegida**: con el criterio roto la vuelta atrás deja la copia como estaba y el remoto sin tocar (PoC M2) |
| TR2 | Push y, si algo falla, revertir en el remoto | Descartada: F1-5 prohíbe borrar o forzar la rama de estado y `develop` solo cambia por PR |
| **CR1** | **Criterio estructural**: instantánea `{id → título, estado, dependencias, sprint, etiquetas}` antes y después, roadmap por estructura, `merge-base` vacío e historia por tarea | **Elegida**: detecta una sola dependencia borrada (PoC M2) |
| CR2 | Comparar el `ROADMAP.md` byte a byte | Descartada: la 2.0 lo regenera con otro contenido (F1-1, decisión 3) |
| **D1a** | Nombres `tc2` (CLI) y `taskcodetwo` (plugin, prefijo de skills, `.git/taskcodetwo/`, rama `taskcodetwo/tareas`, paquete npm) | **Elegida** (consigna «la opción recomendada»; aprobados por Carlos el 2026-11-14): `tc2` no está en el PATH; el paquete npm `tc2` está ocupado, `taskcodetwo` libre (PoC N) |
| D1b | `tctwo` | Descartada: libre en todo, pero no se lee como «TaskCode» y no gana nada frente a `tc2` |
| D1c | `taskcodetwo` como CLI | Descartada: largo para escribirlo a diario |
| D1d | `tc` | Descartada: `tc` es la orden de control de tráfico de iproute2 en Linux (no verificado en una distro con iproute2, ver «No verificado») y el paquete npm `tc` instala un binario `tc` (PoC N) |
| **D2a** | Bloquear a la 0.16 con un fallo cerrado que ya tiene, medido | **Elegida**: su lector de config aborta con una línea sin `:`; la config de la 2.0 empieza por `---` y toda orden de escritura de la 0.16 sale con 1 sin escribir (PoC C) |
| D2b | Publicar una 0.16.2 que respete `formato` | Descartada por innecesaria: la PoC encontró el fallo cerrado |
| D2c | Solo avisar con `doctor` de la 2.0 | Descartada: la 0.16 sin frenar crea `TASK-001` duplicada en un proyecto migrado (PoC C, control A) |
| **D4a** | La misma `.taskcode/config.yml` con `formato: 2` | **Elegida**: una sola config por proyecto, y el bloqueo se ata a ella |
| D4b | `.taskcodetwo/config.yml` aparte | Descartada: dos configs que pueden decir cosas distintas y la 0.16 seguiría leyendo la suya sin frenar |

## Pruebas de concepto

Guion `poc-f1-12.py` y su salida `poc-f1-12.log`, en la carpeta `poc/` de
TASK-013 (sin enlace: la carpeta de la tarea cambia de estado al cerrarse).
Python 3.12.10 (`python -I -B`), PyYAML 6.0.3, Git 2.56.0 y Node 22.23.4,
con la carpeta de trabajo en
`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-013\poc\` (borrada al
terminar). OpenDockTrading **solo se lee**: se clona con
`git clone --no-hardlinks` en la carpeta de trabajo, se fija en `develop`
`8141a08` (2026-11-14) y todo se ejecuta sobre esas copias; el remoto es un
repo vacío local. El `taskctl` 0.16.1 y sus hooks solo se lanzan con el cwd
en la copia; nada se instala ni se toca en `~/.claude`. GitHub no se usa;
npm, Homebrew y winget se consultan en solo lectura.

```bash
python -I -B tareas/<estado>/TASK-013/poc/poc-f1-12.py \
  "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-013/poc/trabajo" . .. \
  ~/.claude/plugins/cache/taskcode-marketplace/taskcode-plugin/0.16.1 > poc-f1-12.log
```

El guion lee la tabla `cifras` de este ADR y sale con 1 si una cifra no es
la medida, si falta una medida o si sobra una fila; también comprueba el
`resumen`, las secciones, los IDs citados, los enlaces y la fila del índice.
Ocho copias mutadas del ADR (una cifra cambiada, una fila borrada, una fila
sin medida, el tiempo de reloj metido en la tabla, un número inventado en
el resumen, un enlace a `tareas/`, un ID que no existe y una sección
borrada) tienen que fallar, cada una por su motivo. El tiempo de reloj no es
una cifra comprobada: va a stderr. Salida observada, recortada (la completa
está en el log):

### N — Nombres

```text
PATH Windows (where tc): ninguna
PATH Windows (where tc2): ninguna
PATH Windows (where taskctl): 2 (bin/ del plugin 0.16.1)
PATH Linux, WSL docker-desktop (command -v tc): ninguna
tc: npm=ocupado (bin tc) ; Homebrew=libre ; winget=libre
tc2: npm=ocupado (sin bin) ; Homebrew=libre ; winget=libre
tctwo: npm=libre ; Homebrew=libre ; winget=libre
taskcodetwo: npm=libre ; Homebrew=libre ; winget=libre
```

La única distro de WSL de esta máquina es `docker-desktop`, mínima, sin
iproute2: no sirve para confirmar el choque de `tc` en una distro de uso
normal. En ella `taskctl` «existe» porque WSL añade el PATH de Windows.

### M — Migración de la copia de OpenDockTrading

```text
commit fijo 8141a08a7: 96 tareas ; por carpeta 00..05: 50 / 0 / 0 / 0 / 44 / 2
--simular: segunda llamada igual ; ficheros cambiados en la copia (con .git): 0
plan: tareas a mover=96 ; a descartada=2 (TASK-015, TASK-068) ; con motivo de ## Descartada=2 ; claves retiradas=192
actualizar sin capacidad tareas-json: codigo 5 ; huecos bloqueantes: scripts/lector-tareas.py (1-44) ...
   ficheros cambiados por la negativa (con .git): 0
subtree split: 1151 commits ; rama de estado tras aplanar y formato 2: 1153 commits
criterio: tareas 96 de 96 ; frontmatter no YAML 0 ; con formato 2 96 ; diferencias 0 ; roadmap iguales (94 filas) ; merge-base ninguna
historia por tarea: commits por ruta >= develop 96/96 ; git log --follow >= el de develop 96/96 ;
   --follow ya roto en develop (la 0.16 mueve y reescribe en el mismo commit): 3 ['TASK-019 1 de 11', 'TASK-020 1 de 11', 'TASK-080 5 de 6']
remoto: refs antes del push 1 ; tras el push 3
ficheros de codigo cambiados por la migracion (fuera de tareas/): ['.gitignore', '.taskcode/config.yml']
lector-tareas.py de OpenDockTrading tras migrar: ve 0 de 94
segunda pasada: formato de origen 2 ; pasos 0 ; codigo 0 ; ficheros cambiados 0
```

Lectura: el plan es puro (0 ficheros cambiados, mismo hash) y la segunda
pasada no hace nada. La migración conserva las 96 tareas, sus dependencias y
su historia; el roadmap de la 0.16 sigue casando fila a fila con las tareas
migradas. `git log --follow` ya estaba roto **en `develop`** para 3 tareas,
porque la 0.16 mueve la carpeta y reescribe `tarea.md` en el mismo commit
(similitud bajo el 50 %); en la rama de estado no empeora y, por ruta, no
falta ningún commit. Por eso el aplanado va en un commit de solo renombres,
separado del formato 2. Sin la capacidad `tareas-json` la migración se niega
(código 5) sin tocar nada: el lector propio de OpenDockTrading vería 0 de 94
tareas tras migrar. La PoC fuerza la migración para medir el resto.

El `subtree split` recorre los 1350 commits de `develop` y deja 1151 en la
rama de estado. Es el paso caro de la migración y su tiempo de reloj varía
entre ejecuciones en la misma máquina, así que no entra en la tabla
comprobada. Medidas con respaldo: «menos de 600 s» en la entrega (el log del
commit 7f431ff, línea 29, solo guardó ese cubo), 614 s en la reejecución
de la revisión 1 y 647 s en la ejecución que deja el log de esta
corrección (su stderr va junto al log, en `poc-f1-12.tiempos.log`).

### M2 — Vuelta atrás con el criterio roto, sobre una migración a medias

```text
estado real recalculado: {"marcador": true, "rama_estado": true, "rama_estado_reutilizable": true}
actualizar con una dependencia borrada en TASK-002: codigo 6 ; diferencias ['TASK-002: dependencias']
tras la vuelta atras: rama develop ; HEAD = SHA previo ; refs nuevas [] ; worktrees 1 ; git status limpio ;
   arbol de trabajo identico ; .git/taskcodetwo no existe ; remoto sin cambios
plan recalculado tras la vuelta atras: igual al inicial
```

Una segunda copia simula una migración interrumpida (rama de estado ya
creada y marcador). `planificar` lo detecta y reutiliza la rama porque su
árbol es el de `tareas/` en la base; se borra una dependencia antes de
evaluar; el criterio lo ve, la vuelta atrás es automática y local, y el plan
recalculado es el inicial.

### C — Convivencia: la 0.16 sobre la copia migrada

Con la copia ya migrada (`develop` con el PR dentro y `tareas/` montada), el
guion lanza `board`, `doctor`, `estado --stdin`, `roadmap --escribir` y
`new --tipo feature` de la 0.16.1 y sus hooks `pre-tool-merge` (con un
`git merge --no-ff` a `develop`), `pre-tool-lectura`, `stop-verificacion` y
`coste-turno` con su JSON por stdin, con cuatro variantes de la config
formato 2 (todas YAML estándar válido). Tras cada orden compara el hash de
cada fichero de la copia, `.git` incluido, y la restaura.

| Orden de la 0.16 | B: con `---` (la que escribe `actualizar`) | A: control, sin `---` | C: listas en bloque | D: `modo_flujo` entre comillas simples |
|---|---|---|---|---|
| `board` | 0, «No hay tareas» | 0, «No hay tareas» | 0 | 0 |
| `doctor` | 1: `config`, `rama-principal`, `tareas-carpetas` | 1: `rama-principal`, `tareas-carpetas` y 18 avisos de claves desconocidas | 1, como B | 1, como B |
| `estado --stdin` | 0, «Nada en marcha» | 0 | 0 | 0 |
| `roadmap --escribir` | **1, no escribe** (el ROADMAP sigue con 94 filas de tarea) | **0, reescribe `tareas/ROADMAP.md`: de 94 filas de tarea a 0** | 1, no escribe | 1, no escribe |
| `new` | **1, no escribe** (`Linea de config invalida (falta ":"): "---"`) | **1, pero escribe `tareas/00-planificadas/TASK-001/tarea.md`**: un `TASK-001` que ya existe en la carpeta fija | 1, no escribe | 1, no escribe |
| hook `pre-tool-merge` | **2, bloquea** (la config no se puede leer) | 0, deja pasar | 2 | 2 |
| hooks `pre-tool-lectura`, `stop-verificacion`, `coste-turno` | 0, sin escribir nada | 0 | 0 | 0 |

```text
.git/taskcode tras la convivencia: [] ; .git/taskcodetwo: ['actualizar'] ; compartidos: 0
D2, variante B: ordenes de escritura de la 0.16 cerradas=True ; hooks que dejan pasar: ['hook stop-verificacion']
D2, variante A: ordenes de escritura de la 0.16 cerradas=False ; hooks que dejan pasar: ['hook pre-tool-merge', 'hook stop-verificacion']
```

Lectura: el lector plano de la config de la 0.16 aborta ante una línea sin
`:` y ante un valor inválido de una clave suya (lo dice su `config.js`: un
valor inválido aborta, una clave desconocida solo avisa). Es un **fallo
cerrado que ya existe**: con `---` en la primera línea ninguna orden de
escritura de la 0.16 entra en el proyecto migrado y su guardián de merges
bloquea. Sin él, la 0.16 crea una tarea con un ID duplicado y reescribe el
roadmap de la rama de estado. Las órdenes de lectura no ven tareas y los
demás hooks son inertes. `doctor` de la 0.16 sale con 1 en todas las
variantes: no encuentra las carpetas de estado (`tareas-carpetas`) y la
copia no tiene `main` (`rama-principal`).

### A — Informe de adopción de OpenDockTrading

Lo que mostraría `tc2 actualizar --simular` en el bloque «hecho a mano»,
con los `fichero:líneas` de F1-11 recalculados en `8141a08` (los scripts no
cambian desde el commit que leyó F1-11), con la 2.0 de hoy (ninguna
capacidad: solo las que implementa el propio `actualizar`) y, tras la
barra, con la 2.0.0 prevista:

```text
parche  scripts/lector-tareas.py:1-44 (44 de 44) ; tareas-json ; queda: hueco bloqueante | retirar con sustitucion
parche  scripts/comprobar-modulos.py:_cargar_grafo 87-92, tareas_vivas 95-105 (17 de 458) ; tareas-json ; hueco bloqueante | retirar con sustitucion
parche  scripts/comprobar-fallos.py:_cargar_grafo 51-56, tareas_vivas 302-310 (15 de 470) ; tareas-json ; hueco bloqueante | retirar con sustitucion
parche  scripts/verificar.bat:3-5 (3 de 92) ; puertas-en-ci ; queda: hueco | retirar
parche  scripts/verificar.bat:16-18, 35-37, 44-47 (10 de 92) ; puertas-en-ci ; queda: hueco | retirar
parche  scripts/verificar.bat:25 (1 de 92) ; cli-2 ; queda: hueco | retirar
parche  scripts/verificar.bat:59 (1 de 92) ; roadmap-nativo ; queda: hueco | retirar
propio  scripts/comprobar-fallos.py (455), scripts/comprobar-modulos.py (441), scripts/verificar.bat (77)
sin 2.0:          lineas que se retiran=0 ; que se quedan=1064 (propias=973, huecos bloqueantes=76)
2.0.0 prevista:   lineas que se retiran=91 (solas=15, con sustitucion=76) ; que se quedan=973
-- ficheros fuera de tareas/ que nombran taskctl o TaskCode: 18 ; sin clasificar en F1-11: 15
   (CHANGELOG.md, CLAUDE.md, README.md, cinco de docs/, dos evidencias, scripts/comprobar-entorno.py:35-275,
    scripts/comprobar-secretos.py:35, scripts/resumen-verificacion.py:136, tools/odt-arquitectura/tests/verificar.rs)
```

Lectura: el **91 / 973** de F1-11 se reproduce en el commit de hoy, ahora
partido en lo que se retira solo (15 líneas de `verificar.bat`) y lo que
necesita sustituir código del proyecto (76). Sin `tareas-json` en la 2.0,
esas 76 son huecos bloqueantes. El informe encuentra además 15 ficheros que
nombran TaskCode y que F1-11 no clasificó: casi todos documentación; los
tres scripts (`comprobar-entorno.py` comprueba la versión de `taskctl`) son
material para el estudio de adopción. El uso de campos sale como en F1-9:
ocho campos de la 0.16 sin ningún valor propio en OpenDockTrading.

### Cifras de la PoC

Cada fila es una medida del guion, con la sección que la produce. El guion
falla si un valor no es el medido, si falta una medida o si sobra una fila.
Solo entran valores que no dependen del reloj. Las ejecuciones de prueba
reutilizaron el `subtree split` de una caché fuera de la copia; la del log,
no.

<!-- cifras:inicio -->
| Cifra | Valor | Fuente |
|---|---|---|
| Homebrew: taskcodetwo | `libre` | N |
| Homebrew: tc | `libre` | N |
| Homebrew: tc2 | `libre` | N |
| Homebrew: tctwo | `libre` | N |
| PATH Linux (WSL docker-desktop): taskcodetwo | `ninguna` | N |
| PATH Linux (WSL docker-desktop): taskctl | `existe (PATH de Windows por la interoperabilidad de WSL)` | N |
| PATH Linux (WSL docker-desktop): tc | `ninguna` | N |
| PATH Linux (WSL docker-desktop): tc2 | `ninguna` | N |
| PATH Linux (WSL docker-desktop): tctwo | `ninguna` | N |
| PATH Windows: taskcodetwo | `ninguna` | N |
| PATH Windows: taskctl | `2 (bin/ del plugin 0.16.1)` | N |
| PATH Windows: tc | `ninguna` | N |
| PATH Windows: tc2 | `ninguna` | N |
| PATH Windows: tctwo | `ninguna` | N |
| npm: taskcodetwo | `libre` | N |
| npm: tc | `ocupado (bin tc)` | N |
| npm: tc2 | `ocupado (sin bin)` | N |
| npm: tctwo | `libre` | N |
| winget: taskcodetwo | `libre` | N |
| winget: tc | `libre` | N |
| winget: tc2 | `libre` | N |
| winget: tctwo | `libre` | N |
| --simular: ficheros cambiados en la copia (con .git) | `0` | M |
| --simular: hash del plan en dos llamadas | `igual` | M |
| OpenDockTrading: tareas | `96` | M |
| OpenDockTrading: tareas por carpeta (00..05) | `50 / 0 / 0 / 0 / 44 / 2` | M |
| actualizar con huecos bloqueantes: codigo y ficheros cambiados | `5 y 0` | M |
| commits de develop que tocan tareas/ | `1151` | M |
| criterio: diferencias de la instantanea | `0` | M |
| criterio: frontmatter que carga PyYAML con formato 2 | `96/96` | M |
| criterio: merge-base de develop y la rama de estado | `ninguna` | M |
| criterio: roadmap por estructura | `iguales (94 filas)` | M |
| criterio: tareas con git log --follow igual o mayor que en develop | `96/96` | M |
| criterio: tareas con todos sus commits en la rama de estado | `96/96` | M |
| criterio: tareas en la rama de estado | `96 de 96` | M |
| ficheros de codigo que cambia la migracion | `.gitignore, .taskcode/config.yml` | M |
| git log --follow ya roto en develop por la 0.16 | `TASK-019 1 de 11, TASK-020 1 de 11, TASK-080 5 de 6` | M |
| lector-tareas.py con el glob de la carpeta fija | `96 de 94` | M |
| lector-tareas.py tras migrar: tareas que ve | `0 de 94` | M |
| lector-tareas.py: descartadas del formato 2 que no reconoce | `TASK-015, TASK-068` | M |
| plan: archivadas a descartada con motivo de ## Descartada | `2/2` | M |
| plan: claves retiradas (revision_codex, ultimo_commit_revisado) | `192` | M |
| plan: tareas cuyo estado del frontmatter no casaba con su carpeta | `2` | M |
| precondicion: tareas en diseno, en curso o en revision | `0` | M |
| remoto: refs antes del criterio y tras el push | `1 -> 3` | M |
| segunda pasada: pasos del plan y ficheros cambiados | `0 y 0` | M |
| subtree split: commits de la rama de estado | `1151` | M |
| reanudar: rama de estado a medias reutilizable | `si` | M2 |
| vuelta atras: codigo con el criterio roto | `6` | M2 |
| vuelta atras: diferencia detectada | `TASK-002: dependencias` | M2 |
| vuelta atras: estado final | `HEAD = previo, 0 refs, 1 worktree, status limpio, remoto sin cambios` | M2 |
| vuelta atras: plan recalculado igual al inicial | `si` | M2 |
| .git/taskcode y .git/taskcodetwo: ficheros compartidos | `0` | C |
| 0.16 board, config A: codigo | `0` | C |
| 0.16 board, config B: codigo | `0` | C |
| 0.16 board, config C: codigo | `0` | C |
| 0.16 board, config D: codigo | `0` | C |
| 0.16 doctor, config A: codigo | `1` | C |
| 0.16 doctor, config A: comprobaciones en error | `rama-principal, tareas-carpetas` | C |
| 0.16 doctor, config B: codigo | `1` | C |
| 0.16 doctor, config B: comprobaciones en error | `config, rama-principal, tareas-carpetas` | C |
| 0.16 doctor, config C: codigo | `1` | C |
| 0.16 doctor, config C: comprobaciones en error | `config, rama-principal, tareas-carpetas` | C |
| 0.16 doctor, config D: codigo | `1` | C |
| 0.16 doctor, config D: comprobaciones en error | `config, rama-principal, tareas-carpetas` | C |
| 0.16 hook pre-tool-merge, config A: codigo | `0` | C |
| 0.16 hook pre-tool-merge, config B: codigo | `2` | C |
| 0.16 hook pre-tool-merge, config C: codigo | `2` | C |
| 0.16 hook pre-tool-merge, config D: codigo | `2` | C |
| 0.16 hook stop-verificacion, config A: codigo | `0` | C |
| 0.16 hook stop-verificacion, config B: codigo | `0` | C |
| 0.16 hook stop-verificacion, config C: codigo | `0` | C |
| 0.16 hook stop-verificacion, config D: codigo | `0` | C |
| 0.16 new, config A: ID duplicado creado | `TASK-001` | C |
| 0.16 new, config A: codigo | `1` | C |
| 0.16 new, config B: ID duplicado creado | `ninguno` | C |
| 0.16 new, config B: codigo | `1` | C |
| 0.16 new, config C: ID duplicado creado | `ninguno` | C |
| 0.16 new, config C: codigo | `1` | C |
| 0.16 new, config D: ID duplicado creado | `ninguno` | C |
| 0.16 new, config D: codigo | `1` | C |
| 0.16 roadmap --escribir, config A: codigo | `0` | C |
| 0.16 roadmap --escribir, config B: codigo | `1` | C |
| 0.16 roadmap --escribir, config C: codigo | `1` | C |
| 0.16 roadmap --escribir, config D: codigo | `1` | C |
| 0.16 roadmap --escribir, config A: filas de tarea del ROADMAP | `94 -> 0` | C |
| 0.16 roadmap --escribir, config B: filas de tarea del ROADMAP | `94 -> 94` | C |
| 0.16 roadmap --escribir, config C: filas de tarea del ROADMAP | `94 -> 94` | C |
| 0.16 roadmap --escribir, config D: filas de tarea del ROADMAP | `94 -> 94` | C |
| D2: ordenes de escritura de la 0.16 cerradas con la config A | `no` | C |
| D2: ordenes de escritura de la 0.16 cerradas con la config B | `si` | C |
| D2: ordenes de escritura de la 0.16 cerradas con la config C | `si` | C |
| D2: ordenes de escritura de la 0.16 cerradas con la config D | `si` | C |
| OpenDockTrading, 2.0.0 prevista: lineas que se retiran / se quedan | `91 / 973` | A |
| OpenDockTrading, 2.0.0 prevista: retiradas solas / con sustitucion | `15 / 76` | A |
| OpenDockTrading, sin 2.0: lineas que se retiran / se quedan | `0 / 1064` | A |
| OpenDockTrading, sin 2.0: retiradas solas / con sustitucion | `0 / 0` | A |
| OpenDockTrading: lineas propias que se quedan | `973` | A |
| ficheros fuera de tareas/ que nombran taskctl o TaskCode: total / sin clasificar | `18 / 15` | A |
| uso: campos de la 0.16 sin ningun valor propio en OpenDockTrading | `agente_revisor, complejidad, modelo_sugerido, regla_seleccion_skill, revision_codex, skills_recomendados, tipo, ultimo_commit_revisado` | A |
| F1-10: lineas con taskctl / taskcode/tareas / .git/taskcode | `9 / 1 / 0` | G |
| F1-11: lineas con taskctl / taskcode/tareas / .git/taskcode | `35 / 9 / 0` | G |
| F1-1: lineas con taskctl / taskcode/tareas / .git/taskcode | `1 / 16 / 0` | G |
| F1-2: lineas con taskctl / taskcode/tareas / .git/taskcode | `0 / 9 / 2` | G |
| F1-3: lineas con taskctl / taskcode/tareas / .git/taskcode | `3 / 8 / 0` | G |
| F1-4: lineas con taskctl / taskcode/tareas / .git/taskcode | `28 / 7 / 0` | G |
| F1-5: lineas con taskctl / taskcode/tareas / .git/taskcode | `1 / 13 / 0` | G |
| F1-6: lineas con taskctl / taskcode/tareas / .git/taskcode | `2 / 9 / 0` | G |
| F1-7: lineas con taskctl / taskcode/tareas / .git/taskcode | `4 / 1 / 0` | G |
| F1-8: lineas con taskctl / taskcode/tareas / .git/taskcode | `5 / 4 / 0` | G |
| F1-9: lineas con taskctl / taskcode/tareas / .git/taskcode | `0 / 1 / 0` | G |
<!-- cifras:fin -->

<!-- resumen:inicio -->
Dónde gana la 2.0 al migrar OpenDockTrading: **96 de 96** tareas con sus
dependencias, **96/96** con todos sus commits y con `git log --follow`
igual o mejor que en `develop`, **2/2** archivadas a `descartada` con su
motivo, y en el código solo cambian `.gitignore` y `.taskcode/config.yml`;
`--simular` cambia **0** ficheros y la segunda pasada, **0 y 0**; con el
criterio roto, la vuelta atrás deja la copia como estaba. Con la 2.0.0
prevista se retiran y se quedan **91 / 973** líneas. Dónde no gana o falta
algo: sin `tareas-json` la migración se niega (código y ficheros cambiados:
**5 y 0**), porque el lector propio vería **0 de 94** tareas; y el
`--follow` que la 0.16 ya rompió en `develop` no se recupera.
<!-- resumen:fin -->

## Decisión

1. **El plan de migración es un dato** (PL1). `planificar(proyecto)` lee el
   repo sin escribir nada (`git cat-file`, `ls-tree`, `status
   --no-optional-locks`) y devuelve un JSON canónico con su hash:
   - `tc2 actualizar --simular [--json]` lo muestra; es el instrumento del
     estudio de adopción (el hito de decisiones en `init` y `actualizar` es de
     TASK-094). Bloques del informe: **uso** (la tabla de campos de F1-9 con
     las tareas de cada campo fuera del valor por defecto), **hecho a mano**
     (las prácticas de F1-11 con `fichero:líneas` recalculados, su
     `cubierto_por` y la acción), **movimientos** (cada tarea `de → a`, sus
     cambios de frontmatter y el hash del `tarea.md` resultante; la config
     antes y después), **precondiciones**, **estado real** (rama de estado,
     marcador, si es reutilizable) y **lo que nombra TaskCode fuera de
     `tareas/` sin clasificar**.
   - `tc2 actualizar` vuelve a calcular el plan y sale con **3** si el hash no
     es el simulado; aplica exactamente ese plan (cada `tarea.md` escrito
     tiene que dar el hash del plan).
   - **Idempotente**: con `formato: 2` el plan sale vacío y no hace nada; una
     migración a medias se recalcula desde el estado real (la rama de estado
     se reutiliza si su árbol es el de `tareas/` en la base).
   - Códigos: 0 hecho o nada que hacer; **3** plan cambiado; **4**
     precondición (tarea en diseño, en curso o en revisión, o árbol sucio);
     **5** hueco bloqueante (punto 8); **6** criterio de salida en rojo, con la
     vuelta atrás ya hecha. Encajan en el contrato de salida de F1-4 cuando
     F2 los fije.

2. **Qué migra** (con la tabla de campos de F1-9):

   | De la 0.16 | En la 2.0 |
   |---|---|
   | `tareas/NN-estado/TASK-NNN/` | `TASK-NNN/` en la raíz de la rama de estado; el estado sale de la carpeta |
   | `05-archivadas/` | `estado: "descartada"` y `motivo` sacado del primer párrafo de `## Descartada…`; si no lo hay, `motivo: null` y el plan lo marca «motivo a pedir» |
   | `tareas/NN-estado/.gitkeep` | se borran (las carpetas de estado desaparecen) |
   | frontmatter de la 0.16 | formato 2 de F1-9: `formato: 2`, cadenas y fechas entre comillas dobles, `modelo_sugerido` → `roles.implementacion.modelo`, `skills_recomendados` y `regla_seleccion_skill` → `roles.implementacion`, `agente_revisor` → `roles.revision.agente`; fuera `revision_codex` y `ultimo_commit_revisado`; `carril` y `estimacion` a `null`; las claves desconocidas se conservan (F1-9 D4a). El cuerpo, byte a byte |
   | `tareas/ROADMAP.md` | se lleva tal cual a la raíz de la rama de estado; la 2.0 lo regenera con el contenido de F1-1 en su primera transición (F3-4) |
   | `.taskcode/config.yml` plana | la misma ruta (D4a) con `---` en la primera línea, `formato: 2`, las claves base de F2-7 y los bloques de F1-11 (`verificacion`, `derivados`, `confianza_incluye`); los valores de la 0.16 pasan tal cual (`comando_verificacion` → `verificacion.comando`, `timeout_verificacion` → `perfiles.completa.timeout`) y lo que la 0.16 no tenía lo declara el proyecto (F1-11, «valores a rellenar»). **Comillas**: las claves que la 0.16 también lee (hoy solo `modo_flujo`) van entre **comillas dobles**, porque la 0.16 quita las dobles pero conserva las simples dentro del valor (variante D de la PoC C); es el mismo criterio que F1-9 fija para las cadenas del frontmatter de `tarea.md`, aplicado aquí por analogía a `config.yml`; las órdenes y rutas de los bloques nuevos, que la 0.16 no lee, van entre **simples** o con `/`, como fija F1-11 (decisión 1), para que el `\v` de una ruta de Windows no sea un escape |
   | `.gitignore` | añade `tareas/` |
   | `.git/taskcode/` (estado interno de la 0.16) | no migra: lo borra la desinstalación de la 0.16 (F9-10) |

3. **Historia** (HI1): `git subtree split -P tareas <base>` crea la rama
   huérfana **`taskcodetwo/tareas`** con la historia de cada fichero; encima
   van **dos** commits con el trailer `Migrado-desde: <sha de la base>`: el
   aplanado (solo `git mv`, para que `--follow` los siga al 100 %) y el
   formato 2. `develop` conserva su historia intacta; los SHA de la rama de
   estado son nuevos, y el trailer dice de dónde salen.

4. **Transacción y vuelta atrás** (TR1).
   - Precondiciones: ninguna tarea en diseño, en curso ni en revisión; árbol
     limpio; plan igual al simulado; ningún hueco bloqueante.
   - Todo en local: marcador `.git/taskcodetwo/actualizar/previo.json` con la
     rama y el SHA previos; ref de la rama de estado; rama
     `actualizar/formato-2` con un commit que quita `tareas/`, la ignora y
     migra la config; worktree de la rama de estado en `tareas/`.
   - Se evalúa el criterio (punto 5). En rojo, **vuelta atrás automática y
     local**: se quita el worktree, se vuelve a la rama y al SHA previos, se
     borran las dos ramas, la caché de `git subtree` y `.git/taskcodetwo/`;
     no se ha empujado nada. Código 6.
   - En verde, push de la rama de estado (directo, F1-5) y de
     `actualizar/formato-2`, que entra en `develop` por PR (F1-5 protege
     `develop`). Hasta que entra, el clon se queda en esa rama.
   - Después del push, `tc2 actualizar --deshacer` revierte el commit de
     `develop` por PR y deja la rama de estado **congelada** con un commit que
     lo dice: F1-5 prohíbe borrarla o forzarla.

5. **Criterio de salida** (CR1), todo medible y sin juicio:
   - misma instantánea `{id → título, estado, dependencias, sprint,
     etiquetas}` desde las carpetas de la 0.16 en la base y desde la rama de
     estado (estado de la 0.16 = el de su carpeta; `05-archivadas` =
     `descartada`);
   - todos los `tarea.md` cargan con un parser YAML estándar y tienen
     `formato: 2`;
   - el `ROADMAP.md` casa **por estructura** con las tareas migradas (cada
     fila de tarea, sus dependencias y si está terminada; descartadas fuera);
   - `merge-base` de la rama de código y la rama de estado vacío (huérfana);
   - historia por tarea: en la rama de estado, al menos tantos commits por
     ruta como en `develop`, y `git log --follow` igual o mayor que en
     `develop`.

6. **Convivencia y bloqueo** (D2a, D4a).
   - **Por proyecto manda el `formato` de `.taskcode/config.yml`**: sin él,
     la 0.16; con `formato: 2`, la 2.0. La 2.0 se niega a escribir en un
     proyecto sin `formato: 2`, salvo `actualizar` y `doctor`.
   - **Fallo cerrado de la 0.16 medido** (D2a): su lector de config aborta
     con una línea sin `:` y con un valor inválido de una clave suya. La
     config que escribe `actualizar` **empieza por `---`** (marca de documento
     que un parser YAML estándar acepta): `new` y `roadmap --escribir` de la
     0.16 salen con 1 sin escribir nada y su guardián de merges bloquea
     (código 2). Sin esa línea (control A), la 0.16 solo avisa de las claves
     desconocidas y `roadmap --escribir` reescribe el `ROADMAP.md` de la rama
     de estado con 0 tareas. El validador de la 2.0 (F2-7) exige la primera
     línea `---` y `doctor` la repone si falta. D2 no necesita publicar una
     0.16.2 (b) ni se queda en un aviso (c).
   - **Lo que no frena**: `board` y `estado --stdin` de la 0.16 solo leen y
     no ven tareas («No hay tareas», «Nada en marcha»); sus hooks
     `stop-verificacion`, `coste-turno` y `pre-tool-lectura` salen con 0 sin
     escribir nada (sin `comando_verificacion` ni tarea en una carpeta de
     estado son inertes). No hace falta desactivar el plugin 0.16 en un
     proyecto migrado; la desinstalación es la de F9-10.
   - **`doctor` de la 2.0** detecta las dos instalaciones (caché
     `taskcode-plugin` y `taskctl` en el PATH) y dice cuál manda en el
     proyecto según `formato`; avisa de estructura mixta (carpetas
     `NN-estado/` dentro de la rama de estado) y de una config formato 2 sin
     `---`.
   - **Estado bajo `.git`**: la 0.16 usa `.git/taskcode/`, la 2.0
     `.git/taskcodetwo/`; no comparten ningún fichero (PoC C).
   - **Status line**: una sola ranura en el `settings.json` del usuario.
     Mientras la 0.16 esté instalada la 2.0 no la toma; `tc2 doctor` lo dice.
     La desinstalación de la 0.16 (F9-10) la pasa a `tc2 estado --stdin`.
   - **Orden**: migrar y verificar (criterio en verde y PR dentro) antes de
     desinstalar la 0.16 (F9-10); nunca con una tarea en curso; opcional y
     por proyecto.

7. **Nombres definitivos** (D1a), aprobados por Carlos el 2026-11-14 en la conversación de la tarea; queda registrado en la sección «Decisiones de la persona» del `tarea.md` de TASK-013.
   Distintos de los de la 0.16 y los mismos tras desinstalarla:

   | Pieza | 0.16 | 2.0 |
   |---|---|---|
   | CLI | `taskctl` | **`tc2`** |
   | Plugin del marketplace | `taskcode-plugin` | **`taskcodetwo`** |
   | Prefijo de skills | `taskcode-plugin:` | **`taskcodetwo:`** |
   | Estado bajo `.git` | `.git/taskcode/` | **`.git/taskcodetwo/`** |
   | Rama de estado | — (F1-1 la llamó `taskcode/tareas`) | **`taskcodetwo/tareas`** |
   | Status line | `taskctl estado --stdin` | **`tc2 estado --stdin`** |
   | Hooks | `hooks/hooks.json` del plugin 0.16 | los del plugin `taskcodetwo` (solo atajos opcionales, F1-11 dec. 10) |
   | Paquete npm | — | **`taskcodetwo`** con el binario `tc2` (el paquete `tc2` está ocupado) |
   | winget y Homebrew | — | `taskcodetwo` (libre en los dos el 2026-11-14) |
   | Config del proyecto | `.taskcode/config.yml` | la misma, con `formato: 2` (D4a) |

8. **Retirada de parches.** Cada parche de F1-11 declara `cubierto_por`, una
   capacidad que la 2.0 instalada anuncia (`estado-descartada`,
   `config-formato-2`, `tareas-json`, `puertas-en-ci`, `cli-2`,
   `roadmap-nativo`). Con la capacidad anunciada:
   - un bloque de líneas o un fichero que nadie carga se **retira solo** en
     el commit de `actualizar/formato-2`;
   - un fichero que otros cargan o una función se **retira con sustitución**:
     el plan da `fichero:líneas` y la sustitución es código del proyecto (la
     2.0 no reescribe código ajeno por heurística).
   Sin la capacidad, el parche **se queda como hueco**. Un hueco que lee la
   forma de carpetas de la 0.16 es **bloqueante**: `actualizar` se niega
   (código 5) sin tocar nada, porque migrar dejaría ciego al proyecto
   (OpenDockTrading: su lector pasa de 94 a 0 tareas). Lo propio se queda y
   pasa a los bloques de F1-11 (comando único con perfiles, F1-7).

9. **Orden de pilotos y dependencias** (D3a): **OpenDockTrading** primero
   (F9-7), después el **relevo de TaskCodeTwo** (F4-5) y por último
   **ToolsaWeb** (F9-8). Propuesta de cambio de dependencias en el plan, que
   se aplica con `taskctl` al cerrar esta tarea (no la cambia este ADR):
   F9-7 deja de depender de F9-6; F4-5 pasa a depender de F9-7; F9-8 pasa a
   depender de F4-5. Antes de migrar OpenDockTrading, sus lectores tienen que
   leer la carpeta fija o `tc2 tareas --json` (punto 8).

## Divergencias

### Nota de nombres para F1-1..F1-11

Los ADR aceptados no se modifican. Donde dicen `taskctl` refiriéndose a la
2.0 se lee **`tc2`**; donde dicen `taskcode/tareas`, **`taskcodetwo/tareas`**
(la rama de F1-1 se renombra por coherencia con el plugin); donde dicen
`.git/taskcode` para la 2.0, **`.git/taskcodetwo`**. Cuando hablan de la
0.16 (el `taskctl` de hoy), el nombre no cambia. Líneas afectadas, medidas
por el guion (G):

| ADR | Líneas con `taskctl` | con `taskcode/tareas` | con `.git/taskcode` |
|---|---|---|---|
| F1-1 | 1 | 16 | 0 |
| F1-2 | 0 | 9 | 2 |
| F1-3 | 3 | 8 | 0 |
| F1-4 | 28 | 7 | 0 |
| F1-5 | 1 | 13 | 0 |
| F1-6 | 2 | 9 | 0 |
| F1-7 | 4 | 1 | 0 |
| F1-8 | 5 | 4 | 0 |
| F1-9 | 0 | 1 | 0 |
| F1-10 | 9 | 1 | 0 |
| F1-11 | 35 | 9 | 0 |

(Son las filas «F1-N: lineas con…» de la tabla `cifras`.) En F1-2, las dos
líneas de `.git/taskcode` hablan de la 2.0 y se leen `.git/taskcodetwo`.
Las órdenes que esos ADR nombran para la 2.0 (`taskctl actualizar`,
`taskctl doctor`, `taskctl tareas --json`, `taskctl confiar`,
`taskctl derivados`…) son `tc2 actualizar`, `tc2 doctor`, etc.

### Divergencia con el plan

1. **Recuento de OpenDockTrading**: el plan decía 33 planificadas, 61
   terminadas y 2 archivadas; en `develop` `8141a08` son **50 / 44 / 2**
   (96 igual) y ninguna en diseño, en curso ni en revisión.
2. **Historia**: el plan pedía «para cada tarea, `git log --follow` en la
   rama de estado con al menos tantos commits como su ruta en `develop`».
   La primera medida falló en TASK-019 y TASK-020 (3 frente a 11); la causa
   está en `develop`: allí `--follow` da 1 de 11, porque la 0.16 movió y
   reescribió el fichero en el mismo commit. El criterio queda en dos
   medidas: commits por ruta (96/96) y `--follow` igual o mayor que el de
   `develop` (96/96). El aplanado va en un commit de **solo renombres** y el
   formato 2 en otro (el plan decía un commit).
3. **Huecos bloqueantes**: el plan dejaba un parche sin capacidad «como
   hueco». La PoC mide que en OpenDockTrading ese hueco deja a su lector en
   0 de 94 tareas; se decide que un hueco que lee la forma de carpetas
   **bloquea** `actualizar` (código 5). La PoC fuerza la migración para
   medir el resto.
4. **Retirada sola o con sustitución**: el plan no lo distinguía; las 76
   líneas que otros ficheros cargan o llaman no se pueden quitar sin código
   nuevo del proyecto.
5. **`.gitkeep`**: las carpetas de estado de la 0.16 llevan un `.gitkeep`;
   el aplanado los borra (el plan no lo nombraba).
6. **D2**: el plan sugería como fallo cerrado «que su parser plano aborte
   con bloques anidados». Los bloques anidados **no** lo hacen abortar (cada
   línea tiene `:`; solo avisa de claves desconocidas): aborta una línea sin
   `:` (`---` o una lista en bloque) o un valor inválido de una clave suya.
   Se elige `---`.
7. **Nombres y comillas**: el paquete npm `tc2` está ocupado; el paquete
   es `taskcodetwo` y el binario `tc2`. La config de la migración pone entre
   comillas dobles los valores de las claves que la 0.16 también lee
   (`modo_flujo`): con simples, la 0.16 conserva las comillas dentro del
   valor y aborta (variante D). Las órdenes de los bloques nuevos siguen
   entre simples (F1-11); ver la fila de la config en el punto 2.
8. **El commit fijo** es `8141a08`, no el `f8a9b5f` de F1-11: los scripts
   del desglose no cambian entre los dos.
9. **ToolsaWeb tiene 404 tareas**, no las 403 del criterio de F9-2 en el
   plan: 404 en la PoC A de F1-9 y 404 en `develop` en el recuento de la
   revisión 1 de esta tarea (234 planificadas, 161 terminadas, 9
   archivadas). Se propone corregir el criterio de F9-2 a 404 junto con las
   dependencias de D3; este ADR no cambia el plan.
10. **El tiempo del `subtree split`** sale de la tabla comprobada
    (revisión 1): varía entre ejecuciones y la puerta fallaba con datos
    honestos al cruzar un tramo.

## Requisitos y evidencia

| Requisito | Decisión | Evidencia |
|---|---|---|
| RR-13 simulación | `--simular` = `planificar` (punto 1) | PoC M: mismo hash en dos llamadas, 0 ficheros cambiados |
| RR-13 idempotente | plan vacío con `formato: 2`; reanudar desde el estado real | PoC M: segunda pasada sin pasos ni cambios; PoC M2: rama a medias reutilizada |
| RR-13 reversible | transacción local y vuelta atrás automática; `--deshacer` por PR | PoC M2: HEAD previo, 0 refs, 1 worktree, status limpio, remoto sin cambios; `--deshacer` no verificado |
| RR-13 primero ToolsaWeb y OpenDockTrading | OpenDockTrading primero; ToolsaWeb tras el relevo (D3a) | PoC M sobre OpenDockTrading; ToolsaWeb no medido (F9-2) |
| RR-14 se construye con la 0.16 | este repo no se migra hasta F4-5 | «Consecuencias» |
| RR-16 desinstalación limpia y `git` como única dependencia | orden de F9-10 tras el criterio; `git subtree` y no `filter-repo`; nombres distintos que sobreviven a la 0.16 | PoC N; PoC C (`.git/taskcode` y `.git/taskcodetwo` sin ficheros comunes) |
| F1-1 rama huérfana con carpeta fija | `subtree split` y aplanado | PoC M: `merge-base` vacío; 96 carpetas `TASK-NNN` |
| F1-9 tabla de campos | punto 2 | PoC M: 96 de 96 con formato 2 y YAML estándar; 192 claves retiradas |
| RF-07 / RF-71 archivadas | `descartada` con `motivo` | PoC M: 2 de 2 con motivo de `## Descartada` |
| F1-11 parches con fichero y líneas | punto 8 e informe A | PoC A: 91 / 973 con la 2.0.0 prevista, recalculados en `8141a08` |
| D2 convivencia | punto 6 | PoC C: variantes B, C y D cerradas; control A abierto |

## Consecuencias

- **F9-1** implementa `planificar` y `--simular` con este informe; **F9-2**
  `actualizar` con la transacción, el criterio y la vuelta atrás, y mide el
  `subtree split` de ToolsaWeb (404 tareas).
- **F2-3** crea y monta la rama de estado con el nombre `taskcodetwo/tareas`;
  **F2-5** reutiliza la conversión a formato 2; **F2-7** exige `---` y
  `formato: 2` en la config; **F3-8** (`doctor`) detecta las dos versiones,
  la estructura mixta y la config sin `---`.
- **F9-10** desinstala la 0.16 solo con el criterio en verde y el PR dentro;
  **F2-14** desinstala la 2.0 con los nombres de este ADR.
- **F1-14** usa `taskcodetwo` como paquete en npm, winget y Homebrew, con el
  binario `tc2`.
- **OpenDockTrading**, antes de F9-7: sus `lector-tareas.py`,
  `comprobar-modulos.py` y `comprobar-fallos.py` tienen que leer la carpeta
  fija o `tc2 tareas --json`; su lector no quita las comillas de las listas
  del formato 2 (PoC M: no reconoce las 2 descartadas).
- **Este repo** sigue en la 0.16 hasta el relevo (RR-14, F4-5).

## Fuera de alcance

- Migraciones entre versiones de la 2.x: F9-3.
- Empaquetado, firma y canales de distribución: F1-14.
- Desinstalación de la 2.0: F2-14; de la 0.16: F9-10 (aquí solo el orden).
- Medida en ToolsaWeb (404 tareas, tiempo del `subtree split`): F9-2.
- Las preguntas del hito de decisiones al migrar: TASK-094.

## Preguntas abiertas

Este ADR se planificó con un solo rol (arquitectura). Lo que no cubrió:

- **Riesgos**: corte a medias entre el push de la rama de estado y el de
  `actualizar/formato-2`; clon que no trae la rama de estado; rutas largas
  en Windows; tareas de la 0.16 mal formadas; firmas de commits tras el
  `subtree split`. Y uno que sale de la PoC: mientras el PR de
  `actualizar/formato-2` no entra, cambiar a la `develop` vieja con el
  worktree montado en `tareas/` puede pisar ficheros ignorados.
- **Dominio**: si `motivo: null` con «a pedir» basta para archivadas sin
  `## Descartada`, y si «hito» es lo mismo que `sprint` (F1-9).
- **Rendimiento**: el `subtree split` se recorre los 1350 commits de
  OpenDockTrading; ToolsaWeb es mayor. Si no cabe, la alternativa es un
  split propio con `git fast-export`, a decidir en F9-2 con la medida.

## No verificado

- **La retirada de parches no se ejecuta en la PoC**: no se editan
  `verificar.bat` ni los lectores de OpenDockTrading; solo se calcula su
  alcance (`fichero:líneas` y el 91 / 973). Los únicos ficheros de código que
  cambia la migración de la PoC son `.gitignore` y `.taskcode/config.yml`.

- **El choque de `tc` con iproute2** en una distro Linux de uso normal: la
  única distro de WSL de esta máquina (`docker-desktop`) no trae iproute2.
- **`tc2 actualizar --deshacer`** después del push (revert por PR y rama
  congelada): no se ha ejecutado.
- **Que `verificar.bat` de OpenDockTrading funcione** tras quitar sus 15
  líneas de parche: la PoC no ejecuta la verificación del proyecto.
- **GitHub**: el push va a un repo vacío local; ni PR ni rulesets reales.
- **macOS y Linux**: todo se ejecutó en Windows.
- **Las consultas de nombres** dependen del registro de npm, de la API de
  Homebrew y del índice de winget el 2026-11-14; pueden cambiar.
- **`taskctl doctor`** de este repo no se ejecutó en la implementación (la
  tarea pidió no lanzar `taskctl` aquí); queda para el cierre.

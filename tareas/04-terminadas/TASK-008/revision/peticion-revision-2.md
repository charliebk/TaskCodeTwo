# Peticion de revision — TASK-008 (ronda 2)

- Tarea: TASK-008 — F1-7 ADR verificacion en CI, perfiles, cache por SHA y runners
- Rama revisada: feature/task-008-f1-7-adr-verificacion-en-ci-perfiles-cac
- Rama base: develop
- Commit revisado (HEAD): 78a62a2d0b376ed31451a0409ad556c0c661e11b
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-008 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde fe0cbabe6390cf053a0d08c3bd2a582ad0681f52 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (5 hallazgo(s)))

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

- tareas/03-en-revision/TASK-008/tarea.md
- tareas/03-en-revision/TASK-008/planificacion/
- docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 3 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md:285 | informe-revision-1.md |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md:269 | informe-revision-1.md |

## Commits a revisar (git log fe0cbabe6390cf053a0d08c3bd2a582ad0681f52..HEAD)

````
78a62a2 chore(TASK-008): constancia de autorrevision ronda 2
fee7038 docs(adr): F1-7 corrige los hallazgos de la revision 1 (pesada en release, huella en el runner, menores)
cef1d67 chore(TASK-008): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff fe0cbabe6390cf053a0d08c3bd2a582ad0681f52..HEAD)

````diff
diff --git a/docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md b/docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md
index 3f9d62a..84eb4a9 100644
--- a/docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md
+++ b/docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md
@@ -42,13 +42,13 @@ cuesta con las suites reales de ToolsaWeb y OpenDockTrading.
 | **D1a** Completa en los PR a `develop` | La puerta de `develop` corre la completa; su resultado se reutiliza en la release. | **Elegida**. |
 | D1b Rápida en `develop`, completa en `main` | Lo literal de las consecuencias de F1-5. | Descartada: `develop` integraría sin la completa y la release la pagaría entera, sin caché que reutilizar; se corrige como divergencia con F1-5. |
 | D1c Impacto en `develop` | La selectiva en el PR. | Descartada: su resultado depende del diff, no solo del árbol, y no se puede reutilizar en la release. |
-| **D2a** Resultado en el check de Actions | El check `verificacion` lleva `arbol=`, `perfil=` y `clave=` en el resumen; se busca en el commit verificado, sus padres y los padres de estos. | **Elegida** (decisión 3). |
+| **D2a** Resultado en el check de Actions | El check `verificacion` lleva, por perfil, `arbol=`, `perfil=`, `runner=`, `clave=` y `local=` en el resumen; se busca en el commit verificado, sus padres y los padres de estos. | **Elegida** (decisión 3). |
 | D2b `actions/cache` | Una entrada de caché por clave. | Descartada: su alcance es por rama (un PR no ve la de otro PR, `main` no ve la del PR) y caduca; no deja rastro en el PR. |
 | D2c Ref, nota de Git o `taskcode/tareas` | Un índice árbol → resultado escrito en el repo. | Descartada: el agente puede escribir ahí (F1-1, F1-6) y fabricar un verde. |
 | **D3a** Runner propio caído: falla cerrado | El check se queda pendiente y `doctor` lo dice. | **Elegida**. Descartadas: (b) caer a un runner de GitHub (otro entorno, otra huella, y en la pesada sin los datos locales) y (c) dar el check por bueno. |
 | **D4a** Pesada dentro de `verificacion` | Con base `main` desde `develop`, el mismo check corre la completa (desde la caché) y la pesada. | **Elegida**: sin check nuevo ni cambio en los rulesets de F1-5. |
 | D4b Check `verificacion-pesada` aparte | Otro check obligatorio en `main`. | Descartada: obliga a enmendar F1-5. |
-| **D5a** Clave = árbol, perfil, huella declarada y versión del workflow | El proyecto declara su huella del entorno (herramientas, carpetas ignoradas que leen los tests). | **Elegida** (decisión 2). |
+| **D5a** Clave = árbol, perfil, huella declarada y versión del workflow | El proyecto declara su huella del entorno (herramientas, carpetas ignoradas que leen los tests), partida en una parte del árbol y otra local que solo se calcula en el runner del perfil; el runner declarado entra en la clave (revisión 1, IMP-2). | **Elegida** (decisión 2). |
 | D5b Solo el SHA del árbol | La clave de P6 tal cual. | Descartada: ToolsaWeb ya mete en su clave herramientas y carpetas ignoradas (`exampleData`, 34 GB); con solo el árbol, cambiar de `rustc` o de datos reutilizaría un verde viejo. |
 | **D6a** No ejecutar las suites | Coste desde tiempos ya medidos en los dos repos; factor de velocidad del runner «no verificado». | **Elegida**: los dos repos son solo lectura para esta tarea. |
 
@@ -69,16 +69,18 @@ este fichero**, entre los marcadores `entradas-coste` y `tabla-coste`. El
 guion sale con código distinto de 0 si alguna sección falla.
 
 ```bash
-bash poc-f1-7.sh "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-008/poc" \
+bash poc-f1-7.sh "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-008-c1" \
   docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md \
   /c/Users/nullcad2025/Documents/sources/Personal > poc-f1-7.log 2>&1
 ```
 
+Tras la revisión 1 se ejecuta en `...\task-008-c1\` (borrada al terminar):
+
 ```text
 RESULTADO A casos=10/10 como se esperaba
-RESULTADO B casos=10/10 ; controles negativos=8/8
+RESULTADO B casos=11/11 ; controles negativos=12/12
 RESULTADO C1 comprobaciones=9/9
-RESULTADO C2 errores ADR real=0 ; mutaciones detectadas=5/5 ; no aplicadas=0
+RESULTADO C2 errores ADR real=0 ; mutaciones detectadas=8/8 ; no aplicadas=0
 RESULTADO llamadas gh api con GET=1 ; gh api con otro metodo=0 ; descargas GET=3
 ```
 
@@ -108,31 +110,41 @@ ref), y actualizar la rama crea un árbol nuevo: **la caché no ahorra esa
 segunda verificación** (divergencia con F1-5, abajo). La release solo
 coincide si cada hotfix ha vuelto a `develop` (F1-3).
 
-### B — Búsqueda por padres
+### B — Búsqueda por padres, por perfil y con huella local
 
 Simulación en Python sobre un repo Git real: un diccionario hace de API de
 check-runs (lo que devolvería `GET /commits/{sha}/check-runs?check_name=verificacion&app_id=15368&filter=latest`),
-y la búsqueda es la de la decisión 3.
+y la búsqueda es la de las decisiones 2 a 4: `buscar` filtra por árbol,
+perfil, runner y la parte de la clave que se calcula desde el árbol; el job
+del runner del perfil da el acierto solo si su huella local coincide con la
+anotada; cada perfil que exige el evento se busca con su propia clave. Entre
+corchetes, qué pasa con cada perfil (los SHA, que cambian en cada ejecución,
+se omiten aquí; están en el log):
 
 ```text
-  ok   caso     fallo    primera verificacion del PR (merge ref nuevo): se verifica
-  ok   caso     acierto  commit vacio en la rama: el check del head anterior (abuelo del merge ref)
-  ok   caso     acierto  push del merge commit en develop: su padre (head del PR) tiene el arbol
-  ok   caso     acierto  PR de release develop->main: abuelo (head del PR de la tarea) con el mismo arbol
-  ok   caso     acierto  push del merge de la release en main
-  ok   caso     fallo    PR desactualizado: el merge ref es un arbol nuevo
-  ok   caso     acierto  tras actualizar-rama (strict): el arbol del head = merge ref verificado antes
-  ok   caso     fallo    develop avanzo y se actualiza la rama otra vez: arbol nuevo
-  ok   caso     acierto  revert al arbol del abuelo
-  ok   neg      fallo    mismo arbol a tres niveles: no se busca (limite conservador)
-  ok   neg      fallo    check verde de otro perfil (rapida)
-  ok   neg      fallo    otra huella del entorno
-  ok   neg      fallo    otra version del workflow
-  ok   neg      fallo    check en rojo
-  ok   neg      fallo    check verde de otra App con el mismo nombre
-  ok   neg      fallo    check verde con otro nombre
-  ok   neg      fallo    check verde sin arbol= en el resumen
-  ok   caso     acierto  control positivo: mismo montaje con todo igual
+  ok   caso  fallo    primera verificacion del PR (merge ref nuevo): se verifica [completa se ejecuta]
+  ok   caso  acierto  commit vacio en la rama: el check del head anterior (abuelo del merge ref) [completa reutilizada]
+  ok   caso  acierto  consulta tras el push del merge commit en develop: su padre (head del PR) tiene el arbol [completa reutilizada]
+  ok   caso  acierto  PR de release, solo el perfil completa: abuelo (head del PR de la tarea) con el mismo arbol [completa reutilizada]
+  ok   neg   fallo    PR de release (completa + pesada): la completa acierta, la pesada no corrio nunca: no hay verde sin ejecutarla [completa reutilizada; pesada se ejecuta]
+  ok   caso  acierto  re-ejecucion del PR de release: completa y pesada reutilizadas [completa reutilizada; pesada reutilizada]
+  ok   caso  acierto  consulta tras el push del merge de la release en main (completa y pesada) [completa reutilizada; pesada reutilizada]
+  ok   caso  fallo    PR desactualizado: el merge ref es un arbol nuevo [completa se ejecuta]
+  ok   caso  acierto  tras actualizar-rama (strict): el arbol del head = merge ref verificado antes [completa reutilizada]
+  ok   caso  fallo    develop avanzo y se actualiza la rama otra vez: arbol nuevo [completa se ejecuta]
+  ok   caso  acierto  revert al arbol del abuelo [completa reutilizada]
+  ok   neg   fallo    mismo arbol a tres niveles: no se busca (limite conservador) [completa se ejecuta]
+  ok   neg   fallo    check verde de otro perfil (rapida) [completa se ejecuta]
+  ok   neg   fallo    otra huella del arbol (lockfile o toolchain fijado) [completa se ejecuta]
+  ok   neg   fallo    misma huella del arbol, otra huella local en el runner (otro rustc, otros datos) [completa se ejecuta]
+  ok   neg   fallo    huella local sin calcular en el runner del perfil [completa se ejecuta]
+  ok   neg   fallo    verde de otro runner (self-hosted) para la completa declarada en runner de GitHub [completa se ejecuta]
+  ok   neg   fallo    otra version del workflow [completa se ejecuta]
+  ok   neg   fallo    check en rojo [completa se ejecuta]
+  ok   neg   fallo    check verde de otra App con el mismo nombre [completa se ejecuta]
+  ok   neg   fallo    check verde con otro nombre [completa se ejecuta]
+  ok   neg   fallo    check verde sin arbol= en el resumen [completa se ejecuta]
+  ok   caso  acierto  control positivo: mismo montaje con todo igual [completa reutilizada]
 ```
 
 Lectura: con los padres solos no basta (divergencia con el plan): en el PR de
@@ -140,7 +152,12 @@ release y tras un commit vacío, el check está en el **abuelo** del commit
 verificado, porque en un PR el check lo lleva el head y lo verificado es el
 merge ref, que tiene como padres la base y el head. Con dos niveles se cubren
 PR, merge en `develop` y release sin índice por árbol. Un acierto tras
-`actualizar-rama` solo ocurre si nadie integró nada entre medias.
+`actualizar-rama` solo ocurre si nadie integró nada entre medias. Un acierto
+solo exime de su perfil: en la release, la completa de `develop` se
+reutiliza y la pesada se ejecuta (revisión 1, IMP-1). Un verde de otro runner,
+con otra huella local o sin huella local calculada en el runner no cuenta
+(revisión 1, IMP-2); que el job del runner pueda calcularla en GitHub
+Actions como aquí se simula, **no verificado**.
 
 ### C — Coste y esquema de la API
 
@@ -163,13 +180,21 @@ PR, merge en `develop` y release sin índice por árbol. Un acierto tras
 La API no busca check-runs por árbol: de ahí la búsqueda por commits.
 
 **C2, coste.** Cada entrada de la decisión 6 se comprueba contra su fuente:
-la línea citada de ToolsaWeb u OpenDockTrading tiene que contener el valor; los
+en la línea citada de ToolsaWeb u OpenDockTrading la cifra tiene que estar **en
+su contexto** (una expresión por entrada: E1 en la fila «primera tras el
+cambio», E2 antes de «(TASK-284», E3 como final de «rondas de … a», E4
+seguida de «en frío»), no basta con que aparezca en la línea; los
 recuentos de tareas se rehacen leyendo el campo `creado` de cada `tarea.md`
 terminada (si hay más que en el ADR, aviso: el repo ha crecido); los precios
 se buscan en las dos páginas de docs.github.com descargadas con `GET`. Luego
-se calcula la tabla y se compara celda a celda con la del ADR. Cinco copias
-mutadas del ADR (una entrada medida, una celda de la tabla, la línea de una
-fuente, un supuesto y un precio) tienen que fallar, y fallan (5/5).
+se calcula la tabla y se compara celda a celda con la del ADR, y las cifras
+de su lectura (marcadores `lectura-coste`) con las calculadas. Ocho copias
+mutadas del ADR tienen que fallar **por el error esperado**, y fallan (8/8):
+una entrada medida, una celda de la tabla, la línea de una fuente, un
+supuesto, un precio, E4 sustituida por la medida en caliente de su misma
+línea con C4, T1 y T2 recalculados a mano (solo lo detecta el contexto), una
+cifra de la lectura y E3 sustituida por el promedio anterior de su misma
+línea.
 
 ### Divergencias con el plan
 
@@ -183,10 +208,11 @@ fuente, un supuesto y un precio) tienen que fallar, y fallan (5/5).
    un árbol que nadie ha verificado. Lo que ahorra es la release, la cola de
    merge, el push del merge commit y las re-ejecuciones sobre el mismo árbol.
    Su coste entra en la tabla como supuesto S1.
-4. **Job de búsqueda aparte en Linux**: para no despertar un runner Windows (ni
-   el propio) cuando hay acierto, `buscar` corre en `ubuntu-latest` y
-   `verificacion` elige su `runs-on` con su salida (decisión 3). Cuesta un
-   minuto Linux por evento (filas C3 y C5).
+4. **Job de búsqueda aparte en Linux**: `buscar` corre en `ubuntu-latest` y
+   hace las llamadas a la API con la parte de la clave que se calcula desde el
+   árbol; `verificacion` corre siempre en el runner del perfil, donde calcula
+   la huella local (decisión 3; revisión 1, IMP-2). Cuesta un minuto Linux por
+   evento (filas C3 y C5), y un acierto despierta el runner un momento.
 5. **Pesada solo en la release, no en el hotfix**: el plan dice «PR a `main`
    completa desde la caché más la pesada»; ToolsaWeb decidió que los hotfix no
    corren los pesados (ToolsaWeb, `docs/verificacion/flujo-de-verificacion.md:299`).
@@ -205,6 +231,22 @@ fuente, un supuesto y un precio) tienen que fallar, y fallan (5/5).
    redondeo por job (22 min 22 s → 23 min).
 9. **`taskctl doctor`** del plan de pruebas no lo ejecuta esta implementación
    (se le pidió no ejecutar `taskctl`); `comprobar-repo.sh` sí, en verde.
+10. **Un acierto solo exime de su perfil** (revisión 1, IMP-1, decidido por
+    Carlos con la consigna «la opción recomendada»): en la release,
+    `verificacion` busca la completa y la pesada cada una con su perfil y su
+    clave; la pesada sin acierto se ejecuta; el resumen anota los dos
+    perfiles. En la primera versión, el acierto de la completa de `develop`
+    daba el check de la release por bueno sin correr la pesada.
+11. **Huella partida y runner en la clave** (revisión 1, IMP-2, misma
+    consigna): la huella del árbol la calcula `buscar`; la local, el job en el
+    runner del perfil; sin la local calculada allí no hay acierto, y el runner
+    declarado entra en la clave. En la primera versión la huella era una sola
+    y `buscar`, en Linux, no podía calcular la del runner.
+12. **Menores de la revisión 1**: E3 es un promedio, no un máximo, y «una
+    ejecución de CI por ronda» pasa a supuesto S4 (MEN-1); cada entrada se
+    comprueba en su contexto y la lectura de la tabla también (MEN-2); filas
+    de `push` y `merge_group` y `completa`/`pesada` independientes de
+    `RAMA_BASE` (MEN-3).
 
 ### No verificado
 
@@ -212,7 +254,8 @@ fuente, un supuesto y un precio) tienen que fallar, y fallan (5/5).
   máquina de Carlos: se usa 1,0 (supuesto S2). Una completa fría de ToolsaWeb
   en un runner de 2 núcleos, sin `target/` caliente, probablemente tarda más.
 - **Actualizaciones de rama por tarea** con `strict`: se usa 0,5 (supuesto
-  S1), sin medir.
+  S1), sin medir; y **una ejecución de CI por ronda de revisión** (supuesto
+  S4).
 - **Releases al mes y duración de la pesada sola**: cero releases medidas en
   la 2.0; la pesada de ToolsaWeb se deriva (paso 18 con pesados 29 min 01 s
   frente a 14 min 53 s sin ellos, `flujo-de-verificacion.md:224` y `:263`:
@@ -227,8 +270,15 @@ fuente, un supuesto y un precio) tienen que fallar, y fallan (5/5).
 - **Cómo escribe el workflow `output.summary`** de su propio check (el
   resumen del paso del job o un check-run propio con `GITHUB_TOKEN`): el
   esquema lo admite (C1); el mecanismo concreto, F5-2.
-- **`runs-on` con una expresión** que depende de la salida de otro job: sintaxis
-  conocida de Actions, no ejecutada.
+- **Dónde se calcula la huella** (revisión 1, IMP-2): que la parte del árbol
+  (lockfiles, toolchain fijado) se pueda calcular en `buscar` sin el runner,
+  que el job en el runner del perfil calcule la local y la compare con la de
+  los candidatos que le pasa `buscar` (salidas de un job a otro), y que un
+  runner self-hosted caído deje el check pendiente también con acierto: PoC B
+  lo simula; en GitHub Actions, no ejecutado.
+- **Que `completa` y `pesada` de ToolsaWeb no dependan de `RAMA_BASE`**: su
+  completa planifica por grupos con un clasificador que lee la base; F1-11 y
+  F5-1 lo comprueban antes de cachear su resultado por árbol.
 - **Cola de merge**: que el commit de `merge_group` tenga el árbol del head;
   si no, falla la caché y se verifica otra vez (caro pero seguro).
 - **Cuánto espera un job en cola por un runner self-hosted** antes de fallar.
@@ -244,9 +294,9 @@ Con este ADR aplicado (conocimiento general de GitHub, **no verificado**):
 
 | Dónde | Qué se ve |
 |---|---|
-| Checks del PR a `develop` | `verificacion` con perfil completa; en el resumen, la línea `arbol=<sha> perfil=completa clave=<k> origen=<sha>` y el tiempo; un acierto sale verde en segundos con «reutilizado de `<sha>`» |
-| Pestaña Actions de una ejecución | dos jobs: `buscar` (Linux, segundos) y `verificacion` (en el runner del perfil si no hubo acierto; en Linux y en segundos si lo hubo) |
-| PR de release `develop`→`main` | `verificacion` con «completa: reutilizado de `<sha>`» y la pesada corriendo en el runner propio |
+| Checks del PR a `develop` | `verificacion` con perfil completa; en el resumen, la línea `arbol=<sha> perfil=completa runner=<r> clave=<k> local=<h> origen=<sha>` y el tiempo; un acierto sale verde en segundos con «completa: reutilizada de `<sha>`» |
+| Pestaña Actions de una ejecución | dos jobs: `buscar` (Linux, segundos) y `verificacion` (siempre en el runner del perfil: segundos si hubo acierto, la suite si no) |
+| PR de release `develop`→`main` | `verificacion` con dos líneas en el resumen: «completa: reutilizada de `<sha>`» y «pesada: ejecutada» (o «reutilizada» al relanzar el mismo árbol), la pesada en el runner propio |
 | Runner propio apagado | `verificacion` pendiente con «Waiting for a runner to pick up this job»; el PR no se puede mergear; `doctor` dice qué runner falta |
 | Settings → Actions → Runners (repos privados) | el runner de la máquina de Carlos con sus etiquetas (`self-hosted`, `windows`, la del proyecto) |
 | Repo público | ningún runner propio; todo en runners de GitHub |
@@ -262,33 +312,54 @@ Con este ADR aplicado (conocimiento general de GitHub, **no verificado**):
    Sustituye a los seis `comando_*` (H18, RF-41). Inclusión: rápida ⊂ impacto
    ⊂ completa; la **pesada es aparte** y no sustituye a la completa. El
    workflow reutilizable de GitHub y la plantilla de GitLab solo llaman a ese
-   comando (P6, P11); no conocen los grupos ni los pasos del proyecto. El
+   comando (P6, P11); no conocen los grupos ni los pasos del proyecto.
+   **El resultado de `completa` y `pesada` no puede variar con `RAMA_BASE`**:
+   se cachea por árbol, y si dependiera de la base dejaría de ser función del
+   árbol (por eso mismo se descartó la impacto en `develop`, D1c);
+   `RAMA_BASE` solo la usan `rapida` e `impacto`. El
    comando corre en el shell que declare el proyecto para el runner (un
    `.bat` exige Windows: OpenDockTrading, H16); el workflow no reinterpreta la
    orden con otro shell.
-2. **Clave de la caché** = árbol verificado + `clave`, donde
-   `clave = sha256(perfil | huella | versión del workflow)`. La **huella** la
-   declara el proyecto como una orden que imprime lo que su suite lee fuera
-   del árbol (versiones de herramientas, contenido de carpetas ignoradas,
-   variables): es una extensión de F1-11, y ToolsaWeb ya la tiene
-   (`cache-verificacion.py`). Los pasos que no dependen solo del árbol
+2. **Clave de la caché** = árbol verificado + `clave` + `local`, donde
+   `clave = sha256(perfil | runner declarado | huella del árbol | versión del workflow)`
+   y `local = sha256(huella local)`. El proyecto declara su **huella** en dos
+   partes (extensión de F1-11; ToolsaWeb ya hace algo así con
+   `cache-verificacion.py`): la **del árbol**, lo que se deduce de ficheros
+   versionados (lockfiles, toolchain fijado), que `buscar` calcula sin el
+   runner; y la **local**, una orden que imprime lo que la suite lee fuera del
+   árbol en la máquina que la corre (versiones de herramientas instaladas,
+   contenido de carpetas ignoradas como `exampleData`, variables), que **solo
+   se calcula en el runner del perfil**. Sin la local calculada en ese runner
+   no hay acierto. El **runner declarado** entra en la clave: cambiar la
+   completa de `github` a `self-hosted` no reutiliza el verde del otro entorno
+   (PoC B). Los pasos que no dependen solo del árbol
    (`cargo audit` y `npm audit` de OpenDockTrading, `scripts/verificar.bat:69-70`)
    salen del check cacheado a un job programado (detalle en F1-11): si no,
    un acierto escondería un aviso nuevo.
 3. **Dónde vive el resultado y cómo se busca.** En el propio check
    `verificacion` que publica GitHub Actions (`integration_id` 15368, F1-5),
-   con `arbol=`, `perfil=` y `clave=` en `output.summary`. El job `buscar`
-   (Linux) toma el commit verificado `V` (el merge ref en un PR, el commit en
-   un push), y para `V`, sus padres y los padres de estos (como mucho 7
-   commits), en ese orden, pide
-   `GET /repos/{o}/{r}/commits/{c}/check-runs?check_name=verificacion&app_id=15368&filter=latest`.
-   Es **acierto** el primero en verde cuyo `arbol` sea el árbol de `V` y cuyos
-   `perfil` y `clave` sean los actuales; con acierto, `verificacion` corre en
-   Linux y sale verde con «reutilizado de `<sha>`»; sin él, corre el comando en
-   el runner del perfil (`runs-on` según la salida de `buscar`). Ningún
-   índice por árbol, ni `actions/cache`, ni nada que el agente pueda escribir
-   (D2). Un check de otra App, de otro nombre, en rojo o sin `arbol=` no
-   cuenta (PoC B).
+   con una línea por perfil en `output.summary`:
+   `arbol=<árbol> perfil=<p> runner=<r> clave=<k> local=<h> origen=<sha>`.
+   Para **cada perfil que exige el evento** (decisión 4), con su propia clave:
+   1. el job `buscar` (Linux) toma el commit verificado `V` (el merge ref en
+      un PR, el commit en el `merge_group`), y para `V`, sus padres y los
+      padres de estos (como mucho 7 commits), en ese orden, pide
+      `GET /repos/{o}/{r}/commits/{c}/check-runs?check_name=verificacion&app_id=15368&filter=latest`;
+      son candidatos las líneas de checks en verde con el `arbol` de `V` y el
+      `perfil`, el `runner` y la `clave` actuales;
+   2. el job `verificacion`, **siempre en el runner del perfil**, calcula la
+      huella local; es **acierto** el primer candidato con el mismo `local`.
+      Con acierto, ese perfil sale «reutilizado de `<sha>`» en segundos; sin
+      él, corre el comando con ese perfil.
+
+   **Un acierto solo exime de su perfil**: en la release, la completa suele
+   acertar (la de `develop`) y la pesada se busca aparte y, sin acierto, se
+   ejecuta; el check solo sale verde cuando todos los perfiles del evento
+   están reutilizados o ejecutados en verde, y el resumen anota cada uno. Un
+   runner propio caído deja el check pendiente aunque haya candidato (D3a).
+   Ningún índice por árbol, ni `actions/cache`, ni nada que el agente pueda
+   escribir (D2). Un check de otra App, de otro nombre, en rojo o sin `arbol=`
+   no cuenta (PoC B).
 4. **Perfil según el evento:**
 
    | Evento | Perfil | Dónde corre |
@@ -296,8 +367,10 @@ Con este ADR aplicado (conocimiento general de GitHub, **no verificado**):
    | Hook local (opcional) | rápida | máquina de la persona, solo con confianza explícita en el repo (H2; F1-8, F5-6) |
    | Rondas del agente antes de `review` | impacto | local, como la selectiva de hoy; no es un check obligatorio |
    | PR a `develop` (todos los carriles) | completa (D1a), desde la caché si acierta | runner de la completa |
-   | PR de release `develop`→`main` | completa desde la caché + pesada, en el mismo check `verificacion` (D4a) | completa: caché; pesada: runner de la pesada |
+   | PR de release `develop`→`main` | completa y pesada, cada una buscada con su clave, en el mismo check `verificacion` (D4a): la completa suele reutilizarse de `develop`; la pesada se ejecuta salvo que ya corriera sobre ese árbol | runner de cada perfil |
    | PR de hotfix a `main` | completa | runner de la completa |
+   | `merge_group` (cola, solo en organización) | completa, desde la caché si el árbol coincide con el del head del PR (no verificado) | runner de la completa |
+   | `push` a `develop` o a `main` | no se ejecuta `verificacion`: ningún ruleset lo exige en push (F1-5) y el resultado de ese árbol ya está en el check del head del PR, que la búsqueda en dos niveles alcanza (PoC B, «consulta tras el push») | — |
 
 5. **Runner según el perfil.** Cada perfil declara su runner: `github` (con el
    sistema: `windows` o `linux`) o una lista de etiquetas
@@ -316,7 +389,7 @@ Con este ADR aplicado (conocimiento general de GitHub, **no verificado**):
    |---|---|---|---|
    | `E1` | ToolsaWeb, completa sin pesados con su caché vacía (primera tras TASK-344) | `22 min 22 s` | `ToolsaWeb/docs/verificacion/flujo-de-verificacion.md:263` (commit `0a8b4747`) |
    | `E2` | ToolsaWeb, completa con la caché caliente, peor caso medido | `14 min 31 s` | `ToolsaWeb/docs/verificacion/flujo-de-verificacion.md:307` (TASK-284, 2026-11-12) |
-   | `E3` | Rondas de revisión por tarea (máximo medido en ToolsaWeb; se usa también para OpenDockTrading) | `3,0` | `ToolsaWeb/docs/verificacion/flujo-de-verificacion.md:291` |
+   | `E3` | Rondas de revisión medias por tarea (ToolsaWeb, `taskctl metricas` sobre 153 tareas; el valor actual de la serie «de 2,2 a 3,0»; se usa también para OpenDockTrading) | `3,0` | `ToolsaWeb/docs/verificacion/flujo-de-verificacion.md:291` |
    | `E4` | OpenDockTrading, verificación en frío (un solo perfil) | `4 min 11 s` | `OpenDockTrading/.taskcode/config.yml:8` (2026-11-13) |
    | `E5` | ToolsaWeb, tareas terminadas | `160` | `ToolsaWeb/tareas/04-terminadas`, `creado` de `2026-10-16` a `2026-11-13` (29 días) |
    | `E6` | OpenDockTrading, tareas terminadas (ritmo de arranque, no estable) | `43` | `OpenDockTrading/tareas/04-terminadas`, `creado` de `2026-11-09` a `2026-11-13` (5 días) |
@@ -327,12 +400,14 @@ Con este ADR aplicado (conocimiento general de GitHub, **no verificado**):
    | `E11` | Uso incluido en GitHub Free, por mes | `$12 USD` | docs.github.com/en/billing/concepts/product-billing/github-actions (2026-11-14) |
    | `S1` | Actualizaciones de rama por tarea (`strict`) | `0,5` | supuesto, no medido |
    | `S2` | Factor de velocidad del runner frente a la máquina de Carlos | `1,0` | supuesto, no verificado |
-   | `S3` | Minutos facturados del job `buscar` (y de `verificacion` con acierto) | `1` | redondeo por job (E10) |
+   | `S3` | Minutos facturados del job `buscar` | `1` | redondeo por job (E10) |
+   | `S4` | Ejecuciones de CI por ronda de revisión (cada ronda acaba en un push al PR) | `1` | supuesto, no medido |
    <!-- entradas-coste:fin -->
 
    Tareas al mes = terminadas × 30 / días de la ventana (redondeado);
-   ejecuciones por tarea = E3 + S1 (cada ronda empuja y cada actualización de
-   rama crea un árbol nuevo: fallos de caché); minutos facturados = minutos ×
+   ejecuciones por tarea = E3 × S4 + S1 (cada ronda empuja y cada
+   actualización de rama crea un árbol nuevo: fallos de caché; las rondas
+   locales del agente antes de `review` no cuentan); minutos facturados = minutos ×
    S2 redondeados al minuto por arriba (E10); $/mes de cada fila redondeado a
    céntimos; T1 y T2 restan el uso incluido (E11).
 
@@ -348,16 +423,22 @@ Con este ADR aplicado (conocimiento general de GitHub, **no verificado**):
    | `T2` | Los dos | completa de ToolsaWeb en su máquina (C2+C3+C4+C5 − E11) | — | — | — | — | 14 714 | — | 42,06 |
    <!-- tabla-coste:fin -->
 
+   <!-- lectura-coste:inicio -->
    Lectura: la completa de ToolsaWeb en runners de GitHub es casi todo el
    coste (133,63 $ de 175,69 $ al mes); en la máquina de Carlos, con la caché
    de `target/` caliente, no gasta minutos (E9) pero la ocupa unas 145 horas
    al mes (8 715 min). OpenDockTrading en GitHub, unos 45 $ al mes con su ritmo
-   de arranque. Dónde corre la completa de ToolsaWeb es configuración del
+   de arranque.
+   <!-- lectura-coste:fin -->
+   Dónde corre la completa de ToolsaWeb es configuración del
    proyecto (decisión 5); la tabla da el dato para elegir. La pesada no entra:
-   corre en la máquina de Carlos (0 $) y no hay releases medidas. Lo que la
-   caché ahorra en estos repos es poco en dinero (la completa de cada release
-   y las re-ejecuciones); su valor es el tiempo de la release y de la cola, y
-   no repetir (RR-07).
+   corre en la máquina de Carlos (0 $) y no hay releases medidas. Un acierto
+   también despierta el runner del perfil un momento (el job que calcula la
+   huella local, decisión 3): en un runner de GitHub, un minuto facturado por
+   acierto; los aciertos son pocos (la release y las re-ejecuciones) y no
+   entran en la tabla. Lo que la caché ahorra en estos repos es poco en dinero
+   (la completa de cada release y las re-ejecuciones); su valor es el tiempo
+   de la release y de la cola, y no repetir (RR-07).
 
 ## Divergencias con F1-5
 
@@ -380,13 +461,13 @@ y en «Consecuencias»:
 
 | Requisito | Decisión | Evidencia |
 |---|---|---|
-| RR-07 / RF-42 nada se verifica dos veces sobre el mismo árbol | resultado en el check con `arbol=` y `clave=`; búsqueda en dos niveles (decisión 3) | PoC A (10/10: PR al día = merge en `develop` = release), B (10/10 casos y 8/8 negativos), C1 (filtros `check_name`, `app_id`, `filter` y `output.summary` en el esquema); comportamiento real: no verificado (F5-2) |
+| RR-07 / RF-42 nada se verifica dos veces sobre el mismo árbol | resultado en el check con `arbol=`, `clave=` y `local=` por perfil; búsqueda en dos niveles; un acierto solo exime de su perfil (decisiones 2 y 3) | PoC A (10/10: PR al día = merge en `develop` = release), B (11/11 casos y 12/12 negativos, con la pesada de la release, otro runner y otra huella local), C1 (filtros `check_name`, `app_id`, `filter` y `output.summary` en el esquema); comportamiento real: no verificado (F5-2) |
 | RF-41 perfiles rápida, impacto, completa y pesada en release | un comando con `--perfil`; tabla de eventos (decisión 4) | decisiones 1 y 4; los cuatro perfiles de ToolsaWeb, en `ToolsaWeb/.taskcode/config.yml:9`, `:16`, `:18` y `:23` (solo lectura); implementación: F5-1 |
 | P6, viñeta 1: workflow reutilizable con perfiles y caché compartida por PR, cola y release | decisiones 1, 3 y 4 | PoC A y B; cola de merge: no verificado (F1-13, F2-6) |
 | P6, viñeta 2: el revisor IA lee el informe de la CI | informe JSON opcional del comando (decisión 1) | formato y uso: F5-4, F6-3; no verificado |
 | P6, viñeta 3 / RN-02: runner propio Windows por perfil | runner declarado por perfil, solo en repos privados; falla cerrado (decisión 5) | decisión 5; tabla de coste C2 y T2; implementación: F5-5 |
 | P6, viñeta 4 / H2: en local solo un hook rápido, opcional y con confianza | hook con la rápida, solo con confianza explícita (decisión 4) | modelo de confianza: F1-8; implementación: F5-6 |
-| Criterio de F1-7: coste con las suites de ToolsaWeb y OpenDockTrading | tabla de la decisión 6 | PoC C2 (11 entradas comprobadas contra su fuente, tabla igual a la calculada, 5/5 mutaciones detectadas); supuestos S1–S3 y factor de velocidad: no verificado |
+| Criterio de F1-7: coste con las suites de ToolsaWeb y OpenDockTrading | tabla de la decisión 6 | PoC C2 (11 entradas comprobadas contra su fuente, las medidas en su contexto en la línea citada; tabla y lectura iguales a las calculadas; 8/8 mutaciones detectadas por el error esperado); supuestos S1–S4 y factor de velocidad: no verificado |
 | H18 superficie de verificación | un comando en lugar de seis claves (decisión 1) | decisión 1; la clave en la configuración: F1-11 |
 | H16 el mismo comando distinto con `cmd.exe` y `sh` | el comando corre en el shell y el runner que declara el proyecto; un `.bat` exige Windows | decisiones 1 y 5; OpenDockTrading (`verificar.bat`) en runner Windows (C4) |
 | RR-15 GitHub y GitLab | el comando y la clave son neutrales; la plantilla de GitLab llama al mismo comando | no verificado (F1-13, F5-1) |
@@ -396,8 +477,9 @@ y en «Consecuencias»:
 - **F5-1** implementa el workflow reutilizable (jobs `buscar` y
   `verificacion`, `runs-on` por perfil) y la plantilla de GitLab, y asegura que
   el check se llame `verificacion` como exige F1-5.
-- **F5-2** implementa la búsqueda en dos niveles y la escritura de `arbol=`,
-  `perfil=` y `clave=` en `output.summary`, y mide las repeticiones.
+- **F5-2** implementa la búsqueda en dos niveles por perfil, el job de la
+  huella local en el runner y la escritura de `arbol=`, `perfil=`, `runner=`,
+  `clave=` y `local=` en `output.summary`, y mide las repeticiones.
 - **F5-3** cambia de expectativa: la impacto no es el check de los PR (D1a);
   queda para las rondas del agente. Hay que actualizar su fila en el plan.
 - **F5-4** fija el informe JSON para el revisor; **F5-5**, el runner de la
@@ -407,9 +489,10 @@ y en «Consecuencias»:
   `pull_request_target`, quién puede publicar un check `verificacion` con un
   resumen falso (cualquier workflow del repo con `checks:write`) y la
   confianza del hook.
-- **F1-11** declara en la configuración el comando, la huella y el runner por
-  perfil, y saca a un job programado los pasos que no dependen solo del
-  árbol.
+- **F1-11** declara en la configuración el comando, la huella (del árbol y
+  local) y el runner por perfil, saca a un job programado los pasos que no
+  dependen solo del árbol y exige que `completa` y `pesada` no dependan de
+  `RAMA_BASE`.
 - **F1-13** y **F2-6** verifican la cola de merge (árbol de `merge_group`) y el
   equivalente en GitLab.
 - **F3-8** (`doctor`) dice qué runner propio falta cuando un check se queda
````

## Excluido del diff (8 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff fe0cbabe6390cf053a0d08c3bd2a582ad0681f52..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/02-en-curso/TASK-008/poc/poc-f1-7.log                                                | 214 ----------------------------------------
 tareas/{02-en-curso => 03-en-revision}/TASK-008/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-008/planificacion/plan-final.md                 |   0
 tareas/03-en-revision/TASK-008/poc/poc-f1-7.log                                             | 336 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-008/poc/poc-f1-7.sh                             | 176 +++++++++++++++++++++++----------
 tareas/03-en-revision/TASK-008/revision/informe-revision-1.md                               |  21 ++++
 tareas/03-en-revision/TASK-008/revision/peticion-revision-1.md                              | 537 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-008/tarea.md                                    |  11 ++-
 8 files changed, 1025 insertions(+), 270 deletions(-)
````

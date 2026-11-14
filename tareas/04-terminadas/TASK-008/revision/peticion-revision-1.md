# Peticion de revision — TASK-008 (ronda 1)

- Tarea: TASK-008 — F1-7 ADR verificacion en CI, perfiles, cache por SHA y runners
- Rama revisada: feature/task-008-f1-7-adr-verificacion-en-ci-perfiles-cac
- Rama base: develop
- Commit revisado (HEAD): fe0cbabe6390cf053a0d08c3bd2a582ad0681f52
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-008 (criterios de aceptacion y plan)
- Alcance: completa (motivo: primera ronda de revision)

## Instrucciones para el agente revisor

Eres un revisor INDEPENDIENTE del agente que implemento. Tu trabajo es
reproducir empiricamente, no leer el diff y opinar: clona el repo a un
directorio temporal, corre la suite tu mismo (salvo que esta
peticion ya traiga la verificacion de la ronda en verde para el commit revisado;
entonces vale ese resultado) y construye el caso que
rompe el codigo antes de reportarlo. Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. Vuelca tu salida en el
informe de esta ronda (informe-revision-1.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-008/tarea.md
- tareas/03-en-revision/TASK-008/planificacion/
- docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md
- docs/adr/README.md

## Commits a revisar (git log develop..HEAD)

````
fe0cbab chore(TASK-008): constancia de autorrevision
1a083e6 docs(adr): F1-7 verificacion en CI, perfiles, cache por arbol y runners, con PoC
116edf4 chore(TASK-008): tarea en curso
````

## Diff completo (git diff develop..HEAD)

````diff
diff --git a/docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md b/docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md
new file mode 100644
index 0000000..3f9d62a
--- /dev/null
+++ b/docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md
@@ -0,0 +1,451 @@
+# ADR F1-7 — Verificación en CI: perfiles, caché por árbol y runners
+
+- Estado: **aceptada** (2026-11-14)
+- Tarea: TASK-008 · Plan: [F1-7 en PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) (§4 P6 y tabla de la Fase 1)
+- Requisitos: RR-07, RR-15, RF-41, RF-42 y RN-02 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); las cuatro viñetas de P6 del plan; H2, H16 y H18 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
+- Depende de: [ADR F1-5](F1-5-puertas-en-github.md) (check `verificacion` obligatorio atado a GitHub Actions, `integration_id` 15368; `strict` en `develop` y no en `main`), [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md) (merge commit sin fast-forward, release por merge de `develop` en `main`), [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (el roadmap y las tareas fuera del árbol de código) y [ADR F1-6](F1-6-identidades-agente-revisor-persona.md) (el agente lee checks y no los escribe; no toca workflows)
+- Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a, D5a y D6a del plan de TASK-008)
+
+## Contexto
+
+F1-5 dejó a este ADR el perfil de `verificacion` según la base del PR y la
+caché que abarata `strict` sin cola de merge. El marco:
+
+- **P6 / RR-07 / RF-42**: nada se verifica dos veces sobre el mismo árbol; la
+  caché por árbol la comparten el PR, la cola de merge y la release. Hoy
+  ToolsaWeb repite una completa en `finish` aunque la acabe de pasar a mano,
+  porque TaskCode 0.16 no ve lo que su caché sí ve (los ficheros ignorados que
+  leen los tests).
+- **P6 / RF-41**: cuatro perfiles (rápida, impacto o selectiva, completa y
+  pesada solo en release); ToolsaWeb usa los cuatro.
+- **P6 / RN-02**: un runner propio Windows además de los de GitHub, elegible
+  por perfil, para suites pesadas en repos privados.
+- **P6 / H2**: en local, solo un hook rápido, opcional y con confianza
+  explícita; nada del repo se ejecuta sin ella.
+- **P6**: el revisor IA lee el informe de la CI y no repite tests.
+- **H18**: hoy hay seis claves `comando_*` de verificación; **H16**: un mismo
+  comando se comporta distinto con `cmd.exe` y con `sh`.
+- **RR-15**: el modelo vale para GitHub y GitLab; lo que una plataforma no dé
+  se documenta.
+
+Preguntas que responde: qué contrato tiene el comando de verificación del
+proyecto, qué clave tiene la caché y dónde vive el resultado, qué perfil corre
+en cada evento, en qué runner, qué pasa si el runner propio no está, y cuánto
+cuesta con las suites reales de ToolsaWeb y OpenDockTrading.
+
+## Alternativas
+
+| | Descripción | Veredicto |
+|---|---|---|
+| **Un comando con `--perfil`** | El proyecto da un solo comando; el perfil es un argumento y el resultado el código de salida. | **Elegida** (decisión 1): sustituye a los seis `comando_*` (H18). |
+| Una clave por perfil | Como hoy: `comando_verificacion`, `_freno`, `_selectiva`, `_pesada`… | Descartada: es la superficie que señala H18. |
+| **D1a** Completa en los PR a `develop` | La puerta de `develop` corre la completa; su resultado se reutiliza en la release. | **Elegida**. |
+| D1b Rápida en `develop`, completa en `main` | Lo literal de las consecuencias de F1-5. | Descartada: `develop` integraría sin la completa y la release la pagaría entera, sin caché que reutilizar; se corrige como divergencia con F1-5. |
+| D1c Impacto en `develop` | La selectiva en el PR. | Descartada: su resultado depende del diff, no solo del árbol, y no se puede reutilizar en la release. |
+| **D2a** Resultado en el check de Actions | El check `verificacion` lleva `arbol=`, `perfil=` y `clave=` en el resumen; se busca en el commit verificado, sus padres y los padres de estos. | **Elegida** (decisión 3). |
+| D2b `actions/cache` | Una entrada de caché por clave. | Descartada: su alcance es por rama (un PR no ve la de otro PR, `main` no ve la del PR) y caduca; no deja rastro en el PR. |
+| D2c Ref, nota de Git o `taskcode/tareas` | Un índice árbol → resultado escrito en el repo. | Descartada: el agente puede escribir ahí (F1-1, F1-6) y fabricar un verde. |
+| **D3a** Runner propio caído: falla cerrado | El check se queda pendiente y `doctor` lo dice. | **Elegida**. Descartadas: (b) caer a un runner de GitHub (otro entorno, otra huella, y en la pesada sin los datos locales) y (c) dar el check por bueno. |
+| **D4a** Pesada dentro de `verificacion` | Con base `main` desde `develop`, el mismo check corre la completa (desde la caché) y la pesada. | **Elegida**: sin check nuevo ni cambio en los rulesets de F1-5. |
+| D4b Check `verificacion-pesada` aparte | Otro check obligatorio en `main`. | Descartada: obliga a enmendar F1-5. |
+| **D5a** Clave = árbol, perfil, huella declarada y versión del workflow | El proyecto declara su huella del entorno (herramientas, carpetas ignoradas que leen los tests). | **Elegida** (decisión 2). |
+| D5b Solo el SHA del árbol | La clave de P6 tal cual. | Descartada: ToolsaWeb ya mete en su clave herramientas y carpetas ignoradas (`exampleData`, 34 GB); con solo el árbol, cambiar de `rustc` o de datos reutilizaría un verde viejo. |
+| **D6a** No ejecutar las suites | Coste desde tiempos ya medidos en los dos repos; factor de velocidad del runner «no verificado». | **Elegida**: los dos repos son solo lectura para esta tarea. |
+
+## Pruebas de concepto
+
+Guion `poc-f1-7.sh` y su salida completa `poc-f1-7.log`, en la carpeta `poc/`
+de TASK-008 (sin enlace: la carpeta de la tarea cambia de estado al
+cerrarse). Ejecutado en Git Bash con Git 2.56.0, `gh` 2.95.0 y Python 3.12
+(`python -I`) bajo `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-008\poc\`
+(borrada al terminar); la descripción OpenAPI y las páginas de precios van a
+sus propias carpetas vacías (`esquema/`, `precios/`) y las copias mutadas del
+ADR a `mutaciones/`, nunca al repo. ToolsaWeb y OpenDockTrading **solo se
+leen** (ni se ejecutan sus suites ni se escribe en ellos). **Solo lectura en
+GitHub**: toda llamada pasa por `gh api --method GET` o es una descarga
+`GET` de un fichero público, y el guion las registra y cuenta las de otro
+método. **Las entradas y la tabla de coste de la decisión 6 se extraen de
+este fichero**, entre los marcadores `entradas-coste` y `tabla-coste`. El
+guion sale con código distinto de 0 si alguna sección falla.
+
+```bash
+bash poc-f1-7.sh "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-008/poc" \
+  docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md \
+  /c/Users/nullcad2025/Documents/sources/Personal > poc-f1-7.log 2>&1
+```
+
+```text
+RESULTADO A casos=10/10 como se esperaba
+RESULTADO B casos=10/10 ; controles negativos=8/8
+RESULTADO C1 comprobaciones=9/9
+RESULTADO C2 errores ADR real=0 ; mutaciones detectadas=5/5 ; no aplicadas=0
+RESULTADO llamadas gh api con GET=1 ; gh api con otro metodo=0 ; descargas GET=3
+```
+
+### A — Árboles en Git local
+
+Repo temporal con `main`, `develop`, ramas de tarea y merges `--no-ff` como en
+F1-3; el merge ref de GitHub (`refs/pull/N/merge`) se simula con
+`git merge-tree --write-tree <base> <head>`.
+
+```text
+  ok   head del PR al dia = merge ref
+  ok   merge --no-ff en develop = head del PR
+  ok   release (merge --no-ff develop en main) = develop
+  ok   PR desactualizado: head distinto del merge ref
+  ok   tras actualizar la rama: head = merge ref verificado antes
+  ok   tras actualizar la rama: head distinto del head anterior (arbol nuevo)
+  ok   merge en develop = merge ref del PR
+  ok   hotfix sin volver a develop: merge ref de la release distinto de develop
+  ok   hotfix devuelto a develop: merge ref de la release = develop
+  ok   release 2 (merge --no-ff) = develop
+```
+
+Lectura: con `strict` (F1-5) el PR al día, su merge commit en `develop` y la
+release en `main` tienen **el mismo árbol**; por eso un resultado guardado por
+árbol sirve a los tres. Un PR desactualizado verifica otro árbol (el del merge
+ref), y actualizar la rama crea un árbol nuevo: **la caché no ahorra esa
+segunda verificación** (divergencia con F1-5, abajo). La release solo
+coincide si cada hotfix ha vuelto a `develop` (F1-3).
+
+### B — Búsqueda por padres
+
+Simulación en Python sobre un repo Git real: un diccionario hace de API de
+check-runs (lo que devolvería `GET /commits/{sha}/check-runs?check_name=verificacion&app_id=15368&filter=latest`),
+y la búsqueda es la de la decisión 3.
+
+```text
+  ok   caso     fallo    primera verificacion del PR (merge ref nuevo): se verifica
+  ok   caso     acierto  commit vacio en la rama: el check del head anterior (abuelo del merge ref)
+  ok   caso     acierto  push del merge commit en develop: su padre (head del PR) tiene el arbol
+  ok   caso     acierto  PR de release develop->main: abuelo (head del PR de la tarea) con el mismo arbol
+  ok   caso     acierto  push del merge de la release en main
+  ok   caso     fallo    PR desactualizado: el merge ref es un arbol nuevo
+  ok   caso     acierto  tras actualizar-rama (strict): el arbol del head = merge ref verificado antes
+  ok   caso     fallo    develop avanzo y se actualiza la rama otra vez: arbol nuevo
+  ok   caso     acierto  revert al arbol del abuelo
+  ok   neg      fallo    mismo arbol a tres niveles: no se busca (limite conservador)
+  ok   neg      fallo    check verde de otro perfil (rapida)
+  ok   neg      fallo    otra huella del entorno
+  ok   neg      fallo    otra version del workflow
+  ok   neg      fallo    check en rojo
+  ok   neg      fallo    check verde de otra App con el mismo nombre
+  ok   neg      fallo    check verde con otro nombre
+  ok   neg      fallo    check verde sin arbol= en el resumen
+  ok   caso     acierto  control positivo: mismo montaje con todo igual
+```
+
+Lectura: con los padres solos no basta (divergencia con el plan): en el PR de
+release y tras un commit vacío, el check está en el **abuelo** del commit
+verificado, porque en un PR el check lo lleva el head y lo verificado es el
+merge ref, que tiene como padres la base y el head. Con dos niveles se cubren
+PR, merge en `develop` y release sin índice por árbol. Un acierto tras
+`actualizar-rama` solo ocurre si nadie integró nada entre medias.
+
+### C — Coste y esquema de la API
+
+**C1, OpenAPI.** Descripción de `github/rest-api-description`, commit
+`2a33d9cc5d1d` (2026-11-14T00:44:38Z, el mismo que usaron F1-5 y F1-6),
+`api.github.com.json` (14 274 027 bytes, OpenAPI 3.0.3, versión 1.1.4):
+
+```text
+  ok   admite check_name
+  ok   admite app_id (entero)
+  ok   admite filter con latest
+  ok   control negativo: no admite tree_sha (no hay busqueda por arbol)
+  ok   check-run trae head_sha, conclusion, app y output
+  ok   output.summary existe
+  ok   app.id existe (se compara con 15368)
+  ok   POST /check-runs acepta output.summary (como se escribe el resumen)
+  ok   GET /commits/{ref} trae parents[].sha y commit.tree.sha
+```
+
+La API no busca check-runs por árbol: de ahí la búsqueda por commits.
+
+**C2, coste.** Cada entrada de la decisión 6 se comprueba contra su fuente:
+la línea citada de ToolsaWeb u OpenDockTrading tiene que contener el valor; los
+recuentos de tareas se rehacen leyendo el campo `creado` de cada `tarea.md`
+terminada (si hay más que en el ADR, aviso: el repo ha crecido); los precios
+se buscan en las dos páginas de docs.github.com descargadas con `GET`. Luego
+se calcula la tabla y se compara celda a celda con la del ADR. Cinco copias
+mutadas del ADR (una entrada medida, una celda de la tabla, la línea de una
+fuente, un supuesto y un precio) tienen que fallar, y fallan (5/5).
+
+### Divergencias con el plan
+
+1. **Dos niveles, no solo padres** (PoC B): el commit verificado, sus padres y
+   los padres de estos.
+2. **`arbol=` es el del merge ref, no el del commit que lleva el check**: en un
+   PR, GitHub ejecuta el workflow sobre el merge ref y publica el check en el
+   head (conocimiento general, **no verificado**). Por eso el árbol va en el
+   resumen y la búsqueda compara con él.
+3. **La caché no abarata `strict`** (PoC A y B): cada `actualizar-rama` crea
+   un árbol que nadie ha verificado. Lo que ahorra es la release, la cola de
+   merge, el push del merge commit y las re-ejecuciones sobre el mismo árbol.
+   Su coste entra en la tabla como supuesto S1.
+4. **Job de búsqueda aparte en Linux**: para no despertar un runner Windows (ni
+   el propio) cuando hay acierto, `buscar` corre en `ubuntu-latest` y
+   `verificacion` elige su `runs-on` con su salida (decisión 3). Cuesta un
+   minuto Linux por evento (filas C3 y C5).
+5. **Pesada solo en la release, no en el hotfix**: el plan dice «PR a `main`
+   completa desde la caché más la pesada»; ToolsaWeb decidió que los hotfix no
+   corren los pesados (ToolsaWeb, `docs/verificacion/flujo-de-verificacion.md:299`).
+   Se mantiene: la pesada corre cuando la base es `main` y el origen es
+   `develop`.
+6. **Ritmo**: hoy hay 160 tareas terminadas en ToolsaWeb y 43 en
+   OpenDockTrading (el plan contaba 159 y 40): los repos han avanzado. La
+   ventana de OpenDockTrading acaba el 2026-11-13, día de su medida.
+7. **Precios comprobados el 2026-11-14**: Windows 2 núcleos 0,010 $/min y Linux
+   0,006 $/min, redondeo por job al minuto. La «tasa de self-hosted anunciada
+   y aplazada» del plan **no aparece** en la documentación consultada, que
+   lista los runners self-hosted entre lo que «sigue gratis»: se usa 0 $ y la
+   tasa queda como pregunta abierta.
+8. **Estimación rehecha**: ToolsaWeb en runners de GitHub sale a 13 363
+   min/mes de Windows (133,63 $), no ~12 700 (~127 $): 166 tareas al mes y el
+   redondeo por job (22 min 22 s → 23 min).
+9. **`taskctl doctor`** del plan de pruebas no lo ejecuta esta implementación
+   (se le pidió no ejecutar `taskctl`); `comprobar-repo.sh` sí, en verde.
+
+### No verificado
+
+- **Factor de velocidad** de un runner de GitHub (2 núcleos) frente a la
+  máquina de Carlos: se usa 1,0 (supuesto S2). Una completa fría de ToolsaWeb
+  en un runner de 2 núcleos, sin `target/` caliente, probablemente tarda más.
+- **Actualizaciones de rama por tarea** con `strict`: se usa 0,5 (supuesto
+  S1), sin medir.
+- **Releases al mes y duración de la pesada sola**: cero releases medidas en
+  la 2.0; la pesada de ToolsaWeb se deriva (paso 18 con pesados 29 min 01 s
+  frente a 14 min 53 s sin ellos, `flujo-de-verificacion.md:224` y `:263`:
+  unos 14 min) y corre en la máquina de Carlos, sin coste de minutos.
+- **Plan de la cuenta**: los dos repos son privados (leído con `GET`); que la
+  cuenta sea GitHub Free (12 $ incluidos al mes) es un supuesto.
+- **Que GitHub publique el check del PR en el head** con el workflow corrido
+  sobre el merge ref (divergencia 2).
+- **Nombre del check de un workflow reutilizable**: GitHub lo compone con el
+  job que llama y el llamado (`<job> / <job>`); que coincida con el
+  `verificacion` obligatorio de F1-5 lo tiene que asegurar F5-1.
+- **Cómo escribe el workflow `output.summary`** de su propio check (el
+  resumen del paso del job o un check-run propio con `GITHUB_TOKEN`): el
+  esquema lo admite (C1); el mecanismo concreto, F5-2.
+- **`runs-on` con una expresión** que depende de la salida de otro job: sintaxis
+  conocida de Actions, no ejecutada.
+- **Cola de merge**: que el commit de `merge_group` tenga el árbol del head;
+  si no, falla la caché y se verifica otra vez (caro pero seguro).
+- **Cuánto espera un job en cola por un runner self-hosted** antes de fallar.
+- **GitLab**: todo el equivalente (F1-13).
+
+## Cómo se ve en GitHub
+
+Hoy (observado): este repo solo tiene `ci.yml` con `comprobar-repo`; ToolsaWeb
+y OpenDockTrading no tienen CI de verificación: verifican en local con
+TaskCode 0.16 (RR-14: nada cambia hasta el relevo).
+
+Con este ADR aplicado (conocimiento general de GitHub, **no verificado**):
+
+| Dónde | Qué se ve |
+|---|---|
+| Checks del PR a `develop` | `verificacion` con perfil completa; en el resumen, la línea `arbol=<sha> perfil=completa clave=<k> origen=<sha>` y el tiempo; un acierto sale verde en segundos con «reutilizado de `<sha>`» |
+| Pestaña Actions de una ejecución | dos jobs: `buscar` (Linux, segundos) y `verificacion` (en el runner del perfil si no hubo acierto; en Linux y en segundos si lo hubo) |
+| PR de release `develop`→`main` | `verificacion` con «completa: reutilizado de `<sha>`» y la pesada corriendo en el runner propio |
+| Runner propio apagado | `verificacion` pendiente con «Waiting for a runner to pick up this job»; el PR no se puede mergear; `doctor` dice qué runner falta |
+| Settings → Actions → Runners (repos privados) | el runner de la máquina de Carlos con sus etiquetas (`self-hosted`, `windows`, la del proyecto) |
+| Repo público | ningún runner propio; todo en runners de GitHub |
+
+## Decisión
+
+1. **Contrato del comando del proyecto.** Un solo comando declarado por el
+   proyecto (la clave la fija F1-11), llamado con
+   `--perfil rapida|impacto|completa|pesada`. Resultado: código de salida (0
+   verde, cualquier otro rojo); la base del PR le llega en `RAMA_BASE` (como
+   hoy en ToolsaWeb, `flujo-de-verificacion.md:25`). Puede dejar un informe
+   JSON para el revisor IA (formato: F5-4; P6, el revisor no repite tests).
+   Sustituye a los seis `comando_*` (H18, RF-41). Inclusión: rápida ⊂ impacto
+   ⊂ completa; la **pesada es aparte** y no sustituye a la completa. El
+   workflow reutilizable de GitHub y la plantilla de GitLab solo llaman a ese
+   comando (P6, P11); no conocen los grupos ni los pasos del proyecto. El
+   comando corre en el shell que declare el proyecto para el runner (un
+   `.bat` exige Windows: OpenDockTrading, H16); el workflow no reinterpreta la
+   orden con otro shell.
+2. **Clave de la caché** = árbol verificado + `clave`, donde
+   `clave = sha256(perfil | huella | versión del workflow)`. La **huella** la
+   declara el proyecto como una orden que imprime lo que su suite lee fuera
+   del árbol (versiones de herramientas, contenido de carpetas ignoradas,
+   variables): es una extensión de F1-11, y ToolsaWeb ya la tiene
+   (`cache-verificacion.py`). Los pasos que no dependen solo del árbol
+   (`cargo audit` y `npm audit` de OpenDockTrading, `scripts/verificar.bat:69-70`)
+   salen del check cacheado a un job programado (detalle en F1-11): si no,
+   un acierto escondería un aviso nuevo.
+3. **Dónde vive el resultado y cómo se busca.** En el propio check
+   `verificacion` que publica GitHub Actions (`integration_id` 15368, F1-5),
+   con `arbol=`, `perfil=` y `clave=` en `output.summary`. El job `buscar`
+   (Linux) toma el commit verificado `V` (el merge ref en un PR, el commit en
+   un push), y para `V`, sus padres y los padres de estos (como mucho 7
+   commits), en ese orden, pide
+   `GET /repos/{o}/{r}/commits/{c}/check-runs?check_name=verificacion&app_id=15368&filter=latest`.
+   Es **acierto** el primero en verde cuyo `arbol` sea el árbol de `V` y cuyos
+   `perfil` y `clave` sean los actuales; con acierto, `verificacion` corre en
+   Linux y sale verde con «reutilizado de `<sha>`»; sin él, corre el comando en
+   el runner del perfil (`runs-on` según la salida de `buscar`). Ningún
+   índice por árbol, ni `actions/cache`, ni nada que el agente pueda escribir
+   (D2). Un check de otra App, de otro nombre, en rojo o sin `arbol=` no
+   cuenta (PoC B).
+4. **Perfil según el evento:**
+
+   | Evento | Perfil | Dónde corre |
+   |---|---|---|
+   | Hook local (opcional) | rápida | máquina de la persona, solo con confianza explícita en el repo (H2; F1-8, F5-6) |
+   | Rondas del agente antes de `review` | impacto | local, como la selectiva de hoy; no es un check obligatorio |
+   | PR a `develop` (todos los carriles) | completa (D1a), desde la caché si acierta | runner de la completa |
+   | PR de release `develop`→`main` | completa desde la caché + pesada, en el mismo check `verificacion` (D4a) | completa: caché; pesada: runner de la pesada |
+   | PR de hotfix a `main` | completa | runner de la completa |
+
+5. **Runner según el perfil.** Cada perfil declara su runner: `github` (con el
+   sistema: `windows` o `linux`) o una lista de etiquetas
+   (`[self-hosted, windows, <proyecto>]`), que el workflow recibe en
+   `runs-on`. **Repo público: solo runners de GitHub** (un runner propio
+   ejecutaría código de cualquier fork; F1-8). La pesada de ToolsaWeb solo en
+   la máquina de Carlos (lee datos locales como los 34 GB de `exampleData` y
+   tiene umbrales de tiempo; RN-02, F5-5). **Runner propio caído: falla
+   cerrado** (D3a): el check se queda pendiente, el PR no se mergea y `doctor`
+   dice qué runner falta; nunca se cae a otro runner ni se da por bueno.
+6. **Coste.** Ejecuciones al mes × minutos facturados por ejecución ×
+   precio, con cada entrada citada:
+
+   <!-- entradas-coste:inicio -->
+   | ID | Entrada | Valor | Fuente (fichero:línea o página, fecha) |
+   |---|---|---|---|
+   | `E1` | ToolsaWeb, completa sin pesados con su caché vacía (primera tras TASK-344) | `22 min 22 s` | `ToolsaWeb/docs/verificacion/flujo-de-verificacion.md:263` (commit `0a8b4747`) |
+   | `E2` | ToolsaWeb, completa con la caché caliente, peor caso medido | `14 min 31 s` | `ToolsaWeb/docs/verificacion/flujo-de-verificacion.md:307` (TASK-284, 2026-11-12) |
+   | `E3` | Rondas de revisión por tarea (máximo medido en ToolsaWeb; se usa también para OpenDockTrading) | `3,0` | `ToolsaWeb/docs/verificacion/flujo-de-verificacion.md:291` |
+   | `E4` | OpenDockTrading, verificación en frío (un solo perfil) | `4 min 11 s` | `OpenDockTrading/.taskcode/config.yml:8` (2026-11-13) |
+   | `E5` | ToolsaWeb, tareas terminadas | `160` | `ToolsaWeb/tareas/04-terminadas`, `creado` de `2026-10-16` a `2026-11-13` (29 días) |
+   | `E6` | OpenDockTrading, tareas terminadas (ritmo de arranque, no estable) | `43` | `OpenDockTrading/tareas/04-terminadas`, `creado` de `2026-11-09` a `2026-11-13` (5 días) |
+   | `E7` | Precio, runner de GitHub Windows 2 núcleos, por minuto | `$0.010` | docs.github.com/en/billing/reference/actions-runner-pricing (consultada el 2026-11-14) |
+   | `E8` | Precio, runner de GitHub Linux 2 núcleos, por minuto | `$0.006` | la misma página (2026-11-14) |
+   | `E9` | Runners self-hosted: no gastan minutos | `Self-hosted runners` | docs.github.com/en/billing/concepts/product-billing/github-actions, lista de lo que «remains free» (2026-11-14) |
+   | `E10` | Redondeo por job | `rounds the minutes and partial minutes each job uses up to the nearest whole minute` | docs.github.com/en/billing/reference/actions-runner-pricing (2026-11-14) |
+   | `E11` | Uso incluido en GitHub Free, por mes | `$12 USD` | docs.github.com/en/billing/concepts/product-billing/github-actions (2026-11-14) |
+   | `S1` | Actualizaciones de rama por tarea (`strict`) | `0,5` | supuesto, no medido |
+   | `S2` | Factor de velocidad del runner frente a la máquina de Carlos | `1,0` | supuesto, no verificado |
+   | `S3` | Minutos facturados del job `buscar` (y de `verificacion` con acierto) | `1` | redondeo por job (E10) |
+   <!-- entradas-coste:fin -->
+
+   Tareas al mes = terminadas × 30 / días de la ventana (redondeado);
+   ejecuciones por tarea = E3 + S1 (cada ronda empuja y cada actualización de
+   rama crea un árbol nuevo: fallos de caché); minutos facturados = minutos ×
+   S2 redondeados al minuto por arriba (E10); $/mes de cada fila redondeado a
+   céntimos; T1 y T2 restan el uso incluido (E11).
+
+   <!-- tabla-coste:inicio -->
+   | Fila | Proyecto | Perfil | Runner | Tareas/mes | Ejecuciones/tarea | Min facturados/ejecución | Min/mes | $/min | $/mes |
+   |---|---|---|---|---|---|---|---|---|---|
+   | `C1` | ToolsaWeb | completa (E1) | GitHub Windows | 166 | 3,5 | 23 | 13 363 | 0,010 | 133,63 |
+   | `C2` | ToolsaWeb | completa (E2) | self-hosted Windows (máquina de Carlos) | 166 | 3,5 | 15 | 8 715 | 0,000 | 0,00 |
+   | `C3` | ToolsaWeb | `buscar` | GitHub Linux | 166 | 3,5 | 1 | 581 | 0,006 | 3,49 |
+   | `C4` | OpenDockTrading | completa (E4) | GitHub Windows | 258 | 3,5 | 5 | 4 515 | 0,010 | 45,15 |
+   | `C5` | OpenDockTrading | `buscar` | GitHub Linux | 258 | 3,5 | 1 | 903 | 0,006 | 5,42 |
+   | `T1` | Los dos | todo en runners de GitHub (C1+C3+C4+C5 − E11) | — | — | — | — | 19 362 | — | 175,69 |
+   | `T2` | Los dos | completa de ToolsaWeb en su máquina (C2+C3+C4+C5 − E11) | — | — | — | — | 14 714 | — | 42,06 |
+   <!-- tabla-coste:fin -->
+
+   Lectura: la completa de ToolsaWeb en runners de GitHub es casi todo el
+   coste (133,63 $ de 175,69 $ al mes); en la máquina de Carlos, con la caché
+   de `target/` caliente, no gasta minutos (E9) pero la ocupa unas 145 horas
+   al mes (8 715 min). OpenDockTrading en GitHub, unos 45 $ al mes con su ritmo
+   de arranque. Dónde corre la completa de ToolsaWeb es configuración del
+   proyecto (decisión 5); la tabla da el dato para elegir. La pesada no entra:
+   corre en la máquina de Carlos (0 $) y no hay releases medidas. Lo que la
+   caché ahorra en estos repos es poco en dinero (la completa de cada release
+   y las re-ejecuciones); su valor es el tiempo de la release y de la cola, y
+   no repetir (RR-07).
+
+## Divergencias con F1-5
+
+F1-5 no se modifica (es un ADR aceptado); lo que este ADR corrige queda aquí
+y en «Consecuencias»:
+
+1. **Perfil en `develop`** (D1a): las consecuencias de F1-5 dicen «rápida en
+   `develop`, completa en `main`». Se corrige: **completa en `develop`** y
+   completa (desde la caché) más pesada en la release. La decisión 2 de F1-5
+   («el workflow elige el perfil por la base del PR») se mantiene.
+2. **La caché y `strict`**: la decisión 7 de F1-5 dice que la verificación
+   repetida tras cada merge «la abarata la caché por SHA». No: actualizar la
+   rama crea un árbol nuevo que hay que verificar (PoC A y B). La caché ahorra
+   la verificación del merge commit, la de la release y la de la cola.
+3. **`verificacion` en `main`**: F1-5 lo describe con perfil completo; aquí
+   lleva además la pesada en la release (D4a), en el mismo check, sin cambiar
+   el ruleset.
+
+## Requisitos y evidencia
+
+| Requisito | Decisión | Evidencia |
+|---|---|---|
+| RR-07 / RF-42 nada se verifica dos veces sobre el mismo árbol | resultado en el check con `arbol=` y `clave=`; búsqueda en dos niveles (decisión 3) | PoC A (10/10: PR al día = merge en `develop` = release), B (10/10 casos y 8/8 negativos), C1 (filtros `check_name`, `app_id`, `filter` y `output.summary` en el esquema); comportamiento real: no verificado (F5-2) |
+| RF-41 perfiles rápida, impacto, completa y pesada en release | un comando con `--perfil`; tabla de eventos (decisión 4) | decisiones 1 y 4; los cuatro perfiles de ToolsaWeb, en `ToolsaWeb/.taskcode/config.yml:9`, `:16`, `:18` y `:23` (solo lectura); implementación: F5-1 |
+| P6, viñeta 1: workflow reutilizable con perfiles y caché compartida por PR, cola y release | decisiones 1, 3 y 4 | PoC A y B; cola de merge: no verificado (F1-13, F2-6) |
+| P6, viñeta 2: el revisor IA lee el informe de la CI | informe JSON opcional del comando (decisión 1) | formato y uso: F5-4, F6-3; no verificado |
+| P6, viñeta 3 / RN-02: runner propio Windows por perfil | runner declarado por perfil, solo en repos privados; falla cerrado (decisión 5) | decisión 5; tabla de coste C2 y T2; implementación: F5-5 |
+| P6, viñeta 4 / H2: en local solo un hook rápido, opcional y con confianza | hook con la rápida, solo con confianza explícita (decisión 4) | modelo de confianza: F1-8; implementación: F5-6 |
+| Criterio de F1-7: coste con las suites de ToolsaWeb y OpenDockTrading | tabla de la decisión 6 | PoC C2 (11 entradas comprobadas contra su fuente, tabla igual a la calculada, 5/5 mutaciones detectadas); supuestos S1–S3 y factor de velocidad: no verificado |
+| H18 superficie de verificación | un comando en lugar de seis claves (decisión 1) | decisión 1; la clave en la configuración: F1-11 |
+| H16 el mismo comando distinto con `cmd.exe` y `sh` | el comando corre en el shell y el runner que declara el proyecto; un `.bat` exige Windows | decisiones 1 y 5; OpenDockTrading (`verificar.bat`) en runner Windows (C4) |
+| RR-15 GitHub y GitLab | el comando y la clave son neutrales; la plantilla de GitLab llama al mismo comando | no verificado (F1-13, F5-1) |
+
+## Consecuencias
+
+- **F5-1** implementa el workflow reutilizable (jobs `buscar` y
+  `verificacion`, `runs-on` por perfil) y la plantilla de GitLab, y asegura que
+  el check se llame `verificacion` como exige F1-5.
+- **F5-2** implementa la búsqueda en dos niveles y la escritura de `arbol=`,
+  `perfil=` y `clave=` en `output.summary`, y mide las repeticiones.
+- **F5-3** cambia de expectativa: la impacto no es el check de los PR (D1a);
+  queda para las rondas del agente. Hay que actualizar su fila en el plan.
+- **F5-4** fija el informe JSON para el revisor; **F5-5**, el runner de la
+  máquina de Carlos para la pesada de ToolsaWeb (y, si el proyecto lo elige,
+  la completa: fila C2); **F5-6**, el hook local.
+- **F1-8** cubre el runner propio con código del PR, `pull_request` frente a
+  `pull_request_target`, quién puede publicar un check `verificacion` con un
+  resumen falso (cualquier workflow del repo con `checks:write`) y la
+  confianza del hook.
+- **F1-11** declara en la configuración el comando, la huella y el runner por
+  perfil, y saca a un job programado los pasos que no dependen solo del
+  árbol.
+- **F1-13** y **F2-6** verifican la cola de merge (árbol de `merge_group`) y el
+  equivalente en GitLab.
+- **F3-8** (`doctor`) dice qué runner propio falta cuando un check se queda
+  pendiente.
+- **F9**: `taskctl roadmap --comparar` de OpenDockTrading deja de tener
+  sentido con el roadmap fuera del árbol (F1-1); lo trata la migración.
+- **Este repo no cambia** (RR-14): `ci.yml` y `comprobar-repo` siguen igual.
+
+## Fuera de alcance
+
+- Seguridad del runner propio, secretos y eventos del workflow: F1-8.
+- Claves de configuración y extensiones (comando, huella, runner): F1-11.
+- Equivalencias en GitLab y la cola de merge: F1-13.
+- Implementación y medidas reales: F5-1 a F5-6, F2-6.
+
+## Preguntas abiertas
+
+Este ADR se planificó con un solo rol (arquitectura). Quedan sin cubrir, como
+riesgos sin prueba:
+
+- **Modos de fallo de la estimación**: el factor de velocidad del runner (S2;
+  una completa fría de ToolsaWeb en 2 núcleos puede tardar bastante más que en
+  la máquina de Carlos), el ritmo de OpenDockTrading con cinco días de datos
+  (E6) y las rondas de ToolsaWeb aplicadas a OpenDockTrading (E3). Se rehace
+  con la primera semana de F5-1 en uso.
+- **Tasa por minuto de los runners self-hosted**: el plan la daba por
+  anunciada y aplazada; la documentación consultada el 2026-11-14 no la
+  menciona. Si llega, C2 deja de ser 0 $.
+- **Casos borde de la búsqueda**: un mismo árbol a tres niveles o más no se
+  reutiliza (conservador); varias ejecuciones del mismo check en el mismo
+  commit (`filter=latest` toma la última); un check en verde publicado por
+  un workflow modificado en el propio PR (F1-5, F1-8).
+- **Reglas de negocio de cada perfil**: qué entra en la rápida y en la
+  impacto lo decide cada proyecto; este ADR solo fija la inclusión.
+- **Cola de merge**: si el árbol del commit de `merge_group` no coincide con
+  el del head, se verifica otra vez.
+- **Seguridad del runner propio con código del PR**: F1-8.
+- **GitLab**: si la plantilla puede buscar el resultado en los pipelines de
+  los padres como aquí en los check-runs (F1-13).
diff --git a/docs/adr/README.md b/docs/adr/README.md
index f5f3899..c4a15af 100644
--- a/docs/adr/README.md
+++ b/docs/adr/README.md
@@ -14,6 +14,7 @@ directo a la tabla de la Fase 1 y a su tarea.
 | [F1-4](F1-4-operaciones-git-y-contrato-de-salida.md) | Capa de operaciones Git/GitHub deterministas: lista cerrada, contrato de salida, códigos y objetivo de tokens por tarea | TASK-005 | aceptada |
 | [F1-5](F1-5-puertas-en-github.md) | Puertas en GitHub por carril: rulesets por rama y tag, checks obligatorios con su origen, check `carril`, CODEOWNERS, cola y rama al día | TASK-006 | aceptada |
 | [F1-6](F1-6-identidades-agente-revisor-persona.md) | Identidades del agente implementador, del revisor IA y de la persona: operación → identidad, permisos mínimos de dos GitHub Apps, contrato del firmante y por qué ninguna IA aprueba ni mergea | TASK-007 | aceptada |
+| [F1-7](F1-7-verificacion-ci-perfiles-cache-runners.md) | Verificación en CI: un comando con perfiles, caché por árbol en el check `verificacion` con búsqueda en dos niveles de padres, perfil por evento, runner por perfil (self-hosted solo en privados, falla cerrado) y coste con ToolsaWeb y OpenDockTrading | TASK-008 | aceptada |
 
 ## Plantilla
````

## Excluido del diff (5 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-008/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{01-en-diseno => 02-en-curso}/TASK-008/planificacion/plan-final.md                 |   0
 tareas/02-en-curso/TASK-008/poc/poc-f1-7.log                                              | 214 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/02-en-curso/TASK-008/poc/poc-f1-7.sh                                               | 390 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{01-en-diseno => 02-en-curso}/TASK-008/tarea.md                                    |   7 +-
 5 files changed, 610 insertions(+), 1 deletion(-)
````

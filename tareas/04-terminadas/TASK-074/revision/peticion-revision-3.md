# Peticion de revision — TASK-074 (ronda 3)

- Tarea: TASK-074 — F1-16 ADR lotes desatendidos con revision humana al final
- Rama revisada: feature/task-074-f1-16-adr-lotes-desatendidos-con-revisio
- Rama base: develop
- Commit revisado (HEAD): e4490f72fa66678ada8a660462e4f1df76b24e2d
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-074 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 08a98c8469db6c3fcb71fd4c7f663215018fc470 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (16 hallazgo(s)))

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

- tareas/03-en-revision/TASK-074/tarea.md
- tareas/03-en-revision/TASK-074/planificacion/
- docs/adr/F1-16-lotes-desatendidos.md
- docs/adr/README.md

## Hallazgos de la ronda 2 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 10 hallazgo(s) MENOR de la ronda 2 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-16-lotes-desatendidos.md:516 y :539 (`carril-subio`) y tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:434 (`siguiente_paso`) y :1889 (C9) | informe-revision-2.md |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-16-lotes-desatendidos.md:567 y tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:343 (`comprobar_primer_padre`) y :1834 (C5c) | informe-revision-2.md |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-16-lotes-desatendidos.md:468 y :538 y tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:283 (`nucleo_tarea`) y :1806 (C6c) | informe-revision-2.md |
| IMP-4 | IMPORTANTE | corregido | docs/adr/F1-16-lotes-desatendidos.md:506 y :580 y tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:1092 y :1901 (C10, C10b) | informe-revision-2.md |
| IMP-5 | IMPORTANTE | corregido | docs/adr/F1-16-lotes-desatendidos.md:645 y tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:862 (`develop_agente`) y :1932 (C11) | informe-revision-2.md |
| NUE-1 | IMPORTANTE | corregido | docs/adr/F1-16-lotes-desatendidos.md:468-478 y tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:279-292 (`CLAVES_NUCLEO`, `SECCIONES_NUCLEO`) | informe-revision-2.md |

## Commits a revisar (git log 08a98c8469db6c3fcb71fd4c7f663215018fc470..HEAD)

````
e4490f7 chore(TASK-074): ronda extra 3 justificada
d574e7f docs(adr): F1-16 corrige hallazgos de la revision 2
f01c1ad chore(TASK-074): informe de revision 2 (cambios solicitados) y coste
f04b85d chore(TASK-074): peticion de revision ronda 2
````

## Diff desde la ronda anterior (git diff 08a98c8469db6c3fcb71fd4c7f663215018fc470..HEAD)

````diff
diff --git a/docs/adr/F1-16-lotes-desatendidos.md b/docs/adr/F1-16-lotes-desatendidos.md
index fc5be21..19a1e52 100644
--- a/docs/adr/F1-16-lotes-desatendidos.md
+++ b/docs/adr/F1-16-lotes-desatendidos.md
@@ -27,7 +27,11 @@ y solo 3 de 17 tareas tenían el `approve` firmado por una persona. La
 revisión 1 añadió cuatro cosas que el diseño no cubría (carril del diff,
 árbol del merge, núcleo de `tarea.md`, escrituras concurrentes con el
 agente) y corrigió una afirmación que no se sostenía en los repos privados
-gratuitos: «ninguna IA mergea en `develop`» (divergencia 1).
+gratuitos: «ninguna IA mergea en `develop`» (divergencia 1). La revisión 2
+cambió el núcleo del manifiesto de lista blanca a exclusión (todo menos lo
+que escribe el CLI, medido), alineó el carril del diff con F1-10, exigió que
+`ajeno-aceptado` lo firme una persona y que revertir desbloquee solo, y
+llevó a «No verificado» el árbol del merge real.
 
 ## Contexto
 
@@ -131,6 +135,13 @@ queda fuera: no es desatendida, hubo 51 commits directos de la persona en
 reales de ToolsaWeb, 3 de código. El orden de `planificar` (dependencias,
 empate por ID) coincide con el de `develop`.
 
+**Lo que escribe el CLI de la 0.16 en `tarea.md`** (para el núcleo del
+manifiesto, decisión 1): en los commits `chore(TASK-NNN)` de las 17 tareas
+cambian solo las claves `estado`, `actualizado`, `asignado_a`,
+`plan_aprobado` y `tokens_*` y las secciones `## Transiciones`, `## Coste` y
+`## Resultado`; los 2 cambios que hacen en `## Criterios de aceptacion` son
+solo marcas de casilla; fuera de esa lista, ninguno.
+
 <!-- tabla-lote:inicio -->
 | Tarea | Tipo | Carril efectivo | Dependencias | Rondas reales | Rondas con tope 2 | Bloqueantes en la ronda 2 | MENOR sin corregir (tope 2) | Tokens de revisión (tope 2) | Tokens de las rondas 3 y 4 | approve en la 0.16 |
 |---|---|---|---|---|---|---|---|---|---|---|
@@ -276,6 +287,11 @@ agente, con un hilo que escribe en el worktree mientras el agente escribe:
   **C4c**, la salida de C4: un `--reanudar` sin decidir nada vuelve a parar;
   cuando la persona acepta el commit ajeno con un evento `ajeno-aceptado` con
   su SHA, `--reanudar` termina las tres y el informe nombra el commit.
+  **C4d**: el mismo evento con `decidido_por: agente` (lo puede escribir
+  cualquiera con escritura en la rama de estado) se rechaza: el lote sigue
+  parado y el informe lo dice. **C4e**: si la persona revierte el commit ajeno
+  (un commit suyo que deja el árbol de antes), `--reanudar` termina sin
+  ningún evento.
 - **C5**: push directo a `lote/*` después de la última tarea: el lote para y
   el PR final no se abre. **C5b**: un merge falsificado con trailer
   `Tarea: TASK-210` y committer de la plataforma: lo para la comprobación del
@@ -289,12 +305,17 @@ agente, con un hilo que escribe en el worktree mientras el agente escribe:
   dos casos. **C6c**: en el mismo sublote, el CLI escribe en cada `tarea.md`
   lo suyo (estado, `asignado_a`, transiciones, `ultimo_commit_revisado`); los
   tres blobs cambian y ninguna tarea se aparta, porque el manifiesto fija el
-  núcleo y no el blob.
+  núcleo y no el blob. **C6d y C6e**: una sección `## Escenarios` nueva, y
+  `carril` y `roles` añadidos al frontmatter, a mitad de TASK-205: se aparta
+  con `manifiesto-cambiado` en los dos casos (el núcleo cubre todo lo que no
+  escribe el CLI).
 - **C7**: el head de TASK-187 se mueve entre la guarda y el merge: la
   plataforma rechaza el `sha` (409) y TASK-187 se aparta como
   `interferida-cas` con TASK-205. **C7b**: lo que se mueve es `lote/*`: la API
   no compara la base, el merge entra, y en la vuelta siguiente la
   comprobación del primer padre para el lote; el PR final no se abre.
+- **C9b**: la misma subida con `.GitHub/workflows/x.yml` (mayúsculas) y
+  `.gitlab-ci.yml`: TASK-187 se aparta con `carril-subio`.
 - **C9**: el trabajo de TASK-187 (un `fix`) trae además
   `.github/workflows/evil.yml`: antes de la ronda 1 el carril del diff sale
   riesgo alto y TASK-187 se aparta con `carril-subio`, con TASK-205; el
@@ -331,16 +352,21 @@ agente, con un hilo que escribe en el worktree mientras el agente escribe:
 | C4 | commit ajeno empujado a `lote/*` sin ruleset | sí | antes de la tarea siguiente | el lote para |
 | C4b | el mismo push con el ruleset impuesto | sí | en el push | lo rechaza la plataforma; el lote sigue |
 | C4c | la persona acepta el commit ajeno de C4 (`ajeno-aceptado`) | no aplica | al reanudar | el lote termina y el informe nombra el commit |
+| C4d | `ajeno-aceptado` que no firma una persona de la lista | sí | al reanudar | se rechaza; el lote sigue parado |
+| C4e | la persona revierte el commit ajeno de C4 | no aplica | al reanudar | el lote termina sin evento |
 | C5 | push directo a `lote/*` tras la última tarea | sí | antes del PR final | el lote para sin abrir el PR |
 | C5b | merge falsificado con trailer `Tarea:` | sí | antes del PR final | el lote para sin abrir el PR |
 | C5c | merge con los mismos padres, trailer y committer y otro árbol | sí | antes del PR final | el lote para sin abrir el PR |
 | C6 | `tarea.md` cambiado antes de empezar | sí | antes de empezar la tarea | la tarea se aparta (manifiesto) |
 | C6b | `tarea.md` cambiado a mitad de la tarea | sí | antes de `review` | la tarea se aparta (manifiesto) |
 | C6c | escrituras legítimas del CLI en `tarea.md` (estado, transiciones) | no aplica | en cada vuelta | no apartan: el núcleo no cambia |
+| C6d | una sección `## Escenarios` nueva en `tarea.md` | sí | antes de `review` | la tarea se aparta (manifiesto) |
+| C6e | `carril` y `roles` añadidos al frontmatter | sí | antes de `review` | la tarea se aparta (manifiesto) |
 | C7 | head de la tarea movido entre la guarda y el merge | sí | en el merge (CAS del `sha`) | la tarea se aparta como interferida con su dependiente |
 | C7b | `lote/*` movido entre la guarda y el merge | sí | en el paso siguiente | el lote para sin abrir el PR |
 | C8 | segundo lanzamiento con el mismo nombre | sí | al reservar | rechazado por el CAS de la rama de estado |
 | C9 | el diff del agente trae un workflow | sí | antes de `review` | la tarea se aparta (`carril-subio`) con su dependiente |
+| C9b | el diff trae `.GitHub/workflows/` y `.gitlab-ci.yml` | sí | antes de `review` | la tarea se aparta (`carril-subio`) |
 | C10 | escritura concurrente con el agente en otro fichero | sí | antes del commit del CLI | la tarea se aparta (`escritura-no-declarada`) |
 | C10b | escritura concurrente con el agente en un fichero que él también escribe | no | no se detecta | entra como trabajo del agente y llega a la revisión |
 | C11 | la App del agente mueve `develop` | sí | antes del PR final | el lote para sin abrir el PR |
@@ -355,7 +381,7 @@ lote. El de A2 (tope 2), tal como lo imprime la PoC, recortado:
 ```text
 # Informe del lote `noche-173-210`
 
-- Base: `develop` en `389e193d7d`; manifiesto `d3156af99b48`; tope de rondas 2.
+- Base: `develop` en `389e193d7d`; manifiesto `a2cbea54ab0a`; tope de rondas 2.
 - Tareas en el manifiesto: 17; terminadas: 15; apartadas: 2 (0 en cascada); fuera desde el lanzamiento: 0.
 - Tokens de revisión: 120203077; tiempo de agente y revisión: 38:20.
 
@@ -466,16 +492,21 @@ taskcodetwo/tareas: lotes/noche-173-210.yml (diario), tareas/TASK-NNN/tarea.md (
    - **Manifiesto fijado al lanzar**: lista y orden, el **núcleo** de cada
      tarea, carril, tope de rondas por tarea, presupuesto y medianas de una
      ronda (tokens y tiempo). Va en el primer evento del diario. El núcleo es
-     un hash de lo que define la tarea y nadie debe cambiar durante la noche:
-     las claves `titulo`, `tipo`, `dependencias` y `etiquetas`, las secciones
-     `## Objetivo` y `## Criterios de aceptacion` de `tarea.md` y el
-     `plan-final.md` aprobado. No entra lo que el CLI escribe en cada
-     transición (F1-9): `estado`, `asignado_a`, `actualizado`,
-     `ultimo_commit_revisado`, `tokens_*`, `## Transiciones`, `## Coste` y
-     `## Resultado`. El lote nunca decide con esos campos (decide con el
-     diario y con Git), así que cambiarlos no le cambia nada; el blob entero
-     no vale, porque el propio CLI lo reescribe en cada `start` y cada
-     `review`.
+     un hash de **todo** `tarea.md` y `plan-final.md` **menos** la lista
+     cerrada de lo que escribe el propio CLI: las claves `estado`,
+     `actualizado`, `asignado_a`, `plan_aprobado`, `tokens_diseno`,
+     `tokens_implementacion`, `tokens_revision`, `ultimo_commit_revisado` y
+     `rama`, las secciones `## Transiciones`, `## Coste` y `## Resultado`, y la
+     marca de las casillas de los criterios (`- [x]`). La lista sale de lo que
+     escriben de verdad los commits del CLI de la 0.16 (PoC L), más
+     `ultimo_commit_revisado` y `rama`, que la 2.0 escribe en `start` y
+     `review` (F1-9). Todo lo demás (objetivo, criterios, `## Escenarios`,
+     `carril`, `roles`, `estimacion`, dependencias en cualquier formato, una
+     sección nueva) entra, así que cambiarlo aparta la tarea. El lote nunca
+     decide con lo excluido (decide con el diario y con Git), así que
+     cambiarlo no le cambia nada; el blob entero no vale, porque el propio
+     CLI lo reescribe en cada `start` y cada `review`. Si el CLI empieza a
+     escribir otra clave, se añade a la lista en el mismo cambio.
    - **Reserva** de `lotes/<nombre>.yml` en `taskcodetwo/tareas` con el CAS de
      F1-2 (un segundo lanzamiento con el mismo nombre falla) y `asignado`
      de cada tarea al lote, para que un compañero no haga `start` sobre ella.
@@ -510,9 +541,12 @@ taskcodetwo/tareas: lotes/noche-173-210.yml (diario), tareas/TASK-NNN/tarea.md (
      el resultado después (veredicto, CRÍTICO/IMPORTANTE abiertos, MENOR sin
      corregir, tokens, tiempo, `verificacion`).
    - antes de cada `ronda` y antes de integrar, el CLI recalcula el **carril
-     del diff** de la tarea (rutas de la base anotada al head, con la regla de
-     F1-5 4.2 y F1-10 decisión 5: solo `docs/` es errata; `.github/`,
-     `.taskcode/` y `CODEOWNERS` son riesgo alto). Si sube por encima del
+     del diff** de la tarea (rutas de la base anotada al head) con la misma
+     función de carril mínimo de F1-10, no con una copia: solo `docs/` es
+     errata; `.github` y `.taskcode` (carpeta, fichero o enlace, sin
+     distinguir mayúsculas y con `./` normalizado), cualquier `CODEOWNERS` y
+     las `rutas_carril` de la configuración son riesgo alto; y, para GitLab,
+     `.gitlab-ci.yml` y `.gitlab/` también (divergencia 11). Si sube por encima del
      carril del manifiesto, la tarea se aparta con `carril-subio` (en lote no
      hay persona que decida la «subida de carril» de F1-10).
    - `integrar`: `integrar-en-lote` (decisión 5).
@@ -543,7 +577,7 @@ taskcodetwo/tareas: lotes/noche-173-210.yml (diario), tareas/TASK-NNN/tarea.md (
    | `presupuesto-de-tarea` | aparta | la tarea pasaría su tope de tokens con otra ronda mediana |
    | `presupuesto-de-lote` | para | queda menos que una ronda mediana de tokens o de tiempo; no se empieza la ronda |
    | `entorno-no-responde` | para | límite de uso del proveedor o red; sin apartar; la ronda cortada se repite entera y sus tokens cuentan |
-   | `lote-con-commits-ajenos` | para | el primer padre de `lote/*` tiene algo que no es un merge del lote con lo revisado y su árbol; salida: la persona revierte el commit o lo acepta con `ajeno-aceptado` y `--reanudar` |
+   | `lote-con-commits-ajenos` | para | el primer padre de `lote/*` tiene algo que no es un merge del lote con lo revisado y su árbol; salidas: la persona lo revierte (desbloquea solo) o lo acepta con un `ajeno-aceptado` que firma ella, y `--reanudar` |
    | `develop-movido-por-agente` | para | el primer padre de `develop` desde la base tiene un commit de la App del agente; el PR del lote no se abre |
    | `token-de-persona-con-a5-abierto` | no lanza | D2a |
    <!-- tabla-paradas:fin -->
@@ -567,7 +601,15 @@ taskcodetwo/tareas: lotes/noche-173-210.yml (diario), tareas/TASK-NNN/tarea.md (
    tarea, una sola vez, y **el árbol que da `git merge-tree` de sus dos
    padres** (sin eso, un merge con los padres, el trailer y el committer
    correctos y otro árbol pasaría, C5c). Lo demás para el lote y el PR final no
-   se abre. Donde la plataforma lo da, se exige además
+   se abre. Que el motor de merge de GitHub y de GitLab dé el mismo árbol
+   que `git merge-tree` local en un merge sin conflictos no está verificado
+   (es de F2-6). Si no coincide, la guarda falla cerrado: para el lote y el
+   informe lista las rutas en que difieren los dos árboles; no se acepta un
+   merge por parecido. Si F2-6 mide que una plataforma da sistemáticamente
+   otro árbol (por ejemplo, por finales de línea o renombrados), la
+   comparación pasa a ser fichero a fichero sobre las rutas que cambia la
+   tarea, más que ninguna otra ruta difiera del primer padre; esa regla se
+   escribiría en F2-6 con su medida. Donde la plataforma lo da, se exige además
    `verification.verified` del merge (no verificado). El ruleset
    `taskcodetwo-lote` se aplica donde la plataforma lo imponga.
 7. **Interferencia (D4a)**: se impide lo portable (agente sin stdin, worktree
@@ -597,10 +639,15 @@ taskcodetwo/tareas: lotes/noche-173-210.yml (diario), tareas/TASK-NNN/tarea.md (
    `ronda-iniciada` sin resultado se repite entera, nunca da verde, y cuenta
    una ronda mediana de tokens y de tiempo. **`lote-con-commits-ajenos` no se
    desbloquea solo**: `--reanudar` vuelve a parar hasta que la persona
-   decide. Tiene dos salidas: revertir el commit ajeno con un PR suyo contra
-   `lote/*` (que entra como cualquier merge de persona y se acepta por SHA), o
-   aceptarlo tal cual con un evento `ajeno-aceptado` firmado que nombra sus
-   SHA; en los dos casos el informe lo dice (C4c).
+   decide. Tiene dos salidas: **revertirlo**: si a continuación del commit
+   ajeno hay un commit de una persona de la lista de F1-6 que deja el árbol
+   como estaba antes de él, la guarda salta la secuencia entera (su efecto
+   neto es cero) sin ningún evento (C4e); o **aceptarlo** tal cual con un
+   evento `ajeno-aceptado` que nombra sus SHA, con `decidido_por: persona` y
+   escrito por una persona de esa lista (en la 2.0, el commit de estado que
+   lo contiene tiene que estar firmado por ella, como el `approve` de F1-6;
+   en la PoC se comprueba el campo `por` contra la lista). Un
+   `ajeno-aceptado` de otra identidad se rechaza y el informe lo dice (C4d).
 9. **Presupuesto**: por lote (tokens y tiempo) y por tarea (tokens), medido
    por el proceso del CLI y nunca por un fichero que pueda escribir el
    agente. No se empieza una ronda (ni una tarea) si lo que queda es menos
@@ -698,7 +745,10 @@ taskcodetwo/tareas: lotes/noche-173-210.yml (diario), tareas/TASK-NNN/tarea.md (
     de persona queda solo para el PR del lote), tope de rondas → `preguntar`
     (en el lote, D6a: aparta o integra con los MENOR al informe) y subida de
     carril → `detener` (en el lote, `carril-subio` aparta). `tc2 lote` no
-    consulta `accion` para esas tres; para lo demás, sí.
+    consulta `accion` para esas tres; para lo demás, sí. Además, el carril
+    mínimo de F1-10 no nombra `.gitlab-ci.yml` ni `.gitlab/`; este ADR los
+    añade como riesgo alto (son los workflows de GitLab, F1-13), y la
+    función de F1-10 los tiene que incluir.
 12. **Plan del unificador, CAS de la base**: el plan pedía integrar «solo si
     `lote/*` sigue en el SHA anotado». Ni GitHub ni GitLab comparan la base
     en el merge por API, solo el `sha` del head; la base la comprueba el CLI
@@ -762,6 +812,16 @@ taskcodetwo/tareas: lotes/noche-173-210.yml (diario), tareas/TASK-NNN/tarea.md (
   simulada. En particular, que un ruleset con `pull_request` de **0 aprobaciones**
   deje mergear por API a la App del agente, y que el merge
   rechace un `sha` distinto (documentado, no ejecutado). Es de F2-6.
+- **El árbol del merge real**: que el merge por API de GitHub y de GitLab dé
+  el mismo árbol que `git merge-tree` local (en la PoC la Plataforma simulada
+  y la guarda usan la misma función, así que coinciden por construcción).
+  Caso a ejecutar en F2-6: un merge por API de una rama de tarea con un
+  renombrado, comparando `git rev-parse <merge>^{tree}` con
+  `git merge-tree --write-tree <p1> <p2>`. Si no coincide, la guarda para el
+  lote (decisión 6).
+- **La firma de `ajeno-aceptado`**: la PoC compara el campo `por` con la
+  lista de personas; que el commit de estado lo firme una persona (F1-6) no
+  se ha probado.
 - **Firma del merge**: en GitHub un merge por API lo firma la plataforma
   (`verification.verified`); la PoC compara el committer, que en local se
   puede falsificar (C5b y C5c los detecta el SHA revisado y el árbol, no la
diff --git a/docs/adr/README.md b/docs/adr/README.md
index 3e72f76..ed80076 100644
--- a/docs/adr/README.md
+++ b/docs/adr/README.md
@@ -23,7 +23,7 @@ directo a la tabla de la Fase 1 y a su tarea.
 | [F1-13](F1-13-adaptador-de-plataforma.md) | Adaptador de plataforma GitHub y GitLab: tabla de 20 garantías con su tier, verificada contra la documentación oficial con fecha y con GET de solo lectura donde se pudo (GitLab: 11 en Free, 6 en Premium y 3 en ningún tier, entre ellas el origen de un check, G09, que ni Ultimate da); sustitutos declarados que `doctor` dice siempre, contrato de `capacidades` con el plan observado, identidades en GitLab y misma batería para los dos adaptadores; tiers de pago y self-managed no verificados | TASK-015 | aceptada |
 | [F1-14](F1-14-distribucion-e-instalacion.md) | Distribución e instalación: CLI en Rust por la regla D1a (gana el 1.º y el 2.º, medidos, y el 4.º, un juicio sobre postject; sin el 4.º la regla no decide) frente a TypeScript con Node SEA (mismo comando, salida idéntica sobre las 404 tareas de ToolsaWeb y las 96 de OpenDockTrading; más rápido en caliente en los 3 repos y 32,1 veces más pequeño), TLS con el almacén del sistema, lanzador sin runtime con `cli.lock` y descarga comprobada por sha256, compatibilidad por protocolo con `tc2 version --json`, canales y firma por etapas (hash y attestation en la 2.0; Authenticode y Developer ID al entrar en winget y Homebrew) y contrato del manifiesto de recursos (`tc2 doctor --recursos`: obligatorio, necesario, conveniente y estorba); Linux y macOS no verificados | TASK-014 | aceptada |
 | [F1-15](F1-15-seleccion-de-skills-agentes-y-modelo.md) | Selección de skills, agentes y modelo por tarea y por rol: función pura `seleccionar` que interpreta tres tablas entre marcadores (reglas, modelo por carril y complejidad, dominios con lista cerrada que la config solo amplía), solo entre lo instalado (inventario de F3-9 y metadatos en la propia skill o en `recursos:`), empate por id sin modelo, suelo de 2 roles con `puertas-seguridad` (17 de 33 IMPORTANTES de la Fase 1 fueron de esa clase; detector con recall 7/8 y precisión 7/9 dentro de muestra), texto de la tarea no confiable que solo sube desde `approve`, revisora del dominio o aviso medido con la config de la base, anotación en `roles.<rol>` y columna «Clase» para medir; 10 tareas reales de ToolsaWeb y OpenDockTrading que Carlos comprobó a mano («Coincide las 10») | TASK-073 | aceptada |
-| [F1-16](F1-16-lotes-desatendidos.md) | Lotes desatendidos con revisión humana al final: `tc2 lote` determinista con núcleo puro (`planificar`, `siguiente_paso`), rama `lote/<nombre>`, operación `integrar-en-lote` (merge por API de la App del agente con el `sha` revisado como CAS) y ruleset `taskcodetwo-lote`, comprobación del primer padre con el árbol de cada merge (D3a), carril del diff recalculado, manifiesto con el núcleo de cada tarea, guarda de procedencia antes de `review` y de integrar, escrituras del agente declaradas (la de la persona en el mismo fichero no se detecta), diario `lotes/<nombre>.yml` reconciliado desde Git para `--reanudar`, presupuesto por lote y por tarea, tope de rondas sin ronda extra automática, `[DECIDE CARLOS]` que no entra y nunca para el lote, informe de la mañana en el PR del lote y equivalente en GitLab; «ninguna IA mergea en `develop`» no garantizado en privados gratuitos, solo detectado después; probado con 17 tareas reales de ToolsaWeb sobre una copia y una Plataforma simulada (GitHub y GitLab reales no verificados) | TASK-074 | aceptada |
+| [F1-16](F1-16-lotes-desatendidos.md) | Lotes desatendidos con revisión humana al final: `tc2 lote` determinista con núcleo puro (`planificar`, `siguiente_paso`), rama `lote/<nombre>`, operación `integrar-en-lote` (merge por API de la App del agente con el `sha` revisado como CAS) y ruleset `taskcodetwo-lote`, comprobación del primer padre con el árbol de cada merge (D3a), carril del diff recalculado, manifiesto con el núcleo de cada tarea (todo `tarea.md` y el plan menos lo que escribe el CLI), salida firmada por la persona de un commit ajeno en `lote/*`, guarda de procedencia antes de `review` y de integrar, escrituras del agente declaradas (la de la persona en el mismo fichero no se detecta), diario `lotes/<nombre>.yml` reconciliado desde Git para `--reanudar`, presupuesto por lote y por tarea, tope de rondas sin ronda extra automática, `[DECIDE CARLOS]` que no entra y nunca para el lote, informe de la mañana en el PR del lote y equivalente en GitLab; «ninguna IA mergea en `develop`» no garantizado en privados gratuitos, solo detectado después; probado con 17 tareas reales de ToolsaWeb sobre una copia y una Plataforma simulada (GitHub y GitLab reales no verificados) | TASK-074 | aceptada |
 | [F1-18](F1-18-hito-de-decisiones.md) | Hito de decisiones (ADR) en `init` y en `actualizar`: estudio de adopción que lo dimensiona (bloque «uso medido» del informe de F1-12 y `proponer_hito` con un elemento por hueco, bloqueantes antes de migrar), proporcionalidad con mediana y p75 de la Fase 1 (45,5–50 min y 13,9–19,1 M tokens por ADR), registro `hito-decisiones.yml` en `taskcodetwo/tareas` cuya lista, descartes y «no necesario» firma una persona, con estado calculado que falla cerrado, y puerta determinista en `tc2 start` y en el check `estado` (código `rechazada`) | TASK-094 | aceptada |
 
 ## Plantilla
````

## Excluido del diff (5 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 08a98c8469db6c3fcb71fd4c7f663215018fc470..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/03-en-revision/TASK-074/poc/poc-f1-16.log               |  30 ++++++-
 tareas/03-en-revision/TASK-074/poc/poc-f1-16.py                | 259 ++++++++++++++++++++++++++++++++++++++++++++++-------
 tareas/03-en-revision/TASK-074/revision/informe-revision-2.md  | 127 ++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-074/revision/peticion-revision-2.md | 632 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-074/tarea.md                        |   8 +-
 5 files changed, 1018 insertions(+), 38 deletions(-)
````

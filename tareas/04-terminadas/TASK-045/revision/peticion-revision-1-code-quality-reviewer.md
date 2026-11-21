# Peticion de revision — TASK-045 (ronda 1)

- Tarea: TASK-045 — F5-5 Runner self-hosted Windows por perfil
- Rama revisada: feature/task-045-f5-5-runner-self-hosted-windows-por-perf
- Rama base: develop
- Commit revisado (HEAD): addfcb2da94056304689989a9516567c77a7c466
- Fecha: 2026-11-21
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-045 (criterios de aceptacion y plan)
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
informe de esta ronda (informe-revision-1-code-quality-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-045/tarea.md
- tareas/03-en-revision/TASK-045/planificacion/
- .github/workflows/verificacion.yml
- docs/adr/F1-8-seguridad-modelo-de-amenazas.md
- docs/ci/verificacion-reutilizable.md

## Commits a revisar (git log develop..HEAD)

````
addfcb2 fix(TASK-045): huella local por la entrada de sha256sum
a3c934e fix(TASK-045): el runner propio exige jq y la guia da el alta real
c471af0 style(TASK-045): formato del shell del aislamiento
20484ad fix(TASK-045): aislamiento con ExecutionPolicy Bypass en el runner propio
2d49e6e feat(TASK-045): guia del runner propio y adenda F5-5 en F1-8
ff4227d feat(TASK-045): whoami por ruta y casos de base y evento sueltos
bc186bd feat(TASK-045): runner propio solo en la release y aislamiento en ejecutar
d703964 chore(TASK-045): tarea en curso
````

## Diff de tu dominio (3 fichero(s) de 4; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/.github/workflows/verificacion.yml b/.github/workflows/verificacion.yml
index 7788311..9e15ad7 100644
--- a/.github/workflows/verificacion.yml
+++ b/.github/workflows/verificacion.yml
@@ -9,9 +9,10 @@
 #     lineas `arbol=` candidatas de cada perfil (F1-7, decision 3; F5-2). No
 #     ejecuta nada del PR: solo lee el arbol con git. Con `cola-de-merge`
 #     (TASK-043, F5-3) el PR corre la impacto, que nunca se cachea, y el
-#     merge_group la completa.
+#     merge_group la completa. Un perfil con runner propio solo pasa en la
+#     release de un repo privado (TASK-045, F5-5; F1-8, decision 6).
 #   - ejecutar (ejecuta, matriz por perfil, runner del perfil, sin secretos):
-#     calcula la huella local, y si un candidato tiene la misma se salta el
+#     en un runner propio comprueba antes su aislamiento; despues calcula la huella local, y si un candidato tiene la misma se salta el
 #     comando (paso `skipped`); si no, corre `comando argumentos-<perfil>` en
 #     el shell que declara el proyecto y sube el informe JSON de `TC2_INFORME`
 #     (F5-4) como artefacto `informe-verificacion-<perfil>`, si se escribio.
@@ -150,6 +151,48 @@ jobs:
           done
           echo "perfiles=[$lista]" >> "$GITHUB_OUTPUT"
           echo "Perfiles de este evento ($EVENTO, cola-de-merge=$COLA): [${lista}]"
+      # F1-8, decision 6 (D3b), y adenda F5-5: un perfil con runner propio
+      # (etiqueta `self-hosted`, o un runs-on que no es cadena ni lista) solo
+      # corre en la release de develop a main de un repo privado, con el head
+      # en el mismo repo. Si no, falla cerrado antes de encolar nada.
+      - name: Runner propio
+        env:
+          PERFILES: ${{ steps.perfiles.outputs.perfiles }}
+          ENTRADAS: ${{ toJSON(inputs) }}
+          EVENTO: ${{ github.event_name }}
+          PRIVADO: ${{ github.event.repository.private }}
+          BASE: ${{ github.base_ref }}
+          ORIGEN: ${{ github.head_ref }}
+          REPO: ${{ github.repository }}
+          REPO_HEAD: ${{ github.event.pull_request.head.repo.full_name }}
+        run: |
+          set -euo pipefail
+          propios=""
+          for p in $(printf '%s' "$PERFILES" | jq -r '.[]' | tr -d '\r'); do
+            es=$(printf '%s' "$ENTRADAS" | jq -r --arg p "$p" '.["runner-" + $p] | fromjson | if type == "string" then ascii_downcase == "self-hosted" elif type == "array" then any(.[]; tostring | ascii_downcase == "self-hosted") else true end' | tr -d '\r') || es=""
+            case "$es" in
+              true) propios="${propios:+$propios }$p" ;;
+              false) ;;
+              *) echo "verificacion: runner-$p ilegible; F1-8, decision 6" >&2; exit 2 ;;
+            esac
+          done
+          if [ -z "$propios" ]; then
+            echo "Runner propio: ningun perfil de este evento lo usa"
+            exit 0
+          fi
+          motivo=""
+          if [ "$PRIVADO" != true ]; then
+            motivo="repo publico (F1-7, decision 5)"
+          elif [ "$EVENTO" != pull_request ] || [ "$BASE" != main ] || [ "$ORIGEN" != develop ]; then
+            motivo="solo en la release, un pull_request de develop a main (evento $EVENTO, base $BASE)"
+          elif [ "$REPO_HEAD" != "$REPO" ]; then
+            motivo="el head del PR viene de otro repo ($REPO_HEAD)"
+          fi
+          if [ -n "$motivo" ]; then
+            echo "verificacion: runner propio en $propios: $motivo; F1-8, decision 6" >&2
+            exit 2
+          fi
+          echo "Runner propio en $propios: release de develop a main en $REPO (privado)"
       # V, sus padres y sus abuelos con sus arboles: profundidad 3 basta.
       - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
         if: steps.perfiles.outputs.perfiles != '[]'
@@ -301,6 +344,29 @@ jobs:
       RAMA_BASE: ${{ github.base_ref }}
       TC2_PERFIL: ${{ matrix.perfil }}
     steps:
+      # F1-8, decision 6 (D3b a) y adenda F5-5: en un runner propio, antes de
+      # sacar nada, Windows y una cuenta que no es SYSTEM ni del grupo
+      # Administradores (SID S-1-5-32-544, tambien solo para denegar con UAC).
+      # En un runner de GitHub el paso se salta. `-ExecutionPolicy Bypass` solo
+      # para este proceso: con la directiva AllSigned de la maquina, el script
+      # sin firmar que genera el runner no llega a ejecutarse (E2E de TASK-045).
+      - name: aislamiento del runner propio
+        if: runner.environment == 'self-hosted'
+        shell: powershell -NoProfile -ExecutionPolicy Bypass -Command ". '{0}'"
+        run: |
+          $ErrorActionPreference = 'Stop'
+          if ($env:RUNNER_OS -ne 'Windows') { Write-Output "verificacion: runner propio fuera de Windows ($env:RUNNER_OS); F1-8, decision 6"; exit 2 }
+          $id = [Security.Principal.WindowsIdentity]::GetCurrent()
+          if ($id.User.Value -eq 'S-1-5-18') { Write-Output "verificacion: runner propio como SYSTEM; F1-8, decision 6"; exit 2 }
+          # Por ruta: el whoami de Git (usr\bin) en el PATH no entiende /groups.
+          $grupos = (& "$env:SystemRoot\System32\whoami.exe" /groups /fo csv) -join "`n"
+          if ($LASTEXITCODE -ne 0) { Write-Output "verificacion: whoami /groups fallo; F1-8, decision 6"; exit 2 }
+          if ($grupos -match 'S-1-5-32-544') { Write-Output "verificacion: $($id.Name) es del grupo Administradores; F1-8, decision 6"; exit 2 }
+          $bash = Get-Command bash -ErrorAction SilentlyContinue
+          $ruta = if ($bash) { $bash.Source } else { '(ninguno)' }
+          Write-Output "Runner propio: usuario $($id.Name) sin administracion; bash: $ruta"
+          # La cache por arbol usa jq, que los runners de GitHub traen y Windows no.
+          if (-not (Get-Command jq -ErrorAction SilentlyContinue)) { Write-Output "verificacion: el runner propio no tiene jq en el PATH; alta minima en docs/ci/verificacion-reutilizable.md"; exit 2 }
       - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
         with:
           persist-credentials: false
@@ -318,7 +384,9 @@ jobs:
         run: |
           set -eu
           touch "$HUELLA"
-          huella=$(sha256sum "$HUELLA" | cut -d' ' -f1)
+          # Por la entrada: con la ruta, en Windows (barras invertidas) GNU
+          # sha256sum antepone `\` a la linea y la huella no casaria nunca.
+          huella=$(sha256sum < "$HUELLA" | cut -d' ' -f1)
           acierto=$(printf '%s' "$CANDIDATOS" | jq -r --arg p "$TC2_PERFIL" --arg l "$huella" '[.[$p][]? | select(.local == $l)][0] // empty | "\(.de) \(.origen)"')
           de=${acierto%% *}
           origen=${acierto#* }
diff --git a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
index 437f3d5..2fd359f 100644
--- a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
+++ b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
@@ -777,3 +777,43 @@ Los ADR aceptados no se modifican; lo que este ADR cambia queda aquí y en
   un reutilizable sin enmascararlas ni truncarlas, y que la clave del modelo
   y la App funcionen en dos jobs distintos del mismo entorno `revisor` con la
   herencia actual (adenda F6-3): se comprueba en el E2E de TASK-050.
+
+## Adenda (F5-5, TASK-045) — runner propio solo en la release
+
+- **Decisión 6 (D3b), concretada** con las opciones que Carlos aceptó el
+  2026-11-20 (D1, D3 y D4 del plan de TASK-045): el runner propio **solo se
+  usa en la release**, un `pull_request` de `develop` a `main` de un repo
+  privado con el head en el mismo repo. Lo comprueba el paso «Runner propio»
+  de `buscar` (Linux, sin código del PR, datos del evento por `env:`), que
+  falla cerrado con `exit 2` y su motivo antes de encolar nada; cuenta como
+  runner propio un `runner-<perfil>` con la etiqueta `self-hosted` o que no
+  es cadena ni lista.
+- **D3b (a)**, usuario de servicio: el primer paso de `ejecutar`, antes del
+  checkout y solo con `runner.environment == 'self-hosted'`, falla si el
+  runner no es Windows, si la cuenta es `SYSTEM` o si es del grupo
+  Administradores. Que no lea el perfil de la persona y que los datos sean de
+  solo lectura depende del alta manual
+  ([guía](../ci/verificacion-reutilizable.md#runner-propio-f5-5)).
+- **D3b (b)**, directorio limpio: `actions/checkout` con `clean: true` por
+  defecto, y el lint da error si un checkout lleva `clean:` con otro valor.
+- **D3b (c)**, la release aparte de los jobs de tarea: **se cumple por
+  construcción**, porque en el runner propio solo corre la release; no hace
+  falta otra instancia con otro usuario. Una instancia por máquina y repo.
+- **Esto limita dos textos anteriores**, que se leen con esta adenda:
+  [F1-7, decisión 5](F1-7-verificacion-ci-perfiles-cache-runners.md)
+  admitía cualquier perfil en `[self-hosted, windows, <proyecto>]` en un repo
+  privado; ahora solo la pesada, que es el único perfil de la release que no
+  corre también en los PR de tarea. Y el ejemplo de ToolsaWeb de
+  [F1-11](F1-11-extensiones-declaradas.md) ponía `completa: {runner:
+  [self-hosted, windows, toolsaweb]}`: con esta regla, la completa de un PR de
+  tarea fallaría en `buscar`; la completa va en un runner de GitHub y solo la
+  pesada en el propio.
+- **Lo que queda**: `doctor` no comprueba las etiquetas ni la visibilidad del
+  repo (leer los runners pide `administration`; F3-8), y la guarda de
+  `buscar` reconoce el runner propio por la etiqueta `self-hosted`: un runner
+  registrado con `--no-default-labels` solo lo frena el paso de aislamiento,
+  que no mira el evento. Código hostil ya integrado en `develop` corre en la
+  release con el usuario de servicio (lo contienen D1a y el aislamiento). En
+  GitLab no hay guarda equivalente: degradación declarada.
+- **No verificado**: el E2E con un runner real (criterio 1 de TASK-045), y
+  que `github.event.repository.private` llegue con valor al reutilizable.
diff --git a/docs/ci/verificacion-reutilizable.md b/docs/ci/verificacion-reutilizable.md
index eedb249..dcd4f96 100644
--- a/docs/ci/verificacion-reutilizable.md
+++ b/docs/ci/verificacion-reutilizable.md
@@ -9,9 +9,9 @@ decisión 1; RR-15): no conocen sus pasos, sus grupos ni sus herramientas.
 | GitHub Actions | [`.github/workflows/verificacion.yml`](../../.github/workflows/verificacion.yml) (`on: workflow_call`) | un job del proyecto con `uses: charliebk/TaskCodeTwo/.github/workflows/verificacion.yml@<sha>` |
 | GitLab CI | [`plantillas/gitlab/verificacion.gitlab-ci.yml`](../../plantillas/gitlab/verificacion.gitlab-ci.yml) (`spec:inputs`) | `include: remote` a la URL raw de la plantilla fijada por SHA |
 
-Fuera de F5-1: el runner propio (F5-5), `carril` y `revision-ia` (F6), y la
-definición de CI en otro proyecto de GitLab (F2-9). El informe JSON para el
-revisor (F5-4) y la caché por árbol (F5-2) se describen más abajo.
+Fuera de F5-1: `carril` y `revision-ia` (F6), y la definición de CI en otro
+proyecto de GitLab (F2-9). El informe JSON para el revisor (F5-4), la caché
+por árbol (F5-2) y el runner propio (F5-5) se describen más abajo.
 
 ## Entradas
 
@@ -168,7 +168,8 @@ workflow: eventos admitidos (`workflow_call`, `pull_request`, `push`,
 `merge_group`, `workflow_dispatch`), `permissions` arriba y en cada job (sin
 `write-all`; de lectura solo `contents`, `checks` y `actions`, F5-2), las
 únicas acciones admitidas son `actions/checkout` fijada por
-SHA, con `persist-credentials: false` y nunca en un job que escribe (F5-2:
+SHA, con `persist-credentials: false`, sin `clean:` distinto de `true` (F5-5)
+y nunca en un job que escribe (F5-2:
 antes solo se prohibían `ref:` y `repository:`) y `actions/upload-artifact` fijada por SHA y solo en un job sin
 escritura (F5-4), ningún `secrets: inherit`, ningún secreto en un job sin escritura y
 ningún `github.event.*`, `github.head_ref` ni `toJSON(github…)` en `run:`
@@ -281,6 +282,142 @@ la clave) en `.tc2/cache-verificacion-<perfil>.txt`, dentro del artefacto
 artefactos). `after_script` no cambia el estado del job: en un runner sin `sh`
 no hay línea.
 
+## Runner propio (F5-5)
+
+TASK-045. ADR [F1-8](../adr/F1-8-seguridad-modelo-de-amenazas.md), decisión 6
+(D3b) y su [adenda F5-5](../adr/F1-8-seguridad-modelo-de-amenazas.md#adenda-f5-5-task-045--runner-propio-solo-en-la-release);
+[F1-7](../adr/F1-7-verificacion-ci-perfiles-cache-runners.md), decisión 5
+(solo repos privados; runner caído, falla cerrado). No hay entradas nuevas:
+el runner propio se elige con `runner-<perfil>`.
+
+**Regla de uso** (D1): un perfil en un runner propio solo corre en la
+**release**, un `pull_request` de `develop` a `main` de un **repo privado** con
+el head **en el mismo repo**. Lo comprueba el paso «Runner propio» de `buscar`
+(Linux, sin código del PR) antes de encolar nada en el runner. Cuenta como
+runner propio un `runner-<perfil>` con la etiqueta `self-hosted` (sin distinguir
+mayúsculas) o que no es cadena ni lista (un grupo de runners, por ejemplo). Si
+falta una condición, `buscar` sale con `exit 2` y el motivo: `repo publico`,
+`solo en la release, un pull_request de develop a main` o `el head del PR viene
+de otro repo`; un `runner-<perfil>` que no es JSON también falla. Con todos
+los perfiles en runners de GitHub el paso solo escribe «ningún perfil de este
+evento lo usa» y todo sigue igual.
+
+Por eso solo la **pesada** va en el runner propio: la completa corre también
+en los PR de tarea y en `workflow_dispatch`, que fallarían. Para ToolsaWeb:
+
+```yaml
+    with:
+      comando: scripts\verificar.bat
+      shell: cmd
+      runner-completa: '"windows-latest"'
+      runner-pesada: '["self-hosted","windows","toolsaweb"]'
+```
+
+**Etiquetas** (D4): `["self-hosted","windows","<proyecto>"]`. No registres el
+runner con `--no-default-labels`: sin `self-hosted`, «Runner propio» no lo
+reconoce (el aislamiento de abajo sí, porque mira `runner.environment`).
+
+**Aislamiento** (D3b a): el primer paso de `ejecutar`, antes del checkout,
+lleva `if: runner.environment == 'self-hosted'` (en un runner de GitHub vale
+`github-hosted` y el paso se salta) y corre en Windows PowerShell con
+`-NoProfile -ExecutionPolicy Bypass` solo para ese proceso: con la directiva
+`AllSigned` de la máquina, el script sin firmar que genera el runner no llega a
+ejecutarse (lo encontró el E2E de TASK-045). Falla con
+`exit 2` si el runner no es Windows, si la cuenta es `SYSTEM` (SID
+`S-1-5-18`) o si pertenece al grupo Administradores (SID `S-1-5-32-544`,
+también cuando UAC lo deja solo para denegar). Escribe el usuario y el `bash`
+que encuentra, y falla si no hay `jq` en el `PATH` (la caché por árbol lo usa). Que la cuenta no lea tu perfil y que los datos sean de solo
+lectura no lo comprueba: depende del alta.
+
+**Directorio limpio** (D3b b): `actions/checkout` con su `clean: true` por
+defecto; el lint da error si un checkout lleva `clean:` con otro valor.
+
+**Directorio de `main` aparte** (D3b c): se cumple por construcción, porque en
+el runner propio solo corre la release (adenda F5-5 de F1-8).
+
+### Alta mínima (D2a, unos 15 minutos)
+
+En una consola de PowerShell **de administrador** en tu máquina, para un repo
+**privado**:
+
+1. **Usuario de servicio sin administración**:
+   `New-LocalUser -Name tc2-runner -Password (Read-Host -AsSecureString "Contraseña")`.
+   No lo añadas a ningún grupo más.
+2. **Carpeta del runner** fuera de tu perfil, una por repo, con permiso de
+   modificación para ese usuario:
+   `New-Item -ItemType Directory C:\ProgramData\tc2-runner\<repo>` y
+   `icacls C:\ProgramData\tc2-runner\<repo> /grant "tc2-runner:(OI)(CI)M"`.
+3. **Descarga**: en GitHub, *Settings → Actions → Runners → New self-hosted
+   runner*, Windows x64. Sigue en esa carpeta los pasos de descarga y
+   comprobación del hash que da la página; de ahí sale también el token de
+   registro (dura poco). Si la página da 404 (repo privado y el navegador con
+   otra cuenta), descarga el zip de `actions/runner` de su release, compara el
+   SHA-256 con el que publica GitHub y pide el token con
+   `gh api -X POST repos/<dueño>/<repo>/actions/runners/registration-token --jq .token`.
+   **bash y jq**: el job usa el bash de Git y `jq`, que Windows no trae. Deja
+   `jq.exe` (release de `jqlang/jq`, con su SHA-256) en
+   `C:\ProgramData\tc2-runner\bin` y crea en la carpeta del runner un `.env`
+   con `PATH=C:\ProgramData\tc2-runner\bin;C:\Program Files\Git\bin;C:\Program Files\Git\usr\bin;C:\WINDOWS\system32;C:\WINDOWS;C:\WINDOWS\System32\WindowsPowerShell\v1.0`.
+   El runner lo lee al arrancar: si lo cambias, reinicia el servicio. El paso
+   de aislamiento falla con `exit 2` si no encuentra `jq`.
+4. **Registro como servicio con ese usuario**:
+
+   ```powershell
+   .\config.cmd --url https://github.com/<dueño>/<repo> --token <token> `
+     --name <máquina>-<repo> --labels <proyecto> `
+     --runasservice --windowslogonaccount .\tc2-runner
+   ```
+
+   No escribas la contraseña en la línea de órdenes
+   (`--windowslogonpassword`): queda en el historial.
+5. **Datos de solo lectura**: `icacls <ruta de los datos> /grant
+   "tc2-runner:(OI)(CI)RX"` (en ToolsaWeb, `exampleData`). Comprueba con
+   `icacls C:\Users\<tú>` que ni `tc2-runner` ni `Usuarios` tienen acceso a tu
+   perfil.
+6. **Una instancia por máquina y repo**: un runner ejecuta un job a la vez,
+   así que dos jobs nunca comparten `target/` ni `_work` (corrige la
+   divergencia 6). No registres una segunda instancia para el mismo repo en
+   la misma máquina.
+7. **Comprobación**: en *Settings → Actions → Runners* el runner sale
+   `Idle` con `self-hosted`, `Windows` y `<proyecto>`. En el primer PR de
+   release, el paso «aislamiento del runner propio» escribe `Runner propio:
+   usuario <máquina>\tc2-runner sin administracion; bash: ...`.
+
+### Retirada
+
+1. Cambia antes `runner-pesada` del llamador a un runner de GitHub (o quítalo):
+   si no, la pesada de la release espera en cola hasta que GitHub la cancela a
+   las 24 h, y el PR no se mergea (falla cerrado, F1-7 decisión 5).
+2. *Settings → Actions → Runners →* el runner *→ Remove*, y en la máquina la
+   orden que muestra la página (`.\config.cmd remove --token <token>`): quita
+   el registro, la configuración local y el servicio. Sin acceso a la
+   máquina, *Force remove this runner*.
+3. Borra la carpeta del runner y el usuario: `Remove-LocalUser tc2-runner`.
+
+### Qué está comprobado y qué no
+
+- **En la documentación de GitHub** (2026-11-20): `runner.environment` vale
+  `github-hosted` o `self-hosted` (contexto `runner`); en un workflow
+  reutilizable el contexto `github` es el del llamador, y el reutilizable
+  puede usar los runners propios del repo llamador; un job sin runner espera
+  en cola y falla a las 24 h; las etiquetas no distinguen mayúsculas.
+- **En el código del runner** (`actions/runner`, `Constants.cs` y
+  `CommandSettings.cs`; la página de GitHub no las lista): `config` acepta
+  `--runasservice`, `--windowslogonaccount`, `--windowslogonpassword`,
+  `--labels`, `--name`, `--url`, `--token` y `--no-default-labels`, y
+  `remove` acepta `--token`.
+- **Probado en local**: el `run:` de «Runner propio» con `bash` y datos
+  simulados (pruebas de `xtask/src/workflows.rs`), y el de aislamiento con
+  Windows PowerShell: con una cuenta del grupo Administradores sale `exit 2`.
+- **No verificado**: que `config.cmd` pida la contraseña si no se le pasa, y
+  que conceda solo el derecho de iniciar sesión como servicio; que la
+  etiqueta `self-hosted` se ponga sola (la página de etiquetas solo cita el
+  sistema y la arquitectura: compruébalo en el paso 7);
+  que `github.event.repository.private` y
+  `github.event.pull_request.head.repo.full_name` lleguen con valor al
+  reutilizable (se deduce de que el contexto es el del llamador); el E2E con
+  un runner real.
+
 ## Divergencias con los ADR
 
 1. **Argumentos por perfil** (D1): F1-7, decisión 1, fija `--perfil
@@ -310,9 +447,10 @@ no hay línea.
    encolan las ejecuciones del mismo PR; con `cancel-in-progress: false` GitHub
    cancela la pendiente más antigua del grupo cuando llega una tercera, que es
    un commit ya superado del mismo PR. PR distintos corren en paralelo: un
-   runner propio con un único ejecutor los serializa por sí mismo, y varios
-   ejecutores en la misma máquina (que compartirían `target/`) quedan para
-   F5-5. Comprobado solo por la prueba `la_concurrencia_es_por_pr_o_ref`; no se
+   runner propio ejecuta un job a la vez y los serializa por sí mismo. F5-5
+   fija **una instancia por máquina y repo** (sección «Runner propio»), así
+   que dos jobs del mismo repo nunca comparten `target/` en la misma máquina.
+   Comprobado solo por la prueba `la_concurrencia_es_por_pr_o_ref`; no se
    reprodujo en el E2E.
 7. **Segunda acción en la lista blanca** (F5-4, D3): F1-8 y esta guía decían
    que la única acción admitida es `actions/checkout`. Se admite también
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-045/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-045/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-045/tarea.md                                    | 3 ++-
 3 files changed, 2 insertions(+), 1 deletion(-)
````

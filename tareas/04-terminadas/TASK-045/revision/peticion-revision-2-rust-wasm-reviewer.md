# Peticion de revision — TASK-045 (ronda 2)

- Tarea: TASK-045 — F5-5 Runner self-hosted Windows por perfil
- Rama revisada: feature/task-045-f5-5-runner-self-hosted-windows-por-perf
- Rama base: develop
- Commit revisado (HEAD): e141ed154087ea00e2e7fa9536bed2fb42c093f7
- Fecha: 2026-11-21
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-045 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde addfcb2da94056304689989a9516567c77a7c466 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (8 hallazgo(s)))

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
informe de esta ronda (informe-revision-2-rust-wasm-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-045/tarea.md
- tareas/03-en-revision/TASK-045/planificacion/
- .github/workflows/verificacion.yml
- docs/adr/F1-8-seguridad-modelo-de-amenazas.md
- docs/ci/verificacion-reutilizable.md
- xtask/src/workflows.rs

## Hallazgos de la ronda 1 pendientes de verificar

Tus 2 hallazgo(s) MENOR de la ronda 1 no se reverifican (los comprueba
quien coordina): recibes esta ronda porque la correccion
toca ficheros de tu dominio. Revisa en el diff de esta ronda lo que la correccion
cambia en tu terreno y si abre un fallo.

## Commits a revisar (git log addfcb2da94056304689989a9516567c77a7c466..HEAD)

````
e141ed1 fix(TASK-045): regla del evento tambien en el aislamiento (revision ronda 1)
a46ec0b chore(TASK-045): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff addfcb2da94056304689989a9516567c77a7c466..HEAD)

````diff
diff --git a/.github/workflows/verificacion.yml b/.github/workflows/verificacion.yml
index 9e15ad7..071e5bb 100644
--- a/.github/workflows/verificacion.yml
+++ b/.github/workflows/verificacion.yml
@@ -12,8 +12,9 @@
 #     merge_group la completa. Un perfil con runner propio solo pasa en la
 #     release de un repo privado (TASK-045, F5-5; F1-8, decision 6).
 #   - ejecutar (ejecuta, matriz por perfil, runner del perfil, sin secretos):
-#     en un runner propio comprueba antes su aislamiento; despues calcula la huella local, y si un candidato tiene la misma se salta el
-#     comando (paso `skipped`); si no, corre `comando argumentos-<perfil>` en
+#     en un runner propio comprueba antes su aislamiento y la regla del
+#     evento; despues calcula la huella local, y si un candidato tiene la
+#     misma se salta el comando (paso `skipped`); si no, corre `comando argumentos-<perfil>` en
 #     el shell que declara el proyecto y sube el informe JSON de `TC2_INFORME`
 #     (F5-4) como artefacto `informe-verificacion-<perfil>`, si se escribio.
 #   - verificacion (privilegio, sin checkout): lee el arbol de V por la API,
@@ -168,7 +169,9 @@ jobs:
         run: |
           set -euo pipefail
           propios=""
-          for p in $(printf '%s' "$PERFILES" | jq -r '.[]' | tr -d '\r'); do
+          # Lista antes del for: un jq que falla dentro de $(...) no lo para set -e.
+          lista=$(printf '%s' "$PERFILES" | jq -r '.[]' | tr -d '\r') || { echo "verificacion: perfiles ilegibles ($PERFILES); F1-8, decision 6" >&2; exit 2; }
+          for p in $lista; do
             es=$(printf '%s' "$ENTRADAS" | jq -r --arg p "$p" '.["runner-" + $p] | fromjson | if type == "string" then ascii_downcase == "self-hosted" elif type == "array" then any(.[]; tostring | ascii_downcase == "self-hosted") else true end' | tr -d '\r') || es=""
             case "$es" in
               true) propios="${propios:+$propios }$p" ;;
@@ -181,15 +184,17 @@ jobs:
             exit 0
           fi
           motivo=""
-          if [ "$PRIVADO" != true ]; then
+          if [ "$PRIVADO" = false ]; then
             motivo="repo publico (F1-7, decision 5)"
+          elif [ "$PRIVADO" != true ]; then
+            motivo="no consta si el repo es privado (private=${PRIVADO:--})"
           elif [ "$EVENTO" != pull_request ] || [ "$BASE" != main ] || [ "$ORIGEN" != develop ]; then
-            motivo="solo en la release, un pull_request de develop a main (evento $EVENTO, base $BASE)"
+            motivo="solo en la release, un pull_request de develop a main (evento $EVENTO, base ${BASE:--}, origen ${ORIGEN:--})"
           elif [ "$REPO_HEAD" != "$REPO" ]; then
             motivo="el head del PR viene de otro repo ($REPO_HEAD)"
           fi
           if [ -n "$motivo" ]; then
-            echo "verificacion: runner propio en $propios: $motivo; F1-8, decision 6" >&2
+            echo "verificacion: runner propio en $propios: $motivo; F1-8, decision 6. Que hacer: en este evento pon runner-<perfil> en un runner de GitHub; el propio solo corre la release" >&2
             exit 2
           fi
           echo "Runner propio en $propios: release de develop a main en $REPO (privado)"
@@ -350,12 +355,23 @@ jobs:
       # En un runner de GitHub el paso se salta. `-ExecutionPolicy Bypass` solo
       # para este proceso: con la directiva AllSigned de la maquina, el script
       # sin firmar que genera el runner no llega a ejecutarse (E2E de TASK-045).
+      # La regla del evento se repite aqui: `buscar` solo reconoce el runner
+      # propio por la etiqueta `self-hosted`, y un runs-on con la etiqueta del
+      # proyecto sin ella llegaria a la maquina (revision de TASK-045, IMP-1).
       - name: aislamiento del runner propio
         if: runner.environment == 'self-hosted'
         shell: powershell -NoProfile -ExecutionPolicy Bypass -Command ". '{0}'"
+        env:
+          EVENTO: ${{ github.event_name }}
+          PRIVADO: ${{ github.event.repository.private }}
+          BASE: ${{ github.base_ref }}
+          ORIGEN: ${{ github.head_ref }}
+          REPO: ${{ github.repository }}
+          REPO_HEAD: ${{ github.event.pull_request.head.repo.full_name }}
         run: |
           $ErrorActionPreference = 'Stop'
           if ($env:RUNNER_OS -ne 'Windows') { Write-Output "verificacion: runner propio fuera de Windows ($env:RUNNER_OS); F1-8, decision 6"; exit 2 }
+          if ($env:PRIVADO -cne 'true' -or $env:EVENTO -cne 'pull_request' -or $env:BASE -cne 'main' -or $env:ORIGEN -cne 'develop' -or $env:REPO_HEAD -cne $env:REPO) { Write-Output "verificacion: runner propio fuera de la release de un repo privado (evento $env:EVENTO, base $env:BASE, origen $env:ORIGEN, privado $env:PRIVADO, head $env:REPO_HEAD); F1-8, decision 6. Que hacer: en este evento pon runner-<perfil> en un runner de GitHub"; exit 2 }
           $id = [Security.Principal.WindowsIdentity]::GetCurrent()
           if ($id.User.Value -eq 'S-1-5-18') { Write-Output "verificacion: runner propio como SYSTEM; F1-8, decision 6"; exit 2 }
           # Por ruta: el whoami de Git (usr\bin) en el PATH no entiende /groups.
@@ -363,8 +379,8 @@ jobs:
           if ($LASTEXITCODE -ne 0) { Write-Output "verificacion: whoami /groups fallo; F1-8, decision 6"; exit 2 }
           if ($grupos -match 'S-1-5-32-544') { Write-Output "verificacion: $($id.Name) es del grupo Administradores; F1-8, decision 6"; exit 2 }
           $bash = Get-Command bash -ErrorAction SilentlyContinue
-          $ruta = if ($bash) { $bash.Source } else { '(ninguno)' }
-          Write-Output "Runner propio: usuario $($id.Name) sin administracion; bash: $ruta"
+          if (-not $bash) { Write-Output "verificacion: el runner propio no tiene bash en el PATH; alta minima en docs/ci/verificacion-reutilizable.md"; exit 2 }
+          Write-Output "Runner propio: usuario $($id.Name) sin administracion; bash: $($bash.Source)"
           # La cache por arbol usa jq, que los runners de GitHub traen y Windows no.
           if (-not (Get-Command jq -ErrorAction SilentlyContinue)) { Write-Output "verificacion: el runner propio no tiene jq en el PATH; alta minima en docs/ci/verificacion-reutilizable.md"; exit 2 }
       - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
diff --git a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
index 2fd359f..0c243d3 100644
--- a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
+++ b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
@@ -790,8 +790,10 @@ Los ADR aceptados no se modifican; lo que este ADR cambia queda aquí y en
   es cadena ni lista.
 - **D3b (a)**, usuario de servicio: el primer paso de `ejecutar`, antes del
   checkout y solo con `runner.environment == 'self-hosted'`, falla si el
-  runner no es Windows, si la cuenta es `SYSTEM` o si es del grupo
-  Administradores. Que no lea el perfil de la persona y que los datos sean de
+  runner no es Windows, si el evento no es la release de un repo privado con
+  el head en el mismo repo (la regla de `buscar`, repetida porque esta no
+  depende de la etiqueta; revisión de TASK-045), si la cuenta es `SYSTEM` o si
+  es del grupo Administradores. Que no lea el perfil de la persona y que los datos sean de
   solo lectura depende del alta manual
   ([guía](../ci/verificacion-reutilizable.md#runner-propio-f5-5)).
 - **D3b (b)**, directorio limpio: `actions/checkout` con `clean: true` por
@@ -810,10 +812,15 @@ Los ADR aceptados no se modifican; lo que este ADR cambia queda aquí y en
   pesada en el propio.
 - **Lo que queda**: `doctor` no comprueba las etiquetas ni la visibilidad del
   repo (leer los runners pide `administration`; F3-8), y la guarda de
-  `buscar` reconoce el runner propio por la etiqueta `self-hosted`: un runner
-  registrado con `--no-default-labels` solo lo frena el paso de aislamiento,
-  que no mira el evento. Código hostil ya integrado en `develop` corre en la
-  release con el usuario de servicio (lo contienen D1a y el aislamiento). En
-  GitLab no hay guarda equivalente: degradación declarada.
-- **No verificado**: el E2E con un runner real (criterio 1 de TASK-045), y
-  que `github.event.repository.private` llegue con valor al reutilizable.
+  `buscar` reconoce el runner propio por la etiqueta `self-hosted`: un
+  `runner-<perfil>` sin ella (o un runner con `--no-default-labels`) llega a
+  la máquina y lo para el aislamiento antes del checkout, sin ejecutar código
+  del PR. Un llamador hostil puede escribir su propio `runs-on` fuera del
+  reutilizable: eso no lo cubre ninguna guarda de este workflow (D1a). Código
+  hostil ya integrado en `develop` corre en la release con el usuario de
+  servicio (lo contienen D1a y el aislamiento). En GitLab no hay guarda
+  equivalente: degradación declarada.
+- **Verificado con un runner real** (E2E de TASK-045, 2026-11-21): la pesada
+  de la release en el runner propio con el usuario de servicio y los datos de
+  solo lectura; `github.event.repository.private` llega con valor al
+  reutilizable.
diff --git a/docs/ci/verificacion-reutilizable.md b/docs/ci/verificacion-reutilizable.md
index dcd4f96..39d0100 100644
--- a/docs/ci/verificacion-reutilizable.md
+++ b/docs/ci/verificacion-reutilizable.md
@@ -296,11 +296,13 @@ el head **en el mismo repo**. Lo comprueba el paso «Runner propio» de `buscar`
 (Linux, sin código del PR) antes de encolar nada en el runner. Cuenta como
 runner propio un `runner-<perfil>` con la etiqueta `self-hosted` (sin distinguir
 mayúsculas) o que no es cadena ni lista (un grupo de runners, por ejemplo). Si
-falta una condición, `buscar` sale con `exit 2` y el motivo: `repo publico`,
-`solo en la release, un pull_request de develop a main` o `el head del PR viene
-de otro repo`; un `runner-<perfil>` que no es JSON también falla. Con todos
-los perfiles en runners de GitHub el paso solo escribe «ningún perfil de este
-evento lo usa» y todo sigue igual.
+falta una condición, `buscar` sale con `exit 2`, el motivo (`repo publico`,
+`no consta si el repo es privado`, `solo en la release, un pull_request de
+develop a main` con el evento, la base y el origen, o `el head del PR viene de
+otro repo`) y qué hacer: poner ese `runner-<perfil>` en un runner de GitHub. Un
+`runner-<perfil>` o unos perfiles que no son JSON también fallan. Con todos los
+perfiles en runners de GitHub el paso solo escribe `Runner propio: ningun
+perfil de este evento lo usa` y todo sigue igual.
 
 Por eso solo la **pesada** va en el runner propio: la completa corre también
 en los PR de tarea y en `workflow_dispatch`, que fallarían. Para ToolsaWeb:
@@ -313,21 +315,28 @@ en los PR de tarea y en `workflow_dispatch`, que fallarían. Para ToolsaWeb:
       runner-pesada: '["self-hosted","windows","toolsaweb"]'
 ```
 
-**Etiquetas** (D4): `["self-hosted","windows","<proyecto>"]`. No registres el
-runner con `--no-default-labels`: sin `self-hosted`, «Runner propio» no lo
-reconoce (el aislamiento de abajo sí, porque mira `runner.environment`).
+**Etiquetas** (D4): `["self-hosted","windows","<proyecto>"]`, siempre con
+`self-hosted` en el `runner-<perfil>`. «Runner propio» de `buscar` solo
+reconoce el runner propio por esa etiqueta: con `'["windows","<proyecto>"]'`
+(o un runner registrado con `--no-default-labels`) el job llega a tu máquina y
+lo para el aislamiento de abajo, que repite la regla del evento y no depende
+de la etiqueta, antes de sacar el código.
 
 **Aislamiento** (D3b a): el primer paso de `ejecutar`, antes del checkout,
 lleva `if: runner.environment == 'self-hosted'` (en un runner de GitHub vale
 `github-hosted` y el paso se salta) y corre en Windows PowerShell con
 `-NoProfile -ExecutionPolicy Bypass` solo para ese proceso: con la directiva
 `AllSigned` de la máquina, el script sin firmar que genera el runner no llega a
-ejecutarse (lo encontró el E2E de TASK-045). Falla con
-`exit 2` si el runner no es Windows, si la cuenta es `SYSTEM` (SID
-`S-1-5-18`) o si pertenece al grupo Administradores (SID `S-1-5-32-544`,
-también cuando UAC lo deja solo para denegar). Escribe el usuario y el `bash`
-que encuentra, y falla si no hay `jq` en el `PATH` (la caché por árbol lo usa). Que la cuenta no lea tu perfil y que los datos sean de solo
-lectura no lo comprueba: depende del alta.
+ejecutarse (lo encontró el E2E de TASK-045). Falla, con su motivo, si el
+runner no es Windows; si el evento no es la release de un repo privado con el
+head en el mismo repo (la misma regla de `buscar`, con los datos del evento por
+`env:`); si la cuenta es `SYSTEM` (SID `S-1-5-18`) o si pertenece al grupo
+Administradores (SID `S-1-5-32-544`, también cuando UAC lo deja solo para
+denegar); y si no encuentra `bash` o `jq` en el `PATH` (la caché por árbol usa
+`jq`). El script hace `exit 2`, pero con `-Command` Windows PowerShell lo
+devuelve como 1: el paso falla igual. Si todo va bien escribe el usuario y el
+`bash`. Que la cuenta no lea tu perfil y que los datos sean de solo lectura no
+lo comprueba: depende del alta.
 
 **Directorio limpio** (D3b b): `actions/checkout` con su `clean: true` por
 defecto; el lint da error si un checkout lleva `clean:` con otro valor.
@@ -341,7 +350,8 @@ En una consola de PowerShell **de administrador** en tu máquina, para un repo
 **privado**:
 
 1. **Usuario de servicio sin administración**:
-   `New-LocalUser -Name tc2-runner -Password (Read-Host -AsSecureString "Contraseña")`.
+   `New-LocalUser -Name tc2-runner -Password (Read-Host -AsSecureString "Contraseña") -PasswordNeverExpires`
+   (si la contraseña caduca, el servicio deja de arrancar).
    No lo añadas a ningún grupo más.
 2. **Carpeta del runner** fuera de tu perfil, una por repo, con permiso de
    modificación para ese usuario:
@@ -359,7 +369,7 @@ En una consola de PowerShell **de administrador** en tu máquina, para un repo
    `C:\ProgramData\tc2-runner\bin` y crea en la carpeta del runner un `.env`
    con `PATH=C:\ProgramData\tc2-runner\bin;C:\Program Files\Git\bin;C:\Program Files\Git\usr\bin;C:\WINDOWS\system32;C:\WINDOWS;C:\WINDOWS\System32\WindowsPowerShell\v1.0`.
    El runner lo lee al arrancar: si lo cambias, reinicia el servicio. El paso
-   de aislamiento falla con `exit 2` si no encuentra `jq`.
+   de aislamiento falla si no encuentra `bash` o `jq`.
 4. **Registro como servicio con ese usuario**:
 
    ```powershell
@@ -408,15 +418,20 @@ En una consola de PowerShell **de administrador** en tu máquina, para un repo
   `remove` acepta `--token`.
 - **Probado en local**: el `run:` de «Runner propio» con `bash` y datos
   simulados (pruebas de `xtask/src/workflows.rs`), y el de aislamiento con
-  Windows PowerShell: con una cuenta del grupo Administradores sale `exit 2`.
+  Windows PowerShell y el shell exacto del paso: un PR de fork, un repo
+  público u `origen Develop` se rechazan por la regla del evento; con una
+  cuenta del grupo Administradores falla (código 1).
+- **E2E con un runner real** (TASK-045, 2026-11-21, repo privado de pruebas):
+  la pesada de la release corrió en el runner propio con `tc2-runner` sin
+  administración y los datos de solo lectura; `github.event.repository.private`
+  y `github.event.pull_request.head.repo.full_name` llegan con valor al
+  reutilizable; la etiqueta `self-hosted` se pone sola. Encontró la directiva
+  `AllSigned`, la falta de `jq` y la huella con `\` en Windows, ya corregidos.
+  En un repo público el runner propio se rechaza en `buscar` sin encolar nada.
 - **No verificado**: que `config.cmd` pida la contraseña si no se le pasa, y
-  que conceda solo el derecho de iniciar sesión como servicio; que la
-  etiqueta `self-hosted` se ponga sola (la página de etiquetas solo cita el
-  sistema y la arquitectura: compruébalo en el paso 7);
-  que `github.event.repository.private` y
-  `github.event.pull_request.head.repo.full_name` lleguen con valor al
-  reutilizable (se deduce de que el contexto es el del llamador); el E2E con
-  un runner real.
+  que conceda solo el derecho de iniciar sesión como servicio; un acierto de la
+  caché por árbol en el runner propio (en el E2E el diff tocaba el workflow y
+  la regla de rutas no reutiliza, por diseño).
 
 ## Divergencias con los ADR
 
diff --git a/xtask/src/workflows.rs b/xtask/src/workflows.rs
index 6769bd3..6585a69 100644
--- a/xtask/src/workflows.rs
+++ b/xtask/src/workflows.rs
@@ -1246,6 +1246,32 @@ fn el_runner_propio_solo_en_la_release_de_un_privado() {
         matches!(runner_propio(&texto, &cambiar(&release, &[("ENTRADAS", &grupo), ("PRIVADO", "false")])), Err(m) if m.contains("repo publico")),
         "un grupo de runners cuenta como runner propio"
     );
+    // Revision de TASK-045, MEN-1: `private` sin valor no se confunde con un
+    // repo publico, y unos perfiles ilegibles fallan cerrado.
+    assert!(
+        matches!(runner_propio(&texto, &cambiar(&release, &[("ENTRADAS", &propio), ("PRIVADO", "")])), Err(m) if m.starts_with("Some(2): ") && m.contains("no consta si el repo es privado")),
+        "private vacio"
+    );
+    assert!(
+        matches!(runner_propio(&texto, &cambiar(&release, &[("ENTRADAS", &propio), ("PERFILES", "basura")])), Err(m) if m.starts_with("Some(2): ") && m.contains("perfiles ilegibles")),
+        "perfiles ilegibles"
+    );
+    // MEN-2: el motivo nombra el origen y dice que hacer.
+    assert!(
+        matches!(runner_propio(&texto, &cambiar(&release, &[("ENTRADAS", &propio), ("ORIGEN", "feature/x")])), Err(m) if m.contains("origen feature/x") && m.contains("Que hacer:")),
+        "motivo con origen y accion"
+    );
+    // IMP-1: una etiqueta sin `self-hosted` no la reconoce `buscar`; la para
+    // el aislamiento de `ejecutar` (el_aislamiento_va_antes_del_checkout...).
+    let suelta = entradas_con_pesada(r#"["windows","tc2-e2e"]"#);
+    assert_eq!(
+        runner_propio(
+            &texto,
+            &cambiar(&release, &[("ENTRADAS", &suelta), ("PRIVADO", "false")])
+        ),
+        Ok("Runner propio: ningun perfil de este evento lo usa".to_owned()),
+        "etiqueta suelta: limite conocido de buscar"
+    );
     let ilegible = entradas_con_pesada("no-es-json");
     assert!(
         matches!(runner_propio(&texto, &cambiar(&release, &[("ENTRADAS", &ilegible)])), Err(m) if m.starts_with("Some(2): ") && m.contains("runner-pesada ilegible")),
@@ -1299,14 +1325,40 @@ fn aislamiento_antes_del_checkout(texto: &str) -> Result<(), String> {
     if campo("shell") != Some(SHELL_AISLAMIENTO) {
         return Err(format!("aislamiento con shell {:?}", campo("shell")));
     }
+    // La regla del evento, con los datos del llamador por `env:` (revision de
+    // TASK-045, IMP-1): no depende de la etiqueta del runs-on.
+    for (clave, valor) in [
+        ("EVENTO", "${{ github.event_name }}"),
+        ("PRIVADO", "${{ github.event.repository.private }}"),
+        ("BASE", "${{ github.base_ref }}"),
+        ("ORIGEN", "${{ github.head_ref }}"),
+        ("REPO", "${{ github.repository }}"),
+        (
+            "REPO_HEAD",
+            "${{ github.event.pull_request.head.repo.full_name }}",
+        ),
+    ] {
+        let real = primero
+            .get("env")
+            .and_then(|e| e.get(clave))
+            .and_then(Nodo::texto);
+        if real != Some(valor) {
+            return Err(format!("aislamiento con env {clave} {real:?}"));
+        }
+    }
     let run = campo("run").unwrap_or_default();
+    // Cada guarda entera, no solo su literal: un `if ($false)` que conserva
+    // el SID tambien tiene que dar error (revision de TASK-045, MEN-1).
     // whoami por ruta: el de Git (usr\bin) en el PATH no entiende /groups.
     for exigido in [
-        "S-1-5-18",
-        "S-1-5-32-544",
-        r#"System32\whoami.exe" /groups"#,
-        "RUNNER_OS",
-        "Get-Command jq",
+        "if ($env:RUNNER_OS -ne 'Windows') {",
+        "if ($env:PRIVADO -cne 'true' -or $env:EVENTO -cne 'pull_request' -or $env:BASE -cne 'main' -or $env:ORIGEN -cne 'develop' -or $env:REPO_HEAD -cne $env:REPO) {",
+        "if ($id.User.Value -eq 'S-1-5-18') {",
+        r#"$grupos = (& "$env:SystemRoot\System32\whoami.exe" /groups /fo csv) -join"#,
+        "if ($LASTEXITCODE -ne 0) {",
+        "if ($grupos -match 'S-1-5-32-544') {",
+        "if (-not $bash) {",
+        "if (-not (Get-Command jq -ErrorAction SilentlyContinue)) {",
     ] {
         if !run.contains(exigido) {
             return Err(format!("aislamiento sin {exigido}"));
@@ -1367,7 +1419,51 @@ fn el_aislamiento_va_antes_del_checkout_en_el_runner_propio() {
         ),
         (
             "sin comprobar jq",
-            texto.replacen("Get-Command jq", "Get-Command bash", 1),
+            texto.replacen(
+                "if (-not (Get-Command jq",
+                "if ($false -and -not (Get-Command jq",
+                1,
+            ),
+        ),
+        (
+            "sin comprobar bash",
+            texto.replacen("if (-not $bash) {", "if ($false) {", 1),
+        ),
+        (
+            "guarda de SYSTEM desactivada",
+            texto.replacen(
+                "if ($id.User.Value -eq 'S-1-5-18') {",
+                "if ($false -and $id.User.Value -eq 'S-1-5-18') {",
+                1,
+            ),
+        ),
+        (
+            "guarda de Windows desactivada",
+            texto.replacen(
+                "if ($env:RUNNER_OS -ne 'Windows') {",
+                "if ($false -and $env:RUNNER_OS -ne 'Windows') {",
+                1,
+            ),
+        ),
+        (
+            "sin mirar el codigo de whoami",
+            texto.replacen("if ($LASTEXITCODE -ne 0) {", "if ($false) {", 1),
+        ),
+        (
+            "sin la regla del evento",
+            texto.replacen(
+                "-or $env:REPO_HEAD -cne $env:REPO) {",
+                ") {",
+                1,
+            ),
+        ),
+        (
+            "regla con el evento de otro sitio",
+            texto.replacen(
+                &format!("        shell: {SHELL_AISLAMIENTO}\n        env:\n          EVENTO: ${{{{ github.event_name }}}}\n"),
+                &format!("        shell: {SHELL_AISLAMIENTO}\n        env:\n          EVENTO: pull_request\n"),
+                1,
+            ),
         ),
         (
             "sin el grupo Administradores",
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff addfcb2da94056304689989a9516567c77a7c466..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-045/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-045/planificacion/plan-final.md                 |   0
 tareas/03-en-revision/TASK-045/revision/informe-revision-1-code-quality-reviewer.md         |  21 +++++
 tareas/03-en-revision/TASK-045/revision/informe-revision-1-rust-wasm-reviewer.md            |  21 +++++
 tareas/03-en-revision/TASK-045/revision/peticion-revision-1-code-quality-reviewer.md        | 412 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-045/revision/peticion-revision-1-rust-wasm-reviewer.md           | 453 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-045/tarea.md                                    |   9 +-
 7 files changed, 913 insertions(+), 3 deletions(-)
````

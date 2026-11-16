# Peticion de revision — TASK-026 (ronda 2)

- Tarea: TASK-026 — F2-11 Instaladores de una linea
- Rama revisada: feature/task-026-f2-11-instaladores-de-una-linea
- Rama base: develop
- Commit revisado (HEAD): dedddf7567e00cff3b26cb454c4e8b9ac67e7822
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-026 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 6609149e64b240c697b4eba4036d5e8006865279 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (7 hallazgo(s)))

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

- tareas/03-en-revision/TASK-026/tarea.md
- tareas/03-en-revision/TASK-026/planificacion/
- .github/scripts/probar-instalador.sh
- docs/distribucion/instalar.md
- instalar/install.ps1
- instalar/install.sh
- xtask/src/instaladores.rs

## Hallazgos de la ronda 1 pendientes de verificar

Tus 2 hallazgo(s) MENOR de la ronda 1 no se reverifican (los comprueba
quien coordina): recibes esta ronda porque la correccion
toca ficheros de tu dominio. Revisa en el diff de esta ronda lo que la correccion
cambia en tu terreno y si abre un fallo.

## Commits a revisar (git log 6609149e64b240c697b4eba4036d5e8006865279..HEAD)

````
dedddf7 fix(TASK-026): hallazgos de la ronda 1
a5ab364 chore(TASK-026): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 6609149e64b240c697b4eba4036d5e8006865279..HEAD)

````diff
diff --git a/.github/scripts/probar-instalador.sh b/.github/scripts/probar-instalador.sh
index 6395bc0..dfbec90 100644
--- a/.github/scripts/probar-instalador.sh
+++ b/.github/scripts/probar-instalador.sh
@@ -129,14 +129,26 @@ medir() {
 
 hash_alterado() {
   export TC2_BASE_URL="$url/malo"
+  # Tiene que fallar por el hash, no por cualquier otra cosa (un 404, la red).
+  set +e
   if [ "$so" = windows ]; then
-    if linea_windows "$(dir_windows "$tmp/dest-malo")"; then rojo "con un hash alterado el instalador salio con 0"; fi
-    [ ! -e "$tmp/dest-malo/tc2.exe" ] || rojo "con un hash alterado quedo tc2.exe instalado"
+    salida=$(linea_windows "$(dir_windows "$tmp/dest-malo")" 2>&1)
+    codigo=$?
+    instalado="$tmp/dest-malo/tc2.exe"
   else
     mkdir -p "$tmp/home-malo"
-    if linea_unix "$tmp/home-malo"; then rojo "con un hash alterado el instalador salio con 0"; fi
-    [ ! -e "$tmp/home-malo/.local/bin/tc2" ] || rojo "con un hash alterado quedo tc2 instalado"
+    salida=$(linea_unix "$tmp/home-malo" 2>&1)
+    codigo=$?
+    instalado="$tmp/home-malo/.local/bin/tc2"
   fi
+  set -e
+  echo "$salida"
+  [ "$codigo" -ne 0 ] || rojo "con un hash alterado el instalador salio con 0"
+  case "$salida" in
+    *"no casa con SHA256SUMS"*) ;;
+    *) rojo "con un hash alterado el instalador fallo, pero no por el hash" ;;
+  esac
+  [ ! -e "$instalado" ] || rojo "con un hash alterado quedo $instalado"
   export TC2_BASE_URL="$url"
   echo "probar: VERDE: un hash que no casa no instala nada"
 }
@@ -160,6 +172,12 @@ path_idempotente() {
     linea_unix "$h"
     n=$(grep -cF '# >>> tc2 (TaskCodeTwo) >>>' "$h/.profile") || n=0
     [ "$n" = 1 ] || rojo "tras instalar dos veces, .profile tiene $n bloques de tc2"
+    # Reinstalar en otro directorio: un solo bloque, con el directorio nuevo.
+    (export TC2_INSTALL_DIR="$h/otro"; linea_unix "$h")
+    n=$(grep -cF '# >>> tc2 (TaskCodeTwo) >>>' "$h/.profile") || n=0
+    [ "$n" = 1 ] || rojo "tras reinstalar en otro directorio, .profile tiene $n bloques de tc2"
+    ruta=$(terminal_nueva_unix "$h" | head -n 1)
+    [ "$ruta" = "$h/otro/tc2" ] || rojo "tras reinstalar en otro directorio, la terminal nueva encuentra '$ruta'"
     h="$tmp/home-sin-path"
     mkdir -p "$h"
     (export TC2_NO_MODIFY_PATH=1; linea_unix "$h")
diff --git a/docs/distribucion/instalar.md b/docs/distribucion/instalar.md
index 81accf8..2a70a88 100644
--- a/docs/distribucion/instalar.md
+++ b/docs/distribucion/instalar.md
@@ -58,8 +58,16 @@ curl -fsSL https://github.com/charliebk/TaskCodeTwo/releases/download/v0.0.0-pru
      # <<< tc2 (TaskCodeTwo) <<<
      ```
 
-     Instalar otra vez no repite el bloque. `tc2 desinstalar` (F2-14) lo quita
-     por sus marcas. Con fish, el instalador dice qué ejecutar.
+     Instalar otra vez no repite el bloque, y reinstalar en otro directorio lo
+     sustituye por uno con el directorio nuevo. `tc2 desinstalar` (F2-14) lo
+     quita por sus marcas. Con fish, el instalador dice qué ejecutar. Como el
+     directorio se escribe en el perfil, se rechaza si lleva comillas, `$`,
+     `` ` ``, `\`, `:` o un salto de línea.
+
+   En Windows, reinstalar en otro directorio añade el nuevo al final del PATH
+   del usuario, pero no quita el anterior: si el `tc2.exe` viejo sigue ahí,
+   gana él. Hasta `tc2 desinstalar` (F2-14), quita a mano la entrada antigua.
+   Un directorio con `;` (el separador del PATH) se rechaza.
 
 Volver a ejecutar la línea reinstala (o cambia de versión con `TC2_VERSION`).
 Actualizar con `tc2 actualizar --cli` es de F2-12. No toca `taskctl` ni nada
diff --git a/instalar/install.ps1 b/instalar/install.ps1
index ee7f5ea..8d6a207 100644
--- a/instalar/install.ps1
+++ b/instalar/install.ps1
@@ -143,7 +143,8 @@ function Install-Tc2 {
 
     $comprobarAttestation = $true
     if ($env:TC2_BASE_URL) {
-        if ($env:TC2_BASE_URL -notmatch '^(https://|http://(127\.0\.0\.1|localhost)(:|/))') {
+        # Sin @: "http://127.0.0.1:80@otro" iria a "otro".
+        if ($env:TC2_BASE_URL -match '@' -or $env:TC2_BASE_URL -notmatch '^(https://|http://(127\.0\.0\.1|localhost)(:|/))') {
             Fallo "TC2_BASE_URL tiene que ser https, o http a 127.0.0.1 o localhost: $env:TC2_BASE_URL" 'corrige TC2_BASE_URL o quitala'
         }
         if (-not $env:TC2_VERSION) {
@@ -177,6 +178,13 @@ function Install-Tc2 {
     # Absoluto (respecto al directorio actual de PowerShell): el directorio
     # acaba en el PATH del usuario.
     $destino = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($destino)
+    # El ; es el separador del PATH: partiria la entrada en dos.
+    if ($destino.Contains(';')) {
+        Fallo "el directorio de instalacion no puede llevar ';': $destino" 'usa otro TC2_INSTALL_DIR'
+    }
+    if (Test-Path -LiteralPath (Join-Path $destino 'tc2.exe') -PathType Container) {
+        Fallo "$destino\tc2.exe es un directorio" 'quitalo o usa otro TC2_INSTALL_DIR'
+    }
 
     $tmp = Join-Path ([IO.Path]::GetTempPath()) ('tc2-instalar-' + [guid]::NewGuid().ToString('N'))
     New-Item -ItemType Directory -Path $tmp | Out-Null
diff --git a/instalar/install.sh b/instalar/install.sh
index 5de519f..d000313 100644
--- a/instalar/install.sh
+++ b/instalar/install.sh
@@ -119,11 +119,26 @@ sha256_de() {
   fi
 }
 
-# escribir_bloque <perfil> <dir>: idempotente, entre marcas.
+# escribir_bloque <perfil> <dir>: idempotente, entre marcas. Si el perfil ya
+# tiene un bloque de tc2 con otro directorio (se reinstalo en otro sitio), se
+# sustituye: un solo bloque, siempre con el directorio de la ultima
+# instalacion. <dir> ya viene validado (sin comillas, $, ` ni \).
 escribir_bloque() {
   if [ -f "$1" ] && grep -qF "$MARCA_INICIO" "$1"; then
-    decir "$1 ya tiene el bloque de tc2"
-    return 0
+    # shellcheck disable=SC2016
+    if grep -qF "export PATH=\"$2:\$PATH\"" "$1"; then
+      decir "$1 ya tiene el bloque de tc2"
+      return 0
+    fi
+    # cat > en vez de mv: se conservan el propietario, los permisos y un
+    # posible enlace simbolico del perfil.
+    if ! { awk -v i="$MARCA_INICIO" -v f="$MARCA_FIN" '$0 == i { s = 1; next } $0 == f { s = 0; next } !s' "$1" >"$1.tc2-tmp" &&
+      cat "$1.tc2-tmp" >"$1"; }; then
+      rm -f "$1.tc2-tmp"
+      fallo "no se pudo reescribir $1" "quita a mano el bloque entre las marcas de tc2, o repite con TC2_NO_MODIFY_PATH=1"
+    fi
+    rm -f "$1.tc2-tmp"
+    decir "quitado de $1 el bloque de tc2 con otro directorio"
   fi
   # El $PATH del bloque se expande al leer el perfil, no ahora.
   # shellcheck disable=SC2016
@@ -143,6 +158,10 @@ anadir_al_path() {
     decir "$1 no esta en el PATH y TC2_NO_MODIFY_PATH=1: anadelo tu (export PATH=\"$1:\$PATH\")"
     return 0
   fi
+  if [ -z "${HOME:-}" ]; then
+    decir "sin HOME no se puede tocar el perfil del shell: anade tu $1 al PATH (export PATH=\"$1:\$PATH\")"
+    return 0
+  fi
   # .profile lo leen los shells de login POSIX (sh, dash, ash, bash sin
   # .bash_profile); zsh y bash interactivo leen el suyo.
   escribir_bloque "$HOME/.profile" "$1"
@@ -163,7 +182,9 @@ anadir_al_path() {
 main() {
   comprobar_attestation=1
   if [ -n "${TC2_BASE_URL:-}" ]; then
+    # Sin @: "http://127.0.0.1:80@otro" iria a "otro".
     case "$TC2_BASE_URL" in
+      *@*) fallo "TC2_BASE_URL no puede llevar @: $TC2_BASE_URL" "corrige TC2_BASE_URL o quitala" ;;
       https://* | http://127.0.0.1:* | http://127.0.0.1/* | http://localhost:* | http://localhost/*) ;;
       *) fallo "TC2_BASE_URL tiene que ser https, o http a 127.0.0.1 o localhost: $TC2_BASE_URL" "corrige TC2_BASE_URL o quitala" ;;
     esac
@@ -200,6 +221,21 @@ main() {
     /*) ;;
     *) destino="$(pwd)/$destino" ;;
   esac
+  # El directorio se escribe en el perfil del shell: nada que el shell
+  # interprete dentro de comillas dobles, ni el separador del PATH.
+  case "$destino" in
+    *[\"\$\`\\:]*)
+      fallo "el directorio de instalacion no puede llevar comillas, \$, \`, \\ ni dos puntos: $destino" \
+        "usa otro TC2_INSTALL_DIR" ;;
+  esac
+  salto='
+'
+  case "$destino" in
+    *"$salto"*) fallo "el directorio de instalacion no puede llevar saltos de linea" "usa otro TC2_INSTALL_DIR" ;;
+  esac
+  if [ -d "$destino/tc2" ]; then
+    fallo "$destino/tc2 es un directorio" "quitalo o usa otro TC2_INSTALL_DIR"
+  fi
 
   tmp=$(mktemp -d 2>/dev/null || mktemp -d -t tc2-instalar)
   trap 'rm -rf "$tmp"' EXIT
diff --git a/xtask/src/instaladores.rs b/xtask/src/instaladores.rs
index 02bcc86..791349d 100644
--- a/xtask/src/instaladores.rs
+++ b/xtask/src/instaladores.rs
@@ -67,6 +67,64 @@ fn la_tabla_de_alias_de_los_instaladores_es_la_de_p12() {
     assert_eq!(alias_en(&leer("install.ps1")), alias_de_p12(true));
 }
 
+/// Lo que detecta cada script (`uname -s/-m` en sh, PROCESSOR_ARCHITECTURE
+/// del registro en ps1) y el alias que tiene que salir en esa misma rama.
+const DETECCION: [(&str, &str, &str); 6] = [
+    ("install.sh", "Linux/x86_64", "linux-x64"),
+    ("install.sh", "Linux/aarch64", "linux-arm64"),
+    ("install.sh", "Darwin/x86_64", "darwin-x64"),
+    ("install.sh", "Darwin/arm64", "darwin-arm64"),
+    ("install.ps1", "'AMD64'", "win-x64"),
+    ("install.ps1", "'ARM64'", "win-arm64"),
+];
+
+#[test]
+fn cada_sistema_detectado_da_su_alias() {
+    let cubiertos: BTreeSet<String> = DETECCION.iter().map(|d| d.2.to_owned()).collect();
+    assert_eq!(
+        cubiertos,
+        alias_de_p12(false)
+            .union(&alias_de_p12(true))
+            .cloned()
+            .collect()
+    );
+    for (nombre, clave, alias) in DETECCION {
+        let texto = leer(nombre);
+        let ramas: Vec<&str> = texto.lines().filter(|l| l.contains(clave)).collect();
+        assert_eq!(ramas.len(), 1, "{nombre}: una sola rama con {clave}");
+        let esperado: BTreeSet<String> = [alias.to_owned()].into_iter().collect();
+        assert_eq!(
+            alias_en(ramas[0]),
+            esperado,
+            "{nombre}: {clave} tiene que dar {alias}"
+        );
+    }
+}
+
+/// `palabra` como palabra entera (sin letra, digito ni _ a los lados), sin
+/// distinguir mayusculas.
+fn tiene_palabra(linea: &str, palabra: &str) -> bool {
+    let linea = linea.to_ascii_lowercase();
+    let es_de_palabra = |c: char| c.is_ascii_alphanumeric() || c == '_';
+    linea.match_indices(palabra).any(|(i, _)| {
+        let antes = linea[..i].chars().next_back().is_some_and(es_de_palabra);
+        let despues = linea[i + palabra.len()..]
+            .chars()
+            .next()
+            .is_some_and(es_de_palabra);
+        !antes && !despues
+    })
+}
+
+#[test]
+fn tiene_palabra_solo_cuenta_palabras_enteras() {
+    assert!(tiene_palabra("exit 1", "exit"));
+    assert!(tiene_palabra("if ($x) { EXIT }", "exit"));
+    assert!(tiene_palabra("[Environment]::Exit(1)", "exit"));
+    assert!(!tiene_palabra("$exito = 1", "exit"));
+    assert!(!tiene_palabra("Exit_Code", "exit"));
+}
+
 #[test]
 fn instaladores_ascii_y_con_la_llamada_en_la_ultima_linea() {
     for (nombre, llamada) in [
@@ -82,9 +140,12 @@ fn instaladores_ascii_y_con_la_llamada_en_la_ultima_linea() {
         !leer("install.sh").contains('\r'),
         "install.sh tiene finales CRLF"
     );
-    // Con `irm | iex`, un `exit` cerraria la ventana de la persona.
-    let sin_exit = leer("install.ps1")
+    // Con `irm | iex`, un `exit` (o `[Environment]::Exit`) en cualquier
+    // sitio cerraria la ventana de la persona. Los comentarios no cuentan.
+    let con_exit: Vec<String> = leer("install.ps1")
         .lines()
-        .all(|l| !l.trim_start().to_ascii_lowercase().starts_with("exit"));
-    assert!(sin_exit, "install.ps1 usa exit");
+        .filter(|l| !l.trim_start().starts_with('#') && tiene_palabra(l, "exit"))
+        .map(str::to_owned)
+        .collect();
+    assert!(con_exit.is_empty(), "install.ps1 usa exit: {con_exit:?}");
 }
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 6609149e64b240c697b4eba4036d5e8006865279..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-026/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-026/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-026/revision/informe-revision-1-code-quality-reviewer.md         |   21 ++
 tareas/03-en-revision/TASK-026/revision/informe-revision-1-rust-wasm-reviewer.md            |   21 ++
 tareas/03-en-revision/TASK-026/revision/peticion-revision-1-code-quality-reviewer.md        | 1176 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-026/revision/peticion-revision-1-rust-wasm-reviewer.md           |  183 ++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-026/tarea.md                                    |    6 +-
 7 files changed, 1405 insertions(+), 2 deletions(-)
````

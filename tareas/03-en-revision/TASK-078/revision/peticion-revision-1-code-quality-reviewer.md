# Peticion de revision — TASK-078 (ronda 1)

- Tarea: TASK-078 — F2-14 tc2 desinstalar: quitar TaskCodeTwo de forma limpia
- Rama revisada: feature/task-078-f2-14-taskctl-desinstalar-quitar-taskcod
- Rama base: develop
- Commit revisado (HEAD): b855c2057a8375e142ebdec0a665bce971e0af33
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-078 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-078/tarea.md
- tareas/03-en-revision/TASK-078/planificacion/
- .github/scripts/probar-instalador.sh
- docs/distribucion/instalar.md

## Commits a revisar (git log develop..HEAD)

````
b855c20 docs(TASK-078): autorrevision con las revisoras de la tarea
390580b refactor(TASK-078): leer y ruta como texto en un solo sitio de desinstalar
4c8c27e docs(TASK-078): como desinstalar tc2 de la maquina y de un proyecto
5e891ff test(TASK-078): paso de desinstalacion en probar-instalador.sh (sin verificar hasta el push)
6de40bd test(TASK-078): un test por criterio con HOME falso y maquina en memoria
71a2810 feat(TASK-078): tc2 desinstalar con plan como dato, --simular y los dos ambitos
91665e6 feat(TASK-078): puerto git con borrado acotado de la rama de estado y lecturas sin cerrojos opcionales
02f2bd4 feat(TASK-078): marcas del perfil y status line como constantes compartidas con install.sh
9a32c3b refactor(TASK-078): extraer instalacion con Modo, rutas y Entorno inyectado
b6fffce chore(TASK-078): tarea en curso
````

## Diff de tu dominio (2 fichero(s) de 18; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/.github/scripts/probar-instalador.sh b/.github/scripts/probar-instalador.sh
index 25539ae..9918c32 100644
--- a/.github/scripts/probar-instalador.sh
+++ b/.github/scripts/probar-instalador.sh
@@ -212,6 +212,59 @@ path_idempotente() {
   echo "probar: VERDE: el PATH queda una sola vez y TC2_NO_MODIFY_PATH=1 no lo toca"
 }
 
+# TASK-078 (F2-14, C2 y C8): tras instalar con la linea, `tc2 desinstalar
+# --maquina` quita el binario y, en Unix, el bloque del perfil (el resto
+# identico byte a byte); en Windows, su entrada del PATH de usuario (las demas
+# intactas) y el propio tc2.exe en marcha, que borra un auxiliar al salir
+# (<= 10 s). Va la ultima: tambien quita lo que dejaron las pruebas de antes.
+desinstalar_maquina() {
+  claude_vacio="$tmp/claude-vacio"
+  mkdir -p "$claude_vacio"
+  if [ "$so" = windows ]; then
+    dir="$tmp/dest-desinstalar"
+    dest=$(dir_windows "$dir")
+    defecto="$LOCALAPPDATA\\taskcodetwo\\bin"
+    linea_windows "$dest"
+    [ "$(veces_en_path_usuario "$dest")" = 1 ] || rojo "el instalador no anadio $dest al PATH del usuario"
+    # El PATH de usuario esperado: el de ahora sin las entradas de tc2.
+    esperado=$(powershell.exe -NoProfile -Command "@(([Environment]::GetEnvironmentVariable('Path','User') -split ';') | Where-Object { \$_.TrimEnd('\\') -ine '$dest' -and [Environment]::ExpandEnvironmentVariables(\$_).TrimEnd('\\') -ine '$defecto' }) -join ';'" | tr -d '\r')
+    set +e
+    salida=$(TC2_INSTALL_DIR="$dest" CLAUDE_CONFIG_DIR=$(dir_windows "$claude_vacio") "$dir/tc2.exe" desinstalar --maquina 2>&1)
+    codigo=$?
+    set -e
+    echo "$salida"
+    [ "$codigo" = 0 ] || rojo "tc2 desinstalar --maquina salio con $codigo"
+    printf '%s\n' "$salida" | grep -q "decision=borra motivo=al-salir" || rojo "no se programo el autoborrado del binario en marcha"
+    n=0
+    while [ -e "$dir/tc2.exe" ] && [ "$n" -lt 20 ]; do
+      sleep 1
+      n=$((n + 1))
+    done
+    [ ! -e "$dir/tc2.exe" ] || rojo "tras 20 s sigue $dir/tc2.exe"
+    echo "probar: el auxiliar borro tc2.exe en ${n} s (limite 10 s)"
+    [ "$n" -le 10 ] || rojo "el auxiliar tardo ${n} s"
+    [ "$(veces_en_path_usuario "$dest")" = 0 ] || rojo "tras desinstalar, $dest sigue en el PATH del usuario"
+    ahora=$(powershell.exe -NoProfile -Command "[Environment]::GetEnvironmentVariable('Path','User')" | tr -d '\r')
+    [ "$ahora" = "$esperado" ] || rojo "desinstalar cambio otras entradas del PATH de usuario: '$ahora', se esperaba '$esperado'"
+  else
+    h="$tmp/home-desinstalar"
+    mkdir -p "$h"
+    printf '# de la persona\nalias ll="ls -l"\n' >"$h/.profile"
+    antes=$(cksum <"$h/.profile")
+    linea_unix "$h"
+    grep -qF '# >>> tc2 (TaskCodeTwo) >>>' "$h/.profile" || rojo "el instalador no escribio el bloque en .profile"
+    set +e
+    salida=$(env -u XDG_CACHE_HOME HOME="$h" CLAUDE_CONFIG_DIR="$claude_vacio" "$h/.local/bin/tc2" desinstalar --maquina 2>&1)
+    codigo=$?
+    set -e
+    echo "$salida"
+    [ "$codigo" = 0 ] || rojo "tc2 desinstalar --maquina salio con $codigo"
+    [ ! -e "$h/.local/bin/tc2" ] || rojo "tras desinstalar sigue $h/.local/bin/tc2"
+    [ "$(cksum <"$h/.profile")" = "$antes" ] || rojo "tras desinstalar, .profile no es el de antes de instalar"
+  fi
+  echo "probar: VERDE: tc2 desinstalar --maquina deja la maquina como antes de instalar"
+}
+
 sin_version() {
   unset TC2_VERSION
   set +e
@@ -239,6 +292,7 @@ case "$modo" in
     medir
     hash_alterado
     path_idempotente
+    desinstalar_maquina
     ;;
   publico)
     medir
diff --git a/docs/distribucion/instalar.md b/docs/distribucion/instalar.md
index e9b2844..8d31cce 100644
--- a/docs/distribucion/instalar.md
+++ b/docs/distribucion/instalar.md
@@ -66,7 +66,8 @@ curl -fsSL https://github.com/charliebk/TaskCodeTwo/releases/download/v0.0.0-pru
 
    En Windows, reinstalar en otro directorio añade el nuevo al final del PATH
    del usuario, pero no quita el anterior: si el `tc2.exe` viejo sigue ahí,
-   gana él. Hasta `tc2 desinstalar` (F2-14), quita a mano la entrada antigua.
+   gana él. `tc2 desinstalar --maquina` (abajo) quita la del directorio por defecto
+   y, con `TC2_INSTALL_DIR` apuntando a otro, también la de ese y su binario.
    Un directorio con `;` (el separador del PATH) se rechaza.
 
 Volver a ejecutar la línea reinstala (o cambia de versión con `TC2_VERSION`).
@@ -137,6 +138,62 @@ tc2 actualizar --cli --version 2.1.0-rc.1   # una versión concreta (también ha
   comprometer la release compromete binario y `SHA256SUMS` a la vez (ver
   [releases.md](releases.md)).
 
+## Desinstalar
+
+Lo construyó TASK-078 (F2-14, RR-16). Orden recomendado: `--proyecto` en
+cada repo, luego `--maquina` y, al final, el plugin con la orden que imprime.
+
+```bash
+tc2 desinstalar --proyecto --simular   # el plan, sin tocar nada
+tc2 desinstalar --proyecto             # worktrees, .git/taskcodetwo/, exclude
+tc2 desinstalar --proyecto --config --borrar-tareas taskcodetwo/tareas
+tc2 desinstalar --maquina --simular
+tc2 desinstalar --maquina
+```
+
+- **Salida**: una línea por acción, `accion ruta=... clase=... dueno=...
+  decision=borra|conserva|persona|pendiente [motivo=...] [orden=...]`, y al
+  final `desinstalar <resultado> ambito=... borra=N conserva=N persona=N
+  pendiente=N`. Con `--simular` la última lleva `simular=si` y no se toca
+  nada. Una segunda ejecución da `sin-cambios`. Las órdenes que tiene que
+  ejecutar la persona salen también, legibles, por stderr.
+- **Qué quita `--maquina`** (solo lo que reconoce por su ruta, sus marcas o
+  su `command` exactos): el binario de `TC2_INSTALL_DIR` y del directorio por
+  defecto, su `tc2.exe.old` y los temporales `.tc2-*.descarga` y
+  `.tc2.nuevo.*`; el bloque entre marcas de `~/.profile`, `~/.bashrc`,
+  `~/.zshrc` y `~/.bash_profile` (el resto del perfil queda igual byte a
+  byte); en Windows, la entrada del PATH de usuario (la inversa de lo que
+  añadió `install.ps1`; las demás entradas, intactas) y la carpeta
+  `%LOCALAPPDATA%	askcodetwo` si queda vacía; la caché del lanzador
+  (`taskcodetwo/cli/`), y la `statusLine` de `~/.claude/settings.json` solo si
+  su `command` es exactamente `tc2 estado --stdin` (sin tocar ningún otro
+  byte del fichero).
+- **En Windows, el `tc2.exe` en marcha** no se puede borrar: lo borra un
+  proceso auxiliar en cuanto `tc2` sale (como mucho 10 s; `motivo=al-salir`).
+  Si otro proceso está ejecutando ese binario, su grupo (binario, `.old`,
+  carpeta y entrada del PATH) queda `pendiente motivo=en-uso` sin tocar, el
+  resto se quita y la orden sale `rechazada` (3).
+- **Qué no toca**: los hooks de un `settings.json` (si uno nombra `tc2`, lo
+  avisa); el plugin y el marketplace, que son de Claude Code (imprime
+  `claude plugin uninstall taskcodetwo@<marketplace> --scope <scope>`); un
+  `tc2` de la caché del plugin o de un gestor de paquetes (imprime la orden
+  del gestor: `npm uninstall -g taskcodetwo`, `brew uninstall taskcodetwo`,
+  `winget uninstall taskcodetwo`, `scoop uninstall taskcodetwo` o `cargo
+  uninstall tc2`); y nada de la 0.16 (`taskctl`, `taskcode-plugin`, su status
+  line, `.git/taskcode/`), que desinstala F9-10.
+- **Qué quita `--proyecto`**: el worktree de la rama de estado (`tareas/`) y
+  los de tarea (`.taskcodetwo/worktrees/<id>`), con sus registros;
+  `.git/taskcodetwo/` (cerrojo, caché de identidad, índices temporales) y las
+  líneas `/.taskcodetwo/worktrees/` y `/tareas/` de `.git/info/exclude`. Si un
+  worktree tiene cambios sin guardar o commits sin publicar, sale `rechazada`
+  sin tocar nada. Nunca usa la red ni toca el remoto.
+- **Con confirmación**: `.taskcode/config.yml` (solo la de formato 2) y
+  `.taskcode/personas.yml` solo con `--config`; la rama local
+  `taskcodetwo/tareas` y las refs `refs/taskcodetwo/*` solo con
+  `--borrar-tareas taskcodetwo/tareas` (el nombre exacto). La rama remota
+  nunca: imprime `git push origin --delete taskcodetwo/tareas` para que lo
+  decida la persona.
+
 ## Compatibilidad con el plugin
 
 El plugin pregunta al `tc2` que encuentra (ADR F1-14, punto 5):
@@ -206,5 +263,13 @@ hay `gh`.
   Hasta el primer push con el job, **no verificado** en CI (TASK-027); su
   paso en Windows x64 se ensayó en local.
 
+`tc2 desinstalar` se prueba en `cargo test -p tc2`
+(`crates/tc2/tests/desinstalar.rs`) con un HOME falso y el PATH de usuario,
+los procesos y el autoborrado en memoria (nunca los de la máquina), y en
+`probar-instalador.sh local`: tras instalar con la línea, `desinstalar
+--maquina` deja el perfil como antes y, en Windows, quita la entrada del PATH
+de usuario y el `tc2.exe` en marcha. Hasta el primer push con ese paso, **no
+verificado** en CI (TASK-078).
+
 Los runners no son máquinas limpias (traen curl, gh y Node). La prueba en
 máquinas limpias de verdad es de F9-9a (TASK-071).
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-078/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-078/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-078/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````

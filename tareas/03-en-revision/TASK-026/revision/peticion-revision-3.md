# Peticion de revision — TASK-026 (ronda 3)

- Tarea: TASK-026 — F2-11 Instaladores de una linea
- Rama revisada: feature/task-026-f2-11-instaladores-de-una-linea
- Rama base: develop
- Commit revisado (HEAD): 0274adfdab383318f050b9939c0d1f1503c62525
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-026 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde dedddf7567e00cff3b26cb454c4e8b9ac67e7822 (el commit revisado en la ronda anterior)
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
informe de esta ronda (informe-revision-3.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-026/tarea.md
- tareas/03-en-revision/TASK-026/planificacion/
- .github/scripts/probar-instalador.sh
- instalar/install.sh

## Hallazgos de la ronda 2 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 2 hallazgo(s) MENOR de la ronda 2 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | instalar/install.sh:226-231 | informe-revision-2-code-quality-reviewer.md |
| IMP-2 | IMPORTANTE | corregido | instalar/install.sh:126-151 | informe-revision-2-code-quality-reviewer.md |
| R2-1 | CRITICO | corregido | instalar/install.sh:135-136 | informe-revision-2-code-quality-reviewer.md |

## Commits a revisar (git log dedddf7567e00cff3b26cb454c4e8b9ac67e7822..HEAD)

````
0274adf fix(TASK-026): hallazgos de la ronda 2
ff07e6a chore(TASK-026): peticion de revision ronda 2
````

## Diff desde la ronda anterior (git diff dedddf7567e00cff3b26cb454c4e8b9ac67e7822..HEAD)

````diff
diff --git a/.github/scripts/probar-instalador.sh b/.github/scripts/probar-instalador.sh
index dfbec90..25539ae 100644
--- a/.github/scripts/probar-instalador.sh
+++ b/.github/scripts/probar-instalador.sh
@@ -178,6 +178,31 @@ path_idempotente() {
     [ "$n" = 1 ] || rojo "tras reinstalar en otro directorio, .profile tiene $n bloques de tc2"
     ruta=$(terminal_nueva_unix "$h" | head -n 1)
     [ "$ruta" = "$h/otro/tc2" ] || rojo "tras reinstalar en otro directorio, la terminal nueva encuentra '$ruta'"
+    # Con la marca de fin borrada a mano, no se toca el perfil (antes se
+    # llevaba todo lo que habia detras) y se dice que hacer.
+    h="$tmp/home-sin-fin"
+    mkdir -p "$h"
+    # shellcheck disable=SC2016 # $PATH literal, como lo escribe el instalador
+    printf '# >>> tc2 (TaskCodeTwo) >>>\nexport PATH="/viejo:$PATH"\nalias ll="ls -l"\n' >"$h/.profile"
+    antes=$(cksum <"$h/.profile")
+    set +e
+    salida=$(export TC2_INSTALL_DIR="$h/bin"; linea_unix "$h" 2>&1)
+    set -e
+    case "$salida" in
+      *"incompleto o repetido"*) ;;
+      *) echo "$salida"; rojo "con la marca de fin borrada, el instalador no lo dijo" ;;
+    esac
+    [ "$(cksum <"$h/.profile")" = "$antes" ] || rojo "con la marca de fin borrada, se modifico .profile"
+    # Un perfil con finales CRLF: el bloque se reconoce y se sustituye.
+    h="$tmp/home-crlf"
+    mkdir -p "$h"
+    # shellcheck disable=SC2016 # $PATH literal, como lo escribe el instalador
+    printf '# mio\r\n\r\n# >>> tc2 (TaskCodeTwo) >>>\r\nexport PATH="/viejo:$PATH"\r\n# <<< tc2 (TaskCodeTwo) <<<\r\n# mio tambien\r\n' >"$h/.profile"
+    (export TC2_INSTALL_DIR="$h/bin"; linea_unix "$h")
+    n=$(grep -cF '# >>> tc2 (TaskCodeTwo) >>>' "$h/.profile") || n=0
+    [ "$n" = 1 ] || rojo "con un perfil CRLF quedan $n bloques de tc2"
+    grep -qF '# mio tambien' "$h/.profile" || rojo "con un perfil CRLF se perdio una linea de la persona"
+    ! grep -qF '/viejo' "$h/.profile" || rojo "con un perfil CRLF quedo el bloque viejo"
     h="$tmp/home-sin-path"
     mkdir -p "$h"
     (export TC2_NO_MODIFY_PATH=1; linea_unix "$h")
diff --git a/instalar/install.sh b/instalar/install.sh
index d000313..c89213f 100644
--- a/instalar/install.sh
+++ b/instalar/install.sh
@@ -130,9 +130,31 @@ escribir_bloque() {
       decir "$1 ya tiene el bloque de tc2"
       return 0
     fi
+    # Solo se reescribe un bloque entero: una marca de inicio y despues una
+    # de fin. Con las marcas tocadas a mano, quitar "hasta la marca de fin"
+    # podria llevarse el resto del perfil; entonces no se toca. Las marcas se
+    # reconocen tambien con finales CRLF.
+    marcas=$(awk -v i="$MARCA_INICIO" -v f="$MARCA_FIN" '
+      { l = $0; sub(/\r$/, "", l) }
+      l == i { ni++; if (!pi) pi = NR }
+      l == f { nf++; if (!pf) pf = NR }
+      END { print ni + 0, nf + 0, (pi && pf && pi < pf) ? "orden" : "desorden" }' "$1")
+    if [ "$marcas" != "1 1 orden" ]; then
+      fallo "tc2 esta instalado en $2, pero el bloque de tc2 de $1 esta incompleto o repetido (marcas: $marcas); no se ha tocado" \
+        "quita a mano de $1 las lineas entre '$MARCA_INICIO' y '$MARCA_FIN' (ambas incluidas) y repite, o repite con TC2_NO_MODIFY_PATH=1"
+    fi
+    # Fuera el bloque y la linea en blanco que se escribio delante de el.
     # cat > en vez de mv: se conservan el propietario, los permisos y un
     # posible enlace simbolico del perfil.
-    if ! { awk -v i="$MARCA_INICIO" -v f="$MARCA_FIN" '$0 == i { s = 1; next } $0 == f { s = 0; next } !s' "$1" >"$1.tc2-tmp" &&
+    if ! { awk -v i="$MARCA_INICIO" -v f="$MARCA_FIN" '
+      { l = $0; sub(/\r$/, "", l) }
+      l == i { s = 1; hay = 0; next }
+      l == f { s = 0; next }
+      s { next }
+      { if (hay) print pendiente; hay = 0 }
+      l == "" { pendiente = $0; hay = 1; next }
+      { print }
+      END { if (hay) print pendiente }' "$1" >"$1.tc2-tmp" &&
       cat "$1.tc2-tmp" >"$1"; }; then
       rm -f "$1.tc2-tmp"
       fallo "no se pudo reescribir $1" "quita a mano el bloque entre las marcas de tc2, o repite con TC2_NO_MODIFY_PATH=1"
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff dedddf7567e00cff3b26cb454c4e8b9ac67e7822..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/03-en-revision/TASK-026/revision/informe-revision-1-code-quality-reviewer.md  |  99 ++++++++++++++++++++++++++---
 tareas/03-en-revision/TASK-026/revision/informe-revision-1-rust-wasm-reviewer.md     |  84 ++++++++++++++++++++++---
 tareas/03-en-revision/TASK-026/revision/informe-revision-2-code-quality-reviewer.md  |  21 +++++++
 tareas/03-en-revision/TASK-026/revision/informe-revision-2-rust-wasm-reviewer.md     |  21 +++++++
 tareas/03-en-revision/TASK-026/revision/peticion-revision-2-code-quality-reviewer.md | 366 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-026/revision/peticion-revision-2-rust-wasm-reviewer.md    | 361 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-026/tarea.md                                              |   7 ++-
 7 files changed, 942 insertions(+), 17 deletions(-)
````

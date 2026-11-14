#!/usr/bin/env bash
# PoC de F1-3 (TASK-004): flujo de ramas develop/main, rama por tarea con su
# worktree, release sin rama release/* y hotfix con su PR de vuelta a develop.
#
# Simula los comandos de la 2.0 en bash (cli_start, cli_estado, cli_review,
# cli_finish, cli_release, cli_hotfix). El papel de la plataforma (el boton
# de merge del PR, la cola de merge) lo hace un clon aparte, "plataforma",
# con merges sin fast-forward; la forma de la historia es la misma, el metodo
# exacto de GitHub/GitLab es de F1-5 (no verificado aqui).
#
# El estado se escribe como en F1-2 pero sin reintentos (las escrituras de la
# PoC son secuenciales): fetch, operacion, commit y push sin force en la rama
# huerfana taskcode/tareas, montada en tareas/ del worktree principal.
#
# Uso: poc-f1-3.sh <carpeta-de-trabajo>   (vacia o de una ejecucion anterior)
# Solo un remoto bare local con ruta absoluta: sin GitHub ni red externa.
set -u
mkdir -p "$1" || exit 2
base="$(cd "$1" && pwd)"
if [ -n "$(ls -A "$base")" ] && [ ! -f "$base/.poc-f1-3" ]; then
  echo "ERROR: $base no esta vacia ni es de una ejecucion anterior de la PoC"; exit 2
fi
rm -rf "$base"; mkdir -p "$base"; : > "$base/.poc-f1-3"; cd "$base" || exit 2
RAIZ=$(pwd -W 2>/dev/null || pwd)   # ruta absoluta C:/... para git.exe
export GIT_AUTHOR_NAME=poc GIT_AUTHOR_EMAIL=poc@local GIT_COMMITTER_NAME=poc GIT_COMMITTER_EMAIL=poc@local
g() { git -c core.autocrlf=false -c init.defaultBranch=develop -c advice.detachedHead=false "$@"; }
sep() { echo; echo "=== $* ==="; }
resultado() { echo "RESULTADO $*"; echo "RESULTADO $*" >> "$RAIZ/resultados.txt"; }
sino() { if "$@"; then echo si; else echo NO; fi; }
vacio() { [ -z "$1" ]; }

ORIGIN="$RAIZ/origin.git"   # el remoto: ruta absoluta
P="$RAIZ/principal"          # clon de la persona: worktree principal en develop
PL="$RAIZ/plataforma"        # hace de GitHub: mergea los PR y crea los tags
NPR=0

######################## estado (F1-1 + F1-2, simplificado) ########################
tarea() { # tarea <dir-tareas> <id> <estado> <asignado>
  mkdir -p "$1/$2"
  printf -- '---\nid: %s\nestado: %s\nasignado: %s\n---\n' "$2" "$3" "$4" > "$1/$2/tarea.md"
}
roadmap() { # regenera ROADMAP.md en <dir-tareas> (nunca se mergea)
  { echo "# Roadmap"
    for f in "$1"/TASK-*/tarea.md; do [ -f "$f" ] || continue
      echo "- $(sed -n 's/^id: //p' "$f"): $(sed -n 's/^estado: //p' "$f") $(sed -n 's/^asignado: //p' "$f")"
    done; } > "$1/ROADMAP.md"
}
principal_de() { # el worktree principal es la primera entrada de worktree list --porcelain
  g -C "$1" worktree list --porcelain | sed -n '1s/^worktree //p'
}
cli_estado() { # cli_estado <desde-cualquier-worktree> <id> <estado> <quien>
  local desde=$1 id=$2 est=$3 quien=$4 pr t
  pr=$(principal_de "$desde"); t="$pr/tareas"
  g -C "$t" fetch -q origin taskcode/tareas && g -C "$t" reset -q --hard origin/taskcode/tareas || return 4
  tarea "$t" "$id" "$est" "$quien"; roadmap "$t"
  g -C "$t" add -- "$id" ROADMAP.md && g -C "$t" commit -q -m "estado: $id $est ($quien)" || return 1
  g -C "$t" push -q origin taskcode/tareas || return 5
  echo "[estado] $id -> $est ($quien); escrito desde ${desde#"$RAIZ"/} en ${t#"$RAIZ"/}"
}
estado_en_origin() { g -C "$P" show "origin/taskcode/tareas:$1/tarea.md" 2>/dev/null | sed -n 's/^estado: //p'; }

######################## comandos de la 2.0 simulados ########################
rama_de() { # rama_de <tipo> <id> <slug>
  local n=${2#TASK-}; echo "$1/task-$n-$3"
}
ramas_del_id() { # ramas del remoto con el mismo ID, sin mirar el slug (F2-4b)
  local n=${1#TASK-}
  g -C "$P" ls-remote --heads origin "*/task-$n-*" | awk '{print $2}' | sed 's#^refs/heads/##'
}
cli_start() { # cli_start <id> <tipo> <slug> <base-ref> <quien>
  local id=$1 tipo=$2 slug=$3 baseref=$4 quien=$5 rama wt otras
  rama=$(rama_de "$tipo" "$id" "$slug"); wt="$P/.taskcode/worktrees/$id"
  g -C "$P" fetch -q origin --tags || return 4
  otras=$(ramas_del_id "$id")
  if [ -n "$otras" ]; then echo "[start] $id Rechazada: el ID ya tiene rama en el remoto: $otras"; return 3; fi
  cli_estado "$P" "$id" en-curso "$quien" || return $?        # primero el asignado publicado (F1-2)
  g -C "$P" worktree add -q --no-track -b "$rama" "$wt" "$baseref" || return 1
  echo "[start] $id: rama $rama desde $baseref; worktree ${wt#"$RAIZ"/}"
}
cli_review() { # cli_review <id>: push de la rama; el PR contra su base lo abre la plataforma
  local wt="$P/.taskcode/worktrees/$1" rama
  rama=$(g -C "$wt" branch --show-current)
  g -C "$wt" push -q -u origin "$rama" || return 5
  cli_estado "$wt" "$1" en-revision "$(sed -n 's/^asignado: //p' "$P/tareas/$1/tarea.md")" >/dev/null
  echo "[review] $1: push de $rama; PR $rama -> ${2:-develop}"
}
pr_merge() { # pr_merge <base> <rama-head> <titulo>: el boton de merge (merge commit, sin ff)
  local b=$1 h=$2; NPR=$((NPR + 1))
  g -C "$PL" fetch -q origin --tags
  g -C "$PL" checkout -q -B "$b" "origin/$b"
  if ! g -C "$PL" merge -q --no-ff "origin/$h" -m "Merge pull request #$NPR from $h" -m "$3"; then
    echo "[pr #$NPR] $h -> $b: CONFLICTO"; g -C "$PL" merge --abort; return 6
  fi
  g -C "$PL" push -q origin "$b" || { echo "[pr #$NPR] push de $b rechazado"; return 5; }
  echo "[pr #$NPR] $h -> $b: merge $(g -C "$PL" rev-parse --short HEAD)"
}
cli_finish() { # cli_finish <id>: tras el merge del PR; quita el worktree, conserva la rama
  local wt="$P/.taskcode/worktrees/$1" quien
  quien=$(sed -n 's/^asignado: //p' "$P/tareas/$1/tarea.md")
  g -C "$P" worktree remove "$wt" || return 1
  g -C "$P" pull -q --ff-only origin develop || return 1
  cli_estado "$P" "$1" terminada "$quien" >/dev/null
  echo "[finish] $1: worktree quitado; rama conservada; develop del principal al dia"
}
ultimo_tag() { g -C "$PL" describe --tags --abbrev=0 "origin/main" 2>/dev/null; }
notas() { # notas <desde-tag-o-vacio> <hasta-ref>: los merges de PR en el primer padre
  local rango=$2; [ -n "$1" ] && rango="$1..$2"
  g -C "$PL" log --merges --first-parent --format='  - %s: %b' "$rango" | sed '/^$/d'
}
notas_filtradas() { # igual, pero sin los PR cuya rama ya estaba publicada en el tag anterior
  local rango=$2 h p1 p2 resto; [ -n "$1" ] && rango="$1..$2"
  g -C "$PL" log --merges --first-parent --format='%H %P' "$rango" | while read -r h p1 p2 resto; do
    if [ -n "$1" ] && g -C "$PL" merge-base --is-ancestor "$p2" "$1"; then
      echo "  (omitido: $(g -C "$PL" log -1 --format=%s "$h"), su rama ya esta en $1)"
    else
      echo "  - $(g -C "$PL" log -1 --format='%s: %b' "$h" | sed '/^$/d' | tr '\n' ' ')"
    fi
  done
}
cli_release() { # cli_release <vX.Y.Z>: PR develop -> main, tag anotado en el merge
  local v=$1 prev
  prev=$(ultimo_tag)
  pr_merge main develop "Release $v" || return $?
  g -C "$PL" tag -a "$v" -m "Release $v" && g -C "$PL" push -q origin "$v" || return 5
  echo "[release] $v sobre $(g -C "$PL" rev-parse --short "$v^{commit}"); notas desde ${prev:-el inicio} (git log --merges --first-parent):"
  notas "$prev" "origin/develop"
  echo "[release] $v notas filtradas (sin los PR cuya rama ya estaba en ${prev:-ningun tag}):"
  notas_filtradas "$prev" "origin/develop"
}

######################## 0. preparacion ########################
sep "0. Preparacion: remoto bare local, develop y main, rama de estado montada en tareas/"
g init -q --bare "$ORIGIN"
g -C "$ORIGIN" config receive.denyNonFastForwards true
g clone -q "$ORIGIN" "$P" 2>/dev/null
printf '/tareas/\n/.taskcode/worktrees/\n' > "$P/.gitignore"
printf 'linea 1\nlinea 2\nlinea 3\n' > "$P/app.txt"
g -C "$P" add .gitignore app.txt && g -C "$P" commit -q -m "inicial"
g -C "$P" branch main && g -C "$P" push -q origin develop main
g -C "$P" worktree add -q --orphan -b taskcode/tareas "$P/tareas"
for id in TASK-007 TASK-008 TASK-009 TASK-010; do tarea "$P/tareas" "$id" planificada ""; done
roadmap "$P/tareas"
g -C "$P/tareas" add -A . && g -C "$P/tareas" commit -q -m "estado: altas TASK-007..010" && g -C "$P/tareas" push -q -u origin taskcode/tareas
g clone -q "$ORIGIN" "$PL" 2>/dev/null
echo "git: $(git --version)"
echo "remoto: $ORIGIN ; receive.denyNonFastForwards=$(g -C "$ORIGIN" config receive.denyNonFastForwards)"
echo "ramas en origin: $(g -C "$P" ls-remote --heads origin | sed 's#.*refs/heads/##' | tr '\n' ' ')"

######################## 1. dos tareas en paralelo ########################
sep "1a. Dos tareas en paralelo: start de TASK-007 (feature) y TASK-008 (fix), cada una en su worktree"
cli_start TASK-007 feature login origin/develop ana
cli_start TASK-008 fix crash-al-guardar origin/develop beto
g -C "$P" worktree list | sed "s#$RAIZ/##"

sep "1b. Limites: la misma rama no entra en dos worktrees; tareas/ no se monta en un worktree de tarea"
out=$(g -C "$P" worktree add "$P/.taskcode/worktrees/otra" feature/task-007-login 2>&1); rc1=$?
echo "worktree add de feature/task-007-login otra vez (exit $rc1): $out"
out=$(g -C "$P" worktree add "$P/.taskcode/worktrees/TASK-007/tareas" taskcode/tareas 2>&1); rc2=$?
echo "worktree add de taskcode/tareas dentro de TASK-007 (exit $rc2): $out"
resultado "1b: segunda vez la misma rama -> exit $rc1; taskcode/tareas en un worktree de tarea -> exit $rc2 (los dos rechazados por Git: $([ $rc1 != 0 ] && [ $rc2 != 0 ] && echo si || echo NO))"

sep "1c. Trabajo en cada worktree y estado escrito desde el worktree de tarea en tareas/ del principal"
W7="$P/.taskcode/worktrees/TASK-007"; W8="$P/.taskcode/worktrees/TASK-008"
echo "login" > "$W7/login.txt"; g -C "$W7" add login.txt && g -C "$W7" commit -q -m "feat(TASK-007): login"
sed -i 's/linea 3/linea 3 (guardar sin crash)/' "$W8/app.txt"; g -C "$W8" add app.txt && g -C "$W8" commit -q -m "fix(TASK-008): crash al guardar"
h7=$(g -C "$W7" rev-parse HEAD)
echo "principal detectado desde TASK-007: $(principal_de "$W7" | sed "s#$RAIZ/##")"
cli_estado "$W7" TASK-007 en-curso:implementando ana
echo "tareas/ existe dentro del worktree de TASK-007: $([ -e "$W7/tareas" ] && echo si || echo no)"
echo "TASK-007 en tareas/ del principal: $(sed -n 's/^estado: //p' "$P/tareas/TASK-007/tarea.md") ; en origin/taskcode/tareas: $(g -C "$P" fetch -q origin; estado_en_origin TASK-007)"
s7=$(g -C "$W7" status --porcelain); s8=$(g -C "$W8" status --porcelain); sp=$(g -C "$P" status --porcelain); st=$(g -C "$P/tareas" status --porcelain)
echo "git status TASK-007: '$s7' ; TASK-008: '$s8' ; principal (develop): '$sp' ; tareas/: '$st' (vacio = limpio)"
echo "HEAD de TASK-007 sin cambios tras escribir estado: $([ "$h7" = "$(g -C "$W7" rev-parse HEAD)" ] && echo si || echo NO)"
echo "commits de las ramas de codigo que tocan tareas/: $(g -C "$P" log --oneline feature/task-007-login fix/task-008-crash-al-guardar -- tareas | wc -l)"
resultado "1c: estado escrito desde el worktree de tarea en tareas/ del principal y publicado=$([ "$(estado_en_origin TASK-007)" = en-curso:implementando ] && echo si || echo NO); git status limpio en TASK-007, TASK-008, principal y tareas/=$(vacio "$s7$s8$sp$st" && echo si || echo NO)"

sep "1d. review: push de las ramas y unicidad del ID contra el remoto sin mirar el slug"
cli_review TASK-007
cli_review TASK-008
echo "ramas del remoto con el ID 007: $(ramas_del_id TASK-007 | tr '\n' ' ')"
cli_start TASK-007 feature otro-slug origin/develop carla; rc=$?
echo "start de feature/task-007-otro-slug: codigo $rc ; existe la rama en local: $(g -C "$P" branch --list 'feature/task-007-otro-slug' | wc -l)"
resultado "1d: feature/task-007-otro-slug detectada como el mismo ID por ls-remote '*/task-007-*' -> Rechazada (codigo $rc), sin rama ni worktree"

######################## 2. ciclo completo y release v0.1.0 ########################
sep "2a. finish de TASK-007 y TASK-008: merge del PR sin fast-forward en develop, worktree fuera, rama conservada"
pr_merge develop feature/task-007-login "TASK-007 login"
pr_merge develop fix/task-008-crash-al-guardar "TASK-008 crash al guardar"
cli_finish TASK-007
cli_finish TASK-008
echo "worktrees del principal:"; g -C "$P" worktree list | sed "s#$RAIZ/##"
echo "ramas de tarea en local: $(g -C "$P" branch --list 'feature/*' 'fix/*' | tr -d ' *' | tr '\n' ' ')"
echo "ramas de tarea en origin: $(g -C "$P" ls-remote --heads origin 'feature/*' 'fix/*' | sed 's#.*refs/heads/##' | tr '\n' ' ')"
echo "primer padre de develop (origin):"; g -C "$PL" log --first-parent --format='  %h %p | %s' origin/develop
ncom=$(g -C "$PL" rev-list --count --first-parent --no-merges origin/develop)
resultado "2a: 2 merges sin fast-forward en develop; en el primer padre solo hay merges de PR salvo $ncom commit (el inicial); ramas conservadas en local y en origin; worktrees quitados"

sep "2b. release v0.1.0: PR develop -> main con merge commit, tag anotado en el merge, notas desde los PR"
cli_release v0.1.0
tm=$(g -C "$PL" rev-parse "v0.1.0^{tree}"); td=$(g -C "$PL" rev-parse "origin/develop^{tree}")
echo "arbol de v0.1.0 (main): $tm"; echo "arbol de develop       : $td"
echo "tipo del tag: $(g -C "$PL" cat-file -t v0.1.0)"
resultado "2b: v0.1.0 anotado sobre el merge develop->main; arbol de main igual al de develop=$([ "$tm" = "$td" ] && echo si || echo NO) (la cache por arbol de P6 se reutiliza)"

######################## 3. hotfix v0.1.1 y release v0.2.0 ########################
sep "3a. Mientras tanto develop avanza: TASK-010 (feature) se integra antes del hotfix"
cli_start TASK-010 feature informes origin/develop ana
W10="$P/.taskcode/worktrees/TASK-010"
echo "informes" > "$W10/informes.txt"; g -C "$W10" add informes.txt && g -C "$W10" commit -q -m "feat(TASK-010): informes"
cli_review TASK-010
pr_merge develop feature/task-010-informes "TASK-010 informes"
cli_finish TASK-010

sep "3b. hotfix TASK-009 desde el ultimo tag de main, PR a main, tag v0.1.1 y PR automatico contra develop"
tag_base=$(g -C "$P" describe --tags --abbrev=0 origin/main)
echo "ultimo tag de main: $tag_base"
cli_start TASK-009 hotfix corrupcion-al-exportar "$tag_base" beto
W9="$P/.taskcode/worktrees/TASK-009"
sed -i 's/linea 1/linea 1 (exportar sin corromper)/' "$W9/app.txt"; g -C "$W9" add app.txt && g -C "$W9" commit -q -m "fix(TASK-009): corrupcion al exportar"
cli_review TASK-009 main
pr_merge main hotfix/task-009-corrupcion-al-exportar "TASK-009 hotfix"
g -C "$PL" tag -a v0.1.1 -m "Hotfix v0.1.1" && g -C "$PL" push -q origin v0.1.1 && echo "[hotfix] tag v0.1.1 sobre $(g -C "$PL" rev-parse --short 'v0.1.1^{commit}')"
pr_merge develop hotfix/task-009-corrupcion-al-exportar "TASK-009 hotfix (vuelta a develop)"; rcb=$?
g -C "$P" worktree remove "$W9" && g -C "$P" pull -q --ff-only origin develop && cli_estado "$P" TASK-009 terminada beto >/dev/null
echo "diferencia v0.1.0..v0.1.1: $(g -C "$PL" diff --stat v0.1.0 v0.1.1 | tail -1)"
echo "v0.1.1 contiene trabajo de develop posterior a v0.1.0 (informes.txt): $(g -C "$PL" cat-file -e v0.1.1:informes.txt 2>/dev/null && echo SI || echo no)"
resultado "3b: hotfix desde $tag_base, tag v0.1.1 solo con la correccion; PR de vuelta a develop sin conflicto=$([ $rcb = 0 ] && echo si || echo NO)"

sep "3c. release v0.2.0 tras el hotfix"
cli_release v0.2.0; rcr=$?
tm=$(g -C "$PL" rev-parse "v0.2.0^{tree}"); td=$(g -C "$PL" rev-parse "origin/develop^{tree}")
echo "arbol de v0.2.0 (main): $tm"; echo "arbol de develop       : $td"
fix=$(g -C "$PL" show v0.2.0:app.txt | grep -c 'exportar sin corromper'); fix8=$(g -C "$PL" show v0.2.0:app.txt | grep -c 'guardar sin crash')
anc=$(sino g -C "$PL" merge-base --is-ancestor v0.1.1 v0.2.0)
echo "v0.2.0 contiene la correccion de v0.1.1: $fix ; la de TASK-008: $fix8 ; v0.1.1 es antecesor de v0.2.0: $anc"
resultado "3c: release v0.2.0 sin conflicto=$([ $rcr = 0 ] && echo si || echo NO); arbol de main igual al de develop=$([ "$tm" = "$td" ] && echo si || echo NO); correccion del hotfix en v0.2.0=$([ "$fix" = 1 ] && echo si || echo NO)"
nlit=$(notas v0.1.1 origin/develop | grep -c 'from hotfix/'); nfil=$(notas_filtradas v0.1.1 origin/develop | grep -c '^  - .*from hotfix/')
resultado "3c: notas de v0.2.0 con la regla literal (merges del primer padre desde v0.1.1): $nlit PR de hotfix ya publicado en v0.1.1; con el filtro (rama ya contenida en el tag anterior): $nfil"

######################## 4. forma de la historia ########################
sep "4. Forma de la historia en origin (main, develop, tags)"
g -C "$PL" fetch -q origin --tags
g -C "$PL" log --graph --format='%h%d %s' --decorate-refs=refs/tags --decorate-refs=refs/remotes/origin/main --decorate-refs=refs/remotes/origin/develop origin/main origin/develop
echo "tags: $(g -C "$PL" tag -l | tr '\n' ' ')"
echo "primer padre de main: $(g -C "$PL" log --first-parent --format='%s' origin/main | tr '\n' '|')"
echo "estado final en origin/taskcode/tareas:"; g -C "$P" fetch -q origin; g -C "$P" show origin/taskcode/tareas:ROADMAP.md | sed 's/^/  /'
echo "pushes con --force en el guion: 0 ; remoto con receive.denyNonFastForwards=true"

sep "5. Resumen"
cat "$RAIZ/resultados.txt"

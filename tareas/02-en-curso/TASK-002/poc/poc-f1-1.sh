#!/usr/bin/env bash
# PoC de F1-1 (TASK-002): alternativa A (rama huerfana taskcode/tareas montada
# como worktree en tareas/) frente a B (estado en develop).
# Uso: poc-f1-1.sh <carpeta-de-trabajo-vacia>
set -u
base="$(mkdir -p "$1" && cd "$1" && pwd)"; rm -rf "$base"; mkdir -p "$base"; cd "$base" || exit 2
export GIT_AUTHOR_NAME=poc GIT_AUTHOR_EMAIL=poc@local GIT_COMMITTER_NAME=poc GIT_COMMITTER_EMAIL=poc@local
g() { git -c core.autocrlf=false -c init.defaultBranch=develop -c advice.detachedHead=false "$@"; }
sep() { echo; echo "=== $* ==="; }

tarea() { # tarea <dir> <id> <estado>
  mkdir -p "$1/$2"; printf -- '---\nid: %s\nestado: %s\n---\n' "$2" "$3" > "$1/$2/tarea.md"
}
roadmap() { # roadmap <dir>: regenera ROADMAP.md desde las carpetas
  { echo "# Roadmap"; for f in "$1"/TASK-*/tarea.md; do
      id=$(sed -n 's/^id: //p' "$f"); es=$(sed -n 's/^estado: //p' "$f"); echo "- $id: $es"; done; } > "$1/ROADMAP.md"
}

########################## A ##########################
sep "A.1 repo con develop, remoto bare y codigo"
g init -q --bare remoto-a.git
g init -q a && cd a
echo "codigo v1" > app.txt; echo "tareas/" > .gitignore
g add . && g commit -qm "codigo inicial" && g remote add origin "$base/remoto-a.git" && g push -q origin develop
sep "A.2 rama huerfana taskcode/tareas como worktree en tareas/"
g worktree add -q --orphan -b taskcode/tareas tareas
cd tareas; tarea . TASK-001 planificada; roadmap .; g add . && g commit -qm "TASK-001 planificada"; g push -q origin taskcode/tareas; cd ..
g worktree list
echo "merge-base develop taskcode/tareas: $(g merge-base develop taskcode/tareas || echo 'NINGUNA (historias distintas)')"
arbol_antes=$(g rev-parse 'develop^{tree}')
sep "A.3 diez transiciones de estado"
cd tareas
for i in $(seq 2 6); do tarea . "TASK-00$i" planificada; roadmap .; g add . && g commit -qm "TASK-00$i planificada"; done
for i in $(seq 2 6); do tarea . "TASK-00$i" en-curso; roadmap .; g add . && g commit -qm "TASK-00$i en-curso"; done
g push -q origin taskcode/tareas; cd ..
arbol_despues=$(g rev-parse 'develop^{tree}')
echo "arbol develop antes : $arbol_antes"; echo "arbol develop despues: $arbol_despues"
[ "$arbol_antes" = "$arbol_despues" ] && echo "RESULTADO A.3: arbol de develop IDENTICO tras 10 transiciones"
echo "git status (develop):"; g status --short; echo "(vacio = limpio)"
echo "commits en develop: $(g rev-list --count develop) ; que tocan tareas/: $(g log --oneline develop -- tareas | wc -l)"
echo "commits en taskcode/tareas: $(g rev-list --count taskcode/tareas)"
sep "A.4 revert en una rama de codigo no puede tocar tareas/"
g switch -q -c feature/x; echo "codigo v2" > app.txt; g commit -qam "cambio"; g revert --no-edit HEAD >/dev/null
echo "TASK-* en tareas/ tras el revert: $(ls -d tareas/TASK-* | wc -l) (eran 6)"
g switch -q develop
cd ..

sep "A.5 dos clones escriben en carpetas TASK distintas"
g clone -q -b develop remoto-a.git c1; g clone -q -b develop remoto-a.git c2
for c in c1 c2; do (cd $c && g worktree add -q tareas taskcode/tareas 2>/dev/null || g worktree add -q -b taskcode/tareas tareas origin/taskcode/tareas); done
(cd c1/tareas && tarea . TASK-007 planificada && roadmap . && g add . && g commit -qm "TASK-007 (c1)" && g push -q origin taskcode/tareas && echo "c1 push OK")
cd c2/tareas; tarea . TASK-008 planificada; roadmap .; g add . && g commit -qm "TASK-008 (c2)"
g push -q origin taskcode/tareas 2>&1 | grep -E "rejected|fetch first" | head -1 | sed 's/^/c2 push: /'
# protocolo minimo: rebase sobre lo nuevo; ROADMAP.md no se mergea: se regenera
g fetch -q origin
if g rebase -q origin/taskcode/tareas 2>/dev/null; then echo "rebase sin conflicto"; else
  echo "conflicto solo en: $(g diff --name-only --diff-filter=U | tr '\n' ' ')"
  roadmap .; g add ROADMAP.md; GIT_EDITOR=true g rebase --continue >/dev/null && echo "ROADMAP.md regenerado (no mergeado) y rebase continuado"; fi
g push -q origin taskcode/tareas && echo "c2 push OK tras rebase"
echo "ROADMAP final:"; cat ROADMAP.md | tail -3
cd ../..

########################## B ##########################
sep "B.1 estado en develop"
g init -q b && cd b
echo "codigo v1" > app.txt; mkdir tareas; tarea tareas TASK-001 planificada; roadmap tareas
g add . && g commit -qm "inicial"
arboles=$(g rev-parse 'HEAD^{tree}')
for i in $(seq 2 6); do tarea tareas "TASK-00$i" planificada; roadmap tareas; g add . && g commit -qm "TASK-00$i planificada"; arboles="$arboles $(g rev-parse 'HEAD^{tree}')"; done
echo "arboles distintos de develop en 6 estados: $(echo $arboles | tr ' ' '\n' | sort -u | wc -l) (6 = cada transicion invalida la cache)"
sep "B.2 conflicto de ROADMAP al integrar una rama de tarea"
g switch -q -c feature/task-002; echo "codigo de 002" >> app.txt; tarea tareas TASK-002 terminada; roadmap tareas; g commit -qam "TASK-002 cierre"
g switch -q develop; tarea tareas TASK-003 en-curso; roadmap tareas; g commit -qam "TASK-003 en-curso"
if g merge -q --no-ff --no-edit feature/task-002 >/dev/null 2>&1; then echo "merge sin conflicto"; else
  echo "CONFLICTO en: $(g diff --name-only --diff-filter=U | tr '\n' ' ')"; g merge --abort; fi
sep "B.3 caso TASK-172: un revert que viaja con un cierre borra otra tarea"
g switch -q -c feature/task-004 develop
# 1) el cierre de 004 arrastra (por error de rebase/merge) la creacion de TASK-007
tarea tareas TASK-007 planificada; roadmap tareas; g add . && g commit -qm "TASK-007 creada (viaja en la rama de 004)"
echo "codigo de 004" >> app.txt; g commit -qam "codigo 004"
g switch -q develop; g merge -q --no-ff --no-edit feature/task-004
echo "TASK-007 existe en develop: $([ -f tareas/TASK-007/tarea.md ] && echo si || echo no)"
# 2) se revierte el merge de 004 por un fallo de codigo
g revert --no-edit -m 1 HEAD >/dev/null
echo "tras revertir el merge de TASK-004, TASK-007 existe: $([ -f tareas/TASK-007/tarea.md ] && echo si || echo 'NO: BORRADA')"
cd ..
echo; echo "FIN PoC"

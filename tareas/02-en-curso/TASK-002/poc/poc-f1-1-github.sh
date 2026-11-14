#!/usr/bin/env bash
# Parte GitHub de la PoC de F1-1. Uso: poc-f1-1-github.sh <carpeta> <owner/repo>
set -u
base="$(mkdir -p "$1" && cd "$1" && pwd)"; repo="$2"
rm -rf "$base/gh"; mkdir -p "$base/gh"; cd "$base/gh" || exit 2
g() { git -c core.autocrlf=false -c init.defaultBranch=develop "$@"; }
g init -q .
mkdir -p .github/workflows
cat > .github/workflows/ci.yml <<'EOF'
name: ci
on: [push, pull_request]
jobs:
  eco:
    runs-on: ubuntu-latest
    steps:
      - run: echo "ci en ${{ github.ref }}"
EOF
echo "codigo v1" > app.txt; echo "tareas/" > .gitignore
g add . && g commit -qm "codigo inicial con CI"
g remote add origin "https://github.com/$repo.git"
g push -q -u origin develop
# A: rama huerfana con 400 tareas sinteticas y ROADMAP con Mermaid
g worktree add -q --orphan -b taskcode/tareas tareas
cd tareas
{
  echo "# Roadmap"; echo; echo "## Grafo global (400 tareas)"; echo; echo '```mermaid'; echo "graph TD"
  for i in $(seq 2 400); do printf '  T%03d --> T%03d\n' $(( (i*7)%i == 0 ? i-1 : i-1 )) "$i"; done
  echo '```'
  echo; echo "## Grafo de un hito (20 tareas)"; echo; echo '```mermaid'; echo "graph TD"
  for i in $(seq 2 20); do printf '  T%03d --> T%03d\n' $((i-1)) "$i"; done
  echo '```'
} > ROADMAP.md
for i in $(seq 1 400); do id=$(printf 'TASK-%03d' "$i"); mkdir -p "$id"; printf -- '---\nid: %s\nestado: planificada\n---\n' "$id" > "$id/tarea.md"; done
g add . && g commit -qm "400 tareas sinteticas y ROADMAP"
g push -q -u origin taskcode/tareas
cd ..
# rama de tarea en A: solo codigo
g switch -q -c feature/a-task-001; echo "cambio A" >> app.txt; g commit -qam "TASK-001 codigo (A)"; g push -q -u origin feature/a-task-001
# B: estado en develop -> rama de tarea lleva tareas/
g switch -q develop; g switch -q -c feature/b-task-001
mkdir -p estado-b/TASK-001; printf -- '---\nid: TASK-001\nestado: en-revision\n---\n' > estado-b/TASK-001/tarea.md
echo "- TASK-001: en-revision" > estado-b/ROADMAP.md
echo "cambio B" >> app.txt; g add . && g commit -qm "TASK-001 codigo + estado (B)"; g push -q -u origin feature/b-task-001
echo "PUSH OK"

#!/usr/bin/env bash
# Medicion E2E de TASK-043: 3 PR sin cola y 3 con cola en charliebk/taskcodetwo-e2e-github.
# Uso: task-043-medir.sh <fase: antes|despues> <cola: false|true>
set -euo pipefail
R=charliebk/taskcodetwo-e2e-github
SHA=8cbac85c31fcd8fce993e3a53ba564946d25c2b8
BASE=/c/Users/nullcad2025/AppData/Local/pruebas-claude/taskcodetwo
D=$BASE/task-043-e2e
fase=$1; cola=$2
cd "$D"
git checkout -q develop && git pull -q
mkdir -p scripts .github/workflows
cp "$BASE/task-043-stub/verificar.sh" scripts/verificar.sh
cat > .github/workflows/verificacion.yml <<EOF
name: verificacion
on:
  pull_request:
    branches: [develop, main]
permissions: {}
jobs:
  verificacion:
    permissions:
      contents: read
      checks: write
      actions: read
    uses: charliebk/TaskCodeTwo/.github/workflows/verificacion.yml@$SHA
    with:
      comando: scripts/verificar.sh
      cola-de-merge: $cola
EOF
git add -A && git add --chmod=+x scripts/verificar.sh
git commit -q -m "e2e TASK-043: verificacion con cola-de-merge=$cola" || true
git push -q origin develop
for i in 1 2 3; do
  b="med/$fase-$i"
  git checkout -q -B "$b" develop
  mkdir -p med && date +%s%N > "med/$fase-$i.txt"
  git add -A && git commit -q -m "e2e TASK-043: $fase $i"
  git push -q -f origin "$b"
  gh pr create --repo $R --base develop --head "$b" --title "E2E TASK-043 $fase $i" --body "Medicion F5-3, se cierra al terminar." | tail -1
done
git checkout -q develop

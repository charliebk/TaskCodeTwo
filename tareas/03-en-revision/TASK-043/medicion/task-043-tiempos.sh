#!/usr/bin/env bash
# Espera y mide el check verificacion de las PR dadas. Uso: task-043-tiempos.sh <pr>...
set -uo pipefail
R=charliebk/taskcodetwo-e2e-github
for pr in "$@"; do
  h=$(gh pr view "$pr" --repo $R --json headRefOid --jq .headRefOid)
  for i in $(seq 1 60); do
    c=$(gh api "repos/$R/commits/$h/check-runs?check_name=verificacion" --jq '.check_runs[0] | select(.status=="completed") | .completed_at' 2>/dev/null)
    [ -n "$c" ] && break
    sleep 10
  done
  run=$(gh api "repos/$R/actions/runs?head_sha=$h&event=pull_request" --jq '[.workflow_runs[] | select(.name=="verificacion")] | sort_by(.created_at) | .[0] | "\(.created_at) \(.id)"')
  ini=${run% *}
  con=$(gh api "repos/$R/commits/$h/check-runs?check_name=verificacion" --jq '.check_runs[0] | "\(.conclusion)"')
  perf=$(gh api "repos/$R/commits/$h/check-runs?check_name=verificacion" --jq '.check_runs[0].output.summary' | grep -m2 -E "^perfiles=|^cola-de-merge=" | tr '\n' ' ')
  arbol=$(gh api "repos/$R/commits/$h/check-runs?check_name=verificacion" --jq '.check_runs[0].output.summary' | grep -c "^arbol=")
  s=$(( $(date -d "$c" +%s) - $(date -d "$ini" +%s) ))
  echo "pr=$pr run=${run#* } inicio=$ini fin=$c segundos=$s conclusion=$con lineas-arbol=$arbol $perf"
done

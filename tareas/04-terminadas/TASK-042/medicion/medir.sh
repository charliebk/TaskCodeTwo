#!/bin/bash
# Medicion de TASK-042 (F5-2, criterio 1: cero repeticiones sobre el mismo
# arbol). Solo lecturas: `gh api --method GET`.
#
# Uso: medir.sh <repo> <rama del PR> <sha del workflow reutilizable>
#   p. ej. medir.sh charliebk/taskcodetwo-e2e-github cache-f5-2 132a9dd...
#
# Para cada ejecucion `pull_request` de la rama que llama al workflow de
# TaskCode en ese sha, y para cada intento (tambien los "Re-run all jobs"),
# lee el paso «comando del proyecto, perfil <p>» de cada job. Agrupa por
# arbol del head y perfil, y cuenta cuantas veces el paso NO salio `skipped`
# (se ejecuto el comando). Debe ser 1, salvo en los arboles cuyo PR toca
# `.github/workflows/**` o `.taskcode/**` (regla de rutas de F1-8, decision
# 7: ahi se verifica siempre entero).
#
# Arbol del head en vez del de V (el merge ref): con la base fija durante la
# medicion, el arbol de V es funcion del arbol del head.
set -euo pipefail
REPO=$1
RAMA=$2
VERSION=$3
tmp=$(mktemp)
trap 'rm -f "$tmp"' EXIT

runs=$(gh api --method GET "repos/$REPO/actions/runs" -f branch="$RAMA" -f event=pull_request -f per_page=100 \
  --jq ".workflow_runs[] | select([.referenced_workflows[]? | select(.sha == \"$VERSION\")] | length > 0) | \"\(.id) \(.run_attempt) \(.head_sha) \(.pull_requests[0].base.sha // \"-\")\"")

echo "ejecucion intento head arbol_head perfil paso_comando regla_rutas"
while read -r id intentos head base; do
  [ -n "$id" ] || continue
  arbol=$(gh api --method GET "repos/$REPO/commits/$head" --jq '.commit.tree.sha')
  regla=no
  if [ "$base" != "-" ]; then
    n=$(gh api --method GET "repos/$REPO/compare/$base...$head" --jq '[.files[].filename | select(startswith(".github/workflows/") or startswith(".taskcode/"))] | length')
    [ "$n" = 0 ] || regla=si
  fi
  for intento in $(seq 1 "$intentos"); do
    gh api --method GET "repos/$REPO/actions/runs/$id/attempts/$intento/jobs" \
      --jq '.jobs[] | .steps[]? | select(.name | startswith("comando del proyecto, perfil ")) | "\(.name | ltrimstr("comando del proyecto, perfil ")) \(.conclusion)"' \
      | while read -r perfil conclusion; do
          echo "$id $intento ${head:0:12} $arbol $perfil $conclusion $regla"
        done
  done
done <<< "$runs" | sort -k1,1n -k2,2n | tee "$tmp"

echo
echo "arbol_head perfil ejecuciones_del_comando regla_rutas veredicto"
awk '{ k = $4 " " $5; if ($6 != "skipped") n[k]++; else if (!(k in n)) n[k] += 0; r[k] = $7 }
     END { for (k in n) { v = (n[k] == 1) ? "OK" : (r[k] == "si" ? "OK (regla de rutas)" : "REPETIDA"); print k, n[k], r[k], v } }' "$tmp" | sort

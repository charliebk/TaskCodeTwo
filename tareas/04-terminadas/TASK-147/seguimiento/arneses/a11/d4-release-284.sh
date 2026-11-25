#!/bin/bash
# A11-01 (TASK-147): el cuerpo del paso «vY, el mismo codigo con la version
# siguiente» de .github/workflows/release.yml (antes lineas 281-285), con el
# shell que GitHub usa para `shell: bash` (bash --noprofile --norc -eo
# pipefail), en un Cargo.toml cuya linea version NO casa con VERSION: el sed
# no cambia nada. Copia corregida del arnes de TASK-138.
#
# Uso: bash d4-release-284.sh [ruta de release.yml]
# Las cinco lineas se sacan de release.yml (por defecto, la del repo) y se
# comparan con la copia guardada en release-281-285.txt (aviso si difieren).
# Esperado con el arreglo: "::error::..." y salida distinta de 0, sin "SIGUE".
# Sin el arreglo (`! cmp -s ...` a secas), el paso sigue e imprime SIGUE.
set -u
aqui=$(cd "$(dirname "$0")" && pwd)
yml=${1:-$aqui/../../../../../../.github/workflows/release.yml}
d=$(mktemp -d)
trap 'rm -rf "$d"' EXIT
tr -d '\r' <"$yml" | sed -n '/^          base="\${VERSION%%-\*}"$/,/^          rm Cargo.toml.orig$/p' >"$d/lineas.txt"
[ "$(wc -l <"$d/lineas.txt")" -eq 5 ] || { echo "no se encontraron las 5 lineas del paso en $yml" >&2; cat "$d/lineas.txt"; exit 2; }
tr -d '\r' <"$aqui/release-281-285.txt" | cmp -s - "$d/lineas.txt" ||
  echo "AVISO: release-281-285.txt no es la copia de las lineas de $yml"
cd "$d"
printf '[workspace.package]\nversion = "1.2.3"\n' >Cargo.toml
sed 's/^          //' lineas.txt >paso.sh
echo 'echo "SIGUE: el sed no cambio Cargo.toml y la guarda no paro"' >>paso.sh
echo "--- paso.sh"
cat paso.sh
echo "---"
VERSION="9.9.9" bash --noprofile --norc -eo pipefail paso.sh
salida=$?
echo "salida=$salida"
cmp -s Cargo.toml <(printf '[workspace.package]\nversion = "1.2.3"\n') && echo "Cargo.toml sin cambios"
exit "$salida"

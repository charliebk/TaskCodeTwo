#!/bin/sh
# A10-01 (TASK-147): la guarda de probar-actualizar.sh (antes la linea 60) con
# dos SHA256SUMS IGUALES. Copia corregida del arnes de TASK-138: en vez de una
# copia literal, ejecuta la linea tal como esta en el script que se le pasa,
# dentro de un script con `set -eu` (con eval no vale: eval devuelve el 1 de
# `! cmp` y set -e para aunque la guarda no lo haga).
#
# Uso: sh d4-cmp.sh .github/scripts/probar-actualizar.sh
# Esperado con el arreglo: "actualizar: ROJO: ..." y codigo 1, sin "SIGUE".
# Sin el arreglo (`! cmp -s ...` a secas), set -e no para e imprime SIGUE.
set -eu
d=$(mktemp -d)
trap 'rm -rf "$d"' EXIT
# shellcheck disable=SC2016 # se busca el texto literal de la linea
LINEA=$(grep -F '! cmp -s "$tmp/srv/y/SHA256SUMS" "$tmp/srv/malo/SHA256SUMS"' "$1") ||
  { echo "no esta la guarda en $1" >&2; exit 2; }
[ "$(printf '%s\n' "$LINEA" | wc -l)" -eq 1 ] || { echo "no hay una sola linea con la guarda en $1" >&2; exit 2; }
echo "linea: $LINEA"
cat >"$d/paso.sh" <<'FIN'
set -eu
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
rojo() {
  echo "actualizar: ROJO: $1" >&2
  exit 1
}
mkdir -p "$tmp/srv/y" "$tmp/srv/malo"
printf 'aaaa  tc2-1.0.0-linux-x64\n' >"$tmp/srv/y/SHA256SUMS"
cp "$tmp/srv/y/SHA256SUMS" "$tmp/srv/malo/SHA256SUMS"
FIN
printf '%s\n' "$LINEA" >>"$d/paso.sh"
echo 'echo "SIGUE: los ficheros son iguales y la guarda no paro"' >>"$d/paso.sh"
sh "$d/paso.sh"

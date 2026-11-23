#!/bin/sh
# A10: la linea 60 de probar-actualizar.sh con dos SHA256SUMS IGUALES.
# Si la guarda funcionase, set -e pararia el script antes de "SIGUE".
set -eu
tmp=$(mktemp -d)
mkdir -p "$tmp/srv/y" "$tmp/srv/malo"
printf 'aaaa  tc2-1.0.0-linux-x64\n' >"$tmp/srv/y/SHA256SUMS"
cp "$tmp/srv/y/SHA256SUMS" "$tmp/srv/malo/SHA256SUMS"
# Linea 60 literal, extraida del commit auditado:
LINEA=$(sed -n '60p' "$1")
echo "linea 60: $LINEA"
! cmp -s "$tmp/srv/y/SHA256SUMS" "$tmp/srv/malo/SHA256SUMS"
st=$?
[ "$LINEA" = '! cmp -s "$tmp/srv/y/SHA256SUMS" "$tmp/srv/malo/SHA256SUMS"' ] && echo "(la linea ejecutada es identica a la 60)"
echo "SIGUE: los ficheros son iguales y set -e no paro (estado de la linea: $st)"
rm -rf "$tmp"

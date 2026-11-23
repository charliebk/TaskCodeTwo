#!/bin/sh
# A10: sin_version() de probar-instalador.sh (lineas 268-287 del commit
# auditado, extraidas tal cual) con un instalador que sale 0 SIN instalar
# nada: lo que pasa en `curl ... | sh` si curl falla (sh lee una entrada
# vacia y sale 0; sin pipefail, el estado de la tuberia es el de sh).
set -eu
so=unix
tmp=$(mktemp -d)
TC2_VERSION=2.0.0
rojo() { echo "probar: ROJO: $1" >&2; exit 1; }
# Lo que hace linea_unix (linea 47) cuando curl falla: la tuberia da 0.
linea_unix() { false | HOME="$1" SHELL=/bin/sh sh; }
eval "$(sed -n '268,287p' "$1")"
sin_version
echo "binario instalado: $(ls "$tmp/home-ultima/.local/bin/tc2" 2>/dev/null || echo NINGUNO)"
rm -rf "$tmp"

#!/bin/sh
# A10-02 (TASK-147): linea_unix() de probar-instalador.sh, sacada por su
# nombre del script que se le pasa (con bajar() si existe), contra una URL
# donde no escucha nadie (http://127.0.0.1:1). Antes, `curl ... | sh` salia
# con el 0 de sh, que lee una entrada vacia.
#
# Uso: sh d4-linea-unix.sh .github/scripts/probar-instalador.sh
# Esperado con el arreglo: "probar: ROJO: no se pudo bajar ..." y codigo 1.
# Sin el arreglo: "SIGUE" y codigo 0.
set -u
script=$1
base=$(mktemp -d)
trap 'rm -rf "$base"' EXIT
funcion() { # <nombre>: la definicion de la funcion en el script
  sed -n "/^$1() {/,/^}/p" "$script"
}
(
  set -eu
  tmp=$base
  url="http://127.0.0.1:1"
  rojo() {
    echo "probar: ROJO: $1" >&2
    exit 1
  }
  eval "$(funcion bajar)"
  eval "$(funcion linea_unix)"
  mkdir -p "$tmp/home"
  linea_unix "$tmp/home"
  echo "SIGUE: la descarga fallo y linea_unix salio con 0"
)
codigo=$?
echo "codigo=$codigo"
exit "$codigo"

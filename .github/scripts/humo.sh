#!/usr/bin/env bash
# Prueba de humo de un binario publicado (TASK-025, F2-10): arranca sin Node.
#
# - Linux: se ejecuta dentro de un contenedor alpine, sin Node y sin glibc
#   (el binario es musl estatico).
# - Windows y macOS: se quitan del PATH los directorios que tienen `node` y se
#   exige que `node` ya no se encuentre (como la PoC S de ADR F1-14); los
#   runners de GitHub traen Node de serie.
#
# Comprueba que `--version` escribe exactamente «tc2 <version>» con codigo 0
# y que sin argumentos sale con 2 y el texto de uso. Solo usa builtins de bash
# tras limpiar el PATH.
#
# Uso: .github/scripts/humo.sh <binario> <version>
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "Uso: $0 <binario> <version>" >&2
  exit 2
fi
bin="$1"
version="$2"
dir="$(cd "$(dirname "$bin")" && pwd)"
nombre="$(basename "$bin")"
chmod +x "$dir/$nombre"
# Fijada por digest (indice multiarquitectura, x64 y arm64), como las actions.
imagen="alpine:3.22@sha256:49340b346798875b5592e5d420241f2f9cfba5f47e62ebb1446a1b34734e898f"

case "$(uname -s)" in
  Linux)
    if docker run --rm "$imagen" sh -c 'command -v node' >/dev/null; then
      echo "humo: ROJO: hay node en $imagen" >&2
      exit 1
    fi
    echo "humo: node no esta en $imagen"
    tc2() { docker run --rm -v "$dir:/w:ro" "$imagen" "/w/$nombre" "$@"; }
    ;;
  *)
    limpio=""
    IFS=':' read -r -a dirs <<< "$PATH"
    for d in "${dirs[@]}"; do
      if [ -e "$d/node" ] || [ -e "$d/node.exe" ]; then
        echo "humo: fuera del PATH: $d"
        continue
      fi
      limpio="${limpio:+$limpio:}$d"
    done
    PATH="$limpio"
    if command -v node >/dev/null 2>&1; then
      echo "humo: ROJO: node sigue en el PATH: $(command -v node)" >&2
      exit 1
    fi
    echo "humo: node no esta en el PATH"
    tc2() { "$dir/$nombre" "$@"; }
    ;;
esac

salida="$(tc2 --version)"
if [ "$salida" != "tc2 $version" ]; then
  echo "humo: ROJO: --version escribio «$salida», se esperaba «tc2 $version»" >&2
  exit 1
fi
echo "humo: $salida"

set +e
uso="$(tc2 2>&1)"
codigo=$?
set -e
if [ "$codigo" -ne 2 ] || [[ "$uso" != *"tc2 --version"* ]]; then
  echo "humo: ROJO: sin argumentos salio con $codigo y «$uso»" >&2
  exit 1
fi
echo "humo: sin argumentos: codigo 2 y texto de uso"
echo "humo: VERDE: $nombre arranca sin Node"

#!/bin/sh
# Ensayo de `tc2 actualizar --cli` (TASK-027, F2-12). POSIX sh: corre en los
# runners (Windows con el sh de Git).
#
# Uso: probar-actualizar.sh <dir-vx> <version-x> <dir-vy> <version-y>
#
# - <dir-vx>: el binario del build (`tc2-<version-x>-<alias>[.exe]` y su
#   `.sha256`), la version que se instala.
# - <dir-vy>: el mismo build recompilado con la version <version-y> (mayor),
#   la version a la que se actualiza.
#
# Sirve las dos releases en 127.0.0.1 (y una tercera con el hash alterado),
# instala vX con el instalador de una linea y comprueba:
# 1. `actualizar --cli --comprobar` da `actualizar ok actual=X disponible=Y`
#    sin tocar el binario;
# 2. con el hash alterado, `rechazada` (3), el binario no cambia y no quedan
#    `.descarga` ni `.old`;
# 3. `actualizar --cli` sustituye vX por vY: despues `tc2 --version` da vY.
set -eu

if [ "$#" -ne 4 ]; then
  echo "Uso: $0 <dir-vx> <version-x> <dir-vy> <version-y>" >&2
  exit 2
fi
vx_dir=$1
vx=$2
vy_dir=$3
vy=$4
PUERTO=8766
raiz=$(pwd)
tmp=$(mktemp -d)
unset TC2_BASE_URL TC2_INSTALL_DIR TC2_NO_MODIFY_PATH TC2_VERSION || true

rojo() {
  echo "actualizar: ROJO: $1" >&2
  exit 1
}

case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*) so=windows; exe_nombre=tc2.exe ;;
  *) so=unix; exe_nombre=tc2 ;;
esac

suma() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi
}

# --- Las tres releases -------------------------------------------------------
mkdir -p "$tmp/srv/x" "$tmp/srv/y" "$tmp/srv/malo"
cp "$vx_dir"/tc2-* "$tmp/srv/x/"
rm -f "$tmp"/srv/x/*.sha256
cat "$vx_dir"/*.sha256 >"$tmp/srv/x/SHA256SUMS"
cp "$raiz/instalar/install.sh" "$raiz/instalar/install.ps1" "$tmp/srv/x/"
cp "$vy_dir"/tc2-* "$tmp/srv/y/"
rm -f "$tmp"/srv/y/*.sha256
cat "$vy_dir"/*.sha256 >"$tmp/srv/y/SHA256SUMS"
cp "$tmp"/srv/y/tc2-* "$tmp/srv/malo/"
awk '{ c = substr($1, 1, 1); print (c == "0" ? "1" : "0") substr($1, 2) "  " $2 }' \
  "$tmp/srv/y/SHA256SUMS" >"$tmp/srv/malo/SHA256SUMS"
! cmp -s "$tmp/srv/y/SHA256SUMS" "$tmp/srv/malo/SHA256SUMS" || rojo "el SHA256SUMS alterado es igual al bueno"

py=python
command -v python >/dev/null 2>&1 || py=python3
"$py" -m http.server "$PUERTO" --bind 127.0.0.1 --directory "$tmp/srv" >/dev/null 2>&1 &
servidor=$!
trap 'kill "$servidor" 2>/dev/null || true' EXIT
for _ in $(seq 1 30); do
  curl -fsS -o /dev/null "http://127.0.0.1:$PUERTO/x/SHA256SUMS" && break
  sleep 1
done
url="http://127.0.0.1:$PUERTO"

# --- Instalar vX con la linea de instalacion ---------------------------------
inst="$tmp/inst"
mkdir -p "$inst"
if [ "$so" = windows ]; then
  TC2_BASE_URL="$url/x" TC2_VERSION="$vx" TC2_NO_MODIFY_PATH=1 TC2_INSTALL_DIR="$(cygpath -w "$inst")" \
    powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "irm '$url/x/install.ps1' | iex"
else
  curl -fsSL "$url/x/install.sh" | TC2_BASE_URL="$url/x" TC2_VERSION="$vx" TC2_NO_MODIFY_PATH=1 TC2_INSTALL_DIR="$inst" sh
fi
exe="$inst/$exe_nombre"
[ -x "$exe" ] || rojo "no se instalo $exe"
v=$("$exe" --version | tr -d '\r')
[ "$v" = "tc2 $vx" ] || rojo "la instalacion da '$v', se esperaba 'tc2 $vx'"
antes=$(suma "$exe")

# --- 1. --comprobar no toca nada ---------------------------------------------
salida=$(TC2_BASE_URL="$url/y" "$exe" actualizar --cli --comprobar | tr -d '\r')
[ "$salida" = "actualizar ok actual=$vx disponible=$vy" ] || rojo "--comprobar dio '$salida'"
[ "$(suma "$exe")" = "$antes" ] || rojo "--comprobar cambio el binario"
echo "actualizar: VERDE: --comprobar"

# --- 2. hash alterado ----------------------------------------------------------
set +e
salida=$(TC2_BASE_URL="$url/malo" "$exe" actualizar --cli 2>&1)
codigo=$?
set -e
echo "$salida"
[ "$codigo" -eq 3 ] || rojo "con el hash alterado salio con $codigo, se esperaba 3"
case "$salida" in
  *"motivo=sha256-no-casa"*) ;;
  *) rojo "con el hash alterado fallo, pero no por el hash" ;;
esac
[ "$(suma "$exe")" = "$antes" ] || rojo "con el hash alterado cambio el binario"
restos=$(find "$inst" -mindepth 1 ! -name "$exe_nombre")
[ -z "$restos" ] || rojo "con el hash alterado quedaron: $restos"
echo "actualizar: VERDE: un hash que no casa no toca nada"

# --- 3. vX -> vY ---------------------------------------------------------------
salida=$(TC2_BASE_URL="$url/y" "$exe" actualizar --cli | tr -d '\r')
echo "$salida"
case "$salida" in
  "actualizar ok anterior=$vx version=$vy sha256="*) ;;
  *) rojo "actualizar --cli dio '$salida'" ;;
esac
v=$("$exe" --version | tr -d '\r')
[ "$v" = "tc2 $vy" ] || rojo "tras actualizar da '$v', se esperaba 'tc2 $vy'"
# En Windows, el .old lo borra la siguiente ejecucion (la de --version).
restos=$(find "$inst" -mindepth 1 ! -name "$exe_nombre")
[ -z "$restos" ] || rojo "tras actualizar quedaron: $restos"
echo "actualizar: VERDE: vX -> vY sin reinstalar"

kill "$servidor" 2>/dev/null || true
wait "$servidor" 2>/dev/null || true
rm -rf "$tmp"
echo "actualizar: VERDE: $so"

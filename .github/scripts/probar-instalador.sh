#!/bin/sh
# Prueba de los instaladores de una linea (TASK-026, F2-11). POSIX sh: corre
# en los runners (Windows con el sh de Git) y dentro de un contenedor alpine.
#
# Uso: probar-instalador.sh local|publico <url> <TC2_VERSION> <version-del-binario>
#
# - <url>: donde estan install.sh e install.ps1.
# - local: la release servida en 127.0.0.1 (<url> tiene tambien SHA256SUMS y
#   el binario, y <url>/malo los mismos con un SHA256SUMS alterado). Mide la
#   instalacion, comprueba que un hash que no casa no instala nada y que el
#   PATH queda una sola vez aunque se instale dos veces.
# - publico: la linea real contra la release publicada. Mide la instalacion y
#   comprueba que sin TC2_VERSION instala la ultima estable o, si no la hay,
#   falla diciendo que se use TC2_VERSION.
#
# La medida va desde antes de lanzar la linea hasta que un proceso nuevo, que
# lee el PATH como una terminal nueva (sh -l en Unix, el registro en Windows),
# encuentra tc2 y escribe exactamente "tc2 <version>". Limite: 60 s.
set -eu

if [ "$#" -ne 4 ]; then
  echo "Uso: $0 local|publico <url> <TC2_VERSION> <version-del-binario>" >&2
  exit 2
fi
modo=$1
url=${2%/}
export TC2_VERSION="$3"
version_bin=$4
LIMITE=60
tmp=$(mktemp -d)
unset TC2_BASE_URL TC2_INSTALL_DIR TC2_NO_MODIFY_PATH || true

rojo() {
  echo "probar: ROJO: $1" >&2
  exit 1
}

case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*) so=windows ;;
  *) so=unix ;;
esac

# --- Unix: la linea de sh con un HOME nuevo (sin perfil) -------------------

linea_unix() { # <home>
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url/install.sh" | HOME="$1" SHELL=/bin/sh sh
  else
    wget -qO- "$url/install.sh" | HOME="$1" SHELL=/bin/sh sh
  fi
}

# Un proceso nuevo, con el entorno vacio, que lee el perfil como un login.
terminal_nueva_unix() { # <home>
  env -i HOME="$1" PATH=/usr/bin:/bin sh -l -c 'command -v tc2 && tc2 --version'
}

# --- Windows: la linea de PowerShell 5.1 ----------------------------------
#
# Sin cambiar LOCALAPPDATA: ahi guarda PowerShell su cache de modulos, y con
# uno vacio la reconstruye en cada proceso (unos 40 s en windows-11-arm), algo
# que no le pasa a nadie que instala. La medida usa el destino por defecto; el
# resto de pruebas aislan el destino con TC2_INSTALL_DIR.

linea_windows() { # [dir de instalacion en formato Windows]
  if [ -n "${1:-}" ]; then
    TC2_INSTALL_DIR="$1" powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "irm '$url/install.ps1' | iex"
  else
    powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "irm '$url/install.ps1' | iex"
  fi
}

# Un proceso nuevo que lee el PATH del registro, como una ventana nueva.
terminal_nueva_windows() {
  powershell.exe -NoProfile -Command "\$env:Path = [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User'); \$c = Get-Command tc2 -ErrorAction Stop; \$c.Source; & \$c.Source --version" | tr -d '\r'
}

veces_en_path_usuario() { # <dir en formato Windows>
  powershell.exe -NoProfile -Command "@(([Environment]::GetEnvironmentVariable('Path','User') -split ';') | Where-Object { \$_.TrimEnd('\\') -ieq '$1' }).Count" | tr -d '\r'
}

dir_windows() { # <ruta de sh>
  cygpath -w "$1"
}

# --- Las pruebas -----------------------------------------------------------

medir() {
  if [ "$so" = windows ]; then
    # La persona escribe la linea en una ventana de PowerShell ya abierta: el
    # primer arranque de powershell.exe en un runner recien creado (unos 12 s
    # en windows-11-arm) no es parte de la instalacion. Se paga aqui, fuera de
    # la medida, y se deja en el log.
    frio=$(date +%s)
    powershell.exe -NoProfile -Command 'exit 0'
    echo "probar: primer arranque de powershell.exe: $(($(date +%s) - frio)) s (fuera de la medida)"
    # El PSModulePath del runner trae los modulos de Azure (C:\Modules\az_*)
    # y de SQL Server, que un Windows limpio no tiene: para encontrar irm,
    # PowerShell los recorre en cada proceso (47 s medidos en windows-11-arm,
    # TASK-026). Se mide con el PSModulePath de un Windows limpio, y el coste
    # con el del runner queda en el log.
    t=$(date +%s)
    powershell.exe -NoProfile -Command "\$null = Get-Date" >/dev/null
    echo "probar: con el PSModulePath del runner ($PSModulePath), un proceso nuevo que usa un cmdlet de Microsoft.PowerShell.Utility tarda $(($(date +%s) - t)) s (fuera de la medida)"
    PSModulePath="$USERPROFILE\\Documents\\WindowsPowerShell\\Modules;C:\\Program Files\\WindowsPowerShell\\Modules;C:\\Windows\\system32\\WindowsPowerShell\\v1.0\\Modules"
    export PSModulePath
    t=$(date +%s)
    powershell.exe -NoProfile -Command "\$null = Get-Date" >/dev/null
    echo "probar: con el PSModulePath de un Windows limpio ($PSModulePath), el mismo proceso tarda $(($(date +%s) - t)) s"
  fi
  inicio=$(date +%s)
  if [ "$so" = windows ]; then
    linea_windows
    encontrado=$(terminal_nueva_windows)
    esperado_dir="$LOCALAPPDATA\\taskcodetwo\\bin\\tc2.exe"
  else
    mkdir -p "$tmp/home-medida"
    linea_unix "$tmp/home-medida"
    encontrado=$(terminal_nueva_unix "$tmp/home-medida")
    esperado_dir="$tmp/home-medida/.local/bin/tc2"
  fi
  fin=$(date +%s)
  echo "$encontrado"
  ruta=$(printf '%s\n' "$encontrado" | head -n 1)
  salida=$(printf '%s\n' "$encontrado" | tail -n 1)
  [ "$(printf '%s' "$ruta" | tr '[:upper:]' '[:lower:]')" = "$(printf '%s' "$esperado_dir" | tr '[:upper:]' '[:lower:]')" ] ||
    rojo "la terminal nueva encuentra tc2 en '$ruta', se esperaba '$esperado_dir'"
  [ "$salida" = "tc2 $version_bin" ] || rojo "tc2 --version escribio '$salida', se esperaba 'tc2 $version_bin'"
  segundos=$((fin - inicio))
  echo "probar: instalacion en ${segundos} s (limite ${LIMITE} s)"
  [ "$segundos" -lt "$LIMITE" ] || rojo "la instalacion tardo ${segundos} s"
  echo "probar: VERDE: instalacion medida"
}

hash_alterado() {
  export TC2_BASE_URL="$url/malo"
  if [ "$so" = windows ]; then
    if linea_windows "$(dir_windows "$tmp/dest-malo")"; then rojo "con un hash alterado el instalador salio con 0"; fi
    [ ! -e "$tmp/dest-malo/tc2.exe" ] || rojo "con un hash alterado quedo tc2.exe instalado"
  else
    mkdir -p "$tmp/home-malo"
    if linea_unix "$tmp/home-malo"; then rojo "con un hash alterado el instalador salio con 0"; fi
    [ ! -e "$tmp/home-malo/.local/bin/tc2" ] || rojo "con un hash alterado quedo tc2 instalado"
  fi
  export TC2_BASE_URL="$url"
  echo "probar: VERDE: un hash que no casa no instala nada"
}

path_idempotente() {
  if [ "$so" = windows ]; then
    dest=$(dir_windows "$tmp/dest-doble")
    linea_windows "$dest"
    linea_windows "$dest"
    n=$(veces_en_path_usuario "$dest")
    [ "$n" = 1 ] || rojo "tras instalar dos veces, el dir esta $n veces en el PATH del usuario"
    dest=$(dir_windows "$tmp/dest-sin-path")
    (export TC2_NO_MODIFY_PATH=1; linea_windows "$dest")
    [ -e "$tmp/dest-sin-path/tc2.exe" ] || rojo "con TC2_NO_MODIFY_PATH=1 no se instalo tc2.exe"
    n=$(veces_en_path_usuario "$dest")
    [ "$n" = 0 ] || rojo "con TC2_NO_MODIFY_PATH=1 el dir esta $n veces en el PATH del usuario"
  else
    h="$tmp/home-doble"
    mkdir -p "$h"
    linea_unix "$h"
    linea_unix "$h"
    n=$(grep -cF '# >>> tc2 (TaskCodeTwo) >>>' "$h/.profile") || n=0
    [ "$n" = 1 ] || rojo "tras instalar dos veces, .profile tiene $n bloques de tc2"
    h="$tmp/home-sin-path"
    mkdir -p "$h"
    (export TC2_NO_MODIFY_PATH=1; linea_unix "$h")
    [ ! -e "$h/.profile" ] || rojo "con TC2_NO_MODIFY_PATH=1 se escribio $h/.profile"
    [ -x "$h/.local/bin/tc2" ] || rojo "con TC2_NO_MODIFY_PATH=1 no se instalo tc2"
  fi
  echo "probar: VERDE: el PATH queda una sola vez y TC2_NO_MODIFY_PATH=1 no lo toca"
}

sin_version() {
  unset TC2_VERSION
  set +e
  if [ "$so" = windows ]; then
    salida=$(linea_windows "$(dir_windows "$tmp/dest-ultima")" 2>&1)
  else
    mkdir -p "$tmp/home-ultima"
    salida=$(linea_unix "$tmp/home-ultima" 2>&1)
  fi
  codigo=$?
  set -e
  echo "$salida"
  if [ "$codigo" -ne 0 ]; then
    case "$salida" in
      *"no hay ninguna release estable"*TC2_VERSION*) ;;
      *) rojo "sin TC2_VERSION fallo sin decir que se use TC2_VERSION" ;;
    esac
  fi
  echo "probar: VERDE: sin TC2_VERSION, ultima estable o error con el arreglo"
}

case "$modo" in
  local)
    export TC2_BASE_URL="$url"
    medir
    hash_alterado
    path_idempotente
    ;;
  publico)
    medir
    sin_version
    ;;
  *) rojo "modo desconocido: $modo" ;;
esac
rm -rf "$tmp"
echo "probar: VERDE: instalador de $so ($modo)"

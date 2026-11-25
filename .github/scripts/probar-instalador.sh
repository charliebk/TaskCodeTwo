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
#   comprueba que sin TC2_VERSION instala la ultima estable (el binario y su
#   --version, la del tag de /releases/latest) o, si no la hay, falla
#   diciendo que se use TC2_VERSION.
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

bajar() { # <url>: a la salida estandar, con curl o, si no lo hay, wget
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$1"
  else
    wget -qO- "$1"
  fi
}

# Sin pipefail, la tuberia da el codigo de sh, que sale con 0 si la descarga
# falla (lee una entrada vacia): el fallo se marca en un fichero, se da en
# rojo y se devuelve el codigo de sh.
linea_unix() { # <home>
  rm -f "$tmp/descarga-fallo"
  if { bajar "$url/install.sh" || : >"$tmp/descarga-fallo"; } | HOME="$1" SHELL=/bin/sh sh; then
    codigo_sh=0
  else
    codigo_sh=$?
  fi
  [ ! -e "$tmp/descarga-fallo" ] || rojo "no se pudo bajar $url/install.sh"
  return "$codigo_sh"
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
    # Solo informativo, y DENTRO de la medida: lo que tarda en este runner un
    # proceso nuevo de PowerShell que usa un cmdlet de
    # Microsoft.PowerShell.Utility (irm lo es), sin el instalador. En
    # windows-11-arm son 42-50 s (TASK-026; no depende de LOCALAPPDATA ni de
    # los modulos de Azure del PSModulePath); en windows-latest, 0-1 s.
    t=$(date +%s)
    powershell.exe -NoProfile -Command "\$null = Get-Date" >/dev/null
    echo "probar: coste del runner: un proceso nuevo de PowerShell con un cmdlet de Utility tarda $(($(date +%s) - t)) s (incluido en la medida)"
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
  # Tiene que fallar por el hash, no por cualquier otra cosa (un 404, la red).
  set +e
  if [ "$so" = windows ]; then
    salida=$(linea_windows "$(dir_windows "$tmp/dest-malo")" 2>&1)
    codigo=$?
    instalado="$tmp/dest-malo/tc2.exe"
  else
    mkdir -p "$tmp/home-malo"
    salida=$(linea_unix "$tmp/home-malo" 2>&1)
    codigo=$?
    instalado="$tmp/home-malo/.local/bin/tc2"
  fi
  set -e
  echo "$salida"
  [ "$codigo" -ne 0 ] || rojo "con un hash alterado el instalador salio con 0"
  case "$salida" in
    *"no casa con SHA256SUMS"*) ;;
    *) rojo "con un hash alterado el instalador fallo, pero no por el hash" ;;
  esac
  [ ! -e "$instalado" ] || rojo "con un hash alterado quedo $instalado"
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
    # Reinstalar en otro directorio: un solo bloque, con el directorio nuevo.
    (export TC2_INSTALL_DIR="$h/otro"; linea_unix "$h")
    n=$(grep -cF '# >>> tc2 (TaskCodeTwo) >>>' "$h/.profile") || n=0
    [ "$n" = 1 ] || rojo "tras reinstalar en otro directorio, .profile tiene $n bloques de tc2"
    ruta=$(terminal_nueva_unix "$h" | head -n 1)
    [ "$ruta" = "$h/otro/tc2" ] || rojo "tras reinstalar en otro directorio, la terminal nueva encuentra '$ruta'"
    # Con la marca de fin borrada a mano, no se toca el perfil (antes se
    # llevaba todo lo que habia detras) y se dice que hacer.
    h="$tmp/home-sin-fin"
    mkdir -p "$h"
    # shellcheck disable=SC2016 # $PATH literal, como lo escribe el instalador
    printf '# >>> tc2 (TaskCodeTwo) >>>\nexport PATH="/viejo:$PATH"\nalias ll="ls -l"\n' >"$h/.profile"
    antes=$(cksum <"$h/.profile")
    set +e
    salida=$(export TC2_INSTALL_DIR="$h/bin"; linea_unix "$h" 2>&1)
    set -e
    case "$salida" in
      *"incompleto o repetido"*) ;;
      *) echo "$salida"; rojo "con la marca de fin borrada, el instalador no lo dijo" ;;
    esac
    [ "$(cksum <"$h/.profile")" = "$antes" ] || rojo "con la marca de fin borrada, se modifico .profile"
    # Un perfil con finales CRLF: el bloque se reconoce y se sustituye.
    h="$tmp/home-crlf"
    mkdir -p "$h"
    # shellcheck disable=SC2016 # $PATH literal, como lo escribe el instalador
    printf '# mio\r\n\r\n# >>> tc2 (TaskCodeTwo) >>>\r\nexport PATH="/viejo:$PATH"\r\n# <<< tc2 (TaskCodeTwo) <<<\r\n# mio tambien\r\n' >"$h/.profile"
    (export TC2_INSTALL_DIR="$h/bin"; linea_unix "$h")
    n=$(grep -cF '# >>> tc2 (TaskCodeTwo) >>>' "$h/.profile") || n=0
    [ "$n" = 1 ] || rojo "con un perfil CRLF quedan $n bloques de tc2"
    grep -qF '# mio tambien' "$h/.profile" || rojo "con un perfil CRLF se perdio una linea de la persona"
    ! grep -qF '/viejo' "$h/.profile" || rojo "con un perfil CRLF quedo el bloque viejo"
    h="$tmp/home-sin-path"
    mkdir -p "$h"
    (export TC2_NO_MODIFY_PATH=1; linea_unix "$h")
    [ ! -e "$h/.profile" ] || rojo "con TC2_NO_MODIFY_PATH=1 se escribio $h/.profile"
    [ -x "$h/.local/bin/tc2" ] || rojo "con TC2_NO_MODIFY_PATH=1 no se instalo tc2"
  fi
  echo "probar: VERDE: el PATH queda una sola vez y TC2_NO_MODIFY_PATH=1 no lo toca"
}

# TASK-078 (F2-14, C2 y C8): tras instalar con la linea, `tc2 desinstalar
# --maquina` quita el binario y, en Unix, el bloque del perfil (el resto
# identico byte a byte); en Windows, su entrada del PATH de usuario (las demas
# intactas) y el propio tc2.exe en marcha, que borra un auxiliar al salir
# (<= 10 s). Va la ultima: tambien quita lo que dejaron las pruebas de antes.
desinstalar_maquina() {
  claude_vacio="$tmp/claude-vacio"
  mkdir -p "$claude_vacio"
  if [ "$so" = windows ]; then
    dir="$tmp/dest-desinstalar"
    dest=$(dir_windows "$dir")
    defecto="$LOCALAPPDATA\\taskcodetwo\\bin"
    linea_windows "$dest"
    [ "$(veces_en_path_usuario "$dest")" = 1 ] || rojo "el instalador no anadio $dest al PATH del usuario"
    # El PATH de usuario esperado: el de ahora sin las entradas de tc2.
    esperado=$(powershell.exe -NoProfile -Command "@(([Environment]::GetEnvironmentVariable('Path','User') -split ';') | Where-Object { \$_.TrimEnd('\\') -ine '$dest' -and [Environment]::ExpandEnvironmentVariables(\$_).TrimEnd('\\') -ine '$defecto' }) -join ';'" | tr -d '\r')
    set +e
    salida=$(TC2_INSTALL_DIR="$dest" CLAUDE_CONFIG_DIR=$(dir_windows "$claude_vacio") "$dir/tc2.exe" desinstalar --maquina 2>&1)
    codigo=$?
    set -e
    echo "$salida"
    [ "$codigo" = 0 ] || rojo "tc2 desinstalar --maquina salio con $codigo"
    printf '%s\n' "$salida" | grep -q "decision=borra motivo=al-salir" || rojo "no se programo el autoborrado del binario en marcha"
    n=0
    while [ -e "$dir/tc2.exe" ] && [ "$n" -lt 20 ]; do
      sleep 1
      n=$((n + 1))
    done
    [ ! -e "$dir/tc2.exe" ] || rojo "tras 20 s sigue $dir/tc2.exe"
    echo "probar: el auxiliar borro tc2.exe en ${n} s (limite 10 s)"
    [ "$n" -le 10 ] || rojo "el auxiliar tardo ${n} s"
    [ "$(veces_en_path_usuario "$dest")" = 0 ] || rojo "tras desinstalar, $dest sigue en el PATH del usuario"
    ahora=$(powershell.exe -NoProfile -Command "[Environment]::GetEnvironmentVariable('Path','User')" | tr -d '\r')
    [ "$ahora" = "$esperado" ] || rojo "desinstalar cambio otras entradas del PATH de usuario: '$ahora', se esperaba '$esperado'"
  else
    h="$tmp/home-desinstalar"
    mkdir -p "$h"
    printf '# de la persona\nalias ll="ls -l"\n' >"$h/.profile"
    antes=$(cksum <"$h/.profile")
    linea_unix "$h"
    grep -qF '# >>> tc2 (TaskCodeTwo) >>>' "$h/.profile" || rojo "el instalador no escribio el bloque en .profile"
    set +e
    salida=$(env -u XDG_CACHE_HOME HOME="$h" CLAUDE_CONFIG_DIR="$claude_vacio" "$h/.local/bin/tc2" desinstalar --maquina 2>&1)
    codigo=$?
    set -e
    echo "$salida"
    [ "$codigo" = 0 ] || rojo "tc2 desinstalar --maquina salio con $codigo"
    [ ! -e "$h/.local/bin/tc2" ] || rojo "tras desinstalar sigue $h/.local/bin/tc2"
    [ "$(cksum <"$h/.profile")" = "$antes" ] || rojo "tras desinstalar, .profile no es el de antes de instalar"
  fi
  echo "probar: VERDE: tc2 desinstalar --maquina deja la maquina como antes de instalar"
}

# La version de la ultima release estable, como la saca install.sh
# (ultima_version): la redireccion de /releases/latest a /releases/tag/v<v>.
# En modo publico, <url> es .../releases/download/<tag>.
ultima_estable() {
  latest="${url%/releases/download/*}/releases/latest"
  if command -v curl >/dev/null 2>&1; then
    final=$(curl -fsSLI -o /dev/null -w '%{url_effective}' "$latest") || rojo "no se pudo consultar $latest"
  else
    final=$(wget -S --spider "$latest" 2>&1 | sed -n 's/^ *[Ll]ocation: *//p' | tail -n 1)
  fi
  final=$(printf '%s' "$final" | tr -d '\r')
  case "$final" in
    */releases/tag/v*) printf '%s\n' "${final##*/releases/tag/v}" ;;
    '') rojo "no se pudo consultar $latest" ;;
    *) rojo "$latest no lleva a ninguna release estable ('$final')" ;;
  esac
}

sin_version() {
  unset TC2_VERSION
  set +e
  if [ "$so" = windows ]; then
    salida=$(linea_windows "$(dir_windows "$tmp/dest-ultima")" 2>&1)
    codigo=$?
    instalado="$tmp/dest-ultima/tc2.exe"
  else
    mkdir -p "$tmp/home-ultima"
    salida=$(linea_unix "$tmp/home-ultima" 2>&1)
    codigo=$?
    instalado="$tmp/home-ultima/.local/bin/tc2"
  fi
  set -e
  echo "$salida"
  if [ "$codigo" -ne 0 ]; then
    case "$salida" in
      *"no hay ninguna release estable"*TC2_VERSION*) ;;
      *) rojo "sin TC2_VERSION fallo sin decir que se use TC2_VERSION" ;;
    esac
    echo "probar: VERDE: sin TC2_VERSION y sin release estable, error con el arreglo"
    return 0
  fi
  [ -e "$instalado" ] || rojo "sin TC2_VERSION el instalador salio con 0, pero no se instalo $instalado"
  ultima=$(ultima_estable)
  v=$("$instalado" --version | tr -d '\r')
  [ "$v" = "tc2 $ultima" ] || rojo "sin TC2_VERSION se instalo '$v', se esperaba 'tc2 $ultima' (la de /releases/latest)"
  echo "probar: VERDE: sin TC2_VERSION, instala la ultima estable ($ultima)"
}

case "$modo" in
  local)
    export TC2_BASE_URL="$url"
    medir
    hash_alterado
    path_idempotente
    desinstalar_maquina
    ;;
  publico)
    medir
    sin_version
    ;;
  *) rojo "modo desconocido: $modo" ;;
esac
rm -rf "$tmp"
echo "probar: VERDE: instalador de $so ($modo)"

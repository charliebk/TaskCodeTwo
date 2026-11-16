#!/bin/sh
# Instalador de una linea de tc2 (TaskCodeTwo) para macOS y Linux
# (TASK-026, F2-11; ADR F1-14).
#
#   curl -fsSL https://github.com/charliebk/TaskCodeTwo/releases/latest/download/install.sh | sh
#
# Baja de la release el binario de este sistema y SHA256SUMS, comprueba el
# sha256 y, si hay `gh` con sesion, la attestation de procedencia. Despues deja
# `tc2` en ~/.local/bin y, si ese directorio no esta en el PATH, anade al
# perfil del shell un bloque entre marcas (lo quita `tc2 desinstalar`, F2-14).
# Nunca deja en el destino un binario sin comprobar. Solo POSIX sh: ni bash ni
# Node. No toca `taskctl` ni nada de TaskCode 0.16.
#
# Variables (todas opcionales):
#   TC2_VERSION         version que se instala (2.0.0, 2.0.0-rc.1...); por
#                       defecto, la ultima release estable.
#   TC2_INSTALL_DIR     directorio del binario (por defecto ~/.local/bin).
#   TC2_NO_MODIFY_PATH  con 1, no se toca el perfil del shell.
#   TC2_BASE_URL        solo para ensayos: URL con los ficheros de la release
#                       (https, o http a 127.0.0.1 o localhost). Exige
#                       TC2_VERSION y no comprueba la attestation.
#
# Codigo de salida: 0 instalado; 1 error, con una linea de arreglo.
# Todo va dentro de `main`, que se llama en la ultima linea: una descarga
# cortada no ejecuta medio script.

set -eu

REPO="charliebk/TaskCodeTwo"
MARCA_INICIO="# >>> tc2 (TaskCodeTwo) >>>"
MARCA_FIN="# <<< tc2 (TaskCodeTwo) <<<"

decir() {
  printf 'tc2-instalar: %s\n' "$1"
}

fallo() {
  printf 'tc2-instalar: ERROR: %s\n' "$1" >&2
  printf 'tc2-instalar: arreglo: %s\n' "$2" >&2
  exit 1
}

hay() {
  command -v "$1" >/dev/null 2>&1
}

# descargar <url> <fichero>: solo se aceptan redirecciones a https.
descargar() {
  if hay curl; then
    curl -fsSL --proto-redir '=https' --retry 3 -o "$2" "$1"
  elif hay wget; then
    wget -q -O "$2" "$1"
  else
    fallo "no hay curl ni wget" "instala curl (apt-get install curl, apk add curl...) y repite"
  fi
}

# La ultima release estable, sin la API (tiene limite anonimo): la redireccion
# de /releases/latest a /releases/tag/v<version>.
#
# Sin ninguna release estable, GitHub redirige a /releases. Si no hay
# respuesta, el fallo es de red (conexion, proxy, certificado), no de version.
ultima_version() {
  url="https://github.com/$REPO/releases/latest"
  final=""
  conectado=0
  if hay curl; then
    if final=$(curl -fsSLI --proto-redir '=https' --retry 3 -o /dev/null -w '%{url_effective}' "$url"); then
      conectado=1
    fi
  elif hay wget; then
    cabeceras=$(wget -S --spider "$url" 2>&1) || true
    case "$cabeceras" in
      *HTTP/*) conectado=1 ;;
    esac
    final=$(printf '%s\n' "$cabeceras" | sed -n 's/^ *[Ll]ocation: *//p' | tail -n 1)
  else
    fallo "no hay curl ni wget" "instala curl (apt-get install curl, apk add curl...) y repite"
  fi
  if [ "$conectado" = 0 ]; then
    fallo "no se pudo consultar $url" "comprueba la conexion, el proxy y los certificados, o fija la version con TC2_VERSION=<version>"
  fi
  final=$(printf '%s' "$final" | tr -d '\r')
  case "$final" in
    */releases/tag/v*) printf '%s\n' "${final##*/releases/tag/v}" ;;
    *) fallo "no hay ninguna release estable de tc2 todavia" \
      "elige una version de https://github.com/$REPO/releases: curl -fsSL <url de install.sh> | TC2_VERSION=<version> sh" ;;
  esac
}

detectar_alias() {
  so=$(uname -s)
  arq=$(uname -m)
  # Un shell x64 bajo Rosetta en un Mac arm64: se instala el binario nativo.
  if [ "$so" = Darwin ] && [ "$arq" = x86_64 ]; then
    if [ "$(sysctl -n sysctl.proc_translated 2>/dev/null || echo 0)" = 1 ]; then
      arq=arm64
    fi
  fi
  case "$so/$arq" in
    Linux/x86_64 | Linux/amd64) echo linux-x64 ;;
    Linux/aarch64 | Linux/arm64) echo linux-arm64 ;;
    Darwin/x86_64) echo darwin-x64 ;;
    Darwin/arm64 | Darwin/aarch64) echo darwin-arm64 ;;
    MINGW* | MSYS* | CYGWIN*)
      fallo "install.sh es para macOS y Linux" \
        "en Windows, desde PowerShell: irm https://github.com/$REPO/releases/latest/download/install.ps1 | iex" ;;
    *) fallo "sistema no soportado: $so/$arq" "tc2 se publica para Windows, macOS y Linux en x64 y arm64" ;;
  esac
}

sha256_de() {
  if hay sha256sum; then
    sha256sum "$1" | cut -d ' ' -f 1
  elif hay shasum; then
    shasum -a 256 "$1" | cut -d ' ' -f 1
  else
    fallo "no hay sha256sum ni shasum" "instala coreutils y repite"
  fi
}

# escribir_bloque <perfil> <dir>: idempotente, entre marcas. Si el perfil ya
# tiene un bloque de tc2 con otro directorio (se reinstalo en otro sitio), se
# sustituye: un solo bloque, siempre con el directorio de la ultima
# instalacion. <dir> ya viene validado (sin comillas, $, ` ni \).
escribir_bloque() {
  if [ -f "$1" ] && grep -qF "$MARCA_INICIO" "$1"; then
    # shellcheck disable=SC2016
    if grep -qF "export PATH=\"$2:\$PATH\"" "$1"; then
      decir "$1 ya tiene el bloque de tc2"
      return 0
    fi
    # Solo se reescribe un bloque entero: una marca de inicio y despues una
    # de fin. Con las marcas tocadas a mano, quitar "hasta la marca de fin"
    # podria llevarse el resto del perfil; entonces no se toca. Las marcas se
    # reconocen tambien con finales CRLF.
    marcas=$(awk -v i="$MARCA_INICIO" -v f="$MARCA_FIN" '
      { l = $0; sub(/\r$/, "", l) }
      l == i { ni++; if (!pi) pi = NR }
      l == f { nf++; if (!pf) pf = NR }
      END { print ni + 0, nf + 0, (pi && pf && pi < pf) ? "orden" : "desorden" }' "$1")
    if [ "$marcas" != "1 1 orden" ]; then
      fallo "tc2 esta instalado en $2, pero el bloque de tc2 de $1 esta incompleto o repetido (marcas: $marcas); no se ha tocado" \
        "quita a mano de $1 las lineas entre '$MARCA_INICIO' y '$MARCA_FIN' (ambas incluidas) y repite, o repite con TC2_NO_MODIFY_PATH=1"
    fi
    # Fuera el bloque y la linea en blanco que se escribio delante de el.
    # cat > en vez de mv: se conservan el propietario, los permisos y un
    # posible enlace simbolico del perfil.
    if ! { awk -v i="$MARCA_INICIO" -v f="$MARCA_FIN" '
      { l = $0; sub(/\r$/, "", l) }
      l == i { s = 1; hay = 0; next }
      l == f { s = 0; next }
      s { next }
      { if (hay) print pendiente; hay = 0 }
      l == "" { pendiente = $0; hay = 1; next }
      { print }
      END { if (hay) print pendiente }' "$1" >"$1.tc2-tmp" &&
      cat "$1.tc2-tmp" >"$1"; }; then
      rm -f "$1.tc2-tmp"
      fallo "no se pudo reescribir $1" "quita a mano el bloque entre las marcas de tc2, o repite con TC2_NO_MODIFY_PATH=1"
    fi
    rm -f "$1.tc2-tmp"
    decir "quitado de $1 el bloque de tc2 con otro directorio"
  fi
  # El $PATH del bloque se expande al leer el perfil, no ahora.
  # shellcheck disable=SC2016
  {
    printf '\n%s\n' "$MARCA_INICIO"
    printf 'case ":$PATH:" in *":%s:"*) ;; *) export PATH="%s:$PATH" ;; esac\n' "$2" "$2"
    printf '%s\n' "$MARCA_FIN"
  } >>"$1" || fallo "no se pudo escribir en $1" "anade tu mismo $2 al PATH, o repite con TC2_NO_MODIFY_PATH=1"
  decir "anadido $2 al PATH en $1"
}

anadir_al_path() {
  case ":${PATH:-}:" in
    *":$1:"*) return 0 ;;
  esac
  if [ "${TC2_NO_MODIFY_PATH:-}" = 1 ]; then
    decir "$1 no esta en el PATH y TC2_NO_MODIFY_PATH=1: anadelo tu (export PATH=\"$1:\$PATH\")"
    return 0
  fi
  if [ -z "${HOME:-}" ]; then
    decir "sin HOME no se puede tocar el perfil del shell: anade tu $1 al PATH (export PATH=\"$1:\$PATH\")"
    return 0
  fi
  # .profile lo leen los shells de login POSIX (sh, dash, ash, bash sin
  # .bash_profile); zsh y bash interactivo leen el suyo.
  escribir_bloque "$HOME/.profile" "$1"
  case "$(basename "${SHELL:-sh}")" in
    zsh) escribir_bloque "$HOME/.zshrc" "$1" ;;
    bash)
      escribir_bloque "$HOME/.bashrc" "$1"
      if [ -f "$HOME/.bash_profile" ]; then
        escribir_bloque "$HOME/.bash_profile" "$1"
      fi
      ;;
    fish) decir "fish no lee esos perfiles: ejecuta fish_add_path $1" ;;
    *) ;;
  esac
  decir "abre otra terminal (o ejecuta: export PATH=\"$1:\$PATH\") para usar tc2"
}

main() {
  comprobar_attestation=1
  if [ -n "${TC2_BASE_URL:-}" ]; then
    # Sin @: "http://127.0.0.1:80@otro" iria a "otro".
    case "$TC2_BASE_URL" in
      *@*) fallo "TC2_BASE_URL no puede llevar @: $TC2_BASE_URL" "corrige TC2_BASE_URL o quitala" ;;
      https://* | http://127.0.0.1:* | http://127.0.0.1/* | http://localhost:* | http://localhost/*) ;;
      *) fallo "TC2_BASE_URL tiene que ser https, o http a 127.0.0.1 o localhost: $TC2_BASE_URL" "corrige TC2_BASE_URL o quitala" ;;
    esac
    if [ -z "${TC2_VERSION:-}" ]; then
      fallo "TC2_BASE_URL exige TC2_VERSION" "define TC2_VERSION=<version>"
    fi
    comprobar_attestation=0
  fi

  version=${TC2_VERSION:-}
  if [ -z "$version" ]; then
    version=$(ultima_version)
  fi
  version=${version#v}
  case "$version" in
    '' | *[!0-9A-Za-z.-]*) fallo "version no valida: '$version'" "usa una version como 2.0.0 o 2.0.0-rc.1" ;;
  esac
  # Un tag de prerelease es v<version>-<sufijo>; el binario lleva <version>.
  version_bin=${version%%-*}
  base=${TC2_BASE_URL:-https://github.com/$REPO/releases/download/v$version}
  base=${base%/}

  alias_p12=$(detectar_alias)
  fichero="tc2-$version_bin-$alias_p12"
  if [ -n "${TC2_INSTALL_DIR:-}" ]; then
    destino=$TC2_INSTALL_DIR
  elif [ -n "${HOME:-}" ]; then
    destino="$HOME/.local/bin"
  else
    fallo "no hay HOME" "define TC2_INSTALL_DIR=<directorio>"
  fi
  # Absoluto: el directorio acaba en el PATH y en el perfil del shell.
  case "$destino" in
    /*) ;;
    *) destino="$(pwd)/$destino" ;;
  esac
  # El directorio se escribe en el perfil del shell: nada que el shell
  # interprete dentro de comillas dobles, ni el separador del PATH.
  case "$destino" in
    *[\"\$\`\\:]*)
      fallo "el directorio de instalacion no puede llevar comillas, \$, \`, \\ ni dos puntos: $destino" \
        "usa otro TC2_INSTALL_DIR" ;;
  esac
  salto='
'
  case "$destino" in
    *"$salto"*) fallo "el directorio de instalacion no puede llevar saltos de linea" "usa otro TC2_INSTALL_DIR" ;;
  esac
  if [ -d "$destino/tc2" ]; then
    fallo "$destino/tc2 es un directorio" "quitalo o usa otro TC2_INSTALL_DIR"
  fi

  tmp=$(mktemp -d 2>/dev/null || mktemp -d -t tc2-instalar)
  trap 'rm -rf "$tmp"' EXIT
  trap 'rm -rf "$tmp"; exit 1' INT TERM

  decir "instalando tc2 $version ($alias_p12) desde $base"
  descargar "$base/SHA256SUMS" "$tmp/SHA256SUMS" ||
    fallo "no se pudo bajar $base/SHA256SUMS" "comprueba la version en https://github.com/$REPO/releases y la conexion"
  esperado=$(awk -v f="$fichero" '$2 == f { print $1 }' "$tmp/SHA256SUMS")
  lineas=$(printf '%s\n' "$esperado" | grep -c .) || lineas=0
  if [ "$lineas" -ne 1 ]; then
    fallo "SHA256SUMS no trae una sola linea para $fichero ($lineas)" "esa release no tiene binario para $alias_p12: prueba otra TC2_VERSION"
  fi
  descargar "$base/$fichero" "$tmp/$fichero" ||
    fallo "no se pudo bajar $base/$fichero" "repite; si sigue, comprueba la conexion o el proxy"
  real=$(sha256_de "$tmp/$fichero")
  if [ "$real" != "$esperado" ]; then
    fallo "el sha256 de $fichero no casa con SHA256SUMS (esperado $esperado, obtenido $real); no se ha instalado nada" \
      "repite la descarga; si sigue, avisa en https://github.com/$REPO/issues"
  fi
  decir "sha256 correcto: $real"

  if [ "$comprobar_attestation" = 0 ]; then
    decir "TC2_BASE_URL: no se comprueba la attestation (solo ensayos)"
  elif hay gh && gh auth status >/dev/null 2>&1; then
    gh attestation verify "$tmp/$fichero" --repo "$REPO" \
      --signer-workflow "$REPO/.github/workflows/release.yml" >/dev/null ||
      fallo "la attestation de procedencia de $fichero no es valida; no se ha instalado nada" \
        "avisa en https://github.com/$REPO/issues"
    decir "attestation correcta: lo construyo $REPO con release.yml"
  else
    decir "aviso: sin gh con sesion no se comprueba la attestation (el sha256 si); ver docs/distribucion/instalar.md"
  fi

  mkdir -p "$destino" || fallo "no se pudo crear $destino" "usa TC2_INSTALL_DIR=<directorio con permiso de escritura>"
  nuevo="$destino/.tc2.nuevo.$$"
  if ! { cp "$tmp/$fichero" "$nuevo" && chmod 755 "$nuevo"; }; then
    rm -f "$nuevo"
    fallo "no se pudo escribir en $destino" "usa TC2_INSTALL_DIR=<directorio con permiso de escritura>"
  fi
  salida=$("$nuevo" --version 2>&1) || salida="(no arranca: $salida)"
  if [ "$salida" != "tc2 $version_bin" ]; then
    rm -f "$nuevo"
    fallo "tc2 --version escribio '$salida', se esperaba 'tc2 $version_bin'; no se ha instalado nada" \
      "avisa en https://github.com/$REPO/issues"
  fi
  mv -f "$nuevo" "$destino/tc2" || {
    rm -f "$nuevo"
    fallo "no se pudo dejar tc2 en $destino" "cierra lo que este usando $destino/tc2 y repite"
  }

  anadir_al_path "$destino"
  decir "instalado: $destino/tc2 ($salida)"
}

main "$@"

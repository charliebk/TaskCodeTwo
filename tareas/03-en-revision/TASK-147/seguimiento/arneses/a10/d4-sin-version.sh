#!/bin/sh
# A10-02 (TASK-147): sin_version() de probar-instalador.sh, sacada por su
# nombre del script que se le pasa (con ultima_estable() si existe), con tres
# instaladores falsos y un /releases/latest servido en 127.0.0.1 que redirige
# a /releases/tag/v2.0.0. Copia corregida del arnes de TASK-138 (que solo
# tenia el caso a).
#
# Uso: sh d4-sin-version.sh .github/scripts/probar-instalador.sh
#
# a) el instalador sale 0 sin instalar (lo que hace `curl ... | sh` si curl
#    falla): se espera ROJO.
# c) instala un tc2 que dice `tc2 1.9.0`: se espera ROJO.
# d) instala un tc2 que dice `tc2 2.0.0` (contraste): se espera VERDE.
# Sale con 0 si los tres casos dan lo esperado, con 1 si no.
set -u
script=$1
PUERTO=8799
base=$(mktemp -d)
servidor=
terminar() {
  [ -z "$servidor" ] || kill "$servidor" 2>/dev/null || true
  rm -rf "$base"
}
trap terminar EXIT

funcion() { # <nombre>: la definicion de la funcion en el script
  sed -n "/^$1() {/,/^}/p" "$script"
}

cat >"$base/srv.py" <<'FIN'
import http.server
import sys


class H(http.server.BaseHTTPRequestHandler):
    def responder(self):
        if self.path.endswith("/releases/latest"):
            self.send_response(302)
            destino = self.path[: -len("/latest")] + "/tag/v" + sys.argv[2]
            self.send_header("Location", destino)
        else:
            self.send_response(200)
        self.send_header("Content-Length", "0")
        self.end_headers()

    do_HEAD = responder
    do_GET = responder

    def log_message(self, *args):
        pass


http.server.HTTPServer(("127.0.0.1", int(sys.argv[1])), H).serve_forever()
FIN
py=python
command -v python >/dev/null 2>&1 || py=python3
"$py" -I "$base/srv.py" "$PUERTO" 2.0.0 &
servidor=$!
for _ in 1 2 3 4 5 6 7 8 9 10; do
  curl -fsS -o /dev/null "http://127.0.0.1:$PUERTO/" 2>/dev/null && break
  sleep 1
done

bien=1
caso() { # <nombre> <codigo esperado: 0 o 1> <definicion de linea_unix>
  (
    set -eu
    so=unix
    tmp=$(mktemp -d "$base/caso.XXXX")
    url="http://127.0.0.1:$PUERTO/o/r/releases/download/v2.1.0-rc.1"
    TC2_VERSION=2.1.0-rc.1
    rojo() {
      echo "probar: ROJO: $1" >&2
      exit 1
    }
    eval "$3"
    eval "$(funcion ultima_estable)"
    eval "$(funcion sin_version)"
    sin_version
  )
  codigo=$?
  [ "$codigo" -ne 0 ] && codigo=1
  if [ "$codigo" = "$2" ]; then
    echo "caso $1: codigo=$codigo (esperado $2)"
  else
    echo "caso $1: codigo=$codigo, NO ESPERADO (esperado $2)"
    bien=0
  fi
}

# shellcheck disable=SC2016 # las definiciones se evaluan dentro de caso
caso "a, sale 0 sin instalar" 1 'linea_unix() { false | HOME="$1" SHELL=/bin/sh sh; }'
# shellcheck disable=SC2016
instala='linea_unix() { mkdir -p "$1/.local/bin"; printf "#!/bin/sh\necho \"tc2 %s\"\n" "$V" >"$1/.local/bin/tc2"; chmod +x "$1/.local/bin/tc2"; }'
caso "c, instala otra version" 1 "V=1.9.0; $instala"
caso "d, instala la de /releases/latest" 0 "V=2.0.0; $instala"

if [ "$bien" = 1 ]; then
  echo "ARNES: los tres casos dan lo esperado"
  exit 0
fi
echo "ARNES: algun caso no da lo esperado"
exit 1

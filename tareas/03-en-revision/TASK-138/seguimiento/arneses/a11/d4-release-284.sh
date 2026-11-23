#!/bin/bash
# Arnes A11 (TASK-138): ejecuta las lineas 281-285 de .github/workflows/release.yml
# (paso «vY, el mismo codigo con la version siguiente») tal cual, con el shell
# que GitHub usa para `shell: bash` (bash --noprofile --norc -eo pipefail), en
# un Cargo.toml cuya linea version NO casa con VERSION: el sed no cambia nada.
# Si la guarda `! cmp -s` de la linea 284 parase, no se imprimiria SIGUE.
set -u
aqui=$(cd "$(dirname "$0")" && pwd)
d=$(mktemp -d "$aqui/tmp-284.XXXX")
cd "$d"
printf '[workspace.package]\nversion = "1.2.3"\n' > Cargo.toml
# Cuerpo: las 5 lineas del commit auditado (sin sangria) y un marcador.
sed 's/^          //' "$aqui/release-281-285.txt" > paso.sh
echo 'echo "SIGUE: el sed no cambio Cargo.toml y la guarda de la linea 284 no paro"' >> paso.sh
echo "--- paso.sh"; cat paso.sh; echo "---"
VERSION="9.9.9" bash --noprofile --norc -eo pipefail paso.sh
echo "salida=$?"
cmp -s Cargo.toml <(printf '[workspace.package]\nversion = "1.2.3"\n') && echo "Cargo.toml sin cambios"
cd "$aqui" && rm -rf "$d"

# Contraste: las lineas 208-214 (la guarda `! cmp -s` es la ULTIMA orden del
# paso) con dos SHA256SUMS que salen iguales: el paso si falla.
d=$(mktemp -d "$aqui/tmp-214.XXXX")
cd "$d"
printf 'x  tc2-a\n' > a.sha256
mkdir -p ../instalar 2>/dev/null
sed 's/^          //' "$aqui/release-208-214.txt" > paso.sh
# Sustituye el awk por una copia identica para forzar dos ficheros iguales.
sed -i 's/^awk .*$/cp SHA256SUMS malo\/SHA256SUMS/' paso.sh
sed -i 's/^cp \.\.\/instalar.*$/: sin instaladores/' paso.sh
echo "--- paso.sh (contraste)"; cat paso.sh; echo "---"
touch tc2-a
bash --noprofile --norc -eo pipefail paso.sh
echo "salida contraste=$?"
cd "$aqui" && rm -rf "$d"

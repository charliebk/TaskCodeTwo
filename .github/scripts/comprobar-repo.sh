#!/usr/bin/env bash
# CI minima de TaskCodeTwo (TASK-001, F1-0).
#
# Comprueba la estructura de tareas/ y los enlaces relativos de la
# documentacion. Sale con codigo distinto de 0 y una linea por fallo.
#
# Uso: .github/scripts/comprobar-repo.sh [raiz-del-repo]
#
# Mientras las tareas vivan en las ramas de codigo (TaskCode 0.16), estas
# comprobaciones tienen sentido aqui. Cuando la 2.0 las lleve a su rama de
# estado (F1-1), se moveran o se retiraran.

set -u
shopt -s nullglob

raiz="${1:-.}"
cd "$raiz" || { echo "ERROR: no existe la raiz $raiz"; exit 2; }

fallos=0
fallo() {
  echo "FALLO: $*"
  fallos=$((fallos + 1))
}

# --- a) carpetas de estado -------------------------------------------------

declare -A estado_de_carpeta=(
  [00-planificadas]=planificada
  [01-en-diseno]=en-diseno
  [02-en-curso]=en-curso
  [03-en-revision]=en-revision
  [04-terminadas]=terminada
)

for carpeta in "${!estado_de_carpeta[@]}"; do
  [[ -d "tareas/$carpeta" ]] || fallo "falta la carpeta tareas/$carpeta"
done

# --- b) frontmatter de cada tarea ------------------------------------------

declare -A tarea_existe=()   # id -> ruta
declare -A deps_de=()        # id -> dependencias separadas por espacios
ntareas=0

for carpeta in "${!estado_de_carpeta[@]}"; do
  for dir in tareas/"$carpeta"/TASK-*/; do
    dir="${dir%/}"
    nombre="${dir##*/}"
    fichero="$dir/tarea.md"
    if [[ ! -f "$fichero" ]]; then
      fallo "$dir no tiene tarea.md"
      continue
    fi
    ntareas=$((ntareas + 1))

    id="" estado="" deps="" en_bloque=0 linea_n=0 cerrado=0 en_deps_bloque=0
    while IFS= read -r linea || [[ -n "$linea" ]]; do
      linea="${linea%$'\r'}"
      linea_n=$((linea_n + 1))
      if (( linea_n == 1 )); then
        [[ "$linea" == "---" ]] && en_bloque=1 && continue
        break
      fi
      if [[ "$linea" == "---" ]]; then cerrado=1; break; fi
      # dependencias en lista de bloque YAML ("- TASK-NNN")
      if (( en_deps_bloque )); then
        if [[ "$linea" =~ ^[[:space:]]*-[[:space:]]*\"?([^\"[:space:]]+)\"?[[:space:]]*$ ]]; then
          deps+=" ${BASH_REMATCH[1]}"
          continue
        fi
        en_deps_bloque=0
      fi
      if [[ "$linea" =~ ^id:[[:space:]]*\"?([^\"[:space:]]+)\"?[[:space:]]*$ ]]; then
        id="${BASH_REMATCH[1]}"
      elif [[ "$linea" =~ ^estado:[[:space:]]*\"?([^\"[:space:]]+)\"?[[:space:]]*$ ]]; then
        estado="${BASH_REMATCH[1]}"
      elif [[ "$linea" =~ ^dependencias:[[:space:]]*\[(.*)\][[:space:]]*$ ]]; then
        lista="${BASH_REMATCH[1]//,/ }"
        lista="${lista//\"/}"
        deps+=" $lista"
      elif [[ "$linea" =~ ^dependencias:[[:space:]]*$ ]]; then
        en_deps_bloque=1
      fi
    done < "$fichero"

    if (( ! en_bloque )); then
      fallo "$fichero no abre con frontmatter (---)"
      continue
    fi
    (( cerrado )) || fallo "$fichero: el frontmatter no se cierra con ---"

    if [[ -z "$id" ]]; then
      fallo "$fichero: falta el campo id"
    elif [[ "$id" != "$nombre" ]]; then
      fallo "$fichero: id $id no coincide con la carpeta $nombre"
    fi
    if [[ -n "$id" ]]; then
      if [[ -n "${tarea_existe[$id]:-}" ]]; then
        fallo "id $id repetido: ${tarea_existe[$id]} y $fichero"
      else
        tarea_existe[$id]="$fichero"
      fi
      deps_de[$id]="$deps"
    fi

    esperado="${estado_de_carpeta[$carpeta]}"
    if [[ -z "$estado" ]]; then
      fallo "$fichero: falta el campo estado"
    elif [[ "$estado" != "$esperado" ]]; then
      fallo "$fichero: estado $estado en la carpeta $carpeta (se espera $esperado)"
    fi
  done
done

# --- c) dependencias existentes --------------------------------------------

for id in "${!deps_de[@]}"; do
  for dep in ${deps_de[$id]}; do
    [[ -n "${tarea_existe[$dep]:-}" ]] || fallo "$id depende de $dep, que no existe"
    [[ "$dep" == "$id" ]] && fallo "$id depende de si misma"
  done
done

# --- d) enlaces relativos de la documentacion ------------------------------

nenlaces=0
comprobar_enlaces() {
  local md="$1" base linea resto destino ruta en_codigo=0
  base="${md%/*}"
  [[ "$base" == "$md" ]] && base="."
  while IFS= read -r linea || [[ -n "$linea" ]]; do
    linea="${linea%$'\r'}"
    if [[ "$linea" =~ ^[[:space:]]*(\`\`\`|~~~) ]]; then
      en_codigo=$((1 - en_codigo))
      continue
    fi
    (( en_codigo )) && continue
    resto="$linea"
    while [[ "$resto" =~ \]\(([^\)[:space:]]+)(\ \"[^\"]*\")?\) ]]; do
      destino="${BASH_REMATCH[1]}"
      resto="${resto#*"${BASH_REMATCH[0]}"}"
      case "$destino" in
        http://*|https://*|mailto:*|\#*) continue ;;
      esac
      ruta="${destino%%#*}"
      ruta="${ruta%%:[0-9]*}"   # file.md:42 (enlace a linea)
      [[ -z "$ruta" ]] && continue
      nenlaces=$((nenlaces + 1))
      if [[ "$ruta" == /* ]]; then
        [[ -e ".$ruta" ]] || fallo "$md: enlace roto $destino"
      else
        [[ -e "$base/$ruta" ]] || fallo "$md: enlace roto $destino"
      fi
    done
  done < "$md"
}

docs_md=(README.md CLAUDE.md)
while IFS= read -r -d '' f; do docs_md+=("${f#./}"); done < <(find ./docs -name '*.md' -print0 2>/dev/null)
for md in "${docs_md[@]}"; do
  [[ -f "$md" ]] && comprobar_enlaces "$md"
done

# --- resumen ---------------------------------------------------------------

echo "Tareas: $ntareas; enlaces relativos: $nenlaces; fallos: $fallos"
(( fallos == 0 ))

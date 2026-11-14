#!/usr/bin/env bash
# PoC de F1-5 (TASK-006): puertas en GitHub por carril (rulesets, checks
# obligatorios con su origen, CODEOWNERS). SOLO LECTURA en GitHub.
#
# (A) Estado real en GitHub, solo con GET (gh api --method GET):
#     rulesets, reglas que aplican a develop y proteccion clasica de los dos
#     repos que se pueden leer (codigo HTTP y mensaje), su visibilidad,
#     CODEOWNERS de TaskCodeTwo y el id de la App de GitHub Actions.
# (B) Esquema: descarga la descripcion OpenAPI publica de GitHub
#     (github/rest-api-description, commit fijado) a una carpeta vacia propia
#     y extrae el cuerpo de POST /repos/{owner}/{repo}/rulesets.
# (C) Modelo neutral de puertas -> traduccion a un JSON de ruleset por destino
#     (develop, main, ramas de tarea, taskcode/tareas, tags v*, y la variante
#     de develop con cola para repos de organizacion, D3), guardados como
#     artefacto, y validacion contra el esquema con un validador estricto
#     (claves desconocidas = fallo; enums de rules[].type,
#     allowed_merge_methods, integration_id entero). Controles negativos.
# (D) La tabla carril -> checks -> quien aprueba del ADR (leida del propio ADR)
#     coincide con los checks de los JSON, y ninguna fila aprueba con IA.
#
# Uso: poc-f1-5.sh <carpeta-de-trabajo> <ruta-del-ADR> <carpeta-artefactos-json>
# Ningun POST/PUT/PATCH/DELETE: todas las llamadas pasan por gh_get.
set -u
[ $# -eq 3 ] || { echo "uso: $0 <trabajo> <adr> <artefactos>"; exit 2; }
[ -f "$2" ] || { echo "ERROR: no existe el ADR $2"; exit 2; }
ADR=$(cd "$(dirname "$2")" && (pwd -W 2>/dev/null || pwd))/$(basename "$2")
mkdir -p "$3" || exit 2
ART=$(cd "$3" && (pwd -W 2>/dev/null || pwd))
mkdir -p "$1" || exit 2
base="$(cd "$1" && pwd)"
if [ -n "$(ls -A "$base")" ] && [ ! -f "$base/.poc-f1-5" ]; then
  echo "ERROR: $base no esta vacia ni es de una ejecucion anterior de la PoC"; exit 2
fi
rm -rf "$base"; mkdir -p "$base/esquema" "$base/rulesets"; : > "$base/.poc-f1-5"
cd "$base" || exit 2
RAIZ=$(pwd -W 2>/dev/null || pwd)          # ruta C:/... para python.exe
PY=$(command -v python || command -v python3)
REPOS="charliebk/taskcodetwo-poc-f1-1 charliebk/TaskCodeTwo"
SCHEMA_REPO=github/rest-api-description
APP_REVISOR_ID="${APP_REVISOR_ID:-0}"   # relleno: la App del revisor la decide F1-6
sep() { echo; echo "=== $* ==="; }
resultado() { echo "RESULTADO $*"; echo "RESULTADO $*" >> "$base/resultados.txt"; }

# Unica puerta a la API: siempre GET. Imprime "HTTP <codigo>" y el cuerpo recortado.
gh_get() {
  local ruta="$1" salida codigo cuerpo
  echo "\$ gh api --method GET $ruta" >> "$base/llamadas.txt"
  salida=$(gh api --method GET -i "$ruta" 2>/dev/null)
  codigo=$(printf '%s\n' "$salida" | head -1 | awk '{print $2}')
  cuerpo=$(printf '%s\n' "$salida" | awk 'f{print} /^\r?$/{f=1}')
  echo "HTTP $codigo $(printf '%s' "$cuerpo" | tr -d '\r\n' | cut -c1-200)"
}

######################################################################
sep "A. Estado real en GitHub (solo GET)"
gh --version | head -1
for r in $REPOS; do
  echo "--- $r"
  echo "visibilidad: $(gh repo view "$r" --json visibility -q .visibility 2>&1)"
  echo "\$ gh repo view $r --json visibility" >> "$base/llamadas.txt"
  for p in rulesets rules/branches/develop branches/develop/protection; do
    printf '%-32s ' "$p:"; gh_get "repos/$r/$p"
  done
done
printf '%-32s ' "TaskCodeTwo codeowners/errors:"; gh_get "repos/charliebk/TaskCodeTwo/codeowners/errors"
printf '%-32s ' "apps/github-actions:"; r=$(gh_get "apps/github-actions"); echo "${r:0:40}..."
ACTIONS_ID=$(gh api --method GET apps/github-actions -q .id)
echo "\$ gh api --method GET apps/github-actions -q .id" >> "$base/llamadas.txt"
echo "id de la App GitHub Actions: $ACTIONS_ID"

######################################################################
sep "B. Esquema OpenAPI de rulesets (descarga a carpeta vacia propia)"
SHA=$(gh api --method GET "repos/$SCHEMA_REPO/commits/main" -q '.sha + " " + .commit.committer.date')
echo "\$ gh api --method GET repos/$SCHEMA_REPO/commits/main" >> "$base/llamadas.txt"
echo "commit de $SCHEMA_REPO: $SHA"
URL="https://raw.githubusercontent.com/$SCHEMA_REPO/${SHA%% *}/descriptions/api.github.com/api.github.com.json"
curl -sSfL -o "$base/esquema/api.github.com.json" "$URL" || { echo "ERROR: descarga"; exit 1; }
echo "descargado: $(wc -c < "$base/esquema/api.github.com.json") bytes ; sha256 $(sha256sum "$base/esquema/api.github.com.json" | cut -c1-16)"

######################################################################
sep "C. Modelo neutral -> rulesets de GitHub -> validacion contra el esquema"
"$PY" -I - "$RAIZ/esquema/api.github.com.json" "$RAIZ/rulesets" "$ACTIONS_ID" "$APP_REVISOR_ID" <<'EOF' | tee "$base/c.txt"
import json, sys, os, copy
esq, salida, actions_id, revisor_id = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
doc = json.load(open(esq, encoding='utf-8'))
print('OpenAPI', doc['openapi'], 'version', doc['info']['version'])
cuerpo = doc['paths']['/repos/{owner}/{repo}/rulesets']['post']['requestBody']['content']['application/json']['schema']
comp = doc['components']['schemas']
def res(s):
    while '$ref' in s: s = comp[s['$ref'].split('/')[-1]]
    return s
tipos = [res(x)['properties']['type']['enum'][0] for x in res(comp['repository-rule'])['oneOf']]
print('rules[].type admitidos (%d):' % len(tipos), ' '.join(tipos))
pr = res(comp['repository-rule-pull-request'])['properties']['parameters']
print('allowed_merge_methods:', pr['properties']['allowed_merge_methods']['items']['enum'])
sc = res(comp['repository-rule-params-status-check-configuration'])
print('status check:', {k: v['type'] for k, v in sc['properties'].items()}, 'required', sc['required'])
print('additionalProperties en el cuerpo:', cuerpo.get('additionalProperties', '(no declarado)'))

# --- Validador estricto (subconjunto de JSON Schema que usa la descripcion) ---
def valida(v, s, ruta, errs):
    s = res(s)
    if 'allOf' in s:
        for x in s['allOf']: valida(v, x, ruta, errs)
    if 'oneOf' in s:
        ok = []
        for i, x in enumerate(s['oneOf']):
            e = []; valida(v, x, ruta, e)
            if not e: ok.append(i)
        if len(ok) != 1: errs.append('%s: oneOf casa con %d variantes' % (ruta, len(ok))); return
    if v is None and s.get('nullable'): return
    t = s.get('type')
    tipo_ok = {'object': isinstance(v, dict), 'array': isinstance(v, list), 'string': isinstance(v, str),
               'boolean': isinstance(v, bool), 'integer': isinstance(v, int) and not isinstance(v, bool),
               'number': isinstance(v, (int, float)) and not isinstance(v, bool)}
    if t and not tipo_ok.get(t, True): errs.append('%s: se esperaba %s, es %r' % (ruta, t, v)); return
    if 'enum' in s and v not in s['enum']: errs.append('%s: %r fuera del enum %s' % (ruta, v, s['enum']))
    if isinstance(v, (int, float)) and not isinstance(v, bool):
        if 'minimum' in s and v < s['minimum']: errs.append('%s: < minimum' % ruta)
        if 'maximum' in s and v > s['maximum']: errs.append('%s: > maximum' % ruta)
    if isinstance(v, dict):
        props = s.get('properties', {})
        for k in s.get('required', []):
            if k not in v: errs.append('%s: falta la clave obligatoria %s' % (ruta, k))
        for k, x in v.items():
            if k in props: valida(x, props[k], ruta + '.' + k, errs)
            elif props or s.get('additionalProperties') is False:
                errs.append('%s: clave desconocida %s' % (ruta, k))
    if isinstance(v, list) and 'items' in s:
        for i, x in enumerate(v): valida(x, s['items'], '%s[%d]' % (ruta, i), errs)

# --- Modelo neutral de puertas (lo que guarda el nucleo, sin GitHub) ---
APROBACION = dict(aprobaciones=1, descartar_viejas=True, ultimo_push=True, propietarios=True,
                  hilos_resueltos=False, metodos=['merge'])
ORIGEN = {'actions': actions_id, 'app-revisor': revisor_id}
modelo = [
  dict(nombre='develop', tipo='rama', patrones=['develop'], sin_borrado=True, sin_forzar=True,
       solo_pr=APROBACION, al_dia=True,
       checks=[('verificacion', 'actions'), ('revision-ia', 'app-revisor'), ('carril', 'actions')]),
  dict(nombre='main', tipo='rama', patrones=['main'], sin_borrado=True, sin_forzar=True,
       solo_pr=APROBACION, al_dia=True,
       checks=[('verificacion', 'actions'), ('carril', 'actions')]),
  dict(nombre='ramas-de-tarea', tipo='rama', patrones=['feature/**', 'fix/**', 'hotfix/**'],
       sin_borrado=True, sin_forzar=True),
  dict(nombre='taskcode-tareas', tipo='rama', patrones=['taskcode/tareas'], sin_borrado=True, sin_forzar=True),
  dict(nombre='tags-version', tipo='tag', patrones=['v*'], sin_borrado=True, sin_mover=True),
]
COLA = dict(check_response_timeout_minutes=60, grouping_strategy='ALLGREEN', max_entries_to_build=5,
            max_entries_to_merge=5, merge_method='MERGE', min_entries_to_merge=1,
            min_entries_to_merge_wait_minutes=5)

def a_github(p, cola=False):
    pref = 'refs/heads/' if p['tipo'] == 'rama' else 'refs/tags/'
    rs = {'name': 'taskcode-' + p['nombre'] + ('-cola' if cola else ''), 'target': 'branch' if p['tipo'] == 'rama' else 'tag',
          'enforcement': 'active', 'bypass_actors': [],
          'conditions': {'ref_name': {'include': [pref + x for x in p['patrones']], 'exclude': []}}, 'rules': []}
    if p.get('sin_borrado'): rs['rules'].append({'type': 'deletion'})
    if p.get('sin_forzar'): rs['rules'].append({'type': 'non_fast_forward'})
    if p.get('sin_mover'): rs['rules'].append({'type': 'update', 'parameters': {'update_allows_fetch_and_merge': False}})
    if p.get('solo_pr'):
        a = p['solo_pr']
        rs['rules'].append({'type': 'pull_request', 'parameters': {
            'allowed_merge_methods': a['metodos'], 'dismiss_stale_reviews_on_push': a['descartar_viejas'],
            'require_code_owner_review': a['propietarios'], 'require_last_push_approval': a['ultimo_push'],
            'required_approving_review_count': a['aprobaciones'],
            'required_review_thread_resolution': a['hilos_resueltos']}})
    if p.get('checks'):
        rs['rules'].append({'type': 'required_status_checks', 'parameters': {
            'strict_required_status_checks_policy': p['al_dia'],
            'required_status_checks': [{'context': c, 'integration_id': ORIGEN[o]} for c, o in p['checks']]}})
    if cola: rs['rules'].append({'type': 'merge_queue', 'parameters': COLA})
    return rs

rulesets = [(p['nombre'], a_github(p)) for p in modelo]
rulesets.append(('develop-cola-org', a_github(modelo[0], cola=True)))
total_err = 0
for nombre, rs in rulesets:
    e = []; valida(rs, cuerpo, nombre, e)
    with open(os.path.join(salida, nombre + '.json'), 'w', encoding='utf-8', newline='\n') as f:
        json.dump(rs, f, indent=2, ensure_ascii=False); f.write('\n')
    reglas = ','.join(r['type'] for r in rs['rules'])
    checks = [c['context'] + '@' + str(c['integration_id']) for r in rs['rules'] if r['type'] == 'required_status_checks'
              for c in r['parameters']['required_status_checks']]
    print('%-18s %-6s %-48s reglas=%s checks=%s -> %s' % (nombre, rs['target'], ','.join(rs['conditions']['ref_name']['include']),
          reglas, ','.join(checks) or '-', 'VALIDO' if not e else 'INVALIDO: ' + '; '.join(e)))
    total_err += len(e)

print('-- controles negativos (el validador tiene que rechazarlos)')
base_rs = rulesets[0][1]
def muta(f):
    x = copy.deepcopy(base_rs); f(x); return x
negativos = {
  'rules[].type inventado (required_approvals)': muta(lambda x: x['rules'].append({'type': 'required_approvals'})),
  'allowed_merge_methods fast-forward': muta(lambda x: x['rules'][2]['parameters'].__setitem__('allowed_merge_methods', ['fast-forward'])),
  'integration_id como texto': muta(lambda x: x['rules'][3]['parameters']['required_status_checks'][0].__setitem__('integration_id', '15368')),
  'clave desconocida en pull_request (required_reviewers_count)': muta(lambda x: x['rules'][2]['parameters'].__setitem__('required_reviewers_count', 2)),
  'clave desconocida en la raiz (lane)': muta(lambda x: x.__setitem__('lane', 'riesgo-alto')),
  'falta strict_required_status_checks_policy': muta(lambda x: x['rules'][3]['parameters'].pop('strict_required_status_checks_policy')),
  'enforcement inventado (required)': muta(lambda x: x.__setitem__('enforcement', 'required')),
}
rech = 0
for k, v in negativos.items():
    e = []; valida(v, cuerpo, 'neg', e)
    rech += bool(e)
    print('  %-60s -> %s' % (k, 'rechazado: ' + e[0] if e else 'ACEPTADO (fallo del validador)'))
print('RESULTADO C rulesets=%d errores=%d ; negativos rechazados=%d/%d ; integration_id actions=%d revisor=%d (relleno)'
      % (len(rulesets), total_err, rech, len(negativos), actions_id, revisor_id))
EOF
grep '^RESULTADO' "$base/c.txt" >> "$base/resultados.txt"
cp "$base/rulesets/"*.json "$ART/" && echo "artefactos copiados: $(ls "$ART" | tr '\n' ' ')"

######################################################################
sep "D. Tabla carril -> checks -> quien aprueba del ADR frente a los JSON"
"$PY" -I - "$ADR" "$RAIZ/rulesets" <<'EOF' | tee "$base/d.txt"
import json, sys, os, re
adr, rdir = sys.argv[1], sys.argv[2]
txt = open(adr, encoding='utf-8').read()
def checks_de(nombre):
    rs = json.load(open(os.path.join(rdir, nombre + '.json'), encoding='utf-8'))
    return {c['context'] for r in rs['rules'] if r['type'] == 'required_status_checks'
            for c in r['parameters']['required_status_checks']}
def bypass(nombre):
    return json.load(open(os.path.join(rdir, nombre + '.json'), encoding='utf-8'))['bypass_actors']
# Lo que el check `carril` impone ademas del ruleset de la base (decision del ADR)
EXTRA_CARRIL = {'hotfix': {'revision-ia'}}
IA = re.compile(r'\b(IA|agente|revisor|bot|app|copilot|claude)\b', re.I)
def comprobar(texto, ver):
    m = re.search(r'<!-- tabla-carriles:inicio -->(.*?)<!-- tabla-carriles:fin -->', texto, re.S)
    if not m: print('ERROR: no hay tabla de carriles'); return 0, 1
    filas = [l for l in m.group(1).strip().splitlines() if l.startswith('|')][2:]
    fallos = 0
    for l in filas:
        c = [x.strip() for x in l.strip().strip('|').split('|')]
        carril, base, checks, aprueba = c[0], c[1].strip('`'), c[2], c[3]
        tabla = set(re.findall(r'`([a-z-]+)`', checks))
        esperado = checks_de(base) | EXTRA_CARRIL.get(carril, set())
        ok_checks = tabla == esperado and 'carril' in checks_de(base)
        ok_aprueba = bool(re.search(r'persona', aprueba)) and not IA.search(aprueba)
        ok_bypass = bypass(base) == []
        fallos += (not ok_checks) + (not ok_aprueba) + (not ok_bypass)
        if ver:
            print('%-14s base=%-8s tabla=%-36s ruleset+carril=%-36s checks:%s aprueba:%s (%s) bypass_actors=%s'
                  % (carril, base, ','.join(sorted(tabla)), ','.join(sorted(esperado)), 'si' if ok_checks else 'NO',
                     'persona, sin IA' if ok_aprueba else 'NO', aprueba, bypass(base)))
    return len(filas), fallos
nfil, fallos = comprobar(txt, True)
print('-- controles negativos (tablas alteradas que la comprobacion tiene que detectar)')
neg = {
  'release con revision-ia': txt.replace('| release | `main` | `verificacion` (perfil completo), `carril`', '| release | `main` | `verificacion`, `revision-ia`, `carril`'),
  'fix sin carril': txt.replace('| fix | `develop` | `verificacion`, `revision-ia`, `carril`', '| fix | `develop` | `verificacion`, `revision-ia`'),
  'errata aprobada por el revisor IA': txt.replace('| errata | `develop` | `verificacion`, `revision-ia` (perfil rápido, D5a), `carril` | 1 persona en el PR', '| errata | `develop` | `verificacion`, `revision-ia` (perfil rápido, D5a), `carril` | el revisor IA'),
}
detect = 0
for k, t in neg.items():
    cambiado = t != txt
    _, f = comprobar(t, False)
    detect += (cambiado and f > 0)
    print('  %-36s -> %s' % (k, ('detectado (%d fallos)' % f) if cambiado and f else ('NO detectado' if cambiado else 'NO APLICADO (texto no encontrado)')))
print('RESULTADO D filas=%d fallos=%d ; negativos detectados=%d/%d' % (nfil, fallos, detect, len(neg)))
EOF

grep '^RESULTADO' "$base/d.txt" >> "$base/resultados.txt"

metodos=$(grep -c -- 'gh api --method GET' "$base/llamadas.txt")
otros=$(grep 'gh api' "$base/llamadas.txt" | grep -vc -- 'gh api --method GET')
resultado "A+B llamadas gh api con GET=$metodos ; gh api con otro metodo=$otros ; gh repo view=$(grep -c 'repo view' "$base/llamadas.txt")"

sep "Resumen"
cat "$base/resultados.txt" 2>/dev/null
echo "llamadas a GitHub registradas:"; sed 's/^/  /' "$base/llamadas.txt"

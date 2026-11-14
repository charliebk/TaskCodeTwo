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
#     coincide con los checks y el numero de aprobaciones de los JSON y del
#     check carril, y ninguna fila aprueba con IA.
# (E) Por que main va sin strict: ancestria tras una release (git local).
# (F) Carril no falsificable: campos de firma del esquema, firma real de un
#     commit (GET) y simulacion de la regla del check carril.
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
"$PY" -I - "$RAIZ/esquema/api.github.com.json" "$RAIZ/rulesets" "$ACTIONS_ID" "$APP_REVISOR_ID" "$ART" <<'EOF' | tee "$base/c.txt"
import json, sys, os, copy
esq, salida, actions_id, revisor_id, art = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4]), sys.argv[5]
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
  # main sin rama al dia (strict=False): con releases por merge commit, main deja de ser
  # ancestro de develop y el PR develop->main saldria siempre desactualizado (seccion E).
  dict(nombre='main', tipo='rama', patrones=['main'], sin_borrado=True, sin_forzar=True,
       solo_pr=APROBACION, al_dia=False,
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
print('-- JSON versionados en la carpeta de artefactos (antes de sobrescribirlos): validacion y comparacion')
ver_ok = ver_igual = 0
for nombre, rs in rulesets:
    ruta = os.path.join(art, nombre + '.json')
    if not os.path.exists(ruta): print('  %-18s FALTA' % nombre); continue
    try: v = json.load(open(ruta, encoding='utf-8'))
    except ValueError as ex: print('  %-18s JSON ilegible: %s' % (nombre, ex)); continue
    e = []; valida(v, cuerpo, nombre, e)
    ver_ok += not e; ver_igual += (v == rs)
    print('  %-18s %s ; %s' % (nombre, 'VALIDO' if not e else 'INVALIDO: ' + e[0],
          'igual al generado' if v == rs else 'DISTINTO del generado (se sobrescribe)'))
print('RESULTADO C rulesets=%d errores=%d ; negativos rechazados=%d/%d ; integration_id actions=%d revisor=%d (relleno) ; versionados validos=%d/%d iguales=%d/%d'
      % (len(rulesets), total_err, rech, len(negativos), actions_id, revisor_id, ver_ok, len(rulesets), ver_igual, len(rulesets)))
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
def aprobaciones_pr(nombre):
    rs = json.load(open(os.path.join(rdir, nombre + '.json'), encoding='utf-8'))
    return sum(r['parameters']['required_approving_review_count'] for r in rs['rules'] if r['type'] == 'pull_request')
# Aprobacion del plan (commit firmado de approve) que exige el check carril, y minimo total (D4a)
PLAN_CARRIL = {'funcionalidad': 1, 'riesgo alto': 1}
MIN_TOTAL = {'riesgo alto': 2}
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
        num = re.search(r'\d+', aprueba)
        total = aprobaciones_pr(base) + PLAN_CARRIL.get(carril, 0)
        ok_num = (num is not None and int(num.group()) == total and total >= MIN_TOTAL.get(carril, 1)
                  and (('plan' in aprueba) == bool(PLAN_CARRIL.get(carril))))
        ok_aprueba = bool(re.search(r'persona', aprueba)) and not IA.search(aprueba) and ok_num
        ok_bypass = bypass(base) == []
        fallos += (not ok_checks) + (not ok_aprueba) + (not ok_bypass)
        if ver:
            print('%-14s base=%-8s tabla=%-36s ruleset+carril=%-36s checks:%s aprobaciones=%d (PR %d + plan %d) aprueba:%s bypass_actors=%s'
                  % (carril, base, ','.join(sorted(tabla)), ','.join(sorted(esperado)), 'si' if ok_checks else 'NO',
                     total, aprobaciones_pr(base), PLAN_CARRIL.get(carril, 0),
                     'persona, sin IA, numero cuadra' if ok_aprueba else 'NO', bypass(base)))
    return len(filas), fallos
nfil, fallos = comprobar(txt, True)
print('-- controles negativos (tablas alteradas que la comprobacion tiene que detectar)')
neg = {
  'release con revision-ia': txt.replace('| release | `main` | `verificacion` (perfil completo), `carril`', '| release | `main` | `verificacion`, `revision-ia`, `carril`'),
  'fix sin carril': txt.replace('| fix | `develop` | `verificacion`, `revision-ia`, `carril`', '| fix | `develop` | `verificacion`, `revision-ia`'),
  'riesgo alto con 1 persona': re.sub(r'(\| riesgo alto \|[^|]*\|[^|]*\|)[^|]*\|', r'\1 1 persona en el PR |', txt),
  'funcionalidad sin aprobacion del plan': re.sub(r'(\| funcionalidad \|[^|]*\|[^|]*\|)[^|]*\|', r'\1 1 persona en el PR |', txt),
  'errata aprobada por el revisor IA': txt.replace('| errata | `develop` | `verificacion`, `revision-ia` (perfil rápido, D5a), `carril` | 1 persona en el PR', '| errata | `develop` | `verificacion`, `revision-ia` (perfil rápido, D5a), `carril` | el revisor IA'),
}
detect = 0
for k, t in neg.items():
    cambiado = t != txt
    _, f = comprobar(t, False)
    detect += (cambiado and f > 0)
    print('  %-40s -> %s' % (k, ('detectado (%d fallos)' % f) if cambiado and f else ('NO detectado' if cambiado else 'NO APLICADO (texto no encontrado)')))
print('RESULTADO D filas=%d fallos=%d ; negativos detectados=%d/%d' % (nfil, fallos, detect, len(neg)))
EOF

grep '^RESULTADO' "$base/d.txt" >> "$base/resultados.txt"

######################################################################
sep "E. Por que main va sin strict: ancestria tras una release por merge commit (git local)"
g() { git -c core.autocrlf=false -c init.defaultBranch=develop -c advice.detachedHead=false "$@"; }
export GIT_AUTHOR_NAME=poc GIT_AUTHOR_EMAIL=poc@local GIT_COMMITTER_NAME=poc GIT_COMMITTER_EMAIL=poc@local
export GIT_AUTHOR_DATE="2026-11-14T10:00:00Z" GIT_COMMITTER_DATE="2026-11-14T10:00:00Z"
mkdir -p "$base/e" && cd "$base/e" && g init -q . && echo 0 > a && g add a && g commit -qm inicial && g branch main
anc() { if g merge-base --is-ancestor main develop; then echo "main ES ancestro de develop"; else echo "main NO es ancestro de develop"; fi; }
echo "develop y main iguales: $(anc)"
echo 1 > a && g commit -qam d1
g switch -q main && g merge -q --no-ff -m "Merge pull request #1 from develop (release v0.1.0)" develop && g switch -q develop
echo "tras la release 1 (merge commit en main): $(anc)"
echo 2 > a && g commit -qam d2
echo "tras d2 en develop: $(anc) -> con strict, el PR develop->main de la release 2 saldria desactualizado"
cd "$base" || exit 2
resultado "E main ancestro de develop tras release por merge commit: $(g -C "$base/e" merge-base --is-ancestor main develop && echo si || echo no)"

######################################################################
sep "F. Carril no falsificable por el agente: firma verificada del approve y carril minimo por ficheros"
printf '%-40s ' "commit de develop en TaskCodeTwo:"
gh api --method GET repos/charliebk/TaskCodeTwo/commits/develop \
  -q '"sha=\(.sha[0:12]) verified=\(.commit.verification.verified) reason=\(.commit.verification.reason) committer=\(.committer.login // "null") commit.committer=\(.commit.committer.name)"'
echo "\$ gh api --method GET repos/charliebk/TaskCodeTwo/commits/develop" >> "$base/llamadas.txt"
"$PY" -I - "$RAIZ/esquema/api.github.com.json" <<'EOF' | tee "$base/f.txt"
import json, sys
doc = json.load(open(sys.argv[1], encoding='utf-8'))
comp = doc['components']['schemas']
def res(s):
    while '$ref' in s: s = comp[s['$ref'].split('/')[-1]]
    return s
r = doc['paths']['/repos/{owner}/{repo}/commits/{ref}']['get']['responses']['200']['content']['application/json']['schema']
c = res(r)
ver = res(res(c['properties']['commit'])['properties']['verification'])
print('GET /repos/{o}/{r}/commits/{ref}: commit.verification =', sorted(ver['properties']), 'required', ver.get('required'))
print('  campos de primer nivel con la cuenta de GitHub:', [k for k in ('author', 'committer') if k in c['properties']])
# --- Regla 1 de la decision 4 (simulada): aprobacion del plan atada a firma y a contenido ---
import hashlib
def blob(texto):                  # sha de blob de git: sha1("blob <n>\0" + contenido)
    b_ = texto.encode('utf-8'); return hashlib.sha1(b'blob %d\0' % len(b_) + b_).hexdigest()
PLAN_V1 = '# Plan\n\nEnfoque aprobado por la persona.\n'
PLAN_V2 = '# Plan\n\nEnfoque cambiado por el agente despues del approve.\n'
print('  blob de plan-final.md: v1=%s v2=%s' % (blob(PLAN_V1)[:12], blob(PLAN_V2)[:12]))
PERSONAS = {'charliebk'}          # identidades de persona del proyecto (F1-6)
def plan_aprobado(approve, plan_actual):
    # el commit firmado de approve registra el blob aprobado (p. ej. trailer "Plan-Blob: <sha>")
    return (bool(approve['verified']) and approve['committer'] in PERSONAS
            and approve.get('plan_blob') is not None and approve['plan_blob'] == blob(plan_actual))
casos = [
  ('firmado, persona, blob = plan actual',         dict(verified=True, committer='charliebk', plan_blob=blob(PLAN_V1)), PLAN_V1, True),
  ('sin firma (lo escribe el agente)',             dict(verified=False, committer='charliebk', plan_blob=blob(PLAN_V1)), PLAN_V1, False),
  ('verificado pero de la App del agente',         dict(verified=True, committer='taskcode-agente[bot]', plan_blob=blob(PLAN_V1)), PLAN_V1, False),
  ('verificado por GitHub (web-flow, por API)',    dict(verified=True, committer='web-flow', plan_blob=blob(PLAN_V1)), PLAN_V1, False),
  ('firmado, pero el agente cambio el plan despues', dict(verified=True, committer='charliebk', plan_blob=blob(PLAN_V1)), PLAN_V2, False),
  ('firmado, sin blob registrado',                 dict(verified=True, committer='charliebk'), PLAN_V1, False),
  ('firmado de nuevo sobre el plan cambiado',      dict(verified=True, committer='charliebk', plan_blob=blob(PLAN_V2)), PLAN_V2, True),
]
ok_plan = 0
for nombre, cm, actual, esperado in casos:
    r_ = plan_aprobado(cm, actual); ok_plan += (r_ == esperado)
    print('  plan: %-48s -> %s %s' % (nombre, 'cuenta' if r_ else 'no cuenta', '' if r_ == esperado else '(INESPERADO)'))
n_plan = len(casos)
# --- Regla 2: carril efectivo = max(declarado, minimo por ficheros). Derivacion: EJEMPLO, la fija F1-10 ---
# Errata solo si TODOS los ficheros estan bajo docs/. Todo PR necesita tarea (tambien la errata).
ORDEN = ['errata', 'fix', 'funcionalidad', 'riesgo alto']
def minimo(ficheros):
    if any(f.startswith(('.github/', '.taskcode/')) or f.split('/')[-1] == 'CODEOWNERS' for f in ficheros): return 'riesgo alto'
    if all(f.startswith('docs/') for f in ficheros): return 'errata'
    return 'fix'
def efectivo(declarado, ficheros):
    if declarado is None:                       # PR sin tarea: siempre falla cerrado
        return 'FALLA (PR sin tarea)'
    return max(declarado, minimo(ficheros), key=ORDEN.index)
casos = [
  ('errata', ['docs/a.md'], 'errata'),
  ('errata', ['src/x.md'], 'fix'),
  ('errata', ['CLAUDE.md'], 'fix'),
  ('errata', ['src/main.rs'], 'fix'),
  ('fix', ['.github/workflows/carril.yml'], 'riesgo alto'),
  ('riesgo alto', ['docs/guia.md'], 'riesgo alto'),
  (None, ['docs/a.md'], 'FALLA (PR sin tarea)'),
  (None, ['src/main.rs'], 'FALLA (PR sin tarea)'),
]
ok_car = 0
for d, fs, esperado in casos:
    r_ = efectivo(d, fs); ok_car += (r_ == esperado)
    print('  carril: declarado=%-12s ficheros=%-32s -> %s %s' % (d, ','.join(fs), r_, '' if r_ == esperado else '(INESPERADO)'))
print('RESULTADO F esquema verification=%s ; plan %d/%d casos ; carril efectivo %d/%d casos (el agente solo puede subirlo)'
      % ('si' if 'verified' in ver['properties'] else 'NO', ok_plan, n_plan, ok_car, len(casos)))
EOF
grep '^RESULTADO' "$base/f.txt" >> "$base/resultados.txt"

metodos=$(grep -c -- 'gh api --method GET' "$base/llamadas.txt")
otros=$(grep 'gh api' "$base/llamadas.txt" | grep -vc -- 'gh api --method GET')
resultado "A+B llamadas gh api con GET=$metodos ; gh api con otro metodo=$otros ; gh repo view=$(grep -c 'repo view' "$base/llamadas.txt")"

sep "Resumen"
cat "$base/resultados.txt" 2>/dev/null
echo "llamadas a GitHub registradas:"; sed 's/^/  /' "$base/llamadas.txt"

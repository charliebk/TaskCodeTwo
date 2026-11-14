#!/usr/bin/env bash
# PoC de F1-6 (TASK-007): identidades del agente implementador, del revisor IA
# y de la persona. SOLO LECTURA en GitHub.
#
# Las tablas que se comprueban se extraen del propio ADR, entre marcadores:
#   <!-- tabla-identidades:... -->  decision 1, operacion -> identidad
#   <!-- tabla-permisos:... -->     decision 2, permisos de cada identidad
#
# (A) Permisos de cada App (del ADR) validados contra el esquema
#     `app-permissions` de la descripcion OpenAPI publica
#     (github/rest-api-description, commit fijado, descargada a una carpeta
#     vacia propia). El agente lee checks y statuses y no los escribe.
#     Controles negativos.
# (B) Con la tabla publica de permisos por endpoint de las GitHub Apps
#     (github/docs, src/github-apps/data/<version>/server-to-server-permissions.json,
#     fuente de la pagina «Permissions required for GitHub Apps"; la
#     descripcion OpenAPI no trae permisos por endpoint, se comprueba):
#     - cada operacion de la decision 1 asignada a una App la alcanza con los
#       permisos de la decision 2; las de solo persona por administration no
#       las alcanza ninguna App; invariantes "ninguna IA aprueba ni mergea";
#       cada permiso se usa (minimo);
#     - enumeracion completa de las escrituras que alcanza cada App;
#     - copias mutadas del ADR en la carpeta temporal: tienen que fallar.
# (C) GET de actions/permissions/workflow en los dos repos (con el token de
#     la persona) y type e id de /users/{login}.
# (D) Simulacion del contrato del firmante (approve) y de la regla de
#     reviews de carril, con los type e id reales de C; campos del esquema.
# (E) Firma SSH local con claves desechables: commit de approve con trailer
#     Plan-Blob, git verify-commit con allowed_signers bueno y malo, clave con
#     frase de paso sin agente ni TTY, y soporte de claves FIDO (-sk).
#
# Uso: poc-f1-6.sh <carpeta-de-trabajo> <ruta-del-ADR>
# Sale con 0 solo si todas las secciones salen como se espera (A..E, sin
# llamadas de otro metodo y todas las mutaciones aplicadas y detectadas).
# Ningun POST/PUT/PATCH/DELETE: todas las llamadas a la API pasan por gh_get.
set -u
[ $# -eq 2 ] || { echo "uso: $0 <trabajo> <adr>"; exit 2; }
[ -f "$2" ] || { echo "ERROR: no existe el ADR $2"; exit 2; }
ADR=$(cd "$(dirname "$2")" && (pwd -W 2>/dev/null || pwd))/$(basename "$2")
mkdir -p "$1" || exit 2
base="$(cd "$1" && pwd)"
if [ -n "$(ls -A "$base")" ] && [ ! -f "$base/.poc-f1-6" ]; then
  echo "ERROR: $base no esta vacia ni es de una ejecucion anterior de la PoC"; exit 2
fi
rm -rf "$base"; mkdir -p "$base/esquema" "$base/docs" "$base/claves" "$base/e" "$base/mutaciones" "$base/py"
: > "$base/.poc-f1-6"; : > "$base/llamadas.txt"; : > "$base/resultados.txt"
cd "$base" || exit 2
RAIZ=$(pwd -W 2>/dev/null || pwd)          # ruta C:/... para python.exe
PY=$(command -v python || command -v python3)
REPOS="charliebk/TaskCodeTwo charliebk/taskcodetwo-poc-f1-1"
USUARIOS="charliebk web-flow github-actions[bot]"
SCHEMA_REPO=github/rest-api-description
DOCS_REPO=github/docs
sep() { echo; echo "=== $* ==="; }
FALLOS=""
falla() { FALLOS="$FALLOS $1"; echo "FALLO en $1"; }

# Unica puerta a la API: siempre GET. Imprime "HTTP <codigo> <cuerpo recortado>".
gh_get() {
  local ruta="$1" salida codigo cuerpo
  echo "\$ gh api --method GET $ruta" >> "$base/llamadas.txt"
  salida=$(gh api --method GET -i "$ruta" 2>/dev/null)
  codigo=$(printf '%s\n' "$salida" | head -1 | awk '{print $2}')
  cuerpo=$(printf '%s\n' "$salida" | awk 'f{print} /^\r?$/{f=1}')
  echo "HTTP $codigo $(printf '%s' "$cuerpo" | tr -d '\r\n' | cut -c1-200)"
}
# GET con filtro jq (solo GET; se registra igual).
gh_get_q() {
  echo "\$ gh api --method GET $1 -q ..." >> "$base/llamadas.txt"
  gh api --method GET "$1" -q "$2" 2>/dev/null
}
descarga() {   # descarga <url> <fichero>: GET de un fichero publico
  echo "descarga GET $1" >> "$base/llamadas.txt"
  curl -sSfL -o "$2" "$1" || { echo "ERROR: descarga $1"; exit 1; }
  echo "  $(basename "$2"): $(wc -c < "$2") bytes ; sha256 $(sha256sum "$2" | cut -c1-16)"
}

######################################################################
# Modulo comun (escrito por el guion en su propia carpeta py/): extrae las
# tablas del ADR y calcula alcances sobre la tabla de permisos por endpoint.
cat > "$base/py/tablas.py" <<'EOF'
import re, json
NIVEL = {'read': 1, 'write': 2, 'admin': 3}
R = '/repos/{owner}/{repo}'
def bloque(txt, marca):
    m = re.search(r'<!-- %s:inicio -->(.*?)<!-- %s:fin -->' % (marca, marca), txt, re.S)
    if not m: raise ValueError('falta la tabla %s en el ADR' % marca)
    filas = [l.strip() for l in m.group(1).splitlines() if l.strip().startswith('|')][2:]
    return [[c.strip() for c in f.strip('|').split('|')] for f in filas]
def permisos(txt):
    """decision 2 -> {app: {permiso: nivel}} (la persona no es una App)."""
    ids = {}
    for c in bloque(txt, 'tabla-permisos'):
        ident = re.search(r'`(agente|revisor|persona)`', c[0]).group(1)
        if ident != 'persona':
            ids[ident] = dict(re.findall(r'`([a-z_]+):([a-z]+)`', c[2]))
    return ids
def operaciones(txt):
    """decision 1 -> [(operacion, [identidades])]: cada `...` de la columna Operacion
    antes del primer parentesis (el parentesis es explicacion) es una operacion."""
    ops = []
    for c in bloque(txt, 'tabla-identidades'):
        nombres = re.findall(r'`([^`]+)`', c[0].split('(')[0])
        if not nombres: raise ValueError('fila sin operacion entre comillas: %s' % c[0])
        idents = re.findall(r'\b(agente|revisor|persona)\b', c[2])
        ops += [(n, idents) for n in nombres]
    return ops
def indice(fich):
    d = json.load(open(fich, encoding='utf-8'))
    idx = {}
    for perm, v in d.items():
        for e in v['permissions']:
            if e.get('server-to-server'):
                idx.setdefault((e['verb'].upper(), e['requestPath']), []).append((perm, e['access']))
    return idx
def alcanza(perms, idx, verbo, ruta):
    """None si el endpoint no esta en la tabla; si no, si algun permiso basta.
    workflows se trata aparte: solo cuenta para ficheros de .github/workflows/."""
    ent = idx.get((verbo, ruta))
    if ent is None: return None
    return any(NIVEL.get(perms.get(p, ''), 0) >= NIVEL[a] for p, a in ent if p != 'workflows')
# Endpoints de cada operacion del puerto (equivalencia REST de las de Git).
EP_OP = {
  'fetch':                 [('GET', R + '/git/matching-refs/{ref}'), ('GET', R + '/commits/{ref}')],
  'comprobar-id':          [('GET', R + '/git/matching-refs/{ref}')],
  'actualizar-principal':  [('GET', R + '/git/matching-refs/{ref}'), ('GET', R + '/commits/{ref}')],
  'push':                  [('POST', R + '/git/refs'), ('PATCH', R + '/git/refs/{ref}')],
  'actualizar-rama':       [('POST', R + '/git/refs'), ('PATCH', R + '/git/refs/{ref}')],
  'commit-estado':         [('PATCH', R + '/git/refs/{ref}')],
  'approve':               [('PATCH', R + '/git/refs/{ref}')],
  'abrir-pr':              [('GET', R + '/pulls'), ('POST', R + '/pulls')],
  'estado-puertas':        [('GET', R + '/pulls/{pull_number}'), ('GET', R + '/pulls/{pull_number}/reviews'),
                            ('GET', R + '/commits/{ref}/check-runs'), ('GET', R + '/commits/{ref}/status')],
  'merge-pr':              [('PUT', R + '/pulls/{pull_number}/merge')],
  'tag-anotado':           [('POST', R + '/git/refs')],
  'aplicar-puertas':       [('POST', R + '/rulesets'), ('PUT', R + '/rulesets/{ruleset_id}')],
  'leer-puertas':          [('GET', R + '/rulesets'), ('GET', R + '/rules/branches/{branch}')],
  'leer-permisos-actions': [('GET', R + '/actions/permissions/workflow')],
  'revision-ia':           [('POST', R + '/check-runs'), ('PATCH', R + '/check-runs/{check_run_id}'),
                            ('GET', R + '/pulls/{pull_number}/files'), ('GET', R + '/contents/{path}')],
}
# Las que el CLI corre con el agente (lectura del remoto, pushes por require_last_push_approval, PR para que la persona apruebe)
CON_AGENTE = {'fetch', 'comprobar-id', 'actualizar-principal', 'push', 'actualizar-rama', 'commit-estado', 'abrir-pr'}
SOLO_PERSONA = {'approve', 'merge-pr', 'tag-anotado', 'aplicar-puertas', 'leer-permisos-actions'}
FUERA_DE_APPS = {'aplicar-puertas', 'leer-permisos-actions'}   # ninguna App las alcanza (administration)
LIMITE = {'merge-pr', 'tag-anotado'}                           # el agente las alcanza: limite escrito (D3a)
def invariantes(ids, idx):
    f = []
    rv, ag = ids.get('revisor', {}), ids.get('agente', {})
    if alcanza(rv, idx, 'POST', R + '/pulls/{pull_number}/reviews'): f.append('el revisor puede crear reviews (APPROVE)')
    if alcanza(rv, idx, 'PUT', R + '/pulls/{pull_number}/merge'): f.append('el revisor puede mergear')
    if alcanza(rv, idx, 'POST', R + '/git/refs'): f.append('el revisor puede empujar')
    for who, p in (('agente', ag), ('revisor', rv)):
        if alcanza(p, idx, 'POST', R + '/rulesets'): f.append('%s administra rulesets' % who)
        if p.get('workflows'): f.append('%s tiene workflows' % who)
        if alcanza(p, idx, 'GET', R + '/actions/permissions/workflow'): f.append('%s lee administration' % who)
    if alcanza(ag, idx, 'POST', R + '/check-runs'): f.append('el agente puede publicar checks (revision-ia)')
    if alcanza(ag, idx, 'POST', R + '/statuses/{sha}'): f.append('el agente puede publicar statuses')
    return f
def evalua(txt, idx, ver=False):
    """Comprueba las dos tablas del ADR. Devuelve la lista de fallos."""
    f = []
    try: ids, ops = permisos(txt), operaciones(txt)
    except (ValueError, AttributeError) as ex: return ['tablas ilegibles: %s' % ex]
    nombres = [o for o, _ in ops]
    for o in sorted(set(EP_OP) - set(nombres)): f.append('operacion %s sin fila en la decision 1' % o)
    usados = {k: set() for k in ids}
    for op, idents in ops:
        if op not in EP_OP: f.append('operacion %s sin endpoints conocidos' % op); continue
        if not idents: f.append('%s sin identidad' % op)
        if op in CON_AGENTE and 'agente' not in idents: f.append('%s sin la identidad del agente' % op)
        if op in SOLO_PERSONA and idents != ['persona']: f.append('%s asignada a %s (solo persona)' % (op, ','.join(idents)))
        for app in ('agente', 'revisor'):
            res = [alcanza(ids.get(app, {}), idx, v, r) for v, r in EP_OP[op]]
            if app in idents:
                if not all(res): f.append('%s no alcanza %s' % (app, op))
                for v, r in EP_OP[op]:
                    for p, a in idx.get((v, r), []):
                        if NIVEL.get(ids.get(app, {}).get(p, ''), 0) >= NIVEL[a]: usados[app].add(p)
            if op in FUERA_DE_APPS and any(res): f.append('%s alcanza %s (exige administration)' % (app, op))
        if ver:
            fl = lambda app: 'si' if all(alcanza(ids.get(app, {}), idx, v, r) for v, r in EP_OP[op]) else 'no'
            exige = ' + '.join('/'.join(sorted({'%s:%s' % pa for pa in idx.get((v, r), [])})) or 'NO LISTADO' for v, r in EP_OP[op])
            print('  %-22s identidad=%-16s agente=%-2s revisor=%-2s %s exige=%s' % (
                op, ','.join(idents), fl('agente'), fl('revisor'), '(limite) ' if op in LIMITE and fl('agente') == 'si' else '', exige))
    for app, perms in ids.items():
        sin_uso = sorted(set(perms) - usados.get(app, set()) - {'metadata'})
        if sin_uso: f.append('%s: permisos sin uso %s' % (app, ','.join(sin_uso)))
        if ver: print('  minimo %-8s usados=%s ; sin uso=%s' % (app, ','.join(sorted(usados.get(app, set()))), ','.join(sin_uso) or 'ninguno'))
    f += invariantes(ids, idx)
    return f
EOF

######################################################################
sep "Descargas publicas (cada una en su carpeta vacia)"
gh --version | head -1
echo "ADR leido: $(basename "$ADR")"
SHA=$(gh_get_q "repos/$SCHEMA_REPO/commits/main" '.sha + " " + .commit.committer.date')
echo "commit de $SCHEMA_REPO: $SHA"
descarga "https://raw.githubusercontent.com/$SCHEMA_REPO/${SHA%% *}/descriptions/api.github.com/api.github.com.json" "$base/esquema/api.github.com.json"
DSHA=$(gh_get_q "repos/$DOCS_REPO/commits/main" '.sha + " " + .commit.committer.date')
echo "commit de $DOCS_REPO: $DSHA"
VERSIONES=$(gh_get_q "repos/$DOCS_REPO/contents/src/github-apps/data?ref=${DSHA%% *}" '[.[].name | select(startswith("fpt-"))] | join(" ")')
echo "versiones fpt de la tabla de permisos: $VERSIONES"
for v in $VERSIONES; do
  descarga "https://raw.githubusercontent.com/$DOCS_REPO/${DSHA%% *}/src/github-apps/data/$v/server-to-server-permissions.json" "$base/docs/$v.json"
done

######################################################################
sep "A. Permisos de cada App (tabla de la decision 2 del ADR) contra app-permissions"
"$PY" -I - "$RAIZ/py" "$RAIZ/esquema/api.github.com.json" "$ADR" <<'EOF' | tee "$base/a.txt"
import json, sys
sys.path.insert(0, sys.argv[1]); import tablas
doc = json.load(open(sys.argv[2], encoding='utf-8'))
ids = tablas.permisos(open(sys.argv[3], encoding='utf-8').read())
print('OpenAPI', doc['openapi'], 'version', doc['info']['version'])
ap = doc['components']['schemas']['app-permissions']
props = ap['properties']
print('app-permissions: %d permisos; additionalProperties=%s' % (len(props), ap.get('additionalProperties', '(no declarado)')))
for p in ('contents', 'pull_requests', 'metadata', 'checks', 'statuses', 'administration', 'workflows'):
    print('  %-15s enum=%s' % (p, props[p]['enum']))
def valida(perms):
    e = []
    for k, v in perms.items():
        if k not in props: e.append('permiso desconocido %s' % k)
        elif v not in props[k]['enum']: e.append('%s=%r fuera del enum %s' % (k, v, props[k]['enum']))
    return e
err = 0
for nombre, perms in ids.items():
    e = valida(perms); err += len(e)
    print('%-8s %-78s -> %s' % (nombre, ', '.join('%s:%s' % kv for kv in perms.items()), 'VALIDO' if not e else 'INVALIDO: ' + '; '.join(e)))
lect = ids['agente'].get('checks') == 'read' and ids['agente'].get('statuses') == 'read'
print('agente lee checks y statuses sin escribirlos (checks:read, statuses:read): %s' % ('si' if lect else 'NO'))
print('-- controles negativos (tienen que salir rechazados)')
neg = {
  'permiso inventado approvals:write': dict(ids['revisor'], approvals='write'),
  'permiso inventado merge:write': dict(ids['agente'], merge='write'),
  'valor inventado contents:admin': dict(ids['agente'], contents='admin'),
  'workflows:read (el enum solo admite write)': dict(ids['agente'], workflows='read'),
}
rech = 0
for k, v in neg.items():
    e = valida(v); rech += bool(e)
    print('  %-45s -> %s' % (k, 'rechazado: ' + e[0] if e else 'ACEPTADO (fallo del validador)'))
print('RESULTADO A apps=%d errores=%d ; agente checks/statuses solo lectura=%s ; negativos rechazados=%d/%d'
      % (len(ids), err, 'si' if lect else 'NO', rech, len(neg)))
sys.exit(0 if (err == 0 and lect and rech == len(neg)) else 1)
EOF
[ "${PIPESTATUS[0]}" -eq 0 ] || falla "A"
grep '^RESULTADO' "$base/a.txt" >> "$base/resultados.txt"

######################################################################
sep "B. Tablas del ADR frente a la tabla publica de permisos por endpoint"
"$PY" -I - "$RAIZ/py" "$RAIZ/esquema/api.github.com.json" "$ADR" "$RAIZ/docs" "$RAIZ/mutaciones" $VERSIONES <<'EOF' | tee "$base/b.txt"
import json, sys, os, collections
sys.path.insert(0, sys.argv[1]); import tablas
doc = json.load(open(sys.argv[2], encoding='utf-8'))
adr = open(sys.argv[3], encoding='utf-8').read()
ddir, mdir, versiones = sys.argv[4], sys.argv[5], sys.argv[6:]
claves = collections.Counter()
for p, v in doc['paths'].items():
    for m, o in v.items():
        if isinstance(o, dict) and 'x-github' in o: claves.update(o['x-github'].keys())
print('claves x-github en la descripcion OpenAPI:', dict(claves))
print('  -> sin permisos por endpoint en OpenAPI: %s' % ('si' if not any('permission' in k.lower() for k in claves) else 'NO'))
ids = tablas.permisos(adr)
print('decision 2 extraida del ADR:', ids)
print('decision 1 extraida del ADR: %d operaciones' % len(tablas.operaciones(adr)))
fallos = {}
for v in versiones:
    idx = tablas.indice(os.path.join(ddir, v + '.json'))
    print('-- tabla %s: %d endpoints server-to-server' % (v, len(idx)))
    fallos[v] = tablas.evalua(adr, idx, ver=(v == versiones[0]))
    print('  fallos: %s' % ('; '.join(fallos[v]) or 'ninguno'))
iguales = len({tuple(x) for x in fallos.values()}) == 1
print('las %d versiones dan el mismo resultado: %s' % (len(versiones), 'si' if iguales else 'NO'))
idx = tablas.indice(os.path.join(ddir, versiones[0] + '.json'))
print('-- enumeracion completa de escrituras (verbo distinto de GET) que alcanza cada App')
nesc = {}
for app, perms in ids.items():
    esc = sorted((r, v) for (v, r), _ in idx.items() if v != 'GET' and tablas.alcanza(perms, idx, v, r))
    nesc[app] = len(esc)
    print('  %s: %d escrituras' % (app, len(esc)))
    for r, v in esc:
        print('    %-6s %-78s %s' % (v, r.replace(tablas.R, '{repo}'), ','.join(sorted({'%s:%s' % pa for pa in idx[(v, r)]}))))
print('-- copias mutadas del ADR (en la carpeta temporal): tienen que fallar')
R2 = '| `revisor` (IA) | otra GitHub App (D2a) | `checks:write`, `contents:read`, `pull_requests:read`, `metadata:read` |'
MUT = {
  'revisor-pr-y-contents-write': (R2, R2.replace('`contents:read`, `pull_requests:read`', '`contents:write`, `pull_requests:write`')),
  'merge-pr-del-agente': ('| `merge-pr` | `finish`, `release` | persona |', '| `merge-pr` | `finish`, `release` | agente |'),
  'agente-sin-checks-read': ('`checks:read`, `statuses:read`, `metadata:read`', '`statuses:read`, `metadata:read`'),
  'leer-permisos-actions-del-agente': ('**solo con el token de la persona** | persona | exige `administration:read`', '**solo con el token de la persona** | agente | exige `administration:read`'),
  'agente-con-administration': ('`contents:write`, `pull_requests:write`, `checks:read`', '`contents:write`, `pull_requests:write`, `administration:write`, `checks:read`'),
  'revisor-sin-fila-de-operacion': ('| check `revision-ia` |', '| check revision |'),
  'comprobar-id-solo-persona': ('| `fetch`, `comprobar-id`, `actualizar-principal` (lectura del remoto) | todo comando que lee el remoto | agente |',
                                '| `fetch` (lectura del remoto) | todo comando que lee el remoto | agente | x |\n   | `comprobar-id`, `actualizar-principal` | todo comando que lee el remoto | persona |'),
}
det = no_aplicadas = 0
for nombre, (viejo, nuevo) in MUT.items():
    if adr.count(viejo) != 1:
        no_aplicadas += 1; print('  %-34s -> NO APLICADA (texto no encontrado)' % nombre); continue
    ruta = os.path.join(mdir, 'F1-6-' + nombre + '.md')
    open(ruta, 'w', encoding='utf-8', newline='\n').write(adr.replace(viejo, nuevo))
    f = tablas.evalua(open(ruta, encoding='utf-8').read(), idx)
    det += bool(f)
    print('  %-34s -> %s' % (nombre, ('detectada: ' + f[0] + (' (+%d)' % (len(f) - 1) if len(f) > 1 else '')) if f else 'NO detectada'))
print('RESULTADO B operaciones=%d x versiones=%d fallos=%d ; versiones iguales=%s ; escrituras agente=%d revisor=%d ; mutaciones del ADR detectadas=%d/%d'
      % (len(tablas.operaciones(adr)), len(versiones), sum(len(x) for x in fallos.values()), 'si' if iguales else 'NO',
         nesc.get('agente', 0), nesc.get('revisor', 0), det, len(MUT)))
print('mutaciones no aplicadas: %d' % no_aplicadas)
sys.exit(0 if (not any(fallos.values()) and iguales and det == len(MUT) and no_aplicadas == 0) else 1)
EOF
[ "${PIPESTATUS[0]}" -eq 0 ] || falla "B"
grep '^RESULTADO' "$base/b.txt" >> "$base/resultados.txt"

######################################################################
sep "C. Permisos de los workflows en los repos y tipo de cuenta (solo GET)"
echo "token usado: $(gh_get_q user '.login + " (" + .type + ")"') -- la persona; ninguna App"
: > "$base/caw.txt"
for r in $REPOS; do
  echo "--- $r (visibilidad: $(gh repo view "$r" --json visibility -q .visibility 2>&1))"
  echo "\$ gh repo view $r --json visibility" >> "$base/llamadas.txt"
  linea=$(gh_get "repos/$r/actions/permissions/workflow")
  printf '%-34s %s\n' "actions/permissions/workflow:" "$linea"
  printf '%s\n' "$linea" | grep -o '"can_approve_pull_request_reviews":[a-z]*' | cut -d: -f2 >> "$base/caw.txt"
done
: > "$base/usuarios.txt"
for u in $USUARIOS; do
  enc=$(printf '%s' "$u" | sed 's/\[/%5B/g; s/\]/%5D/g')
  linea=$(gh_get_q "users/$enc" '.login + " " + .type + " " + (.id|tostring)')
  echo "users/$u: ${linea:-ERROR}"
  [ -n "$linea" ] && echo "$linea" >> "$base/usuarios.txt"
done
echo "RESULTADO C can_approve_pull_request_reviews en los dos repos: $(sort -u "$base/caw.txt" | tr '\n' ' ')(leido con el token de la persona) ; cuentas: $(awk '{print $1"="$2"/"$3}' "$base/usuarios.txt" | tr '\n' ' ')" | tee -a "$base/resultados.txt"

######################################################################
sep "D. Contrato del firmante y regla de reviews de carril (simulados con los type e id reales)"
"$PY" -I - "$RAIZ/usuarios.txt" "$RAIZ/esquema/api.github.com.json" <<'EOF' | tee "$base/d.txt"
import json, sys
cuentas = {}
for l in open(sys.argv[1], encoding='utf-8'):
    login, tipo, id_ = l.split(); cuentas[login] = (tipo, int(id_))
print('cuentas observadas en C:', cuentas)
doc = json.load(open(sys.argv[2], encoding='utf-8'))
comp = doc['components']['schemas']
def res(s):
    while '$ref' in s: s = comp[s['$ref'].split('/')[-1]]
    return s
def props(s):
    s = res(s)
    for k in ('anyOf', 'oneOf', 'allOf'):
        if k in s:
            out = {}
            for x in s[k]: out.update(props(x))
            return out
    return s.get('properties', {})
c = res(doc['paths']['/repos/{owner}/{repo}/commits/{ref}']['get']['responses']['200']['content']['application/json']['schema'])
cm = props(c['properties']['committer'])
rv = res(comp['pull-request-review'])
ru = props(rv['properties']['user'])
print('esquema: commit.committer tiene id=%s type=%s ; review.user tiene id=%s type=%s ; review.state=%s'
      % ('id' in cm, 'type' in cm, 'id' in ru, 'type' in ru, 'state' in rv['properties']))
# Lista de personas (.taskcode/ en la rama de codigo, bajo CODEOWNERS): login e id numerico.
PERSONAS = {'charliebk': cuentas['charliebk'][1]}
EXCLUIDAS = {'web-flow'}
def es_persona(login, id_, tipos, personas):
    return (login is not None and login in personas and personas[login] == id_
            and login not in EXCLUIDAS and tipos.get(login, (None,))[0] == 'User')
T = dict(cuentas); T['otra-persona'] = ('User', 99999901); T['charliebk-nuevo-dueno'] = ('User', 99999902)
# --- reglas 1-5: commit de approve ---
def approve_cuenta(c_, personas, tipos=T):
    return c_.get('verified') is True and es_persona(c_.get('login'), c_.get('id'), tipos, personas)
ID = cuentas['charliebk'][1]
casos = [
  ('persona verificada en la lista (login e id)',      dict(verified=True,  login='charliebk', id=ID), PERSONAS, None, True),
  ('Bot verificado (github-actions[bot])',             dict(verified=True,  login='github-actions[bot]', id=cuentas['github-actions[bot]'][1]), PERSONAS, None, False),
  ('Bot metido en la lista por error',                 dict(verified=True,  login='github-actions[bot]', id=cuentas['github-actions[bot]'][1]), dict(PERSONAS, **{'github-actions[bot]': cuentas['github-actions[bot]'][1]}), None, False),
  ('persona verificada fuera de la lista',             dict(verified=True,  login='otra-persona', id=99999901), PERSONAS, None, False),
  ('web-flow (firma de la plataforma)',                dict(verified=True,  login='web-flow', id=cuentas['web-flow'][1]), PERSONAS, None, False),
  ('web-flow metido en la lista por error',            dict(verified=True,  login='web-flow', id=cuentas['web-flow'][1]), dict(PERSONAS, **{'web-flow': cuentas['web-flow'][1]}), None, False),
  ('persona sin firma',                                dict(verified=False, login='charliebk', id=ID), PERSONAS, None, False),
  ('firmado, correo sin cuenta (committer null)',      dict(verified=True,  login=None, id=None), PERSONAS, None, False),
  ('login reutilizado por otra cuenta (otro id)',      dict(verified=True,  login='charliebk', id=99999902), PERSONAS, dict(T, charliebk=('User', 99999902)), False),
]
ok = 0
for nombre, c_, lista, tipos, esperado in casos:
    r = approve_cuenta(c_, lista, tipos or T); ok += (r == esperado)
    print('  approve: %-46s -> %-9s %s' % (nombre, 'cuenta' if r else 'no cuenta', '' if r == esperado else '(INESPERADO)'))
# --- regla 6: reviews del PR que cuenta carril ---
def review_cuenta(rv_):
    u = rv_['user']
    return rv_['state'] == 'APPROVED' and es_persona(u['login'], u['id'], T, PERSONAS)
rcasos = [
  ('APPROVED de la persona',                 dict(state='APPROVED', user=dict(login='charliebk', id=ID)), True),
  ('APPROVED de la App del agente',          dict(state='APPROVED', user=dict(login='taskcode-agente[bot]', id=99999903)), False),
  ('APPROVED de github-actions[bot]',        dict(state='APPROVED', user=dict(login='github-actions[bot]', id=cuentas['github-actions[bot]'][1])), False),
  ('COMMENTED de la persona',                dict(state='COMMENTED', user=dict(login='charliebk', id=ID)), False),
  ('APPROVED de persona fuera de la lista',  dict(state='APPROVED', user=dict(login='otra-persona', id=99999901)), False),
]
rok = 0
for nombre, r_, esperado in rcasos:
    r = review_cuenta(r_); rok += (r == esperado)
    print('  review:  %-46s -> %-9s %s' % (nombre, 'cuenta' if r else 'no cuenta', '' if r == esperado else '(INESPERADO)'))
print('  nota: web-flow es type=%s; lo excluye la lista y la exclusion explicita, no el type' % cuentas['web-flow'][0])
print('RESULTADO D approve %d/%d casos ; reviews %d/%d casos ; esquema con id y type=%s'
      % (ok, len(casos), rok, len(rcasos), 'si' if all(('id' in cm, 'type' in cm, 'id' in ru, 'type' in ru)) else 'NO'))
sys.exit(0 if (ok == len(casos) and rok == len(rcasos) and all(('id' in cm, 'type' in cm, 'id' in ru, 'type' in ru))) else 1)
EOF
[ "${PIPESTATUS[0]}" -eq 0 ] || falla "D"
grep '^RESULTADO' "$base/d.txt" >> "$base/resultados.txt"

######################################################################
sep "E. Firma SSH local: approve con Plan-Blob, allowed_signers bueno y malo"
unset SSH_AUTH_SOCK; export SSH_ASKPASS_REQUIRE=never   # sin agente de claves: nada en cache
echo "$(ssh -V 2>&1) ; $(git --version)"
K="$base/claves"
ssh-keygen -q -t ed25519 -N '' -C persona-desechable -f "$K/persona"
ssh-keygen -q -t ed25519 -N '' -C agente-desechable -f "$K/agente"
ssh-keygen -q -t ed25519 -N 'frase-desechable' -C persona-con-frase -f "$K/persona-frase"
printf 'persona@local namespaces="git" %s\n' "$(cat "$K/persona.pub")" > "$K/allowed_signers"
printf 'persona@local namespaces="git" %s\n' "$(cat "$K/agente.pub")"  > "$K/allowed_signers_malo"
echo "claves desechables: $(ls "$K" | tr '\n' ' ')"
g() { git -c core.autocrlf=false -c init.defaultBranch=taskcode/tareas -c commit.gpgsign=false -c gpg.format=ssh "$@"; }
export GIT_AUTHOR_NAME=persona GIT_AUTHOR_EMAIL=persona@local GIT_COMMITTER_NAME=persona GIT_COMMITTER_EMAIL=persona@local
export GIT_AUTHOR_DATE="2026-11-14T10:00:00Z" GIT_COMMITTER_DATE="2026-11-14T10:00:00Z"
cd "$base/e" && g init -q .
mkdir -p tareas/TASK-NNN/planificacion && P=tareas/TASK-NNN/planificacion/plan-final.md
printf '# Plan\n\nEnfoque v1.\n' > "$P" && g add "$P" && g commit -qm "plan TASK-NNN (lo escribe el agente, sin firma)"
BLOB=$(g rev-parse "HEAD:$P")
g -c user.signingkey="$K/persona" commit -q -S --allow-empty -m "approve TASK-NNN" --trailer "Plan-Blob: $BLOB"
APPROVE=$(g rev-parse HEAD)
g -c gpg.ssh.allowedSignersFile="$K/allowed_signers" log -1 --format='commit de approve %h ; firma=%G? ; trailer Plan-Blob=%(trailers:key=Plan-Blob,valueonly)' | tr -s '\n' ' '; echo
vc() {  # vc <allowed_signers> <commit>: imprime ok/falla
  if g -c gpg.ssh.allowedSignersFile="$1" verify-commit "$2" >/dev/null 2>&1; then echo ok; else echo falla; fi
}
plan_cuenta() {  # trailer del approve firmado y verificado == blob actual del plan
  local t; t=$(g log -1 --format='%(trailers:key=Plan-Blob,valueonly)' "$APPROVE" | tr -d '\n')
  if [ "$(vc "$K/allowed_signers" "$APPROVE")" = ok ] && [ "$t" = "$(g rev-parse "HEAD:$P")" ]; then echo cuenta; else echo "no cuenta"; fi
}
E1=$(vc "$K/allowed_signers" "$APPROVE");      echo "verify-commit approve, allowed_signers bueno:            $E1"
E2=$(vc "$K/allowed_signers_malo" "$APPROVE"); echo "verify-commit approve, allowed_signers con otra clave:  $E2"
E3=$(plan_cuenta);                             echo "plan aprobado (firma ok y Plan-Blob = blob actual):     $E3"
printf '# Plan\n\nEnfoque v2, cambiado por el agente.\n' > "$P" && g add "$P" && g commit -qm "el agente cambia el plan"
E4=$(plan_cuenta);                             echo "tras cambiar el plan (blob v2):                         $E4"
printf '# Plan\n\nEnfoque v1.\n' > "$P" && g add "$P" && g commit -qm "el agente lo deja como estaba"
E5=$(plan_cuenta);                             echo "tras revertir el contenido (blob v1 otra vez):          $E5"
g -c user.signingkey="$K/agente" commit -q -S --allow-empty -m "approve falso TASK-NNN" --trailer "Plan-Blob: $(g rev-parse "HEAD:$P")"
E6=$(vc "$K/allowed_signers" HEAD);            echo "approve firmado con la clave del agente, bueno:         $E6"
g commit -q --allow-empty -m "approve sin firma TASK-NNN" --trailer "Plan-Blob: $(g rev-parse "HEAD:$P")"
E7=$(vc "$K/allowed_signers" HEAD);            echo "approve sin firma, bueno:                               $E7"
# D4b: clave con frase de paso, sin agente de claves ni TTY: el agente no puede firmar
if g -c user.signingkey="$K/persona-frase" commit -q -S --allow-empty -m "approve con frase" </dev/null >/dev/null 2>"$base/e-frase.err"; then
  E8="firmo (INESPERADO)"; else E8="no firma"; fi
echo "clave con frase, sin ssh-agent ni TTY:                  $E8 ($(head -c 90 "$base/e-frase.err" | tr '\n' ' ')...)"
SK=$(ssh -Q key | grep -c '^sk-ssh-ed25519@openssh.com$')
echo "OpenSSH admite claves FIDO ed25519-sk: $([ "$SK" -ge 1 ] && echo si || echo no) (crear una exige llave fisica: no se prueba)"
cd "$base" || exit 2
ESP="ok falla cuenta no_cuenta cuenta falla falla no_firma"
OBS=$(echo "$E1 $E2 $E3 $E4 $E5 $E6 $E7 $E8" | sed 's/no cuenta/no_cuenta/g; s/no firma/no_firma/')
nok=0; i=1; for e in $ESP; do [ "$(echo "$OBS" | cut -d' ' -f$i)" = "$e" ] && nok=$((nok+1)); i=$((i+1)); done
echo "RESULTADO E casos=$nok/8 como se esperaba ; claves FIDO admitidas por OpenSSH=$SK" | tee -a "$base/resultados.txt"
[ "$nok" -eq 8 ] || falla "E"

######################################################################
metodos=$(grep -c -- 'gh api --method GET' "$base/llamadas.txt")
otros=$(grep 'gh api' "$base/llamadas.txt" | grep -vc -- 'gh api --method GET')
echo "RESULTADO llamadas gh api con GET=$metodos ; gh api con otro metodo=$otros ; gh repo view=$(grep -c 'repo view' "$base/llamadas.txt") ; descargas GET=$(grep -c '^descarga GET' "$base/llamadas.txt")" >> "$base/resultados.txt"

[ "$otros" -eq 0 ] || falla "llamadas de otro metodo"
[ "$(grep -c '^false$' "$base/caw.txt")" -eq 2 ] && ! grep -qv '^false$' "$base/caw.txt" || falla "C"

sep "Resumen"
cat "$base/resultados.txt"
echo "llamadas a GitHub registradas:"; sed 's/^/  /' "$base/llamadas.txt"
if [ -n "$FALLOS" ]; then echo "PoC con fallos en:$FALLOS"; exit 1; fi
echo "PoC sin fallos"

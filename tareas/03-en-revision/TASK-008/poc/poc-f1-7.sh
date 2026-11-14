#!/usr/bin/env bash
# PoC de F1-7 (TASK-008): verificacion en CI, perfiles, cache por arbol y
# runners. SOLO LECTURA en GitHub y en los repos de ToolsaWeb y OpenDockTrading.
#
# (A) Arboles en git local: el arbol del head de un PR al dia = el del merge
#     ref = el del merge --no-ff en develop = el de la release (merge --no-ff
#     de develop en main). Distinto en un PR desactualizado; tras actualizar
#     la rama, otro arbol nuevo; un hotfix sin volver a develop cambia el de
#     la release.
# (B) Simulacion de la busqueda por padres (decisiones 2 a 4 del ADR): `buscar`
#     (Linux) toma los checks `verificacion` verdes de la App 15368 en el commit
#     verificado, sus padres o los padres de estos con `arbol=`, `perfil=`,
#     `runner=` y `clave=` (perfil, runner, huella del arbol, version) iguales
#     en una linea del resumen; el job del runner del perfil solo da acierto si
#     su huella local coincide con `local=`. Cada perfil que exige el evento se
#     busca con su clave (release: completa y pesada). Casos de acierto y fallo
#     y controles negativos (pesada sin correr en la release, otro perfil, otra
#     huella del arbol, otra huella local, huella local sin calcular, otro
#     runner, otra version, rojo, otra App, otro nombre, sin arbol=, tres
#     niveles).
# (C) Coste: entradas leidas del ADR entre los marcadores `entradas-coste`;
#     cada una se comprueba contra su fuente (cifra en su contexto en la
#     linea citada de ToolsaWeb u
#     OpenDockTrading, recuento de tareas, o pagina publica de precios de
#     GitHub descargada con GET); la tabla se calcula desde ellas y se compara
#     con la del ADR (marcadores `tabla-coste`), igual que las cifras de su
#     lectura (`lectura-coste`). Copias mutadas del ADR: tienen que fallar por
#     el error esperado. Ademas, en la descripcion OpenAPI publica de GitHub
#     (commit fijado, carpeta vacia propia): filtros check_name/app_id/filter
#     de GET /commits/{ref}/check-runs, output.summary, app.id, tree y parents.
#
# Uso: poc-f1-7.sh <carpeta-de-trabajo> <ruta-del-ADR> <carpeta-Personal>
#   <carpeta-Personal> contiene ToolsaWeb/ y OpenDockTrading/ (solo lectura).
# Sale con 0 solo si A, B y C salen como se espera, todas las mutaciones se
# aplican y se detectan y no hay llamadas a GitHub de otro metodo que GET.
set -u
[ $# -eq 3 ] || { echo "uso: $0 <trabajo> <adr> <personal>"; exit 2; }
[ -f "$2" ] || { echo "ERROR: no existe el ADR $2"; exit 2; }
[ -d "$3/ToolsaWeb" ] && [ -d "$3/OpenDockTrading" ] || { echo "ERROR: $3 sin ToolsaWeb/ u OpenDockTrading/"; exit 2; }
ADR=$(cd "$(dirname "$2")" && (pwd -W 2>/dev/null || pwd))/$(basename "$2")
PERSONAL=$(cd "$3" && (pwd -W 2>/dev/null || pwd))
mkdir -p "$1" || exit 2
base="$(cd "$1" && pwd)"
if [ -n "$(ls -A "$base")" ] && [ ! -f "$base/.poc-f1-7" ]; then
  echo "ERROR: $base no esta vacia ni es de una ejecucion anterior de la PoC"; exit 2
fi
rm -rf "$base"; mkdir -p "$base/esquema" "$base/precios" "$base/mutaciones" "$base/py"
: > "$base/.poc-f1-7"; : > "$base/llamadas.txt"; : > "$base/resultados.txt"
cd "$base" || exit 2
RAIZ=$(pwd -W 2>/dev/null || pwd)          # ruta C:/... para python.exe
PY=$(command -v python || command -v python3)
SCHEMA_REPO=github/rest-api-description
URL_PRECIOS=https://docs.github.com/en/billing/reference/actions-runner-pricing
URL_PRODUCTO=https://docs.github.com/en/billing/concepts/product-billing/github-actions
sep() { echo; echo "=== $* ==="; }
FALLOS=""
falla() { FALLOS="$FALLOS $1"; echo "FALLO en $1"; }
gh_get_q() {   # unica puerta a la API: siempre GET (se registra)
  echo "\$ gh api --method GET $1 -q ..." >> "$base/llamadas.txt"
  gh api --method GET "$1" -q "$2" 2>/dev/null
}
descarga() {   # descarga <url> <fichero>: GET de un fichero publico
  echo "descarga GET $1" >> "$base/llamadas.txt"
  curl -sSfL -o "$2" "$1" || { echo "ERROR: descarga $1"; exit 1; }
  echo "  $(basename "$2"): $(wc -c < "$2") bytes ; sha256 $(sha256sum "$2" | cut -c1-16)"
}
echo "fecha: $(date -u +%Y-%m-%dT%H:%M:%SZ) ; $(git --version) ; $(gh --version | head -1) ; $("$PY" --version)"

######################################################################
sep "A. Arboles: PR al dia, merge en develop, release en main (git local)"
A="$base/a"; mkdir -p "$A"
g() { git -C "$A" -c core.autocrlf=false -c user.name=poc -c user.email=poc@ejemplo.invalid -c commit.gpgsign=false "$@"; }
t() { g rev-parse "$1^{tree}"; }
mt() { g merge-tree --write-tree "$1" "$2" | head -1; }   # arbol del merge ref de GitHub
g init -q -b main; echo base > "$A/f.txt"; g add f.txt; g commit -qm base
g branch develop
g switch -q -c feature/task-001 develop; echo t1 > "$A/t1.txt"; g add t1.txt; g commit -qm "tarea 1"
H1=$(t feature/task-001); MR1=$(mt develop feature/task-001)
g switch -q develop; g merge -q --no-ff -m "merge tarea 1" feature/task-001; D1=$(t develop)
g switch -q main; g merge -q --no-ff -m "release 1" develop; M1=$(t main)
echo "PR al dia:      head=${H1:0:12} merge-ref=${MR1:0:12} develop=${D1:0:12} main=${M1:0:12}"
ok=0; n=0
chk() { n=$((n+1)); if [ "$2" = "$3" ]; then ok=$((ok+1)); echo "  ok   $1"; else echo "  MAL  $1 (esperado $3, observado $2)"; fi; }
eq() { [ "$1" = "$2" ] && echo igual || echo distinto; }
chk "head del PR al dia = merge ref"                 "$(eq "$H1" "$MR1")" igual
chk "merge --no-ff en develop = head del PR"         "$(eq "$D1" "$H1")" igual
chk "release (merge --no-ff develop en main) = develop" "$(eq "$M1" "$D1")" igual
g switch -q -c feature/task-002 develop; echo t2 > "$A/t2.txt"; g add t2.txt; g commit -qm "tarea 2"
g switch -q develop; echo d2 > "$A/d2.txt"; g add d2.txt; g commit -qm "otra tarea integrada (simulada)"
H2=$(t feature/task-002); MR2=$(mt develop feature/task-002)
echo "PR desactualizado: head=${H2:0:12} merge-ref=${MR2:0:12}"
chk "PR desactualizado: head distinto del merge ref"  "$(eq "$H2" "$MR2")" distinto
g switch -q feature/task-002; g merge -q --no-ff -m "actualizar-rama" develop; H2B=$(t feature/task-002)
chk "tras actualizar la rama: head = merge ref verificado antes" "$(eq "$H2B" "$MR2")" igual
chk "tras actualizar la rama: head distinto del head anterior (arbol nuevo)" "$(eq "$H2B" "$H2")" distinto
g switch -q develop; g merge -q --no-ff -m "merge tarea 2" feature/task-002; D2=$(t develop)
chk "merge en develop = merge ref del PR"            "$(eq "$D2" "$MR2")" igual
g switch -q -c hotfix/task-003 main; echo h > "$A/h.txt"; g add h.txt; g commit -qm hotfix
g switch -q main; g merge -q --no-ff -m "hotfix" hotfix/task-003
chk "hotfix sin volver a develop: merge ref de la release distinto de develop" "$(eq "$(mt main develop)" "$(t develop)")" distinto
g switch -q develop; g merge -q --no-ff -m "vuelta del hotfix" hotfix/task-003
chk "hotfix devuelto a develop: merge ref de la release = develop" "$(eq "$(mt main develop)" "$(t develop)")" igual
g switch -q main; g merge -q --no-ff -m "release 2" develop
chk "release 2 (merge --no-ff) = develop"           "$(eq "$(t main)" "$(t develop)")" igual
echo "RESULTADO A casos=$ok/$n como se esperaba" | tee -a "$base/resultados.txt"
[ "$ok" -eq "$n" ] || falla "A"

######################################################################
sep "B. Simulacion de la busqueda por padres, por perfil y con huella local (decisiones 2 a 4)"
"$PY" -I - "$RAIZ/b" <<'EOF' | tee "$base/b.txt"
import subprocess, hashlib, os, sys
sys.stdout.reconfigure(encoding='utf-8', newline='\n')
R = sys.argv[1]; os.makedirs(R)
def g(*a):
    return subprocess.run(['git', '-C', R, '-c', 'core.autocrlf=false', '-c', 'user.name=poc', '-c', 'user.email=poc@ejemplo.invalid',
                           '-c', 'commit.gpgsign=false', *a], capture_output=True, text=True, check=True).stdout.strip()
def commit(msg, fich=None, txt=None):
    if fich:
        open(os.path.join(R, fich), 'w').write(txt); g('add', fich)
    g('commit', '-q', '--allow-empty', '-m', msg); return g('rev-parse', 'HEAD')
tree = lambda s: g('rev-parse', s + '^{tree}')
parents = lambda s: g('rev-list', '--parents', '-n1', s).split()[1:]
def merge_ref(base, head):   # refs/pull/N/merge: commit con padres base y head
    return g('commit-tree', g('merge-tree', '--write-tree', base, head).splitlines()[0], '-p', base, '-p', head, '-m', 'merge ref')
ACTIONS = 15368
RUNNERS = {'completa': 'github-windows', 'pesada': 'self-hosted-windows', 'rapida': 'github-windows'}
h = lambda *x: hashlib.sha256('|'.join(map(str, x)).encode()).hexdigest()[:16]
# clave (la calcula `buscar`, en Linux) = perfil | runner declarado | huella del arbol | version del workflow;
# la huella local (local=) solo la calcula el job `verificacion` en el runner del perfil.
clave = lambda perfil, runner, harbol, version: h(perfil, runner, harbol, version)
checks = {}   # sha -> check-runs publicados en ese commit
def publicar(sha, arbol, perfiles=('completa',), runners=None, harbol='HA1', hlocal='HL1', version='v1',
             app=ACTIONS, nombre='verificacion', concl='success', con_arbol=True):
    rs = dict(RUNNERS, **(runners or {})); lineas = []
    for p in perfiles:   # una linea del resumen por perfil
        lineas.append(('arbol=%s ' % arbol if con_arbol else '') + 'perfil=%s runner=%s clave=%s local=%s origen=%s'
                      % (p, rs[p], clave(p, rs[p], harbol, version), h(hlocal), sha[:10]))
    checks.setdefault(sha, []).append(dict(name=nombre, app_id=app, conclusion=concl, summary='\n'.join(lineas)))
def candidatos(v):            # el verificado, sus padres y los padres de estos (dos niveles)
    out = [v]
    for p in parents(v):
        out.append(p)
    for p in parents(v):
        out += [q for q in parents(p) if q not in out]
    return out
def buscar(v, perfil, runner, harbol, version):
    """Job `buscar` (Linux): candidatos con el mismo arbol, perfil, runner, huella del arbol y version."""
    arbol, k, out = tree(v), clave(perfil, runner, harbol, version), []
    for c in candidatos(v):
        # = GET /repos/{o}/{r}/commits/{c}/check-runs?check_name=verificacion&app_id=15368&filter=latest
        for ch in [x for x in checks.get(c, []) if x['name'] == 'verificacion' and x['app_id'] == ACTIONS]:
            if ch['conclusion'] != 'success': continue
            for linea in ch['summary'].split('\n'):
                campos = dict(x.split('=', 1) for x in linea.split() if '=' in x)
                if campos.get('arbol') == arbol and campos.get('perfil') == perfil and \
                   campos.get('runner') == runner and campos.get('clave') == k:
                    out.append((c, campos.get('local')))
    return out
def en_runner(cands, hlocal):
    """Job `verificacion` en el runner del perfil: calcula la huella local; sin ella, no hay acierto."""
    if hlocal is None: return None
    for c, local in cands:
        if local == h(hlocal): return c
    return None
def evento(v, perfiles, runners, harbol, hlocal, version):
    """Cada perfil que exige el evento se busca con su propia clave: un acierto solo exime de ese perfil."""
    rs = dict(RUNNERS, **(runners or {}))
    return {p: en_runner(buscar(v, p, rs[p], harbol, version), hlocal) for p in perfiles}
res = []
def caso(tipo, desc, v, esperado, perfiles=('completa',), runners=None, harbol='HA1', hlocal='HL1', version='v1'):
    r = evento(v, perfiles, runners, harbol, hlocal, version)
    obs = 'acierto' if all(r.values()) else 'fallo'
    res.append((tipo, obs == esperado))
    det = '; '.join('%s %s' % (p, ('reutilizada de %s' % o[:10]) if o else 'se ejecuta') for p, o in r.items())
    print('  %-4s %-5s %-8s %s [%s]' % ('ok' if obs == esperado else 'MAL', tipo, obs, desc, det))
g('init', '-q', '-b', 'main'); d0 = commit('base', 'f.txt', 'base'); g('branch', 'develop')
# tarea 1: PR al dia
g('switch', '-q', '-c', 'feature/t1', 'develop'); f1 = commit('tarea 1', 't1.txt', '1')
mr1 = merge_ref(d0, f1)
caso('caso', 'primera verificacion del PR (merge ref nuevo): se verifica', mr1, 'fallo')
publicar(f1, tree(mr1))      # el check del PR va en el head, con el arbol del merge ref en el resumen
f1b = commit('commit vacio en la rama')
caso('caso', 'commit vacio en la rama: el check del head anterior (abuelo del merge ref)', merge_ref(d0, f1b), 'acierto')
publicar(f1b, tree(merge_ref(d0, f1b)))
g('switch', '-q', 'develop'); g('merge', '-q', '--no-ff', '-m', 'merge t1', 'feature/t1'); m1 = g('rev-parse', 'HEAD')
caso('caso', 'consulta tras el push del merge commit en develop: su padre (head del PR) tiene el arbol', m1, 'acierto')
mrel = merge_ref(d0, m1)     # PR de release develop -> main (main todavia en d0)
caso('caso', 'PR de release, solo el perfil completa: abuelo (head del PR de la tarea) con el mismo arbol', mrel, 'acierto')
caso('neg', 'PR de release (completa + pesada): la completa acierta, la pesada no corrio nunca: no hay verde sin ejecutarla',
     mrel, 'fallo', perfiles=('completa', 'pesada'))
publicar(m1, tree(mrel), perfiles=('completa', 'pesada'))   # check del PR de release, en su head (develop), tras correr la pesada
caso('caso', 're-ejecucion del PR de release: completa y pesada reutilizadas', mrel, 'acierto', perfiles=('completa', 'pesada'))
g('switch', '-q', 'main'); g('merge', '-q', '--no-ff', '-m', 'release 1', 'develop'); r1 = g('rev-parse', 'HEAD')
caso('caso', 'consulta tras el push del merge de la release en main (completa y pesada)', r1, 'acierto', perfiles=('completa', 'pesada'))
# tarea 2: PR desactualizado y actualizar-rama
g('switch', '-q', '-c', 'feature/t2', 'develop'); f2 = commit('tarea 2', 't2.txt', '2')
g('switch', '-q', 'develop'); d2 = commit('otra tarea integrada', 'd2.txt', 'd')
mr2 = merge_ref(d2, f2)
caso('caso', 'PR desactualizado: el merge ref es un arbol nuevo', mr2, 'fallo')
publicar(f2, tree(mr2))      # verificado el merge ref (base d2 + head f2)
g('switch', '-q', 'feature/t2'); g('merge', '-q', '--no-ff', '-m', 'actualizar-rama', 'develop'); f2b = g('rev-parse', 'HEAD')
caso('caso', 'tras actualizar-rama (strict): el arbol del head = merge ref verificado antes', merge_ref(d2, f2b), 'acierto')
g('switch', '-q', 'develop'); d3 = commit('develop avanza otra vez', 'd3.txt', 'd3')
g('switch', '-q', 'feature/t2'); g('merge', '-q', '--no-ff', '-m', 'actualizar-rama 2', 'develop'); f2c = g('rev-parse', 'HEAD')
caso('caso', 'develop avanzo y se actualiza la rama otra vez: arbol nuevo', merge_ref(d3, f2c), 'fallo')
# revert y profundidad
c1 = commit('c1', 'r.txt', 'a'); publicar(c1, tree(c1)); c2 = commit('c2', 'r.txt', 'b'); c3 = commit('c3 revierte c2', 'r.txt', 'a')
caso('caso', 'revert al arbol del abuelo', c3, 'acierto')
c4 = commit('c4', 'r.txt', 'b'); c5 = commit('c5', 'r.txt', 'c'); c6 = commit('c6 vuelve al arbol de c1', 'r.txt', 'a')
caso('neg', 'mismo arbol a tres niveles: no se busca (limite conservador)', c6, 'fallo')
# controles negativos sobre un commit nuevo con un unico check en su padre
def nuevo(msg, **kw):
    p = commit(msg, 'n.txt', msg); hh = commit(msg + ' vacio'); publicar(p, tree(p), **kw); return hh
caso('neg', 'check verde de otro perfil (rapida)', nuevo('n1', perfiles=('rapida',)), 'fallo')
caso('neg', 'otra huella del arbol (lockfile o toolchain fijado)', nuevo('n2', harbol='HA2'), 'fallo')
caso('neg', 'misma huella del arbol, otra huella local en el runner (otro rustc, otros datos)', nuevo('n3', hlocal='HL2'), 'fallo')
caso('neg', 'huella local sin calcular en el runner del perfil', nuevo('n4'), 'fallo', hlocal=None)
caso('neg', 'verde de otro runner (self-hosted) para la completa declarada en runner de GitHub',
     nuevo('n5', runners={'completa': 'self-hosted-windows'}), 'fallo')
caso('neg', 'otra version del workflow', nuevo('n6', version='v2'), 'fallo')
caso('neg', 'check en rojo', nuevo('n7', concl='failure'), 'fallo')
caso('neg', 'check verde de otra App con el mismo nombre', nuevo('n8', app=99999), 'fallo')
caso('neg', 'check verde con otro nombre', nuevo('n9', nombre='verificacion-falsa'), 'fallo')
caso('neg', 'check verde sin arbol= en el resumen', nuevo('n10', con_arbol=False), 'fallo')
caso('caso', 'control positivo: mismo montaje con todo igual', nuevo('n11'), 'acierto')
nc = sum(1 for t, _ in res if t == 'caso'); nn = sum(1 for t, _ in res if t == 'neg')
okc = sum(1 for t, o in res if t == 'caso' and o); okn = sum(1 for t, o in res if t == 'neg' and o)
print('RESULTADO B casos=%d/%d ; controles negativos=%d/%d' % (okc, nc, okn, nn))
sys.exit(0 if okc == nc and okn == nn and nc >= 6 and nn >= 4 else 1)
EOF
[ "${PIPESTATUS[0]}" -eq 0 ] || falla "B"
grep '^RESULTADO' "$base/b.txt" >> "$base/resultados.txt"

######################################################################
sep "C1. OpenAPI: filtros de check-runs, output.summary, tree y parents"
SHA=$(gh_get_q "repos/$SCHEMA_REPO/commits/main" '.sha + " " + .commit.committer.date')
echo "commit de $SCHEMA_REPO: $SHA"
descarga "https://raw.githubusercontent.com/$SCHEMA_REPO/${SHA%% *}/descriptions/api.github.com/api.github.com.json" "$base/esquema/api.github.com.json"
"$PY" -I - "$RAIZ/esquema/api.github.com.json" <<'EOF' | tee "$base/c1.txt"
import json, sys
sys.stdout.reconfigure(encoding='utf-8', newline='\n')
doc = json.load(open(sys.argv[1], encoding='utf-8')); comp = doc['components']
print('OpenAPI', doc['openapi'], 'version', doc['info']['version'])
def res(s):
    while isinstance(s, dict) and '$ref' in s:
        parte = s['$ref'].split('/'); s = comp[parte[-2]][parte[-1]]
    return s
def params(ruta, metodo='get'):
    op = doc['paths'][ruta][metodo]
    return {res(p)['name']: res(p) for p in op.get('parameters', [])}
def props(s):
    s = res(s); out = dict(s.get('properties', {}))
    for x in s.get('allOf', []): out.update(props(x))
    return out
oks = []
def ok(desc, cond): oks.append(cond); print('  %-4s %s' % ('ok' if cond else 'MAL', desc))
P = '/repos/{owner}/{repo}/commits/{ref}/check-runs'
pa = params(P)
print('parametros de GET', P, ':', ' '.join(sorted(pa)))
ok('admite check_name', 'check_name' in pa)
ok('admite app_id (entero)', 'app_id' in pa and res(pa['app_id']['schema']).get('type') == 'integer')
ok('admite filter con latest', 'filter' in pa and 'latest' in res(pa['filter']['schema']).get('enum', []))
ok('control negativo: no admite tree_sha (no hay busqueda por arbol)', 'tree_sha' not in pa)
r200 = res(doc['paths'][P]['get']['responses']['200'])['content']['application/json']['schema']
cr = res(props(r200)['check_runs']['items'])
pc = props(cr)
ok('check-run trae head_sha, conclusion, app y output', all(k in pc for k in ('head_sha', 'conclusion', 'app', 'output')))
ok('output.summary existe', 'summary' in props(pc['output']))
app = res(pc['app']); app = res(app.get('anyOf', [app])[0]) if 'anyOf' in app else app
ok('app.id existe (se compara con 15368)', 'id' in props(app))
body = res(doc['paths']['/repos/{owner}/{repo}/check-runs']['post']['requestBody'])['content']['application/json']['schema']
bp = props(body)
if not bp and 'oneOf' in res(body):
    for x in res(body)['oneOf']: bp.update(props(x))
ok('POST /check-runs acepta output.summary (como se escribe el resumen)', 'output' in bp and 'summary' in props(bp['output']))
cm = res(doc['paths']['/repos/{owner}/{repo}/commits/{ref}']['get']['responses']['200'])['content']['application/json']['schema']
pcm = props(cm)
ok('GET /commits/{ref} trae parents[].sha y commit.tree.sha',
   'parents' in pcm and 'sha' in props(pcm['parents']['items']) and 'sha' in props(props(pcm['commit'])['tree']))
print('RESULTADO C1 comprobaciones=%d/%d' % (sum(oks), len(oks)))
sys.exit(0 if all(oks) else 1)
EOF
[ "${PIPESTATUS[0]}" -eq 0 ] || falla "C1"
grep '^RESULTADO' "$base/c1.txt" >> "$base/resultados.txt"

######################################################################
sep "C2. Coste: entradas del ADR contra sus fuentes, tabla calculada contra la del ADR"
descarga "$URL_PRECIOS" "$base/precios/actions-runner-pricing.html"
descarga "$URL_PRODUCTO" "$base/precios/product-billing-github-actions.html"
cat > "$base/py/coste.py" <<'EOF'
import re, os, math, html, datetime
def bloque(txt, marca):
    m = re.search(r'<!-- %s:inicio -->(.*?)<!-- %s:fin -->' % (marca, marca), txt, re.S)
    if not m: raise ValueError('falta la tabla %s en el ADR' % marca)
    filas = [l.strip() for l in m.group(1).splitlines() if l.strip().startswith('|')][2:]
    return [[c.strip() for c in f.strip('|').split('|')] for f in filas]
def texto_html(f):
    t = open(f, encoding='utf-8').read().split('</main>')[0]
    return re.sub(r'\s+', ' ', html.unescape(re.sub(r'<[^>]+>', ' ', t)))
def minutos(v):
    m = re.fullmatch(r'(\d+) min (\d+) s', v); return int(m.group(1)) + int(m.group(2)) / 60
def num(v):
    return float(v.replace('$', '').replace(' ', '').replace('\u00a0', '').replace(',', '.'))
def contar(personal, carpeta, d1, d2):
    n, raiz = 0, os.path.join(personal, carpeta)
    for d in sorted(os.listdir(raiz)):
        f = os.path.join(raiz, d, 'tarea.md')
        if not d.startswith('TASK-') or not os.path.isfile(f): continue
        lineas = open(f, encoding='utf-8').read().split('\n')
        fm = lineas[1:lineas[1:].index('---') + 1] if lineas[0].strip() == '---' else []
        c = [l.split(':', 1)[1].strip().strip('"') for l in fm if l.startswith('creado:')]
        if c and d1 <= c[0] <= d2: n += 1
    return n
# Patrones de las paginas de precios (el valor sale del ADR y se busca dentro).
PAGINA = {'E7': ('precios', r'Windows 2-core \(x64\) actions_windows %s'),
          'E8': ('precios', r'Linux 2-core \(x64\) actions_linux %s'),
          'E9': ('producto', r'remains free independently and does not reduce the included compute usage:[^.]*\.[^.]*\. %s'),
          'E10': ('precios', r'GitHub %s'),
          'E11': ('producto', r'GitHub Free 500 MB %s')}
# Contexto de cada entrada en su linea (revision 1, MEN-2): no basta con que la
# cifra aparezca; tiene que estar en su sitio (la linea de E4 trae tambien la
# medida en caliente, y la de E3 el promedio anterior).
ANCLA = {'E1': r'primera tras el cambio[^|]*\| %s \|',
         'E2': r'\| %s \(TASK-284',
         'E3': r'rondas de [\d,]+ a %s\b',
         'E4': r'%s en fr[ií]o'}
def es(x, d=2):   # formato del ADR: coma decimal y espacio de miles
    s = '{:,.{}f}'.format(x, d).replace(',', ' ').replace('.', ',')
    return s
def evaluar(adr, personal, paginas):
    txt = open(adr, encoding='utf-8').read(); errores, E, ventanas = [], {}, {}
    for c in bloque(txt, 'entradas-coste'):
        ident, valor, fuente = c[0].strip('`*'), re.findall(r'`([^`]+)`', c[2]), c[3]
        if not valor: errores.append('%s sin valor entre comillas' % ident); continue
        v = valor[0]; E[ident] = v
        loc = re.search(r'`([A-Za-z][\w./-]*):(\d+)`', fuente)
        dirs = re.search(r'`([A-Za-z][\w/-]*tareas/04-terminadas)`.*?`(\d{4}-\d\d-\d\d)`.*?`(\d{4}-\d\d-\d\d)`', fuente)
        if ident.startswith('S'):
            estado = 'supuesto (sin fuente que comprobar)'
        elif loc:
            f = os.path.join(personal, loc.group(1)); n = int(loc.group(2))
            lineas = open(f, encoding='utf-8').read().split('\n') if os.path.isfile(f) else []
            linea = lineas[n - 1] if 0 < n <= len(lineas) else ''
            if ident not in ANCLA: errores.append('%s sin contexto declarado en la PoC' % ident)
            ok = ident in ANCLA and re.search(ANCLA[ident] % re.escape(v), linea) is not None
            estado = ('en su contexto en %s:%d' if ok else 'NO ESTA en su contexto en %s:%d') % (loc.group(1), n)
            if not ok: errores.append('%s: %r no esta en su contexto en %s:%d' % (ident, v, loc.group(1), n))
        elif dirs:
            real = contar(personal, dirs.group(1), dirs.group(2), dirs.group(3)); ventanas[ident] = (dirs.group(2), dirs.group(3))
            estado = 'recuento %d con creado en [%s, %s]' % (real, dirs.group(2), dirs.group(3))
            if real < int(v): errores.append('%s: recuento %d < %s' % (ident, real, v))
            elif real > int(v): estado += ' (AVISO: el repo ha crecido desde el ADR)'
        elif ident in PAGINA:
            pag, pat = PAGINA[ident]
            ok = re.search(pat % re.escape(v), paginas[pag]) is not None
            estado = ('en la pagina %s' if ok else 'NO ESTA en la pagina %s') % pag
            if not ok: errores.append('%s: %r no esta en la pagina %s' % (ident, v, pag))
        else:
            estado = 'SIN FUENTE'; errores.append('%s sin fuente comprobable' % ident)
        print('  %-4s %-28s %s' % (ident, v[:28], estado))
    falta = [k for k in 'E1 E2 E3 E4 E5 E6 E7 E8 E9 E10 E11 S1 S2 S3 S4'.split() if k not in E]
    if falta: errores.append('faltan entradas: %s' % ' '.join(falta)); return errores, {}
    def mes(ident):
        d1, d2 = (datetime.date.fromisoformat(x) for x in ventanas.get(ident, ('2000-01-01', '2000-01-30')))
        return round(int(E[ident]) * 30 / ((d2 - d1).days + 1))
    ej = num(E['E3']) * num(E['S4']) + num(E['S1']); fact = num(E['S2'])
    precio = {'win': num(E['E7']), 'linux': num(E['E8']), 'self': 0.0}
    receta = {'C1': ('E5', minutos(E['E1']), 'win'), 'C2': ('E5', minutos(E['E2']), 'self'),
              'C3': ('E5', num(E['S3']), 'linux'), 'C4': ('E6', minutos(E['E4']), 'win'),
              'C5': ('E6', num(E['S3']), 'linux')}
    calc = {}
    for fila, (tar, m, runner) in receta.items():
        tm = mes(tar); mf = math.ceil(round(m * fact, 6)); mm = round(tm * ej * mf)
        calc[fila] = [tm, ej, mf, mm, precio[runner], round(mm * precio[runner], 2)]
    incl = num(E['E11'].replace('USD', ''))
    for fila, partes in (('T1', ['C1', 'C3', 'C4', 'C5']), ('T2', ['C2', 'C3', 'C4', 'C5'])):
        calc[fila] = [None, None, None, sum(calc[p][3] for p in partes),
                      None, round(sum(calc[p][5] for p in partes) - incl, 2)]
    adr_t = {c[0].strip('`*'): c for c in bloque(txt, 'tabla-coste')}
    print('  %-3s %8s %6s %5s %8s %7s %9s   ADR' % ('', 'tar/mes', 'ej/t', 'min', 'min/mes', '$/min', '$/mes'))
    for fila, v in calc.items():
        a = adr_t.get(fila)
        if a is None: errores.append('fila %s no esta en la tabla del ADR' % fila); continue
        celdas = a[4:10]
        for i, x in enumerate(v):
            if x is None: continue
            try: y = num(celdas[i])
            except (ValueError, IndexError): errores.append('%s col %d ilegible: %r' % (fila, i, celdas[i:i+1])); continue
            if abs(y - x) > 0.005: errores.append('%s col %d: ADR %s, calculado %s' % (fila, i + 1, celdas[i], x))
        f = lambda x, d: '-' if x is None else ('%.*f' % (d, x))
        print('  %-3s %8s %6s %5s %8s %7s %9s   %s' % (fila, f(v[0], 0), f(v[1], 1), f(v[2], 0), f(v[3], 0),
              f(v[4], 3), f(v[5], 2), ' | '.join(celdas)))
    for fila in adr_t:
        if fila not in calc: errores.append('fila %s del ADR sin receta' % fila)
    # Cifras de la «Lectura» de la tabla (revision 1, MEN-2), entre marcadores lectura-coste.
    m = re.search(r'<!-- lectura-coste:inicio -->(.*?)<!-- lectura-coste:fin -->', txt, re.S)
    lect = re.sub(r'\s+', ' ', m.group(1)) if m else ''
    if not m: errores.append('lectura: faltan los marcadores lectura-coste')
    for frase in ('%s $ de %s $' % (es(calc['C1'][5]), es(calc['T1'][5])),
                  'unas %d horas' % round(calc['C2'][3] / 60), '(%s min)' % es(calc['C2'][3], 0),
                  'unos %d $' % round(calc['C4'][5])):
        ok = frase in lect
        print('  lectura %-26s %s' % (frase, 'en el ADR' if ok else 'NO ESTA en el ADR'))
        if not ok: errores.append('lectura: falta %r' % frase)
    return errores, calc
EOF
"$PY" -I - "$RAIZ/py" "$ADR" "$PERSONAL" "$RAIZ/precios" "$RAIZ/mutaciones" <<'EOF' | tee "$base/c2.txt"
import sys, shutil, os
sys.stdout.reconfigure(encoding='utf-8', newline='\n')
sys.path.insert(0, sys.argv[1]); import coste
adr, personal, prec, mut = sys.argv[2:6]
paginas = {'precios': coste.texto_html(os.path.join(prec, 'actions-runner-pricing.html')),
           'producto': coste.texto_html(os.path.join(prec, 'product-billing-github-actions.html'))}
print('-- ADR real')
errs, calc = coste.evaluar(adr, personal, paginas)
for e in errs: print('  ERROR', e)
txt = open(adr, encoding='utf-8').read()
# Cada mutacion: descripcion, sustituciones (todas tienen que aplicarse una vez) y el
# error que la tiene que detectar (prefijo), no uno cualquiera.
C4 = '| `C4` | OpenDockTrading | completa (E4) | GitHub Windows | 258 | 3,5 | 5 | 4 515 | 0,010 | 45,15 |'
T1 = '| 19 362 | — | 175,69 |'
T2 = '| 14 714 | — | 42,06 |'
MUT = [('M1 entrada E1 cambiada (22 min 22 s -> 32 min 22 s)', [('`22 min 22 s`', '`32 min 22 s`')], 'E1:'),
       ('M2 celda $/mes de C4 cambiada', [('| 45,15 |', '| 40,15 |')], 'C4 col 6'),
       ('M3 linea de la fuente de E4 cambiada (:8 -> :9)', [('config.yml:8`', 'config.yml:9`')], 'E4:'),
       ('M4 supuesto S1 cambiado (0,5 -> 1,0)', [('| `S1` | Actualizaciones de rama por tarea (`strict`) | `0,5` |',
        '| `S1` | Actualizaciones de rama por tarea (`strict`) | `1,0` |')], 'C1 col 2'),
       ('M5 precio Windows cambiado en las entradas', [('`$0.010`', '`$0.008`')], 'E7:'),
       ('M6 E4 por la medida en caliente de la misma linea (1 min 56 s) y C4, T1 y T2 recalculados a mano',
        [('`4 min 11 s`', '`1 min 56 s`'), (C4, C4.replace('| 5 | 4 515 | 0,010 | 45,15 |', '| 2 | 1 806 | 0,010 | 18,06 |')),
         (T1, '| 16 653 | — | 148,60 |'), (T2, '| 12 005 | — | 14,97 |')], 'E4:'),
       ('M7 cifras de la lectura cambiadas (145 -> 99 horas)', [('unas 145 horas', 'unas 99 horas')], 'lectura'),
       ('M8 E3 por el promedio anterior de la misma linea (3,0 -> 2,2)', [('| `3,0` |', '| `2,2` |')], 'E3:')]
det = noap = 0
for i, (desc, subs, espera) in enumerate(MUT):
    t2, malo = txt, [a for a, _ in subs if txt.count(a) != 1]
    if malo: print('  MAL  %s: no se aplica (%s)' % (desc, malo)); noap += 1; continue
    for a, b in subs: t2 = t2.replace(a, b)
    f = os.path.join(mut, 'adr-m%d.md' % (i + 1)); open(f, 'w', encoding='utf-8').write(t2)
    print('-- %s' % desc)
    e, _ = coste.evaluar(f, personal, paginas)
    ok = any(x.startswith(espera) for x in e)
    if ok: det += 1
    print('  %-4s detectada por %r con %d error(es): %s' % ('ok' if ok else 'MAL', espera, len(e), '; '.join(e)[:200]))
print('RESULTADO C2 errores ADR real=%d ; mutaciones detectadas=%d/%d ; no aplicadas=%d' % (len(errs), det, len(MUT), noap))
sys.exit(0 if not errs and det == len(MUT) and noap == 0 else 1)
EOF
[ "${PIPESTATUS[0]}" -eq 0 ] || falla "C2"
grep '^RESULTADO' "$base/c2.txt" >> "$base/resultados.txt"

######################################################################
metodos=$(grep -c -- 'gh api --method GET' "$base/llamadas.txt")
otros=$(grep 'gh api' "$base/llamadas.txt" | grep -vc -- 'gh api --method GET')
echo "RESULTADO llamadas gh api con GET=$metodos ; gh api con otro metodo=$otros ; descargas GET=$(grep -c '^descarga GET' "$base/llamadas.txt")" >> "$base/resultados.txt"
[ "$otros" -eq 0 ] || falla "llamadas de otro metodo"
sep "Resumen"
cat "$base/resultados.txt"
echo "llamadas a GitHub y descargas registradas:"; sed 's/^/  /' "$base/llamadas.txt"
if [ -n "$FALLOS" ]; then echo "PoC con fallos en:$FALLOS"; exit 1; fi
echo "PoC sin fallos"

#!/usr/bin/env bash
# PoC de F1-8 (TASK-009): seguridad, modelo de amenazas. SOLO LECTURA en
# GitHub y en los repos de ToolsaWeb y OpenDockTrading (se leen sus ficheros,
# nunca se ejecuta nada de ellos).
#
# (A) Hash de confianza local (decision 5, D4b): interfaz unica
#     confianza(repo, comando) -> confiado | cambiado | nunca, con la huella de
#     la orden y de lo que ejecuta (cierre de los ficheros del repo que nombra
#     la orden, transitivo, mas globs declarados) y el almacen fuera del arbol,
#     con clave el directorio comun de Git. Casos: sin cambios, orden cambiada,
#     script nombrado y script transitivo cambiados o borrados, vuelta al
#     contenido confiado, worktree y otro clon, el repo no se da confianza a si
#     mismo, y el limite (fichero alcanzado por glob). Comparacion con D4a
#     (hash de la config). Coste medido con las ordenes reales de los pilotos.
# (B) B1: clasificacion estatica de workflows de ejemplo por clase de job
#     (ejecuta codigo del PR / tiene privilegio), permisos, eventos,
#     repository_dispatch e interpolacion de datos del evento en run:, con
#     controles negativos y el ci.yml real del repo. B2: la regla D5a (la linea
#     arbol= solo cuenta si la publico el job publicador) frente a la de F1-7.
# (C) El ADR: matriz activo -> mitigacion -> duena y tabla de preguntas
#     heredadas leidas entre marcadores; cada duena existe en el plan; temas
#     heredados y activos obligatorios presentes; requisitos citados existen;
#     secciones y fila del indice; sin enlaces a tareas/. Copias mutadas del
#     ADR tienen que fallar por el error esperado.
# (D) GET: visibilidad, permisos por defecto de los workflows, ajustes de
#     Actions, workflows de forks y runners de los tres repos; ficheros en
#     .github/ de los pilotos (git local); donde guarda gh el token.
#
# Uso: poc-f1-8.sh <carpeta-de-trabajo> <ruta-del-ADR> <carpeta-Personal>
# Sale con 0 solo si A, B, C y D salen como se espera, todas las mutaciones se
# aplican y se detectan y no hay llamadas a GitHub de otro metodo que GET.
set -u
[ $# -eq 3 ] || { echo "uso: $0 <trabajo> <adr> <personal>"; exit 2; }
[ -f "$2" ] || { echo "ERROR: no existe el ADR $2"; exit 2; }
[ -d "$3/ToolsaWeb" ] && [ -d "$3/OpenDockTrading" ] || { echo "ERROR: $3 sin ToolsaWeb/ u OpenDockTrading/"; exit 2; }
ADR=$(cd "$(dirname "$2")" && (pwd -W 2>/dev/null || pwd))/$(basename "$2")
REPO=$(cd "$(dirname "$2")/../.." && (pwd -W 2>/dev/null || pwd))
PERSONAL=$(cd "$3" && (pwd -W 2>/dev/null || pwd))
mkdir -p "$1" || exit 2
base="$(cd "$1" && pwd)"
if [ -n "$(ls -A "$base")" ] && [ ! -f "$base/.poc-f1-8" ]; then
  echo "ERROR: $base no esta vacia ni es de una ejecucion anterior de la PoC"; exit 2
fi
rm -rf "$base"; mkdir -p "$base/py" "$base/a" "$base/almacen" "$base/workflows" "$base/mutaciones"
: > "$base/.poc-f1-8"; : > "$base/llamadas.txt"; : > "$base/resultados.txt"
cd "$base" || exit 2
RAIZ=$(pwd -W 2>/dev/null || pwd)          # ruta C:/... para python.exe
PY=$(command -v python || command -v python3)
REPOS="charliebk/TaskCodeTwo charliebk/ToolsaWeb charliebk/OpenDockTrading"
sep() { echo; echo "=== $* ==="; }
FALLOS=""
falla() { FALLOS="$FALLOS $1"; echo "FALLO en $1"; }
gh_get() {     # unica puerta a la API: siempre GET (se registra)
  echo "\$ gh api --method GET $1" >> "$base/llamadas.txt"
  gh api --method GET "$1" 2>&1
}
echo "fecha: $(date -u +%Y-%m-%dT%H:%M:%SZ) ; $(git --version) ; $(gh --version | head -1) ; $("$PY" --version) ; PyYAML $("$PY" -I -c 'import yaml; print(yaml.__version__)')"
echo "repo: $REPO ($(git -C "$REPO" rev-parse --short HEAD)) ; ADR: $(basename "$ADR")"

######################################################################
sep "A. Hash de confianza local (decision 5, D4b)"
cat > "$base/py/confianza.py" <<'EOF'
"""Prototipo de confianza(repo, comando) -> confiado | cambiado | nunca (ADR F1-8, decision 5)."""
import glob, hashlib, json, os, re, subprocess
import yaml

SCRIPT = {'.bat', '.cmd', '.sh', '.ps1', '.py', '.js', '.mjs', '.cjs', '.ts', ''}
TOKEN = re.compile(r'[A-Za-z0-9_.%~\-\\/]+')
_VERSIONADOS = {}

def versionados(repo):
    """Lo que un git pull puede cambiar: los ficheros versionados (no los ignorados ni los generados)."""
    if repo not in _VERSIONADOS:
        out = subprocess.run(['git', '-C', repo, 'ls-files', '-z'], capture_output=True, check=True).stdout
        _VERSIONADOS[repo] = set(x.decode('utf-8', 'replace') for x in out.split(b'\0') if x)
    return _VERSIONADOS[repo]

def _resuelve(repo, tok, dir_rel):
    t = tok.strip('"\'').replace('%~dp0', (dir_rel + '/') if dir_rel else '').replace('\\', '/')
    if not re.search(r'[./]', t) or t in ('.', '..'):
        return None
    for b in ((dir_rel,) if dir_rel else ()) + ('',):
        p = os.path.normpath(os.path.join(repo, b, t))
        try:
            rel = os.path.relpath(p, repo).replace('\\', '/')
        except ValueError:          # otra unidad o ruta UNC: fuera del repo
            continue
        if (not rel.startswith('..') and rel in versionados(repo) and os.path.isfile(p)
                and os.path.splitext(rel)[1].lower() in SCRIPT):   # solo lo que se ejecuta como script
            return rel
    return None

def cierre(repo, orden, globs=(), profundidad=None):
    """Ficheros versionados que la orden nombra y, transitivamente, los que nombran los scripts."""
    vistos, pend = set(), [(r, 0) for r in (_resuelve(repo, t, '') for t in TOKEN.findall(orden)) if r]
    while pend:
        rel, n = pend.pop()
        if rel in vistos:
            continue
        vistos.add(rel)
        if profundidad is not None and n >= profundidad:
            continue
        p = os.path.join(repo, rel)
        if os.path.splitext(rel)[1].lower() not in SCRIPT or os.path.getsize(p) > 2_000_000:
            continue
        txt = open(p, 'rb').read().decode('utf-8', 'replace')
        d = os.path.dirname(rel)
        for t in TOKEN.findall(txt):
            r = _resuelve(repo, t, d)
            if r and r not in vistos:
                pend.append((r, n + 1))
    for g in globs:
        for p in glob.glob(os.path.join(repo, g), recursive=True):
            rel = os.path.relpath(p, repo).replace('\\', '/')
            if os.path.isfile(p) and rel in versionados(repo):
                vistos.add(rel)
    return sorted(vistos)

def huella(repo, orden, globs=()):
    h = hashlib.sha256()
    h.update(b'orden\0' + orden.encode() + b'\0globs\0' + '\0'.join(sorted(globs)).encode() + b'\0')
    fich = cierre(repo, orden, globs)
    for rel in fich:
        h.update(rel.encode() + b'\0' + hashlib.sha256(open(os.path.join(repo, rel), 'rb').read()).digest())
    return h.hexdigest(), fich

def clave_repo(repo):
    d = subprocess.run(['git', '-C', repo, 'rev-parse', '--path-format=absolute', '--git-common-dir'],
                       capture_output=True, text=True, check=True).stdout.strip()
    return os.path.normcase(os.path.abspath(d))

def _config(repo):
    return yaml.safe_load(open(os.path.join(repo, '.taskcode', 'config.yml'), encoding='utf-8')) or {}

def _entrada(repo, comando):
    cfg = _config(repo)
    orden, globs = cfg.get(comando, ''), tuple(cfg.get('confianza_incluye') or ())
    return orden, globs

def _carga(almacen):
    return json.load(open(almacen, encoding='utf-8')) if os.path.exists(almacen) else {}

def confianza(repo, comando, almacen):
    """El almacen vive fuera del arbol; lo que diga el repo no cuenta."""
    k = clave_repo(repo) + '|' + comando
    guardada = _carga(almacen).get(k)
    if guardada is None:
        return 'nunca'
    return 'confiado' if guardada == huella(repo, *_entrada(repo, comando))[0] else 'cambiado'

def confiar(repo, comando, almacen):
    a = _carga(almacen)
    a[clave_repo(repo) + '|' + comando] = huella(repo, *_entrada(repo, comando))[0]
    json.dump(a, open(almacen, 'w', encoding='utf-8'), indent=1)

def confianza_d4a(repo, guardada):
    """D4a descartada: hash solo del fichero de configuracion."""
    h = hashlib.sha256(open(os.path.join(repo, '.taskcode', 'config.yml'), 'rb').read()).hexdigest()
    return 'confiado' if h == guardada else 'cambiado'

def hash_d4a(repo):
    return hashlib.sha256(open(os.path.join(repo, '.taskcode', 'config.yml'), 'rb').read()).hexdigest()
EOF
A="$base/a/repo"; mkdir -p "$A/.taskcode" "$A/scripts/lib" "$A/tests"
g() { git -C "$A" -c core.autocrlf=false -c user.name=poc -c user.email=poc@ejemplo.invalid -c commit.gpgsign=false "$@"; }
printf 'comando_verificacion: "bash scripts/verificar.sh --rapida"\n' > "$A/.taskcode/config.yml"
printf '. scripts/lib/paso.sh\nfor t in tests/*.sh; do bash "$t"; done\n' > "$A/scripts/verificar.sh"
printf 'paso() { echo "$1"; }\n' > "$A/scripts/lib/paso.sh"
printf 'echo test 1\n' > "$A/tests/t1.sh"
printf 'repo de juguete\n' > "$A/README.md"
g init -q -b develop; g add -A; g commit -qm base
"$PY" -I - "$RAIZ/py" "$(cd "$A" && pwd -W)" "$RAIZ/almacen" "$RAIZ/a" <<'EOF' | tee "$base/a.txt"
import os, shutil, subprocess, sys
sys.stdout.reconfigure(encoding='utf-8', newline='\n')
sys.path.insert(0, sys.argv[1]); import confianza as c
repo, alm_dir, adir = sys.argv[2], sys.argv[3], sys.argv[4]
ALM = os.path.join(alm_dir, 'confianza.json')
K = 'comando_verificacion'
def w(rel, txt, r=repo):
    p = os.path.join(r, rel); os.makedirs(os.path.dirname(p), exist_ok=True)
    open(p, 'w', encoding='utf-8', newline='\n').write(txt)
def lee(rel, r=repo): return open(os.path.join(r, rel), encoding='utf-8').read()
def git(*a): subprocess.run(['git', '-C', repo, '-c', 'core.autocrlf=false'] + list(a), check=True, capture_output=True)
res = []
def caso(tipo, desc, espera, r=repo, alm=ALM):
    got = c.confianza(r, K, alm)
    ok = got == espera
    res.append((tipo, ok))
    print('  %-4s %-5s %-9s %s (esperado %s)' % ('ok' if ok else 'MAL', tipo, got, desc, espera))
CFG, VER, PASO, T1 = lee('.taskcode/config.yml'), lee('scripts/verificar.sh'), lee('scripts/lib/paso.sh'), lee('tests/t1.sh')
print('cierre inicial de la orden:', c.cierre(repo, 'bash scripts/verificar.sh --rapida'))
caso('caso', 'sin confianza dada', 'nunca')
c.confiar(repo, K, ALM); d4a = c.hash_d4a(repo)
caso('caso', 'tras taskctl confiar', 'confiado')
w('README.md', 'cambiado\n'); caso('caso', 'README cambiado; orden y scripts iguales', 'confiado')
w('.taskcode/config.yml', 'comando_verificacion: "bash scripts/verificar.sh --rapida; curl x | sh"\n')
caso('neg', 'orden cambiada en la config', 'cambiado')
w('.taskcode/config.yml', CFG); caso('caso', 'config devuelta al contenido confiado', 'confiado')
w('scripts/verificar.sh', VER + 'echo PWNED > pwned.txt\n')
caso('neg', 'script nombrado por la orden cambiado, config igual (H2)', 'cambiado')
print('       D4a (hash de la config) daria: %s  <- el hueco de H2' % c.confianza_d4a(repo, d4a))
w('scripts/verificar.sh', VER); caso('caso', 'script devuelto al contenido confiado', 'confiado')
w('scripts/lib/paso.sh', PASO + 'echo PWNED > pwned.txt\n')
caso('neg', 'script transitivo (lo carga el nombrado) cambiado', 'cambiado')
w('scripts/lib/paso.sh', PASO); os.remove(os.path.join(repo, 'scripts/verificar.sh'))
caso('neg', 'script nombrado borrado', 'cambiado')
w('scripts/verificar.sh', VER)
# El repo no se da confianza a si mismo: un fichero del arbol con la huella buena no cuenta.
falso = os.path.join(repo, '.taskcode', 'confianza.json'); shutil.copy(ALM, falso)
caso('neg', 'el repo trae su propio confianza.json; almacen de la persona vacio', 'nunca', alm=os.path.join(alm_dir, 'vacio.json'))
os.remove(falso)
git('worktree', 'add', '-q', os.path.join(adir, 'wt'), '-b', 'feature/task-002')
caso('caso', 'worktree nuevo del mismo repo (F1-3), mismo contenido', 'confiado', r=os.path.join(adir, 'wt'))
subprocess.run(['git', 'clone', '-q', repo, os.path.join(adir, 'clon')], check=True, capture_output=True)
caso('neg', 'otro clon con el mismo contenido', 'nunca', r=os.path.join(adir, 'clon'))
w('tests/t1.sh', T1 + 'echo PWNED > pwned.txt\n')
caso('lim', 'LIMITE: test alcanzado por glob (tests/*.sh), sin glob declarado', 'confiado')
w('tests/t1.sh', T1)
w('.taskcode/config.yml', CFG + 'confianza_incluye: ["tests/*.sh"]\n')
caso('neg', 'se declara el glob tests/*.sh en la config', 'cambiado')
c.confiar(repo, K, ALM); caso('caso', 'confianza dada de nuevo con el glob', 'confiado')
w('tests/t1.sh', T1 + 'echo PWNED > pwned.txt\n')
caso('neg', 'test cambiado con el glob declarado', 'cambiado')
fuera = os.path.commonpath([os.path.abspath(ALM), os.path.abspath(repo)]) != os.path.abspath(repo)
print('  %-4s almacen fuera del arbol del repo: %s' % ('ok' if fuera else 'MAL', ALM))
n = lambda t: sum(1 for x, ok in res if x == t)
okn = lambda t: sum(1 for x, ok in res if x == t and ok)
print('RESULTADO A1 casos=%d/%d ; controles negativos=%d/%d ; limite documentado=%d/%d ; almacen fuera del arbol=%s'
      % (okn('caso'), n('caso'), okn('neg'), n('neg'), okn('lim'), n('lim'), 'si' if fuera else 'NO'))
sys.exit(0 if all(ok for _, ok in res) and fuera else 1)
EOF
[ "${PIPESTATUS[0]}" -eq 0 ] || falla "A1"
grep '^RESULTADO' "$base/a.txt" >> "$base/resultados.txt"

echo "--- coste del hash con las ordenes reales de los pilotos (solo lectura de ficheros)"
for r in ToolsaWeb OpenDockTrading; do
  echo "  $r: rama $(git -C "$PERSONAL/$r" branch --show-current) @ $(git -C "$PERSONAL/$r" rev-parse --short HEAD)"
done
"$PY" -I - "$RAIZ/py" "$PERSONAL" <<'EOF' | tee "$base/a2.txt"
import os, re, subprocess, sys, time
sys.stdout.reconfigure(encoding='utf-8', newline='\n')
sys.path.insert(0, sys.argv[1]); import confianza as c
import yaml
personal = sys.argv[2]
MED = [('ToolsaWeb', 'comando_verificacion'), ('ToolsaWeb', 'comando_verificacion_freno'),
       ('ToolsaWeb', 'comando_sincronizacion'), ('OpenDockTrading', 'comando_verificacion')]
mal = 0
print('  %-16s %-31s %9s %9s %10s %9s' % ('repo', 'clave', 'nombrados', 'cierre', 'bytes', 'ms (max)'))
for repo, clave in MED:
    r = os.path.join(personal, repo)
    # Se lee la linea tal cual: con YAML estandar, "scripts\verificar.bat" entre comillas
    # dobles da un tabulador vertical (\v); se avisa y se usa la orden literal.
    txtcfg = open(os.path.join(r, '.taskcode', 'config.yml'), encoding='utf-8').read()
    m = re.search(r'^%s:\s*"?(.*?)"?\s*$' % clave, txtcfg, re.M)
    orden = m.group(1) if m else None
    y = (yaml.safe_load(txtcfg) or {}).get(clave)
    if y != orden:
        print('  aviso %s %s: YAML estandar lee %r; la linea dice %r' % (repo, clave, y, orden))
    if not orden:
        print('  MAL  %s %s: sin orden' % (repo, clave)); mal += 1; continue
    nombrados = c.cierre(r, orden, profundidad=0)
    tiempos = []
    for _ in range(5):
        t0 = time.perf_counter(); h, fich = c.huella(r, orden); tiempos.append((time.perf_counter() - t0) * 1000)
    tam = sum(os.path.getsize(os.path.join(r, f)) for f in fich)
    print('  %-16s %-31s %9d %9d %10d %9.1f   orden: %s' % (repo, clave, len(nombrados), len(fich), tam, max(tiempos), orden))
    if len(fich) < len(nombrados) or not nombrados:
        mal += 1
    extra = [f for f in fich if f not in nombrados]
    print('      fuera de la orden y dentro del cierre (%d): %s' % (len(extra), ', '.join(extra[:8]) + (' ...' if len(extra) > 8 else '')))
    # Cuantas veces habria que volver a confiar: commits recientes de develop que tocan el cierre.
    log = subprocess.run(['git', '-C', r, 'log', 'develop', '-n', '100', '--no-merges', '--format=format:@@', '--name-only'],
                         capture_output=True, text=True, encoding='utf-8', errors='replace', check=True).stdout
    commits = [set(l for l in b.splitlines() if l) for b in log.split('@@')[1:]]
    toca = sum(1 for s in commits if s & set(fich))
    print('      commits de develop (ultimos %d, sin merges) que cambian el cierre: %d' % (len(commits), toca))
print('RESULTADO A2 ordenes medidas=%d ; sin orden o cierre vacio=%d' % (len(MED), mal))
sys.exit(0 if mal == 0 else 1)
EOF
[ "${PIPESTATUS[0]}" -eq 0 ] || falla "A2"
grep '^RESULTADO' "$base/a2.txt" >> "$base/resultados.txt"

######################################################################
sep "B1. Clasificacion estatica de workflows por clase de job"
W="$base/workflows"
cp "$REPO/.github/workflows/ci.yml" "$W/P5-ci-real.yml"
cat > "$W/P1-verificacion.yml" <<'EOF'
name: verificacion
on:
  pull_request:
    branches: [develop, main]
  merge_group:
permissions:
  contents: read
jobs:
  buscar:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1
        with: {persist-credentials: false}
      - run: gh api --method GET "repos/$REPO/commits/$SHA/check-runs?check_name=verificacion&app_id=15368&filter=latest"
        env: {GH_TOKEN: "${{ github.token }}", REPO: "${{ github.repository }}", SHA: "${{ github.sha }}"}
  ejecutar:
    needs: buscar
    runs-on: [self-hosted, windows, toolsaweb]
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1
        with: {persist-credentials: false}
      - run: scripts\verificar.bat --perfil "%PERFIL%"
        env: {PERFIL: completa, RAMA_BASE: "${{ github.base_ref }}"}
  verificacion:
    needs: ejecutar
    runs-on: ubuntu-latest
    steps:
      - run: |
          arbol=$(gh api --method GET "repos/$REPO/commits/$SHA" -q .commit.tree.sha)
          echo "arbol=$arbol perfil=completa resultado=$RESULTADO" >> "$GITHUB_STEP_SUMMARY"
        env: {GH_TOKEN: "${{ github.token }}", REPO: "${{ github.repository }}", SHA: "${{ github.sha }}", RESULTADO: "${{ needs.ejecutar.result }}"}
EOF
cat > "$W/P2-carril.yml" <<'EOF'
name: carril
on:
  pull_request_target:
    types: [opened, synchronize, reopened, edited]
permissions:
  contents: read
  pull-requests: read
jobs:
  carril:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1
        with: {persist-credentials: false}
      - run: gh api --method GET "repos/$REPO/pulls/$PR/reviews"
        env: {GH_TOKEN: "${{ github.token }}", REPO: "${{ github.repository }}", PR: "${{ github.event.pull_request.number }}"}
EOF
cat > "$W/P3-revision-reproducir.yml" <<'EOF'
name: revision-reproducir
on:
  pull_request:
    branches: [develop]
permissions:
  contents: read
jobs:
  reproducir:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1
        with: {persist-credentials: false}
      - run: scripts/reproducir.sh > salida.json
      - uses: actions/upload-artifact@v4
        with: {name: reproduccion, path: salida.json}
EOF
cat > "$W/P4-revision-ia.yml" <<'EOF'
name: revision-ia
on:
  workflow_run:
    workflows: [revision-reproducir]
    types: [completed]
permissions:
  actions: read
  contents: read
jobs:
  revision-ia:
    runs-on: ubuntu-latest
    environment: revisor
    steps:
      - uses: actions/download-artifact@v4
        with: {name: reproduccion, path: artefacto, run-id: "${{ github.event.workflow_run.id }}", github-token: "${{ github.token }}"}
      - uses: actions/create-github-app-token@v2
        id: app
        with: {app-id: "${{ vars.REVISOR_APP_ID }}", private-key: "${{ secrets.REVISOR_APP_CLAVE }}"}
      - run: taskctl-revisor publicar --datos artefacto/salida.json
        env: {GH_TOKEN: "${{ steps.app.outputs.token }}", MODELO_CLAVE: "${{ secrets.MODELO_CLAVE }}"}
EOF
neg() { cat > "$W/$1.yml"; }
neg N01-pr-con-checks-write <<'EOF'
on: pull_request
permissions: {contents: read, checks: write}
jobs:
  verificacion:
    runs-on: ubuntu-latest
    steps: [{uses: actions/checkout@v4}, {run: scripts/verificar.sh}]
EOF
neg N02-pr-con-secreto <<'EOF'
on: pull_request
permissions: {contents: read}
jobs:
  reproducir:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: scripts/reproducir.sh --clave "${{ secrets.MODELO_CLAVE }}"
EOF
neg N03-prt-checkout-head <<'EOF'
on: pull_request_target
permissions: {contents: read, pull-requests: write}
jobs:
  carril:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: {ref: "${{ github.event.pull_request.head.sha }}"}
      - run: npm ci && npm test
EOF
neg N04-repository-dispatch <<'EOF'
on:
  repository_dispatch:
    types: [lanzar]
permissions: {contents: read}
jobs:
  x:
    runs-on: ubuntu-latest
    steps: [{run: echo hola}]
EOF
neg N05-interpola-titulo <<'EOF'
on: pull_request_target
permissions: {contents: read}
jobs:
  carril:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ github.event.pull_request.title }}"
EOF
neg N06-sin-permissions <<'EOF'
on: pull_request
jobs:
  verificacion:
    runs-on: ubuntu-latest
    steps: [{uses: actions/checkout@v4}, {run: scripts/verificar.sh}]
EOF
neg N07-write-all <<'EOF'
on: pull_request
permissions: write-all
jobs:
  verificacion:
    runs-on: ubuntu-latest
    steps: [{uses: actions/checkout@v4}, {run: scripts/verificar.sh}]
EOF
neg N08-ejecuta-artefacto <<'EOF'
on:
  workflow_run:
    workflows: [revision-reproducir]
    types: [completed]
permissions: {actions: read, contents: read}
jobs:
  revision-ia:
    runs-on: ubuntu-latest
    environment: revisor
    steps:
      - uses: actions/download-artifact@v4
        with: {name: reproduccion, path: artefacto}
      - run: bash artefacto/reproducir.sh
        env: {MODELO_CLAVE: "${{ secrets.MODELO_CLAVE }}"}
EOF
neg N09-review-con-escritura <<'EOF'
on: pull_request_review
permissions: {contents: read, checks: write}
jobs:
  carril:
    runs-on: ubuntu-latest
    steps: [{run: echo carril}]
EOF
neg N10-interpola-head-ref <<'EOF'
on: pull_request
permissions: {contents: read}
jobs:
  x:
    runs-on: ubuntu-latest
    steps: [{run: "git log origin/${{ github.head_ref }}"}]
EOF
neg N11-prt-checkout-merge-ref <<'EOF'
on: pull_request_target
permissions: {contents: read, checks: write}
jobs:
  carril:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: {ref: "refs/pull/${{ github.event.number }}/merge"}
      - run: scripts/carril.sh
EOF
neg N12-pr-secreto-en-env <<'EOF'
on: pull_request
permissions: {contents: read}
jobs:
  verificacion:
    runs-on: ubuntu-latest
    env: {CLAVE: "${{ secrets.REVISOR_APP_CLAVE }}"}
    steps: [{uses: actions/checkout@v4}, {run: scripts/verificar.sh}]
EOF
neg N13-dispatch-en-lista <<'EOF'
on: [push, repository_dispatch]
permissions: {contents: read}
jobs:
  x:
    runs-on: ubuntu-latest
    steps: [{run: echo hola}]
EOF
neg N14-job-sin-permissions <<'EOF'
on: pull_request
jobs:
  a:
    runs-on: ubuntu-latest
    permissions: {contents: read}
    steps: [{run: echo a}]
  b:
    runs-on: ubuntu-latest
    steps: [{uses: actions/checkout@v4}, {run: echo b}]
EOF
cat > "$base/py/clases.py" <<'EOF'
"""Lint de clases de job (ADR F1-8, decision 1): prototipo del de F5-1 y doctor."""
import re
import yaml

EJECUTA_EV = {'pull_request', 'merge_group', 'push', 'pull_request_review', 'pull_request_review_comment'}
REVIEW_EV = {'pull_request_review', 'pull_request_review_comment'}
SECRETO = re.compile(r'\$\{\{\s*secrets\.(?!GITHUB_TOKEN\b)', re.I)
EVENTO_EN_RUN = re.compile(r'\$\{\{[^}]*\b(github\.event\.|github\.head_ref)', re.I)
HEAD = re.compile(r'head|refs/pull|github\.event\.pull_request', re.I)
INTERPRETE = re.compile(r'^\s*(?:bash|sh|pwsh|powershell|python3?|node|source|\.)\s+(\S+)|^\s*(\./\S+)')

def eventos(y):
    on = y.get('on', y.get(True))
    if isinstance(on, str): return {on}
    if isinstance(on, list): return set(on)
    return set(on or {})

def _pasos(job): return job.get('steps') or []

def _escribe(perm):
    if perm == 'write-all': return True
    return isinstance(perm, dict) and any(str(v) == 'write' for v in perm.values())

def evalua(y):
    ev, errores, clases = eventos(y), [], {}
    if 'repository_dispatch' in ev:
        errores.append('R4 escucha repository_dispatch')
    for nombre, job in (y.get('jobs') or {}).items():
        perm = job.get('permissions', y.get('permissions'))
        texto = yaml.safe_dump(job, allow_unicode=True)
        checkout = [s for s in _pasos(job) if str(s.get('uses', '')).startswith('actions/checkout')]
        head = any(HEAD.search(str((s.get('with') or {}).get('ref', ''))) for s in checkout)
        dirs = [str((s.get('with') or {}).get('path', '.')).rstrip('/') for s in _pasos(job)
                if str(s.get('uses', '')).startswith('actions/download-artifact')]
        artefacto = False
        for s in _pasos(job):
            for linea in str(s.get('run', '')).splitlines():
                m = INTERPRETE.match(linea)
                if m and dirs and any((m.group(1) or m.group(2) or '').lstrip('./').startswith(d + '/') for d in dirs):
                    artefacto = True
                if EVENTO_EN_RUN.search(linea):
                    errores.append('R5 %s interpola datos del evento en run: %s' % (nombre, linea.strip()[:60]))
        ejecuta = bool((ev & EJECUTA_EV) and checkout) or head or artefacto or bool(ev & REVIEW_EV)
        privilegio = _escribe(perm) or bool(SECRETO.search(texto)) or 'environment' in job
        if perm is None:
            errores.append('R2 %s sin bloque permissions (ni del job ni del workflow)' % nombre)
        if perm == 'write-all':
            errores.append('R3 %s con permissions: write-all' % nombre)
        if artefacto:
            errores.append('R7 %s ejecuta un fichero del artefacto' % nombre)
        if ejecuta and privilegio:
            if ev & REVIEW_EV:
                errores.append('R6 %s con escritura o secretos en pull_request_review (corre el workflow del PR; D1b)' % nombre)
            else:
                errores.append('R1 %s ejecuta codigo del PR y tiene privilegio' % nombre)
        clases[nombre] = 'ejecuta' if ejecuta else 'privilegio'
    return clases, errores
EOF
"$PY" -I - "$RAIZ/py" "$RAIZ/workflows" <<'EOF' | tee "$base/b1.txt"
import os, sys
sys.stdout.reconfigure(encoding='utf-8', newline='\n')
sys.path.insert(0, sys.argv[1]); import clases
import yaml
W = sys.argv[2]
POS = {'P1-verificacion': {'buscar': 'ejecuta', 'ejecutar': 'ejecuta', 'verificacion': 'privilegio'},
       'P2-carril': {'carril': 'privilegio'},
       'P3-revision-reproducir': {'reproducir': 'ejecuta'},
       'P4-revision-ia': {'revision-ia': 'privilegio'},
       'P5-ci-real': {'repo': 'ejecuta'}}
NEG = {'N01-pr-con-checks-write': 'R1', 'N02-pr-con-secreto': 'R1', 'N03-prt-checkout-head': 'R1',
       'N04-repository-dispatch': 'R4', 'N05-interpola-titulo': 'R5', 'N06-sin-permissions': 'R2',
       'N07-write-all': 'R3', 'N08-ejecuta-artefacto': 'R7', 'N09-review-con-escritura': 'R6',
       'N10-interpola-head-ref': 'R5', 'N11-prt-checkout-merge-ref': 'R1', 'N12-pr-secreto-en-env': 'R1',
       'N13-dispatch-en-lista': 'R4', 'N14-job-sin-permissions': 'R2'}
okp = okn = 0
for f in sorted(os.listdir(W)):
    n = f[:-4]; y = yaml.safe_load(open(os.path.join(W, f), encoding='utf-8'))
    cl, err = clases.evalua(y)
    if n in POS:
        ok = not err and cl == POS[n]; okp += ok
        print('  %-4s pos  %-26s eventos=%s clases=%s errores=%d' % ('ok' if ok else 'MAL', n, sorted(clases.eventos(y)), cl, len(err)))
        for e in err: print('         ', e)
    else:
        ok = any(e.startswith(NEG[n]) for e in err); okn += ok
        print('  %-4s neg  %-26s esperado %s ; detectado: %s' % ('ok' if ok else 'MAL', n, NEG[n], ' | '.join(err) or 'nada'))
print('RESULTADO B1 positivos sin errores y con su clase=%d/%d ; controles negativos=%d/%d' % (okp, len(POS), okn, len(NEG)))
sys.exit(0 if okp == len(POS) and okn == len(NEG) else 1)
EOF
[ "${PIPESTATUS[0]}" -eq 0 ] || falla "B1"
grep '^RESULTADO' "$base/b1.txt" >> "$base/resultados.txt"

sep "B2. Regla D5a: la linea arbol= solo cuenta si la publico el job publicador"
"$PY" -I - <<'EOF' | tee "$base/b2.txt"
import sys
sys.stdout.reconfigure(encoding='utf-8', newline='\n')
V = 'a1b2c3'                                   # arbol del commit verificado
WF = '.github/workflows/verificacion.yml'      # workflow de TaskCode, leido de la base
def linea(arbol): return 'arbol=%s perfil=completa runner=github clave=k1 local=l1 origen=s0' % arbol
# Check-runs simulados como los devolveria GET /commits/{c}/check-runs (mas el job, si lo hay).
C = [('publicador legitimo (control positivo)', dict(nombre='verificacion', app=15368, concl='success', job='verificacion', wf=WF, resumen=linea(V))),
     ('check-run creado por la API desde un script del PR con checks:write', dict(nombre='verificacion', app=15368, concl='success', job=None, wf=None, resumen=linea(V))),
     ('linea escrita en el resumen del job ejecutor', dict(nombre='ejecutar', app=15368, concl='success', job='ejecutar', wf=WF, resumen=linea(V))),
     ('job verificacion de otro workflow anadido en el PR', dict(nombre='verificacion', app=15368, concl='success', job='verificacion', wf='.github/workflows/otro.yml', resumen=linea(V))),
     ('check verificacion de otra App', dict(nombre='verificacion', app=999, concl='success', job=None, wf=None, resumen=linea(V))),
     ('publicador con otro arbol', dict(nombre='verificacion', app=15368, concl='success', job='verificacion', wf=WF, resumen=linea('ffff00'))),
     ('publicador en rojo', dict(nombre='verificacion', app=15368, concl='failure', job='verificacion', wf=WF, resumen=linea(V)))]
def f17(c):   # regla de F1-7, decision 3: nombre, App, verde y arbol en el resumen
    return c['nombre'] == 'verificacion' and c['app'] == 15368 and c['concl'] == 'success' and ('arbol=%s ' % V) in c['resumen']
def f18(c):   # D5a: ademas, el check-run es el job publicador del workflow de TaskCode
    return f17(c) and c['job'] == 'verificacion' and c['wf'] == WF
ESP18 = [True, False, False, False, False, False, False]
ok = 0; huecos17 = 0
for (desc, c), e in zip(C, ESP18):
    a17, a18 = f17(c), f18(c)
    ok += a18 == e; huecos17 += a17 and not e
    print('  %-4s F1-7 %-8s D5a %-8s %s' % ('ok' if a18 == e else 'MAL', 'acepta' if a17 else 'rechaza', 'acepta' if a18 else 'rechaza', desc))
print('RESULTADO B2 casos D5a=%d/%d ; falsos que la regla de F1-7 aceptaria=%d' % (ok, len(C), huecos17))
sys.exit(0 if ok == len(C) and huecos17 > 0 else 1)
EOF
[ "${PIPESTATUS[0]}" -eq 0 ] || falla "B2"
grep '^RESULTADO' "$base/b2.txt" >> "$base/resultados.txt"

######################################################################
sep "C. El ADR: matriz de activos, preguntas heredadas, requisitos e indice"
cat > "$base/py/adr.py" <<'EOF'
"""Comprobaciones del ADR F1-8 leidas entre marcadores."""
import os, re

def tabla(txt, marca):
    m = re.search(r'<!-- %s:inicio -->(.*?)<!-- %s:fin -->' % (marca, marca), txt, re.S)
    if not m: return None
    filas = [l for l in m.group(1).strip().splitlines() if l.startswith('|')]
    out = []
    for l in filas[2:]:
        celdas = [x.strip() for x in l.strip().strip('|').split('|')]
        out.append(celdas)
    return out

ACTIVOS = ['app del agente', 'app del revisor', 'clave del modelo', 'firma de la persona', 'token de la persona',
           'github_token', '`verificacion`', '`carril`', '`revision-ia`', 'taskcode/tareas', '`comando_*`',
           'runner propio', 'workflows', 'rulesets', 'worktrees', 'lote']
HEREDADAS = ['modificado en el propio PR', 'checks:write', 'relanza', 'firma de la persona', 'credencial',
             'repository_dispatch', 'D5a', '.github/workflows/', 'Merge por el agente', 'Runner propio',
             'pull_request_target', 'resumen falso', 'hook']
SECCIONES = ['## Contexto', '## Modelo de amenazas', '### Zonas', '### Activos, amenazas y mitigaciones',
             '### Dos clases de jobs', '## Alternativas', '## Pruebas de concepto', '## Cómo se ve en GitHub',
             '## Decisión', '## Preguntas heredadas', '## Divergencias con F1-5, F1-6 y F1-7',
             '## Requisitos y evidencia', '## Consecuencias', '## Fuera de alcance', '## Preguntas abiertas',
             '## No verificado']
PARTES = {'local y CI': '**Dos clases de jobs**', 'confianza': '**Confianza local única**',
          'prompt injection': '**Prompt injection: defensa por capacidades.**', 'runner propio': '**Runner propio**'}

def ids_plan(raiz):
    ids = set()
    for f in ('docs/plan/PLAN-TASKCODE-2.md', 'docs/plan/AMPLIACION-2026-11-14.md'):
        ids |= set(re.findall(r'^\| (F\d+-\d+[a-z]?) ', open(os.path.join(raiz, f), encoding='utf-8').read(), re.M))
    return ids

def evaluar(ruta, raiz):
    txt = open(ruta, encoding='utf-8').read()
    e, plan = [], ids_plan(raiz)
    m = tabla(txt, 'matriz-amenazas')
    if m is None: return ['marcadores: falta matriz-amenazas']
    vistos = []
    for f in m:
        if len(f) != 7: e.append('fila con %d columnas: %s' % (len(f), f[:1])); continue
        i, activo, zona, quien, amen, mit, duena = f
        i = i.strip('`')
        vistos.append(activo.lower())
        zs = re.findall(r'Z\d+', zona)
        if not zs or any(z not in ('Z1', 'Z2', 'Z3', 'Z4', 'Z5') for z in zs): e.append('%s: zona invalida %r' % (i, zona))
        if not re.search(r'\((CRÍTICA|ALTA|MEDIA)', amen): e.append('%s: amenaza sin gravedad' % i)
        if len(mit) < 20: e.append('%s: sin mitigacion' % i)
        fids = re.findall(r'F\d+-\d+', duena)
        if not fids: e.append('%s: sin duena' % i)
        for x in fids:
            if x not in plan: e.append('%s: duena %s no existe en el plan' % (i, x))
        if 'no verificado' in duena and 'F2-6' not in duena: e.append('%s: no verificado sin F2-6' % i)
    for a in ACTIVOS:
        if not any(a in v for v in vistos): e.append('falta el activo %r' % a)
    h = tabla(txt, 'preguntas-heredadas')
    if h is None: return e + ['marcadores: falta preguntas-heredadas']
    origenes = set()
    for f in h:
        if len(f) != 4: e.append('heredada con %d columnas' % len(f)); continue
        i, preg, dec, ori = f
        i = i.strip('`')
        if len(dec) < 15: e.append('%s: sin decision' % i)
        o = set(re.findall(r'F1-[567]', ori))
        if not o: e.append('%s: sin origen F1-5/F1-6/F1-7' % i)
        origenes |= o
    for o in ('F1-5', 'F1-6', 'F1-7'):
        if o not in origenes: e.append('ninguna heredada de %s' % o)
    for k in HEREDADAS:
        if not any(k.lower() in f[1].lower() for f in h if len(f) == 4): e.append('falta la pregunta heredada %r' % k)
    for s in SECCIONES:
        if not re.search('^' + re.escape(s) + r'\s*$', txt, re.M): e.append('falta la seccion %r' % s)
    for p, marca in PARTES.items():
        if marca not in txt: e.append('parte del enunciado sin decision propia: %s' % p)
    div = txt.split('## Divergencias con F1-5, F1-6 y F1-7')[-1].split('\n## ')[0]
    for k in ('F1-7, decisión 3', 'arbol=', 'publicador', 'F1-5, decisión 6', 'D1b', 'F1-6, D5a'):
        if k not in div: e.append('divergencias: falta %r' % k)
    cab = next((l for l in txt.splitlines() if l.startswith('- Requisitos:')), '')
    req = open(os.path.join(raiz, 'docs/plan/REQUISITOS-FUNCIONALES.md'), encoding='utf-8').read()
    aud = open(os.path.join(raiz, 'docs/auditoria/AUDITORIA-EXTERNA-2026-11-13.md'), encoding='utf-8').read()
    for r in re.findall(r'\bR[RFN]-\d+', cab):
        if not re.search(r'^\| %s \|' % r, req, re.M): e.append('requisito %s no existe' % r)
    for r in re.findall(r'\bH\d+\b', cab):
        if not re.search(r'^\| %s \|' % r, aud, re.M): e.append('hallazgo %s no existe' % r)
    if re.search(r'\]\([^)]*tareas/', txt): e.append('enlace a tareas/')
    idx = open(os.path.join(raiz, 'docs/adr/README.md'), encoding='utf-8').read()
    if not re.search(r'^\| \[F1-8\]\(%s\) \|.*\| TASK-009 \|' % re.escape(os.path.basename(ruta).replace('-mut', '')), idx, re.M) \
            and not re.search(r'^\| \[F1-8\]\(F1-8-seguridad-modelo-de-amenazas\.md\) \|.*\| TASK-009 \|', idx, re.M):
        e.append('indice: falta la fila de F1-8')
    return e
EOF
"$PY" -I - "$RAIZ/py" "$ADR" "$REPO" "$RAIZ/mutaciones" <<'EOF' | tee "$base/c.txt"
import os, re, sys
sys.stdout.reconfigure(encoding='utf-8', newline='\n')
sys.path.insert(0, sys.argv[1]); import adr
ruta, raiz, mdir = sys.argv[2:5]
txt = open(ruta, encoding='utf-8').read()
m, h = adr.tabla(txt, 'matriz-amenazas'), adr.tabla(txt, 'preguntas-heredadas')
print('-- ADR real: %d activos, %d preguntas heredadas' % (len(m), len(h)))
for f in m: print('  %-5s %-4s duena: %s' % (f[0], f[2], f[6]))
for f in h: print('  %-5s origen: %s' % (f[0], f[3]))
deleg = {o: sum(1 for l in open(os.path.join(raiz, 'docs/adr', n), encoding='utf-8') if 'F1-8' in l)
         for o, n in (('F1-5', 'F1-5-puertas-en-github.md'), ('F1-6', 'F1-6-identidades-agente-revisor-persona.md'),
                      ('F1-7', 'F1-7-verificacion-ci-perfiles-cache-runners.md'))}
print('  lineas que citan F1-8 en los ADR aceptados: %s' % deleg)
errs = adr.evaluar(ruta, raiz)
for x in errs: print('  ERROR', x)
fila = lambda i: next(l for l in txt.splitlines() if l.startswith('| `%s` |' % i))
A2, A12, A6, P6 = fila('A2'), fila('A12'), fila('A6'), fila('P6')
MUT = [('M1 duena de A2 vaciada', [(A2, A2.rsplit('|', 2)[0] + '| |')], 'A2: sin duena'),
       ('M2 duena inexistente (F5-99)', [(A6, A6.replace('| F5-1, F3-8,', '| F5-99, F3-8,'))], 'A6: duena F5-99'),
       ('M3 zona invalida (Z9)', [(A12, A12.replace('| Z4 |', '| Z9 |'))], 'A12: zona'),
       ('M4 fila del runner propio borrada', [(A12 + '\n', '')], "falta el activo 'runner propio'"),
       ('M5 pregunta heredada de repository_dispatch borrada', [(P6 + '\n', '')], "falta la pregunta heredada 'repository_dispatch'"),
       ('M6 mitigacion de A6 vaciada', [(A6, '|'.join(A6.split('|')[:6] + [' x '] + A6.split('|')[7:]))], 'A6: sin mitigacion'),
       ('M7 requisito inexistente en la cabecera (RR-99)', [('RR-03, RR-08,', 'RR-99, RR-08,')], 'requisito RR-99'),
       ('M8 enlace a una carpeta de tareas', [('## Contexto\n', '## Contexto\n\nVer [plan](../../tareas/02-en-curso/TASK-009/planificacion/plan-final.md).\n')], 'enlace a tareas/'),
       ('M9 gravedad quitada en A12', [(A12, A12.replace('(CRÍTICA)', ''))], 'A12: amenaza sin gravedad'),
       ('M10 divergencia con F1-7 (D5a) quitada', [('1. **F1-7, decisión 3** (D5a)', '1. **F1-7** (D5a)')], "divergencias: falta 'F1-7, decisión 3'"),
       ('M11 decision del runner propio quitada', [('6. **Runner propio** (D3b)', '6. Runner (D3b)')], 'parte del enunciado sin decision propia: runner propio'),
       ('M12 marcador de la matriz borrado', [('<!-- matriz-amenazas:fin -->', '')], 'marcadores')]
det = noap = 0
for k, (desc, subs, espera) in enumerate(MUT):
    malo = [a for a, _ in subs if txt.count(a) != 1]
    if malo: print('  MAL  %s: no se aplica' % desc); noap += 1; continue
    t2 = txt
    for a, b in subs: t2 = t2.replace(a, b)
    f = os.path.join(mdir, 'F1-8-seguridad-modelo-de-amenazas-mut%d.md' % (k + 1))
    open(f, 'w', encoding='utf-8', newline='\n').write(t2)
    e = adr.evaluar(f, raiz)
    ok = any(x.startswith(espera) for x in e); det += ok
    print('  %-4s %-50s detectada por %r (%d error(es))' % ('ok' if ok else 'MAL', desc, espera, len(e)))
print('RESULTADO C errores ADR real=%d ; activos=%d ; preguntas heredadas=%d ; mutaciones detectadas=%d/%d ; no aplicadas=%d'
      % (len(errs), len(m), len(h), det, len(MUT), noap))
sys.exit(0 if not errs and det == len(MUT) and noap == 0 else 1)
EOF
[ "${PIPESTATUS[0]}" -eq 0 ] || falla "C"
grep '^RESULTADO' "$base/c.txt" >> "$base/resultados.txt"

######################################################################
sep "D. Ajustes de Actions de los tres repos (solo GET) y workflows de los pilotos"
okd=0; nd=0
chk() { nd=$((nd + 1)); if [ "$2" = "$3" ]; then okd=$((okd + 1)); echo "  ok   $1: $2"; else echo "  MAL  $1: $2 (esperado $3)"; fi; }
for r in $REPOS; do
  echo "--- $r"
  vis=$(gh_get "repos/$r" | "$PY" -I -c 'import json,sys; print(json.load(sys.stdin)["visibility"])')
  echo "  visibilidad: $vis"
  wf=$(gh_get "repos/$r/actions/permissions/workflow"); echo "  actions/permissions/workflow: $wf"
  chk "permisos por defecto de los workflows" "$(printf '%s' "$wf" | grep -o '"default_workflow_permissions":"[a-z]*"' | cut -d'"' -f4)" read
  chk "can_approve_pull_request_reviews" "$(printf '%s' "$wf" | grep -o '"can_approve_pull_request_reviews":[a-z]*' | cut -d: -f2)" false
  echo "  actions/permissions: $(gh_get "repos/$r/actions/permissions")"
  if [ "$vis" = "private" ]; then
    fk=$(gh_get "repos/$r/actions/permissions/fork-pr-workflows-private-repos"); echo "  fork-pr-workflows-private-repos: $fk"
    chk "workflows de PR de forks en privado" "$(printf '%s' "$fk" | grep -o '"run_workflows_from_fork_pull_requests":[a-z]*' | cut -d: -f2)" false
  else
    echo "  fork-pr-contributor-approval: $(gh_get "repos/$r/actions/permissions/fork-pr-contributor-approval")"
  fi
  chk "runners propios registrados" "$(gh_get "repos/$r/actions/runners" | "$PY" -I -c 'import json,sys; print(json.load(sys.stdin)["total_count"])')" 0
done
for r in ToolsaWeb OpenDockTrading; do
  for b in develop main; do
    chk "$r $b: ficheros en .github/ (git ls-tree local)" "$(git -C "$PERSONAL/$r" ls-tree -r --name-only "$b" | grep -c '^\.github/')" 0
  done
done
echo "\$ gh auth status (sin el token)" >> "$base/llamadas.txt"
alm=$(gh auth status 2>&1 | grep -o 'Logged in to github.com account [^ ]* ([a-z]*)' | head -1)
echo "  gh auth status: $alm"
chk "token de la persona guardado en el llavero, al alcance de esta sesion (riesgo A5)" "$(printf '%s' "$alm" | grep -c '(keyring)')" 1
echo "RESULTADO D comprobaciones=$okd/$nd" | tee -a "$base/resultados.txt"
[ "$okd" -eq "$nd" ] || falla "D"

######################################################################
metodos=$(grep -c -- 'gh api --method GET' "$base/llamadas.txt")
otros=$(grep 'gh api' "$base/llamadas.txt" | grep -vc -- 'gh api --method GET')
echo "RESULTADO llamadas gh api con GET=$metodos ; gh api con otro metodo=$otros" >> "$base/resultados.txt"
[ "$otros" -eq 0 ] || falla "llamadas de otro metodo"
sep "Resumen"
cat "$base/resultados.txt"
echo "llamadas a GitHub registradas:"; sed 's/^/  /' "$base/llamadas.txt"
if [ -n "$FALLOS" ]; then echo "PoC con fallos en:$FALLOS"; exit 1; fi
echo "PoC sin fallos"

#!/usr/bin/env bash
# PoC de F1-9 (TASK-010): formato de tarea, plan y especificacion. SOLO
# LECTURA en GitHub (GET) y en los repos de ToolsaWeb, OpenDockTrading y
# TaskCodeTwo (se leen sus tarea.md del arbol de trabajo; nunca se ejecuta
# nada de ellos).
#
# (A) Uso real de los 22 campos de TASK_FIELD_ORDER (src/core/task.ts del
#     plugin 0.16.1): por campo y por repo, cuantas tareas tienen un valor
#     distinto del valor por defecto de la 0.16; claves fuera de la lista,
#     estado frente a carpeta, titulos con \ o ", fechas que PyYAML lee como
#     fecha y no como texto.
# (B) Secciones `##` por repo; Resultado antes o despues de Transiciones.
# (C) Plantilla de tarea.md del ADR (entre marcadores) cargada con PyYAML
#     (parser estandar) con titulos `C:\ruta\` y con comillas; control
#     negativo con el escape de la 0.16 (H11); claves duplicadas.
# (D) Plantilla de plan-final.md del ADR frente al esqueleto de la 0.16
#     (planTemplate de src/commands/plan.ts) y frente al plan-final.md
#     terminado de TASK-009.
# (E) Tabla de campos del ADR (entre marcadores): cada fila de la 0.16 cita
#     la cifra medida en A, cada ruta del formato 2 existe en la plantilla y
#     los campos retirados no estan; copias mutadas del ADR tienen que fallar.
# (F) Spec Kit: GET del listado de plantillas, descarga de spec, plan y tasks
#     a una carpeta propia vacia; cada seccion citada en la tabla de
#     correspondencia existe en la plantilla; control negativo.
# (G) El ADR: IDs citados existen, secciones de la plantilla, sin enlaces a
#     tareas/ ni a otros repos, fila del indice.
#
# Uso: poc-f1-9.sh <carpeta-de-trabajo> <ruta-del-ADR> <carpeta-Personal> <task.ts del plugin 0.16.1>
# Sale con 0 solo si A..G salen como se espera, cada mutacion falla por el
# error esperado y no hay llamadas a GitHub de otro metodo que GET.
set -u
[ $# -eq 4 ] || { echo "uso: $0 <trabajo> <adr> <personal> <task.ts>"; exit 2; }
[ -f "$2" ] || { echo "ERROR: no existe el ADR $2"; exit 2; }
[ -f "$4" ] || { echo "ERROR: no existe $4"; exit 2; }
for r in ToolsaWeb OpenDockTrading TaskCodeTwo; do
  [ -d "$3/$r/tareas" ] || { echo "ERROR: $3/$r/tareas no existe"; exit 2; }
done
win() { (cd "$1" && (pwd -W 2>/dev/null || pwd)); }
ADR="$(win "$(dirname "$2")")/$(basename "$2")"
REPO="$(win "$(dirname "$2")/../..")"
PERSONAL="$(win "$3")"
TASKTS="$(win "$(dirname "$4")")/$(basename "$4")"
PLANTS="$(win "$(dirname "$4")/..")/commands/plan.ts"
[ -f "$PLANTS" ] || { echo "ERROR: no existe $PLANTS"; exit 2; }
mkdir -p "$1" || exit 2
base="$(cd "$1" && pwd)"
if [ -n "$(ls -A "$base")" ] && [ ! -f "$base/.poc-f1-9" ]; then
  echo "ERROR: $base no esta vacia ni es de una ejecucion anterior de la PoC"; exit 2
fi
rm -rf "$base"; mkdir -p "$base/py" "$base/speckit" "$base/mutaciones"
: > "$base/.poc-f1-9"; : > "$base/llamadas.txt"
cd "$base" || exit 2
RAIZ=$(pwd -W 2>/dev/null || pwd)
PY=$(command -v python || command -v python3)
export PYTHONIOENCODING=utf-8
SPECKIT_REF=2ee2b1e8005905db05f5717300a469b3068b908f   # main de github/spec-kit leido el 2026-11-14
sep() { echo; echo "=== $* ==="; }
FALLOS=""
falla() { FALLOS="$FALLOS $1"; echo "FALLO en $1"; }
gh_get() {     # unica puerta a la API: siempre GET (se registra)
  echo "\$ gh api --method GET $*" >> "$base/llamadas.txt"
  gh api --method GET "$@"
}
echo "fecha: $(date -u +%Y-%m-%dT%H:%M:%SZ) ; $(git --version) ; $(gh --version | head -1) ; $("$PY" --version) ; PyYAML $("$PY" -I -c 'import yaml; print(yaml.__version__)')"
echo "ADR: $(basename "$ADR") ; plugin: $(basename "$(dirname "$(dirname "$(dirname "$TASKTS")")")")"
# Commits fijos (leidos el 2026-11-14) para que la medida se pueda reproducir
# aunque los repos sigan avanzando: se extrae tareas/ con git archive (lectura)
# a la carpeta de trabajo. TaskCodeTwo: develop antes de que TASK-010 empezara.
declare -A FIJO=([ToolsaWeb]=93c9618ac156c4357278993df983361afccf1dc2
                 [OpenDockTrading]=fab8d7e05eee2784b9cd4695b94f612a6a1ee257
                 [TaskCodeTwo]=afc3791b5a856d1bc16fa6d762b7f3125071122c)
mkdir -p "$base/repos"
for r in ToolsaWeb OpenDockTrading TaskCodeTwo; do
  mkdir -p "$base/repos/$r"
  git -C "$PERSONAL/$r" archive "${FIJO[$r]}" tareas | tar -x -C "$base/repos/$r" || { echo "ERROR: git archive de $r"; exit 2; }
  echo "repo $r: commit ${FIJO[$r]:0:9} ($(git -C "$PERSONAL/$r" log -1 --format=%cs "${FIJO[$r]}")); HEAD actual $(git -C "$PERSONAL/$r" rev-parse --short HEAD)"
done

cat > "$base/py/poc.py" <<'EOF'
"""PoC de F1-9. Subcomandos: medir, plantilla-tarea, plantilla-plan, tabla, speckit, adr."""
import datetime, glob, json, os, re, sys, textwrap, unicodedata
import yaml

REPOS = ['ToolsaWeb', 'OpenDockTrading', 'TaskCodeTwo']
ABREV = {'ToolsaWeb': 'TW', 'OpenDockTrading': 'ODT', 'TaskCodeTwo': 'TC2'}

def campos_016(task_ts):
    t = open(task_ts, encoding='utf-8').read()
    m = re.search(r'TASK_FIELD_ORDER[^=]*=\s*\[(.*?)\];', t, re.S)
    return re.findall(r"'([a-z_]+)'", m.group(1))

def slugify(titulo):                      # copia de slugify() de src/commands/new.ts (0.16.1)
    b = unicodedata.normalize('NFD', titulo.lower())
    b = re.sub(r'[\u0300-\u036f]', '', b)
    b = re.sub(r'[^a-z0-9]+', '-', b).strip('-')
    s = b[:40].strip('-')
    return s if s else 'tarea'

def partes(texto):
    lin = texto.split('\n')
    if lin[0].strip() != '---':
        return None, lin
    fin = next(i for i in range(1, len(lin)) if lin[i].strip() == '---')
    return '\n'.join(lin[1:fin]), lin[fin + 1:]

def entre(texto, nombre):
    m = re.search(r'<!-- %s:inicio -->(.*?)<!-- %s:fin -->' % (nombre, nombre), texto, re.S)
    if not m:
        raise SystemExit('ERROR: no hay marcadores %s en el ADR' % nombre)
    return m.group(1)

def bloque_codigo(trozo):        # el bloque puede ir sangrado dentro de una lista
    m = re.search(r'```[a-z]*\n(.*?)\n[ \t]*```', trozo, re.S)
    return textwrap.dedent(m.group(1))

# Valor por defecto de la 0.16 y criterio de "distinto" (src/commands/new.ts e import.ts,
# core/config.ts agente_revisor_por_defecto; ninguno de los tres repos lo cambia en su config).
def distinto(campo, v, d):
    if campo in ('id', 'titulo', 'creado'):
        return v not in (None, '')                     # siempre propio
    if campo == 'actualizado':
        return str(v) != str(d.get('creado'))
    if campo == 'rama':
        return v != '%s/%s-%s' % (d.get('tipo'), str(d.get('id')).lower(), slugify(str(d.get('titulo'))))
    defecto = {'tipo': 'feature', 'sprint': 0, 'etiquetas': [], 'complejidad': None,
               'modelo_sugerido': 'sonnet', 'estado': 'planificada', 'plan_aprobado': False,
               'asignado_a': None, 'agente_revisor': 'general-purpose', 'skills_recomendados': [],
               'regla_seleccion_skill': None, 'ultimo_commit_revisado': None, 'revision_codex': False,
               'tokens_diseno': None, 'tokens_implementacion': None, 'tokens_revision': None,
               'dependencias': []}[campo]
    return v != defecto

CARPETA = {'00-planificadas': 'planificada', '01-en-diseno': 'en-diseno', '02-en-curso': 'en-curso',
           '03-en-revision': 'en-revision', '04-terminadas': 'terminada', '05-archivadas': '(archivada)'}

def medir(personal, task_ts, salida):
    campos = campos_016(task_ts)
    print('campos de TASK_FIELD_ORDER: %d' % len(campos))
    res = {'campos': campos, 'repos': {}}
    for r in REPOS:
        fs = sorted(glob.glob(os.path.join(personal, r, 'tareas', '*', 'TASK-*', 'tarea.md')))
        cuenta = {c: 0 for c in campos}
        extra, malos, fecha, titulos, desorden, malestado = {}, 0, 0, 0, 0, []
        secc, resultado_tras_trans, total_secc = {}, 0, 0
        for f in fs:
            fm, cuerpo = partes(open(f, encoding='utf-8').read())
            try:
                d = yaml.safe_load(fm)
            except yaml.YAMLError:
                malos += 1
                continue
            for c in campos:
                if distinto(c, d.get(c), d):
                    cuenta[c] += 1
            for k in d:
                if k not in campos:
                    extra[k] = extra.get(k, 0) + 1
            if list(d.keys()) != [c for c in campos if c in d]:
                desorden += 1
            if isinstance(d.get('creado'), datetime.date):
                fecha += 1
            if re.search(r'[\\"]', str(d.get('titulo'))):
                titulos += 1
            carpeta = f.replace('\\', '/').split('/')[-3]
            if CARPETA.get(carpeta) != d.get('estado'):
                malestado.append('%s:%s/%s' % (d.get('id'), carpeta, d.get('estado')))
            tit = [l[3:].strip() for l in cuerpo if l.startswith('## ')]
            vistos = set()
            for t in tit:
                n = t
                if n.startswith('Descartada'):
                    n = 'Descartada*'
                elif n.startswith('Nota de'):
                    n = 'Nota de*'
                if n not in vistos:
                    secc[n] = secc.get(n, 0) + 1
                    vistos.add(n)
            if 'Resultado' in tit and 'Transiciones' in tit and tit.index('Resultado') > tit.index('Transiciones'):
                resultado_tras_trans += 1
        res['repos'][r] = {'total': len(fs), 'cuenta': cuenta, 'secciones': secc}
        print('%s: %d tarea.md ; frontmatter no YAML para PyYAML: %d ; claves fuera de la lista: %s ; '
              'orden distinto: %d ; creado leido como fecha (sin comillas): %d ; titulos con \\ o ": %d'
              % (r, len(fs), malos, extra or 'ninguna', desorden, fecha, titulos))
        print('   estado distinto de su carpeta: %d %s' % (len(malestado), malestado[:12]))
        print('   ## Resultado despues de ## Transiciones: %d' % resultado_tras_trans)
        res['repos'][r].update({'malos': malos, 'extra': extra, 'malestado': len(malestado),
                                'fecha': fecha, 'resultado_tras_trans': resultado_tras_trans})
    print()
    print('%-24s %8s %8s %8s   (tareas con valor distinto del valor por defecto de la 0.16)' % ('campo', 'TW', 'ODT', 'TC2'))
    for c in campos:
        print('%-24s %8d %8d %8d' % ((c,) + tuple(res['repos'][r]['cuenta'][c] for r in REPOS)))
    print('%-24s %8d %8d %8d' % (('(total tarea.md)',) + tuple(res['repos'][r]['total'] for r in REPOS)))
    print()
    print('secciones ## (tareas que la tienen):')
    nombres = sorted(set().union(*[res['repos'][r]['secciones'] for r in REPOS]),
                     key=lambda n: -sum(res['repos'][r]['secciones'].get(n, 0) for r in REPOS))
    for n in nombres:
        cifras = tuple(res['repos'][r]['secciones'].get(n, 0) for r in REPOS)
        if sum(cifras) >= 2:
            print('%-40s %8d %8d %8d' % ((n[:40],) + cifras))
    print('(secciones con un solo uso en total: %d)' % sum(1 for n in nombres if sum(res['repos'][r]['secciones'].get(n, 0) for r in REPOS) < 2))
    json.dump(res, open(salida, 'w', encoding='utf-8'), ensure_ascii=False, indent=1)
    ok = all(res['repos'][r]['malos'] == 0 and not res['repos'][r]['extra'] for r in REPOS)
    return 0 if ok and len(campos) == 22 else 1

def esc(s):                    # contrato del ADR: comillas dobles, escapar \ y "
    return '"' + s.replace('\\', '\\\\').replace('"', '\\"') + '"'

def esc_016(s):                # lo que hace la 0.16 hoy (H11): solo escapa "
    return '"' + s.replace('"', '\\"') + '"'

def con_titulo(fm, t, f=esc):
    return re.sub(r'^titulo: .*$', lambda m: 'titulo: ' + f(t), fm, count=1, flags=re.M)

def cargar_plantilla_tarea(adr_txt):
    tpl = bloque_codigo(entre(adr_txt, 'plantilla-tarea'))
    fm, cuerpo = partes(tpl)
    return tpl, fm, cuerpo

def plantilla_tarea(adr):
    txt = open(adr, encoding='utf-8').read()
    tpl, fm, cuerpo = cargar_plantilla_tarea(txt)
    fallos = 0
    d = yaml.safe_load(fm)
    print('plantilla: %d claves de primer nivel: %s' % (len(d), ', '.join(d)))
    for t in ['C:\\ruta\\', 'dice "hola"', 'C:\\ruta\\ "con comillas" \\', 'dos puntos: y # almohadilla', 'tilde: canción, eñe']:
        v = yaml.safe_load(con_titulo(fm, t))['titulo']
        ok = v == t
        fallos += not ok
        print('  titulo %-34s -> %-36s %s' % (repr(t), repr(v), 'IGUAL' if ok else 'DISTINTO'))
    # control negativo: el escape de la 0.16 con el mismo titulo
    for t in ['C:\\ruta\\', 'C:\\ruta']:
        try:
            v = yaml.safe_load(con_titulo(fm, t, esc_016))['titulo']
            r = 'carga, pero da %r' % v
            bien = v != t
        except yaml.YAMLError as e:
            r = 'ERROR %s' % type(e).__name__
            bien = True
        fallos += not bien
        print('  control negativo, escape de la 0.16, %-12r -> %s %s' % (t, r, '(esperado: no da el titulo)' if bien else 'INESPERADO'))
    tipos = {k: type(v).__name__ for k, v in d.items() if not isinstance(v, (dict, list, type(None), bool, int))}
    no_texto = [k for k, v in tipos.items() if v != 'str']
    print('  escalares de texto en la plantilla: %s ; no texto: %s' % (len(tipos), no_texto or 'ninguno'))
    fallos += bool(no_texto)
    sin_comillas = [l for l in fm.split('\n') if re.match(r'^\s*[a-z_]+: [A-Za-z0-9]', l)
                    and not re.match(r'^\s*[a-z_]+: (null|true|false|-?\d+)\s*$', l)]
    print('  lineas con texto sin comillas: %s' % (sin_comillas or 'ninguna'))
    fallos += bool(sin_comillas)
    dup = yaml.safe_load('a: 1\na: 2\n')
    print('  observacion: PyYAML safe_load acepta claves duplicadas sin error y se queda con la ultima: %r' % dup)
    tit = [l[3:].strip() for l in cuerpo if l.startswith('## ')]
    print('  secciones de la plantilla: %s' % tit)
    for s in ['Objetivo', 'Criterios de aceptacion', 'Resultado', 'Transiciones']:
        if s not in tit:
            print('  FALTA la seccion obligatoria %s' % s); fallos += 1
    return 1 if fallos else 0

def titulos_md(lineas):
    return [l[3:].strip() for l in lineas if l.startswith('## ')]

def plantilla_plan(adr, plan_ts, plan_009):
    txt = open(adr, encoding='utf-8').read()
    tpl = bloque_codigo(entre(txt, 'plantilla-plan'))
    t_adr = titulos_md(tpl.split('\n'))
    src = open(plan_ts, encoding='utf-8').read()
    cuerpo = src[src.index('export function planTemplate'):]
    cuerpo = cuerpo[:cuerpo.index('\n}\n')]
    t_016 = re.findall(r"'(?:\\n)*## ([^'\\]+)\\n", cuerpo)
    p9 = open(plan_009, encoding='utf-8').read().split('\n')
    t_009 = titulos_md(p9)
    print('titulos ## de la plantilla del ADR : %s' % t_adr)
    print('titulos ## de planTemplate (0.16)  : %s' % t_016)
    print('titulos ## de TASK-009 (2 roles)   : %s' % t_009)
    fallos = 0
    opcionales = {'Troceo'}
    faltan_016 = [t for t in t_016 if t not in t_adr]
    sobran = [t for t in t_adr if t not in t_016 and t not in opcionales]
    print('  de la 0.16 que faltan en la plantilla: %s ; de la plantilla que la 0.16 no tiene (salvo opcionales %s): %s'
          % (faltan_016 or 'ninguno', sorted(opcionales), sobran or 'ninguno'))
    fallos += bool(faltan_016) + bool(sobran)
    # TASK-009 lanzo 2 roles: lleva "Desacuerdos", no "Lo que el rol no cubrio"
    esperados_009 = [t for t in t_adr if t not in opcionales and t != 'Lo que el rol no cubrio']
    faltan_009 = [t for t in esperados_009 if t not in t_009]
    extra_009 = [t for t in t_009 if t not in t_adr]
    print('  de la plantilla (2 roles) que faltan en TASK-009: %s' % (faltan_009 or 'ninguno'))
    print('  de TASK-009 que la plantilla no nombra (texto libre del unificador): %s' % (extra_009 or 'ninguno'))
    fallos += bool(faltan_009)
    for linea, patron in [('Duracion estimada', r'^Duracion estimada: (N|\d+) min'),
                          ('presupuesto de tests', r'^Tests nuevos por criterio \(presupuesto de 1 a 3')]:
        en_adr = any(re.match(patron, l) for l in tpl.split('\n'))
        en_009 = any(re.match(patron.replace('(N|\\d+)', '\\d+'), l) for l in p9)
        print('  linea "%s": en la plantilla %s, en TASK-009 %s' % (linea, en_adr, en_009))
        fallos += not en_adr
        if linea == 'Duracion estimada':
            fallos += not en_009
    primera = tpl.split('\n')[0]
    ok1 = re.match(r'^# Plan — TASK-NNN: ', primera) and re.match(r'^# Plan — TASK-\d+: ', p9[0])
    print('  primera linea "# Plan — TASK-NNN: <titulo>": %s' % ('si' if ok1 else 'NO'))
    fallos += not ok1
    return 1 if fallos else 0

def ruta(d, r):
    for p in r.split('.'):
        if not isinstance(d, dict) or p not in d:
            return False
        d = d[p]
    return True

def claves_planas(d, pre=''):
    out = set()
    for k, v in d.items():
        out.add(pre + k)
        if isinstance(v, dict):
            out |= claves_planas(v, pre + k + '.')
    return out

VEREDICTOS = ('se conserva', 'cambia de sentido', 'se retira', 'se añade')

def tabla(adr, medidas):
    txt = open(adr, encoding='utf-8').read()
    m = json.load(open(medidas, encoding='utf-8'))
    campos = m['campos']
    _, fm, _ = cargar_plantilla_tarea(txt)
    d = yaml.safe_load(fm)
    planas = claves_planas(d)
    filas = [l.strip() for l in entre(txt, 'campos-tarea').split('\n') if l.strip().startswith('| `')]
    vistos, fallos, usados = {}, [], set()
    for f in filas:
        cel = [c.strip() for c in f.strip().strip('|').split('|')]
        if len(cel) < 6:
            fallos.append('fila con %d columnas: %s' % (len(cel), f[:60])); continue
        campo = cel[0].strip('`')
        ver = next((v for v in VEREDICTOS if cel[3].lower().startswith('**' + v) or cel[3].lower().startswith(v)), None)
        if ver is None:
            fallos.append('%s: veredicto desconocido %r' % (campo, cel[3])); continue
        rutas = re.findall(r'`([a-z_.]+)`', cel[4])
        cif = re.match(r'^(\d+) / (\d+) / (\d+)$', cel[2])
        vistos[campo] = vistos.get(campo, 0) + 1
        if campo in campos:
            esperado = tuple(m['repos'][r]['cuenta'][campo] for r in REPOS)
            if not cif:
                fallos.append('%s: sin cifra "N / N / N" (medido %s)' % (campo, esperado))
            elif tuple(int(x) for x in cif.groups()) != esperado:
                fallos.append('%s: cifra del ADR %s distinta de la medida %s' % (campo, cif.groups(), esperado))
            if ver == 'se añade':
                fallos.append('%s: es de la 0.16 y dice "se añade"' % campo)
        else:
            if ver != 'se añade':
                fallos.append('%s: no es de la 0.16 y no dice "se añade"' % campo)
            if cel[2] != 'nuevo':
                fallos.append('%s: campo nuevo sin "nuevo" en la cifra' % campo)
        if ver == 'se retira':
            if rutas:
                fallos.append('%s: se retira pero nombra rutas %s' % (campo, rutas))
            if campo in planas:
                fallos.append('%s: se retira pero esta en la plantilla' % campo)
        else:
            if not rutas:
                fallos.append('%s: sin ruta en el formato 2' % campo)
            for r in rutas:
                usados.add(r)
                if not ruta(d, r):
                    fallos.append('%s: la ruta %s no esta en la plantilla' % (campo, r))
    for c in campos:
        if vistos.get(c, 0) != 1:
            fallos.append('%s: %d filas (se espera 1)' % (c, vistos.get(c, 0)))
    hojas = {k for k in planas if not isinstance(eval_ruta(d, k), dict)}
    huerfanas = sorted(h for h in hojas if h not in usados)
    if huerfanas:
        fallos.append('claves de la plantilla sin fila en la tabla: %s' % huerfanas)
    print('filas leidas: %d ; campos de la 0.16 con fila: %d de %d' % (len(filas), sum(1 for c in campos if vistos.get(c)), len(campos)))
    cuenta = {}
    for f in filas:
        cel = [c.strip() for c in f.strip().strip('|').split('|')]
        for v in VEREDICTOS:
            if cel[3].lower().lstrip('*').startswith(v):
                cuenta[v] = cuenta.get(v, 0) + 1
    print('veredictos: %s' % cuenta)
    for x in fallos:
        print('  ERROR: ' + x)
    return 1 if fallos else 0

def eval_ruta(d, r):
    for p in r.split('.'):
        d = d[p]
    return d

def speckit(adr, carpeta):
    txt = open(adr, encoding='utf-8').read()
    heads = {}
    for f in ('spec', 'plan', 'tasks'):
        p = os.path.join(carpeta, f + '-template.md')
        heads[f] = [re.sub(r'^#+\s*', '', l).strip() for l in open(p, encoding='utf-8') if l.startswith('#')]
    citas = re.findall(r'`(spec|plan|tasks)\.md` «([^»]+)»', entre(txt, 'correspondencia-speckit'))
    fallos = 0
    for f, s in citas:
        ok = any(h.startswith(s) for h in heads[f])
        fallos += not ok
        print('  %-9s «%s»: %s' % (f + '.md', s, 'existe' if ok else 'NO EXISTE'))
    print('citas leidas: %d' % len(citas))
    return 1 if fallos or not citas else 0

def adr_check(adr, repo):
    txt = open(adr, encoding='utf-8').read()
    req = open(os.path.join(repo, 'docs/plan/REQUISITOS-FUNCIONALES.md'), encoding='utf-8').read()
    plan = open(os.path.join(repo, 'docs/plan/PLAN-TASKCODE-2.md'), encoding='utf-8').read()
    aud = ''.join(open(p, encoding='utf-8').read() for p in glob.glob(os.path.join(repo, 'docs/auditoria/*.md')))
    fallos = []
    ids = sorted(set(re.findall(r'\b(R[RFN]-\d+)\b', txt)))
    for i in ids:
        if ('| %s |' % i) not in req:
            fallos.append('%s no existe en REQUISITOS' % i)
    hs = sorted(set(re.findall(r'\b(H\d{1,2})\b', txt)))
    for h in hs:
        if ('| %s |' % h) not in aud:
            fallos.append('%s no existe en la auditoria' % h)
    fs = sorted(set(re.findall(r'\b(F\d-\d+b?)\b', txt)))
    for f in fs:
        if ('| %s |' % f) not in plan:
            fallos.append('%s no existe en el plan' % f)
    print('IDs de requisitos citados: %d ; de la auditoria: %d ; tareas del plan: %d' % (len(ids), len(hs), len(fs)))
    for s in ['## Contexto', '## Alternativas', '## Pruebas de concepto', '## Decisión',
              '## Requisitos y evidencia', '## Consecuencias', '## Fuera de alcance']:
        if ('\n' + s + '\n') not in txt:
            fallos.append('falta la seccion ' + s)
    malos = re.findall(r'\]\(([^)]*(?:tareas/|ToolsaWeb|OpenDockTrading)[^)]*)\)', txt)
    if malos:
        fallos.append('enlaces a tareas/ u otros repos: %s' % malos)
    idx = open(os.path.join(repo, 'docs/adr/README.md'), encoding='utf-8').read()
    if not re.search(r'^\| \[F1-9\]\(F1-9-formato-de-tarea-plan-y-especificacion\.md\) \|.*\| TASK-010 \| aceptada \|$', idx, re.M):
        fallos.append('falta la fila F1-9 en docs/adr/README.md')
    if not re.search(r'Estado: \*\*aceptada\*\* \(2026-11-14\)', txt):
        fallos.append('cabecera sin estado aceptada 2026-11-14')
    for x in fallos:
        print('  ERROR: ' + x)
    return 1 if fallos else 0

if __name__ == '__main__':
    c, a = sys.argv[1], sys.argv[2:]
    f = {'medir': medir, 'plantilla-tarea': plantilla_tarea, 'plantilla-plan': plantilla_plan,
         'tabla': tabla, 'speckit': speckit, 'adr': adr_check}[c]
    sys.exit(f(*a))
EOF
POC="$RAIZ/py/poc.py"
pyrun() { "$PY" -I "$POC" "$@"; }

######################################################################
sep "A. Uso real de los campos de la 0.16 en los tres repos"
pyrun medir "$RAIZ/repos" "$TASKTS" "$RAIZ/medidas.json" || falla A

######################################################################
sep "B. Secciones ## (en la salida de A) frente a REQUISITOS §1"
grep -n "Tareas con \`## Resultado\`" "$REPO/docs/plan/REQUISITOS-FUNCIONALES.md" | cut -c1-200
"$PY" -I -c "
import json; m=json.load(open(r'$RAIZ/medidas.json',encoding='utf-8'))
for s in ('Resultado','Coste','Autorrevision'):
    print('medido hoy %-14s' % s, ' / '.join(str(m['repos'][r]['secciones'].get(s,0)) for r in ('ToolsaWeb','OpenDockTrading','TaskCodeTwo')))
" || falla B
echo "(REQUISITOS mide el repo TaskCode, no TaskCodeTwo: la tercera columna no es comparable)"

######################################################################
sep "C. Plantilla de tarea.md del ADR con un parser YAML estandar (PyYAML)"
pyrun plantilla-tarea "$ADR" || falla C

######################################################################
sep "D. Plantilla de plan-final.md frente a la 0.16 y a TASK-009"
pyrun plantilla-plan "$ADR" "$PLANTS" "$REPO/tareas/04-terminadas/TASK-009/planificacion/plan-final.md" || falla D

######################################################################
sep "E. Tabla de campos del ADR: cada veredicto cita la cifra medida"
pyrun tabla "$ADR" "$RAIZ/medidas.json" || falla E
echo "--- controles negativos (copias mutadas del ADR; cada una tiene que fallar)"
mut() {   # mut <nombre> <error esperado> <programa sed>
  cp "$ADR" "$base/mutaciones/$1.md"
  sed -i "$3" "$base/mutaciones/$1.md"
  if cmp -s "$ADR" "$base/mutaciones/$1.md"; then echo "  $1: la mutacion no cambio nada"; falla "E-$1"; return; fi
  out=$(pyrun tabla "$RAIZ/mutaciones/$1.md" "$RAIZ/medidas.json"); rc=$?
  if [ $rc -ne 0 ] && echo "$out" | grep -q "$2"; then echo "  $1: falla como se espera ($(echo "$out" | grep -m1 "$2" | sed 's/^ *//'))"
  else echo "  $1: NO falla como se espera (rc=$rc)"; falla "E-$1"; fi
}
mut cifra-cambiada "distinta de la medida" '/campos-tarea:inicio/,/campos-tarea:fin/{/^ *| `sprint`/s/| \([0-9]*\) \/ /| 9999 \/ /}'
mut fila-borrada "revision_codex: 0 filas" '/campos-tarea:inicio/,/campos-tarea:fin/{/^ *| `revision_codex`/d}'
mut ruta-inexistente "no esta en la plantilla" '/campos-tarea:inicio/,/campos-tarea:fin/{/^ *| `titulo`/s/`titulo`/`titulo_inventado`/2}'
mut sin-cifra "sin cifra" '/campos-tarea:inicio/,/campos-tarea:fin/{/^ *| `tipo`/s/| \([0-9]*\) \/ \([0-9]*\) \/ \([0-9]*\) |/| muchos |/}'

######################################################################
sep "F. Spec Kit (github/spec-kit, solo GET)"
if gh_get "repos/github/spec-kit/contents/templates?ref=$SPECKIT_REF" --jq '.[] | .name' > "$base/speckit/listado.txt" 2>"$base/speckit/err.txt"; then
  echo "plantillas en templates/ ($SPECKIT_REF): $(tr '\n' ' ' < "$base/speckit/listado.txt")"
  for f in spec plan tasks; do
    gh_get "repos/github/spec-kit/contents/templates/$f-template.md?ref=$SPECKIT_REF" -H "Accept: application/vnd.github.raw" > "$base/speckit/$f-template.md" || falla "F-$f"
    echo "  $f-template.md: $(wc -c < "$base/speckit/$f-template.md") bytes, sha256 $(sha256sum "$base/speckit/$f-template.md" | cut -c1-16)"
  done
  pyrun speckit "$ADR" "$RAIZ/speckit" || falla F
  cp "$ADR" "$base/mutaciones/speckit.md"
  sed -i '/correspondencia-speckit:inicio/,/correspondencia-speckit:fin/s/«User Scenarios & Testing»/«User Stories Inventadas»/' "$base/mutaciones/speckit.md"
  if pyrun speckit "$RAIZ/mutaciones/speckit.md" "$RAIZ/speckit" > "$base/speckit/neg.txt"; then
    echo "  control negativo: NO falla"; falla F-neg
  else echo "  control negativo (seccion inventada): falla como se espera ($(grep -m1 'NO EXISTE' "$base/speckit/neg.txt" | sed 's/^ *//'))"; fi
else
  echo "NO VERIFICADO: no se pudo leer github/spec-kit ($(head -1 "$base/speckit/err.txt"))"
fi

######################################################################
sep "G. El ADR: IDs, secciones, enlaces e indice"
pyrun adr "$ADR" "$REPO" || falla G

######################################################################
sep "Llamadas a GitHub"
cat "$base/llamadas.txt"
if grep -v -- '--method GET' "$base/llamadas.txt" | grep -q .; then falla llamadas; fi

sep "RESULTADO"
if [ -z "$FALLOS" ]; then echo "OK: A..G como se espera, mutaciones detectadas, solo GET"; exit 0
else echo "FALLOS:$FALLOS"; exit 1; fi

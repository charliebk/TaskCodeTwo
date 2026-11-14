#!/usr/bin/env bash
# PoC de F1-6 (TASK-007): identidades del agente implementador, del revisor IA
# y de la persona. SOLO LECTURA en GitHub.
#
# (A) Permisos de cada identidad validados contra el esquema `app-permissions`
#     de la descripcion OpenAPI publica (github/rest-api-description, commit
#     fijado, descargada a una carpeta vacia propia). Controles negativos.
# (B) Endpoints que alcanza cada identidad segun la tabla publica de permisos
#     por endpoint de las GitHub Apps (github/docs,
#     src/github-apps/data/<version>/server-to-server-permissions.json, la
#     fuente de la pagina «Permissions required for GitHub Apps»). La
#     descripcion OpenAPI no declara permisos por endpoint (x-github solo
#     trae enabledForGitHubApps): se comprueba y se dice.
#     Ademas: la tabla operacion -> identidad es ejecutable con esos permisos
#     y cada permiso se usa (minimo).
# (C) GET de actions/permissions/workflow en los dos repos y type de
#     /users/{login} (persona, web-flow, github-actions[bot]).
# (D) Simulacion del contrato del firmante que usa `carril` (con los type
#     reales de C) y controles negativos sobre la tabla de permisos.
# (E) Firma SSH local con claves desechables: commit de approve con trailer
#     Plan-Blob, git verify-commit con allowed_signers bueno y malo, clave con
#     frase de paso sin agente ni TTY, y soporte de claves FIDO (-sk).
#
# Uso: poc-f1-6.sh <carpeta-de-trabajo>
# Ningun POST/PUT/PATCH/DELETE: todas las llamadas a la API pasan por gh_get.
set -u
[ $# -eq 1 ] || { echo "uso: $0 <trabajo>"; exit 2; }
mkdir -p "$1" || exit 2
base="$(cd "$1" && pwd)"
if [ -n "$(ls -A "$base")" ] && [ ! -f "$base/.poc-f1-6" ]; then
  echo "ERROR: $base no esta vacia ni es de una ejecucion anterior de la PoC"; exit 2
fi
rm -rf "$base"; mkdir -p "$base/esquema" "$base/docs" "$base/claves" "$base/e"; : > "$base/.poc-f1-6"
: > "$base/llamadas.txt"; : > "$base/resultados.txt"
cd "$base" || exit 2
RAIZ=$(pwd -W 2>/dev/null || pwd)          # ruta C:/... para python.exe
PY=$(command -v python || command -v python3)
REPOS="charliebk/TaskCodeTwo charliebk/taskcodetwo-poc-f1-1"
USUARIOS="charliebk web-flow github-actions[bot]"
SCHEMA_REPO=github/rest-api-description
DOCS_REPO=github/docs
sep() { echo; echo "=== $* ==="; }

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
descarga() {   # descarga() <url> <fichero>: GET de un fichero publico
  echo "descarga GET $1" >> "$base/llamadas.txt"
  curl -sSfL -o "$2" "$1" || { echo "ERROR: descarga $1"; exit 1; }
  echo "  $(basename "$2"): $(wc -c < "$2") bytes ; sha256 $(sha256sum "$2" | cut -c1-16)"
}

######################################################################
sep "Descargas publicas (cada una en su carpeta vacia)"
gh --version | head -1
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
# Identidades y tabla operacion -> identidad (decision del ADR), en un JSON
# que leen A, B y D.
cat > "$base/identidades.json" <<'EOF'
{
  "agente":  {"contents": "write", "pull_requests": "write", "metadata": "read"},
  "revisor": {"checks": "write", "contents": "read", "pull_requests": "read", "metadata": "read"}
}
EOF

######################################################################
sep "A. Permisos de cada identidad contra el esquema app-permissions"
"$PY" -I - "$RAIZ/esquema/api.github.com.json" "$RAIZ/identidades.json" <<'EOF' | tee "$base/a.txt"
import json, sys, copy
doc = json.load(open(sys.argv[1], encoding='utf-8'))
ids = json.load(open(sys.argv[2], encoding='utf-8'))
print('OpenAPI', doc['openapi'], 'version', doc['info']['version'])
ap = doc['components']['schemas']['app-permissions']
props = ap['properties']
print('app-permissions: %d permisos; additionalProperties=%s' % (len(props), ap.get('additionalProperties', '(no declarado)')))
for p in ('contents', 'pull_requests', 'metadata', 'checks', 'administration', 'workflows', 'statuses', 'secrets'):
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
    print('%-8s %-60s -> %s' % (nombre, ', '.join('%s:%s' % kv for kv in perms.items()), 'VALIDO' if not e else 'INVALIDO: ' + '; '.join(e)))
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
print('RESULTADO A identidades=%d errores=%d ; negativos rechazados=%d/%d' % (len(ids), err, rech, len(neg)))
EOF
grep '^RESULTADO' "$base/a.txt" >> "$base/resultados.txt"

######################################################################
sep "B. Endpoints que alcanza cada identidad (tabla publica de permisos por endpoint)"
"$PY" -I - "$RAIZ/esquema/api.github.com.json" "$RAIZ/identidades.json" "$RAIZ/docs" $VERSIONES <<'EOF' | tee "$base/b.txt"
import json, sys, os, collections
doc = json.load(open(sys.argv[1], encoding='utf-8'))
ids = json.load(open(sys.argv[2], encoding='utf-8'))
ddir, versiones = sys.argv[3], sys.argv[4:]
# 1) La descripcion OpenAPI no trae permisos por endpoint: claves de x-github.
claves = collections.Counter()
for p, v in doc['paths'].items():
    for m, o in v.items():
        if isinstance(o, dict) and 'x-github' in o: claves.update(o['x-github'].keys())
print('claves x-github en la descripcion OpenAPI:', dict(claves))
print('  -> sin permisos por endpoint en OpenAPI: %s' % ('si' if not any('permission' in k.lower() for k in claves) else 'NO'))
NIVEL = {'read': 1, 'write': 2}
def indice(fich):
    d = json.load(open(fich, encoding='utf-8'))
    idx = {}
    for perm, v in d.items():
        for e in v['permissions']:
            if e.get('server-to-server'):
                idx.setdefault((e['verb'].upper(), e['requestPath']), []).append((perm, e['access'], e['additional-permissions']))
    return idx
def alcanza(perms, idx, verbo, ruta, workflow=False):
    ent = idx.get((verbo, ruta))
    if ent is None: return None                      # endpoint no listado
    ok = any(NIVEL.get(perms.get(p, ''), 0) >= NIVEL[a] for p, a, _ in ent if p != 'workflows')
    if workflow:   # fichero bajo .github/workflows/: ademas workflows:write (descripcion del permiso)
        ok = ok and perms.get('workflows') == 'write' and any(p == 'workflows' for p, _, _ in ent)
    return ok
R = '/repos/{owner}/{repo}'
# (operacion, verbo, ruta, workflow?, esperado agente, esperado revisor)
# esperado: True alcanza, False no alcanza. Las filas marcadas 'limite' son alcances
# del agente que el ADR deja escritos como limite (no los quita la plataforma).
EP = [
  ('crear o mover rama (equivale a push)',  'POST',  R + '/git/refs', False,            True,  False, ''),
  ('abrir-pr',                              'POST',  R + '/pulls', False,               True,  False, ''),
  ('editar el PR (titulo, descripcion)',    'PATCH', R + '/pulls/{pull_number}', False, True,  False, ''),
  ('actualizar rama del PR (boton)',        'PUT',   R + '/pulls/{pull_number}/update-branch', False, True, False, ''),
  ('merge-pr',                              'PUT',   R + '/pulls/{pull_number}/merge', False, True, False, 'limite'),
  ('merge directo de ramas',                'POST',  R + '/merges', False,              True,  False, 'limite'),
  ('review (tambien APPROVE)',              'POST',  R + '/pulls/{pull_number}/reviews', False, True, False, 'limite'),
  ('descartar una review',                  'PUT',   R + '/pulls/{pull_number}/reviews/{review_id}/dismissals', False, True, False, 'limite'),
  ('check revision-ia',                     'POST',  R + '/check-runs', False,          False, True,  ''),
  ('estado de commit (statuses)',           'POST',  R + '/statuses/{sha}', False,      False, False, ''),
  ('aplicar-puertas (crear ruleset)',       'POST',  R + '/rulesets', False,            False, False, ''),
  ('aplicar-puertas (cambiar ruleset)',     'PUT',   R + '/rulesets/{ruleset_id}', False, False, False, ''),
  ('permisos de workflows del repo',        'PUT',   R + '/actions/permissions/workflow', False, False, False, ''),
  ('anadir colaborador',                    'PUT',   R + '/collaborators/{username}', False, False, False, ''),
  ('secreto de Actions',                    'PUT',   R + '/actions/secrets/{secret_name}', False, False, False, ''),
  ('fichero normal por API',                'PUT',   R + '/contents/{path}', False,     True,  False, ''),
  ('fichero de .github/workflows/ por API', 'PUT',   R + '/contents/{path}', True,      False, False, ''),
  ('tag (objeto anotado)',                  'POST',  R + '/git/tags', False,            True,  False, 'limite'),
  ('release',                               'POST',  R + '/releases', False,            True,  False, 'limite'),
  ('leer commit y su firma',                'GET',   R + '/commits/{ref}', False,       True,  True,  ''),
  ('leer ficheros del PR',                  'GET',   R + '/pulls/{pull_number}/files', False, True, True, ''),
  ('leer-puertas (reglas de la rama)',      'GET',   R + '/rules/branches/{branch}', False, True, True, ''),
  ('comentar en el PR',                     'POST',  R + '/issues/{issue_number}/comments', False, True, False, ''),
]
fallos = 0; detalle = {}
for v in versiones:
    idx = indice(os.path.join(ddir, v + '.json'))
    print('-- tabla %s: %d endpoints server-to-server' % (v, len(idx)))
    for op, verbo, ruta, wf, e_ag, e_rv, nota in EP:
        ag, rv = alcanza(ids['agente'], idx, verbo, ruta, wf), alcanza(ids['revisor'], idx, verbo, ruta, wf)
        ok = (ag == e_ag) and (rv == e_rv)
        fallos += not ok
        perm = ','.join(sorted({'%s:%s' % (p, a) for p, a, _ in idx.get((verbo, ruta), [])})) or 'NO LISTADO'
        detalle.setdefault((op, verbo, ruta, wf), []).append((ag, rv, perm))
        if v == versiones[0]:
            f = lambda x: '-' if x is None else ('si' if x else 'no')
            print('  %-38s %-6s %-58s exige=%-45s agente=%-2s revisor=%-2s %s%s' % (
                op, verbo, ruta.replace(R, '') + (' [workflow]' if wf else ''), perm, f(ag), f(rv),
                '(limite) ' if nota and ag else '', '' if ok else 'INESPERADO'))
iguales = all(len(set(x)) == 1 for x in detalle.values())
print('las %d versiones dan el mismo resultado en las %d filas: %s' % (len(versiones), len(EP), 'si' if iguales else 'NO'))
# 2) Minimo: cada permiso de cada App lo usa al menos una operacion que la App tiene asignada.
USO = {
  'agente':  [('POST', R + '/git/refs'), ('POST', R + '/pulls'), ('PATCH', R + '/pulls/{pull_number}'), ('GET', R + '/commits/{ref}')],
  'revisor': [('POST', R + '/check-runs'), ('GET', R + '/pulls/{pull_number}/files'), ('GET', R + '/commits/{ref}')],
}
idx = indice(os.path.join(ddir, versiones[0] + '.json'))
sobran = 0
for nombre, perms in ids.items():
    usados = set()
    for verbo, ruta in USO[nombre]:
        for p, a, _ in idx.get((verbo, ruta), []):
            if NIVEL.get(perms.get(p, ''), 0) >= NIVEL[a]: usados.add(p)
    resto = sorted(set(perms) - usados - {'metadata'})   # metadata:read es obligatorio en toda App
    sobran += len(resto)
    print('minimo %-8s usados=%s ; sin uso=%s' % (nombre, ','.join(sorted(usados)), ','.join(resto) or 'ninguno'))
print('RESULTADO B filas=%d x versiones=%d fallos=%d ; versiones iguales=%s ; permisos sin uso=%d'
      % (len(EP), len(versiones), fallos, 'si' if iguales else 'NO', sobran))
EOF
grep '^RESULTADO' "$base/b.txt" >> "$base/resultados.txt"

######################################################################
sep "C. Permisos de los workflows en los repos y tipo de cuenta (solo GET)"
for r in $REPOS; do
  echo "--- $r (visibilidad: $(gh repo view "$r" --json visibility -q .visibility 2>&1))"
  echo "\$ gh repo view $r --json visibility" >> "$base/llamadas.txt"
  printf '%-34s ' "actions/permissions/workflow:"; gh_get "repos/$r/actions/permissions/workflow"
done
: > "$base/usuarios.txt"
for u in $USUARIOS; do
  enc=$(printf '%s' "$u" | sed 's/\[/%5B/g; s/\]/%5D/g')
  linea=$(gh_get_q "users/$enc" '.login + " " + .type + " " + (.id|tostring)')
  echo "users/$u: ${linea:-ERROR}"
  [ -n "$linea" ] && echo "$linea" >> "$base/usuarios.txt"
done
caw=$(for r in $REPOS; do gh_get_q "repos/$r/actions/permissions/workflow" '.can_approve_pull_request_reviews'; done | sort -u | tr '\n' ' ')
echo "RESULTADO C can_approve_pull_request_reviews en los dos repos: $caw; tipos: $(awk '{print $1"="$2}' "$base/usuarios.txt" | tr '\n' ' ')" | tee -a "$base/resultados.txt"

######################################################################
sep "D. Contrato del firmante (simulado con los type reales) y controles negativos"
"$PY" -I - "$RAIZ/usuarios.txt" "$RAIZ/identidades.json" "$RAIZ/docs/${VERSIONES%% *}.json" <<'EOF' | tee "$base/d.txt"
import json, sys, copy
tipos = {}
for l in open(sys.argv[1], encoding='utf-8'):
    login, tipo, _ = l.split(); tipos[login] = tipo
tipos['otra-persona'] = 'User'                       # cuenta de persona ficticia, fuera de la lista
ids = json.load(open(sys.argv[2], encoding='utf-8'))
print('type observado en C:', {k: v for k, v in tipos.items() if k != 'otra-persona'})
# --- Contrato del firmante (decision del ADR) ---
PERSONAS = {'charliebk'}                             # .taskcode/personas en la rama de codigo, bajo CODEOWNERS
EXCLUIDAS = {'web-flow'}                             # firma de la propia plataforma: nunca vale
def firmante_valido(c, personas):
    login = c.get('committer')
    return (c.get('verified') is True and login is not None and login in personas
            and login not in EXCLUIDAS and tipos.get(login) == 'User')
casos = [
  ('persona verificada en la lista',               dict(verified=True,  committer='charliebk'), PERSONAS, True),
  ('Bot verificado (github-actions[bot])',         dict(verified=True,  committer='github-actions[bot]'), PERSONAS, False),
  ('Bot metido en la lista por error',             dict(verified=True,  committer='github-actions[bot]'), PERSONAS | {'github-actions[bot]'}, False),
  ('persona verificada fuera de la lista',         dict(verified=True,  committer='otra-persona'), PERSONAS, False),
  ('web-flow (firma de la plataforma)',            dict(verified=True,  committer='web-flow'), PERSONAS, False),
  ('web-flow metido en la lista por error',        dict(verified=True,  committer='web-flow'), PERSONAS | {'web-flow'}, False),
  ('persona sin firma',                            dict(verified=False, committer='charliebk'), PERSONAS, False),
  ('firmado, correo sin cuenta (committer null)',  dict(verified=True,  committer=None), PERSONAS, False),
]
ok = 0
for nombre, c, lista, esperado in casos:
    r = firmante_valido(c, lista); ok += (r == esperado)
    print('  firmante: %-46s -> %-9s %s' % (nombre, 'cuenta' if r else 'no cuenta', '' if r == esperado else '(INESPERADO)'))
print('  nota: web-flow es type=%s; lo excluye la lista y la exclusion explicita, no el type' % tipos.get('web-flow'))
# --- Invariantes "ninguna IA aprueba ni mergea" sobre la tabla de permisos ---
d = json.load(open(sys.argv[3], encoding='utf-8'))
idx = {}
for perm, v in d.items():
    for e in v['permissions']:
        if e.get('server-to-server'): idx.setdefault((e['verb'].upper(), e['requestPath']), []).append((perm, e['access']))
N = {'read': 1, 'write': 2}
def alc(perms, verbo, ruta):
    return any(N.get(perms.get(p, ''), 0) >= N[a] for p, a in idx.get((verbo, ruta), []) if p != 'workflows')
R = '/repos/{owner}/{repo}'
def invariantes(t):
    f = []
    rv, ag = t['revisor'], t['agente']
    if alc(rv, 'POST', R + '/pulls/{pull_number}/reviews'): f.append('el revisor puede crear reviews (APPROVE)')
    if alc(rv, 'PUT', R + '/pulls/{pull_number}/merge'): f.append('el revisor puede mergear')
    if alc(rv, 'POST', R + '/git/refs'): f.append('el revisor puede empujar')
    for who, p in (('agente', ag), ('revisor', rv)):
        if alc(p, 'POST', R + '/rulesets'): f.append('%s administra rulesets' % who)
        if p.get('workflows'): f.append('%s tiene workflows' % who)
    if alc(ag, 'POST', R + '/check-runs'): f.append('el agente puede publicar checks (revision-ia)')
    if alc(ag, 'POST', R + '/statuses/{sha}'): f.append('el agente puede publicar statuses')
    return f
f0 = invariantes(ids)
print('  invariantes sobre la tabla del ADR: %s' % ('se cumplen' if not f0 else 'FALLAN: ' + '; '.join(f0)))
def muta(fn):
    t = copy.deepcopy(ids); fn(t); return t
neg = {
  'revisor con pull_requests:write': muta(lambda t: t['revisor'].__setitem__('pull_requests', 'write')),
  'revisor con contents:write':      muta(lambda t: t['revisor'].__setitem__('contents', 'write')),
  'agente con administration:write': muta(lambda t: t['agente'].__setitem__('administration', 'write')),
  'agente con workflows:write':      muta(lambda t: t['agente'].__setitem__('workflows', 'write')),
  'agente con checks:write':         muta(lambda t: t['agente'].__setitem__('checks', 'write')),
}
det = 0
for k, t in neg.items():
    f = invariantes(t); det += bool(f)
    print('  negativo %-34s -> %s' % (k, 'detectado: ' + f[0] if f else 'NO detectado'))
print('RESULTADO D firmante %d/%d casos ; invariantes=%s ; negativos detectados=%d/%d'
      % (ok, len(casos), 'ok' if not f0 else 'FALLO', det, len(neg)))
EOF
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

######################################################################
metodos=$(grep -c -- 'gh api --method GET' "$base/llamadas.txt")
otros=$(grep 'gh api' "$base/llamadas.txt" | grep -vc -- 'gh api --method GET')
echo "RESULTADO llamadas gh api con GET=$metodos ; gh api con otro metodo=$otros ; gh repo view=$(grep -c 'repo view' "$base/llamadas.txt") ; descargas GET=$(grep -c '^descarga GET' "$base/llamadas.txt")" >> "$base/resultados.txt"

sep "Resumen"
cat "$base/resultados.txt"
echo "llamadas a GitHub registradas:"; sed 's/^/  /' "$base/llamadas.txt"

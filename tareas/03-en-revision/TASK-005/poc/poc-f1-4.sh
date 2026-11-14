#!/usr/bin/env bash
# PoC de F1-4 (TASK-005): capa de operaciones Git deterministas y su contrato
# de salida para el agente.
#
# (A) Linea base de tokens: lee EN SOLO LECTURA las transcripciones de los
#     agentes de TASK-002..004 y cuenta, por tarea, las llamadas Bash/PowerShell
#     del agente que lanzan `git` o `gh` (lectura frente a escritura/red), los
#     bytes de su resultado y los turnos que quedan tras cada una (cada turno
#     relee ese resultado de la cache). Solo saca cifras agregadas y nombres de
#     subcomando; no copia contenido de las transcripciones.
# (B) Operaciones de la lista cerrada simuladas en bash (op_*), con entorno fijo
#     y una linea de contrato, sobre remotos bare locales con ruta absoluta:
#     B1 salida igual con LC_ALL=C y LC_ALL=es_ES e idempotencia de cada
#        operacion (segunda ejecucion sin cambios de refs, worktrees ni status);
#     B2 ciclo con conflicto provocado -> codigo 6, sin rebase/force/stash/reset
#        (grep del registro de comandos de las operaciones);
#     B3 bytes de la salida cruda de git frente a la linea de contrato.
#     El papel de la plataforma (merge de PR) lo hace un clon aparte; el puerto
#     Plataforma real (API HTTP) no se prueba aqui.
#
# Uso: poc-f1-4.sh <carpeta-de-trabajo> [carpeta-de-transcripciones]
# Sin GitHub ni red externa.
set -u
mkdir -p "$1" || exit 2
base="$(cd "$1" && pwd)"
if [ -n "$(ls -A "$base")" ] && [ ! -f "$base/.poc-f1-4" ]; then
  echo "ERROR: $base no esta vacia ni es de una ejecucion anterior de la PoC"; exit 2
fi
rm -rf "$base"; mkdir -p "$base"; : > "$base/.poc-f1-4"; cd "$base" || exit 2
RAIZ=$(pwd -W 2>/dev/null || pwd)   # ruta absoluta C:/... para git.exe
TRANS="${2:-$HOME/.claude/projects/C--Users-nullcad2025-Documents-sources-Personal-TaskCodeTwo}"
export GIT_AUTHOR_NAME=poc GIT_AUTHOR_EMAIL=poc@local GIT_COMMITTER_NAME=poc GIT_COMMITTER_EMAIL=poc@local
export GIT_AUTHOR_DATE="2026-11-14T10:00:00Z" GIT_COMMITTER_DATE="2026-11-14T10:00:00Z"   # hashes reproducibles
g() { git -c core.autocrlf=false -c init.defaultBranch=develop -c advice.detachedHead=false "$@"; }
sep() { echo; echo "=== $* ==="; }
resultado() { echo "RESULTADO $*"; echo "RESULTADO $*" >> "$RAIZ/resultados.txt"; }
sino() { if "$@"; then echo si; else echo NO; fi; }

######################################################################
# (A) Linea base de tokens a partir de las transcripciones
######################################################################
sep "A. Linea base: llamadas git/gh de los agentes en TASK-002..004 (solo lectura de transcripciones)"
echo "carpeta de transcripciones: $TRANS"
PY=$(command -v python || command -v python3)
"$PY" -I - "$TRANS" <<'EOF' | tee "$RAIZ/a.txt"
import json, os, re, sys, collections
raiz = sys.argv[1]
ses = 'f7ef1603-d233-4a21-a36b-4afb2c4b0c0e'
# Agentes de cada tarea (seccion ## Coste de tarea.md) y ventana de la sesion
# principal (de "tarea en diseno" de la primera a "tarea terminada", UTC).
tareas = {
  'TASK-002': (['aef3992f14554e6a0', 'a481e4ea76a12e0a3'], '2026-11-14T02:48:26', '2026-11-14T03:06:24'),
  'TASK-003': (['aef5cb6ada7ad952d', 'a6bb76b046a01c68d', 'a3509ae8439189572'], '2026-11-14T03:06:24', '2026-11-14T03:58:26'),
  'TASK-004': (['ab0cf0508f7832d0e', 'aa4dd1d805c5a1533', 'ad3141abac661826c', 'a9beee2def15847a6'], '2026-11-14T03:58:26', '2026-11-14T04:50:27'),
}
ESCRITURA = {'add','commit','push','fetch','pull','merge','rebase','reset','checkout','switch','tag','stash',
             'clone','ls-remote','cherry-pick','revert','restore','init','rm','mv','clean','am','apply','update-ref','gc'}
def subcmd(seg):
    t = seg.split()
    i = 0
    while i < len(t) and (re.match(r'^[A-Za-z_]+=', t[i]) or t[i] in ('rtk','command','env','(','{')):
        i += 1
    if i >= len(t): return None
    w = t[i].strip('(')
    if w not in ('git','git.exe','gh'): return None
    if w == 'gh': return ('gh', 'gh ' + (t[i+1] if i+1 < len(t) else ''), True)
    j = i + 1
    while j < len(t) and t[j].startswith('-'):
        j += 2 if t[j] in ('-C','-c','--git-dir','--work-tree') else 1
    s = t[j] if j < len(t) else ''
    resto = t[j+1:]
    esc = s in ESCRITURA
    if s == 'worktree': esc = bool(resto) and resto[0] in ('add','remove','prune','move')
    if s == 'branch': esc = any(a in ('-d','-D','-m','-M','-f','--delete','--move') for a in resto) or \
                           (len([a for a in resto if not a.startswith('-')]) > 0 and not any(a in ('--list','-l','-a','-r','--show-current','--contains','--merged','--format') or a.startswith('--format') for a in resto))
    if s == 'tag': esc = bool(resto) and not any(a in ('-l','--list','--contains','--points-at') for a in resto)
    if s == 'stash': esc = not (resto and resto[0] in ('list','show'))
    if s == 'remote': esc = bool(resto) and resto[0] in ('add','set-url','remove','rename','prune','update')
    if s == 'config': esc = len([a for a in resto if not a.startswith('-')]) >= 2
    return ('git', 'git ' + s, esc)
def limpia(cmd):
    # quita cuerpos de heredoc y texto entre comillas: un "git clean" dentro de
    # un mensaje de commit o de un fichero escrito no es una llamada a git
    cmd = re.sub(r"<<-?\s*['\"]?(\w+)['\"]?[^\n]*\n.*?\n\s*\1\s*(\n|$)", '\n', cmd, flags=re.S)
    cmd = re.sub(r'"(?:[^"\\]|\\.)*"', '""', cmd)
    cmd = re.sub(r"'[^']*'", "''", cmd)
    return cmd
def analiza_comando(cmd):
    cmd = limpia(cmd)
    segs = re.split(r'&&|\|\||;|\||\n|\$\(|`', cmd)
    res = [r for r in (subcmd(s.strip()) for s in segs) if r]
    return res
def texto(c):
    if isinstance(c, str): return c
    if isinstance(c, list):
        return ''.join(b.get('text','') if isinstance(b, dict) else str(b) for b in c)
    return ''
def lee(fichero, desde=None, hasta=None, solo_principal=False):
    """Devuelve (llamadas, turnos) de una transcripcion. llamadas: lista de dict."""
    msgs = []        # ids de mensaje del asistente en orden
    vistos = set()
    usos = {}        # tool_use_id -> (indice de turno, comando)
    resultados = {}  # tool_use_id -> bytes
    for linea in open(fichero, encoding='utf-8'):
        try: d = json.loads(linea)
        except Exception: continue
        if solo_principal and d.get('isSidechain'): continue
        ts = d.get('timestamp') or ''
        if desde and (ts[:19] < desde or ts[:19] > hasta): continue
        m = d.get('message') or {}
        c = m.get('content')
        if d.get('type') == 'assistant':
            mid = m.get('id')
            if mid and mid not in vistos:
                vistos.add(mid); msgs.append(mid)
            if isinstance(c, list):
                for b in c:
                    if isinstance(b, dict) and b.get('type') == 'tool_use' and b.get('name') in ('Bash','PowerShell'):
                        usos[b['id']] = (len(msgs) - 1, (b.get('input') or {}).get('command',''))
        elif d.get('type') == 'user' and isinstance(c, list):
            for b in c:
                if isinstance(b, dict) and b.get('type') == 'tool_result':
                    resultados[b.get('tool_use_id')] = len(texto(b.get('content')).encode('utf-8'))
    total = len(msgs)
    llamadas = []
    for tid, (idx, cmd) in usos.items():
        a = analiza_comando(cmd)
        tc = bool(re.search(r'(^|[\s;&|(])taskctl(\s|$)', cmd))
        if not a and not tc: continue
        llamadas.append({'sub': [x[1] for x in a], 'esc': any(x[2] for x in a), 'git': bool(a), 'taskctl': tc,
                         'pura': bool(a) and all(subcmd(s.strip()) for s in re.split(r'&&|\|\||;|\n', limpia(cmd)) if s.strip() and not s.strip().startswith('cd ')),
                         'bytes': resultados.get(tid, 0), 'resto': total - idx - 1})
    return llamadas, total
agregado = collections.Counter()
for t, (agentes, desde, hasta) in tareas.items():
    llamadas, turnos, falt = [], 0, []
    for a in agentes:
        f = os.path.join(raiz, ses, 'subagents', 'agent-%s.jsonl' % a)
        if not os.path.exists(f): falt.append(a); continue
        l, n = lee(f); llamadas += l; turnos += n
    fp = os.path.join(raiz, ses + '.jsonl')
    if os.path.exists(fp):
        l, n = lee(fp, desde, hasta, solo_principal=True); llamadas += l; turnos += n
    else:
        falt.append('sesion principal')
    gl = [x for x in llamadas if x['git']]
    lec = [x for x in gl if not x['esc']]; esc = [x for x in gl if x['esc']]
    pur = [x for x in gl if x['pura']]
    tcl = [x for x in llamadas if x['taskctl'] and not x['git']]
    by = sum(x['bytes'] for x in gl); byt = sum(x['bytes'] * x['resto'] for x in gl)
    sub = collections.Counter(s for x in gl for s in x['sub'])
    print('%s: agentes %d (faltan: %s) + sesion principal en [%sZ, %sZ]; turnos del asistente: %d' %
          (t, len(agentes), ', '.join(falt) or 'ninguno', desde, hasta, turnos))
    print('  llamadas con git/gh: %d (lectura %d, escritura/red %d; solo git %d, compuestas %d)' %
          (len(gl), len(lec), len(esc), len(pur), len(gl) - len(pur)))
    print('  bytes de su resultado: %d (lectura %d, escritura/red %d; solo git %d)' %
          (by, sum(x['bytes'] for x in lec), sum(x['bytes'] for x in esc), sum(x['bytes'] for x in pur)))
    print('  turnos restantes tras cada una: media %.1f, max %d ; bytes x turnos restantes: %d (~%d tokens releidos, a 4 bytes/token)' %
          ((sum(x['resto'] for x in gl) / len(gl)) if gl else 0, max([x['resto'] for x in gl] or [0]), byt, byt // 4))
    print('  subcomandos: ' + ', '.join('%s=%d' % kv for kv in sub.most_common()))
    print('  llamadas a taskctl (sin git en la misma linea): %d, bytes %d' % (len(tcl), sum(x['bytes'] for x in tcl)))
    agregado.update({'llamadas': len(gl), 'esc': len(esc), 'bytes': by, 'byt': byt, 'tc': len(tcl), 'tcb': sum(x['bytes'] for x in tcl)})
    print('RESULTADO A: %s llamadas git/gh=%d escritura/red=%d bytes=%d bytes*turnos=%d' % (t, len(gl), len(esc), by, byt))
print('RESULTADO A: total 3 tareas llamadas git/gh=%d escritura/red=%d bytes=%d bytes*turnos=%d taskctl=%d (bytes %d)' %
      (agregado['llamadas'], agregado['esc'], agregado['bytes'], agregado['byt'], agregado['tc'], agregado['tcb']))
EOF
grep '^RESULTADO A' "$RAIZ/a.txt" >> "$RAIZ/resultados.txt"

######################################################################
# (B) Capa de operaciones simulada
######################################################################
# gop: git con entorno fijo, registrado en el log de comandos de la fixture.
# Formato del registro: <op> [<carpeta>] rama=<rama actual> git <args>
gop() {
  local rama
  rama=$(git symbolic-ref -q --short HEAD 2>/dev/null || echo -)
  printf '%s [%s] rama=%s git %s\n' "$OP" "${PWD#"$FIX"/}" "$rama" "$*" >> "$FIX/comandos-ops.log"
  env LC_ALL=C LANG=C LANGUAGE= GIT_TERMINAL_PROMPT=0 GIT_PAGER=cat GIT_EDITOR=true GIT_ASKPASS= \
    git -c color.ui=never -c core.autocrlf=false -c core.quotepath=false -c advice.detachedHead=false "$@"
}
linea() { echo "$*"; }                       # la linea de contrato (stdout)
diag() { echo "$*" >&2; }                    # diagnostico (stderr, solo si falla)
corto() { cut -c1-12; }
num_tarea() { local n=${1#TASK-}; echo "$n"; }
base_de() { case "$1" in hotfix/*) echo origin/main ;; *) echo origin/develop ;; esac; }
principal() { git worktree list --porcelain | sed -n '1s/^worktree //p'; }
rel() { local p=$1; echo "${p#"$FIX"/}"; }

op_fetch() { OP=fetch
  local antes despues
  antes=$(gop for-each-ref --format='%(refname) %(objectname)' refs/remotes refs/tags)
  if ! gop fetch -q --prune --tags origin 2>/dev/null; then
    linea "fetch sinred remoto=origin"; diag "fetch: no se pudo leer origin"; return 4
  fi
  despues=$(gop for-each-ref --format='%(refname) %(objectname)' refs/remotes refs/tags)
  if [ "$antes" = "$despues" ]; then linea "fetch sin-cambios"; else
    linea "fetch ok refs=$(diff <(echo "$antes") <(echo "$despues") | grep -c '^>')"; fi
}
op_validar_base() { OP=validar-base   # validar-base <develop|main|hotfix>
  local ref
  case "$1" in develop) ref=origin/develop ;; main) ref=origin/main ;;
    hotfix) ref=$(gop describe --tags --abbrev=0 origin/main 2>/dev/null || echo origin/main) ;; *) ref=$1 ;; esac
  if ! gop rev-parse -q --verify "$ref^{commit}" >/dev/null; then
    linea "validar-base rechazada base=$ref motivo=no-existe"; return 3; fi
  linea "validar-base ok base=$ref commit=$(gop rev-parse "$ref^{commit}" | corto)"
}
op_comprobar_id() { OP=comprobar-id   # comprobar-id TASK-NNN
  local n remotas locales
  n=$(num_tarea "$1")
  remotas=$(gop ls-remote --heads origin "*/task-$n-*" 2>/dev/null) || { linea "comprobar-id sinred id=$1"; return 4; }
  remotas=$(echo "$remotas" | awk '{print $2}' | sed 's#^refs/heads/##')
  locales=$(gop branch --list "*/task-$n-*" --format='%(refname:short)')
  local todas; todas=$(printf '%s\n%s\n' "$remotas" "$locales" | sed '/^$/d' | sort -u | paste -sd, -)
  if [ -z "$todas" ]; then linea "comprobar-id libre id=$1"; return 0; fi
  linea "comprobar-id ocupada id=$1 ramas=$todas"; return 3
}
op_crear_rama_y_worktree() { OP=crear-rama-y-worktree   # <id> <tipo> <slug> <base-ref>
  local id=$1 rama wt pr
  rama="$2/task-$(num_tarea "$id")-$3"; pr=$(principal); wt="$pr/.taskcode/worktrees/$id"
  if [ -d "$wt" ] && [ "$(git -C "$wt" symbolic-ref -q --short HEAD)" = "$rama" ]; then
    linea "crear-rama-y-worktree sin-cambios rama=$rama worktree=.taskcode/worktrees/$id"; return 0; fi
  if gop show-ref -q --verify "refs/heads/$rama" || gop show-ref -q --verify "refs/remotes/origin/$rama"; then
    linea "crear-rama-y-worktree rechazada rama=$rama motivo=rama-existe"; return 3; fi
  gop worktree add -q --no-track -b "$rama" "$wt" "$4" 2>/dev/null || { diag "worktree add fallo"; linea "crear-rama-y-worktree error"; return 1; }
  linea "crear-rama-y-worktree ok rama=$rama worktree=.taskcode/worktrees/$id base=$(gop rev-parse "$4" | corto)"
}
op_reanudar() { OP=reanudar   # reanudar <id>: recrea rama local y worktree desde origin
  local id=$1 n rama wt pr hecho=""
  n=$(num_tarea "$id"); pr=$(principal); wt="$pr/.taskcode/worktrees/$id"
  rama=$(gop for-each-ref --format='%(refname:short)' "refs/heads/*/task-$n-*" | head -1)
  [ -z "$rama" ] && rama=$(gop for-each-ref --format='%(refname:lstrip=3)' "refs/remotes/origin/*/task-$n-*" | head -1)
  [ -z "$rama" ] && { linea "reanudar rechazada id=$id motivo=sin-rama"; return 3; }
  if [ -d "$wt" ] && [ "$(git -C "$wt" symbolic-ref -q --short HEAD)" = "$rama" ]; then
    linea "reanudar sin-cambios rama=$rama worktree=.taskcode/worktrees/$id"; return 0; fi
  if ! gop show-ref -q --verify "refs/heads/$rama"; then
    gop branch -q --no-track "$rama" "origin/$rama" && hecho="rama"; fi
  gop worktree prune
  gop worktree add -q "$wt" "$rama" 2>/dev/null && hecho="${hecho:+$hecho,}worktree"
  linea "reanudar ok rama=$rama worktree=.taskcode/worktrees/$id creado=$hecho"
}
op_guardar() { OP=guardar   # guardar "<mensaje>" <rutas...> (en el worktree de la tarea; nunca add -A)
  local msg=$1; shift
  [ $# -gt 0 ] || { linea "guardar uso motivo=sin-rutas"; diag "uso: guardar \"<mensaje>\" <rutas...>"; return 2; }
  if [ -f "$(git rev-parse --git-dir)/MERGE_HEAD" ]; then
    local pend otras p
    pend=$(gop diff --name-only --diff-filter=U)
    otras=$pend; for p in "$@"; do otras=$(echo "$otras" | grep -vxF "$p"); done
    if [ -n "$otras" ]; then
      linea "guardar rechazada motivo=conflicto-sin-resolver ficheros=$(echo "$otras" | paste -sd, -)"; return 3; fi
    if grep -lE '^(<<<<<<<|>>>>>>>) ' -- "$@" >/dev/null 2>&1; then
      linea "guardar rechazada motivo=marcas-de-conflicto ficheros=$(grep -lE '^(<<<<<<<|>>>>>>>) ' -- "$@" | paste -sd, -)"; return 3; fi
    gop add -- "$@" && gop commit -q --no-edit -m "$msg" || { linea "guardar error"; return 1; }
    linea "guardar ok commit=$(gop rev-parse HEAD | corto) tipo=merge ficheros=$#"; return 0
  fi
  if [ -z "$(gop status --porcelain -- "$@")" ]; then linea "guardar sin-cambios"; return 0; fi
  gop add -- "$@" && gop commit -q -m "$msg" -- "$@" || { linea "guardar error"; return 1; }
  local resto; resto=$(gop status --porcelain | wc -l | tr -d ' ')
  linea "guardar ok commit=$(gop rev-parse HEAD | corto) ficheros=$# sin-guardar=$resto"
}
op_push() { OP=push   # push de la rama actual, con reintento, sin force
  local rama i salida
  rama=$(git symbolic-ref -q --short HEAD)
  if [ "$(gop rev-parse -q --verify "refs/remotes/origin/$rama")" = "$(gop rev-parse HEAD)" ]; then
    linea "push sin-cambios rama=$rama"; return 0; fi
  for i in 1 2 3; do
    if salida=$(gop push -q origin "$rama" 2>&1); then
      linea "push ok rama=$rama commit=$(gop rev-parse HEAD | corto) intentos=$i"; return 0; fi
    if echo "$salida" | grep -q 'non-fast-forward\|fetch first'; then
      linea "push rechazada rama=$rama motivo=no-fast-forward"; diag "lanza actualizar-rama"; return 3; fi
  done
  linea "push sinred rama=$rama"; return 4
}
op_actualizar_rama() { OP=actualizar-rama   # merge de la base en la rama de la tarea (nunca rebase)
  local rama b
  rama=$(git symbolic-ref -q --short HEAD); b=$(base_de "$rama")
  if [ -f "$(git rev-parse --git-dir)/MERGE_HEAD" ]; then
    linea "actualizar-rama conflicto base=$b ficheros=$(gop diff --name-only --diff-filter=U | paste -sd, -)"; return 6; fi
  if gop merge-base --is-ancestor "$b" HEAD; then linea "actualizar-rama sin-cambios base=$b"; return 0; fi
  if gop merge -q --no-ff --no-edit "$b" >/dev/null 2>&1; then
    linea "actualizar-rama ok base=$b commit=$(gop rev-parse HEAD | corto)"; return 0; fi
  local u; u=$(gop diff --name-only --diff-filter=U | paste -sd, -)
  if [ -n "$u" ]; then
    linea "actualizar-rama conflicto base=$b ficheros=$u"; diag "resuelve $u y lanza guardar"; return 6; fi
  linea "actualizar-rama error"; return 1
}
op_diff_tarea() { OP=diff-tarea   # resumen del diff de la tarea contra su base (la base la calcula el CLI)
  local rama b mb st
  rama=$(git symbolic-ref -q --short HEAD); b=$(base_de "$rama")
  mb=$(gop merge-base "$b" HEAD)
  st=$(gop diff --numstat "$mb" HEAD | awk '{f++; a+=$1; d+=$2} END {printf "ficheros=%d lineas=+%d,-%d", f, a, d}')
  linea "diff-tarea ok base=$(echo "$mb" | corto) $st"
}
op_localizar_principal() { OP=localizar-principal
  local p; p=$(gop worktree list --porcelain | sed -n '1s/^worktree //p')
  linea "localizar-principal ok ruta=$(rel "$p")"
}
op_quitar_worktree() { OP=quitar-worktree   # quitar-worktree <id>: la rama se conserva
  local id=$1 pr wt rama
  pr=$(principal); wt="$pr/.taskcode/worktrees/$id"
  [ -d "$wt" ] || { linea "quitar-worktree sin-cambios id=$id"; return 0; }
  rama=$(git -C "$wt" symbolic-ref -q --short HEAD)
  if [ -n "$(git -C "$wt" status --porcelain)" ]; then linea "quitar-worktree rechazada id=$id motivo=cambios-sin-guardar"; return 3; fi
  if [ "$(gop rev-parse -q --verify "refs/remotes/origin/$rama")" != "$(gop rev-parse "refs/heads/$rama")" ]; then
    linea "quitar-worktree rechazada id=$id motivo=sin-publicar"; return 3; fi
  gop worktree remove "$wt" || { linea "quitar-worktree error"; return 1; }
  linea "quitar-worktree ok id=$id rama=$rama conservada=si"
}
op_actualizar_principal() { OP=actualizar-principal   # pull --ff-only de develop en el principal
  local antes
  [ "$(git symbolic-ref -q --short HEAD)" = develop ] || { linea "actualizar-principal rechazada motivo=principal-no-en-develop"; return 3; }
  antes=$(gop rev-parse HEAD)
  gop pull -q --ff-only origin develop 2>/dev/null || { linea "actualizar-principal rechazada motivo=no-fast-forward"; return 3; }
  if [ "$antes" = "$(gop rev-parse HEAD)" ]; then linea "actualizar-principal sin-cambios commit=$(echo "$antes" | corto)"; else
    linea "actualizar-principal ok commit=$(gop rev-parse HEAD | corto)"; fi
}
op_tag_anotado() { OP=tag-anotado   # tag-anotado <vX.Y.Z> <ref>
  local t=$1 c remoto
  c=$(gop rev-parse "$2^{commit}")
  remoto=$(gop ls-remote --tags origin "refs/tags/$t^{}" | awk '{print $1}')
  if [ -n "$remoto" ]; then
    if [ "$remoto" = "$c" ]; then linea "tag-anotado sin-cambios tag=$t commit=$(echo "$c" | corto)"; return 0; fi
    linea "tag-anotado rechazada tag=$t motivo=existe-en-otro-commit"; return 3
  fi
  gop rev-parse -q --verify "refs/tags/$t" >/dev/null || gop tag -a "$t" -m "Release $t" "$c"
  gop push -q origin "refs/tags/$t" 2>/dev/null || { linea "tag-anotado sinred tag=$t"; return 4; }
  linea "tag-anotado ok tag=$t commit=$(echo "$c" | corto)"
}
op_notas_release() { OP=notas-release   # notas-release <tag> [<tag-anterior>]: notas a un fichero
  local t=$1 prev=${2:-} rango f n=0 h p1 p2 resto
  rango="$t^2"; [ -n "$prev" ] && rango="$prev..$t^2"
  f="$(git rev-parse --git-common-dir)/taskcode/notas/$t.md"; mkdir -p "$(dirname "$f")"
  : > "$f"
  while read -r h p1 p2 resto; do
    [ -z "$h" ] && continue
    if [ -n "$prev" ] && gop merge-base --is-ancestor "$p2" "$prev"; then continue; fi
    gop log -1 --format='- %s' "$h" >> "$f"; n=$((n + 1))
  done < <(gop log --merges --first-parent --format='%H %P' "$rango")
  linea "notas-release ok tag=$t prs=$n fichero=.git/taskcode/notas/$t.md"
}
op_commit_estado() { OP=commit-estado   # commit-estado <id> <estado>: el punto unico de F1-2, simplificado
  local id=$1 est=$2 t i
  t="$(principal)/tareas"
  for i in 1 2 3 4 5; do
    ( cd "$t" || exit 1
      gop fetch -q origin taskcode/tareas 2>/dev/null || exit 4
      gop merge -q --ff-only origin/taskcode/tareas >/dev/null || exit 1
      if grep -qx "estado: $est" "$id/tarea.md" 2>/dev/null; then exit 10; fi
      sed -i "s/^estado: .*/estado: $est/" "$id/tarea.md"
      gop add -- "$id/tarea.md" && gop commit -q -m "estado: $id $est" || exit 1
      gop push -q origin taskcode/tareas 2>/dev/null && exit 0
      gop reset -q --hard origin/taskcode/tareas   # F1-2: descartar el commit propio y re-ejecutar
      exit 5 )
    case $? in
      0) linea "commit-estado ok id=$id estado=$est intentos=$i"; return 0 ;;
      10) linea "commit-estado sin-cambios id=$id estado=$est"; return 0 ;;
      4) linea "commit-estado sinred id=$id pendiente=no"; return 4 ;;
      5) continue ;;
      *) linea "commit-estado error"; return 1 ;;
    esac
  done
  linea "commit-estado agotada id=$id"; return 5
}

######################## fixtures ########################
semilla() { # semilla <carpeta>: origin.git, principal (develop + tareas/), plataforma
  local F=$1
  mkdir -p "$F"
  g init -q --bare "$F/origin.git"; g -C "$F/origin.git" config receive.denyNonFastForwards true
  g clone -q "$F/origin.git" "$F/principal" 2>/dev/null
  printf '/tareas/\n/.taskcode/worktrees/\n' > "$F/principal/.gitignore"
  printf 'linea 1\nlinea 2\nlinea 3\n' > "$F/principal/f.txt"
  g -C "$F/principal" add .gitignore f.txt && g -C "$F/principal" commit -q -m inicial
  g -C "$F/principal" branch main && g -C "$F/principal" push -q origin develop main 2>/dev/null
  g -C "$F/principal" worktree add -q --orphan -b taskcode/tareas "$F/principal/tareas" 2>/dev/null
  for id in TASK-007 TASK-008; do mkdir -p "$F/principal/tareas/$id"
    printf -- '---\nid: %s\nestado: en-curso\n---\n' "$id" > "$F/principal/tareas/$id/tarea.md"; done
  g -C "$F/principal/tareas" add TASK-007 TASK-008 && g -C "$F/principal/tareas" commit -q -m "estado: altas"
  g -C "$F/principal/tareas" push -q -u origin taskcode/tareas 2>/dev/null
  g clone -q "$F/origin.git" "$F/plataforma" 2>/dev/null
  : > "$F/comandos-ops.log"
}
plat() { echo "$*" >> "$FIX/comandos-plataforma.log"; g -C "$FIX/plataforma" "$@"; }
plat_commit_develop() { # plat_commit_develop <fichero> <contenido> <msg>: otro PR ya integrado en develop
  plat fetch -q origin; plat checkout -q -B develop origin/develop
  printf '%s\n' "$2" > "$FIX/plataforma/$1"; plat add -- "$1"; plat commit -q -m "$3"; plat push -q origin develop 2>/dev/null
}
plat_merge_pr() { # plat_merge_pr <base> <rama>: el boton de merge (merge commit sin ff)
  plat fetch -q origin; plat checkout -q -B "$1" "origin/$1"
  plat merge -q --no-ff --no-edit -m "Merge pull request from $2" "origin/$2" >/dev/null && plat push -q origin "$1" 2>/dev/null
}
snap() { # huella de refs (clon y remoto), worktrees y status
  { g -C "$FIX/principal" for-each-ref --format='%(refname) %(objectname)'
    g -C "$FIX/origin.git" for-each-ref --format='O %(refname) %(objectname)'
    g -C "$FIX/principal" worktree list --porcelain
    for w in "$FIX/principal" "$FIX/principal/tareas" "$FIX"/principal/.taskcode/worktrees/*; do
      [ -d "$w" ] || continue; echo "== $w"; g -C "$w" status --porcelain; g -C "$w" rev-parse HEAD; done
    ls "$FIX/principal/.git/taskcode/notas" 2>/dev/null; } | md5sum | cut -c1-12
}
NOK=0; NMAL=0; NCONTRATO=0
contrato_ok() { # <op> <salida>: <=5 lineas, sin ANSI, primera linea "<op> <resultado> clave=valor..."
  local n; n=$(printf '%s\n' "$2" | wc -l)
  [ "$n" -le 5 ] && ! printf '%s' "$2" | grep -q $'\e' && \
    printf '%s\n' "$2" | head -1 | grep -qE "^$1 [a-z-]+( [a-z-]+=[^ ]+)*$"
}
paso() { # paso <dir-relativo> <op> args...: dos ejecuciones; idempotencia y contrato
  local d=$1 op=$2; shift 2
  local o1 o2 c1 c2 s1 s2 nom=${op//_/-}
  o1=$(cd "$FIX/$d" && LC_ALL=$LOC op_"$op" "$@" 2>"$FIX/err"); c1=$?
  s1=$(snap)
  o2=$(cd "$FIX/$d" && LC_ALL=$LOC op_"$op" "$@" 2>/dev/null); c2=$?
  s2=$(snap)
  local idem=si; [ "$s1" = "$s2" ] || idem=NO
  local seg=si; [ "$o1" = "$o2" ] || echo "$o2" | grep -q " sin-cambios" || seg=NO
  contrato_ok "$nom" "$o1" && contrato_ok "$nom" "$o2" || NCONTRATO=$((NCONTRATO + 1))
  if [ $idem = si ] && [ $seg = si ]; then NOK=$((NOK + 1)); else NMAL=$((NMAL + 1)); fi
  echo "\$ taskctl $nom $*   [en ${d}]"
  echo "  1a: $o1   (codigo $c1, $(printf '%s\n' "$o1" | wc -c) bytes)"
  [ -s "$FIX/err" ] && echo "      stderr: $(head -1 "$FIX/err")"
  echo "  2a: $o2   (codigo $c2) ; estado igual tras la 2a: $idem ; 2a sin-cambios o igual: $seg"
  printf '%s | %s | %s | %s | %s\n' "$nom $*" "$o1" "$c1" "$o2" "$c2" >> "$FIX/salidas.txt"
  return "$c1"
}
W7=principal/.taskcode/worktrees/TASK-007
W8=principal/.taskcode/worktrees/TASK-008
escenario_b1() { # la misma secuencia en cada fixture; LOC es el LC_ALL de quien llama
  paso principal fetch
  paso principal validar_base develop
  paso principal comprobar_id TASK-007
  paso principal crear_rama_y_worktree TASK-007 feature login origin/develop
  paso "$W7" localizar_principal
  printf 'login\n' > "$FIX/$W7/g.txt"
  paso "$W7" guardar "feat(TASK-007): login" g.txt
  paso "$W7" push
  paso principal comprobar_id TASK-007
  plat_commit_develop h.txt "otro cambio" "Merge pull request from feature/task-006-otro"
  paso principal fetch
  paso "$W7" actualizar_rama
  paso "$W7" push
  paso "$W7" diff_tarea
  echo "-- se pierden worktree y rama local de TASK-007 (simulado por el guion, fuera del registro de operaciones)"
  rm -rf "$FIX/$W7"; g -C "$FIX/principal" worktree prune; g -C "$FIX/principal" branch -q -D feature/task-007-login
  paso principal reanudar TASK-007
  plat_merge_pr develop feature/task-007-login
  plat_merge_pr main develop
  paso principal fetch
  paso principal quitar_worktree TASK-007
  paso principal actualizar_principal
  paso principal tag_anotado v0.1.0 origin/main
  paso principal notas_release v0.1.0
  paso principal commit_estado TASK-007 terminada
  paso principal validar_base hotfix
  echo "-- sin red: origin apunta a una ruta inexistente (cambio del guion)"
  g -C "$FIX/principal" remote set-url origin "$FIX/no-existe.git"
  paso principal fetch
  paso principal comprobar_id TASK-009
  g -C "$FIX/principal" remote set-url origin "$FIX/origin.git"
}

sep "B0. Entorno"
echo "git: $(git --version)"
echo "LC_ALL=es_ES.UTF-8 git status (crudo, en este repo de prueba) frente a LC_ALL=C:"
semilla "$RAIZ/R"
es=$(cd "$RAIZ/R/principal" && LC_ALL=es_ES.UTF-8 LANG=es_ES.UTF-8 git status 2>&1); c=$(cd "$RAIZ/R/principal" && LC_ALL=C git status 2>&1)
echo "  es_ES: $(echo "$es" | head -1)"; echo "  C    : $(echo "$c" | head -1)"
echo "  salida cruda distinta entre es_ES y C: $([ "$es" = "$c" ] && echo 'no (este Git for Windows no trae catalogos de traduccion)' || echo si)"
ls "$(git --exec-path)/../../share/locale" >/dev/null 2>&1 && echo "  catalogos: si" || echo "  catalogos de traduccion de git instalados: no"

sep "B1. Operaciones de la lista cerrada: caller con LC_ALL=C (fixture C)"
FIX="$RAIZ/C"; LOC=C; semilla "$FIX"; escenario_b1
OK_C=$NOK; MAL_C=$NMAL
sep "B1. La misma secuencia con LC_ALL=es_ES.UTF-8 (fixture ES)"
FIX="$RAIZ/ES"; LOC=es_ES.UTF-8; NOK=0; NMAL=0; semilla "$FIX"; escenario_b1 > "$RAIZ/b1-es.txt"
OK_ES=$NOK; MAL_ES=$NMAL
echo "(salida completa igual a la de la fixture C salvo el LC_ALL; comparacion abajo)"
if diff -q "$RAIZ/C/salidas.txt" "$RAIZ/ES/salidas.txt" >/dev/null; then dif=0; else dif=$(diff "$RAIZ/C/salidas.txt" "$RAIZ/ES/salidas.txt" | grep -c '^<'); fi
echo "operaciones ejecutadas por fixture: $(wc -l < "$RAIZ/C/salidas.txt") (cada una dos veces)"
echo "lineas de salida distintas entre LC_ALL=C y LC_ALL=es_ES.UTF-8: $dif"
resultado "B1: operaciones idempotentes (2a ejecucion sin cambios y sin-cambios o igual): C $OK_C ok / $MAL_C mal ; es_ES $OK_ES ok / $MAL_ES mal"
resultado "B1: salidas distintas entre LC_ALL=C y LC_ALL=es_ES.UTF-8: $dif de $(wc -l < "$RAIZ/C/salidas.txt")"
resultado "B1: salidas que incumplen el contrato (<=5 lineas, sin ANSI, 'op resultado clave=valor'): $NCONTRATO"
codigos=$(cut -d'|' -f3 "$RAIZ/C/salidas.txt" | tr -d ' ' | sort | uniq -c | awk '{printf "%s:%s ", $2, $1}')
bytes_ciclo=$(cut -d'|' -f2 "$RAIZ/C/salidas.txt" | sed 's/^ //; s/ $//' | awk '{b+=length($0)+1} END {print b}')
resultado "B1: bytes de las lineas de contrato de las $(wc -l < "$RAIZ/C/salidas.txt") operaciones del ciclo (1a ejecucion): $bytes_ciclo"
resultado "B1: codigos de salida observados (codigo:veces): $codigos"

sep "B2. Ciclo con conflicto provocado (fixture K, caller LC_ALL=es_ES.UTF-8)"
FIX="$RAIZ/K"; LOC=es_ES.UTF-8; NOK=0; NMAL=0; NCONTRATO=0; semilla "$FIX"
paso principal fetch
paso principal crear_rama_y_worktree TASK-008 fix crash origin/develop
sed -i 's/^linea 1$/linea 1 de TASK-008/' "$FIX/$W8/f.txt"
paso "$W8" guardar "fix(TASK-008): crash" f.txt
paso "$W8" push
echo "-- la plataforma integra en develop otro PR que toca la misma linea"
plat fetch -q origin; plat checkout -q -B develop origin/develop
sed -i 's/^linea 1$/linea 1 de otro PR/' "$FIX/plataforma/f.txt"; plat commit -q -am "Merge pull request from feature/task-009-otro"; plat push -q origin develop 2>/dev/null
paso principal fetch
paso "$W8" actualizar_rama; cconf=$?
echo "estado del worktree: MERGE_HEAD=$([ -f "$FIX/principal/.git/worktrees/TASK-008/MERGE_HEAD" ] && echo si || echo no) ; status: $(g -C "$FIX/$W8" status --porcelain | tr '\n' ' ')"
printf 'nota\n' > "$FIX/$W8/nota.txt"
paso "$W8" guardar "otra cosa" nota.txt; cotra=$?
paso "$W8" guardar "resolver" f.txt; cmarcas=$?
printf 'linea 1 de TASK-008 y de otro PR\nlinea 2\nlinea 3\n' > "$FIX/$W8/f.txt"
echo "-- el agente resuelve f.txt (lo hace el guion) y lanza guardar"
paso "$W8" guardar "merge(TASK-008): develop en la rama" f.txt
echo "padres del commit guardado: $(g -C "$FIX/$W8" log -1 --format=%P | wc -w)"
rm -f "$FIX/$W8/nota.txt"
paso "$W8" push
paso "$W8" actualizar_rama
plat_merge_pr develop fix/task-008-crash
paso principal fetch
paso principal quitar_worktree TASK-008
paso principal actualizar_principal
echo "f.txt en develop: $(g -C "$FIX/principal" show HEAD:f.txt | head -1)"
resultado "B2: actualizar-rama con conflicto -> codigo $cconf (esperado 6); guardar de otra ruta con el conflicto abierto -> $cotra (3); guardar con marcas -> $cmarcas (3)"
resultado "B2: pasos idempotentes $NOK ok / $NMAL mal ; salidas fuera de contrato: $NCONTRATO"
echo "-- registro de comandos de las operaciones (todas las fixtures): $(cat "$RAIZ"/{C,ES,K}/comandos-ops.log | wc -l) llamadas a git"
PROHIB=' git (rebase|stash|clean)|git push .*(--force|-f( |$)|--delete| :)|git reset|git branch .*(-d|-D|--delete)( |$)'
# control del patron: debe casar las 7 primeras lineas y ninguna de las 3 ultimas
control=$(printf '%s\n' 'x [w] rama=f git rebase origin/develop' 'x [w] rama=f git push --force origin f' 'x [w] rama=f git push -f origin f' \
  'x [w] rama=f git stash' 'x [w] rama=f git reset --hard HEAD' 'x [w] rama=f git branch -D f' 'x [w] rama=f git push origin --delete f' \
  'x [w] rama=f git push -q origin f' 'x [w] rama=f git merge -q --no-ff --no-edit origin/develop' 'x [w] rama=f git branch -q --no-track f origin/f' | grep -cE "$PROHIB")
echo "control del patron de prohibidos: casa $control de 7 lineas prohibidas y 0 de 3 permitidas (esperado 7)"
prohib=$(cat "$RAIZ"/{C,ES,K}/comandos-ops.log | grep -cE "$PROHIB")
mergeprinc=$(cat "$RAIZ"/{C,ES,K}/comandos-ops.log | grep -cE 'rama=(develop|main) git merge')
excep=$(cat "$RAIZ"/{C,ES,K}/comandos-ops.log | grep -cE '^commit-estado .*git reset')
echo "subcomandos usados por las operaciones: $(cat "$RAIZ"/{C,ES,K}/comandos-ops.log | sed -E 's/.* git ([a-z-]+).*/\1/' | sort | uniq -c | awk '{printf "%s=%s ", $2, $1}')"
resultado "B2: comandos prohibidos en el registro (rebase, stash, clean, reset, push --force/--delete, branch -d/-D): $prohib ; merge con el principal en develop/main: $mergeprinc ; reset de commit-estado ejecutado: $excep"

sep "B3. Bytes: salida cruda de git frente a la linea de contrato (fixture R, sin -q y con el entorno por defecto)"
FIX="$RAIZ/R"
gc() { git -c core.autocrlf=false "$@"; }   # crudo: sin -q y con el locale de quien llama
cd "$FIX/principal" || exit 2
crudo_wt=$(gc worktree add -b feature/task-007-login .taskcode/worktrees/TASK-007 origin/develop 2>&1)
printf 'login\n' > .taskcode/worktrees/TASK-007/g.txt
gc -C .taskcode/worktrees/TASK-007 add g.txt ; gc -C .taskcode/worktrees/TASK-007 commit -q -m login
crudo_push=$(gc -C .taskcode/worktrees/TASK-007 push -u origin feature/task-007-login 2>&1)
plat_commit_develop h.txt "otro cambio" "Merge pull request from feature/task-006-otro"
crudo_fetch=$(gc fetch origin 2>&1)
crudo_merge=$(gc -C .taskcode/worktrees/TASK-007 merge --no-edit origin/develop 2>&1)
crudo_status=$(gc -C .taskcode/worktrees/TASK-007 status 2>&1)
cd "$RAIZ" || exit 2
contrato() { grep "^$1" "$RAIZ/C/salidas.txt" | head -1 | cut -d'|' -f2 | sed 's/^ //; s/ $//'; }
cmp_bytes() { # <nombre> <crudo> <linea de contrato>
  local bc bl; bc=$(printf '%s\n' "$2" | wc -c); bl=$(printf '%s\n' "$3" | wc -c)
  printf '%-22s crudo %4d bytes / %2d lineas ; contrato %3d bytes / 1 linea ; %s\n' "$1" "$bc" "$(printf '%s\n' "$2" | wc -l)" "$bl" "$3"
  echo "$bc $bl" >> "$RAIZ/bytes.txt"
}
: > "$RAIZ/bytes.txt"
cmp_bytes "worktree add" "$crudo_wt" "$(contrato 'crear-rama-y-worktree TASK-007')"
cmp_bytes "push" "$crudo_push" "$(grep '^push ' "$RAIZ/C/salidas.txt" | head -1 | cut -d'|' -f2 | sed 's/^ //; s/ $//')"
cmp_bytes "fetch" "$crudo_fetch" "$(grep '^fetch ' "$RAIZ/C/salidas.txt" | sed -n 2p | cut -d'|' -f2 | sed 's/^ //; s/ $//')"
cmp_bytes "merge (actualizar)" "$crudo_merge" "$(contrato 'actualizar-rama')"
cmp_bytes "status / diff-tarea" "$crudo_status" "$(contrato 'diff-tarea')"
echo "salida cruda de push (rutas acortadas):"; printf '%s\n' "$crudo_push" | sed "s#$RAIZ#<poc>#g; s/^/  | /"
tot=$(awk '{c+=$1; l+=$2} END {printf "crudo %d bytes, contrato %d bytes (%.0f%% menos)", c, l, 100*(c-l)/c}' "$RAIZ/bytes.txt")
resultado "B3: 5 operaciones: $tot"

sep "Resumen"
cat "$RAIZ/resultados.txt"

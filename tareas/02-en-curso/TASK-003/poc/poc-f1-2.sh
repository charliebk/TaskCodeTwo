#!/usr/bin/env bash
# PoC de F1-2 (TASK-003): escrituras atomicas en la rama de estado
# taskcode/tareas, reserva de IDs y toma de tareas.
#
# Simula el punto unico de escritura de la 2.0 (funcion tx) con el protocolo
# del ADR: cerrojo local, fetch, operacion sobre el estado fresco, commit,
# push sin force; si el push se rechaza (non-fast-forward), se descarta el
# commit y se RE-EJECUTA la operacion, hasta MAX_INTENTOS.
#
# Uso: poc-f1-2.sh <carpeta-de-trabajo>   (vacia o de una ejecucion anterior)
# Solo remotos bare locales con rutas absolutas: sin GitHub ni red externa.
set -u
mkdir -p "$1" || exit 2
base="$(cd "$1" && pwd)"
if [ -n "$(ls -A "$base")" ] && [ ! -f "$base/.poc-f1-2" ]; then
  echo "ERROR: $base no esta vacia ni es de una ejecucion anterior de la PoC"; exit 2
fi
rm -rf "$base"; mkdir -p "$base"; : > "$base/.poc-f1-2"; cd "$base" || exit 2
RAIZ=$base
nueva_prueba() { base="$RAIZ/$1"; mkdir -p "$base"; cd "$base" || exit 2; }   # cada prueba en su carpeta
export GIT_AUTHOR_NAME=poc GIT_AUTHOR_EMAIL=poc@local GIT_COMMITTER_NAME=poc GIT_COMMITTER_EMAIL=poc@local
g() { git -c core.autocrlf=false -c init.defaultBranch=develop -c advice.detachedHead=false "$@"; }
sep() { echo; echo "=== $* ==="; }
resultado() { echo "RESULTADO $*"; echo "RESULTADO $*" >> "$RAIZ/resultados.txt"; }

# Codigos de salida de una transaccion (valores propuestos por el ADR)
RC_PUBLICADA=0 RC_RECHAZADA=3 RC_SINRED=4 RC_AGOTADA=5
MAX_INTENTOS=${MAX_INTENTOS:-5}
ESPERA_BASE=${ESPERA_BASE:-2}   # segundos, del orden de una transaccion; ver prueba 1b
nombre_rc() { case $1 in 0) echo Publicada;; 3) echo Rechazada;; 4) echo SinRed;; 5) echo Agotada;; *) echo "Error($1)";; esac; }
# Operaciones que reservan algo compartido: exigen red (D2b)
es_compartida() { case $1 in alta|import|start|liberar|reasignar) return 0;; *) return 1;; esac; }

######################## estado de una tarea y roadmap ########################
tarea() { # tarea <dir> <id> <titulo> <estado> <asignado>
  mkdir -p "$1/$2"
  printf -- '---\nid: %s\ntitulo: %s\nestado: %s\nasignado: %s\n---\n' "$2" "$3" "$4" "$5" > "$1/$2/tarea.md"
}
campo() { sed -n "s/^$2: \{0,1\}//p" "$1"; }   # campo <fichero> <nombre>
roadmap() { # regenera ROADMAP.md en el directorio actual (nunca se mergea)
  { echo "# Roadmap"
    for f in TASK-*/tarea.md; do [ -f "$f" ] || continue
      a=$(campo "$f" asignado)
      echo "- $(campo "$f" id): $(campo "$f" estado)${a:+ (lo lleva $a)} - $(campo "$f" titulo)"
    done; } > ROADMAP.md
}

######################## operaciones: estado -> cambios | rechazo ########################
# Devuelven 0 = hay cambios, 1 = rechazo (MOTIVO), 2 = sin cambios (idempotente)
siguiente_id() { # maximo de las carpetas TASK-* + 1 (las carpetas no se borran nunca)
  local n=0 d m
  for d in TASK-*/; do [ -d "$d" ] || continue; m=${d#TASK-}; m=${m%/}; m=$((10#$m)); [ "$m" -gt "$n" ] && n=$m; done
  printf 'TASK-%03d' $((n + 1))
}
op_alta() { local id; id=$(siguiente_id); tarea . "$id" "$1" planificada ""; OP_INFO="$id"; }
op_import() { # import <n> <prefijo>: N IDs consecutivos en UNA operacion
  local i id ids=""
  for i in $(seq 1 "$1"); do id=$(siguiente_id); tarea . "$id" "$2 $i" planificada ""; ids="$ids $id"; done
  OP_INFO="${ids# }"
}
op_start() { # start <id> <quien>
  local f="$1/tarea.md" a
  [ -f "$f" ] || { MOTIVO="no existe $1"; return 1; }
  a=$(campo "$f" asignado)
  if [ -n "$a" ] && [ "$a" != "$2" ]; then MOTIVO="la lleva $a"; return 1; fi
  if [ "$a" = "$2" ]; then OP_INFO="ya la llevaba $2"; return 2; fi
  tarea . "$1" "$(campo "$f" titulo)" en-curso "$2"; OP_INFO="$1 para $2"
}
op_transicion() { # transicion <id> <estado> <quien>: solo sobre la tarea propia
  local f="$1/tarea.md" a
  [ -f "$f" ] || { MOTIVO="no existe $1"; return 1; }
  a=$(campo "$f" asignado)
  [ "$a" = "$3" ] || { MOTIVO="la lleva ${a:-nadie}, no $3"; return 1; }
  tarea . "$1" "$(campo "$f" titulo)" "$2" "$3"; OP_INFO="$1 -> $2"
}
ejecutar_op() { # ejecutar_op "op|arg1|arg2..." sobre el estado del directorio actual
  local -a p; IFS='|' read -ra p <<< "$1"
  MOTIVO="" OP_INFO=""
  "op_${p[0]}" "${p[@]:1}"; local rc=$?
  [ $rc = 0 ] || return $rc
  roadmap; g add -A . && g commit -qm "${p[*]}" -m "Operacion: $1"
}

######################## la transaccion (punto unico de escritura) ########################
# tx <clon> <etiqueta> "op|args": imprime una linea por intento y sale con el codigo
tx() {
  local clon=$1 et=$2 opstr=$3
  ( RANDOM=$BASHPID
    log() { echo "[$et] $*"; }
    cd "$clon/tareas" || exit 2
    gd=$(cd "$(g rev-parse --git-common-dir)" && pwd)
    lock="$gd/taskcode-estado.lock"           # 1. cerrojo local del clon
    until ( set -o noclobber; echo $BASHPID > "$lock" ) 2>/dev/null; do sleep 0.05; done
    trap 'rm -f "$lock"' EXIT
    op=${opstr%%|*}

    if ! g remote | grep -qx origin; then     # RN-06: sin remoto, el commit local es la transaccion
      ejecutar_op "$opstr"; rc=$?
      [ $rc = 1 ] && { log "$opstr -> Rechazada ($MOTIVO)"; exit $RC_RECHAZADA; }
      log "$opstr -> Publicada (repo sin remoto: commit local) ${OP_INFO}"; exit $RC_PUBLICADA
    fi

    orig=$(g rev-parse HEAD)
    # operaciones propias pendientes de publicar (commits locales de una sesion sin red)
    pendientes=$(g log --reverse --format=%B origin/taskcode/tareas..HEAD | sed -n 's/^Operacion: //p')
    sin_red() {
      g reset -q --hard "$orig"
      if es_compartida "$op"; then log "$opstr -> SinRed (exige red; nada escrito)"; exit $RC_SINRED; fi
      ejecutar_op "$opstr"; rc=$?
      [ $rc = 1 ] && { log "$opstr -> Rechazada ($MOTIVO)"; exit $RC_RECHAZADA; }
      log "$opstr -> SinRed (pendiente de publicar: commit local)"; exit $RC_SINRED
    }
    intento=1
    while :; do
      g fetch -q origin taskcode/tareas 2>/dev/null || sin_red      # 2. fetch
      g reset -q --hard origin/taskcode/tareas                       #    estado fresco
      while IFS= read -r p; do [ -n "$p" ] || continue
        if ejecutar_op "$p"; then log "pendiente re-ejecutada: $p"; else
          [ $? = 1 ] && log "pendiente RECHAZADA al re-ejecutar: $p ($MOTIVO)"; fi
      done <<< "$pendientes"
      ejecutar_op "$opstr"; rc=$?                                    # 3. operacion
      if [ $rc = 1 ] && [ -z "$pendientes" ]; then
        log "$opstr -> Rechazada ($MOTIVO) en el intento $intento"; exit $RC_RECHAZADA
      fi
      if [ "$(g rev-parse HEAD)" = "$(g rev-parse origin/taskcode/tareas)" ]; then
        log "$opstr -> Publicada (sin cambios: ${OP_INFO})"; exit $RC_PUBLICADA
      fi
      if [ -n "${TX_PAUSA:-}" ] && [ $intento = 1 ]; then              # solo para la prueba 3
        : > "$TX_PAUSA.listo"; while [ ! -f "$TX_PAUSA" ]; do sleep 0.05; done
      fi
      if g push -q origin taskcode/tareas 2> "$gd/tx-push.err"; then   # 4. push sin force (CAS)
        [ $rc = 1 ] && { log "$opstr -> Rechazada ($MOTIVO); pendientes publicadas"; exit $RC_RECHAZADA; }
        log "$opstr -> Publicada (intentos=$intento) ${OP_INFO}"; exit $RC_PUBLICADA
      fi
      if grep -qE 'rejected|fetch first|non-fast-forward' "$gd/tx-push.err"; then
        if [ $intento -ge "$MAX_INTENTOS" ]; then
          g reset -q --hard "$orig"; log "$opstr -> Agotada tras $intento intentos"; exit $RC_AGOTADA
        fi
        # espera exponencial con jitter: base * 2^(n-1) * U(0.5, 1)
        espera=$(awk -v b="$ESPERA_BASE" -v n=$intento -v s=$RANDOM 'BEGIN { printf "%.2f", b * 2^(n-1) * (0.5 + (s % 1000) / 2000) }')
        log "intento $intento: push rechazado (non-fast-forward); commit descartado, re-ejecucion tras ${espera}s"
        sleep "$espera"; intento=$((intento + 1)); continue           # 5. reintento
      fi
      sin_red                                                         # el remoto desaparecio a mitad
    done )
}

# start: la toma se publica ANTES de crear rama y worktree
cmd_start() { # cmd_start <clon> <etiqueta> <id> <quien>
  local rama; rama="feature/$(echo "$3" | tr 'A-Z' 'a-z')"
  tx "$1" "$2" "start|$3|$4"; local rc=$?
  if [ $rc = 0 ]; then
    g -C "$1" rev-parse -q --verify "refs/heads/$rama" >/dev/null || g -C "$1" branch -q "$rama" develop
    [ -d "$base/wt-$2-$3" ] || g -C "$1" worktree add -q "$base/wt-$2-$3" "$rama"
    echo "[$2] start: rama $rama y worktree creados"
  else
    echo "[$2] start: $(nombre_rc $rc) (codigo $rc); no se crea rama ni worktree"
  fi
  return $rc
}

######################## preparacion ########################
preparar() { # preparar <nombre>: remoto bare (sin force-push) + semilla con develop y la rama huerfana
  local r="$base/$1.git"
  g init -q --bare "$r"; g -C "$r" config receive.denyNonFastForwards true
  g init -q "$base/$1-semilla"
  ( cd "$base/$1-semilla" && echo "codigo v1" > app.txt && echo "tareas/" > .gitignore &&
    g add . && g commit -qm "codigo inicial" && g remote add origin "$r" && g push -q origin develop &&
    g worktree add -q --orphan -b taskcode/tareas tareas && cd tareas && roadmap &&
    g add . && g commit -qm "estado inicial" && g push -q origin taskcode/tareas )
}
clonar() { # clonar <nombre-remoto> <dir>
  g clone -q -b develop "$base/$1.git" "$base/$2"
  g -C "$base/$2" worktree add -q --track -b taskcode/tareas tareas origin/taskcode/tareas
}
en_origin() { g -C "$base/$1/tareas" fetch -q origin; }   # en_origin <clon>: trae el estado
ls_origin() { g -C "$base/$1/tareas" ls-tree --name-only origin/taskcode/tareas | grep '^TASK-'; }
ver_origin() { g -C "$base/$1/tareas" show "origin/taskcode/tareas:$2"; }

echo "git: $(git --version); MAX_INTENTOS=$MAX_INTENTOS; ESPERA_BASE=${ESPERA_BASE}s"

######################## 1. altas concurrentes ########################
ronda() { # ronda <carpeta> <espera-base> <detalle 0|1>: 3 clones x 4 altas a la vez
  nueva_prueba "$1"
  preparar r; for c in c1 c2 c3; do clonar r $c; done
  local t0 t1 c i
  t0=$(date +%s%N)
  for c in c1 c2 c3; do
    ( ESPERA_BASE=$2; while [ ! -f "$base/salida" ]; do sleep 0.01; done
      for i in 1 2 3 4; do tx "$base/$c" $c "alta|alta $i de $c"; done ) > "$base/$c.log" 2>&1 &
  done
  : > "$base/salida"; wait
  t1=$(date +%s%N); SEG=$(( (t1 - t0) / 1000000000 ))
  [ "$3" = 1 ] && cat "$base"/c?.log
  en_origin c1
  IDS=$(ls_origin c1); N_IDS=$(echo "$IDS" | grep -c TASK)
  PUBL=$(cat "$base"/c?.log | grep -c 'Publicada'); AGOT=$(cat "$base"/c?.log | grep -c 'Agotada')
  REINT=$(cat "$base"/c?.log | grep -c 'push rechazado')
  MAXI=$(cat "$base"/c?.log | sed -n 's/.*intentos=\([0-9]*\).*/\1/p' | sort -n | tail -1)
  CONSEC=$([ "$IDS" = "$(for i in $(seq 1 "$PUBL"); do printf 'TASK-%03d\n' $i; done)" ] && echo si || echo NO)
  TITULOS=$(for id in $IDS; do ver_origin c1 "$id/tarea.md" | sed -n 's/^titulo: //p'; done | sort -u | wc -l)
  N_ROAD=$(ver_origin c1 ROADMAP.md | grep -c '^- TASK-')
  LOCALES=0; for c in c1 c2 c3; do en_origin $c; LOCALES=$((LOCALES + $(g -C "$base/$c/tareas" rev-list --count origin/taskcode/tareas..HEAD))); done
}
sep "1. Altas concurrentes: 3 clones x 4 altas, a la vez, en segundo plano (ESPERA_BASE=${ESPERA_BASE}s)"
ronda t1 "$ESPERA_BASE" 1
echo "carpetas TASK-* en origin: $N_IDS ; titulos distintos: $TITULOS ; entradas en ROADMAP.md: $N_ROAD"
echo "IDs consecutivos desde TASK-001: $CONSEC"
echo "altas publicadas: $PUBL ; agotadas: $AGOT ; pushes rechazados y reintentados: $REINT ; maximo de intentos de una alta publicada: $MAXI ; duracion: ${SEG}s"
echo "commits locales sin publicar en los 3 clones: $LOCALES (una Agotada no deja nada escrito)"
echo "commits en origin/taskcode/tareas: $(g -C "$base/c1/tareas" rev-list --count origin/taskcode/tareas) (1 inicial + $PUBL) ; merges: $(g -C "$base/c1/tareas" rev-list --merges --count origin/taskcode/tareas)"
echo "remoto con receive.denyNonFastForwards=$(g -C "$base/r.git" config receive.denyNonFastForwards); el guion no usa --force"
resultado "1: $PUBL/12 altas publicadas, $AGOT agotadas; $N_IDS IDs distintos y consecutivos=$CONSEC; $TITULOS titulos (ninguna alta pisada); ROADMAP con $N_ROAD entradas; $REINT reintentos; historia lineal sin force"

sep "1b. Base de la espera: 3 rondas con 0.1s, 0.5s y 2s (MAX_INTENTOS=$MAX_INTENTOS)"
nueva_prueba t1b-medida; preparar r; clonar r c1
t0=$(date +%s%N); for i in 1 2 3 4 5; do tx "$base/c1" c1 "alta|sin contencion $i" > /dev/null; done; t1=$(date +%s%N)
echo "duracion media de una transaccion sin contencion: $(( (t1 - t0) / 5000000 )) ms"
for eb in 0.1 0.5 2; do
  ta=0; tr=0
  for k in 1 2 3; do
    ronda "t1b-$eb-$k" "$eb" 0
    echo "base ${eb}s ronda $k: publicadas $PUBL/12, agotadas $AGOT, reintentos $REINT, max intentos $MAXI, IDs consecutivos $CONSEC, commits locales $LOCALES, ${SEG}s"
    ta=$((ta + AGOT)); tr=$((tr + REINT))
  done
  resultado "1b: base ${eb}s -> $ta agotadas de 36 altas, $tr reintentos en 3 rondas"
done

######################## 2. toma concurrente ########################
sep "2. Toma concurrente: ana y beto lanzan start sobre TASK-001 a la vez"
nueva_prueba t2
preparar r2; clonar r2 ana; clonar r2 beto
tx "$base/ana" ana "alta|tarea disputada"
for q in ana beto; do
  ( while [ ! -f "$base/t2-salida" ]; do sleep 0.01; done
    cmd_start "$base/$q" $q TASK-001 $q; echo "[$q] codigo de salida: $?" ) > "$base/t2-$q.log" 2>&1 &
done
: > "$base/t2-salida"; wait
cat "$base"/t2-ana.log "$base"/t2-beto.log
en_origin ana
asig=$(ver_origin ana TASK-001/tarea.md | sed -n 's/^asignado: //p')
echo "asignado en origin: $asig"
for q in ana beto; do echo "ramas feature/* en el clon de $q: $(g -C "$base/$q" branch --list 'feature/*' --format='%(refname:short)' | tr '\n' ' ')"; done
perdedor=$([ "$asig" = ana ] && echo beto || echo ana)
r_g=$(g -C "$base/$asig" branch --list 'feature/*' | wc -l); r_p=$(g -C "$base/$perdedor" branch --list 'feature/*' | wc -l)
echo "-- idempotencia: $asig repite start"
cmd_start "$base/$asig" $asig TASK-001 $asig; echo "[$asig] codigo de salida: $?"
echo "commits en origin tras repetir: $(en_origin ana; g -C "$base/ana/tareas" rev-list --count origin/taskcode/tareas) (1 inicial + alta + toma)"
resultado "2: TASK-001 la lleva $asig; el otro recibe Rechazada (la lleva $asig); ramas: ganador $r_g, perdedor $r_p; repetir start es idempotente"

######################## 3. rebase frente a re-ejecucion ########################
sep "3a. Rebase ingenuo: c1 y c2 crean la siguiente desde el mismo estado y c2 hace pull --rebase"
nueva_prueba t3
preparar r3; clonar r3 c1; clonar r3 c2
for c in c1 c2; do ( cd "$base/$c/tareas" && g pull -q && id=$(siguiente_id) && tarea . "$id" "de $c" planificada "" && roadmap && g add -A . && g commit -qm "alta $id ($c)" && echo "$c calcula en local: $id" ); done
( cd "$base/c1/tareas" && g push -q origin taskcode/tareas && echo "c1 push OK" )
cd "$base/c2/tareas"
g push -q origin taskcode/tareas 2>&1 | grep -E "rejected" | head -1 | sed 's/^/c2 push: /'
if g pull -q --rebase origin taskcode/tareas >/dev/null 2>&1; then reb="sin conflicto"; else
  reb="conflicto en: $(g diff --name-only --diff-filter=U | tr '\n' ' ')"; g status --short | sed 's/^/  /'; g rebase --abort; fi
echo "pull --rebase: $reb"
g reset -q --hard origin/taskcode/tareas; cd "$base"
resultado "3a: pull --rebase con el mismo ID -> $reb"

sep "3b. Re-ejecucion: c2 calcula su alta, c1 publica antes, c2 reintenta"
rm -f "$base/t3-pausa" "$base/t3-pausa.listo"
TX_PAUSA="$base/t3-pausa" tx "$base/c2" c2 "alta|de c2 (re-ejecucion)" > "$base/t3-c2.log" 2>&1 &
while [ ! -f "$base/t3-pausa.listo" ]; do sleep 0.05; done
echo "c2 tiene listo en local, sin publicar: $(g -C "$base/c2/tareas" log -1 --format=%s)"
tx "$base/c1" c1 "alta|de c1 (re-ejecucion)"
: > "$base/t3-pausa"; wait
cat "$base/t3-c2.log"
en_origin c1; echo "origin/taskcode/tareas:"; ver_origin c1 ROADMAP.md | sed 's/^/  /'
n3=$(ls_origin c1 | sort -u | wc -l)
resultado "3b: re-ejecucion -> $n3 carpetas distintas, sin conflicto; c2 recalculo su ID ($(grep -o 'TASK-[0-9]*' "$base/t3-c2.log" | tail -1))"

######################## 4. sin red: operaciones compartidas ########################
sep "4. Sin red: alta y start con el remoto inalcanzable"
nueva_prueba t4
preparar r4; clonar r4 ana
tx "$base/ana" ana "alta|tarea de ana"
g -C "$base/ana" remote set-url origin "$base/no-existe.git"
antes=$(g -C "$base/ana/tareas" rev-parse HEAD)
tx "$base/ana" ana "alta|alta sin red"; rc_a=$?
cmd_start "$base/ana" ana TASK-001 ana; rc_s=$?
despues=$(g -C "$base/ana/tareas" rev-parse HEAD)
st=$(g -C "$base/ana/tareas" status --short)
echo "codigos: alta=$rc_a ($(nombre_rc $rc_a)), start=$rc_s ($(nombre_rc $rc_s))"
echo "HEAD de tareas/ igual que antes: $([ "$antes" = "$despues" ] && echo si || echo NO) ; git status de tareas/: '${st}' (vacio = limpio)"
echo "ramas feature/*: '$(g -C "$base/ana" branch --list 'feature/*' | tr -d ' ')' (vacio = ninguna)"
resultado "4: sin red alta=$(nombre_rc $rc_a) y start=$(nombre_rc $rc_s); sin commit local=$([ "$antes" = "$despues" ] && echo si || echo NO); worktree limpio=$([ -z "$st" ] && echo si || echo NO)"

######################## 5. sin red: transicion propia pendiente ########################
sep "5. Sin red: transicion de una tarea propia queda pendiente y se publica re-ejecutandola"
g -C "$base/ana" remote set-url origin "$base/r4.git"
cmd_start "$base/ana" ana TASK-001 ana
clonar r4 beto
g -C "$base/ana" remote set-url origin "$base/no-existe.git"
tx "$base/ana" ana "transicion|TASK-001|en-revision|ana"; rc_t=$?
pend=$(g -C "$base/ana/tareas" rev-parse HEAD)
echo "codigo: $rc_t ($(nombre_rc $rc_t)) ; commits pendientes de publicar: $(g -C "$base/ana/tareas" rev-list --count origin/taskcode/tareas..HEAD) ($(g -C "$base/ana/tareas" log -1 --format='%h %s'))"
echo "-- una tarea ajena sin red se rechaza (beto sin red intenta mover TASK-001)"
g -C "$base/beto" remote set-url origin "$base/no-existe.git"
tx "$base/beto" beto "transicion|TASK-001|terminada|beto"; echo "[beto] codigo de salida: $?"
g -C "$base/beto" remote set-url origin "$base/r4.git"
echo "-- mientras, beto (con red) da de alta otra tarea: el remoto avanza"
tx "$base/beto" beto "alta|tarea de beto"
echo "-- vuelve la red de ana; su siguiente operacion publica lo pendiente"
g -C "$base/ana" remote set-url origin "$base/r4.git"
tx "$base/ana" ana "alta|ana tras volver la red"
en_origin ana
est=$(ver_origin ana TASK-001/tarea.md | sed -n 's/^estado: //p')
anc=$(g -C "$base/ana/tareas" merge-base --is-ancestor "$pend" origin/taskcode/tareas && echo si || echo no)
echo "estado de TASK-001 en origin: $est"
echo "el commit pendiente ${pend:0:7} esta tal cual en origin: $anc (no = se re-ejecuto sobre lo nuevo, no se rebaso)"
echo "pendientes tras publicar: $(g -C "$base/ana/tareas" rev-list --count origin/taskcode/tareas..HEAD)"
ver_origin ana ROADMAP.md | sed 's/^/  /'
resultado "5: sin red la transicion propia queda pendiente ($(nombre_rc $rc_t)); al volver la red TASK-001=$est en origin, re-ejecutada; la ajena se rechaza"

######################## 6. repo sin remoto ########################
sep "6. Repo sin remoto (una persona, RN-06): el commit local es la transaccion"
nueva_prueba t6
g init -q "$base/solo"
( cd "$base/solo" && echo "codigo v1" > app.txt && echo "tareas/" > .gitignore && g add . && g commit -qm "codigo inicial" &&
  g worktree add -q --orphan -b taskcode/tareas tareas && cd tareas && roadmap && g add . && g commit -qm "estado inicial" )
tx "$base/solo" solo "alta|primera"; tx "$base/solo" solo "import|2|lote"
cmd_start "$base/solo" solo TASK-001 carlos; rc6=$?
tx "$base/solo" solo "start|TASK-001|agente"; rc6b=$?
echo "commits en taskcode/tareas: $(g -C "$base/solo/tareas" rev-list --count HEAD) ; ramas feature/*: $(g -C "$base/solo" branch --list 'feature/*' --format='%(refname:short)')"
( cd "$base/solo/tareas" && cat ROADMAP.md | sed 's/^/  /' )
resultado "6: sin remoto alta, import (3 IDs en 1 commit) y start=$(nombre_rc $rc6); segundo start=$(nombre_rc $rc6b)"

######################## 7. resumen ########################
sep "7. Resumen"
cat "$RAIZ/resultados.txt"
echo; echo "FIN PoC"

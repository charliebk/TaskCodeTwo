"""PoC de F1-17 (TASK-075): progreso en vivo y estimacion de tiempo.

Se ejecuta con `python -I -B`. SOLO LECTURA sobre ToolsaWeb, OpenDockTrading y TaskCodeTwo: se leen sus
objetos con `git log`, `git ls-tree` y `git cat-file --batch` en commits fijos (FIJO_*), sin clonar ni
escribir nada en ellos. No habla con la red, con GitHub ni con ningun modelo.

Alcance reducido por decision de la persona (2026-11-15): completo en el lector del historico y la
estimacion fuera de muestra con su error frente a las lineas base; comprobacion minima en lo demas.

Bloques:
(H) Lector del historico tolerante: las tareas terminadas de las dos copias, con los descartes contados
    por motivo; tareas troceadas agrupadas (un grupo es una sola muestra).
(R) Real recalculado desde ## Transiciones con corte de huecos (D6a: huecos de mas de 45 min sin commit
    ni coste), frente a la linea «Real ... sin pausas» de la 0.16; casos de control TASK-096 (ODT) y
    TASK-296 (ToolsaWeb); sensibilidad a otros umbrales y a la variante «tope».
(E) Estimacion fuera de muestra (rolling origin por repo y cruzado) por fases medibles (implementacion,
    rondas, cierre; diseno sin calibrar, D7a) con la cascada de tareas parecidas; mediana/p80/p90 del
    error relativo, % dentro de +-50 % y error medio (D8a); lineas base: la «Estimada» de la 0.16, la
    mediana ingenua y la mediana por complejidad; control dentro de muestra (fuga).
(D) `resumen` (dashboard) y `repartir --personas N` con camino critico y zonas (D4a: sin zona = choca
    con todas) sobre las tareas de TaskCodeTwo; reproduccion del prototipo de la sesion.
(S) `start` y predecesoras (D3a: fetch, merge en develop, negarse sin red) y aviso de solape, con un
    remoto bare local simulado.
(L) Status line: lectura O(1) de un valor derivado (D5a, estado.json con ultimo valor bueno) frente a
    escritura atomica concurrente; rename con el lector abierto en Windows; dos escritores de coste.
(F) Hora de fin de un lote con intervalo p50/p80, contra la suma de medianas.
(G) El ADR como dato: tablas entre marcadores, cifras de la prosa por contexto, IDs, enlaces, secciones;
    mutaciones de control, cada una por su motivo.

Uso: python -I -B poc-f1-17.py <repo-TaskCodeTwo> <ToolsaWeb> <OpenDockTrading>
La carpeta de trabajo es fija: %LOCALAPPDATA%/pruebas-claude/taskcodetwo/task-075; tiene que no existir,
estar vacia o llevar la marca; se borra al empezar y al terminar. Los tiempos de reloj van a stderr.
Sale con 0 solo si todo sale como se espera.
"""
import json
import math
import os
import random
import re
import shutil
import statistics
import subprocess
import sys
import threading
import time
from datetime import datetime, timezone

POC = os.path.dirname(os.path.abspath(__file__))
ADR_REL = os.path.join("docs", "adr", "F1-17-progreso-y-estimacion.md")
FILA_INDICE = "| [F1-17](F1-17-progreso-y-estimacion.md) |"
MARCA = ".poc-f1-17"
FIJO_TW = "43ab5907e22c8e4fd552cfbbf989e049ab76493b"    # develop de ToolsaWeb, leido el 2026-11-15
FIJO_ODT = "2b0064f2c99a70b1bcd14e952379f97a457a661e"   # develop de OpenDockTrading, leido el 2026-11-15
FIJO_TC2 = "d9a5b10fd4302fa1b0066a7412d07a5dc55972c1"   # TaskCodeTwo con TASK-075 en curso
PROTO_TC2 = "8b0184792024aab91ed422d2297ce166e1f9252f"  # TaskCodeTwo cuando se hizo el prototipo (2026-11-14 09:11 -03)
UMBRAL = 45                     # D6a: minutos de hueco sin commit ni coste que se cortan
UMBRALES = [30, 45, 60, 90, 120, 180, None]
N_MIN = 5                       # muestras minimas de un nivel de la cascada (declarado antes de medir)
PCT = 0.80                      # D2a: se ensena la mediana y el p80
POR_DEFECTO = 55                # duracion por defecto del prototipo de la sesion donde no hay plan
SEMILLA = 17                    # remuestreo determinista
REMUESTREOS = 4000
AHORA = datetime(2026, 11, 15, 12, 0, 0, tzinfo=timezone.utc).timestamp()   # «ahora» fijo de D, L y F
FECHA = "1763208000 +0000"      # fecha fija de los commits de S: SHA deterministas
AGENTE = ("taskcodetwo-agente[bot]", "agente@taskcodetwo.invalid")
PERSONA = ("Carlos", "carlos@personas.invalid")
COMPANERO = ("Companero", "companero@personas.invalid")
ORDEN_CARRIL = ["errata", "fix", "funcionalidad", "riesgo-alto"]
FASES = ["implementacion", "rondas", "cierre"]

FALLOS = []
T0 = time.time()


def falla(seccion, texto):
    FALLOS.append(f"{seccion}: {texto}")
    print(f"FALLO en {seccion}: {texto}")


def comprueba(seccion, cond, texto):
    if not cond:
        falla(seccion, texto)
    return cond


def sep(t):
    print()
    print(f"=== {t} ===")


def reloj(texto):
    print(f"[tiempo] {texto} ({time.time() - T0:.1f} s desde el inicio)", file=sys.stderr, flush=True)


def borrar(ruta):
    if os.path.lexists(ruta):
        def _rw(func, p, _exc):
            os.chmod(p, 0o700)
            func(p)
        shutil.rmtree(ruta, onerror=_rw)


def carpeta_trabajo():
    base = os.environ.get("LOCALAPPDATA")
    if not base:
        raise SystemExit("ERROR: falta LOCALAPPDATA")
    t = os.path.join(base, "pruebas-claude", "taskcodetwo", "task-075")
    if os.path.lexists(t):
        if not os.path.isdir(t):
            raise SystemExit(f"ERROR: {t} existe y no es una carpeta")
        if os.listdir(t) and not os.path.exists(os.path.join(t, MARCA)):
            raise SystemExit(f"ERROR: {t} no esta vacia ni lleva la marca {MARCA}: no se borra")
    return t


class ErrorGit(Exception):
    pass


def git(repo, *args, entrada=None, ok=(0,), quien=AGENTE, env=None):
    e = dict(os.environ)
    e.update(GIT_AUTHOR_NAME=quien[0], GIT_AUTHOR_EMAIL=quien[1], GIT_COMMITTER_NAME=quien[0],
             GIT_COMMITTER_EMAIL=quien[1], GIT_AUTHOR_DATE=FECHA, GIT_COMMITTER_DATE=FECHA, GIT_TERMINAL_PROMPT="0")
    if env:
        e.update(env)
    if isinstance(entrada, str):
        entrada = entrada.encode("utf-8")
    r = subprocess.run(["git", "-C", repo, "-c", "core.quotepath=false", "-c", "core.longpaths=true",
                        "-c", "core.autocrlf=false", "-c", "advice.detachedHead=false", "-c", "init.defaultBranch=develop",
                        *args], input=entrada, capture_output=True, env=e,
                       stdin=None if entrada is not None else subprocess.DEVNULL)
    if r.returncode not in ok:
        raise ErrorGit(f"git {' '.join(args)[:140]} en {os.path.basename(repo)}: "
                       f"{r.stderr.decode('utf-8', 'replace').strip()[:400]}")
    return r.returncode, r.stdout.decode("utf-8", "replace")


def g1(repo, *a, **k):
    return git(repo, *a, **k)[1].strip()


def coma(x, d=1):
    return f"{x:.{d}f}".replace(".", ",")


def pct(a, b):
    return coma(100 * a / b) if b else "0,0"


def mediana(xs):
    xs = sorted(x for x in xs if x is not None)
    return statistics.median(xs) if xs else None


def cuantil(xs, q):
    """Rango mas cercano: el valor en la posicion ceil(q*n) de la lista ordenada (el de F1-18)."""
    xs = sorted(x for x in xs if x is not None)
    if not xs:
        return None
    k = max(1, math.ceil(q * len(xs) - 1e-9))
    return xs[min(len(xs), k) - 1]


def fecha(s):
    return datetime.strptime(s, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc).timestamp()


def hm(ts):
    return datetime.fromtimestamp(ts, timezone.utc).strftime("%Y-%m-%d %H:%M")


# =========================================================================== H: lector del historico

def frontmatter(texto):
    partes = texto.split("---")
    if not texto.startswith("---") or len(partes) < 3:
        return None
    fm = {}
    for l in partes[1].splitlines():
        m = re.match(r"^([a-z_]+):\s*(.*?)\s*$", l)
        if m:
            fm[m.group(1)] = m.group(2).strip('"')
    return fm


def seccion_md(texto, titulo):
    m = re.search(rf"^## {re.escape(titulo)}\s*$(.*?)(?=^## |\Z)", texto, re.S | re.M)
    return m.group(1) if m else None


def tabla_md(bloque):
    filas = [[c.strip() for c in l.strip().strip("|").split("|")] for l in (bloque or "").splitlines()
             if l.strip().startswith("|")]
    if len(filas) < 2:
        return []
    cab = filas[0]
    return [dict(zip(cab, f)) for f in filas[2:]]


PATRONES_TROCEO = [
    r"\b(?:Primera|Segunda|Tercera|Cuarta|Quinta|Sexta) parte de (TASK-\d+)",
    r"parte del [^.]{0,80}? troceado en (TASK-\d+)",
    r"\bParte de (TASK-\d+)",
    r"\bTroceo de (TASK-\d+)",
    r"pasan? a (TASK-\d+) \(troceo",
    r"\bcontinuaci[oó]n de (TASK-\d+)",
    r"\bSigue a (TASK-\d+)",
    r"(TASK-\d+) dej[oó] resuelto",
]


def troceo(tid_, objetivo):
    """IDs con los que se agrupa una tarea por lo que dice su Objetivo (regla textual declarada)."""
    ids = set()
    for p in PATRONES_TROCEO:
        ids |= set(re.findall(p, objetivo))
    m = re.search(r"Troceada en el dise[nñ]o[^.]*\.?[^.]*", objetivo)
    if m:
        ids |= set(re.findall(r"TASK-\d+", m.group(0)))
    ids.discard(tid_)
    return ids


def estimada_016(texto_tarea, texto_plan):
    """(minutos, fuente) de la estimacion del agente que guarda la 0.16, o (None, motivo)."""
    dur = seccion_md(texto_tarea, "Duracion")
    if dur:
        m = re.search(r"^- Estimada: (\d+) min\s*$", dur, re.M)
        if m:
            return int(m.group(1)), "duracion"
    if texto_plan:
        m = re.search(r"^\s*Duraci[oó]n estimada:\s*(\d+)\s*min\s*$", texto_plan, re.M)
        if m:
            return int(m.group(1)), "plan"
        m = re.search(r"Duraci[oó]n estimada:\s*(\d+)\s*min", texto_plan)
        if m:
            return int(m.group(1)), "tolerante"
    if dur:
        m = re.search(r"^- Estimada: .*?(\d+) min", dur, re.M)
        if m:
            return int(m.group(1)), "tolerante"
    return None, "sin-estimada"


def real_016(texto_tarea):
    dur = seccion_md(texto_tarea, "Duracion")
    m = re.search(r"^- Real \(.*?\): (\d+) min", dur or "", re.M)
    return int(m.group(1)) if m else None


def es_riesgo_alto(r):
    p = r.lower().lstrip("./").split("/")
    return p[0] in (".github", ".taskcode", ".gitlab") or r.lower().endswith(".gitlab-ci.yml") or p[-1] == "codeowners"


def carril_minimo(rutas):
    """F1-10 decision 4 (sin rutas_carril): riesgo alto por ruta, errata si todo esta bajo docs/, fix si no."""
    if not rutas:
        return None
    if any(es_riesgo_alto(r) for r in rutas):
        return "riesgo-alto"
    if all(r.startswith("docs/") for r in rutas):
        return "errata"
    return "fix"


def carril_efectivo(tipo, rutas):
    declarado = "fix" if tipo == "fix" else "funcionalidad"
    minimo = carril_minimo(rutas)
    if minimo is None:
        return declarado
    if minimo == "errata":   # el plan de una tarea solo de documentacion declararia errata (proxy, ver ADR)
        return "errata"
    return max(declarado, minimo, key=ORDEN_CARRIL.index)


class Repo:
    """Lo que se lee de un repo en su commit fijo, solo en lectura (`git log`, `git ls-tree` y
    `git cat-file --batch`): tareas, commits por tarea y merges de tarea."""

    def __init__(self, nombre, repo, fijo):
        self.nombre, self.copia, self.fijo = nombre, repo, fijo
        git(repo, "cat-file", "-e", fijo + "^{commit}")
        self.ficheros = g1(repo, "ls-tree", "-r", "--name-only", fijo, "tareas").splitlines()
        self.commits = {}       # TASK-NNN -> [timestamps] de commits cuyo asunto la nombra
        self.merges = {}        # TASK-NNN -> (p1, p2) del merge de su rama
        log = g1(repo, "log", "--format=%H%x1f%ct%x1f%P%x1f%s", fijo)
        for l in log.splitlines():
            sha, ct, padres, asunto = l.split("\x1f", 3)
            for n in set(re.findall(r"\bTASK-(\d+)\b", asunto, re.I)):
                self.commits.setdefault(f"TASK-{int(n):03d}", []).append(int(ct))
            m = re.match(r"merge\((?:feature|fix)\): (?:feature|fix)/task-(\d+)-", asunto)
            if m and len(padres.split()) == 2:
                self.merges.setdefault(f"TASK-{int(m.group(1)):03d}", tuple(padres.split()))
        self._cat = subprocess.Popen(["git", "-C", repo, "cat-file", "--batch"], stdin=subprocess.PIPE,
                                     stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)

    def texto(self, ruta):
        self._cat.stdin.write(f"{self.fijo}:{ruta}\n".encode("utf-8"))
        self._cat.stdin.flush()
        cab = self._cat.stdout.readline().decode().split()
        if len(cab) < 3 or cab[1] != "blob":
            return ""
        datos = self._cat.stdout.read(int(cab[2]))
        self._cat.stdout.read(1)
        return datos.decode("utf-8", "replace")

    def cerrar(self):
        self._cat.stdin.close()
        self._cat.wait()

    def rama(self, t):
        """(timestamps de los commits de la rama, rutas cambiadas fuera de tareas/, lineas cambiadas), con un
        solo `git log --numstat` de M^1..M^2 (suma de los commits, no el diff neto)."""
        if t not in self.merges:
            return [], None, None
        p1, p2 = self.merges[t]
        ts, rutas, lineas = [], set(), 0
        for l in g1(self.copia, "log", "--format=@%ct", "--numstat", "--no-renames", f"{p1}..{p2}").splitlines():
            if l.startswith("@"):
                ts.append(int(l[1:]))
            elif "\t" in l:
                a, b, r = l.split("\t", 2)
                if not r.startswith("tareas/"):
                    rutas.add(r)
                    lineas += (int(a) if a.isdigit() else 0) + (int(b) if b.isdigit() else 0)
        return ts, sorted(rutas), lineas


def leer_historico(repo, estados=("04-terminadas",)):
    """historico(tareas) -> (muestras, descartes): el unico sitio que interpreta Transiciones, Duracion y Coste."""
    muestras, descartes, filas_malas = [], {}, 0
    rutas = sorted(f for f in repo.ficheros if f.endswith("/tarea.md") and f.split("/")[1] in estados)

    def descarta(t, motivo):
        descartes.setdefault(motivo, []).append(t)

    for ruta in rutas:
        t = ruta.split("/")[2]
        texto = repo.texto(ruta)
        fm = frontmatter(texto)
        if fm is None:
            descarta(t, "frontmatter-ilegible")
            continue
        trans_raw = tabla_md(seccion_md(texto, "Transiciones"))
        if not trans_raw:
            descarta(t, "sin-transiciones")
            continue
        trans = []
        for x in trans_raw:
            try:
                trans.append((fecha(x.get("fecha", "")), x.get("fase", ""), x.get("version", "")))
            except ValueError:
                filas_malas += 1
        fases = [f for _, f, _ in trans]
        if "start" not in fases:
            descarta(t, "sin-start")
            continue
        if "finish" not in fases:
            descarta(t, "sin-finish")
            continue
        if "review" not in fases:
            descarta(t, "sin-review")
            continue
        start = min(ts for ts, f, _ in trans if f == "start")
        finish = max(ts for ts, f, _ in trans if f == "finish")
        reviews = sorted(ts for ts, f, _ in trans if f == "review")
        if not (start <= reviews[0] and reviews[-1] <= finish):
            descarta(t, "duracion-negativa")
            continue
        coste = []
        for x in tabla_md(seccion_md(texto, "Coste")):
            try:
                coste.append((fecha(x["fecha"]), x.get("fase", ""), int(x.get("total", "0") or 0)))
            except (ValueError, KeyError):
                filas_malas += 1
        base = ruta.rsplit("/", 1)[0]
        plan = f"{base}/planificacion/plan-final.md"
        texto_plan = repo.texto(plan) if plan in repo.ficheros else ""
        est, fuente = estimada_016(texto, texto_plan)
        ts_rama, rutas_diff, lineas = repo.rama(t)
        objetivo = seccion_md(texto, "Objetivo") or ""
        criterios = len(re.findall(r"^- \[[ xX]\]", seccion_md(texto, "Criterios de aceptacion") or "", re.M))
        compl = fm.get("complejidad")
        muestras.append({
            "repo": repo.nombre, "id": t, "tipo": fm.get("tipo", ""), "sprint": fm.get("sprint", ""),
            "complejidad": None if compl in (None, "", "null") else compl,
            "deps": re.findall(r"TASK-\d+", fm.get("dependencias", "")),
            "criterios": criterios, "start": start, "finish": finish, "reviews": reviews,
            "approve": max([ts for ts, f, _ in trans if f == "approve" and ts <= start] or [start]),
            "trans": [(ts, f) for ts, f, _ in trans], "versiones": sorted({v for _, _, v in trans}),
            "coste": coste, "tokens": sum(c for _, _, c in coste), "est_016": est, "fuente_016": fuente,
            "real_016": real_016(texto), "commits": sorted(repo.commits.get(t, []) + ts_rama),
            "rutas": rutas_diff, "lineas": lineas, "con_merge": t in repo.merges,
            "carril": carril_efectivo(fm.get("tipo", ""), rutas_diff),
            "troceo": troceo(t, objetivo), "rondas_n": len(reviews),
        })
    return muestras, descartes, filas_malas


def agrupar(muestras):
    """Union-find por la regla de troceo; devuelve {id: grupo} con el grupo nombrado por su ID menor."""
    padre = {m["id"]: m["id"] for m in muestras}

    def raiz(x):
        while padre[x] != x:
            padre[x] = padre[padre[x]]
            x = padre[x]
        return x
    for m in muestras:
        for o in m["troceo"]:
            if o in padre:
                a, b = raiz(m["id"]), raiz(o)
                if a != b:
                    padre[max(a, b)] = min(a, b)
    return {m["id"]: raiz(m["id"]) for m in muestras}


# =========================================================================== R: real con corte de huecos

def ventanas_pausa(m):
    """`pausa` hasta la transicion siguiente (`reanudar` o la que venga): la declara la persona."""
    ts = sorted(m["trans"])
    v = []
    for i, (t, f) in enumerate(ts):
        if f == "pausa":
            sig = next((u for u, g in ts[i + 1:] if g != "pausa"), None)
            if sig is not None and sig > t:
                v.append((t, sig))
    return v


def tramo(ini, fin, eventos, umbral, modo="tope", pausas=()):
    """Minutos activos entre ini y fin: suma de huecos entre eventos, sin las ventanas de pausa; un hueco
    > umbral se corta (modo «tope»: cuenta el umbral, la regla de D6a; modo «corte»: cuenta 0).
    umbral None = sin corte."""
    pts = {ini, fin} | {e for e in eventos if ini < e < fin}
    for a, b in pausas:
        pts |= {x for x in (a, b) if ini < x < fin}
    pts = sorted(pts)
    seg = 0.0
    for a, b in zip(pts, pts[1:]):
        medio = (a + b) / 2
        if any(pa <= medio <= pb for pa, pb in pausas):
            continue
        h = (b - a) / 60
        if umbral is not None and h > umbral:
            h = 0 if modo == "corte" else umbral
        seg += h
    return seg


def real(m, umbral=UMBRAL, modo="tope", con_pausas=True):
    """Real por fase desde Transiciones con corte de huecos (D6a). Eventos: transiciones, filas de
    ## Coste y commits de la tarea (asunto con su ID o commits de su rama)."""
    ev = set(m["commits"]) | {c[0] for c in m["coste"]} | {t for t, _ in m["trans"]}
    pa = ventanas_pausa(m) if con_pausas else ()
    r1, rn = m["reviews"][0], m["reviews"][-1]
    imp = tramo(m["start"], r1, ev, umbral, modo, pa)
    ron = tramo(r1, rn, ev, umbral, modo, pa)
    cie = tramo(rn, m["finish"], ev, umbral, modo, pa)
    return {"implementacion": imp, "rondas": ron, "cierre": cie, "total": imp + ron + cie}


def muestra_de_grupo(ms, umbral=UMBRAL, modo="tope"):
    """Un grupo de troceo es una muestra: suma por fase, fechas del primero y del ultimo."""
    ms = sorted(ms, key=lambda m: m["start"])
    rs = [real(m, umbral, modo) for m in ms]
    base = dict(ms[0])
    base.update({
        "id": ms[0]["id"] if len(ms) == 1 else "+".join(m["id"] for m in ms),
        "miembros": [m["id"] for m in ms], "finish": max(m["finish"] for m in ms),
        "approve": min(m["approve"] for m in ms), "start": ms[0]["start"],
        "real": {k: sum(r[k] for r in rs) for k in rs[0]},
        "real_016": sum(m["real_016"] for m in ms) if all(m["real_016"] is not None for m in ms) else None,
        "est_016": sum(m["est_016"] for m in ms) if all(m["est_016"] is not None for m in ms) else None,
        "tokens": sum(m["tokens"] for m in ms), "criterios": sum(m["criterios"] for m in ms),
        "lineas": sum(m["lineas"] or 0 for m in ms) if any(m["lineas"] is not None for m in ms) else None,
        "rondas_n": sum(m["rondas_n"] for m in ms),
        "carril": max((m["carril"] for m in ms), key=ORDEN_CARRIL.index),
    })
    return base


def muestras_agrupadas(muestras, umbral=UMBRAL, modo="tope", agrupar_troceo=True):
    if not agrupar_troceo:
        return [muestra_de_grupo([m], umbral, modo) for m in muestras]
    g = agrupar(muestras)
    grupos = {}
    for m in muestras:
        grupos.setdefault(g[m["id"]], []).append(m)
    return sorted((muestra_de_grupo(v, umbral, modo) for v in grupos.values()), key=lambda s: s["start"])


# =========================================================================== E: estimacion

def tam(s):
    c = s["criterios"]
    return "c1-3" if c <= 3 else ("c4-6" if c <= 6 else "c7+")


def tam_diff(s):
    l = s["lineas"]
    if l is None:
        return "d?"
    return "d<=200" if l <= 200 else ("d<=1000" if l <= 1000 else "d>1000")


NIVELES = ["carril+complejidad+tamano", "carril+complejidad", "carril", "repo", "semilla-carril", "semilla"]


def clave(s, nivel, tamf=tam):
    return {"carril+complejidad+tamano": (s["carril"], s["complejidad"], tamf(s)),
            "carril+complejidad": (s["carril"], s["complejidad"]),
            "carril": (s["carril"],), "repo": (), "semilla-carril": (s["carril"],), "semilla": ()}[nivel]


def resumen_muestras(ms, clave_real="real"):
    out = {"n": len(ms)}
    for k in FASES + ["total"]:
        xs = [m[clave_real][k] for m in ms]
        out[k] = (mediana(xs), cuantil(xs, PCT))
    out["tokens"] = (mediana([m["tokens"] for m in ms]), cuantil([m["tokens"] for m in ms], PCT))
    return out


def estimar(obj, propias, semilla, niveles=NIVELES, tamf=tam, n_min=N_MIN):
    """estimar(tarea, muestras) -> Estimacion: mediana y p80 por fase y total del primer nivel de la
    cascada con n_min muestras parecidas. `propias` son las del proyecto; `semilla`, las de otros."""
    for nivel in niveles:
        pool = semilla if nivel.startswith("semilla") else propias
        k = clave(obj, nivel, tamf)
        ms = [m for m in pool if clave(m, nivel, tamf) == k]
        if len(ms) >= n_min:
            e = resumen_muestras(ms)
            e["nivel"], e["muestras"] = nivel, ms
            return e
    return None


def disponibles(pool, obj, dentro=False):
    """Rolling origin: solo las que terminaron antes de que se apruebe el plan de la tarea."""
    if dentro:
        return list(pool)
    return [m for m in pool if m["finish"] <= obj["approve"] and m["id"] != obj["id"]]


def err_rel(est, real_):
    return abs(est - real_) / max(real_, 1.0)


def metricas(pares):
    """pares: [(estimado, real)] -> cifras de D8a."""
    if not pares:
        return None
    er = [err_rel(e, r) for e, r in pares]
    return {"n": len(pares), "med": mediana(er), "p80": cuantil(er, 0.8), "p90": cuantil(er, 0.9),
            "dentro50": sum(1 for x in er if x <= 0.5) / len(er), "mae": statistics.mean(abs(e - r) for e, r in pares),
            "mape": statistics.mean(er), "sesgo": mediana([e / max(r, 1.0) for e, r in pares])}


def evaluar(objetivos, propias, semilla, dentro=False, niveles=NIVELES, tamf=tam, n_min=N_MIN):
    """Para cada objetivo: estimacion del modelo y de las lineas base, con lo disponible antes."""
    filas = []
    for o in objetivos:
        p = disponibles(propias, o, dentro)
        s = disponibles(semilla, o, dentro)
        e = estimar(o, p, s, niveles, tamf, n_min)
        ingenua = mediana([m["real"]["total"] for m in p]) if p else None
        pc = [m["real"]["total"] for m in p if m["complejidad"] == o["complejidad"]] if o["complejidad"] else []
        filas.append({"o": o, "e": e, "ingenua": ingenua, "complejidad": mediana(pc) if pc else ingenua,
                      "016": o["est_016"]})
    return filas


def tabla_metricas(filas, quien):
    res = {}
    for nombre in ["modelo", "016", "ingenua", "complejidad"]:
        pares = []
        for f in filas:
            if not quien(f):
                continue
            est = f["e"]["total"][0] if nombre == "modelo" else f[nombre]
            if est is not None:
                pares.append((est, f["o"]["real"]["total"]))
        res[nombre] = metricas(pares)
    return res


def bootstrap_mejor(filas, a, b, quien, rng):
    """Remuestreo pareado: fraccion de remuestras en que la mediana del error de `a` es menor que la de `b`."""
    pares = []
    for f in filas:
        if not quien(f):
            continue
        ea = f["e"]["total"][0] if a == "modelo" else f[a]
        eb = f["e"]["total"][0] if b == "modelo" else f[b]
        if ea is None or eb is None:
            continue
        r = f["o"]["real"]["total"]
        pares.append((err_rel(ea, r), err_rel(eb, r)))
    if not pares:
        return None
    gana = 0
    for _ in range(2000):
        m = [pares[rng.randrange(len(pares))] for _ in pares]
        if mediana([x for x, _ in m]) < mediana([y for _, y in m]):
            gana += 1
    return gana / 2000


# =========================================================================== D: resumen y repartir

def leer_tc2(repo):
    """Todas las tareas de TaskCodeTwo en el commit fijo: id, estado, sprint, deps, plan, transiciones, coste."""
    out = {}
    for ruta in sorted(f for f in repo.ficheros if f.endswith("/tarea.md")):
        t = ruta.split("/")[2]
        texto = repo.texto(ruta)
        fm = frontmatter(texto) or {}
        base = ruta.rsplit("/", 1)[0]
        plan = f"{base}/planificacion/plan-final.md"
        tp = repo.texto(plan) if plan in repo.ficheros else ""
        m = re.search(r"^\s*Duraci[oó]n estimada:\s*(\d+)\s*min", tp, re.M)
        trans = []
        for x in tabla_md(seccion_md(texto, "Transiciones")):
            try:
                trans.append((fecha(x["fecha"]), x["fase"]))
            except (ValueError, KeyError):
                pass
        coste = []
        for x in tabla_md(seccion_md(texto, "Coste")):
            try:
                coste.append((fecha(x["fecha"]), x.get("fase", ""), int(x.get("total", "0") or 0)))
            except (ValueError, KeyError):
                pass
        out[t] = {"id": t, "estado": fm.get("estado", ""), "carpeta": ruta.split("/")[1], "titulo": fm.get("titulo", ""),
                  "sprint": fm.get("sprint", ""), "deps": re.findall(r"TASK-\d+", fm.get("dependencias", "")),
                  "plan_min": int(m.group(1)) if m else None, "trans": trans, "coste": coste,
                  "tokens": sum(c for _, _, c in coste), "complejidad": None, "tipo": fm.get("tipo", ""),
                  "criterios": len(re.findall(r"^- \[[ xX]\]", seccion_md(texto, "Criterios de aceptacion") or "", re.M))}
    return out


def prefijo_glob(g):
    p = []
    for c in g.strip("/").split("/"):
        if any(ch in c for ch in "*?["):
            break
        p.append(c)
    return p


def solapan(za, zb):
    """Zonas declaradas (listas de globs). Sin zona (None) = choca con todas (D4a). Solapan si el prefijo
    literal de un glob es prefijo, por componentes, del de otro."""
    if za is None or zb is None:
        return True
    for a in za:
        for b in zb:
            pa, pb = prefijo_glob(a), prefijo_glob(b)
            n = min(len(pa), len(pb))
            if pa[:n] == pb[:n]:
                return True
    return False


def nivel_inferior(tareas, dur, deps):
    """b(t) = dur(t) + max b(sucesoras): el trabajo que queda por el camino mas largo desde t."""
    suc = {t: [] for t in tareas}
    for t in tareas:
        for d in deps[t]:
            suc[d].append(t)
    memo = {}

    def b(t):
        if t not in memo:
            memo[t] = dur[t] + max((b(s) for s in suc[t]), default=0)
        return memo[t]
    for t in sorted(tareas):
        b(t)
    return memo


def repartir(tareas, dur, deps, zonas, n, respetar_zonas=True):
    """repartir(dag, duraciones, zonas, n) -> Reparto: listas con prioridad al trabajo restante (camino
    critico); dos tareas cuyas zonas solapan no van a la vez; sin zona choca con todas."""
    deps = {t: [d for d in deps[t] if d in tareas] for t in tareas}
    b = nivel_inferior(tareas, dur, deps)
    hecho, empezado, corriendo = set(), set(), {}
    libres = list(range(n))
    listas = {i: [] for i in range(n)}
    tramos = []
    ahora = 0.0
    while len(hecho) < len(tareas):
        listas_ok = sorted((t for t in tareas if t not in empezado and all(d in hecho for d in deps[t])),
                           key=lambda t: (-b[t], t))
        for t in listas_ok:
            if not libres:
                break
            if respetar_zonas and any(solapan(zonas.get(t), zonas.get(u)) for u in corriendo):
                continue
            p = libres.pop(0)
            corriendo[t] = (ahora + dur[t], p)
            tramos.append((ahora, ahora + dur[t], t))
            empezado.add(t)
            listas[p].append(t)
        if not corriendo:
            raise RuntimeError("reparto bloqueado: dependencias en ciclo")
        fin = min(f for f, _ in corriendo.values())
        ahora = fin
        for t in sorted([t for t, (f, _) in corriendo.items() if f == fin]):
            hecho.add(t)
            libres.append(corriendo.pop(t)[1])
            libres.sort()
    return {"total": ahora, "camino": max(b.values()) if b else 0, "listas": listas, "b": b, "tramos": tramos}


def fase_actual(trans):
    """Fase en curso y fases que faltan desde la ultima transicion (diseno sin calibrar, D7a)."""
    fases = [f for _, f in sorted(trans) if f != "pausa"]
    if "finish" in fases:
        return "terminada", []
    if "review" in fases:
        return "rondas", ["cierre"]
    if "start" in fases:
        return "implementacion", ["rondas", "cierre"]
    return "diseno", ["implementacion", "rondas", "cierre"]


def transcurrido(trans, coste, commits, fase, ahora, umbral=UMBRAL):
    """Minutos activos de la fase en curso hasta `ahora`: la regla de corte hasta el ultimo evento y el
    hueco abierto entero (puede haber alguien trabajando sin commit todavia; la siguiente transicion lo
    recalcula con el corte). Devuelve (minutos, minutos sin actividad registrada)."""
    ts = sorted(trans)
    ini = {"implementacion": max([t for t, f in ts if f == "start"] or [ahora]),
           "rondas": min([t for t, f in ts if f == "review"] or [ahora])}.get(fase, ahora)
    ev = {e for e in set(commits) | {c[0] for c in coste} | {t for t, _ in ts} if ini <= e <= ahora}
    ultimo = max(ev | {ini})
    m = {"trans": ts}
    return tramo(ini, ultimo, ev, umbral, "tope", ventanas_pausa(m)) + (ahora - ultimo) / 60, (ahora - ultimo) / 60


def restante(est, fase, transc):
    """restante(est, transiciones, ahora): distribucion de lo que falta entre las tareas parecidas; si
    hay N_MIN que ya pasaron de lo transcurrido en la fase, se condiciona a ellas (supervivencia)."""
    if est is None or fase == "terminada":
        return None
    ms = est["muestras"]
    siguientes = {"diseno": FASES, "implementacion": FASES, "rondas": ["rondas", "cierre"]}[fase]
    actual = None if fase == "diseno" else fase
    vivas = [m for m in ms if actual is None or m["real"][actual] > transc]
    base = vivas if len(vivas) >= N_MIN else ms
    xs = []
    for m in base:
        r = sum(m["real"][k] for k in siguientes)
        if actual:
            r -= min(transc, m["real"][actual])
        xs.append(max(0.0, r))
    return {"med": mediana(xs), "p80": cuantil(xs, PCT), "n": len(base), "condicionada": base is vivas and actual is not None}


def suma_remuestreada(distribuciones, rng, n=REMUESTREOS):
    """Hora de fin / total de varias tareas: p50 y p80 de la suma remuestreada (no la suma de medianas)."""
    tot = []
    for _ in range(n):
        tot.append(sum(d[rng.randrange(len(d))] for d in distribuciones if d))
    return mediana(tot), cuantil(tot, PCT)


def carril_tc2(t):
    """Carril declarado de una tarea pendiente de TaskCodeTwo (proxy: los ADR de la Fase 1 son solo docs)."""
    return "errata" if re.match(r"F1-\d+ ADR", t["titulo"]) else ("fix" if t["tipo"] == "fix" else "funcionalidad")


def linea_estado(e):
    """Lo que pinta la status line a partir de estado.json (sin recorrer nada)."""
    if e is None:
        return "tc2: sin estado"
    faltan = ", ".join(e["faltan"]) if e["faltan"] else "ninguna"
    if e.get("restante") is None:
        est = "sin estimacion"
    else:
        est = f"faltan ≈{round(e['restante'][0])} min (p80 {round(e['restante'][1])})"
    q = f" · sin actividad registrada hace {e['quieto_min']} min" if e.get("quieto_min", 0) > UMBRAL else ""
    return f"{e['tarea']} · {e['fase']} · fases que faltan: {faltan} · {est} · n={e.get('n', 0)} {e.get('nivel', '')}{q}"


# =========================================================================== L: status line y escrituras

def escribir_atomico(ruta, texto, reintentos=50, espera=0.01):
    """Temporal + os.replace, con reintento si Windows niega el rename porque un lector tiene el fichero
    abierto. Devuelve los reintentos usados; lanza PermissionError si se agotan."""
    tmp = f"{ruta}.{os.getpid()}.{threading.get_ident()}.tmp"
    with open(tmp, "w", encoding="utf-8", newline="\n") as f:
        f.write(texto)
        f.flush()
        os.fsync(f.fileno())
    for i in range(reintentos + 1):
        try:
            os.replace(tmp, ruta)
            return i
        except PermissionError:
            if i == reintentos:
                os.remove(tmp)
                raise
            time.sleep(espera)


def escribir_estado(carpeta, datos, reintentos=50):
    """estado.json (D5a): el valor derivado que lee la status line; antes guarda el actual como el
    ultimo bueno (estado.anterior.json). Cada paso es un rename atomico."""
    ruta = os.path.join(carpeta, "estado.json")
    usados = 0
    if os.path.exists(ruta):
        try:
            with open(ruta, encoding="utf-8") as f:
                actual = f.read()
            json.loads(actual)
            usados += escribir_atomico(os.path.join(carpeta, "estado.anterior.json"), actual, reintentos)
        except (ValueError, OSError):
            pass
    usados += escribir_atomico(ruta, json.dumps(datos, ensure_ascii=False, sort_keys=True), reintentos)
    return usados


def leer_estado(carpeta, ahora=None):
    """Lectura O(1) de la status line: un fichero; si falta o no se puede leer, el ultimo valor bueno con
    su antiguedad. Devuelve (datos, origen, ficheros_abiertos, bytes_leidos)."""
    abiertos, leidos = 0, 0
    for nombre, origen in (("estado.json", "actual"), ("estado.anterior.json", "ultimo-bueno")):
        try:
            with open(os.path.join(carpeta, nombre), encoding="utf-8") as f:
                abiertos += 1
                txt = f.read()
            leidos += len(txt.encode("utf-8"))
            d = json.loads(txt)
            if origen == "ultimo-bueno" and ahora is not None:
                d["antiguedad_min"] = round((ahora - d.get("calculado", ahora)) / 60)
            return d, origen, abiertos, leidos
        except (OSError, ValueError):
            continue
    return None, "nada", abiertos, leidos


HIJO_COSTE = r'''
import os, sys
ruta, fuente, n, modo = sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4]
for i in range(n):
    fila = f"| 2026-11-15T10:00:00Z | revision | {fuente} | {i} |\n"
    if modo == "ingenuo":   # leer, anadir, reescribir el tarea.md entero (lo que haria un CLI sin cuidado)
        txt = open(ruta, encoding="utf-8").read()
        open(ruta, "w", encoding="utf-8").write(txt + fila)
    else:                   # un fichero por fuente, solo anadir (O_APPEND)
        with open(os.path.join(ruta, f"coste-{fuente}.md"), "a", encoding="utf-8") as f:
            f.write(fila)
'''


def hijo(codigo, *args):
    return subprocess.Popen([sys.executable, "-I", "-B", "-c", codigo, *[str(a) for a in args]],
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE)


# =========================================================================== S: start, predecesoras y solape

ESTADO = "taskcodetwo/tareas"


def tarea_md(t, estado, deps=(), zona=None, asignado="null"):
    z = "null" if zona is None else "[" + ", ".join(f'"{x}"' for x in zona) + "]"
    return (f'---\nformato: 2\nid: "{t}"\nestado: "{estado}"\nasignado_a: "{asignado}"\nzonas: {z}\n'
            f'dependencias: [{", ".join(deps)}]\n---\n## Objetivo\n\nx\n')


class MundoS:
    """Remoto bare local, dos maquinas (Carlos y un companero) y la rama de estado."""

    def __init__(self, raiz):
        self.raiz = raiz
        self.remoto = os.path.join(raiz, "remoto.git")
        os.makedirs(raiz)
        git(raiz, "init", "-q", "--bare", self.remoto)
        semilla = os.path.join(raiz, "semilla")
        git(raiz, "init", "-q", semilla)
        open(os.path.join(semilla, "README.md"), "w").write("x\n")
        os.makedirs(os.path.join(semilla, "core", "a"))
        os.makedirs(os.path.join(semilla, "core", "b"))
        open(os.path.join(semilla, "core", "a", "uno.rs"), "w").write("// a\n")
        open(os.path.join(semilla, "core", "b", "dos.rs"), "w").write("// b\n")
        git(semilla, "add", "-A")
        git(semilla, "commit", "-q", "-m", "inicio")
        git(semilla, "remote", "add", "origin", self.remoto)
        git(semilla, "push", "-q", "origin", "HEAD:develop")
        git(semilla, "checkout", "-q", "--orphan", "estado")
        git(semilla, "rm", "-q", "-rf", ".")
        self.semilla = semilla
        self.escribir_estado({"TASK-001": tarea_md("TASK-001", "en-curso", asignado="carlos"),
                              "TASK-002": tarea_md("TASK-002", "planificada", ["TASK-001"])}, quien=PERSONA)
        self.maq = {}
        for nombre in ("carlos", "companero"):
            d = os.path.join(raiz, nombre)
            git(raiz, "clone", "-q", "-b", "develop", self.remoto, d)
            git(d, "fetch", "-q", "origin", f"+{ESTADO}:refs/remotes/origin/{ESTADO}")
            self.maq[nombre] = d

    def escribir_estado(self, ficheros, quien=AGENTE):
        """Commit en la rama de estado y push sin force (F1-2); lo hace la semilla en nombre de quien sea."""
        d = self.semilla
        for t, txt in ficheros.items():
            os.makedirs(os.path.join(d, t), exist_ok=True)
            open(os.path.join(d, t, "tarea.md"), "w", encoding="utf-8", newline="\n").write(txt)
        git(d, "add", "-A")
        git(d, "commit", "-q", "-m", "estado", quien=quien)
        git(d, "push", "-q", "origin", f"HEAD:{ESTADO}")

    def trabajo(self, maquina, t, ficheros, merge=True, empujar=True):
        """Rama de la tarea con un cambio; con `merge`, merge --no-ff en develop con trailer «Tarea:»."""
        d = self.maq[maquina]
        git(d, "fetch", "-q", "origin")
        git(d, "checkout", "-q", "-B", f"feature/{t.lower()}", "origin/develop")
        for r, txt in ficheros.items():
            os.makedirs(os.path.dirname(os.path.join(d, r)), exist_ok=True)
            open(os.path.join(d, r), "a", encoding="utf-8").write(txt)
        git(d, "add", "-A")
        git(d, "commit", "-q", "-m", f"feat({t}): trabajo")
        rama = g1(d, "rev-parse", "HEAD")
        git(d, "push", "-q", "origin", f"feature/{t.lower()}")
        git(d, "checkout", "-q", "-B", "develop", "origin/develop")
        if merge:
            git(d, "merge", "-q", "--no-ff", "-m", f"merge(feature): feature/{t.lower()} -> develop\n\nTarea: {t}", rama)
            if empujar:
                git(d, "push", "-q", "origin", "develop")
        return rama


def leer_estado_remoto(d, ref=f"origin/{ESTADO}"):
    out = {}
    for f in g1(d, "ls-tree", "-r", "--name-only", ref).splitlines():
        if f.endswith("/tarea.md"):
            txt = g1(d, "show", f"{ref}:{f}")
            fm = frontmatter(txt) or {}
            z = fm.get("zonas", "null")
            out[f.split("/")[0]] = {"estado": fm.get("estado"), "asignado": fm.get("asignado_a"),
                                    "deps": re.findall(r"TASK-\d+", fm.get("dependencias", "")),
                                    "zonas": None if z == "null" else re.findall(r'"([^"]+)"', z)}
    return out


def mergeada(d, t, ref="origin/develop"):
    """La predecesora tiene su merge en develop: un merge del primer padre con trailer «Tarea: TASK-NNN»."""
    log = g1(d, "log", "--first-parent", "--merges", "--format=%B%x1e", ref)
    return any(re.search(rf"^Tarea: {t}\s*$", b, re.M) for b in log.split("\x1e"))


def puede_empezar(d, t, mutacion=None):
    """Precondiciones de `start` (D3a; codigo en F3-6): fetch de develop y de la rama de estado; cada
    predecesora terminada en la rama de estado Y con su merge en develop; sin red se niega."""
    if mutacion != "sin-fetch":
        r = git(d, "fetch", "-q", "origin", "+develop:refs/remotes/origin/develop",
                f"+{ESTADO}:refs/remotes/origin/{ESTADO}", ok=(0, 1, 128))
        if r[0] != 0 and mutacion != "sin-red-permite":
            return False, "sin-red", "no se puede traer develop ni la rama de estado: no se arranca a ciegas"
    est = leer_estado_remoto(d)
    for p in est[t]["deps"]:
        e = est.get(p, {}).get("estado")
        if e != "terminada":
            return False, "predecesora-sin-terminar", f"{p} esta {e} (la lleva {est.get(p, {}).get('asignado')})"
        if mutacion != "sin-merge" and not mergeada(d, p):
            return False, "predecesora-sin-merge", f"{p} dice terminada pero su merge no esta en origin/develop"
    return True, "ok", ""


def avisos_solape(d, t, zona_nueva, mutacion=None):
    """Aviso de solape (F3-11, RR-12): tareas en curso cuya zona declarada solapa, las que no declaran
    zona (choca con todas, D4a) y las que tocan en su rama ficheros fuera de su zona (cruce con el diff
    real). Siempre «aviso», nunca garantia."""
    git(d, "fetch", "-q", "origin", f"+{ESTADO}:refs/remotes/origin/{ESTADO}")
    git(d, "fetch", "-q", "origin")
    est = leer_estado_remoto(d)
    ramas = g1(d, "branch", "-r", "--format=%(refname:short)").split()
    av = []
    for u, x in sorted(est.items()):
        if u == t or x["estado"] != "en-curso":
            continue
        sin = x["zonas"] is None or zona_nueva is None
        if not (mutacion == "sin-zona-no-choca" and sin) and solapan(zona_nueva, x["zonas"]):
            av.append(f"{u} ({x['asignado']}): " + ("sin zona declarada, choca con todas" if sin else "su zona declarada solapa"))
            continue
        rama = next((r for r in ramas if r.endswith(f"feature/{u.lower()}")), None)
        if rama and x["zonas"] is not None:
            base = g1(d, "merge-base", rama, "origin/develop")
            fuera = [f for f in g1(d, "diff", "--name-only", base, rama).split() if not solapan([f], x["zonas"])]
            if fuera and mutacion != "sin-cruce-diff" and solapan(zona_nueva, fuera):
                av.append(f"{u} ({x['asignado']}): su rama toca fuera de su zona: {', '.join(fuera)}")
    return av


def comprobar_cas(d, rama, esperado):
    """Aviso de cambios ajenos (CAS de F1-16): la cabeza remota de la rama es la que anoto el CLI."""
    git(d, "fetch", "-q", "origin")
    actual = g1(d, "rev-parse", f"origin/{rama}")
    return actual == esperado, actual


# =========================================================================== secciones

def fmt_m(x):
    if x is None:
        return "-"
    return (f"n={x['n']} mediana {coma(x['med'], 2)} p80 {coma(x['p80'], 2)} p90 {coma(x['p90'], 2)} "
            f"±50 % {pct(x['dentro50'], 1)} % error medio {round(x['mae'])} min ({pct(x['mape'], 1)} %) sesgo {coma(x['sesgo'], 2)}")


def fila_err(ambito, nombre, x):
    if x is None:
        return f"| {ambito} | {nombre} | 0 | - | - | - | - | - | - | - |"
    return (f"| {ambito} | {nombre} | {x['n']} | {coma(x['med'], 2)} | {coma(x['p80'], 2)} | {coma(x['p90'], 2)} | "
            f"{pct(x['dentro50'], 1)} % | {round(x['mae'])} | {pct(x['mape'], 1)} % | {coma(x['sesgo'], 2)} |")


def seccion_h(repos):
    sep("H: lector del historico (tolerante, descartes contados) y tareas troceadas")
    H, tabla = {}, ["| Repo | Commit | Terminadas | Descartadas | Usables | Grupos de troceo | Muestras tras agrupar | Con «Estimada» de la 0.16 | De ellas, leída con tolerancia | Con «Real» de la 0.16 |",
                    "|---|---|---|---|---|---|---|---|---|---|"]
    for r in repos:
        ms, desc, malas = leer_historico(r)
        term = len([f for f in r.ficheros if f.endswith("/tarea.md") and f.split("/")[1] == "04-terminadas"])
        g = agrupar(ms)
        grupos = {}
        for m in ms:
            grupos.setdefault(g[m["id"]], []).append(m["id"])
        multi = {k: v for k, v in sorted(grupos.items()) if len(v) > 1}
        S = muestras_agrupadas(ms)
        n016 = sum(1 for m in ms if m["est_016"] is not None)
        tol = sum(1 for m in ms if m["fuente_016"] == "tolerante")
        r016 = sum(1 for m in ms if m["real_016"] is not None)
        nd = sum(len(v) for v in desc.values())
        print(f"{r.nombre}@{r.fijo[:9]}: terminadas {term} ; descartadas {nd} {dict((k, len(v)) for k, v in sorted(desc.items()))} ; "
              f"filas ilegibles {malas} ; usables {len(ms)} ; versiones {sorted({v for m in ms for v in m['versiones']})}")
        print(f"  con merge de su rama {sum(m['con_merge'] for m in ms)} ; carriles (proxy del diff) "
              f"{dict((c, sum(1 for m in ms if m['carril'] == c)) for c in ORDEN_CARRIL)} ; complejidad declarada "
              f"{sum(1 for m in ms if m['complejidad'])}")
        print(f"  «Estimada» de la 0.16: {n016} (en ## Duracion {sum(1 for m in ms if m['fuente_016'] == 'duracion')}, "
              f"solo en el plan {sum(1 for m in ms if m['fuente_016'] == 'plan')}, leidas con tolerancia {tol}) ; «Real» de la 0.16: {r016}")
        for k, v in multi.items():
            print(f"  grupo de troceo {k}: {', '.join(v)}")
        print(f"  muestras tras agrupar: {len(S)}")
        H[r.nombre] = {"ms": ms, "S": S, "multi": multi, "desc": desc, "term": term, "n016": n016, "r016": r016, "tol": tol}
        tabla.append(f"| {r.nombre} | `{r.fijo[:9]}` | {term} | {nd} | {len(ms)} | {len(multi)} ({sum(len(v) for v in multi.values())} tareas) | "
                     f"{len(S)} | {n016} | {tol} | {r016} |")
    return H, tabla


def buscar(ms, t):
    return next(m for m in ms if m["id"] == t)


def seccion_r(H):
    sep("R: real recalculado desde Transiciones con corte de huecos (D6a) frente a «Real ... sin pausas» de la 0.16")
    tw, odt = H["ToolsaWeb"]["ms"], H["OpenDockTrading"]["ms"]
    c096, c296 = buscar(odt, "TASK-096"), buscar(tw, "TASK-296")
    res = {}
    for nombre, m in (("TASK-096 (OpenDockTrading)", c096), ("TASK-296 (ToolsaWeb)", c296)):
        r0, r45 = real(m, None, con_pausas=False), real(m)
        print(f"{nombre}: «Real» de la 0.16 {m['real_016']} min ; reloj start→finish {round(r0['total'])} min ; "
              f"recalculado (huecos > {UMBRAL} min a {UMBRAL}, pausas declaradas fuera) {round(r45['total'])} min "
              f"(implementacion {round(r45['implementacion'])}, rondas {round(r45['rondas'])}, cierre {round(r45['cierre'])}) ; "
              f"con el hueco cortado a 0 {round(real(m, UMBRAL, 'corte')['total'])} ; sin respetar la pausa {round(real(m, UMBRAL, con_pausas=False)['total'])}")
    res["096"] = (c096["real_016"], round(real(c096)["total"]), round(real(c096)["implementacion"]),
                  round(real(c096, UMBRAL, "corte")["implementacion"]), round(real(c096, UMBRAL, "corte")["total"]))
    res["296"] = (c296["real_016"], round(real(c296)["total"]), round(real(c296, UMBRAL, con_pausas=False)["total"]))
    # huecos
    for nom in ("ToolsaWeb", "OpenDockTrading"):
        hs = []
        for m in H[nom]["ms"]:
            ev = sorted({e for e in set(m["commits"]) | {c[0] for c in m["coste"]} | {t for t, _ in m["trans"]}
                         if m["start"] <= e <= m["finish"]} | {m["start"], m["finish"]})
            hs += [(b - a) / 60 for a, b in zip(ev, ev[1:])]
        mayores = sum(1 for h in hs if h > UMBRAL)
        afect = sum(1 for m in H[nom]["ms"] if abs(real(m)["total"] - real(m, None)["total"]) > 0.5)
        iguales = [m for m in H[nom]["ms"] if m["real_016"] is not None]
        coinc = sum(1 for m in iguales if abs(m["real_016"] - real(m, None)["total"]) <= 2)
        print(f"{nom}: huecos entre eventos dentro de start→finish {len(hs)} ; p50 {coma(cuantil(hs, .5))} min, p90 "
              f"{coma(cuantil(hs, .9))}, p95 {coma(cuantil(hs, .95))} ; mayores de {UMBRAL} min {mayores} ; tareas que cambian con el corte "
              f"{afect} de {len(H[nom]['ms'])} ; «Real» de la 0.16 igual (±2 min) al reloj start→finish menos pausas: {coinc} de {len(iguales)}")
        res[nom] = {"huecos": len(hs), "mayores": mayores, "afect": afect, "coinc": coinc, "con016": len(iguales)}
    tabla = ["| Regla | Mediana del real, ToolsaWeb (min) | Media, ToolsaWeb | Mediana, OpenDockTrading | Media, OpenDockTrading | TASK-096 de OpenDockTrading | TASK-296 de ToolsaWeb |",
             "|---|---|---|---|---|---|---|"]
    variantes = [(f"huecos > {UMBRAL} min a {UMBRAL} min (D6a)", UMBRAL, "tope", True),
                 (f"huecos > {UMBRAL} min a 0 min", UMBRAL, "corte", True),
                 ("sin corte (reloj menos pausas, como la 0.16)", None, "tope", True)]
    for nombre, u, modo, cp in variantes:
        a = [real(m, u, modo, cp)["total"] for m in tw]
        b = [real(m, u, modo, cp)["total"] for m in odt]
        fila = (f"| {nombre} | {round(mediana(a))} | {round(statistics.mean(a))} | {round(mediana(b))} | {round(statistics.mean(b))} | "
                f"{round(real(c096, u, modo, cp)['total'])} | {round(real(c296, u, modo, cp)['total'])} |")
        tabla.append(fila)
    print("\ntabla-corte calculada:")
    for l in tabla:
        print(l)
    return res, tabla


def seccion_e(H, rng):
    sep("E: estimacion fuera de muestra (rolling origin por repo y cruzado), fases medibles y lineas base (D7a, D8a)")
    S = {n: H[n]["S"] for n in H}
    tabla = ["| Ámbito | Estimador | n | Mediana del error relativo | p80 | p90 | Dentro de ±50 % | Error medio (min) | Error medio relativo | Sesgo (estimado/real) |",
             "|---|---|---|---|---|---|---|---|---|---|"]
    res = {}
    for nom, otro, corto in (("ToolsaWeb", "OpenDockTrading", "TW"), ("OpenDockTrading", "ToolsaWeb", "ODT")):
        f = evaluar(S[nom], S[nom], S[otro])
        con = [x for x in f if x["e"]]
        niveles = {lv: sum(1 for x in con if x["e"]["nivel"] == lv) for lv in NIVELES}
        print(f"{nom}: muestras {len(f)} ; con estimacion {len(con)} ; por nivel de la cascada {niveles}")
        comun = tabla_metricas(f, lambda x: x["e"] is not None and x["ingenua"] is not None)
        sub = tabla_metricas(f, lambda x: x["e"] is not None and x["016"] is not None)
        cal = []
        for x in f:
            o = x["o"]
            if x["e"] is None or o["est_016"] is None:
                continue
            prev = [m for m in S[nom] if m["finish"] <= o["approve"] and m["est_016"] is not None]
            if len(prev) >= 3:
                k = mediana([m["real"]["total"] / m["est_016"] for m in prev])
                cal.append((o["est_016"] * k, o["real"]["total"], x["e"]["total"][0], o["est_016"]))
        mcal = metricas([(a, b) for a, b, _, _ in cal])
        mcal_mod = metricas([(c, b) for _, b, c, _ in cal])
        mcal_016 = metricas([(d, b) for _, b, _, d in cal])
        for k in ("modelo", "ingenua", "complejidad"):
            print(f"  {k:12s} (comun) {fmt_m(comun[k])}")
        for k in ("modelo", "016", "ingenua"):
            print(f"  {k:12s} (con «Estimada» de la 0.16) {fmt_m(sub[k])}")
        print(f"  0.16 calibrada con la razon real/estimada mediana de las anteriores (n>=3): {fmt_m(mcal)}")
        print(f"     en las mismas: modelo {fmt_m(mcal_mod)} ; 0.16 cruda {fmt_m(mcal_016)}")
        pb_ing = bootstrap_mejor(f, "modelo", "ingenua", lambda x: x["e"] is not None and x["ingenua"] is not None, rng)
        pb_016 = bootstrap_mejor(f, "modelo", "016", lambda x: x["e"] is not None and x["016"] is not None, rng)
        print(f"  remuestreo pareado (2000): el modelo tiene menor mediana que la ingenua en el {pct(pb_ing, 1)} % ; "
              f"que la 0.16 en el {pct(pb_016, 1)} %")
        cov = [x["o"]["real"]["total"] <= x["e"]["total"][1] for x in con]
        print(f"  cobertura del p80 (real <= p80 estimado): {sum(cov)} de {len(cov)} ({pct(sum(cov), len(cov))} %)")
        dentro = evaluar(S[nom], S[nom], S[otro], dentro=True)
        md = tabla_metricas(dentro, lambda x: x["e"] is not None and x["ingenua"] is not None)["modelo"]
        print(f"  control dentro de muestra (fuga): {fmt_m(md)}")
        comprueba("E", md["med"] < comun["modelo"]["med"], f"{nom}: dentro de muestra el error no baja: el guion tiene fuga")
        cruz = evaluar(S[nom], [], S[otro])
        mc = tabla_metricas(cruz, lambda x: x["e"] is not None)
        print(f"  cruzado (solo con {otro}, anteriores a cada tarea): estimables {sum(1 for x in cruz if x['e'])} de {len(cruz)} ; {fmt_m(mc['modelo'])}")
        mc016 = tabla_metricas(cruz, lambda x: x["e"] is not None and x["016"] is not None)
        # filas de la tabla
        tabla.append(fila_err(f"{nom}, por repo", "cascada (modelo)", comun["modelo"]))
        tabla.append(fila_err(f"{nom}, por repo", "mediana ingenua del repo", comun["ingenua"]))
        tabla.append(fila_err(f"{nom}, por repo", "mediana por complejidad" if corto == "TW" else "mediana por complejidad (= ingenua: sin complejidad)", comun["complejidad"]))
        tabla.append(fila_err(f"{nom}, con «Estimada»", "cascada (modelo)", sub["modelo"]))
        tabla.append(fila_err(f"{nom}, con «Estimada»", "«Estimada» de la 0.16", sub["016"]))
        tabla.append(fila_err(f"{nom}, con «Estimada»", "mediana ingenua del repo", sub["ingenua"]))
        tabla.append(fila_err(f"{nom}, con «Estimada» y 3 anteriores", "0.16 calibrada", mcal))
        tabla.append(fila_err(f"{nom}, cruzado", f"cascada solo con {otro}", mc["modelo"]))
        tabla.append(fila_err(f"{nom}, cruzado, con «Estimada»", "«Estimada» de la 0.16", mc016["016"]))
        # fases
        fases = []
        for fase in FASES:
            er = [abs(x["e"][fase][0] - x["o"]["real"][fase]) / max(x["o"]["real"][fase], 5) for x in con]
            ab = [abs(x["e"][fase][0] - x["o"]["real"][fase]) for x in con]
            rm = mediana([x["o"]["real"][fase] for x in con])
            print(f"  fase {fase}: real mediano {round(rm)} min ; error absoluto mediano {round(mediana(ab))} min, p80 "
                  f"{round(cuantil(ab, .8))} ; error relativo mediano (denominador >= 5 min) {coma(mediana(er), 2)}")
            fases.append(f"| {nom} | {fase} | {len(con)} | {round(rm)} | {round(mediana(ab))} | {round(cuantil(ab, .8))} | {coma(mediana(er), 2)} |")
        def con_coste(ms):
            return [m["tokens"] for m in ms if m["tokens"] > 0]
        tok_m = metricas([(mediana(con_coste(x["e"]["muestras"])), x["o"]["tokens"]) for x in con
                          if x["o"]["tokens"] > 0 and con_coste(x["e"]["muestras"])])
        tok_i = metricas([(mediana(con_coste(disponibles(S[nom], x["o"]))), x["o"]["tokens"]) for x in con
                          if x["o"]["tokens"] > 0 and con_coste(disponibles(S[nom], x["o"]))])
        for nombre, x in (("cascada", tok_m), ("mediana ingenua", tok_i)):
            print(f"  tokens (total de ## Coste, solo tareas con coste), {nombre}: n={x['n']} mediana del error {coma(x['med'], 2)} ; "
                  f"p80 {coma(x['p80'], 2)} ; ±50 % {pct(x['dentro50'], 1)} % ; sesgo {coma(x['sesgo'], 2)}")
        rondas = [x["o"]["rondas_n"] for x in con]
        print(f"  rondas por muestra: mediana {mediana(rondas)} ; p80 {cuantil(rondas, .8)} ; maximo {max(rondas)}")
        # recalculo tras la implementacion (tamano del diff)
        rest = {}
        for nombre, tamf in (("criterios", tam), ("diff", tam_diff)):
            pares = []
            for o in S[nom]:
                p = [m for m in S[nom] if m["finish"] <= o["reviews"][0] and m["id"] != o["id"]]
                s = [m for m in S[otro] if m["finish"] <= o["reviews"][0]]
                e = estimar(o, p, s, tamf=tamf)
                if e:
                    pares.append((mediana([m["real"]["rondas"] + m["real"]["cierre"] for m in e["muestras"]]),
                                  o["real"]["rondas"] + o["real"]["cierre"]))
            rest[nombre] = metricas(pares)
            print(f"  recalculo en la 1.a review, rondas+cierre con tamano por {nombre}: {fmt_m(rest[nombre])}")
        # variantes
        var = {}
        for nombre, nv in (("cascada entera", NIVELES), ("sin tamano", NIVELES[1:]), ("solo carril", NIVELES[2:]),
                           ("solo repo (con semilla)", NIVELES[3:]),
                           ("semilla del carril antes que el repo", NIVELES[:3] + ["semilla-carril", "repo", "semilla"])):
            var[nombre] = tabla_metricas(evaluar(S[nom], S[nom], S[otro], niveles=nv),
                                         lambda x: x["e"] is not None and x["ingenua"] is not None)["modelo"]
        sin_agr = muestras_agrupadas(H[nom]["ms"], agrupar_troceo=False)
        otro_sin = muestras_agrupadas(H[otro]["ms"], agrupar_troceo=False)
        var["sin agrupar las troceadas"] = tabla_metricas(evaluar(sin_agr, sin_agr, otro_sin),
                                                          lambda x: x["e"] is not None and x["ingenua"] is not None)["modelo"]
        for u in (30, 60, 90, None):
            su = muestras_agrupadas(H[nom]["ms"], u)
            so = muestras_agrupadas(H[otro]["ms"], u)
            var[f"umbral {u} min" if u else "sin corte"] = tabla_metricas(evaluar(su, su, so),
                                                                           lambda x: x["e"] is not None and x["ingenua"] is not None)["modelo"]
        for k, v in var.items():
            print(f"  variante {k}: {fmt_m(v)}")
        res[nom] = {"comun": comun, "sub": sub, "cal": mcal, "cruz": mc, "dentro": md, "cov": (sum(cov), len(cov)),
                    "pb_ing": pb_ing, "pb_016": pb_016, "fases": fases, "rest": rest, "var": var, "niveles": niveles,
                    "n": len(f), "con": len(con), "cal_mod": mcal_mod, "cal_016": mcal_016, "tok": (tok_m, tok_i)}
    print("\ntabla-error calculada:")
    for l in tabla:
        print(l)
    tf = ["| Repo | Fase | n | Real mediano (min) | Error absoluto mediano (min) | p80 del error absoluto (min) | Error relativo mediano |",
          "|---|---|---|---|---|---|---|"] + res["ToolsaWeb"]["fases"] + res["OpenDockTrading"]["fases"]
    tv = ["| Variante | ToolsaWeb: mediana | ToolsaWeb: ±50 % | OpenDockTrading: mediana | OpenDockTrading: ±50 % |", "|---|---|---|---|---|"]
    for k in res["ToolsaWeb"]["var"]:
        a, b = res["ToolsaWeb"]["var"][k], res["OpenDockTrading"]["var"][k]
        tv.append(f"| {k} | {coma(a['med'], 2)} | {pct(a['dentro50'], 1)} % | {coma(b['med'], 2)} | {pct(b['dentro50'], 1)} % |")
    print("\ntabla-fases calculada:")
    for l in tf:
        print(l)
    print("\ntabla-variantes calculada:")
    for l in tv:
        print(l)
    return res, tabla, tf, tv


def zona_ejemplo(t):
    """Zonas de ejemplo para la PoC (TaskCodeTwo aun no declara `zonas`): una por fase del plan; las de la
    Fase 9 (E2E de todo el flujo) sin zona, para ver que van en serie."""
    m = re.match(r"F(\d+)-", t["titulo"])
    if not m or m.group(1) == "9":
        return None
    return [f"fase-{m.group(1)}/**"]


def seccion_d(repo_tc2, repo_proto, H, rng):
    sep("D: `resumen` (dashboard) y `repartir --personas N` sobre TaskCodeTwo, y el prototipo de la sesion")
    # prototipo: plan o 55 min, sin zonas
    P_ = leer_tc2(repo_proto)
    pend_p = sorted(t for t, x in P_.items() if x["estado"] != "terminada")
    dur_p = {t: P_[t]["plan_min"] or POR_DEFECTO for t in pend_p}
    deps_p = {t: P_[t]["deps"] for t in pend_p}
    proto = {n: repartir(set(pend_p), dur_p, deps_p, {}, n, respetar_zonas=False) for n in (1, 2, 3, 4)}
    print(f"prototipo en {PROTO_TC2[:9]}: tareas {len(P_)}, pendientes {len(pend_p)}, con plan {[t for t in pend_p if P_[t]['plan_min']]} ; "
          f"camino critico {coma(proto[1]['camino'] / 60)} h ; 1-4 personas "
          + " / ".join(coma(proto[n]["total"] / 60) for n in (1, 2, 3, 4)) + " h")
    for n, v in zip((1, 2, 3, 4), (77.1, 38.6, 25.8, 19.3)):
        comprueba("D", coma(proto[n]["total"] / 60) == coma(v), f"el prototipo con {n} personas no da {v} h")
    comprueba("D", coma(proto[1]["camino"] / 60) == "16,6", "el camino critico del prototipo no da 16,6 h")
    # estado de hoy
    T = leer_tc2(repo_tc2)
    ms_tc2, desc_tc2, _ = leer_historico(repo_tc2)
    S_tc2 = muestras_agrupadas(ms_tc2)
    semilla = H["ToolsaWeb"]["S"] + H["OpenDockTrading"]["S"]
    print(f"TaskCodeTwo@{FIJO_TC2[:9]}: tareas {len(T)} ; terminadas {sum(1 for x in T.values() if x['estado'] == 'terminada')} ; "
          f"historico usable {len(ms_tc2)} (descartes {dict((k, len(v)) for k, v in desc_tc2.items())}) ; "
          f"carril de las terminadas {dict((c, sum(1 for m in ms_tc2 if m['carril'] == c)) for c in ORDEN_CARRIL)}")
    pend = sorted(t for t, x in T.items() if x["estado"] != "terminada")
    est, dur, dist, tok = {}, {}, {}, {}
    for t in pend:
        x = T[t]
        obj = {"carril": carril_tc2(x), "complejidad": None, "criterios": x["criterios"], "id": t}
        e = estimar(obj, S_tc2, semilla)
        est[t] = e
        if x["estado"] == "en-curso":
            fase, _ = fase_actual(x["trans"])
            tr, _ = transcurrido(x["trans"], x["coste"], repo_tc2.commits.get(t, []), fase, AHORA)
            rs = restante(e, fase, tr)
            dur[t] = rs["med"]
            act = None if fase == "diseno" else fase
            sig = {"diseno": FASES, "implementacion": FASES, "rondas": ["rondas", "cierre"]}[fase]
            dist[t] = [max(0.0, sum(m["real"][k] for k in sig) - (min(tr, m["real"][act]) if act else 0)) for m in e["muestras"]]
        else:
            dur[t] = e["total"][0]
            dist[t] = [m["real"]["total"] for m in e["muestras"]]
        tok[t] = [m["tokens"] for m in e["muestras"]]
    niveles = {}
    for t in pend:
        k = f"{carril_tc2(T[t])}: {est[t]['nivel']} (n={est[t]['n']})"
        niveles[k] = niveles.get(k, 0) + 1
    print(f"pendientes {len(pend)} ; nivel de la cascada usado: {dict(sorted(niveles.items()))}")
    # resumen por sprint
    filas = ["| Sprint | Terminadas | Pendientes | Tiempo real (h) | Tokens (M) | Falta, mediana (h) | Falta, p80 (h) | Tokens que faltan, mediana (M) |",
             "|---|---|---|---|---|---|---|---|"]
    real_tc2 = {m["id"]: real(m)["total"] for m in ms_tc2}
    sprints = sorted({x["sprint"] for x in T.values()}, key=lambda s: int(s) if s.isdigit() else 99)
    tot = {"term": 0, "pend": 0, "real": 0.0, "tok": 0}
    for sp in sprints + ["Total"]:
        ids = sorted(t for t, x in T.items() if sp == "Total" or x["sprint"] == sp)
        term = [t for t in ids if T[t]["estado"] == "terminada"]
        pe = [t for t in ids if T[t]["estado"] != "terminada"]
        r_ = sum(real_tc2.get(t, 0) for t in term)
        tk = sum(T[t]["tokens"] for t in ids)
        med, p80 = suma_remuestreada([dist[t] for t in pe], random.Random(SEMILLA)) if pe else (0, 0)
        tkm, _ = suma_remuestreada([tok[t] for t in pe], random.Random(SEMILLA)) if pe else (0, 0)
        filas.append(f"| {sp} | {len(term)} | {len(pe)} | {coma(r_ / 60)} | {coma(tk / 1e6)} | {coma(med / 60)} | {coma(p80 / 60)} | {coma(tkm / 1e6)} |")
    suma_med = sum(dur[t] for t in pend)
    print("\ntabla-resumen calculada:")
    for l in filas:
        print(l)
    med_t, p80_t = suma_remuestreada([dist[t] for t in pend], random.Random(SEMILLA))
    print(f"total que falta: suma de medianas {coma(suma_med / 60)} h ; mediana de la suma remuestreada {coma(med_t / 60)} h ; p80 {coma(p80_t / 60)} h")
    # repartir
    deps = {t: T[t]["deps"] for t in pend}
    zonas_ej = {t: zona_ejemplo(T[t]) for t in pend}
    sin_zona_decl = {}
    rep = {}
    for n in (1, 2, 3, 4):
        rep[n] = {"libre": repartir(set(pend), dur, deps, {}, n, respetar_zonas=False),
                  "d4a": repartir(set(pend), dur, deps, sin_zona_decl, n),
                  "ejemplo": repartir(set(pend), dur, deps, zonas_ej, n)}
    sin = sorted(t for t in pend if zonas_ej[t] is None)
    print(f"zonas de ejemplo: {len(pend) - len(sin)} tareas con una zona por fase del plan ; {len(sin)} sin zona (Fase 9 y sin F-id): {', '.join(sin)}")
    tabla = ["| Personas | Prototipo de la sesión (h) | Estimación de F1-17, sin mirar zonas (h) | D4a sin zonas declaradas (h) | D4a con zonas de ejemplo (h) |",
             "|---|---|---|---|---|"]
    for n in (1, 2, 3, 4):
        tabla.append(f"| {n} | {coma(proto[n]['total'] / 60)} | {coma(rep[n]['libre']['total'] / 60)} | "
                     f"{coma(rep[n]['d4a']['total'] / 60)} | {coma(rep[n]['ejemplo']['total'] / 60)} |")
    tabla.append(f"| camino crítico | {coma(proto[1]['camino'] / 60)} | {coma(rep[1]['libre']['camino'] / 60)} | "
                 f"{coma(rep[1]['d4a']['camino'] / 60)} | {coma(rep[1]['ejemplo']['camino'] / 60)} |")
    print("\ntabla-reparto calculada:")
    for l in tabla:
        print(l)
    # comprobaciones del reparto
    comprueba("D", abs(rep[1]["libre"]["total"] - sum(dur.values())) < 1e-6, "con 1 persona el reparto no es la suma")
    for n in (2, 3, 4):
        comprueba("D", abs(rep[n]["d4a"]["total"] - rep[1]["d4a"]["total"]) < 1e-6, "sin zonas declaradas el reparto paraleliza (D4a)")
        comprueba("D", rep[n]["libre"]["total"] >= rep[1]["libre"]["camino"] - 1e-6, "el reparto baja del camino critico")
    lista = rep[2]["ejemplo"]["listas"]
    for p in sorted(lista):
        print(f"  persona {p + 1} (2 personas, zonas de ejemplo): {len(lista[p])} tareas, primeras {', '.join(lista[p][:6])}")
    # una tarea sin zona en serie: en el reparto de ejemplo, ninguna otra corre mientras una sin zona corre
    solapes = 0
    for n in (2, 3, 4):
        solapes += serie_violada(rep[n]["ejemplo"], zonas_ej)
    print(f"tramos en que una tarea sin zona corre a la vez que otra (2-4 personas, zonas de ejemplo): {solapes}")
    comprueba("D", solapes == 0, "una tarea sin zona corre en paralelo")
    return {"proto": proto, "rep": rep, "tabla": tabla, "resumen": filas, "pend": len(pend), "T": T, "sin": sin,
            "commits_075": repo_tc2.commits.get("TASK-075", []), "niveles_tc2": niveles,
            "med_t": med_t, "p80_t": p80_t, "suma_med": suma_med, "est": est, "S_tc2": S_tc2, "dur": dur, "deps": deps,
            "zonas_ej": zonas_ej, "ms_tc2": ms_tc2}


def serie_violada(rp, zonas):
    """Pares de tareas que corren a la vez con zonas que solapan (o alguna sin zona)."""
    tr = rp["tramos"]
    return sum(1 for i, (a0, a1, a) in enumerate(tr) for (b0, b1, b) in tr[i + 1:]
               if a0 < b1 and b0 < a1 and solapan(zonas.get(a), zonas.get(b)))


def seccion_s(trabajo):
    sep("S: precondiciones de `start` (D3a) y aviso de solape (D4a) con un remoto bare local")
    w = MundoS(os.path.join(trabajo, "s"))
    car, comp = w.maq["carlos"], w.maq["companero"]
    filas = []

    def caso(ide, texto, r, esperado):
        ok, motivo, det = r
        res = "arranca" if ok else f"se niega (`{motivo}`)"
        print(f"  {ide}: {texto}: {res}{' — ' + det if det else ''}")
        comprueba("S", (ok, motivo) == esperado, f"{ide}: {texto}: sale {(ok, motivo)} y se esperaba {esperado}")
        filas.append(f"| {ide} | {texto} | {res} |")
        return ok

    # S1: predecesora en curso
    caso("S1", "TASK-002 con su predecesora TASK-001 en curso", puede_empezar(car, "TASK-002"), (False, "predecesora-sin-terminar"))
    # S2: estado terminada sin merge en develop
    w.trabajo("carlos", "TASK-001", {"core/a/uno.rs": "// TASK-001\n"}, merge=False)
    w.escribir_estado({"TASK-001": tarea_md("TASK-001", "terminada", asignado="carlos")})
    caso("S2", "TASK-001 dice `terminada` en la rama de estado y su merge no está en `develop`",
         puede_empezar(car, "TASK-002"), (False, "predecesora-sin-merge"))
    # S3: el merge se empuja desde la maquina del companero
    w.trabajo("companero", "TASK-001", {"core/a/uno.rs": "// TASK-001\n"}, merge=True, empujar=True)
    caso("S3", "TASK-001 terminada y con su merge empujado a `develop`", puede_empezar(car, "TASK-002"), (True, "ok"))
    # S4: sin red
    git(car, "remote", "set-url", "origin", os.path.join(trabajo, "s", "no-existe.git"))
    caso("S4", "sin red, con la caché local diciendo que TASK-001 está terminada e integrada", puede_empezar(car, "TASK-002"),
         (False, "sin-red"))
    git(car, "remote", "set-url", "origin", w.remoto)
    # Solape
    w.escribir_estado({"TASK-003": tarea_md("TASK-003", "en-curso", zona=["core/a/**"], asignado="companero"),
                       "TASK-004": tarea_md("TASK-004", "en-curso", zona=None, asignado="carlos"),
                       "TASK-005": tarea_md("TASK-005", "planificada", zona=["core/a/x/**"]),
                       "TASK-006": tarea_md("TASK-006", "planificada", zona=["web/**"]),
                       "TASK-007": tarea_md("TASK-007", "planificada", zona=["core/b/**"])})
    w.trabajo("companero", "TASK-003", {"core/a/uno.rs": "// 3\n", "core/b/dos.rs": "// 3 fuera de su zona\n"}, merge=False)
    av = {}
    for t, z in (("TASK-005", ["core/a/x/**"]), ("TASK-006", ["web/**"]), ("TASK-007", ["core/b/**"])):
        av[t] = avisos_solape(car, t, z)
        print(f"  start {t} (zona {z}): {len(av[t])} avisos")
        for a in av[t]:
            print(f"    aviso: {a}")
    comprueba("S", any("TASK-003" in a and "solapa" in a for a in av["TASK-005"]), "TASK-005 no avisa del solape con TASK-003")
    comprueba("S", any("TASK-004" in a and "sin zona" in a for a in av["TASK-006"]), "TASK-006 no avisa de TASK-004 sin zona")
    comprueba("S", not any("TASK-003" in a for a in av["TASK-006"]), "TASK-006 avisa de TASK-003 sin solapar")
    comprueba("S", any("TASK-003" in a and "fuera de su zona" in a for a in av["TASK-007"]), "TASK-007 no ve que TASK-003 toca core/b")
    filas.append(f"| S5 | `start` de TASK-005 (zona `core/a/x/**`) con TASK-003 en curso en `core/a/**` | aviso: solapa con TASK-003 (companero) |")
    filas.append(f"| S6 | `start` de TASK-006 (zona `web/**`) con TASK-004 en curso sin zona | aviso: TASK-004 sin zona choca con todas |")
    filas.append(f"| S7 | `start` de TASK-007 (zona `core/b/**`); la rama de TASK-003 toca `core/b/dos.rs` fuera de su zona | aviso: cruce con el diff real |")
    tabla = ["| ID | Caso | Resultado |", "|---|---|---|"] + filas
    print("\ntabla-start calculada:")
    for l in tabla:
        print(l)
    return tabla


def seccion_l(trabajo, d):
    sep("L: status line O(1) sobre estado.json (D5a), escritura atomica concurrente y coste con dos escritores")
    raiz = os.path.join(trabajo, "l")
    os.makedirs(raiz)
    filas = []
    # L1: O(1) frente a O(N)
    for n in (10, 1000):
        c = os.path.join(raiz, f"n{n}")
        os.makedirs(c)
        for i in range(n):
            open(os.path.join(c, f"TASK-{i:04d}.md"), "w", encoding="utf-8").write("| x |\n" * 20)
        t0 = time.perf_counter()
        ficheros = sorted(os.listdir(c))
        datos = {"tarea": "TASK-0001", "fase": "implementacion", "faltan": ["rondas", "cierre"], "restante": [95, 140],
                 "n": n, "nivel": "repo", "calculado": AHORA}
        escribir_estado(c, datos)
        t1 = time.perf_counter()
        d_, origen, abiertos, leidos = leer_estado(c)
        t2 = time.perf_counter()
        print(f"  L1 n={n}: el CLI en la transicion recorre {len(ficheros)} tarea.md ; la status line abre {abiertos} fichero, "
              f"lee {leidos} bytes ({origen})")
        print(f"[tiempo] L1 n={n}: escritura {1000 * (t1 - t0):.2f} ms, lectura {1000 * (t2 - t1):.3f} ms", file=sys.stderr)
        comprueba("L", abiertos == 1, f"L1: la status line abre {abiertos} ficheros con n={n}")
    filas.append("| L1 | lectura de la status line con 10 y con 1000 tareas | 1 fichero y los mismos bytes en los dos casos |")
    # L2: rename con el lector abierto (Windows)
    c = os.path.join(raiz, "l2")
    os.makedirs(c)
    escribir_estado(c, {"tarea": "TASK-001", "calculado": AHORA})
    h = open(os.path.join(c, "estado.json"), encoding="utf-8")
    try:
        escribir_atomico(os.path.join(c, "estado.json"), json.dumps({"tarea": "TASK-001", "v": 2}), reintentos=0)
        l2 = "entra a la primera"
    except PermissionError as e:
        l2 = f"falla con PermissionError (WinError {getattr(e, 'winerror', '?')})"
    print(f"  L2 plataforma {sys.platform}: os.replace sobre estado.json con un lector abierto y sin reintento: {l2}")
    hilo = threading.Timer(0.05, h.close)
    hilo.start()
    usados = escribir_atomico(os.path.join(c, "estado.json"), json.dumps({"tarea": "TASK-001", "v": 3}), reintentos=200, espera=0.01)
    hilo.join()
    final = json.load(open(os.path.join(c, "estado.json"), encoding="utf-8"))["v"]
    print(f"  L2 con reintento y el lector cerrandose a los 50 ms: entra ({'con reintentos' if usados else 'sin reintentos'}), valor final {final}")
    print(f"[tiempo] L2 reintentos usados: {usados}", file=sys.stderr)
    if sys.platform == "win32":
        comprueba("L", l2.startswith("falla"), "L2: en Windows el rename con el lector abierto no falla")
        comprueba("L", usados > 0 and final == 3, "L2: el reintento no entra")
    filas.append(f"| L2 | `os.replace` con la status line leyendo el fichero (Windows) | sin reintento {l2.split(' (')[0]}; con reintento entra |")
    # L3: escritura rota (intercalado determinista)
    c = os.path.join(raiz, "l3")
    os.makedirs(c)
    escribir_estado(c, {"tarea": "TASK-001", "fase": "implementacion", "calculado": AHORA - 600})
    escribir_estado(c, {"tarea": "TASK-001", "fase": "rondas", "calculado": AHORA - 300})
    nuevo = json.dumps({"tarea": "TASK-001", "fase": "cierre", "calculado": AHORA})
    with open(os.path.join(c, "estado.json"), "w", encoding="utf-8") as f:   # escritor no atomico, cortado a medias
        f.write(nuevo[: len(nuevo) // 2])
    d3, origen3, _, _ = leer_estado(c, AHORA)
    print(f"  L3 escritura no atomica cortada a medias: la status line ensena {origen3} (fase {d3['fase']}, hace {d3.get('antiguedad_min')} min)")
    comprueba("L", origen3 == "ultimo-bueno" and d3["fase"] == "implementacion", "L3: no cae al ultimo valor bueno")
    filas.append("| L3 | `estado.json` a medio escribir | la status line enseña el último valor bueno con su antigüedad |")
    # L5: dos escritores de coste
    c = os.path.join(raiz, "l5")
    os.makedirs(c)
    base = "## Coste\n\n| fecha | fase | fuente | total |\n|---|---|---|---|\n"
    ruta = os.path.join(c, "tarea.md")
    open(ruta, "w", encoding="utf-8").write(base)
    a = open(ruta, encoding="utf-8").read()   # intercalado determinista: los dos leen, los dos escriben
    b = open(ruta, encoding="utf-8").read()
    open(ruta, "w", encoding="utf-8").write(a + "| t | revision | agente:a | 1 |\n")
    open(ruta, "w", encoding="utf-8").write(b + "| t | revision | agente:b | 2 |\n")
    perdidas = 2 - open(ruta, encoding="utf-8").read().count("| t | revision")
    print(f"  L4 leer-anadir-reescribir con dos escritores intercalados: filas perdidas {perdidas} de 2")
    comprueba("L", perdidas == 1, "L5: el intercalado no pierde la fila")
    ps = [hijo(HIJO_COSTE, c, f, 200, "por-fuente") for f in ("a", "b")]
    for p in ps:
        p.communicate()
    filas_f = sum(open(os.path.join(c, f"coste-{f}.md"), encoding="utf-8").read().count("| revision |") for f in ("a", "b"))
    print(f"  L4 un fichero por fuente, solo anadir, con dos procesos reales: {filas_f} de 400 filas")
    comprueba("L", filas_f == 400, "L5: con un fichero por fuente se pierden filas")
    filas.append("| L4 | dos escritores añaden filas de `## Coste` a la vez | leer-añadir-reescribir pierde 1 de 2 filas; un fichero por fuente, ninguna de 400 |")
    # L6: la linea para TASK-075 en el commit fijo
    T = d["T"]
    x = T["TASK-075"]
    fase, faltan_ = fase_actual(x["trans"])
    tr, quieto = transcurrido(x["trans"], x["coste"], d["commits_075"], fase, AHORA)
    rs = restante(d["est"]["TASK-075"], fase, tr)
    e = d["est"]["TASK-075"]
    datos = {"tarea": "TASK-075", "fase": fase, "faltan": faltan_, "restante": [rs["med"], rs["p80"]], "n": e["n"],
             "nivel": e["nivel"], "calculado": AHORA, "quieto_min": round(quieto)}
    c = os.path.join(raiz, "l6")
    os.makedirs(c)
    escribir_estado(c, datos)
    dd, _, _, _ = leer_estado(c)
    print(f"  L5 TASK-075 a las {hm(AHORA)} UTC: fase {fase} desde {hm(min(t for t, f in x['trans'] if f == 'start'))}, "
          f"{round(tr)} min transcurridos ({round(quieto)} desde el ultimo commit o coste) ; estimacion total de su nivel: mediana {round(e['total'][0])} min, p80 {round(e['total'][1])} ; "
          f"restante {'condicionado a las que pasaron de ' + str(round(tr)) + ' min' if rs['condicionada'] else 'sin condicionar'} (n={rs['n']})")
    print(f"  status line: {linea_estado(dd)}")
    print(f"  la estimacion del agente en el plan: {x['plan_min']} min")
    filas.append(f"| L5 | status line de TASK-075 a las {hm(AHORA)[11:]} UTC | `{linea_estado(dd)}` |")
    tabla = ["| ID | Caso | Resultado |", "|---|---|---|"] + filas
    print("\ntabla-statusline calculada:")
    for l in tabla:
        print(l)
    return tabla, {"l2": l2, "tr": tr, "rs": rs, "fase": fase, "e": e, "linea": linea_estado(dd)}


def seccion_f(H, rng):
    sep("F: hora de fin de un lote con intervalo p50/p80 frente a la suma de medianas")
    tabla = ["| Repo | Lotes de 5 tareas seguidas | Real ≤ suma de medianas | Real ≤ p50 remuestreado | Real ≤ p80 remuestreado |",
             "|---|---|---|---|---|"]
    res = {}
    for nom, otro in (("ToolsaWeb", "OpenDockTrading"), ("OpenDockTrading", "ToolsaWeb")):
        S, So = H[nom]["S"], H[otro]["S"]
        n = c_sum = c50 = c80 = 0
        for i in range(len(S) - 4):
            lote = S[i:i + 5]
            lanz = lote[0]["approve"]
            p = [m for m in S if m["finish"] <= lanz]
            s = [m for m in So if m["finish"] <= lanz]
            es = [estimar(o, p, s) for o in lote]
            if any(e is None for e in es):
                continue
            n += 1
            r = sum(o["real"]["total"] for o in lote)
            sm = sum(e["total"][0] for e in es)
            p50, p80 = suma_remuestreada([[m["real"]["total"] for m in e["muestras"]] for e in es], rng, 1000)
            c_sum += r <= sm
            c50 += r <= p50
            c80 += r <= p80
        print(f"{nom}: lotes de 5 con estimacion para todas {n} ; real <= suma de medianas {c_sum} ({pct(c_sum, n)} %) ; "
              f"<= p50 {c50} ({pct(c50, n)} %) ; <= p80 {c80} ({pct(c80, n)} %)")
        tabla.append(f"| {nom} | {n} | {pct(c_sum, n)} % | {pct(c50, n)} % | {pct(c80, n)} % |")
        res[nom] = (n, c_sum, c50, c80)
    # la serie de F1-16
    S, So = H["ToolsaWeb"]["S"], H["OpenDockTrading"]["S"]
    ids = {f"TASK-{i}" for i in list(range(173, 192)) + [205, 210]}
    lote = [o for o in S if set(o["miembros"]) & ids]
    lanz = min(o["approve"] for o in lote)
    p = [m for m in S if m["finish"] <= lanz]
    s = [m for m in So if m["finish"] <= lanz]
    es = [estimar(o, p, s) for o in lote]
    r = sum(o["real"]["total"] for o in lote)
    sm = sum(e["total"][0] for e in es)
    p50, p80 = suma_remuestreada([[m["real"]["total"] for m in e["muestras"]] for e in es], random.Random(SEMILLA))
    print(f"serie de F1-16 ({len(lote)} muestras, de {lote[0]['id']} a {lote[-1]['id']}), lanzada en {hm(lanz)} UTC con lo terminado antes: "
          f"suma de medianas {coma(sm / 60)} h ; p50 {coma(p50 / 60)} h ; p80 {coma(p80 / 60)} h ; real {coma(r / 60)} h")
    print(f"  hora de fin si corriera seguido: p50 {hm(lanz + 60 * p50)} ; p80 {hm(lanz + 60 * p80)} ; con el real {hm(lanz + 60 * r)} UTC")
    res["serie"] = (len(lote), sm, p50, p80, r)
    print("\ntabla-lote calculada:")
    for l in tabla:
        print(l)
    return res, tabla


# =========================================================================== G: el ADR como dato

SECCIONES = ["## Contexto", "## Alternativas", "## Pruebas de concepto", "## Decisión", "## Divergencias con el plan",
             "## Requisitos y evidencia", "## Consecuencias", "## Fuera de alcance", "## No verificado"]


def entre(adr, marca):
    m = re.search(rf"<!-- {marca}:inicio -->\n(.*?)\n[ \t]*<!-- {marca}:fin -->", adr, re.S)
    return m.group(1) if m else None


def errores_adr(adr, ruta, repo, esperadas):
    err = []
    for marca, filas in esperadas.items():
        b = entre(adr, marca)
        if b is None:
            err.append(f"el ADR no tiene el bloque {marca}")
            continue
        leidas = [l.strip() for l in b.strip().splitlines()]
        if leidas[:2] != filas[:2]:
            err.append(f"{marca}: cabecera distinta")
        for f in filas[2:]:
            if f not in leidas:
                err.append(f"{marca}: falta o cambia la fila {f[:60]}")
        if len(leidas) != len(filas):
            err.append(f"{marca}: {len(leidas) - 2} filas y son {len(filas) - 2}")
    faltan = [s for s in SECCIONES if not re.search(rf"^{re.escape(s)}\s*$", adr, re.M)]
    if faltan:
        err.append(f"faltan secciones {faltan}")
    if not re.search(r"^- Estado: \*\*aceptada\*\*", adr, re.M):
        err.append("el estado no es aceptada")
    req = open(os.path.join(repo, "docs", "plan", "REQUISITOS-FUNCIONALES.md"), encoding="utf-8").read()
    plan = open(os.path.join(repo, "docs", "plan", "PLAN-TASKCODE-2.md"), encoding="utf-8").read()
    adrs = os.listdir(os.path.join(repo, "docs", "adr"))
    malos = [i for i in sorted(set(re.findall(r"\b(R[RFN]-\d+)\b", adr))) if not re.search(rf"\|\s*{i}\s*\|", req)]
    malos += [i for i in sorted(set(re.findall(r"\b(F\d+-\d+b?)\b", adr)))
              if not re.search(rf"\|\s*{i}\s*\|", plan) and not any(a.startswith(i + "-") for a in adrs)]
    if malos:
        err.append(f"IDs que no existen: {malos}")
    enlaces = re.findall(r"\]\(([^)]+)\)", adr)
    for e in enlaces:
        if "tareas/" in e or e.startswith("../../"):
            err.append(f"enlace prohibido {e}")
        elif not e.startswith("http") and not os.path.exists(os.path.join(os.path.dirname(ruta), e.split("#")[0])):
            err.append(f"enlace que no resuelve: {e}")
    idx = open(os.path.join(repo, "docs", "adr", "README.md"), encoding="utf-8").read()
    if FILA_INDICE not in idx:
        err.append("falta la fila en docs/adr/README.md")
    return err


def seccion_g(repo, esperadas):
    sep("G: el ADR como dato (tablas entre marcadores, IDs, enlaces, secciones) y mutaciones de control")
    ruta = os.path.join(repo, ADR_REL)
    if not os.path.exists(ruta):
        falla("G", "no existe el ADR")
        return
    adr = open(ruta, encoding="utf-8").read()
    err = errores_adr(adr, ruta, repo, esperadas)
    for e in err:
        falla("G", e)
    print(f"errores del ADR: {len(err)}")
    fila_err = next((l for l in esperadas["tabla-error"][2:] if "cascada (modelo)" in l), "")
    muts = [
        ("cifra-del-error-cambiada", adr.replace(fila_err, fila_err.replace("|", "| x", 1), 1) if fila_err else adr,
         "tabla-error: falta o cambia la fila"),
        ("id-inexistente", adr.replace("## Contexto", "## Contexto\n\nVer F7-99.", 1), "IDs que no existen: ['F7-99']"),
        ("enlace-roto", adr.replace("## Contexto", "## Contexto\n\n[x](F1-99-no-existe.md)", 1),
         "enlace que no resuelve: F1-99-no-existe.md"),
        ("seccion-borrada", adr.replace("## No verificado", "## Sin verificar"), "faltan secciones"),
    ]
    for nombre, mut, esperado in muts:
        if mut == adr:
            falla("G", f"la mutacion {nombre} no cambia nada")
            continue
        errs = errores_adr(mut, ruta, repo, esperadas)
        ok = any(esperado in e for e in errs)
        print(f"  mutacion {nombre}: {'falla como se espera' if ok else 'NO FALLA'}")
        if not ok:
            falla("G", f"la mutacion {nombre} no la detecta el guion")


# ===========================================================================

def resultado():
    print()
    if FALLOS:
        print(f"RESULTADO: {len(FALLOS)} fallos")
        for f in FALLOS:
            print(f"  - {f}")
        sys.exit(1)
    print("RESULTADO: todo como se espera")


def main():
    sys.stdout.reconfigure(encoding="utf-8", newline="\n")
    args = sys.argv[1:]
    if len(args) != 3:
        raise SystemExit(__doc__)
    repo, toolsaweb, odt = (os.path.abspath(a) for a in args)
    if not os.path.normcase(POC).startswith(os.path.normcase(os.path.join(repo, "tareas")) + os.sep):
        raise SystemExit(f"ERROR: el guion ({POC}) no esta dentro de tareas/ del repo {repo}")
    trabajo = carpeta_trabajo()
    borrar(trabajo)
    os.makedirs(trabajo)
    open(os.path.join(trabajo, MARCA), "w", encoding="utf-8").write("carpeta de trabajo de poc-f1-17.py\n")
    print(f"PoC F1-17 ; solo lectura con git log, ls-tree y cat-file sobre los originales: ToolsaWeb@{FIJO_TW[:9]} ; "
          f"OpenDockTrading@{FIJO_ODT[:9]} ; TaskCodeTwo@{FIJO_TC2[:9]} y @{PROTO_TC2[:9]} ; corte {UMBRAL} min ; n minimo {N_MIN} ; "
          f"p{round(PCT * 100)} ; ahora {hm(AHORA)} UTC")
    abiertos = []
    try:
        repos = [Repo("ToolsaWeb", toolsaweb, FIJO_TW), Repo("OpenDockTrading", odt, FIJO_ODT)]
        tc2, proto = Repo("TaskCodeTwo", repo, FIJO_TC2), Repo("TaskCodeTwo", repo, PROTO_TC2)
        abiertos = repos + [tc2, proto]
        H, t_hist = seccion_h(repos)
        print("\ntabla-historico calculada:")
        for l in t_hist:
            print(l)
        reloj("H terminada")
        r_res, t_corte = seccion_r(H)
        rng = random.Random(SEMILLA)
        e_res, t_err, t_fases, t_var = seccion_e(H, rng)
        reloj("E terminada")
        d_res = seccion_d(tc2, proto, H, rng)
        reloj("D terminada")
        t_start = seccion_s(trabajo)
        reloj("S terminada")
        t_sl, l_res = seccion_l(trabajo, d_res)
        reloj("L terminada")
        f_res, t_lote = seccion_f(H, random.Random(SEMILLA))
        reloj("F terminada")
        esperadas = {"tabla-historico": t_hist, "tabla-corte": t_corte, "tabla-error": t_err, "tabla-fases": t_fases,
                     "tabla-variantes": t_var, "tabla-resumen": d_res["resumen"], "tabla-reparto": d_res["tabla"],
                     "tabla-start": t_start, "tabla-statusline": t_sl, "tabla-lote": t_lote}
        seccion_g(repo, esperadas)
    finally:
        for r in abiertos:
            r.cerrar()
        borrar(trabajo)
    print()
    print(f"carpeta de trabajo borrada al terminar: {'no' if os.path.lexists(trabajo) else 'si'}")
    if os.path.lexists(trabajo):
        falla("main", "la carpeta de trabajo sigue existiendo")
    reloj("fin")
    resultado()


if __name__ == "__main__":
    main()

"""PoC de F1-15 (TASK-073): seleccion de skills, agentes y modelo por tarea y por rol.

Se ejecuta con `python -I -B`. SOLO LECTURA: ToolsaWeb y OpenDockTrading se leen
de copias `git clone --no-hardlinks` en la carpeta de trabajo y de commits fijos;
TaskCodeTwo, de su historia (commit fijo) y de sus ficheros; la maquina, de
~/.claude y de la cache del plugin 0.16.1, sin escribir nada. No habla con la
red ni con ningun modelo.

(R) Retrospectiva de las 15 tareas de la Fase 1 (TASK-002..TASK-015 y TASK-094):
    roles del plan, puntos de la heuristica de la 0.16, IMPORTANTES por clase
    (tabla clasificacion-hallazgos.md contrastada con los informes: mismos IDs,
    cita presente en el informe de su ronda, clase de la lista cerrada) y
    revisor usado. Mutaciones de control de la tabla.
(D) Detector `dominios()` con la `tabla-dominios` del ADR sobre esas 15 tareas:
    recall y precision frente a «tiene al menos un IMPORTANTE de
    puerta-seguridad», tareas que dispararian el suelo de roles y su coste
    (coste medio de un agente de diseno, de las filas `diseno` de `## Coste`).
(I) Inventario real de la maquina (plugin 0.16.1, otros plugins, ~/.claude/skills,
    ~/.claude/agents, agentes integrados) y `.claude/` de los tres proyectos.
(T) Las 10 tareas reales con el muestreo estratificado fijado en el ADR: carril
    efectivo (funcion de F1-10), complejidad (heuristica de la 0.16), propuesta
    de `seleccionar` por rol, lo que uso la 0.16 y la columna «comprobado a mano
    (Carlos)»: «coincide (Carlos, 2026-11-14)» (tarea.md, commit 5768e3c).
(S) `seleccionar` interpreta `tabla-seleccion` del ADR (vocabulario cerrado de condiciones y acciones) y tiene
    un caso de control por fila (S01..S15, cuatro de S13); evasion por vocabulario (IMP-3) y catalogo del PR
    (MEN-4) con lo que contiene la defensa y lo que no. Determinista: mismo hash en dos ejecuciones, inventario y
    catalogo permutados dan lo mismo, quitar una skill la saca con aviso,
    empate por id lexicografico, el modelo cambia con carril y complejidad en
    filas reales, sin camino a la red ni a un modelo (grep del fuente y guardia
    de auditoria con empate total).
(G) El ADR: tablas entre marcadores con ids unicos, modelos en la escala, reglas
    que solo dan ids del inventario, muestra y retrospectiva iguales a lo
    calculado, IDs de plan y requisitos, enlaces, secciones, cifras de la prosa
    por contexto, y copias mutadas que fallan cada una por su motivo.

Uso: python -I -B poc-f1-15.py <trabajo> <repo-TaskCodeTwo> <carpeta-Personal>
Sale con 0 solo si todo sale como se espera. <trabajo> tiene que estar bajo
%LOCALAPPDATA%/pruebas-claude/<proyecto>/<motivo>; se borra al empezar y al
terminar. Los tiempos de reloj van a stderr.
"""
import hashlib
import inspect
import io
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import time
import unicodedata

POC = os.path.dirname(os.path.abspath(__file__))
CLASIF = os.path.join(POC, "clasificacion-hallazgos.md")
ADR_REL = os.path.join("docs", "adr", "F1-15-seleccion-de-skills-agentes-y-modelo.md")
FILA_INDICE = "| [F1-15](F1-15-seleccion-de-skills-agentes-y-modelo.md) |"
MARCA = ".poc-f1-15"
INICIO = os.environ.get("USERPROFILE", os.path.expanduser("~"))
CLAUDE = os.path.join(INICIO, ".claude")
PLUGIN = os.path.join(CLAUDE, "plugins", "cache", "taskcode-marketplace", "taskcode-plugin", "0.16.1")

FASE1 = ["TASK-002", "TASK-003", "TASK-004", "TASK-005", "TASK-006", "TASK-007", "TASK-008", "TASK-009",
         "TASK-010", "TASK-011", "TASK-012", "TASK-013", "TASK-014", "TASK-015", "TASK-094"]
ADR_DE = {"TASK-002": "F1-1", "TASK-003": "F1-2", "TASK-004": "F1-3", "TASK-005": "F1-4", "TASK-006": "F1-5",
          "TASK-007": "F1-6", "TASK-008": "F1-7", "TASK-009": "F1-8", "TASK-010": "F1-9", "TASK-011": "F1-10",
          "TASK-012": "F1-11", "TASK-013": "F1-12", "TASK-014": "F1-14", "TASK-015": "F1-13", "TASK-094": "F1-18"}
CLASES = ["puerta-seguridad", "contrato", "correccion", "fidelidad-evidencia", "reproducibilidad"]
# Commits fijos (leidos el 2026-11-14). Los de los pilotos son los de las PoC de F1-9 y F1-10; el de
# TaskCodeTwo es `develop` al empezar TASK-073 (contiene los 15 merges de la Fase 1).
FIJO = {
    "ToolsaWeb": "93c9618ac156c4357278993df983361afccf1dc2",
    "OpenDockTrading": "fab8d7e05eee2784b9cd4695b94f612a6a1ee257",
    "TaskCodeTwo": "df615c14ca095c2d181885d90f7f20ffa17a0fd1",
}
PILOTOS = ["ToolsaWeb", "OpenDockTrading"]

ESCALA = ["haiku", "sonnet", "opus"]                     # de menos a mas
NIVELES = ["trivial", "simple", "media", "alta", "critica"]
ORDEN = ["errata", "fix", "funcionalidad", "riesgo-alto"]
DOMINIOS = ["puertas-seguridad", "rendimiento"]           # lista cerrada en el binario; la config solo anade senales
SENALES = ["etiqueta", "palabra", "ruta"]
ROLES_ORDEN = ["arquitectura", "riesgos", "testing", "dominio"]
ROLES_MIN = {"errata": 0, "fix": 1, "funcionalidad": 1, "riesgo-alto": 4}   # tabla-politica de F1-10
NCX = {"trivial": 0, "simple": 1, "media": 2, "alta": 3, "critica": 4}        # tabla de la 0.16 (agentes_brainstorm_*)
INTEGRADOS = ["general-purpose"]                         # agente integrado de Claude Code (lanza cada revisora)
PREFIJOS_PLUGIN = ("taskcode-plugin:", "taskcodetwo:")   # la generica solo puede ser la del plugin (MEN-4)
ROL_COL = {"diseno": 0, "implementacion": 1, "revision": 2}
MANO = "coincide (Carlos, 2026-11-14)"                   # tarea.md, «Decisiones de la persona», commit 5768e3c
# Vocabulario cerrado de tabla-seleccion (IMP-2 de la revision 1): la columna «Condición» es una lista de atomos
# unidos por « y » (cada uno puede ir precedido de «no »); «Acción» es un solo token. Lo que no casa es error.
ATOMOS = [r"siempre", r"carril=(errata|fix|funcionalidad|riesgo-alto)", r"carril!=(errata|fix|funcionalidad|riesgo-alto)",
          r"dominio=(puertas-seguridad|rendimiento)", r"roles>=(\d)", r"ronda>=(\d)", r"hay-skill-diseno-del-dominio",
          r"hay-skill-ejecucion-con-solape", r"rutas-casan-entre-1-y-umbral", r"rutas-no-casan-o-pasan-umbral",
          r"dominio-con-revisora", r"dominio-sin-revisora", r"hay-no-instalado", r"modelo-no-permitido"]
# Una accion que necesita candidatos exige su atomo de condicion (MEN-9 de la revision 2)
EXIGE = {"skills-diseno-del-dominio": "hay-skill-diseno-del-dominio",
         "skill-ejecucion=": "hay-skill-ejecucion-con-solape",
         "revisoras-de-rutas": "rutas-casan-entre-1-y-umbral", "revisora-del-dominio": "dominio-con-revisora",
         "aviso=dominio-sin-revisora": "dominio-sin-revisora", "quitar-no-instalado": "hay-no-instalado",
         "restringir-modelo": "modelo-no-permitido"}
ACCIONES = [r"sin-plan", r"roles=max\(roles_min,complejidad\)", r"suelo=(\d)", r"mas-rol=(rendimiento|unificador)",
            r"skills-diseno-del-dominio", r"skill-ejecucion=solape,prioridad,id", r"revisoras-de-rutas\+generica-si-sueltas",
            r"generica-del-plugin", r"revisora-del-dominio", r"aviso=dominio-sin-revisora", r"lanzador",
            r"modelo-baja=(\d)", r"quitar-no-instalado", r"restringir-modelo"]

FALLOS = []
T0 = time.time()


def falla(seccion, texto):
    FALLOS.append(f"{seccion}: {texto}")
    print(f"FALLO en {seccion}: {texto}")


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


def comprobar_trabajo(trabajo):
    """<...>/pruebas-claude/<proyecto>/<motivo>[/...] y la carpeta no existe, esta vacia o lleva la marca."""
    base = os.environ.get("LOCALAPPDATA")
    if not base:
        raise SystemExit("ERROR: falta LOCALAPPDATA")
    raiz = os.path.normcase(os.path.abspath(os.path.join(base, "pruebas-claude")))
    if not os.path.normcase(os.path.abspath(trabajo)).startswith(raiz + os.sep):
        raise SystemExit(f"ERROR: {trabajo} no esta bajo %LOCALAPPDATA%/pruebas-claude ({raiz})")
    debajo = os.path.relpath(os.path.abspath(trabajo), os.path.join(base, "pruebas-claude")).replace("\\", "/").split("/")
    if len(debajo) < 2:
        raise SystemExit(f"ERROR: {trabajo}: hacen falta dos niveles bajo pruebas-claude (<proyecto>/<motivo>)")
    if os.path.lexists(trabajo):
        if not os.path.isdir(trabajo):
            raise SystemExit(f"ERROR: {trabajo} existe y no es una carpeta")
        if os.listdir(trabajo) and not os.path.exists(os.path.join(trabajo, MARCA)):
            raise SystemExit(f"ERROR: {trabajo} no esta vacia ni lleva la marca {MARCA}: no se borra")


def git(repo, *args, binario=False):
    r = subprocess.run(["git", "-C", repo, "-c", "core.quotepath=false", *args], capture_output=True, check=False)
    if r.returncode != 0:
        raise SystemExit(f"ERROR: git {' '.join(args)} en {os.path.basename(repo)}: "
                         f"{r.stderr.decode('utf-8', 'replace').strip()}")
    return r.stdout if binario else r.stdout.decode("utf-8", "replace")


def h12(obj):
    return hashlib.sha256(json.dumps(obj, sort_keys=True, ensure_ascii=False).encode("utf-8")).hexdigest()[:12]


# =========================================================================== nucleo (funciones puras)

def normalizar(texto):
    return "".join(c for c in unicodedata.normalize("NFD", texto) if not unicodedata.combining(c)).lower()


def casa(glob, ruta):
    """Glob de rutas: `**/` cero o mas carpetas, `**` cualquier cosa, `*` sin /, `?` un caracter sin /."""
    rx, i = "", 0
    while i < len(glob):
        if glob.startswith("**/", i):
            rx += "(?:.*/)?"
            i += 3
        elif glob.startswith("**", i):
            rx += ".*"
            i += 2
        elif glob[i] == "*":
            rx += "[^/]*"
            i += 1
        elif glob[i] == "?":
            rx += "[^/]"
            i += 1
        else:
            rx += re.escape(glob[i])
            i += 1
    return re.fullmatch(rx, ruta) is not None


class ErrorConfig(Exception):
    pass


class ErrorTabla(Exception):
    pass


def validar_cfg(cfg):
    for k in cfg:
        if k not in ("dominios_extra", "modelos_permitidos"):
            raise ErrorConfig(f"clave desconocida: {k}")
    for d, sen in cfg.get("dominios_extra", {}).items():
        if d not in DOMINIOS:
            raise ErrorConfig(f"dominio fuera de la lista cerrada: {d}")
        for s in sen:
            if s not in ("etiquetas", "palabras", "rutas"):
                raise ErrorConfig(f"senal desconocida: {s}")
    mp = cfg.get("modelos_permitidos")
    if mp is not None and (not mp or any(m not in ESCALA for m in mp)):
        raise ErrorConfig(f"modelos_permitidos fuera de la escala: {mp}")


def dominios(t, cambios, tabla_dom, cfg):
    """[(dominio, (senales...))] ordenado. Senales: etiqueta exacta, palabra al inicio de palabra en objetivo y
    criterios (sin tildes ni mayusculas), ruta con glob sobre los dos nombres de cada renombrado."""
    validar_cfg(cfg)
    # El texto de tarea.md lo escribe el agente: no confiable (IMP-3). Si la tarea tiene un texto aprobado (el del
    # commit firmado de `approve`, t["aprobado"]), sus senales se suman a las del texto vivo: reescribir el texto
    # despues de aprobar solo puede anadir dominios, nunca quitarlos (como el carril de F1-10, solo sube).
    fuentes = [t] + ([t["aprobado"]] if t.get("aprobado") else [])
    texto = normalizar("\n".join(x for f in fuentes for x in [f["objetivo"], *f["criterios"]]))
    etiquetas = {normalizar(e) for f in fuentes for e in f["etiquetas"]}
    rutas = [r for _e, rs in cambios for r in rs]
    filas = list(tabla_dom)
    for d, sen in sorted(cfg.get("dominios_extra", {}).items()):
        for clave, tipo in (("etiquetas", "etiqueta"), ("palabras", "palabra"), ("rutas", "ruta")):
            for v in sen.get(clave, []):
                filas.append(("cfg", d, tipo, v))
    vistos = {}
    for _id, d, tipo, valor in filas:
        if tipo == "etiqueta":
            ok = normalizar(valor) in etiquetas
        elif tipo == "palabra":
            ok = re.search(r"\b" + re.escape(normalizar(valor)), texto) is not None
        else:
            ok = any(casa(valor.lower(), r.lower()) for r in rutas)   # sin mayusculas (Windows y macOS, MEN-10)
        if ok:
            vistos.setdefault(d, set()).add(f"{tipo}:{valor}")
    return [(d, tuple(sorted(vistos[d]))) for d in DOMINIOS if d in vistos]


def modelo_de(carril, cx, rol, tabla_mod):
    filas = [f for f in tabla_mod if f["carril"] == carril and (cx in f["cx"] or "cualquiera" in f["cx"])]
    if len(filas) != 1:
        raise ErrorTabla(f"tabla-modelo: {len(filas)} filas para {carril}/{cx}")
    return filas[0][rol], filas[0]["id"]


def restringir(modelo, cfg):
    mp = cfg.get("modelos_permitidos")
    if modelo is None or not mp or modelo in mp:
        return modelo, None
    i = ESCALA.index(modelo)
    abajo = [m for m in ESCALA[:i] if m in mp]
    nuevo = abajo[-1] if abajo else [m for m in ESCALA[i + 1:] if m in mp][0]
    return nuevo, f"modelo-restringido:{modelo}->{nuevo}"


def parsear_regla(i, cond, accion):
    """Condicion y accion de una fila de tabla-seleccion en el vocabulario cerrado. ErrorTabla si no casan."""
    atomos = []
    for a in cond:
        neg = a.startswith("no ")
        a2 = a[3:] if neg else a
        m = next((re.fullmatch(p, a2) for p in ATOMOS if re.fullmatch(p, a2)), None)
        if not m:
            raise ErrorTabla(f"tabla-seleccion {i}: condición desconocida: {a}")
        atomos.append((neg, a2))
    if len(accion) != 1 or not any(re.fullmatch(p, accion[0]) for p in ACCIONES):
        raise ErrorTabla(f"tabla-seleccion {i}: acción desconocida: {accion}")
    for pref, atomo in EXIGE.items():
        if accion[0].startswith(pref) and (False, atomo) not in atomos:
            raise ErrorTabla(f"tabla-seleccion {i}: la acción {accion[0]} exige el átomo {atomo}")
    return atomos, accion[0]


def _cumple(atomos, ctx, dom=None):
    for neg, a in atomos:
        if a == "siempre":
            v = True
        elif a.startswith("carril!="):
            v = ctx["carril"] != a.split("=")[1]
        elif a.startswith("carril="):
            v = ctx["carril"] == a.split("=")[1]
        elif a.startswith("dominio="):
            v = a.split("=")[1] in ctx["nd"]
        elif a.startswith("roles>="):
            v = len(ctx["roles"]()) >= int(a.split("=")[1])
        elif a.startswith("ronda>="):
            v = ctx["ronda"] >= int(a.split("=")[1])
        elif a == "dominio-con-revisora":
            v = dom is not None and bool(ctx["rev_dom"](dom))
        elif a == "dominio-sin-revisora":
            v = dom is not None and not ctx["rev_dom"](dom)
        else:
            v = ctx["hechos"][a]()
        if v == neg:
            return False
    return True


def seleccionar(rol, t, carril, cx, cambios, inv, cat, cfg, tablas, ronda=1):
    """Funcion pura que INTERPRETA tabla-seleccion del ADR fila a fila, en su orden (IMP-2). inv: ids instalados;
    cat: entradas con metadatos. Nunca llama a un modelo ni a la red. Una fila se anota si su condicion se cumple y
    su accion cambia algo."""
    if rol not in ROL_COL or carril not in ORDEN or cx not in NIVELES:
        raise ErrorTabla(f"entrada no valida: {rol} {carril} {cx}")
    reglas, avisos = set(), set()
    doms = dominios(t, cambios, tablas["dominios"], cfg)
    nd = {d for d, _s in doms}
    modelo, fila_m = modelo_de(carril, cx, rol, tablas["modelo"])
    reglas.add(fila_m)
    cat_ord = sorted(cat, key=lambda e: (e["id"], e["tipo"]))
    instalados = set(inv)
    est = {"n": 0, "extra": [], "plan": True, "agentes": [], "skills": [], "modelo": modelo}
    rutas = [r for _e, rs in cambios for r in rs]
    revs = [e for e in cat_ord if e["tipo"] == "skill" and e.get("rol") == "revisor"]
    gen = [e for e in revs if e.get("fallback") and e["id"].startswith(PREFIJOS_PLUGIN)]
    umbral = gen[0].get("umbral", 3) if gen else 3
    casadas = [e["id"] for e in revs if e.get("patrones") and any(casa(p, r) for p in e["patrones"] for r in rutas)]
    etq = {normalizar(x) for x in t["etiquetas"]}

    def cands_ejecucion():
        out = []
        for e in cat_ord:
            if e["tipo"] == "skill" and e.get("rol") == "ejecucion":
                s = len({normalizar(x) for x in e.get("etiquetas", [])} & etq) + len(set(e.get("dominios", [])) & nd)
                if s > 0:
                    out.append((-s, -e.get("prioridad", 0), e["id"]))
        return sorted(out)

    skills_dis = [e["id"] for e in cat_ord if e["tipo"] == "skill" and e.get("rol") == "diseno"
                  and set(e.get("dominios", [])) & nd]
    ctx = {"carril": carril, "nd": nd, "ronda": ronda,
           "roles": lambda: (ROLES_ORDEN[:est["n"]] + est["extra"]) if est["plan"] else [],
           "rev_dom": lambda d: [e["id"] for e in revs if d in e.get("dominios", [])],
           "hechos": {"hay-skill-diseno-del-dominio": lambda: bool(skills_dis),
                      "hay-skill-ejecucion-con-solape": lambda: bool(cands_ejecucion()),
                      "rutas-casan-entre-1-y-umbral": lambda: 1 <= len(casadas) <= umbral,
                      "rutas-no-casan-o-pasan-umbral": lambda: not casadas or len(casadas) > umbral,
                      "hay-no-instalado": lambda: any(x not in instalados for x in est["agentes"] + est["skills"]),
                      "modelo-no-permitido": lambda: restringir(est["modelo"], cfg)[1] is not None}}

    def materializar_diseno():
        if rol != "diseno" or est.get("hecho"):
            return
        est["hecho"] = True
        for r in ctx["roles"]():
            c = [e["id"] for e in cat_ord if e["tipo"] == "agente" and e.get("rol") == "diseno" and e.get("rol_diseno") == r]
            if c:
                est["agentes"].append(c[0])
            else:
                avisos.add(f"rol-sin-agente:{r}")

    for i, fila in tablas["seleccion"].items():
        if fila["rol"] not in (rol, "todos"):
            continue
        if fila["rol"] == "todos":
            materializar_diseno()
        elif rol == "diseno" and not est["plan"]:
            continue
        atomos, acc = parsear_regla(i, fila["cond"], fila["accion"])
        por_dominio = any(a in ("dominio-con-revisora", "dominio-sin-revisora") for _n, a in atomos)
        doms_fila = sorted(nd) if por_dominio else [None]
        for dom in doms_fila:
            if not _cumple(atomos, ctx, dom):
                continue
            antes = json.dumps([est["n"], est["extra"], est["plan"], est["agentes"], est["skills"], est["modelo"],
                                sorted(avisos)])
            if acc == "sin-plan":
                est["plan"] = False
                reglas.add(i)
                continue
            if acc.startswith("roles=max"):
                est["n"] = 4 if carril == "riesgo-alto" else max(ROLES_MIN[carril], NCX[cx] if carril == "funcionalidad" else 0)
                reglas.add(i)
                continue
            if acc.startswith("suelo="):
                est["n"] = max(est["n"], int(acc.split("=")[1]))
            elif acc.startswith("mas-rol="):
                r = acc.split("=")[1]
                if r not in est["extra"]:
                    est["extra"].append(r)
            elif acc == "skills-diseno-del-dominio":
                est["skills"] += skills_dis
            elif acc.startswith("skill-ejecucion="):
                c = cands_ejecucion()
                emp = [x[2] for x in c if x[:2] == c[0][:2]]
                if len(emp) > 1:
                    avisos.add("empate-por-id:" + ",".join(emp))
                est["skills"].append(c[0][2])
            elif acc.startswith("revisoras-de-rutas"):
                est["skills"] += casadas
                sueltas = [r for r in rutas if not any(casa(p, r) for e in revs for p in e.get("patrones", []))]
                if sueltas and gen:
                    est["skills"].append(gen[0]["id"])
            elif acc == "generica-del-plugin":
                if gen:
                    est["skills"].append(gen[0]["id"])
            elif acc == "revisora-del-dominio":
                est["skills"].append(ctx["rev_dom"](dom)[0])
            elif acc == "aviso=dominio-sin-revisora":
                avisos.add(f"dominio-sin-revisora:{dom}")
            elif acc == "lanzador":
                lanz = [e["id"] for e in cat_ord if e["tipo"] == "agente" and e.get("rol") == "lanzador-revision"]
                est["agentes"] += lanz[:1]
            elif acc.startswith("modelo-baja="):
                if est["modelo"]:
                    est["modelo"] = ESCALA[max(0, ESCALA.index(est["modelo"]) - int(acc.split("=")[1]))]
            elif acc == "quitar-no-instalado":
                for x in est["agentes"] + est["skills"]:
                    if x not in instalados:
                        avisos.add(f"no-instalado:{x}")
            elif acc == "restringir-modelo":
                est["modelo"], av = restringir(est["modelo"], cfg)
                avisos.add(av)
            despues = json.dumps([est["n"], est["extra"], est["plan"], est["agentes"], est["skills"], est["modelo"],
                                  sorted(avisos)])
            if despues != antes:
                reglas.add(i)
    materializar_diseno()
    unico = lambda xs: [x for k, x in enumerate(xs) if x in instalados and x not in xs[:k]]
    return {"agentes": unico(est["agentes"]), "skills": unico(est["skills"]), "modelo": est["modelo"],
            "reglas": sorted(reglas), "avisos": sorted(avisos),
            "senales": [f"{d}:{s[0]}" for d, s in doms]}


# --------------------------------------------------------------------------- carril (funciones de F1-10)

SUELO_RIESGO = (".github", ".taskcode")


def clase_ruta(ruta):
    ruta = "/".join(c for c in ruta.split("/") if c not in ("", "."))
    baja = ruta.lower()
    if baja.split("/", 1)[0] in SUELO_RIESGO or baja.rsplit("/", 1)[-1] == "codeowners":
        return "riesgo-alto"
    if ruta.startswith("docs/"):
        return "errata"
    return "fix"


def carril_minimo(cambios):
    carril = "errata"
    for _e, rutas in cambios:
        for r in rutas:
            c = clase_ruta(r)
            if ORDEN.index(c) > ORDEN.index(carril):
                carril = c
    return carril


def carril_efectivo(declarado, minimo):
    return declarado if ORDEN.index(declarado) >= ORDEN.index(minimo) else minimo


# --------------------------------------------------------------------------- heuristica de la 0.16

def leer_heuristica(ruta):
    txt = open(ruta, encoding="utf-8").read()
    num = {k: int(v) for k, v in re.findall(r"^(\w+):\s*(\d+)\s*$", txt, re.M)}
    pal = re.search(r"^palabras_alto_riesgo:\s*\[(.*)\]\s*$", txt, re.M).group(1)
    num["palabras"] = [p.strip().strip('"') for p in pal.split(",")]
    return num


def puntuar(t, h):
    p = max(0, len(t["etiquetas"]) - 1) * h["peso_etiqueta_adicional"]
    texto = normalizar("\n".join([t["objetivo"], *t["criterios"]]))
    enc = [w for w in h["palabras"] if normalizar(w) in texto]
    p += len(enc) * h["peso_palabra_alto_riesgo"]
    p += len(t["dependencias"]) * h["peso_dependencia"]
    if t["tipo"] == "release":
        p += h["peso_tipo_release"]
    if t["tipo"] == "hotfix" and enc:
        p += h["peso_tipo_hotfix_con_palabra_riesgo"]
    if len(t["criterios"]) >= h["umbral_criterios_aceptacion"]:
        p += h["peso_criterios_aceptacion"]
    return p


def nivel(p, h):
    for n, k in (("trivial", "nivel_trivial_hasta"), ("simple", "nivel_simple_hasta"), ("media", "nivel_media_hasta"),
                 ("alta", "nivel_alta_hasta")):
        if p <= h[k]:
            return n
    return "critica"


def cx_efectiva(t, p, h):
    heur = nivel(p, h)
    dec = t["complejidad"] if t["complejidad"] in NIVELES else None
    return heur if dec is None else max(heur, dec, key=NIVELES.index)


# --------------------------------------------------------------------------- lectura de tareas

def leer_tarea(texto):
    texto = texto.replace("\r\n", "\n")
    m = re.match(r"---\n(.*?)\n---\n(.*)", texto, re.S)
    fm, cuerpo = (m.group(1), m.group(2)) if m else ("", texto)

    def campo(k):
        x = re.search(rf"^{k}:\s*(.*?)\s*$", fm, re.M)
        return x.group(1).strip().strip('"') if x else None

    def lista(k):
        v = campo(k) or "[]"
        v = v.strip("[]").strip()
        return [x.strip().strip('"') for x in v.split(",") if x.strip()] if v else []

    objetivo, criterios, sec, destino = [], [], None, True
    for linea in cuerpo.split("\n"):
        if re.match(r"^## ", linea):
            tit = normalizar(linea[3:].strip())
            sec = "obj" if tit == "objetivo" else "cri" if tit.startswith("criterios de aceptacion") else None
            destino = True
            continue
        if sec == "obj":
            objetivo.append(linea)
        elif sec == "cri":
            if re.match(r"^###+\s", linea):
                destino = normalizar(linea).strip("# ").strip() != "tras el cierre"
                continue
            if not destino:
                continue
            if re.match(r"^\s*- \[[ xX]\]", linea):
                criterios.append(re.sub(r"^\s*- \[[ xX]\]\s?", "", linea))
            elif criterios and linea.startswith((" ", "\t")) and linea.strip():
                criterios[-1] += " " + linea.strip()
    return {"id": campo("id"), "titulo": campo("titulo") or "", "tipo": campo("tipo") or "feature",
            "etiquetas": lista("etiquetas"), "dependencias": lista("dependencias"),
            "complejidad": campo("complejidad"), "modelo_sugerido": campo("modelo_sugerido"),
            "skills_recomendados": lista("skills_recomendados"), "regla_seleccion_skill": campo("regla_seleccion_skill"),
            "agente_revisor": campo("agente_revisor"), "objetivo": "\n".join(objetivo).strip(), "criterios": criterios,
            "cuerpo": cuerpo}


def cambios_de_rango(repo, desde, hasta):
    raw = git(repo, "log", "-M", "--name-status", "-z", "--format=", f"{desde}..{hasta}")
    tok = [t.lstrip("\n") for t in raw.split("\0")]
    tok = [t for t in tok if t != ""]
    out, i = [], 0
    while i < len(tok):
        est = tok[i]
        if not re.fullmatch(r"[ACDMRTUXB]\d*", est):
            raise SystemExit(f"ERROR: name-status inesperado {est!r}")
        n = 2 if est[0] in "RC" else 1
        out.append((est, tok[i + 1:i + 1 + n]))
        i += 1 + n
    return out


def cambios_por_tarea(repo, commit):
    """{TASK-NNN: [(estado, rutas)]} de los merges de primer padre, sin lo que vive bajo tareas/."""
    out = {}
    for linea in git(repo, "log", "--first-parent", "--merges", "--format=%H%x09%P%x09%s", commit).splitlines():
        _h, padres, asunto = linea.split("\t", 2)
        m = re.search(r"merge\((\w+)\): (\S+) ->", asunto)
        mid = re.search(r"task-(\d+)", m.group(2)) if m else None
        if not mid:
            continue
        p = padres.split()
        pr = [(e, r) for e, r in cambios_de_rango(repo, p[0], p[1]) if not all(x.startswith("tareas/") for x in r)]
        out.setdefault(f"TASK-{mid.group(1)}", []).extend(pr)
    return out


def archivo_tareas(repo, commit):
    """{TASK-NNN: {ruta_relativa: texto}} de tareas/04-terminadas en el commit (en memoria)."""
    data = git(repo, "archive", commit, "tareas/04-terminadas", binario=True)
    out = {}
    with tarfile.open(fileobj=io.BytesIO(data)) as tf:
        for m in tf.getmembers():
            x = re.fullmatch(r"tareas/04-terminadas/(TASK-\d+)/(.+)", m.name)
            if m.isfile() and x:
                out.setdefault(x.group(1), {})[x.group(2)] = tf.extractfile(m).read().decode("utf-8", "replace")
    return out


def uso_016(ficheros):
    """Lo que hizo la 0.16 en una tarea: roles del plan (salidas de brainstorm), revisores (nombre del informe o
    linea Revisor:)."""
    roles = sorted({m.group(1) for f in ficheros if (m := re.fullmatch(r"planificacion/brainstorm/salida-brainstorm-([a-z]+)-\d+\.md", f))})
    plan = ficheros.get("planificacion/plan-final.md", "")
    if not roles and plan:
        roles = ["arquitectura"] if re.search(r"unico rol", normalizar(plan[:600])) else ["?"]
    revs = set()
    for f, txt in ficheros.items():
        m = re.fullmatch(r"revision/informe-revision-\d+(?:-([a-z0-9-]+))?\.md", f)
        if m:
            if m.group(1):
                revs.add(m.group(1))
            r = re.search(r"^- Revisor:\s*`?([\w:.-]+)`?", txt, re.M)
            if r:
                revs.add(r.group(1))
    return roles, sorted(revs)


# =========================================================================== tablas del ADR

def entre_marcadores(texto, nombre):
    m = re.search(rf"<!-- {nombre}:inicio -->(.*?)<!-- {nombre}:fin -->", texto, re.S)
    return m.group(1) if m else None


def filas_tabla(bloque):
    filas = []
    for l in bloque.strip().splitlines():
        l = l.strip()
        if not l.startswith("|") or re.match(r"^\|[-| :]+\|$", l):
            continue
        filas.append([c.strip() for c in l.strip("|").split(" | ")])
    return filas[1:]


def bt(celda):
    return re.findall(r"`([^`]+)`", celda)


def leer_tablas(adr):
    """{'dominios': [(id, dom, senal, valor)], 'modelo': [...], 'seleccion': {id: fila}} y errores de forma."""
    err = []
    dom, mod, sel = [], [], {}
    b = entre_marcadores(adr, "tabla-dominios")
    if b is None:
        err.append("el ADR no tiene el bloque tabla-dominios")
    else:
        ids = set()
        for f in filas_tabla(b):
            if len(f) != 4:
                err.append(f"tabla-dominios: fila con {len(f)} columnas: {f[:1]}")
                continue
            i, d, s, v = f[0], f[1].strip("`"), f[2], bt(f[3])
            if not re.fullmatch(r"D\d{2}", i):
                err.append(f"tabla-dominios: fila sin id: {f}")
            if i in ids:
                err.append(f"id repetido en tabla-dominios: {i}")
            ids.add(i)
            if d not in DOMINIOS:
                err.append(f"tabla-dominios {i}: dominio fuera de la lista cerrada: {d}")
            if s not in SENALES:
                err.append(f"tabla-dominios {i}: senal desconocida: {s}")
            if len(v) != 1:
                err.append(f"tabla-dominios {i}: valor sin backticks")
                continue
            dom.append((i, d, s, v[0]))
    b = entre_marcadores(adr, "tabla-modelo")
    if b is None:
        err.append("el ADR no tiene el bloque tabla-modelo")
    else:
        ids = set()
        for f in filas_tabla(b):
            if len(f) != 6:
                err.append(f"tabla-modelo: fila con {len(f)} columnas: {f[:1]}")
                continue
            i = f[0]
            if not re.fullmatch(r"M\d{2}", i):
                err.append(f"tabla-modelo: fila sin id: {f}")
            if i in ids:
                err.append(f"id repetido en tabla-modelo: {i}")
            ids.add(i)
            carril = f[1].strip("`")
            cxs = [x.strip("`") for x in f[2].split(", ")]
            if carril not in ORDEN:
                err.append(f"tabla-modelo {i}: carril desconocido {carril}")
            if any(x not in NIVELES and x != "cualquiera" for x in cxs):
                err.append(f"tabla-modelo {i}: complejidad desconocida {cxs}")
            ms = []
            for c in f[3:6]:
                c = c.strip("`")
                if c == "—":
                    ms.append(None)
                elif c in ESCALA:
                    ms.append(c)
                else:
                    err.append(f"tabla-modelo {i}: modelo fuera de la escala: {c}")
                    ms.append(None)
            mod.append({"id": i, "carril": carril, "cx": cxs, "diseno": ms[0], "implementacion": ms[1], "revision": ms[2]})
        for c in ORDEN:
            for n in NIVELES:
                k = [f["id"] for f in mod if f["carril"] == c and (n in f["cx"] or "cualquiera" in f["cx"])]
                if len(k) != 1:
                    err.append(f"tabla-modelo: {len(k)} filas para {c}/{n} ({k})")
    b = entre_marcadores(adr, "tabla-seleccion")
    if b is None:
        err.append("el ADR no tiene el bloque tabla-seleccion")
    else:
        for f in filas_tabla(b):
            if len(f) != 6:
                err.append(f"tabla-seleccion: fila con {len(f)} columnas: {f[:1]}")
                continue
            i = f[0]
            if not re.fullmatch(r"S\d{2}", i):
                err.append(f"tabla-seleccion: fila sin id: {f}")
            if i in sel:
                err.append(f"id repetido en tabla-seleccion: {i}")
            if f[1].strip("`") not in ("diseno", "implementacion", "revision", "todos"):
                err.append(f"tabla-seleccion {i}: rol desconocido {f[1]}")
            cond = [x.strip() for x in re.split(r"\s+y\s+", f[2].strip())]
            cond = [x.strip("`") if x.startswith("`") else x for x in cond]
            cond = [("no " + x[4:].strip("`")) if x.startswith("no `") else x for x in cond]
            fila = {"rol": f[1].strip("`"), "cond": cond, "accion": bt(f[3]), "ids": bt(f[4])}
            try:
                parsear_regla(i, fila["cond"], fila["accion"])
            except ErrorTabla as e:
                err.append(str(e))
            sel[i] = fila
    return {"dominios": dom, "modelo": mod, "seleccion": sel}, err


# =========================================================================== R

def leer_clasificacion(texto):
    filas, err = [], []
    b = entre_marcadores(texto, "clasificacion")
    if b is None:
        return [], ["clasificacion-hallazgos.md no tiene el bloque clasificacion"]
    for f in filas_tabla(b):
        if len(f) != 6:
            err.append(f"clasificacion: fila con {len(f)} columnas: {f[:2]}")
            continue
        filas.append({"tarea": f[0], "id": f[1], "ronda": f[2], "clase": f[3], "cita": f[4], "motivo": f[5]})
    return filas, err


def informes_fase1(repo):
    """{tarea: {ronda: texto}}, {(tarea, IMP-n): primera ronda}, {tarea: revisores}."""
    textos, ids, revs = {}, {}, {}
    for t in FASE1:
        d = os.path.join(repo, "tareas", "04-terminadas", t, "revision")
        textos[t] = {}
        revs[t] = set()
        for f in sorted(os.listdir(d)):
            m = re.fullmatch(r"informe-revision-(\d+)\.md", f)
            if not m:
                continue
            r = int(m.group(1))
            txt = open(os.path.join(d, f), encoding="utf-8").read().replace("\r\n", "\n")
            textos[t][r] = txt
            rv = re.search(r"^- Revisor:\s*(\S+)", txt, re.M)
            if rv:
                revs[t].add(rv.group(1))
            for x in re.findall(r"^\| (IMP-\d+|IMPORTANTE-\d+) \| IMPORTANTE \|", txt, re.M):
                k = (t, "IMP-" + x.split("-")[1])
                ids[k] = min(ids.get(k, r), r)
    return textos, ids, revs


def errores_clasificacion(filas, ids, textos):
    err = []
    vistos = set()
    for f in filas:
        k = (f["tarea"], f["id"])
        if k in vistos:
            err.append(f"clasificacion: fila repetida {k}")
        vistos.add(k)
        if f["clase"] not in CLASES:
            err.append(f"clasificacion {k}: clase fuera de la lista cerrada: {f['clase']}")
        if k not in ids:
            err.append(f"clasificacion {k}: no es un IMPORTANTE de los informes")
            continue
        if f["ronda"] != str(ids[k]):
            err.append(f"clasificacion {k}: ronda {f['ronda']} y aparece por primera vez en la {ids[k]}")
        cita = f["cita"]
        if not (cita.startswith("«") and cita.endswith("»")):
            err.append(f"clasificacion {k}: cita sin «»")
        cita = cita[1:-1]
        if len(cita.split()) >= 15:
            err.append(f"clasificacion {k}: cita de {len(cita.split())} palabras (menos de 15)")
        txt = re.sub(r"\s+", " ", textos.get(f["tarea"], {}).get(ids[k], ""))
        if re.sub(r"\s+", " ", cita) not in txt:
            err.append(f"clasificacion {k}: la cita no esta en informe-revision-{ids[k]}.md")
    for k in sorted(set(ids) - vistos):
        err.append(f"clasificacion: falta {k}")
    return err


def mutaciones_clasificacion(texto):
    f = re.search(r"^\| TASK-009 \| IMP-2 \|.*$", texto, re.M).group(0)
    return [
        ("clase-fuera-de-lista", texto.replace(f, f.replace("| puerta-seguridad |", "| seguridad |")),
         "clase fuera de la lista cerrada"),
        ("cita-inexistente", texto.replace(f, f.replace("variantes clásicas", "variantes modernas")), "la cita no esta"),
        ("fila-borrada", texto.replace(f + "\n", ""), "clasificacion: falta ('TASK-009', 'IMP-2')"),
        ("fila-duplicada", texto.replace(f, f + "\n" + f), "fila repetida"),
        ("ronda-cambiada", texto.replace(f, f.replace("| IMP-2 | 1 |", "| IMP-2 | 2 |")), "aparece por primera vez en la 1"),
    ]


def roles_plan(repo, t):
    d = os.path.join(repo, "tareas", "04-terminadas", t, "planificacion")
    fic = {}
    for raiz, _ds, fs in os.walk(d):
        for f in fs:
            rel = os.path.relpath(os.path.join(raiz, f), os.path.dirname(d)).replace("\\", "/")
            fic[rel] = open(os.path.join(raiz, f), encoding="utf-8").read()
    roles, _r = uso_016(fic)
    pts = None
    for txt in fic.values():
        m = re.search(r"Heuristica \((\d+) puntos?\)", txt)
        if m:
            pts = int(m.group(1))
    return roles, pts


def coste_diseno(texto):
    for linea in texto.splitlines():
        c = [x.strip() for x in linea.strip().strip("|").split("|")]
        if len(c) >= 9 and c[1] == "diseno" and c[2].startswith("agente:"):
            return int(c[8]), len(c[2][len("agente:"):].split(","))
    return None, None


def seccion_r(repo, h, tablas):
    sep("R: retrospectiva de las 15 tareas de la Fase 1 (roles, heuristica, IMPORTANTES por clase, revisor)")
    textos, ids, revs = informes_fase1(repo)
    ctexto = open(CLASIF, encoding="utf-8").read()
    filas, err = leer_clasificacion(ctexto)
    err += errores_clasificacion(filas, ids, textos)
    for e in err:
        falla("R", e)
    print(f"IMPORTANTES distintos en las tablas de los informes: {len(ids)} ; filas de la clasificacion: {len(filas)} ; "
          f"errores: {len(err)}")
    for nombre, mut, esperado in mutaciones_clasificacion(ctexto):
        f2, e2 = leer_clasificacion(mut)
        e2 += errores_clasificacion(f2, ids, textos)
        ok = any(esperado in e for e in e2)
        print(f"  mutacion {nombre}: {'falla como se espera' if ok else 'NO FALLA'}")
        if not ok:
            falla("R", f"la mutacion {nombre} no la detecta el guion")
    clase = {(f["tarea"], f["id"]): f for f in filas}
    retro = {}
    print("tarea     adr    roles                    pts  nivel    pts-peticion  IMP  " + " ".join(c[:6] for c in CLASES)
          + "  revisor                 coste-diseno/agente")
    cmp_ok = cmp_n = 0
    for t in FASE1:
        tt = leer_tarea(open(os.path.join(repo, "tareas", "04-terminadas", t, "tarea.md"), encoding="utf-8").read())
        p = puntuar(tt, h)
        roles, pp = roles_plan(repo, t)
        if pp is not None:
            cmp_n += 1
            cmp_ok += pp == p
            if pp != p:
                falla("R", f"{t}: la heuristica reimplementada da {p} y la peticion de la 0.16 dijo {pp}")
        mios = [clase[k] for k in sorted(clase) if k[0] == t]
        por = {c: sum(1 for f in mios if f["clase"] == c) for c in CLASES}
        tot, nag = coste_diseno(open(os.path.join(repo, "tareas", "04-terminadas", t, "tarea.md"), encoding="utf-8").read())
        txt_t = open(os.path.join(repo, "tareas", "04-terminadas", t, "tarea.md"), encoding="utf-8").read()
        pal016 = [w for w in h["palabras"] if normalizar(w) in normalizar("\n".join([tt["objetivo"], *tt["criterios"]]))]
        impl_ag = bool(re.search(r"^\| [^|]+ \| implementacion \| agente:", txt_t, re.M))
        retro[t] = {"pal016": pal016, "impl_agente": impl_ag,
                    "tarea": tt, "roles": roles, "puntos": p, "nivel": nivel(p, h), "imp": len(mios), "por": por,
                    "revisor": sorted(revs[t]), "coste": tot, "agentes": nag,
                    "nuevos_r2": [f for f in mios if f["ronda"] != "1"]}
        print(f"{t}  {ADR_DE[t]:5}  {str(len(roles)) + ' (' + ', '.join(roles) + ')':24} {p:3}  {nivel(p, h):8} "
              f"{str(pp) if pp is not None else '-':>12}  {len(mios):3}  " + " ".join(f"{por[c]:6}" for c in CLASES)
              + f"  {', '.join(sorted(revs[t])):22}  {tot // nag if tot else '-'}")
    tot_c = {c: sum(r["por"][c] for r in retro.values()) for c in CLASES}
    ps = tot_c["puerta-seguridad"]
    ps1 = sum(r["por"]["puerta-seguridad"] for r in retro.values() if len(r["roles"]) == 1)
    con_ps = [t for t, r in retro.items() if r["por"]["puerta-seguridad"]]
    nuevos = [f for r in retro.values() for f in r["nuevos_r2"]]
    nuevos_ps = [f for f in nuevos if f["clase"] == "puerta-seguridad"]
    niveles = {n: sum(1 for r in retro.values() if r["nivel"] == n) for n in NIVELES}
    sin_riesgos = sum(r["por"]["puerta-seguridad"] for r in retro.values() if "riesgos" not in r["roles"])
    print(f"heuristica reimplementada frente a la que anoto la 0.16 en la peticion de plan: {cmp_ok}/{cmp_n} iguales")
    print("IMPORTANTES por clase: " + " ; ".join(f"{c}={tot_c[c]}" for c in CLASES) + f" ; total={sum(tot_c.values())}")
    print(f"puerta-seguridad: {ps} de {sum(tot_c.values())} ; en planes de un solo rol: {ps1} ; en planes sin rol de riesgos "
          f"(escapados): {sin_riesgos} ; tareas con al menos uno: {len(con_ps)} ({' '.join(con_ps)})")
    print(f"IMPORTANTES que aparecen en una ronda de verificacion (2 o mas): {len(nuevos)} ; de ellos puerta-seguridad: "
          f"{len(nuevos_ps)}")
    print("nivel de la heuristica: " + " ; ".join(f"{n}={niveles[n]}" for n in NIVELES))
    print(f"revisor usado: {sorted({x for r in retro.values() for x in r['revisor']})} en las 15")
    con_pal = [t for t, r in retro.items() if r["pal016"]]
    print(f"tareas con alguna palabra de riesgo de la 0.16 en objetivo o criterios: {len(con_pal)} "
          f"({' '.join(t + ':' + ','.join(retro[t]['pal016']) for t in con_pal)})")
    pal_puertas = sorted({normalizar(v) for _i, d, sen, v in tablas["dominios"] if d == "puertas-seguridad" and sen == "palabra"})
    comunes = sorted(set(pal_puertas) & {normalizar(x) for x in h["palabras"]})
    print(f"palabras de riesgo de la 0.16 (scripts/heuristica-complejidad.yml del plugin 0.16.1): {len(h['palabras'])} ; "
          f"palabras de puertas-seguridad en tabla-dominios: {len(pal_puertas)} ; en las dos listas: {comunes}")
    impl = [t for t, r in retro.items() if r["impl_agente"]]
    print(f"tareas con fila `implementacion | agente:` en ## Coste (la implementacion la hizo un subagente): {len(impl)} de 15")
    cifras = {"imp": sum(tot_c.values()), "por": tot_c, "ps": ps, "ps1": ps1, "con_ps": con_ps, "escapados": sin_riesgos,
              "nuevos": len(nuevos), "nuevos_ps": len(nuevos_ps), "niveles": niveles,
              "simples": niveles["simple"], "un_rol": sum(1 for r in retro.values() if len(r["roles"]) == 1),
              "con_pal": len(con_pal), "impl": len(impl), "comunes": comunes}
    return retro, cifras


def fila_retro(t, r):
    p = r["por"]
    return (f"| {t} | {ADR_DE[t]} | {len(r['roles'])} ({', '.join(r['roles'])}) | {r['puntos']} ({r['nivel']}) | {r['imp']} | "
            f"{p['puerta-seguridad']} | {p['contrato']} | {p['correccion']} | {p['fidelidad-evidencia']} | "
            f"{p['reproducibilidad']} | {', '.join('`' + x + '`' for x in r['revisor'])} |")


# =========================================================================== D

def suelo_de(tablas, dom):
    """El suelo de roles que tabla-seleccion da a un dominio (fila con `dominio=<dom>` y `suelo=n`)."""
    for f in tablas["seleccion"].values():
        if f"dominio={dom}" in f["cond"] and f["accion"] and f["accion"][0].startswith("suelo="):
            return int(f["accion"][0].split("=")[1])
    return 0


def seccion_d(repo, retro, tablas):
    sep("D: detector dominios() con la tabla-dominios del ADR sobre las 15 tareas de la Fase 1")
    cambios = cambios_por_tarea(repo, FIJO["TaskCodeTwo"])
    tp = fp = fn = tn = 0
    dispara, suelo = [], []
    for t in FASE1:
        r = retro[t]
        doms = dominios(r["tarea"], cambios.get(t, []), tablas["dominios"], {})
        nd = {d for d, _s in doms}
        pos = r["por"]["puerta-seguridad"] > 0
        det = "puertas-seguridad" in nd
        tp += pos and det
        fp += det and not pos
        fn += pos and not det
        tn += not pos and not det
        if det:
            dispara.append(t)
            if len(r["roles"]) < suelo_de(tablas, "puertas-seguridad"):
                suelo.append(t)
        sen = "; ".join(f"{d}: {', '.join(s[:4])}{' …' if len(s) > 4 else ''}" for d, s in doms) or "—"
        rutas = sorted({x.split("/")[0] for _e, rs in cambios.get(t, []) for x in rs})
        print(f"{t}  positivo={'si' if pos else 'no'}  detecta={'si' if det else 'no'}  rutas={rutas}  senales={sen}")
    costes = [r["coste"] / r["agentes"] for r in retro.values() if r["coste"]]
    medio = int(round(sum(costes) / len(costes)))
    extra = len(suelo) * 2 * medio
    print(f"verdaderos positivos={tp} falsos positivos={fp} falsos negativos={fn} verdaderos negativos={tn}")
    print(f"recall {tp}/{tp + fn} ; precision {tp}/{tp + fp}")
    print(f"tareas en las que salta puertas-seguridad: {len(dispara)} ; con el suelo de 2 roles subirian de 1 a 2 roles "
          f"(mas unificador): {len(suelo)} ({' '.join(suelo)})")
    print(f"coste medio de un agente de diseno (filas diseno de ## Coste, total / agentes, {len(costes)} tareas): {medio} tokens")
    print(f"coste del suelo: {len(suelo)} tareas x 2 agentes (riesgos y unificador) x {medio} = {extra} tokens "
          f"({extra / 1e6:.1f} M; {extra / len(FASE1) / 1e6:.2f} M por tarea de media)")
    return {"tp": tp, "fp": fp, "fn": fn, "tn": tn, "dispara": len(dispara), "suelo": len(suelo), "suelo_ids": suelo,
            "medio": medio, "extra": extra}


# =========================================================================== I

def items_dir(base, prefijo, origen):
    out = []
    sk, ag = os.path.join(base, "skills"), os.path.join(base, "agents")
    if os.path.isdir(sk):
        for d in sorted(os.listdir(sk)):
            if os.path.isfile(os.path.join(sk, d, "SKILL.md")):
                out.append({"id": prefijo + d, "tipo": "skill", "origen": origen, "ruta": os.path.join(sk, d, "SKILL.md")})
    if os.path.isdir(ag):
        for f in sorted(os.listdir(ag)):
            if f.endswith(".md") and os.path.isfile(os.path.join(ag, f)):
                out.append({"id": prefijo + f[:-3], "tipo": "agente", "origen": origen, "ruta": os.path.join(ag, f)})
    return out


def inventario_maquina():
    items = items_dir(PLUGIN, "taskcode-plugin:", "plugin taskcode 0.16.1")
    inst = json.load(open(os.path.join(CLAUDE, "plugins", "installed_plugins.json"), encoding="utf-8"))
    for clave in sorted(inst.get("plugins", {})):
        nombre = clave.split("@")[0]
        if nombre == "taskcode-plugin":
            continue
        for e in inst["plugins"][clave]:
            items += items_dir(e["installPath"], nombre + ":", f"plugin {clave}")
    items += items_dir(CLAUDE, "", "usuario (~/.claude)")
    items += [{"id": x, "tipo": "agente", "origen": "integrado", "ruta": None} for x in INTEGRADOS]
    return items


def bloque_yaml(texto):
    m = re.search(r"^```yaml\n(.*?)^```", texto.replace("\r\n", "\n"), re.S | re.M)
    if not m:
        return {}
    out = {}
    for l in m.group(1).splitlines():
        l = re.sub(r"\s+#.*$", "", l).strip()
        x = re.match(r"^(\w+):\s*(.*)$", l)
        if x:
            v = x.group(2).strip()
            out[x.group(1)] = [y.strip().strip('"') for y in v.strip("[]").split(",") if y.strip()] if v.startswith("[") else v
    return out


def catalogo(items):
    """Metadatos que hoy existen: bloque yaml de las revisoras del plugin, catalogo-skills.yml de la 0.16.1 y el
    nombre de los agentes de brainstorm. Lo instalado sin metadatos no entra (no esta declarado)."""
    cat = {}
    cy = open(os.path.join(PLUGIN, "scripts", "catalogo-skills.yml"), encoding="utf-8").read()
    planos = dict(re.findall(r"^(skill_\d+_\w+):\s*(.*?)\s*$", cy, re.M))
    for k, v in planos.items():
        if k.endswith("_id"):
            n = k[:-3]
            ide = "taskcode-plugin:" + v
            etq = planos.get(n + "_etiquetas", "[]").strip("[]")
            cat[ide] = {"id": ide, "tipo": "skill", "rol": planos.get(n + "_rol"),
                        "prioridad": int(planos.get(n + "_prioridad", "0")),
                        "etiquetas": [x.strip() for x in etq.split(",") if x.strip()], "dominios": []}
    for it in items:
        if not it["id"].startswith("taskcode-plugin:"):
            continue
        if it["tipo"] == "skill":
            b = bloque_yaml(open(it["ruta"], encoding="utf-8").read())
            if b.get("rol") == "revisor":
                e = cat.setdefault(it["id"], {"id": it["id"], "tipo": "skill", "prioridad": 0, "etiquetas": [],
                                              "dominios": []})
                e.update({"rol": "revisor", "patrones": b.get("patrones_archivo") or [],
                          "fallback": b.get("fallback") == "true", "umbral": int(b.get("umbral_dominios", "3"))})
        else:
            m = re.fullmatch(r"taskcode-plugin:brainstorm-([a-z]+)", it["id"])
            if m:
                cat[it["id"]] = {"id": it["id"], "tipo": "agente", "rol": "diseno", "rol_diseno": m.group(1)}
    for x in INTEGRADOS:
        cat[x] = {"id": x, "tipo": "agente", "rol": "lanzador-revision"}
    return sorted(cat.values(), key=lambda e: e["id"])


def seccion_i(repo, trabajo, clones):
    sep("I: inventario real de la maquina y de los proyectos (solo lectura)")
    items = inventario_maquina()
    por = {}
    for it in items:
        por.setdefault((it["origen"], it["tipo"]), []).append(it["id"])
    for (o, ti), ids in sorted(por.items()):
        print(f"  {o:45} {ti:6} {len(ids):3}  {' '.join(sorted(ids)[:6])}{' …' if len(ids) > 6 else ''}")
    ids = sorted({it["id"] for it in items})
    if len(ids) != len(items):
        falla("I", "ids repetidos en el inventario")
    print(f"total: {len(items)} ({sum(1 for i in items if i['tipo'] == 'skill')} skills, "
          f"{sum(1 for i in items if i['tipo'] == 'agente')} agentes) ; hash {h12(ids)}")
    proyectos = {"TaskCodeTwo": sorted(os.listdir(os.path.join(repo, ".claude"))) if os.path.isdir(os.path.join(repo, ".claude")) else []}
    for p in PILOTOS:
        proyectos[p] = git(clones[p], "ls-tree", "-r", "--name-only", FIJO[p], "--", ".claude").split()
    for p, fs in proyectos.items():
        sk = [f for f in fs if re.match(r"(\.claude/)?skills/", f)]
        ag = [f for f in fs if re.match(r"(\.claude/)?agents/", f)]
        print(f"  proyecto {p}: .claude/ = {fs or 'no existe'} ; skills del proyecto: {len(sk)} ; agentes del proyecto: {len(ag)}")
    cat = catalogo(items)
    print(f"catalogo (lo instalado con metadatos): {len(cat)} entradas ; hash {h12(cat)}")
    for e in cat:
        extra = ""
        if e.get("rol") == "revisor":
            extra = f" patrones={len(e['patrones'])} fallback={e['fallback']}" + (f" umbral={e['umbral']}" if e["fallback"] else "")
        elif e.get("rol") == "ejecucion":
            extra = f" etiquetas={e['etiquetas']} prioridad={e['prioridad']}"
        print(f"  {e['id']:45} {e['tipo']:6} rol={e.get('rol')}{' rol_diseno=' + e['rol_diseno'] if e.get('rol_diseno') else ''}{extra}")
    sin_meta = [i for i in ids if i not in {e["id"] for e in cat}]
    print(f"instalado sin metadatos de seleccion (no declarado): {len(sin_meta)} ; entre ellos con «secur» en el id: "
          f"{[i for i in sin_meta if 'secur' in i]}")
    faltan = [e["id"] for e in cat if e["id"] not in ids]
    if faltan:
        falla("I", f"el catalogo nombra algo que no esta instalado: {faltan}")
    return items, ids, cat


# =========================================================================== T

ESTRATOS = {
    "ToolsaWeb": ["skills-016", "fix", "riesgo-alto", "solo-docs", "multidominio"],
    "OpenDockTrading": ["seguridad", "riesgo-alto", "solo-docs", "multidominio", "un-dominio"],
}
PALABRAS_SEGURIDAD = ["seguridad", "autenticacion", "autorizacion", "cifrado", "datos personales"]


def grupos_016(cambios, cat):
    """Grupos de clasificarPorDominio de la 0.16: una revisora por patrones y el grupo «sin dominio»."""
    revs = [e for e in cat if e.get("rol") == "revisor" and e.get("patrones")]
    g = set()
    for _e, rs in cambios:
        for r in rs:
            hit = [e["id"] for e in revs if any(casa(p, r) for p in e["patrones"])]
            g |= set(hit) if hit else {"sin-dominio"}
    return g


def cumple(estrato, t, cambios, cat):
    minimo = carril_minimo(cambios)
    texto = normalizar("\n".join([t["objetivo"], *t["criterios"]]))
    g = grupos_016(cambios, cat)
    return {
        "solo-docs": minimo == "errata",
        "fix": t["tipo"] == "fix",
        "multidominio": t["tipo"] == "feature" and len(g) >= 2,
        "un-dominio": t["tipo"] == "feature" and len(g) == 1 and minimo == "fix",
        "riesgo-alto": minimo == "riesgo-alto",
        "skills-016": bool(t["skills_recomendados"]),
        "seguridad": any(w in texto for w in PALABRAS_SEGURIDAD) or "seguridad" in [normalizar(e) for e in t["etiquetas"]],
    }[estrato]


def muestra(repo_nombre, tareas, cambios, cat):
    elegidas = []
    for est in ESTRATOS[repo_nombre]:
        cands = sorted((int(i[5:]), i) for i in tareas if cambios.get(i) and i not in [e for _s, e in elegidas]
                       and cumple(est, tareas[i], cambios[i], cat))
        if not cands:
            elegidas.append((est, None))
        else:
            elegidas.append((est, cands[-1][1]))
    return elegidas


def corto(x):
    return x.split(":")[-1].replace("brainstorm-", "")


def celda_diseno(s):
    if not s["agentes"]:
        return "0 (sin plan)"
    roles = [corto(a) for a in s["agentes"] if corto(a) != "unificador"]
    txt = f"{len(roles)}: {', '.join(roles)}" + (" + unificador" if any(corto(a) == "unificador" for a in s["agentes"]) else "")
    return txt + (f"; skills {', '.join(corto(k) for k in s['skills'])}" if s["skills"] else "")


def celda_revision(s):
    txt = ", ".join(corto(k) for k in s["skills"]) or "—"
    avisos = [a.split(":", 1)[1] for a in s["avisos"] if a.startswith("dominio-sin-revisora:")]
    return txt + (f"; aviso: {', '.join(avisos)} sin revisora" if avisos else "")


def fila_muestra(r, sel=None):
    s = sel or r["sel"]
    m = " / ".join(s[k]["modelo"] or "—" for k in ("diseno", "implementacion", "revision"))
    reglas = " · ".join(" ".join(s[k]["reglas"]) for k in ("diseno", "implementacion", "revision"))
    doms = ", ".join(x.replace(":", " (", 1) + ")" for x in s["diseno"]["senales"]) or "—"
    u = r["uso"]
    uso = (f"roles {len(u['roles'])} ({', '.join(u['roles']) or '—'}); rev {', '.join(u['revs']) or '—'}; "
           f"modelo {u['modelo']}; skills {', '.join(u['skills']) or '—'}")
    return [f"{r['repo']}/{r['id']}", r["estrato"], f"{r['declarado']} → {r['efectivo']}", f"{r['cx']} ({r['puntos']} p)",
            doms, celda_diseno(s["diseno"]), ", ".join(corto(k) for k in s["implementacion"]["skills"]) or "—",
            celda_revision(s["revision"]), m, reglas, uso]


def seccion_t(clones, h, ids, cat, tablas):
    sep("T: las 10 tareas reales (muestreo estratificado fijado en el ADR)")
    filas = []
    for p in PILOTOS:
        arch = archivo_tareas(clones[p], FIJO[p])
        tareas = {i: leer_tarea(f["tarea.md"]) for i, f in arch.items() if "tarea.md" in f}
        cambios = cambios_por_tarea(clones[p], FIJO[p])
        print(f"--- {p} @ {FIJO[p][:9]}: {len(tareas)} tareas terminadas ; con merge y ficheros fuera de tareas/: "
              f"{sum(1 for i in tareas if cambios.get(i))}")
        for est, i in muestra(p, tareas, cambios, cat):
            if i is None:
                falla("T", f"{p}: estrato {est} vacio")
                continue
            t, cb = tareas[i], cambios[i]
            dec = "fix" if t["tipo"] == "fix" else "funcionalidad"
            mi = carril_minimo(cb)
            efe = carril_efectivo(dec, mi)
            pts = puntuar(t, h)
            cx = cx_efectiva(t, pts, h)
            sel = {rol: seleccionar(rol, t, efe, cx, cb, ids, cat, {}, tablas) for rol in ROL_COL}
            roles, revs = uso_016(arch[i])
            r = {"repo": p, "id": i, "estrato": est, "declarado": dec, "minimo": mi, "efectivo": efe, "puntos": pts,
                 "cx": cx, "t": t, "cambios": cb, "sel": sel,
                 "uso": {"roles": roles, "revs": revs, "modelo": t["modelo_sugerido"], "skills": t["skills_recomendados"]}}
            filas.append(r)
            print(f"  {i} [{est}] «{t['titulo'][:70]}» ficheros={sum(len(x[1]) for x in cb)} grupos-0.16="
                  f"{sorted(corto(x) for x in grupos_016(cb, cat))}")
    print()
    print("| Tarea | Estrato | Carril (declarado → efectivo) | Complejidad | Dominios (señal) | Diseño | Implementación | "
          "Revisión | Modelo D / I / R | Reglas D · I · R | Lo que usó la 0.16 | Comprobado a mano (Carlos) |")
    print("|---|---|---|---|---|---|---|---|---|---|---|---|")
    for r in filas:
        print("| " + " | ".join(fila_muestra(r) + [MANO]) + " |")
    return filas


# =========================================================================== S

GUARDIA = {"activa": False, "eventos": []}


def _auditoria(evento, _args):
    if GUARDIA["activa"] and (evento.startswith(("socket.", "subprocess.", "urllib.", "http.", "ftplib.", "smtplib."))
                              or evento in ("os.system", "os.exec", "os.posix_spawn", "os.spawn", "os.startfile")):
        GUARDIA["eventos"].append(evento)
        raise RuntimeError(f"guardia: {evento} durante seleccionar")


def todas(filas, ids, cat, tablas, cfg=None):
    out = {}
    for r in filas:
        for rol in ROL_COL:
            out[f"{r['repo']}/{r['id']}/{rol}"] = seleccionar(rol, r["t"], r["efectivo"], r["cx"], r["cambios"], ids, cat,
                                                              cfg or {}, tablas)
    return out


PUERTAS = "Puerta del check obligatorio: token de operador y aprobacion de la persona"
EVASIVO = "Validacion obligatoria: credencial del operador y conformidad de la persona"


def _t(obj, etq=(), aprobado=None):
    t = {"objetivo": obj, "criterios": ["criterio"], "etiquetas": list(etq), "tipo": "feature"}
    if aprobado:
        t["aprobado"] = aprobado
    return t


def _correr(casos, tablas, ids, cat, salidas=None):
    """casos: (nombre, rol, t, carril, cx, cambios, extra{inv,cat,cfg,ronda}, comprobar(s) -> bool, que). Pura."""
    err, vistas, lin = [], set(), []
    for nombre, rol, t, carril, cx, cambios, ex, comprobar, que in casos:
        try:
            s = seleccionar(rol, t, carril, cx, cambios, ex.get("inv", ids), ex.get("cat", cat), ex.get("cfg", {}), tablas,
                            ronda=ex.get("ronda", 1))
        except (ErrorTabla, ErrorConfig) as e:
            err.append(f"caso {nombre}: {e}")
            continue
        vistas.update(s["reglas"])
        if salidas is not None:
            salidas[nombre] = s
        ok = comprobar(s)
        lin.append(f"  {nombre}: {'bien' if ok else 'MAL'} ({que}) -> agentes={[corto(a) for a in s['agentes']]} "
                   f"skills={[corto(k) for k in s['skills']]} modelo={s['modelo']} reglas={s['reglas']} avisos={s['avisos']}")
        if not ok:
            err.append(f"caso {nombre}: no da lo esperado ({que})")
    return err, vistas, lin


def controles_reglas(tablas, ids, cat, salidas=None):
    """Un caso por fila de tabla-seleccion (y cuatro de S13). Pura: la usa S y la usa G con las tablas de cada copia
    mutada del ADR. Devuelve (errores, reglas vistas, lineas)."""
    src, txt, doc = [("M", ["src/x.rs"])], [("M", ["src/x.txt"])], [("M", ["docs/a.md"])]
    roles = lambda s: [corto(a) for a in s["agentes"] if corto(a) != "unificador"]
    zd = {"id": "zz-diseno-puertas", "tipo": "skill", "rol": "diseno", "dominios": ["puertas-seguridad"]}
    zr = {"id": "zz-revisora-puertas", "tipo": "skill", "rol": "revisor", "dominios": ["puertas-seguridad"], "patrones": []}
    cq = "taskcode-plugin:code-quality-reviewer"
    casos = [
        ("S01 errata sin plan", "diseno", _t("Errata en la guia"), "errata", "simple", doc, {},
         lambda s: s["agentes"] == [] and "S01" in s["reglas"] and "S02" not in s["reglas"], "0 agentes, S01"),
        ("S02 fix con un rol", "diseno", _t("Arreglo"), "fix", "simple", src, {},
         lambda s: roles(s) == ["arquitectura"] and "S02" in s["reglas"], "arquitectura, S02"),
        ("S03 suelo por puertas", "diseno", _t(PUERTAS), "funcionalidad", "simple", src, {},
         lambda s: roles(s) == ["arquitectura", "riesgos"] and "S03" in s["reglas"], "arquitectura y riesgos, S03"),
        ("S04 rendimiento", "diseno", _t("Malla grande", ["wasm"]), "funcionalidad", "simple", src, {},
         lambda s: "rendimiento" in [corto(a) for a in s["agentes"]] and {"S04", "S05"} <= set(s["reglas"]),
         "rol de rendimiento y unificador, S04 y S05"),
        ("S06 skill de diseno del dominio", "diseno", _t(PUERTAS), "funcionalidad", "simple", src,
         {"cat": cat + [zd], "inv": ids + [zd["id"]]},
         lambda s: zd["id"] in s["skills"] and "S06" in s["reglas"], "zz-diseno-puertas, S06"),
        ("S07 skill de ejecucion", "implementacion", _t("x", ["planificacion"]), "funcionalidad", "simple", src, {},
         lambda s: s["skills"] == ["taskcode-plugin:task-workflow"] and "S07" in s["reglas"], "task-workflow, S07"),
        ("S08 revisoras por rutas", "revision", _t("x"), "funcionalidad", "simple", src + doc, {},
         lambda s: s["skills"] == ["taskcode-plugin:rust-wasm-reviewer", cq] and "S08" in s["reglas"],
         "rust-wasm y generica, S08"),
        ("S09 y S12 generica y lanzador", "revision", _t("x"), "funcionalidad", "simple", txt, {},
         lambda s: s["skills"] == [cq] and s["agentes"] == ["general-purpose"] and {"S09", "S12"} <= set(s["reglas"]),
         "generica y general-purpose, S09 y S12"),
        ("S10 revisora declarada del dominio", "revision", _t(PUERTAS), "funcionalidad", "simple", txt,
         {"cat": cat + [zr], "inv": ids + [zr["id"]]},
         lambda s: zr["id"] in s["skills"] and "S10" in s["reglas"] and "S11" not in s["reglas"],
         "zz-revisora-puertas sin aviso, S10"),
        ("S11 dominio sin revisora", "revision", _t(PUERTAS), "funcionalidad", "simple", txt, {},
         lambda s: "dominio-sin-revisora:puertas-seguridad" in s["avisos"] and "S11" in s["reglas"], "aviso, S11"),
        ("S13 ronda 2 sin dominio baja", "revision", _t("Pantalla nueva"), "funcionalidad", "media", txt, {"ronda": 2},
         lambda s: s["modelo"] == "sonnet" and "S13" in s["reglas"], "opus -> sonnet, S13"),
        ("S13 ronda 2 en riesgo-alto no baja", "revision", _t("Pantalla nueva"), "riesgo-alto", "media",
         txt, {"ronda": 2},
         lambda s: s["modelo"] == "opus" and "S13" not in s["reglas"], "sigue opus"),
        ("S13 ronda 2 con puertas por texto no baja", "revision", _t(PUERTAS), "funcionalidad", "media", txt, {"ronda": 2},
         lambda s: s["modelo"] == "opus" and "S13" not in s["reglas"], "sigue opus"),
        ("S13 ronda 2 con puertas por ruta no baja", "revision", _t("Ajuste de la sesion"), "funcionalidad", "media",
         [("M", [".claude/settings.json"])], {"ronda": 2},
         lambda s: s["modelo"] == "opus" and "S13" not in s["reglas"], "sigue opus"),
        ("S13 ronda 2 con puertas por ruta en mayusculas no baja", "revision", _t("Ajuste de la sesion"),
         "funcionalidad", "media", [("M", [".Claude/settings.json"])], {"ronda": 2},
         lambda s: s["modelo"] == "opus" and "S13" not in s["reglas"], "sigue opus: .Claude/ es .claude/"),
        ("S14 no instalado", "revision", _t("x"), "funcionalidad", "simple", txt, {"inv": [i for i in ids if i != cq]},
         lambda s: f"no-instalado:{cq}" in s["avisos"] and cq not in s["skills"] and "S14" in s["reglas"],
         "se quita con aviso, S14"),
        ("S15 modelos_permitidos", "diseno", _t("x"), "funcionalidad", "media", src,
         {"cfg": {"modelos_permitidos": ["haiku", "sonnet"]}},
         lambda s: s["modelo"] == "sonnet" and "S15" in s["reglas"], "opus -> sonnet, S15"),
    ]
    # Por carril (MEN-9): las filas de implementacion, revision y todos no dependen del carril salvo S13 en
    # riesgo-alto; un caso por carril hace que anadir «y no carril=X» a cualquiera de ellas cambie un resultado.
    for c in ORDEN:
        m_rev, _f = modelo_de(c, "simple", "revision", tablas["modelo"])
        casos += [
            (f"S07 en {c}", "implementacion", _t("x", ["planificacion"]), c, "simple", src, {},
             lambda s: s["skills"] == ["taskcode-plugin:task-workflow"], "task-workflow"),
            (f"S08 en {c}", "revision", _t("x"), c, "simple", src + doc, {},
             lambda s: s["skills"] == ["taskcode-plugin:rust-wasm-reviewer", cq], "rust-wasm y generica"),
            (f"S09 y S12 en {c}", "revision", _t("x"), c, "simple", txt, {},
             lambda s: s["skills"] == [cq] and s["agentes"] == ["general-purpose"], "generica y general-purpose"),
            (f"S10 en {c}", "revision", _t(PUERTAS), c, "simple", txt, {"cat": cat + [zr], "inv": ids + [zr["id"]]},
             lambda s: zr["id"] in s["skills"], "zz-revisora-puertas"),
            (f"S11 en {c}", "revision", _t(PUERTAS), c, "simple", txt, {},
             lambda s: "dominio-sin-revisora:puertas-seguridad" in s["avisos"], "aviso"),
            (f"S14 en {c}", "revision", _t("x"), c, "simple", txt, {"inv": [i for i in ids if i != cq]},
             lambda s: f"no-instalado:{cq}" in s["avisos"], "aviso no-instalado"),
            (f"S15 en {c}", "revision", _t("x"), c, "simple", txt, {"cfg": {"modelos_permitidos": ["haiku"]}},
             lambda s: s["modelo"] == "haiku" and "S15" in s["reglas"], "restringido a haiku"),
        ]
        if c != "riesgo-alto" and m_rev:
            baja = ESCALA[max(0, ESCALA.index(m_rev) - 1)]
            casos.append((f"S13 en {c}", "revision", _t("x"), c, "simple", txt, {"ronda": 2},
                          lambda s, b=baja: s["modelo"] == b and "S13" in s["reglas"], f"{m_rev} -> {baja}"))
    err, vistas, lin = _correr(casos, tablas, ids, cat, salidas)
    for i in tablas["seleccion"]:
        if i not in vistas:
            err.append(f"la fila {i} de tabla-seleccion no tiene ningun caso que la ejercite")
    return err, vistas, lin


def caso_vocabulario(tablas, ids, cat, salidas=None):
    """IMP-3: el mismo trabajo con otro vocabulario. Dice que contiene la defensa y que no (lo esperado esta fijado)."""
    aprobado = {"objetivo": PUERTAS, "criterios": ["criterio"], "etiquetas": []}
    roles = lambda s: [corto(a) for a in s["agentes"] if corto(a) != "unificador"]
    casos = [
        ("1 plan, texto con puertas", "diseno", _t(PUERTAS), "funcionalidad", "simple", [("M", ["src/x.rs"])], {},
         lambda s: len(roles(s)) == 2, "contenido: suelo de 2 roles"),
        ("2 plan, mismo trabajo con otro vocabulario", "diseno", _t(EVASIVO), "funcionalidad", "simple",
         [("M", ["src/x.rs"])], {}, lambda s: len(roles(s)) == 1,
         "NO lo contiene el binario: 1 rol; solo la persona en approve, que ve «dominios: ninguno»"),
        ("3 revision ronda 2, aprobado con puertas y texto vivo reescrito", "revision", _t(EVASIVO, aprobado=aprobado),
         "funcionalidad", "media", [("M", ["src/x.rs"])], {"ronda": 2},
         lambda s: s["modelo"] == "opus" and "dominio-sin-revisora:puertas-seguridad" in s["avisos"],
         "contenido: el texto aprobado suma; sigue opus y el aviso"),
        ("3b revision ronda 2, etiqueta `puertas` aprobada y quitada del texto vivo", "revision",
         _t(EVASIVO, aprobado={"objetivo": EVASIVO, "criterios": ["criterio"], "etiquetas": ["puertas"]}),
         "funcionalidad", "media", [("M", ["src/x.rs"])], {"ronda": 2},
         lambda s: s["modelo"] == "opus" and "dominio-sin-revisora:puertas-seguridad" in s["avisos"],
         "contenido: la etiqueta aprobada suma"),
        ("4 revision ronda 2, otro vocabulario y diff en .claude/", "revision", _t(EVASIVO), "funcionalidad", "media",
         [("M", [".claude/settings.json"])], {"ronda": 2},
         lambda s: s["modelo"] == "opus" and "dominio-sin-revisora:puertas-seguridad" in s["avisos"],
         "contenido: la ruta del diff con la config de la base"),
        ("5 fix sin approve, otro vocabulario y diff solo en src/", "revision", _t(EVASIVO), "fix", "media",
         [("M", ["src/auth/x.rs"])], {"ronda": 2},
         lambda s: s["modelo"] == "sonnet" and not s["avisos"],
         "NO contenido: sin dominio, sin aviso y S13 baja; riesgo aceptado"),
    ]
    return _correr(casos, tablas, ids, cat, salidas)


def caso_catalogo_pr(tablas, ids, cat, salidas=None):
    """MEN-4: un PR que anade a su arbol una generica y una revisora propias. Pura."""
    aaa = {"id": "aaa-generica", "tipo": "skill", "rol": "revisor", "fallback": True, "umbral": 9, "patrones": [],
           "dominios": []}
    zzz = {"id": "zzz-revisora-rs", "tipo": "skill", "rol": "revisor", "patrones": ["**/*.rs"], "dominios": []}
    cq = "taskcode-plugin:code-quality-reviewer"
    pr = {"cat": cat + [aaa, zzz], "inv": ids + [aaa["id"], zzz["id"]]}
    casos = [
        ("catalogo de la base, diff .txt", "revision", _t("x"), "funcionalidad", "simple", [("M", ["src/x.txt"])], {},
         lambda s: s["skills"] == [cq], "generica del plugin"),
        ("catalogo del PR con otra generica, diff .txt", "revision", _t("x"), "funcionalidad", "simple",
         [("M", ["src/x.txt"])], pr, lambda s: s["skills"] == [cq],
         "contenido por la funcion: la generica solo puede ser la del plugin"),
        ("catalogo del PR con una revisora propia, diff .rs", "revision", _t("x"), "funcionalidad", "simple",
         [("M", ["src/x.rs"])], pr, lambda s: zzz["id"] in s["skills"],
         "la funcion NO lo contiene: entra la revisora del PR; por eso el catalogo se lee de la base"),
    ]
    return _correr(casos, tablas, ids, cat, salidas)


def huella(tablas, ids, cat, filas):
    """Todo lo que la PoC calcula con unas tablas: casos de control, evasion, catalogo del PR y la muestra."""
    sal = {}
    controles_reglas(tablas, ids, cat, sal)
    caso_vocabulario(tablas, ids, cat, sal)
    caso_catalogo_pr(tablas, ids, cat, sal)
    for r in filas:
        for rol in ROL_COL:
            sal[f"{r['repo']}/{r['id']}/{rol}"] = seleccionar(rol, r["t"], r["efectivo"], r["cx"], r["cambios"], ids, cat,
                                                              {}, tablas)
    return h12(sal)


def cobertura_atomos(tablas, ids, cat, filas):
    """MEN-9: quitar cada atomo de cada fila (o dejar `siempre`) y anadir «no carril=X» a las filas de implementacion,
    revision y todos tiene que cambiar algun resultado o romper la lectura de la tabla. Devuelve (errores, cuentas)."""
    base = huella(tablas, ids, cat, filas)
    err, cuenta = [], [0, 0]

    def prueba(i, cond, que):
        cuenta[1] += 1
        t2 = dict(tablas)
        t2["seleccion"] = {k: (dict(v, cond=cond) if k == i else v) for k, v in tablas["seleccion"].items()}
        try:
            parsear_regla(i, cond, tablas["seleccion"][i]["accion"])
            h = huella(t2, ids, cat, filas)
        except (ErrorTabla, ErrorConfig, IndexError, KeyError):
            cuenta[0] += 1
            return
        if h == base:
            err.append(f"{i}: {que} no cambia ningun resultado (atomo sin caso que lo acote)")
        else:
            cuenta[0] += 1

    for i, f in tablas["seleccion"].items():
        cond = f["cond"]
        for k, a in enumerate(cond):
            if a == "siempre":
                continue
            resto = cond[:k] + cond[k + 1:]
            prueba(i, resto or ["siempre"], f"quitar «{a}»")
        if f["rol"] in ("implementacion", "revision", "todos"):
            for c in ORDEN:
                if any(x.endswith(f"carril={c}") for x in cond):
                    continue
                prueba(i, [x for x in cond if x != "siempre"] + [f"no carril={c}"], f"anadir «no carril={c}»")
    return err, tuple(cuenta)


def seccion_s(filas, ids, cat, tablas):
    sep("S: seleccionar determinista, sin red ni modelo")
    a = h12(todas(filas, ids, cat, tablas))
    b = h12(todas(filas, ids, cat, tablas))
    print(f"dos ejecuciones sobre las 10 tareas x 3 roles: {a} y {b} -> {'iguales' if a == b else 'DISTINTAS'}")
    if a != b:
        falla("S", "dos ejecuciones dan hash distinto")
    perm_ids = sorted(ids, key=lambda x: hashlib.sha256(x.encode()).hexdigest())
    perm_cat = sorted(cat, key=lambda e: hashlib.sha256(e["id"].encode()).hexdigest(), reverse=True)
    c = h12(todas(filas, perm_ids, perm_cat, tablas))
    print(f"inventario y catalogo permutados: {c} -> {'iguales' if c == a else 'DISTINTOS'}")
    if c != a:
        falla("S", "permutar inventario y catalogo cambia la salida")
    base = todas(filas, ids, cat, tablas)
    elegida = sorted({k for s in base.values() for k in s["skills"]})
    quitar = [k for k in elegida if "reviewer" in k and "code-quality" not in k]
    quitar = quitar[0] if quitar else elegida[0]
    sin = todas(filas, [i for i in ids if i != quitar], cat, tablas)
    afect = [k for k, s in base.items() if quitar in s["skills"]]
    ok = all(quitar not in sin[k]["skills"] and f"no-instalado:{quitar}" in sin[k]["avisos"] and "S14" in sin[k]["reglas"]
             for k in afect)
    print(f"quitar {quitar} del inventario: sale de {len(afect)} selecciones con aviso no-instalado y regla S14 -> "
          f"{'si' if ok and afect else 'NO'}")
    if not (ok and afect):
        falla("S", f"quitar {quitar} no lo saca con aviso")
    t_emp = {"objetivo": "Cambio de prueba", "criterios": ["criterio"], "etiquetas": ["comun"], "tipo": "feature"}
    cat_emp = cat + [{"id": n, "tipo": "skill", "rol": "ejecucion", "prioridad": 5, "etiquetas": ["comun"], "dominios": []}
                     for n in ("zeta-skill", "alfa-skill", "medio-skill")]
    inv_emp = ids + ["zeta-skill", "alfa-skill", "medio-skill"]
    GUARDIA["activa"] = True
    try:
        s = seleccionar("implementacion", t_emp, "funcionalidad", "simple", [("M", ["src/x.txt"])], inv_emp, cat_emp, {},
                        tablas)
        s2 = seleccionar("implementacion", t_emp, "funcionalidad", "simple", [("M", ["src/x.txt"])], list(reversed(inv_emp)),
                         list(reversed(cat_emp)), {}, tablas)
        todas(filas, ids, cat, tablas)
    finally:
        GUARDIA["activa"] = False
    ok = s["skills"] == ["alfa-skill"] and "empate-por-id:alfa-skill,medio-skill,zeta-skill" in s["avisos"] and s == s2
    print(f"empate total (3 skills, mismo solape y prioridad): elige {s['skills']} con aviso "
          f"{[x for x in s['avisos'] if x.startswith('empate')]} ; en orden inverso igual: {s == s2} -> "
          f"{'bien' if ok else 'MAL'}")
    if not ok:
        falla("S", "el empate no se resuelve por id lexicografico con aviso")
    print(f"guardia de auditoria (socket, subprocess, urllib, http, os.system) durante seleccionar: "
          f"{len(GUARDIA['eventos'])} eventos")
    if GUARDIA["eventos"]:
        falla("S", f"seleccionar intento salir: {GUARDIA['eventos']}")
    fuente = "".join(inspect.getsource(f) for f in (seleccionar, parsear_regla, _cumple, dominios, modelo_de, restringir,
                                                     casa, normalizar, validar_cfg))
    prohibido = re.findall(r"\b(subprocess|urllib|socket|http|requests|anthropic|openai|open\(|os\.)", fuente)
    print(f"grep del fuente de seleccionar y sus funciones: llamadas a red, procesos, ficheros o modelos: {len(prohibido)}")
    if prohibido:
        falla("S", f"el nucleo tiene {prohibido}")
    todo = open(os.path.abspath(__file__), encoding="utf-8").read()
    imp = re.findall(r"^\s*(?:import|from)\s+(urllib|http|socket|requests|ssl|anthropic|openai)\b", todo, re.M)
    print(f"imports de red o de modelos en todo el guion: {len(imp)}")
    if imp:
        falla("S", f"el guion importa {imp}")
    # cfg: solo restringe el modelo y solo anade senales
    r0 = filas[0]
    s_op = [x for x in filas if base[f"{x['repo']}/{x['id']}/diseno"]["modelo"] == "opus"]
    if s_op:
        x = s_op[0]
        sr = seleccionar("diseno", x["t"], x["efectivo"], x["cx"], x["cambios"], ids, cat,
                         {"modelos_permitidos": ["haiku", "sonnet"]}, tablas)
        print(f"modelos_permitidos [haiku, sonnet] sobre {x['id']} (opus): {sr['modelo']} con {sr['avisos']}")
        if sr["modelo"] != "sonnet" or "S15" not in sr["reglas"]:
            falla("S", "modelos_permitidos no restringe")
    for mal in ({"dominios_extra": {"datos": {"palabras": ["tabla"]}}}, {"modelos_permitidos": ["gpt"]}, {"roles": 1}):
        try:
            seleccionar("diseno", r0["t"], r0["efectivo"], r0["cx"], r0["cambios"], ids, cat, mal, tablas)
            falla("S", f"config aceptada: {mal}")
            print(f"config {mal}: ACEPTADA")
        except ErrorConfig as e:
            print(f"config {json.dumps(mal)}: rechazada ({e})")
    extra = {"dominios_extra": {"puertas-seguridad": {"rutas": ["src/auth/**"]}}}
    t_auth = {"objetivo": "Pantalla de acceso", "criterios": ["x"], "etiquetas": [], "tipo": "feature"}
    d = dominios(t_auth, [("M", ["src/auth/login.rs"])], tablas["dominios"], extra)
    print(f"dominios_extra anade la ruta src/auth/** a puertas-seguridad: {d}")
    if not d or d[0][0] != "puertas-seguridad":
        falla("S", "dominios_extra no anade la senal")
    # el modelo cambia con carril y con complejidad en filas reales
    mods = {(x["efectivo"], x["cx"]): tuple(base[f"{x['repo']}/{x['id']}/{k}"]["modelo"] for k in ROL_COL) for x in filas}
    por_carril = [(a1, a2) for a1 in mods for a2 in mods if a1[1] == a2[1] and a1[0] < a2[0] and mods[a1] != mods[a2]]
    por_cx = [(a1, a2) for a1 in mods for a2 in mods if a1[0] == a2[0] and NIVELES.index(a1[1]) < NIVELES.index(a2[1])
              and mods[a1] != mods[a2]]
    distintos = len(set(mods.values()))
    print(f"combinaciones carril/complejidad en las 10: {sorted(mods)} ; modelos distintos: {distintos}")
    print(f"  mismo nivel, distinto carril, distinto modelo: {por_carril[:3]}")
    print(f"  mismo carril, distinto nivel, distinto modelo: {por_cx[:3]}")
    if not por_carril or not por_cx:
        falla("S", "el modelo no cambia con carril y con complejidad en filas reales")
    usadas = sorted({x for s in base.values() for x in s["reglas"]})
    print(f"reglas emitidas en las 10 tareas reales: {usadas}")
    fuera = [x for x in usadas if x[0] == "S" and x not in tablas["seleccion"] or x[0] == "M" and x not in
             {f["id"] for f in tablas["modelo"]}]
    if fuera:
        falla("S", f"reglas emitidas que no estan en las tablas del ADR: {fuera}")
    print(f"filas de tabla-seleccion que no salen en las 10 reales: {sorted(set(tablas['seleccion']) - set(usadas))}")
    print("controles: un caso por fila de tabla-seleccion, interpretada desde el ADR (y cuatro de S13)")
    e, vistas, lin = controles_reglas(tablas, ids, cat)
    for l in lin:
        print(l)
    for x in e:
        falla("S", x)
    print(f"filas de tabla-seleccion ejercitadas por los controles: {len(set(tablas['seleccion']) & vistas)} de "
          f"{len(tablas['seleccion'])}")
    print("evasion por vocabulario (IMP-3): el mismo trabajo con otro texto; que contiene la defensa y que no")
    e, _v, lin = caso_vocabulario(tablas, ids, cat)
    for l in lin:
        print(l)
    for x in e:
        falla("S", x)
    e, (n_ok, n) = cobertura_atomos(tablas, ids, cat, filas)
    print(f"cobertura por atomo (MEN-9): {n_ok} de {n} variaciones de condicion (quitar cada atomo; anadir «no carril=X» "
          f"en implementacion, revision y todos) cambian algun resultado o rompen la tabla")
    for x in e:
        falla("S", x)
    print("catalogo del arbol del PR frente al de la base (MEN-4)")
    e, _v, lin = caso_catalogo_pr(tablas, ids, cat)
    for l in lin:
        print(l)
    for x in e:
        falla("S", x)
    return {"hash": a, "quitada": quitar, "afectadas": len(afect), "distintos": distintos}


# =========================================================================== G

SECCIONES = ["## Contexto", "## Alternativas", "## Pruebas de concepto", "## Cómo se ve en la tarea y en GitHub",
             "## Decisión", "## Divergencias con el plan", "## Requisitos y evidencia", "## Consecuencias",
             "## Fuera de alcance", "## No verificado"]
FRASES_OBLIGATORIAS = ["`seleccionar`", "`dominios`", "`tc2 doctor --recursos`", "`taskcodetwo`", "roles.<rol>",
                       "Coincide las 10", "configuración de la base"]
DIVERGENCIAS = ["F1-9", "F1-11", "F1-10", "F1-14", "F5-4", "desempate", "puerta-seguridad"]
NO_VERIFICADO = ["vocabulario", "modelo", "Linux"]


def errores_prosa(adr, ct):
    err = []
    prosa = re.sub(r"<!-- (\S+):inicio -->.*?<!-- \1:fin -->", " ", adr, flags=re.S)
    prosa = re.sub(r"^[ \t]*```[^\n]*\n.*?^[ \t]*```[ \t]*$", " ", prosa, flags=re.S | re.M)
    prosa = re.sub(r"^[ \t]*\|.*\|[ \t]*$", " ", prosa, flags=re.M)
    prosa = re.sub(r"\s+", " ", prosa.replace("**", ""))
    cubierto = []
    for frase in ct["frases"]:
        pos = [m.start() for m in re.finditer(re.escape(frase), prosa)]
        if not pos:
            err.append(f"prosa: falta la frase «{frase}»")
        cubierto += [(i, i + len(frase)) for i in pos]
    for m in re.finditer(r"\b\d+ (?:IMPORTANTES?|de puerta-seguridad)\b|\brecall \d+/\d+|\bprecisión \d+/\d+", prosa):
        if not any(a <= m.start() and m.end() <= b for a, b in cubierto):
            err.append(f"prosa: «{m.group(0)}» fuera de una frase que diga a qué corresponde "
                       f"(…{prosa[max(0, m.start() - 60):m.end()]})")
    return err


def errores_adr(adr, ruta, repo, ct, ids, cat, filas_m, retro, con_indice=True):
    tablas, err = leer_tablas(adr)
    try:
        e_ctl, _v, _l = controles_reglas(tablas, ids, cat)
        err += [f"control de reglas: {x}" for x in e_ctl]
        sels = {r["id"] + r["repo"]: {rol: seleccionar(rol, r["t"], r["efectivo"], r["cx"], r["cambios"], ids, cat, {}, tablas)
                                     for rol in ROL_COL} for r in filas_m}
    except (ErrorTabla, ErrorConfig, KeyError, IndexError) as e:
        err.append(f"las tablas del ADR no se pueden aplicar: {e}")
        sels = {r["id"] + r["repo"]: r["sel"] for r in filas_m}
    inst = set(ids)
    for i, f in tablas["seleccion"].items():
        for x in f["ids"]:
            if x not in inst:
                err.append(f"tabla-seleccion {i}: {x} no está en el inventario")
    b = entre_marcadores(adr, "tabla-muestra")
    if b is None:
        err.append("el ADR no tiene el bloque tabla-muestra")
    else:
        leidas = filas_tabla(b)
        esperadas = {fila_muestra(r)[0]: fila_muestra(r, sels[r["id"] + r["repo"]]) for r in filas_m}
        if len(leidas) != 10:
            err.append(f"la muestra tiene {len(leidas)} filas y son 10")
        vistas = set()
        for f in leidas:
            if len(f) != 12:
                err.append(f"tabla-muestra: fila con {len(f)} columnas: {f[:1]}")
                continue
            vistas.add(f[0])
            if f[0] not in esperadas:
                err.append(f"tabla-muestra: {f[0]} no es de la muestra calculada")
            elif f[:11] != esperadas[f[0]]:
                dif = [i for i in range(11) if f[i] != esperadas[f[0]][i]]
                err.append(f"tabla-muestra {f[0]}: fila distinta de lo calculado (columnas {dif})")
            if f[11] != "cambiada tras la marca de Carlos" and not re.fullmatch(
                    r"(coincide|no coincide|en parte)[^|]*\(Carlos, \d{4}-\d{2}-\d{2}\)", f[11]):
                err.append(f"tabla-muestra {f[0]}: comprobado a mano sin persona y fecha: «{f[11]}»")
        for k in sorted(set(esperadas) - vistas):
            err.append(f"tabla-muestra: falta {k}")
    b = entre_marcadores(adr, "tabla-retro")
    if b is None:
        err.append("el ADR no tiene el bloque tabla-retro")
    else:
        leidas = {l.strip() for l in b.strip().splitlines()[2:]}
        for t in FASE1:
            if fila_retro(t, retro[t]) not in leidas:
                err.append(f"tabla-retro: la fila de {t} no es la calculada")
        if len(leidas) != len(FASE1):
            err.append(f"tabla-retro: {len(leidas)} filas y son {len(FASE1)}")
    faltan = [s for s in SECCIONES if not re.search(rf"^{re.escape(s)}\s*$", adr, re.M)]
    if faltan:
        err.append(f"faltan secciones {faltan}")
    if not re.search(r"^- Estado: \*\*aceptada\*\*", adr, re.M):
        err.append("el estado no es aceptada")
    plano = re.sub(r"\s+", " ", adr)
    for frase in FRASES_OBLIGATORIAS:
        if frase not in plano:
            err.append(f"falta la regla: {frase}")
    div = re.search(r"^## Divergencias con el plan\s*$(.*?)(?=^## |\Z)", adr, re.S | re.M)
    for d in DIVERGENCIAS:
        if not div or d not in div.group(1):
            err.append(f"«Divergencias con el plan» no trata {d}")
    nv = re.search(r"^## No verificado\s*$(.*?)(?=^## |\Z)", adr, re.S | re.M)
    for d in NO_VERIFICADO:
        if not nv or d not in nv.group(1):
            err.append(f"«No verificado» no nombra «{d}»")
    req = open(os.path.join(repo, "docs", "plan", "REQUISITOS-FUNCIONALES.md"), encoding="utf-8").read()
    plan = open(os.path.join(repo, "docs", "plan", "PLAN-TASKCODE-2.md"), encoding="utf-8").read()
    adrs = os.listdir(os.path.join(repo, "docs", "adr"))
    malos = [i for i in sorted(set(re.findall(r"\b(R[RFN]-\d+)\b", adr))) if not re.search(rf"\|\s*{i}\s*\|", req)]
    malos += [i for i in sorted(set(re.findall(r"\b(F\d+-\d+)\b", adr)))
              if not re.search(rf"\|\s*{i}\s*\|", plan) and not any(a.startswith(i + "-") for a in adrs)]
    malos += [i for i in sorted(set(re.findall(r"\b(P\d+)\b", adr))) if not re.search(rf"^### {i} ", plan, re.M)]
    if malos:
        err.append(f"IDs que no existen: {malos}")
    enlaces = re.findall(r"\]\(([^)]+)\)", adr)
    prohibidos = [e for e in enlaces if "tareas/" in e or e.startswith("../../")]
    if prohibidos:
        err.append(f"enlaces prohibidos {prohibidos}")
    for e in enlaces:
        if not e.startswith("http") and not os.path.exists(os.path.join(os.path.dirname(ruta), e.split("#")[0])):
            err.append(f"enlace que no resuelve: {e}")
    if con_indice:
        idx = open(os.path.join(repo, "docs", "adr", "README.md"), encoding="utf-8").read()
        if FILA_INDICE not in idx:
            err.append("falta la fila en docs/adr/README.md")
    err += errores_prosa(adr, ct)
    return err


def _fila(adr, ide):
    m = re.search(rf"^[ \t]*\| {re.escape(ide)} \|.*\|$", adr, re.M)
    return m.group(0) if m else "(sin fila)"


def mutaciones(adr, ct, muestra_ids):
    m04, s09, d01, m1 = _fila(adr, "M04"), _fila(adr, "S09"), _fila(adr, "D01"), _fila(adr, muestra_ids[0])
    r002 = _fila(adr, "TASK-006")
    s03, s10, s13 = _fila(adr, "S03"), _fila(adr, "S10"), _fila(adr, "S13")
    return [
        ("fila-modelo-duplicada", adr.replace(m04, m04 + "\n" + m04), "id repetido en tabla-modelo: M04"),
        ("modelo-fuera-de-escala", adr.replace(m04, m04.replace("`sonnet`", "`gpt-x`", 1)), "modelo fuera de la escala: gpt-x"),
        ("dos-filas-para-un-caso", adr.replace(m04, m04 + "\n" + m04.replace("| M04 |", "| M98 |", 1)),
         "tabla-modelo: 2 filas para funcionalidad/"),
        ("regla-no-instalada", adr.replace(s09, s09.replace("`taskcode-plugin:code-quality-reviewer` |",
                                                            "`taskcode-plugin:code-quality-reviewer`, "
                                                            "`taskcode-plugin:no-existe` |", 1)),
         "S09: taskcode-plugin:no-existe no está en el inventario"),
        ("dominio-desconocido", adr.replace(d01, d01.replace("| `puertas-seguridad` |", "| `datos` |", 1)),
         "dominio fuera de la lista cerrada: datos"),
        ("dominio-id-repetido", adr.replace(d01, d01 + "\n" + d01), "id repetido en tabla-dominios: D01"),
        ("seleccion-sin-id", adr.replace(s09, s09.replace("| S09 |", "|  |", 1)), "tabla-seleccion: fila sin id"),
        ("muestra-cambiada", adr.replace(m1, m1.replace(" → ", " → riesgo-", 1) if "riesgo-alto" not in m1.split("|")[3]
                                         else m1.replace("→ riesgo-alto", "→ fix", 1)), "fila distinta de lo calculado"),
        ("muestra-con-9", adr.replace(m1 + "\n", ""), "la muestra tiene 9 filas"),
        ("a-mano-sin-persona", adr.replace(m1, m1.rstrip().rstrip("|").rstrip()[:-len(MANO)] + "sí |"),
         "comprobado a mano sin persona y fecha"),
        ("retro-cambiada", adr.replace(r002, re.sub(r"\| (\d+) \| (\d+) \| (\d+) \|", lambda x: f"| {int(x.group(1)) + 1} | "
                                                     f"{x.group(2)} | {x.group(3)} |", r002, count=1)),
         "tabla-retro: la fila de TASK-006 no es la calculada"),
        ("prosa-recall", adr.replace(ct["recall"], ct["recall"].replace("recall ", "recall 9", 1), 1),
         "prosa: falta la frase"),
        ("prosa-importantes", re.sub(r"\bde (\d+) IMPORTANTES", lambda x: f"de {int(x.group(1)) + 1} IMPORTANTES", adr, count=1),
         "IMPORTANTES» fuera de una frase"),
        ("id-inexistente", adr.replace("## Contexto", "## Contexto\n\nVer F7-99.", 1), "IDs que no existen: ['F7-99']"),
        ("enlace-roto", adr.replace("## Contexto", "## Contexto\n\n[x](F1-99-no-existe.md)", 1),
         "enlace que no resuelve: F1-99-no-existe.md"),
        ("seccion-borrada", adr.replace("## Cómo se ve en la tarea y en GitHub", "## Cómo se ve"), "faltan secciones"),
        ("estado-propuesta", adr.replace("- Estado: **aceptada**", "- Estado: **propuesta**"), "el estado no es aceptada"),
        ("divergencia-sin-f1-9", re.sub(r"(^## Divergencias con el plan\s*$)(.*?)(?=^## )",
                                        lambda x: x.group(1) + x.group(2).replace("F1-9", "F1-x"), adr, flags=re.S | re.M),
         "«Divergencias con el plan» no trata F1-9"),
        ("no-verificado-sin-vocabulario", re.sub(r"(^## No verificado\s*$)(.*?)(?=^## |\Z)",
                                                 lambda x: x.group(1) + x.group(2).replace("vocabulario", "texto"), adr,
                                                 flags=re.S | re.M), "«No verificado» no nombra «vocabulario»"),
        ("s09-no-en-fix", adr.replace(s09, s09.replace("| `rutas-no-casan-o-pasan-umbral` |",
                                                       "| `rutas-no-casan-o-pasan-umbral` y no `carril=fix` |", 1)),
         "control de reglas: caso S09 y S12 en fix"),
        ("s13-siempre", adr.replace(s13, re.sub(r"\| `ronda>=2`[^|]*\|", "| `siempre` |", s13, count=1)),
         "control de reglas: caso S13 ronda 2 en riesgo-alto no baja"),
        ("s03-suelo-3", adr.replace(s03, s03.replace("`suelo=2`", "`suelo=3`", 1)),
         "control de reglas: caso S03 suelo por puertas"),
        ("s10-da-la-generica", adr.replace(s10, s10.replace("`revisora-del-dominio`", "`generica-del-plugin`", 1)),
         "control de reglas: caso S10 revisora declarada del dominio"),
        ("condicion-desconocida", adr.replace(s10, s10.replace("`dominio-con-revisora`", "`nunca`", 1)),
         "condición desconocida: nunca"),
        ("muestra-con-suelo-3", adr.replace(s03, s03.replace("`suelo=2`", "`suelo=3`", 1)),
         "tabla-muestra OpenDockTrading/TASK-096: fila distinta de lo calculado"),
        ("sin-config-de-la-base", adr.replace("configuración de la base", "configuración del PR"),
         "falta la regla: configuración de la base"),
    ]


def seccion_g(repo, ct, ids, cat, filas_m, retro):
    sep("G: el ADR (tablas, muestra, retrospectiva, IDs, enlaces, prosa) y sus mutaciones de control")
    ruta = os.path.join(repo, ADR_REL)
    if not os.path.exists(ruta):
        falla("G", "no existe el ADR")
        return
    adr = open(ruta, encoding="utf-8").read()
    err = errores_adr(adr, ruta, repo, ct, ids, cat, filas_m, retro)
    for e in err:
        falla("G", e)
    print(f"errores del ADR: {len(err)}")
    tablas, _ = leer_tablas(adr)
    print(f"filas: tabla-dominios={len(tablas['dominios'])} tabla-modelo={len(tablas['modelo'])} "
          f"tabla-seleccion={len(tablas['seleccion'])} ; ids del inventario citados en tabla-seleccion: "
          f"{len({x for f in tablas['seleccion'].values() for x in f['ids']})}")
    mano = sum(1 for f in filas_tabla(entre_marcadores(adr, "tabla-muestra") or "") if len(f) == 12 and f[11] == MANO)
    print(f"filas de la muestra con «{MANO}»: {mano} de 10")
    for nombre, mut, esperado in mutaciones(adr, ct, [fila_muestra(r)[0] for r in filas_m]):
        if mut == adr:
            falla("G", f"la mutacion {nombre} no cambia nada")
            continue
        errs = errores_adr(mut, ruta, repo, ct, ids, cat, filas_m, retro)
        ok = any(esperado in e for e in errs)
        print(f"  mutacion {nombre}: {'falla como se espera' if ok else 'NO FALLA'} "
              f"({str(next((e for e in errs if esperado in e), errs[:1]))[:100]})")
        if not ok:
            falla("G", f"la mutacion {nombre} no la detecta el guion")


def frases(cr, cd, cs):
    rec = f"recall {cd['tp']}/{cd['tp'] + cd['fn']}"
    return {"recall": rec, "frases": [
        f"los {cr['imp']} IMPORTANTES de la Fase 1",
        f"{cr['ps']} de {cr['imp']} IMPORTANTES son de puerta-seguridad",
        f"{cr['ps1']} de ellos en planes de un solo rol",
        f"{cr['escapados']} IMPORTANTES de puerta-seguridad escaparon a un plan sin rol de riesgos",
        f"{cr['nuevos']} IMPORTANTES aparecieron en una ronda de verificación",
        f"{cr['nuevos_ps']} de puerta-seguridad",
        f"{rec} y precisión {cd['tp']}/{cd['tp'] + cd['fp']}",
        f"el suelo subiría de 1 a 2 roles en {cd['suelo']} de 15 tareas",
        f"{cr['con_pal']} de las 15 tareas de la Fase 1 contiene alguna palabra de riesgo de la 0.16",
        "ninguna palabra de puertas salvo `" + "`, `".join(cr["comunes"]) + "`",
        f"{cr['impl']} de las 15 tareas de la Fase 1 tienen una fila de implementación hecha por un subagente",
    ]}


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
    sys.addaudithook(_auditoria)
    args = sys.argv[1:]
    if len(args) != 3:
        raise SystemExit(__doc__)
    trabajo, repo, personal = (os.path.abspath(a) for a in args)
    if not os.path.normcase(POC).startswith(os.path.normcase(os.path.join(repo, "tareas")) + os.sep):
        raise SystemExit(f"ERROR: el guion ({POC}) no esta dentro de tareas/ del repo {repo}")
    comprobar_trabajo(trabajo)
    borrar(trabajo)
    os.makedirs(trabajo)
    open(os.path.join(trabajo, MARCA), "w", encoding="utf-8").write("carpeta de trabajo de poc-f1-15.py\n")
    print(f"PoC F1-15 ; " + " ; ".join(f"{k}@{v[:9]}" for k, v in FIJO.items()) + " ; plugin 0.16.1 de la cache")
    try:
        h = leer_heuristica(os.path.join(PLUGIN, "scripts", "heuristica-complejidad.yml"))
        adr_ruta = os.path.join(repo, ADR_REL)
        adr = open(adr_ruta, encoding="utf-8").read() if os.path.exists(adr_ruta) else ""
        tablas, terr = leer_tablas(adr)
        retro, cr = seccion_r(repo, h, tablas)
        reloj("R terminada")
        if terr:
            for e in terr:
                falla("G", e)
            raise SystemExit("las tablas del ADR no se pueden leer: se para antes de D")
        cd = seccion_d(repo, retro, tablas)
        clones = {}
        for p in PILOTOS:
            dst = os.path.join(trabajo, "clones", p)
            r = subprocess.run(["git", "clone", "--no-hardlinks", "--no-checkout", "-q", os.path.join(personal, p), dst],
                               capture_output=True, check=False)
            if r.returncode != 0:
                raise SystemExit(f"ERROR: clone de {p}: {r.stderr.decode('utf-8', 'replace')}")
            git(dst, "cat-file", "-e", FIJO[p] + "^{commit}")
            clones[p] = dst
        print()
        print("copias: " + " ; ".join(f"{p} git clone --no-hardlinks --no-checkout, commit fijo {FIJO[p][:12]}" for p in PILOTOS))
        reloj("copias hechas")
        _items, ids, cat = seccion_i(repo, trabajo, clones)
        filas = seccion_t(clones, h, ids, cat, tablas)
        reloj("T terminada")
        cs = seccion_s(filas, ids, cat, tablas)
        ct = frases(cr, cd, cs)
        seccion_g(repo, ct, ids, cat, filas, retro)
        print()
        print("tabla-retro calculada:")
        for t in FASE1:
            print(fila_retro(t, retro[t]))
    finally:
        borrar(trabajo)
    print()
    print(f"carpeta de trabajo borrada al terminar: {'no' if os.path.lexists(trabajo) else 'si'}")
    if os.path.lexists(trabajo):
        falla("main", "la carpeta de trabajo sigue existiendo")
    reloj("fin")
    resultado()


if __name__ == "__main__":
    main()

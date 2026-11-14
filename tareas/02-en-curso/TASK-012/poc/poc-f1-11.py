"""PoC de F1-11 (TASK-012): extensiones declaradas. SOLO LECTURA de los pilotos.

Se ejecuta con `python -I -B` (sin el directorio del guion ni PYTHON* en
sys.path, sin __pycache__). ToolsaWeb y OpenDockTrading solo se leen con
`git show`, `git ls-tree`, `git grep` y `git archive` de commits fijos; lo que
se ejecuta de ellos (PoC D y E) es una COPIA en la carpeta de trabajo. No
habla con GitHub. El plugin 0.16.1 solo se lee.

(A) Cobertura: la tabla `inventario` del ADR (entre marcadores). Cada cita
    `REPO ruta:linea` existe en el commit fijo y contiene el texto citado;
    cada practica tiene destino y, si es una clave, la clave esta en la tabla
    `esquema` del ADR, que coincide con la que valida este guion.
(B) Esquema y presupuesto: valida las configuraciones de ToolsaWeb y
    OpenDockTrading traducidas al esquema nuevo y rechaza las hostiles; mide
    claves frente a la 0.16 y frente al tope de 12; confianza (F1-8 dec. 5).
(C) Derivados en la rama de estado: repo temporal con la rama huerfana
    `taskcode/tareas` montada en `tareas/`; transiciones con derivados que
    ejecuta un comando explicito; el arbol de `develop` no cambia (RR-11).
(D) Lectura del estado: el glob `0?-*/TASK-*/tarea.md` de los lectores
    propios (copiados) sobre la carpeta fija da 0 tareas; un `tareas --json`
    simulado da el mismo conjunto que hoy.
(E) `clasificar-verificacion.py plan --completa` de ToolsaWeb (copia) con
    varios valores de `RAMA_BASE`: los grupos de la completa no cambian.

Al final compara la tabla `ventaja` del ADR con lo medido.

Uso: python -I -B poc-f1-11.py <trabajo> <ruta-del-ADR-F1-11> <carpeta-Personal> <plugin-0.16.1>
Sale con 0 solo si A..E y la tabla de ventaja salen como se espera.
"""
import ast
import hashlib
import importlib.util
import io
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tarfile

import yaml

# Commits fijos (leidos el 2026-11-14) para que la medida se reproduzca
# aunque los repos avancen.
FIJO = {
    "TW": ("ToolsaWeb", "cb46c0f1bdd3619322e08a44832c4cf04ccba111"),
    "ODT": ("OpenDockTrading", "f8a9b5f271a7862ff91dd482e9c682ed15078912"),
}
FALLOS = []
MEDIDO = {}          # cifra -> valor, para la tabla `ventaja` del ADR
FECHA_GIT = "2026-11-14T12:00:00+00:00"


def falla(seccion, texto):
    FALLOS.append(f"{seccion}: {texto}")
    print(f"FALLO en {seccion}: {texto}")


def sep(t):
    print()
    print(f"=== {t} ===")


def run(argv, cwd=None, env=None, binario=False, check=True, timeout=600):
    r = subprocess.run(argv, cwd=cwd, env=env, capture_output=True, check=False,
                       stdin=subprocess.DEVNULL, timeout=timeout)
    if check and r.returncode != 0:
        raise SystemExit(f"ERROR: {' '.join(argv[:4])}...: {r.stderr.decode('utf-8', 'replace').strip()[:400]}")
    return r if not check else (r.stdout if binario else r.stdout.decode("utf-8", "replace"))


def env_git():
    e = dict(os.environ)
    e.update({"GIT_AUTHOR_NAME": "poc", "GIT_AUTHOR_EMAIL": "poc@ejemplo.invalid",
              "GIT_COMMITTER_NAME": "poc", "GIT_COMMITTER_EMAIL": "poc@ejemplo.invalid",
              "GIT_AUTHOR_DATE": FECHA_GIT, "GIT_COMMITTER_DATE": FECHA_GIT,
              "GIT_CONFIG_NOSYSTEM": "1"})
    return e


def git(repo, *args, binario=False, check=True):
    return run(["git", "-C", repo, "-c", "core.quotepath=false", "-c", "core.autocrlf=false",
                "-c", "core.longpaths=true", *args],
               env=env_git(), binario=binario, check=check)


class Pilotos:
    def __init__(self, personal):
        self.personal = personal

    def repo(self, clave):
        return os.path.join(self.personal, FIJO[clave][0])

    def show(self, clave, ruta):
        return git(self.repo(clave), "show", f"{FIJO[clave][1]}:{ruta}", binario=True).decode("utf-8")

    def ls(self, clave, *rutas):
        return [x for x in git(self.repo(clave), "ls-tree", "-r", "--name-only", FIJO[clave][1], "--", *rutas)
                .splitlines() if x]


def norm(t):
    return re.sub(r"\s+", " ", t).strip()


# ---------------------------------------------------------------------------
# Esquema de las tres claves (lo que el ADR fija para F2-7)
# ---------------------------------------------------------------------------

PERFILES = ("rapida", "impacto", "completa", "pesada")
SHELLS = ("cmd", "bash", "pwsh", "sh")
TIPOS_DERIVADO = ("genera", "comprueba")
FASES = ("new", "import", "plan", "approve", "start", "review", "finish", "descartar", "pausa")
CLAVES_F110 = ("politica_carriles", "rutas_carril")      # F1-10: se aceptan, no se validan aqui
CLAVES_F111 = ("verificacion", "derivados", "confianza_incluye")
ESQUEMA = {
    "verificacion", "verificacion.comando", "verificacion.shell",
    "verificacion.perfiles.<p>.timeout", "verificacion.perfiles.<p>.runner",
    "verificacion.huella_arbol", "verificacion.huella_local",
    "verificacion.programada.comando", "verificacion.programada.cron", "verificacion.programada.runner",
    "derivados", "derivados[].comando", "derivados[].tipo", "derivados[].escribe",
    "derivados[].fases", "derivados[].timeout",
    "confianza_incluye",
}
RESERVADAS_TAREAS = ("tareas/ROADMAP.md",)          # la escribe taskctl (F1-1 dec. 3)


class ErrorConfig(Exception):
    pass


def es_orden(x):
    return isinstance(x, str) and x.strip() != "" and not re.search(r"[\x00-\x1f\x7f]", x)


def mal_orden(p, x):
    if not isinstance(x, str) or not x.strip():
        return f"{p}: orden vacia o no es texto"
    c = re.search(r"[\x00-\x1f\x7f]", x)
    return (f"{p}: caracter de control {c.group()!r} en la orden (¿\\v, \\t o \\n dentro de comillas "
            f"dobles?): escribe las rutas con comillas simples o con /")


def glob_ok(g):
    return (isinstance(g, str) and g and not g.startswith(("!", "/")) and ".." not in g.split("/")
            and "\\" not in g and g.strip("*/") != "")


def mal_runner(p, r):
    if r in ("github-windows", "github-linux", "github-macos"):
        return None
    if isinstance(r, list) and r and all(isinstance(e, str) and e for e in r) and r[0] == "self-hosted":
        return None
    return f"{p}: runner {r!r}: github-windows|github-linux|github-macos o [self-hosted, ...]"


def validar(cfg):
    """Errores del fragmento de configuracion (vacio = valida). Falla cerrado ante lo desconocido."""
    err = []
    if not isinstance(cfg, dict):
        return ["la configuracion no es un mapa"]
    for k in cfg:
        if k not in CLAVES_F111 + CLAVES_F110:
            err.append(f"clave desconocida {k}")
    v = cfg.get("verificacion")
    if v is not None:
        if not isinstance(v, dict):
            err.append("verificacion no es un mapa")
            v = {}
        for k in v:
            if k not in ("comando", "shell", "perfiles", "huella_arbol", "huella_local", "programada"):
                err.append(f"verificacion.{k}: clave desconocida")
        if not es_orden(v.get("comando")):
            err.append(mal_orden("verificacion.comando", v.get("comando")))
        if v.get("shell") not in SHELLS:
            err.append(f"verificacion.shell: {v.get('shell')!r} no es uno de {list(SHELLS)}")
        pf = v.get("perfiles")
        if not isinstance(pf, dict) or "completa" not in pf:
            err.append("verificacion.perfiles: mapa con al menos completa")
            pf = pf if isinstance(pf, dict) else {}
        for p, d in pf.items():
            q = f"verificacion.perfiles.{p}"
            if p not in PERFILES:
                err.append(f"{q}: perfil desconocido (los de F1-7: {list(PERFILES)})")
                continue
            if not isinstance(d, dict):
                err.append(f"{q}: no es un mapa")
                continue
            for k in d:
                if k not in ("timeout", "runner"):
                    err.append(f"{q}.{k}: clave desconocida")
            t = d.get("timeout")
            if type(t) is not int or not 1 <= t <= 6 * 3600:
                err.append(f"{q}.timeout: entero de 1 a 21600 segundos")
            if p in ("completa", "pesada"):
                m = mal_runner(f"{q}.runner", d.get("runner"))
                if m:
                    err.append(m)
            elif "runner" in d:
                err.append(f"{q}.runner: {p} corre en local (F1-7 dec. 4); sin runner")
        ha = v.get("huella_arbol", [])
        if not isinstance(ha, list) or any(not glob_ok(g) for g in ha):
            err.append("verificacion.huella_arbol: lista de globs de ficheros versionados")
        if "huella_local" in v and not es_orden(v["huella_local"]):
            err.append(mal_orden("verificacion.huella_local", v["huella_local"]))
        pr = v.get("programada")
        if pr is not None:
            if not isinstance(pr, dict):
                err.append("verificacion.programada no es un mapa")
            else:
                for k in pr:
                    if k not in ("comando", "cron", "runner"):
                        err.append(f"verificacion.programada.{k}: clave desconocida")
                if not es_orden(pr.get("comando")):
                    err.append(mal_orden("verificacion.programada.comando", pr.get("comando")))
                if not isinstance(pr.get("cron"), str) or len(pr["cron"].split()) != 5:
                    err.append("verificacion.programada.cron: expresion cron de 5 campos")
                m = mal_runner("verificacion.programada.runner", pr.get("runner"))
                if m:
                    err.append(m)
    ds = cfg.get("derivados", [])
    if not isinstance(ds, list):
        err.append("derivados no es una lista")
        ds = []
    for i, d in enumerate(ds):
        q = f"derivados[{i}]"
        if not isinstance(d, dict):
            err.append(f"{q}: no es un mapa")
            continue
        for k in d:
            if k not in ("comando", "tipo", "escribe", "fases", "timeout"):
                err.append(f"{q}.{k}: clave desconocida")
        if not es_orden(d.get("comando")):
            err.append(mal_orden(f"{q}.comando", d.get("comando")))
        tipo = d.get("tipo")
        if tipo not in TIPOS_DERIVADO:
            err.append(f"{q}.tipo: {tipo!r} no es genera ni comprueba")
        es = d.get("escribe", [])
        if not isinstance(es, list):
            err.append(f"{q}.escribe: lista de rutas")
            es = []
        if tipo == "genera" and not es:
            err.append(f"{q}.escribe: un derivado genera declara lo que escribe")
        if tipo == "comprueba" and es:
            err.append(f"{q}.escribe: un derivado comprueba no escribe nada")
        for r in es:
            if not isinstance(r, str) or not r.startswith("tareas/") or ".." in r.split("/") or "\\" in r \
                    or "*" in r:
                err.append(f"{q}.escribe: {r!r} fuera de tareas/ (D2a: solo la rama de estado)")
            elif r in RESERVADAS_TAREAS or re.match(r"tareas/TASK-\d+(/|$)", r):
                err.append(f"{q}.escribe: {r!r} lo escribe taskctl (roadmap o carpeta de tarea)")
        fs = d.get("fases", list(FASES))
        if not isinstance(fs, list) or not fs or any(f not in FASES for f in fs):
            err.append(f"{q}.fases: subconjunto no vacio de {list(FASES)}")
        t = d.get("timeout", 60)
        if type(t) is not int or not 1 <= t <= 600:
            err.append(f"{q}.timeout: entero de 1 a 600 segundos")
    ci = cfg.get("confianza_incluye", [])
    if not isinstance(ci, list) or any(not glob_ok(g) for g in ci):
        err.append("confianza_incluye: globs sin negacion, sin rutas absolutas ni .., y no todo el arbol")
    return err


def cargar(texto):
    try:
        cfg = yaml.safe_load(texto)
    except yaml.YAMLError as e:
        raise ErrorConfig(f"YAML no valido: {str(e).splitlines()[0]}")
    err = validar(cfg)
    if err:
        raise ErrorConfig("; ".join(err))
    return cfg


# ---------------------------------------------------------------------------
# Confianza local unica (F1-8 dec. 5), version minima para la PoC
# ---------------------------------------------------------------------------

EXT_SCRIPT = ("bat", "cmd", "sh", "ps1", "py", "js", "ts")
RX_RUTA = re.compile(r"[\w./\\%~-]+\.(?:" + "|".join(EXT_SCRIPT) + r")\b")


def casa(glob, ruta):
    rx = ""
    i = 0
    while i < len(glob):
        if glob.startswith("**", i):
            rx += ".*"
            i += 2
        elif glob[i] == "*":
            rx += "[^/]*"
            i += 1
        else:
            rx += re.escape(glob[i])
            i += 1
    return re.fullmatch(rx, ruta) is not None


def huella_confianza(repo, orden, incluye):
    versionados = set(git(repo, "ls-files").splitlines())
    vistos, pend = [], [orden]
    while pend:
        texto = pend.pop()
        for tok in RX_RUTA.findall(texto):
            r = tok.replace("\\", "/").replace("%~dp0", "scripts/")
            r = r[2:] if r.startswith("./") else r
            if r in versionados and r not in vistos:
                vistos.append(r)
                pend.append(open(os.path.join(repo, r), encoding="utf-8", errors="replace").read())
    for g in incluye:
        vistos += [r for r in sorted(versionados) if casa(g, r) and r not in vistos]
    h = hashlib.sha256(orden.encode())
    for r in sorted(vistos):
        h.update(r.encode() + b"\0" + open(os.path.join(repo, r), "rb").read())
    return h.hexdigest(), sorted(vistos)


def confianza(repo, clave, orden, incluye, almacen):
    datos = json.load(open(almacen)) if os.path.exists(almacen) else {}
    h, _ = huella_confianza(repo, orden, incluye)
    k = f"{os.path.normcase(repo)}|{clave}"
    if k not in datos:
        return "nunca"
    return "confiado" if datos[k] == h else "cambiado"


def confiar(repo, clave, orden, incluye, almacen):
    datos = json.load(open(almacen)) if os.path.exists(almacen) else {}
    datos[f"{os.path.normcase(repo)}|{clave}"] = huella_confianza(repo, orden, incluye)[0]
    json.dump(datos, open(almacen, "w"), sort_keys=True)


# ---------------------------------------------------------------------------
# Tablas del ADR
# ---------------------------------------------------------------------------

def entre_marcadores(texto, nombre):
    m = re.search(rf"<!-- {nombre}:inicio -->\n(.*?)<!-- {nombre}:fin -->", texto, re.S)
    if not m:
        raise SystemExit(f"ERROR: faltan los marcadores {nombre}")
    filas = []
    for linea in m.group(1).splitlines():
        linea = linea.strip()
        if linea.startswith("|") and not re.match(r"^\|[-| ]+\|$", linea):
            filas.append([c.strip() for c in linea.strip().strip("|").split("|")])
    return filas[0], filas[1:]


RX_CITA = re.compile(r"`(TW|ODT|PL) ([^`:]+):(\d+)(?:-(\d+))?` «([^»]+)»")


# ---------------------------------------------------------------------------
# A: cobertura
# ---------------------------------------------------------------------------

def seccion_a(pil, plugin, adr):
    sep("A: cobertura de las practicas del inventario (citas comprobadas en los commits fijos)")
    cab, filas = entre_marcadores(adr, "inventario")
    col = {c: i for i, c in enumerate(cab)}
    for c in ("#", "Práctica", "Evidencia (repo fichero:línea «texto»)", "En la 0.16", "Destino",
              "Punto de extensión en la 2.0"):
        if c not in col:
            raise SystemExit(f"ERROR: falta la columna {c!r} en inventario")
    _c, esq = entre_marcadores(adr, "esquema")
    claves_adr = {re.findall(r"`([^`]+)`", f[0])[0] for f in esq if re.findall(r"`([^`]+)`", f[0])}
    print(f"claves en la tabla esquema del ADR: {len(claves_adr)} ; en el validador: {len(ESQUEMA)} ; "
          f"iguales: {claves_adr == ESQUEMA}")
    if claves_adr != ESQUEMA:
        falla("A", f"esquema del ADR distinto del validador: sobran {sorted(claves_adr - ESQUEMA)}, "
                   f"faltan {sorted(ESQUEMA - claves_adr)}")
    cache = {}

    def texto_de(repo, ruta):
        if (repo, ruta) not in cache:
            if repo == "PL":
                p = os.path.join(plugin, *ruta.split("/"))
                cache[(repo, ruta)] = open(p, encoding="utf-8").read().replace("\r\n", "\n") \
                    if os.path.isfile(p) else None
            else:
                try:
                    cache[(repo, ruta)] = pil.show(repo, ruta).replace("\r\n", "\n")
                except SystemExit:
                    cache[(repo, ruta)] = None
        return cache[(repo, ruta)]

    citas = ok = 0
    cuenta016 = {}
    cuenta20 = {}
    for f in filas:
        n, practica = f[col["#"]], f[col["Práctica"]]
        ev = f[col["Evidencia (repo fichero:línea «texto»)"]]
        refs = RX_CITA.findall(ev)
        if not refs:
            falla("A", f"practica {n} sin citas con el formato `REPO ruta:linea` «texto»")
        malas = []
        for repo, ruta, l1, l2, frag in refs:
            citas += 1
            t = texto_de(repo, ruta)
            if t is None:
                malas.append(f"{repo} {ruta}: no existe")
                continue
            lineas = t.split("\n")
            a, b = int(l1), int(l2 or l1)
            if not 1 <= a <= b <= len(lineas):
                malas.append(f"{repo} {ruta}:{l1}-{l2}: fuera del fichero ({len(lineas)} lineas)")
                continue
            if norm(frag) not in norm(" ".join(lineas[a - 1:b])):
                malas.append(f"{repo} {ruta}:{a}-{b}: no contiene «{frag}»")
                continue
            ok += 1
        for m in malas:
            falla("A", f"practica {n}: {m}")
        e016 = f[col["En la 0.16"]]
        dest = f[col["Destino"]]
        punto = f[col["Punto de extensión en la 2.0"]]
        cuenta016[e016] = cuenta016.get(e016, 0) + 1
        cuenta20[dest] = cuenta20.get(dest, 0) + 1
        if e016 not in ("clave", "plugin", "a mano"):
            falla("A", f"practica {n}: «En la 0.16» = {e016!r}")
        if dest == "clave":
            usadas = set(re.findall(r"`([^`]+)`", punto)) & ESQUEMA
            nombradas = {x for x in re.findall(r"`([^`]+)`", punto)
                         if re.match(r"^(verificacion|derivados|confianza_incluye)", x)}
            if not usadas or nombradas - ESQUEMA:
                falla("A", f"practica {n}: destino clave sin clave del esquema ({sorted(nombradas - ESQUEMA)})")
        elif dest not in ("nativo", "skill", "fuera"):
            falla("A", f"practica {n}: destino {dest!r} no admitido")
        print(f"  {n:>2} {practica[:58]:58} citas={len(refs)} 0.16={e016:7} 2.0={dest:7} "
              f"{'OK' if not malas and refs else 'FALLO'}")
    print(f"citas comprobadas: {ok}/{citas}")
    print(f"en la 0.16: por clave={cuenta016.get('clave', 0)} ; del plugin={cuenta016.get('plugin', 0)} ; "
          f"a mano={cuenta016.get('a mano', 0)} (de {len(filas)})")
    print(f"en la 2.0: clave={cuenta20.get('clave', 0)} ; nativo={cuenta20.get('nativo', 0)} ; "
          f"skill={cuenta20.get('skill', 0)} ; fuera de alcance={cuenta20.get('fuera', 0)} ; "
          f"sin destino=0")
    if len(filas) != 14:
        falla("A", f"el inventario tiene {len(filas)} practicas, el plan cuenta 14")
    if cuenta20.get("fuera", 0) != 1:
        falla("A", "solo la practica 14 (guardas del arnes) queda fuera de alcance")
    MEDIDO["practicas"] = str(len(filas))
    MEDIDO["practicas a mano en la 0.16"] = str(cuenta016.get("a mano", 0))
    MEDIDO["practicas con destino declarado en la 2.0"] = str(len(filas) - cuenta20.get("fuera", 0))
    MEDIDO["citas comprobadas"] = f"{ok}/{citas}"

    print("-- estados propios (practica 1): 05-archivadas")
    total = coh = 0
    for clave in ("TW", "ODT"):
        tareas = sorted({p.split("/")[2] for p in pil.ls(clave, "tareas/05-archivadas") if p.endswith("tarea.md")})
        bien = 0
        estados = {}
        for t in tareas:
            fm = pil.show(clave, f"tareas/05-archivadas/{t}/tarea.md")
            et = re.search(r"^etiquetas:\s*\[(.*)\]", fm, re.M)
            es = re.search(r"^estado:\s*(\S+)", fm, re.M)
            if et and "descartada" in [x.strip() for x in et.group(1).split(",")]:
                bien += 1
            estados.setdefault(es.group(1) if es else "?", []).append(t)
        total += len(tareas)
        coh += bien
        print(f"  {FIJO[clave][0]}: {len(tareas)} en 05-archivadas ; con etiqueta descartada: {bien} ; estado del "
              f"frontmatter: " + " ; ".join(f"{e}={len(v)} ({' '.join(v)})" for e, v in sorted(estados.items())))
        incoh = sum(len(v) for e, v in estados.items() if e != "descartada")
        MEDIDO[f"{FIJO[clave][0]}: archivadas con estado distinto de descartada"] = f"{incoh}/{len(tareas)}"
    print(f"  casos que cubre el estado nativo `descartada`: {coh}/{total}")
    MEDIDO["casos de 05-archivadas cubiertos por descartada"] = f"{coh}/{total}"
    if coh != total or total != 11:
        falla("A", f"05-archivadas: {coh}/{total}, el plan mide 11 de 11")
    print("-- revisoras propias (practica 13): skills con `rol: revisor` en los pilotos")
    for clave in ("TW", "ODT"):
        r = git(pil.repo(clave), "grep", "-l", "rol: revisor", FIJO[clave][1], "--", "*.md", check=False)
        n = len([x for x in r.stdout.decode("utf-8", "replace").splitlines() if x])
        print(f"  {FIJO[clave][0]}: {n}")
        if n:
            falla("A", f"{clave} declara revisoras propias: el inventario dice que no")
    print(f"RESULTADO A practicas={len(filas)} ; con destino={len(filas) - cuenta20.get('fuera', 0)} ; "
          f"citas={ok}/{citas}")


# ---------------------------------------------------------------------------
# B: esquema, presupuesto y ventaja en claves
# ---------------------------------------------------------------------------

CFG_TW = """\
# ToolsaWeb traducida al esquema nuevo (lo que hoy dicen sus 12 claves planas)
verificacion:
  comando: 'scripts\\verificar.bat'
  shell: 'cmd'
  perfiles:
    rapida:   {timeout: 180}
    impacto:  {timeout: 1800}
    completa: {timeout: 4800, runner: [self-hosted, windows, toolsaweb]}
    pesada:   {timeout: 4800, runner: [self-hosted, windows, toolsaweb]}
  huella_arbol: ['Cargo.lock', 'web/package-lock.json']
  huella_local: 'python -I scripts/huella-local.py'
derivados:
  - comando: 'python -I scripts/grafo-tareas.py --comprobar'
    tipo: 'comprueba'
    fases: ['plan', 'approve', 'finish']
confianza_incluye: ['scripts/cache-verificacion.py', 'scripts/memo_ficheros.py']
"""

CFG_ODT = """\
# OpenDockTrading traducida al esquema nuevo
verificacion:
  comando: 'scripts\\verificar.bat'
  shell: 'cmd'
  perfiles:
    completa: {timeout: 1200, runner: 'github-windows'}
  huella_arbol: ['Cargo.lock', 'web/package-lock.json']
  programada:
    comando: 'cargo audit --no-yanked --deny unsound && npm --prefix web audit'
    cron: '0 6 * * 1'
    runner: 'github-windows'
derivados:
  - comando: 'python -I scripts/comprobar-modulos.py --comprobar'
    tipo: 'comprueba'
    fases: ['plan', 'finish']
confianza_incluye: ['scripts/lector-tareas.py']
"""

HOSTILES = [
    ("clave desconocida (la 0.16 en plano)", "comando_verificacion_freno: 'scripts/verificar.bat --rapida'\n"),
    ("clave desconocida dentro de verificacion",
     "verificacion: {comando: 'a.bat', shell: 'cmd', perfiles: {completa: {timeout: 60, runner: 'github-windows'}},"
     " presupuesto_freno_s: 180}\n"),
    ("ruta de Windows con \\v entre comillas dobles (linea 9 de ToolsaWeb)",
     "verificacion:\n  comando: \"scripts\\verificar.bat\"\n  shell: 'cmd'\n"
     "  perfiles: {completa: {timeout: 60, runner: 'github-windows'}}\n"),
    ("derivado que escribe fuera de tareas/ (hitos.md de ToolsaWeb)",
     "derivados:\n  - {comando: 'python -I scripts/grafo-tareas.py --escribir', tipo: 'genera',"
     " escribe: ['docs/planificacion/hitos.md']}\n"),
    ("derivado que escribe el ROADMAP nativo", "derivados:\n  - {comando: 'g.py', tipo: 'genera', escribe: ['tareas/ROADMAP.md']}\n"),
    ("derivado que escribe una carpeta de tarea",
     "derivados:\n  - {comando: 'g.py', tipo: 'genera', escribe: ['tareas/TASK-001/tarea.md']}\n"),
    ("derivado comprueba que escribe", "derivados:\n  - {comando: 'c.py', tipo: 'comprueba', escribe: ['tareas/x.md']}\n"),
    ("derivado con fase inventada", "derivados:\n  - {comando: 'c.py', tipo: 'comprueba', fases: ['push']}\n"),
    ("estado propio declarado (D1a: sin clave)", "estados: ['archivada']\n"),
    ("perfil fuera de F1-7", "verificacion: {comando: 'a.bat', shell: 'cmd', perfiles: {completa: {timeout: 60,"
     " runner: 'github-windows'}, selectiva: {timeout: 60}}}\n"),
    ("rapida con runner (corre en local)", "verificacion: {comando: 'a.bat', shell: 'cmd', perfiles: {completa: "
     "{timeout: 60, runner: 'github-windows'}, rapida: {timeout: 60, runner: 'github-linux'}}}\n"),
    ("confianza_incluye que hashea todo el arbol", "confianza_incluye: ['**']\n"),
    ("timeout como texto", "verificacion: {comando: 'a.bat', shell: 'cmd', perfiles: {completa: {timeout: '60',"
     " runner: 'github-windows'}}}\n"),
]

# Claves de la 0.16 que sustituyen los bloques verificacion y derivados (catalogo de src/core/config.ts)
SUSTITUIDAS_016 = [
    "comando_verificacion", "timeout_verificacion", "comando_verificacion_selectiva",
    "timeout_verificacion_selectiva", "comando_verificacion_freno", "presupuesto_freno_s",
    "comando_verificacion_pesada", "verificacion_pesada_en",
    "comando_sincronizacion", "rutas_sincronizacion", "timeout_sincronizacion", "fases_sincronizacion",
]


def seccion_b(pil, plugin, trabajo):
    sep("B: esquema de las tres claves, hostiles, presupuesto y ventaja en claves")
    cfgs = {}
    for clave, texto in (("TW", CFG_TW), ("ODT", CFG_ODT)):
        try:
            cfgs[clave] = cargar(texto)
            print(f"  {FIJO[clave][0]} traducida -> valida ; claves de primer nivel: {sorted(cfgs[clave])}")
        except ErrorConfig as e:
            falla("B", f"{clave} traducida rechazada: {e}")
            cfgs[clave] = {}
    print("-- lo que la traduccion nombra existe en el commit fijo")
    falta_orden = []
    for clave, cfg in cfgs.items():
        arbol = set(pil.ls(clave))
        v = cfg.get("verificacion", {})
        for g in v.get("huella_arbol", []) + cfg.get("confianza_incluye", []):
            n = sum(1 for r in arbol if casa(g, r))
            print(f"  {clave} glob {g!r}: {n} fichero(s)")
            if n == 0:
                falla("B", f"{clave}: el glob {g!r} no casa con nada")
        ordenes = [("verificacion.comando", v.get("comando")), ("verificacion.huella_local", v.get("huella_local"))]
        ordenes += [(f"derivados[{i}].comando", d["comando"]) for i, d in enumerate(cfg.get("derivados", []))]
        for k, o in ordenes:
            if not o:
                continue
            rutas = [t.replace("\\", "/") for t in RX_RUTA.findall(o)]
            for r in rutas:
                existe = r in arbol
                print(f"  {clave} {k}: {r} {'existe' if existe else 'NO EXISTE en el commit fijo (a crear)'}")
                if not existe:
                    falta_orden.append(f"{clave} {r}")
    MEDIDO["ordenes de la traduccion que no existen hoy"] = str(len(falta_orden))
    print("-- contrato --perfil frente al verificar.bat de ToolsaWeb (lectura, no se ejecuta)")
    bat = pil.show("TW", "scripts/verificar.bat").replace("\r\n", "\n").split("\n")
    acepta = [re.search(r'"(--\w+)"', l).group(1) for l in bat[67:72] if re.search(r'"(--\w+)"', l)]
    rechaza = "exit /b 2" in bat[78]
    print(f"  verificar.bat:68-79 acepta {acepta} ; otro argumento -> exit /b 2: {rechaza} ; "
          f"«--perfil completa» lo acepta: {'--perfil' in acepta}")
    MEDIDO["argumentos de perfil que ToolsaWeb acepta hoy"] = ", ".join(acepta)

    print("-- hostiles (falla cerrado)")
    rech = 0
    for nombre, texto in HOSTILES:
        try:
            cargar(texto)
            print(f"  {nombre:66} -> ACEPTADA")
            falla("B", f"hostil aceptada: {nombre}")
        except ErrorConfig as e:
            rech += 1
            print(f"  {nombre:66} -> rechazada: {str(e)[:110]}")
    MEDIDO["configuraciones hostiles rechazadas"] = f"{rech}/{len(HOSTILES)}"
    tw_yaml = yaml.safe_load(pil.show("TW", ".taskcode/config.yml"))
    odt_yaml = yaml.safe_load(pil.show("ODT", ".taskcode/config.yml"))
    print(f"  lector YAML estandar sobre la config real de ToolsaWeb: comando_verificacion = "
          f"{tw_yaml['comando_verificacion']!r}")
    print(f"  y sobre la de OpenDockTrading (sin comillas): comando_verificacion = {odt_yaml['comando_verificacion']!r}")

    print("-- confianza (F1-8 dec. 5): una orden sin confianza no se ejecuta")
    repo = os.path.join(trabajo, "b", "repo")
    os.makedirs(os.path.join(repo, "scripts"))
    git(repo, "init", "-q", "-b", "develop")
    open(os.path.join(repo, "scripts", "grafo.py"), "w").write("print('grafo')\n")
    open(os.path.join(repo, "scripts", "ayuda.py"), "w").write("X = 1\n")
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", "base")
    alm = os.path.join(trabajo, "b", "almacen-confianza.json")
    orden, inc = "python -I scripts/grafo.py --escribir", ["scripts/ayuda.py"]
    e1 = confianza(repo, "derivados[0]", orden, inc, alm)
    confiar(repo, "derivados[0]", orden, inc, alm)
    e2 = confianza(repo, "derivados[0]", orden, inc, alm)
    open(os.path.join(repo, "scripts", "grafo.py"), "a").write("import os  # un pull cambia el script\n")
    e3 = confianza(repo, "derivados[0]", orden, inc, alm)
    git(repo, "checkout", "-q", "--", "scripts/grafo.py")
    e4 = confianza(repo, "derivados[0]", orden, inc, alm)
    open(os.path.join(repo, "scripts", "ayuda.py"), "a").write("Y = 2\n")
    e5 = confianza(repo, "derivados[0]", orden, inc, alm)
    git(repo, "checkout", "-q", "--", "scripts/ayuda.py")
    e6 = confianza(repo, "derivados[0]", orden + " --mas", inc, alm)
    _h, cierre = huella_confianza(repo, orden, inc)
    print(f"  sin entrada: {e1} ; tras confiar: {e2} ; script cambiado: {e3} ; vuelta al contenido: {e4} ; "
          f"fichero de confianza_incluye cambiado: {e5} ; orden cambiada: {e6} ; cierre={cierre}")
    if (e1, e2, e3, e4, e5, e6) != ("nunca", "confiado", "cambiado", "confiado", "cambiado", "cambiado"):
        falla("B", "estados de confianza inesperados")

    print("-- presupuesto de claves y ventaja frente a la 0.16")
    ts = open(os.path.join(plugin, "src", "core", "config.ts"), encoding="utf-8").read()
    cabecera = ts.split("*/", 1)[0]
    catalogo = re.findall(r"^\s*\*\s*\|\s*([a-z_]+)\s*\|", cabecera, re.M)
    catalogo = [c for c in catalogo if c != "clave"]
    print(f"  claves del catalogo de la 0.16 (src/core/config.ts, tabla de cabecera): {len(catalogo)}")
    faltan = [c for c in SUSTITUIDAS_016 if c not in catalogo]
    if faltan:
        falla("B", f"claves sustituidas que no estan en el catalogo: {faltan}")
    print(f"  de ellas, de verificacion y sincronizacion (las sustituyen verificacion y derivados): "
          f"{len(SUSTITUIDAS_016)}")
    MEDIDO["claves de la 0.16 en el catalogo"] = str(len(catalogo))
    MEDIDO["claves de la 0.16 de verificacion y sincronizacion"] = str(len(SUSTITUIDAS_016))
    for clave, y in (("TW", tw_yaml), ("ODT", odt_yaml)):
        usadas = sorted(y)
        ext = [k for k in usadas if k in SUSTITUIDAS_016]
        nuevas = [k for k in sorted(cfgs[clave]) if k in ("verificacion", "derivados")]
        print(f"  {FIJO[clave][0]}: claves en su config 0.16 = {len(usadas)} ; de verificacion y sincronizacion = "
              f"{len(ext)} ({', '.join(ext)}) ; en el esquema nuevo = {len(nuevas)} ({', '.join(nuevas)}) ; "
              f"resto, de F2-7: {[k for k in usadas if k not in SUSTITUIDAS_016]}")
        MEDIDO[f"{FIJO[clave][0]}: claves de verificacion y sincronizacion 0.16 -> 2.0"] = f"{len(ext)} -> {len(nuevas)}"
    total = len(CLAVES_F111) + len(CLAVES_F110)
    print(f"  presupuesto: F1-11 {len(CLAVES_F111)} + F1-10 {len(CLAVES_F110)} = {total} de 12 "
          f"(F1-8 no anade clave propia: sus globs son confianza_incluye)")
    MEDIDO["claves de primer nivel de F1-10 y F1-11 frente al tope de 12"] = f"{total} de 12"
    if total > 12:
        falla("B", "se pasa del tope de 12")
    print(f"RESULTADO B traducidas validas={sum(1 for c in cfgs.values() if c)}/2 ; hostiles rechazadas="
          f"{rech}/{len(HOSTILES)} ; claves={total}/12")
    return cfgs


# ---------------------------------------------------------------------------
# C: derivados en la rama de estado
# ---------------------------------------------------------------------------

GRAFO_PY = '''\
"""Derivado `genera` de la PoC (estilo grafo-tareas): Mermaid por hito de lo no terminado."""
import os, re, sys
T = os.environ["TASKCODE_TAREAS"]
tareas = {}
for d in sorted(os.listdir(T)):
    p = os.path.join(T, d, "tarea.md")
    if re.fullmatch(r"TASK-\\d{3}", d) and os.path.isfile(p):
        fm = open(p, encoding="utf-8").read().split("\\n---\\n", 1)[0]
        v = {k: x.strip().strip('"') for k, x in re.findall(r"^(\\w+): *(.*)$", fm, re.M)}
        deps = [x.strip() for x in v.get("dependencias", "[]").strip("[]").split(",") if x.strip()]
        tareas[d] = (v.get("estado", "?"), v.get("sprint", "0"), deps)
mal = [f"{t} -> {x}" for t, (_e, _s, ds) in tareas.items() for x in ds if x not in tareas]
if mal:
    print("dependencia inexistente: " + ", ".join(mal), file=sys.stderr)
    sys.exit(1)
out = ["# Grafo por hito (generado; no editar)", ""]
for h in sorted({s for (_e, s, _d) in tareas.values()}):
    out += [f"## H{h}", "", "```mermaid", "graph TD"]
    for t, (e, s, ds) in sorted(tareas.items()):
        if s == h and e != "terminada":
            out.append(f"  {t}[{t} {e}]")
            out += [f"  {x} --> {t}" for x in ds]
    out += ["```", ""]
open(os.path.join(T, "grafo-hitos.md"), "w", encoding="utf-8", newline="\\n").write("\\n".join(out))
'''

COMPRUEBA_PY = '''\
"""Derivado `comprueba` de la PoC: cada TASK-NNN citada en el codigo existe en el estado."""
import os, re, sys
T, C = os.environ["TASKCODE_TAREAS"], os.environ["TASKCODE_CODIGO"]
citadas = set()
for f in sorted(os.listdir(os.path.join(C, "src"))):
    citadas |= set(re.findall(r"TASK-\\d{3}", open(os.path.join(C, "src", f), encoding="utf-8").read()))
faltan = sorted(t for t in citadas if not os.path.isfile(os.path.join(T, t, "tarea.md")))
if faltan:
    print("citadas en el codigo y sin tarea: " + ", ".join(faltan), file=sys.stderr)
    sys.exit(1)
'''

MALO_PY = '''\
"""Derivado hostil: escribe en el arbol de codigo."""
import os
open(os.path.join(os.environ["TASKCODE_CODIGO"], "src", "intruso.txt"), "w").write("x")
'''

CFG_C = """\
verificacion:
  comando: 'scripts/verificar.sh'
  shell: 'cmd'
  perfiles: {completa: {timeout: 60, runner: 'github-linux'}}
derivados:
  - comando: 'python -I -B scripts/grafo.py'
    tipo: 'genera'
    escribe: ['tareas/grafo-hitos.md']
  - comando: 'python -I -B scripts/comprueba.py'
    tipo: 'comprueba'
    fases: ['review', 'finish']
"""


def tarea_md(tid, estado, sprint, deps):
    return (f"---\nid: \"{tid}\"\ntitulo: \"Tarea {tid}\"\nestado: \"{estado}\"\nsprint: {sprint}\n"
            f"dependencias: [{', '.join(deps)}]\n---\n## Objetivo\n\nx\n")


def porcelana(repo):
    return set(git(repo, "status", "--porcelain", "--untracked-files=all").splitlines())


def deshacer(repo, lineas):
    for l in sorted(lineas):
        ruta = l[3:]
        if l.startswith("??"):
            os.remove(os.path.join(repo, ruta))
        else:
            git(repo, "checkout", "-q", "--", ruta)


def transicion(raiz, cfg, alm, tid, fase, estado, deps=None, derivados=None):
    """Lo que haria `taskctl <fase>`: escribe la tarea, ejecuta los derivados de esa fase (punto explicito,
    no un hook), y lo commitea todo junto en la rama de estado. Devuelve (avisos, ficheros del commit)."""
    tareas = os.path.join(raiz, "tareas")
    p = os.path.join(tareas, tid, "tarea.md")
    t = open(p, encoding="utf-8").read()
    t = re.sub(r'^estado: .*$', f'estado: "{estado}"', t, count=1, flags=re.M)
    if deps is not None:
        t = re.sub(r'^dependencias: .*$', f"dependencias: [{', '.join(deps)}]", t, count=1, flags=re.M)
    open(p, "w", encoding="utf-8", newline="\n").write(t)
    avisos = []
    env = dict(os.environ, TASKCODE_TAREAS=tareas, TASKCODE_CODIGO=raiz)
    for i, d in enumerate(derivados if derivados is not None else cfg["derivados"]):
        clave = f"derivados[{i}]"
        if fase not in d.get("fases", list(FASES)):
            continue
        st = confianza(raiz, clave, d["comando"], cfg.get("confianza_incluye", []), alm)
        if st != "confiado":
            avisos.append(f"{clave} {st}: no se ejecuta; revisa la orden y `taskctl confiar {clave}`")
            continue
        cod0, tar0 = porcelana(raiz), porcelana(tareas)
        r = subprocess.run(["cmd", "/d", "/c", d["comando"]], cwd=raiz, env=env, capture_output=True,
                           stdin=subprocess.DEVNULL, timeout=d.get("timeout", 60))
        cod1, tar1 = porcelana(raiz), porcelana(tareas)
        permitidas = {f"{tid}/tarea.md"} | {e[len("tareas/"):] for e in d.get("escribe", [])}
        fuera_tar = {l for l in tar1 - tar0 if l[3:] not in permitidas}
        fuera_cod = cod1 - cod0
        if fuera_cod or fuera_tar:
            deshacer(raiz, fuera_cod)
            deshacer(tareas, fuera_tar | {l for l in tar1 - tar0 if l[3:] != f"{tid}/tarea.md"})
            avisos.append(f"{clave} escribio fuera de `escribe` ({sorted(l[3:] for l in fuera_cod | fuera_tar)}): "
                          f"descartado; la transicion sigue")
        elif r.returncode != 0:
            deshacer(tareas, {l for l in tar1 - tar0 if l[3:] != f"{tid}/tarea.md"})
            ult = r.stderr.decode("utf-8", "replace").strip().splitlines()[-1:]
            avisos.append(f"{clave} fallo (codigo {r.returncode}: {' '.join(ult)}); la transicion sigue; "
                          f"arreglalo y ejecuta `taskctl derivados`")
    git(tareas, "add", "-A")
    git(tareas, "commit", "-q", "-m", f"{fase}({tid})")
    ficheros = git(tareas, "show", "--name-only", "--format=", "HEAD").split()
    return avisos, ficheros


def seccion_c(trabajo):
    sep("C: derivados ejecutados por taskctl en la transicion, en la rama de estado")
    raiz = os.path.join(trabajo, "c", "repo")
    os.makedirs(os.path.join(raiz, "src"))
    os.makedirs(os.path.join(raiz, "scripts"))
    git(raiz, "init", "-q", "-b", "develop")
    open(os.path.join(raiz, ".gitignore"), "w").write("tareas/\n")
    open(os.path.join(raiz, "src", "app.txt"), "w").write("implementa TASK-001 y TASK-002\n")
    for n, c in (("grafo.py", GRAFO_PY), ("comprueba.py", COMPRUEBA_PY), ("malo.py", MALO_PY)):
        open(os.path.join(raiz, "scripts", n), "w", encoding="utf-8", newline="\n").write(c)
    os.makedirs(os.path.join(raiz, ".taskcode"))
    open(os.path.join(raiz, ".taskcode", "config.yml"), "w", newline="\n").write(CFG_C)
    git(raiz, "add", "-A")
    git(raiz, "commit", "-q", "-m", "codigo")
    git(raiz, "worktree", "add", "-q", "--orphan", "-b", "taskcode/tareas", "tareas")
    tareas = os.path.join(raiz, "tareas")
    for tid, s, deps in (("TASK-001", 1, []), ("TASK-002", 1, ["TASK-001"]), ("TASK-003", 2, ["TASK-002"])):
        os.makedirs(os.path.join(tareas, tid))
        open(os.path.join(tareas, tid, "tarea.md"), "w", newline="\n").write(tarea_md(tid, "planificada", s, deps))
    git(tareas, "add", "-A")
    git(tareas, "commit", "-q", "-m", "alta")
    mb = git(raiz, "merge-base", "develop", "taskcode/tareas", check=False)
    print(f"merge-base develop taskcode/tareas: {'NINGUNA (huerfana)' if mb.returncode == 1 else 'HAY'}")
    cfg = cargar(open(os.path.join(raiz, ".taskcode", "config.yml")).read())   # la de la base
    alm = os.path.join(trabajo, "c", "almacen-confianza.json")
    for i, d in enumerate(cfg["derivados"]):
        confiar(raiz, f"derivados[{i}]", d["comando"], [], alm)
    arbol0 = git(raiz, "rev-parse", "develop^{tree}").strip()
    ncod0 = git(raiz, "rev-list", "--count", "develop").strip()
    pasos = []

    def paso(nombre, *a, **k):
        av, fs = transicion(raiz, cfg, alm, *a, **k)
        gen = "grafo-hitos.md" in fs
        pasos.append((nombre, av, fs, gen))
        print(f"  {nombre:46} commit={fs} ; grafo regenerado={gen}")
        for x in av:
            print(f"      aviso: {x}")
        return av, gen

    print("-- transiciones")
    av, gen = paso("T1 plan TASK-001", "TASK-001", "plan", "en-diseno")
    ok1 = gen and not av
    av, gen = paso("T2 approve TASK-001", "TASK-001", "approve", "en-curso")
    ok2 = gen and not av
    av, gen = paso("T3 plan TASK-003 con dependencia rota", "TASK-003", "plan", "en-diseno", deps=["TASK-099"])
    ok3 = not gen and any("fallo (codigo 1" in x and "TASK-099" in x for x in av)
    open(os.path.join(raiz, "scripts", "grafo.py"), "a").write("# un pull cambia el generador\n")
    av, gen = paso("T4 review TASK-001 con el generador cambiado", "TASK-001", "review", "en-revision")
    ok4 = not gen and any("derivados[0] cambiado" in x for x in av) and not any("derivados[1]" in x for x in av)
    git(raiz, "checkout", "-q", "--", "scripts/grafo.py")
    open(os.path.join(tareas, "TASK-003", "tarea.md"), "w", newline="\n").write(
        tarea_md("TASK-003", "en-diseno", 2, ["TASK-002"]))
    av, gen = paso("T5 finish TASK-001 (generador restaurado)", "TASK-001", "finish", "terminada")
    ok5 = gen and not av
    print("-- hostiles en ejecucion")
    open(os.path.join(raiz, "src", "nuevo.txt"), "w").write("usa TASK-404\n")
    av, gen = paso("T6 finish TASK-002, codigo cita TASK-404", "TASK-002", "finish", "terminada")
    ok6 = any("derivados[1] fallo" in x and "TASK-404" in x for x in av)
    os.remove(os.path.join(raiz, "src", "nuevo.txt"))
    malo = {"comando": "python -I -B scripts/malo.py", "tipo": "genera", "escribe": ["tareas/grafo-hitos.md"]}
    confiar(raiz, "derivados[0]", malo["comando"], [], alm)
    av, gen = paso("T7 pausa TASK-003, derivado que escribe en codigo", "TASK-003", "pausa", "pausa",
                   derivados=[malo])
    ok7 = any("escribio fuera de `escribe`" in x and "src/intruso.txt" in x for x in av) \
        and not os.path.exists(os.path.join(raiz, "src", "intruso.txt"))
    arbol1 = git(raiz, "rev-parse", "develop^{tree}").strip()
    ncod1 = git(raiz, "rev-list", "--count", "develop").strip()
    limpio = git(raiz, "status", "--porcelain", "--untracked-files=all").strip() == ""
    nest = git(raiz, "rev-list", "--count", "taskcode/tareas").strip()
    print(f"arbol de develop antes={arbol0[:12]} despues={arbol1[:12]} identico={arbol0 == arbol1} ; "
          f"commits en develop {ncod0}->{ncod1} ; status limpio={limpio}")
    print(f"commits en taskcode/tareas: {nest} (alta + {len(pasos)} transiciones, uno por transicion)")
    g = open(os.path.join(tareas, "grafo-hitos.md"), encoding="utf-8").read()
    print(f"grafo-hitos.md final: TASK-001 fuera (terminada)={'TASK-001[' not in g} ; "
          f"TASK-003 en H2={'TASK-003[TASK-003 en-diseno]' in g.split('## H2')[1]}")
    fin_ok = "TASK-001[" not in g and "TASK-003[TASK-003 en-diseno]" in g.split("## H2")[1]
    res = [ok1, ok2, ok3, ok4, ok5, ok6, ok7, arbol0 == arbol1, ncod0 == ncod1, limpio, nest == str(1 + len(pasos)),
           fin_ok]
    for i, (nombre, *_r) in enumerate(pasos):
        if not res[i]:
            falla("C", f"{nombre}: resultado inesperado")
    if not all(res[7:]):
        falla("C", "develop cambio, la rama de estado no tiene un commit por transicion o el grafo final no cuadra")
    MEDIDO["transiciones con derivados en la rama de estado"] = str(len(pasos))
    MEDIDO["arbol de develop tras las transiciones"] = "identico" if arbol0 == arbol1 else "distinto"
    print(f"RESULTADO C transiciones={len(pasos)} ; como se espera={sum(res[:7])}/7 ; develop identico="
          f"{arbol0 == arbol1}")


# ---------------------------------------------------------------------------
# D: lectura del estado
# ---------------------------------------------------------------------------

def vivas_desde_json(salida):
    """Lo que necesita un script del proyecto con `taskctl tareas --json`: ni glob ni lector de frontmatter."""
    return {t["id"] for t in json.loads(salida) if t["estado"] != "descartada"}


def tareas_json_simulado(raiz_tareas):
    """Simula `taskctl tareas --json` sobre la carpeta fija: frontmatter leido con YAML estandar; las
    tareas de 05-archivadas migran con estado `descartada` (D1a; la migracion la fija F1-12)."""
    out = []
    for d in sorted(os.listdir(raiz_tareas)):
        p = os.path.join(raiz_tareas, d, "tarea.md")
        if not os.path.isfile(p):
            continue
        fm = yaml.safe_load(open(p, encoding="utf-8").read().replace("\r\n", "\n").split("\n---\n", 1)[0][4:])
        out.append({"id": fm["id"], "estado": fm["estado"], "etiquetas": fm.get("etiquetas") or [],
                    "dependencias": fm.get("dependencias") or [], "sprint": fm.get("sprint")})
    return json.dumps(out, ensure_ascii=False, default=str)


def cargar_modulo(nombre, ruta):
    spec = importlib.util.spec_from_file_location(nombre, ruta)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


def lineas_funciones(fuente, nombres):
    arbol = ast.parse(fuente)
    return {n.name: n.end_lineno - n.lineno + 1 for n in arbol.body
            if isinstance(n, ast.FunctionDef) and n.name in nombres}


def seccion_d(pil, trabajo):
    sep("D: lectura del estado (lectores propios copiados frente a `tareas --json` simulado)")
    casos = (("ODT", "scripts/lector-tareas.py", "lector_tareas"), ("TW", "scripts/grafo-tareas.py", "grafo_tareas"))
    for clave, script, mod in casos:
        base = os.path.join(trabajo, "d", clave)
        rutas = [p for p in pil.ls(clave, "tareas") if re.fullmatch(r"tareas/0\d-[^/]+/TASK-\d+/tarea\.md", p)]
        for capa in ("v016", "v20"):
            os.makedirs(os.path.join(base, capa, "scripts"))
            open(os.path.join(base, capa, script), "w", encoding="utf-8", newline="\n").write(pil.show(clave, script))
        for p in rutas:
            _t, carpeta, tid, _f = p.split("/")
            txt = pil.show(clave, p)
            for capa, rel in (("v016", (carpeta, tid)), ("v20", (tid,))):
                dst = os.path.join(base, capa, "tareas", *rel)
                os.makedirs(dst, exist_ok=True)
                if capa == "v20" and carpeta == "05-archivadas":
                    txt2 = re.sub(r"^estado: .*$", "estado: descartada", txt, count=1, flags=re.M)
                else:
                    txt2 = txt
                open(os.path.join(dst, "tarea.md"), "w", encoding="utf-8", newline="").write(txt2)
        hoy = cargar_modulo(f"{mod}_v016", os.path.join(base, "v016", script))
        fijo = cargar_modulo(f"{mod}_v20", os.path.join(base, "v20", script))
        def vivas(m):
            t = m.cargar(m.TAREAS)
            return {k for k, fm in t.items() if "descartada" not in (fm.get("etiquetas") or [])}
        v_hoy, v_fijo = vivas(hoy), vivas(fijo)
        salida = tareas_json_simulado(os.path.join(base, "v20", "tareas"))
        v_json = vivas_desde_json(salida)
        print(f"  {FIJO[clave][0]} ({script}): tareas.md={len(rutas)} ; vivas hoy (0.16)={len(v_hoy)} ; "
              f"mismo glob en carpeta fija={len(v_fijo)} ; desde tareas --json={len(v_json)} ; "
              f"mismo conjunto que hoy={v_json == v_hoy}")
        if len(v_fijo) != 0 or v_json != v_hoy or not v_hoy:
            falla("D", f"{clave}: lectura del estado inesperada")
        MEDIDO[f"{FIJO[clave][0]}: tareas que encuentra su lector en la carpeta fija"] = str(len(v_fijo))
    fuente_odt = pil.show("ODT", "scripts/lector-tareas.py")
    fuente_tw = pil.show("TW", "scripts/grafo-tareas.py")
    n_odt = len(fuente_odt.replace("\r\n", "\n").rstrip("\n").split("\n"))
    f_tw = lineas_funciones(fuente_tw, {"quitar_comentario", "leer_tarea", "cargar"})
    f_rm = lineas_funciones(fuente_tw, {"mermaid", "orden_de_trabajo", "roadmap"})
    import inspect
    n_json = len(inspect.getsource(vivas_desde_json).rstrip("\n").split("\n"))
    print(f"  lineas que deja de necesitar un lector propio: OpenDockTrading lector-tareas.py entero = {n_odt} ; "
          f"ToolsaWeb grafo-tareas.py {f_tw} = {sum(f_tw.values())} ; el consumidor de tareas --json = {n_json}")
    print(f"  ToolsaWeb grafo-tareas.py que solapa con el roadmap nativo (F1-1 dec. 3, F3-4) {f_rm} = "
          f"{sum(f_rm.values())} (no se cuenta como ahorro: falta ver que F3-4 cubra su orden)")
    MEDIDO["lineas de lectores propios que sustituye tareas --json"] = f"{n_odt + sum(f_tw.values())} -> {n_json}"
    lineas_bat = pil.show("ODT", "scripts/verificar.bat").replace("\r\n", "\n").split("\n")
    roadmap_paso = [i + 1 for i, l in enumerate(lineas_bat) if "taskctl roadmap --comparar" in l]
    print(f"  paso de OpenDockTrading que desaparece con el roadmap fuera del arbol (F1-7): verificar.bat:{roadmap_paso}")
    MEDIDO["pasos de verificacion que desaparecen (OpenDockTrading)"] = str(len(roadmap_paso))
    print(f"RESULTADO D ok")


# ---------------------------------------------------------------------------
# E: la completa de ToolsaWeb y RAMA_BASE
# ---------------------------------------------------------------------------

def seccion_e(pil, trabajo):
    sep("E: plan --completa del clasificador de ToolsaWeb (copia) con varios RAMA_BASE")
    raiz = os.path.join(trabajo, "e", "tw")
    os.makedirs(raiz)
    datos = git(pil.repo("TW"), "archive", "--format=tar", FIJO["TW"][1], "--", ".",
                ":(exclude,glob)docs*/**",":(exclude,glob)tareas/**", ":(exclude,glob)evidencias/**", binario=True)
    with tarfile.open(fileobj=io.BytesIO(datos)) as tf:
        miembros = [m for m in tf.getmembers() if not (m.name.startswith("/") or ".." in m.name.split("/"))
                    and (m.isfile() or m.isdir())]
        tf.extractall(raiz, members=miembros, filter="data")
    print(f"copia: {sum(1 for m in miembros if m.isfile())} ficheros (sin docs/, tareas/ ni evidencias/)")
    git(raiz, "init", "-q", "-b", "develop")
    git(raiz, "add", "-A")
    git(raiz, "commit", "-q", "-m", "copia")
    git(raiz, "switch", "-q", "-c", "feature/poc")
    with open(os.path.join(raiz, "core", "tw-superficie", "src", "lib.rs"), "a", newline="\n") as f:
        f.write("\n// cambio de la PoC E\n")
    git(raiz, "commit", "-q", "-am", "toca un crate con tests pesados")
    variantes = [("sin definir", None), ("develop", "develop"), ("vacia", ""), ("main (no existe)", "main")]
    planes = {}
    for nombre, valor in variantes:
        shutil.rmtree(os.path.join(raiz, ".estado"), ignore_errors=True)
        env = dict(os.environ)
        env.pop("RAMA_BASE", None)
        for k in [k for k in env if k.startswith("TW_")]:
            env.pop(k)
        if valor is not None:
            env["RAMA_BASE"] = valor
        r = subprocess.run([sys.executable, "-I", "-B", os.path.join(raiz, "scripts", "clasificar-verificacion.py"),
                            "plan", "--completa"], cwd=raiz, env=env, capture_output=True,
                           stdin=subprocess.DEVNULL, timeout=900)
        pj = os.path.join(raiz, ".estado", "verificar-plan.json")
        if r.returncode != 0 or not os.path.exists(pj):
            print(f"  {nombre}: codigo {r.returncode} ; {r.stderr.decode('utf-8', 'replace').strip()[-300:]}")
            planes[nombre] = None
            continue
        p = json.load(open(pj, encoding="utf-8"))
        cmdp = open(os.path.join(raiz, ".estado", "verificar-plan.cmd"), encoding="utf-8").read()
        planes[nombre] = (p, cmdp)
        avisos_cache = [l for l in r.stdout.decode("utf-8", "replace").splitlines() if "caché" in l and "aviso" in l]
        print(f"  RAMA_BASE {nombre:17}: codigo {r.returncode} ; modo={p.get('modo')} es_completa={p.get('es_completa')} "
              f"; grupos={len(p.get('grupos', {}))} ; cacheados={len(p.get('cacheados', {}))} ; "
              f"aviso_pesada={p.get('aviso_pesada')} ; rango={p.get('rango')!r}"
              + (f" ; {avisos_cache[0][:80]}" if avisos_cache else ""))
    validos = {k: v for k, v in planes.items() if v}
    if len(validos) != len(variantes):
        falla("E", "el clasificador no dejo plan en todas las variantes: no verificado")
        return
    ref = validos["develop"][0]
    VARIA = {"rango", "ficheros", "muestra", "aviso_pesada"}
    distintos = set()
    for nombre, (p, cmdp) in validos.items():
        for k in set(p) | set(ref):
            if p.get(k) != ref.get(k):
                distintos.add(k)
    grupos_iguales = all(set(p["grupos"]) == set(ref["grupos"]) for p, _c in validos.values())
    cmd_sin_aviso = {re.sub(r"(?m)^.*TW_AVISO_PESADA.*\n?", "", c) for _p, c in validos.values()}
    print(f"grupos de la completa iguales en las {len(validos)} variantes: {grupos_iguales} ; campos que cambian: "
          f"{sorted(distintos)} ; .cmd igual salvo el aviso: {len(cmd_sin_aviso) == 1}")
    MEDIDO["grupos de la completa de ToolsaWeb segun RAMA_BASE"] = "iguales" if grupos_iguales else "distintos"
    if not grupos_iguales or distintos - VARIA or len(cmd_sin_aviso) != 1:
        falla("E", "la completa depende de RAMA_BASE mas alla del aviso y del rango")
    print(f"RESULTADO E grupos iguales={grupos_iguales} ; cambia solo={sorted(distintos)}")


# ---------------------------------------------------------------------------

def comparar_ventaja(adr):
    sep("Tabla `ventaja` del ADR frente a lo medido")
    _c, filas = entre_marcadores(adr, "ventaja")
    vistas = set()
    for f in filas:
        cifra, valor = f[0], f[1].strip("`")
        vistas.add(cifra)
        med = MEDIDO.get(cifra)
        print(f"  {cifra:70} ADR={valor!r:22} medido={med!r} {'OK' if med == valor else 'DISTINTO'}")
        if med != valor:
            falla("ventaja", f"{cifra}: el ADR dice {valor!r} y lo medido es {med!r}")
    for k in sorted(set(MEDIDO) - vistas):
        print(f"  (medido y no citado en el ADR) {k} = {MEDIDO[k]!r}")


def main():
    sys.stdout.reconfigure(encoding="utf-8", newline="\n")
    if len(sys.argv) != 5:
        raise SystemExit("uso: python -I -B poc-f1-11.py <trabajo> <adr-F1-11> <carpeta-Personal> <plugin-0.16.1>")
    trabajo, adr_p, personal, plugin = (os.path.abspath(a) for a in sys.argv[1:])
    repo_raiz = os.path.abspath(os.path.join(os.path.dirname(adr_p), "..", ".."))
    partes = trabajo.replace("\\", "/").strip("/").split("/")
    if len(partes) < 3:
        raise SystemExit("ERROR: la carpeta de trabajo no puede estar en la raiz de una unidad")
    if os.path.normcase(trabajo).startswith(os.path.normcase(repo_raiz) + os.sep):
        raise SystemExit("ERROR: la carpeta de trabajo no puede estar dentro del repo")
    if os.path.exists(trabajo):
        if os.listdir(trabajo) and not os.path.exists(os.path.join(trabajo, ".poc-f1-11")):
            raise SystemExit("ERROR: la carpeta de trabajo no esta vacia ni es de una ejecucion anterior")
        shutil.rmtree(trabajo, onerror=lambda f, p, e: (os.chmod(p, 0o700), f(p)))
    os.makedirs(trabajo)
    open(os.path.join(trabajo, ".poc-f1-11"), "w").close()
    git_v = run(["git", "--version"]).strip()
    print(f"python {platform.python_version()} (-I: {bool(sys.flags.isolated)}, -B: {bool(sys.flags.dont_write_bytecode)})"
          f" ; {git_v} ; PyYAML {yaml.__version__}")
    if not (sys.flags.isolated and sys.flags.dont_write_bytecode):
        falla("inicio", "no se ejecuta con python -I -B")
    print(f"ADR: {os.path.basename(adr_p)} ; trabajo: .../{'/'.join(partes[-3:])} ; plugin: "
          f"{'/'.join(plugin.replace(chr(92), '/').split('/')[-2:])}")
    pil = Pilotos(personal)
    for clave, (nombre, c) in FIJO.items():
        fecha = git(pil.repo(clave), "log", "-1", "--format=%cs", c).strip()
        print(f"repo {nombre}: commit {c[:9]} ({fecha})")
    adr = open(adr_p, encoding="utf-8").read().replace("\r\n", "\n")
    seccion_a(pil, plugin, adr)
    seccion_b(pil, plugin, trabajo)
    seccion_c(trabajo)
    seccion_d(pil, trabajo)
    seccion_e(pil, trabajo)
    comparar_ventaja(adr)
    sep("RESUMEN")
    if FALLOS:
        print(f"FALLOS ({len(FALLOS)}):")
        for f in FALLOS:
            print(f"  {f}")
        sys.exit(1)
    print("todo como se espera: A, B, C, D, E y la tabla de ventaja")


if __name__ == "__main__":
    main()

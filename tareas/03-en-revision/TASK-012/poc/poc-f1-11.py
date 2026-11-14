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
    """Vocabulario de F1-7 dec. 5: `github` con el sistema (windows o linux) o una lista de etiquetas."""
    if isinstance(r, dict) and list(r) == ["github"] and r["github"] in ("windows", "linux"):
        return None
    if isinstance(r, list) and r and all(isinstance(e, str) and e for e in r) and r[0] == "self-hosted":
        return None
    return f"{p}: runner {r!r}: {{github: windows|linux}} o [self-hosted, ...] (F1-7 dec. 5)"


RX_RESERVADO_WIN = re.compile(r"^(con|prn|aux|nul|com[1-9]|lpt[1-9])(\..*)?$", re.I)


def normalizar_escribe(r):
    """(ruta normalizada, None) o (None, motivo). Resuelve `.`, `..` y separadores repetidos antes de comparar;
    las reservadas se comparan sin distinguir mayusculas (NTFS y APFS no las distinguen)."""
    if not isinstance(r, str) or not r:
        return None, "ruta vacia o no es texto"
    if "\\" in r or ":" in r or r.startswith("/") or re.search(r"[\x00-\x1f*?\"<>|]", r):
        return None, "ruta absoluta o con un caracter no admitido (\\ : * ? \" < > | o de control)"
    partes = []
    for c in r.split("/"):
        if c in ("", "."):
            continue
        if c == "..":
            if not partes:
                return None, "sale de la raiz"
            partes.pop()
            continue
        if c.lower() == ".git":
            return None, "toca .git"
        if RX_RESERVADO_WIN.match(c) or c.endswith((".", " ")):
            return None, f"nombre reservado o no portable en Windows ({c!r})"
        partes.append(c)
    if not partes or partes[0] != "tareas":
        return None, "fuera de tareas/ (D2a: solo la rama de estado)"
    if len(partes) < 2:
        return None, "tareas/ entera (toda la rama de estado)"
    n = "/".join(partes)
    baja = n.lower()
    if baja in {x.lower() for x in RESERVADAS_TAREAS} or re.match(r"tareas/task-\d+(/|$)", baja):
        return None, "lo escribe taskctl (roadmap o carpeta de tarea)"
    return n, None


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
            _n, motivo = normalizar_escribe(r)
            if motivo:
                err.append(f"{q}.escribe: {r!r}: {motivo}")
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
# A2: clasificacion parche de TaskCode / propio del proyecto (decision de la persona, 2026-11-14)
# ---------------------------------------------------------------------------

# Parche: existe para que la 0.16 funcione o para tapar sus carencias, y la 2.0 lo cubre de forma generica:
# se retira del proyecto al migrar. Propio: como se verifica o que es el proyecto: se queda.
# Ficheros enteros o por funciones (lo que no se nombra de un fichero partido es propio).
DESGLOSE_FICHEROS = [
    ("TW", "scripts/cache-verificacion.py", "entero", "parche",
     "cache por paso y completa del mismo arbol, para no repetir lo que la 0.16 relanza en cada turno y ronda (F1-7 dec. 2)"),
    ("TW", "scripts/cerrojo-verificacion.py", "entero", "parche",
     "una verificacion a la vez: la 0.16 la lanza desde el freno, finish y el guardian (F1-7, F1-8 dec. 6)"),
    ("TW", "scripts/clasificar-verificacion.py", "entero", "parche",
     "selectiva para las rondas de review de la 0.16; generica solo si F5-3 da el impacto (pregunta abierta)"),
    ("TW", "scripts/recuento-tests.py", "entero", "parche", "linea «# tests N» para el registro de puertas de la 0.16 (F5-4)"),
    ("TW", "scripts/resumen-verificacion.py", "entero", "parche",
     "x/N de TASKCTL_PROGRESO, evidencia de la ronda y «# tests N» (F5-4); incluye el resumen del panel de tareas"),
    ("TW", ".claude/settings.json", "entero", "parche", "guardas del arnes frente a fallos de la 0.16 (F1-3, F1-5)"),
    ("TW", "scripts/grafo-tareas.py", "quitar_comentario leer_tarea cargar", "parche", "lector propio (tareas --json)"),
    ("TW", "scripts/grafo-tareas.py", "roadmap", "parche",
     "tareas/ROADMAP.md lo escribe taskctl (F1-1 dec. 3); orden_de_trabajo se queda: lo usa el comprueba"),
    ("TW", "scripts/verificar-rapida.py", "entero", "propio", "que corre la rapida del proyecto"),
    ("ODT", "scripts/lector-tareas.py", "entero", "parche", "lector propio (tareas --json)"),
    ("ODT", "scripts/comprobar-modulos.py", "_cargar_grafo tareas_vivas", "parche", "lectura propia del estado (tareas --json)"),
    ("ODT", "scripts/comprobar-fallos.py", "_cargar_grafo tareas_vivas", "parche", "lectura propia del estado (tareas --json)"),
]

# verificar.bat de OpenDockTrading: lineas de parche; el resto, propio.
ODT_BAT_PARCHE = [(3, 5, "lo lanzan el freno"), (16, 18, "Un candado"), (25, 25, "taskctl tiene que estar en el PATH"),
                  (35, 37, "candado"), (44, 47, "sin_candado"), (59, 59, "taskctl roadmap --comparar")]

# verificar.bat de ToolsaWeb: bloques de parche (con un texto que tienen que contener) y patrones por linea.
TW_BAT_BLOQUES = [(34, 35, "panel de tareas"), (37, 43, "clasificar-verificacion.py"), (48, 52, "TASKCTL_PROGRESO"),
                  (61, 67, "TW_CACHE"), (70, 70, "--selectiva"), (76, 76, "--selectiva"), (80, 107, "plan --completa"),
                  (111, 112, "cache-verificacion.py"), (535, 537, "TW_SELECTIVA"), (543, 554, "completa_cacheada"),
                  (630, 631, "Sale con 10"), (634, 634, ":paso_corre"), (642, 649, ":grupo"),
                  (651, 653, "Cerrojo"), (658, 668, "verificar.lck"), (670, 675, "con_cerrojo"), (678, 682, "recuento")]
TW_BAT_PATRONES = [r"cache-verificacion\.py (verde|completa|instantanea)", r"clasificar-verificacion\.py",
                   r"cerrojo-verificacion\.py", r"recuento-tests\.py", r"resumen-verificacion\.py", r"verificar\.lck",
                   r"TW_CERROJO", r"call :grupo", r"if errorlevel 10", r"^:tras_", r"TASKCTL_PROGRESO", r"TW_PLAN",
                   r"TW_G_", r"TW_AVISO_PESADA", r"TW_CACHE", r"[Rr]ecuento", r"comando_verificacion", r"\btaskctl\b",
                   r"\bfreno\b", r"TaskCode"]
TW_BAT_ENVOLTORIO = r"cache-verificacion\.py correr "   # propio: se queda sin el envoltorio de la cache


def rangos(lineas):
    out, ini, prev = [], None, None
    for n in sorted(lineas):
        if ini is None:
            ini = prev = n
        elif n == prev + 1:
            prev = n
        else:
            out.append((ini, prev))
            ini = prev = n
    if ini is not None:
        out.append((ini, prev))
    return ", ".join(f"{a}" if a == b else f"{a}-{b}" for a, b in out)


def seccion_a2(pil, adr):
    sep("A2: clasificacion de cada practica y de los ficheros (parche de TaskCode / propio del proyecto)")
    cab, filas = entre_marcadores(adr, "inventario")
    col = {c: i for i, c in enumerate(cab)}
    if "Clasificación" not in col:
        raise SystemExit("ERROR: falta la columna Clasificación en inventario")
    cuenta = {}
    for f in filas:
        c = f[col["Clasificación"]]
        cuenta.setdefault(c, []).append(f[col["#"]])
        if c not in ("parche", "propio"):
            falla("A2", f"practica {f[col['#']]}: clasificacion {c!r}")
    print(f"practicas: parche={len(cuenta.get('parche', []))} ({' '.join(cuenta.get('parche', []))}) ; "
          f"propio={len(cuenta.get('propio', []))} ({' '.join(cuenta.get('propio', []))})")
    a_mano = [f for f in filas if f[col["En la 0.16"]] == "a mano"]
    poc = {}
    for f in a_mano:
        poc.setdefault(f[col["En la PoC"]], []).append(f[col["#"]])
    print(f"las {len(a_mano)} practicas a mano en la 0.16, segun la PoC: " +
          " ; ".join(f"{k}={len(v)} ({' '.join(v)})" for k, v in sorted(poc.items())))
    for k in ("comprobado", "proyectado", "dentro del comando", "convención", "fuera"):
        MEDIDO[f"practicas a mano en la 0.16: {k}"] = str(len(poc.get(k, [])))
    MEDIDO["practicas que son parche de TaskCode"] = str(len(cuenta.get("parche", [])))
    MEDIDO["practicas propias del proyecto"] = str(len(cuenta.get("propio", [])))
    retira, queda = {"TW": 0, "ODT": 0}, {"TW": 0, "ODT": 0}
    partidos = {}
    print("-- ficheros enteros o por funciones")
    for clave, ruta, alcance, clase, motivo in DESGLOSE_FICHEROS:
        fuente = pil.show(clave, ruta).replace("\r\n", "\n")
        total = len(fuente.rstrip("\n").split("\n"))
        if alcance == "entero":
            n = total
            donde = f"1-{total}"
        else:
            arbol = ast.parse(fuente)
            fs = {x.name: (x.lineno, x.end_lineno) for x in arbol.body if isinstance(x, ast.FunctionDef)}
            faltan = [a for a in alcance.split() if a not in fs]
            if faltan:
                falla("A2", f"{clave} {ruta}: no tiene {faltan}")
                continue
            n = sum(fs[a][1] - fs[a][0] + 1 for a in alcance.split())
            donde = ", ".join(f"{a} {fs[a][0]}-{fs[a][1]}" for a in alcance.split())
        if clase == "parche":
            retira[clave] += n
            partidos[(clave, ruta)] = (total, partidos.get((clave, ruta), (total, 0))[1] + n)
        else:
            queda[clave] += n
        print(f"  {clave} {ruta}: {clase} {n} de {total} lineas ({donde}) ; {motivo}")
    for (clave, ruta), (total, n) in sorted(partidos.items()):
        if n < total:
            queda[clave] += total - n
            print(f"  {clave} {ruta}: propio el resto, {total - n} lineas")
    print("-- OpenDockTrading scripts/verificar.bat")
    bat = pil.show("ODT", "scripts/verificar.bat").replace("\r\n", "\n").rstrip("\n").split("\n")
    parche = set()
    for a, b, frag in ODT_BAT_PARCHE:
        if norm(frag) not in norm(" ".join(bat[a - 1:b])):
            falla("A2", f"ODT verificar.bat:{a}-{b} no contiene «{frag}»")
        parche |= set(range(a, b + 1))
    retira["ODT"] += len(parche)
    queda["ODT"] += len(bat) - len(parche)
    print(f"  parche {len(parche)} de {len(bat)} lineas ({rangos(parche)}) ; propio {len(bat) - len(parche)}")
    print("-- ToolsaWeb scripts/verificar.bat (bloques declarados y patrones por linea)")
    bat = pil.show("TW", "scripts/verificar.bat").replace("\r\n", "\n").rstrip("\n").split("\n")
    parche = set()
    for a, b, frag in TW_BAT_BLOQUES:
        if norm(frag) not in norm(" ".join(bat[a - 1:b])):
            falla("A2", f"TW verificar.bat:{a}-{b} no contiene «{frag}»")
        parche |= set(range(a, b + 1))
    por_patron = {i + 1 for i, l in enumerate(bat) if any(re.search(p, l) for p in TW_BAT_PATRONES)}
    parche |= por_patron
    envoltorio = {i + 1 for i, l in enumerate(bat) if re.search(TW_BAT_ENVOLTORIO, l)} - parche
    retira["TW"] += len(parche)
    queda["TW"] += len(bat) - len(parche)
    print(f"  parche {len(parche)} de {len(bat)} lineas: {rangos(parche)}")
    print(f"  propio {len(bat) - len(parche)} lineas ; de ellas {len(envoltorio)} se quedan sin el envoltorio "
          f"«cache-verificacion.py correr»")
    MEDIDO["ToolsaWeb verificar.bat: lineas de parche frente a propias"] = f"{len(parche)} / {len(bat) - len(parche)}"
    for clave in ("TW", "ODT"):
        print(f"  {FIJO[clave][0]}: lineas que se retiran={retira[clave]} ; lineas que se quedan={queda[clave]} "
              f"(de los ficheros desglosados)")
        MEDIDO[f"{FIJO[clave][0]}: lineas que se retiran frente a las que se quedan"] = f"{retira[clave]} / {queda[clave]}"
    n_ficheros = sum(1 for x in DESGLOSE_FICHEROS if x[2] == "entero" and x[3] == "parche")
    MEDIDO["ficheros enteros que se retiran"] = str(n_ficheros)
    print(f"RESULTADO A2 parches={len(cuenta.get('parche', []))} ; ficheros enteros que se retiran={n_ficheros} ; "
          f"lineas TW {retira['TW']}/{queda['TW']} ; ODT {retira['ODT']}/{queda['ODT']}")


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
  - comando: 'python -I scripts/grafo-tareas.py --escribir'
    tipo: 'genera'
    escribe: ['tareas/hitos.md']
  - comando: 'python -I scripts/grafo-tareas.py --comprobar'
    tipo: 'comprueba'
confianza_incluye: ['scripts/cache-verificacion.py', 'scripts/memo_ficheros.py']
"""

CFG_ODT = """\
# OpenDockTrading traducida al esquema nuevo
verificacion:
  comando: 'scripts\\verificar.bat'
  shell: 'cmd'
  perfiles:
    completa: {timeout: 1200, runner: {github: 'windows'}}
  huella_arbol: ['Cargo.lock', 'web/package-lock.json']
  programada:
    comando: 'cargo audit --no-yanked --deny unsound && npm --prefix web audit'
    cron: '0 6 * * 1'
    runner: {github: 'windows'}
derivados:
  - comando: 'python -I scripts/comprobar-modulos.py --comprobar'
    tipo: 'comprueba'
  - comando: 'python -I scripts/comprobar-fallos.py --probar'
    tipo: 'comprueba'
confianza_incluye: ['scripts/lector-tareas.py']
"""

HOSTILES = [
    ("clave desconocida (la 0.16 en plano)", "comando_verificacion_freno: 'scripts/verificar.bat --rapida'\n"),
    ("clave desconocida dentro de verificacion",
     "verificacion: {comando: 'a.bat', shell: 'cmd', perfiles: {completa: {timeout: 60, runner: {github: 'windows'}}},"
     " presupuesto_freno_s: 180}\n"),
    ("ruta de Windows con \\v entre comillas dobles (linea 9 de ToolsaWeb)",
     "verificacion:\n  comando: \"scripts\\verificar.bat\"\n  shell: 'cmd'\n"
     "  perfiles: {completa: {timeout: 60, runner: {github: 'windows'}}}\n"),
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
     " runner: {github: 'windows'}}, selectiva: {timeout: 60}}}\n"),
    ("rapida con runner (corre en local)", "verificacion: {comando: 'a.bat', shell: 'cmd', perfiles: {completa: "
     "{timeout: 60, runner: {github: 'windows'}}, rapida: {timeout: 60, runner: {github: 'linux'}}}}\n"),
    ("confianza_incluye que hashea todo el arbol", "confianza_incluye: ['**']\n"),
    ("timeout como texto", "verificacion: {comando: 'a.bat', shell: 'cmd', perfiles: {completa: {timeout: '60',"
     " runner: {github: 'windows'}}}}\n"),
    ("runner con el vocabulario viejo (github-windows)", "verificacion: {comando: 'a.bat', shell: 'cmd', perfiles: "
     "{completa: {timeout: 60, runner: 'github-windows'}}}\n"),
] + [(f"escribe {r!r} (revision 1, IMP-2)",
      f"derivados:\n  - {{comando: 'g.py', tipo: 'genera', escribe: ['{r}']}}\n")
     for r in ("tareas/./ROADMAP.md", "tareas/roadmap.md", "tareas/./TASK-001/tarea.md", "tareas//TASK-001/tarea.md",
               "tareas/task-001/tarea.md", "tareas/", "tareas/.git", "tareas/x:ads")]

ACEPTADAS_ESCRIBE = ["tareas/hitos.md", "tareas/informes/grafo.md", "tareas/./hitos.md"]

def hojas(x, p=""):
    """Valores que una persona rellena: cada escalar, o cada lista de escalares como un solo valor."""
    if isinstance(x, dict):
        return [h for k, v in x.items() for h in hojas(v, f"{p}.{k}" if p else k)]
    if isinstance(x, list) and x and all(isinstance(e, dict) for e in x):
        return [h for i, e in enumerate(x) for h in hojas(e, f"{p}[{i}]")]
    return [p]


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
    print("-- rutas de escribe que se aceptan, ya normalizadas")
    for r in ACEPTADAS_ESCRIBE:
        n, motivo = normalizar_escribe(r)
        print(f"  {r!r:30} -> {repr(n) if n else 'RECHAZADA: ' + motivo}")
        if not n:
            falla("B", f"ruta legitima rechazada: {r}")
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
        valores = hojas({k: v for k, v in cfgs[clave].items()})
        print(f"  {FIJO[clave][0]}: claves en su config 0.16 = {len(usadas)} ; de verificacion y sincronizacion = "
              f"{len(ext)} ({', '.join(ext)}) ; bloques en el esquema nuevo = {len(nuevas)} ({', '.join(nuevas)}) ; "
              f"resto, de F2-7: {[k for k in usadas if k not in SUSTITUIDAS_016]}")
        print(f"  {FIJO[clave][0]}: valores a rellenar 0.16 = {len(ext)} ; 2.0 = {len(valores)} ({', '.join(valores)})")
        MEDIDO[f"{FIJO[clave][0]}: claves planas 0.16 -> bloques 2.0"] = f"{len(ext)} -> {len(nuevas)}"
        MEDIDO[f"{FIJO[clave][0]}: valores a rellenar 0.16 -> 2.0"] = f"{len(ext)} -> {len(valores)}"
    total = len(CLAVES_F111) + len(CLAVES_F110)
    MEDIDO["claves hoja del esquema de F1-11"] = str(len(ESQUEMA - {"verificacion", "derivados"}))
    print(f"  esquema de F1-11: {len(CLAVES_F111)} claves de primer nivel y {len(ESQUEMA - {"verificacion", "derivados"})} hojas")
    print(f"  presupuesto (el tope de 12 de F2-7 cuenta claves de primer nivel): F1-11 {len(CLAVES_F111)} + F1-10 "
          f"{len(CLAVES_F110)} = {total} de 12 (F1-8 no anade clave propia: sus globs son confianza_incluye)")
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
"""Derivados hostiles de la PoC: escriben donde no deben, segun el argumento."""
import os, re, sys
T, C = os.environ["TASKCODE_TAREAS"], os.environ["TASKCODE_CODIGO"]
modo = sys.argv[1]
if modo == "codigo":
    open(os.path.join(C, "src", "intruso.txt"), "w").write("x")
elif modo == "ignorado":
    os.makedirs(os.path.join(C, ".estado"), exist_ok=True)
    open(os.path.join(C, ".estado", "x.txt"), "w").write("x")
elif modo == "tarea":
    p = os.path.join(T, sys.argv[2], "tarea.md")
    t = open(p, encoding="utf-8").read()
    t = re.sub(r'^estado: .*$', 'estado: "terminada"', t, count=1, flags=re.M)
    open(p, "w", encoding="utf-8", newline="\\n").write(t)
'''

CFG_C = """\
verificacion:
  comando: 'scripts/verificar.sh'
  shell: 'cmd'
  perfiles: {completa: {timeout: 60, runner: {github: 'linux'}}}
derivados:
  - comando: 'python -I -B scripts/grafo.py'
    tipo: 'genera'
    escribe: ['tareas/grafo-hitos.md']
  - comando: 'python -I -B scripts/comprueba.py'
    tipo: 'comprueba'
    fases: ['review', 'finish']
"""

AUSENTE = object()


def tarea_md(tid, estado, sprint, deps):
    return (f"---\nid: \"{tid}\"\ntitulo: \"Tarea {tid}\"\nestado: \"{estado}\"\nsprint: {sprint}\n"
            f"dependencias: [{', '.join(deps)}]\n---\n## Objetivo\n\nx\n")


def foto_tareas(tareas):
    """Contenido de todo fichero de la rama de estado montada (sin su enlace .git)."""
    f = {}
    for base, dirs, ficheros in os.walk(tareas):
        dirs[:] = [d for d in dirs if d != ".git"]
        for n in ficheros:
            rel = os.path.relpath(os.path.join(base, n), tareas).replace(os.sep, "/")
            if rel != ".git":
                f[rel] = open(os.path.join(base, n), "rb").read()
    return f


def foto_codigo(raiz):
    """Contenido de lo modificado, nuevo o ignorado del arbol de codigo (sin tareas/): un fichero limpio que
    cambia aparece despues; uno sucio que se vuelve a cambiar se ve por contenido; uno borrado, como None."""
    f = {}
    for e in git(raiz, "status", "--porcelain=v1", "-z", "--ignored", "--untracked-files=all").split("\0"):
        if len(e) > 3:
            rel = e[3:]
            if rel == "tareas/" or rel.startswith("tareas/"):
                continue
            p = os.path.join(raiz, rel)
            f[rel] = open(p, "rb").read() if os.path.isfile(p) else None
    return f


def cambios(a, b):
    return sorted(p for p in set(a) | set(b) if a.get(p, AUSENTE) != b.get(p, AUSENTE))


def restaurar(base, antes, rutas, versionados=frozenset()):
    for p in rutas:
        q = os.path.join(base, p)
        v = antes.get(p, AUSENTE)
        if v is AUSENTE and p in versionados:
            git(base, "checkout", "-q", "--", p)
        elif v is AUSENTE or v is None:
            if os.path.isfile(q):
                os.remove(q)
        else:
            os.makedirs(os.path.dirname(q), exist_ok=True)
            open(q, "wb").write(v)


def lanzar(raiz, tareas, d, codigo=None):
    """Lanza la orden con cwd=raiz (de donde salen los scripts); TASKCODE_CODIGO es el arbol que se comprueba."""
    env = dict(os.environ, TASKCODE_TAREAS=tareas, TASKCODE_CODIGO=codigo or raiz)
    return subprocess.run(["cmd", "/d", "/c", d["comando"]], cwd=raiz, env=env, capture_output=True,
                          stdin=subprocess.DEVNULL, timeout=d.get("timeout", 60))


def sale_de_tareas(tareas, ruta):
    """True si algun componente de la ruta (bajo tareas/) es un enlace simbolico o una union de Windows, o si
    su destino real sale de tareas/ (revision 2, nota del revisor)."""
    base = os.path.realpath(tareas)
    actual = tareas
    for c in ruta[len("tareas/"):].split("/"):
        actual = os.path.join(actual, c)
        if os.path.lexists(actual) and (os.path.islink(actual) or os.path.isjunction(actual)):
            return True
    real = os.path.realpath(actual)
    return os.path.commonpath([os.path.normcase(base), os.path.normcase(real)]) != os.path.normcase(base)


def transicion(raiz, cfg, alm, tid, fase, estado, deps=None, derivados=None):
    """Lo que haria `taskctl <fase>`: escribe la tarea, ejecuta los derivados de esa fase (punto explicito,
    no un hook), y lo commitea todo junto en la rama de estado. Lo escrito se compara por CONTENIDO: en la
    rama de estado, todo fichero fuera de `escribe` (tambien el tarea.md de la transicion); en el codigo,
    cualquier cambio, tambien en rutas ignoradas. Devuelve (avisos, ficheros del commit)."""
    tareas = os.path.join(raiz, "tareas")
    p = os.path.join(tareas, tid, "tarea.md")
    t = open(p, encoding="utf-8").read()
    t = re.sub(r'^estado: .*$', f'estado: "{estado}"', t, count=1, flags=re.M)
    if deps is not None:
        t = re.sub(r'^dependencias: .*$', f"dependencias: [{', '.join(deps)}]", t, count=1, flags=re.M)
    open(p, "w", encoding="utf-8", newline="\n").write(t)
    avisos = []
    versionados = frozenset(git(raiz, "ls-files").splitlines())
    for i, d in enumerate(derivados if derivados is not None else cfg["derivados"]):
        clave = f"derivados[{i}]"
        if fase not in d.get("fases", list(FASES)):
            continue
        st = confianza(raiz, clave, d["comando"], cfg.get("confianza_incluye", []), alm)
        if st != "confiado":
            avisos.append(f"{clave} {st}: no se ejecuta; revisa la orden y `taskctl confiar {clave}`")
            continue
        escribe = [normalizar_escribe(e)[0] for e in d.get("escribe", [])]
        enlaces = [e for e in escribe if sale_de_tareas(tareas, e)]
        if enlaces:
            avisos.append(f"{clave} escribe a traves de un enlace o fuera de tareas/ ({enlaces}): no se ejecuta")
            continue
        ignoradas = [e for e in escribe
                     if git(tareas, "check-ignore", "-q", e[len("tareas/"):], check=False).returncode == 0]
        if ignoradas:
            avisos.append(f"{clave} escribe una ruta ignorada en la rama de estado ({ignoradas}): no se "
                          f"commitearia; no se ejecuta")
            continue
        permitidas = {e[len("tareas/"):].lower() for e in escribe}
        ft0, fc0 = foto_tareas(tareas), foto_codigo(raiz)
        r = lanzar(raiz, tareas, d)
        ft1, fc1 = foto_tareas(tareas), foto_codigo(raiz)
        ct, cc = cambios(ft0, ft1), cambios(fc0, fc1)
        fuera = [x for x in ct if x.lower() not in permitidas]
        if cc or fuera:
            restaurar(tareas, ft0, ct)
            restaurar(raiz, fc0, cc, versionados)
            avisos.append(f"{clave} escribio fuera de `escribe` ({sorted(fuera + ['codigo:' + x for x in cc])}): "
                          f"descartado; la transicion sigue")
        elif r.returncode != 0:
            restaurar(tareas, ft0, ct)
            ult = r.stderr.decode("utf-8", "replace").strip().splitlines()[-1:]
            avisos.append(f"{clave} fallo (codigo {r.returncode}: {' '.join(ult)}); la transicion sigue; "
                          f"arreglalo y ejecuta `taskctl derivados`")
    git(tareas, "add", "-A")
    git(tareas, "commit", "-q", "-m", f"{fase}({tid})")
    ficheros = git(tareas, "show", "--name-only", "--format=", "HEAD").split()
    return avisos, ficheros


def check_estado(raiz, pr, trabajo, base="develop", de_la_base=True):
    """El check `estado` del PR (revision 2, IMP-5): la config `derivados` y los scripts `comprueba` salen DE LA
    RAMA BASE (una copia de `git archive <base>`), nunca del arbol del PR; se ejecutan contra el arbol del PR
    (TASKCODE_CODIGO) y la rama de estado; no se cachea; rojo (bloquea) si alguno sale distinto de 0.
    Con de_la_base=False hace lo vulnerable (config y scripts del PR), solo para contrastar."""
    if de_la_base:
        origen = os.path.join(trabajo, "c", f"base-{git(raiz, 'rev-parse', '--short=12', base).strip()}")
        if not os.path.isdir(origen):
            os.makedirs(origen)
            with tarfile.open(fileobj=io.BytesIO(git(raiz, "archive", "--format=tar", base, binario=True))) as tf:
                tf.extractall(origen, filter="data")
    else:
        origen = pr
    cfg = cargar(open(os.path.join(origen, ".taskcode", "config.yml"), encoding="utf-8").read())
    rojos = []
    for i, d in enumerate(cfg.get("derivados", [])):
        if d["tipo"] == "comprueba":
            r = lanzar(origen, os.path.join(raiz, "tareas"), d, codigo=pr)
            if r.returncode != 0:
                rojos.append(f"derivados[{i}] codigo {r.returncode}: "
                             f"{' '.join(r.stderr.decode('utf-8', 'replace').strip().splitlines()[-1:])}")
    return rojos


def seccion_c(trabajo):
    sep("C: derivados ejecutados por taskctl en la transicion, en la rama de estado")
    raiz = os.path.join(trabajo, "c", "repo")
    os.makedirs(os.path.join(raiz, "src"))
    os.makedirs(os.path.join(raiz, "scripts"))
    git(raiz, "init", "-q", "-b", "develop")
    open(os.path.join(raiz, ".gitignore"), "w").write("tareas/\n.estado/\n")
    open(os.path.join(raiz, "src", "app.txt"), "w").write("implementa TASK-001 y TASK-002\n")
    for n, c in (("grafo.py", GRAFO_PY), ("comprueba.py", COMPRUEBA_PY), ("malo.py", MALO_PY)):
        open(os.path.join(raiz, "scripts", n), "w", encoding="utf-8", newline="\n").write(c)
    os.makedirs(os.path.join(raiz, ".taskcode"))
    open(os.path.join(raiz, ".taskcode", "config.yml"), "w", newline="\n").write(CFG_C)
    git(raiz, "add", "-A")
    git(raiz, "commit", "-q", "-m", "codigo")
    git(raiz, "worktree", "add", "-q", "--orphan", "-b", "taskcode/tareas", "tareas")
    tareas = os.path.join(raiz, "tareas")
    open(os.path.join(tareas, ".gitignore"), "w", newline="\n").write("*.tmp\nenlace\n")
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
        print(f"  {nombre:52} commit={fs} ; grafo regenerado={gen}")
        for x in av:
            print(f"      aviso: {x}")
        return av, gen

    def hostil(modo, *extra, escribe=("tareas/grafo-hitos.md",), comando=None):
        d = {"comando": comando or " ".join(["python -I -B scripts/malo.py", modo, *extra]), "tipo": "genera",
             "escribe": list(escribe)}
        confiar(raiz, "derivados[0]", d["comando"], [], alm)
        return [d]

    def estado_de(tid):
        return re.search(r'^estado: "?([^"\n]+)"?$', open(os.path.join(tareas, tid, "tarea.md"), encoding="utf-8").read(),
                         re.M).group(1)

    print("-- transiciones")
    av, gen = paso("T1 plan TASK-001", "TASK-001", "plan", "en-diseno")
    ok = [gen and not av]
    av, gen = paso("T2 approve TASK-001", "TASK-001", "approve", "en-curso")
    ok.append(gen and not av)
    av, gen = paso("T3 plan TASK-003 con dependencia rota", "TASK-003", "plan", "en-diseno", deps=["TASK-099"])
    ok.append(not gen and any("fallo (codigo 1" in x and "TASK-099" in x for x in av))
    open(os.path.join(raiz, "scripts", "grafo.py"), "a").write("# un pull cambia el generador\n")
    av, gen = paso("T4 review TASK-001 con el generador cambiado", "TASK-001", "review", "en-revision")
    ok.append(not gen and any("derivados[0] cambiado" in x for x in av) and not any("derivados[1]" in x for x in av))
    git(raiz, "checkout", "-q", "--", "scripts/grafo.py")
    open(os.path.join(tareas, "TASK-003", "tarea.md"), "w", newline="\n").write(
        tarea_md("TASK-003", "en-diseno", 2, ["TASK-002"]))
    av, gen = paso("T5 finish TASK-001 (generador restaurado)", "TASK-001", "finish", "terminada")
    ok.append(gen and not av)
    print("-- un comprueba en rojo: la transicion avisa; el check `estado` del PR bloquea")
    open(os.path.join(raiz, "src", "nuevo.txt"), "w").write("usa TASK-404\n")
    av, gen = paso("T6 finish TASK-002, codigo cita TASK-404", "TASK-002", "finish", "terminada")
    rojo = check_estado(raiz, raiz, trabajo)
    print(f"  check estado en el PR con ese codigo: {'ROJO (bloquea): ' + '; '.join(rojo) if rojo else 'verde'}")
    os.remove(os.path.join(raiz, "src", "nuevo.txt"))
    verde = check_estado(raiz, raiz, trabajo)
    print(f"  check estado sin la cita rota: {'ROJO' if verde else 'verde'}")
    ok.append(any("derivados[1] fallo" in x and "TASK-404" in x for x in av) and bool(rojo) and not verde)
    MEDIDO["check estado con un comprueba en rojo"] = "rojo (bloquea)" if rojo else "verde"
    print("-- PR que cita TASK-404 y ademas cambia el comprobador y la config de derivados para ponerse en verde")
    pr = os.path.join(trabajo, "c", "pr")
    git(raiz, "worktree", "add", "-q", "-b", "feature/pr", pr, "develop")
    open(os.path.join(pr, "src", "nuevo.txt"), "w").write("usa TASK-404\n")
    open(os.path.join(pr, "scripts", "comprueba.py"), "w", newline="\n").write("import sys\nsys.exit(0)\n")
    cfg_pr = CFG_C.split("  - comando: 'python -I -B scripts/comprueba.py'")[0]
    open(os.path.join(pr, ".taskcode", "config.yml"), "w", newline="\n").write(cfg_pr)
    git(pr, "add", "-A")
    git(pr, "commit", "-q", "-m", "pr que se pone en verde")
    tocados = git(raiz, "diff", "--name-only", "develop...feature/pr").split()
    con_base = check_estado(raiz, pr, trabajo)
    con_pr = check_estado(raiz, pr, trabajo, de_la_base=False)
    riesgo = any(t.split("/")[0] == ".taskcode" for t in tocados)
    print(f"  ficheros del PR: {tocados} ; toca .taskcode/ -> riesgo alto por ruta (F1-10): {riesgo}")
    print(f"  check estado con config y comprobadores de la base: "
          f"{'ROJO (bloquea): ' + '; '.join(con_base) if con_base else 'verde'}")
    print(f"  lo vulnerable, con los del PR (revision 2, IMP-5): {'ROJO' if con_pr else 'verde'}")
    if not (con_base and not con_pr and riesgo):
        falla("C", "el check estado no lee los comprobadores de la base")
    MEDIDO["check estado de un PR que cambia el comprobador"] = "rojo (bloquea)" if con_base else "verde"
    print("-- hostiles en ejecucion (lo escrito se compara por contenido)")
    av, gen = paso("T7 pausa TASK-003, derivado que escribe en codigo", "TASK-003", "pausa", "pausa",
                   derivados=hostil("codigo"))
    ok.append(any("codigo:src/intruso.txt" in x for x in av) and not os.path.exists(os.path.join(raiz, "src", "intruso.txt")))
    av, gen = paso("T8 plan TASK-003, derivado que reescribe su tarea.md", "TASK-003", "plan", "en-diseno",
                   derivados=hostil("tarea", "TASK-003"))
    ok.append(any("TASK-003/tarea.md" in x for x in av) and estado_de("TASK-003") == "en-diseno"
              and "TASK-003/tarea.md" in git(tareas, "show", "--name-only", "--format=", "HEAD"))
    print(f"      estado de TASK-003 tras T8: {estado_de('TASK-003')}")
    av, gen = paso("T9 start TASK-003, derivado que escribe en una ruta ignorada", "TASK-003", "start", "en-curso",
                   derivados=hostil("ignorado"))
    ok.append(any("codigo:.estado/x.txt" in x for x in av) and not os.path.exists(os.path.join(raiz, ".estado", "x.txt")))
    av, gen = paso("T10 review TASK-003, escribe una ruta ignorada del estado", "TASK-003", "review", "en-revision",
                   derivados=hostil("", escribe=("tareas/borrador.tmp",), comando="python -I -B scripts/grafo.py"))
    ok.append(any("ruta ignorada" in x for x in av))
    enlace = os.path.join(tareas, "enlace")
    subprocess.run(["cmd", "/d", "/c", "mklink", "/J", enlace, os.path.join(raiz, "src")], capture_output=True,
                   stdin=subprocess.DEVNULL)
    if os.path.isjunction(enlace):
        av, gen = paso("T11 plan TASK-003, escribe a traves de una union", "TASK-003", "plan", "en-diseno",
                       derivados=hostil("", escribe=("tareas/enlace/x.md",), comando="python -I -B scripts/grafo.py"))
        ok.append(any("a traves de un enlace" in x for x in av) and not os.path.exists(os.path.join(raiz, "src", "x.md")))
        os.rmdir(enlace)
        MEDIDO["escribe a traves de una union de Windows"] = "rechazado"
    else:
        print("  T11 sin hacer: el sistema no deja crear la union")
        MEDIDO["escribe a traves de una union de Windows"] = "no verificado"
    try:
        os.symlink(os.path.join(raiz, "src"), os.path.join(trabajo, "c", "prueba-enlace"), target_is_directory=True)
        os.remove(os.path.join(trabajo, "c", "prueba-enlace"))
        simbolico = "se pueden crear"
    except OSError as e:
        simbolico = f"no verificado (sin privilegios: WinError {getattr(e, 'winerror', '?')})"
    print(f"  enlaces simbolicos en este sistema: {simbolico}; la comprobacion es la misma (islink y realpath)")
    MEDIDO["enlaces simbolicos en escribe"] = simbolico
    arbol1 = git(raiz, "rev-parse", "develop^{tree}").strip()
    ncod1 = git(raiz, "rev-list", "--count", "develop").strip()
    limpio = git(raiz, "status", "--porcelain", "--untracked-files=all", "--ignored").strip() == "!! tareas/"
    nest = git(raiz, "rev-list", "--count", "taskcode/tareas").strip()
    print(f"arbol de develop antes={arbol0[:12]} despues={arbol1[:12]} identico={arbol0 == arbol1} ; "
          f"commits en develop {ncod0}->{ncod1} ; status limpio, tambien lo ignorado={limpio}")
    print(f"commits en taskcode/tareas: {nest} (alta + {len(pasos)} transiciones, uno por transicion)")
    g = open(os.path.join(tareas, "grafo-hitos.md"), encoding="utf-8").read()
    fin_ok = "TASK-001[" not in g and "TASK-003[TASK-003 en-diseno]" in g.split("## H2")[1]
    print(f"grafo-hitos.md final: TASK-001 fuera (terminada)={'TASK-001[' not in g} ; "
          f"TASK-003 en H2={'TASK-003[TASK-003 en-diseno]' in g.split('## H2')[1]}")
    for i, (nombre, *_r) in enumerate(pasos):
        if not ok[i]:
            falla("C", f"{nombre}: resultado inesperado")
    if not (arbol0 == arbol1 and ncod0 == ncod1 and limpio and nest == str(1 + len(pasos)) and fin_ok):
        falla("C", "develop cambio, la rama de estado no tiene un commit por transicion o el grafo final no cuadra")
    MEDIDO["transiciones con derivados en la rama de estado"] = str(len(pasos))
    MEDIDO["hostiles en ejecucion descartados"] = f"{sum(ok[6:])}/{len(ok[6:])}"
    MEDIDO["arbol de develop tras las transiciones"] = "identico" if arbol0 == arbol1 else "distinto"
    print(f"RESULTADO C transiciones={len(pasos)} ; como se espera={sum(ok)}/{len(ok)} ; develop identico="
          f"{arbol0 == arbol1}")


# ---------------------------------------------------------------------------
# Tareas de los pilotos en la estructura de la 0.16 y en la carpeta fija, y `tareas --json` simulado
# ---------------------------------------------------------------------------

def copiar_tareas(pil, clave, base):
    """Escribe base/v016/tareas/<NN-estado>/TASK-NNN/tarea.md y base/v20/tareas/TASK-NNN/tarea.md (las de
    05-archivadas, con estado descartada: lo que haria la migracion de D1a). Devuelve las rutas leidas."""
    rutas = [p for p in pil.ls(clave, "tareas") if re.fullmatch(r"tareas/0\d-[^/]+/TASK-\d+/tarea\.md", p)]
    for p in rutas:
        _t, carpeta, tid, _f = p.split("/")
        txt = pil.show(clave, p)
        for capa, rel in (("v016", (carpeta, tid)), ("v20", (tid,))):
            dst = os.path.join(base, capa, "tareas", *rel)
            os.makedirs(dst, exist_ok=True)
            t2 = re.sub(r"^estado: .*$", "estado: descartada", txt, count=1, flags=re.M) \
                if capa == "v20" and carpeta == "05-archivadas" else txt
            open(os.path.join(dst, "tarea.md"), "w", encoding="utf-8", newline="").write(t2)
    return rutas


def tareas_json_simulado(raiz_tareas):
    """Simula `taskctl tareas --json` sobre la carpeta fija. Campos (revision 1, IMP-3): todo el frontmatter
    leido con YAML estandar, `criterios` (las lineas `- [ ]`/`- [x]` de «## Criterios de aceptacion») y
    `texto` (tarea.md entero), que es lo que leen los consumidores reales de los pilotos."""
    out = []
    for d in sorted(os.listdir(raiz_tareas)):
        p = os.path.join(raiz_tareas, d, "tarea.md")
        if not os.path.isfile(p):
            continue
        texto = open(p, encoding="utf-8").read()
        cab, cuerpo = texto.split("\n---\n", 1)
        fm = yaml.safe_load(cab[4:])
        criterios, dentro = [], False
        for linea in cuerpo.splitlines():
            if linea.startswith("## "):
                dentro = linea.strip().lower().startswith("## criterios de aceptacion")
            elif dentro and re.match(r"^- \[[ xX]\] \S", linea):
                criterios.append(linea)
        out.append({**fm, "criterios": criterios, "texto": texto})
    return json.dumps(out, ensure_ascii=False, default=str)


# Consumidores con `tareas --json`: lo que sustituye a los lectores propios de los pilotos.

def modulos_desde_json(js):
    """comprobar-modulos.py de OpenDockTrading: {id: texto} de las vivas."""
    return {t["id"]: t["texto"] for t in js if t["estado"] != "descartada"}


def fallos_desde_json(js):
    """comprobar-fallos.py de OpenDockTrading: {id: casillas de criterio} de las vivas."""
    return {t["id"]: len(re.findall(r"^- \[[ x]\] ", t["texto"], re.M)) for t in js if t["estado"] != "descartada"}


def grafo_desde_json(js):
    """grafo-tareas.py de ToolsaWeb: frontmatter como texto (como su lector) y numero de criterios."""
    plano = lambda v: "null" if v is None else str(v).lower() if isinstance(v, bool) else str(v)
    return {t["id"]: {**{k: [plano(x) for x in v] if isinstance(v, list) else plano(v) for k, v in t.items()
                         if k not in ("criterios", "texto")}, "criterios": len(t["criterios"])} for t in js}


# ---------------------------------------------------------------------------
# C2: hitos de ToolsaWeb con la disposicion nueva (funciones reales del script copiado)
# ---------------------------------------------------------------------------

def quitar_generado(texto, m):
    """hitos.md sin lo generado: las tablas de §1 (duplican el frontmatter) y el bloque del Mermaid de §4."""
    i, f = texto.find("\n## 1."), texto.find("\n## 2.")
    sec = "\n".join(l for l in texto[i:f].split("\n") if not l.lstrip().startswith("|"))
    sec = re.sub(r"\n{3,}", "\n\n", sec)
    a, b = texto.find(m.INICIO), texto.find(m.FIN) + len(m.FIN)
    resto = texto[f:a] + "Las tablas de tareas por hito y el grafo se generan en la rama de estado (`tareas/hitos.md`)." \
        + texto[b:]
    return texto[:i] + sec + resto


def generar_hitos(m, tareas, titulos):
    """Derivado `genera` de ToolsaWeb en la 2.0: tablas de §1 y Mermaid, en la rama de estado."""
    lineas = ["# Hitos: tablas y grafo (generado en la rama de estado; no editar)", "",
              "## 1. Tareas pendientes por hito", ""]
    por_hito = {}
    for t in sorted(m.pendientes(tareas)):
        por_hito.setdefault(int(tareas[t].get("sprint", 0)), []).append(t)
    for h in sorted(por_hito, key=m.rango):
        lineas += [f"### H{h} — {titulos.get(h, f'H{h}')}", "", "| ID | Tarea | Complejidad | Depende de |",
                   "|---|---|---|---|"]
        for t in por_hito[h]:
            deps = ", ".join(tareas[t].get("dependencias", [])) or "—"
            lineas.append(f"| {t} | {tareas[t].get('titulo', '').replace('|', '/')} | "
                          f"{tareas[t].get('complejidad', '')} | {deps} |")
        lineas.append("")
    lineas += ["## 2. Grafo de dependencias", "", m.mermaid(tareas, titulos), ""]
    return "\n".join(lineas)


def seccion_c2(pil, trabajo):
    sep("C2: hitos de ToolsaWeb, hoy y con la disposicion nueva (script real copiado)")
    base = os.path.join(trabajo, "c2")
    copiar_tareas(pil, "TW", base)
    texto = pil.show("TW", "docs/planificacion/hitos.md")
    for capa in ("v016", "v20"):
        os.makedirs(os.path.join(base, capa, "scripts"))
        os.makedirs(os.path.join(base, capa, "docs", "planificacion"))
        open(os.path.join(base, capa, "scripts", "grafo-tareas.py"), "w", encoding="utf-8", newline="").write(
            pil.show("TW", "scripts/grafo-tareas.py"))
    hoy = os.path.join(base, "v016")
    open(os.path.join(hoy, "docs", "planificacion", "hitos.md"), "w", encoding="utf-8", newline="").write(texto)
    open(os.path.join(hoy, "tareas", "ROADMAP.md"), "w", encoding="utf-8", newline="").write(
        pil.show("TW", "tareas/ROADMAP.md"))

    def cli(arg):
        env = {k: v for k, v in os.environ.items() if not k.startswith("TW_")}
        r = subprocess.run([sys.executable, "-I", "-B", "-X", "utf8", os.path.join(hoy, "scripts", "grafo-tareas.py"),
                            arg],
                           cwd=hoy, env=env, capture_output=True, stdin=subprocess.DEVNULL, timeout=300)
        return r.returncode, [l for l in r.stdout.decode("utf-8", "replace").splitlines() if l.startswith("[")]

    c0, s0 = cli("--comprobar")
    mh = cargar_modulo("grafo_c2_hoy", os.path.join(hoy, "scripts", "grafo-tareas.py"))
    th = mh.cargar(mh.TAREAS)
    elegida = next(t for t, fm in sorted(th.items()) if fm.get("estado") == "planificada"
                   and "descartada" not in fm.get("etiquetas", [])
                   and all(th.get(d, {}).get("estado") == "terminada" for d in fm.get("dependencias", [])))
    carpeta = next(c for c in os.listdir(os.path.join(hoy, "tareas")) if os.path.isdir(os.path.join(hoy, "tareas", c, elegida)))

    def terminar(p):
        t = open(p, encoding="utf-8", newline="").read()
        open(p, "w", encoding="utf-8", newline="").write(re.sub(r"^estado: .*$", "estado: terminada", t, count=1, flags=re.M))

    terminar(os.path.join(hoy, "tareas", carpeta, elegida, "tarea.md"))
    c1, s1 = cli("--comprobar")
    print(f"-- hoy (0.16, hitos.md en docs/): transicion simulada = finish de {elegida} (lista para empezar)")
    print(f"  --comprobar antes: codigo {c0} ; {s0[-1][:90] if s0 else ''}")
    print(f"  --comprobar despues, sin poder regenerar docs/ (D2a): codigo {c1} ; errores={len(s1)}")
    for l in s1[:3]:
        print(f"    {l[:120]}")
    nueva = os.path.join(base, "v20")
    m = cargar_modulo("grafo_c2_nueva", os.path.join(nueva, "scripts", "grafo-tareas.py"))
    manual = quitar_generado(texto.replace("\r\n", "\n"), m)
    open(os.path.join(nueva, "docs", "planificacion", "hitos.md"), "w", encoding="utf-8", newline="\n").write(manual)
    n0 = len(texto.replace("\r\n", "\n").rstrip("\n").split("\n"))
    n1 = len(manual.rstrip("\n").split("\n"))
    titulos = m.titulos_de_hitos(manual)
    m.fijar_rango(titulos)
    generado = os.path.join(nueva, "tareas", "hitos.md")

    def ronda():
        tareas = grafo_desde_json(json.loads(tareas_json_simulado(os.path.join(nueva, "tareas"))))
        g = generar_hitos(m, tareas, titulos)                       # el derivado genera
        open(generado, "w", encoding="utf-8", newline="\n").write(g)
        comprueba = m.errores(tareas) + m.errores_de_titulos(tareas, titulos)   # el comprueba codigo<->estado
        tablas = m.errores_de_tablas(tareas, m.tablas_de_hitos(g))  # coherencia de lo generado
        return comprueba, tablas, hashlib.sha256(g.encode()).hexdigest()

    sha_docs = hashlib.sha256(manual.encode()).hexdigest()
    e0, t0, g0 = ronda()
    terminar(os.path.join(nueva, "tareas", elegida, "tarea.md"))
    e1, t1, g1 = ronda()
    docs_igual = hashlib.sha256(open(os.path.join(nueva, "docs", "planificacion", "hitos.md"), encoding="utf-8")
                                .read().encode()).hexdigest() == sha_docs
    print("-- disposicion nueva: en docs/ solo el texto a mano; tablas y Mermaid en tareas/hitos.md (genera); "
          "titulos de docs/ frente al estado (comprueba)")
    print(f"  docs/planificacion/hitos.md: {n0} lineas hoy ; {n1} a mano ; {n0 - n1} generadas salen a la rama de estado")
    print(f"  antes de la transicion: comprueba={len(e0)} errores ; tablas generadas incoherentes={len(t0)}")
    print(f"  tras el finish de {elegida}: comprueba={len(e1)} errores ; tablas incoherentes={len(t1)} ; "
          f"tareas/hitos.md regenerado={g0 != g1} ; docs/ sin tocar={docs_igual}")
    if c0 != 0 or c1 == 0 or e0 or t0 or e1 or t1 or g0 == g1 or not docs_igual:
        falla("C2", "la disposicion nueva de hitos no se comporta como se espera")
    MEDIDO["comprobar de ToolsaWeb tras una transicion, hitos en docs (hoy)"] = f"codigo {c1}"
    MEDIDO["comprueba de ToolsaWeb tras una transicion, disposicion nueva"] = f"{len(e1)} errores"
    MEDIDO["lineas generadas que salen de docs/planificacion/hitos.md"] = f"{n0 - n1} de {n0}"
    print(f"RESULTADO C2 hoy={c0}->{c1} ; nueva={len(e0)}->{len(e1)} errores ; docs sin tocar={docs_igual}")


# ---------------------------------------------------------------------------
# D: lectura del estado
# ---------------------------------------------------------------------------

def cargar_modulo(nombre, ruta):
    spec = importlib.util.spec_from_file_location(nombre, ruta)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def lineas_funciones(fuente, nombres):
    arbol = ast.parse(fuente)
    return {n.name: n.end_lineno - n.lineno + 1 for n in arbol.body
            if isinstance(n, ast.FunctionDef) and n.name in nombres}


def seccion_d(pil, trabajo):
    sep("D: lectura del estado (tres consumidores reales copiados frente a `tareas --json` simulado)")
    import inspect
    resultados = {}
    # OpenDockTrading: lector-tareas.py + sus dos consumidores
    base = os.path.join(trabajo, "d", "ODT")
    rutas = copiar_tareas(pil, "ODT", base)
    scripts = ("lector-tareas.py", "comprobar-modulos.py", "comprobar-fallos.py")
    for capa in ("v016", "v20"):
        os.makedirs(os.path.join(base, capa, "scripts"))
        for s in scripts:
            open(os.path.join(base, capa, "scripts", s), "w", encoding="utf-8", newline="").write(
                pil.show("ODT", f"scripts/{s}"))
    js = json.loads(tareas_json_simulado(os.path.join(base, "v20", "tareas")))
    for nombre, mod, desde_json in (("comprobar-modulos.py", "cm", modulos_desde_json),
                                    ("comprobar-fallos.py", "cf", fallos_desde_json)):
        hoy = cargar_modulo(f"{mod}_v016", os.path.join(base, "v016", "scripts", nombre)).tareas_vivas()
        fijo = cargar_modulo(f"{mod}_v20", os.path.join(base, "v20", "scripts", nombre)).tareas_vivas()
        nuevo = desde_json(js)
        resultados[nombre] = (hoy == nuevo and len(fijo) == 0 and len(hoy) > 0)
        print(f"  OpenDockTrading {nombre}: tareas.md={len(rutas)} ; tareas_vivas() hoy={len(hoy)} ; en carpeta fija="
              f"{len(fijo)} ; desde tareas --json={len(nuevo)} ; mismo resultado (ids y valores)={hoy == nuevo}")
        MEDIDO[f"OpenDockTrading {nombre}: tareas que ve en la carpeta fija"] = str(len(fijo))
    # ToolsaWeb: grafo-tareas.py
    base = os.path.join(trabajo, "d", "TW")
    rutas = copiar_tareas(pil, "TW", base)
    for capa in ("v016", "v20"):
        os.makedirs(os.path.join(base, capa, "scripts"))
        open(os.path.join(base, capa, "scripts", "grafo-tareas.py"), "w", encoding="utf-8", newline="").write(
            pil.show("TW", "scripts/grafo-tareas.py"))
    mh = cargar_modulo("gt_v016", os.path.join(base, "v016", "scripts", "grafo-tareas.py"))
    mf = cargar_modulo("gt_v20", os.path.join(base, "v20", "scripts", "grafo-tareas.py"))
    titulos = mh.titulos_de_hitos(pil.show("TW", "docs/planificacion/hitos.md"))
    mh.fijar_rango(titulos)
    th, tf = mh.cargar(mh.TAREAS), mf.cargar(mf.TAREAS)
    tj = grafo_desde_json(json.loads(tareas_json_simulado(os.path.join(base, "v20", "tareas"))))

    def salida(t):
        return (mh.errores(t), mh.mermaid(t, titulos), mh.roadmap(t, titulos))
    igual = salida(th) == salida(tj)
    resultados["grafo-tareas.py"] = igual and len(tf) == 0
    print(f"  ToolsaWeb grafo-tareas.py: tareas.md={len(rutas)} ; cargar() hoy={len(th)} ; en carpeta fija={len(tf)} ; "
          f"desde tareas --json={len(tj)} ; errores, Mermaid y roadmap iguales a hoy={igual}")
    MEDIDO["ToolsaWeb grafo-tareas.py: tareas que ve en la carpeta fija"] = str(len(tf))
    for k, v in resultados.items():
        if not v:
            falla("D", f"{k}: lectura del estado inesperada")
    MEDIDO["consumidores reales con el mismo resultado desde tareas --json (interfaz simulada)"] = \
        f"{sum(resultados.values())}/{len(resultados)}"
    f_lector = len(pil.show("ODT", "scripts/lector-tareas.py").replace("\r\n", "\n").rstrip("\n").split("\n"))
    f_cm = lineas_funciones(pil.show("ODT", "scripts/comprobar-modulos.py"), {"_cargar_grafo", "tareas_vivas"})
    f_cf = lineas_funciones(pil.show("ODT", "scripts/comprobar-fallos.py"), {"_cargar_grafo", "tareas_vivas"})
    f_tw = lineas_funciones(pil.show("TW", "scripts/grafo-tareas.py"), {"quitar_comentario", "leer_tarea", "cargar"})
    hoy_l = f_lector + sum(f_cm.values()) + sum(f_cf.values()) + sum(f_tw.values())
    nuevo_l = sum(len(inspect.getsource(f).rstrip("\n").split("\n"))
                  for f in (modulos_desde_json, fallos_desde_json, grafo_desde_json))
    print(f"  lectura propia hoy: lector-tareas.py={f_lector} ; comprobar-modulos.py {f_cm} ; comprobar-fallos.py {f_cf} ; "
          f"grafo-tareas.py {f_tw} ; total={hoy_l}")
    print(f"  consumidores de tareas --json (modulos_desde_json, fallos_desde_json, grafo_desde_json): {nuevo_l} lineas")
    MEDIDO["lineas de lectura propia que sustituye tareas --json (interfaz simulada)"] = f"{hoy_l} -> {nuevo_l}"
    f_rm = lineas_funciones(pil.show("TW", "scripts/grafo-tareas.py"), {"mermaid", "orden_de_trabajo", "roadmap"})
    print(f"  ToolsaWeb grafo-tareas.py que solapa con el roadmap nativo (F1-1 dec. 3, F3-4) {f_rm} = "
          f"{sum(f_rm.values())} (no se cuenta como ahorro: falta ver que F3-4 cubra su orden)")
    lineas_bat = pil.show("ODT", "scripts/verificar.bat").replace("\r\n", "\n").split("\n")
    roadmap_paso = [i + 1 for i, l in enumerate(lineas_bat) if "taskctl roadmap --comparar" in l]
    print(f"  paso de OpenDockTrading que desaparece con el roadmap fuera del arbol (F1-7): verificar.bat:{roadmap_paso}")
    MEDIDO["pasos de verificacion que desaparecen (OpenDockTrading)"] = str(len(roadmap_paso))
    print(f"RESULTADO D consumidores iguales={sum(resultados.values())}/{len(resultados)}")


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

PROYECTADAS = {
    "lineas de lectura propia que sustituye tareas --json (interfaz simulada)",
    "consumidores reales con el mismo resultado desde tareas --json (interfaz simulada)",
}


def comparar_ventaja(adr):
    sep("Tabla `ventaja` del ADR frente a lo medido, y cifras del resumen")
    cab, filas = entre_marcadores(adr, "ventaja")
    if cab[:3] != ["Cifra", "Valor", "Tipo"]:
        raise SystemExit("ERROR: la tabla ventaja tiene que empezar por Cifra | Valor | Tipo")
    vistas, valores = set(), set()
    for f in filas:
        cifra, valor, tipo = f[0], f[1].strip("`"), f[2]
        vistas.add(cifra)
        valores.add(valor)
        med = MEDIDO.get(cifra)
        esperado = "proyectado" if cifra in PROYECTADAS else "medido"
        print(f"  {cifra:84} ADR={valor!r:16} medido={med!r} {tipo} {'OK' if med == valor and tipo == esperado else 'DISTINTO'}")
        if med != valor:
            falla("ventaja", f"{cifra}: el ADR dice {valor!r} y lo medido es {med!r}")
        if tipo != esperado:
            falla("ventaja", f"{cifra}: tipo {tipo!r}, se espera {esperado!r}")
    for k in sorted(set(MEDIDO) - vistas):
        print(f"  (medido y no citado en el ADR) {k} = {MEDIDO[k]!r}")
        falla("ventaja", f"cifra medida que no esta en la tabla: {k}")
    m = re.search(r"<!-- resumen:inicio -->\n(.*?)<!-- resumen:fin -->", adr, re.S)
    if not m:
        raise SystemExit("ERROR: faltan los marcadores resumen")
    negritas = re.findall(r"\*\*([^*]+)\*\*", m.group(1))
    fuera = re.findall(r"\d+", re.sub(r"\*\*[^*]+\*\*", "", m.group(1)))
    malas = [x for x in negritas if x not in valores]
    print(f"  resumen: {len(negritas)} cifras en negrita, todas en la tabla={not malas} ; numeros fuera de negrita={len(fuera)}")
    if malas or fuera:
        falla("resumen", f"cifras que no estan en la tabla {malas} o numeros sin negrita {fuera}")


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
    seccion_a2(pil, adr)
    seccion_b(pil, plugin, trabajo)
    seccion_c(trabajo)
    seccion_c2(pil, trabajo)
    seccion_d(pil, trabajo)
    seccion_e(pil, trabajo)
    comparar_ventaja(adr)
    sep("RESUMEN")
    if FALLOS:
        print(f"FALLOS ({len(FALLOS)}):")
        for f in FALLOS:
            print(f"  {f}")
        sys.exit(1)
    print("todo como se espera: A, A2, B, C, C2, D, E, la tabla de ventaja y el resumen")


if __name__ == "__main__":
    main()

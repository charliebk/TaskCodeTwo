"""PoC de F1-12 (TASK-013): actualizacion de proyectos 0.16 -> 2.0 y convivencia.

Se ejecuta con `python -I -B` (sin el directorio del guion ni PYTHON* en
sys.path, sin __pycache__). OpenDockTrading y ToolsaWeb SOLO SE LEEN: de
OpenDockTrading se hace `git clone --no-hardlinks` a la carpeta de trabajo
y todo se ejecuta sobre esas copias, nunca con el cwd en el repo original.
El `taskctl` de la 0.16 y sus hooks (por stdin) solo se lanzan con el cwd en
una copia. No toca ~/.claude/settings.json ni instala nada. Las consultas de
nombres a npm, Homebrew y winget son de solo lectura.

(N) Nombres: `where` en Windows y `command -v` en Linux (WSL) de los
    candidatos y de `taskctl`; npm, Homebrew (API de formulae.brew.sh) y winget.
(M) Migracion de una copia de OpenDockTrading en el commit fijo: el plan como
    dato (`planificar`, dos llamadas, mismo hash; `--simular` no cambia ningun
    byte), negativa con huecos bloqueantes, `git subtree split -P tareas`,
    aplanado y formato 2 (F1-9) en la rama de estado montada en `tareas/`,
    config a los bloques de F1-11, criterio de salida estructural, push solo
    con el criterio en verde, segunda pasada con plan vacio, y vuelta atras
    automatica con el criterio roto sobre una segunda copia que reanuda una
    migracion a medias.
(C) Convivencia: board, doctor, estado --stdin, roadmap --escribir y new de la
    0.16 y sus hooks con stdin sobre la copia migrada, con cuatro variantes de la
    config formato 2 (una es control sin centinela); busca un fallo cerrado de la 0.16 para D2.
(A) Informe de adopcion de OpenDockTrading: uso de campos (F1-9), parches y
    propio de F1-11 con fichero:lineas recalculados en el commit fijo, y lo
    que nombra TaskCode fuera de tareas/ sin clasificar.
(G) El ADR: cada cifra de la tabla `cifras` es la medida y toda cifra medida
    esta en la tabla; los numeros en negrita del `resumen` son valores de la
    tabla; secciones de la plantilla; IDs citados que existen; ningun enlace a
    tareas/ ni a otros repos; fila en el indice; y ocho copias mutadas del ADR que
    tienen que fallar cada una por su motivo.

Uso: python -I -B poc-f1-12.py <trabajo> <repo-TaskCodeTwo> <carpeta-Personal> <plugin-0.16.1> [--cache-split <dir>]
Sale con 0 solo si todo sale como se espera. El log (stdout) es determinista
salvo las consultas de red, que dependen de npm, Homebrew y winget; los
tiempos van a stderr.
"""
import hashlib
import importlib.util
import json
import os
import re
import shutil
import stat
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

import yaml

FIJO_ODT = "8141a08a703ef1b549bdf4b6275adbac0c68e7cc"   # develop de OpenDockTrading, 2026-11-14
FECHA_GIT = "2026-11-14T12:00:00+00:00"
NOMBRE = {"cli": "tc2", "plugin": "taskcodetwo", "dir_git": "taskcodetwo",
          "rama_estado": "taskcodetwo/tareas", "rama_actualizar": "actualizar/formato-2", "npm": "taskcodetwo"}
CANDIDATOS = ("tc", "tc2", "tctwo", "taskcodetwo")
CARPETA_ESTADO = {"00-planificadas": "planificada", "01-en-diseno": "en-diseno", "02-en-curso": "en-curso",
                  "03-en-revision": "en-revision", "04-terminadas": "terminada", "05-archivadas": "descartada"}
ACTIVOS = ("en-diseno", "en-curso", "en-revision")
FALLOS = []
MEDIDO = {}
T0 = time.time()


def falla(seccion, texto):
    FALLOS.append(f"{seccion}: {texto}")
    print(f"FALLO en {seccion}: {texto}")


def sep(t):
    print()
    print(f"=== {t} ===")


def tiempo(texto):
    print(f"[tiempo] {texto} ({time.time() - T0:.0f} s desde el inicio)", file=sys.stderr, flush=True)


def run(argv, cwd=None, env=None, entrada=None, check=True, timeout=1800):
    r = subprocess.run(argv, cwd=cwd, env=env, capture_output=True, check=False,
                       input=entrada if entrada is not None else None,
                       stdin=None if entrada is not None else subprocess.DEVNULL, timeout=timeout)
    if check and r.returncode != 0:
        raise SystemExit(f"ERROR: {' '.join(str(a) for a in argv[:5])}...: "
                         f"{r.stderr.decode('utf-8', 'replace').strip()[:600]}")
    return r


def env_git():
    e = dict(os.environ)
    e.update({"GIT_AUTHOR_NAME": "poc", "GIT_AUTHOR_EMAIL": "poc@ejemplo.invalid",
              "GIT_COMMITTER_NAME": "poc", "GIT_COMMITTER_EMAIL": "poc@ejemplo.invalid",
              "GIT_AUTHOR_DATE": FECHA_GIT, "GIT_COMMITTER_DATE": FECHA_GIT,
              "GIT_CONFIG_NOSYSTEM": "1", "GIT_TERMINAL_PROMPT": "0"})
    return e


def git(repo, *args, check=True, entrada=None, binario=False):
    r = run(["git", "-C", repo, "-c", "core.quotepath=false", "-c", "core.autocrlf=false",
             "-c", "core.longpaths=true", "-c", "commit.gpgsign=false", *args],
            env=env_git(), entrada=entrada, check=check)
    if not check:
        return r
    return r.stdout if binario else r.stdout.decode("utf-8", "replace")


def borrar(ruta):
    def quitar_ro(f, p, _):
        os.chmod(p, stat.S_IWRITE)
        f(p)
    if os.path.lexists(ruta):
        shutil.rmtree(ruta, onerror=quitar_ro)


def hash_dir(raiz, con_git=True):
    """{ruta relativa: sha256} de todos los ficheros (con .git si con_git)."""
    out = {}
    for d, subs, fs in os.walk(raiz):
        subs.sort()
        rel_d = os.path.relpath(d, raiz).replace("\\", "/")
        if not con_git and (rel_d == ".git" or rel_d.startswith(".git/")):
            subs[:] = []
            continue
        for f in sorted(fs):
            p = os.path.join(d, f)
            rel = os.path.relpath(p, raiz).replace("\\", "/")
            if not con_git and (rel == ".git" or rel.startswith(".git/")):
                continue
            if not con_git and "/.git" in "/" + rel and rel.split("/")[-1] == ".git":
                continue      # el fichero .git del worktree montado
            with open(p, "rb") as fh:
                out[rel] = hashlib.sha256(fh.read()).hexdigest()
    return out


def diferencias(a, b):
    return sorted(set(k for k in set(a) | set(b) if a.get(k) != b.get(k)))


def resumir_rutas(rutas, maximo=6):
    objetos = [r for r in rutas if r.startswith(".git/objects/")]
    resto = [r for r in rutas if not r.startswith(".git/objects/")]
    txt = ", ".join(resto[:maximo]) + (f" (+{len(resto) - maximo})" if len(resto) > maximo else "")
    if objetos:
        txt += (", " if txt else "") + f"objetos git: {len(objetos)}"
    return txt or "ninguno"


def sha_texto(t):
    return hashlib.sha256(t.encode("utf-8")).hexdigest()


# ---------------------------------------------------------------------------
# N: nombres
# ---------------------------------------------------------------------------

def donde_windows(n):
    r = run(["where", n], check=False)
    if r.returncode != 0:
        return []
    return [x.strip() for x in r.stdout.decode("utf-8", "replace").splitlines() if x.strip()]


def describir_rutas(rutas):
    if not rutas:
        return "ninguna"
    if all("taskcode-plugin" in r.replace("\\", "/") and "/0.16.1/" in r.replace("\\", "/") for r in rutas):
        return f"{len(rutas)} (bin/ del plugin 0.16.1)"
    casa = os.path.expanduser("~")
    return "; ".join(r.replace(casa, "~") for r in rutas)


def linux_command_v(nombres):
    """command -v en la distro de WSL disponible; None si no hay."""
    guion = "; ".join(f"if command -v {n} >/dev/null 2>&1; then echo {n}=si; else echo {n}=no; fi" for n in nombres)
    try:
        r = run(["wsl", "-d", "docker-desktop", "--", "sh", "-c", guion], check=False, timeout=60)
    except Exception:
        return None
    if r.returncode != 0:
        return None
    out = r.stdout.replace(b"\x00", b"").decode("utf-8", "replace")
    res = dict(re.findall(r"(\w+)=(si|no)", out))
    return res if all(n in res for n in nombres) else None


def npm_estado(n):
    npm = shutil.which("npm.cmd") or shutil.which("npm")
    if not npm:
        return "no verificado (sin npm)"
    r = run([npm, "view", n, "name", "bin", "--json"], check=False, timeout=60)
    err = r.stderr.decode("utf-8", "replace") + r.stdout.decode("utf-8", "replace")
    if r.returncode != 0 and "E404" in err:
        return "libre"
    if r.returncode != 0:
        return "no verificado"
    try:
        d = json.loads(r.stdout.decode("utf-8") or "{}")
    except ValueError:
        return "no verificado"
    bins = sorted((d.get("bin") or {}).keys()) if isinstance(d, dict) else []
    return f"ocupado (bin {', '.join(bins)})" if bins else "ocupado (sin bin)"


def http_codigo(url):
    for intento in range(4):
        try:
            with urllib.request.urlopen(urllib.request.Request(url, method="GET"), timeout=20) as r:
                return r.status
        except urllib.error.HTTPError as e:
            if e.code < 500:
                return e.code
        except Exception:
            pass
        time.sleep(2 + intento * 2)
    return None


def brew_estado(n):
    f = http_codigo(f"https://formulae.brew.sh/api/formula/{n}.json")
    c = http_codigo(f"https://formulae.brew.sh/api/cask/{n}.json")
    if f == 200 or c == 200:
        return "ocupado"
    if f == 404 and c == 404:
        return "libre"
    return "no verificado"


def winget_estado(n):
    w = shutil.which("winget")
    if not w:
        return "no verificado (sin winget)"
    ocupado = False
    for campo in ("--moniker", "--name", "--id"):
        r = run([w, "search", campo, n, "--exact", "--source", "winget", "--disable-interactivity"],
                check=False, timeout=120)
        out = r.stdout.decode("utf-8", "replace")
        if "No package found" in out or "No se encontr" in out:
            continue
        if r.returncode != 0:
            return "no verificado"
        if re.search(r"^-{5,}", out, re.M):
            ocupado = True
    return "ocupado" if ocupado else "libre"


def seccion_n():
    sep("N: nombres (PATH de Windows y Linux, npm, Homebrew, winget)")
    rutas = {}
    for n in CANDIDATOS + ("taskctl",):
        rutas[n] = donde_windows(n)
        d = describir_rutas(rutas[n])
        print(f"PATH Windows (where {n}): {d}")
        MEDIDO[f"PATH Windows: {n}"] = d
    lin = linux_command_v(CANDIDATOS + ("taskctl",))
    for n in CANDIDATOS + ("taskctl",):
        v = "no verificado" if lin is None else ("ninguna" if lin[n] == "no" else
                                                  ("existe (PATH de Windows por la interoperabilidad de WSL)"
                                                   if rutas[n] else "existe"))
        print(f"PATH Linux, WSL docker-desktop (command -v {n}): {v}")
        MEDIDO[f"PATH Linux (WSL docker-desktop): {n}"] = v
    for n in CANDIDATOS:
        a, b, c = npm_estado(n), brew_estado(n), winget_estado(n)
        print(f"{n}: npm={a} ; Homebrew={b} ; winget={c}")
        MEDIDO[f"npm: {n}"] = a
        MEDIDO[f"Homebrew: {n}"] = b
        MEDIDO[f"winget: {n}"] = c
    # Comprobaciones de la decision D1a
    if rutas["tc2"]:
        falla("N", f"tc2 ya esta en el PATH: {rutas['tc2']}")
    if not rutas["taskctl"]:
        falla("N", "taskctl de la 0.16 no esta en el PATH: la convivencia no se puede medir")
    if set(rutas["tc2"]) & set(rutas["taskctl"]):
        falla("N", "tc2 y taskctl comparten ruta")
    print(f"RESULTADO N: CLI elegido {NOMBRE['cli']} sin ruta en Windows; taskctl en {describir_rutas(rutas['taskctl'])}; "
          f"paquete npm previsto {NOMBRE['npm']} con bin {NOMBRE['cli']}")


# ---------------------------------------------------------------------------
# Tareas de la 0.16 y del formato 2
# ---------------------------------------------------------------------------

RX_FM = re.compile(r"\A---\r?\n(.*?\r?\n)---\r?\n", re.S)


def partir(texto):
    m = RX_FM.match(texto)
    if not m:
        raise ValueError("sin frontmatter")
    return m.group(1), texto[m.end():]


def cat_files(repo, rev, rutas):
    """{ruta: texto} con un solo `git cat-file --batch`."""
    if not rutas:
        return {}
    entrada = "".join(f"{rev}:{r}\n" for r in rutas).encode("utf-8")
    out = git(repo, "cat-file", "--batch", entrada=entrada, binario=True)
    res, i = {}, 0
    for r in rutas:
        fin = out.index(b"\n", i)
        cab = out[i:fin].decode("utf-8")
        if cab.endswith(" missing"):
            raise SystemExit(f"ERROR: falta {rev}:{r}")
        tam = int(cab.split()[2])
        res[r] = out[fin + 1:fin + 1 + tam].decode("utf-8")
        i = fin + 1 + tam + 1
    return res


def tareas_016(repo, rev):
    """{id: {carpeta, ruta, texto, fm}} de tareas/NN-estado/TASK-NNN/tarea.md en rev."""
    rutas = [r for r in git(repo, "ls-tree", "-r", "--name-only", rev, "--", "tareas/").splitlines()
             if re.match(r"^tareas/0\d-[a-z-]+/TASK-\d+[a-z]?/tarea\.md$", r)]
    textos = cat_files(repo, rev, rutas)
    out = {}
    for r in rutas:
        carpeta, tid = r.split("/")[1:3]
        fm = yaml.safe_load(partir(textos[r])[0])
        out[tid] = {"carpeta": carpeta, "ruta": r, "texto": textos[r], "fm": fm}
    return out


def tareas_f2(repo, rev):
    rutas = [r for r in git(repo, "ls-tree", "-r", "--name-only", rev).splitlines()
             if re.match(r"^TASK-\d+[a-z]?/tarea\.md$", r)]
    textos = cat_files(repo, rev, rutas)
    out, errores = {}, []
    for r in rutas:
        try:
            fm = yaml.safe_load(partir(textos[r])[0])
        except Exception as e:      # noqa: BLE001 - se cuenta como no YAML
            errores.append(f"{r}: {type(e).__name__}")
            continue
        out[r.split("/")[0]] = {"ruta": r, "texto": textos[r], "fm": fm}
    return out, errores


def instantanea(tareas, estado_de):
    return {tid: {"titulo": t["fm"].get("titulo"), "estado": estado_de(t),
                  "dependencias": list(t["fm"].get("dependencias") or []),
                  "sprint": t["fm"].get("sprint"), "etiquetas": list(t["fm"].get("etiquetas") or [])}
            for tid, t in tareas.items()}


# ---------------------------------------------------------------------------
# Formato 2 (F1-9)
# ---------------------------------------------------------------------------

def q(v):
    """Escalar YAML segun el contrato de F1-9: cadenas entre comillas dobles."""
    if v is None:
        return "null"
    if v is True:
        return "true"
    if v is False:
        return "false"
    if isinstance(v, int):
        return str(v)
    s = str(v)
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


def lista(v):
    return "[" + ", ".join(q(x) for x in (v or [])) + "]"


def motivo_descartada(cuerpo):
    m = re.search(r"^## Descartada[^\n]*\n+(.*?)(?:\n\s*\n|\n## |\Z)", cuerpo, re.S | re.M)
    if not m:
        return None
    return " ".join(x.strip() for x in m.group(1).strip().splitlines())


def a_formato2(t):
    """(texto nuevo de tarea.md, lista de cambios) para una tarea de la 0.16."""
    fm, cuerpo = t["fm"], partir(t["texto"])[1]
    estado = CARPETA_ESTADO[t["carpeta"]]
    cambios = ["+ formato: 2"]
    if fm.get("estado") != estado:
        cambios.append(f"estado: {fm.get('estado')} -> {estado}")
    motivo = None
    if estado == "descartada":
        motivo = motivo_descartada(cuerpo)
        cambios.append("motivo: de ## Descartada" if motivo else "motivo: a pedir")
    conocidas = {"id", "titulo", "tipo", "sprint", "etiquetas", "complejidad", "modelo_sugerido", "estado",
                 "plan_aprobado", "rama", "asignado_a", "agente_revisor", "skills_recomendados",
                 "regla_seleccion_skill", "ultimo_commit_revisado", "revision_codex", "tokens_diseno",
                 "tokens_implementacion", "tokens_revision", "creado", "actualizado", "dependencias"}
    for k in ("revision_codex", "ultimo_commit_revisado"):
        if k in fm:
            cambios.append(f"- {k}")
    cambios.append("modelo_sugerido, agente_revisor, skills_recomendados, regla_seleccion_skill -> roles")
    desconocidas = [k for k in fm if k not in conocidas]
    lineas = [
        "formato: 2",
        f"id: {q(fm['id'])}",
        f"titulo: {q(fm['titulo'])}",
        f"tipo: {q(fm.get('tipo'))}",
        f"sprint: {q(fm.get('sprint'))}",
        f"etiquetas: {lista(fm.get('etiquetas'))}",
        f"complejidad: {q(fm.get('complejidad'))}",
        "carril: null",
        f"estado: {q(estado)}",
        f"motivo: {q(motivo)}",
        f"plan_aprobado: {q(bool(fm.get('plan_aprobado')))}",
        f"rama: {q(fm.get('rama'))}",
        f"asignado_a: {q(fm.get('asignado_a'))}",
        "roles:",
        "  diseno: {modelo: null, skills: [], regla: null}",
        f"  implementacion: {{modelo: {q(fm.get('modelo_sugerido'))}, skills: {lista(fm.get('skills_recomendados'))}, "
        f"regla: {q(fm.get('regla_seleccion_skill'))}}}",
        f"  revision: {{agente: {q(fm.get('agente_revisor'))}, modelo: null, skills: [], regla: null}}",
        "estimacion: null",
        f"tokens_diseno: {q(fm.get('tokens_diseno'))}",
        f"tokens_implementacion: {q(fm.get('tokens_implementacion'))}",
        f"tokens_revision: {q(fm.get('tokens_revision'))}",
        f"creado: {q(str(fm.get('creado')))}",
        f"actualizado: {q(str(fm.get('actualizado')))}",
        f"dependencias: {lista(fm.get('dependencias'))}",
    ]
    for k in desconocidas:          # F1-9 D4a: se conservan
        lineas.append(f"{k}: {json.dumps(fm[k], ensure_ascii=False, default=str)}")
        cambios.append(f"= {k} (desconocida, se conserva)")
    return "---\n" + "\n".join(lineas) + "\n---\n" + cuerpo, cambios


# ---------------------------------------------------------------------------
# Config a los bloques de F1-11
# ---------------------------------------------------------------------------

# Traduccion de OpenDockTrading que valida la PoC B de F1-11 (los dos valores de la 0.16 y lo que el proyecto
# declara de nuevo), mas `formato: 2` y la clave base `modo_flujo` (F2-7).
def config_formato2(cfg016, variante="B"):
    """B es la que escribe `actualizar` (D2a); A (control sin centinela), C y D las mide la convivencia."""
    comando = str(cfg016["comando_verificacion"])
    timeout = int(cfg016["timeout_verificacion"])
    modo = cfg016.get("modo_flujo", "automatico")
    # D: modo_flujo entre comillas simples; la 0.16 conserva las simples dentro del valor
    linea_modo = f"modo_flujo: '{modo}'\n" if variante == "D" else f'modo_flujo: "{modo}"\n'
    huella = "  huella_arbol: ['Cargo.lock', 'web/package-lock.json']\n"
    confianza = "confianza_incluye: ['scripts/lector-tareas.py']\n"
    if variante == "C":     # listas en bloque (YAML estandar igual de valido)
        huella = "  huella_arbol:\n    - 'Cargo.lock'\n    - 'web/package-lock.json'\n"
        confianza = "confianza_incluye:\n  - 'scripts/lector-tareas.py'\n"
    texto = (
        "formato: 2\n"
        "# Configuracion de TaskCode 2 (formato 2), migrada desde la 0.16 por tc2 actualizar.\n"
        f"{linea_modo}"
        "verificacion:\n"
        f"  comando: '{comando}'\n"
        "  shell: 'cmd'\n"
        "  perfiles:\n"
        f"    completa: {{timeout: {timeout}, runner: {{github: 'windows'}}}}\n"
        f"{huella}"
        "  programada:\n"
        "    comando: 'cargo audit --no-yanked --deny unsound && npm --prefix web audit'\n"
        "    cron: '0 6 * * 1'\n"
        "    runner: {github: 'windows'}\n"
        "derivados:\n"
        "  - comando: 'python -I scripts/comprobar-modulos.py --comprobar'\n"
        "    tipo: 'comprueba'\n"
        "  - comando: 'python -I scripts/comprobar-fallos.py --probar'\n"
        "    tipo: 'comprueba'\n"
        f"{confianza}"
    )
    if variante == "B":     # marca de documento YAML en la primera linea: la 0.16 aborta (D2a)
        texto = "---\n" + texto
    return texto


# ---------------------------------------------------------------------------
# Hecho a mano (F1-11): parches y propio de OpenDockTrading, con capacidad que los cubre
# ---------------------------------------------------------------------------

# (fichero, alcance, cubierto_por, motivo). alcance: "entero", funciones o bloques (a, b, texto) de lineas.
PARCHES_ODT = [
    ("scripts/lector-tareas.py", "entero", "tareas-json", "lector propio de tareas/0?-*/TASK-*/tarea.md"),
    ("scripts/comprobar-modulos.py", "_cargar_grafo tareas_vivas", "tareas-json", "lectura propia del estado"),
    ("scripts/comprobar-fallos.py", "_cargar_grafo tareas_vivas", "tareas-json", "lectura propia del estado"),
    ("scripts/verificar.bat", [(3, 5, "lo lanzan el freno")], "puertas-en-ci", "comentario del freno de la 0.16"),
    ("scripts/verificar.bat", [(16, 18, "Un candado"), (35, 37, "candado"), (44, 47, "sin_candado")],
     "puertas-en-ci", "candado: una verificacion a la vez"),
    ("scripts/verificar.bat", [(25, 25, "taskctl tiene que estar en el PATH")], "cli-2", "taskctl en el PATH"),
    ("scripts/verificar.bat", [(59, 59, "taskctl roadmap --comparar")], "roadmap-nativo", "roadmap al dia"),
]
# Capacidades que anuncia una 2.0 instalada. La PoC no tiene 2.0: la migracion de la PoC solo implementa las
# dos primeras (las que hace el propio `actualizar`); la 2.0.0 prevista anuncia todas.
CAP_POC = ("estado-descartada", "config-formato-2")
CAP_2_0 = ("estado-descartada", "config-formato-2", "tareas-json", "puertas-en-ci", "cli-2", "roadmap-nativo")
LEEN_CARPETAS = ("tareas-json",)       # un hueco de esta capacidad rompe al proyecto al migrar: es bloqueante


def lineas_parche(repo, rev, fichero, alcance):
    """(lineas de parche, total, donde) recalculados en rev; falla si un bloque no contiene su texto."""
    import ast
    fuente = cat_files(repo, rev, [fichero])[fichero].replace("\r\n", "\n")
    lin = fuente.rstrip("\n").split("\n")
    total = len(lin)
    if alcance == "entero":
        return set(range(1, total + 1)), total, f"1-{total}"
    if isinstance(alcance, str):
        fs = {x.name: (x.lineno, x.end_lineno) for x in ast.parse(fuente).body if isinstance(x, ast.FunctionDef)}
        falta = [a for a in alcance.split() if a not in fs]
        if falta:
            falla("A", f"{fichero}: no tiene {falta}")
            return set(), total, "?"
        s = set()
        for a in alcance.split():
            s |= set(range(fs[a][0], fs[a][1] + 1))
        return s, total, ", ".join(f"{a} {fs[a][0]}-{fs[a][1]}" for a in alcance.split())
    s = set()
    for a, b, frag in alcance:
        if re.sub(r"\s+", " ", frag) not in re.sub(r"\s+", " ", " ".join(lin[a - 1:b])):
            falla("A", f"{fichero}:{a}-{b} no contiene «{frag}»")
        s |= set(range(a, b + 1))
    return s, total, ", ".join(f"{a}-{b}" if a != b else f"{a}" for a, b, _ in alcance)


def hecho_a_mano(repo, rev, capacidades):
    filas = []
    for fichero, alcance, cap, motivo in PARCHES_ODT:
        s, total, donde = lineas_parche(repo, rev, fichero, alcance)
        tipo = "fichero" if alcance == "entero" else ("funciones" if isinstance(alcance, str) else "bloques")
        # un fichero entero que otros cargan, o una funcion, se retira con sustitucion (codigo del proyecto)
        sustitucion = tipo == "funciones" or (tipo == "fichero" and bool(cargado_por(repo, rev, fichero)))
        if cap in capacidades:
            accion = "retirar con sustitucion" if sustitucion else "retirar"
        else:
            accion = "queda: hueco bloqueante" if cap in LEEN_CARPETAS else "queda: hueco"
        filas.append({"clase": "parche", "fichero": fichero, "lineas": len(s), "total": total, "donde": donde,
                      "cubierto_por": cap, "accion": accion, "motivo": motivo})
    # lo que no es parche en los ficheros partidos es propio
    vistos = {}
    for f in filas:
        vistos.setdefault(f["fichero"], [0, f["total"]])[0] += f["lineas"]
    for fichero, (n, total) in sorted(vistos.items()):
        if n < total:
            filas.append({"clase": "propio", "fichero": fichero, "lineas": total - n, "total": total,
                          "donde": "el resto", "cubierto_por": None, "accion": "queda: propio", "motivo": "del proyecto"})
    return filas


def cargado_por(repo, rev, fichero):
    base = os.path.basename(fichero)
    r = git(repo, "grep", "-l", "-F", base, rev, "--", "scripts/", check=False)
    return sorted({x.split(":", 1)[1] for x in r.stdout.decode("utf-8").splitlines()} - {fichero})


# ---------------------------------------------------------------------------
# El plan de migracion como dato
# ---------------------------------------------------------------------------

def marcador(repo):
    return os.path.join(repo, ".git", NOMBRE["dir_git"], "actualizar", "previo.json")


def existe_ref(repo, ref):
    return git(repo, "rev-parse", "-q", "--verify", f"refs/heads/{ref}", check=False).returncode == 0


def planificar(repo, capacidades):
    """Funcion de lectura pura: no escribe nada en el repo. Devuelve el plan y su hash."""
    base = git(repo, "rev-parse", "HEAD").strip()
    rama = git(repo, "rev-parse", "--abbrev-ref", "HEAD").strip()
    cfg_r = git(repo, "cat-file", "-p", f"{base}:.taskcode/config.yml", check=False)
    cfg_txt = cfg_r.stdout.decode("utf-8") if cfg_r.returncode == 0 else ""
    cfg = yaml.safe_load(cfg_txt) if cfg_txt else {}
    plan = {"proyecto": "OpenDockTrading", "rama": rama, "formato_origen": 2 if (cfg or {}).get("formato") == 2 else 1}
    if plan["formato_origen"] == 2:
        plan.update({"base": base, "pasos": []})
        return finalizar(plan)
    tareas = tareas_016(repo, base)
    sucio = git(repo, "--no-optional-locks", "status", "--porcelain").strip() != ""
    estado_real = {"rama_estado": existe_ref(repo, NOMBRE["rama_estado"]),
                   "marcador": os.path.exists(marcador(repo))}
    if estado_real["rama_estado"]:
        arbol_estado = git(repo, "rev-parse", f"refs/heads/{NOMBRE['rama_estado']}^{{tree}}").strip()
        estado_real["rama_estado_reutilizable"] = arbol_estado == git(repo, "rev-parse", f"{base}:tareas").strip()
    movs, uso = [], {}
    for tid in sorted(tareas):
        t = tareas[tid]
        nuevo, cambios = a_formato2(t)
        movs.append({"id": tid, "de": f"tareas/{t['carpeta']}/{tid}", "a": tid, "estado": CARPETA_ESTADO[t["carpeta"]],
                     "cambios": cambios, "sha_nuevo": sha_texto(nuevo)})
    hecho = hecho_a_mano(repo, base, capacidades)
    plan.update({
        "base": base,
        "capacidades": sorted(capacidades),
        "estado_real": estado_real,
        "precondiciones": {"tareas_activas": sorted(tid for tid, t in tareas.items()
                                                    if CARPETA_ESTADO[t["carpeta"]] in ACTIVOS),
                           "arbol_limpio": not sucio},
        "uso": uso_campos(tareas),
        "hecho_a_mano": hecho,
        "movimientos": {"rama_estado": NOMBRE["rama_estado"], "rama_actualizar": NOMBRE["rama_actualizar"],
                        "gitignore": "tareas/", "tareas": movs,
                        "config": {"sha_antes": sha_texto(cfg_txt), "sha_despues": sha_texto(config_formato2(cfg))}},
        "pasos": ["rama-estado", "quitar-tareas-de-develop", "config-formato-2", "montar-worktree", "aplanar",
                  "formato-2", "criterio", "push"],
    })
    return finalizar(plan)


def finalizar(plan):
    canon = json.dumps(plan, sort_keys=True, ensure_ascii=False, separators=(",", ":"))
    plan["hash"] = hashlib.sha256(canon.encode("utf-8")).hexdigest()
    return plan


DEFECTOS_016 = {"tipo": "feature", "sprint": 0, "etiquetas": [], "complejidad": None, "modelo_sugerido": "sonnet",
                "estado": "planificada", "plan_aprobado": False, "asignado_a": None, "agente_revisor": "general-purpose",
                "skills_recomendados": [], "regla_seleccion_skill": None, "ultimo_commit_revisado": None,
                "revision_codex": False, "tokens_diseno": None, "tokens_implementacion": None, "tokens_revision": None,
                "dependencias": []}


def uso_campos(tareas):
    """Tareas con valor distinto del defecto de la 0.16 (criterio de F1-9; `rama` no se mide aqui)."""
    uso = {}
    for k, d in DEFECTOS_016.items():
        uso[k] = sum(1 for t in tareas.values() if (t["fm"].get(k) if t["fm"].get(k) is not None else None) != d
                     and not (d == [] and t["fm"].get(k) in ([], None)))
    uso["actualizado"] = sum(1 for t in tareas.values() if str(t["fm"].get("actualizado")) != str(t["fm"].get("creado")))
    for k in ("id", "titulo", "creado"):
        uso[k] = len(tareas)
    return dict(sorted(uso.items()))


# ---------------------------------------------------------------------------
# Aplicar, criterio de salida y vuelta atras
# ---------------------------------------------------------------------------

class Rechazo(Exception):
    def __init__(self, codigo, texto):
        super().__init__(texto)
        self.codigo = codigo


def subtree_split(repo, base, cache):
    """`git subtree split -P tareas base`; con cache, reutiliza un split anterior del mismo commit."""
    ref_cache = f"refs/heads/split-{base}"
    if cache and os.path.isdir(cache) and git(cache, "rev-parse", "-q", "--verify", ref_cache, check=False).returncode == 0:
        git(repo, "fetch", "-q", cache, f"{ref_cache}:refs/heads/{NOMBRE['rama_estado']}")
        tiempo("subtree split reutilizado de la cache")
        return git(repo, "rev-parse", f"refs/heads/{NOMBRE['rama_estado']}").strip(), None
    t = time.time()
    sha = git(repo, "subtree", "split", "-q", "-P", "tareas", base).strip()
    dur = time.time() - t
    tiempo(f"subtree split calculado en {dur:.0f} s")
    git(repo, "branch", NOMBRE["rama_estado"], sha)
    if cache:
        if not os.path.isdir(cache):
            run(["git", "init", "-q", "--bare", cache])
        git(repo, "push", "-q", cache, f"refs/heads/{NOMBRE['rama_estado']}:{ref_cache}")
    return sha, dur


def refs_remoto(remoto):
    return sorted(git(remoto, "for-each-ref", "--format=%(refname)").split())


def aplicar(repo, plan, capacidades, cache=None, romper=False, forzar_huecos=False, remoto=None):
    """Aplica exactamente `plan`. Devuelve (codigo, informe)."""
    actual = planificar(repo, capacidades)
    if actual["hash"] != plan["hash"]:
        raise Rechazo(3, "el plan ha cambiado desde --simular: vuelve a simular")
    if not plan["pasos"]:
        return 0, {"nada": True}
    pre = plan["precondiciones"]
    if pre["tareas_activas"]:
        raise Rechazo(4, f"hay tareas en curso: {pre['tareas_activas']}")
    if not pre["arbol_limpio"]:
        raise Rechazo(4, "el arbol de trabajo tiene cambios")
    bloq = [f"{h['fichero']} ({h['donde']}) sin {h['cubierto_por']}" for h in plan["hecho_a_mano"]
            if h["accion"] == "queda: hueco bloqueante"]
    if bloq and not forzar_huecos:
        raise Rechazo(5, "huecos bloqueantes: " + "; ".join(bloq))
    base, informe = plan["base"], {}
    os.makedirs(os.path.dirname(marcador(repo)), exist_ok=True)
    if not os.path.exists(marcador(repo)):
        with open(marcador(repo), "w", encoding="utf-8") as f:
            json.dump({"rama": plan["rama"], "sha": base}, f)
    # 1. rama de estado (huerfana) con la historia de tareas/
    if plan["estado_real"].get("rama_estado_reutilizable"):
        split, informe["split"] = git(repo, "rev-parse", f"refs/heads/{NOMBRE['rama_estado']}").strip(), "reutilizada"
        informe["dur_split"] = None
    else:
        split, informe["dur_split"] = subtree_split(repo, base, cache)
        informe["split"] = "calculada"
    informe["sha_split"] = split
    # 2 y 3. rama actualizar: quita tareas/ de develop, la ignora y migra la config
    git(repo, "checkout", "-q", "-b", NOMBRE["rama_actualizar"])
    git(repo, "rm", "-r", "-q", "tareas")
    gi = os.path.join(repo, ".gitignore")
    previo_gi = open(gi, encoding="utf-8").read() if os.path.exists(gi) else ""
    with open(gi, "w", encoding="utf-8", newline="\n") as f:
        f.write(previo_gi + ("" if previo_gi.endswith("\n") or not previo_gi else "\n") + "tareas/\n")
    cfg = yaml.safe_load(cat_files(repo, base, [".taskcode/config.yml"])[".taskcode/config.yml"])
    nueva = config_formato2(cfg)
    if sha_texto(nueva) != plan["movimientos"]["config"]["sha_despues"]:
        raise SystemExit("ERROR: la config escrita no es la del plan")
    with open(os.path.join(repo, ".taskcode", "config.yml"), "w", encoding="utf-8", newline="\n") as f:
        f.write(nueva)
    git(repo, "add", "-A", ".gitignore", ".taskcode/config.yml")
    git(repo, "commit", "-q", "-m", f"actualizar: tareas a la rama de estado y config formato 2\n\nMigrado-desde: {base}")
    # 4. worktree de la rama de estado en tareas/
    git(repo, "worktree", "add", "-q", "tareas", NOMBRE["rama_estado"])
    wt = os.path.join(repo, "tareas")
    # 5. aplanado: solo renombres (historia con --follow al 100 % de similitud)
    tareas = tareas_016(repo, base)
    for m in plan["movimientos"]["tareas"]:
        git(wt, "mv", m["de"][len("tareas/"):], m["a"])
    for d in CARPETA_ESTADO:          # las carpetas de estado de la 0.16 (solo les queda su .gitkeep)
        if os.path.exists(os.path.join(wt, d, ".gitkeep")):
            git(wt, "rm", "-q", f"{d}/.gitkeep")
        p = os.path.join(wt, d)
        if os.path.isdir(p) and not os.listdir(p):
            os.rmdir(p)
    git(wt, "commit", "-q", "-m", f"actualizar: aplanado TASK-NNN (estado como campo)\n\nMigrado-desde: {base}")
    # 6. formato 2
    for m in plan["movimientos"]["tareas"]:
        nuevo, _ = a_formato2(tareas[m["id"]])
        if sha_texto(nuevo) != m["sha_nuevo"]:
            raise SystemExit(f"ERROR: {m['id']}: el contenido no es el del plan")
        with open(os.path.join(wt, m["a"], "tarea.md"), "w", encoding="utf-8", newline="") as f:
            f.write(nuevo)
    if romper:      # criterio roto a proposito: se borra una dependencia antes de evaluar
        victima = next(m["id"] for m in plan["movimientos"]["tareas"] if tareas[m["id"]]["fm"].get("dependencias"))
        p = os.path.join(wt, victima, "tarea.md")
        t = open(p, encoding="utf-8").read()
        t = re.sub(r"^dependencias: \[.*\]$", "dependencias: []", t, count=1, flags=re.M)
        open(p, "w", encoding="utf-8", newline="").write(t)
        informe["roto"] = victima
    git(wt, "add", "-A")
    git(wt, "commit", "-q", "-m", f"actualizar: tareas a formato 2 (F1-9)\n\nMigrado-desde: {base}")
    # 7. criterio de salida
    ok, detalle = criterio(repo, base)
    informe["criterio"] = detalle
    informe["refs_remoto_antes_del_push"] = len(refs_remoto(remoto)) if remoto else None
    if not ok:
        vuelta_atras(repo)
        return 6, informe
    # 8. push, solo con el criterio en verde
    if remoto:
        git(repo, "push", "-q", "origin", NOMBRE["rama_estado"], NOMBRE["rama_actualizar"])
        informe["refs_remoto_tras_push"] = len(refs_remoto(remoto))
    return 0, informe


RX_FILA_ROADMAP = re.compile(r"^\|[^|]*\|\s*(TASK-\d+[a-z]?)\s*\|.*\|\s*([^|]*)\|\s*([^|]*)\|\s*$")


def roadmap_estructura(texto):
    """{id: (dependencias, estado)} de las tablas de «Estado del trabajo» del ROADMAP.md de la 0.16."""
    out = {}
    for l in texto.splitlines():
        m = RX_FILA_ROADMAP.match(l)
        if m:
            deps = [] if m.group(2).strip() in ("—", "-", "") else [d.strip() for d in m.group(2).split(",")]
            # la columna Estado del roadmap de la 0.16 es derivada: «✅ terminada», «lista para empezar», «espera a ...»
            out[m.group(1)] = (tuple(sorted(deps)), "terminada" if "terminada" in m.group(3) else "pendiente")
    return out


def criterio(repo, base):
    """Criterio de salida estructural. Devuelve (ok, detalle)."""
    rama = f"refs/heads/{NOMBRE['rama_estado']}"
    d = {}
    antes_t = tareas_016(repo, base)
    antes = instantanea(antes_t, lambda t: CARPETA_ESTADO[t["carpeta"]])
    despues_t, no_yaml = tareas_f2(repo, rama)
    despues = instantanea(despues_t, lambda t: t["fm"].get("estado"))
    d["tareas"] = f"{len(despues)} de {len(antes)}"
    d["no_yaml"] = len(no_yaml)
    d["formato2"] = sum(1 for t in despues_t.values() if t["fm"].get("formato") == 2)
    difs = sorted(f"{tid}: {k}" for tid in set(antes) | set(despues)
                  for k in ("titulo", "estado", "dependencias", "sprint", "etiquetas")
                  if (antes.get(tid) or {}).get(k) != (despues.get(tid) or {}).get(k))
    d["diferencias"] = difs
    rm = roadmap_estructura(cat_files(repo, rama, ["ROADMAP.md"])["ROADMAP.md"])
    vivas = {tid: (tuple(sorted(v["dependencias"])), "terminada" if v["estado"] == "terminada" else "pendiente")
             for tid, v in despues.items() if v["estado"] != "descartada"}
    d["roadmap"] = "iguales" if rm == vivas else f"distinto: {sorted(set(rm.items()) ^ set(vivas.items()))[:3]}"
    d["roadmap_filas"] = len(rm)
    r = git(repo, "merge-base", "HEAD", rama, check=False)
    d["merge_base"] = "ninguna" if r.returncode == 1 and not r.stdout.strip() else r.stdout.decode().strip()
    # historia por tarea: todos los commits que la tocan (por ruta, en cualquier carpeta) y `git log --follow`
    completas, seguidas, cortas, rotas_016 = 0, 0, [], []
    for tid, t in sorted(antes_t.items()):
        n_dev = len(git(repo, "log", "--format=%H", base, "--", f":(glob)tareas/*/{tid}/tarea.md").split())
        n_est = len(git(repo, "log", "--format=%H", rama, "--", f":(glob)*/{tid}/tarea.md", f"{tid}/tarea.md").split())
        f_dev = len(git(repo, "log", "--follow", "--format=%H", base, "--", t["ruta"]).split())
        f_est = len(git(repo, "log", "--follow", "--format=%H", rama, "--", f"{tid}/tarea.md").split())
        completas += n_est >= n_dev
        seguidas += f_est >= f_dev
        if n_est < n_dev or f_est < f_dev:
            cortas.append(f"{tid} ruta {n_est}/{n_dev} follow {f_est}/{f_dev}")
        if f_dev < n_dev:
            rotas_016.append(f"{tid} {f_dev} de {n_dev}")
    d["historia"] = f"{completas}/{len(antes_t)}"
    d["follow"] = f"{seguidas}/{len(antes_t)}"
    d["follow_cortas"] = cortas[:5]
    d["follow_roto_en_develop"] = rotas_016
    ok = (not difs and not no_yaml and d["formato2"] == len(antes) and len(despues) == len(antes)
          and d["roadmap"] == "iguales" and d["merge_base"] == "ninguna" and seguidas == len(antes_t)
          and completas == len(antes_t))
    return ok, d


def vuelta_atras(repo):
    """Deja el repo como estaba antes de `actualizar`: local, sin push."""
    with open(marcador(repo), encoding="utf-8") as f:
        previo = json.load(f)
    if os.path.exists(os.path.join(repo, "tareas", ".git")):
        git(repo, "worktree", "remove", "--force", "tareas")
    git(repo, "worktree", "prune")
    git(repo, "checkout", "-q", "-f", previo["rama"])
    git(repo, "reset", "-q", "--hard", previo["sha"])
    for ref in (NOMBRE["rama_estado"], NOMBRE["rama_actualizar"]):
        if existe_ref(repo, ref):
            git(repo, "branch", "-q", "-D", ref)
    borrar(os.path.join(repo, ".git", "subtree-cache"))
    borrar(os.path.join(repo, ".git", NOMBRE["dir_git"]))


# ---------------------------------------------------------------------------
# M: migracion de la copia de OpenDockTrading
# ---------------------------------------------------------------------------

def preparar_copia(personal, trabajo, nombre):
    origen = os.path.join(personal, "OpenDockTrading")
    copia = os.path.join(trabajo, nombre)
    run(["git", "clone", "-q", "--no-hardlinks", "--no-checkout", origen, copia], cwd=trabajo, env=env_git())
    git(copia, "checkout", "-q", "-B", "develop", FIJO_ODT)
    git(copia, "remote", "remove", "origin")
    for r in git(copia, "for-each-ref", "--format=%(refname)", "refs/remotes/").split():
        git(copia, "update-ref", "-d", r)
    return copia


def contar_por_carpeta(tareas):
    return " / ".join(str(sum(1 for t in tareas.values() if t["carpeta"] == c)) for c in CARPETA_ESTADO)


def seccion_m(personal, trabajo, cache):
    sep("M: migracion de una copia de OpenDockTrading (commit fijo)")
    copia = preparar_copia(personal, trabajo, "odt-migrada")
    remoto = os.path.join(trabajo, "odt-remoto.git")
    run(["git", "init", "-q", "--bare", remoto], cwd=trabajo)
    git(copia, "remote", "add", "origin", remoto)
    git(copia, "push", "-q", "origin", "develop")
    tareas = tareas_016(copia, FIJO_ODT)
    por = contar_por_carpeta(tareas)
    print(f"commit fijo {FIJO_ODT[:9]}: {len(tareas)} tareas ; por carpeta {' / '.join(CARPETA_ESTADO)}: {por}")
    MEDIDO["OpenDockTrading: tareas"] = str(len(tareas))
    MEDIDO["OpenDockTrading: tareas por carpeta (00..05)"] = por
    n_hist = len(git(copia, "log", "--format=%H", FIJO_ODT, "--", "tareas").split())
    print(f"commits de develop que tocan tareas/: {n_hist}")
    MEDIDO["commits de develop que tocan tareas/"] = str(n_hist)

    # --simular: dos llamadas, mismo hash, ningun byte cambiado
    h0 = hash_dir(copia)
    p1 = planificar(copia, CAP_POC)
    p1b = planificar(copia, CAP_POC)
    h1 = hash_dir(copia)
    cambiados = diferencias(h0, h1)
    print(f"--simular: hash del plan {p1['hash'][:16]} ; segunda llamada {'igual' if p1b['hash'] == p1['hash'] else 'DISTINTA'} ; "
          f"ficheros cambiados en la copia (con .git): {len(cambiados)}")
    MEDIDO["--simular: hash del plan en dos llamadas"] = "igual" if p1b["hash"] == p1["hash"] else "distinto"
    MEDIDO["--simular: ficheros cambiados en la copia (con .git)"] = str(len(cambiados))
    if p1b["hash"] != p1["hash"] or cambiados:
        falla("M", f"--simular no es puro: {cambiados[:5]}")
    with open(os.path.join(trabajo, "plan-odt.json"), "w", encoding="utf-8", newline="\n") as f:
        json.dump(p1, f, sort_keys=True, ensure_ascii=False, indent=1)
    movs = p1["movimientos"]["tareas"]
    desc = [m for m in movs if m["estado"] == "descartada"]
    con_motivo = [m for m in desc if "motivo: de ## Descartada" in m["cambios"]]
    retiradas = sum(1 for m in movs for c in m["cambios"] if c in ("- revision_codex", "- ultimo_commit_revisado"))
    est_cambiado = [m["id"] for m in movs if any(c.startswith("estado:") for c in m["cambios"])]
    print(f"plan: pasos={len(p1['pasos'])} ; tareas a mover={len(movs)} ; a descartada={len(desc)} "
          f"({', '.join(m['id'] for m in desc)}) ; con motivo de ## Descartada={len(con_motivo)} ; "
          f"claves retiradas={retiradas} ; estado del frontmatter corregido={len(est_cambiado)} ({', '.join(est_cambiado)})")
    MEDIDO["plan: archivadas a descartada con motivo de ## Descartada"] = f"{len(con_motivo)}/{len(desc)}"
    MEDIDO["plan: claves retiradas (revision_codex, ultimo_commit_revisado)"] = str(retiradas)
    MEDIDO["plan: tareas cuyo estado del frontmatter no casaba con su carpeta"] = str(len(est_cambiado))
    act = p1["precondiciones"]["tareas_activas"]
    print(f"precondiciones: tareas activas={len(act)} ; arbol limpio={p1['precondiciones']['arbol_limpio']}")
    MEDIDO["precondicion: tareas en diseno, en curso o en revision"] = str(len(act))

    # negativa: con la 2.0 de la PoC (sin tareas-json) hay huecos bloqueantes
    h0 = hash_dir(copia)
    try:
        aplicar(copia, p1, CAP_POC, cache=cache, remoto=remoto)
        falla("M", "actualizar no se nego con huecos bloqueantes")
        codigo_neg = 0
    except Rechazo as e:
        codigo_neg = e.codigo
        print(f"actualizar sin capacidad tareas-json: codigo {e.codigo} ; {e}")
    neg_cambios = len(diferencias(h0, hash_dir(copia)))
    print(f"   ficheros cambiados por la negativa (con .git): {neg_cambios}")
    MEDIDO["actualizar con huecos bloqueantes: codigo y ficheros cambiados"] = f"{codigo_neg} y {neg_cambios}"
    if codigo_neg != 5 or neg_cambios:
        falla("M", "la negativa por huecos no es limpia")

    # migracion (forzada en la PoC para medir el resto)
    hwt0 = hash_dir(copia, con_git=False)
    codigo, inf = aplicar(copia, p1, CAP_POC, cache=cache, forzar_huecos=True, remoto=remoto)
    tiempo("migracion aplicada")
    c = inf["criterio"]
    print(f"actualizar (huecos forzados en la PoC): codigo {codigo} ; rama de estado {inf['split']} "
          f"{inf['sha_split'][:9]}")
    if codigo != 0:
        print(f"criterio en rojo: {json.dumps(c, ensure_ascii=False)[:1500]}")
        falla("M", "la migracion no paso el criterio")
        return copia, remoto, tareas
    n_split = len(git(copia, "rev-list", inf["sha_split"]).split())
    n_estado = len(git(copia, "rev-list", f"refs/heads/{NOMBRE['rama_estado']}").split())
    print(f"subtree split: {n_split} commits ; rama de estado tras aplanar y formato 2: {n_estado} commits "
          f"(el tiempo de reloj va a stderr: no es una cifra comprobada)")
    MEDIDO["subtree split: commits de la rama de estado"] = str(n_split)
    print(f"criterio: tareas {c['tareas']} ; frontmatter no YAML {c['no_yaml']} ; con formato 2 {c['formato2']} ; "
          f"diferencias {len(c['diferencias'])} ; roadmap {c['roadmap']} ({c['roadmap_filas']} filas) ; "
          f"merge-base {c['merge_base']}")
    print(f"historia por tarea: commits por ruta >= develop {c['historia']} ; git log --follow >= el de develop {c['follow']} "
          f"{c['follow_cortas']} ; --follow ya roto en develop (la 0.16 mueve y reescribe en el mismo commit): "
          f"{len(c['follow_roto_en_develop'])} {c['follow_roto_en_develop']}")
    MEDIDO["criterio: tareas en la rama de estado"] = c["tareas"]
    MEDIDO["criterio: frontmatter que carga PyYAML con formato 2"] = f"{c['formato2']}/{len(tareas)}"
    MEDIDO["criterio: diferencias de la instantanea"] = str(len(c["diferencias"]))
    MEDIDO["criterio: roadmap por estructura"] = f"{c['roadmap']} ({c['roadmap_filas']} filas)"
    MEDIDO["criterio: merge-base de develop y la rama de estado"] = c["merge_base"]
    MEDIDO["criterio: tareas con todos sus commits en la rama de estado"] = c["historia"]
    MEDIDO["criterio: tareas con git log --follow igual o mayor que en develop"] = c["follow"]
    MEDIDO["git log --follow ya roto en develop por la 0.16"] = (", ".join(c["follow_roto_en_develop"]) or "ninguna")
    print(f"remoto: refs antes del push {inf['refs_remoto_antes_del_push']} ; tras el push {inf.get('refs_remoto_tras_push')}")
    MEDIDO["remoto: refs antes del criterio y tras el push"] = \
        f"{inf['refs_remoto_antes_del_push']} -> {inf.get('refs_remoto_tras_push')}"
    if codigo != 0:
        falla("M", f"la migracion no paso el criterio: {c}")
        return copia, remoto, tareas
    # el PR entra en develop (simulado: fast-forward, mismo arbol) y se monta develop
    git(copia, "update-ref", "refs/heads/develop", f"refs/heads/{NOMBRE['rama_actualizar']}")
    git(copia, "checkout", "-q", "develop")
    git(copia, "push", "-q", "origin", "develop")
    raiz = sorted(os.listdir(os.path.join(copia, "tareas")))
    print(f"tareas/ montada: {sum(1 for x in raiz if x.startswith('TASK-'))} carpetas TASK-NNN ; otras entradas: "
          f"{[x for x in raiz if not x.startswith('TASK-')]}")
    cambios_wt = diferencias(hwt0, hash_dir(copia, con_git=False))
    fuera = sorted({x for x in cambios_wt if not x.startswith("tareas/")})
    print(f"ficheros de codigo cambiados por la migracion (fuera de tareas/): {fuera}")
    MEDIDO["ficheros de codigo que cambia la migracion"] = ", ".join(fuera)

    # el lector propio de OpenDockTrading frente a la carpeta fija (hueco bloqueante, medido)
    spec = importlib.util.spec_from_file_location("lector_odt", os.path.join(copia, "scripts", "lector-tareas.py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    vistas_016 = len([t for t in tareas.values() if "descartada" not in (t["fm"].get("etiquetas") or [])])
    vistas = len(mod.cargar(Path(copia) / "tareas"))
    fijas, descartadas_vistas = 0, []
    for f in sorted((Path(copia) / "tareas").glob("TASK-*/tarea.md")):
        fm = mod.leer_tarea(f.read_text(encoding="utf-8"))
        if "descartada" not in fm.get("etiquetas", []):
            fijas += 1
            if f.parent.name in {m for m, t in tareas.items() if t["carpeta"] == "05-archivadas"}:
                descartadas_vistas.append(f.parent.name)
    print(f"lector-tareas.py de OpenDockTrading tras migrar: ve {vistas} de {vistas_016} ; con el glob de la carpeta fija "
          f"(TASK-*/tarea.md): {fijas} de {vistas_016} ; descartadas que no reconoce (su parser no quita las comillas "
          f"de las listas del formato 2): {descartadas_vistas}")
    MEDIDO["lector-tareas.py tras migrar: tareas que ve"] = f"{vistas} de {vistas_016}"
    MEDIDO["lector-tareas.py con el glob de la carpeta fija"] = f"{fijas} de {vistas_016}"
    MEDIDO["lector-tareas.py: descartadas del formato 2 que no reconoce"] = ", ".join(descartadas_vistas) or "ninguna"

    # segunda pasada: plan vacio, nada cambia
    h0 = hash_dir(copia)
    p2 = planificar(copia, CAP_POC)
    cod2, _ = aplicar(copia, p2, CAP_POC, remoto=remoto)
    cambios2 = len(diferencias(h0, hash_dir(copia)))
    print(f"segunda pasada: formato de origen {p2['formato_origen']} ; pasos {len(p2['pasos'])} ; codigo {cod2} ; "
          f"ficheros cambiados {cambios2}")
    MEDIDO["segunda pasada: pasos del plan y ficheros cambiados"] = f"{len(p2['pasos'])} y {cambios2}"
    if p2["pasos"] or cambios2 or cod2:
        falla("M", "la segunda pasada no es vacia")

    # vuelta atras: segunda copia que reanuda una migracion a medias (rama de estado ya creada) y rompe el criterio
    sep("M2: vuelta atras automatica con el criterio roto (copia que reanuda una migracion a medias)")
    copia2 = preparar_copia(personal, trabajo, "odt-vuelta-atras")
    remoto2 = os.path.join(trabajo, "odt-remoto-2.git")
    run(["git", "init", "-q", "--bare", remoto2], cwd=trabajo)
    git(copia2, "remote", "add", "origin", remoto2)
    git(copia2, "push", "-q", "origin", "develop")
    hwt_prist = hash_dir(copia2, con_git=False)
    refs2 = refs_remoto(remoto2)
    git(copia2, "fetch", "-q", copia, f"{inf['sha_split']}:refs/heads/{NOMBRE['rama_estado']}")
    os.makedirs(os.path.dirname(marcador(copia2)), exist_ok=True)
    with open(marcador(copia2), "w", encoding="utf-8") as f:
        json.dump({"rama": "develop", "sha": FIJO_ODT}, f)
    p3 = planificar(copia2, CAP_POC)
    print(f"estado real recalculado: {json.dumps(p3['estado_real'], sort_keys=True)}")
    MEDIDO["reanudar: rama de estado a medias reutilizable"] = \
        "si" if p3["estado_real"].get("rama_estado_reutilizable") else "no"
    cod3, inf3 = aplicar(copia2, p3, CAP_POC, romper=True, forzar_huecos=True, remoto=remoto2)
    c3 = inf3["criterio"]
    print(f"actualizar con una dependencia borrada en {inf3.get('roto')}: codigo {cod3} ; diferencias {c3['diferencias']} ; "
          f"roadmap {c3['roadmap'][:60]}")
    head = git(copia2, "rev-parse", "HEAD").strip()
    rama2 = git(copia2, "rev-parse", "--abbrev-ref", "HEAD").strip()
    refs_tc = git(copia2, "for-each-ref", "--format=%(refname)", "refs/heads/taskcodetwo", "refs/heads/actualizar").split()
    wts = len([l for l in git(copia2, "worktree", "list", "--porcelain").splitlines() if l.startswith("worktree ")])
    limpio = git(copia2, "status", "--porcelain").strip() == ""
    igual_wt = hash_dir(copia2, con_git=False) == hwt_prist
    dir_tc = os.path.exists(os.path.join(copia2, ".git", NOMBRE["dir_git"]))
    remoto_igual = refs_remoto(remoto2) == refs2
    print(f"tras la vuelta atras: rama {rama2} ; HEAD {'= SHA previo' if head == FIJO_ODT else head} ; refs nuevas {refs_tc} ; "
          f"worktrees {wts} ; git status {'limpio' if limpio else 'SUCIO'} ; arbol de trabajo "
          f"{'identico' if igual_wt else 'DISTINTO'} ; .git/{NOMBRE['dir_git']} {'existe' if dir_tc else 'no existe'} ; "
          f"remoto {'sin cambios' if remoto_igual else 'CAMBIADO'}")
    MEDIDO["vuelta atras: codigo con el criterio roto"] = str(cod3)
    MEDIDO["vuelta atras: diferencia detectada"] = ", ".join(c3["diferencias"])
    MEDIDO["vuelta atras: estado final"] = (
        f"HEAD {'= previo' if head == FIJO_ODT else 'distinto'}, {len(refs_tc)} refs, {wts} worktree, "
        f"status {'limpio' if limpio else 'sucio'}, remoto {'sin cambios' if remoto_igual else 'cambiado'}")
    if cod3 != 6 or head != FIJO_ODT or refs_tc or wts != 1 or not limpio or not igual_wt or dir_tc or not remoto_igual:
        falla("M2", "la vuelta atras no deja la copia como estaba")
    p4 = planificar(copia2, CAP_POC)
    igual = p4["hash"] == p1["hash"]
    print(f"plan recalculado tras la vuelta atras: {'igual' if igual else 'DISTINTO'} al inicial")
    MEDIDO["vuelta atras: plan recalculado igual al inicial"] = "si" if igual else "no"
    if not igual:
        falla("M2", "el plan tras la vuelta atras no es el inicial")
    return copia, remoto, tareas


# ---------------------------------------------------------------------------
# C: convivencia con la 0.16 sobre la copia migrada
# ---------------------------------------------------------------------------

def taskctl(plugin, copia, *args, entrada=None):
    e = dict(os.environ)
    e.pop("CLAUDE_PROJECT_DIR", None)
    e["FRENO_VERIFICACION"] = e.get("FRENO_VERIFICACION", "1")
    r = run(["node", os.path.join(plugin, "bin", "taskctl"), *args], cwd=copia, env=e, entrada=entrada,
            check=False, timeout=300)
    return r.returncode, (r.stdout + r.stderr).decode("utf-8", "replace")


def hook(plugin, copia, nombre, payload):
    e = dict(os.environ)
    e["CLAUDE_PLUGIN_ROOT"] = plugin
    e["CLAUDE_PROJECT_DIR"] = copia
    r = run(["node", os.path.join(plugin, "dist", "src", "hooks", nombre)], cwd=copia, env=e,
            entrada=json.dumps(payload).encode("utf-8"), check=False, timeout=300)
    return r.returncode, (r.stdout + r.stderr).decode("utf-8", "replace")


def primera(texto, maximo=110):
    """La primera linea de error si la hay; si no, la primera linea."""
    lineas = texto.splitlines()
    errores = [l for l in lineas if re.match(r"\s*\[(ERROR|error)\]", l) or l.strip().startswith("[ERROR]")]
    for l in errores + lineas:
        l = l.strip()
        if l:
            l = re.sub(r"[A-Za-z]:[\\/][^\s\"']*pruebas-claude[^\s\"']*", "<copia>", l)
            return l[:maximo]
    return "(sin salida)"


def restaurar(copia, sha):
    git(copia, "reset", "-q", "--hard", sha)
    wt = os.path.join(copia, "tareas")
    git(wt, "reset", "-q", "--hard")
    git(wt, "clean", "-q", "-fdx")


def seccion_c(plugin, copia):
    sep("C: convivencia, la 0.16 sobre la copia migrada")
    sha_a = git(copia, "rev-parse", "HEAD").strip()
    estado_payload = {"cwd": copia, "workspace": {"current_dir": copia}}
    sesion = {"session_id": "poc-f1-12", "transcript_path": os.path.join(copia, "no-existe.jsonl"), "cwd": copia}
    ordenes = [
        ("board", lambda: taskctl(plugin, copia, "board")),
        ("doctor", lambda: taskctl(plugin, copia, "doctor")),
        ("estado --stdin", lambda: taskctl(plugin, copia, "estado", "--stdin",
                                           entrada=json.dumps(estado_payload).encode("utf-8"))),
        ("roadmap --escribir", lambda: taskctl(plugin, copia, "roadmap", "--escribir")),
        ("new", lambda: taskctl(plugin, copia, "new", "--tipo", "feature", "--titulo", "Tarea de convivencia", "--objetivo",
                                "Comprobar la convivencia", "--criterio", "Se crea o se rechaza")),
        ("hook pre-tool-merge", lambda: hook(plugin, copia, "pre-tool-merge.js", dict(
            sesion, hook_event_name="PreToolUse", tool_name="Bash",
            tool_input={"command": "git merge --no-ff feature/task-021-x"}))),
        ("hook pre-tool-lectura", lambda: hook(plugin, copia, "pre-tool-lectura.js", dict(
            sesion, hook_event_name="PreToolUse", tool_name="Read",
            tool_input={"file_path": os.path.join(copia, "tareas", "TASK-021", "tarea.md")}))),
        ("hook stop-verificacion", lambda: hook(plugin, copia, "stop-verificacion.js", dict(
            sesion, hook_event_name="Stop", stop_hook_active=False))),
        ("hook coste-turno", lambda: hook(plugin, copia, "coste-turno.js", dict(
            sesion, hook_event_name="Stop", stop_hook_active=False))),
    ]
    escritura = ("roadmap --escribir", "new")
    variantes = {}
    cfg016 = yaml.safe_load(cat_files(copia, FIJO_ODT, [".taskcode/config.yml"])[".taskcode/config.yml"])
    for v, desc in (("B", "con '---' en la primera linea (la que escribe la migracion)"),
                    ("A", "control: la misma sin '---'"), ("C", "listas en bloque, sin '---'"),
                    ("D", "modo_flujo entre comillas simples, sin '---'")):
        restaurar(copia, sha_a)
        if v != "B":
            with open(os.path.join(copia, ".taskcode", "config.yml"), "w", encoding="utf-8", newline="\n") as f:
                f.write(config_formato2(cfg016, v))
            yaml.safe_load(config_formato2(cfg016, v))     # YAML estandar valido
            git(copia, "commit", "-q", "-am", f"poc: config formato 2, variante {v}")
        sha_v = git(copia, "rev-parse", "HEAD").strip()
        print(f"-- config formato 2, variante {v}: {desc}")
        res = {}
        for nombre, f in ordenes:
            h0 = hash_dir(copia)
            rm_ruta = os.path.join(copia, "tareas", "ROADMAP.md")
            filas0 = len(roadmap_estructura(open(rm_ruta, encoding="utf-8").read()))
            codigo, salida = f()
            filas1 = len(roadmap_estructura(open(rm_ruta, encoding="utf-8").read())) if os.path.exists(rm_ruta) else 0
            camb = diferencias(h0, hash_dir(copia))
            proyecto = [x for x in camb if not x.startswith(".git/")]
            ids = sorted(set(re.findall(r"TASK-\d{3,}", salida)))
            res[nombre] = {"codigo": codigo, "proyecto": proyecto, "git": [x for x in camb if x.startswith(".git/")]}
            extra = f" ; TASK distintas en la salida: {len(ids)}" if nombre in ("board", "doctor") else ""
            print(f"   {nombre}: codigo {codigo} ; escribe en el proyecto: {resumir_rutas(proyecto)} ; "
                  f"en .git: {resumir_rutas([x for x in camb if x.startswith('.git/')], 3)}{extra}")
            print(f"      {primera(salida)}")
            if nombre == "roadmap --escribir":
                print(f"      filas de tarea en tareas/ROADMAP.md: {filas0} -> {filas1}")
                res[nombre]["filas"] = f"{filas0} -> {filas1}"
            if nombre == "doctor":
                errs = sorted(set(re.findall(r"\[ERROR\]\s+([\w-]+):", salida)))
                avisos = len(re.findall(r"\[AVISO\]", salida))
                print(f"      doctor: comprobaciones en error {errs} ; avisos {avisos}")
                res[nombre]["errores"] = errs
            if nombre == "new" and proyecto:
                creadas = sorted({"/".join(x.split("/")[:3]) for x in proyecto})
                choca = [p for p in creadas if os.path.isdir(os.path.join(copia, "tareas", p.split("/")[-1]))]
                print(f"      new crea {creadas} ; ID que ya existe en la carpeta fija: {len(choca)}")
                res[nombre]["creadas"] = creadas
                res[nombre]["choca"] = choca
            restaurar(copia, sha_v)
        variantes[v] = res
    # ficheros de estado de cada version bajo .git
    tc16 = sorted(os.listdir(os.path.join(copia, ".git", "taskcode"))) if os.path.isdir(
        os.path.join(copia, ".git", "taskcode")) else []
    tc2 = sorted(os.listdir(os.path.join(copia, ".git", NOMBRE["dir_git"]))) if os.path.isdir(
        os.path.join(copia, ".git", NOMBRE["dir_git"])) else []
    print(f".git/taskcode tras la convivencia: {tc16} ; .git/{NOMBRE['dir_git']}: {tc2} ; compartidos: "
          f"{len(set(tc16) & set(tc2))}")
    MEDIDO[f".git/taskcode y .git/{NOMBRE['dir_git']}: ficheros compartidos"] = str(len(set(tc16) & set(tc2)))
    # Medidas de la tabla
    for v in variantes:
        for nombre in ("board", "doctor", "new", "roadmap --escribir", "hook pre-tool-merge", "hook stop-verificacion"):
            r = variantes[v][nombre]
            MEDIDO[f"0.16 {nombre}, config {v}: codigo"] = str(r["codigo"])
        MEDIDO[f"0.16 roadmap --escribir, config {v}: filas de tarea del ROADMAP"] = variantes[v]["roadmap --escribir"]["filas"]
        MEDIDO[f"0.16 doctor, config {v}: comprobaciones en error"] = ", ".join(variantes[v]["doctor"]["errores"])
        r = variantes[v]["new"]
        MEDIDO[f"0.16 new, config {v}: ID duplicado creado"] = \
            (", ".join(c.split("/")[-1] for c in r.get("choca", [])) or "ninguno")
    # D2: fallo cerrado = toda orden de escritura de la 0.16 sale != 0 sin escribir nada en el proyecto
    cerrado = {v: all(variantes[v][o]["codigo"] != 0 and not variantes[v][o]["proyecto"] for o in escritura)
               for v in variantes}
    abiertos = {v: [o for o in ("hook pre-tool-merge", "hook stop-verificacion") if variantes[v][o]["codigo"] == 0]
                for v in variantes}
    for v in variantes:
        print(f"D2, variante {v}: ordenes de escritura de la 0.16 cerradas={cerrado[v]} ; hooks que dejan pasar: {abiertos[v]}")
        MEDIDO[f"D2: ordenes de escritura de la 0.16 cerradas con la config {v}"] = "si" if cerrado[v] else "no"
    if not cerrado["B"]:
        falla("C", "la config que escribe la migracion (B) no frena a la 0.16")
    if cerrado["A"]:
        falla("C", "el control A (sin centinela) tambien frena: la medida no distingue")
    if variantes["B"]["hook pre-tool-merge"]["codigo"] != 2:
        falla("C", "el guardian de merges de la 0.16 no bloquea con la config B")
    return variantes, cerrado


# ---------------------------------------------------------------------------
# A: informe de adopcion
# ---------------------------------------------------------------------------

def seccion_a(copia, tareas):
    sep("A: informe de adopcion de OpenDockTrading (lo que mostraria actualizar --simular)")
    uso = uso_campos(tareas)
    sin_uso = [k for k, n in uso.items() if n == 0]
    print("uso de los campos (tareas con valor distinto del defecto de la 0.16; `rama` no se mide aqui):")
    print("   " + " ; ".join(f"{k}={n}" for k, n in uso.items()))
    print(f"   campos sin uso: {sin_uso}")
    MEDIDO["uso: campos de la 0.16 sin ningun valor propio en OpenDockTrading"] = ", ".join(sin_uso)
    for caps, etiqueta in ((CAP_POC, "sin 2.0 (capacidades de la PoC)"), (CAP_2_0, "con la 2.0.0 prevista")):
        filas = hecho_a_mano(copia, FIJO_ODT, caps)
        print(f"-- hecho a mano, {etiqueta}")
        for h in filas:
            print(f"   {h['clase']:7} {h['fichero']}:{h['donde']} ({h['lineas']} de {h['total']}) ; "
                  f"cubierto_por={h['cubierto_por']} ; {h['accion']}")
        ret = sum(h["lineas"] for h in filas if h["accion"].startswith("retirar"))
        auto = sum(h["lineas"] for h in filas if h["accion"] == "retirar")
        sust = sum(h["lineas"] for h in filas if h["accion"] == "retirar con sustitucion")
        queda = sum(h["lineas"] for h in filas if not h["accion"].startswith("retirar"))
        prop = sum(h["lineas"] for h in filas if h["clase"] == "propio")
        bloq = sum(h["lineas"] for h in filas if h["accion"] == "queda: hueco bloqueante")
        print(f"   lineas que se retiran={ret} (solas={auto}, con sustitucion={sust}) ; que se quedan={queda} "
              f"(propias={prop}, huecos bloqueantes={bloq})")
        clave = "sin 2.0" if caps is CAP_POC else "2.0.0 prevista"
        MEDIDO[f"OpenDockTrading, {clave}: lineas que se retiran / se quedan"] = f"{ret} / {queda}"
        MEDIDO[f"OpenDockTrading, {clave}: retiradas solas / con sustitucion"] = f"{auto} / {sust}"
        if caps is CAP_2_0:
            MEDIDO["OpenDockTrading: lineas propias que se quedan"] = str(prop)
    # lo que nombra TaskCode fuera de tareas/ y no esta clasificado en F1-11
    r = git(copia, "grep", "-n", "-I", "-E", r"\btaskctl\b|TaskCode|taskcode", FIJO_ODT, "--", ".", ":!tareas", check=False)
    por_fichero = {}
    for l in r.stdout.decode("utf-8", "replace").splitlines():
        _, ruta, num, _ = l.split(":", 3)
        por_fichero.setdefault(ruta, []).append(int(num))
    clasificados = {p[0] for p in PARCHES_ODT} | {".taskcode/config.yml"}
    sin = {k: v for k, v in sorted(por_fichero.items()) if k not in clasificados}
    print(f"-- ficheros fuera de tareas/ que nombran taskctl o TaskCode: {len(por_fichero)} ; sin clasificar en F1-11: {len(sin)}")
    for k, v in sin.items():
        print(f"   {k}: {len(v)} lineas ({', '.join(map(str, v[:6]))}{'...' if len(v) > 6 else ''})")
    MEDIDO["ficheros fuera de tareas/ que nombran taskctl o TaskCode: total / sin clasificar"] = f"{len(por_fichero)} / {len(sin)}"


# ---------------------------------------------------------------------------
# G: el ADR
# ---------------------------------------------------------------------------

def entre_marcadores(texto, nombre):
    m = re.search(rf"<!-- {nombre}:inicio -->(.*?)<!-- {nombre}:fin -->", texto, re.S)
    if not m:
        raise SystemExit(f"ERROR: el ADR no tiene el bloque {nombre}")
    return m.group(1)


def filas_tabla(bloque):
    filas = []
    for l in bloque.strip().splitlines():
        l = l.strip()
        if not l.startswith("|") or re.match(r"^\|[-| ]+\|$", l):
            continue
        filas.append([c.strip() for c in l.strip("|").split("|")])
    return filas[0], filas[1:]


def errores_adr(adr, ruta, repo_tc2, medido, con_indice=True):
    """Lista de errores del ADR frente a lo medido (funcion pura: la usan G y sus mutaciones)."""
    err = []
    try:
        _, filas = filas_tabla(entre_marcadores(adr, "cifras"))
    except SystemExit as e:
        return [str(e)]
    tabla = {}
    for f in filas:
        if f[0] in tabla:
            err.append(f"fila repetida en la tabla: {f[0]}")
        tabla[f[0]] = f[1].strip("`")
    for k, v in sorted(medido.items()):
        if k not in tabla:
            err.append(f"cifra medida que falta en la tabla: {k} = {v}")
        elif tabla[k] != v:
            err.append(f"{k}: el ADR dice {tabla[k]!r} y lo medido es {v!r}")
    for k in tabla:
        if k not in medido:
            err.append(f"fila de la tabla sin medida: {k}")
    try:
        res = entre_marcadores(adr, "resumen")
    except SystemExit as e:
        return err + [str(e)]
    valores = set(tabla.values()) | {p for v in tabla.values() for p in re.split(r"\s*(?:/|->|y|de)\s*", v)}
    for b in re.findall(r"\*\*([^*]+)\*\*", res):
        if re.search(r"\d", b) and b not in valores:
            err.append(f"el resumen cita {b!r}, que no es un valor de la tabla")
    secciones = ["## Contexto", "## Alternativas", "## Pruebas de concepto", "## Decisión", "## Requisitos y evidencia",
                 "## Consecuencias", "## Fuera de alcance", "## Preguntas abiertas", "## No verificado",
                 "## Divergencias"]
    faltan = [s for s in secciones if not re.search(rf"^{re.escape(s)}", adr, re.M)]
    if faltan:
        err.append(f"faltan secciones {faltan}")
    req = open(os.path.join(repo_tc2, "docs", "plan", "REQUISITOS-FUNCIONALES.md"), encoding="utf-8").read()
    plan = open(os.path.join(repo_tc2, "docs", "plan", "PLAN-TASKCODE-2.md"), encoding="utf-8").read()
    aud = "".join(open(os.path.join(repo_tc2, "docs", "auditoria", x), encoding="utf-8").read()
                  for x in sorted(os.listdir(os.path.join(repo_tc2, "docs", "auditoria"))))
    ids_r = sorted(set(re.findall(r"\b(R[RFN]-\d+)\b", adr)))
    ids_f = sorted(set(re.findall(r"\b(F\d+-\d+[a-z]?)\b", adr)))
    ids_h = sorted(set(re.findall(r"\b(H\d+)\b", adr)))
    malos = [i for i in ids_r if not re.search(rf"\|\s*{i}\s*\|", req)]
    malos += [i for i in ids_f if not re.search(rf"\|\s*{i}\s*\|", plan)]
    malos += [i for i in ids_h if not re.search(rf"\b{i}\b", aud)]
    if malos:
        err.append(f"IDs que no existen: {malos}")
    enlaces = re.findall(r"\]\(([^)]+)\)", adr)
    prohibidos = [e for e in enlaces if "tareas/" in e or "OpenDockTrading" in e or "ToolsaWeb" in e
                  or e.startswith("../../")]
    if prohibidos:
        err.append(f"enlaces prohibidos {prohibidos}")
    for e in enlaces:
        if not e.startswith("http") and not os.path.exists(os.path.join(os.path.dirname(ruta), e.split("#")[0])):
            err.append(f"enlace que no resuelve: {e}")
    if con_indice:
        idx = open(os.path.join(repo_tc2, "docs", "adr", "README.md"), encoding="utf-8").read()
        if "[F1-12](F1-12-actualizacion-entre-versiones.md)" not in idx:
            err.append("falta la fila en docs/adr/README.md")
    return err, tabla, ids_r, ids_f, ids_h, enlaces


# Mutaciones de control del propio ADR: cada copia mutada tiene que fallar por el motivo esperado.
def mutaciones(adr):
    fila_split = "| subtree split: commits de la rama de estado | `1151` | M |"
    return [
        ("cifra-cambiada", adr.replace(fila_split, fila_split.replace("`1151`", "`1150`")),
         "subtree split: commits de la rama de estado: el ADR dice"),
        ("fila-borrada", adr.replace(fila_split + "\n", ""),
         "cifra medida que falta en la tabla: subtree split: commits de la rama de estado"),
        ("fila-sin-medida", adr.replace("<!-- cifras:fin -->", "| cifra inventada | `1` | M |\n<!-- cifras:fin -->"),
         "fila de la tabla sin medida: cifra inventada"),
        ("tiempo-de-reloj-en-la-tabla", adr.replace("<!-- cifras:fin -->",
                                                     "| subtree split: tiempo en OpenDockTrading | `menos de 600 s` | M |\n"
                                                     "<!-- cifras:fin -->"),
         "fila de la tabla sin medida: subtree split: tiempo en OpenDockTrading"),
        ("resumen-inventado", adr.replace("<!-- resumen:fin -->", "Cifra suelta: **9999**.\n<!-- resumen:fin -->"),
         "el resumen cita '9999'"),
        ("enlace-a-tareas", adr.replace("## Contexto", "## Contexto\n\n[x](../../tareas/x.md)", 1),
         "enlaces prohibidos"),
        ("id-inexistente", adr.replace("## Contexto", "## Contexto\n\nVer F7-99.", 1), "IDs que no existen: ['F7-99']"),
        ("seccion-borrada", adr.replace("## No verificado", "## Sin verificar"), "faltan secciones"),
    ]


def seccion_g(repo_tc2):
    sep("G: el ADR (cifras, resumen, secciones, IDs, enlaces e indice) y sus mutaciones de control")
    ruta = os.path.join(repo_tc2, "docs", "adr", "F1-12-actualizacion-entre-versiones.md")
    if not os.path.exists(ruta):
        falla("G", "no existe el ADR")
        return
    adr = open(ruta, encoding="utf-8").read()
    # nota de nombres: lineas de F1-1..F1-11 que nombran taskctl, taskcode/tareas o .git/taskcode
    for n in range(1, 12):
        f = [x for x in os.listdir(os.path.join(repo_tc2, "docs", "adr")) if x.startswith(f"F1-{n}-")]
        t = open(os.path.join(repo_tc2, "docs", "adr", f[0]), encoding="utf-8").read().splitlines()
        a = sum(1 for l in t if re.search(r"\btaskctl\b", l))
        b = sum(1 for l in t if "taskcode/tareas" in l)
        c = sum(1 for l in t if ".git/taskcode" in l)
        MEDIDO[f"F1-{n}: lineas con taskctl / taskcode/tareas / .git/taskcode"] = f"{a} / {b} / {c}"
    r = errores_adr(adr, ruta, repo_tc2, MEDIDO)
    if isinstance(r, list):
        for e in r:
            falla("G", e)
        return
    err, tabla, ids_r, ids_f, ids_h, enlaces = r
    print(f"cifras en la tabla del ADR: {len(tabla)} ; medidas: {len(MEDIDO)}")
    print(f"IDs citados: requisitos {len(ids_r)} ; tareas del plan {len(ids_f)} ; auditoria {len(ids_h)} ; enlaces: {len(enlaces)}")
    for e in err:
        falla("G", e)
    print(f"errores del ADR: {len(err)}")
    for nombre, mutado, esperado in mutaciones(adr):
        if mutado == adr:
            falla("G", f"la mutacion {nombre} no cambia el ADR")
            continue
        rm = errores_adr(mutado, ruta, repo_tc2, MEDIDO)
        errs = rm if isinstance(rm, list) else rm[0]
        ok = any(esperado in e for e in errs)
        print(f"  mutacion {nombre}: {'falla como se espera' if ok else 'NO FALLA'} "
              f"({str(next((e for e in errs if esperado in e), errs[:1]))[:110]})")
        if not ok:
            falla("G", f"la mutacion {nombre} no la detecta el guion")


MARCA = ".poc-f1-12"


def comprobar_trabajo(trabajo):
    """<...>/pruebas-claude/<proyecto>/<motivo>[/...]: dos niveles como minimo bajo pruebas-claude, y la carpeta
    no existe, esta vacia o lleva la marca de esta PoC. Si no, no se borra nada."""
    base = os.environ.get("LOCALAPPDATA")
    if not base:
        raise SystemExit("ERROR: falta LOCALAPPDATA: la carpeta de trabajo va en %LOCALAPPDATA%/pruebas-claude/<proyecto>/<motivo>")
    raiz = os.path.normcase(os.path.abspath(os.path.join(base, "pruebas-claude")))
    if not os.path.normcase(os.path.abspath(trabajo)).startswith(raiz + os.sep):
        raise SystemExit(f"ERROR: {trabajo} no esta bajo %LOCALAPPDATA%/pruebas-claude ({raiz})")
    partes = trabajo.replace("\\", "/").rstrip("/").split("/")
    if "pruebas-claude" not in partes:
        raise SystemExit("ERROR: la carpeta de trabajo tiene que estar bajo pruebas-claude/<proyecto>/<motivo>")
    debajo = partes[len(partes) - partes[::-1].index("pruebas-claude"):]
    if len(debajo) < 2:
        raise SystemExit(f"ERROR: {trabajo}: hacen falta dos niveles bajo pruebas-claude (<proyecto>/<motivo>)")
    if os.path.lexists(trabajo):
        if not os.path.isdir(trabajo):
            raise SystemExit(f"ERROR: {trabajo} existe y no es una carpeta")
        if os.listdir(trabajo) and not os.path.exists(os.path.join(trabajo, MARCA)):
            raise SystemExit(f"ERROR: {trabajo} no esta vacia ni lleva la marca {MARCA}: no se borra")


def main():
    args = sys.argv[1:]
    cache = None
    if "--cache-split" in args:
        i = args.index("--cache-split")
        cache = os.path.abspath(args[i + 1])
        del args[i:i + 2]
    if len(args) != 4:
        raise SystemExit(__doc__)
    trabajo, repo_tc2, personal, plugin = (os.path.abspath(a) for a in args)
    sys.stdout.reconfigure(encoding="utf-8", newline="\n")
    comprobar_trabajo(trabajo)
    borrar(trabajo)
    os.makedirs(trabajo)
    open(os.path.join(trabajo, MARCA), "w", encoding="utf-8").write("carpeta de trabajo de poc-f1-12.py\n")
    print(f"PoC F1-12 ; OpenDockTrading {FIJO_ODT[:9]} ; Python {sys.version.split()[0]} ; PyYAML {yaml.__version__} ; "
          f"{git(trabajo, 'version').strip()} ; node {run(['node', '--version']).stdout.decode().strip()}")
    seccion_n()
    tiempo("N")
    copia, remoto, tareas = seccion_m(personal, trabajo, cache)
    tiempo("M")
    seccion_c(plugin, copia)
    tiempo("C")
    seccion_a(copia, tareas)
    seccion_g(repo_tc2)
    print()
    if FALLOS:
        print(f"RESULTADO: {len(FALLOS)} fallos")
        for f in FALLOS:
            print(f"  - {f}")
        sys.exit(1)
    print("RESULTADO: todo como se espera")


if __name__ == "__main__":
    main()

"""PoC de F1-13 (TASK-015): adaptador de plataforma GitHub y GitLab.

Se ejecuta con `python -I -B`. GitHub y GitLab SOLO SE LEEN: toda llamada sale
por un envoltorio unico (`Lector`) que rechaza cualquier metodo distinto de GET
ANTES de tocar la red o lanzar `gh`, y registra metodo, URL, estado y hash del
cuerpo. No se crea nada en ninguna plataforma.

(I) Inventario: gh, glab y Python (version y sesion de gh, sin token).
(N) Pruebas negativas del envoltorio: un POST y un DELETE rechazados sin
    salir a la red (contadores de transporte y del mock a 0).
(R) GET reales: GitHub con `gh api --method GET --include` sobre
    charliebk/TaskCodeTwo (publico) y charliebk/taskcodetwo-poc-f1-1
    (privado, cuenta gratuita); GitLab.com anonimo sobre gitlab-org/gitlab
    (proyecto 278964) con un MR y un commit fijos. En el log, por llamada,
    solo lo ESTABLE (estado HTTP y presencia o valor de campos que no
    cambian); el cuerpo va a la carpeta de trabajo y su hash a la seccion V.
    Lo estable se compara con fixtures/observado.json (deriva = fallo).
(M) Mock en 127.0.0.1 con los fixtures de fixtures/{github,gitlab}: un
    adaptador minimo por plataforma devuelve el MISMO EstadoPuertas neutral
    en la misma bateria (verde, esperando 7, sin aprobacion, aprobacion de
    bot que no cuenta, rechazada motivo=plan, sin-cambios); se cuentan las
    llamadas por operacion; los campos de cada fixture se contrastan con los
    observados donde hubo respuesta real.
(C) `capacidades` y la salida de `doctor` (5 lineas como mucho) por plan,
    leyendo la tabla de equivalencias del ADR como dato.
(G) El ADR: tabla completa (20 garantias, URL y fecha por celda de
    plataforma, tier igual al de fuentes.md y al esperado, filas «empirica»
    con id de llamada del log), fuentes.md (fecha y cita de menos de 15
    palabras), IDs, enlaces, secciones, cifras de la prosa por contexto y
    copias mutadas que fallan cada una por su motivo.
(V) Lo VARIABLE (hash y tamano de cada cuerpo, cabeceras de limite): al
    final, fuera de lo comprobado. Los tiempos de reloj van a stderr.

Uso: python -I -B poc-f1-13.py <trabajo> <repo-TaskCodeTwo>
     python -I -B poc-f1-13.py --solo-adr <trabajo> <repo-TaskCodeTwo>
       (sin red: N, M, C y G con fixtures/observado.json; para iterar el ADR)
     --grabar-observado reescribe fixtures/observado.json con lo observado.
Sale con 0 solo si todo sale como se espera. <trabajo> tiene que estar bajo
%LOCALAPPDATA%/pruebas-claude/<proyecto>/<motivo>; se borra al empezar y al
terminar.
"""
import hashlib
import http.server
import json
import os
import re
import shutil
import subprocess
import sys
import threading
import time
import urllib.error
import urllib.request

GH = r"C:\Program Files\GitHub CLI\gh.exe"
GLAB = os.path.join(os.environ.get("LOCALAPPDATA", ""), "Programs", "glab", "glab.exe")
GH_REPO = "charliebk/TaskCodeTwo"                       # publico, cuenta personal gratuita
GH_PRIV = "charliebk/taskcodetwo-poc-f1-1"              # privado, cuenta personal gratuita (PoC de F1-5)
GH_SHA = "e8ee970802d968a3c8ae0599bc1e7ad62f69f8bc"      # develop en el remoto el 2026-11-14
GH_SHA_REN = "03f0c42464c6b5bd2e9d3cc46a0e51f4629e0a15"  # commit con 4 renombrados (cierre de TASK-001)
GL = "https://gitlab.com/api/v4"
GL_P = 278964                                           # gitlab-org/gitlab (publico)
GL_MR = 271075                                          # MR mergeado el 2026-11-14
GL_SHA = "e5b0c3022386a560e8377e9cd057dc883316cfeb"      # head de ese MR
GL_USR = 26073976                                       # un aprobador de ese MR (cuenta bot)
MARCA = ".poc-f1-13"
POC = os.path.dirname(os.path.abspath(__file__))
FIX = os.path.join(POC, "fixtures")
OBSERVADO = os.path.join(FIX, "observado.json")
FUENTES = os.path.join(POC, "fuentes.md")
ADR_REL = os.path.join("docs", "adr", "F1-13-adaptador-de-plataforma.md")
FILA_INDICE = "| [F1-13](F1-13-adaptador-de-plataforma.md) |"
CASOS = ["verde", "esperando-checks", "sin-aprobacion", "aprobacion-bot", "plan", "sin-cambios"]
ESPERADO_BATERIA = {
    "verde": ("estado-puertas ok pr=12 aprobaciones=1 ignoradas=0", 0),
    "esperando-checks": ("estado-puertas esperando pr=12 aprobaciones=1 ignoradas=0 falta=revision-ia", 7),
    "sin-aprobacion": ("estado-puertas esperando pr=12 aprobaciones=0 ignoradas=0 falta=aprobacion", 7),
    "aprobacion-bot": ("estado-puertas esperando pr=12 aprobaciones=0 ignoradas=1 falta=aprobacion", 7),
    "plan": ("estado-puertas rechazada pr=12 motivo=plan garantia=G03", 3),
    "sin-cambios": ("estado-puertas sin-cambios pr=12 estado=mergeado", 0),
}
PERSONAS = {1001}          # lista de personas por id (F1-6, decision 3)
ACTIONS = 15368            # integration_id de GitHub Actions (F1-5; GH-10 lo vuelve a leer)
APP_REVISOR = 99001        # App del revisor (relleno de fixture)
BOT_REVISOR_GL = 2002      # bot o cuenta de servicio del revisor en GitLab (relleno de fixture)
CHECKS = {"verificacion": ACTIONS, "carril": ACTIONS, "revision-ia": APP_REVISOR}

T_RS = "Free público; Pro/Team/Enterprise"
T_TODOS = "todos"
T_COLA = "org pública; Enterprise Cloud"
# Lo que dice la documentacion (fuentes.md) por garantia: (tier GitHub, sustituto GitHub, tier GitLab, sustituto
# GitLab). El ADR no puede decir otra cosa sin que G lo vea.
ESPERADO_TABLA = {
    "G01": (T_RS, None, "ninguno", "maintainer-solo-persona"),
    "G02": (T_RS, None, "Free", None),
    "G03": (T_RS, None, "Premium", "merge-solo-persona"),
    "G04": (T_RS, None, "Premium", "merge-solo-persona"),
    "G05": (T_RS, None, "Premium", "merge-solo-persona"),
    "G06": (T_RS, None, "Premium", "carril-propietarios"),
    "G07": (T_RS, None, "Free", None),
    "G08": (T_RS, None, "ninguno", "al-dia-por-cli"),
    "G09": (T_RS, None, "ninguno", "autor-del-estado"),
    "G10": (T_COLA, "al-dia-por-cli", "Premium", "al-dia-por-cli"),
    "G11": (T_RS, None, "Free", None),
    "G12": (T_TODOS, None, "Premium", "cuenta-de-servicio"),
    "G13": (T_TODOS, None, "Free", None),
    "G14": (T_TODOS, None, "Free", None),
    "G15": (T_TODOS, None, "Free", None),
    "G16": (T_TODOS, None, "Free", None),
    "G17": (T_TODOS, None, "Free", None),
    "G18": (T_TODOS, None, "Free", None),
    "G19": (T_TODOS, None, "Free", None),
    "G20": (T_TODOS, None, "Free", None),
}
# Que prueba cada llamada y para que filas vale como evidencia empirica (IMP-2 de la revision 1). Una fila
# «empirica» solo puede citar ids de aqui, para una fila de su lista, y el hecho tiene que estar en lo observado
# (un 2xx con el campo, o el 403 con aviso de plan). Un 401/403 anonimo solo prueba que hace falta sesion.
PRUEBA = {
    "GH-03": ({"G01", "G02"}, "200 [] en el publico gratuito: la API de reglas esta y lee", lambda o: o["estado"] == 200),
    "GH-12": ({"G01", "G03"}, "403 con aviso de plan en el privado gratuito: sin rulesets",
              lambda o: o["estado"] == 403 and o["hechos"].get("aviso_de_plan") is True),
    "GH-05": ({"G09", "G17"}, "check-run con app.id (origen) leido por sha",
              lambda o: o["estado"] == 200 and "app" in (o["hechos"].get("claves_check_run") or [])),
    "GH-10": ({"G09"}, "integration_id de Actions", lambda o: o["estado"] == 200 and o["hechos"].get("id") == 15368),
    "GH-07": ({"G14"}, "commit.verification con verified y reason",
              lambda o: o["estado"] == 200 and "verification.verified" in o["hechos"]),
    "GH-09": ({"G16"}, "can_approve_pull_request_reviews leido",
              lambda o: o["estado"] == 200 and o["hechos"].get("can_approve_pull_request_reviews") is False),
    "GH-11": ({"G18"}, "previous_filename en cada renombrado",
              lambda o: o["estado"] == 200 and o["hechos"].get("con_previous_filename", 0) > 0),
    "GL-02": ({"G02"}, "ramas protegidas con push_access_levels",
              lambda o: o["estado"] == 200 and "push_access_levels" in (o["hechos"].get("claves_rama") or [])),
    "GL-03": ({"G01", "G06"}, "rama protegida con allow_force_push y code_owner_approval_required",
              lambda o: o["estado"] == 200 and {"allow_force_push", "code_owner_approval_required"}
              <= set(o["hechos"].get("claves_rama") or [])),
    "GL-08": ({"G08"}, "diverged_commits_count en el MR",
              lambda o: o["estado"] == 200 and "diverged_commits_count" in (o["hechos"].get("claves_mr") or [])),
    "GL-09": ({"G03", "G12", "G15"}, "approved_by con un usuario bot de proyecto y sin campo bot",
              lambda o: o["estado"] == 200 and o["hechos"].get("aprobador_bot_de_proyecto") is True),
    "GL-12": ({"G10"}, "pipeline del MR sobre la ref del tren",
              lambda o: o["estado"] == 200 and o["hechos"].get("ref_de_tren") is True),
    "GL-17": ({"G10"}, "endpoint de trenes en un proyecto de pago", lambda o: o["estado"] == 200),
    "GL-13": ({"G17"}, "pipelines por sha", lambda o: o["estado"] == 200 and o["hechos"].get("es_lista") is True),
    "GL-14": ({"G18"}, "renamed_file, old_path y new_path en los diffs",
              lambda o: o["estado"] == 200 and "renamed_file" in (o["hechos"].get("claves_diff") or [])),
    "GL-15": ({"G14"}, "verification_status y key de la firma",
              lambda o: o["estado"] == 200 and "verification_status" in o["hechos"]),
}

PLANES_GH = [("free", "publico", "usuario"), ("free", "privado", "usuario"), ("pro", "privado", "usuario"),
             ("free", "publico", "org"), ("team", "privado", "org"), ("enterprise", "privado", "org")]
PLANES_GL = ["free", "premium", "ultimate"]

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
        shutil.rmtree(ruta)


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


# --------------------------------------------------------------------------- envoltorio de solo lectura

class SoloLectura(Exception):
    pass


class _SinRedireccion(urllib.request.HTTPRedirectHandler):
    """Una redireccion podria llevar a un host fuera de la lista blanca: no se sigue (queda como 3xx)."""

    def redirect_request(self, *a, **k):
        return None


SIN_REDIRECCIONES = urllib.request.build_opener(_SinRedireccion)
RUTA_GH = re.compile(r"^(?:repos/charliebk/[A-Za-z0-9._-]+(?:[/?][A-Za-z0-9._/?=&-]*)?|apps/github-actions)$")


def destino_permitido(destino, via, mock_url):
    """Lista blanca: gh solo con rutas de repos de charliebk (y la App de Actions), nunca una ruta que empiece por
    '-' (seria una opcion de gh); HTTP solo a la API v4 de GitLab.com para el proyecto publico 278964 y el usuario
    aprobador fijo, y al mock de 127.0.0.1 de esta ejecucion."""
    if via == "gh":
        return not destino.startswith("-") and bool(RUTA_GH.match(destino))
    permitidos = [f"{GL}/projects/{GL_P}", f"{GL}/users/{GL_USR}"] + ([mock_url] if mock_url else [])
    return any(destino == p or destino.startswith(p + "/") or destino.startswith(p + "?") for p in permitidos)


class Lector:
    """Unico punto de salida a GitHub, GitLab y el mock. Solo GET; el metodo se comprueba antes de todo."""

    def __init__(self, trabajo=None, mock_url=None):
        self.trabajo = trabajo
        self.mock_url = mock_url    # unico origen local admitido (el del mock de esta ejecucion)
        self.registro = []          # (id, metodo, url, estado, sha256, bytes)
        self.transporte = 0         # peticiones HTTP que salieron
        self.procesos_gh = 0        # procesos gh lanzados
        self.rechazos = 0

    def _guardia(self, metodo, destino, via):
        """Metodo GET exacto y destino en la lista blanca, comprobados antes de abrir red o lanzar gh."""
        if metodo != "GET":
            self.rechazos += 1
            raise SoloLectura(f"metodo {metodo} rechazado antes de la red: {destino}")
        if not destino_permitido(destino, via, self.mock_url):
            self.rechazos += 1
            raise SoloLectura(f"destino fuera de la lista blanca ({via}): {destino}")

    def _anotar(self, ident, url, estado, cuerpo):
        h = hashlib.sha256(cuerpo).hexdigest()
        self.registro.append((ident, "GET", url, estado, h, len(cuerpo)))
        if self.trabajo and ident:
            with open(os.path.join(self.trabajo, "cuerpos", ident + ".json"), "wb") as f:
                f.write(cuerpo)

    def http(self, ident, metodo, url):
        self._guardia(metodo, url, "http")
        req = urllib.request.Request(url, method="GET", headers={"User-Agent": "taskcodetwo-poc-f1-13",
                                                                  "Accept": "application/json"})
        self.transporte += 1
        cab = {}
        try:
            with SIN_REDIRECCIONES.open(req, timeout=60) as r:
                estado, cuerpo, cab = r.status, r.read(), dict(r.headers)
        except urllib.error.HTTPError as e:
            estado, cuerpo, cab = e.code, e.read(), dict(e.headers)
        self._anotar(ident, url, estado, cuerpo)
        return estado, cuerpo, cab

    def gh(self, ident, metodo, ruta):
        self._guardia(metodo, ruta, "gh")
        self.procesos_gh += 1
        r = subprocess.run([GH, "api", "--method", "GET", "--include", ruta], capture_output=True, timeout=120)
        salida = r.stdout.replace(b"\r\n", b"\n")
        cab_txt, _, cuerpo = salida.partition(b"\n\n")
        m = re.match(rb"HTTP/\S+ (\d{3})", cab_txt)
        if not m:
            raise SystemExit(f"ERROR: gh api {ruta}: sin linea de estado ({r.stderr.decode('utf-8', 'replace')[-300:]})")
        cab = {}
        for l in cab_txt.split(b"\n")[1:]:
            k, _, v = l.decode("utf-8", "replace").partition(":")
            cab[k.strip()] = v.strip()
        self._anotar(ident, "https://api.github.com/" + ruta, int(m.group(1)), cuerpo)
        return int(m.group(1)), cuerpo, cab


# --------------------------------------------------------------------------- I

def seccion_i():
    sep("I: inventario (where, --version y sesion de gh sin token)")
    for nombre, cmd in (("gh", [GH, "--version"]), ("glab", [GLAB, "--version"])):
        try:
            r = subprocess.run(cmd, capture_output=True, timeout=60)
            linea = (r.stdout or r.stderr).decode("utf-8", "replace").strip().splitlines()[0]
        except (OSError, subprocess.TimeoutExpired, IndexError) as e:
            linea = f"no se ejecuta ({type(e).__name__})"
        print(f"{nombre}: {linea}")
    print(f"python: {sys.version.split()[0]}")
    r = subprocess.run([GH, "auth", "status"], capture_output=True, timeout=60)
    texto = (r.stdout + r.stderr).decode("utf-8", "replace")
    print(f"gh auth status: sesion={'si' if 'Logged in to github.com' in texto else 'no'} ; "
          f"ambitos incluyen escritura (repo)={'si' if re.search(r'scopes:.*repo', texto) else 'no'} "
          f"(la lectura la garantiza el envoltorio, no el token)")
    print("glab: no se usa (GET anonimos con urllib por el envoltorio)")


# --------------------------------------------------------------------------- N

def seccion_n(mock):
    sep("N: el envoltorio rechaza, antes de la red, lo que no es GET o no va a un destino de la lista blanca")
    lector = Lector(mock_url=mock.url)
    antes = (lector.transporte, lector.procesos_gh, mock.contador["total"])
    pruebas = [("NEG-01", "POST", lambda: lector.http("NEG-01", "POST", mock.url + "/gh/verde/repos/o/r/pulls")),
               ("NEG-02", "DELETE", lambda: lector.gh("NEG-02", "DELETE", f"repos/{GH_REPO}/git/refs/heads/x")),
               ("NEG-03", "PUT", lambda: lector.http("NEG-03", "PUT", f"{GL}/projects/{GL_P}/merge_requests/1/merge")),
               ("NEG-04", "get", lambda: lector.http("NEG-04", "get", mock.url + "/gh/verde/repos/o/r/pulls/12")),
               ("NEG-05", "GET a otro host", lambda: lector.http("NEG-05", "GET", "https://example.com/api/v4/projects/278964")),
               ("NEG-06", "GET a otro proyecto de GitLab", lambda: lector.http("NEG-06", "GET", f"{GL}/projects/1")),
               ("NEG-07", "GET con prefijo enganoso", lambda: lector.http("NEG-07", "GET", f"{GL}/projects/{GL_P}9")),
               ("NEG-08", "gh GET a un repo ajeno", lambda: lector.gh("NEG-08", "GET", "repos/otra-cuenta/repo")),
               ("NEG-09", "gh GET con una ruta que es una opcion", lambda: lector.gh("NEG-09", "GET", "--method=POST")),
               ("NEG-10", "GET a otro puerto local", lambda: lector.http("NEG-10", "GET", "http://127.0.0.1:1/gh/verde/x")),
               ("NEG-11", "gh GET fuera de repos/", lambda: lector.gh("NEG-11", "GET", "user/repos"))]
    for ident, metodo, f in pruebas:
        try:
            f()
            falla("N", f"{ident} {metodo} no se rechazo")
        except SoloLectura:
            print(f"{ident} {metodo}: rechazado por el envoltorio")
    despues = (lector.transporte, lector.procesos_gh, mock.contador["total"])
    print(f"peticiones HTTP que salieron: {despues[0] - antes[0]} ; procesos gh: {despues[1] - antes[1]} ; "
          f"peticiones que llegaron al mock: {despues[2] - antes[2]} ; rechazos: {lector.rechazos}")
    if despues != antes or lector.rechazos != len(pruebas) or lector.registro:
        falla("N", "algo salio a la red o se registro con un metodo distinto de GET")


# --------------------------------------------------------------------------- R

def _j(c):
    try:
        return json.loads(c.decode("utf-8"))
    except ValueError:
        return None


def _claves(x):
    return sorted(x.keys()) if isinstance(x, dict) else None


def hechos_gh(ident, estado, d):
    h = {}
    if ident == "GH-01":
        h = {"visibility": d["visibility"], "owner.type": d["owner"]["type"]}
    elif ident in ("GH-02", "GH-03"):
        h = {"lista_vacia": d == []}
    elif ident in ("GH-04", "GH-14", "GH-15"):
        h = {"message": d.get("message")}
    elif ident == "GH-05":
        runs = d["check_runs"]
        h = {"total_count": d["total_count"], "runs": sorted(f"{r['name']}<-{r['app']['id']}:{r['conclusion']}" for r in runs),
             "claves_check_run": _claves(runs[0]) if runs else None}
    elif ident == "GH-06":
        h = {"state": d["state"], "total_count": d["total_count"]}
    elif ident == "GH-07":
        h = {"verification.verified": d["commit"]["verification"]["verified"],
             "verification.reason": d["commit"]["verification"]["reason"]}
    elif ident == "GH-08":
        h = {"prs": len(d)}
    elif ident == "GH-09":
        h = dict(sorted(d.items()))
    elif ident == "GH-10":
        h = {"id": d["id"]}
    elif ident == "GH-11":
        ren = [f for f in d["files"] if f["status"] == "renamed"]
        h = {"renombrados": len(ren), "con_previous_filename": sum("previous_filename" in f for f in ren)}
    elif ident in ("GH-12", "GH-13"):
        h = {"aviso_de_plan": "Upgrade to GitHub Pro" in (d or {}).get("message", "")}
    return h


def hechos_gl(ident, estado, d):
    if estado != 200:
        return {}
    if ident == "GL-01":
        return {"visibility": d["visibility"], "merge_method_visible_anonimo": d.get("merge_method") is not None,
                "only_allow_merge_if_pipeline_succeeds_visible_anonimo":
                    d.get("only_allow_merge_if_pipeline_succeeds") is not None}
    if ident == "GL-02":
        return {"es_lista": isinstance(d, list), "claves_rama": _claves(d[0]) if d else None}
    if ident == "GL-03":
        return {"claves_rama": _claves(d), "allow_force_push": d["allow_force_push"]}
    if ident == "GL-08":
        return {"state": d["state"], "claves_mr": _claves(d)}
    if ident == "GL-09":
        ab = d["approved_by"]
        return {"claves": _claves(d), "aprobador_bot_de_proyecto": any(
            re.fullmatch(r"project_\d+_bot_\w+", a["user"]["username"]) for a in ab),
                "campo_bot_en_aprobador": any("bot" in a["user"] for a in ab),
                "claves_usuario_aprobador": _claves(ab[0]["user"]) if ab else None}
    if ident == "GL-10":
        return {"claves": _claves(d), "rule_types": sorted({r["rule_type"] for r in d["rules"]})}
    if ident == "GL-11":
        return {"es_lista": isinstance(d, list)}
    if ident == "GL-12":
        return {"sources": sorted({p["source"] for p in d}), "ref_de_tren": any(p["ref"].endswith("/train") for p in d)}
    if ident == "GL-13":
        return {"es_lista": isinstance(d, list), "claves_pipeline": _claves(d[0]) if d else None}
    if ident == "GL-14":
        return {"claves_diff": _claves(d[0]) if d else None}
    if ident == "GL-15":
        return {"signature_type": d["signature_type"], "verification_status": d["verification_status"],
                "claves": _claves(d), "claves_key": _claves(d.get("key"))}
    if ident == "GL-17":
        return {"es_lista": isinstance(d, list)}
    return {}


def llamadas_reales():
    g = f"repos/{GH_REPO}"
    p = f"{GL}/projects/{GL_P}"
    m = f"{p}/merge_requests/{GL_MR}"
    return [
        ("GH-01", "gh", g, "repo: visibilidad y tipo de cuenta"),
        ("GH-02", "gh", f"{g}/rulesets", "rulesets del repo"),
        ("GH-03", "gh", f"{g}/rules/branches/develop", "reglas efectivas de develop"),
        ("GH-04", "gh", f"{g}/branches/develop/protection", "proteccion clasica de develop"),
        ("GH-05", "gh", f"{g}/commits/{GH_SHA}/check-runs", "check-runs del commit de develop"),
        ("GH-06", "gh", f"{g}/commits/{GH_SHA}/status", "estado combinado (statuses) del commit"),
        ("GH-07", "gh", f"{g}/commits/{GH_SHA}", "firma del commit (verification)"),
        ("GH-08", "gh", f"{g}/pulls?state=all&per_page=5", "PR existentes"),
        ("GH-09", "gh", f"{g}/actions/permissions/workflow", "permisos de GITHUB_TOKEN"),
        ("GH-10", "gh", "apps/github-actions", "integration_id de GitHub Actions"),
        ("GH-11", "gh", f"repos/{GH_REPO}/commits/{GH_SHA_REN}", "ficheros renombrados de un commit"),
        ("GH-12", "gh", f"repos/{GH_PRIV}/rulesets", "rulesets en el privado gratuito"),
        ("GH-13", "gh", f"repos/{GH_PRIV}/branches/develop/protection", "proteccion en el privado gratuito"),
        ("GH-14", "gh", f"{g}/codeowners/errors", "CODEOWNERS"),
        ("GH-15", "gh", f"{g}/pulls/1/reviews", "reviews del PR 1 (no hay PR)"),
        ("GL-01", "http", p, "proyecto"),
        ("GL-02", "http", f"{p}/protected_branches", "ramas protegidas"),
        ("GL-03", "http", f"{p}/protected_branches/master", "rama protegida master"),
        ("GL-04", "http", f"{p}/protected_tags", "tags protegidos"),
        ("GL-05", "http", f"{p}/approval_rules", "reglas de aprobacion del proyecto"),
        ("GL-06", "http", f"{p}/approvals", "ajustes de aprobacion del proyecto"),
        ("GL-07", "http", f"{p}/push_rule", "push rules"),
        ("GL-08", "http", f"{m}?include_diverged_commits_count=true", "MR con commits por detras"),
        ("GL-09", "http", f"{m}/approvals", "aprobaciones del MR"),
        ("GL-10", "http", f"{m}/approval_state", "estado de las reglas de aprobacion del MR"),
        ("GL-11", "http", f"{m}/approval_rules", "reglas de aprobacion del MR"),
        ("GL-12", "http", f"{m}/pipelines", "pipelines del MR"),
        ("GL-13", "http", f"{p}/pipelines?sha={GL_SHA}", "pipelines por sha"),
        ("GL-14", "http", f"{m}/diffs?per_page=5", "diffs del MR"),
        ("GL-15", "http", f"{p}/repository/commits/{GL_SHA}/signature", "firma del commit"),
        ("GL-16", "http", f"{p}/repository/commits/{GL_SHA}/statuses", "estados del commit"),
        ("GL-17", "http", f"{p}/merge_trains", "trenes de merge"),
        ("GL-18", "http", f"{p}/external_status_checks", "checks de estado externos"),
        ("GL-19", "http", f"{m}/status_checks", "checks externos del MR"),
        ("GL-20", "http", f"{GL}/users/{GL_USR}", "usuario aprobador (bot)"),
        ("GL-21", "http", f"{p}/runners", "runners del proyecto"),
        ("GL-22", "http", f"{p}/variables", "variables de CI"),
        ("GL-23", "http", f"{p}/access_tokens", "tokens de proyecto"),
    ]


def seccion_r(trabajo, grabar):
    sep("R: GET reales (solo lo estable: estado HTTP y campos; el cuerpo en la carpeta de trabajo)")
    lector = Lector(trabajo)    # sin mock: solo GitHub (charliebk/*) y GitLab.com (proyecto 278964)
    os.makedirs(os.path.join(trabajo, "cuerpos"))
    observado, variable = {}, []
    for ident, via, destino, que in llamadas_reales():
        if via == "gh":
            estado, cuerpo, cab = lector.gh(ident, "GET", destino)
            url = "https://api.github.com/" + destino
            hechos = hechos_gh(ident, estado, _j(cuerpo) if cuerpo else None)
            lim = f"x-ratelimit-remaining={ {k.lower(): v for k, v in cab.items()}.get('x-ratelimit-remaining', '?')}"
        else:
            estado, cuerpo, cab = lector.http(ident, "GET", destino)
            url = destino
            hechos = hechos_gl(ident, estado, _j(cuerpo))
            lim = f"ratelimit-remaining={ {k.lower(): v for k, v in cab.items()}.get('ratelimit-remaining', '?')}"
        observado[ident] = {"url": url, "estado": estado, "hechos": hechos}
        print(f"{ident} GET {url} -> {estado} ; {que}")
        for k, v in hechos.items():
            print(f"    {k} = {json.dumps(v, ensure_ascii=False)}")
        variable.append(f"{ident} sha256={hashlib.sha256(cuerpo).hexdigest()[:16]} bytes={len(cuerpo)} {lim}")
        reloj(f"{ident} {estado}")
    metodos = sorted({r[1] for r in lector.registro})
    print(f"llamadas registradas: {len(lector.registro)} ; metodos en el registro: {metodos} ; "
          f"procesos gh: {lector.procesos_gh} ; HTTP: {lector.transporte} ; rechazos: {lector.rechazos}")
    if metodos != ["GET"] or len(lector.registro) != len(llamadas_reales()):
        falla("R", "el registro no son solo los GET previstos")
    if grabar:
        with open(OBSERVADO, "w", encoding="utf-8", newline="\n") as f:
            json.dump(observado, f, ensure_ascii=False, indent=1, sort_keys=True)
            f.write("\n")
        print("fixtures/observado.json reescrito con lo observado")
    previo = json.load(open(OBSERVADO, encoding="utf-8")) if os.path.exists(OBSERVADO) else {}
    for ident, o in observado.items():
        if previo.get(ident) != json.loads(json.dumps(o)):
            falla("R", f"{ident} distinto de fixtures/observado.json (deriva de la plataforma o del guion)")
    print(f"comparado con fixtures/observado.json: {len(observado)} llamadas")
    return observado, variable


# --------------------------------------------------------------------------- M: mock y adaptadores

class Mock:
    def __init__(self):
        self.fixtures = {}
        for plat in ("github", "gitlab"):
            for caso in CASOS:
                self.fixtures[(plat, caso)] = json.load(open(os.path.join(FIX, plat, caso + ".json"), encoding="utf-8"))
        self.contador = {"total": 0}
        self.peticiones = {}
        mock = self

        class H(http.server.BaseHTTPRequestHandler):
            def log_message(self, *a):
                pass

            def _no(self):
                mock.contador["total"] += 1
                self.send_response(405)
                self.end_headers()

            do_POST = do_PUT = do_PATCH = do_DELETE = _no

            def do_GET(self):
                mock.contador["total"] += 1
                _, pref, caso, resto = self.path.split("/", 3)
                plat = {"gh": "github", "gl": "gitlab"}[pref]
                mock.peticiones.setdefault((plat, caso), []).append("/" + resto)
                r = mock.fixtures[(plat, caso)]["respuestas"].get("/" + resto)
                cuerpo = json.dumps(r["cuerpo"] if r else {"message": "404 no hay fixture"}).encode()
                self.send_response(r["estado"] if r else 404)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(cuerpo)

        self.srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), H)
        self.url = f"http://127.0.0.1:{self.srv.server_address[1]}"
        threading.Thread(target=self.srv.serve_forever, daemon=True).start()

    def parar(self):
        self.srv.shutdown()
        self.srv.server_close()


def _evaluar(n, checks, aprob, ignor, al_dia):
    """EstadoPuertas neutral a partir de lo leido (igual para las dos plataformas)."""
    rojo = sorted(k for k, v in checks.items() if v == "fallo")
    if rojo:
        return f"estado-puertas rechazada pr={n} motivo=check-rojo checks={','.join(rojo)}", 3
    falta = sorted(k for k, v in checks.items() if v != "ok")
    if aprob < 1:
        falta.append("aprobacion")
    if not al_dia:
        falta.append("al-dia")
    linea = f"estado-puertas {'esperando' if falta else 'ok'} pr={n} aprobaciones={aprob} ignoradas={ignor}"
    return (linea + f" falta={','.join(falta)}", 7) if falta else (linea, 0)


def estado_puertas_github(lector, base, n, politica):
    """Lecturas: PR, reglas de la rama base (el 403 de plan se observa aqui), check-runs del head, reviews."""
    _, c, _ = lector.http(None, "GET", f"{base}/repos/o/r/pulls/{n}")
    pr = json.loads(c)
    if pr["merged"]:
        return f"estado-puertas sin-cambios pr={n} estado=mergeado", 0
    e, c, _ = lector.http(None, "GET", f"{base}/repos/o/r/rules/branches/{pr['base']['ref']}")
    plan_da_reglas = not (e == 403 and "Upgrade to GitHub Pro" in json.loads(c).get("message", ""))
    for g in politica["exigir_impuestas"]:
        if not plan_da_reglas and ESPERADO_TABLA[g][0] == T_RS:
            return f"estado-puertas rechazada pr={n} motivo=plan garantia={g}", 3
    _, c, _ = lector.http(None, "GET", f"{base}/repos/o/r/commits/{pr['head']['sha']}/check-runs")
    checks = {k: "pendiente" for k in CHECKS}
    for r in json.loads(c)["check_runs"]:
        if CHECKS.get(r["name"]) != r["app"]["id"]:
            continue            # otro origen: no cuenta (integration_id)
        checks[r["name"]] = ("ok" if r["conclusion"] == "success" else "fallo") if r["status"] == "completed" else "pendiente"
    _, c, _ = lector.http(None, "GET", f"{base}/repos/o/r/pulls/{n}/reviews")
    ultima = {}
    for rv in json.loads(c):
        ultima[rv["user"]["id"]] = rv
    aprob = sum(1 for i, rv in ultima.items() if rv["state"] == "APPROVED" and rv["user"]["type"] == "User" and i in PERSONAS)
    ignor = sum(1 for i, rv in ultima.items() if rv["state"] == "APPROVED") - aprob
    return _evaluar(n, checks, aprob, ignor, pr["mergeable_state"] != "behind")


def estado_puertas_gitlab(lector, base, n, politica):
    """Lecturas: MR (con commits por detras), reglas de aprobacion (403 en Free), rama protegida destino,
    estados del commit (jobs y estados externos con su autor), aprobaciones."""
    p = f"{base}/api/v4/projects/7"
    _, c, _ = lector.http(None, "GET", f"{p}/merge_requests/{n}?include_diverged_commits_count=true")
    mr = json.loads(c)
    if mr["state"] == "merged":
        return f"estado-puertas sin-cambios pr={n} estado=mergeado", 0
    e, _, _ = lector.http(None, "GET", f"{p}/approval_rules")
    plan_premium = e == 200
    for g in politica["exigir_impuestas"]:
        if not plan_premium and ESPERADO_TABLA[g][2] in ("Premium", "Ultimate"):
            return f"estado-puertas rechazada pr={n} motivo=plan garantia={g}", 3
    _, c, _ = lector.http(None, "GET", f"{p}/protected_branches/{mr['target_branch']}")
    rama = json.loads(c)
    if [a["access_level"] for a in rama["merge_access_levels"]] != [40] or rama["allow_force_push"]:
        return f"estado-puertas rechazada pr={n} motivo=config garantia=G01", 3
    _, c, _ = lector.http(None, "GET", f"{p}/repository/commits/{mr['sha']}/statuses")
    checks = {k: "pendiente" for k in CHECKS}
    ultimo = {}
    for s in sorted(json.loads(c), key=lambda s: s["id"]):
        if s["name"] == "revision-ia" and s["author"]["id"] != BOT_REVISOR_GL:
            continue            # sustituto de G09: solo cuenta el estado que publica el bot del revisor
        ultimo[s["name"]] = s["status"]
    for k, v in ultimo.items():
        if k in checks:
            checks[k] = {"success": "ok", "failed": "fallo", "canceled": "fallo"}.get(v, "pendiente")
    _, c, _ = lector.http(None, "GET", f"{p}/merge_requests/{n}/approvals")
    ids = [a["user"]["id"] for a in json.loads(c)["approved_by"]]
    aprob = sum(1 for i in ids if i in PERSONAS)
    return _evaluar(n, checks, aprob, len(ids) - aprob, mr["diverged_commits_count"] == 0)


def contrastar_fixtures(mock, observado):
    """Campos de los fixtures frente a los observados en las respuestas reales equivalentes."""
    print("contraste de campos (fixture subconjunto de lo observado):")
    pares = [("github", "/repos/o/r/commits/" + "a" * 40 + "/check-runs", "GH-05", "claves_check_run",
              lambda b: b["check_runs"][0]),
             ("gitlab", "/api/v4/projects/7/merge_requests/12?include_diverged_commits_count=true", "GL-08", "claves_mr",
              lambda b: b),
             ("gitlab", "/api/v4/projects/7/merge_requests/12/approvals", "GL-09", "claves", lambda b: b),
             ("gitlab", "/api/v4/projects/7/merge_requests/12/approvals", "GL-09", "claves_usuario_aprobador",
              lambda b: b["approved_by"][0]["user"]),
             ("gitlab", "/api/v4/projects/7/protected_branches/develop", "GL-03", "claves_rama", lambda b: b)]
    for plat, ruta, ident, clave, sacar in pares:
        obs = set(observado.get(ident, {}).get("hechos", {}).get(clave) or [])
        fix = set(sacar(mock.fixtures[(plat, "verde")]["respuestas"][ruta]["cuerpo"]).keys())
        sobran = sorted(fix - obs)
        print(f"  {plat} {ident}.{clave}: {'ok' if not sobran else 'SOBRAN ' + str(sobran)}")
        if sobran:
            falla("M", f"el fixture {plat} usa campos no observados en {ident}: {sobran}")
    print("  sin respuesta real que contrastar: GitHub pulls/{n} y reviews (no hay PR en el repo, GH-08), "
          "rules/branches con reglas (GH-03 vacio); GitLab statuses (GL-16: 401 anonimo) y approval_rules "
          "en Free (GL-05: 401 anonimo; el 403 de plan del fixture no se ha observado)")


def seccion_m(mock, observado):
    sep("M: misma bateria contra el mock (127.0.0.1) con los dos adaptadores")
    lector = Lector(mock_url=mock.url)
    llamadas = {}
    for caso in CASOS:
        res = {}
        for plat, f, pref in (("github", estado_puertas_github, "gh"), ("gitlab", estado_puertas_gitlab, "gl")):
            antes = mock.contador["total"]
            linea, cod = f(lector, f"{mock.url}/{pref}/{caso}", 12, mock.fixtures[(plat, caso)]["politica"])
            res[plat] = (linea, cod)
            llamadas[(plat, caso)] = mock.contador["total"] - antes
        igual = res["github"] == res["gitlab"]
        esperado = ESPERADO_BATERIA[caso]
        print(f"{caso}: github [{res['github'][1]}] {res['github'][0]}")
        print(f"{' ' * len(caso)}  gitlab [{res['gitlab'][1]}] {res['gitlab'][0]}")
        print(f"{' ' * len(caso)}  iguales={'si' if igual else 'no'} ; esperado={'si' if res['github'] == esperado else 'no'} ; "
              f"llamadas github={llamadas[('github', caso)]} gitlab={llamadas[('gitlab', caso)]}")
        if not igual or res["github"] != esperado:
            falla("M", f"caso {caso}: {res} (esperado {esperado})")
    metodos = sorted({r[1] for r in lector.registro})
    print(f"peticiones al mock: {lector.transporte} ; metodos: {metodos or ['GET']}")
    contrastar_fixtures(mock, observado)
    return {"estado-puertas GitHub (verde)": llamadas[("github", "verde")],
            "estado-puertas GitLab (verde)": llamadas[("gitlab", "verde")],
            "estado-puertas GitHub (sin-cambios)": llamadas[("github", "sin-cambios")],
            "estado-puertas GitLab (sin-cambios)": llamadas[("gitlab", "sin-cambios")]}


# --------------------------------------------------------------------------- C: capacidades y doctor

def cubre(plataforma, tier, plan):
    if plataforma == "github":
        nivel, vis, prop = plan
        return {T_TODOS: True, T_RS: vis == "publico" or nivel in ("pro", "team", "enterprise"),
                T_COLA: prop == "org" and (vis == "publico" or nivel == "enterprise"), "ninguno": False}[tier]
    return {"Free": True, "Premium": plan in ("premium", "ultimate"), "Ultimate": plan == "ultimate", "ninguno": False}[tier]


def arreglo(plataforma, tier):
    return {T_RS: "repo-publico-o-github-pro", T_COLA: "organizacion-o-enterprise-cloud", "Premium": "gitlab-premium",
            "Ultimate": "gitlab-ultimate", "ninguno": "ninguno-limite-de-la-plataforma"}.get(tier, "-")


def capacidades(plataforma, plan, puertas, tabla):
    """Contrato: (plataforma, plan_observado, puertas leidas) -> lista de Garantia. Lee la tabla del ADR como dato."""
    out = []
    for gid in sorted(tabla):
        fila = tabla[gid]
        tier, sust = (fila["tier_gh"], fila["sust_gh"]) if plataforma == "github" else (fila["tier_gl"], fila["sust_gl"])
        if not puertas.get(gid, True):
            out.append({"id": gid, "estado": "no_disponible", "motivo": "config", "arreglo": "tc2-init"})
        elif cubre(plataforma, tier, plan):
            out.append({"id": gid, "estado": "impuesta", "motivo": "-", "arreglo": "-"})
        else:
            motivo = "plataforma" if tier == "ninguno" else "plan"
            out.append({"id": gid, "estado": f"sustituida({sust})" if sust else "no_disponible", "motivo": motivo,
                        "arreglo": arreglo(plataforma, tier)})
    return out


def doctor(plataforma, plan, gars):
    no = [g for g in gars if g["estado"] != "impuesta"]
    n_sus = sum(g["estado"].startswith("sustituida") for g in no)
    plan_txt = plan if isinstance(plan, str) else "-".join(plan)
    lineas = [f"doctor puertas plataforma={plataforma} plan={plan_txt} impuestas={len(gars) - len(no)} "
              f"sustituidas={n_sus} no-disponibles={len(no) - n_sus}"]
    items = [f"{g['id']} {g['estado'].split('(')[0]} motivo={g['motivo']}"
             + (f" por={g['estado'][11:-1]}" if g["estado"].startswith("sustituida") else "") + f" arreglo={g['arreglo']}"
             for g in no]
    if len(items) > 4:
        lineas += items[:3] + [f"y {len(items) - 3} mas: tc2 doctor --json"]
    else:
        lineas += items
    return lineas


def seccion_c(tabla):
    sep("C: capacidades por plan y salida de doctor (tabla del ADR como dato)")
    cifras = {}
    if len(tabla) != 20:
        falla("C", f"la tabla del ADR tiene {len(tabla)} filas legibles, no 20")
        return cifras
    for plataforma, planes in (("github", PLANES_GH), ("gitlab", PLANES_GL)):
        for plan in planes:
            gars = capacidades(plataforma, plan, {}, tabla)
            lineas = doctor(plataforma, plan, gars)
            plan_txt = plan if isinstance(plan, str) else "-".join(plan)
            print(f"-- {plataforma} {plan_txt}")
            for l in lineas:
                print(f"   {l}")
            if len(lineas) > 5:
                falla("C", f"doctor {plataforma} {plan_txt}: mas de 5 lineas")
            imp = sum(g["estado"] == "impuesta" for g in gars)
            sus = sum(g["estado"].startswith("sustituida") for g in gars)
            cifras[f"{plataforma} {plan_txt}"] = f"{imp} / {sus} / {20 - imp - sus}"
            print(f"   lista completa: " + " ".join(f"{g['id']}={g['estado']}" for g in gars if g["estado"] != "impuesta"))
    for plan in PLANES_GL:      # IMP-1 de la revision 1: ningun plan de GitLab da el origen de un check
        g9 = next(g for g in capacidades("gitlab", plan, {}, tabla) if g["id"] == "G09")
        print(f"-- G09 en GitLab {plan}: {g9['estado']} motivo={g9['motivo']} arreglo={g9['arreglo']}")
        if g9["estado"] == "impuesta" or g9["arreglo"] != "ninguno-limite-de-la-plataforma":
            falla("C", f"G09 en GitLab {plan} sale impuesta o con un arreglo de pago ({g9})")
    gars = capacidades("gitlab", "premium", {"G06": False}, tabla)
    g6 = next(g for g in gars if g["id"] == "G06")
    print(f"-- puertas leidas sin CODEOWNERS en GitLab Premium: G06 {g6['estado']} motivo={g6['motivo']} arreglo={g6['arreglo']}")
    if (g6["estado"], g6["motivo"]) != ("no_disponible", "config"):
        falla("C", "una puerta sin configurar no sale motivo=config")
    return cifras


# --------------------------------------------------------------------------- G: el ADR

SECCIONES = ["## Contexto", "## Alternativas", "## Pruebas de concepto", "## Cómo se ve en GitHub y en GitLab",
             "## Decisión", "## Divergencias con el plan", "## Requisitos y evidencia", "## Consecuencias",
             "## Fuera de alcance", "## No verificado"]
FRASES_OBLIGATORIAS = ["`capacidades`", "`taskcodetwo/tareas`", "`tc2 doctor`", "motivo=plan", "motivo=plataforma",
                       "`EstadoPuertas`"]
DIVERGENCIAS = ["F1-4", "F1-5", "F1-6", "F1-7", "F1-8", "F1-10", "P11", "`taskcodetwo`"]
NO_VERIFICADO = ["self-managed", "sin escribir", "tier de pago"]
URL_FECHA = re.compile(r"\]\((https://[^)\s]+)\),? (\d{4}-\d{2}-\d{2})")


def entre_marcadores(texto, nombre):
    m = re.search(rf"<!-- {nombre}:inicio -->(.*?)<!-- {nombre}:fin -->", texto, re.S)
    return m.group(1) if m else None


def filas_tabla(bloque):
    filas = []
    for l in bloque.strip().splitlines():
        l = l.strip()
        if not l.startswith("|") or re.match(r"^\|[-| ]+\|$", l):
            continue
        filas.append([c.strip() for c in l.strip("|").split(" | ")])
    return filas[1:]


def leer_fuentes(texto):
    """{url: (id, fecha, tier, cita)} y errores de fuentes.md."""
    err, fu = [], {}
    bloque = entre_marcadores(texto, "fuentes")
    if bloque is None:
        return {}, ["fuentes.md no tiene el bloque fuentes"]
    for f in filas_tabla(bloque):
        if len(f) != 5:
            err.append(f"fuentes.md: fila con {len(f)} columnas: {f[:1]}")
            continue
        fid, url, fecha, tier, cita = f
        if url in fu:
            err.append(f"fuentes.md: URL repetida {url}")
        if not re.fullmatch(r"\d{4}-\d{2}-\d{2}", fecha):
            err.append(f"fuentes.md: {fid} sin fecha de consulta")
        palabras = len(cita.strip("«»").split())
        if not cita.startswith("«") or palabras >= 15:
            err.append(f"fuentes.md: {fid} cita de {palabras} palabras (tiene que ser de menos de 15, entre «»)")
        fu[url] = (fid, fecha, tier, cita)
    if not any("github.com/pricing" in u for u in fu) or not any("about.gitlab.com/pricing" in u for u in fu):
        err.append("fuentes.md: faltan las paginas de precios de GitHub y GitLab")
    return fu, err


def _sust(celda):
    m = re.fullmatch(r"GitHub: (.+?) / GitLab: (.+)", celda)
    if not m:
        return None, None, False
    sl = []
    for parte in m.groups():
        if parte.strip() == "—":
            sl.append(None)
        else:
            s = re.match(r"`([a-z0-9-]+)`", parte.strip())
            sl.append(s.group(1) if s else "?")
    return sl[0], sl[1], True


def leer_tabla(adr):
    """{gid: fila} y errores de forma."""
    err, tabla = [], {}
    bloque = entre_marcadores(adr, "tabla-equivalencias")
    if bloque is None:
        return {}, ["el ADR no tiene el bloque tabla-equivalencias"]
    for f in filas_tabla(bloque):
        if len(f) != 9:
            err.append(f"fila con {len(f)} columnas: {f[:1]}")
            continue
        gid, nombre, neutral, c_gh, t_gh, c_gl, t_gl, sust, evid = f
        sgh, sgl, ok = _sust(sust)
        if not ok:
            err.append(f"{gid}: la celda de sustituto no es «GitHub: … / GitLab: …»")
        if gid in tabla:
            err.append(f"fila repetida: {gid}")
        tabla[gid] = {"nombre": nombre, "neutral": neutral, "c_gh": c_gh, "tier_gh": t_gh, "c_gl": c_gl,
                      "tier_gl": t_gl, "sust_gh": sgh, "sust_gl": sgl, "sust": sust, "evid": evid}
    return tabla, err


def errores_tabla(tabla, fuentes, observado):
    err = []
    ids = set(tabla)
    esperados = set(ESPERADO_TABLA)
    for g in sorted(esperados - ids):
        err.append(f"falta la garantia {g}")
    for g in sorted(ids - esperados):
        err.append(f"sobra la garantia {g}")
    for g in sorted(ids & esperados):
        f = tabla[g]
        for u in re.findall(r"\]\((https://[^)\s]+)\)", " ".join(str(v) for v in f.values())):
            if u not in fuentes:
                err.append(f"{g}: URL que no esta en fuentes.md: {u}")
        e_tgh, e_sgh, e_tgl, e_sgl = ESPERADO_TABLA[g]
        for plat, celda, hosts, tier, e_tier, sust, e_sust in (
                ("GitHub", f["c_gh"], ("docs.github.com", "github.com"), f["tier_gh"], e_tgh, f["sust_gh"], e_sgh),
                ("GitLab", f["c_gl"], ("docs.gitlab.com", "about.gitlab.com"), f["tier_gl"], e_tgl, f["sust_gl"], e_sgl)):
            citas = URL_FECHA.findall(celda)
            propias = [(u, d) for u, d in citas if u.split("/")[2] in hosts]
            if not propias:
                err.append(f"{g}: la celda {plat} no cita una URL oficial de {plat} con fecha")
            for u, d in citas:
                if u not in fuentes:
                    err.append(f"{g}: URL que no esta en fuentes.md: {u}")
                elif fuentes[u][1] != d:
                    err.append(f"{g}: fecha {d} distinta de la de fuentes.md ({fuentes[u][1]}) para {u}")
            if re.search(r"\]\(https://[^)]+\)(?!,? \d{4}-\d{2}-\d{2})", celda):
                err.append(f"{g}: la celda {plat} tiene una URL sin fecha")
            if tier != e_tier:
                err.append(f"{g}: tier {plat} {tier!r} y la documentacion dice {e_tier!r}")
            elif tier != "ninguno" and not any(fuentes.get(u, (0, 0, ""))[2] == tier for u, _ in propias):
                err.append(f"{g}: tier {plat} {tier!r} no coincide con el de ninguna URL citada en fuentes.md")
            if sust != e_sust:
                err.append(f"{g}: sustituto {plat} {sust!r} y se espera {e_sust!r}")
            if tier == "ninguno" and not sust:
                err.append(f"{g}: {plat} sin la garantia en ningun tier y sin sustituto")
        ev = f["evid"]
        llamadas = re.findall(r"\b(G[HL]-\d{2})\b", ev)
        if ev.startswith("empírica"):
            if not llamadas:
                err.append(f"{g}: fila empírica sin id de llamada del log")
            for c in llamadas:
                if c not in observado:
                    err.append(f"{g}: id de llamada que no esta en el log: {c}")
                elif c not in PRUEBA or g not in PRUEBA[c][0]:
                    err.append(f"{g}: {c} no prueba nada de esta garantia "
                               f"({observado[c]['estado']}: {PRUEBA[c][1] if c in PRUEBA else 'sin valor de prueba'})")
                elif not PRUEBA[c][2](observado[c]):
                    err.append(f"{g}: {c} no tiene en lo observado el hecho que lo hace prueba ({PRUEBA[c][1]})")
        elif ev == "documental":
            pass
        else:
            err.append(f"{g}: evidencia que no es «empírica: …» ni «documental»")
    return err


def cifras_tabla(tabla):
    cnt = lambda col, v: sum(1 for f in tabla.values() if f[col] == v)
    return {"total": len(tabla), "gh_rs": cnt("tier_gh", T_RS), "gh_todos": cnt("tier_gh", T_TODOS),
            "gh_cola": cnt("tier_gh", T_COLA), "gl_free": cnt("tier_gl", "Free"), "gl_premium": cnt("tier_gl", "Premium"),
            "gl_ultimate": cnt("tier_gl", "Ultimate"), "gl_ninguno": cnt("tier_gl", "ninguno"),
            "empiricas": sum(1 for f in tabla.values() if f["evid"].startswith("empírica"))}


def linea_f17(h):
    return f"arbol={'0' * h} perfil=completa runner=github-windows clave={'0' * (64 if h == 40 else h)} " \
           f"local={'0' * (64 if h == 40 else h)} origen={'0' * h}"


def errores_prosa(adr, ct, llamadas):
    """Cifras de la prosa por contexto: cada «N garantías», «N llamadas» y «N caracteres» fuera de las tablas y del
    codigo tiene que estar dentro de una frase esperada, y cada frase esperada tiene que estar."""
    err = []
    prosa = re.sub(r"<!-- (\S+):inicio -->.*?<!-- \1:fin -->", " ", adr, flags=re.S)
    prosa = re.sub(r"```.*?```", " ", prosa, flags=re.S)
    prosa = re.sub(r"^[ \t]*\|.*\|[ \t]*$", " ", prosa, flags=re.M)
    prosa = re.sub(r"\s+", " ", prosa.replace("**", ""))
    lg, lc = len(linea_f17(40)), len(linea_f17(12))
    esperadas = [
        f"las {ct['total']} garantías",
        f"en GitHub, {ct['gh_rs']} garantías dependen del plan de los rulesets",
        f"{ct['gh_todos']} garantías no dependen del plan en GitHub",
        f"en GitLab Free se imponen {ct['gl_free']} garantías",
        f"{ct['gl_premium']} garantías exigen Premium",
        (f"{ct['gl_ultimate']} garantías exigen Ultimate" if ct["gl_ultimate"] else "ninguna garantía exige Ultimate"),
        f"{ct['gl_ninguno']} garantías no las impone GitLab en ningún tier",
        f"{ct['empiricas']} garantías tienen además evidencia empírica",
        f"`estado-puertas` hace {llamadas['estado-puertas GitHub (verde)']} llamadas en GitHub y "
        f"{llamadas['estado-puertas GitLab (verde)']} en GitLab",
        f"la línea de F1-7 ocupa {lg} caracteres y el límite es 255",
        f"con hashes de 12 caracteres ocupa {lc}",
    ]
    cubierto = []
    for frase in esperadas:
        pos = [m.start() for m in re.finditer(re.escape(frase), prosa)]
        if not pos:
            err.append(f"prosa: falta la frase «{frase}»")
        cubierto += [(i, i + len(frase)) for i in pos]
    for m in re.finditer(r"\b\d+ (?:garantías?|llamadas|caracteres)\b", prosa):
        if not any(a <= m.start() and m.end() <= b for a, b in cubierto):
            err.append(f"prosa: «{m.group(0)}» fuera de una frase que diga a qué corresponde "
                       f"(…{prosa[max(0, m.start() - 60):m.end()]})")
    return err


def errores_capacidades(adr, cifras_c):
    err = []
    bloque = entre_marcadores(adr, "capacidades")
    if bloque is None:
        return ["el ADR no tiene el bloque capacidades"]
    leido = {}
    for f in filas_tabla(bloque):
        if len(f) == 2:
            leido[f[0].strip("`")] = f[1].strip("`")
    for k, v in cifras_c.items():
        if leido.get(k) != v:
            err.append(f"capacidades {k}: el ADR dice {leido.get(k)!r} y lo calculado es {v!r}")
    for k in leido:
        if k not in cifras_c:
            err.append(f"capacidades: fila sin calculo: {k}")
    return err


def errores_adr(adr, ruta, repo, fuentes_txt, observado, llamadas, con_indice=True):
    """Funcion pura (la usan G y sus mutaciones)."""
    fuentes, err = leer_fuentes(fuentes_txt)
    tabla, e2 = leer_tabla(adr)
    err += e2 + errores_tabla(tabla, fuentes, observado)
    ct = cifras_tabla(tabla)
    cifras_c = {}
    if len(tabla) == 20:
        for plataforma, planes in (("github", PLANES_GH), ("gitlab", PLANES_GL)):
            for plan in planes:
                gars = capacidades(plataforma, plan, {}, tabla)
                imp = sum(g["estado"] == "impuesta" for g in gars)
                sus = sum(g["estado"].startswith("sustituida") for g in gars)
                cifras_c[f"{plataforma} {plan if isinstance(plan, str) else '-'.join(plan)}"] = f"{imp} / {sus} / {20 - imp - sus}"
        err += errores_capacidades(adr, cifras_c)
    res = entre_marcadores(adr, "resumen")
    if res is None:
        err.append("el ADR no tiene el bloque resumen")
    else:
        valores = {str(v) for v in ct.values()} | {str(v) for v in llamadas.values()} | {"255"}
        valores |= {x for v in cifras_c.values() for x in v.split(" / ")}
        for b in re.findall(r"\*\*([^*]+)\*\*", res):
            for n in re.findall(r"\d+", b):
                if n not in valores:
                    err.append(f"el resumen cita {n!r} ({b!r}), que no es una cifra calculada")
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
    err += errores_prosa(adr, ct, llamadas)
    return err, tabla


def _fila(adr, gid):
    m = re.search(rf"^[ \t]*\| {gid} \|.*\|$", adr, re.M)
    return m.group(0) if m else "(sin fila)"


def mutaciones(adr, fuentes_txt):
    f07, f03, f14, f09, f12, f15, f20 = (_fila(adr, g) for g in ("G07", "G03", "G14", "G09", "G12", "G15", "G20"))
    url = re.search(r"\]\((https://docs\.github\.com[^)]+)\), (\d{4}-\d{2}-\d{2})", f07)
    gl_url = re.search(r"\]\((https://docs\.gitlab\.com[^)]+)\)", f12)
    return [
        ("fila-borrada", adr.replace(f07 + "\n", ""), fuentes_txt, "falta la garantia G07"),
        ("fila-extra", adr.replace("<!-- tabla-equivalencias:fin -->", f07.replace("| G07 |", "| G21 |", 1)
                                   + "\n<!-- tabla-equivalencias:fin -->"), fuentes_txt, "sobra la garantia G21"),
        ("url-sin-fecha", adr.replace(f07, f07.replace(f"), {url.group(2)}", ")", 1)) if url else adr, fuentes_txt,
         "G07: la celda GitHub no cita una URL oficial de GitHub con fecha"),
        ("url-fuera-de-fuentes", adr.replace(f07, f07.replace(url.group(1), url.group(1) + "-otra", 1)) if url else adr,
         fuentes_txt, "G07: URL que no esta en fuentes.md"),
        ("tier-cambiado", adr.replace(f03, f03.replace("| Premium |", "| Free |", 1)), fuentes_txt,
         "G03: tier GitLab 'Free' y la documentacion dice 'Premium'"),
        ("tier-de-fuente-cambiado", adr, fuentes_txt.replace(f"| {gl_url.group(1)} | 2026-11-14 | Premium |",
                                                             f"| {gl_url.group(1)} | 2026-11-14 | Free |") if gl_url else "",
         "G12: tier GitLab 'Premium' no coincide con el de ninguna URL citada"),
        ("sustituto-borrado", adr.replace(f12, re.sub(r"GitLab: `cuenta-de-servicio`[^|]*", "GitLab: — ", f12)),
         fuentes_txt, "G12: sustituto GitLab None"),
        ("empirica-sin-id", adr.replace(f14, re.sub(r"\| empírica: [^|]+\|$", "| empírica |", f14.rstrip())), fuentes_txt,
         "G14: fila empírica sin id de llamada del log"),
        ("id-de-llamada-inexistente", adr.replace(f14, re.sub(r"(\| empírica: )", r"\1GH-99, ", f14)), fuentes_txt,
         "G14: id de llamada que no esta en el log: GH-99"),
        ("cita-larga", adr, fuentes_txt.replace("«Lists check runs for a commit ref»",
                                                "«Lists check runs for a commit ref and many more words to exceed the limit»"),
         "fuentes.md: D07 cita de"),
        ("capacidades-cambiadas", re.sub(r"(\| `gitlab free` \| `)\d+", r"\g<1>99", adr), fuentes_txt,
         "capacidades gitlab free: el ADR dice"),
        ("prosa-garantias", re.sub(r"Free(\s+)se(\s+)imponen(\s+)11(\s+)garantías", r"Free\1se\2imponen\g<3>12\4garantías",
                                   adr, count=1), fuentes_txt, "prosa: «12 garantías» fuera de una frase"),
        ("prosa-llamadas", re.sub(r"hace(\s+)4(\s+)llamadas", r"hace\g<1>3\2llamadas", adr, count=1), fuentes_txt,
         "prosa: «3 llamadas» fuera de una frase"),
        ("prosa-caracteres", re.sub(r"ocupa(\s+)(\d+)(\s+)caracteres(\s+)y", r"ocupa\g<1>250\3caracteres\4y", adr, count=1),
         fuentes_txt, "prosa: «250 caracteres» fuera de una frase"),
        ("resumen-inventado", adr.replace("<!-- resumen:fin -->", "Cifra suelta: **9999**.\n<!-- resumen:fin -->"),
         fuentes_txt, "el resumen cita '9999'"),
        ("id-inexistente", adr.replace("## Contexto", "## Contexto\n\nVer F7-99.", 1), fuentes_txt,
         "IDs que no existen: ['F7-99']"),
        ("enlace-roto", adr.replace("## Contexto", "## Contexto\n\n[x](F1-99-no-existe.md)", 1), fuentes_txt,
         "enlace que no resuelve: F1-99-no-existe.md"),
        ("seccion-borrada", adr.replace("## Cómo se ve en GitHub y en GitLab", "## Cómo se ve"), fuentes_txt,
         "faltan secciones"),
        ("estado-propuesta", adr.replace("- Estado: **aceptada**", "- Estado: **propuesta**"), fuentes_txt,
         "el estado no es aceptada"),
        ("sin-self-managed", re.sub(r"(^## No verificado\s*$)(.*?)(?=^## |\Z)",
                                    lambda x: x.group(1) + x.group(2).replace("self-managed", "autogestionado"), adr,
                                    flags=re.S | re.M), fuentes_txt, "«No verificado» no nombra «self-managed»"),
        ("divergencia-sin-f1-8", re.sub(r"(^## Divergencias con el plan\s*$)(.*?)(?=^## )",
                                        lambda x: x.group(1) + x.group(2).replace("F1-8", "F1-x"), adr,
                                        flags=re.S | re.M), fuentes_txt, "«Divergencias con el plan» no trata F1-8"),
        ("sin-motivo-plataforma", adr.replace("motivo=plataforma", "motivo=otro"), fuentes_txt,
         "falta la regla: motivo=plataforma"),
        ("g09-tier-ultimate", adr.replace(f09, f09.replace("| ninguno |", "| Ultimate |", 1)), fuentes_txt,
         "G09: tier GitLab 'Ultimate' y la documentacion dice 'ninguno'"),
        ("ids-de-g15-cambiados", adr.replace(f15, re.sub(r"\| empírica: [^|]+\|$", "| empírica: GL-01, GL-04 |",
                                                         f15.rstrip())), fuentes_txt,
         "G15: GL-01 no prueba nada de esta garantia"),
        ("empirica-con-401", adr.replace(f20, re.sub(r"\| documental \|$", "| empírica: GL-21 |", f20.rstrip())),
         fuentes_txt, "G20: GL-21 no prueba nada de esta garantia (401"),
    ]


def seccion_g(repo, observado, llamadas):
    sep("G: el ADR (tabla, fuentes, capacidades, IDs, enlaces, prosa) y sus mutaciones de control")
    ruta = os.path.join(repo, ADR_REL)
    if not os.path.exists(ruta):
        falla("G", "no existe el ADR")
        return {}
    adr = open(ruta, encoding="utf-8").read()
    fuentes_txt = open(FUENTES, encoding="utf-8").read()
    err, tabla = errores_adr(adr, ruta, repo, fuentes_txt, observado, llamadas)
    ct = cifras_tabla(tabla)
    print("cifras de la tabla: " + " ; ".join(f"{k}={v}" for k, v in ct.items()))
    print(f"linea de F1-7: {len(linea_f17(40))} caracteres con hashes completos, {len(linea_f17(12))} con 12 "
          f"(limite de description en GitLab: 255, L13)")
    for e in err:
        falla("G", e)
    print(f"errores del ADR: {len(err)}")
    for nombre, mut_adr, mut_fu, esperado in mutaciones(adr, fuentes_txt):
        if (mut_adr, mut_fu) == (adr, fuentes_txt):
            falla("G", f"la mutacion {nombre} no cambia nada")
            continue
        errs, _ = errores_adr(mut_adr, ruta, repo, mut_fu, observado, llamadas)
        ok = any(esperado in e for e in errs)
        print(f"  mutacion {nombre}: {'falla como se espera' if ok else 'NO FALLA'} "
              f"({str(next((e for e in errs if esperado in e), errs[:1]))[:110]})")
        if not ok:
            falla("G", f"la mutacion {nombre} no la detecta el guion")
    return tabla


# ---------------------------------------------------------------------------

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
    solo_adr = "--solo-adr" in args
    grabar = "--grabar-observado" in args
    args = [a for a in args if a not in ("--solo-adr", "--grabar-observado")]
    if len(args) != 2:
        raise SystemExit(__doc__)
    trabajo, repo = (os.path.abspath(a) for a in args)
    if not os.path.normcase(POC).startswith(os.path.normcase(os.path.join(repo, "tareas")) + os.sep):
        raise SystemExit(f"ERROR: el guion ({POC}) no esta dentro de tareas/ del repo {repo}")
    comprobar_trabajo(trabajo)
    borrar(trabajo)
    os.makedirs(trabajo)
    open(os.path.join(trabajo, MARCA), "w", encoding="utf-8").write("carpeta de trabajo de poc-f1-13.py\n")
    print(f"PoC F1-13 ; modo {'solo-adr (sin red)' if solo_adr else 'completo'} ; GitHub {GH_REPO}@{GH_SHA[:12]} ; "
          f"GitLab.com proyecto {GL_P} MR !{GL_MR} @{GL_SHA[:12]}")
    mock = Mock()
    variable = []
    try:
        if not solo_adr:
            seccion_i()
        seccion_n(mock)
        if solo_adr:
            observado = json.load(open(OBSERVADO, encoding="utf-8"))
            sep("R: GET reales no ejecutados (--solo-adr): se usa fixtures/observado.json")
            print(f"llamadas en observado.json: {len(observado)}")
        else:
            observado, variable = seccion_r(trabajo, grabar)
        reloj("R terminada")
        llamadas = seccion_m(mock, observado)
        adr = os.path.join(repo, ADR_REL)
        tabla, _ = leer_tabla(open(adr, encoding="utf-8").read()) if os.path.exists(adr) else ({}, [])
        seccion_c(tabla)
        seccion_g(repo, observado, llamadas)
        if variable:
            sep("V: lo variable (hash y tamano de cada cuerpo, limites de peticiones); NO se comprueba")
            for v in variable:
                print(v)
    finally:
        mock.parar()
        borrar(trabajo)
    print()
    print(f"carpeta de trabajo borrada al terminar: {'no' if os.path.lexists(trabajo) else 'si'}")
    if os.path.lexists(trabajo):
        falla("main", "la carpeta de trabajo sigue existiendo")
    reloj("fin")
    resultado()


if __name__ == "__main__":
    main()

"""PoC de F1-10 (TASK-011): carriles y modo automatico. SOLO LECTURA.

Se ejecuta con `python -I` (sin el directorio del guion ni PYTHON* en sys.path).
ToolsaWeb, OpenDockTrading y TaskCodeTwo solo se leen con `git log`,
`git show` y `git archive` de commits fijos (el archivo se lee en memoria; no
se extrae nada ni se ejecuta nada de esos repos). No habla con GitHub.

(A) Derivacion del carril minimo sobre `git log -M --name-status` de las ramas
    de tarea integradas (merges de primer padre en develop) de ToolsaWeb y
    OpenDockTrading: cuantas saldrian errata, fix o riesgo-alto; renombrados
    con sus dos nombres; casos sinteticos (un renombrado .github/ -> docs/ no
    es errata); por ficheros nunca sale funcionalidad.
(B) Tabla de politica del ADR (entre marcadores) frente a la tabla de carriles
    de F1-5 y a los perfiles por evento de F1-7, mas su coherencia interna;
    desvios inyectados que la comparacion tiene que detectar.
(C) `accion` simulada sobre las `## Transiciones` reales de las tareas de la
    0.16: paradas humanas por tarea con la politica nueva frente a hoy;
    nunca `continuar` sobre approve, merge-pr o tag.
(D) Configuraciones que bajan un carril o quitan una parada: rechazadas
    (N/N); las que endurecen, aceptadas y efectivas.

Uso: python -I poc-f1-10.py <carpeta-de-trabajo> <ruta-del-ADR-F1-10> <carpeta-Personal>
Sale con 0 solo si A..D salen como se espera.
"""
import io
import json
import os
import platform
import re
import subprocess
import sys
import tarfile

ORDEN = ["errata", "fix", "funcionalidad", "riesgo-alto"]
TIPOS_MAIN = ["release", "hotfix"]
FILAS = ORDEN + TIPOS_MAIN
FIJAS = ("approve", "merge-pr", "tag")          # paradas fijas por fase
# Commits fijos (leidos el 2026-11-14), los mismos que la PoC de F1-9, para
# que la medida se reproduzca aunque los repos avancen.
FIJO = {
    "ToolsaWeb": "93c9618ac156c4357278993df983361afccf1dc2",
    "OpenDockTrading": "fab8d7e05eee2784b9cd4695b94f612a6a1ee257",
    "TaskCodeTwo": "afc3791b5a856d1bc16fa6d762b7f3125071122c",
}
REPOS_A = ["ToolsaWeb", "OpenDockTrading"]
FALLOS = []


def falla(seccion, texto):
    FALLOS.append(f"{seccion}: {texto}")
    print(f"FALLO en {seccion}: {texto}")


def sep(t):
    print()
    print(f"=== {t} ===")


def git(repo, *args, binario=False):
    r = subprocess.run(["git", "-C", repo, "-c", "core.quotepath=false", *args],
                       capture_output=True, check=False)
    if r.returncode != 0:
        raise SystemExit(f"ERROR: git {' '.join(args)} en {os.path.basename(repo)}: "
                         f"{r.stderr.decode('utf-8', 'replace').strip()}")
    return r.stdout if binario else r.stdout.decode("utf-8", "replace")


def rango(c):
    return ORDEN.index(c)


# ---------------------------------------------------------------------------
# Nucleo (lo que el ADR fija como funciones puras, sin E/S)
# ---------------------------------------------------------------------------

class ErrorCarril(Exception):
    pass


class ErrorConfig(Exception):
    pass


def casa(glob, ruta):
    """Glob de rutas: `**` cualquier cosa (con /), `*` sin /, `?` un caracter sin /."""
    rx, i = "", 0
    while i < len(glob):
        if glob.startswith("**", i):
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


SUELO_RIESGO = (".github", ".taskcode")      # primer componente de la ruta, sin mayusculas


def clase_ruta(ruta, reglas):
    """Carril que exige una ruta sola. El suelo va en el binario; `reglas`
    (clave `rutas_carril`) solo puede subir."""
    if ruta.startswith("/") or ".." in ruta.split("/") or not ruta:
        raise ErrorCarril(f"ruta no valida: {ruta!r}")
    baja = ruta.lower()
    if baja.split("/", 1)[0] in SUELO_RIESGO or baja.rsplit("/", 1)[-1] == "codeowners":
        return "riesgo-alto"
    if any(casa(g, ruta) for g in reglas.get("riesgo-alto", [])):
        return "riesgo-alto"
    if any(casa(g, ruta) for g in reglas.get("fix", [])):
        return "fix"
    if ruta.startswith("docs/"):
        return "errata"
    return "fix"


def carril_minimo(cambios, reglas):
    """cambios: lista de (estado, [ruta] o [ruta_vieja, ruta_nueva]).
    Devuelve (carril, ruta que lo decide). Nunca devuelve funcionalidad."""
    if not cambios:
        raise ErrorCarril("PR sin ficheros: falla cerrado")
    carril, motivo = "errata", "todo bajo docs/"
    for _estado, rutas in cambios:
        for ruta in rutas:                     # en R y C, los dos nombres
            c = clase_ruta(ruta, reglas)
            if rango(c) > rango(carril):
                carril, motivo = c, ruta
    assert carril != "funcionalidad"
    return carril, motivo


def carril_efectivo(declarado, minimo):
    if declarado not in ORDEN:
        raise ErrorCarril(f"carril declarado no valido: {declarado!r}")
    return declarado if rango(declarado) >= rango(minimo) else minimo


PERFILES = ["rapida", "impacto", "completa", "completa y pesada"]
CLAVES_POLITICA = {"roles_diseno", "plan_aprobado", "paradas_extra",
                   "revisoras_extra", "lote", "perfil"}
FASES_EXTRA = {"start", "review"}


def validar_config(cfg, tabla):
    """Esquema de `politica_carriles` y `rutas_carril`: solo endurecen.
    Devuelve la lista de errores (vacia = valida)."""
    err = []
    for clave in cfg:
        if clave not in ("politica_carriles", "rutas_carril"):
            err.append(f"clave desconocida {clave}")
    pc = cfg.get("politica_carriles", {})
    if not isinstance(pc, dict):
        err.append("politica_carriles no es un mapa")
        pc = {}
    for carril, v in pc.items():
        if carril not in ORDEN:
            err.append(f"politica_carriles.{carril}: carril desconocido")
            continue
        suelo = tabla[carril]
        if not isinstance(v, dict):
            err.append(f"politica_carriles.{carril} no es un mapa")
            continue
        for k, x in v.items():
            p = f"politica_carriles.{carril}.{k}"
            if k not in CLAVES_POLITICA:
                err.append(f"{p}: clave no admitida")
            elif k == "roles_diseno":
                if type(x) is not int or x < suelo["roles_min"] or x > 5:
                    err.append(f"{p}: {x!r} por debajo del suelo {suelo['roles_min']} o fuera de 0..5")
            elif k == "plan_aprobado":
                if x is not True:
                    err.append(f"{p}: solo admite true (no se quita el plan aprobado)")
            elif k == "paradas_extra":
                if not isinstance(x, list) or any(not isinstance(f, str) or f not in FASES_EXTRA for f in x):
                    err.append(f"{p}: solo fases de {sorted(FASES_EXTRA)}")
            elif k == "revisoras_extra":
                if not isinstance(x, list) or any(not isinstance(s, str) or not s for s in x):
                    err.append(f"{p}: lista de nombres")
            elif k == "lote":
                if x is not False:
                    err.append(f"{p}: solo admite false (no se abre el lote)")
            elif k == "perfil":
                if x not in PERFILES or PERFILES.index(x) < PERFILES.index(suelo["perfil"]):
                    err.append(f"{p}: {x!r} por debajo de {suelo['perfil']!r}")
    rc = cfg.get("rutas_carril", {})
    if not isinstance(rc, dict):
        err.append("rutas_carril no es un mapa")
        rc = {}
    for carril, globs in rc.items():
        p = f"rutas_carril.{carril}"
        if carril not in ("fix", "riesgo-alto"):
            err.append(f"{p}: solo fix o riesgo-alto (errata no se da por rutas; funcionalidad nunca por ficheros)")
            continue
        if not isinstance(globs, list) or not globs:
            err.append(f"{p}: lista de globs no vacia")
            continue
        for g in globs:
            if not isinstance(g, str) or not g or g.startswith("!") or g.startswith("/") or ".." in g:
                err.append(f"{p}: glob no admitido {g!r} (sin negaciones ni rutas absolutas)")
    return err


def cargar_config(cfg, tabla):
    err = validar_config(cfg, tabla)
    if err:
        raise ErrorConfig("; ".join(err))
    return cfg


def fila_politica(carril, tipo, base, rama):
    """Fila de la tabla. `tipo` (texto de tarea.md, no confiable) solo elige
    fila con la base REAL del PR: release con base main; hotfix con base main
    y rama hotfix/*. Contra develop el tipo se ignora y manda el carril."""
    if base == "main":
        if tipo == "release":
            return "release"
        if tipo == "hotfix" and (rama or "").startswith("hotfix/"):
            return "hotfix"
        raise ErrorCarril(f"PR contra main con tipo {tipo!r} y rama {rama!r}: falla cerrado")
    if base != "develop":
        raise ErrorCarril(f"base no admitida {base!r}")
    return carril


def politica(carril, tipo, cfg, tabla, base="develop", rama=None):
    """Fila de la tabla endurecida con la configuracion ya validada. En una fila
    de main (release, hotfix), si el carril efectivo es riesgo-alto (minimo por
    ficheros), se exige ademas lo de riesgo-alto: plan firmado, 2 aprobaciones."""
    fila = fila_politica(carril, tipo, base, rama)
    fb = tabla[fila]
    pol = dict(fb)
    pol["paradas"] = set(fb["paradas"])
    pol["checks"] = set(fb["checks"])
    pol["fila"] = fila
    if fila in TIPOS_MAIN and carril == "riesgo-alto":
        ra = tabla["riesgo-alto"]
        pol["fila"] = f"{fila}+riesgo-alto"
        pol["plan"] = True
        pol["roles_min"] = max(pol["roles_min"], ra["roles_min"])
        pol["aprobaciones"] = max(pol["aprobaciones"], ra["aprobaciones"])
        pol["checks"] |= ra["checks"]
        pol["paradas"] |= ra["paradas"]
        pol["lote"] = False
    extra = cfg.get("politica_carriles", {}).get(carril, {}) if fila in ORDEN else {}
    if "roles_diseno" in extra:
        pol["roles_min"] = max(pol["roles_min"], extra["roles_diseno"])
    if extra.get("plan_aprobado"):
        pol["plan"] = True
        pol["aprobaciones"] = max(pol["aprobaciones"], 2)
        pol["paradas"].add("approve")
    pol["paradas"] |= set(extra.get("paradas_extra", []))
    if extra.get("lote") is False:
        pol["lote"] = False
    if "perfil" in extra:
        pol["perfil"] = extra["perfil"]
    pol["revisoras_extra"] = list(extra.get("revisoras_extra", []))
    return pol


def accion(fase, tarea, pol, modo, cfg, tabla):
    """-> (detener|preguntar|continuar, motivo). Las paradas fijas no dependen
    de la politica ni del modo."""
    if fase == "approve":
        return "detener", "falta el approve firmado de la persona"
    if fase == "merge-pr":
        return "detener", "merge-pr solo lo lanza una persona (finish)"
    if fase == "tag":
        return "detener", "el tag de release o hotfix lo crea la persona"
    if fase == "review":
        ef = tarea.get("efectivo", tarea["declarado"])
        if rango(ef) > rango(tarea["declarado"]):
            nueva = politica(ef, tarea.get("tipo"), cfg, tabla, tarea.get("base", "develop"), tarea.get("rama"))
            return "detener", (f"el carril sube de {tarea['declarado']} a {ef}"
                               + (": vuelve a plan" if nueva["plan"] or nueva["roles_min"] > 0 else ""))
        if tarea.get("ronda", 1) > tarea.get("tope", 2):
            return "preguntar", "tope de rondas: cerrar, ronda-extra o volver-a-diseno"
    if fase in pol["paradas"]:
        return "detener", f"parada de la politica en {fase}"
    if modo == "manual":
        return "detener", "modo manual"
    if modo == "semiautomatico":
        return "preguntar", "modo semiautomatico"
    if modo == "automatico":
        return "continuar", "sin parada en esta fase"
    raise ErrorCarril(f"modo desconocido {modo!r}")


# ---------------------------------------------------------------------------
# Lectura de tablas
# ---------------------------------------------------------------------------

def entre_marcadores(texto, nombre):
    m = re.search(rf"<!-- {nombre}:inicio -->\n(.*?)<!-- {nombre}:fin -->", texto, re.S)
    if not m:
        raise SystemExit(f"ERROR: faltan los marcadores {nombre}")
    filas = []
    for linea in m.group(1).splitlines():
        linea = linea.strip()     # la tabla puede ir sangrada dentro de una lista
        if linea.startswith("|") and not re.match(r"^\|[-| ]+\|$", linea):
            filas.append([c.strip() for c in linea.strip().strip("|").split("|")])
    return filas[0], filas[1:]


def backticks(celda):
    return set(re.findall(r"`([^`]+)`", celda))


def leer_tabla_politica(texto):
    cab, filas = entre_marcadores(texto, "tabla-politica")
    esperada = ["Carril", "Base del PR", "Roles de diseño", "Plan aprobado", "Revisoras",
                "Perfil de `verificacion`", "Checks obligatorios", "Aprobaciones de persona",
                "Paradas del automático", "Lote"]
    if cab != esperada:
        raise SystemExit(f"ERROR: cabecera de tabla-politica inesperada: {cab}")
    tabla, orden = {}, []
    for f in filas:
        nombre = f[0].strip("`").split(" ")[0].strip("`")
        roles = f[2]
        m = re.match(r"(\d+)", roles)
        roles_min = 4 if roles.startswith("todos") else (int(m.group(1)) if m else -1)
        tabla[nombre] = {
            "base": (re.findall(r"`([^`]+)`", f[1]) or [f[1]])[0],
            "roles_min": roles_min,
            "plan": f[3].lower().startswith("sí"),
            "revisoras": f[4],
            "perfil": f[5].split(" (")[0].strip(),
            "checks": backticks(f[6]),
            "aprobaciones": int(re.match(r"(\d+)", f[7]).group(1)),
            "paradas": backticks(f[8]),
            "lote": f[9].lower().startswith("sí"),
        }
        orden.append(nombre)
    return tabla, orden


def leer_f15(texto):
    _cab, filas = entre_marcadores(texto, "tabla-carriles")
    out = {}
    for f in filas:
        nombre = f[0].replace(" ", "-")
        out[nombre] = {"base": f[1].strip("`"), "checks": backticks(f[2]),
                       "aprobaciones": int(re.search(r"(\d+)", f[3]).group(1)),
                       "plan": "plan" in f[3]}
    return out


def leer_f17(texto):
    """Perfil por evento (decision 4 de F1-7) para las filas de PR."""
    perfil = {}
    ini = texto.index("**Perfil según el evento:**")
    texto = texto[ini:texto.index("5. **Runner según el perfil.**", ini)]
    for linea in texto.splitlines():
        linea = linea.strip()
        if linea.startswith("| PR a `develop` (todos los carriles)"):
            perfil["develop"] = linea.split("|")[2].strip()
        elif linea.startswith("| PR de release"):
            perfil["release"] = linea.split("|")[2].strip()
        elif linea.startswith("| PR de hotfix"):
            perfil["hotfix"] = linea.split("|")[2].strip()
    if len(perfil) != 3:
        raise SystemExit(f"ERROR: no se encuentran los perfiles de F1-7: {perfil}")

    def norm(p):
        p = p.split(" (")[0].split(",")[0].strip()
        return "completa y pesada" if p.startswith("completa y pesada") else p.split(" ")[0]
    return {k: norm(v) for k, v in perfil.items()}


def comparar(tabla, orden, f15, f17):
    fallos = []
    if orden[:4] != ORDEN:
        fallos.append(f"orden de los carriles {orden[:4]} != {ORDEN}")
    if set(tabla) != set(f15):
        fallos.append(f"filas {sorted(tabla)} != F1-5 {sorted(f15)}")
    for n in FILAS:
        if n not in tabla or n not in f15:
            continue
        t, g = tabla[n], f15[n]
        if t["base"] != g["base"]:
            fallos.append(f"{n}: base {t['base']} != F1-5 {g['base']}")
        if t["checks"] != g["checks"]:
            fallos.append(f"{n}: checks {sorted(t['checks'])} != F1-5 {sorted(g['checks'])}")
        if t["aprobaciones"] != g["aprobaciones"]:
            fallos.append(f"{n}: aprobaciones {t['aprobaciones']} != F1-5 {g['aprobaciones']}")
        if t["plan"] != g["plan"]:
            fallos.append(f"{n}: plan aprobado {t['plan']} != F1-5 {g['plan']}")
        if ("approve" in t["paradas"]) != t["plan"]:
            fallos.append(f"{n}: parada approve {'approve' in t['paradas']} sin casar con plan {t['plan']}")
        if "merge-pr" not in t["paradas"]:
            fallos.append(f"{n}: falta la parada merge-pr")
        if ("tag" in t["paradas"]) != (n in TIPOS_MAIN):
            fallos.append(f"{n}: parada tag {'tag' in t['paradas']} y es {'tipo de main' if n in TIPOS_MAIN else 'carril'}")
        extra = t["paradas"] - set(FIJAS)
        if extra:
            fallos.append(f"{n}: paradas fuera de las fijas {sorted(extra)}")
        esperado = f17["develop"] if t["base"] == "develop" else f17.get(n, "?")
        if t["perfil"] != esperado:
            fallos.append(f"{n}: perfil {t['perfil']!r} != F1-7 {esperado!r}")
        if n in ("riesgo-alto",) + tuple(TIPOS_MAIN) and t["lote"]:
            fallos.append(f"{n}: entra en lote y no debe")
        if ("revision-ia" in t["checks"]) != (t["revisoras"].lower() != "ninguna"):
            fallos.append(f"{n}: revisoras {t['revisoras']!r} sin casar con revision-ia en checks")
        if (t["roles_min"] == 0) != (n in ("errata", "release")):
            fallos.append(f"{n}: roles minimos {t['roles_min']}")
        if n in ("fix", "hotfix") and t["roles_min"] != 1 or n == "riesgo-alto" and t["roles_min"] != 4:
            fallos.append(f"{n}: roles {t['roles_min']} (F4-1 del plan: uno en fix, todos en riesgo alto)")
        if t["roles_min"] < 0:
            fallos.append(f"{n}: roles ilegibles")
    for a, b in zip(ORDEN, ORDEN[1:]):
        if a in tabla and b in tabla:
            ta, tb = tabla[a], tabla[b]
            if tb["aprobaciones"] < ta["aprobaciones"] or tb["roles_min"] < ta["roles_min"] \
                    or (ta["plan"] and not tb["plan"]) or not (tb["paradas"] >= ta["paradas"]):
                fallos.append(f"{b} es menos estricto que {a}")
    return fallos


# ---------------------------------------------------------------------------
# Lectura de los repos (solo git, commits fijos)
# ---------------------------------------------------------------------------

def tareas_de(repo, commit):
    data = git(repo, "archive", commit, "tareas/04-terminadas", binario=True)
    out = {}
    with tarfile.open(fileobj=io.BytesIO(data)) as tf:
        for m in tf.getmembers():
            if m.isfile() and re.fullmatch(r"tareas/04-terminadas/TASK-\d+/tarea\.md", m.name):
                texto = tf.extractfile(m).read().decode("utf-8", "replace").replace("\r\n", "\n")
                id_ = m.name.split("/")[2]
                tipo = re.search(r"^tipo:\s*\"?([\w-]+)", texto, re.M)
                trans = []
                sec = re.search(r"^## Transiciones\n(.*?)(?=^## |\Z)", texto, re.M | re.S)
                if sec:
                    for linea in sec.group(1).splitlines():
                        c = [x.strip() for x in linea.strip().strip("|").split("|")]
                        if len(c) >= 4 and re.match(r"\d{4}-", c[0]):
                            trans.append({"fase": c[1], "modo": c[2], "decidido_por": c[3]})
                out[id_] = {"tipo": tipo.group(1) if tipo else None, "trans": trans}
    return out


def tope_de(repo, commit):
    try:
        cfg = git(repo, "show", f"{commit}:.taskcode/config.yml")
    except SystemExit:
        return 2
    m = re.search(r"^tope_rondas:\s*(\d+)", cfg, re.M)
    return int(m.group(1)) if m else 2


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


def seccion_a(personal, trabajo):
    sep("A: carril minimo por ficheros sobre las ramas de tarea integradas")
    resumen, minimos = {}, {}
    for repo in REPOS_A:
        ruta = os.path.join(personal, repo)
        c = FIJO[repo]
        merges = git(ruta, "log", "--first-parent", "--merges", "--format=%H%x09%P%x09%s", c)
        cuenta = {k: 0 for k in ORDEN}
        solo_estado, sin_id, ramas = [], 0, {}
        riesgo, renombres, cruzan, erratas = [], 0, [], []
        excl_estado = 0
        for linea in merges.splitlines():
            h, padres, asunto = linea.split("\t", 2)
            p = padres.split()
            m = re.search(r"merge\((\w+)\): (\S+) ->", asunto)
            mid = re.search(r"task-(\d+)", m.group(2)) if m else None
            if not m or not mid:
                sin_id += 1
                continue
            ramas[m.group(1)] = ramas.get(m.group(1), 0) + 1
            id_ = f"TASK-{mid.group(1)}"
            todos = cambios_de_rango(ruta, p[0], p[1])
            pr = [(e, r) for e, r in todos if not all(x.startswith("tareas/") for x in r)]
            excl_estado += len(todos) - len(pr)
            if not pr:
                solo_estado.append(id_)
                continue
            carril, motivo = carril_minimo(pr, {})
            cuenta[carril] += 1
            minimos[(repo, id_)] = max(minimos.get((repo, id_), "errata"), carril, key=rango)
            if carril == "riesgo-alto":
                riesgo.append(f"{id_} ({motivo})")
            if carril == "errata":
                erratas.append(id_)
            for e, r in pr:
                if e.startswith("R"):
                    renombres += 1
                    cv, cn = clase_ruta(r[0], {}), clase_ruta(r[1], {})
                    if cv != cn:
                        cruzan.append(f"{id_} {e} {r[0]} -> {r[1]} ({cv} -> {cn})")
        total = sum(cuenta.values())
        print(f"--- {repo} (commit {c[:9]})")
        print(f"merges de primer padre con rama de tarea: {total + len(solo_estado)} "
              f"(por prefijo: {', '.join(f'{k}={v}' for k, v in sorted(ramas.items()))}; sin ID de tarea: {sin_id})")
        print(f"cambios solo de tareas/ (estado de la 0.16, fuera del PR en la 2.0) excluidos: {excl_estado}")
        print(f"tareas sin ningun fichero fuera de tareas/: {' '.join([str(len(solo_estado))] + sorted(solo_estado))}")
        print("carril minimo: " + " ; ".join(f"{k}={cuenta[k]}" for k in ORDEN))
        print(f"  errata ({len(erratas)}): {' '.join(sorted(erratas, key=lambda x: int(x[5:])))}")
        print(f"  riesgo-alto ({len(riesgo)}):")
        for r in sorted(riesgo, key=lambda x: int(x.split()[0][5:])):
            print(f"    {r}")
        print(f"renombrados fuera de tareas/: {renombres} ; que cambian de clase con los dos nombres: {len(cruzan)}")
        for x in sorted(cruzan):
            print(f"    {x}")
        if cuenta["funcionalidad"]:
            falla("A", f"{repo}: {cuenta['funcionalidad']} tareas derivadas como funcionalidad")
        resumen[repo] = dict(cuenta, solo_estado=len(solo_estado), renombres=renombres, cruzan=len(cruzan))
    print("-- casos sinteticos")
    casos = [
        ("renombrado .github/ -> docs/", [("R100", [".github/workflows/ci.yml", "docs/ci.yml"])], {}, "riesgo-alto"),
        ("renombrado docs/ -> .taskcode/", [("R090", ["docs/a.md", ".taskcode/config.yml"])], {}, "riesgo-alto"),
        ("renombrado src/ -> docs/", [("R100", ["src/x.md", "docs/x.md"])], {}, "fix"),
        ("solo docs/", [("M", ["docs/a.md"]), ("A", ["docs/b/c.md"])], {}, "errata"),
        ("docs/CODEOWNERS", [("M", ["docs/CODEOWNERS"])], {}, "riesgo-alto"),
        (".GitHub/ en mayusculas", [("A", [".GitHub/workflows/x.yml"])], {}, "riesgo-alto"),
        ("Docs/ en mayusculas no es errata", [("M", ["Docs/a.md"])], {}, "fix"),
        ("CLAUDE.md en la raiz", [("M", ["CLAUDE.md"])], {}, "fix"),
        ("borrado de un workflow", [("D", [".github/workflows/carril.yml"])], {}, "riesgo-alto"),
        ("copia desde .github/", [("C075", [".github/scripts/x.sh", "scripts/x.sh"])], {}, "riesgo-alto"),
        ("rutas_carril sube src/auth/**", [("M", ["src/auth/login.rs"])], {"riesgo-alto": ["src/auth/**"]}, "riesgo-alto"),
        ("rutas_carril fix sobre docs/api/**", [("M", ["docs/api/x.md"])], {"fix": ["docs/api/**"]}, "fix"),
        (".taskcode exacto (fichero o enlace)", [("A", [".taskcode"])], {}, "riesgo-alto"),
        (".github exacto (submodulo)", [("A", [".github"])], {}, "riesgo-alto"),
        (".githubx/ no es el suelo", [("A", [".githubx/a"])], {}, "fix"),
        ("PR sin ficheros", [], {}, "ERROR"),
        ("ruta con ..", [("M", ["docs/../src/x.rs"])], {}, "ERROR"),
    ]
    ok = 0
    for nombre, cambios, reglas, esperado in casos:
        try:
            obtenido = carril_minimo(cambios, reglas)[0]
        except ErrorCarril:
            obtenido = "ERROR"
        bien = obtenido == esperado
        ok += bien
        print(f"  {nombre:40} -> {obtenido:12} {'ok' if bien else 'FALLO (se esperaba ' + esperado + ')'}")
        if not bien:
            falla("A", f"caso {nombre}")
    print("-- carril efectivo (solo sube)")
    efe = [("errata", "fix", "fix"), ("funcionalidad", "fix", "funcionalidad"),
           ("fix", "riesgo-alto", "riesgo-alto"), ("riesgo-alto", "errata", "riesgo-alto")]
    for d, mi, esp in efe:
        o = carril_efectivo(d, mi)
        ok += o == esp
        print(f"  declarado={d:14} minimo={mi:12} -> {o}")
        if o != esp:
            falla("A", f"efectivo {d}/{mi}")
    try:
        carril_efectivo("release", "fix")
        falla("A", "carril declarado release aceptado")
    except ErrorCarril:
        ok += 1
        print("  declarado=release (es un tipo, no un carril) -> rechazado")
    tot_casos = len(casos) + len(efe) + 1
    print(f"RESULTADO A " + " ; ".join(
        f"{r}: errata={v['errata']} fix={v['fix']} riesgo-alto={v['riesgo-alto']} funcionalidad={v['funcionalidad']} "
        f"solo-estado={v['solo_estado']} renombrados={v['renombres']} cruzan={v['cruzan']}" for r, v in resumen.items())
        + f" ; casos {ok}/{tot_casos}")
    with open(os.path.join(trabajo, "a-minimos.json"), "w", encoding="utf-8") as f:
        json.dump({f"{r}/{i}": c for (r, i), c in sorted(minimos.items())}, f, indent=1, sort_keys=True)
    return minimos


def seccion_b(adr_texto, f15_texto, f17_texto, trabajo):
    sep("B: tabla de politica del ADR frente a F1-5 y F1-7")
    tabla, orden = leer_tabla_politica(adr_texto)
    f15, f17 = leer_f15(f15_texto), leer_f17(f17_texto)
    print(f"perfiles de F1-7: {f17}")
    for n in orden:
        t = tabla[n]
        print(f"{n:14} base={t['base']:8} roles>={t['roles_min']} plan={'si' if t['plan'] else 'no'} "
              f"aprob={t['aprobaciones']} perfil={t['perfil']!r} checks={','.join(sorted(t['checks']))} "
              f"paradas={','.join(sorted(t['paradas']))} lote={'si' if t['lote'] else 'no'}")
    fallos = comparar(tabla, orden, f15, f17)
    for x in fallos:
        falla("B", x)
    print("-- desvios inyectados (la comparacion tiene que detectarlos)")
    import copy

    def mut(f):
        t = copy.deepcopy(tabla)
        o = list(orden)
        f(t, o)
        return t, o
    muts = [
        ("errata sin revision-ia", lambda t, o: t["errata"]["checks"].discard("revision-ia")),
        ("funcionalidad sin parada approve", lambda t, o: t["funcionalidad"]["paradas"].discard("approve")),
        ("riesgo-alto con 1 aprobacion", lambda t, o: t["riesgo-alto"].update(aprobaciones=1)),
        ("release sin parada tag", lambda t, o: t["release"]["paradas"].discard("tag")),
        ("fix con plan aprobado", lambda t, o: t["fix"].update(plan=True)),
        ("hotfix con base develop", lambda t, o: t["hotfix"].update(base="develop")),
        ("riesgo-alto en lote", lambda t, o: t["riesgo-alto"].update(lote=True)),
        ("fix con perfil rapida", lambda t, o: t["fix"].update(perfil="rapida")),
        ("errata sin parada merge-pr", lambda t, o: t["errata"]["paradas"].discard("merge-pr")),
        ("fila errata quitada", lambda t, o: (t.pop("errata"), o.remove("errata"))),
        ("riesgo-alto con 1 rol", lambda t, o: t["riesgo-alto"].update(roles_min=1)),
        ("parada inventada en fix (review)", lambda t, o: t["fix"]["paradas"].add("review")),
    ]
    det = 0
    for nombre, f in muts:
        t, o = mut(f)
        n = len(comparar(t, o, f15, f17))
        det += n > 0
        print(f"  {nombre:36} -> {'detectado (' + str(n) + ' fallos)' if n else 'NO DETECTADO'}")
        if not n:
            falla("B", f"desvio no detectado: {nombre}")
    print(f"RESULTADO B filas={len(tabla)} fallos={len(fallos)} ; desvios detectados={det}/{len(muts)}")
    with open(os.path.join(trabajo, "b-tabla.json"), "w", encoding="utf-8") as fh:
        json.dump({k: {kk: sorted(vv) if isinstance(vv, set) else vv for kk, vv in v.items()}
                   for k, v in tabla.items()}, fh, indent=1, sort_keys=True)
    return tabla


def paradas_hoy(t, tope):
    filas = [r for r in t["trans"] if r["fase"] != "pausa"]
    pausas = len(t["trans"]) - len(filas)
    if not filas:
        return 0
    modo = filas[0]["modo"]
    if modo in ("manual", "semiautomatico"):
        return len(filas) - 1
    reviews = sum(r["fase"] == "review" for r in filas)
    aprob = sum(r["fase"] == "approve" and r["decidido_por"] == "persona" for r in filas)
    return aprob + pausas + max(0, reviews - tope)


def cadena_nueva(t, pol):
    reviews = max(1, sum(r["fase"] == "review" for r in t["trans"]))
    fases = ["plan"] + (["approve"] if pol["plan"] else []) + ["start"]
    fases += [("review", i + 1) for i in range(reviews)]
    fases += ["merge-pr"] + (["tag"] if t.get("tipo") in TIPOS_MAIN else [])
    return fases


def simular(t, tarea, tabla, cfg, modo):
    """Recorre la cadena nueva de una tarea con accion(). Si en review el carril
    efectivo supera al declarado, para (subida), vuelve a plan con el efectivo
    y, si este exige plan aprobado, pasa otra vez por approve."""
    b, r = tarea.get("base", "develop"), tarea.get("rama")
    pol_d = politica(tarea["declarado"], tarea.get("tipo"), cfg, tabla, b, r)
    pol_e = politica(tarea["efectivo"], tarea.get("tipo"), cfg, tabla, b, r)
    res = {"paradas": 0, "tipos": {}, "vistas": set(), "malas": []}

    def paso(fase, actual, pol, ronda):
        a, motivo = accion(fase, dict(actual, ronda=ronda or 1), pol, modo, cfg, tabla)
        if fase in FIJAS:
            res["vistas"].add(fase)
            if a == "continuar":
                res["malas"].append(fase)
        if a != "continuar":
            res["paradas"] += 1
            clave = (fase if fase in FIJAS else "subida" if "sube" in motivo else
                     "tope" if "tope" in motivo else "politica" if "politica" in motivo else f"modo-{modo}")
            res["tipos"][clave] = res["tipos"].get(clave, 0) + 1
        return motivo

    actual, pol = dict(tarea), pol_d
    for f in cadena_nueva(t, pol_d)[1:]:
        fase, ronda = (f if isinstance(f, tuple) else (f, None))
        motivo = paso(fase, actual, pol, ronda)
        if fase == "review" and "sube" in motivo:
            actual, pol = dict(tarea, declarado=tarea["efectivo"]), pol_e
            if pol_e["plan"]:
                paso("approve", actual, pol, None)
            paso("review", actual, pol, ronda)
    return res["paradas"], res["tipos"], res["vistas"], res["malas"], pol_e


def seccion_c(personal, tabla, minimos, trabajo):
    sep("C: accion simulada sobre las ## Transiciones reales de la 0.16")
    print("hoy (0.16): manual y semiautomatico = transiciones - 1; automatico = approve de persona"
          " + pausas + rondas por encima del tope")
    print("nuevo: cadena plan -> [approve] -> start -> review x N -> merge-pr -> [tag] con accion();"
          " declarado = fix si tipo fix, funcionalidad si no; efectivo = max(declarado, minimo de A)")
    total_malas, discrepancias = 0, 0
    filas_res = []
    for repo in ["ToolsaWeb", "OpenDockTrading", "TaskCodeTwo"]:
        ruta = os.path.join(personal, repo)
        tareas = tareas_de(ruta, FIJO[repo])
        tope = tope_de(ruta, FIJO[repo])
        hoy_t = nue_t = cota_t = 0
        tipos_t, dist, por_carril = {}, {}, {}
        filas_auto = [r for t in tareas.values() for r in t["trans"] if r["modo"] == "automatico"]
        fin_auto = sum(r["fase"] == "finish" for r in filas_auto)
        apr_auto = sum(r["fase"] == "approve" and r["decidido_por"] == "automatico" for r in filas_auto)
        apr_pers = sum(r["fase"] == "approve" and r["decidido_por"] == "persona" for r in filas_auto)
        sin_trans = 0
        for id_, t in sorted(tareas.items(), key=lambda x: int(x[0][5:])):
            if not t["trans"]:
                sin_trans += 1
                continue
            declarado = "fix" if t["tipo"] == "fix" else "funcionalidad"
            minimo = minimos.get((repo, id_), "errata")
            tarea = {"declarado": declarado, "efectivo": carril_efectivo(declarado, minimo),
                     "tipo": t["tipo"], "tope": tope}
            modo = [r for r in t["trans"] if r["fase"] != "pausa"][0]["modo"]
            hoy = paradas_hoy(t, tope)
            nue, tipos, fijas, malas, pol = simular(t, tarea, tabla, {}, modo)
            # cota inferior: la tarea se habria declarado con su carril minimo
            # (solo docs/ -> errata, codigo -> fix): sin approve salvo riesgo-alto
            cota = simular(t, dict(tarea, declarado=minimo, efectivo=minimo), tabla, {}, modo)[0]
            cota_t += cota
            total_malas += len(malas)
            if modo == "automatico" and not (pol["paradas"] <= fijas):
                discrepancias += 1
                falla("C", f"{repo} {id_}: paradas vistas {sorted(fijas)} != politica {sorted(pol['paradas'])}")
            hoy_t += hoy
            nue_t += nue
            dist[(hoy, nue)] = dist.get((hoy, nue), 0) + 1
            por_carril[tarea["efectivo"]] = por_carril.get(tarea["efectivo"], 0) + 1
            for k, v in tipos.items():
                tipos_t[k] = tipos_t.get(k, 0) + v
        n = sum(dist.values())
        print(f"--- {repo} (commit {FIJO[repo][:9]}, tope_rondas={tope}) tareas={n} sin transiciones={sin_trans}")
        print(f"carril efectivo: {', '.join(f'{k}={por_carril.get(k, 0)}' for k in ORDEN)}")
        print(f"filas en automatico: finish={fin_auto} ; approve decidido_por automatico={apr_auto} ; "
              f"approve decidido_por persona={apr_pers}")
        print(f"paradas humanas: hoy={hoy_t} ({hoy_t / n:.2f}/tarea) ; nuevo={nue_t} ({nue_t / n:.2f}/tarea) ; "
              f"cota inferior con declarado = minimo por ficheros={cota_t} ({cota_t / n:.2f}/tarea)")
        print(f"paradas nuevas por tipo: {', '.join(f'{k}={v}' for k, v in sorted(tipos_t.items()))}")
        print("reparto (hoy -> nuevo: tareas): " + ", ".join(f"{h}->{x}: {c}" for (h, x), c in sorted(dist.items())))
        filas_res.append(f"{repo}: tareas={n} hoy={hoy_t / n:.2f} nuevo={nue_t / n:.2f} cota={cota_t / n:.2f}")
    print("-- tareas sinteticas de tipo release y hotfix (la 0.16 no tiene ninguna terminada)")
    sint = [("release", "funcionalidad", "develop"), ("hotfix", "fix", "hotfix/task-900-x")]
    for tipo, decl, rama in sint:
        t = {"tipo": tipo, "trans": [{"fase": "plan", "modo": "automatico", "decidido_por": "persona"},
                                     {"fase": "review", "modo": "automatico", "decidido_por": "persona"}]}
        tarea = {"declarado": decl, "efectivo": decl, "tipo": tipo, "tope": 2, "base": "main", "rama": rama}
        nue, tipos, fijas, malas, pol = simular(t, tarea, tabla, {}, "automatico")
        total_malas += len(malas)
        bien = fijas == pol["paradas"] and not malas
        print(f"  {tipo:8} paradas={nue} {sorted(tipos.items())} politica={sorted(pol['paradas'])} -> {'ok' if bien else 'FALLO'}")
        if not bien:
            falla("C", f"sintetica {tipo}")
    print("-- tipo de la tarea frente a la base real del PR (revision 1, IMP-1)")
    gh = [("M", [".github/workflows/ci.yml"]), ("M", ["src/a.rs"])]
    src = [("M", ["src/a.rs"])]
    casos = [
        # nombre, tipo, base, rama, cambios, declarado, esperado (fila, plan, aprobaciones, paradas) o ERROR
        ("hotfix con .github/ contra develop", "hotfix", "develop", "hotfix/task-901-x", gh, "fix",
         ("riesgo-alto", True, 2, ["approve", "merge-pr"])),
        ("hotfix con .github/ contra main", "hotfix", "main", "hotfix/task-901-x", gh, "fix",
         ("hotfix+riesgo-alto", True, 2, ["approve", "merge-pr", "tag"])),
        ("hotfix solo src/ contra main", "hotfix", "main", "hotfix/task-902-x", src, "fix",
         ("hotfix", False, 1, ["merge-pr", "tag"])),
        ("release con .github/ contra develop", "release", "develop", "feature/task-903-x", gh, "fix",
         ("riesgo-alto", True, 2, ["approve", "merge-pr"])),
        ("release con .taskcode/ contra main", "release", "main", "develop", [("M", [".taskcode/config.yml"])], "fix",
         ("release+riesgo-alto", True, 2, ["approve", "merge-pr", "tag"])),
        ("release solo src/ contra main", "release", "main", "develop", src, "fix",
         ("release", False, 1, ["merge-pr", "tag"])),
        ("hotfix contra main desde feature/*", "hotfix", "main", "feature/task-904-x", src, "fix", "ERROR"),
        ("feature contra main", "feature", "main", "feature/task-905-x", src, "funcionalidad", "ERROR"),
    ]
    okt = 0
    for nombre, tipo, base, rama, cambios, decl, esp in casos:
        ef = carril_efectivo(decl, carril_minimo(cambios, {})[0])
        try:
            p = politica(ef, tipo, {}, tabla, base, rama)
            obt = (p["fila"], p["plan"], p["aprobaciones"], sorted(p["paradas"]))
        except ErrorCarril:
            obt = "ERROR"
        bien = obt == esp
        okt += bien
        print(f"  {nombre:38} efectivo={ef:12} -> {obt} {'ok' if bien else 'FALLO (se esperaba ' + str(esp) + ')'}")
        if not bien:
            falla("C", f"tipo y base: {nombre}")
    print("-- barrido: approve, merge-pr y tag en todo carril, tipo, base, modo y configuracion valida de D")
    barr = cont = rech = 0
    bases = (("develop", "feature/task-1-x"), ("main", "develop"), ("main", "hotfix/task-1-x"))
    for cfg in CONFIGS_VALIDAS:
        for carril in ORDEN:
            for tipo in [None, "feature", "fix"] + TIPOS_MAIN:
                for base, rama in bases:
                    try:
                        pol = politica(carril, tipo, cfg, tabla, base, rama)
                    except ErrorCarril:
                        rech += 1
                        continue
                    for modo in ("manual", "semiautomatico", "automatico"):
                        for fase in FIJAS:
                            barr += 1
                            a, _ = accion(fase, {"declarado": carril, "tipo": tipo, "base": base, "rama": rama},
                                          pol, modo, cfg, tabla)
                            cont += a == "continuar"
    print(f"  combinaciones={barr} ; continuar={cont} ; tipo y base que fallan cerrado (sin fila)={rech}")
    if cont:
        falla("C", f"{cont} continuar sobre una parada fija")
    total_malas += cont
    print(f"RESULTADO C {' ; '.join(filas_res)} ; tipo y base {okt}/{len(casos)} ; continuar sobre approve/merge-pr/tag={total_malas} ; "
          f"paradas distintas de la politica={discrepancias}")


CONFIGS_VALIDAS = [
    {},
    {"politica_carriles": {"fix": {"roles_diseno": 2, "plan_aprobado": True, "lote": False}}},
    {"politica_carriles": {"funcionalidad": {"paradas_extra": ["start", "review"], "perfil": "completa y pesada"}},
     "rutas_carril": {"riesgo-alto": ["src/auth/**"]}},
    {"politica_carriles": {"errata": {"revisoras_extra": ["code-quality-reviewer"], "roles_diseno": 1}},
     "rutas_carril": {"fix": ["docs/api/**"]}},
    {"politica_carriles": {"riesgo-alto": {"roles_diseno": 5, "lote": False}}},
]


def seccion_d(tabla, trabajo):
    sep("D: configuracion que solo endurece")
    print("-- aceptadas (endurecen o no cambian nada)")
    acept = 0
    for i, cfg in enumerate(CONFIGS_VALIDAS):
        err = validar_config(cfg, tabla)
        mas = True
        for carril in ORDEN:
            a, b = politica(carril, None, {}, tabla), politica(carril, None, cfg, tabla)
            if b["roles_min"] < a["roles_min"] or (a["plan"] and not b["plan"]) or not (b["paradas"] >= a["paradas"]) \
                    or (b["lote"] and not a["lote"]) or PERFILES.index(b["perfil"]) < PERFILES.index(a["perfil"]):
                mas = False
        bien = not err and mas
        acept += bien
        print(f"  V{i} {json.dumps(cfg, sort_keys=True, ensure_ascii=False)} -> {'aceptada, no baja nada' if bien else 'FALLO ' + str(err)}")
        if not bien:
            falla("D", f"config valida V{i}")
    pol = politica("fix", None, CONFIGS_VALIDAS[1], tabla)
    print(f"  efecto de V1 en fix: plan={pol['plan']} paradas={sorted(pol['paradas'])} roles>={pol['roles_min']} lote={pol['lote']}")
    if not (pol["plan"] and "approve" in pol["paradas"]):
        falla("D", "V1 no anade approve en fix")
    print("-- rechazadas (bajan un carril, quitan una parada o no se reconocen: falla cerrado)")
    negativos = [
        ("funcionalidad sin plan aprobado", {"politica_carriles": {"funcionalidad": {"plan_aprobado": False}}}),
        ("riesgo-alto con 1 aprobacion", {"politica_carriles": {"riesgo-alto": {"aprobaciones": 1}}}),
        ("quitar merge-pr de fix", {"politica_carriles": {"fix": {"paradas_quitar": ["merge-pr"]}}}),
        ("paradas de riesgo-alto sustituidas", {"politica_carriles": {"riesgo-alto": {"paradas": []}}}),
        ("paradas_extra con approve quitado (fase no admitida)", {"politica_carriles": {"fix": {"paradas_extra": ["-approve"]}}}),
        ("rutas que dan errata", {"rutas_carril": {"errata": ["src/**"]}}),
        ("rutas que dan funcionalidad", {"rutas_carril": {"funcionalidad": ["src/**"]}}),
        ("glob negado en riesgo-alto", {"rutas_carril": {"riesgo-alto": ["!docs/**"]}}),
        ("riesgo-alto entra en lote", {"politica_carriles": {"riesgo-alto": {"lote": True}}}),
        ("funcionalidad con 0 roles", {"politica_carriles": {"funcionalidad": {"roles_diseno": 0}}}),
        ("fix con perfil rapida", {"politica_carriles": {"fix": {"perfil": "rapida"}}}),
        ("errata sin revision-ia", {"politica_carriles": {"errata": {"revision_ia": False}}}),
        ("politica de release (es un tipo)", {"politica_carriles": {"release": {"lote": False}}}),
        ("clave raiz desconocida", {"carril_maximo": "fix"}),
        ("modo que se salta paradas", {"politica_carriles": {"funcionalidad": {"modo": "sin-paradas"}}}),
        ("riesgo-alto con roles como texto", {"politica_carriles": {"riesgo-alto": {"roles_diseno": "4"}}}),
        ("paradas_extra con una lista dentro", {"politica_carriles": {"fix": {"paradas_extra": [["review"]]}}}),
    ]
    rech = 0
    for nombre, cfg in negativos:
        try:
            cargar_config(cfg, tabla)
            print(f"  {nombre:52} -> ACEPTADA")
            falla("D", f"negativo aceptado: {nombre}")
        except ErrorConfig as e:
            rech += 1
            print(f"  {nombre:52} -> rechazada: {str(e)[:90]}")
    print("-- PR que cambia .taskcode/config.yml para bajar la politica")
    cfg_base, cfg_rama = {}, {"politica_carriles": {"funcionalidad": {"plan_aprobado": False}}}
    carril, motivo = carril_minimo([("M", [".taskcode/config.yml"]), ("M", ["src/x.rs"])], {})
    usada = cfg_base   # el CLI y el check leen la de la base (origin/develop), nunca la de la rama
    try:
        cargar_config(cfg_rama, tabla)
        rama_ok = True
    except ErrorConfig:
        rama_ok = False
    pol = politica("funcionalidad", None, usada, tabla)
    print(f"  carril minimo={carril} ({motivo}) ; config de la rama valida={rama_ok} ; "
          f"politica con la de la base: plan={pol['plan']} paradas={sorted(pol['paradas'])}")
    if carril != "riesgo-alto" or rama_ok or not pol["plan"]:
        falla("D", "PR que cambia la configuracion")
    print(f"RESULTADO D aceptadas={acept}/{len(CONFIGS_VALIDAS)} ; rechazadas={rech}/{len(negativos)}")


def main():
    if len(sys.argv) != 4:
        raise SystemExit("uso: python -I poc-f1-10.py <trabajo> <adr-F1-10> <carpeta-Personal>")
    trabajo, adr, personal = (os.path.abspath(a) for a in sys.argv[1:])
    repo_raiz = os.path.abspath(os.path.join(os.path.dirname(adr), "..", ".."))
    if os.path.dirname(trabajo.rstrip("\\/")) == trabajo.rstrip("\\/")[:3].rstrip("\\/") or \
            len(trabajo.replace("\\", "/").strip("/").split("/")) < 3:
        raise SystemExit("ERROR: la carpeta de trabajo no puede estar en la raiz de una unidad")
    if os.path.normcase(trabajo).startswith(os.path.normcase(repo_raiz) + os.sep):
        raise SystemExit("ERROR: la carpeta de trabajo no puede estar dentro del repo")
    os.makedirs(trabajo, exist_ok=True)
    marca = os.path.join(trabajo, ".poc-f1-10")
    if os.listdir(trabajo) and not os.path.exists(marca):
        raise SystemExit("ERROR: la carpeta de trabajo no esta vacia ni es de una ejecucion anterior")
    open(marca, "w").close()
    git_v = subprocess.run(["git", "--version"], capture_output=True, text=True).stdout.strip()
    print(f"python {platform.python_version()} (-I: {bool(sys.flags.isolated)}) ; {git_v}")
    if not sys.flags.isolated:
        falla("inicio", "no se ejecuta con python -I")
    print(f"ADR: {os.path.basename(adr)} ; trabajo: .../{'/'.join(trabajo.replace(chr(92), '/').split('/')[-3:])}")
    for r, c in FIJO.items():
        fecha = git(os.path.join(personal, r), "log", "-1", "--format=%cs", c).strip()
        print(f"repo {r}: commit {c[:9]} ({fecha})")
    leer = lambda p: open(p, encoding="utf-8").read().replace("\r\n", "\n")
    adr_texto = leer(adr)
    f15 = leer(os.path.join(os.path.dirname(adr), "F1-5-puertas-en-github.md"))
    f17 = leer(os.path.join(os.path.dirname(adr), "F1-7-verificacion-ci-perfiles-cache-runners.md"))
    minimos = seccion_a(personal, trabajo)
    tabla = seccion_b(adr_texto, f15, f17, trabajo)
    seccion_c(personal, tabla, minimos, trabajo)
    seccion_d(tabla, trabajo)
    sep("RESUMEN")
    if FALLOS:
        print(f"FALLOS ({len(FALLOS)}):")
        for f in FALLOS:
            print(f"  {f}")
        sys.exit(1)
    print("todo como se espera: A, B, C y D")


if __name__ == "__main__":
    main()

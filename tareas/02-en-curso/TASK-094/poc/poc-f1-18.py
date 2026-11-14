"""PoC de F1-18 (TASK-094): hito de decisiones (ADR) en init y en actualizar.

Se ejecuta con `python -I -B` (sin el directorio del guion ni PYTHON* en
sys.path, sin __pycache__). OpenDockTrading SOLO SE LEE: se hace
`git clone --no-hardlinks` a la carpeta de trabajo y todo se ejecuta sobre la
copia. De TaskCodeTwo solo se leen objetos de git de un commit fijo
(`git cat-file`, `ls-tree`, `log`, `diff`) y, en G, el ADR y el indice del
arbol de trabajo. No se lanza `taskctl` ni ningun hook. No usa la red.

Reutiliza, cargandolo como modulo, el guion de la PoC de F1-12
(TASK-013, terminada): sus lectores de tareas de la 0.16, su inventario
«hecho a mano» de F1-11 y su uso de campos, para que los bloques del informe
de F1-12 se recalculen con el mismo codigo.

(H) Fase 1 de TaskCodeTwo: minutos (`## Duracion`, «Real») y tokens
    (`## Coste`: total y salida) por ADR de TASK-002..TASK-013; mediana y p75.
(A) Estudio de adopcion de una copia de OpenDockTrading en el commit fijo:
    informe ampliado (bloques de F1-12 recalculados y uso medido),
    `proponer_hito` (un elemento por hueco, bloqueantes aparte) con
    presupuesto, y hash de la copia igual antes y despues.
(R) Registro `hito-decisiones.yml` en una rama de estado de prueba:
    `hito_estado` en ausente, abierto, cerrado y no-necesario, mas registros
    rotos que tienen que salir abierto (falla cerrado).
(P) Puerta simulada: parada local en `start` (carril declarado) y check
    `estado` del PR (carril por ficheros, F1-10) sobre tareas reales de
    TaskCodeTwo y de OpenDockTrading; determinista en dos llamadas.
(G) El ADR: cifras, resumen, secciones, IDs, enlaces e indice, y copias
    mutadas del ADR que fallan cada una por su motivo.

Uso: python -I -B poc-f1-18.py <trabajo> <repo-TaskCodeTwo> <carpeta-Personal>
Sale con 0 solo si todo sale como se espera. El log (stdout) es determinista;
los tiempos de reloj van a stderr.
"""
import hashlib
import importlib.util
import json
import math
import os
import re
import sys
import time

import yaml

FIJO_TC2 = "0cc5bc3533e343c1444a4ab232a5fa75d449fe03"   # rama de TASK-094 al empezar, 2026-11-14
H_TAREAS = [f"TASK-{n:03d}" for n in range(2, 14)]      # ADR F1-1..F1-12
RAMA_ESTADO = "taskcodetwo/tareas"
FIN = ("terminada", "descartada")
COD_PRECONDICION = 4                                    # el de F1-12 (F1-4 deja los numeros a F2)
AREAS_INIT = ("pila", "verificacion", "ramas y puertas", "seguridad", "equipo", "plataforma")
FALLOS = []
MEDIDO = {}
T0 = time.time()
F12 = None          # modulo de la PoC de F1-12


def falla(seccion, texto):
    FALLOS.append(f"{seccion}: {texto}")
    print(f"FALLO en {seccion}: {texto}")


def sep(t):
    print()
    print(f"=== {t} ===")


def tiempo(texto):
    print(f"[tiempo] {texto} ({time.time() - T0:.0f} s desde el inicio)", file=sys.stderr, flush=True)


def cargar_f12(repo_tc2):
    ruta = os.path.join(repo_tc2, "tareas", "04-terminadas", "TASK-013", "poc", "poc-f1-12.py")
    spec = importlib.util.spec_from_file_location("poc_f1_12", ruta)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


# ---------------------------------------------------------------------------
# Lectura del uso real de una tarea de la 0.16: ## Coste y ## Duracion
# ---------------------------------------------------------------------------

def seccion_md(texto, titulo):
    m = re.search(rf"^## {re.escape(titulo)}[^\n]*\n(.*?)(?=^## |\Z)", texto, re.S | re.M)
    return m.group(1) if m else None


def coste(texto):
    """Filas de ## Coste como dicts con enteros; None si la tarea no tiene la seccion o la tabla."""
    s = seccion_md(texto, "Coste")
    if s is None:
        return None
    lineas = [l.strip() for l in s.splitlines() if l.strip().startswith("|")]
    if len(lineas) < 2:
        return None
    cab = [c.strip() for c in lineas[0].strip("|").split("|")]
    filas = []
    for l in lineas[2:]:
        c = [x.strip() for x in l.strip("|").split("|")]
        d = dict(zip(cab, c))
        for k in ("entrada", "cache_escritura", "cache_lectura", "salida", "lecturas", "total"):
            d[k] = int(d[k])
        d["ronda"] = int(d["ronda"]) if d.get("ronda", "-").isdigit() else None
        filas.append(d)
    return filas


def minutos(texto, clave="Real"):
    s = seccion_md(texto, "Duracion")
    if s is None:
        return None
    m = re.search(rf"^- {clave}[^:\n]*:\s*(\d+)\s*min", s, re.M)
    return int(m.group(1)) if m else None


def mediana(xs):
    xs = sorted(xs)
    n = len(xs)
    if not n:
        return None
    v = xs[n // 2] if n % 2 else (xs[n // 2 - 1] + xs[n // 2]) / 2
    return int(v) if v == int(v) else v


def fmt(v):
    """Numero sin «.0» cuando es entero (la mediana de una lista par puede acabar en .5)."""
    return str(int(v)) if isinstance(v, (int, float)) and v == int(v) else str(v)


def p75(xs):
    """Percentil 75 por rango mas cercano: el valor en la posicion ceil(0,75 n) de la lista ordenada."""
    xs = sorted(xs)
    return xs[math.ceil(0.75 * len(xs)) - 1] if xs else None


def rondas(filas):
    r = [f["ronda"] for f in filas if f["fase"] == "revision" and f["ronda"] is not None]
    return max(r) if r else 0


def reparto(xs):
    return ", ".join(f"{k}: {xs.count(k)}" for k in sorted(set(xs)))


def uso_medido(tareas):
    """Bloque «uso medido» del informe: lo que dicen las tablas de cada tarea; «no medido» si no hay."""
    con_coste = {tid: coste(t["texto"]) for tid, t in tareas.items()}
    con_coste = {k: v for k, v in con_coste.items() if v}
    reales = {tid: minutos(t["texto"]) for tid, t in tareas.items()}
    reales = {k: v for k, v in reales.items() if v is not None}
    u = {"tareas": len(tareas), "con_coste": len(con_coste), "con_real": len(reales)}
    if con_coste:
        fases = {}
        for filas in con_coste.values():
            for f in filas:
                a = fases.setdefault(f["fase"], [0, 0])
                a[0] += f["total"]
                a[1] += f["salida"]
        u["tokens_por_fase"] = {k: fases[k] for k in sorted(fases)}
        tot = [sum(f["total"] for f in v) for v in con_coste.values()]
        u["tokens_tarea"] = (mediana(tot), p75(tot))
        u["rondas"] = reparto([rondas(v) for v in con_coste.values()])
    if reales:
        v = list(reales.values())
        u["minutos"] = (sum(v), mediana(v), p75(v))
    u["verificacion_ligera"] = sum(1 for t in tareas.values() if seccion_md(t["texto"], "Verificacion ligera") is not None)
    return u


# ---------------------------------------------------------------------------
# H: cifras de la Fase 1 de TaskCodeTwo
# ---------------------------------------------------------------------------

def seccion_h(repo_tc2):
    sep(f"H: minutos y tokens por ADR de TaskCodeTwo ({H_TAREAS[0]}..{H_TAREAS[-1]}, commit {FIJO_TC2[:7]})")
    tareas = F12.tareas_016(repo_tc2, FIJO_TC2)
    filas_h, faltan = [], []
    print("tarea     estimada  real  rondas  diseno?  total_tokens  salida  diseno_total")
    for tid in H_TAREAS:
        t = tareas.get(tid)
        if t is None or t["carpeta"] != "04-terminadas":
            faltan.append(f"{tid} no terminada")
            continue
        c, real, est = coste(t["texto"]), minutos(t["texto"]), minutos(t["texto"], "Estimada")
        if not c or real is None:
            faltan.append(f"{tid} sin ## Coste o sin Real")
            continue
        tot, sal = sum(f["total"] for f in c), sum(f["salida"] for f in c)
        dis = sum(f["total"] for f in c if f["fase"] == "diseno")
        filas_h.append({"id": tid, "est": est, "real": real, "total": tot, "salida": sal, "diseno": dis,
                        "con_diseno": any(f["fase"] == "diseno" for f in c), "rondas": rondas(c)})
        print(f"{tid}  {est!s:>8}  {real:>4}  {rondas(c):>6}  {'si' if dis else 'no':>7}  {tot:>12}  {sal:>6}  {dis:>12}")
    if faltan:
        falla("H", f"tareas sin datos: {faltan}")
    reales = [f["real"] for f in filas_h]
    tots = [f["total"] for f in filas_h]
    sals = [f["salida"] for f in filas_h]
    sin_dis = [f["total"] - f["diseno"] for f in filas_h]
    MEDIDO["H: ADR con ## Coste y minutos Real"] = f"{len(filas_h)}/{len(H_TAREAS)}"
    MEDIDO["H: ADR cuyo ## Coste incluye la fila de diseno"] = f"{sum(f['con_diseno'] for f in filas_h)}/{len(filas_h)}"
    MEDIDO["H: minutos Real por ADR (TASK-002..TASK-013)"] = ", ".join(str(x) for x in reales)
    MEDIDO["H: minutos Real: suma / mediana / p75"] = f"{sum(reales)} / {mediana(reales)} / {p75(reales)}"
    MEDIDO["H: minutos Real: minimo / maximo"] = f"{min(reales)} / {max(reales)}"
    MEDIDO["H: minutos estimados: suma"] = str(sum(f["est"] for f in filas_h if f["est"] is not None))
    MEDIDO["H: ADR con Real mayor que lo estimado"] = \
        f"{sum(1 for f in filas_h if f['est'] is not None and f['real'] > f['est'])}/{len(filas_h)}"
    MEDIDO["H: tokens total por ADR (con diseno): mediana / p75"] = f"{mediana(tots)} / {p75(tots)}"
    MEDIDO["H: tokens total por ADR sin la fase de diseno: mediana / p75"] = f"{mediana(sin_dis)} / {p75(sin_dis)}"
    MEDIDO["H: tokens salida por ADR: mediana / p75"] = f"{mediana(sals)} / {p75(sals)}"
    MEDIDO["H: tokens total: suma"] = str(sum(tots))
    MEDIDO["H: rondas de revision por ADR (rondas: ADR)"] = reparto([f["rondas"] for f in filas_h])
    print(f"minutos Real: suma {sum(reales)} ; mediana {mediana(reales)} ; p75 {p75(reales)} ; "
          f"de {min(reales)} a {max(reales)} (excluyen el diseno: «Real» va de la rama abierta al merge)")
    print(f"tokens total (incluye la fila de diseno de ## Coste): mediana {mediana(tots)} ; p75 {p75(tots)} ; "
          f"sin diseno: mediana {mediana(sin_dis)} ; p75 {p75(sin_dis)}")
    print(f"tokens salida: mediana {mediana(sals)} ; p75 {p75(sals)}")
    return {"min": (mediana(reales), p75(reales)), "tok": (mediana(tots), p75(tots)),
            "sal": (mediana(sals), p75(sals))}, tareas


# ---------------------------------------------------------------------------
# A: estudio de adopcion de OpenDockTrading
# ---------------------------------------------------------------------------

RX_SIN_CLASIFICAR = r"\btaskctl\b|TaskCode|taskcode"
EXT_CODIGO = (".py", ".rs", ".bat", ".sh", ".js", ".ts", ".ps1")


def sin_clasificar(copia, rev):
    """Lo que nombra TaskCode fuera de tareas/ y F1-11 no clasifico (mismo criterio que la PoC A de F1-12)."""
    r = F12.git(copia, "grep", "-n", "-I", "-E", RX_SIN_CLASIFICAR, rev, "--", ".", ":!tareas", check=False)
    por = {}
    for l in r.stdout.decode("utf-8", "replace").splitlines():
        _, ruta, num, _ = l.split(":", 3)
        por.setdefault(ruta, []).append(int(num))
    clasificados = {p[0] for p in F12.PARCHES_ODT} | {".taskcode/config.yml"}
    return por, {k: v for k, v in sorted(por.items()) if k not in clasificados}


def informe(copia, rev, capacidades):
    """Informe ampliado de `tc2 actualizar --simular` (solo lectura). Devuelve el dict y su hash."""
    tareas = F12.tareas_016(copia, rev)
    todos, sin = sin_clasificar(copia, rev)
    inf = {
        "base": rev,
        "capacidades": sorted(capacidades),
        "uso": F12.uso_campos(tareas),
        "uso_medido": uso_medido(tareas),
        "por_carpeta": F12.contar_por_carpeta(tareas),
        "hecho_a_mano": F12.hecho_a_mano(copia, rev, capacidades),
        "sin_clasificar": {"total": len(todos), "ficheros": sin},
        "max_id": max(int(re.sub(r"\D", "", t)) for t in tareas),
    }
    canon = json.dumps(inf, sort_keys=True, ensure_ascii=False, separators=(",", ":"), default=list)
    return inf, hashlib.sha256(canon.encode("utf-8")).hexdigest()


def proponer_hito(inf, inf_hash, presupuesto_adr, presupuesto_tarea):
    """Funcion pura: un elemento por hueco. Bloqueantes (tareas de la 0.16 antes de migrar) aparte."""
    elems, n = [], inf["max_id"]
    for h in inf["hecho_a_mano"]:
        if h["clase"] != "parche" or h["accion"] == "retirar":
            continue            # lo propio se queda; lo que se retira solo no pide trabajo
        n += 1
        if h["accion"] == "queda: hueco bloqueante":
            fase, titulo = "antes de migrar (tarea de la 0.16)", f"{h['fichero']}: leer la carpeta fija o tc2 tareas --json"
        elif h["accion"] == "retirar con sustitucion":
            fase, titulo = "tras migrar (tc2 import)", f"{h['fichero']} ({h['donde']}): sustituir por {h['cubierto_por']}"
        else:
            fase, titulo = "tras migrar (tc2 import)", f"{h['fichero']} ({h['donde']}): hueco sin {h['cubierto_por']}"
        elems.append({"id": f"TASK-{n:03d}", "clase": "tarea", "fase": fase, "titulo": titulo,
                      "bloqueante": h["accion"] == "queda: hueco bloqueante", "presupuesto": presupuesto_tarea})
    grupos = {}
    for f in inf["sin_clasificar"]["ficheros"]:
        grupos.setdefault("codigo" if f.endswith(EXT_CODIGO) else "documentacion", []).append(f)
    for g in sorted(grupos):
        n += 1
        elems.append({"id": f"TASK-{n:03d}", "clase": "ADR", "fase": "tras migrar (tc2 import)",
                      "titulo": f"clasificar lo que nombra TaskCode ({g}: {len(grupos[g])} ficheros)",
                      "bloqueante": False, "presupuesto": presupuesto_adr})
    return {"informe_hash": inf_hash, "elementos": elems}


def suma_presupuesto(elems, clase):
    ps = [e["presupuesto"] for e in elems if e["clase"] == clase]
    if not ps or any(p is None for p in ps):
        return "no medido" if ps else "0"
    return " / ".join(f"{fmt(sum(p[i][0] for p in ps))}-{fmt(sum(p[i][1] for p in ps))}" for i in range(len(ps[0])))


def seccion_a(personal, trabajo, h):
    sep("A: estudio de adopcion de una copia de OpenDockTrading (commit fijo)")
    copia = F12.preparar_copia(personal, trabajo, "odt")
    rev = F12.FIJO_ODT
    print(f"commit fijo {rev[:9]} (develop de OpenDockTrading el 2026-11-14; F1-12 midio el mismo)")
    MEDIDO["OpenDockTrading: commit fijo"] = rev[:7]
    h0 = F12.hash_dir(copia)
    inf, ih = informe(copia, rev, F12.CAP_POC)
    inf_b, ih_b = informe(copia, rev, F12.CAP_POC)
    inf2, ih2 = informe(copia, rev, F12.CAP_2_0)
    u = inf["uso_medido"]
    print(f"tareas por carpeta 00..05: {inf['por_carpeta']} ; total {u['tareas']}")
    MEDIDO["OpenDockTrading: tareas por carpeta (00..05)"] = inf["por_carpeta"]
    for etiqueta, i in (("sin 2.0", inf), ("2.0.0 prevista", inf2)):
        filas = i["hecho_a_mano"]
        ret = sum(x["lineas"] for x in filas if x["accion"].startswith("retirar"))
        queda = sum(x["lineas"] for x in filas if not x["accion"].startswith("retirar"))
        solas = sum(x["lineas"] for x in filas if x["accion"] == "retirar")
        sust = sum(x["lineas"] for x in filas if x["accion"] == "retirar con sustitucion")
        bloq = sum(x["lineas"] for x in filas if x["accion"] == "queda: hueco bloqueante")
        print(f"-- hecho a mano, {etiqueta}: se retiran / se quedan {ret} / {queda} ; solas / con sustitucion "
              f"{solas} / {sust} ; lineas en huecos bloqueantes {bloq}")
        MEDIDO[f"F1-12 recalculado, {etiqueta}: lineas que se retiran / se quedan"] = f"{ret} / {queda}"
        MEDIDO[f"F1-12 recalculado, {etiqueta}: retiradas solas / con sustitucion"] = f"{solas} / {sust}"
        MEDIDO[f"F1-12 recalculado, {etiqueta}: lineas en huecos bloqueantes"] = str(bloq)
    sc = inf["sin_clasificar"]
    print(f"-- sin clasificar: ficheros que nombran TaskCode fuera de tareas/ {sc['total']} ; sin clasificar "
          f"{len(sc['ficheros'])}")
    MEDIDO["F1-12 recalculado: ficheros que nombran TaskCode / sin clasificar"] = f"{sc['total']} / {len(sc['ficheros'])}"
    sin_uso = [k for k, n in inf["uso"].items() if n == 0]
    MEDIDO["F1-12 recalculado: campos de la 0.16 sin uso"] = str(len(sin_uso))
    # uso medido
    print(f"-- uso medido: tareas con ## Coste {u['con_coste']}/{u['tareas']} ; con minutos Real {u['con_real']}/{u['tareas']}")
    MEDIDO["uso medido: tareas con ## Coste / con minutos Real"] = f"{u['con_coste']}/{u['tareas']} / {u['con_real']}/{u['tareas']}"
    if "tokens_por_fase" in u:
        tf = u["tokens_por_fase"]
        print("   tokens por fase (total / salida): " + " ; ".join(f"{k} {v[0]} / {v[1]}" for k, v in tf.items()))
        for k, v in tf.items():
            MEDIDO[f"uso medido: tokens de {k} (total / salida)"] = f"{v[0]} / {v[1]}"
        print(f"   tokens por tarea: mediana {u['tokens_tarea'][0]} ; p75 {u['tokens_tarea'][1]}")
        MEDIDO["uso medido: tokens total por tarea: mediana / p75"] = f"{u['tokens_tarea'][0]} / {u['tokens_tarea'][1]}"
        print(f"   rondas de revision por tarea (rondas: tareas): {u['rondas']}")
        MEDIDO["uso medido: rondas de revision por tarea (rondas: tareas)"] = u["rondas"]
    else:
        MEDIDO["uso medido: tokens"] = "no medido"
    if "minutos" in u:
        print(f"   minutos Real: suma {u['minutos'][0]} ; mediana {u['minutos'][1]} ; p75 {u['minutos'][2]}")
        MEDIDO["uso medido: minutos Real: suma / mediana / p75"] = " / ".join(str(x) for x in u["minutos"])
    else:
        MEDIDO["uso medido: minutos Real"] = "no medido"
    print(f"   verificacion: tareas con ## Verificacion ligera {u['verificacion_ligera']} ; "
          f"registro de verificacion por tarea: no medido (la 0.16 no lo guarda en tarea.md)")
    MEDIDO["uso medido: tareas con ## Verificacion ligera"] = str(u["verificacion_ligera"])
    # precedente del propio proyecto: sus tareas ADR (titulo «ADR-NNN»), para contrastar la regla con otro proyecto
    adr_odt = {tid: t for tid, t in F12.tareas_016(copia, rev).items() if re.match(r"ADR-\d+", str(t["fm"].get("titulo")))}
    ua = uso_medido(adr_odt)
    print(f"-- ADR propios de OpenDockTrading: {', '.join(sorted(adr_odt))} ; con ## Coste {ua['con_coste']} ; "
          f"con minutos Real {ua['con_real']}")
    for tid, t in sorted(adr_odt.items()):
        c = coste(t["texto"]) or []
        print(f"   {tid}: Real {minutos(t['texto'])} min ; tokens total {sum(f['total'] for f in c)} ; rondas {rondas(c)}")
    MEDIDO["ADR de OpenDockTrading (titulo ADR-NNN): tareas"] = ", ".join(sorted(adr_odt))
    MEDIDO["ADR de OpenDockTrading: con ## Coste / con minutos Real"] = f"{ua['con_coste']} / {ua['con_real']}"
    if "minutos" in ua:
        print(f"   minutos Real: suma {ua['minutos'][0]} ; mediana {ua['minutos'][1]} ; p75 {ua['minutos'][2]}")
        MEDIDO["ADR de OpenDockTrading: minutos Real: suma / mediana / p75"] = " / ".join(str(x) for x in ua["minutos"])
    if "tokens_tarea" in ua:
        print(f"   tokens total por ADR: mediana {ua['tokens_tarea'][0]} ; p75 {ua['tokens_tarea'][1]} ; "
              f"rondas (rondas: ADR) {ua['rondas']}")
        MEDIDO["ADR de OpenDockTrading: tokens total: mediana / p75"] = f"{ua['tokens_tarea'][0]} / {ua['tokens_tarea'][1]}"
        MEDIDO["ADR de OpenDockTrading: rondas de revision (rondas: ADR)"] = ua["rondas"]
    # hash del informe en dos llamadas
    MEDIDO["informe: hash en dos llamadas"] = "igual" if ih == ih_b and inf == inf_b else "distinto"
    if ih != ih_b:
        falla("A", "el informe no es determinista")
    # proponer_hito
    p_adr = (h["min"], h["tok"])
    p_tarea = ((u["minutos"][1], u["minutos"][2]), u["tokens_tarea"]) if "minutos" in u and "tokens_tarea" in u else None
    props = {}
    for etiqueta, i, hh in (("sin 2.0", inf, ih), ("2.0.0 prevista", inf2, ih2)):
        p = proponer_hito(i, hh, p_adr, p_tarea)
        p_b = proponer_hito(i, hh, p_adr, p_tarea)
        if p != p_b:
            falla("A", "proponer_hito no es determinista")
        props[etiqueta] = p
        el = p["elementos"]
        bl = [e for e in el if e["bloqueante"]]
        tareas_n = [e for e in el if e["clase"] == "tarea"]
        adrs = [e for e in el if e["clase"] == "ADR"]
        print(f"-- proponer_hito, {etiqueta}: {len(el)} elementos ; bloqueantes {len(bl)} ; tareas {len(tareas_n)} ; ADR {len(adrs)}")
        for e in el:
            print(f"   {e['id']} {e['clase']:5} {'BLOQUEANTE ' if e['bloqueante'] else ''}{e['fase']}: {e['titulo']}")
        huecos = sum(1 for x in i["hecho_a_mano"] if x["clase"] == "parche" and x["accion"] != "retirar")
        grupos = len({("codigo" if f.endswith(EXT_CODIGO) else "documentacion") for f in i["sin_clasificar"]["ficheros"]})
        if len(el) != huecos + grupos:
            falla("A", f"{etiqueta}: {len(el)} elementos para {huecos} huecos y {grupos} grupos")
        if len(bl) != sum(1 for x in i["hecho_a_mano"] if x["accion"] == "queda: hueco bloqueante"):
            falla("A", f"{etiqueta}: bloqueantes mal separados")
        MEDIDO[f"proponer_hito, {etiqueta}: elementos / bloqueantes / tareas / ADR"] = \
            f"{len(el)} / {len(bl)} / {len(tareas_n)} / {len(adrs)}"
        MEDIDO[f"proponer_hito, {etiqueta}: presupuesto ADR (min / tokens, mediana-p75)"] = suma_presupuesto(el, "ADR")
        MEDIDO[f"proponer_hito, {etiqueta}: presupuesto tareas (min / tokens, mediana-p75)"] = suma_presupuesto(el, "tarea")
        print(f"   presupuesto ADR (min / tokens, mediana-p75 de H): {suma_presupuesto(el, 'ADR')} ; "
              f"tareas (mediana-p75 del propio proyecto): {suma_presupuesto(el, 'tarea')}")
    h1 = F12.hash_dir(copia)
    camb = F12.diferencias(h0, h1)
    print(f"hash de la copia (con .git) antes y despues del estudio: {'igual' if not camb else 'DISTINTO'} ; "
          f"ficheros cambiados {len(camb)}")
    MEDIDO["estudio: ficheros cambiados en la copia (con .git)"] = str(len(camb))
    if camb:
        falla("A", f"el estudio cambio la copia: {camb[:5]}")
    return copia, inf, ih, props


# ---------------------------------------------------------------------------
# R: registro hito-decisiones.yml y hito_estado
# ---------------------------------------------------------------------------

def q(v):
    return F12.q(v)


def registro_yml(origen, informe_hash, tareas, no_necesario=None):
    nn = "null" if no_necesario is None else \
        "{" + ", ".join(f"{k}: {q(no_necesario[k])}" for k in ("por", "fecha", "motivo") if k in no_necesario) + "}"
    return (f"formato: 2\norigen: {q(origen)}\ninforme_hash: {q(informe_hash)}\n"
            f"tareas: [{', '.join(q(t) for t in tareas)}]\nno_necesario: {nn}\n")


def leer_en(repo, ref, ruta):
    r = F12.git(repo, "cat-file", "-p", f"{ref}:{ruta}", check=False)
    return r.stdout.decode("utf-8") if r.returncode == 0 else None


def hito_estado(repo, ref=RAMA_ESTADO):
    """(estado, motivo). El estado se calcula, no se guarda. Un registro roto sale `abierto` (falla cerrado)."""
    texto = leer_en(repo, ref, "hito-decisiones.yml")
    if texto is None:
        return "ausente", "no hay hito-decisiones.yml"
    try:
        reg = yaml.safe_load(texto)
    except yaml.YAMLError:
        return "abierto", "registro ilegible"
    if not isinstance(reg, dict) or reg.get("formato") != 2 or reg.get("origen") not in ("init", "actualizar"):
        return "abierto", "registro sin formato 2 u origen"
    tareas = reg.get("tareas")
    if not isinstance(tareas, list) or not all(isinstance(t, str) and re.fullmatch(r"TASK-\d+[a-z]?", t) for t in tareas):
        return "abierto", "lista de tareas invalida"
    nn = reg.get("no_necesario")
    if nn is not None:
        if (isinstance(nn, dict) and all(isinstance(nn.get(k), str) and nn[k].strip() for k in ("por", "fecha", "motivo"))
                and isinstance(reg.get("informe_hash"), str) and reg["informe_hash"].strip()):
            return "no-necesario", f"firmado por {nn['por']}"
        return "abierto", "no_necesario sin por, fecha, motivo o informe_hash"
    if not tareas:
        return "abierto", "lista vacia sin firma de no necesario"
    pendientes = []
    for t in tareas:
        md = leer_en(repo, ref, f"{t}/tarea.md")
        if md is None:
            pendientes.append(f"{t} no existe")
            continue
        try:
            est = yaml.safe_load(F12.partir(md)[0]).get("estado")
        except Exception:      # noqa: BLE001 - frontmatter roto cuenta como pendiente
            est = None
        if est not in FIN:
            pendientes.append(f"{t} {est}")
    if pendientes:
        return "abierto", f"{len(tareas) - len(pendientes)} de {len(tareas)} terminadas o descartadas"
    return "cerrado", f"{len(tareas)} de {len(tareas)} terminadas o descartadas"


def tarea_f2(tid, titulo, estado, carril=None):
    return (f"---\nformato: 2\nid: {q(tid)}\ntitulo: {q(titulo)}\ncarril: {q(carril)}\nestado: {q(estado)}\n---\n"
            f"## Objetivo\n\nTarea sintetica de la PoC de F1-18.\n")


def commit_estado(repo, ficheros, borrar=()):
    """Escribe y borra ficheros en la rama de estado de prueba y hace un commit. Devuelve el SHA."""
    for ruta, texto in ficheros.items():
        p = os.path.join(repo, ruta)
        os.makedirs(os.path.dirname(p) or repo, exist_ok=True)
        with open(p, "w", encoding="utf-8", newline="\n") as f:
            f.write(texto)
    for ruta in borrar:
        F12.git(repo, "rm", "-q", "--ignore-unmatch", ruta)
    F12.git(repo, "add", "-A")
    F12.git(repo, "commit", "-q", "--allow-empty", "-m", "poc: estado de prueba")
    return F12.git(repo, "rev-parse", "HEAD").strip()


def hito_tc2(tareas):
    """Tareas del hito de TaskCodeTwo: las ADR de la Fase 1 (titulo F1-N ... ADR)."""
    return sorted(tid for tid, t in tareas.items() if re.match(r"F1-\d+ ADR", str(t["fm"].get("titulo"))))


def seccion_r(trabajo, tareas_tc2, ih_odt):
    sep("R: registro hito-decisiones.yml en una rama de estado de prueba y hito_estado")
    repo = os.path.join(trabajo, "estado-prueba")
    F12.run(["git", "init", "-q", repo])
    F12.git(repo, "symbolic-ref", "HEAD", f"refs/heads/{RAMA_ESTADO}")
    # las tareas reales de TaskCodeTwo en el commit fijo, aplanadas, con el estado de su carpeta
    fich = {f"{tid}/tarea.md": tarea_f2(tid, t["fm"].get("titulo"), F12.CARPETA_ESTADO[t["carpeta"]])
            for tid, t in sorted(tareas_tc2.items())}
    hito = hito_tc2(tareas_tc2)
    term = [t for t in hito if F12.CARPETA_ESTADO[tareas_tc2[t]["carpeta"]] in FIN]
    casos = []
    sha = commit_estado(repo, fich)
    casos.append(("ausente", "sin registro", sha))
    sha = commit_estado(repo, {"hito-decisiones.yml": registro_yml("init", "sin-informe", hito)})
    casos.append(("abierto", f"TaskCodeTwo real: las {len(hito)} ADR de la Fase 1", sha))
    sha = commit_estado(repo, {"hito-decisiones.yml": registro_yml("init", "sin-informe", term)})
    casos.append(("cerrado", f"solo las {len(term)} ADR terminadas", sha))
    firma = {"por": "carlos (persona)", "fecha": "2026-11-14", "motivo": "proyecto sin decisiones pendientes"}
    sha = commit_estado(repo, {"hito-decisiones.yml": registro_yml("actualizar", ih_odt, [], firma)})
    casos.append(("no-necesario", "lista vacia, firmado con motivo e informe_hash", sha))
    # registros rotos o editados a mano: tienen que salir abierto
    rotos = [
        ("firma sin motivo", registro_yml("actualizar", ih_odt, [], {"por": "carlos", "fecha": "2026-11-14"})),
        ("firma sin informe_hash", registro_yml("actualizar", "", [], firma)),
        ("lista vacia sin firma", registro_yml("init", "x", [])),
        ("tarea que no existe", registro_yml("init", "x", term + ["TASK-999"])),
        ("YAML ilegible", "formato: 2\ntareas: [\n"),
        ("sin formato 2", registro_yml("init", "x", term).replace("formato: 2", "formato: 1")),
    ]
    for nombre, texto in rotos:
        sha = commit_estado(repo, {"hito-decisiones.yml": texto})
        casos.append(("abierto", f"roto: {nombre}", sha))
    res = {}
    for esperado, desc, sha in casos:
        e1, m1 = hito_estado(repo, sha)
        e2, m2 = hito_estado(repo, sha)
        ok = e1 == esperado and (e1, m1) == (e2, m2)
        print(f"{desc}: {e1} ({m1}) ; segunda llamada {'igual' if (e1, m1) == (e2, m2) else 'DISTINTA'}"
              f"{'' if ok else ' ; FALLO: se esperaba ' + esperado}")
        if not ok:
            falla("R", f"{desc}: {e1}, se esperaba {esperado}")
        res[desc] = (e1, m1, sha)
        if not desc.startswith("roto"):
            MEDIDO[f"hito_estado: {desc}"] = f"{e1} ({m1})"
    MEDIDO["hito_estado: registros rotos que salen abierto"] = \
        f"{sum(1 for d, v in res.items() if d.startswith('roto') and v[0] == 'abierto')}/{len(rotos)}"
    return repo, res, hito, term


# ---------------------------------------------------------------------------
# P: puerta simulada
# ---------------------------------------------------------------------------

def puerta(tid, carril, estado, tareas_hito):
    """(codigo, motivo) de la puerta: parada de `tc2 start` o check `estado` (misma funcion pura)."""
    if estado in ("cerrado", "no-necesario"):
        return 0, f"hito {estado}"
    if tid in tareas_hito:
        return 0, "tarea del hito"
    if carril == "errata":
        return 0, "carril errata"
    return COD_PRECONDICION, f"hito de decisiones {estado}: cierralo o firma `tc2 hito no-necesario` (ver `tc2 hito estado`)"


def carril_por_ficheros(repo, merge):
    """errata si todas las rutas (con los dos nombres de un renombrado) fuera de tareas/ estan bajo docs/ (F1-10)."""
    out = F12.git(repo, "diff", "-M", "--name-status", f"{merge}^1", merge)
    rutas = []
    for l in out.splitlines():
        partes = l.split("\t")
        rutas += partes[1:]
    rutas = [r for r in rutas if not r.startswith("tareas/")]
    if not rutas:
        return None, rutas
    return ("errata" if all(r.startswith("docs/") for r in rutas) else "otro"), rutas


def merges_de_tarea(repo, rev):
    out = F12.git(repo, "log", "--merges", "--first-parent", "--format=%H%x09%s", rev)
    res = []
    for l in out.splitlines():
        sha, asunto = l.split("\t", 1)
        m = re.search(r"task-(\d+)", asunto)
        if m:
            res.append((f"TASK-{int(m.group(1)):03d}", sha))
    return sorted(res)


def simular(repo_tareas, tareas, merges, estado, tareas_hito, repo_merges):
    """Una pasada de la puerta: `start` sobre planificadas y en curso, check `estado` sobre los PR ya mergeados."""
    start = {}
    for tid, t in sorted(tareas.items()):
        if F12.CARPETA_ESTADO[t["carpeta"]] not in ("planificada", "en-curso"):
            continue
        start[tid] = puerta(tid, t["fm"].get("carril"), estado, tareas_hito)
    check = {}
    for tid, sha in merges:
        carril, _ = carril_por_ficheros(repo_merges, sha)
        check[tid] = (carril,) + puerta(tid, carril, estado, tareas_hito)
    return start, check


def resumen_puerta(start, check):
    s_pasa = [t for t, v in start.items() if v[0] == 0]
    s_neg = [t for t, v in start.items() if v[0] != 0]
    c_err = [t for t, v in check.items() if v[0] == "errata"]
    c_pasa = [t for t, v in check.items() if v[1] == 0]
    c_rojo = [t for t, v in check.items() if v[1] != 0]
    return s_pasa, s_neg, c_err, c_pasa, c_rojo


def seccion_p(repo_tc2, tareas_tc2, hito_tc2_ids, copia_odt, props_odt):
    sep("P: puerta simulada (parada local en start y check estado del PR) sobre tareas reales")
    tareas_odt = F12.tareas_016(copia_odt, F12.FIJO_ODT)
    merges_tc2 = merges_de_tarea(repo_tc2, FIJO_TC2)
    merges_odt = merges_de_tarea(copia_odt, F12.FIJO_ODT)
    hito_odt = [e["id"] for e in props_odt["sin 2.0"]["elementos"]]
    declaradas = sum(1 for t in list(tareas_tc2.values()) + list(tareas_odt.values()) if t["fm"].get("carril"))
    print(f"tareas con carril declarado (la 0.16 no tiene el campo): {declaradas}")
    MEDIDO["P: tareas con carril declarado en TaskCodeTwo y OpenDockTrading"] = str(declaradas)
    for proyecto, repo, tareas, merges, hito in (
            ("TaskCodeTwo", repo_tc2, tareas_tc2, merges_tc2, hito_tc2_ids),
            ("OpenDockTrading", copia_odt, tareas_odt, merges_odt, hito_odt)):
        print(f"-- {proyecto}: tareas del hito {len(hito)} ; PR de tarea mergeados {len(merges)}")
        for estado in ("ausente", "abierto", "cerrado", "no-necesario"):
            h_ef = [] if estado == "ausente" else hito      # sin registro no hay tareas del hito
            r1 = simular(repo, tareas, merges, estado, h_ef, repo)
            r2 = simular(repo, tareas, merges, estado, h_ef, repo)
            if r1 != r2:
                falla("P", f"{proyecto} {estado}: dos llamadas distintas")
            s_pasa, s_neg, c_err, c_pasa, c_rojo = resumen_puerta(*r1)
            codigos = sorted({v[0] for v in r1[0].values()} | {v[1] for v in r1[1].values()})
            print(f"   hito {estado}: start pasa {len(s_pasa)} / se niega {len(s_neg)} ; check estado: errata "
                  f"{len(c_err)}, verde {len(c_pasa)}, rojo {len(c_rojo)} ; codigos {codigos} ; "
                  f"dos llamadas {'iguales' if r1 == r2 else 'DISTINTAS'}")
            MEDIDO[f"P {proyecto}, hito {estado}: start pasa / se niega"] = f"{len(s_pasa)} / {len(s_neg)}"
            MEDIDO[f"P {proyecto}, hito {estado}: check estado verde / rojo"] = f"{len(c_pasa)} / {len(c_rojo)}"
            if estado in ("abierto", "ausente"):
                # comprobaciones: abierto o ausente niega toda tarea de codigo y deja pasar hito y errata
                malos = [t for t, v in r1[0].items() if (v[0] == 0) != (t in h_ef)]
                malos += [t for t, v in r1[1].items() if (v[1] == 0) != (t in h_ef or v[0] == "errata")]
                if malos:
                    falla("P", f"{proyecto} {estado}: puerta incoherente en {malos[:5]}")
                if {v[0] for v in r1[0].values() if v[0]} - {COD_PRECONDICION}:
                    falla("P", f"{proyecto}: codigo distinto de {COD_PRECONDICION}")
            if estado == "abierto":
                print(f"      start pasa: {', '.join(s_pasa) or 'ninguna'}")
                print(f"      PR errata (solo docs/): {', '.join(c_err) or 'ninguno'}")
                print(f"      PR en rojo: {', '.join(c_rojo) or 'ninguno'}")
                MEDIDO[f"P {proyecto}, hito abierto: start pasa"] = ", ".join(s_pasa) or "ninguna"
                MEDIDO[f"P {proyecto}: PR con carril errata por ficheros"] = ", ".join(c_err) or "ninguno"
            elif estado not in ("abierto", "ausente") and (s_neg or c_rojo):
                falla("P", f"{proyecto} {estado}: la puerta niega con el hito {estado}")
    return merges_tc2, merges_odt


# ---------------------------------------------------------------------------
# G: el ADR
# ---------------------------------------------------------------------------

SECCIONES = ["## Contexto", "## Alternativas", "## Pruebas de concepto", "## Cómo se ve en GitHub", "## Decisión",
             "## Requisitos y evidencia", "## Consecuencias", "## Fuera de alcance", "## Divergencias con el plan",
             "## Preguntas abiertas", "## No verificado"]
ID_PROPIO = "F1-18"


def errores_adr(adr, ruta, repo_tc2, medido, con_indice=True):
    """Lista de errores del ADR frente a lo medido (funcion pura: la usan G y sus mutaciones)."""
    err = []
    try:
        _, filas = F12.filas_tabla(F12.entre_marcadores(adr, "cifras"))
    except SystemExit as e:
        return [str(e)], {}
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
        res = F12.entre_marcadores(adr, "resumen")
    except SystemExit as e:
        return err + [str(e)], tabla
    valores = set(tabla.values()) | {p for v in tabla.values() for p in re.split(r"\s*(?:/|->|,|\(|\)|y|de)\s*", v) if p}
    for b in re.findall(r"\*\*([^*]+)\*\*", res):
        if re.search(r"\d", b) and b not in valores:
            err.append(f"el resumen cita {b!r}, que no es un valor de la tabla")
    faltan = [s for s in SECCIONES if not re.search(rf"^{re.escape(s)}\s*$", adr, re.M)]
    if faltan:
        err.append(f"faltan secciones {faltan}")
    if not re.search(r"^- Estado: \*\*aceptada\*\*", adr, re.M):
        err.append("el estado no es aceptada")
    req = open(os.path.join(repo_tc2, "docs", "plan", "REQUISITOS-FUNCIONALES.md"), encoding="utf-8").read()
    plan = open(os.path.join(repo_tc2, "docs", "plan", "PLAN-TASKCODE-2.md"), encoding="utf-8").read()
    ids_r = sorted(set(re.findall(r"\b(R[RFN]-\d+)\b", adr)))
    ids_f = sorted(set(re.findall(r"\b(F\d+-\d+[a-z]?)\b", adr)))
    malos = [i for i in ids_r if not re.search(rf"\|\s*{i}\s*\|", req)]
    malos += [i for i in ids_f if not re.search(rf"\|\s*{i}\s*\|", plan) and i != ID_PROPIO]
    if malos:
        err.append(f"IDs que no existen: {malos}")
    div = re.search(r"^## Divergencias con el plan\s*$(.*?)(?=^## )", adr, re.S | re.M)
    if ID_PROPIO not in re.findall(r"\|\s*(F\d+-\d+)\s*\|", plan) and not (div and f"| {ID_PROPIO} |" in div.group(1)):
        err.append(f"{ID_PROPIO} no esta en el plan y «Divergencias con el plan» no propone su fila")
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
        if "| [F1-18](F1-18-hito-de-decisiones.md) |" not in idx:
            err.append("falta la fila en docs/adr/README.md")
    return err, tabla


def mutaciones(adr):
    clave = "H: minutos Real: suma / mediana / p75"
    m = re.search(rf"^\| {re.escape(clave)} \| `([^`]+)` \| H \|$", adr, re.M)
    fila = m.group(0) if m else "(sin fila)"
    return [
        ("cifra-cambiada", adr.replace(fila, fila.replace(m.group(1), m.group(1) + "0")) if m else adr,
         f"{clave}: el ADR dice"),
        ("fila-borrada", adr.replace(fila + "\n", ""), f"cifra medida que falta en la tabla: {clave}"),
        ("fila-sin-medida", adr.replace("<!-- cifras:fin -->", "| cifra inventada | `1` | H |\n<!-- cifras:fin -->"),
         "fila de la tabla sin medida: cifra inventada"),
        ("tiempo-de-reloj-en-la-tabla", adr.replace("<!-- cifras:fin -->",
                                                     "| PoC: segundos de reloj | `95` | G |\n<!-- cifras:fin -->"),
         "fila de la tabla sin medida: PoC: segundos de reloj"),
        ("resumen-inventado", adr.replace("<!-- resumen:fin -->", "Cifra suelta: **9999**.\n<!-- resumen:fin -->"),
         "el resumen cita '9999'"),
        ("id-inexistente", adr.replace("## Contexto", "## Contexto\n\nVer F7-99.", 1), "IDs que no existen: ['F7-99']"),
        ("enlace-a-tareas", adr.replace("## Contexto", "## Contexto\n\n[x](../../tareas/x.md)", 1), "enlaces prohibidos"),
        ("enlace-roto", adr.replace("## Contexto", "## Contexto\n\n[x](F1-99-no-existe.md)", 1),
         "enlace que no resuelve: F1-99-no-existe.md"),
        ("seccion-borrada", adr.replace("## No verificado", "## Sin verificar"), "faltan secciones"),
        ("sin-fila-f1-18-propuesta", re.sub(r"^(\s*)\| F1-18 \|", r"| F1-18bis |", adr, flags=re.M),
         "F1-18 no esta en el plan"),
        ("estado-propuesta", adr.replace("- Estado: **aceptada**", "- Estado: **propuesta**"), "el estado no es aceptada"),
    ]


def seccion_g(repo_tc2):
    sep("G: el ADR (cifras, resumen, secciones, IDs, enlaces e indice) y sus mutaciones de control")
    ruta = os.path.join(repo_tc2, "docs", "adr", "F1-18-hito-de-decisiones.md")
    if not os.path.exists(ruta):
        falla("G", "no existe el ADR")
        return
    adr = open(ruta, encoding="utf-8").read()
    err, tabla = errores_adr(adr, ruta, repo_tc2, MEDIDO)
    print(f"cifras en la tabla del ADR: {len(tabla)} ; medidas: {len(MEDIDO)}")
    for e in err:
        falla("G", e)
    print(f"errores del ADR: {len(err)}")
    for nombre, mutado, esperado in mutaciones(adr):
        if mutado == adr:
            falla("G", f"la mutacion {nombre} no cambia el ADR")
            continue
        errs, _ = errores_adr(mutado, ruta, repo_tc2, MEDIDO)
        ok = any(esperado in e for e in errs)
        print(f"  mutacion {nombre}: {'falla como se espera' if ok else 'NO FALLA'} "
              f"({str(next((e for e in errs if esperado in e), errs[:1]))[:110]})")
        if not ok:
            falla("G", f"la mutacion {nombre} no la detecta el guion")


# ---------------------------------------------------------------------------

MARCA = ".poc-f1-18"


def main():
    global F12
    args = sys.argv[1:]
    if len(args) != 3:
        raise SystemExit(__doc__)
    trabajo, repo_tc2, personal = (os.path.abspath(a) for a in args)
    sys.stdout.reconfigure(encoding="utf-8", newline="\n")
    F12 = cargar_f12(repo_tc2)
    F12.MARCA = MARCA
    F12.comprobar_trabajo(trabajo)      # bajo %LOCALAPPDATA%/pruebas-claude/<proyecto>/<motivo>, vacia o con la marca
    F12.borrar(trabajo)
    os.makedirs(trabajo)
    open(os.path.join(trabajo, MARCA), "w", encoding="utf-8").write("carpeta de trabajo de poc-f1-18.py\n")
    print(f"PoC F1-18 ; TaskCodeTwo {FIJO_TC2[:7]} ; OpenDockTrading {F12.FIJO_ODT[:9]} ; Python {sys.version.split()[0]} ; "
          f"PyYAML {yaml.__version__} ; {F12.git(trabajo, 'version').strip()}")
    h, tareas_tc2 = seccion_h(repo_tc2)
    tiempo("H")
    copia, inf, ih, props = seccion_a(personal, trabajo, h)
    tiempo("A")
    _, _, hito, _ = seccion_r(trabajo, tareas_tc2, ih)
    tiempo("R")
    seccion_p(repo_tc2, tareas_tc2, hito, copia, props)
    tiempo("P")
    seccion_g(repo_tc2)
    if F12.FALLOS:
        for f in F12.FALLOS:
            falla("F1-12", f)
    print()
    if FALLOS:
        print(f"RESULTADO: {len(FALLOS)} fallos")
        for f in FALLOS:
            print(f"  - {f}")
        sys.exit(1)
    print("RESULTADO: todo como se espera")


if __name__ == "__main__":
    main()

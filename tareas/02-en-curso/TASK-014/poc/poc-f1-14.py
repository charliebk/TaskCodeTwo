"""PoC de F1-14 (TASK-014): distribucion e instalacion, Rust frente a TypeScript medido.

Se ejecuta con `python -I -B`. Mide el MISMO comando (`tablero`, ver
especificacion-comando.md) escrito en Rust (poc-rust/) y en TypeScript
empaquetado con Node SEA (poc-ts/). Los dos se compilan desde las fuentes del
repo hacia la carpeta de trabajo; nada se compila ni se escribe en el repo.

ToolsaWeb y OpenDockTrading SOLO SE LEEN: `git clone --no-hardlinks` a la
carpeta de trabajo, en un commit fijo, y todo se ejecuta sobre las copias. No
se lanza `taskctl` ni ningun hook. No se descarga ninguna toolchain, target ni
imagen: se usa lo instalado y lo que falta sale «no verificado». La red solo
se usa para `npm ci` (dependencias fijadas en package-lock.json), para bajar
de crates.io las crates propias de Linux y macOS en el intento de compilacion
cruzada (X; Cargo.lock fijado) y para el GET anonimo a api.github.com del
propio comando (seccion T, sin comprobar).

(I) Inventario de herramientas con su version.
(B) Build en limpio de los dos, dos veces: sha256 igual o distinto.
(C) Copias de ToolsaWeb y OpenDockTrading en commit fijo; hash del arbol
    (con .git) antes y despues de todas las ejecuciones.
(S) Salidas: identicas byte a byte y con el mismo codigo; recuentos
    contrastados con PyYAML (lector estricto); clave duplicada en una tarea
    real copiada; ejecucion sin node en el PATH ni variables NODE_*.
(F) Firma Authenticode con un certificado autofirmado desechable creado con
    openssl en la carpeta de trabajo (sin tocar el almacen de certificados,
    comprobado antes y despues); ejecucion tras firmar; signtool verify /pa.
(X) Compilacion cruzada con lo instalado; ejecucion en Linux (Docker, WSL).
(M) Tamanos, comprimidos y peso de meter binarios en el repo del marketplace.
(D) Dependencias y lineas de codigo; regla D1a del plan aplicada.
(T) Tiempos de reloj: build, arranque en frio y en caliente. NO deterministas:
    van al log en su seccion y no se comprueban en la tabla del ADR; solo
    entran relaciones estables (p. ej. «Rust mas rapido en caliente»).
(G) El ADR: cifras, resumen, secciones, IDs, enlaces e indice, y copias
    mutadas que fallan cada una por su motivo.

Uso: python -I -B poc-f1-14.py <trabajo> <repo-TaskCodeTwo> <carpeta-Personal>
     python -I -B poc-f1-14.py --solo-g <medido.json> <repo-TaskCodeTwo>
       (solo G con las medidas guardadas por una ejecucion completa con
        --guardar-medido <medido.json>; para iterar la redaccion del ADR)
Sale con 0 solo si todo sale como se espera. Todo el log es determinista
salvo la seccion T. La carpeta de trabajo se borra al empezar y al terminar.
"""
import gzip
import hashlib
import json
import os
import re
import secrets
import shutil
import stat
import statistics
import subprocess
import sys
import time

import yaml

FIJO_TW = "8bebfd049c2af47fa0c05ff24af15e2d5f7dc382"    # ToolsaWeb, develop el 2026-11-14
FIJO_ODT = "8141a08a703ef1b549bdf4b6275adbac0c68e7cc"   # OpenDockTrading, develop (el de F1-12 y F1-18)
ESPERADO = {"ToolsaWeb": 404, "OpenDockTrading": 96}    # tarea.md medidos en F1-9
TARGETS_P12 = ["win-x64", "win-arm64", "darwin-x64", "darwin-arm64", "linux-x64", "linux-arm64"]
CRUZADOS_RUST = ["x86_64-unknown-linux-gnu", "aarch64-apple-darwin", "aarch64-pc-windows-msvc"]
N_CALIENTE = 20
MIB = 1024 * 1024
SIGNTOOL = r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe"
CN_PRUEBA = "TaskCodeTwo PoC desechable"
MARCA = ".poc-f1-14"
ADR_REL = os.path.join("docs", "adr", "F1-14-distribucion-e-instalacion.md")
FILA_INDICE = "| [F1-14](F1-14-distribucion-e-instalacion.md) |"

# Criterios 4 y 5 de D1a: no salen de un cronometro sino de lo anotado (DEPENDENCIAS.md, NOTAS.md);
# se declaran aqui con su motivo para que el ADR no pueda decir otra cosa sin que G lo vea.
CRITERIO_4 = ("Rust", "las dos librerias YAML estan mantenidas y rechazan duplicadas (F1-9), pero la cadena de "
                      "empaquetado de TypeScript depende de postject 1.0.0-alpha.6 (sin publicar desde 2023-05-19)")
CRITERIO_5 = ("ninguno", "minutos parecidos (~8 frente a ~9) y tokens no anotados por implementacion")

FALLOS = []
MEDIDO = {}
TIEMPOS = []
T0 = time.time()


def falla(seccion, texto):
    FALLOS.append(f"{seccion}: {texto}")
    print(f"FALLO en {seccion}: {texto}")


def medir(clave, valor, seccion):
    MEDIDO[clave] = (str(valor), seccion)


def sep(t):
    print()
    print(f"=== {t} ===")


def reloj(texto):
    TIEMPOS.append(texto)
    print(f"[tiempo] {texto} ({time.time() - T0:.0f} s desde el inicio)", file=sys.stderr, flush=True)


def largo(p):
    """Ruta con prefijo \\\\?\\ para rutas largas en Windows (las copias tienen rutas de mas de 260)."""
    p = os.path.abspath(p)
    return p if os.name != "nt" or p.startswith("\\\\?\\") else "\\\\?\\" + p


def borrar(ruta):
    def quitar_ro(f, p, _):
        os.chmod(p, stat.S_IWRITE)
        f(p)
    if os.path.lexists(largo(ruta)):
        shutil.rmtree(largo(ruta), onerror=quitar_ro)


def run(cmd, cwd=None, env=None, check=False, timeout=1800):
    r = subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, timeout=timeout)
    if check and r.returncode != 0:
        raise SystemExit(f"ERROR: {cmd[0]} {' '.join(cmd[1:4])}... salio con {r.returncode}: "
                         f"{r.stderr.decode('utf-8', 'replace')[-800:]}")
    return r


def git(repo, *args, check=True):
    r = run(["git", "-C", repo, "-c", "core.longpaths=true", "-c", "core.autocrlf=false", *args], check=check)
    return r.stdout.decode("utf-8", "replace") if check else r


def sha256_fichero(p):
    h = hashlib.sha256()
    with open(largo(p), "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def hash_arbol(raiz):
    """sha256 de (ruta relativa, sha256) de todos los ficheros, .git incluido; y numero de ficheros."""
    h = hashlib.sha256()
    n = 0
    base = largo(raiz)
    for d, subs, fs in os.walk(base):
        subs.sort()
        for f in sorted(fs):
            p = os.path.join(d, f)
            rel = os.path.relpath(p, base).replace("\\", "/")
            h.update(rel.encode() + b"\0" + sha256_fichero(p).encode() + b"\n")
            n += 1
    return h.hexdigest(), n


def mediana(xs):
    return statistics.median(xs)


def mib(n):
    return f"{n / MIB:.1f}".replace(".", ",")


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


# --------------------------------------------------------------------------- I

def version_de(cmd):
    try:
        r = run(cmd, timeout=60)
    except (OSError, subprocess.TimeoutExpired) as e:
        return f"no se ejecuta ({type(e).__name__})"
    texto = (r.stdout or r.stderr).decode("utf-8", "replace").replace("\x00", "").strip().splitlines()
    return texto[0].strip() if texto else f"sin salida (codigo {r.returncode})"


def seccion_i():
    sep("I: inventario de herramientas (where y --version)")
    herr = {
        "node": ["node", "--version"], "npm": ["npm.cmd", "--version"], "cargo": ["cargo", "--version"],
        "rustc": ["rustc", "--version"], "rustup": ["rustup", "--version"], "git": ["git", "--version"],
        "openssl": [r"C:\Program Files\Git\usr\bin\openssl.exe", "version"],
        "docker": ["docker", "--version"], "wsl": ["wsl", "--version"],
        "bun": ["bun", "--version"], "zig": ["zig", "version"], "deno": ["deno", "--version"],
    }
    inv = {}
    for nombre, cmd in herr.items():
        ruta = shutil.which(cmd[0]) if not os.path.isabs(cmd[0]) else (cmd[0] if os.path.exists(cmd[0]) else None)
        ver = version_de(cmd) if ruta else "no encontrado"
        if nombre == "wsl" and ruta:
            ver = ver.split(":")[-1].strip() if ":" in ver else ver
        inv[nombre] = ver
        print(f"{nombre:8s} {'encontrado' if ruta else 'NO encontrado':14s} {ver}")
    st = os.path.exists(SIGNTOOL)
    print(f"signtool {'encontrado' if st else 'NO encontrado':14s} {SIGNTOOL if st else '-'} (no esta en el PATH)")
    print(f"python   encontrado     {sys.version.split()[0]} ; PyYAML {yaml.__version__}")
    targets = run(["rustup", "target", "list", "--installed"]).stdout.decode().split()
    print(f"rustup targets instalados: {', '.join(targets)}")
    srv = run(["docker", "version", "--format", "{{.Server.Version}}"], timeout=60)
    docker_srv = "responde" if srv.returncode == 0 and srv.stdout.strip() else "no responde"
    print(f"docker daemon: {docker_srv}")
    distros = run(["wsl", "-l", "-q"], timeout=60).stdout.decode("utf-16-le", "replace").replace("\x00", "").split()
    print(f"wsl distribuciones: {', '.join(distros) or 'ninguna'}")
    medir("I: bun / zig / deno encontrados", f"{'si' if shutil.which('bun') else 'no'} / "
          f"{'si' if shutil.which('zig') else 'no'} / {'si' if shutil.which('deno') else 'no'}", "I")
    medir("I: signtool (Windows SDK 10.0.26100.0)", "si" if st else "no", "I")
    medir("I: openssl (de Git for Windows)", inv["openssl"], "I")
    medir("I: node", inv["node"], "I")
    medir("I: rustc", inv["rustc"], "I")
    medir("I: rustup targets instalados", ", ".join(targets), "I")
    medir("I: docker daemon", docker_srv, "I")
    medir("I: distribuciones WSL", ", ".join(distros) or "ninguna", "I")
    return {"targets": targets, "docker": docker_srv, "distros": distros, "signtool": st}


# --------------------------------------------------------------------------- B

def build_rust(fuentes, destino, rustflags=None):
    env = dict(os.environ, CARGO_TARGET_DIR=destino)
    if rustflags:
        env["RUSTFLAGS"] = rustflags
    t = time.perf_counter()
    run(["cargo", "build", "--release", "--locked", "--offline"], cwd=fuentes, env=env, check=True)
    return os.path.join(destino, "release", "tablero.exe"), time.perf_counter() - t


def build_ts(fuentes, destino):
    t = time.perf_counter()
    run(["node", os.path.join(fuentes, "scripts", "build.mjs"), destino], cwd=fuentes, check=True)
    return os.path.join(destino, "dist", "tablero.exe"), time.perf_counter() - t


def seccion_b(repo, trabajo):
    sep("B: build en limpio de los dos, dos veces (fuera del repo)")
    poc = os.path.join(repo, "tareas", "02-en-curso", "TASK-014", "poc")
    exes = {}
    for nombre, fn, sub in (("Rust", build_rust, "poc-rust"), ("TypeScript", build_ts, "poc-ts")):
        hashes = []
        for i in (1, 2):
            destino = os.path.join(trabajo, f"build-{sub}")
            borrar(destino)
            exe, seg = fn(os.path.join(poc, sub), destino)
            hashes.append(sha256_fichero(exe))
            reloj(f"build en limpio {nombre} ({i}.a): {seg:.1f} s")
            if i == 1:
                guardado = os.path.join(trabajo, f"tablero-{sub}.exe")
                shutil.copyfile(exe, guardado)
        igual = "igual" if hashes[0] == hashes[1] else "distinto"
        print(f"{nombre}: dos builds en limpio: sha256 {igual} ; tamano {os.path.getsize(guardado)} bytes")
        medir(f"B: {nombre}, dos builds en limpio (win-x64): sha256", igual, "B")
        exes[nombre] = guardado
    # El enlazador de MSVC pone una marca de tiempo en la cabecera PE y en el directorio de depuracion; /Brepro la
    # sustituye por un hash del contenido. Dos builds mas con esa opcion (no cambian el .exe que se mide).
    hashes = []
    for i in (1, 2):
        destino = os.path.join(trabajo, "build-poc-rust-brepro")
        borrar(destino)
        exe, seg = build_rust(os.path.join(poc, "poc-rust"), destino, rustflags="-C link-arg=/Brepro")
        hashes.append(sha256_fichero(exe))
        reloj(f"build en limpio Rust con /Brepro ({i}.a): {seg:.1f} s")
        if i == 1:
            # El .exe de Rust que se mide es este: el build por defecto cambia de bytes entre builds (y con el,
            # p. ej., su tamano comprimido), y /Brepro es el perfil que decide el ADR.
            shutil.copyfile(exe, exes["Rust"])
    igual = "igual" if hashes[0] == hashes[1] else "distinto"
    print(f"Rust con -C link-arg=/Brepro: dos builds en limpio: sha256 {igual} ; tamano {os.path.getsize(exes['Rust'])} "
          f"bytes (es el .exe de Rust que se mide desde aqui)")
    medir("B: Rust con -C link-arg=/Brepro, dos builds en limpio (win-x64): sha256", igual, "B")
    if igual != "igual":
        falla("B", "el build de Rust con /Brepro no es reproducible: las cifras de M no serian deterministas")
    borrar(os.path.join(trabajo, "build-poc-rust-brepro"))
    # El repo no cambia por compilar: las fuentes no se tocan.
    sucio = git(repo, "status", "--porcelain", "--", os.path.join("tareas", "02-en-curso", "TASK-014", "poc"))
    ajenos = [l for l in sucio.splitlines() if re.search(r"target/|node_modules/|dist/|\.exe", l)]
    print(f"artefactos de build dentro del repo: {len(ajenos)}")
    medir("B: artefactos de build dentro del repo", len(ajenos), "B")
    return exes


# --------------------------------------------------------------------------- C

def seccion_c(personal, trabajo):
    sep("C: copias de ToolsaWeb y OpenDockTrading en commit fijo (los originales solo se leen)")
    copias = {}
    for nombre, fijo in (("ToolsaWeb", FIJO_TW), ("OpenDockTrading", FIJO_ODT)):
        destino = os.path.join(trabajo, f"copia-{nombre}")
        run(["git", "-c", "core.longpaths=true", "clone", "-q", "--no-hardlinks", os.path.join(personal, nombre), destino],
            check=True)
        git(destino, "-c", "advice.detachedHead=false", "checkout", "-q", "--detach", fijo)
        head = git(destino, "rev-parse", "HEAD").strip()
        h, n = hash_arbol(destino)
        print(f"{nombre}: commit {head[:9]} ; ficheros (con .git) {n}")
        if head != fijo:
            falla("C", f"{nombre}: HEAD {head} no es el commit fijo")
        medir(f"C: {nombre}: commit fijo", head[:9], "C")
        copias[nombre] = (destino, h)
    vacio = os.path.join(trabajo, "repo-vacio")
    os.makedirs(vacio)
    run(["git", "init", "-q", "-b", "main", vacio], check=True)
    git(vacio, "-c", "user.name=poc", "-c", "user.email=poc@poc.invalid", "commit", "-q", "--allow-empty", "-m", "vacio")
    return copias, vacio


def seccion_c_final(copias):
    sep("C': hash del arbol de las copias despues de todas las ejecuciones")
    for nombre, (destino, antes) in copias.items():
        despues, _ = hash_arbol(destino)
        igual = "igual" if antes == despues else "distinto"
        print(f"{nombre}: hash del arbol (con .git) antes y despues: {igual}")
        medir(f"C: {nombre}: hash del arbol (con .git) antes / despues", igual, "C")
        if igual != "igual":
            falla("C", f"{nombre}: la copia cambio")


# --------------------------------------------------------------------------- S

class ClaveDuplicada(Exception):
    pass


class CargadorEstricto(yaml.SafeLoader):
    pass


def _mapa_estricto(loader, node, deep=False):
    vistas = set()
    for k, _ in node.value:
        clave = loader.construct_object(k, deep=True)
        if clave in vistas:
            raise ClaveDuplicada(clave)
        vistas.add(clave)
    return loader.construct_mapping(node, deep=deep)


CargadorEstricto.add_constructor(yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, _mapa_estricto)


def frontmatter(texto):
    lineas = [l[:-1] if l.endswith("\r") else l for l in texto.lstrip("\ufeff").split("\n")]
    if not lineas or lineas[0] != "---" or "---" not in lineas[1:]:
        return None
    return "\n".join(lineas[1:lineas.index("---", 1)])


def lector_pyyaml(raiz):
    """Recuento independiente con PyYAML y cargador que rechaza duplicadas (contraste, como en F1-9)."""
    validas, errores, estados = 0, {}, {}
    base = os.path.join(raiz, "tareas")
    for d, subs, fs in os.walk(base):
        for f in fs:
            if f != "tarea.md":
                continue
            rel = os.path.relpath(os.path.join(d, f), raiz).replace("\\", "/")
            fm = frontmatter(open(os.path.join(d, f), encoding="utf-8", errors="replace").read())
            if fm is None:
                errores[rel] = "sin frontmatter"
                continue
            try:
                datos = yaml.load(fm, Loader=CargadorEstricto)
            except ClaveDuplicada:
                errores[rel] = "clave duplicada"
                continue
            except yaml.YAMLError:
                errores[rel] = "yaml invalido"
                continue
            escalar = lambda v: v is not None and not isinstance(v, (list, dict))
            if not isinstance(datos, dict) or not escalar(datos.get("id")) or not escalar(datos.get("titulo")) \
                    or isinstance(datos.get("estado"), (list, dict)):
                errores[rel] = "yaml invalido"
                continue
            validas += 1
            e = datos.get("estado")
            e = "desconocido" if e is None else str(e)
            estados[e] = estados.get(e, 0) + 1
    return validas, errores, estados


def parsear_salida(texto):
    lineas = texto.splitlines()
    out = {"tareas": int(lineas[0].split(": ")[1]), "errores": int(lineas[1].split(": ")[1]), "estados": {},
           "lista_errores": {}}
    for l in lineas[2:]:
        if l.startswith("git: ") or l.startswith("red: "):
            continue
        if l.startswith("error "):
            rel, motivo = l[6:].rsplit(": ", 1)
            out["lista_errores"][rel] = motivo
        else:
            e, n = l.rsplit(": ", 1)
            out["estados"][e] = int(n)
    return out


def ejecutar(exe, raiz, *extra, env=None):
    r = run([exe, raiz, *extra], env=env, timeout=120)
    return r.returncode, r.stdout


def entorno_sin_node():
    env = {k: v for k, v in os.environ.items() if not k.upper().startswith("NODE_")}
    partes = [p for p in env.get("PATH", "").split(os.pathsep)
              if p and not os.path.exists(os.path.join(p.strip('"'), "node.exe"))]
    env["PATH"] = os.pathsep.join(partes)
    return env


def seccion_s(exes, copias, vacio, trabajo):
    sep("S: salidas sobre las copias, contraste con PyYAML, clave duplicada y sin node en el PATH")
    rust, ts = exes["Rust"], exes["TypeScript"]
    salidas = {}
    for nombre, raiz in [(n, c[0]) for n, c in copias.items()] + [("repo vacio", vacio)]:
        cr, sr = ejecutar(rust, raiz, "--sin-red")
        ct, st = ejecutar(ts, raiz, "--sin-red")
        igual = "si" if (cr, sr) == (ct, st) else "no"
        p = parsear_salida(sr.decode("utf-8"))
        salidas[nombre] = sr
        print(f"{nombre}: codigos {cr} / {ct} ; stdout identico byte a byte: {igual} ; tareas {p['tareas']} ; "
              f"errores {p['errores']} ; estados {', '.join(f'{k} {v}' for k, v in p['estados'].items()) or '-'}")
        medir(f"S: {nombre}: salida Rust y TypeScript identica (stdout y codigo)", igual, "S")
        if igual != "si":
            falla("S", f"{nombre}: salidas distintas")
        if nombre == "repo vacio":
            medir("S: repo vacio (sin tareas/): tareas / errores", f"{p['tareas']} / {p['errores']}", "S")
            continue
        v, errs, est = lector_pyyaml(raiz)
        coincide = "si" if (v, errs, est) == (p["tareas"], p["lista_errores"], p["estados"]) else "no"
        print(f"   PyYAML (cargador estricto): validas {v} ; errores {len(errs)} ; coincide con los binarios: {coincide}")
        medir(f"S: {nombre}: tareas validas / errores", f"{p['tareas']} / {p['errores']}", "S")
        medir(f"S: {nombre}: estados", ", ".join(f"{k} {n}" for k, n in p["estados"].items()), "S")
        medir(f"S: {nombre}: recuento igual al de PyYAML", coincide, "S")
        if coincide != "si" or p["tareas"] + p["errores"] != ESPERADO[nombre]:
            falla("S", f"{nombre}: recuento {p['tareas']}+{p['errores']} frente a PyYAML {v}+{len(errs)} y F1-9 {ESPERADO[nombre]}")

    # Clave duplicada en una tarea real copiada de OpenDockTrading (la primera en orden de ruta).
    odt = copias["OpenDockTrading"][0]
    rutas = sorted(os.path.relpath(os.path.join(d, f), odt).replace("\\", "/")
                   for d, _, fs in os.walk(os.path.join(odt, "tareas")) for f in fs if f == "tarea.md")
    origen = rutas[0]
    texto = open(os.path.join(odt, origen), encoding="utf-8").read()
    m = re.search(r"^estado:.*$", texto, re.M)
    mutado = texto[:m.end()] + "\n" + m.group(0) + texto[m.end():]
    dup = os.path.join(trabajo, "repo-duplicada")
    for sub, contenido in (("original", texto), ("duplicada", mutado)):
        os.makedirs(os.path.join(dup, "tareas", sub))
        open(os.path.join(dup, "tareas", sub, "tarea.md"), "w", encoding="utf-8", newline="").write(contenido)
    run(["git", "init", "-q", "-b", "main", dup], check=True)
    git(dup, "add", "-A")
    git(dup, "-c", "user.name=poc", "-c", "user.email=poc@poc.invalid", "commit", "-q", "-m", "dup")
    res = []
    for exe in (rust, ts):
        c, s = ejecutar(exe, dup, "--sin-red")
        res.append("tareas: 1\nerrores: 1\n" in s.decode() and "error tareas/duplicada/tarea.md: clave duplicada" in s.decode()
                   and c == 0)
    try:
        yaml.safe_load(frontmatter(mutado))
        pyyaml_acepta = "si"
    except yaml.YAMLError:
        pyyaml_acepta = "no"
    print(f"clave duplicada (estado repetido en {origen} de OpenDockTrading): Rust la rechaza {'si' if res[0] else 'no'} ; "
          f"TypeScript la rechaza {'si' if res[1] else 'no'} ; PyYAML safe_load la acepta en silencio: {pyyaml_acepta}")
    medir("S: clave duplicada en una tarea real: rechazada por Rust / TypeScript",
          f"{'si' if res[0] else 'no'} / {'si' if res[1] else 'no'}", "S")
    medir("S: clave duplicada: PyYAML safe_load la acepta en silencio", pyyaml_acepta, "S")
    if not all(res):
        falla("S", "alguna implementacion no rechaza la clave duplicada")

    # Sin node en el PATH ni variables NODE_*.
    env = entorno_sin_node()
    sin_node = shutil.which("node", path=env["PATH"]) is None
    res = []
    for exe in (rust, ts):
        c, s = ejecutar(exe, odt, "--sin-red", env=env)
        res.append(c == 0 and s == salidas["OpenDockTrading"])
    print(f"sin node en el PATH (where node vacio: {'si' if sin_node else 'no'}) ni NODE_*: Rust igual {'si' if res[0] else 'no'} ; "
          f"TypeScript igual {'si' if res[1] else 'no'}")
    medir("S: sin node en el PATH: Rust / TypeScript dan la misma salida", f"{'si' if res[0] else 'no'} / {'si' if res[1] else 'no'}", "S")
    if not (sin_node and all(res)):
        falla("S", "sin node en el PATH algo cambia")
    return env


# --------------------------------------------------------------------------- F

def huellas_almacen():
    ps = ("(Get-ChildItem Cert:\\CurrentUser\\My, Cert:\\CurrentUser\\Root, Cert:\\CurrentUser\\TrustedPublisher, "
          "Cert:\\CurrentUser\\CA | Select-Object -ExpandProperty Thumbprint | Sort-Object) -join ','")
    r = run(["powershell", "-NoProfile", "-NonInteractive", "-Command", ps], timeout=120)
    return hashlib.sha256(r.stdout.strip()).hexdigest(), r.returncode


def seccion_f(exes, copias, trabajo, inv):
    sep("F: firma Authenticode con certificado autofirmado desechable (copias de los .exe)")
    if not inv["signtool"]:
        print("signtool no encontrado: firma no verificada")
        medir("F: firma", "no verificado (sin signtool)", "F")
        return
    firma = os.path.join(trabajo, "firma")
    os.makedirs(firma)
    antes = huellas_almacen()
    cnf = os.path.join(firma, "min.cnf")
    open(cnf, "w").write("[req]\ndistinguished_name=dn\n[dn]\n")
    env = dict(os.environ, OPENSSL_CONF=cnf)
    openssl = r"C:\Program Files\Git\usr\bin\openssl.exe"
    clave, cert, pfx = (os.path.join(firma, n) for n in ("clave.pem", "cert.pem", "desechable.pfx"))
    contrasena = secrets.token_hex(16)    # solo para este PFX desechable; no se imprime
    run([openssl, "req", "-x509", "-newkey", "rsa:3072", "-sha256", "-days", "2", "-nodes", "-keyout", clave,
         "-out", cert, "-subj", f"/CN={CN_PRUEBA}", "-addext", "extendedKeyUsage=codeSigning",
         "-addext", "keyUsage=critical,digitalSignature"], env=env, check=True)
    run([openssl, "pkcs12", "-export", "-inkey", clave, "-in", cert, "-out", pfx, "-passout", f"pass:{contrasena}"],
        env=env, check=True)
    odt = copias["OpenDockTrading"][0]
    for nombre, exe in exes.items():
        copia = os.path.join(firma, f"firmado-{nombre}.exe")
        shutil.copyfile(exe, copia)
        v0 = run([SIGNTOOL, "verify", "/pa", copia])
        s = run([SIGNTOOL, "sign", "/fd", "SHA256", "/f", pfx, "/p", contrasena, copia])
        c0, s0 = ejecutar(exe, odt, "--sin-red")
        c1, s1 = ejecutar(copia, odt, "--sin-red")
        v = run([SIGNTOOL, "verify", "/pa", "/v", copia])
        salida_v = (v.stdout + v.stderr).decode("utf-8", "replace")
        ve_cn = CN_PRUEBA in salida_v
        no_confianza = "not trusted" in salida_v or "no es de confianza" in salida_v
        estado_v = ("rechazada: raiz no de confianza" if v.returncode != 0 and ve_cn and no_confianza
                    else f"inesperado (codigo {v.returncode})")
        reloj(f"firma {nombre}: el .exe crece {os.path.getsize(copia) - os.path.getsize(exe)} bytes")
        print(f"{nombre}: sin firma antes: verify {'falla' if v0.returncode else 'pasa'} ; sign codigo {s.returncode} ; "
              f"ejecucion tras firmar igual: {'si' if (c0, s0) == (c1, s1) else 'no'} ; verify /pa: {estado_v} ; "
              f"firmante visto: {'si' if ve_cn else 'no'}")
        medir(f"F: {nombre}: signtool sign con el PFX desechable", "codigo 0" if s.returncode == 0 else f"codigo {s.returncode}", "F")
        medir(f"F: {nombre}: se ejecuta igual tras firmar", "si" if (c0, s0) == (c1, s1) else "no", "F")
        medir(f"F: {nombre}: signtool verify /pa", estado_v, "F")
        if s.returncode != 0 or (c0, s0) != (c1, s1) or not estado_v.startswith("rechazada"):
            falla("F", f"{nombre}: firma o ejecucion inesperada")
    despues = huellas_almacen()
    igual = "si" if antes == despues and antes[1] == 0 else "no"
    print(f"almacen de certificados del usuario (My, Root, CA, TrustedPublisher) sin cambios: {igual}")
    medir("F: almacen de certificados del usuario sin cambios", igual, "F")
    if igual != "si":
        falla("F", "el almacen de certificados cambio")


# --------------------------------------------------------------------------- X

def seccion_x(repo, trabajo, inv):
    sep("X: compilacion cruzada y ejecucion fuera de Windows, solo con lo instalado")
    fuentes = os.path.join(repo, "tareas", "02-en-curso", "TASK-014", "poc", "poc-rust")
    env = dict(os.environ, CARGO_TARGET_DIR=os.path.join(trabajo, "build-cruzada"))
    for t in CRUZADOS_RUST:
        # Sin --offline: las crates propias de Linux y macOS (p. ej. openssl-probe) no estan en la cache y se bajan
        # de crates.io como cualquier dependencia; el target (la std de destino) no se instala.
        r = run(["cargo", "build", "--release", "--locked", "--target", t], cwd=fuentes, env=env)
        err = r.stderr.decode("utf-8", "replace")
        if r.returncode == 0:
            res = "compila"
        elif "may not be installed" in err or "can't find crate" in err:
            res = "no compila: target no instalado (sin la std de destino)"
        else:
            res = f"no compila: codigo {r.returncode}"
        print(f"Rust --target {t}: {res}")
        medir(f"X: Rust --target {t}", res, "X")
    print("TypeScript (Node SEA) para Linux o macOS: hace falta el node de ese sistema como base; no hay ninguno "
          "instalado y no se descarga")
    print(f"bun build --compile --target: bun {'encontrado' if shutil.which('bun') else 'no encontrado'}")
    medir("X: TypeScript para Linux o macOS (Node SEA o bun)", "no verificado: falta el node de destino y bun", "X")
    linux = ("no verificado: daemon de Docker sin respuesta y WSL sin distribucion de usuario"
             if inv["docker"] != "responde" and all(d.lower().startswith("docker-desktop") for d in inv["distros"])
             else "posible (no ejecutado)")
    print(f"ejecucion en Linux: {linux}")
    print("ejecucion en macOS: no verificado (no hay macOS)")
    medir("X: ejecucion en Linux", linux, "X")
    medir("X: ejecucion en macOS", "no verificado: no hay macOS", "X")
    cruzada = {"Rust": all(MEDIDO[f"X: Rust --target {t}"][0] == "compila" for t in CRUZADOS_RUST), "TypeScript": False}
    return cruzada


# --------------------------------------------------------------------------- M y D

def seccion_m(exes):
    sep("M: tamanos y peso de meter binarios en el repo del marketplace")
    tam = {}
    for nombre, exe in exes.items():
        datos = open(exe, "rb").read()
        gz = len(gzip.compress(datos, compresslevel=9, mtime=0))
        tam[nombre] = (len(datos), gz)
        print(f"{nombre}: .exe win-x64 {len(datos)} bytes ({mib(len(datos))} MiB) ; gzip -9 {gz} bytes ({mib(gz)} MiB)")
        medir(f"M: {nombre}: tamano del .exe win-x64 (bytes)", len(datos), "M")
        medir(f"M: {nombre}: .exe win-x64 comprimido con gzip -9 (bytes)", gz, "M")
        por_version = len(datos) * len(TARGETS_P12)
        medir(f"M: {nombre}: por version con {len(TARGETS_P12)} targets de P12, estimado con win-x64 (MiB)", mib(por_version), "M")
        medir(f"M: {nombre}: 10 versiones en el repo del marketplace, estimado (MiB)", mib(10 * por_version), "M")
        aviso = "por encima" if len(datos) > 50 * MIB else "por debajo"
        bloqueo = "por encima" if len(datos) > 100 * MIB else "por debajo"
        medir(f"M: {nombre}: .exe frente al aviso de 50 MiB / bloqueo de 100 MiB de GitHub", f"{aviso} / {bloqueo}", "M")
        print(f"   por version ({len(TARGETS_P12)} targets, estimado con win-x64): {mib(por_version)} MiB ; 10 versiones: "
              f"{mib(10 * por_version)} MiB ; frente a 50 / 100 MiB de GitHub: {aviso} / {bloqueo}")
    razon = tam["TypeScript"][0] / tam["Rust"][0]
    print(f"TypeScript / Rust en tamano: {razon:.1f} veces")
    medir("M: tamano TypeScript / Rust (veces)", f"{razon:.1f}".replace(".", ","), "M")
    medir("M: targets de P12", ", ".join(TARGETS_P12), "M")
    return tam


def lineas(p):
    ls = open(p, encoding="utf-8").read().splitlines()
    return len(ls), sum(1 for l in ls if l.strip() and not l.strip().startswith("//"))


def seccion_d(repo):
    sep("D: dependencias y lineas de codigo (de las fuentes del repo)")
    poc = os.path.join(repo, "tareas", "02-en-curso", "TASK-014", "poc")
    lock = open(os.path.join(poc, "poc-rust", "Cargo.lock"), encoding="utf-8").read()
    toml = open(os.path.join(poc, "poc-rust", "Cargo.toml"), encoding="utf-8").read()
    deps = re.search(r"^\[dependencies\]\s*$(.*?)(?=^\[|\Z)", toml, re.S | re.M).group(1)
    directas_rs = sorted(re.findall(r"^([A-Za-z0-9_-]+)\s*=", deps, re.M))
    paquetes_rs = len(re.findall(r"^\[\[package\]\]", lock, re.M))
    pkg = json.load(open(os.path.join(poc, "poc-ts", "package.json"), encoding="utf-8"))
    plock = json.load(open(os.path.join(poc, "poc-ts", "package-lock.json"), encoding="utf-8"))
    paquetes_ts = len([k for k in plock["packages"] if k])
    runtime_ts = sorted(pkg.get("dependencies", {}))
    postject = plock["packages"].get("node_modules/postject", {}).get("version", "-")
    l_rs, n_rs = lineas(os.path.join(poc, "poc-rust", "src", "main.rs"))
    l_ts, n_ts = lineas(os.path.join(poc, "poc-ts", "src", "main.ts"))
    l_b, n_b = lineas(os.path.join(poc, "poc-ts", "scripts", "build.mjs"))
    print(f"Rust: dependencias directas {len(directas_rs)} ({', '.join(directas_rs)}) ; paquetes en Cargo.lock {paquetes_rs}")
    print(f"TypeScript: dependencias de runtime {len(runtime_ts)} ({', '.join(runtime_ts)}) ; paquetes en package-lock.json "
          f"{paquetes_ts} ; postject {postject}")
    print(f"lineas de main.rs {l_rs} ({n_rs} sin blancos ni comentarios de linea) ; main.ts {l_ts} ({n_ts}) ; "
          f"build.mjs {l_b} ({n_b})")
    medir("D: Rust: dependencias directas", f"{len(directas_rs)} ({', '.join(directas_rs)})", "D")
    medir("D: Rust: paquetes en Cargo.lock", paquetes_rs, "D")
    medir("D: TypeScript: dependencias de runtime", f"{len(runtime_ts)} ({', '.join(runtime_ts)})", "D")
    medir("D: TypeScript: paquetes en package-lock.json", paquetes_ts, "D")
    medir("D: TypeScript: version de postject", postject, "D")
    medir("D: lineas de main.rs / main.ts (sin blancos ni comentarios de linea)", f"{n_rs} / {n_ts}", "D")
    medir("D: lineas de build.mjs (sin blancos ni comentarios de linea)", n_b, "D")


# --------------------------------------------------------------------------- T

def tiempo_de(exe, raiz):
    t = time.perf_counter()
    r = run([exe, raiz, "--sin-red"], timeout=120)
    if r.returncode != 0:
        raise SystemExit(f"ERROR: {exe} salio con {r.returncode} en {raiz}")
    return (time.perf_counter() - t) * 1000


def seccion_t(exes, copias, vacio, trabajo, env_sin_node):
    repos = [("repo vacio", vacio)] + [(n, c[0]) for n, c in copias.items()]
    frio_dir = os.path.join(trabajo, "frio")
    os.makedirs(frio_dir)
    caliente = {}
    for nombre, exe in exes.items():
        for rn, raiz in repos:
            copia = os.path.join(frio_dir, f"{nombre}-{rn.replace(' ', '-')}-{secrets.token_hex(4)}.exe")
            shutil.copyfile(exe, copia)
            frio = tiempo_de(copia, raiz)
            tiempo_de(exe, raiz)          # calentamiento
            xs = [tiempo_de(exe, raiz) for _ in range(N_CALIENTE)]
            caliente[(nombre, rn)] = mediana(xs)
            reloj(f"{nombre} sobre {rn}: frio (1.a ejecucion de copia nueva) {frio:.0f} ms ; caliente mediana de "
                  f"{N_CALIENTE} {mediana(xs):.0f} ms (min {min(xs):.0f}, max {max(xs):.0f})")
            caliente[(nombre, rn, "frio")] = frio
    rapido = all(caliente[("Rust", rn)] < caliente[("TypeScript", rn)] for rn, _ in repos)
    rapido_frio = all(caliente[("Rust", rn, "frio")] < caliente[("TypeScript", rn, "frio")] for rn, _ in repos)
    medir("T: Rust mas rapido en caliente (mediana de 20) en los 3 repos", "si" if rapido else "no", "T")
    medir("T: Rust mas rapido en frio en los 3 repos", "si" if rapido_frio else "no", "T")
    # Red: depende de la conexion; solo informativo, fuera de la tabla.
    for nombre, exe in exes.items():
        c, s = ejecutar(exe, vacio, env=env_sin_node)
        red = next((l for l in s.decode().splitlines() if l.startswith("red: ")), "red: ?")
        reloj(f"{nombre} con red y sin variables NODE_* (almacen del sistema): {red}")
    return rapido


# --------------------------------------------------------------------------- D1

def regla_d1(rapido, tam, cruzada):
    sep("D1: regla declarada antes de medir (plan, D1a)")
    crit = [
        ("1 arranque en caliente", "Rust" if rapido else "ninguno"),
        ("2 tamano", "Rust" if tam["Rust"][0] < tam["TypeScript"][0] else "TypeScript"),
        ("3 compilacion cruzada reproducible", "Rust" if cruzada["Rust"] and not cruzada["TypeScript"] else
         "TypeScript" if cruzada["TypeScript"] and not cruzada["Rust"] else "ninguno"),
        ("4 librerias mantenidas que cumplan F1-9", CRITERIO_4[0]),
        ("5 coste del agente", CRITERIO_5[0]),
    ]
    for c, g in crit:
        print(f"criterio {c}: {g}")
        medir(f"D1: criterio {c}", g, "D1")
    print(f"   motivo del 4: {CRITERIO_4[1]}")
    print(f"   motivo del 5: {CRITERIO_5[1]}")
    ganador = "ninguno"
    for lenguaje in ("Rust", "TypeScript"):
        otros = sum(1 for c, g in crit[1:] if g == lenguaje)
        if crit[0][1] == lenguaje and otros >= 2:
            ganador = lenguaje
    print(f"gana: {ganador} (el 1.o y al menos dos mas)")
    medir("D1: ganador (el 1.o y al menos dos mas)", ganador, "D1")
    sin4 = sum(1 for c, g in crit[1:3] if g == crit[0][1])
    medir("D1: criterios ganados por el ganador del 1.o sin contar el 4 ni el 5", sin4, "D1")


# --------------------------------------------------------------------------- G

SECCIONES = ["## Contexto", "## Alternativas", "## Pruebas de concepto", "## Cómo se ve en GitHub", "## Decisión",
             "## Divergencias con el plan", "## Requisitos y evidencia", "## Consecuencias", "## Fuera de alcance",
             "## No verificado"]
FRASES_OBLIGATORIAS = ["engram no es obligatorio", "almacén del sistema", "`cli.lock`", "`tc2 version --json`",
                       "`tc2 doctor --recursos`"]


def entre_marcadores(texto, nombre):
    m = re.search(rf"<!-- {nombre}:inicio -->(.*?)<!-- {nombre}:fin -->", texto, re.S)
    return m.group(1) if m else None


def filas_tabla(bloque):
    filas = []
    for l in bloque.strip().splitlines():
        l = l.strip()
        if not l.startswith("|") or re.match(r"^\|[-| ]+\|$", l):
            continue
        filas.append([c.strip() for c in l.strip("|").split("|")])
    return filas[1:]


def errores_adr(adr, ruta, repo, medido, con_indice=True):
    """Lista de errores del ADR frente a lo medido (funcion pura: la usan G y sus mutaciones)."""
    err = []
    bloque = entre_marcadores(adr, "cifras")
    if bloque is None:
        return ["el ADR no tiene el bloque cifras"], {}
    tabla = {}
    for f in filas_tabla(bloque):
        if f[0] in tabla:
            err.append(f"fila repetida en la tabla: {f[0]}")
        tabla[f[0]] = (f[1].strip("`"), f[2] if len(f) > 2 else "")
    for k, (v, s) in sorted(medido.items()):
        if k not in tabla:
            err.append(f"cifra medida que falta en la tabla: {k} = {v}")
        elif tabla[k][0] != v:
            err.append(f"{k}: el ADR dice {tabla[k][0]!r} y lo medido es {v!r}")
        elif tabla[k][1] != s:
            err.append(f"{k}: fuente {tabla[k][1]!r} y es {s!r}")
    for k in tabla:
        if k not in medido:
            err.append(f"fila de la tabla sin medida: {k}")
    res = entre_marcadores(adr, "resumen")
    if res is None:
        err.append("el ADR no tiene el bloque resumen")
    else:
        valores = {v for v, _ in tabla.values()}
        valores |= {p for v in valores for p in re.split(r"\s*(?:/|,|\(|\)|:| y | de )\s*", v) if p}
        for b in re.findall(r"\*\*([^*]+)\*\*", res):
            if re.search(r"\d", b) and b not in valores:
                err.append(f"el resumen cita {b!r}, que no es un valor de la tabla")
    faltan = [s for s in SECCIONES if not re.search(rf"^{re.escape(s)}\s*$", adr, re.M)]
    if faltan:
        err.append(f"faltan secciones {faltan}")
    if not re.search(r"^- Estado: \*\*aceptada\*\*", adr, re.M):
        err.append("el estado no es aceptada")
    plano = re.sub(r"\s+", " ", adr)
    for frase in FRASES_OBLIGATORIAS:
        if frase not in plano:
            err.append(f"falta la regla: {frase}")
    nv = re.search(r"^## No verificado\s*$(.*?)(?=^## |\Z)", adr, re.S | re.M)
    if not nv or "Linux" not in nv.group(1) or "macOS" not in nv.group(1):
        err.append("«No verificado» no nombra Linux y macOS")
    ganador = tabla.get("D1: ganador (el 1.o y al menos dos mas)", ("?", ""))[0]
    dec = re.search(r"^## Decisión\s*$(.*?)(?=^## )", adr, re.S | re.M)
    if not dec or f"**{ganador}**" not in dec.group(1):
        err.append(f"la decision no nombra en negrita al ganador medido de D1 ({ganador})")
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
    prohibidos = [e for e in enlaces if "tareas/" in e or "OpenDockTrading" in e or "ToolsaWeb" in e or e.startswith("../../")]
    if prohibidos:
        err.append(f"enlaces prohibidos {prohibidos}")
    for e in enlaces:
        if not e.startswith("http") and not os.path.exists(os.path.join(os.path.dirname(ruta), e.split("#")[0])):
            err.append(f"enlace que no resuelve: {e}")
    if con_indice:
        idx = open(os.path.join(repo, "docs", "adr", "README.md"), encoding="utf-8").read()
        if FILA_INDICE not in idx:
            err.append("falta la fila en docs/adr/README.md")
    return err, tabla


def mutaciones(adr):
    clave = "M: Rust: tamano del .exe win-x64 (bytes)"
    m = re.search(rf"^\| {re.escape(clave)} \| `([^`]+)` \| M \|$", adr, re.M)
    fila = m.group(0) if m else "(sin fila)"
    g = re.search(r"^\| D1: ganador \(el 1\.o y al menos dos mas\) \| `([^`]+)` \| D1 \|$", adr, re.M)
    return [
        ("cifra-cambiada", adr.replace(fila, fila.replace(m.group(1), m.group(1) + "0")) if m else adr, f"{clave}: el ADR dice"),
        ("fila-borrada", adr.replace(fila + "\n", ""), f"cifra medida que falta en la tabla: {clave}"),
        ("fila-sin-medida", adr.replace("<!-- cifras:fin -->", "| cifra inventada | `1` | M |\n<!-- cifras:fin -->"),
         "fila de la tabla sin medida: cifra inventada"),
        ("tiempo-de-reloj-en-la-tabla", adr.replace("<!-- cifras:fin -->",
                                                     "| T: Rust caliente en ms | `9` | T |\n<!-- cifras:fin -->"),
         "fila de la tabla sin medida: T: Rust caliente en ms"),
        ("fuente-cambiada", adr.replace(fila, fila[:-3] + "D |") if m else adr, f"{clave}: fuente"),
        ("resumen-inventado", adr.replace("<!-- resumen:fin -->", "Cifra suelta: **9999**.\n<!-- resumen:fin -->"),
         "el resumen cita '9999'"),
        ("id-inexistente", adr.replace("## Contexto", "## Contexto\n\nVer F7-99.", 1), "IDs que no existen: ['F7-99']"),
        ("enlace-a-tareas", adr.replace("## Contexto", "## Contexto\n\n[x](../../tareas/x.md)", 1), "enlaces prohibidos"),
        ("enlace-roto", adr.replace("## Contexto", "## Contexto\n\n[x](F1-99-no-existe.md)", 1),
         "enlace que no resuelve: F1-99-no-existe.md"),
        ("seccion-borrada", adr.replace("## No verificado", "## Sin verificar"), "faltan secciones"),
        ("estado-propuesta", adr.replace("- Estado: **aceptada**", "- Estado: **propuesta**"), "el estado no es aceptada"),
        ("engram-obligatorio", adr.replace("engram no es obligatorio", "engram es obligatorio"),
         "falta la regla: engram no es obligatorio"),
        ("ganador-cambiado", adr.replace(g.group(0), g.group(0).replace(f"`{g.group(1)}`", "`TypeScript`")) if g else adr,
         "D1: ganador (el 1.o y al menos dos mas): el ADR dice"),
        ("sin-macos-en-no-verificado", re.sub(r"(^## No verificado\s*$)(.*?)(?=^## |\Z)",
                                              lambda x: x.group(1) + x.group(2).replace("macOS", "Mac"), adr,
                                              flags=re.S | re.M), "«No verificado» no nombra Linux y macOS"),
    ]


def seccion_g(repo, medido):
    sep("G: el ADR (cifras, resumen, secciones, IDs, enlaces e indice) y sus mutaciones de control")
    ruta = os.path.join(repo, ADR_REL)
    if not os.path.exists(ruta):
        falla("G", "no existe el ADR")
        return
    adr = open(ruta, encoding="utf-8").read()
    err, tabla = errores_adr(adr, ruta, repo, medido)
    print(f"cifras en la tabla del ADR: {len(tabla)} ; medidas: {len(medido)}")
    for e in err:
        falla("G", e)
    print(f"errores del ADR: {len(err)}")
    for nombre, mutado, esperado in mutaciones(adr):
        if mutado == adr:
            falla("G", f"la mutacion {nombre} no cambia el ADR")
            continue
        errs, _ = errores_adr(mutado, ruta, repo, medido)
        ok = any(esperado in e for e in errs)
        print(f"  mutacion {nombre}: {'falla como se espera' if ok else 'NO FALLA'} "
              f"({str(next((e for e in errs if esperado in e), errs[:1]))[:110]})")
        if not ok:
            falla("G", f"la mutacion {nombre} no la detecta el guion")


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
    if args[:1] == ["--solo-g"] and len(args) == 3:
        medido = {k: tuple(v) for k, v in json.load(open(args[1], encoding="utf-8")).items()}
        seccion_g(os.path.abspath(args[2]), medido)
        resultado()
        return
    guardar = None
    if "--guardar-medido" in args:
        i = args.index("--guardar-medido")
        guardar = os.path.abspath(args[i + 1])
        del args[i:i + 2]
    if len(args) != 3:
        raise SystemExit(__doc__)
    trabajo, repo, personal = (os.path.abspath(a) for a in args)
    comprobar_trabajo(trabajo)
    borrar(trabajo)
    os.makedirs(trabajo)
    open(os.path.join(trabajo, MARCA), "w", encoding="utf-8").write("carpeta de trabajo de poc-f1-14.py\n")
    print(f"PoC F1-14 ; ToolsaWeb {FIJO_TW[:9]} ; OpenDockTrading {FIJO_ODT[:9]} ; Python {sys.version.split()[0]} ; "
          f"PyYAML {yaml.__version__}")
    try:
        inv = seccion_i()
        exes = seccion_b(repo, trabajo)
        copias, vacio = seccion_c(personal, trabajo)
        env_sin_node = seccion_s(exes, copias, vacio, trabajo)
        seccion_f(exes, copias, trabajo, inv)
        cruzada = seccion_x(repo, trabajo, inv)
        tam = seccion_m(exes)
        seccion_d(repo)
        rapido = seccion_t(exes, copias, vacio, trabajo, env_sin_node)
        seccion_c_final(copias)
        regla_d1(rapido, tam, cruzada)
        if guardar:
            json.dump(MEDIDO, open(guardar, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
        seccion_g(repo, MEDIDO)
        sep("T: tiempos de reloj (NO deterministas; fuera de la tabla comprobada del ADR)")
        for t in TIEMPOS:
            print(t)
    finally:
        borrar(trabajo)     # tambien si algo falla
    print()
    print(f"carpeta de trabajo borrada al terminar: {'no' if os.path.lexists(largo(trabajo)) else 'si'}")
    if os.path.lexists(largo(trabajo)):
        falla("main", "la carpeta de trabajo sigue existiendo")
    resultado()


if __name__ == "__main__":
    main()

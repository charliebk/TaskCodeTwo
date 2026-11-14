// PoC de F1-14 (TASK-014): comando `tablero` segun especificacion-comando.md.
// Uso: tablero <raiz-del-repo> [--sin-red]
import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { getCACertificates, setDefaultCACertificates } from "node:tls";
import { type Document, isAlias, isMap, isScalar, parseDocument } from "yaml";

const URL_RED = "https://api.github.com/zen";
const USER_AGENT = "taskcodetwo-poc";
// Alineado con la PoC Rust (poc-rust/NOTAS.md, interpretacion 6): 10 s.
const TIMEOUT_RED_MS = 10_000;

/** Error de E/S propio del programa: termina con codigo 1 (como la PoC Rust). */
class ErrorPrograma extends Error {}

interface Tarea {
  id: string;
  titulo: string;
  estado: string;
}

type Resultado =
  | { ok: true; tarea: Tarea }
  | { ok: false; motivo: "clave duplicada" | "yaml invalido" | "sin frontmatter" };

interface Argumentos {
  raiz: string;
  sinRed: boolean;
}

function leerArgumentos(argv: readonly string[]): Argumentos {
  let raiz: string | undefined;
  let sinRed = false;
  for (const a of argv) {
    if (a === "--sin-red") {
      sinRed = true;
    } else if (raiz !== undefined) {
      throw new ErrorPrograma(`argumento no reconocido: ${a}\nuso: tablero <raiz-del-repo> [--sin-red]`);
    } else {
      raiz = a;
    }
  }
  if (raiz === undefined) {
    throw new ErrorPrograma("uso: tablero <raiz-del-repo> [--sin-red]");
  }
  return { raiz, sinRed };
}

/** Rutas relativas (con `/`) de todos los `tarea.md` bajo `<raiz>/tareas`, en orden de bytes UTF-8. */
function buscarTareas(raiz: string): string[] {
  const encontradas: string[] = [];
  const recorrer = (relativa: string): void => {
    let entradas;
    try {
      entradas = readdirSync(join(raiz, relativa), { withFileTypes: true });
    } catch (e) {
      throw new ErrorPrograma(`no se puede leer ${relativa}: ${(e as Error).message}`);
    }
    for (const entrada of entradas) {
      const hija = `${relativa}/${entrada.name}`;
      if (entrada.isDirectory()) {
        recorrer(hija);
      } else if (entrada.isFile() && entrada.name === "tarea.md") {
        encontradas.push(hija);
      }
    }
  };
  // Como la PoC Rust: si `tareas/` no existe (o no es carpeta) hay 0 tareas.
  let esCarpeta = false;
  try {
    esCarpeta = statSync(join(raiz, "tareas")).isDirectory();
  } catch {
    esCarpeta = false;
  }
  if (esCarpeta) {
    recorrer("tareas");
  }
  return encontradas.sort((a, b) => Buffer.compare(Buffer.from(a, "utf8"), Buffer.from(b, "utf8")));
}

/** Texto YAML entre la primera linea `---` del fichero y la siguiente `---`; null si no hay frontmatter. */
function extraerFrontmatter(contenido: string): string | null {
  const lineas = contenido.replace(/^﻿/, "").split("\n").map((l) => l.replace(/\r$/, ""));
  if (lineas[0] !== "---") {
    return null;
  }
  const cierre = lineas.indexOf("---", 1);
  if (cierre === -1) {
    return null;
  }
  return lineas.slice(1, cierre).join("\n");
}

/** Valor de un campo: `ausente`, `nulo`, el texto fuente del escalar o `invalido` (lista, mapa). */
type Campo = { tipo: "ausente" } | { tipo: "nulo" } | { tipo: "texto"; texto: string } | { tipo: "invalido" };

/**
 * Busca `clave` en el mapa raiz (y en las claves de fusion `<<`, como serde-saphyr) y devuelve su
 * escalar como el texto fuente (`1.50` sigue siendo `1.50`, no `1.5`), igual que la PoC Rust.
 */
function campo(doc: Document, mapa: unknown, clave: string, profundidad = 0): Campo {
  if (!isMap(mapa) || profundidad > 8) {
    return { tipo: "ausente" };
  }
  const resolver = (n: unknown): unknown => (isAlias(n) ? n.resolve(doc) : n);
  const fusiones: unknown[] = [];
  for (const par of mapa.items) {
    const k = resolver(par.key);
    // Con `merge: true`, la clave `<<` llega como Symbol("<<").
    const nombre = isScalar(k) ? (typeof k.value === "symbol" ? k.value.description : String(k.value)) : undefined;
    if (nombre === clave) {
      const v = resolver(par.value);
      if (v === null || v === undefined || (isScalar(v) && v.value === null)) {
        return { tipo: "nulo" };
      }
      if (isScalar(v)) {
        return { tipo: "texto", texto: typeof v.source === "string" ? v.source : String(v.value) };
      }
      return { tipo: "invalido" };
    }
    if (nombre === "<<") {
      fusiones.push(resolver(par.value));
    }
  }
  for (const f of fusiones) {
    const r = campo(doc, f, clave, profundidad + 1);
    if (r.tipo !== "ausente") {
      return r;
    }
  }
  return { tipo: "ausente" };
}

function analizarTarea(contenido: string): Resultado {
  const yamlTexto = extraerFrontmatter(contenido);
  if (yamlTexto === null) {
    return { ok: false, motivo: "sin frontmatter" };
  }
  const doc = parseDocument(yamlTexto, { uniqueKeys: true, prettyErrors: false, merge: true });
  if (doc.errors.some((e) => e.code === "DUPLICATE_KEY")) {
    return { ok: false, motivo: "clave duplicada" };
  }
  if (doc.errors.length > 0 || !isMap(doc.contents)) {
    return { ok: false, motivo: "yaml invalido" };
  }
  // Como la PoC Rust: `id` y `titulo` obligatorios y escalares no nulos; `estado` opcional.
  const id = campo(doc, doc.contents, "id");
  const titulo = campo(doc, doc.contents, "titulo");
  const estado = campo(doc, doc.contents, "estado");
  if (id.tipo !== "texto" || titulo.tipo !== "texto" || estado.tipo === "invalido") {
    return { ok: false, motivo: "yaml invalido" };
  }
  return {
    ok: true,
    tarea: {
      id: id.texto,
      titulo: titulo.texto,
      estado: estado.tipo === "texto" ? estado.texto : "desconocido",
    },
  };
}

function ultimoCommit(raiz: string): string {
  try {
    const salida = execFileSync("git", ["-C", raiz, "log", "-1", "--format=%H"], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
      windowsHide: true,
    });
    return (salida.split("\n")[0] ?? "").replace(/\r$/, "");
  } catch (e) {
    throw new ErrorPrograma(`fallo git log: ${(e as Error).message.trim()}`);
  }
}

async function comprobarRed(): Promise<"si" | "no"> {
  try {
    // Como la PoC Rust (platform-verifier): confiar tambien en el almacen del sistema. Sin esto, el
    // fetch de Node solo usa las raices de Mozilla embebidas y falla con una CA local
    // (UNABLE_TO_VERIFY_LEAF_SIGNATURE) salvo que el entorno defina NODE_USE_SYSTEM_CA o NODE_EXTRA_CA_CERTS.
    setDefaultCACertificates([...getCACertificates("default"), ...getCACertificates("system")]);
    const respuesta = await fetch(URL_RED, {
      method: "GET",
      headers: { "User-Agent": USER_AGENT },
      signal: AbortSignal.timeout(TIMEOUT_RED_MS),
    });
    await respuesta.body?.cancel();
    return respuesta.ok ? "si" : "no";
  } catch {
    return "no";
  }
}

function compararBytes(a: string, b: string): number {
  return Buffer.compare(Buffer.from(a, "utf8"), Buffer.from(b, "utf8"));
}

async function principal(argv: readonly string[]): Promise<string> {
  const { raiz, sinRed } = leerArgumentos(argv);
  const rutas = buscarTareas(raiz);

  const porEstado = new Map<string, number>();
  const errores: string[] = [];
  let validas = 0;
  for (const ruta of rutas) {
    let contenido: string;
    try {
      contenido = readFileSync(join(raiz, ruta), "utf8");
    } catch (e) {
      throw new ErrorPrograma(`no se puede leer ${ruta}: ${(e as Error).message}`);
    }
    const r = analizarTarea(contenido);
    if (r.ok) {
      validas++;
      porEstado.set(r.tarea.estado, (porEstado.get(r.tarea.estado) ?? 0) + 1);
    } else {
      errores.push(`error ${ruta}: ${r.motivo}`);
    }
  }

  const git = ultimoCommit(raiz);
  const red = sinRed ? "omitida" : await comprobarRed();

  const lineas = [`tareas: ${validas}`, `errores: ${errores.length}`];
  for (const estado of [...porEstado.keys()].sort(compararBytes)) {
    lineas.push(`${estado}: ${porEstado.get(estado)}`);
  }
  lineas.push(`git: ${git}`, `red: ${red}`, ...errores);
  return lineas.join("\n") + "\n";
}

principal(process.argv.slice(2)).then(
  (salida) => {
    process.stdout.write(salida);
    process.exitCode = 0;
  },
  (e: unknown) => {
    const mensaje = e instanceof Error ? e.message : String(e);
    process.stderr.write(`tablero: ${mensaje}\n`);
    process.exitCode = 1;
  },
);

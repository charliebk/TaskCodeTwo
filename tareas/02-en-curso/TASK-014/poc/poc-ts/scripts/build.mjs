// Build en limpio del ejecutable unico (Node SEA) FUERA del repo.
// Uso: node scripts/build.mjs [carpeta-de-build]
// Por defecto: %LOCALAPPDATA%/pruebas-claude/taskcodetwo/task-014/build-ts
// Pasos: copiar fuentes -> npm ci -> tsc --noEmit -> esbuild (bundle CJS)
//        -> blob SEA -> copiar node.exe -> quitar firma -> postject.
import { spawnSync } from "node:child_process";
import { copyFileSync, cpSync, existsSync, mkdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const FUSE = "NODE_SEA_FUSE_fce680ab2cc467b6e072b8b5df1996b2";
const fuentes = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const porDefecto = process.env.LOCALAPPDATA
  ? join(process.env.LOCALAPPDATA, "pruebas-claude", "taskcodetwo", "task-014", "build-ts")
  : undefined;
const destino = resolve(process.argv[2] ?? porDefecto ?? "");
if (!process.argv[2] && !porDefecto) {
  throw new Error("indica la carpeta de build");
}
if (destino.toLowerCase().startsWith(fuentes.toLowerCase())) {
  throw new Error(`la build no puede ir dentro de las fuentes: ${destino}`);
}

const tiempos = [];
function paso(nombre, fn) {
  const t0 = performance.now();
  fn();
  const ms = Math.round(performance.now() - t0);
  tiempos.push([nombre, ms]);
  console.log(`[build] ${nombre}: ${ms} ms`);
}
function ejecutar(comando, args, opciones = {}) {
  const r = spawnSync(comando, args, { cwd: destino, stdio: "inherit", shell: process.platform === "win32", ...opciones });
  if (r.status !== 0) {
    throw new Error(`fallo (${r.status}): ${comando} ${args.join(" ")}`);
  }
}
const q = (s) => (process.platform === "win32" ? `"${s}"` : s);

paso("copiar fuentes", () => {
  for (const sub of ["src", "dist"]) {
    rmSync(join(destino, sub), { recursive: true, force: true });
  }
  mkdirSync(destino, { recursive: true });
  for (const f of ["package.json", "package-lock.json", "tsconfig.json"]) {
    if (existsSync(join(fuentes, f))) {
      copyFileSync(join(fuentes, f), join(destino, f));
    }
  }
  cpSync(join(fuentes, "src"), join(destino, "src"), { recursive: true });
});

paso("npm ci", () => {
  rmSync(join(destino, "node_modules"), { recursive: true, force: true });
  ejecutar("npm", existsSync(join(destino, "package-lock.json")) ? ["ci", "--no-audit", "--no-fund"] : ["install", "--no-audit", "--no-fund"]);
});

paso("tsc --noEmit", () => ejecutar(q(join(destino, "node_modules", ".bin", "tsc")), ["--noEmit", "-p", "tsconfig.json"]));

paso("esbuild bundle", () =>
  ejecutar(q(join(destino, "node_modules", ".bin", "esbuild")), [
    "src/main.ts",
    "--bundle",
    "--platform=node",
    "--target=node22",
    "--format=cjs",
    "--legal-comments=none",
    "--outfile=dist/tablero.cjs",
  ]),
);

const exe = join(destino, "dist", process.platform === "win32" ? "tablero.exe" : "tablero");
paso("blob SEA", () => {
  writeFileSync(
    join(destino, "sea-config.json"),
    JSON.stringify({ main: "dist/tablero.cjs", output: "dist/sea-prep.blob", disableExperimentalSEAWarning: true, useCodeCache: false }, null, 2),
  );
  ejecutar(q(process.execPath), ["--experimental-sea-config", "sea-config.json"]);
});

paso("copiar node y quitar firma", () => {
  copyFileSync(process.execPath, exe);
  if (process.platform === "win32") {
    const signtool = buscarSigntool();
    if (signtool) {
      ejecutar(q(signtool), ["remove", "/s", q(exe)]);
    } else {
      console.warn("[build] signtool no encontrado: postject avisara de la firma, el exe sigue funcionando sin firma valida");
    }
  }
});

paso("postject", () =>
  ejecutar(q(join(destino, "node_modules", ".bin", "postject")), [q(exe), "NODE_SEA_BLOB", "dist/sea-prep.blob", "--sentinel-fuse", FUSE]),
);

const total = tiempos.reduce((s, [, ms]) => s + ms, 0);
console.log(`[build] total: ${total} ms`);
console.log(`[build] exe: ${exe} (${statSync(exe).size} bytes)`);

function buscarSigntool() {
  const base = "C:\\Program Files (x86)\\Windows Kits\\10\\bin";
  const candidatas = ["10.0.26100.0", "10.0.22621.0", "10.0.22000.0", "10.0.19041.0", "10.0.17134.0"].map((v) => join(base, v, "x64", "signtool.exe"));
  return candidatas.find((c) => existsSync(c));
}

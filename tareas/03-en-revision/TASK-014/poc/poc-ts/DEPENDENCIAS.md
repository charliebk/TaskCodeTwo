# Dependencias de la PoC TypeScript (poc-ts)

Versiones fijadas exactas en `package.json` y `package-lock.json`. Consulta al
registro de npm el 2026-11-14.

## En el binario (runtime)

| Paquete | Versión | Publicada | Última del paquete | Dependencias | Motivo |
|---|---|---|---|---|---|
| `yaml` | 2.9.1 | 2026-09-11 | 2.9.1 (2026-09-11) | 0 | Librería YAML 1.2 mantenida (eemeli). `parseDocument(..., { uniqueKeys: true })` deja un error con código `DUPLICATE_KEY`, lo que permite distinguir `clave duplicada` de `yaml invalido` sin parser propio. `js-yaml` descartada: menos activa y el error de duplicado sólo se distingue por el texto del mensaje. |

Lo demás es de Node 22: `fetch` nativo (undici interno) para el GET HTTPS,
`child_process.execFileSync` para `git`, `fs` para el recorrido.

## Sólo de build (no van al binario)

| Paquete | Versión | Publicada | Última del paquete | Dependencias | Motivo |
|---|---|---|---|---|---|
| `typescript` | 7.0.2 | 2026-07-08 | 7.0.2 | binario nativo por plataforma (`@typescript/typescript-win32-x64`) | Comprobación de tipos (`tsc --noEmit`, modo estricto). No emite: el JS lo produce esbuild. |
| `esbuild` | 0.28.2 | 2026-08-08 | 0.28.2 | binario nativo (`@esbuild/win32-x64`) | Empaqueta `src/main.ts` + `yaml` en un único CJS, requisito de Node SEA (el script principal sólo puede hacer `require` de módulos internos). |
| `postject` | 1.0.0-alpha.6 | 2023-05-19 | 1.0.0-alpha.6 | `commander` ^9 | Inyecta el blob SEA en la copia de `node.exe`. Es la herramienta que indica la documentación oficial de Node SEA y está bajo la organización nodejs, **pero su última publicación es de mayo de 2023 y sigue en alfa**. No cumple estrictamente "solo dependencias mantenidas"; en Node 22 no hay alternativa oficial (`node --build-sea` llega en versiones posteriores). Riesgo anotado. |
| `@types/node` | 22.20.5 | 2026-10-01 | 26.6.6 (rama 22 fijada a propósito) | `undici-types` | Tipos de la API de Node 22. |

Transitivas: `commander` (de postject), `undici-types` (de @types/node).

## Herramientas del sistema usadas (ya instaladas)

- Node 22.23.4 y npm 10.9.9 (el `node.exe` se copia como base del ejecutable).
- `signtool.exe` del Windows SDK 10.0.26100.0 (`signtool remove /s`), opcional: si no está, postject inyecta igualmente sobre el binario firmado (firma inválida).
- `git` en el PATH en tiempo de ejecución (la especificación pide proceso hijo).

Total en `node_modules` tras `npm ci`: **9 paquetes** (yaml, typescript,
@typescript/typescript-win32-x64, esbuild, @esbuild/win32-x64, postject,
commander, @types/node, undici-types).

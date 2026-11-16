---
id: TASK-026
titulo: "F2-11 Instaladores de una linea"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-026-f2-11-instaladores-de-una-linea
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 352073
tokens_implementacion: 23843522
tokens_revision: 12837681
creado: 2026-11-14
actualizado: 2026-11-16
dependencias: [TASK-025]
---
## Objetivo

Instaladores de una línea (PowerShell y shell) que descargan, verifican la firma o el hash y ponen `taskctl` en el PATH

Tarea F2-11 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-10.

## Criterios de aceptacion
- [x] Instalación en una máquina limpia de cada sistema en menos de un minuto (con la definición aprobada en el plan: menos de 60 s en los 6 runners nativos y en alpine; la máquina limpia real es de F9-9a, TASK-071)
- [x] Cumple lo que pide la tarea F2-11 del plan y los requisitos que cita

## Resultado

**Implementado.**

- `instalar/install.sh` (POSIX sh, curl o wget) e `instalar/install.ps1` (Windows PowerShell 5.1 y 7). Bajan `SHA256SUMS` y el binario de su alias de P12, exigen una sola línea que case y comprueban el sha256. Si hay `gh` con sesión, comprueban también la attestation (`--signer-workflow`); sin `gh`, lo avisan y siguen. Prueban `tc2 --version` antes de dejar el binario en su sitio y lo ponen en el PATH: en Windows, PATH de usuario en el registro, sin administrador; en Unix, un bloque con marcas en el perfil del shell. Instalan `tc2`, no `taskctl` (divergencia 5 del ADR F1-14), y no tocan la 0.16.
- `release.yml`:
  - `instaladores-ensayo`: los binarios del build servidos en 127.0.0.1, en 6 runners más 2 alpine.
  - `publicar` adjunta los instaladores, con attestation y fuera de `SHA256SUMS`: 9 assets.
  - `verificar-instaladores`: la línea real contra la release publicada.
- `.github/scripts/probar-instalador.sh`: instalación medida hasta que una terminal nueva encuentra `tc2`, hash alterado, PATH idempotente, reinstalación en otro directorio, perfil con marcas rotas y perfil CRLF.
- `xtask/src/instaladores.rs`: tabla de alias contra `dist::TARGETS`, cada uname o arquitectura con su alias, ASCII, LF, llamada en la última línea y sin `exit` en el ps1.
- `docs/distribucion/instalar.md` (nuevo); `releases.md` y la fila F2-11 del plan, actualizados.

**Evidencia.**

- Prerelease `v0.0.0-prueba.26`, run 43851819615: 37 jobs en verde y 9 assets.

  | Caso | Tiempo de `verificar-instaladores` | Attestation |
  |---|---|---|
  | darwin-x64 | 6 s | correcta |
  | darwin-arm64 | 6 s | correcta |
  | linux-x64 | 5 s | correcta |
  | linux-arm64 | 8 s | correcta |
  | win-x64 | 11 s | correcta |
  | win-arm64 | 54 s | correcta |
  | alpine x64 | 1 s | sin `gh`: aviso |
  | alpine arm64 | 1 s | sin `gh`: aviso |

  Sin `TC2_VERSION`, los 8 casos fallan con «no hay ninguna release estable» y el arreglo. La release y el tag se borraron después, con el permiso del plan.
- Último ensayo de la rama (run 43859923647, commit 0274adf): verde en los 8 casos.

**Revisión.** 3 rondas; la tercera fue extra, decidida por la persona.

- Ronda 1, `code-quality-reviewer`: IMP-1, IMP-2 y MEN-1 a MEN-3.
- Ronda 1, `rust-wasm-reviewer`: MEN-1 y MEN-2.
- Ronda 2: confirmó IMP-1 e IMP-2 y encontró R2-1 (CRÍTICO, introducido por la corrección de IMP-2: con la marca de fin borrada, se perdía el resto del perfil), además de dos MENORES.
- Ronda 3: aprobada.

Todos los hallazgos están corregidos. Las notas sin tabla de la ronda 3 quedan sin corregir, porque no hay pérdida de datos:

- una marca de fin huérfana obliga a arreglar a mano;
- se quita el blanco pegado al bloque;
- el texto del error cuando las marcas no van solas en su línea;
- el bloque nuevo se escribe en LF dentro de un perfil CRLF.

Defectos encontrados en las pruebas propias, antes de la revisión, y corregidos:

- un `.exe` recién ejecutado quedaba bloqueado y no se podía mover al destino;
- un fallo de red se anunciaba como «no hay release estable»;
- un `TC2_INSTALL_DIR` relativo quedaba relativo.

**Riesgos y límites aceptados.**

- **win-arm64 está cerca del límite: 44-54 s frente a 60.** El instalador tarda menos de 1 s. El resto es un coste del runner `windows-11-arm`: un proceso nuevo de PowerShell que usa un cmdlet de `Microsoft.PowerShell.Utility` tarda 42-56 s. No lo explican ni el `LOCALAPPDATA` vacío ni los módulos de Azure del `PSModulePath`: las dos causas se probaron y se descartaron. La causa sigue sin encontrarse. El coste entra en la medida y el log lo muestra aparte. Puede hacer fallar `verificar-instaladores` unas veces sí y otras no; F9-9a (TASK-071) lo medirá en una máquina ARM real.
- **La ruta `latest`:** solo está probado su error, porque todavía no hay ninguna release estable.
- **Reinstalar en otro directorio en Windows** no quita la entrada antigua del PATH. Está documentado; lo resuelve F2-14.
- **Sin probar:** Rosetta, proxy corporativo con CA propia, SmartScreen o MOTW, y la cuarentena de macOS.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-16T08:55:05Z | plan | automatico | persona | 0.16.1 |
| 2026-11-16T09:18:26Z | approve | automatico | persona | 0.16.1 |
| 2026-11-16T09:18:38Z | start | automatico | persona | 0.16.1 |
| 2026-11-16T11:00:28Z | review | automatico | persona | 0.16.1 |
| 2026-11-16T11:26:17Z | review | automatico | persona | 0.16.1 |
| 2026-11-16T11:49:49Z | review | automatico | persona | 0.16.1 |
| 2026-11-16T12:03:25Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-16T09:00:18Z | diseno | agente:a5d80e07e9effb23b | 16 | 66714 | 268770 | 16573 | 21 | 352073 | - | - |
| 2026-11-16T11:00:28Z | implementacion | sesion:bb6e0871-29bf-42e4-b8be-661dbe95cd05 | 200 | 193055 | 23542909 | 107358 | 12 | 23843522 | - | - |
| 2026-11-16T11:06:37Z | revision | agente:affebe3dec23302b7 | 20 | 112325 | 919266 | 7836 | 2 | 1039447 | 1 | rust-wasm-reviewer |
| 2026-11-16T11:06:37Z | revision | agente:a08316f48c9d20785 | 28 | 142626 | 1586182 | 19765 | 1 | 1748601 | 1 | code-quality-reviewer |
| 2026-11-16T11:35:39Z | revision | agente:a60563144f58d1cf8 | 86 | 178083 | 6171117 | 55621 | 9 | 6404907 | 2 | code-quality-reviewer |
| 2026-11-16T11:35:39Z | revision | agente:a78d78d01a8598de9 | 20 | 108244 | 889801 | 8319 | 1 | 1006384 | 2 | rust-wasm-reviewer |
| 2026-11-16T11:58:29Z | revision | agente:aabddc7cfbab4f809 | 46 | 130488 | 2493843 | 13965 | 2 | 2638342 | 3 | - |

## Autorrevision

- rust-wasm-reviewer — f309ec2
- code-quality-reviewer — f309ec2
- rust-wasm-reviewer — dedddf7
- code-quality-reviewer — dedddf7
- code-quality-reviewer — 0274adf

## Rondas extra

- Ronda 3: la persona (Carlos), en el chat de la sesion, para corregir R2-1 (perdida de lineas del perfil del shell que introdujo la correccion de IMP-2) antes de integrar.

## Duracion

- Estimada: 90 min
- Real (de la rama abierta al cierre con merge, sin pausas): 165 min
- Paso de la hora: 165 min reales frente a 90 estimados
- Cierre (finish): 250 s, de 2026-11-16T11:59:15Z a 2026-11-16T12:03:25Z

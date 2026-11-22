---
id: TASK-078
titulo: "F2-14 tc2 desinstalar: quitar TaskCodeTwo de forma limpia"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-078-f2-14-taskctl-desinstalar-quitar-taskcod
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: 23910346
tokens_revision: 15038066
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-018, TASK-025, TASK-027, TASK-116]
---
## Objetivo

`taskctl desinstalar`: quita TaskCodeTwo de forma limpia (plugin, CLI, status line, hooks, worktree de la rama de estado); en un proyecto, la config solo con confirmación y nunca las tareas ni su historial sin confirmación explícita; con `--simular` (RR-16)

Tarea F2-14 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-3, F2-10, F2-12.

Contexto (Carlos, 2026-11-14, RR-16): hace falta un comando para desinstalar de forma limpia tanto TaskCode (0.16) como TaskCodeTwo, sin dejar rastro en la máquina ni en los proyectos (plugin, CLI, status line, hooks, ficheros de estado, logs, caché del marketplace) y sin perder tareas ni historial salvo confirmación explícita de la persona.

## Criterios de aceptacion
- [x] C1: `tc2 desinstalar --maquina --simular` y `--proyecto --simular` imprimen una linea por accion (`ruta`, `clase`, `dueno`, `decision` borra|conserva|persona) y salen 0; la instantanea (rutas, tamanos, sha256) del HOME falso y del repo es igual antes y despues (`tests/desinstalar.rs::simular_no_toca_nada`)
- [x] C2: maquina en modo instalador: tras instalar con `install.sh`/`install.ps1` y `TC2_INSTALL_DIR` en un HOME falso, desinstalar quita el binario, `tc2.exe.old`, los `.tc2-*.descarga`, el bloque de marcas de cada perfil (resto del perfil identico byte a byte), la entrada del PATH de usuario en Windows (las demas intactas) y la cache `taskcodetwo/cli/` (test; paso nuevo en `probar-instalador.sh`, no verificado hasta el push)
- [x] C3: la `statusLine` de `~/.claude/settings.json` se quita solo si su `command` es nuestro por el reconocedor cerrado; una ajena o la de la 0.16 queda intacta (test con tres muestras)
- [x] C4: `desinstalar` nunca edita la clave `hooks` de un `settings.json`; si alguna entrada nombra `tc2` la lista como aviso (settings con hooks ajenos identico byte a byte)
- [x] C5: proyecto: quita el worktree de la rama de estado y su registro, la carpeta de la 2.0 en el directorio comun de git y sus lineas de `.git/info/exclude`; los worktrees de tarea solo si estan limpios y publicados (si no, `rechazada` sin tocar nada); el remoto queda igual (`git ls-remote` antes y despues)
- [x] C6: `.taskcode/config.yml` y `personas.yml` solo se borran con `--config`; sin ella se conservan y la linea lo dice
- [x] C7: la rama local de estado solo se borra con `--borrar-tareas <nombre-exacto>`; la remota nunca (se imprime el `git push --delete` para la persona)
- [x] C8: Windows con el binario en uso: un proceso auxiliar lo borra tras salir (<= 10 s); si otro proceso lo bloquea, `pendiente motivo=en-uso` sin dejar nada a medias (runner Windows)
- [x] C9: en modo plugin o gestor el binario no se borra y se escribe la orden exacta que lo quita
- [x] C10: con huella de la 0.16 en el HOME falso y en el repo, todo eso queda identico byte a byte
- [x] C11: una segunda ejecucion responde `sin-cambios`

## Resultado

Hecho: `tc2 desinstalar` con ambitos `--maquina` y `--proyecto`, plan como dato (inventario → plan → aplicacion) y `--simular`, sin red. Reconocedor cerrado de propiedad (rutas de `huella.rs`, marcas del perfil y `STATUS_LINE` como constantes en `distribucion`, exigidas a `install.sh` por un test de xtask): solo se borra lo de la 2.0; lo ajeno, de Claude Code, de gestores o de la 0.16 se lista con la orden que lo quita. `--config` (nunca si la config o `personas.yml` son de la 0.16) y `--borrar-tareas <rama>` (avisa con el sha si la rama local va por delante del remoto); la remota nunca se borra. Autoborrado del binario en Windows con un `powershell.exe` auxiliar y PATH de usuario por su inversa exacta, tras puertos con implementacion en memoria. Escrituras atomicas; rechazo sin tocar nada con worktrees bloqueados o desde dentro. Excepcion acotada al puerto git: `borrar_rama_de_estado` (ref fija, sha viejo exigido) y `GIT_OPTIONAL_LOCKS=0` para todo el CLI. Cada criterio C1..C11 con su test (18 en `tests/desinstalar.rs`, todos con HOME falso) mas el test real del auxiliar de autoborrado; `cargo xtask verificar --perfil rapida` en verde.

No verificado: el paso de desinstalacion de `probar-instalador.sh` (CI, sin push), las llamadas reales a `powershell.exe` sobre el registro y el PATH reales (regla de seguridad) y `es_tc2` con un `tc2` real ajeno.

Revision (rust-wasm-reviewer y code-quality-reviewer):
- Ronda 1: cambios solicitados. IMP-1 rust (autoborrado con `DETACHED_PROCESS` no hacia nada), IMP-2 rust (`personas.yml` de la 0.16 se borraba con `--config`), IMP-1 calidad (diagnostico sin `tc2.exe`), IMP-2 calidad (falta test de dos directorios en el PATH) y los MENORES corregidos en d7ed058, ca7a71a, c378e94 y b0f38bd.
- Ronda 2: aprobada por los dos, con mutaciones que ponen los tests en rojo.

MENORES aceptados sin corregir: MEN-9 calidad r1 (separadores `/` y `\` mezclados en las rutas); notas de r2: `un_tc2_ajeno_no_se_borra` no distingue «se ejecuta y no es tc2» de «no se puede ejecutar» y falta test de la rama positiva de `es_tc2`; `es_tc2` ejecuta el binario del directorio de instalacion tambien con `--simular`; `escribir_atomico` rompe un enlace duro y exige escritura en la carpeta; las comillas simples de `claude plugin uninstall` no valen en `cmd.exe`; una config con `formato: 2` y esquema roto cuenta como de la 2.0. Van a una tarea de seguimiento.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-16T22:31:19Z | plan | automatico | persona | 0.16.1 |
| 2026-11-16T22:37:05Z | approve | automatico | persona | 0.16.1 |
| 2026-11-16T23:54:31Z | start | automatico | persona | 0.16.1 |
| 2026-11-17T00:41:30Z | review | automatico | persona | 0.16.1 |
| 2026-11-17T01:08:39Z | review | automatico | persona | 0.16.1 |
| 2026-11-17T01:16:51Z | finish | automatico | persona | 0.16.1 |

## Autorrevision

- rust-wasm-reviewer — 390580b
- code-quality-reviewer — 390580b

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T00:41:30Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 20 | 8855 | 4040024 | 2905 | 0 | 4051804 | - | - |
| 2026-11-17T00:41:30Z | implementacion | agente:a2bf6b38f49ca3b69 | 172 | 609269 | 18447750 | 119936 | 25 | 19177127 | - | - |
| 2026-11-17T00:57:25Z | revision | agente:a285245acc158d49c | 80 | 170746 | 4917372 | 39476 | 2 | 5127674 | 1 | rust-wasm-reviewer |
| 2026-11-17T00:57:26Z | revision | agente:a7fea3660749fafa5 | 82 | 201474 | 5967016 | 45022 | 10 | 6213594 | 1 | code-quality-reviewer |
| 2026-11-17T01:08:39Z | implementacion | sesion:dif:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 2 | 1140 | 679922 | 351 | 0 | 681415 | - | - |
| 2026-11-17T01:14:30Z | revision | agente:a081f2d09db1664dc | 36 | 114863 | 1711179 | 9598 | 4 | 1835676 | 2 | rust-wasm-reviewer |
| 2026-11-17T01:14:30Z | revision | agente:a295ef0b672c4ecb3 | 42 | 103458 | 1747321 | 10301 | 1 | 1861122 | 2 | code-quality-reviewer |

## Duracion

- Estimada: 300 min
- Real (de la rama abierta al cierre con merge, sin pausas): 82 min
- Paso de la hora: 82 min reales frente a 300 estimados
- Cierre (finish): 139 s, de 2026-11-17T01:14:32Z a 2026-11-17T01:16:51Z

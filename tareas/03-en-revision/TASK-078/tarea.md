---
id: TASK-078
titulo: "F2-14 tc2 desinstalar: quitar TaskCodeTwo de forma limpia"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-078-f2-14-taskctl-desinstalar-quitar-taskcod
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: 23228931
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-018, TASK-025, TASK-027, TASK-116]
---
## Objetivo

`taskctl desinstalar`: quita TaskCodeTwo de forma limpia (plugin, CLI, status line, hooks, worktree de la rama de estado); en un proyecto, la config solo con confirmación y nunca las tareas ni su historial sin confirmación explícita; con `--simular` (RR-16)

Tarea F2-14 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-3, F2-10, F2-12.

Contexto (Carlos, 2026-11-14, RR-16): hace falta un comando para desinstalar de forma limpia tanto TaskCode (0.16) como TaskCodeTwo, sin dejar rastro en la máquina ni en los proyectos (plugin, CLI, status line, hooks, ficheros de estado, logs, caché del marketplace) y sin perder tareas ni historial salvo confirmación explícita de la persona.

## Criterios de aceptacion
- [ ] C1: `tc2 desinstalar --maquina --simular` y `--proyecto --simular` imprimen una linea por accion (`ruta`, `clase`, `dueno`, `decision` borra|conserva|persona) y salen 0; la instantanea (rutas, tamanos, sha256) del HOME falso y del repo es igual antes y despues (`tests/desinstalar.rs::simular_no_toca_nada`)
- [ ] C2: maquina en modo instalador: tras instalar con `install.sh`/`install.ps1` y `TC2_INSTALL_DIR` en un HOME falso, desinstalar quita el binario, `tc2.exe.old`, los `.tc2-*.descarga`, el bloque de marcas de cada perfil (resto del perfil identico byte a byte), la entrada del PATH de usuario en Windows (las demas intactas) y la cache `taskcodetwo/cli/` (test; paso nuevo en `probar-instalador.sh`, no verificado hasta el push)
- [ ] C3: la `statusLine` de `~/.claude/settings.json` se quita solo si su `command` es nuestro por el reconocedor cerrado; una ajena o la de la 0.16 queda intacta (test con tres muestras)
- [ ] C4: `desinstalar` nunca edita la clave `hooks` de un `settings.json`; si alguna entrada nombra `tc2` la lista como aviso (settings con hooks ajenos identico byte a byte)
- [ ] C5: proyecto: quita el worktree de la rama de estado y su registro, la carpeta de la 2.0 en el directorio comun de git y sus lineas de `.git/info/exclude`; los worktrees de tarea solo si estan limpios y publicados (si no, `rechazada` sin tocar nada); el remoto queda igual (`git ls-remote` antes y despues)
- [ ] C6: `.taskcode/config.yml` y `personas.yml` solo se borran con `--config`; sin ella se conservan y la linea lo dice
- [ ] C7: la rama local de estado solo se borra con `--borrar-tareas <nombre-exacto>`; la remota nunca (se imprime el `git push --delete` para la persona)
- [ ] C8: Windows con el binario en uso: un proceso auxiliar lo borra tras salir (<= 10 s); si otro proceso lo bloquea, `pendiente motivo=en-uso` sin dejar nada a medias (runner Windows)
- [ ] C9: en modo plugin o gestor el binario no se borra y se escribe la orden exacta que lo quita
- [ ] C10: con huella de la 0.16 en el HOME falso y en el repo, todo eso queda identico byte a byte
- [ ] C11: una segunda ejecucion responde `sin-cambios`

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-16T22:31:19Z | plan | automatico | persona | 0.16.1 |
| 2026-11-16T22:37:05Z | approve | automatico | persona | 0.16.1 |
| 2026-11-16T23:54:31Z | start | automatico | persona | 0.16.1 |
| 2026-11-17T00:41:30Z | review | automatico | persona | 0.16.1 |

## Autorrevision

- rust-wasm-reviewer — 390580b
- code-quality-reviewer — 390580b

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T00:41:30Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 20 | 8855 | 4040024 | 2905 | 0 | 4051804 | - | - |
| 2026-11-17T00:41:30Z | implementacion | agente:a2bf6b38f49ca3b69 | 172 | 609269 | 18447750 | 119936 | 25 | 19177127 | - | - |

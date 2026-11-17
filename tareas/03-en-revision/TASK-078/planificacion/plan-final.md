# Plan — TASK-078: F2-14 tc2 desinstalar: quitar TaskCodeTwo de forma limpia

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 300 min

`tc2 desinstalar` con dos ambitos explicitos, `--maquina` y `--proyecto` (sin ninguno → `uso`). Cada ambito: inventario de solo lectura con la huella encontrada y su dueno → plan calculado del inventario y de los permisos dados → aplicacion de ese mismo plan. `--simular` imprime el plan y para (patron «plan como dato» de F1-12). Ningun paso usa la red. Solo se borra lo que el inventario reconoce como propio de la 2.0; lo de Claude Code, gestores, otras herramientas o la 0.16 se lista con la orden que lo quita y no se toca. Titulo corregido a `tc2` (F1-14 divergencia 5).

Inventario (leido en los instaladores y en `actualizar_cli.rs`):
- Windows: `%LOCALAPPDATA%\\taskcodetwo\\bin\\tc2.exe` (o `TC2_INSTALL_DIR`), `tc2.exe.old`, `.tc2-<v>-<pid>.descarga`, `.tc2.nuevo.<pid>.exe`, entrada en `HKCU\\Environment\\Path`, cache `%LOCALAPPDATA%\\taskcodetwo\\cli\\<v>\\`.
- Linux: `~/.local/bin/tc2` (o `TC2_INSTALL_DIR`), bloque de marcas en `~/.profile`, `~/.bashrc`/`~/.zshrc`, `~/.bash_profile`; cache `~/.cache/taskcodetwo/cli/<v>/`.
- macOS: como Linux; cache `~/Library/Caches/taskcodetwo/cli/<v>/` o `~/.cache/...`.
- Claude Code: plugin y marketplace en `~/.claude/plugins/...` (son de Claude Code); `statusLine` de `~/.claude/settings.json` (la escribira F3-12).
- Proyecto: worktree de la rama de estado, worktrees de tarea, lineas de `.git/info/exclude`, cerrojo y cache de identidad en el directorio comun de git, `.taskcode/config.yml` y `personas.yml` (versionados), rama de estado local y remota. Los nombres exactos salen de TASK-116 (alineados con F1-12 D1a).
- Logs: la 2.0 no escribe ninguno hoy.

Frontera: el CLI borra lo de la 2.0 de esa lista (incluida la cache `taskcodetwo/cli/`, que Claude Code no conoce). La persona desinstala el plugin y el marketplace con la orden que imprime el CLI (los hooks se van con el plugin). El gestor de paquetes quita su binario. Orden recomendado: `--proyecto` en cada repo, luego `--maquina`, luego el plugin.

Propiedad: reconocedor cerrado (ruta exacta, marcas exactas o `command` exacto), nunca por parecido. Lo no reconocido se queda y se informa como `ajeno`. Marcas del perfil y forma de la status line como constantes compartidas con quien las escribe (instalador y F3-12).

Piezas:
- `instalacion` (se crea extrayendo de `actualizar_cli.rs` `Modo`, `detectar_modo`, `ruta_old`, `limpiar_old`, mas rutas por defecto desde un `Entorno` inyectado).
- `desinstalar::{inventario, plan, aplicar}` (nuevo; plan como `Vec<Accion{ruta, clase, dueno, requiere}>`).
- Marcas del perfil y reconocedor de status line como constantes en `distribucion`; test que exige que `install.sh` contenga exactamente esas marcas.
- `git::worktree` y `git::rama::excluir` se extienden: se reutiliza `quitar_worktree` y se anade la inversa de `excluir`.
- PATH de usuario en Windows y autoborrado del binario tras un puerto con implementacion real y en memoria.
- `lib.rs` y catalogo i18n `desinstalar`.

Orden de construccion:
1. Extraer `instalacion` y `Entorno` sin cambiar comportamiento.
2. Inventario, plan y `--simular` en los dos ambitos (C1).
3. Aplicacion en proyecto: worktrees, caches y `exclude`; luego `--config`; luego `--borrar-tareas` (C5, C6, C7).
4. Aplicacion en maquina Unix: binario, perfiles, cache del lanzador, status line (C2, C3, C4, C9).
5. Windows: PATH del registro y autoborrado (C2, C8); paso de desinstalacion en `probar-instalador.sh`.
6. C10 y C11 transversales.

Alternativas descartadas: `install.ps1 -Desinstalar`/`install.sh --desinstalar` (no llegan al estado de git ni a la status line; triplica la logica de proyecto); que el CLI edite `installed_plugins.json` y `enabledPlugins` (formato privado de Claude Code).

## Lo que el rol no cubrio

- Sin roles de riesgos, testing ni dominio: corte a mitad de la aplicacion (que queda), carrera con otro `tc2` abierto, suficiencia de las pruebas y si la confirmacion por opciones casa con RR-16/RN-06.
- Suposiciones sin verificar: existencia de `claude plugin uninstall` no interactivo y el nombre del plugin; que un plugin no pueda fijar `statusLine`; que la 0.16 no use la misma carpeta de worktrees (lo resuelve TASK-116); cache de macOS del lanzador (no existe aun, TASK-114); reescribir `settings.json` respetando el orden de claves (`serde_json` con `preserve_order`; el formato no se conserva).

## Riesgos aceptados y que los contiene

- Borrar lo ajeno: reconocedor cerrado; lo no reconocido se conserva y se informa; tests byte a byte (C3, C4, C10).
- Perder tareas: la rama local solo con `--borrar-tareas <nombre-exacto>`; la remota nunca (C7).
- Binario en uso en Windows: proceso auxiliar que espera al PID; si falla, `pendiente motivo=en-uso` sin estado a medias (C8).
- Nombres compartidos con la 0.16: resuelto antes por TASK-116 (dependencia).

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):
- Cada test con su HOME, `LOCALAPPDATA`, `XDG_CACHE_HOME` y `USERPROFILE` falsos inyectados por `Entorno`, en `temp_dir` o `%LOCALAPPDATA%\\pruebas-claude\\taskcodetwo\\task-078\\<test>`, y borra su carpeta.
- Un repo git de prueba con remoto bare por criterio de proyecto.
- Ningun test toca el registro real; el PATH real solo en los runners de CI (C2, C8, no verificables sin push).

## Lo que necesita decision de una persona

Carlos acepta las recomendaciones (consigna del 2026-11-16):
- D1: confirmacion por opciones (`--config`, `--borrar-tareas <rama>`), no interactiva.
- D2: la rama remota de tareas nunca se borra desde `tc2 desinstalar`; se imprime el `git push --delete`.
- D3: el plugin lo desinstala la persona con la orden que imprime el CLI.
- D4: binario en uso en Windows con proceso auxiliar que espera al PID.
- D5: PATH de Windows con un hijo `powershell.exe` que hace la inversa exacta de `Add-PathUsuario`.
- D6: la 0.16 no se toca (su desinstalacion es F9-10); solo se avisa de su huella.
- D7: divergencia de nombres con F1-12 D1a: se abre antes TASK-116 que los alinea; TASK-078 depende de ella.

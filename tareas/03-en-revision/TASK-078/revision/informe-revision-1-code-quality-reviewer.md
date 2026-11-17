# Informe de revision — TASK-078 (ronda 1)

- Commit revisado: b855c2057a8375e142ebdec0a665bce971e0af33
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

Propuesta del revisor: **aprobada-con-correcciones** (0 CRITICO, 2 IMPORTANTE, 9 MENOR). Dominio unico (Rust, scripts y docs); no llegue por exceso de dominios.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | abierto | crates/tc2/src/desinstalar/mod.rs:485 | implementacion |
| IMP-2 | IMPORTANTE | abierto | crates/tc2/src/desinstalar/maquina.rs:182 | implementacion |
| MEN-1 | MENOR | abierto | crates/tc2/src/desinstalar/maquina.rs:287 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/desinstalar/proyecto.rs:206 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/src/desinstalar/proyecto.rs:88 | implementacion |
| MEN-4 | MENOR | abierto | crates/tc2/src/desinstalar/proyecto.rs:239 | implementacion |
| MEN-5 | MENOR | abierto | docs/distribucion/instalar.md:167 | implementacion |
| MEN-6 | MENOR | abierto | crates/tc2/src/main.rs:17 | plan |
| MEN-7 | MENOR | abierto | crates/tc2/src/desinstalar/mod.rs:528 | implementacion |
| MEN-8 | MENOR | abierto | crates/tc2/src/desinstalar/maquina.rs:338 | implementacion |
| MEN-9 | MENOR | abierto | crates/tc2/src/desinstalar/mod.rs:368 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

## Verificacion ejecutada

Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-078-r1-calidad`, rama de la tarea, HEAD 2fa75ca (codigo igual al de b855c20; la peticion no traia verificacion de ronda, la corri yo).

- `cargo fmt --check`: limpio. `cargo clippy --workspace --all-targets`: limpio.
- `cargo test --workspace`: todo verde, sin caidas por LNK1104 ni por 127.0.0.1. `tests/desinstalar.rs`: 14 de 14; unitarios de `tc2`: 96; `xtask`: 10.
- No ejecute `probar-instalador.sh` (en Windows instala de verdad en el PATH de usuario real): su paso nuevo queda **no verificado** por mi; solo lo lei (cksum del perfil, espera de 20 s al auxiliar, PATH esperado calculado antes) y no vi fallos.

### Criterio por criterio (test citado y mutacion)

Todas las mutaciones se aplicaron en el clon y se revirtieron con `git checkout`.

| Criterio | Test | Mutacion | Resultado |
|---|---|---|---|
| C1 | `simular_no_toca_nada` | `if !op.simular` pasa a `if true` | roja (cae en `borrados.is_empty()`) |
| C2 (Unix) | `maquina_unix_quita_lo_suyo_y_deja_el_perfil_igual` | no quitar la linea en blanco previa del bloque | roja |
| C2/C8 (Win) | `maquina_windows_path_y_autoborrado` | sin mutar (Windows real solo en CI) | verde; ver IMP-2 |
| C3 | `status_line_solo_la_de_la_2` | `==` pasa a `contains("tc2")` | roja |
| C4 | `hooks_nunca_se_editan_y_se_avisa` | suprimir el aviso | roja |
| C5 | `proyecto_quita_worktrees_...` y `worktree_de_tarea_sin_publicar_...` | quitar bloqueo por sucio; quitar bloqueo por no publicado; exclude: borrar toda linea que empiece por `/` | rojas las tres |
| C6 | `config_solo_con_la_opcion` | `(de_la_2, true)` | roja |
| C7 | `rama_local_solo_con_borrar_tareas_...` | borrar la rama sin `--borrar-tareas`; aceptar cualquier nombre | rojas las dos |
| C8 | `binario_en_uso_por_otro_...` | no revertir `path = path_antes` | **VERDE (sobrevive)**; ver IMP-2 |
| C9 | `plugin_y_gestor_no_borran_el_binario` | borrar el binario en modo plugin | roja |
| C10 | `huella_de_la_016_intacta` | tratar toda config como de la 2.0; status line `taskctl` como nuestra | rojas las dos |
| C11 | `segunda_ejecucion_sin_cambios` | siempre `ok` | roja |

Cada test existe, prueba lo que dice el criterio y pasa. C2 en Windows real y C8 real (auxiliar, `Get-CimInstance`, registro) solo se ejercitan en CI: no verificado por mi.

### A mano (HOME falso, repo con remoto bare, es y en)

- `--proyecto --simular` y `--proyecto` en un repo con worktree de estado y de tarea publicado: sale 0, quita worktrees, `.taskcodetwo/`, `.git/taskcodetwo/` y solo las dos lineas propias de `exclude` (queda `/mio/`); `git ls-remote origin` igual antes y despues; segunda pasada `sin-cambios`. Con la tarea sin publicar: `rechazada` (3) sin tocar nada. Fuera de un repo: `rechazada motivo=no-es-repo` (tambien con `--maquina --proyecto`, sin hacer nada de la maquina).
- `--maquina --simular` con instalacion falsa y `settings.json` con statusLine y hook (solo simular: con `LOCALAPPDATA` falso un `--maquina` real en esta maquina podria casar la entrada real del PATH de usuario, porque `expandir` usa el valor falso): lista temporales, binario `al-salir`, carpetas, cache, statusLine `borra`, hook `conserva nombra-tc2`; `settings.json` identico byte a byte. En es y en las lineas `accion` y la final son contrato (no se traducen); stderr si se traduce (`aviso:` / `warning:`, `quitalo tu` / `remove it yourself`). Correcto.

## Reproduccion de los hallazgos

**IMP-1 — el diagnostico de «en uso» nombra el fichero equivocado.** `desinstalar()` toma `acciones.iter().find(|a| Pendiente && motivo == "en-uso")`: la primera del grupo, que es un temporal o `tc2.exe.old`, no `tc2.exe`. Repro (test temporal en el clon, fixture de `binario_en_uso...`, que ya tiene `.old` y temporales): la salida es `desinstalar rechazada ... motivo=en-uso ruta=...\bin\.tc2-2.0.0-77.descarga` y el diagnostico dice «`.tc2-2.0.0-77.descarga` esta en uso por otro proceso... cierralo». Pasa siempre que haya un `.old` (queda tras cada `actualizar --cli` en Windows) o un temporal. El test del criterio solo comprueba `contains("motivo=en-uso")`, por eso no lo ve. Correccion: reportar la accion de clase `binario` y asertar la ruta.

**IMP-2 — la reversion del PATH en «en uso» no la cubre ningun test.** Mutacion: cambiar `path = path_antes;` (maquina.rs:182) por `let _ = &path_antes;` deja **14 de 14 verdes**. Si `TC2_INSTALL_DIR` (en uso) y el directorio por defecto tienen cada uno su entrada, sin esa linea la segunda entrada se planifica con un `viejo` que ya no coincide y la aplicacion falla con «el PATH cambio entre medias». Con un test temporal de dos directorios (el de `TC2_INSTALL_DIR` en uso, ambos en el PATH) la mutacion si lo pone en rojo y el codigo actual pasa: la logica es correcta, falta el test. Es la proteccion de C8 («sin dejar nada a medias») sin red. Correccion: anadir ese caso a `binario_en_uso_por_otro_...`.

**MEN-1 — lineas `borra` que no borran.** (a) En «en uso», la carpeta raiz `%LOCALAPPDATA%\taskcodetwo` sale `decision=borra` y cuenta en `borra=2` aunque `bin` sigue ahi (`quitados` ya tenia `bin` y `dir`; el efecto `CarpetaSiVacia` no hace nada). (b) En `rechazada` por `bloquea` (mod.rs:427) las demas acciones salen `borra` y el resumen `borra=3` junto a «no se ha tocado nada» (visto con la tarea sin publicar). Es el reflejo del plan, pero el resumen miente en ambos casos. Sugerencia: no insertar en `quitados` lo del grupo en uso; en `rechazada`, marcar las demas como `pendiente motivo=no-aplicada` o documentar que `borra` es «lo que haria».

**MEN-2 — `--borrar-tareas` borra commits locales sin publicar sin avisar.** Repro: worktree de estado con un commit solo local (`91fde05`, remoto en `4a7af5a`), `--proyecto --simular --borrar-tareas taskcodetwo/tareas` imprime `rama-estado ... decision=borra` sin decir que la rama local va por delante del remoto ni dar el sha; tras `update-ref -d` el commit queda colgante y sin reflog. C7 lo permite (la confirmacion es el nombre exacto), pero RR-16 insiste en no perder historial: bastaria `motivo=adelantada-del-remoto` con el sha en la linea y una frase en instalar.md. Hoy `bloqueo()` devuelve `None` para el worktree de estado sin comprobar publicacion.

**MEN-3 — un `git worktree remove` que falla deja el plan a medias.** Un worktree de tarea con `git worktree lock` pasa `bloqueo()` (solo mira sucio y publicado) y falla al aplicar: resultado `error`, `pendiente=4`, mensaje honesto, pero no es el `rechazada sin tocar nada` de C5. Si el bloqueado va despues del de estado en `git worktree list`, el de estado ya se quito. Repro: `git worktree lock .taskcodetwo/worktrees/TASK-001`. El mensaje de git trae ademas un `;` colgando («locked working tree;.»). Sugerencia: leer `locked` del `--porcelain` y bloquear en el plan.

**MEN-4 — `formato: 2` se detecta por linea literal, no con `config::leer`.** Repro: `.taskcode/config.yml` = `---` y `formato: 2 # version` (YAML valido que la 2.0 lee), con `--proyecto --simular --config` sale `dueno=taskcode-016 decision=conserva`: nunca se borra. Es el lado seguro, pero es la misma decision escrita dos veces (el lector de la config y aqui) y divergen. Sugerencia: reutilizar `config::leer`.

**MEN-5 — instalar.md:167 tiene un TAB.** `%LOCALAPPDATA%\taskcodetwo` se escribio como `%LOCALAPPDATA%` + TAB + `askcodetwo` (`\t` interpretado); confirmado con `cat -A`. El resto de la seccion «Desinstalar» coincide con lo observado (salida, `sin-cambios`, rechazo, binario en uso, plugin y gestor, `--config`, `--borrar-tareas`). Faltan dos lineas: que `--config` y `--borrar-tareas` sin `--proyecto` dan `uso`, y que `--maquina --proyecto` juntos no hacen nada fuera de un repo. Dice que la status line se reconoce por el `command` «exactamente» y el codigo hace `trim()`.

**MEN-6 — `--simular` toca el disco en Windows (herencia de F2-12).** `main.rs` ejecuta `limpiar_old(current_exe)` antes de cualquier orden: en Windows `tc2.exe desinstalar --simular` borra su propio `tc2.exe.old`. Visto: el `.old` creado a mano desaparecio al lanzar `--simular` y no sale en el plan. Inofensivo (es basura) pero contradice «no se toca nada» (C1, instalar.md); los tests no lo ven porque llaman a la biblioteca, no a `main`. Sugerencia: no limpiar en `desinstalar`, o documentarlo. Origen `plan`: el plan no lo previo.

**MEN-7 — reescritura no atomica de perfiles y settings.** `Efecto::Reescribir` usa `std::fs::write` (trunca y escribe): un corte entre medias deja `.bashrc` o `settings.json` vacio. Guarda contra cambios desde el plan (bien), pero no escribe a un temporal y `rename`. Riesgo bajo; son ficheros de la persona.

**MEN-8 — orden `claude plugin uninstall` sin comillas.** `plugins()` interpola la clave de `installed_plugins.json` y el scope tal cual en la orden que se imprime para ejecutar. Solo importa con un fichero hostil o corrupto; `orden_borrar` si cita. Ademas `installed_plugins.json` se lee y parsea dos veces (`plugins()` y `claude()`).

**MEN-9 — separadores mezclados.** Las rutas salen como `...\repo\.taskcodetwo/worktrees` y `.../.git\taskcodetwo` (mezcla de `/` de git y `\` de `join`), tambien en `--json`. Cosmetico, pero quien compare rutas con el inventario lo notara.

## Valoracion de las decisiones fuera del plan

- **`config.yml` sin `formato: 2` nunca se borra**: de acuerdo (D6, C10); solo la forma de detectarlo (MEN-4).
- **Refs `refs/taskcodetwo/*` solo con `--borrar-tareas`**: de acuerdo; son operaciones pendientes o rechazadas, es decir tareas sin publicar. Ningun test crea una ref propia: **no verificado** empiricamente por mi (solo por lectura).
- **`push --delete` solo se imprime con `--borrar-tareas`**: de acuerdo y coherente (sin borrar la local no tiene sentido sugerirlo); cubierto por el test C7 y visto a mano en es y en.
- **Worktree bloqueado deja todo `rechazada`**: de acuerdo para sucio y sin publicar (probado a mano y por test). No cubre el `lock` de git ni un fallo de `worktree remove` (MEN-3).
- **Binario en uso deja su grupo `pendiente` y sale `rechazada` (3)**: correcto en intencion (lo demas se quita, el grupo y su entrada del PATH no se tocan); el diagnostico nombra el fichero equivocado (IMP-1) y la reversion no esta cubierta (IMP-2).
- **`--maquina` y `--proyecto` juntos**: de acuerdo, orden proyecto y luego maquina; un error del proyecto aborta todo antes de tocar nada (visto fuera de un repo), que es lo prudente.

## Lecturas fuera del contexto permitido

Para reproducir lei `crates/tc2/src/desinstalar/*`, `crates/tc2/src/instalacion.rs`, `crates/tc2/tests/desinstalar.rs`, `crates/tc2/src/main.rs`, `crates/tc2/src/git/puerto.rs` (`borrar_rama_de_estado`) y `crates/tc2/src/config/mod.rs` (como lee `formato`): el dominio asignado era solo el script y la doc, y los criterios se prueban en ese codigo.

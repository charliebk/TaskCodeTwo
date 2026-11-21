# Informe de revision — TASK-046 (ronda 1)

- Commit revisado: e33879a25a4399d5755559aa147ff96497ca231a
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

Pase generico (el diff cae en Rust y documentacion; esta revision se centra en el comportamiento del binario y en la guia `docs/ci/verificacion-reutilizable.md`, seccion F5-6). La verificacion completa de `e33879a` en verde viene de la peticion y no se repitio. Se compilo `tc2` (`cargo build -p tc2`) en un clon propio y se uso de verdad en un repo git temporal con config de formato 2 (Windows 11, `cmd`), con `LOCALAPPDATA` y `XDG_CACHE_HOME` redirigidos a una carpeta temporal.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/verificacion/ejecutar.rs:52-72 | implementacion |
| MEN-1 | MENOR | corregido | crates/tc2/src/verificacion/ejecutar.rs:356-358 | implementacion |
| MEN-2 | MENOR | corregido | crates/tc2/tests/desinstalar.rs:128-133 | implementacion |
| MEN-3 | MENOR | corregido | crates/tc2/src/verificacion/ejecutar.rs:362-399 | implementacion |
| MEN-4 | MENOR | corregido | crates/tc2/src/verificacion/ejecutar.rs:124-126, 362-390 | implementacion |
| MEN-5 | MENOR | corregido | docs/ci/verificacion-reutilizable.md (Limites) | plan |

Detalle del estado:
- IMP-1: corregido (c6ffbc1)
- MEN-1: corregido (c6ffbc1)
- MEN-2: corregido (c6ffbc1)
- MEN-3: corregido (c6ffbc1)
- MEN-4: corregido (c6ffbc1)
- MEN-5: corregido en la guia (8fd2d67)

### IMP-1 — Un error de uso con `--hook` sale con 2 y bloquea la parada, contra lo que dice la guia

La guia (tabla de `--hook`) y el plan dicen que solo el rojo de la rapida bloquea y que todo lo demas sale con 0. Pero los errores de argumentos se devuelven antes de mirar `stop_hook_active` y con el codigo de `Uso` (2), que en el evento `Stop` es justo «bloquear y pasar stderr al agente».

Reproduccion (ejecutada):

```
$ echo '{}' | tc2 verificar --perfil nope --hook ; echo rc=$?
verificar uso motivo=argumentos
uso: verificar [--json] --perfil <rapida|impacto|completa|pesada> [--hook]
rc=2
$ echo '{}' | tc2 verificar --perfil rapida --hook --foo ; echo rc=$?   -> rc=2
$ echo '{"stop_hook_active":true}' | tc2 verificar --perfil nope --hook ; echo rc=$?   -> rc=2 (el guardia de reentrada no llega a leerse)
```

Efecto real: una errata en la entrada de `settings.json` (p. ej. `--perfil rapid`, o una version futura con un perfil renombrado) hace que Claude Code no pueda parar nunca, y como el argumento se valida antes que `stop_hook_active`, ni siquiera lo corta la reentrada. Es el unico camino de `--hook` que bloquea sin que sea un rojo, y la tabla de la guia no lo menciona. Arreglo: con `--hook` presente (detectado antes de validar el resto) devolver 0 con una linea en stderr que diga que el hook esta mal escrito y como arreglarlo, y comprobar `stop_hook_active` antes. Anadir un test (hoy P6/P7 no lo cubren).

### MEN-1 — `es_rojo` acepta `timeout` y nadie lo ejercita bajo `--hook` (mutante superviviente)

Mutacion: `Some("rojo" | "timeout")` -> `Some("rojo")` en `es_rojo`: los 7 tests de `verificar` siguen verdes. Solo es alcanzable con `--hook --perfil completa|pesada|impacto` (el hook documentado fija `rapida`, donde el timeout es `excedido`), asi que es poco probable en uso real, pero esa rama no tiene red. O se acota `--hook` a `rapida` (D6 dice que el hook la fija; hoy acepta cualquier perfil y bloquea con un timeout de `completa`) o se anade el test. Mismo grupo: la mutacion que sustituye la cola de 15 lineas por la salida entera tambien sobrevive (el test de rojo comprueba que aparece el texto del fallo, no que se recorta).

### MEN-2 — La prueba de `desinstalar --maquina` solo cubre la fixture Unix

`instalacion_windows` no siembra `taskcodetwo/confianza`; solo la fixture Unix (`.cache/taskcodetwo/confianza`) lo hace. La mutacion `if confianza.is_dir()` -> `if false` solo la mata `maquina_unix_quita_lo_suyo_y_deja_el_perfil_igual`; el test de Windows pasa (18 pasan, 1 falla). El codigo es comun y la prueba manual en Windows borro el almacen (ver «Lo comprobado»), asi que no hay bug; pero la plataforma donde la guia mide y documenta (`%LOCALAPPDATA%`) no tiene su propia asercion.

### MEN-3 — El aviso de `--hook` por configuracion invalida ocupa varias lineas

La guia dice «una linea en stderr». Con un YAML invalido el diagnostico trae el volcado multilinea del parser (visto: 10 lineas en stderr). Sigue diciendo el arreglo («Arreglo: corrige el YAML de .taskcode/config.yml») y sale con 0, asi que no bloquea; es solo la discrepancia con la guia y ruido para el agente.

### MEN-4 — Salida de la rapida en `--hook`: fichero temporal sin cota

Con `--hook` la salida del comando se captura en `%TEMP%\tc2-verificar-<pid>.log` (nombre predecible, sin cota de tamano) y se lee entera (`std::fs::read`) para quedarse con 15 lineas. Si Claude Code mata el hook por su propio `timeout` o el proceso cae, el fichero queda (y puede contener secretos de la salida del comando). Sugerencia: leer solo la cola y crear el fichero con nombre aleatorio y exclusivo. No reproducido como fallo; es de robustez.

### MEN-5 — Limite no declarado: scripts no versionados quedan fuera del cierre

El cierre sale de `git ls-files`; un script que el cierre llama y que esta sin versionar (o ignorado) se ejecuta sin entrar en la huella. La guia declara los limites de herramientas/modulos, enlaces simbolicos y TOCTOU, pero no este. El modelo de amenaza (repo ajeno) lo hace de bajo riesgo y es coherente con «scripts versionados», pero conviene decirlo junto a los demas limites. Derivado de leer el codigo, no reproducido.

## Lo comprobado (reproduccion ejecutada)

Sobre un repo temporal (`scripts\verificar.bat` que escribe `%TC2_PERFIL%` en un fichero testigo y llama a `lib\paso.bat` por `%~dp0`) con `formato: 2` y `rapida: {timeout: 20}`:

- Sin confianza: `tc2 verificar --perfil rapida` -> codigo 3, `motivo=sin-confianza confianza=nunca`, mensaje «revisa la orden y los scripts que lanza y, si te fias, ejecuta `tc2 confiar verificacion.comando`»; el testigo NO existe. Con `--hook` y `{}`: codigo 0, mismo consejo en stderr, nada se ejecuta.
- `tc2 confiar verificacion.comando` con stdin en `/dev/null` (sin TTY): `confiar ok ... ficheros=2 hasheados=<n>`, codigo 0. Divergencia 8 (no exigir TTY) coherente con que la confianza es un acto explicito de la persona y con que tests y CI la usan sin terminal; sin hallazgo.
- Confiado: `resultado=verde hasheados=0`, el testigo recibe `rapida`. Cambiar `lib\paso.bat` -> `confianza=cambiado hasheados=18`, codigo 3, no se ejecuta, y el mensaje distingue «cambio desde que confiaste».
- Rojo (`exit /b 5`): sin hook codigo 3 `motivo=rojo codigo=5`, la salida del comando va a stderr; con `--hook` codigo 2, stderr con el motivo y las ultimas lineas (`FALLO-X`). `completa` en rojo: 3.
- `stop_hook_active:true` -> codigo 0, `sin-cambios motivo=stop-hook-activo`, aunque la rapida este en rojo; con stdin vacio o basura se sigue (como dice el codigo).
- `rapida` con `timeout: 2` y comando de 30 s: `resultado=excedido segundos=2`, codigo 0 con y sin `--hook`; `tasklist` no deja el `ping` hijo (se mata el arbol). Perfil no declarado (`impacto`): codigo 3 sin hook, 0 con aviso con `--hook`.
- `--json` produce un objeto con `claves` coherente con la linea de texto. Fuera de un repo con `--hook`: 0 y aviso.
- `tc2 desinstalar --maquina` (con `LOCALAPPDATA` redirigido): lista `confianza` y la carpeta `taskcodetwo` como `decision=borra`, borra el almacen y el repo vuelve a `confianza=nunca`. Ojo: sin `--simular` borra de verdad (esperado).
- La entrada de `settings.json` de la guia es JSON valido y coherente con el CLI (`Stop`, sin `matcher`, `command: "tc2 verificar --perfil rapida --hook"`); no se probo dentro de un Claude Code real (no verificado), solo el comando con el JSON por stdin que describe.

Codigos de salida (divergencia 9): coherentes con la guia (0 verde/excedido/no-ejecuta bajo hook; 3 rechazada sin hook; 2 solo rojo bajo hook) salvo IMP-1, donde el 2 de «uso» se cuela bajo `--hook`. Cada mensaje dice que hacer.

## Mutacion (sobre `cargo test -p tc2 --test verificar`, salvo la ultima)

| Mutacion | Resultado |
|---|---|
| dejar pasar `Nunca` sin confianza | 3 tests rojos (matada) |
| `stop_hook_activo` -> siempre false | rojo (matada) |
| quitar excepcion de timeout en `rapida` | rojo (matada) |
| `LINEAS_RESUMEN` = 0 | rojo (matada) |
| quitar resolucion de `%~dp0` | 2 tests rojos (matada) |
| ignorar globs de `confianza_incluye` | rojo (matada) |
| ignorar la cache `(tamano, mtime)` / ignorar `mtime` | rojo (matada) |
| id del almacen constante (en vez del dir comun) | rojo, «otro clon» (matada) |
| `desinstalar --maquina` sin borrar `confianza` (`--test desinstalar`) | rojo solo en el test Unix (MEN-2) |
| `es_rojo` sin `timeout` | sobrevive (MEN-1) |
| cola de 15 lineas -> salida entera | sobrevive (MEN-1) |

## Divergencias declaradas

1-8 y 10-11 aceptables tal como estan anotadas. La 9 (3 para rechazada, 2 en hook) es correcta salvo IMP-1. La 8 (solo `verificacion.comando`) es honesta y el error de uso lo dice («se pueden confiar: verificacion.comando»). La 4 (mayusculas) no se pudo probar en Linux/macOS (no verificado).

## Lecturas fuera del contexto permitido

- `crates/tc2/src/verificacion/ejecutar.rs`, `crates/tc2/src/confianza/{mod,cierre}.rs`, `crates/tc2/src/desinstalar/maquina.rs`, `crates/tc2/tests/verificar.rs` y `desinstalar.rs`: para localizar la linea que protege cada criterio y construir las mutaciones. No se leyo nada mas.

# Informe de revisión — TASK-082 (ronda 1)

- Commit revisado: c574e8d
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| H1 | MENOR | abierto | crates/tc2/src/solape.rs:418 | implementacion |
| H2 | MENOR | abierto | crates/tc2/src/git/diff.rs:96 | implementacion |
| H3 | MENOR | abierto | crates/tc2/src/solape.rs:406 | implementacion |

**H1.** La rama `Err` de `incompletas` (clave `solape-incompleto`) no tiene test: `Err(_) => {}` deja `--test solape` 10/10 y los unitarios en verde. La conducta es correcta (rama huerfana sin merge-base: `start ok solape-incompleto=TASK-003`). Falta un test que fuerce el `diff` fallido.

**H2.** `--no-renames` y `-z` solo los protege la cadena literal de C10: quitar `--no-renames` solo hace caer C10; ningun test de comportamiento usa renombrados, borrados, binarios ni rutas raras (con renombrados se perderia el nombre viejo, F1-15 S08). Comportamiento actual correcto (renombrado a `nuevo dir/ñandú, 100%.txt`, borrado y binario: salen los dos nombres; la linea escapa bien). En stderr las rutas con coma se separan con ", " sin escape (solo cosmetico). Anadir un test con renombrado y borrado.

**H3.** Sin tope al numero de `git diff` ni al largo de stderr con muchas tareas en curso: con 25, `calcular` lanza 1 + 25 + 1 procesos en serie; la linea se corta a 20 (`solape-truncado=si`), pero `avisos()` emite 25 lineas `aviso:` y la nota de C9 (`tareas.join("; ")`, `solape.rs:538`) no corta el numero de tareas.

## Reproduccion

Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-082-r1-rust` (borrado). `fmt --check` limpio (clippy no, por la carga). `--lib solape` 7/7, `--test solape` 10/10, toma 11, liberar 12, estimacion 13. Bordes con un test propio desechado: renombrado, borrado, binario, rutas con espacio, unicode, coma y `%`; base o rama inexistente; rama huerfana; 25 tareas. Mutaciones: quitar el filtro `!ya.contains(*m)` en `cruzar` → rojo; quitar `--no-renames` → solo C10 (H2); `Err(_) => {}` → verde (H1).

## Revisado sin hallazgos

El aviso nunca cambia resultado ni codigo de `start` (`con_solape` solo en exito; `con_aviso_de_solape` solo anade a stderr con codigo 0; fallos de lectura dejan `None` o `incompletas`; C4 verde). Sin `unwrap`/`expect`/`panic` ni indexado sobre entrada en produccion. Solo `for-each-ref` y `diff` (C5, C10). Saneado en stderr cubierto por `avisos_saneados`. Nota: el solape no lanza `cat-file` propio (reutiliza la lectura de la toma). Limite JS-WASM: no aplica.

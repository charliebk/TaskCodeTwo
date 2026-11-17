# Informe de revisión — TASK-033 (ronda 1)

- Commit revisado: ded3359
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

Nota de quien orquesta: el revisor escribio «aprobada (1 IMPORTANTE de red de
regresion y 3 MENORES, ninguno bloquea)». Se registra como
`cambios-solicitados` porque un IMPORTANTE abierto no puede cerrarse sin
corregir (regla del proyecto); el texto de los hallazgos es el del revisor.

## Puerta determinista

- `cargo test -p tc2`: sin FAILED; `--test toma` 11/11; `--lib toma` 8/8.
- `cargo fmt --check` y `clippy --all-targets -D warnings` limpios.
- Mutaciones que se ponen rojas: C1 (ignorar el rechazo de la transaccion), C2 (`en-revision` no cuenta como empezada), C4 (dependencias no bloquean), C6 (quitar `comprobar_id`).
- C3 reproducido a mano: repo sin origen da `rechazada motivo=sin-rama-de-estado` (3); origen caido da `sinred pendiente=no` (4).
- F1-2 D4 cumplida: `Destino::crear` va despues de `estado::transaccion`. F1-3: base `origin/develop`, hotfix desde el ultimo tag de `origin/main`, sin upstream, worktree en `.taskcodetwo/worktrees/TASK-NNN`, `release` rechazada.

## Hallazgos

### IMPORTANTE

**I1. La proteccion «el tag del hotfix tiene que estar en `origin/main`» no tiene test que se ponga rojo al quitarla.** `crates/tc2/src/toma/mod.rs`, `base_de`, `"--merged", MAIN` de `for-each-ref`. Quitarlo deja `--test toma` 11/11 en verde: `d4_...` solo crea tags dentro de `main`. Arreglo: test con un tag mas nuevo fuera de `main`; debe ganar el ultimo tag de `main`.

### MENOR

**M1.** `Destino::crear_paso`: cambiar `&& !nueva` por `&& true` (saldria `creado=` en la primera toma) sobrevive; anadir en `c7` `assert!(valor(&out, "creado").is_none())`.

**M2.** `rama-invalida` (`toma/mod.rs`, lineas 142-148) sin E2E; `estado-no-valido`, `asignado-heredado` e `ilegible` solo con unitarios. Arreglo: E2E con `rama:` mal formada esperando 3 `motivo=rama-invalida`.

**M3.** Casos limite aceptados a anotar en `## Resultado`: dependencia inexistente no bloquea (regla de `next`, sin mencion en ADR); tarea mia planificada con rama propia en el remoto da `ocupada`; al reanudar se valida igualmente la base y `tipo-release`; `Destino::crear` reconstruye el `Informe` con `std::mem::take` (algo enrevesado).

## Sin hallazgo

Sin duplicacion (regla de dependencias y deteccion de plataforma extraidas y usadas por `tablero`); entradas externas validadas (`tarea.md` dependencias como ID, `rama:` por `partes_de_rama`); i18n es/en completo; `--json` de una linea tambien en rechazos; `RefCell` reiniciados por intento.

## Limpieza

Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\` borrado; repo real sin tocar.

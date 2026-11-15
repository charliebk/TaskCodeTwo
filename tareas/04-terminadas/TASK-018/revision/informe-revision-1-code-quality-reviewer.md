# Informe de revision — TASK-018 (ronda 1)

- Commit revisado: 4f80989fc93337d12966d31c9be87260371a8ddd
- Revisor: code-quality-reviewer (agente general-purpose, sonnet)
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido | crates/tc2/src/estado/transaccion.rs (`intento`, `adelantar`) | implementacion |
| MENOR-1 | MENOR | aceptado | crates/tc2/src/estado/transaccion.rs (`adelantar`, `Tareas::Diverge`) | implementacion |
| MENOR-2 | MENOR | corregido | crates/tc2/tests/estado_casos.rs (y ADR F1-2 / F1-4) | implementacion |
| MENOR-3 | MENOR | aceptado | crates/tc2/src/estado/lectura.rs (`Estado::leer`) | especificacion |
| MENOR-4 | MENOR | corregido | crates/tc2/src/git/remoto.rs (`clasificar_fallo_remoto`) | implementacion |

### IMPORTANTE-1. Rama de estado creada sin remoto y publicada después: `ok` con una rama publicada sin lo anterior; la local diverge para siempre

Con remoto, `intento` toma como base `origin/taskcode/tareas`; si el remoto no tiene la rama (`RamaInexistente`) la base es `None` y la operación se aplica sobre un estado vacío, sin mirar la rama local. Pasa en un repo montado sin remoto (RN-06) al que luego se añade `origin`: se publica una raíz nueva sin `LEEME.md` ni lo escrito antes, `adelantar` da `Diverge` y el informe sale `ok`.

```
T1 b: estado ok operacion=anotar commit=6bd0cb741a99 intentos=1 tareas=diverge
T1 remoto tiene: "notas/b.md\n"
T1 local tiene: "LEEME.md\nnotas/a.md\n"
T1 c: estado ok operacion=anotar commit=1b0238f05b31 intentos=1 tareas=diverge
```

Propuesta: si el remoto no tiene la rama y existe la local, tomar la local como base (o rechazar con motivo claro). Test de «remoto añadido después».

### MENOR-1. `tareas=diverge` sale con `ok` (código 0), sin diagnóstico ni salida

Un commit manual en `tareas/` queda varado sin aviso; cada transacción vuelve a divergir.

```
T4 0: estado ok operacion=anotar commit=24ec006b1795 intentos=1 tareas=diverge
T4 1: estado ok operacion=anotar commit=096beca43aa6 intentos=1 tareas=diverge
T4 remoto: "LEEME.md\nnotas/0.md\nnotas/1.md\n"   (manual.md no está)
```

Propuesta: diagnóstico en `tareas=diverge` y `tareas=con-cambios`, y decidir en el ADR si es `ok`.

### MENOR-2. Caminos de error sin test que se ponga rojo; la adenda afirma comportamientos no probados

Sobreviven: M3 (tope del cerrojo, `agotada motivo=cerrojo`), M18 (`fetch` denegada en `transaccion` y `montar`), M19/M20 (`Diverge`, `ConCambios`), M15 (numeración que no reutiliza una rechazada), M21 (borrado de la carpeta temporal; la carpeta padre `taskcode-estado-tmp` queda). `tag-anotado` y `actualizar-rama` con `denegada` verificados solo a mano. Sin test: `anotacion-invalida`, `sin-intentos`, `espera()`. Reproducidos a mano: se comportan bien.

Propuesta: un test por camino marcado (los dos primeros y `Diverge` son los que más importan).

### MENOR-3. `Estado::leer` con conversión lossy: un fichero no UTF-8 se corrompe al reescribirlo

```
T6: estado ok operacion=reescribe ...
T6 blob antes=6f83395d... despues=d758f37e... cambio=true
```

Propuesta: rechazar con `no-utf8` o trabajar con bytes; si se acepta como límite, decirlo en el ADR.

### MENOR-4. `clasificar_fallo_remoto`: «permission denied» y la contención, amplios

`permission denied` también casa con fallos del sistema de ficheros local (saldría `denegada motivo=credenciales`); `failed to update ref` en `[remote rejected]` permanente se reintentaría hasta `agotada` con un texto no cierto. No verificado contra servidor real.

Propuesta: aceptarlo y anotarlo, o acotar los patrones.

## Estado tras la ronda 1 (lo anota quien implementa)

- IMPORTANTE-1 corregido: con el remoto sin la rama, la base es la rama local (`transaccion.rs`); test `estado_bordes::remoto_anadido_despues_publica_la_historia_local`, rojo con el mutante que lo deshace.
- MENOR-1 aceptado: lo escrito está publicado; `tareas=diverge|con-cambios` en la línea y avisar de forma visible es de `status` (F3-5b). Motivo en la adenda de F1-2. Ahora con test (`estado_bordes::tareas_con_cambios_o_divergente_lo_dice_y_no_pierde_nada`).
- MENOR-2 corregido: tests de cerrojo ocupado (transacción y montar), `fetch` denegado en transacción y montar, `con-cambios`/`diverge`/`tareas-con-cambios`, numeración que no reutiliza una rechazada, temporales borrados (también la carpeta padre), `tag-anotado` y `actualizar-rama` con `denegada`, `anotacion-invalida`, `sin-intentos` y `espera()`.
- MENOR-3 aceptado: límite UTF-8 escrito en la adenda de F1-2 (el formato de tarea de F1-9 es UTF-8).
- MENOR-4 corregido en lo de credenciales (`permission denied` solo con «could not read from remote repository» o `publickey`; test `permiso_local_no_es_de_credenciales`); la contención por texto, aceptada y anotada en la adenda.

## Verificación ejecutada

- Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-018-r1-calidad` (commit 4f80989), `CARGO_TARGET_DIR` dentro; borrado al terminar.
- `cargo test -p tc2`: 33 passed (11 suites, 252 s). `cargo fmt --check` y `cargo clippy -D warnings` limpios.
- Mutantes ROJOS: M1 (`es_contencion` desactivada, `dos_clones`), M2 (cerrojo sin `try_lock`), M4, M5, M6, M8, M9, M10, M11, M14, M23, M24, M25.
- Mutantes que SOBREVIVEN: M3, M15, M18, M19, M20, M21.
- Adendas F1-1, F1-2, F1-4 contrastadas: ciertas; la fila del cerrojo solo se verifica sin remoto; el tope de 60 s no está probado.

## Lecturas fuera del contexto permitido

El código completo de `crates/tc2/src/estado/`, `git/estado.rs`, `git/remoto.rs`, `git/puerto.rs` y los tests `estado_*.rs` y `tests/comun/*` (la petición solo trae el diff de los ADR; sin el código no se puede reproducir ni mutar), y el diff de `git_push.rs`, `git_casos_limite.rs` y `tag.rs` para el cambio de comportamiento de F2-2.

# Informe de revision — TASK-022 (ronda 2)

- Commit revisado: 7b4697f67ade1f22276dd31d65c166ba74f6224d
- Revisor: code-quality-reviewer (verificacion acotada de la ronda 2)
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/plataforma/gitlab.rs:565 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

## Verificacion

- Commit: el HEAD es 8cf878c, pero entre 7b4697f y 8cf878c solo cambian ficheros de `tareas/` y de revision; el codigo de `crates/` es identico al de 7b4697f. Se ha verificado ese codigo.
- Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-022-r2-calidad`, rama `feature/task-022-f2-9-adaptador-gitlab`, con `CARGO_TARGET_DIR` dentro del clon. El clon se borra al terminar.
- Suite `cargo test -p tc2` ejecutada sobre el clon: todos los binarios en verde (41 unitarios; alta 3; cli 2; estado_bordes 9; estado_casos 4; estado_concurrencia 3; git_* 1+5+1+1+1; plataforma 4; plataforma_gitlab 3; E2E ignorados 2+2 sin ejecutar). Cero fallos.
- Una primera ejecucion de `plataforma_gitlab` dio dos fallos con `sinred` (conexion local a 127.0.0.1 atascada). Repetida en solitario, pasa 3/3. No es hallazgo; es el atasco intermitente indicado en la peticion.

### IMP-1 — `estado-puertas` ok con `detailed_merge_status` en `checking` (verificado: corregido)

- Donde: crates/tc2/src/plataforma/gitlab.rs:562-569 (`sin_conflictos`).
- Verificacion: el codigo ahora trata `detailed_merge_status` `checking`, `unchecked`, `preparing` y `conflict` como `sin_conflictos = false`, con `falta=sin-conflictos` y `esperando`. El test `protecciones_de_gitlab` añade el caso `checking` (linea 647).
- Mutante: se quita la condicion `matches!(...)` dejando `sin_conflictos: !m.has_conflicts`. Resultado: `protecciones_de_gitlab` FALLA (`left: "estado-puertas ok pr=12 aprobaciones=1 ignoradas=0"`, `right: "estado-puertas esperando pr=12 aprobaciones=1 ignoradas=0 falta=sin-conflictos"`). El mutante queda muerto. Tras la prueba se restauro el fichero del clon (git checkout) y se comprobo que el arbol estaba limpio.
- Estado: corregido. El caso real de GitLab (que la API devuelva `checking` justo tras abrir el MR) no se ha ejecutado contra GitLab; se verifica con el fixture.

### Busqueda de fallos que abre la correccion (sin hallazgos nuevos CRITICOS ni introducidos)

Revisado el diff de la ronda 2 (G01 con `user_id`/`group_id`, 401 en `merge-pr`, `Clave` SSH/GPG, `all=true` en statuses, `detailed_merge_status`):

- `Clave`: SSH y GPG ya no comparten espacio de ids. Tests `protecciones_de_gitlab` cubren firma SSH verificada, PGP con id colisionante (no cuenta como aprobacion) y firma sin clave. Sin fallo.
- `detailed_merge_status` ausente (`None`): cae en el comportamiento anterior. Sin fallo.
- `estado-puertas` con `conflict` devuelve `esperando ... falta=sin-conflictos`, igual que antes con `has_conflicts == true`: no es regresion. El texto de `puertas.rs:85` ya indica lanzar `actualizar-rama`.

## Notas (fuera de la tabla, no abren ronda)

- El 401 en `merge-pr` se diagnostica como `motivo=permisos` (gitlab.rs:624-633). Un token expirado entre el `GET` del MR y el `PUT` daria el mismo diagnostico. Coherente con el mitigante de MEN-1 de la ronda 1; no se considera hallazgo.
- Los tests de G01 cubren `user_id` y el nivel 30, pero no `group_id`. El codigo lo comprueba. Anotado como cobertura pendiente, no bloquea.
- Los MENORES de la ronda 1 (MEN-1..MEN-4) no se han revisado de nuevo, como pide la peticion.

## Lecturas fuera del contexto permitido

- crates/tc2/src/plataforma/errores.rs (funcion `mensaje_api` y `sanear`, para comprobar que el 401 no filtra el cuerpo sin sanear).
- crates/tc2/src/plataforma/puertas.rs (linea 85, texto del diagnostico de `sin-conflictos`).
- Lectura de crates/tc2/src/plataforma/conexion.rs solo por la llamada a `traducir`.
